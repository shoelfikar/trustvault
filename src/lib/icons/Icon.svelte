<script lang="ts" module>
  /**
   * The project's icon set, inlined at build time.
   *
   * `MASTER.md` §8 asks for one set and no mixing. That set is `src/lib/icons/ui/` — 31
   * hand-drawn glyphs on a 24px viewBox at 1.6px stroke, shipped with the design (D-28) and
   * added to only under the rule in `docs/icon-gaps.md`.
   *
   * They are inlined rather than loaded through `<img>` for one reason that decides it: an
   * `<img>` cannot inherit `currentColor`, and §8 specifies an icon that is `--fg-muted` at
   * rest, `--fg` on hover and `--accent` when active. `import.meta.glob` is used instead of a
   * sprite sheet so every glyph stays a plain file a designer can open and replace.
   */
  const files = import.meta.glob<string>('./ui/*.svg', {
    query: '?raw',
    eager: true,
    import: 'default',
  });

  /**
   * Every glyph in the set.
   *
   * Written out rather than derived from the glob because Vite types the glob's keys as
   * `string`, so a union inferred from it would accept any name at all and fail at runtime.
   * The cost of writing it by hand is drift, which is what the assertion below is for.
   */
  const NAMES = [
    'alert',
    'card',
    'check',
    'chev',
    'clock',
    'code',
    'copy',
    'enter',
    'eye',
    'eye-off',
    'globe',
    'history',
    'key',
    'list',
    'lock',
    'note',
    'plus',
    'refresh',
    'search',
    'settings',
    'shield',
    'shield-check',
    'star',
    'tag',
    'terminal',
    'trash',
    'unlock',
    'user',
    'vault',
    'wifi',
    'x',
  ] as const;

  export type IconName = (typeof NAMES)[number];

  const set = new Map<string, string>(
    Object.entries(files).map(([path, source]) => [
      path.slice('./ui/'.length, -'.svg'.length),
      // Drop the file's own 24x24 wrapper; this component re-emits it so the caller controls
      // the rendered size. Keeping the file's width/height would make `size` a lie.
      source.replace(/^[\s\S]*?<svg[^>]*>/, '').replace(/<\/svg>\s*$/, ''),
    ]),
  );

  // Fails the dev server on the edit that causes the drift, rather than at the moment
  // somebody opens the one screen using the glyph that went missing. Stripped from
  // production by the `import.meta.env.DEV` guard.
  if (import.meta.env.DEV) {
    const missing = NAMES.filter((name) => !set.has(name));
    const unlisted = [...set.keys()].filter((name) => !(NAMES as readonly string[]).includes(name));
    if (missing.length || unlisted.length) {
      throw new Error(
        `Icon set out of step with src/lib/icons/ui/: ${missing.length ? `declared but absent: ${missing.join(', ')}. ` : ''}${unlisted.length ? `present but undeclared: ${unlisted.join(', ')}.` : ''}`,
      );
    }
  }
</script>

<script lang="ts">
  interface Props {
    name: IconName;
    /** Rendered edge in px. §8's grid is 16; list rows and empty states go larger. */
    size?: number;
    /**
     * Set this when the icon is the only content of a control. Without it the icon is
     * `aria-hidden`, which is correct beside a text label and wrong on its own.
     */
    label?: string;
  }

  const { name, size = 16, label }: Props = $props();

  const body = $derived(set.get(name) ?? '');
</script>

<svg
  class="icon"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  aria-hidden={label ? undefined : 'true'}
  aria-label={label}
  role={label ? 'img' : undefined}
>
  <!-- Build-time asset from this repository, never user input. -->
  {@html body}
</svg>

<style>
  .icon {
    display: block;
    flex: none;
    /* §8: the glyph takes its colour from whatever it sits in. */
    color: inherit;
  }
</style>
