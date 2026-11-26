//! The translation from the device's state and settings to the owned platform sections
//! (chapter 6 §3; ENK-CFG-001, 003). Pure: the daemon reads the current files, asks this module
//! for the sections, and writes them back with `uci::File::replace_owned`.
//!
//! Interface names are predictable so that the health check knows which `hostapd.*` objects to
//! expect. A Transition Mode pair is named by `wifi-scripts` after its network: `<network>-open`
//! and `<network>-owe` (the open BSS and the hidden OWE BSS); a single BSS gets an explicit
//! `ifname`: `enk-mgmt`, `enk-guest`.

use std::net::Ipv4Addr;

use crate::settings::{GuestMode, Settings};
use crate::subnet::Addresses;
use crate::uci::Section;

/// Every section the daemon owns starts with this (chapter 6 §3).
pub const OWNED_PREFIX: &str = "enk_";

/// The management UI's address (ENK-MGMT-007), the same on the setup and the management network,
/// unless the uplink collides with its subnet (`crate::subnet::renumber`).
pub const MANAGEMENT_ADDRESS: &str = "192.168.207.1";
/// Both Wi-Fi networks are /24 (`crate::subnet::PREFIX`).
pub const NETMASK: &str = "255.255.255.0";
/// The port the daemon listens on. Port 80 on the management interface is redirected to it by the
/// firewall, so that the operator types the address alone (ENK-MGMT-007) and the daemon needs no
/// capability: `procd` grants capabilities only inside a `ujail`, which the constrained target
/// lacks (chapter 2 §6).
pub const LISTEN_PORT: u16 = 8080;
/// The guest network's default address; renumbered when the uplink collides (ENK-TOPO-003, 006).
pub const GUEST_ADDRESS: &str = "10.207.0.1";

/// The setup network's name ends with the label suffix (ENK-OWN-004).
pub fn setup_network_name(label_suffix: &str) -> String {
    format!("Enkrnett Setup {label_suffix}")
}

/// The management network's name the device proposes at setup.
pub fn management_network_name(label_suffix: &str) -> String {
    format!("{}-{label_suffix}", crate::PRODUCT)
}

/// Which networks exist (chapter 2 §1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    /// `UNCLAIMED`: no Wi-Fi network at all (the setup window closed, ENK-OWN-003).
    Closed,
    /// `SETUP`: the setup network only, on the management subnet, no Internet.
    Setup { label_suffix: String },
    /// `PENDING`: the management network only.
    Pending {
        management_name: String,
        management_key: String,
    },
    /// `CLAIMED`: the management network and, when enabled, the guest network.
    Claimed {
        management_name: String,
        management_key: String,
        settings: Settings,
    },
}

/// What the translation needs to know about the device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub country: String,
    /// The `wifi-device` section that carries Enkrnett's networks (a 2.4 GHz radio).
    pub radio: String,
    pub phase: Phase,
    /// The device's address on each network, after renumbering (ENK-TOPO-003).
    pub addresses: Addresses,
}

/// The owned sections per configuration file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Owned {
    /// The `wifi-device` sections, replaced in place (`uci::File::rewrite`).
    pub radios: Vec<Section>,
    /// The owned `wifi-iface` sections.
    pub wireless: Vec<Section>,
    pub network: Vec<Section>,
    pub dhcp: Vec<Section>,
    pub firewall: Vec<Section>,
}

/// How a BSS is protected, as the health check and the connection line need to know it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BssRole {
    /// The open half of a Transition Mode pair: no encryption.
    Open,
    /// An OWE BSS (the hidden half of a pair, or the only BSS in `encrypted_only`).
    Owe,
    /// The management network (WPA2/WPA3-Personal).
    Management,
}

/// A BSS the platform must be running for a phase. `hostapd.*` objects are found by their SSID,
/// because `wifi-scripts` names a Transition Mode pair `phyN-apM` (M-6 material; verified on the
/// VM 2026-11-26): the open BSS carries the name, the hidden OWE BSS the name followed by `OWE`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpectedBss {
    pub ssid: String,
    pub role: BssRole,
    /// Whether the BSS belongs to the guest network (else to the management or setup network).
    pub guest: bool,
}

/// The suffix `wifi-scripts` gives the hidden OWE BSS of a Transition Mode pair.
pub const OWE_HIDDEN_SUFFIX: &str = "OWE";

