import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

if (process.argv.length !== 3) {
    throw new Error("Usage: node scripts/check-performance.mjs JECT_BINARY");
}
const executable = resolve(process.argv[2]);
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const scratch = mkdtempSync(join(tmpdir(), "ject-perf-"));

function check(label, args, budgetMs, validOutput) {
    const samples = [];
    for (let run = 0; run < 7; run++) {
        const start = process.hrtime.bigint();
        const result = spawnSync(executable, args, {
            cwd: scratch,
            encoding: "utf8",
            timeout: 10_000,
        });
        const elapsed = Number(process.hrtime.bigint() - start) / 1e6;
        if (result.status !== 0 || !validOutput(result.stdout)) {
            throw new Error(`${label} failed: ${result.error ?? result.stderr ?? result.stdout}`);
        }
        if (run >= 2) samples.push(elapsed);
    }
    samples.sort((left, right) => left - right);
    const median = samples[2];
    console.log(`${label}: ${median.toFixed(2)} ms median (budget ${budgetMs} ms)`);
    if (median > budgetMs) throw new Error(`${label} exceeded its release budget`);
}

try {
    const input = join(scratch, "access.log");
    writeFileSync(input, "INFO request served\n".repeat(40000));
    const logcount = join(root, "examples", "logcount", "main.ject");
    check("CLI startup", ["--version"], 100, (output) => /^ject [0-9]/.test(output));
    check("Count 40,000 log lines", [logcount, input], 250,
        (output) => output.endsWith("40000\ttotal\n"));
} finally {
    rmSync(scratch, { recursive: true, force: true });
}
