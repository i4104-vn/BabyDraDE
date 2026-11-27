use crate::system::dbus_menu::DbusMenuItem;

/// Scores a menu by the number of usable entries it exposes. A candidate that
/// merely returns separators or nameless entries is rejected, so an unrelated
/// exporter with an empty/inert menu cannot win the race.
pub(in crate::system::dbus_menu) fn menu_quality(items: &[DbusMenuItem]) -> usize {
    items
        .iter()
        .filter(|item| item.visible && !item.is_separator && !item.label.trim().is_empty())
        .map(|item| 1 + item.children.len().min(6))
        .sum()
}

#[cfg(test)]
mod tests {
    use crate::system::dbus_menu::DbusMenuItem;

    use super::menu_quality;

    #[test]
    fn menu_quality_rejects_empty_and_separator_only_menus() {
        assert_eq!(menu_quality(&[]), 0);
        let separators = vec![DbusMenuItem {
            visible: true,
            is_separator: true,
            ..Default::default()
        }];
        assert_eq!(menu_quality(&separators), 0);
    }

    #[test]
    fn menu_quality_prefers_richer_menus() {
        let small = vec![DbusMenuItem {
            label: "File".to_string(),
            visible: true,
            ..Default::default()
        }];
        let rich = vec![
            DbusMenuItem {
                label: "File".to_string(),
                visible: true,
                children: vec![DbusMenuItem::default(); 8],
                ..Default::default()
            },
            DbusMenuItem {
                label: "Edit".to_string(),
                visible: true,
                ..Default::default()
            },
        ];
        assert!(menu_quality(&rich) > menu_quality(&small));
        assert_eq!(
            menu_quality(&[DbusMenuItem {
                label: "Hidden".to_string(),
                visible: false,
                ..Default::default()
            }]),
            0
        );
    }
}
