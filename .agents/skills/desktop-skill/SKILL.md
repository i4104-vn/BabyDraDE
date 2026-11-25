---
name: desktop-skill
description: >-
  Desktop component and module development for BabyDraDE in Rust/GTK4.
  Activate when creating components or modules, refactoring, enforcing architecture,
  structuring animations, writing meaningful tests, or cleaning dead code.
---

# Desktop Development Skill

Enforces layered clean architecture, SOLID principles, DRY, uniform module structure, unified animation drivers, purposeful testing, and automatic post-task cleanup in Rust.

## References

- Meaningful testing rules: [testing-rules.md](./references/testing-rules.md)
- Post-refactor checklist & library extractions: [post-refactor-checklist.md](./references/post-refactor-checklist.md)
- Module layout & file naming: [file-naming-structure.md](./references/file-naming-structure.md)
- Unified animation architecture: [animation-architecture.md](./references/animation-architecture.md)
- Architecture & layers: [architecture.md](./references/architecture.md)
- SOLID & anti-bloat: [solid-guidelines.md](./references/solid-guidelines.md)
- Component rules: [component-design.md](./references/component-design.md)
- Dynamic Island architecture: [dynamic-island-architecture.md](./references/dynamic-island-architecture.md)
- Adding an island feature: [adding-island-feature.md](./references/adding-island-feature.md)

## Workflow

### 1. Pre-Implementation
- Identify target layer (`domain`, `application`, `adapters`, `infrastructure`, `composition`).
- Check existing libraries in `libs/` (`ui-kit`, `context`, `services`, `common`, `animation`) before writing new code.
- Apply standardized file layouts:
  - Non-UI Module: `mod.rs`, `context.rs`, `handler.rs`, `models/`.
  - UI Module: `mod.rs`, `context.rs`, `handler.rs`, `render.rs`, `render/`, `models/`.
  - Animation Module: `mod.rs`, `handler.rs`, `render.rs`.

### 2. Implementation
- Module File Roles:
  - `mod.rs`: exports and facades only.
  - `handler.rs`: core business/event/math logic.
  - `context.rs`: bundled parameters and context methods.
  - `models/`: all structs, traits, and implementations.
  - `render.rs` & `render/`: assemble sub-views (UI modules only).
- Naming: Function names 1-3 words, concise `<verb>_<noun>`.
- Documentation: Every function must have standard Rust doc comments (`///`) with purpose, `# Arguments`, and `# Returns`.
- No Pass-Through Functions: Eliminate wrappers that only forward arguments.

### 3. Purposeful Testing
Follow [testing-rules.md](./references/testing-rules.md):
- Test real logic, calculations, state transitions, edge cases, and error branches.
- Strictly forbidden: Trivial getter/setter tests, constructor mapping checks, and dummy UI instantiation tests.

### 4. Post-Implementation Refactoring & Cleanup
Follow [post-refactor-checklist.md](./references/post-refactor-checklist.md) to extract shared elements:
- Shared Contexts (> 3 params or multi-module use) -> `libs/context/`.
- Reusable UI widgets & renderers -> `libs/ui-kit/`.
- System calls, D-Bus, device integrations -> `libs/services/`.
- Constants and global enums -> `libs/common/`.
- Animation math & effects -> `libs/animation/`.

Run validation:

```powershell
cargo test --workspace
cargo check
cargo clippy -- -D warnings
```

Required actions:
- Verify extractions to `libs/` are completed.
- Remove duplicate structs, unused imports, and dead code.
- Report all tested, refactored, and cleaned items.
