use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TempTree(PathBuf);

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_ject(directory: &Path, command: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ject"))
        .current_dir(directory)
        .arg(command)
        .output()
        .expect("ject should start")
}

fn assert_success(output: &Output, command: &str) {
    assert!(
        output.status.success(),
        "{command} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_does_not_build_native_but_test_and_run_do() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "ject-native-workflow-{}-{unique}",
        std::process::id()
    ));
    let _cleanup = TempTree(root.clone());
    let library = root.join("probe_native");
    let app = root.join("app");
    let sdk_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("crates/ject-native")
        .to_string_lossy()
        .replace('\\', "/");
    fs::create_dir_all(library.join("native/src")).unwrap();
    fs::create_dir_all(library.join("src")).unwrap();
    fs::create_dir_all(library.join("tests")).unwrap();
    fs::create_dir_all(app.join("src")).unwrap();

    fs::write(
        library.join("Ject.toml"),
        "[package]\nname = \"probe_native\"\nversion = \"0.1.0\"\nedition = \"2026\"\nentry = \"src/lib.ject\"\n\n[native]\nlanguage = \"rust\"\npath = \"native\"\nabi = \"ject-native-2\"\nlibrary = \"probe_native\"\n\n[dependencies]\n",
    )
    .unwrap();
    fs::write(
        library.join("native/Cargo.toml"),
        format!(
            "[package]\nname = \"probe_native\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nject-native = {{ path = \"{sdk_path}\" }}\nserde_json = \"1.0\"\n\n[workspace]\n"
        ),
    )
    .unwrap();
    fs::write(
        library.join("native/src/lib.rs"),
        "fn call(_: &str, args: Vec<serde_json::Value>, _: *const ject_native::HostV1) -> Result<serde_json::Value, String> {\n    Ok(args.first().cloned().unwrap_or(serde_json::Value::Null))\n}\nject_native::ject_plugin_v2!(\"probe_native\", [\"echo\"], call);\n",
    )
    .unwrap();
    fs::write(
        library.join("src/lib.ject"),
        "import \"@native/probe_native\" as native\nexport fn echo(value) return native.echo(value) end\n",
    )
    .unwrap();
    fs::write(
        library.join("tests/probe.ject"),
        "import \"../src/lib\" as probe\nassert(probe.echo(42) == 42, \"native test failed\")\n",
    )
    .unwrap();
    fs::write(
        app.join("Ject.toml"),
        "[package]\nname = \"probe_app\"\nversion = \"0.1.0\"\nedition = \"2026\"\nentry = \"src/main.ject\"\n\n[security]\ntrusted-native = []\n\n[dependencies]\nprobe_native = { path = \"../probe_native\" }\n",
    )
    .unwrap();
    fs::write(
        app.join("src/main.ject"),
        "import \"probe_native\" as probe\nassert(probe.echo(43) == 43, \"native dependency failed\")\n",
    )
    .unwrap();

    let artifact = library.join("native/target/debug").join(format!(
        "{}probe_native{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    assert!(!artifact.exists());
    assert_success(&run_ject(&library, "check"), "ject check");
    assert_success(&run_ject(&app, "check"), "dependency check");
    assert!(!artifact.exists(), "ject check must not build native code");

    assert_success(&run_ject(&library, "test"), "ject test");
    assert!(
        artifact.exists(),
        "ject test should build the native package"
    );

    // The app imports the same package after its Rust implementation changes.
    // A previously built .so must not silently win over the updated source.
    fs::write(
        library.join("native/src/lib.rs"),
        "fn call(_: &str, _: Vec<serde_json::Value>, _: *const ject_native::HostV1) -> Result<serde_json::Value, String> {\n    Ok(serde_json::Value::from(44))\n}\nject_native::ject_plugin_v2!(\"probe_native\", [\"echo\"], call);\n",
    )
    .unwrap();
    fs::write(
        app.join("src/main.ject"),
        "import \"probe_native\" as probe\nassert(probe.echo(43) == 44, \"stale native dependency was loaded\")\n",
    )
    .unwrap();
    let untrusted = run_ject(&app, "run");
    assert!(!untrusted.status.success());
    assert!(String::from_utf8_lossy(&untrusted.stderr)
        .contains("native dependency 'probe_native' not trusted"));
    let app_manifest = app.join("Ject.toml");
    let manifest = fs::read_to_string(&app_manifest).unwrap();
    fs::write(
        &app_manifest,
        manifest.replace("trusted-native = []", "trusted-native = [\"probe_native\"]"),
    )
    .unwrap();
    assert_success(&run_ject(&app, "run"), "ject run");

    let manifest_path = library.join("Ject.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    fs::write(
        &manifest_path,
        manifest.replace("library = \"probe_native\"", "library = \"missing_probe\""),
    )
    .unwrap();
    let wrong_library = run_ject(&app, "run");
    assert!(!wrong_library.status.success());
    assert!(String::from_utf8_lossy(&wrong_library.stderr)
        .contains("native build succeeded but artifact was not found"));
    fs::write(&manifest_path, &manifest).unwrap();

    fs::write(
        &manifest_path,
        manifest.replace("abi = \"ject-native-2\"", "abi = \"unknown\""),
    )
    .unwrap();
    let wrong_abi = run_ject(&app, "run");
    assert!(!wrong_abi.status.success());
    assert!(String::from_utf8_lossy(&wrong_abi.stderr).contains("unsupported native ABI"));
    fs::write(&manifest_path, manifest).unwrap();

    assert_success(&run_ject(&app, "install"), "ject install");
    assert!(
        app.join("Ject.lock").is_file(),
        "install should create a lockfile"
    );
    let locked = Command::new(env!("CARGO_BIN_EXE_ject"))
        .current_dir(&app)
        .args(["install", "--locked"])
        .output()
        .unwrap();
    assert_success(&locked, "ject install --locked");
}
