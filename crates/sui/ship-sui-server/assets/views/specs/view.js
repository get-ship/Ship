// views/specs/page.sui
import { attachRef, bindAttr, bindClass, bindFor, bindIf, bindText, bindValue, bindWhileError, bindWhileLoading, getTemplateHook, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateBindValue, hydrateOn, hydrateRef, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";
import"@ship/sui-runtime/template-hooks";

// views/specs/controller.ts
import { controller, signal, derived, effect, live, ref, unwrap } from "@ship/sui-runtime";
import { liveExpr } from "@ship/sdk-client";
import { route as routeFactory } from "@ship/sui-runtime/route";
var liveSelected = (state) => state.$signals.selected;
var STATUS_CYCLE = ["draft", "ready", "launched"];
var controller_default = controller({
  scope: () => {
    const route = routeFactory();
    const q = signal(route.query.q() || "");
    const selectedId = derived(() => route.query.id() || "");
    const confirmDelete = signal(false);
    const filterInputEl = ref(null);
    const editing = signal(false);
    const saving = signal(false);
    const draftBody = signal("");
    const draftTitle = signal("");
    const agentsList = signal([]);
    const rail = live(() => liveExpr(`ship.specs.summaries(${JSON.stringify({ q: q(), selected: selectedId() })})`));
    const selected = live(() => liveExpr(`ship.specs.detail(${JSON.stringify({ id: selectedId() })})`));
    const noMatches = derived(() => {
      const s = rail();
      if (s.kind !== "ok")
        return false;
      const v = s.value;
      return v.total > 0 && v.groups.every((g) => g.items.length === 0);
    });
    fetch("/api/v1/agents", { credentials: "include" }).then((r) => r.ok ? r.json() : { items: [] }).then((b) => {
      const items = Array.isArray(b?.items) ? b.items : [];
      agentsList.set(items.filter((a) => typeof a?.id === "string"));
    }).catch(() => {});
    effect(() => {
      selectedId();
      editing.set(false);
    });
    return {
      q,
      selectedId,
      confirmDelete,
      filterInputEl,
      rail,
      selected,
      noMatches,
      editing,
      saving,
      draftBody,
      draftTitle,
      agentsList
    };
  },
  on: {
    newSpec: async () => {
      await fetch("/api/v1/specs", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({ title: "New spec", body: `## Goal

## Acceptance
- [ ] ` })
      });
    },
    selectSpec: (state, id) => {
      state.navigate("/specs?id=" + id);
    },
    duplicateSpec: async (state) => {
      const s = unwrap(liveSelected(state));
      if (!s)
        return;
      await fetch("/api/v1/specs", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({ title: (s.title ?? "") + " (copy)", body: s.body || "" })
      });
    },
    launchSpec: async (state) => {
      const id = state.selectedId;
      if (!id)
        return;
      await fetch(`/api/v1/specs/${id}/launch`, { method: "POST", credentials: "include" });
    },
    deleteSpec: async (state) => {
      const id = state.selectedId;
      if (!id)
        return;
      await fetch(`/api/v1/specs/${id}`, { method: "DELETE", credentials: "include" });
      state.confirmDelete = false;
      state.navigate("/specs");
    },
    focusFilter: (state) => {
      const el = state.filterInputEl;
      el?.focus();
    },
    startEdit: (state) => {
      const sel = unwrap(liveSelected(state));
      if (!sel)
        return;
      state.draftTitle = sel.title ?? "";
      state.draftBody = sel.body ?? "";
      state.editing = true;
    },
    cancelEdit: (state) => {
      state.editing = false;
    },
    onTitleInput: (state, evt) => {
      const el = evt.target;
      state.draftTitle = typeof el?.value === "string" ? el.value : "";
    },
    onBodyInput: (state, evt) => {
      const detail = evt.detail;
      if (detail && typeof detail.value === "string") {
        state.draftBody = detail.value;
        return;
      }
      const el = evt.target;
      if (typeof el?.value === "string")
        state.draftBody = el.value;
    },
    saveEdit: async (state) => {
      const id = state.selectedId;
      if (!id)
        return;
      state.saving = true;
      try {
        await fetch(`/api/v1/specs/${id}`, {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          credentials: "include",
          body: JSON.stringify({ title: state.draftTitle, body: state.draftBody })
        });
        state.editing = false;
      } finally {
        state.saving = false;
      }
    },
    setAgent: async (state, evt) => {
      const id = state.selectedId;
      if (!id)
        return;
      const detail = evt.detail;
      const el = evt.target;
      const next = detail && typeof detail.value === "string" ? detail.value : typeof el?.value === "string" ? el.value : "";
      await fetch(`/api/v1/specs/${id}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({ agent: next === "" ? null : next })
      });
    },
    cycleStatus: async (state) => {
      const id = state.selectedId;
      if (!id)
        return;
      const sel = unwrap(liveSelected(state));
      if (!sel)
        return;
      const current = sel.status ?? "draft";
      const idx = STATUS_CYCLE.indexOf(current);
      const next = STATUS_CYCLE[(idx + 1) % STATUS_CYCLE.length];
      await fetch(`/api/v1/specs/${id}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        credentials: "include",
        body: JSON.stringify({ status: next })
      });
    }
  }
});

