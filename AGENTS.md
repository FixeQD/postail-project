# AGENTS.md

## Project Overview

**Postail** — modern, privacy-focused multiplatform email client (GPLv3). Desktop app built with **Tauri 2** (Rust backend) and **React 19** (TypeScript frontend). All data stored in a **SQLCipher-encrypted SQLite** database; security stack supports Argon2 passphrase, TPM 2.0 (Linux/Windows) and OS keyring.

- Frontend: React 19 + TypeScript + Vite 7 + Tailwind 4 + Zustand + TanStack Query + Framer Motion + i18next (EN resources only)
- Backend: Rust (edition 2024), Tauri 2, tokio, rusqlite/SQLCipher (r2d2 pool), async IMAP, SMTP (lettre), OAuth2/PKCE
- Package managers: **bun** (frontend), **cargo** (Rust workspace)
- Identifier: `com.fixeq.postail`, version 0.1.0

## graphify (knowledge graph)

This project has a knowledge graph at `graphify-out/` with god nodes, community structure, and cross-file relationships. The folder is committed to the repo (currently also gitignored — commit with `git add -f` if needed).

Rules:
- For codebase questions, first run `graphify query "<question>"` when `graphify-out/graph.json` exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts. These return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw grep output.
- Dirty graphify-out/ files are expected after hooks or incremental updates; dirty graph files are not a reason to skip graphify. Only skip graphify if the task is about stale or incorrect graph output, or the user explicitly says not to use it.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).

## Architecture

Three layers:

1. **`src/` — React frontend.** State machine in `src/App.tsx` (`currentState`: init/welcome/data-dir/customize/security/argon2-setup/recovery-setup/argon2-unlock/tpm-unlock-failed/dashboard/settings/accounts/contacts/calendar/reencrypt). Zustand stores in `src/stores/`, hooks in `src/hooks/`, i18n in `src/i18n/`, types barrel in `src/types/`.
2. **`src-tauri/` — Rust backend.** ~140 Tauri commands registered in `src-tauri/src/lib.rs` (`invoke_handler`). Modules: `cmd/` (IPC boundary), `db/` (SQLCipher + r2d2), `imap/` (sync loop: idle/poll), `smtp/` (outbox worker), `oauth/` (PKCE flow), `security/` (lock settings/timer), `email_view.rs` (HTML view preparation), `protocol.rs` (custom `postail://` URI scheme handler). Shared singletons in `globals.rs` (`DB_CONN`, `SECURITY`, `CRYPTO_ACTOR`, `IMAP_MANAGER`, `SMTP_MANAGER`).
3. **`crates/` — shared libraries** (Cargo workspace, resolver 2):
   - `postail_security` — SecurityManager, MasterKey, Crypto actor, Argon2/TPM/keyring storage tiers, app lock
   - `html_transpiler` — email-safe HTML transpiler (flexbox/grid → tables) + Ammonia sanitization; public API: `auto_fix_email_html`, `sanitize_email_html(_with_details)`
   - `email_webview` — embedded WebView2/WebKitGTK email preview with strict network policy (null-proxy 502) and watchdog
   - `email_network` — resource cache (encrypted) and external-resource rewriting

Frontend ↔ backend: Tauri IPC (`invoke` from `src/lib/tauri.ts`), `postail://` scheme, and backend-emitted events (sync status, messages).

## Setup

```sh
# Nix dev shell (recommended — provides GTK/WebKit/TPM system deps, Rust toolchain, bun, cargo-tauri)
nix develop .#default

# Windows cross-compile shell
nix develop .#windows-cross

# Outside Nix, frontend deps only:
bun install
```

Required system deps (non-Nix): glib, gtk3, webkit2gtk-4.1, libsoup3, cairo, pango, gdk-pixbuf, librsvg, dbus, openssl, sqlite, libayatana-appindicator, tpm2-tss.

## Development Workflow

```sh
# Frontend only (Vite dev server, port 1420)
bun run dev

# Full Tauri app (backend + frontend, hot reload via beforeDevCommand)
bun run tauri dev
```

- `tauri.conf.json` wires `bun run dev` / `bun run build` into Tauri's dev/build pipeline.
- Tauri capabilities/permissions: `src-tauri/capabilities/default.json` (schemas generated into `src-tauri/gen/schemas/`).
- TPM feature is default-on: `[features] default = ["tpm"]` in `src-tauri/Cargo.toml`; the crate `postail_security` gates TPM behind its own `tpm` feature.
- Rebuild the graph after code changes: `graphify update .`
- **Read actual source before generating code.** To avoid hallucinating APIs, types, or function signatures, always check real files: `node_modules/` for frontend deps, `$HOME/.cargo/registry/src/index.crates.io-*/` for Rust crates. Never guess a library's API — read it first.

