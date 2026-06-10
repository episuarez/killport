//! Map pid -> Windows service name via the Service Control Manager.
//! Empty on any failure (e.g. insufficient rights). Lets us tell a registered
//! service (PostgreSQL, MySQL) from an ad-hoc launched process.

use std::collections::HashMap;

#[cfg(windows)]
pub fn service_map() -> HashMap<u32, String> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Services::{
        CloseServiceHandle, EnumServicesStatusExW, OpenSCManagerW, ENUM_SERVICE_STATUS_PROCESSW,
        SC_ENUM_PROCESS_INFO, SC_MANAGER_ENUMERATE_SERVICE, SERVICE_STATE_ALL, SERVICE_WIN32,
    };

    let mut map = HashMap::new();

    unsafe {
        let Ok(scm) = OpenSCManagerW(PCWSTR::null(), PCWSTR::null(), SC_MANAGER_ENUMERATE_SERVICE)
        else {
            return map;
        };

        let mut needed = 0u32;
        let mut returned = 0u32;
        let mut resume = 0u32;

        // First call sizes the buffer (expected to fail with ERROR_MORE_DATA).
        let _ = EnumServicesStatusExW(
            scm,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32,
            SERVICE_STATE_ALL,
            None,
            &mut needed,
            &mut returned,
            Some(&mut resume),
            PCWSTR::null(),
        );

        if needed == 0 {
            let _ = CloseServiceHandle(scm);
            return map;
        }

        // Zero-initialize: Vec::with_capacity leaves memory uninit; LPWSTR fields
        // in ENUM_SERVICE_STATUS_PROCESSW require valid memory for the byte-slice view.
        let entry_size = std::mem::size_of::<ENUM_SERVICE_STATUS_PROCESSW>();
        let count = (needed as usize / entry_size) + 1;
        let mut store: Vec<ENUM_SERVICE_STATUS_PROCESSW> =
            vec![ENUM_SERVICE_STATUS_PROCESSW::default(); count];
        let bytes =
            std::slice::from_raw_parts_mut(store.as_mut_ptr() as *mut u8, count * entry_size);

        resume = 0;
        let ok = EnumServicesStatusExW(
            scm,
            SC_ENUM_PROCESS_INFO,
            SERVICE_WIN32,
            SERVICE_STATE_ALL,
            Some(bytes),
            &mut needed,
            &mut returned,
            Some(&mut resume),
            PCWSTR::null(),
        );

        if ok.is_ok() {
            // Clamp returned to allocated count to prevent OOB if the API
            // returns a value greater than count (defensive against API contract violation).
            let safe_count = (returned as usize).min(store.len());
            let arr = std::slice::from_raw_parts(store.as_ptr(), safe_count);
            for e in arr {
                let pid = e.ServiceStatusProcess.dwProcessId;
                if pid != 0 {
                    if let Ok(name) = e.lpServiceName.to_string() {
                        map.entry(pid).or_insert(name);
                    }
                }
            }
        }

        let _ = CloseServiceHandle(scm);
    }

    map
}

#[cfg(not(windows))]
pub fn service_map() -> HashMap<u32, String> {
    HashMap::new()
}
