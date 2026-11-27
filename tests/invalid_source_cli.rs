use std::{fs, process::Command};

#[test]
fn malformed_source_is_rejected_by_check_and_run_without_a_panic() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "ject-invalid-source-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    let script = root.join("invalid.ject");
    for (source, code) in [
        ("\n  #* unfinished", "E1003"),
        ("\nlet x = 1e+", "E1004"),
        ("\nlet x = 9223372036854775808", "E1004"),
    ] {
        fs::write(&script, source).unwrap();
        for check in [true, false] {
            let mut command = Command::new(env!("CARGO_BIN_EXE_ject"));
            if check {
                command.arg("--check");
            }
            let output = command.arg(&script).output().unwrap();
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_eq!(output.status.code(), Some(1), "{stderr}");
            assert!(stderr.contains(code), "{stderr}");
            assert!(stderr.contains("invalid.ject:2:"), "{stderr}");
            assert!(!stderr.contains("panicked"), "{stderr}");
            assert!(output.stdout.is_empty());
        }
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn imported_parse_errors_keep_dependency_locations_and_import_chains() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("ject-import-parse-{}-{unique}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("main.ject"), "import \"./broken\" as broken\n").unwrap();
    fs::write(root.join("broken.ject"), "\nlet bad = 1e+\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ject"))
        .arg("--check")
        .arg(root.join("main.ject"))
        .output()
        .unwrap();
    fs::remove_dir_all(root).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("E1004"), "{stderr}");
    assert!(stderr.contains("broken.ject:2:11"), "{stderr}");
    assert!(stderr.contains("import chain:"), "{stderr}");
    assert!(stderr.contains("main.ject"), "{stderr}");
}

#[test]
fn arithmetic_overflow_exits_with_a_language_diagnostic() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let script = std::env::temp_dir().join(format!(
        "ject-overflow-{}-{unique}.ject",
        std::process::id()
    ));
    fs::write(&script, "print 9223372036854775807 + 1\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ject"))
        .arg(&script)
        .output()
        .unwrap();
    fs::remove_file(script).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("E3008"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}
