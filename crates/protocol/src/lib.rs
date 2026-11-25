//! Typed bridge protocol between BabyDraDE features and the window manager.
//!
//! Feature apps and the compositor are separate processes. This crate is the
//! contract both sides link against: a versioned envelope carrying a
//! [`Request`], [`Response`] or [`Event`], encoded as one JSON line and moved by
//! a pluggable [`transport::Transport`].
//!
//! ```text
//! feature app                         window manager
//!   FeatureBridge::send_request  -->  ProtocolServer::drain_requests
//!   FeatureBridge::poll_events   <--  ProtocolServer::publish
//! ```
//!
//! No feature detail (GTK, Smithay, file paths) leaks into the protocol: the
//! payloads are plain data, and each side owns its own interpretation.

pub mod client;
pub mod codec;
pub mod models;
pub mod server;
pub mod transport;

pub use client::FeatureBridge;
pub use codec::{decode, encode, CodecError};
pub use models::{
    ActiveWindowInfo, Envelope, Event, MessageId, Payload, ProtocolError, Request, Response,
    WindowAction, WindowTarget, PROTOCOL_VERSION,
};
pub use server::{NullHandler, ProtocolServer, RequestHandler};
pub use transport::{FileTransport, Transport};
