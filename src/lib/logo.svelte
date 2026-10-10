<script lang="ts">
  /* The MailThis mark in white: an envelope whose flap is the middle of an "M". Also drawn in assets/icon.svg. */
  let { class: className = "" }: { class?: string } = $props();
</script>

<svg
  viewBox="0 0 32 24"
  class={className}
  role="img"
  aria-label="MailThis"
  fill="none"
  stroke="#ffffff"
  stroke-width="3"
  stroke-linecap="round"
  stroke-linejoin="round"
>
  <rect x="1.5" y="1.5" width="29" height="21" rx="4.5" />
  <path d="M8 17V7.5L16 13.5L24 7.5V17" />
</svg>
