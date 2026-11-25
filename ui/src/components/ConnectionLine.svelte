<script lang="ts">
  import { i18n } from '../lib/i18n/index.svelte';
  import type { Connection } from '../lib/types';

  // The connection line (chapter 10 §5.4). On the management network only `wpa2` and unknown
  // say anything; `wpa3` is silent, because a reassurance badge is what ENK-OWE-017 forbids.
  let { connection, where }: { connection: Connection; where: 'setup' | 'management' } = $props();

  const key = $derived.by(() => {
    if (where === 'setup') {
      if (connection.security === 'owe') return 'connection.owe';
      if (connection.security === 'none') return 'connection.none';
      return 'connection.unknown';
    }
    if (connection.security === 'wpa2') return 'connection.wpa2';
    if (connection.security === null) return 'connection.unknown';
    return null;
  });
</script>

{#if key}
  <p class="notice-bar" data-connection={connection.security ?? 'unknown'}>{i18n.t(key)}</p>
{/if}
