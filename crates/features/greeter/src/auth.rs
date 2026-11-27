//! greetd IPC authentication for the integrated BabyDraDE greeter.
//!
//! The protocol is length-prefixed JSON over the `GREETD_SOCK` unix socket.

use greetd_ipc::{AuthMessageType, Request, Response};
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

/// The next step of the greetd authentication state machine.
///
/// This is deliberately independent of the socket so retry behaviour can be
/// unit-tested without a running greetd daemon.
#[derive(Debug, PartialEq, Eq)]
enum AuthStep {
    /// greetd asked a question; answer it with the password.
    Answer(String),
    /// greetd sent an info/error message; acknowledge it with `None`.
    Acknowledge(String),
    /// Authentication completed successfully.
    Ready,
    /// greetd rejected the attempt; the session has to be cancelled.
    Failed(String),
}

/// Maps a greetd response to the next authentication step.
fn auth_step(stage: &str, response: Response) -> AuthStep {
    match response {
        Response::Success => AuthStep::Ready,
        Response::AuthMessage {
            auth_message_type,
            auth_message,
        } => match auth_message_type {
            AuthMessageType::Visible | AuthMessageType::Secret => AuthStep::Answer(auth_message),
            AuthMessageType::Info | AuthMessageType::Error => AuthStep::Acknowledge(auth_message),
        },
        Response::Error {
            error_type,
            description,
        } => AuthStep::Failed(format!("{stage} failed: {error_type:?}: {description}")),
    }
}

/// Guard against a misbehaving PAM stack that keeps asking questions forever.
const MAX_AUTH_STEPS: usize = 8;

/// Discard the session greetd keeps "under configuration" after a failed login.
///
/// greetd does not drop that session when authentication fails: a following
/// `CreateSession` is rejected with "a session is already being configured"
/// (reported to the user as an "another session" error). `CancelSession` is a
/// no-op when no session is pending, so it is safe to send before each attempt
/// and after a failed one.
fn cancel_session(stream: &mut UnixStream) {
    if let Err(err) = write_req(stream, &Request::CancelSession) {
        tracing::warn!("could not cancel stale greetd session: {err}");
        return;
    }
    // Always consume the reply so it cannot be mistaken for the response to
    // the request that follows on this stream.
    match read_res(stream) {
        Ok(Response::Success) => {}
        Ok(other) => {
            tracing::warn!("Unexpected response from greetd after CancelSession: {other:?}")
        }
        Err(err) => tracing::warn!("could not read greetd CancelSession response: {err}"),
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

/// Runs one full login attempt on an already connected socket.
fn authenticate(stream: &mut UnixStream, user: &str, password: &str) -> Result<(), String> {
    write_req(
        stream,
        &Request::CreateSession {
            username: user.to_string(),
        },
    )?;
    let mut response = read_res(stream)?;
    let mut stage = "CreateSession";
    let mut steps = 0;

    loop {
        match auth_step(stage, response) {
            AuthStep::Ready => break,
            AuthStep::Answer(message) => {
                tracing::debug!("greetd asks for a secret: {message}");
                steps += 1;
                if steps > MAX_AUTH_STEPS {
                    return Err("Too many authentication prompts from greetd".to_string());
                }
                write_req(
                    stream,
                    &Request::PostAuthMessageResponse {
                        response: Some(password.to_string()),
                    },
                )?;
            }
            AuthStep::Acknowledge(message) => {
                tracing::debug!("greetd message: {message}");
                steps += 1;
                if steps > MAX_AUTH_STEPS {
                    return Err("Too many authentication prompts from greetd".to_string());
                }
                write_req(stream, &Request::PostAuthMessageResponse { response: None })?;
            }
            AuthStep::Failed(message) => return Err(message),
        }
        stage = "Authentication";
        response = read_res(stream)?;
    }

    let command = session_command();
    if command.is_empty() {
        return Err("No desktop session command configured for the greeter".to_string());
    }
    tracing::debug!("greetd authentication succeeded; starting {command:?}");
    write_req(
        stream,
        &Request::StartSession {
            cmd: command.clone(),
            env: Vec::new(),
        },
    )?;
    match read_res(stream)? {
        Response::Success => {
            tracing::info!("greetd started session: {command:?}");
            Ok(())
        }
        other => Err(status_error("StartSession", other)),
    }
}

/// Authenticates `user` with `password` and starts the desktop session.
pub(crate) fn do_login(user: String, password: String) -> Result<(), String> {
    let socket_path = std::env::var("GREETD_SOCK")
        .map_err(|_| "GREETD_SOCK is not set; the greeter must run under greetd".to_string())?;
    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|err| format!("Cannot connect to greetd socket {socket_path}: {err}"))?;

    // A previous attempt (including one made by an older greeter) may have
    // left a session pending. Clear it before creating a new one.
    cancel_session(&mut stream);

    let result = authenticate(&mut stream, &user, &password);
    if result.is_err() {
        cancel_session(&mut stream);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{auth_step, AuthStep};
    use greetd_ipc::{AuthMessageType, ErrorType, Response};

    #[test]
    fn success_means_ready_to_start_session() {
        assert_eq!(
            auth_step("Authentication", Response::Success),
            AuthStep::Ready
        );
    }

    #[test]
    fn password_prompts_are_answered_with_the_password() {
        for auth_message_type in [AuthMessageType::Visible, AuthMessageType::Secret] {
            let step = auth_step(
                "Authentication",
                Response::AuthMessage {
                    auth_message_type,
                    auth_message: "Password:".to_string(),
                },
            );
            assert_eq!(step, AuthStep::Answer("Password:".to_string()));
        }
    }

    #[test]
    fn info_and_error_messages_are_acknowledged() {
        for auth_message_type in [AuthMessageType::Info, AuthMessageType::Error] {
            let step = auth_step(
                "Authentication",
                Response::AuthMessage {
                    auth_message_type,
                    auth_message: "Login incorrect".to_string(),
                },
            );
            assert_eq!(step, AuthStep::Acknowledge("Login incorrect".to_string()));
        }
    }

    #[test]
    fn auth_error_is_reported_with_stage_and_description() {
        let step = auth_step(
            "Authentication",
            Response::Error {
                error_type: ErrorType::AuthError,
                description: "authenticate: AUTH_ERR".to_string(),
            },
        );
        assert_eq!(
            step,
            AuthStep::Failed(
                "Authentication failed: AuthError: authenticate: AUTH_ERR".to_string()
            )
        );
    }

    #[test]
    fn create_session_error_keeps_its_stage() {
        let step = auth_step(
            "CreateSession",
            Response::Error {
                error_type: ErrorType::Error,
                description: "a session is already being configured".to_string(),
            },
        );
        assert_eq!(
            step,
            AuthStep::Failed(
                "CreateSession failed: Error: a session is already being configured".to_string()
            )
        );
    }
}
