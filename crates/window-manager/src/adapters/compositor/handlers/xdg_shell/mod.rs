//! XDG shell protocol handling.

mod commit;
mod handler;
mod window_ops;

pub(super) use commit::handle_commit;
