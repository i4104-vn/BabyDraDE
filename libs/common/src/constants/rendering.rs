//! Shading, blur, and cursor rendering constants.

/// Maximum Dual Kawase downsample / upsample blur iterations.
pub const MAX_BLUR_PASSES: usize = 4;

/// Lifetime of inactive blur textures in the GPU cache (in seconds).
pub const BLUR_CACHE_LIFETIME: u64 = 600;

/// Maximum GPU VRAM memory allocated for cached blur textures (96 MB).
pub const MAX_BLUR_CACHE_BYTES: usize = 96 * 1024 * 1024;

/// Built-in fallback cursor bitmap width.
pub const CURSOR_WIDTH: i32 = 16;

/// Built-in fallback cursor bitmap height.
pub const CURSOR_HEIGHT: i32 = 20;
