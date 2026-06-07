use std::fs;

mod support;

use support::{temp_home, tofu_command, write_config};

#[test]
fn config_show_json_redacts_saved_token() {
    let home = temp_home("config-show");
    write_config(
        &home,
        "api_base_url = \"http://127.0.0.1:4321\"\ntoken = \"tofu_pat_secret\"\n",
    );

    let output = tofu_command(&home)
        .args(["--json", "config", "show"])
        .output()
        .expect("run tofu-cli config show");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(r#""api_base_url":"http://127.0.0.1:4321""#));
    assert!(stdout.contains(r#""token":"<redacted>""#));
    assert!(!stdout.contains("tofu_pat_secret"));

    fs::remove_dir_all(home).expect("remove temp home");
}

#[test]
fn config_show_table_redacts_saved_token() {
    let home = temp_home("config-show-table");
    write_config(
        &home,
        "api_base_url = \"http://127.0.0.1:4321\"\ntoken = \"tofu_pat_secret\"\n",
    );

    let output = tofu_command(&home)
        .args(["config", "show"])
        .output()
        .expect("run tofu-cli config show");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("http://127.0.0.1:4321"));
    assert!(stdout.contains("<redacted>"));
    assert!(!stdout.contains("tofu_pat_secret"));
    assert!(!stdout.contains("(not set)"));

    fs::remove_dir_all(home).expect("remove temp home");
}
