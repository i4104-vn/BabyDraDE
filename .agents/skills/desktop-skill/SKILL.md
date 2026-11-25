---
name: desktop-skill
description: >-
  Desktop component and module development for BabyDraDE in Rust/GTK4.
  Activate when creating components or modules, refactoring, enforcing architecture,
  structuring animations, writing meaningful tests, or cleaning dead code.
---

# Desktop Development Skill

Enforces feature-oriented architecture, SOLID, DRY, unified animation drivers, purposeful testing, and automatic post-task cleanup in Rust/GTK4.

> Dynamic Island (topbar crate) reference: [dynamic-island.md](./references/dynamic-island.md) — load only when working on `crates/features/topbar/src/center/`.

---

## 1. Architecture — Feature-Oriented Zones

The window-manager crate (`crates/window-manager/src/`) uses 8 functional zones, max 3 directory levels:

| Zone | Purpose | Examples |
|:--|:--|:--|
| `core/` | Shared types, state, config, ports, shortcuts | `state.rs`, `ports.rs`, `config.rs` |
| `wayland/` | Protocol layer — runtime, protocols, clipboard | `runtime.rs`, `protocols.rs` |
| `window/` | Window lifecycle — registry, ops, hit_test | `registry.rs`, `ops.rs`, `interaction.rs` |
| `input/` | Device events — keyboard, pointer, touchpad, touch | `keyboard.rs`, `pointer.rs` |
| `render/` | Frame composition — titlebar, animation, blur | `titlebar.rs`, `blur/` |
| `shell/` | Desktop overlays — launcher, switcher, lock, screenshot | `launcher.rs`, `switcher.rs` |
| `backend/` | Hardware — winit, udev | `winit.rs`, `udev.rs` |
| `bootstrap/` | Startup — wiring, runtime, process | `wiring.rs`, `runtime.rs` |

**Dependency rule:** `bootstrap` → `shell`/`backend` → `render`/`input`/`window` → `wayland` → `core`. Inner zones never import outer.

**Other crates:** `crates/features/{desktop,greeter,topbar}`, `libs/{animation,animation-gtk,common,services,ui-kit}`.

## 2. File & Module Rules

| Rule | Detail |
|:--|:--|
| Capability module | One file under the zone root — never a wrapper folder |
| Allowed subfolder | `render/blur/` only (GPU pipeline + shaders) |
| Forbidden | `<name>.rs` beside `<name>/`, `#[path]`, folders with only thin `mod.rs` |
| Max file size | None. Related code may live in one long file |
| Visibility | `pub(crate)` by default; minimize public exposure |
| Naming | `snake_case` dirs/files, `PascalCase` types, `SCREAMING_SNAKE` constants |
| Function names | 1–3 words, `<verb>_<noun>` pattern. No redundant context from struct/module |

### Role-Named Files

| File | Contains | Must NOT contain |
|:--|:--|:--|
| `mod.rs` | Declarations, re-exports, short facades | Business logic, math, rendering |
| `ports.rs` | Narrow boundary traits (ISP) | Implementations |
| `state.rs` / `registry.rs` | Internal state, live-object lookup | UI code |
| `layout.rs` / `draw.rs` / `paint.rs` | Geometry, Cairo painting | State machines |
| `dispatch.rs` / `filters.rs` / `focus.rs` | Event routing, selection | Rendering |
| `render.rs` / `render/` | Assemble sub-views (UI modules) | Core logic |
| `service.rs` | External service integration | Widget code |
| `config.rs` | Configuration structs and defaults | Logic beyond validation |

## 3. SOLID & Code Quality

- **SRP:** 1 purpose per struct/function. Split if function > 30 lines or struct > 7 fields.
- **OCP:** Extend via traits, not modifying match branches.
- **LSP:** All trait impls fulfill full contract. No `unimplemented!()` / `todo!()` in production.
- **ISP:** Small focused traits. Split if consumers only need subsets.
- **DIP:** Depend on traits, not concrete types. Inject via constructors or generic bounds.

| Smell | Fix |
|:--|:--|
| Function > 3 params | Package into Context struct |
| Pass-through wrapper | Delete, call target directly |
| Logic duplicated ≥ 2× | Extract helper or shared trait |
| Inline constants | Move to `libs/common/` |
| Identical match branches | Move into enum method |

**Errors:** `thiserror` in libs, explicit error enums. Never `unwrap()`/`expect()` in runtime. Use `tracing` macros for logging.

**Docs:** Every function gets `///` with purpose, `# Arguments`, `# Returns`/`# Errors`.

## 4. Component Design (GTK4)

Structure order: Config/Props → Widget struct → Constructor → Public API.

- Connect signals **outside** constructor, in adapter/wiring layer.
- Use CSS classes (design tokens) instead of hardcoded values.
- Interactive states: Default, Hover (150ms ease-out), Active/Pressed, Disabled (opacity 0.4).

## 5. Animation Architecture

All animations live in `libs/animation/` (pure math) or `libs/animation-gtk/` (widget-specific):

```text
<animation_name>/
├── mod.rs       # Re-exports only
├── handler.rs   # Pure physics/math (no GTK/Smithay/Cairo)
└── render.rs    # Connects calculated values to visual output
```

- Presets (stiffness, damping, durations) → `libs/common/`.
- Features import from `libs/animation::<name>`. Never write ad-hoc physics in UI files.
- Durations: micro 120–180ms, popovers 200–280ms, workspace switch 300–350ms.

## 6. Testing

**Test:** State transitions, math/physics, parsing, error paths, business rules/invariants.

**Forbidden:** Getter/setter tests, constructor mapping, UI widget instantiation, mock parrots, type/enum variant assertions.

- Location: `#[cfg(test)] mod tests` in-file or `tests/` for integration.
- Naming: `should_<result>_when_<condition>` or `<action>_<scenario>`.
- One assertion per test.

## 7. Post-Implementation Checklist

After writing feature code, extract shared code to libs:

| Shared Type | Destination |
|:--|:--|
| Reusable UI widgets/renderers | `libs/ui-kit/` |
| System calls, D-Bus, device | `libs/services/` |
| Constants, global enums, math | `libs/common/` |
| Animation math & effects | `libs/animation/` |

Then run:

```bash
cargo test --workspace && cargo check && cargo clippy -- -D warnings
```

Remove duplicate structs, unused imports, and dead code. Verify zone dependency boundaries.
