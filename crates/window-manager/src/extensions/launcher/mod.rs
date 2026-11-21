//! Compositor-owned application launcher: a vertically centred icon bar on the
//! left, shown only via its shortcut, over a dimmed full-screen backdrop.

use cairo::{Context, FontSlant, Format, FontWeight, ImageSurface, Operator};
use gtk4::{
    gdk::prelude::GdkCairoContextExt,
    gdk_pixbuf::Pixbuf,
    gio::{prelude::*, AppInfo},
};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::element::{
            memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            Kind,
        },
    },
    utils::{Buffer, Logical, Physical, Point, Rectangle, Scale, Transform},
};

use crate::rendering::CustomRenderElements;

use super::switcher::draw::{app_color, draw_text, rounded_rect};

const LAUNCHER_X: f64 = 14.0;
const ICON: f64 = 56.0;
const ICON_GAP: f64 = 10.0;
const ICON_PAD: f64 = 10.0;
const PANEL_W: f64 = ICON + ICON_PAD * 2.0;
const VISIBLE_SLOTS: usize = 12;
const PANEL_H: f64 = ICON_PAD * 2.0
    + VISIBLE_SLOTS as f64 * ICON
    + (VISIBLE_SLOTS - 1) as f64 * ICON_GAP;
const TOOLTIP_W: f64 = 240.0;
const BACKDROP_ALPHA: f64 = 0.58;

struct LauncherApp {
    name: String,
    info: AppInfo,
    icon: Option<Pixbuf>,
}

/// State of the launcher: app list, visibility, hover, scroll and cached buffer.
pub(crate) struct LauncherState {
    apps: Vec<LauncherApp>,
    visible: bool,
    hovered: Option<usize>,
    scroll_offset: usize,
    panel_y: f64,
    buffer: Option<MemoryRenderBuffer>,
    cache_key: Option<(usize, Option<usize>, usize, i32, i32)>,
}

impl Default for LauncherState {
    fn default() -> Self {
        Self::new()
    }
}

impl LauncherState {
    pub(crate) fn new() -> Self {
        let mut apps: Vec<LauncherApp> = AppInfo::all()
            .into_iter()
            .filter(|info| info.should_show())
            .map(|info| {
                let id = info.id().map(|id| id.to_string()).unwrap_or_default();
                LauncherApp {
                    name: info.display_name().to_string(),
                    icon: ui_kit::get_app_icon_pixbuf_any(&id, 96),
                    info,
                }
            })
            .collect();
        apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Self {
            apps,
            visible: false,
            hovered: None,
            scroll_offset: 0,
            panel_y: 0.0,
            buffer: None,
            cache_key: None,
        }
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.visible
    }

    pub(crate) fn toggle(&mut self) -> bool {
        self.visible = !self.visible;
        if !self.visible {
            self.hovered = None;
            self.scroll_offset = 0;
        }
        self.cache_key = None;
        self.visible
    }

    fn max_scroll(&self) -> usize {
        self.apps.len().saturating_sub(VISIBLE_SLOTS)
    }

    /// Scrolls by a number of icon slots. Used by the mouse wheel.
    pub(crate) fn scroll_by(&mut self, slots: f64) {
        let max = self.max_scroll() as f64;
        let next = (self.scroll_offset as f64 + slots).round().clamp(0.0, max) as usize;
        if next != self.scroll_offset {
            self.scroll_offset = next;
            self.hovered = None;
            self.cache_key = None;
        }
    }

    fn icon_rect(&self, slot: usize) -> (f64, f64) {
        let x = LAUNCHER_X + ICON_PAD;
        let y = self.panel_y + ICON_PAD + slot as f64 * (ICON + ICON_GAP);
        (x, y)
    }

    /// Returns the absolute app index under `point`, if any.
    pub(crate) fn index_at(&self, point: Point<f64, Logical>) -> Option<usize> {
        if !self.visible {
            return None;
        }
        if point.x < LAUNCHER_X || point.x > LAUNCHER_X + PANEL_W {
            return None;
        }
        let local_y = point.y - self.panel_y - ICON_PAD;
        if local_y < 0.0 {
            return None;
        }
        let slot = (local_y / (ICON + ICON_GAP)) as usize;
        if slot >= VISIBLE_SLOTS {
            return None;
        }
        let (_, y) = self.icon_rect(slot);
        if point.y <= y + ICON {
            let index = self.scroll_offset + slot;
            if index < self.apps.len() {
                return Some(index);
            }
        }
        None
    }

