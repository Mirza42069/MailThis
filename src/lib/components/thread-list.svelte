<script lang="ts">
  import { displayName } from "../api";
  import { listTime } from "../format";
  import Icon from "../icon.svelte";
  import { mail } from "../mail.svelte";
  import Avatar from "./avatar.svelte";
  import IconButton from "./icon-button.svelte";

  let search = $state(mail.query);
  let timer: ReturnType<typeof setTimeout>;

  function onSearch() {
    clearTimeout(timer);
    timer = setTimeout(() => {
      mail.query = search;
      mail.load();
    }, 400);
  }

  function onScroll(e: Event) {
    const el = e.currentTarget as HTMLElement;
    if (el.scrollHeight - el.scrollTop - el.clientHeight < 400) mail.more();
  }

  const isSelected = (account: string, id: string) => mail.selected?.account === account && mail.selected?.id === id;

  $effect(() => {
    // Keep the selection in view when it moves by keyboard.
    if (!mail.selected) return;
    document.querySelector(`[data-thread="${CSS.escape(`${mail.selected.account}/${mail.selected.id}`)}"]`)?.scrollIntoView({
      block: "nearest",
    });
  });
</script>

<section class="flex w-[420px] shrink-0 flex-col overflow-hidden rounded-xl bg-background shadow-sm">
  <header class="flex h-[72px] shrink-0 items-center gap-2 border-b px-5">
    <h1 class="min-w-0 flex-1 truncate text-xl font-semibold tracking-tight" title={mail.title}>{mail.title}</h1>
    <IconButton icon="RefreshCw" label="Refresh" spin={mail.loading} onclick={() => (mail.load(), mail.refreshLabels())} />
    <IconButton
      icon="Mail"
      label={mail.unreadOnly ? "Show all" : "Unread only"}
      active={mail.unreadOnly}
      onclick={() => ((mail.unreadOnly = !mail.unreadOnly), mail.load())}
    />
    <IconButton icon="Pencil" label="New Message" onclick={() => mail.newMessage()} />
  </header>

  <div class="shrink-0 px-5 pt-4 pb-2">
    <label class="flex h-10 items-center gap-2.5 rounded-full border px-4 focus-within:border-ring">
      <Icon name="Search" class="size-4 shrink-0 text-muted-foreground" />
      <input
        bind:value={search}
        oninput={onSearch}
        onkeydown={(e) => e.key === "Escape" && ((search = ""), onSearch())}
        placeholder="Search"
        spellcheck="false"
        class="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
      />
    </label>
  </div>

  <ul class="flex-1 overflow-y-auto px-3 pb-3" onscroll={onScroll}>
    {#each mail.threads as t (`${t.account}/${t.id}`)}
      {@const selected = isSelected(t.account, t.id)}
      <li data-thread="{t.account}/{t.id}" class="group">
        <button
          type="button"
          onclick={() => mail.open(t)}
          class="relative flex w-full gap-3 rounded-xl px-3 py-3 text-left outline-none focus-visible:ring-2 focus-visible:ring-ring {selected
            ? 'bg-secondary'
            : 'hover:bg-muted'}"
        >
          {#if t.unread}
            <span class="absolute top-[22px] left-0.5 size-2 rounded-full bg-[#005bd3]" aria-label="Unread"></span>
          {/if}
          <Avatar addr={t.from} />
          <span class="min-w-0 flex-1">
            <span class="flex items-baseline gap-2">
              <span class="min-w-0 truncate text-[15px] {t.unread ? 'font-semibold' : 'font-medium'}">{displayName(t.from)}</span>
              {#if t.count > 1}
                <span class="shrink-0 rounded-full bg-secondary px-1.5 text-xs leading-5 text-muted-foreground tabular-nums">{t.count}</span>
              {/if}
              {#if t.starred}<Icon name="StarFilled" class="size-3.5 shrink-0 self-center text-[#e5a400]" />{/if}
              <span class="ml-auto shrink-0 text-sm text-muted-foreground tabular-nums">{listTime(t.date)}</span>
            </span>
            <span class="block truncate text-[15px] {t.unread ? 'font-medium' : ''}">{t.subject}</span>
            <span class="block truncate text-sm text-muted-foreground">{t.snippet}</span>
            {#if mail.accounts.length > 1 || mail.mailbox.kind !== "label"}
              <span class="mt-0.5 block truncate text-xs font-medium text-muted-foreground/90">{t.account}</span>
            {/if}
          </span>
        </button>
        <div class="mx-3 border-b group-last:hidden {selected ? 'invisible' : ''}"></div>
      </li>
    {:else}
      <li class="grid h-full place-items-center px-6 text-center text-sm text-muted-foreground">
        {#if mail.loading}
          Loading…
        {:else if !mail.accounts.length}
          Add a Gmail account to get started.
        {:else if mail.query || mail.unreadOnly}
          No conversations match.
        {:else}
          Nothing here.
        {/if}
      </li>
    {/each}
    {#if mail.threads.length && mail.loading}
      <li class="py-3 text-center text-xs text-muted-foreground">Loading…</li>
    {/if}
  </ul>
</section>
