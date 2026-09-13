# Packages and native libraries

This guide explains how Ject 0.10.0 packages are created, resolved, locked, tested,
published, and optionally backed by Rust. The package system is part of the `ject`
binary; there is no separate package-manager command.

## 1. Package kinds

A Ject package is one directory with `Ject.toml`. It can be:

- an application with an executable entry file;
- a source library exporting Ject declarations;
- a mixed library with a public Ject facade and a private native plugin.

Applications and libraries participate in the same dependency graph. “Mixed” describes
an implementation detail, not a different kind of dependency.

## 2. Create a package

```sh
ject new notebook
ject new text_tools --lib
ject new audio_engine --native
```

Use `ject init`, with the same optional flags, to initialize the current directory.

The standard layouts are:

```text
notebook/
├── Ject.toml
└── src/
    └── main.ject

text_tools/
├── Ject.toml
├── src/
│   └── lib.ject
└── tests/
    └── text.ject

audio_engine/
├── Ject.toml
├── src/
│   └── lib.ject
├── tests/
│   └── engine.ject
└── native/
    ├── Cargo.lock
    ├── Cargo.toml
    ├── ject-native/
    └── src/
        └── lib.rs
```

The generated native package vendors the small `ject-native` SDK under `native/`.
That makes a published package independent of the author's Ject checkout.

## 3. `Ject.toml`

A source package needs only `[package]` and `[dependencies]`:

```toml
[package]
name = "text_tools"
version = "1.2.0"
edition = "2026"
entry = "src/lib.ject"

[dependencies]
```

| Field | Meaning |
| --- | --- |
| `name` | Package identity used by imports and dependency keys |
| `version` | Semantic version; defaults to `0.0.0` |
| `edition` | Declared source edition; generated projects use `2026` |
| `entry` | Source entry; defaults to `src/main.ject` |

Names may contain ASCII letters, digits, `-`, and `_`. A dependency key must match the
dependency package's declared name; aliases that conceal a different package name are
rejected.

Mixed packages add:

```toml
[native]
language = "rust"
path = "native"
abi = "ject-native-2"
library = "audio_engine"
```

`path` locates the Cargo project. `library` is the dynamic-library stem and usually
replaces package-name hyphens with underscores. Use ABI v1 for ordinary calls and opaque
resources. Use ABI v2 only when Rust must invoke a Ject callback during a native call.

## 4. Source libraries

The entry file defines the public API explicitly:

```ject
export const VERSION = "1.2.0"

fn normalize_name(name)
    return name.trim().lower()
end

export fn greeting(name, prefix="Hello")
    return "$prefix, ${normalize_name(name)}"
end
```

Unexported declarations remain private. Public constants should use `export const`.
Use `export let` only when the defining module genuinely maintains mutable state.

A consumer declares the dependency and imports its package name:

```ject
import "text_tools" as text
print text.greeting(" ADA ")
```

The package entry is the root module. A path such as `text_tools/format` resolves a
module under that package's `src/` tree.

## 5. Add dependencies

Prefer `ject add` over hand-editing the manifest. It validates package identity,
updates `Ject.toml`, resolves the complete graph, and refreshes `Ject.lock`.

### Local path

```sh
ject add text_tools --path ../text_tools
```

The manifest entry is:

```toml
[dependencies]
text_tools = { path = "../text_tools" }
```

Paths are resolved from the package root. Ject rejects self-dependencies, duplicate
package names resolving to different directories, and dependency cycles.

### Registry

```sh
ject add colors --version '^2.1' --registry https://packages.example
```

Ject resolves the newest published version matching the requirement and records both
the selected version and the continuing requirement:

```toml
[dependencies]
colors = { version = "2.4.0", requirement = "^2.1", registry = "https://packages.example" }
```

An exact version can also be written as a string:

```toml
[dependencies]
colors = "2.4.0"
```

The default registry is `https://packages.ject.dev`. Override it with `--registry` or
the `JECT_REGISTRY` environment variable.

### Git

Pin a commit directly:

```sh
ject add widgets --git https://github.com/example/widgets --rev 0123456789abcdef0123456789abcdef01234567
```

Or track a branch or tag:

```sh
ject add widgets --git https://github.com/example/widgets --branch main
ject add widgets --git https://github.com/example/widgets --tag v1.4.0
```

The manifest always stores the resolved 40-character commit. A tracked branch, tag, or
HEAD is stored alongside it so `ject update` knows what may move.

## 6. Install, lock, and update

```sh
ject install
```

Installation materializes registry and Git sources in the package cache, validates
every manifest, resolves transitive dependencies, verifies content, writes a
deterministic `Ject.lock`, and builds native components in dependency order.

The lockfile records each package's exact version, source, checksum, and native status.
Commit it for applications and CI. Do not edit it by hand.

```sh
ject install --locked
```

Locked installation recomputes the graph and fails if it differs from the file. It does
not repair or rewrite the lockfile.

Update every movable registry/Git dependency, or one selected package:

```sh
ject update
ject update colors
```

Exact registry selections and commit-only Git dependencies do not move. Path
dependencies cannot be updated because their source is already the local directory.

Remove a dependency with:

```sh
ject remove colors
```

## 7. Build and test

```sh
ject check
ject test
ject build
ject build --release
```

`check` covers every `.ject` file under `src/` plus an external configured entry. It
does not execute code. `test` runs top-level `.ject` files in `tests/` in sorted order;
use `assert` and let any failure produce a non-zero exit. `build` checks source and runs
Cargo for native components.

Registry and Git native packages build with Cargo's `--locked` flag. Commit
`native/Cargo.lock` before publishing a mixed package.

## 8. Design a mixed library

The public API belongs in Ject. Rust is a private provider.

