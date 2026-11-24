# Layer and module rules

Short reference for the rules enforced in this repository. See
[ARCHITECTURE.md](ARCHITECTURE.md) for the rationale and the state/component
design.

## Layers

```text
composition → infrastructure → adapters → application → domain
```

Dependencies point inward only; inner layers never import outer ones.

- `domain/` holds pure entities and logic. No GTK4, Smithay, Wayland, filesystem
  or process code.
- `application/` holds use cases and ports, and imports `domain` only.
- `adapters/` holds GTK4 widgets and translators; it imports `application` and
  `domain` but must not call `infrastructure/` directly. Within adapters the
  order is `window → input → rendering → desktop → compositor`.
- `infrastructure/` holds backends, config loading and process launching.
- `composition/` wires everything together.

`scripts/check-layers.sh` checks these imports mechanically over
`crates/window-manager/src`, plus a 200-line cap per `.rs` file.

## Module file layout

Per the desktop skill, each module uses a uniform layout:

| File | Role |
| --- | --- |
| `mod.rs` | Declarations and re-exports only; no logic. |
| `handler.rs` | Business logic, event handling and math. |
| `context.rs` | Bundled parameters and reusable context methods. |
| `models/` | All module structs, traits and impls (`types.rs`, `traits.rs`). |
| `render.rs` / `render/` | Assemble views or animation values (UI/animation modules only). |

Non-UI modules use `mod.rs`, `context.rs`, `handler.rs` and `models/`. UI modules
also use `render.rs`/`render/`. Each animation in `libs/animation/` is a folder
with `mod.rs`, `handler.rs` and `render.rs`.

The size and shape limits from the skill are:

- Files: at most **200 lines**.
- Functions: at most **30 lines** and at most **3 arguments**; use a
  `*Context` struct beyond that. Names are 1–3 words (`<verb>_<noun>`).
- Structs: at most **7 fields**; split otherwise.

The 200-line file cap is enforced by the script below; the function and struct
limits are review standards, not script-enforced.

## Running the guardrail

From the repository root:

```sh
scripts/check-layers.sh
```

It runs in CI on every push and pull request (see
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml)). Every file must be
at most 200 lines; the script fails on any violation.
