<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";

  import { api } from "../api";
  import Icon from "../icon.svelte";
  import { mail } from "../mail.svelte";
  import Avatar from "./avatar.svelte";

  let clientId = $state(mail.settings.clientId);
  let clientSecret = $state("");
  let saving = $state(false);
  const dirty = $derived(clientId.trim() !== mail.settings.clientId || clientSecret.trim() !== "");

  async function save() {
    saving = true;
    try {
      await api.setClient(clientId, clientSecret);
      mail.settings = await api.settings();
      clientSecret = "";
    } catch (e) {
      mail.fail(e);
    } finally {
      saving = false;
    }
  }

  async function add() {
    try {
      if (dirty) await save();
      await mail.addAccountFlow();
    } catch (e) {
      mail.fail(e);
    }
  }

  const input =
    "h-10 w-full rounded-lg border border-input/40 bg-background px-3 text-sm outline-none focus:border-ring placeholder:text-muted-foreground";
</script>

<div
  role="presentation"
  class="fixed inset-0 z-40 grid place-items-center bg-black/40 p-8"
  onkeydown={(e) => e.key === "Escape" && (mail.settingsOpen = false)}
>
  <div class="flex max-h-full w-[min(560px,100%)] flex-col overflow-hidden rounded-2xl bg-background shadow-modal">
    <header class="flex h-14 shrink-0 items-center border-b px-5">
      <h2 class="flex-1 text-base font-semibold">Settings</h2>
      <button type="button" aria-label="Close" class="grid size-8 place-items-center rounded-lg hover:bg-muted" onclick={() => (mail.settingsOpen = false)}>
        <Icon name="X" class="size-4" />
      </button>
    </header>

    <div class="space-y-7 overflow-y-auto p-5">
      <section class="space-y-3">
        <div>
          <h3 class="text-sm font-semibold">Google OAuth client{mail.settings.builtIn ? " (optional)" : ""}</h3>
          <p class="mt-1 text-sm text-muted-foreground">
            {#if mail.settings.builtIn}
              MailThis comes with its own Google client. To use yours instead, save it here (type <em>Desktop app</em>, Gmail API
              enabled). Accounts signed in with the other client need to sign in again. Open the
            {:else}
              MailThis signs in with your own Google Cloud client (type <em>Desktop app</em>, Gmail API enabled). See the README
              for the steps, or open the
            {/if}
            <button type="button" class="text-[#005bd3] underline" onclick={() => openUrl("https://console.cloud.google.com/auth/clients")}>
              Google Cloud console</button
            >.
          </p>
        </div>
        <input bind:value={clientId} class={input} placeholder="Client ID (….apps.googleusercontent.com)" spellcheck="false" />
        <input
          bind:value={clientSecret}
          type="password"
          class={input}
          placeholder={mail.settings.hasSecret ? "Client secret (saved; leave blank to keep)" : "Client secret"}
        />
        <div class="flex justify-end">
          <button
            type="button"
            disabled={!dirty || saving}
            onclick={save}
            class="h-9 rounded-lg border px-4 text-sm font-medium hover:bg-muted disabled:opacity-50"
          >
            Save
          </button>
        </div>
      </section>

      <section class="space-y-3">
        <h3 class="text-sm font-semibold">Accounts</h3>
        {#each mail.settings.accounts as account (account.email)}
          <div class="flex items-center gap-3 rounded-xl border px-3 py-2.5">
            <Avatar addr={account} />
            <div class="min-w-0 flex-1">
              <p class="truncate text-sm font-medium">{account.name ?? account.email}</p>
              <p class="truncate text-xs text-muted-foreground">{account.email}</p>
            </div>
            <button
              type="button"
              class="h-8 rounded-lg px-3 text-sm text-destructive hover:bg-muted"
              onclick={() => mail.removeAccount(account.email).catch((e) => mail.fail(e))}
            >
              Remove
            </button>
          </div>
        {:else}
          <p class="text-sm text-muted-foreground">No accounts yet.</p>
        {/each}
        <button
          type="button"
          onclick={add}
          disabled={!(clientId.trim() || mail.settings.builtIn) || mail.signingIn}
          class="flex h-10 w-full items-center justify-center gap-2 rounded-lg bg-primary text-sm font-medium text-primary-foreground disabled:opacity-50"
        >
          <Icon name="UserPlus" class="size-4" />
          {mail.signingIn ? "Finish signing in in your browser…" : "Add Gmail account"}
        </button>
      </section>
    </div>
  </div>
</div>
