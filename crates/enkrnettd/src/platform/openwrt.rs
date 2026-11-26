//! The OpenWrt platform (chapter 6; RFC 014): the daemon's user owns the four configuration
//! files it writes (measurement M-5b), replaces its own sections through `enkrnett_model::uci`,
//! asks `netifd` to reload through ubus, and reads the effective state back through ubus and
//! `/proc`. Nothing here runs as root and nothing spawns a process.
//!
//! The health check treats the platform's options as requests and the kernel's and the services'
//! reported state as the truth (ENK-CFG-026): interfaces and BSSes through ubus, bridge-port
//! isolation through sysfs (ENK-CFG-023), DNS listeners and the daemon's own listener through
//! `/proc/net` (ENK-CFG-024, 025). Not yet: renumbering on an uplink collision (ENK-TOPO-003,
//! 006).

use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use enkrnett_model::api::{
    CheckResult, Connection, GuestStatus, HardeningItem, HealthDetail, InstalledAs,
    ManagementStatus, Memory, Radio, Storage, UplinkStatus,
};
use enkrnett_model::plan::{
    self, BssRole, ExpectedBss, LISTEN_PORT, OWNED_PREFIX, Phase, Plan, owned_sections,
};
use enkrnett_model::profile::DeviceProfile;
use enkrnett_model::settings::{GuestMode, Settings};
use enkrnett_model::state::{ConnectionNetwork, ConnectionSecurity, OwnershipState};
use enkrnett_model::subnet::{Addresses, Subnet, renumber};
use enkrnett_model::uci::{File as UciFile, Section};
use enkrnett_ubus::client::Client;
use serde_json::{Map, Value, json};

