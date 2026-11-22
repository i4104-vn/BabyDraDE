use std::collections::HashMap;

use smithay::{
    backend::renderer::element::Id,
    desktop::Window,
    utils::{Physical, Rectangle},
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};
use tracing::{error, info};

use super::{config, config::BlurParams, gl::BlurCache, gl::BlurPipeline};

use common::constants::{
    BLUR_CACHE_LIFETIME as CACHE_LIFETIME, MAX_BLUR_CACHE_BYTES as MAX_CACHE_BYTES,
};

pub struct BlurEngine {
    pub params: BlurParams,
    rules_lower: Vec<String>,
    pipeline: Option<BlurPipeline>,
    /// Set when the blur pipeline could not be created (shader compilation,
    /// missing GL features, ...). Blur is disabled in that case instead of
    /// retrying - and forcing a full redraw - on every frame.
    pipeline_failed: bool,
    /// Per element caches holding the captured background and the blurred
    /// result. `None` marks an element whose offscreen target could not be
    /// created, so that it also stops forcing full redraws.
    backdrops: HashMap<Id, Option<BlurCache>>,
    /// Counter used to expire caches of elements that are gone.
    frame: u64,
}

impl std::fmt::Debug for BlurEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlurEngine")
            .field("params", &self.params)
            .field("has_pipeline", &self.pipeline.is_some())
            .field("pipeline_failed", &self.pipeline_failed)
            .field("cached_backdrops", &self.backdrops.len())
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

    /// Rebuild lowercase cache of app_rules for fast matching
    pub fn sync_rules(&mut self) {
        self.rules_lower = self
            .params
            .app_rules
            .iter()
            .map(|r| r.trim().to_lowercase())
            .filter(|r| !r.is_empty())
            .collect();
    }

    /// Reload blur configuration from disk and environment variables
    pub fn reload_config(&mut self) {
        self.params = config::load();
        self.sync_rules();
    }

    /// Check if a window should have background blur applied (strictly matching apps recorded in config)
    pub fn matches_window(&mut self, window: &Window) -> bool {
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
    pub fn matches_layer(&mut self, namespace: &str) -> bool {
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
    pub fn matches_app_or_title(&self, app_id: Option<&str>, title: Option<&str>) -> bool {
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
            self.sync_rules();
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
            self.sync_rules();
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

    /// Initialize the blur pipeline shaders and pass targets on the OpenGL context
    pub fn init_pipeline(&mut self, gl: &glow::Context) {
        if self.pipeline.is_some() || self.pipeline_failed {
            return;
        }
        unsafe {
            match BlurPipeline::new(gl) {
                Ok(p) => self.pipeline = Some(p),
                Err(err) => {
                    error!("Failed to initialize BabyDraDE BlurPipeline: {err}");
                    self.pipeline_failed = true;
                }
            }
        }
    }

    /// Render Dual Kawase blur over the destination physical rectangle.
    ///
    /// Captures the framebuffer background, blurs it, and composites the result
    /// on every call. GPU texture objects are cached per element Id so that the
    /// allocation cost is paid only once per blurred window.
    pub fn render_blur(
        &mut self,
        gl: &glow::Context,
        id: &Id,
        dst: Rectangle<i32, Physical>,
        corner_radius: f32,
        projection: [f32; 9],
    ) {
        if !self.params.enabled || self.pipeline_failed {
            return;
        }

        if self.pipeline.is_none() {
            self.init_pipeline(gl);
            if self.pipeline.is_none() {
                return;
            }
        }

        // Take the pipeline out of the struct so that the caches can be borrowed
        // at the same time.
        let Some(mut pipeline) = self.pipeline.take() else {
            return;
        };

        let frame = self.frame;
        self.frame = self.frame.wrapping_add(1);

        if !self.backdrops.contains_key(id) {
            let cache = match unsafe { BlurCache::new(gl) } {
                Ok(cache) => Some(cache),
                Err(err) => {
                    error!("Failed to create the BabyDraDE blur cache: {err}");
                    None
                }
            };
            self.backdrops.insert(id.clone(), cache);
        }

        let Some(Some(cache)) = self.backdrops.get_mut(id) else {
            self.pipeline = Some(pipeline);
            return;
        };
        cache.last_used = frame;

        unsafe {
            pipeline.render_blur(gl, dst, corner_radius, projection, &self.params, cache);
        }

        self.pipeline = Some(pipeline);

        self.prune_caches(gl, id, frame);
    }

    /// Drops the caches of elements that are gone and keeps the memory usage
    /// of the remaining ones bounded.
    fn prune_caches(&mut self, gl: &glow::Context, keep: &Id, frame: u64) {
        let mut expired = Vec::new();
        self.backdrops.retain(|id, cache| {
            let last_used = cache.as_ref().map(|cache| cache.last_used).unwrap_or(0);
            if frame.saturating_sub(last_used) >= CACHE_LIFETIME {
                expired.push(id.clone());
                false
            } else {
                true
            }
        });

        for id in expired {
            if let Some(Some(mut cache)) = self.backdrops.remove(&id) {
                unsafe { cache.release(gl) };
            }
        }

        let mut total: usize = self
            .backdrops
            .values()
            .filter_map(|cache| cache.as_ref())
            .map(|cache| cache.bytes)
            .sum();
        if total <= MAX_CACHE_BYTES {
            return;
        }

        // Drop the least recently used caches until the memory usage is back
        // below the limit.
        let mut entries: Vec<(u64, Id)> = self
            .backdrops
            .iter()
            .filter_map(|(id, cache)| cache.as_ref().map(|cache| (cache.last_used, id.clone())))
            .collect();
        entries.sort_by_key(|(last_used, _)| *last_used);

        for (_, id) in entries {
            if total <= MAX_CACHE_BYTES {
                break;
            }
            if id == *keep {
                continue;
            }
            if let Some(Some(mut cache)) = self.backdrops.remove(&id) {
                total = total.saturating_sub(cache.bytes);
                unsafe { cache.release(gl) };
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
