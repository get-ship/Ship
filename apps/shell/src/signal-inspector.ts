/**
 * Signal Inspector — floating debug panel for Datastar signals.
 * Toggle with Ctrl+Shift+D. Shows all current signal values.
 */

const PANEL_ID = "ship-signal-inspector";
const STORAGE_KEY = "ship-signal-inspector-open";

let panel: HTMLElement | null = null;
let updateInterval: ReturnType<typeof setInterval> | null = null;

function createPanel(): HTMLElement {
  const el = document.createElement("div");
  el.id = PANEL_ID;
  el.innerHTML = `
    <div class="si-header">
      <span class="si-title">Signals</span>
      <button class="si-close" title="Close (Ctrl+Shift+D)">&times;</button>
    </div>
    <div class="si-body"></div>
  `;

  const style = document.createElement("style");
  style.textContent = `
    #${PANEL_ID} {
      position: fixed;
      bottom: 12px;
      right: 12px;
      width: 320px;
      max-height: 400px;
      background: var(--card, #fff);
      border: 1px solid var(--border);
      border-radius: var(--radius-lg, 8px);
      box-shadow: 0 8px 24px rgba(0,0,0,.12);
      font-family: var(--font-body);
      font-size: var(--text-xs, 12px);
      z-index: 9999;
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }
    .dark #${PANEL_ID} {
      box-shadow: 0 8px 24px rgba(0,0,0,.4);
    }
    #${PANEL_ID} .si-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 8px 12px;
      border-bottom: 1px solid var(--border);
      background: var(--muted, #f5f5f5);
    }
    #${PANEL_ID} .si-title {
      font-weight: 600;
      font-size: var(--text-sm, 13px);
      color: var(--foreground);
    }
    #${PANEL_ID} .si-close {
      background: none;
      border: none;
      font-size: 18px;
      cursor: pointer;
      color: var(--muted-foreground);
      padding: 0 4px;
      line-height: 1;
    }
    #${PANEL_ID} .si-close:hover { color: var(--foreground); }
    #${PANEL_ID} .si-body {
      overflow-y: auto;
      flex: 1;
      padding: 4px 0;
    }
    #${PANEL_ID} .si-row {
      display: flex;
      justify-content: space-between;
      align-items: baseline;
      padding: 4px 12px;
      gap: 8px;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 40%, transparent);
    }
    #${PANEL_ID} .si-row:last-child { border-bottom: none; }
    #${PANEL_ID} .si-key {
      font-family: ui-monospace, monospace;
      color: var(--primary, #c05);
      font-weight: 500;
      white-space: nowrap;
      flex-shrink: 0;
    }
    #${PANEL_ID} .si-val {
      font-family: ui-monospace, monospace;
      color: var(--muted-foreground);
      text-align: right;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      max-width: 180px;
    }
    #${PANEL_ID} .si-empty {
      padding: 16px 12px;
      color: var(--muted-foreground);
      text-align: center;
    }
  `;
  document.head.appendChild(style);
  document.body.appendChild(el);

  el.querySelector(".si-close")!.addEventListener("click", () => toggle(false));

  return el;
}

function getSignals(): Record<string, unknown> {
  // Datastar stores signals on the document via a proxy. Access the
  // internal store if available, otherwise return empty.
  const ds = (document as any)._ds;
  if (ds?.signals) {
    // Datastar v1 internal signal store
    const raw = ds.signals;
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(raw)) {
      out[k] = (v as any)?.value ?? v;
    }
    return out;
  }
  // Try window.ds or globalThis.__ds
  const alt = (window as any).ds ?? (globalThis as any).__ds;
  if (alt?.signals) {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(alt.signals)) {
      out[k] = (v as any)?.value ?? v;
    }
    return out;
  }
  // Fallback: scan DOM for data-signals attributes (both syntaxes)
  const out: Record<string, unknown> = {};
  // Object syntax: data-signals="{key: 'val', ...}"
  document.querySelectorAll("[data-signals]").forEach((el) => {
    const raw = el.getAttribute("data-signals");
    if (!raw) return;
    try {
      const parsed = new Function(`return (${raw})`)();
      Object.assign(out, parsed);
    } catch { /* ignore parse errors */ }
  });
  // Colon syntax: data-signals:signalName="expression"
  const view = document.getElementById("ship-view-root");
  if (view) {
    const walk = (node: Element) => {
      for (const attr of Array.from(node.attributes)) {
        if (attr.name.startsWith("data-signals:")) {
          const name = attr.name.slice("data-signals:".length);
          try {
            out[name] = new Function(`return (${attr.value})`)();
          } catch {
            out[name] = attr.value;
          }
        }
      }
      for (const child of Array.from(node.children)) walk(child);
    };
    walk(view);
  }
  return out;
}

function renderSignals() {
  if (!panel) return;
  const body = panel.querySelector(".si-body")!;
  const signals = getSignals();
  const keys = Object.keys(signals).sort();

  if (keys.length === 0) {
    body.innerHTML = `<div class="si-empty">No signals found</div>`;
    return;
  }

  body.innerHTML = keys
    .map((k) => {
      const v = signals[k];
      const display =
        typeof v === "string"
          ? v.length > 40
            ? `"${v.slice(0, 37)}…"`
            : `"${v}"`
          : JSON.stringify(v);
      return `<div class="si-row"><span class="si-key">${k}</span><span class="si-val" title="${String(v).replace(/"/g, "&quot;")}">${display}</span></div>`;
    })
    .join("");
}

function toggle(force?: boolean) {
  const open = force ?? !panel?.isConnected;
  if (open) {
    if (!panel) panel = createPanel();
    else if (!panel.isConnected) document.body.appendChild(panel);
    renderSignals();
    updateInterval = setInterval(renderSignals, 1000);
    localStorage.setItem(STORAGE_KEY, "1");
  } else {
    panel?.remove();
    if (updateInterval) clearInterval(updateInterval);
    updateInterval = null;
    localStorage.removeItem(STORAGE_KEY);
  }
}

export function initSignalInspector() {
  // Keyboard shortcut: Ctrl+Shift+D
  document.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.shiftKey && e.key === "D") {
      e.preventDefault();
      toggle();
    }
  });

  // Restore if previously open
  if (localStorage.getItem(STORAGE_KEY)) {
    // Delay so Datastar has time to initialize
    setTimeout(() => toggle(true), 1000);
  }
}
