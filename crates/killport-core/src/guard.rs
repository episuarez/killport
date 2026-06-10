//! Hard guard-list: processes Killport must never terminate, even on explicit click.

const PROTECTED: &[&str] = &[
    "system",
    "registry",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "svchost.exe",
    "lsm.exe",
    "spoolsv.exe",
    "dwm.exe",
    "fontdrvhost.exe",
    "memory compression",
];

pub fn is_protected(name: &str) -> bool {
    let n = name.to_lowercase();
    PROTECTED
        .iter()
        .any(|p| n == *p || n == p.trim_end_matches(".exe"))
}

#[cfg(test)]
mod tests {
    use super::is_protected;

    #[test]
    fn protects_system_processes() {
        assert!(is_protected("svchost.exe"));
        assert!(is_protected("System"));
        assert!(is_protected("lsass.exe"));
    }

    #[test]
    fn allows_dev_processes() {
        assert!(!is_protected("node.exe"));
        assert!(!is_protected("python.exe"));
    }
}
