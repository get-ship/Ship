#!/usr/bin/env node
/**
 * Click-test every interactive thing in the shell and see what fires.
 * Captures console logs, network calls, and whether the DOM actually changed.
 */
import { chromium } from "playwright";
import { mkdir, writeFile } from "node:fs/promises";

const BASE = process.env.SHELL_URL ?? "http://127.0.0.1:3100";
const OUT = "findings-evidence/click-test.json";

async function main() {
  const browser = await chromium.launch();
  const ctx = await browser.newContext();
  const page = await ctx.newPage();

  const logs = [];
  const net = [];
  page.on("console", (msg) => logs.push({ type: msg.type(), text: msg.text() }));
  page.on("pageerror", (err) => logs.push({ type: "pageerror", text: err.message }));
  page.on("request", (req) =>
    net.push({ method: req.method(), url: req.url() }),
  );

  await page.goto(`${BASE}/pipelines`, { waitUntil: "load" });
  await page.waitForTimeout(800);

  // Enumerate clickable things inside the mounted view
  const clickables = await page.evaluate(() => {
    const sel =
      'main ship-button, main button, main [data-on-click], main a[href], main [onclick]';
    return Array.from(document.querySelectorAll(sel)).slice(0, 10).map((el, i) => ({
      i,
      tag: el.tagName.toLowerCase(),
      text: (el.textContent || "").trim().slice(0, 40),
      hasDataOn: el.hasAttribute("data-on-click"),
      href: el.getAttribute("href") ?? null,
      selector: `main ${el.tagName.toLowerCase()}:nth-of-type(${i + 1})`,
    }));
  });

  console.log(`Found ${clickables.length} clickable things in /pipelines`);

  const clickResults = [];
  for (const c of clickables.slice(0, 5)) {
    const netBefore = net.length;
    const logsBefore = logs.length;
    const urlBefore = page.url();
    const domBefore = await page.evaluate(
      () => document.querySelector("main")?.innerHTML.length ?? 0,
    );

    try {
      // Click by index in a way that hits a visible one
      await page.evaluate((sel) => {
        const els = document.querySelectorAll(
          "main ship-button, main button, main [data-on-click], main a[href], main [onclick]",
        );
        els[sel.i]?.click();
      }, c);
      await page.waitForTimeout(500);
    } catch (e) {
      clickResults.push({ ...c, error: String(e) });
      continue;
    }

    const netAfter = net.length;
    const logsAfter = logs.length;
    const urlAfter = page.url();
    const domAfter = await page.evaluate(
      () => document.querySelector("main")?.innerHTML.length ?? 0,
    );

    clickResults.push({
      ...c,
      newRequests: net.slice(netBefore),
      newLogs: logs.slice(logsBefore),
      urlChanged: urlBefore !== urlAfter,
      urlAfter,
      domDelta: domAfter - domBefore,
    });
  }

  await mkdir("findings-evidence", { recursive: true });
  await writeFile(OUT, JSON.stringify({ clickables, clickResults, logs, net: net.slice(-20) }, null, 2));

  console.log("\n=== Click results ===");
  for (const r of clickResults) {
    const fired = (r.newRequests?.length ?? 0) > 0 || r.urlChanged || (r.domDelta ?? 0) !== 0;
    console.log(
      `  [${fired ? "✓" : "✗"}] <${r.tag}> "${r.text.padEnd(30)}"  reqs=${r.newRequests?.length ?? 0}  dom=${r.domDelta ?? 0}  url=${r.urlChanged ? "CHANGED" : "same"}`,
    );
    if (r.newRequests?.length)
      for (const req of r.newRequests) console.log(`      → ${req.method} ${req.url}`);
    if (r.newLogs?.length)
      for (const l of r.newLogs) console.log(`      log: ${l.type} ${l.text.slice(0, 80)}`);
  }

  console.log("\n=== All console logs ===");
  for (const l of logs.slice(-15)) console.log(`  [${l.type}] ${l.text.slice(0, 120)}`);

  await browser.close();
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
