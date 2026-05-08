import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import * as vscode from "vscode";

const languageSelector: vscode.DocumentSelector = { language: "ject", scheme: "file" };

// ─── Language data ────────────────────────────────────────────────────────────

const keywords = [
  "let", "fn", "lambda", "if", "elseif", "else", "while", "for", "in", "return", "true", "false",
  "nil", "end", "do", "then", "print", "import", "export", "from", "as", "struct",
  "new", "try", "catch", "throw", "break", "continue", "to", "and", "or"
];

interface BuiltinSig {
  sig: string;
  doc: string;
  params: string[];
}

const builtinSignatures = new Map<string, BuiltinSig>([
  // Type & conversion
  ["type_of",    { sig: "type_of(value) -> string",         doc: "Returns the type of `value` as a string: `\"integer\"`, `\"float\"`, `\"string\"`, `\"boolean\"`, `\"nil\"`, `\"array\"`, `\"dictionary\"`.", params: ["value"] }],
  ["to_int",     { sig: "to_int(value) -> integer",         doc: "Converts `value` to an integer. Truncates floats; parses strings.", params: ["value"] }],
  ["to_float",   { sig: "to_float(value) -> float",         doc: "Converts `value` to a float.", params: ["value"] }],
  ["to_string",  { sig: "to_string(value) -> string",       doc: "Converts any value to its string representation.", params: ["value"] }],
  ["to_bool",    { sig: "to_bool(value) -> boolean",        doc: "Converts `value` to a boolean. `0`, `\"\"`, `nil`, `false` are falsy.", params: ["value"] }],
  // Array
  ["len",        { sig: "len(value) -> integer",            doc: "Returns the length of an array or string.", params: ["value"] }],
  ["range",      { sig: "range(stop) | range(start, stop) | range(start, stop, step) -> array", doc: "Returns an integer array. `range(5)` → `[0,1,2,3,4]`.", params: ["start_or_stop", "stop?", "step?"] }],
  ["push",       { sig: "push(arr, item) -> array",         doc: "Returns a new array with `item` appended. Original is unchanged.", params: ["arr", "item"] }],
  ["pop",        { sig: "pop(arr) -> array",                doc: "Returns a new array with the last element removed.", params: ["arr"] }],
  ["sum",        { sig: "sum(arr) -> number",               doc: "Returns the sum of all numeric elements in `arr`.", params: ["arr"] }],
  ["contains",   { sig: "contains(arr, item) -> boolean",   doc: "Returns `true` if `arr` contains `item`.", params: ["arr", "item"] }],
  ["index_of",   { sig: "index_of(arr, item) -> integer",   doc: "Returns the index of `item` in `arr`, or `-1` if not found.", params: ["arr", "item"] }],
  ["first",      { sig: "first(arr) -> any",                doc: "Returns the first element of `arr`.", params: ["arr"] }],
  ["last",       { sig: "last(arr) -> any",                 doc: "Returns the last element of `arr`.", params: ["arr"] }],
  ["slice",      { sig: "slice(arr, start, end) -> array",  doc: "Returns elements from index `start` up to (not including) `end`.", params: ["arr", "start", "end"] }],
  ["sort",       { sig: "sort(arr) -> array",               doc: "Returns a sorted copy of `arr`.", params: ["arr"] }],
  ["reverse",    { sig: "reverse(arr) -> array",            doc: "Returns a reversed copy of `arr`.", params: ["arr"] }],
  ["unique",     { sig: "unique(arr) -> array",             doc: "Returns a deduplicated copy of `arr`.", params: ["arr"] }],
  ["map",        { sig: "map(arr, fn) -> array",            doc: "Returns a new array by applying `fn` to each element.", params: ["arr", "fn"] }],
  ["filter",     { sig: "filter(arr, fn) -> array",         doc: "Returns elements of `arr` for which `fn` returns truthy.", params: ["arr", "fn"] }],
  ["reduce",     { sig: "reduce(arr, fn, initial) -> any",  doc: "Folds `arr` left using `fn(accumulator, item)`, starting from `initial`.", params: ["arr", "fn", "initial"] }],
  // Math
  ["abs",        { sig: "abs(x) -> number",                 doc: "Absolute value of `x`.", params: ["x"] }],
  ["sqrt",       { sig: "sqrt(x) -> float",                 doc: "Square root of `x`.", params: ["x"] }],
  ["pow",        { sig: "pow(base, exp) -> number",         doc: "`base` raised to the power `exp`.", params: ["base", "exp"] }],
  ["sin",        { sig: "sin(radians) -> float",            doc: "Sine of `radians`.", params: ["radians"] }],
  ["cos",        { sig: "cos(radians) -> float",            doc: "Cosine of `radians`.", params: ["radians"] }],
  ["tan",        { sig: "tan(radians) -> float",            doc: "Tangent of `radians`.", params: ["radians"] }],
  ["floor",      { sig: "floor(x) -> integer",              doc: "Round `x` down to the nearest integer.", params: ["x"] }],
  ["ceil",       { sig: "ceil(x) -> integer",               doc: "Round `x` up to the nearest integer.", params: ["x"] }],
  ["round",      { sig: "round(x) -> integer",              doc: "Round `x` to the nearest integer.", params: ["x"] }],
  ["min",        { sig: "min(a, b) -> number",              doc: "Returns the smaller of `a` and `b`.", params: ["a", "b"] }],
  ["max",        { sig: "max(a, b) -> number",              doc: "Returns the larger of `a` and `b`.", params: ["a", "b"] }],
  ["random",     { sig: "random() -> float",                doc: "Returns a random float in `[0.0, 1.0)`.", params: [] }],
  ["random_int", { sig: "random_int(min, max) -> integer",  doc: "Returns a random integer in `[min, max)`.", params: ["min", "max"] }],
  // String
  ["upper",      { sig: "upper(str) -> string",             doc: "Converts `str` to uppercase.", params: ["str"] }],
  ["lower",      { sig: "lower(str) -> string",             doc: "Converts `str` to lowercase.", params: ["str"] }],
  ["trim",       { sig: "trim(str) -> string",              doc: "Removes leading and trailing whitespace from `str`.", params: ["str"] }],
  ["split",      { sig: "split(str, sep) -> array",         doc: "Splits `str` on `sep`, returning an array of strings.", params: ["str", "sep"] }],
  ["join",       { sig: "join(arr, sep) -> string",         doc: "Joins elements of `arr` into a string, separated by `sep`.", params: ["arr", "sep"] }],
  ["replace",    { sig: "replace(str, old, new) -> string", doc: "Replaces all occurrences of `old` with `new` in `str`.", params: ["str", "old", "new"] }],
  ["char_at",    { sig: "char_at(str, index) -> string",   doc: "Returns the character at `index` (negative indexing supported).", params: ["str", "index"] }],
  ["substring",  { sig: "substring(str, start, end) -> string", doc: "Returns the substring from `start` to `end` (exclusive).", params: ["str", "start", "end"] }],
  ["repeat",     { sig: "repeat(str, n) -> string",         doc: "Returns `str` repeated `n` times.", params: ["str", "n"] }],
  // I/O
  ["print",      { sig: "print ...values [sep:string] [end:string]", doc: "Prints values to stdout. Optional `sep:` and `end:` keyword args.", params: ["...values"] }],
  ["input",      { sig: "input(prompt) -> string",          doc: "Displays `prompt` and reads a line from stdin.", params: ["prompt"] }],
  ["read_file",  { sig: "read_file(path) -> string",        doc: "Reads the file at `path` and returns its contents as a string.", params: ["path"] }],
  ["write_file", { sig: "write_file(path, content)",        doc: "Writes `content` to the file at `path`, creating or overwriting it.", params: ["path", "content"] }],
  ["file_exists",{ sig: "file_exists(path) -> boolean",     doc: "Returns `true` if a file or directory exists at `path`.", params: ["path"] }],
  ["is_file",    { sig: "is_file(path) -> boolean",         doc: "Returns `true` if `path` refers to a regular file.", params: ["path"] }],
  ["is_dir",     { sig: "is_dir(path) -> boolean",          doc: "Returns `true` if `path` refers to a directory.", params: ["path"] }],
  // JSON
  ["parse_json", { sig: "parse_json(str) -> any",           doc: "Parses a JSON string into a Ject value.", params: ["str"] }],
  ["to_json",    { sig: "to_json(value) -> string",         doc: "Serializes a Ject value to a JSON string.", params: ["value"] }],
  // System
  ["exec",       { sig: "exec(command) -> string",          doc: "Runs `command` in a shell and returns stdout as a string.", params: ["command"] }],
  ["env",        { sig: "env(name) -> string",              doc: "Returns the value of environment variable `name`.", params: ["name"] }],
  ["exit",       { sig: "exit(code)",                       doc: "Exits the program with the given status `code`.", params: ["code"] }],
  ["now",        { sig: "now() -> float",                   doc: "Returns the current time as a Unix timestamp (float seconds).", params: [] }],
  ["timestamp",  { sig: "timestamp() -> integer",           doc: "Returns the current Unix timestamp as an integer.", params: [] }],
  ["sleep",      { sig: "sleep(ms)",                        doc: "Sleeps for `ms` milliseconds.", params: ["ms"] }],
]);