use super::{Adopted, DeviceInfo, Effective, Platform, PlatformError};
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
    /// What the device runs now: the phase and the addresses, set when the daemon applied or
    /// adopted it.
    current: Mutex<Option<Plan>>,
    /// The management address the daemon's listener is about to leave: set when an apply moved
    /// the management network (ENK-TOPO-006), so that the health check does not count the
    /// daemon's own listener there as exposed while the restart is pending.
    leaving: Mutex<Option<Ipv4Addr>>,
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
            current: Mutex::new(None),
            leaving: Mutex::new(None),
        }
    }

    // ----- ubus ------------------------------------------------------------------------------

    fn ubus(&self) -> Result<Client, PlatformError> {
        match &self.ubus_socket {
            Some(p) => Client::connect_path(p),
            None => Client::connect(),
        }
        .map_err(|e| PlatformError::Failed(format!("ubus: {e}")))
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
            .map_err(|e| PlatformError::Failed(format!("ubus {object} {method}: {e}")))
    }

    fn objects(&self, prefix: &str) -> Vec<String> {
        match self.ubus().and_then(|mut c| {
            c.lookup("")
                .map_err(|e| PlatformError::Failed(e.to_string()))
        }) {
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
        let text = fs::read_to_string(&path).map_err(|e| {
            let m = format!("read {}: {e}", path.display());
            // A root-side `uci commit` leaves the file root's and 0600: not even readable.
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                PlatformError::NotOurs(m)
            } else {
                PlatformError::Failed(m)
            }
        })?;
        UciFile::parse(&text).map_err(|e| {
            PlatformError::Failed(format!(
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
        // 0600: `wireless` carries the management key (ENK-OWN-011 in spirit); the others follow.
        let not_ours = |e: &std::io::Error| e.kind() == std::io::ErrorKind::PermissionDenied;
        crate::store::write_private(&tmp, text.as_bytes()).map_err(|e| {
            let m = format!("write {}: {e}", tmp.display());
            if not_ours(&e) {
                PlatformError::NotOurs(m)
            } else {
                PlatformError::Failed(m)
            }
        })?;
        fs::rename(&tmp, &path).map_err(|e| {
            let m = format!("rename {}: {e}", path.display());
            if not_ours(&e) {
                PlatformError::NotOurs(m)
            } else {
                PlatformError::Failed(m)
            }
        })
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
            .ok_or_else(|| PlatformError::Failed("no wifi-device section".into()))
    }

    fn country(&self, settings: Option<&Settings>) -> String {
        self.profile
            .country
            .fixed
            .clone()
            .or_else(|| settings.map(|s| s.country.clone()))
            .unwrap_or_else(|| "00".into())
    }

    /// The uplink's subnet as `netifd` reports it (ENK-TOPO-003).
    fn uplink_subnet(&self) -> Option<Subnet> {
        let wan = self
            .call("network.interface.wan", "status", json!({}))
            .ok()?;
        let a = wan.get("ipv4-address")?.as_array()?.first()?;
        let address = a.get("address")?.as_str()?;
        let mask = u8::try_from(a.get("mask")?.as_u64()?).ok()?;
        Subnet::parse(address, mask)
    }

    /// The addresses the device uses now: the current plan's, else what the owned `network`
    /// sections on flash say, else the defaults.
    fn current_addresses(&self) -> Addresses {
        if let Some(p) = self.current.lock().expect("current").as_ref() {
            return p.addresses;
        }
        let mut a = Addresses::default();
        if let Ok(network) = self.read_config("network") {
            for (section, slot) in [("enk_mgmt", &mut a.management), ("enk_guest", &mut a.guest)] {
                if let Some(ip) = network
                    .named_section(section)
                    .and_then(|s| s.get("ipaddr"))
                    .and_then(|v| v.parse().ok())
                {
                    *slot = ip;
                }
            }
        }
        a
    }

    /// The plan for `phase`: radios and country, and the addresses after renumbering.
    fn make_plan(&self, phase: Phase) -> Result<(Plan, Vec<Section>), PlatformError> {
        let radios = self.radios()?;
        let settings = match &phase {
            Phase::Claimed { settings, .. } => Some(settings),
            _ => None,
        };
        let current = self.current_addresses();
        let addresses = renumber(current, self.uplink_subnet());
        if addresses != current {
            log::event(
                "renumbered",
                &[
                    ("management", &addresses.management),
                    ("guest", &addresses.guest),
                    ("uplink", &format!("{:?}", self.uplink_subnet())),
                ],
            );
        }
        let plan = Plan {
            country: self.country(settings),
            radio: Self::choose_radio(&radios)?,
            phase,
            addresses,
        };
        Ok((plan, radios))
    }

    /// Whether the owned sections on flash differ from what `plan` produces (ENK-CFG-014).
    fn differs(&self, plan: &Plan, radios: &[Section]) -> Result<bool, PlatformError> {
        let owned = owned_sections(plan, radios);
        let wireless = self.read_config("wireless")?;
        let radios_equal = owned.radios.iter().all(|r| {
            r.name
                .as_deref()
                .and_then(|n| wireless.named_section(n))
                .is_some_and(|existing| existing.equivalent(r))
        });
        Ok(!(radios_equal
            && wireless.owned_equal(OWNED_PREFIX, &owned.wireless)
            && self
                .read_config("network")?
                .owned_equal(OWNED_PREFIX, &owned.network)
            && self
                .read_config("dhcp")?
                .owned_equal(OWNED_PREFIX, &owned.dhcp)
            && self
                .read_config("firewall")?
                .owned_equal(OWNED_PREFIX, &owned.firewall)))
    }

    /// Writes the owned sections for `phase`, reloads, and verifies.
    fn apply_phase(&self, phase: Phase) -> Result<Vec<HealthDetail>, PlatformError> {
        let (plan, radios) = self.make_plan(phase)?;
        self.write_plan(&plan, &radios)?;
        Ok(self.health_check(&plan))
    }

    fn write_plan(&self, plan: &Plan, radios: &[Section]) -> Result<(), PlatformError> {
        let owned = owned_sections(plan, radios);
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
        let previous = self.current.lock().expect("current").replace(plan.clone());
        if let Some(previous) = previous
            && previous.addresses.management != plan.addresses.management
        {
            *self.leaving.lock().expect("leaving") = Some(previous.addresses.management);
        }
        log::event("platform_applied", &[("phase", &phase_name(&plan.phase))]);
        Ok(())
    }

    /// Waits for the networks of `phase` and reports what is missing (ENK-CFG-007).
    fn health_check(&self, plan: &Plan) -> Vec<HealthDetail> {
        let deadline = Instant::now() + HEALTH_TIMEOUT;
        let mut missing = self.missing(plan);
        while !missing.is_empty() && Instant::now() < deadline {
            std::thread::sleep(HEALTH_POLL);
            missing = self.missing(plan);
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

    fn missing(&self, plan: &Plan) -> Vec<HealthDetail> {
        let phase = &plan.phase;
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
        // The kernel's view (ENK-CFG-026): the ports really isolated, DNS really listening on
        // each network's address, the daemon's listener on the management address only.
        let udp = sockets("/proc/net/udp", None);
        for (bridge, address) in plan::expected_bridges(plan) {
            if bridge_ports(bridge).iter().any(|(_, isolated)| !isolated) {
                out.push(HealthDetail::IsolationMissing);
            }
            if !udp.iter().any(|s| s.port == 53 && s.address == address) {
                out.push(HealthDetail::DhcpMissing);
            }
        }
        if listener_exposed(
            &sockets("/proc/net/tcp", Some("0A")),
            plan.addresses.management,
            *self.leaving.lock().expect("leaving"),
        ) {
            out.push(HealthDetail::ListenerExposed);
        }
        out.sort_by_key(|d| *d as u8);
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

    /// The hardening report (ENK-HARD-006). What the daemon cannot observe as its own user is
    /// reported as not met rather than guessed met.
    fn hardening(&self) -> Vec<HardeningItem> {
        let services = self.call("service", "list", json!({})).unwrap_or_default();
        let dropbear = services.contains_key("dropbear");
        let luci = Path::new("/usr/lib/lua/luci").exists()
            || Path::new("/www/luci-static").exists()
            || Path::new("/usr/share/ucode/luci").exists();
        // ENK-HARD-005: nothing listens but DNS, DHCP, and the daemon on the management address.
        let mgmt = self.current_addresses().management;
        let unexpected = sockets("/proc/net/tcp", Some("0A"))
            .into_iter()
            .chain(sockets("/proc/net/tcp6", Some("0A")))
            .chain(sockets("/proc/net/udp", None))
            .chain(sockets("/proc/net/udp6", None))
            .any(|s| {
                !(s.port == 53 || s.port == 67 || (s.port == LISTEN_PORT && s.address == mgmt))
            });
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
            .ok_or_else(|| PlatformError::Failed("no light".into()))?
            .led
            .clone();
        let dir = PathBuf::from("/sys/class/leds").join(&led);
        let max = fs::read_to_string(dir.join("max_brightness"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(1);
        let before = fs::read_to_string(dir.join("brightness"))
            .map_err(|e| PlatformError::Failed(format!("led {led}: {e}")))?;
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
            .map_err(|e| PlatformError::Failed(e.to_string()))?;
        Ok(())
    }

    fn fixed_country(&self) -> Option<String> {
        self.profile.country.fixed.clone()
    }

    fn country_choices(&self) -> Vec<String> {
        COUNTRY_CHOICES.iter().map(|c| c.to_string()).collect()
    }

    fn listen_follows_management_address(&self) -> bool {
        true
    }

    fn management_address(&self) -> String {
        self.current_addresses().management.to_string()
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
        let current = self.current.lock().expect("current").clone();
        let expected = current
            .as_ref()
            .map(|p| plan::expected_bss(&p.phase))
            .unwrap_or_default();
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

    fn adopt_settings(
        &self,
        management_name: &str,
        management_key: &str,
        settings: &Settings,
    ) -> Result<Adopted, PlatformError> {
        let phase = Phase::Claimed {
            management_name: management_name.into(),
            management_key: management_key.into(),
            settings: settings.clone(),
        };
        let before = self.current_addresses();
        let (plan, radios) = self.make_plan(phase)?;
        if plan.addresses != before {
            // ENK-TOPO-003: the colliding network is renumbered; this is the daemon's own work,
            // not drift.
            self.write_plan(&plan, &radios)?;
            return Ok(Adopted {
                health: self.health_check(&plan),
                drift: false,
            });
        }
        let drift = self.differs(&plan, &radios)?;
        *self.current.lock().expect("current") = Some(plan.clone());
        log::event(
            "platform_adopted",
            &[("phase", &phase_name(&plan.phase)), ("drift", &drift)],
        );
        Ok(Adopted {
            health: self.health_check(&plan),
            drift,
        })
    }

    fn owned_drift(&self) -> Result<bool, PlatformError> {
        let Some(plan) = self.current.lock().expect("current").clone() else {
            return Ok(false);
        };
        let radios = self.radios()?;
        self.differs(&plan, &radios)
    }

    fn uplink_collides(&self) -> bool {
        let Some(plan) = self.current.lock().expect("current").clone() else {
            return false;
        };
        renumber(plan.addresses, self.uplink_subnet()) != plan.addresses
    }

    fn disconnect_management_stations(&self) -> Result<(), PlatformError> {
        let current = self.current.lock().expect("current").clone();
        let expected = current
            .as_ref()
            .map(|p| plan::expected_bss(&p.phase))
            .unwrap_or_default();
        let Some(ssid) = expected
            .iter()
            .find(|b| b.role == BssRole::Management && !b.guest)
            .map(|b| b.ssid.clone())
        else {
            return Ok(());
        };
        let objects: Vec<String> = self
            .bss_objects()
            .into_iter()
            .filter(|(_, s, _)| *s == ssid)
            .map(|(o, _, _)| o)
            .collect();
        let mut disconnected = 0u64;
        for object in objects {
            for addr in self.clients(&object).keys() {
                // reason 3: deauthenticated because the sending station is leaving.
                self.call(
                    &object,
                    "del_client",
                    json!({"addr": addr, "reason": 3, "deauth": true, "ban_time": 0}),
                )?;
                disconnected += 1;
            }
        }
        log::event("management_disconnected", &[("stations", &disconnected)]);
        Ok(())
    }

    fn effective(&self) -> Effective {
        let current = self.current.lock().expect("current").clone();
        let expected = current
            .as_ref()
            .map(|p| plan::expected_bss(&p.phase))
            .unwrap_or_default();
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
        let guest_mode = match current.as_ref().map(|p| &p.phase) {
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
            address: current
                .as_ref()
                .map(|p| p.addresses.management)
                .unwrap_or(Addresses::default().management)
                .to_string(),
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
        .map_err(|e| PlatformError::Failed(format!("erase marker: {e}")))?;
        self.apply_phase(Phase::Closed).map(|_| ())
    }

    /// A `config.change` event for the package `enkrnett`: procd runs the init script's reload
    /// trigger (`procd_add_reload_trigger "enkrnett"`), which is `restart`: stop, then
    /// `start_service` with the ownership fixed and a fresh command line. procd's own `service`
    /// object serves it, so a minimal image needs nothing else (the `rc` object would be rpcd's).
    fn restart_service(&self) -> Result<(), PlatformError> {
        self.call(
            "service",
            "event",
            json!({"type": "config.change", "data": {"package": "enkrnett"}}),
        )
        .map(|_| ())
    }

    fn restart(&self) -> Result<(), PlatformError> {
        self.call("system", "reboot", json!({})).map(|_| ())
    }
}

/// A bound socket as `/proc/net/{tcp,udp}` lists it. IPv6 sockets report the mapped or
/// unspecified address as `0.0.0.0`, which is what matters: "bound to every address".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundSocket {
    pub address: Ipv4Addr,
    pub port: u16,
    pub uid: u32,
}

/// Parses `/proc/net/tcp`-style text. `state` keeps only that state (`0A` = LISTEN); `None`
/// keeps every line (UDP sockets have no listening state).
pub fn parse_sockets(text: &str, state: Option<&str>) -> Vec<BoundSocket> {
    text.lines()
        .skip(1)
        .filter_map(|l| {
            let cols: Vec<&str> = l.split_whitespace().collect();
            let local = cols.get(1)?;
            let st = cols.get(3)?;
            if state.is_some_and(|wanted| *st != wanted) {
                return None;
            }
            let (addr, port) = local.rsplit_once(':')?;
            let port = u16::from_str_radix(port, 16).ok()?;
            let uid = cols.get(7)?.parse().ok()?;
            let address = match addr.len() {
                // little-endian host order, as the kernel prints it
                8 => Ipv4Addr::from(u32::from_str_radix(addr, 16).ok()?.swap_bytes()),
                32 => {
                    // IPv6: the last 32 bits hold a mapped IPv4 address; anything else counts
                    // as "every address".
                    let v4 = u32::from_str_radix(&addr[24..32], 16).ok()?;
                    if &addr[0..24] == "0000000000000000FFFF0000" {
                        Ipv4Addr::from(v4.swap_bytes())
                    } else {
                        Ipv4Addr::UNSPECIFIED
                    }
                }
                _ => return None,
            };
            Some(BoundSocket { address, port, uid })
        })
        .collect()
}

fn sockets(path: &str, state: Option<&str>) -> Vec<BoundSocket> {
    parse_sockets(&fs::read_to_string(path).unwrap_or_default(), state)
}

/// ENK-CFG-025: the daemon's port is bound to the management address and to nothing else;
/// nothing else answers on the UI's ports. Whatever listens on every address or on the guest
/// address at port 80 or the daemon's port is exposed to guests. The daemon's own listener on
/// the management address it is `leaving` (the network moved, the restart is pending,
/// ENK-TOPO-006) is not: that address is on no interface any more.
pub fn listener_exposed(
    listening: &[BoundSocket],
    mgmt: Ipv4Addr,
    leaving: Option<Ipv4Addr>,
) -> bool {
    listening.iter().any(|s| {
        (s.port == LISTEN_PORT || s.port == 80)
            && s.address != mgmt
            && !(s.port == LISTEN_PORT && Some(s.address) == leaving)
    })
}

/// The ports of a bridge with their kernel `isolated` flag (ENK-CFG-023).
pub fn bridge_ports(bridge: &str) -> Vec<(String, bool)> {
    bridge_ports_in(Path::new("/sys/class/net"), bridge)
}

pub fn bridge_ports_in(sys_class_net: &Path, bridge: &str) -> Vec<(String, bool)> {
    let dir = sys_class_net.join(bridge).join("brif");
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut ports: Vec<(String, bool)> = entries
        .filter_map(|e| e.ok())
        .map(|e| {
            let isolated = fs::read_to_string(e.path().join("isolated"))
                .map(|t| t.trim() == "1")
                .unwrap_or(false);
            (e.file_name().to_string_lossy().into_owned(), isolated)
        })
        .collect();
    ports.sort();
    ports
}
