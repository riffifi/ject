# Ject VS Code extension

## Build `out/*.js` from TypeScript

```bash
npm install
npm run compile
```

If you do not have npm, you can still compile using the TypeScript package alone plus typings (see CI or local `tsc -p .` with `@types/node` and `@types/vscode` installed).

## Features

- Syntax highlighting, snippets, formatter
- **Run** / REPL / selection-run / `cargo build`
- **Safe diagnostics**: `ject --check` (never runs your script)
- Testing sidebar: discovers `test_*.ject` and `*_test.ject`, runs `ject --test <file>`

See [DEBUG_AND_LSP_ROADMAP.md](https://github.com/riffifi/ject/blob/master/vscode-ject/DEBUG_AND_LSP_ROADMAP.md) for planned DAP debugger and LSP integration.
