//! Pushes metadata into the capsule labels and media popover.

use super::super::service::format_time;
use super::super::MediaFeature;

impl MediaFeature {
    /// Updates track labels, play icon and starts artwork fetching.
    pub(crate) fn update_view(&mut self) {
        let base = {
            let meta = &self.cached;
            if meta.title.is_empty() {
                if meta.artist.is_empty() {
                    "Music Player".to_string()
                } else {
                    meta.artist.clone()
                }
            } else if meta.artist.is_empty() {
                meta.title.clone()
            } else {
                format!("{} - {}", meta.artist, meta.title)
            }
        };
        let short = if base.chars().count() > 18 {
            base.chars().take(15).collect::<String>() + "..."
        } else {
            base
        };
        self.track_label.set_label(&short);

        let meta = &self.cached;
        let song_key = format!(
            "{}|{}|{}|{}",
            meta.title, meta.artist, meta.player_name_raw, meta.art_url
        );
        if song_key == self.song_key {
            return;
        }
        self.song_key = song_key;
        self.art_loaded.set(false);

        if let Some(popover) = self.popover.borrow().as_ref() {
            popover.title.set_text(if meta.title.is_empty() {
                "Unknown title"
            } else {
                &meta.title
            });
            popover.artist.set_text(if meta.artist.is_empty() {
                "Unknown artist"
            } else {
                &meta.artist
            });
            let player = if meta.player_name_raw.is_empty() {
                "Music Player".to_string()
            } else {
                let mut chars = meta.player_name_raw.chars();
                chars
                    .next()
                    .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                    .unwrap_or_default()
            };
            popover.app_name.set_text(&player);
        }

        if self.play_state.get() != Some(meta.playing) {
            if let Some(popover) = self.popover.borrow().as_ref() {
                let icon = if meta.playing { "pause" } else { "play" };
                ui_kit::icons::set_image_from_icon(&popover.play_btn_icon, icon, 22);
                self.play_state.set(Some(meta.playing));
            }
        }

        if !meta.art_url.is_empty() {
            *self.current_art_url.borrow_mut() = meta.art_url.clone();
            let url = meta.art_url.clone();
            let sender = self.art_tx.clone();
            std::thread::spawn(move || {
                let bytes = match url.strip_prefix("file://") {
                    Some(path) => std::fs::read(path).ok(),
                    None if url.starts_with('/') => std::fs::read(&url).ok(),
                    None if url.starts_with("http") => std::process::Command::new("curl")
                        .args(["-s", "-L", "--max-time", "5", &url])
                        .output()
                        .ok()
                        .filter(|out| out.status.success())
                        .map(|out| out.stdout),
                    None => Some(Vec::new()),
                };
                if let Some(bytes) = bytes {
                    let _ = sender.send((url, bytes));
                }
            });
        }
    }

    /// Advances the progress bar using the last MPRIS position plus local time.
    pub(crate) fn update_progress(&self) {
        let binding = self.popover.borrow();
        let Some(popover) = binding.as_ref() else {
            return;
        };
        let length = self.len_secs.get();
        if length <= 0.0 {
            return;
        }
        let elapsed = if self.playing_state.get() {
            self.pos_at.get().elapsed().as_secs_f64()
        } else {
            0.0
        };
        let position = (self.pos_base.get() + elapsed).min(length);
        popover
            .progress_bar
            .set_fraction((position / length).clamp(0.0, 1.0));
        popover.position_lbl.set_text(&format_time(position));
        popover.length_lbl.set_text(&format_time(length));
    }
}
