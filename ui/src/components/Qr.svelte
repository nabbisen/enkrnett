<script lang="ts">
  import { renderSVG } from 'uqr';

  // The QR code is drawn in the UI without any network request (ENK-UI-025). The SVG carries
  // only path data, never text from the API.
  let { text, label }: { text: string; label: string } = $props();
  const svg = $derived(renderSVG(text, { ecc: 'M', border: 1 }));
</script>

<figure class="qr" role="img" aria-label={label}>
  {@html svg}
</figure>
<p class="field-note">{label}</p>
