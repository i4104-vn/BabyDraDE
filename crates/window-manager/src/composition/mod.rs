//! Application composition root: lifecycle, bootstrap and subcommand dispatch.

pub(crate) mod bootstrap;
pub(crate) mod commands;
pub(crate) mod input_method;
pub(crate) mod runtime;
pub(crate) mod wiring;

pub(crate) use runtime::run;
