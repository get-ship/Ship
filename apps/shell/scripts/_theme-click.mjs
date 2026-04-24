import { chromium } from "playwright";
const b = await chromium.launch();
const p = await b.newPage({ viewport: { width: 1280, height: 800 } });
p.on("console", (m) => {
  if (m.type() === "error") console.log("[err]", m.text());
});
await p.goto("http://127.0.0.1:3100/", { waitUntil: "domcontentloaded" });
await p.waitForTimeout(2500);

// Open the theme picker popover
await p.evaluate(() => {
  const picker = document.getElementById("theme-picker");
  picker?.show?.(); // ship-popover API
});
await p.waitForTimeout(500);

// Find and report all theme cards
const cards = await p.evaluate(() => {
  const cards = document.querySelectorAll("[data-theme-card]");
  return Array.from(cards).map((c) => ({
    id: c.dataset.themeCard,
    visible: !!c.offsetParent,
  }));
});
console.log("CARDS:", JSON.stringify(cards, null, 2));

// Click the paper card
await p.evaluate(() => {
  const card = document.querySelector('[data-theme-card="paper"]');
  card?.click();
});
await p.waitForTimeout(500);
const state = await p.evaluate(() => ({
  htmlDataTheme: document.documentElement.dataset.theme,
  htmlClass: document.documentElement.className,
  bodyBg: getComputedStyle(document.body).backgroundColor,
  statusTheme: document.getElementById("status-theme")?.textContent,
  stored: localStorage.getItem("ship-site-theme"),
}));
console.log("AFTER paper click:", JSON.stringify(state, null, 2));

await b.close();
