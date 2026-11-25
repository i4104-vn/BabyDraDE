//! Data source for the brightness indicator (read by `crate::center::poller`).

use services::system::hardware::BacklightReader;

/// Reads the current backlight level as a 0..100 percentage.
pub fn read() -> f64 {
    let mut reader = BacklightReader::new();
    reader
        .read()
        .map(|info| f64::from(info.percent))
        .unwrap_or(0.0)
}
