# Ject native kernel (Rust)

This document is the contract for what **must** stay in the Rust runtime versus what **should** live in `stdlib/*.ject`.

## Principles

1. **CorLib** (`create_corlib()` in `src/stdlib.rs`) — names injected into every interpreter environment at startup. These are host primitives, runtime bridges, or operations that require the VM/interpreter (e.g. `map`/`filter`/`reduce`).
2. **Native-only modules** — resolved entirely in Rust (`get_module()`). No `stdlib/<name>.ject` override for these names.
3. **Standard modules** — implemented primarily in Ject under `stdlib/`. Rust may still **inject** selected builtins into the module scope (see `inject_module_file_builtins`) so Ject code can call fast/native helpers without recursive `export fn x() return x()` wrappers.

## Native-only modules (`import "…"` → Rust)

| Module | Role |
|--------|------|
| `numpy` | Numerical arrays and linear algebra (`src/numpy.rs`). |
| `gui` | Native UI (`src/gui.rs`). |
| `base` | Radix conversion helpers; no `stdlib/base.ject`. |

All other simple names (`math`, `string`, `array`, `io`, …) resolve from **`stdlib/<name>.ject`** when that file exists.

## CorLib categories (non-exhaustive summary)

- **Type / conversion**: `type_of`, `to_int`, `to_float`, `to_string`, `to_bool`
- **Collections / arrays**: `len`, `range`, `push`, `pop`, `sum`, `contains`, `index_of`, `first`, `last`, `slice`, `sort`, `reverse`, `unique`, `map`, `filter`, `reduce`
- **Math (scalar)**: `abs`, `sqrt`, `pow`, `sin`, `cos`, `tan`, `floor`, `ceil`, `round`, `min`, `max`, `random`, `random_int`
- **Strings**: `upper`, `lower`, `trim`, `split`, `join`, `replace`, `char_at`, `substring`, `repeat`
- **I/O (host)**: `print`, `input`, `read_file`, `write_file`, `append_file`, `read_lines`, `file_exists`
- **Dict helpers**: `has_key`, `delete`, `keys`, `values`
- **Testing**: `assert`
- **Constants**: `PI`, `E`

Additional builtins exist in `call_builtin_function` for filesystem, JSON, process, time, etc.; some are also re-exported through `inject_module_file_builtins` for `io`, `json`, `system`, `collections`, `datetime`, `array`, **`math`**, and **`util`**.

## Machine-readable introspection

Run:

```bash
ject --introspect
```

Emits JSON describing CorLib names, native-only modules, and injected module stems (for tooling / VSIX parity checks).
