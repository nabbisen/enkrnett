<script lang="ts">
  import { api, ApiError, setToken } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { errorSentence, fieldSentence } from '../lib/errors';
  import { i18n } from '../lib/i18n/index.svelte';
  import type { Device, Update } from '../lib/types';
  import PasswordField from '../components/PasswordField.svelte';

  // S-14: version line, update block, model and identifier, Internet connection, hardening
  // report, change owner password, Restart (own step), Log out. Erase is not here.
  let device = $state<Device | null>(null);
  let update = $state<Update | null>(null);
  let error = $state('');
  let checking = $state(false);

  let changingPassword = $state(false);
  let current = $state('');
  let next = $state('');
  let passwordError = $state('');
  let passwordDone = $state(false);

  let restarting = $state(false);
  let restarted = $state(false);

  $effect(() => {
    void load();
  });

  async function load() {
    try {
      device = await api.device();
      update = await api.update();
    } catch (e) {
      if (!app.handle(e)) error = errorSentence(e);
    }
  }

  async function check() {
    checking = true;
    try {
      update = await api.updateCheck();
    } catch (e) {
      if (!app.handle(e)) error = errorSentence(e);
    } finally {
      checking = false;
    }
  }

  // ENK-UPD-008: a device not installed from a published image looks for updates only once
  // the owner says so; the first check follows in the same request.
  async function enableChecks() {
    checking = true;
    try {
      update = await api.updateEnable();
    } catch (e) {
      if (!app.handle(e)) error = errorSentence(e);
    } finally {
      checking = false;
    }
  }

  async function changePassword(e: Event) {
    e.preventDefault();
    passwordError = '';
    try {
      await api.changePassword(current, next);
      passwordDone = true;
      setToken(null);
      app.loggedOut = true;
    } catch (err) {
      if (err instanceof ApiError && err.detail.code === 'invalid' && err.detail.fields?.[0]) {
        passwordError = fieldSentence(err.detail.fields[0].code);
      } else if (!app.handle(err)) {
        passwordError = errorSentence(err);
      }
    }
  }

  async function restart() {
    try {
      await api.restart();
      restarted = true;
      setToken(null);
      setTimeout(() => {
        app.loggedOut = true;
        app.status = null;
      }, 3_000);
    } catch (e) {
      if (!app.handle(e)) error = errorSentence(e);
    }
  }

  async function logout() {
    try {
      await api.logout();
    } catch {
      // the session may already have ended
    }
    setToken(null);
    app.loggedOut = true;
    app.status = null;
  }

  function uptime(seconds: number): string {
    return i18n.t('details.uptime.value', { h: Math.floor(seconds / 3600), m: Math.floor((seconds % 3600) / 60) });
  }

  function checkSentence(u: Update): string {
    if (!u.last_check) return '';
    switch (u.last_check.result) {
      case 'ok':
        return u.available ? '' : i18n.t('update.uptodate');
      case 'unreachable':
        return i18n.t('update.unreachable');
      case 'stale_index':
        return i18n.t('update.stale');
      default:
        return i18n.t('update.bad');
    }
  }
</script>

