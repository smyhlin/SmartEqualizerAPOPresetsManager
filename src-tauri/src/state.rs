use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, MutexGuard},
};

use serde::{Deserialize, Serialize};
use tauri::Error as TauriError;
use thiserror::Error;
use winreg::{
    enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, KEY_WOW64_64KEY},
    RegKey,
};

pub const APP_FOLDER_NAME: &str = "SmartEqualizerAPO";
pub const EVENT_PRESETS_UPDATED: &str = "smart-equalizer://presets-updated";
pub const REGISTRY_KEY_PATH: &str = r"SOFTWARE\EqualizerAPO";
pub const REGISTRY_VALUE_NAME: &str = "ConfigPath";

#[derive(Debug, Error)]
pub enum AppError {
    #[error("AppData is unavailable on this system.")]
    AppDataUnavailable,
    #[error("The tray icon was not initialized.")]
    MissingTray,
    #[error("The bundled application icon is missing.")]
    MissingIcon,
    #[error("A concurrent operation failed because the app state lock was poisoned.")]
    StatePoisoned,
    #[error("Group '{0}' was not found.")]
    GroupNotFound(String),
    #[error("Preset '{name}' was not found in group '{group}'.")]
    PresetNotFound { group: String, name: String },
    #[error("'{0}' already exists.")]
    AlreadyExists(String),
    #[error("'{0}' is not a valid Windows file name.")]
    InvalidName(String),
    #[error("The tray menu item '{0}' is no longer valid.")]
    UnknownMenuItem(String),
    #[error("The Equalizer APO config path registry entry is missing.")]
    RegistryValueMissing,
    #[error("Failed to start the elevated helper to update the Equalizer APO config path.")]
    ElevationDeclined,
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Tauri(#[from] TauriError),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PresetLibrary {
    pub app_data_dir: String,
    pub config_path: String,
    pub default_config_path: String,
    pub groups: Vec<PresetGroup>,
    pub needs_config_migration: bool,
    pub config_path_prompted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PresetGroup {
    pub name: String,
    pub order: usize,
    pub emoji: Option<String>,
    pub active_preset: Option<String>,
    pub presets: Vec<PresetItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PresetItem {
    pub name: String,
    pub order: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetsMetadata {
    #[serde(default)]
    pub config_path_prompted: bool,
    #[serde(default)]
    pub groups: Vec<GroupMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GroupMetadata {
    pub name: String,
    pub order: usize,
    #[serde(default)]
    pub emoji: Option<String>,
    pub active_preset: Option<String>,
    #[serde(default)]
    pub presets: Vec<PresetMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetMetadata {
    pub name: String,
    pub order: usize,
}

#[derive(Debug, Clone)]
pub struct TraySelection {
    pub group: String,
    pub preset: String,
}

#[derive(Debug)]
pub struct AppState {
    inner: Mutex<AppStateInner>,
}

#[derive(Debug)]
pub struct AppStateInner {
    app_data_dir: PathBuf,
    presets_dir: PathBuf,
    metadata_path: PathBuf,
    default_config_path: PathBuf,
    current_config_path: PathBuf,
    metadata: PresetsMetadata,
    tray_menu_targets: Vec<(String, TraySelection)>,
}

impl AppState {
    pub fn initialize() -> Result<Self, AppError> {
        let app_data_dir = dirs::config_dir()
            .ok_or(AppError::AppDataUnavailable)?
            .join(APP_FOLDER_NAME);
        let presets_dir = app_data_dir.join("presets");
        let metadata_path = app_data_dir.join("presets.json");
        let default_config_path = app_data_dir.join("config");

        fs::create_dir_all(&presets_dir)?;
        fs::create_dir_all(&default_config_path)?;

        let metadata = load_metadata(&metadata_path)?;
        let current_config_path = read_registry_config_path().unwrap_or_else(|_| default_config_path.clone());

        let mut inner = AppStateInner {
            app_data_dir,
            presets_dir,
            metadata_path,
            default_config_path,
            current_config_path,
            metadata,
            tray_menu_targets: Vec::new(),
        };

        inner.sync_metadata_with_disk()?;
        inner.persist_metadata()?;

        if inner.is_config_path_writable()? {
            inner.write_active_config()?;
        }

        Ok(Self {
            inner: Mutex::new(inner),
        })
    }

    pub fn lock(&self) -> Result<MutexGuard<'_, AppStateInner>, AppError> {
        self.inner.lock().map_err(|_| AppError::StatePoisoned)
    }
}

impl AppStateInner {
    pub fn snapshot(&mut self) -> Result<PresetLibrary, AppError> {
        self.sync_metadata_with_disk()?;
        self.normalize_single_active_selection();

        let groups = self
            .metadata
            .groups
            .iter()
            .enumerate()
            .map(|(group_index, group)| {
                let presets = group
                    .presets
                    .iter()
                    .enumerate()
                    .filter_map(|(preset_index, preset)| {
                        let preset_path = self.preset_path(&group.name, &preset.name);
                        if !preset_path.exists() {
                            return None;
                        }

                        let content = fs::read_to_string(preset_path).unwrap_or_default();
                        Some(PresetItem {
                            name: preset.name.clone(),
                            order: preset_index,
                            content,
                        })
                    })
                    .collect::<Vec<_>>();

                PresetGroup {
                    name: group.name.clone(),
                    order: group_index,
                    emoji: group.emoji.clone(),
                    active_preset: group.active_preset.clone(),
                    presets,
                }
            })
            .collect::<Vec<_>>();

        Ok(PresetLibrary {
            app_data_dir: path_to_string(&self.app_data_dir),
            config_path: path_to_string(&self.current_config_path),
            default_config_path: path_to_string(&self.default_config_path),
            groups,
            needs_config_migration: !self.is_config_path_writable()?,
            config_path_prompted: self.metadata.config_path_prompted,
        })
    }

    pub fn get_config_path(&self) -> String {
        path_to_string(&self.current_config_path)
    }

    pub fn default_config_path_string(&self) -> String {
        path_to_string(&self.default_config_path)
    }

    pub fn mark_config_path_prompted(&mut self, prompted: bool) -> Result<(), AppError> {
        self.metadata.config_path_prompted = prompted;
        self.persist_metadata()
    }

    pub fn should_prompt_for_config_migration(&self) -> Result<bool, AppError> {
        Ok(!self.metadata.config_path_prompted
            && self.current_config_path != self.default_config_path
            && !self.is_config_path_writable()?)
    }

    pub fn resolve_tray_selection(&self, menu_id: &str) -> Result<TraySelection, AppError> {
        self.tray_menu_targets
            .iter()
            .find(|(id, _)| id == menu_id)
            .map(|(_, selection)| selection.clone())
            .ok_or_else(|| AppError::UnknownMenuItem(menu_id.to_string()))
    }

    pub fn replace_tray_targets(&mut self, targets: Vec<(String, TraySelection)>) {
        self.tray_menu_targets = targets;
    }

    pub fn set_config_path(&mut self, new_path: PathBuf) -> Result<(), AppError> {
        ensure_directory(&new_path)?;

        match write_registry_config_path(&new_path) {
            Ok(()) => {}
            Err(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied
                    || error.kind() == std::io::ErrorKind::Other =>
            {
                run_elevated_registry_helper(&new_path)?;
            }
            Err(error) => return Err(error.into()),
        }

        self.current_config_path = new_path;
        self.write_active_config()
    }

    pub fn create_group(&mut self, name: &str) -> Result<(), AppError> {
        let valid_name = validate_name(name)?;
        if self.group_index(&valid_name).is_some() {
            return Err(AppError::AlreadyExists(valid_name));
        }

        ensure_directory(&self.group_path(&valid_name))?;
        self.metadata.groups.push(GroupMetadata {
            name: valid_name,
            order: self.metadata.groups.len(),
            emoji: None,
            active_preset: None,
            presets: Vec::new(),
        });
        self.reindex_orders();
        self.persist_metadata()
    }

    pub fn rename_group(&mut self, old_name: &str, new_name: &str) -> Result<(), AppError> {
        let new_name = validate_name(new_name)?;
        let group_index = self
            .group_index(old_name)
            .ok_or_else(|| AppError::GroupNotFound(old_name.to_string()))?;
        if self.group_index(&new_name).is_some() {
            return Err(AppError::AlreadyExists(new_name));
        }

        fs::rename(self.group_path(old_name), self.group_path(&new_name))?;
        self.metadata.groups[group_index].name = new_name;
        self.persist_metadata()?;
        self.write_active_config()
    }

    pub fn set_group_emoji(
        &mut self,
        group_name: &str,
        emoji: Option<String>,
    ) -> Result<(), AppError> {
        let group_index = self
            .group_index(group_name)
            .ok_or_else(|| AppError::GroupNotFound(group_name.to_string()))?;

        let normalized_emoji = emoji.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });

        self.metadata.groups[group_index].emoji = normalized_emoji;
        self.persist_metadata()
    }

    pub fn delete_group(&mut self, name: &str) -> Result<(), AppError> {
        let group_index = self
            .group_index(name)
            .ok_or_else(|| AppError::GroupNotFound(name.to_string()))?;
        let group_path = self.group_path(name);
        if group_path.exists() {
            fs::remove_dir_all(group_path)?;
        }

        self.metadata.groups.remove(group_index);
        self.reindex_orders();
        self.persist_metadata()?;
        self.write_active_config()
    }

    pub fn reorder_groups(&mut self, ordered_names: &[String]) -> Result<(), AppError> {
        let existing_names = self
            .metadata
            .groups
            .iter()
            .map(|group| group.name.clone())
            .collect::<Vec<_>>();
        if ordered_names.len() != existing_names.len()
            || ordered_names
                .iter()
                .any(|name| !existing_names.iter().any(|existing| existing == name))
        {
            return Err(AppError::Message(
                "The supplied group order does not match the existing groups.".to_string(),
            ));
        }

        self.metadata.groups.sort_by_key(|group| {
            ordered_names
                .iter()
                .position(|name| name == &group.name)
                .unwrap_or(usize::MAX)
        });
        self.reindex_orders();
        self.persist_metadata()?;
        self.write_active_config()
    }

    pub fn create_preset(
        &mut self,
        group_name: &str,
        preset_name: &str,
        content: Option<String>,
    ) -> Result<(), AppError> {
        let preset_name = validate_name(preset_name)?;
        let group_index = self
            .group_index(group_name)
            .ok_or_else(|| AppError::GroupNotFound(group_name.to_string()))?;

        if self.preset_index(group_index, &preset_name).is_some() {
            return Err(AppError::AlreadyExists(format!("{group_name}/{preset_name}")));
        }

        let order = self.metadata.groups[group_index].presets.len();
        write_text_file_atomically(
            &self.preset_path(group_name, &preset_name),
            content.unwrap_or_default().as_str(),
        )?;

        self.metadata.groups[group_index].presets.push(PresetMetadata {
            name: preset_name,
            order,
        });
        self.reindex_orders();
        self.persist_metadata()
    }

    pub fn save_preset(
        &mut self,
        group_name: &str,
        preset_name: &str,
        content: &str,
    ) -> Result<(), AppError> {
        let preset_name = validate_name(preset_name)?;
        let group_index = self
            .group_index(group_name)
            .ok_or_else(|| AppError::GroupNotFound(group_name.to_string()))?;

        let active_before_change = self.metadata.groups[group_index]
            .active_preset
            .as_deref()
            == Some(preset_name.as_str());
        let order = self.metadata.groups[group_index].presets.len();
        write_text_file_atomically(&self.preset_path(group_name, &preset_name), content)?;
        if self.preset_index(group_index, &preset_name).is_none() {
            self.metadata.groups[group_index].presets.push(PresetMetadata {
                name: preset_name,
                order,
            });
        }

        self.reindex_orders();
        self.persist_metadata()?;
        if active_before_change {
            self.write_active_config()?;
        }
        Ok(())
    }

    pub fn rename_preset(
        &mut self,
        group_name: &str,
        old_name: &str,
        new_name: &str,
    ) -> Result<(), AppError> {
        let new_name = validate_name(new_name)?;
        let group_index = self
            .group_index(group_name)
            .ok_or_else(|| AppError::GroupNotFound(group_name.to_string()))?;
        let preset_index = self
            .preset_index(group_index, old_name)
            .ok_or_else(|| AppError::PresetNotFound {
                group: group_name.to_string(),
                name: old_name.to_string(),
            })?;

        if self.preset_index(group_index, &new_name).is_some() {
            return Err(AppError::AlreadyExists(format!("{group_name}/{new_name}")));
        }

        fs::rename(
            self.preset_path(group_name, old_name),
            self.preset_path(group_name, &new_name),
        )?;

        self.metadata.groups[group_index].presets[preset_index].name = new_name.clone();
        if self.metadata.groups[group_index].active_preset.as_deref() == Some(old_name) {
            self.metadata.groups[group_index].active_preset = Some(new_name);
        }

        self.persist_metadata()?;
        self.write_active_config()
    }

    pub fn delete_preset(&mut self, group_name: &str, preset_name: &str) -> Result<(), AppError> {
        let group_index = self
            .group_index(group_name)
            .ok_or_else(|| AppError::GroupNotFound(group_name.to_string()))?;
        let preset_index = self
            .preset_index(group_index, preset_name)
            .ok_or_else(|| AppError::PresetNotFound {
                group: group_name.to_string(),
                name: preset_name.to_string(),
            })?;

        let preset_path = self.preset_path(group_name, preset_name);
        if preset_path.exists() {
            fs::remove_file(preset_path)?;
        }

        self.metadata.groups[group_index].presets.remove(preset_index);
        if self.metadata.groups[group_index].active_preset.as_deref() == Some(preset_name) {
            self.metadata.groups[group_index].active_preset = None;
        }

        self.reindex_orders();
        self.persist_metadata()?;
        self.write_active_config()
    }

    pub fn move_preset(
        &mut self,
        old_group_name: &str,
        new_group_name: &str,
        preset_name: &str,
        target_index: Option<usize>,
    ) -> Result<(), AppError> {
        let old_group_index = self
            .group_index(old_group_name)
            .ok_or_else(|| AppError::GroupNotFound(old_group_name.to_string()))?;
        let new_group_index = self
            .group_index(new_group_name)
            .ok_or_else(|| AppError::GroupNotFound(new_group_name.to_string()))?;
        let preset_index = self
            .preset_index(old_group_index, preset_name)
            .ok_or_else(|| AppError::PresetNotFound {
                group: old_group_name.to_string(),
                name: preset_name.to_string(),
            })?;

        let preset_metadata = self.metadata.groups[old_group_index].presets.remove(preset_index);
        let was_active = self.metadata.groups[old_group_index]
            .active_preset
            .as_deref()
            == Some(preset_name);

        if old_group_name != new_group_name {
            let destination = self.preset_path(new_group_name, preset_name);
            if destination.exists() {
                return Err(AppError::AlreadyExists(format!("{new_group_name}/{preset_name}")));
            }

            fs::rename(self.preset_path(old_group_name, preset_name), destination)?;
            if was_active {
                self.metadata.groups[old_group_index].active_preset = None;
                self.metadata.groups[new_group_index].active_preset = Some(preset_name.to_string());
            }
        }

        let mut target_slot = target_index.unwrap_or(self.metadata.groups[new_group_index].presets.len());
        if old_group_name == new_group_name && preset_index < target_slot {
            target_slot = target_slot.saturating_sub(1);
        }
        target_slot = target_slot.min(self.metadata.groups[new_group_index].presets.len());
        self.metadata.groups[new_group_index]
            .presets
            .insert(target_slot, preset_metadata);

        self.reindex_orders();
        self.persist_metadata()?;
        self.write_active_config()
    }

    pub fn export_app_settings(&mut self, destination: &Path) -> Result<(), AppError> {
        let snapshot = self.snapshot()?;
        let payload = serde_json::to_string_pretty(&snapshot)?;
        write_text_file_atomically(destination, payload.as_str())
    }

    pub fn import_app_settings(&mut self, source: &Path) -> Result<(), AppError> {
        let payload = fs::read_to_string(source)?;
        let imported: PresetLibrary = serde_json::from_str(&payload)?;
        let imported_config_path = imported.config_path.trim().to_string();
        if imported_config_path.is_empty() {
            return Err(AppError::Message(
                "The imported backup is missing a config path.".to_string(),
            ));
        }

        let mut rebuilt_groups = Vec::with_capacity(imported.groups.len());
        for (group_order, group) in imported.groups.iter().enumerate() {
            let group_name = validate_name(&group.name)?;
            let normalized_emoji = group.emoji.as_ref().and_then(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            });

            let mut rebuilt_presets = Vec::with_capacity(group.presets.len());
            for (preset_order, preset) in group.presets.iter().enumerate() {
                let preset_name = validate_name(&preset.name)?;
                rebuilt_presets.push(PresetMetadata {
                    name: preset_name,
                    order: preset_order,
                });
            }

            let active_preset = group.active_preset.as_ref().and_then(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return None;
                }

                if rebuilt_presets.iter().any(|preset| preset.name == trimmed) {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            });

            rebuilt_groups.push(GroupMetadata {
                name: group_name,
                order: group_order,
                emoji: normalized_emoji,
                active_preset,
                presets: rebuilt_presets,
            });
        }

        let staging_dir = self.app_data_dir.join("presets.importing");
        let backup_dir = self.app_data_dir.join("presets.backup");
        if staging_dir.exists() {
            fs::remove_dir_all(&staging_dir)?;
        }
        if backup_dir.exists() {
            fs::remove_dir_all(&backup_dir)?;
        }
        fs::create_dir_all(&staging_dir)?;

        for group in &imported.groups {
            let group_name = validate_name(&group.name)?;
            let group_dir = staging_dir.join(&group_name);
            fs::create_dir_all(&group_dir)?;

            for preset in &group.presets {
                let preset_name = validate_name(&preset.name)?;
                write_text_file_atomically(
                    &group_dir.join(format!("{preset_name}.txt")),
                    preset.content.as_str(),
                )?;
            }
        }

        if self.presets_dir.exists() {
            if let Err(error) = fs::rename(&self.presets_dir, &backup_dir) {
                let _ = fs::remove_dir_all(&staging_dir);
                return Err(error.into());
            }
        }

        if let Err(error) = fs::rename(&staging_dir, &self.presets_dir) {
            if backup_dir.exists() {
                let _ = fs::rename(&backup_dir, &self.presets_dir);
            }
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(error.into());
        }

        if backup_dir.exists() {
            let _ = fs::remove_dir_all(&backup_dir);
        }

        self.metadata.groups = rebuilt_groups;
        self.metadata.config_path_prompted = imported.config_path_prompted;
        self.reindex_orders();
        self.persist_metadata()?;
        self.set_config_path(PathBuf::from(imported_config_path))
    }

    pub fn apply_preset(&mut self, group_name: &str, preset_name: &str) -> Result<(), AppError> {
        let group_index = self
            .group_index(group_name)
            .ok_or_else(|| AppError::GroupNotFound(group_name.to_string()))?;
        if self.preset_index(group_index, preset_name).is_none() {
            return Err(AppError::PresetNotFound {
                group: group_name.to_string(),
                name: preset_name.to_string(),
            });
        }

        self.clear_active_selection();
        self.metadata.groups[group_index].active_preset = Some(preset_name.to_string());
        self.persist_metadata()?;
        self.write_active_config()
    }

    pub fn import_presets(&mut self, group_name: &str, paths: &[String]) -> Result<(), AppError> {
        let group_index = self
            .group_index(group_name)
            .ok_or_else(|| AppError::GroupNotFound(group_name.to_string()))?;

        for raw_path in paths {
            let file_path = PathBuf::from(raw_path);
            let content = fs::read_to_string(&file_path)?;
            let base_name = file_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(sanitize_import_name)
                .unwrap_or_else(|| "Imported Preset".to_string());
            let unique_name = self.unique_preset_name(group_index, &base_name);
            write_text_file_atomically(
                &self.preset_path(group_name, &unique_name),
                content.as_str(),
            )?;

            let order = self.metadata.groups[group_index].presets.len();
            self.metadata.groups[group_index].presets.push(PresetMetadata {
                name: unique_name,
                order,
            });
        }

        self.reindex_orders();
        self.persist_metadata()
    }

    pub fn export_preset(
        &self,
        group_name: &str,
        preset_name: &str,
        destination: &Path,
    ) -> Result<(), AppError> {
        let group_index = self
            .group_index(group_name)
            .ok_or_else(|| AppError::GroupNotFound(group_name.to_string()))?;
        if self.preset_index(group_index, preset_name).is_none() {
            return Err(AppError::PresetNotFound {
                group: group_name.to_string(),
                name: preset_name.to_string(),
            });
        }

        let content = fs::read_to_string(self.preset_path(group_name, preset_name))?;
        write_text_file_atomically(destination, content.as_str())
    }

    fn sync_metadata_with_disk(&mut self) -> Result<(), AppError> {
        let mut ordered_groups = Vec::new();
        let disk_group_names = list_group_names(&self.presets_dir)?;

        for group in &self.metadata.groups {
            if disk_group_names.contains(&group.name) {
                ordered_groups.push(group.name.clone());
            }
        }
        for group_name in disk_group_names {
            if !ordered_groups.contains(&group_name) {
                ordered_groups.push(group_name);
            }
        }

        let mut rebuilt_groups = Vec::new();
        for (group_order, group_name) in ordered_groups.iter().enumerate() {
            let old_group = self.metadata.groups.iter().find(|group| group.name == *group_name);
            let disk_presets = list_preset_names(&self.group_path(group_name))?;
            let mut ordered_presets = Vec::new();

            if let Some(old_group) = old_group {
                for preset in &old_group.presets {
                    if disk_presets.contains(&preset.name) {
                        ordered_presets.push(preset.name.clone());
                    }
                }
            }
            for preset_name in disk_presets {
                if !ordered_presets.contains(&preset_name) {
                    ordered_presets.push(preset_name);
                }
            }

            let active_preset = old_group
                .and_then(|group| group.active_preset.clone())
                .filter(|active| ordered_presets.contains(active));
            let emoji = old_group.and_then(|group| group.emoji.clone());

            rebuilt_groups.push(GroupMetadata {
                name: group_name.clone(),
                order: group_order,
                emoji,
                active_preset,
                presets: ordered_presets
                    .into_iter()
                    .enumerate()
                    .map(|(preset_order, preset_name)| PresetMetadata {
                        name: preset_name,
                        order: preset_order,
                    })
                    .collect(),
            });
        }

        self.metadata.groups = rebuilt_groups;
        self.normalize_single_active_selection();
        Ok(())
    }

    fn persist_metadata(&mut self) -> Result<(), AppError> {
        self.normalize_single_active_selection();
        self.reindex_orders();
        let payload = serde_json::to_string_pretty(&self.metadata)?;
        write_text_file_atomically(&self.metadata_path, payload.as_str())
    }

    fn reindex_orders(&mut self) {
        for (group_order, group) in self.metadata.groups.iter_mut().enumerate() {
            group.order = group_order;
            for (preset_order, preset) in group.presets.iter_mut().enumerate() {
                preset.order = preset_order;
            }
        }
    }

    fn write_active_config(&mut self) -> Result<(), AppError> {
        ensure_directory(&self.current_config_path)?;
        let payload = if let Some((group_name, preset_name)) = self.active_selection() {
            let active_path = self.preset_path(group_name.as_str(), preset_name.as_str());
            if active_path.exists() {
                let include_path = self.include_path_for_preset(&active_path);
                format!(
                    "# SmartEqualizerAPOPresetsManager\r\n# Active preset: {} / {}\r\nInclude: {}\r\n",
                    group_name,
                    preset_name,
                    include_path
                )
            } else {
                "# SmartEqualizerAPOPresetsManager\r\n# No active preset selected.\r\n".to_string()
            }
        } else {
            "# SmartEqualizerAPOPresetsManager\r\n# No active preset selected.\r\n".to_string()
        };

        write_text_file_atomically(&self.current_config_path.join("config.txt"), payload.as_str())
    }

    fn is_config_path_writable(&self) -> Result<bool, AppError> {
        is_directory_writable(&self.current_config_path)
    }

    fn group_index(&self, name: &str) -> Option<usize> {
        self.metadata.groups.iter().position(|group| group.name == name)
    }

    fn preset_index(&self, group_index: usize, name: &str) -> Option<usize> {
        self.metadata.groups[group_index]
            .presets
            .iter()
            .position(|preset| preset.name == name)
    }

    fn group_path(&self, group_name: &str) -> PathBuf {
        self.presets_dir.join(group_name)
    }

    fn preset_path(&self, group_name: &str, preset_name: &str) -> PathBuf {
        self.group_path(group_name).join(format!("{preset_name}.txt"))
    }

    fn unique_preset_name(&self, group_index: usize, base_name: &str) -> String {
        if self.preset_index(group_index, base_name).is_none() {
            return base_name.to_string();
        }

        let mut suffix = 2usize;
        loop {
            let candidate = format!("{base_name} {suffix}");
            if self.preset_index(group_index, &candidate).is_none() {
                return candidate;
            }
            suffix += 1;
        }
    }

    fn clear_active_selection(&mut self) {
        for group in &mut self.metadata.groups {
            group.active_preset = None;
        }
    }

    fn normalize_single_active_selection(&mut self) {
        let mut active_seen = false;
        for group in &mut self.metadata.groups {
            if group.active_preset.is_some() {
                if active_seen {
                    group.active_preset = None;
                } else {
                    active_seen = true;
                }
            }
        }
    }

    fn active_selection(&self) -> Option<(String, String)> {
        self.metadata.groups.iter().find_map(|group| {
            group
                .active_preset
                .as_ref()
                .map(|preset| (group.name.clone(), preset.clone()))
        })
    }

    fn include_path_for_preset(&self, preset_path: &Path) -> String {
        if let Some(relative) = relative_path(&self.current_config_path, preset_path) {
            return path_to_string(&relative);
        }

        path_to_string(preset_path)
    }
}

pub fn try_handle_cli_mode() -> Option<i32> {
    let mut args = env::args().skip(1);
    let command = args.next()?;
    if command != "--elevated-set-config-path" {
        return None;
    }

    let Some(path) = args.next() else {
        return Some(1);
    };

    let exit_code = match write_registry_config_path(Path::new(&path)) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            1
        }
    };
    Some(exit_code)
}

