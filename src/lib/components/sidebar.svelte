<script lang="ts">
  import type { IconName } from "../glyphs";
  import Icon from "../icon.svelte";
  import { LABEL_NAMES, mail, type Mailbox } from "../mail.svelte";

  /* Rows on the glass frame, a translucent white when hovered or current. */
  let expanded = $state<Record<string, boolean>>({});

  const current = (m: Mailbox) => {
    const c = mail.mailbox;
    if (m.kind !== c.kind) return false;
    return m.kind !== "label" || (c.kind === "label" && c.account === m.account && c.label === m.label);
  };

  const inbox = (account: string): Mailbox => ({ kind: "label", account, label: "INBOX", name: `Inbox · ${account}` });

  const ICONS: Record<string, IconName> = {
    INBOX: "Inbox",
    STARRED: "Star",
    IMPORTANT: "CircleAlert",
    SENT: "Send",
    SPAM: "CircleAlert",
    TRASH: "Trash",
  };

  async function addAccount() {
    if (!mail.canSignIn) return void (mail.settingsOpen = true);
    try {
      await mail.addAccountFlow();
    } catch (e) {
      mail.fail(e);
    }
  }
</script>

{#snippet row(icon: IconName, label: string, active: boolean, onclick: () => void, count?: number)}
  <button
    type="button"
    {onclick}
    class="flex h-9 w-full items-center gap-2.5 rounded-lg px-3 text-left text-sm transition-colors outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring {active
      ? 'bg-muted text-white'
      : ''}"
  >
    <Icon name={icon} class="size-[18px] shrink-0" />
    <span class="min-w-0 flex-1 truncate">{label}</span>
    {#if count}<span class="text-xs text-muted-foreground tabular-nums">{count}</span>{/if}
  </button>
{/snippet}

<aside data-surface="dark" class="flex w-64 shrink-0 flex-col pb-2">
  <nav class="flex-1 overflow-y-auto px-2">
    <h2 class="px-3 pt-3 pb-1.5 text-xs font-medium tracking-wider text-muted-foreground uppercase">Mailboxes</h2>
    {@render row("Inbox", "All Inboxes", current({ kind: "all" }), () => mail.go({ kind: "all" }), mail.inboxUnread)}
    {@render row("Star", "Starred", current({ kind: "starred" }), () => mail.go({ kind: "starred" }), mail.starredTotal)}

    {#if mail.accounts.length}
      <h2 class="px-3 pt-5 pb-1.5 text-xs font-medium tracking-wider text-muted-foreground uppercase">Accounts</h2>
      {#each mail.accounts as account (account)}
        <div class="group relative">
          {@render row("Mail", account, current(inbox(account)), () => mail.go(inbox(account)))}
          <button
            type="button"
            aria-label={expanded[account] ? `Hide ${account} folders` : `Show ${account} folders`}
            aria-expanded={!!expanded[account]}
            onclick={() => (expanded[account] = !expanded[account])}
            class="absolute top-1 right-1 grid size-7 place-items-center rounded-md text-muted-foreground hover:bg-white/10 hover:text-white"
          >
            <Icon name="ChevronDown" class="size-3.5 transition-transform {expanded[account] ? '' : '-rotate-90'}" />
          </button>
        </div>
        {#if expanded[account]}
          <div class="mb-1 ml-4 border-l pl-1.5">
            {#each (mail.labels[account] ?? []).filter((l) => l.id !== "DRAFT") as label (label.id)}
              {@const box: Mailbox = {
                kind: "label",
                account,
                label: label.id,
                name: `${LABEL_NAMES[label.id] ?? label.name} · ${account}`,
              }}
              {@render row(
                ICONS[label.id] ?? "Tag",
                LABEL_NAMES[label.id] ?? label.name,
                current(box),
                () => mail.go(box),
                label.id === "INBOX" || !label.system ? label.unread : undefined,
              )}
            {/each}
          </div>
        {/if}
      {/each}
    {/if}
  </nav>

  <div class="mx-2 border-t pt-2">
    {@render row("Pencil", "New Message", false, () => mail.newMessage())}
    {@render row("UserPlus", "Add Account", false, addAccount)}
    {@render row("SlidersHorizontal", "Settings", false, () => (mail.settingsOpen = true))}
  </div>
</aside>
