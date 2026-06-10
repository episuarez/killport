//! Application name from an executable's version resource (FileDescription, then
//! ProductName). This is the human "what app is this" label. Falls back to None.

#[cfg(windows)]
pub fn app_name(exe_path: &str) -> Option<String> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW};

    let wide: Vec<u16> = exe_path.encode_utf16().chain(std::iter::once(0)).collect();
    let path = PCWSTR(wide.as_ptr());

    unsafe {
        let mut dummy = 0u32;
        let size = GetFileVersionInfoSizeW(path, Some(&mut dummy));
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        GetFileVersionInfoW(path, Some(0), size, buf.as_mut_ptr() as *mut _).ok()?;

        // Resolve the translation (lang + codepage) of the version resource.
        let (lang, cp) = translation(&buf)?;

        for field in ["FileDescription", "ProductName"] {
            if let Some(name) = query_string(&buf, lang, cp, field) {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }
}

#[cfg(windows)]
unsafe fn translation(buf: &[u8]) -> Option<(u16, u16)> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::VerQueryValueW;

    let key: Vec<u16> = "\\VarFileInfo\\Translation"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut ptr = std::ptr::null_mut();
    let mut len = 0u32;
    let ok = VerQueryValueW(
        buf.as_ptr() as *const _,
        PCWSTR(key.as_ptr()),
        &mut ptr,
        &mut len,
    );
    if !ok.as_bool() || len < 4 || ptr.is_null() {
        return None;
    }
    let lang = *(ptr as *const u16);
    let cp = *((ptr as *const u16).add(1));
    Some((lang, cp))
}

#[cfg(windows)]
unsafe fn query_string(buf: &[u8], lang: u16, cp: u16, field: &str) -> Option<String> {
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::VerQueryValueW;

    let sub = format!("\\StringFileInfo\\{lang:04x}{cp:04x}\\{field}");
    let subw: Vec<u16> = sub.encode_utf16().chain(std::iter::once(0)).collect();
    let mut ptr = std::ptr::null_mut();
    let mut len = 0u32;
    let ok = VerQueryValueW(
        buf.as_ptr() as *const _,
        PCWSTR(subw.as_ptr()),
        &mut ptr,
        &mut len,
    );
    if !ok.as_bool() || len == 0 || ptr.is_null() {
        return None;
    }
    let slice = std::slice::from_raw_parts(ptr as *const u16, len as usize);
    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
    Some(String::from_utf16_lossy(&slice[..end]))
}

#[cfg(not(windows))]
pub fn app_name(_exe_path: &str) -> Option<String> {
    None
}
