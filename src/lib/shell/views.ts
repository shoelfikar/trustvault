/**
 * What the sidebar can be pointed at.
 *
 * One union rather than a `view` plus a `tag` plus a `filter`: the prototype keeps three
 * variables in step by hand and gets away with it because it never reloads. Here the selection
 * has to survive a lock, an unlock and a webview reload, and three variables that must agree
 * is three chances for them not to.
 */
import type { ItemKind, ItemStatus, ItemSummary } from '../ipc';

export type View =
  | { kind: 'all' }
  | { kind: 'favourites' }
  | { kind: 'type'; type: ItemKind }
  | { kind: 'tag'; tag: string }
  | { kind: 'trash' }
  | { kind: 'watchtower' }
  | { kind: 'settings' };

/** The two views that replace the list *and* the detail pane with a full-width surface. */
export const isFullWidth = (view: View) => view.kind === 'watchtower' || view.kind === 'settings';

export function viewTitle(view: View): string {
  switch (view.kind) {
    case 'all':
      return 'All Items';
    case 'favourites':
      return 'Favorites';
    case 'type':
      return typeLabel(view.type);
    case 'tag':
      return view.tag;
    case 'trash':
      return 'Trash';
    case 'watchtower':
      return 'Watchtower';
    case 'settings':
      return 'Settings';
  }
}

export function sameView(a: View, b: View): boolean {
  if (a.kind !== b.kind) return false;
  if (a.kind === 'type' && b.kind === 'type') return a.type === b.type;
  if (a.kind === 'tag' && b.kind === 'tag') return a.tag === b.tag;
  return true;
}

export function matches(view: View, item: ItemSummary): boolean {
  switch (view.kind) {
    case 'all':
      return true;
    case 'favourites':
      return item.favourite;
    case 'type':
      return item.kind === view.type;
    case 'tag':
      return item.tags.includes(view.tag);
    // Deleting an item is a Phase 3 command, so nothing can be in the trash yet. It is drawn
    // rather than hidden because an absent Trash reads as "this app does not have one", which
    // is a different and wrong promise.
    case 'trash':
      return false;
    case 'watchtower':
    case 'settings':
      return false;
  }
}

/** The statuses Watchtower has something to say about. `strong` and `unknown` are not findings. */
export const FLAGGED: ItemStatus[] = ['weak', 'reused', 'breached', 'expired'];

export const isFlagged = (item: ItemSummary) => FLAGGED.includes(item.status);

const TYPE_LABELS: Record<ItemKind, string> = {
  login: 'Login',
  api_key: 'API Key',
  card: 'Card',
  note: 'Secure Note',
  wifi: 'Wi-Fi',
  ssh_key: 'SSH Key',
  identity: 'Identity',
};

export const typeLabel = (kind: ItemKind) => TYPE_LABELS[kind];

/**
 * One glyph per item type.
 *
 * `ssh_key` and `api_key` both land on `terminal`, which makes two of the seven types
 * indistinguishable in the list — the shipped 30-glyph set has no third candidate and `key` is
 * already spent on `login`. Tracked in `docs/icon-gaps.md`; it is a glyph somebody has to draw.
 */
export const TYPE_GLYPHS = {
  login: 'key',
  api_key: 'terminal',
  card: 'card',
  note: 'note',
  wifi: 'wifi',
  ssh_key: 'terminal',
  identity: 'user',
} as const;

/**
 * A tag's dot colour.
 *
 * The prototype hard-codes four tags to four colours. Real tags are arbitrary strings, so the
 * colour is chosen by hashing the name into the same four tokens — stable across launches,
 * which is the only property that matters here. These are *decoration*, not status: the four
 * include `--fg-subtle` precisely so a tag row cannot be read as a verdict.
 */
const TAG_COLOURS = ['var(--info)', 'var(--ok)', 'var(--warn)', 'var(--fg-subtle)'];

export function tagColour(tag: string): string {
  let hash = 0;
  for (const char of tag) hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  return TAG_COLOURS[hash % TAG_COLOURS.length]!;
}
