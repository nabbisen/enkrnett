// ENK-WORD-001, ENK-UI-008: the check finds what it must and lets the notice say OWE once.
import { describe, expect, it } from 'vitest';
import { check, checkAll } from './wordcheck.mjs';

describe('wordcheck', () => {
  it('finds forbidden English words as whole words', () => {
    expect(check({ 'a.b': 'This network is secure.' }, 'en')).toEqual([{ key: 'a.b', word: 'secure' }]);
    expect(check({ 'a.b': 'Press reset.' }, 'en')).toEqual([{ key: 'a.b', word: 'reset' }]);
    expect(check({ 'a.b': 'Transition Mode is on' }, 'en').map((o) => o.word)).toContain('transition mode');
    expect(check({ 'a.b': 'the management key' }, 'en').map((o) => o.word)).toContain('management key');
    expect(check({ 'a.b': 'Enter a value.' }, 'en')).toEqual([]);
    expect(check({ 'a.b': 'securely is not a listed word' }, 'en')).toEqual([]);
  });

  it('finds forbidden Japanese words and the lone label', () => {
    expect(check({ 'a.b': '管理用Wi-Fiに接続' }, 'ja')).toEqual([{ key: 'a.b', word: '管理用' }]);
    expect(check({ 'a.b': '初期設定を開始' }, 'ja')).toEqual([{ key: 'a.b', word: '初期設定' }]);
    expect(check({ 'a.b': '安全' }, 'ja').map((o) => o.word)).toContain('安全');
    expect(check({ 'a.b': '安全な場所に控えてください。' }, 'ja')).toEqual([]);
  });

  it('allows OWE and Enhanced Open in the notice only', () => {
    expect(check({ 'notice.p1': 'called "Enhanced Open" or "OWE"' }, 'en')).toEqual([]);
    expect(check({ 'status.x': 'uses OWE' }, 'en')).toEqual([{ key: 'status.x', word: 'owe' }]);
    expect(check({ 'notice.p1': '規格の名前は「Enhanced Open」または「OWE」です' }, 'ja')).toEqual([]);
    expect(check({ 'status.x': 'OWEで暗号化' }, 'ja')).toEqual([{ key: 'status.x', word: 'OWE' }]);
  });

  it('passes on the shipped catalogues', () => {
    expect(checkAll()).toEqual([]);
  });
});
