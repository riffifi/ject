use ject::module_artifact::{self, Artifact};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output, Stdio};
use std::{env, io};

const BUNDLED_COMPILER: &str = "ject-compiler-v1.json";

const USAGE: &str = "Usage:
  ject bootstrap install [COMPILER_CHECKOUT]
  ject bootstrap install-cli BIN_DIR
  ject bootstrap run PATH [ARGS...]
  ject bootstrap run [-- ARGS...]
  ject bootstrap check [PATH]
  ject bootstrap build [PATH] [-o ARTIFACT]

PATH defaults to the current package. It can be a script or package directory.
Build writes target/ject/program.json by default.";

pub(super) fn run(args: &[String]) {
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h") {
        println!("{USAGE}");
        return;
    }
    finish(dispatch(&args[0], &args[1..]));
}

pub(super) fn finish(result: Result<bool, String>) {
    if let Err(error) = &result {
        super::emit_cli_error("E4201", error, Some("try `ject bootstrap --help`"));
    }
    if !matches!(result, Ok(true)) {
        process::exit(1);
    }
}

fn dispatch(command: &str, args: &[String]) -> Result<bool, String> {
    if command == "install" {
        return match args {
            [] => install(Path::new(".")),
            [checkout] => install(Path::new(checkout)),
            _ => Err("install accepts one compiler checkout path".into()),
        }
        .map(|()| true);
    }
    if command == "install-cli" {
        return match args {
            [directory] => install_cli(Path::new(directory)).map(|()| true),
            _ => Err("install-cli requires a new bin directory".into()),
        };
    }
    let (options, script_args) = if command == "run" {
        match args {
            [flag, rest @ ..] if flag == "--" => (&[][..], rest),
            [_, rest @ ..] => (&args[..1], super::script_arguments(rest)),
            [] => (args, args),
        }
    } else {
        (args, &[][..])
    };
    let (input, destination) = match (command, options) {
        ("build", [path, flag, output]) if flag == "-o" => (path.as_str(), Some(output.as_str())),
        ("build", [flag, output]) if flag == "-o" => (".", Some(output.as_str())),
        ("build", [path]) if !path.starts_with('-') => {
            (path.as_str(), Some("target/ject/program.json"))
        }
        ("build", []) => (".", Some("target/ject/program.json")),
        ("run" | "check", [path]) if !path.starts_with('-') => (path.as_str(), None),
        ("run" | "check", []) => (".", None),
        _ => {
            return Err(format!(
                "invalid arguments for bootstrap command {command:?}"
            ))
        }
    };
    let compiler = compiler_path()?;
    use_compiler(
        &compiler,
        Path::new(input),
        (command == "run").then_some(script_args),
        destination.map(Path::new),
    )
}

pub(super) fn bundled_compiler() -> Option<PathBuf> {
    if env::var_os("JECT_BOOTSTRAP_SEED").is_some() {
        return None;
    }
    let compiler = env::current_exe().ok()?.with_file_name(BUNDLED_COMPILER);
    compiler.is_file().then_some(compiler)
}

pub(super) fn use_compiler(
    compiler: &Path,
    input: &Path,
    script_args: Option<&[String]>,
    destination: Option<&Path>,
) -> Result<bool, String> {
    if !compiler.is_file() {
        return Err(format!(
            "compiler missing at {}. Run `ject bootstrap install PATH_TO_BOOTSTRAP_CHECKOUT`",
            compiler.display()
        ));
    }
    let result = compile(compiler, input, true)?;
    if !result.status.success() {
        return Ok(false);
    }
    let bytes = result.stdout;
    let artifact = linked_artifact(&bytes)?;
    if let Some(destination) = destination {
        write_artifact(destination, &bytes)
            .map_err(|error| format!("cannot write {}: {error}", destination.display()))?;
        println!("Built {}", destination.display());
    } else if let Some(script_args) = script_args {
        let location = input
            .canonicalize()
            .map_err(|error| format!("cannot open {}: {error}", input.display()))?;
        let entry = if location.is_dir() {
            ject::package::discover(&location)?.entry
        } else {
            location.clone()
        };
        let native_project = match &artifact {
            Artifact::Program(program) if program.requires_native_project() => {
                Some(location.as_path())
            }
            _ => None,
        };
        return super::execute_artifact(artifact, &entry, native_project, script_args);
    }
    Ok(true)
}

