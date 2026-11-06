//! BabyDraDE Domain Layer — Entities, Value Objects, Business Rules, Traits
//!
//! Pure domain logic with zero external runtime dependencies.

pub mod entities;
pub mod rules;
pub mod traits;
pub mod value_objects;

pub use entities::*;
pub use rules::*;
pub use traits::*;
pub use value_objects::*;
