# Ject packages and native extensions

## Implemented in Ject 0.9

The local mixed-package workflow is operational:

- `ject new <name> --native` scaffolds the Ject and Rust sides.
- `ject build` invokes Cargo for the package and native path dependencies.
- `ject run`, `check`, and `test` load the built platform artifacts.
- Path dependencies use `name = { path = "../name" }`.
- `ject add <name> --path <path>` validates and records a local dependency.
- `ject remove <name>` removes it, and `ject install` resolves the complete local
  graph, writes a deterministic `Ject.lock`, and builds all native components.
- `ject install --locked` verifies the exact graph and package checksums without
  changing the lockfile, making drift a hard error in CI.
- Public package source imports its backend as `@native/<package-name>`.
- The `ject-native-1` ABI discovers functions and transports ordinary values plus
  opaque typed resources with plugin-owned destruction.

Path and registry dependencies, authenticated publishing, and checksum-verified
archives are implemented. Capability grants, callbacks, and sandboxed providers are
later layers and do not change imports or the native ABI.

This document defines the target architecture for the package system. The package
manager is part of the `ject` executable; it is not a second tool.

## Goals

- A normal library is Ject source and needs no Rust toolchain.
- A mixed library presents one Ject API while hiding its native implementation.
- Installing a third-party native library never requires editing or rebuilding Ject.
- Resolution is reproducible from a lockfile and safe by default.
- Built-in modules (`jnum` and `jgui`) use the same runtime call boundary as installed
  native modules, even though they ship with the interpreter.

## Project layout

```text
my_app/
  Ject.toml
  Ject.lock
  src/main.ject
  tests/*.ject
```

A library uses `src/lib.ject`. A mixed library additionally contains a native
component, initially Rust:

```text
image_filters/
  Ject.toml
  src/lib.ject          # public Ject facade
  native/Cargo.toml
  native/src/lib.rs     # private native implementation
```

Example manifest:

```toml
[package]
name = "image_filters"
version = "1.2.0"
edition = "2026"

[dependencies]
colors = "^2.1"

[native]
language = "rust"
path = "native"
abi = "ject-native-1"
library = "image_filters"
```

The manifest records requested versions. The version 2 `Ject.lock` records exact
versions, canonical sources, SHA-256 checksums, and whether a package has a native
component. Applications commit the lockfile; published libraries normally do not.
Build output and VCS metadata are excluded from the checksum.

## Commands

The intended command surface is:

```text
ject new <name> [--lib]
ject init [--lib]
ject run [-- <args>]
ject check
ject test
ject add <package> --path <directory>
ject add <package> --version <exact-version> [--registry <url>]
ject remove <package>
ject install
ject install --locked
ject build [--release]
ject publish [--registry <url>]
```

The existing `ject file.ject`, `--check`, and `--test` forms remain valid. Registry
versions are exact in 0.9; version-range solving can be added later without changing
the archive protocol or manifest shape.

### Registry protocol

A registry is an HTTP(S) base URL. Packages are immutable gzip-compressed tar
archives at `<base>/<name>/<version>.tar.gz`; the adjacent `.tar.gz.sha256` object
contains the archive digest. Publishing uses conditional HTTP PUT and optionally
sends `JECT_REGISTRY_TOKEN` as a bearer token. `JECT_REGISTRY` selects the default
base URL. A `file://` base implements the same layout for private or local registries.

Published archives exclude `.git`, `target`, `Ject.lock`, and Ject's internal source
metadata. Installation verifies the archive before extraction, records provenance in
the cache, and normal execution uses that cache without downloading during imports.
Published versions cannot be overwritten.

### Installing a local library

From the application package:

```text
ject add image_filters --path ../image_filters
ject install
ject run
```

`add` checks that the target has a valid `Ject.toml` and that its declared package
name is exactly the requested import name. It updates `[dependencies]` and refreshes
`Ject.lock`. `install` resolves transitive dependencies and builds Rust components;
source-only libraries do not require Cargo. `install --locked` performs the same
native build only after the lockfile matches all manifests and package contents.
Commit `Ject.lock` for applications.

