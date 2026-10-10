<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";

  /*
   * Email HTML runs in a sandboxed frame with scripts off: `allow-same-origin`
   * only lets this component size the frame and catch link clicks, which open
   * in the browser rather than inside the app. The page CSP (tauri.conf.json)
   * applies to srcdoc frames too, so inline handlers and javascript: URLs are
   * blocked a second time.
   */
  let { html, text }: { html: string | null; text: string | null } = $props();

  let frame = $state<HTMLIFrameElement>();
  let height = $state(80);

  const escape = (s: string) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);
  const linkify = (s: string) => escape(s).replace(/https?:\/\/[^\s<>"]+[^\s<>".,;:!?)\]]/g, (url) => `<a href="${url}">${url}</a>`);

  const base = `
    html { color-scheme: light; }
    body { margin: 0; font: 15px/1.55 "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif; color: #303030;
      overflow-wrap: anywhere; }
    img { max-width: 100%; height: auto; }
    a { color: #005bd3; }
    pre.plain { margin: 0; font: inherit; white-space: pre-wrap; }
    blockquote { margin: 0 0 0 .5em; padding-left: .75em; border-left: 2px solid #e3e3e3; color: #616161; }`;

  const doc = $derived(
    `<!doctype html><html><head><meta charset="utf-8"><base target="_blank"><style>${base}</style></head><body>${
      html ?? `<pre class="plain">${linkify(text ?? "")}</pre>`
    }</body></html>`,
  );

  function onload() {
    const d = frame?.contentDocument;
    if (!d) return;
    const fit = () => (height = Math.max(d.documentElement.scrollHeight, d.body?.scrollHeight ?? 0));
    fit();
    // Images arriving late grow the message.
    new ResizeObserver(fit).observe(d.body);
    d.addEventListener("click", (e) => {
      const link = (e.target as Element | null)?.closest?.("a[href]") as HTMLAnchorElement | null;
      if (!link) return;
      e.preventDefault();
      if (/^(https?|mailto):/i.test(link.href)) openUrl(link.href);
    });
  }
</script>

<iframe
  bind:this={frame}
  title="Message"
  sandbox="allow-same-origin"
  srcdoc={doc}
  {onload}
  class="block w-full border-0 bg-white"
  style:height="{height}px"
></iframe>
