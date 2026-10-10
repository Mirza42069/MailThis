<script lang="ts">
  import { displayName, type Message } from "../api";
  import { fullTime } from "../format";
  import Icon from "../icon.svelte";
  import { LABEL_NAMES, mail } from "../mail.svelte";
  import Avatar from "./avatar.svelte";
  import EmailBody from "./email-body.svelte";
  import IconButton from "./icon-button.svelte";

  let quick = $state("");
  let sending = $state(false);
  let moveOpen = $state(false);
  /** Earlier messages in the thread stay collapsed until clicked, as in Mail. */
  let open = $state<Record<string, boolean>>({});

  const thread = $derived(mail.selected);
  const messages = $derived(mail.messages);
  const last = $derived(messages?.at(-1));
  const moveTargets = $derived(
    (thread ? (mail.labels[thread.account] ?? []) : []).filter(
      (l) => !["INBOX", "STARRED", "IMPORTANT", "DRAFT", "SENT", "SPAM", "TRASH"].includes(l.id),
    ),
  );

  $effect(() => {
    void thread?.id;
    open = {};
    quick = "";
    moveOpen = false;
  });

  const expanded = (m: Message) => m === last || open[m.id];
  const recipients = (m: Message) => [...m.to, ...m.cc].map((a) => a.name || a.email).join(", ");

  async function sendQuick() {
    if (!quick.trim() || sending) return;
    sending = true;
    try {
      await mail.quickReply(quick.trim());
      quick = "";
    } catch (e) {
      mail.fail(e);
    } finally {
      sending = false;
    }
  }
</script>

<section class="flex min-w-0 flex-1 flex-col overflow-hidden rounded-xl bg-background shadow-sm">
  <header class="flex h-[72px] shrink-0 items-center gap-2 border-b px-5">
    {#if thread}
      <IconButton icon="Reply" label="Reply (R)" disabled={!last} onclick={() => mail.reply("reply")} />
      <IconButton icon="ReplyAll" label="Reply all" disabled={!last} onclick={() => mail.reply("all")} />
      <IconButton icon="Forward" label="Forward" disabled={!last} onclick={() => mail.reply("forward")} />
      <span class="mx-1.5 h-6 border-l"></span>
      <IconButton icon="Archive" label="Archive (E)" onclick={() => mail.archive(thread)} />
      <IconButton icon="CircleAlert" label="Report spam" onclick={() => mail.spam(thread)} />
      <IconButton icon="Trash" label="Delete (#)" onclick={() => mail.trash(thread)} />
      <div class="relative">
        <IconButton icon="Tag" label="Move to label" active={moveOpen} onclick={() => (moveOpen = !moveOpen)} />
        {#if moveOpen}
          <div
            role="menu"
            class="absolute top-11 left-0 z-20 max-h-80 w-56 overflow-y-auto rounded-xl bg-popover p-1.5 shadow-popover"
          >
            {#each moveTargets as label (label.id)}
              <button
                type="button"
                role="menuitem"
                class="flex h-8 w-full items-center gap-2 rounded-lg px-2.5 text-left text-sm hover:bg-muted"
                onclick={() => ((moveOpen = false), mail.moveTo(thread, label.id))}
              >
                <Icon name="Tag" class="size-4 text-muted-foreground" />
                <span class="truncate">{LABEL_NAMES[label.id] ?? label.name}</span>
              </button>
            {:else}
              <p class="px-2.5 py-2 text-sm text-muted-foreground">No labels yet. Create them in Gmail.</p>
            {/each}
          </div>
        {/if}
      </div>
      <span class="flex-1"></span>
      <IconButton
        icon={thread.starred ? "StarFilled" : "Star"}
        label={thread.starred ? "Unstar (S)" : "Star (S)"}
        active={thread.starred}
        onclick={() => mail.toggleStar(thread)}
      />
      <IconButton icon="Mail" label="Mark as unread" onclick={() => mail.markUnread(thread)} />
    {/if}
  </header>

  {#if !thread}
    <div class="grid flex-1 place-items-center text-sm text-muted-foreground">No conversation selected</div>
  {:else}
    <article class="flex-1 overflow-y-auto px-14 pt-10 pb-6" data-selectable>
      <h1 class="mb-6 text-[28px] leading-tight font-semibold tracking-tight">{thread.subject}</h1>
      {#if !messages}
        <div class="space-y-3">
          <div class="h-11 w-72 animate-pulse rounded-lg bg-muted"></div>
          <div class="h-40 animate-pulse rounded-lg bg-muted"></div>
        </div>
      {:else}
        {#each messages as m (m.id)}
          <div class="border-b pb-6 last:border-0 {expanded(m) ? '' : 'pb-4'} mb-6">
            <button
              type="button"
              class="flex w-full items-start gap-3.5 text-left {expanded(m) ? 'cursor-default' : ''}"
              onclick={() => (open[m.id] = !open[m.id])}
              disabled={m === last}
            >
              <Avatar addr={m.from} size={44} />
              <span class="min-w-0 flex-1">
                <span class="flex flex-wrap items-center gap-2">
                  <span class="text-[15px] font-semibold">{displayName(m.from)}</span>
                  {#if m.from.name}
                    <span class="rounded-full bg-secondary px-2.5 py-0.5 text-sm text-muted-foreground">{m.from.email}</span>
                  {/if}
                </span>
                <span class="mt-0.5 block truncate text-sm text-muted-foreground">
                  {#if expanded(m)}To: {recipients(m)}{:else}{(m.text ?? "").slice(0, 140)}{/if}
                </span>
              </span>
              <span class="shrink-0 pt-1 text-sm text-muted-foreground">{fullTime(m.date)}</span>
            </button>
            {#if expanded(m)}
              <div class="mt-6 border-t pt-6">
                <EmailBody html={m.html} text={m.text} />
                {#if m.attachments.length}
                  <div class="mt-5 flex flex-wrap gap-2">
                    {#each m.attachments as name, i (i)}
                      <span class="flex items-center gap-1.5 rounded-full border px-3 py-1 text-sm text-muted-foreground">
                        <Icon name="Paperclip" class="size-3.5" />{name}
                      </span>
                    {/each}
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      {/if}
    </article>

    <form
      class="shrink-0 px-8 pt-1 pb-5"
      onsubmit={(e) => {
        e.preventDefault();
        sendQuick();
      }}
    >
      <label class="flex h-14 items-center gap-3 rounded-full border bg-background pr-2 pl-6 shadow-sm focus-within:border-ring">
        <input
          bind:value={quick}
          placeholder={last ? `Quick reply to ${displayName(last.from)}…` : "Quick reply…"}
          disabled={!last || sending}
          class="min-w-0 flex-1 bg-transparent text-[15px] outline-none placeholder:text-muted-foreground"
        />
        <button
          type="submit"
          aria-label="Send reply"
          title="Send reply"
          disabled={!quick.trim() || sending}
          class="grid size-10 place-items-center rounded-full bg-secondary text-muted-foreground transition-colors enabled:bg-primary enabled:text-primary-foreground disabled:opacity-60"
        >
          <Icon name="Send" class="size-4" />
        </button>
      </label>
    </form>
  {/if}
</section>
