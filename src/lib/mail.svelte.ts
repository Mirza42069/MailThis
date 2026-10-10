import { api, displayName, formatAddr, type Addr, type Label, type Message, type Settings, type ThreadSummary } from "./api";

export type Mailbox =
  | { kind: "all" }
  | { kind: "starred" }
  | { kind: "label"; account: string; label: string; name: string };

export type Compose = {
  account: string;
  to: string;
  cc: string;
  subject: string;
  body: string;
  threadId?: string;
  inReplyTo?: string;
  references?: string[];
};

export const LABEL_NAMES: Record<string, string> = {
  INBOX: "Inbox",
  STARRED: "Starred",
  IMPORTANT: "Important",
  SENT: "Sent",
  DRAFT: "Drafts",
  SPAM: "Spam",
  TRASH: "Trash",
};

const message = (e: unknown) => (typeof e === "string" ? e : e instanceof Error ? e.message : String(e));

class Mail {
  settings = $state<Settings>({ clientId: "", hasSecret: false, builtIn: false, accounts: [] });
  labels = $state<Record<string, Label[]>>({});
  mailbox = $state<Mailbox>({ kind: "all" });
  query = $state("");
  unreadOnly = $state(false);
  threads = $state<ThreadSummary[]>([]);
  loading = $state(false);
  selected = $state<ThreadSummary | null>(null);
  messages = $state<Message[] | null>(null);
  compose = $state<Compose | null>(null);
  settingsOpen = $state(false);
  notice = $state<string | null>(null);

  /** Next page per account; absent once an account has no more. */
  #pages = $state<Record<string, string>>({});
  /** Drops responses from a list request that a newer one replaced. */
  #generation = 0;

  accounts = $derived(this.settings.accounts.map((a) => a.email));
  /** A Google client is available: the built-in one or one saved in Settings. */
  canSignIn = $derived(this.settings.builtIn || !!this.settings.clientId);

  title = $derived(
    this.mailbox.kind === "all" ? "All Inboxes" : this.mailbox.kind === "starred" ? "Starred" : this.mailbox.name,
  );

  inboxUnread = $derived(this.#count("INBOX", "unread"));
  starredTotal = $derived(this.#count("STARRED", "total"));

  #count(id: string, field: "unread" | "total") {
    return Object.values(this.labels).reduce((sum, labels) => sum + (labels.find((l) => l.id === id)?.[field] ?? 0), 0);
  }

  #scope(): { accounts: string[]; label: string } {
    const m = this.mailbox;
    if (m.kind === "label") return { accounts: [m.account], label: m.label };
    return { accounts: this.accounts, label: m.kind === "all" ? "INBOX" : "STARRED" };
  }

  fail(e: unknown) {
    this.notice = message(e);
    setTimeout(() => (this.notice = null), 6000);
  }

  /** Settings have loaded: until then the window shows neither the sign-in page nor the mail. */
  ready = $state(false);

  async init() {
    this.settings = await api.settings();
    this.ready = true;
    await Promise.all([this.load(), this.refreshLabels()]);
  }

  /** Runs Google sign-in in the browser; resolves once the account is saved. */
  signingIn = $state(false);
  async addAccountFlow() {
    this.signingIn = true;
    try {
      const added = await api.addAccount();
      this.settings = await api.settings();
      await Promise.all([this.refreshLabels(), this.load()]);
      return added;
    } finally {
      this.signingIn = false;
    }
  }

  async removeAccount(account: string) {
    await api.removeAccount(account);
    this.settings = await api.settings();
    delete this.labels[account];
    if (this.mailbox.kind === "label" && this.mailbox.account === account) this.mailbox = { kind: "all" };
    this.selected = null;
    this.messages = null;
    await this.load();
  }

  async refreshLabels() {
    const entries = await Promise.all(
      this.accounts.map(async (a) => [a, await api.labels(a).catch(() => this.labels[a] ?? [])] as const),
    );
    this.labels = Object.fromEntries(entries);
  }

