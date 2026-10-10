# MailThis

[![CI](https://github.com/Mirza42069/MailThis/actions/workflows/ci.yml/badge.svg)](https://github.com/Mirza42069/MailThis/actions/workflows/ci.yml)

A Gmail desktop client for Windows, written in Rust with [Tauri 2](https://tauri.app). The UI is Svelte 5 + Tailwind v4: the sign-in page, title bar and sidebar are dark acrylic glass, and the mail sits in white panels on black, set in Inter with [Hugeicons](https://hugeicons.com) (free set, MIT).

- **All Inboxes** across several Gmail accounts, **Starred**, and every Gmail folder and label per account
- Gmail search (the same syntax as gmail.com), an unread-only filter, and infinite scroll
- Conversation view: HTML mail in a sandboxed frame (scripts off, links open in your browser) and attachment names
- Reply, Reply all, Forward, quick reply, and New Message (plain text, threaded with `In-Reply-To`/`References`)
- Archive, Spam, Delete, Move to label, Star, Mark unread
- Keyboard: `j`/`k` or ↑/↓ to move, `e` archive, `#`/Del delete, `s` star, `r` reply, `a` reply all, `f` forward, `u` unread, `c` compose, `Ctrl+Enter` send

## Layout

```
src/                 Svelte UI
  app.css            theme tokens and the glass frame
  lib/api.ts         typed wrappers for the Tauri commands
  lib/mail.svelte.ts app state: mailboxes, list, selection, actions
  lib/components/    sign-in, sidebar, thread list, reader, compose, settings, window controls
src-tauri/src/
  lib.rs             commands, config and token cache
  build.rs           bakes the Google client from .env into the app
  oauth.rs           Google sign-in (PKCE, loopback redirect)
  gmail.rs           Gmail REST client, MIME parsing and building (+ unit tests)
assets/icon.svg      app icon source (`bun run tauri icon assets/icon.svg`)
```

## Requirements

- [Bun](https://bun.sh)
- Rust stable, MSVC toolchain: `winget install Rustlang.Rustup`
- Visual Studio Build Tools with **Desktop development with C++**: `winget install Microsoft.VisualStudio.2022.BuildTools`
- WebView2 (included with Windows 11)

## Google setup (once, when building)

MailThis talks to Gmail through a Google OAuth client that is compiled into the app, so people only choose **Sign in with Google**. No server sits in between, so nobody else ever sees the mail.

1. Open the [Google Cloud console](https://console.cloud.google.com/) and create a project.
2. **APIs & Services → Library**: enable the **Gmail API**.
3. **Google Auth Platform → Branding**: fill in an app name and your email. **Audience**: leave it as *External / Testing* and add every Gmail address that will sign in as a **test user**.
4. **Data Access**: add the scope `https://www.googleapis.com/auth/gmail.modify`.
5. **Clients → Create client**: choose application type **Desktop app**. Copy the client ID and client secret.
6. Copy `.env.example` to `.env` (git-ignored) and fill in `MAILTHIS_GOOGLE_CLIENT_ID` and `MAILTHIS_GOOGLE_CLIENT_SECRET`. Environment variables with the same names work too, for example in CI. Rebuild.

Then choose **Sign in with Google** in MailThis and finish the consent screen in your browser. In Testing mode Google warns that the app is unverified: choose *Continue*.

A build without a client, or anyone who prefers their own, can paste a client into **Settings** instead (*Use your own Google client* on the sign-in page). A saved client takes precedence over the built-in one.

Where things are stored:

| What | Where |
| --- | --- |
| Your own OAuth client (if set), account list | `%APPDATA%\app.mailthis.desktop\config.json` |
| Refresh tokens | Windows Credential Manager, service `MailThis` |
| Mail | Nowhere on disk: always read live from Gmail |

Testing-mode refresh tokens expire after 7 days. When that happens MailThis asks you to sign in again: remove the account and add it back in Settings. Publishing the consent screen to *In production* avoids the expiry, but Google then asks you to verify the app.

## Develop

```sh
bun install
bun run tauri dev      # Vite on :1420 + the Rust app with hot reload
bun run check          # svelte-check
cd src-tauri && cargo test && cargo clippy
bun run tauri build    # NSIS installer in src-tauri/target/release/bundle/nsis
```

## Contributing

Contributions are welcome: see [CONTRIBUTING.md](CONTRIBUTING.md). MailThis is released under the [MIT License](LICENSE).
