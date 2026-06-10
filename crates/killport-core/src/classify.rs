//! Runtime classification. Databases/services detected by exe name; framework
//! guesses (vite/next) only when the runtime is Node, so unrelated processes
//! (e.g. NextDNS.exe) don't match on a name substring.

pub fn classify(name: &str, cmd: &[String]) -> &'static str {
    let n = name.to_lowercase();
    let tokens: Vec<String> = cmd.iter().map(|s| s.to_lowercase()).collect();
    let has = |needle: &str| tokens.iter().any(|t| t.contains(needle));

    // Databases / common services (by exe name, most reliable).
    if n.contains("postgres") {
        return "postgresql";
    }
    if n.contains("mysqld") || n == "mysql.exe" || n.contains("mariadb") {
        return "mysql";
    }
    if n.contains("redis") {
        return "redis";
    }
    if n.contains("mongod") {
        return "mongodb";
    }
    if n.contains("sqlservr") {
        return "sqlserver";
    }
    if n.contains("docker") || n.contains("com.docker") || n.contains("vpnkit") {
        return "docker";
    }
    if n.contains("wslrelay") || n.contains("wslhost") || n == "wsl.exe" {
        return "wsl";
    }

    let is_node = n.contains("node") || has("npm") || has("pnpm") || has("yarn") || has("bun");

    if is_node {
        if has("vite") {
            "vite"
        } else if has("next") {
            "next.js"
        } else {
            "node"
        }
    } else if n.contains("python")
        || has("flask")
        || has("django")
        || has("uvicorn")
        || has("gunicorn")
    {
        "python"
    } else if n.contains("php") || has("artisan") {
        "php"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::classify;

    fn v(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn detects_node_from_cmd() {
        assert_eq!(classify("node.exe", &v(&["node", "server.js"])), "node");
    }

    #[test]
    fn detects_vite_over_node() {
        assert_eq!(
            classify("node.exe", &v(&["node", "node_modules/.bin/vite"])),
            "vite"
        );
    }

    #[test]
    fn detects_python() {
        assert_eq!(
            classify("python.exe", &v(&["python", "-m", "uvicorn"])),
            "python"
        );
    }

    #[test]
    fn detects_postgres() {
        assert_eq!(classify("postgres.exe", &[]), "postgresql");
    }

    #[test]
    fn detects_redis() {
        assert_eq!(classify("redis-server.exe", &[]), "redis");
    }

    #[test]
    fn unknown_when_no_signal() {
        assert_eq!(classify("svchost.exe", &[]), "unknown");
    }

    #[test]
    fn nextdns_is_not_nextjs() {
        assert_eq!(
            classify(
                "NextDNS.exe",
                &v(&["C:\\Program Files\\NextDNS\\NextDNS.exe"])
            ),
            "unknown"
        );
    }
}
