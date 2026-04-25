#!/usr/bin/env node
import { chromium } from "playwright";
const BASE = "http://127.0.0.1:3100";
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
await page.goto(BASE + "/agents", { waitUntil: "domcontentloaded" });
await page.waitForTimeout(2500);
const layout = await page.evaluate(() => {
  const report = [];
  const body = document.body;
  const walk = (el, depth) => {
    const rect = el.getBoundingClientRect?.();
    const cs = getComputedStyle(el);
    if (rect && (rect.width > 0 || rect.height > 0)) {
      report.push({
        depth,
        tag: el.tagName.toLowerCase(),
        id: el.id || undefined,
        cls: el.className?.toString?.() || undefined,
        x: Math.round(rect.x),
        y: Math.round(rect.y),
        w: Math.round(rect.width),
        h: Math.round(rect.height),
        display: cs.display,
        gridColumn: cs.gridColumn,
        gridRow: cs.gridRow,
      });
    }
  };
  walk(body, 0);
  for (const child of body.children) walk(child, 1);
  const shell = document.querySelector("ship-shell");
  if (shell) {
    for (const c of shell.children) walk(c, 2);
  }
  return report;
});
console.log(JSON.stringify(layout, null, 2));
await browser.close();
