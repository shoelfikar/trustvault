<script lang="ts">
  /**
   * The prototype's 440px *Edit profile* dialog — D-70.
   *
   * Three of its four blocks are the design's unchanged: the initials preview beside a sentence
   * about where the avatar comes from, a Full name field, and an Email field with a caption
   * saying what the address is and is not for. The fourth is gone. The prototype's last block is
   * a *Lifetime license · Active since 12 Jan 2025* card with a **Manage** link, and there is no
   * license, no purchase and nowhere for Manage to go — deleted rather than drawn inert, which
   * is the same call `ProfileMenu` makes about *Sign out*.
   *
   * **The avatar updates as you type**, which is why the initials are derived here rather than
   * passed in: it is the only feedback in the dialog that shows what the name will look like in
   * the 22px footer circle, where two characters is all there is room for.
   *
   * Nothing in here is a secret, so nothing in here is masked and neither field is `mono`:
   * `MASTER.md` §3 makes the mono face a correctness requirement for **transcribed** strings,
   * and a name is not one.
   */
  import { untrack } from 'svelte';
  import Button from '../components/Button.svelte';
  import Dialog from '../components/Dialog.svelte';
  import Icon from '../icons/Icon.svelte';
  import TextField from '../components/TextField.svelte';
  import { asIpcError, setProfile } from '../ipc';
  import { initialsOf } from './profile';

  interface Props {
    name: string;
    email: string;
    onclose: () => void;
    /** The profile landed. The shell re-reads `vault_status` rather than being told the values. */
    onsaved: () => void;
  }

  const { name, email, onclose, onsaved }: Props = $props();

  // Seeded once, by design: from here on the fields are the authority and the props are the
  // values the dialog opened with. Reading them reactively would overwrite what is being typed
  // the moment the router re-reads status. Same capture `Shell.svelte` makes for its pane
  // widths, and `untrack` is what says it is the intent rather than an oversight.
  let typedName = $state(untrack(() => name));
  let typedEmail = $state(untrack(() => email));
  let saving = $state(false);
  let error = $state('');

  /**
   * The same two-letter rule the footer, the menu and the Settings card use, run against the
   * **unsaved** string so the circle changes as the name is typed.
   *
   * The fallback is an em dash rather than the vault's initials, which is the one place it is:
   * a preview showing "PV" while the name field is empty would read as the vault name having
   * been typed into it.
   */
  const initials = $derived(initialsOf(typedName, '—'));

  /**
   * Saves and closes.
   *
   * There is no client-side validation and that is deliberate — the contract says the host does
   * not validate either. An empty name is legal: it is how a user removes a profile they no
   * longer want in the footer. An address without an `@` is legal too, because this string
   * labels a recovery kit rather than receiving mail, and refusing what the user chose for
   * their own label would be the app inventing a rule the format does not have.
   */
  async function save() {
    if (saving) return;
    saving = true;
    error = '';
    try {
      // Sent whether or not anything was typed. The host compares and skips the write when
      // nothing changed, so this side does not need its own copy of that decision — §6.4.
      await setProfile(typedName, typedEmail);
      onsaved();
    } catch (thrown) {
      error = asIpcError(thrown).message;
      saving = false;
    }
  }
</script>

<Dialog title="Edit profile" icon="user" width={440} {onclose}>
  <div class="pane">
    <div class="preview">
      <span class="avatar">{initials}</span>
      <p class="note">
        Your avatar is the initials of your name. No photo is uploaded — everything stays inside
        this vault, on this computer.
      </p>
    </div>

    <TextField
      label="Full name"
      bind:value={typedName}
      surface="surface"
      autofocus
      onenter={() => void save()}
    />

    <TextField
      label="Email"
      bind:value={typedEmail}
      surface="surface"
      hint="Only used to label your profile and your recovery kit. TrustVault has no account — it is never used to sign in and never leaves this device."
      onenter={() => void save()}
    />

    {#if error}
      <p class="error" role="alert"><Icon name="alert" size={13} />{error}</p>
    {/if}
  </div>

  {#snippet footer()}
    <span class="grow"></span>
    <Button onclick={onclose}>Cancel</Button>
    <Button variant="primary" disabled={saving} onclick={() => void save()}>
      {saving ? 'Saving…' : 'Save changes'}
    </Button>
  {/snippet}
</Dialog>

<style>
  .pane {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .preview {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .avatar {
    display: grid;
    place-items: center;
    width: 52px;
    height: 52px;
    flex: none;
    /* §4: --radius-full is for avatars only. */
    border-radius: var(--radius-full);
    background: var(--accent-wash);
    font-size: 18px;
    font-weight: var(--weight-semibold);
    color: var(--accent);
  }
  .note {
    font-size: var(--text-sm);
    line-height: var(--text-sm-lh);
    color: var(--fg-muted);
    text-wrap: pretty;
  }

  .grow {
    flex: 1;
  }

  /* Status is never colour alone — §2. */
  .error {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-sm);
    color: var(--danger);
  }
</style>