const builtins = Array.from(builtinSignatures.keys());

const modules = ["math", "string", "array", "io", "json", "system", "datetime", "util", "numpy", "gui"];

const moduleMembers: Record<string, string[]> = {
  math:     ["sqrt", "log", "log10", "log2", "ln", "exp", "sin", "cos", "tan", "asin", "acos", "atan", "atan2",
             "sinh", "cosh", "tanh", "floor", "ceil", "round", "abs", "pow", "PI", "E", "PHI", "SQRT2", "SQRT3",
             "gcd", "lcm", "factorial", "fibonacci", "is_prime", "primes_up_to", "average", "median",
             "variance", "stddev", "product", "combinations", "permutations", "clamp", "sign",
             "round_to", "lerp", "remap", "deg_to_rad", "rad_to_deg", "distance_2d", "distance_3d",
             "dot_2d", "cross_2d", "magnitude", "normalize_2d", "quadratic_roots",
             "is_even", "is_odd", "is_power_of_2", "next_power_of_2", "nth_root"],
  string:   ["capitalize", "title_case", "trim_left", "trim_right", "pad_left", "pad_right", "pad_center",
             "starts_with", "ends_with", "contains_str", "find", "count", "replace_all", "replace_first",
             "remove", "reverse_str", "left", "right", "truncate", "is_empty", "is_numeric", "is_alpha",
             "word_count", "sentence_count", "lines", "format", "escape", "unescape", "wrap_text", "extract_numbers"],
  array:    ["average", "median", "take", "drop", "initial", "rest", "concat", "zip", "union", "intersection",
             "difference", "flatten", "chunk", "shuffle", "rotate_left", "rotate_right", "insert_at",
             "remove_at", "compact", "enumerate", "fill_arr", "range_arr", "sample", "sort_by"],
  io:       ["write_lines", "read_json", "write_json"],
  json:     ["to_json_pretty", "is_valid_json", "json_get"],
  system:   ["get_cwd", "change_dir"],
  util:     ["identity", "constant", "compose", "apply", "is_nil", "is_truthy", "deep_equal", "copy"],
  numpy:    ["array", "zeros", "ones", "arange", "linspace", "eye", "identity", "shape", "ndim", "size",
             "reshape", "flatten", "transpose", "concatenate", "stack", "sqrt", "exp", "log", "abs",
             "sin", "cos", "add", "subtract", "multiply", "divide", "sum", "min", "max", "mean", "std",
             "var", "dot", "norm", "det", "inv", "solve"],
  gui:      ["window", "label", "separator", "button", "input", "run"],
};

