"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
const languageSelector = { language: "ject", scheme: "file" };
const keywords = [
    "let", "fn", "lambda", "if", "elseif", "else", "while", "for", "in", "return", "true", "false",
    "nil", "end", "do", "then", "print", "import", "export", "from", "as", "match", "when", "struct",
    "new", "try", "catch", "throw", "break", "continue", "to", "and", "or"
];
const builtins = [
    "type_of", "to_int", "to_float", "to_string", "to_bool", "len", "range", "push", "pop", "sum",
    "contains", "index_of", "first", "last", "sort", "reverse", "unique", "map", "filter", "reduce",
    "abs", "sqrt", "pow", "sin", "cos", "tan", "floor", "ceil", "round", "min", "max", "random",
    "random_int", "upper", "lower", "trim", "split", "join", "replace", "char_at", "substring",
    "repeat", "input", "read_file", "write_file", "assert", "parse_json", "to_json", "env", "exit",
    "now", "timestamp", "sleep", "to_binary", "to_octal", "to_hex", "from_binary", "from_octal",
    "from_hex", "base_repr", "from_base"
];
const modules = ["math", "string", "array", "io", "json", "system", "base", "numpy"];
const keywordDocs = new Map([
    ["let", "Declare a variable: `let name = value`."],
    ["fn", "Define an `end`-closed function. Parameters may have default values."],
    ["lambda", "Create an inline function: `lambda(x) -> x * x`."],
    ["if", "Start a conditional block. Supports `elseif`, `else`, optional `then`, and closes with `end`."],
    ["for", "Iterate over arrays, strings, ranges, and other iterable values: `for item in items do`."],
    ["while", "Run a block while a condition is truthy. Optional `do` is supported."],
    ["import", "Load a module: `import \"math\"`, `import \"math\" as m`, or `import {sqrt} from \"math\"`."],
    ["export", "Export a value or function from a module."],
    ["struct", "Define a lightweight struct: `struct Point {x, y}`."],
    ["try", "Catch thrown errors with `catch name` and close the catch body with `end`."],
    ["print", "Print one or more values. Supports keyword arguments: `print a, b, sep:\", \", end:\"\\n\"`."]
]);
let runTerminal;
let replTerminal;
let diagnosticCollection;
function activate(context) {
    diagnosticCollection = vscode.languages.createDiagnosticCollection("ject");
    context.subscriptions.push(diagnosticCollection);
    context.subscriptions.push(vscode.commands.registerCommand("ject.runFile", runCurrentFile), vscode.commands.registerCommand("ject.runSelection", runSelection), vscode.commands.registerCommand("ject.startRepl", startRepl), vscode.commands.registerCommand("ject.buildInterpreter", buildInterpreter), vscode.commands.registerCommand("ject.checkFile", checkCurrentFile), vscode.commands.registerCommand("ject.openExamples", openExamples), vscode.languages.registerCompletionItemProvider(languageSelector, new JectCompletionProvider(), ".", "\""), vscode.languages.registerHoverProvider(languageSelector, new JectHoverProvider()), vscode.languages.registerDocumentSymbolProvider(languageSelector, new JectDocumentSymbolProvider()), vscode.languages.registerDocumentFormattingEditProvider(languageSelector, new JectFormattingProvider()), vscode.workspace.onDidCloseTextDocument((document) => diagnosticCollection.delete(document.uri)));
}
function deactivate() {
    diagnosticCollection?.dispose();
}
async function runCurrentFile(uri) {
    const document = await getTargetDocument(uri);
    if (!document) {
        return;
    }
    await document.save();
    const executable = await resolveJectExecutable();
    const terminal = getRunTerminal();
    terminal.show(true);
    terminal.sendText(`${quote(executable)} ${quote(document.uri.fsPath)}`);
}
async function runSelection() {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== "ject") {
        vscode.window.showWarningMessage("Open a Ject file and select code to run.");
        return;
    }
    const selectedText = editor.document.getText(editor.selection);
    if (!selectedText.trim()) {
        vscode.window.showWarningMessage("Select some Ject code first.");
        return;
    }
    const tmpFile = path.join(os.tmpdir(), `ject-selection-${Date.now()}.ject`);
    await fs.promises.writeFile(tmpFile, selectedText, "utf8");
    const executable = await resolveJectExecutable();
    const terminal = getRunTerminal();
    terminal.show(true);
    terminal.sendText(`${quote(executable)} ${quote(tmpFile)}`);
}
async function startRepl() {
    const executable = await resolveJectExecutable();
    replTerminal = replTerminal ?? vscode.window.createTerminal({ name: "Ject REPL", cwd: workspaceRoot() });
    replTerminal.show(true);
    replTerminal.sendText(quote(executable));
}
async function buildInterpreter() {
    const root = workspaceRoot();
    if (!root || !fs.existsSync(path.join(root, "Cargo.toml"))) {
        vscode.window.showWarningMessage("Open the Ject repository to build the interpreter with Cargo.");
        return;
    }
    const terminal = vscode.window.createTerminal({ name: "Ject Build", cwd: root });
    terminal.show(true);
    terminal.sendText("cargo build");
}
async function checkCurrentFile() {
    const document = await getTargetDocument();
    if (!document) {
        return;
    }
    await document.save();
    diagnosticCollection.delete(document.uri);
    const executable = await resolveJectExecutable();
    const childProcess = await Promise.resolve().then(() => __importStar(require("child_process")));
    childProcess.execFile(executable, [document.uri.fsPath], { cwd: workspaceRoot(), timeout: 15000 }, (error, stdout, stderr) => {
        const output = `${stdout}\n${stderr}`;
        const diagnostics = parseDiagnostics(output, document);
        diagnosticCollection.set(document.uri, diagnostics);
        if (diagnostics.length === 0 && !error) {
            vscode.window.showInformationMessage("Ject check passed.");
        }
        else if (diagnostics.length === 0) {
            vscode.window.showWarningMessage("Ject returned an error, but no line diagnostics were found. See the Ject terminal output.");
            const terminal = getRunTerminal();
            terminal.show(true);
            terminal.sendText(`${quote(executable)} ${quote(document.uri.fsPath)}`);
        }
        else {
            vscode.window.showInformationMessage(`Ject check found ${diagnostics.length} issue${diagnostics.length === 1 ? "" : "s"}.`);
        }
    });
}
async function openExamples() {
    const root = workspaceRoot();
    const examples = root ? path.join(root, "examples") : undefined;
    if (!examples || !fs.existsSync(examples)) {
        vscode.window.showWarningMessage("No examples folder was found in this workspace.");
        return;
    }
    await vscode.commands.executeCommand("vscode.openFolder", vscode.Uri.file(examples), { forceNewWindow: true });
}
class JectCompletionProvider {
    provideCompletionItems(document, position) {
        const linePrefix = document.lineAt(position).text.slice(0, position.character);
        if (/import\s+"[^"]*$/.test(linePrefix) || /from\s+"[^"]*$/.test(linePrefix)) {
            return modules.map((moduleName) => {
                const item = new vscode.CompletionItem(moduleName, vscode.CompletionItemKind.Module);
                item.detail = "Ject standard module";
                item.insertText = moduleName;
                return item;
            });
        }
        const items = [];
        for (const keyword of keywords) {
            const item = new vscode.CompletionItem(keyword, vscode.CompletionItemKind.Keyword);
            item.documentation = keywordDocs.get(keyword);
            items.push(item);
        }
        for (const builtin of builtins) {
            const item = new vscode.CompletionItem(builtin, vscode.CompletionItemKind.Function);
            item.detail = "Ject built-in";
            item.insertText = new vscode.SnippetString(`${builtin}($0)`);
            items.push(item);
        }
        items.push(...snippetCompletions());
        return items;
    }
}
class JectHoverProvider {
    provideHover(document, position) {
        const range = document.getWordRangeAtPosition(position);
        const word = range ? document.getText(range) : undefined;
        if (!word) {
            return undefined;
        }
        if (keywordDocs.has(word)) {
            return new vscode.Hover(new vscode.MarkdownString(keywordDocs.get(word)));
        }
        if (builtins.includes(word)) {
            return new vscode.Hover(new vscode.MarkdownString(`\`${word}\` is available from Ject's built-in library.`));
        }
        if (word === "PI" || word === "E") {
            return new vscode.Hover(new vscode.MarkdownString(`\`${word}\` is a built-in numeric constant.`));
        }
        return undefined;
    }
}
class JectDocumentSymbolProvider {
    provideDocumentSymbols(document) {
        const symbols = [];
        const stack = [];
        for (let line = 0; line < document.lineCount; line += 1) {
            const text = document.lineAt(line).text;
            const trimmed = text.trim();
            const range = new vscode.Range(line, 0, line, text.length);
            const fnMatch = trimmed.match(/^export\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)|^fn\s+([A-Za-z_][A-Za-z0-9_]*)/);
            const structMatch = trimmed.match(/^struct\s+([A-Za-z_][A-Za-z0-9_]*)/);
            const exportMatch = trimmed.match(/^export\s+([A-Za-z_][A-Za-z0-9_]*)\s*=/);
            let symbol;
            if (fnMatch) {
                const name = fnMatch[1] ?? fnMatch[2];
                symbol = new vscode.DocumentSymbol(name, "function", vscode.SymbolKind.Function, range, range);
            }
            else if (structMatch) {
                symbol = new vscode.DocumentSymbol(structMatch[1], "struct", vscode.SymbolKind.Struct, range, range);
            }
            else if (exportMatch) {
                symbol = new vscode.DocumentSymbol(exportMatch[1], "export", vscode.SymbolKind.Variable, range, range);
            }
            if (symbol) {
                if (stack.length > 0) {
                    stack[stack.length - 1].children.push(symbol);
                }
                else {
                    symbols.push(symbol);
                }
                if (/^(export\s+)?fn\b/.test(trimmed)) {
                    stack.push(symbol);
                }
            }
            if (/^end\b/.test(trimmed) && stack.length > 0) {
                const finished = stack.pop();
                if (finished) {
                    finished.range = new vscode.Range(finished.range.start, range.end);
                }
            }
        }
        return symbols;
    }
}
class JectFormattingProvider {
    provideDocumentFormattingEdits(document) {
        const indentSize = vscode.workspace.getConfiguration("ject").get("format.indentSize", 4);
        const formatted = formatJect(document.getText(), indentSize);
        const fullRange = new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length));
        return [vscode.TextEdit.replace(fullRange, formatted)];
    }
}
function formatJect(source, indentSize) {
    const lines = source.replace(/\r\n/g, "\n").split("\n");
    let indent = 0;
    const result = [];
    for (const rawLine of lines) {
        const trimmed = rawLine.trim();
        if (!trimmed) {
            result.push("");
            continue;
        }
        if (/^(end|elseif\b|else\b|catch\b|when\b)/.test(trimmed)) {
            indent = Math.max(0, indent - 1);
        }
        result.push(`${" ".repeat(indent * indentSize)}${trimmed}`);
        if (opensBlock(trimmed)) {
            indent += 1;
        }
    }
    return result.join("\n");
}
function opensBlock(trimmed) {
    if (/^(end|else\b|elseif\b|catch\b|when\b)/.test(trimmed)) {
        return /^(else\b|elseif\b|catch\b|when\b)/.test(trimmed);
    }
    return /^(fn\b|if\b|while\b|for\b|try\b|match\b)/.test(trimmed)
        || (/^struct\b/.test(trimmed) && !trimmed.includes("}"));
}
function parseDiagnostics(output, document) {
    const diagnostics = [];
    const chunks = output.split(/\n(?=(?:Error|Warning|Runtime Error|Parse error|Parse Error)\b)/i);
    for (const chunk of chunks) {
        const severity = /warning/i.test(chunk) ? vscode.DiagnosticSeverity.Warning : vscode.DiagnosticSeverity.Error;
        const lineMatch = chunk.match(/line\s+(\d+)(?:,\s*column\s+(\d+))?/i);
        const codeMatch = chunk.match(/\b([EW]\d{4})\b/);
        const messageLine = chunk.split(/\r?\n/).find((line) => /(error|warning)/i.test(line))?.replace(/\x1b\[[0-9;]*m/g, "").trim();
        if (!lineMatch && !messageLine) {
            continue;
        }
        const line = Math.max(0, Number(lineMatch?.[1] ?? 1) - 1);
        const column = Math.max(0, Number(lineMatch?.[2] ?? 1) - 1);
        const range = document.lineAt(Math.min(line, document.lineCount - 1)).range;
        const diagnosticRange = new vscode.Range(line, Math.min(column, range.end.character), line, range.end.character);
        const diagnostic = new vscode.Diagnostic(diagnosticRange, messageLine ?? chunk.trim(), severity);
        diagnostic.source = "ject";
        diagnostic.code = codeMatch?.[1];
        diagnostics.push(diagnostic);
    }
    return diagnostics;
}
async function getTargetDocument(uri) {
    if (uri) {
        const document = await vscode.workspace.openTextDocument(uri);
        if (document.languageId === "ject" || [".ject", ".jt"].includes(path.extname(document.uri.fsPath))) {
            return document;
        }
    }
    const editor = vscode.window.activeTextEditor;
    if (editor?.document.languageId === "ject") {
        return editor.document;
    }
    vscode.window.showWarningMessage("Open a Ject file first.");
    return undefined;
}
async function resolveJectExecutable() {
    const config = vscode.workspace.getConfiguration("ject");
    const preferWorkspace = config.get("workspaceExecutable", true);
    const root = workspaceRoot();
    if (preferWorkspace && root) {
        for (const candidate of [
            path.join(root, "target", "debug", executableName("ject")),
            path.join(root, "target", "release", executableName("ject"))
        ]) {
            if (fs.existsSync(candidate)) {
                return candidate;
            }
        }
    }
    return config.get("executablePath", "ject");
}
function workspaceRoot() {
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}
function getRunTerminal() {
    runTerminal = runTerminal ?? vscode.window.createTerminal({ name: "Ject", cwd: workspaceRoot() });
    return runTerminal;
}
function executableName(base) {
    return process.platform === "win32" ? `${base}.exe` : base;
}
function quote(value) {
    if (process.platform === "win32") {
        return `"${value.replace(/"/g, "\\\"")}"`;
    }
    return `'${value.replace(/'/g, "'\\''")}'`;
}
function snippetCompletions() {
    const snippets = [
        ["fn block", "fn ${1:name}(${2:params})\n    $0\nend", "Function block"],
        ["if block", "if ${1:condition}\n    $0\nend", "If block"],
        ["for loop", "for ${1:item} in ${2:items} do\n    $0\nend", "For loop"],
        ["try catch", "try\n    ${1}\ncatch ${2:err}\n    $0\nend", "Try/catch block"],
        ["unique array", "{|${1:items}|}", "Unique array literal"]
    ];
    return snippets.map(([label, body, detail]) => {
        const item = new vscode.CompletionItem(label, vscode.CompletionItemKind.Snippet);
        item.insertText = new vscode.SnippetString(body);
        item.detail = detail;
        return item;
    });
}
//# sourceMappingURL=extension.js.map