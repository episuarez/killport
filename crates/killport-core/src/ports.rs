//! Listening TCP ports (IPv4 + IPv6) via Windows IP Helper API (no netstat parsing).

#[derive(Debug, Clone)]
pub struct ListeningPort {
    pub port: u16,
    pub pid: u32,
}

#[cfg(windows)]
const MIB_TCP_STATE_LISTEN: u32 = 2;

/// Convert dwLocalPort (network byte order in low 16 bits) to host u16.
#[cfg(windows)]
#[inline]
fn ntohs_low(raw: u32) -> u16 {
    (((raw & 0xFF) << 8) | ((raw >> 8) & 0xFF)) as u16
}

#[cfg(windows)]
pub fn listening_ports() -> Vec<ListeningPort> {
    use std::collections::HashSet;

    let mut out = Vec::new();
    let mut seen = HashSet::new();
    unsafe {
        collect_v4(&mut out, &mut seen);
        collect_v6(&mut out, &mut seen);
    }
    out.sort_by_key(|p| p.port);
    out
}

#[cfg(windows)]
unsafe fn collect_v4(
    out: &mut Vec<ListeningPort>,
    seen: &mut std::collections::HashSet<(u16, u32)>,
) {
    use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
    use windows::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCPTABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
    };
    use windows::Win32::Networking::WinSock::AF_INET;

    // Retry up to 3 times: the table can grow between the size query and the fill call.
    for _ in 0..3u8 {
        let mut size: u32 = 0;
        let _ = GetExtendedTcpTable(
            None,
            &mut size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
        if size == 0 {
            return;
        }
        let mut buf = vec![0u8; size as usize];
        let ret = GetExtendedTcpTable(
            Some(buf.as_mut_ptr() as *mut _),
            &mut size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
        if ret == ERROR_INSUFFICIENT_BUFFER.0 {
            continue; // table grew between calls; retry with new size
        }
        if ret != NO_ERROR.0 {
            return;
        }
        // Verify reported size doesn't exceed allocated buffer before parsing.
        if size as usize > buf.len() {
            return;
        }
        let table = &*(buf.as_ptr() as *const MIB_TCPTABLE_OWNER_PID);
        let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
        for row in rows {
            if row.dwState == MIB_TCP_STATE_LISTEN {
                let port = ntohs_low(row.dwLocalPort);
                let pid = row.dwOwningPid;
                if seen.insert((port, pid)) {
                    out.push(ListeningPort { port, pid });
                }
            }
        }
        return;
    }
}

#[cfg(windows)]
unsafe fn collect_v6(
    out: &mut Vec<ListeningPort>,
    seen: &mut std::collections::HashSet<(u16, u32)>,
) {
    use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
    use windows::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCP6TABLE_OWNER_PID, TCP_TABLE_OWNER_PID_ALL,
    };
    use windows::Win32::Networking::WinSock::AF_INET6;

    for _ in 0..3u8 {
        let mut size: u32 = 0;
        let _ = GetExtendedTcpTable(
            None,
            &mut size,
            false,
            AF_INET6.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
        if size == 0 {
            return;
        }
        let mut buf = vec![0u8; size as usize];
        let ret = GetExtendedTcpTable(
            Some(buf.as_mut_ptr() as *mut _),
            &mut size,
            false,
            AF_INET6.0 as u32,
            TCP_TABLE_OWNER_PID_ALL,
            0,
        );
        if ret == ERROR_INSUFFICIENT_BUFFER.0 {
            continue;
        }
        if ret != NO_ERROR.0 {
            return;
        }
        if size as usize > buf.len() {
            return;
        }
        let table = &*(buf.as_ptr() as *const MIB_TCP6TABLE_OWNER_PID);
        let rows = std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize);
        for row in rows {
            if row.dwState == MIB_TCP_STATE_LISTEN {
                let port = ntohs_low(row.dwLocalPort);
                let pid = row.dwOwningPid;
                if seen.insert((port, pid)) {
                    out.push(ListeningPort { port, pid });
                }
            }
        }
        return;
    }
}

#[cfg(not(windows))]
pub fn listening_ports() -> Vec<ListeningPort> {
    Vec::new()
}
