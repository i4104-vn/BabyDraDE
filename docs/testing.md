# Testing

## Local checks

CI runs on every push and pull request with `fmt`, Clippy (with `-D warnings`),
workspace tests, `cargo deny` and the architecture test. Reproduce the same
gates locally before merging:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

`cargo test --workspace` also runs
`crates/window-manager/tests/architecture.rs`, so the architecture gate is
checked locally as well as in CI.

The `window-manager` crate follows [Clean Architecture](architecture.md).
Dependencies point inward (`composition → infrastructure → adapters →
application → domain`); keep new imports aligned with that rule. The decision
is recorded in [ADR 0003](adr/0003-clean-architecture.md). `domain` and
`application` are kept free of filesystem/process/toolkit effects so their
rules can be covered by fast unit tests.

## Architecture gate

`crates/window-manager/tests/architecture.rs` currently pins the known import
graph as a baseline (three adapter cycles) instead of claiming it is clean. The
baseline is expected to shrink during the refactor as each cycle is broken; new
outward imports or new edges between adapter areas fail the test. See
[report.md](../report.md) for the full plan.

## Coverage

`nextest.toml` configures deterministic `cargo nextest` runs (no retries).

| Area | Target |
| --- | --- |
| `domain` (pure rules) | 80% |
| `services::config` | 80% |
| `application` use cases | 80% |
| `adapters` pure logic | 60% |
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

DRM/GTK paths require manual runs on a machine with a GPU and a seat.
