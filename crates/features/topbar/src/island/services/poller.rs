//! One background-polling helper shared by every system indicator feature.
//!
//! Instead of each feature writing its own thread/channel/glib task trio, they
//! all call `spawn_poller` with a reader and a callback. The reader runs on a
//! worker thread; the callback always runs on the GTK main context.

use std::time::Duration;

/// Polls `read` every `interval` and delivers the latest value to `on_change`
/// on the main context. The worker exits when the channel receiver is dropped.
/// The channel holds at most one pending sample so a stalled main loop cannot
/// grow the queue without bound.
pub fn spawn_poller<T, R, F>(name: &'static str, interval: Duration, read: R, on_change: F)
where
    T: Send + 'static,
    R: Fn() -> T + Send + 'static,
    F: Fn(T) + 'static,
{
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<T>(1);

    glib::MainContext::default().spawn_local(async move {
        while let Some(value) = receiver.recv().await {
            // Coalesce samples that arrived while this callback was pending.
            let mut latest = value;
            while let Ok(newer) = receiver.try_recv() {
                latest = newer;
            }
            on_change(latest);
        }
    });

    std::thread::Builder::new()
        .name(name.into())
        .spawn(move || loop {
            std::thread::sleep(interval);
            if sender.try_send(read()).is_err() {
                // Full means a sample is already pending; drop this one rather
                // than queueing indefinitely. A closed receiver ends the worker.
                if sender.is_closed() {
                    break;
                }
            }
        })
        .ok();
}
