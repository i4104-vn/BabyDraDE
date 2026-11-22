# Architecture baseline

`xtask/baseline.toml` records architecture debt so `cargo xtask check` fails on
new violations. The file is currently **empty**: every check is clean.

| Check | Remaining entries |
| --- | ---: |
| Layering | 0 |
| Visibility | 0 |
| Hot path | 0 |
| Unsafe | 0 |

Rules:

- Do not add baseline entries. A PR that introduces a violation fails CI.
- Init-time I/O that is intentionally allowed is listed explicitly in
  `xtask/architecture.toml` under `hot_path_allow`, with the rationale that it
  runs once at startup or device hotplug, never per frame or per motion.
- The layering matrix in `xtask/architecture.toml` is the source of truth for
  allowed `crate::` edges.

See [Testing](testing.md) for the enforced checks and [ADRs](adr/) for the
decisions behind the structure.
