//! Source-only compilation of Ject projects into native executables.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Archive, Builder};
use toml_edit::{DocumentMut, Value};

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::module_graph::ModuleGraph;
use crate::parser::Parser;
use crate::{package, stdlib};

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub project_dir: PathBuf,
    pub output: Option<PathBuf>,
    pub release: bool,
}

impl CompileOptions {
    pub fn for_project(project_dir: impl Into<PathBuf>) -> Self {
        Self {
            project_dir: project_dir.into(),
            output: None,
            release: false,
        }
    }
}

pub fn compile_project(options: CompileOptions) -> Result<PathBuf, String> {
    let project = package::discover(&options.project_dir)?;

    if !project.entry.is_file() {
        return Err(format!(
            "package entry does not exist: {}",
            project.entry.display()
        ));
    }

    if project.native.is_some() {
        return Err(
            "binary compilation does not support a native implementation in the root package yet"
                .to_string(),
        );
    }

    let dependencies = if has_external_dependencies(&project) {
        package::verify_lockfile(&project)?
    } else {
        let dependencies = package::dependency_projects(&project)?;
        if dependencies.iter().any(has_external_dependencies) {
            package::verify_lockfile(&project)?
        } else {
            dependencies
        }
    };
    if dependencies
        .iter()
        .any(|dependency| dependency.native.is_some())
    {
        return Err(
            "binary compilation does not support native package dependencies yet".to_string(),
        );
    }
    ModuleGraph::build(&project.entry)
        .map_err(|error| format!("could not build module graph: {error}"))?;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if !manifest_dir.join("Cargo.toml").is_file() {
        return Err("build --binary requires Ject's source checkout and Cargo; build and run Ject from that checkout".into());
    }

    let workspace = BundleWorkspace::create()?;
    let temp = &workspace.0;
    let app = temp.join("app");
    let output = temp.join("output");
    fs::create_dir_all(&output)
        .map_err(|error| format!("could not create temporary output: {error}"))?;

    copy_project(&project.root, &app)?;
    let vendor = app.join(".ject-bundle-dependencies");
    if vendor.exists() {
        return Err("project contains the reserved .ject-bundle-dependencies directory".into());
    }
    rewrite_dependencies(&project, &app, ".ject-bundle-dependencies")?;
    for dependency in &dependencies {
        let destination = vendor.join(&dependency.name);
        copy_project(&dependency.root, &destination)?;
        rewrite_dependencies(dependency, &destination, "..")?;
    }

    let archive = temp.join("project.tar.gz");
    write_project_archive(&app, &archive)?;

    let cargo = format!(
        "[package]\nname = \"ject_bundle\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nject = {{ path = {:?} }}\n\n[workspace]\n",
        manifest_dir.to_string_lossy()
    );
    fs::write(temp.join("Cargo.toml"), cargo)
        .map_err(|error| format!("could not create temporary Cargo.toml: {error}"))?;
    fs::copy(manifest_dir.join("Cargo.lock"), temp.join("Cargo.lock"))
        .map_err(|error| format!("could not copy the host dependency lockfile: {error}"))?;
    fs::create_dir_all(temp.join("src"))
        .map_err(|error| format!("could not create wrapper source: {error}"))?;

    fs::copy(&archive, temp.join("src/project.tar.gz"))
        .map_err(|error| format!("could not copy embedded project: {error}"))?;
    fs::write(
        temp.join("src/main.rs"),
        "fn main() { ject::bundle::run_embedded_project(include_bytes!(\"project.tar.gz\")); }\n",
    )
    .map_err(|error| format!("could not write temporary launcher: {error}"))?;

    let mut cargo = Command::new("cargo");
    cargo
        .arg("build")
        .arg("--manifest-path")
        .arg(temp.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(&output);
    if options.release {
        cargo.arg("--release");
    }

    let status = cargo
        .status()
        .map_err(|error| format!("could not start Cargo: {error}"))?;
    if !status.success() {
        return Err("Cargo failed while compiling the Ject binary".to_string());
    }

    let profile = if options.release { "release" } else { "debug" };
    let executable_name = if cfg!(windows) {
        format!("{}.exe", project.name)
    } else {
        project.name.clone()
    };
    let built = output.join(profile).join(if cfg!(windows) {
        "ject_bundle.exe"
    } else {
        "ject_bundle"
    });
    if !built.is_file() {
        return Err(format!(
            "Cargo did not produce the expected executable: {}",
            built.display()
        ));
    }

    let destination = options.output.unwrap_or_else(|| {
        project
            .root
            .join("target")
            .join("ject-bundle")
            .join(executable_name)
    });
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("could not create output directory: {error}"))?;
    }
    fs::copy(&built, &destination).map_err(|error| {
        format!(
            "could not copy executable to {}: {error}",
            destination.display()
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&destination)
            .map_err(|error| format!("could not read executable permissions: {error}"))?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o111);
        fs::set_permissions(&destination, permissions)
            .map_err(|error| format!("could not set executable permissions: {error}"))?;
    }

    Ok(destination)
}

struct BundleWorkspace(PathBuf);

