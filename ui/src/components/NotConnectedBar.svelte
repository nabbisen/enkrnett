<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { i18n } from '../lib/i18n/index.svelte';

  // ENK-UI-019: a failed poll shows this sentence and the screen continues by itself.
  let { variant = 'management' }: { variant?: 'management' | 'setup' } = $props();
</script>

{#if app.notConnected}
  <div class="notice-bar" role="status" aria-live="polite">
    {#if variant === 'setup'}
      {i18n.t('notconnected.setup')}
    {:else}
      {i18n.t('notconnected.management', { name: app.managementName ?? app.proposal?.management.name ?? '' })}
    {/if}
  </div>
{/if}
