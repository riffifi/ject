# Compatibility and release policy

This policy defines the compatibility contract for Ject 1.0 and later.

## Version promises

Ject follows semantic versioning:

- Minor releases may add language, library, and tooling features while
  preserving documented behavior for existing code and manifests.
- Patch releases preserve compatibility and fix incorrect behavior.
- Breaking changes to documented syntax, public standard-library APIs,
  CLI contracts, and package formats require a major release and a migration
  note. Experimental compiler artifacts follow their own format version.
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
3. Format, strict Clippy, all targets in debug and release profiles, package
   tests, installed-CLI smokes, and native dependency smokes pass from a clean
   checkout. Optimized runtime tests must preserve the same language behavior
   and error handling as debug builds.
4. Native ABI changes include SDK tests and clean rebuilds of bundled mixed
   packages.
5. The release tag is created only after the cross-platform workflow succeeds.

When migrating from pre-1.0, run `ject install` to regenerate a portable
`Ject.lock` and commit it. Version 2 lockfiles can still be verified where their
original path and checksum data match.

### Printing

Use `print("one", "two", sep=", ", end="\n")` for new code. Parenthesized
printing uses ordinary function calls, so `let emit = print` and
`emit("ready")` also work. Legacy statements such as `print "ready"` still
parse in statement position.

### Self-hosted compiler

The compiler written in Ject lexes, parses, checks, and links source into
artifacts that execute on the Rust host. The verified bootstrap builds
stage 1 from a Rust seed and rebuilds the same source with stage 1 to produce
a byte-identical stage 2. Its verification covers compiler source relocation,
lexer and parser parity, diagnostics, and program execution.

The standard distribution uses the Rust frontend for immediate script and
project execution. The self-hosted compiler is available through
`ject bootstrap` for development and artifact builds. See
[Releasing Ject](RELEASING.md) for the performance comparison.

The bootstrap lexer uses [Langkit](https://github.com/riffifi/ject-langkit).
The host exposes
`unicode_alphabetic(text)` and `unicode_alphanumeric(text)` so language tools
can share the Rust host's Unicode character classification. Both require
nonempty text for a positive result and classify every scalar in the input.
