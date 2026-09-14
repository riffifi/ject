#[cfg(unix)]
#[test]
fn changing_directory_affects_the_running_program_and_children() {
    use std::process::Command;

    let directory = std::env::temp_dir().join(format!(
        "ject-system-cli-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir(&directory).unwrap();
    let script = directory.join("main.ject");
    let source = format!(
        "import \"system\" as system\n\
         system.change_dir(\"{}\")\n\
         assert(system.cwd() == system.get_cwd(), \"cwd wrappers differ\")\n\
         let child = system.run_process(\"pwd\")\n\
         assert(child.stdout.trim() == system.cwd(), \"child did not inherit cwd\")\n\
         let other = system.run_process(\"pwd\", [], \"/\")\n\
         assert(other.stdout.trim() == \"/\", \"child directory was ignored\")\n\
         assert(system.cwd() == \"{}\", \"child changed parent cwd\")\n",
        directory.display(),
        directory.display()
    );
    std::fs::write(&script, source).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_ject"))
        .arg(&script)
        .output()
        .unwrap();
    std::fs::remove_dir_all(&directory).unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn script_arguments_exclude_ject_commands_and_script_paths() {
    use std::process::Command;

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("ject-script-args-{}-{unique}", std::process::id()));
    std::fs::create_dir(&directory).unwrap();
    std::fs::create_dir(directory.join("src")).unwrap();
    std::fs::write(
        directory.join("Ject.toml"),
        "[package]\nname = \"args_test\"\nversion = \"0.1.0\"\nedition = \"2026\"\nentry = \"src/main.ject\"\n\n[dependencies]\n",
    )
    .unwrap();
    let script = directory.join("src/main.ject");
    std::fs::write(
        &script,
        "import \"system\" as system\nlet values = system.args()\nassert(len(values) == 2, \"wrong argument count\")\nassert(values[0] == \"first\", \"wrong first argument\")\nassert(values[1] == \"--second\", \"wrong second argument\")\n",
    )
    .unwrap();
    let binary = env!("CARGO_BIN_EXE_ject");
    let direct = Command::new(binary)
        .current_dir(&directory)
        .arg(&script)
        .args(["first", "--second"])
        .output()
        .unwrap();
    let package = Command::new(binary)
        .current_dir(&directory)
        .args(["run", "--", "first", "--second"])
        .output()
        .unwrap();
    std::fs::remove_dir_all(&directory).unwrap();
    for (name, output) in [("script", direct), ("package", package)] {
        assert!(
            output.status.success(),
            "{name} failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
