//! OS-side side effects driven from the tray. Kept out of killport-core (core stays pure).

use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn spawn(program: &str, args: &[&str]) {
    let _ = Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn();
}

pub fn open_url(url: &str) {
    // `cmd /C start "" <url>` opens the default browser.
    spawn("cmd", &["/C", "start", "", url]);
}

pub fn open_folder(path: &str) {
    spawn("explorer", &[path]);
}

pub fn copy(text: &str) {
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text.to_string());
    }
}
