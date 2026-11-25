<script lang="ts">
  import { api } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { errorSentence } from '../lib/errors';
  import { i18n } from '../lib/i18n/index.svelte';
  import type { Change, ManagementKey } from '../lib/types';
  import { addressUrl, wifiQrPayload } from '../lib/wifi';
  import ChangeProgress from '../components/ChangeProgress.svelte';
  import Qr from '../components/Qr.svelte';

  // S-12: name, password on request, Wi-Fi QR code, address; replacing the password on its own
  // confirmation step (ENK-UI-013).
  let key = $state<ManagementKey | null>(null);
  let shown = $state(false);
  let confirming = $state(false);
  let busy = $state(false);
  let error = $state('');
  let result = $state<(Change & { management_key: ManagementKey }) | null>(null);

  $effect(() => {
    void load();
  });

  async function load() {
    try {
      key = await api.managementKey();
    } catch (e) {
      if (!app.handle(e)) error = errorSentence(e);
    }
  }

  async function replace() {
    busy = true;
    error = '';
    try {
      result = await api.replaceManagementKey();
      key = result.management_key;
      shown = true;
      confirming = false;
      await app.loadStatus();
    } catch (e) {
      if (!app.handle(e)) error = errorSentence(e);
    } finally {
      busy = false;
    }
  }

  const address = $derived(app.status?.management.address ?? '');
</script>

{#if key}
  <div class="card">
    <h2>{i18n.t('mgmt.name')}</h2>
    <p class="mono">{key.name}</p>
    <h2>{i18n.t('mgmt.password')}</h2>
    {#if shown}
      <p class="mono">{key.key}</p>
      <Qr text={wifiQrPayload(key.name, key.key)} label={i18n.t('keep.qr_wifi')} />
    {:else}
      <button onclick={() => (shown = true)}>{i18n.t('mgmt.show_password')}</button>
    {/if}
  </div>
  {#if address}
    <div class="card">
      <h2>{i18n.t('mgmt.address')}</h2>
      <p class="mono">{addressUrl(address)}</p>
      <Qr text={addressUrl(address)} label={i18n.t('keep.qr_address')} />
    </div>
  {/if}
  {#if result}
    <h2>{i18n.t('mgmt.new_password')}</h2>
    <ChangeProgress change={result} managementKey={result.management_key} />
  {/if}
  {#if !confirming}
    <button onclick={() => (confirming = true)}>{i18n.t('mgmt.replace')}</button>
  {:else}
    <div class="card">
      <h2>{i18n.t('mgmt.replace')}</h2>
      <p>{i18n.t('mgmt.replace.lead')}</p>
      {#if error}<p class="field-error" role="alert">{error}</p>{/if}
      <button class="primary" onclick={replace} disabled={busy}>{i18n.t('mgmt.replace.button')}</button>
      <button class="link" onclick={() => (confirming = false)}>{i18n.t('common.cancel')}</button>
    </div>
  {/if}
{:else}
  <p class="muted">{error || i18n.t('common.loading')}</p>
{/if}
