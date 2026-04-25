#!/usr/bin/env node
/**
 * Loop test runner.
 *
 * Discovers every `*.spec.mjs` under `tests/loops/` and runs each as a
 * subprocess. By default skips `*.ui.spec.mjs` (Playwright-driven, slower);
 * pass `--ui` to include them.
 *
 * Each loop is a self-contained sanity gate for one piece of the
 * substrate: it spawns/touches whatever it needs against the running
 * daemon, asserts behaviour, exits 0 (pass) or non-zero (fail).
 *
 *   node tests/loops/run.mjs           # headless only
 *   node tests/loops/run.mjs --ui      # headless + UI
 *   node tests/loops/run.mjs hook      # filter to loops matching "hook"
 *
 * Reads TOKEN from env if present, otherwise from ~/.ship/daemon.token.
 */
import { readdir, readFile, writeFile, mkdir } from "node:fs/promises";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join, basename } from "node:path";
import { homedir } from "node:os";

const __dirname = dirname(fileURLToPath(import.meta.url));
const RESULTS = join(__dirname, "results");

const args = process.argv.slice(2);
const includeUi = args.includes("--ui");
const filter = args.find((a) => !a.startsWith("--"));

async function token() {
  if (process.env.TOKEN) return process.env.TOKEN;
  try {
    return (await readFile(join(homedir(), ".ship/daemon.token"), "utf8")).trim();
  } catch {
    console.error("no TOKEN env and no ~/.ship/daemon.token — start the daemon first");
    process.exit(2);
  }
}

async function discover() {
  const files = await readdir(__dirname);
  return files
    .filter((f) => f.endsWith(".spec.mjs"))
    .filter((f) => includeUi || !f.endsWith(".ui.spec.mjs"))
    .filter((f) => !filter || f.includes(filter))
    .sort();
}

function runOne(file, env) {
  return new Promise((resolve) => {
    const child = spawn(process.execPath, [join(__dirname, file)], {
      env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const start = Date.now();
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (d) => (stdout += d.toString()));
    child.stderr.on("data", (d) => (stderr += d.toString()));
    const killer = setTimeout(() => child.kill("SIGKILL"), 180_000);
    child.on("exit", (code) => {
      clearTimeout(killer);
      resolve({
        file,
        code,
        elapsed_ms: Date.now() - start,
        stdout,
        stderr,
      });
    });
  });
}

const COLORS = {
  pass: "\x1b[32m",
  fail: "\x1b[31m",
  dim: "\x1b[2m",
  reset: "\x1b[0m",
};

async function main() {
  const TOKEN = await token();
  const env = { ...process.env, TOKEN };
  await mkdir(RESULTS, { recursive: true });

  const files = await discover();
  if (files.length === 0) {
    console.error(`no loops matched (filter=${filter ?? "*"} ui=${includeUi})`);
    process.exit(2);
  }

  console.log(`running ${files.length} loop(s)${includeUi ? " (incl. UI)" : ""}\n`);

  const results = [];
  for (const file of files) {
    process.stdout.write(`  ${file.padEnd(36)} `);
    const r = await runOne(file, env);
    const ok = r.code === 0;
    const tag = ok ? `${COLORS.pass}PASS${COLORS.reset}` : `${COLORS.fail}FAIL${COLORS.reset}`;
    const ms = `${COLORS.dim}${(r.elapsed_ms / 1000).toFixed(1)}s${COLORS.reset}`;
    console.log(`${tag}  ${ms}`);
    if (!ok) {
      const summary = (r.stdout + r.stderr).trim().split("\n").slice(-6).join("\n      ");
      console.log(`      ${summary}`);
    }
    results.push(r);
    await writeFile(
      join(RESULTS, basename(file, ".spec.mjs") + ".log"),
      `exit=${r.code}\nelapsed_ms=${r.elapsed_ms}\n--- stdout ---\n${r.stdout}\n--- stderr ---\n${r.stderr}\n`,
    );
  }

  const failed = results.filter((r) => r.code !== 0).length;
  const passed = results.length - failed;
  await writeFile(
    join(RESULTS, "summary.json"),
    JSON.stringify(
      {
        generated_at: new Date().toISOString(),
        total: results.length,
        passed,
        failed,
        ui_mode: includeUi,
        results: results.map((r) => ({
          file: r.file,
          code: r.code,
          elapsed_ms: r.elapsed_ms,
        })),
      },
      null,
      2,
    ),
  );

  console.log(
    `\n${passed}/${results.length} passed${failed ? `, ${failed} failed` : ""}` +
      `  ${COLORS.dim}(logs in tests/loops/results/)${COLORS.reset}`,
  );
  process.exit(failed === 0 ? 0 : 1);
}

main().catch((e) => {
  console.error(e);
  process.exit(2);
});
