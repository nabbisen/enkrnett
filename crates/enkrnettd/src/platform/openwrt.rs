//! The OpenWrt platform (chapter 6; RFC 014): the daemon's user owns the four configuration
//! files it writes (measurement M-5b), replaces its own sections through `enkrnett_model::uci`,
//! asks `netifd` to reload through ubus, and reads the effective state back through ubus and
//! `/proc`. Nothing here runs as root and nothing spawns a process.
//!
//! What this stage does not yet do (M2): the pending marker and the undo-at-start-up of chapter
//! 6 §4, the bridge-port `isolated` read-back (ENK-CFG-023), the listener port scan
//! (ENK-CFG-025), and renumbering on an uplink collision (ENK-TOPO-003, 006).

use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use enkrnett_model::api::{
    CheckResult, Connection, GuestStatus, HardeningItem, HealthDetail, InstalledAs,
    ManagementStatus, Memory, Radio, Storage, UplinkStatus,
};
use enkrnett_model::plan::{
    self, BssRole, ExpectedBss, MANAGEMENT_ADDRESS, OWNED_PREFIX, Phase, Plan, owned_sections,
};
use enkrnett_model::profile::DeviceProfile;
use enkrnett_model::settings::{GuestMode, Settings};
use enkrnett_model::state::{ConnectionNetwork, ConnectionSecurity, OwnershipState};
use enkrnett_model::uci::{File as UciFile, Section};
use enkrnett_ubus::client::Client;
use serde_json::{Map, Value, json};

use super::{DeviceInfo, Effective, Platform, PlatformError};
use crate::log;

/// The health check waits this long for the networks to come up (ENK-CFG-007).
pub const HEALTH_TIMEOUT: Duration = Duration::from_secs(60);
const HEALTH_POLL: Duration = Duration::from_secs(2);
/// The file the init script looks for at boot to erase the device (ENK-OWN-018).
pub const ERASE_MARKER: &str = "ERASE";

const COUNTRY_CHOICES: [&str; 12] = [
    "JP", "US", "CA", "GB", "DE", "FR", "IT", "ES", "NL", "AU", "NZ", "SG",
];

pub struct OpenWrtPlatform {
    profile: DeviceProfile,
    config_dir: PathBuf,
    state_dir: PathBuf,
    ubus_socket: Option<String>,
    phase: Mutex<Option<Phase>>,
}

impl OpenWrtPlatform {
    pub fn new(
        profile: DeviceProfile,
        config_dir: PathBuf,
        state_dir: PathBuf,
        ubus_socket: Option<String>,
    ) -> OpenWrtPlatform {
        OpenWrtPlatform {
            profile,
            config_dir,
            state_dir,
            ubus_socket,
            phase: Mutex::new(None),
        }
    }

    // ----- ubus ------------------------------------------------------------------------------

    fn ubus(&self) -> Result<Client, PlatformError> {
        match &self.ubus_socket {
            Some(p) => Client::connect_path(p),
            None => Client::connect(),
        }
        .map_err(|e| PlatformError(format!("ubus: {e}")))
    }

    fn call(
        &self,
        object: &str,
        method: &str,
        args: Value,
    ) -> Result<Map<String, Value>, PlatformError> {
        let args = args.as_object().cloned().unwrap_or_default();
        self.ubus()?
            .call(object, method, &args)
            .map_err(|e| PlatformError(format!("ubus {object} {method}: {e}")))
    }

