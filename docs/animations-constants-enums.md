# Animations, Constants and Enums

This document describes where animation logic, numeric tokens and shared
enumerations live, how the frame loop drives them, and how the two screen-edge
docks (launcher and switcher) reuse the same implementation.

## 1. Ownership map

| Location | Responsibility |
| --- | --- |
| `libs/animation/` | Clock-driven primitives (`Tween`, `Easing`), frame pacing helpers, **all animation durations** |
| `libs/common/src/constants/` | Geometry, input thresholds, colour tokens, layout numbers |
| `libs/common/src/enums/` | Shared enumerations (`Action`, `SwipeDirection`, `KeyboardAction`, `DockSide`) |
| `libs/ui-kit/src/drawing/cairo.rs` | Shared Cairo primitives (rounded paths, text, app colours, docked-capsule path/fill) |
| `libs/ui-kit/src/components/dock/` | Shared screen-edge dock: layout, state machine, rendering |
| `crates/window-manager/src/extensions/launcher/` | Left dock: app list, title capsule, backdrop |
| `crates/window-manager/src/extensions/switcher/` | Right dock: live preview + title strip, backdrop |
| `crates/window-manager/src/backends/` | Render loops (udev timer, winit redraw) and frame scheduling |

Rule of thumb: **durations belong to `animation`, geometry/thresholds belong to
`common`, behaviour belongs to the owning component.**

## 2. Animation system

### 2.1 Primitives (`libs/animation`)

`Tween` stores `started_at`, `duration`, `from`, `to` and an `Easing`. Values are
evaluated from `Instant::now()`, so motion is **frame-rate independent**:
a 200 ms transition looks identical at 60 Hz and 120 Hz. Key methods:

- `value()` / `value_at(instant)` — interpolated value.
- `retarget(to, duration, easing)` — restarts from the current value.
- `is_animating()` / `frame_tick()` — cache invalidation and render scheduling.
- `frame_index()` — the same progress quantised to the 120 fps budget, useful
  when a stable per-frame key is preferred over a microsecond tick.

`Easing` is defined in `libs/animation/src/curves/` (not in `common`) and
provides Linear, Quad, Cubic, Sine, Quart, Quint, Expo, Circ, Back and Spring
curves. The UI mostly uses `EaseOutCubic` (single steps),
`EaseInOutCubic` (fades) and `Linear` (held-key repeats).

Frame helpers in `libs/animation/src/time/frame.rs`:

- `frame_index_since(start)` — monotonic index, one step per `FRAME_INTERVAL`.
- `bounded_delta(&mut last)` — delta since last call, capped at four frame
  budgets so a long stall cannot produce an oversized step.

### 2.2 Frame pacing

- **Udev/DRM**: a calloop `Timer` reschedules with
  `TimeoutAction::ToInstant(deadline + FRAME_INTERVAL)`, i.e. from the previous
  deadline rather than from render completion. A slow frame therefore cannot
  permanently drift the 120 fps cadence.
- **Winit**: the window reports `TARGET_FPS * 1000` as its refresh rate and
  calls `request_redraw()` every frame.
- **GTK overlays** (wallpaper, lock): `add_tick_callback` is vsync-synchronised,
  so on a 120 Hz output it fires at 120 fps.

Every compositor frame calls `update_window_animations()`, which advances window
animations, ticks the switcher key-repeat timer and finalises a closing
switcher once its presence tween finishes.

### 2.3 Duration catalogue (`libs/animation/src/time/constants.rs`)

