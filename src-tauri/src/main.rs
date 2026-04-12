#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Some(exit_code) = smart_equalizer_apo_presets_manager_lib::try_handle_cli_mode() {
        std::process::exit(exit_code);
    }

    smart_equalizer_apo_presets_manager_lib::run();
}
