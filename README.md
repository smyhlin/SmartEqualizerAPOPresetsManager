# SmartEqualizer APO Presets Manager

SmartEqualizer APO Presets Manager is a tray-first Windows 11 desktop app for organizing, editing, applying, importing, exporting, and backing up Equalizer APO presets. It is built with SvelteKit, TypeScript, Rust, and Tauri 2.

## What It Does

- Organizes presets into groups with drag-and-drop ordering.
- Applies presets from the main window or directly from the system tray.
- Edits preset `.txt` files in-app and exports them back to disk.
- Imports Equalizer APO preset files and convolution `.wav` files.
- Keeps convolution file references synced and can reveal linked files in Explorer.
- Imports and exports full app-data backups as JSON.
- Supports optional launch on Windows startup.

## Scope And Requirements

- Windows 11 only.
- Requires an existing Equalizer APO installation.
- Designed for local desktop use; this repository does not include release binaries.
- The tracked app icons in `src-tauri/icons/` are source assets and are intentionally committed.

## Runtime Layout

- App data folder: `%APPDATA%\\SmartEqualizerAPO`
- Managed preset library: `%APPDATA%\\SmartEqualizerAPO\\presets`
- Default writable Equalizer APO config target: `%APPDATA%\\SmartEqualizerAPO\\config`

If Equalizer APO is still pointing at a protected config directory, the app prompts to move its `ConfigPath` to the writable app-managed folder. Changing `ConfigPath` or updating protected Equalizer APO files can trigger a Windows UAC prompt.

## Development

### Prerequisites

- Node.js with npm
- Rust stable toolchain
- Windows 11

### Commands

```bash
npm install
npm run check
npm run tauri dev
npm run tauri build
```

- `npm run check` runs `svelte-check`.
- `npm run tauri dev` starts the frontend dev server and the Tauri desktop shell.
- `npm run tauri build` produces the Windows bundle.

## Repository Hygiene

Committed on purpose:

- Application source code
- `package-lock.json`
- `src-tauri/Cargo.lock`
- Tauri icon assets in `src-tauri/icons/`

Ignored on purpose:

- `node_modules/`
- `.svelte-kit/`
- `build/`
- `src-tauri/gen/`
- `src-tauri/target/`
- `.cargo-target/`

This repository is prepared for source upload, not binary distribution.

## License

MIT. See [LICENSE](LICENSE).
