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
  // — atoms
  "ship-button": [
    { props: { variant: "primary" }, innerHTML: "Primary" },
    { props: { variant: "secondary" }, innerHTML: "Secondary" },
    { props: { variant: "ghost" }, innerHTML: "Ghost" },
    { props: { variant: "destructive" }, innerHTML: "Destructive" },
  ],
  "ship-badge": [
    { props: { variant: "success" }, innerHTML: "ok" },
    { props: { variant: "warning" }, innerHTML: "warn" },
    { props: { variant: "destructive" }, innerHTML: "err" },
  ],
  "ship-chip": [{ innerHTML: "alpha" }, { innerHTML: "beta" }],
  "ship-alert": [
    { props: { variant: "info" }, innerHTML: "Informational message." },
    { props: { variant: "warning" }, innerHTML: "Warning message." },
  ],
  "ship-card": [{ innerHTML: "<p style='margin:0'>Card body content.</p>" }],
  "ship-spinner": [{}],
  "ship-progress": [{ props: { value: "42", max: "100" } }],
  "ship-status-dot": [
    { props: { status: "ok" } },
    { props: { status: "warn" } },
    { props: { status: "error" } },
  ],
  "ship-stat": [{ props: { label: "Uptime", value: "99.9%" } }],
  "ship-separator": [{}],
  "ship-label": [{ innerHTML: "Label text" }],
  "ship-input": [{ props: { placeholder: "type here" } }],
  "ship-checkbox": [{ props: { checked: "" } }],
  "ship-switch": [{ props: { checked: "" } }],
  "ship-textarea": [{ props: { placeholder: "notes…" } }],
  "ship-empty-state": [{ props: { title: "Nothing yet" }, innerHTML: "No items to show." }],
  "ship-icon": [{ props: { name: "check" } }, { props: { name: "x" } }, { props: { name: "chevron-down" } }],
  "ship-theme-toggle": [{}],
  "ship-kbd": [{ innerHTML: "⌘K" }, { innerHTML: "Esc" }],
  "ship-link": [{ props: { href: "#" }, innerHTML: "A link" }],
  "ship-avatar": [{ props: { name: "Ada Lovelace" } }],

  // — text content
  "ship-markdown": [
    { props: { content: "# Heading\n\nA paragraph with **bold** and `code`." } },
  ],
  "ship-json-view": [
    { props: { data: JSON.stringify({ name: "ship", version: "0.2.0", ok: true }) } },
  ],
  "ship-timestamp": [{ props: { value: new Date().toISOString() } }],

  // — overlays / portals (force-open so gallery shows them)
  "ship-dialog": [
    {
      props: { open: "", heading: "Dialog example" },
      innerHTML: "<p style='margin:0'>Dialog body content.</p>",
    },
  ],
  "ship-alert-dialog": [
    {
      props: { open: "", heading: "Alert dialog" },
      innerHTML: "<p style='margin:0'>Are you sure?</p>",
    },
  ],
  "ship-popover": [
    {
      props: { open: "" },
      innerHTML:
        '<ship-button slot="trigger">Open</ship-button><div>popover body</div>',
    },
  ],
  "ship-tooltip": [
    {
      props: { open: "", text: "Tooltip text" },
      innerHTML: '<ship-button>Hover</ship-button>',
    },
  ],
  "ship-dropdown-menu": [
    {
      props: { open: "" },
      innerHTML:
        '<ship-button slot="trigger">Menu</ship-button>' +
        '<ship-menu-item>First</ship-menu-item>' +
        '<ship-menu-item>Second</ship-menu-item>',
    },
  ],
  "ship-toast-container": [
    { innerHTML: '<ship-toast variant="info">Hello toast</ship-toast>' },
  ],
  "ship-toast": [{ props: { variant: "info" }, innerHTML: "A toast." }],

  // — composite
  "ship-breadcrumb": [
    {
      innerHTML:
        '<ship-breadcrumb-item href="#">Home</ship-breadcrumb-item>' +
        '<ship-breadcrumb-item href="#">Projects</ship-breadcrumb-item>' +
        '<ship-breadcrumb-item>Current</ship-breadcrumb-item>',
    },
  ],
  "ship-tabs": [
    {
      innerHTML:
        '<ship-tab slot="tab" active>One</ship-tab>' +
        '<ship-tab slot="tab">Two</ship-tab>' +
        '<ship-tab-panel>Panel one content.</ship-tab-panel>' +
        '<ship-tab-panel hidden>Panel two content.</ship-tab-panel>',
    },
  ],
  "ship-tab": [{ props: { active: "" }, innerHTML: "Tab label" }],
  "ship-tab-panel": [{ innerHTML: "Tab panel content." }],
  "ship-tree-view": [
    {
      innerHTML:
        '<ship-tree-node label="src">' +
        '<ship-tree-node label="components"></ship-tree-node>' +
        '<ship-tree-node label="utils"></ship-tree-node>' +
        "</ship-tree-node>",
    },
  ],
  "ship-timeline": [
    {
      innerHTML:
        '<ship-timeline-item time="10:00">Started</ship-timeline-item>' +
        '<ship-timeline-item time="10:05">Gate passed</ship-timeline-item>' +
        '<ship-timeline-item time="10:08">Merged</ship-timeline-item>',
    },
  ],
  "ship-toolbar": [
    {
      innerHTML:
        '<ship-button variant="ghost">Save</ship-button>' +
        '<ship-button variant="ghost">Undo</ship-button>' +
        '<ship-separator></ship-separator>' +
        '<ship-button variant="ghost">Run</ship-button>',
    },
  ],
  "ship-list-item": [{ innerHTML: "A list row with some content" }],
  "ship-select": [
    {
      innerHTML:
        '<ship-option value="a">Option A</ship-option>' +
        '<ship-option value="b">Option B</ship-option>',
    },
  ],
  "ship-option": [{ props: { value: "a" }, innerHTML: "Option A" }],
  "ship-field": [
    {
      props: { label: "Email" },
      innerHTML: '<ship-input placeholder="you@example.com"></ship-input>',
    },
  ],
  "ship-step-segments": [{ props: { value: "2", total: "4" } }],
  "ship-heatmap": [{ props: { data: JSON.stringify([1, 2, 3, 2, 1, 0, 2]) } }],

  // — git
  "ship-git-diff-line": [
    { props: { kind: "add", content: "+ added line" } },
    { props: { kind: "del", content: "- removed line" } },
  ],

  // — annotations (Phase 2)
  "ship-annotation-layer": [
    { innerHTML: "<p style='margin:0'>Annotatable surface.</p>" },
  ],
  "ship-annotation-marker": [{ props: { count: "3", x: "50", y: "10" } }],
  "ship-annotation-list": [
    { innerHTML: '<div style="padding:8px">(no annotations)</div>' },
  ],

  // — charts
  "ship-bar-chart": [
    {
      props: {
        data: JSON.stringify([12, 19, 7, 24, 15, 22, 30]),
        labels: JSON.stringify(["M", "T", "W", "T", "F", "S", "S"]),
        tone: "primary",
      },
    },
  ],
  "ship-line-chart": [
    {
      props: {
        data: JSON.stringify([1, 4, 3, 7, 9, 8, 11, 14]),
        tone: "success",
      },
    },
  ],
  "ship-area-chart": [
    {
      props: {
        data: JSON.stringify([5, 9, 7, 14, 12, 18, 20, 24]),
        tone: "primary",
      },
    },
  ],
  "ship-sparkline": [
    {
      props: {
        data: JSON.stringify([3, 5, 4, 7, 6, 9, 8, 11, 10, 13]),
        tone: "primary",
      },
    },
    {
      props: {
        data: JSON.stringify([13, 10, 11, 9, 7, 9, 5, 8, 6, 4]),
        tone: "destructive",
      },
    },
  ],
  "ship-donut": [
    { props: { value: "64", max: "100", tone: "primary" } },
    { props: { value: "87", max: "100", tone: "success" } },
  ],
  "ship-gauge": [
    { props: { value: "72", max: "100", label: "Score" } },
  ],
  "ship-meter": [
    { props: { value: "42", max: "100", tone: "primary" } },
  ],
  "ship-progress-ring": [
    { props: { value: "40", max: "100" } },
    { props: { value: "92", max: "100" } },
  ],

  // — composite / ship-specific
  "ship-code": [{ props: { language: "rust" }, innerHTML: "fn main() {\n    println!(\"hello\");\n}" }],
  "ship-copy-button": [{ props: { value: "npm i @ship/ui" } }],
  "ship-diff": [
    {
      props: {
        before: "line one\nline two\nline three",
        after: "line one\nline 2\nline three\nnew line",
      },
    },
  ],
  "ship-event-row": [
    {
      props: {
        kind: "job.started",
        at: new Date().toISOString(),
      },
      innerHTML: "agent=rust-lane job=fix-import",
    },
  ],
  "ship-history-row": [
    { innerHTML: "pipeline run — <strong>2m ago</strong> — passed" },
  ],
  "ship-log-line": [
    { props: { level: "info" }, innerHTML: "[12:03:44] started" },
    { props: { level: "warn" }, innerHTML: "[12:03:45] slow response" },
    { props: { level: "error" }, innerHTML: "[12:03:46] connection refused" },
  ],
  "ship-metachip": [
    { props: { label: "agent" }, innerHTML: "rust-lane" },
  ],
  "ship-pipeline-dag": [
    {
      innerHTML:
        '<ship-pipeline-step id="tests" state="done" agent="test-writer"></ship-pipeline-step>' +
        '<ship-pipeline-step id="impl" state="running" agent="rust-lane"></ship-pipeline-step>' +
        '<ship-pipeline-step id="review" state="pending" agent="reviewer"></ship-pipeline-step>',
    },
  ],
  "ship-pipeline-step": [
    { props: { id: "impl", state: "running", agent: "rust-lane" } },
  ],
  "ship-pipeline-run": [
    {
      props: { slug: "fix-auth", state: "running", agent: "rust-lane" },
    },
  ],
  "ship-pipeline-definition": [
    { props: { name: "tdd", description: "Test-driven development" } },
  ],
  "ship-skeleton": [{}, { props: { width: "200px", height: "14px" } }],
  "ship-spec-row": [{ innerHTML: "spec: auth-tokens — <strong>ready</strong>" }],
  "ship-table": [
    {
      innerHTML:
        "<thead><tr><th>name</th><th>status</th></tr></thead>" +
        "<tbody><tr><td>rust-lane</td><td>idle</td></tr>" +
        "<tr><td>web-lane</td><td>busy</td></tr></tbody>",
    },
  ],
  "ship-tree-node": [
    {
      props: { label: "src", open: "" },
      innerHTML:
        '<ship-tree-node label="components"></ship-tree-node>' +
        '<ship-tree-node label="utils"></ship-tree-node>',
    },
  ],
  "ship-timeline-item": [
    { props: { time: "10:05" }, innerHTML: "Gate passed" },
  ],
  "ship-breadcrumb-item": [
    { props: { href: "#" }, innerHTML: "Projects" },
  ],
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

  const meta = document.createElement("div");
  meta.className = "card-meta";
  const tagLabel = document.createElement("span");
  tagLabel.className = "card-tag";
  tagLabel.textContent = `<${tag}>`;
  meta.appendChild(tagLabel);
  if (sample.props && Object.keys(sample.props).length > 0) {
    const props = document.createElement("span");
    props.className = "card-props";
    props.textContent = Object.entries(sample.props)
      .map(([k, v]) => (v === "" ? k : k))
      .join(" · ");
    meta.appendChild(props);
  }
  card.appendChild(meta);

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
    const grid = document.createElement("div");
    grid.className = "group-grid";
    for (const tag of groups.get(g)!.sort()) {
      const samples = SAMPLES[tag] ?? [{}];
      for (const s of samples) grid.appendChild(renderSample(tag, s));
    }
    section.appendChild(grid);
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
