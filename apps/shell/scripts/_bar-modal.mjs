import { chromium } from "playwright";
const b = await chromium.launch();
const p = await b.newPage({ viewport: { width: 1280, height: 800 } });
p.on("pageerror", (e) => console.log("[err]", e.message));
await p.goto("http://127.0.0.1:3100/", { waitUntil: "domcontentloaded" });
await p.waitForTimeout(3000);
await p.keyboard.press("Control+K");
await p.waitForTimeout(500);
await p.screenshot({ path: ".artifacts/web-qa/bar-modal.png" });
console.log("panel visible:", await p.$eval("ship-command-bar", (el) =>
  !!el.shadowRoot?.querySelector(".panel")
));