    fn objects(&self, prefix: &str) -> Vec<String> {
        match self
            .ubus()
            .and_then(|mut c| c.lookup("").map_err(|e| PlatformError(e.to_string())))
        {
            Ok(list) => list
                .into_iter()
                .map(|o| o.path)
                .filter(|p| p.starts_with(prefix))
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    // ----- configuration files -------------------------------------------------------------

    fn read_config(&self, name: &str) -> Result<UciFile, PlatformError> {
        let path = self.config_dir.join(name);
        let text = fs::read_to_string(&path)
            .map_err(|e| PlatformError(format!("read {}: {e}", path.display())))?;
        UciFile::parse(&text).map_err(|e| {
            PlatformError(format!(
                "parse {}: line {} {}",
                path.display(),
                e.line,
                e.message
            ))
        })
    }

    /// Writes a configuration file atomically: a temporary file beside it, then a rename, which
    /// the sticky directory allows only onto the daemon's own files (M-5b).
    fn write_config(&self, name: &str, text: &str) -> Result<(), PlatformError> {
        let path = self.config_dir.join(name);
        let tmp = self.config_dir.join(format!(".{name}.enkrnett.tmp"));
        fs::write(&tmp, text)
            .map_err(|e| PlatformError(format!("write {}: {e}", tmp.display())))?;
        fs::rename(&tmp, &path)
            .map_err(|e| PlatformError(format!("rename {}: {e}", path.display())))
    }

    fn radios(&self) -> Result<Vec<Section>, PlatformError> {
        let wireless = self.read_config("wireless")?;
        Ok(wireless
            .sections()
            .filter(|s| s.kind == "wifi-device")
            .cloned()
            .collect())
    }

    /// The radio that carries Enkrnett's networks: the first 2.4 GHz one, else the first.
    fn choose_radio(radios: &[Section]) -> Result<String, PlatformError> {
        radios
            .iter()
            .find(|r| r.get("band") == Some("2g"))
            .or_else(|| radios.first())
            .and_then(|r| r.name.clone())
            .ok_or_else(|| PlatformError("no wifi-device section".into()))
    }

    fn country(&self, settings: Option<&Settings>) -> String {
        self.profile
            .country
            .fixed
            .clone()
            .or_else(|| settings.map(|s| s.country.clone()))
            .unwrap_or_else(|| "00".into())
    }

    /// Writes the owned sections for `phase`, reloads, and verifies.
    fn apply_phase(&self, phase: Phase) -> Result<Vec<HealthDetail>, PlatformError> {
        let radios = self.radios()?;
        let settings = match &phase {
            Phase::Claimed { settings, .. } => Some(settings),
            _ => None,
        };
        let plan = Plan {
            country: self.country(settings),
            radio: Self::choose_radio(&radios)?,
            phase: phase.clone(),
        };
        let owned = owned_sections(&plan, &radios);
        let wireless =
            self.read_config("wireless")?
                .rewrite(OWNED_PREFIX, &owned.wireless, &owned.radios);
        let network = self
            .read_config("network")?
            .replace_owned(OWNED_PREFIX, &owned.network);
        let dhcp = self
            .read_config("dhcp")?
            .replace_owned(OWNED_PREFIX, &owned.dhcp);
        let firewall = self
            .read_config("firewall")?
            .replace_owned(OWNED_PREFIX, &owned.firewall);
        self.write_config("wireless", &wireless)?;
        self.write_config("network", &network)?;
        self.write_config("dhcp", &dhcp)?;
        self.write_config("firewall", &firewall)?;
        self.call("network", "reload", json!({}))?;
        // netifd reloads network and wireless; dnsmasq and firewall4 reload on the config.change
        // event that `uci commit` would otherwise raise.
        for package in ["dhcp", "firewall"] {
            self.call(
                "service",
                "event",
                json!({"type": "config.change", "data": {"package": package}}),
            )?;
        }
        *self.phase.lock().expect("phase") = Some(phase.clone());
        log::event("platform_applied", &[("phase", &phase_name(&phase))]);
        Ok(self.health_check(&phase))
    }

    /// Waits for the networks of `phase` and reports what is missing (ENK-CFG-007).
    fn health_check(&self, phase: &Phase) -> Vec<HealthDetail> {
        let deadline = Instant::now() + HEALTH_TIMEOUT;
        let mut missing = self.missing(phase);
        while !missing.is_empty() && Instant::now() < deadline {
            std::thread::sleep(HEALTH_POLL);
            missing = self.missing(phase);
        }
        missing
    }

    fn interface_up(&self, name: &str) -> bool {
        self.call(&format!("network.interface.{name}"), "status", json!({}))
            .ok()
            .and_then(|m| m.get("up").and_then(Value::as_bool))
            .unwrap_or(false)
    }

    /// Every `hostapd.<iface>` object with its SSID and whether it is enabled.
    fn bss_objects(&self) -> Vec<(String, String, bool)> {
        self.objects("hostapd.")
            .into_iter()
            .filter_map(|object| {
                let status = self.call(&object, "get_status", json!({})).ok()?;
                let ssid = status.get("ssid").and_then(Value::as_str)?.to_string();
                let enabled = status.get("status").and_then(Value::as_str) == Some("ENABLED");
                Some((object, ssid, enabled))
            })
            .collect()
    }

    fn dnsmasq_instances_running(&self, names: &[&str]) -> bool {
        let Ok(list) = self.call("service", "list", json!({"name": "dnsmasq"})) else {
            return false;
        };
        let instances = list
            .get("dnsmasq")
            .and_then(|d| d.get("instances"))
            .and_then(Value::as_object);
        names.iter().all(|n| {
            instances
                .and_then(|i| i.get(*n))
                .and_then(|i| i.get("running"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
    }

    fn missing(&self, phase: &Phase) -> Vec<HealthDetail> {
        let mut out = Vec::new();
        for iface in plan::expected_interfaces(phase) {
            if !self.interface_up(iface) {
                out.push(if iface == "enk_guest" {
                    HealthDetail::GuestDown
                } else {
                    HealthDetail::ManagementDown
                });
            }
        }
        let running = self.bss_objects();
        for ExpectedBss { ssid, guest, .. } in plan::expected_bss(phase) {
            if !running.iter().any(|(_, s, enabled)| *s == ssid && *enabled) {
                out.push(if guest {
                    HealthDetail::GuestDown
                } else {
                    HealthDetail::ManagementDown
                });
            }
        }
        let mut instances = vec!["enk_dns_mgmt"];
        if matches!(phase, Phase::Claimed { settings, .. } if settings.guest.enabled) {
            instances.push("enk_dns_guest");
        }
        if !self.dnsmasq_instances_running(&instances) {
            out.push(HealthDetail::DhcpMissing);
        }
        out.dedup();
        out
    }

    // ----- readings ----------------------------------------------------------------------------

    fn clients(&self, object: &str) -> Map<String, Value> {
        self.call(object, "get_clients", json!({}))
            .ok()
            .and_then(|m| m.get("clients").and_then(Value::as_object).cloned())
            .unwrap_or_default()
    }

    fn mac_of(&self, client: IpAddr) -> Option<String> {
        let arp = fs::read_to_string("/proc/net/arp").ok()?;
        let wanted = client.to_string();
        arp.lines().skip(1).find_map(|line| {
            let mut cols = line.split_whitespace();
            let ip = cols.next()?;
            let mac = cols.nth(2)?;
            (ip == wanted && mac != "00:00:00:00:00:00").then(|| mac.to_ascii_lowercase())
        })
    }

    fn meminfo() -> Memory {
        let text = fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let field = |name: &str| -> u64 {
            text.lines()
                .find(|l| l.starts_with(name))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
        };
        Memory {
            total_kib: field("MemTotal:"),
            available_kib: field("MemAvailable:"),
        }
    }

    fn overlay() -> Storage {
        let path = std::ffi::CString::new("/overlay").expect("literal");
        let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
        // SAFETY: a valid NUL-terminated path and a zeroed statvfs buffer of the right type.
        let rc = unsafe { libc::statvfs(path.as_ptr(), &mut st) };
        if rc != 0 {
            return Storage {
                overlay_total_kib: 0,
                overlay_free_kib: 0,
            };
        }
        let frsize = st.f_frsize as u64;
        Storage {
            overlay_total_kib: st.f_blocks as u64 * frsize / 1024,
            overlay_free_kib: st.f_bavail as u64 * frsize / 1024,
        }
    }

    /// Listening TCP ports of the device (`/proc/net/tcp`, state 0A).
    fn listening_tcp_ports() -> Vec<u16> {
        let text = fs::read_to_string("/proc/net/tcp").unwrap_or_default();
        text.lines()
            .skip(1)
            .filter_map(|l| {
                let mut cols = l.split_whitespace();
                let local = cols.nth(1)?;
                let state = cols.nth(1)?;
                (state == "0A")
                    .then(|| u16::from_str_radix(local.rsplit(':').next()?, 16).ok())
                    .flatten()
            })
            .collect()
    }

    /// The hardening report (ENK-HARD-006). What the daemon cannot observe as its own user is
    /// reported as not met rather than guessed met.
    fn hardening(&self) -> Vec<HardeningItem> {
        let services = self.call("service", "list", json!({})).unwrap_or_default();
        let dropbear = services.contains_key("dropbear");
        let luci = Path::new("/usr/lib/lua/luci").exists()
            || Path::new("/www/luci-static").exists()
            || Path::new("/usr/share/ucode/luci").exists();
        let allowed: [u16; 3] = [53, 67, 80];
        let unexpected = Self::listening_tcp_ports()
            .into_iter()
            .any(|p| !allowed.contains(&p));
        let wps = self.read_config("wireless").map(|w| w.sections().any(|s| s.entries.iter().any(|e| matches!(e, enkrnett_model::uci::Entry::Option(k, _) if k.starts_with("wps"))))).unwrap_or(true);
        vec![
            HardeningItem {
                id: "ENK-HARD-001".into(),
                met: false,
            },
            HardeningItem {
                id: "ENK-HARD-002".into(),
                met: !dropbear,
            },
            HardeningItem {
                id: "ENK-HARD-003".into(),
                met: false,
            },
            HardeningItem {
                id: "ENK-HARD-004".into(),
                met: !luci,
            },
            HardeningItem {
                id: "ENK-HARD-005".into(),
                met: !unexpected,
            },
            HardeningItem {
                id: "ENK-HARD-008".into(),
                met: !wps,
            },
        ]
    }

    fn openwrt_version(&self) -> String {
        self.call("system", "board", json!({}))
            .ok()
            .and_then(|m| {
                m.get("release")
                    .and_then(|r| r.get("version"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_else(|| "unknown".into())
    }
}

fn phase_name(phase: &Phase) -> &'static str {
    match phase {
        Phase::Closed => "closed",
        Phase::Setup { .. } => "setup",
        Phase::Pending { .. } => "pending",
        Phase::Claimed { .. } => "claimed",
    }
}

impl Platform for OpenWrtPlatform {
    fn label_suffix(&self) -> String {
        let text = fs::read_to_string(&self.profile.label_address.source).unwrap_or_default();
        let hex: String = text
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .collect::<String>()
            .to_ascii_uppercase();
        if hex.len() >= 4 {
            hex[hex.len() - 4..].to_string()
        } else {
            "0000".into()
        }
    }

    fn has_light(&self) -> bool {
        self.profile.light.is_some()
    }

    fn blink(&self) -> Result<(), PlatformError> {
        let led = self
            .profile
            .light
            .as_ref()
            .ok_or_else(|| PlatformError("no light".into()))?
            .led
            .clone();
        let dir = PathBuf::from("/sys/class/leds").join(&led);
        let max = fs::read_to_string(dir.join("max_brightness"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(1);
        let before = fs::read_to_string(dir.join("brightness"))
            .map_err(|e| PlatformError(format!("led {led}: {e}")))?;
        std::thread::Builder::new()
            .name("enkrnettd-blink".into())
            .spawn(move || {
                for i in 0..10 {
                    let value = if i % 2 == 0 {
                        max.to_string()
                    } else {
                        "0".to_string()
                    };
                    let _ = fs::write(dir.join("brightness"), value);
                    std::thread::sleep(Duration::from_millis(300));
                }
                let _ = fs::write(dir.join("brightness"), before.trim());
            })
            .map_err(|e| PlatformError(e.to_string()))?;
        Ok(())
    }

    fn fixed_country(&self) -> Option<String> {
        self.profile.country.fixed.clone()
    }

    fn country_choices(&self) -> Vec<String> {
        COUNTRY_CHOICES.iter().map(|c| c.to_string()).collect()
    }

    fn management_address(&self) -> String {
        MANAGEMENT_ADDRESS.to_string()
    }

    fn connection(&self, client: IpAddr, state: OwnershipState) -> Connection {
        let network = match state {
            OwnershipState::Setup | OwnershipState::Unclaimed => ConnectionNetwork::Setup,
            _ => ConnectionNetwork::Management,
        };
        let Some(mac) = self.mac_of(client) else {
            return Connection {
                network,
                security: None,
            };
        };
        let phase = self.phase.lock().expect("phase").clone();
        let expected = phase.as_ref().map(plan::expected_bss).unwrap_or_default();
        for (object, ssid, _) in self.bss_objects() {
            if self.clients(&object).get(&mac).is_none() {
                continue;
            }
            let Some(bss) = expected.iter().find(|b| b.ssid == ssid) else {
                continue;
            };
            let entry = self.clients(&object).get(&mac).cloned().unwrap_or_default();
            let mfp = entry.get("mfp").and_then(Value::as_bool).unwrap_or(false);
            let security = match bss.role {
                BssRole::Open => ConnectionSecurity::None,
                BssRole::Owe => ConnectionSecurity::Owe,
                // sae-mixed: SAE requires management frame protection; a WPA2 phone rarely uses it.
                BssRole::Management => {
                    if mfp {
                        ConnectionSecurity::Wpa3
                    } else {
                        ConnectionSecurity::Wpa2
                    }
                }
            };
            return Connection {
                network,
                security: Some(security),
            };
        }
        Connection {
            network,
            security: None,
        }
    }

    fn enter_setup(&self) -> Result<(), PlatformError> {
        let suffix = self.label_suffix();
        let details = self.apply_phase(Phase::Setup {
            label_suffix: suffix,
        })?;
        if !details.is_empty() {
            log::event(
                "health_check_failed",
                &[("at", &"enter_setup"), ("detail", &format!("{details:?}"))],
            );
        }
        Ok(())
    }

    fn close_setup(&self) -> Result<(), PlatformError> {
        self.apply_phase(Phase::Closed).map(|_| ())
    }

    fn enter_pending(
        &self,
        management_name: &str,
        management_key: &str,
    ) -> Result<(), PlatformError> {
        let details = self.apply_phase(Phase::Pending {
            management_name: management_name.into(),
            management_key: management_key.into(),
        })?;
        if !details.is_empty() {
            log::event(
                "health_check_failed",
                &[
                    ("at", &"enter_pending"),
                    ("detail", &format!("{details:?}")),
                ],
            );
        }
        Ok(())
    }

    fn apply_settings(
        &self,
        management_name: &str,
        management_key: &str,
        settings: &Settings,
    ) -> Result<Vec<HealthDetail>, PlatformError> {
        self.apply_phase(Phase::Claimed {
            management_name: management_name.into(),
            management_key: management_key.into(),
            settings: settings.clone(),
        })
    }

    fn effective(&self) -> Effective {
        let phase = self.phase.lock().expect("phase").clone();
        let expected = phase.as_ref().map(plan::expected_bss).unwrap_or_default();
        let running = self.bss_objects();
        let object_of = |role: BssRole, guest: bool| -> Option<String> {
            let ssid = &expected
                .iter()
                .find(|b| b.role == role && b.guest == guest)?
                .ssid;
            running
                .iter()
                .find(|(_, s, enabled)| s == ssid && *enabled)
                .map(|(o, _, _)| o.clone())
        };
        let count = |object: &Option<String>, authorized_and_mfp: bool| -> u64 {
            object
                .as_ref()
                .map(|o| {
                    self.clients(o)
                        .values()
                        .filter(|c| {
                            let assoc = c.get("assoc").and_then(Value::as_bool).unwrap_or(false);
                            let authorized = c
                                .get("authorized")
                                .and_then(Value::as_bool)
                                .unwrap_or(false);
                            let mfp = c.get("mfp").and_then(Value::as_bool).unwrap_or(false);
                            if authorized_and_mfp {
                                authorized && mfp
                            } else {
                                assoc
                            }
                        })
                        .count() as u64
                })
                .unwrap_or(0)
        };
        let guest_mode = match &phase {
            Some(Phase::Claimed { settings, .. }) if settings.guest.enabled => {
                Some((settings.guest.mode, settings.guest.name.clone()))
            }
            _ => None,
        };
        let guest = match guest_mode {
            Some((mode, name)) => {
                let owe = object_of(BssRole::Owe, true);
                let open = object_of(BssRole::Open, true);
                let up = self.interface_up("enk_guest")
                    && owe.is_some()
                    && (mode == GuestMode::EncryptedOnly || open.is_some());
                GuestStatus {
                    up,
                    name: if up { Some(name) } else { None },
                    mode: if up { Some(mode) } else { None },
                    clients_encrypted: count(&owe, true),
                    clients_unencrypted: count(&open, false),
                    checked_ago: 0,
                }
            }
            None => GuestStatus {
                up: false,
                name: None,
                mode: None,
                clients_encrypted: 0,
                clients_unencrypted: 0,
                checked_ago: 0,
            },
        };
        let mgmt_object = object_of(BssRole::Management, false);
        let mgmt_up = self.interface_up("enk_mgmt") && mgmt_object.is_some();
        let management = ManagementStatus {
            up: mgmt_up,
            name: if mgmt_up {
                expected
                    .iter()
                    .find(|b| b.role == BssRole::Management)
                    .map(|b| b.ssid.clone())
            } else {
                None
            },
            address: MANAGEMENT_ADDRESS.into(),
            clients: count(&mgmt_object, false),
        };
        let wan = self
            .call("network.interface.wan", "status", json!({}))
            .unwrap_or_default();
        let uplink = UplinkStatus {
            up: wan.get("up").and_then(Value::as_bool).unwrap_or(false),
            address: wan
                .get("ipv4-address")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(|a| a.get("address"))
                .and_then(Value::as_str)
                .map(str::to_string),
        };
        Effective {
            guest,
            management,
            uplink,
        }
    }

    fn device_info(&self) -> DeviceInfo {
        let p = &self.profile;
        DeviceInfo {
            model: format!("{} {}", p.vendor, p.model),
            revision: p.revision.clone(),
            tier: p.tier,
            device_class: match p.device_class {
                enkrnett_model::profile::DeviceClass::Standard => "standard".into(),
                enkrnett_model::profile::DeviceClass::Constrained => "constrained".into(),
            },
            memory: Self::meminfo(),
            storage: Self::overlay(),
            hardening: self.hardening(),
            installed_as: if Path::new("/rom/usr/sbin/enkrnettd").exists() {
                InstalledAs::Image
            } else {
                InstalledAs::Package
            },
            radios: p
                .radios
                .iter()
                .map(|r| Radio {
                    band: r.band.clone(),
                    in_use: r.band == "2g",
                })
                .collect(),
            baseline: "25.12".into(),
            openwrt: self.openwrt_version(),
        }
    }

    fn check_update_index(&self) -> CheckResult {
        CheckResult::Unreachable
    }

    /// Erase: the marker the init script acts on at the next boot, and no Wi-Fi meanwhile.
    fn erase(&self) -> Result<(), PlatformError> {
        let _ = fs::create_dir_all(&self.state_dir);
        fs::write(
            self.state_dir.join(ERASE_MARKER),
            b"erase requested by the owner\n",
        )
        .map_err(|e| PlatformError(format!("erase marker: {e}")))?;
        self.apply_phase(Phase::Closed).map(|_| ())
    }

    fn restart(&self) -> Result<(), PlatformError> {
        self.call("system", "reboot", json!({})).map(|_| ())
    }
}
