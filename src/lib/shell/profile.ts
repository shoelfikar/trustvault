/**
 * What the four profile surfaces agree on — D-70.
 *
 * The sidebar footer, its popover, the Settings card and the Edit-profile preview all draw the
 * same avatar, and all four fall back the same way when no profile has been filled in. Written
 * once because "the same two letters" is exactly the kind of rule that drifts into four
 * slightly different rules, and the drift is invisible until two of them are on screen at once.
 */

import type { Profile } from '../ipc';

/**
 * The first letter of the first two words, uppercased.
 *
 * `fallback` is what an empty string produces, and every caller passes something different on
 * purpose: the footer falls back to the vault's initials, the dialog's live preview to an em
 * dash, because a preview showing "PV" while the name field is empty would look like the vault
 * name had been typed into it.
 */
export function initialsOf(name: string, fallback = ''): string {
  return (
    name
      .trim()
      .split(/\s+/)
      .map((word) => word[0] ?? '')
      .slice(0, 2)
      .join('')
      .toUpperCase() || fallback
  );
}

/** Whether a profile exists to show. `null` — a locked vault — counts as absent. */
export const hasProfile = (profile: Profile | null): boolean =>
  Boolean(profile && (profile.name || profile.email));
