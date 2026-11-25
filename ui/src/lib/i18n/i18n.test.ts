// ENK-UI-009, ENK-UI-017: both catalogues have the same keys; every API value has a sentence;
// placeholders match between the languages.
import { describe, expect, it } from 'vitest';
import en from './en.json';
import ja from './ja.json';
import { fill } from './index.svelte';

const ERROR_CODES = [
  'no_session', 'wrong_password', 'try_later', 'wrong_state', 'not_available', 'setup_in_progress_elsewhere',
  'no_proposal', 'version_mismatch', 'change_pending', 'update_in_progress', 'nothing_to_install', 'cannot_update', 'invalid',
];
const FIELD_CODES = [
  'required', 'unknown_field', 'wrong_type', 'too_short', 'too_long', 'not_allowed', 'forbidden_characters',
  'common_password', 'equals_network_name', 'equals_key', 'equals_device_id', 'not_in_choices',
];
const CHANGE_STATES = ['validating', 'applying', 'waiting_confirmation', 'settled'];
const UNDO_REASONS = ['health_check_failed', 'not_confirmed', 'restart'];
const DETAILS = ['guest_down', 'management_down', 'isolation_missing', 'dhcp_missing', 'listener_exposed', 'pmf_missing', 'mode_differs'];
const UPDATE_FAILURES = ['bad_signature', 'wrong_device', 'wrong_baseline', 'not_enough_memory', 'download_failed', 'health_check_failed'];

function placeholders(text: string): string[] {
  return [...text.matchAll(/\{(\w+)\}/g)].map((m) => m[1] ?? '').sort();
}

describe('catalogues', () => {
  it('have the same keys', () => {
    expect(Object.keys(ja).sort()).toEqual(Object.keys(en).sort());
  });

  it('use the same placeholders per key', () => {
    for (const [key, text] of Object.entries(en)) {
      expect(placeholders((ja as Record<string, string>)[key] ?? ''), key).toEqual(placeholders(text));
    }
  });

  it('cover every enumeration value and error code of chapter 9', () => {
    for (const c of ERROR_CODES) expect(en, c).toHaveProperty(`error.${c}`);
    for (const c of FIELD_CODES) expect(en, c).toHaveProperty(`field.${c}`);
    for (const s of CHANGE_STATES) expect(en, s).toHaveProperty(`change.${s}`);
    for (const r of UNDO_REASONS) expect(en, r).toHaveProperty(`change.undone.${r}`);
    for (const d of DETAILS) expect(en, d).toHaveProperty(`change.detail.${d}`);
    for (const f of UPDATE_FAILURES) expect(en, f).toHaveProperty(`update.failed.${f}`);
    expect(en).toHaveProperty('error.generic');
  });

  it('have no empty sentences and no markup', () => {
    for (const cat of [en, ja]) {
      for (const [key, text] of Object.entries(cat)) {
        expect(text.trim().length, key).toBeGreaterThan(0);
        expect(text, key).not.toMatch(/<[a-z]/i);
      }
    }
  });

  it('fill placeholders and leave unknown ones visible', () => {
    expect(fill('{n} left', { n: 3 })).toBe('3 left');
    expect(fill('{a} {b}', { a: 'x' })).toBe('x {b}');
  });
});
