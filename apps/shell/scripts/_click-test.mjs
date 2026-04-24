#!/usr/bin/env node
import { chromium } from "playwright";
const BASE = "http://127.0.0.1:3100";
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const pages = [];
page.context().on("page", (p) => pages.push(p.url()));
page.on("pageerror", (e) => console.log("[pageerror]", e.message));
await page.goto(BASE + "/", { waitUntil: "domcontentloaded" });
await page.waitForTimeout(2000);
console.log("=== initial URL:", page.url());
console.log("=== body classes:", await page.evaluate(() => document.body.className));

// Click rail toggle
console.log("\n--- click #toggle-rail ---");
await page.locator("#toggle-rail").click();
await page.waitForTimeout(400);
console.log("body classes:", await page.evaluate(() => document.body.className));
console.log("rail width:", await page.evaluate(() => document.querySelector("ship-rail")?.getBoundingClientRect().width));
await page.locator("#toggle-rail").click();
await page.waitForTimeout(400);
console.log("after second click, body classes:", await page.evaluate(() => document.body.className));

// Click chat drawer toggle
console.log("\n--- click #toggle-drawer ---");
await page.locator("#toggle-drawer").click();
await page.waitForTimeout(800);
console.log("URL:", page.url());
console.log("body classes:", await page.evaluate(() => document.body.className));
console.log("drawer width:", await page.evaluate(() => document.querySelector(".ship-drawer")?.getBoundingClientRect().width));
console.log("new pages opened:", pages.length);

await browser.close();
