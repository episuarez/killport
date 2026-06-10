//! Killport tray app. Headless at startup (no window shown); the tray icon opens
//! a frameless popup window, and "open app" shows the main dashboard window.
//! All logic lives in killport-core; this shell wires windows, tray, and IPC.

mod actions;
mod commands;
mod notify;
mod probe;

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use killport_core::{config, scan_fast, Config, PortProcess};
use tauri::{
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewUrl, WebviewWindowBuilder,
    WindowEvent,
};

const TRAY_ID: &str = "main";

fn show_tray_popup<R: Runtime>(app: &AppHandle<R>, cursor: PhysicalPosition<f64>) {
    let Some(win) = app.get_webview_window("tray") else {
        return;
    };
    let size = win.outer_size().unwrap_or(PhysicalSize::new(320, 540));
    // Anchor the popup's bottom-right near the click (taskbar is usually bottom).
    let x = (cursor.x - size.width as f64).max(0.0);
    let y = (cursor.y - size.height as f64).max(0.0);
    let _ = win.set_position(PhysicalPosition::new(x, y));
    let _ = win.show();
    let _ = win.set_focus();
}

fn dev_ports(ports: &[PortProcess]) -> HashSet<u16> {
    ports
        .iter()
        .filter(|p| !p.is_system && p.kind != "unknown")
        .map(|p| p.port)
        .collect()
}

fn reserved_occupied(ports: &[PortProcess], reserved: &[u16]) -> HashSet<u16> {
    if reserved.is_empty() {
        return HashSet::new();
    }
    ports
        .iter()
        .filter(|p| reserved.contains(&p.port))
        .map(|p| p.port)
        .collect()
}

fn filtered_scan<R: Runtime>(app: &AppHandle<R>) -> Vec<PortProcess> {
    let ignore = app
        .state::<Mutex<Config>>()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .ignore_ports
        .clone();
    let ports = scan_fast();
    if ignore.is_empty() {
        ports
    } else {
        ports
            .into_iter()
            .filter(|p| !ignore.contains(&p.port))
            .collect()
    }
}

fn spawn_poll_loop<R: Runtime>(app: AppHandle<R>, shutdown: Arc<AtomicBool>) {
    std::thread::spawn(move || {
        let initial = filtered_scan(&app);
        let mut prev = dev_ports(&initial);
        let mut prev_reserved = {
            let cfg = app
                .state::<Mutex<Config>>()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            reserved_occupied(&initial, &cfg.reserved_ports)
        };
        loop {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            let cfg = app
                .state::<Mutex<Config>>()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            std::thread::sleep(Duration::from_secs(cfg.poll_interval_secs.clamp(1, 300)));

            let current = filtered_scan(&app);
            let cur = dev_ports(&current);

            // Reserved port occupancy changes (independent of dev-port filter).
            let cur_reserved = reserved_occupied(&current, &cfg.reserved_ports);
            if cfg.notifications {
                for &rp in &cfg.reserved_ports {
                    let now = cur_reserved.contains(&rp);
                    let was = prev_reserved.contains(&rp);
                    if now && !was {
                        if let Some(p) = current.iter().find(|p| p.port == rp) {
                            let fw = p.framework.as_deref().unwrap_or(&p.kind);
                            notify::toast(
                                &app,
                                "Puerto reservado ocupado",
                                &format!(":{rp} — {fw} ({})", p.name),
                            );
                        }
                    } else if !now && was {
                        notify::toast(&app, "Puerto reservado liberado", &format!(":{rp}"));
                    }
                }
            }
            prev_reserved = cur_reserved;

            if cur == prev {
                continue;
            }

            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                let _ = tray.set_tooltip(Some(&format!("Killport — {} dev port(s)", cur.len())));
            }
            if cfg.notifications {
                for p in current
                    .iter()
                    .filter(|p| !p.is_system && p.kind != "unknown" && !prev.contains(&p.port))
                {
                    let fw = p.framework.as_deref().unwrap_or(&p.kind);
                    notify::toast(
                        &app,
                        "Puerto abierto",
                        &format!(":{} — {} ({})", p.port, fw, p.name),
                    );
                }
                for port in prev.difference(&cur) {
                    notify::toast(&app, "Puerto cerrado", &format!(":{port}"));
                }
            }
            prev = cur;
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // TODO: add tauri-plugin-log with APPDATA\Killport\logs appender and wire
    // the tracing subscriber here so kill/config spans are persisted to disk.
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(Mutex::new(config::load()))
        .invoke_handler(tauri::generate_handler![
            commands::list_ports,
            commands::kill_port,
            commands::restart_port,
            commands::open_url,
            commands::copy_url,
            commands::open_folder,
            commands::get_config,
            commands::set_config,
            commands::set_autostart,
            commands::get_autostart,
            commands::open_main,
            commands::hide_tray,
            commands::quit_app,
            commands::probe_port,
            commands::get_qr_code,
            commands::kill_ports,
            commands::check_firewall,
        ])
        .on_window_event(|window, event| match (window.label(), event) {
            ("tray", WindowEvent::Focused(false)) => {
                let _ = window.hide();
            }
            ("main", WindowEvent::CloseRequested { api, .. }) => {
                api.prevent_close();
                let _ = window.hide();
            }
            _ => {}
        })
        .setup(|app| {
            let handle = app.handle().clone();

            WebviewWindowBuilder::new(&handle, "main", WebviewUrl::App("index.html".into()))
                .title("Killport")
                .inner_size(1200.0, 780.0)
                .min_inner_size(900.0, 600.0)
                .visible(false)
                .build()?;

            WebviewWindowBuilder::new(&handle, "tray", WebviewUrl::App("tray.html".into()))
                .inner_size(320.0, 540.0)
                .decorations(false)
                .resizable(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .visible(false)
                .build()?;

            let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/icon.png"))?;
            TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon)
                .tooltip("Killport")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { position, .. } = event {
                        show_tray_popup(tray.app_handle(), position);
                    }
                })
                .build(app)?;

            let shutdown = Arc::new(AtomicBool::new(false));
            app.manage(shutdown.clone());
            spawn_poll_loop(handle, shutdown);
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Killport failed to start: {e}");
            std::process::exit(1);
        });
}
