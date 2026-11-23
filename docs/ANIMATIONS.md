# Animations, Constants and Enums

This document describes where animation logic, numeric tokens and shared
enumerations live, how the frame loop drives them, and how the two screen-edge
docks (launcher and switcher) reuse the same implementation.

## 1. Ownership map

| Location | Responsibility |
| --- | --- |
| `libs/animation/` | Clock-driven primitives (`Tween`, `Easing`), frame pacing, all animation durations. Framework-free. |
| `libs/animation-gtk/` | GTK frame-clock helpers (`slide_in`, `island_zoom_in`, …) isolated so the core animation crate does not pull in GTK4 |
| `libs/common/src/constants/` | Geometry, input thresholds, colour tokens, layout numbers |
| `libs/common/src/enums/` | Shared enumerations (`Action`, `SwipeDirection`, `KeyboardAction`, `DockSide`) |
| `libs/ui-kit/src/drawing/cairo.rs` | Cairo primitives: rounded paths, text, app colours, docked capsule |
| `libs/ui-kit/src/components/dock/` | Shared screen-edge dock: layout, state machine, rendering |
| `crates/window-manager/src/adapters/desktop/launcher/` | Left dock: app list, title capsule, backdrop |
| `crates/window-manager/src/adapters/desktop/switcher/` | Right dock: live preview, title strip, backdrop |
| `crates/window-manager/src/infrastructure/backends/` | Shared render loop plus Winit and Udev/DRM presentation |

Rule of thumb: **durations belong to `animation`, geometry and thresholds belong
to `common`, GTK frame-clock helpers belong to `animation-gtk`, behaviour belongs
to the owning component.**

## 2. Animation system

### 2.1 Primitives

`Tween` stores `started_at`, `duration`, `from`, `to` and an `Easing`. Values are
evaluated from `Instant::now()`, so motion is frame-rate independent: a 200 ms
transition looks identical at 60 Hz and 120 Hz. Key methods:

- `value()` / `value_at(instant)` — interpolated value.
- `retarget(to, duration, easing)` — restart from the current value.
- `is_animating()` / `frame_tick()` — cache invalidation and render scheduling.
- `frame_index()` — the same progress quantised to the 120 fps budget.

`Easing` lives in `libs/animation/src/curves/`, not in `common`. It provides
Linear, Quad, Cubic, Sine, Quart, Quint, Expo, Circ, Back and Spring curves. The
UI mostly uses `EaseOutCubic` for single steps, `EaseInOutCubic` for fades and
`Linear` for held-key repeats.

Frame helpers:

- `frame_index_since(start)` — monotonic index, one step per `FRAME_INTERVAL`.
- `bounded_delta(&mut last)` — delta capped at four frame budgets so a long
  stall cannot produce an oversized step.

### 2.2 Frame pacing

- **Udev/DRM:** a calloop `Timer` reschedules with
  `TimeoutAction::ToInstant(deadline + FRAME_INTERVAL)`, from the previous
  deadline rather than from render completion. A slow frame cannot drift the
  120 fps cadence.
- **Winit:** the window reports `TARGET_FPS * 1000` as its refresh rate and
  calls `request_redraw()` every frame.
- **GTK overlays:** `add_tick_callback` is vsync-synchronised.

Both compositor backends call the shared helpers in
`infrastructure/backends/render_loop.rs`: `begin_frame` applies pending window
actions and advances animations, `send_frames` delivers frame callbacks, and
`end_frame` refreshes the space, cleans up popups and flushes clients.

### 2.3 Duration catalogue

