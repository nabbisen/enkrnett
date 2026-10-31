//! A blocking HTTP/1.1 server without a framework (ENK-FOOT-012): one accept loop, four worker
//! threads, `Connection: close` on every response, and the limits of chapter 9 §6.

pub mod request;
pub mod response;

use std::net::{TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use enkrnett_model::api::ErrorCode;

use crate::config::Config;
pub use request::Request;
pub use response::Response;

/// Worker threads, one connection each (ENK-SEC-019).
pub const WORKERS: usize = 4;
/// Listen backlog (ENK-SEC-019).
pub const LISTEN_BACKLOG: i32 = 16;

/// Handles one parsed request.
pub trait Handler: Send + Sync {
    fn handle(&self, request: &Request) -> Response;
}

/// Sets the listen backlog on an already listening socket.
pub fn set_backlog(listener: &TcpListener, backlog: i32) {
    // SAFETY: `listen(2)` on a valid, bound, listening socket only changes its backlog.
    unsafe {
        libc::listen(listener.as_raw_fd(), backlog);
    }
}

/// Accepts connections and hands each to one of the workers. Never returns.
pub fn serve(listener: TcpListener, handler: Arc<dyn Handler>, config: &Config) {
    let (tx, rx) = mpsc::sync_channel::<TcpStream>(0);
    let rx = Arc::new(std::sync::Mutex::new(rx));
    for n in 0..WORKERS {
        let rx = Arc::clone(&rx);
        let handler = Arc::clone(&handler);
        let config = config.clone();
        std::thread::Builder::new()
            .name(format!("enkrnettd-worker-{n}"))
            .spawn(move || {
                loop {
                    let stream = {
                        let guard = rx.lock().expect("worker queue");
                        guard.recv()
                    };
                    match stream {
                        Ok(stream) => handle_connection(stream, handler.as_ref(), &config),
                        Err(_) => return,
                    }
                }
            })
            .expect("spawn worker");
    }
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if tx.send(stream).is_err() {
                    return;
                }
            }
            Err(e) => {
                crate::log::event("accept_failed", &[("error", &e)]);
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, handler: &dyn Handler, config: &Config) {
    let peer = match stream.peer_addr() {
        Ok(a) => a.ip(),
        Err(_) => return,
    };
    let _ = stream.set_nodelay(true);
    let response = match request::read_request(&mut stream, peer, config.request_timeout) {
        Ok(request) => handler.handle(&request),
        Err(request::ReadError::ClosedOrTimedOut) => return,
        Err(request::ReadError::HeadersTooLarge) => Response::error(
            431,
            ErrorCode::TooLarge,
            "request line and headers over 8 KiB",
        ),
        Err(request::ReadError::BodyTooLarge) => {
            Response::error(413, ErrorCode::TooLarge, "body over 16 KiB")
        }
        Err(request::ReadError::Malformed(why)) => Response::error(400, ErrorCode::Invalid, why),
    };
    let _ = stream.set_write_timeout(Some(config.response_timeout));
    let _ = response.write_to(&mut stream);
    let _ = stream.shutdown(std::net::Shutdown::Both);
}
