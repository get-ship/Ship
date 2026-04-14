/**
 * Capture POC evidence screenshots and one JSON report.
 * Run: node scripts/capture-evidence.mjs
 * Requires: shell dev server on :3100, daemon on $VITE_DAEMON_URL.
 */
import { chromium } from "playwright";
import { writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const OUT = resolve(__dirname, "..", "findings-evidence");
const SHELL = process.env.SHELL_URL ?? "http://127.0.0.1:3100";

const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1200, height: 800 } });
const page = await ctx.newPage();

const consoleLog = [];
page.on("console", (m) => consoleLog.push(`[${m.type()}] ${m.text()}`));
page.on("pageerror", (e) => consoleLog.push(`[pageerror] ${e.message}`));

async function snap(name, path, afterLoad) {
  await page.goto(`${SHELL}${path}`, { waitUntil: "networkidle" });
  if (afterLoad) await afterLoad();
  await page.screenshot({ path: `${OUT}/${name}.png`, fullPage: false });
  console.log(`captured ${name}`);
}

await snap("01-home-fragment-mount", "/");
await snap("02-pipelines-route", "/pipelines");
await snap("03-chat-sse", "/chat", async () => {
  // Simulate an SSE message arriving into the mounted fragment.
  // The real daemon endpoint doesn't stream in this env (see findings §3).
  await page.evaluate(() => {
    const el = document.getElementById("sse-target");
    if (el)
      el.insertAdjacentHTML(
        "afterbegin",
        "<li>[simulated SSE message — daemon stream auth pending]</li>",
      );
  });
  await page.waitForTimeout(200);
});
await snap("04-form-fragment", "/form");
await snap("05-multi-mount-drawer", "/", async () => {
  await page.click("#toggle-drawer");
  await page.waitForSelector("aside#drawer ship-card", { timeout: 5000 }).catch(() => {});
  await page.waitForTimeout(300);
});

// Verify: custom element upgraded (shadow root exists when ship-elements hydrated).
const hydration = await page.evaluate(() => {
  const cards = document.querySelectorAll("ship-card");
  return Array.from(cards).map((c) => ({
    tag: c.tagName,
    hasShadowRoot: !!c.shadowRoot,
    connected: c.isConnected,
  }));
});

await writeFile(
  `${OUT}/report.json`,
  JSON.stringify({ consoleLog, hydration }, null, 2),
);
console.log("hydration:", hydration);

await browser.close();
