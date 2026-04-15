#!/usr/bin/env node
import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";

const BASE = process.env.SHELL_URL ?? "http://127.0.0.1:3100";
const OUT = "findings-evidence/diagnose";

async function main() {
  await mkdir(OUT, { recursive: true });
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 1600 } });
  const page = await ctx.newPage();

  await page.goto(`${BASE}/devtools.html`, { waitUntil: "load" });
  await page.waitForTimeout(1500);

  const data = await page.evaluate(() => {
    const cards = Array.from(document.querySelectorAll(".card")).map((card) => {
      const title = card.querySelector(".card-title")?.textContent ?? "?";
      const render = card.querySelector(".card-render");
      const el = render?.firstElementChild;
      const tag = el?.tagName?.toLowerCase() ?? "";
      const bbox = el?.getBoundingClientRect();
      return {
        title,
        tag,
        defined: !!(tag && customElements.get(tag)),
        hasShadow: !!el?.shadowRoot,
        width: Math.round(bbox?.width ?? 0),
        height: Math.round(bbox?.height ?? 0),
        innerLen: el?.shadowRoot?.innerHTML?.length ?? el?.innerHTML?.length ?? 0,
      };
    });
    return {
      total: cards.length,
      defined: cards.filter((c) => c.defined).length,
      hydrated: cards.filter((c) => c.hasShadow).length,
      zeroSize: cards.filter((c) => c.width === 0 || c.height === 0).length,
      cards,
    };
  });

  await page.screenshot({ path: join(OUT, "devtools-light.png"), fullPage: true });

  await ctx.addInitScript(() => localStorage.setItem("ship-shell-theme", "dark"));
  await page.reload({ waitUntil: "load" });
  await page.waitForTimeout(1500);
  await page.screenshot({ path: join(OUT, "devtools-dark.png"), fullPage: true });

  console.log(`cards: ${data.total} total, ${data.defined} defined, ${data.hydrated} with shadow, ${data.zeroSize} zero-size`);
  console.log("\nZero-size cards (likely broken):");
  data.cards.filter((c) => c.width === 0 || c.height === 0).forEach((c) =>
    console.log(`  ${c.title.padEnd(28)} tag=${c.tag} defined=${c.defined} shadow=${c.hasShadow} innerLen=${c.innerLen}`)
  );
  console.log("\nHydrated samples:");
  data.cards.filter((c) => c.hasShadow).slice(0, 10).forEach((c) =>
    console.log(`  ${c.title.padEnd(28)} ${c.width}×${c.height}`)
  );

  await browser.close();
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
