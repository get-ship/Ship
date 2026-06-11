import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import markdoc from "@astrojs/markdoc";
import starlightSidebarTopics from "starlight-sidebar-topics";

// Internals visibility:
//   - dev (bun run dev): always on — contributors want access locally.
//   - build: off unless AUDIENCE=internal is set explicitly (for building an
//     internal variant of the site).
// The command arg is threaded in via defineConfig's functional form at bottom
// of this file. We compute isInternal there.

// Content groups the collector produces (from `group:` frontmatter in
// docs/<skill>/references/docs/) that belong exclusively to the Internals
// topic. In public builds they get no sidebar topic (excluded from topic
// resolution) but still render at their URL with no topic navigation.
const INTERNALS_DIRS = [
  "architecture",
  "contributing",
];

// Build the topics array. Internals is conditionally included via buildTopics(isInternal).
function buildTopics(isInternal) {
  const topics = [
  // User-facing topics, ordered by the path a new visitor walks:
  // Start (concepts) → Build (agent-coordination surfaces: skills,
  // agents, views) → Components (SUI library) → Tutorial (external) →
  // Reference (ADRs, generated SDK ops, agent surfaces). Each `items`
  // autogenerate directory and each `link` resolves to a real page the
  // collector emits or a real Astro page under src/pages — sourceless
  // sections were removed so nothing 404s.
  {
    id: "start",
    label: "Start",
    icon: "open-book",
    link: "/concepts/",
    items: [
      {
        label: "Concepts",
        autogenerate: { directory: "concepts" },
      },
    ],
  },
  {
    id: "build",
    label: "Build",
    icon: "puzzle",
    link: "/smart-skills/",
    items: [
      {
        label: "Smart Skills",
        autogenerate: { directory: "smart-skills" },
      },
      {
        label: "Agents",
        autogenerate: { directory: "agents" },
      },
      {
        label: "Views",
        autogenerate: { directory: "views" },
      },
    ],
  },
  {
    id: "components",
    label: "Components",
    icon: "pencil",
    link: "/components/",
    items: [
      {
        label: "Overview",
        link: "/components/",
      },
      {
        label: "What is SUI",
        link: "/components/framework/",
      },
      {
        label: "Installing",
        link: "/components/installing/",
      },
      {
        label: "Tokens",
        link: "/components/tokens/",
      },
      {
        label: "Themes",
        link: "/components/themes/",
      },
      {
        label: "Cookbook",
        link: "/components/cookbook/",
      },
      {
        label: "Playground",
        link: "/components/playground/",
      },
    ],
  },
  {
    // The interactive SUI tutorial moved to the consumer hub
    // (getship.dev/tutorial) — it's a live editor + sandboxed preview,
    // a conversion surface that belongs where we sell the product and
    // own the chrome. This reference site links out to it; the prose
    // source of truth stays in the SUI doc-skill either way.
    id: "tutorial",
    label: "Tutorial",
    icon: "open-book",
    link: "https://getship.dev/docs/ship-ui/tutorial/",
    badge: { text: "live", variant: "tip" },
  },
  {
    id: "reference",
    label: "Reference",
    icon: "document",
    link: "/adrs/",
    items: [
      {
        label: "ADRs",
        autogenerate: { directory: "adrs" },
      },
      // Generated SDK op reference — one page per ConnectRPC op, harvested
      // by collect-skill-docs.ts from skills/sdk/<skill>/references/docs/
      // (emitted by `cargo xtask gen-skills`). Pages nest under ops/<domain>/
      // so autogenerate builds a per-domain subgroup tree from one directory.
      {
        label: "SDK (Ops)",
        autogenerate: { directory: "ops" },
      },
      // Agent-facing reference — every MCP tool, LSP surface,
      // sui-eval entrypoint, and ship:// resource that powers
      // agent workflows. Single page link (not autogenerate)
      // because the agents/ directory also feeds the Build →
      // Agents subtree and we don't want it duplicated.
      {
        label: "Agent Surfaces",
        link: "/agents/agent-surfaces/",
      },
    ],
  },
  ...(isInternal
    ? [
        {
          id: "internals",
          label: "Internals",
          icon: "setting",
          link: "/architecture/",
          items: [
            {
              label: "Architecture",
              autogenerate: { directory: "architecture" },
            },
            {
              label: "Contributing",
              autogenerate: { directory: "contributing" },
            },
          ],
        },
      ]
    : []),
  ];
  return topics;
}

// Patterns excluded from topic resolution. In public builds, Internals-only
// dirs are excluded so their URLs keep working without requiring a topic
// assignment. The homepage is mapped to the Guide topic below so that the
// topic rail renders with proper labels (not numeric indices).
function buildExcludedPatterns(isInternal) {
  // In internal builds every group has a topic, so nothing is excluded.
  // In public builds the Internals-only dirs keep rendering at their URL but
  // without a topic rail (they have no public topic assignment).
  if (isInternal) return [];
  return INTERNALS_DIRS.flatMap((d) => [`/${d}/**`, `/${d}`]);
}

// predev sets AUDIENCE=internal so dev runs internal mode automatically.
// Public build omits AUDIENCE and hides Internals.
const isInternal = process.env.AUDIENCE === 'internal';
const topics = buildTopics(isInternal);
const excludedPatterns = buildExcludedPatterns(isInternal);

