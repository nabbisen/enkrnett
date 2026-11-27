#!/usr/bin/env node
// The word check (ENK-WORD-001, ENK-UI-008; RFC 017 §8): the build fails when a catalogue string
// or a paragraph of an operator guide (docs/src/guides) contains a word that never appears in
// operator-facing text. `OWE` and `Enhanced Open` are
// allowed once, in the guest notice entry only.
import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));

// Case-insensitive for English, exact for Japanese. Matched as whole words where the language
// has word boundaries.
export const FORBIDDEN_EN = [
  'secure', 'insecure', 'security', 'transition mode', 'transition', 'wpa2', 'wpa3', 'pmf', 'sae',
  'ssid', 'bssid', 'bss', 'reset', 'apply', 'commit', 'rollback', 'pairing', 'pair', 'uplink', 'claim',
  'management key', 'guest network', 'management network', 'setup network',
];
export const FORBIDDEN_EN_NOTICE_ONLY = ['owe', 'enhanced open'];
export const FORBIDDEN_JA = [
  'セキュア', 'セキュリティ', 'トランジション', '移行モード', 'WPA2', 'WPA3', 'PMF', 'SAE', 'SSID', 'BSSID',
  'リセット', '適用', 'コミット', 'ロールバック', 'ペアリング', '管理用', '初期設定', '管理キー',
  'ゲストネットワーク', '管理ネットワーク', 'セットアップネットワーク',
];
export const FORBIDDEN_JA_NOTICE_ONLY = ['OWE', 'Enhanced Open'];
// 安全 only as a one-word label: a string that is exactly that word.
export const FORBIDDEN_JA_EXACT = ['安全', 'セキュリティ'];

function englishHits(text, words) {
  const lower = text.toLowerCase();
  return words.filter((w) => new RegExp(`(^|[^a-z0-9])${w.replace(/ /g, '\\s+')}([^a-z0-9]|$)`).test(lower));
}

function japaneseHits(text, words) {
  return words.filter((w) => text.includes(w));
}

/** Returns a list of `{ key, word }` offences for one catalogue. */
export function check(catalogue, lang) {
  const offences = [];
  for (const [key, text] of Object.entries(catalogue)) {
    const isNotice = key.startsWith('notice.');
    let hits = [];
    if (lang === 'en') {
      hits = englishHits(text, FORBIDDEN_EN);
      if (!isNotice) hits = hits.concat(englishHits(text, FORBIDDEN_EN_NOTICE_ONLY));
    } else {
      hits = japaneseHits(text, FORBIDDEN_JA);
      if (!isNotice) hits = hits.concat(japaneseHits(text, FORBIDDEN_JA_NOTICE_ONLY));
      if (FORBIDDEN_JA_EXACT.includes(text.trim())) hits.push(text.trim());
    }
    for (const word of hits) offences.push({ key, word });
  }
  return offences;
}

/** The operator guides (docs/src/guides/*.md) are operator-facing text too (chapter 1 §4); the
 * expert install page is the one exempt view, clearly marked as such. Each guide is checked
 * paragraph by paragraph, as English, with the notice-only words forbidden as well. */
export function checkGuides(dir = join(here, '..', '..', 'docs', 'src', 'guides')) {
  const out = [];
  let files = [];
  try { files = readdirSync(dir).filter((f) => f.endsWith('.md') && f !== 'expert-install.md').sort(); } catch { return out; }
  for (const f of files) {
    const paragraphs = readFileSync(join(dir, f), 'utf8').split(/\n\s*\n/);
    paragraphs.forEach((text, i) => {
      const hits = englishHits(text, FORBIDDEN_EN).concat(englishHits(text, FORBIDDEN_EN_NOTICE_ONLY));
      for (const word of hits) out.push({ lang: 'guide', key: `${f} ¶${i + 1}`, word });
    });
  }
  return out;
}

export function checkAll() {
  const out = [];
  for (const lang of ['en', 'ja']) {
    const catalogue = JSON.parse(readFileSync(join(here, '..', 'src', 'lib', 'i18n', `${lang}.json`), 'utf8'));
    for (const o of check(catalogue, lang)) out.push({ lang, ...o });
  }
  return out.concat(checkGuides());
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const offences = checkAll();
  if (offences.length > 0) {
    for (const o of offences) console.error(`wordcheck: ${o.lang} "${o.key}" contains "${o.word}"`);
    process.exit(1);
  }
  console.log('wordcheck: both catalogues and the guides clean');
}
