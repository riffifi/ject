# Ject

Ject is a small, dynamically typed language for scripts, applications, and libraries.
It pairs readable, expression-friendly syntax with one tool for running code, checking
projects, testing packages, managing dependencies, and building optional Rust extensions.

The current language version is **0.10.0**.

```ject
const greeting = "Hello"

fn greet(name, punctuation="!")
    return "$greeting, $name$punctuation"
end

print greet("Ject")
```

## Why Ject

- One `ject` binary provides the REPL, runner, checker, test runner, package manager,
  native builder, module tree, and language server.
- `let` and `const` make mutable and stable bindings explicit without static type
  annotations.
- Functions, closures, comprehensions, structs, pattern matching, and exceptions are
  built into a compact syntax.
- Source-only packages need no Rust toolchain.
- Mixed packages keep a friendly public API in Ject and put only systems work or hot
  paths behind a stable native ABI.
- Diagnostics use stable codes, source spans, notes, and actionable help.

## Install from source

Ject currently builds with Cargo:

```sh
git clone https://github.com/riffifi/ject.git
cd ject
cargo build --release
cargo install --path .
ject --version
```

Native packages additionally need a working Rust toolchain. Ordinary Ject scripts and
source-only packages do not.

## Start here

Open the REPL:

```sh
ject
```

Run a file:

```sh
ject hello.ject
```

Create and run an application:

```sh
ject new hello
cd hello
ject run
```

Create a library instead with `ject new useful --lib`. Add `--native` when the package
needs a private Rust component.

## A quick language tour

Bindings are dynamically typed. Use `let` when the binding will change and `const` when
it must not be assigned through:

```ject
let attempts = 0
attempts += 1

const MAX_ATTEMPTS = 3
const names = ["Ada", "Lin"]
```

Arrays, dictionaries, comprehensions, and method-call sugar cover common data work:

```ject
let scores = {Ada: 95, Lin: 88}
let doubled = [value * 2 for value in [1, 2, 3]]
let loud = names.map(fn(name) -> name.upper())

print scores.Ada
print doubled
print loud
```

Functions support defaults and named arguments. Required parameters come first:

```ject
fn label(name, prefix="User")
    return "$prefix: $name"
end

print label("Ada")
print label(prefix="Admin", name="Lin")
```

Blocks close with `end`:

```ject
for value in 0..5
    if value % 2 == 0
        print value
    end
end
```

Ranges are end-exclusive, so `0..5` contains `0` through `4`.

`match` is an expression. It supports literals, alternatives, comparisons, ranges,
bindings, and guards:

```ject
let score = 82
let description = match score
    >= 90 -> "excellent"
    60..90 -> "passing"
    n when n >= 0 -> "keep practicing"
    _ -> "invalid"
end
```

Use `try`, `catch`, and `throw` for recoverable failures:

```ject
fn divide(left, right)
    if right == 0
        throw "division by zero"
    end
    return left / right
end

try
    print divide(10, 0)
catch error
    print "failed: $error"
end
```

## Packages

Every package is described by `Ject.toml`:

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2026"
entry = "src/main.ject"

[dependencies]
```

Add local, registry, or Git dependencies through the same executable:

```sh
ject add helpers --path ../helpers
ject add colors --version '^2.1' --registry https://packages.example
ject add widgets --git https://github.com/example/widgets --tag v1.4.0
ject install
```

Then import the dependency by its declared package name:

```ject
import "helpers" as helpers
print helpers.greet("Ada")
```

`Ject.lock` records exact sources, versions, commits, and checksums. Use
`ject install --locked` in CI.

## Modules and libraries

A library exports its public surface from `src/lib.ject`:

```ject
export const VERSION = "1.0.0"

export fn greet(name)
    return "Hello, $name"
end
```

Consumers can use a namespace or select individual exports:

```ject
import "greeter" as greeter
import {VERSION} from "greeter"

print greeter.greet("Ada")
print VERSION
```

Relative imports such as `import "./format" as format` resolve from the importing file.

## Native libraries

A mixed package is still a normal dependency. Its Ject facade imports a private native
backend:

```ject
import "@native/image" as native

export fn resize(image, width, height)
    assert(width > 0 and height > 0, "dimensions must be positive")
    return native.resize(image, width, height)
end
```

The interpreter contains no package-specific registry for libraries such as JGUI or
JNUM. Their manifests describe native artifacts, their plugins declare exports, and the
loader discovers them through ordinary dependency resolution.

See [Packages and native libraries](docs/PACKAGES.md) for the complete authoring guide.

## Commands

| Command | Purpose |
| --- | --- |
| `ject` | Start the REPL |
| `ject file.ject` | Run one file |
| `ject new NAME [--lib\|--native]` | Create a package |
| `ject init [--lib\|--native]` | Initialize the current directory |
| `ject run` | Run the current package entry |
| `ject check` | Parse and lint every package source |
| `ject test` | Run `tests/*.ject` |
| `ject build [--release]` | Check source and build native components |
| `ject add`, `remove`, `update` | Edit and resolve dependencies |
| `ject install [--locked]` | Resolve, lock, and build the graph |
| `ject publish --registry URL` | Publish an immutable package version |
| `ject tree` | Print the resolved source-module tree |
| `ject lsp` | Run the language server over standard I/O |
| `ject --check FILE...` | Check standalone files without running them |
| `ject --test FILE...` | Run explicit test files |

Run `ject --help` for the command summary.

## Documentation

- [The Ject guide and language reference](docs/DOCS.md)
- [Packages and native libraries](docs/PACKAGES.md)
- [Runnable examples](examples/README.md)
- [Release history](CHANGELOG.md)

## Project status

Ject is usable but still pre-1.0. Language and package behavior may evolve between minor
versions. Registry archives and lockfiles are integrity-checked; native plugins execute
with the permissions of the Ject process and must be treated as trusted code.

## License

[MIT](LICENSE.md)
