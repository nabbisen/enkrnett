import { defineConfig, devices } from '@playwright/test';

// Two phone-sized projects (ENK-UI-020: 360 and 430 CSS pixels wide). WebKit stands in for
// Safari until the iOS Simulator joins CI; Chromium for Android.
const PORT = Number(process.env['ENKRNETT_PORT'] ?? 18080);

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: 1,
  retries: process.env['CI'] ? 1 : 0,
  timeout: 60_000,
  reporter: process.env['CI'] ? [['github'], ['list']] : 'list',
  globalSetup: './global-setup.ts',
  globalTeardown: './global-teardown.ts',
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    trace: 'retain-on-failure',
    locale: 'en-US',
  },
  projects: [
    {
      name: 'chromium-phone',
      use: { ...devices['Pixel 7'], viewport: { width: 360, height: 740 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true },
    },
    {
      name: 'webkit-phone',
      use: { ...devices['iPhone 14 Pro Max'], viewport: { width: 430, height: 932 }, isMobile: true, hasTouch: true },
    },
  ],
});
