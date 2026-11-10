pub mod config;
pub mod desktop;
pub mod i18n;
pub mod settings {
    pub use crate::config::*;
}
pub mod system;
pub mod theme;
pub mod tray;
pub mod wallpaper;

pub use i18n::trans;
