<script lang="ts">
  import { app, type Route } from '../lib/app.svelte';
  import { i18n } from '../lib/i18n/index.svelte';
  import NotConnectedBar from '../components/NotConnectedBar.svelte';
  import Status from './Status.svelte';
  import Settings from './Settings.svelte';
  import Management from './Management.svelte';
  import Erase from './Erase.svelte';
  import Details from './Details.svelte';
  import Protects from './Protects.svelte';

  // The claimed device, logged in: three menu entries; Back always returns to Status
  // (chapter 10 §7). Status is polled every 5 seconds (chapter 9 §9).
  const menu: { route: Route; key: string }[] = [
    { route: 'status', key: 'status.title' },
    { route: 'settings', key: 'settings.title' },
    { route: 'management', key: 'mgmt.title' },
    { route: 'details', key: 'details.title' },
  ];

  $effect(() => {
    void app.loadStatus();
    const timer = setInterval(() => void app.loadStatus(), 5_000);
    return () => clearInterval(timer);
  });

  const titles: Record<Route, string> = {
    status: 'status.title',
    settings: 'settings.title',
    management: 'mgmt.title',
    erase: 'erase.title',
    details: 'details.title',
    protects: 'protects.title',
    notice: 'notice.title',
  };
</script>

<main>
  <NotConnectedBar />
  {#if app.route !== 'notice'}
    <nav class="menu" aria-label={i18n.t('common.menu')}>
      {#each menu as item (item.route)}
        <button onclick={() => app.navigate(item.route)} aria-current={app.route === item.route ? 'page' : undefined}>{i18n.t(item.key)}</button>
      {/each}
    </nav>
    {#if app.route !== 'status' && app.route !== 'erase' && app.route !== 'protects'}
      <h1>{i18n.t(titles[app.route])}</h1>
    {/if}
  {/if}

  {#if app.route === 'status'}
    <h1>{i18n.t('status.title')}</h1>
    <Status />
  {:else if app.route === 'settings'}
    <Settings />
  {:else if app.route === 'management'}
    <Management />
  {:else if app.route === 'erase'}
    <Erase />
  {:else if app.route === 'details'}
    <Details />
  {:else if app.route === 'protects'}
    <Protects />
    <button class="link" onclick={() => app.navigate('status')}>{i18n.t('common.back')}</button>
  {:else if app.route === 'notice'}
    <Protects print={true} />
    <button class="link no-print" onclick={() => app.navigate('protects')}>{i18n.t('common.back')}</button>
  {/if}
</main>
