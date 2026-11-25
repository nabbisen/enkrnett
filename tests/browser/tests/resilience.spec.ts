// ENK-UI-019 (not connected), ENK-UI-023 (storage blocked), ENK-UI-020 (no horizontal scroll),
// ENK-UI-026 (accessibility), ENK-UI-010 (language from the browser).
import AxeBuilder from '@axe-core/playwright';
import { expect, test } from '@playwright/test';
import { claimViaApi, loginInBrowser, resetDevice } from './helpers';

test('a failed poll shows "not connected" and the screen continues by itself', async ({ page }) => {
  await claimViaApi();
  await loginInBrowser(page);
  await page.route('**/api/1/**', (route) => route.abort('connectionrefused'));
  await expect(page.getByText(/Not connected to the device\. Join the management Wi-Fi "Enkrnett-3F2A"/)).toBeVisible({ timeout: 10_000 });
  await page.unroute('**/api/1/**');
  await expect(page.getByText(/Not connected to the device/)).toBeHidden({ timeout: 10_000 });
  await expect(page.getByRole('heading', { name: 'Guest Wi-Fi "Cafe Example"' })).toBeVisible();
});

test('the UI works with browser storage blocked', async ({ browser }) => {
  await resetDevice();
  const context = await browser.newContext({ viewport: { width: 360, height: 740 } });
  await context.addInitScript(() => {
    const deny = () => {
      throw new DOMException('blocked', 'SecurityError');
    };
    Object.defineProperty(window, 'localStorage', { get: deny });
    Object.defineProperty(window, 'sessionStorage', { get: deny });
  });
  const page = await context.newPage();
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Set up this device' })).toBeVisible();
  await page.getByLabel('Language').selectOption('ja');
  await expect(page.getByRole('heading', { name: 'この本体をセットアップ' })).toBeVisible();
  await page.reload();
  // The choice could not be remembered; the screen still works and falls back to the browser's language.
  await expect(page.getByRole('heading', { name: 'Set up this device' })).toBeVisible();
  await context.close();
});

test('no horizontal scrolling at 360 pixels and targets of at least 44 pixels', async ({ page }) => {
  await resetDevice();
  await page.setViewportSize({ width: 360, height: 740 });
  await page.goto('/');
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth);
  expect(overflow).toBe(false);
  const small = await page.evaluate(() =>
    [...document.querySelectorAll('button, select, input')]
      .map((el) => el.getBoundingClientRect())
      .filter((r) => r.width > 0 && (r.height < 44 || r.width < 44))
      .length,
  );
  expect(small).toBe(0);
});

test('accessibility audit of the start, login, and status screens', async ({ page }) => {
  await resetDevice();
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Set up this device' })).toBeVisible();
  let results = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
  expect(results.violations, JSON.stringify(results.violations, null, 2)).toEqual([]);
  await claimViaApi();
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Log in' })).toBeVisible();
  results = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
  expect(results.violations, JSON.stringify(results.violations, null, 2)).toEqual([]);
  await loginInBrowser(page);
  results = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
  expect(results.violations, JSON.stringify(results.violations, null, 2)).toEqual([]);
});

test('the language follows the browser', async ({ browser }) => {
  await resetDevice();
  const context = await browser.newContext({ locale: 'ja-JP', viewport: { width: 360, height: 740 } });
  const page = await context.newPage();
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'この本体をセットアップ' })).toBeVisible();
  await context.close();
});