## Testing

```sh
# All workspace tests (frontend has no test suite)
cargo test --workspace

# Default members only (CI behavior — src-tauri only)
cd src-tauri && cargo test

# Per crate / targeted
cargo test -p postail -p html_transpiler
cargo test -p postail_security --features tpm
cargo test -p html_transpiler --test flexbox
```

- Backend integration tests: `src-tauri/tests/` (db, imap_flags, mdn, mime_builder, oauth, recovery)
- Crate tests: `crates/html_transpiler/tests/` (auto_fix, clamp, flexbox, html_sanitization, opacity_bug, parse_css_value, style_sanitization), `crates/postail_security/tests/security.rs`
- SQLCipher tests require the encryption setup to run (key derivation is part of the code under test).
- CI only runs `cargo test` (self-hosted runner) on PRs touching `src-tauri/**`, `Cargo.toml`, `Cargo.lock`.

## Code Style

- **Rust:** `cargo fmt` (rustfmt, default config), `cargo clippy` clean. Edition 2024, serde `camelCase` renames on IPC structs, `tracing` for logging (never `println!`). Never log secrets — use `postail_security::ZeroizingBytes` for sensitive buffers.
- **Frontend:** Prettier with `useTabs: true`, printWidth 100, single quotes, no semicolons (`bun run format` writes the whole repo). TypeScript `strict` + `noUnusedLocals`/`noUnusedParameters` — `bun run build` fails on type errors. Import alias `@/*` → `src/*`. Tailwind 4 (CSS-first config in `src/App.css`); shadcn-style components in `src/components/ui/`.
- **New frontend code must pass `tsc`** — the build script is `tsc && vite build`.
- Commit style (conventional, from git log): `type(scope): summary` — e.g. `feat(nix):`, `fix(email_webview):`, `refactor(ui):`. English.
- **Prefer extending existing modules over introducing new abstractions** unless they provide clear value. If a new file or function is warranted, explain why in the PR description.
- **Do not:**
  - rewrite unrelated code
  - rename symbols without reason
  - change formatting outside touched code
  - introduce new dependencies unless requested
  - replace existing patterns with personal preference
- **Prefer minimal, localized changes.** Unless explicitly requested, avoid:
  - architecture rewrites
  - large refactors
  - moving files
  - introducing new abstractions

## Build and Deployment

```sh
bun run build            # tsc + vite build → dist/
bun run tauri build      # full app bundle (uses bun run build via beforeBuildCommand)
nix build                # Nix package (src-tauri only, tests skipped: no sandbox network)
```

- Outputs: `dist/` (frontend), `target/` (cargo).
- Windows cross-compile from Linux: `nix develop .#windows-cross` (sets MinGW CC/CXX/AR, OpenSSL static, fake `rustup` shim for tauri-cli). Known quirk: crate-type `["staticlib", "cdylib", "rlib"]` in `src-tauri/Cargo.toml` requires `-C link-arg=-Wl,--exclude-all-symbols` on the GNU target.
- CI: `.github/workflows/pr-check.yml` — powers on a self-hosted runner via API, runs `cargo test` in `src-tauri`, schedules shutdown after 5 min.

## Security Considerations

- All persisted data sits behind SQLCipher; the hex key comes from `SecurityManager` (master key derived per chosen tier: Argon2/TPM/keyring). Never bypass `SECURITY`/`CRYPTO_ACTOR` for encryption.
- Don't log passphrases, master keys, tokens (OAuth refresh tokens) or encrypted-buffer contents.
- Auto-lock: `src-tauri/src/security/lock_timer.rs` + frontend `useAutoLock` — keep in sync on UI changes.

## Troubleshooting / Gotchas

- **Graph staleness:** compare `git rev-parse HEAD` with graph build commit in `graphify-out/GRAPH_REPORT.md`; run `graphify update .` (AST-only, no API cost).
- **WebKit deps missing:** use `nix develop`; on distro systems install webkitgtk-4.1 + libsoup3 explicitly (Tauri 2 requires 4.1, not 4.0).
- **Vite port conflicts:** dev server is `strictPort: 1420`; HMR uses 1421.
- **`graphify-out/` is gitignored** (`graphify-out/` in .gitignore) but intentionally committed — use `git add -f graphify-out/`.
- `esbuild.drop: ['console', 'debugger']` removes console calls in production builds — don't rely on `console.log` for debugging prod bundles.
- The frontend has no lint/test script; type-checking via `bun run build` is the gate.
