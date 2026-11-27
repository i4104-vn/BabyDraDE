//! Background accelerometer sampler.
//!
//! Sensor reads perform blocking I/O: D-Bus round-trips to `iio-sensor-proxy`
//! and sysfs reads. Running them on a compositor's or greeter's event loop
//! stalls input, rendering and every other source for the whole duration of a
//! read, which is perceived as severe system-wide lag. This worker owns the
//! driver and polls it on a dedicated thread; consumers only drain finished
//! samples and never touch the hardware on their main loop.

use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::{DriverManager, Gravity, RotationDriver};

/// How often the main loop checks for a fresh sample.
///
/// Reads happen on the worker thread at the driver's own interval, so this
/// value only bounds how quickly a finished reading reaches the UI.
const DRAIN_INTERVAL: Duration = Duration::from_millis(100);

/// Polls a rotation driver off-thread and publishes its readings.
#[derive(Debug)]
pub struct RotationWorker {
    name: String,
    samples: Receiver<Gravity>,
    _thread: Option<JoinHandle<()>>,
}

impl RotationWorker {
    /// Starts sampling `driver` on a dedicated thread.
    ///
    /// # Arguments
    /// * `driver` - Resolved driver manager to read gravity samples from.
    pub fn spawn(mut driver: DriverManager) -> Self {
        let name = driver.name().to_owned();
        let interval = driver.poll_interval();
        let (sender, samples) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("babydra-rotation".into())
            .spawn(move || loop {
                if let Some(gravity) = driver.read() {
                    if sender.send(gravity).is_err() {
                        break;
                    }
                }
                thread::sleep(interval);
            });
        Self {
            name,
            samples,
            _thread: thread.ok(),
        }
    }

    /// Returns the newest pending sample without blocking.
    ///
    /// Older queued samples are discarded so a busy main loop never replays a
    /// stale orientation after the device has moved again.
    pub fn try_read(&mut self) -> Option<Gravity> {
        let mut latest = None;
        while let Ok(gravity) = self.samples.try_recv() {
            latest = Some(gravity);
        }
        latest
    }
}

impl RotationDriver for RotationWorker {
    fn name(&self) -> &str {
        &self.name
    }

    fn read(&mut self) -> Option<Gravity> {
        self.try_read()
    }

    fn poll_interval(&self) -> Duration {
        DRAIN_INTERVAL
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::RotationWorker;
    use crate::sensors::Gravity;

    fn worker_with(samples: Vec<Gravity>) -> RotationWorker {
        let (sender, receiver) = mpsc::channel();
        for sample in samples {
            sender.send(sample).expect("send sample");
        }
        RotationWorker {
            name: "fake".into(),
            samples: receiver,
            _thread: None,
        }
    }

    #[test]
    fn try_read_discards_stale_samples_and_keeps_the_newest() {
        let mut worker = worker_with(vec![Gravity { x: 0.0, y: 1.0 }, Gravity { x: 1.0, y: 0.0 }]);
        assert_eq!(worker.try_read(), Some(Gravity { x: 1.0, y: 0.0 }));
    }

    #[test]
    fn try_read_returns_none_when_no_sample_is_pending() {
        let mut worker = worker_with(Vec::new());
        assert_eq!(worker.try_read(), None);
    }
}
