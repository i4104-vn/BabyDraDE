//! Bus connection, request correlation and response decoding helpers.

mod error;
mod response;
mod state;

pub(crate) use error::io_error;
pub(crate) use response::{await_response, into_ack, into_entries, into_removal};
pub(crate) use state::{connect_bus, ensure_bus, lock_state, valid_owner, ClientState, WM_PEER};
