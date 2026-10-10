const sameDay = (a: Date, b: Date) => a.toDateString() === b.toDateString();

/** List timestamps, as Mail does: the time today, then Yesterday, the weekday this week, else the date. */
export function listTime(ms: number) {
  const date = new Date(ms);
  const now = new Date();
  if (sameDay(date, now)) return date.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (sameDay(date, yesterday)) return "Yesterday";
  if (now.getTime() - ms < 6 * 86_400_000) return date.toLocaleDateString(undefined, { weekday: "long" });
  return date.toLocaleDateString(undefined, { day: "numeric", month: "short", year: date.getFullYear() === now.getFullYear() ? undefined : "2-digit" });
}

/** "Oct 8, 2026 at 19:04". */
export function fullTime(ms: number) {
  const date = new Date(ms);
  return `${date.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" })} at ${date.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" })}`;
}

export function initials(name: string) {
  const words = name.replace(/[^\p{L}\p{N} ]/gu, " ").trim().split(/\s+/);
  return ((words[0]?.[0] ?? "?") + (words.length > 1 ? (words.at(-1)?.[0] ?? "") : "")).toUpperCase();
}
