//! Source-only compilation of Ject projects into native executables.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Archive, Builder};
use toml_edit::{DocumentMut, Item, Value};

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

    let dependencies = package::dependency_projects(&project)?;
    if dependencies
        .iter()
        .any(|dependency| dependency.native.is_some())
    {
        return Err(
            "binary compilation does not support native package dependencies yet".to_string(),
        );
    }
    if !project.registry_dependencies.is_empty() || !project.git_dependencies.is_empty() {
        return Err(
            "binary compilation currently supports only local path dependencies".to_string(),
        );
    }

    ModuleGraph::build(&project.entry)
        .map_err(|error| format!("could not build module graph: {error}"))?;

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp = std::env::temp_dir().join(format!("ject-binary-{}-{unique}", std::process::id()));
    let app = temp.join("app");
    let output = temp.join("output");
    fs::create_dir_all(&output)
        .map_err(|error| format!("could not create temporary output: {error}"))?;

    copy_project(&project.root, &app)?;
    rewrite_root_dependencies(&project.root, &app)?;

    let mut copied = HashSet::new();
    copy_dependency_projects(&project, &app.join("vendor"), &mut copied)?;

    let archive = temp.join("project.tar.gz");
    write_project_archive(&app, &archive)?;

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo = format!(
        "[package]\nname = \"ject_bundle\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\nject = {{ path = {:?} }}\n\n[workspace]\n",
        manifest_dir.to_string_lossy()
    );
    fs::write(temp.join("Cargo.toml"), cargo)
        .map_err(|error| format!("could not create temporary Cargo.toml: {error}"))?;
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

fn copy_dependency_projects(
    project: &package::Project,
    vendor: &Path,
    copied: &mut HashSet<String>,
) -> Result<(), String> {
    for (alias, source) in &project.dependencies {
        if !copied.insert(source.to_string_lossy().into_owned()) {
            continue;
        }
        let destination = vendor.join(alias);
        copy_project(source, &destination)?;

        let dependency = package::load(source)
            .map_err(|error| format!("could not load bundled dependency '{alias}': {error}"))?;
        copy_dependency_projects(&dependency, vendor, copied)?;

        rewrite_manifest_dependencies(source, &destination)?;
    }

    Ok(())
}

fn rewrite_root_dependencies(source_root: &Path, bundled_root: &Path) -> Result<(), String> {
    let manifest = bundled_root.join(package::MANIFEST_FILE);
    rewrite_manifest_path_dependencies(&source_root.join(package::MANIFEST_FILE), &manifest, true)
}

fn rewrite_manifest_dependencies(source_root: &Path, bundled_root: &Path) -> Result<(), String> {
    rewrite_manifest_path_dependencies(
        &source_root.join(package::MANIFEST_FILE),
        &bundled_root.join(package::MANIFEST_FILE),
        false,
    )
}

fn rewrite_manifest_path_dependencies(
    source_manifest: &Path,
    bundled_manifest: &Path,
    root: bool,
) -> Result<(), String> {
    let text = fs::read_to_string(bundled_manifest)
        .map_err(|error| format!("could not read bundled manifest: {error}"))?;
    let mut document: DocumentMut = text
        .parse()
        .map_err(|error| format!("could not parse bundled manifest: {error}"))?;

    let source_text = fs::read_to_string(source_manifest)
        .map_err(|error| format!("could not read source manifest: {error}"))?;
    let source_document: DocumentMut = source_text
        .parse()
        .map_err(|error| format!("could not parse source manifest: {error}"))?;

    if let Some(dependencies) = source_document.get("dependencies").and_then(Item::as_table) {
        let bundled_table = document
            .entry("dependencies")
            .or_insert(toml_edit::table())
            .as_table_mut()
            .ok_or_else(|| "[dependencies] must be a table".to_string())?;

        for (alias, original) in dependencies.iter() {
            let inline = original.as_inline_table();
            let table = original.as_table();
            let path = if let Some(inline) = inline {
                inline.get("path").and_then(toml_edit::Value::as_str)
            } else if let Some(table) = table {
                table.get("path").and_then(toml_edit::Item::as_str)
            } else {
                None
            };
            if path.is_some() {
                let bundled_relative = if root {
                    format!("vendor/{alias}")
                } else {
                    format!("../{alias}")
                };
                if bundled_table.contains_key(alias) {
                    bundled_table[alias] = toml_edit::value(toml_edit::InlineTable::new());
                }
                let mut inline = toml_edit::InlineTable::new();
                inline.insert("path", Value::from(bundled_relative));
                bundled_table[alias] = toml_edit::value(inline);
            }
        }
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
    let temp = std::env::temp_dir().join(format!("ject-bundle-runtime-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    if let Err(error) = fs::create_dir_all(&temp) {
        eprintln!("failed to create bundle runtime directory: {error}");
        std::process::exit(1);
    }

    let decoder = GzDecoder::new(archive);
    let mut tar = Archive::new(decoder);
    if let Err(error) = tar.unpack(&temp) {
        eprintln!("failed to extract Ject bundle: {error}");
        std::process::exit(1);
    }

    let project = match package::discover(&temp) {
        Ok(project) => project,
        Err(error) => {
            eprintln!("failed to load bundled Ject project: {error}");
            std::process::exit(1);
        }
    };
    let source = match fs::read_to_string(&project.entry) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read bundled entry: {error}");
            std::process::exit(1);
        }
    };

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
    let statements = match parser.parse() {
        Ok(statements) => statements,
        Err(error) => {
            eprintln!(
                "{}:{}: {}",
                project.entry.display(),
                error.line.unwrap_or(1),
                error.message
            );
            std::process::exit(1);
        }
    };

    if let Err(error) = interpreter.interpret(&statements) {
        eprintln!("runtime error: {}", error.message);
        std::process::exit(1);
    }
}
