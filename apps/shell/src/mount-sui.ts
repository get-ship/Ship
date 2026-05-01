// Mount a SUI view into a shell-owned slot.
//
// Fetches `/_sui/fragment/<name>[/<sub-path>]`, drops the markup into
// `slot.innerHTML`, and re-creates the inline `<script type="module">`
// boot block so the browser actually executes it (innerHTML doesn't
// run scripts on its own). Returns a disposer that clears the slot
// after invoking the runtime's attach() teardown — `attach()` stashes
// it on the root element as `__suiDispose`.
//
// Multi-mount: multiple fragments coexist on one page because the
// daemon scopes its boot script with `[data-sui-root][data-view="X"]`.
// The shell's importmap + runtime modules (in index.html) are loaded
// once and shared.

interface SuiRoot extends HTMLElement {
  __suiDispose?: () => void;
}

const ATTACH_TIMEOUT_MS = 4000;

export async function mountSuiView(
  slot: HTMLElement,
  name: string,
  subPath?: string,
): Promise<() => void> {
  const path = subPath ? `${name}/${subPath}` : name;
  const url = `/_sui/fragment/${path}`;
  const res = await fetch(url, { credentials: "same-origin" });
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`mountSuiView(${path}) ${res.status}: ${body}`);
  }

  slot.innerHTML = await res.text();

  // Activate any inline scripts. Cloning is required because innerHTML
  // does not execute <script> nodes; cloning + DOM replacement does.
  for (const stale of Array.from(slot.querySelectorAll("script"))) {
    const fresh = document.createElement("script");
    for (const attr of Array.from(stale.attributes)) {
      fresh.setAttribute(attr.name, attr.value);
    }
    fresh.textContent = stale.textContent ?? "";
    stale.replaceWith(fresh);
  }

  const root = slot.querySelector<SuiRoot>(
    `[data-sui-root][data-view="${cssEscape(name)}"]`,
  );
  if (!root) {
    throw new Error(`mountSuiView(${path}): wrapper missing after boot`);
  }

  // Poll for attach() to publish the disposer. Module imports are async,
  // so the boot script's `await import(...)` chain hasn't resolved yet
  // when the script element first runs.
  const deadline = Date.now() + ATTACH_TIMEOUT_MS;
  while (!root.__suiDispose && Date.now() < deadline) {
    await new Promise((r) => setTimeout(r, 16));
  }

  return () => {
    try {
      root.__suiDispose?.();
    } catch (err) {
      console.error(`[mountSuiView] dispose failed for ${path}:`, err);
    }
    slot.innerHTML = "";
  };
}

function cssEscape(s: string): string {
  return s.replace(/["\\]/g, "\\$&");
}
