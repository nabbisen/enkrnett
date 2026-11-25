<script lang="ts">
  import { Countdown } from '../lib/countdown.svelte';
  import { changeSentence } from '../lib/errors';
  import { i18n } from '../lib/i18n/index.svelte';
  import type { Change, ManagementKey } from '../lib/types';
  import Qr from './Qr.svelte';
  import { wifiQrPayload } from '../lib/wifi';

  // S-11: the sentence for the change's state (chapter 10 §5.5); while the owner must join the
  // new management Wi-Fi, the new name and password, a Wi-Fi QR code, and the countdown.
  let { change, managementKey = null }: { change: Change; managementKey?: ManagementKey | null } = $props();

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
