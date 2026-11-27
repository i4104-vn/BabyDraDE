use std::sync::atomic::{AtomicUsize, Ordering};

use super::{DriverFactory, DriverSelection, Gravity, RotationDriver};

struct FakeDriver {
    name: &'static str,
    value: Option<Gravity>,
}

impl RotationDriver for FakeDriver {
    fn name(&self) -> &str {
        self.name
    }
    fn read(&mut self) -> Option<Gravity> {
        self.value
    }
}

fn down() -> Option<Box<dyn RotationDriver>> {
    Some(Box::new(FakeDriver {
        name: "iio",
        value: Some(Gravity { x: 0.0, y: 1.0 }),
    }))
}

fn unavailable() -> Option<Box<dyn RotationDriver>> {
    None
}

fn proxy() -> Option<Box<dyn RotationDriver>> {
    Some(Box::new(FakeDriver {
        name: "sensor-proxy",
        value: Some(Gravity { x: 1.0, y: 0.0 }),
    }))
}

fn failing() -> Option<Box<dyn RotationDriver>> {
    Some(Box::new(FakeDriver {
        name: "iio",
        value: None,
    }))
}

static PROBES: AtomicUsize = AtomicUsize::new(0);

/// Counts probes without capturing state, since probe functions are plain
/// function pointers in the driver registry.
fn counting_unavailable() -> Option<Box<dyn RotationDriver>> {
    PROBES.fetch_add(1, Ordering::SeqCst);
    None
}

#[test]
fn auto_selection_falls_back_when_primary_probe_fails() {
    let factory = DriverFactory::default()
        .register("iio", &[], unavailable)
        .register("sensor-proxy", &[], proxy);
    let manager = factory.create(DriverSelection::Auto);
    assert_eq!(manager.name(), "sensor-proxy");
}

#[test]
fn named_selection_uses_only_the_matching_driver() {
    let factory = DriverFactory::default()
        .register("iio", &[], down)
        .register("sensor-proxy", &["proxy"], proxy);
    let manager = factory.create(DriverSelection::Named("PROXY".into()));
    assert_eq!(manager.name(), "sensor-proxy");
}

#[test]
fn unknown_selection_falls_back_to_auto_detection() {
    let factory = DriverFactory::default()
        .register("iio", &[], down)
        .register("sensor-proxy", &[], proxy);
    let manager = factory.create(DriverSelection::Named("bogus".into()));
    assert_eq!(manager.name(), "iio");
}

#[test]
fn retries_unavailable_drivers_with_backoff() {
    PROBES.store(0, Ordering::SeqCst);
    let factory = DriverFactory::default().register("iio", &[], counting_unavailable);
    let mut manager = factory.create(DriverSelection::Auto);
    assert_eq!(PROBES.load(Ordering::SeqCst), 1);
    for _ in 0..2 {
        let _ = manager.read();
    }
    assert_eq!(PROBES.load(Ordering::SeqCst), 1);
    let _ = manager.read();
    assert_eq!(PROBES.load(Ordering::SeqCst), 2);
}

#[test]
fn switches_driver_after_repeated_read_failures() {
    let factory = DriverFactory::default()
        .register("iio", &[], failing)
        .register("sensor-proxy", &[], proxy);
    let mut manager = factory.create(DriverSelection::Auto);
    for _ in 0..3 {
        let _ = manager.read();
    }
    assert_eq!(manager.name(), "sensor-proxy");
}
