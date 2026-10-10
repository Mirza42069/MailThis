<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  import type { IconName } from "../glyphs";
  import Icon from "../icon.svelte";

  /* The window is undecorated (tauri.conf.json), so the frame draws its own caption buttons. */
  const win = getCurrentWindow();
  const buttons: { icon: IconName; label: string; run: () => Promise<void>; danger?: boolean }[] = [
    { icon: "WinMinimize", label: "Minimize", run: () => win.minimize() },
    { icon: "WinMaximize", label: "Maximize", run: () => win.toggleMaximize() },
    { icon: "X", label: "Close", run: () => win.close(), danger: true },
  ];
</script>

<div class="flex h-full">
  {#each buttons as b (b.label)}
    <button
      type="button"
      aria-label={b.label}
      title={b.label}
      onclick={b.run}
      class="grid h-full w-11 place-items-center text-muted-foreground transition-colors hover:text-white {b.danger
        ? 'hover:bg-[#c42b1c]'
        : 'hover:bg-white/10'}"
    >
      <Icon name={b.icon} class="size-4" />
    </button>
  {/each}
</div>
