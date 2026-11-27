//! Pointer hit testing across windows, borders and layer surfaces.

pub(crate) mod frame;
mod window;

pub(crate) use frame::titlebar;
