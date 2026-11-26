// The update's own step and the install (chapter 11 §3; RFC 017 §5.6; ENK-UPD-002, 006, 007;
// ENK-UI-013). The harness's mock daemon offers 9.9.9 from a signed index on disk and, once the
// owner confirms, "flashes" it: the mock's root side answers flashing and the shape stays at
// installing, as on a device until it restarts. A mock cannot restart, so five seconds later its
// root side answers failed and the daemon is usable again, with the failure on the screen.
import { test, expect } from '@playwright/test';
import { claimViaApi, expectNoForbiddenWords, loginInBrowser } from './helpers';

test.describe.configure({ mode: 'serial' });

// One claimed device for both tests: the second continues where the first left the offer.
test.beforeAll(async () => {
  await claimViaApi();
});

test('the update has its own step that states the consequence, and can be left', async ({ page }) => {
  await loginInBrowser(page);
  await page.getByRole('button', { name: 'Details' }).click();
  await page.getByRole('button', { name: 'Turn on update checks?' }).click();
  await expect(page.getByText('Update available: Enkrnett 9.9.9')).toBeVisible();
  // The status screen's summary line says the same (RFC 017 §5.1), from its next poll (5 s).
  await page.getByRole('button', { name: 'Status' }).click();
  await expect(page.getByText('Update available: Enkrnett 9.9.9')).toBeVisible({ timeout: 10_000 });
  await page.getByRole('button', { name: 'Details' }).click();
  await page.getByRole('button', { name: 'Update', exact: true }).click();
  // ENK-UPD-002, 006: version, duration, the Wi-Fi stops, cannot be undone from here.
  await expect(page.getByText('Update available: Enkrnett 9.9.9. Installing takes about 3 minutes. The Wi-Fi stops meanwhile. This cannot be undone from here.')).toBeVisible();
  // ENK-UPD-007: the same baseline, so no second restart is announced.
  await expect(page.getByText('This one takes longer and restarts the Wi-Fi twice.')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Update', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Erase this device' })).toHaveCount(0);
  await expectNoForbiddenWords(page);
  await page.getByRole('button', { name: 'Cancel' }).click();
  await expect(page.getByText('Update available: Enkrnett 9.9.9', { exact: true })).toBeVisible();
});

test('confirming starts the install and the screen follows it to installing', async ({ page }) => {
  await loginInBrowser(page);
  await page.getByRole('button', { name: 'Details' }).click();
  await page.getByRole('button', { name: 'Update', exact: true }).click();
  await page.getByRole('button', { name: 'Update', exact: true }).click();
  await expect(page.getByText(/Downloading…|Verifying…|Installing\. The Wi-Fi stops for about two minutes/)).toBeVisible();
  await expect(page.getByText('Installing. The Wi-Fi stops for about two minutes. Come back to this address afterwards.')).toBeVisible({ timeout: 15_000 });
  // No button offers anything else while it runs.
  await expect(page.getByRole('button', { name: 'Check for updates' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Update', exact: true })).toHaveCount(0);
  await expectNoForbiddenWords(page);
  // The mock's root side gives up: the outcome of the attempt, and the offer stands again.
  await expect(page.getByText('The update could not be installed. Nothing was changed.')).toBeVisible({ timeout: 20_000 });
  await expect(page.getByRole('button', { name: 'Update', exact: true })).toBeVisible();
  await expectNoForbiddenWords(page);
});