fn owe_pair(name: &str, guest: bool) -> [ExpectedBss; 2] {
    [
        ExpectedBss {
            ssid: name.to_string(),
            role: BssRole::Open,
            guest,
        },
        ExpectedBss {
            ssid: format!("{name}{OWE_HIDDEN_SUFFIX}"),
            role: BssRole::Owe,
            guest,
        },
    ]
}

/// The BSSes the health check expects for a phase (ENK-CFG-007).
pub fn expected_bss(phase: &Phase) -> Vec<ExpectedBss> {
    match phase {
        Phase::Closed => Vec::new(),
        Phase::Setup { label_suffix } => {
            owe_pair(&setup_network_name(label_suffix), false).to_vec()
        }
        Phase::Pending {
            management_name, ..
        } => vec![ExpectedBss {
            ssid: management_name.clone(),
            role: BssRole::Management,
            guest: false,
        }],
        Phase::Claimed {
            management_name,
            settings,
            ..
        } => {
            let mut v = vec![ExpectedBss {
                ssid: management_name.clone(),
                role: BssRole::Management,
                guest: false,
            }];
            if settings.guest.enabled {
                match settings.guest.mode {
                    GuestMode::OpenToAll => v.extend(owe_pair(&settings.guest.name, true)),
                    GuestMode::EncryptedOnly => v.push(ExpectedBss {
                        ssid: settings.guest.name.clone(),
                        role: BssRole::Owe,
                        guest: true,
                    }),
                }
            }
            v
        }
    }
}

/// The bridges of a phase with the address each carries: every port of these bridges must have
/// the kernel's `isolated` flag (ENK-CFG-023, ENK-ISO-002) and each address must have its own
/// DNS listener (ENK-CFG-024).
pub fn expected_bridges(plan: &Plan) -> Vec<(&'static str, Ipv4Addr)> {
    let mut v = vec![("br-mgmt", plan.addresses.management)];
    if matches!(&plan.phase, Phase::Claimed { settings, .. } if settings.guest.enabled) {
        v.push(("br-guest", plan.addresses.guest));
    }
    v
}

/// The `network.interface.*` names that must be up for a phase.
pub fn expected_interfaces(phase: &Phase) -> Vec<&'static str> {
    match phase {
        Phase::Closed | Phase::Setup { .. } | Phase::Pending { .. } => vec!["enk_mgmt"],
        Phase::Claimed { settings, .. } => {
            if settings.guest.enabled {
                vec!["enk_mgmt", "enk_guest"]
            } else {
                vec!["enk_mgmt"]
            }
        }
    }
}

/// A `wifi-device` section as the daemon writes it: the hardware options of the existing section
/// (type, path, band, channel, …) kept, the country set on every radio (ENK-CFG-003), the chosen
/// radio enabled.
pub fn radio_section(existing: &Section, country: &str, in_use: bool) -> Section {
    let mut out = Section {
        kind: existing.kind.clone(),
        name: existing.name.clone(),
        entries: Vec::new(),
    };
    for e in &existing.entries {
        match e {
            crate::uci::Entry::Option(k, _) if k == "country" || (k == "disabled" && in_use) => {}
            other => out.entries.push(other.clone()),
        }
    }
    out.entries.push(crate::uci::Entry::Option(
        "country".into(),
        country.to_string(),
    ));
    if in_use {
        out.entries
            .push(crate::uci::Entry::Option("disabled".into(), "0".into()));
    }
    out
}

fn owe_iface(
    name: &str,
    ifname: &str,
    radio: &str,
    network: &str,
    ssid: &str,
    transition: bool,
) -> Section {
    let mut s = Section::named("wifi-iface", name)
        .option("device", radio)
        .option("mode", "ap");
    if transition {
        s = s.option("owe_transition", "1");
    } else {
        s = s.option("ifname", ifname);
    }
    s.option("network", network)
        .option("ssid", ssid)
        .option("encryption", "owe")
        .option("isolate", "1")
        .option("bridge_isolate", "1")
}

/// A bridge that stays up with no ports (`bridge_empty`): its address, the management UI's
/// address, must exist before the first Wi-Fi interface joins and while hostapd restarts.
fn bridge(name: &str, device: &str) -> Section {
    Section::named("device", name)
        .option("name", device)
        .option("type", "bridge")
        .option("bridge_empty", "1")
        .option("ipv6", "0")
}

