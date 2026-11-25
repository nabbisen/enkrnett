<script lang="ts">
  import { api } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { errorSentence } from '../lib/errors';
  import { i18n } from '../lib/i18n/index.svelte';
  import PasswordField from '../components/PasswordField.svelte';

  // S-13: the consequence in one paragraph, the owner password, one button. Never on the same
  // screen as Restart (ENK-WORD-003).
  let password = $state('');
  let busy = $state(false);
  let error = $state('');
  let done = $state(false);

  async function erase(e: Event) {
    e.preventDefault();
    busy = true;
    error = '';
    try {
      await api.erase(password);
      done = true;
      password = '';
      // The device erases and restarts; the greeting will say SETUP when it is back.
      const timer = setInterval(async () => {
        const g = await app.refreshGreeting();
        if (g && g.state !== 'CLAIMED') {
          clearInterval(timer);
          app.loggedOut = true;
          app.status = null;
          app.navigate('status');
        }
      }, 2_000);
    } catch (err) {
      if (!app.handle(err)) error = errorSentence(err);
    } finally {
      busy = false;
    }
  }
</script>

<h1>{i18n.t('erase.title')}</h1>
{#if done}
  <p class="notice-bar" role="status">{i18n.t('erase.after')}</p>
{:else}
  <p class="lead">{i18n.t('erase.lead')}</p>
  <form onsubmit={erase}>
    <PasswordField id="erase-password" label={i18n.t('password.label')} bind:value={password} autocomplete="current-password" {error} />
    <button class="primary" type="submit" disabled={busy || password === ''}>{i18n.t('erase.button')}</button>
  </form>
  <button class="link" onclick={() => app.navigate('settings')}>{i18n.t('common.back')}</button>
{/if}
