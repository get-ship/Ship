#!/usr/bin/env node
import { chromium } from "playwright";
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
await page.goto("http://127.0.0.1:3100/", { waitUntil: "domcontentloaded" });
await page.waitForTimeout(2500);
await page.locator("#toggle-drawer").click();
await page.waitForTimeout(2500);
const inspect = await page.evaluate(() => {
  const pick = (sel) => {
    const el = document.querySelector(sel);
    if (!el) return null;
    const r = el.getBoundingClientRect();
    return {
      sel,
      x: Math.round(r.x),
      y: Math.round(r.y),
      w: Math.round(r.width),
      h: Math.round(r.height),
      display: getComputedStyle(el).display,
    };
  };
  return {
    bodyClasses: document.body.className,
    content: pick("#ship-view-root"),
    drawer: pick(".ship-drawer"),
    drawerContent: pick("#drawer-content"),
    rail: pick("ship-rail"),
  };
});
console.log(JSON.stringify(inspect, null, 2));
const contentInner = await page.evaluate(() =>
  document.querySelector("#ship-view-root")?.innerHTML?.slice(0, 400) ?? "",
);
console.log("\n=== #ship-view-root innerHTML (400c) ===\n" + contentInner);
await browser.close();
