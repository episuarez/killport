//! Verifies port enumeration against a real listener owned by this test process.

use std::net::TcpListener;

#[test]
#[cfg(windows)]
fn finds_own_listening_port() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let pid = std::process::id();

    let found = killport_core::ports::listening_ports();
    let hit = found.iter().any(|p| p.port == port && p.pid == pid);

    assert!(hit, "expected port {port} owned by pid {pid} in {found:?}");
}