impl BundleWorkspace {
    fn create() -> Result<Self, String> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("ject-bundle-{}-{unique}", std::process::id()));
        #[cfg(unix)]
        let created = {
            use std::os::unix::fs::DirBuilderExt;
            fs::DirBuilder::new().mode(0o700).create(&directory)
        };
        #[cfg(not(unix))]
        let created = fs::create_dir(&directory);
        created.map_err(|error| {
            format!(
                "could not create bundle workspace {}: {error}",
                directory.display()
            )
        })?;
        Ok(Self(directory))
    }
}

impl Drop for BundleWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_project(source: &Path, destination: &Path) -> Result<(), String> {
    copy_tree(source, destination)
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("could not create bundle directory: {error}"))?;
    for entry in fs::read_dir(source)
        .map_err(|error| format!("could not read {}: {error}", source.display()))?
    {
        let entry = entry.map_err(|error| format!("could not read directory entry: {error}"))?;
        let name = entry.file_name();
        if name == ".git" || name == "target" || name == ".DS_Store" {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(&name);
        let metadata = fs::symlink_metadata(&source_path)
            .map_err(|error| format!("could not inspect {}: {error}", source_path.display()))?;
        if metadata.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &destination_path)
                .map_err(|error| format!("could not copy {}: {error}", source_path.display()))?;
        }
    }
    Ok(())
}

fn has_external_dependencies(project: &package::Project) -> bool {
    !project.registry_dependencies.is_empty() || !project.git_dependencies.is_empty()
}

fn rewrite_dependencies(
    project: &package::Project,
    entry: &Path,
    prefix: &str,
) -> Result<(), String> {
    if project.dependencies.is_empty() {
        return Ok(());
    }
    let bundled_manifest = entry.join(package::MANIFEST_FILE);
    let text = fs::read_to_string(&bundled_manifest)
        .map_err(|error| format!("could not read bundled manifest: {error}"))?;
    let mut document: DocumentMut = text
        .parse()
        .map_err(|error| format!("could not parse bundled manifest: {error}"))?;

    let bundled_table = document["dependencies"]
        .as_table_like_mut()
        .ok_or_else(|| "[dependencies] must be a table".to_string())?;
    for name in project.dependencies.keys() {
        let mut inline = toml_edit::InlineTable::new();
        inline.insert("path", Value::from(format!("{prefix}/{name}")));
        bundled_table.insert(name, toml_edit::value(inline));
    }

    fs::write(bundled_manifest, document.to_string())
        .map_err(|error| format!("could not write bundled manifest: {error}"))
}

fn write_project_archive(app: &Path, archive: &Path) -> Result<(), String> {
    let file = fs::File::create(archive)
        .map_err(|error| format!("could not create project archive: {error}"))?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);
    builder
        .append_dir_all(".", app)
        .map_err(|error| format!("could not archive project: {error}"))?;
    let encoder = builder
        .into_inner()
        .map_err(|error| format!("could not finish project archive: {error}"))?;
    encoder
        .finish()
        .map_err(|error| format!("could not finish gzip archive: {error}"))?;
    Ok(())
}

pub fn run_embedded_project(archive: &[u8]) {
    if let Err(error) = execute_embedded_project(archive) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn execute_embedded_project(archive: &[u8]) -> Result<(), String> {
    let workspace = BundleWorkspace::create()?;
    let temp = &workspace.0;
    let decoder = GzDecoder::new(archive);
    let mut tar = Archive::new(decoder);
    tar.unpack(temp)
        .map_err(|error| format!("failed to extract Ject bundle: {error}"))?;

    let project = package::discover(temp)
        .map_err(|error| format!("failed to load bundled Ject project: {error}"))?;
    let source = fs::read_to_string(&project.entry)
        .map_err(|error| format!("failed to read bundled entry: {error}"))?;

    stdlib::set_script_args(&std::env::args().skip(1).collect::<Vec<_>>());
    let mut interpreter = Interpreter::new();
    if let Some(parent) = project.entry.parent() {
        interpreter.set_script_dir(parent.to_path_buf());
    }

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize_with_positions();
    let positioned = tokens
        .into_iter()
        .map(|located| (located.token, located.position))
        .collect::<Vec<_>>();
    let mut parser = Parser::new(positioned);
    let statements = parser.parse().map_err(|error| {
        format!(
            "{}:{}: {}",
            project.entry.display(),
            error.line.unwrap_or(1),
            error.message
        )
    })?;

    interpreter
        .interpret(&statements)
        .map_err(|error| format!("runtime error: {}", error.message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_manifest_rewrites_all_dependency_sources_to_local_paths() {
        let workspace = BundleWorkspace::create().unwrap();
        let root = &workspace.0;
        fs::write(
            root.join(package::MANIFEST_FILE),
            r#"
[package]
name = "example"
[dependencies]
local = { path = "../local" }
remote = { git = "https://example.test/remote", rev = "0123456789abcdef0123456789abcdef01234567" }
registered = { version = "1.0.0", registry = "https://example.test/registry" }
"#,
        )
        .unwrap();
        let original = package::load(root).unwrap();
        assert!(has_external_dependencies(&original));
        rewrite_dependencies(&original, root, ".ject-bundle-dependencies").unwrap();

        let bundled = package::load(root).unwrap();
        assert!(!has_external_dependencies(&bundled));
        for name in ["local", "remote", "registered"] {
            assert_eq!(
                bundled.dependencies[name],
                root.join(".ject-bundle-dependencies").join(name)
            );
        }
    }
}