fn compiler_path() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("JECT_BOOTSTRAP_COMPILER").filter(|path| !path.is_empty()) {
        return Ok(path.into());
    }
    if let Some(compiler) = bundled_compiler() {
        return Ok(compiler);
    }
    let data = env::var_os("XDG_DATA_HOME")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var_os("LOCALAPPDATA").map(PathBuf::from))
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")))
        .ok_or("cannot locate a data directory. Set JECT_BOOTSTRAP_COMPILER")?;
    Ok(data.join("ject").join("compiler-v1.json"))
}

fn install_cli(directory: &Path) -> Result<(), String> {
    let compiler = compiler_path()?;
    let bytes = fs::read(&compiler)
        .map_err(|error| format!("cannot read {}: {error}", compiler.display()))?;
    linked_artifact(&bytes)?;
    let executable = env::current_exe().map_err(|error| error.to_string())?;
    let name = format!("ject{}", env::consts::EXE_SUFFIX);
    if directory.exists() {
        return Err(format!(
            "bin directory already exists: {}",
            directory.display()
        ));
    }
    let parent = artifact_directory(directory);
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let staged = parent.join(format!(".ject-cli-{}", process::id()));
    fs::create_dir(&staged).map_err(|error| error.to_string())?;
    let result = (|| -> io::Result<()> {
        fs::copy(executable, staged.join(&name))?;
        fs::write(staged.join(BUNDLED_COMPILER), bytes)?;
        fs::rename(&staged, directory)
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(staged.join(&name));
        let _ = fs::remove_file(staged.join(BUNDLED_COMPILER));
        let _ = fs::remove_dir(staged);
        return Err(format!(
            "cannot install CLI at {}: {error}",
            directory.display()
        ));
    }
    println!("Installed {}", directory.join(name).display());
    Ok(())
}

fn install(checkout: &Path) -> Result<(), String> {
    let destination = compiler_path()?;
    let source = checkout.join("compile.ject");
    let source = source
        .canonicalize()
        .map_err(|error| format!("cannot open compiler source {}: {error}", source.display()))?;
    eprintln!("Building stage 1");
    let stage1 = compiler_stdout(compile(&source, &source, false)?);
    linked_artifact(&stage1)?;
    let parent = artifact_directory(&destination);
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let staged = parent.join(format!(".ject-stage1-{}.json", process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staged)
        .map_err(|error| format!("cannot stage compiler at {}: {error}", staged.display()))?;
    let result = file.write_all(&stage1);
    drop(file);
    let stage2 = match result {
        Ok(()) => {
            eprintln!("Building stage 2");
            compile(&staged, &source, true)
        }
        Err(error) => Err(error.to_string()),
    };
    let _ = fs::remove_file(&staged);
    let stage2 = compiler_stdout(stage2?);
    if stage2 != stage1 {
        return Err("stage 1 and stage 2 produced different compiler artifacts".into());
    }
    write_artifact(&destination, &stage2)
        .map_err(|error| format!("cannot install {}: {error}", destination.display()))?;
    println!("Installed {}", destination.display());
    Ok(())
}

fn compile(compiler: &Path, input: &Path, is_artifact: bool) -> Result<Output, String> {
    let executable = env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new(executable);
    if is_artifact {
        command.arg("run-artifact");
    } else {
        command.env("JECT_BOOTSTRAP_SEED", "1");
    }
    command
        .arg(compiler)
        .arg("--project")
        .arg(input)
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|error| format!("cannot start compiler: {error}"))
}

fn compiler_stdout(output: Output) -> Vec<u8> {
    if !output.status.success() {
        process::exit(output.status.code().unwrap_or(1));
    }
    output.stdout
}

fn linked_artifact(bytes: &[u8]) -> Result<Artifact, String> {
    match module_artifact::decode(bytes)? {
        artifact @ Artifact::Program(_) => Ok(artifact),
        Artifact::Module(_) => {
            Err("compiler returned a single module instead of a linked program".into())
        }
    }
}

fn artifact_directory(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn write_artifact(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let directory = artifact_directory(path);
    fs::create_dir_all(directory)?;
    let staged = directory.join(format!(".ject-output-{}.tmp", process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staged)?;
    let written = file.write_all(bytes);
    drop(file);
    let result = written.and_then(|()| fs::rename(&staged, path));
    if result.is_err() {
        let _ = fs::remove_file(staged);
    }
    result
}
