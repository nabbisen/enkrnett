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
