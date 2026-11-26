//! Tests for the feature bus client.

use std::collections::{HashMap, VecDeque};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Condvar, Mutex, PoisonError, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use common::protocol::writer::{lock, Writer};
use common::protocol::{
    abstract_addr, decode, encode, verify_peer, Control, Envelope, Incoming, Kind, LineBuffer,
    Target,
};

use super::bus::Bus;
use super::state::{handle_frame, BusState, Inbox, ReaderPending, CHUNK};

/// Accepts one client, performs the handshake and answers one request.
///
/// # Arguments
/// * `listener` - Nonblocking abstract listener owned by the fake router.
/// * `done` - Channel notified when the exchange finishes.
fn serve_one_request(listener: UnixListener, done: mpsc::Sender<()>) {
    let (mut peer, mut buffer, mut chunk, mut answered) =
        (None, LineBuffer::new(), [0u8; CHUNK], false);
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && !answered {
        if peer.is_none() {
            match listener.accept() {
                Ok((stream, _)) if verify_peer(&stream).is_ok() => {
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(10)));
                    peer = Some(stream);
                }
                Ok(_) => {}
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(2))
                }
                Err(_) => break,
            }
            continue;
        }
        let Some(stream) = peer.as_mut() else { break };
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => {
                if buffer.push(&chunk[..count]).is_err() {
                    break;
                }
                while let Some(line) = buffer.take_line() {
                    let Ok(frame) = decode(&line) else { continue };
                    if let Some(Control::Hello { name }) = frame.control() {
                        let welcome = Envelope::new(
                            frame.id,
                            Target::Router,
                            Kind::Response,
                            serde_json::to_value(&Control::Welcome {
                                name,
                                peers: vec!["wm".to_string()],
                            })
                            .expect("welcome payload"),
                        );
                        let _ = stream.write_all(&encode(&welcome).expect("welcome frame"));
                    }
                    if frame.kind == Kind::Request && frame.target != Target::Router {
                        let mut response = Envelope::new(
                            frame.id,
                            Target::Peer("tester".to_string()),
                            Kind::Response,
                            serde_json::json!("pong"),
                        );
                        response.source = "wm".to_string();
                        let _ = stream.write_all(&encode(&response).expect("response frame"));
                        answered = true;
                    }
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(_) => break,
        }
    }
    let _ = done.send(());
}

/// Builds empty state for frame-handling tests.
fn test_state() -> BusState {
    BusState {
        name: "tester".to_string(),
        inbox: Inbox {
            messages: Mutex::new(VecDeque::new()),
            wake: Condvar::new(),
        },
        peers: RwLock::new(Vec::new()),
        pending: Mutex::new(HashMap::new()),
        stop: AtomicBool::new(false),
    }
}

/// Encodes an envelope into a raw frame line.
fn frame_line(envelope: &Envelope) -> Vec<u8> {
    encode(envelope).expect("encoded frame")
}

#[test]
fn writers_get_distinct_id_spaces() {
    let (left, _) = UnixStream::pair().expect("socket pair");
    let (right, _) = UnixStream::pair().expect("socket pair");
    assert_ne!(
        Writer::new(left).allocate_id(),
        Writer::new(right).allocate_id()
    );
}

#[test]
fn welcome_frame_merges_self_into_visible_peers() {
    let state = test_state();
    let control = Control::Welcome {
        name: "tester".to_string(),
        peers: vec!["wm".to_string()],
    };
    let envelope = Envelope::new(
        0,
        Target::Router,
        Kind::Response,
        serde_json::to_value(&control).expect("control payload"),
    );
    handle_frame(&frame_line(&envelope), &state);
    assert_eq!(
        state
            .peers
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone(),
        vec!["tester".to_string(), "wm".to_string()]
    );
}

#[test]
fn malformed_frame_reports_invalid_payload() {
    let state = test_state();
    handle_frame(b"{not json", &state);
    assert!(matches!(
        lock(&state.inbox.messages).pop_front().expect("error"),
        Incoming::Error(_)
    ));
}

#[test]
fn response_frame_clears_pending_request() {
    let state = test_state();
    lock(&state.pending).insert(
        42,
        ReaderPending {
            deadline: Instant::now(),
            target: "wm".to_string(),
        },
    );
    let envelope = Envelope::new(
        42,
        Target::Peer("wm".to_string()),
        Kind::Response,
        serde_json::json!({}),
    );
    handle_frame(&frame_line(&envelope), &state);
    assert!(!lock(&state.pending).contains_key(&42u64));
}

#[test]
fn exchanges_a_request_and_response_over_the_abstract_socket() {
    let listener = match abstract_addr().and_then(|address| UnixListener::bind_addr(&address)) {
        Ok(listener) => match listener.set_nonblocking(true) {
            Ok(()) => listener,
            Err(error) => {
                eprintln!("skipping bus socket test: {error}");
                return;
            }
        },
        Err(error) => {
            eprintln!("skipping bus socket test: {error}");
            return;
        }
    };
    let (done_tx, done_rx) = mpsc::channel();
    let server = thread::spawn(move || serve_one_request(listener, done_tx));
    let bus = match Bus::connect("tester") {
        Ok(bus) => bus,
        Err(error) => {
            eprintln!("skipping bus socket test: {error}");
            let _ = done_rx.recv_timeout(Duration::from_secs(1));
            let _ = server.join();
            return;
        }
    };
    assert!(bus.peers().iter().any(|name| name == "wm"));
    let id = bus
        .request("wm", &serde_json::json!("ping"))
        .expect("request");
    match bus.recv_timeout(Duration::from_secs(5)).expect("response") {
        Incoming::Response(message) => {
            assert_eq!(message.id, id);
            assert_eq!(message.source, "wm");
            assert_eq!(message.decode::<String>().expect("body"), "pong");
        }
        other => panic!("unexpected incoming: {other:?}"),
    }
    bus.shutdown();
    let _ = done_rx.recv_timeout(Duration::from_secs(1));
    let _ = server.join();
}
