//! Project discovery, local dependency installation, locking, and native builds.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use semver::{Version, VersionReq};
use sha2::{Digest, Sha256};
use tar::{Archive, Builder};

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
    pub registry_dependencies: HashMap<String, RegistryDependency>,
    pub git_dependencies: HashMap<String, GitDependency>,
    pub registry_source: Option<RegistrySource>,
    pub git_source: Option<GitSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryDependency {
    pub version: String,
    pub requirement: Option<String>,
    pub registry: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitDependency {
    pub url: String,
    pub revision: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyUpdate {
    pub name: String,
    pub previous: String,
    pub current: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrySource {
    pub registry: String,
    pub archive_checksum: String,
    pub content_checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitSource {
    pub url: String,
    pub revision: String,
    pub content_checksum: String,
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
    let mut registry_dependencies = HashMap::new();
    let mut git_dependencies = HashMap::new();

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
                } else if let Some(url) = inline_string(raw_value, "git") {
                    let revision = inline_string(raw_value, "rev").ok_or_else(|| {
                        format!("git dependency '{dependency}' is missing an exact rev")
                    })?;
                    let reference = inline_string(raw_value, "branch")
                        .map(|branch| format!("branch:{branch}"))
                        .or_else(|| inline_string(raw_value, "tag").map(|tag| format!("tag:{tag}")))
                        .or_else(|| inline_string(raw_value, "head").map(|_| "HEAD".to_string()));
                    dependencies.insert(
                        dependency.to_string(),
                        git_cache_root()?
                            .join(repository_hash(&url))
                            .join(&revision),
                    );
                    git_dependencies.insert(
                        dependency.to_string(),
                        GitDependency {
                            url,
                            revision,
                            reference,
                        },
                    );
                } else if let Some(version) = dependency_version(raw_value) {
                    let requirement = inline_string(raw_value, "requirement");
                    let registry = dependency_registry(raw_value)
                        .or_else(|| std::env::var("JECT_REGISTRY").ok())
                        .unwrap_or_else(|| "https://packages.ject.dev".to_string());
                    dependencies.insert(
                        dependency.to_string(),
                        registry_cache_root()?.join(dependency).join(&version),
                    );
                    registry_dependencies.insert(
                        dependency.to_string(),
                        RegistryDependency {
                            version,
                            requirement,
                            registry,
                        },
                    );
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
    let registry_source = read_registry_source(root)?;
    let git_source = read_git_source(root)?;
    Ok(Project {
        root: root.to_path_buf(),
        name,
        version: version.unwrap_or_else(|| "0.0.0".to_string()),
        entry,
        native,
        dependencies,
        registry_dependencies,
        git_dependencies,
        registry_source,
        git_source,
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
    update_dependency_value(
        &project.root.join(MANIFEST_FILE),
        name,
        Some(dependency_line(path)),
    )?;
    load(&project.root)
}

pub fn add_registry_dependency(
    project: &Project,
    name: &str,
    version: &str,
    registry: Option<&str>,
) -> Result<Project, String> {
    validate_package_name(name)?;
    let registry = registry
        .map(str::to_string)
        .or_else(|| std::env::var("JECT_REGISTRY").ok())
        .unwrap_or_else(|| "https://packages.ject.dev".to_string());
    let resolved_version = resolve_registry_version(name, version, &registry)?;
    let dependency = RegistryDependency {
        version: resolved_version.clone(),
        requirement: (resolved_version != version).then(|| version.to_string()),
        registry: registry.clone(),
    };
    let destination = registry_cache_root()?.join(name).join(&resolved_version);
    if !destination.join(MANIFEST_FILE).is_file() {
        download_registry_package(name, &dependency, &destination)?;
    }
    let installed = load(&destination)?;
    if installed.name != name || installed.version != resolved_version {
        return Err(format!(
            "registry package {name}@{resolved_version} identifies itself as {}@{}",
            installed.name, installed.version
        ));
    }
    let registry = registry.replace('\\', "\\\\").replace('"', "\\\"");
    let value = if resolved_version == version {
        format!("{{ version = \"{resolved_version}\", registry = \"{registry}\" }}")
    } else {
        format!("{{ version = \"{resolved_version}\", requirement = \"{version}\", registry = \"{registry}\" }}")
    };
    update_dependency_value(&project.root.join(MANIFEST_FILE), name, Some(value))?;
    load(&project.root)
}

pub fn add_git_dependency(
    project: &Project,
    name: &str,
    url: &str,
    reference: Option<&str>,
) -> Result<Project, String> {
    validate_package_name(name)?;
    let reference = reference.unwrap_or("HEAD");
    let revision = resolve_git_revision(url, reference)?;
    let tracked = (!reference.starts_with("rev:")).then(|| reference.to_string());
    let dependency = GitDependency {
        url: url.to_string(),
        revision: revision.clone(),
        reference: tracked.clone(),
    };
    let destination = git_cache_root()?.join(repository_hash(url)).join(&revision);
    materialize_git_dependency(name, &dependency, &destination)?;
    let installed = load(&destination)?;
    validate_git_install(name, &dependency, &installed)?;
    if installed.name != name {
        return Err(format!(
            "git package at {url} identifies itself as '{}' instead of '{name}'",
            installed.name
        ));
    }
    let url = escape_manifest_string(url);
    let selector = match tracked.as_deref() {
        Some("HEAD") => ", head = \"true\"".to_string(),
        Some(reference) if reference.starts_with("branch:") => {
            format!(", branch = \"{}\"", escape_manifest_string(&reference[7..]))
        }
        Some(reference) if reference.starts_with("tag:") => {
            format!(", tag = \"{}\"", escape_manifest_string(&reference[4..]))
        }
        _ => String::new(),
    };
    let value = format!("{{ git = \"{url}\", rev = \"{revision}\"{selector} }}");
    update_dependency_value(&project.root.join(MANIFEST_FILE), name, Some(value))?;
    load(&project.root)
}

pub fn remove_dependency(project: &Project, name: &str) -> Result<Project, String> {
    if !project.dependencies.contains_key(name) {
        return Err(format!("package '{name}' is not a dependency"));
    }
    update_dependency_value(&project.root.join(MANIFEST_FILE), name, None)?;
    load(&project.root)
}

pub fn update_dependencies(
    project: &Project,
    selected: Option<&str>,
) -> Result<(Project, Vec<DependencyUpdate>), String> {
    if let Some(name) = selected {
        if !project.dependencies.contains_key(name) {
            return Err(format!("package '{name}' is not a dependency"));
        }
        if !project.registry_dependencies.contains_key(name)
            && !project.git_dependencies.contains_key(name)
        {
            return Err(format!(
                "package '{name}' is a path dependency and cannot be updated"
            ));
        }
    }
    let mut names: Vec<_> = project.registry_dependencies.keys().cloned().collect();
    names.sort();
    let mut changes = Vec::new();
    for name in names {
        if selected.is_some_and(|selected| selected != name) {
            continue;
        }
        let dependency = &project.registry_dependencies[&name];
        let Some(requirement) = dependency.requirement.as_deref() else {
            continue;
        };
        let resolved = resolve_registry_version(&name, requirement, &dependency.registry)?;
        if resolved == dependency.version {
            continue;
        }
        let registry = dependency
            .registry
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        let value = format!(
            "{{ version = \"{resolved}\", requirement = \"{requirement}\", registry = \"{registry}\" }}"
        );
        update_dependency_value(&project.root.join(MANIFEST_FILE), &name, Some(value))?;
        changes.push(DependencyUpdate {
            name,
            previous: dependency.version.clone(),
            current: resolved,
        });
    }
    let mut names: Vec<_> = project.git_dependencies.keys().cloned().collect();
    names.sort();
    for name in names {
        if selected.is_some_and(|selected| selected != name) {
            continue;
        }
        let dependency = &project.git_dependencies[&name];
        let Some(reference) = dependency.reference.as_deref() else {
            continue;
        };
        let resolved = resolve_git_revision(&dependency.url, reference)?;
        if resolved == dependency.revision {
            continue;
        }
        let selector = if reference == "HEAD" {
            ", head = \"true\"".to_string()
        } else if let Some(branch) = reference.strip_prefix("branch:") {
            format!(", branch = \"{}\"", escape_manifest_string(branch))
        } else if let Some(tag) = reference.strip_prefix("tag:") {
            format!(", tag = \"{}\"", escape_manifest_string(tag))
        } else {
            return Err(format!("invalid git reference '{reference}'"));
        };
        let url = escape_manifest_string(&dependency.url);
        let value = format!("{{ git = \"{url}\", rev = \"{resolved}\"{selector} }}");
        update_dependency_value(&project.root.join(MANIFEST_FILE), &name, Some(value))?;
        changes.push(DependencyUpdate {
            name,
            previous: dependency.revision.clone(),
            current: resolved,
        });
    }
    Ok((load(&project.root)?, changes))
}

fn escape_manifest_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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

fn update_dependency_value(
    manifest: &Path,
    name: &str,
    value: Option<String>,
) -> Result<(), String> {
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
                if let Some(value) = &value {
                    output.push(format!("{name} = {value}"));
                }
                written = true;
            }
            in_dependencies = trimmed == "[dependencies]";
            found_section |= in_dependencies;
        }
        if in_dependencies && trimmed.split_once('=').map(|(key, _)| key.trim()) == Some(name) {
            if let Some(value) = &value {
                output.push(format!("{name} = {value}"));
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
        if let Some(value) = &value {
            output.push(format!("{name} = {value}"));
        }
    }
    let updated = format!("{}\n", output.join("\n"));
    let temporary = manifest.with_extension("toml.tmp");
    fs::write(&temporary, updated)
        .map_err(|e| format!("failed to write {}: {e}", temporary.display()))?;
    fs::rename(&temporary, manifest)
        .map_err(|e| format!("failed to replace {}: {e}", manifest.display()))
}

fn dependency_line(path: &Path) -> String {
    let escaped = path
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    format!("{{ path = \"{escaped}\" }}")
}

/// Resolve every dependency and write a deterministic, content-verified lockfile.
pub fn install(project: &Project) -> Result<Vec<Project>, String> {
    materialize_dependencies(project)?;
    let project = load(&project.root)?;
    let dependencies = dependency_projects(&project)?;
    let lock = render_lockfile(&project, &dependencies)?;
    write_lockfile(&project, &lock)?;
    Ok(dependencies)
}

/// Verify that the existing lockfile exactly describes the current dependency graph.
/// This never changes the lockfile and is intended for reproducible CI builds.
pub fn verify_lockfile(project: &Project) -> Result<Vec<Project>, String> {
    materialize_dependencies(project)?;
    let project = load(&project.root)?;
    let dependencies = dependency_projects(&project)?;
    let expected = render_lockfile(&project, &dependencies)?;
    let path = project.root.join(LOCK_FILE);
    let actual = fs::read_to_string(&path).map_err(|error| {
        format!(
            "locked install requires {}; run `ject install` to create it: {error}",
            path.display()
        )
    })?;
    if actual != expected {
        return Err(format!(
            "{} is out of date; run `ject install` and commit the result",
            path.display()
        ));
    }
    Ok(dependencies)
}

fn render_lockfile(project: &Project, dependencies: &[Project]) -> Result<String, String> {
    let mut locked = dependencies.to_vec();
    locked.push(project.clone());
    locked.sort_by(|a, b| a.name.cmp(&b.name).then(a.root.cmp(&b.root)));
    let mut lock = String::from("# Generated by Ject. Do not edit by hand.\nversion = 2\n");
    for package in &locked {
        let canonical = package
            .root
            .canonicalize()
            .map_err(|e| format!("failed to lock {}: {e}", package.root.display()))?;
        lock.push_str("\n[[package]]\n");
        lock.push_str(&format!("name = \"{}\"\n", package.name));
        lock.push_str(&format!("version = \"{}\"\n", package.version));
        if let Some(source) = &package.registry_source {
            lock.push_str(&format!(
                "source = \"registry+{}\"\n",
                source.registry.replace('"', "\\\"")
            ));
            lock.push_str(&format!(
                "archive-checksum = \"{}\"\n",
                source.archive_checksum
            ));
        } else if let Some(source) = &package.git_source {
            lock.push_str(&format!(
                "source = \"git+{}#{}\"\n",
                source.url.replace('"', "\\\""),
                source.revision
            ));
        } else {
            lock.push_str(&format!("source = \"path+{}\"\n", canonical.display()));
        }
        lock.push_str(&format!("checksum = \"{}\"\n", package_checksum(package)?));
        lock.push_str(&format!("native = {}\n", package.native.is_some()));
    }
    Ok(lock)
}

fn write_lockfile(project: &Project, lock: &str) -> Result<(), String> {
    let destination = project.root.join(LOCK_FILE);
    let temporary = project.root.join("Ject.lock.tmp");
    fs::write(&temporary, lock)
        .map_err(|e| format!("failed to write {}: {e}", temporary.display()))?;
    fs::rename(&temporary, &destination)
        .map_err(|e| format!("failed to replace {}: {e}", destination.display()))
}

/// Hash all package inputs that affect its Ject or native implementation.
pub fn package_checksum(project: &Project) -> Result<String, String> {
    package_checksum_root(&project.root)
}

fn package_checksum_root(root: &Path) -> Result<String, String> {
    fn collect(root: &Path, directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
        let entries = fs::read_dir(directory)
            .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
        for entry in entries {
            let path = entry
                .map_err(|error| format!("failed to read {}: {error}", directory.display()))?
                .path();
            let relative = path.strip_prefix(root).unwrap_or(&path);
            if path.is_dir() {
                if matches!(
                    relative.file_name().and_then(|name| name.to_str()),
                    Some("target" | ".git")
                ) {
                    continue;
                }
                collect(root, &path, files)?;
            } else if !matches!(
                relative.file_name().and_then(|name| name.to_str()),
                Some(LOCK_FILE | "Ject.lock.tmp" | ".ject-source" | ".ject-git-source")
            ) {
                files.push(path);
            }
        }
        Ok(())
    }

    let root = root
        .canonicalize()
        .map_err(|error| format!("failed to checksum package {}: {error}", root.display()))?;
    let mut files = Vec::new();
    collect(&root, &root, &mut files)?;
    files.sort();
    let mut digest = Sha256::new();
    for path in files {
        let relative = path.strip_prefix(&root).unwrap_or(&path);
        let bytes = fs::read(&path)
            .map_err(|error| format!("failed to checksum {}: {error}", path.display()))?;
        let relative = relative.to_string_lossy();
        digest.update((relative.len() as u64).to_le_bytes());
        digest.update(relative.as_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn dependency_path(value: &str) -> Option<PathBuf> {
    inline_string(value, "path").map(PathBuf::from)
}

fn inline_string(value: &str, key: &str) -> Option<String> {
    let value = value.trim().trim_start_matches('{').trim_end_matches('}');
    let mut quoted = false;
    let mut escaped = false;
    let mut start = 0;
    for (index, character) in value.char_indices().chain([(value.len(), ',')]) {
        if character == '"' && !escaped {
            quoted = !quoted;
        }
        if character == ',' && !quoted {
            let field = &value[start..index];
            if let Some((field_key, field_value)) = field.split_once('=') {
                if field_key.trim() == key {
                    return Some(field_value.trim().trim_matches('"').to_string());
                }
            }
            start = index + character.len_utf8();
        }
        escaped = character == '\\' && !escaped;
        if character != '\\' {
            escaped = false;
        }
    }
    None
}

fn dependency_version(value: &str) -> Option<String> {
    if value.starts_with('"') {
        return Some(value.trim_matches('"').to_string());
    }
    inline_string(value, "version")
}

fn dependency_registry(value: &str) -> Option<String> {
    inline_string(value, "registry")
}

fn registry_cache_root() -> Result<PathBuf, String> {
    if let Some(root) = std::env::var_os("JECT_HOME") {
        return Ok(PathBuf::from(root).join("registry/src"));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|root| root.join(".ject/registry/src"))
        .ok_or_else(|| "cannot locate Ject cache; set JECT_HOME".to_string())
}

fn git_cache_root() -> Result<PathBuf, String> {
    if let Some(root) = std::env::var_os("JECT_HOME") {
        return Ok(PathBuf::from(root).join("git/checkouts"));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|root| root.join(".ject/git/checkouts"))
        .ok_or_else(|| "cannot locate Ject cache; set JECT_HOME".to_string())
}

fn repository_hash(url: &str) -> String {
    format!("{:x}", Sha256::digest(url.as_bytes()))
}

fn read_registry_source(root: &Path) -> Result<Option<RegistrySource>, String> {
    let path = root.join(".ject-source");
    if !path.is_file() {
        return Ok(None);
    }
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let mut lines = source.lines();
    let registry = lines.next().unwrap_or("").to_string();
    let archive_checksum = lines.next().unwrap_or("").to_string();
    let content_checksum = lines.next().map(str::to_string);
    if registry.is_empty() || archive_checksum.len() != 64 {
        return Err(format!("invalid registry metadata in {}", path.display()));
    }
    if let Some(expected) = &content_checksum {
        let actual = package_checksum_root(root)?;
        if expected != &actual {
            return Err(format!(
                "cached registry package at {} was modified: expected {expected}, got {actual}; remove it and run `ject install`",
                root.display()
            ));
        }
    }
    Ok(Some(RegistrySource {
        registry,
        archive_checksum,
        content_checksum,
    }))
}

fn read_git_source(root: &Path) -> Result<Option<GitSource>, String> {
    let path = root.join(".ject-git-source");
    if !path.is_file() {
        return Ok(None);
    }
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let mut lines = source.lines();
    let url = lines.next().unwrap_or("").to_string();
    let revision = lines.next().unwrap_or("").to_string();
    let content_checksum = lines.next().unwrap_or("").to_string();
    if url.is_empty() || !is_full_git_revision(&revision) || content_checksum.len() != 64 {
        return Err(format!("invalid git metadata in {}", path.display()));
    }
    let actual = package_checksum_root(root)?;
    if actual != content_checksum {
        return Err(format!(
            "cached git package at {} was modified: expected {content_checksum}, got {actual}; remove it and run `ject install`",
            root.display()
        ));
    }
    Ok(Some(GitSource {
        url,
        revision,
        content_checksum,
    }))
}

fn is_full_git_revision(revision: &str) -> bool {
    revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn resolve_git_revision(url: &str, reference: &str) -> Result<String, String> {
    if let Some(revision) = reference.strip_prefix("rev:") {
        if is_full_git_revision(revision) {
            return Ok(revision.to_ascii_lowercase());
        }
        return Err("--rev requires a full 40-character commit ID".to_string());
    }
    let remote_reference = if reference == "HEAD" {
        "HEAD".to_string()
    } else if let Some(branch) = reference.strip_prefix("branch:") {
        format!("refs/heads/{branch}")
    } else if let Some(tag) = reference.strip_prefix("tag:") {
        format!("refs/tags/{tag}")
    } else {
        return Err(format!("invalid git reference '{reference}'"));
    };
    let output = Command::new("git")
        .arg("ls-remote")
        .arg(url)
        .arg(&remote_reference)
        .arg(format!("{remote_reference}^{{}}"))
        .output()
        .map_err(|error| format!("failed to start git: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git could not resolve {reference} from {url}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .rev()
        .filter_map(|line| line.split_whitespace().next())
        .find(|revision| is_full_git_revision(revision))
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| format!("git reference '{reference}' was not found at {url}"))
}

fn validate_version(version: &str) -> Result<(), String> {
    Version::parse(version)
        .map(|_| ())
        .map_err(|error| format!("invalid semantic version '{version}': {error}"))
}

fn registry_url(registry: &str, name: &str, version: &str, suffix: &str) -> String {
    format!(
        "{}/{name}/{version}.tar.gz{suffix}",
        registry.trim_end_matches('/')
    )
}

fn registry_index_url(registry: &str, name: &str) -> String {
    format!("{}/{name}/index.json", registry.trim_end_matches('/'))
}

fn registry_versions(registry: &str, name: &str) -> Result<Vec<Version>, String> {
    let url = registry_index_url(registry, name);
    let bytes = read_registry_object(&url)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid registry index {url}: {error}"))?;
    let versions = value
        .get("versions")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| format!("registry index {url} has no versions array"))?;
    let mut parsed = versions
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(|version| {
            Version::parse(version)
                .map_err(|error| format!("invalid version '{version}' in {url}: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    parsed.sort();
    parsed.dedup();
    Ok(parsed)
}

fn resolve_registry_version(
    name: &str,
    requirement: &str,
    registry: &str,
) -> Result<String, String> {
    if let Ok(version) = Version::parse(requirement) {
        return Ok(version.to_string());
    }
    let requirement = VersionReq::parse(requirement)
        .map_err(|error| format!("invalid version requirement '{requirement}': {error}"))?;
    registry_versions(registry, name)?
        .into_iter()
        .rev()
        .find(|version| requirement.matches(version))
        .map(|version| version.to_string())
        .ok_or_else(|| format!("no version of '{name}' matches '{requirement}' in {registry}"))
}

fn read_registry_object(url: &str) -> Result<Vec<u8>, String> {
    read_optional_registry_object(url)?.ok_or_else(|| format!("registry object not found: {url}"))
}

fn read_optional_registry_object(url: &str) -> Result<Option<Vec<u8>>, String> {
    if let Some(path) = url.strip_prefix("file://") {
        return match fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("failed to read {url}: {error}")),
        };
    }
    let response = match ureq::get(url).call() {
        Ok(response) => response,
        Err(ureq::Error::Status(404, _)) => return Ok(None),
        Err(error) => return Err(format!("failed to download {url}: {error}")),
    };
    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|error| format!("failed to read response from {url}: {error}"))?;
    Ok(Some(bytes))
}

fn write_registry_object(url: &str, bytes: &[u8], token: Option<&str>) -> Result<(), String> {
    if let Some(path) = url.strip_prefix("file://") {
        let path = Path::new(path);
        if path.exists() {
            if fs::read(path).map_err(|error| format!("failed to read {url}: {error}"))? == bytes {
                return Ok(());
            }
            return Err(format!(
                "registry object {url} already exists; published versions are immutable"
            ));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, bytes)
            .map_err(|error| format!("failed to write {}: {error}", temporary.display()))?;
        return fs::rename(&temporary, path)
            .map_err(|error| format!("failed to publish {}: {error}", path.display()));
    }
    let mut request = ureq::put(url)
        .set("Content-Type", "application/octet-stream")
        .set("If-None-Match", "*");
    if let Some(token) = token {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    match request.send_bytes(bytes) {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(409 | 412, _)) => {
            if read_optional_registry_object(url)?.as_deref() == Some(bytes) {
                Ok(())
            } else {
                Err(format!(
                    "registry object {url} already exists; published versions are immutable"
                ))
            }
        }
        Err(error) => Err(format!("failed to publish {url}: {error}")),
    }
}

fn replace_registry_object(url: &str, bytes: &[u8], token: Option<&str>) -> Result<(), String> {
    if let Some(path) = url.strip_prefix("file://") {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        let temporary = path.with_extension("tmp");
        fs::write(&temporary, bytes)
            .map_err(|error| format!("failed to write {}: {error}", temporary.display()))?;
        return fs::rename(&temporary, path)
            .map_err(|error| format!("failed to replace {}: {error}", path.display()));
    }
    let mut request = ureq::put(url).set("Content-Type", "application/json");
    if let Some(token) = token {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    request
        .send_bytes(bytes)
        .map_err(|error| format!("failed to update {url}: {error}"))?;
    Ok(())
}

fn publish_registry_index(
    registry: &str,
    name: &str,
    version: &str,
    token: Option<&str>,
) -> Result<(), String> {
    let url = registry_index_url(registry, name);
    let mut versions = match read_optional_registry_object(&url)? {
        Some(bytes) => serde_json::from_slice::<serde_json::Value>(&bytes)
            .map_err(|error| format!("invalid registry index {url}: {error}"))?
            .get("versions")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .ok_or_else(|| format!("registry index {url} has no versions array"))?,
        None => Vec::new(),
    };
    if !versions.iter().any(|value| value.as_str() == Some(version)) {
        versions.push(serde_json::Value::String(version.to_string()));
    }
    versions.sort_by(|left, right| {
        let left = left.as_str().and_then(|value| Version::parse(value).ok());
        let right = right.as_str().and_then(|value| Version::parse(value).ok());
        left.cmp(&right)
    });
    let bytes = serde_json::to_vec_pretty(&serde_json::json!({ "versions": versions }))
        .map_err(|error| format!("failed to encode registry index: {error}"))?;
    replace_registry_object(&url, &bytes, token)
}

/// Publish an immutable package archive and checksum to an HTTP(S) or file registry.
pub fn publish(project: &Project, registry: &str, token: Option<&str>) -> Result<String, String> {
    validate_package_name(&project.name)?;
    validate_version(&project.version)?;
    if project.version == "0.0.0" {
        return Err("set [package].version before publishing".to_string());
    }
    if project.dependencies.len()
        != project.registry_dependencies.len() + project.git_dependencies.len()
    {
        return Err(
            "published packages cannot contain path dependencies; use registry or git sources"
                .to_string(),
        );
    }
    if !project.entry.is_file() {
        return Err(format!(
            "package entry does not exist: {}",
            project.entry.display()
        ));
    }
    if let Some(native) = &project.native {
        let cargo_lock = native.path.join("Cargo.lock");
        if !cargo_lock.is_file() {
            return Err(format!(
                "native packages require a committed {} before publishing; run `cargo generate-lockfile --manifest-path {}`",
                cargo_lock.display(),
                native.path.join("Cargo.toml").display()
            ));
        }
    }
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut archive = Builder::new(&mut encoder);
        append_package_directory(&mut archive, &project.root, &project.root)?;
        archive
            .finish()
            .map_err(|error| format!("failed to finish package archive: {error}"))?;
    }
    let bytes = encoder
        .finish()
        .map_err(|error| format!("failed to compress package archive: {error}"))?;
    let checksum = format!("{:x}", Sha256::digest(&bytes));
    let archive_url = registry_url(registry, &project.name, &project.version, "");
    let checksum_url = registry_url(registry, &project.name, &project.version, ".sha256");
    write_registry_object(&archive_url, &bytes, token)?;
    write_registry_object(&checksum_url, format!("{checksum}\n").as_bytes(), token)?;
    publish_registry_index(registry, &project.name, &project.version, token)?;
    Ok(checksum)
}

fn append_package_directory<W: Write>(
    archive: &mut Builder<W>,
    root: &Path,
    directory: &Path,
) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        if matches!(
            name.to_str(),
            Some(
                "target"
                    | ".git"
                    | LOCK_FILE
                    | "Ject.lock.tmp"
                    | ".ject-source"
                    | ".ject-git-source"
            )
        ) {
            continue;
        }
        if path.is_dir() {
            append_package_directory(archive, root, &path)?;
        } else if path.is_file() {
            let relative = path.strip_prefix(root).unwrap_or(&path);
            archive
                .append_path_with_name(&path, relative)
                .map_err(|error| format!("failed to archive {}: {error}", path.display()))?;
        }
    }
    Ok(())
}

fn materialize_dependencies(project: &Project) -> Result<(), String> {
    for (name, dependency) in &project.registry_dependencies {
        validate_package_name(name)?;
        validate_version(&dependency.version)?;
        let destination = project
            .dependencies
            .get(name)
            .ok_or_else(|| format!("registry dependency '{name}' has no cache destination"))?;
        if !destination.join(MANIFEST_FILE).is_file() {
            download_registry_package(name, dependency, destination)?;
        }
        let installed = load(destination)?;
        if installed.name != *name || installed.version != dependency.version {
            return Err(format!(
                "registry package {name}@{} identifies itself as {}@{}",
                dependency.version, installed.name, installed.version
            ));
        }
        materialize_dependencies(&installed)?;
    }
    for (name, dependency) in &project.git_dependencies {
        validate_package_name(name)?;
        if !is_full_git_revision(&dependency.revision) {
            return Err(format!(
                "git dependency '{name}' must use a full 40-character rev"
            ));
        }
        let destination = project
            .dependencies
            .get(name)
            .ok_or_else(|| format!("git dependency '{name}' has no cache destination"))?;
        materialize_git_dependency(name, dependency, destination)?;
        let installed = load(destination)?;
        validate_git_install(name, dependency, &installed)?;
        if installed.name != *name {
            return Err(format!(
                "git package at {} identifies itself as '{}' instead of '{name}'",
                dependency.url, installed.name
            ));
        }
        materialize_dependencies(&installed)?;
    }
    Ok(())
}

fn validate_git_install(
    name: &str,
    dependency: &GitDependency,
    installed: &Project,
) -> Result<(), String> {
    let source = installed.git_source.as_ref().ok_or_else(|| {
        format!(
            "cached git package '{name}' at {} has no source metadata; remove it and run `ject install`",
            installed.root.display()
        )
    })?;
    if source.url != dependency.url || source.revision != dependency.revision {
        return Err(format!(
            "cached git package '{name}' has source {}#{}, expected {}#{}; remove it and run `ject install`",
            source.url, source.revision, dependency.url, dependency.revision
        ));
    }
    Ok(())
}

fn materialize_git_dependency(
    name: &str,
    dependency: &GitDependency,
    destination: &Path,
) -> Result<(), String> {
    if destination.join(MANIFEST_FILE).is_file() {
        return Ok(());
    }
    let parent = destination
        .parent()
        .ok_or_else(|| format!("invalid git cache path {}", destination.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    let temporary = parent.join(format!(".{name}-{}.tmp", std::process::id()));
    if temporary.exists() {
        fs::remove_dir_all(&temporary)
            .map_err(|error| format!("failed to clear {}: {error}", temporary.display()))?;
    }
    let clone = Command::new("git")
        .arg("clone")
        .arg("--quiet")
        .arg("--no-checkout")
        .arg(&dependency.url)
        .arg(&temporary)
        .status()
        .map_err(|error| format!("failed to start git: {error}"))?;
    if !clone.success() {
        let _ = fs::remove_dir_all(&temporary);
        return Err(format!("failed to clone git dependency '{name}'"));
    }
    let checkout = Command::new("git")
        .arg("-C")
        .arg(&temporary)
        .arg("checkout")
        .arg("--quiet")
        .arg("--detach")
        .arg(&dependency.revision)
        .status()
        .map_err(|error| format!("failed to start git: {error}"))?;
    if !checkout.success() {
        let _ = fs::remove_dir_all(&temporary);
        return Err(format!(
            "git dependency '{name}' does not contain revision {}",
            dependency.revision
        ));
    }
    fs::remove_dir_all(temporary.join(".git"))
        .map_err(|error| format!("failed to finalize git dependency '{name}': {error}"))?;
    let content_checksum = package_checksum_root(&temporary)?;
    fs::write(
        temporary.join(".ject-git-source"),
        format!(
            "{}\n{}\n{}\n",
            dependency.url, dependency.revision, content_checksum
        ),
    )
    .map_err(|error| format!("failed to write git metadata: {error}"))?;
    fs::rename(&temporary, destination)
        .map_err(|error| format!("failed to install {}: {error}", destination.display()))
}

fn download_registry_package(
    name: &str,
    dependency: &RegistryDependency,
    destination: &Path,
) -> Result<(), String> {
    let archive_url = registry_url(&dependency.registry, name, &dependency.version, "");
    let checksum_url = registry_url(&dependency.registry, name, &dependency.version, ".sha256");
    let expected = String::from_utf8(read_registry_object(&checksum_url)?)
        .map_err(|_| format!("registry checksum for {name} is not UTF-8"))?;
    let expected = expected.split_whitespace().next().unwrap_or("");
    let bytes = read_registry_object(&archive_url)?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != expected {
        return Err(format!(
            "checksum mismatch for {name}@{}: expected {expected}, got {actual}",
            dependency.version
        ));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| format!("invalid cache path {}", destination.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    let temporary = parent.join(format!(".{}-{}.tmp", name, std::process::id()));
    if temporary.exists() {
        fs::remove_dir_all(&temporary)
            .map_err(|error| format!("failed to clear {}: {error}", temporary.display()))?;
    }
    fs::create_dir(&temporary)
        .map_err(|error| format!("failed to create {}: {error}", temporary.display()))?;
    let extraction = (|| {
        let mut archive = Archive::new(GzDecoder::new(bytes.as_slice()));
        archive
            .unpack(&temporary)
            .map_err(|error| format!("failed to unpack {archive_url}: {error}"))?;
        let content_checksum = package_checksum_root(&temporary)?;
        fs::write(
            temporary.join(".ject-source"),
            format!("{}\n{actual}\n{content_checksum}\n", dependency.registry),
        )
        .map_err(|error| format!("failed to write registry metadata: {error}"))
    })();
    if let Err(error) = extraction {
        let _ = fs::remove_dir_all(&temporary);
        return Err(error);
    }
    fs::rename(&temporary, destination)
        .map_err(|error| format!("failed to install {}: {error}", destination.display()))
}

pub fn dependency_projects(project: &Project) -> Result<Vec<Project>, String> {
    fn visit(
        project: &Project,
        seen: &mut HashSet<PathBuf>,
        active: &mut Vec<(String, PathBuf)>,
        names: &mut HashMap<String, PathBuf>,
        output: &mut Vec<Project>,
    ) -> Result<(), String> {
        let mut dependencies: Vec<_> = project.dependencies.iter().collect();
        dependencies.sort_by_key(|(name, _)| *name);
        for (declared_name, root) in dependencies {
            let canonical = root.canonicalize().map_err(|e| {
                format!("failed to resolve dependency path {}: {e}", root.display())
            })?;
            if let Some(position) = active.iter().position(|(_, path)| path == &canonical) {
                let mut chain: Vec<_> = active[position..]
                    .iter()
                    .map(|(name, _)| name.as_str())
                    .collect();
                chain.push(declared_name);
                return Err(format!("dependency cycle detected: {}", chain.join(" -> ")));
            }
            let dependency = load(&canonical)?;
            if dependency.name != *declared_name {
                return Err(format!(
                    "dependency is named '{}' but is declared as '{}' in package '{}'",
                    dependency.name, declared_name, project.name
                ));
            }
            if let Some(existing) = names.get(&dependency.name) {
                if existing != &canonical {
                    return Err(format!(
                        "package '{}' resolves to both {} and {}",
                        dependency.name,
                        existing.display(),
                        canonical.display()
                    ));
                }
            } else {
                names.insert(dependency.name.clone(), canonical.clone());
            }
            if !seen.insert(canonical.clone()) {
                continue;
            }
            active.push((dependency.name.clone(), canonical));
            visit(&dependency, seen, active, names, output)?;
            active.pop();
            output.push(dependency);
        }
        Ok(())
    }

    let mut output = Vec::new();
    let mut seen = HashSet::new();
    let root = project.root.canonicalize().map_err(|error| {
        format!(
            "failed to resolve package path {}: {error}",
            project.root.display()
        )
    })?;
    seen.insert(root.clone());
    let mut active = vec![(project.name.clone(), root.clone())];
    let mut names = HashMap::from([(project.name.clone(), root)]);
    visit(project, &mut seen, &mut active, &mut names, &mut output)?;
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
    if project.registry_source.is_some() || project.git_source.is_some() {
        command.arg("--locked");
    }
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
    fn parses_registry_dependencies_by_exact_inline_keys() {
        assert_eq!(dependency_version("\"1.2.0\""), Some("1.2.0".into()));
        assert_eq!(dependency_path("\"1.2.0\""), None);
        let value = "{ version = \"1.2.0\", registry = \"https://example.test/path/packages\" }";
        assert_eq!(dependency_version(value), Some("1.2.0".into()));
        assert_eq!(
            dependency_registry(value),
            Some("https://example.test/path/packages".into())
        );
        assert_eq!(dependency_path(value), None);
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
        assert_eq!(project.native.as_ref().unwrap().library, "hello_native");
        assert!(root.join("native/Cargo.toml").is_file());
        let facade = fs::read_to_string(root.join("src/lib.ject")).unwrap();
        assert!(facade.contains("@native/hello_native"));
        let rust = fs::read_to_string(root.join("native/src/lib.rs")).unwrap();
        assert!(rust.contains("ject_native::ject_plugin!"));
        let registry = format!("file://{}", root.join("registry").display());
        let error = publish(&project, &registry, None).unwrap_err();
        assert!(error.contains("native packages require a committed"));

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
        assert!(lock.contains("version = 2"));
        assert!(lock.contains("checksum = \""));
        verify_lockfile(&updated).unwrap();

        fs::write(library.join("src/lib.ject"), "export changed = true\n").unwrap();
        let error = verify_lockfile(&updated).unwrap_err();
        assert!(error.contains("out of date"));
        install(&updated).unwrap();
        verify_lockfile(&updated).unwrap();

        let updated = remove_dependency(&updated, "helper").unwrap();
        assert!(!updated.dependencies.contains_key("helper"));
        install(&updated).unwrap();
        let lock = fs::read_to_string(app.join(LOCK_FILE)).unwrap();
        assert!(!lock.contains("name = \"helper\""));
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn package_checksum_is_stable_and_ignores_build_output() {
        let root = std::env::temp_dir().join(format!(
            "ject-checksum-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let project = init(&root, "checksummed", true, false).unwrap();
        let original = package_checksum(&project).unwrap();
        assert_eq!(original, package_checksum(&project).unwrap());

        fs::create_dir_all(root.join("target/debug")).unwrap();
        fs::write(root.join("target/debug/artifact"), "ignored").unwrap();
        assert_eq!(original, package_checksum(&project).unwrap());

        fs::write(root.join("src/lib.ject"), "export value = 2\n").unwrap();
        assert_ne!(original, package_checksum(&project).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn publishes_and_installs_a_checksummed_registry_archive() {
        let base = std::env::temp_dir().join(format!(
            "ject-registry-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let source = base.join("source");
        let registry = base.join("registry");
        let destination = base.join("cache/demo/0.1.0");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&source).unwrap();
        let project = init(&source, "demo", true, false).unwrap();
        fs::create_dir_all(source.join("target")).unwrap();
        fs::write(source.join("target/ignored"), "build output").unwrap();
        let registry_url = format!("file://{}", registry.display());

        let checksum = publish(&project, &registry_url, None).unwrap();
        assert!(registry.join("demo/0.1.0.tar.gz").is_file());
        let index = fs::read_to_string(registry.join("demo/index.json")).unwrap();
        assert!(index.contains("0.1.0"));
        let dependency = RegistryDependency {
            version: "0.1.0".to_string(),
            requirement: None,
            registry: registry_url.clone(),
        };
        download_registry_package("demo", &dependency, &destination).unwrap();
        assert!(destination.join("src/lib.ject").is_file());
        assert!(!destination.join("target").exists());
        let installed = load(&destination).unwrap();
        assert_eq!(
            installed.registry_source.as_ref().unwrap().archive_checksum,
            checksum
        );
        let lock = render_lockfile(&installed, &[]).unwrap();
        assert!(lock.contains(&format!("source = \"registry+{registry_url}\"")));
        assert!(lock.contains(&format!("archive-checksum = \"{checksum}\"")));
        assert_eq!(publish(&project, &registry_url, None).unwrap(), checksum);
        fs::write(source.join("src/lib.ject"), "export changed = true\n").unwrap();
        let error = publish(&project, &registry_url, None).unwrap_err();
        assert!(error.contains("published versions are immutable"));

        fs::write(destination.join("src/lib.ject"), "export tampered = true\n").unwrap();
        let error = load(&destination).unwrap_err();
        assert!(error.contains("cached registry package") && error.contains("was modified"));

        fs::write(registry.join("demo/0.1.0.tar.gz.sha256"), "00\n").unwrap();
        let error = download_registry_package("demo", &dependency, &base.join("bad")).unwrap_err();
        assert!(error.contains("checksum mismatch"));
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn resolves_ranges_and_updates_exact_manifest_selections() {
        let base = std::env::temp_dir().join(format!(
            "ject-version-range-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let registry = base.join("registry");
        let registry_url = format!("file://{}", registry.display());
        let _ = fs::remove_dir_all(&base);
        for version in ["1.0.0", "1.4.0", "2.0.0"] {
            let root = base.join(format!("source-{version}"));
            fs::create_dir_all(&root).unwrap();
            init(&root, "demo", true, false).unwrap();
            let manifest = root.join(MANIFEST_FILE);
            let source = fs::read_to_string(&manifest)
                .unwrap()
                .replace("version = \"0.1.0\"", &format!("version = \"{version}\""));
            fs::write(&manifest, source).unwrap();
            publish(&load(&root).unwrap(), &registry_url, None).unwrap();
        }
        assert_eq!(
            resolve_registry_version("demo", "^1.0", &registry_url).unwrap(),
            "1.4.0"
        );
        assert_eq!(
            resolve_registry_version("demo", ">=1.0, <2.0", &registry_url).unwrap(),
            "1.4.0"
        );

        let app = base.join("app");
        fs::create_dir_all(&app).unwrap();
        init(&app, "app", false, false).unwrap();
        let manifest = app.join(MANIFEST_FILE);
        fs::write(
            &manifest,
            format!(
                "[package]\nname = \"app\"\nversion = \"0.1.0\"\n[dependencies]\ndemo = {{ version = \"1.0.0\", requirement = \"^1.0\", registry = \"{registry_url}\" }}\n"
            ),
        )
        .unwrap();
        let (updated, changes) = update_dependencies(&load(&app).unwrap(), None).unwrap();
        assert_eq!(
            changes,
            vec![DependencyUpdate {
                name: "demo".to_string(),
                previous: "1.0.0".to_string(),
                current: "1.4.0".to_string(),
            }]
        );
        assert_eq!(updated.registry_dependencies["demo"].version, "1.4.0");
        assert_eq!(
            updated.registry_dependencies["demo"].requirement.as_deref(),
            Some("^1.0")
        );
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn installs_locks_and_updates_git_dependencies() {
        let base = std::env::temp_dir().join(format!(
            "ject-git-dependency-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let repository = base.join("repository");
        let app = base.join("app");
        let cache = base.join("home");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&repository).unwrap();
        init(&repository, "demo", true, false).unwrap();
        let git = |arguments: &[&str]| {
            let status = Command::new("git")
                .arg("-C")
                .arg(&repository)
                .args(arguments)
                .status()
                .unwrap();
            assert!(status.success(), "git {arguments:?} failed");
        };
        git(&["init", "--quiet"]);
        git(&["config", "user.name", "Ject Tests"]);
        git(&["config", "user.email", "tests@ject.invalid"]);
        git(&["add", "."]);
        git(&["commit", "--quiet", "-m", "first"]);

        fs::create_dir_all(&app).unwrap();
        let project = init(&app, "app", false, false).unwrap();
        let previous_home = std::env::var_os("JECT_HOME");
        std::env::set_var("JECT_HOME", &cache);
        let result = (|| {
            let dependency =
                add_git_dependency(&project, "demo", repository.to_str().unwrap(), None)?;
            assert_eq!(
                dependency.git_dependencies["demo"].reference.as_deref(),
                Some("HEAD")
            );
            install(&dependency)?;
            let lock = fs::read_to_string(app.join(LOCK_FILE)).unwrap();
            assert!(lock.contains("source = \"git+") && lock.contains('#'));

            fs::write(repository.join("src/lib.ject"), "export value = 2\n").unwrap();
            git(&["add", "."]);
            git(&["commit", "--quiet", "-m", "second"]);
            let (updated, changes) = update_dependencies(&dependency, Some("demo"))?;
            assert_eq!(changes.len(), 1);
            assert_ne!(changes[0].previous, changes[0].current);
            install(&updated)?;
            let installed = load(&updated.dependencies["demo"])?;
            let source = installed.git_source.as_ref().unwrap();
            fs::write(
                installed.root.join(".ject-git-source"),
                format!(
                    "different-url\n{}\n{}\n",
                    source.revision, source.content_checksum
                ),
            )
            .unwrap();
            let error = install(&updated).unwrap_err();
            assert!(error.contains("has source") && error.contains("expected"));
            fs::write(
                installed.root.join(".ject-git-source"),
                format!(
                    "{}\n{}\n{}\n",
                    source.url, source.revision, source.content_checksum
                ),
            )
            .unwrap();
            fs::write(installed.root.join("src/lib.ject"), "tampered\n").unwrap();
            let error = load(&installed.root).unwrap_err();
            assert!(error.contains("cached git package") && error.contains("was modified"));
            Ok::<(), String>(())
        })();
        if let Some(home) = previous_home {
            std::env::set_var("JECT_HOME", home);
        } else {
            std::env::remove_var("JECT_HOME");
        }
        result.unwrap();
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

    #[test]
    fn rejects_dependency_cycles_from_manually_edited_manifests() {
        let base = std::env::temp_dir().join(format!(
            "ject-cycle-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let app = base.join("app");
        let helper = base.join("helper");
        fs::create_dir_all(&app).unwrap();
        fs::create_dir_all(&helper).unwrap();
        init(&app, "app", true, false).unwrap();
        init(&helper, "helper", true, false).unwrap();
        fs::write(
            app.join(MANIFEST_FILE),
            "[package]\nname = \"app\"\n[dependencies]\nhelper = { path = \"../helper\" }\n",
        )
        .unwrap();
        fs::write(
            helper.join(MANIFEST_FILE),
            "[package]\nname = \"helper\"\n[dependencies]\napp = { path = \"../app\" }\n",
        )
        .unwrap();

        let error = dependency_projects(&load(&app).unwrap()).unwrap_err();
        assert!(error.contains("dependency cycle detected: app -> helper -> app"));
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn rejects_dependency_aliases_in_manually_edited_manifests() {
        let base = std::env::temp_dir().join(format!(
            "ject-manifest-alias-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let app = base.join("app");
        let helper = base.join("helper");
        fs::create_dir_all(&app).unwrap();
        fs::create_dir_all(&helper).unwrap();
        init(&app, "app", true, false).unwrap();
        init(&helper, "helper", true, false).unwrap();
        fs::write(
            app.join(MANIFEST_FILE),
            "[package]\nname = \"app\"\n[dependencies]\nwrong = { path = \"../helper\" }\n",
        )
        .unwrap();

        let error = dependency_projects(&load(&app).unwrap()).unwrap_err();
        assert!(error.contains("named 'helper' but is declared as 'wrong'"));
        fs::remove_dir_all(base).unwrap();
    }
}
