mod commands;
mod state;

use tauri::{
    menu::{CheckMenuItemBuilder, Menu, MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, State, WindowEvent,
};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::{
    commands::{
        apply_preset, attach_convolution_wav, create_group, create_preset, delete_group,
        delete_preset, export_app_settings, export_preset, get_config_path, import_app_settings,
        import_presets, load_presets, move_preset, rebuild_tray_menu, remove_convolution_wav,
        rename_group, rename_preset, reorder_groups, reveal_path_in_explorer, save_preset,
        set_config_path, set_group_emoji,
    },
    state::{AppError, AppState, PresetLibrary, TraySelection, EVENT_PRESETS_UPDATED},
};

const TRAY_ID: &str = "smart-equalizer-tray";
const WINDOW_LABEL: &str = "main";
const MENU_ID_MANAGE: &str = "menu.manage";
const MENU_ID_ABOUT: &str = "menu.about";
const MENU_ID_EXIT: &str = "menu.exit";
const MENU_ID_EMPTY_GROUPS: &str = "menu.empty-groups";
const MENU_ID_EMPTY_PRESETS_PREFIX: &str = "menu.empty-presets";

pub fn try_handle_cli_mode() -> Option<i32> {
    state::try_handle_cli_mode()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let state = AppState::initialize()?;
            app.manage(state);

            let menu = construct_tray_menu(app.handle())?;
            let icon = app
                .default_window_icon()
                .cloned()
                .ok_or(AppError::MissingIcon)?;

            let _tray = TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon)
                .tooltip("SmartEqualizer APO Presets Manager")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(handle_tray_menu_event)
                .on_tray_icon_event(handle_tray_icon_event)
                .build(app)?;

            configure_main_window(app.handle())?;
            maybe_prompt_for_config_migration(app.handle())?;
            let _ = refresh_runtime(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config_path,
            set_config_path,
            load_presets,
            apply_preset,
            save_preset,
            create_group,
            set_group_emoji,
            rename_group,
            delete_group,
            reorder_groups,
            create_preset,
            rename_preset,
            delete_preset,
            move_preset,
            import_presets,
            attach_convolution_wav,
            remove_convolution_wav,
            export_app_settings,
            import_app_settings,
            export_preset,
            rebuild_tray_menu,
            reveal_path_in_explorer
        ]);

    if let Err(error) = builder.run(tauri::generate_context!()) {
        eprintln!("{error}");
    }
}

pub(crate) fn refresh_runtime<R: Runtime>(app: &AppHandle<R>) -> Result<PresetLibrary, AppError> {
    rebuild_native_tray_menu(app)?;

    let snapshot = {
        let state: State<'_, AppState> = app.state();
        let mut guard = state.lock()?;
        guard.snapshot()?
    };

    app.emit(EVENT_PRESETS_UPDATED, snapshot.clone())?;
    Ok(snapshot)
}

fn configure_main_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        let window_clone = window.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(error) = window_clone.hide() {
                    eprintln!("{error}");
                }
            }
        });
    }

    Ok(())
}

fn maybe_prompt_for_config_migration<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let (should_prompt_migration, default_path) = {
        let state: State<'_, AppState> = app.state();
        let guard = state.lock()?;
        (
            guard.should_prompt_for_config_migration()?,
            guard.default_config_path_string(),
        )
    };

    if !should_prompt_migration {
        return Ok(());
    }

    let accepted = app
        .dialog()
        .message(format!(
            "Equalizer APO is currently configured to use a protected config folder.\n\nSmartEqualizerAPOPresetsManager works best with a writable config path:\n{default_path}\n\nLet the app switch Equalizer APO to that location now?"
        ))
        .title("Move Equalizer APO ConfigPath")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Switch Now".to_string(),
            "Keep Current Path".to_string(),
        ))
        .blocking_show();

    {
        let state: State<'_, AppState> = app.state();
        let mut guard = state.lock()?;
        guard.mark_config_path_prompted(true)?;
    }

    if accepted {
        let update_result = {
            let state: State<'_, AppState> = app.state();
            let mut guard = state.lock()?;
            guard.set_config_path(std::path::PathBuf::from(default_path))
        };

        match update_result {
            Ok(()) => {
                let _ = refresh_runtime(app);
            }
            Err(error) => {
                app.dialog()
                    .message(format!(
                        "The config path was not changed.\n\n{}\n\nYou can try again later from the main window.",
                        error
                    ))
                    .title("Config Path Update Failed")
                    .kind(MessageDialogKind::Error)
                    .blocking_show();
            }
        }
    }

    Ok(())
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        if window.is_minimized()? {
            window.unminimize()?;
        }
        window.show()?;
        window.set_focus()?;
    }

    Ok(())
}

fn show_about_dialog<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let snapshot = {
        let state: State<'_, AppState> = app.state();
        let mut guard = state.lock()?;
        guard.snapshot()?
    };

    app.dialog()
        .message(format!(
            "SmartEqualizerAPOPresetsManager\n\nWindows 11 tray-first preset manager for Equalizer APO.\n\nConfig path:\n{}\n\nGroups: {}\nPresets: {}",
            snapshot.config_path,
            snapshot.groups.len(),
            snapshot.groups.iter().map(|group| group.presets.len()).sum::<usize>()
        ))
        .title("About SmartEqualizer APO")
        .kind(MessageDialogKind::Info)
        .blocking_show();

    Ok(())
}