    /// True when the pointer is over the launcher column (used for wheel scroll).
    pub(crate) fn is_over_panel(&self, point: Point<f64, Logical>) -> bool {
        self.visible && point.x >= LAUNCHER_X && point.x <= LAUNCHER_X + PANEL_W
    }

    pub(crate) fn update_hover(&mut self, point: Point<f64, Logical>) -> bool {
        let next = self.index_at(point);
        if next != self.hovered {
            self.hovered = next;
            true
        } else {
            false
        }
    }

    pub(crate) fn launch(&mut self, index: usize) {
        if let Some(app) = self.apps.get(index) {
            let _ = app.info.launch(&[], None::<&gtk4::gio::AppLaunchContext>);
        }
        self.visible = false;
        self.hovered = None;
        self.cache_key = None;
    }

    pub(crate) fn render(
        &mut self,
        renderer: &mut smithay::backend::renderer::glow::GlowRenderer,
        output_size: (i32, i32),
        scale: Scale<f64>,
    ) -> Vec<CustomRenderElements> {
        if !self.visible || self.apps.is_empty() {
            return Vec::new();
        }

        let (screen_w, screen_h) = output_size;
        self.panel_y = ((screen_h as f64 - PANEL_H) / 2.0).max(0.0);
        let (buf_w, buf_h) = (screen_w.max(1), screen_h.max(1));

        let key = (
            self.apps.len(),
            self.hovered,
            self.scroll_offset,
            buf_w,
            buf_h,
        );
        if self.cache_key != Some(key) || self.buffer.is_none() {
            if let Some(pixels) = render_pixels(buf_w, buf_h, scale, |ctx| self.draw(ctx)) {
                let mut buffer = MemoryRenderBuffer::new(
                    Fourcc::Argb8888,
                    (buf_w, buf_h),
                    1,
                    Transform::Normal,
                    None,
                );
                {
                    let mut context = buffer.render();
                    let _ = context.draw(|dst| {
                        if dst.len() != pixels.len() {
                            return Err(());
                        }
                        dst.copy_from_slice(&pixels);
                        Ok(vec![Rectangle::<i32, Buffer>::from_size((buf_w, buf_h).into())])
                    });
                }
                self.buffer = Some(buffer);
                self.cache_key = Some(key);
            }
        }

        let Some(buffer) = self.buffer.as_mut() else {
            return Vec::new();
        };
        match MemoryRenderBufferRenderElement::from_buffer(
            renderer,
            Point::<f64, Physical>::from((0.0, 0.0)),
            buffer,
            None,
            None,
            None,
            Kind::Unspecified,
        ) {
            Ok(elem) => vec![CustomRenderElements::Memory(elem)],
            Err(_) => Vec::new(),
        }
    }

