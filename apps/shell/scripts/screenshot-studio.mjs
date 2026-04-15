#!/usr/bin/env node
/**
 * Screenshot every first-party Studio view (React app) + shell hypermedia
 * view side by side. Output is per-view paired PNGs for visual parity work.
 */
import { chromium } from "playwright";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";

const STUDIO = process.env.STUDIO_URL ?? "http://127.0.0.1:3000";
const SHELL = process.env.SHELL_URL ?? "http://127.0.0.1:3100";
const OUT = "findings-evidence/parity";

const VIEWS = [
  // studio path, shell path, filename
  { name: "pipelines", studio: "/studio/workflows", shell: "/pipelines" },
  { name: "git", studio: "/studio/git", shell: "/git" },
  { name: "settings", studio: "/studio/settings", shell: "/settings" },
  { name: "agents", studio: "/studio/agents", shell: "/agents" },
  { name: "skills", studio: "/studio/skills", shell: "/skills" },
];

async function shoot(page, url, path, theme) {
  await page.context().addInitScript((t) => {
    localStorage.setItem("ship-shell-theme", t);
    // Studio uses its own theme key. Set both to be safe.
    localStorage.setItem("theme", t);
    localStorage.setItem("ship-theme", t);
  }, theme);
  try {
    await page.goto(url, { waitUntil: "load", timeout: 15000 });
  } catch (e) {
    console.warn(`  timeout: ${url}`);
  }
  await page.waitForTimeout(1200);
  await page.screenshot({ path, fullPage: true });
}

async function main() {
  await mkdir(OUT, { recursive: true });
  const browser = await chromium.launch();
  for (const theme of ["light", "dark"]) {
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    const page = await ctx.newPage();
    for (const v of VIEWS) {
      const studioUrl = STUDIO + v.studio;
      const shellUrl = SHELL + v.shell;
      console.log(`[${theme}] ${v.name.padEnd(10)}  studio=${v.studio}  shell=${v.shell}`);
      await shoot(page, studioUrl, join(OUT, `${v.name}-${theme}-studio.png`), theme);
      await shoot(page, shellUrl, join(OUT, `${v.name}-${theme}-shell.png`), theme);
    }
    await ctx.close();
  }
  await browser.close();
  console.log(`\nPairs in ${OUT}/`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