fn interface(name: &str, device: &str, address: &str, netmask: &str) -> Section {
    Section::named("interface", name)
        .option("device", device)
        .option("proto", "static")
        .option("ipaddr", address)
        .option("netmask", netmask)
        .option("delegate", "0")
}

fn dnsmasq(
    name: &str,
    interface_name: &str,
    leasefile: &str,
    loopback: bool,
    address: Option<&str>,
) -> Section {
    let mut s = Section::named("dnsmasq", name)
        .option("domainneeded", "1")
        .option("localservice", "1")
        .option("nonwildcard", "1")
        .option("rebind_protection", "1")
        .option("localuse", "0")
        .option("leasefile", leasefile)
        .option("resolvfile", "/tmp/resolv.conf.d/resolv.conf.auto")
        .list("interface", &[interface_name]);
    if !loopback {
        s = s.list("notinterface", &["loopback"]);
    }
    if let Some(a) = address {
        s = s.list("address", &[a]);
    }
    s
}

fn dhcp_pool(name: &str, interface_name: &str, instance: &str, leasetime: &str) -> Section {
    Section::named("dhcp", name)
        .option("interface", interface_name)
        .option("instance", instance)
        .option("start", "100")
        .option("limit", "150")
        .option("leasetime", leasetime)
        .option("dhcpv6", "disabled")
        .option("ra", "disabled")
}

const PRIVATE_RANGES: [&str; 5] = [
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "169.254.0.0/16",
    "100.64.0.0/10",
];

fn zone(name: &str, network: &str) -> Section {
    Section::named("zone", name)
        .option("name", name)
        .list("network", &[network])
        .option("input", "REJECT")
        .option("output", "ACCEPT")
        .option("forward", "REJECT")
}

/// The input and forward rules of one zone. DNS and the UI are accepted towards the network's
/// own address only, so that a client of one network cannot reach the other network's
/// instance on the device (chapter 2 §3: each network sees its own services). DHCP is
/// broadcast and stays unbound.
fn zone_rules(
    zone_name: &str,
    prefix: &str,
    address: &str,
    with_ui: bool,
    internet: bool,
) -> Vec<Section> {
    let mut v = vec![
        Section::named("rule", &format!("{prefix}_no_site"))
            .option(
                "name",
                format!("{zone_name}: no private or uplink addresses"),
            )
            .option("src", zone_name)
            .option("dest", "wan")
            .list("dest_ip", &PRIVATE_RANGES)
            .option("proto", "all")
            .option("target", "REJECT"),
        Section::named("rule", &format!("{prefix}_dhcp"))
            .option("src", zone_name)
            .option("proto", "udp")
            .option("dest_port", "67")
            .option("target", "ACCEPT"),
        Section::named("rule", &format!("{prefix}_dns"))
            .option("src", zone_name)
            .option("proto", "tcp udp")
            .option("dest_ip", address)
            .option("dest_port", "53")
            .option("target", "ACCEPT"),
        Section::named("rule", &format!("{prefix}_no_ip6_in"))
            .option("family", "ipv6")
            .option("src", zone_name)
            .option("proto", "all")
            .option("target", "DROP"),
        Section::named("rule", &format!("{prefix}_no_ip6_fwd"))
            .option("family", "ipv6")
            .option("src", zone_name)
            .option("dest", "*")
            .option("proto", "all")
            .option("target", "DROP"),
    ];
    if with_ui {
        v.push(
            Section::named("redirect", &format!("{prefix}_redirect"))
                .option("name", "management UI and API: port 80 to the daemon")
                .option("src", zone_name)
                .option("proto", "tcp")
                .option("src_dip", address)
                .option("src_dport", "80")
                .option("dest_ip", address)
                .option("dest_port", LISTEN_PORT.to_string())
                .option("target", "DNAT"),
        );
        v.push(
            Section::named("rule", &format!("{prefix}_ui"))
                .option("name", "management UI and API")
                .option("src", zone_name)
                .option("proto", "tcp")
                .option("dest_ip", address)
                .option("dest_port", format!("80 {LISTEN_PORT}"))
                .option("target", "ACCEPT"),
        );
    }
    if internet {
        v.push(
            Section::named("forwarding", &format!("{prefix}_fwd"))
                .option("src", zone_name)
                .option("dest", "wan"),
        );
    }
    v
}

