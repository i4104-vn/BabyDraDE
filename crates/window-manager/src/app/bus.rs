//! Calloop adapter hosting the in-memory protocol router.
//!
//! The compositor owns the router and one local peer (`wm`). Feature apps
//! connect over an abstract Unix socket; each accepted connection becomes a
//! calloop source that feeds the router and drains its outbox.

use std::{
    cell::RefCell,
    collections::HashMap,
    fmt, io,
    os::{
        fd::{AsFd, BorrowedFd},
        unix::net::{UnixListener, UnixStream},
    },
    rc::Rc,
    time::{Duration, Instant},
};

use crate::protocol::{
    connection::{Connection, ReadResult},
    socket::bind,
    verify_peer, BusSender, Envelope, Kind, Message, PeerId, Router,
};
use smithay::reexports::calloop::{
    generic::{Generic, NoIoDrop},
    timer::{TimeoutAction, Timer},
    EventLoop, Interest, LoopHandle, Mode, PostAction, RegistrationToken,
};
use tracing::{error, warn};

use crate::app::state::{BabydraState, CalloopData};

/// Polling source for one connected feature.
struct ClientSource {
    stream: Rc<UnixStream>,
    conn: Rc<RefCell<Connection>>,
}

impl AsFd for ClientSource {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.stream.as_fd()
    }
}

/// Polling source for the compositor's local `wm` peer.
struct WmSource {
    stream: Rc<UnixStream>,
    conn: Rc<RefCell<Connection>>,
}

impl AsFd for WmSource {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.stream.as_fd()
    }
}

/// Registration bookkeeping for a feature connection.
struct Client {
    token: RegistrationToken,
    conn: Rc<RefCell<Connection>>,
    stream: Rc<UnixStream>,
}

/// Owns the router and every calloop-registered endpoint.
pub(crate) struct BusHost {
    router: Router,
    clients: HashMap<PeerId, Client>,
    wm_id: PeerId,
    /// Local `wm` endpoint once its calloop source has been registered.
    wm: Option<(RegistrationToken, Rc<RefCell<Connection>>, Rc<UnixStream>)>,
    /// Local `wm` endpoint before [`init`] gives it a registration token.
    wm_pending: Option<(Rc<RefCell<Connection>>, Rc<UnixStream>)>,
    listener: Option<UnixListener>,
}

/// Creates the listening socket, router and compositor-side `wm` endpoint.
///
/// # Returns
/// The uninitialised host and the sender handed to compositor state.
///
/// # Errors
/// Returns an error when the socket cannot be bound or the peer pair fails.
pub(crate) fn build() -> io::Result<(BusHost, BusSender)> {
    let listener = bind()?;

    // The router half is polled by calloop and must be non-blocking; the other
    // half backs `BusSender`, which uses blocking `write_all`.
    let (router_end, wm_end) = UnixStream::pair()?;
    router_end.set_nonblocking(true)?;

    let mut router = Router::new();
    let wm_id = router.register_local("wm".into());

    let conn_stream = router_end.try_clone()?;
    let conn = Rc::new(RefCell::new(Connection::new(conn_stream)));
    let stream = Rc::new(router_end);
    let sender = BusSender::new(wm_end, "wm".into());

    Ok((
        BusHost {
            router,
            clients: HashMap::new(),
            wm_id,
            wm: None,
            wm_pending: Some((conn, stream)),
            listener: Some(listener),
        },
        sender,
    ))
}

