use std::collections::HashMap;

use domain::{Size, WindowId};

/// Identifier for imported GPU texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GpuTextureId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedTexture {
    pub id: GpuTextureId,
    pub size: Size,
    pub buffer_age: u64,
}

/// Cache of GPU textures to avoid re-importing unchanged client buffers every frame.
#[derive(Debug, Default)]
pub struct TextureCache {
    entries: HashMap<WindowId, CachedTexture>,
    next_id: u64,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn get(&self, window_id: WindowId) -> Option<&CachedTexture> {
        self.entries.get(&window_id)
    }

    /// Imports or updates a texture entry for a window.
    pub fn update_texture(&mut self, window_id: WindowId, size: Size) -> GpuTextureId {
        if let Some(entry) = self.entries.get_mut(&window_id) {
            entry.size = size;
            entry.buffer_age += 1;
            entry.id
        } else {
            let id = GpuTextureId(self.next_id);
            self.next_id += 1;
            self.entries.insert(
                window_id,
                CachedTexture {
                    id,
                    size,
                    buffer_age: 1,
                },
            );
            id
        }
    }

    /// Removes cached texture when client window closes.
    pub fn invalidate(&mut self, window_id: WindowId) -> Option<CachedTexture> {
        self.entries.remove(&window_id)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_cache_caching_and_invalidation() {
        let mut cache = TextureCache::new();
        let win = WindowId::new(10);
        let size = Size::new(800, 600);

        let tex1 = cache.update_texture(win, size);
        assert_eq!(cache.get(win).unwrap().id, tex1);
        assert_eq!(cache.get(win).unwrap().buffer_age, 1);

        // Update keeps same texture id but increments age
        let tex2 = cache.update_texture(win, size);
        assert_eq!(tex1, tex2);
        assert_eq!(cache.get(win).unwrap().buffer_age, 2);

        // Invalidate removes entry
        assert!(cache.invalidate(win).is_some());
        assert!(cache.get(win).is_none());
    }
}
