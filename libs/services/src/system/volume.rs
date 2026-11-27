//! System audio volume controls via wpctl / pactl / amixer

use std::process::Command;

/// Volume used when no backend reports a value; a conservative audible level.
const DEFAULT_VOLUME_PERCENT: f64 = 80.0;
/// Upper bound accepted by `set_volume`; some backends allow boosting above 100.
const MAX_VOLUME_PERCENT: f64 = 150.0;

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
        _ if command_succeeds("wpctl", &["--version"]) => AudioBackend::Wpctl,
        _ if command_succeeds("pactl", &["--version"]) => AudioBackend::Pactl,
        _ => AudioBackend::Amixer,
    }
}

/// Runs a command and reports whether it actually succeeded (not just spawned).
fn command_succeeds(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Primary backend first, then every other installed backend as a fallback.
fn backend_order() -> Vec<AudioBackend> {
    let primary = detect_audio_backend();
    let mut order = vec![primary];
    for backend in [
        AudioBackend::Wpctl,
        AudioBackend::Pactl,
        AudioBackend::Amixer,
    ] {
        if !order.contains(&backend) && command_succeeds(backend.program(), &["--version"]) {
            order.push(backend);
        }
    }
    order
}

impl AudioBackend {
    fn program(self) -> &'static str {
        match self {
            AudioBackend::Wpctl => "wpctl",
            AudioBackend::Pactl => "pactl",
            AudioBackend::Amixer => "amixer",
        }
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

    text = text.trim_start_matches(|c: char| c.is_whitespace() || "│├└┌┐┬┴┼─*".contains(c));

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

/// Parses `Volume: 0.50` (or `Volume: 50%`) into a percentage.
fn parse_wpctl_volume_percent(stdout: &str) -> Option<f64> {
    let token = stdout.split_whitespace().nth(1)?;
    let is_percent = token.ends_with('%');
    let value = token.trim_end_matches('%').parse::<f64>().ok()?;
    let percent = if is_percent { value } else { value * 100.0 };
    percent.is_finite().then(|| percent.clamp(0.0, 100.0))
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
                let vol = parse_wpctl_volume_percent(&stdout).unwrap_or(DEFAULT_VOLUME_PERCENT);
                return (vol, is_muted);
            }
            (DEFAULT_VOLUME_PERCENT, false)
        }
        AudioBackend::Pactl => {
            let mut vol = DEFAULT_VOLUME_PERCENT;
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
        AudioBackend::Amixer => (DEFAULT_VOLUME_PERCENT, false),
    }
}

/// Sets the default output volume.
///
/// # Arguments
/// * `pct` - Target volume percentage (clamped by the backend).
pub fn set_volume(pct: f64) {
    let clamped = pct.clamp(0.0, MAX_VOLUME_PERCENT);
    let percent = format!("{:.0}%", clamped);
    for backend in backend_order() {
        let ok = match backend {
            // An explicit `VOL%` is an absolute target (without +/- it is
            // never treated as a relative step).
            AudioBackend::Wpctl => command_succeeds(
                "wpctl",
                &[
                    "set-volume",
                    "@DEFAULT_AUDIO_SINK@",
                    &format!("{}%", clamped.round() as u64),
                ],
            ),
            AudioBackend::Pactl => {
                command_succeeds("pactl", &["set-sink-volume", "@DEFAULT_SINK@", &percent])
            }
            AudioBackend::Amixer => command_succeeds("amixer", &["set", "Master", &percent]),
        };
        if ok {
            return;
        }
    }
    tracing::warn!("could not set volume through any available audio backend");
}

/// Desired mute state of the default output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MuteState {
    /// Silence the default output.
    Muted,
    /// Restore audio on the default output.
    Unmuted,
}

impl MuteState {
    fn wpctl_value(self) -> &'static str {
        match self {
            Self::Muted => "1",
            Self::Unmuted => "0",
        }
    }

    fn amixer_value(self) -> &'static str {
        match self {
            Self::Muted => "mute",
            Self::Unmuted => "unmute",
        }
    }
}

/// Sets or clears the default output mute flag.
///
/// # Arguments
/// * `state` - Whether to mute or unmute the default output.
pub fn set_mute(state: MuteState) {
    for backend in backend_order() {
        let ok = match backend {
            AudioBackend::Wpctl => command_succeeds(
                "wpctl",
                &["set-mute", "@DEFAULT_AUDIO_SINK@", state.wpctl_value()],
            ),
            AudioBackend::Pactl => command_succeeds(
                "pactl",
                &["set-sink-mute", "@DEFAULT_SINK@", state.wpctl_value()],
            ),
            AudioBackend::Amixer => {
                command_succeeds("amixer", &["set", "Master", state.amixer_value()])
            }
        };
        if ok {
            return;
        }
    }
    tracing::warn!("could not change mute state through any available audio backend");
}

/// Compatibility wrapper for callers that pass a boolean.
///
/// # Arguments
/// * `muted` - `true` to mute, `false` to unmute.
pub fn set_muted(muted: bool) {
    set_mute(if muted {
        MuteState::Muted
    } else {
        MuteState::Unmuted
    });
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
