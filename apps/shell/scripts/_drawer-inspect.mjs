#!/usr/bin/env node
import { chromium } from "playwright";
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
page.on("pageerror", (e) => console.log("[pageerror]", e.message));
page.on("console", (m) => {
  if (m.type() === "error") console.log("[console err]", m.text());
});
page.on("response", (r) => {
  if (r.status() >= 400) console.log("[net]", r.status(), r.url());
});
await page.goto("http://127.0.0.1:3100/", { waitUntil: "domcontentloaded" });
await page.waitForTimeout(2000);
// Click drawer toggle
await page.locator("#toggle-drawer").click();
await page.waitForTimeout(2500);
const drawerHTML = await page.evaluate(() => {
  const d = document.getElementById("drawer-content");
  return d?.innerHTML?.slice(0, 3000) ?? "(no #drawer-content)";
});
console.log("=== drawer-content innerHTML (first 3000 chars) ===");
console.log(drawerHTML);
await page.screenshot({ path: ".artifacts/web-qa/drawer-open.png" });
await browser.close();
