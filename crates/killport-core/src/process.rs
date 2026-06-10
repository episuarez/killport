//! Process metadata (command line, cwd) via sysinfo.
//! Command-line extraction is the load-bearing spike: it tells `npm run dev`
//! from a bare `node.exe`. Some elevated/other-user processes may refuse.

use std::collections::HashMap;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

#[derive(Debug, Clone)]
pub struct ProcInfo {
    pub pid: u32,
    pub parent: Option<u32>,
    pub name: String,
    pub exe: Option<String>,
    pub cmd: Vec<String>,
    pub cwd: Option<String>,
}

/// Snapshot all processes once, keyed by pid. Caller joins with ports.
pub fn snapshot() -> HashMap<u32, ProcInfo> {
    let mut sys = System::new();
    let refresh = ProcessRefreshKind::nothing()
        .with_cmd(UpdateKind::Always)
        .with_exe(UpdateKind::Always)
        .with_cwd(UpdateKind::Always);
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh);

    let mut map = HashMap::new();
    for (pid, proc_) in sys.processes() {
        let info = ProcInfo {
            pid: pid.as_u32(),
            parent: proc_.parent().map(|p| p.as_u32()),
            name: proc_.name().to_string_lossy().into_owned(),
            exe: proc_.exe().map(|p| p.to_string_lossy().into_owned()),
            cmd: proc_
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect(),
            cwd: proc_.cwd().map(|p| p.to_string_lossy().into_owned()),
        };
        map.insert(pid.as_u32(), info);
    }
    map
}

/// Single-process lookup (kept for callers that already have a pid).
pub fn info_for(pid: u32) -> Option<ProcInfo> {
    let mut sys = System::new();
    let refresh = ProcessRefreshKind::nothing()
        .with_cmd(UpdateKind::Always)
        .with_exe(UpdateKind::Always)
        .with_cwd(UpdateKind::Always);
    sys.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[Pid::from_u32(pid)]),
        true,
        refresh,
    );
    let proc_ = sys.process(Pid::from_u32(pid))?;
    Some(ProcInfo {
        pid,
        parent: proc_.parent().map(|p| p.as_u32()),
        name: proc_.name().to_string_lossy().into_owned(),
        exe: proc_.exe().map(|p| p.to_string_lossy().into_owned()),
        cmd: proc_
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect(),
        cwd: proc_.cwd().map(|p| p.to_string_lossy().into_owned()),
    })
}
