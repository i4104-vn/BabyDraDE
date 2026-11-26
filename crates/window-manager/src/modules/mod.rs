//! Feature modules. Each is a self-contained vertical slice and may only
//! depend on the shared kernel, core rules, app ports and infra adapters.

pub(crate) mod auto_rotate;
pub(crate) mod blur;
pub(crate) mod input;
pub(crate) mod render;
pub(crate) mod shell;
pub(crate) mod wayland;
pub(crate) mod window;