    fn draw(&self, context: &Context) -> Result<(), cairo::Error> {
        context.set_operator(Operator::Source);
        context.set_source_rgba(0.0, 0.0, 0.0, BACKDROP_ALPHA);
        context.paint()?;
        context.set_operator(Operator::Over);

        // Soft shadow and translucent rounded panel.
        rounded_rect(
            context,
            LAUNCHER_X + 3.0,
            self.panel_y + 5.0,
            PANEL_W,
            PANEL_H,
            22.0,
        );
        context.set_source_rgba(0.0, 0.0, 0.0, 0.40);
        context.fill()?;

        rounded_rect(context, LAUNCHER_X, self.panel_y, PANEL_W, PANEL_H, 22.0);
        context.set_source_rgba(0.13, 0.13, 0.18, 0.94);
        context.fill_preserve()?;
        context.set_source_rgba(1.0, 1.0, 1.0, 0.16);
        context.set_line_width(1.0);
        context.stroke()?;

        let end = (self.scroll_offset + VISIBLE_SLOTS).min(self.apps.len());
        for slot in 0..VISIBLE_SLOTS {
            let index = self.scroll_offset + slot;
            if index >= end {
                break;
            }
            let app = &self.apps[index];
            let (x, y) = self.icon_rect(slot);
            let hovered = self.hovered == Some(index);
            let color = app_color(&app.name);

            if hovered {
                rounded_rect(context, x - 3.0, y - 3.0, ICON + 6.0, ICON + 6.0, 17.0);
                context.set_source_rgba(color.0, color.1, color.2, 0.60);
                context.fill()?;
            }

            rounded_rect(context, x, y, ICON, ICON, 14.0);
            context.set_source_rgba(
                (color.0 * 0.55 + 0.22).min(1.0),
                (color.1 * 0.55 + 0.26).min(1.0),
                (color.2 * 0.55 + 0.32).min(1.0),
                if hovered { 1.0 } else { 0.92 },
            );
            context.fill_preserve()?;
            context.set_source_rgba(1.0, 1.0, 1.0, if hovered { 0.85 } else { 0.22 });
            context.set_line_width(if hovered { 2.0 } else { 1.0 });
            context.stroke()?;

            if let Some(ref pixbuf) = app.icon {
                let pw = pixbuf.width() as f64;
                let ph = pixbuf.height() as f64;
                if pw > 0.0 && ph > 0.0 {
                    context.save()?;
                    rounded_rect(context, x, y, ICON, ICON, 14.0);
                    context.clip();
                    let pad = 5.0;
                    let size = ICON - pad * 2.0;
                    context.translate(x + pad, y + pad);
                    context.scale(size / pw, size / ph);
                    context.set_source_pixbuf(pixbuf, 0.0, 0.0);
                    context.paint()?;
                    context.restore()?;
                    continue;
                }
            }

            let initial = app.name.chars().next().unwrap_or('A').to_uppercase().to_string();
            context.select_font_face("Sans", FontSlant::Normal, FontWeight::Bold);
            context.set_font_size(ICON * 0.44);
            let extents = context.text_extents(&initial)?;
            let tx = x + (ICON - extents.x_advance()) / 2.0 - extents.x_bearing();
            let ty = y + (ICON - extents.height()) / 2.0 - extents.y_bearing();
            draw_text(context, &initial, tx, ty, (1.0, 1.0, 1.0, 0.95))?;
        }

        // Scrollbar when there are more apps than visible slots.
        if self.max_scroll() > 0 {
            let track_x = LAUNCHER_X + PANEL_W - 5.0;
            let track_y = self.panel_y + ICON_PAD;
            let track_h = PANEL_H - ICON_PAD * 2.0;
            rounded_rect(context, track_x, track_y, 3.0, track_h, 1.5);
            context.set_source_rgba(1.0, 1.0, 1.0, 0.14);
            context.fill()?;
            let thumb_h = (track_h * VISIBLE_SLOTS as f64 / self.apps.len() as f64).max(24.0);
            let progress = if self.max_scroll() > 0 {
                self.scroll_offset as f64 / self.max_scroll() as f64
            } else {
                0.0
            };
            let thumb_y = track_y + (track_h - thumb_h) * progress;
            rounded_rect(context, track_x, thumb_y, 3.0, thumb_h, 1.5);
            context.set_source_rgba(0.85, 0.90, 1.0, 0.75);
            context.fill()?;
        }

        // Name card on hover.
        if let Some(index) = self.hovered {
            if index >= self.scroll_offset && index < self.scroll_offset + VISIBLE_SLOTS {
                if let Some(app) = self.apps.get(index) {
                    let (_, y) = self.icon_rect(index - self.scroll_offset);
                    let tx = LAUNCHER_X + PANEL_W + 12.0;
                    let ty = y + (ICON - 40.0) / 2.0;
                    context.select_font_face("Sans", FontSlant::Normal, FontWeight::Normal);
                    context.set_font_size(16.0);
                    let extents = context.text_extents(&app.name)?;
                    let card_w = (extents.x_advance() + 30.0).min(TOOLTIP_W);
                    rounded_rect(context, tx, ty, card_w, 40.0, 12.0);
                    context.set_source_rgba(0.07, 0.07, 0.11, 0.94);
                    context.fill_preserve()?;
                    context.set_source_rgba(1.0, 1.0, 1.0, 0.18);
                    context.set_line_width(1.0);
                    context.stroke()?;
                    draw_text(
                        context,
                        &app.name,
                        tx + 15.0,
                        ty + 20.0 - extents.height() / 2.0 - extents.y_bearing(),
                        (1.0, 1.0, 1.0, 1.0),
                    )?;
                }
            }
        }

        Ok(())
    }
}

fn render_pixels<F>(width: i32, height: i32, scale: Scale<f64>, draw_fn: F) -> Option<Vec<u8>>
where
    F: FnOnce(&Context) -> Result<(), cairo::Error>,
{
    let mut surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
    {
        let context = Context::new(&surface).ok()?;
        context.scale(scale.x, scale.y);
        draw_fn(&context).ok()?;
    }
    surface.flush();
    surface.data().ok().map(|data| data.to_vec())
}
