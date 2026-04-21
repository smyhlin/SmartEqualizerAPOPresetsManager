import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import type { AppRuntimeSettings, LogSnapshot, PresetLibrary } from '$lib/types';

export const PRESETS_UPDATED_EVENT = 'smart-equalizer://presets-updated';
export const SETTINGS_UPDATED_EVENT = 'smart-equalizer://settings-updated';

export function getConfigPath() {
  return invoke<string>('get_config_path');
}

export function setConfigPath(newPath: string) {
  return invoke<PresetLibrary>('set_config_path', { newPath });
}

export function installOrReinstallApo() {
  return invoke<PresetLibrary>('install_or_reinstall_apo');
}

export function openApoDeviceSelector() {
  return invoke<void>('open_apo_device_selector');
}

export function openRepositoryUrl() {
  return invoke<void>('open_repository_url');
}

export function loadLogs() {
  return invoke<LogSnapshot>('load_logs');
}

export function openLogsLocation() {
  return invoke<void>('open_logs_location');
}

export function loadPresets() {
  return invoke<PresetLibrary>('load_presets');
}

export function applyPreset(group: string, name: string) {
  return invoke<PresetLibrary>('apply_preset', { group, name });
}

export function savePreset(group: string, name: string, content: string) {
  return invoke<PresetLibrary>('save_preset', { group, name, content });
}

export function createGroup(name: string) {
  return invoke<PresetLibrary>('create_group', { name });
}

export function setGroupEmoji(group: string, emoji: string | null) {
  return invoke<PresetLibrary>('set_group_emoji', { group, emoji });
}

export function renameGroup(oldName: string, newName: string) {
  return invoke<PresetLibrary>('rename_group', { oldName, newName });
}

export function deleteGroup(name: string) {
  return invoke<PresetLibrary>('delete_group', { name });
}

export function reorderGroups(order: string[]) {
  return invoke<PresetLibrary>('reorder_groups', { order });
}

export function createPreset(group: string, name: string, content: string) {
  return invoke<PresetLibrary>('create_preset', { group, name, content });
}

export function renamePreset(group: string, oldName: string, newName: string) {
  return invoke<PresetLibrary>('rename_preset', { group, oldName, newName });
}

export function deletePreset(group: string, name: string) {
  return invoke<PresetLibrary>('delete_preset', { group, name });
}

export function movePreset(
  oldGroup: string,
  newGroup: string,
  name: string,
  targetIndex?: number
) {
  return invoke<PresetLibrary>('move_preset', {
    oldGroup,
    newGroup,
    name,
    targetIndex
  });
}

export function importPresets(group: string, paths: string[]) {
  return invoke<PresetLibrary>('import_presets', { group, paths });
}

export function attachConvolutionWav(
  group: string,
  name: string,
  content: string,
  sourcePath: string
) {
  return invoke<PresetLibrary>('attach_convolution_wav', {
    group,
    name,
    content,
    sourcePath
  });
}

export function removeConvolutionWav(group: string, name: string, content: string) {
  return invoke<PresetLibrary>('remove_convolution_wav', { group, name, content });
}

export function exportPreset(group: string, name: string, destination: string) {
  return invoke<string>('export_preset', { group, name, destination });
}

export function exportAppSettings(destination: string) {
  return invoke<void>('export_app_settings', { destination });
}

export function importAppSettings(source: string) {
  return invoke<PresetLibrary>('import_app_settings', { source });
}

export function rebuildTrayMenu() {
  return invoke<PresetLibrary>('rebuild_tray_menu');
}

export function getAutorunEnabled() {
  return invoke<boolean>('get_autorun_enabled');
}

export function setAutorunEnabled(enabled: boolean) {
  return invoke<boolean>('set_autorun_enabled', { enabled });
}

export function revealPathInExplorer(path: string) {
  return invoke<void>('reveal_path_in_explorer', { path });
}

export function onPresetsUpdated(callback: (payload: PresetLibrary) => void) {
  return listen<PresetLibrary>(PRESETS_UPDATED_EVENT, (event) => {
    callback(event.payload);
  });
}

export function onRuntimeSettingsUpdated(
  callback: (payload: AppRuntimeSettings) => void
) {
  return listen<AppRuntimeSettings>(SETTINGS_UPDATED_EVENT, (event) => {
    callback(event.payload);
  });
}
