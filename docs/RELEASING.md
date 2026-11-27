# Releasing Ject

Ship the optimized Rust frontend as the default CLI. The Ject-written
compiler remains available through `ject bootstrap` for compiler work,
verification, and saved artifacts.

A local same-build benchmark on Linux x64 measured a median of 7.43 ms
for a 13-module package with the Rust frontend, 3.43 s with the
self-hosted frontend, and 7.31 ms to run the saved artifact. Each
median uses five timed runs. The main performance gap is in the
source-to-artifact path.

## Validate a release candidate

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --release --workspace --all-targets --locked
cargo build --release --locked --bin ject
node scripts/check-performance.mjs target/release/ject
```

The GitHub release workflow also builds and tests Linux and macOS
executables for x64 and ARM64, plus x64 Windows. It installs from the
actual generated binary and exercises update, uninstall, and checksum
rejection on each runner. Run the workflow manually for an unpublished
candidate.

## Publish

Set the version in `Cargo.toml` and commit the corresponding lockfile and
release notes. Push a tag matching the executable's version, such as
`v1.0.0`. The workflow publishes the installer scripts, executables,
and SHA-256 files only after all platform jobs pass. Tags with a prerelease
suffix are published as prereleases. Users can install them by tag.

Check the published release from a clean shell by downloading and
running the installer in the [installation guide](../README.md#install-ject).
Run `ject --version`, `ject new smoke-test`, and `ject run` in the new
package. Confirm that the installed manager can select the tagged release
and uninstall it.
