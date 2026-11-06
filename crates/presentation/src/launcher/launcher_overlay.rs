use domain::{AppRegistry, CanvasRenderer, Color, DesktopEntry, Point, Rect, RenderError, Size};

/// Interactive App Launcher overlay UI state and renderer.
#[derive(Debug, Clone, Default)]
pub struct LauncherOverlay {
    pub visible: bool,
    pub search_query: String,
    pub selected_index: usize,
    pub matched_entries: Vec<DesktopEntry>,
}

impl LauncherOverlay {
    pub const WIDTH: u32 = 560;
    pub const HEIGHT: u32 = 420;
    pub const ITEM_HEIGHT: u32 = 48;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle(&mut self, registry: &dyn AppRegistry) {
        if self.visible {
            self.close();
        } else {
            self.open(registry);
        }
    }

    pub fn open(&mut self, registry: &dyn AppRegistry) {
        self.visible = true;
        self.search_query.clear();
        self.selected_index = 0;
        self.matched_entries = registry.search("");
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.search_query.clear();
        self.selected_index = 0;
    }

    pub fn on_query_changed(&mut self, query: String, registry: &dyn AppRegistry) {
        self.search_query = query;
        self.matched_entries = registry.search(&self.search_query);
        self.selected_index = 0;
    }

    pub fn on_char_input(&mut self, c: char, registry: &dyn AppRegistry) {
        self.search_query.push(c);
        let q = self.search_query.clone();
        self.on_query_changed(q, registry);
    }

    pub fn on_backspace(&mut self, registry: &dyn AppRegistry) {
        self.search_query.pop();
        let q = self.search_query.clone();
        self.on_query_changed(q, registry);
    }

    pub fn select_next(&mut self) {
        if !self.matched_entries.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.matched_entries.len() - 1);
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn selected_app(&self) -> Option<&DesktopEntry> {
        self.matched_entries.get(self.selected_index)
    }

    pub fn overlay_rect(screen_rect: Rect) -> Rect {
        let x = screen_rect.x() + (screen_rect.width() as i32 - Self::WIDTH as i32) / 2;
        let y = screen_rect.y() + (screen_rect.height() as i32 - Self::HEIGHT as i32) / 3;
        Rect::from_point_and_size(Point::new(x, y), Size::new(Self::WIDTH, Self::HEIGHT))
    }

    /// Renders the launcher popup and search results.
    pub fn render(
        &self,
        renderer: &mut dyn CanvasRenderer,
        screen_rect: Rect,
    ) -> Result<(), RenderError> {
        if !self.visible {
            return Ok(());
        }

        // 1. Semi-transparent backdrop overlay
        renderer.draw_rect(screen_rect, Color::rgba(0, 0, 0, 140))?;

        // 2. Launcher container box
        let box_rect = Self::overlay_rect(screen_rect);
        let bg_color = Color::rgb(30, 30, 46);
        let border_color = Color::rgb(137, 180, 250);
        renderer.draw_rect(box_rect, bg_color)?;

        // Border around box (2px)
        let border_top = Rect::new(box_rect.x(), box_rect.y(), box_rect.width(), 2);
        let border_bottom = Rect::new(box_rect.x(), box_rect.bottom() - 2, box_rect.width(), 2);
        let border_left = Rect::new(box_rect.x(), box_rect.y(), 2, box_rect.height());
        let border_right = Rect::new(box_rect.right() - 2, box_rect.y(), 2, box_rect.height());
        renderer.draw_rect(border_top, border_color)?;
        renderer.draw_rect(border_bottom, border_color)?;
        renderer.draw_rect(border_left, border_color)?;
        renderer.draw_rect(border_right, border_color)?;

        // 3. Search input header
        let search_bar = Rect::new(
            box_rect.x() + 16,
            box_rect.y() + 16,
            box_rect.width() - 32,
            40,
        );
        renderer.draw_rect(search_bar, Color::rgb(49, 50, 68))?;

        let search_display = if self.search_query.is_empty() {
            "Type to search applications..."
        } else {
            &self.search_query
        };
        let text_color = if self.search_query.is_empty() {
            Color::rgb(140, 145, 160)
        } else {
            Color::WHITE
        };
        renderer.draw_text(
            search_display,
            Point::new(search_bar.x() + 12, search_bar.y() + 10),
            16.0,
            text_color,
        )?;

        // 4. Render app items list (up to 6 visible items)
        let items_start_y = box_rect.y() + 72;
        let max_items = 6;
        for (i, entry) in self.matched_entries.iter().take(max_items).enumerate() {
            let item_y = items_start_y + (i as u32 * Self::ITEM_HEIGHT) as i32;
            let item_rect = Rect::new(
                box_rect.x() + 16,
                item_y,
                box_rect.width() - 32,
                Self::ITEM_HEIGHT - 4,
            );

            let is_selected = i == self.selected_index;
            let item_bg = if is_selected {
                Color::rgb(69, 71, 90)
            } else {
                Color::rgb(35, 38, 52)
            };
            renderer.draw_rect(item_rect, item_bg)?;

            let item_text_color = if is_selected {
                Color::rgb(137, 180, 250)
            } else {
                Color::WHITE
            };

            renderer.draw_text(
                &entry.name,
                Point::new(item_rect.x() + 16, item_rect.y() + 10),
                14.0,
                item_text_color,
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRegistry;
    impl AppRegistry for MockRegistry {
        fn refresh(&mut self) -> Result<Vec<DesktopEntry>, domain::AppRegistryError> {
            Ok(vec![
                DesktopEntry::new("alacritty.desktop", "Alacritty", "alacritty"),
                DesktopEntry::new("firefox.desktop", "Firefox", "firefox"),
            ])
        }
        fn find_by_id(&self, id: &str) -> Option<DesktopEntry> {
            if id == "firefox.desktop" {
                Some(DesktopEntry::new("firefox.desktop", "Firefox", "firefox"))
            } else {
                None
            }
        }
        fn search(&self, q: &str) -> Vec<DesktopEntry> {
            let mut list = vec![
                DesktopEntry::new("alacritty.desktop", "Alacritty", "alacritty"),
                DesktopEntry::new("firefox.desktop", "Firefox", "firefox"),
            ];
            if !q.is_empty() {
                list.retain(|e| e.name.to_lowercase().contains(&q.to_lowercase()));
            }
            list
        }
    }

    #[test]
    fn test_launcher_navigation() {
        let registry = MockRegistry;
        let mut launcher = LauncherOverlay::new();

        launcher.open(&registry);
        assert!(launcher.visible);
        assert_eq!(launcher.matched_entries.len(), 2);
        assert_eq!(launcher.selected_index, 0);

        launcher.select_next();
        assert_eq!(launcher.selected_index, 1);
        assert_eq!(launcher.selected_app().unwrap().name, "Firefox");

        launcher.select_previous();
        assert_eq!(launcher.selected_index, 0);
        assert_eq!(launcher.selected_app().unwrap().name, "Alacritty");

        launcher.on_query_changed("fire".into(), &registry);
        assert_eq!(launcher.matched_entries.len(), 1);
        assert_eq!(launcher.selected_app().unwrap().name, "Firefox");

        launcher.close();
        assert!(!launcher.visible);
    }
}
