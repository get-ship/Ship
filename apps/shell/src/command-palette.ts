/**
 * Command palette — ⌘K overlay for quick navigation and actions.
 * Fuzzy-matches view names and actions, navigates on Enter/click.
 */

const PALETTE_ID = "ship-cmdk";

interface Command {
  label: string;
  path: string;
  icon: string;
  keywords?: string;
}

const COMMANDS: Command[] = [
  { label: "Dashboard", path: "/dashboard", icon: "sparkles", keywords: "home overview" },
  { label: "Pipelines", path: "/pipelines", icon: "play", keywords: "workflows runs jobs" },
  { label: "Agents", path: "/agents", icon: "users", keywords: "configure profiles" },
  { label: "Skills", path: "/skills", icon: "zap", keywords: "browse edit files" },
  { label: "Git", path: "/git", icon: "git", keywords: "status diff commit branch" },
  { label: "Data", path: "/data", icon: "folder", keywords: "collections items query" },
  { label: "Canvas", path: "/canvas", icon: "image", keywords: "docs visual feedback" },
  { label: "Events", path: "/events", icon: "sparkles", keywords: "stream log kernel" },
  { label: "Settings", path: "/settings", icon: "settings", keywords: "config theme provider" },
  { label: "Chat", path: "/chat", icon: "message-square-plus", keywords: "agent conversation" },
  { label: "Specs", path: "/specs", icon: "file", keywords: "features design" },
];

let overlay: HTMLElement | null = null;
let isOpen = false;

function createOverlay(): HTMLElement {
  const el = document.createElement("div");
  el.id = PALETTE_ID;
  el.innerHTML = `
    <div class="cmdk-backdrop"></div>
    <div class="cmdk-panel">
      <div class="cmdk-input-wrap">
        <ship-icon name="search" size="sm"></ship-icon>
        <input class="cmdk-input" type="text" placeholder="Type a command or search..." autofocus />
        <kbd>esc</kbd>
      </div>
      <div class="cmdk-list"></div>
    </div>
  `;

  const style = document.createElement("style");
  style.textContent = `
    #${PALETTE_ID} { position: fixed; inset: 0; z-index: 9999; display: none; }
    #${PALETTE_ID}.open { display: block; }
    .cmdk-backdrop {
      position: absolute; inset: 0;
      background: rgba(0,0,0,0.4);
      backdrop-filter: blur(2px);
    }
    .dark .cmdk-backdrop { background: rgba(0,0,0,0.6); }
    .cmdk-panel {
      position: absolute;
      top: max(80px, 15%);
      left: 50%;
      transform: translateX(-50%);
      width: min(520px, 90vw);
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: var(--radius-xl);
      box-shadow: 0 16px 48px rgba(0,0,0,0.2);
      overflow: hidden;
    }
    .dark .cmdk-panel { box-shadow: 0 16px 48px rgba(0,0,0,0.5); }
    .cmdk-input-wrap {
      display: flex;
      align-items: center;
      gap: var(--space-2);
      padding: var(--space-3) var(--space-4);
      border-bottom: 1px solid var(--border);
    }
    .cmdk-input-wrap ship-icon { color: var(--muted-foreground); flex-shrink: 0; }
    .cmdk-input {
      flex: 1;
      border: none;
      outline: none;
      background: transparent;
      font-family: var(--font-body);
      font-size: var(--text-md);
      color: var(--foreground);
    }
    .cmdk-input::placeholder { color: var(--muted-foreground); }
    .cmdk-input-wrap kbd {
      font-family: var(--font-mono);
      font-size: 10px;
      padding: 2px 6px;
      border: 1px solid var(--border);
      border-radius: var(--radius-sm);
      background: var(--muted);
      color: var(--muted-foreground);
    }
    .cmdk-list {
      max-height: 320px;
      overflow-y: auto;
      padding: var(--space-1) 0;
    }
    .cmdk-item {
      display: flex;
      align-items: center;
      gap: var(--space-3);
      padding: var(--space-2) var(--space-4);
      cursor: pointer;
      color: var(--foreground);
      font-size: var(--text-sm);
      transition: background var(--duration-fast) var(--ease-out);
    }
    .cmdk-item:hover, .cmdk-item.selected {
      background: color-mix(in oklch, var(--primary) 10%, transparent);
    }
    .cmdk-item ship-icon { color: var(--muted-foreground); }
    .cmdk-item.selected ship-icon { color: var(--primary); }
    .cmdk-empty {
      padding: var(--space-6) var(--space-4);
      text-align: center;
      color: var(--muted-foreground);
      font-size: var(--text-sm);
    }
  `;
  document.head.appendChild(style);
  document.body.appendChild(el);

  // Events
  el.querySelector(".cmdk-backdrop")!.addEventListener("click", close);
  const input = el.querySelector(".cmdk-input") as HTMLInputElement;
  input.addEventListener("input", () => render(input.value));
  input.addEventListener("keydown", (e) => {
    if (e.key === "Escape") { e.preventDefault(); close(); }
    if (e.key === "Enter") { e.preventDefault(); activateSelected(); }
    if (e.key === "ArrowDown") { e.preventDefault(); moveSelection(1); }
    if (e.key === "ArrowUp") { e.preventDefault(); moveSelection(-1); }
  });

  return el;
}

let selectedIndex = 0;
let filteredCommands: Command[] = [];

function render(query: string) {
  if (!overlay) return;
  const list = overlay.querySelector(".cmdk-list")!;
  const q = query.toLowerCase().trim();

  filteredCommands = q
    ? COMMANDS.filter((c) =>
        c.label.toLowerCase().includes(q) ||
        c.path.toLowerCase().includes(q) ||
        (c.keywords ?? "").toLowerCase().includes(q)
      )
    : COMMANDS;

  selectedIndex = Math.min(selectedIndex, Math.max(0, filteredCommands.length - 1));

  if (filteredCommands.length === 0) {
    list.innerHTML = `<div class="cmdk-empty">No results for "${query}"</div>`;
    return;
  }

  list.innerHTML = filteredCommands
    .map((c, i) =>
      `<div class="cmdk-item${i === selectedIndex ? " selected" : ""}" data-path="${c.path}">
        <ship-icon name="${c.icon}" size="sm"></ship-icon>
        <span>${c.label}</span>
      </div>`
    )
    .join("");

  list.querySelectorAll(".cmdk-item").forEach((item) => {
    item.addEventListener("click", () => {
      const path = (item as HTMLElement).dataset.path!;
      close();
      (window as any).router?.navigate(path);
    });
  });
}

function moveSelection(dir: number) {
  selectedIndex = Math.max(0, Math.min(filteredCommands.length - 1, selectedIndex + dir));
  const input = overlay?.querySelector(".cmdk-input") as HTMLInputElement;
  render(input?.value ?? "");
}

function activateSelected() {
  const cmd = filteredCommands[selectedIndex];
  if (cmd) {
    close();
    (window as any).router?.navigate(cmd.path);
  }
}

export function open() {
  if (!overlay) overlay = createOverlay();
  overlay.classList.add("open");
  isOpen = true;
  selectedIndex = 0;
  const input = overlay.querySelector(".cmdk-input") as HTMLInputElement;
  input.value = "";
  render("");
  requestAnimationFrame(() => input.focus());
}

export function close() {
  overlay?.classList.remove("open");
  isOpen = false;
}

export function toggle() {
  isOpen ? close() : open();
}

export function isCommandPaletteOpen() {
  return isOpen;
}
