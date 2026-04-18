#!/usr/bin/env node
/**
 * Interactive QA — clicks every button, nav link, and interactive element.
 * Reports console errors, broken elements, and CSS issues.
 */
import { chromium } from "playwright";
import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";

const BASE = process.env.SHELL_URL ?? "http://127.0.0.1:3100";
const OUT = "qa-evidence/interactive";

async function main() {
  await mkdir(OUT, { recursive: true });
  const browser = await chromium.launch();
  const results = [];

  // ── Test 1: Header buttons ──
  console.log("\n═══ Header Buttons ═══");
  {
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    const page = await ctx.newPage();
    const errors = [];
    page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text().slice(0, 150)); });
    page.on("pageerror", (err) => errors.push(err.message.slice(0, 150)));

    await page.goto(`${BASE}/dashboard`, { waitUntil: "load" });
    await page.waitForTimeout(3000);

    // Check theme toggle
    const themeBtn = await page.$("#toggle-theme");
    const themeBefore = await page.evaluate(() => document.documentElement.classList.contains("dark"));
    if (themeBtn) {
      await themeBtn.click();
      await page.waitForTimeout(500);
    }
    const themeAfter = await page.evaluate(() => document.documentElement.classList.contains("dark"));
    const themeWorks = themeBefore !== themeAfter;
    console.log(`  Theme toggle: ${themeWorks ? "✓" : "✗"} (${themeBefore} → ${themeAfter})`);

    // Check theme toggle icon renders
    const iconInfo = await page.evaluate(() => {
      const icon = document.querySelector("#toggle-theme ship-icon");
      const sr = icon?.shadowRoot;
      const svg = sr?.querySelector("svg");
      return { hasSvg: !!svg, w: icon?.getBoundingClientRect().width ?? 0 };
    });
    console.log(`  Theme icon renders: ${iconInfo.hasSvg ? "✓" : "✗"} (${iconInfo.w}px)`);

    // Check drawer toggle
    const drawerBtn = await page.$("#toggle-drawer");
    if (drawerBtn) {
      await drawerBtn.click();
      await page.waitForTimeout(2000);
    }
    const drawerOpen = await page.evaluate(() => document.body.classList.contains("drawer-open"));
    console.log(`  Drawer toggle: ${drawerOpen ? "✓" : "✗"}`);

    // Check layout isn't broken with drawer open
    const layout = await page.evaluate(() => {
      const main = document.querySelector("main");
      const drawer = document.querySelector("aside#drawer");
      return {
        mainW: main?.getBoundingClientRect().width ?? 0,
        drawerW: drawer?.getBoundingClientRect().width ?? 0,
        bodyW: document.body.getBoundingClientRect().width,
      };
    });
    const layoutOk = layout.mainW > 500 && layout.drawerW > 0 && (layout.mainW + layout.drawerW) <= layout.bodyW + 5;
    console.log(`  Drawer layout: ${layoutOk ? "✓" : "✗"} main=${Math.round(layout.mainW)}px drawer=${Math.round(layout.drawerW)}px`);

    await page.screenshot({ path: join(OUT, "header-drawer.png") });

    // Close drawer
    if (drawerBtn) {
      await drawerBtn.click();
      await page.waitForTimeout(500);
    }

    results.push({ test: "header", errors, themeWorks, iconRenders: iconInfo.hasSvg, drawerOpen, layoutOk });
    await ctx.close();
  }

  // ── Test 2: Nav links ──
  console.log("\n═══ Nav Links ═══");
  {
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    const page = await ctx.newPage();
    const allErrors = {};

    await page.goto(`${BASE}/dashboard`, { waitUntil: "load" });
    await page.waitForTimeout(2000);

    const navLinks = await page.evaluate(() => {
      return Array.from(document.querySelectorAll("header nav a[data-route]")).map(a => ({
        route: a.dataset.route,
        text: a.textContent.trim(),
      }));
    });

    for (const link of navLinks) {
      const errors = [];
      page.removeAllListeners("console");
      page.removeAllListeners("pageerror");
      page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text().slice(0, 150)); });
      page.on("pageerror", (err) => errors.push(err.message.slice(0, 150)));

      const navA = await page.$(`header nav a[data-route="${link.route}"]`);
      if (navA) await navA.click();
      await page.waitForTimeout(3000);

      const state = await page.evaluate((route) => {
        const active = document.querySelector(`header nav a.active`);
        const main = document.getElementById("ship-view-root");
        const hasContent = main?.children.length > 0 && main.textContent.trim().length > 10;
        return {
          activeRoute: active?.dataset?.route ?? "none",
          hasContent,
          mainH: main?.getBoundingClientRect().height ?? 0,
        };
      }, link.route);

      const ok = state.hasContent && state.mainH > 50;
      console.log(`  ${ok ? "✓" : "✗"} ${link.text.padEnd(12)} route=${link.route} active=${state.activeRoute === link.route} content=${state.hasContent} errors=${errors.length}`);

      if (errors.length > 0) {
        allErrors[link.route] = errors;
        for (const e of errors.slice(0, 2)) console.log(`    ERR: ${e.slice(0, 100)}`);
      }

      await page.screenshot({ path: join(OUT, `nav-${link.text}.png`) });
    }

    results.push({ test: "nav", errors: allErrors });
    await ctx.close();
  }

  // ── Test 3: View interactions ──
  console.log("\n═══ View Interactions ═══");
  {
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    const page = await ctx.newPage();

    // Skills: click a skill to open detail
    {
      const errors = [];
      page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text().slice(0, 150)); });
      await page.goto(`${BASE}/skills`, { waitUntil: "load" });
      await page.waitForTimeout(3000);
      const firstSkill = await page.$('#ship-view-root [data-on\\:click*="detail"]');
      if (firstSkill) {
        await firstSkill.click();
        await page.waitForTimeout(3000);
        const hasDetail = await page.evaluate(() => {
          const root = document.getElementById("ship-view-root");
          return root?.innerHTML?.includes("SKILL.md") || root?.innerHTML?.length > 500;
        });
        console.log(`  Skills detail: ${hasDetail ? "✓" : "✗"} errors=${errors.length}`);
        await page.screenshot({ path: join(OUT, "skills-detail.png") });
      } else {
        console.log("  Skills detail: ⚠ no clickable skill found");
      }
    }

    // Agents: click an agent card
    {
      const errors = [];
      page.removeAllListeners("console");
      page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text().slice(0, 150)); });
      await page.goto(`${BASE}/agents`, { waitUntil: "load" });
      await page.waitForTimeout(3000);
      const firstAgent = await page.$('#ship-view-root a[href*="agent"], #ship-view-root [data-on\\:click*="agent"]');
      if (firstAgent) {
        await firstAgent.click();
        await page.waitForTimeout(3000);
        console.log(`  Agents detail: clicked, errors=${errors.length}`);
        await page.screenshot({ path: join(OUT, "agents-detail.png") });
      } else {
        console.log("  Agents detail: ⚠ no clickable agent found");
      }
    }

    // Pipelines: click tabs
    {
      const errors = [];
      page.removeAllListeners("console");
      page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text().slice(0, 150)); });
      await page.goto(`${BASE}/pipelines`, { waitUntil: "load" });
      await page.waitForTimeout(3000);
      const tabs = await page.$$('#ship-view-root ship-tab, #ship-view-root [role="tab"]');
      console.log(`  Pipelines tabs: ${tabs.length} found`);
      for (const tab of tabs.slice(0, 3)) {
        try {
          await tab.click({ timeout: 3000 });
          await page.waitForTimeout(1000);
        } catch { /* tab might be hidden */ }
      }
      console.log(`  Pipelines tab clicks: errors=${errors.length}`);
      await page.screenshot({ path: join(OUT, "pipelines-tabs.png") });
    }

    // Data: collection selector
    {
      const errors = [];
      page.removeAllListeners("console");
      page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text().slice(0, 150)); });
      await page.goto(`${BASE}/data`, { waitUntil: "load" });
      await page.waitForTimeout(3000);
      await page.screenshot({ path: join(OUT, "data-view.png") });
      console.log(`  Data view: errors=${errors.length}`);
    }

    // Settings: expand advanced
    {
      const errors = [];
      page.removeAllListeners("console");
      page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text().slice(0, 150)); });
      await page.goto(`${BASE}/settings`, { waitUntil: "load" });
      await page.waitForTimeout(5000);
      const advancedBtn = await page.$('#ship-view-root .s-collapse-btn, #ship-view-root button[aria-controls="advanced-body"]');
      if (advancedBtn) {
        await advancedBtn.click();
        await page.waitForTimeout(500);
        console.log(`  Settings advanced: ✓ clicked`);
      }
      await page.screenshot({ path: join(OUT, "settings-advanced.png") });
      console.log(`  Settings: errors=${errors.length}`);
    }

    // Canvas: switch docs
    {
      const errors = [];
      page.removeAllListeners("console");
      page.on("console", (msg) => { if (msg.type() === "error") errors.push(msg.text().slice(0, 150)); });
      await page.goto(`${BASE}/canvas`, { waitUntil: "load" });
      await page.waitForTimeout(3000);
      const docLinks = await page.$$('.canvas-rail-item');
      console.log(`  Canvas docs: ${docLinks.length} found`);
      if (docLinks.length > 1) {
        await docLinks[1].click();
        await page.waitForTimeout(3000);
        await page.screenshot({ path: join(OUT, "canvas-doc2.png") });
      }
      console.log(`  Canvas: errors=${errors.length}`);
      if (errors.length > 0) {
        for (const e of errors.slice(0, 3)) console.log(`    ERR: ${e.slice(0, 100)}`);
      }
    }

    await ctx.close();
  }

  // ── Test 4: CSS consistency check ──
  console.log("\n═══ CSS Consistency ═══");
  {
    const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
    const page = await ctx.newPage();
    const views = ["/dashboard", "/skills", "/agents", "/settings", "/pipelines", "/git", "/data", "/canvas", "/events"];

    for (const view of views) {
      await page.goto(`${BASE}${view}`, { waitUntil: "load" });
      await page.waitForTimeout(3000);

      const cssCheck = await page.evaluate(() => {
        const issues = [];
        const root = document.getElementById("ship-view-root");
        if (!root) return ["no ship-view-root"];

        // Check if tokens CSS is loaded
        const bg = getComputedStyle(document.body).backgroundColor;
        if (bg === "rgba(0, 0, 0, 0)" || bg === "") issues.push("body bg not set (tokens missing?)");

        // Check font
        const font = getComputedStyle(document.body).fontFamily;
        if (!font || font === "serif" || font === '""') issues.push("font not set");

        // Check for elements with no styling (default browser styles)
        const h1s = root.querySelectorAll("h1, h2, h3");
        for (const h of h1s) {
          const cs = getComputedStyle(h);
          if (cs.fontFamily === "serif" || cs.fontFamily === "Times New Roman") {
            issues.push(`${h.tagName} using browser default font`);
          }
        }

        // Check for zero-height content sections
        const sections = root.querySelectorAll("section, div[class]");
        let zeroH = 0;
        for (const s of sections) {
          if (s.getBoundingClientRect().height === 0 && s.children.length > 0) zeroH++;
        }
        if (zeroH > 0) issues.push(`${zeroH} zero-height sections`);

        return issues;
      });

      const ok = cssCheck.length === 0;
      console.log(`  ${ok ? "✓" : "✗"} ${view.padEnd(14)} ${cssCheck.length === 0 ? "clean" : cssCheck.join(", ")}`);
    }

    await ctx.close();
  }

  console.log("\n═══ Done ═══");
  await writeFile(join(OUT, "qa-interactive.json"), JSON.stringify(results, null, 2));
  await browser.close();
}

main().catch((e) => { console.error(e); process.exit(1); });
