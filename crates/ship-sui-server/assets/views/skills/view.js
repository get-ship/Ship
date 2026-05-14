// views/skills/page.sui
import { bindAttr, bindClass, bindFor, bindIf, bindProp, bindText, bindValue, bindWhileError, bindWhileLoading, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateBindValue, hydrateOn, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/skills/controller.ts
import { controller, signal, live } from "@ship/sui-runtime";
function sseSource(expr) {
  return {
    subscribe(sig) {
      const es = new EventSource(`/_sui/data?expr=${encodeURIComponent(expr)}`);
      const onPatch = (ev) => {
        try {
          const patch = JSON.parse(ev.data);
          if (patch.op === "set")
            sig.set({ kind: "ok", value: patch.value });
        } catch (err) {
          sig.set({ kind: "error", error: err instanceof Error ? err : new Error(String(err)) });
        }
      };
      es.addEventListener("patch", onPatch);
      return () => {
        es.close();
      };
    }
  };
}
function runsSource(skillId) {
  const expr = `ship.events.list(${JSON.stringify({
    event_type: "skill.started",
    entity: skillId || "__none__",
    limit: 20
  })})`;
  return {
    subscribe(sig) {
      if (!skillId) {
        sig.set({ kind: "ok", value: { rows: [] } });
        return () => {};
      }
      const es = new EventSource(`/_sui/data?expr=${encodeURIComponent(expr)}`);
      const onPatch = (ev) => {
        try {
          const patch = JSON.parse(ev.data);
          if (patch.op === "set") {
            const groups = patch.value?.groups ?? [];
            const rows = groups.flatMap((g) => g.rows ?? []);
            sig.set({ kind: "ok", value: { rows } });
          }
        } catch (err) {
          sig.set({ kind: "error", error: err instanceof Error ? err : new Error(String(err)) });
        }
      };
      es.addEventListener("patch", onPatch);
      return () => {
        es.close();
      };
    }
  };
}
function skillsExpr(args) {
  const filtered = {};
  for (const [k, v] of Object.entries(args)) {
    if (typeof v === "string" && v.length > 0 && v !== "all")
      filtered[k] = v;
  }
  const lit = Object.keys(filtered).length === 0 ? "" : JSON.stringify(filtered);
  return `ship.skills.summaries(${lit})`;
}
var controller_default = controller({
  scope: () => {
    const q = signal("");
    const kind = signal("all");
    const tag = signal("");
    const skillId = signal("");
    const tab = signal("content");
    const viewRaw = signal(false);
    const invokeBusy = signal("");
    const invokeResult = signal(null);
    const rail = live(() => sseSource(skillsExpr({ q: q(), kind: kind(), tag: tag() })));
    const skill = live(() => sseSource(`ship.skills.detail(${JSON.stringify({ id: skillId() || "__none__" })})`));
    const runs = live(() => runsSource(skillId()));
    function openSource() {
      viewRaw.set(!viewRaw());
      if (viewRaw())
        tab.set("content");
    }
    async function invoke(id) {
      invokeBusy.set(id);
      invokeResult.set(null);
      try {
        const resp = await fetch(`/api/v1/skills/${encodeURIComponent(id)}/invoke`, {
          method: "POST",
          credentials: "same-origin",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({})
        });
        const text = await resp.text().catch(() => "");
        if (!resp.ok) {
          let msg = text || `Skill invoke failed (${resp.status})`;
          if (resp.status === 404 || resp.status === 501 || text.includes("not_implemented")) {
            msg = "Skill runs are coming soon — server route not wired yet.";
          }
          invokeResult.set({ ok: false, error: msg });
          return;
        }
        const body = text ? JSON.parse(text) : {};
        if (body.ok && typeof body.run_id === "string" && typeof body.session_id === "string") {
          invokeResult.set({
            ok: true,
            run_id: body.run_id,
            session_id: body.session_id,
            workspace_id: body.workspace_id ?? null,
            mode: body.mode ?? "library"
          });
        } else {
          invokeResult.set({ ok: false, error: "Skill invoke returned an unexpected response." });
        }
      } catch (e) {
        invokeResult.set({ ok: false, error: e.message ?? "Skill invoke failed." });
      } finally {
        invokeBusy.set("");
      }
    }
    return {
      q,
      kind,
      tag,
      skillId,
      tab,
      viewRaw,
      rail,
      skill,
      runs,
      invoke,
      invokeBusy,
      invokeResult,
      openSource
    };
  }
});

// views/skills/page.sui
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
  const __wireBoundary_b3 = (anchor) => bindFor({
    anchor,
    items: () => $scope.rail().value.kind_counts,
    key: (kc, __i) => {
      const __child = Object.create($scope);
      __child.kc = kc;
      __child.__i = __i;
      return (($scope2) => kc.id)(__child);
    },
    render: (kc, __i) => {
      const __childScope12 = Object.create($scope);
      __childScope12.kc = kc;
      __childScope12.__i = __i;
      __childScope12.__v = signal2(0);
      return (($scope2) => {
        const __frag13 = document.createDocumentFragment();
        const __c14 = [];
        const __e15 = document.createElement("button");
        __e15.setAttribute("class", "se-pill");
        __c14.push(bindClass(__e15, "is-active", () => (void $scope2.__v(), $scope2.kind() === $scope2.kc.id)));
        __c14.push(on(__e15, "click", function() {
          const __h = () => $scope2.kind.set($scope2.kc.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t16 = document.createTextNode("");
        __e15.appendChild(__t16);
        __t16.textContent = $scope2.kc.label;
        const __e17 = document.createElement("span");
        __e17.setAttribute("style", "opacity:0.6;font-variant-numeric:tabular-nums");
        const __t18 = document.createTextNode("");
        __e17.appendChild(__t18);
        __t18.textContent = $scope2.kc.count;
        __e15.appendChild(__e17);
        __frag13.appendChild(__e15);
        return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
          for (const c of __c14)
            try {
              c();
            } catch (e) {}
        }, update: (__new19, __newI20) => {
          $scope2.kc = __new19;
          $scope2.__i = __newI20;
          __t16.textContent = $scope2.kc.label;
          __t18.textContent = $scope2.kc.count;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope12);
    }
  });
  const __wireBoundary_b2 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.rail().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag8 = document.createDocumentFragment();
        const __c9 = [];
        const __e10 = document.createElement("div");
        __e10.setAttribute("class", "se-filters");
        __e10.setAttribute("role", "group");
        __e10.setAttribute("aria-label", "Filter by kind");
        const __forAnchor11 = document.createComment("for");
        __e10.appendChild(__forAnchor11);
        __c9.push(__wireBoundary_b3(__forAnchor11));
        __frag8.appendChild(__e10);
        return { nodes: Array.from(__frag8.childNodes), cleanup: () => {
          for (const c of __c9)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b9 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1, 2, 3, 4, 5],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope26 = Object.create($scope);
      __childScope26.n = n;
      __childScope26.__i = __i;
      return (($scope2) => {
        const __frag27 = document.createDocumentFragment();
        const __c28 = [];
        const __e29 = document.createElement("div");
        __e29.setAttribute("class", "se-skel-row");
        const __e30 = document.createElement("sui-skeleton");
        __e30.setAttribute("shape", "line");
        __e30.setAttribute("size", "md");
        __e29.appendChild(__e30);
        __frag27.appendChild(__e29);
        return { nodes: Array.from(__frag27.childNodes), cleanup: () => {
          for (const c of __c28)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope26);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag22 = document.createDocumentFragment();
    const __c23 = [];
    const __e24 = document.createElement("div");
    __e24.setAttribute("class", "se-skel");
    __e24.setAttribute("aria-busy", "true");
    __e24.setAttribute("aria-label", "Loading skills");
    const __forAnchor25 = document.createComment("for");
    __e24.appendChild(__forAnchor25);
    __c23.push(__wireBoundary_b9(__forAnchor25));
    __frag22.appendChild(__e24);
    return { nodes: Array.from(__frag22.childNodes), cleanup: () => {
      for (const c of __c23)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope32 = Object.create($scope);
    __childScope32.err = err;
    return (($scope2) => {
      const __frag33 = document.createDocumentFragment();
      const __c34 = [];
      const __e35 = document.createElement("sui-error-state");
      __e35.setAttribute("variant", "inline");
      __e35.setAttribute("size", "sm");
      __e35.setAttribute("title", "Couldn't load skills");
      const __t36 = document.createTextNode("");
      __e35.appendChild(__t36);
      __c34.push(bindText(__t36, () => $scope2.err.message));
      __frag33.appendChild(__e35);
      return { nodes: Array.from(__frag33.childNodes), cleanup: () => {
        for (const c of __c34)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope32);
  };
  const __wireBoundary_b15 = (anchor) => bindFor({
    anchor,
    items: () => $scope.rail().value.groups,
    key: (g, __i) => {
      const __child = Object.create($scope);
      __child.g = g;
      __child.__i = __i;
      return (($scope2) => g.source)(__child);
    },
    render: (g, __i) => {
      const __childScope49 = Object.create($scope);
      __childScope49.g = g;
      __childScope49.__i = __i;
      __childScope49.__v = signal2(0);
      return (($scope2) => {
        const __wireBoundary_b20 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.g.skills,
          key: (s, __i2) => {
            const __child = Object.create($scope2);
            __child.s = s;
            __child.__i = __i2;
            return (($scope3) => s.id)(__child);
          },
          render: (s, __i2) => {
            const __childScope69 = Object.create($scope2);
            __childScope69.s = s;
            __childScope69.__i = __i2;
            __childScope69.__v = signal2(0);
            return (($scope3) => {
              const __wireBoundary_b26 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.s.has_vars)
                    return 0;
                  return -1;
                },
                branches: [
                  () => {
                    const __frag82 = document.createDocumentFragment();
                    const __c83 = [];
                    const __e84 = document.createElement("span");
                    __e84.setAttribute("class", "se-dot vars");
                    __e84.setAttribute("title", "Variables");
                    __frag82.appendChild(__e84);
                    return { nodes: Array.from(__frag82.childNodes), cleanup: () => {
                      for (const c of __c83)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __wireBoundary_b27 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.s.has_refs)
                    return 0;
                  return -1;
                },
                branches: [
                  () => {
                    const __frag86 = document.createDocumentFragment();
                    const __c87 = [];
                    const __e88 = document.createElement("span");
                    __e88.setAttribute("class", "se-dot refs");
                    __e88.setAttribute("title", "Reference docs");
                    __frag86.appendChild(__e88);
                    return { nodes: Array.from(__frag86.childNodes), cleanup: () => {
                      for (const c of __c87)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __wireBoundary_b28 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.s.has_evals)
                    return 0;
                  return -1;
                },
                branches: [
                  () => {
                    const __frag90 = document.createDocumentFragment();
                    const __c91 = [];
                    const __e92 = document.createElement("span");
                    __e92.setAttribute("class", "se-dot evals");
                    __e92.setAttribute("title", "Evals");
                    __frag90.appendChild(__e92);
                    return { nodes: Array.from(__frag90.childNodes), cleanup: () => {
                      for (const c of __c91)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __frag70 = document.createDocumentFragment();
              const __c71 = [];
              const __e72 = document.createElement("div");
              __e72.setAttribute("role", "treeitem");
              __e72.setAttribute("class", "se-skill-row");
              __c71.push(bindAttr(__e72, "aria-selected", () => (void $scope3.__v(), $scope3.skillId() === $scope3.s.id ? "true" : "false")));
              const __e73 = document.createElement("button");
              __e73.setAttribute("class", "se-skill");
              __c71.push(bindClass(__e73, "is-active", () => (void $scope3.__v(), $scope3.skillId() === $scope3.s.id)));
              __c71.push(on(__e73, "click", function() {
                const __h = () => $scope3.skillId.set($scope3.s.id);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              {
                const __av74 = $scope3.s.description;
                if (__av74 == null || __av74 === false || __av74 === "")
                  __e73.removeAttribute("title");
                else
                  __e73.setAttribute("title", __av74 === true ? "" : String(__av74));
              }
              const __e75 = document.createElement("span");
              __e75.setAttribute("class", "se-chevron");
              __e75.setAttribute("aria-hidden", "true");
              const __e76 = document.createElement("sui-icon");
              __e76.setAttribute("name", "chevron-right");
              __e76.setAttribute("size", "xs");
              __e75.appendChild(__e76);
              __e73.appendChild(__e75);
              const __e77 = document.createElement("sui-icon");
              __e77.setAttribute("name", "zap");
              __e77.setAttribute("size", "xs");
              __e77.setAttribute("tone", "primary");
              __e73.appendChild(__e77);
              const __e78 = document.createElement("span");
              __e78.setAttribute("class", "se-name");
              const __t79 = document.createTextNode("");
              __e78.appendChild(__t79);
              __t79.textContent = $scope3.s.name;
              __e73.appendChild(__e78);
              const __e80 = document.createElement("span");
              __e80.setAttribute("class", "se-dots");
              const __ifAnchor81 = document.createComment("if");
              __e80.appendChild(__ifAnchor81);
              __c71.push(__wireBoundary_b26(__ifAnchor81));
              const __ifAnchor85 = document.createComment("if");
              __e80.appendChild(__ifAnchor85);
              __c71.push(__wireBoundary_b27(__ifAnchor85));
              const __ifAnchor89 = document.createComment("if");
              __e80.appendChild(__ifAnchor89);
              __c71.push(__wireBoundary_b28(__ifAnchor89));
              __e73.appendChild(__e80);
              __e72.appendChild(__e73);
              const __e93 = document.createElement("button");
              __e93.setAttribute("class", "se-invoke");
              {
                const __av94 = "Invoke " + $scope3.s.name;
                if (__av94 == null || __av94 === false || __av94 === "")
                  __e93.removeAttribute("aria-label");
                else
                  __e93.setAttribute("aria-label", __av94 === true ? "" : String(__av94));
              }
              __c71.push(bindProp(__e93, "disabled", () => (void $scope3.__v(), $scope3.invokeBusy() === $scope3.s.id)));
              __c71.push(on(__e93, "click", function() {
                const __h = () => $scope3.invoke($scope3.s.id);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              const __e95 = document.createElement("sui-icon");
              __e95.setAttribute("name", "play");
              __e95.setAttribute("size", "xs");
              __e93.appendChild(__e95);
              const __e96 = document.createElement("span");
              const __t97 = document.createTextNode("");
              __e96.appendChild(__t97);
              __c71.push(bindText(__t97, () => (void $scope3.__v(), $scope3.invokeBusy() === $scope3.s.id ? "Invoking…" : "Invoke")));
              __e93.appendChild(__e96);
              __e72.appendChild(__e93);
              __frag70.appendChild(__e72);
              return { nodes: Array.from(__frag70.childNodes), cleanup: () => {
                for (const c of __c71)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new98, __newI99) => {
                $scope3.s = __new98;
                $scope3.__i = __newI99;
                {
                  const __av74 = $scope3.s.description;
                  if (__av74 == null || __av74 === false || __av74 === "")
                    __e73.removeAttribute("title");
                  else
                    __e73.setAttribute("title", __av74 === true ? "" : String(__av74));
                }
                __t79.textContent = $scope3.s.name;
                {
                  const __av94 = "Invoke " + $scope3.s.name;
                  if (__av94 == null || __av94 === false || __av94 === "")
                    __e93.removeAttribute("aria-label");
                  else
                    __e93.setAttribute("aria-label", __av94 === true ? "" : String(__av94));
                }
                $scope3.__v.set((c) => c + 1);
              } };
            })(__childScope69);
          }
        });
        const __wireBoundary_b18 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.g.skills.length === 0)
              return 0;
            return 1;
          },
          branches: [
            () => {
              const __frag62 = document.createDocumentFragment();
              const __c63 = [];
              const __e64 = document.createElement("div");
              __e64.setAttribute("class", "se-empty");
              const __t65 = document.createTextNode("");
              __e64.appendChild(__t65);
              __c63.push(bindText(__t65, () => (void $scope2.__v(), $scope2.q() ? "No matches." : "No skills yet.")));
              __frag62.appendChild(__e64);
              return { nodes: Array.from(__frag62.childNodes), cleanup: () => {
                for (const c of __c63)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag66 = document.createDocumentFragment();
              const __c67 = [];
              const __forAnchor68 = document.createComment("for");
              __frag66.appendChild(__forAnchor68);
              __c67.push(__wireBoundary_b20(__forAnchor68));
              return { nodes: Array.from(__frag66.childNodes), cleanup: () => {
                for (const c of __c67)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag50 = document.createDocumentFragment();
        const __c51 = [];
        const __e52 = document.createElement("div");
        __e52.setAttribute("class", "se-section");
        const __e53 = document.createElement("div");
        __e53.setAttribute("class", "se-section-head");
        const __e54 = document.createElement("span");
        __e54.setAttribute("class", "se-section-label");
        const __e55 = document.createElement("sui-icon");
        __e55.setAttribute("name", "folder");
        __e55.setAttribute("size", "xs");
        __e55.setAttribute("tone", "muted");
        __e54.appendChild(__e55);
        const __t56 = document.createTextNode("");
        __e54.appendChild(__t56);
        __t56.textContent = $scope2.g.source === "PROJECT SKILLS" ? "Project Skills" : $scope2.g.source === "USER SKILLS" ? "Library" : $scope2.g.source;
        const __e57 = document.createElement("span");
        __e57.setAttribute("class", "count");
        const __t58 = document.createTextNode("(");
        __e57.appendChild(__t58);
        const __t59 = document.createTextNode("");
        __e57.appendChild(__t59);
        __t59.textContent = $scope2.g.skills.length;
        const __t60 = document.createTextNode(")");
        __e57.appendChild(__t60);
        __e54.appendChild(__e57);
        __e53.appendChild(__e54);
        __e52.appendChild(__e53);
        const __ifAnchor61 = document.createComment("if");
        __e52.appendChild(__ifAnchor61);
        __c51.push(__wireBoundary_b18(__ifAnchor61));
        __frag50.appendChild(__e52);
        return { nodes: Array.from(__frag50.childNodes), cleanup: () => {
          for (const c of __c51)
            try {
              c();
            } catch (e) {}
        }, update: (__new100, __newI101) => {
          $scope2.g = __new100;
          $scope2.__i = __newI101;
          __t56.textContent = $scope2.g.source === "PROJECT SKILLS" ? "Project Skills" : $scope2.g.source === "USER SKILLS" ? "Library" : $scope2.g.source;
          __t59.textContent = $scope2.g.skills.length;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope49);
    }
  });
  const __wireBoundary_b13 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.rail().value.groups.length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag42 = document.createDocumentFragment();
        const __c43 = [];
        const __e44 = document.createElement("div");
        __e44.setAttribute("class", "se-empty");
        const __t45 = document.createTextNode("");
        __e44.appendChild(__t45);
        __c43.push(bindText(__t45, () => $scope.q() ? 'No skills match "' + $scope.q() + '".' : "No skills discovered."));
        __frag42.appendChild(__e44);
        return { nodes: Array.from(__frag42.childNodes), cleanup: () => {
          for (const c of __c43)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag46 = document.createDocumentFragment();
        const __c47 = [];
        const __forAnchor48 = document.createComment("for");
        __frag46.appendChild(__forAnchor48);
        __c47.push(__wireBoundary_b15(__forAnchor48));
        return { nodes: Array.from(__frag46.childNodes), cleanup: () => {
          for (const c of __c47)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b12 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.rail().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag39 = document.createDocumentFragment();
        const __c40 = [];
        const __ifAnchor41 = document.createComment("if");
        __frag39.appendChild(__ifAnchor41);
        __c40.push(__wireBoundary_b13(__ifAnchor41));
        return { nodes: Array.from(__frag39.childNodes), cleanup: () => {
          for (const c of __c40)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b45 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skill().value.description)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag125 = document.createDocumentFragment();
        const __c126 = [];
        const __e127 = document.createElement("p");
        __e127.setAttribute("class", "sd-description");
        const __t128 = document.createTextNode("");
        __e127.appendChild(__t128);
        __c126.push(bindText(__t128, () => $scope.skill().value.description));
        __frag125.appendChild(__e127);
        return { nodes: Array.from(__frag125.childNodes), cleanup: () => {
          for (const c of __c126)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b49 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.invokeResult().ok)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag134 = document.createDocumentFragment();
        const __c135 = [];
        const __t136 = document.createTextNode(`
                Invoked · run `);
        __frag134.appendChild(__t136);
        const __e137 = document.createElement("code");
        const __t138 = document.createTextNode("");
        __e137.appendChild(__t138);
        __c135.push(bindText(__t138, () => $scope.invokeResult().run_id));
        __frag134.appendChild(__e137);
        const __t139 = document.createTextNode(" · session ");
        __frag134.appendChild(__t139);
        const __e140 = document.createElement("code");
        const __t141 = document.createTextNode("");
        __e140.appendChild(__t141);
        __c135.push(bindText(__t141, () => $scope.invokeResult().session_id));
        __frag134.appendChild(__e140);
        const __e142 = document.createElement("button");
        __e142.setAttribute("class", "sd-invoke-dismiss");
        __e142.setAttribute("aria-label", "Dismiss");
        __c135.push(on(__e142, "click", function() {
          const __h = () => $scope.invokeResult.set(null);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t143 = document.createTextNode("×");
        __e142.appendChild(__t143);
        __frag134.appendChild(__e142);
        return { nodes: Array.from(__frag134.childNodes), cleanup: () => {
          for (const c of __c135)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag144 = document.createDocumentFragment();
        const __c145 = [];
        const __t146 = document.createTextNode("");
        __frag144.appendChild(__t146);
        __c145.push(bindText(__t146, () => $scope.invokeResult().error));
        const __e147 = document.createElement("button");
        __e147.setAttribute("class", "sd-invoke-dismiss");
        __e147.setAttribute("aria-label", "Dismiss");
        __c145.push(on(__e147, "click", function() {
          const __h = () => $scope.invokeResult.set(null);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t148 = document.createTextNode("×");
        __e147.appendChild(__t148);
        __frag144.appendChild(__e147);
        return { nodes: Array.from(__frag144.childNodes), cleanup: () => {
          for (const c of __c145)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b47 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.invokeResult())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag130 = document.createDocumentFragment();
        const __c131 = [];
        const __e132 = document.createElement("sui-alert");
        __c131.push(bindAttr(__e132, "tone", () => $scope.invokeResult().ok ? "success" : "warning"));
        __e132.setAttribute("class", "sd-invoke-result");
        const __ifAnchor133 = document.createComment("if");
        __e132.appendChild(__ifAnchor133);
        __c131.push(__wireBoundary_b49(__ifAnchor133));
        __frag130.appendChild(__e132);
        return { nodes: Array.from(__frag130.childNodes), cleanup: () => {
          for (const c of __c131)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b64 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.viewRaw())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag166 = document.createDocumentFragment();
        const __c167 = [];
        const __e168 = document.createElement("pre");
        __e168.setAttribute("class", "sd-raw");
        const __e169 = document.createElement("code");
        const __t170 = document.createTextNode("");
        __e169.appendChild(__t170);
        __c167.push(bindText(__t170, () => $scope.skill().value.body));
        __e168.appendChild(__e169);
        __frag166.appendChild(__e168);
        return { nodes: Array.from(__frag166.childNodes), cleanup: () => {
          for (const c of __c167)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag171 = document.createDocumentFragment();
        const __c172 = [];
        const __e173 = document.createElement("sui-markdown-editor");
        __c172.push(bindAttr(__e173, "value", () => $scope.skill().value.body));
        __frag171.appendChild(__e173);
        return { nodes: Array.from(__frag171.childNodes), cleanup: () => {
          for (const c of __c172)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b63 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skill().value.body)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag163 = document.createDocumentFragment();
        const __c164 = [];
        const __ifAnchor165 = document.createComment("if");
        __frag163.appendChild(__ifAnchor165);
        __c164.push(__wireBoundary_b64(__ifAnchor165));
        return { nodes: Array.from(__frag163.childNodes), cleanup: () => {
          for (const c of __c164)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag174 = document.createDocumentFragment();
        const __c175 = [];
        const __e176 = document.createElement("div");
        __e176.setAttribute("class", "se-empty-state");
        __e176.setAttribute("style", "padding:40px 0");
        const __e177 = document.createElement("sui-icon");
        __e177.setAttribute("name", "file");
        __e177.setAttribute("size", "xl");
        __e177.setAttribute("tone", "muted");
        __e176.appendChild(__e177);
        const __e178 = document.createElement("span");
        __e178.setAttribute("class", "se-empty-desc");
        const __t179 = document.createTextNode("This skill has no SKILL.md content yet.");
        __e178.appendChild(__t179);
        __e176.appendChild(__e178);
        __frag174.appendChild(__e176);
        return { nodes: Array.from(__frag174.childNodes), cleanup: () => {
          for (const c of __c175)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b70 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skill().value.description)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag198 = document.createDocumentFragment();
        const __c199 = [];
        const __e200 = document.createElement("dt");
        const __t201 = document.createTextNode("description");
        __e200.appendChild(__t201);
        __frag198.appendChild(__e200);
        const __e202 = document.createElement("dd");
        const __t203 = document.createTextNode("");
        __e202.appendChild(__t203);
        __c199.push(bindText(__t203, () => $scope.skill().value.description));
        __frag198.appendChild(__e202);
        return { nodes: Array.from(__frag198.childNodes), cleanup: () => {
          for (const c of __c199)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b73 = (anchor) => bindFor({
    anchor,
    items: () => $scope.skill().value.tags,
    key: (t, __i) => {
      const __child = Object.create($scope);
      __child.t = t;
      __child.__i = __i;
      return (($scope2) => t)(__child);
    },
    render: (t, __i) => {
      const __childScope212 = Object.create($scope);
      __childScope212.t = t;
      __childScope212.__i = __i;
      __childScope212.__v = signal2(0);
      return (($scope2) => {
        const __frag213 = document.createDocumentFragment();
        const __c214 = [];
        const __e215 = document.createElement("sui-chip");
        __e215.setAttribute("variant", "soft");
        __e215.setAttribute("size", "sm");
        __e215.setAttribute("tone", "muted");
        const __t216 = document.createTextNode("");
        __e215.appendChild(__t216);
        __t216.textContent = $scope2.t;
        __frag213.appendChild(__e215);
        return { nodes: Array.from(__frag213.childNodes), cleanup: () => {
          for (const c of __c214)
            try {
              c();
            } catch (e) {}
        }, update: (__new217, __newI218) => {
          $scope2.t = __new217;
          $scope2.__i = __newI218;
          __t216.textContent = $scope2.t;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope212);
    }
  });
  const __wireBoundary_b72 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skill().value.tags && $scope.skill().value.tags.length > 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag205 = document.createDocumentFragment();
        const __c206 = [];
        const __e207 = document.createElement("div");
        __e207.setAttribute("class", "sd-meta-section");
        const __e208 = document.createElement("div");
        __e208.setAttribute("class", "sd-meta-label");
        const __t209 = document.createTextNode("Tags");
        __e208.appendChild(__t209);
        __e207.appendChild(__e208);
        const __e210 = document.createElement("div");
        __e210.setAttribute("class", "sd-meta-chips");
        const __forAnchor211 = document.createComment("for");
        __e210.appendChild(__forAnchor211);
        __c206.push(__wireBoundary_b73(__forAnchor211));
        __e207.appendChild(__e210);
        __frag205.appendChild(__e207);
        return { nodes: Array.from(__frag205.childNodes), cleanup: () => {
          for (const c of __c206)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b76 = (anchor) => bindFor({
    anchor,
    items: () => $scope.skill().value.authors,
    key: (a, __i) => {
      const __child = Object.create($scope);
      __child.a = a;
      __child.__i = __i;
      return (($scope2) => a)(__child);
    },
    render: (a, __i) => {
      const __childScope227 = Object.create($scope);
      __childScope227.a = a;
      __childScope227.__i = __i;
      __childScope227.__v = signal2(0);
      return (($scope2) => {
        const __frag228 = document.createDocumentFragment();
        const __c229 = [];
        const __e230 = document.createElement("sui-chip");
        __e230.setAttribute("variant", "soft");
        __e230.setAttribute("size", "sm");
        __e230.setAttribute("tone", "muted");
        const __t231 = document.createTextNode("");
        __e230.appendChild(__t231);
        __t231.textContent = $scope2.a;
        __frag228.appendChild(__e230);
        return { nodes: Array.from(__frag228.childNodes), cleanup: () => {
          for (const c of __c229)
            try {
              c();
            } catch (e) {}
        }, update: (__new232, __newI233) => {
          $scope2.a = __new232;
          $scope2.__i = __newI233;
          __t231.textContent = $scope2.a;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope227);
    }
  });
  const __wireBoundary_b75 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skill().value.authors && $scope.skill().value.authors.length > 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag220 = document.createDocumentFragment();
        const __c221 = [];
        const __e222 = document.createElement("div");
        __e222.setAttribute("class", "sd-meta-section");
        const __e223 = document.createElement("div");
        __e223.setAttribute("class", "sd-meta-label");
        const __t224 = document.createTextNode("Authors");
        __e223.appendChild(__t224);
        __e222.appendChild(__e223);
        const __e225 = document.createElement("div");
        __e225.setAttribute("class", "sd-meta-chips");
        const __forAnchor226 = document.createComment("for");
        __e225.appendChild(__forAnchor226);
        __c221.push(__wireBoundary_b76(__forAnchor226));
        __e222.appendChild(__e225);
        __frag220.appendChild(__e222);
        return { nodes: Array.from(__frag220.childNodes), cleanup: () => {
          for (const c of __c221)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b80 = (anchor) => bindFor({
    anchor,
    items: () => $scope.skill().value.variables,
    key: (v, __i) => {
      const __child = Object.create($scope);
      __child.v = v;
      __child.__i = __i;
      return (($scope2) => v)(__child);
    },
    render: (v, __i) => {
      const __childScope244 = Object.create($scope);
      __childScope244.v = v;
      __childScope244.__i = __i;
      __childScope244.__v = signal2(0);
      return (($scope2) => {
        const __frag245 = document.createDocumentFragment();
        const __c246 = [];
        const __e247 = document.createElement("li");
        const __e248 = document.createElement("code");
        const __t249 = document.createTextNode("");
        __e248.appendChild(__t249);
        __t249.textContent = $scope2.v;
        __e247.appendChild(__e248);
        __frag245.appendChild(__e247);
        return { nodes: Array.from(__frag245.childNodes), cleanup: () => {
          for (const c of __c246)
            try {
              c();
            } catch (e) {}
        }, update: (__new250, __newI251) => {
          $scope2.v = __new250;
          $scope2.__i = __newI251;
          __t249.textContent = $scope2.v;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope244);
    }
  });
  const __wireBoundary_b79 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skill().value.variables && $scope.skill().value.variables.length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag240 = document.createDocumentFragment();
        const __c241 = [];
        const __e242 = document.createElement("ul");
        __e242.setAttribute("class", "sd-meta-vars");
        const __forAnchor243 = document.createComment("for");
        __e242.appendChild(__forAnchor243);
        __c241.push(__wireBoundary_b80(__forAnchor243));
        __frag240.appendChild(__e242);
        return { nodes: Array.from(__frag240.childNodes), cleanup: () => {
          for (const c of __c241)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag252 = document.createDocumentFragment();
        const __c253 = [];
        const __e254 = document.createElement("p");
        __e254.setAttribute("class", "sd-meta-empty");
        const __t255 = document.createTextNode("No variables defined.");
        __e254.appendChild(__t255);
        __frag252.appendChild(__e254);
        return { nodes: Array.from(__frag252.childNodes), cleanup: () => {
          for (const c of __c253)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b83 = (anchor) => bindFor({
    anchor,
    items: () => $scope.skill().value.files || [],
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return (($scope2) => f)(__child);
    },
    render: (f, __i) => {
      const __childScope263 = Object.create($scope);
      __childScope263.f = f;
      __childScope263.__i = __i;
      __childScope263.__v = signal2(0);
      return (($scope2) => {
        const __frag264 = document.createDocumentFragment();
        const __c265 = [];
        const __e266 = document.createElement("li");
        const __e267 = document.createElement("code");
        const __t268 = document.createTextNode("");
        __e267.appendChild(__t268);
        __t268.textContent = $scope2.f;
        __e266.appendChild(__e267);
        __frag264.appendChild(__e266);
        return { nodes: Array.from(__frag264.childNodes), cleanup: () => {
          for (const c of __c265)
            try {
              c();
            } catch (e) {}
        }, update: (__new269, __newI270) => {
          $scope2.f = __new269;
          $scope2.__i = __newI270;
          __t268.textContent = $scope2.f;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope263);
    }
  });
  const __renderWhileLoading_1 = () => {
    const __frag274 = document.createDocumentFragment();
    const __c275 = [];
    const __e276 = document.createElement("div");
    __e276.setAttribute("class", "se-empty-state");
    const __e277 = document.createElement("sui-spinner");
    __e277.setAttribute("size", "md");
    __e276.appendChild(__e277);
    const __e278 = document.createElement("span");
    __e278.setAttribute("class", "se-empty-desc");
    const __t279 = document.createTextNode("Loading runs…");
    __e278.appendChild(__t279);
    __e276.appendChild(__e278);
    __frag274.appendChild(__e276);
    return { nodes: Array.from(__frag274.childNodes), cleanup: () => {
      for (const c of __c275)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_1 = (err) => {
    const __childScope281 = Object.create($scope);
    __childScope281.err = err;
    return (($scope2) => {
      const __frag282 = document.createDocumentFragment();
      const __c283 = [];
      const __e284 = document.createElement("sui-error-state");
      __e284.setAttribute("variant", "inline");
      __e284.setAttribute("size", "sm");
      __e284.setAttribute("title", "Couldn't load runs");
      const __t285 = document.createTextNode("");
      __e284.appendChild(__t285);
      __c283.push(bindText(__t285, () => $scope2.err.message));
      __frag282.appendChild(__e284);
      return { nodes: Array.from(__frag282.childNodes), cleanup: () => {
        for (const c of __c283)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope281);
  };
  const __wireBoundary_b90 = (anchor) => bindFor({
    anchor,
    items: () => $scope.runs().value.rows,
    key: (r, __i) => {
      const __child = Object.create($scope);
      __child.r = r;
      __child.__i = __i;
      return (($scope2) => r.id)(__child);
    },
    render: (r, __i) => {
      const __childScope306 = Object.create($scope);
      __childScope306.r = r;
      __childScope306.__i = __i;
      __childScope306.__v = signal2(0);
      return (($scope2) => {
        const __wireBoundary_b94 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.r.session_id)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag317 = document.createDocumentFragment();
              const __c318 = [];
              const __e319 = document.createElement("span");
              __e319.setAttribute("class", "sd-run-session");
              {
                const __av320 = "Session " + $scope2.r.session_id;
                if (__av320 == null || __av320 === false || __av320 === "")
                  __e319.removeAttribute("title");
                else
                  __e319.setAttribute("title", __av320 === true ? "" : String(__av320));
              }
              const __e321 = document.createElement("sui-icon");
              __e321.setAttribute("name", "link");
              __e321.setAttribute("size", "xs");
              __e321.setAttribute("tone", "muted");
              __e319.appendChild(__e321);
              const __t322 = document.createTextNode("");
              __e319.appendChild(__t322);
              __c318.push(bindText(__t322, () => (void $scope2.__v(), $scope2.r.session_id.slice(0, 8))));
              __frag317.appendChild(__e319);
              return { nodes: Array.from(__frag317.childNodes), cleanup: () => {
                for (const c of __c318)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag307 = document.createDocumentFragment();
        const __c308 = [];
        const __e309 = document.createElement("li");
        __e309.setAttribute("class", "sd-run-row");
        const __e310 = document.createElement("sui-icon");
        __e310.setAttribute("name", "play");
        __e310.setAttribute("size", "xs");
        __e310.setAttribute("tone", "primary");
        __e309.appendChild(__e310);
        const __e311 = document.createElement("span");
        __e311.setAttribute("class", "sd-run-time");
        {
          const __av312 = $scope2.r.timestamp;
          if (__av312 == null || __av312 === false || __av312 === "")
            __e311.removeAttribute("title");
          else
            __e311.setAttribute("title", __av312 === true ? "" : String(__av312));
        }
        const __t313 = document.createTextNode("");
        __e311.appendChild(__t313);
        __t313.textContent = $scope2.r.timestamp;
        __e309.appendChild(__e311);
        const __e314 = document.createElement("span");
        __e314.setAttribute("class", "sd-run-actor");
        const __t315 = document.createTextNode("");
        __e314.appendChild(__t315);
        __t315.textContent = $scope2.r.actor;
        __e309.appendChild(__e314);
        const __ifAnchor316 = document.createComment("if");
        __e309.appendChild(__ifAnchor316);
        __c308.push(__wireBoundary_b94(__ifAnchor316));
        const __e323 = document.createElement("code");
        __e323.setAttribute("class", "sd-run-id");
        {
          const __av324 = $scope2.r.id;
          if (__av324 == null || __av324 === false || __av324 === "")
            __e323.removeAttribute("title");
          else
            __e323.setAttribute("title", __av324 === true ? "" : String(__av324));
        }
        const __t325 = document.createTextNode("");
        __e323.appendChild(__t325);
        __c308.push(bindText(__t325, () => (void $scope2.__v(), $scope2.r.id.slice(0, 12))));
        __e309.appendChild(__e323);
        __frag307.appendChild(__e309);
        return { nodes: Array.from(__frag307.childNodes), cleanup: () => {
          for (const c of __c308)
            try {
              c();
            } catch (e) {}
        }, update: (__new326, __newI327) => {
          $scope2.r = __new326;
          $scope2.__i = __newI327;
          {
            const __av312 = $scope2.r.timestamp;
            if (__av312 == null || __av312 === false || __av312 === "")
              __e311.removeAttribute("title");
            else
              __e311.setAttribute("title", __av312 === true ? "" : String(__av312));
          }
          __t313.textContent = $scope2.r.timestamp;
          __t315.textContent = $scope2.r.actor;
          {
            const __av324 = $scope2.r.id;
            if (__av324 == null || __av324 === false || __av324 === "")
              __e323.removeAttribute("title");
            else
              __e323.setAttribute("title", __av324 === true ? "" : String(__av324));
          }
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope306);
    }
  });
  const __wireBoundary_b89 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.runs().value.rows.length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag291 = document.createDocumentFragment();
        const __c292 = [];
        const __e293 = document.createElement("div");
        __e293.setAttribute("class", "se-empty-state");
        __e293.setAttribute("style", "padding:40px 0");
        const __e294 = document.createElement("sui-icon");
        __e294.setAttribute("name", "zap");
        __e294.setAttribute("size", "xl");
        __e294.setAttribute("tone", "muted");
        __e293.appendChild(__e294);
        const __e295 = document.createElement("span");
        __e295.setAttribute("class", "se-empty-title");
        const __t296 = document.createTextNode("No invocations yet");
        __e295.appendChild(__t296);
        __e293.appendChild(__e295);
        const __e297 = document.createElement("span");
        __e297.setAttribute("class", "se-empty-desc");
        const __t298 = document.createTextNode("Click ");
        __e297.appendChild(__t298);
        const __e299 = document.createElement("b");
        const __t300 = document.createTextNode("Invoke");
        __e299.appendChild(__t300);
        __e297.appendChild(__e299);
        const __t301 = document.createTextNode(" to start a session attached to this skill.");
        __e297.appendChild(__t301);
        __e293.appendChild(__e297);
        __frag291.appendChild(__e293);
        return { nodes: Array.from(__frag291.childNodes), cleanup: () => {
          for (const c of __c292)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag302 = document.createDocumentFragment();
        const __c303 = [];
        const __e304 = document.createElement("ul");
        __e304.setAttribute("class", "sd-runs-list");
        const __forAnchor305 = document.createComment("for");
        __e304.appendChild(__forAnchor305);
        __c303.push(__wireBoundary_b90(__forAnchor305));
        __frag302.appendChild(__e304);
        return { nodes: Array.from(__frag302.childNodes), cleanup: () => {
          for (const c of __c303)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b88 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.runs().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag288 = document.createDocumentFragment();
        const __c289 = [];
        const __ifAnchor290 = document.createComment("if");
        __frag288.appendChild(__ifAnchor290);
        __c289.push(__wireBoundary_b89(__ifAnchor290));
        return { nodes: Array.from(__frag288.childNodes), cleanup: () => {
          for (const c of __c289)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b62 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tab() === "content")
        return 0;
      if ($scope.tab() === "metadata")
        return 1;
      if ($scope.tab() === "runs")
        return 2;
      return -1;
    },
    branches: [
      () => {
        const __frag159 = document.createDocumentFragment();
        const __c160 = [];
        const __e161 = document.createElement("div");
        __e161.setAttribute("class", "se-editor-content");
        const __ifAnchor162 = document.createComment("if");
        __e161.appendChild(__ifAnchor162);
        __c160.push(__wireBoundary_b63(__ifAnchor162));
        __frag159.appendChild(__e161);
        return { nodes: Array.from(__frag159.childNodes), cleanup: () => {
          for (const c of __c160)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag180 = document.createDocumentFragment();
        const __c181 = [];
        const __e182 = document.createElement("div");
        __e182.setAttribute("class", "se-editor-content sd-meta");
        const __e183 = document.createElement("dl");
        __e183.setAttribute("class", "sd-meta-list");
        const __e184 = document.createElement("dt");
        const __t185 = document.createTextNode("id");
        __e184.appendChild(__t185);
        __e183.appendChild(__e184);
        const __e186 = document.createElement("dd");
        const __e187 = document.createElement("code");
        const __t188 = document.createTextNode("");
        __e187.appendChild(__t188);
        __c181.push(bindText(__t188, () => $scope.skill().value.id));
        __e186.appendChild(__e187);
        __e183.appendChild(__e186);
        const __e189 = document.createElement("dt");
        const __t190 = document.createTextNode("kind");
        __e189.appendChild(__t190);
        __e183.appendChild(__e189);
        const __e191 = document.createElement("dd");
        const __t192 = document.createTextNode("");
        __e191.appendChild(__t192);
        __c181.push(bindText(__t192, () => $scope.skill().value.kind));
        __e183.appendChild(__e191);
        const __e193 = document.createElement("dt");
        const __t194 = document.createTextNode("source");
        __e193.appendChild(__t194);
        __e183.appendChild(__e193);
        const __e195 = document.createElement("dd");
        const __t196 = document.createTextNode("");
        __e195.appendChild(__t196);
        __c181.push(bindText(__t196, () => $scope.skill().value.source));
        __e183.appendChild(__e195);
        const __ifAnchor197 = document.createComment("if");
        __e183.appendChild(__ifAnchor197);
        __c181.push(__wireBoundary_b70(__ifAnchor197));
        __e182.appendChild(__e183);
        const __ifAnchor204 = document.createComment("if");
        __e182.appendChild(__ifAnchor204);
        __c181.push(__wireBoundary_b72(__ifAnchor204));
        const __ifAnchor219 = document.createComment("if");
        __e182.appendChild(__ifAnchor219);
        __c181.push(__wireBoundary_b75(__ifAnchor219));
        const __e234 = document.createElement("div");
        __e234.setAttribute("class", "sd-meta-section");
        const __e235 = document.createElement("div");
        __e235.setAttribute("class", "sd-meta-label");
        const __t236 = document.createTextNode("Variables (");
        __e235.appendChild(__t236);
        const __t237 = document.createTextNode("");
        __e235.appendChild(__t237);
        __c181.push(bindText(__t237, () => $scope.skill().value.variables?.length ?? 0));
        const __t238 = document.createTextNode(")");
        __e235.appendChild(__t238);
        __e234.appendChild(__e235);
        const __ifAnchor239 = document.createComment("if");
        __e234.appendChild(__ifAnchor239);
        __c181.push(__wireBoundary_b79(__ifAnchor239));
        __e182.appendChild(__e234);
        const __e256 = document.createElement("div");
        __e256.setAttribute("class", "sd-meta-section");
        const __e257 = document.createElement("div");
        __e257.setAttribute("class", "sd-meta-label");
        const __t258 = document.createTextNode("Files (");
        __e257.appendChild(__t258);
        const __t259 = document.createTextNode("");
        __e257.appendChild(__t259);
        __c181.push(bindText(__t259, () => $scope.skill().value.files?.length ?? 0));
        const __t260 = document.createTextNode(")");
        __e257.appendChild(__t260);
        __e256.appendChild(__e257);
        const __e261 = document.createElement("ul");
        __e261.setAttribute("class", "sd-meta-files");
        const __forAnchor262 = document.createComment("for");
        __e261.appendChild(__forAnchor262);
        __c181.push(__wireBoundary_b83(__forAnchor262));
        __e256.appendChild(__e261);
        __e182.appendChild(__e256);
        __frag180.appendChild(__e182);
        return { nodes: Array.from(__frag180.childNodes), cleanup: () => {
          for (const c of __c181)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag271 = document.createDocumentFragment();
        const __c272 = [];
        const __e273 = document.createElement("div");
        __e273.setAttribute("class", "se-editor-content sd-runs");
        const __wlAnchor280 = document.createComment("while-loading");
        __e273.appendChild(__wlAnchor280);
        __c272.push(bindWhileLoading({ anchor: __wlAnchor280, signal: $scope.runs, render: __renderWhileLoading_1 }));
        const __weAnchor286 = document.createComment("while-error");
        __e273.appendChild(__weAnchor286);
        __c272.push(bindWhileError({ anchor: __weAnchor286, signal: $scope.runs, render: __renderWhileError_1 }));
        const __ifAnchor287 = document.createComment("if");
        __e273.appendChild(__ifAnchor287);
        __c272.push(__wireBoundary_b88(__ifAnchor287));
        __frag271.appendChild(__e273);
        return { nodes: Array.from(__frag271.childNodes), cleanup: () => {
          for (const c of __c272)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b33 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.skillId() && $scope.skill().kind === "ok" && $scope.skill().value)
        return 0;
      if ($scope.skillId() && $scope.skill().kind === "loading")
        return 1;
      if ($scope.skillId() && $scope.skill().kind === "error")
        return 2;
      return 3;
    },
    branches: [
      () => {
        const __frag104 = document.createDocumentFragment();
        const __c105 = [];
        const __e106 = document.createElement("header");
        __e106.setAttribute("class", "sd-header");
        const __e107 = document.createElement("div");
        __e107.setAttribute("class", "sd-title-row");
        const __e108 = document.createElement("div");
        __e108.setAttribute("class", "sd-title-main");
        const __e109 = document.createElement("h1");
        __e109.setAttribute("class", "sd-name");
        const __t110 = document.createTextNode("");
        __e109.appendChild(__t110);
        __c105.push(bindText(__t110, () => $scope.skill().value.name));
        __e108.appendChild(__e109);
        const __e111 = document.createElement("sui-chip");
        __e111.setAttribute("variant", "soft");
        __e111.setAttribute("size", "sm");
        __c105.push(bindAttr(__e111, "tone", () => $scope.skill().value.kind === "smart" ? "primary" : $scope.skill().value.kind === "docs" ? "info" : $scope.skill().value.kind === "operator" ? "warning" : "muted"));
        const __t112 = document.createTextNode("");
        __e111.appendChild(__t112);
        __c105.push(bindText(__t112, () => $scope.skill().value.kind));
        __e108.appendChild(__e111);
        const __e113 = document.createElement("span");
        __e113.setAttribute("class", "sd-source-label");
        const __t114 = document.createTextNode("");
        __e113.appendChild(__t114);
        __c105.push(bindText(__t114, () => $scope.skill().value.source));
        __e108.appendChild(__e113);
        __e107.appendChild(__e108);
        const __e115 = document.createElement("div");
        __e115.setAttribute("class", "sd-title-actions");
        const __e116 = document.createElement("button");
        __e116.setAttribute("class", "sd-action sd-source-btn");
        __c105.push(bindClass(__e116, "is-active", () => $scope.viewRaw()));
        __c105.push(bindAttr(__e116, "aria-pressed", () => $scope.viewRaw() ? "true" : "false"));
        __c105.push(on(__e116, "click", function() {
          const __h = $scope.openSource;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e117 = document.createElement("sui-icon");
        __e117.setAttribute("name", "file-code");
        __e117.setAttribute("size", "xs");
        __e116.appendChild(__e117);
        const __e118 = document.createElement("span");
        const __t119 = document.createTextNode("");
        __e118.appendChild(__t119);
        __c105.push(bindText(__t119, () => $scope.viewRaw() ? "Rendered" : "Source"));
        __e116.appendChild(__e118);
        __e115.appendChild(__e116);
        const __e120 = document.createElement("button");
        __e120.setAttribute("class", "sd-action sd-invoke-btn");
        __c105.push(bindProp(__e120, "disabled", () => $scope.invokeBusy() === $scope.skill().value.id));
        __c105.push(on(__e120, "click", function() {
          const __h = () => $scope.invoke($scope.skill().value.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e121 = document.createElement("sui-icon");
        __e121.setAttribute("name", "play");
        __e121.setAttribute("size", "xs");
        __e120.appendChild(__e121);
        const __e122 = document.createElement("span");
        const __t123 = document.createTextNode("");
        __e122.appendChild(__t123);
        __c105.push(bindText(__t123, () => $scope.invokeBusy() === $scope.skill().value.id ? "Invoking…" : "Invoke"));
        __e120.appendChild(__e122);
        __e115.appendChild(__e120);
        __e107.appendChild(__e115);
        __e106.appendChild(__e107);
        const __ifAnchor124 = document.createComment("if");
        __e106.appendChild(__ifAnchor124);
        __c105.push(__wireBoundary_b45(__ifAnchor124));
        const __ifAnchor129 = document.createComment("if");
        __e106.appendChild(__ifAnchor129);
        __c105.push(__wireBoundary_b47(__ifAnchor129));
        __frag104.appendChild(__e106);
        const __e149 = document.createElement("div");
        __e149.setAttribute("class", "sd-tabs-row");
        const __e150 = document.createElement("sui-tabs");
        __e150.setAttribute("variant", "underline");
        const __e151 = document.createElement("sui-tab");
        __e151.setAttribute("name", "content");
        __c105.push(bindAttr(__e151, "active", () => $scope.tab() === "content"));
        __c105.push(on(__e151, "click", function() {
          const __h = () => $scope.tab.set("content");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t152 = document.createTextNode("SKILL.md");
        __e151.appendChild(__t152);
        __e150.appendChild(__e151);
        const __e153 = document.createElement("sui-tab");
        __e153.setAttribute("name", "metadata");
        __c105.push(bindAttr(__e153, "active", () => $scope.tab() === "metadata"));
        __c105.push(on(__e153, "click", function() {
          const __h = () => $scope.tab.set("metadata");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t154 = document.createTextNode("Metadata");
        __e153.appendChild(__t154);
        __e150.appendChild(__e153);
        const __e155 = document.createElement("sui-tab");
        __e155.setAttribute("name", "runs");
        __c105.push(bindAttr(__e155, "active", () => $scope.tab() === "runs"));
        __c105.push(bindAttr(__e155, "count", () => $scope.runs().kind === "ok" ? $scope.runs().value.rows.length + "" : ""));
        __c105.push(on(__e155, "click", function() {
          const __h = () => $scope.tab.set("runs");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t156 = document.createTextNode("Runs");
        __e155.appendChild(__t156);
        __e150.appendChild(__e155);
        __e149.appendChild(__e150);
        __frag104.appendChild(__e149);
        const __e157 = document.createElement("div");
        __e157.setAttribute("class", "sd-tab-body");
        const __ifAnchor158 = document.createComment("if");
        __e157.appendChild(__ifAnchor158);
        __c105.push(__wireBoundary_b62(__ifAnchor158));
        __frag104.appendChild(__e157);
        return { nodes: Array.from(__frag104.childNodes), cleanup: () => {
          for (const c of __c105)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag328 = document.createDocumentFragment();
        const __c329 = [];
        const __e330 = document.createElement("div");
        __e330.setAttribute("class", "se-empty-state");
        const __e331 = document.createElement("sui-spinner");
        __e331.setAttribute("size", "md");
        __e330.appendChild(__e331);
        const __e332 = document.createElement("span");
        __e332.setAttribute("class", "se-empty-desc");
        const __t333 = document.createTextNode("Loading skill…");
        __e332.appendChild(__t333);
        __e330.appendChild(__e332);
        __frag328.appendChild(__e330);
        return { nodes: Array.from(__frag328.childNodes), cleanup: () => {
          for (const c of __c329)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag334 = document.createDocumentFragment();
        const __c335 = [];
        const __e336 = document.createElement("div");
        __e336.setAttribute("class", "se-empty-state");
        const __e337 = document.createElement("sui-icon");
        __e337.setAttribute("name", "alert-triangle");
        __e337.setAttribute("size", "xl");
        __e337.setAttribute("tone", "warning");
        __e336.appendChild(__e337);
        const __e338 = document.createElement("span");
        __e338.setAttribute("class", "se-empty-title");
        const __t339 = document.createTextNode("Couldn't load skill");
        __e338.appendChild(__t339);
        __e336.appendChild(__e338);
        const __e340 = document.createElement("span");
        __e340.setAttribute("class", "se-empty-desc");
        const __t341 = document.createTextNode("");
        __e340.appendChild(__t341);
        __c335.push(bindText(__t341, () => $scope.skill().error.message));
        __e336.appendChild(__e340);
        __frag334.appendChild(__e336);
        return { nodes: Array.from(__frag334.childNodes), cleanup: () => {
          for (const c of __c335)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag342 = document.createDocumentFragment();
        const __c343 = [];
        const __e344 = document.createElement("div");
        __e344.setAttribute("class", "se-empty-state");
        const __e345 = document.createElement("sui-icon");
        __e345.setAttribute("name", "zap");
        __e345.setAttribute("size", "xl");
        __e345.setAttribute("class", "se-empty-icon");
        __e345.setAttribute("tone", "muted");
        __e344.appendChild(__e345);
        const __e346 = document.createElement("span");
        __e346.setAttribute("class", "se-empty-title");
        const __t347 = document.createTextNode("No skill selected");
        __e346.appendChild(__t347);
        __e344.appendChild(__e346);
        const __e348 = document.createElement("span");
        __e348.setAttribute("class", "se-empty-desc");
        const __t349 = document.createTextNode("Select a skill from the explorer to view its content and invoke it.");
        __e348.appendChild(__t349);
        __e344.appendChild(__e348);
        __frag342.appendChild(__e344);
        return { nodes: Array.from(__frag342.childNodes), cleanup: () => {
          for (const c of __c343)
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
      __e0.setAttribute("class", "ship-view ship-view-skills");
      const __e1 = document.createElement("div");
      __e1.setAttribute("class", "skills-ide");
      const __e2 = document.createElement("aside");
      __e2.setAttribute("class", "skills-explorer");
      __e2.setAttribute("aria-label", "Skills file explorer");
      const __e3 = document.createElement("div");
      __e3.setAttribute("class", "se-head");
      const __e4 = document.createElement("div");
      __e4.setAttribute("class", "se-search");
      const __e5 = document.createElement("sui-icon");
      __e5.setAttribute("name", "search");
      __e5.setAttribute("size", "sm");
      __e5.setAttribute("tone", "muted");
      __e4.appendChild(__e5);
      const __e6 = document.createElement("input");
      __e6.setAttribute("type", "text");
      __e6.setAttribute("placeholder", "Search skills…");
      __e6.setAttribute("spellcheck", "false");
      __e6.setAttribute("aria-label", "Search skills");
      __cleanups.push(bindValue(__e6, $scope.q));
      __cleanups.push(on(__e6, "input", function() {
        const __h = (e) => $scope.q.set(e.target.value);
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e4.appendChild(__e6);
      __e3.appendChild(__e4);
      const __ifAnchor7 = document.createComment("if");
      __e3.appendChild(__ifAnchor7);
      __cleanups.push(__wireBoundary_b2(__ifAnchor7));
      __e2.appendChild(__e3);
      const __e21 = document.createElement("div");
      __e21.setAttribute("class", "se-body");
      __e21.setAttribute("role", "tree");
      __e21.setAttribute("aria-label", "Skills tree");
      const __wlAnchor31 = document.createComment("while-loading");
      __e21.appendChild(__wlAnchor31);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor31, signal: $scope.rail, render: __renderWhileLoading_0 }));
      const __weAnchor37 = document.createComment("while-error");
      __e21.appendChild(__weAnchor37);
      __cleanups.push(bindWhileError({ anchor: __weAnchor37, signal: $scope.rail, render: __renderWhileError_0 }));
      const __ifAnchor38 = document.createComment("if");
      __e21.appendChild(__ifAnchor38);
      __cleanups.push(__wireBoundary_b12(__ifAnchor38));
      __e2.appendChild(__e21);
      __e1.appendChild(__e2);
      const __e102 = document.createElement("div");
      __e102.setAttribute("class", "skills-editor");
      const __ifAnchor103 = document.createComment("if");
      __e102.appendChild(__ifAnchor103);
      __cleanups.push(__wireBoundary_b33(__ifAnchor103));
      __e1.appendChild(__e102);
      __e0.appendChild(__e1);
      const __e350 = document.createElement("div");
      __e350.setAttribute("class", "skills-mobile");
      const __e351 = document.createElement("sui-icon");
      __e351.setAttribute("name", "monitor");
      __e351.setAttribute("size", "xl");
      __e351.setAttribute("tone", "muted");
      __e350.appendChild(__e351);
      const __e352 = document.createElement("span");
      const __t353 = document.createTextNode("Best experienced on desktop");
      __e352.appendChild(__t353);
      __e350.appendChild(__e352);
      __e0.appendChild(__e350);
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
      const __m = collectElementMarkers(root);
      if (__m["b0"])
        __cleanups.push(hydrateBindValue(__m["b0"], $scope.q));
      if (__m["b1"])
        __cleanups.push(hydrateOn(__m["b1"], "input", function() {
          const __h = (e) => $scope.q.set(e.target.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b2"]) {
        const __t = __m["b2"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b2(__anchor));
      }
      if (__m["b8"])
        __cleanups.push(hydrateWhileLoading(__m["b8"], "b8", () => $scope.rail(), __renderWhileLoading_0));
      if (__m["b10"])
        __cleanups.push(hydrateWhileError(__m["b10"], "b10", () => $scope.rail(), __renderWhileError_0));
      if (__m["b12"]) {
        const __t = __m["b12"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b12(__anchor));
      }
      if (__m["b33"]) {
        const __t = __m["b33"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b33(__anchor));
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
