#!/usr/bin/env node
/**
 * 1. Visit Studio, extract pair code from the rendered DOM
 * 2. Call `ship auth pair <code>` to confirm
 * 3. Wait for Studio to redirect into the app
 * 4. Capture screenshots of every Studio route + matching shell route
 */
import { chromium } from "playwright";
import { mkdir, writeFile } from "node:fs/promises";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { homedir } from "node:os";

const STUDIO = "http://127.0.0.1:3000";
const SHELL = "http://127.0.0.1:3100";
const DAEMON = "http://127.0.0.1:9315";
const OUT = "findings-evidence/parity";
const TOKEN = (await readFile(join(homedir(), ".ship", "daemon.token"), "utf8")).trim();

const VIEWS = [
  { name: "pipelines", studio: "/studio/workflows", shell: "/pipelines" },
  { name: "git", studio: "/studio/git", shell: "/git" },
  { name: "settings", studio: "/studio/settings", shell: "/settings" },
  { name: "agents", studio: "/studio/agents", shell: "/agents" },
  { name: "skills", studio: "/studio/skills", shell: "/skills" },
];

async function pair(page) {
  await page.goto(STUDIO + "/studio", { waitUntil: "load" });
  await page.waitForFunction(
    () => /Pair this browser|Workflows|Agents|Skills|Workspaces/i.test(document.body.innerText || ""),
    { timeout: 15000 },
  ).catch(() => {});
  let code = null;
  for (let i = 0; i < 20; i++) {
    await page.waitForTimeout(300);
    const probe = await page.evaluate(() => {
      const txt = document.body.innerText || "";
      if (!/Pair this browser/i.test(txt)) return { paired: true, code: null };
      const cands = [...txt.matchAll(/\b([A-Z0-9]{3})-([A-Z0-9]{3})\b/g)];
      const m = cands.find((c) => /\d/.test(c[1]) && /\d/.test(c[2]));
      return { paired: false, code: m ? `${m[1]}-${m[2]}` : null };
    });
    if (probe.paired) return;
    if (probe.code) {
      code = probe.code;
      break;
    }
  }
  if (!code) {
    console.log("no pair code found");
    return;
  }
  console.log(`  pairing ${code}`);
  const resp = await fetch(`${DAEMON}/api/auth/pair/confirm`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${TOKEN}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ code }),
  });
  const body = await resp.text();
  if (!resp.ok) {
    console.log(`  pair http ${resp.status}: ${body}`);
    return;
  }
  // Wait for Studio to detect the session and leave the pair screen.
  for (let i = 0; i < 15; i++) {
    await page.waitForTimeout(500);
    const paired = await page.evaluate(() => !/Pair this browser/i.test(document.body.innerText || ""));
    if (paired) return;
  }
}

async function shoot(page, url, path) {
  try {
    await page.goto(url, { waitUntil: "load", timeout: 15000 });
  } catch {}
  await page.waitForTimeout(1500);
  await page.screenshot({ path, fullPage: true });
}

async function main() {
  await mkdir(OUT, { recursive: true });
  const browser = await chromium.launch();

  for (const theme of ["light", "dark"]) {
    const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    await ctx.addInitScript((t) => {
      localStorage.setItem("ship-shell-theme", t);
      localStorage.setItem("ship-theme", t);
      localStorage.setItem("theme", t);
    }, theme);
    const page = await ctx.newPage();

    await pair(page);

    for (const v of VIEWS) {
      console.log(`[${theme}] ${v.name.padEnd(10)}`);
      await shoot(page, STUDIO + v.studio, join(OUT, `${v.name}-${theme}-studio.png`));
      await shoot(page, SHELL + v.shell, join(OUT, `${v.name}-${theme}-shell.png`));
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
