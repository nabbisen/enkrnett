<script lang="ts">
  import { api, ApiError, setToken } from '../lib/api';
  import { app } from '../lib/app.svelte';
  import { errorSentence, fieldSentence } from '../lib/errors';
  import { i18n } from '../lib/i18n/index.svelte';
  import { isSendable } from '../lib/names';
  import type { Claim, FieldError, GuestMode } from '../lib/types';
  import { addressUrl, wifiQrPayload } from '../lib/wifi';
  import ConnectionLine from '../components/ConnectionLine.svelte';
  import NameField from '../components/NameField.svelte';
  import PasswordField from '../components/PasswordField.svelte';
  import Qr from '../components/Qr.svelte';
  import NotConnectedBar from '../components/NotConnectedBar.svelte';

  // S-1 to S-6: one decision per screen (ENK-UI-012). The device persists nothing before the
  // operator presses "Set up" on S-6 (ENK-OWN-007).
  type Step = 'start' | 'identify' | 'country' | 'guest' | 'password' | 'keep';

  let step = $state<Step>('start');
  let claim = $state<Claim | null>(null);
  let error = $state('');
  let busy = $state(false);
  let blinked = $state(false);

  let country = $state('');
  let guestName = $state('');
  let mode = $state<GuestMode>('open_to_all');
  let password = $state('');
  let fieldErrors = $state<Record<string, string>>({});

  const steps = $derived.by((): Step[] => {
    const list: Step[] = ['start'];
    if (!claim || claim.identify_available) list.push('identify');
    if (!claim || claim.country.fixed === null) list.push('country');
    list.push('guest', 'password', 'keep');
    return list;
  });
  const stepIndex = $derived(steps.indexOf(step) + 1);
  const maxBytes = $derived(
    mode === 'open_to_all' ? (claim?.limits.guest_name_max_bytes.open_to_all ?? 29) : (claim?.limits.guest_name_max_bytes.encrypted_only ?? 32),
  );

  function next() {
    const i = steps.indexOf(step);
    const n = steps[i + 1];
    if (n) step = n;
    error = '';
  }

  function back() {
    const i = steps.indexOf(step);
    const p = steps[i - 1];
    if (p) step = p;
    error = '';
  }

  async function start() {
    busy = true;
    error = '';
    try {
      const session = await api.claimStart();
      setToken(session.token);
      claim = await api.claim();
      if (claim.country.fixed) country = claim.country.fixed;
      mode = claim.guest_mode_default;
      next();
    } catch (e) {
      if (!app.handle(e)) error = errorSentence(e);
    } finally {
      busy = false;
    }
  }

  async function blink() {
    try {
      await api.claimIdentify();
      blinked = true;
    } catch (e) {
      if (!app.handle(e)) error = errorSentence(e);
    }
  }

  function applyFieldErrors(fields: FieldError[]) {
    const map: Record<string, string> = {};
    for (const f of fields) {
      if (f.field === 'guest.name' && f.code === 'too_long' && mode === 'open_to_all' && f.limit !== undefined) {
        map[f.field] = i18n.t('names.over_open', { k: Math.max(1, new TextEncoder().encode(guestName).length - f.limit) });
      } else {
        map[f.field] = fieldSentence(f.code);
      }
    }
    fieldErrors = map;
    if (map['country']) step = 'country';
    else if (map['guest.name']) step = 'guest';
    else if (map['owner_password']) step = 'password';
  }

  async function check() {
    busy = true;
    error = '';
    fieldErrors = {};
    try {
      const body: { country?: string; guest: { name: string; mode: GuestMode }; owner_password: string } = {
        guest: { name: guestName, mode },
        owner_password: password,
      };
      if (claim && claim.country.fixed === null) body.country = country;
      app.proposal = await api.claimCheck(body);
      step = 'keep';
    } catch (e) {
      if (e instanceof ApiError && e.detail.code === 'invalid' && e.detail.fields) {
        applyFieldErrors(e.detail.fields);
        error = i18n.t('error.invalid');
      } else if (!app.handle(e)) {
        error = errorSentence(e);
      }
    } finally {
      busy = false;
    }
  }

  async function confirm() {
    busy = true;
    error = '';
    try {
      const result = await api.claimConfirm();
      app.proposal = result;
      setToken(null);
      password = '';
      await app.refreshGreeting();
    } catch (e) {
      if (!app.handle(e)) error = errorSentence(e);
    } finally {
      busy = false;
    }
  }

  // The claim session ends after 5 minutes without a request; a poll of `claim` keeps it alive
  // and refreshes the window's countdown (chapter 9 §9).
  $effect(() => {
    if (!claim) return;
    const timer = setInterval(async () => {
      try {
        claim = await api.claim();
        app.notConnected = false;
      } catch (e) {
        if (e instanceof ApiError && e.detail.code === 'no_session') {
          claim = null;
          step = 'start';
          setToken(null);
        } else {
          app.handle(e);
        }
      }
    }, 30_000);
    return () => clearInterval(timer);
  });
</script>

