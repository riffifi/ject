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
