//! Pure rule models shared by the business-rule modules.

#[allow(unused_imports)]
pub(crate) mod policy;
pub(crate) mod shortcuts;

#[allow(unused_imports)]
pub(crate) use policy::GesturePolicy;
#[allow(unused_imports)]
pub(crate) use shortcuts::{Mods, ResolvedShortcut, Shortcut};
