mod commands;
pub mod crypto;
mod db;
pub mod error;
mod models;
mod state;
mod sync;

use std::path::PathBuf;

use tauri::Manager;

use crate::commands::{auth, device, feuilles, import_export, maintenance, password_gen, recovery, saladiers, settings};
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Resolve the app data directory for storing DB and device key
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));

            if let Err(e) = std::fs::create_dir_all(&data_dir) {
                log::error!("Failed to create application data directory: {e}");
                panic!("Failed to create application data directory: {e}");
            }

            let db_path = data_dir.join("saladvault.db");

            // Open the database.
            // All sensitive data is encrypted at the application level (XChaCha20-Poly1305)
            // before being stored, so the DB file contains only encrypted blobs.
            let conn = match db::open_database(&db_path) {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to open database: {e}");
                    panic!("Failed to open database: {e}");
                }
            };

            let app_state = AppState::new(conn, data_dir);
            app.manage(app_state);

            // Apply screenshot protection by default (desktop only)
            #[cfg(not(mobile))]
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_content_protected(true);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            auth::register,
            auth::unlock,
            auth::lock,
            auth::is_unlocked,
            auth::verify_master_password,
            auth::change_master_password,
            // Device commands
            device::init_device_key,
            device::check_device_key,
            device::get_device_key_path,
            device::move_device_key,
            device::export_device_key_qrcode,
            device::generate_device_key_qr_svg,
            device::regenerate_device_key,
            // Saladier commands
            saladiers::create_saladier,
            saladiers::list_saladiers,
            saladiers::open_saladier,
            saladiers::delete_saladier,
            saladiers::unlock_hidden_saladier,
            saladiers::get_saladier_attempts_info,
            // Feuille commands
            feuilles::create_feuille,
            feuilles::get_feuille,
            feuilles::list_feuilles,
            feuilles::update_feuille,
            feuilles::delete_feuille,
            // Recovery commands
            recovery::generate_recovery_phrase,
            recovery::recover_from_phrase,
            recovery::check_recovery_status,
            recovery::confirm_recovery_saved,
            // Settings commands
            settings::get_settings,
            settings::save_settings,
            settings::apply_screenshot_protection,
            settings::write_to_clipboard,
            settings::clear_clipboard,
            settings::update_last_activity,
            settings::get_inactivity_seconds,
            // Import/Export commands
            import_export::import_passwords,
            import_export::export_encrypted_json,
            import_export::export_csv_clear,
            // Maintenance commands
            maintenance::vacuum_database,
            maintenance::check_integrity,
            maintenance::check_for_update,
            maintenance::install_update,
            // Password generator
            password_gen::generate_password,
            // Sync commands
            sync::commands::server_register,
            sync::commands::server_register_confirm_mfa,
            sync::commands::server_login,
            sync::commands::server_login_verify_mfa,
            sync::commands::server_logout,
            sync::commands::server_is_connected,
            sync::commands::server_delete_account,
            sync::commands::server_send_verification,
            sync::commands::server_verify_code,
            sync::commands::sync_status,
            sync::commands::sync_push,
            sync::commands::sync_pull,
            sync::commands::deadman_status,
            sync::commands::deadman_heartbeat,
            sync::commands::deadman_update_config,
            sync::commands::generate_recovery_kit,
            sync::commands::subscription_status,
            sync::commands::subscription_checkout,
            sync::commands::subscription_portal,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
