//! Canonical module discovery shared by execution, diagnostics, and editor tooling.

use crate::package::{self, Project};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ResolveError {
    NotFound {
        specifier: String,
        searched: Option<PathBuf>,
    },
    Io {
        specifier: String,
        path: PathBuf,
        source: std::io::Error,
    },
    HomeUnavailable,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound {
                specifier,
                searched: Some(path),
            } => write!(
                formatter,
                "module '{specifier}' not found at {}",
                path.display()
            ),
            Self::NotFound {
                specifier,
                searched: None,
            } => write!(
                formatter,
                "module '{specifier}' not found as a package, path, or standard module"
            ),
            Self::Io {
                specifier,
                path,
                source,
            } => write!(
                formatter,
                "failed to load module '{specifier}' from {}: {source}",
                path.display()
            ),
            Self::HomeUnavailable => write!(formatter, "cannot resolve '~': HOME is not set"),
        }
    }
}

impl std::error::Error for ResolveError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleIdentity {
    File(PathBuf),
    Embedded(String),
}

impl ModuleIdentity {
    pub fn cache_key(&self) -> String {
        match self {
            Self::File(path) => path.to_string_lossy().into_owned(),
            Self::Embedded(name) => format!("<embedded:{name}>"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedModule {
    pub identity: ModuleIdentity,
    pub source: String,
    pub directory: PathBuf,
    pub package: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModuleResolver {
    base: PathBuf,
    project: Option<Project>,
    packages: HashMap<String, Project>,
    stdlib_roots: Vec<PathBuf>,
}

impl ModuleResolver {
    pub fn for_path(path: &Path) -> Self {
        let base = if path.is_file() {
            path.parent().unwrap_or(path).to_path_buf()
        } else {
            path.to_path_buf()
        };
        let project = package::discover(path).ok();
        let mut packages = HashMap::new();
        if let Some(root) = &project {
            for (alias, dependency_root) in &root.dependencies {
                if let Ok(dependency) = package::load(dependency_root) {
                    packages.insert(alias.clone(), dependency);
                }
            }
            if let Ok(dependencies) = package::dependency_projects(root) {
                for dependency in dependencies {
                    packages
                        .entry(dependency.name.clone())
                        .or_insert(dependency);
                }
            }
        }
        let mut stdlib_roots = Vec::new();
        if let Some(project) = &project {
            stdlib_roots.push(project.root.join("stdlib"));
        }
        stdlib_roots.push(base.join("stdlib"));
        stdlib_roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("stdlib"));
        // Preserve lookup precedence: a project's standard modules may
        // intentionally override the installation fallback.
        let mut unique_roots = Vec::new();
        for root in stdlib_roots {
            if !unique_roots.contains(&root) {
                unique_roots.push(root);
            }
        }
        Self {
            base,
            project,
            packages,
            stdlib_roots: unique_roots,
        }
    }

    pub fn with_base(&self, base: &Path) -> Self {
        let mut resolver = self.clone();
        resolver.base = base.to_path_buf();
        resolver
    }

    pub fn current_package(&self) -> Option<&str> {
        self.project.as_ref().map(|project| project.name.as_str())
    }

    pub fn package_entry(&self, name: &str) -> Option<&Path> {
        self.packages
            .get(name)
            .map(|project| project.entry.as_path())
    }

    pub fn resolve(&self, specifier: &str) -> Result<ResolvedModule, ResolveError> {
        if let Some(package) = self.packages.get(specifier) {
            return resolve_file(&package.entry, Some(package.name.clone()), specifier);
        }

        if let Some(path) = self.explicit_path(specifier)? {
            return resolve_file(&path, None, specifier);
        }

        let module_name = specifier.trim_end_matches(".ject");
        for root in &self.stdlib_roots {
            let candidate = root.join(format!("{module_name}.ject"));
            if candidate.is_file() {
                return resolve_file(&candidate, None, specifier);
            }
        }
        if let Some(source) = crate::stdlib::embedded_stdlib_module_source(module_name) {
            return Ok(ResolvedModule {
                identity: ModuleIdentity::Embedded(module_name.to_string()),
                source: source.to_string(),
                directory: self.base.clone(),
                package: None,
            });
        }
        Err(ResolveError::NotFound {
            specifier: specifier.to_string(),
            searched: None,
        })
    }

    fn explicit_path(&self, specifier: &str) -> Result<Option<PathBuf>, ResolveError> {
        let looks_like_path = specifier.starts_with('~')
            || specifier.starts_with('.')
            || specifier.starts_with('/')
            || specifier.contains('/');
        if !looks_like_path {
            return Ok(None);
        }
        let with_extension = |path: PathBuf| {
            if path.extension().is_some() {
                path
            } else {
                path.with_extension("ject")
            }
        };
        let candidate = if let Some(rest) = specifier.strip_prefix("~/") {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .ok_or(ResolveError::HomeUnavailable)?;
            with_extension(home.join(rest))
        } else if Path::new(specifier).is_absolute() {
            with_extension(PathBuf::from(specifier))
        } else if specifier.starts_with("./") || specifier.starts_with("../") {
            with_extension(self.base.join(specifier))
        } else {
            let relative = with_extension(PathBuf::from(specifier));
            let project_candidate = self
                .project
                .as_ref()
                .map(|project| project.root.join(&relative));
            project_candidate
                .filter(|path| path.is_file())
                .unwrap_or_else(|| self.base.join(relative))
        };
        if candidate.is_file() {
            Ok(Some(candidate))
        } else {
            Err(ResolveError::NotFound {
                specifier: specifier.to_string(),
                searched: Some(candidate),
            })
        }
    }
}

fn resolve_file(
    path: &Path,
    package: Option<String>,
    specifier: &str,
) -> Result<ResolvedModule, ResolveError> {
    let canonical = path.canonicalize().map_err(|source| ResolveError::Io {
        specifier: specifier.to_string(),
        path: path.to_path_buf(),
        source,
    })?;
    let source = fs::read_to_string(&canonical).map_err(|source| ResolveError::Io {
        specifier: specifier.to_string(),
        path: canonical.clone(),
        source,
    })?;
    Ok(ResolvedModule {
        directory: canonical.parent().unwrap_or(&canonical).to_path_buf(),
        identity: ModuleIdentity::File(canonical),
        source,
        package,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "ject-resolver-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }

    #[test]
    fn resolves_relative_files_and_path_dependencies() {
        let root = temporary("packages");
        let app = root.join("app");
        let helper = root.join("helper");
        fs::create_dir_all(app.join("src")).unwrap();
        fs::create_dir_all(helper.join("src")).unwrap();
        fs::write(
            app.join("Ject.toml"),
            "[package]\nname = \"app\"\nentry = \"src/main.ject\"\n[dependencies]\nhelper = { path = \"../helper\" }\n",
        )
        .unwrap();
        fs::write(
            helper.join("Ject.toml"),
            "[package]\nname = \"helper\"\nentry = \"src/lib.ject\"\n",
        )
        .unwrap();
        fs::write(app.join("src/main.ject"), "import \"helper\"").unwrap();
        fs::write(app.join("src/local.ject"), "export value = 1").unwrap();
        fs::write(helper.join("src/lib.ject"), "export value = 2").unwrap();

        let resolver = ModuleResolver::for_path(&app.join("src/main.ject"));
        assert_eq!(resolver.current_package(), Some("app"));
        assert!(matches!(
            resolver.resolve("helper").unwrap().identity,
            ModuleIdentity::File(path) if path.ends_with("helper/src/lib.ject")
        ));
        assert!(matches!(
            resolver.resolve("./local").unwrap().identity,
            ModuleIdentity::File(path) if path.ends_with("app/src/local.ject")
        ));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolves_standard_modules_with_stable_identity() {
        let resolver = ModuleResolver::for_path(Path::new(env!("CARGO_MANIFEST_DIR")));
        let module = resolver.resolve("color").unwrap();
        assert!(!module.source.is_empty());
        assert!(matches!(
            module.identity,
            ModuleIdentity::File(_) | ModuleIdentity::Embedded(_)
        ));
    }

    #[test]
    fn project_standard_modules_override_the_installation_fallback() {
        let root = temporary("stdlib-precedence");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("stdlib")).unwrap();
        fs::write(
            root.join("Ject.toml"),
            "[package]\nname = \"app\"\nentry = \"src/main.ject\"\n",
        )
        .unwrap();
        fs::write(root.join("src/main.ject"), "import \"color\"").unwrap();
        fs::write(root.join("stdlib/color.ject"), "export local = true").unwrap();

        let module = ModuleResolver::for_path(&root.join("src/main.ject"))
            .resolve("color")
            .unwrap();
        assert_eq!(module.source, "export local = true");
        assert!(matches!(
            module.identity,
            ModuleIdentity::File(path) if path.ends_with("stdlib/color.ject")
        ));
        fs::remove_dir_all(root).unwrap();
    }
}
