// views/ship-ui-studio/page.sui
import { bindAttr, bindFor, bindIf, bindText, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateAttr, hydrateOn, hydrateText } from "@ship/sui-dom";

// views/ship-ui-studio/controller.ts
import { controller, signal } from "@ship/sui-runtime";

// views/ship-ui-studio/catalog.ts
var STUDIO_HIDE_TAGS = new Set([
  "sui-command-bar",
  "sui-command-palette",
  "sui-command-chat",
  "sui-launcher",
  "sui-panel-dock",
  "sui-panel-dock-tab",
  "sui-nav-top",
  "sui-rail",
  "sui-rail-item",
  "sui-rail-section",
  "sui-status-bar",
  "sui-toast",
  "sui-toast-container",
  "sui-terminal",
  "sui-devtools-source",
  "sui-devtools-signals",
  "sui-devtools-lint",
  "sui-devtools-schema",
  "sui-devtools-components"
]);
var BAKED_SUI_VIEWS = [
  { id: "agents", tag: "agents", kind: "view", description: "Agent list" },
  { id: "git", tag: "git", kind: "view", description: "Git overview" }
];
async function loadSuiComponents() {
  const propsByTag = await loadSuiPropMap();
  return Object.keys(propsByTag).filter((tag) => !STUDIO_HIDE_TAGS.has(tag)).sort().map((tag) => ({
    id: tag,
    tag,
    kind: "sui",
    props: propsByTag[tag] ?? []
  }));
}
async function loadSuiPropMap() {
  try {
    const res = await fetch("/api/assets/sui-custom-elements.json", { credentials: "same-origin" });
    if (!res.ok)
      return {};
    const cem = await res.json();
    const out = {};
    for (const mod of cem.modules ?? []) {
      for (const decl of mod.declarations ?? []) {
        if (decl.kind !== "class" || !decl.tagName)
          continue;
        const attrs = decl.attributes ?? [];
        out[decl.tagName] = attrs.map((a) => ({
          name: a.name,
          type: a.type?.text ?? "string",
          default: a.default,
          description: a.description,
          values: Array.isArray(a.values) ? a.values : undefined
        }));
      }
    }
    return out;
  } catch {
    return {};
  }
}
async function loadDaemonViews() {
  try {
    const res = await fetch("/api/v1/views", { credentials: "same-origin" });
    if (!res.ok)
      return [];
    const body = await res.json();
    return (body.data?.views ?? []).filter((v) => v.type === "sui").map((v) => ({
      id: v.name,
      tag: v.name,
      kind: "view",
      description: v.rail?.label
    }));
  } catch {
    return [];
  }
}
async function loadCatalog() {
  const [daemonViews, sui] = await Promise.all([
    loadDaemonViews(),
    loadSuiComponents()
  ]);
  const daemonIds = new Set(daemonViews.map((v) => v.id));
  const extraViews = BAKED_SUI_VIEWS.filter((v) => !daemonIds.has(v.id));
  const views = [...daemonViews, ...extraViews];
  return { lit: [], sui, views };
}

// views/ship-ui-studio/controller.ts
var controller_default = controller({
  scope: () => ({
    sui: signal([]),
    views: signal([]),
    loading: signal(true),
    filterKind: signal("all"),
    filterText: signal("")
  }),
  derived: {
    allItems: ({ sui, views }) => [...sui(), ...views()],
    filtered: ({ filterKind, filterText, allItems }) => {
      const kind = filterKind();
      const text = filterText().toLowerCase();
      return allItems().filter((e) => {
        const kindMatch = kind === "all" || e.kind === kind;
        const textMatch = !text || e.id.toLowerCase().includes(text);
        return kindMatch && textMatch;
      });
    },
    totalCount: ({ allItems }) => allItems().length,
    filteredCount: ({ filtered }) => filtered().length
  },
  mount: async (state) => {
    const catalog = await loadCatalog();
    state.sui = catalog.sui;
    state.views = catalog.views;
    state.loading = false;
  }
});

