//! The application: every operation of chapter 9 §3 against the ownership state machine, the
//! sessions, the store, and the platform. One mutex guards the state; password hashing happens
//! outside it. Actions that end the client's own network (confirm, restart, erase) are deferred
//! by `act_delay` after the response is sent (ENK-API-015).

use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use enkrnett_model::api::{
    Available, ChangeKind, CheckResult, Claim, ClaimCheck, ClaimGuest, CountryChoice, Device,
    Empty, EraseRequest, ErrorCode, ErrorDetail, FieldCode, FieldError, Greeting, Installed,
    InstalledAs, LastCheck, LoginRequest, ManagementKey, ManagementProposal, PasswordChange,
    Proposal, Session, SessionKind, SettingsShape, Status, Update, UpdateCheckRequest, UpdateState,
    Version,
};
use enkrnett_model::password::{PasswordContext, check_new_password};
use enkrnett_model::settings::{
    self, GuestMode, GuestSettings, Limits, ManagementSettings, Settings, SettingsUpdate,
};
use enkrnett_model::state::OwnershipState;
use enkrnett_model::{API_VERSION, PRODUCT};
use serde_json::{Value, json};

use crate::auth::{Hasher, LoginGuard};
use crate::clock::{Clock, seconds_since, seconds_until};
use crate::config::Config;
use crate::ownership::{Event, Ownership};
use crate::platform::Platform;
use crate::session::SessionStore;
use crate::store::{OwnerRecord, Store};
use crate::{log, random};

mod change;
mod readings;
mod update;
pub use change::CONFIRMATION_LIMIT;
use change::ChangeRecord;
use readings::Readings;
pub use readings::{DRIFT_INTERVAL, READINGS_MAX_AGE, STATUS_DRIFT_INTERVAL, UPLINK_INTERVAL};
pub use update::{AUTO_CHECK_FIRST_DELAY, AUTO_CHECK_INTERVAL_SECS, auto_check_due};

/// A successful reply: status and JSON body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub status: u16,
    pub body: Value,
}

impl Reply {
    pub fn ok<T: serde::Serialize>(value: &T) -> Reply {
        Reply {
            status: 200,
            body: serde_json::to_value(value).expect("serialisable shape"),
        }
    }

    pub fn accepted<T: serde::Serialize>(value: &T) -> Reply {
        Reply {
            status: 202,
            body: serde_json::to_value(value).expect("serialisable shape"),
        }
    }
}

/// An error reply: the one error shape (chapter 9 §5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiError(pub Box<ErrorDetail>);

impl ApiError {
    pub fn code(code: ErrorCode, message: &str) -> ApiError {
        ApiError(Box::new(ErrorDetail::new(code).message(message)))
    }

    pub fn invalid(fields: Vec<FieldError>, message: &str) -> ApiError {
        let mut d = ErrorDetail::new(ErrorCode::Invalid).message(message);
        d.fields = fields;
        ApiError(Box::new(d))
    }

    pub fn wrong_state(state: OwnershipState) -> ApiError {
        let mut d = ErrorDetail::new(ErrorCode::WrongState).message("not offered in this state");
        d.state = Some(state);
        ApiError(Box::new(d))
    }
}

type Result<T> = std::result::Result<T, ApiError>;

struct HeldProposal {
    record: OwnerRecord,
    shape: Proposal,
}

struct UpdateRecord {
    state: UpdateState,
    last_check: Option<(Instant, CheckResult)>,
    available: Option<Available>,
    /// A check is running (by request or the daily one); a second one waits for the shape.
    checking: bool,
}

/// What the platform said about itself at start-up and will not change while running.
struct DeviceStatic {
    profile: String,
    installed_as: InstalledAs,
    baseline: String,
}

struct Inner {
    ownership: Ownership,
    sessions: SessionStore,
    guard: LoginGuard,
    store: Store,
    proposal: Option<HeldProposal>,
    change: Option<ChangeRecord>,
    change_counter: u64,
    update: UpdateRecord,
    /// The effective state as last read (ENK-API-027).
    readings: Option<Readings>,
    /// The owned sections differ from what the daemon last wrote (ENK-CFG-014).
    drift: bool,
    drift_checked: Option<Instant>,
    uplink_checked: Option<Instant>,
}

struct Shared {
    inner: Mutex<Inner>,
    platform: Arc<dyn Platform>,
    clock: Arc<dyn Clock>,
    config: Arc<Config>,
    hasher: Hasher,
    device: DeviceStatic,
    started: Instant,
    /// The address the HTTP listener is bound to, once it is (`App::set_bound`).
    bound: Mutex<Option<std::net::SocketAddr>>,
}

