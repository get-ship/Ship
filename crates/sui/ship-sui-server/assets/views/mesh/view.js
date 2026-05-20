// views/mesh/page.sui
import { bindAttr, bindFor, bindIf, bindProp, bindText, bindWhileError, bindWhileLoading, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateOn, hydrateProp, hydrateText, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/mesh/controller.ts
import { controller, derived, live, signal, unwrap } from "@ship/sui-runtime";
import { liveExpr } from "@ship/sdk-client";
function formatTs(raw) {
  if (!raw)
    return "—";
  return raw.slice(0, 19).replace("T", " ");
}
var controller_default = controller({
  scope: () => {
    const snap = live(() => liveExpr("ship.mesh.snapshot()"));
    const actorsFormatted = derived(() => {
      const v = unwrap(snap);
      const list = v?.actors ?? [];
      return list.map((a) => ({ ...a, last_event_at_pretty: formatTs(a.last_event_at) }));
    });
    const attachSlug = signal("");
    const attachOpen = signal(false);
    return { snap, actorsFormatted, attachSlug, attachOpen };
  },
  on: {
    openAttach: (state, slug) => {
      const ds = state;
      ds.attachSlug = slug;
      ds.attachOpen = true;
    },
    closeAttach: (state) => {
      const ds = state;
      ds.attachOpen = false;
      ds.attachSlug = "";
    }
  }
});

// views/mesh/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var page_default = Object.assign(view((ctx) => {
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
  const __wireBoundary_b1 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope8 = Object.create($scope);
      __childScope8.n = n;
      __childScope8.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b2 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => [0, 1, 2],
          key: (r, __i2) => {
            const __child = Object.create($scope2);
            __child.r = r;
            __child.__i = __i2;
            return (($scope3) => r)(__child);
          },
          render: (r, __i2) => {
            const __childScope14 = Object.create($scope2);
            __childScope14.r = r;
            __childScope14.__i = __i2;
            return (($scope3) => {
              const __frag15 = document.createDocumentFragment();
              const __c16 = [];
              const __e17 = document.createElement("div");
              __e17.setAttribute("class", "mesh-skel-row");
              const __e18 = document.createElement("sui-skeleton");
              __e18.setAttribute("shape", "circle");
              __e18.setAttribute("size", "sm");
              __e17.appendChild(__e18);
              const __e19 = document.createElement("sui-skeleton");
              __e19.setAttribute("shape", "line");
              __e19.setAttribute("size", "md");
              __e17.appendChild(__e19);
              __frag15.appendChild(__e17);
              return { nodes: Array.from(__frag15.childNodes), cleanup: () => {
                for (const c of __c16)
                  try {
                    c();
                  } catch (e) {}
              } };
            })(__childScope14);
          }
        });
        const __frag9 = document.createDocumentFragment();
        const __c10 = [];
        const __e11 = document.createElement("section");
        __e11.setAttribute("class", "mesh-skel-section");
        const __e12 = document.createElement("sui-skeleton");
        __e12.setAttribute("shape", "line");
        __e12.setAttribute("size", "md");
        __e12.setAttribute("style", "width:140px");
        __e11.appendChild(__e12);
        const __forAnchor13 = document.createComment("for");
        __e11.appendChild(__forAnchor13);
        __c10.push(__wireBoundary_b2(__forAnchor13));
        __frag9.appendChild(__e11);
        return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
          for (const c of __c10)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope8);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag1 = document.createDocumentFragment();
    const __c2 = [];
    const __e3 = document.createElement("sui-stack");
    __e3.setAttribute("gap", "5");
    __e3.setAttribute("aria-busy", "true");
    __e3.setAttribute("aria-label", "Loading mesh");
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "mesh-skel-head");
    const __e5 = document.createElement("sui-skeleton");
    __e5.setAttribute("shape", "line");
    __e5.setAttribute("size", "lg");
    __e5.setAttribute("style", "width:120px");
    __e4.appendChild(__e5);
    const __e6 = document.createElement("sui-skeleton");
    __e6.setAttribute("shape", "line");
    __e6.setAttribute("size", "sm");
    __e6.setAttribute("style", "width:320px");
    __e4.appendChild(__e6);
    __e3.appendChild(__e4);
    const __forAnchor7 = document.createComment("for");
    __e3.appendChild(__forAnchor7);
    __c2.push(__wireBoundary_b1(__forAnchor7));
    __frag1.appendChild(__e3);
    return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
      for (const c of __c2)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope21 = Object.create($scope);
    __childScope21.err = err;
    return (($scope2) => {
      const __frag22 = document.createDocumentFragment();
      const __c23 = [];
      const __e24 = document.createElement("sui-error-state");
      __e24.setAttribute("variant", "default");
      __e24.setAttribute("size", "md");
      __e24.setAttribute("title", "Couldn't load mesh");
      const __t25 = document.createTextNode("");
      __e24.appendChild(__t25);
      __c23.push(bindText(__t25, () => $scope2.err.message));
      __frag22.appendChild(__e24);
      return { nodes: Array.from(__frag22.childNodes), cleanup: () => {
        for (const c of __c23)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope21);
  };
  const __wireBoundary_b11 = (anchor) => bindFor({
    anchor,
    items: () => $scope.actorsFormatted(),
    key: (a, __i) => {
      const __child = Object.create($scope);
      __child.a = a;
      __child.__i = __i;
      return (($scope2) => a.actor_id)(__child);
    },
    render: (a, __i) => {
      const __childScope82 = Object.create($scope);
      __childScope82.a = a;
      __childScope82.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b13 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.agent_id)
              return 0;
            return 1;
          },
          branches: [
            () => {
              const __frag93 = document.createDocumentFragment();
              const __c94 = [];
              const __t95 = document.createTextNode("");
              __frag93.appendChild(__t95);
              __t95.textContent = $scope2.a.agent_id;
              const __e96 = document.createElement("div");
              __e96.setAttribute("class", "mesh-id mesh-muted");
              const __t97 = document.createTextNode("");
              __e96.appendChild(__t97);
              __t97.textContent = $scope2.a.actor_id;
              __frag93.appendChild(__e96);
              return { nodes: Array.from(__frag93.childNodes), cleanup: () => {
                for (const c of __c94)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag98 = document.createDocumentFragment();
              const __c99 = [];
              const __e100 = document.createElement("span");
              __e100.setAttribute("class", "mesh-id");
              const __t101 = document.createTextNode("");
              __e100.appendChild(__t101);
              __t101.textContent = $scope2.a.actor_id;
              __frag98.appendChild(__e100);
              return { nodes: Array.from(__frag98.childNodes), cleanup: () => {
                for (const c of __c99)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b22 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.transport === "pty")
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag113 = document.createDocumentFragment();
              const __c114 = [];
              const __e115 = document.createElement("sui-button");
              __e115.setAttribute("size", "sm");
              __e115.setAttribute("variant", "outline");
              __c114.push(on(__e115, "click", function() {
                const __h = () => $scope2.openAttach($scope2.a.actor_id);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              __e115.setAttribute("data-test", "attach-action");
              const __t116 = document.createTextNode(`
                      Attach
                    `);
              __e115.appendChild(__t116);
              __frag113.appendChild(__e115);
              return { nodes: Array.from(__frag113.childNodes), cleanup: () => {
                for (const c of __c114)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag83 = document.createDocumentFragment();
        const __c84 = [];
        const __e85 = document.createElement("tr");
        const __e86 = document.createElement("td");
        const __e87 = document.createElement("span");
        __e87.setAttribute("class", "dot dot-live");
        __e86.appendChild(__e87);
        __e85.appendChild(__e86);
        const __e88 = document.createElement("td");
        const __e89 = document.createElement("span");
        __e89.setAttribute("class", "mesh-badge");
        const __t90 = document.createTextNode("");
        __e89.appendChild(__t90);
        __t90.textContent = $scope2.a.kind;
        __e88.appendChild(__e89);
        __e85.appendChild(__e88);
        const __e91 = document.createElement("td");
        const __ifAnchor92 = document.createComment("if");
        __e91.appendChild(__ifAnchor92);
        __c84.push(__wireBoundary_b13(__ifAnchor92));
        __e85.appendChild(__e91);
        const __e102 = document.createElement("td");
        const __t103 = document.createTextNode("");
        __e102.appendChild(__t103);
        __t103.textContent = $scope2.a.provider || "—";
        const __t104 = document.createTextNode("");
        __e102.appendChild(__t104);
        __t104.textContent = $scope2.a.transport ? " · " + $scope2.a.transport : "";
        __e85.appendChild(__e102);
        const __e105 = document.createElement("td");
        const __t106 = document.createTextNode("");
        __e105.appendChild(__t106);
        __t106.textContent = $scope2.a.workspace_id || "—";
        __e85.appendChild(__e105);
        const __e107 = document.createElement("td");
        __e107.setAttribute("class", "mesh-pid");
        const __t108 = document.createTextNode("");
        __e107.appendChild(__t108);
        __t108.textContent = $scope2.a.pid || "—";
        __e85.appendChild(__e107);
        const __e109 = document.createElement("td");
        __e109.setAttribute("class", "mesh-ts");
        const __t110 = document.createTextNode("");
        __e109.appendChild(__t110);
        __t110.textContent = $scope2.a.last_event_at_pretty;
        __e85.appendChild(__e109);
        const __e111 = document.createElement("td");
        __e111.setAttribute("class", "mesh-actions");
        const __ifAnchor112 = document.createComment("if");
        __e111.appendChild(__ifAnchor112);
        __c84.push(__wireBoundary_b22(__ifAnchor112));
        __e85.appendChild(__e111);
        __frag83.appendChild(__e85);
        return { nodes: Array.from(__frag83.childNodes), cleanup: () => {
          for (const c of __c84)
            try {
              c();
            } catch (e) {}
        }, update: (__new117, __newI118) => {
          const __old = $scope2.a;
          $scope2.a = __new117;
          $scope2.__i = __newI118;
          if (__old.kind !== $scope2.a.kind) {
            __t90.textContent = $scope2.a.kind;
          }
          if (__old.provider !== $scope2.a.provider) {
            __t103.textContent = $scope2.a.provider || "—";
          }
          if (__old.transport !== $scope2.a.transport) {
            __t104.textContent = $scope2.a.transport ? " · " + $scope2.a.transport : "";
          }
          if (__old.workspace_id !== $scope2.a.workspace_id) {
            __t106.textContent = $scope2.a.workspace_id || "—";
          }
          if (__old.pid !== $scope2.a.pid) {
            __t108.textContent = $scope2.a.pid || "—";
          }
          if (__old.last_event_at_pretty !== $scope2.a.last_event_at_pretty) {
            __t110.textContent = $scope2.a.last_event_at_pretty;
          }
        } };
      })(__childScope82);
    }
  });
  const __wireBoundary_b10 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.snap().value.actors.length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag57 = document.createDocumentFragment();
        const __c58 = [];
        const __e59 = document.createElement("div");
        __e59.setAttribute("class", "mesh-empty");
        const __t60 = document.createTextNode("No actors on the mesh. Spawn an agent or run a pipeline.");
        __e59.appendChild(__t60);
        __frag57.appendChild(__e59);
        return { nodes: Array.from(__frag57.childNodes), cleanup: () => {
          for (const c of __c58)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag61 = document.createDocumentFragment();
        const __c62 = [];
        const __e63 = document.createElement("table");
        __e63.setAttribute("class", "mesh-table");
        const __e64 = document.createElement("thead");
        const __e65 = document.createElement("tr");
        const __e66 = document.createElement("th");
        __e65.appendChild(__e66);
        const __e67 = document.createElement("th");
        const __t68 = document.createTextNode("Kind");
        __e67.appendChild(__t68);
        __e65.appendChild(__e67);
        const __e69 = document.createElement("th");
        const __t70 = document.createTextNode("Name");
        __e69.appendChild(__t70);
        __e65.appendChild(__e69);
        const __e71 = document.createElement("th");
        const __t72 = document.createTextNode("Provider");
        __e71.appendChild(__t72);
        __e65.appendChild(__e71);
        const __e73 = document.createElement("th");
        const __t74 = document.createTextNode("Workspace");
        __e73.appendChild(__t74);
        __e65.appendChild(__e73);
        const __e75 = document.createElement("th");
        const __t76 = document.createTextNode("Pid");
        __e75.appendChild(__t76);
        __e65.appendChild(__e75);
        const __e77 = document.createElement("th");
        const __t78 = document.createTextNode("Last activity");
        __e77.appendChild(__t78);
        __e65.appendChild(__e77);
        const __e79 = document.createElement("th");
        __e65.appendChild(__e79);
        __e64.appendChild(__e65);
        __e63.appendChild(__e64);
        const __e80 = document.createElement("tbody");
        const __forAnchor81 = document.createComment("for");
        __e80.appendChild(__forAnchor81);
        __c62.push(__wireBoundary_b11(__forAnchor81));
        __e63.appendChild(__e80);
        __frag61.appendChild(__e63);
        return { nodes: Array.from(__frag61.childNodes), cleanup: () => {
          for (const c of __c62)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b26 = (anchor) => bindFor({
    anchor,
    items: () => $scope.snap().value.processes,
    key: (p, __i) => {
      const __child = Object.create($scope);
      __child.p = p;
      __child.__i = __i;
      return (($scope2) => p.pid + "_" + p.name)(__child);
    },
    render: (p, __i) => {
      const __childScope148 = Object.create($scope);
      __childScope148.p = p;
      __childScope148.__i = __i;
      return (($scope2) => {
        const __frag149 = document.createDocumentFragment();
        const __c150 = [];
        const __e151 = document.createElement("tr");
        const __e152 = document.createElement("td");
        const __e153 = document.createElement("span");
        {
          const __av154 = $scope2.p.alive ? "dot dot-live" : "dot dot-dead";
          if (__av154 == null || __av154 === false || __av154 === "")
            __e153.removeAttribute("class");
          else
            __e153.setAttribute("class", __av154 === true ? "" : String(__av154));
        }
        __e152.appendChild(__e153);
        __e151.appendChild(__e152);
        const __e155 = document.createElement("td");
        const __e156 = document.createElement("strong");
        const __t157 = document.createTextNode("");
        __e156.appendChild(__t157);
        __t157.textContent = $scope2.p.name;
        __e155.appendChild(__e156);
        __e151.appendChild(__e155);
        const __e158 = document.createElement("td");
        const __e159 = document.createElement("span");
        __e159.setAttribute("class", "mesh-badge");
        const __t160 = document.createTextNode("");
        __e159.appendChild(__t160);
        __t160.textContent = $scope2.p.role;
        __e158.appendChild(__e159);
        __e151.appendChild(__e158);
        const __e161 = document.createElement("td");
        __e161.setAttribute("class", "mesh-pid");
        const __t162 = document.createTextNode("");
        __e161.appendChild(__t162);
        __t162.textContent = $scope2.p.pid;
        __e151.appendChild(__e161);
        const __e163 = document.createElement("td");
        __e163.setAttribute("class", "mesh-muted");
        __e163.setAttribute("style", "font-size:var(--text-xs)");
        const __t164 = document.createTextNode("");
        __e163.appendChild(__t164);
        __t164.textContent = $scope2.p.version || "";
        const __t165 = document.createTextNode("");
        __e163.appendChild(__t165);
        __t165.textContent = $scope2.p.log ? " · " + $scope2.p.log : "";
        const __t166 = document.createTextNode("");
        __e163.appendChild(__t166);
        __t166.textContent = $scope2.p.cmdline_preview || "";
        __e151.appendChild(__e163);
        __frag149.appendChild(__e151);
        return { nodes: Array.from(__frag149.childNodes), cleanup: () => {
          for (const c of __c150)
            try {
              c();
            } catch (e) {}
        }, update: (__new167, __newI168) => {
          const __old = $scope2.p;
          $scope2.p = __new167;
          $scope2.__i = __newI168;
          if (__old.alive !== $scope2.p.alive) {
            {
              const __av154 = $scope2.p.alive ? "dot dot-live" : "dot dot-dead";
              if (__av154 == null || __av154 === false || __av154 === "")
                __e153.removeAttribute("class");
              else
                __e153.setAttribute("class", __av154 === true ? "" : String(__av154));
            }
          }
          if (__old.name !== $scope2.p.name) {
            __t157.textContent = $scope2.p.name;
          }
          if (__old.role !== $scope2.p.role) {
            __t160.textContent = $scope2.p.role;
          }
          if (__old.pid !== $scope2.p.pid) {
            __t162.textContent = $scope2.p.pid;
          }
          if (__old.version !== $scope2.p.version) {
            __t164.textContent = $scope2.p.version || "";
          }
          if (__old.log !== $scope2.p.log) {
            __t165.textContent = $scope2.p.log ? " · " + $scope2.p.log : "";
          }
          if (__old.cmdline_preview !== $scope2.p.cmdline_preview) {
            __t166.textContent = $scope2.p.cmdline_preview || "";
          }
        } };
      })(__childScope148);
    }
  });
  const __wireBoundary_b25 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.snap().value.processes.length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag128 = document.createDocumentFragment();
        const __c129 = [];
        const __e130 = document.createElement("div");
        __e130.setAttribute("class", "mesh-empty");
        const __t131 = document.createTextNode("No process info available.");
        __e130.appendChild(__t131);
        __frag128.appendChild(__e130);
        return { nodes: Array.from(__frag128.childNodes), cleanup: () => {
          for (const c of __c129)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag132 = document.createDocumentFragment();
        const __c133 = [];
        const __e134 = document.createElement("table");
        __e134.setAttribute("class", "mesh-table");
        const __e135 = document.createElement("thead");
        const __e136 = document.createElement("tr");
        const __e137 = document.createElement("th");
        __e136.appendChild(__e137);
        const __e138 = document.createElement("th");
        const __t139 = document.createTextNode("Name");
        __e138.appendChild(__t139);
        __e136.appendChild(__e138);
        const __e140 = document.createElement("th");
        const __t141 = document.createTextNode("Role");
        __e140.appendChild(__t141);
        __e136.appendChild(__e140);
        const __e142 = document.createElement("th");
        const __t143 = document.createTextNode("Pid");
        __e142.appendChild(__t143);
        __e136.appendChild(__e142);
        const __e144 = document.createElement("th");
        const __t145 = document.createTextNode("Detail");
        __e144.appendChild(__t145);
        __e136.appendChild(__e144);
        __e135.appendChild(__e136);
        __e134.appendChild(__e135);
        const __e146 = document.createElement("tbody");
        const __forAnchor147 = document.createComment("for");
        __e146.appendChild(__forAnchor147);
        __c133.push(__wireBoundary_b26(__forAnchor147));
        __e134.appendChild(__e146);
        __frag132.appendChild(__e134);
        return { nodes: Array.from(__frag132.childNodes), cleanup: () => {
          for (const c of __c133)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b35 = (anchor) => bindFor({
    anchor,
    items: () => $scope.snap().value.integrations,
    key: (i, __i) => {
      const __child = Object.create($scope);
      __child.i = i;
      __child.__i = __i;
      return (($scope2) => i.name)(__child);
    },
    render: (i, __i) => {
      const __childScope189 = Object.create($scope);
      __childScope189.i = i;
      __childScope189.__i = __i;
      return (($scope2) => {
        const __frag190 = document.createDocumentFragment();
        const __c191 = [];
        const __e192 = document.createElement("tr");
        const __e193 = document.createElement("td");
        const __e194 = document.createElement("span");
        {
          const __av195 = $scope2.i.active ? "dot dot-live" : "dot dot-dead";
          if (__av195 == null || __av195 === false || __av195 === "")
            __e194.removeAttribute("class");
          else
            __e194.setAttribute("class", __av195 === true ? "" : String(__av195));
        }
        __e193.appendChild(__e194);
        __e192.appendChild(__e193);
        const __e196 = document.createElement("td");
        const __e197 = document.createElement("strong");
        const __t198 = document.createTextNode("");
        __e197.appendChild(__t198);
        __t198.textContent = $scope2.i.name;
        __e196.appendChild(__e197);
        __e192.appendChild(__e196);
        const __e199 = document.createElement("td");
        const __e200 = document.createElement("span");
        __e200.setAttribute("class", "mesh-badge");
        const __t201 = document.createTextNode("");
        __e200.appendChild(__t201);
        __t201.textContent = $scope2.i.kind;
        __e199.appendChild(__e200);
        __e192.appendChild(__e199);
        const __e202 = document.createElement("td");
        __e202.setAttribute("class", "mesh-muted");
        const __t203 = document.createTextNode("");
        __e202.appendChild(__t203);
        __t203.textContent = $scope2.i.hint;
        __e192.appendChild(__e202);
        __frag190.appendChild(__e192);
        return { nodes: Array.from(__frag190.childNodes), cleanup: () => {
          for (const c of __c191)
            try {
              c();
            } catch (e) {}
        }, update: (__new204, __newI205) => {
          const __old = $scope2.i;
          $scope2.i = __new204;
          $scope2.__i = __newI205;
          if (__old.active !== $scope2.i.active) {
            {
              const __av195 = $scope2.i.active ? "dot dot-live" : "dot dot-dead";
              if (__av195 == null || __av195 === false || __av195 === "")
                __e194.removeAttribute("class");
              else
                __e194.setAttribute("class", __av195 === true ? "" : String(__av195));
            }
          }
          if (__old.name !== $scope2.i.name) {
            __t198.textContent = $scope2.i.name;
          }
          if (__old.kind !== $scope2.i.kind) {
            __t201.textContent = $scope2.i.kind;
          }
          if (__old.hint !== $scope2.i.hint) {
            __t203.textContent = $scope2.i.hint;
          }
        } };
      })(__childScope189);
    }
  });
  const __wireBoundary_b5 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.snap().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag28 = document.createDocumentFragment();
        const __c29 = [];
        const __e30 = document.createElement("header");
        __e30.setAttribute("class", "mesh-head");
        const __e31 = document.createElement("h1");
        const __t32 = document.createTextNode("Mesh");
        __e31.appendChild(__t32);
        __e30.appendChild(__e31);
        const __e33 = document.createElement("div");
        __e33.setAttribute("class", "mesh-sub");
        const __t34 = document.createTextNode("what Ship is running on this machine, right now");
        __e33.appendChild(__t34);
        __e30.appendChild(__e33);
        const __e35 = document.createElement("div");
        __e35.setAttribute("class", "mesh-counts");
        const __e36 = document.createElement("span");
        const __e37 = document.createElement("strong");
        const __t38 = document.createTextNode("");
        __e37.appendChild(__t38);
        __c29.push(bindText(__t38, () => $scope.snap().value.counts.actors));
        __e36.appendChild(__e37);
        const __t39 = document.createTextNode(" actors");
        __e36.appendChild(__t39);
        __e35.appendChild(__e36);
        const __e40 = document.createElement("span");
        const __e41 = document.createElement("strong");
        const __t42 = document.createTextNode("");
        __e41.appendChild(__t42);
        __c29.push(bindText(__t42, () => $scope.snap().value.counts.processes));
        __e40.appendChild(__e41);
        const __t43 = document.createTextNode(" processes");
        __e40.appendChild(__t43);
        __e35.appendChild(__e40);
        const __e44 = document.createElement("span");
        const __e45 = document.createElement("strong");
        const __t46 = document.createTextNode("");
        __e45.appendChild(__t46);
        __c29.push(bindText(__t46, () => $scope.snap().value.counts.integrations));
        __e44.appendChild(__e45);
        const __t47 = document.createTextNode(" integrations");
        __e44.appendChild(__t47);
        __e35.appendChild(__e44);
        __e30.appendChild(__e35);
        __frag28.appendChild(__e30);
        const __e48 = document.createElement("section");
        __e48.setAttribute("class", "mesh-section");
        const __e49 = document.createElement("header");
        __e49.setAttribute("class", "mesh-section-head");
        const __e50 = document.createElement("h2");
        const __t51 = document.createTextNode("Actors");
        __e50.appendChild(__t51);
        __e49.appendChild(__e50);
        const __e52 = document.createElement("span");
        __e52.setAttribute("class", "n");
        const __t53 = document.createTextNode("");
        __e52.appendChild(__t53);
        __c29.push(bindText(__t53, () => $scope.snap().value.actors.length));
        __e49.appendChild(__e52);
        const __e54 = document.createElement("span");
        __e54.setAttribute("class", "desc");
        const __t55 = document.createTextNode("live mailboxes — anything reachable via target_actor_id");
        __e54.appendChild(__t55);
        __e49.appendChild(__e54);
        __e48.appendChild(__e49);
        const __ifAnchor56 = document.createComment("if");
        __e48.appendChild(__ifAnchor56);
        __c29.push(__wireBoundary_b10(__ifAnchor56));
        __frag28.appendChild(__e48);
        const __e119 = document.createElement("section");
        __e119.setAttribute("class", "mesh-section");
        const __e120 = document.createElement("header");
        __e120.setAttribute("class", "mesh-section-head");
        const __e121 = document.createElement("h2");
        const __t122 = document.createTextNode("Ship processes");
        __e121.appendChild(__t122);
        __e120.appendChild(__e121);
        const __e123 = document.createElement("span");
        __e123.setAttribute("class", "n");
        const __t124 = document.createTextNode("");
        __e123.appendChild(__t124);
        __c29.push(bindText(__t124, () => $scope.snap().value.processes.length));
        __e120.appendChild(__e123);
        const __e125 = document.createElement("span");
        __e125.setAttribute("class", "desc");
        const __t126 = document.createTextNode("OS-level processes Ship spawned on your machine");
        __e125.appendChild(__t126);
        __e120.appendChild(__e125);
        __e119.appendChild(__e120);
        const __ifAnchor127 = document.createComment("if");
        __e119.appendChild(__ifAnchor127);
        __c29.push(__wireBoundary_b25(__ifAnchor127));
        __frag28.appendChild(__e119);
        const __e169 = document.createElement("section");
        __e169.setAttribute("class", "mesh-section");
        const __e170 = document.createElement("header");
        __e170.setAttribute("class", "mesh-section-head");
        const __e171 = document.createElement("h2");
        const __t172 = document.createTextNode("Integrations");
        __e171.appendChild(__t172);
        __e170.appendChild(__e171);
        const __e173 = document.createElement("span");
        __e173.setAttribute("class", "n");
        const __t174 = document.createTextNode("");
        __e173.appendChild(__t174);
        __c29.push(bindText(__t174, () => $scope.snap().value.integrations.length));
        __e170.appendChild(__e173);
        const __e175 = document.createElement("span");
        __e175.setAttribute("class", "desc");
        const __t176 = document.createTextNode("external bridges + channel capabilities");
        __e175.appendChild(__t176);
        __e170.appendChild(__e175);
        __e169.appendChild(__e170);
        const __e177 = document.createElement("table");
        __e177.setAttribute("class", "mesh-table");
        const __e178 = document.createElement("thead");
        const __e179 = document.createElement("tr");
        const __e180 = document.createElement("th");
        __e179.appendChild(__e180);
        const __e181 = document.createElement("th");
        const __t182 = document.createTextNode("Name");
        __e181.appendChild(__t182);
        __e179.appendChild(__e181);
        const __e183 = document.createElement("th");
        const __t184 = document.createTextNode("Kind");
        __e183.appendChild(__t184);
        __e179.appendChild(__e183);
        const __e185 = document.createElement("th");
        const __t186 = document.createTextNode("Hint");
        __e185.appendChild(__t186);
        __e179.appendChild(__e185);
        __e178.appendChild(__e179);
        __e177.appendChild(__e178);
        const __e187 = document.createElement("tbody");
        const __forAnchor188 = document.createComment("for");
        __e187.appendChild(__forAnchor188);
        __c29.push(__wireBoundary_b35(__forAnchor188));
        __e177.appendChild(__e187);
        __e169.appendChild(__e177);
        __frag28.appendChild(__e169);
        return { nodes: Array.from(__frag28.childNodes), cleanup: () => {
          for (const c of __c29)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b44 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.attachOpen() && $scope.attachSlug())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag213 = document.createDocumentFragment();
        const __c214 = [];
        const __e215 = document.createElement("sui-agent-attach");
        __c214.push(bindAttr(__e215, "slug", () => $scope.attachSlug()));
        __e215.setAttribute("style", "flex: 1; min-height: 0");
        __frag213.appendChild(__e215);
        return { nodes: Array.from(__frag213.childNodes), cleanup: () => {
          for (const c of __c214)
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
      __e0.setAttribute("class", "ship-view ship-mesh");
      const __wlAnchor20 = document.createComment("while-loading");
      __e0.appendChild(__wlAnchor20);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor20, signal: $scope.snap, render: __renderWhileLoading_0 }));
      const __weAnchor26 = document.createComment("while-error");
      __e0.appendChild(__weAnchor26);
      __cleanups.push(bindWhileError({ anchor: __weAnchor26, signal: $scope.snap, render: __renderWhileError_0 }));
      const __ifAnchor27 = document.createComment("if");
      __e0.appendChild(__ifAnchor27);
      __cleanups.push(__wireBoundary_b5(__ifAnchor27));
      const __e206 = document.createElement("sui-drawer");
      __e206.setAttribute("size", "lg");
      __cleanups.push(bindProp(__e206, "open", () => $scope.attachOpen()));
      __cleanups.push(on(__e206, "sui-close", function() {
        const __h = $scope.closeAttach;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __cleanups.push(on(__e206, "sui-dismiss", function() {
        const __h = $scope.closeAttach;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e207 = document.createElement("span");
      __e207.setAttribute("slot", "title");
      const __t208 = document.createTextNode("Agent session");
      __e207.appendChild(__t208);
      __e206.appendChild(__e207);
      const __e209 = document.createElement("span");
      __e209.setAttribute("slot", "description");
      const __t210 = document.createTextNode("");
      __e209.appendChild(__t210);
      __cleanups.push(bindText(__t210, () => $scope.attachSlug()));
      __e206.appendChild(__e209);
      const __e211 = document.createElement("div");
      __e211.setAttribute("class", "mesh-attach-body");
      __e211.setAttribute("style", "height: 60vh; min-height: 320px; display: flex;");
      const __ifAnchor212 = document.createComment("if");
      __e211.appendChild(__ifAnchor212);
      __cleanups.push(__wireBoundary_b44(__ifAnchor212));
      __e206.appendChild(__e211);
      __e0.appendChild(__e206);
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
        __cleanups.push(hydrateWhileLoading(__m["b0"], "b0", () => $scope.snap(), __renderWhileLoading_0));
      if (__m["b3"])
        __cleanups.push(hydrateWhileError(__m["b3"], "b3", () => $scope.snap(), __renderWhileError_0));
      if (__m["b5"]) {
        const __t2 = __m["b5"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b5(__anchor));
      }
      if (__m["b40"])
        __cleanups.push(hydrateProp(__m["b40"], "open", () => $scope.attachOpen()));
      if (__m["b41"])
        __cleanups.push(hydrateOn(__m["b41"], "sui-close", function() {
          const __h = $scope.closeAttach;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b42"])
        __cleanups.push(hydrateOn(__m["b42"], "sui-dismiss", function() {
          const __h = $scope.closeAttach;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__t["b43"])
        __cleanups.push(hydrateText(__t["b43"], () => $scope.attachSlug()));
      if (__m["b44"]) {
        const __t2 = __m["b44"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b44(__anchor));
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
