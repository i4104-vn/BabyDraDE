//! Disk Storage UI item for Control Center

use gtk4::prelude::*;
use services::i18n::trans;
use services::system::resources::disk::DiskReader;
use ui_kit::icons::get_icon_colored;

/// Formats bytes into human-readable string (e.g. "120.5 GB")
fn format_size(bytes: u64) -> String {
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if gb >= 1.0 {
        format!("{:.1} GB", gb)
    } else {
        let mb = bytes as f64 / (1024.0 * 1024.0);
        format!("{:.0} MB", mb)
    }
}

/// Creates a new storage usage box card.
pub fn create_disk_list_box() -> gtk4::Box {
    let card = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    card.add_css_class("control-disk-card");

    let title_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let disk_icon = get_icon_colored("server", 12, "#10b981");
    let title_label = gtk4::Label::new(Some(&trans("panel.storage_usage")));
    title_label.add_css_class("control-slider-title");

    title_row.append(&disk_icon);
    title_row.append(&title_label);
    card.append(&title_row);

    let list_container = gtk4::Box::new(gtk4::Orientation::Vertical, 8);

    // Read root partition
    let root_info = DiskReader::new("/").read();

    if let Some(disk) = root_info {
        let disk_item = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
        disk_item.add_css_class("control-disk-item");

        let label_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        label_box.add_css_class("control-disk-title-box");

        let name_label = gtk4::Label::new(Some(&disk.mount_point));
        name_label.add_css_class("control-disk-name");
        name_label.set_hexpand(true);
        name_label.set_halign(gtk4::Align::Start);

        let usage_label = gtk4::Label::new(Some(&format!(
            "{} / {} ({:.0}%)",
            format_size(disk.used_bytes),
            format_size(disk.total_bytes),
            disk.used_percent
        )));
        usage_label.add_css_class("control-disk-usage");
        usage_label.set_halign(gtk4::Align::End);

        label_box.append(&name_label);
        label_box.append(&usage_label);

        let progress = gtk4::ProgressBar::new();
        progress.set_fraction((disk.used_percent as f64) / 100.0);
        progress.set_hexpand(true);

        disk_item.append(&label_box);
        disk_item.append(&progress);

        list_container.append(&disk_item);
    } else {
        let no_disks = gtk4::Label::new(Some(&trans("panel.no_storage")));
        no_disks.add_css_class("tile-subtitle");
        list_container.append(&no_disks);
    }

    card.append(&list_container);
    card
}

