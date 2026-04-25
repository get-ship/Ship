import { chromium } from "playwright";
const b = await chromium.launch();
const p = await b.newPage({ viewport: { width: 1280, height: 800 } });
await p.goto("http://127.0.0.1:3100/", { waitUntil: "domcontentloaded" });
await p.waitForTimeout(2500);
const initial = await p.evaluate(() => ({
  htmlDataTheme: document.documentElement.dataset.theme,
  htmlClass: document.documentElement.className,
  bodyBg: getComputedStyle(document.body).backgroundColor,
  paperRuleMatches: (() => {
    // Check if any stylesheet has a rule for [data-theme='paper']
    for (const s of document.styleSheets) {
      try {
        for (const r of s.cssRules) {
          if (r.selectorText?.includes("data-theme='paper'") || r.selectorText?.includes('data-theme="paper"')) return s.href || "inline";
        }
      } catch (e) {}
    }
    return null;
  })(),
}));
console.log("INITIAL:", JSON.stringify(initial, null, 2));

// Try clicking paper theme via applyTheme()
await p.evaluate(() => {
  document.documentElement.dataset.theme = "paper";
  document.documentElement.classList.remove("dark");
});
await p.waitForTimeout(300);
const afterPaper = await p.evaluate(() => ({
  htmlDataTheme: document.documentElement.dataset.theme,
  htmlClass: document.documentElement.className,
  bodyBg: getComputedStyle(document.body).backgroundColor,
  fgColor: getComputedStyle(document.body).color,
}));
console.log("\nAFTER setting data-theme=paper:", JSON.stringify(afterPaper, null, 2));

// Try halo
await p.evaluate(() => {
  document.documentElement.dataset.theme = "halo";
  document.documentElement.classList.remove("dark");
});
await p.waitForTimeout(300);
const afterHalo = await p.evaluate(() => ({
  htmlDataTheme: document.documentElement.dataset.theme,
  bodyBg: getComputedStyle(document.body).backgroundColor,
}));
console.log("\nAFTER setting data-theme=halo:", JSON.stringify(afterHalo, null, 2));
await b.close();
