# Ject examples

These examples are meant to be run, changed, and broken while learning. Start from the
repository root after building or installing `ject`:

```sh
ject examples/01_hello.ject
```

The numbered files progress from syntax to small applications:

| File | What it demonstrates |
| --- | --- |
| `01_hello.ject` | Output and the smallest program |
| `02_variables.ject` | Bindings and runtime values |
| `03_math.ject` | Arithmetic and numeric helpers |
| `04_arrays.ject` | Arrays, indexing, and collection operations |
| `05_strings.ject` | Strings and the `string` module |
| `06_conditionals.ject` | `if`, `elseif`, and `else` |
| `07_loops.ject` | `while`, `for`, ranges, `break`, and `continue` |
| `08_functions.ject` | Named functions, defaults, and return values |
| `09_structs.ject` | Struct declarations and field access |
| `10_input.ject` | Interactive terminal input |
| `11_files.ject` | Reading and writing files |
| `12_system.ject` | Environment and process helpers |
| `13_error_handling.ject` | `throw`, `try`, and `catch` |
| `14_calculator.ject` | A small interactive calculator |
| `15_todo_manager.ject` | A larger stateful terminal application |

Some examples create files or wait for input. Read the source before using those in an
automated run.

## Package examples

The subdirectories demonstrate the package system rather than standalone execution.

### `native_double`

A complete mixed Ject/Rust library. Its Ject facade provides validation and stable public
functions; its ABI v2 Rust plugin provides an integer kernel, an opaque counter resource,
and a callback example.

```sh
cd examples/native_double
ject build
```

### `native_double_demo`

An application with a path dependency on `native_double`:

```sh
cd examples/native_double_demo
ject install
ject run
```

### `jgui_demo`

A small application using the separately packaged JGUI facade and native renderer. Build
the JGUI dependency before running when it has not already been installed:

```sh
cd examples/jgui_demo
ject install
ject run
```

For explanations of the syntax, see [the language guide](../docs/DOCS.md). For the package
layout and native ABI, see [the package guide](../docs/PACKAGES.md).
