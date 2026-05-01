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
      const __wlAnchor24 = document.createComment("while-loading");
      __e3.appendChild(__wlAnchor24);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor24, signal: $scope.agents, render: () => {
        const __frag25 = document.createDocumentFragment();
        const __c26 = [];
        const __t27 = document.createTextNode("\n      ");
        __frag25.appendChild(__t27);
        const __e28 = document.createElement("p");
        __e28.setAttribute("class", "loading-copy");
        __e28.setAttribute("data-test", "loading");
        const __t29 = document.createTextNode("Loading agents&hellip;");
        __e28.appendChild(__t29);
        __frag25.appendChild(__e28);
        const __t30 = document.createTextNode("\n    ");
        __frag25.appendChild(__t30);
        return { nodes: Array.from(__frag25.childNodes), cleanup: () => {
          for (const c of __c26) try {
            c();
          } catch (e) {
          }
        } };
      } }));
      const __t31 = document.createTextNode("\n\n    ");
      __e3.appendChild(__t31);
      const __weAnchor32 = document.createComment("while-error");
      __e3.appendChild(__weAnchor32);
      __cleanups.push(bindWhileError({ anchor: __weAnchor32, signal: $scope.agents, render: (err) => {
        const __childScope35 = Object.create($scope2);
        __childScope35.err = err;
        const __frag33 = document.createDocumentFragment();
        const __c34 = [];
        const $scope2 = __childScope35;
        const __t36 = document.createTextNode("\n      ");
        __frag33.appendChild(__t36);
        const __e37 = document.createElement("p");
        __e37.setAttribute("class", "loading-copy");
        __e37.setAttribute("data-test", "error");
        const __t38 = document.createTextNode("Error loading agents: ");
        __e37.appendChild(__t38);
        const __t39 = document.createTextNode("");
        __e37.appendChild(__t39);
        __c34.push(bindText(__t39, () => __E[3]($scope2)));
        __frag33.appendChild(__e37);
        const __t40 = document.createTextNode("\n    ");
        __frag33.appendChild(__t40);
        return { nodes: Array.from(__frag33.childNodes), cleanup: () => {
          for (const c of __c34) try {
            c();
          } catch (e) {
          }
        } };
      } }));
      const __t41 = document.createTextNode("\n\n    ");
      __e3.appendChild(__t41);
      const __ifAnchor42 = document.createComment("if");
      __e3.appendChild(__ifAnchor42);
      __cleanups.push(bindIf({
        anchor: __ifAnchor42,
        test: () => {
          if (__E[4]($scope)) return 0;
          return -1;
        },
        branches: [
          () => {
            const __frag43 = document.createDocumentFragment();
            const __c44 = [];
            const __t45 = document.createTextNode("\n      ");
            __frag43.appendChild(__t45);
            const __ifAnchor46 = document.createComment("if");
            __frag43.appendChild(__ifAnchor46);
            __c44.push(bindIf({
              anchor: __ifAnchor46,
              test: () => {
                if (__E[5]($scope)) return 0;
                return 1;
              },
              branches: [
                () => {
                  const __frag47 = document.createDocumentFragment();
                  const __c48 = [];
                  const __t49 = document.createTextNode("\n        ");
                  __frag47.appendChild(__t49);
                  const __e50 = document.createElement("ship-empty-state");
                  __e50.setAttribute("icon", "sparkles");
                  __e50.setAttribute("variant", "default");
                  __e50.setAttribute("size", "lg");
                  __e50.setAttribute("data-test", "empty");
                  const __t51 = document.createTextNode("\n          ");
                  __e50.appendChild(__t51);
                  const __e52 = document.createElement("span");
                  __e52.setAttribute("slot", "title");
                  const __t53 = document.createTextNode("No agents yet");
                  __e52.appendChild(__t53);
                  __e50.appendChild(__e52);
                  const __t54 = document.createTextNode("\n          Drop a JSONC file under\n          ");
                  __e50.appendChild(__t54);
                  const __e55 = document.createElement("code");
                  __e55.setAttribute("class", "agents-empty-code");
                  const __t56 = document.createTextNode(".ship/agents/");
                  __e55.appendChild(__t56);
                  __e50.appendChild(__e55);
                  const __t57 = document.createTextNode("\n          or create one from the Studio.\n          ");
                  __e50.appendChild(__t57);
                  const __e58 = document.createElement("ship-button");
                  __e58.setAttribute("slot", "action");
                  __e58.setAttribute("tone", "primary");
                  __e58.setAttribute("variant", "solid");
                  __c48.push(on(__e58, "click", (function() {
                    const __h = __E[2]($scope);
                    return typeof __h === "function" ? __h.bind(void 0) : (() => {
                    });
                  })(), { modifiers: [] }));
                  const __t59 = document.createTextNode("\n            ");
                  __e58.appendChild(__t59);
                  const __e60 = document.createElement("ship-icon");
                  __e60.setAttribute("slot", "leading");
                  __e60.setAttribute("name", "plus");
                  __e58.appendChild(__e60);
                  const __t61 = document.createTextNode("\n            New agent\n          ");
                  __e58.appendChild(__t61);
                  __e50.appendChild(__e58);
                  const __t62 = document.createTextNode("\n        ");
                  __e50.appendChild(__t62);
                  __frag47.appendChild(__e50);
                  const __t63 = document.createTextNode("\n      ");
                  __frag47.appendChild(__t63);
                  return { nodes: Array.from(__frag47.childNodes), cleanup: () => {
                    for (const c of __c48) try {
                      c();
                    } catch (e) {
                    }
                  } };
                },
                () => {
                  const __frag64 = document.createDocumentFragment();
                  const __c65 = [];
                  const __t66 = document.createTextNode("\n        ");
                  __frag64.appendChild(__t66);
                  const __e67 = document.createElement("div");
                  __e67.setAttribute("class", "agents-grid");
                  __e67.setAttribute("data-view-enter", "");
                  __e67.setAttribute("data-test", "grid");
                  const __t68 = document.createTextNode("\n          ");
                  __e67.appendChild(__t68);
                  const __forAnchor69 = document.createComment("for");
                  __e67.appendChild(__forAnchor69);
                  __c65.push(bindFor({
                    anchor: __forAnchor69,
                    source: () => __E[6]($scope),
                    key: (a, __i) => {
                      const __child = Object.create($scope);
                      __child.a = a;
                      __child.__i = __i;
                      return __E[7](__child);
                    },
                    render: (a, __i) => {
                      const __childScope72 = Object.create($scope2);
                      __childScope72.a = a;
                      __childScope72.__i = __i;
                      const __frag70 = document.createDocumentFragment();
                      const __c71 = [];
                      const $scope2 = __childScope72;
                      const __t73 = document.createTextNode("\n            ");
                      __frag70.appendChild(__t73);
                      const __e74 = document.createElement("ship-list-tile");
                      __e74.setAttribute("variant", "card");
                      __e74.setAttribute("density", "normal");
                      __e74.setAttribute("interactive", "");
                      __c71.push(bindAttr(__e74, "navigable", () => __E[8]($scope2)));
                      __e74.setAttribute("class", "agents-card-tile");
                      __e74.setAttribute("data-test", "agent");
                      const __t75 = document.createTextNode("\n              ");
                      __e74.appendChild(__t75);
                      const __e76 = document.createElement("div");
                      __e76.setAttribute("class", "agents-card-icon-tile");
                      __e76.setAttribute("slot", "leading");
                      const __t77 = document.createTextNode("\n                ");
                      __e76.appendChild(__t77);
                      const __e78 = document.createElement("span");
                      __e78.setAttribute("class", "agents-card-letter");
                      const __t79 = document.createTextNode("");
                      __e78.appendChild(__t79);
                      __c71.push(bindText(__t79, () => __E[9]($scope2)));
                      __e76.appendChild(__e78);
                      const __t80 = document.createTextNode("\n              ");
                      __e76.appendChild(__t80);
                      __e74.appendChild(__e76);
                      const __t81 = document.createTextNode("\n              ");
                      __e74.appendChild(__t81);
                      const __e82 = document.createElement("div");
                      __e82.setAttribute("class", "agents-card-body");
                      const __t83 = document.createTextNode("\n                ");
                      __e82.appendChild(__t83);
                      const __e84 = document.createElement("div");
                      __e84.setAttribute("class", "agents-card-head");
                      const __t85 = document.createTextNode("\n                  ");
                      __e84.appendChild(__t85);
                      const __e86 = document.createElement("div");
                      __e86.setAttribute("class", "agents-card-title");
                      const __t87 = document.createTextNode("");
                      __e86.appendChild(__t87);
                      __c71.push(bindText(__t87, () => __E[10]($scope2)));
                      __e84.appendChild(__e86);
                      const __t88 = document.createTextNode("\n                  ");
                      __e84.appendChild(__t88);
                      const __e89 = document.createElement("p");
                      __e89.setAttribute("class", "agents-card-desc");
                      const __t90 = document.createTextNode("");
                      __e89.appendChild(__t90);
                      __c71.push(bindText(__t90, () => __E[11]($scope2)));
                      __e84.appendChild(__e89);
                      const __t91 = document.createTextNode("\n                ");
                      __e84.appendChild(__t91);
                      __e82.appendChild(__e84);
                      const __t92 = document.createTextNode("\n                ");
                      __e82.appendChild(__t92);
                      const __e93 = document.createElement("div");
                      __e93.setAttribute("class", "agents-card-stats");
                      const __t94 = document.createTextNode("\n                  ");
                      __e93.appendChild(__t94);
                      const __e95 = document.createElement("span");
                      const __t96 = document.createTextNode("");
                      __e95.appendChild(__t96);
                      __c71.push(bindText(__t96, () => __E[12]($scope2)));
                      const __t97 = document.createTextNode(" skill");
                      __e95.appendChild(__t97);
                      const __t98 = document.createTextNode("");
                      __e95.appendChild(__t98);
                      __c71.push(bindText(__t98, () => __E[13]($scope2)));
                      __e93.appendChild(__e95);
                      const __t99 = document.createTextNode("\n                  ");
                      __e93.appendChild(__t99);
                      const __e100 = document.createElement("span");
                      __e100.setAttribute("class", "agents-card-sep");
                      const __t101 = document.createTextNode("&middot;");
                      __e100.appendChild(__t101);
                      __e93.appendChild(__e100);
                      const __t102 = document.createTextNode("\n                  ");
                      __e93.appendChild(__t102);
                      const __e103 = document.createElement("span");
                      const __t104 = document.createTextNode("");
                      __e103.appendChild(__t104);
                      __c71.push(bindText(__t104, () => __E[14]($scope2)));
                      const __t105 = document.createTextNode(" MCP");
                      __e103.appendChild(__t105);
                      __e93.appendChild(__e103);
                      const __t106 = document.createTextNode("\n                  ");
                      __e93.appendChild(__t106);
                      const __e107 = document.createElement("span");
                      __e107.setAttribute("class", "agents-card-sep");
                      const __t108 = document.createTextNode("&middot;");
                      __e107.appendChild(__t108);
                      __e93.appendChild(__e107);
                      const __t109 = document.createTextNode("\n                  ");
                      __e93.appendChild(__t109);
                      const __e110 = document.createElement("span");
                      const __t111 = document.createTextNode("");
                      __e110.appendChild(__t111);
                      __c71.push(bindText(__t111, () => __E[15]($scope2)));
                      const __t112 = document.createTextNode(" rule");
                      __e110.appendChild(__t112);
                      const __t113 = document.createTextNode("");
                      __e110.appendChild(__t113);
                      __c71.push(bindText(__t113, () => __E[16]($scope2)));
                      __e93.appendChild(__e110);
                      const __t114 = document.createTextNode("\n                ");
                      __e93.appendChild(__t114);
                      __e82.appendChild(__e93);
                      const __t115 = document.createTextNode("\n                ");
                      __e82.appendChild(__t115);
                      const __ifAnchor116 = document.createComment("if");
                      __e82.appendChild(__ifAnchor116);
                      __c71.push(bindIf({
                        anchor: __ifAnchor116,
                        test: () => {
                          if (__E[17]($scope2)) return 0;
                          return -1;
                        },
                        branches: [
                          () => {
                            const __frag117 = document.createDocumentFragment();
                            const __c118 = [];
                            const __t119 = document.createTextNode("\n                  ");
                            __frag117.appendChild(__t119);
                            const __e120 = document.createElement("div");
                            __e120.setAttribute("class", "agents-card-providers");
                            __e120.setAttribute("aria-label", "Compiles to");
                            const __t121 = document.createTextNode("\n                    ");
                            __e120.appendChild(__t121);
                            const __forAnchor122 = document.createComment("for");
                            __e120.appendChild(__forAnchor122);
                            __c118.push(bindFor({
                              anchor: __forAnchor122,
                              source: () => __E[18]($scope2),
                              render: (p, __i2) => {
                                const __childScope125 = Object.create($scope3);
                                __childScope125.p = p;
                                __childScope125.__i = __i2;
                                const __frag123 = document.createDocumentFragment();
                                const __c124 = [];
                                const $scope3 = __childScope125;
                                const __t126 = document.createTextNode("\n                      ");
                                __frag123.appendChild(__t126);
                                const __e127 = document.createElement("ship-provider-blob");
                                __c124.push(bindAttr(__e127, "provider", () => __E[19]($scope3)));
                                __e127.setAttribute("size", "sm");
                                __e127.setAttribute("variant", "glyph");
                                __c124.push(bindAttr(__e127, "title", () => __E[19]($scope3)));
                                __frag123.appendChild(__e127);
                                const __t128 = document.createTextNode("\n                    ");
                                __frag123.appendChild(__t128);
                                return { nodes: Array.from(__frag123.childNodes), cleanup: () => {
                                  for (const c of __c124) try {
                                    c();
                                  } catch (e) {
                                  }
                                } };
                              }
                            }));
                            const __t129 = document.createTextNode("\n                    ");
                            __e120.appendChild(__t129);
                            const __ifAnchor130 = document.createComment("if");
                            __e120.appendChild(__ifAnchor130);
                            __c118.push(bindIf({
                              anchor: __ifAnchor130,
                              test: () => {
                                if (__E[20]($scope2)) return 0;
                                return -1;
                              },
                              branches: [
                                () => {
                                  const __frag131 = document.createDocumentFragment();
                                  const __c132 = [];
                                  const __t133 = document.createTextNode("\n                      ");
                                  __frag131.appendChild(__t133);
                                  const __e134 = document.createElement("ship-chip");
                                  __e134.setAttribute("tone", "muted");
                                  __e134.setAttribute("variant", "outline");
                                  __e134.setAttribute("size", "sm");
                                  const __t135 = document.createTextNode("+");
                                  __e134.appendChild(__t135);
                                  const __t136 = document.createTextNode("");
                                  __e134.appendChild(__t136);
                                  __c132.push(bindText(__t136, () => __E[21]($scope2)));
                                  __frag131.appendChild(__e134);
                                  const __t137 = document.createTextNode("\n                    ");
                                  __frag131.appendChild(__t137);
                                  return { nodes: Array.from(__frag131.childNodes), cleanup: () => {
                                    for (const c of __c132) try {
                                      c();
                                    } catch (e) {
                                    }
                                  } };
                                }
                              ]
                            }));
                            const __t138 = document.createTextNode("\n                  ");
                            __e120.appendChild(__t138);
                            __frag117.appendChild(__e120);
                            const __t139 = document.createTextNode("\n                ");
                            __frag117.appendChild(__t139);
                            return { nodes: Array.from(__frag117.childNodes), cleanup: () => {
                              for (const c of __c118) try {
                                c();
                              } catch (e) {
                              }
                            } };
                          }
                        ]
                      }));
                      const __t140 = document.createTextNode("\n                ");
                      __e82.appendChild(__t140);
                      const __e141 = document.createElement("div");
                      __e141.setAttribute("class", "agents-card-badges");
                      const __t142 = document.createTextNode("\n                  ");
                      __e141.appendChild(__t142);
                      const __e143 = document.createElement("ship-badge");
                      __e143.setAttribute("tone", "muted");
                      __e143.setAttribute("variant", "outline");
                      __e143.setAttribute("size", "sm");
                      __e143.setAttribute("class", "agents-card-badge-source");
                      const __t144 = document.createTextNode("");
                      __e143.appendChild(__t144);
                      __c71.push(bindText(__t144, () => __E[22]($scope2)));
                      __e141.appendChild(__e143);
                      const __t145 = document.createTextNode("\n                  ");
                      __e141.appendChild(__t145);
                      const __ifAnchor146 = document.createComment("if");
                      __e141.appendChild(__ifAnchor146);
                      __c71.push(bindIf({
                        anchor: __ifAnchor146,
                        test: () => {
                          if (__E[23]($scope2)) return 0;
                          return -1;
                        },
                        branches: [
                          () => {
                            const __frag147 = document.createDocumentFragment();
                            const __c148 = [];
                            const __t149 = document.createTextNode("\n                    ");
                            __frag147.appendChild(__t149);
                            const __e150 = document.createElement("ship-chip");
                            __e150.setAttribute("tone", "primary");
                            __e150.setAttribute("variant", "soft");
                            __e150.setAttribute("size", "sm");
                            const __t151 = document.createTextNode("\n                      ");
                            __e150.appendChild(__t151);
                            const __e152 = document.createElement("set-status-dot");
                            __e152.setAttribute("slot", "leading");
                            __e152.setAttribute("tone", "primary");
                            __e152.setAttribute("size", "sm");
                            __e152.setAttribute("pulse", "");
                            __e152.setAttribute("label", "Active session");
                            __e150.appendChild(__e152);
                            const __t153 = document.createTextNode("\n                      active session\n                    ");
                            __e150.appendChild(__t153);
                            __frag147.appendChild(__e150);
                            const __t154 = document.createTextNode("\n                  ");
                            __frag147.appendChild(__t154);
                            return { nodes: Array.from(__frag147.childNodes), cleanup: () => {
                              for (const c of __c148) try {
                                c();
                              } catch (e) {
                              }
                            } };
                          }
                        ]
                      }));
                      const __t155 = document.createTextNode("\n                  ");
                      __e141.appendChild(__t155);
                      const __forAnchor156 = document.createComment("for");
                      __e141.appendChild(__forAnchor156);
                      __c71.push(bindFor({
                        anchor: __forAnchor156,
                        source: () => __E[24]($scope2),
                        render: (t, __i2) => {
                          const __childScope159 = Object.create($scope3);
                          __childScope159.t = t;
                          __childScope159.__i = __i2;
                          const __frag157 = document.createDocumentFragment();
                          const __c158 = [];
                          const $scope3 = __childScope159;
                          const __t160 = document.createTextNode("\n                    ");
                          __frag157.appendChild(__t160);
                          const __e161 = document.createElement("ship-chip");
                          __e161.setAttribute("tone", "muted");
                          __e161.setAttribute("variant", "soft");
                          __e161.setAttribute("size", "sm");
                          const __t162 = document.createTextNode("");
                          __e161.appendChild(__t162);
                          __c158.push(bindText(__t162, () => __E[25]($scope3)));
                          __frag157.appendChild(__e161);
                          const __t163 = document.createTextNode("\n                  ");
                          __frag157.appendChild(__t163);
                          return { nodes: Array.from(__frag157.childNodes), cleanup: () => {
                            for (const c of __c158) try {
                              c();
                            } catch (e) {
                            }
                          } };
                        }
                      }));
                      const __t164 = document.createTextNode("\n                  ");
                      __e141.appendChild(__t164);
                      const __ifAnchor165 = document.createComment("if");
                      __e141.appendChild(__ifAnchor165);
                      __c71.push(bindIf({
                        anchor: __ifAnchor165,
                        test: () => {
                          if (__E[26]($scope2)) return 0;
                          return -1;
                        },
                        branches: [
                          () => {
                            const __frag166 = document.createDocumentFragment();
                            const __c167 = [];
                            const __t168 = document.createTextNode("\n                    ");
                            __frag166.appendChild(__t168);
                            const __e169 = document.createElement("ship-chip");
                            __e169.setAttribute("tone", "muted");
                            __e169.setAttribute("variant", "outline");
                            __e169.setAttribute("size", "sm");
                            const __t170 = document.createTextNode("+");
                            __e169.appendChild(__t170);
                            const __t171 = document.createTextNode("");
                            __e169.appendChild(__t171);
                            __c167.push(bindText(__t171, () => __E[27]($scope2)));
                            __frag166.appendChild(__e169);
                            const __t172 = document.createTextNode("\n                  ");
                            __frag166.appendChild(__t172);
                            return { nodes: Array.from(__frag166.childNodes), cleanup: () => {
                              for (const c of __c167) try {
                                c();
                              } catch (e) {
                              }
                            } };
                          }
                        ]
                      }));
                      const __t173 = document.createTextNode("\n                ");
                      __e141.appendChild(__t173);
                      __e82.appendChild(__e141);
                      const __t174 = document.createTextNode("\n              ");
                      __e82.appendChild(__t174);
                      __e74.appendChild(__e82);
                      const __t175 = document.createTextNode("\n            ");
                      __e74.appendChild(__t175);
                      __frag70.appendChild(__e74);
                      const __t176 = document.createTextNode("\n          ");
                      __frag70.appendChild(__t176);
                      return { nodes: Array.from(__frag70.childNodes), cleanup: () => {
                        for (const c of __c71) try {
                          c();
                        } catch (e) {
                        }
                      } };
                    }
                  }));
                  const __t177 = document.createTextNode("\n        ");
                  __e67.appendChild(__t177);
                  __frag64.appendChild(__e67);
                  const __t178 = document.createTextNode("\n      ");
                  __frag64.appendChild(__t178);
                  return { nodes: Array.from(__frag64.childNodes), cleanup: () => {
                    for (const c of __c65) try {
                      c();
                    } catch (e) {
                    }
                  } };
                }
              ]
            }));
            const __t179 = document.createTextNode("\n    ");
            __frag43.appendChild(__t179);
            return { nodes: Array.from(__frag43.childNodes), cleanup: () => {
              for (const c of __c44) try {
                c();
              } catch (e) {
              }
            } };
          }
        ]
      }));
      const __t180 = document.createTextNode("\n\n  ");
      __e3.appendChild(__t180);
      __e1.appendChild(__e3);
      const __t181 = document.createTextNode("\n");
      __e1.appendChild(__t181);
      container.appendChild(__e1);
      const __t182 = document.createTextNode("\n");
      container.appendChild(__t182);
      return () => {
        for (const c of __cleanups) try {
          c();
        } catch (e) {
        }
      };
    }
  });
});
var bindings = [
  { key: "b0", kind: "text", expr: "total()", deps: ["total"] },
  { key: "b1", kind: "text", expr: "total() === 1 ? '' : 's'", deps: ["total"] },
  { key: "b2", kind: "on:click", expr: "newAgent", deps: ["newAgent"] },
  { key: "b3", kind: "text", expr: "err.message", deps: ["err"] },
  { key: "b4", kind: "if", expr: "agents().kind === 'ok'", deps: ["agents"] },
  { key: "b5", kind: "if", expr: "items().length === 0", deps: ["items"] },
  { key: "b6", kind: "on:click", expr: "newAgent", deps: ["newAgent"] },
  { key: "b7", kind: "for", expr: "items()", deps: ["items"] },
  { key: "b8", kind: "attr:navigable", expr: "'/agents/' + a.id", deps: ["a"] },
  { key: "b9", kind: "text", expr: "a.letter", deps: ["a"] },
  { key: "b10", kind: "text", expr: "a.name", deps: ["a"] },
  { key: "b11", kind: "text", expr: "a.description || 'No description'", deps: ["a"] },
  { key: "b12", kind: "text", expr: "a.skills_count", deps: ["a"] },
  { key: "b13", kind: "text", expr: "a.skills_count === 1 ? '' : 's'", deps: ["a"] },
  { key: "b14", kind: "text", expr: "a.mcp_count", deps: ["a"] },
  { key: "b15", kind: "text", expr: "a.rules_count", deps: ["a"] },
  { key: "b16", kind: "text", expr: "a.rules_count === 1 ? '' : 's'", deps: ["a"] },
  { key: "b17", kind: "if", expr: "a.providers && a.providers.length > 0", deps: ["a"] },
  { key: "b18", kind: "for", expr: "a.providers.slice(0, 4)", deps: ["a"] },
  { key: "b19", kind: "attr:provider", expr: "p", deps: ["p"] },
  { key: "b20", kind: "attr:title", expr: "p", deps: ["p"] },
  { key: "b21", kind: "if", expr: "a.providers.length > 4", deps: ["a"] },
  { key: "b22", kind: "text", expr: "a.providers.length - 4", deps: ["a"] },
  { key: "b23", kind: "text", expr: "a.source", deps: ["a"] },
  { key: "b24", kind: "if", expr: "a.active", deps: ["a"] },
  { key: "b25", kind: "for", expr: "(a.tags || []).slice(0, 2)", deps: ["a"] },
  { key: "b26", kind: "text", expr: "t", deps: ["t"] },
  { key: "b27", kind: "if", expr: "(a.tags || []).length > 2", deps: ["a"] },
  { key: "b28", kind: "text", expr: "(a.tags || []).length - 2", deps: ["a"] }
];
export {
  bindings,
  view_default2 as default
};
