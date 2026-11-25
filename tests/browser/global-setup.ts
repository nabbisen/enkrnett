// Starts `enkrnettd --mock` with the built UI on a fixed port and waits for the greeting. Each
// test file erases the device through the daemon's own API when it needs a fresh state.
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
  const child: ChildProcess = spawn(
    binary,
    ['--mock', '--listen', `127.0.0.1:${PORT}`, '--state-dir', stateDir, '--ui-dir', join(ROOT, 'ui', 'dist'), '--act-delay-ms', '0', '--development'],
    { stdio: ['ignore', 'inherit', 'inherit'], detached: false },
  );
  writeFileSync(join(tmpdir(), 'enkrnett-harness.pid'), String(child.pid));
  await waitForGreeting();
}
