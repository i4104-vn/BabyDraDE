//! Widget construction and pure layout rules for the panel tray.
//!
//! The compact tray shows at most [`TRAY_ICONS_PER_ROW`] icons followed by the
//! temperature widget. The expanded tray reuses the same icons in a grid with
//! exactly [`TRAY_ICONS_PER_ROW`] icons per row and keeps the temperature
//! widget pinned to the top-right corner.

use gtk4::prelude::*;
use services::tray::TrayItem;
use std::time::{Duration, Instant};
use ui_kit::icons::get_theme_or_svg_icon;

const TRAY_ICON_SIZE: i32 = 16;

/// Icons per row in both the collapsed and the expanded tray.
pub(crate) const TRAY_ICONS_PER_ROW: usize = 3;

/// Grace period after the pointer (or the last context menu) leaves both
/// tray surfaces before the expanded view is closed.
pub(crate) const AUTO_CLOSE_GRACE: Duration = Duration::from_millis(300);

/// Interaction state used to decide whether the expanded tray stays open.
///
/// The state is deliberately display-independent so the visibility rule can be
/// unit tested without a running GTK main loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TrayVisibility {
    /// Pointer is over the compact tray in the panel.
    pub pointer_in_tray: bool,
    /// Pointer is over the expanded tray surface.
    pub pointer_in_expanded: bool,
    /// A shared context menu is currently open somewhere in the shell.
    pub menu_open: bool,
    /// Moment the pointer left both surfaces, if it has left at all.
    pub left_at: Option<Instant>,
}

impl Default for TrayVisibility {
    fn default() -> Self {
        Self {
            pointer_in_tray: false,
            pointer_in_expanded: false,
            menu_open: false,
            left_at: None,
        }
    }
}

/// Whether the expanded tray should be visible for `visibility` at `now`.
///
/// Hovering either surface or an open context menu keeps it open. Once both
/// surfaces are left and no menu is open, it survives for [`AUTO_CLOSE_GRACE`]
/// so that crossing the small gap between the panel and the popup does not
/// make it flicker. No click-outside handler is involved.
pub(crate) fn should_show_expanded(visibility: &TrayVisibility, now: Instant) -> bool {
    if visibility.pointer_in_tray || visibility.pointer_in_expanded || visibility.menu_open {
        return true;
    }

    visibility
        .left_at
        .is_some_and(|left| now.saturating_duration_since(left) < AUTO_CLOSE_GRACE)
}

/// Splits tray items into rows of at most [`TRAY_ICONS_PER_ROW`] icons.
pub(crate) fn chunk_tray_items<T>(items: &[T]) -> Vec<&[T]> {
    items.chunks(TRAY_ICONS_PER_ROW).collect()
}

/// Icons displayed in the collapsed tray; the caller appends the temperature
/// widget separately so it always stays on the top row at the right.
pub(crate) fn collapsed_tray_items<T>(items: &[T]) -> &[T] {
    &items[..items.len().min(TRAY_ICONS_PER_ROW)]
}

pub fn build_tray_container() -> gtk4::Box {
    let tray_container = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    tray_container.add_css_class("panel-tray-box");
    tray_container.set_valign(gtk4::Align::Center);
    tray_container.set_halign(gtk4::Align::Center);
    tray_container.set_visible(false);
    tray_container
}

/// Grid for the expanded view: three icon columns plus the temperature column.
pub(crate) fn build_expanded_grid() -> gtk4::Grid {
    let grid = gtk4::Grid::new();
    grid.set_column_spacing(6);
    grid.set_row_spacing(6);
    grid.set_column_homogeneous(true);
    grid.set_row_homogeneous(true);
    grid.set_halign(gtk4::Align::End);
    grid.set_valign(gtk4::Align::Start);
    grid
}

pub fn build_tray_button(item: &TrayItem) -> gtk4::Button {
    let btn = gtk4::Button::new();
    btn.add_css_class("panel-tray-item-btn");

    // Hover always shows a name, even for items that only advertise an id.
    let tooltip = if item.title.trim().is_empty() {
        if item.icon_name.is_empty() {
            &item.service
        } else {
            &item.icon_name
        }
    } else {
        &item.title
    };
    btn.set_tooltip_text(Some(tooltip));

    btn.set_valign(gtk4::Align::Center);
    btn.set_halign(gtk4::Align::Center);
    btn.set_receives_default(false);
    btn.set_focusable(false);
    btn.set_child(Some(&get_theme_or_svg_icon(
        &item.icon_name,
        TRAY_ICON_SIZE,
    )));

    btn
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(count: usize) -> Vec<usize> {
        (0..count).collect()
    }

    #[test]
    fn chunking_uses_rows_of_three_and_keeps_a_short_last_row() {
        assert!(chunk_tray_items::<usize>(&[]).is_empty());
        assert_eq!(chunk_tray_items(&items(1)), vec![&[0][..]]);
        assert_eq!(chunk_tray_items(&items(3)), vec![&[0, 1, 2][..]]);
        assert_eq!(
            chunk_tray_items(&items(7)),
            vec![&[0, 1, 2][..], &[3, 4, 5][..], &[6][..]]
        );
    }

    #[test]
    fn collapsed_view_never_shows_more_than_three_icons() {
        assert!(collapsed_tray_items::<usize>(&[]).is_empty());
        assert_eq!(collapsed_tray_items(&items(2)), &[0, 1]);
        assert_eq!(collapsed_tray_items(&items(3)), &[0, 1, 2]);
        assert_eq!(collapsed_tray_items(&items(9)), &[0, 1, 2]);
    }

    #[test]
    fn hover_or_open_menu_keeps_expanded_view_visible() {
        let now = Instant::now();
        let state = TrayVisibility {
            pointer_in_tray: true,
            ..Default::default()
        };
        assert!(should_show_expanded(&state, now));

        let state = TrayVisibility {
            pointer_in_expanded: true,
            ..Default::default()
        };
        assert!(should_show_expanded(&state, now));

        let state = TrayVisibility {
            menu_open: true,
            left_at: Some(now - AUTO_CLOSE_GRACE * 2),
            ..Default::default()
        };
        assert!(should_show_expanded(&state, now));
    }

    #[test]
    fn leaving_both_surfaces_closes_only_after_the_grace_period() {
        let now = Instant::now();
        let state = TrayVisibility {
            left_at: Some(now - Duration::from_millis(50)),
            ..Default::default()
        };
        assert!(should_show_expanded(&state, now));

        let state = TrayVisibility {
            left_at: Some(now - AUTO_CLOSE_GRACE),
            ..Default::default()
        };
        assert!(!should_show_expanded(&state, now));

        // Never having entered the tray must not keep the view open either.
        assert!(!should_show_expanded(&TrayVisibility::default(), now));
    }
}
