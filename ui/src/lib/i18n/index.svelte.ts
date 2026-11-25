// Languages (ENK-UI-009, ENK-UI-010): one catalogue of complete sentences per language, named
// placeholders, the language's own plural rule, and a remembered choice that the UI never
// depends on.

import en from './en.json';
import ja from './ja.json';

export type Lang = 'en' | 'ja';
export const LANGUAGES: Lang[] = ['en', 'ja'];
export const LANGUAGE_NAMES: Record<Lang, string> = { en: 'English', ja: '日本語' };

const catalogues: Record<Lang, Record<string, string>> = { en, ja };

function fromBrowser(): Lang {
  const candidates = typeof navigator !== 'undefined' ? [...navigator.languages, navigator.language] : [];
  for (const c of candidates) {
    const base = (c ?? '').toLowerCase().split('-')[0];
    if (base === 'ja') return 'ja';
    if (base === 'en') return 'en';
  }
  return 'en';
}

function remembered(): Lang | null {
  try {
    const v = localStorage.getItem('enkrnett-lang');
    return v === 'en' || v === 'ja' ? v : null;
  } catch {
    return null;
  }
}

class I18n {
  lang = $state<Lang>(remembered() ?? fromBrowser());

  set(lang: Lang): void {
    this.lang = lang;
    try {
      localStorage.setItem('enkrnett-lang', lang);
    } catch {
      // a convenience only (ENK-UI-010)
    }
    if (typeof document !== 'undefined') document.documentElement.lang = lang;
  }

  /** The sentence for `key`, with `{name}` placeholders filled. */
  t(key: string, params: Record<string, string | number> = {}): string {
    const text = catalogues[this.lang][key];
    if (text === undefined) {
      console.warn(`i18n: missing key ${key} (${this.lang})`);
      return key;
    }
    return fill(text, params);
  }

  /** Plural: English picks `.one` for exactly one, `.other` otherwise; Japanese always `.other`. */
  n(key: string, count: number, params: Record<string, string | number> = {}): string {
    const form = this.lang === 'en' && count === 1 ? 'one' : 'other';
    return this.t(`${key}.${form}`, { n: count, ...params });
  }
}

export function fill(text: string, params: Record<string, string | number>): string {
  return text.replace(/\{(\w+)\}/g, (m, name: string) => (name in params ? String(params[name]) : m));
}

export const i18n = new I18n();

/** For tests: the raw catalogues. */
export function catalogue(lang: Lang): Record<string, string> {
  return catalogues[lang];
}
