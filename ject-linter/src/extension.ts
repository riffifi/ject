import * as vscode from 'vscode';

const JECT_KEYWORDS = [
  'let', 'print', 'for', 'in', 'do', 'end', 'if', 'else', 'and', 'or', 'not', 'range', 'true', 'false', 'null',
  'sum', 'max', 'min', 'round', 'sqrt', 'pow', 'abs', 'len', 'type_of', 'trim', 'upper', 'PI', 'E'
];

function lintJect(document: vscode.TextDocument): vscode.Diagnostic[] {
  const diagnostics: vscode.Diagnostic[] = [];
  const lines: string[] = document.getText().split(/\r?\n/);
  let doStack: number[] = [];

  lines.forEach((line: string, i: number) => {
    // Unmatched do/end
    if (/\bdo\b/.test(line)) doStack.push(i);
    if (/\bend\b/.test(line)) {
      if (doStack.length === 0) {
        diagnostics.push(new vscode.Diagnostic(
          new vscode.Range(i, line.indexOf('end'), i, line.indexOf('end') + 3),
          "Unmatched 'end'",
          vscode.DiagnosticSeverity.Error
        ));
      } else {
        doStack.pop();
      }
    }

    // Unknown keyword
    const keywordMatch: string[] | null = line.match(/\b([a-zA-Z_][a-zA-Z0-9_]*)\b/g);
    if (keywordMatch) {
      keywordMatch.forEach((word: string) => {
        if (/^[a-z]+$/.test(word) && !JECT_KEYWORDS.includes(word)) {
          diagnostics.push(new vscode.Diagnostic(
            new vscode.Range(i, line.indexOf(word), i, line.indexOf(word) + word.length),
            `Unknown keyword: '${word}'`,
            vscode.DiagnosticSeverity.Warning
          ));
        }
      });
    }

    // Check for missing 'end' at EOF
    if (i === lines.length - 1 && doStack.length > 0) {
      doStack.forEach((idx: number) => {
        diagnostics.push(new vscode.Diagnostic(
          new vscode.Range(idx, 0, idx, lines[idx].length),
          "Unmatched 'do' (missing 'end')",
          vscode.DiagnosticSeverity.Error
        ));
      });
    }
  });

  return diagnostics;
}

export function activate(context: vscode.ExtensionContext) {
  const collection = vscode.languages.createDiagnosticCollection('ject');

  function lintActiveDocument(document?: vscode.TextDocument) {
    if (!document || document.languageId !== 'ject') return;
    const diagnostics = lintJect(document);
    collection.set(document.uri, diagnostics);
  }

  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument(lintActiveDocument),
    vscode.workspace.onDidSaveTextDocument(lintActiveDocument),
    vscode.workspace.onDidChangeTextDocument((e: vscode.TextDocumentChangeEvent) => lintActiveDocument(e.document)),
    vscode.window.onDidChangeActiveTextEditor((editor: vscode.TextEditor | undefined) => {
      if (editor) lintActiveDocument(editor.document);
    })
  );

  // Lint all open .ject files on activation
  vscode.workspace.textDocuments.forEach(lintActiveDocument);
}

export function deactivate() {}
