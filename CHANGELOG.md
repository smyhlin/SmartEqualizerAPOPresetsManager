# Changelog

## 0.2.0 - 2026-04-21

### Added

- New footer utility controls: `Logs`, `Troubleshoot`, `About`, and the `Launch on Windows startup` toggle.
- In-app logs viewer with readable local timestamps and direct opening of the logs folder.
- About modal with the project description and a repository link that opens in the default browser.
- Troubleshoot modal for detecting Equalizer APO state, reinstalling it, and reopening the official Device Selector.

### Changed

- Equalizer APO install and reinstall now resolve the real SourceForge mirror URL before downloading the installer.
- Installer and launcher diagnostics now append detailed step-by-step output into the app log for easier support.
- Device Selector launching now uses the resolved Equalizer APO install path and working directory.

### Backend Functions

- Added `load_logs` and `open_logs_location` for log viewing and log-folder access.
- Added `open_repository_url` for browser launching from the About panel.
- Added `install_or_reinstall_apo` and `open_apo_device_selector` for the troubleshooting workflow.
