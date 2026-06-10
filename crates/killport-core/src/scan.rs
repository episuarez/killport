//! Join listening ports with process metadata into the unified model.

use std::collections::HashMap;

use serde::Serialize;

use crate::appinfo;
use crate::classify::classify;
use crate::docker;
use crate::framework;
use crate::guard;
use crate::ports::listening_ports;
use crate::process::{snapshot, ProcInfo};
use crate::project;
use crate::service;

#[derive(Debug, Clone, Serialize)]
pub struct PortProcess {
    pub port: u16,
    pub pid: u32,
    /// Executable name, e.g. "node.exe".
    pub name: String,
    /// Human application name from the exe version resource, e.g. "PostgreSQL Server".
    pub app: Option<String>,
    /// Runtime class: node / python / php / postgresql / docker / unknown ...
    pub kind: String,
    pub framework: Option<String>,
    pub project: Option<String>,
    pub project_path: Option<String>,
    /// Registered Windows service name, if this process is a service.
    pub service: Option<String>,
    /// Docker container name, if the port maps to one.
    pub container: Option<String>,
    /// Name of the process that launched this one (e.g. "pwsh.exe", "Code.exe").
    pub parent_name: Option<String>,
    pub parent_pid: Option<u32>,
    /// OS/system process (exe under %WINDIR% or guarded). Hidden by default.
    pub is_system: bool,
    pub url: String,
    pub cmd: Vec<String>,
    pub cwd: Option<String>,
    pub exe: Option<String>,
}

/// Full scan: Docker mapping + app name + service mapping. Heavier.
pub fn scan() -> Vec<PortProcess> {
    scan_inner(true, true)
}

/// Cheap scan for high-frequency polling / change-detection: no Docker, no
/// version-info, no SCM. Still sets `is_system` and `parent_name` (cheap).
pub fn scan_fast() -> Vec<PortProcess> {
    scan_inner(false, false)
}

fn scan_inner(with_docker: bool, with_enrich: bool) -> Vec<PortProcess> {
    let procs = snapshot();
    let containers = if with_docker {
        docker::container_map()
    } else {
        HashMap::new()
    };
    let services = if with_enrich {
        service::service_map()
    } else {
        HashMap::new()
    };

    let mut out: Vec<PortProcess> = listening_ports()
        .into_iter()
        .map(|lp| {
            let info = procs.get(&lp.pid);
            let name = info.map(|i| i.name.clone()).unwrap_or_default();
            let cmd = info.map(|i| i.cmd.clone()).unwrap_or_default();
            let cwd = info.and_then(|i| i.cwd.clone());
            let exe = info.and_then(|i| i.exe.clone());
            let kind = classify(&name, &cmd).to_string();
            let framework = framework::detect(&cmd, lp.port);
            let proj = project::detect(cwd.as_deref(), exe.as_deref());
            let app = if with_enrich {
                exe.as_deref().and_then(appinfo::app_name)
            } else {
                None
            };

            PortProcess {
                port: lp.port,
                pid: lp.pid,
                app,
                kind,
                framework,
                project: proj.as_ref().map(|p| p.name.clone()),
                project_path: proj.as_ref().map(|p| p.path.clone()),
                service: services.get(&lp.pid).cloned(),
                container: containers.get(&lp.port).cloned(),
                parent_name: parent_name(info, &procs),
                parent_pid: info.and_then(|i| i.parent),
                is_system: is_system(&name, exe.as_deref()),
                url: format!("http://localhost:{}", lp.port),
                name,
                cmd,
                cwd,
                exe,
            }
        })
        .collect();
    out.sort_by_key(|p| p.port);
    out
}

fn parent_name(info: Option<&ProcInfo>, procs: &HashMap<u32, ProcInfo>) -> Option<String> {
    let parent = info?.parent?;
    procs.get(&parent).map(|p| p.name.clone())
}

static WINDIR_LOWER: std::sync::OnceLock<String> = std::sync::OnceLock::new();

fn windir_lower() -> &'static str {
    WINDIR_LOWER.get_or_init(|| {
        std::env::var("WINDIR")
            .unwrap_or_else(|_| "C:\\Windows".to_string())
            .to_lowercase()
            .replace('/', "\\")
    })
}

fn is_system(name: &str, exe: Option<&str>) -> bool {
    if guard::is_protected(name) {
        return true;
    }
    match exe {
        None => true,
        Some(e) => {
            let el = e.to_lowercase().replace('/', "\\");
            el.starts_with(windir_lower())
        }
    }
}
