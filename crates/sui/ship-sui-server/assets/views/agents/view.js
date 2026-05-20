// views/agents/page.sui
import { bindAttr, bindFor, bindIf, bindText, bindWhileError, bindWhileLoading, data, live as live2, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateAttr, hydrateOn, hydrateText, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/agents/controller.ts
import { controller, signal, unwrap, live, derived } from "@ship/sui-runtime";

// views/agents/[id]/hue.ts
var HUES = [
  "orange",
  "amber",
  "red",
  "violet",
  "blue",
  "teal",
  "green",
  "pink",
  "slate"
];
function pickHue(id) {
  if (!id)
    return "slate";
  let h = 0;
  for (let i = 0;i < id.length; i++)
    h = h * 31 + id.charCodeAt(i) | 0;
  return HUES[Math.abs(h) % HUES.length];
}

// views/agents/filter.ts
function matchesKind(a, kind) {
  if (kind === "all")
    return true;
  if (kind === "project")
    return a.source === "PROJECT AGENTS";
  if (kind === "user")
    return a.source === "USER AGENTS";
  return true;
}
function matchesQuery(a, q) {
  if (!q)
    return true;
  const needle = q.toLowerCase().trim();
  if (!needle)
    return true;
  if (a.name.toLowerCase().includes(needle))
    return true;
  if (a.id.toLowerCase().includes(needle))
    return true;
  if ((a.description || "").toLowerCase().includes(needle))
    return true;
  for (const t of a.tags ?? []) {
    if (t.toLowerCase().includes(needle))
      return true;
  }
  return false;
}

// views/agents/controller.ts
var controller_default = controller({
  scope: ({ ship }) => {
    const q = signal("");
    const kind = signal("all");
    const agentEvents = live(() => ship.events.list({ event_type: "agent.", limit: 50 }));
    const agentStatus = derived(() => {
      const env = agentEvents();
      if (env.kind !== "ok")
        return {};
      const v = env.value;
      const rows = (v?.groups ?? []).flatMap((g) => g.rows ?? []);
      rows.sort((a, b) => a.timestamp.localeCompare(b.timestamp));
      const map = {};
      for (const r of rows) {
        if (!r.actor)
          continue;
        if (r.event_type === "agent.started")
          map[r.actor] = "running";
        else if (r.event_type === "agent.completed")
          map[r.actor] = "completed";
        else if (r.event_type === "agent.error" || r.event_type === "agent.errored" || r.event_type === "agent.failed")
          map[r.actor] = "errored";
      }
      return map;
    });
    return { q, kind, agentEvents, agentStatus };
  },
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    filteredItems: ({ agents, q, kind }) => {
      const all = unwrap(agents)?.items ?? [];
      const k = typeof kind === "function" ? kind() : kind;
      const term = typeof q === "function" ? q() : q;
      return all.filter((a) => matchesKind(a, k) && matchesQuery(a, term ?? "")).map((a) => ({ ...a, hue: pickHue(a.id) }));
    },
    activeId: ({ agents }) => {
      const all = unwrap(agents)?.items ?? [];
      return all.find((a) => a.active)?.id ?? "";
    },
    kindFilters: ({ agents }) => {
      const all = unwrap(agents)?.items ?? [];
      const project = all.filter((a) => a.source === "PROJECT AGENTS").length;
      const user = all.filter((a) => a.source === "USER AGENTS").length;
      return [
        { id: "all", label: "All", count: all.length },
        { id: "project", label: "Project", count: project },
        { id: "user", label: "Library", count: user }
      ];
    }
  },
  on: {
    goCreate: ({ navigate }) => navigate("/agents/create"),
    onSearchInput: (s, e) => {
      const target = e.target;
      s.q = target.value ?? "";
    },
    clearSearch: (s) => {
      s.q = "";
    },
    resetFilters: (s) => {
      s.q = "";
      s.kind = "all";
    }
  }
});

