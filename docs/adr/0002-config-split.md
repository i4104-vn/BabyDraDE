# ADR 0002 — Split configuration across files

- Status: accepted
- Date: 2026-11-22

## Context

`~/.babydra/babydra.conf` contained every subsystem section. Saving any single
setting rewrote the entire file, and input hot paths re-read it on every event.

## Decision

Keep only main settings in `babydra.conf`: `1st`, `[autostart]`, `[power]`,
`[wallpaper]`, `[notification]`. Move subsystem settings into
`~/.babydra/configs/*.toml`, with `input.toml` grouping `[mouse]`, `[touchpad]`
and `[gestures]`. Introduce `services::config::store` for mtime-cached, atomic
per-file writes. Migrate legacy sections automatically at startup; a target
file always wins over a legacy value.

## Consequences

- Smaller, independently saved config files.
- Unknown top-level keys in `babydra.conf` survive saves.
- Input code uses snapshots (`BabydraState::gesture_config`) instead of reading
  disk in hot paths.
- See [docs/configuration.md](../configuration.md).
