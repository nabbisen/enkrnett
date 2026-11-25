<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { i18n } from '../lib/i18n/index.svelte';

  // S-15: one text, used on the screen, in the printable notice, and in the documentation
  // (ENK-UI-016; chapter 7 §5). `print` renders the notice alone.
  let { print = false }: { print?: boolean } = $props();
  const name = $derived(app.status?.guest.name ?? app.status?.settings.guest.name ?? '');
</script>

<article class={print ? 'notice-print' : ''}>
  {#if print}
    <h1>{i18n.t('notice.title', { name })}</h1>
  {:else}
    <h1>{i18n.t('protects.title')}</h1>
  {/if}
  <p><strong>{i18n.t('notice.p1')}</strong></p>
  <p>{i18n.t('notice.p2')}</p>
  <p>{i18n.t('notice.p3')}</p>
  <p>{i18n.t('notice.p4')}</p>
  {#if !print}
    <p class="muted">{i18n.t('compat.text')}</p>
    <button class="no-print" onclick={() => { app.navigate('notice'); setTimeout(() => window.print(), 300); }}>{i18n.t('notice.print')}</button>
  {/if}
</article>
