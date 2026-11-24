//! Search-surface buffers and render-element assembly.

use smithay::backend::{
    allocator::Fourcc,
    renderer::element::{
        memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
        Kind,
    },
};
use smithay::utils::{Physical, Point, Transform};

use super::context::SurfaceContext;
use super::LauncherState;
use crate::adapters::desktop::launcher::search::buffer::{
    buffer_damage as search_buffer_damage, panel_region,
    render_backdrop_pixels as render_search_backdrop,
};
use crate::adapters::desktop::launcher::search::draw::draw_panel;
use crate::adapters::desktop::launcher::search::layout::SearchLayout;
use crate::adapters::rendering::buffer::render_pixels;
use crate::adapters::rendering::CustomRenderElements;
use common::constants::launcher::SEARCH_BACKDROP_ALPHA;

/// Builds (or refreshes) the search panel and dimmer buffers.
pub(super) fn render_search(state: &mut LauncherState, ctx: &SurfaceContext<'_>) {
    // -- search surface -------------------------------------------------
    let layout = SearchLayout::compute(ctx.logical.0, ctx.logical.1, state.search.content_height());
    state.search.set_layout(layout);
    let search_region = panel_region(&layout, ctx.scale, ctx.screen.0, ctx.screen.1);
    let panel_key = state.search.panel_key(search_region);

    if state.search_panel_buffer.is_none() || state.search_panel_region != Some(search_region) {
        state.search_panel_buffer = Some(MemoryRenderBuffer::new(
            Fourcc::Argb8888,
            (search_region.2, search_region.3),
            1,
            Transform::Normal,
            None,
        ));
        state.search_panel_region = Some(search_region);
        state.search_panel_key = None;
    }

    if ctx.search_presence > 0.001 && state.search_panel_key.as_ref() != Some(&panel_key) {
        let search = &state.search;
        let presence = ctx.search_presence;
        if let Some(pixels) = render_pixels(
            search_region.2,
            search_region.3,
            ctx.scale,
            (search_region.0, search_region.1),
            |ctx| draw_panel(ctx, search, &layout, presence),
        ) {
            if let Some(buffer) = state.search_panel_buffer.as_mut() {
                let mut context = buffer.render();
                let _ = context.draw(|dst| {
                    if dst.len() != pixels.len() {
                        return Err(());
                    }
                    dst.copy_from_slice(&pixels);
                    Ok(search_buffer_damage(search_region.2, search_region.3))
                });
                state.search_panel_key = Some(panel_key);
            }
        }
    }

    let search_backdrop_size = (ctx.screen.0, ctx.screen.1);
    let search_backdrop_key = (
        search_backdrop_size,
        ctx.search_tick,
        ctx.search_presence.to_bits(),
    );
    if ctx.search_presence > 0.001
        && (state.search_backdrop_buffer.is_none()
            || state.search_backdrop_size != Some(search_backdrop_size)
            || state.search_backdrop_key != Some(search_backdrop_key))
    {
        if let Some(pixels) = render_search_backdrop(
            ctx.screen.0,
            ctx.screen.1,
            ctx.scale,
            SEARCH_BACKDROP_ALPHA * ctx.search_presence,
        ) {
            let mut buffer = MemoryRenderBuffer::new(
                Fourcc::Argb8888,
                search_backdrop_size,
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
                    Ok(search_buffer_damage(ctx.screen.0, ctx.screen.1))
                });
            }
            state.search_backdrop_buffer = Some(buffer);
            state.search_backdrop_size = Some(search_backdrop_size);
            state.search_backdrop_key = Some(search_backdrop_key);
        }
    }
}

/// Collects the cached buffers as front-to-back render elements.
pub(super) fn collect_elements(
    state: &LauncherState,
    ctx: &mut SurfaceContext<'_>,
) -> Vec<CustomRenderElements> {
    // Front-to-back: search panel, its dimmer, then the fading dock.
    let mut elements = Vec::with_capacity(4);
    if ctx.search_presence > 0.001 {
        if let Some(buffer) = state.search_panel_buffer.as_ref() {
            if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                &mut *ctx.renderer,
                Point::<f64, Physical>::from((
                    ctx.search_region.0 as f64,
                    ctx.search_region.1 as f64,
                )),
                buffer,
                None,
                None,
                None,
                Kind::Unspecified,
            ) {
                elements.push(CustomRenderElements::Memory(elem));
            }
        }
        if let Some(backdrop) = state.search_backdrop_buffer.as_ref() {
            if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                &mut *ctx.renderer,
                Point::<f64, Physical>::from((0.0, 0.0)),
                backdrop,
                None,
                None,
                None,
                Kind::Unspecified,
            ) {
                elements.push(CustomRenderElements::Memory(elem));
            }
        }
    }
    if ctx.dock_presence > 0.001 {
        if let Some(buffer) = state.buffer.as_ref() {
            if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                &mut *ctx.renderer,
                Point::<f64, Physical>::from((ctx.region.0 as f64, ctx.region.1 as f64)),
                buffer,
                None,
                None,
                None,
                Kind::Unspecified,
            ) {
                elements.push(CustomRenderElements::Memory(elem));
            }
        }
        if let Some(backdrop) = state.backdrop_buffer.as_ref() {
            if let Ok(elem) = MemoryRenderBufferRenderElement::from_buffer(
                &mut *ctx.renderer,
                Point::<f64, Physical>::from((0.0, 0.0)),
                backdrop,
                None,
                None,
                None,
                Kind::Unspecified,
            ) {
                elements.push(CustomRenderElements::Memory(elem));
            }
        }
    }
    elements
}
