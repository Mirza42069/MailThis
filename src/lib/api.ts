import { invoke } from "@tauri-apps/api/core";

// Mirrors the #[tauri::command] surface in src-tauri/src/lib.rs (serde camelCase).

export type Addr = { name: string | null; email: string };

export type Settings = { clientId: string; hasSecret: boolean; builtIn: boolean; accounts: Addr[] };

export type Label = { id: string; name: string; system: boolean; unread: number; total: number };

export type ThreadSummary = {
  account: string;
  id: string;
  from: Addr;
  subject: string;
  snippet: string;
  date: number;
  count: number;
  unread: boolean;
  starred: boolean;
  labelIds: string[];
};

export type ThreadPage = { threads: ThreadSummary[]; nextPageToken: string | null };

export type Message = {
  id: string;
  from: Addr;
  replyTo: Addr[];
  to: Addr[];
  cc: Addr[];
  date: number;
  subject: string;
  html: string | null;
  text: string | null;
  messageId: string | null;
  references: string[];
  attachments: string[];
  labelIds: string[];
};

export type Draft = {
  to: string;
  cc: string;
  subject: string;
  body: string;
  threadId?: string;
  inReplyTo?: string;
  references?: string[];
};

export const api = {
  settings: () => invoke<Settings>("settings"),
  setClient: (clientId: string, clientSecret: string) => invoke<void>("set_client", { clientId, clientSecret }),
  addAccount: () => invoke<Addr>("add_account"),
  removeAccount: (account: string) => invoke<void>("remove_account", { account }),
  labels: (account: string) => invoke<Label[]>("labels", { account }),
  threads: (account: string, label: string | null, query: string, pageToken?: string | null) =>
    invoke<ThreadPage>("threads", { account, label, query, pageToken: pageToken ?? null }),
  thread: (account: string, id: string) => invoke<Message[]>("thread", { account, id }),
  modifyThread: (account: string, id: string, add: string[], remove: string[]) =>
    invoke<void>("modify_thread", { account, id, add, remove }),
  modifyMessage: (account: string, id: string, add: string[], remove: string[]) =>
    invoke<void>("modify_message", { account, id, add, remove }),
  trash: (account: string, id: string) => invoke<void>("trash", { account, id }),
  send: (account: string, draft: Draft) => invoke<void>("send", { account, draft }),
};

export const formatAddr = (a: Addr) => (a.name ? `"${a.name.replaceAll('"', "")}" <${a.email}>` : a.email);
export const displayName = (a: Addr) => a.name || a.email;
