<script lang="ts">
  import { app } from './lib/app.svelte';
  import { i18n } from './lib/i18n/index.svelte';
  import Header from './components/Header.svelte';
  import Setup from './screens/Setup.svelte';
  import Join from './screens/Join.svelte';
  import Login from './screens/Login.svelte';
  import Main from './screens/Main.svelte';

  // The screen follows the device's state (chapter 4 §1) and whether a session exists.
  $effect(() => {
    void app.refreshGreeting();
  });

  // Without a greeting yet, retry every 3 seconds (the "not connected" path, ENK-UI-019).
  $effect(() => {
    if (app.greeting) return;
    const timer = setInterval(() => void app.refreshGreeting(), 3_000);
    return () => clearInterval(timer);
  });

  const view = $derived.by(() => {
    const g = app.greeting;
    if (!g) return app.notConnected ? 'notconnected' : 'loading';
    switch (g.state) {
      case 'SETUP':
        return 'setup';
      case 'PENDING':
        return app.proposal ? 'join' : 'login';
      case 'CLAIMED':
        return app.loggedIn ? 'main' : 'login';
      default:
        return 'unclaimed';
    }
  });
</script>

<Header />
{#if view === 'loading'}
  <main><p class="muted">{i18n.t('common.loading')}</p></main>
{:else if view === 'notconnected'}
  <main><p class="notice-bar" role="status">{i18n.t('notconnected.setup')}</p></main>
{:else if view === 'setup'}
  <Setup />
{:else if view === 'join'}
  <Join />
{:else if view === 'login'}
  <Login />
{:else if view === 'main'}
  <Main />
{:else}
  <main><p class="muted">{i18n.t('common.loading')}</p></main>
{/if}
