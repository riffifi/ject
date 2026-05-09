import * as cp from "child_process";
import * as path from "path";
import * as vscode from "vscode";
import { resolveJectExecutable, workspaceRoot } from "./jectExec";

/** File basename looks like a Ject test file (convention-based discovery). */
export function isJectTestFile(uri: vscode.Uri): boolean {
  const base = path.basename(uri.fsPath);
  return /\.(ject|jt)$/.test(base) && (base.includes("_test.") || /^test_/.test(base));
}

export function registerJectTesting(context: vscode.ExtensionContext): void {
  const controller = vscode.tests.createTestController("ject", "Ject");
  context.subscriptions.push(controller);

  const scanWorkspace = async (): Promise<void> => {
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

  context.subscriptions.push(
    vscode.workspace.onDidChangeWorkspaceFolders(() => {
      void scanWorkspace();
    }),
  );

  void scanWorkspace();

  const run = async (request: vscode.TestRunRequest, token: vscode.CancellationToken): Promise<void> => {
    const runInstance = controller.createTestRun(request);
    const queue: vscode.TestItem[] = [];
    const excluded = request.exclude ?? [];

    const collect = (item: vscode.TestItem): void => {
      if (excluded.includes(item)) {
        return;
      }
      if (item.children.size > 0) {
        item.children.forEach((c) => collect(c));
      } else {
        queue.push(item);
      }
    };

    const roots = request.include?.length ? [...request.include] : [];
    if (roots.length === 0) {
      controller.items.forEach((item) => collect(item));
    } else {
      for (const item of roots) {
        collect(item);
      }
    }

    const executable = await resolveJectExecutable();
    const cwd = workspaceRoot();

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
        await new Promise<void>((resolve) => {
          const proc = cp.execFile(
            executable,
            ["--test", uri.fsPath],
            { cwd, timeout: 120_000, maxBuffer: 10 * 1024 * 1024 },
            (error, stdout, stderr) => {
              const out = `${stderr}\n${stdout}`.trim();
              if (error) {
                const msg = out || error.message || "Test failed";
                runInstance.failed(item, new vscode.TestMessage(msg));
              } else {
                runInstance.passed(item);
              }
              resolve();
            },
          );
          token.onCancellationRequested(() => proc.kill());
        });
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        runInstance.failed(item, new vscode.TestMessage(msg));
      }
    }

    runInstance.end();
  };

  context.subscriptions.push(
    controller.createRunProfile("Run tests", vscode.TestRunProfileKind.Run, run, true),
  );
}
