use std::{path::PathBuf, process::Command};

use tauri::{AppHandle, State};

use crate::{
    current_runtime_settings, refresh_runtime, refresh_runtime_settings, set_autorun_enabled_state,
    state::{AppError, AppState, PresetLibrary},
};

#[tauri::command]
pub fn get_config_path(state: State<'_, AppState>) -> Result<String, AppError> {
    let guard = state.lock()?;
    Ok(guard.get_config_path())
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
