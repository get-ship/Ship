#!/usr/bin/env node
import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";

const OUT = process.env.OUT ?? "/workspaces/ship/.artifacts/web-qa/devtools-gallery-adhoc";
const URL = process.env.URL ?? "http://127.0.0.1:3100/devtools.html";

async function main() {
  await mkdir(join(OUT, "screenshots"), { recursive: true });
  const browser = await chromium.launch();

  for (const theme of ["light", "dark"]) {
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    const page = await ctx.newPage();

    const errors = [];
    page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
    page.on("console", (m) => {
      if (m.type() === "error") errors.push(`console.error: ${m.text()}`);
    });

    await page.goto(URL, { waitUntil: "networkidle" });
    await page.waitForTimeout(800);

    // Flip to dark via the theme toggle. addInitScript + localStorage
    // lost a race with the inline head script under Playwright, so we
    // reach through the public toggle instead.
    if (theme === "dark") {
      const currentlyDark = await page.evaluate(() =>
        document.documentElement.classList.contains("dark"),
      );
      if (!currentlyDark) {
        await page.click("#toggle-theme");
        await page.waitForTimeout(400);
      }
    }

    // Full-page screenshot
    await page.screenshot({
      path: join(OUT, "screenshots", `gallery-${theme}.png`),
      fullPage: true,
    });

    // Inventory the rendered DOM.
    const info = await page.evaluate(() => {
      const cards = Array.from(document.querySelectorAll(".card"));
      const sections = Array.from(document.querySelectorAll("section.group h2")).map(
        (h) => h.textContent
      );
      let openDialogs = 0;
      for (const dlg of document.querySelectorAll("ship-dialog, ship-alert-dialog, ship-popover, ship-tooltip, ship-dropdown-menu")) {
        const el = dlg;
        if (el.hasAttribute && el.hasAttribute("open")) openDialogs++;
      }
      const countText = document.getElementById("count")?.textContent ?? "";
      return { cards: cards.length, sections, openDialogs, countText };
    });

    // Click the first dialog trigger to verify the button opens a dialog.
    let clickedOpen = false;
    if (theme === "light") {
      const trigger = page
        .locator(".card .card-trigger ship-button")
        .first();
      if (await trigger.count()) {
        await trigger.click();
        await page.waitForTimeout(400);
        await page.screenshot({
          path: join(OUT, "screenshots", `gallery-${theme}-dialog-open.png`),
        });
        // Close via Escape and confirm
        await page.keyboard.press("Escape");
        await page.waitForTimeout(200);
        clickedOpen = true;
      }
    }

    console.log(JSON.stringify({ theme, ...info, errors, clickedOpen }, null, 2));

    await ctx.close();
  }

  await browser.close();
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
