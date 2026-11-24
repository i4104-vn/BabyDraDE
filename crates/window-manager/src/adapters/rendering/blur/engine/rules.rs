//! Blur rule matching and per-app toggles.

use std::collections::HashMap;

use super::super::config;
#[cfg(test)]
use tracing::info;

use super::*;

impl BlurEngine {
    pub(crate) fn new() -> Self {
        let params = config::load();
        let rules_lower = params
            .app_rules
            .iter()
            .map(|r| r.trim().to_lowercase())
            .filter(|r| !r.is_empty())
            .collect();
        Self {
            params,
            rules_lower,
            pipeline: None,
            pipeline_failed: false,
            backdrops: HashMap::new(),
            frame: 0,
        }
    }


    /// Add an app rule to the blur engine and persist configuration
    #[cfg(test)]
    pub(crate) fn add_app(&mut self, app_id: &str) {
        let trimmed = app_id.trim();
        if !trimmed.is_empty()
            && !self
                .params
                .app_rules
                .iter()
                .any(|r| r.eq_ignore_ascii_case(trimmed))
        {
            info!("BabyDraDE BlurEngine: adding app rule '{}'", trimmed);
            self.params.app_rules.push(trimmed.to_string());
            self.sync_rules();
            config::save(&self.params);
        }
    }

    /// Remove an app rule from the blur engine and persist configuration
    #[cfg(test)]
    pub(crate) fn remove_app(&mut self, app_id: &str) {
        let trimmed = app_id.trim();
        if let Some(pos) = self
            .params
            .app_rules
            .iter()
            .position(|r| r.eq_ignore_ascii_case(trimmed))
        {
            info!("BabyDraDE BlurEngine: removing app rule '{}'", trimmed);
            self.params.app_rules.remove(pos);
            self.sync_rules();
            config::save(&self.params);
        }
    }

    /// Toggle blur on a specific app_id (returns new state: true = blurred, false = unblurred)
    #[cfg(test)]
    pub(crate) fn toggle_app(&mut self, app_id: &str) -> bool {
        let trimmed = app_id.trim();
        if self.matches_app_or_title(Some(trimmed), None) {
            self.remove_app(trimmed);
            false
        } else {
            self.add_app(trimmed);
            true
        }
    }

    /// Toggle blur on all windows globally
    #[cfg(test)]
    pub(crate) fn set_blur_all(&mut self, blur_all: bool) {
        self.params.blur_all = blur_all;
        config::save(&self.params);
        info!("BabyDraDE BlurEngine: blur_all set to {}", blur_all);
    }


}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

use super::super::config;
#[cfg(test)]
use tracing::info;

use super::*;

    #[test]
    fn test_blur_engine_rules_and_matching() {
        let mut engine = BlurEngine::new();
        engine.params.blur_all = false;
        engine.params.app_rules = vec![
            "foot".to_string(),
            "alacritty".to_string(),
            "kitty".to_string(),
            "babydra-panel".to_string(),
        ];

        // Direct matching
        assert!(engine.matches_app_or_title(Some("foot"), None));
        assert!(engine.matches_app_or_title(Some("alacritty"), None));
        assert!(engine.matches_app_or_title(Some("Kitty"), None)); // case insensitive
        assert!(engine.matches_app_or_title(None, Some("foot terminal")));
        assert!(!engine.matches_app_or_title(Some("unknown-app"), Some("game")));

        // Add custom app
        engine.add_app("my-custom-terminal");
        assert!(engine.matches_app_or_title(Some("my-custom-terminal"), None));

        // Remove app
        engine.remove_app("my-custom-terminal");
        assert!(!engine.matches_app_or_title(Some("my-custom-terminal"), None));

        // Toggle app
        let toggled_on = engine.toggle_app("my-custom-terminal");
        assert!(toggled_on);
        assert!(engine.matches_app_or_title(Some("my-custom-terminal"), None));

        let toggled_off = engine.toggle_app("my-custom-terminal");
        assert!(!toggled_off);
        assert!(!engine.matches_app_or_title(Some("my-custom-terminal"), None));

        // Blur all toggle
        engine.set_blur_all(true);
        assert!(!engine.matches_layer("random-layer"));
        assert!(!engine.matches_layer("babydra-topbar"));
        engine.set_blur_all(false);
        assert!(engine.matches_layer("babydra-panel"));
        assert!(!engine.matches_layer("babydra-topbar"));
        assert!(!engine.matches_layer("random-layer"));
    }

    #[test]
    fn blur_all_matches_every_app() {
        let mut engine = BlurEngine::new();
        engine.params.blur_all = true;
        assert!(engine.matches_app_or_title(Some("unknown-app"), Some("anything")));
        assert!(engine.matches_app_or_title(None, None));

        engine.params.blur_all = false;
        assert!(!engine.matches_app_or_title(Some("unknown-app"), Some("anything")));
    }
}
