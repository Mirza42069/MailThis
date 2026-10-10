# Contributing to MailThis

Thanks for helping. Bug fixes, small features and polish are all welcome.

## Set up

Follow **Requirements** and **Develop** in the [README](README.md). To sign in while developing you need a Google OAuth client of your own: follow **Google setup** there and put it in a `.env` (copied from `.env.example`). `.env` is git-ignored; never commit it.

## Before you open a pull request

```sh
bun run check                                   # svelte-check
cd src-tauri
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the same checks on every pull request.

- Keep a pull request to one change, and say what it does and why. Screenshots help for UI changes.
- Match the surrounding code: its naming, comment style and Tailwind classes.
- Prefer the simplest thing that works. Add a dependency only when it clearly earns its place.
- Icons come from Hugeicons' free set (`@hugeicons/core-free-icons`); add new ones in `src/lib/glyphs.ts`.
- Rust changes to MIME parsing or building in `gmail.rs` should come with a unit test.

## Reporting bugs

Open an issue with the steps to reproduce and your Windows version. Never paste OAuth tokens, client secrets or mail content.

By contributing you agree that your contributions are licensed under the [MIT License](LICENSE).
