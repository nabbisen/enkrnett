<script lang="ts">
  import { api, ApiError } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { errorSentence, fieldSentence } from '../lib/errors';
  import { i18n } from '../lib/i18n/index.svelte';
  import { byteLength, isSendable } from '../lib/names';
  import type { Change, GuestMode } from '../lib/types';
  import ChangeProgress from '../components/ChangeProgress.svelte';
  import NameField from '../components/NameField.svelte';

  // S-10. One Save; the mode switch shows the count that will lose the network (ENK-OWE-011);
  // a management name change says that every phone must join again (needs confirmation).
  const s = $derived(app.status);

  let version = $state(0);
  let guestName = $state('');
  let mode = $state<GuestMode>('open_to_all');
  let enabled = $state(true);
  let managementName = $state('');
  let loaded = $state(false);
  let busy = $state(false);
  let error = $state('');
  let fieldErrors = $state<Record<string, string>>({});
  let result = $state<Change | null>(null);

  $effect(() => {
    if (s && (!loaded || version !== s.settings.version)) {
      version = s.settings.version;
      guestName = s.settings.guest.name;
      mode = s.settings.guest.mode;
      enabled = s.settings.guest.enabled;
      managementName = s.settings.management.name;
      loaded = true;
    }
  });

  const maxBytes = $derived(
    mode === 'open_to_all' ? (s?.settings.limits.guest_name_max_bytes.open_to_all ?? 29) : (s?.settings.limits.guest_name_max_bytes.encrypted_only ?? 32),
  );
  const switchingToEncrypted = $derived(s !== null && s.settings.guest.mode === 'open_to_all' && mode === 'encrypted_only');
  const switchingToOpen = $derived(s !== null && s.settings.guest.mode === 'encrypted_only' && mode === 'open_to_all');
  const managementChanges = $derived(s !== null && managementName !== s.settings.management.name);
  const overOpen = $derived(mode === 'open_to_all' && byteLength(guestName) > maxBytes);
  const canSave = $derived(
    s !== null && isSendable(guestName, maxBytes) && isSendable(managementName, s.settings.limits.management_name_max_bytes) && !busy,
  );

  async function save(e: Event) {
    e.preventDefault();
    if (!s) return;
    busy = true;
    error = '';
    fieldErrors = {};
    result = null;
    try {
      const change = await api.saveSettings({
        version,
        guest: { name: guestName, mode, enabled },
        management: { name: managementName },
      });
      result = change;
      await app.loadStatus();
    } catch (err) {
      if (err instanceof ApiError && err.detail.code === 'invalid' && err.detail.fields) {
        const map: Record<string, string> = {};
        for (const f of err.detail.fields) map[f.field] = fieldSentence(f.code);
        fieldErrors = map;
        error = i18n.t('error.invalid');
      } else if (err instanceof ApiError && err.detail.code === 'version_mismatch') {
        error = errorSentence(err);
        loaded = false;
        await app.loadStatus();
      } else if (!app.handle(err)) {
        error = errorSentence(err);
      }
    } finally {
      busy = false;
    }
  }
</script>

{#if s}
  <form onsubmit={save}>
    <h2>{i18n.t('guest.title')}</h2>
    <label class="choice">
      <input type="checkbox" bind:checked={enabled} />
      <strong>{i18n.t('settings.guest_on')}</strong>
    </label>
    <NameField
      id="settings-guest-name"
      label={i18n.t('guest.name_label')}
      note={i18n.t('names.before', { max: maxBytes })}
      bind:value={guestName}
      {maxBytes}
      error={overOpen ? i18n.t('names.over_open', { k: byteLength(guestName) - maxBytes }) : (fieldErrors['guest.name'] ?? '')}
    />
    <fieldset style="border:0;padding:0;margin:1rem 0 0">
      <legend style="font-weight:600;margin-top:1rem">{i18n.t('mode.label')}</legend>
      <label class="choice">
        <input type="radio" name="settings-mode" value="open_to_all" bind:group={mode} />
        <strong>{i18n.t('mode.open')}</strong>
        <span class="muted">{i18n.t('mode.open.line')}</span>
      </label>
      <label class="choice">
        <input type="radio" name="settings-mode" value="encrypted_only" bind:group={mode} />
        <strong>{i18n.t('mode.encrypted')}</strong>
        <span class="muted">{i18n.t('mode.encrypted.line')}</span>
      </label>
    </fieldset>
    {#if switchingToEncrypted}
      <p class="notice-bar" role="status">{i18n.n('settings.mode_warning', s.guest.clients_unencrypted, { u: s.guest.clients_unencrypted })}</p>
    {/if}
    {#if switchingToOpen}
      <p class="notice-bar" role="status">{i18n.t('settings.mode_note')}</p>
    {/if}

    <h2>{i18n.t('mgmt.title')}</h2>
    <NameField
      id="settings-management-name"
      label={i18n.t('settings.management_name_label')}
      note={i18n.t('names.management')}
      bind:value={managementName}
      maxBytes={s.settings.limits.management_name_max_bytes}
      error={fieldErrors['management.name'] ?? ''}
    />
    {#if managementChanges}
      <p class="notice-bar" role="status">{i18n.t('settings.needs_join')}</p>
    {/if}

    <p class="muted">{i18n.t('settings.country', { c: s.settings.country })}</p>
    {#if error}<p class="field-error" role="alert">{error}</p>{/if}
    <button class="primary" type="submit" disabled={!canSave}>{i18n.t('common.save')}</button>
  </form>
  {#if result}
    <ChangeProgress change={result} />
  {/if}
  <p style="margin-top:2rem">
    <button class="link" onclick={() => app.navigate('erase')}>{i18n.t('settings.erase_link')}</button>
  </p>
{:else}
  <p class="muted">{i18n.t('common.loading')}</p>
{/if}