const keywordDocs = new Map<string, string>([
  ["let",      "Declare a variable: `let name = value`."],
  ["fn",       "Define a named function. Closes with `end`.\n\n```ject\nfn greet(name, greeting = \"Hello\")\n    print \"$greeting, $name!\"\nend\n```"],
  ["lambda",   "Create an inline function: `lambda(x) -> x * x`. `fn` also works for lambdas."],
  ["if",       "Conditional block. `then` is optional. Closes with `end`.\n\n```ject\nif x > 0\n    print \"positive\"\nelseif x < 0\n    print \"negative\"\nelse\n    print \"zero\"\nend\n```"],
  ["elseif",   "Additional condition branch inside an `if` block."],
  ["else",     "Fallback branch inside an `if` block."],
  ["for",      "Iterate over an array, string, or range.\n\n```ject\nfor item in list do\n    print item\nend\n\nfor i in 0..10:2 do  # 0, 2, 4, 6, 8\n    print i\nend\n```"],
  ["while",    "Loop while condition is truthy. `do` is optional.\n\n```ject\nwhile i < 10 do\n    i = i + 1\nend\n```"],
  ["return",   "Return a value from a function. A function with no `return` returns `nil`."],
  ["import",   "Load a module.\n\n```ject\nimport \"math\" as m\nimport {sqrt, PI} from \"math\"\n```"],
  ["export",   "Export a value or function from a module.\n\n```ject\nexport PI = 3.14159\nexport fn square(x)\n    return x * x\nend\n```"],
  ["struct",   "Define a named data type with fields.\n\n```ject\nstruct Point { x, y }\nlet p = new Point { x: 10, y: 20 }\nprint p.x\n```"],
  ["new",      "Instantiate a struct: `new StructName { field: value, ... }`."],
  ["try",      "Catch thrown errors.\n\n```ject\ntry\n    risky()\ncatch err\n    print \"Error: \" + err\nend\n```"],
  ["throw",    "Throw an error value: `throw \"something went wrong\"`."],
  ["break",    "Exit the current loop immediately."],
  ["continue", "Skip the rest of the current loop iteration."],
  ["print",    "Print values to stdout.\n\n```ject\nprint \"Hello\"\nprint a, b, c sep:\", \" end:\"\"\n```"],
  ["and",      "Logical AND: `a and b`"],
  ["or",       "Logical OR: `a or b`"],
  ["nil",      "The null/absence value. Use `== nil` to check for it."],
  ["true",     "Boolean true."],
  ["false",    "Boolean false."],
]);

// ─── State ───────────────────────────────────────────────────────────────────

let runTerminal: vscode.Terminal | undefined;
let replTerminal: vscode.Terminal | undefined;
let diagnosticCollection: vscode.DiagnosticCollection;
let statusBarItem: vscode.StatusBarItem;
let checkTimeout: ReturnType<typeof setTimeout> | undefined;

// ─── Activate ────────────────────────────────────────────────────────────────

export function activate(context: vscode.ExtensionContext): void {
  diagnosticCollection = vscode.languages.createDiagnosticCollection("ject");

  statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  statusBarItem.command = "ject.runFile";
  statusBarItem.tooltip = "Run current Ject file";
  context.subscriptions.push(statusBarItem);

  context.subscriptions.push(
    diagnosticCollection,

    // Commands
    vscode.commands.registerCommand("ject.runFile", runCurrentFile),
    vscode.commands.registerCommand("ject.runSelection", runSelection),
    vscode.commands.registerCommand("ject.startRepl", startRepl),
    vscode.commands.registerCommand("ject.buildInterpreter", buildInterpreter),
    vscode.commands.registerCommand("ject.checkFile", checkCurrentFile),
    vscode.commands.registerCommand("ject.openExamples", openExamples),
    vscode.commands.registerCommand("ject.newFile", newJectFile),

    // Language providers
    vscode.languages.registerCompletionItemProvider(languageSelector, new JectCompletionProvider(), ".", "\"", "("),
    vscode.languages.registerHoverProvider(languageSelector, new JectHoverProvider()),
    vscode.languages.registerDocumentSymbolProvider(languageSelector, new JectDocumentSymbolProvider()),
    vscode.languages.registerWorkspaceSymbolProvider(new JectWorkspaceSymbolProvider()),
    vscode.languages.registerDocumentFormattingEditProvider(languageSelector, new JectFormattingProvider()),
    vscode.languages.registerSignatureHelpProvider(languageSelector, new JectSignatureHelpProvider(), "(", ","),
    vscode.languages.registerDefinitionProvider(languageSelector, new JectDefinitionProvider()),
    vscode.languages.registerReferenceProvider(languageSelector, new JectReferenceProvider()),
    vscode.languages.registerRenameProvider(languageSelector, new JectRenameProvider()),
    vscode.languages.registerFoldingRangeProvider(languageSelector, new JectFoldingProvider()),
    vscode.languages.registerCodeActionsProvider(languageSelector, new JectCodeActionProvider(), {
      providedCodeActionKinds: [vscode.CodeActionKind.QuickFix, vscode.CodeActionKind.RefactorExtract]
    }),

    // Events
    vscode.window.onDidChangeActiveTextEditor(updateStatusBar),
    vscode.workspace.onDidChangeTextDocument(onDocumentChange),
    vscode.workspace.onDidSaveTextDocument(onDocumentSave),
    vscode.workspace.onDidCloseTextDocument((doc) => diagnosticCollection.delete(doc.uri)),
    vscode.window.onDidCloseTerminal((t) => {
      if (t === runTerminal) { runTerminal = undefined; }
      if (t === replTerminal) { replTerminal = undefined; }
    }),
  );

  updateStatusBar(vscode.window.activeTextEditor);
}

export function deactivate(): void {
  diagnosticCollection?.dispose();
  statusBarItem?.dispose();
}

// ─── Status bar ──────────────────────────────────────────────────────────────

function updateStatusBar(editor: vscode.TextEditor | undefined): void {
  if (editor?.document.languageId === "ject") {
    statusBarItem.text = "$(play) Run Ject";
    statusBarItem.show();
  } else {
    statusBarItem.hide();
  }
}

// ─── Auto-check on change / save ─────────────────────────────────────────────

function onDocumentChange(event: vscode.TextDocumentChangeEvent): void {
  if (event.document.languageId !== "ject") { return; }
  if (!vscode.workspace.getConfiguration("ject").get<boolean>("lintOnType", true)) { return; }
  if (checkTimeout) { clearTimeout(checkTimeout); }
  checkTimeout = setTimeout(() => lintDocument(event.document), 600);
}