fn load_metadata(metadata_path: &Path) -> Result<PresetsMetadata, AppError> {
    if !metadata_path.exists() {
        return Ok(PresetsMetadata::default());
    }

    let payload = fs::read_to_string(metadata_path)?;
    Ok(serde_json::from_str(&payload)?)
}

fn read_registry_config_path() -> Result<PathBuf, AppError> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey_with_flags(REGISTRY_KEY_PATH, KEY_READ | KEY_WOW64_64KEY)?;
    let value: String = key
        .get_value(REGISTRY_VALUE_NAME)
        .map_err(|_| AppError::RegistryValueMissing)?;
    Ok(PathBuf::from(value))
}

fn write_registry_config_path(path: &Path) -> Result<(), std::io::Error> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm.open_subkey_with_flags(REGISTRY_KEY_PATH, KEY_SET_VALUE | KEY_WOW64_64KEY)?;
    key.set_value(REGISTRY_VALUE_NAME, &path_to_string(path))?;
    Ok(())
}

fn run_elevated_registry_helper(path: &Path) -> Result<(), AppError> {
    let current_exe = env::current_exe()?;
    let command = format!(
        "$process = Start-Process -FilePath '{}' -ArgumentList @('--elevated-set-config-path','{}') -Verb RunAs -Wait -PassThru; exit $process.ExitCode",
        escape_for_powershell(current_exe.as_os_str().to_string_lossy().as_ref()),
        escape_for_powershell(path_to_string(path).as_str())
    );

    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            command.as_str(),
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(AppError::ElevationDeclined)
    }
}