```ject
import "@native/audio_engine" as native

export fn render(samples, gain=1.0)
    assert(type_of(samples) == "array", "samples must be an array")
    assert(gain >= 0, "gain must not be negative")
    return native.render(samples, gain)
end
```

Application code imports `audio_engine`, never `@native/audio_engine`. The special
native path is available only while that package's facade is executing, so consumers
cannot couple themselves to private exports.

Keep these responsibilities on the Ject side:

- defaults and ergonomic overload-like helpers;
- user-facing validation and error wording;
- composition of several primitive native calls;
- stable names and backward-compatible wrappers.

Keep these responsibilities on the Rust side:

- operating-system and hardware integration;
- ownership of opaque external resources;
- computation where native performance materially matters;
- conversion at the ABI boundary.

This split is why JGUI and JNUM can evolve as packages without adding their names or
APIs to the interpreter.

## 9. Implement an ABI v1 plugin

The native crate is a `cdylib` and depends on the vendored SDK and `serde_json`:

```toml
[package]
name = "audio_engine_native"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
ject-native = { path = "ject-native" }
serde_json = "1.0"
```

Each call receives an export name and JSON-compatible arguments. Return a value or a
plain error string:

```rust
use serde_json::{json, Value};

fn call(function: &str, args: Vec<Value>) -> Result<Value, String> {
    match function {
        "double" => {
            let value = args
                .first()
                .and_then(Value::as_i64)
                .ok_or("double expects an integer")?;
            Ok(json!(value * 2))
        }
        _ => Err(format!("unknown function '{function}'")),
    }
}

ject_native::ject_plugin!("audio_engine", ["double"], call);
```

The macro publishes the stable entry symbol and export catalog. No Rust-owned layout,
string, vector, or error crosses the dynamic-library boundary directly; the SDK owns
encoding, result envelopes, panic containment, and buffer release.
If you use the ABI directly, return buffers created by `Buffer::from_vec` and
release them with the matching SDK's `free_buffer` exactly once. Do not change
the pointer or length, or free a buffer with a different allocator.

## 10. Values across the native boundary

Ordinary values map through JSON:

| Ject | Rust `serde_json::Value` |
| --- | --- |
| `nil` | `Null` |
| boolean | `Bool` |
| integer | signed JSON number |
| finite float | JSON number |
| string | string |
| array / unique array | array |
| dictionary / struct instance | object |

Functions become callback handles under ABI v2. Opaque resources and non-finite floats
use tagged wire objects created by SDK helpers.

### Non-finite floats

JSON cannot represent NaN or infinity as ordinary numbers:

```rust
let infinity = ject_native::special_float(f64::INFINITY)
    .map_err(str::to_string)?;
```

The host reconstructs the corresponding Ject float.

### Opaque resources

Rust keeps the actual object and gives Ject a typed handle:

```rust
let handle = ject_native::resource(id, "audio_stream");
```

Store resources in plugin-owned state keyed by `id`. Implement the reserved
`__drop_resource` call to release them when the final Ject handle is dropped:

```rust
"__drop_resource" => {
    if let Some(id) = args.first().and_then(Value::as_u64) {
        streams().lock().map_err(|_| "stream lock poisoned")?.remove(&id);
    }
    Ok(Value::Null)
}
```

The type name improves diagnostics and prevents a raw integer from pretending to be a
resource.

## 11. ABI v2 callbacks

Use ABI v2 when Rust must synchronously call a Ject function supplied as an argument:

```rust
use serde_json::Value;

fn call(
    function: &str,
    args: Vec<Value>,
    host: *const ject_native::HostV1,
) -> Result<Value, String> {
    match function {
        "apply" => {
            let callback = args
                .first()
                .and_then(ject_native::callback_id)
                .ok_or("apply expects a callback")?;
            let value = args.get(1).cloned().unwrap_or(Value::Null);
            unsafe { ject_native::invoke_callback(host, callback, vec![value]) }
        }
        _ => Err(format!("unknown function '{function}'")),
    }
}

ject_native::ject_plugin_v2!("audio_engine", ["apply"], call);
```

The host pointer is valid only for the current call and on its calling thread. Callback
handles must not be retained for later use. Errors thrown by Ject propagate back through
`invoke_callback` as `Err(String)`.

## 12. Publishing

Set a semantic package version, check and test the package, then publish:

```sh
ject check
ject test
ject build --release
ject publish --registry https://packages.example
```

Set `JECT_REGISTRY_TOKEN` when the server requires bearer authentication. Published
versions are immutable: attempting to replace the same package and version fails.
Archives carry an archive checksum and a content checksum; cached packages are verified
again when loaded.

The publisher excludes transient build and repository output such as `target/` and
`.git/`. A mixed package must include `native/Cargo.lock`.

## 13. Reproducibility and security

- Path sources are checksummed in the lockfile but remain local and editable.
- Git sources are pinned to a full commit and content checksum.
- Registry archives are immutable and verified before use.
- Native builds from non-path sources use Cargo's lockfile.
- Native plugins run in-process with the permissions of `ject`.

Checksums provide integrity, not isolation. Only install native packages you trust.
Ject does not yet provide capability restrictions or a WebAssembly sandbox.

## 14. Package design checklist

- Keep the exported API small and name it in Ject.
- Use `export const` for public constants.
- Do not expose `@native/...` to application code.
- Prefer a source-only package until Rust has a concrete benefit.
- Give native resources specific type names and deterministic cleanup.
- Use ABI v1 unless callbacks are required.
- Test the public facade rather than only the Rust handler.
- Commit `Ject.lock` for applications and `native/Cargo.lock` for published mixed
  packages.
- Run `ject install --locked`, `ject check`, and `ject test` in CI.
