# Ject Linter for VS Code

This extension provides linting for the Ject language in Visual Studio Code.

## Features
- Highlights unmatched `do`/`end` blocks
- Warns about unknown keywords
- Runs on open, save, and edit of `.ject` files

## Getting Started
1. Run `npm install` in this folder.
2. Run `npm run compile` to build the extension.
3. Press `F5` in VS Code to launch the extension in a new Extension Development Host window.

## File Structure
- `src/extension.ts`: Main linter logic
- `language-configuration.json`: Language configuration
- `syntaxes/ject.tmLanguage.json`: Syntax highlighting (optional)

## Example Linting
Open a `.ject` file and see diagnostics for unmatched blocks and unknown keywords.

## License
MIT
