<script lang="ts">
  /**
   * The Watchtower surface — the prototype's stat cards and grouped findings.
   *
   * Everything on this screen is read from the `status` each item already carries, which D-26
   * defines as a **cache of the last scan** and labels stale by definition. It is drawn here
   * with that said out loud rather than implied: there is no scanner in this build, so a "last
   * scanned" timestamp would be the one number on the page nobody could justify.
   *
   * Brass is absent from the whole surface. §2 reserves the accent for brand and interaction,
   * and a page whose entire job is verdicts is where that rule is most easily broken.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import EmptyState from '../components/EmptyState.svelte';
  import type { ItemStatus, ItemSummary } from '../ipc';
  import { TYPE_GLYPHS, isFlagged } from './views';

  interface Props {
    items: ItemSummary[];
    onopen: (id: string) => void;
  }

  const { items, onopen }: Props = $props();

  type Tone = 'ok' | 'warn' | 'danger';

  const GROUPS: {
    status: ItemStatus;
    title: string;
    desc: string;
    icon: IconName;
    tone: Tone;
    note: string;
    action: string;
  }[] = [
    {
      status: 'breached',
      title: 'Breached passwords',
      desc: 'Found in public breach databases, checked with k-anonymity hashes. Change these now.',
      icon: 'shield',
      tone: 'danger',
      note: 'Seen in a public breach corpus',
      action: 'Change password',
    },
    {
      status: 'reused',
      title: 'Reused passwords',
      desc: 'One password is used on more than one item.',
      icon: 'alert',
      tone: 'warn',
      note: 'Shared with another item in this vault',
      action: 'Make it unique',
    },
    {
      status: 'weak',
      title: 'Weak passwords',
      desc: 'Crackable in a matter of hours.',
      icon: 'alert',
      tone: 'warn',
      note: 'Short, or built from a word and a year',
      action: 'Strengthen it',
    },
    {
      status: 'expired',
      title: 'Expired credentials',
      desc: 'Past the rotation date recorded on the item.',
      icon: 'clock',
      tone: 'warn',
      note: 'Rotation date has passed',
      action: 'Rotate it',
    },
  ];

  const count = (status: ItemStatus) => items.filter((item) => item.status === status).length;

  const stats = $derived([
    { label: 'Breached', n: count('breached'), tone: 'danger' as Tone, icon: 'shield' as IconName },
    { label: 'Reused', n: count('reused'), tone: 'warn' as Tone, icon: 'alert' as IconName },
    { label: 'Weak', n: count('weak'), tone: 'warn' as Tone, icon: 'alert' as IconName },
    {
      label: 'Safe',
      n: items.filter((item) => item.status === 'strong').length,
      tone: 'ok' as Tone,
      icon: 'shield-check' as IconName,
    },
  ]);

  const groups = $derived(
    GROUPS.map((group) => ({
      ...group,
      rows: items.filter((item) => item.status === group.status),
    })).filter((group) => group.rows.length > 0),
  );

  const unscanned = $derived(items.filter((item) => item.status === 'unknown').length);
  const anyFlagged = $derived(items.some(isFlagged));
</script>

<div class="watchtower sb">
  <div class="column">
    <h1>Watchtower</h1>
    <p class="lede">
      Not scanned yet · {items.length} items · statuses below come from the vault's stored scan cache,
      which is stale by definition — nothing on this screen has left this device.
    </p>

    <div class="stats">
      {#each stats as stat (stat.label)}
        <div class="stat">
          <p class="stat-label {stat.tone}">
            <Icon name={stat.icon} size={14} />
            <span>{stat.label}</span>
          </p>
          <p class="stat-n">{stat.n}</p>
        </div>
      {/each}
    </div>

    {#each groups as group (group.status)}
      <section class="group">
        <header>
          <span class="group-icon {group.tone}"><Icon name={group.icon} size={14} /></span>
          <span class="group-title">{group.title}</span>
          <span class="group-desc">{group.desc}</span>
        </header>
        <div class="rows">
          {#each group.rows as item, index (item.id)}
            <button class="row" class:first={index === 0} onclick={() => onopen(item.id)}>
              <span class="row-icon"><Icon name={TYPE_GLYPHS[item.kind]} size={16} /></span>
              <span class="row-title">{item.title}</span>
              <span class="row-note">{group.note}</span>
              <span class="row-action {group.tone}">{group.action}</span>
              <span class="chev"><Icon name="chev" size={14} /></span>
            </button>
          {/each}
        </div>
      </section>
    {/each}

    {#if !anyFlagged}
      <div class="clear">
        <EmptyState
          icon="shield-check"
          message={items.length === 0
            ? 'Nothing to check yet — this vault is empty.'
            : unscanned === items.length
              ? 'No item has been scanned yet, so there is nothing to report.'
              : 'Watchtower has nothing to report.'}
          quiet
        />
      </div>
    {/if}
  </div>
</div>

<style>
  .watchtower {
    flex: 1;
    min-width: 0;
    background: var(--bg-surface);
    overflow-y: auto;
  }
  .column {
    max-width: 840px;
    padding: var(--space-6) var(--space-7) var(--space-7);
  }

  h1 {
    font-size: var(--text-lg);
    line-height: var(--text-lg-lh);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-lg);
  }
  .lede {
    margin-top: var(--space-1);
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  .stats {
    display: flex;
    gap: 10px;
    margin-top: 20px;
  }
  /* §4: no shadow on a card. A hairline and the raised surface do the separating. */
  .stat {
    flex: 1;
    padding: var(--space-4) 14px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
  }
  .stat-label {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: var(--text-micro);
    font-weight: var(--weight-medium);
    letter-spacing: var(--tracking-micro);
    text-transform: uppercase;
  }
  .stat-n {
    margin-top: 6px;
    font-size: var(--text-xl);
    line-height: 1.1;
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-xl);
    font-variant-numeric: tabular-nums;
  }

  .group {
    margin-top: var(--space-6);
  }
  .group header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }
  .group-icon {
    display: flex;
  }
  .group-title {
    font-size: var(--text-base);
    font-weight: var(--weight-medium);
  }
  .group-desc {
    font-size: var(--text-sm);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  .rows {
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
    overflow: hidden;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    height: 38px;
    padding: 0 14px;
    border-top: 1px solid var(--border);
    text-align: left;
    transition: background var(--dur-instant) var(--ease-out);
  }
  .row.first {
    border-top: none;
  }
  .row:hover {
    background: var(--bg-hover);
  }
  .row:focus-visible {
    outline: 1px solid var(--accent);
    outline-offset: -1px;
  }
  .row-icon {
    display: flex;
    color: var(--fg-muted);
  }
  .row-title {
    width: 180px;
    font-size: var(--text-base);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .row-note {
    flex: 1;
    min-width: 0;
    font-size: var(--text-sm);
    color: var(--fg-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .row-action {
    font-size: var(--text-sm);
    font-weight: var(--weight-medium);
    white-space: nowrap;
  }
  .chev {
    display: flex;
    color: var(--fg-subtle);
  }

  .clear {
    display: flex;
    margin-top: var(--space-6);
    padding: var(--space-6) 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-raised);
  }

  .ok {
    color: var(--ok);
  }
  .warn {
    color: var(--warn);
  }
  .danger {
    color: var(--danger);
  }
</style>
