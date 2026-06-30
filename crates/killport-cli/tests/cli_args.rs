//! Regression test: invoking the binary with no subcommand must not panic.
//! It previously sliced `args[2..]` on a 1-element argv, which panics.

use std::process::Command;

#[test]
fn no_args_does_not_panic() {
    let exe = env!("CARGO_BIN_EXE_killport");
    let output = Command::new(exe).output().expect("spawn killport");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panicked"),
        "killport with no args panicked: {stderr}"
    );
}