  /** Reloads the list; `quiet` keeps the current rows on screen while it does (the background poll). */
  async load(quiet = false) {
    const generation = ++this.#generation;
    const { accounts, label } = this.#scope();
    const q = [this.query.trim(), this.unreadOnly ? "is:unread" : ""].filter(Boolean).join(" ");
    if (!quiet) this.loading = true;
    try {
      const pages = await Promise.all(
        accounts.map(async (account) => [account, await api.threads(account, label, q)] as const),
      );
      if (generation !== this.#generation) return;
      this.#pages = Object.fromEntries(pages.flatMap(([a, p]) => (p.nextPageToken ? [[a, p.nextPageToken]] : [])));
      this.threads = pages.flatMap(([, p]) => p.threads).sort((a, b) => b.date - a.date);
      if (this.selected && !this.threads.some((t) => this.#same(t, this.selected!)) && !quiet) this.selected = null;
    } catch (e) {
      if (generation === this.#generation) this.fail(e);
    } finally {
      if (generation === this.#generation) this.loading = false;
    }
  }

  get hasMore() {
    return Object.keys(this.#pages).length > 0;
  }

  async more() {
    if (this.loading || !this.hasMore) return;
    const generation = this.#generation;
    const { label } = this.#scope();
    const q = [this.query.trim(), this.unreadOnly ? "is:unread" : ""].filter(Boolean).join(" ");
    this.loading = true;
    try {
      const pages = await Promise.all(
        Object.entries(this.#pages).map(async ([account, token]) => [account, await api.threads(account, label, q, token)] as const),
      );
      if (generation !== this.#generation) return;
      this.#pages = Object.fromEntries(pages.flatMap(([a, p]) => (p.nextPageToken ? [[a, p.nextPageToken]] : [])));
      const seen = new Set(this.threads.map((t) => `${t.account}/${t.id}`));
      const added = pages.flatMap(([, p]) => p.threads).filter((t) => !seen.has(`${t.account}/${t.id}`));
      this.threads = [...this.threads, ...added].sort((a, b) => b.date - a.date);
    } catch (e) {
      this.fail(e);
    } finally {
      this.loading = false;
    }
  }

  go(mailbox: Mailbox) {
    this.mailbox = mailbox;
    this.selected = null;
    this.messages = null;
    this.threads = [];
    this.load();
  }

  #same(a: ThreadSummary, b: ThreadSummary) {
    return a.account === b.account && a.id === b.id;
  }

  #patch(thread: ThreadSummary, change: Partial<ThreadSummary>) {
    this.threads = this.threads.map((t) => (this.#same(t, thread) ? { ...t, ...change } : t));
    if (this.selected && this.#same(this.selected, thread)) this.selected = { ...this.selected, ...change };
  }

  async open(thread: ThreadSummary) {
    this.selected = thread;
    this.messages = null;
    try {
      const messages = await api.thread(thread.account, thread.id);
      if (this.selected && this.#same(this.selected, thread)) this.messages = messages;
      if (thread.unread) {
        this.#patch(thread, { unread: false });
        await api.modifyThread(thread.account, thread.id, [], ["UNREAD"]);
        this.refreshLabels();
      }
    } catch (e) {
      this.fail(e);
    }
  }

  step(by: 1 | -1) {
    const index = this.selected ? this.threads.findIndex((t) => this.#same(t, this.selected!)) : -1;
    const next = this.threads[Math.min(Math.max(index + by, 0), this.threads.length - 1)];
    if (next) this.open(next);
  }

  /** Takes the thread out of this list, opening the one after it, then tells Gmail. */
  async #leave(thread: ThreadSummary, action: () => Promise<void>) {
    const index = this.threads.findIndex((t) => this.#same(t, thread));
    this.threads = this.threads.filter((t) => !this.#same(t, thread));
    const next = this.threads[Math.min(index, this.threads.length - 1)];
    if (this.selected && this.#same(this.selected, thread)) {
      if (next) this.open(next);
      else (this.selected = null), (this.messages = null);
    }
    try {
      await action();
      this.refreshLabels();
    } catch (e) {
      this.fail(e);
      this.load(true);
    }
  }

  archive(t: ThreadSummary) {
    return this.#leave(t, () => api.modifyThread(t.account, t.id, [], ["INBOX"]));
  }

  spam(t: ThreadSummary) {
    return this.#leave(t, () => api.modifyThread(t.account, t.id, ["SPAM"], ["INBOX"]));
  }

  trash(t: ThreadSummary) {
    return this.#leave(t, () => api.trash(t.account, t.id));
  }

  /** Move = file under the label and take it out of the inbox, as Gmail's own "Move to" does. */
  moveTo(t: ThreadSummary, label: string) {
    return this.#leave(t, () => api.modifyThread(t.account, t.id, [label], ["INBOX"]));
  }

  async toggleStar(t: ThreadSummary) {
    const starred = !t.starred;
    this.#patch(t, { starred });
    try {
      if (starred) {
        // Gmail stars a message, not a thread: star the newest one.
        const last = this.messages?.at(-1);
        if (last && this.selected && this.#same(this.selected, t)) await api.modifyMessage(t.account, last.id, ["STARRED"], []);
        else await api.modifyThread(t.account, t.id, ["STARRED"], []).catch(() => {});
      } else {
        await api.modifyThread(t.account, t.id, [], ["STARRED"]);
      }
      if (!starred && this.mailbox.kind === "starred") this.threads = this.threads.filter((x) => !this.#same(x, t));
      this.refreshLabels();
    } catch (e) {
      this.#patch(t, { starred: !starred });
      this.fail(e);
    }
  }

  async markUnread(t: ThreadSummary) {
    this.#patch(t, { unread: true });
    if (this.selected && this.#same(this.selected, t)) (this.selected = null), (this.messages = null);
    try {
      await api.modifyThread(t.account, t.id, ["UNREAD"], []);
      this.refreshLabels();
    } catch (e) {
      this.fail(e);
    }
  }

  newMessage() {
    const account = this.mailbox.kind === "label" ? this.mailbox.account : this.accounts[0];
    if (!account) return void (this.settingsOpen = true);
    this.compose = { account, to: "", cc: "", subject: "", body: "" };
  }

  reply(mode: "reply" | "all" | "forward", body = "") {
    const t = this.selected;
    const m = this.messages?.at(-1);
    if (!t || !m) return;
    const me = t.account.toLowerCase();
    const notMe = (a: Addr) => a.email.toLowerCase() !== me;
    const fromMe = !notMe(m.from);
    const target = fromMe ? m.to : m.replyTo.length ? m.replyTo : [m.from];
    const subject = mode === "forward" ? prefix("Fwd:", m.subject) : prefix("Re:", m.subject);
    const original = m.text ?? (m.html ? htmlToText(m.html) : "");
    const when = new Date(m.date).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });

    if (mode === "forward") {
      const header = [
        "---------- Forwarded message ---------",
        `From: ${formatAddr(m.from)}`,
        `Date: ${when}`,
        `Subject: ${m.subject}`,
        `To: ${m.to.map(formatAddr).join(", ")}`,
      ].join("\n");
      this.compose = { account: t.account, to: "", cc: "", subject, body: `${body}\n\n${header}\n\n${original}` };
      return;
    }
    const to = mode === "all" ? unique([...target, ...m.to.filter(notMe)]) : target;
    const cc = mode === "all" ? unique(m.cc.filter(notMe)).filter((c) => !to.some((x) => x.email === c.email)) : [];
    const quote = original
      .trimEnd()
      .split("\n")
      .map((line) => `> ${line}`)
      .join("\n");
    this.compose = {
      account: t.account,
      to: to.map(formatAddr).join(", "),
      cc: cc.map(formatAddr).join(", "),
      subject,
      body: `${body}\n\nOn ${when}, ${displayName(m.from)} wrote:\n${quote}`,
      threadId: t.id,
      inReplyTo: m.messageId ?? undefined,
      references: m.references,
    };
  }

  /** The quick reply bar: a reply to the sender, sent straight away. */
  async quickReply(text: string) {
    this.reply("reply", text);
    const draft = this.compose;
    this.compose = null;
    if (draft) await this.send(draft);
  }

  async send(draft: Compose) {
    const { account, ...rest } = draft;
    await api.send(account, rest);
    this.notice = "Sent";
    setTimeout(() => (this.notice = null), 2500);
    if (this.selected && draft.threadId === this.selected.id) this.open(this.selected);
  }
}

const prefix = (p: string, subject: string) =>
  subject.toLowerCase().startsWith(p.toLowerCase()) ? subject : `${p} ${subject}`;

const unique = (list: Addr[]) => list.filter((a, i) => list.findIndex((b) => b.email === a.email) === i);

const htmlToText = (html: string) => new DOMParser().parseFromString(html, "text/html").body.innerText ?? "";

export const mail = new Mail();
