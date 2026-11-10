use smithay::{
    desktop::Window,
    utils::{Physical, Rectangle},
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};
use tracing::{error, info};

use super::{config, config::BlurParams, gl::BlurPipeline};

pub struct BlurEngine {
    pub params: BlurParams,
    pipeline: Option<BlurPipeline>,
}

impl std::fmt::Debug for BlurEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlurEngine")
            .field("params", &self.params)
            .field("has_pipeline", &self.pipeline.is_some())
            .finish()
    }
}

impl Default for BlurEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl BlurEngine {
    pub fn new() -> Self {
        let params = config::load();
        Self {
            params,
            pipeline: None,
        }
    }

    /// Check if a window should have background blur applied
    pub fn matches_window(&self, window: &Window) -> bool {
        if !self.params.enabled {
            return false;
        }

        if self.params.blur_all {
            return true;
        }

        let app_id = self.get_window_app_id(window);
        let title = self.get_window_title(window);

        self.matches_app_or_title(app_id.as_deref(), title.as_deref())
    }

    /// Check if a LayerShell surface namespace should have background blur applied
    pub fn matches_layer(&self, namespace: &str) -> bool {
        if !self.params.enabled {
            return false;
        }

        let ns_lower = namespace.to_lowercase();
        if ns_lower.contains("topbar") || ns_lower.contains("desktop") || ns_lower.contains("wallpaper") {
            return false;
        }

        self.params.app_rules.iter().any(|rule| {
            let rule_lower = rule.to_lowercase();
            ns_lower == rule_lower || ns_lower.contains(&rule_lower)
        })
    }

    /// Check if an app_id or window title matches any configured blur rule
    pub fn matches_app_or_title(&self, app_id: Option<&str>, title: Option<&str>) -> bool {
        for rule in &self.params.app_rules {
            let rule_lower = rule.trim().to_lowercase();
            if rule_lower == "*" {
                return true;
            }

            if let Some(app) = app_id {
                let app_lower = app.to_lowercase();
                if app_lower == rule_lower || app_lower.contains(&rule_lower) {
                    return true;
                }
            }

            if let Some(t) = title {
                let t_lower = t.to_lowercase();
                if t_lower == rule_lower || t_lower.contains(&rule_lower) {
                    return true;
                }
            }
        }

        false
    }

    /// Get the XDG app_id of a window
    pub fn get_window_app_id(&self, window: &Window) -> Option<String> {
        let toplevel = window.toplevel()?;
        with_states(toplevel.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|d| d.lock().ok().and_then(|role| role.app_id.clone()))
        })
    }

    /// Get the XDG title of a window
    pub fn get_window_title(&self, window: &Window) -> Option<String> {
        let toplevel = window.toplevel()?;
        with_states(toplevel.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|d| d.lock().ok().and_then(|role| role.title.clone()))
        })
    }

    /// Add an app rule to the blur engine and persist configuration
    pub fn add_app(&mut self, app_id: &str) {
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
            config::save(&self.params);
        }
    }

    /// Remove an app rule from the blur engine and persist configuration
    pub fn remove_app(&mut self, app_id: &str) {
        let trimmed = app_id.trim();
        if let Some(pos) = self
            .params
            .app_rules
            .iter()
            .position(|r| r.eq_ignore_ascii_case(trimmed))
        {
            info!("BabyDraDE BlurEngine: removing app rule '{}'", trimmed);
            self.params.app_rules.remove(pos);
            config::save(&self.params);
        }
    }

    /// Toggle blur on a specific app_id (returns new state: true = blurred, false = unblurred)
    pub fn toggle_app(&mut self, app_id: &str) -> bool {
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
    pub fn set_blur_all(&mut self, blur_all: bool) {
        self.params.blur_all = blur_all;
        config::save(&self.params);
        info!("BabyDraDE BlurEngine: blur_all set to {}", blur_all);
    }

    /// Render Dual Kawase blur over the destination physical rectangle
    pub fn render_blur(
        &mut self,
        gl: &glow::Context,
        dst: Rectangle<i32, Physical>,
        corner_radius: f32,
        projection: [f32; 9],
    ) {
        if !self.params.enabled {
            return;
        }

        if self.pipeline.is_none() {
            unsafe {
                match BlurPipeline::new(gl) {
                    Ok(p) => self.pipeline = Some(p),
                    Err(err) => {
                        error!("Failed to initialize BabyDraDE BlurPipeline: {err}");
                        return;
                    }
                }
            }
        }

        if let Some(pipeline) = self.pipeline.as_mut() {
            unsafe {
                pipeline.render_blur(gl, dst, corner_radius, projection, &self.params);
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}
