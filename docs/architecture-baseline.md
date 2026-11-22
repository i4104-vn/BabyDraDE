# Architecture baseline

`xtask/baseline.toml` records existing architecture debt so `cargo xtask check`
fails only on **new** violations. Entries must be removed as the code is cleaned
up; do not add new ones without a linked issue and an expiry date.

Current debt (to be cleared phase by phase):

- Layer cycles: `compositor` ↔ `rendering`, `compositor` ↔ `shell` (extensions).
- Binary crate `pub` items in `window-manager`.
- Config/file/env/GTK access inside `compositor`, `input` and `rendering`.
- `unsafe` blocks without `// SAFETY` comments.

Ratchet rule: every refactor PR must remove at least one baseline entry or add
no new ones. The final target is an empty `xtask/baseline.toml`.