export default defineConfig({
  site: "https://docs.getship.dev",
  integrations: [
    markdoc(),
    starlight({
      title: "Ship",
      social: [],
      // Starlight's Expressive Code defaults to `prefers-color-scheme`
      // to switch between its dark and light shiki themes — that ignores
      // Ship's own `data-theme` choices (ship-light/ship-dark/paper/halo/
      // console/contrast). Switch instead on the `.dark` class our
      // pre-paint script toggles for inherently-dark Ship themes, so the
      // code block tint follows the theme picker.
      expressiveCode: {
        // `github-light` washes out against our cream-tinted Ship surfaces
        // (comments ~#6e7781 on `--background` give ~3:1 contrast, below
        // WCAG AA). `light-plus` keeps higher chroma on identifiers and
        // pushes comments to a readable mid-tone. Pair with the saturated
        // `night-owl` for dark so syntax tokens stay distinct against the
        // dark Ship surfaces too. Switch on `html.dark` (toggled by our
        // pre-paint script), not prefers-color-scheme.
        themes: ["night-owl", "light-plus"],
        // expressive-code concatenates `:root` directly with this
        // value (no descendant combinator), so a bare `html.dark`
        // returned here produces invalid `:roothtml.dark`. Workaround:
        // return a class selector that the theme-switcher script also
        // toggles on <body>. expressive-code then emits
        // `:root.<class>` / `:root:not(.<class>)` — both are valid
        // since they're attribute matches on the same element.
        themeCssSelector: (theme) =>
          theme.name === "night-owl" ? ".dark" : ":not(.dark)",
        useDarkModeMediaQuery: false,
      },
      customCss: [
        "./src/styles/tokens.css",
        "./src/styles/themes.css",
        "./src/styles/ship-theme.css",
      ],
      components: {
        Header: "./src/components/Header.astro",
        SiteTitle: "./src/components/SiteTitle.astro",
        ThemeProvider: "./src/components/ThemeProvider.astro",
        ThemeSelect: "./src/components/ThemeProvider.astro",
        // Custom PageFrame switches to a no-sidebar full-canvas layout
        // for pages with frontmatter `template: 'sui-canvas'` — used by
        // playground + component-reference pages where the prose chrome
        // would compete with the live preview. Other pages get the
        // upstream behaviour byte-for-byte.
        PageFrame: "./src/components/PageFrame.astro",
      },
      head: [
        {
          tag: "link",
          attrs: { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" },
        },
        // SUI bundles import the runtime by bare specifier. The importmap
        // must precede any module script that loads `/sui-components.js`,
        // so it is emitted first via head.unshift order: importmap → bundle.
        {
          tag: "script",
          attrs: { type: "importmap" },
          content: JSON.stringify({
            imports: {
              "@ship/sui-runtime": "/sui-runtime.js",
              "@ship/sui-runtime/route": "/sui-runtime.js",
              "@ship/sui-runtime/template-hooks": "/sui-runtime.js",
              "@ship/sui-dom": "/sui-dom.js",
              "@ship/sdk-client": "/sui-sdk-client.js",
            },
          }),
        },
        {
          tag: "script",
          attrs: { type: "module", src: "/sui-components.js" },
        },
        // Ship theme ownership.
        //
        // Starlight ships its own theme init that fights over `data-theme` and
        // the `.dark` class. We take ownership: apply the stored ship theme
        // before first paint, then defend against any external writes to
        // either attribute via a MutationObserver.
        {
          tag: "script",
          content: `
            (function() {
              var KEY = 'ship-site-theme';
              var VALID = {
                'ship-light':1,'ship-dark':1,'paper':1,
                'halo':1,'console':1,'contrast':1
              };
              var DARK = {
                'ship-dark':1,'console':1,'contrast':1
              };
              function apply(id) {
                if (!VALID[id]) return false;
                var root = document.documentElement;
                if (root.dataset.theme !== id) root.dataset.theme = id;
                root.classList.toggle('dark', !!DARK[id]);
                return true;
              }
              function restore() {
                try {
                  var stored = localStorage.getItem(KEY);
                  if (stored && VALID[stored]) return apply(stored);
                } catch(e) {}
                return false;
              }
              // Pre-paint restore.
              restore();
              // Defend data-theme + .dark against external writes.
              new MutationObserver(function() {
                var current = document.documentElement.dataset.theme;
                if (!VALID[current]) { restore(); return; }
                var expectedDark = !!DARK[current];
                if (document.documentElement.classList.contains('dark') !== expectedDark) {
                  document.documentElement.classList.toggle('dark', expectedDark);
                }
              }).observe(document.documentElement, {
                attributes: true, attributeFilter: ['data-theme', 'class']
              });
            })();
          `,
        },
      ],
      plugins: [
        starlightSidebarTopics(topics, {
          // Map non-content Astro pages (under src/pages) to a topic by ID —
          // the homepage to Start, every /components/* page to Components.
          topics: {
            // Map the homepage to the Start topic so the topic rail
            // renders with a named label on "/" instead of numeric fallbacks.
            start: ["/"],
            components: ["/components/**", "/components"],
          },
          // In public builds, Internals-only pages render without a topic bar.
          exclude: excludedPatterns,
        }),
        // starlight-openapi removed: openapi.json was a stale artifact from
        // the pre-Connect REST cull. Endpoints now documented via Connect ops.
        // ...(isInternal
        //   ? [
        //       starlightOpenAPI([
        //         {
        //           base: "api-reference",
        //           schema: "./openapi.json",
        //           sidebar: {
        //             label: "HTTP API Reference",
        //             collapsed: false,
        //           },
        //         },
        //       ]),
        //     ]
        //   : []),
      ],
      // All URLs are preserved — no content was moved, only sidebar topology
      // changed. No redirects needed.
    }),
  ],
});
