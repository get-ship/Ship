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
const INTERNALS_DIRS = [
  "reference/architecture",
  "reference/ship-architecture",
  "reference/runtime",
  "reference/compiler",
  "reference/ship-stack",
  "reference/http-api",
];

// Build the topics array. Internals is conditionally included via buildTopics(isInternal).
function buildTopics(isInternal) {
  const topics = [
  {
    label: "Guide",
    icon: "open-book",
    link: "/reference/getting-started/",
    items: [
      {
        label: "Getting Started",
        autogenerate: { directory: "reference/getting-started" },
      },
      {
        label: "Smart Skills",
        autogenerate: { directory: "reference/smart-skills" },
      },
      {
        label: "Agents",
        autogenerate: { directory: "reference/agents" },
      },
      {
        label: "Workflow",
        autogenerate: { directory: "reference/workflow" },
      },
    ],
  },
  {
    label: "Reference",
    icon: "document",
    link: "/reference/cli/",
    items: [
      {
        label: "CLI Reference",
        autogenerate: { directory: "reference/cli" },
      },
      {
        label: "MCP Tools",
        autogenerate: { directory: "reference/mcp-tools" },
      },
      {
        label: "Schemas",
        autogenerate: { directory: "reference/schema-reference" },
      },
      {
        label: "Registry",
        autogenerate: { directory: "reference/registry" },
      },
      {
        label: "Studio",
        autogenerate: { directory: "reference/studio" },
      },
      {
        label: "Studio (Ship)",
        autogenerate: { directory: "reference/ship-studio" },
      },
      {
        label: "Contributing",
        autogenerate: { directory: "reference/contributing" },
      },
    ],
  },
  {
    label: "Extensions",
    icon: "puzzle",
    link: "/reference/smart-skills/",
    items: [
      {
        label: "Smart Skills",
        autogenerate: { directory: "reference/smart-skills" },
      },
      {
        label: "Agents",
        autogenerate: { directory: "reference/agents" },
      },
      {
        label: "Views",
        autogenerate: { directory: "reference/views" },
      },
      {
        label: "Pipelines",
        autogenerate: { directory: "reference/pipelines" },
      },
      {
        label: "Artifacts",
        autogenerate: { directory: "reference/artifacts" },
      },
      {
        label: "Collections",
        autogenerate: { directory: "reference/collections" },
      },
      {
        label: "Triggers",
        autogenerate: { directory: "reference/triggers" },
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
  ...(isInternal
    ? [
        {
          id: "internals",
          label: "Internals",
          icon: "setting",
          link: "/reference/architecture/",
          items: [
            {
              label: "Architecture",
              autogenerate: { directory: "reference/architecture" },
            },
            {
              label: "Ship Architecture",
              autogenerate: { directory: "reference/ship-architecture" },
            },
            {
              label: "Runtime",
              autogenerate: { directory: "reference/runtime" },
            },
            {
              label: "Compiler",
              autogenerate: { directory: "reference/compiler" },
            },
            {
              label: "Ship Stack",
              autogenerate: { directory: "reference/ship-stack" },
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

// In public builds, pages that live only under Internals are excluded from
// topic resolution so they render without a topic bar (sidebar-free). Their
// URLs continue to work — they just won't appear in any topic's navigation.
function buildExcludedPatterns(isInternal) {
  return isInternal
    ? []
    : INTERNALS_DIRS.map((d) => `/${d}/**`).concat(
        INTERNALS_DIRS.map((d) => `/${d}`)
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
      logo: {
        src: "./src/assets/logo.svg",
      },
      social: [],
      customCss: [
        "@ship/ui/tokens.css",
        "@ship/ui/themes.css",
        "./src/styles/ship-theme.css",
      ],
      components: {
        Header: "./src/components/Header.astro",
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
                'ship-light':1,'ship-dark':1,'high-contrast':1,'warm':1,
                'gruvbox-dark':1,'gruvbox-light':1,'nord':1,'solarized-dark':1
              };
              var DARK = {
                'ship-dark':1,'high-contrast':1,'gruvbox-dark':1,'nord':1,'solarized-dark':1
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
          // Map HTTP API pages (generated by starlight-openapi) to the internals topic.
          topics: {
            components: ["/components/**", "/components"],
            ...(isInternal
              ? {
                  internals: [
                    "/reference/http-api/**",
                    "/reference/http-api",
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
                  base: "reference/http-api",
                  schema: "./openapi.json",
                  sidebar: {
                    label: "HTTP API",
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
