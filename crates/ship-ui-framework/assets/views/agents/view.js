// views/agents/view.js
import { bindAttr, bindFor, bindIf, bindText, bindWhileError, bindWhileLoading, data, live, on, view } from "@ship/ui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateOn, hydrateText, hydrateWhileError, hydrateWhileLoading } from "@ship/ui-dom";

// views/agents/view.controller.ts
import { controller, unwrap } from "@ship/ui-runtime";
var view_controller_default = controller({
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    total: ({ agents }) => unwrap(agents)?.items.length ?? 0
  },
  on: {
    newAgent: ({ navigate }) => navigate("/agents/create")
  }
});

// views/agents/view.js
var view_default = view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = /* @__PURE__ */ Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.agents = live(({ ship: ship2, route: route2 }) => ship2.agents.summaries());
  if (view_controller_default && typeof view_controller_default.setup === "function") {
    const __r = view_controller_default.setup(ctx);
    if (__r && typeof __r === "object") Object.assign($scope, __r);
  }
  const __renderWhileLoading_0 = () => {
    const __frag13 = document.createDocumentFragment();
    const __c14 = [];
    const __e15 = document.createElement("p");
    __e15.setAttribute("class", "loading-copy");
    __e15.setAttribute("data-test", "loading");
    const __t16 = document.createTextNode("Loading agents&hellip;");
    __e15.appendChild(__t16);
    __frag13.appendChild(__e15);
    return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
      for (const c of __c14) try {
        c();
      } catch (e) {
      }
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope18 = Object.create($scope);
    __childScope18.err = err;
    return (($scope2) => {
      const __frag19 = document.createDocumentFragment();
      const __c20 = [];
      const __e21 = document.createElement("p");
      __e21.setAttribute("class", "loading-copy");
      __e21.setAttribute("data-test", "error");
      const __t22 = document.createTextNode("Error loading agents: ");
      __e21.appendChild(__t22);
      const __t23 = document.createTextNode("");
      __e21.appendChild(__t23);
      __c20.push(bindText(__t23, () => $scope2.err.message));
      __frag19.appendChild(__e21);
      return { nodes: Array.from(__frag19.childNodes), cleanup: () => {
        for (const c of __c20) try {
          c();
        } catch (e) {
        }
      } };
    })(__childScope18);
  };
  const __wireBoundary_b9 = (anchor) => bindFor({
    anchor,
    items: () => $scope.items(),
    key: (a, __i) => {
      const __child = Object.create($scope);
      __child.a = a;
      __child.__i = __i;
      return (($scope2) => a.id)(__child);
    },
    render: (a, __i) => {
      const __childScope45 = Object.create($scope);
      __childScope45.a = a;
      __childScope45.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b20 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.a.providers.slice(0, 4),
          render: (p, __i2) => {
            const __childScope79 = Object.create($scope2);
            __childScope79.p = p;
            __childScope79.__i = __i2;
            return (($scope3) => {
              const __frag80 = document.createDocumentFragment();
              const __c81 = [];
              const __e82 = document.createElement("ship-provider-blob");
              __c81.push(bindAttr(__e82, "provider", () => $scope3.p));
              __e82.setAttribute("size", "sm");
              __e82.setAttribute("variant", "glyph");
              __c81.push(bindAttr(__e82, "title", () => $scope3.p));
              __frag80.appendChild(__e82);
              return { nodes: Array.from(__frag80.childNodes), cleanup: () => {
                for (const c of __c81) try {
                  c();
                } catch (e) {
                }
              } };
            })(__childScope79);
          }
        });
        const __wireBoundary_b23 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.providers.length > 4) return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag84 = document.createDocumentFragment();
              const __c85 = [];
              const __e86 = document.createElement("ship-chip");
              __e86.setAttribute("tone", "muted");
              __e86.setAttribute("variant", "outline");
              __e86.setAttribute("size", "sm");
              const __t87 = document.createTextNode("+");
              __e86.appendChild(__t87);
              const __t88 = document.createTextNode("");
              __e86.appendChild(__t88);
              __c85.push(bindText(__t88, () => $scope2.a.providers.length - 4));
              __frag84.appendChild(__e86);
              return { nodes: Array.from(__frag84.childNodes), cleanup: () => {
                for (const c of __c85) try {
                  c();
                } catch (e) {
                }
              } };
            }
          ]
        });
        const __wireBoundary_b19 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.providers && $scope2.a.providers.length > 0) return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag75 = document.createDocumentFragment();
              const __c76 = [];
              const __e77 = document.createElement("div");
              __e77.setAttribute("class", "agents-card-providers");
              __e77.setAttribute("aria-label", "Compiles to");
              const __forAnchor78 = document.createComment("for");
              __e77.appendChild(__forAnchor78);
              __c76.push(__wireBoundary_b20(__forAnchor78));
              const __ifAnchor83 = document.createComment("if");
              __e77.appendChild(__ifAnchor83);
              __c76.push(__wireBoundary_b23(__ifAnchor83));
              __frag75.appendChild(__e77);
              return { nodes: Array.from(__frag75.childNodes), cleanup: () => {
                for (const c of __c76) try {
                  c();
                } catch (e) {
                }
              } };
            }
          ]
        });
        const __wireBoundary_b26 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.active) return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag93 = document.createDocumentFragment();
              const __c94 = [];
              const __e95 = document.createElement("ship-chip");
              __e95.setAttribute("tone", "primary");
              __e95.setAttribute("variant", "soft");
              __e95.setAttribute("size", "sm");
              const __e96 = document.createElement("sui-status-dot");
              __e96.setAttribute("slot", "leading");
              __e96.setAttribute("tone", "primary");
              __e96.setAttribute("size", "sm");
              __e96.setAttribute("pulse", "");
              __e96.setAttribute("label", "Active session");
              __e95.appendChild(__e96);
              const __t97 = document.createTextNode("\n                      active session\n                    ");
              __e95.appendChild(__t97);
              __frag93.appendChild(__e95);
              return { nodes: Array.from(__frag93.childNodes), cleanup: () => {
                for (const c of __c94) try {
                  c();
                } catch (e) {
                }
              } };
            }
          ]
        });
        const __wireBoundary_b27 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => ($scope2.a.tags || []).slice(0, 2),
          render: (t, __i2) => {
            const __childScope99 = Object.create($scope2);
            __childScope99.t = t;
            __childScope99.__i = __i2;
            return (($scope3) => {
              const __frag100 = document.createDocumentFragment();
              const __c101 = [];
              const __e102 = document.createElement("ship-chip");
              __e102.setAttribute("tone", "muted");
              __e102.setAttribute("variant", "soft");
              __e102.setAttribute("size", "sm");
              const __t103 = document.createTextNode("");
              __e102.appendChild(__t103);
              __c101.push(bindText(__t103, () => $scope3.t));
              __frag100.appendChild(__e102);
              return { nodes: Array.from(__frag100.childNodes), cleanup: () => {
                for (const c of __c101) try {
                  c();
                } catch (e) {
                }
              } };
            })(__childScope99);
          }
        });
        const __wireBoundary_b29 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if (($scope2.a.tags || []).length > 2) return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag105 = document.createDocumentFragment();
              const __c106 = [];
              const __e107 = document.createElement("ship-chip");
              __e107.setAttribute("tone", "muted");
              __e107.setAttribute("variant", "outline");
              __e107.setAttribute("size", "sm");
              const __t108 = document.createTextNode("+");
              __e107.appendChild(__t108);
              const __t109 = document.createTextNode("");
              __e107.appendChild(__t109);
              __c106.push(bindText(__t109, () => ($scope2.a.tags || []).length - 2));
              __frag105.appendChild(__e107);
              return { nodes: Array.from(__frag105.childNodes), cleanup: () => {
                for (const c of __c106) try {
                  c();
                } catch (e) {
                }
              } };
            }
          ]
        });
        const __frag46 = document.createDocumentFragment();
        const __c47 = [];
        const __e48 = document.createElement("ship-list-tile");
        __e48.setAttribute("variant", "card");
        __e48.setAttribute("density", "normal");
        __e48.setAttribute("interactive", "");
        __c47.push(bindAttr(__e48, "navigable", () => "/agents/" + $scope2.a.id));
        __e48.setAttribute("class", "agents-card-tile");
        __e48.setAttribute("data-test", "agent");
        const __e49 = document.createElement("div");
        __e49.setAttribute("class", "agents-card-icon-tile");
        __e49.setAttribute("slot", "leading");
        const __e50 = document.createElement("span");
        __e50.setAttribute("class", "agents-card-letter");
        const __t51 = document.createTextNode("");
        __e50.appendChild(__t51);
        __c47.push(bindText(__t51, () => $scope2.a.letter));
        __e49.appendChild(__e50);
        __e48.appendChild(__e49);
        const __e52 = document.createElement("div");
        __e52.setAttribute("class", "agents-card-body");
        const __e53 = document.createElement("div");
        __e53.setAttribute("class", "agents-card-head");
        const __e54 = document.createElement("div");
        __e54.setAttribute("class", "agents-card-title");
        const __t55 = document.createTextNode("");
        __e54.appendChild(__t55);
        __c47.push(bindText(__t55, () => $scope2.a.name));
        __e53.appendChild(__e54);
        const __e56 = document.createElement("p");
        __e56.setAttribute("class", "agents-card-desc");
        const __t57 = document.createTextNode("");
        __e56.appendChild(__t57);
        __c47.push(bindText(__t57, () => $scope2.a.description || "No description"));
        __e53.appendChild(__e56);
        __e52.appendChild(__e53);
        const __e58 = document.createElement("div");
        __e58.setAttribute("class", "agents-card-stats");
        const __e59 = document.createElement("span");
        const __t60 = document.createTextNode("");
        __e59.appendChild(__t60);
        __c47.push(bindText(__t60, () => $scope2.a.skills_count));
        const __t61 = document.createTextNode(" skill");
        __e59.appendChild(__t61);
        const __t62 = document.createTextNode("");
        __e59.appendChild(__t62);
        __c47.push(bindText(__t62, () => $scope2.a.skills_count === 1 ? "" : "s"));
        __e58.appendChild(__e59);
        const __e63 = document.createElement("span");
        __e63.setAttribute("class", "agents-card-sep");
        const __t64 = document.createTextNode("&middot;");
        __e63.appendChild(__t64);
        __e58.appendChild(__e63);
        const __e65 = document.createElement("span");
        const __t66 = document.createTextNode("");
        __e65.appendChild(__t66);
        __c47.push(bindText(__t66, () => $scope2.a.mcp_count));
        const __t67 = document.createTextNode(" MCP");
        __e65.appendChild(__t67);
        __e58.appendChild(__e65);
        const __e68 = document.createElement("span");
        __e68.setAttribute("class", "agents-card-sep");
        const __t69 = document.createTextNode("&middot;");
        __e68.appendChild(__t69);
        __e58.appendChild(__e68);
        const __e70 = document.createElement("span");
        const __t71 = document.createTextNode("");
        __e70.appendChild(__t71);
        __c47.push(bindText(__t71, () => $scope2.a.rules_count));
        const __t72 = document.createTextNode(" rule");
        __e70.appendChild(__t72);
        const __t73 = document.createTextNode("");
        __e70.appendChild(__t73);
        __c47.push(bindText(__t73, () => $scope2.a.rules_count === 1 ? "" : "s"));
        __e58.appendChild(__e70);
        __e52.appendChild(__e58);
        const __ifAnchor74 = document.createComment("if");
        __e52.appendChild(__ifAnchor74);
        __c47.push(__wireBoundary_b19(__ifAnchor74));
        const __e89 = document.createElement("div");
        __e89.setAttribute("class", "agents-card-badges");
        const __e90 = document.createElement("ship-badge");
        __e90.setAttribute("tone", "muted");
        __e90.setAttribute("variant", "outline");
        __e90.setAttribute("size", "sm");
        __e90.setAttribute("class", "agents-card-badge-source");
        const __t91 = document.createTextNode("");
        __e90.appendChild(__t91);
        __c47.push(bindText(__t91, () => $scope2.a.source));
        __e89.appendChild(__e90);
        const __ifAnchor92 = document.createComment("if");
        __e89.appendChild(__ifAnchor92);
        __c47.push(__wireBoundary_b26(__ifAnchor92));
        const __forAnchor98 = document.createComment("for");
        __e89.appendChild(__forAnchor98);
        __c47.push(__wireBoundary_b27(__forAnchor98));
        const __ifAnchor104 = document.createComment("if");
        __e89.appendChild(__ifAnchor104);
        __c47.push(__wireBoundary_b29(__ifAnchor104));
        __e52.appendChild(__e89);
        __e48.appendChild(__e52);
        __frag46.appendChild(__e48);
        return { nodes: Array.from(__frag46.childNodes), cleanup: () => {
          for (const c of __c47) try {
            c();
          } catch (e) {
          }
        } };
      })(__childScope45);
    }
  });
  const __wireBoundary_b7 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.items().length === 0) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag29 = document.createDocumentFragment();
        const __c30 = [];
        const __e31 = document.createElement("ship-empty-state");
        __e31.setAttribute("icon", "sparkles");
        __e31.setAttribute("variant", "default");
        __e31.setAttribute("size", "lg");
        __e31.setAttribute("data-test", "empty");
        const __e32 = document.createElement("span");
        __e32.setAttribute("slot", "title");
        const __t33 = document.createTextNode("No agents yet");
        __e32.appendChild(__t33);
        __e31.appendChild(__e32);
        const __t34 = document.createTextNode("\n          Drop a JSONC file under\n          ");
        __e31.appendChild(__t34);
        const __e35 = document.createElement("code");
        __e35.setAttribute("class", "agents-empty-code");
        const __t36 = document.createTextNode(".ship/agents/");
        __e35.appendChild(__t36);
        __e31.appendChild(__e35);
        const __t37 = document.createTextNode("\n          or create one from the Studio.\n          ");
        __e31.appendChild(__t37);
        const __e38 = document.createElement("ship-button");
        __e38.setAttribute("slot", "action");
        __e38.setAttribute("tone", "primary");
        __e38.setAttribute("variant", "solid");
        __c30.push(on(__e38, "click", (function() {
          const __h = $scope.newAgent;
          return typeof __h === "function" ? __h.bind(void 0) : (() => {
          });
        })(), { modifiers: [] }));
        const __e39 = document.createElement("ship-icon");
        __e39.setAttribute("slot", "leading");
        __e39.setAttribute("name", "plus");
        __e38.appendChild(__e39);
        const __t40 = document.createTextNode("\n            New agent\n          ");
        __e38.appendChild(__t40);
        __e31.appendChild(__e38);
        __frag29.appendChild(__e31);
        return { nodes: Array.from(__frag29.childNodes), cleanup: () => {
          for (const c of __c30) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag41 = document.createDocumentFragment();
        const __c42 = [];
        const __e43 = document.createElement("div");
        __e43.setAttribute("class", "agents-grid");
        __e43.setAttribute("data-view-enter", "");
        __e43.setAttribute("data-test", "grid");
        const __forAnchor44 = document.createComment("for");
        __e43.appendChild(__forAnchor44);
        __c42.push(__wireBoundary_b9(__forAnchor44));
        __frag41.appendChild(__e43);
        return { nodes: Array.from(__frag41.childNodes), cleanup: () => {
          for (const c of __c42) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b6 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.agents().kind === "ok") return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag26 = document.createDocumentFragment();
        const __c27 = [];
        const __ifAnchor28 = document.createComment("if");
        __frag26.appendChild(__ifAnchor28);
        __c27.push(__wireBoundary_b7(__ifAnchor28));
        return { nodes: Array.from(__frag26.childNodes), cleanup: () => {
          for (const c of __c27) try {
            c();
          } catch (e) {
          }
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
      const __t7 = document.createTextNode("");
      __e6.appendChild(__t7);
      __cleanups.push(bindText(__t7, () => $scope.total()));
      const __t8 = document.createTextNode(" agent");
      __e6.appendChild(__t8);
      const __t9 = document.createTextNode("");
      __e6.appendChild(__t9);
      __cleanups.push(bindText(__t9, () => $scope.total() === 1 ? "" : "s"));
      __e3.appendChild(__e6);
      __e2.appendChild(__e3);
      const __e10 = document.createElement("ship-button");
      __e10.setAttribute("tone", "primary");
      __e10.setAttribute("variant", "solid");
      __cleanups.push(on(__e10, "click", (function() {
        const __h = $scope.newAgent;
        return typeof __h === "function" ? __h.bind(void 0) : (() => {
        });
      })(), { modifiers: [] }));
      const __e11 = document.createElement("ship-icon");
      __e11.setAttribute("slot", "leading");
      __e11.setAttribute("name", "plus");
      __e10.appendChild(__e11);
      const __t12 = document.createTextNode("\n        New agent\n      ");
      __e10.appendChild(__t12);
      __e2.appendChild(__e10);
      __e1.appendChild(__e2);
      const __wlAnchor17 = document.createComment("while-loading");
      __e1.appendChild(__wlAnchor17);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor17, signal: $scope.agents, render: __renderWhileLoading_0 }));
      const __weAnchor24 = document.createComment("while-error");
      __e1.appendChild(__weAnchor24);
      __cleanups.push(bindWhileError({ anchor: __weAnchor24, signal: $scope.agents, render: __renderWhileError_0 }));
      const __ifAnchor25 = document.createComment("if");
      __e1.appendChild(__ifAnchor25);
      __cleanups.push(__wireBoundary_b6(__ifAnchor25));
      __e0.appendChild(__e1);
      container.appendChild(__e0);
      return () => {
        for (const c of __cleanups) try {
          c();
        } catch (e) {
        }
      };
    },
    hydrate(root, ctx2) {
      const __cleanups = [];
      const __t = collectTextOpens(root);
      const __m = collectElementMarkers(root);
      if (__t["b0"]) __cleanups.push(hydrateText(__t["b0"], () => $scope.total()));
      if (__t["b1"]) __cleanups.push(hydrateText(__t["b1"], () => $scope.total() === 1 ? "" : "s"));
      if (__m["b2"]) __cleanups.push(hydrateOn(__m["b2"], "click", (function() {
        const __h = $scope.newAgent;
        return typeof __h === "function" ? __h.bind(void 0) : (() => {
        });
      })(), []));
      if (__m["b3"]) __cleanups.push(hydrateWhileLoading(__m["b3"], "b3", () => $scope.agents(), __renderWhileLoading_0));
      if (__m["b4"]) __cleanups.push(hydrateWhileError(__m["b4"], "b4", () => $scope.agents(), __renderWhileError_0));
      if (__m["b6"]) {
        const __t2 = __m["b6"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode) __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b6(__anchor));
      }
      return () => {
        for (const c of __cleanups) try {
          c();
        } catch (e) {
        }
      };
    }
  });
});
export {
  view_default as default
};