fn list_group_names(presets_dir: &Path) -> Result<Vec<String>, AppError> {
    let mut names = fs::read_dir(presets_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect::<Vec<_>>();
    names.sort_unstable();
    Ok(names)
}

fn list_preset_names(group_dir: &Path) -> Result<Vec<String>, AppError> {
    if !group_dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = fs::read_dir(group_dir)?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("txt"))
        .filter_map(|entry| {
            entry
                .path()
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|value| value.to_string())
        })
        .collect::<Vec<_>>();
    names.sort_unstable();
    Ok(names)
}

fn validate_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::InvalidName(name.to_string()));
    }

    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    if trimmed.chars().any(|character| invalid.contains(&character)) {
        return Err(AppError::InvalidName(trimmed.to_string()));
    }

    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved
        .iter()
        .any(|reserved_name| reserved_name.eq_ignore_ascii_case(trimmed))
    {
        return Err(AppError::InvalidName(trimmed.to_string()));
    }

    Ok(trimmed.to_string())
}

fn sanitize_import_name(name: &str) -> String {
    let cleaned = name
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => ' ',
            other => other,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if cleaned.trim().is_empty() {
        "Imported Preset".to_string()
    } else {
        cleaned
    }
}

fn ensure_directory(path: &Path) -> Result<(), AppError> {
    fs::create_dir_all(path)?;
    Ok(())
}