function onDocumentSave(document: vscode.TextDocument): void {
  if (document.languageId !== "ject") { return; }
  const cfg = vscode.workspace.getConfiguration("ject");
  if (cfg.get<boolean>("checkOnSave", true)) { lintDocument(document); }
  if (cfg.get<boolean>("runOnSave", false)) { runCurrentFile(); }
}

async function lintDocument(document: vscode.TextDocument): Promise<void> {
  const executable = await resolveJectExecutable();
  if (!fs.existsSync(executable) && executable === "ject") { return; }
  const childProcess = await import("child_process");
  childProcess.execFile(executable, [document.uri.fsPath], { cwd: workspaceRoot(), timeout: 10000 }, (_error, stdout, stderr) => {
    const diagnostics = parseDiagnostics(`${stdout}\n${stderr}`, document);
    diagnosticCollection.set(document.uri, diagnostics);
    const errCount  = diagnostics.filter(d => d.severity === vscode.DiagnosticSeverity.Error).length;
    const warnCount = diagnostics.filter(d => d.severity === vscode.DiagnosticSeverity.Warning).length;
    statusBarItem.text = (errCount > 0)
      ? `$(error) Ject ${errCount} error${errCount > 1 ? "s" : ""}`
      : (warnCount > 0)
        ? `$(warning) Ject ${warnCount} warning${warnCount > 1 ? "s" : ""}`
        : "$(play) Run Ject";
  });
}

// ─── Commands ────────────────────────────────────────────────────────────────

async function runCurrentFile(uri?: vscode.Uri): Promise<void> {
  const document = await getTargetDocument(uri);
  if (!document) { return; }
  await document.save();
  const executable = await resolveJectExecutable();
  const terminal = getRunTerminal();
  terminal.show(true);
  terminal.sendText(`${quote(executable)} ${quote(document.uri.fsPath)}`);
}

async function runSelection(): Promise<void> {
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

async function startRepl(): Promise<void> {
  const executable = await resolveJectExecutable();
  replTerminal = replTerminal ?? vscode.window.createTerminal({ name: "Ject REPL", cwd: workspaceRoot() });
  replTerminal.show(true);
  replTerminal.sendText(quote(executable));
}

async function buildInterpreter(): Promise<void> {
  const root = workspaceRoot();
  if (!root || !fs.existsSync(path.join(root, "Cargo.toml"))) {
    vscode.window.showWarningMessage("Open the Ject repository to build the interpreter with Cargo.");
    return;
  }
  const terminal = vscode.window.createTerminal({ name: "Ject Build", cwd: root });
  terminal.show(true);
  terminal.sendText("cargo build --release");
}

async function checkCurrentFile(): Promise<void> {
  const document = await getTargetDocument();
  if (!document) { return; }
  await document.save();
  diagnosticCollection.delete(document.uri);
  const executable = await resolveJectExecutable();
  const childProcess = await import("child_process");
  childProcess.execFile(executable, [document.uri.fsPath], { cwd: workspaceRoot(), timeout: 15000 }, (error, stdout, stderr) => {
    const diagnostics = parseDiagnostics(`${stdout}\n${stderr}`, document);
    diagnosticCollection.set(document.uri, diagnostics);
    if (diagnostics.length === 0 && !error) {
      vscode.window.showInformationMessage("Ject: no issues found.");
    } else if (diagnostics.length === 0) {
      vscode.window.showWarningMessage("Ject returned an error but no line diagnostics. Check the terminal.");
      getRunTerminal().show(true);
      getRunTerminal().sendText(`${quote(executable)} ${quote(document.uri.fsPath)}`);
    } else {
      vscode.window.showInformationMessage(`Ject: ${diagnostics.length} issue${diagnostics.length === 1 ? "" : "s"} found.`);
    }
  });
}

async function openExamples(): Promise<void> {
  const root = workspaceRoot();
  const examples = root ? path.join(root, "examples") : undefined;
  if (!examples || !fs.existsSync(examples)) {
    vscode.window.showWarningMessage("No examples folder found in this workspace.");
    return;
  }
  await vscode.commands.executeCommand("vscode.openFolder", vscode.Uri.file(examples), { forceNewWindow: true });
}

async function newJectFile(): Promise<void> {
  const doc = await vscode.workspace.openTextDocument({ language: "ject", content: "# New Ject file\n\n" });
  await vscode.window.showTextDocument(doc);
}

// ─── Completion ──────────────────────────────────────────────────────────────

class JectCompletionProvider implements vscode.CompletionItemProvider {
  provideCompletionItems(document: vscode.TextDocument, position: vscode.Position): vscode.CompletionItem[] {
    const linePrefix = document.lineAt(position).text.slice(0, position.character);

    // Module name completions inside import strings
    if (/import\s+"[^"]*$/.test(linePrefix) || /from\s+"[^"]*$/.test(linePrefix)) {
      return modules.map((m) => {
        const item = new vscode.CompletionItem(m, vscode.CompletionItemKind.Module);
        item.detail = "Ject standard module";
        item.insertText = m;
        return item;
      });
    }

    // Post-dot member completions
    if (/\.\s*$/.test(linePrefix)) {
      const aliasMatch = linePrefix.match(/(\w+)\.\s*$/);
      const alias = aliasMatch?.[1];
      if (alias) {
        // Try to resolve alias → module name from `import "X" as alias` in the doc
        const resolvedModule = resolveAlias(document, alias);
        const members = resolvedModule ? moduleMembers[resolvedModule] : undefined;
        const candidates = members ?? (moduleMembers[alias] ?? []);
        if (candidates.length > 0) {
          return candidates.map((m) => {
            const item = new vscode.CompletionItem(m, vscode.CompletionItemKind.Method);
            item.insertText = new vscode.SnippetString(`${m}($0)`);
            return item;
          });
        }
      }
      return [];
    }

    const items: vscode.CompletionItem[] = [];
    const userSymbols = collectDocumentSymbols(document);

    // User-defined functions
    for (const sym of userSymbols.functions) {
      const item = new vscode.CompletionItem(sym.name, vscode.CompletionItemKind.Function);
      item.detail = `fn ${sym.name}(${sym.params.join(", ")})`;
      item.documentation = new vscode.MarkdownString(`User-defined function at line ${sym.line + 1}.`);
      item.insertText = sym.params.length > 0
        ? new vscode.SnippetString(`${sym.name}($0)`)
        : new vscode.SnippetString(`${sym.name}()`);
      items.push(item);
    }

    // User-defined variables
    for (const sym of userSymbols.variables) {
      const item = new vscode.CompletionItem(sym.name, vscode.CompletionItemKind.Variable);
      item.detail = "variable";
      items.push(item);
    }

    // User-defined structs
    for (const sym of userSymbols.structs) {
      const item = new vscode.CompletionItem(sym.name, vscode.CompletionItemKind.Struct);
      item.detail = "struct";
      item.insertText = new vscode.SnippetString(`new ${sym.name} { $0 }`);
      items.push(item);
    }

    // Keywords
    for (const kw of keywords) {
      const item = new vscode.CompletionItem(kw, vscode.CompletionItemKind.Keyword);
      const doc = keywordDocs.get(kw);
      if (doc) { item.documentation = new vscode.MarkdownString(doc); }
      items.push(item);
    }

    // Builtins
    for (const [name, sig] of builtinSignatures) {
      const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Function);
      item.detail = sig.sig;
      item.documentation = new vscode.MarkdownString(sig.doc);
      item.insertText = sig.params.length === 0
        ? new vscode.SnippetString(`${name}()`)
        : new vscode.SnippetString(`${name}($0)`);
      items.push(item);
    }

    // Constants
    for (const [name, val, doc] of [
      ["PI", "3.141592653589793", "π — built-in numeric constant."],
      ["E",  "2.718281828459045", "Euler's number — built-in numeric constant."],
    ] as const) {
      const item = new vscode.CompletionItem(name, vscode.CompletionItemKind.Constant);
      item.detail = val;
      item.documentation = new vscode.MarkdownString(doc);
      items.push(item);
    }

    items.push(...snippetCompletions());
    return items;
  }
}

