// The claim in the browser, S-1 to S-9 (chapter 10 §4; ENK-UI-012, 015, 017, 025).
import { expect, test } from '@playwright/test';
import { PASSWORD, expectNoForbiddenWords, finishClaim, resetDevice } from './helpers';

test.describe.configure({ mode: 'serial' });

test.beforeEach(async () => {
  await resetDevice();
});

test('setup from start to the status screen, in English', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Set up this device' })).toBeVisible();
  await expect(page.getByText('Your phone joined the setup Wi-Fi with encryption')).toBeVisible();
  await expect(page.getByText('Development build.')).toBeVisible();
  await expectNoForbiddenWords(page);

  await page.getByRole('button', { name: 'Start' }).click();
  await expect(page.getByRole('heading', { name: 'Is this the device?' })).toBeVisible();
  await expect(page.getByText(/Step 2 of \d/)).toBeVisible();
  await page.getByRole('button', { name: 'Blink', exact: true }).click();
  await page.getByRole('button', { name: 'I saw it blink' }).click();

  await expect(page.getByRole('heading', { name: 'Country or region' })).toBeVisible();
  await page.getByLabel('Country or region', { exact: true }).selectOption('JP');
  await page.getByRole('button', { name: 'Next' }).click();

  await expect(page.getByRole('heading', { name: 'Guest Wi-Fi' })).toBeVisible();
  await expect(page.getByText('Up to 29 characters.')).toBeVisible();
  const name = page.getByLabel('Name of the guest Wi-Fi');
  await name.fill('カフェの無料ワイファイ'); // 11 characters = 33 bytes: too long for 29
  await expect(page.getByText('-4 left')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Next' })).toBeDisabled();
  await name.fill('Cafe Example');
  await expect(page.getByText('17 left')).toBeVisible();
  await expect(page.getByText('Open to every device')).toBeVisible();
  await expect(page.getByText('Encrypted only')).toBeVisible();
  await page.getByRole('button', { name: 'Next' }).click();

  await expect(page.getByRole('heading', { name: 'Owner password' })).toBeVisible();
  await page.getByLabel('Owner password', { exact: true }).fill('password123');
  await page.getByRole('button', { name: 'Next' }).click();
  await expect(page.getByText('This is a commonly used password.')).toBeVisible();
  await page.getByLabel('Owner password', { exact: true }).fill(PASSWORD);
  await page.getByRole('button', { name: 'Next' }).click();

  await expect(page.getByRole('heading', { name: 'Keep these' })).toBeVisible();
  await expect(page.getByText('Enkrnett-3F2A')).toBeVisible();
  const key = await page.locator('.mono').nth(1).innerText();
  expect(key).toMatch(/^[a-z2-9]{5}-[a-z2-9]{5}-[a-z2-9]{5}-[a-z2-9]{5}$/);
  await expect(page.getByRole('img', { name: 'Scan with another phone to join the management Wi-Fi' })).toBeVisible();
  await expect(page.getByRole('img', { name: 'Scan with another phone to open the settings screen' })).toBeVisible();
  await expect(page.getByText('http://192.168.207.1/')).toBeVisible();
  await expectNoForbiddenWords(page);
  await page.getByRole('button', { name: 'Set up' }).click();

  await expect(page.getByRole('heading', { name: 'Join the management Wi-Fi' })).toBeVisible();
  await expect(page.getByText('You have 10 minutes.')).toBeVisible();
  await expect(page.getByText(/\d+:\d\d left/)).toBeVisible();
  // The mock answers from PENDING at once, so the login form appears below the steps.
  await page.getByLabel('Owner password', { exact: true }).fill(PASSWORD);
  await page.getByRole('button', { name: 'Log in' }).click();

  await expect(page.getByRole('heading', { name: 'Status', exact: true })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Guest Wi-Fi "Cafe Example"' })).toBeVisible();
  await expect(page.getByText('running').first()).toBeVisible();
  await expect(page.getByText('No devices connected')).toBeVisible();
  await expect(page.getByText(/Enkrnett 0\.1\.0 on OpenWrt 25\.12/)).toBeVisible();
  await expectNoForbiddenWords(page);
});

test('a second phone is told that setup is in progress elsewhere', async ({ page, browser }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Start' }).click();
  await expect(page.getByRole('heading', { name: 'Is this the device?' })).toBeVisible();
  const other = await browser.newPage();
  await other.goto('/');
  await other.getByRole('button', { name: 'Start' }).click();
  await expect(other.getByText('Setup is in progress on another phone.')).toBeVisible();
  await other.close();
  // Do not leave the claim session behind: finish it from the first page's token.
  const token = await page.evaluate(() => sessionStorage.getItem('enkrnett-session'));
  expect(token).not.toBeNull();
  await finishClaim(token as string);
});

test('the Japanese catalogue renders the setup screens', async ({ page }) => {
  await page.goto('/');
  await page.getByLabel('Language').selectOption('ja');
  await expect(page.getByRole('heading', { name: 'この本体をセットアップ' })).toBeVisible();
  await expect(page.getByText('暗号化ありでセットアップ用Wi-Fiに接続しています')).toBeVisible();
  await expectNoForbiddenWords(page);
  await page.getByRole('button', { name: '開始' }).click();
  await expect(page.getByRole('heading', { name: 'この本体で間違いありませんか？' })).toBeVisible();
  await page.getByRole('button', { name: 'ランプを点滅' }).click();
  await page.getByRole('button', { name: '点滅を確認した' }).click();
  await page.getByLabel('国・地域', { exact: true }).selectOption('JP');
  await page.getByRole('button', { name: '次へ' }).click();
  await expect(page.getByText('最大29文字分')).toBeVisible();
  await page.getByLabel('ゲストWi-Fiの名前').fill('カフェの無料ワイフ'); // 9 characters, 27 bytes
  await expect(page.getByText('あと2')).toBeVisible();
  await expect(page.getByText('すべての端末が接続できる')).toBeVisible();
  await expectNoForbiddenWords(page);
  expect(await page.evaluate(() => document.documentElement.lang)).toBe('ja');
});
