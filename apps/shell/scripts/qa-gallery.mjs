#!/usr/bin/env node
/**
 * Phase 1 QA — Gallery + Views
 * Captures screenshots, console errors, component status, visual checks.
 */
import { chromium } from "playwright";
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const BASE = process.env.SHELL_URL ?? "http://127.0.0.1:3100";
const OUT = "qa-evidence";

const VIEWS = [
  "/dashboard",
  "/skills",
  "/agents",
  "/settings",
  "/pipelines",
  "/git",
  "/data",
  "/canvas",
  "/events",
];

async function main() {
  await mkdir(join(OUT, "gallery"), { recursive: true });
  await mkdir(join(OUT, "views"), { recursive: true });

  const browser = await chromium.launch();
  const results = { gallery: null, views: [] };

  // ── Gallery QA ──
  console.log("\n═══ Gallery QA ═══");
  const galleryResult = await qaGallery(browser);
  results.gallery = galleryResult;

  // ── View QA ──
  console.log("\n═══ View QA ═══");
  for (const view of VIEWS) {
    const viewResult = await qaView(browser, view);
    results.views.push(viewResult);
  }

  // ── Summary ──
  console.log("\n═══ QA Summary ═══");
  const g = results.gallery;
  console.log(`Gallery: ${g.totalCards} cards, ${g.definedCount} defined, ${g.zeroSizeCount} zero-size, ${g.consoleErrors.length} console errors`);
  for (const v of results.views) {
    const status = v.loaded ? "✓" : "✗";
    console.log(`${status} ${v.path.padEnd(14)} shell=${v.shellIntact ? "ok" : "MISSING"} errors=${v.consoleErrors.length} css=${v.cssIssues.length}`);
  }

  await writeFile(join(OUT, "qa-report.json"), JSON.stringify(results, null, 2));
  console.log(`\nReport written to ${OUT}/qa-report.json`);

  await browser.close();
}

async function qaGallery(browser) {
  const result = {
    totalCards: 0,
    definedCount: 0,
    hydratedCount: 0,
    zeroSizeCount: 0,
    consoleErrors: [],
    cards: [],
    issues: [],
  };

  for (const theme of ["light", "dark"]) {
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    const page = await ctx.newPage();

    // Collect console errors
    const errors = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") errors.push(msg.text());
    });
    page.on("pageerror", (err) => errors.push(err.message));

    if (theme === "dark") {
      await ctx.addInitScript(() => localStorage.setItem("ship-shell-theme", "dark"));
    } else {
      await ctx.addInitScript(() => localStorage.removeItem("ship-shell-theme"));
    }

    await page.goto(`${BASE}/devtools.html`, { waitUntil: "load" });
    await page.waitForTimeout(2000);

    // Screenshot
    await page.screenshot({
      path: join(OUT, "gallery", `gallery-${theme}.png`),
      fullPage: true,
    });
    console.log(`  ✓ gallery-${theme}.png captured`);

    if (theme === "light") {
      // Detailed card analysis only once
      const data = await page.evaluate(() => {
        const cards = Array.from(document.querySelectorAll(".card"));
        return cards.map((card) => {
          const meta = card.querySelector(".card-meta");
          const tagEl = meta?.querySelector(".card-tag");
          const tagText = tagEl?.textContent?.replace(/[<>]/g, "") ?? "";
          const render = card.querySelector(".card-render");
          const el = render?.firstElementChild;
          const tag = el?.tagName?.toLowerCase() ?? tagText;
          const bbox = el?.getBoundingClientRect();
          return {
            tag,
            defined: !!(tag && customElements.get(tag)),
            hasShadow: !!el?.shadowRoot,
            width: Math.round(bbox?.width ?? 0),
            height: Math.round(bbox?.height ?? 0),
          };
        });
      });

      result.totalCards = data.length;
      result.definedCount = data.filter((c) => c.defined).length;
      result.hydratedCount = data.filter((c) => c.hasShadow).length;
      result.zeroSizeCount = data.filter((c) => c.width === 0 || c.height === 0).length;
      result.cards = data;

      // Check specific issues from the action plan
      const specificChecks = await page.evaluate(() => {
        const issues = [];

        // Check badge/chip padding symmetry
        const badges = document.querySelectorAll("ship-badge, ship-chip");
        for (const b of badges) {
          const shadow = b.shadowRoot;
          if (!shadow) continue;
          const host = shadow.querySelector(".host, :first-child");
          if (!host) continue;
          const cs = getComputedStyle(host);
          const pl = parseFloat(cs.paddingLeft);
          const pr = parseFloat(cs.paddingRight);
          if (Math.abs(pl - pr) > 2) {
            issues.push(`${b.tagName}: padding asymmetric L=${pl} R=${pr}`);
          }
        }

        // Check chart overflow
        const charts = document.querySelectorAll("ship-line-chart, ship-area-chart, ship-bar-chart");
        for (const c of charts) {
          const shadow = c.shadowRoot;
          if (!shadow) continue;
          const svgEl = shadow.querySelector("svg");
          if (svgEl) {
            const cs = getComputedStyle(svgEl);
            if (cs.overflow !== "hidden") {
              issues.push(`${c.tagName}: SVG overflow=${cs.overflow} (should be hidden)`);
            }
          }
        }

        // Check skeleton animation
        const skeletons = document.querySelectorAll("ship-skeleton");
        for (const s of skeletons) {
          const shadow = s.shadowRoot;
          if (!shadow) continue;
          const inner = shadow.querySelector("[part], .skeleton, :first-child");
          if (inner) {
            const cs = getComputedStyle(inner);
            const hasAnim = cs.animationName && cs.animationName !== "none";
            if (!hasAnim) {
              issues.push(`ship-skeleton: no animation running`);
            }
          }
        }

        // Check json-view key readability
        const jsonViews = document.querySelectorAll("ship-json-view");
        for (const jv of jsonViews) {
          const shadow = jv.shadowRoot;
          if (!shadow) continue;
          const keys = shadow.querySelectorAll(".key");
          for (const k of keys) {
            const cs = getComputedStyle(k);
            const color = cs.color;
            // Just note the color for manual review
            if (keys.length > 0) {
              issues.push(`ship-json-view: key color=${color} (verify dark mode readability)`);
              break;
            }
          }
        }

        // Check tree-view nodes
        const trees = document.querySelectorAll("ship-tree-view");
        for (const t of trees) {
          const shadow = t.shadowRoot;
          if (!shadow) continue;
          const nodes = shadow.querySelectorAll("ship-tree-node");
          issues.push(`ship-tree-view: ${nodes.length} nodes found`);
        }

        // Check all 12 new components render
        const newComponents = [
          "ship-accordion", "ship-collapsible", "ship-slider",
          "ship-radio-group", "ship-context-menu", "ship-hover-card",
          "ship-scroll-area", "ship-toggle-group", "ship-navigation-menu",
        ];
        for (const tag of newComponents) {
          const el = document.querySelector(tag);
          if (!el) {
            issues.push(`${tag}: NOT FOUND in gallery`);
          } else {
            const sr = el.shadowRoot;
            if (!sr) issues.push(`${tag}: no shadow root`);
          }
        }

        return issues;
      });

      result.issues = specificChecks;
      for (const issue of specificChecks) {
        console.log(`  [check] ${issue}`);
      }
    }

    result.consoleErrors.push(...errors.map((e) => ({ theme, error: e })));
    if (errors.length > 0) {
      console.log(`  ${theme}: ${errors.length} console errors`);
      for (const e of errors.slice(0, 5)) console.log(`    - ${e.slice(0, 120)}`);
    } else {
      console.log(`  ${theme}: 0 console errors`);
    }

    await ctx.close();
  }

  // Zero-size cards
  if (result.zeroSizeCount > 0) {
    console.log(`  ⚠ ${result.zeroSizeCount} zero-size cards:`);
    result.cards
      .filter((c) => c.width === 0 || c.height === 0)
      .forEach((c) => console.log(`    ${c.tag} defined=${c.defined} shadow=${c.hasShadow}`));
  }

  return result;
}

