/**
 * Gallery — enumerate every registered `<ship-*>` element and render each
 * with one or two sensible prop combinations.
 *
 * The DOM custom-element registry has no "list all defined tags" API, so we
 * discover tags by regex over the (already-loaded, cached) ship-elements.js
 * module text, then filter to the ones `customElements.get()` confirms.
 */

type Sample = { props?: Record<string, string>; innerHTML?: string };

/**
 * Per-tag prop samples. Keep tiny — one or two meaningful variants each.
 * Unlisted tags render once with default props and no children.
 */
const SAMPLES: Record<string, Sample[]> = {
  "ship-button": [
    { props: { variant: "primary" }, innerHTML: "Primary" },
    { props: { variant: "secondary" }, innerHTML: "Secondary" },
  ],
  "ship-badge": [
    { props: { variant: "success" }, innerHTML: "ok" },
    { props: { variant: "warning" }, innerHTML: "warn" },
  ],
  "ship-alert": [{ props: { variant: "info" }, innerHTML: "An informational alert." }],
  "ship-card": [{ innerHTML: "<p>Card body</p>" }],
  "ship-spinner": [{}],
  "ship-progress": [{ props: { value: "42", max: "100" } }],
  "ship-status-dot": [{ props: { status: "ok" } }, { props: { status: "error" } }],
  "ship-stat": [{ props: { label: "Uptime", value: "99.9%" } }],
  "ship-separator": [{}],
  "ship-label": [{ innerHTML: "Label text" }],
  "ship-input": [{ props: { placeholder: "type here" } }],
  "ship-checkbox": [{ props: { checked: "" } }],
  "ship-switch": [{ props: { checked: "" } }],
  "ship-textarea": [{ props: { placeholder: "notes…" } }],
  "ship-empty-state": [{ props: { title: "Nothing yet" } }],
  "ship-icon": [{ props: { name: "check" } }],
  "ship-theme-toggle": [{}],
};

async function discoverTags(): Promise<string[]> {
  const res = await fetch("/vendor/ship-elements.js", { credentials: "same-origin" });
  const text = await res.text();
  const re = /customElements\.define\(\s*["']([a-z][a-z0-9-]*)["']/g;
  const set = new Set<string>();
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) {
    if (customElements.get(m[1])) set.add(m[1]);
  }
  return Array.from(set).sort();
}

function prefixOf(tag: string): string {
  // Group by the first two segments — `ship-pipeline-step` → `ship-pipeline`.
  const parts = tag.split("-");
  if (parts.length <= 2) return tag;
  return `${parts[0]}-${parts[1]}`;
}

function renderSample(tag: string, sample: Sample): HTMLElement {
  const card = document.createElement("div");
  card.className = "card";
  const title = document.createElement("div");
  title.className = "card-title";
  title.textContent = `<${tag}>`;
  card.appendChild(title);

  if (sample.props && Object.keys(sample.props).length > 0) {
    const props = document.createElement("div");
    props.className = "card-props";
    props.textContent = Object.entries(sample.props)
      .map(([k, v]) => (v === "" ? k : `${k}="${v}"`))
      .join(" ");
    card.appendChild(props);
  }

  const render = document.createElement("div");
  render.className = "card-render";
  try {
    const el = document.createElement(tag);
    if (sample.props) {
      for (const [k, v] of Object.entries(sample.props)) el.setAttribute(k, v);
    }
    if (sample.innerHTML) el.innerHTML = sample.innerHTML;
    render.appendChild(el);
  } catch (err) {
    const e = document.createElement("div");
    e.className = "card-error";
    e.textContent = `render failed: ${(err as Error).message}`;
    render.appendChild(e);
  }
  card.appendChild(render);
  return card;
}

/**
 * Render the gallery into `root`. Returns the number of tags rendered.
 */
export async function renderGalleryAsync(root: HTMLElement): Promise<number> {
  const tags = await discoverTags();
  const groups = new Map<string, string[]>();
  for (const t of tags) {
    const p = prefixOf(t);
    const list = groups.get(p) ?? [];
    list.push(t);
    groups.set(p, list);
  }

  root.textContent = "";
  const groupKeys = Array.from(groups.keys()).sort();
  for (const g of groupKeys) {
    const section = document.createElement("section");
    section.className = "group";
    const h = document.createElement("h2");
    h.textContent = g;
    section.appendChild(h);
    for (const tag of groups.get(g)!.sort()) {
      const samples = SAMPLES[tag] ?? [{}];
      for (const s of samples) section.appendChild(renderSample(tag, s));
    }
    root.appendChild(section);
  }
  return tags.length;
}

/**
 * Synchronous-looking wrapper for main.ts. Kicks off the async render and
 * returns an optimistic count placeholder (updated once the fetch resolves).
 */
export function renderGallery(root: HTMLElement): number {
  let count = 0;
  void renderGalleryAsync(root).then((n) => {
    count = n;
    const c = document.getElementById("count");
    if (c) c.textContent = `${n} element${n === 1 ? "" : "s"}`;
  });
  return count;
}
