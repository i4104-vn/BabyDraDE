# Development

## 1. Local checks

CI runs on every push and pull request. Reproduce the same gates locally before
merging:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

The `window-manager` package is binary-only. Use `cargo test -p window-manager`
to run both its unit tests and its integration tests; `cargo test --lib` does not
apply because there is no library target.

`nextest.toml` configures deterministic `cargo nextest` runs with no retries.
Prefer it when a flaky test needs to be identified.

## 2. CI workflow

| Job | Command | Purpose |
| --- | --- | --- |
| `fmt` | `cargo fmt --all -- --check` | Consistent formatting |
| `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` | Warning-free code |
| `test` | `cargo test --workspace` | Unit, integration and architecture tests |
| `deny` | `cargo deny check` | Dependency policy and licence checks |

## 3. Architecture gate

`crates/window-manager/tests/architecture.rs` is an executable description of
the boundary rules in [ARCHITECTURE.md](ARCHITECTURE.md). It checks that:

- `domain` and `application` do not import `adapters`, `infrastructure` or
  `composition`, and contain no Smithay, GTK, Wayland server, filesystem or
  process references;
- `application` has no global service locator (`OnceLock`, `static REGISTRY`);
- no source file hides dead imports with `#[allow(unused_imports)]`;
- a toplevel commit has exactly one owner;
- the cross-area adapter import graph equals the shrinking baseline.

The baseline currently contains six one-way edges and no cycles. To remove an
edge, first remove the import, then shrink the constant in the same commit. Never
add an edge to make a test pass.

## 4. Testing layout

Unit tests are inline `#[cfg(test)]` modules next to the code they cover.
Integration tests live in `crates/window-manager/tests/`. This keeps coverage
tools and IDEs working without `#[path]` indirection.

| Area | What is covered |
| --- | --- |
| `domain` | Shortcut matching, titlebar layout and hit testing, cursor mapping, pure geometry |
| `application` | Gesture intents, clipboard MIME choice, sizing policy, switcher scrubbing, autostart filtering, shortcut reload and launch errors with fakes |
| `adapters` | Protocol mapping, config parsing, render element construction, blur rules, window registry |
| Integration | Architecture boundary rules |

Current totals: 68 unit tests and 6 architecture tests for `window-manager`,
plus 24/3/2 tests in `ui-kit`, `babydra-topbar` and `animation`.

## 5. Coverage

`nextest.toml` drives deterministic runs. The targets below are iterative goals,
not yet enforced as a CI gate:

| Area | Target |
| --- | --- |
| `domain` | 80% |
| `services::config` | 80% |
| `application` use cases | 80% |
| `adapters` pure logic | 60% |
| Whole workspace | 50% |

Write characterization tests for the P0 flows before changing them further:
toplevel commit, sizing, focus/raise, input dispatch, switcher selection and
config reload. Raise the numbers gradually rather than setting an adapter-wide
gate too early.

## 6. Smoke matrix

| Scenario | Winit | Udev/DRM |
| --- | --- | --- |
| Startup, socket, autostart | yes | yes |
| Map/unmap/focus/raise | yes | yes |
| Move/resize (pointer and touch) | yes | yes |
| Maximise/minimise/fullscreen | yes | yes |
| Launcher / switcher | yes | yes |
| Blur (`blur_all` and rules) | yes | yes |
| Screenshot (single and multi-output) | yes | yes |
| Lock screen (PAM success/failure) | yes | yes |
| Clipboard history | yes | yes |
| Touch / touchpad gestures | partial | yes |
| VT switch, output hotplug | no | yes |

DRM and GTK paths require a machine with a GPU and a seat. Run the matrix before
a release; automated tests do not cover the framebuffer or PAM paths.

## 7. Keeping builds light

Dependencies are prebuilt in `target/`. When iterating on a single package,
prefer `cargo check -p window-manager` with a low job count over a workspace
build. Several agents can work in separate git worktrees while sharing a warm
target directory; set `CARGO_BUILD_JOBS=2` to avoid contention.

Do not reuse test binaries from a different worktree when validating a change.
Touch the test source or use a clean target so the architecture test actually
reads the current tree.
