// Shared steps: resetting the device through the API and walking the claim in the browser.
import { expect, type Page } from '@playwright/test';

export const PASSWORD = 'correct horse battery staple';
export const BASE = `http://127.0.0.1:${process.env['ENKRNETT_PORT'] ?? 18080}`;

async function api(path: string, method: 'GET' | 'POST', token?: string, body?: unknown): Promise<{ status: number; json: any }> {
  const headers: Record<string, string> = {};
  if (token) headers['Enkrnett-Session'] = token;
  if (method === 'POST') headers['Content-Type'] = 'application/json';
  const r = await fetch(`${BASE}/api/1/${path}`, { method, headers, body: method === 'POST' ? JSON.stringify(body ?? {}) : null });
  let json: any = null;
  try {
    json = await r.json();
  } catch {
    json = null;
  }
  return { status: r.status, json };
}

/** Brings the device to SETUP whatever it is in: erases a claimed device, or waits in PENDING. */
export async function resetDevice(): Promise<void> {
  for (let i = 0; i < 50; i++) {
    const g = await api('', 'GET');
    if (g.json?.state === 'SETUP') return;
    if (g.json?.state === 'CLAIMED') {
      const login = await api('login', 'POST', undefined, { password: PASSWORD });
      if (login.status === 200) {
        await api('erase', 'POST', login.json.token, { password: PASSWORD });
      } else if (login.status === 429) {
        throw new Error('login delay active; the device cannot be reset by the harness');
      } else {
        throw new Error(`cannot log in to reset: ${login.status} ${JSON.stringify(login.json)}`);
      }
    }
    await new Promise((f) => setTimeout(f, 200));
  }
  throw new Error('device did not return to SETUP');
}

/** Finishes a claim whose session `token` a browser page already holds, so that the device
 * ends up CLAIMED and `resetDevice` can erase it. A test must never leave a claim session
 * behind: the device would rightly refuse the next test's Start for five minutes. */
export async function finishClaim(token: string): Promise<void> {
  await api('claim/check', 'POST', token, {
    country: 'JP',
    guest: { name: 'Cafe Example', mode: 'open_to_all' },
    owner_password: PASSWORD,
  });
  await api('claim/confirm', 'POST', token);
  for (let i = 0; i < 50; i++) {
    const g = await api('', 'GET');
    if (g.json?.state === 'PENDING') break;
    await new Promise((f) => setTimeout(f, 100));
  }
  await api('login', 'POST', undefined, { password: PASSWORD });
}

/** Claims the device through the API (fast path for screens after setup). */
export async function claimViaApi(): Promise<void> {
  await resetDevice();
  const start = await api('claim/start', 'POST');
  if (start.status !== 200) {
    throw new Error(`claim/start refused (${start.status}): a previous test left a claim session behind`);
  }
  const token = start.json.token as string;
  const check = await api('claim/check', 'POST', token, {
    country: 'JP',
    guest: { name: 'Cafe Example', mode: 'open_to_all' },
    owner_password: PASSWORD,
  });
  expect(check.status).toBe(200);
  const confirm = await api('claim/confirm', 'POST', token);
  expect(confirm.status).toBe(200);
  for (let i = 0; i < 50; i++) {
    const g = await api('', 'GET');
    if (g.json?.state === 'PENDING') break;
    await new Promise((f) => setTimeout(f, 100));
  }
  const login = await api('login', 'POST', undefined, { password: PASSWORD });
  expect(login.status).toBe(200);
}

/** Logs in through the browser and lands on Status. */
export async function loginInBrowser(page: Page): Promise<void> {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: /Log in|ログイン/ })).toBeVisible();
  await page.getByLabel(/Owner password|管理者パスワード/).fill(PASSWORD);
  await page.getByRole('button', { name: /^Log in$|^ログイン$/ }).click();
  await expect(page.getByRole('heading', { name: /^Status$|^状態$/ })).toBeVisible();
}

/** Forbidden words (RFC 017 §8) that must not appear in any rendered screen. */
export const FORBIDDEN_RENDERED = [
  /\bsecure\b/i,
  /\binsecure\b/i,
  /\bSSID\b/,
  /\bWPA2\b/,
  /\bWPA3\b/,
  /\bPMF\b/,
  /transition mode/i,
  /\brollback\b/i,
  /\bpairing\b/i,
  /\breset\b/i,
  /管理用/,
  /初期設定/,
  /リセット/,
  /セキュリティ/,
  /トランジション/,
];

export async function expectNoForbiddenWords(page: Page): Promise<void> {
  const text = await page.locator('body').innerText();
  for (const re of FORBIDDEN_RENDERED) {
    expect(text, `forbidden word ${re} on screen`).not.toMatch(re);
  }
}
