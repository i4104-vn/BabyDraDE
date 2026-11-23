//! Compositor-side clipboard persistence.
//!
//! Wayland selections are owned by the client that copied the data, so the
//! clipboard is lost as soon as that client exits. We cache the selection in
//! `services` and then take ownership of it ourselves, which keeps copy/paste
//! working exactly like on other desktops.

use std::{
    io::Read,
    os::unix::{io::OwnedFd, net::UnixStream},
    time::Duration,
};

use smithay::{
    input::Seat,
    reexports::{
        calloop::channel::Sender,
        wayland_server::DisplayHandle,
    },
    wayland::selection::data_device::{
        request_data_device_client_selection, set_data_device_selection,
    },
};
use tracing::{info, warn};

use crate::adapters::compositor::BabydraState;
use crate::application::clipboard::preferred_mime;

/// State owned by the clipboard capture pipeline.
///
/// The capture completion sender remains a facade field because the composition
/// root (outside this adapter crate) installs it directly; it is passed in as a
/// narrow argument when a capture starts.
#[derive(Default)]
pub(crate) struct ClipboardState {
    /// MIME types of a client selection that still needs to be cached.
    pending: Option<Vec<String>>,
    /// MIME types of a capture currently running in a background thread.
    capture: Option<Vec<String>>,
}

impl ClipboardState {
    /// Records that a client installed a new selection. The actual capture is
    /// deferred because Smithay stores the client source *after* calling
    /// `SelectionHandler::new_selection`.
    pub(crate) fn note_client_selection(&mut self, mimes: Vec<String>) {
        if !mimes.is_empty() {
            self.pending = Some(mimes);
        }
    }

    /// Requests the pending client selection and caches it on a background
    /// thread. Called once per frame from both backends.
    pub(crate) fn update_clipboard(
        &mut self,
        seat: &Seat<BabydraState>,
        sender: Option<Sender<Option<String>>>,
    ) {
        // Wait for the previous capture to finish before touching the source.
        if self.capture.is_some() {
            return;
        }
        let Some(mimes) = self.pending.take() else {
            return;
        };
        let Some(mime) = preferred_mime(&mimes) else {
            return;
        };

        let (mut reader, writer) = match UnixStream::pair() {
            Ok(pair) => pair,
            Err(err) => {
                warn!("Failed to create clipboard pipe: {err}");
                return;
            }
        };
        let fd: OwnedFd = writer.into();

        if let Err(err) = request_data_device_client_selection(seat, mime.clone(), fd) {
            warn!("Failed to request clipboard data for '{mime}': {err:?}");
            return;
        }

        let capture_mime = mime.clone();
        let capture_mimes = mimes.clone();
        std::thread::Builder::new()
            .name("babydrade-clipboard".to_string())
            .spawn(move || {
                let _ = reader.set_read_timeout(Some(Duration::from_secs(2)));
                let mut data = Vec::new();
                let mut buf = [0u8; 8192];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => data.extend_from_slice(&buf[..n]),
                        Err(err)
                            if err.kind() == std::io::ErrorKind::TimedOut
                                || err.kind() == std::io::ErrorKind::Interrupted =>
                        {
                            break;
                        }
                        Err(_) => break,
                    }
                }
                if !data.is_empty() {
                    services::system::clipboard::store_selection(
                        &capture_mime,
                        &capture_mimes,
                        &data,
                    );
                    info!(
                        "Cached clipboard selection ({:?}, {} bytes)",
                        capture_mime,
                        data.len()
                    );
                }
                if let Some(sender) = sender {
                    // Only take ownership when we actually have bytes to
                    // serve; otherwise leave the client selection untouched.
                    let _ = sender.send((!data.is_empty()).then_some(capture_mime));
                }
            })
            .ok();

        self.capture = Some(mimes);
    }

    /// Called from the calloop channel once the data has been cached. Now it
    /// is safe to replace the client source: `cancelled` cannot destroy data
    /// we already have.
    pub(crate) fn complete_clipboard_capture(
        &mut self,
        cached_mime: Option<String>,
        seat: &Seat<BabydraState>,
        display_handle: &DisplayHandle,
    ) {
        if self.capture.take().is_none() {
            return;
        }
        let Some(cached_mime) = cached_mime else {
            return;
        };

        // Only advertise MIME types we can actually serve. We cache one
        // representation, so for text we also advertise the common aliases
        // that GTK/Qt/XWayland clients request when pasting.
        let mut offered = vec![cached_mime.clone()];
        let is_text = cached_mime.starts_with("text/")
            || matches!(cached_mime.as_str(), "UTF8_STRING" | "STRING");
        if is_text {
            for alias in [
                "UTF8_STRING",
                "STRING",
                "text/plain",
                "text/plain;charset=utf-8",
            ] {
                if !offered.iter().any(|mime| mime == alias) {
                    offered.push(alias.to_string());
                }
            }
        }

        info!("Owning clipboard selection with MIMEs: {:?}", offered);
        set_data_device_selection(display_handle, seat, offered, ());
    }
}
