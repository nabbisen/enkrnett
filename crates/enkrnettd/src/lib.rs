//! The Enkrnett daemon: the management API (chapter 9) and the UI files (chapter 3) on the
//! device, the ownership state machine (chapter 4), sessions, and the settings store.
//!
//! Internal design: `docs/src/design/daemon.md`. The daemon is a blocking HTTP/1.1 server with
//! four worker threads (ENK-FOOT-012, ENK-SEC-019) and no framework. Everything that touches the
//! platform (Wi-Fi networks, lights, the kernel's view) goes through the [`platform::Platform`]
//! trait, so that the whole API runs against [`platform::MockPlatform`] on a workstation and in
//! CI; the OpenWrt implementation arrives with milestone M2.

pub mod api;
pub mod app;
pub mod auth;
pub mod clock;
pub mod config;
pub mod http;
pub mod log;
pub mod ownership;
pub mod platform;
pub mod random;
pub mod session;
pub mod static_files;
pub mod store;

use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;

use app::App;
use clock::Clock;
use config::Config;
use platform::Platform;

/// A running daemon: its address and the application state (for tests).
pub struct Running {
    pub addr: SocketAddr,
    pub app: App,
}

/// Binds the listener, builds the application, and serves on background threads. Returns once
/// the socket is bound, so that callers (and tests) know the address.
pub fn start(
    config: Config,
    platform: Arc<dyn Platform>,
    clock: Arc<dyn Clock>,
) -> std::io::Result<Running> {
    let listener = TcpListener::bind(config.listen)?;
    http::set_backlog(&listener, http::LISTEN_BACKLOG);
    let addr = listener.local_addr()?;
    let config = Arc::new(config);
    let app = App::new(Arc::clone(&config), platform, clock)?;
    app.start_ticking();
    let handler = api::Api::new(app.clone(), Arc::clone(&config));
    let worker_config = Arc::clone(&config);
    std::thread::Builder::new()
        .name("enkrnettd-accept".into())
        .spawn(move || http::serve(listener, Arc::new(handler), &worker_config))?;
    log::event("started", &[("listen", &addr)]);
    Ok(Running { addr, app })
}
