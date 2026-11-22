# Architecture baseline

`xtask/baseline.toml` records existing architecture debt so `cargo xtask check`
fails only on **new** violations. Entries must be removed as the code is cleaned
up; do not add new ones without a linked issue and an expiry date.

## Current debt (after phases 0–5)

| Check | Remaining entries | Notes |
| --- | ---: | --- |
| Layering | 12 | Mostly shell overlay rendering in backends/input and rendering types held by compositor state. |
| Visibility | 67 | Binary crate still exposes `pub` items; phase 5 can convert them to `pub(crate)`. |
| Hot path | 9 | Config/GTK access at startup paths, not in per-event loops (gesture hot path is fixed). |
| Unsafe | 39 | `// SAFETY` comments still to be written, mostly in PAM, GL and EGL code. |

Ratchet rule: every refactor PR must remove at least one baseline entry or add
no new entries. The final target is an empty `xtask/baseline.toml`.

## Deliberately deferred

- Full `babydra-shell` process split: GTK UI still shares the compositor
  process for launcher/switcher/render elements, behind explicit module
  boundaries. Lock, screenshot editor and clipboard picker already use threads
  or subprocesses.
- Complete `InputState` / `WindowStore` / `RenderState` split: `BabydraState`
  still owns those fields, but config snapshots and metadata are now separated.
- Ratcheting `unsafe_op_in_unsafe_fn` from `warn` to `deny`: blocked on wrapping
  existing `unsafe fn` bodies in explicit `unsafe` blocks.

See [Testing](testing.md) for the enforced checks and [ADRs](adr/) for the
decisions behind the current structure.
