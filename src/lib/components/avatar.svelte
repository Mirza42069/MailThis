<script lang="ts">
  import type { Addr } from "../api";
  import { initials } from "../format";

  /*
   * The sender's site icon, as in the reference: Google's favicon service for the
   * address's domain. Personal mailboxes, and domains without an icon (the
   * service answers those with a 16px globe), get coloured initials instead.
   */
  let { addr, size = 32 }: { addr: Addr; size?: number } = $props();

  const PERSONAL = /^(gmail|googlemail|outlook|hotmail|live|yahoo|icloud|me|proton|protonmail|aol)\./;
  const COLORS = ["#2563eb", "#7c3aed", "#db2777", "#ea580c", "#16a34a", "#0891b2", "#4f46e5", "#ca8a04"];

  const domain = $derived.by(() => {
    const parts = (addr.email.split("@")[1] ?? "").toLowerCase().split(".");
    // Keep "co.uk"-style suffixes whole.
    const keep = parts.length > 2 && parts.at(-2)!.length <= 3 && parts.at(-1)!.length === 2 ? 3 : 2;
    return parts.slice(-keep).join(".");
  });
  const color = $derived(COLORS[[...addr.email].reduce((h, c) => h + c.charCodeAt(0), 0) % COLORS.length]);
  let failed = $state(false);
  const icon = $derived(!failed && domain && !PERSONAL.test(domain));
</script>

<span
  class="relative grid shrink-0 place-items-center overflow-hidden rounded-full bg-white text-[11px] font-semibold text-white ring-1 ring-black/10"
  style:width="{size}px"
  style:height="{size}px"
  style:background={icon ? undefined : color}
  style:font-size="{Math.round(size * 0.36)}px"
>
  {#if icon}
    <img
      src="https://www.google.com/s2/favicons?domain={domain}&sz=64"
      alt=""
      class="size-[70%]"
      onload={(e) => (failed = (e.currentTarget as HTMLImageElement).naturalWidth < 32)}
      onerror={() => (failed = true)}
    />
  {:else}
    {initials(addr.name || addr.email)}
  {/if}
</span>
