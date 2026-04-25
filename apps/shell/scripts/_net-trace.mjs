import { chromium } from "playwright";
const b = await chromium.launch();
const p = await b.newPage({ viewport: { width: 1280, height: 800 } });
p.on("request", (r) => {
  if (r.url().includes("/api/")) console.log("→", r.method(), r.url());
});
await p.goto("http://127.0.0.1:3100/", { waitUntil: "domcontentloaded" });
await p.waitForTimeout(2500);
console.log("\n=== CLICK DRAWER TOGGLE ===\n");
await p.locator("#toggle-drawer").click();
await p.waitForTimeout(2500);
const children = await p.evaluate(() => {
  const shell = document.querySelector("ship-shell");
  return Array.from(shell.children).map((c) => ({
    tag: c.tagName.toLowerCase(),
    id: c.id,
    cls: c.className,
    x: Math.round(c.getBoundingClientRect().x),
    y: Math.round(c.getBoundingClientRect().y),
    w: Math.round(c.getBoundingClientRect().width),
  }));
});
console.log("\n=== SHELL CHILDREN ===");
console.log(JSON.stringify(children, null, 2));
await b.close();
