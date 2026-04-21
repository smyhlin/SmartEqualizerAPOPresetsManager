# SmartEqualizer APO Presets Manager

SmartEqualizer APO Presets Manager is a tray-first Windows 11 desktop app for organizing, editing, applying, importing, exporting, and backing up Equalizer APO presets. It is built with SvelteKit, TypeScript, Rust, and Tauri 2.

---
<img width="1641" height="1128" alt="image" src="https://github.com/user-attachments/assets/bb28b951-ea68-4f92-b30e-f389ffd3f726" />


---
<img width="1038" height="187" alt="{822CCB8E-E623-43B4-8D31-B20FFC6518A6}" src="https://github.com/user-attachments/assets/8b0a058e-7568-4b07-83f8-e07f96651217" />


---

## What It Does

- Organizes presets into groups with drag-and-drop ordering.
- Applies presets from the main window or directly from the system tray.
- Edits preset `.txt` files in-app and exports them back to disk.
- Imports Equalizer APO preset files and convolution `.wav` files.
- Keeps convolution file references synced and can reveal linked files in Explorer.
- Imports and exports full app-data backups as JSON.
- Can install or reinstall Equalizer APO from the troubleshooting panel and then open the official Device Selector for playback and capture device selection.
- Includes footer utility actions for `Logs`, `Troubleshoot`, and `About`, plus a launch-on-startup toggle.
- Stores readable local-timestamped logs and can open the logs folder directly from the app.
- Supports optional launch on Windows startup.

## Scope And Requirements

- Windows 11 only.
- Can set up Equalizer APO on demand from the app, but the installer still requires Windows admin approval.
- Designed for local desktop use; this repository does not include release binaries.
- The tracked app icons in `src-tauri/icons/` are source assets and are intentionally committed.

## Runtime Layout

- App data folder: `%APPDATA%\\SmartEqualizerAPO`
- Managed preset library: `%APPDATA%\\SmartEqualizerAPO\\presets`
- Default writable Equalizer APO config target: `%APPDATA%\\SmartEqualizerAPO\\config`

If Equalizer APO is still pointing at a protected config directory, the app prompts to move its `ConfigPath` to the writable app-managed folder. Changing `ConfigPath` or updating protected Equalizer APO files can trigger a Windows UAC prompt.

## Equalizer APO Setup

Use the `Troubleshoot` button in the main window to:

- Download and silently install Equalizer APO with the official `/S` installer.
- Re-run the same install chain if the install needs to be repaired.
- Open the official Device Selector so you can choose the playback and capture devices that should receive APO processing.

The troubleshooting panel also shows whether Equalizer APO is detected, the active config path, and whether the current config path looks writable.

## Release Notes

Version `0.2.0` adds the new footer utility buttons (`Logs`, `Troubleshoot`, and `About`), a log viewer with readable timestamps and direct log-folder access, browser launching for the repository link, and backend install/reinstall functions for Equalizer APO repair and Device Selector reopening.

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