#[derive(Clone)]
pub struct App(Arc<Shared>);

impl App {
    pub fn new(
        config: Arc<Config>,
        platform: Arc<dyn Platform>,
        clock: Arc<dyn Clock>,
    ) -> std::io::Result<App> {
        let store = Store::open(&config.state_dir, random::device_id)?;
        let now = clock.now();
        let (ownership, event) = Ownership::start(store.data.owner.is_some(), now);
        // ENK-CFG-009: a marker left by a change that did not finish. The stored document is
        // the one from before that change; applying it below is the undo.
        // A restore completes at start-up (the stored document is applied below); any other
        // change is undone by the restart, which the same apply performs.
        let had_marker = store.has_pending();
        let (change, change_counter) = match store.read_pending() {
            None => (None, 0),
            Some(Ok(marker)) if marker.kind == ChangeKind::Restore => {
                log::event("change_settled", &[("id", &marker.id), ("by", &"restart")]);
                let id = marker.id;
                (Some(ChangeRecord::settled_by_restart(marker, now)), id)
            }
            Some(Ok(marker)) => {
                log::event(
                    "change_undone",
                    &[
                        ("id", &marker.id),
                        ("reason", &"restart"),
                        ("kind", &format!("{:?}", marker.kind)),
                    ],
                );
                let id = marker.id;
                (Some(ChangeRecord::undone_by_restart(marker, now)), id)
            }
            Some(Err(why)) => {
                log::event("change_undone", &[("reason", &"restart"), ("marker", &why)]);
                (None, 0)
            }
        };
        if had_marker {
            store.remove_pending()?;
        }
        let (m, t, p) = config.argon2;
        let info = platform.device_info();
        let device = DeviceStatic {
            profile: info.profile,
            installed_as: info.installed_as,
            baseline: info.baseline,
        };
        let app = App(Arc::new(Shared {
            inner: Mutex::new(Inner {
                ownership,
                sessions: SessionStore::new(),
                guard: LoginGuard::default(),
                store,
                proposal: None,
                change,
                change_counter,
                update: UpdateRecord {
                    state: UpdateState::Idle,
                    last_check: None,
                    available: None,
                    checking: false,
                },
                readings: None,
                drift: false,
                drift_checked: None,
                uplink_checked: None,
            }),
            platform,
            clock,
            config,
            hasher: Hasher::new(m, t, p),
            device,
            started: now,
            bound: Mutex::new(None),
        }));
        app.bring_platform_up(event, had_marker);
        Ok(app)
    }

    /// At start and after a restart: the platform offers what the state says.
    /// Start-up (chapter 6 §4, ENK-CFG-014): with a marker, the stored document is applied,
    /// which undoes or completes the cut-short change; without one, the device is adopted as
    /// the boot left it and a difference is reported as drift, never overwritten.
    fn bring_platform_up(&self, event: Option<Event>, had_marker: bool) {
        let owner = {
            let inner = self.0.inner.lock().expect("app");
            inner.store.data.owner.clone()
        };
        let platform = &self.0.platform;
        let now = self.0.clock.now();
        let result = match (owner, event) {
            (Some(record), _) if had_marker => platform
                .apply_settings(
                    &record.management_name,
                    &record.management_key,
                    &record.settings,
                )
                .map(|details| {
                    if !details.is_empty() {
                        log::event(
                            "start_health_check_failed",
                            &[("detail", &format!("{details:?}"))],
                        );
                    }
                }),
            (Some(record), _) => platform
                .adopt_settings(
                    &record.management_name,
                    &record.management_key,
                    &record.settings,
                )
                .map(|adopted| {
                    if !adopted.health.is_empty() {
                        log::event(
                            "start_health_check_failed",
                            &[("detail", &format!("{:?}", adopted.health))],
                        );
                    }
                    if adopted.drift {
                        log::event("drift_detected", &[("at", &"start")]);
                    }
                    let mut inner = self.0.inner.lock().expect("app");
                    inner.drift = adopted.drift;
                    inner.drift_checked = Some(now);
                }),
            (None, _) => platform.enter_setup(),
        };
        if let Err(e) = result {
            self.platform_failed("start", &e);
        }
        log::event("state", &[("state", &self.state().as_str())]);
    }

