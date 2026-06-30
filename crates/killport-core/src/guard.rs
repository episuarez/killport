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
    // Normalize to the basename so a full path (or any other variant a caller
    // might pass) can't slip past the guard the way a bare process name would not.
    let base = std::path::Path::new(name)
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_string());
    let n = base.to_lowercase();
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

    #[test]
    fn protects_full_paths_by_basename() {
        assert!(is_protected(r"C:\Windows\System32\lsass.exe"));
        assert!(is_protected(r"C:\Windows\System32\SVCHOST.EXE"));
    }
}