/// Registers the listener, the local `wm` endpoint and the housekeeping timer.
///
/// # Arguments
/// * `event_loop` - Compositor event loop that will own the sources.
/// * `data` - Loop data holding the [`BusHost`] built by [`build`].
///
/// # Errors
/// Returns an error when a source cannot be registered.
pub(crate) fn init(
    event_loop: &mut EventLoop<'_, CalloopData>,
    data: &mut CalloopData,
) -> io::Result<()> {
    let handle = event_loop.handle();

    let listener = data
        .bus
        .listener
        .take()
        .ok_or_else(|| io::Error::other("protocol listener already registered"))?;
    let listener_weak = handle.downgrade();
    handle
        .insert_source(
            Generic::new(listener, Interest::READ, Mode::Level),
            move |_, source, data| {
                let Some(handle) = listener_weak.upgrade() else {
                    return Ok(PostAction::Continue);
                };
                accept_clients(&handle, source, data)
            },
        )
        .map_err(|error| io::Error::other(error.to_string()))?;

    let (wm_conn, wm_stream) = data
        .bus
        .wm_pending
        .take()
        .ok_or_else(|| io::Error::other("protocol wm endpoint already registered"))?;
    let wm_source = WmSource {
        stream: wm_stream.clone(),
        conn: wm_conn.clone(),
    };
    let wm_weak = handle.downgrade();
    let wm_id = data.bus.wm_id;
    let token = handle
        .insert_source(
            Generic::new(wm_source, Interest::BOTH, Mode::Edge),
            move |_, source, data| {
                let Some(handle) = wm_weak.upgrade() else {
                    return Ok(PostAction::Continue);
                };
                wm_callback(&handle, wm_id, source, data)
            },
        )
        .map_err(|error| io::Error::other(error.to_string()))?;
    data.bus.wm = Some((token, wm_conn, wm_stream));

    handle
        .insert_source(Timer::immediate(), |_, _, data| {
            data.bus.router.tick(Instant::now());
            flush_peers(data);
            TimeoutAction::ToDuration(Duration::from_secs(1))
        })
        .map_err(|error| io::Error::other(error.to_string()))?;

    Ok(())
}

/// Accepts every pending feature connection and registers it with the loop.
fn accept_clients(
    handle: &LoopHandle<'_, CalloopData>,
    source: &mut NoIoDrop<UnixListener>,
    data: &mut CalloopData,
) -> io::Result<PostAction> {
    loop {
        let (stream, _) = match source.accept() {
            Ok(pair) => pair,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(error) => {
                warn!("failed to accept protocol client: {error}");
                break;
            }
        };

        if let Err(error) = verify_peer(&stream) {
            warn!("rejected protocol client: {error}");
            continue;
        }
        if let Err(error) = stream.set_nonblocking(true) {
            warn!("failed to configure protocol client: {error}");
            continue;
        }

        let id = data.bus.router.add_connection();
        let conn_stream = match stream.try_clone() {
            Ok(clone) => clone,
            Err(error) => {
                warn!("failed to clone protocol client stream: {error}");
                data.bus.router.remove(id);
                continue;
            }
        };
        let conn = Rc::new(RefCell::new(Connection::new(conn_stream)));
        let stream = Rc::new(stream);
        let client_source = ClientSource {
            stream: stream.clone(),
            conn: conn.clone(),
        };

        let client_weak = handle.downgrade();
        match handle.insert_source(
            Generic::new(client_source, Interest::BOTH, Mode::Edge),
            move |_, source, data| {
                let Some(client_handle) = client_weak.upgrade() else {
                    return Ok(PostAction::Continue);
                };
                client_callback(&client_handle, id, source, data)
            },
        ) {
            Ok(token) => {
                data.bus.clients.insert(
                    id,
                    Client {
                        token,
                        conn,
                        stream,
                    },
                );
            }
            Err(error) => {
                warn!("failed to register protocol client: {error}");
                data.bus.router.remove(id);
            }
        }
    }

    flush_peers(data);
    Ok(PostAction::Continue)
}

