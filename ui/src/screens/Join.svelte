<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { Countdown } from '../lib/countdown.svelte';
  import { i18n } from '../lib/i18n/index.svelte';
  import { addressUrl, wifiQrPayload } from '../lib/wifi';
  import Qr from '../components/Qr.svelte';
  import Login from './Login.svelte';

  // S-7: three numbered steps, the 10-minute countdown, and the login form as soon as the
  // device answers from the management network. The greeting is polled every 3 seconds; a
  // failed poll is the expected path while the phone changes network (ENK-UI-019).
  const proposal = $derived(app.proposal);
  const countdown = new Countdown();

  $effect(() => {
    if (proposal?.final_in != null) countdown.set(proposal.final_in);
  });

  $effect(() => {
    const timer = setInterval(() => void app.refreshGreeting(), 3_000);
    return () => {
      clearInterval(timer);
      countdown.stop();
    };
  });

  const reachable = $derived(app.greeting?.state === 'PENDING' && !app.notConnected);
</script>

<main>
  {#if proposal}
    <h1>{i18n.t('join.title')}</h1>
    <p class="lead">{i18n.t('join.lead')}</p>
    <div class="card">
      <h2>{i18n.t('keep.management_name')}</h2>
      <p class="mono">{proposal.management.name}</p>
      <h2>{i18n.t('keep.management_password')}</h2>
      <p class="mono">{proposal.management.key}</p>
      <Qr text={wifiQrPayload(proposal.management.name, proposal.management.key)} label={i18n.t('keep.qr_wifi')} />
    </div>
    <ol class="steps">
      <li>{i18n.t('join.step1')}</li>
      <li>{i18n.t('join.step2', { name: proposal.management.name })}</li>
      <li>{i18n.t('join.step3', { address: addressUrl(proposal.address) })}</li>
    </ol>
    <p>
      {i18n.t('join.time')}
      <span class="countdown">{i18n.t('common.countdown', { m: countdown.text.split(':')[0] ?? '0', ss: countdown.text.split(':')[1] ?? '00' })}</span>
    </p>
    {#if app.notConnected}
      <p class="notice-bar" role="status">{i18n.t('notconnected.management', { name: proposal.management.name })}</p>
    {/if}
  {/if}
  {#if reachable}
    <Login embedded={true} />
  {/if}
</main>
