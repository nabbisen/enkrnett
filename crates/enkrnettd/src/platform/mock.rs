//! The in-memory platform. It applies settings instantly and healthily, reports effective
//! values derived from what it was given, and records every call for tests.

use std::net::IpAddr;
use std::sync::Mutex;

use enkrnett_model::api::{
    CheckResult, Connection, GuestStatus, HardeningItem, HealthDetail, InstalledAs,
    ManagementStatus, Memory, Radio, Storage, UplinkStatus,
};
use enkrnett_model::settings::Settings;
use enkrnett_model::state::{ConnectionNetwork, ConnectionSecurity, OwnershipState};

use super::{DeviceInfo, Effective, Platform, PlatformError};

#[derive(Clone, Debug)]
struct State {
    calls: Vec<String>,
    setup_up: bool,
    management: Option<(String, String)>,
    applied: Option<Settings>,
    light: bool,
    fixed_country: Option<String>,
    setup_security: Option<ConnectionSecurity>,
    management_security: Option<ConnectionSecurity>,
    hardening_met: bool,
    installed_as: InstalledAs,
    fail_apply_with: Option<HealthDetail>,
    index_result: CheckResult,
}

pub struct MockPlatform {
    state: Mutex<State>,
}

impl Default for MockPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPlatform {
    pub fn new() -> MockPlatform {
        MockPlatform {
            state: Mutex::new(State {
                calls: Vec::new(),
                setup_up: false,
                management: None,
                applied: None,
                light: true,
                fixed_country: None,
                setup_security: Some(ConnectionSecurity::Owe),
                management_security: Some(ConnectionSecurity::Wpa3),
                hardening_met: true,
                installed_as: InstalledAs::Image,
                fail_apply_with: None,
                index_result: CheckResult::Unreachable,
            }),
        }
    }

    fn record(&self, call: &str) {
        self.state
            .lock()
            .expect("mock")
            .calls
            .push(call.to_string());
    }

    /// Every call so far, in order, as `name(args)`.
    pub fn calls(&self) -> Vec<String> {
        self.state.lock().expect("mock").calls.clone()
    }

    pub fn set_light(&self, present: bool) {
        self.state.lock().expect("mock").light = present;
    }

    pub fn set_fixed_country(&self, country: Option<&str>) {
        self.state.lock().expect("mock").fixed_country = country.map(str::to_string);
    }

    pub fn set_setup_security(&self, security: Option<ConnectionSecurity>) {
        self.state.lock().expect("mock").setup_security = security;
    }

    pub fn set_management_security(&self, security: Option<ConnectionSecurity>) {
        self.state.lock().expect("mock").management_security = security;
    }

    pub fn set_hardening_met(&self, met: bool) {
        self.state.lock().expect("mock").hardening_met = met;
    }

    pub fn set_installed_as(&self, how: InstalledAs) {
        self.state.lock().expect("mock").installed_as = how;
    }

    /// Makes the next `apply_settings` fail its health check with `detail`.
    pub fn fail_next_apply(&self, detail: HealthDetail) {
        self.state.lock().expect("mock").fail_apply_with = Some(detail);
    }

    pub fn set_index_result(&self, result: CheckResult) {
        self.state.lock().expect("mock").index_result = result;
    }

    /// The settings last applied, if any.
    pub fn applied(&self) -> Option<Settings> {
        self.state.lock().expect("mock").applied.clone()
    }

    /// The management network as last started.
    pub fn management(&self) -> Option<(String, String)> {
        self.state.lock().expect("mock").management.clone()
    }
}

impl Platform for MockPlatform {
    fn label_suffix(&self) -> String {
        "3F2A".to_string()
    }

    fn has_light(&self) -> bool {
        self.state.lock().expect("mock").light
    }

    fn blink(&self) -> Result<(), PlatformError> {
        if !self.has_light() {
            return Err(PlatformError("no light".into()));
        }
        self.record("blink()");
        Ok(())
    }

    fn fixed_country(&self) -> Option<String> {
        self.state.lock().expect("mock").fixed_country.clone()
    }

