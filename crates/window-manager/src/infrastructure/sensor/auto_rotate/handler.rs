//! Sensor discovery and transform calculation helpers.

use super::models::{AutoRotate, Sensor};
use super::*;

pub(super) fn baseline_transform(data: &CalloopData) -> Transform {
    data.state
        .windows
        .space
        .outputs()
        .next()
        .map(|output| output.current_transform())
        .unwrap_or(Transform::Normal)
}

/// Rotates a transform clockwise by `steps` quarter turns.
pub(super) fn rotate(transform: Transform, steps: u8) -> Transform {
    let mut result = transform;
    for _ in 0..steps % 4 {
        result = match result {
            Transform::Normal => Transform::_90,
            Transform::_90 => Transform::_180,
            Transform::_180 => Transform::_270,
            Transform::_270 => Transform::Normal,
            Transform::Flipped => Transform::Flipped90,
            Transform::Flipped90 => Transform::Flipped180,
            Transform::Flipped180 => Transform::Flipped270,
            Transform::Flipped270 => Transform::Flipped,
        };
    }
    result
}

pub(super) fn transform_name(transform: Transform) -> &'static str {
    match transform {
        Transform::Normal => "normal",
        Transform::_90 => "90",
        Transform::_180 => "180",
        Transform::_270 => "270",
        Transform::Flipped => "flipped",
        Transform::Flipped90 => "flipped-90",
        Transform::Flipped180 => "flipped-180",
        Transform::Flipped270 => "flipped-270",
    }
}

pub(super) fn read_number(path: &Path) -> Option<f64> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<f64>()
        .ok()
}

fn find_iio_accelerometer() -> Option<Sensor> {
    for root in ["/sys/bus/iio/devices", "/sys/class/iio"] {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("in_accel_x_raw").is_file() && path.join("in_accel_y_raw").is_file() {
                let scale = read_number(&path.join("in_accel_scale")).unwrap_or(1.0);
                return Some(Sensor::Iio { path, scale });
            }
        }
    }
    None
}

fn sensor_proxy() -> Option<Sensor> {
    let connection = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
    // Calling the proxy directly also activates it over D-Bus; `NameHasOwner`
    // would report "false" for an installed-but-not-yet-started service.
    if connection
        .call_sync(
            Some(PROXY_NAME),
            PROXY_PATH,
            PROXY_IFACE,
            "ClaimAccelerometer",
            None,
            None,
            gio::DBusCallFlags::NONE,
            1000,
            gio::Cancellable::NONE,
        )
        .is_err()
    {
        return None;
    }

    let available = connection
        .call_sync(
            Some(PROXY_NAME),
            PROXY_PATH,
            "org.freedesktop.DBus.Properties",
            "Get",
            Some(&Variant::from((PROXY_IFACE, "Accelerate"))),
            None,
            gio::DBusCallFlags::NONE,
            1000,
            gio::Cancellable::NONE,
        )
        .is_ok();
    if available {
        Some(Sensor::Proxy(connection))
    } else {
        None
    }
}

fn detect_sensor() -> Option<Sensor> {
    find_iio_accelerometer().or_else(sensor_proxy)
}

/// Starts automatic rotation when enabled. A sensor being present is enough to
/// enable it by default; `auto_rotate = false` in display.toml or
/// `BABYDRADE_AUTO_ROTATE=0` opts out.
pub(crate) fn spawn(event_loop: &mut EventLoop<'_, CalloopData>) {
    let enabled = match std::env::var("BABYDRADE_AUTO_ROTATE").as_deref() {
        Ok("0") | Ok("false") => false,
        Ok(_) => true,
        Err(_) => services::config::load_display_config()
            .auto_rotate
            .unwrap_or(true),
    };
    if !enabled {
        tracing::info!("auto-rotate disabled by configuration");
        return;
    }

    let Some(sensor) = detect_sensor() else {
        tracing::info!("auto-rotate: no accelerometer found (IIO or iio-sensor-proxy)");
        return;
    };

    let mut controller = AutoRotate::new(sensor);
    tracing::info!("auto-rotate enabled using {:?}", controller.sensor);
    if let Err(err) = event_loop.handle().insert_source(
        Timer::from_duration(POLL_INTERVAL),
        move |_, _, data: &mut CalloopData| {
            controller.poll(data);
            TimeoutAction::ToDuration(POLL_INTERVAL)
        },
    ) {
        tracing::warn!("cannot start auto-rotate timer: {err}");
    }
}
