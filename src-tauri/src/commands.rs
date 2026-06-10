//! IPC commands exposed to the webview frontends (main window + tray popup).
//! Thin wrappers over killport-core; all logic stays in core.

use std::sync::Mutex;

use killport_core::{autostart, config, kill_tree, restart, scan, Config, KillMode, PortProcess};
use qrcode::{Color as QrColor, QrCode};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize)]
pub struct QrCodeResult {
    pub url: String,
    pub size: usize,
    pub cells: Vec<bool>,
}

#[derive(Serialize)]
pub struct FirewallResult {
    pub has_allow_rule: bool,
    pub blocked: bool,
    pub rule_count: usize,
}

fn local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}

use crate::actions;

#[tauri::command]
pub fn list_ports(state: State<'_, Mutex<Config>>) -> Vec<PortProcess> {
    let ignore = state.lock().unwrap_or_else(|e| e.into_inner()).ignore_ports.clone();
    let ports = scan();
    if ignore.is_empty() {
        ports
    } else {
        ports.into_iter().filter(|p| !ignore.contains(&p.port)).collect()
    }
}

#[tauri::command]
pub fn kill_port(pid: u32) -> usize {
    kill_tree(pid, KillMode::Graceful).unwrap_or(0)
}

#[tauri::command]
pub fn restart_port(pid: u32) -> bool {
    let Some(p) = scan().into_iter().find(|p| p.pid == pid) else {
        return false;
    };
    let cmd = p.cmd.clone();
    let cwd = p.cwd.clone();
    let _ = kill_tree(pid, KillMode::Graceful);
    restart(&cmd, cwd.as_deref())
}

#[tauri::command]
pub fn open_url(port: u16) {
    actions::open_url(&format!("http://localhost:{port}"));
}

#[tauri::command]
pub fn copy_url(port: u16) {
    actions::copy(&format!("http://localhost:{port}"));
}

#[tauri::command]
pub fn open_folder(path: String) {
    // Validate path against known process directories from a fresh scan to prevent
    // a webview-supplied path from opening arbitrary folders.
    let known = scan();
    let req = std::path::Path::new(&path);
    let valid = known.iter().any(|p| {
        let cwd_match = p.cwd.as_deref().map(std::path::Path::new) == Some(req);
        let exe_match = p
            .exe
            .as_deref()
            .and_then(|e| std::path::Path::new(e).parent())
            == Some(req);
        cwd_match || exe_match
    });
    if valid {
        actions::open_folder(&path);
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, Mutex<Config>>) -> Config {
    state.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
pub fn set_config(state: State<'_, Mutex<Config>>, mut cfg: Config) {
    {
        let mut guard = state.lock().unwrap_or_else(|e| e.into_inner());
        // Reject port 0; dedup in case frontend sent duplicates.
        cfg.ignore_ports.retain(|&p| p > 0);
        cfg.ignore_ports.dedup();
        *guard = cfg.clone();
    }
    if let Err(e) = config::save(&cfg) {
        eprintln!("warn: config save failed: {e}");
    }
}

#[tauri::command]
pub fn set_autostart(enabled: bool) -> bool {
    let exe = std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    autostart::set(enabled, &exe).is_ok()
}

#[tauri::command]
pub fn get_autostart() -> bool {
    autostart::is_enabled()
}

/// Show the main dashboard window (created hidden at startup).
#[tauri::command]
pub fn open_main(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
    if let Some(t) = app.get_webview_window("tray") {
        let _ = t.hide();
    }
}

/// Hide the tray popup window.
#[tauri::command]
pub fn hide_tray(app: AppHandle) {
    if let Some(t) = app.get_webview_window("tray") {
        let _ = t.hide();
    }
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub async fn probe_port(port: u16) -> crate::probe::ProbeResult {
    crate::probe::probe(port).await
}

#[tauri::command]
pub fn get_qr_code(port: u16) -> Option<QrCodeResult> {
    let ip = local_ip()?;
    let url = format!("http://{}:{}", ip, port);
    let code = QrCode::new(url.as_bytes()).ok()?;
    let size = code.width();
    let cells: Vec<bool> = code.into_colors().into_iter().map(|c| c == QrColor::Dark).collect();
    Some(QrCodeResult { url, size, cells })
}

#[tauri::command]
pub fn kill_ports(pids: Vec<u32>) -> usize {
    pids.iter().map(|&pid| kill_tree(pid, KillMode::Graceful).unwrap_or(0)).sum()
}

#[tauri::command]
pub fn check_firewall(port: u16) -> FirewallResult {
    let mut builder = std::process::Command::new("netsh");
    builder.args([
        "advfirewall", "firewall", "show", "rule",
        "name=all", "dir=in", "protocol=tcp",
        &format!("localport={port}"),
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        builder.creation_flags(0x08000000);
    }
    let Ok(out) = builder.output() else {
        return FirewallResult { has_allow_rule: false, blocked: false, rule_count: 0 };
    };
    let text = String::from_utf8_lossy(&out.stdout);
    if text.contains("No rules match") {
        return FirewallResult { has_allow_rule: false, blocked: true, rule_count: 0 };
    }
    let has_allow = text.lines().any(|l| l.trim().starts_with("Action:") && l.contains("Allow"));
    let rule_count = text.lines().filter(|l| l.trim().starts_with("Rule Name:")).count();
    FirewallResult { has_allow_rule: has_allow, blocked: !has_allow, rule_count }
}
