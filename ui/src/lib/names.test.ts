// ENK-UI-015; ENK-CFG-016 to 018.
import { describe, expect, it } from 'vitest';
import { byteLength, isSendable, normalise, roomLeft } from './names';

describe('names', () => {
  it('counts UTF-8 bytes after NFC normalisation', () => {
    expect(byteLength('Cafe')).toBe(4);
    expect(byteLength('カフェ')).toBe(9);
    // "か" + combining dakuten (NFD) becomes "が" (NFC): 3 bytes, not 6.
    expect(normalise('が')).toBe('が');
    expect(byteLength('が')).toBe(3);
  });

  it('reports the room left and refuses what the device would refuse', () => {
    expect(roomLeft('a'.repeat(29), 29)).toBe(0);
    expect(roomLeft('a'.repeat(30), 29)).toBe(-1);
    expect(isSendable('a'.repeat(29), 29)).toBe(true);
    expect(isSendable('a'.repeat(30), 29)).toBe(false);
    expect(isSendable('カフェの無料ワイファ', 29)).toBe(false); // 10 characters, 30 bytes
    expect(isSendable('カフェの無料ワイフ', 29)).toBe(true); // 9 characters, 27 bytes
    expect(isSendable(' cafe', 32)).toBe(false);
    expect(isSendable('cafe ', 32)).toBe(false);
    expect(isSendable('ca\tfe', 32)).toBe(false);
    expect(isSendable('', 32)).toBe(false);
    expect(isSendable('Cafe Example', 32)).toBe(true);
  });
});