fn handle_tray_menu_event<R: Runtime>(app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
    let result = match event.id().as_ref() {
        MENU_ID_MANAGE => show_main_window(app),
        MENU_ID_ABOUT => show_about_dialog(app),
        MENU_ID_EXIT => {
            app.exit(0);
            Ok(())
        }
        item_id => apply_from_tray(app, item_id),
    };

    if let Err(error) = result {
        app.dialog()
            .message(error.to_string())
            .title("SmartEqualizer APO")
            .kind(MessageDialogKind::Error)
            .blocking_show();
    }
}

fn handle_tray_icon_event<R: Runtime>(tray: &tauri::tray::TrayIcon<R>, event: TrayIconEvent) {
    if let TrayIconEvent::DoubleClick {
        button: MouseButton::Left,
        ..
    } = event
    {
        if let Err(error) = show_main_window(tray.app_handle()) {
            eprintln!("{error}");
        }
    }
}

fn apply_from_tray<R: Runtime>(app: &AppHandle<R>, item_id: &str) -> Result<(), AppError> {
    let selection = {
        let state: State<'_, AppState> = app.state();
        let guard = state.lock()?;
        guard.resolve_tray_selection(item_id)?
    };

    {
        let state: State<'_, AppState> = app.state();
        let mut guard = state.lock()?;
        guard.apply_preset(&selection.group, &selection.preset)?;
    }

    let _ = refresh_runtime(app)?;
    Ok(())
}

fn rebuild_native_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Result<(), AppError> {
    let menu = construct_tray_menu(app)?;
    let tray = app.tray_by_id(TRAY_ID).ok_or(AppError::MissingTray)?;
    tray.set_menu(Some(menu))?;
    Ok(())
}

fn construct_tray_menu<R: Runtime>(app: &AppHandle<R>) -> Result<Menu<R>, AppError> {
    let (snapshot, targets) = {
        let state: State<'_, AppState> = app.state();
        let mut guard = state.lock()?;
        let snapshot = guard.snapshot()?;
        let targets = build_tray_targets(&snapshot);
        guard.replace_tray_targets(targets.clone());
        (snapshot, targets)
    };

    let presets_submenu = build_presets_submenu(app, &snapshot, &targets)?;
    let manage_item = MenuItemBuilder::with_id(MENU_ID_MANAGE, "Manage Presets...").build(app)?;
    let about_item = MenuItemBuilder::with_id(MENU_ID_ABOUT, "About...").build(app)?;
    let exit_item = MenuItemBuilder::with_id(MENU_ID_EXIT, "Exit").build(app)?;
    let separator = PredefinedMenuItem::separator(app)?;

    MenuBuilder::new(app)
        .items(&[&presets_submenu, &manage_item, &separator, &about_item, &exit_item])
        .build()
        .map_err(AppError::from)
}

fn build_presets_submenu<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &PresetLibrary,
    targets: &[(String, TraySelection)],
) -> Result<tauri::menu::Submenu<R>, AppError> {
    if snapshot.groups.is_empty() {
        let empty_item = MenuItemBuilder::with_id(MENU_ID_EMPTY_GROUPS, "No presets available")
            .enabled(false)
            .build(app)?;
        return SubmenuBuilder::new(app, "Presets")
            .item(&empty_item)
            .build()
            .map_err(AppError::from);
    }

    let mut builder = SubmenuBuilder::new(app, "Presets");

    let active_label = active_preset_label(snapshot);
    let active_item = MenuItemBuilder::with_id("menu.active.current", active_label.as_str())
    .enabled(false)
    .build(app)?;
    builder = builder.item(&active_item);

    for group in &snapshot.groups {
        let group_label = menu_group_label(group);
        let mut group_builder = SubmenuBuilder::new(app, group_label.as_str());

        if group.presets.is_empty() {
            let empty_id = format!("{MENU_ID_EMPTY_PRESETS_PREFIX}.{}", group.name);
            let empty = MenuItemBuilder::with_id(empty_id, "No presets yet")
                .enabled(false)
                .build(app)?;
            group_builder = group_builder.item(&empty);
        } else {
            for preset in &group.presets {
                let menu_id = targets
                    .iter()
                    .find(|(_, selection)| selection.group == group.name && selection.preset == preset.name)
                    .map(|(id, _)| id.as_str())
                    .ok_or_else(|| AppError::UnknownMenuItem(format!("{}/{}", group.name, preset.name)))?;
                let item = CheckMenuItemBuilder::with_id(menu_id, &preset.name)
                    .checked(group.active_preset.as_deref() == Some(preset.name.as_str()))
                    .build(app)?;
                group_builder = group_builder.item(&item);
            }
        }

        let submenu = group_builder.build()?;
        builder = builder.item(&submenu);
    }

    builder.build().map_err(AppError::from)
}

fn build_tray_targets(snapshot: &PresetLibrary) -> Vec<(String, TraySelection)> {
    let mut targets = Vec::new();
    let mut index = 0usize;

    for group in &snapshot.groups {
        for preset in &group.presets {
            targets.push((
                format!("preset.{index}"),
                TraySelection {
                    group: group.name.clone(),
                    preset: preset.name.clone(),
                },
            ));
            index += 1;
        }
    }

    targets
}

fn menu_group_label(group: &crate::state::PresetGroup) -> String {
    match group.emoji.as_deref().map(str::trim).filter(|emoji| !emoji.is_empty()) {
        Some(emoji) => format!("{emoji} {}", group.name),
        None => group.name.clone(),
    }
}

fn active_preset_label(snapshot: &PresetLibrary) -> String {
    snapshot
        .groups
        .iter()
        .find_map(|group| {
            group
                .active_preset
                .as_deref()
                .map(|preset| format!("Active: {preset}"))
        })
        .unwrap_or_else(|| "Active: None".to_string())
}
