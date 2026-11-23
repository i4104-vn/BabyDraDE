//! Interface adapters — translate between the outside world and the inner layers.
//!
//! Wayland protocol handlers, input event translation, rendering and desktop
//! overlays live here. Adapters may depend on `application` (use cases) and
//! `domain` (entities), but never on `infrastructure` or `composition`: those
//! outer layers depend on adapters, not the other way around.

pub(crate) mod compositor;
pub(crate) mod desktop;
pub(crate) mod input;
pub(crate) mod rendering;
pub(crate) mod window;
