# SOLID & Anti-Bloat Guidelines

## 1. SOLID in Rust

### S - Single Responsibility
- 1 purpose per struct or function.
- If a function exceeds 30 lines or a struct exceeds 7 fields, split it.

### O - Open/Closed
- Extend functionality by implementing traits, not modifying existing match branches.
- Use generics or trait objects instead of concrete type branching.

### L - Liskov Substitution
- All trait implementations must fulfill the full trait contract.
- Never use `unimplemented!()` or `todo!()` in production paths.

### I - Interface Segregation
- Keep traits small and focused.
- Split multi-method traits if consumers only need subsets of behavior.

### D - Dependency Inversion
- Depend on abstractions (traits), not concrete types.
- Inject dependencies via constructors or generic bounds.

## 2. Function Rules

### Concise & Direct Naming
- 1 to 3 words maximum. Focus directly on the core action.
- Pattern: `<verb>_<noun>` (e.g., `update_bounds()`, `render_border()`).
- Omit redundant context already clear from the struct or module (e.g., use `window.title()` instead of `window.get_current_active_window_title()`, `spring.step()` instead of `spring.step_physics_simulation()`).

### No Pass-Through / Forwarding Functions
- A function must contain real logic.
- Do not create a function that only calls another function with the same or forwarded arguments.
- If `foo()` only calls `bar()`, delete `foo()` and call `bar()` directly at the caller site.

### Context Structs for Many Parameters
- When a function needs > 3 parameters, create a dedicated Context struct (e.g., `RenderContext`, `ActionContext`).
- Implement helper methods directly on the Context struct so it can be reused across modules.

### Mandatory Rustdoc Documentation
- Every function must have documentation on top using `///`.
- Document purpose, `# Arguments`, and `# Returns` (or `# Errors` if returning `Result`).

## 3. Code Smells & Fixes

| Smell | Action |
| :--- | :--- |
| Long function name (> 3 words, redundant context) | Shorten to focused `<verb>_<noun>` or `<verb>` |
| Logic duplicated >= 2 times | Extract into helper function or shared trait |
| Function only forwards arguments | Delete wrapper function and call target directly |
| Function has > 3 parameters | Package into a Context struct and `impl` methods on it |
| Missing rustdoc header | Add `///` docs describing purpose, arguments, and returns |
| Structs sharing same fields | Extract trait or compose using newtypes |
| Identical match branches | Move branch logic into an enum method |

## 4. Limits & Standards

- Functions: <= 30 lines, <= 3 arguments (otherwise use Context), name <= 3 words.
- Structs: <= 7 fields.
- Files: <= 200 lines.
- Use `Option`/`Result` instead of sentinel values (-1, empty string).
- Use enums instead of boolean flag parameters.
- Define named constants instead of hardcoded numbers.

## 5. Error Handling & Logs

- In `libs/`: Use `thiserror`. Define explicit error enums.
- Never use `unwrap()` or `expect()` in runtime paths.
- Use `tracing` macros (`info!`, `warn!`, `error!`) for structured logging.
