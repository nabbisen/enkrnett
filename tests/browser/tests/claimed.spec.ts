// The claimed device in the browser: Status, Settings, Management Wi-Fi, Details, Erase
// (ENK-UI-013, 016, 017, 021, 024, 027; ENK-OWE-011 the count of devices that lose the network
// before saving; ENK-OWE-016 mode and counts on the first screen; ENK-OWE-018 and ENK-GUEST-001
// "What this protects" and the printable notice carry the same text).
import { expect, test } from '@playwright/test';
import { PASSWORD, claimViaApi, expectNoForbiddenWords, loginInBrowser } from './helpers';

test.describe.configure({ mode: 'serial' });

test.beforeEach(async () => {
  await claimViaApi();
});

test('the status screen answers the first question with no tap', async ({ page }) => {
  await loginInBrowser(page);
  await expect(page.getByRole('heading', { name: 'Guest Wi-Fi "Cafe Example"' })).toBeVisible();
  await expect(page.getByText('Open to every device')).toBeVisible();
  await expect(page.getByText('No devices connected')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Management Wi-Fi "Enkrnett-3F2A"' })).toBeVisible();
  await expect(page.getByText('Internet connection')).toBeVisible();
  // Colour marks only running / not running: the dots exist, no other coloured badge.
  expect(await page.locator('.dot.running').count()).toBeGreaterThanOrEqual(2);
  await expectNoForbiddenWords(page);
  // Nothing loads from outside the device (ENK-MGMT-002).
  const external: string[] = [];
  page.on('request', (r) => {
    if (!r.url().startsWith('http://127.0.0.1:')) external.push(r.url());
  });
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
  expect(external).toEqual([]);
});

test('settings: the mode switch warns, the save settles, the name limit follows the mode', async ({ page }) => {
  await loginInBrowser(page);
  await page.getByRole('button', { name: 'Settings' }).click();
  await page.getByText('Encrypted only').click();
  await expect(page.getByText(/connected now (is|are) not encrypted/)).toBeVisible();
  await expect(page.getByText('Up to 32 characters.')).toBeVisible();
  await page.getByText('Open to every device').click();
  await expect(page.getByText('Up to 29 characters.')).toBeVisible();
  await page.getByLabel('Name of the guest Wi-Fi').fill('Cafe Example 2');
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByText('Saved.')).toBeVisible();
  await page.getByRole('button', { name: 'Status' }).click();
  await expect(page.getByRole('heading', { name: 'Guest Wi-Fi "Cafe Example 2"' })).toBeVisible();
  await page.getByRole('button', { name: 'Settings' }).click();
  await page.getByLabel('Name of the management Wi-Fi').fill('Office');
  await expect(page.getByText('Saving this changes the management Wi-Fi.')).toBeVisible();
  await expectNoForbiddenWords(page);
});

test('management Wi-Fi: password on request, QR code, replace on its own step', async ({ page }) => {
  await loginInBrowser(page);
  await page.getByRole('button', { name: 'Management Wi-Fi' }).click();
  await expect(page.getByText('Enkrnett-3F2A')).toBeVisible();
  await expect(page.locator('.mono').filter({ hasText: /-/ })).toHaveCount(1); // only the name before showing
  await page.getByRole('button', { name: 'Show the password' }).click();
  await expect(page.getByRole('img', { name: 'Scan with another phone to join the management Wi-Fi' })).toBeVisible();
  await page.getByRole('button', { name: 'Replace the password', exact: true }).click();
  await expect(page.getByText('Every device on the management Wi-Fi, including this phone, must join again')).toBeVisible();
  await page.getByRole('button', { name: 'Replace the password now' }).click();
  await expect(page.getByRole('heading', { name: 'The new management Wi-Fi password' })).toBeVisible();
  await expectNoForbiddenWords(page);
});

test('details: version, hardening report, restart on its own step, log out', async ({ page }) => {
  await loginInBrowser(page);
  await page.getByRole('button', { name: 'Details' }).click();
  await expect(page.getByText(/Enkrnett 0\.1\.0 on OpenWrt 25\.12/)).toBeVisible();
  await expect(page.getByText('ENK-HARD-001: in place')).toBeVisible();
  await expect(page.getByText('Installed from an image')).toBeVisible();
  await page.getByRole('button', { name: 'Check for updates' }).click();
  await expect(page.getByText('Could not reach the update server.')).toBeVisible();
  await page.getByRole('button', { name: 'Restart', exact: true }).first().click();
  await expect(page.getByText('Restart turns the device off and on.')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Erase this device' })).toHaveCount(0); // never beside Restart
  await page.getByRole('button', { name: 'Cancel' }).click();
  await expectNoForbiddenWords(page);
  await page.getByRole('button', { name: 'Log out' }).click();
  await expect(page.getByRole('heading', { name: 'Log in' })).toBeVisible();
});

test('what this protects and the printable notice carry the same text', async ({ page }) => {
  await loginInBrowser(page);
  await page.getByRole('button', { name: 'What this protects' }).click();
  await expect(page.getByRole('heading', { name: 'What this protects' })).toBeVisible();
  await expect(page.getByText('This access point keeps no record of the devices that connect to it.')).toBeVisible();
  await expect(page.getByText(/"Enhanced Open" or "OWE"/)).toBeVisible();
  await page.goto('/notice');
  await loginInBrowser(page).catch(() => {});
  await page.goto('/notice');
  await expect(page.getByRole('heading', { name: 'Wi-Fi "Cafe Example"' })).toBeVisible();
  await expect(page.getByText('This access point keeps no record of the devices that connect to it.')).toBeVisible();
});

test('erase asks the password again and returns the device to setup', async ({ page }) => {
  await loginInBrowser(page);
  await page.getByRole('button', { name: 'Settings' }).click();
  await page.getByRole('button', { name: 'Erase this device…' }).click();
  await expect(page.getByRole('heading', { name: 'Erase' })).toBeVisible();
  await expect(page.getByText('Erase removes everything you set up')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Restart' })).toHaveCount(0);
  await page.getByLabel('Owner password').fill('wrong wrong wrong');
  await page.getByRole('button', { name: 'Erase this device' }).click();
  await expect(page.getByText('The password is not correct.')).toBeVisible();
  await page.getByLabel('Owner password').fill(PASSWORD);
  await page.getByRole('button', { name: 'Erase this device' }).click();
  await expect(page.getByText('Erasing. The device restarts')).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Set up this device' })).toBeVisible({ timeout: 15_000 });
});
