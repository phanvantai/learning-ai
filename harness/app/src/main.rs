//! Desktop shell: a window onto the local web app in `lai-harness`, plus what
//! only a desktop app can do (tray icon, notifications, starting with the computer).
//!
//! All logic lives in the library; the window simply loads the UI it serves.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_notification::NotificationExt;

const WINDOW: &str = "main";
/// Passed by the autostart entry so the app comes up in the tray, not in the user's face.
const HIDDEN_FLAG: &str = "--hidden";

struct AppHost(AppHandle);

impl lai_harness::Host for AppHost {
    fn notify(&self, title: &str, body: &str) {
        let _ = self.0.notification().builder().title(title).body(body).show();
    }
    fn autostart(&self) -> Option<bool> {
        Some(self.0.autolaunch().is_enabled().unwrap_or(false))
    }
    fn set_autostart(&self, enabled: bool) -> anyhow::Result<()> {
        let launcher = self.0.autolaunch();
        if enabled { launcher.enable() } else { launcher.disable() }.map_err(|e| anyhow::anyhow!("{e}"))
    }
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn main() {
    // The vendor CLIs start this same executable as the MCP server that
    // carries our tools; in that role there is no window at all.
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("mcp-serve") {
        let agent = args.iter().position(|a| a == "--agent").and_then(|i| args.get(i + 1));
        let result = match agent {
            Some(agent) => tauri::async_runtime::block_on(lai_harness::mcp_serve(agent)),
            None => Err(anyhow::anyhow!("thiếu --agent")),
        };
        if let Err(e) = result {
            eprintln!("Lỗi: {e:#}");
            std::process::exit(1);
        }
        return;
    }
    let start_hidden = args.iter().any(|a| a == HIDDEN_FLAG);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| show_window(app)))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![HIDDEN_FLAG])))
        .setup(move |app| {
            let handle = app.handle().clone();
            let url = tauri::async_runtime::block_on(lai_harness::server::start(Arc::new(AppHost(handle))))?;

            WebviewWindowBuilder::new(app, WINDOW, WebviewUrl::External(url.parse()?))
                .title("Trợ lý AI")
                .inner_size(1120.0, 780.0)
                .min_inner_size(420.0, 520.0)
                .visible(!start_hidden)
                .build()?;

            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "open", "Mở Trợ lý AI", true, None::<&str>)?,
                    &MenuItem::with_id(app, "quit", "Thoát", true, None::<&str>)?,
                ],
            )?;
            let mut tray = TrayIconBuilder::new().tooltip("Trợ lý AI").menu(&menu).on_menu_event(
                |app, event| match event.id().as_ref() {
                    "open" => show_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                },
            );
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            Ok(())
        })
        // Closing the window keeps the app (and scheduled jobs) alive in the tray.
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("không khởi động được app");

    app.run(|_app, _event| {
        // Clicking the Dock icon while the window is hidden brings it back.
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen { .. } = _event {
            show_window(_app);
        }
    });
}
