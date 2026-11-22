# Testing and enforcement

## Automated checks

Every PR runs the CI workflow in `.github/workflows/ci.yml`:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask check
```

`cargo xtask check` enforces four architecture fitness functions:

| Check | Rule |
| --- | --- |
| `check-layering` | No `use crate::<layer>` edge outside `xtask/architecture.toml`. |
| `check-visibility` | Binary crate exposes no `pub` items. |
| `check-hot-path` | `compositor`, `input`, `rendering` contain no file/env/config-I/O. |
| `check-unsafe` | Every `unsafe` block has a `// SAFETY` comment. |

Existing debt is recorded in `xtask/baseline.toml`. Regenerate it with
`cargo xtask baseline` only when intentionally accepting new debt; each PR
should remove at least one entry.

## Coverage

`nextest.toml` configures deterministic `cargo nextest` runs (no retries).
Once `cargo-llvm-cov` is available in CI, the target thresholds are:

| Area | Target |
| --- | --- |
| `model` | 80% |
| `services::config` | 80% |
| `input` / `rendering` pure logic | 60% |
| Whole workspace | 50% |

## Smoke matrix

| Scenario | Winit | Udev/DRM |
| --- | --- | --- |
| Startup, socket, autostart | yes | yes |
| Map/unmap/focus/raise | yes | yes |
| Move/resize (pointer + touch) | yes | yes |
| Maximize/minimize/fullscreen | yes | yes |
| Launcher / switcher | yes | yes |
| Blur (`blur_all` and rules) | yes | yes |
| Screenshot (single + multi-output) | yes | yes |
| Lock screen (PAM success/fail) | yes | yes |
| Clipboard history | yes | yes |
| Touch / touchpad gestures | partial | yes |
| VT switch, output hotplug | no | yes |

Manual runs are still required for DRM/GTK paths until a runner with a GPU and
a seat is available in CI.