| Constant | Value | Used for |
| --- | --- | --- |
| `TARGET_FPS` / `FRAME_INTERVAL` | 120 / ~8.33 ms | Frame budget |
| `DEFAULT_DURATION` | 420 ms | Generic UI transitions |
| `NAVIGATION_DURATION` | 200 ms | Single arrow-key / hover move in a dock |
| `SCROLL_DURATION` | 260 ms | Wheel / touchpad dock scroll |
| `FADE_IN_DURATION` / `FADE_OUT_DURATION` | 320 / 260 ms | Launcher open/close presence |
| `REVEAL_DURATION` | 900 ms | Wallpaper circular reveal |
| `WINDOW_OPEN_DURATION` | 280 ms | Window map animation |
| `WINDOW_CLOSE_DURATION` | 260 ms | Window unmap animation |
| `WINDOW_MINIMIZE_DURATION` | 280 ms | Minimize shrink |
| `WINDOW_RESTORE_DURATION` | 300 ms | Restore grow |
| `WINDOW_GEOMETRY_DURATION` | 340 ms | Maximise / fullscreen transitions |
| `STAGE_ANIM_DURATION` | 500 ms | Legacy switcher stage pulse |
| `DOCK_TARGET_ANIM_DURATION` | 420 ms | Legacy dock highlight frame |
| `SWITCHER_PREVIEW_DURATION` | 180 ms | Switcher presence + preview fade |
| `SWITCHER_SCROLL_DURATION` | 160 ms | Single Tab step |

All are `std::time::Duration` values; callers pass them directly to
`Tween::retarget`/`Tween::new` instead of converting milliseconds at the call
site.

### 2.4 Dock animation state machine

`DockState` (`ui-kit/src/components/dock/state.rs`) is shared by both docks. It
holds:

- `position: Tween` — continuous (fractional) selection index.
- `presence: Tween` — 0 = hidden, 1 = docked; drives fade **and** slide.
- `selected: Option<usize>` — logical selection used by the caller.
- `closing: bool` — keeps the overlay alive until the fade completes.
- held-key tracking (`held_direction`, `repeat_started`, `last_repeat`).

Transitions:

1. `open(index, animate)` snaps `position` to `index` and fades `presence` to 1
   with `FADE_IN_DURATION` / `EaseOutCubic`.
2. `begin_close()` fades `presence` to 0 with `FADE_OUT_DURATION`; the state is
   removed only after `presence.is_animating()` becomes false.
3. `slide_offset(side)` returns `-(1 - presence) * PANEL_WIDTH` on the left dock
   and `+(1 - presence) * PANEL_WIDTH` on the right dock, so the capsule slides
   in from / out to its screen edge. The composited dock group is painted with
   `paint_with_alpha(presence)`, giving a simultaneous slide + fade.
4. `move_selection` / `scroll_by` retarget `position` with a smooth curve.
5. Held keys: `key_pressed` starts a timer (backend auto-repeat presses are
   ignored). On every frame `tick_key_repeat(count)` fires after
   `KEY_REPEAT_DELAY_MS` and then every `KEY_REPEAT_INTERVAL_MS`, calling
   `repeat_move` — a short **70 ms Linear** step. Consecutive steps chain into a
   continuous glide instead of restarting a long easing curve and “rushing” at
   the end.

