import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import markdoc from "@astrojs/markdoc";
import starlightSidebarTopics from "starlight-sidebar-topics";
import starlightOpenAPI, { openAPISidebarGroups } from "starlight-openapi";

// Internals visibility:
//   - dev (pnpm dev): always on — contributors want access locally.
//   - build: off unless AUDIENCE=internal is set explicitly (for building an
//     internal variant of the site).
// The command arg is threaded in via defineConfig's functional form at bottom
// of this file. We compute isInternal there.

// Directories that belong exclusively to the Internals topic.
// In public builds they get no sidebar topic (we exclude them from topic
// resolution so they still render at their URL with no topic navigation).
//
// `pipelines` and `triggers` are power-user / experimental surfaces — kept
// here so the content stays reachable internally but no public navigation
// promises them. Promote out of this list when they ship publicly.
const INTERNALS_DIRS = [
  "architecture",
  "ship-architecture",
  "runtime",
  "compiler",
  "ship-stack",
  "http-api",
  "contributing",
  "pipelines",
  "triggers",
];

// Build the topics array. Internals is conditionally included via buildTopics(isInternal).
function buildTopics(isInternal) {
  const topics = [
  {
    id: "getting-started",
    label: "Getting Started",
    icon: "open-book",
    link: "/getting-started/",
    items: [
      {
        label: "Getting Started",
        autogenerate: { directory: "getting-started" },
      },
    ],
  },
  {
    id: "concepts",
    label: "Concepts",
    icon: "puzzle",
    link: "/concepts/",
    items: [
      {
        label: "Concepts",
        autogenerate: { directory: "concepts" },
      },
    ],
  },
  {
    id: "extensions",
    label: "Extensions",
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
      {
        label: "Artifacts",
        autogenerate: { directory: "artifacts" },
      },
      {
        label: "Collections",
        autogenerate: { directory: "collections" },
      },
      // Pipelines + Triggers are experimental power-user surfaces — hidden
      // from the public Extensions sidebar. Content stays reachable
      // internally via the Internals topic (see INTERNALS_DIRS).
      {
        label: "Components",
        autogenerate: { directory: "components" },
      },
      {
        label: "Component Packs",
        autogenerate: { directory: "component-packs" },
      },
      {
        label: "Design System",
        autogenerate: { directory: "design-system" },
      },
      {
        label: "SDK",
        autogenerate: { directory: "sdk" },
      },
      {
        label: "Recipes",
        autogenerate: { directory: "recipes" },
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
    ],
  },
  {
    id: "reference",
    label: "Reference",
    icon: "document",
    link: "/cli/",
    items: [
      {
        label: "CLI Reference",
        autogenerate: { directory: "cli" },
      },
      {
        label: "MCP Tools",
        autogenerate: { directory: "mcp-tools" },
      },
      {
        label: "Schemas",
        autogenerate: { directory: "schema-reference" },
      },
      {
        label: "Registry",
        autogenerate: { directory: "registry" },
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
              label: "Ship Architecture",
              autogenerate: { directory: "ship-architecture" },
            },
            {
              label: "Runtime",
              autogenerate: { directory: "runtime" },
            },
            {
              label: "Compiler",
              autogenerate: { directory: "compiler" },
            },
            {
              label: "Ship Stack",
              autogenerate: { directory: "ship-stack" },
            },
            {
              label: "Pipelines (experimental)",
              autogenerate: { directory: "pipelines" },
            },
            {
              label: "Triggers (experimental)",
              autogenerate: { directory: "triggers" },
            },
            {
              label: "Contributing",
              autogenerate: { directory: "contributing" },
            },
            // openAPISidebarGroups is a placeholder — starlight-openapi replaces
            // it with the generated per-endpoint sidebar entries at build time.
            ...openAPISidebarGroups,
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
  // `reference/adrs` lives outside any topic's autogenerated directory; keep
  // it rendering at its URL without a topic rail.
  const base = ["/adrs/**", "/adrs"];
  return isInternal
    ? base
    : base.concat(
        INTERNALS_DIRS.map((d) => `/${d}/**`),
        INTERNALS_DIRS.map((d) => `/${d}`),
      );
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
      customCss: [
        "@ship/set/tokens.css",
        "@ship/set/themes.css",
        "./src/styles/ship-theme.css",
      ],
      components: {
        Header: "./src/components/Header.astro",
        SiteTitle: "./src/components/SiteTitle.astro",
        ThemeProvider: "./src/components/ThemeProvider.astro",
        ThemeSelect: "./src/components/ThemeProvider.astro",
      },
      head: [
        {
          tag: "link",
          attrs: { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" },
        },
        {
          tag: "script",
          attrs: { type: "module", src: "/ship-elements.js" },
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
                'ship-dark':1,'contrast':1
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
          // Map all /components/* Astro pages (not in content collection) to
          // the Components topic by ID.
          // Map the HTTP API guide (ship-http-api skill, /http-api/*) and the
          // generated OpenAPI reference (/api-reference/*) to the internals topic.
          topics: {
            // Map the homepage to the Getting Started topic so the topic rail
            // renders with named labels on "/" instead of numeric fallbacks.
            "getting-started": ["/"],
            components: ["/components/**", "/components"],
            ...(isInternal
              ? {
                  internals: [
                    "/http-api/**",
                    "/http-api",
                    "/api-reference/**",
                    "/api-reference",
                  ],
                }
              : {}),
          },
          // In public builds, Internals-only pages render without a topic bar.
          exclude: excludedPatterns,
        }),
        ...(isInternal
          ? [
              starlightOpenAPI([
                {
                  // `api-reference`, not `http-api`: the ship-http-api skill
                  // owns the hand-written /http-api/ guide pages. The generated
                  // per-endpoint reference lives alongside it at /api-reference/.
                  base: "api-reference",
                  schema: "./openapi.json",
                  sidebar: {
                    label: "HTTP API Reference",
                    collapsed: false,
                  },
                },
              ]),
            ]
          : []),
      ],
      // All URLs are preserved — no content was moved, only sidebar topology
      // changed. No redirects needed.
    }),
  ],
});
