//! Windows toast notifications.
//!
//! `tauri-plugin-notification` only sets the WinRT AppUserModelID when running
//! the installed app (it skips it for anything launched from `target/debug` or
//! `target/release`), and it swallows the result of `show()` internally — so a
//! toast that fails to display (no registered AUMID, no Start Menu shortcut,
//! WinRT error) does so silently with nothing logged.
//!
//! We go straight to `notify-rust` instead, always set the app id, and log a
//! failure instead of swallowing it, so notifications work the same way in
//! dev and in the installed build.

use tauri::Runtime;

pub fn toast<R: Runtime>(app: &tauri::AppHandle<R>, title: &str, body: &str) {
    let identifier = app.config().identifier.clone();
    let mut notification = notify_rust::Notification::new();
    notification
        .summary(title)
        .body(body)
        .app_id(&identifier)
        .auto_icon();

    if let Err(e) = notification.show() {
        eprintln!("warn: toast notification failed ({title:?}): {e}");
    }
}
