<script lang="ts">
  import { i18n } from '../lib/i18n/index.svelte';
  import { byteLength, normalise, roomLeft } from '../lib/names';

  // A network name (ENK-UI-015): normalised to NFC, counted in bytes against the limit from the
  // API, the room left shown while typing, the limit and its reason stated before typing.
  let {
    id,
    label,
    note,
    value = $bindable(''),
    maxBytes,
    error = '',
  }: { id: string; label: string; note: string; value: string; maxBytes: number; error?: string } = $props();

  const left = $derived(roomLeft(value, maxBytes));
  const over = $derived(byteLength(value) > maxBytes);

  function onInput(e: Event) {
    value = normalise((e.currentTarget as HTMLInputElement).value);
  }
</script>

<label for={id}>{label}</label>
<p class="field-note" id="{id}-note">{note}</p>
<input
  {id}
  type="text"
  autocomplete="off"
  autocapitalize="none"
  spellcheck="false"
  {value}
  oninput={onInput}
  aria-describedby="{id}-note {id}-left"
  aria-invalid={over || error !== '' ? 'true' : undefined}
/>
<p class="field-note" id="{id}-left" aria-live="polite">{i18n.t('names.left', { n: left })}</p>
{#if error}
  <p class="field-error" role="alert">{error}</p>
{/if}
