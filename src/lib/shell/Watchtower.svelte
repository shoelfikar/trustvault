<script lang="ts">
  /**
   * The Watchtower surface — the prototype's stat cards and grouped findings, drawn from a real
   * scan since 2026-08-16.
   *
   * **Where each number comes from matters, and the two sources are not interchangeable.** The
   * groups and their rows come from the `watchtower_scan` report: it is the fresh answer and the
   * only thing carrying crack times and who a password is shared with. The four **tiles** count
   * items by their cached `status` — every one of them, including *Safe*, which could not come
   * from anywhere else: §6.9 gives the report **no row saying strong**, so "scanned and clean"
   * exists only as an absence and the cache is what was written to hold it (D-26). See the
   * comment on `stats` for the two other things that choice buys.
   *
   * Two departures from the prototype's copy, both deletions of a claim we cannot make — D-82:
   *
   * * The weak rows read *"Crackable in 31 minutes"* (zxcvbn's own phrasing, R-24) where the
   *   prototype reads *"10 characters · word + year"*. A character count is the **length of a
   *   password**, and D-32 went to the trouble of making every mask a fixed width precisely so
   *   the length of a secret never crosses this boundary. Printing it in a findings list would
   *   undo that for the passwords most worth guessing.
   * * The lede does not mention Have I Been Pwned. The prototype's says "hashes matched against
   *   Have I Been Pwned", and with breach checking off — the default, R-26 — nothing was matched
   *   against anything. It says what actually happened instead.
   *
   * Brass is absent from the whole surface. §2 reserves the accent for brand and interaction,
   * and a page whose entire job is verdicts is where that rule is most easily broken.
   */
  import Icon, { type IconName } from '../icons/Icon.svelte';
  import EmptyState from '../components/EmptyState.svelte';
  import type { Finding, ItemStatus, ItemSummary, Verdict, WatchtowerReport } from '../ipc';
  import { TYPE_GLYPHS } from './views';

  interface Props {
    items: ItemSummary[];
    /** The last scan's report, or `null` when none has run in this session. */
    report: WatchtowerReport | null;
    /** A scan is in flight. Milliseconds for a normal vault (S-07a), so it is rarely seen. */
    scanning: boolean;
    /** A scan the host refused. Shown instead of findings — never as an empty findings list. */
    error?: string;
    /** When the vault says it was last scanned, from `vault_status` — `null` means never. */
    lastScanAt: number | null;
    onopen: (id: string) => void;
    /** Scan again. Reachable only from the error state — see the comment on it. */
    onretry: () => void;
  }

  const { items, report, scanning, error = '', lastScanAt, onopen, onretry }: Props = $props();

  type Tone = 'ok' | 'warn' | 'danger';

  const GROUPS: {
    verdict: Verdict;
    title: string;
    desc: string;
    icon: IconName;
    tone: Tone;
    action: string;
  }[] = [
    {
      verdict: 'breached',
      title: 'Breached passwords',
      desc: 'Found in public breach databases, checked with k-anonymity hashes. Change these now.',
      icon: 'shield',
      tone: 'danger',
      action: 'Change password',
    },
    {
      verdict: 'reused',
      title: 'Reused passwords',
      desc: 'One password is used on more than one item.',
      icon: 'alert',
      tone: 'warn',
      action: 'Make it unique',
    },
    {
      verdict: 'weak',
      title: 'Weak passwords',
      desc: 'Crackable in a matter of hours.',
      icon: 'alert',
      tone: 'warn',
      action: 'Strengthen it',
    },
    {
      verdict: 'expired',
      title: 'Expired credentials',
      desc: 'Past the rotation date recorded on the item.',
      icon: 'clock',
      tone: 'warn',
      action: 'Rotate it',
    },
  ];

  const titleOf = (id: string) => items.find((item) => item.id === id)?.title ?? 'another item';

  /**
   * What the row says about the finding, in the group's own terms.
   *
   * Reuse names the other items, which is the prototype's copy and the reason `shared_with`
   * carries ids rather than a group key: the ids are already on this side, so resolving them to
   * titles costs nothing and discloses nothing.
   */
  function note(finding: Finding): string {
    switch (finding.verdict) {
      case 'reused':
        return `Same as ${finding.sharedWith.map(titleOf).join(', ')}`;
      case 'weak':
        // zxcvbn's phrasing, verbatim — "31 minutes", "less than a second" (R-24).
        return finding.crackTime ? `Crackable in ${finding.crackTime}` : 'Scored weak by zxcvbn';
      case 'breached':
        // Nothing produces this yet. The count the row will show is a property of the corpus,
        // not of the password, and it arrives with `watchtower_breach_check`.
        return 'Seen in a public breach corpus';
      default:
        return 'Rotation date has passed';
    }
  }

  /**
   * One row per item per verdict, not one row per finding.
   *
   * An item with **two** password fields can earn two `weak` findings, and both rows would carry
   * the same title and open the same item — a list where the second row does nothing the first
   * did not. The field is not nameable from here either: the report carries `field_id` and the
   * item summary carries no fields (§6.1), so a row could not even say which of the two it meant.
   */
  const groups = $derived(
    GROUPS.map((group) => {
      const seen = new Set<string>();
      const rows = [];
      for (const finding of report?.findings ?? []) {
        if (finding.verdict !== group.verdict || seen.has(finding.itemId)) continue;
        seen.add(finding.itemId);
        rows.push({
          id: finding.itemId,
          kind: items.find((item) => item.id === finding.itemId)?.kind ?? 'login',
          title: titleOf(finding.itemId),
          note: note(finding),
        });
      }
      return { ...group, rows };
    }).filter((group) => group.rows.length > 0),
  );

  /**
   * The four tiles count **items by their cached status**, not findings — and all four come from
   * the same place on purpose.
   *
   * Three properties fall out of that choice, and the third is why it was made:
   *
   * * They agree with the item list's pips and the sidebar's badge, because those read the same
   *   field. Counting findings instead would put *Reused 2* above a list showing one reused pip,
   *   the difference being an item that is breached **and** reused — true, and unexplainable in a
   *   tile with one number in it.
   * * Each item is counted once, under its worst verdict, so the four tiles partition the vault
   *   instead of overlapping. The **groups** below are per-finding and do overlap: that is where
   *   an item appears twice, with a row explaining each verdict.
   * * They survive a scan that did not run. A vault whose scan was refused shows what the last
   *   one concluded and says the new one failed; tiles fed from the report would read **0
   *   breached, 0 reused, 0 weak** over an error message, which is a clean bill of health issued
   *   by a check that never happened — R-25's "not checked, never safe", one screen early.
   */
  const countOf = (status: ItemStatus) => items.filter((item) => item.status === status).length;

  const stats = $derived([
    {
      label: 'Breached',
      n: countOf('breached'),
      tone: 'danger' as Tone,
      icon: 'shield' as IconName,
    },
    { label: 'Reused', n: countOf('reused'), tone: 'warn' as Tone, icon: 'alert' as IconName },
    { label: 'Weak', n: countOf('weak'), tone: 'warn' as Tone, icon: 'alert' as IconName },
    {
      label: 'Safe',
      n: countOf('strong'),
      tone: 'ok' as Tone,
      icon: 'shield-check' as IconName,
    },
  ]);

  /**
   * The lede, which is a statement about what has happened and must not overstate it.
   *
   * `lastScanAt` is `null` on a vault nobody has scanned; the caller only renders this screen
   * while unlocked, so `null` here means never rather than unknowable (§6.9 names both).
   */
  const lede = $derived.by(() => {
    if (scanning) return `Scanning ${items.length} items — nothing leaves this device.`;
    if (lastScanAt === null) {
      return `${items.length} items · not scanned yet.`;
    }
    const when = new Date(lastScanAt).toLocaleString(undefined, {
      dateStyle: 'medium',
      timeStyle: 'short',
    });
    const passwords = report
      ? `${report.passwords} passwords in ${items.length} items`
      : `${items.length} items`;
    // No mention of HIBP: with breach checking off, nothing was matched against it — D-82.
    return `Last scanned ${when} · ${passwords} checked · nothing on this screen has left this device.`;
  });

  const anyFindings = $derived(groups.length > 0);

  /**
   * ↑/↓ within a findings list — row 17 of `docs/keyboard-audit.md`.
   *
   * The same shape `ItemList.svelte` uses, deliberately: every row stays a real tab stop and the
   * arrows move focus between them, so nothing here is a roving group with a `tabindex="-1"`
   * control in it (D-71's audit, and finding 6 is what that rule was written for).
   */
  function onkeydown(event: KeyboardEvent, count: number, index: number) {
    const next = event.key === 'ArrowDown' ? index + 1 : event.key === 'ArrowUp' ? index - 1 : null;
    if (next === null || next < 0 || next >= count) return;
    event.preventDefault();
    const row = event.currentTarget as HTMLElement;
    const sibling = row.parentElement?.parentElement?.children[next]?.querySelector('button');
    (sibling as HTMLElement | null)?.focus();
  }
