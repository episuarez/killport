# Notifications

**Module:** `src-tauri/src/notify.rs`

Windows toast notifications for port lifecycle events.

## Implementation

Uses `tauri-winrt-notification` which wraps the Windows Runtime `ToastNotificationManager`. Notifications appear in the Windows Action Center and support the system's Do Not Disturb mode.

## Events

| Event | Trigger |
|-------|---------|
| Port opened | A new port appears in the scan that wasn't there before |
| Port closed | A previously tracked port disappears from the scan |

Both are detected by comparing consecutive `scan_fast()` results in the polling loop.

## Content

Each notification shows:
- App name / process name
- Port number
- Framework label (if detected)

## No interaction required

Notifications are fire-and-forget. There are no action buttons on the toasts (no "Kill" button in the notification). Actions are performed from the tray popup or main window.

## Desktop-only

`notify.rs` lives in `src-tauri`, not in `killport-core`. The CLI has no notifications.
