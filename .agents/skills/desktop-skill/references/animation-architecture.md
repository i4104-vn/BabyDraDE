# Animation Architecture & Modular Structure

Defines the modular animation system in `libs/animation`. Every animation effect is an isolated, reusable module adhering to a uniform structure.

## 1. Modular Animation Directory Layout

Every individual animation must be located in its own folder under `libs/animation/` (or `libs/animation-gtk/` for widget-specific visual animations) with three mandatory files:

```text
libs/animation/
├── mod.rs                  # Re-exports all animation modules and driver traits
├── models/                 # Shared base traits, driver interfaces, and math configs
└── <animation_name>/       # E.g., spring/, fade/, slide/, scale/, bounce/
    ├── mod.rs              # Module entry point, public API, and re-exports
    ├── handler.rs          # Physics, formulas, calculations, and state stepping
    └── render.rs           # Converts calculated values into final visual effects/transforms
```

## 2. Mandatory File Roles per Animation Module

### `mod.rs` (Export & API)
- Declares `mod handler;` and `mod render;`.
- Re-exports the primary animation struct and high-level builder functions.
- Contains no calculation logic and no direct rendering code.

```rust
mod handler;
mod render;

pub use handler::{SpringCalculation, SpringState};
pub use render::SpringAnimation;
```

### `handler.rs` (Math & Calculation Engine)
- Contains pure calculation functions, interpolation formulas, and physics updates.
- Steps values over elapsed time (`dt`).
- Does not depend on UI toolkits (GTK, Smithay, or Cairo). Returns raw numbers, coordinates, or matrices.

```rust
/// Calculates the next spring step based on elapsed time and physics constants.
pub struct SpringCalculation {
    pub current: f64,
    pub target: f64,
    pub velocity: f64,
}

impl SpringCalculation {
    /// Steps the physics simulation.
    ///
    /// # Arguments
    /// * `dt` - Delta time in seconds.
    /// * `stiffness` - Spring stiffness coefficient.
    /// * `damping` - Damping coefficient.
    ///
    /// # Returns
    /// `true` if motion continues; `false` if settled.
    pub fn step(&mut self, dt: f64, stiffness: f64, damping: f64) -> bool {
        // Pure physics calculation here
        let force = -stiffness * (self.current - self.target);
        let damping_force = -damping * self.velocity;
        self.velocity += (force + damping_force) * dt;
        self.current += self.velocity * dt;
        (self.current - self.target).abs() > 0.001 || self.velocity.abs() > 0.001
    }
}
```

### `render.rs` (Visual Effect Assembly)
- Connects calculated values from `handler.rs` to visual outputs.
- Translates values into widget transformations (opacity, translation, scale, blur, clip rects).
- Drives frame callbacks and applies final styling or GTK tick updates.

```rust
use super::handler::SpringCalculation;

pub struct SpringAnimation {
    calc: SpringCalculation,
}

impl SpringAnimation {
    /// Applies calculated animation state to the target output.
    pub fn render_frame(&mut self, dt: f64) -> f64 {
        self.calc.step(dt, 200.0, 20.0);
        self.calc.current
    }
}
```

## 3. Presets & Centralization

- Core animation presets (stiffness, damping, duration constants) must live in `libs/common`.
- UI features import and use the animation module's facade from `libs/animation::<name>` or `libs/animation-gtk::<name>`.
- Never write ad-hoc physics or frame calculation loops inside UI component files.
