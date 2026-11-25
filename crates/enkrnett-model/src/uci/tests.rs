//! ENK-CFG-001 (owned sections written completely, never merged), ENK-CFG-015 (foreign
//! sections untouched), the `uci` quoting rules.

use super::*;

const NETWORK: &str = "\nconfig interface 'loopback'\n\toption device 'lo'\n\toption proto 'static'\n\toption ipaddr '127.0.0.1'\n\toption netmask '255.0.0.0'\n\nconfig globals 'globals'\n\toption ula_prefix 'fd12:3456:789a::/48'\n\n# the uplink\nconfig interface 'wan'\n\toption device 'eth1'\n\toption proto 'dhcp'\n\nconfig device 'enk_br_guest'\n\toption name 'br-guest'\n\toption type 'bridge'\n\tlist ports 'wlan0'\n\tlist ports 'wlan1'\n\nconfig interface 'enk_guest'\n\toption device 'br-guest'\n\toption proto 'static'\n\toption ipaddr '10.207.0.1'\n\nconfig rule\n\toption name 'anonymous keeps its place'\n";

#[test]
fn parses_sections_options_and_lists() {
    let f = File::parse(NETWORK).unwrap();
    let names: Vec<Option<&str>> = f.sections().map(|s| s.name.as_deref()).collect();
    assert_eq!(
        names,
        vec![
            Some("loopback"),
            Some("globals"),
            Some("wan"),
            Some("enk_br_guest"),
            Some("enk_guest"),
            None
        ]
    );
    let bridge = f.named_section("enk_br_guest").unwrap();
    assert_eq!(bridge.kind, "device");
    assert_eq!(bridge.get("type"), Some("bridge"));
    assert_eq!(
        bridge.entries.last(),
        Some(&Entry::List(
            "ports".into(),
            vec!["wlan0".into(), "wlan1".into()]
        ))
    );
    assert_eq!(f.sections().last().unwrap().kind, "rule");
}

#[test]
fn quoting_round_trips() {
    let text = "config wifi-iface 'enk_guest'\n\toption ssid 'Cafe '\\''Example'\\'''\n\toption key \"dq\\\"x\"\n";
    let f = File::parse(text).unwrap();
    let s = f.named_section("enk_guest").unwrap();
    assert_eq!(s.get("ssid"), Some("Cafe 'Example'"));
    assert_eq!(s.get("key"), Some("dq\"x"));
    let rendered = File::render_section(s);
    assert!(rendered.contains("option ssid 'Cafe '\\''Example'\\'''"));
    assert!(rendered.contains("option key 'dq\"x'"));
}

#[test]
fn replace_owned_keeps_foreign_lines_byte_for_byte_and_appends_owned() {
    let f = File::parse(NETWORK).unwrap();
    let owned = vec![
        Section::named("device", "enk_br_mgmt")
            .option("name", "br-mgmt")
            .option("type", "bridge"),
        Section::named("interface", "enk_mgmt")
            .option("device", "br-mgmt")
            .option("proto", "static")
            .option("ipaddr", "192.168.207.1")
            .option("netmask", "255.255.255.0"),
    ];
    let out = f.replace_owned("enk_", &owned);
    assert!(out.contains(
        "# the uplink\nconfig interface 'wan'\n\toption device 'eth1'\n\toption proto 'dhcp'\n"
    ));
    assert!(out.contains("config interface 'loopback'"));
    assert!(out.contains("config rule\n\toption name 'anonymous keeps its place'\n"));
    assert!(!out.contains("enk_br_guest"), "old owned sections are gone");
    assert!(!out.contains("10.207.0.1"));
    assert!(out.ends_with("config interface 'enk_mgmt'\n\toption device 'br-mgmt'\n\toption proto 'static'\n\toption ipaddr '192.168.207.1'\n\toption netmask '255.255.255.0'\n"));
    let again = File::parse(&out).unwrap();
    assert!(again.owned_equal("enk_", &owned));
    assert_eq!(
        again.named_section("wan").unwrap().get("proto"),
        Some("dhcp")
    );
    // Writing the same owned sections again changes nothing.
    assert_eq!(again.replace_owned("enk_", &owned), out);
}

#[test]
fn owned_equal_detects_drift() {
    let f = File::parse(NETWORK).unwrap();
    let as_written = vec![
        Section::named("device", "enk_br_guest")
            .option("name", "br-guest")
            .option("type", "bridge")
            .list("ports", &["wlan0", "wlan1"]),
        Section::named("interface", "enk_guest")
            .option("device", "br-guest")
            .option("proto", "static")
            .option("ipaddr", "10.207.0.1"),
    ];
    assert!(f.owned_equal("enk_", &as_written));
    let mut drifted = as_written.clone();
    drifted[1] = Section::named("interface", "enk_guest")
        .option("device", "br-guest")
        .option("proto", "static")
        .option("ipaddr", "10.9.9.1");
    assert!(!f.owned_equal("enk_", &drifted));
}

#[test]
fn an_empty_file_gets_only_the_owned_sections() {
    let f = File::parse("").unwrap();
    let out = f.replace_owned("enk_", &[Section::named("x", "enk_a").option("k", "v")]);
    assert_eq!(out, "\nconfig x 'enk_a'\n\toption k 'v'\n");
}

#[test]
fn errors_name_the_line() {
    assert_eq!(
        File::parse("option a 'b'\n").unwrap_err(),
        ParseError {
            line: 1,
            message: "option outside a section"
        }
    );
    assert_eq!(File::parse("config\n").unwrap_err().line, 1);
    assert_eq!(
        File::parse("config x 'y'\n\tbogus z\n").unwrap_err(),
        ParseError {
            line: 2,
            message: "unknown keyword"
        }
    );
}

#[test]
fn rewrite_replaces_named_sections_in_place() {
    let text = "config wifi-device 'radio0'\n\toption type 'mac80211'\n\toption disabled '1'\n\nconfig wifi-iface 'default_radio0'\n\toption device 'radio0'\n\nconfig wifi-iface 'enk_old'\n\toption device 'radio0'\n";
    let f = File::parse(text).unwrap();
    let radio = Section::named("wifi-device", "radio0")
        .option("type", "mac80211")
        .option("country", "JP")
        .option("disabled", "0");
    let iface = Section::named("wifi-iface", "enk_mgmt").option("device", "radio0");
    let out = f.rewrite("enk_", &[iface], &[radio]);
    let again = File::parse(&out).unwrap();
    let names: Vec<Option<&str>> = again.sections().map(|s| s.name.as_deref()).collect();
    assert_eq!(
        names,
        vec![Some("radio0"), Some("default_radio0"), Some("enk_mgmt")]
    );
    assert_eq!(
        again.named_section("radio0").unwrap().get("country"),
        Some("JP")
    );
    assert_eq!(
        again.named_section("radio0").unwrap().get("disabled"),
        Some("0")
    );
    assert!(
        out.starts_with("config wifi-device 'radio0'\n"),
        "position kept"
    );
}
