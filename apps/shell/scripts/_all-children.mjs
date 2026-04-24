import { chromium } from "playwright";
const b = await chromium.launch();
const p = await b.newPage({ viewport: { width: 1280, height: 800 } });
await p.goto("http://127.0.0.1:3100/", { waitUntil: "domcontentloaded" });
await p.waitForTimeout(2500);
await p.locator("#toggle-drawer").click();
await p.waitForTimeout(1500);
const out = await p.evaluate(() => {
  const children = [];
  for (const el of document.body.children) {
    if (el.tagName === "SHIP-SHELL") {
      for (const c of el.children) {
        const cs = getComputedStyle(c);
        const r = c.getBoundingClientRect();
        children.push({
          tag: c.tagName.toLowerCase(),
          id: c.id || null,
          cls: c.className?.toString?.() || null,
          display: cs.display,
          position: cs.position,
          gc: cs.gridColumnStart + "/" + cs.gridColumnEnd,
          gr: cs.gridRowStart + "/" + cs.gridRowEnd,
          x: Math.round(r.x),
          y: Math.round(r.y),
          w: Math.round(r.width),
          h: Math.round(r.height),
        });
      }
    }
  }
  return children;
});
console.log(JSON.stringify(out, null, 2));
await b.close();
