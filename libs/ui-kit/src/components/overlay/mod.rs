//! Overlay components for BabyDraDE UI Kit.

pub mod context_menu;
pub mod popover;

pub use context_menu::{ContextMenu, ContextMenuBuilder};
pub use popover::{
    clamp_to_bounds, create_popover, fit_popover, fit_popover_default, fit_rect, flip_position,
    max_popover_height, max_popover_width, monitor_rect_for, popup_fit, rect_for_position,
    widget_rect, CardTooltip, CrossAlign, FluentPopover, PopoverFit, ScreenRect, TooltipPopover,
    TooltipRow, ANCHOR_GAP, EDGE_MARGIN,
};
