<script lang="ts">
  import ComposeDialog from "./lib/components/compose-dialog.svelte";
  import Reader from "./lib/components/reader.svelte";
  import SettingsDialog from "./lib/components/settings-dialog.svelte";
  import Sidebar from "./lib/components/sidebar.svelte";
  import SignIn from "./lib/components/sign-in.svelte";
  import ThreadList from "./lib/components/thread-list.svelte";
  import WindowControls from "./lib/components/window-controls.svelte";
  import Icon from "./lib/icon.svelte";
  import Logo from "./lib/logo.svelte";
  import { mail } from "./lib/mail.svelte";

  let sidebarOpen = $state(true);
  const signedIn = $derived(mail.accounts.length > 0);

  $effect(() => {
    mail.init().catch((e) => mail.fail(e));
    // New mail without a push channel: poll quietly.
    const poll = setInterval(() => {
      if (document.visibilityState === "visible" && !mail.compose) (mail.load(true), mail.refreshLabels());
    }, 60_000);
    return () => clearInterval(poll);
  });

  function onkeydown(e: KeyboardEvent) {
    const target = e.target as HTMLElement;
    if (!signedIn || mail.compose || mail.settingsOpen || e.ctrlKey || e.metaKey || e.altKey) return;
    if (target.closest("input, textarea, select, [contenteditable]")) return;
    const t = mail.selected;
    const actions: Record<string, () => unknown> = {
      j: () => mail.step(1),
      ArrowDown: () => mail.step(1),
      k: () => mail.step(-1),
      ArrowUp: () => mail.step(-1),
      c: () => mail.newMessage(),
      ...(t && {
        e: () => mail.archive(t),
        "#": () => mail.trash(t),
        Delete: () => mail.trash(t),
        s: () => mail.toggleStar(t),
        r: () => mail.reply("reply"),
        a: () => mail.reply("all"),
        f: () => mail.reply("forward"),
        u: () => mail.markUnread(t),
      }),
    };
    const run = actions[e.key];
    if (run) {
      e.preventDefault();
      run();
    }
  }
</script>

<svelte:window {onkeydown} />

<!--
  The window is acrylic (tauri.conf.json); the frame tints it, so the title strip,
  the sidebar and the sign-in page are glass. The mail itself sits on solid black.
-->
<div class="flex h-full flex-col bg-sidebar/55">
  <!-- The frame's title strip: a drag region, as the window has no native title bar. -->
  <div data-surface="dark" data-tauri-drag-region class="flex h-8 shrink-0 items-center">
    <!-- As wide as the sidebar while it is open, so the toggle sits at its top-right corner. -->
    <div
      data-tauri-drag-region
      class="flex h-full shrink-0 items-center gap-3 pl-5 {signedIn && sidebarOpen ? 'w-64 pr-2' : ''}"
    >
      <div class="pointer-events-none flex items-center gap-2.5">
        <Logo class="h-3.5 w-auto" />
        <span class="text-sm font-semibold tracking-tight">MailThis</span>
      </div>
      {#if signedIn}
        <button
          type="button"
          aria-label={sidebarOpen ? "Hide sidebar" : "Show sidebar"}
          title={sidebarOpen ? "Hide sidebar" : "Show sidebar"}
          onclick={() => (sidebarOpen = !sidebarOpen)}
          class="grid size-7 place-items-center rounded-md text-muted-foreground outline-none hover:bg-muted hover:text-white focus-visible:ring-2 focus-visible:ring-ring {sidebarOpen
            ? 'ml-auto'
            : ''}"
        >
          <Icon name="PanelLeft" class="size-4" />
        </button>
      {/if}
    </div>
    <div data-tauri-drag-region class="h-full flex-1"></div>
    <WindowControls />
  </div>

  {#if !mail.ready}
    <!-- Settings are still loading: just the glass, so neither page flashes. -->
  {:else if !signedIn}
    <SignIn />
  {:else}
    <div class="flex min-h-0 flex-1">
      {#if sidebarOpen}<Sidebar />{/if}
      <div class="flex min-w-0 flex-1 gap-2 bg-sidebar p-2 {sidebarOpen ? 'rounded-tl-xl' : ''}">
        <ThreadList />
        <Reader />
      </div>
    </div>
  {/if}
</div>

{#if mail.compose}
  {#key mail.compose}<ComposeDialog draft={mail.compose} />{/key}
{/if}
{#if mail.settingsOpen}<SettingsDialog />{/if}

{#if mail.notice}
  <div
    role="status"
    class="fixed bottom-6 left-1/2 z-50 max-w-[min(560px,90vw)] -translate-x-1/2 rounded-xl bg-[#1a1a1a] px-4 py-2.5 text-sm text-white shadow-modal"
  >
    {mail.notice}
  </div>
{/if}
