use std::sync::Mutex;

mod commands;
mod store;
mod wallpaper;
mod virtual_desktop;

pub struct AppState {
    pub store: Mutex<store::Store>,
    pub desktop_manager: Mutex<virtual_desktop::DesktopManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(AppState {
            store: Mutex::new(store::Store::new()),
            desktop_manager: Mutex::new(virtual_desktop::DesktopManager::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_cpu_load,
            commands::get_memory,
            commands::get_disk,
            commands::get_battery,
            commands::get_cpu_temp,
            commands::get_network,
            commands::get_weather,
            commands::get_installed_apps,
            commands::launch_app,
            commands::get_media_sessions,
            commands::media_play,
            commands::media_pause,
            commands::media_next,
            commands::media_prev,
            commands::media_toggle,
            commands::get_theme,
            commands::set_theme,
            commands::get_accent_color,
            commands::set_accent_color,
            commands::get_store_value,
            commands::set_store_value,
            commands::get_notes,
            commands::save_notes,
            commands::screen_information,
            commands::pick_file,
            commands::pick_folder,
            commands::media_seek,
            commands::media_volume_up,
            commands::media_volume_down,
            commands::media_mute,
            commands::create_widget,
            commands::close_widget,
            commands::get_widget_windows,
            commands::open_launcher_window,
            commands::close_launcher_window,
            commands::toggle_launcher_window,
            commands::open_settings_window,
            commands::close_settings_window,
            commands::move_window,
            commands::get_window_position,
            commands::set_taskbar_config,
            commands::create_wallpaper_window,
            commands::start_video_server,
            commands::stop_video_server,
            commands::set_window_size,
            commands::get_desktops,
            commands::switch_desktop,
            commands::create_desktop,
            commands::delete_desktop,
            commands::rename_desktop,
            commands::enum_open_windows,
            commands::move_app_window,
            commands::show_app_window,
            commands::hide_app_window,
            commands::add_desktop_shortcut,
            commands::update_shortcut_position,
            commands::delete_desktop_shortcut,
            commands::show_desktop,
            commands::tile_current_desktop,
            commands::promote_master,
            commands::move_window_to_desktop,
            commands::assign_window_to_desktop,
            commands::get_auto_assign_windows,
            commands::set_auto_assign_windows,
            commands::get_app_desktop_map,
            commands::set_app_desktop_map,
            commands::get_window_under_cursor,
            commands::get_current_mouse_pos,
            commands::remove_window_from_all_desktops,
        ])
        .setup(|app| {
            use tauri::Manager;

            // Global shortcut - hata yut
            let _ = {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                app.global_shortcut().on_shortcut("ctrl+alt+t", move |app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
                            commands::toggle_settings_window(app.clone()).await
                        });
                    }
                })
            };

            // System tray menu
            if let Some(tray) = app.tray_by_id("main") {
                let _ = (|| -> Result<(), Box<dyn std::error::Error>> {
                    use tauri::menu::{MenuBuilder, MenuItemBuilder};
                    let show_settings = MenuItemBuilder::with_id("settings", "Ayarlar").build(app)?;
                    let show_launcher = MenuItemBuilder::with_id("launcher", "Başlatıcı").build(app)?;
                    let quit = MenuItemBuilder::with_id("quit", "Çıkış").build(app)?;
                    let menu = MenuBuilder::new(app)
                        .item(&show_settings)
                        .item(&show_launcher)
                        .separator()
                        .item(&quit)
                        .build()?;
                    tray.set_menu(Some(menu))?;
                    tray.on_menu_event(move |app, event| {
                        match event.id().as_ref() {
                            "settings" => {
                                let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
                                    commands::open_settings_window(app.clone()).await
                                });
                            }
                            "launcher" => {
                                let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
                                    commands::open_launcher_window(app.clone()).await
                                });
                            }
                            "quit" => { app.exit(0); }
                            _ => {}
                        }
                    });
                    Ok(())
                })();
            }

            // Taskbar pozisyonu
            if let Some(taskbar) = app.get_webview_window("taskbar") {
                if let Ok(Some(monitor)) = app.primary_monitor() {
                    let sw = monitor.size().width as i32;
                    let sh = monitor.size().height as i32;
                    let sf = monitor.scale_factor() as i32;
                    let _ = taskbar.set_position(tauri::PhysicalPosition::new(
                        (sw - 700 * sf) / 2,
                        sh - 48 * sf,
                    ));
                }
            }

            // Windows gorev cubugunu otomatik gizle
            #[cfg(target_os = "windows")]
            {
                crate::wallpaper::auto_hide_taskbar();
            }

            // Windows Snap ozelligini devre disi birak
            #[cfg(target_os = "windows")]
            {
                crate::virtual_desktop::disable_windows_snap();
            }

            // Edge transfer background task - 100ms aralikla kontrol
            let handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let transfers = crate::virtual_desktop::check_edge_transfer(1920, 1080, 48);
                for (hwnd, delta) in &transfers {
                    if *delta == 0 { continue; }
                    let state = handle.state::<AppState>();
                    let cur = state.desktop_manager.lock().unwrap().current;
                    let new_id = ((cur as i32) + delta).max(0) as usize;
                    drop(state);
                    let (h, d) = (*hwnd, new_id);
                    let h2 = handle.clone();
                    let _ = tokio::runtime::Runtime::new().unwrap().block_on(async move {
                        let _ = commands::assign_window_to_desktop(h2.state::<AppState>(), h, d).await;
                    });
                }
            });

            // Masaustu penceresini ac
            {
                let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
                    commands::show_desktop(app.handle().clone()).await
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
