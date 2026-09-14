use std::fs;
use std::process::Command;

#[test]
fn test_command_runs_all_files_and_summarizes_failures() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ject-test-cli-{}-{unique}", std::process::id()));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(
        root.join("Ject.toml"),
        "[package]\nname = \"test_cli\"\nversion = \"0.1.0\"\nedition = \"2026\"\nentry = \"src/main.ject\"\n\n[dependencies]\n",
    )
    .unwrap();
    fs::write(root.join("src/main.ject"), "print \"app\"\n").unwrap();
    fs::write(
        root.join("tests/01_fail.ject"),
        "assert(false, \"expected failure\")\n",
    )
    .unwrap();
    fs::write(root.join("tests/02_pass.ject"), "assert(true)\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ject"))
        .current_dir(&root)
        .arg("test")
        .output()
        .unwrap();
    fs::remove_dir_all(root).unwrap();

    assert!(!output.status.success(), "one test should fail");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tests/01_fail.ject"), "{stdout}");
    assert!(stdout.contains("tests/02_pass.ject"), "{stdout}");
    assert!(stdout.contains("1 passed; 1 failed"), "{stdout}");
}