// views/agents/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var page_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.agents = live2(() => ship.agents.summaries());
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b4 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.activeId())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag23 = document.createDocumentFragment();
        const __c24 = [];
        const __e25 = document.createElement("span");
        __e25.setAttribute("class", "agents-sub-sep");
        __e25.setAttribute("aria-hidden", "true");
        __frag23.appendChild(__e25);
        const __e26 = document.createElement("span");
        __e26.setAttribute("class", "agents-sub-active");
        __e26.setAttribute("data-test", "active-summary");
        const __t27 = document.createTextNode("active: ");
        __e26.appendChild(__t27);
        const __t28 = document.createTextNode("");
        __e26.appendChild(__t28);
        __c24.push(bindText(__t28, () => $scope.activeId()));
        __frag23.appendChild(__e26);
        return { nodes: Array.from(__frag23.childNodes), cleanup: () => {
          for (const c of __c24)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b9 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.q())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag38 = document.createDocumentFragment();
        const __c39 = [];
        const __e40 = document.createElement("button");
        __e40.setAttribute("type", "button");
        __e40.setAttribute("class", "agents-search-clear");
        __e40.setAttribute("aria-label", "Clear search");
        __c39.push(on(__e40, "click", function() {
          const __h = $scope.clearSearch;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e41 = document.createElement("sui-icon");
        __e41.setAttribute("name", "x");
        __e41.setAttribute("size", "xs");
        __e40.appendChild(__e41);
        __frag38.appendChild(__e40);
        return { nodes: Array.from(__frag38.childNodes), cleanup: () => {
          for (const c of __c39)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b13 = (anchor) => bindFor({
    anchor,
    items: () => $scope.kindFilters(),
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return (($scope2) => f.id)(__child);
    },
    render: (f, __i) => {
      const __childScope44 = Object.create($scope);
      __childScope44.f = f;
      __childScope44.__i = __i;
      return (($scope2) => {
        const __frag45 = document.createDocumentFragment();
        const __c46 = [];
        const __e47 = document.createElement("sui-segmented-option");
        {
          const __av48 = $scope2.f.id;
          if (__av48 == null || __av48 === false || __av48 === "")
            __e47.removeAttribute("value");
          else
            __e47.setAttribute("value", __av48 === true ? "" : String(__av48));
        }
        __e47.setAttribute("data-test", "filter");
        {
          const __av49 = $scope2.f.id;
          if (__av49 == null || __av49 === false || __av49 === "")
            __e47.removeAttribute("data-filter");
          else
            __e47.setAttribute("data-filter", __av49 === true ? "" : String(__av49));
        }
        const __t50 = document.createTextNode("");
        __e47.appendChild(__t50);
        __t50.textContent = $scope2.f.label;
        __frag45.appendChild(__e47);
        return { nodes: Array.from(__frag45.childNodes), cleanup: () => {
          for (const c of __c46)
            try {
              c();
            } catch (e) {}
        }, update: (__new51, __newI52) => {
          const __old = $scope2.f;
          $scope2.f = __new51;
          $scope2.__i = __newI52;
          if (__old.id !== $scope2.f.id) {
            {
              const __av48 = $scope2.f.id;
              if (__av48 == null || __av48 === false || __av48 === "")
                __e47.removeAttribute("value");
              else
                __e47.setAttribute("value", __av48 === true ? "" : String(__av48));
            }
            {
              const __av49 = $scope2.f.id;
              if (__av49 == null || __av49 === false || __av49 === "")
                __e47.removeAttribute("data-filter");
              else
                __e47.setAttribute("data-filter", __av49 === true ? "" : String(__av49));
            }
          }
          if (__old.label !== $scope2.f.label) {
            __t50.textContent = $scope2.f.label;
          }
        } };
      })(__childScope44);
    }
  });
  const __wireBoundary_b18 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1, 2, 3, 4, 5],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope57 = Object.create($scope);
      __childScope57.n = n;
      __childScope57.__i = __i;
      return (($scope2) => {
        const __frag58 = document.createDocumentFragment();
        const __c59 = [];
        const __e60 = document.createElement("div");
        __e60.setAttribute("class", "agents-skel-card");
        __e60.setAttribute("aria-hidden", "true");
        const __e61 = document.createElement("sui-skeleton");
        __e61.setAttribute("shape", "block");
        __e61.setAttribute("size", "md");
        __e61.setAttribute("class", "agents-skel-icon");
        __e60.appendChild(__e61);
        const __e62 = document.createElement("div");
        __e62.setAttribute("class", "agents-skel-body");
        const __e63 = document.createElement("sui-skeleton");
        __e63.setAttribute("shape", "line");
        __e63.setAttribute("size", "md");
        __e63.setAttribute("style", "width:70%");
        __e62.appendChild(__e63);
        const __e64 = document.createElement("sui-skeleton");
        __e64.setAttribute("shape", "line");
        __e64.setAttribute("size", "sm");
        __e64.setAttribute("style", "width:95%");
        __e62.appendChild(__e64);
        const __e65 = document.createElement("sui-skeleton");
        __e65.setAttribute("shape", "line");
        __e65.setAttribute("size", "sm");
        __e65.setAttribute("style", "width:55%");
        __e62.appendChild(__e65);
        __e60.appendChild(__e62);
        __frag58.appendChild(__e60);
        return { nodes: Array.from(__frag58.childNodes), cleanup: () => {
          for (const c of __c59)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope57);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag53 = document.createDocumentFragment();
    const __c54 = [];
    const __e55 = document.createElement("div");
    __e55.setAttribute("class", "agents-grid");
    __e55.setAttribute("data-test", "loading");
    __e55.setAttribute("role", "status");
    __e55.setAttribute("aria-live", "polite");
    __e55.setAttribute("aria-busy", "true");
    __e55.setAttribute("aria-label", "Loading agents");
    const __forAnchor56 = document.createComment("for");
    __e55.appendChild(__forAnchor56);
    __c54.push(__wireBoundary_b18(__forAnchor56));
    __frag53.appendChild(__e55);
    return { nodes: Array.from(__frag53.childNodes), cleanup: () => {
      for (const c of __c54)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope67 = Object.create($scope);
    __childScope67.err = err;
    return (($scope2) => {
      const __frag68 = document.createDocumentFragment();
      const __c69 = [];
      const __e70 = document.createElement("sui-error-state");
      __e70.setAttribute("title", "Couldn't load agents");
      __e70.setAttribute("data-test", "error");
      __e70.setAttribute("role", "alert");
      __e70.setAttribute("aria-live", "assertive");
      const __t71 = document.createTextNode("");
      __e70.appendChild(__t71);
      __c69.push(bindText(__t71, () => $scope2.err.message));
      const __e72 = document.createElement("sui-button");
      __e72.setAttribute("slot", "action");
      __e72.setAttribute("tone", "primary");
      __e72.setAttribute("variant", "solid");
      __c69.push(on(__e72, "click", function() {
        const __h = () => window.location.reload();
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t73 = document.createTextNode("Retry");
      __e72.appendChild(__t73);
      __e70.appendChild(__e72);
      __frag68.appendChild(__e70);
      return { nodes: Array.from(__frag68.childNodes), cleanup: () => {
        for (const c of __c69)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope67);
  };
  const __wireBoundary_b27 = (anchor) => bindFor({
    anchor,
    items: () => $scope.filteredItems(),
    key: (a, __i) => {
      const __child = Object.create($scope);
      __child.a = a;
      __child.__i = __i;
      return (($scope2) => a.id)(__child);
    },
    render: (a, __i) => {
      const __childScope103 = Object.create($scope);
      __childScope103.a = a;
      __childScope103.__i = __i;
      return (($scope2) => {
        __childScope103.__v_id = signal2(0);
        __childScope103.__v_tags = signal2(0);
        const __wireBoundary_b41 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.a.providers.slice(0, 4),
          key: (p, __i2) => {
            const __child = Object.create($scope2);
            __child.p = p;
            __child.__i = __i2;
            return (($scope3) => p)(__child);
          },
          render: (p, __i2) => {
            const __childScope141 = Object.create($scope2);
            __childScope141.p = p;
            __childScope141.__i = __i2;
            return (($scope3) => {
              const __frag142 = document.createDocumentFragment();
              const __c143 = [];
              const __e144 = document.createElement("sui-provider-blob");
              {
                const __av145 = $scope3.p;
                if (__av145 == null || __av145 === false || __av145 === "")
                  __e144.removeAttribute("provider");
                else
                  __e144.setAttribute("provider", __av145 === true ? "" : String(__av145));
              }
              __e144.setAttribute("size", "sm");
              __e144.setAttribute("variant", "glyph");
              {
                const __av146 = $scope3.p;
                if (__av146 == null || __av146 === false || __av146 === "")
                  __e144.removeAttribute("title");
                else
                  __e144.setAttribute("title", __av146 === true ? "" : String(__av146));
              }
              __frag142.appendChild(__e144);
              return { nodes: Array.from(__frag142.childNodes), cleanup: () => {
                for (const c of __c143)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new147, __newI148) => {
                $scope3.p = __new147;
                $scope3.__i = __newI148;
                {
                  const __av145 = $scope3.p;
                  if (__av145 == null || __av145 === false || __av145 === "")
                    __e144.removeAttribute("provider");
                  else
                    __e144.setAttribute("provider", __av145 === true ? "" : String(__av145));
                }
                {
                  const __av146 = $scope3.p;
                  if (__av146 == null || __av146 === false || __av146 === "")
                    __e144.removeAttribute("title");
                  else
                    __e144.setAttribute("title", __av146 === true ? "" : String(__av146));
                }
              } };
            })(__childScope141);
          }
        });
        const __wireBoundary_b44 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.providers.length > 4)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag150 = document.createDocumentFragment();
              const __c151 = [];
              const __e152 = document.createElement("sui-chip");
              __e152.setAttribute("tone", "muted");
              __e152.setAttribute("variant", "outline");
              __e152.setAttribute("size", "sm");
              const __t153 = document.createTextNode("+");
              __e152.appendChild(__t153);
              const __t154 = document.createTextNode("");
              __e152.appendChild(__t154);
              __t154.textContent = $scope2.a.providers.length - 4;
              __frag150.appendChild(__e152);
              return { nodes: Array.from(__frag150.childNodes), cleanup: () => {
                for (const c of __c151)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b40 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.providers && $scope2.a.providers.length > 0)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag137 = document.createDocumentFragment();
              const __c138 = [];
              const __e139 = document.createElement("div");
              __e139.setAttribute("class", "agents-card-providers");
              __e139.setAttribute("aria-label", "Compiles to");
              const __forAnchor140 = document.createComment("for");
              __e139.appendChild(__forAnchor140);
              __c138.push(__wireBoundary_b41(__forAnchor140));
              const __ifAnchor149 = document.createComment("if");
              __e139.appendChild(__ifAnchor149);
              __c138.push(__wireBoundary_b44(__ifAnchor149));
              __frag137.appendChild(__e139);
              return { nodes: Array.from(__frag137.childNodes), cleanup: () => {
                for (const c of __c138)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b47 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.active)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag159 = document.createDocumentFragment();
              const __c160 = [];
              const __e161 = document.createElement("sui-chip");
              __e161.setAttribute("tone", "success");
              __e161.setAttribute("variant", "soft");
              __e161.setAttribute("size", "sm");
              __e161.setAttribute("data-test", "active-chip");
              const __e162 = document.createElement("sui-status-dot");
              __e162.setAttribute("slot", "leading");
              __e162.setAttribute("tone", "success");
              __e162.setAttribute("size", "sm");
              __e162.setAttribute("pulse", "");
              __e162.setAttribute("label", "Active");
              __e161.appendChild(__e162);
              const __t163 = document.createTextNode(`
                      active
                    `);
              __e161.appendChild(__t163);
              __frag159.appendChild(__e161);
              return { nodes: Array.from(__frag159.childNodes), cleanup: () => {
                for (const c of __c160)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b48 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.agentStatus()[$scope2.a.id])
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag165 = document.createDocumentFragment();
              const __c166 = [];
              const __e167 = document.createElement("sui-chip");
              __c166.push(bindAttr(__e167, "tone", () => (void $scope2.__v_id(), $scope2.agentStatus()[$scope2.a.id] === "errored" ? "destructive" : $scope2.agentStatus()[$scope2.a.id] === "completed" ? "success" : "primary")));
              __e167.setAttribute("variant", "soft");
              __e167.setAttribute("size", "sm");
              __e167.setAttribute("data-test", "live-status");
              __c166.push(bindAttr(__e167, "data-status", () => (void $scope2.__v_id(), $scope2.agentStatus()[$scope2.a.id])));
              const __e168 = document.createElement("sui-status-dot");
              __e168.setAttribute("slot", "leading");
              __c166.push(bindAttr(__e168, "tone", () => (void $scope2.__v_id(), $scope2.agentStatus()[$scope2.a.id] === "errored" ? "destructive" : $scope2.agentStatus()[$scope2.a.id] === "completed" ? "success" : "primary")));
              __e168.setAttribute("size", "sm");
              __c166.push(bindAttr(__e168, "pulse", () => (void $scope2.__v_id(), $scope2.agentStatus()[$scope2.a.id] === "running" ? "" : null)));
              __c166.push(bindAttr(__e168, "label", () => (void $scope2.__v_id(), $scope2.agentStatus()[$scope2.a.id])));
              __e167.appendChild(__e168);
              const __t169 = document.createTextNode("");
              __e167.appendChild(__t169);
              __c166.push(bindText(__t169, () => (void $scope2.__v_id(), $scope2.agentStatus()[$scope2.a.id])));
              __frag165.appendChild(__e167);
              return { nodes: Array.from(__frag165.childNodes), cleanup: () => {
                for (const c of __c166)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b55 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => ($scope2.a.tags || []).slice(0, 2),
          key: (t, __i2) => {
            const __child = Object.create($scope2);
            __child.t = t;
            __child.__i = __i2;
            return (($scope3) => t)(__child);
          },
          render: (t, __i2) => {
            const __childScope171 = Object.create($scope2);
            __childScope171.t = t;
            __childScope171.__i = __i2;
            return (($scope3) => {
              const __frag172 = document.createDocumentFragment();
              const __c173 = [];
              const __e174 = document.createElement("sui-chip");
              __e174.setAttribute("tone", "muted");
              __e174.setAttribute("variant", "soft");
              __e174.setAttribute("size", "sm");
              const __t175 = document.createTextNode("");
              __e174.appendChild(__t175);
              __t175.textContent = $scope3.t;
              __frag172.appendChild(__e174);
              return { nodes: Array.from(__frag172.childNodes), cleanup: () => {
                for (const c of __c173)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new176, __newI177) => {
                $scope3.t = __new176;
                $scope3.__i = __newI177;
                __t175.textContent = $scope3.t;
              } };
            })(__childScope171);
          }
        });
        const __wireBoundary_b57 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if (($scope2.a.tags || []).length > 2)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag179 = document.createDocumentFragment();
              const __c180 = [];
              const __e181 = document.createElement("sui-chip");
              __e181.setAttribute("tone", "muted");
              __e181.setAttribute("variant", "outline");
              __e181.setAttribute("size", "sm");
              const __t182 = document.createTextNode("+");
              __e181.appendChild(__t182);
              const __t183 = document.createTextNode("");
              __e181.appendChild(__t183);
              __c180.push(bindText(__t183, () => (void $scope2.__v_tags(), ($scope2.a.tags || []).length - 2)));
              __frag179.appendChild(__e181);
              return { nodes: Array.from(__frag179.childNodes), cleanup: () => {
                for (const c of __c180)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag104 = document.createDocumentFragment();
        const __c105 = [];
        const __e106 = document.createElement("sui-list-tile");
        __e106.setAttribute("variant", "card");
        __e106.setAttribute("density", "normal");
        __e106.setAttribute("interactive", "");
        {
          const __av107 = "/agents/" + $scope2.a.id;
          if (__av107 == null || __av107 === false || __av107 === "")
            __e106.removeAttribute("navigable");
          else
            __e106.setAttribute("navigable", __av107 === true ? "" : String(__av107));
        }
        __e106.setAttribute("class", "agents-card-tile");
        __e106.setAttribute("data-test", "agent");
        {
          const __av108 = $scope2.a.id;
          if (__av108 == null || __av108 === false || __av108 === "")
            __e106.removeAttribute("data-id");
          else
            __e106.setAttribute("data-id", __av108 === true ? "" : String(__av108));
        }
        {
          const __av109 = $scope2.a.active ? "" : null;
          if (__av109 == null || __av109 === false || __av109 === "")
            __e106.removeAttribute("data-active");
          else
            __e106.setAttribute("data-active", __av109 === true ? "" : String(__av109));
        }
        const __e110 = document.createElement("div");
        {
          const __av111 = "agents-card-icon-tile hue-" + $scope2.a.hue;
          if (__av111 == null || __av111 === false || __av111 === "")
            __e110.removeAttribute("class");
          else
            __e110.setAttribute("class", __av111 === true ? "" : String(__av111));
        }
        __e110.setAttribute("slot", "leading");
        __e110.setAttribute("aria-hidden", "true");
        __e110.setAttribute("data-test", "hue-tile");
        const __e112 = document.createElement("span");
        __e112.setAttribute("class", "agents-card-letter");
        const __t113 = document.createTextNode("");
        __e112.appendChild(__t113);
        __t113.textContent = $scope2.a.letter;
        __e110.appendChild(__e112);
        __e106.appendChild(__e110);
        const __e114 = document.createElement("div");
        __e114.setAttribute("class", "agents-card-body");
        const __e115 = document.createElement("div");
        __e115.setAttribute("class", "agents-card-head");
        const __e116 = document.createElement("div");
        __e116.setAttribute("class", "agents-card-title");
        const __t117 = document.createTextNode("");
        __e116.appendChild(__t117);
        __t117.textContent = $scope2.a.name;
        __e115.appendChild(__e116);
        const __e118 = document.createElement("p");
        __e118.setAttribute("class", "agents-card-desc");
        const __t119 = document.createTextNode("");
        __e118.appendChild(__t119);
        __t119.textContent = $scope2.a.description || "No description";
        __e115.appendChild(__e118);
        __e114.appendChild(__e115);
        const __e120 = document.createElement("div");
        __e120.setAttribute("class", "agents-card-stats");
        const __e121 = document.createElement("span");
        const __t122 = document.createTextNode("");
        __e121.appendChild(__t122);
        __t122.textContent = $scope2.a.skills_count;
        const __t123 = document.createTextNode(" skill");
        __e121.appendChild(__t123);
        const __t124 = document.createTextNode("");
        __e121.appendChild(__t124);
        __t124.textContent = $scope2.a.skills_count === 1 ? "" : "s";
        __e120.appendChild(__e121);
        const __e125 = document.createElement("span");
        __e125.setAttribute("class", "agents-card-sep");
        const __t126 = document.createTextNode("·");
        __e125.appendChild(__t126);
        __e120.appendChild(__e125);
        const __e127 = document.createElement("span");
        const __t128 = document.createTextNode("");
        __e127.appendChild(__t128);
        __t128.textContent = $scope2.a.mcp_count;
        const __t129 = document.createTextNode(" MCP");
        __e127.appendChild(__t129);
        __e120.appendChild(__e127);
        const __e130 = document.createElement("span");
        __e130.setAttribute("class", "agents-card-sep");
        const __t131 = document.createTextNode("·");
        __e130.appendChild(__t131);
        __e120.appendChild(__e130);
        const __e132 = document.createElement("span");
        const __t133 = document.createTextNode("");
        __e132.appendChild(__t133);
        __t133.textContent = $scope2.a.rules_count;
        const __t134 = document.createTextNode(" rule");
        __e132.appendChild(__t134);
        const __t135 = document.createTextNode("");
        __e132.appendChild(__t135);
        __t135.textContent = $scope2.a.rules_count === 1 ? "" : "s";
        __e120.appendChild(__e132);
        __e114.appendChild(__e120);
        const __ifAnchor136 = document.createComment("if");
        __e114.appendChild(__ifAnchor136);
        __c105.push(__wireBoundary_b40(__ifAnchor136));
        const __e155 = document.createElement("div");
        __e155.setAttribute("class", "agents-card-badges");
        const __e156 = document.createElement("sui-badge");
        __e156.setAttribute("tone", "muted");
        __e156.setAttribute("variant", "outline");
        __e156.setAttribute("size", "sm");
        __e156.setAttribute("class", "agents-card-badge-source");
        const __t157 = document.createTextNode("");
        __e156.appendChild(__t157);
        __t157.textContent = $scope2.a.source;
        __e155.appendChild(__e156);
        const __ifAnchor158 = document.createComment("if");
        __e155.appendChild(__ifAnchor158);
        __c105.push(__wireBoundary_b47(__ifAnchor158));
        const __ifAnchor164 = document.createComment("if");
        __e155.appendChild(__ifAnchor164);
        __c105.push(__wireBoundary_b48(__ifAnchor164));
        const __forAnchor170 = document.createComment("for");
        __e155.appendChild(__forAnchor170);
        __c105.push(__wireBoundary_b55(__forAnchor170));
        const __ifAnchor178 = document.createComment("if");
        __e155.appendChild(__ifAnchor178);
        __c105.push(__wireBoundary_b57(__ifAnchor178));
        __e114.appendChild(__e155);
        __e106.appendChild(__e114);
        __frag104.appendChild(__e106);
        return { nodes: Array.from(__frag104.childNodes), cleanup: () => {
          for (const c of __c105)
            try {
              c();
            } catch (e) {}
        }, update: (__new184, __newI185) => {
          const __old = $scope2.a;
          $scope2.a = __new184;
          $scope2.__i = __newI185;
          if (__old.id !== $scope2.a.id) {
            {
              const __av107 = "/agents/" + $scope2.a.id;
              if (__av107 == null || __av107 === false || __av107 === "")
                __e106.removeAttribute("navigable");
              else
                __e106.setAttribute("navigable", __av107 === true ? "" : String(__av107));
            }
            {
              const __av108 = $scope2.a.id;
              if (__av108 == null || __av108 === false || __av108 === "")
                __e106.removeAttribute("data-id");
              else
                __e106.setAttribute("data-id", __av108 === true ? "" : String(__av108));
            }
          }
          if (__old.active !== $scope2.a.active) {
            {
              const __av109 = $scope2.a.active ? "" : null;
              if (__av109 == null || __av109 === false || __av109 === "")
                __e106.removeAttribute("data-active");
              else
                __e106.setAttribute("data-active", __av109 === true ? "" : String(__av109));
            }
          }
          if (__old.hue !== $scope2.a.hue) {
            {
              const __av111 = "agents-card-icon-tile hue-" + $scope2.a.hue;
              if (__av111 == null || __av111 === false || __av111 === "")
                __e110.removeAttribute("class");
              else
                __e110.setAttribute("class", __av111 === true ? "" : String(__av111));
            }
          }
          if (__old.letter !== $scope2.a.letter) {
            __t113.textContent = $scope2.a.letter;
          }
          if (__old.name !== $scope2.a.name) {
            __t117.textContent = $scope2.a.name;
          }
          if (__old.description !== $scope2.a.description) {
            __t119.textContent = $scope2.a.description || "No description";
          }
          if (__old.skills_count !== $scope2.a.skills_count) {
            __t122.textContent = $scope2.a.skills_count;
            __t124.textContent = $scope2.a.skills_count === 1 ? "" : "s";
          }
          if (__old.mcp_count !== $scope2.a.mcp_count) {
            __t128.textContent = $scope2.a.mcp_count;
          }
          if (__old.rules_count !== $scope2.a.rules_count) {
            __t133.textContent = $scope2.a.rules_count;
            __t135.textContent = $scope2.a.rules_count === 1 ? "" : "s";
          }
          if (__old.source !== $scope2.a.source) {
            __t157.textContent = $scope2.a.source;
          }
          if (__old.id !== $scope2.a.id) {
            $scope2.__v_id.set((c) => c + 1);
          }
          if (__old.tags !== $scope2.a.tags) {
            $scope2.__v_tags.set((c) => c + 1);
          }
        } };
      })(__childScope103);
    }
  });
  const __wireBoundary_b23 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.items().length === 0)
        return 0;
      if ($scope.filteredItems().length === 0)
        return 1;
      return 2;
    },
    branches: [
      () => {
        const __frag79 = document.createDocumentFragment();
        const __c80 = [];
        const __e81 = document.createElement("sui-empty-state");
        __e81.setAttribute("icon", "sparkles");
        __e81.setAttribute("variant", "default");
        __e81.setAttribute("size", "sm");
        __e81.setAttribute("data-test", "empty");
        __e81.setAttribute("role", "status");
        const __e82 = document.createElement("span");
        __e82.setAttribute("slot", "title");
        const __t83 = document.createTextNode("No agents yet");
        __e82.appendChild(__t83);
        __e81.appendChild(__e82);
        const __t84 = document.createTextNode(`
          Drop a JSONC file under
          `);
        __e81.appendChild(__t84);
        const __e85 = document.createElement("code");
        __e85.setAttribute("class", "agents-empty-code");
        const __t86 = document.createTextNode(".ship/agents/");
        __e85.appendChild(__t86);
        __e81.appendChild(__e85);
        const __t87 = document.createTextNode(`
          or create one from the Studio.
          `);
        __e81.appendChild(__t87);
        const __e88 = document.createElement("sui-button");
        __e88.setAttribute("slot", "action");
        __e88.setAttribute("tone", "primary");
        __e88.setAttribute("variant", "solid");
        __c80.push(on(__e88, "click", function() {
          const __h = $scope.goCreate;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e88.setAttribute("data-test", "cta-create-first");
        const __e89 = document.createElement("sui-icon");
        __e89.setAttribute("slot", "leading");
        __e89.setAttribute("name", "plus");
        __e88.appendChild(__e89);
        const __t90 = document.createTextNode(`
            Create your first agent
          `);
        __e88.appendChild(__t90);
        __e81.appendChild(__e88);
        __frag79.appendChild(__e81);
        return { nodes: Array.from(__frag79.childNodes), cleanup: () => {
          for (const c of __c80)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag91 = document.createDocumentFragment();
        const __c92 = [];
        const __e93 = document.createElement("sui-empty-state");
        __e93.setAttribute("icon", "search-x");
        __e93.setAttribute("variant", "default");
        __e93.setAttribute("size", "sm");
        __e93.setAttribute("data-test", "empty-filtered");
        __e93.setAttribute("role", "status");
        const __e94 = document.createElement("span");
        __e94.setAttribute("slot", "title");
        const __t95 = document.createTextNode("No matching agents");
        __e94.appendChild(__t95);
        __e93.appendChild(__e94);
        const __t96 = document.createTextNode("");
        __e93.appendChild(__t96);
        __c92.push(bindText(__t96, () => $scope.q() ? 'Nothing matches "' + $scope.q() + '"' : "No agents in this scope."));
        const __e97 = document.createElement("sui-button");
        __e97.setAttribute("slot", "action");
        __e97.setAttribute("variant", "outline");
        __e97.setAttribute("size", "sm");
        __c92.push(on(__e97, "click", function() {
          const __h = $scope.resetFilters;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t98 = document.createTextNode(`
            Clear filters
          `);
        __e97.appendChild(__t98);
        __e93.appendChild(__e97);
        __frag91.appendChild(__e93);
        return { nodes: Array.from(__frag91.childNodes), cleanup: () => {
          for (const c of __c92)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag99 = document.createDocumentFragment();
        const __c100 = [];
        const __e101 = document.createElement("div");
        __e101.setAttribute("class", "agents-grid");
        __e101.setAttribute("data-view-enter", "");
        __e101.setAttribute("data-test", "grid");
        const __forAnchor102 = document.createComment("for");
        __e101.appendChild(__forAnchor102);
        __c100.push(__wireBoundary_b27(__forAnchor102));
        __frag99.appendChild(__e101);
        return { nodes: Array.from(__frag99.childNodes), cleanup: () => {
          for (const c of __c100)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b22 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.agents().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag76 = document.createDocumentFragment();
        const __c77 = [];
        const __ifAnchor78 = document.createComment("if");
        __frag76.appendChild(__ifAnchor78);
        __c77.push(__wireBoundary_b23(__ifAnchor78));
        return { nodes: Array.from(__frag76.childNodes), cleanup: () => {
          for (const c of __c77)
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
      __e0.setAttribute("class", "ship-view agents-view");
      const __e1 = document.createElement("div");
      __e1.setAttribute("class", "agents-wrap");
      const __e2 = document.createElement("header");
      __e2.setAttribute("class", "agents-head");
      const __e3 = document.createElement("div");
      __e3.setAttribute("class", "agents-head-titles");
      const __e4 = document.createElement("h1");
      __e4.setAttribute("class", "agents-title");
      const __t5 = document.createTextNode("Agents");
      __e4.appendChild(__t5);
      __e3.appendChild(__e4);
      const __e6 = document.createElement("p");
      __e6.setAttribute("class", "agents-sub");
      const __e7 = document.createElement("strong");
      const __t8 = document.createTextNode("");
      __e7.appendChild(__t8);
      __cleanups.push(bindText(__t8, () => $scope.items().length));
      __e6.appendChild(__e7);
      const __t9 = document.createTextNode(" agent");
      __e6.appendChild(__t9);
      const __t10 = document.createTextNode("");
      __e6.appendChild(__t10);
      __cleanups.push(bindText(__t10, () => $scope.items().length === 1 ? "" : "s"));
      const __e11 = document.createElement("span");
      __e11.setAttribute("class", "agents-sub-sep");
      __e11.setAttribute("aria-hidden", "true");
      __e6.appendChild(__e11);
      const __e12 = document.createElement("strong");
      const __t13 = document.createTextNode("");
      __e12.appendChild(__t13);
      __cleanups.push(bindText(__t13, () => $scope.kindFilters()[1].count));
      __e6.appendChild(__e12);
      const __t14 = document.createTextNode(` project
          `);
      __e6.appendChild(__t14);
      const __e15 = document.createElement("span");
      __e15.setAttribute("class", "agents-sub-sep");
      __e15.setAttribute("aria-hidden", "true");
      __e6.appendChild(__e15);
      const __e16 = document.createElement("strong");
      const __t17 = document.createTextNode("");
      __e16.appendChild(__t17);
      __cleanups.push(bindText(__t17, () => $scope.kindFilters()[2].count));
      __e6.appendChild(__e16);
      const __t18 = document.createTextNode(` library
          `);
      __e6.appendChild(__t18);
      const __e19 = document.createElement("span");
      __e19.setAttribute("class", "agents-sub-sep");
      __e19.setAttribute("aria-hidden", "true");
      __e6.appendChild(__e19);
      const __e20 = document.createElement("code");
      __e20.setAttribute("class", "agents-sub-code");
      const __t21 = document.createTextNode(".ship/agents/");
      __e20.appendChild(__t21);
      __e6.appendChild(__e20);
      const __ifAnchor22 = document.createComment("if");
      __e6.appendChild(__ifAnchor22);
      __cleanups.push(__wireBoundary_b4(__ifAnchor22));
      __e3.appendChild(__e6);
      const __e29 = document.createElement("div");
      __e29.setAttribute("class", "agents-rule");
      __e29.setAttribute("aria-hidden", "true");
      __e3.appendChild(__e29);
      __e2.appendChild(__e3);
      const __e30 = document.createElement("sui-button");
      __e30.setAttribute("tone", "primary");
      __e30.setAttribute("variant", "solid");
      __cleanups.push(on(__e30, "click", function() {
        const __h = $scope.goCreate;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e30.setAttribute("data-test", "cta-new");
      const __e31 = document.createElement("sui-icon");
      __e31.setAttribute("slot", "leading");
      __e31.setAttribute("name", "plus");
      __e30.appendChild(__e31);
      const __t32 = document.createTextNode(`
        New agent
      `);
      __e30.appendChild(__t32);
      __e2.appendChild(__e30);
      __e1.appendChild(__e2);
      const __e33 = document.createElement("div");
      __e33.setAttribute("class", "agents-filters");
      const __e34 = document.createElement("div");
      __e34.setAttribute("class", "agents-search");
      const __e35 = document.createElement("sui-icon");
      __e35.setAttribute("name", "search");
      __e35.setAttribute("size", "sm");
      __e35.setAttribute("tone", "muted");
      __e34.appendChild(__e35);
      const __e36 = document.createElement("input");
      __e36.setAttribute("type", "text");
      __e36.setAttribute("placeholder", "Search by name, description, or tag…");
      __e36.setAttribute("aria-label", "Search agents");
      __e36.setAttribute("spellcheck", "false");
      __e36.setAttribute("data-test", "search");
      __cleanups.push(bindAttr(__e36, "value", () => $scope.q()));
      __cleanups.push(on(__e36, "input", function() {
        const __h = $scope.onSearchInput;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e34.appendChild(__e36);
      const __ifAnchor37 = document.createComment("if");
      __e34.appendChild(__ifAnchor37);
      __cleanups.push(__wireBoundary_b9(__ifAnchor37));
      __e33.appendChild(__e34);
      const __e42 = document.createElement("sui-segmented");
      __cleanups.push(bindAttr(__e42, "value", () => $scope.kind()));
      __e42.setAttribute("size", "sm");
      __e42.setAttribute("aria-label", "Filter by source");
      __cleanups.push(on(__e42, "sui-change", function() {
        const __h = (e) => $scope.kind.set(e.detail.value);
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __forAnchor43 = document.createComment("for");
      __e42.appendChild(__forAnchor43);
      __cleanups.push(__wireBoundary_b13(__forAnchor43));
      __e33.appendChild(__e42);
      __e1.appendChild(__e33);
      const __wlAnchor66 = document.createComment("while-loading");
      __e1.appendChild(__wlAnchor66);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor66, signal: $scope.agents, render: __renderWhileLoading_0 }));
      const __weAnchor74 = document.createComment("while-error");
      __e1.appendChild(__weAnchor74);
      __cleanups.push(bindWhileError({ anchor: __weAnchor74, signal: $scope.agents, render: __renderWhileError_0 }));
      const __ifAnchor75 = document.createComment("if");
      __e1.appendChild(__ifAnchor75);
      __cleanups.push(__wireBoundary_b22(__ifAnchor75));
      __e0.appendChild(__e1);
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
      if (__t["b0"])
        __cleanups.push(hydrateText(__t["b0"], () => $scope.items().length));
      if (__t["b1"])
        __cleanups.push(hydrateText(__t["b1"], () => $scope.items().length === 1 ? "" : "s"));
      if (__t["b2"])
        __cleanups.push(hydrateText(__t["b2"], () => $scope.kindFilters()[1].count));
      if (__t["b3"])
        __cleanups.push(hydrateText(__t["b3"], () => $scope.kindFilters()[2].count));
      if (__m["b4"]) {
        const __t2 = __m["b4"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b4(__anchor));
      }
      if (__m["b6"])
        __cleanups.push(hydrateOn(__m["b6"], "click", function() {
          const __h = $scope.goCreate;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b7"])
        __cleanups.push(hydrateAttr(__m["b7"], "value", () => $scope.q()));
      if (__m["b8"])
        __cleanups.push(hydrateOn(__m["b8"], "input", function() {
          const __h = $scope.onSearchInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b9"]) {
        const __t2 = __m["b9"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b9(__anchor));
      }
      if (__m["b11"])
        __cleanups.push(hydrateAttr(__m["b11"], "value", () => $scope.kind()));
      if (__m["b12"])
        __cleanups.push(hydrateOn(__m["b12"], "sui-change", function() {
          const __h = (e) => $scope.kind.set(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b13"]) {
        const __t2 = __m["b13"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("for") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b13(__anchor));
      }
      if (__m["b17"])
        __cleanups.push(hydrateWhileLoading(__m["b17"], "b17", () => $scope.agents(), __renderWhileLoading_0));
      if (__m["b19"])
        __cleanups.push(hydrateWhileError(__m["b19"], "b19", () => $scope.agents(), __renderWhileError_0));
      if (__m["b22"]) {
        const __t2 = __m["b22"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b22(__anchor));
      }
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "01329411", dirty: true, builtAt: "2026-05-19T16:54:50Z" } });
export {
  page_default as default
};
