//! Dynamic Island State and Arbitration Manager

use super::features::{DefaultFeature, IslandFeature, MediaFeature, NotificationFeature, RecordingFeature};

pub struct IslandManager {
    features: Vec<Box<dyn IslandFeature>>,
}

impl IslandManager {
    pub fn new() -> Self {
        let mut features: Vec<Box<dyn IslandFeature>> = Vec::new();

        // Register built-in extensible features
        features.push(Box::new(DefaultFeature::new()));
        features.push(Box::new(MediaFeature::new()));
        features.push(Box::new(RecordingFeature::new()));
        features.push(Box::new(NotificationFeature::new()));

        Self { features }
    }

    /// Register an external custom feature
    pub fn register_feature(&mut self, feature: Box<dyn IslandFeature>) {
        self.features.push(feature);
    }

    /// Selects the highest priority active feature
    pub fn active_feature(&self) -> Option<&Box<dyn IslandFeature>> {
        self.features
            .iter()
            .filter(|f| f.is_active())
            .max_by_key(|f| f.priority())
    }
}