fn is_directory_writable(path: &Path) -> Result<bool, AppError> {
    if let Err(error) = fs::create_dir_all(path) {
        return if error.kind() == std::io::ErrorKind::PermissionDenied {
            Ok(false)
        } else {
            Err(error.into())
        };
    }

    let probe = path.join(".write-test.tmp");
    match File::create(&probe) {
        Ok(mut file) => {
            file.write_all(b"probe")?;
            file.sync_all()?;
            fs::remove_file(probe)?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn write_text_file_atomically(path: &Path, content: &str) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        ensure_directory(parent)?;
    }

    let temporary_path = path.with_extension("tmp");
    {
        let mut file = File::create(&temporary_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
    }

    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temporary_path, path)?;
    Ok(())
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn relative_path(from_dir: &Path, target: &Path) -> Option<PathBuf> {
    use std::path::Component;

    let from_components = from_dir.components().collect::<Vec<_>>();
    let target_components = target.components().collect::<Vec<_>>();
    if from_components.is_empty() || target_components.is_empty() {
        return None;
    }

    let mut common_len = 0usize;
    while common_len < from_components.len()
        && common_len < target_components.len()
        && from_components[common_len] == target_components[common_len]
    {
        common_len += 1;
    }

    let same_root = matches!(
        (from_components.first(), target_components.first()),
        (Some(Component::Prefix(a)), Some(Component::Prefix(b))) if a == b
    ) || matches!(
        (from_components.first(), target_components.first()),
        (Some(Component::RootDir), Some(Component::RootDir))
    );

    if !same_root {
        return None;
    }

    let mut relative = PathBuf::new();
    for component in &from_components[common_len..] {
        match component {
            Component::Normal(_) | Component::CurDir | Component::ParentDir => {
                relative.push("..");
            }
            Component::Prefix(_) | Component::RootDir => {}
        }
    }

    for component in &target_components[common_len..] {
        relative.push(component.as_os_str());
    }

    Some(relative)
}

fn escape_for_powershell(value: &str) -> String {
    value.replace('\'', "''")
}
