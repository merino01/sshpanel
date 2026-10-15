## Agent skills

### Issue tracker

Issues live as GitHub Issues in `merino01/sshpanel`, worked with the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Code layout

- **Core (`crates/core`)**: the only place with SSH/SFTP logic, tested through its public API (`Core`) against the OpenSSH test bench in `tests/`. `src/connection/` is split by area: `explorer`, `terminal`, `transfer/`, `trash/`, plus shared `naming` (free names, never overwrite) and `paths`.
- **Tauri (`src-tauri`)**: thin commands in `src/commands/`, one file per area; errors cross as `Error::code()` strings.
- **UI (`ui/src`, Svelte 5)**: `lib/api.ts` is the only caller of `invoke` (one typed function per command); `lib/` holds logic without DOM (tested with Vitest, `npm test`); `state/` holds app state as runes modules (`*.svelte.ts`); `components/` are generic pieces (`ContextMenu`, `ConfirmDialog`…) driven by `state/menu` and `state/confirm`; `screens/` are the screens and their parts. User-facing text lives in `i18n/es.json`.