<main>
  <NotConnectedBar variant="setup" />
  {#if step !== 'start'}
    <p class="muted">{i18n.t('common.step', { n: stepIndex, m: steps.length })}</p>
  {/if}

  {#if step === 'start'}
    <h1>{i18n.t('start.title')}</h1>
    <p class="lead">{i18n.t('start.lead')}</p>
    {#if app.greeting}
      <ConnectionLine connection={app.greeting.connection} where="setup" />
    {/if}
    {#if error}<p class="field-error" role="alert">{error}</p>{/if}
    <button class="primary" onclick={start} disabled={busy}>{i18n.t('start.button')}</button>
  {:else if step === 'identify'}
    <h1>{i18n.t('identify.title')}</h1>
    {#if claim?.identify_available}
      <p class="lead">{i18n.t('identify.lead')}</p>
      <button onclick={blink} disabled={busy}>{i18n.t('identify.blink')}</button>
      {#if error}<p class="field-error" role="alert">{error}</p>{/if}
      <button class="primary" onclick={next} disabled={!blinked}>{i18n.t('identify.saw')}</button>
    {:else}
      <p class="lead">{i18n.t('identify.none', { suffix: app.proposal?.management.name.split('-').pop() ?? '' })}</p>
      <button class="primary" onclick={next}>{i18n.t('common.continue')}</button>
    {/if}
  {:else if step === 'country'}
    <h1>{i18n.t('country.title')}</h1>
    <p class="lead">{i18n.t('country.lead')}</p>
    <label for="country">{i18n.t('country.title')}</label>
    <select id="country" bind:value={country} aria-invalid={fieldErrors['country'] ? 'true' : undefined}>
      <option value="" disabled>—</option>
      {#each claim?.country.choices ?? [] as c (c)}
        <option value={c}>{c}</option>
      {/each}
    </select>
    {#if fieldErrors['country']}<p class="field-error" role="alert">{fieldErrors['country']}</p>{/if}
    <button class="primary" onclick={next} disabled={country === ''}>{i18n.t('common.next')}</button>
    <button class="link" onclick={back}>{i18n.t('common.back')}</button>
  {:else if step === 'guest'}
    <h1>{i18n.t('guest.title')}</h1>
    <NameField
      id="guest-name"
      label={i18n.t('guest.name_label')}
      note={i18n.t('names.before', { max: maxBytes })}
      bind:value={guestName}
      {maxBytes}
      error={fieldErrors['guest.name'] ?? ''}
    />
    <fieldset style="border:0;padding:0;margin:1rem 0 0">
      <legend style="font-weight:600;margin-top:1rem">{i18n.t('mode.label')}</legend>
      <label class="choice">
        <input type="radio" name="mode" value="open_to_all" bind:group={mode} />
        <strong>{i18n.t('mode.open')}</strong>
        <span class="muted">{i18n.t('mode.open.line')}</span>
      </label>
      <label class="choice">
        <input type="radio" name="mode" value="encrypted_only" bind:group={mode} />
        <strong>{i18n.t('mode.encrypted')}</strong>
        <span class="muted">{i18n.t('mode.encrypted.line')}</span>
      </label>
    </fieldset>
    <button class="primary" onclick={next} disabled={!isSendable(guestName, maxBytes)}>{i18n.t('common.next')}</button>
    <button class="link" onclick={back}>{i18n.t('common.back')}</button>
  {:else if step === 'password'}
    <h1>{i18n.t('password.title')}</h1>
    <p class="lead">{i18n.t('password.enter')}</p>
    <PasswordField
      id="owner-password"
      label={i18n.t('password.label')}
      bind:value={password}
      autocomplete="new-password"
      note={i18n.t('password.rules')}
      error={fieldErrors['owner_password'] ?? ''}
    />
    <p class="muted">{i18n.t('password.keep')}</p>
    {#if error && !fieldErrors['owner_password']}<p class="field-error" role="alert">{error}</p>{/if}
    <button class="primary" onclick={check} disabled={busy || password.length < (claim?.limits.password.min ?? 10)}>{i18n.t('common.next')}</button>
    <button class="link" onclick={back}>{i18n.t('common.back')}</button>
  {:else if step === 'keep' && app.proposal}
    <h1>{i18n.t('keep.title')}</h1>
    <p class="lead">{i18n.t('keep.lead')}</p>
    <div class="card">
      <h2>{i18n.t('keep.management_name')}</h2>
      <p class="mono">{app.proposal.management.name}</p>
      <h2>{i18n.t('keep.management_password')}</h2>
      <p class="mono">{app.proposal.management.key}</p>
      <Qr text={wifiQrPayload(app.proposal.management.name, app.proposal.management.key)} label={i18n.t('keep.qr_wifi')} />
    </div>
    <div class="card">
      <h2>{i18n.t('keep.address')}</h2>
      <p class="mono">{addressUrl(app.proposal.address)}</p>
      <Qr text={addressUrl(app.proposal.address)} label={i18n.t('keep.qr_address')} />
    </div>
    {#if error}<p class="field-error" role="alert">{error}</p>{/if}
    <button class="primary" onclick={confirm} disabled={busy}>{i18n.t('keep.button')}</button>
    <button class="link" onclick={back}>{i18n.t('common.back')}</button>
  {/if}
</main>
