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
      const __childScope14 = Object.create($scope);
      __childScope14.kc = kc;
      __childScope14.__i = __i;
      __childScope14.__v = signal2(0);
      return (($scope2) => {
        const __frag15 = document.createDocumentFragment();
        const __c16 = [];
        const __e17 = document.createElement("button");
        __e17.setAttribute("class", "se-pill");
        __c16.push(bindClass(__e17, "is-active", () => (void $scope2.__v(), $scope2.kind() === $scope2.kc.id)));
        __c16.push(on(__e17, "click", function() {
          const __h = () => $scope2.kind.set($scope2.kc.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t18 = document.createTextNode("");
        __e17.appendChild(__t18);
        __t18.textContent = $scope2.kc.label;
        const __e19 = document.createElement("span");
        __e19.setAttribute("style", "opacity:0.6;font-variant-numeric:tabular-nums");
        const __t20 = document.createTextNode("");
        __e19.appendChild(__t20);
        __t20.textContent = $scope2.kc.count;
        __e17.appendChild(__e19);
        __frag15.appendChild(__e17);
        return { nodes: Array.from(__frag15.childNodes), cleanup: () => {
          for (const c of __c16)
            try {
              c();
            } catch (e) {}
        }, update: (__new21, __newI22) => {
          $scope2.kc = __new21;
          $scope2.__i = __newI22;
          __t18.textContent = $scope2.kc.label;
          __t20.textContent = $scope2.kc.count;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope14);
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
        const __frag10 = document.createDocumentFragment();
        const __c11 = [];
        const __e12 = document.createElement("div");
        __e12.setAttribute("class", "se-filters");
        __e12.setAttribute("role", "group");
        __e12.setAttribute("aria-label", "Filter by kind");
        const __forAnchor13 = document.createComment("for");
        __e12.appendChild(__forAnchor13);
        __c11.push(__wireBoundary_b3(__forAnchor13));
        __frag10.appendChild(__e12);
        return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
          for (const c of __c11)
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
      const __childScope28 = Object.create($scope);
      __childScope28.n = n;
      __childScope28.__i = __i;
      return (($scope2) => {
        const __frag29 = document.createDocumentFragment();
        const __c30 = [];
        const __e31 = document.createElement("div");
        __e31.setAttribute("class", "se-skel-row");
        const __e32 = document.createElement("sui-skeleton");
        __e32.setAttribute("shape", "line");
        __e32.setAttribute("size", "md");
        __e31.appendChild(__e32);
        __frag29.appendChild(__e31);
        return { nodes: Array.from(__frag29.childNodes), cleanup: () => {
          for (const c of __c30)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope28);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag24 = document.createDocumentFragment();
    const __c25 = [];
    const __e26 = document.createElement("div");
    __e26.setAttribute("class", "se-skel");
    __e26.setAttribute("role", "status");
    __e26.setAttribute("aria-live", "polite");
    __e26.setAttribute("aria-busy", "true");
    __e26.setAttribute("aria-label", "Loading skills");
    const __forAnchor27 = document.createComment("for");
    __e26.appendChild(__forAnchor27);
    __c25.push(__wireBoundary_b9(__forAnchor27));
    __frag24.appendChild(__e26);
    return { nodes: Array.from(__frag24.childNodes), cleanup: () => {
      for (const c of __c25)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope34 = Object.create($scope);
    __childScope34.err = err;
    return (($scope2) => {
      const __frag35 = document.createDocumentFragment();
      const __c36 = [];
      const __e37 = document.createElement("sui-error-state");
      __e37.setAttribute("variant", "inline");
      __e37.setAttribute("size", "sm");
      __e37.setAttribute("title", "Couldn't load skills");
      __e37.setAttribute("role", "alert");
      __e37.setAttribute("aria-live", "assertive");
      const __t38 = document.createTextNode("");
      __e37.appendChild(__t38);
      __c36.push(bindText(__t38, () => $scope2.err.message));
      __frag35.appendChild(__e37);
      return { nodes: Array.from(__frag35.childNodes), cleanup: () => {
        for (const c of __c36)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope34);
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
      const __childScope51 = Object.create($scope);
      __childScope51.g = g;
      __childScope51.__i = __i;
      __childScope51.__v = signal2(0);
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
            const __childScope71 = Object.create($scope2);
            __childScope71.s = s;
            __childScope71.__i = __i2;
            __childScope71.__v = signal2(0);
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
                    const __frag84 = document.createDocumentFragment();
                    const __c85 = [];
                    const __e86 = document.createElement("span");
                    __e86.setAttribute("class", "se-dot vars");
                    __e86.setAttribute("title", "Variables");
                    __frag84.appendChild(__e86);
                    return { nodes: Array.from(__frag84.childNodes), cleanup: () => {
                      for (const c of __c85)
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
                    const __frag88 = document.createDocumentFragment();
                    const __c89 = [];
                    const __e90 = document.createElement("span");
                    __e90.setAttribute("class", "se-dot refs");
                    __e90.setAttribute("title", "Reference docs");
                    __frag88.appendChild(__e90);
                    return { nodes: Array.from(__frag88.childNodes), cleanup: () => {
                      for (const c of __c89)
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
                    const __frag92 = document.createDocumentFragment();
                    const __c93 = [];
                    const __e94 = document.createElement("span");
                    __e94.setAttribute("class", "se-dot evals");
                    __e94.setAttribute("title", "Evals");
                    __frag92.appendChild(__e94);
                    return { nodes: Array.from(__frag92.childNodes), cleanup: () => {
                      for (const c of __c93)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __frag72 = document.createDocumentFragment();
              const __c73 = [];
              const __e74 = document.createElement("div");
              __e74.setAttribute("role", "treeitem");
              __e74.setAttribute("class", "se-skill-row");
              __c73.push(bindAttr(__e74, "aria-selected", () => (void $scope3.__v(), $scope3.skillId() === $scope3.s.id ? "true" : "false")));
              const __e75 = document.createElement("button");
              __e75.setAttribute("class", "se-skill");
              __c73.push(bindClass(__e75, "is-active", () => (void $scope3.__v(), $scope3.skillId() === $scope3.s.id)));
              __c73.push(on(__e75, "click", function() {
                const __h = () => $scope3.skillId.set($scope3.s.id);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              {
                const __av76 = $scope3.s.description;
                if (__av76 == null || __av76 === false || __av76 === "")
                  __e75.removeAttribute("title");
                else
                  __e75.setAttribute("title", __av76 === true ? "" : String(__av76));
              }
              const __e77 = document.createElement("span");
              __e77.setAttribute("class", "se-chevron");
              __e77.setAttribute("aria-hidden", "true");
              const __e78 = document.createElement("sui-icon");
              __e78.setAttribute("name", "chevron-right");
              __e78.setAttribute("size", "xs");
              __e77.appendChild(__e78);
              __e75.appendChild(__e77);
              const __e79 = document.createElement("sui-icon");
              __e79.setAttribute("name", "zap");
              __e79.setAttribute("size", "xs");
              __e79.setAttribute("tone", "primary");
              __e75.appendChild(__e79);
              const __e80 = document.createElement("span");
              __e80.setAttribute("class", "se-name");
              const __t81 = document.createTextNode("");
              __e80.appendChild(__t81);
              __t81.textContent = $scope3.s.name;
              __e75.appendChild(__e80);
              const __e82 = document.createElement("span");
              __e82.setAttribute("class", "se-dots");
              const __ifAnchor83 = document.createComment("if");
              __e82.appendChild(__ifAnchor83);
              __c73.push(__wireBoundary_b26(__ifAnchor83));
              const __ifAnchor87 = document.createComment("if");
              __e82.appendChild(__ifAnchor87);
              __c73.push(__wireBoundary_b27(__ifAnchor87));
              const __ifAnchor91 = document.createComment("if");
              __e82.appendChild(__ifAnchor91);
              __c73.push(__wireBoundary_b28(__ifAnchor91));
              __e75.appendChild(__e82);
              __e74.appendChild(__e75);
              const __e95 = document.createElement("button");
              __e95.setAttribute("class", "se-invoke");
              {
                const __av96 = "Invoke " + $scope3.s.name;
                if (__av96 == null || __av96 === false || __av96 === "")
                  __e95.removeAttribute("aria-label");
                else
                  __e95.setAttribute("aria-label", __av96 === true ? "" : String(__av96));
              }
              __c73.push(bindProp(__e95, "disabled", () => (void $scope3.__v(), $scope3.invokeBusy() === $scope3.s.id)));
              __c73.push(on(__e95, "click", function() {
                const __h = () => $scope3.invoke($scope3.s.id);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              const __e97 = document.createElement("sui-icon");
              __e97.setAttribute("name", "play");
              __e97.setAttribute("size", "xs");
              __e95.appendChild(__e97);
              const __e98 = document.createElement("span");
              const __t99 = document.createTextNode("");
              __e98.appendChild(__t99);
              __c73.push(bindText(__t99, () => (void $scope3.__v(), $scope3.invokeBusy() === $scope3.s.id ? "Invoking…" : "Invoke")));
              __e95.appendChild(__e98);
              __e74.appendChild(__e95);
              __frag72.appendChild(__e74);
              return { nodes: Array.from(__frag72.childNodes), cleanup: () => {
                for (const c of __c73)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new100, __newI101) => {
                $scope3.s = __new100;
                $scope3.__i = __newI101;
                {
                  const __av76 = $scope3.s.description;
                  if (__av76 == null || __av76 === false || __av76 === "")
                    __e75.removeAttribute("title");
                  else
                    __e75.setAttribute("title", __av76 === true ? "" : String(__av76));
                }
                __t81.textContent = $scope3.s.name;
                {
                  const __av96 = "Invoke " + $scope3.s.name;
                  if (__av96 == null || __av96 === false || __av96 === "")
                    __e95.removeAttribute("aria-label");
                  else
                    __e95.setAttribute("aria-label", __av96 === true ? "" : String(__av96));
                }
                $scope3.__v.set((c) => c + 1);
              } };
            })(__childScope71);
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
              const __frag64 = document.createDocumentFragment();
              const __c65 = [];
              const __e66 = document.createElement("div");
              __e66.setAttribute("class", "se-empty");
              const __t67 = document.createTextNode("");
              __e66.appendChild(__t67);
              __c65.push(bindText(__t67, () => (void $scope2.__v(), $scope2.q() ? "No matches." : "No skills yet.")));
              __frag64.appendChild(__e66);
              return { nodes: Array.from(__frag64.childNodes), cleanup: () => {
                for (const c of __c65)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag68 = document.createDocumentFragment();
              const __c69 = [];
              const __forAnchor70 = document.createComment("for");
              __frag68.appendChild(__forAnchor70);
              __c69.push(__wireBoundary_b20(__forAnchor70));
              return { nodes: Array.from(__frag68.childNodes), cleanup: () => {
                for (const c of __c69)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag52 = document.createDocumentFragment();
        const __c53 = [];
        const __e54 = document.createElement("div");
        __e54.setAttribute("class", "se-section");
        const __e55 = document.createElement("div");
        __e55.setAttribute("class", "se-section-head");
        const __e56 = document.createElement("span");
        __e56.setAttribute("class", "se-section-label");
        const __e57 = document.createElement("sui-icon");
        __e57.setAttribute("name", "folder");
        __e57.setAttribute("size", "xs");
        __e57.setAttribute("tone", "muted");
        __e56.appendChild(__e57);
        const __t58 = document.createTextNode("");
        __e56.appendChild(__t58);
        __t58.textContent = $scope2.g.source === "PROJECT SKILLS" ? "Project Skills" : $scope2.g.source === "USER SKILLS" ? "Library" : $scope2.g.source;
        const __e59 = document.createElement("span");
        __e59.setAttribute("class", "count");
        const __t60 = document.createTextNode("(");
        __e59.appendChild(__t60);
        const __t61 = document.createTextNode("");
        __e59.appendChild(__t61);
        __t61.textContent = $scope2.g.skills.length;
        const __t62 = document.createTextNode(")");
        __e59.appendChild(__t62);
        __e56.appendChild(__e59);
        __e55.appendChild(__e56);
        __e54.appendChild(__e55);
        const __ifAnchor63 = document.createComment("if");
        __e54.appendChild(__ifAnchor63);
        __c53.push(__wireBoundary_b18(__ifAnchor63));
        __frag52.appendChild(__e54);
        return { nodes: Array.from(__frag52.childNodes), cleanup: () => {
          for (const c of __c53)
            try {
              c();
            } catch (e) {}
        }, update: (__new102, __newI103) => {
          $scope2.g = __new102;
          $scope2.__i = __newI103;
          __t58.textContent = $scope2.g.source === "PROJECT SKILLS" ? "Project Skills" : $scope2.g.source === "USER SKILLS" ? "Library" : $scope2.g.source;
          __t61.textContent = $scope2.g.skills.length;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope51);
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
        const __frag44 = document.createDocumentFragment();
        const __c45 = [];
        const __e46 = document.createElement("div");
        __e46.setAttribute("class", "se-empty");
        const __t47 = document.createTextNode("");
        __e46.appendChild(__t47);
        __c45.push(bindText(__t47, () => $scope.q() ? 'No skills match "' + $scope.q() + '".' : "No skills discovered."));
        __frag44.appendChild(__e46);
        return { nodes: Array.from(__frag44.childNodes), cleanup: () => {
          for (const c of __c45)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag48 = document.createDocumentFragment();
        const __c49 = [];
        const __forAnchor50 = document.createComment("for");
        __frag48.appendChild(__forAnchor50);
        __c49.push(__wireBoundary_b15(__forAnchor50));
        return { nodes: Array.from(__frag48.childNodes), cleanup: () => {
          for (const c of __c49)
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
        const __frag41 = document.createDocumentFragment();
        const __c42 = [];
        const __ifAnchor43 = document.createComment("if");
        __frag41.appendChild(__ifAnchor43);
        __c42.push(__wireBoundary_b13(__ifAnchor43));
        return { nodes: Array.from(__frag41.childNodes), cleanup: () => {
          for (const c of __c42)
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
        const __frag127 = document.createDocumentFragment();
        const __c128 = [];
        const __e129 = document.createElement("p");
        __e129.setAttribute("class", "sd-description");
        const __t130 = document.createTextNode("");
        __e129.appendChild(__t130);
        __c128.push(bindText(__t130, () => $scope.skill().value.description));
        __frag127.appendChild(__e129);
        return { nodes: Array.from(__frag127.childNodes), cleanup: () => {
          for (const c of __c128)
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
        const __frag136 = document.createDocumentFragment();
        const __c137 = [];
        const __t138 = document.createTextNode(`
                Invoked · run `);
        __frag136.appendChild(__t138);
        const __e139 = document.createElement("code");
        const __t140 = document.createTextNode("");
        __e139.appendChild(__t140);
        __c137.push(bindText(__t140, () => $scope.invokeResult().run_id));
        __frag136.appendChild(__e139);
        const __t141 = document.createTextNode(" · session ");
        __frag136.appendChild(__t141);
        const __e142 = document.createElement("code");
        const __t143 = document.createTextNode("");
        __e142.appendChild(__t143);
        __c137.push(bindText(__t143, () => $scope.invokeResult().session_id));
        __frag136.appendChild(__e142);
        const __e144 = document.createElement("button");
        __e144.setAttribute("class", "sd-invoke-dismiss");
        __e144.setAttribute("aria-label", "Dismiss");
        __c137.push(on(__e144, "click", function() {
          const __h = () => $scope.invokeResult.set(null);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t145 = document.createTextNode("×");
        __e144.appendChild(__t145);
        __frag136.appendChild(__e144);
        return { nodes: Array.from(__frag136.childNodes), cleanup: () => {
          for (const c of __c137)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag146 = document.createDocumentFragment();
        const __c147 = [];
        const __t148 = document.createTextNode("");
        __frag146.appendChild(__t148);
        __c147.push(bindText(__t148, () => $scope.invokeResult().error));
        const __e149 = document.createElement("button");
        __e149.setAttribute("class", "sd-invoke-dismiss");
        __e149.setAttribute("aria-label", "Dismiss");
        __c147.push(on(__e149, "click", function() {
          const __h = () => $scope.invokeResult.set(null);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t150 = document.createTextNode("×");
        __e149.appendChild(__t150);
        __frag146.appendChild(__e149);
        return { nodes: Array.from(__frag146.childNodes), cleanup: () => {
          for (const c of __c147)
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
        const __frag132 = document.createDocumentFragment();
        const __c133 = [];
        const __e134 = document.createElement("sui-alert");
        __c133.push(bindAttr(__e134, "tone", () => $scope.invokeResult().ok ? "success" : "warning"));
        __e134.setAttribute("class", "sd-invoke-result");
        const __ifAnchor135 = document.createComment("if");
        __e134.appendChild(__ifAnchor135);
        __c133.push(__wireBoundary_b49(__ifAnchor135));
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
  const __wireBoundary_b64 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.viewRaw())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag168 = document.createDocumentFragment();
        const __c169 = [];
        const __e170 = document.createElement("pre");
        __e170.setAttribute("class", "sd-raw");
        const __e171 = document.createElement("code");
        const __t172 = document.createTextNode("");
        __e171.appendChild(__t172);
        __c169.push(bindText(__t172, () => $scope.skill().value.body));
        __e170.appendChild(__e171);
        __frag168.appendChild(__e170);
        return { nodes: Array.from(__frag168.childNodes), cleanup: () => {
          for (const c of __c169)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag173 = document.createDocumentFragment();
        const __c174 = [];
        const __e175 = document.createElement("sui-markdown-editor");
        __c174.push(bindAttr(__e175, "value", () => $scope.skill().value.body));
        __frag173.appendChild(__e175);
        return { nodes: Array.from(__frag173.childNodes), cleanup: () => {
          for (const c of __c174)
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
        const __frag165 = document.createDocumentFragment();
        const __c166 = [];
        const __ifAnchor167 = document.createComment("if");
        __frag165.appendChild(__ifAnchor167);
        __c166.push(__wireBoundary_b64(__ifAnchor167));
        return { nodes: Array.from(__frag165.childNodes), cleanup: () => {
          for (const c of __c166)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag176 = document.createDocumentFragment();
        const __c177 = [];
        const __e178 = document.createElement("div");
        __e178.setAttribute("class", "se-empty-state");
        __e178.setAttribute("style", "padding:40px 0");
        const __e179 = document.createElement("sui-icon");
        __e179.setAttribute("name", "file");
        __e179.setAttribute("size", "xl");
        __e179.setAttribute("tone", "muted");
        __e178.appendChild(__e179);
        const __e180 = document.createElement("span");
        __e180.setAttribute("class", "se-empty-desc");
        const __t181 = document.createTextNode("This skill has no SKILL.md content yet.");
        __e180.appendChild(__t181);
        __e178.appendChild(__e180);
        __frag176.appendChild(__e178);
        return { nodes: Array.from(__frag176.childNodes), cleanup: () => {
          for (const c of __c177)
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
        const __frag200 = document.createDocumentFragment();
        const __c201 = [];
        const __e202 = document.createElement("dt");
        const __t203 = document.createTextNode("description");
        __e202.appendChild(__t203);
        __frag200.appendChild(__e202);
        const __e204 = document.createElement("dd");
        const __t205 = document.createTextNode("");
        __e204.appendChild(__t205);
        __c201.push(bindText(__t205, () => $scope.skill().value.description));
        __frag200.appendChild(__e204);
        return { nodes: Array.from(__frag200.childNodes), cleanup: () => {
          for (const c of __c201)
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
      const __childScope214 = Object.create($scope);
      __childScope214.t = t;
      __childScope214.__i = __i;
      __childScope214.__v = signal2(0);
      return (($scope2) => {
        const __frag215 = document.createDocumentFragment();
        const __c216 = [];
        const __e217 = document.createElement("sui-chip");
        __e217.setAttribute("variant", "soft");
        __e217.setAttribute("size", "sm");
        __e217.setAttribute("tone", "muted");
        const __t218 = document.createTextNode("");
        __e217.appendChild(__t218);
        __t218.textContent = $scope2.t;
        __frag215.appendChild(__e217);
        return { nodes: Array.from(__frag215.childNodes), cleanup: () => {
          for (const c of __c216)
            try {
              c();
            } catch (e) {}
        }, update: (__new219, __newI220) => {
          $scope2.t = __new219;
          $scope2.__i = __newI220;
          __t218.textContent = $scope2.t;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope214);
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
        const __frag207 = document.createDocumentFragment();
        const __c208 = [];
        const __e209 = document.createElement("div");
        __e209.setAttribute("class", "sd-meta-section");
        const __e210 = document.createElement("div");
        __e210.setAttribute("class", "sd-meta-label");
        const __t211 = document.createTextNode("Tags");
        __e210.appendChild(__t211);
        __e209.appendChild(__e210);
        const __e212 = document.createElement("div");
        __e212.setAttribute("class", "sd-meta-chips");
        const __forAnchor213 = document.createComment("for");
        __e212.appendChild(__forAnchor213);
        __c208.push(__wireBoundary_b73(__forAnchor213));
        __e209.appendChild(__e212);
        __frag207.appendChild(__e209);
        return { nodes: Array.from(__frag207.childNodes), cleanup: () => {
          for (const c of __c208)
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
      const __childScope229 = Object.create($scope);
      __childScope229.a = a;
      __childScope229.__i = __i;
      __childScope229.__v = signal2(0);
      return (($scope2) => {
        const __frag230 = document.createDocumentFragment();
        const __c231 = [];
        const __e232 = document.createElement("sui-chip");
        __e232.setAttribute("variant", "soft");
        __e232.setAttribute("size", "sm");
        __e232.setAttribute("tone", "muted");
        const __t233 = document.createTextNode("");
        __e232.appendChild(__t233);
        __t233.textContent = $scope2.a;
        __frag230.appendChild(__e232);
        return { nodes: Array.from(__frag230.childNodes), cleanup: () => {
          for (const c of __c231)
            try {
              c();
            } catch (e) {}
        }, update: (__new234, __newI235) => {
          $scope2.a = __new234;
          $scope2.__i = __newI235;
          __t233.textContent = $scope2.a;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope229);
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
        const __frag222 = document.createDocumentFragment();
        const __c223 = [];
        const __e224 = document.createElement("div");
        __e224.setAttribute("class", "sd-meta-section");
        const __e225 = document.createElement("div");
        __e225.setAttribute("class", "sd-meta-label");
        const __t226 = document.createTextNode("Authors");
        __e225.appendChild(__t226);
        __e224.appendChild(__e225);
        const __e227 = document.createElement("div");
        __e227.setAttribute("class", "sd-meta-chips");
        const __forAnchor228 = document.createComment("for");
        __e227.appendChild(__forAnchor228);
        __c223.push(__wireBoundary_b76(__forAnchor228));
        __e224.appendChild(__e227);
        __frag222.appendChild(__e224);
        return { nodes: Array.from(__frag222.childNodes), cleanup: () => {
          for (const c of __c223)
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
      const __childScope246 = Object.create($scope);
      __childScope246.v = v;
      __childScope246.__i = __i;
      __childScope246.__v = signal2(0);
      return (($scope2) => {
        const __frag247 = document.createDocumentFragment();
        const __c248 = [];
        const __e249 = document.createElement("li");
        const __e250 = document.createElement("code");
        const __t251 = document.createTextNode("");
        __e250.appendChild(__t251);
        __t251.textContent = $scope2.v;
        __e249.appendChild(__e250);
        __frag247.appendChild(__e249);
        return { nodes: Array.from(__frag247.childNodes), cleanup: () => {
          for (const c of __c248)
            try {
              c();
            } catch (e) {}
        }, update: (__new252, __newI253) => {
          $scope2.v = __new252;
          $scope2.__i = __newI253;
          __t251.textContent = $scope2.v;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope246);
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
        const __frag242 = document.createDocumentFragment();
        const __c243 = [];
        const __e244 = document.createElement("ul");
        __e244.setAttribute("class", "sd-meta-vars");
        const __forAnchor245 = document.createComment("for");
        __e244.appendChild(__forAnchor245);
        __c243.push(__wireBoundary_b80(__forAnchor245));
        __frag242.appendChild(__e244);
        return { nodes: Array.from(__frag242.childNodes), cleanup: () => {
          for (const c of __c243)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag254 = document.createDocumentFragment();
        const __c255 = [];
        const __e256 = document.createElement("p");
        __e256.setAttribute("class", "sd-meta-empty");
        const __t257 = document.createTextNode("No variables defined.");
        __e256.appendChild(__t257);
        __frag254.appendChild(__e256);
        return { nodes: Array.from(__frag254.childNodes), cleanup: () => {
          for (const c of __c255)
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
      const __childScope265 = Object.create($scope);
      __childScope265.f = f;
      __childScope265.__i = __i;
      __childScope265.__v = signal2(0);
      return (($scope2) => {
        const __frag266 = document.createDocumentFragment();
        const __c267 = [];
        const __e268 = document.createElement("li");
        const __e269 = document.createElement("code");
        const __t270 = document.createTextNode("");
        __e269.appendChild(__t270);
        __t270.textContent = $scope2.f;
        __e268.appendChild(__e269);
        __frag266.appendChild(__e268);
        return { nodes: Array.from(__frag266.childNodes), cleanup: () => {
          for (const c of __c267)
            try {
              c();
            } catch (e) {}
        }, update: (__new271, __newI272) => {
          $scope2.f = __new271;
          $scope2.__i = __newI272;
          __t270.textContent = $scope2.f;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope265);
    }
  });
  const __renderWhileLoading_1 = () => {
    const __frag276 = document.createDocumentFragment();
    const __c277 = [];
    const __e278 = document.createElement("div");
    __e278.setAttribute("class", "se-empty-state");
    __e278.setAttribute("role", "status");
    __e278.setAttribute("aria-live", "polite");
    __e278.setAttribute("aria-busy", "true");
    const __e279 = document.createElement("sui-spinner");
    __e279.setAttribute("size", "md");
    __e278.appendChild(__e279);
    const __e280 = document.createElement("span");
    __e280.setAttribute("class", "se-empty-desc");
    const __t281 = document.createTextNode("Loading runs…");
    __e280.appendChild(__t281);
    __e278.appendChild(__e280);
    __frag276.appendChild(__e278);
    return { nodes: Array.from(__frag276.childNodes), cleanup: () => {
      for (const c of __c277)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_1 = (err) => {
    const __childScope283 = Object.create($scope);
    __childScope283.err = err;
    return (($scope2) => {
      const __frag284 = document.createDocumentFragment();
      const __c285 = [];
      const __e286 = document.createElement("sui-error-state");
      __e286.setAttribute("variant", "inline");
      __e286.setAttribute("size", "sm");
      __e286.setAttribute("title", "Couldn't load runs");
      __e286.setAttribute("role", "alert");
      __e286.setAttribute("aria-live", "assertive");
      const __t287 = document.createTextNode("");
      __e286.appendChild(__t287);
      __c285.push(bindText(__t287, () => $scope2.err.message));
      __frag284.appendChild(__e286);
      return { nodes: Array.from(__frag284.childNodes), cleanup: () => {
        for (const c of __c285)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope283);
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
      const __childScope308 = Object.create($scope);
      __childScope308.r = r;
      __childScope308.__i = __i;
      __childScope308.__v = signal2(0);
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
              const __frag319 = document.createDocumentFragment();
              const __c320 = [];
              const __e321 = document.createElement("span");
              __e321.setAttribute("class", "sd-run-session");
              {
                const __av322 = "Session " + $scope2.r.session_id;
                if (__av322 == null || __av322 === false || __av322 === "")
                  __e321.removeAttribute("title");
                else
                  __e321.setAttribute("title", __av322 === true ? "" : String(__av322));
              }
              const __e323 = document.createElement("sui-icon");
              __e323.setAttribute("name", "link");
              __e323.setAttribute("size", "xs");
              __e323.setAttribute("tone", "muted");
              __e321.appendChild(__e323);
              const __t324 = document.createTextNode("");
              __e321.appendChild(__t324);
              __c320.push(bindText(__t324, () => (void $scope2.__v(), $scope2.r.session_id.slice(0, 8))));
              __frag319.appendChild(__e321);
              return { nodes: Array.from(__frag319.childNodes), cleanup: () => {
                for (const c of __c320)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag309 = document.createDocumentFragment();
        const __c310 = [];
        const __e311 = document.createElement("li");
        __e311.setAttribute("class", "sd-run-row");
        const __e312 = document.createElement("sui-icon");
        __e312.setAttribute("name", "play");
        __e312.setAttribute("size", "xs");
        __e312.setAttribute("tone", "primary");
        __e311.appendChild(__e312);
        const __e313 = document.createElement("span");
        __e313.setAttribute("class", "sd-run-time");
        {
          const __av314 = $scope2.r.timestamp;
          if (__av314 == null || __av314 === false || __av314 === "")
            __e313.removeAttribute("title");
          else
            __e313.setAttribute("title", __av314 === true ? "" : String(__av314));
        }
        const __t315 = document.createTextNode("");
        __e313.appendChild(__t315);
        __t315.textContent = $scope2.r.timestamp;
        __e311.appendChild(__e313);
        const __e316 = document.createElement("span");
        __e316.setAttribute("class", "sd-run-actor");
        const __t317 = document.createTextNode("");
        __e316.appendChild(__t317);
        __t317.textContent = $scope2.r.actor;
        __e311.appendChild(__e316);
        const __ifAnchor318 = document.createComment("if");
        __e311.appendChild(__ifAnchor318);
        __c310.push(__wireBoundary_b94(__ifAnchor318));
        const __e325 = document.createElement("code");
        __e325.setAttribute("class", "sd-run-id");
        {
          const __av326 = $scope2.r.id;
          if (__av326 == null || __av326 === false || __av326 === "")
            __e325.removeAttribute("title");
          else
            __e325.setAttribute("title", __av326 === true ? "" : String(__av326));
        }
        const __t327 = document.createTextNode("");
        __e325.appendChild(__t327);
        __c310.push(bindText(__t327, () => (void $scope2.__v(), $scope2.r.id.slice(0, 12))));
        __e311.appendChild(__e325);
        __frag309.appendChild(__e311);
        return { nodes: Array.from(__frag309.childNodes), cleanup: () => {
          for (const c of __c310)
            try {
              c();
            } catch (e) {}
        }, update: (__new328, __newI329) => {
          $scope2.r = __new328;
          $scope2.__i = __newI329;
          {
            const __av314 = $scope2.r.timestamp;
            if (__av314 == null || __av314 === false || __av314 === "")
              __e313.removeAttribute("title");
            else
              __e313.setAttribute("title", __av314 === true ? "" : String(__av314));
          }
          __t315.textContent = $scope2.r.timestamp;
          __t317.textContent = $scope2.r.actor;
          {
            const __av326 = $scope2.r.id;
            if (__av326 == null || __av326 === false || __av326 === "")
              __e325.removeAttribute("title");
            else
              __e325.setAttribute("title", __av326 === true ? "" : String(__av326));
          }
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope308);
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
        const __frag293 = document.createDocumentFragment();
        const __c294 = [];
        const __e295 = document.createElement("div");
        __e295.setAttribute("class", "se-empty-state");
        __e295.setAttribute("style", "padding:40px 0");
        const __e296 = document.createElement("sui-icon");
        __e296.setAttribute("name", "zap");
        __e296.setAttribute("size", "xl");
        __e296.setAttribute("tone", "muted");
        __e295.appendChild(__e296);
        const __e297 = document.createElement("span");
        __e297.setAttribute("class", "se-empty-title");
        const __t298 = document.createTextNode("No invocations yet");
        __e297.appendChild(__t298);
        __e295.appendChild(__e297);
        const __e299 = document.createElement("span");
        __e299.setAttribute("class", "se-empty-desc");
        const __t300 = document.createTextNode("Click ");
        __e299.appendChild(__t300);
        const __e301 = document.createElement("b");
        const __t302 = document.createTextNode("Invoke");
        __e301.appendChild(__t302);
        __e299.appendChild(__e301);
        const __t303 = document.createTextNode(" to start a session attached to this skill.");
        __e299.appendChild(__t303);
        __e295.appendChild(__e299);
        __frag293.appendChild(__e295);
        return { nodes: Array.from(__frag293.childNodes), cleanup: () => {
          for (const c of __c294)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag304 = document.createDocumentFragment();
        const __c305 = [];
        const __e306 = document.createElement("ul");
        __e306.setAttribute("class", "sd-runs-list");
        const __forAnchor307 = document.createComment("for");
        __e306.appendChild(__forAnchor307);
        __c305.push(__wireBoundary_b90(__forAnchor307));
        __frag304.appendChild(__e306);
        return { nodes: Array.from(__frag304.childNodes), cleanup: () => {
          for (const c of __c305)
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
        const __frag290 = document.createDocumentFragment();
        const __c291 = [];
        const __ifAnchor292 = document.createComment("if");
        __frag290.appendChild(__ifAnchor292);
        __c291.push(__wireBoundary_b89(__ifAnchor292));
        return { nodes: Array.from(__frag290.childNodes), cleanup: () => {
          for (const c of __c291)
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
        const __frag161 = document.createDocumentFragment();
        const __c162 = [];
        const __e163 = document.createElement("div");
        __e163.setAttribute("class", "se-editor-content");
        const __ifAnchor164 = document.createComment("if");
        __e163.appendChild(__ifAnchor164);
        __c162.push(__wireBoundary_b63(__ifAnchor164));
        __frag161.appendChild(__e163);
        return { nodes: Array.from(__frag161.childNodes), cleanup: () => {
          for (const c of __c162)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag182 = document.createDocumentFragment();
        const __c183 = [];
        const __e184 = document.createElement("div");
        __e184.setAttribute("class", "se-editor-content sd-meta");
        const __e185 = document.createElement("dl");
        __e185.setAttribute("class", "sd-meta-list");
        const __e186 = document.createElement("dt");
        const __t187 = document.createTextNode("id");
        __e186.appendChild(__t187);
        __e185.appendChild(__e186);
        const __e188 = document.createElement("dd");
        const __e189 = document.createElement("code");
        const __t190 = document.createTextNode("");
        __e189.appendChild(__t190);
        __c183.push(bindText(__t190, () => $scope.skill().value.id));
        __e188.appendChild(__e189);
        __e185.appendChild(__e188);
        const __e191 = document.createElement("dt");
        const __t192 = document.createTextNode("kind");
        __e191.appendChild(__t192);
        __e185.appendChild(__e191);
        const __e193 = document.createElement("dd");
        const __t194 = document.createTextNode("");
        __e193.appendChild(__t194);
        __c183.push(bindText(__t194, () => $scope.skill().value.kind));
        __e185.appendChild(__e193);
        const __e195 = document.createElement("dt");
        const __t196 = document.createTextNode("source");
        __e195.appendChild(__t196);
        __e185.appendChild(__e195);
        const __e197 = document.createElement("dd");
        const __t198 = document.createTextNode("");
        __e197.appendChild(__t198);
        __c183.push(bindText(__t198, () => $scope.skill().value.source));
        __e185.appendChild(__e197);
        const __ifAnchor199 = document.createComment("if");
        __e185.appendChild(__ifAnchor199);
        __c183.push(__wireBoundary_b70(__ifAnchor199));
        __e184.appendChild(__e185);
        const __ifAnchor206 = document.createComment("if");
        __e184.appendChild(__ifAnchor206);
        __c183.push(__wireBoundary_b72(__ifAnchor206));
        const __ifAnchor221 = document.createComment("if");
        __e184.appendChild(__ifAnchor221);
        __c183.push(__wireBoundary_b75(__ifAnchor221));
        const __e236 = document.createElement("div");
        __e236.setAttribute("class", "sd-meta-section");
        const __e237 = document.createElement("div");
        __e237.setAttribute("class", "sd-meta-label");
        const __t238 = document.createTextNode("Variables (");
        __e237.appendChild(__t238);
        const __t239 = document.createTextNode("");
        __e237.appendChild(__t239);
        __c183.push(bindText(__t239, () => $scope.skill().value.variables?.length ?? 0));
        const __t240 = document.createTextNode(")");
        __e237.appendChild(__t240);
        __e236.appendChild(__e237);
        const __ifAnchor241 = document.createComment("if");
        __e236.appendChild(__ifAnchor241);
        __c183.push(__wireBoundary_b79(__ifAnchor241));
        __e184.appendChild(__e236);
        const __e258 = document.createElement("div");
        __e258.setAttribute("class", "sd-meta-section");
        const __e259 = document.createElement("div");
        __e259.setAttribute("class", "sd-meta-label");
        const __t260 = document.createTextNode("Files (");
        __e259.appendChild(__t260);
        const __t261 = document.createTextNode("");
        __e259.appendChild(__t261);
        __c183.push(bindText(__t261, () => $scope.skill().value.files?.length ?? 0));
        const __t262 = document.createTextNode(")");
        __e259.appendChild(__t262);
        __e258.appendChild(__e259);
        const __e263 = document.createElement("ul");
        __e263.setAttribute("class", "sd-meta-files");
        const __forAnchor264 = document.createComment("for");
        __e263.appendChild(__forAnchor264);
        __c183.push(__wireBoundary_b83(__forAnchor264));
        __e258.appendChild(__e263);
        __e184.appendChild(__e258);
        __frag182.appendChild(__e184);
        return { nodes: Array.from(__frag182.childNodes), cleanup: () => {
          for (const c of __c183)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag273 = document.createDocumentFragment();
        const __c274 = [];
        const __e275 = document.createElement("div");
        __e275.setAttribute("class", "se-editor-content sd-runs");
        const __wlAnchor282 = document.createComment("while-loading");
        __e275.appendChild(__wlAnchor282);
        __c274.push(bindWhileLoading({ anchor: __wlAnchor282, signal: $scope.runs, render: __renderWhileLoading_1 }));
        const __weAnchor288 = document.createComment("while-error");
        __e275.appendChild(__weAnchor288);
        __c274.push(bindWhileError({ anchor: __weAnchor288, signal: $scope.runs, render: __renderWhileError_1 }));
        const __ifAnchor289 = document.createComment("if");
        __e275.appendChild(__ifAnchor289);
        __c274.push(__wireBoundary_b88(__ifAnchor289));
        __frag273.appendChild(__e275);
        return { nodes: Array.from(__frag273.childNodes), cleanup: () => {
          for (const c of __c274)
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
        const __frag106 = document.createDocumentFragment();
        const __c107 = [];
        const __e108 = document.createElement("header");
        __e108.setAttribute("class", "sd-header");
        const __e109 = document.createElement("div");
        __e109.setAttribute("class", "sd-title-row");
        const __e110 = document.createElement("div");
        __e110.setAttribute("class", "sd-title-main");
        const __e111 = document.createElement("h1");
        __e111.setAttribute("class", "sd-name");
        const __t112 = document.createTextNode("");
        __e111.appendChild(__t112);
        __c107.push(bindText(__t112, () => $scope.skill().value.name));
        __e110.appendChild(__e111);
        const __e113 = document.createElement("sui-chip");
        __e113.setAttribute("variant", "soft");
        __e113.setAttribute("size", "sm");
        __c107.push(bindAttr(__e113, "tone", () => $scope.skill().value.kind === "smart" ? "primary" : $scope.skill().value.kind === "docs" ? "info" : $scope.skill().value.kind === "operator" ? "warning" : "muted"));
        const __t114 = document.createTextNode("");
        __e113.appendChild(__t114);
        __c107.push(bindText(__t114, () => $scope.skill().value.kind));
        __e110.appendChild(__e113);
        const __e115 = document.createElement("span");
        __e115.setAttribute("class", "sd-source-label");
        const __t116 = document.createTextNode("");
        __e115.appendChild(__t116);
        __c107.push(bindText(__t116, () => $scope.skill().value.source));
        __e110.appendChild(__e115);
        __e109.appendChild(__e110);
        const __e117 = document.createElement("div");
        __e117.setAttribute("class", "sd-title-actions");
        const __e118 = document.createElement("button");
        __e118.setAttribute("class", "sd-action sd-source-btn");
        __c107.push(bindClass(__e118, "is-active", () => $scope.viewRaw()));
        __c107.push(bindAttr(__e118, "aria-pressed", () => $scope.viewRaw() ? "true" : "false"));
        __c107.push(on(__e118, "click", function() {
          const __h = $scope.openSource;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e119 = document.createElement("sui-icon");
        __e119.setAttribute("name", "file-code");
        __e119.setAttribute("size", "xs");
        __e118.appendChild(__e119);
        const __e120 = document.createElement("span");
        const __t121 = document.createTextNode("");
        __e120.appendChild(__t121);
        __c107.push(bindText(__t121, () => $scope.viewRaw() ? "Rendered" : "Source"));
        __e118.appendChild(__e120);
        __e117.appendChild(__e118);
        const __e122 = document.createElement("button");
        __e122.setAttribute("class", "sd-action sd-invoke-btn");
        __c107.push(bindProp(__e122, "disabled", () => $scope.invokeBusy() === $scope.skill().value.id));
        __c107.push(on(__e122, "click", function() {
          const __h = () => $scope.invoke($scope.skill().value.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e123 = document.createElement("sui-icon");
        __e123.setAttribute("name", "play");
        __e123.setAttribute("size", "xs");
        __e122.appendChild(__e123);
        const __e124 = document.createElement("span");
        const __t125 = document.createTextNode("");
        __e124.appendChild(__t125);
        __c107.push(bindText(__t125, () => $scope.invokeBusy() === $scope.skill().value.id ? "Invoking…" : "Invoke"));
        __e122.appendChild(__e124);
        __e117.appendChild(__e122);
        __e109.appendChild(__e117);
        __e108.appendChild(__e109);
        const __ifAnchor126 = document.createComment("if");
        __e108.appendChild(__ifAnchor126);
        __c107.push(__wireBoundary_b45(__ifAnchor126));
        const __ifAnchor131 = document.createComment("if");
        __e108.appendChild(__ifAnchor131);
        __c107.push(__wireBoundary_b47(__ifAnchor131));
        __frag106.appendChild(__e108);
        const __e151 = document.createElement("div");
        __e151.setAttribute("class", "sd-tabs-row");
        const __e152 = document.createElement("sui-tabs");
        __e152.setAttribute("variant", "underline");
        const __e153 = document.createElement("sui-tab");
        __e153.setAttribute("name", "content");
        __c107.push(bindAttr(__e153, "active", () => $scope.tab() === "content"));
        __c107.push(on(__e153, "click", function() {
          const __h = () => $scope.tab.set("content");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t154 = document.createTextNode("SKILL.md");
        __e153.appendChild(__t154);
        __e152.appendChild(__e153);
        const __e155 = document.createElement("sui-tab");
        __e155.setAttribute("name", "metadata");
        __c107.push(bindAttr(__e155, "active", () => $scope.tab() === "metadata"));
        __c107.push(on(__e155, "click", function() {
          const __h = () => $scope.tab.set("metadata");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t156 = document.createTextNode("Metadata");
        __e155.appendChild(__t156);
        __e152.appendChild(__e155);
        const __e157 = document.createElement("sui-tab");
        __e157.setAttribute("name", "runs");
        __c107.push(bindAttr(__e157, "active", () => $scope.tab() === "runs"));
        __c107.push(bindAttr(__e157, "count", () => $scope.runs().kind === "ok" ? $scope.runs().value.rows.length + "" : ""));
        __c107.push(on(__e157, "click", function() {
          const __h = () => $scope.tab.set("runs");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t158 = document.createTextNode("Runs");
        __e157.appendChild(__t158);
        __e152.appendChild(__e157);
        __e151.appendChild(__e152);
        __frag106.appendChild(__e151);
        const __e159 = document.createElement("div");
        __e159.setAttribute("class", "sd-tab-body");
        const __ifAnchor160 = document.createComment("if");
        __e159.appendChild(__ifAnchor160);
        __c107.push(__wireBoundary_b62(__ifAnchor160));
        __frag106.appendChild(__e159);
        return { nodes: Array.from(__frag106.childNodes), cleanup: () => {
          for (const c of __c107)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag330 = document.createDocumentFragment();
        const __c331 = [];
        const __e332 = document.createElement("div");
        __e332.setAttribute("class", "se-empty-state");
        const __e333 = document.createElement("sui-spinner");
        __e333.setAttribute("size", "md");
        __e332.appendChild(__e333);
        const __e334 = document.createElement("span");
        __e334.setAttribute("class", "se-empty-desc");
        const __t335 = document.createTextNode("Loading skill…");
        __e334.appendChild(__t335);
        __e332.appendChild(__e334);
        __frag330.appendChild(__e332);
        return { nodes: Array.from(__frag330.childNodes), cleanup: () => {
          for (const c of __c331)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag336 = document.createDocumentFragment();
        const __c337 = [];
        const __e338 = document.createElement("div");
        __e338.setAttribute("class", "se-empty-state");
        const __e339 = document.createElement("sui-icon");
        __e339.setAttribute("name", "alert-triangle");
        __e339.setAttribute("size", "xl");
        __e339.setAttribute("tone", "warning");
        __e338.appendChild(__e339);
        const __e340 = document.createElement("span");
        __e340.setAttribute("class", "se-empty-title");
        const __t341 = document.createTextNode("Couldn't load skill");
        __e340.appendChild(__t341);
        __e338.appendChild(__e340);
        const __e342 = document.createElement("span");
        __e342.setAttribute("class", "se-empty-desc");
        const __t343 = document.createTextNode("");
        __e342.appendChild(__t343);
        __c337.push(bindText(__t343, () => $scope.skill().error.message));
        __e338.appendChild(__e342);
        __frag336.appendChild(__e338);
        return { nodes: Array.from(__frag336.childNodes), cleanup: () => {
          for (const c of __c337)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag344 = document.createDocumentFragment();
        const __c345 = [];
        const __e346 = document.createElement("div");
        __e346.setAttribute("class", "se-empty-state");
        const __e347 = document.createElement("sui-icon");
        __e347.setAttribute("name", "zap");
        __e347.setAttribute("size", "xl");
        __e347.setAttribute("class", "se-empty-icon");
        __e347.setAttribute("tone", "muted");
        __e346.appendChild(__e347);
        const __e348 = document.createElement("span");
        __e348.setAttribute("class", "se-empty-title");
        const __t349 = document.createTextNode("No skill selected");
        __e348.appendChild(__t349);
        __e346.appendChild(__e348);
        const __e350 = document.createElement("span");
        __e350.setAttribute("class", "se-empty-desc");
        const __t351 = document.createTextNode("Select a skill from the explorer to view its content and invoke it.");
        __e350.appendChild(__t351);
        __e346.appendChild(__e350);
        __frag344.appendChild(__e346);
        return { nodes: Array.from(__frag344.childNodes), cleanup: () => {
          for (const c of __c345)
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
      const __e1 = document.createElement("h1");
      __e1.setAttribute("class", "sr-only");
      const __t2 = document.createTextNode("Skills");
      __e1.appendChild(__t2);
      __e0.appendChild(__e1);
      const __e3 = document.createElement("div");
      __e3.setAttribute("class", "skills-ide");
      const __e4 = document.createElement("aside");
      __e4.setAttribute("class", "skills-explorer");
      __e4.setAttribute("aria-label", "Skills file explorer");
      const __e5 = document.createElement("div");
      __e5.setAttribute("class", "se-head");
      const __e6 = document.createElement("div");
      __e6.setAttribute("class", "se-search");
      const __e7 = document.createElement("sui-icon");
      __e7.setAttribute("name", "search");
      __e7.setAttribute("size", "sm");
      __e7.setAttribute("tone", "muted");
      __e6.appendChild(__e7);
      const __e8 = document.createElement("input");
      __e8.setAttribute("type", "text");
      __e8.setAttribute("placeholder", "Search skills…");
      __e8.setAttribute("spellcheck", "false");
      __e8.setAttribute("aria-label", "Search skills");
      __cleanups.push(bindValue(__e8, $scope.q));
      __cleanups.push(on(__e8, "input", function() {
        const __h = (e) => $scope.q.set(e.target.value);
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e6.appendChild(__e8);
      __e5.appendChild(__e6);
      const __ifAnchor9 = document.createComment("if");
      __e5.appendChild(__ifAnchor9);
      __cleanups.push(__wireBoundary_b2(__ifAnchor9));
      __e4.appendChild(__e5);
      const __e23 = document.createElement("div");
      __e23.setAttribute("class", "se-body");
      __e23.setAttribute("role", "tree");
      __e23.setAttribute("aria-label", "Skills tree");
      const __wlAnchor33 = document.createComment("while-loading");
      __e23.appendChild(__wlAnchor33);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor33, signal: $scope.rail, render: __renderWhileLoading_0 }));
      const __weAnchor39 = document.createComment("while-error");
      __e23.appendChild(__weAnchor39);
      __cleanups.push(bindWhileError({ anchor: __weAnchor39, signal: $scope.rail, render: __renderWhileError_0 }));
      const __ifAnchor40 = document.createComment("if");
      __e23.appendChild(__ifAnchor40);
      __cleanups.push(__wireBoundary_b12(__ifAnchor40));
      __e4.appendChild(__e23);
      __e3.appendChild(__e4);
      const __e104 = document.createElement("div");
      __e104.setAttribute("class", "skills-editor");
      const __ifAnchor105 = document.createComment("if");
      __e104.appendChild(__ifAnchor105);
      __cleanups.push(__wireBoundary_b33(__ifAnchor105));
      __e3.appendChild(__e104);
      __e0.appendChild(__e3);
      const __e352 = document.createElement("div");
      __e352.setAttribute("class", "skills-mobile");
      const __e353 = document.createElement("sui-icon");
      __e353.setAttribute("name", "monitor");
      __e353.setAttribute("size", "xl");
      __e353.setAttribute("tone", "muted");
      __e352.appendChild(__e353);
      const __e354 = document.createElement("span");
      const __t355 = document.createTextNode("Best experienced on desktop");
      __e354.appendChild(__t355);
      __e352.appendChild(__e354);
      __e0.appendChild(__e352);
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