</script>

<div class="watchtower sb">
  <div class="column">
    <h1>Watchtower</h1>
    <p class="lede">{lede}</p>

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

    {#each groups as group (group.verdict)}
      <section class="group">
        <header>
          <span class="group-icon {group.tone}"><Icon name={group.icon} size={14} /></span>
          <span class="group-title">{group.title}</span>
          <span class="group-desc">{group.desc}</span>
        </header>
        <!-- Row 17's own words: the findings list is a list. -->
        <ul class="rows" aria-label={group.title}>
          {#each group.rows as row, index (row.id)}
            <li>
              <button
                class="row"
                class:first={index === 0}
                onclick={() => onopen(row.id)}
                onkeydown={(event) => onkeydown(event, group.rows.length, index)}
              >
                <span class="row-icon"><Icon name={TYPE_GLYPHS[row.kind]} size={16} /></span>
                <span class="row-title">{row.title}</span>
                <span class="row-note">{row.note}</span>
                <span class="row-action {group.tone}">{group.action}</span>
                <span class="chev"><Icon name="chev" size={14} /></span>
              </button>
            </li>
          {/each}
        </ul>
      </section>
    {/each}

    {#if error}
      <!-- A refused scan is not a clean vault. R-25's rule about "not checked, never safe" is
           written for the breach half and it is the same rule here, in two shapes: with no earlier
           report there are no groups above this and the sentence is the only thing on the page,
           and with one there are stale groups above it that the sentence dates. Neither may be
           allowed to read as "nothing to report". -->
      <div class="clear">
        <EmptyState
          icon="alert"
          message={error}
          actionLabel="Try again"
          actionDisabled={scanning}
          onaction={onretry}
        />
      </div>
    {:else if !anyFindings}
      <div class="clear">
        <EmptyState
          icon="shield-check"
          message={items.length === 0
            ? 'Nothing to check yet — this vault is empty.'
            : scanning
              ? 'Scanning…'
              : lastScanAt === null
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
