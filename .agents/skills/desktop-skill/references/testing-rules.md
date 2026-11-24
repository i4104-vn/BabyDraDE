# Meaningful Testing Rules

Defines standards for writing purposeful, high-value unit and integration tests. Bans trivial, zero-value tests.

## 1. Core Testing Philosophy

- Test real behavior and business logic only.
- Never write tests just to inflate line count or test count.
- If a test cannot realistically fail due to a logic regression, do not write it.

## 2. Forbidden Tests (Zero Value)

| Anti-Pattern | Description | Reason to Ban |
| :--- | :--- | :--- |
| **Getter/Setter Tests** | Testing that `foo.get_x()` returns `x` after `foo.set_x(x)`. | Tests compiler, not code logic. |
| **Constructor Field Mapping** | Testing that `Foo::new(a, b)` has `.a == a`. | Trivial mapping with zero regression risk. |
| **UI Widget Instantiation** | Calling `gtk4::Button::new()` just to assert it is not null. | Tests the GTK toolkit, not your code. |
| **Mock Parrots** | Mocks that strictly verify a function called another function without verifying real state changes. | Fragile, over-coupled, tests implementation details instead of outcomes. |
| **Type/Enum Variant Tests** | Asserting an enum has a specific variant. | Enforced by the Rust type system already. |

## 3. Mandatory Tests (High Value)

Write tests strictly for:

1. **State Transitions & Reducers**:
   - Given state A and action X, does state become B?
   - Edge cases: Empty lists, out-of-bounds inputs, boundary coordinates.
2. **Mathematical & Physics Calculations**:
   - In `libs/animation` (e.g., `handler.rs`): Spring step calculations, settling conditions, damping decay over `dt`.
3. **Parsing & Formatting**:
   - Converting raw strings/configs into domain structs.
   - Handling invalid/malformed inputs gracefully without panicking.
4. **Error Pathways**:
   - Verifying that errors return expected `Result::Err` variants rather than panicking.
5. **Business Rules & Invariants**:
   - Window tiling layout calculations, screen boundary clamping, Z-order reordering.

## 4. Test Structure & Naming

- Location: In the same file using `#[cfg(test)] mod tests`, or in a crate-level `tests/` directory for integration tests.
- Naming: `should_<expected_result>_when_<condition>` or `<action>_<scenario>`.
- Keep tests focused: Assert 1 behavior per test case.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_settle_when_delta_below_epsilon() {
        let mut spring = SpringCalculation { current: 1.0, target: 1.0, velocity: 0.0 };
        let active = spring.step(0.016, 200.0, 20.0);
        assert!(!active, "Spring should settle when current equals target and velocity is zero");
    }

    #[test]
    fn should_return_err_when_config_is_empty() {
        let result = parse_layout_config("");
        assert!(matches!(result, Err(ConfigError::Empty)));
    }
}
```

## 5. Verification Command

Run meaningful tests efficiently:

```powershell
cargo test --workspace
```

