# Post-Implementation Refactoring Checklist

Defines mandatory refactoring and extraction rules to execute immediately after completing code implementation, ensuring zero duplication and strict library centralization.

## 1. Extraction Targets for Shared Code

Any code, struct, or logic used across multiple modules or components must be extracted into its designated library under `libs/`:

| Shared Type | Destination | Contents & Responsibilities |
| :--- | :--- | :--- |
| **Shared Contexts** | `libs/context/` | Multi-module contexts, environment state, composite parameter structs (`AppContext`, `InputContext`, `ScreenContext`) and their reusable `impl` methods. |
| **Reusable UI** | `libs/ui-kit/` | Shared GTK4 widgets, reusable buttons, cards, popover containers, capsule wrappers, and universal render functions. |
| **System Services** | `libs/services/` | External integrations, D-Bus clients, NetworkManager, UPower, audio/pipewire interfaces, and background daemon connectors. |
| **Primitives & Config** | `libs/common/` | Global enums, configuration constants, domain error types, ID newtypes, and math/geometry primitives. |
| **Animations** | `libs/animation/` | Core physics/easing math and per-effect animation modules (`spring/`, `fade/`, etc.). |

## 2. Refactoring Step-by-Step Procedure

Immediately after writing feature code:

### Step 1: Scan for Parameter & Context Duplication
- Review newly created functions with > 3 parameters.
- Check if a context struct is defined locally in multiple modules.
- **Action**: Extract shared context structs and their common helper methods into `libs/context/`.

### Step 2: Scan for UI Widget Duplication
- Review new rendering logic in `render.rs` or `render/`.
- Check if visual patterns (buttons, pills, dialog containers, card styles) are duplicated or useful elsewhere.
- **Action**: Move common UI builders and render code into `libs/ui-kit/`.

### Step 3: Scan for System & Service Code
- Check if modules directly invoke D-Bus, system calls, or device querying.
- **Action**: Extract concrete service logic into `libs/services/`, exposing clean traits/facades.

### Step 4: Scan for Constants, Enums & Math
- Check for inline constants, status enums, or duplicated math helper functions.
- **Action**: Move constants and enums into `libs/common/`. Move animation math into `libs/animation/`.

### Step 5: Verification & Dead Code Removal
Run compiler and linter checks to catch unused code created during extraction:

```powershell
cargo check
cargo clippy -- -D warnings
```

- Delete unused local copies replaced by library imports.
- Delete unused `use` declarations.
- Verify inward-only dependency boundaries are maintained.

