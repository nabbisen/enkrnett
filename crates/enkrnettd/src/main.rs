//! `enkrnettd` command line.
//!
//! ```text
//! enkrnettd [--listen ADDR] [--state-dir DIR] [--ui-dir DIR] [--development] [--act-delay-ms N] [--host NAME]...
//!           (--mock | --profile FILE | --profiles-dir DIR [--config-dir DIR] [--ubus-socket PATH])
//! ```
//!
//! `--mock` runs against the in-memory platform (no Wi-Fi, no OpenWrt). On a device, `--profile`
//! names the device profile, or `--profiles-dir` holds one file per profile and the daemon picks
//! the one whose `board_names` contains `ubus call system board`'s `board_name`.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use enkrnett_model::profile::DeviceProfile;
use enkrnettd::clock::SystemClock;
use enkrnettd::config::Config;
use enkrnettd::platform::{MockPlatform, OpenWrtPlatform, Platform};

fn usage() -> ! {
    eprintln!(
        "usage: enkrnettd [--listen ADDR] [--state-dir DIR] [--ui-dir DIR] [--development] [--act-delay-ms N] [--host NAME]...\n       (--mock | --profile FILE | --profiles-dir DIR [--config-dir DIR] [--ubus-socket PATH])"
    );
    std::process::exit(2)
}

fn load_profile(path: &Path) -> DeviceProfile {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("enkrnettd: cannot read profile {}: {e}", path.display());
        std::process::exit(1)
    });
    let profile = DeviceProfile::parse(&text).unwrap_or_else(|e| {
        eprintln!("enkrnettd: profile {}: {e}", path.display());
        std::process::exit(1)
    });
    let errors = profile.check();
    if !errors.is_empty() {
        eprintln!(
            "enkrnettd: profile {} breaks the rules of chapter 5: {errors:?}",
            path.display()
        );
        std::process::exit(1);
    }
    profile
}

fn board_name(ubus_socket: Option<&str>) -> Option<String> {
    let mut client = match ubus_socket {
        Some(p) => enkrnett_ubus::client::Client::connect_path(p).ok()?,
        None => enkrnett_ubus::client::Client::connect().ok()?,
    };
    let board = client
        .call("system", "board", &serde_json::Map::new())
        .ok()?;
    board
        .get("board_name")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn select_profile(dir: &Path, ubus_socket: Option<&str>) -> DeviceProfile {
    let board = board_name(ubus_socket).unwrap_or_else(|| {
        eprintln!("enkrnettd: cannot read the board name through ubus");
        std::process::exit(1)
    });
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| {
            eprintln!("enkrnettd: cannot read {}: {e}", dir.display());
            std::process::exit(1)
        })
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    entries.sort();
    for path in &entries {
        let profile = load_profile(path);
        if profile.board_names.contains(&board) {
            return profile;
        }
    }
    eprintln!(
        "enkrnettd: no profile in {} names the board {board}",
        dir.display()
    );
    std::process::exit(1)
}

fn main() {
    let mut config = Config::default();
    let mut mock = false;
    let mut profile_path: Option<PathBuf> = None;
    let mut profiles_dir: Option<PathBuf> = None;
    let mut config_dir = PathBuf::from("/etc/config");
    let mut ubus_socket: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mock" => mock = true,
            "--development" => config.development = true,
            "--listen" => {
                config.listen = args
                    .next()
                    .and_then(|a| a.parse::<SocketAddr>().ok())
                    .unwrap_or_else(|| usage())
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
            "--profile" => {
                profile_path = Some(PathBuf::from(args.next().unwrap_or_else(|| usage())))
            }
            "--profiles-dir" => {
                profiles_dir = Some(PathBuf::from(args.next().unwrap_or_else(|| usage())))
            }
            "--config-dir" => config_dir = PathBuf::from(args.next().unwrap_or_else(|| usage())),
            "--ubus-socket" => ubus_socket = Some(args.next().unwrap_or_else(|| usage())),
            "--help" | "-h" => usage(),
            _ => usage(),
        }
    }
    let platform: Arc<dyn Platform> = if mock {
        config.hosts.push("localhost".to_string());
        Arc::new(MockPlatform::new())
    } else {
        let profile = match (&profile_path, &profiles_dir) {
            (Some(p), _) => load_profile(p),
            (None, Some(d)) => select_profile(d, ubus_socket.as_deref()),
            (None, None) => {
                eprintln!("enkrnettd: give --mock, --profile FILE, or --profiles-dir DIR");
                std::process::exit(2)
            }
        };
        let (m, t, p) = (
            profile.argon2.memory_kib,
            profile.argon2.iterations,
            profile.argon2.parallelism,
        );
        config.argon2 = (m, t, p);
        eprintln!(
            "enkrnettd: device profile {} ({} {})",
            profile.id, profile.vendor, profile.model
        );
        Arc::new(OpenWrtPlatform::new(
            profile,
            config_dir,
            config.state_dir.clone(),
            ubus_socket,
        ))
    };
    match enkrnettd::start(config, platform, Arc::new(SystemClock)) {
        Ok(running) => {
            eprintln!(
                "enkrnettd: listening on http://{}/{}",
                running.addr,
                if mock { " (mock platform)" } else { "" }
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