/// The owned sections for a plan. `radios` are the existing `wifi-device` sections, every one of
/// which is rewritten with the country.
pub fn owned_sections(plan: &Plan, radios: &[Section]) -> Owned {
    let mut owned = Owned::default();
    for r in radios {
        let in_use = r.name.as_deref() == Some(plan.radio.as_str());
        owned.radios.push(radio_section(r, &plan.country, in_use));
    }
    // The management bridge and interface exist in every phase; the setup network uses them too
    // (same address, ENK-MGMT-007), so the setup client finds the UI at the one address.
    let mgmt = plan.addresses.management.to_string();
    let guest_address = plan.addresses.guest.to_string();
    owned.network.push(bridge("enk_br_mgmt", "br-mgmt"));
    owned
        .network
        .push(interface("enk_mgmt", "br-mgmt", &mgmt, NETMASK));
    owned.dhcp.push(dnsmasq(
        "enk_dns_mgmt",
        "enk_mgmt",
        "/tmp/dhcp.leases.enk_mgmt",
        true,
        None,
    ));
    owned
        .dhcp
        .push(dhcp_pool("enk_mgmt", "enk_mgmt", "enk_dns_mgmt", "1h"));
    owned.firewall.push(zone("enk_mgmt", "enk_mgmt"));
    match &plan.phase {
        Phase::Closed => {
            owned
                .firewall
                .extend(zone_rules("enk_mgmt", "enk_mgmt", &mgmt, true, false));
        }
        Phase::Setup { label_suffix } => {
            owned.wireless.push(owe_iface(
                "enk_setup",
                "enk-setup",
                &plan.radio,
                "enk_mgmt",
                &setup_network_name(label_suffix),
                true,
            ));
            // No Internet for setup clients (chapter 2 §2).
            owned
                .firewall
                .extend(zone_rules("enk_mgmt", "enk_mgmt", &mgmt, true, false));
        }
        Phase::Pending {
            management_name,
            management_key,
        } => {
            owned.wireless.push(management_iface(
                &plan.radio,
                management_name,
                management_key,
            ));
            owned
                .firewall
                .extend(zone_rules("enk_mgmt", "enk_mgmt", &mgmt, true, true));
        }
        Phase::Claimed {
            management_name,
            management_key,
            settings,
        } => {
            owned.wireless.push(management_iface(
                &plan.radio,
                management_name,
                management_key,
            ));
            owned
                .firewall
                .extend(zone_rules("enk_mgmt", "enk_mgmt", &mgmt, true, true));
            if settings.guest.enabled {
                let transition = settings.guest.mode == GuestMode::OpenToAll;
                owned.wireless.push(owe_iface(
                    "enk_guest",
                    "enk-guest",
                    &plan.radio,
                    "enk_guest",
                    &settings.guest.name,
                    transition,
                ));
                owned.network.push(bridge("enk_br_guest", "br-guest"));
                owned
                    .network
                    .push(interface("enk_guest", "br-guest", &guest_address, NETMASK));
                owned.dhcp.push(dnsmasq(
                    "enk_dns_guest",
                    "enk_guest",
                    "/tmp/dhcp.leases.enk_guest",
                    false,
                    None,
                ));
                owned
                    .dhcp
                    .push(dhcp_pool("enk_guest", "enk_guest", "enk_dns_guest", "2h"));
                owned.firewall.push(zone("enk_guest", "enk_guest"));
                owned.firewall.extend(zone_rules(
                    "enk_guest",
                    "enk_guest",
                    &guest_address,
                    false,
                    true,
                ));
            }
        }
    }
    owned
}

fn management_iface(radio: &str, name: &str, key: &str) -> Section {
    Section::named("wifi-iface", "enk_mgmt")
        .option("device", radio)
        .option("mode", "ap")
        .option("ifname", "enk-mgmt")
        .option("network", "enk_mgmt")
        .option("ssid", name)
        .option("encryption", "sae-mixed")
        .option("key", key)
        .option("ieee80211w", "1")
        .option("isolate", "1")
        .option("bridge_isolate", "1")
}

#[cfg(test)]
mod tests;
