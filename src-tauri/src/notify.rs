use tauri::Runtime;
use tauri_plugin_notification::NotificationExt;

pub fn toast<R: Runtime>(app: &tauri::AppHandle<R>, title: &str, body: &str) {
    let _ = app.notification().builder().title(title).body(body).show();
}
