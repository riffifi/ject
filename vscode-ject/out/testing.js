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
exports.isJectTestFile = isJectTestFile;
exports.registerJectTesting = registerJectTesting;
const cp = __importStar(require("child_process"));
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
const jectExec_1 = require("./jectExec");
/** File basename looks like a Ject test file (convention-based discovery). */
function isJectTestFile(uri) {
    const base = path.basename(uri.fsPath);
    return /\.(ject|jt)$/.test(base) && (base.includes("_test.") || /^test_/.test(base));
}
function registerJectTesting(context) {
    const controller = vscode.tests.createTestController("ject", "Ject");
    context.subscriptions.push(controller);
    const scanWorkspace = async () => {
        controller.items.replace([]);
        const pattern = "**/*.{ject,jt}";
        const exclude = "**/{node_modules,.git,target}/**";
        const files = await vscode.workspace.findFiles(pattern, exclude);
        for (const uri of files) {
            if (!isJectTestFile(uri)) {
                continue;
            }
            const id = uri.toString();
            const label = vscode.workspace.asRelativePath(uri);
            const item = controller.createTestItem(id, label, uri);
            controller.items.add(item);
        }
    };
    context.subscriptions.push(vscode.workspace.onDidChangeWorkspaceFolders(() => {
        void scanWorkspace();
    }));
    void scanWorkspace();
    const run = async (request, token) => {
        const runInstance = controller.createTestRun(request);
        const queue = [];
        const excluded = request.exclude ?? [];
        const collect = (item) => {
            if (excluded.includes(item)) {
                return;
            }
            if (item.children.size > 0) {
                item.children.forEach((c) => collect(c));
            }
            else {
                queue.push(item);
            }
        };
        const roots = request.include?.length ? [...request.include] : [];
        if (roots.length === 0) {
            controller.items.forEach((item) => collect(item));
        }
        else {
            for (const item of roots) {
                collect(item);
            }
        }
        const executable = await (0, jectExec_1.resolveJectExecutable)();
        const cwd = (0, jectExec_1.workspaceRoot)();
        for (const item of queue) {
            if (token.isCancellationRequested) {
                break;
            }
            const uri = item.uri;
            if (!uri) {
                continue;
            }
            runInstance.started(item);
            try {
                await new Promise((resolve) => {
                    const proc = cp.execFile(executable, ["--test", uri.fsPath], { cwd, timeout: 120_000, maxBuffer: 10 * 1024 * 1024 }, (error, stdout, stderr) => {
                        const out = `${stderr}\n${stdout}`.trim();
                        if (error) {
                            const msg = out || error.message || "Test failed";
                            runInstance.failed(item, new vscode.TestMessage(msg));
                        }
                        else {
                            runInstance.passed(item);
                        }
                        resolve();
                    });
                    token.onCancellationRequested(() => proc.kill());
                });
            }
            catch (e) {
                const msg = e instanceof Error ? e.message : String(e);
                runInstance.failed(item, new vscode.TestMessage(msg));
            }
        }
        runInstance.end();
    };
    context.subscriptions.push(controller.createRunProfile("Run tests", vscode.TestRunProfileKind.Run, run, true));
}
//# sourceMappingURL=testing.js.map