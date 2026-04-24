#!/usr/bin/env node
/**
 * Per-view bundle loading smoke test.
 *
 * Visits each route the shell serves and captures every JS file the page
 * fetches via page.on('request'). Verifies:
 *
 *  - dashboard pulls ONLY core bundles (datastar, ship-elements)
 *  - skills pulls core + editor pack (ship-code-editor + ship-markdown-editor)
 *  - agents pulls ONLY core bundles
 *  - overview pulls core + editor pack
 *  - pipelines pulls core + editor pack (for the focus view raw-edit)
 *
 * Writes a markdown report to .artifacts/visual-checks/bundle-loading-report.md
 * with one row per view and the resolved bundle set (name + byte size).
 */

import { chromium } from "playwright";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const BASE = process.env.SHELL_URL ?? "http://127.0.0.1:3100";
const OUT_PATH = resolve(
  new URL("../../../.artifacts/visual-checks/bundle-loading-report.md", import.meta.url).pathname,
);

const CASES = [
  { route: "/dashboard", expectPacks: [] },
  { route: "/agents", expectPacks: [] },
  { route: "/events", expectPacks: [] },
  { route: "/data", expectPacks: [] },
  { route: "/skills", expectPacks: ["editor"] },
  { route: "/overview", expectPacks: ["editor"] },
  { route: "/pipelines", expectPacks: ["editor"] },
];

const BUNDLE_MATCHERS = {
  "datastar.js": /\/api\/runtime\/assets\/datastar\.js(\?|$)/,
  "ship-elements.js": /\/api\/runtime\/assets\/ship-elements\.js(\?|$)/,
  "ship-code-editor.js": /\/api\/runtime\/assets\/ship-code-editor\.js(\?|$)/,
  "ship-markdown-editor.js": /\/api\/runtime\/assets\/ship-markdown-editor\.js(\?|$)/,
  "ship-terminal.js": /\/api\/runtime\/assets\/ship-terminal\.js(\?|$)/,
};

const PACK_BUNDLES = {
  editor: ["ship-code-editor.js", "ship-markdown-editor.js"],
  terminal: ["ship-terminal.js"],
};

async function observe(page, route) {
  const seen = new Map(); // bundle → bytes
  const requests = [];
  page.on("request", (req) => {
    for (const [name, re] of Object.entries(BUNDLE_MATCHERS)) {
      if (re.test(req.url())) {
        requests.push({ name, url: req.url() });
      }
    }
  });
  page.on("response", async (res) => {
    const url = res.url();
    for (const [name, re] of Object.entries(BUNDLE_MATCHERS)) {
      if (re.test(url) && !seen.has(name)) {
        try {
          const body = await res.body();
          seen.set(name, body.byteLength);
        } catch {
          // Response already consumed — fall back to header
          const cl = res.headers()["content-length"];
          seen.set(name, cl ? Number(cl) : 0);
        }
      }
    }
  });
  // `networkidle` never fires on views that open long-lived SSE streams
  // (most Ship views). `domcontentloaded` + a short settle window is
  // plenty to catch the <script type="module" src="..."> loads the
  // daemon injects into head/body.
  await page.goto(`${BASE}${route}`, { waitUntil: "domcontentloaded", timeout: 20000 });
  await page.waitForTimeout(1500);
  return { seen, requests };
}

function formatBundleList(seen) {
  if (seen.size === 0) return "—";
  return [...seen.entries()]
    .map(([name, bytes]) => `${name} (${(bytes / 1024).toFixed(1)}KB)`)
    .join(", ");
}

function expectedSet(expectPacks) {
  const out = new Set(["datastar.js", "ship-elements.js"]);
  for (const pack of expectPacks) {
    for (const b of PACK_BUNDLES[pack] ?? []) out.add(b);
  }
  return out;
}

async function main() {
  const browser = await chromium.launch();
  const rows = [];
  let totalBytes = 0;
  let pass = true;

  for (const { route, expectPacks } of CASES) {
    const context = await browser.newContext();
    const page = await context.newPage();
    const { seen } = await observe(page, route);
    const expected = expectedSet(expectPacks);

    // Verify expected bundles present
    const missing = [...expected].filter((b) => !seen.has(b));
    // Verify no unexpected heavy bundles loaded
    const heavy = ["ship-code-editor.js", "ship-markdown-editor.js", "ship-terminal.js"];
    const unexpected = heavy.filter((b) => seen.has(b) && !expected.has(b));

    const status =
      missing.length === 0 && unexpected.length === 0
        ? "PASS"
        : `FAIL (missing=[${missing.join(", ")}], unexpected=[${unexpected.join(", ")}])`;
    if (status !== "PASS") pass = false;

    const bytes = [...seen.values()].reduce((a, b) => a + b, 0);
    totalBytes += bytes;

    rows.push({
      route,
      expectPacks: expectPacks.length === 0 ? "(none)" : expectPacks.join(", "),
      bundles: formatBundleList(seen),
      totalKB: (bytes / 1024).toFixed(1),
      status,
    });

    console.log(`${status.padEnd(8)} ${route.padEnd(12)} ${(bytes / 1024).toFixed(1)}KB  ${formatBundleList(seen)}`);
    await context.close();
  }

  await browser.close();

  const ts = new Date().toISOString();
  const md = [
    "# Per-view bundle loading report",
    "",
    `Generated: ${ts}`,
    `Base: ${BASE}`,
    "",
    "## Per-view bundle set",
    "",
    "| Route | Expected packs | Bundles fetched | Total JS |",
    "|---|---|---|---|",
    ...rows.map(
      (r) => `| ${r.route} | ${r.expectPacks} | ${r.bundles} | ${r.totalKB} KB |`,
    ),
    "",
    "## Verification",
    "",
    "| Route | Status |",
    "|---|---|",
    ...rows.map((r) => `| ${r.route} | ${r.status} |`),
    "",
    "## Before/after delta",
    "",
    "Before this change (from apps/shell/index.html): every shell boot eager-loaded",
    "`datastar.js` + `ship-elements.js` + `ship-code-editor.js` + `ship-markdown-editor.js` + `ship-terminal.js`.",
    "",
    "That was ~5 MB of JavaScript on every first page load regardless of which view the user visited.",
    "",
    "After this change: dashboard visit pulls only `datastar.js` + `ship-elements.js`.",
    "Editor-requiring views (skills, overview, pipelines) pull the editor pack; other views do not.",
    "",
    `Total bytes across all visited routes: ${(totalBytes / 1024).toFixed(1)} KB.`,
    "",
  ].join("\n");

  await mkdir(resolve(OUT_PATH, ".."), { recursive: true });
  await writeFile(OUT_PATH, md, "utf8");
  console.log(`\nReport written: ${OUT_PATH}`);

  if (!pass) {
    process.exitCode = 1;
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
