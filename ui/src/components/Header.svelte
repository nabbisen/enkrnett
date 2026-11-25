<script lang="ts">
  import { app } from '../lib/app.svelte';
  import { LANGUAGES, LANGUAGE_NAMES, i18n, type Lang } from '../lib/i18n/index.svelte';

  // The header on every screen: the product name, the language switch (ENK-UI-010), the
  // development banner when the greeting says so (ENK-UI-027).
  function onLanguage(e: Event) {
    i18n.set((e.currentTarget as HTMLSelectElement).value as Lang);
  }
</script>

{#if app.greeting?.development}
  <div class="banner" role="note">{i18n.t('banner.development')}</div>
{/if}
<header class="bar">
  <span class="brand">{i18n.t('app.name')}</span>
  <label class="sr-only" for="language">{i18n.t('common.language')}</label>
  <select id="language" value={i18n.lang} onchange={onLanguage} aria-label={i18n.t('common.language')}>
    {#each LANGUAGES as lang (lang)}
      <option value={lang}>{LANGUAGE_NAMES[lang]}</option>
    {/each}
  </select>
</header>
