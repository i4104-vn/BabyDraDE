//! Dynamic Island State and Arbitration Manager

use super::features::{DefaultFeature, IslandFeature, MediaFeature, RecordingFeature};

pub struct IslandManager {
    features: Vec<Box<dyn IslandFeature>>,
}

impl Default for IslandManager {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandManager {
    pub fn new() -> Self {
        Self {
            features: vec![
                Box::new(DefaultFeature::new()),
                Box::new(MediaFeature::new()),
                Box::new(RecordingFeature::new()),
            ],
        }
    }

    /// Register an external custom feature
    pub fn register_feature(&mut self, feature: Box<dyn IslandFeature>) {
        self.features.push(feature);
    }

    /// Selects the highest priority active feature
    pub fn active_feature(&self) -> Option<&dyn IslandFeature> {
        self.features
            .iter()
            .map(|f| f.as_ref())
            .filter(|f| f.is_active())
            .max_by_key(|f| f.priority())
    }
}
