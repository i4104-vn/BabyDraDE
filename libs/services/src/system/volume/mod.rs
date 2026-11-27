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

/// Extracts the device name from a `wpctl status` line.
///
/// Status lines look like ` │  *   43. Built-in Audio Stereo [vol: 0.25]`.
/// The tree prefix, default marker, numeric id and trailing status metadata
/// are dropped; only the sink name is kept.
fn clean_output_name(line: &str) -> Option<String> {
    let mut text = line.trim();

    // The `*` marks the default sink; other sinks are not the active output.
    if !text.contains('*') {
        return None;
    }

    text = text.trim_start_matches(|c: char| {
        c.is_whitespace() || "│├└┌┐┬┴┼─*".contains(c)
    });

    // Drop the leading sink id, e.g. `43. `.
    if let Some((id, rest)) = text.split_once('.') {
        if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
            text = rest.trim_start();
        }
    }

    // Drop trailing bracketed metadata, e.g. `[vol: 0.25] [MUTED]`.
    while text.ends_with(']') {
        match text.rfind('[') {
            Some(start) => text = text[..start].trim_end(),
            None => break,
        }
    }

    // Drop bare trailing volume / mute annotations, if present.
    let lower = text.to_ascii_lowercase();
    for marker in [" vol:", " volume:", "\tvol:", "\tvolume:"] {
        if let Some(pos) = lower.rfind(marker) {
            text = text[..pos].trim_end();
            break;
        }
    }
    if text.to_ascii_lowercase().ends_with(" muted") {
        let end = text.len() - " muted".len();
        text = text[..end].trim_end();
    }

    let name = text.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
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
                let mut vol = 80.0;
                if let Some(vol_str) = stdout.split_whitespace().nth(1) {
                    if let Ok(parsed) = vol_str.parse::<f64>() {
                        vol = parsed * 100.0;
                    }
                }
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
                        vol = val;
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
            let frac = clamped / 100.0;
            let _ = Command::new("wpctl")
                .args([
                    "set-volume",
                    "@DEFAULT_AUDIO_SINK@",
                    &format!("{:.2}", frac),
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
                return clean_output_name(line);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_volume_suffix_from_wpctl_line() {
        let line = " │  *   43. Built-in Audio Analog Stereo        [vol: 0.25]";
        assert_eq!(
            clean_output_name(line).as_deref(),
            Some("Built-in Audio Analog Stereo")
        );
    }

    #[test]
    fn strips_percent_volume_and_mute_flags() {
        let line = " │  *   12. HDMI Output [volume: 50%] [MUTED]";
        assert_eq!(clean_output_name(line).as_deref(), Some("HDMI Output"));
    }

    #[test]
    fn keeps_long_device_names() {
        let name = "Very Long USB Audio Interface Name That Should Not Be Trimmed Here";
        let line = format!("  *    7. {} [vol: 100%]", name);
        assert_eq!(clean_output_name(&line).as_deref(), Some(name));
    }

    #[test]
    fn strips_bare_volume_suffix() {
        let line = " *  9. Headset vol: 80%";
        assert_eq!(clean_output_name(line).as_deref(), Some("Headset"));
    }

    #[test]
    fn ignores_lines_without_the_default_marker() {
        let line = " │  43. Built-in Audio Analog Stereo [vol: 0.25]";
        assert_eq!(clean_output_name(line), None);
    }

}