Both docks snap straight to the first/last entry when a step crosses a list
boundary (`step_target` in `DockState`, plus the switcher's Tab repeat helper),
avoiding an empty scroll before the wrap. For long lists `visual_center` clamps
the scrolled content to `[half, count - 1 - half]`, so the first icon starts at
the top of the capsule and the last one ends at the bottom while the selection
still loops.

Pointer hover is deliberately kept **separate** from the scrolling tween:
`DockState::hovered` (and `AltTabSwitcher::hovered`) only move the highlight and
never retarget `position`. Otherwise moving the cursor changes the carousel
offset, which changes hit-testing, which moves the cursor target again — a
feedback loop that made the highlight jump around. Pointer motion and touch
motion both call `update_hover`; a touch tap on an icon launches/focuses it and
a tap outside dismisses the overlay.

### 2.5 Cache invalidation

Both overlays rasterise into small off-screen buffers. While a tween is
animating they use `frame_tick()` (microsecond tick since the tween started) as
part of the cache key; this guarantees a fresh buffer on every render even if
the event loop runs faster than 120 fps. When all tweens rest the key becomes
`u64::MAX`, so static frames are cached.

The backdrop buffers are keyed by physical size plus the presence tick. The
udev switcher backdrop is mirrored (`render_right_backdrop_pixels`): its
gradient spans `0..gradient_w` with the darkest stop at the **right** edge.

### 2.6 Other animations

- **Wallpaper reveal**: a `Tween` created with `REVEAL_DURATION` and
  `EaseInOutCubic`, sampled in a GTK `add_tick_callback`. The wallpaper bitmap is
  cached; each frame only changes the Cairo clip radius.
- **Lock screen fades**: `Tween` + `add_tick_callback` with `EaseInOutCubic`
  (200 ms / 240 ms), replacing fixed 8.33 ms timers.
- **Window animations**: Tween-driven open/close/minimize/restore/geometry
  transitions using the `WINDOW_*_DURATION` constants above.

## 3. Constants

### 3.1 `libs/common/src/constants/launcher.rs` — shared dock tokens

| Constant | Value | Meaning |
| --- | --- | --- |
| `PANEL_X` / `PANEL_WIDTH` | 0.0 / 72.0 | Capsule offset from its edge and width |
| `PANEL_CORNER_RADIUS` / `PANEL_FILLET_RADIUS` | 24 / 20 | Convex outer corners / concave edge fillets |
| `PANEL_PAD` | 24 | Vertical padding above and below slots |
| `ICON` / `ICON_SELECTED` | 30 / 42 | Regular and selected icon size |
| `SLOT` | 50 | Distance between icon centres |
| `VISIBLE_SLOTS` | 9 | Maximum icons rendered in the capsule |
| `SELECTION_SIZE` | 48 | Highlight behind the selected icon |
| `INDICATOR_WIDTH` / `INDICATOR_HEIGHT` | 3 / 16 | White selection pill |
| `LABEL_GAP`, `LABEL_HEIGHT`, `LABEL_PAD`, `LABEL_FONT_SIZE` | 10, 32, 12, 13.5 | Launcher title capsule |
| `LABEL_MAX_TEXT` / `LABEL_MAX_WIDTH` | 190 / 214 | Ellipsis thresholds |
| `SHADOW_PAD` | 30 | Off-screen buffer margin for the shadow |
| `BACKDROP_WIDTH_MIN` / `BACKDROP_WIDTH_RATIO` | 600 / 0.60 | Backdrop gradient size |
| `PANEL_SURFACE` | rgba(14,14,18,0.96) | Capsule fill, matches topbar |
| `HAIRLINE` | rgba(255,255,255,0.12) | 1 px border |

### 3.2 `libs/common/src/constants/input.rs`

| Constant | Value | Meaning |
| --- | --- | --- |
| `BTN_LEFT` / `BTN_RIGHT` / `BTN_MIDDLE` | 272 / 273 / 0x112 | evdev button codes |
| `DOUBLE_CLICK_TIMEOUT_MS` | 350 | Maximum gap between clicks |
| `DOUBLE_CLICK_DISTANCE_SQ` | 144 | 12 px squared threshold |
| `DRAG_THRESHOLD_SQ` | 16 | 4 px squared drag threshold |
| `SWIPE_THREE_FINGERS` / `SWIPE_FOUR_FINGERS` | 3 / 4 | Gesture finger counts |
| `DEFAULT_SWIPE_THRESHOLD` | 120 | Default displacement in px |
| `KEY_REPEAT_DELAY_MS` / `KEY_REPEAT_INTERVAL_MS` | 250 / 70 | Hold-to-navigate timing (used by `DockState` and switcher Tab repeat) |

### 3.3 Other `common` groups

- `window.rs`: `CORNER_RADIUS` 14, `TITLEBAR_HEIGHT` 34,
  `TITLEBAR_BUTTON_RADIUS` 12, `BORDER_MARGIN` 8, `WINDOW_MIN_WIDTH` 150,
  `WINDOW_MIN_HEIGHT` 100.
- `switcher.rs`: legacy Stage Manager layout numbers (`DOCK_X` 24, margins 32,
  thumbnail 175×115, gap 12, stage gaps/aspect, `STAGE_SLIDE_DISTANCE` 24,
  `STAGE_SCALE_GROW` 0.02). The current right dock itself uses the launcher
  tokens above.
- `rendering.rs`: `MAX_BLUR_PASSES` 4, `BLUR_CACHE_LIFETIME` 600 s,
  `MAX_BLUR_CACHE_BYTES` 96 MB, fallback cursor 16×20.

### 3.4 Local constants

Feature-only numbers that are not shared stay next to their feature: preview
size/gap/title metrics in
`crates/window-manager/src/extensions/switcher/view.rs`, and buffer/backdrop
cache keys in the launcher/switcher modules.

## 4. Enums

All shared enums live in `libs/common/src/enums/` and are re-exported from the
crate root (`common::…`).

| Enum | Variants | Purpose / consumer |
| --- | --- | --- |
| `DockSide` | `Left`, `Right` | Which screen edge a dock attaches to. Used by `ui_kit::drawing::draw_docked_panel`, `DockLayout`, `DockState::slide_offset`, launcher (Left) and switcher (Right). |
| `Action` | `Close`, `Minimize`, `Maximize`, `Move` | Titlebar buttons and gesture actions. |
| `SwipeDirection` | `Left`, `Right`, `Up`, `Down` | Touchpad/touch gestures; `from_delta(dx, dy, threshold)` resolves the dominant axis. |
| `KeyboardAction` | `QuitCompositor`, `CloseActiveWindow`, `LockScreen`, `ReloadConfig`, `ClipboardPicker`, `ToggleLauncher`, `Screenshot`, `ToggleMaximize`, `Minimize`, `ToggleFullscreen`, `CycleWindowFocus { forward }`, `AltTabStep { backwards }`, `AltTabCancel`, `AltTabCommit` | High-level actions produced by global shortcut matching and executed by the compositor. |

`Easing` is intentionally **not** in `common`: it is an animation primitive and
lives in `libs/animation/src/curves/`.

## 5. Shared dock component

`libs/ui-kit/src/components/dock/` is split into small files:

- `mod.rs` — re-exports (`DockLayout`, `DockState`, `DockSide`, `draw_dock`,
  `visual_center`) and unit tests.
- `layout.rs` — capsule rectangle, pointer hit-test, icon geometry,
  `visual_center` (short lists stay still; long lists scroll between clamped
  ends).
- `state.rs` — selection/presence tweens, open/close, held-key repeat, cache tick.
- `draw.rs` — `draw_dock`: draws the capsule, selection highlight, items via a
  caller-supplied closure, and the inner white pill; applies the presence fade
  with a Cairo group.

Launcher supplies app icons and the floating **title** capsule; switcher
supplies window badges and the small **preview** with its bottom title strip.
Everything else — surface, border, fillets, shadow, animation, key repeat and
wrap behaviour — is identical.

## 6. Adding something new

- New duration: add a `Duration` constant in
  `libs/animation/src/time/constants.rs` and re-export it in `time/mod.rs` and
  `lib.rs`. Never add millisecond constants to `common` for animation timing.
- New geometry/threshold token: add it to the matching file in
  `libs/common/src/constants/`.
- New shared enum: add a file under `libs/common/src/enums/` and re-export it
  from `enums/mod.rs`.
- New shared drawing primitive: add it to `libs/ui-kit/src/drawing/cairo.rs`.
- New dock behaviour: change `ui-kit/src/components/dock/` so both sides stay
  in sync; only side-specific content belongs in the launcher/switcher crates.
