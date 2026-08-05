<script lang="ts" module>
  /**
   * The TrustVault mark, inlined from the design's own brand folder.
   *
   * Separate from `Icon.svelte` because it is a 64px viewBox, not the 24px UI grid — feeding it
   * through the UI set would either distort it or force a second viewBox into a component whose
   * whole contract is "one set, one grid" (§8). It is inlined for the same reason the UI glyphs
   * are: an `<img>` cannot inherit `currentColor`, and the mark is `--accent` everywhere it
   * appears.
   */
  const files = import.meta.glob<string>('./brand/trustvault-mark.svg', {
    query: '?raw',
    eager: true,
    import: 'default',
  });

  const source = files['./brand/trustvault-mark.svg'] ?? '';

  const body = source.replace(/^[\s\S]*?<svg[^>]*>/, '').replace(/<\/svg>\s*$/, '');
</script>

<script lang="ts">
  interface Props {
    /** Rendered edge in px. The prototype uses 22 (chrome), 30 (onboarding), 46 (lock). */
    size?: number;
    label?: string;
  }

  const { size = 22, label }: Props = $props();
</script>

<svg
  class="mark"
  width={size}
  height={size}
  viewBox="0 0 64 64"
  fill="none"
  aria-hidden={label ? undefined : 'true'}
  aria-label={label}
  role={label ? 'img' : undefined}
>
  <!-- Build-time asset from this repository, never user input. -->
  {@html body}
</svg>

<style>
  .mark {
    display: block;
    flex: none;
    color: inherit;
  }
</style>