    fn country_choices(&self) -> Vec<String> {
        ["JP", "US", "DE", "GB", "FR", "AU"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn management_address(&self) -> String {
        "192.168.207.1".to_string()
    }

    fn connection(&self, _client: IpAddr, state: OwnershipState) -> Connection {
        let s = self.state.lock().expect("mock");
        match state {
            OwnershipState::Setup | OwnershipState::Unclaimed => Connection {
                network: ConnectionNetwork::Setup,
                security: s.setup_security,
            },
            OwnershipState::Pending | OwnershipState::Claimed => Connection {
                network: ConnectionNetwork::Management,
                security: s.management_security,
            },
        }
    }

    fn enter_setup(&self) -> Result<(), PlatformError> {
        self.record("enter_setup()");
        let mut s = self.state.lock().expect("mock");
        s.setup_up = true;
        s.management = None;
        Ok(())
    }

    fn close_setup(&self) -> Result<(), PlatformError> {
        self.record("close_setup()");
        let mut s = self.state.lock().expect("mock");
        s.setup_up = false;
        s.management = None;
        Ok(())
    }

    fn enter_pending(
        &self,
        management_name: &str,
        management_key: &str,
    ) -> Result<(), PlatformError> {
        self.record(&format!("enter_pending({management_name})"));
        let mut s = self.state.lock().expect("mock");
        s.setup_up = false;
        s.management = Some((management_name.to_string(), management_key.to_string()));
        Ok(())
    }

    fn apply_settings(
        &self,
        management_name: &str,
        management_key: &str,
        settings: &Settings,
    ) -> Result<Vec<HealthDetail>, PlatformError> {
        self.record(&format!(
            "apply_settings(v{}, {}, {:?})",
            settings.version, settings.guest.name, settings.guest.mode
        ));
        let mut s = self.state.lock().expect("mock");
        if let Some(detail) = s.fail_apply_with.take() {
            return Ok(vec![detail]);
        }
        s.setup_up = false;
        s.management = Some((management_name.to_string(), management_key.to_string()));
        s.applied = Some(settings.clone());
        Ok(Vec::new())
    }

    fn effective(&self) -> Effective {
        let s = self.state.lock().expect("mock");
        let guest = match &s.applied {
            Some(settings) if settings.guest.enabled => GuestStatus {
                up: true,
                name: Some(settings.guest.name.clone()),
                mode: Some(settings.guest.mode),
                clients_encrypted: 0,
                clients_unencrypted: 0,
                checked_ago: 0,
            },
            _ => GuestStatus {
                up: false,
                name: None,
                mode: None,
                clients_encrypted: 0,
                clients_unencrypted: 0,
                checked_ago: 0,
            },
        };
        let management = match &s.management {
            Some((name, _)) => ManagementStatus {
                up: true,
                name: Some(name.clone()),
                address: self.management_address(),
                clients: 1,
            },
            None => ManagementStatus {
                up: false,
                name: None,
                address: self.management_address(),
                clients: 0,
            },
        };
        Effective {
            guest,
            management,
            uplink: UplinkStatus {
                up: true,
                address: Some("192.168.1.57".into()),
            },
        }
    }

    fn device_info(&self) -> DeviceInfo {
        let s = self.state.lock().expect("mock");
        let ids = [
            "ENK-HARD-001",
            "ENK-HARD-002",
            "ENK-HARD-003",
            "ENK-HARD-004",
            "ENK-HARD-005",
            "ENK-HARD-008",
        ];
        DeviceInfo {
            model: "Mock device".into(),
            revision: "1".into(),
            tier: 2,
            device_class: "standard".into(),
            memory: Memory {
                total_kib: 131_072,
                available_kib: 98_304,
            },
            storage: Storage {
                overlay_total_kib: 12_288,
                overlay_free_kib: 10_240,
            },
            hardening: ids
                .iter()
                .map(|id| HardeningItem {
                    id: id.to_string(),
                    met: s.hardening_met,
                })
                .collect(),
            installed_as: s.installed_as,
            radios: vec![
                Radio {
                    band: "2g".into(),
                    in_use: true,
                },
                Radio {
                    band: "5g".into(),
                    in_use: false,
                },
            ],
            baseline: "25.12".into(),
            openwrt: "25.12.5".into(),
        }
    }

    fn check_update_index(&self) -> CheckResult {
        self.record("check_update_index()");
        self.state.lock().expect("mock").index_result
    }

    fn erase(&self) -> Result<(), PlatformError> {
        self.record("erase()");
        let mut s = self.state.lock().expect("mock");
        s.management = None;
        s.applied = None;
        s.setup_up = false;
        Ok(())
    }

    fn restart(&self) -> Result<(), PlatformError> {
        self.record("restart()");
        Ok(())
    }
}
