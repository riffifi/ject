# Compatibility and release policy

Ject is pre-1.0. This document defines what users can rely on now and what a
version number means while the language is still settling.

## Version promises

Ject follows semantic versioning with a stricter patch-release rule:

- `0.MINOR.0` may change language syntax, runtime behavior, standard-library
  APIs, commands, manifests, lockfiles, or native interfaces. Breaking changes
  must be called out in the changelog with a practical migration.
- `0.MINOR.PATCH` must remain source- and package-compatible with that minor
  release. Patch releases may fix behavior that was unsafe, security-sensitive,
  undocumented, or impossible to use as documented.
- Deprecations should remain available for at least one minor release when that
  is practical. Security fixes and removal of behavior that never worked may be
  immediate and must be explained in the changelog.

Packages use ordinary semantic-version requirements. Applications should commit
`Ject.lock`; libraries should commit it when reproducible tests or native builds
matter. `ject install --locked` is the release and CI check for an existing lock.

## Native libraries

The native ABI is versioned separately from the language. A mixed package must
declare its ABI in `[native]`, such as `ject-native-1` or `ject-native-2`. An ABI
identifier keeps its wire format and entry-point contract for as long as that
identifier is supported. New callback or value capabilities require a new ABI
identifier rather than silently changing an old one.

Native libraries are target-specific and run with the full permissions of the
Ject process. ABI compatibility is not a sandbox or a promise that one compiled
artifact works across operating systems, CPU architectures, or Rust targets.

## Supported environments

Every change is expected to pass stable Rust on Linux, macOS, and Windows. The CI
beta lane is advisory and gives early warning of upcoming Rust changes. Ject does
not currently declare a Rust MSRV; building from source requires the stable Rust
toolchain exercised by CI. Installed Ject binaries do not require Rust unless a
project builds a Rust native package.

Display-backed JGUI behavior additionally requires a working window system.
Headless validation and package tests remain supported without one.

## Release checklist

A release is not complete until all of these agree:

1. `Cargo.toml`, `ject --version`, the README, both user guides, and the changelog
   name the same language version.
2. Breaking changes have migration notes and removed compatibility aliases are
   listed explicitly.
3. Format, strict Clippy, all targets, package tests, installed-CLI smokes, and
   native dependency smokes pass from a clean checkout.
4. Native ABI changes include SDK tests and clean rebuilds of bundled mixed
   packages.
5. The release tag is created only after the cross-platform workflow succeeds.

The goal for 1.0 is to make minor releases backward-compatible as well. Until
then, the rules above favor honest, documented evolution over accidental promises.
