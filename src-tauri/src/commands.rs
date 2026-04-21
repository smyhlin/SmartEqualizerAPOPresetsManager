use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use tauri::{AppHandle, State};

use crate::{
    logging::{append_log_line, log_folder_path, read_log_snapshot, LogSnapshot},
    current_runtime_settings, refresh_runtime, refresh_runtime_settings, set_autorun_enabled_state,
    state::{
        classify_elevation_failure, detect_installed_install_path, run_elevated_process, AppError,
        AppState, PresetLibrary,
    },
};

const INSTALL_APO_SCRIPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/scripts/install-apo.ps1"
));

fn escape_for_powershell(value: &str) -> String {
    value.replace('\'', "''")
}

fn write_install_script() -> Result<PathBuf, AppError> {
    let token = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let script_path = env::temp_dir().join(format!("install-apo-{token}.ps1"));
    fs::write(&script_path, INSTALL_APO_SCRIPT)?;
    Ok(script_path)
}

fn installer_log_path() -> PathBuf {
    let token = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    env::temp_dir().join(format!("install-apo-{token}.log"))
}

fn selector_path_from_install_path(install_path: &Path) -> PathBuf {
    install_path.join("DeviceSelector.exe")
}

fn should_retry_with_windows_powershell(error: &AppError) -> bool {
    let message = error.to_string().to_lowercase();
    message.contains("pwsh.exe")
        && (message.contains("cannot find")
            || message.contains("does not exist")
            || message.contains("not recognized"))
}

fn run_installer_script(script_path: &Path, log_path: &Path) -> Result<(), AppError> {
    let arguments = vec![
        "-NoProfile".to_string(),
        "-NonInteractive".to_string(),
        "-ExecutionPolicy".to_string(),
        "Bypass".to_string(),
        "-File".to_string(),
        script_path.to_string_lossy().into_owned(),
        "-LogPath".to_string(),
        log_path.to_string_lossy().into_owned(),
    ];

    match run_elevated_process(Path::new("pwsh.exe"), &arguments) {
        Ok(()) => Ok(()),
        Err(error) if should_retry_with_windows_powershell(&error) => {
            append_log_line(
                "WARN",
                "pwsh.exe was unavailable for the installer; retrying with powershell.exe.",
            );
            run_elevated_process(Path::new("powershell.exe"), &arguments)
        }
        Err(error) => Err(error),
    }
}

fn append_installer_log_to_app(log_path: &Path) {
    match fs::read_to_string(log_path) {
        Ok(content) => {
            for line in content.lines().map(str::trim).filter(|line| !line.is_empty()) {
                append_log_line("INFO", format!("[Installer] {line}"));
            }
        }
        Err(error) => append_log_line(
            "WARN",
            format!(
                "Unable to read the installer log at '{}': {error}",
                log_path.display()
            ),
        ),
    }
}

fn installer_log_summary(log_path: &Path) -> Option<String> {
    let content = fs::read_to_string(log_path).ok()?;
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .last()
        .map(|line| line.to_string())
}

fn launch_device_selector(selector_path: &Path, working_directory: &Path) -> Result<(), AppError> {
    let command = format!(
        "Start-Process -FilePath '{}' -WorkingDirectory '{}' -Verb RunAs -ErrorAction Stop",
        escape_for_powershell(selector_path.as_os_str().to_string_lossy().as_ref()),
        escape_for_powershell(working_directory.as_os_str().to_string_lossy().as_ref())
    );
    let shells = ["pwsh.exe", "powershell.exe"];
    let mut last_error: Option<std::io::Error> = None;

    for shell in shells {
        let result = Command::new(shell)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                command.as_str(),
            ])
            .output();

        match result {
            Ok(output) if output.status.success() => return Ok(()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let fallback_message = format!(
                    "Failed to launch Device Selector from '{}'.",
                    selector_path.display()
                );
                return Err(classify_elevation_failure(
                    output.status.code(),
                    stdout.as_str(),
                    stderr.as_str(),
                    fallback_message,
                ));
            }
            Err(error) => {
                last_error = Some(error);
            }
        }
    }

    Err(last_error.map_or_else(
        || AppError::Message("Unable to start an elevated PowerShell shell.".to_string()),
        |error| error.into(),
    ))
}

