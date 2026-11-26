//! ENK-CFG-001, 003; chapter 2 §1 to §3 as configuration.

use super::*;
use crate::settings::{GuestSettings, ManagementSettings};
use crate::uci::{File, Section};

fn radios() -> Vec<Section> {
    vec![
        Section::named("wifi-device", "radio0")
            .option("type", "mac80211")
            .option("path", "virtual/mac80211_hwsim/hwsim0")
            .option("band", "2g")
            .option("channel", "6")
            .option("htmode", "HT20")
            .option("disabled", "1"),
        Section::named("wifi-device", "radio1")
            .option("type", "mac80211")
            .option("path", "virtual/mac80211_hwsim/hwsim1")
            .option("band", "5g")
            .option("channel", "36")
            .option("country", "00")
            .option("disabled", "1"),
    ]
}

fn settings(mode: GuestMode, enabled: bool) -> Settings {
    Settings {
        version: 3,
        country: "JP".into(),
        guest: GuestSettings {
            name: "Cafe Example".into(),
            mode,
            enabled,
        },
        management: ManagementSettings {
            name: "Enkrnett-3F2A".into(),
        },
    }
}

fn names(sections: &[Section]) -> Vec<&str> {
    sections
        .iter()
        .map(|s| s.name.as_deref().unwrap_or(""))
        .collect()
}

#[test]
fn every_radio_gets_the_country_and_only_the_chosen_one_is_enabled() {
    let plan = Plan {
        country: "JP".into(),
        radio: "radio0".into(),
        phase: Phase::Setup {
            label_suffix: "3F2A".into(),
        },
    };
    let owned = owned_sections(&plan, &radios());
    let r0 = &owned.radios[0];
    assert_eq!(r0.get("country"), Some("JP"));
    assert_eq!(r0.get("disabled"), Some("0"));
    assert_eq!(
        r0.get("path"),
        Some("virtual/mac80211_hwsim/hwsim0"),
        "hardware options kept"
    );
    let r1 = &owned.radios[1];
    assert_eq!(
        r1.get("country"),
        Some("JP"),
        "ENK-CFG-003: the country on every radio"
    );
    assert_eq!(
        r1.get("disabled"),
        Some("1"),
        "an unused radio keeps its state"
    );
    assert_eq!(
        r1.entries
            .iter()
            .filter(|e| matches!(e, crate::uci::Entry::Option(k, _) if k == "country"))
            .count(),
        1
    );
}

#[test]
fn setup_phase_offers_the_setup_network_on_the_management_subnet_without_internet() {
    let plan = Plan {
        country: "JP".into(),
        radio: "radio0".into(),
        phase: Phase::Setup {
            label_suffix: "3F2A".into(),
        },
    };
    let owned = owned_sections(&plan, &radios());
    let setup = owned
        .wireless
        .iter()
        .find(|s| s.name.as_deref() == Some("enk_setup"))
        .unwrap();
    assert_eq!(setup.get("ssid"), Some("Enkrnett Setup 3F2A"));
    assert!(setup.get("ssid").unwrap().ends_with("3F2A"), "ENK-OWN-004");
    assert_eq!(setup.get("encryption"), Some("owe"));
    assert_eq!(setup.get("owe_transition"), Some("1"));
    assert_eq!(setup.get("network"), Some("enk_mgmt"));
    assert_eq!(
        setup.get("ifname"),
        None,
        "the pair is named after the network"
    );
    assert_eq!(names(&owned.network), vec!["enk_br_mgmt", "enk_mgmt"]);
    assert_eq!(owned.network[1].get("ipaddr"), Some(MANAGEMENT_ADDRESS));
    assert!(
        !names(&owned.firewall).contains(&"enk_mgmt_fwd"),
        "no Internet during setup"
    );
    assert!(names(&owned.firewall).contains(&"enk_mgmt_ui"));
    let redirect = owned
        .firewall
        .iter()
        .find(|s| s.name.as_deref() == Some("enk_mgmt_redirect"))
        .unwrap();
    assert_eq!(redirect.get("src_dport"), Some("80"));
    assert_eq!(redirect.get("dest_port"), Some("8080"));
    assert_eq!(redirect.get("target"), Some("DNAT"));
    let bss = expected_bss(&plan.phase);
    assert_eq!(bss.len(), 2);
    assert_eq!(
        (bss[0].ssid.as_str(), bss[0].role),
        ("Enkrnett Setup 3F2A", BssRole::Open)
    );
    assert_eq!(
        (bss[1].ssid.as_str(), bss[1].role),
        ("Enkrnett Setup 3F2AOWE", BssRole::Owe)
    );
    assert!(!bss[0].guest);
}

#[test]
fn pending_phase_has_the_management_network_with_internet() {
    let plan = Plan {
        country: "JP".into(),
        radio: "radio0".into(),
        phase: Phase::Pending {
            management_name: "Enkrnett-3F2A".into(),
            management_key: "abcde-fghjk-mnpqr-stuvw".into(),
        },
    };
    let owned = owned_sections(&plan, &radios());
    let mgmt = owned
        .wireless
        .iter()
        .find(|s| s.name.as_deref() == Some("enk_mgmt"))
        .unwrap();
    assert_eq!(mgmt.get("encryption"), Some("sae-mixed"));
    assert_eq!(mgmt.get("key"), Some("abcde-fghjk-mnpqr-stuvw"));
    assert_eq!(mgmt.get("ifname"), Some("enk-mgmt"));
    assert!(names(&owned.firewall).contains(&"enk_mgmt_fwd"));
    assert!(!names(&owned.wireless).contains(&"enk_setup"));
    assert!(!names(&owned.wireless).contains(&"enk_guest"));
    let bss = expected_bss(&plan.phase);
    assert_eq!(bss.len(), 1);
    assert_eq!(
        (bss[0].ssid.as_str(), bss[0].role),
        ("Enkrnett-3F2A", BssRole::Management)
    );
}

