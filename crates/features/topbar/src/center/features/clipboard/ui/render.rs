//! Rendering logic for the Dynamic Island clipboard popover.

use super::popover::{ClipboardPopover, MAX_VISIBLE_ITEMS};
use gtk4::prelude::*;
use services::system::clipboard::ClipboardEntry;

fn load_thumbnail(bytes: &[u8], size: i32) -> Option<gdk_pixbuf::Pixbuf> {
    let loader = gdk_pixbuf::PixbufLoader::new();
    loader.write(bytes).ok()?;
    loader.close().ok()?;
    let pixbuf = loader.pixbuf()?;
    pixbuf.scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear)
}

/// BabyDraDE stores a plain UTF-8 preview; split it the same way upstream's
/// `ClipboardEntry::preview_lines()` did (up to three lines + overflow count).
fn preview_lines(entry: &ClipboardEntry) -> (String, usize) {
    let lines: Vec<&str> = entry.preview.lines().filter(|l| !l.trim().is_empty()).collect();
    let shown = lines.iter().take(3).copied().collect::<Vec<_>>().join("\n");
    (shown, lines.len().saturating_sub(3))
}

fn format_more_lines(count: usize) -> String {
    if count == 1 {
        services::i18n::trans("island.clipboard_more_lines_one")
    } else {
        services::i18n::trans("island.clipboard_more_lines_many")
            .replace("{count}", &count.to_string())
    }
}

/// Renders the current clipboard entries into the popover and updates highlight.
pub fn render_popover(
    popover: &ClipboardPopover,
    entries: &[ClipboardEntry],
    selected_index: usize,
) {
    let total = entries.len();
    let max_items = services::config::load_clipboard_config().max_items.max(1);

    popover
        .counter_label
        .set_text(&format!("{}/{}", total, max_items));

    if total == 0 {
        popover.empty_label.set_visible(true);
        popover.list_box.set_visible(false);
        return;
    }

    popover.empty_label.set_visible(false);
    popover.list_box.set_visible(true);

    let visible_count = total.min(MAX_VISIBLE_ITEMS);
    let start = if total <= MAX_VISIBLE_ITEMS {
        0
    } else if selected_index >= 3 {
        (selected_index - 2).min(total - MAX_VISIBLE_ITEMS)
    } else {
        0
    };

    for i in 0..MAX_VISIBLE_ITEMS {
        let row = &popover.rows[i];
        if i < visible_count {
            let actual_idx = start + i;
            let entry = &entries[actual_idx];

            row.container.set_visible(true);
            row.num_label.set_text(&format!("{}.", actual_idx + 1));

            while let Some(child) = row.icon_holder.first_child() {
                row.icon_holder.remove(&child);
            }

            if entry.mime.starts_with("image/") {
                row.text_label
                    .set_text(&services::i18n::trans("island.clipboard_image"));
                row.more_label.set_visible(false);

                if let Some(pixbuf) = load_thumbnail(&entry.data, 18) {
                    let image = gtk4::Image::from_pixbuf(Some(&pixbuf));
                    image.add_css_class("clipboard-popover-thumb");
                    image.set_valign(gtk4::Align::Center);
                    row.icon_holder.append(&image);
                } else {
                    let icon = ui_kit::icons::get_icon("camera", 13);
                    icon.set_valign(gtk4::Align::Center);
                    row.icon_holder.append(&icon);
                }
            } else {
                let (preview, more_count) = preview_lines(entry);
                let display_text = if preview.is_empty() {
                    services::i18n::trans("island.clipboard_empty_item")
                } else {
                    preview
                };
                row.text_label.set_text(&display_text);

                if more_count > 0 {
                    row.more_label.set_text(&format_more_lines(more_count));
                    row.more_label.set_visible(true);
                } else {
                    row.more_label.set_visible(false);
                }

                let icon = ui_kit::icons::get_icon("text", 13);
                icon.set_valign(gtk4::Align::Center);
                row.icon_holder.append(&icon);
            }

            if actual_idx == selected_index {
                row.container.add_css_class("selected");
            } else {
                row.container.remove_css_class("selected");
            }
        } else {
            row.container.set_visible(false);
        }
    }
}