// views/specs/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var ISLANDS = ["sui-markdown-editor"];
var page_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope["filterInputEl"] = signal2(null);
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __renderWhileLoading_0 = () => {
    const __frag12 = document.createDocumentFragment();
    const __c13 = [];
    const __e14 = document.createElement("div");
    __e14.setAttribute("style", "padding:var(--space-3) var(--space-4);font-size:var(--text-xs);color:var(--muted-foreground)");
    const __t15 = document.createTextNode("Loading…");
    __e14.appendChild(__t15);
    __frag12.appendChild(__e14);
    return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
      for (const c of __c13)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (error) => {
    const __childScope17 = Object.create($scope);
    __childScope17.error = error;
    return (($scope2) => {
      const __frag18 = document.createDocumentFragment();
      const __c19 = [];
      const __e20 = document.createElement("sui-error-state");
      const __t21 = document.createTextNode("");
      __e20.appendChild(__t21);
      __c19.push(bindText(__t21, () => $scope2.error.message));
      __frag18.appendChild(__e20);
      return { nodes: Array.from(__frag18.childNodes), cleanup: () => {
        for (const c of __c19)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope17);
  };
  const __wireBoundary_b9 = (anchor) => bindFor({
    anchor,
    items: () => $scope.rail().value.groups,
    key: (group, __i) => {
      const __child = Object.create($scope);
      __child.group = group;
      __child.__i = __i;
      return (($scope2) => group.key)(__child);
    },
    render: (group, __i) => {
      const __childScope44 = Object.create($scope);
      __childScope44.group = group;
      __childScope44.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b13 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.group.items,
          key: (spec, __i2) => {
            const __child = Object.create($scope2);
            __child.spec = spec;
            __child.__i = __i2;
            return (($scope3) => spec.id)(__child);
          },
          render: (spec, __i2) => {
            const __childScope58 = Object.create($scope2);
            __childScope58.spec = spec;
            __childScope58.__i = __i2;
            return (($scope3) => {
              __childScope58.__v_id = signal2(0);
              const __frag59 = document.createDocumentFragment();
              const __c60 = [];
              const __e61 = document.createElement("sui-list-tile");
              __e61.setAttribute("interactive", "");
              __c60.push(bindAttr(__e61, "selected", () => (void $scope3.__v_id(), $scope3.selectedId() === $scope3.spec.id ? "" : null)));
              __c60.push(on(__e61, "click", function() {
                const __h = () => $scope3.selectSpec($scope3.spec.id);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              const __e62 = document.createElement("sui-status-dot");
              __e62.setAttribute("slot", "leading");
              __e62.setAttribute("size", "sm");
              {
                const __av63 = $scope3.spec.tone;
                if (__av63 == null || __av63 === false || __av63 === "")
                  __e62.removeAttribute("tone");
                else
                  __e62.setAttribute("tone", __av63 === true ? "" : String(__av63));
              }
              {
                const __av64 = $scope3.group.key === "launched" ? "" : null;
                if (__av64 == null || __av64 === false || __av64 === "")
                  __e62.removeAttribute("pulse");
                else
                  __e62.setAttribute("pulse", __av64 === true ? "" : String(__av64));
              }
              __e61.appendChild(__e62);
              const __e65 = document.createElement("div");
              __e65.setAttribute("class", "spec-row-main");
              const __e66 = document.createElement("span");
              __e66.setAttribute("class", "spec-row-name");
              const __t67 = document.createTextNode("");
              __e66.appendChild(__t67);
              __t67.textContent = $scope3.spec.title;
              __e65.appendChild(__e66);
              const __e68 = document.createElement("span");
              __e68.setAttribute("class", "spec-row-when");
              const __t69 = document.createTextNode("");
              __e68.appendChild(__t69);
              __t69.textContent = $scope3.spec.when;
              __e65.appendChild(__e68);
              __e61.appendChild(__e65);
              const __e70 = document.createElement("span");
              __e70.setAttribute("slot", "trailing");
              __e70.setAttribute("class", "spec-row-pipeline");
              const __t71 = document.createTextNode("");
              __e70.appendChild(__t71);
              __t71.textContent = $scope3.spec.pipeline || "—";
              __e61.appendChild(__e70);
              __frag59.appendChild(__e61);
              return { nodes: Array.from(__frag59.childNodes), cleanup: () => {
                for (const c of __c60)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new72, __newI73) => {
                const __old = $scope3.spec;
                $scope3.spec = __new72;
                $scope3.__i = __newI73;
                if (__old.tone !== $scope3.spec.tone) {
                  {
                    const __av63 = $scope3.spec.tone;
                    if (__av63 == null || __av63 === false || __av63 === "")
                      __e62.removeAttribute("tone");
                    else
                      __e62.setAttribute("tone", __av63 === true ? "" : String(__av63));
                  }
                }
                if (__old.title !== $scope3.spec.title) {
                  __t67.textContent = $scope3.spec.title;
                }
                if (__old.when !== $scope3.spec.when) {
                  __t69.textContent = $scope3.spec.when;
                }
                if (__old.pipeline !== $scope3.spec.pipeline) {
                  __t71.textContent = $scope3.spec.pipeline || "—";
                }
                if (__old.id !== $scope3.spec.id) {
                  $scope3.__v_id.set((c) => c + 1);
                }
              } };
            })(__childScope58);
          }
        });
        const __wireBoundary_b10 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.group.items.length > 0)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag48 = document.createDocumentFragment();
              const __c49 = [];
              const __e50 = document.createElement("section");
              __e50.setAttribute("class", "specs-rail-group");
              const __e51 = document.createElement("div");
              __e51.setAttribute("class", "specs-rail-group-head");
              const __e52 = document.createElement("span");
              const __t53 = document.createTextNode("");
              __e52.appendChild(__t53);
              __t53.textContent = $scope2.group.label;
              __e51.appendChild(__e52);
              const __e54 = document.createElement("span");
              __e54.setAttribute("class", "count");
              const __t55 = document.createTextNode("");
              __e54.appendChild(__t55);
              __t55.textContent = $scope2.group.count;
              __e51.appendChild(__e54);
              __e50.appendChild(__e51);
              const __e56 = document.createElement("sui-stack");
              const __forAnchor57 = document.createComment("for");
              __e56.appendChild(__forAnchor57);
              __c49.push(__wireBoundary_b13(__forAnchor57));
              __e50.appendChild(__e56);
              __frag48.appendChild(__e50);
              return { nodes: Array.from(__frag48.childNodes), cleanup: () => {
                for (const c of __c49)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag45 = document.createDocumentFragment();
        const __c46 = [];
        const __ifAnchor47 = document.createComment("if");
        __frag45.appendChild(__ifAnchor47);
        __c46.push(__wireBoundary_b10(__ifAnchor47));
        return { nodes: Array.from(__frag45.childNodes), cleanup: () => {
          for (const c of __c46)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope44);
    }
  });
  const __wireBoundary_b7 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.rail().value.total === 0)
        return 0;
      if ($scope.noMatches())
        return 1;
      return 2;
    },
    branches: [
      () => {
        const __frag27 = document.createDocumentFragment();
        const __c28 = [];
        const __e29 = document.createElement("sui-empty-state");
        __e29.setAttribute("variant", "placeholder");
        __e29.setAttribute("style", "padding:24px 14px");
        const __e30 = document.createElement("span");
        __e30.setAttribute("slot", "title");
        const __t31 = document.createTextNode("No specs yet");
        __e30.appendChild(__t31);
        __e29.appendChild(__e30);
        const __t32 = document.createTextNode(`
            Click + New to start one.
          `);
        __e29.appendChild(__t32);
        __frag27.appendChild(__e29);
        return { nodes: Array.from(__frag27.childNodes), cleanup: () => {
          for (const c of __c28)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag33 = document.createDocumentFragment();
        const __c34 = [];
        const __e35 = document.createElement("sui-empty-state");
        __e35.setAttribute("variant", "placeholder");
        __e35.setAttribute("style", "padding:24px 14px");
        const __e36 = document.createElement("span");
        __e36.setAttribute("slot", "title");
        const __t37 = document.createTextNode("No matches");
        __e36.appendChild(__t37);
        __e35.appendChild(__e36);
        const __t38 = document.createTextNode(`
            No specs match "`);
        __e35.appendChild(__t38);
        const __t39 = document.createTextNode("");
        __e35.appendChild(__t39);
        __c34.push(bindText(__t39, () => $scope.q()));
        const __t40 = document.createTextNode(`". Clear the filter to see all.
          `);
        __e35.appendChild(__t40);
        __frag33.appendChild(__e35);
        return { nodes: Array.from(__frag33.childNodes), cleanup: () => {
          for (const c of __c34)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag41 = document.createDocumentFragment();
        const __c42 = [];
        const __forAnchor43 = document.createComment("for");
        __frag41.appendChild(__forAnchor43);
        __c42.push(__wireBoundary_b9(__forAnchor43));
        return { nodes: Array.from(__frag41.childNodes), cleanup: () => {
          for (const c of __c42)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b6 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.rail().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag24 = document.createDocumentFragment();
        const __c25 = [];
        const __ifAnchor26 = document.createComment("if");
        __frag24.appendChild(__ifAnchor26);
        __c25.push(__wireBoundary_b7(__ifAnchor26));
        return { nodes: Array.from(__frag24.childNodes), cleanup: () => {
          for (const c of __c25)
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
      if ($scope.editing())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag96 = document.createDocumentFragment();
        const __c97 = [];
        const __e98 = document.createElement("sui-input");
        __e98.setAttribute("data-testid", "specs-title-input");
        __c97.push(bindAttr(__e98, "value", () => $scope.draftTitle()));
        __c97.push(on(__e98, "input", function() {
          const __h = $scope.onTitleInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __frag96.appendChild(__e98);
        return { nodes: Array.from(__frag96.childNodes), cleanup: () => {
          for (const c of __c97)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag99 = document.createDocumentFragment();
        const __c100 = [];
        const __e101 = document.createElement("h1");
        const __t102 = document.createTextNode("");
        __e101.appendChild(__t102);
        __c100.push(bindText(__t102, () => $scope.selected().value.title || "(untitled)"));
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
  const __wireBoundary_b26 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.editing())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag105 = document.createDocumentFragment();
        const __c106 = [];
        const __e107 = document.createElement("sui-button");
        __e107.setAttribute("data-testid", "specs-cancel");
        __e107.setAttribute("variant", "outline");
        __e107.setAttribute("size", "sm");
        __c106.push(on(__e107, "click", function() {
          const __h = $scope.cancelEdit;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t108 = document.createTextNode("Cancel");
        __e107.appendChild(__t108);
        __frag105.appendChild(__e107);
        const __e109 = document.createElement("sui-button");
        __e109.setAttribute("data-testid", "specs-save");
        __e109.setAttribute("variant", "solid");
        __e109.setAttribute("tone", "accent");
        __e109.setAttribute("size", "sm");
        __c106.push(bindAttr(__e109, "disabled", () => $scope.saving() ? "" : null));
        __c106.push(on(__e109, "click", function() {
          const __h = $scope.saveEdit;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t110 = document.createTextNode("");
        __e109.appendChild(__t110);
        __c106.push(bindText(__t110, () => $scope.saving() ? "Saving…" : "Save"));
        __frag105.appendChild(__e109);
        return { nodes: Array.from(__frag105.childNodes), cleanup: () => {
          for (const c of __c106)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag111 = document.createDocumentFragment();
        const __c112 = [];
        const __e113 = document.createElement("sui-button");
        __e113.setAttribute("data-testid", "specs-edit");
        __e113.setAttribute("variant", "outline");
        __e113.setAttribute("size", "sm");
        __c112.push(on(__e113, "click", function() {
          const __h = $scope.startEdit;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t114 = document.createTextNode("Edit");
        __e113.appendChild(__t114);
        __frag111.appendChild(__e113);
        const __e115 = document.createElement("sui-button");
        __e115.setAttribute("variant", "outline");
        __e115.setAttribute("size", "sm");
        __c112.push(on(__e115, "click", function() {
          const __h = $scope.duplicateSpec;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t116 = document.createTextNode("Duplicate");
        __e115.appendChild(__t116);
        __frag111.appendChild(__e115);
        const __e117 = document.createElement("sui-button");
        __e117.setAttribute("variant", "outline");
        __e117.setAttribute("size", "sm");
        __e117.setAttribute("tone", "destructive");
        __c112.push(on(__e117, "click", function() {
          const __h = () => $scope.confirmDelete.set(true);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t118 = document.createTextNode("Delete");
        __e117.appendChild(__t118);
        __frag111.appendChild(__e117);
        return { nodes: Array.from(__frag111.childNodes), cleanup: () => {
          for (const c of __c112)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b40 = (anchor) => bindFor({
    anchor,
    items: () => $scope.agentsList(),
    key: (a, __i) => {
      const __child = Object.create($scope);
      __child.a = a;
      __child.__i = __i;
      return (($scope2) => a.id)(__child);
    },
    render: (a, __i) => {
      const __childScope131 = Object.create($scope);
      __childScope131.a = a;
      __childScope131.__i = __i;
      return (($scope2) => {
        __childScope131.__v_id = signal2(0);
        const __frag132 = document.createDocumentFragment();
        const __c133 = [];
        const __e134 = document.createElement("option");
        {
          const __av135 = $scope2.a.id;
          if (__av135 == null || __av135 === false || __av135 === "")
            __e134.removeAttribute("value");
          else
            __e134.setAttribute("value", __av135 === true ? "" : String(__av135));
        }
        __c133.push(bindAttr(__e134, "selected", () => (void $scope2.__v_id(), $scope2.a.id === $scope2.selected().value.agent_id ? "" : null)));
        const __t136 = document.createTextNode("");
        __e134.appendChild(__t136);
        __t136.textContent = $scope2.a.id;
        __frag132.appendChild(__e134);
        return { nodes: Array.from(__frag132.childNodes), cleanup: () => {
          for (const c of __c133)
            try {
              c();
            } catch (e) {}
        }, update: (__new137, __newI138) => {
          const __old = $scope2.a;
          $scope2.a = __new137;
          $scope2.__i = __newI138;
          if (__old.id !== $scope2.a.id) {
            {
              const __av135 = $scope2.a.id;
              if (__av135 == null || __av135 === false || __av135 === "")
                __e134.removeAttribute("value");
              else
                __e134.setAttribute("value", __av135 === true ? "" : String(__av135));
            }
            __t136.textContent = $scope2.a.id;
          }
          if (__old.id !== $scope2.a.id) {
            $scope2.__v_id.set((c) => c + 1);
          }
        } };
      })(__childScope131);
    }
  });
  const __wireBoundary_b50 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.selected().value.goal_md)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag150 = document.createDocumentFragment();
        const __c151 = [];
        const __e152 = document.createElement("h2");
        const __t153 = document.createTextNode("Goal");
        __e152.appendChild(__t153);
        __frag150.appendChild(__e152);
        const __e154 = document.createElement("sui-markdown");
        __c151.push(bindAttr(__e154, "value", () => $scope.selected().value.goal_md));
        __frag150.appendChild(__e154);
        return { nodes: Array.from(__frag150.childNodes), cleanup: () => {
          for (const c of __c151)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b53 = (anchor) => bindFor({
    anchor,
    items: () => $scope.selected().value.acceptance,
    key: (item, __i) => {
      const __child = Object.create($scope);
      __child.item = item;
      __child.__i = __i;
      return (($scope2) => item.text)(__child);
    },
    render: (item, __i) => {
      const __childScope162 = Object.create($scope);
      __childScope162.item = item;
      __childScope162.__i = __i;
      return (($scope2) => {
        const __frag163 = document.createDocumentFragment();
        const __c164 = [];
        const __e165 = document.createElement("li");
        if ($scope2.item.done)
          __e165.classList.add("done");
        else
          __e165.classList.remove("done");
        const __e166 = document.createElement("sui-checkbox");
        {
          const __av167 = $scope2.item.done ? "" : null;
          if (__av167 == null || __av167 === false || __av167 === "")
            __e166.removeAttribute("checked");
          else
            __e166.setAttribute("checked", __av167 === true ? "" : String(__av167));
        }
        __e166.setAttribute("disabled", "");
        __e165.appendChild(__e166);
        const __e168 = document.createElement("span");
        __e168.setAttribute("class", "acc-text");
        const __t169 = document.createTextNode("");
        __e168.appendChild(__t169);
        __t169.textContent = $scope2.item.text;
        __e165.appendChild(__e168);
        __frag163.appendChild(__e165);
        return { nodes: Array.from(__frag163.childNodes), cleanup: () => {
          for (const c of __c164)
            try {
              c();
            } catch (e) {}
        }, update: (__new170, __newI171) => {
          const __old = $scope2.item;
          $scope2.item = __new170;
          $scope2.__i = __newI171;
          if (__old.done !== $scope2.item.done) {
            if ($scope2.item.done)
              __e165.classList.add("done");
            else
              __e165.classList.remove("done");
            {
              const __av167 = $scope2.item.done ? "" : null;
              if (__av167 == null || __av167 === false || __av167 === "")
                __e166.removeAttribute("checked");
              else
                __e166.setAttribute("checked", __av167 === true ? "" : String(__av167));
            }
          }
          if (__old.text !== $scope2.item.text) {
            __t169.textContent = $scope2.item.text;
          }
        } };
      })(__childScope162);
    }
  });
  const __wireBoundary_b52 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.selected().value.acceptance && $scope.selected().value.acceptance.length > 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag156 = document.createDocumentFragment();
        const __c157 = [];
        const __e158 = document.createElement("h2");
        const __t159 = document.createTextNode("Acceptance");
        __e158.appendChild(__t159);
        __frag156.appendChild(__e158);
        const __e160 = document.createElement("ul");
        __e160.setAttribute("class", "specs-acceptance");
        const __forAnchor161 = document.createComment("for");
        __e160.appendChild(__forAnchor161);
        __c157.push(__wireBoundary_b53(__forAnchor161));
        __frag156.appendChild(__e160);
        return { nodes: Array.from(__frag156.childNodes), cleanup: () => {
          for (const c of __c157)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b57 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.selected().value.scope_md)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag173 = document.createDocumentFragment();
        const __c174 = [];
        const __e175 = document.createElement("h2");
        const __t176 = document.createTextNode("Scope");
        __e175.appendChild(__t176);
        __frag173.appendChild(__e175);
        const __e177 = document.createElement("sui-markdown");
        __c174.push(bindAttr(__e177, "value", () => $scope.selected().value.scope_md));
        __frag173.appendChild(__e177);
        return { nodes: Array.from(__frag173.childNodes), cleanup: () => {
          for (const c of __c174)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b59 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.selected().value.linked_md)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag179 = document.createDocumentFragment();
        const __c180 = [];
        const __e181 = document.createElement("h2");
        const __t182 = document.createTextNode("Linked");
        __e181.appendChild(__t182);
        __frag179.appendChild(__e181);
        const __e183 = document.createElement("sui-markdown");
        __c180.push(bindAttr(__e183, "value", () => $scope.selected().value.linked_md));
        __frag179.appendChild(__e183);
        return { nodes: Array.from(__frag179.childNodes), cleanup: () => {
          for (const c of __c180)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b61 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.selected().value.other_md)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag185 = document.createDocumentFragment();
        const __c186 = [];
        const __e187 = document.createElement("sui-markdown");
        __c186.push(bindAttr(__e187, "value", () => $scope.selected().value.other_md));
        __frag185.appendChild(__e187);
        return { nodes: Array.from(__frag185.childNodes), cleanup: () => {
          for (const c of __c186)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b46 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.editing())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag142 = document.createDocumentFragment();
        const __c143 = [];
        const __e144 = document.createElement("div");
        __e144.setAttribute("class", "specs-editor");
        const __e145 = document.createElement("sui-markdown-editor");
        __e145.setAttribute("data-testid", "specs-body-editor");
        __e145.setAttribute("full-height", "");
        __c143.push(bindAttr(__e145, "value", () => $scope.draftBody()));
        __c143.push(on(__e145, "sui-change", function() {
          const __h = $scope.onBodyInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c143.push(on(__e145, "input", function() {
          const __h = $scope.onBodyInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e144.appendChild(__e145);
        __frag142.appendChild(__e144);
        return { nodes: Array.from(__frag142.childNodes), cleanup: () => {
          for (const c of __c143)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag146 = document.createDocumentFragment();
        const __c147 = [];
        const __e148 = document.createElement("article");
        __e148.setAttribute("class", "specs-prose");
        const __ifAnchor149 = document.createComment("if");
        __e148.appendChild(__ifAnchor149);
        __c147.push(__wireBoundary_b50(__ifAnchor149));
        const __ifAnchor155 = document.createComment("if");
        __e148.appendChild(__ifAnchor155);
        __c147.push(__wireBoundary_b52(__ifAnchor155));
        const __ifAnchor172 = document.createComment("if");
        __e148.appendChild(__ifAnchor172);
        __c147.push(__wireBoundary_b57(__ifAnchor172));
        const __ifAnchor178 = document.createComment("if");
        __e148.appendChild(__ifAnchor178);
        __c147.push(__wireBoundary_b59(__ifAnchor178));
        const __ifAnchor184 = document.createComment("if");
        __e148.appendChild(__ifAnchor184);
        __c147.push(__wireBoundary_b61(__ifAnchor184));
        __frag146.appendChild(__e148);
        return { nodes: Array.from(__frag146.childNodes), cleanup: () => {
          for (const c of __c147)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b21 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (!$scope.selectedId())
        return 0;
      if ($scope.selected().kind === "loading")
        return 1;
      if ($scope.selected().kind === "ok")
        return 2;
      return -1;
    },
    branches: [
      () => {
        const __frag76 = document.createDocumentFragment();
        const __c77 = [];
        const __e78 = document.createElement("div");
        __e78.setAttribute("class", "specs-empty");
        const __e79 = document.createElement("sui-empty-state");
        __e79.setAttribute("variant", "placeholder");
        const __e80 = document.createElement("sui-icon");
        __e80.setAttribute("slot", "icon");
        __e80.setAttribute("name", "file-text");
        __e79.appendChild(__e80);
        const __e81 = document.createElement("span");
        __e81.setAttribute("slot", "title");
        const __t82 = document.createTextNode("No spec selected");
        __e81.appendChild(__t82);
        __e79.appendChild(__e81);
        const __t83 = document.createTextNode(`
            Select a spec from the list to view and launch it.
          `);
        __e79.appendChild(__t83);
        __e78.appendChild(__e79);
        __frag76.appendChild(__e78);
        return { nodes: Array.from(__frag76.childNodes), cleanup: () => {
          for (const c of __c77)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag84 = document.createDocumentFragment();
        const __c85 = [];
        const __e86 = document.createElement("div");
        __e86.setAttribute("class", "specs-empty");
        const __e87 = document.createElement("sui-empty-state");
        __e87.setAttribute("variant", "placeholder");
        const __e88 = document.createElement("span");
        __e88.setAttribute("slot", "title");
        const __t89 = document.createTextNode("Loading…");
        __e88.appendChild(__t89);
        __e87.appendChild(__e88);
        __e86.appendChild(__e87);
        __frag84.appendChild(__e86);
        return { nodes: Array.from(__frag84.childNodes), cleanup: () => {
          for (const c of __c85)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag90 = document.createDocumentFragment();
        const __c91 = [];
        const __e92 = document.createElement("div");
        __e92.setAttribute("class", "specs-detail-inner");
        const __e93 = document.createElement("header");
        __e93.setAttribute("class", "specs-detail-head");
        const __e94 = document.createElement("sui-cluster");
        __e94.setAttribute("gap", "4");
        __e94.setAttribute("align", "start");
        __e94.setAttribute("justify", "between");
        const __ifAnchor95 = document.createComment("if");
        __e94.appendChild(__ifAnchor95);
        __c91.push(__wireBoundary_b22(__ifAnchor95));
        const __e103 = document.createElement("div");
        __e103.setAttribute("class", "specs-detail-actions");
        const __ifAnchor104 = document.createComment("if");
        __e103.appendChild(__ifAnchor104);
        __c91.push(__wireBoundary_b26(__ifAnchor104));
        __e94.appendChild(__e103);
        __e93.appendChild(__e94);
        const __e119 = document.createElement("div");
        __e119.setAttribute("class", "specs-meta-controls");
        const __e120 = document.createElement("button");
        __e120.setAttribute("type", "button");
        __e120.setAttribute("class", "status-pill");
        __e120.setAttribute("data-testid", "specs-status-pill");
        __c91.push(bindAttr(__e120, "data-tone", () => $scope.selected().value.tone));
        __c91.push(on(__e120, "click", function() {
          const __h = $scope.cycleStatus;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e120.setAttribute("title", "Click to cycle status");
        const __e121 = document.createElement("sui-status-dot");
        __e121.setAttribute("size", "sm");
        __c91.push(bindAttr(__e121, "tone", () => $scope.selected().value.tone));
        __e120.appendChild(__e121);
        const __e122 = document.createElement("span");
        __e122.setAttribute("class", "status-label");
        const __t123 = document.createTextNode("");
        __e122.appendChild(__t123);
        __c91.push(bindText(__t123, () => $scope.selected().value.status_label));
        __e120.appendChild(__e122);
        __e119.appendChild(__e120);
        const __e124 = document.createElement("label");
        __e124.setAttribute("class", "agent-picker");
        const __e125 = document.createElement("span");
        __e125.setAttribute("class", "agent-picker-label");
        const __t126 = document.createTextNode("Agent");
        __e125.appendChild(__t126);
        __e124.appendChild(__e125);
        const __e127 = document.createElement("sui-select");
        __e127.setAttribute("size", "sm");
        __e127.setAttribute("data-testid", "specs-agent-select");
        __c91.push(bindAttr(__e127, "value", () => $scope.selected().value.agent_id));
        __c91.push(on(__e127, "sui-change", function() {
          const __h = $scope.setAgent;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e128 = document.createElement("option");
        __e128.setAttribute("value", "");
        const __t129 = document.createTextNode("— none —");
        __e128.appendChild(__t129);
        __e127.appendChild(__e128);
        const __forAnchor130 = document.createComment("for");
        __e127.appendChild(__forAnchor130);
        __c91.push(__wireBoundary_b40(__forAnchor130));
        __e124.appendChild(__e127);
        __e119.appendChild(__e124);
        const __e139 = document.createElement("sui-metachip");
        __e139.setAttribute("label", "Pipeline");
        __c91.push(bindAttr(__e139, "value", () => $scope.selected().value.pipeline_label));
        __e119.appendChild(__e139);
        const __e140 = document.createElement("sui-metachip");
        __e140.setAttribute("label", "Branch");
        __c91.push(bindAttr(__e140, "value", () => $scope.selected().value.branch));
        __e119.appendChild(__e140);
        __e93.appendChild(__e119);
        __e92.appendChild(__e93);
        const __ifAnchor141 = document.createComment("if");
        __e92.appendChild(__ifAnchor141);
        __c91.push(__wireBoundary_b46(__ifAnchor141));
        __frag90.appendChild(__e92);
        const __e188 = document.createElement("div");
        __e188.setAttribute("class", "specs-launch-bar");
        const __e189 = document.createElement("div");
        __e189.setAttribute("class", "specs-launch-bar-inner");
        const __e190 = document.createElement("sui-button");
        __e190.setAttribute("data-testid", "specs-launch");
        __e190.setAttribute("variant", "solid");
        __e190.setAttribute("tone", "primary");
        __e190.setAttribute("size", "lg");
        __c91.push(bindAttr(__e190, "disabled", () => !$scope.selected().value.pipeline && !$scope.selected().value.agent_id || $scope.editing() ? "" : null));
        __c91.push(on(__e190, "click", function() {
          const __h = $scope.launchSpec;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t191 = document.createTextNode("");
        __e190.appendChild(__t191);
        __c91.push(bindText(__t191, () => $scope.selected().value.launch_label));
        __e189.appendChild(__e190);
        const __e192 = document.createElement("span");
        __e192.setAttribute("class", "specs-launch-hint");
        const __t193 = document.createTextNode("");
        __e192.appendChild(__t193);
        __c91.push(bindText(__t193, () => $scope.selected().value.launch_hint));
        __e189.appendChild(__e192);
        __e188.appendChild(__e189);
        __frag90.appendChild(__e188);
        const __e194 = document.createElement("sui-alert-dialog");
        __e194.setAttribute("tone", "destructive");
        __e194.setAttribute("confirm-label", "Delete");
        __e194.setAttribute("cancel-label", "Cancel");
        __c91.push(bindAttr(__e194, "open", () => $scope.confirmDelete() ? "" : null));
        __c91.push(on(__e194, "sui-cancel", function() {
          const __h = () => $scope.confirmDelete.set(false);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c91.push(on(__e194, "sui-confirm", function() {
          const __h = $scope.deleteSpec;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e195 = document.createElement("span");
        __e195.setAttribute("slot", "title");
        const __t196 = document.createTextNode("Delete spec");
        __e195.appendChild(__t196);
        __e194.appendChild(__e195);
        const __t197 = document.createTextNode(`
          This removes the spec from the project. This cannot be undone.
        `);
        __e194.appendChild(__t197);
        __frag90.appendChild(__e194);
        return { nodes: Array.from(__frag90.childNodes), cleanup: () => {
          for (const c of __c91)
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
      const __e0 = document.createElement("use-kbd");
      const __e1 = document.createElement("on");
      __e1.setAttribute("key", "/");
      __e1.setAttribute("action", "focusFilter");
      __e0.appendChild(__e1);
      container.appendChild(__e0);
      const __hook_kbd = getTemplateHook("kbd");
      if (__hook_kbd) {
        const __hookAttrs = Object.create(null);
        __hookAttrs.shortcuts = [{ key: "/", action: "focusFilter" }];
        __hookAttrs.resolve = (__n) => $scope[__n];
        __hook_kbd.factory({ attrs: __hookAttrs, el: __e0, cleanup: (__fn) => __cleanups.push(__fn) });
      }
      const __e2 = document.createElement("div");
      __e2.setAttribute("class", "ship-view ship-view-specs");
      const __e3 = document.createElement("div");
      __e3.setAttribute("class", "specs-grid");
      const __e4 = document.createElement("aside");
      __e4.setAttribute("class", "specs-rail");
      __e4.setAttribute("aria-label", "Spec list");
      const __e5 = document.createElement("div");
      __e5.setAttribute("class", "specs-rail-head");
      const __e6 = document.createElement("h1");
      const __t7 = document.createTextNode("Specs");
      __e6.appendChild(__t7);
      __e5.appendChild(__e6);
      const __e8 = document.createElement("sui-button");
      __e8.setAttribute("variant", "solid");
      __e8.setAttribute("tone", "accent");
      __e8.setAttribute("size", "sm");
      __cleanups.push(on(__e8, "click", function() {
        const __h = $scope.newSpec;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t9 = document.createTextNode("+ New");
      __e8.appendChild(__t9);
      __e5.appendChild(__e8);
      __e4.appendChild(__e5);
      const __e10 = document.createElement("div");
      __e10.setAttribute("class", "specs-rail-filter");
      const __e11 = document.createElement("sui-input");
      __e11.setAttribute("size", "sm");
      __e11.setAttribute("placeholder", "Filter…");
      __cleanups.push(bindValue(__e11, $scope.q));
      attachRef($scope, "filterInputEl", __e11);
      __e10.appendChild(__e11);
      __e4.appendChild(__e10);
      const __wlAnchor16 = document.createComment("while-loading");
      __e4.appendChild(__wlAnchor16);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor16, signal: $scope.rail, render: __renderWhileLoading_0 }));
      const __weAnchor22 = document.createComment("while-error");
      __e4.appendChild(__weAnchor22);
      __cleanups.push(bindWhileError({ anchor: __weAnchor22, signal: $scope.rail, render: __renderWhileError_0 }));
      const __ifAnchor23 = document.createComment("if");
      __e4.appendChild(__ifAnchor23);
      __cleanups.push(__wireBoundary_b6(__ifAnchor23));
      __e3.appendChild(__e4);
      const __e74 = document.createElement("main");
      __e74.setAttribute("class", "specs-detail");
      __e74.setAttribute("aria-label", "Spec detail");
      const __ifAnchor75 = document.createComment("if");
      __e74.appendChild(__ifAnchor75);
      __cleanups.push(__wireBoundary_b21(__ifAnchor75));
      __e3.appendChild(__e74);
      __e2.appendChild(__e3);
      container.appendChild(__e2);
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
        __cleanups.push(hydrateOn(__m["b0"], "click", function() {
          const __h = $scope.newSpec;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b1"])
        __cleanups.push(hydrateBindValue(__m["b1"], $scope.q));
      if (__m["b2"])
        __cleanups.push(hydrateRef(__m["b2"], $scope, "filterInputEl"));
      if (__m["b3"])
        __cleanups.push(hydrateWhileLoading(__m["b3"], "b3", () => $scope.rail(), __renderWhileLoading_0));
      if (__m["b4"])
        __cleanups.push(hydrateWhileError(__m["b4"], "b4", () => $scope.rail(), __renderWhileError_0));
      if (__m["b6"]) {
        const __t = __m["b6"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b6(__anchor));
      }
      if (__m["b21"]) {
        const __t = __m["b21"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b21(__anchor));
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
  page_default as default,
  ISLANDS
};