// views/ship-ui-studio/page.sui
var page_default = view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b9 = (anchor) => bindFor({
    anchor,
    items: () => $scope.filtered(),
    key: (item, __i) => {
      const __child = Object.create($scope);
      __child.item = item;
      __child.__i = __i;
      return (($scope2) => item.id)(__child);
    },
    render: (item, __i) => {
      const __childScope29 = Object.create($scope);
      __childScope29.item = item;
      __childScope29.__i = __i;
      __childScope29.__v = signal2(0);
      return (($scope2) => {
        const __frag30 = document.createDocumentFragment();
        const __c31 = [];
        const __e32 = document.createElement("a");
        __e32.setAttribute("class", "catalog-item");
        {
          const __av33 = "/ship-ui-studio/" + $scope2.item.id;
          if (__av33 == null || __av33 === false || __av33 === "")
            __e32.removeAttribute("href");
          else
            __e32.setAttribute("href", __av33 === true ? "" : String(__av33));
        }
        __e32.setAttribute("data-test", "catalog-item");
        const __e34 = document.createElement("code");
        __e34.setAttribute("class", "catalog-item-name");
        const __t35 = document.createTextNode("");
        __e34.appendChild(__t35);
        __t35.textContent = $scope2.item.id;
        __e32.appendChild(__e34);
        const __e36 = document.createElement("sui-badge");
        {
          const __av37 = $scope2.item.kind === "sui" ? "primary" : "info";
          if (__av37 == null || __av37 === false || __av37 === "")
            __e36.removeAttribute("tone");
          else
            __e36.setAttribute("tone", __av37 === true ? "" : String(__av37));
        }
        __e36.setAttribute("variant", "soft");
        __e36.setAttribute("size", "sm");
        const __t38 = document.createTextNode("");
        __e36.appendChild(__t38);
        __t38.textContent = $scope2.item.kind;
        __e32.appendChild(__e36);
        __frag30.appendChild(__e32);
        return { nodes: Array.from(__frag30.childNodes), cleanup: () => {
          for (const c of __c31)
            try {
              c();
            } catch (e) {}
        }, update: (__new39, __newI40) => {
          $scope2.item = __new39;
          $scope2.__i = __newI40;
          {
            const __av33 = "/ship-ui-studio/" + $scope2.item.id;
            if (__av33 == null || __av33 === false || __av33 === "")
              __e32.removeAttribute("href");
            else
              __e32.setAttribute("href", __av33 === true ? "" : String(__av33));
          }
          __t35.textContent = $scope2.item.id;
          {
            const __av37 = $scope2.item.kind === "sui" ? "primary" : "info";
            if (__av37 == null || __av37 === false || __av37 === "")
              __e36.removeAttribute("tone");
            else
              __e36.setAttribute("tone", __av37 === true ? "" : String(__av37));
          }
          __t38.textContent = $scope2.item.kind;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope29);
    }
  });
  const __wireBoundary_b8 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.loading())
        return 0;
      if ($scope.filtered().length === 0)
        return 1;
      return 2;
    },
    branches: [
      () => {
        const __frag18 = document.createDocumentFragment();
        const __c19 = [];
        const __e20 = document.createElement("div");
        __e20.setAttribute("class", "studio-catalog-loading");
        const __e21 = document.createElement("sui-spinner");
        __e21.setAttribute("size", "sm");
        __e20.appendChild(__e21);
        __frag18.appendChild(__e20);
        return { nodes: Array.from(__frag18.childNodes), cleanup: () => {
          for (const c of __c19)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag22 = document.createDocumentFragment();
        const __c23 = [];
        const __e24 = document.createElement("p");
        __e24.setAttribute("class", "studio-catalog-empty");
        const __t25 = document.createTextNode("No matches");
        __e24.appendChild(__t25);
        __frag22.appendChild(__e24);
        return { nodes: Array.from(__frag22.childNodes), cleanup: () => {
          for (const c of __c23)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag26 = document.createDocumentFragment();
        const __c27 = [];
        const __forAnchor28 = document.createComment("for");
        __frag26.appendChild(__forAnchor28);
        __c27.push(__wireBoundary_b9(__forAnchor28));
        return { nodes: Array.from(__frag26.childNodes), cleanup: () => {
          for (const c of __c27)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b16 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.loading())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag51 = document.createDocumentFragment();
        const __c52 = [];
        const __e53 = document.createElement("div");
        __e53.setAttribute("class", "studio-loading");
        const __e54 = document.createElement("sui-spinner");
        __e54.setAttribute("size", "md");
        __e53.appendChild(__e54);
        const __e55 = document.createElement("p");
        __e55.setAttribute("class", "studio-loading-text");
        const __t56 = document.createTextNode("Loading catalog");
        __e55.appendChild(__t56);
        __e53.appendChild(__e55);
        __frag51.appendChild(__e53);
        return { nodes: Array.from(__frag51.childNodes), cleanup: () => {
          for (const c of __c52)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag57 = document.createDocumentFragment();
        const __c58 = [];
        const __e59 = document.createElement("div");
        __e59.setAttribute("class", "studio-welcome");
        const __e60 = document.createElement("div");
        __e60.setAttribute("class", "studio-welcome-mark");
        __e60.setAttribute("aria-hidden", "true");
        const __t61 = document.createTextNode("◐");
        __e60.appendChild(__t61);
        __e59.appendChild(__e60);
        const __e62 = document.createElement("h2");
        __e62.setAttribute("class", "studio-welcome-title");
        const __t63 = document.createTextNode("SUI Studio");
        __e62.appendChild(__t63);
        __e59.appendChild(__e62);
        const __e64 = document.createElement("p");
        __e64.setAttribute("class", "studio-welcome-desc");
        const __t65 = document.createTextNode("Select a component from the sidebar to preview it.");
        __e64.appendChild(__t65);
        __e59.appendChild(__e64);
        const __e66 = document.createElement("p");
        __e66.setAttribute("class", "studio-welcome-stats");
        const __t67 = document.createTextNode("");
        __e66.appendChild(__t67);
        __c58.push(bindText(__t67, () => $scope.totalCount()));
        const __t68 = document.createTextNode(" components available");
        __e66.appendChild(__t68);
        __e59.appendChild(__e66);
        __frag57.appendChild(__e59);
        return { nodes: Array.from(__frag57.childNodes), cleanup: () => {
          for (const c of __c58)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("div");
      __e0.setAttribute("class", "studio");
      const __e1 = document.createElement("aside");
      __e1.setAttribute("class", "studio-sidebar");
      const __e2 = document.createElement("div");
      __e2.setAttribute("class", "studio-sidebar-header");
      const __e3 = document.createElement("span");
      __e3.setAttribute("class", "studio-brand-mark");
      __e3.setAttribute("aria-hidden", "true");
      const __t4 = document.createTextNode("◐");
      __e3.appendChild(__t4);
      __e2.appendChild(__e3);
      const __e5 = document.createElement("span");
      __e5.setAttribute("class", "studio-sidebar-title");
      const __t6 = document.createTextNode("SUI Studio");
      __e5.appendChild(__t6);
      __e2.appendChild(__e5);
      const __e7 = document.createElement("sui-theme-picker");
      __e7.setAttribute("label-style", "compact");
      __e7.setAttribute("class", "studio-theme-picker");
      __e2.appendChild(__e7);
      __e1.appendChild(__e2);
      const __e8 = document.createElement("sui-input");
      __e8.setAttribute("type", "search");
      __e8.setAttribute("placeholder", "Search components…");
      __cleanups.push(bindAttr(__e8, "value", () => $scope.filterText()));
      __cleanups.push(on(__e8, "sui-input", function() {
        const __h = (e) => $scope.filterText.set(e.detail.value);
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e8.setAttribute("size", "sm");
      __e8.setAttribute("autocomplete", "off");
      __e1.appendChild(__e8);
      const __e9 = document.createElement("div");
      __e9.setAttribute("class", "studio-filter");
      __e9.setAttribute("role", "tablist");
      const __e10 = document.createElement("button");
      __e10.setAttribute("class", "studio-filter-pill");
      __cleanups.push(bindAttr(__e10, "data-active", () => $scope.filterKind() === "all"));
      __cleanups.push(on(__e10, "click", function() {
        const __h = () => $scope.filterKind.set("all");
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t11 = document.createTextNode("All");
      __e10.appendChild(__t11);
      __e9.appendChild(__e10);
      const __e12 = document.createElement("button");
      __e12.setAttribute("class", "studio-filter-pill");
      __cleanups.push(bindAttr(__e12, "data-active", () => $scope.filterKind() === "sui"));
      __cleanups.push(on(__e12, "click", function() {
        const __h = () => $scope.filterKind.set("sui");
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t13 = document.createTextNode("Components");
      __e12.appendChild(__t13);
      __e9.appendChild(__e12);
      const __e14 = document.createElement("button");
      __e14.setAttribute("class", "studio-filter-pill");
      __cleanups.push(bindAttr(__e14, "data-active", () => $scope.filterKind() === "view"));
      __cleanups.push(on(__e14, "click", function() {
        const __h = () => $scope.filterKind.set("view");
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t15 = document.createTextNode("Views");
      __e14.appendChild(__t15);
      __e9.appendChild(__e14);
      __e1.appendChild(__e9);
      const __e16 = document.createElement("nav");
      __e16.setAttribute("class", "studio-catalog-nav");
      __e16.setAttribute("aria-label", "Component catalog");
      const __ifAnchor17 = document.createComment("if");
      __e16.appendChild(__ifAnchor17);
      __cleanups.push(__wireBoundary_b8(__ifAnchor17));
      __e1.appendChild(__e16);
      const __e41 = document.createElement("div");
      __e41.setAttribute("class", "studio-sidebar-counts");
      const __e42 = document.createElement("span");
      __e42.setAttribute("class", "studio-count-text");
      const __t43 = document.createTextNode("");
      __e42.appendChild(__t43);
      __cleanups.push(bindText(__t43, () => $scope.filteredCount()));
      const __t44 = document.createTextNode(" of ");
      __e42.appendChild(__t44);
      const __t45 = document.createTextNode("");
      __e42.appendChild(__t45);
      __cleanups.push(bindText(__t45, () => $scope.totalCount()));
      __e41.appendChild(__e42);
      const __e46 = document.createElement("span");
      __e46.setAttribute("class", "studio-search-hint");
      const __e47 = document.createElement("sui-kbd");
      const __t48 = document.createTextNode("/");
      __e47.appendChild(__t48);
      __e46.appendChild(__e47);
      __e41.appendChild(__e46);
      __e1.appendChild(__e41);
      __e0.appendChild(__e1);
      const __e49 = document.createElement("main");
      __e49.setAttribute("class", "studio-main");
      const __ifAnchor50 = document.createComment("if");
      __e49.appendChild(__ifAnchor50);
      __cleanups.push(__wireBoundary_b16(__ifAnchor50));
      __e0.appendChild(__e49);
      container.appendChild(__e0);
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    },
    hydrate(root, ctx2) {
      const __cleanups = [];
      const __t = collectTextOpens(root);
      const __m = collectElementMarkers(root);
      if (__m["b0"])
        __cleanups.push(hydrateAttr(__m["b0"], "value", () => $scope.filterText()));
      if (__m["b1"])
        __cleanups.push(hydrateOn(__m["b1"], "sui-input", function() {
          const __h = (e) => $scope.filterText.set(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b2"])
        __cleanups.push(hydrateAttr(__m["b2"], "data-active", () => $scope.filterKind() === "all"));
      if (__m["b3"])
        __cleanups.push(hydrateOn(__m["b3"], "click", function() {
          const __h = () => $scope.filterKind.set("all");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b4"])
        __cleanups.push(hydrateAttr(__m["b4"], "data-active", () => $scope.filterKind() === "sui"));
      if (__m["b5"])
        __cleanups.push(hydrateOn(__m["b5"], "click", function() {
          const __h = () => $scope.filterKind.set("sui");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b6"])
        __cleanups.push(hydrateAttr(__m["b6"], "data-active", () => $scope.filterKind() === "view"));
      if (__m["b7"])
        __cleanups.push(hydrateOn(__m["b7"], "click", function() {
          const __h = () => $scope.filterKind.set("view");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b8"]) {
        const __t2 = __m["b8"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b8(__anchor));
      }
      if (__t["b14"])
        __cleanups.push(hydrateText(__t["b14"], () => $scope.filteredCount()));
      if (__t["b15"])
        __cleanups.push(hydrateText(__t["b15"], () => $scope.totalCount()));
      if (__m["b16"]) {
        const __t2 = __m["b16"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b16(__anchor));
      }
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
});
export {
  page_default as default
};
