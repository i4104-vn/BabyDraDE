//! BabyDraDE Domain Layer — Entities, Value Objects, Business Rules, Traits
//!
//! Pure domain logic with zero external runtime dependencies.

pub mod entities;
pub mod value_objects;

pub use entities::*;
pub use value_objects::*;
