//! Project discovery, local dependency installation, locking, and native builds.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const MANIFEST_FILE: &str = "Ject.toml";
pub const LOCK_FILE: &str = "Ject.lock";

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub name: String,
    pub version: String,
    pub entry: PathBuf,
    pub native: Option<NativeConfig>,
    pub dependencies: HashMap<String, PathBuf>,
}

#[derive(Debug, Clone)]
pub struct NativeConfig {
    pub path: PathBuf,
    pub library: String,
}

pub fn discover(start: &Path) -> Result<Project, String> {
    let start = if start.is_file() {
        start.parent().unwrap_or(start)
    } else {
        start
    };
    let root = start
        .ancestors()
        .find(|dir| dir.join(MANIFEST_FILE).is_file())
        .ok_or_else(|| format!("no {MANIFEST_FILE} found in this directory or its parents"))?;
    load(root)
}

pub fn load(root: &Path) -> Result<Project, String> {
    let manifest_path = root.join(MANIFEST_FILE);
    let source = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("failed to read {}: {e}", manifest_path.display()))?;
    let mut section = "";
    let mut name = None;
    let mut entry = None;
    let mut version = None;
    let mut native_path = None;
    let mut native_library = None;
    let mut dependencies = HashMap::new();

    for raw_line in source.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = &line[1..line.len() - 1];
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let raw_value = value.trim();
        let value = raw_value.trim_matches('"');
        match (section, key.trim()) {
            ("package", "name") => name = Some(value.to_string()),
            ("package", "version") => version = Some(value.to_string()),
            ("package", "entry") | ("lib", "entry") => entry = Some(PathBuf::from(value)),
            ("native", "path") => native_path = Some(PathBuf::from(value)),
            ("native", "library") => native_library = Some(value.to_string()),
            ("dependencies", dependency) => {
                if let Some(path) = dependency_path(raw_value) {
                    dependencies.insert(dependency.to_string(), root.join(path));
                }
            }
            _ => {}
        }
    }

    let name = name.ok_or_else(|| "Ject.toml is missing [package].name".to_string())?;
    let entry = root.join(entry.unwrap_or_else(|| PathBuf::from("src/main.ject")));
    let native = native_path.map(|path| NativeConfig {
        path: root.join(path),
        library: native_library.unwrap_or_else(|| name.replace('-', "_")),
    });
    Ok(Project {
        root: root.to_path_buf(),
        name,
        version: version.unwrap_or_else(|| "0.0.0".to_string()),
        entry,
        native,
        dependencies,
    })
}

/// Add or replace a local path dependency in Ject.toml.
pub fn add_path_dependency(project: &Project, name: &str, path: &Path) -> Result<Project, String> {
    validate_package_name(name)?;
    let dependency_root = if path.is_absolute() {
        path.to_path_buf()
    } else {
        project.root.join(path)
    };
    let project_canonical = project.root.canonicalize().map_err(|e| {
        format!(
            "failed to resolve current package {}: {e}",
            project.root.display()
        )
    })?;
    let dependency_canonical = dependency_root.canonicalize().map_err(|e| {
        format!(
            "failed to resolve dependency path {}: {e}",
            dependency_root.display()
        )
    })?;
    if project_canonical == dependency_canonical {
        return Err(format!(
            "package '{}' cannot depend on itself; '{}' resolves to the current package",
            project.name,
            path.display()
        ));
    }
    let dependency = load(&dependency_canonical).map_err(|e| {
        format!(
            "invalid dependency '{name}' at {}: {e}",
            dependency_root.display()
        )
    })?;
    if dependency.name != name {
        return Err(format!(
            "dependency is named '{}' but was added as '{name}'",
            dependency.name
        ));
    }
    update_dependency_line(&project.root.join(MANIFEST_FILE), name, Some(path))?;
    load(&project.root)
}

