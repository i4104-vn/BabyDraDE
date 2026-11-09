//! System audio volume controls via wpctl / pactl / amixer

use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBackend {
    Wpctl,
    Pactl,
    Amixer,
}

pub fn detect_audio_backend() -> AudioBackend {
    if Command::new("wpctl").arg("--version").output().is_ok() {
        AudioBackend::Wpctl
    } else if Command::new("pactl").arg("--version").output().is_ok() {
        AudioBackend::Pactl
    } else {
        AudioBackend::Amixer
    }
}

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
        AudioBackend::Amixer => {
            (80.0, false)
        }
    }
}

pub fn set_volume(pct: f64) {
    let clamped = pct.clamp(0.0, 150.0);
    match detect_audio_backend() {
        AudioBackend::Wpctl => {
            let frac = clamped / 100.0;
            let _ = Command::new("wpctl")
                .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &format!("{:.2}", frac)])
                .output();
        }
        AudioBackend::Pactl => {
            let _ = Command::new("pactl")
                .args(["set-sink-volume", "@DEFAULT_SINK@", &format!("{:.0}%", clamped)])
                .output();
        }
        AudioBackend::Amixer => {
            let _ = Command::new("amixer")
                .args(["set", "Master", &format!("{:.0}%", clamped)])
                .output();
        }
    }
}

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
            let _ = Command::new("amixer")
                .args(["set", "Master", val])
                .output();
        }
    }
}

pub fn get_active_output_name() -> Option<String> {
    if let Ok(output) = Command::new("wpctl").args(["status"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains('*') && (line.contains("Sink") || line.contains("Audio/Sink") || line.contains("Built-in") || line.contains("HDMI") || line.contains("Headset")) {
                let trimmed = line.trim_start_matches(|c| c == ' ' || c == '*' || c == '.');
                return Some(trimmed.to_string());
            }
        }
    }
    None
}
