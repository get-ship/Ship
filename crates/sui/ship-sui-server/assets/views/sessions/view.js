// views/sessions/page.sui
import { bindAttr, bindFor, bindIf, bindProp, bindText, bindWhileError, bindWhileLoading, data, live, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateOn, hydrateProp, hydrateText, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/sessions/controller.ts
import { controller, signal, unwrap } from "@ship/sui-runtime";
var controller_default = controller({
  scope: ({ ship }) => {
    const spawning = signal(false);
    const endingId = signal("");
    const mutationError = signal("");
    const s = ship;
    async function spawnShell() {
      spawning.set(true);
      mutationError.set("");
      try {
        const ws = await activeWorkspaceId();
        await s.sessions.start({ workspace_id: ws, kind: "shell" });
      } catch (e) {
        mutationError.set(e.message);
      } finally {
        spawning.set(false);
      }
    }
    async function endSession(id) {
      endingId.set(id);
      mutationError.set("");
      try {
        await s.sessions.end({ id });
      } catch (e) {
        mutationError.set(e.message);
      } finally {
        endingId.set("");
      }
    }
    return {
      spawning,
      endingId,
      mutationError,
      spawnShell,
      endSession
    };
  },
  derived: {
    items: ({ sessions }) => unwrap(sessions)?.items ?? [],
    total: ({ sessions }) => unwrap(sessions)?.items.length ?? 0
  }
});
async function activeWorkspaceId() {
  try {
    const resp = await fetch("/api/v1/projects", { credentials: "same-origin" });
    if (!resp.ok)
      return "";
    const body = await resp.json();
    return body.active ?? "";
  } catch {
    return "";
  }
}

// views/sessions/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var page_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.sessions = live(() => ship.sessions.list());
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b5 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.mutationError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag14 = document.createDocumentFragment();
        const __c15 = [];
        const __e16 = document.createElement("sui-error-state");
        __e16.setAttribute("variant", "inline");
        __e16.setAttribute("size", "sm");
        __e16.setAttribute("title", "Session operation failed");
        __e16.setAttribute("data-test", "mutation-error");
        const __t17 = document.createTextNode("");
        __e16.appendChild(__t17);
        __c15.push(bindText(__t17, () => $scope.mutationError()));
        const __e18 = document.createElement("sui-button");
        __e18.setAttribute("slot", "action");
        __e18.setAttribute("size", "sm");
        __e18.setAttribute("variant", "outline");
        __c15.push(on(__e18, "click", function() {
          const __h = () => $scope.mutationError.set("");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t19 = document.createTextNode("Dismiss");
        __e18.appendChild(__t19);
        __e16.appendChild(__e18);
        __frag14.appendChild(__e16);
        return { nodes: Array.from(__frag14.childNodes), cleanup: () => {
          for (const c of __c15)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b9 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1, 2, 3],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope24 = Object.create($scope);
      __childScope24.n = n;
      __childScope24.__i = __i;
      return (($scope2) => {
        const __frag25 = document.createDocumentFragment();
        const __c26 = [];
        const __e27 = document.createElement("li");
        __e27.setAttribute("class", "ses-skel-row");
        __e27.setAttribute("aria-hidden", "true");
        const __e28 = document.createElement("div");
        __e28.setAttribute("class", "ses-skel-main");
        const __e29 = document.createElement("sui-skeleton");
        __e29.setAttribute("shape", "circle");
        __e29.setAttribute("size", "sm");
        __e28.appendChild(__e29);
        const __e30 = document.createElement("sui-skeleton");
        __e30.setAttribute("shape", "line");
        __e30.setAttribute("size", "sm");
        __e30.setAttribute("style", "width:64px");
        __e28.appendChild(__e30);
        const __e31 = document.createElement("sui-skeleton");
        __e31.setAttribute("shape", "line");
        __e31.setAttribute("size", "sm");
        __e31.setAttribute("style", "width:200px");
        __e28.appendChild(__e31);
        __e27.appendChild(__e28);
        const __e32 = document.createElement("sui-skeleton");
        __e32.setAttribute("shape", "line");
        __e32.setAttribute("size", "sm");
        __e32.setAttribute("style", "width:56px");
        __e27.appendChild(__e32);
        __frag25.appendChild(__e27);
        return { nodes: Array.from(__frag25.childNodes), cleanup: () => {
          for (const c of __c26)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope24);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag20 = document.createDocumentFragment();
    const __c21 = [];
    const __e22 = document.createElement("ul");
    __e22.setAttribute("class", "ses-list");
    __e22.setAttribute("data-test", "loading");
    __e22.setAttribute("aria-label", "Loading sessions");
    const __forAnchor23 = document.createComment("for");
    __e22.appendChild(__forAnchor23);
    __c21.push(__wireBoundary_b9(__forAnchor23));
    __frag20.appendChild(__e22);
    return { nodes: Array.from(__frag20.childNodes), cleanup: () => {
      for (const c of __c21)
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
      __e37.setAttribute("title", "Couldn't load sessions");
      __e37.setAttribute("data-test", "error");
      const __t38 = document.createTextNode("");
      __e37.appendChild(__t38);
      __c36.push(bindText(__t38, () => $scope2.err.message));
      const __e39 = document.createElement("sui-button");
      __e39.setAttribute("slot", "action");
      __e39.setAttribute("tone", "primary");
      __e39.setAttribute("variant", "solid");
      __c36.push(on(__e39, "click", function() {
        const __h = () => window.location.reload();
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t40 = document.createTextNode("Retry");
      __e39.appendChild(__t40);
      __e37.appendChild(__e39);
      __frag35.appendChild(__e37);
      return { nodes: Array.from(__frag35.childNodes), cleanup: () => {
        for (const c of __c36)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope34);
  };
  const __wireBoundary_b17 = (anchor) => bindFor({
    anchor,
    items: () => $scope.items(),
    key: (s, __i) => {
      const __child = Object.create($scope);
      __child.s = s;
      __child.__i = __i;
      return (($scope2) => s.id)(__child);
    },
    render: (s, __i) => {
      const __childScope60 = Object.create($scope);
      __childScope60.s = s;
      __childScope60.__i = __i;
      return (($scope2) => {
        __childScope60.__v_id = signal2(0);
        __childScope60.__v_status = signal2(0);
        const __wireBoundary_b24 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.s.agent_id)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag74 = document.createDocumentFragment();
              const __c75 = [];
              const __e76 = document.createElement("sui-chip");
              __e76.setAttribute("tone", "primary");
              __e76.setAttribute("variant", "soft");
              __e76.setAttribute("size", "sm");
              const __t77 = document.createTextNode("");
              __e76.appendChild(__t77);
              __t77.textContent = $scope2.s.agent_id;
              __frag74.appendChild(__e76);
              return { nodes: Array.from(__frag74.childNodes), cleanup: () => {
                for (const c of __c75)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b26 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.s.provider)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag79 = document.createDocumentFragment();
              const __c80 = [];
              const __e81 = document.createElement("sui-chip");
              __e81.setAttribute("tone", "muted");
              __e81.setAttribute("variant", "outline");
              __e81.setAttribute("size", "sm");
              const __t82 = document.createTextNode("");
              __e81.appendChild(__t82);
              __t82.textContent = $scope2.s.provider;
              __frag79.appendChild(__e81);
              return { nodes: Array.from(__frag79.childNodes), cleanup: () => {
                for (const c of __c80)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag61 = document.createDocumentFragment();
        const __c62 = [];
        const __e63 = document.createElement("li");
        __e63.setAttribute("class", "ses-row");
        __e63.setAttribute("data-test", "row");
        {
          const __av64 = $scope2.s.status;
          if (__av64 == null || __av64 === false || __av64 === "")
            __e63.removeAttribute("data-status");
          else
            __e63.setAttribute("data-status", __av64 === true ? "" : String(__av64));
        }
        const __e65 = document.createElement("div");
        __e65.setAttribute("class", "ses-row-main");
        const __e66 = document.createElement("sui-status-dot");
        __e66["tone"] = $scope2.s.status === "running" ? "success" : $scope2.s.status === "exited" ? "muted" : "warning";
        __e66.setAttribute("size", "sm");
        {
          const __av67 = $scope2.s.status;
          if (__av67 == null || __av67 === false || __av67 === "")
            __e66.removeAttribute("label");
          else
            __e66.setAttribute("label", __av67 === true ? "" : String(__av67));
        }
        __e65.appendChild(__e66);
        const __e68 = document.createElement("span");
        __e68.setAttribute("class", "ses-row-kind");
        const __t69 = document.createTextNode("");
        __e68.appendChild(__t69);
        __t69.textContent = $scope2.s.kind;
        __e65.appendChild(__e68);
        const __e70 = document.createElement("span");
        __e70.setAttribute("class", "ses-row-id");
        {
          const __av71 = $scope2.s.id;
          if (__av71 == null || __av71 === false || __av71 === "")
            __e70.removeAttribute("title");
          else
            __e70.setAttribute("title", __av71 === true ? "" : String(__av71));
        }
        const __t72 = document.createTextNode("");
        __e70.appendChild(__t72);
        __t72.textContent = $scope2.s.id;
        __e65.appendChild(__e70);
        const __ifAnchor73 = document.createComment("if");
        __e65.appendChild(__ifAnchor73);
        __c62.push(__wireBoundary_b24(__ifAnchor73));
        const __ifAnchor78 = document.createComment("if");
        __e65.appendChild(__ifAnchor78);
        __c62.push(__wireBoundary_b26(__ifAnchor78));
        __e63.appendChild(__e65);
        const __e83 = document.createElement("div");
        __e83.setAttribute("class", "ses-row-meta");
        const __e84 = document.createElement("span");
        __e84.setAttribute("class", "ses-row-status");
        const __t85 = document.createTextNode("");
        __e84.appendChild(__t85);
        __t85.textContent = $scope2.s.status;
        __e83.appendChild(__e84);
        const __e86 = document.createElement("sui-button");
        __e86.setAttribute("size", "sm");
        __e86.setAttribute("variant", "outline");
        {
          const __av87 = "End session " + $scope2.s.id;
          if (__av87 == null || __av87 === false || __av87 === "")
            __e86.removeAttribute("aria-label");
          else
            __e86.setAttribute("aria-label", __av87 === true ? "" : String(__av87));
        }
        __c62.push(bindProp(__e86, "disabled", () => (void $scope2.__v_id(), void $scope2.__v_status(), $scope2.endingId() === $scope2.s.id || $scope2.s.status === "exited")));
        __c62.push(on(__e86, "click", function() {
          const __h = () => $scope2.endSession($scope2.s.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t88 = document.createTextNode("");
        __e86.appendChild(__t88);
        __c62.push(bindText(__t88, () => (void $scope2.__v_id(), $scope2.endingId() === $scope2.s.id ? "Ending…" : "End")));
        __e83.appendChild(__e86);
        __e63.appendChild(__e83);
        __frag61.appendChild(__e63);
        return { nodes: Array.from(__frag61.childNodes), cleanup: () => {
          for (const c of __c62)
            try {
              c();
            } catch (e) {}
        }, update: (__new89, __newI90) => {
          const __old = $scope2.s;
          $scope2.s = __new89;
          $scope2.__i = __newI90;
          if (__old.status !== $scope2.s.status) {
            {
              const __av64 = $scope2.s.status;
              if (__av64 == null || __av64 === false || __av64 === "")
                __e63.removeAttribute("data-status");
              else
                __e63.setAttribute("data-status", __av64 === true ? "" : String(__av64));
            }
            __e66["tone"] = $scope2.s.status === "running" ? "success" : $scope2.s.status === "exited" ? "muted" : "warning";
            {
              const __av67 = $scope2.s.status;
              if (__av67 == null || __av67 === false || __av67 === "")
                __e66.removeAttribute("label");
              else
                __e66.setAttribute("label", __av67 === true ? "" : String(__av67));
            }
            __t85.textContent = $scope2.s.status;
          }
          if (__old.kind !== $scope2.s.kind) {
            __t69.textContent = $scope2.s.kind;
          }
          if (__old.id !== $scope2.s.id) {
            {
              const __av71 = $scope2.s.id;
              if (__av71 == null || __av71 === false || __av71 === "")
                __e70.removeAttribute("title");
              else
                __e70.setAttribute("title", __av71 === true ? "" : String(__av71));
            }
            __t72.textContent = $scope2.s.id;
            {
              const __av87 = "End session " + $scope2.s.id;
              if (__av87 == null || __av87 === false || __av87 === "")
                __e86.removeAttribute("aria-label");
              else
                __e86.setAttribute("aria-label", __av87 === true ? "" : String(__av87));
            }
          }
          if (__old.id !== $scope2.s.id) {
            $scope2.__v_id.set((c) => c + 1);
          }
          if (__old.status !== $scope2.s.status) {
            $scope2.__v_status.set((c) => c + 1);
          }
        } };
      })(__childScope60);
    }
  });
  const __wireBoundary_b14 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.items().length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag46 = document.createDocumentFragment();
        const __c47 = [];
        const __e48 = document.createElement("sui-empty-state");
        __e48.setAttribute("variant", "default");
        __e48.setAttribute("size", "sm");
        __e48.setAttribute("data-test", "empty");
        const __e49 = document.createElement("sui-icon");
        __e49.setAttribute("slot", "icon");
        __e49.setAttribute("name", "terminal");
        __e48.appendChild(__e49);
        const __e50 = document.createElement("span");
        __e50.setAttribute("slot", "title");
        const __t51 = document.createTextNode("No sessions running");
        __e50.appendChild(__t51);
        __e48.appendChild(__e50);
        const __t52 = document.createTextNode(`
          Spawn a shell or start an agent chat to populate this list.
          `);
        __e48.appendChild(__t52);
        const __e53 = document.createElement("sui-button");
        __e53.setAttribute("slot", "action");
        __e53.setAttribute("tone", "primary");
        __e53.setAttribute("variant", "solid");
        __c47.push(bindProp(__e53, "disabled", () => $scope.spawning()));
        __c47.push(on(__e53, "click", function() {
          const __h = $scope.spawnShell;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e54 = document.createElement("sui-icon");
        __e54.setAttribute("slot", "leading");
        __e54.setAttribute("name", "plus");
        __e53.appendChild(__e54);
        const __t55 = document.createTextNode(`
            New shell session
          `);
        __e53.appendChild(__t55);
        __e48.appendChild(__e53);
        __frag46.appendChild(__e48);
        return { nodes: Array.from(__frag46.childNodes), cleanup: () => {
          for (const c of __c47)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag56 = document.createDocumentFragment();
        const __c57 = [];
        const __e58 = document.createElement("ul");
        __e58.setAttribute("class", "ses-list");
        __e58.setAttribute("data-test", "list");
        __e58.setAttribute("data-view-enter", "");
        const __forAnchor59 = document.createComment("for");
        __e58.appendChild(__forAnchor59);
        __c57.push(__wireBoundary_b17(__forAnchor59));
        __frag56.appendChild(__e58);
        return { nodes: Array.from(__frag56.childNodes), cleanup: () => {
          for (const c of __c57)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b13 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.sessions().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag43 = document.createDocumentFragment();
        const __c44 = [];
        const __ifAnchor45 = document.createComment("if");
        __frag43.appendChild(__ifAnchor45);
        __c44.push(__wireBoundary_b14(__ifAnchor45));
        return { nodes: Array.from(__frag43.childNodes), cleanup: () => {
          for (const c of __c44)
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
      __e0.setAttribute("class", "ship-view ship-view-sessions");
      const __e1 = document.createElement("div");
      __e1.setAttribute("class", "ses-wrap");
      const __e2 = document.createElement("header");
      __e2.setAttribute("class", "ses-head");
      const __e3 = document.createElement("div");
      __e3.setAttribute("class", "ses-head-titles");
      const __e4 = document.createElement("h1");
      __e4.setAttribute("class", "ses-title");
      const __t5 = document.createTextNode("Sessions");
      __e4.appendChild(__t5);
      __e3.appendChild(__e4);
      const __e6 = document.createElement("p");
      __e6.setAttribute("class", "ses-sub");
      const __t7 = document.createTextNode("");
      __e6.appendChild(__t7);
      __cleanups.push(bindText(__t7, () => $scope.total()));
      const __t8 = document.createTextNode(" live session");
      __e6.appendChild(__t8);
      const __t9 = document.createTextNode("");
      __e6.appendChild(__t9);
      __cleanups.push(bindText(__t9, () => $scope.total() === 1 ? "" : "s"));
      __e3.appendChild(__e6);
      __e2.appendChild(__e3);
      const __e10 = document.createElement("sui-button");
      __e10.setAttribute("tone", "primary");
      __e10.setAttribute("variant", "solid");
      __cleanups.push(bindProp(__e10, "disabled", () => $scope.spawning()));
      __cleanups.push(on(__e10, "click", function() {
        const __h = $scope.spawnShell;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e11 = document.createElement("sui-icon");
      __e11.setAttribute("slot", "leading");
      __e11.setAttribute("name", "plus");
      __e10.appendChild(__e11);
      const __t12 = document.createTextNode("");
      __e10.appendChild(__t12);
      __cleanups.push(bindText(__t12, () => $scope.spawning() ? "Spawning…" : "New shell session"));
      __e2.appendChild(__e10);
      __e1.appendChild(__e2);
      const __ifAnchor13 = document.createComment("if");
      __e1.appendChild(__ifAnchor13);
      __cleanups.push(__wireBoundary_b5(__ifAnchor13));
      const __wlAnchor33 = document.createComment("while-loading");
      __e1.appendChild(__wlAnchor33);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor33, signal: $scope.sessions, render: __renderWhileLoading_0 }));
      const __weAnchor41 = document.createComment("while-error");
      __e1.appendChild(__weAnchor41);
      __cleanups.push(bindWhileError({ anchor: __weAnchor41, signal: $scope.sessions, render: __renderWhileError_0 }));
      const __ifAnchor42 = document.createComment("if");
      __e1.appendChild(__ifAnchor42);
      __cleanups.push(__wireBoundary_b13(__ifAnchor42));
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
        __cleanups.push(hydrateText(__t["b0"], () => $scope.total()));
      if (__t["b1"])
        __cleanups.push(hydrateText(__t["b1"], () => $scope.total() === 1 ? "" : "s"));
      if (__m["b2"])
        __cleanups.push(hydrateProp(__m["b2"], "disabled", () => $scope.spawning()));
      if (__m["b3"])
        __cleanups.push(hydrateOn(__m["b3"], "click", function() {
          const __h = $scope.spawnShell;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__t["b4"])
        __cleanups.push(hydrateText(__t["b4"], () => $scope.spawning() ? "Spawning…" : "New shell session"));
      if (__m["b5"]) {
        const __t2 = __m["b5"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b5(__anchor));
      }
      if (__m["b8"])
        __cleanups.push(hydrateWhileLoading(__m["b8"], "b8", () => $scope.sessions(), __renderWhileLoading_0));
      if (__m["b10"])
        __cleanups.push(hydrateWhileError(__m["b10"], "b10", () => $scope.sessions(), __renderWhileError_0));
      if (__m["b13"]) {
        const __t2 = __m["b13"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b13(__anchor));
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
