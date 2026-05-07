# Ject for VS Code

Language support for Ject (`.ject` and `.jt`) files.

## Features

- Syntax highlighting for Ject keywords, strings, interpolation, comments, ranges, slices, unique arrays, structs, imports, and built-ins.
- The Ject file icon supplied by the language author.
- Snippets for functions, loops, conditionals, modules, structs, try/catch, interpolation, and unique arrays.
- Completion items and hover notes for core keywords, standard modules, built-ins, and constants.
- Document symbols for functions, exported functions, structs, and exported values.
- Built-in formatter for `end`-closed Ject blocks.
- Commands to run the current file, run a selection, start the REPL, build the Rust interpreter, check a file, and open examples.

## Commands

- `Ject: Run Current File`
- `Ject: Run Selection`
- `Ject: Start REPL`
- `Ject: Build Interpreter`
- `Ject: Check Current File`
- `Ject: Open Examples Folder`

## Settings

- `ject.executablePath`: Path to the Ject executable. Defaults to `ject`.
- `ject.workspaceExecutable`: Prefer `target/debug/ject` or `target/release/ject` from the workspace when present.
- `ject.format.indentSize`: Number of spaces for formatter indentation.

`Ject: Check Current File` invokes the Ject CLI. Because the current CLI parses, lints, and runs files in one command, checks are opt-in instead of automatic on every keystroke.
