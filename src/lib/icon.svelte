<script lang="ts">
  import type { SVGAttributes } from "svelte/elements";

  import { glyphs, type IconName } from "./glyphs";

  /*
   * aria-hidden by default, before the spread so an icon that needs its own
   * name can pass aria-hidden={false} with an aria-label. Size is left to the
   * call site's class or the parent's `[&_svg]:size-*` rule.
   */
  let {
    name,
    strokeWidth = 2,
    ...rest
  }: Omit<SVGAttributes<SVGSVGElement>, "stroke-width"> & { name: IconName; strokeWidth?: number } = $props();

  const filled = $derived(name.endsWith("Filled"));
  const kebab = (key: string) => key.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);

  /** The package's camelCase React-style attributes as SVG ones, with our stroke width. */
  const elements = $derived(
    glyphs[name].map(([tag, { key, strokeWidth: _, ...attrs }]) => ({
      tag,
      key: String(key),
      attrs: {
        ...Object.fromEntries(Object.entries(attrs).map(([k, v]) => [kebab(k), String(v)])),
        "stroke-width": String(strokeWidth),
        ...(filled && { fill: "currentColor" }),
      },
    })),
  );
</script>

<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" aria-hidden="true" {...rest}>
  {#each elements as el (el.key)}
    <svelte:element this={el.tag} xmlns="http://www.w3.org/2000/svg" {...el.attrs} />
  {/each}
</svg>
