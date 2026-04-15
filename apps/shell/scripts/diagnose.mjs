#!/usr/bin/env node
/**
 * Diagnose shell rendering. Loads each route, captures a screenshot, and
 * dumps computed styles for a small battery of elements so we can see
 * concretely which tokens resolved, which elements registered/hydrated,
 * and which rendered as empty or unstyled.
 *
 * Usage: pnpm -F shell diagnose [--route /pipelines]
 */
import { chromium } from "playwright";
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const BASE = process.env.SHELL_URL ?? "http://127.0.0.1:3100";
const OUT = "findings-evidence/diagnose";

const ROUTES = [
  "/pipelines",
  "/specs",
  "/git",
  "/settings",
  "/agents",
  "/skills",
  "/chat",
  "/events",
];

const THEMES = ["light", "dark"];

async function probe(page) {
  // Eval once, gather everything we care about in one round trip.
  return await page.evaluate(() => {
    function computed(sel, props) {
      const el = document.querySelector(sel);
      if (!el) return { present: false };
      const cs = getComputedStyle(el);
      const out = { present: true };
      for (const p of props) out[p] = cs.getPropertyValue(p).trim();
      return out;
    }

    const rootVars = (() => {
      const cs = getComputedStyle(document.documentElement);
      const names = [
        "--background",
        "--foreground",
        "--card",
        "--border",
        "--primary",
        "--muted-foreground",
        "--font-display",
        "--font-body",
        "--radius",
      ];
      return Object.fromEntries(names.map((n) => [n, cs.getPropertyValue(n).trim()]));
    })();

    const shipTags = Array.from(document.querySelectorAll("*"))
      .map((el) => el.tagName.toLowerCase())
      .filter((t) => t.startsWith("ship-"));
    const shipTagCounts = shipTags.reduce((acc, t) => {
      acc[t] = (acc[t] ?? 0) + 1;
      return acc;
    }, {});
    const shipUnhydrated = Array.from(document.querySelectorAll("*"))
      .filter((el) => el.tagName.toLowerCase().startsWith("ship-"))
      .filter((el) => !el.shadowRoot && (el.innerHTML.length === 0 || !customElements.get(el.tagName.toLowerCase())))
      .map((el) => el.tagName.toLowerCase());

    return {
      url: location.href,
      theme: document.documentElement.classList.contains("dark") ? "dark" : "light",
      rootVars,
      body: computed("body", ["background-color", "color", "font-family", "font-size"]),
      header: computed("header", ["background-color", "border-bottom-color", "padding"]),
      navActive: computed("header nav a.active", ["color", "background-color"]),
      mainMounted: (() => {
        const main = document.querySelector("main");
        return { innerHTMLLength: main ? main.innerHTML.length : 0 };
      })(),
      shipTagCounts,
      shipUnhydrated: Array.from(new Set(shipUnhydrated)),
      mountError: document.querySelector(".mount-error")?.textContent ?? null,
    };
  });
}

async function main() {
  await mkdir(OUT, { recursive: true });
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
  const page = await ctx.newPage();

  const report = { base: BASE, at: new Date().toISOString(), routes: [] };

  for (const theme of THEMES) {
    for (const route of ROUTES) {
      const url = BASE + route;
      try {
        // Seed theme preference via init script, then go.
        await ctx.addInitScript((t) => {
          localStorage.setItem("ship-shell-theme", t);
        }, theme);
        // SSE routes never reach `networkidle` — keep it to "load" and let
        // the mount resolve via explicit settle time below.
        await page.goto(url, { waitUntil: "load", timeout: 10000 });
        await page.waitForTimeout(600); // let SSE mount settle
        const data = await probe(page);
        const screenshotName = `${theme}-${route.replace(/\//g, "_")}.png`;
        await page.screenshot({
          path: join(OUT, screenshotName),
          fullPage: true,
        });
        report.routes.push({ theme, route, ...data, screenshot: screenshotName });
        console.log(
          `[${theme}] ${route.padEnd(12)}  mounted=${data.mainMounted.innerHTMLLength.toString().padStart(6)}  ship-tags=${Object.keys(data.shipTagCounts).length}  unhydrated=${data.shipUnhydrated.length}`,
        );
      } catch (err) {
        report.routes.push({ theme, route, error: String(err) });
        console.error(`[${theme}] ${route} ERROR: ${err.message}`);
      }
    }
  }

  await writeFile(join(OUT, "report.json"), JSON.stringify(report, null, 2));
  console.log(`\nReport: ${join(OUT, "report.json")}`);
  await browser.close();
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
