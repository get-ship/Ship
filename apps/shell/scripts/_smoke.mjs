#!/usr/bin/env node
/**
 * Phase 1 + 2 smoke test. Loads the shell, captures console + network
 * errors per route, screenshots each. Reports a per-route status line.
 *
 * Usage: node .artifacts/web-qa/smoke.mjs <out-dir>
 */
import { chromium } from "playwright";
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const BASE = process.env.SHELL_URL ?? "http://127.0.0.1:3100";
const OUT = process.argv[2] ?? ".artifacts/web-qa/latest";

const ROUTES = [
  "/",
  "/agents",
  "/skills",
  "/pipelines",
  "/settings",
  "/chat",
  "/events",
  "/git",
];

async function run() {
  await mkdir(`${OUT}/screenshots`, { recursive: true });
  await mkdir(`${OUT}/console`, { recursive: true });

  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
  const page = await ctx.newPage();

  const report = [];

  for (const route of ROUTES) {
    const url = BASE + route;
    const logs = [];
    const errors = [];
    const netErrors = [];
    const onConsole = (msg) => {
      const type = msg.type();
      if (type === "error" || type === "warning") {
        logs.push(`[${type}] ${msg.text()}`);
      }
    };
    const onPageError = (err) => errors.push(err.stack ?? String(err));
    const onResponse = (resp) => {
      if (resp.status() >= 400) {
        netErrors.push(`${resp.status()} ${resp.url()}`);
      }
    };

    page.on("console", onConsole);
    page.on("pageerror", onPageError);
    page.on("response", onResponse);

    let ok = true;
    try {
      await page.goto(url, { waitUntil: "domcontentloaded", timeout: 15000 });
      // Give the shell a chance to hydrate + apply any post-mount work.
      await page.waitForTimeout(2500);
    } catch (e) {
      ok = false;
      errors.push(String(e));
    }

    const slug = route === "/" ? "root" : route.replace(/\//g, "-").replace(/^-/, "");
    const shotPath = `${OUT}/screenshots/${slug}.png`;
    try {
      await page.screenshot({ path: shotPath, fullPage: false });
    } catch {
      /* screenshot best-effort */
    }

    const logPath = `${OUT}/console/${slug}.log`;
    await writeFile(
      logPath,
      [
        `URL: ${url}`,
        ``,
        `=== Console (errors/warnings) ===`,
        ...logs,
        ``,
        `=== Page errors ===`,
        ...errors,
        ``,
        `=== Network ≥400 ===`,
        ...netErrors,
      ].join("\n"),
    );

    page.off("console", onConsole);
    page.off("pageerror", onPageError);
    page.off("response", onResponse);

    const status = !ok || errors.length > 0 ? "FAIL" : netErrors.length > 0 ? "WARN" : "OK";
    const line = `${status.padEnd(4)} ${route.padEnd(12)}  errs=${errors.length} warns=${
      logs.filter((l) => l.startsWith("[warning]")).length
    } net=${netErrors.length}`;
    report.push(line);
    console.log(line);
  }

  await writeFile(`${OUT}/report.txt`, report.join("\n") + "\n");
  await browser.close();
}

run().catch((e) => {
  console.error(e);
  process.exit(1);
});