## Import and resolution rules

- `import "./x"` and `import "../x"` resolve relative to the importing file.
- `import "foo"` first resolves package `foo`, then the standard library.
- `import "foo/bar"` resolves an exported module inside package `foo`.
- Package source is immutable in a global content-addressed cache. A project-local
  lockfile selects the exact cached package.
- A package may only expose paths declared in its manifest. Native implementation
  modules are private unless explicitly exported.

The package resolver must be separate from the interpreter. It produces a module
graph (canonical package ID plus canonical source/native location); the interpreter
only loads that graph. This removes filesystem and built-in-name policy from
`Interpreter::load_module`.

## Regular libraries

A source-only library exports values from `src/lib.ject`. Its dependencies are
resolved before execution. The module is evaluated once per interpreter and its
exports are cached by canonical package ID, not by the spelling of an import.

## Mixed Ject/native libraries

The public entry point is still `src/lib.ject`. It imports a private native module,
validates or reshapes friendly Ject values, and exports the stable public API. This
keeps native code small and allows most compatibility changes in Ject source.

Native code must not expose Rust trait objects or Rust-owned layouts across a dynamic
library boundary: Rust has no stable ABI. The implemented boundary is the versioned
`ject-native-1` C descriptor ABI. It uses serialized values and opaque handles, never
`Value`, `String`, `Vec`, or `dyn Trait` directly. A future WebAssembly component
provider can offer isolation and portable artifacts without changing public facades.

The v1 value interface should support nil, bool, integer, float, UTF-8 string, bytes,
arrays, dictionaries, errors, and opaque resource handles. Native resources carry
the owning module ID, type name, and handle. Calls on a resource are always routed
back to its owner, so the core `Value` enum stays independent of `jnum`, `jgui`, and
future plugins.

### Bundled libraries migrating to packages

JGUI and JNUM should follow the same rule as third-party mixed libraries. Their
user-facing constructors, validation, defaults, and convenience operations belong
in `src/lib.ject`. Only rendering and operating-system integration for JGUI, and
array storage and numerical kernels for JNUM, belong in Rust.

An explicitly installed native package may replace a bundled compatibility backend
with the same module name. This allows JGUI and JNUM to be upgraded as packages
without rebuilding the Ject executable. The standalone JGUI proof at
`/home/leo/dev/ject/packages/jgui` uses a deliberately small native boundary: Ject
builds a declarative widget document and Rust exports only `run(document)`.

The bundled backends remain temporarily for scripts that declare no dependencies.
JNUM should be extracted through the same mechanism, while retaining opaque native
array resources so ordinary operations do not serialize entire arrays on every call.

## Runtime architecture

```text
CLI / project discovery
        -> dependency resolver + Ject.lock
        -> module graph
        -> module loader
             -> Ject source module
             -> native component host
        -> interpreter
```

`NativeRegistry` is a provider for built-in and dynamically discovered ABI modules;
it contains no package function names. The next structural refactor should wrap all
source, standard-library, and native resolution in explicit `ModuleProvider` values
before adding git, registry, or WebAssembly providers.

## Security and reproducibility

- Source packages run with the program's normal Ject permissions.
- Native components receive no filesystem, network, environment, clock, process, or
  GUI capability unless the package declares it and the application grants it.
- Lockfile checksums are verified before loading.
- Native artifacts are keyed by package checksum, ABI version, OS, architecture, and
  debug/release profile.
- Installation never executes arbitrary build scripts silently; native builds and
  requested capabilities are shown explicitly.

## Delivery plan

1. Complete: mixed facades, private native imports, local manifests, path dependencies,
   Rust SDK, dynamic ABI, generic resources, scaffolding, and native build graphs.
2. Introduce explicit `ModuleLoader`/`ModuleProvider` types around the now-generic
   resolution paths.
3. Add deterministic git/registry resolution and `Ject.lock`.
4. Add capability declarations/enforcement and callback/event handles.
5. Add a WebAssembly component provider for portable and untrusted plugins.
6. Add registry download/publish, checksums, and cache management.
