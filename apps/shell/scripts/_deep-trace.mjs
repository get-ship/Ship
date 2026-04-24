import { chromium } from "playwright";
const b = await chromium.launch();
const p = await b.newPage({ viewport: { width: 1280, height: 800 } });
await p.goto("http://127.0.0.1:3100/", { waitUntil: "domcontentloaded" });
await p.waitForTimeout(2500);

const snap = (label) =>
  p.evaluate((label) => {
    const shell = document.querySelector("ship-shell");
    const children = shell ? Array.from(shell.children).map((c) => ({
      tag: c.tagName.toLowerCase(),
      id: c.id,
      cls: c.className,
    })) : [];
    const drawerContent = document.getElementById("drawer-content");
    return {
      label,
      bodyChildrenCount: document.body.children.length,
      shellChildrenCount: shell?.children.length ?? 0,
      shellChildren: children,
      drawerContentExists: !!drawerContent,
      drawerContentInnerHTMLLen: drawerContent?.innerHTML.length ?? -1,
      drawerContentParent: drawerContent?.parentElement?.id ?? drawerContent?.parentElement?.tagName,
      viewRootExists: !!document.getElementById("ship-view-root"),
    };
  }, label);

console.log("POST-BOOT:", JSON.stringify(await snap("boot"), null, 2));
await p.locator("#toggle-drawer").click();
await p.waitForTimeout(500);
console.log("\nAFTER CLICK 500ms:", JSON.stringify(await snap("500ms"), null, 2));
await p.waitForTimeout(2500);
console.log("\nAFTER CLICK 3s:", JSON.stringify(await snap("3s"), null, 2));
await b.close();