// ─── Hover ───────────────────────────────────────────────────────────────────

class JectHoverProvider implements vscode.HoverProvider {
  provideHover(document: vscode.TextDocument, position: vscode.Position): vscode.Hover | undefined {
    const range = document.getWordRangeAtPosition(position);
    const word = range ? document.getText(range) : undefined;
    if (!word) { return undefined; }

    const sig = builtinSignatures.get(word);
    if (sig) {
      const md = new vscode.MarkdownString();
      md.appendCodeblock(sig.sig, "ject");
      md.appendMarkdown(`\n\n${sig.doc}`);
      return new vscode.Hover(md, range);
    }

    if (keywordDocs.has(word)) {
      const md = new vscode.MarkdownString(keywordDocs.get(word)!);
      md.isTrusted = true;
      return new vscode.Hover(md, range);
    }

    if (word === "PI") { return new vscode.Hover(new vscode.MarkdownString("`PI` = 3.141592653589793"), range); }
    if (word === "E")  { return new vscode.Hover(new vscode.MarkdownString("`E` = 2.718281828459045"), range); }

    // Hover over user-defined function → show its signature
    const userSymbols = collectDocumentSymbols(document);
    const fn = userSymbols.functions.find(f => f.name === word);
    if (fn) {
      const md = new vscode.MarkdownString();
      md.appendCodeblock(`fn ${fn.name}(${fn.params.join(", ")})`, "ject");
      md.appendMarkdown(`\n\nDefined at line ${fn.line + 1}.`);
      return new vscode.Hover(md, range);
    }

    return undefined;
  }
}

// ─── Signature help ──────────────────────────────────────────────────────────

class JectSignatureHelpProvider implements vscode.SignatureHelpProvider {
  provideSignatureHelp(document: vscode.TextDocument, position: vscode.Position): vscode.SignatureHelp | undefined {
    const lineText = document.lineAt(position).text.slice(0, position.character);

    let depth = 0;
    let argIndex = 0;
    let i = lineText.length - 1;

    while (i >= 0) {
      const ch = lineText[i];
      if (ch === ")") { depth++; }
      else if (ch === "(") {
        if (depth === 0) { break; }
        depth--;
      } else if (ch === "," && depth === 0) {
        argIndex++;
      }
      i--;
    }

    if (i < 0) { return undefined; }

    const nameMatch = lineText.slice(0, i).match(/([A-Za-z_][A-Za-z0-9_]*)$/);
    if (!nameMatch) { return undefined; }
    const fnName = nameMatch[1];

    // Builtin
    const builtin = builtinSignatures.get(fnName);
    if (builtin) {
      const sig = new vscode.SignatureInformation(builtin.sig, new vscode.MarkdownString(builtin.doc));
      sig.parameters = builtin.params.map(p => new vscode.ParameterInformation(p));
      const help = new vscode.SignatureHelp();
      help.signatures = [sig];
      help.activeSignature = 0;
      help.activeParameter = Math.min(argIndex, sig.parameters.length - 1);
      return help;
    }

    // User-defined
    const fn = collectDocumentSymbols(document).functions.find(f => f.name === fnName);
    if (fn) {
      const sigText = `fn ${fn.name}(${fn.params.join(", ")})`;
      const sig = new vscode.SignatureInformation(sigText);
      sig.parameters = fn.params.map(p => new vscode.ParameterInformation(p));
      const help = new vscode.SignatureHelp();
      help.signatures = [sig];
      help.activeSignature = 0;
      help.activeParameter = Math.min(argIndex, sig.parameters.length - 1);
      return help;
    }

    return undefined;
  }
}

// ─── Go to definition ────────────────────────────────────────────────────────

