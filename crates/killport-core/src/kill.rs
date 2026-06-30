//! Process termination. Graceful path posts WM_CLOSE to the target's windows and
//! waits briefly; console dev servers (no window) fall straight through to force.
//! `kill_tree` reaches child processes (npm -> node) so the port is actually freed.

use crate::guard;
use crate::process::{info_for, snapshot};
use tracing::{debug, warn};

#[derive(Debug, thiserror::Error)]
pub enum KillError {
    #[error("process is protected")]
    Protected,
    #[error("PID {0} is a system process")]
    SystemPid(u32),
    #[error("PID {0} was reused after snapshot")]
    PidReused(u32),
    #[error("OpenProcess failed for PID {0}")]
    OpenFailed(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KillMode {
    /// Try WM_CLOSE first, then force after a short timeout.
    Graceful,
    /// TerminateProcess immediately.
    Force,
}

/// Never touch obvious system PIDs.
const MIN_KILLABLE_PID: u32 = 10;

/// Kill a single process. Returns Ok(true) if it is gone afterwards.
pub fn kill(pid: u32, mode: KillMode) -> Result<bool, KillError> {
    let name = info_for(pid).map(|i| i.name).unwrap_or_default();
    kill_one(pid, &name, mode)
}

fn kill_one(pid: u32, name: &str, mode: KillMode) -> Result<bool, KillError> {
    if pid < MIN_KILLABLE_PID {
        warn!(pid, "kill rejected: system PID");
        return Err(KillError::SystemPid(pid));
    }
    if guard::is_protected(name) {
        warn!(pid, name, "kill rejected: protected process");
        return Err(KillError::Protected);
    }
    #[cfg(windows)]
    {
        if mode == KillMode::Graceful {
            let windows = windows_of(pid);
            if !windows.is_empty() {
                for hwnd in &windows {
                    post_close(*hwnd);
                }
                let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1500);
                while std::time::Instant::now() < deadline {
                    if !is_alive(pid) {
                        return Ok(true);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        }
        if terminate(pid) {
            return Ok(true);
        }
        Ok(!is_alive(pid))
    }
    #[cfg(not(windows))]
    {
        let _ = (pid, mode);
        Ok(false)
    }
}

/// Kill `pid` and all its descendants (leaves first). Returns Ok(count) terminated.
/// Returns Err if the root pid is a protected or system process.
pub fn kill_tree(pid: u32, mode: KillMode) -> Result<usize, KillError> {
    use std::collections::{HashMap, HashSet};

    let snap = snapshot();
    let mut children: HashMap<u32, Vec<u32>> = HashMap::new();
    for info in snap.values() {
        if let Some(parent) = info.parent {
            children.entry(parent).or_default().push(info.pid);
        }
    }

    // BFS to collect the subtree; root is discovered first.
    let mut order = Vec::new();
    let mut seen = HashSet::new();
    let mut stack = vec![pid];
    while let Some(cur) = stack.pop() {
        if !seen.insert(cur) {
            continue;
        }
        order.push(cur);
        if let Some(kids) = children.get(&cur) {
            stack.extend(kids);
        }
    }

    // Guard-check root before attempting any kills.
    let root_name = snap.get(&pid).map(|i| i.name.as_str()).unwrap_or("");
    if pid < MIN_KILLABLE_PID {
        return Err(KillError::SystemPid(pid));
    }
    if guard::is_protected(root_name) {
        return Err(KillError::Protected);
    }

    // Capture creation times before killing to detect PID reuse.
    let creation_times: HashMap<u32, u64> = order
        .iter()
        .filter_map(|&p| process_creation_time(p).map(|t| (p, t)))
        .collect();

    // Reverse => deepest-discovered first, root last.
    let mut killed = 0;
    for p in order.into_iter().rev() {
        // Fail closed: only kill a PID whose creation time was captured at snapshot
        // time AND still matches right now. If either read is unavailable (process
        // already gone, access denied, or it never resolved at snapshot time), we
        // cannot prove this PID still refers to the same process, so skip it rather
        // than risk killing a process that was recycled into this PID since the scan.
        match (creation_times.get(&p), process_creation_time(p)) {
            (Some(&expected), Some(current)) if expected == current => {}
            _ => {
                warn!(
                    pid = p,
                    "skipping kill: PID identity unconfirmed (reused or unreadable)"
                );
                continue;
            }
        }
        // Re-resolve the live process name right before killing instead of trusting
        // the pre-kill snapshot, so the guard check reflects what is actually at
        // this PID right now.
        let name = info_for(p).map(|i| i.name).unwrap_or_default();
        match kill_one(p, &name, mode) {
            Ok(true) => {
                debug!(pid = p, name, "killed");
                killed += 1;
            }
            Ok(false) => warn!(
                pid = p,
                "kill_one returned false; process may still be alive"
            ),
            Err(e) => debug!(pid = p, err = %e, "kill_one skipped"),
        }
    }
    Ok(killed)
}

/// Returns the process creation time as a u64 FILETIME value, or None if unavailable.
/// Used to detect PID reuse between snapshot and kill.
#[cfg(windows)]
fn process_creation_time(pid: u32) -> Option<u64> {
    use windows::Win32::Foundation::{CloseHandle, FILETIME};
    use windows::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        if handle.is_invalid() {
            return None;
        }
        let mut created = FILETIME::default();
        let mut exited = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        let ok = GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user).is_ok();
        let _ = CloseHandle(handle);
        if ok {
            Some(((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64)
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
fn process_creation_time(_pid: u32) -> Option<u64> {
    None
}

/// Returns whether `pid` is still running. Defaults to "alive" when the liveness
/// check itself is inconclusive (e.g. access denied), so a process we failed to
/// confirm dead is never mistakenly reported as successfully killed.
#[cfg(windows)]
fn is_alive(pid: u32) -> bool {
    use windows::Win32::Foundation::{CloseHandle, ERROR_ACCESS_DENIED, WAIT_TIMEOUT};
    use windows::Win32::System::Threading::{
        OpenProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    unsafe {
        match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(handle) if !handle.is_invalid() => {
                let r = WaitForSingleObject(handle, 0);
                let _ = CloseHandle(handle);
                r == WAIT_TIMEOUT
            }
            Err(e) if e.code() == ERROR_ACCESS_DENIED.to_hresult() => true,
            _ => false,
        }
    }
}

#[cfg(windows)]
fn terminate(pid: u32) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
    unsafe {
        match OpenProcess(PROCESS_TERMINATE, false, pid) {
            Ok(handle) if !handle.is_invalid() => {
                let ok = TerminateProcess(handle, 1).is_ok();
                let _ = CloseHandle(handle);
                ok
            }
            _ => false,
        }
    }
}

#[cfg(windows)]
struct EnumCtx {
    pid: u32,
    windows: Vec<windows::Win32::Foundation::HWND>,
}

#[cfg(windows)]
fn windows_of(pid: u32) -> Vec<windows::Win32::Foundation::HWND> {
    use windows::Win32::Foundation::{BOOL, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::EnumWindows;

    let mut ctx = EnumCtx {
        pid,
        windows: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut ctx as *mut _ as isize));
    }
    let _ = BOOL::default();
    ctx.windows
}

#[cfg(windows)]
unsafe extern "system" fn enum_proc(
    hwnd: windows::Win32::Foundation::HWND,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::BOOL {
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
    let ctx = &mut *(lparam.0 as *mut EnumCtx);
    let mut wpid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut wpid));
    if wpid == ctx.pid {
        ctx.windows.push(hwnd);
    }
    true.into()
}

#[cfg(windows)]
fn post_close(hwnd: windows::Win32::Foundation::HWND) {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_CLOSE};
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_pid_rejected() {
        let result = kill(1, KillMode::Graceful);
        assert!(matches!(result, Err(KillError::SystemPid(1))));
    }

    #[test]
    fn protected_name_rejected() {
        let result = kill_one(1000, "svchost.exe", KillMode::Graceful);
        assert!(matches!(result, Err(KillError::Protected)));
    }

    #[test]
    fn kill_tree_protected_root_errors() {
        // kill_tree on a protected name must return Err, not Ok(0).
        // We can't easily inject a pid, so use pid=1 (always SystemPid).
        let result = kill_tree(1, KillMode::Graceful);
        assert!(matches!(result, Err(KillError::SystemPid(1))));
    }

    #[cfg(windows)]
    #[test]
    fn kill_tree_terminates_child_process() {
        use std::process::Command;

        // Spawn a long-running child we can safely kill in tests.
        let mut child = Command::new("ping")
            .args(["-n", "30", "127.0.0.1"])
            .spawn()
            .expect("failed to spawn ping");
        let pid = child.id();

        let result = kill_tree(pid, KillMode::Force);
        assert!(
            matches!(result, Ok(n) if n >= 1),
            "expected at least 1 process killed, got {result:?}"
        );
        // PID should no longer be alive.
        assert!(!is_alive(pid), "process {pid} still alive after kill_tree");
        // Reap the child to satisfy the OS and avoid clippy's must-use warning.
        child.wait().ok();
    }
}