async function qaView(browser, path) {
  const result = {
    path,
    loaded: false,
    shellIntact: false,
    consoleErrors: [],
    cssIssues: [],
    datastarErrors: [],
  };

  const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
  const page = await ctx.newPage();

  const errors = [];
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(msg.text());
  });
  page.on("pageerror", (err) => errors.push(err.message));

  try {
    // Load shell first
    await page.goto(`${BASE}/`, { waitUntil: "load" });
    await page.waitForTimeout(500);

    // Navigate to view via shell router
    await page.goto(`${BASE}${path}`, { waitUntil: "load" });
    await page.waitForTimeout(2500);

    result.loaded = true;

    // Check shell chrome
    const shellCheck = await page.evaluate(() => {
      const header = document.querySelector("header") || document.querySelector(".ship-topbar");
      const main = document.getElementById("ship-view-root");
      const nav = document.querySelector(".ship-rail") || document.querySelector("header nav");
      return {
        hasHeader: !!header,
        hasMain: !!main,
        hasNav: !!nav,
        mainContent: main?.innerHTML?.length ?? 0,
      };
    });

    result.shellIntact = shellCheck.hasHeader && shellCheck.hasMain && shellCheck.hasNav;

    // Screenshot
    await page.screenshot({
      path: join(OUT, "views", `${path.slice(1)}-light.png`),
      fullPage: true,
    });

    // Dark mode
    await page.evaluate(() => {
      localStorage.setItem("ship-shell-theme", "dark");
      document.documentElement.classList.add("dark");
    });
    await page.waitForTimeout(500);
    await page.screenshot({
      path: join(OUT, "views", `${path.slice(1)}-dark.png`),
      fullPage: true,
    });
    console.log(`  ✓ ${path} screenshots captured (shell=${result.shellIntact ? "ok" : "BROKEN"})`);

    // CSS check — look for zero-height sections or missing styles
    const cssCheck = await page.evaluate(() => {
      const issues = [];
      const main = document.querySelector("#ship-view-root");
      if (!main) return issues;

      // Check for elements with zero padding where we'd expect some
      const sections = main.querySelectorAll("section, .card, ship-card");
      for (const s of sections) {
        const cs = getComputedStyle(s);
        const bbox = s.getBoundingClientRect();
        if (bbox.height === 0 && s.children.length > 0) {
          issues.push(`zero-height: ${s.tagName}.${s.className}`);
        }
      }
      return issues;
    });
    result.cssIssues = cssCheck;

  } catch (err) {
    console.log(`  ✗ ${path} failed: ${err.message}`);
  }

  result.consoleErrors = errors;
  if (errors.length > 0) {
    console.log(`  ${path}: ${errors.length} console errors`);
    for (const e of errors.slice(0, 3)) console.log(`    - ${e.slice(0, 120)}`);
  }

  await ctx.close();
  return result;
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
