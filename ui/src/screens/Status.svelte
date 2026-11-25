<script lang="ts">
  import { api } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { i18n } from '../lib/i18n/index.svelte';
  import ChangeProgress from '../components/ChangeProgress.svelte';
  import ConnectionLine from '../components/ConnectionLine.svelte';

  // S-9 (chapter 10 §5.1): what the status screen shows, with no further tap.
  const s = $derived(app.status);
  let busy = $state(false);

  function guestLabel(up: boolean, enabled: boolean): string {
    if (!enabled) return i18n.t('status.off');
    return up ? i18n.t('status.running') : i18n.t('status.stopped');
  }

  async function restore() {
    busy = true;
    try {
      await api.restoreSettings();
      await app.loadStatus();
    } catch (e) {
      app.handle(e);
    } finally {
      busy = false;
    }
  }
</script>

{#if s}
  <section class="card" aria-labelledby="guest-heading">
    <div class="row">
      <h2 id="guest-heading">
        {#if s.settings.guest.enabled}
          {i18n.t('status.guest', { name: s.guest.name ?? s.settings.guest.name })}
        {:else}
          {i18n.t('status.guest_off')}
        {/if}
      </h2>
      <span><span class="dot" class:running={s.guest.up && s.settings.guest.enabled} aria-hidden="true"></span>{guestLabel(s.guest.up, s.settings.guest.enabled)}</span>
    </div>
    {#if s.settings.guest.enabled}
      <p>{(s.guest.mode ?? s.settings.guest.mode) === 'open_to_all' ? i18n.t('mode.open') : i18n.t('mode.encrypted')}</p>
      {#if s.guest.clients_encrypted + s.guest.clients_unencrypted === 0}
        <p>{i18n.t('status.counts.none')}</p>
      {:else if (s.guest.mode ?? s.settings.guest.mode) === 'open_to_all'}
        <p>{i18n.n('status.counts.open', s.guest.clients_encrypted + s.guest.clients_unencrypted, { e: s.guest.clients_encrypted, u: s.guest.clients_unencrypted })}</p>
      {:else}
        <p>{i18n.n('status.counts.encrypted', s.guest.clients_encrypted)}</p>
      {/if}
    {/if}
    <button class="link" onclick={() => app.navigate('protects')}>{i18n.t('status.protects')} ›</button>
  </section>

  <ChangeProgress change={s.change} />

  {#if s.drift}
    <div class="notice-bar" role="status">
      <p>{i18n.t('status.drift')}</p>
      <button onclick={restore} disabled={busy}>{i18n.t('settings.restore')}</button>
    </div>
  {/if}

  <section class="card">
    <div class="row">
      <h2>{i18n.t('status.management', { name: s.management.name ?? s.settings.management.name })}</h2>
      <span><span class="dot" class:running={s.management.up} aria-hidden="true"></span>{s.management.up ? i18n.t('status.running') : i18n.t('status.stopped')}</span>
    </div>
    <p class="muted">{i18n.n('status.management_devices', s.management.clients)}</p>
    <div class="row">
      <span>{i18n.t('status.internet')}</span>
      <span><span class="dot" class:running={s.uplink.up} aria-hidden="true"></span>{s.uplink.up ? i18n.t('status.internet.connected') : i18n.t('status.internet.not_connected')}</span>
    </div>
  </section>

  {#if app.greeting}
    <ConnectionLine connection={app.greeting.connection} where="management" />
  {/if}

  {#if !s.hardening_met}
    <p class="notice-bar" role="status">{i18n.t('status.hardening')}</p>
  {/if}

  <p class="muted">
    {i18n.t('status.version', { v: s.version.enkrnett, b: s.version.baseline })}
    ·
    {#if s.update.available}
      {i18n.t('update.available_short', { v: s.update.available.version })}
    {:else if s.update.last_check?.result === 'ok'}
      {i18n.t('update.uptodate')}
    {:else}
      <button class="link" onclick={() => app.navigate('details')}>{i18n.t('update.check')}</button>
    {/if}
  </p>
{:else}
  <p class="muted">{i18n.t('common.loading')}</p>
{/if}
