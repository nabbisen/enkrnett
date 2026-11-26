<script lang="ts">
  import { api } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { Countdown } from '../lib/countdown.svelte';
  import { changeSentence } from '../lib/errors';
  import { i18n } from '../lib/i18n/index.svelte';
  import type { Change, ManagementKey } from '../lib/types';
  import Qr from './Qr.svelte';
  import { wifiQrPayload } from '../lib/wifi';

  // S-11: the sentence for the change's state (chapter 10 §5.5); while the owner must join the
  // new management Wi-Fi, the new name and password, a Wi-Fi QR code, and the countdown.
  // While the change runs, `change` is polled every second (chapter 9 §6); the poll that finds
  // it settled or undone also refreshes the status. A poll that fails (the phone is between
  // networks) is simply retried.
  let { change: given, managementKey = null }: { change: Change; managementKey?: ManagementKey | null } = $props();
  // What the polls found, until the parent hands in a new change.
  let live = $state<Change | null>(null);
  const change = $derived(live ?? given);

  const inProgress = (c: Change) => c.state === 'validating' || c.state === 'applying' || c.state === 'waiting_confirmation';

  $effect(() => {
    void given;
    live = null;
  });

  $effect(() => {
    if (!inProgress(change)) return;
    const id = change.id;
    const timer = setInterval(async () => {
      try {
        const latest = await api.change();
        if (latest.id !== id) return;
        live = latest;
        if (!inProgress(latest)) await app.loadStatus();
      } catch {
        // between networks or the device restarting the Wi-Fi: try again next second
      }
    }, 1000);
    return () => clearInterval(timer);
  });

  const countdown = new Countdown();
  $effect(() => {
    if (change.confirm_in !== null) countdown.set(change.confirm_in);
  });
  $effect(() => () => countdown.stop());
</script>

{#if change.state !== 'none'}
  <div class="notice-bar" role="status" aria-live="polite">
    <p>{changeSentence(change.state, change.reason, change.detail)}</p>
    {#if change.state === 'waiting_confirmation'}
      {#if managementKey}
        <p><strong>{managementKey.name}</strong></p>
        <p class="mono">{managementKey.key}</p>
        <Qr text={wifiQrPayload(managementKey.name, managementKey.key)} label={i18n.t('keep.qr_wifi')} />
      {/if}
      <p class="countdown">{i18n.t('common.countdown', { m: countdown.text.split(':')[0] ?? '0', ss: countdown.text.split(':')[1] ?? '00' })}</p>
    {/if}
  </div>
{/if}