pub fn remove_dependency(project: &Project, name: &str) -> Result<Project, String> {
    if !project.dependencies.contains_key(name) {
        return Err(format!("package '{name}' is not a dependency"));
    }
    update_dependency_line(&project.root.join(MANIFEST_FILE), name, None)?;
    load(&project.root)
}

fn validate_package_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(format!(
            "invalid package name '{name}'; use letters, digits, '-' or '_'"
        ));
    }
    Ok(())
}

fn update_dependency_line(manifest: &Path, name: &str, path: Option<&Path>) -> Result<(), String> {
    let source = fs::read_to_string(manifest)
        .map_err(|e| format!("failed to read {}: {e}", manifest.display()))?;
    let mut output = Vec::new();
    let mut in_dependencies = false;
    let mut found_section = false;
    let mut written = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if in_dependencies && !written {
                if let Some(path) = path {
                    output.push(dependency_line(name, path));
                }
                written = true;
            }
            in_dependencies = trimmed == "[dependencies]";
            found_section |= in_dependencies;
        }
        if in_dependencies && trimmed.split_once('=').map(|(key, _)| key.trim()) == Some(name) {
            if let Some(path) = path {
                output.push(dependency_line(name, path));
            }
            written = true;
            continue;
        }
        output.push(line.to_string());
    }
    if !found_section {
        output.push(String::new());
        output.push("[dependencies]".into());
    }
    if !written {
        if let Some(path) = path {
            output.push(dependency_line(name, path));
        }
    }
    let updated = format!("{}\n", output.join("\n"));
    let temporary = manifest.with_extension("toml.tmp");
    fs::write(&temporary, updated)
        .map_err(|e| format!("failed to write {}: {e}", temporary.display()))?;
    fs::rename(&temporary, manifest)
        .map_err(|e| format!("failed to replace {}: {e}", manifest.display()))
}

fn dependency_line(name: &str, path: &Path) -> String {
    let escaped = path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!("{name} = {{ path = \"{escaped}\" }}")
}

/// Resolve every local dependency and write a deterministic lockfile.
pub fn install(project: &Project) -> Result<Vec<Project>, String> {
    let dependencies = dependency_projects(project)?;
    let mut locked = dependencies.clone();
    locked.push(project.clone());
    locked.sort_by(|a, b| a.name.cmp(&b.name).then(a.root.cmp(&b.root)));
    let mut lock = String::from("# Generated by Ject. Do not edit by hand.\nversion = 1\n");
    for package in &locked {
        let canonical = package
            .root
            .canonicalize()
            .map_err(|e| format!("failed to lock {}: {e}", package.root.display()))?;
        lock.push_str("\n[[package]]\n");
        lock.push_str(&format!("name = \"{}\"\n", package.name));
        lock.push_str(&format!("version = \"{}\"\n", package.version));
        lock.push_str(&format!("source = \"path+{}\"\n", canonical.display()));
        lock.push_str(&format!("native = {}\n", package.native.is_some()));
    }
    let destination = project.root.join(LOCK_FILE);
    let temporary = project.root.join("Ject.lock.tmp");
    fs::write(&temporary, lock)
        .map_err(|e| format!("failed to write {}: {e}", temporary.display()))?;
    fs::rename(&temporary, &destination)
        .map_err(|e| format!("failed to replace {}: {e}", destination.display()))?;
    Ok(dependencies)
}

fn dependency_path(value: &str) -> Option<PathBuf> {
    if value.starts_with('"') {
        return Some(PathBuf::from(value.trim_matches('"')));
    }
    let path_at = value.find("path")?;
    let after = &value[path_at + 4..];
    let quote = after.find('"')?;
    let rest = &after[quote + 1..];
    let end = rest.find('"')?;
    Some(PathBuf::from(&rest[..end]))
}