    pub fn start_ticking(&self) {
        let app = self.clone();
        std::thread::Builder::new()
            .name("enkrnettd-tick".into())
            .spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_secs(1));
                    app.tick();
                    app.check_drift_if_due(DRIFT_INTERVAL);
                    app.check_uplink_if_due(UPLINK_INTERVAL);
                    app.check_update_if_due();
                }
            })
            .expect("spawn tick");
    }

    /// Advances the ownership timers and acts on what they say (ENK-OWN-002, 003, 008).
    pub fn tick(&self) {
        let events = {
            let mut inner = self.0.inner.lock().expect("app");
            let now = self.0.clock.now();
            let claim_under_way =
                inner.sessions.has_kind(SessionKind::Claim, now) || inner.proposal.is_some();
            let events = inner.ownership.tick(now, claim_under_way);
            for e in &events {
                match e {
                    Event::SetupClosed => {
                        inner.sessions.end_all();
                        inner.proposal = None;
                    }
                    Event::PendingExpired => {
                        inner.sessions.end_all();
                        inner.proposal = None;
                    }
                    Event::SetupOpened => {}
                }
            }
            events
        };
        for e in events {
            let result = match e {
                Event::SetupClosed => self.0.platform.close_setup(),
                Event::PendingExpired | Event::SetupOpened => self.0.platform.enter_setup(),
            };
            if let Err(err) = result {
                log::event(
                    "platform_failed",
                    &[("at", &format!("{e:?}")), ("error", &err)],
                );
            }
            log::event(
                "state",
                &[
                    ("state", &self.state().as_str()),
                    ("because", &format!("{e:?}")),
                ],
            );
        }
        self.tick_change();
    }

    /// Records where the listener was bound; the `Host` check and the renumbering follow it.
    pub fn set_bound(&self, addr: std::net::SocketAddr) {
        *self.0.bound.lock().expect("bound") = Some(addr);
    }

    pub fn bound(&self) -> Option<std::net::SocketAddr> {
        *self.0.bound.lock().expect("bound")
    }

    /// The address to bind: on a platform whose listen address follows the management address,
    /// that address with the configured port; else the configured address.
    pub fn listen_address(&self) -> std::net::SocketAddr {
        let configured = self.0.config.listen;
        if !self.0.platform.listen_follows_management_address() {
            return configured;
        }
        match self.0.platform.management_address().parse() {
            Ok(ip) => std::net::SocketAddr::new(ip, configured.port()),
            Err(_) => configured,
        }
    }

    pub fn state(&self) -> OwnershipState {
        self.0.inner.lock().expect("app").ownership.state()
    }

    /// Advances the mock clock's view of time for tests: nothing here, tests use their clock.
    pub fn now(&self) -> Instant {
        self.0.clock.now()
    }

    fn defer<F: FnOnce(App) + Send + 'static>(&self, what: &'static str, f: F) {
        let app = self.clone();
        let delay = self.0.config.act_delay;
        std::thread::Builder::new()
            .name(format!("enkrnettd-{what}"))
            .spawn(move || {
                std::thread::sleep(delay);
                f(app);
            })
            .expect("spawn deferred action");
    }

    // ----- authorization helpers -------------------------------------------------------------

    fn require_state(inner: &Inner, allowed: &[OwnershipState]) -> Result<()> {
        let state = inner.ownership.state();
        if allowed.contains(&state) {
            Ok(())
        } else {
            Err(ApiError::wrong_state(state))
        }
    }

    fn require_session(
        &self,
        inner: &mut Inner,
        token: Option<&str>,
        kind: SessionKind,
    ) -> Result<String> {
        let token =
            token.ok_or_else(|| ApiError::code(ErrorCode::NoSession, "no session token"))?;
        match inner.sessions.authenticate(token, self.0.clock.now()) {
            Some(k) if k == kind => Ok(token.to_string()),
            _ => Err(ApiError::code(ErrorCode::NoSession, "no such session")),
        }
    }

    fn owner(&self, inner: &mut Inner, token: Option<&str>) -> Result<()> {
        Self::require_state(inner, &[OwnershipState::Claimed])?;
        self.require_session(inner, token, SessionKind::Owner)?;
        Ok(())
    }

    fn claimer(&self, inner: &mut Inner, token: Option<&str>) -> Result<()> {
        Self::require_state(inner, &[OwnershipState::Setup])?;
        self.require_session(inner, token, SessionKind::Claim)?;
        Ok(())
    }

    /// ENK-API-012: while an update runs, every POST but logout is refused.
    pub fn refuse_while_updating(&self) -> Result<()> {
        let inner = self.0.inner.lock().expect("app");
        if inner.update.state != UpdateState::Idle {
            return Err(ApiError::code(
                ErrorCode::UpdateInProgress,
                "an update is in progress",
            ));
        }
        Ok(())
    }

    // ----- shapes ----------------------------------------------------------------------------

    fn version(&self) -> Version {
        let info = self.0.platform.device_info();
        Version {
            enkrnett: self.0.config.version.clone(),
            baseline: info.baseline,
            openwrt: info.openwrt,
            api: API_VERSION,
        }
    }

    fn update_shape(&self, inner: &Inner, now: Instant) -> Update {
        let checks_enabled = self.checks_enabled(inner);
        Update {
            installed: Installed {
                version: self.0.config.version.clone(),
                baseline: self.0.device.baseline.clone(),
            },
            checks_enabled,
            last_check: inner.update.last_check.map(|(at, result)| LastCheck {
                checked_ago: seconds_since(at, now),
                result,
            }),
            // ENK-UPD-008: a device that does not look for updates is offered none.
            available: if checks_enabled {
                inner.update.available.clone()
            } else {
                None
            },
            state: inner.update.state,
            last_result: None,
            can_undo: false,
        }
    }

    fn settings_shape(record: &OwnerRecord) -> SettingsShape {
        SettingsShape {
            settings: record.settings.clone(),
            limits: Limits::current(),
        }
    }

    // ----- resources -------------------------------------------------------------------------

    pub fn greeting(&self, client: IpAddr) -> Reply {
        let state = self.state();
        Reply::ok(&Greeting {
            product: PRODUCT,
            api: API_VERSION,
            state,
            languages: self.0.config.languages.clone(),
            development: self.0.config.development,
            connection: self.0.platform.connection(client, state),
        })
    }

    pub fn claim_start(&self, _client: IpAddr) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        Self::require_state(&inner, &[OwnershipState::Setup])?;
        let now = self.0.clock.now();
        if inner.sessions.has_kind(SessionKind::Claim, now) {
            return Err(ApiError::code(
                ErrorCode::SetupInProgressElsewhere,
                "another client holds the claim session",
            ));
        }
        let created = inner
            .sessions
            .create(SessionKind::Claim, random::session_token(), now);
        log::event("claim_started", &[]);
        Ok(Reply::ok(&Session {
            token: created.token,
            kind: SessionKind::Claim,
            state: OwnershipState::Setup,
            idle_ends_in: created.idle_ends_in,
            ends_in: created.ends_in,
        }))
    }

    pub fn claim_get(&self, token: Option<&str>, client: IpAddr) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        self.claimer(&mut inner, token)?;
        let now = self.0.clock.now();
        let fixed = self.0.platform.fixed_country();
        Ok(Reply::ok(&Claim {
            window_ends_in: inner.ownership.window_ends_in(now),
            country: CountryChoice {
                choices: if fixed.is_some() {
                    Vec::new()
                } else {
                    self.0.platform.country_choices()
                },
                fixed,
            },
            guest_mode_default: GuestMode::OpenToAll,
            identify_available: self.0.platform.has_light(),
            limits: Limits::current(),
            connection: self.0.platform.connection(client, OwnershipState::Setup),
        }))
    }

    pub fn claim_identify(&self, token: Option<&str>) -> Result<Reply> {
        {
            let mut inner = self.0.inner.lock().expect("app");
            self.claimer(&mut inner, token)?;
        }
        if !self.0.platform.has_light() {
            return Err(ApiError::code(
                ErrorCode::NotAvailable,
                "this device has no light Enkrnett can control",
            ));
        }
        self.0
            .platform
            .blink()
            .map_err(|e| ApiError::code(ErrorCode::Internal, &e.to_string()))?;
        Ok(Reply::ok(&Empty {}))
    }

    pub fn claim_check(&self, token: Option<&str>, body: ClaimCheck) -> Result<Reply> {
        let (device_id, management_name) = {
            let mut inner = self.0.inner.lock().expect("app");
            self.claimer(&mut inner, token)?;
            (
                inner.store.data.device_id.clone(),
                format!("{}-{}", PRODUCT, self.0.platform.label_suffix()),
            )
        };
        // Validation (ENK-CFG-005, ENK-OWN-009, 010): every failing field at once.
        let mut errors = Vec::new();
        let country = match (self.0.platform.fixed_country(), &body.country) {
            (Some(fixed), None) => fixed,
            (Some(fixed), Some(given)) if *given == fixed => fixed,
            (Some(_), Some(_)) => {
                errors.push(FieldError::new("country", FieldCode::NotAllowed));
                String::new()
            }
            (None, Some(given)) if self.0.platform.country_choices().contains(given) => {
                given.clone()
            }
            (None, Some(_)) => {
                errors.push(FieldError::new("country", FieldCode::NotInChoices));
                String::new()
            }
            (None, None) => {
                errors.push(FieldError::new("country", FieldCode::Required));
                String::new()
            }
        };
        let guest = GuestSettings {
            name: body.guest.name.clone(),
            mode: body.guest.mode,
            enabled: true,
        };
        errors.extend(settings::validate_guest(&guest));
        let key = random::management_key();
        let names = [body.guest.name.as_str(), management_name.as_str()];
        let context = PasswordContext {
            network_names: &names,
            management_key: Some(&key),
            device_id: Some(&device_id),
        };
        errors.extend(check_new_password(
            "owner_password",
            &body.owner_password,
            &context,
        ));
        if !errors.is_empty() {
            return Err(ApiError::invalid(errors, "setup values refused"));
        }
        // Hashing outside the lock (it takes a noticeable fraction of a second).
        let password_hash = self.0.hasher.hash(&body.owner_password);
        let record = OwnerRecord {
            password_hash,
            management_name: management_name.clone(),
            management_key: key.clone(),
            settings: Settings {
                version: 1,
                country: country.clone(),
                guest,
                management: ManagementSettings {
                    name: management_name.clone(),
                },
            },
        };
        let shape = Proposal {
            management: ManagementProposal {
                name: management_name,
                key,
            },
            address: self.0.platform.management_address(),
            guest: ClaimGuest {
                name: body.guest.name,
                mode: body.guest.mode,
            },
            country,
            state: OwnershipState::Setup,
            final_in: None,
        };
        let mut inner = self.0.inner.lock().expect("app");
        self.claimer(&mut inner, token)?;
        inner.proposal = Some(HeldProposal {
            record,
            shape: shape.clone(),
        });
        Ok(Reply::ok(&shape))
    }

    pub fn claim_confirm(&self, token: Option<&str>) -> Result<Reply> {
        let (record, mut shape) = {
            let mut inner = self.0.inner.lock().expect("app");
            self.claimer(&mut inner, token)?;
            let held = inner
                .proposal
                .take()
                .ok_or_else(|| ApiError::code(ErrorCode::NoProposal, "check the settings first"))?;
            let now = self.0.clock.now();
            inner
                .ownership
                .confirm(held.record.clone(), now)
                .map_err(ApiError::wrong_state)?;
            inner.sessions.end_kind(SessionKind::Claim);
            (held.record, held.shape)
        };
        shape.state = OwnershipState::Pending;
        shape.final_in = Some(crate::ownership::PENDING_LIMIT.as_secs());
        log::event("claim_confirmed", &[]);
        self.defer("confirm", move |app| {
            if let Err(e) = app
                .0
                .platform
                .enter_pending(&record.management_name, &record.management_key)
            {
                log::event(
                    "platform_failed",
                    &[("at", &"enter_pending"), ("error", &e)],
                );
            }
        });
        Ok(Reply::ok(&shape))
    }

    pub fn login(&self, body: LoginRequest, _client: IpAddr) -> Result<Reply> {
        let hash = {
            let inner = self.0.inner.lock().expect("app");
            Self::require_state(&inner, &[OwnershipState::Pending, OwnershipState::Claimed])?;
            let now = self.0.clock.now();
            if let Err(retry_in) = inner.guard.check(now) {
                let mut d = ErrorDetail::new(ErrorCode::TryLater).message("too many failed logins");
                d.retry_in = Some(retry_in);
                return Err(ApiError(Box::new(d)));
            }
            match (inner.ownership.pending(), &inner.store.data.owner) {
                (Some(p), _) => p.record.password_hash.clone(),
                (None, Some(o)) => o.password_hash.clone(),
                (None, None) => return Err(ApiError::wrong_state(inner.ownership.state())),
            }
        };
        let ok = self.0.hasher.verify(&body.password, &hash);
        let mut inner = self.0.inner.lock().expect("app");
        let now = self.0.clock.now();
        if !ok {
            inner.guard.failure(now);
            log::event(
                "login_refused",
                &[("consecutive_failures", &inner.guard.consecutive_failures())],
            );
            return Err(ApiError::code(ErrorCode::WrongPassword, "wrong password"));
        }
        inner.guard.success();
        let mut became_claimed = None;
        if inner.ownership.state() == OwnershipState::Pending {
            if let Some(record) = inner.ownership.make_final() {
                inner.store.data.owner = Some(record.clone());
                inner
                    .store
                    .save()
                    .map_err(|e| ApiError::code(ErrorCode::Internal, &e.to_string()))?;
                became_claimed = Some(record);
            }
        }
        let state = inner.ownership.state();
        let created = inner
            .sessions
            .create(SessionKind::Owner, random::session_token(), now);
        drop(inner);
        if let Some(record) = became_claimed {
            log::event("claim_final", &[]);
            match self.0.platform.apply_settings(
                &record.management_name,
                &record.management_key,
                &record.settings,
            ) {
                Ok(details) if details.is_empty() => {}
                Ok(details) => log::event(
                    "health_check_failed",
                    &[("at", &"claim_final"), ("detail", &format!("{details:?}"))],
                ),
                Err(e) => log::event("platform_failed", &[("at", &"claim_final"), ("error", &e)]),
            }
        }
        log::event("session_created", &[("kind", &"owner")]);
        Ok(Reply::ok(&Session {
            token: created.token,
            kind: SessionKind::Owner,
            state,
            idle_ends_in: created.idle_ends_in,
            ends_in: created.ends_in,
        }))
    }

    pub fn logout(&self, token: Option<&str>) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        let token = self.require_session(&mut inner, token, SessionKind::Owner)?;
        inner.sessions.end(&token);
        log::event("session_ended", &[("kind", &"owner"), ("cause", &"logout")]);
        Ok(Reply::ok(&Empty {}))
    }

    pub fn status(&self, token: Option<&str>) -> Result<Reply> {
        {
            let mut inner = self.0.inner.lock().expect("app");
            self.owner(&mut inner, token)?;
        }
        // While the status is watched, drift is checked at most every 30 s (ENK-CFG-014).
        self.check_drift_if_due(STATUS_DRIFT_INTERVAL);
        let mut inner = self.0.inner.lock().expect("app");
        self.owner(&mut inner, token)?;
        let now = self.0.clock.now();
        let record = inner
            .store
            .data
            .owner
            .clone()
            .expect("claimed has an owner");
        let (mut effective, hardening_met) = {
            let r = self.readings(&mut inner, now);
            (r.effective.clone(), r.hardening_met)
        };
        effective.guest.checked_ago = Self::readings_age(&inner, now);
        Ok(Reply::ok(&Status {
            state: inner.ownership.state(),
            version: self.version(),
            settings: Self::settings_shape(&record),
            guest: effective.guest,
            management: effective.management,
            uplink: effective.uplink,
            change: Self::change_shape(&inner, now),
            drift: inner.drift,
            update: self.update_shape(&inner, now),
            hardening_met,
        }))
    }

    /// Save (chapter 9 §3): validates, then starts the transaction of `app/change.rs`.
    pub fn settings_post(&self, token: Option<&str>, body: SettingsUpdate) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        self.owner(&mut inner, token)?;
        let record = inner
            .store
            .data
            .owner
            .clone()
            .expect("claimed has an owner");
        if body.version != record.settings.version {
            let mut d = ErrorDetail::new(ErrorCode::VersionMismatch)
                .message("the settings changed since they were read");
            d.settings = Some(Self::settings_shape(&record));
            return Err(ApiError(Box::new(d)));
        }
        let next = settings::apply_update(&record.settings, &body)
            .map_err(|fields| ApiError::invalid(fields, "settings refused"))?;
        let change = self.start_change(&mut inner, ChangeKind::Settings, next, None)?;
        Ok(Reply::accepted(&change))
    }

    pub fn settings_restore(&self, token: Option<&str>) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        self.owner(&mut inner, token)?;
        let record = inner
            .store
            .data
            .owner
            .clone()
            .expect("claimed has an owner");
        let change = self.start_change(&mut inner, ChangeKind::Restore, record.settings, None)?;
        Ok(Reply::accepted(&change))
    }

    pub fn change_get(&self, token: Option<&str>) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        self.owner(&mut inner, token)?;
        let now = self.0.clock.now();
        Ok(Reply::ok(&Self::change_shape(&inner, now)))
    }

    pub fn management_key_get(&self, token: Option<&str>) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        self.owner(&mut inner, token)?;
        let record = inner
            .store
            .data
            .owner
            .as_ref()
            .expect("claimed has an owner");
        Ok(Reply::ok(&ManagementKey {
            name: record.management_name.clone(),
            key: record.management_key.clone(),
        }))
    }

    /// ENK-API-023: the change shape plus `management_key`, the only compound response.
    pub fn management_key_post(&self, token: Option<&str>) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        self.owner(&mut inner, token)?;
        let record = inner
            .store
            .data
            .owner
            .clone()
            .expect("claimed has an owner");
        let new_key = random::management_key();
        let change = self.start_change(
            &mut inner,
            ChangeKind::ManagementKey,
            record.settings,
            Some(new_key.clone()),
        )?;
        let mut body = serde_json::to_value(&change).expect("shape");
        let key = ManagementKey {
            name: record.management_name,
            key: new_key,
        };
        body["management_key"] = serde_json::to_value(&key).expect("shape");
        Ok(Reply { status: 202, body })
    }

    pub fn password_post(&self, token: Option<&str>, body: PasswordChange) -> Result<Reply> {
        let (hash, context_names, key, device_id) = {
            let mut inner = self.0.inner.lock().expect("app");
            self.owner(&mut inner, token)?;
            let record = inner
                .store
                .data
                .owner
                .as_ref()
                .expect("claimed has an owner");
            (
                record.password_hash.clone(),
                [
                    record.settings.guest.name.clone(),
                    record.management_name.clone(),
                ],
                record.management_key.clone(),
                inner.store.data.device_id.clone(),
            )
        };
        // ENK-API-020: the new value first (400), then the current one (401).
        let names = [context_names[0].as_str(), context_names[1].as_str()];
        let errors = check_new_password(
            "new",
            &body.new,
            &PasswordContext {
                network_names: &names,
                management_key: Some(&key),
                device_id: Some(&device_id),
            },
        );
        if !errors.is_empty() {
            return Err(ApiError::invalid(errors, "new password refused"));
        }
        if !self.0.hasher.verify(&body.current, &hash) {
            return Err(ApiError::code(ErrorCode::WrongPassword, "wrong password"));
        }
        let new_hash = self.0.hasher.hash(&body.new);
        let mut inner = self.0.inner.lock().expect("app");
        self.owner(&mut inner, token)?;
        if let Some(record) = inner.store.data.owner.as_mut() {
            record.password_hash = new_hash;
        }
        inner
            .store
            .save()
            .map_err(|e| ApiError::code(ErrorCode::Internal, &e.to_string()))?;
        inner.sessions.end_all();
        log::event("password_changed", &[]);
        log::event(
            "session_ended",
            &[("kind", &"owner"), ("cause", &"password_changed")],
        );
        Ok(Reply::ok(&Empty {}))
    }

    pub fn device_get(&self, token: Option<&str>) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        self.owner(&mut inner, token)?;
        let info = self.0.platform.device_info();
        Ok(Reply::ok(&Device {
            model: info.model,
            revision: info.revision,
            tier: info.tier,
            device_class: info.device_class,
            device_id: inner.store.data.device_id.clone(),
            version: self.version(),
            uptime_seconds: seconds_since(self.0.started, self.0.clock.now()),
            memory: info.memory,
            storage: info.storage,
            hardening: info.hardening,
            installed_as: info.installed_as,
            radios: info.radios,
        }))
    }

    pub fn update_get(&self, token: Option<&str>) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        self.owner(&mut inner, token)?;
        let now = self.0.clock.now();
        Ok(Reply::ok(&self.update_shape(&inner, now)))
    }

    /// `update/check`: fetches the index now (ENK-UPD-001). With `enable_checks`, first sets
    /// the owner's switch (ENK-UPD-008). While checks are off nothing is fetched and the shape
    /// says so; while another check runs, the shape as it stands is answered.
    pub fn update_check(&self, token: Option<&str>, body: UpdateCheckRequest) -> Result<Reply> {
        {
            let mut inner = self.0.inner.lock().expect("app");
            self.owner(&mut inner, token)?;
            if let Some(on) = body.enable_checks
                && inner.store.data.update_checks != Some(on)
            {
                inner.store.data.update_checks = Some(on);
                self.save_store(&mut inner, "update_checks");
                log::event("update_checks_set", &[("enabled", &on)]);
            }
            let now = self.0.clock.now();
            if !self.checks_enabled(&inner) || inner.update.checking {
                return Ok(Reply::ok(&self.update_shape(&inner, now)));
            }
            inner.update.checking = true;
        }
        let checked = self.run_update_check();
        let mut inner = self.0.inner.lock().expect("app");
        self.record_check(&mut inner, checked, "request");
        let now = self.0.clock.now();
        Ok(Reply::ok(&self.update_shape(&inner, now)))
    }

    pub fn update_install(&self, token: Option<&str>) -> Result<Reply> {
        let mut inner = self.0.inner.lock().expect("app");
        self.owner(&mut inner, token)?;
        if inner.change.as_ref().is_some_and(ChangeRecord::in_progress) {
            return Err(ApiError::code(
                ErrorCode::ChangePending,
                "a change is in progress",
            ));
        }
        if inner.update.available.is_none() {
            return Err(ApiError::code(
                ErrorCode::NothingToInstall,
                "no update is available",
            ));
        }
        // Downloading, verifying, and installing (chapter 11) arrive with the OpenWrt platform.
        Err(ApiError::code(
            ErrorCode::NothingToInstall,
            "installing is not part of this stage",
        ))
    }

    pub fn restart(&self, token: Option<&str>) -> Result<Reply> {
        {
            let mut inner = self.0.inner.lock().expect("app");
            self.owner(&mut inner, token)?;
        }
        log::event("restart_requested", &[]);
        self.defer("restart", |app| app.do_restart());
        Ok(Reply::accepted(&Empty {}))
    }

    pub fn erase(&self, token: Option<&str>, body: EraseRequest) -> Result<Reply> {
        let hash = {
            let mut inner = self.0.inner.lock().expect("app");
            self.owner(&mut inner, token)?;
            inner
                .store
                .data
                .owner
                .as_ref()
                .expect("claimed has an owner")
                .password_hash
                .clone()
        };
        if !self.0.hasher.verify(&body.password, &hash) {
            return Err(ApiError::code(ErrorCode::WrongPassword, "wrong password"));
        }
        {
            let mut inner = self.0.inner.lock().expect("app");
            self.owner(&mut inner, token)?;
        }
        log::event("erase_requested", &[]);
        self.defer("erase", |app| app.do_erase());
        Ok(Reply::accepted(&Empty {}))
    }

    // ----- deferred actions ------------------------------------------------------------------

    /// What a reboot does to the daemon's state: sessions, delays, and the pending claim are
    /// gone; the state is recomputed from the store (ENK-AUTH-003, ENK-OWN-008, ENK-OWN-012).
    pub fn do_restart(&self) {
        let event = {
            let mut inner = self.0.inner.lock().expect("app");
            inner.sessions.end_all();
            inner.guard = LoginGuard::default();
            inner.proposal = None;
            inner.change = None;
            let now = self.0.clock.now();
            let (ownership, event) = Ownership::start(inner.store.data.owner.is_some(), now);
            inner.ownership = ownership;
            event
        };
        if let Err(e) = self.0.platform.restart() {
            log::event("platform_failed", &[("at", &"restart"), ("error", &e)]);
        }
        log::event("restarted", &[]);
        self.bring_platform_up(event, false);
    }

    /// Erase (ENK-OWN-018): the store, then the platform, then a restart.
    pub fn do_erase(&self) {
        {
            let mut inner = self.0.inner.lock().expect("app");
            if let Err(e) = inner.store.erase(random::device_id) {
                log::event("erase_failed", &[("error", &e)]);
            }
            inner.ownership.erase();
            inner.sessions.end_all();
        }
        if let Err(e) = self.0.platform.erase() {
            log::event("platform_failed", &[("at", &"erase"), ("error", &e)]);
        }
        log::event("erased", &[]);
        self.do_restart();
    }

    /// For tests: the seconds left in the setup window.
    pub fn window_ends_in(&self) -> u64 {
        let inner = self.0.inner.lock().expect("app");
        inner.ownership.window_ends_in(self.0.clock.now())
    }

    /// For tests: the stored device identifier.
    pub fn device_id(&self) -> String {
        self.0
            .inner
            .lock()
            .expect("app")
            .store
            .data
            .device_id
            .clone()
    }

    /// For tests: seconds until the window would close as the JSON would show it.
    pub fn debug_json(&self) -> Value {
        let inner = self.0.inner.lock().expect("app");
        let now = self.0.clock.now();
        json!({
            "state": inner.ownership.state(),
            "window_ends_in": inner.ownership.window_ends_in(now),
            "final_in": inner.ownership.final_in(now),
            "sessions": inner.sessions.len(),
            "has_owner": inner.store.data.owner.is_some(),
            "next_window_check": seconds_until(now, now),
        })
    }
}
