/**
 * What each of the seven item types is made of — R-17.
 *
 * The New-item dialog drew these as hand-written markup, one block per type, which was right
 * while nothing could be saved: the blocks were a picture of an item. They have to become data
 * the moment a form produces `NewField[]`, because otherwise the labels and kinds that reach
 * the vault are typed twice — once in the markup and once in whatever builds the payload — and
 * the two drift on the first edit.
 *
 * `secret` is a property of the template, never inferred from the label. That is the same rule
 * `docs/vault-format.md` §6.3 states for storage, one step earlier: a field is masked because
 * the type says it is, not because it is spelled "password".
 *
 * `custom` is not here at all. Every field a template produces is one of the type's own, so it
 * is `false` by construction; the only `true` comes from a field the user added by hand or an
 * import created (D-43).
 */
import type { FieldKind, ItemKind } from '../ipc';

export interface FieldTemplate {
  label: string;
  kind: FieldKind;
  /** Masked in the detail pane, revealable only through `reveal_field`. */
  secret: boolean;
  placeholder?: string;
  /** Renders as a textarea. */
  multiline?: boolean;
  /** Renders in `--font-mono` — anything the user may transcribe (`MASTER.md` §3). */
  mono?: boolean;
  /** Offered as a fixed set rather than a free-text input. */
  options?: string[];
  /** Sits on the same row as the field before it. */
  inline?: boolean;
  /**
   * Drawn only when the dialog's 2FA switch is on.
   *
   * The switch exists because the design draws one, and it earns its keep: a TOTP seed field
   * standing open on every new login is an invitation to paste a password into it.
   */
  behindTotpSwitch?: boolean;
}

export const ITEM_FIELDS: Record<ItemKind, FieldTemplate[]> = {
  login: [
    {
      label: 'Username',
      kind: 'username',
      secret: false,
      placeholder: 'budi.santoso@gmail.com',
    },
    { label: 'Password', kind: 'password', secret: true, mono: true },
    { label: 'Website', kind: 'url', secret: false, placeholder: 'tokopedia.com', mono: true },
    {
      label: '2FA secret',
      kind: 'otp',
      secret: true,
      mono: true,
      placeholder: 'JBSW Y3DP EHPK 3PXP or otpauth://…',
      behindTotpSwitch: true,
    },
  ],
  api_key: [
    {
      label: 'API key',
      // `text` and not `password`: it is secret, but it is not scored, not rotated on a
      // schedule, and not what a strength meter has anything to say about. `secret` is what
      // masks it; `kind` is what the surfaces read to decide how to treat it.
      kind: 'text',
      secret: true,
      mono: true,
      multiline: true,
      placeholder: 'sk-ant-api03-… or ghp_… — paste the token here',
    },
    {
      label: 'Environment',
      kind: 'text',
      secret: false,
      options: ['Production', 'Staging', 'Local'],
    },
    { label: 'Expiry', kind: 'date', secret: false, placeholder: '31 Dec 2026', mono: true },
    {
      label: 'Docs URL',
      kind: 'url',
      secret: false,
      placeholder: 'console.anthropic.com',
      mono: true,
    },
  ],
  card: [
    { label: 'Name on card', kind: 'text', secret: false, placeholder: 'BUDI SANTOSO' },
    {
      label: 'Card number',
      kind: 'text',
      secret: true,
      mono: true,
      placeholder: '4811 7742 9930 4417',
    },
    { label: 'Expiry', kind: 'date', secret: false, mono: true, placeholder: '08 / 29' },
    { label: 'CVV', kind: 'text', secret: true, mono: true, placeholder: '•••', inline: true },
  ],
  note: [
    {
      label: 'Note',
      kind: 'note',
      // Secret, because a secure note is the surface people keep recovery codes on. The
      // prototype's placeholder says so out loud and the storage has to agree with it.
      secret: true,
      multiline: true,
      placeholder: 'This note is encrypted just like a password.',
    },
  ],
  wifi: [
    { label: 'Network name', kind: 'text', secret: false, placeholder: 'Santoso-5G' },
    { label: 'Password', kind: 'password', secret: true, mono: true },
    { label: 'Security', kind: 'text', secret: false, placeholder: 'WPA3-Personal' },
  ],
  ssh_key: [
    { label: 'Host', kind: 'text', secret: false, mono: true, placeholder: '10.4.1.22' },
    { label: 'User', kind: 'username', secret: false, mono: true, placeholder: 'deploy' },
    { label: 'Passphrase', kind: 'password', secret: true, mono: true },
  ],
  identity: [
    { label: 'Full name', kind: 'text', secret: false, placeholder: 'Budi Santoso' },
    { label: 'Email', kind: 'email', secret: false, mono: true, placeholder: 'budi@example.com' },
    { label: 'Phone', kind: 'text', secret: false, mono: true, placeholder: '+62 812 8891 4402' },
  ],
};

/** The placeholder title the prototype shows per type, kept here so the two lists stay together. */
export const TITLE_PLACEHOLDERS: Record<ItemKind, string> = {
  login: 'Tokopedia',
  api_key: 'Anthropic API',
  card: 'BCA Visa Platinum',
  note: '2FA recovery codes',
  wifi: 'Home Wi-Fi',
  ssh_key: 'Production server',
  identity: 'Passport',
};

/** Which field of a type the strength meter reads, if the type has one at all. */
export const passwordIndex = (kind: ItemKind) =>
  ITEM_FIELDS[kind].findIndex((field) => field.kind === 'password');
