//! Blur engine construction, configuration and rule matching.

use std::collections::HashMap;

use smithay::desktop::Window;

#[cfg(test)]
use tracing::info;

use super::super::config;
use super::BlurEngine;

impl BlurEngine {
    /// Creates the engine from the persisted configuration.
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

    pub(crate) fn sync_rules(&mut self) {
        self.rules_lower = self
            .params
            .app_rules
            .iter()
            .map(|r| r.trim().to_lowercase())
            .filter(|r| !r.is_empty())
            .collect();
    }

    /// Reload blur configuration from disk and environment variables
    pub(crate) fn reload_config(&mut self) {
        self.params = config::load();
        self.sync_rules();
    }

    /// Check if a window should have background blur applied (strictly matching apps recorded in config)
    pub(crate) fn matches_window(&mut self, window: &Window) -> bool {
        if !self.params.enabled {
            return false;
        }

        if self.rules_lower.len() != self.params.app_rules.len() {
            self.sync_rules();
        }

        let app_id = self.get_window_app_id(window);
        let title = self.get_window_title(window);

        self.matches_app_or_title(app_id.as_deref(), title.as_deref())
    }

    /// Check if a LayerShell surface namespace should have background blur applied
    pub(crate) fn matches_layer(&mut self, namespace: &str) -> bool {
        if !self.params.enabled {
            return false;
        }

        let ns_lower = namespace.to_lowercase();
        if ns_lower.contains("topbar")
            || ns_lower.contains("desktop")
            || ns_lower.contains("wallpaper")
        {
            return false;
        }

        if self.rules_lower.len() != self.params.app_rules.len() {
            self.sync_rules();
        }

        self.rules_lower.iter().any(|rule_lower| {
            rule_lower == "*" || ns_lower == *rule_lower || ns_lower.contains(rule_lower)
        })
    }

    /// Check if an app_id or window title matches any configured blur rule
    pub(crate) fn matches_app_or_title(&self, app_id: Option<&str>, title: Option<&str>) -> bool {
        if self.params.blur_all {
            return true;
        }

        let app_lower = app_id.map(str::to_lowercase);
        let title_lower = title.map(str::to_lowercase);

        let use_cached = self.rules_lower.len() == self.params.app_rules.len();
        if use_cached {
            for rule in &self.rules_lower {
                if rule == "*" {
                    return true;
                }
                if let Some(ref app) = app_lower {
                    if app == rule || app.contains(rule) {
                        return true;
                    }
                }
                if let Some(ref t) = title_lower {
                    if t == rule || t.contains(rule) {
                        return true;
                    }
                }
            }
        } else {
            for rule in &self.params.app_rules {
                let rule_lower = rule.trim().to_lowercase();
                if rule_lower == "*" {
                    return true;
                }
                if let Some(ref app) = app_lower {
                    if app == &rule_lower || app.contains(&rule_lower) {
                        return true;
                    }
                }
                if let Some(ref t) = title_lower {
                    if t == &rule_lower || t.contains(&rule_lower) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Get the cached XDG app_id of a window
    pub(crate) fn get_window_app_id(&self, window: &Window) -> Option<String> {
        crate::modules::window::metadata::get(window).map(|metadata| metadata.app_id)
    }

    /// Get the cached XDG title of a window
    pub(crate) fn get_window_title(&self, window: &Window) -> Option<String> {
        crate::modules::window::metadata::get(window).map(|metadata| metadata.title)
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
