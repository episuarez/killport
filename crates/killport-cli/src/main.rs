//! Headless CLI. F0 validates the core spike: enumerate dev ports + cmdline.

use killport_core::{kill_tree, restart, scan, KillMode};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("list");

    match cmd {
        "list" => list(&args[2..]),
        "kill" => kill_cmd(&args[2..]),
        "restart" => restart_cmd(&args[2..]),
        "--help" | "-h" | "help" => {
            println!(
                "killport {}\n\n\
                 USAGE:\n  \
                 killport list [--all]\n  \
                 killport kill <port> [--force]\n  \
                 killport kill --pid <pid> [--force]\n  \
                 killport restart <port>\n\n\
                 OPTIONS:\n  \
                 --all    Include system/unknown processes in list\n  \
                 --force  Skip graceful close; TerminateProcess immediately\n  \
                 --pid    Treat the numeric argument as a PID, not a port",
                env!("CARGO_PKG_VERSION")
            );
        }
        other => {
            eprintln!("unknown command: {other}\nRun 'killport --help' for usage.");
            std::process::exit(2);
        }
    }
}

fn restart_cmd(args: &[String]) {
    let Some(port) = args.first().and_then(|a| a.parse::<u16>().ok()) else {
        eprintln!("restart: missing numeric <port>");
        std::process::exit(2);
    };
    let targets: Vec<_> = scan().into_iter().filter(|p| p.port == port).collect();
    if targets.is_empty() {
        eprintln!("restart: nothing listening on port {port}");
        std::process::exit(1);
    }
    for p in targets {
        let cmd = p.cmd.clone();
        let cwd = p.cwd.clone();
        let pid = p.pid;
        if let Err(e) = kill_tree(pid, KillMode::Graceful) {
            eprintln!("restart: kill failed: {e}");
            std::process::exit(1);
        }
        if restart(&cmd, cwd.as_deref()) {
            println!("restarted port {port} (pid {pid} -> respawned)");
        } else {
            eprintln!("restart: failed to respawn (no captured command line?)");
            std::process::exit(1);
        }
    }
}

fn kill_cmd(args: &[String]) {
    let force = args.iter().any(|a| a == "--force");
    let by_pid = args.iter().any(|a| a == "--pid");
    let target: Option<u32> = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .and_then(|a| a.parse().ok());

    let Some(value) = target else {
        if by_pid {
            eprintln!("kill: --pid requires a numeric argument");
        } else {
            eprintln!("kill: missing numeric <port>");
        }
        std::process::exit(2);
    };
    let mode = if force {
        KillMode::Force
    } else {
        KillMode::Graceful
    };

    let pids: Vec<u32> = if by_pid {
        vec![value]
    } else {
        scan()
            .into_iter()
            .filter(|p| p.port as u32 == value)
            .map(|p| p.pid)
            .collect()
    };

    if pids.is_empty() {
        eprintln!("kill: nothing listening on port {value}");
        std::process::exit(1);
    }

    let mut total = 0;
    for pid in pids {
        let n = match kill_tree(pid, mode) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("kill: {e}");
                std::process::exit(1);
            }
        };
        println!("killed {n} process(es) in tree of pid {pid}");
        total += n;
    }
    if total == 0 {
        eprintln!("kill: no processes terminated (insufficient privileges?)");
        std::process::exit(1);
    }
}

fn list(args: &[String]) {
    let show_all = args.iter().any(|a| a == "--all" || a == "-a");
    let ports: Vec<_> = scan()
        .into_iter()
        .filter(|p| show_all || (!p.is_system && p.kind != "unknown"))
        .collect();

    if ports.is_empty() {
        println!("no dev ports listening (use --all to include system/other processes)");
        return;
    }
    println!(
        "{:<6} {:<7} {:<9} {:<22} {:<22} {:<6} ORIGIN",
        "PORT", "PID", "KIND", "APP", "PROJECT", "SYS"
    );
    for p in ports {
        let app = p.app.as_deref().unwrap_or(&p.name);
        let origin = match (&p.service, &p.parent_name) {
            (Some(svc), _) => format!("service: {svc}"),
            (None, Some(parent)) => format!("by {parent}"),
            (None, None) => "ad-hoc".to_string(),
        };
        println!(
            "{:<6} {:<7} {:<9} {:<22} {:<22} {:<6} {}",
            p.port,
            p.pid,
            p.kind,
            truncate(app, 22),
            truncate(p.project.as_deref().unwrap_or("-"), 22),
            if p.is_system { "yes" } else { "no" },
            origin,
        );
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{t}…")
    }
}
