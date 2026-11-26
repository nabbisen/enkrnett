use std::net::{IpAddr, Ipv4Addr};

use enkrnett_model::api::HealthDetail;
use enkrnett_model::settings::{GuestMode, GuestSettings, ManagementSettings, Settings};
use enkrnett_model::state::{ConnectionNetwork, ConnectionSecurity, OwnershipState};

use super::*;

fn settings() -> Settings {
    Settings {
        version: 1,
        country: "JP".into(),
        guest: GuestSettings {
            name: "Cafe".into(),
            mode: GuestMode::OpenToAll,
            enabled: true,
        },
        management: ManagementSettings {
            name: "Enkrnett-3F2A".into(),
        },
    }
}

#[test]
fn mock_records_calls_and_reports_effective_state() {
    let p = MockPlatform::new();
    p.enter_setup().unwrap();
    assert!(!p.effective().management.up);
    p.enter_pending("Enkrnett-3F2A", "key").unwrap();
    assert!(p.effective().management.up);
    assert!(!p.effective().guest.up);
    assert!(
        p.apply_settings("Enkrnett-3F2A", "key", &settings())
            .unwrap()
            .is_empty()
    );
    assert!(p.effective().guest.up);
    assert_eq!(p.effective().guest.name.as_deref(), Some("Cafe"));
    assert_eq!(
        p.calls(),
        vec![
            "enter_setup()",
            "enter_pending(Enkrnett-3F2A)",
            "apply_settings(v1, Cafe, OpenToAll)"
        ]
    );
}

#[test]
fn mock_can_fail_a_health_check_once() {
    let p = MockPlatform::new();
    p.fail_next_apply(HealthDetail::GuestDown);
    assert_eq!(
        p.apply_settings("m", "k", &settings()).unwrap(),
        vec![HealthDetail::GuestDown]
    );
    assert!(p.apply_settings("m", "k", &settings()).unwrap().is_empty());
}

#[test]
fn connection_depends_on_state() {
    let p = MockPlatform::new();
    let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let c = p.connection(ip, OwnershipState::Setup);
    assert_eq!(c.network, ConnectionNetwork::Setup);
    assert_eq!(c.security, Some(ConnectionSecurity::Owe));
    p.set_management_security(Some(ConnectionSecurity::Wpa2));
    let c = p.connection(ip, OwnershipState::Claimed);
    assert_eq!(c.network, ConnectionNetwork::Management);
    assert_eq!(c.security, Some(ConnectionSecurity::Wpa2));
}

#[test]
fn light_can_be_absent() {
    let p = MockPlatform::new();
    assert!(p.blink().is_ok());
    p.set_light(false);
    assert!(!p.has_light());
    assert!(p.blink().is_err());
}

// ----- the kernel's view: /proc/net sockets and bridge ports (ENK-CFG-023, 024, 025) -----------

const PROC_TCP: &str = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 01CFA8C0:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 3650 1 0 100 0 0 10 5
   1: 0100007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 3652 1 0 100 0 0 10 5
   2: 01CFA8C0:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1900        0 27859 1 0 100 0 0 10 0
   3: 00000000:0016 00000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 2289 1 0 100 0 0 10 5
   4: 01CFA8C0:1F90 0A00CF0A:C3B2 01 00000000:00000000 00:00000000 00000000  1900        0 27999 1 0 20 4 30 10 -1
";

#[test]
fn proc_net_tcp_is_parsed_with_the_kernel_byte_order_and_only_listeners() {
    let s = openwrt::parse_sockets(PROC_TCP, Some("0A"));
    assert_eq!(s.len(), 4, "{s:?}");
    assert_eq!(
        s[2],
        openwrt::BoundSocket {
            address: "192.168.207.1".parse().unwrap(),
            port: 8080,
            uid: 1900
        }
    );
    assert_eq!(s[3].address, Ipv4Addr::UNSPECIFIED);
    assert_eq!(s[3].port, 22);
    assert_eq!(openwrt::parse_sockets(PROC_TCP, None).len(), 5);
}

#[test]
fn ipv6_sockets_count_as_every_address_unless_mapped() {
    let text = "  sl  local_address                         remote_address                        st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 00000000000000000000000000000000:0016 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000     0        0 2290 1 0 100 0 0 10 0
   1: 0000000000000000FFFF000001CFA8C0:1F90 00000000000000000000000000000000:0000 0A 00000000:00000000 00:00000000 00000000  1900        0 2291 1 0 100 0 0 10 0
";
    let s = openwrt::parse_sockets(text, Some("0A"));
    assert_eq!(s[0].address, Ipv4Addr::UNSPECIFIED);
    assert_eq!(s[1].address, "192.168.207.1".parse::<Ipv4Addr>().unwrap());
}

#[test]
fn the_listener_is_exposed_when_the_ui_port_is_bound_anywhere_but_the_management_address() {
    let mgmt = |port| openwrt::BoundSocket {
        address: "192.168.207.1".parse().unwrap(),
        port,
        uid: 1900,
    };
    let any = |port| openwrt::BoundSocket {
        address: Ipv4Addr::UNSPECIFIED,
        port,
        uid: 0,
    };
    let guest = |port| openwrt::BoundSocket {
        address: "10.207.0.1".parse().unwrap(),
        port,
        uid: 0,
    };
    assert!(!openwrt::listener_exposed(&[
        mgmt(8080),
        any(53),
        any(67),
        guest(53)
    ]));
    assert!(openwrt::listener_exposed(&[any(8080)]));
    assert!(openwrt::listener_exposed(&[mgmt(8080), guest(8080)]));
    assert!(
        openwrt::listener_exposed(&[any(80)]),
        "a stray web server on port 80"
    );
    // Other services are the hardening report's business, not the health check's.
    assert!(!openwrt::listener_exposed(&[any(22)]));
}

#[test]
fn bridge_ports_report_the_isolated_flag_of_every_port() {
    let dir = std::env::temp_dir().join(format!("enkrnett-brif-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for (port, flag) in [("phy0-ap1", "1\n"), ("phy0-ap2", "0\n")] {
        let d = dir.join("br-guest/brif").join(port);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("isolated"), flag).unwrap();
    }
    assert_eq!(
        openwrt::bridge_ports_in(&dir, "br-guest"),
        vec![
            ("phy0-ap1".to_string(), true),
            ("phy0-ap2".to_string(), false)
        ]
    );
    assert!(openwrt::bridge_ports_in(&dir, "br-mgmt").is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}
