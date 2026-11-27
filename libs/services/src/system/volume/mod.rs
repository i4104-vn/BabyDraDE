//! System audio volume controls via wpctl / pactl / amixer

use std::process::Command;

/// Audio control utility available on the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBackend {
    Wpctl,
    Pactl,
    Amixer,
}

/// Detects the first available audio control utility.
///
/// # Returns
/// The detected [`AudioBackend`].
pub fn detect_audio_backend() -> AudioBackend {
    match true {
        _ if Command::new("wpctl").arg("--version").output().is_ok() => AudioBackend::Wpctl,
        _ if Command::new("pactl").arg("--version").output().is_ok() => AudioBackend::Pactl,
        _ => AudioBackend::Amixer,
    }
}

/// Parses `Volume: 0.50 [MUTED]` (or `Volume: 50%`) into a percentage.
fn parse_wpctl_volume_percent(stdout: &str) -> Option<f64> {
    let token = stdout.split_whitespace().nth(1)?;
    let is_percent = token.ends_with('%');
    let value = token.trim_end_matches('%').parse::<f64>().ok()?;
    let percent = if is_percent { value } else { value * 100.0 };

    if percent.is_finite() {
        Some(percent.clamp(0.0, 100.0))
    } else {
        None
    }
}

/// Reads current volume percentage and mute state.
///
/// # Returns
/// `(volume_percent, is_muted)`; defaults to `(80.0, false)`.
pub fn get_volume_state() -> (f64, bool) {
    let backend = detect_audio_backend();
    match backend {
        AudioBackend::Wpctl => {
            if let Ok(output) = Command::new("wpctl")
                .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let is_muted = stdout.contains("[MUTED]");
                let vol = parse_wpctl_volume_percent(&stdout).unwrap_or(80.0);
                return (vol, is_muted);
            }
            (80.0, false)
        }
        AudioBackend::Pactl => {
            let mut vol = 80.0;
            let mut is_muted = false;
            if let Ok(output) = Command::new("pactl")
                .args(["get-sink-volume", "@DEFAULT_SINK@"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(pct_str) = stdout.split('/').nth(1) {
                    if let Ok(val) = pct_str.trim().trim_end_matches('%').parse::<f64>() {
                        vol = val.clamp(0.0, 100.0);
                    }
                }
            }
            if let Ok(output) = Command::new("pactl")
                .args(["get-sink-mute", "@DEFAULT_SINK@"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                is_muted = stdout.contains("yes");
            }
            (vol, is_muted)
        }
        AudioBackend::Amixer => (80.0, false),
    }
}

/// Sets the default output volume.
///
/// # Arguments
/// * `pct` - Target volume percentage (clamped by the backend).
pub fn set_volume(pct: f64) {
    let clamped = pct.clamp(0.0, 150.0);
    match detect_audio_backend() {
        AudioBackend::Wpctl => {
            // An explicit `VOL%` is an absolute target (without +/- it is
            // never treated as a relative step).
            let pct = clamped.round() as u64;
            let _ = Command::new("wpctl")
                .args([
                    "set-volume",
                    "@DEFAULT_AUDIO_SINK@",
                    &format!("{}%", pct),
                ])
                .output();
        }
        AudioBackend::Pactl => {
            let _ = Command::new("pactl")
                .args([
                    "set-sink-volume",
                    "@DEFAULT_SINK@",
                    &format!("{:.0}%", clamped),
                ])
                .output();
        }
        AudioBackend::Amixer => {
            let _ = Command::new("amixer")
                .args(["set", "Master", &format!("{:.0}%", clamped)])
                .output();
        }
    }
}

/// Sets or clears the default output mute flag.
///
/// # Arguments
/// * `muted` - `true` to mute, `false` to unmute.
pub fn set_muted(muted: bool) {
    let flag = if muted { "1" } else { "0" };
    match detect_audio_backend() {
        AudioBackend::Wpctl => {
            let _ = Command::new("wpctl")
                .args(["set-mute", "@DEFAULT_AUDIO_SINK@", flag])
                .output();
        }
        AudioBackend::Pactl => {
            let val = if muted { "1" } else { "0" };
            let _ = Command::new("pactl")
                .args(["set-sink-mute", "@DEFAULT_SINK@", val])
                .output();
        }
        AudioBackend::Amixer => {
            let val = if muted { "mute" } else { "unmute" };
            let _ = Command::new("amixer").args(["set", "Master", val]).output();
        }
    }
}

/// Reads the display name of the default audio output.
///
/// # Returns
/// The output name, or `None` when it cannot be determined.
pub fn get_active_output_name() -> Option<String> {
    if let Ok(output) = Command::new("wpctl").args(["status"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains('*')
                && (line.contains("Sink")
                    || line.contains("Audio/Sink")
                    || line.contains("Built-in")
                    || line.contains("HDMI")
                    || line.contains("Headset"))
            {
                let trimmed = line.trim_start_matches([' ', '*', '.']);
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fraction_and_percent_volume() {
        assert_eq!(parse_wpctl_volume_percent("Volume: 0.50"), Some(50.0));
        assert_eq!(
            parse_wpctl_volume_percent("Volume: 0.25 [MUTED]"),
            Some(25.0)
        );
        assert_eq!(parse_wpctl_volume_percent("Volume: 50%"), Some(50.0));
        assert_eq!(parse_wpctl_volume_percent("garbage"), None);
    }
}
