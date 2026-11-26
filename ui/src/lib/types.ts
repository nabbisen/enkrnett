// The shapes of chapter 9 §4 as the UI reads them. They mirror `enkrnett-model::api`; a generated
// JSON Schema will replace the hand-written copy (RFC 015 open question 4).

export type OwnershipState = 'UNCLAIMED' | 'SETUP' | 'PENDING' | 'CLAIMED';
export type ConnectionNetwork = 'setup' | 'management';
export type ConnectionSecurity = 'none' | 'owe' | 'wpa2' | 'wpa3';
export type GuestMode = 'open_to_all' | 'encrypted_only';
export type SessionKind = 'claim' | 'owner';
export type ChangeState = 'none' | 'validating' | 'applying' | 'waiting_confirmation' | 'settled' | 'undone';
export type ChangeKind = 'settings' | 'restore' | 'management_key';
export type UndoReason = 'health_check_failed' | 'not_confirmed' | 'restart';
export type HealthDetail =
  | 'guest_down'
  | 'management_down'
  | 'isolation_missing'
  | 'dhcp_missing'
  | 'listener_exposed'
  | 'pmf_missing'
  | 'mode_differs';
export type CheckResult = 'ok' | 'unreachable' | 'bad_signature' | 'bad_index' | 'stale_index';
export type UpdateState = 'idle' | 'downloading' | 'verifying' | 'installing';
export type UpdateFailure =
  | 'bad_signature'
  | 'wrong_device'
  | 'wrong_baseline'
  | 'not_enough_memory'
  | 'download_failed'
  | 'health_check_failed'
  // Not in the accepted API (RFC 015): the root side did not flash the verified image; recorded
  // for the next amendment RFC.
  | 'install_failed';

export interface Connection {
  network: ConnectionNetwork;
  security: ConnectionSecurity | null;
}

export interface Greeting {
  product: string;
  api: number;
  state: OwnershipState;
  languages: string[];
  development: boolean;
  connection: Connection;
}

export interface Session {
  token: string;
  kind: SessionKind;
  state: OwnershipState;
  idle_ends_in: number;
  ends_in: number;
}

export interface Limits {
  guest_name_max_bytes: { open_to_all: number; encrypted_only: number };
  management_name_max_bytes: number;
  password: { min: number; max: number };
}

export interface Claim {
  window_ends_in: number;
  country: { fixed: string | null; choices: string[] };
  guest_mode_default: GuestMode;
  identify_available: boolean;
  limits: Limits;
  connection: Connection;
}

export interface Proposal {
  management: { name: string; key: string };
  address: string;
  guest: { name: string; mode: GuestMode };
  country: string;
  state: OwnershipState;
  final_in: number | null;
}

export interface Settings {
  version: number;
  country: string;
  guest: { name: string; mode: GuestMode; enabled: boolean };
  management: { name: string };
  limits: Limits;
}

export interface SettingsUpdate {
  version: number;
  guest: { name: string; mode: GuestMode; enabled: boolean };
  management: { name: string };
}

export interface Change {
  id: number;
  state: ChangeState;
  kind: ChangeKind | null;
  from_version: number | null;
  to_version: number | null;
  needs_confirmation: boolean | null;
  confirm_in: number | null;
  reason: UndoReason | null;
  detail: HealthDetail | null;
  started_ago: number | null;
}

export interface Version {
  enkrnett: string;
  baseline: string;
  openwrt: string;
  api: number;
}

export interface Update {
  installed: { version: string; baseline: string };
  checks_enabled: boolean;
  last_check: { checked_ago: number; result: CheckResult } | null;
  available: {
    version: string;
    baseline: string;
    changes_baseline: boolean;
    size_kib: number;
    notes_url: string;
    install_seconds: number;
  } | null;
  state: UpdateState;
  last_result: { result: 'done' | 'failed'; reason: UpdateFailure | null } | null;
  can_undo: boolean;
}

export interface Status {
  state: OwnershipState;
  version: Version;
  settings: Settings;
  guest: {
    up: boolean;
    name: string | null;
    mode: GuestMode | null;
    clients_encrypted: number;
    clients_unencrypted: number;
    checked_ago: number;
  };
  management: { up: boolean; name: string | null; address: string; clients: number };
  uplink: { up: boolean; address: string | null };
  change: Change;
  drift: boolean;
  update: Update;
  hardening_met: boolean;
}

export interface Device {
  model: string;
  revision: string;
  tier: number;
  device_class: string;
  device_id: string;
  version: Version;
  uptime_seconds: number;
  memory: { total_kib: number; available_kib: number };
  storage: { overlay_total_kib: number; overlay_free_kib: number };
  hardening: { id: string; met: boolean }[];
  installed_as: 'image' | 'package';
  radios: { band: string; in_use: boolean }[];
}

export interface ManagementKey {
  name: string;
  key: string;
}

export type ErrorCode =
  | 'invalid'
  | 'cannot_update'
  | 'no_session'
  | 'wrong_password'
  | 'wrong_state'
  | 'not_available'
  | 'unknown_resource'
  | 'method_not_allowed'
  | 'setup_in_progress_elsewhere'
  | 'no_proposal'
  | 'version_mismatch'
  | 'change_pending'
  | 'nothing_to_install'
  | 'update_in_progress'
  | 'too_large'
  | 'wrong_content_type'
  | 'try_later'
  | 'internal';

export type FieldCode =
  | 'required'
  | 'unknown_field'
  | 'wrong_type'
  | 'too_short'
  | 'too_long'
  | 'not_allowed'
  | 'forbidden_characters'
  | 'common_password'
  | 'equals_network_name'
  | 'equals_key'
  | 'equals_device_id'
  | 'not_in_choices';

export interface FieldError {
  field: string;
  code: FieldCode;
  limit?: number;
}

export interface ErrorDetail {
  code: ErrorCode;
  fields?: FieldError[];
  message?: string;
  state?: OwnershipState;
  retry_in?: number;
  reason?: UpdateFailure;
  reference?: string;
  settings?: Settings;
  change?: Change;
}
