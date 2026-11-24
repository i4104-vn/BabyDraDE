# File Naming & Module Code Structure

Defines uniform file naming, module layouts, and file roles across all modules in BabyDraDE.

## 1. Naming Conventions

- Directories & Files: `snake_case` (e.g., `window_manager/`, `render.rs`, `spring/`).
- Structs, Enums, Traits: `PascalCase` (e.g., `TopBar`, `ButtonConfig`, `SpringAnimation`).
- Constants & Enums: Placed in `libs/common`. Named with `SCREAMING_SNAKE_CASE`.
- Context Structs: Suffix with `Context` (e.g., `ModuleContext`).

### Function Naming Rules: Concise & Direct
- Format: `snake_case`.
- Length: 1 to 3 words maximum. Focus directly on the action.
- Pattern: `<verb>_<noun>` (or just `<verb>` / `<noun>` when context is obvious from the struct or module).
- Do not repeat struct or module names in function names.

| Bad (Too Long / Redundant) | Good (Concise & Focused) |
| :--- | :--- |
| `calculate_and_update_surface_bounds_rect()` | `update_bounds()` |
| `window.get_current_active_window_title()` | `window.title()` |
| `spring_animation.step_physics_simulation_tick()` | `spring.step()` |
| `render_button_background_and_border_style()` | `render_border()` |
| `convert_input_event_to_domain_action()` | `to_action()` |
| `check_is_surface_valid_and_visible()` | `is_visible()` |

## 2. Standard Non-UI Module Directory Structure

For any functional or backend module (e.g., state management, service integration, input handling):

```text
<module_name>/
├── mod.rs          # Entry point, module declarations, public re-exports
├── context.rs      # Context struct and its method implementations
├── handler.rs      # Business logic, event processing, state manipulation
└── models/         # All structs, traits, and impl blocks for this module
    ├── mod.rs      # Re-exports all models
    ├── types.rs    # Core structs and types
    └── traits.rs   # Behavior traits
```

## 3. Standard UI Component Directory Structure

For dedicated UI components (e.g., inside `libs/ui-kit/` or feature crates):

```text
<component_name>/
├── mod.rs          # Component entry point, declares modules, public exports
├── render.rs       # Assembles sub-UI elements from utils/ into final component
├── context.rs      # Context struct and method impls for complex state
├── models/         # Structs, traits, and configs specific to this component
│   └── mod.rs
└── utils/          # Sub-widgets, small UI builders, helpers
    ├── mod.rs
    ├── sub_widget.rs
    └── style.rs
```

## 4. Standard Animation Module Directory Structure

Every individual animation in `libs/animation/` must be an isolated folder with 3 files:

```text
<animation_name>/   # E.g., spring/, fade/, slide/, scale/
├── mod.rs          # Module entry point, public API, re-exports
├── handler.rs      # Mathematical calculations, physics, interpolation
└── render.rs       # Assembles values into the finished visual effect
```

## 5. Mandatory File Roles

### `mod.rs` (Export & Facade)
- Declares submodules and re-exports public items.
- Must not contain business logic, math calculations, or UI assembly code.

### `context.rs` (State & Parameter Bundling)
- Bundles parameters when functions exceed 3 arguments.
- Implements helper methods directly on the Context struct for reuse across handlers.

### `handler.rs` (Execution, Math & Logic)
- Implements business logic, event loops, or mathematical/physics calculations.
- Must not mix UI rendering code.

### `render.rs` (Visual Effect Assembly)
- UI components: Assembles sub-views from `render/` or `utils/` into the root widget.
- Animation modules: Converts raw values from `handler.rs` into visual effects (transforms, opacity, scale).

### `models/` (Data & Contracts)
- Holds all module-specific structs, trait definitions, and their implementations.
- Subdivided into clear files (e.g., `types.rs`, `traits.rs`) with clean re-exports via `models/mod.rs`.

## 6. Centralization Rules

- **Constants & Global Enums**: Move to `libs/common`. Never duplicate or scatter configuration constants across individual feature files.
- **Animations**: Move all animation math and drivers to `libs/animation`. Each animation has its own folder with `mod.rs`, `handler.rs`, and `render.rs`.
