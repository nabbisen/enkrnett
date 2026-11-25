// Network names (ENK-UI-015; ENK-CFG-016 to 018): normalised to NFC, counted in UTF-8 bytes as
// the device counts them, with the room left shown while typing.

const encoder = new TextEncoder();

export function normalise(name: string): string {
  return name.normalize('NFC');
}

export function byteLength(name: string): number {
  return encoder.encode(normalise(name)).length;
}

/** Bytes still available; negative when over the limit. */
export function roomLeft(name: string, maxBytes: number): number {
  return maxBytes - byteLength(name);
}

/** Whether the value may be sent (ENK-CFG-016: 1..max bytes, no edge spaces, no control chars). */
export function isSendable(name: string, maxBytes: number): boolean {
  const n = normalise(name);
  if (n.length === 0) return false;
  if (byteLength(n) > maxBytes) return false;
  if (n.startsWith(' ') || n.endsWith(' ')) return false;
  for (const ch of n) {
    const code = ch.codePointAt(0) ?? 0;
    if (code < 0x20 || (code >= 0x7f && code <= 0x9f)) return false;
  }
  return true;
}
