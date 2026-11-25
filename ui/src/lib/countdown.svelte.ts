// Countdowns from the device's `*_in` seconds and a monotonic clock in the browser, never the
// date (ENK-UI-018). `set(seconds)` on every poll; `remaining` ticks down between polls.

export class Countdown {
  private base = $state(0);
  private at = $state(0);
  private now = $state(performance.now());
  private timer: ReturnType<typeof setInterval> | null = null;

  constructor() {
    this.timer = setInterval(() => {
      this.now = performance.now();
    }, 500);
  }

  set(seconds: number): void {
    this.base = seconds;
    this.at = performance.now();
    this.now = this.at;
  }

  get remaining(): number {
    const elapsed = Math.floor((this.now - this.at) / 1000);
    return Math.max(0, this.base - elapsed);
  }

  get text(): string {
    const r = this.remaining;
    const m = Math.floor(r / 60);
    const ss = String(r % 60).padStart(2, '0');
    return `${m}:${ss}`;
  }

  stop(): void {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
  }
}
