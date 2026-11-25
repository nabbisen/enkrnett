import { readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

export default async function globalTeardown(): Promise<void> {
  const pidFile = join(tmpdir(), 'enkrnett-harness.pid');
  try {
    const pid = Number(readFileSync(pidFile, 'utf8'));
    if (pid > 0) process.kill(pid, 'SIGTERM');
  } catch {
    // already gone
  }
  rmSync(pidFile, { force: true });
}
