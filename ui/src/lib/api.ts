// The management API client (chapter 9). One function per resource; the session token travels
// in `Enkrnett-Session` (ENK-API-008); every POST carries a JSON body (ENK-API-002). A failed
// connection is a `NotConnected` error, an API error an `ApiError` with the one error shape.

import type {
  Change,
  Claim,
  Device,
  ErrorDetail,
  Greeting,
  ManagementKey,
  Proposal,
  Session,
  SettingsUpdate,
  Status,
  Update,
} from './types';

export const PREFIX = '/api/1/';

export class ApiError extends Error {
  readonly status: number;
  readonly detail: ErrorDetail;
  constructor(status: number, detail: ErrorDetail) {
    super(detail.message ?? detail.code);
    this.status = status;
    this.detail = detail;
  }
}

export class NotConnected extends Error {
  constructor() {
    super('not connected');
  }
}

// The token lives in memory and in the tab's session storage only (ENK-UI-023); storage may be
// absent or throw, and the UI works without it.
let token: string | null = null;

export function setToken(value: string | null): void {
  token = value;
  try {
    if (value === null) sessionStorage.removeItem('enkrnett-session');
    else sessionStorage.setItem('enkrnett-session', value);
  } catch {
    // storage unavailable: the token lives in memory for this page
  }
}

export function getToken(): string | null {
  if (token !== null) return token;
  try {
    token = sessionStorage.getItem('enkrnett-session');
  } catch {
    token = null;
  }
  return token;
}

async function call<T>(method: 'GET' | 'POST', path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = {};
  const t = getToken();
  if (t) headers['Enkrnett-Session'] = t;
  if (method === 'POST') headers['Content-Type'] = 'application/json';
  let response: Response;
  try {
    response = await fetch(PREFIX + path, {
      method,
      headers,
      body: method === 'POST' ? JSON.stringify(body ?? {}) : null,
      cache: 'no-store',
      credentials: 'omit',
    });
  } catch {
    throw new NotConnected();
  }
  if (response.status === 421) throw new NotConnected();
  let json: unknown = null;
  try {
    json = await response.json();
  } catch {
    json = null;
  }
  if (!response.ok) {
    const detail = (json as { error?: ErrorDetail } | null)?.error ?? { code: 'internal' as const };
    throw new ApiError(response.status, detail);
  }
  return json as T;
}

export const api = {
  greeting: () => call<Greeting>('GET', ''),
  claimStart: () => call<Session>('POST', 'claim/start'),
  claim: () => call<Claim>('GET', 'claim'),
  claimIdentify: () => call<Record<string, never>>('POST', 'claim/identify'),
  claimCheck: (body: { country?: string; guest: { name: string; mode: string }; owner_password: string }) =>
    call<Proposal>('POST', 'claim/check', body),
  claimConfirm: () => call<Proposal>('POST', 'claim/confirm'),
  login: (password: string) => call<Session>('POST', 'login', { password }),
  logout: () => call<Record<string, never>>('POST', 'logout'),
  status: () => call<Status>('GET', 'status'),
  saveSettings: (body: SettingsUpdate) => call<Change>('POST', 'settings', body),
  restoreSettings: () => call<Change>('POST', 'settings/restore'),
  change: () => call<Change>('GET', 'change'),
  managementKey: () => call<ManagementKey>('GET', 'management-key'),
  replaceManagementKey: () => call<Change & { management_key: ManagementKey }>('POST', 'management-key'),
  changePassword: (current: string, next: string) => call<Record<string, never>>('POST', 'password', { current, new: next }),
  device: () => call<Device>('GET', 'device'),
  update: () => call<Update>('GET', 'update'),
  updateCheck: () => call<Update>('POST', 'update/check'),
  updateInstall: () => call<Update>('POST', 'update/install'),
  restart: () => call<Record<string, never>>('POST', 'restart'),
  erase: (password: string) => call<Record<string, never>>('POST', 'erase', { password }),
};
