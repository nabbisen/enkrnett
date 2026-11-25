// Every enumeration value and error code of chapter 9 maps to exactly one sentence
// (ENK-UI-017); an unknown value maps to the generic sentence and is logged.

import { i18n } from './i18n/index.svelte';
import type { ChangeState, FieldCode, HealthDetail, UndoReason } from './types';
import { ApiError, NotConnected } from './api';

const KNOWN_ERRORS = new Set([
  'no_session',
  'wrong_password',
  'try_later',
  'wrong_state',
  'not_available',
  'setup_in_progress_elsewhere',
  'no_proposal',
  'version_mismatch',
  'change_pending',
  'update_in_progress',
  'nothing_to_install',
  'cannot_update',
  'invalid',
]);

const KNOWN_FIELDS: FieldCode[] = [
  'required',
  'too_short',
  'too_long',
  'forbidden_characters',
  'common_password',
  'equals_network_name',
  'equals_key',
  'equals_device_id',
  'not_in_choices',
  'not_allowed',
  'unknown_field',
  'wrong_type',
];

export function errorSentence(e: unknown): string {
  if (e instanceof NotConnected) return '';
  if (e instanceof ApiError) {
    const code = e.detail.code;
    if (code === 'try_later') return i18n.t('error.try_later', { s: e.detail.retry_in ?? 30 });
    if (KNOWN_ERRORS.has(code)) return i18n.t(`error.${code}`);
    console.warn('unknown error code', code, e.detail);
    return i18n.t('error.generic', { ref: e.detail.reference ?? code });
  }
  console.warn('unexpected error', e);
  return i18n.t('error.generic', { ref: '-' });
}

export function fieldSentence(code: FieldCode): string {
  if (KNOWN_FIELDS.includes(code)) return i18n.t(`field.${code}`);
  console.warn('unknown field code', code);
  return i18n.t('field.wrong_type');
}

export function changeSentence(state: ChangeState, reason: UndoReason | null, detail: HealthDetail | null): string {
  switch (state) {
    case 'none':
      return '';
    case 'validating':
    case 'applying':
    case 'waiting_confirmation':
    case 'settled':
      return i18n.t(`change.${state}`);
    case 'undone': {
      const base = i18n.t(`change.undone.${reason ?? 'health_check_failed'}`);
      return detail ? `${base} ${i18n.t(`change.detail.${detail}`)}` : base;
    }
    default:
      console.warn('unknown change state', state);
      return i18n.t('error.generic', { ref: String(state) });
  }
}
