<script lang="ts">
  /**
   * A live code generated from a seed the user is typing — R-20, `docs/ipc-contract.md` §5.
   *
   * Two jobs, and the second is the one that earns the component. It **shows** a code so the
   * user can check it against the site before saving, and it **validates**: a base32 string
   * that will not decode is refused here, with the field on screen and the phone still in the
   * user's hand. Without it, a mistyped seed is discovered a month later at a login prompt,
   * when the QR code is gone and the only move left is the site's support desk.
   *
   * It lives in `components/` rather than inside the Add dialog because the Edit dialog can
   * change a seed too, and a validator that only guards one of the two ways in is a validator
   * that will be reported as "sometimes it checks".
   *
   * The code is generated in the **host**, per call, and this side never holds the decoded
   * seed — the same argument D-44 made for the password generator.
   */
  import Icon from '../icons/Icon.svelte';
  import TotpRing from './TotpRing.svelte';
  import { asIpcError, totpPreview, type TotpCode } from '../ipc';

  interface Props {
    /** The seed as typed. Empty renders the hint instead of a code. */
    seed: string;
    /** Shown while the field is empty. */
    hint?: string;
  }

  const { seed, hint = '' }: Props = $props();

  let preview = $state<TotpCode | null>(null);
  let error = $state('');

  $effect(() => {
    const current = seed.trim();
    preview = null;
    error = '';
    if (!current) return;

    let live = true;
    let debounce: ReturnType<typeof setTimeout>;
    let refreshAt: ReturnType<typeof setTimeout>;

    const refresh = async () => {
      try {
        const next = await totpPreview(current);
        if (!live) return;
        preview = next;
        error = '';
        // Kept live across the step boundary. The whole point is that the user compares this
        // with their phone, and a frozen code that stops matching reads as "the secret is
        // wrong" — the opposite of what it means.
        refreshAt = setTimeout(
          () => void refresh(),
          Math.max(250, next.expiresAt - Date.now() + 250),
        );
      } catch (thrown) {
        if (!live) return;
        preview = null;
        error = asIpcError(thrown).message;
      }
    };

    // Debounced like the strength meter: a half-typed seed is not a wrong seed, and an error
    // that appears on the third character of a paste is noise the user learns to ignore.
    debounce = setTimeout(() => void refresh(), 180);

    return () => {
      live = false;
      clearTimeout(debounce);
      clearTimeout(refreshAt);
    };
  });

  /**
   * `271934` → `271 934`, and an eight-digit code into two fours.
   *
   * Grouped for the same reason §3 puts every secret in the mono face: this is read off a
   * screen and typed somewhere else, against a clock.
   */
  const grouped = $derived.by(() => {
    const code = preview?.code ?? '';
    const half = Math.ceil(code.length / 2);
    return `${code.slice(0, half)} ${code.slice(half)}`.trim();
  });
</script>

{#if preview}
  <div class="preview">
    <span class="code">{grouped}</span>
    <TotpRing expiresAt={preview.expiresAt} period={preview.period} size={18} showSeconds={false} />
    <span class="note">
      Match this code with the one the site asks for to confirm the secret is right.
    </span>
  </div>
{:else if error}
  <!-- Status is never colour alone — MASTER.md §2: the icon and the sentence carry it. -->
  <p class="error" role="alert"><Icon name="alert" size={13} />{error}</p>
{:else if hint}
  <p class="hint">{hint}</p>
{/if}

<style>
  /* The prototype's preview strip: a hairline box around the code, the ring, and the sentence
     that says what to do with them. */
  .preview {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 10px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-surface);
  }

  /* §3 — a code read off the screen and typed elsewhere is a rendered secret. */
  .code {
    font-family: var(--font-mono);
    font-size: var(--text-md);
    font-weight: var(--weight-medium);
    letter-spacing: 0.14em;
    font-variant-numeric: tabular-nums slashed-zero;
    font-feature-settings:
      'ss01' 1,
      'ss02' 1;
  }

  .note {
    flex: 1;
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  .hint {
    margin-top: 6px;
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-subtle);
    text-wrap: pretty;
  }

  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 6px;
    font-size: var(--text-sm);
    color: var(--danger);
  }
</style>
