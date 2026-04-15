#!/usr/bin/env node
import { chromium } from "playwright";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const exec = promisify(execFile);
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const page = await ctx.newPage();

await page.goto("http://127.0.0.1:3000/studio", { waitUntil: "load" });
// Wait for React to actually render — either pair UI or app shell.
await page.waitForFunction(
  () => {
    const t = document.body.innerText || "";
    return /Pair this browser|ship[- ]dashboard|pipelines|agents|skills/i.test(t);
  },
  { timeout: 15000 },
).catch(() => {});
for (let i = 0; i < 20; i++) {
  await page.waitForTimeout(300);
  const t = await page.evaluate(() => document.body.innerText || "");
  // Real pair codes mix letters+digits (3-3). Require at least one digit
  // in each half so we don't false-match "WEB-LANE" and the like.
  const candidates = [...t.matchAll(/\b([A-Z0-9]{3})-([A-Z0-9]{3})\b/g)];
  const m = candidates.find((c) => /\d/.test(c[1]) && /\d/.test(c[2]));
  if (m) {
    console.log(`found code after ${i * 300}ms: ${m[0]}`);
    try {
      const r = await exec("/home/madvib/dev/ship/target/debug/ship", ["auth", "pair", m[0]]);
      console.log("pair stdout:", r.stdout);
      console.log("pair stderr:", r.stderr);
    } catch (e) {
      console.log("pair failed:", e.message, e.stderr);
    }
    break;
  }
  if (!/Pair this browser/i.test(t)) {
    console.log(`no pair UI after ${i * 300}ms — paired or elsewhere`);
    console.log("body sample:", t.slice(0, 200));
    break;
  }
}

await page.waitForTimeout(3000);
await page.screenshot({ path: "findings-evidence/parity/_debug-after-pair.png", fullPage: true });
await browser.close();
