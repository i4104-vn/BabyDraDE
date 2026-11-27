---
name: rust-cleanup-checklist
description: >-
  Concrete cleanup checklist for a Rust crate: remove glob re-exports and broken
  rustdoc links, delete redundant code and comments, fix pre-existing clippy lints,
  name magic constants, and keep behavior unchanged. Use when the user asks to
  tidy/clean/don dep Rust code, or as the final pass of a refactor started with
  rust-refactor-workflow / rust-module-layout.
---

# Rust cleanup checklist

Run this as a final pass on a crate. **Behavior, signatures, visibility and output bytes stay identical** unless the user asks otherwise.

## 1. Glob re-exports → explicit lists

Forbidden: `pub use foo::*;`. Replace with an explicit list of the actual public items:

```rust
// before
pub use meta::*;
// after
pub use meta::{merge_transient_metadata, parse_metadata, PlayerMeta};
```

If a glob re-exports a whole set of submodules, list the module names:

```rust
pub use modules::{backlight, bluetooth, clean, storage, vpn, wifi, volume};
```

**Allowed exception:** a framework's documented prelude, e.g. `use gtk4::prelude::*;`, `use bevy::prelude::*;`.

Verify: `grep -rn 'pub use .*::\*' src` → no results.

## 2. Broken rustdoc links → code spans

`cargo doc --no-deps` warns when public docs link to private items. These links are common after a module is made private or moved:

```rust
//! - [`animation`]: toggle and cancelable slide-out   // warning
//! - `animation`: toggle and cancelable slide-out     // clean
```

Same for `[`register`]`, `[`bind`]`, `[`state`]`, `[`index`]`. Do not make the item public just to satisfy a doc link.

Verify: `cargo doc -p <crate> --no-deps` emits no warnings.

## 3. Comments: delete what, keep why

**Delete:**
- comments that restate the identifier or the obvious (`// increment`, `// set value`, numbered step comments that just restate the code)
- commented-out code and stale/completed TODOs
- duplicate doc blocks left behind after a move

**Keep:**
- all `//!` module docs and `///` public docs (required by rust-best-practices §10)
- `# Errors`, `# Panics`, `# Safety` sections
- `// SAFETY:` comments
- comments explaining non-obvious *why*: quirks, workarounds, contract rationale, magic-value provenance, compatibility re-export rationale

If a doc is **inaccurate**, fix it rather than delete it (e.g. a function documented as returning geometry but actually returning CPU/RAM percentages).

## 4. Delete meaningless code

- dead imports, no-op statements, redundant `return`, redundant derefs
- `if x { true } else { false }` shapes
- unreachable clones that only satisfy the borrow checker (prefer `mem::take` / `Option::take`)
- `use super::*` globs outside tests → explicit imports, then prune parent imports that become unused

**Do not delete load-bearing code just because it looks odd.** A `clone()` needed for recursion, a double state initializer (early touch vs. worker use), or a `match true` guard may be deliberate. If unsure, leave it and report it.

## 5. Fix clippy lints, don't silence them

Run `cargo clippy --all-targets --all-features -- -D warnings`. Group repeats into one finding. Typical behavior-preserving fixes:

| Lint | Fix |
|---|---|
| `manual_clamp` | `x.min(hi).max(lo)` → `x.clamp(lo, hi)` |
| `derivable_impls` | hand-written `Default` with all-default fields → `#[derive(Default)]` |
| `items_after_test_module` | move items before the `#[cfg(test)] mod tests` block |
| `clippy::needless_borrow` | drop redundant `&` |

Use `#[expect(clippy::name, reason = "...")]` only when the lint is genuinely wrong for one item. Never `#![deny(warnings)]` in source; enforce with CI `-D warnings`.

## 6. Named constants

Replace magic numbers with a `const` plus a short doc comment explaining origin/effect:

```rust
/// Default cursor size used by Xcursor lookup.
const DEFAULT_CURSOR_SIZE: u32 = 24;
```

## 7. Naming — careful with public API

- Remove `get_` prefixes on **private/internal** functions.
- Leave public `get_*` functions alone if other crates call them — renaming is a breaking change.
- Leave weasel-suffix public types (`DriverManager`) unless a rename is explicitly requested.
- `bool` parameters in public signatures are a smell, but changing one breaks callers; report instead of changing.

## 8. Verification

Same gates as `rust-refactor-workflow`: fmt → clippy `-D warnings` → test count unchanged → doc clean → no pub globs → workspace check. Report suspicious code you deliberately kept.

Final step: run `rust-shared-extraction` — shared enums/constants belong in `libs/common`, reusable UI components in `libs/ui-kit/components` named after the component, and duplicated definitions get one owner. Then clean comments/rustdoc again in both the donor and destination crates.
