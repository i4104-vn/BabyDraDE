//! Overlay components for BabyDraDE UI Kit.

pub mod context_menu;
pub mod popover;

pub use context_menu::{any_open, ContextMenu, ContextMenuBuilder};
pub use popover::{
    clamp_to_bounds, create_popover, fit_popover, fit_popover_at_point, fit_popover_below,
    fit_popover_default, fit_popover_with_policy, fit_rect, fit_rect_with_policy, flip_position,
    max_popover_height, max_popover_width, monitor_rect_for, native_rect_for, offset_for_rect,
    pointing_rect_for, popover_rect_from_pointing, popup_fit, popup_fit_below, rect_for_position,
    widget_rect, CardTooltip, CrossAlign, FitPolicy, FluentPopover, PopoverFit, ScreenRect,
    TooltipPopover, TooltipRow, ANCHOR_GAP, EDGE_MARGIN,
};
