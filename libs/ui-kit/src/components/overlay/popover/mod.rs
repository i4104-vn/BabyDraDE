//! Popover overlay module for BabyDraDE UI Kit.

pub mod base;
pub mod geometry;
pub mod tooltip;

pub use base::{create_popover, FluentPopover};
pub use geometry::{
    clamp_to_bounds, fit_popover, fit_popover_default, fit_rect, flip_position, max_popover_height,
    max_popover_width, monitor_rect_for, popup_fit, rect_for_position, widget_rect, CrossAlign,
    PopoverFit, ScreenRect, ANCHOR_GAP, EDGE_MARGIN,
};
pub use tooltip::{CardTooltip, TooltipPopover, TooltipRow};
