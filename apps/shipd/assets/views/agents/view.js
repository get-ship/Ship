// views/agents/view.js
import { bindAttr, bindFor, bindIf, bindText, bindWhileError, bindWhileLoading, data, live, on, view } from "@ship/ui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateOn, hydrateText, hydrateWhileError, hydrateWhileLoading } from "@ship/ui-dom";

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
var view_default2 = view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = /* @__PURE__ */ Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.agents = live(({ ship: ship2, route: route2 }) => ship2.agents.summaries());
  if (view_default && typeof view_default.setup === "function") {
    const __r = view_default.setup(ctx);
    if (__r && typeof __r === "object") Object.assign($scope, __r);
  }
  const __renderWhileLoading_0 = () => {
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
  };
  const __renderWhileError_0 = (err) => {
    const __childScope34 = Object.create($scope2);
    __childScope34.err = err;
    const __frag32 = document.createDocumentFragment();
    const __c33 = [];
    const $scope2 = __childScope34;
    const __t35 = document.createTextNode("\n      ");
    __frag32.appendChild(__t35);
    const __e36 = document.createElement("p");
    __e36.setAttribute("class", "loading-copy");
    __e36.setAttribute("data-test", "error");
    const __t37 = document.createTextNode("Error loading agents: ");
    __e36.appendChild(__t37);
    const __t38 = document.createTextNode("");
    __e36.appendChild(__t38);
    __c33.push(bindText(__t38, () => $scope2.err.message));
    __frag32.appendChild(__e36);
    const __t39 = document.createTextNode("\n    ");
    __frag32.appendChild(__t39);
    return { nodes: Array.from(__frag32.childNodes), cleanup: () => {
      for (const c of __c33) try {
        c();
      } catch (e) {
      }
    } };
  };
  const __wireBoundary_b20 = (anchor) => bindFor({
    anchor,
    items: () => $scope.a.providers.slice(0, 4),
    render: (p, __i) => {
      const __childScope125 = Object.create($scope2);
      __childScope125.p = p;
      __childScope125.__i = __i;
      const __frag123 = document.createDocumentFragment();
      const __c124 = [];
      const $scope2 = __childScope125;
      const __t126 = document.createTextNode("\n                      ");
      __frag123.appendChild(__t126);
      const __e127 = document.createElement("ship-provider-blob");
      __c124.push(bindAttr(__e127, "provider", () => $scope2.p));
      __e127.setAttribute("size", "sm");
      __e127.setAttribute("variant", "glyph");
      __c124.push(bindAttr(__e127, "title", () => $scope2.p));
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
  });
  const __wireBoundary_b23 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.a.providers.length > 4) return 0;
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
        __c132.push(bindText(__t136, () => $scope.a.providers.length - 4));
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
  });
  const __wireBoundary_b19 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.a.providers && $scope.a.providers.length > 0) return 0;
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
        __c118.push(__wireBoundary_b20(__forAnchor122));
        const __t129 = document.createTextNode("\n                    ");
        __e120.appendChild(__t129);
        const __ifAnchor130 = document.createComment("if");
        __e120.appendChild(__ifAnchor130);
        __c118.push(__wireBoundary_b23(__ifAnchor130));
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
  });
  const __wireBoundary_b26 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.a.active) return 0;
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
        const __e152 = document.createElement("sui-status-dot");
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
  });
  const __wireBoundary_b27 = (anchor) => bindFor({
    anchor,
    items: () => ($scope.a.tags || []).slice(0, 2),
    render: (t, __i) => {
      const __childScope159 = Object.create($scope2);
      __childScope159.t = t;
      __childScope159.__i = __i;
      const __frag157 = document.createDocumentFragment();
      const __c158 = [];
      const $scope2 = __childScope159;
      const __t160 = document.createTextNode("\n                    ");
      __frag157.appendChild(__t160);
      const __e161 = document.createElement("ship-chip");
      __e161.setAttribute("tone", "muted");
      __e161.setAttribute("variant", "soft");
      __e161.setAttribute("size", "sm");
      const __t162 = document.createTextNode("");
      __e161.appendChild(__t162);
      __c158.push(bindText(__t162, () => $scope2.t));
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
  });
  const __wireBoundary_b29 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (($scope.a.tags || []).length > 2) return 0;
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
        __c167.push(bindText(__t171, () => ($scope.a.tags || []).length - 2));
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
  });
  const __wireBoundary_b9 = (anchor) => bindFor({
    anchor,
    items: () => $scope.items(),
    key: (a, __i) => {
      const __child = Object.create($scope2);
      __child.a = a;
      __child.__i = __i;
      const $scope2 = __child;
      return a.id;
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
      __c71.push(bindAttr(__e74, "navigable", () => "/agents/" + $scope2.a.id));
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
      __c71.push(bindText(__t79, () => $scope2.a.letter));
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
      __c71.push(bindText(__t87, () => $scope2.a.name));
      __e84.appendChild(__e86);
      const __t88 = document.createTextNode("\n                  ");
      __e84.appendChild(__t88);
      const __e89 = document.createElement("p");
      __e89.setAttribute("class", "agents-card-desc");
      const __t90 = document.createTextNode("");
      __e89.appendChild(__t90);
      __c71.push(bindText(__t90, () => $scope2.a.description || "No description"));
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
      __c71.push(bindText(__t96, () => $scope2.a.skills_count));
      const __t97 = document.createTextNode(" skill");
      __e95.appendChild(__t97);
      const __t98 = document.createTextNode("");
      __e95.appendChild(__t98);
      __c71.push(bindText(__t98, () => $scope2.a.skills_count === 1 ? "" : "s"));
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
      __c71.push(bindText(__t104, () => $scope2.a.mcp_count));
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
      __c71.push(bindText(__t111, () => $scope2.a.rules_count));
      const __t112 = document.createTextNode(" rule");
      __e110.appendChild(__t112);
      const __t113 = document.createTextNode("");
      __e110.appendChild(__t113);
      __c71.push(bindText(__t113, () => $scope2.a.rules_count === 1 ? "" : "s"));
      __e93.appendChild(__e110);
      const __t114 = document.createTextNode("\n                ");
      __e93.appendChild(__t114);
      __e82.appendChild(__e93);
      const __t115 = document.createTextNode("\n                ");
      __e82.appendChild(__t115);
      const __ifAnchor116 = document.createComment("if");
      __e82.appendChild(__ifAnchor116);
      __c71.push(__wireBoundary_b19(__ifAnchor116));
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
      __c71.push(bindText(__t144, () => $scope2.a.source));
      __e141.appendChild(__e143);
      const __t145 = document.createTextNode("\n                  ");
      __e141.appendChild(__t145);
      const __ifAnchor146 = document.createComment("if");
      __e141.appendChild(__ifAnchor146);
      __c71.push(__wireBoundary_b26(__ifAnchor146));
      const __t155 = document.createTextNode("\n                  ");
      __e141.appendChild(__t155);
      const __forAnchor156 = document.createComment("for");
      __e141.appendChild(__forAnchor156);
      __c71.push(__wireBoundary_b27(__forAnchor156));
      const __t164 = document.createTextNode("\n                  ");
      __e141.appendChild(__t164);
      const __ifAnchor165 = document.createComment("if");
      __e141.appendChild(__ifAnchor165);
      __c71.push(__wireBoundary_b29(__ifAnchor165));
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
  });
  const __wireBoundary_b7 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.items().length === 0) return 0;
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
          const __h = $scope.newAgent;
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
        __c65.push(__wireBoundary_b9(__forAnchor69));
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
  });
  const __wireBoundary_b6 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.agents().kind === "ok") return 0;
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
        __c44.push(__wireBoundary_b7(__ifAnchor46));
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
  });
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
      __cleanups.push(bindText(__t13, () => $scope.total()));
      const __t14 = document.createTextNode(" agent");
      __e12.appendChild(__t14);
      const __t15 = document.createTextNode("");
      __e12.appendChild(__t15);
      __cleanups.push(bindText(__t15, () => $scope.total() === 1 ? "" : "s"));
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
        const __h = $scope.newAgent;
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
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor30, signal: $scope.agents, render: __renderWhileLoading_0 }));
      const __t31 = document.createTextNode("\n\n    ");
      __e3.appendChild(__t31);
      const __weAnchor40 = document.createComment("while-error");
      __e3.appendChild(__weAnchor40);
      __cleanups.push(bindWhileError({ anchor: __weAnchor40, signal: $scope.agents, render: __renderWhileError_0 }));
      const __t41 = document.createTextNode("\n\n    ");
      __e3.appendChild(__t41);
      const __ifAnchor42 = document.createComment("if");
      __e3.appendChild(__ifAnchor42);
      __cleanups.push(__wireBoundary_b6(__ifAnchor42));
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
  view_default2 as default
};
