use std::process::Command;

#[test]
fn cli_help_runs() {
    let out = Command::new(env!("CARGO_BIN_EXE_diycad-migrate"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(out.status.success());
}