class JectDefinitionProvider implements vscode.DefinitionProvider {
  provideDefinition(document: vscode.TextDocument, position: vscode.Position): vscode.Location | undefined {
    const range = document.getWordRangeAtPosition(position);
    const word = range ? document.getText(range) : undefined;
    if (!word) { return undefined; }

    for (let line = 0; line < document.lineCount; line++) {
      const text = document.lineAt(line).text.trim();
      if (new RegExp(`^(?:export\\s+)?fn\\s+${word}\\s*\\(`).test(text)
        || new RegExp(`^let\\s+${word}\\s*=`).test(text)
        || new RegExp(`^struct\\s+${word}\\s*[{(]`).test(text)
        || new RegExp(`^export\\s+${word}\\s*=`).test(text)) {
        return new vscode.Location(document.uri, new vscode.Position(line, text.indexOf(word)));
      }
    }
    return undefined;
  }
}

// ─── Find all references ─────────────────────────────────────────────────────

class JectReferenceProvider implements vscode.ReferenceProvider {
  provideReferences(document: vscode.TextDocument, position: vscode.Position, context: vscode.ReferenceContext): vscode.Location[] {
    const range = document.getWordRangeAtPosition(position);
    const word = range ? document.getText(range) : undefined;
    if (!word) { return []; }

    const locations: vscode.Location[] = [];
    const wordRe = new RegExp(`\\b${escapeRegExp(word)}\\b`, "g");

    for (let line = 0; line < document.lineCount; line++) {
      const text = document.lineAt(line).text;
      let match: RegExpExecArray | null;
      wordRe.lastIndex = 0;
      while ((match = wordRe.exec(text)) !== null) {
        const isDecl = new RegExp(`^(?:export\\s+)?fn\\s+${word}\\b|^let\\s+${word}\\s*=|^struct\\s+${word}\\b`).test(text.trim());
        if (context.includeDeclaration || !isDecl) {
          locations.push(new vscode.Location(document.uri, new vscode.Position(line, match.index)));
        }
      }
    }
    return locations;
  }
}

// ─── Rename ──────────────────────────────────────────────────────────────────

class JectRenameProvider implements vscode.RenameProvider {
  prepareRename(document: vscode.TextDocument, position: vscode.Position): vscode.Range | undefined {
    const range = document.getWordRangeAtPosition(position);
    const word = range ? document.getText(range) : undefined;
    if (!word || keywords.includes(word) || builtins.includes(word)) { return undefined; }
    return range;
  }

  provideRenameEdits(document: vscode.TextDocument, position: vscode.Position, newName: string): vscode.WorkspaceEdit {
    const range = document.getWordRangeAtPosition(position);
    const word = range ? document.getText(range) : undefined;
    const edit = new vscode.WorkspaceEdit();
    if (!word) { return edit; }

    const wordRe = new RegExp(`\\b${escapeRegExp(word)}\\b`, "g");
    for (let line = 0; line < document.lineCount; line++) {
      const text = document.lineAt(line).text;
      let match: RegExpExecArray | null;
      wordRe.lastIndex = 0;
      while ((match = wordRe.exec(text)) !== null) {
        edit.replace(document.uri, new vscode.Range(line, match.index, line, match.index + word.length), newName);
      }
    }
    return edit;
  }
}

// ─── Document symbols ────────────────────────────────────────────────────────

