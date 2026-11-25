<script lang="ts">
  import { i18n } from '../lib/i18n/index.svelte';

  // Password fields with a show/hide toggle and the browser's hints so that a phone's password
  // manager can offer to save the owner password (chapter 10 §5.3).
  let {
    id,
    label,
    value = $bindable(''),
    autocomplete,
    note = '',
    error = '',
  }: {
    id: string;
    label: string;
    value: string;
    autocomplete: 'current-password' | 'new-password';
    note?: string;
    error?: string;
  } = $props();

  let shown = $state(false);
</script>

<label for={id}>{label}</label>
{#if note}
  <p class="field-note" id="{id}-note">{note}</p>
{/if}
<div class="row">
  <input
    {id}
    type={shown ? 'text' : 'password'}
    {autocomplete}
    autocapitalize="none"
    spellcheck="false"
    bind:value
    aria-describedby={note ? `${id}-note` : undefined}
    aria-invalid={error ? 'true' : undefined}
  />
  <button type="button" onclick={() => (shown = !shown)} aria-pressed={shown}>
    {shown ? i18n.t('common.hide') : i18n.t('common.show')}
  </button>
</div>
{#if error}
  <p class="field-error" role="alert">{error}</p>
{/if}
