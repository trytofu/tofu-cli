mod support;

use support::{temp_home, tofu_command};

#[test]
fn version_prints_package_version() {
    let home = temp_home("version");

    let output = tofu_command(&home)
        .arg("version")
        .output()
        .expect("run tofu-cli version");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        env!("CARGO_PKG_VERSION")
    );

    std::fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn version_prints_json() {
    let home = temp_home("version-json");

    let output = tofu_command(&home)
        .args(["--json", "version"])
        .output()
        .expect("run tofu-cli version json");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("version JSON output");
    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));

    std::fs::remove_dir_all(home).expect("remove temp home");
}
