<script lang="ts">
  import Icon from "../icon.svelte";
  import { mail, type Compose } from "../mail.svelte";

  let { draft }: { draft: Compose } = $props();

  // svelte-ignore state_referenced_locally
  let form = $state({ ...draft });
  let sending = $state(false);
  let showCc = $state(!!form.cc);
  let body = $state<HTMLTextAreaElement>();
  let to = $state<HTMLInputElement>();

  $effect(() => {
    // Replies start in the body, above the quote; new messages start at To.
    if (form.to) {
      body?.focus();
      body?.setSelectionRange(0, 0);
    } else to?.focus();
  });

  async function send() {
    if (!form.to.trim() || sending) return;
    sending = true;
    try {
      await mail.send(form);
      mail.compose = null;
    } catch (e) {
      mail.fail(e);
    } finally {
      sending = false;
    }
  }

  const field = "min-w-0 flex-1 bg-transparent py-2.5 text-sm outline-none placeholder:text-muted-foreground";
</script>

<div
  role="presentation"
  class="fixed inset-0 z-40 grid place-items-center bg-black/40 p-8"
  onkeydown={(e) => {
    if (e.key === "Escape") mail.compose = null;
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) send();
  }}
>
  <form
    class="flex h-[min(640px,100%)] w-[min(720px,100%)] flex-col overflow-hidden rounded-2xl bg-background shadow-modal"
    onsubmit={(e) => {
      e.preventDefault();
      send();
    }}
  >
    <header class="flex h-14 shrink-0 items-center gap-3 border-b px-5">
      <h2 class="flex-1 text-base font-semibold">{form.threadId ? "Reply" : "New Message"}</h2>
      <button type="button" aria-label="Close" class="grid size-8 place-items-center rounded-lg hover:bg-muted" onclick={() => (mail.compose = null)}>
        <Icon name="X" class="size-4" />
      </button>
    </header>

    <div class="divide-y border-b px-5">
      {#if mail.accounts.length > 1}
        <label class="flex items-center gap-3">
          <span class="w-14 text-sm text-muted-foreground">From</span>
          <select bind:value={form.account} class="{field} cursor-pointer" disabled={!!form.threadId}>
            {#each mail.accounts as account (account)}<option value={account}>{account}</option>{/each}
          </select>
        </label>
      {/if}
      <label class="flex items-center gap-3">
        <span class="w-14 text-sm text-muted-foreground">To</span>
        <input bind:this={to} bind:value={form.to} class={field} spellcheck="false" placeholder="name@example.com, …" />
        {#if !showCc}
          <button type="button" class="text-sm text-muted-foreground hover:text-foreground" onclick={() => (showCc = true)}>Cc</button>
        {/if}
      </label>
      {#if showCc}
        <label class="flex items-center gap-3">
          <span class="w-14 text-sm text-muted-foreground">Cc</span>
          <input bind:value={form.cc} class={field} spellcheck="false" />
        </label>
      {/if}
      <label class="flex items-center gap-3">
        <span class="w-14 text-sm text-muted-foreground">Subject</span>
        <input bind:value={form.subject} class={field} />
      </label>
    </div>

    <textarea bind:this={body} bind:value={form.body} class="flex-1 resize-none px-5 py-4 text-[15px] leading-relaxed outline-none"></textarea>

    <footer class="flex h-16 shrink-0 items-center justify-between border-t px-5">
      <span class="text-xs text-muted-foreground">Ctrl + Enter to send</span>
      <button
        type="submit"
        disabled={!form.to.trim() || sending}
        class="flex h-9 items-center gap-2 rounded-lg bg-primary px-4 text-sm font-medium text-primary-foreground transition-opacity disabled:opacity-50"
      >
        <Icon name="Send" class="size-4" />
        {sending ? "Sending…" : "Send"}
      </button>
    </footer>
  </form>
</div>
