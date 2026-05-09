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
exports.workspaceRoot = workspaceRoot;
exports.resolveJectExecutable = resolveJectExecutable;
exports.executableName = executableName;
exports.quoteShellArg = quoteShellArg;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const vscode = __importStar(require("vscode"));
function workspaceRoot() {
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}
async function resolveJectExecutable() {
    const config = vscode.workspace.getConfiguration("ject");
    const root = workspaceRoot();
    if (config.get("workspaceExecutable", true) && root) {
        for (const candidate of [
            path.join(root, "target", "release", executableName("ject")),
            path.join(root, "target", "debug", executableName("ject")),
        ]) {
            if (fs.existsSync(candidate)) {
                return candidate;
            }
        }
    }
    return config.get("executablePath", "ject");
}
function executableName(base) {
    return process.platform === "win32" ? `${base}.exe` : base;
}
function quoteShellArg(value) {
    return process.platform === "win32"
        ? `"${value.replace(/"/g, '\\"')}"`
        : `'${value.replace(/'/g, "'\\''")}'`;
}
//# sourceMappingURL=jectExec.js.map