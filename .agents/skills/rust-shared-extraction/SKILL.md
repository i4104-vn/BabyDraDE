---
name: rust-shared-extraction
description: >-
  Extract shared code out of feature crates into the right library crate: enums
  and constants into libs/common, reusable UI interface/widget code into
  libs/ui-kit/components named after the component (not after the service).
  Use after a refactor to find duplicated parts across crates, move them into
  libs, then clean up comments and rustdoc. Pairs with rust-module-layout and
  rust-cleanup-checklist.
---

# Shared code extraction into libs

Apply this **after** the structural refactor is complete (see `rust-refactor-workflow`). Goal: one definition per concept, owned by the right library crate.

## 1. Where things belong

| Kind of code | Destination | Naming |
|---|---|---|
| Shared enums, value types, constants, protocol models | `libs/common/src/...` | Named after the domain concept (`PowerAction`, `PowerSource`, `MonitorOrientation`), not after the consuming service |
| Shared traits/ports used across crates | `libs/common/src/...` or the crate that defines the boundary | Named after the capability |
| Reusable UI widgets, panels, popovers, dialogs, render helpers | `libs/ui-kit/src/components/<component>/` | **Named after what the component is** (`file_chooser`, `context_menu`, `slider`, `capsule`), **not after the service that uses it** (`tray_widget`, `greeter_panel`) |
| Service implementations, system I/O, hardware drivers | stay in `libs/services` or the feature crate | — |

Anti-pattern: `libs/ui-kit/src/components/tray/` because only the topbar tray uses it. Correct: a generic `status_item_list` / `popover` component that the tray happens to compose.

## 2. Procedure

1. **Finish the structural refactor first.** Do not mix cross-crate moves with folder reshuffles — it makes verification noisy.
2. **Inventory duplicates.** Search for the same enum/const/component defined in more than one crate:
   ```bash
   grep -rnE '^(pub )?(enum|const|struct) ' crates libs --include='*.rs'
   ```
   Compare names and fields, not just identifiers. A copy with a different name is still a duplicate.
3. **Pick the owner by concept, not by current caller.** Ask: "would another crate reasonably need this?" If yes, it goes to `libs/common` or `libs/ui-kit`.
4. **Move the definition**, then update every call site. Keep the old path working with a re-export from the original module if other crates depend on it:
   ```rust
   // old location
   pub use common::power::PowerAction;
   ```
   A re-export is a compatibility path, not a duplicated definition — one source of truth remains.
5. **Add workspace dependency** in the consuming crate's `Cargo.toml` if needed (`common` / `ui-kit` are usually already present). Share versions through `[workspace.dependencies]`, never `"*"` or needless `"="`.
6. **Keep `libs` code library-grade**: derive `Debug`/`Clone`/`PartialEq`/`Eq`/`Default` where applicable, implement `Display`/`Error` for errors, document `# Errors`/`# Panics`, take `&str`/`&[T]`/`impl AsRef<...>` parameters, keep `Rc`/`RefCell`/`Arc` out of public signatures.

## 3. After the move — cleanup

- Delete the now-empty modules/folders in the feature crate (apply `rust-module-layout` rules again).
- Remove imports that became unused; avoid `#[allow(unused_imports)]` unless the item is genuinely test-only (prefer `#[cfg(test)]`).
- Replace any glob re-exports introduced by the move with explicit lists.
- Fix rustdoc links that now point to private or moved items → plain code spans or correct paths.
- **Delete redundant comments**: ones restating the identifier or the obvious, commented-out code, stale TODOs.
- **Keep** `//!`/`///`, `# Errors`/`# Panics`/`# Safety`, `// SAFETY:`, and "why" comments.
- **Write concise rustdoc**: first sentence ≈15 words, one line, says what the signature cannot. Do not restate the name. Example:
  ```rust
  /// Converts a configured orientation into the Wayland protocol transform.
  ```
  Not: `/// Converts orientation.` on `to_wl_transform`.

## 4. Verification (all must pass)

- `rust-module-layout` folder script → `ALL_COMPLIANT` in every touched crate
- `cargo fmt` on each touched crate
- `cargo clippy --all-targets --all-features -- -D warnings` per crate
- `cargo test` per crate — **same counts** as baseline
- `cargo doc --no-deps` per crate — no warnings
- `grep -rn 'pub use .*::\*'` → 0 (framework preludes exempt)
- `cargo check --workspace --all-targets` — every downstream crate still compiles
- Grep again for duplicate enum/const/component definitions — each concept now has exactly one owner

Report: what moved where, which old paths were preserved by re-export, and the verification results.
