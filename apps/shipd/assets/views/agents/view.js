// views/agents/view.js
import { bindAttr, bindFor, bindIf, bindText, bindWhileError, bindWhileLoading, data, live, on, view } from "@ship/ui-runtime";

// views/agents/view.ts
import { controller, unwrap } from "@ship/ui-runtime";
var view_default = controller({
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    total: ({ agents }) => unwrap(agents)?.items.length ?? 0
  },
  on: {
    newAgent: ({ navigate }) => navigate("/agents/create")
  }
});

// views/agents/view.js
var __scopeEval = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E = [];
__E[0] = __scopeEval("total()");
__E[1] = __scopeEval("total() === 1 ? '' : 's'");
__E[2] = __scopeEval("newAgent");
__E[3] = __scopeEval("err.message");
__E[4] = __scopeEval("agents().kind === 'ok'");
__E[5] = __scopeEval("items().length === 0");
__E[6] = __scopeEval("items()");
__E[7] = __scopeEval("a.id");
__E[8] = __scopeEval("'/agents/' + a.id");
__E[9] = __scopeEval("a.letter");
__E[10] = __scopeEval("a.name");
__E[11] = __scopeEval("a.description || 'No description'");
__E[12] = __scopeEval("a.skills_count");
__E[13] = __scopeEval("a.skills_count === 1 ? '' : 's'");
__E[14] = __scopeEval("a.mcp_count");
__E[15] = __scopeEval("a.rules_count");
__E[16] = __scopeEval("a.rules_count === 1 ? '' : 's'");
__E[17] = __scopeEval("a.providers && a.providers.length > 0");
__E[18] = __scopeEval("a.providers.slice(0, 4)");
__E[19] = __scopeEval("p");
__E[20] = __scopeEval("a.providers.length > 4");
__E[21] = __scopeEval("a.providers.length - 4");
__E[22] = __scopeEval("a.source");
__E[23] = __scopeEval("a.active");
__E[24] = __scopeEval("(a.tags || []).slice(0, 2)");
__E[25] = __scopeEval("t");
__E[26] = __scopeEval("(a.tags || []).length > 2");
__E[27] = __scopeEval("(a.tags || []).length - 2");
var view_default2 = view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = /* @__PURE__ */ Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.agents = live(({ ship: ship2, route: route2 }) => ship2.agents.summaries());
  if (typeof view_default === "function") {
    const __r = view_default(ctx);
    if (__r && typeof __r === "object") Object.assign($scope, __r);
  } else if (view_default && typeof view_default.setup === "function") {
    const __r = view_default.setup(ctx);
    if (__r && typeof __r === "object") Object.assign($scope, __r);
  }
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __t0 = document.createTextNode("\n\n");
      container.appendChild(__t0);
      const __e1 = document.createElement("div");
      __e1.setAttribute("class", "ship-view agents-view");
      const __t2 = document.createTextNode("\n  ");
      __e1.appendChild(__t2);
      const __e3 = document.createElement("div");
      __e3.setAttribute("class", "agents-wrap");
      const __t4 = document.createTextNode("\n\n    ");
      __e3.appendChild(__t4);
      const __e5 = document.createElement("header");
      __e5.setAttribute("class", "agents-head");
      const __t6 = document.createTextNode("\n      ");
      __e5.appendChild(__t6);
      const __e7 = document.createElement("div");
      __e7.setAttribute("class", "agents-head-titles");
      const __t8 = document.createTextNode("\n        ");
      __e7.appendChild(__t8);
      const __e9 = document.createElement("h1");
      __e9.setAttribute("class", "agents-title");
      const __t10 = document.createTextNode("Agents");
      __e9.appendChild(__t10);
      __e7.appendChild(__e9);
      const __t11 = document.createTextNode("\n        ");
      __e7.appendChild(__t11);
      const __e12 = document.createElement("p");
      __e12.setAttribute("class", "agents-sub");
      const __t13 = document.createTextNode("");
      __e12.appendChild(__t13);
      __cleanups.push(bindText(__t13, () => __E[0]($scope)));
      const __t14 = document.createTextNode(" agent");
      __e12.appendChild(__t14);
      const __t15 = document.createTextNode("");
      __e12.appendChild(__t15);
      __cleanups.push(bindText(__t15, () => __E[1]($scope)));
      __e7.appendChild(__e12);
      const __t16 = document.createTextNode("\n      ");
      __e7.appendChild(__t16);
      __e5.appendChild(__e7);
      const __t17 = document.createTextNode("\n      ");
      __e5.appendChild(__t17);
      const __e18 = document.createElement("ship-button");
      __e18.setAttribute("tone", "primary");
      __e18.setAttribute("variant", "solid");
      __cleanups.push(on(__e18, "click", (function() {
        const __h = __E[2]($scope);
        return typeof __h === "function" ? __h.bind(void 0) : (() => {
        });
      })(), { modifiers: [] }));
      const __t19 = document.createTextNode("\n        ");
      __e18.appendChild(__t19);
      const __e20 = document.createElement("ship-icon");
      __e20.setAttribute("slot", "leading");
      __e20.setAttribute("name", "plus");
      __e18.appendChild(__e20);
      const __t21 = document.createTextNode("\n        New agent\n      ");
      __e18.appendChild(__t21);
      __e5.appendChild(__e18);
      const __t22 = document.createTextNode("\n    ");
      __e5.appendChild(__t22);
      __e3.appendChild(__e5);
      const __t23 = document.createTextNode("\n\n    ");
      __e3.appendChild(__t23);
      const __wlAnchor30 = document.createComment("while-loading");
      __e3.appendChild(__wlAnchor30);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor30, signal: $scope.agents, render: () => {
        const __frag31 = document.createDocumentFragment();
        const __c32 = [];
        const __t33 = document.createTextNode("\n      ");
        __frag31.appendChild(__t33);
        const __e34 = document.createElement("p");
        __e34.setAttribute("class", "loading-copy");
        __e34.setAttribute("data-test", "loading");
        const __t35 = document.createTextNode("Loading agents&hellip;");
        __e34.appendChild(__t35);
        __frag31.appendChild(__e34);
        const __t36 = document.createTextNode("\n    ");
        __frag31.appendChild(__t36);
        return { nodes: Array.from(__frag31.childNodes), cleanup: () => {
          for (const c of __c32) try {
            c();
          } catch (e) {
          }
        } };
      } }));
      const __t37 = document.createTextNode("\n\n    ");
      __e3.appendChild(__t37);
      const __weAnchor46 = document.createComment("while-error");
      __e3.appendChild(__weAnchor46);
      __cleanups.push(bindWhileError({ anchor: __weAnchor46, signal: $scope.agents, render: (err) => {
        const __childScope49 = Object.create($scope2);
        __childScope49.err = err;
        const __frag47 = document.createDocumentFragment();
        const __c48 = [];
        const $scope2 = __childScope49;
        const __t50 = document.createTextNode("\n      ");
        __frag47.appendChild(__t50);
        const __e51 = document.createElement("p");
        __e51.setAttribute("class", "loading-copy");
        __e51.setAttribute("data-test", "error");
        const __t52 = document.createTextNode("Error loading agents: ");
        __e51.appendChild(__t52);
        const __t53 = document.createTextNode("");
        __e51.appendChild(__t53);
        __c48.push(bindText(__t53, () => __E[3]($scope2)));
        __frag47.appendChild(__e51);
        const __t54 = document.createTextNode("\n    ");
        __frag47.appendChild(__t54);
        return { nodes: Array.from(__frag47.childNodes), cleanup: () => {
          for (const c of __c48) try {
            c();
          } catch (e) {
          }
        } };
      } }));
      const __t55 = document.createTextNode("\n\n    ");
      __e3.appendChild(__t55);
      const __ifAnchor56 = document.createComment("if");
      __e3.appendChild(__ifAnchor56);
      __cleanups.push(bindIf({
        anchor: __ifAnchor56,
        test: () => {
          if (__E[4]($scope)) return 0;
          return -1;
        },
        branches: [
          () => {
            const __frag57 = document.createDocumentFragment();
            const __c58 = [];
            const __t59 = document.createTextNode("\n      ");
            __frag57.appendChild(__t59);
            const __ifAnchor60 = document.createComment("if");
            __frag57.appendChild(__ifAnchor60);
            __c58.push(bindIf({
              anchor: __ifAnchor60,
              test: () => {
                if (__E[5]($scope)) return 0;
                return 1;
              },
              branches: [
                () => {
                  const __frag61 = document.createDocumentFragment();
                  const __c62 = [];
                  const __t63 = document.createTextNode("\n        ");
                  __frag61.appendChild(__t63);
                  const __e64 = document.createElement("ship-empty-state");
                  __e64.setAttribute("icon", "sparkles");
                  __e64.setAttribute("variant", "default");
                  __e64.setAttribute("size", "lg");
                  __e64.setAttribute("data-test", "empty");
                  const __t65 = document.createTextNode("\n          ");
                  __e64.appendChild(__t65);
                  const __e66 = document.createElement("span");
                  __e66.setAttribute("slot", "title");
                  const __t67 = document.createTextNode("No agents yet");
                  __e66.appendChild(__t67);
                  __e64.appendChild(__e66);
                  const __t68 = document.createTextNode("\n          Drop a JSONC file under\n          ");
                  __e64.appendChild(__t68);
                  const __e69 = document.createElement("code");
                  __e69.setAttribute("class", "agents-empty-code");
                  const __t70 = document.createTextNode(".ship/agents/");
                  __e69.appendChild(__t70);
                  __e64.appendChild(__e69);
                  const __t71 = document.createTextNode("\n          or create one from the Studio.\n          ");
                  __e64.appendChild(__t71);
                  const __e72 = document.createElement("ship-button");
                  __e72.setAttribute("slot", "action");
                  __e72.setAttribute("tone", "primary");
                  __e72.setAttribute("variant", "solid");
                  __c62.push(on(__e72, "click", (function() {
                    const __h = __E[2]($scope);
                    return typeof __h === "function" ? __h.bind(void 0) : (() => {
                    });
                  })(), { modifiers: [] }));
                  const __t73 = document.createTextNode("\n            ");
                  __e72.appendChild(__t73);
                  const __e74 = document.createElement("ship-icon");
                  __e74.setAttribute("slot", "leading");
                  __e74.setAttribute("name", "plus");
                  __e72.appendChild(__e74);
                  const __t75 = document.createTextNode("\n            New agent\n          ");
                  __e72.appendChild(__t75);
                  __e64.appendChild(__e72);
                  const __t76 = document.createTextNode("\n        ");
                  __e64.appendChild(__t76);
                  __frag61.appendChild(__e64);
                  const __t77 = document.createTextNode("\n      ");
                  __frag61.appendChild(__t77);
                  return { nodes: Array.from(__frag61.childNodes), cleanup: () => {
                    for (const c of __c62) try {
                      c();
                    } catch (e) {
                    }
                  } };
                },
                () => {
                  const __frag78 = document.createDocumentFragment();
                  const __c79 = [];
                  const __t80 = document.createTextNode("\n        ");
                  __frag78.appendChild(__t80);
                  const __e81 = document.createElement("div");
                  __e81.setAttribute("class", "agents-grid");
                  __e81.setAttribute("data-view-enter", "");
                  __e81.setAttribute("data-test", "grid");
                  const __t82 = document.createTextNode("\n          ");
                  __e81.appendChild(__t82);
                  const __forAnchor83 = document.createComment("for");
                  __e81.appendChild(__forAnchor83);
                  __c79.push(bindFor({
                    anchor: __forAnchor83,
                    source: () => __E[6]($scope),
                    key: (a, __i) => {
                      const __child = Object.create($scope);
                      __child.a = a;
                      __child.__i = __i;
                      return __E[7](__child);
                    },
                    render: (a, __i) => {
                      const __childScope86 = Object.create($scope2);
                      __childScope86.a = a;
                      __childScope86.__i = __i;
                      const __frag84 = document.createDocumentFragment();
                      const __c85 = [];
                      const $scope2 = __childScope86;
                      const __t87 = document.createTextNode("\n            ");
                      __frag84.appendChild(__t87);
                      const __e88 = document.createElement("ship-list-tile");
                      __e88.setAttribute("variant", "card");
                      __e88.setAttribute("density", "normal");
                      __e88.setAttribute("interactive", "");
                      __c85.push(bindAttr(__e88, "navigable", () => __E[8]($scope2)));
                      __e88.setAttribute("class", "agents-card-tile");
                      __e88.setAttribute("data-test", "agent");
                      const __t89 = document.createTextNode("\n              ");
                      __e88.appendChild(__t89);
                      const __e90 = document.createElement("div");
                      __e90.setAttribute("class", "agents-card-icon-tile");
                      __e90.setAttribute("slot", "leading");
                      const __t91 = document.createTextNode("\n                ");
                      __e90.appendChild(__t91);
                      const __e92 = document.createElement("span");
                      __e92.setAttribute("class", "agents-card-letter");
                      const __t93 = document.createTextNode("");
                      __e92.appendChild(__t93);
                      __c85.push(bindText(__t93, () => __E[9]($scope2)));
                      __e90.appendChild(__e92);
                      const __t94 = document.createTextNode("\n              ");
                      __e90.appendChild(__t94);
                      __e88.appendChild(__e90);
                      const __t95 = document.createTextNode("\n              ");
                      __e88.appendChild(__t95);
                      const __e96 = document.createElement("div");
                      __e96.setAttribute("class", "agents-card-body");
                      const __t97 = document.createTextNode("\n                ");
                      __e96.appendChild(__t97);
                      const __e98 = document.createElement("div");
                      __e98.setAttribute("class", "agents-card-head");
                      const __t99 = document.createTextNode("\n                  ");
                      __e98.appendChild(__t99);
                      const __e100 = document.createElement("div");
                      __e100.setAttribute("class", "agents-card-title");
                      const __t101 = document.createTextNode("");
                      __e100.appendChild(__t101);
                      __c85.push(bindText(__t101, () => __E[10]($scope2)));
                      __e98.appendChild(__e100);
                      const __t102 = document.createTextNode("\n                  ");
                      __e98.appendChild(__t102);
                      const __e103 = document.createElement("p");
                      __e103.setAttribute("class", "agents-card-desc");
                      const __t104 = document.createTextNode("");
                      __e103.appendChild(__t104);
                      __c85.push(bindText(__t104, () => __E[11]($scope2)));
                      __e98.appendChild(__e103);
                      const __t105 = document.createTextNode("\n                ");
                      __e98.appendChild(__t105);
                      __e96.appendChild(__e98);
                      const __t106 = document.createTextNode("\n                ");
                      __e96.appendChild(__t106);
                      const __e107 = document.createElement("div");
                      __e107.setAttribute("class", "agents-card-stats");
                      const __t108 = document.createTextNode("\n                  ");
                      __e107.appendChild(__t108);
                      const __e109 = document.createElement("span");
                      const __t110 = document.createTextNode("");
                      __e109.appendChild(__t110);
                      __c85.push(bindText(__t110, () => __E[12]($scope2)));
                      const __t111 = document.createTextNode(" skill");
                      __e109.appendChild(__t111);
                      const __t112 = document.createTextNode("");
                      __e109.appendChild(__t112);
                      __c85.push(bindText(__t112, () => __E[13]($scope2)));
                      __e107.appendChild(__e109);
                      const __t113 = document.createTextNode("\n                  ");
                      __e107.appendChild(__t113);
                      const __e114 = document.createElement("span");
                      __e114.setAttribute("class", "agents-card-sep");
                      const __t115 = document.createTextNode("&middot;");
                      __e114.appendChild(__t115);
                      __e107.appendChild(__e114);
                      const __t116 = document.createTextNode("\n                  ");
                      __e107.appendChild(__t116);
                      const __e117 = document.createElement("span");
                      const __t118 = document.createTextNode("");
                      __e117.appendChild(__t118);
                      __c85.push(bindText(__t118, () => __E[14]($scope2)));
                      const __t119 = document.createTextNode(" MCP");
                      __e117.appendChild(__t119);
                      __e107.appendChild(__e117);
                      const __t120 = document.createTextNode("\n                  ");
                      __e107.appendChild(__t120);
                      const __e121 = document.createElement("span");
                      __e121.setAttribute("class", "agents-card-sep");
                      const __t122 = document.createTextNode("&middot;");
                      __e121.appendChild(__t122);
                      __e107.appendChild(__e121);
                      const __t123 = document.createTextNode("\n                  ");
                      __e107.appendChild(__t123);
                      const __e124 = document.createElement("span");
                      const __t125 = document.createTextNode("");
                      __e124.appendChild(__t125);
                      __c85.push(bindText(__t125, () => __E[15]($scope2)));
                      const __t126 = document.createTextNode(" rule");
                      __e124.appendChild(__t126);
                      const __t127 = document.createTextNode("");
                      __e124.appendChild(__t127);
                      __c85.push(bindText(__t127, () => __E[16]($scope2)));
                      __e107.appendChild(__e124);
                      const __t128 = document.createTextNode("\n                ");
                      __e107.appendChild(__t128);
                      __e96.appendChild(__e107);
                      const __t129 = document.createTextNode("\n                ");
                      __e96.appendChild(__t129);
                      const __ifAnchor130 = document.createComment("if");
                      __e96.appendChild(__ifAnchor130);
                      __c85.push(bindIf({
                        anchor: __ifAnchor130,
                        test: () => {
                          if (__E[17]($scope2)) return 0;
                          return -1;
                        },
                        branches: [
                          () => {
                            const __frag131 = document.createDocumentFragment();
                            const __c132 = [];
                            const __t133 = document.createTextNode("\n                  ");
                            __frag131.appendChild(__t133);
                            const __e134 = document.createElement("div");
                            __e134.setAttribute("class", "agents-card-providers");
                            __e134.setAttribute("aria-label", "Compiles to");
                            const __t135 = document.createTextNode("\n                    ");
                            __e134.appendChild(__t135);
                            const __forAnchor136 = document.createComment("for");
                            __e134.appendChild(__forAnchor136);
                            __c132.push(bindFor({
                              anchor: __forAnchor136,
                              source: () => __E[18]($scope2),
                              render: (p, __i2) => {
                                const __childScope139 = Object.create($scope3);
                                __childScope139.p = p;
                                __childScope139.__i = __i2;
                                const __frag137 = document.createDocumentFragment();
                                const __c138 = [];
                                const $scope3 = __childScope139;
                                const __t140 = document.createTextNode("\n                      ");
                                __frag137.appendChild(__t140);
                                const __e141 = document.createElement("ship-provider-blob");
                                __c138.push(bindAttr(__e141, "provider", () => __E[19]($scope3)));
                                __e141.setAttribute("size", "sm");
                                __e141.setAttribute("variant", "glyph");
                                __c138.push(bindAttr(__e141, "title", () => __E[19]($scope3)));
                                __frag137.appendChild(__e141);
                                const __t142 = document.createTextNode("\n                    ");
                                __frag137.appendChild(__t142);
                                return { nodes: Array.from(__frag137.childNodes), cleanup: () => {
                                  for (const c of __c138) try {
                                    c();
                                  } catch (e) {
                                  }
                                } };
                              }
                            }));
                            const __t143 = document.createTextNode("\n                    ");
                            __e134.appendChild(__t143);
                            const __ifAnchor144 = document.createComment("if");
                            __e134.appendChild(__ifAnchor144);
                            __c132.push(bindIf({
                              anchor: __ifAnchor144,
                              test: () => {
                                if (__E[20]($scope2)) return 0;
                                return -1;
                              },
                              branches: [
                                () => {
                                  const __frag145 = document.createDocumentFragment();
                                  const __c146 = [];
                                  const __t147 = document.createTextNode("\n                      ");
                                  __frag145.appendChild(__t147);
                                  const __e148 = document.createElement("ship-chip");
                                  __e148.setAttribute("tone", "muted");
                                  __e148.setAttribute("variant", "outline");
                                  __e148.setAttribute("size", "sm");
                                  const __t149 = document.createTextNode("+");
                                  __e148.appendChild(__t149);
                                  const __t150 = document.createTextNode("");
                                  __e148.appendChild(__t150);
                                  __c146.push(bindText(__t150, () => __E[21]($scope2)));
                                  __frag145.appendChild(__e148);
                                  const __t151 = document.createTextNode("\n                    ");
                                  __frag145.appendChild(__t151);
                                  return { nodes: Array.from(__frag145.childNodes), cleanup: () => {
                                    for (const c of __c146) try {
                                      c();
                                    } catch (e) {
                                    }
                                  } };
                                }
                              ]
                            }));
                            const __t152 = document.createTextNode("\n                  ");
                            __e134.appendChild(__t152);
                            __frag131.appendChild(__e134);
                            const __t153 = document.createTextNode("\n                ");
                            __frag131.appendChild(__t153);
                            return { nodes: Array.from(__frag131.childNodes), cleanup: () => {
                              for (const c of __c132) try {
                                c();
                              } catch (e) {
                              }
                            } };
                          }
                        ]
                      }));
                      const __t154 = document.createTextNode("\n                ");
                      __e96.appendChild(__t154);
                      const __e155 = document.createElement("div");
                      __e155.setAttribute("class", "agents-card-badges");
                      const __t156 = document.createTextNode("\n                  ");
                      __e155.appendChild(__t156);
                      const __e157 = document.createElement("ship-badge");
                      __e157.setAttribute("tone", "muted");
                      __e157.setAttribute("variant", "outline");
                      __e157.setAttribute("size", "sm");
                      __e157.setAttribute("class", "agents-card-badge-source");
                      const __t158 = document.createTextNode("");
                      __e157.appendChild(__t158);
                      __c85.push(bindText(__t158, () => __E[22]($scope2)));
                      __e155.appendChild(__e157);
                      const __t159 = document.createTextNode("\n                  ");
                      __e155.appendChild(__t159);
                      const __ifAnchor160 = document.createComment("if");
                      __e155.appendChild(__ifAnchor160);
                      __c85.push(bindIf({
                        anchor: __ifAnchor160,
                        test: () => {
                          if (__E[23]($scope2)) return 0;
                          return -1;
                        },
                        branches: [
                          () => {
                            const __frag161 = document.createDocumentFragment();
                            const __c162 = [];
                            const __t163 = document.createTextNode("\n                    ");
                            __frag161.appendChild(__t163);
                            const __e164 = document.createElement("ship-chip");
                            __e164.setAttribute("tone", "primary");
                            __e164.setAttribute("variant", "soft");
                            __e164.setAttribute("size", "sm");
                            const __t165 = document.createTextNode("\n                      ");
                            __e164.appendChild(__t165);
                            const __e166 = document.createElement("set-status-dot");
                            __e166.setAttribute("slot", "leading");
                            __e166.setAttribute("tone", "primary");
                            __e166.setAttribute("size", "sm");
                            __e166.setAttribute("pulse", "");
                            __e166.setAttribute("label", "Active session");
                            __e164.appendChild(__e166);
                            const __t167 = document.createTextNode("\n                      active session\n                    ");
                            __e164.appendChild(__t167);
                            __frag161.appendChild(__e164);
                            const __t168 = document.createTextNode("\n                  ");
                            __frag161.appendChild(__t168);
                            return { nodes: Array.from(__frag161.childNodes), cleanup: () => {
                              for (const c of __c162) try {
                                c();
                              } catch (e) {
                              }
                            } };
                          }
                        ]
                      }));
                      const __t169 = document.createTextNode("\n                  ");
                      __e155.appendChild(__t169);
                      const __forAnchor170 = document.createComment("for");
                      __e155.appendChild(__forAnchor170);
                      __c85.push(bindFor({
                        anchor: __forAnchor170,
                        source: () => __E[24]($scope2),
                        render: (t, __i2) => {
                          const __childScope173 = Object.create($scope3);
                          __childScope173.t = t;
                          __childScope173.__i = __i2;
                          const __frag171 = document.createDocumentFragment();
                          const __c172 = [];
                          const $scope3 = __childScope173;
                          const __t174 = document.createTextNode("\n                    ");
                          __frag171.appendChild(__t174);
                          const __e175 = document.createElement("ship-chip");
                          __e175.setAttribute("tone", "muted");
                          __e175.setAttribute("variant", "soft");
                          __e175.setAttribute("size", "sm");
                          const __t176 = document.createTextNode("");
                          __e175.appendChild(__t176);
                          __c172.push(bindText(__t176, () => __E[25]($scope3)));
                          __frag171.appendChild(__e175);
                          const __t177 = document.createTextNode("\n                  ");
                          __frag171.appendChild(__t177);
                          return { nodes: Array.from(__frag171.childNodes), cleanup: () => {
                            for (const c of __c172) try {
                              c();
                            } catch (e) {
                            }
                          } };
                        }
                      }));
                      const __t178 = document.createTextNode("\n                  ");
                      __e155.appendChild(__t178);
                      const __ifAnchor179 = document.createComment("if");
                      __e155.appendChild(__ifAnchor179);
                      __c85.push(bindIf({
                        anchor: __ifAnchor179,
                        test: () => {
                          if (__E[26]($scope2)) return 0;
                          return -1;
                        },
                        branches: [
                          () => {
                            const __frag180 = document.createDocumentFragment();
                            const __c181 = [];
                            const __t182 = document.createTextNode("\n                    ");
                            __frag180.appendChild(__t182);
                            const __e183 = document.createElement("ship-chip");
                            __e183.setAttribute("tone", "muted");
                            __e183.setAttribute("variant", "outline");
                            __e183.setAttribute("size", "sm");
                            const __t184 = document.createTextNode("+");
                            __e183.appendChild(__t184);
                            const __t185 = document.createTextNode("");
                            __e183.appendChild(__t185);
                            __c181.push(bindText(__t185, () => __E[27]($scope2)));
                            __frag180.appendChild(__e183);
                            const __t186 = document.createTextNode("\n                  ");
                            __frag180.appendChild(__t186);
                            return { nodes: Array.from(__frag180.childNodes), cleanup: () => {
                              for (const c of __c181) try {
                                c();
                              } catch (e) {
                              }
                            } };
                          }
                        ]
                      }));
                      const __t187 = document.createTextNode("\n                ");
                      __e155.appendChild(__t187);
                      __e96.appendChild(__e155);
                      const __t188 = document.createTextNode("\n              ");
                      __e96.appendChild(__t188);
                      __e88.appendChild(__e96);
                      const __t189 = document.createTextNode("\n            ");
                      __e88.appendChild(__t189);
                      __frag84.appendChild(__e88);
                      const __t190 = document.createTextNode("\n          ");
                      __frag84.appendChild(__t190);
                      return { nodes: Array.from(__frag84.childNodes), cleanup: () => {
                        for (const c of __c85) try {
                          c();
                        } catch (e) {
                        }
                      } };
                    }
                  }));
                  const __t191 = document.createTextNode("\n        ");
                  __e81.appendChild(__t191);
                  __frag78.appendChild(__e81);
                  const __t192 = document.createTextNode("\n      ");
                  __frag78.appendChild(__t192);
                  return { nodes: Array.from(__frag78.childNodes), cleanup: () => {
                    for (const c of __c79) try {
                      c();
                    } catch (e) {
                    }
                  } };
                }
              ]
            }));
            const __t193 = document.createTextNode("\n    ");
            __frag57.appendChild(__t193);
            return { nodes: Array.from(__frag57.childNodes), cleanup: () => {
              for (const c of __c58) try {
                c();
              } catch (e) {
              }
            } };
          }
        ]
      }));
      const __t194 = document.createTextNode("\n\n  ");
      __e3.appendChild(__t194);
      __e1.appendChild(__e3);
      const __t195 = document.createTextNode("\n");
      __e1.appendChild(__t195);
      container.appendChild(__e1);
      const __t196 = document.createTextNode("\n");
      container.appendChild(__t196);
      return () => {
        for (const c of __cleanups) try {
          c();
        } catch (e) {
        }
      };
    },
    "__renderWhileLoading_0": () => {
      const __frag24 = document.createDocumentFragment();
      const __c25 = [];
      const __t26 = document.createTextNode("\n      ");
      __frag24.appendChild(__t26);
      const __e27 = document.createElement("p");
      __e27.setAttribute("class", "loading-copy");
      __e27.setAttribute("data-test", "loading");
      const __t28 = document.createTextNode("Loading agents&hellip;");
      __e27.appendChild(__t28);
      __frag24.appendChild(__e27);
      const __t29 = document.createTextNode("\n    ");
      __frag24.appendChild(__t29);
      return { nodes: Array.from(__frag24.childNodes), cleanup: () => {
        for (const c of __c25) try {
          c();
        } catch (e) {
        }
      } };
    },
    "__renderWhileError_0": (err) => {
      const __childScope40 = Object.create($scope2);
      __childScope40.err = err;
      const __frag38 = document.createDocumentFragment();
      const __c39 = [];
      const $scope2 = __childScope40;
      const __t41 = document.createTextNode("\n      ");
      __frag38.appendChild(__t41);
      const __e42 = document.createElement("p");
      __e42.setAttribute("class", "loading-copy");
      __e42.setAttribute("data-test", "error");
      const __t43 = document.createTextNode("Error loading agents: ");
      __e42.appendChild(__t43);
      const __t44 = document.createTextNode("");
      __e42.appendChild(__t44);
      __c39.push(bindText(__t44, () => __E[3]($scope2)));
      __frag38.appendChild(__e42);
      const __t45 = document.createTextNode("\n    ");
      __frag38.appendChild(__t45);
      return { nodes: Array.from(__frag38.childNodes), cleanup: () => {
        for (const c of __c39) try {
          c();
        } catch (e) {
        }
      } };
    }
  });
});
var bindings = [
  { key: "b0", kind: "text", expr: "total()", deps: ["total"] },
  { key: "b1", kind: "text", expr: "total() === 1 ? '' : 's'", deps: ["total"] },
  { key: "b2", kind: "on:click", expr: "newAgent", deps: ["newAgent"] },
  { key: "b3", kind: "while-loading", expr: "agents", deps: ["agents"], renderKey: "__renderWhileLoading_0" },
  { key: "b4", kind: "while-error", expr: "agents", deps: ["agents"], renderKey: "__renderWhileError_0" },
  { key: "b6", kind: "if", expr: "agents().kind === 'ok'", deps: ["agents"] },
  { key: "b7", kind: "if", expr: "items().length === 0", deps: ["items"] },
  { key: "b8", kind: "on:click", expr: "newAgent", deps: ["newAgent"] },
  { key: "b9", kind: "for", expr: "items()", deps: ["items"] },
  { key: "b10", kind: "attr:navigable", expr: "'/agents/' + a.id", deps: ["a"] },
  { key: "b11", kind: "text", expr: "a.letter", deps: ["a"] },
  { key: "b12", kind: "text", expr: "a.name", deps: ["a"] },
  { key: "b13", kind: "text", expr: "a.description || 'No description'", deps: ["a"] },
  { key: "b14", kind: "text", expr: "a.skills_count", deps: ["a"] },
  { key: "b15", kind: "text", expr: "a.skills_count === 1 ? '' : 's'", deps: ["a"] },
  { key: "b16", kind: "text", expr: "a.mcp_count", deps: ["a"] },
  { key: "b17", kind: "text", expr: "a.rules_count", deps: ["a"] },
  { key: "b18", kind: "text", expr: "a.rules_count === 1 ? '' : 's'", deps: ["a"] },
  { key: "b19", kind: "if", expr: "a.providers && a.providers.length > 0", deps: ["a"] },
  { key: "b20", kind: "for", expr: "a.providers.slice(0, 4)", deps: ["a"] },
  { key: "b21", kind: "attr:provider", expr: "p", deps: ["p"] },
  { key: "b22", kind: "attr:title", expr: "p", deps: ["p"] },
  { key: "b23", kind: "if", expr: "a.providers.length > 4", deps: ["a"] },
  { key: "b24", kind: "text", expr: "a.providers.length - 4", deps: ["a"] },
  { key: "b25", kind: "text", expr: "a.source", deps: ["a"] },
  { key: "b26", kind: "if", expr: "a.active", deps: ["a"] },
  { key: "b27", kind: "for", expr: "(a.tags || []).slice(0, 2)", deps: ["a"] },
  { key: "b28", kind: "text", expr: "t", deps: ["t"] },
  { key: "b29", kind: "if", expr: "(a.tags || []).length > 2", deps: ["a"] },
  { key: "b30", kind: "text", expr: "(a.tags || []).length - 2", deps: ["a"] }
];
export {
  bindings,
  view_default2 as default
};
