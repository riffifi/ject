# Making packages and libraries in Ject

Ject packages are a simple way to split a project into reusable pieces. A package can be:

- a plain Ject library with no Rust toolchain
- a mixed library where Ject exposes the public API and Rust handles the low-level work
- an application that depends on other packages

The important idea is that the package system stays out of the way. You usually write normal Ject code, add a manifest, and let `ject` handle dependency resolution and builds.

## A package is just a folder with a manifest

The smallest package looks like this:

```text
my_app/
  Ject.toml
  src/
    main.ject
```

A library looks like this:

```text
my_lib/
  Ject.toml
  src/
    lib.ject
```

A mixed library has a native part too:

```text
image_filters/
  Ject.toml
  src/
    lib.ject
  native/
    Cargo.toml
    src/
      lib.rs
```

This is the usual layout:

- `Ject.toml` describes the package
- `src/main.ject` is the app entry point
- `src/lib.ject` is the library entry point
- `native` is optional and used for Rust-backed functionality

## The manifest

`Ject.toml` is the package file. It tells Ject the package name, version, and dependencies.

```toml
[package]
name = "image_filters"
version = "1.2.0"
edition = "2026"

[dependencies]
colors = { version = "2.4.0", requirement = "^2.1", registry = "https://packages.ject.dev" }

[native]
language = "rust"
path = "native"
abi = "ject-native-1"
library = "image_filters"
```

That is enough to describe a normal package or a mixed package. The key pieces are:

- `[package]` gives the package identity
- `[dependencies]` lists other packages it uses
- `[native]` tells Ject that this package has a native implementation

Keep the manifest clear and explicit. Avoid adding unrelated fields or using shortcut names that are easy to mistype.

## Creating a new package

The usual flow is:

```text
ject new my_app
```

For a library:

```text
ject new my_lib --lib
```

For a mixed package with native Rust support:

```text
ject new image_filters --native
```

This creates the basic structure and wiring. You can then edit the generated files and start adding your code.

## Writing a normal library

A source-only library is the easiest kind to make. It has no Rust component and no native build step.

```text
// src/lib.ject
pub fn greet(name) {
  return "hello, " + name
}
```

Then another project can import it by name:

```text
import "image_filters"

print(image_filters.greet("world"))
```

A package should expose a small, stable public API. Keep the implementation details in private modules if needed.

## Importing modules and packages

Ject has a few simple rules:

- `import "./x"` or `import "../x"` resolves relative to the current file
- `import "foo"` looks for package `foo` first
- `import "foo/bar"` imports a module inside that package

That means your package can be structured cleanly without leaking implementation details.

```text
src/
  lib.ject
  math.ject
```

Inside `lib.ject` you might do:

```text
import "./math"

pub fn add(a, b) {
  return math.sum(a, b)
}
```

In a package, only files and modules that are meant to be public should be exposed. The rest stays internal.

## Local dependencies

If you are working on a package locally, add it like this:

```text
ject add image_filters --path ../image_filters
```

That records the dependency in the manifest and refreshes the lockfile. Then run:

```text
ject install
```

This resolves the dependency graph, downloads or prepares what is needed, and builds any native pieces.

A local dependency is often the easiest way to develop multiple packages together. It keeps the workflow fast while still keeping the dependency graph clean.

## Git and registry dependencies

Packages can also come from:

- a local path
- a Git repository
- a registry

Example registry dependency:

```toml
[dependencies]
colors = { version = "2.4.0", requirement = "^2.1", registry = "https://packages.ject.dev" }
```

Git dependencies are pinned to a specific commit, so the exact source is reproducible.

```text
ject add my_dep --git https://github.com/example/my_dep --rev <commit>
```

This is especially useful when you want a known-good version rather than moving targets.

## Mixed Ject/Rust libraries

A mixed library keeps the public Ject API in `src/lib.ject` and hides the Rust implementation in `native/src/lib.rs`.

The Ject side should be the stable interface. It validates inputs, provides friendly helpers, and exposes a clean API.

```text
// src/lib.ject
pub fn blur(image, radius) {
  if radius < 0 {
    error("radius must be >= 0")
  }
  return @native/image_filters.blur(image, radius)
}
```

The Rust side is private and only does the heavy lifting. It is not the public API contract. This keeps the native code isolated and makes compatibility easier to manage.

A good rule is:

- Ject handles user-facing behavior
- native code handles the engine or OS integration
- the boundary is small and controlled

## How native code is loaded

Mixed packages declare their native configuration in the manifest:

```toml
[native]
language = "rust"
path = "native"
abi = "ject-native-1"
library = "image_filters"
```

This tells Ject how to find and build the Rust component. When the package is installed or built, Ject runs Cargo for the native part and loads the resulting artifact.

This makes mixed packages feel like regular packages to users. They just import the package and use it, without manually editing Ject internals or rebuilding the interpreter.

## Lockfiles and reproducibility

`Ject.lock` records the exact versions and sources used for a project. This is important because it makes builds reproducible.

You should commit `Ject.lock` for applications. For libraries, it is typically not required to publish, but it helps for testing and verification.

`ject install --locked` is useful in CI or release scripts because it refuses to continue if the dependency graph has drifted. It makes sure the environment matches the expected package set exactly.

## Publishing a package

When a package is ready, publish it to a registry:

```text
ject publish --registry https://packages.ject.dev
```

The registry stores immutable package versions and checksums. That means published versions cannot be silently replaced.

This helps avoid "works on my machine" problems and gives consumers a stable package source.

## Good package design

A package is easier to maintain when you follow a few rules:

- keep the public API small and clear
- hide implementation details behind a stable facade
- avoid package-specific names in the interpreter itself
- prefer one package per concern
- make native boundaries narrow
- keep imports explicit and predictable

For example, rather than building one huge package with everything in it, split things into:

- a math package
- a file I/O package
- a GUI package
- a native image-processing package

This keeps code easier to test and reuse.

## Practical workflow

A common development cycle looks like this:

```text
# create app
ject new my_app

# add a dependency from a local folder
ject add image_filters --path ../image_filters

# install and resolve deps
ject install

# run the app
ject run
```

For a library you might do:

```text
ject new my_lib --lib
# write src/lib.ject
ject build
```

For a mixed library:

```text
ject new my_native_lib --native
# implement public Ject API in src/lib.ject
# implement private Rust logic in native/src/lib.rs
ject build
ject test
```

## Tips

- Start with a source-only package if you can.
- Add native code only when you truly need it.
- Keep Ject code as the public surface.
- Version packages clearly.
- Commit lockfiles for applications.
- Validate dependency paths and package names when adding local packages.

## In one sentence

A Ject package is simply a well-defined unit of code and metadata that can be installed, shared, and reused without exposing the rest of the system to its internals.
