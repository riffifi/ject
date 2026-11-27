use serde_json::json;
use std::{fs, process::Command};

#[test]
fn compiled_program_loads_mapped_modules_without_source() {
    let directory = std::env::temp_dir().join(format!("ject-artifact-{}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let artifact = directory.join("main.json");
    let program = json!({
        "format": "ject-program-ast",
        "version": 1,
        "entry": "main",
        "modules": {
            "main": {
                "imports": {"./lib": "lib"},
                "statements": [
                    {"kind": "Import", "module_path": "./lib", "items": ["answer"]},
                    {"kind": "Print", "values": [{"kind": "Identifier", "value": "answer"}]}
                ]
            },
            "lib": {
                "imports": {},
                "statements": [
                    {"kind": "Export", "name": "answer", "mutable": false,
                     "value": {"kind": "Integer", "value": 42}}
                ]
            }
        }
    });
    let run = |program: &serde_json::Value| {
        fs::write(&artifact, program.to_string()).unwrap();
        Command::new(env!("CARGO_BIN_EXE_ject"))
            .arg("run-artifact")
            .arg(&artifact)
            .output()
            .unwrap()
    };
    let result = run(&program);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(result.stdout, b"42\n");

    let mut unmapped = program.clone();
    unmapped["modules"]["main"]["imports"] = json!({});
    fs::write(directory.join("lib.ject"), "export const answer = 999\n").unwrap();
    let result = run(&unmapped);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("no import mapping"));

    let mut cycle = program;
    cycle["modules"]["lib"]["imports"] = json!({"./main": "main"});
    cycle["modules"]["lib"]["statements"] = json!([
        {"kind": "Import", "module_path": "./main"}
    ]);
    let result = run(&cycle);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("Circular compiled import"));
    fs::remove_dir_all(directory).unwrap();
}