pub fn dependency_projects(project: &Project) -> Result<Vec<Project>, String> {
    fn visit(
        project: &Project,
        seen: &mut HashSet<PathBuf>,
        output: &mut Vec<Project>,
    ) -> Result<(), String> {
        for root in project.dependencies.values() {
            let canonical = root.canonicalize().map_err(|e| {
                format!("failed to resolve dependency path {}: {e}", root.display())
            })?;
            if seen.insert(canonical.clone()) {
                let dependency = load(&canonical)?;
                visit(&dependency, seen, output)?;
                output.push(dependency);
            }
        }
        Ok(())
    }

    let mut output = Vec::new();
    let mut seen = HashSet::new();
    if let Ok(root) = project.root.canonicalize() {
        seen.insert(root);
    }
    visit(project, &mut seen, &mut output)?;
    Ok(output)
}

/// Return every Ject source owned by a package in deterministic order.
/// The configured entry is included even when it lives outside `src/`.
pub fn source_files(project: &Project) -> Result<Vec<PathBuf>, String> {
    fn visit(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
        if !directory.is_dir() {
            return Ok(());
        }
        let entries = fs::read_dir(directory)
            .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
        for entry in entries {
            let path = entry
                .map_err(|error| format!("failed to read {}: {error}", directory.display()))?
                .path();
            if path.is_dir() {
                visit(&path, files)?;
            } else if path.extension().and_then(|value| value.to_str()) == Some("ject") {
                files.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(&project.root.join("src"), &mut files)?;
    if project.entry.is_file() && !files.contains(&project.entry) {
        files.push(project.entry.clone());
    }
    files.sort();
    files.dedup();
    Ok(files)
}

pub fn build_native(project: &Project, release: bool) -> Result<Option<PathBuf>, String> {
    let Some(native) = &project.native else {
        return Ok(None);
    };
    let manifest = native.path.join("Cargo.toml");
    if !manifest.is_file() {
        return Err(format!(
            "native Cargo manifest not found: {}",
            manifest.display()
        ));
    }
    let mut command = Command::new("cargo");
    command.arg("build").arg("--manifest-path").arg(&manifest);
    if release {
        command.arg("--release");
    }
    let status = command
        .status()
        .map_err(|e| format!("failed to start Cargo for native package: {e}"))?;
    if !status.success() {
        return Err(format!("native build failed with status {status}"));
    }
    let profile = if release { "release" } else { "debug" };
    let artifact = native
        .path
        .join("target")
        .join(profile)
        .join(dynamic_library_name(&native.library));
    if !artifact.is_file() {
        return Err(format!(
            "native build succeeded but artifact was not found: {}",
            artifact.display()
        ));
    }
    Ok(Some(artifact))
}

pub fn build_native_graph(project: &Project, release: bool) -> Result<Vec<PathBuf>, String> {
    let mut artifacts = Vec::new();
    for dependency in dependency_projects(project)? {
        if let Some(artifact) = build_native(&dependency, release)? {
            artifacts.push(artifact);
        }
    }
    if let Some(artifact) = build_native(project, release)? {
        artifacts.push(artifact);
    }
    Ok(artifacts)
}

pub fn find_native_artifact(project: &Project) -> Result<Option<PathBuf>, String> {
    let Some(native) = &project.native else {
        return Ok(None);
    };
    for profile in ["debug", "release"] {
        let artifact = native
            .path
            .join("target")
            .join(profile)
            .join(dynamic_library_name(&native.library));
        if artifact.is_file() {
            return Ok(Some(artifact));
        }
    }
    Err(format!(
        "native artifact for '{}' is missing; run `ject build`",
        project.name
    ))
}

fn dynamic_library_name(name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{name}.dylib")
    } else {
        format!("lib{name}.so")
    }
}

pub fn init(root: &Path, name: &str, library: bool, native: bool) -> Result<Project, String> {
    let manifest = root.join(MANIFEST_FILE);
    if manifest.exists() {
        return Err(format!("{} already exists", manifest.display()));
    }
    let source_dir = root.join("src");
    fs::create_dir_all(&source_dir)
        .map_err(|e| format!("failed to create {}: {e}", source_dir.display()))?;
    let library = library || native;
    let entry_name = if library { "lib.ject" } else { "main.ject" };
    let entry_rel = format!("src/{entry_name}");
    let native_manifest = if native {
        format!(
            "\n[native]\nlanguage = \"rust\"\npath = \"native\"\nabi = \"ject-native-1\"\nlibrary = \"{}\"\n",
            name.replace('-', "_")
        )
    } else {
        String::new()
    };
    let manifest_source = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2026\"\nentry = \"{entry_rel}\"\n{native_manifest}\n[dependencies]\n"
    );
    fs::write(&manifest, manifest_source)
        .map_err(|e| format!("failed to write {}: {e}", manifest.display()))?;
    let entry = source_dir.join(entry_name);
    let starter = if native {
        format!(
            "import \"@native/{name}\" as native\n\nexport fn hello(name)\n    assert(type_of(name) == \"string\", \"hello requires a string\")\n    return native.hello(name)\nend\n"
        )
    } else if library {
        "export fn hello(name)\n    return \"Hello, $name!\"\nend\n".to_string()
    } else {
        "print \"Hello from Ject!\"\n".to_string()
    };
    fs::write(&entry, starter).map_err(|e| format!("failed to write {}: {e}", entry.display()))?;
    if native {
        scaffold_native(root, name)?;
    }
    load(root)
}

fn scaffold_native(root: &Path, name: &str) -> Result<(), String> {
    let native_src = root.join("native/src");
    fs::create_dir_all(&native_src)
        .map_err(|e| format!("failed to create {}: {e}", native_src.display()))?;
    let development_sdk = Path::new(env!("CARGO_MANIFEST_DIR")).join("crates/ject-native");
    let sdk_dependency = std::env::var_os("JECT_NATIVE_SDK_PATH")
        .map(PathBuf::from)
        .or_else(|| development_sdk.is_dir().then_some(development_sdk))
        .map(|path| format!("ject-native = {{ path = \"{}\" }}", path.display()))
        .unwrap_or_else(|| "ject-native = \"0.1\"".to_string());
    let cargo = format!(
        "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\n{sdk_dependency}\nserde_json = \"1.0\"\n\n[workspace]\n"
    );
    fs::write(root.join("native/Cargo.toml"), cargo)
        .map_err(|e| format!("failed to write native/Cargo.toml: {e}"))?;
    let rust = format!(
        "use serde_json::{{json, Value}};\n\nfn call(function: &str, args: Vec<Value>) -> Result<Value, String> {{\n    match function {{\n        \"hello\" => {{\n            let name = args.first().and_then(Value::as_str).ok_or_else(|| \"hello expects a string\".to_string())?;\n            Ok(json!(format!(\"Hello, {{name}}!\")))\n        }}\n        _ => Err(format!(\"unknown function '{{function}}'\")),\n    }}\n}}\n\nject_native::ject_plugin!(\"{name}\", [\"hello\"], call);\n"
    );
    fs::write(native_src.join("lib.rs"), rust)
        .map_err(|e| format!("failed to write native/src/lib.rs: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_manifest_defaults() {
        let root = std::env::temp_dir().join(format!("ject-package-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src/deep")).unwrap();
        fs::write(root.join(MANIFEST_FILE), "[package]\nname = \"demo\"\n").unwrap();
        let project = discover(&root.join("src/deep")).unwrap();
        assert_eq!(project.name, "demo");
        assert_eq!(project.entry, root.join("src/main.ject"));
        assert!(project.native.is_none());
        assert!(project.dependencies.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn loads_native_and_path_dependencies() {
        let root =
            std::env::temp_dir().join(format!("ject-native-manifest-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join(MANIFEST_FILE),
            "[package]\nname = \"mixed\"\n[native]\npath = \"native\"\nlibrary = \"mixed_impl\"\n[dependencies]\nhelper = { path = \"../helper\" }\n",
        )
        .unwrap();
        let project = load(&root).unwrap();
        assert_eq!(project.native.unwrap().library, "mixed_impl");
        assert_eq!(project.dependencies["helper"], root.join("../helper"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_files_include_nested_modules_and_external_entry() {
        let root = std::env::temp_dir().join(format!(
            "ject-package-sources-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(root.join("src/nested")).unwrap();
        fs::create_dir_all(root.join("app")).unwrap();
        fs::write(
            root.join(MANIFEST_FILE),
            "[package]\nname = \"sources\"\nentry = \"app/start.ject\"\n",
        )
        .unwrap();
        fs::write(root.join("src/lib.ject"), "export value = 1").unwrap();
        fs::write(root.join("src/nested/tool.ject"), "export value = 2").unwrap();
        fs::write(root.join("src/ignored.txt"), "not Ject").unwrap();
        fs::write(root.join("app/start.ject"), "print \"start\"").unwrap();

        let files = source_files(&load(&root).unwrap()).unwrap();
        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|path| path.ends_with("app/start.ject")));
        assert!(files
            .iter()
            .any(|path| path.ends_with("src/nested/tool.ject")));
        assert!(!files.iter().any(|path| path.ends_with("ignored.txt")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scaffolds_a_mixed_package() {
        let root =
            std::env::temp_dir().join(format!("ject-native-scaffold-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let project = init(&root, "hello_native", false, true).unwrap();
        assert_eq!(project.entry, root.join("src/lib.ject"));
        assert_eq!(project.native.unwrap().library, "hello_native");
        assert!(root.join("native/Cargo.toml").is_file());
        let facade = fs::read_to_string(root.join("src/lib.ject")).unwrap();
        assert!(facade.contains("@native/hello_native"));
        let rust = fs::read_to_string(root.join("native/src/lib.rs")).unwrap();
        assert!(rust.contains("ject_native::ject_plugin!"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn adds_installs_and_removes_local_dependency() {
        let base = std::env::temp_dir().join(format!("ject-install-test-{}", std::process::id()));
        let app = base.join("app");
        let library = base.join("helper");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&app).unwrap();
        fs::create_dir_all(&library).unwrap();
        init(&app, "app", false, false).unwrap();
        init(&library, "helper", true, false).unwrap();

        let project = load(&app).unwrap();
        let updated = add_path_dependency(&project, "helper", Path::new("../helper")).unwrap();
        assert!(updated.dependencies.contains_key("helper"));
        let installed = install(&updated).unwrap();
        assert_eq!(installed.len(), 1);
        let lock = fs::read_to_string(app.join(LOCK_FILE)).unwrap();
        assert!(lock.contains("name = \"app\""));
        assert!(lock.contains("name = \"helper\""));
        assert!(lock.contains("source = \"path+"));

        let updated = remove_dependency(&updated, "helper").unwrap();
        assert!(!updated.dependencies.contains_key("helper"));
        install(&updated).unwrap();
        let lock = fs::read_to_string(app.join(LOCK_FILE)).unwrap();
        assert!(!lock.contains("name = \"helper\""));
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rejects_dependency_alias_that_hides_package_name() {
        let base = std::env::temp_dir().join(format!("ject-alias-test-{}", std::process::id()));
        let app = base.join("app");
        let library = base.join("helper");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&app).unwrap();
        fs::create_dir_all(&library).unwrap();
        init(&app, "app", false, false).unwrap();
        init(&library, "helper", true, false).unwrap();
        let error =
            add_path_dependency(&load(&app).unwrap(), "wrong", Path::new("../helper")).unwrap_err();
        assert!(error.contains("named 'helper'"));
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rejects_self_dependency_even_through_parent_path() {
        let base =
            std::env::temp_dir().join(format!("ject-self-dependency-test-{}", std::process::id()));
        let app = base.join("hi");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&app).unwrap();
        let project = init(&app, "hi", false, false).unwrap();
        let error = add_path_dependency(&project, "hi", Path::new("../hi")).unwrap_err();
        assert!(error.contains("cannot depend on itself"));
        assert!(!load(&app).unwrap().dependencies.contains_key("hi"));
        fs::remove_dir_all(base).unwrap();
    }
}