/// Drains one feature connection into the router.
fn client_callback(
    handle: &LoopHandle<'_, CalloopData>,
    id: PeerId,
    source: &mut NoIoDrop<ClientSource>,
    data: &mut CalloopData,
) -> io::Result<PostAction> {
    let conn = source.conn.clone();
    loop {
        let result = conn.borrow_mut().read_available();
        match result {
            Ok(ReadResult::Messages(messages)) => {
                for envelope in messages {
                    data.bus.router.route(id, envelope);
                }
            }
            Ok(ReadResult::WouldBlock) => break,
            Ok(ReadResult::Closed) => {
                remove_client(handle, id, data, "closed");
                return Ok(PostAction::Remove);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(error) => {
                warn!("protocol client {id} read failed: {error}");
                remove_client(handle, id, data, "error");
                return Ok(PostAction::Remove);
            }
        }
    }

    flush_peers(data);
    Ok(PostAction::Continue)
}

/// Drains the compositor's own endpoint into the router.
fn wm_callback(
    handle: &LoopHandle<'_, CalloopData>,
    wm_id: PeerId,
    source: &mut NoIoDrop<WmSource>,
    data: &mut CalloopData,
) -> io::Result<PostAction> {
    let conn = source.conn.clone();
    loop {
        let result = conn.borrow_mut().read_available();
        match result {
            Ok(ReadResult::Messages(messages)) => {
                for envelope in messages {
                    data.bus.router.route(wm_id, envelope);
                }
            }
            Ok(ReadResult::WouldBlock) => break,
            Ok(ReadResult::Closed) => {
                warn!("protocol wm endpoint disconnected");
                drop_wm(handle, wm_id, data);
                return Ok(PostAction::Remove);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(error) => {
                error!("protocol wm endpoint read failed: {error}");
                drop_wm(handle, wm_id, data);
                return Ok(PostAction::Remove);
            }
        }
    }

    flush_peers(data);
    Ok(PostAction::Continue)
}

/// Removes a disconnected feature from the router and the event loop.
fn remove_client(
    handle: &LoopHandle<'_, CalloopData>,
    id: PeerId,
    data: &mut CalloopData,
    reason: &str,
) {
    warn!("protocol client {id} disconnected ({reason})");
    data.bus.router.remove(id);
    if let Some(client) = data.bus.clients.remove(&id) {
        let _ = client.stream.shutdown(std::net::Shutdown::Both);
        handle.remove(client.token);
    }
}

/// Drops the compositor-side endpoint without tearing down the compositor.
fn drop_wm(handle: &LoopHandle<'_, CalloopData>, wm_id: PeerId, data: &mut CalloopData) {
    data.bus.router.remove(wm_id);
    if let Some((token, conn, stream)) = data.bus.wm.take() {
        if conn.borrow().has_outbox() {
            warn!("protocol wm endpoint disconnected with a pending outbox");
        }
        let _ = stream.shutdown(std::net::Shutdown::Both);
        handle.remove(token);
    }
}

/// Drains every registered peer's outbox after the router has ticked.
///
/// # Arguments
/// * `data` - Loop data holding the router and compositor state.
fn flush_peers(data: &mut CalloopData) {
    let ids: Vec<PeerId> = data
        .bus
        .clients
        .keys()
        .copied()
        .chain(std::iter::once(data.bus.wm_id))
        .collect();

    for id in ids {
        flush_peer(id, data);
    }
}

/// Drains one peer's outbox, either dispatching locally or queueing to a socket.
///
/// # Arguments
/// * `id` - Peer whose outbox should be drained.
/// * `data` - Loop data holding the router and compositor state.
fn flush_peer(id: PeerId, data: &mut CalloopData) {
    let envelopes = data.bus.router.take_outbox(id);
    if envelopes.is_empty() {
        return;
    }

    if id == data.bus.wm_id {
        // The compositor is a local router peer: requests are dispatched
        // in-process and answers travel back through `BusSender`.
        dispatch_local(envelopes, &mut data.state);
        return;
    }

    let Some(conn) = data.bus.clients.get(&id).map(|client| client.conn.clone()) else {
        return;
    };
    let mut borrowed = conn.borrow_mut();
    for envelope in &envelopes {
        if let Err(error) = borrowed.queue(envelope) {
            warn_peer(id, "queue", error);
        }
    }
    if let Err(error) = borrowed.flush() {
        if error.kind() != io::ErrorKind::WouldBlock {
            warn_peer(id, "flush", error);
        }
    }
}

/// Dispatches request envelopes addressed to the compositor's local peer.
///
/// # Arguments
/// * `envelopes` - Outbox entries produced by the router for the `wm` peer.
/// * `state` - Live compositor state the requests act on.
fn dispatch_local(envelopes: Vec<Envelope>, state: &mut BabydraState) {
    for envelope in envelopes {
        if envelope.kind == Kind::Request {
            let message = Message::from_envelope(envelope);
            crate::app::bridge::dispatch(message, state);
        }
    }
}

/// Logs a failed queue or flush for a protocol peer.
///
/// # Arguments
/// * `id` - Peer whose connection failed.
/// * `action` - Name of the failed operation, used in the log message.
/// * `error` - Error returned by the connection.
fn warn_peer<E: fmt::Display>(id: PeerId, action: &str, error: E) {
    warn!("failed to {action} protocol peer {id}: {error}");
}
