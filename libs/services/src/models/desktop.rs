//! Desktop icon layout and custom context menu items.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_true() -> bool {
    true
}

fn default_icon_size() -> u32 {
    48
}

fn default_grid_spacing() -> u32 {
    16
}

fn default_desktop_sort_by() -> String {
    "name".to_string()
}

/// Custom context menu entry configured for desktop right-click menus.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CustomContextItem {
    /// Human-readable label displayed in the menu.
    pub name: String,
    /// Shell command executed upon selecting this item.
    pub command: String,
    /// Optional freedesktop icon name or image path.
    #[serde(default)]
    pub icon: Option<String>,
}

/// Desktop icon grid and context menu configuration persisted in `~/.babydra/babydra.conf` under `[desktop]`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DesktopConfig {
    /// Whether file and application icons are displayed on the desktop background.
    #[serde(default = "default_true")]
    pub show_icons: bool,
    /// Desktop icon dimension in logical pixels.
    #[serde(default = "default_icon_size")]
    pub icon_size: u32,
    /// Spacing between desktop grid cells in logical pixels.
    #[serde(default = "default_grid_spacing")]
    pub grid_spacing: u32,
    /// Attribute used to sort desktop icons (`"name"`, `"date"`, `"type"`, `"size"`).
    #[serde(default = "default_desktop_sort_by")]
    pub sort_by: String,
    /// Whether icons automatically align to grid cells.
    #[serde(default = "default_true")]
    pub auto_arrange: bool,
    /// In-memory cache of icon coordinates on the grid (skipped from persistent config).
    #[serde(skip)]
    pub icon_positions: HashMap<String, (i32, i32)>,
    /// User-defined custom context menu actions.
    #[serde(default)]
    pub custom_context_items: Vec<CustomContextItem>,
}

impl Default for DesktopConfig {
    /// Returns default desktop icon configuration.
    ///
    /// # Returns
    ///
    /// A new [`DesktopConfig`] with 48px icons, 16px grid spacing, and auto arrange enabled.
    fn default() -> Self {
        Self {
            show_icons: true,
            icon_size: default_icon_size(),
            grid_spacing: default_grid_spacing(),
            sort_by: default_desktop_sort_by(),
            auto_arrange: true,
            icon_positions: HashMap::new(),
            custom_context_items: Vec::new(),
        }
    }
}
