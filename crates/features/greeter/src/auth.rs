//! greetd IPC authentication for the integrated BabyDraDE greeter.
//!
//! The protocol is length-prefixed JSON over the `GREETD_SOCK` unix socket.

use greetd_ipc::{Request, Response};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

fn write_req(stream: &mut UnixStream, req: &Request) -> Result<(), String> {
    let body = serde_json::to_vec(req).map_err(|err| err.to_string())?;
    let len = (body.len() as u32).to_ne_bytes();
    stream
        .write_all(&len)
        .and_then(|_| stream.write_all(&body))
        .map_err(|err| err.to_string())
}

fn read_res(stream: &mut UnixStream) -> Result<Response, String> {
    let mut len_bytes = [0u8; 4];
    stream
        .read_exact(&mut len_bytes)
        .map_err(|err| err.to_string())?;
    let len = u32::from_ne_bytes(len_bytes) as usize;
    let mut body = vec![0u8; len];
    stream
        .read_exact(&mut body)
        .map_err(|err| err.to_string())?;
    serde_json::from_slice(&body).map_err(|err| err.to_string())
}

fn unexpected(stage: &str, response: Response) -> String {
    let message = format!("Unexpected response from greetd after {stage}: {response:?}");
    tracing::error!("{message}");
    message
}

fn status_error(stage: &str, response: Response) -> String {
    match response {
        Response::Error {
            error_type,
            description,
        } => {
            let message = format!("{stage} failed: {error_type:?}: {description}");
            tracing::error!("{message}");
            message
        }
        other => unexpected(stage, other),
    }
}

/// Default session started after a successful login.
fn session_command() -> Vec<String> {
    std::env::var("BABYDRADE_GREETER_SESSION")
        .unwrap_or_else(|_| "BabyDra".to_string())
        .split_whitespace()
        .map(str::to_string)
        .filter(|part| !part.is_empty())
        .collect()
}

/// Authenticates `user` with `password` and starts the desktop session.
pub(crate) fn do_login(user: String, password: String) -> Result<(), String> {
    let socket_path = std::env::var("GREETD_SOCK")
        .map_err(|_| "GREETD_SOCK is not set; the greeter must run under greetd".to_string())?;
    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|err| format!("Cannot connect to greetd socket {socket_path}: {err}"))?;

    write_req(&mut stream, &Request::CreateSession { username: user })?;
    let response = read_res(&mut stream)?;
    let (auth_type, auth_message) = match response {
        Response::AuthMessage {
            auth_message_type,
            auth_message,
        } => (auth_message_type, auth_message),
        other => return Err(status_error("CreateSession", other)),
    };
    tracing::debug!("greetd requested auth ({auth_type:?}): {auth_message}");

    write_req(
        &mut stream,
        &Request::PostAuthMessageResponse {
            response: Some(password),
        },
    )?;
    match read_res(&mut stream)? {
        Response::Success => {}
        other => return Err(status_error("Authentication", other)),
    }

    let command = session_command();
    if command.is_empty() {
        return Err("No desktop session command configured for the greeter".to_string());
    }
    write_req(
        &mut stream,
        &Request::StartSession {
            cmd: command.clone(),
            env: Vec::new(),
        },
    )?;
    match read_res(&mut stream)? {
        Response::Success => {
            tracing::info!("greetd started session: {command:?}");
            Ok(())
        }
        other => Err(status_error("StartSession", other)),
    }
}