| Constant | Value | Used for |
| --- | --- | --- |
| `TARGET_FPS` / `FRAME_INTERVAL` | 120 / ~8.33 ms | Frame budget |
| `DEFAULT_DURATION` | 420 ms | Generic UI transitions |
| `NAVIGATION_DURATION` | 200 ms | Arrow-key or hover move in a dock |
| `SCROLL_DURATION` | 260 ms | Wheel and touchpad dock scroll |
| `FADE_IN_DURATION` / `FADE_OUT_DURATION` | 320 / 260 ms | Launcher presence |
| `REVEAL_DURATION` | 900 ms | Wallpaper circular reveal |
| `WINDOW_OPEN_DURATION` | 280 ms | Window map animation |
| `WINDOW_CLOSE_DURATION` | 260 ms | Window unmap animation |
| `WINDOW_MINIMIZE_DURATION` | 280 ms | Minimize shrink |
| `WINDOW_RESTORE_DURATION` | 300 ms | Restore grow |
| `WINDOW_GEOMETRY_DURATION` | 340 ms | Maximise and fullscreen |
| `SWITCHER_PREVIEW_DURATION` | 180 ms | Switcher presence and preview fade |
| `SWITCHER_SCROLL_DURATION` | 160 ms | Single Tab step |

All are `std::time::Duration` values passed directly to `Tween::retarget` or
`Tween::new`.

### 2.4 Dock state machine

`DockState` in `ui-kit/src/components/dock/state.rs` is shared by both docks:

- `position: Tween` — continuous selection index.
- `presence: Tween` — 0 hidden, 1 docked; drives fade and slide.
- `selected: Option<usize>` — logical selection.
- `closing: bool` — keeps the overlay alive until the fade completes.
- Held-key tracking (`held_direction`, `repeat_started`, `last_repeat`).

Transitions:

1. `open(index, animate)` snaps `position` and fades `presence` to 1.
2. `begin_close()` fades `presence` to 0; removal waits for the tween to settle.
3. `slide_offset(side)` returns `-(1 - presence) * PANEL_WIDTH` on the left dock
   and its positive counterpart on the right, so the capsule slides toward its
   screen edge while fading.
4. `move_selection` and `scroll_by` retarget `position` smoothly.
5. Held keys start a timer; repeats fire after `KEY_REPEAT_DELAY_MS` and then
   every `KEY_REPEAT_INTERVAL_MS` using a short 70 ms linear step. Consecutive
   steps chain into a continuous glide instead of restarting a long easing curve.

Both docks snap to the first or last entry when a step crosses a boundary. For
long lists `visual_center` clamps the scrolled content so the selection can loop
without showing an empty capsule. Pointer hover is deliberately separate from
the scrolling tween: `hovered` only retargets a dedicated `highlight` tween,
avoiding a feedback loop between hit testing and the carousel offset.

### 2.5 Cache invalidation

Both overlays rasterise into small off-screen buffers. While a tween is active,
`frame_tick()` contributes to the cache key so every frame gets a fresh buffer.
When all tweens rest the key becomes `u64::MAX` and static frames are cached.
Backdrop buffers are keyed by physical size plus presence tick.

### 2.6 Other animations

- **Wallpaper reveal:** `Tween` with `REVEAL_DURATION` and `EaseInOutCubic`,
  sampled in a GTK tick callback. Only the Cairo clip radius changes per frame.
- **Lock screen fades:** `Tween` with `EaseInOutCubic` (200/240 ms).
- **Window animations:** open, close, minimise, restore and geometry tweens.
- **Three-finger gestures:** a horizontal drag opens the switcher and scrubs
  the entry live (about 60 px per icon, wrapping at the ends). Lifting the
  fingers commits the selection; vertical swipes keep maximise/minimise.

## 3. Constants

### 3.1 Shared dock tokens