#[test]
fn claimed_phase_adds_the_guest_network_in_both_modes() {
    let open = Plan {
        country: "JP".into(),
        radio: "radio0".into(),
        phase: Phase::Claimed {
            management_name: "Enkrnett-3F2A".into(),
            management_key: "k".into(),
            settings: settings(GuestMode::OpenToAll, true),
        },
    };
    let owned = owned_sections(&open, &radios());
    let guest = owned
        .wireless
        .iter()
        .find(|s| s.name.as_deref() == Some("enk_guest"))
        .unwrap();
    assert_eq!(guest.get("ssid"), Some("Cafe Example"));
    assert_eq!(guest.get("owe_transition"), Some("1"));
    assert_eq!(guest.get("bridge_isolate"), Some("1"));
    assert_eq!(
        names(&owned.network),
        vec!["enk_br_mgmt", "enk_mgmt", "enk_br_guest", "enk_guest"]
    );
    assert_eq!(
        names(&owned.dhcp),
        vec!["enk_dns_mgmt", "enk_mgmt", "enk_dns_guest", "enk_guest"]
    );
    let guest_dns = &owned.dhcp[2];
    assert!(
        File::render_section(guest_dns).contains("list notinterface 'loopback'"),
        "RFC 014 finding 2"
    );
    assert!(
        !File::render_section(&owned.dhcp[0]).contains("notinterface"),
        "one instance keeps loopback"
    );
    assert!(names(&owned.firewall).contains(&"enk_guest_no_site"));
    assert!(names(&owned.firewall).contains(&"enk_guest_fwd"));
    assert!(
        !names(&owned.firewall).contains(&"enk_guest_ui"),
        "no UI from the guest network (ENK-ISO-003)"
    );
    let bss = expected_bss(&open.phase);
    let summary: Vec<(&str, BssRole, bool)> = bss
        .iter()
        .map(|b| (b.ssid.as_str(), b.role, b.guest))
        .collect();
    assert_eq!(
        summary,
        vec![
            ("Enkrnett-3F2A", BssRole::Management, false),
            ("Cafe Example", BssRole::Open, true),
            ("Cafe ExampleOWE", BssRole::Owe, true)
        ]
    );
    assert_eq!(
        expected_interfaces(&open.phase),
        vec!["enk_mgmt", "enk_guest"]
    );

    let encrypted = Plan {
        country: "JP".into(),
        radio: "radio0".into(),
        phase: Phase::Claimed {
            management_name: "m".into(),
            management_key: "k".into(),
            settings: settings(GuestMode::EncryptedOnly, true),
        },
    };
    let owned = owned_sections(&encrypted, &radios());
    let guest = owned
        .wireless
        .iter()
        .find(|s| s.name.as_deref() == Some("enk_guest"))
        .unwrap();
    assert_eq!(guest.get("owe_transition"), None);
    let bss = expected_bss(&encrypted.phase);
    assert_eq!(bss.len(), 2);
    assert_eq!(
        (bss[1].ssid.as_str(), bss[1].role, bss[1].guest),
        ("Cafe Example", BssRole::Owe, true)
    );

    let off = Plan {
        country: "JP".into(),
        radio: "radio0".into(),
        phase: Phase::Claimed {
            management_name: "m".into(),
            management_key: "k".into(),
            settings: settings(GuestMode::OpenToAll, false),
        },
    };
    let owned = owned_sections(&off, &radios());
    assert!(!names(&owned.wireless).contains(&"enk_guest"));
    assert_eq!(names(&owned.network), vec!["enk_br_mgmt", "enk_mgmt"]);
    assert_eq!(expected_interfaces(&off.phase), vec!["enk_mgmt"]);
}

#[test]
fn every_owned_section_carries_the_prefix_and_the_firewall_rules_reject_private_ranges() {
    let plan = Plan {
        country: "JP".into(),
        radio: "radio0".into(),
        phase: Phase::Claimed {
            management_name: "m".into(),
            management_key: "k".into(),
            settings: settings(GuestMode::OpenToAll, true),
        },
    };
    let owned = owned_sections(&plan, &radios());
    for s in owned
        .network
        .iter()
        .chain(&owned.dhcp)
        .chain(&owned.firewall)
    {
        assert!(
            s.name.as_deref().unwrap().starts_with(OWNED_PREFIX),
            "{:?}",
            s.name
        );
    }
    let rule = owned
        .firewall
        .iter()
        .find(|s| s.name.as_deref() == Some("enk_guest_no_site"))
        .unwrap();
    let text = File::render_section(rule);
    for range in PRIVATE_RANGES {
        assert!(text.contains(range), "{range}");
    }
}

#[test]
fn interface_names_fit_the_kernel_limit() {
    for name in [
        "enk_mgmt-open",
        "enk_mgmt-owe",
        "enk_guest-open",
        "enk_guest-owe",
        "enk-mgmt",
        "enk-guest",
    ] {
        assert!(name.len() <= 15, "{name}");
    }
}

#[test]
fn closed_phase_offers_no_wifi() {
    let plan = Plan {
        country: "JP".into(),
        radio: "radio0".into(),
        phase: Phase::Closed,
    };
    let owned = owned_sections(&plan, &radios());
    assert!(owned.wireless.is_empty());
    assert_eq!(owned.radios.len(), 2);
    assert!(expected_bss(&plan.phase).is_empty());
}