{#if device && update}
  <p class="muted">{i18n.t('status.version', { v: device.version.enkrnett, b: device.version.baseline })}</p>

  <section class="card">
    <h2>{i18n.t('details.update')}</h2>
    {#if !update.checks_enabled}
      <p>{i18n.t('update.disabled')}</p>
      <button onclick={enableChecks} disabled={checking}>{checking ? i18n.t('update.checking') : i18n.t('update.enable')}</button>
    {:else}
      {#if update.state === 'downloading'}<p>{i18n.t('update.downloading')}</p>
      {:else if update.state === 'verifying'}<p>{i18n.t('update.verifying')}</p>
      {:else if update.state === 'installing'}<p>{i18n.t('update.installing')}</p>
      {/if}
      {#if update.last_result?.result === 'done'}
        <p>{i18n.t('update.done', { v: update.installed.version })}</p>
      {:else if update.last_result?.result === 'failed'}
        <p class="field-error">{i18n.t(`update.failed.${update.last_result.reason ?? 'download_failed'}`)}</p>
      {/if}
      {#if update.available}
        <p>{i18n.t('update.available', { v: update.available.version, m: Math.max(1, Math.round(update.available.install_seconds / 60)) })}</p>
        {#if update.available.changes_baseline}<p>{i18n.t('update.changes_baseline')}</p>{/if}
      {:else}
        <p>{checkSentence(update)}</p>
      {/if}
      {#if update.last_check}
        <p class="field-note">{i18n.t('update.last_checked', { s: update.last_check.checked_ago })}</p>
      {/if}
      <button onclick={check} disabled={checking || update.state !== 'idle'}>{checking ? i18n.t('update.checking') : i18n.t('update.check')}</button>
    {/if}
  </section>

  <section class="card">
    <div class="row"><span>{i18n.t('details.model')}</span><span>{device.model} ({device.revision})</span></div>
    <div class="row"><span>{i18n.t('details.id')}</span><span class="mono">{device.device_id}</span></div>
    <div class="row"><span>{i18n.t('details.uptime')}</span><span>{uptime(device.uptime_seconds)}</span></div>
    <div class="row"><span>{i18n.t('details.memory')}</span><span>{i18n.t('details.kib', { n: device.memory.available_kib })}</span></div>
    <div class="row"><span>{i18n.t('details.storage')}</span><span>{i18n.t('details.kib', { n: device.storage.overlay_free_kib })}</span></div>
    <p class="muted">{i18n.t(`details.installed_as.${device.installed_as}`)}</p>
  </section>

  <section class="card">
    <h2>{i18n.t('details.hardening')}</h2>
    <ul>
      {#each device.hardening as item (item.id)}
        <li>{item.id}: {item.met ? i18n.t('details.met') : i18n.t('details.not_met')}</li>
      {/each}
    </ul>
  </section>

  <section class="card">
    <h2>{i18n.t('password.change.title')}</h2>
    {#if passwordDone}
      <p class="notice-bar" role="status">{i18n.t('password.changed')}</p>
    {:else if !changingPassword}
      <button onclick={() => (changingPassword = true)}>{i18n.t('password.change.title')}</button>
    {:else}
      <p>{i18n.t('password.change.lead')}</p>
      <form onsubmit={changePassword}>
        <PasswordField id="current-password" label={i18n.t('password.current')} bind:value={current} autocomplete="current-password" />
        <PasswordField id="new-password" label={i18n.t('password.new')} bind:value={next} autocomplete="new-password" note={i18n.t('password.rules')} error={passwordError} />
        <button class="primary" type="submit" disabled={current === '' || next === ''}>{i18n.t('password.change.button')}</button>
        <button class="link" type="button" onclick={() => (changingPassword = false)}>{i18n.t('common.cancel')}</button>
      </form>
    {/if}
  </section>

  <section class="card">
    <h2>{i18n.t('restart.title')}</h2>
    {#if restarted}
      <p class="notice-bar" role="status">{i18n.t('restart.after')}</p>
    {:else if !restarting}
      <button onclick={() => (restarting = true)}>{i18n.t('restart.button')}</button>
    {:else}
      <p>{i18n.t('restart.lead')}</p>
      <button class="primary" onclick={restart}>{i18n.t('restart.button')}</button>
      <button class="link" onclick={() => (restarting = false)}>{i18n.t('common.cancel')}</button>
    {/if}
  </section>

  {#if error}<p class="field-error" role="alert">{error}</p>{/if}
  <button class="link" onclick={logout}>{i18n.t('details.logout')}</button>
  <p class="field-note">{i18n.t('channel.not_secure')}</p>
{:else}
  <p class="muted">{error || i18n.t('common.loading')}</p>
{/if}
