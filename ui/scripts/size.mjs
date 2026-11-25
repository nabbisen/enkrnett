#!/usr/bin/env node
// ENK-FOOT-002: the UI bundle, all languages included, is at most 150 KiB inside the squashfs.
// The squashfs uses xz, so the sum of the files compressed with `xz -9` stands in for the
// figure until the image build measures the real one.
import { readdirSync, statSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { execFileSync } from 'node:child_process';
import { gzipSync } from 'node:zlib';
import { fileURLToPath } from 'node:url';
import { dirname } from 'node:path';

const BUDGET_KIB = 150;
const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, '..', 'dist');

function walk(dir) {
  const out = [];
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) out.push(...walk(p));
    else out.push(p);
  }
  return out;
}

function compressedSize(path) {
  try {
    const out = execFileSync('xz', ['-9', '-k', '-c', path], { maxBuffer: 64 * 1024 * 1024 });
    return { bytes: out.length, how: 'xz' };
  } catch {
    return { bytes: gzipSync(readFileSync(path), { level: 9 }).length, how: 'gzip' };
  }
}

let total = 0;
let raw = 0;
let how = 'xz';
const rows = [];
for (const file of walk(dist)) {
  const r = compressedSize(file);
  how = r.how;
  total += r.bytes;
  raw += statSync(file).size;
  rows.push([relative(dist, file), statSync(file).size, r.bytes]);
}
rows.sort((a, b) => b[2] - a[2]);
for (const [name, size, c] of rows) console.log(`${String(c).padStart(8)}  ${String(size).padStart(8)}  ${name}`);
const kib = total / 1024;
console.log(`UI bundle: ${kib.toFixed(1)} KiB compressed (${how}), ${(raw / 1024).toFixed(1)} KiB raw; budget ${BUDGET_KIB} KiB (ENK-FOOT-002)`);
if (kib > BUDGET_KIB) {
  console.error(`size: over budget by ${(kib - BUDGET_KIB).toFixed(1)} KiB`);
  process.exit(1);
}
