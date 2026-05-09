import * as fs from "fs";
import * as path from "path";
import * as vscode from "vscode";

export function workspaceRoot(): string | undefined {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}

export async function resolveJectExecutable(): Promise<string> {
  const config = vscode.workspace.getConfiguration("ject");
  const root = workspaceRoot();
  if (config.get<boolean>("workspaceExecutable", true) && root) {
    for (const candidate of [
      path.join(root, "target", "release", executableName("ject")),
      path.join(root, "target", "debug", executableName("ject")),
    ]) {
      if (fs.existsSync(candidate)) {
        return candidate;
      }
    }
  }
  return config.get<string>("executablePath", "ject");
}

export function executableName(base: string): string {
  return process.platform === "win32" ? `${base}.exe` : base;
}

export function quoteShellArg(value: string): string {
  return process.platform === "win32"
    ? `"${value.replace(/"/g, '\\"')}"`
    : `'${value.replace(/'/g, "'\\''")}'`;
}
