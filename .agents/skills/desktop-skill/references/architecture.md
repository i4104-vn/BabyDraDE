# Architecture & Layering

Defines the dependency rules and structure for BabyDraDE crates.

## 1. Layer Rules

```text
composition/     -> Bootstrap, wiring, main.rs (imports all)
infrastructure/  -> I/O, D-Bus, Wayland backends (imports application, domain)
adapters/        -> GTK4 widgets, translators (imports application, domain)
application/     -> Use cases, state machines (imports domain only)
domain/          -> Pure entities, logic (stdlib only, imports nothing)
```

Rules:
- Inward dependencies only. Inner layers never import outer layers.
- `domain/` must have no GTK4, Smithay, or I/O dependencies.
- `adapters/` must not directly call `infrastructure/`.

## 2. File Layout

```text
src/
├── domain/
│   ├── mod.rs
│   ├── entities.rs
│   └── value_objects.rs
├── application/
│   ├── mod.rs
│   ├── use_cases.rs
│   └── ports.rs
├── adapters/
│   ├── mod.rs
│   ├── ui/
│   └── input_handler.rs
├── infrastructure/
│   ├── mod.rs
│   └── config_loader.rs
└── composition/
    └── mod.rs
```

## 3. Module Rules

- Re-export items only through `mod.rs`.
- Maximum 200 lines per file. Split if larger.
- Use `pub(crate)` by default; avoid public exposure when unnecessary.