| Constant | Value | Meaning |
| --- | --- | --- |
| `PANEL_X` / `PANEL_WIDTH` | 0.0 / 72.0 | Capsule offset and width |
| `PANEL_CORNER_RADIUS` / `PANEL_FILLET_RADIUS` | 24 / 20 | Outer corners and edge fillets |
| `PANEL_PAD` | 24 | Vertical padding |
| `ICON` / `ICON_SELECTED` | 30 / 42 | Regular and selected icon size |
| `SLOT` | 50 | Distance between icon centres |
| `VISIBLE_SLOTS` | 9 | Maximum rendered icons |
| `SELECTION_SIZE` | 48 | Selection highlight |
| `INDICATOR_WIDTH` / `INDICATOR_HEIGHT` | 3 / 16 | Selection pill |
| `LABEL_GAP`, `LABEL_HEIGHT`, `LABEL_PAD`, `LABEL_FONT_SIZE` | 10, 32, 12, 13.5 | Launcher title capsule |
| `SHADOW_PAD` | 30 | Off-screen shadow margin |
| `BACKDROP_WIDTH_MIN` / `BACKDROP_WIDTH_RATIO` | 600 / 0.60 | Backdrop gradient size |

### 3.2 Input constants

| Constant | Value | Meaning |
| --- | --- | --- |
| `BTN_LEFT` / `BTN_RIGHT` / `BTN_MIDDLE` | 272 / 273 / 0x112 | evdev button codes |
| `DOUBLE_CLICK_TIMEOUT_MS` | 350 | Maximum gap between clicks |
| `DOUBLE_CLICK_DISTANCE_SQ` | 144 | 12 px squared threshold |
| `DRAG_THRESHOLD_SQ` | 16 | 4 px squared drag threshold |
| `SWIPE_THREE_FINGERS` / `SWIPE_FOUR_FINGERS` | 3 / 4 | Gesture finger counts |
| `DEFAULT_SWIPE_THRESHOLD` | 120 | Default displacement in px |
| `KEY_REPEAT_DELAY_MS` / `KEY_REPEAT_INTERVAL_MS` | 250 / 70 | Hold-to-navigate timing |

Other groups: `window.rs` holds `CORNER_RADIUS` 14, `TITLEBAR_HEIGHT` 34,
`TITLEBAR_BUTTON_RADIUS` 12, `BORDER_MARGIN` 8 and the minimum window size;
`rendering.rs` holds `MAX_BLUR_PASSES` 4, `BLUR_CACHE_LIFETIME` 600 s and
`MAX_BLUR_CACHE_BYTES` 96 MB.

Feature-only numbers stay next to their feature, for example the preview metrics
in `adapters/desktop/switcher/view.rs` and buffer cache keys in the launcher and
switcher modules.

## 4. Enums

All shared enums live in `libs/common/src/enums/` and are re-exported from the
crate root.

| Enum | Variants | Purpose |
| --- | --- | --- |
| `DockSide` | `Left`, `Right` | Which edge a dock attaches to |
| `Action` | `Close`, `Minimize`, `Maximize`, `Move` | Titlebar buttons and gestures |
| `SwipeDirection` | `Left`, `Right`, `Up`, `Down` | Gesture axis resolution via `from_delta` |
| `KeyboardAction` | quit, close, lock, reload, clipboard, launcher, screenshot, maximise, minimise, fullscreen, cycle focus, switcher step/cancel/commit | High-level actions from global shortcuts |

`Easing` is intentionally not in `common`: it is an animation primitive in
`libs/animation`.

## 5. Shared dock component

`libs/ui-kit/src/components/dock/` is split into `layout.rs` (capsule geometry,
hit testing, `visual_center`), `state.rs` (tweens, open/close, key repeat, cache
tick) and `draw.rs` (`draw_dock`). `mod.rs` re-exports the API and holds tests.

Launcher supplies app icons and the floating title capsule; switcher supplies
window badges and a small preview with a title strip. Surface, border, fillets,
shadow, animation, key repeat and wrap behaviour are identical for both.

## 6. Adding something new

- New duration: add a `Duration` constant in `libs/animation` and re-export it.
  Never put millisecond timing constants in `common`.
- New GTK frame-clock helper: add it to `libs/animation-gtk`.
- New geometry or threshold: add it to the matching file in `libs/common/src/constants/`.
- New shared enum: add a file under `libs/common/src/enums/` and re-export it.
- New dock behaviour: change `ui-kit/src/components/dock/` so both sides stay
  in sync; only side-specific content belongs in launcher or switcher.
