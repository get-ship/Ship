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
  await page.goto(`${SHELL}${path}`, { waitUntil: "domcontentloaded" });
  // Give fragment fetch + hydration time; SSE keeps the network busy so we
  // don't wait for networkidle.
  await page.waitForTimeout(1200);
  if (afterLoad) await afterLoad();
  await page.screenshot({ path: `${OUT}/${name}.png`, fullPage: false });
  console.log(`captured ${name}`);
}

await snap("01-agents-view", "/");
await snap("02-pipelines-view", "/pipelines");
await snap("03-chat-view", "/chat");
await snap("04-events-view", "/events");
await snap("05-multi-mount-drawer", "/pipelines", async () => {
  await page.click("#toggle-drawer");
  await page.waitForTimeout(1000);
});

// Verify: custom element upgraded (shadow root exists when ship-elements hydrated).
const hydration = await page.evaluate(() => {
  const tags = ["ship-card", "ship-button", "ship-badge", "ship-list-tile", "ship-tabs"];
  const out = [];
  for (const t of tags) {
    for (const el of document.querySelectorAll(t)) {
      out.push({ tag: el.tagName, hasShadowRoot: !!el.shadowRoot, connected: el.isConnected });
    }
  }
  return out;
});

await writeFile(
  `${OUT}/report.json`,
  JSON.stringify({ consoleLog, hydration }, null, 2),
);
console.log("hydration:", hydration);

await browser.close();
