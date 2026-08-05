/**
 * Candidate-password generation for the *preview* surfaces.
 *
 * This is deliberately the only place in the frontend that produces a password, and it is not
 * the generator the product ships. `docs/ipc-contract.md` names `generate_password` as a fourth
 * sanctioned command that does not exist yet, and the reason it belongs in the host is that a
 * password minted here lives in a heap that cannot be wiped, and the clipboard clear that makes
 * a copy safe is scheduled by Rust in `copy_field`. Nothing in this file may be handed to the
 * clipboard for that reason.
 *
 * Two properties are non-negotiable and both are easy to lose in a refactor:
 *
 * * **`crypto.getRandomValues`, never `Math.random`.** The prototype uses `Math.random`, which
 *   is a seeded PRNG with a recoverable state — fine for a mock, catastrophic if it ever became
 *   the real one by nobody noticing.
 * * **Rejection sampling, not modulo.** `byte % alphabet.length` is biased towards the first
 *   characters whenever 256 is not a multiple of the length, which it never is here. Discarding
 *   the tail costs a few extra bytes and removes the bias entirely.
 */

/** Look-alike glyphs are out of every set: no 0/O, no 1/l/I. The same reasoning as D-24. */
export const CHARACTER_SETS = {
  lower: 'abcdefghijkmnpqrstuvwxyz',
  upper: 'ABCDEFGHJKLMNPQRSTUVWXYZ',
  digits: '23456789',
  symbols: '!@#$%&*-_=+?',
} as const;

export type CharacterSet = keyof typeof CHARACTER_SETS;

export function generatePassword(length: number, sets: Record<CharacterSet, boolean>): string {
  const alphabet =
    (Object.keys(CHARACTER_SETS) as CharacterSet[])
      .filter((name) => sets[name])
      .map((name) => CHARACTER_SETS[name])
      .join('') || CHARACTER_SETS.lower;

  const limit = 256 - (256 % alphabet.length);
  const out: string[] = [];
  const buffer = new Uint8Array(64);

  while (out.length < length) {
    crypto.getRandomValues(buffer);
    for (const byte of buffer) {
      if (byte >= limit) continue;
      out.push(alphabet[byte % alphabet.length]!);
      if (out.length === length) break;
    }
  }
  return out.join('');
}

export const ALL_SETS: Record<CharacterSet, boolean> = {
  lower: true,
  upper: true,
  digits: true,
  symbols: true,
};
