//! Docker container mapping: host port -> container name, via `docker ps`.
//! Empty when Docker is absent or the daemon is down (command just fails).

use std::collections::HashMap;
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn container_map() -> HashMap<u16, String> {
    let mut map = HashMap::new();

    let mut cmd = Command::new("docker");
    cmd.args(["ps", "--format", "{{.Names}}\t{{.Ports}}"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let Ok(out) = cmd.output() else {
        #[cfg(debug_assertions)]
        eprintln!("docker: command failed to spawn");
        return map;
    };
    if !out.status.success() {
        #[cfg(debug_assertions)]
        eprintln!("docker: daemon unavailable or no containers");
        return map;
    }

    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        let mut it = line.splitn(2, '\t');
        let name = it.next().unwrap_or("").trim();
        let ports = it.next().unwrap_or("");
        if name.is_empty() {
            continue;
        }
        for hp in host_ports(ports) {
            map.entry(hp).or_insert_with(|| name.to_string());
        }
    }
    map
}

/// Parse host ports from a `docker ps` Ports field, e.g.
/// "0.0.0.0:5432->5432/tcp, :::5432->5432/tcp".
fn host_ports(s: &str) -> Vec<u16> {
    let mut v = Vec::new();
    for part in s.split(',') {
        if let Some(idx) = part.find("->") {
            let left = &part[..idx];
            if let Some(colon) = left.rfind(':') {
                if let Ok(p) = left[colon + 1..].trim().parse::<u16>() {
                    v.push(p);
                }
            }
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::host_ports;

    #[test]
    fn parses_dual_stack_mapping() {
        let ports = "0.0.0.0:5432->5432/tcp, :::5432->5432/tcp";
        assert_eq!(host_ports(ports), vec![5432, 5432]);
    }

    #[test]
    fn ignores_unmapped() {
        assert!(host_ports("5432/tcp").is_empty());
    }
}
