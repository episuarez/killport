//! Launch-at-login via the HKCU Run key.

#[cfg(windows)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(windows)]
const VALUE: &str = "Killport";

#[cfg(windows)]
pub fn is_enabled() -> bool {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(RUN_KEY)
        .and_then(|k| k.get_value::<String, _>(VALUE))
        .is_ok()
}

#[cfg(windows)]
pub fn set(enabled: bool, exe_path: &str) -> std::io::Result<()> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu.create_subkey(RUN_KEY)?;
    if enabled {
        // Windows paths cannot legally contain " (NTFS-illegal), but sanitize
        // defensively before embedding in the registry quoted-path value.
        let safe_path = exe_path.replace('"', "");
        key.set_value(VALUE, &format!("\"{safe_path}\""))
    } else {
        match key.delete_value(VALUE) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(not(windows))]
pub fn is_enabled() -> bool {
    false
}

#[cfg(not(windows))]
pub fn set(_enabled: bool, _exe_path: &str) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn autostart_round_trip() {
        let fake_exe = r"C:\fake\killport.exe";
        set(true, fake_exe).expect("set(true) failed");
        assert!(is_enabled(), "expected autostart to be enabled");
        set(false, fake_exe).expect("set(false) failed");
        assert!(!is_enabled(), "expected autostart to be disabled");
    }

    #[cfg(windows)]
    #[test]
    fn autostart_disable_when_not_set_is_noop() {
        // Disabling when never enabled must not error.
        let result = set(false, r"C:\fake\exe.exe");
        assert!(result.is_ok());
    }

    #[cfg(not(windows))]
    #[test]
    fn autostart_stubs_are_noop() {
        assert!(!is_enabled());
        assert!(set(true, "").is_ok());
        assert!(set(false, "").is_ok());
    }
}
