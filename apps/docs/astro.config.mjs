import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import markdoc from "@astrojs/markdoc";
import starlightSidebarTopics from "starlight-sidebar-topics";

// The Internals topic is only visible when AUDIENCE=internal.
const isInternal = process.env.AUDIENCE === "internal";

// Directories that belong exclusively to the Internals topic.
// In public builds they get no sidebar topic (we exclude them from topic
// resolution so they still render at their URL with no topic navigation).
const INTERNALS_DIRS = [
  "reference/architecture",
  "reference/ship-architecture",
  "reference/runtime",
  "reference/compiler",
  "reference/ship-stack",
];

// Build the topics array. Internals is conditionally included.
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
          ],
        },
      ]
    : []),
];

// In public builds, pages that live only under Internals are excluded from
// topic resolution so they render without a topic bar (sidebar-free). Their
// URLs continue to work — they just won't appear in any topic's navigation.
const excludedPatterns = isInternal
  ? []
  : INTERNALS_DIRS.map((d) => `/${d}/**`).concat(
      INTERNALS_DIRS.map((d) => `/${d}`)
    );

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
          tag: "script",
          attrs: { type: "module", src: "/ship-elements.js" },
        },
        // Restore persisted theme before first paint to avoid flash.
        // Must run before first paint — inline script, no src.
        {
          tag: "script",
          content: `
            (function() {
              var KEY = 'ship-site-theme';
              var VALID = ['ship-light','ship-dark','high-contrast','warm','gruvbox-dark','gruvbox-light','nord','solarized-dark'];
              var DARK = ['ship-dark','high-contrast','gruvbox-dark','nord','solarized-dark'];
              try {
                var stored = localStorage.getItem(KEY);
                if (stored && VALID.indexOf(stored) !== -1) {
                  document.documentElement.dataset.theme = stored;
                  if (DARK.indexOf(stored) !== -1) {
                    document.documentElement.classList.add('dark');
                  }
                }
              } catch(e) {}
            })();
          `,
        },
        // Keep html.dark in sync whenever data-theme changes at runtime.
        {
          tag: "script",
          content: `
            var DARK_THEMES = ['ship-dark','high-contrast','gruvbox-dark','nord','solarized-dark'];
            var sync = function() {
              var t = document.documentElement.dataset.theme;
              document.documentElement.classList.toggle('dark', DARK_THEMES.indexOf(t) !== -1);
            };
            sync();
            new MutationObserver(sync).observe(document.documentElement, {
              attributes: true, attributeFilter: ['data-theme']
            });
          `,
        },
      ],
      plugins: [
        starlightSidebarTopics(topics, {
          // Map all /components/* Astro pages (not in content collection) to
          // the Components topic by ID.
          topics: {
            components: ["/components/**", "/components"],
          },
          // In public builds, Internals-only pages render without a topic bar.
          exclude: excludedPatterns,
        }),
      ],
      // All URLs are preserved — no content was moved, only sidebar topology
      // changed. No redirects needed.
    }),
  ],
});