fn selector_path_error(selector_path: &Path) -> AppError {
    AppError::Message(format!(
        "Device Selector was not found at '{}'. Reinstall Equalizer APO to repair the install.",
        selector_path.display()
    ))
}

#[tauri::command]
pub fn get_config_path(state: State<'_, AppState>) -> Result<String, AppError> {
    let guard = state.lock()?;
    Ok(guard.get_config_path())
}

#[tauri::command]
pub fn load_logs() -> Result<LogSnapshot, AppError> {
    read_log_snapshot()
}

#[tauri::command]
pub fn install_or_reinstall_apo(app: AppHandle) -> Result<PresetLibrary, AppError> {
    append_log_line("INFO", "Starting Equalizer APO install or reinstall.");
    let script_path = write_install_script()?;
    let log_path = installer_log_path();
    let result = run_installer_script(script_path.as_path(), log_path.as_path());

    let cleanup_result = fs::remove_file(&script_path);
    if let Err(error) = cleanup_result {
        append_log_line("WARN", error.to_string());
    }

    append_installer_log_to_app(log_path.as_path());
    let installer_summary = installer_log_summary(log_path.as_path());

    let cleanup_log_result = fs::remove_file(&log_path);
    if let Err(error) = cleanup_log_result {
        append_log_line("WARN", error.to_string());
    }

    match &result {
        Ok(_) => append_log_line("INFO", "Equalizer APO install script completed."),
        Err(error) => {
            let summary = installer_summary.unwrap_or_else(|| error.to_string());
            let failure_message = format!("Equalizer APO install failed: {summary}");
            append_log_line("ERROR", failure_message.as_str());
            return Err(AppError::Message(format!(
                "{failure_message}. Open Logs for the full installer output."
            )));
        }
    }

    let snapshot = refresh_runtime(&app)?;
    append_log_line(
        "INFO",
        format!(
            "Equalizer APO snapshot refreshed after install. Detected install path: {}",
            snapshot
                .installed_config_path
                .as_deref()
                .unwrap_or("not detected")
        ),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn set_config_path(
    app: AppHandle,
    state: State<'_, AppState>,
    new_path: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.set_config_path(PathBuf::from(new_path))?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn load_presets(state: State<'_, AppState>) -> Result<PresetLibrary, AppError> {
    let mut guard = state.lock()?;
    guard.snapshot()
}

#[tauri::command]
pub fn apply_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    group: String,
    name: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.apply_preset(&group, &name)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn save_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    group: String,
    name: String,
    content: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.save_preset(&group, &name, &content)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn create_group(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.create_group(&name)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn set_group_emoji(
    app: AppHandle,
    state: State<'_, AppState>,
    group: String,
    emoji: Option<String>,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.set_group_emoji(&group, emoji)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn rename_group(
    app: AppHandle,
    state: State<'_, AppState>,
    old_name: String,
    new_name: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.rename_group(&old_name, &new_name)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn delete_group(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.delete_group(&name)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn reorder_groups(
    app: AppHandle,
    state: State<'_, AppState>,
    order: Vec<String>,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.reorder_groups(&order)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn create_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    group: String,
    name: String,
    content: Option<String>,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.create_preset(&group, &name, content)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn rename_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    group: String,
    old_name: String,
    new_name: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.rename_preset(&group, &old_name, &new_name)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn delete_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    group: String,
    name: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.delete_preset(&group, &name)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn move_preset(
    app: AppHandle,
    state: State<'_, AppState>,
    old_group: String,
    new_group: String,
    name: String,
    target_index: Option<usize>,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.move_preset(&old_group, &new_group, &name, target_index)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn import_presets(
    app: AppHandle,
    state: State<'_, AppState>,
    group: String,
    paths: Vec<String>,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.import_presets(&group, &paths)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn attach_convolution_wav(
    app: AppHandle,
    state: State<'_, AppState>,
    group: String,
    name: String,
    content: String,
    source_path: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.attach_convolution_wav(&group, &name, &content, &PathBuf::from(source_path))?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn remove_convolution_wav(
    app: AppHandle,
    state: State<'_, AppState>,
    group: String,
    name: String,
    content: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.remove_convolution_wav(&group, &name, &content)?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn export_preset(
    state: State<'_, AppState>,
    group: String,
    name: String,
    destination: String,
) -> Result<String, AppError> {
    let guard = state.lock()?;
    guard.export_preset(&group, &name, &PathBuf::from(destination))?;
    Ok(name)
}

#[tauri::command]
pub fn export_app_settings(
    state: State<'_, AppState>,
    destination: String,
) -> Result<(), AppError> {
    let mut guard = state.lock()?;
    guard.export_app_settings(&PathBuf::from(destination))
}

#[tauri::command]
pub fn import_app_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    source: String,
) -> Result<PresetLibrary, AppError> {
    {
        let mut guard = state.lock()?;
        guard.import_app_settings(&PathBuf::from(source))?;
    }
    refresh_runtime(&app)
}

#[tauri::command]
pub fn rebuild_tray_menu(app: AppHandle) -> Result<PresetLibrary, AppError> {
    refresh_runtime(&app)
}

#[tauri::command]
pub fn get_autorun_enabled(app: AppHandle) -> Result<bool, AppError> {
    Ok(current_runtime_settings(&app)?.autorun_enabled)
}

#[tauri::command]
pub fn set_autorun_enabled(app: AppHandle, enabled: bool) -> Result<bool, AppError> {
    set_autorun_enabled_state(&app, enabled)?;
    Ok(refresh_runtime_settings(&app)?.autorun_enabled)
}

#[tauri::command]
pub fn reveal_path_in_explorer(path: String) -> Result<(), AppError> {
    let target = PathBuf::from(path);
    if !target.exists() {
        return Err(AppError::Message(format!(
            "The file or folder does not exist: {}",
            target.display()
        )));
    }

    if target.is_dir() {
        Command::new("explorer.exe").arg(target).spawn()?;
    } else {
        Command::new("explorer.exe")
            .arg(format!("/select,{}", target.display()))
            .spawn()?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_apo_device_selector() -> Result<(), AppError> {
    append_log_line("INFO", "Opening Equalizer APO Device Selector.");
    let install_path = detect_installed_install_path().ok_or_else(|| {
        AppError::Message(
            "Equalizer APO is not installed yet. Install or reinstall it first.".to_string(),
        )
    })?;
    let selector_path = selector_path_from_install_path(&install_path);

    if !selector_path.exists() {
        append_log_line(
            "ERROR",
            format!(
                "Device Selector was not found at '{}'.",
                selector_path.display()
            ),
        );
        return Err(selector_path_error(&selector_path));
    }

    let result = launch_device_selector(&selector_path, &install_path);
    match &result {
        Ok(_) => append_log_line(
            "INFO",
            format!("Device Selector launched from '{}'.", selector_path.display()),
        ),
        Err(error) => append_log_line("ERROR", format!("Device Selector launch failed: {error}")),
    }
    result
}

#[tauri::command]
pub fn open_repository_url() -> Result<(), AppError> {
    append_log_line("INFO", "Opening the project repository in the default browser.");
    webbrowser::open("https://github.com/smyhlin/SmartEqualizerAPOPresetsManager")
        .map_err(|error| AppError::Message(format!("Failed to open the repository URL: {error}")))
}

#[tauri::command]
pub fn open_logs_location() -> Result<(), AppError> {
    let folder_path = log_folder_path()?;
    append_log_line(
        "INFO",
        format!("Opening the logs folder at '{}'.", folder_path.display()),
    );
    reveal_path_in_explorer(folder_path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_path_from_install_path_should_point_to_device_selector_exe() {
        let install_path = Path::new(r"C:\Program Files\EqualizerAPO");

        let selector_path = selector_path_from_install_path(install_path);

        assert_eq!(
            selector_path,
            PathBuf::from(r"C:\Program Files\EqualizerAPO\DeviceSelector.exe")
        );
    }

    #[test]
    fn should_retry_with_windows_powershell_only_when_pwsh_is_missing() {
        let error = AppError::Message(
            "The elevated PowerShell command 'pwsh.exe' exited with code 1. The system cannot find the file specified.".to_string(),
        );

        assert!(should_retry_with_windows_powershell(&error));
    }

    #[test]
    fn should_not_retry_windows_powershell_for_installer_failures() {
        let error = AppError::Message(
            "Equalizer APO installer exited with code 1603.".to_string(),
        );

        assert!(!should_retry_with_windows_powershell(&error));
    }
}
