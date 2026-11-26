//! The in-memory platform. It applies settings instantly and healthily, reports effective
//! values derived from what it was given, and records every call for tests.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Condvar, Mutex};

use enkrnett_model::api::{
    Connection, GuestStatus, HardeningItem, HealthDetail, InstalledAs, ManagementStatus, Memory,
    Radio, Storage, UplinkStatus,
};
use enkrnett_model::settings::Settings;
use enkrnett_model::state::{ConnectionNetwork, ConnectionSecurity, OwnershipState};

use super::FetchError;
use super::{Adopted, DeviceInfo, Effective, Platform, PlatformError};

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
    update_index_url: Option<String>,
    /// What `fetch` answers per URL; an unknown URL is unreachable.
    fetches: HashMap<String, Result<Vec<u8>, FetchError>>,
    release_keys: Vec<String>,
    /// While true, `apply_settings` waits (tests observe the change while it is applying).
    hold_apply: bool,
    /// The owned sections differ from what was last written (ENK-CFG-014).
    drift: bool,
    /// The uplink's subnet collides with a network (ENK-TOPO-003).
    uplink_collides: bool,
}

pub struct MockPlatform {
    state: Mutex<State>,
    released: Condvar,
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
                update_index_url: Some("https://updates.example/index.json".into()),
                fetches: HashMap::new(),
                release_keys: Vec::new(),
                hold_apply: false,
                drift: false,
                uplink_collides: false,
            }),
            released: Condvar::new(),
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

    /// Pretends an expert edited an owned section by hand; cleared by the next apply.
    pub fn set_drift(&self, drift: bool) {
        self.state.lock().expect("mock").drift = drift;
    }

    /// Pretends the uplink's subnet collides with a network; cleared by the next apply.
    pub fn set_uplink_collides(&self, collides: bool) {
        self.state.lock().expect("mock").uplink_collides = collides;
    }

    /// Holds every `apply_settings` until `release_apply`: the change stays *applying*.
    pub fn hold_apply(&self) {
        self.state.lock().expect("mock").hold_apply = true;
    }

    pub fn release_apply(&self) {
        self.state.lock().expect("mock").hold_apply = false;
        self.released.notify_all();
    }

    /// Makes the next `apply_settings` fail its health check with `detail`.
    pub fn fail_next_apply(&self, detail: HealthDetail) {
        self.state.lock().expect("mock").fail_apply_with = Some(detail);
    }

    pub fn set_update_index_url(&self, url: Option<&str>) {
        self.state.lock().expect("mock").update_index_url = url.map(str::to_string);
    }

    /// What the next `fetch` of `url` returns.
    pub fn set_fetch(&self, url: &str, result: Result<Vec<u8>, FetchError>) {
        self.state
            .lock()
            .expect("mock")
            .fetches
            .insert(url.to_string(), result);
    }

    /// The signify public keys the "image" carries, as file texts.
    pub fn set_release_keys(&self, keys: Vec<String>) {
        self.state.lock().expect("mock").release_keys = keys;
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
            return Err(PlatformError::Failed("no light".into()));
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
        while s.hold_apply {
            s = self.released.wait(s).expect("mock");
        }
        if let Some(detail) = s.fail_apply_with.take() {
            return Ok(vec![detail]);
        }
        s.setup_up = false;
        s.management = Some((management_name.to_string(), management_key.to_string()));
        s.applied = Some(settings.clone());
        s.drift = false;
        s.uplink_collides = false;
        Ok(Vec::new())
    }

    fn adopt_settings(
        &self,
        management_name: &str,
        management_key: &str,
        settings: &Settings,
    ) -> Result<Adopted, PlatformError> {
        self.record(&format!(
            "adopt_settings(v{}, {})",
            settings.version, settings.guest.name
        ));
        let mut s = self.state.lock().expect("mock");
        if s.uplink_collides {
            // Renumbering: the one case where adopting writes.
            drop(s);
            let health = self.apply_settings(management_name, management_key, settings)?;
            return Ok(Adopted {
                health,
                drift: false,
            });
        }
        s.setup_up = false;
        s.management = Some((management_name.to_string(), management_key.to_string()));
        s.applied = Some(settings.clone());
        let health = s.fail_apply_with.take().into_iter().collect();
        Ok(Adopted {
            health,
            drift: s.drift,
        })
    }

    fn owned_drift(&self) -> Result<bool, PlatformError> {
        Ok(self.state.lock().expect("mock").drift)
    }

    fn uplink_collides(&self) -> bool {
        self.state.lock().expect("mock").uplink_collides
    }

    fn disconnect_management_stations(&self) -> Result<(), PlatformError> {
        self.record("disconnect_management_stations()");
        Ok(())
    }

    fn effective(&self) -> Effective {
        self.record("effective()");
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
            profile: "mock".into(),
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

    fn update_index_url(&self) -> Option<String> {
        self.state.lock().expect("mock").update_index_url.clone()
    }

    fn fetch(&self, url: &str, max_bytes: usize) -> Result<Vec<u8>, FetchError> {
        self.record(&format!("fetch({url})"));
        let s = self.state.lock().expect("mock");
        match s.fetches.get(url) {
            Some(Ok(bytes)) if bytes.len() > max_bytes => Err(FetchError::TooLarge),
            Some(r) => r.clone(),
            None => Err(FetchError::Unreachable("mock: no such url".into())),
        }
    }

    fn release_keys(&self) -> Vec<String> {
        self.state.lock().expect("mock").release_keys.clone()
    }

    fn erase(&self) -> Result<(), PlatformError> {
        self.record("erase()");
        let mut s = self.state.lock().expect("mock");
        s.management = None;
        s.applied = None;
        s.setup_up = false;
        Ok(())
    }

    fn restart_service(&self) -> Result<(), PlatformError> {
        self.record("restart_service()");
        Ok(())
    }

    fn restart(&self) -> Result<(), PlatformError> {
        self.record("restart()");
        Ok(())
    }
}
