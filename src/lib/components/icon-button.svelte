<script lang="ts">
  import type { IconName } from "../glyphs";
  import Icon from "../icon.svelte";

  /* The round outlined buttons from the reference's toolbars. */
  let {
    icon,
    label,
    onclick,
    active = false,
    disabled = false,
    spin = false,
  }: { icon: IconName; label: string; onclick?: (e: MouseEvent) => void; active?: boolean; disabled?: boolean; spin?: boolean } =
    $props();
</script>

<button
  type="button"
  title={label}
  aria-label={label}
  aria-pressed={active || undefined}
  {disabled}
  {onclick}
  class="grid size-9 shrink-0 place-items-center rounded-full border text-foreground shadow-[0_1px_0_0_rgb(26_26_26/0.04)] transition-colors outline-none hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-40 {active
    ? 'bg-secondary'
    : 'bg-background'}"
>
  <Icon name={icon} class="size-[18px] {spin ? 'animate-spin' : ''}" />
</button>
