<script lang="ts">
  import { api, ApiError } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { Countdown } from '../lib/countdown.svelte';
  import { errorSentence } from '../lib/errors';
  import { i18n } from '../lib/i18n/index.svelte';
  import ConnectionLine from '../components/ConnectionLine.svelte';
  import PasswordField from '../components/PasswordField.svelte';
  import NotConnectedBar from '../components/NotConnectedBar.svelte';

  // S-8. In PENDING the login also makes the claim final (RFC 004 step 10).
  let { embedded = false }: { embedded?: boolean } = $props();

  let password = $state('');
  let error = $state('');
  let busy = $state(false);
  let retry = $state<number | null>(null);
  const countdown = new Countdown();
  $effect(() => () => countdown.stop());

  async function login(e: Event) {
    e.preventDefault();
    busy = true;
    error = '';
    try {
      const session = await api.login(password);
      app.loginSucceeded(session.token);
      app.proposal = null;
      password = '';
      await app.refreshGreeting();
      await app.loadStatus();
      app.navigate('status');
    } catch (err) {
      if (err instanceof ApiError && err.detail.code === 'try_later') {
        retry = err.detail.retry_in ?? 30;
        countdown.set(retry);
      }
      if (!app.handle(err)) error = errorSentence(err);
    } finally {
      busy = false;
    }
  }
</script>

<svelte:element this={embedded ? 'section' : 'main'}>
  {#if !embedded}
    <NotConnectedBar />
  {/if}
  <h1>{i18n.t('login.title')}</h1>
  {#if app.greeting && !embedded}
    <ConnectionLine connection={app.greeting.connection} where="management" />
  {/if}
  <form onsubmit={login}>
    <PasswordField id="login-password" label={i18n.t('password.label')} bind:value={password} autocomplete="current-password" {error} />
    {#if retry !== null && countdown.remaining > 0}
      <p class="muted">{i18n.t('common.countdown', { m: countdown.text.split(':')[0] ?? '0', ss: countdown.text.split(':')[1] ?? '00' })}</p>
    {/if}
    <button class="primary" type="submit" disabled={busy || password === ''}>{i18n.t('login.button')}</button>
  </form>
  <p class="muted">{i18n.t('login.forgot')}</p>
  {#if !embedded}
    <p class="field-note">{i18n.t('channel.not_secure')}</p>
  {/if}
</svelte:element>