class JectDocumentSymbolProvider implements vscode.DocumentSymbolProvider {
  provideDocumentSymbols(document: vscode.TextDocument): vscode.DocumentSymbol[] {
    const symbols: vscode.DocumentSymbol[] = [];
    const stack: vscode.DocumentSymbol[] = [];

    for (let line = 0; line < document.lineCount; line++) {
      const text = document.lineAt(line).text;
      const trimmed = text.trim();
      const range = new vscode.Range(line, 0, line, text.length);
      const fnMatch     = trimmed.match(/^(?:export\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/);
      const structMatch = trimmed.match(/^struct\s+([A-Za-z_][A-Za-z0-9_]*)/);
      const exportMatch = trimmed.match(/^export\s+([A-Za-z_][A-Za-z0-9_]*)\s*=/);

      let symbol: vscode.DocumentSymbol | undefined;
      if (fnMatch) {
        symbol = new vscode.DocumentSymbol(fnMatch[1], "function", vscode.SymbolKind.Function, range, range);
      } else if (structMatch) {
        symbol = new vscode.DocumentSymbol(structMatch[1], "struct", vscode.SymbolKind.Struct, range, range);
      } else if (exportMatch) {
        symbol = new vscode.DocumentSymbol(exportMatch[1], "export", vscode.SymbolKind.Variable, range, range);
      }

      if (symbol) {
        if (stack.length > 0) {
          stack[stack.length - 1].children.push(symbol);
        } else {
          symbols.push(symbol);
        }
        if (/^(?:export\s+)?fn\b/.test(trimmed)) { stack.push(symbol); }
      }

      if (/^end\b/.test(trimmed) && stack.length > 0) {
        const finished = stack.pop()!;
        finished.range = new vscode.Range(finished.range.start, range.end);
      }
    }
    return symbols;
  }
}

// ─── Workspace symbols ───────────────────────────────────────────────────────

class JectWorkspaceSymbolProvider implements vscode.WorkspaceSymbolProvider {
  async provideWorkspaceSymbols(query: string): Promise<vscode.SymbolInformation[]> {
    const files = await vscode.workspace.findFiles("**/*.{ject,jt}", "**/node_modules/**");
    const results: vscode.SymbolInformation[] = [];
    const q = query.toLowerCase();

    for (const file of files) {
      try {
        const doc = await vscode.workspace.openTextDocument(file);
        for (let line = 0; line < doc.lineCount; line++) {
          const trimmed = doc.lineAt(line).text.trim();
          const fnMatch     = trimmed.match(/^(?:export\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/);
          const structMatch = trimmed.match(/^struct\s+([A-Za-z_][A-Za-z0-9_]*)/);
          if (fnMatch && (!q || fnMatch[1].toLowerCase().includes(q))) {
            results.push(new vscode.SymbolInformation(fnMatch[1], vscode.SymbolKind.Function, "", new vscode.Location(file, new vscode.Position(line, 0))));
          }
          if (structMatch && (!q || structMatch[1].toLowerCase().includes(q))) {
            results.push(new vscode.SymbolInformation(structMatch[1], vscode.SymbolKind.Struct, "", new vscode.Location(file, new vscode.Position(line, 0))));
          }
        }
      } catch (_) { /* skip unreadable files */ }
    }
    return results;
  }
}

// ─── Folding ─────────────────────────────────────────────────────────────────

class JectFoldingProvider implements vscode.FoldingRangeProvider {
  provideFoldingRanges(document: vscode.TextDocument): vscode.FoldingRange[] {
    const ranges: vscode.FoldingRange[] = [];
    const stack: Array<{ line: number; isRegion: boolean }> = [];

    for (let line = 0; line < document.lineCount; line++) {
      const trimmed = document.lineAt(line).text.trim();

      if (/^#\s*region\b/i.test(trimmed)) {
        stack.push({ line, isRegion: true });
      } else if (/^#\s*endregion\b/i.test(trimmed)) {
        const top = stack.findIndex(s => s.isRegion);
        if (top >= 0) {
          ranges.push(new vscode.FoldingRange(stack[top].line, line, vscode.FoldingRangeKind.Region));
          stack.splice(top, 1);
        }
      } else if (opensBlock(trimmed)) {
        stack.push({ line, isRegion: false });
      } else if (/^end\b/.test(trimmed)) {
        const top = stack.findLastIndex(s => !s.isRegion);
        if (top >= 0 && line > stack[top].line + 1) {
          ranges.push(new vscode.FoldingRange(stack[top].line, line));
          stack.splice(top, 1);
        }
      }
    }
    return ranges;
  }
}

// ─── Code actions ────────────────────────────────────────────────────────────

class JectCodeActionProvider implements vscode.CodeActionProvider {
  provideCodeActions(document: vscode.TextDocument, range: vscode.Range): vscode.CodeAction[] {
    const actions: vscode.CodeAction[] = [];
    const lineText = document.lineAt(range.start.line).text;
    const selected = document.getText(range).trim();

    // Wrap selection in print
    if (selected && !selected.includes("\n")) {
      const action = new vscode.CodeAction("Wrap in print", vscode.CodeActionKind.RefactorExtract);
      const edit = new vscode.WorkspaceEdit();
      edit.replace(document.uri, range, `print ${selected}`);
      action.edit = edit;
      actions.push(action);
    }

    // Suggest `let` for bare assignment (e.g. `x = 5` at start of line)
    const bareAssign = lineText.match(/^(\s*)([A-Za-z_][A-Za-z0-9_]*)\s*=\s*[^=]/);
    if (bareAssign && !/^\s*let\s/.test(lineText) && !/^\s*#/.test(lineText)) {
      const action = new vscode.CodeAction("Add `let` declaration", vscode.CodeActionKind.QuickFix);
      const edit = new vscode.WorkspaceEdit();
      edit.insert(document.uri, new vscode.Position(range.start.line, bareAssign[1].length), "let ");
      action.edit = edit;
      actions.push(action);
    }

    return actions;
  }
}

// ─── Formatter ───────────────────────────────────────────────────────────────

class JectFormattingProvider implements vscode.DocumentFormattingEditProvider {
  provideDocumentFormattingEdits(document: vscode.TextDocument): vscode.TextEdit[] {
    const indentSize = vscode.workspace.getConfiguration("ject").get<number>("format.indentSize", 4);
    const formatted = formatJect(document.getText(), indentSize);
    const fullRange = new vscode.Range(document.positionAt(0), document.positionAt(document.getText().length));
    return [vscode.TextEdit.replace(fullRange, formatted)];
  }
}

function formatJect(source: string, indentSize: number): string {
  const lines = source.replace(/\r\n/g, "\n").split("\n");
  let indent = 0;
  const result: string[] = [];

  for (const rawLine of lines) {
    const trimmed = rawLine.trim();
    if (!trimmed) { result.push(""); continue; }
    if (/^(end|elseif\b|else\b|catch\b)/.test(trimmed)) { indent = Math.max(0, indent - 1); }
    result.push(`${" ".repeat(indent * indentSize)}${trimmed}`);
    if (opensBlock(trimmed)) { indent++; }
  }

  return result.join("\n");
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function opensBlock(trimmed: string): boolean {
  if (/^(end|else\b|elseif\b|catch\b)/.test(trimmed)) {
    return /^(else\b|elseif\b|catch\b)/.test(trimmed);
  }
  return /^(export\s+)?fn\b/.test(trimmed)
    || /^if\b/.test(trimmed)
    || /^while\b/.test(trimmed)
    || /^for\b/.test(trimmed)
    || /^try\b/.test(trimmed)
    || (/^struct\b/.test(trimmed) && !trimmed.includes("}"));
}

interface DocSymbolCache {
  functions: Array<{ name: string; line: number; params: string[] }>;
  variables: Array<{ name: string; line: number }>;
  structs:   Array<{ name: string; line: number }>;
}

function collectDocumentSymbols(document: vscode.TextDocument): DocSymbolCache {
  const result: DocSymbolCache = { functions: [], variables: [], structs: [] };
  for (let line = 0; line < document.lineCount; line++) {
    const trimmed = document.lineAt(line).text.trim();
    const fnMatch     = trimmed.match(/^(?:export\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)/);
    const letMatch    = trimmed.match(/^let\s+([A-Za-z_][A-Za-z0-9_]*)\s*=/);
    const structMatch = trimmed.match(/^struct\s+([A-Za-z_][A-Za-z0-9_]*)/);
    if (fnMatch) {
      const params = fnMatch[2].split(",").map(p => p.trim().split("=")[0].trim()).filter(Boolean);
      result.functions.push({ name: fnMatch[1], line, params });
    }
    if (letMatch)    { result.variables.push({ name: letMatch[1], line }); }
    if (structMatch) { result.structs.push({ name: structMatch[1], line }); }
  }
  return result;
}

/** Resolve `import "X" as alias` → returns the module name for the given alias. */
function resolveAlias(document: vscode.TextDocument, alias: string): string | undefined {
  for (let line = 0; line < document.lineCount; line++) {
    const trimmed = document.lineAt(line).text.trim();
    const m = trimmed.match(/^import\s+"([^"]+)"\s+as\s+(\w+)/);
    if (m && m[2] === alias) { return path.basename(m[1]); }
  }
  return undefined;
}

function parseDiagnostics(output: string, document: vscode.TextDocument): vscode.Diagnostic[] {
  const diagnostics: vscode.Diagnostic[] = [];
  const chunks = output.split(/\n(?=(?:Error|Warning|Runtime Error|Parse error|Parse Error)\b)/i);
  for (const chunk of chunks) {
    const severity    = /warning/i.test(chunk) ? vscode.DiagnosticSeverity.Warning : vscode.DiagnosticSeverity.Error;
    const lineMatch   = chunk.match(/line\s+(\d+)(?:,\s*column\s+(\d+))?/i);
    const codeMatch   = chunk.match(/\b([EW]\d{4})\b/);
    const messageLine = chunk.split(/\r?\n/).find(l => /(error|warning)/i.test(l))?.replace(/\x1b\[[0-9;]*m/g, "").trim();
    if (!lineMatch && !messageLine) { continue; }
    const line    = Math.max(0, Number(lineMatch?.[1] ?? 1) - 1);
    const column  = Math.max(0, Number(lineMatch?.[2] ?? 1) - 1);
    const endLine = document.lineAt(Math.min(line, document.lineCount - 1));
    const diagRange = new vscode.Range(line, Math.min(column, endLine.range.end.character), line, endLine.range.end.character);
    const diag = new vscode.Diagnostic(diagRange, messageLine ?? chunk.trim(), severity);
    diag.source = "ject";
    diag.code = codeMatch?.[1];
    diagnostics.push(diag);
  }
  return diagnostics;
}

async function getTargetDocument(uri?: vscode.Uri): Promise<vscode.TextDocument | undefined> {
  if (uri) {
    const doc = await vscode.workspace.openTextDocument(uri);
    if (doc.languageId === "ject" || [".ject", ".jt"].includes(path.extname(doc.uri.fsPath))) { return doc; }
  }
  const editor = vscode.window.activeTextEditor;
  if (editor?.document.languageId === "ject") { return editor.document; }
  vscode.window.showWarningMessage("Open a Ject file first.");
  return undefined;
}

async function resolveJectExecutable(): Promise<string> {
  const config = vscode.workspace.getConfiguration("ject");
  const root = workspaceRoot();
  if (config.get<boolean>("workspaceExecutable", true) && root) {
    for (const candidate of [
      path.join(root, "target", "release", executableName("ject")),
      path.join(root, "target", "debug",   executableName("ject")),
    ]) {
      if (fs.existsSync(candidate)) { return candidate; }
    }
  }
  return config.get<string>("executablePath", "ject");
}

function workspaceRoot(): string | undefined {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}

function getRunTerminal(): vscode.Terminal {
  runTerminal = runTerminal ?? vscode.window.createTerminal({ name: "Ject", cwd: workspaceRoot() });
  return runTerminal;
}

function executableName(base: string): string {
  return process.platform === "win32" ? `${base}.exe` : base;
}

function quote(value: string): string {
  return process.platform === "win32"
    ? `"${value.replace(/"/g, '\\"')}"`
    : `'${value.replace(/'/g, "'\\''")}' `;
}

function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function snippetCompletions(): vscode.CompletionItem[] {
  const defs: Array<[string, string, string]> = [
    ["fn block",        "fn ${1:name}(${2:params})\n    $0\nend",                              "Function definition"],
    ["fn with return",  "fn ${1:name}(${2:params})\n    return $0\nend",                       "Function with return"],
    ["if block",        "if ${1:condition}\n    $0\nend",                                      "If block"],
    ["if/else",         "if ${1:condition}\n    ${2}\nelse\n    $0\nend",                       "If/else block"],
    ["if/elseif/else",  "if ${1:cond1}\n    ${2}\nelseif ${3:cond2}\n    ${4}\nelse\n    $0\nend", "Full if chain"],
    ["for loop",        "for ${1:item} in ${2:items} do\n    $0\nend",                         "For loop"],
    ["for range",       "for ${1:i} in ${2:0}..${3:10} do\n    $0\nend",                      "For range loop"],
    ["while loop",      "while ${1:condition} do\n    $0\nend",                                "While loop"],
    ["try/catch",       "try\n    ${1}\ncatch ${2:err}\n    $0\nend",                          "Try/catch"],
    ["struct def",      "struct ${1:Name} {\n    ${2:field}\n}",                               "Struct definition"],
    ["struct new",      "let ${1:name} = new ${2:Type} { ${3:field}: $0 }",                   "Struct instantiation"],
    ["import as",       "import \"${1:module}\" as ${2:alias}",                                "Import with alias"],
    ["import from",     "import {${1:items}} from \"${2:module}\"",                            "Selective import"],
    ["export fn",       "export fn ${1:name}(${2:params})\n    $0\nend",                       "Export function"],
    ["unique array",    "{|${1:items}|}",                                                       "Unique array literal"],
    ["named slice",     "${1:arr}[from:${2:0} to:${3:end}]",                                  "Named slice"],
    ["map lambda",      "map(${1:arr}, fn(${2:x}) -> $0)",                                    "Map with lambda"],
    ["filter lambda",   "filter(${1:arr}, fn(${2:x}) -> $0)",                                 "Filter with lambda"],
    ["reduce lambda",   "reduce(${1:arr}, fn(${2:acc}, ${3:x}) -> $0, ${4:0})",               "Reduce with lambda"],
    ["string interp",   "\"${1:text} \\${${2:expr}}\"",                                        "String interpolation"],
    ["file read",       "let ${1:content} = read_file(\"${2:path}\")",                        "Read file"],
    ["file write",      "write_file(\"${1:path}\", ${2:content})",                             "Write file"],
    ["json parse",      "let ${1:data} = parse_json(${2:str})",                               "Parse JSON"],
    ["json write",      "write_file(\"${1:path}\", to_json(${2:value}))",                     "Write JSON"],
  ];

  return defs.map(([label, body, detail]) => {
    const item = new vscode.CompletionItem(label, vscode.CompletionItemKind.Snippet);
    item.insertText = new vscode.SnippetString(body);
    item.detail = detail;
    item.sortText = `z_${label}`;
    return item;
  });
}