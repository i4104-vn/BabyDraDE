//! Session and device context setup for the Udev backend.

use std::error::Error;

use smithay::{
    backend::{
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        session::{libseat::{LibSeatSession, LibSeatSessionNotifier}, Session},
    },
    reexports::{calloop::EventLoop, input::Libinput},
};
use tracing::info;

use crate::app::state::CalloopData;

/// Claims the LibSeat seat and installs the libinput event source.
///
/// Returns the session, its notifier and the seat name used to assign libinput.
pub(crate) fn claim_seat(
    event_loop: &mut EventLoop<'_, CalloopData>,
) -> Result<(LibSeatSession, LibSeatSessionNotifier, String), Box<dyn Error>> {
    // Claim seat via LibSeat (logind / seatd)
    let (session, notifier) = LibSeatSession::new().map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!(
                "Failed to claim seat session via LibSeat: {err}.\n\
                 Ensure systemd-logind or seatd is running and current user is in the 'seat' / 'video' / 'input' group."
            ),
        )
    })?;

    let seat_name = session.seat();
    info!("LibSeat session established on seat '{}'", seat_name);

    // Setup Libinput for keyboard, pointer, and touch inputs on bare TTY
    let mut libinput_context =
        Libinput::new_with_udev(LibinputSessionInterface::from(session.clone()));
    libinput_context.udev_assign_seat(&seat_name).map_err(|_| {
        std::io::Error::other(format!("Failed to assign seat '{seat_name}' to Libinput"))
    })?;

    let libinput_backend = LibinputInputBackend::new(libinput_context);

    event_loop
        .handle()
        .insert_source(libinput_backend, move |event, _, data| {
            data.state.process_input_event(event);
        })
        .map_err(|err| {
            std::io::Error::other(format!("Failed to insert Libinput source: {err}"))
        })?;

    Ok((
        session,
        notifier,
        seat_name,
    ))
}
