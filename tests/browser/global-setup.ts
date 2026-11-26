// Starts `enkrnettd --mock` with the built UI on a fixed port and waits for the greeting. Each
// test file erases the device through the daemon's own API when it needs a fresh state. The mock
// is given a signed release index from disk, so that an update can be offered and started.
import { spawn, execFileSync, type ChildProcess } from 'node:child_process';
import { mkdtempSync, writeFileSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const PORT = Number(process.env['ENKRNETT_PORT'] ?? 18080);
const ROOT = resolve(import.meta.dirname, '..', '..');

async function waitForGreeting(): Promise<void> {
  for (let i = 0; i < 100; i++) {
    try {
      const r = await fetch(`http://127.0.0.1:${PORT}/api/1/`);
      if (r.ok) return;
    } catch {
      // not yet
    }
    await new Promise((f) => setTimeout(f, 200));
  }
  throw new Error('enkrnettd did not answer');
}

export default async function globalSetup(): Promise<void> {
  const uiDist = join(ROOT, 'ui', 'dist', 'index.html');
  if (!existsSync(uiDist)) {
    throw new Error(`UI not built: ${uiDist} (run \`npm run build\` in ui/)`);
  }
  const binary = process.env['ENKRNETTD'] ?? join(ROOT, 'target', 'debug', 'enkrnettd');
  if (!existsSync(binary)) {
    execFileSync('cargo', ['build', '-q', '-p', 'enkrnettd'], { cwd: ROOT, stdio: 'inherit' });
  }
  const stateDir = mkdtempSync(join(tmpdir(), 'enkrnett-harness-'));
  // A signed release index and a pretend image, made with the release tool and served to the
  // mock daemon from disk (file:// URLs; the key beside the index is the one the "image" carries),
  // so that the update screens can be exercised (chapter 11 §3).
  const tool = process.env['ENKRNETT_RELEASE'] ?? join(ROOT, 'target', 'debug', 'enkrnett-release');
  if (!existsSync(tool)) {
    execFileSync('cargo', ['build', '-q', '-p', 'enkrnett-release'], { cwd: ROOT, stdio: 'inherit' });
  }
  const www = mkdtempSync(join(tmpdir(), 'enkrnett-harness-www-'));
  const image = join(www, 'enkrnett-9.9.9-25.12-mock-sysupgrade.bin');
  writeFileSync(image, Buffer.alloc(300_000, 7));
  execFileSync(tool, ['keygen', '-c', 'enkrnett browser harness', join(www, 'harness')], { stdio: 'inherit' });
  execFileSync(tool, ['sign', join(www, 'harness.sec'), image], { stdio: 'inherit' });
  execFileSync(
    tool,
    ['index', '--sequence', '1', '--out', join(www, 'index.json'), '--release', 'mock', '9.9.9', '25.12', '25.12', image, '16384', 'https://example.org/releases/9.9.9', '180'],
    { stdio: 'inherit' },
  );
  execFileSync(tool, ['sign', join(www, 'harness.sec'), join(www, 'index.json')], { stdio: 'inherit' });
  const child: ChildProcess = spawn(
    binary,
    [
      '--mock', '--listen', `127.0.0.1:${PORT}`, '--state-dir', stateDir, '--ui-dir', join(ROOT, 'ui', 'dist'), '--act-delay-ms', '0', '--development',
      '--update-index', `file://${join(www, 'index.json')}`,
    ],
    { stdio: ['ignore', 'inherit', 'inherit'], detached: false },
  );
  writeFileSync(join(tmpdir(), 'enkrnett-harness.pid'), String(child.pid));
  await waitForGreeting();
}
