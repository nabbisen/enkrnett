//! `enkrnettd` command line.
//!
//! ```text
//! enkrnettd --mock [--listen 127.0.0.1:8080] [--state-dir DIR] [--ui-dir DIR]
//!           [--development] [--act-delay-ms N] [--host NAME]...
//! ```
//!
//! `--mock` runs the daemon against the in-memory platform (no Wi-Fi, no OpenWrt). The OpenWrt
//! platform is not part of this stage; without `--mock` the daemon says so and exits.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use enkrnettd::clock::SystemClock;
use enkrnettd::config::Config;
use enkrnettd::platform::MockPlatform;

fn usage() -> ! {
    eprintln!(
        "usage: enkrnettd --mock [--listen ADDR] [--state-dir DIR] [--ui-dir DIR] [--development] [--act-delay-ms N] [--host NAME]..."
    );
    std::process::exit(2)
}

fn main() {
    let mut config = Config::default();
    let mut mock = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mock" => mock = true,
            "--development" => config.development = true,
            "--listen" => {
                config.listen = args
                    .next()
                    .and_then(|a| a.parse::<SocketAddr>().ok())
                    .unwrap_or_else(|| usage());
            }
            "--state-dir" => {
                config.state_dir = PathBuf::from(args.next().unwrap_or_else(|| usage()))
            }
            "--ui-dir" => {
                config.ui_dir = Some(PathBuf::from(args.next().unwrap_or_else(|| usage())))
            }
            "--act-delay-ms" => {
                let ms = args
                    .next()
                    .and_then(|a| a.parse::<u64>().ok())
                    .unwrap_or_else(|| usage());
                config.act_delay = Duration::from_millis(ms);
            }
            "--host" => config.hosts.push(args.next().unwrap_or_else(|| usage())),
            "--help" | "-h" => usage(),
            _ => usage(),
        }
    }
    if !mock {
        eprintln!("enkrnettd: the OpenWrt platform is not part of this stage; run with --mock");
        std::process::exit(2);
    }
    config.hosts.push("localhost".to_string());
    let platform = Arc::new(MockPlatform::new());
    match enkrnettd::start(config, platform, Arc::new(SystemClock)) {
        Ok(running) => {
            eprintln!(
                "enkrnettd: listening on http://{}/ (mock platform)",
                running.addr
            );
            loop {
                std::thread::park();
            }
        }
        Err(e) => {
            eprintln!("enkrnettd: cannot start: {e}");
            std::process::exit(1);
        }
    }
}
