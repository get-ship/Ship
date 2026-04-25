import { chromium } from "playwright";
const b = await chromium.launch();
const p = await b.newPage({ viewport: { width: 1280, height: 800 } });
await p.goto("http://127.0.0.1:3100/", { waitUntil: "domcontentloaded" });
await p.waitForTimeout(2500);
await p.locator("#toggle-drawer").click();
await p.waitForTimeout(1500);
const r = await p.evaluate(() => {
  const vals = {};
  for (const sel of [
    "ship-nav-top",
    "ship-rail",
    "#ship-view-root",
    ".ship-drawer",
    "ship-panel-dock",
    ".ship-statusbar",
  ]) {
    const el = document.querySelector(sel);
    if (!el) {
      vals[sel] = null;
      continue;
    }
    const cs = getComputedStyle(el);
    const r = el.getBoundingClientRect();
    vals[sel] = {
      x: Math.round(r.x),
      w: Math.round(r.width),
      gc: cs.gridColumnStart + " / " + cs.gridColumnEnd,
      gr: cs.gridRowStart + " / " + cs.gridRowEnd,
    };
  }
  return vals;
});
console.log(JSON.stringify(r, null, 2));
await b.close();
