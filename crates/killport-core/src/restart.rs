//! Smart restart: re-spawn a process from its captured command line + cwd.
//! Capture cmd/cwd BEFORE killing (the process is gone afterwards).

use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn restart(cmd: &[String], cwd: Option<&str>) -> bool {
    let Some((program, args)) = cmd.split_first() else {
        return false;
    };
    let mut c = Command::new(program);
    c.args(args);
    if let Some(dir) = cwd {
        c.current_dir(dir);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c.spawn().is_ok()
}
