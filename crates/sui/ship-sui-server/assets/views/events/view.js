// views/events/page.sui
import { bindAttr, bindFor, bindIf, bindText, bindValue, bindWhileError, bindWhileLoading, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/events/controller.ts
import { controller, signal, derived, live, unwrap } from "@ship/sui-runtime";
import { liveExpr } from "@ship/sdk-client";
function eventsExpr(args) {
  const filtered = {};
  for (const [k, v] of Object.entries(args)) {
    if (typeof v === "string" && v.length > 0)
      filtered[k] = v;
  }
  const lit = Object.keys(filtered).length === 0 ? "" : JSON.stringify(filtered);
  return `ship.events.list(${lit})`;
}
var controller_default = controller({
  scope: () => {
    const q = signal("");
    const eventType = signal("");
    const actor = signal("");
    const entity = signal("");
    const eventsData = live(() => liveExpr(eventsExpr({
      q: q(),
      event_type: eventType(),
      actor: actor(),
      entity: entity()
    })));
    const totalCount = derived(() => {
      const d = unwrap(eventsData);
      if (!d)
        return 0;
      return d.groups.reduce((acc, g) => acc + g.rows.length, 0);
    });
    return { q, eventType, actor, entity, eventsData, totalCount };
  }
});

// views/events/page.sui
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
  const __wireBoundary_b0 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.eventsData().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag7 = document.createDocumentFragment();
        const __c8 = [];
        const __e9 = document.createElement("div");
        __e9.setAttribute("class", "events-sub");
        const __t10 = document.createTextNode("");
        __e9.appendChild(__t10);
        __c8.push(bindText(__t10, () => $scope.totalCount()));
        const __t11 = document.createTextNode(" events");
        __e9.appendChild(__t11);
        __frag7.appendChild(__e9);
        return { nodes: Array.from(__frag7.childNodes), cleanup: () => {
          for (const c of __c8)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b3 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1, 2, 3],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope20 = Object.create($scope);
      __childScope20.n = n;
      __childScope20.__i = __i;
      return (($scope2) => {
        const __frag21 = document.createDocumentFragment();
        const __c22 = [];
        const __e23 = document.createElement("div");
        __e23.setAttribute("class", "events-skel-row");
        const __e24 = document.createElement("sui-skeleton");
        __e24.setAttribute("shape", "line");
        __e24.setAttribute("size", "md");
        __e23.appendChild(__e24);
        __frag21.appendChild(__e23);
        return { nodes: Array.from(__frag21.childNodes), cleanup: () => {
          for (const c of __c22)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope20);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag12 = document.createDocumentFragment();
    const __c13 = [];
    const __e14 = document.createElement("div");
    __e14.setAttribute("class", "events-skel");
    __e14.setAttribute("aria-busy", "true");
    __e14.setAttribute("aria-label", "Loading events");
    const __e15 = document.createElement("div");
    __e15.setAttribute("class", "events-skel-filters");
    const __e16 = document.createElement("sui-skeleton");
    __e16.setAttribute("shape", "line");
    __e16.setAttribute("size", "md");
    __e16.setAttribute("style", "width:60px");
    __e15.appendChild(__e16);
    const __e17 = document.createElement("sui-skeleton");
    __e17.setAttribute("shape", "line");
    __e17.setAttribute("size", "md");
    __e17.setAttribute("style", "width:90px");
    __e15.appendChild(__e17);
    const __e18 = document.createElement("sui-skeleton");
    __e18.setAttribute("shape", "line");
    __e18.setAttribute("size", "md");
    __e18.setAttribute("style", "width:70px");
    __e15.appendChild(__e18);
    __e14.appendChild(__e15);
    const __forAnchor19 = document.createComment("for");
    __e14.appendChild(__forAnchor19);
    __c13.push(__wireBoundary_b3(__forAnchor19));
    __frag12.appendChild(__e14);
    return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
      for (const c of __c13)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope26 = Object.create($scope);
    __childScope26.err = err;
    return (($scope2) => {
      const __frag27 = document.createDocumentFragment();
      const __c28 = [];
      const __e29 = document.createElement("sui-error-state");
      __e29.setAttribute("variant", "inline");
      __e29.setAttribute("size", "sm");
      __e29.setAttribute("title", "Couldn't load events");
      const __t30 = document.createTextNode("");
      __e29.appendChild(__t30);
      __c28.push(bindText(__t30, () => $scope2.err.message));
      __frag27.appendChild(__e29);
      return { nodes: Array.from(__frag27.childNodes), cleanup: () => {
        for (const c of __c28)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope26);
  };
  const __wireBoundary_b9 = (anchor) => bindFor({
    anchor,
    items: () => $scope.eventsData().value.types,
    key: (t, __i) => {
      const __child = Object.create($scope);
      __child.t = t;
      __child.__i = __i;
      return (($scope2) => t)(__child);
    },
    render: (t, __i) => {
      const __childScope39 = Object.create($scope);
      __childScope39.t = t;
      __childScope39.__i = __i;
      return (($scope2) => {
        __childScope39.__v = signal2(0);
        const __frag40 = document.createDocumentFragment();
        const __c41 = [];
        const __e42 = document.createElement("sui-chip");
        __e42.setAttribute("size", "sm");
        __c41.push(bindAttr(__e42, "data-active", () => (void $scope2.__v(), $scope2.eventType() === $scope2.t ? "" : null)));
        __c41.push(on(__e42, "click", function() {
          const __h = () => $scope2.eventType.set($scope2.t);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t43 = document.createTextNode("");
        __e42.appendChild(__t43);
        __t43.textContent = $scope2.t;
        __frag40.appendChild(__e42);
        return { nodes: Array.from(__frag40.childNodes), cleanup: () => {
          for (const c of __c41)
            try {
              c();
            } catch (e) {}
        }, update: (__new44, __newI45) => {
          $scope2.t = __new44;
          $scope2.__i = __newI45;
          __t43.textContent = $scope2.t;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope39);
    }
  });
  const __wireBoundary_b16 = (anchor) => bindFor({
    anchor,
    items: () => $scope.eventsData().value.actors,
    key: (a, __i) => {
      const __child = Object.create($scope);
      __child.a = a;
      __child.__i = __i;
      return (($scope2) => a)(__child);
    },
    render: (a, __i) => {
      const __childScope53 = Object.create($scope);
      __childScope53.a = a;
      __childScope53.__i = __i;
      return (($scope2) => {
        __childScope53.__v = signal2(0);
        const __frag54 = document.createDocumentFragment();
        const __c55 = [];
        const __e56 = document.createElement("sui-chip");
        __e56.setAttribute("size", "sm");
        __c55.push(bindAttr(__e56, "data-active", () => (void $scope2.__v(), $scope2.actor() === $scope2.a ? "" : null)));
        __c55.push(on(__e56, "click", function() {
          const __h = () => $scope2.actor.set($scope2.a);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t57 = document.createTextNode("");
        __e56.appendChild(__t57);
        __t57.textContent = $scope2.a;
        __frag54.appendChild(__e56);
        return { nodes: Array.from(__frag54.childNodes), cleanup: () => {
          for (const c of __c55)
            try {
              c();
            } catch (e) {}
        }, update: (__new58, __newI59) => {
          $scope2.a = __new58;
          $scope2.__i = __newI59;
          __t57.textContent = $scope2.a;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope53);
    }
  });
  const __wireBoundary_b13 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.eventsData().value.actors.length > 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag47 = document.createDocumentFragment();
        const __c48 = [];
        const __e49 = document.createElement("span");
        __e49.setAttribute("class", "events-sep");
        __e49.setAttribute("aria-hidden", "true");
        __frag47.appendChild(__e49);
        const __e50 = document.createElement("sui-chip");
        __e50.setAttribute("size", "sm");
        __c48.push(bindAttr(__e50, "data-active", () => $scope.actor() === "" ? "" : null));
        __c48.push(on(__e50, "click", function() {
          const __h = () => $scope.actor.set("");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t51 = document.createTextNode("All actors");
        __e50.appendChild(__t51);
        __frag47.appendChild(__e50);
        const __forAnchor52 = document.createComment("for");
        __frag47.appendChild(__forAnchor52);
        __c48.push(__wireBoundary_b16(__forAnchor52));
        return { nodes: Array.from(__frag47.childNodes), cleanup: () => {
          for (const c of __c48)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b23 = (anchor) => bindFor({
    anchor,
    items: () => $scope.eventsData().value.groups,
    key: (g, __i) => {
      const __child = Object.create($scope);
      __child.g = g;
      __child.__i = __i;
      return (($scope2) => g.entity_id)(__child);
    },
    render: (g, __i) => {
      const __childScope72 = Object.create($scope);
      __childScope72.g = g;
      __childScope72.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b28 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.g.rows,
          key: (r, __i2) => {
            const __child = Object.create($scope2);
            __child.r = r;
            __child.__i = __i2;
            return (($scope3) => r.id)(__child);
          },
          render: (r, __i2) => {
            const __childScope85 = Object.create($scope2);
            __childScope85.r = r;
            __childScope85.__i = __i2;
            return (($scope3) => {
              const __frag86 = document.createDocumentFragment();
              const __c87 = [];
              const __e88 = document.createElement("details");
              __e88.setAttribute("class", "events-row-wrap");
              const __e89 = document.createElement("summary");
              __e89.setAttribute("class", "events-row");
              const __e90 = document.createElement("span");
              __e90.setAttribute("class", "ts");
              const __t91 = document.createTextNode("");
              __e90.appendChild(__t91);
              __t91.textContent = $scope3.r.timestamp;
              __e89.appendChild(__e90);
              const __e92 = document.createElement("span");
              __e92.setAttribute("class", "kind");
              const __t93 = document.createTextNode("");
              __e92.appendChild(__t93);
              __t93.textContent = $scope3.r.event_type;
              __e89.appendChild(__e92);
              const __e94 = document.createElement("span");
              __e94.setAttribute("class", "actor-cell");
              const __e95 = document.createElement("sui-avatar");
              __e95.setAttribute("size", "sm");
              {
                const __av96 = $scope3.r.actor;
                if (__av96 == null || __av96 === false || __av96 === "")
                  __e95.removeAttribute("identity-hue");
                else
                  __e95.setAttribute("identity-hue", __av96 === true ? "" : String(__av96));
              }
              {
                const __av97 = $scope3.r.actor;
                if (__av97 == null || __av97 === false || __av97 === "")
                  __e95.removeAttribute("name");
                else
                  __e95.setAttribute("name", __av97 === true ? "" : String(__av97));
              }
              __e94.appendChild(__e95);
              const __e98 = document.createElement("span");
              __e98.setAttribute("class", "actor-name");
              const __t99 = document.createTextNode("");
              __e98.appendChild(__t99);
              __t99.textContent = $scope3.r.actor;
              __e94.appendChild(__e98);
              __e89.appendChild(__e94);
              const __e100 = document.createElement("span");
              __e100.setAttribute("class", "id");
              {
                const __av101 = $scope3.r.id;
                if (__av101 == null || __av101 === false || __av101 === "")
                  __e100.removeAttribute("title");
                else
                  __e100.setAttribute("title", __av101 === true ? "" : String(__av101));
              }
              const __t102 = document.createTextNode("");
              __e100.appendChild(__t102);
              __t102.textContent = $scope3.r.id;
              __e89.appendChild(__e100);
              __e88.appendChild(__e89);
              const __e103 = document.createElement("pre");
              const __t104 = document.createTextNode("");
              __e103.appendChild(__t104);
              __t104.textContent = $scope3.r.payload_pretty;
              __e88.appendChild(__e103);
              __frag86.appendChild(__e88);
              return { nodes: Array.from(__frag86.childNodes), cleanup: () => {
                for (const c of __c87)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new105, __newI106) => {
                const __old = $scope3.r;
                $scope3.r = __new105;
                $scope3.__i = __newI106;
                if (__old.timestamp !== $scope3.r.timestamp) {
                  __t91.textContent = $scope3.r.timestamp;
                }
                if (__old.event_type !== $scope3.r.event_type) {
                  __t93.textContent = $scope3.r.event_type;
                }
                if (__old.actor !== $scope3.r.actor) {
                  {
                    const __av96 = $scope3.r.actor;
                    if (__av96 == null || __av96 === false || __av96 === "")
                      __e95.removeAttribute("identity-hue");
                    else
                      __e95.setAttribute("identity-hue", __av96 === true ? "" : String(__av96));
                  }
                  {
                    const __av97 = $scope3.r.actor;
                    if (__av97 == null || __av97 === false || __av97 === "")
                      __e95.removeAttribute("name");
                    else
                      __e95.setAttribute("name", __av97 === true ? "" : String(__av97));
                  }
                  __t99.textContent = $scope3.r.actor;
                }
                if (__old.id !== $scope3.r.id) {
                  {
                    const __av101 = $scope3.r.id;
                    if (__av101 == null || __av101 === false || __av101 === "")
                      __e100.removeAttribute("title");
                    else
                      __e100.setAttribute("title", __av101 === true ? "" : String(__av101));
                  }
                  __t102.textContent = $scope3.r.id;
                }
                if (__old.payload_pretty !== $scope3.r.payload_pretty) {
                  __t104.textContent = $scope3.r.payload_pretty;
                }
              } };
            })(__childScope85);
          }
        });
        const __frag73 = document.createDocumentFragment();
        const __c74 = [];
        const __e75 = document.createElement("details");
        {
          const __av76 = !$scope2.g.collapsed ? "" : null;
          if (__av76 == null || __av76 === false || __av76 === "")
            __e75.removeAttribute("open");
          else
            __e75.setAttribute("open", __av76 === true ? "" : String(__av76));
        }
        const __e77 = document.createElement("summary");
        const __e78 = document.createElement("span");
        __e78.setAttribute("class", "entity");
        const __t79 = document.createTextNode("");
        __e78.appendChild(__t79);
        __t79.textContent = $scope2.g.entity_id;
        __e77.appendChild(__e78);
        const __e80 = document.createElement("span");
        __e80.setAttribute("class", "count");
        const __t81 = document.createTextNode("");
        __e80.appendChild(__t81);
        __t81.textContent = $scope2.g.count;
        const __t82 = document.createTextNode(" event");
        __e80.appendChild(__t82);
        const __t83 = document.createTextNode("");
        __e80.appendChild(__t83);
        __t83.textContent = $scope2.g.count !== 1 ? "s" : "";
        __e77.appendChild(__e80);
        __e75.appendChild(__e77);
        const __forAnchor84 = document.createComment("for");
        __e75.appendChild(__forAnchor84);
        __c74.push(__wireBoundary_b28(__forAnchor84));
        __frag73.appendChild(__e75);
        return { nodes: Array.from(__frag73.childNodes), cleanup: () => {
          for (const c of __c74)
            try {
              c();
            } catch (e) {}
        }, update: (__new107, __newI108) => {
          const __old = $scope2.g;
          $scope2.g = __new107;
          $scope2.__i = __newI108;
          if (__old.collapsed !== $scope2.g.collapsed) {
            {
              const __av76 = !$scope2.g.collapsed ? "" : null;
              if (__av76 == null || __av76 === false || __av76 === "")
                __e75.removeAttribute("open");
              else
                __e75.setAttribute("open", __av76 === true ? "" : String(__av76));
            }
          }
          if (__old.entity_id !== $scope2.g.entity_id) {
            __t79.textContent = $scope2.g.entity_id;
          }
          if (__old.count !== $scope2.g.count) {
            __t81.textContent = $scope2.g.count;
            __t83.textContent = $scope2.g.count !== 1 ? "s" : "";
          }
        } };
      })(__childScope72);
    }
  });
  const __wireBoundary_b22 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.eventsData().value.groups.length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag63 = document.createDocumentFragment();
        const __c64 = [];
        const __e65 = document.createElement("sui-empty-state");
        __e65.setAttribute("icon", "activity");
        __e65.setAttribute("variant", "placeholder");
        const __e66 = document.createElement("span");
        __e66.setAttribute("slot", "title");
        const __t67 = document.createTextNode("Waiting for events…");
        __e66.appendChild(__t67);
        __e65.appendChild(__e66);
        const __t68 = document.createTextNode(`
            No events match the current filter yet.
          `);
        __e65.appendChild(__t68);
        __frag63.appendChild(__e65);
        return { nodes: Array.from(__frag63.childNodes), cleanup: () => {
          for (const c of __c64)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag69 = document.createDocumentFragment();
        const __c70 = [];
        const __forAnchor71 = document.createComment("for");
        __frag69.appendChild(__forAnchor71);
        __c70.push(__wireBoundary_b23(__forAnchor71));
        return { nodes: Array.from(__frag69.childNodes), cleanup: () => {
          for (const c of __c70)
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
      if ($scope.eventsData().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag33 = document.createDocumentFragment();
        const __c34 = [];
        const __e35 = document.createElement("div");
        __e35.setAttribute("class", "events-filters");
        __e35.setAttribute("role", "group");
        __e35.setAttribute("aria-label", "Filter events");
        const __e36 = document.createElement("sui-chip");
        __e36.setAttribute("size", "sm");
        __c34.push(bindAttr(__e36, "data-active", () => $scope.eventType() === "" ? "" : null));
        __c34.push(on(__e36, "click", function() {
          const __h = () => $scope.eventType.set("");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t37 = document.createTextNode("All types");
        __e36.appendChild(__t37);
        __e35.appendChild(__e36);
        const __forAnchor38 = document.createComment("for");
        __e35.appendChild(__forAnchor38);
        __c34.push(__wireBoundary_b9(__forAnchor38));
        const __ifAnchor46 = document.createComment("if");
        __e35.appendChild(__ifAnchor46);
        __c34.push(__wireBoundary_b13(__ifAnchor46));
        const __e60 = document.createElement("input");
        __e60.setAttribute("class", "events-search");
        __e60.setAttribute("placeholder", "Search events…");
        __c34.push(bindValue(__e60, $scope.q));
        __c34.push(on(__e60, "input", function() {
          const __h = (e) => $scope.q.set(e.target.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e35.appendChild(__e60);
        __frag33.appendChild(__e35);
        const __e61 = document.createElement("div");
        __e61.setAttribute("class", "events-content");
        __e61.setAttribute("id", "events-content");
        const __ifAnchor62 = document.createComment("if");
        __e61.appendChild(__ifAnchor62);
        __c34.push(__wireBoundary_b22(__ifAnchor62));
        __frag33.appendChild(__e61);
        return { nodes: Array.from(__frag33.childNodes), cleanup: () => {
          for (const c of __c34)
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
      __e0.setAttribute("class", "ship-view ship-view-events");
      const __e1 = document.createElement("div");
      __e1.setAttribute("class", "events-wrap");
      const __e2 = document.createElement("div");
      __e2.setAttribute("class", "events-head");
      const __e3 = document.createElement("div");
      const __e4 = document.createElement("h1");
      __e4.setAttribute("class", "events-title");
      const __t5 = document.createTextNode("Events");
      __e4.appendChild(__t5);
      __e3.appendChild(__e4);
      const __ifAnchor6 = document.createComment("if");
      __e3.appendChild(__ifAnchor6);
      __cleanups.push(__wireBoundary_b0(__ifAnchor6));
      __e2.appendChild(__e3);
      __e1.appendChild(__e2);
      const __wlAnchor25 = document.createComment("while-loading");
      __e1.appendChild(__wlAnchor25);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor25, signal: $scope.eventsData, render: __renderWhileLoading_0 }));
      const __weAnchor31 = document.createComment("while-error");
      __e1.appendChild(__weAnchor31);
      __cleanups.push(bindWhileError({ anchor: __weAnchor31, signal: $scope.eventsData, render: __renderWhileError_0 }));
      const __ifAnchor32 = document.createComment("if");
      __e1.appendChild(__ifAnchor32);
      __cleanups.push(__wireBoundary_b6(__ifAnchor32));
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
      const __m = collectElementMarkers(root);
      if (__m["b0"]) {
        const __t = __m["b0"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b0(__anchor));
      }
      if (__m["b2"])
        __cleanups.push(hydrateWhileLoading(__m["b2"], "b2", () => $scope.eventsData(), __renderWhileLoading_0));
      if (__m["b4"])
        __cleanups.push(hydrateWhileError(__m["b4"], "b4", () => $scope.eventsData(), __renderWhileError_0));
      if (__m["b6"]) {
        const __t = __m["b6"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b6(__anchor));
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
