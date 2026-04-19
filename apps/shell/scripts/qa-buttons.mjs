#!/usr/bin/env node
/**
 * QA — Click every button, link, and interactive element across all views.
 * Reports what worked, what errored, and what broke.
 */
import { chromium } from "playwright";
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const BASE = process.env.SHELL_URL ?? "http://127.0.0.1:3100";
const OUT = "qa-evidence/buttons";

async function main() {
  await mkdir(OUT, { recursive: true });
  const browser = await chromium.launch();
  const allResults = [];

  const views = [
    { path: "/dashboard", wait: 3000 },
    { path: "/pipelines", wait: 3000 },
    { path: "/agents", wait: 3000 },
    { path: "/skills", wait: 3000 },
    { path: "/git", wait: 4000 },
    { path: "/data", wait: 3000 },
    { path: "/canvas", wait: 3000 },
    { path: "/settings", wait: 5000 },
    { path: "/events", wait: 3000 },
  ];

  for (const { path, wait } of views) {
    const slug = path.slice(1);
    console.log(`\n═══ ${slug} ═══`);
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    const page = await ctx.newPage();
    const pageErrors = [];
    const consoleErrors = [];
    page.on("pageerror", (err) => pageErrors.push(err.message.slice(0, 200)));
    page.on("console", (msg) => {
      if (msg.type() === "error") consoleErrors.push(msg.text().slice(0, 200));
    });

    await page.goto(`${BASE}${path}`, { waitUntil: "load" });
    await page.waitForTimeout(wait);

    // Discover all interactive elements
    const elements = await page.evaluate(() => {
      const root = document.getElementById("ship-view-root");
      if (!root) return [];
      const els = [];
      const seen = new Set();

      // ship-button, ship-chip, button, a[href], [data-on:click], ship-tab, ship-switch, ship-checkbox
      const selectors = [
        "ship-button",
        "ship-chip",
        "button:not(ship-button):not([disabled])",
        "a[href]:not([href='#']):not([href='javascript:void(0)'])",
        "[data-on\\:click]",
        "ship-tab",
        "ship-switch",
        "ship-checkbox",
        "ship-select",
        "ship-toggle",
      ];

      for (const sel of selectors) {
        for (const el of root.querySelectorAll(sel)) {
          const rect = el.getBoundingClientRect();
          // Skip invisible/zero-size elements
          if (rect.width === 0 || rect.height === 0) continue;
          // Skip elements outside viewport
          if (rect.top > window.innerHeight || rect.bottom < 0) continue;

          const key = `${el.tagName}-${rect.x.toFixed(0)}-${rect.y.toFixed(0)}`;
          if (seen.has(key)) continue;
          seen.add(key);

          const text = el.textContent?.trim()?.slice(0, 40) || el.getAttribute("aria-label") || el.tagName;
          const tag = el.tagName.toLowerCase();
          const disabled = el.hasAttribute("disabled") || el.getAttribute("aria-disabled") === "true";

          els.push({
            tag,
            text,
            x: Math.round(rect.x),
            y: Math.round(rect.y),
            w: Math.round(rect.width),
            h: Math.round(rect.height),
            disabled,
            href: el.getAttribute("href"),
            hasClick: el.hasAttribute("data-on:click") || el.hasAttribute("data-on:click.prevent"),
          });
        }
      }

      return els.sort((a, b) => a.y - b.y || a.x - b.x);
    });

    console.log(`  Found ${elements.length} interactive elements`);

    // Screenshot before clicking
    await page.screenshot({ path: join(OUT, `${slug}-before.png`) });

    const results = [];
    let clickedCount = 0;
    let errorCount = 0;

    for (const el of elements) {
      if (el.disabled) {
        results.push({ ...el, status: "disabled" });
        continue;
      }

      // Skip links that navigate away (to external URLs or the gallery)
      if (el.href && (el.href.startsWith("http") || el.href === "/devtools.html")) {
        results.push({ ...el, status: "skip-external" });
        continue;
      }

      // Skip elements that would navigate to another view (would lose our context)
      if (el.hasClick && (el.text.includes("@get") || el.tag === "a")) {
        // Only skip cross-view navigation, not in-view actions
        const isNavLink = el.href && !el.href.startsWith("#") && !el.href.includes(slug);
        if (isNavLink) {
          results.push({ ...el, status: "skip-nav" });
          continue;
        }
      }

      // Record errors before click
      const errBefore = pageErrors.length + consoleErrors.length;

      try {
        // Click by coordinates (most reliable for shadow DOM elements)
        await page.mouse.click(el.x + el.w / 2, el.y + el.h / 2);
        await page.waitForTimeout(300);
        clickedCount++;

        const errAfter = pageErrors.length + consoleErrors.length;
        if (errAfter > errBefore) {
          const newErrors = [
            ...pageErrors.slice(errBefore),
            ...consoleErrors.slice(errBefore),
          ];
          results.push({ ...el, status: "error", errors: newErrors.map(e => e.slice(0, 100)) });
          errorCount++;
          console.log(`  ✗ ${el.tag} "${el.text}" — ${newErrors[0]?.slice(0, 80)}`);
        } else {
          results.push({ ...el, status: "ok" });
        }
      } catch (err) {
        results.push({ ...el, status: "click-failed", error: err.message.slice(0, 100) });
        errorCount++;
      }
    }

    // Screenshot after all clicks
    await page.screenshot({ path: join(OUT, `${slug}-after.png`) });

    // Check if the view is still intact
    const intact = await page.evaluate(() => {
      const header = document.querySelector("header") || document.querySelector(".ship-topbar");
      const main = document.getElementById("ship-view-root");
      return !!header && !!main;
    });

    const ok = results.filter(r => r.status === "ok").length;
    const disabled = results.filter(r => r.status === "disabled").length;
    const skipped = results.filter(r => r.status.startsWith("skip")).length;
    const errors = results.filter(r => r.status === "error" || r.status === "click-failed").length;

    console.log(`  Clicked: ${clickedCount} | OK: ${ok} | Errors: ${errors} | Disabled: ${disabled} | Skipped: ${skipped} | Shell intact: ${intact}`);

    allResults.push({
      view: slug,
      total: elements.length,
      clicked: clickedCount,
      ok,
      errors,
      disabled,
      skipped,
      intact,
      details: results,
      consoleErrors: consoleErrors,
      pageErrors: pageErrors,
    });

    await ctx.close();
  }

  // ── Summary ──
  console.log("\n═══ SUMMARY ═══");
  let totalClicked = 0, totalOk = 0, totalErrors = 0;
  for (const r of allResults) {
    const status = r.errors === 0 ? "✓" : "✗";
    console.log(`${status} ${r.view.padEnd(12)} elements=${r.total} clicked=${r.clicked} ok=${r.ok} errors=${r.errors} intact=${r.intact}`);
    totalClicked += r.clicked;
    totalOk += r.ok;
    totalErrors += r.errors;
  }
  console.log(`\nTotal: ${totalClicked} clicked, ${totalOk} OK, ${totalErrors} errors`);

  await writeFile(join(OUT, "qa-buttons.json"), JSON.stringify(allResults, null, 2));
  console.log(`\nReport: ${OUT}/qa-buttons.json`);
  await browser.close();
}

main().catch((e) => { console.error(e); process.exit(1); });
