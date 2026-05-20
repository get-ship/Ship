// views/canvas/page.sui
import { attachRef, bindAttr, bindClass, bindFor, bindIf, bindText, data, live, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateAttr, hydrateOn, hydrateText } from "@ship/sui-dom";

// views/canvas/controller.ts
import { controller, signal, derived, ref } from "@ship/sui-runtime";
import { route as routeFactory } from "@ship/sui-runtime/route";
var SUI_CANVAS_BUNDLE = "/_sui/sui-components/sui-canvas.js";
var SUI_CHROME_BUNDLE = "/_sui/sui-components/sui-chrome.js";
var PAIR_KEY_PREFIX = "ship.canvas.pair.";
function ensureSuiCanvasLoaded() {
  if (typeof window === "undefined")
    return;
  if (customElements.get("sui-canvas"))
    return;
  if (document.querySelector("script[data-sui-canvas-island]"))
    return;
  const s = document.createElement("script");
  s.type = "module";
  s.src = SUI_CANVAS_BUNDLE;
  s.dataset["suiCanvasIsland"] = "1";
  document.head.appendChild(s);
}
function ensureChromeLoaded() {
  if (typeof window === "undefined")
    return;
  if (customElements.get("sui-annotation-layer"))
    return;
  if (document.querySelector("script[data-sui-chrome-island]"))
    return;
  const s = document.createElement("script");
  s.type = "module";
  s.src = SUI_CHROME_BUNDLE;
  s.dataset["suiChromeIsland"] = "1";
  document.head.appendChild(s);
}
function readPair(canvasId) {
  try {
    return localStorage.getItem(PAIR_KEY_PREFIX + canvasId) ?? "";
  } catch {
    return "";
  }
}
function writePair(canvasId, agentId) {
  try {
    if (agentId)
      localStorage.setItem(PAIR_KEY_PREFIX + canvasId, agentId);
    else
      localStorage.removeItem(PAIR_KEY_PREFIX + canvasId);
  } catch {}
}
function newCanvasId() {
  const c = typeof crypto !== "undefined" && crypto?.randomUUID ? crypto.randomUUID() : "";
  return c || `c-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
}
var controller_default = controller({
  scope: ({ agentsSummaries, artifactsList }) => {
    const route = routeFactory();
    const initialId = route.query.canvas() || newCanvasId();
    const canvasId = signal(initialId);
    const pairedAgentId = signal(readPair(initialId));
    const recentOpen = signal(false);
    const annMode = signal("off");
    const annThread = signal([]);
    const statusText = signal("ready");
    const canvasEl = ref(null);
    const annLayerEl = ref(null);
    const shortId = derived(() => canvasId().slice(0, 8));
    const annCount = derived(() => annThread().length);
    const recentItems = derived(() => {
      const v = artifactsList();
      if (v?.kind !== "ok")
        return [];
      return (v.value?.items ?? []).filter((a) => a.collection_id === "canvas");
    });
    const pairedAgentLabel = derived(() => {
      const id = pairedAgentId();
      if (!id)
        return "";
      const list = agentsSummaries();
      if (list?.kind !== "ok")
        return id;
      const a = list.value?.items?.find((x) => x.id === id);
      return a?.name || id;
    });
    return {
      canvasId,
      pairedAgentId,
      recentOpen,
      annMode,
      annThread,
      statusText,
      canvasEl,
      annLayerEl,
      shortId,
      annCount,
      pairedAgentLabel,
      recentItems
    };
  },
  on: {
    pair: (s, agentId) => {
      const cid = s.canvasId;
      s.navigate("/canvas?canvas=" + encodeURIComponent(cid));
      s.pairedAgentId = agentId;
      writePair(cid, agentId);
    },
    unpair: (s) => {
      writePair(s.canvasId, "");
      s.pairedAgentId = "";
    },
    toggleRecent: (s) => {
      s.recentOpen = !s.recentOpen;
    },
    newCanvas: (s) => {
      const id = newCanvasId();
      const nav = s.navigate;
      s.canvasId = id;
      s.pairedAgentId = "";
      s.annThread = [];
      s.statusText = "new canvas";
      nav("/canvas?canvas=" + encodeURIComponent(id));
    },
    setAnnMode: (s, mode) => {
      s.annMode = mode;
    },
    clearAnnotations: (s) => {
      const layer = s.annLayerEl;
      if (layer)
        layer.clear();
      s.annThread = [];
    },
    sendAnnotations: async (s) => {
      const cid = s.canvasId;
      const agentId = s.pairedAgentId;
      const layer = s.annLayerEl;
      const staged = layer ? Array.from(layer.staged) : [];
      if (staged.length === 0) {
        s.statusText = "no annotations to send";
        return;
      }
      const resp = await fetch("/api/v1/events/emit", {
        method: "POST",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          event_type: "canvas.annotation",
          entity_id: cid,
          actor_id: "studio",
          payload: { canvas_id: cid, agent_id: agentId, annotations: staged }
        })
      });
      if (!resp.ok) {
        s.statusText = "send failed: " + resp.status;
        return;
      }
      if (layer)
        layer.clear();
      s.annThread = [];
      s.statusText = `sent ${staged.length} annotation${staged.length === 1 ? "" : "s"}`;
    },
    saveCanvas: async (s) => {
      const cid = s.canvasId;
      const el = s.canvasEl;
      if (!el) {
        s.statusText = "canvas not ready";
        return;
      }
      const html = el.getContentHtml();
      if (!html || html.trim().length === 0) {
        s.statusText = "nothing to save (empty canvas)";
        return;
      }
      const fd = new FormData;
      const blob = new Blob([html], { type: "text/html" });
      fd.append("file", blob, `${cid}.html`);
      fd.append("collection_id", "canvas");
      fd.append("item_id", cid);
      const resp = await fetch("/api/v1/artifacts", {
        method: "POST",
        credentials: "same-origin",
        body: fd
      });
      if (!resp.ok) {
        const t = await resp.text().catch(() => "");
        s.statusText = `save failed: ${resp.status} ${t.slice(0, 80)}`;
        return;
      }
      const body = await resp.json().catch(() => ({}));
      s.statusText = `saved · artifact ${(body.data?.id || "").slice(0, 8)}`;
    },
    loadArtifact: async (s, artifactId) => {
      const resp = await fetch(`/api/v1/artifacts/${encodeURIComponent(artifactId)}`, {
        credentials: "same-origin"
      });
      if (!resp.ok) {
        s.statusText = `load failed: ${resp.status}`;
        return;
      }
      const html = await resp.text();
      const el = s.canvasEl;
      if (!el) {
        s.statusText = "canvas not mounted";
        return;
      }
      el.setContent(html);
      s.recentOpen = false;
      s.statusText = `loaded · artifact ${artifactId.slice(0, 8)}`;
    }
  },
  mount: (state, host) => {
    ensureSuiCanvasLoaded();
    ensureChromeLoaded();
    const s = state;
    function refreshThread() {
      const layer = s.annLayerEl;
      const staged = layer && Array.isArray(layer.staged) ? layer.staged : [];
      state.annThread = staged.map((a) => ({
        id: a.id,
        kind: a.kind,
        note: a.note ?? ""
      }));
    }
    const onStaged = (e) => {
      const d = e.detail;
      if (!d?.id)
        return;
      state.annThread = [
        ...s.annThread,
        { id: d.id, kind: d.kind ?? "point", note: d.note ?? "" }
      ];
    };
    host.addEventListener("sui-annotation-staged", onStaged);
    const onContent = (e) => {
      const d = e.detail;
      state.statusText = `content updated (${d?.length ?? 0} bytes)`;
    };
    const onMorph = (e) => {
      const d = e.detail;
      state.statusText = d?.miss ? `morph miss: ${d.target}` : `morph applied: ${d?.target}`;
    };
    host.addEventListener("sui-canvas-content", onContent);
    host.addEventListener("sui-canvas-morph", onMorph);
    const sCanvas = s;
    window.__shipCanvas = {
      setContent: (sui) => sCanvas.canvasEl?.setContent(sui),
      applyMorph: (target, patch) => sCanvas.canvasEl?.applyMorph(target, patch),
      clear: () => sCanvas.canvasEl?.clear(),
      getContentHtml: () => sCanvas.canvasEl?.getContentHtml() ?? "",
      canvasId: () => s.canvasId,
      pairedAgentId: () => s.pairedAgentId
    };
    refreshThread();
    return () => {
      host.removeEventListener("sui-annotation-staged", onStaged);
      host.removeEventListener("sui-canvas-content", onContent);
      host.removeEventListener("sui-canvas-morph", onMorph);
      try {
        delete window.__shipCanvas;
      } catch {}
    };
  }
});

// views/canvas/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var page_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.agentsSummaries = live(() => ship.agents.summaries());
  $scope.artifactsList = live(() => ship.artifacts.list());
  $scope["annLayerEl"] = signal2(null);
  $scope["canvasEl"] = signal2(null);
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b1 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.pairedAgentId())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag10 = document.createDocumentFragment();
        const __c11 = [];
        const __e12 = document.createElement("span");
        __e12.setAttribute("class", "canvas-pair-chip");
        const __e13 = document.createElement("sui-icon");
        __e13.setAttribute("name", "user");
        __e13.setAttribute("aria-hidden", "true");
        __e12.appendChild(__e13);
        const __t14 = document.createTextNode("");
        __e12.appendChild(__t14);
        __c11.push(bindText(__t14, () => $scope.pairedAgentLabel()));
        const __e15 = document.createElement("button");
        __e15.setAttribute("type", "button");
        __e15.setAttribute("class", "canvas-pair-unpair");
        __c11.push(on(__e15, "click", function() {
          const __h = $scope.unpair;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e15.setAttribute("title", "Unpair");
        const __e16 = document.createElement("sui-icon");
        __e16.setAttribute("name", "x");
        __e16.setAttribute("aria-hidden", "true");
        __e15.appendChild(__e16);
        __e12.appendChild(__e15);
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
  const __wireBoundary_b10 = (anchor) => bindFor({
    anchor,
    items: () => $scope.recentItems(),
    key: (art, __i) => {
      const __child = Object.create($scope);
      __child.art = art;
      __child.__i = __i;
      return (($scope2) => art.id)(__child);
    },
    render: (art, __i) => {
      const __childScope37 = Object.create($scope);
      __childScope37.art = art;
      __childScope37.__i = __i;
      return (($scope2) => {
        __childScope37.__v_created_at = signal2(0);
        const __frag38 = document.createDocumentFragment();
        const __c39 = [];
        const __e40 = document.createElement("button");
        __e40.setAttribute("type", "button");
        __e40.setAttribute("class", "canvas-recent-item");
        __c39.push(on(__e40, "click", function() {
          const __h = (e) => {
            e.preventDefault();
            $scope2.loadArtifact($scope2.art.id);
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e41 = document.createElement("span");
        __e41.setAttribute("class", "canvas-recent-name");
        const __t42 = document.createTextNode("");
        __e41.appendChild(__t42);
        __t42.textContent = $scope2.art.name || $scope2.art.id;
        __e40.appendChild(__e41);
        const __e43 = document.createElement("span");
        __e43.setAttribute("class", "canvas-recent-meta");
        const __t44 = document.createTextNode("");
        __e43.appendChild(__t44);
        __c39.push(bindText(__t44, () => (void $scope2.__v_created_at(), ($scope2.art.created_at || "").slice(0, 16))));
        __e40.appendChild(__e43);
        __frag38.appendChild(__e40);
        return { nodes: Array.from(__frag38.childNodes), cleanup: () => {
          for (const c of __c39)
            try {
              c();
            } catch (e) {}
        }, update: (__new45, __newI46) => {
          const __old = $scope2.art;
          $scope2.art = __new45;
          $scope2.__i = __newI46;
          if (__old.id !== $scope2.art.id || __old.name !== $scope2.art.name) {
            __t42.textContent = $scope2.art.name || $scope2.art.id;
          }
          if (__old.created_at !== $scope2.art.created_at) {
            $scope2.__v_created_at.set((c) => c + 1);
          }
        } };
      })(__childScope37);
    }
  });
  const __wireBoundary_b9 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.recentItems().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag33 = document.createDocumentFragment();
        const __c34 = [];
        const __e35 = document.createElement("div");
        __e35.setAttribute("class", "canvas-recent-list");
        const __forAnchor36 = document.createComment("for");
        __e35.appendChild(__forAnchor36);
        __c34.push(__wireBoundary_b10(__forAnchor36));
        __frag33.appendChild(__e35);
        return { nodes: Array.from(__frag33.childNodes), cleanup: () => {
          for (const c of __c34)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag47 = document.createDocumentFragment();
        const __c48 = [];
        const __e49 = document.createElement("div");
        __e49.setAttribute("class", "canvas-recent-empty");
        const __t50 = document.createTextNode("No saved canvases yet.");
        __e49.appendChild(__t50);
        __frag47.appendChild(__e49);
        return { nodes: Array.from(__frag47.childNodes), cleanup: () => {
          for (const c of __c48)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b8 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.recentOpen())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag27 = document.createDocumentFragment();
        const __c28 = [];
        const __e29 = document.createElement("div");
        __e29.setAttribute("class", "canvas-recent-popover");
        __e29.setAttribute("role", "dialog");
        __e29.setAttribute("aria-label", "Recent canvases");
        const __e30 = document.createElement("div");
        __e30.setAttribute("class", "canvas-recent-header");
        const __t31 = document.createTextNode("Recent canvases");
        __e30.appendChild(__t31);
        __e29.appendChild(__e30);
        const __ifAnchor32 = document.createComment("if");
        __e29.appendChild(__ifAnchor32);
        __c28.push(__wireBoundary_b9(__ifAnchor32));
        __frag27.appendChild(__e29);
        return { nodes: Array.from(__frag27.childNodes), cleanup: () => {
          for (const c of __c28)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b16 = (anchor) => bindFor({
    anchor,
    items: () => $scope.agentsSummaries().value.items,
    key: (ag, __i) => {
      const __child = Object.create($scope);
      __child.ag = ag;
      __child.__i = __i;
      return (($scope2) => ag.id)(__child);
    },
    render: (ag, __i) => {
      const __childScope66 = Object.create($scope);
      __childScope66.ag = ag;
      __childScope66.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b19 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.ag.description)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag73 = document.createDocumentFragment();
              const __c74 = [];
              const __e75 = document.createElement("div");
              __e75.setAttribute("class", "canvas-agent-desc");
              const __t76 = document.createTextNode("");
              __e75.appendChild(__t76);
              __t76.textContent = $scope2.ag.description;
              __frag73.appendChild(__e75);
              return { nodes: Array.from(__frag73.childNodes), cleanup: () => {
                for (const c of __c74)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag67 = document.createDocumentFragment();
        const __c68 = [];
        const __e69 = document.createElement("button");
        __e69.setAttribute("type", "button");
        __e69.setAttribute("class", "canvas-agent-card");
        __c68.push(on(__e69, "click", function() {
          const __h = (e) => {
            e.preventDefault();
            $scope2.pair($scope2.ag.id);
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e70 = document.createElement("div");
        __e70.setAttribute("class", "canvas-agent-name");
        const __t71 = document.createTextNode("");
        __e70.appendChild(__t71);
        __t71.textContent = $scope2.ag.name || $scope2.ag.id;
        __e69.appendChild(__e70);
        const __ifAnchor72 = document.createComment("if");
        __e69.appendChild(__ifAnchor72);
        __c68.push(__wireBoundary_b19(__ifAnchor72));
        __frag67.appendChild(__e69);
        return { nodes: Array.from(__frag67.childNodes), cleanup: () => {
          for (const c of __c68)
            try {
              c();
            } catch (e) {}
        }, update: (__new77, __newI78) => {
          const __old = $scope2.ag;
          $scope2.ag = __new77;
          $scope2.__i = __newI78;
          if (__old.id !== $scope2.ag.id || __old.name !== $scope2.ag.name) {
            __t71.textContent = $scope2.ag.name || $scope2.ag.id;
          }
        } };
      })(__childScope66);
    }
  });
  const __wireBoundary_b15 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.agentsSummaries().kind === "ok" && $scope.agentsSummaries().value && $scope.agentsSummaries().value.items && $scope.agentsSummaries().value.items.length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag63 = document.createDocumentFragment();
        const __c64 = [];
        const __forAnchor65 = document.createComment("for");
        __frag63.appendChild(__forAnchor65);
        __c64.push(__wireBoundary_b16(__forAnchor65));
        return { nodes: Array.from(__frag63.childNodes), cleanup: () => {
          for (const c of __c64)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag79 = document.createDocumentFragment();
        const __c80 = [];
        const __e81 = document.createElement("div");
        __e81.setAttribute("class", "canvas-agent-empty");
        const __t82 = document.createTextNode("No agents installed.");
        __e81.appendChild(__t82);
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
  const __wireBoundary_b35 = (anchor) => bindFor({
    anchor,
    items: () => $scope.annThread(),
    key: (ann, __i) => {
      const __child = Object.create($scope);
      __child.ann = ann;
      __child.__i = __i;
      return (($scope2) => ann.id)(__child);
    },
    render: (ann, __i) => {
      const __childScope113 = Object.create($scope);
      __childScope113.ann = ann;
      __childScope113.__i = __i;
      return (($scope2) => {
        __childScope113.__v_note = signal2(0);
        const __frag114 = document.createDocumentFragment();
        const __c115 = [];
        const __e116 = document.createElement("div");
        __e116.setAttribute("class", "canvas-rail-thread-item");
        const __e117 = document.createElement("div");
        __e117.setAttribute("class", "canvas-rail-thread-kind");
        const __t118 = document.createTextNode("");
        __e117.appendChild(__t118);
        __t118.textContent = $scope2.ann.kind;
        __e116.appendChild(__e117);
        const __e119 = document.createElement("div");
        __e119.setAttribute("class", "canvas-rail-thread-note");
        const __t120 = document.createTextNode("");
        __e119.appendChild(__t120);
        __c115.push(bindText(__t120, () => (void $scope2.__v_note(), $scope2.ann.note || "(no note)")));
        __e116.appendChild(__e119);
        __frag114.appendChild(__e116);
        return { nodes: Array.from(__frag114.childNodes), cleanup: () => {
          for (const c of __c115)
            try {
              c();
            } catch (e) {}
        }, update: (__new121, __newI122) => {
          const __old = $scope2.ann;
          $scope2.ann = __new121;
          $scope2.__i = __newI122;
          if (__old.kind !== $scope2.ann.kind) {
            __t118.textContent = $scope2.ann.kind;
          }
          if (__old.note !== $scope2.ann.note) {
            $scope2.__v_note.set((c) => c + 1);
          }
        } };
      })(__childScope113);
    }
  });
  const __wireBoundary_b34 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.annCount() === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag105 = document.createDocumentFragment();
        const __c106 = [];
        const __e107 = document.createElement("div");
        __e107.setAttribute("class", "canvas-rail-empty");
        const __t108 = document.createTextNode("No annotations queued.");
        __e107.appendChild(__t108);
        __frag105.appendChild(__e107);
        return { nodes: Array.from(__frag105.childNodes), cleanup: () => {
          for (const c of __c106)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag109 = document.createDocumentFragment();
        const __c110 = [];
        const __e111 = document.createElement("sui-stack");
        __e111.setAttribute("gap", "2");
        const __forAnchor112 = document.createComment("for");
        __e111.appendChild(__forAnchor112);
        __c110.push(__wireBoundary_b35(__forAnchor112));
        __frag109.appendChild(__e111);
        return { nodes: Array.from(__frag109.childNodes), cleanup: () => {
          for (const c of __c110)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b14 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (!$scope.pairedAgentId())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag53 = document.createDocumentFragment();
        const __c54 = [];
        const __e55 = document.createElement("div");
        __e55.setAttribute("class", "canvas-empty-state");
        const __e56 = document.createElement("sui-icon");
        __e56.setAttribute("name", "users");
        __e56.setAttribute("aria-hidden", "true");
        __e55.appendChild(__e56);
        const __e57 = document.createElement("div");
        __e57.setAttribute("class", "canvas-empty-title");
        const __t58 = document.createTextNode("Pair with an agent to begin");
        __e57.appendChild(__t58);
        __e55.appendChild(__e57);
        const __e59 = document.createElement("div");
        __e59.setAttribute("class", "canvas-empty-desc");
        const __t60 = document.createTextNode(`
          Pick an agent to author content into this canvas. The agent emits SUI markup;
          you annotate and send feedback; the agent applies morph patches in place.
        `);
        __e59.appendChild(__t60);
        __e55.appendChild(__e59);
        const __e61 = document.createElement("div");
        __e61.setAttribute("class", "canvas-agent-picker");
        const __ifAnchor62 = document.createComment("if");
        __e61.appendChild(__ifAnchor62);
        __c54.push(__wireBoundary_b15(__ifAnchor62));
        __e55.appendChild(__e61);
        __frag53.appendChild(__e55);
        return { nodes: Array.from(__frag53.childNodes), cleanup: () => {
          for (const c of __c54)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag83 = document.createDocumentFragment();
        const __c84 = [];
        const __e85 = document.createElement("div");
        __e85.setAttribute("class", "canvas-paired");
        const __e86 = document.createElement("sui-annotation-layer");
        __c84.push(bindAttr(__e86, "view-id", () => $scope.canvasId()));
        __c84.push(bindAttr(__e86, "mode", () => $scope.annMode()));
        __e86.setAttribute("staging", "");
        attachRef($scope, "annLayerEl", __e86);
        __e86.setAttribute("class", "canvas-annotation-layer");
        const __e87 = document.createElement("sui-canvas");
        attachRef($scope, "canvasEl", __e87);
        __c84.push(bindAttr(__e87, "canvas-id", () => $scope.canvasId()));
        __c84.push(bindAttr(__e87, "agent-id", () => $scope.pairedAgentId()));
        __e87.setAttribute("full-height", "");
        __e86.appendChild(__e87);
        __e85.appendChild(__e86);
        const __e88 = document.createElement("aside");
        __e88.setAttribute("class", "canvas-side-rail");
        const __e89 = document.createElement("div");
        __e89.setAttribute("class", "canvas-rail-section");
        const __e90 = document.createElement("div");
        __e90.setAttribute("class", "canvas-rail-section-title");
        const __t91 = document.createTextNode("Annotate");
        __e90.appendChild(__t91);
        __e89.appendChild(__e90);
        const __e92 = document.createElement("sui-cluster");
        __e92.setAttribute("gap", "1");
        const __e93 = document.createElement("button");
        __e93.setAttribute("type", "button");
        __e93.setAttribute("class", "canvas-mode-btn");
        __c84.push(bindClass(__e93, "active", () => $scope.annMode() === "point"));
        __c84.push(on(__e93, "click", function() {
          const __h = () => $scope.setAnnMode("point");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t94 = document.createTextNode("Point");
        __e93.appendChild(__t94);
        __e92.appendChild(__e93);
        const __e95 = document.createElement("button");
        __e95.setAttribute("type", "button");
        __e95.setAttribute("class", "canvas-mode-btn");
        __c84.push(bindClass(__e95, "active", () => $scope.annMode() === "region"));
        __c84.push(on(__e95, "click", function() {
          const __h = () => $scope.setAnnMode("region");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t96 = document.createTextNode("Region");
        __e95.appendChild(__t96);
        __e92.appendChild(__e95);
        const __e97 = document.createElement("button");
        __e97.setAttribute("type", "button");
        __e97.setAttribute("class", "canvas-mode-btn");
        __c84.push(bindClass(__e97, "active", () => $scope.annMode() === "off"));
        __c84.push(on(__e97, "click", function() {
          const __h = () => $scope.setAnnMode("off");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t98 = document.createTextNode("Off");
        __e97.appendChild(__t98);
        __e92.appendChild(__e97);
        __e89.appendChild(__e92);
        __e88.appendChild(__e89);
        const __e99 = document.createElement("div");
        __e99.setAttribute("class", "canvas-rail-section canvas-rail-thread");
        const __e100 = document.createElement("div");
        __e100.setAttribute("class", "canvas-rail-section-title");
        const __t101 = document.createTextNode(`
              Thread `);
        __e100.appendChild(__t101);
        const __e102 = document.createElement("span");
        __e102.setAttribute("class", "canvas-rail-count");
        const __t103 = document.createTextNode("");
        __e102.appendChild(__t103);
        __c84.push(bindText(__t103, () => $scope.annCount()));
        __e100.appendChild(__e102);
        __e99.appendChild(__e100);
        const __ifAnchor104 = document.createComment("if");
        __e99.appendChild(__ifAnchor104);
        __c84.push(__wireBoundary_b34(__ifAnchor104));
        __e88.appendChild(__e99);
        const __e123 = document.createElement("div");
        __e123.setAttribute("class", "canvas-rail-footer");
        const __e124 = document.createElement("button");
        __e124.setAttribute("type", "button");
        __e124.setAttribute("class", "canvas-btn");
        __c84.push(bindAttr(__e124, "disabled", () => $scope.annCount() === 0 ? "" : null));
        __c84.push(on(__e124, "click", function() {
          const __h = $scope.clearAnnotations;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t125 = document.createTextNode("Clear");
        __e124.appendChild(__t125);
        __e123.appendChild(__e124);
        const __e126 = document.createElement("button");
        __e126.setAttribute("type", "button");
        __e126.setAttribute("class", "canvas-btn canvas-btn-primary");
        __c84.push(bindAttr(__e126, "disabled", () => $scope.annCount() === 0 ? "" : null));
        __c84.push(on(__e126, "click", function() {
          const __h = $scope.sendAnnotations;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t127 = document.createTextNode("Send to agent");
        __e126.appendChild(__t127);
        __e123.appendChild(__e126);
        __e88.appendChild(__e123);
        __e85.appendChild(__e88);
        __frag83.appendChild(__e85);
        return { nodes: Array.from(__frag83.childNodes), cleanup: () => {
          for (const c of __c84)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b43 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.pairedAgentId())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag134 = document.createDocumentFragment();
        const __c135 = [];
        const __e136 = document.createElement("span");
        __e136.setAttribute("class", "canvas-footer-sep");
        const __t137 = document.createTextNode("·");
        __e136.appendChild(__t137);
        __frag134.appendChild(__e136);
        const __t138 = document.createTextNode(" agent ");
        __frag134.appendChild(__t138);
        const __e139 = document.createElement("code");
        const __t140 = document.createTextNode("");
        __e139.appendChild(__t140);
        __c135.push(bindText(__t140, () => $scope.pairedAgentLabel()));
        __frag134.appendChild(__e139);
        return { nodes: Array.from(__frag134.childNodes), cleanup: () => {
          for (const c of __c135)
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
      __e0.setAttribute("class", "ship-view canvas-shell");
      const __e1 = document.createElement("header");
      __e1.setAttribute("class", "canvas-header");
      const __e2 = document.createElement("sui-cluster");
      __e2.setAttribute("gap", "2");
      __e2.setAttribute("align", "center");
      const __e3 = document.createElement("sui-icon");
      __e3.setAttribute("name", "image");
      __e3.setAttribute("aria-hidden", "true");
      __e2.appendChild(__e3);
      const __e4 = document.createElement("h1");
      const __t5 = document.createTextNode("Canvas");
      __e4.appendChild(__t5);
      __e2.appendChild(__e4);
      const __e6 = document.createElement("span");
      __e6.setAttribute("class", "canvas-id-chip");
      __e6.setAttribute("title", "canvas session id");
      const __t7 = document.createTextNode("");
      __e6.appendChild(__t7);
      __cleanups.push(bindText(__t7, () => $scope.shortId()));
      __e2.appendChild(__e6);
      __e1.appendChild(__e2);
      const __e8 = document.createElement("sui-cluster");
      __e8.setAttribute("gap", "2");
      __e8.setAttribute("align", "center");
      const __ifAnchor9 = document.createComment("if");
      __e8.appendChild(__ifAnchor9);
      __cleanups.push(__wireBoundary_b1(__ifAnchor9));
      const __e17 = document.createElement("button");
      __e17.setAttribute("type", "button");
      __e17.setAttribute("class", "canvas-btn");
      __cleanups.push(on(__e17, "click", function() {
        const __h = $scope.toggleRecent;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __cleanups.push(bindAttr(__e17, "aria-pressed", () => $scope.recentOpen()));
      const __e18 = document.createElement("sui-icon");
      __e18.setAttribute("name", "history");
      __e18.setAttribute("aria-hidden", "true");
      __e17.appendChild(__e18);
      const __t19 = document.createTextNode(`
        Recent
      `);
      __e17.appendChild(__t19);
      __e8.appendChild(__e17);
      const __e20 = document.createElement("button");
      __e20.setAttribute("type", "button");
      __e20.setAttribute("class", "canvas-btn");
      __cleanups.push(on(__e20, "click", function() {
        const __h = $scope.newCanvas;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e21 = document.createElement("sui-icon");
      __e21.setAttribute("name", "plus");
      __e21.setAttribute("aria-hidden", "true");
      __e20.appendChild(__e21);
      const __t22 = document.createTextNode(`
        New
      `);
      __e20.appendChild(__t22);
      __e8.appendChild(__e20);
      const __e23 = document.createElement("button");
      __e23.setAttribute("type", "button");
      __e23.setAttribute("class", "canvas-btn canvas-btn-primary");
      __cleanups.push(on(__e23, "click", function() {
        const __h = $scope.saveCanvas;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e24 = document.createElement("sui-icon");
      __e24.setAttribute("name", "save");
      __e24.setAttribute("aria-hidden", "true");
      __e23.appendChild(__e24);
      const __t25 = document.createTextNode(`
        Save
      `);
      __e23.appendChild(__t25);
      __e8.appendChild(__e23);
      __e1.appendChild(__e8);
      const __ifAnchor26 = document.createComment("if");
      __e1.appendChild(__ifAnchor26);
      __cleanups.push(__wireBoundary_b8(__ifAnchor26));
      __e0.appendChild(__e1);
      const __e51 = document.createElement("main");
      __e51.setAttribute("class", "canvas-main");
      const __ifAnchor52 = document.createComment("if");
      __e51.appendChild(__ifAnchor52);
      __cleanups.push(__wireBoundary_b14(__ifAnchor52));
      __e0.appendChild(__e51);
      const __e128 = document.createElement("footer");
      __e128.setAttribute("class", "canvas-footer");
      const __e129 = document.createElement("span");
      __e129.setAttribute("class", "canvas-footer-info");
      const __t130 = document.createTextNode(`
      canvas `);
      __e129.appendChild(__t130);
      const __e131 = document.createElement("code");
      const __t132 = document.createTextNode("");
      __e131.appendChild(__t132);
      __cleanups.push(bindText(__t132, () => $scope.shortId()));
      __e129.appendChild(__e131);
      const __ifAnchor133 = document.createComment("if");
      __e129.appendChild(__ifAnchor133);
      __cleanups.push(__wireBoundary_b43(__ifAnchor133));
      __e128.appendChild(__e129);
      const __e141 = document.createElement("span");
      __e141.setAttribute("class", "canvas-footer-info");
      const __t142 = document.createTextNode("");
      __e141.appendChild(__t142);
      __cleanups.push(bindText(__t142, () => $scope.statusText()));
      __e128.appendChild(__e141);
      __e0.appendChild(__e128);
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
        __cleanups.push(hydrateText(__t["b0"], () => $scope.shortId()));
      if (__m["b1"]) {
        const __t2 = __m["b1"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b1(__anchor));
      }
      if (__m["b4"])
        __cleanups.push(hydrateOn(__m["b4"], "click", function() {
          const __h = $scope.toggleRecent;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b5"])
        __cleanups.push(hydrateAttr(__m["b5"], "aria-pressed", () => $scope.recentOpen()));
      if (__m["b6"])
        __cleanups.push(hydrateOn(__m["b6"], "click", function() {
          const __h = $scope.newCanvas;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b7"])
        __cleanups.push(hydrateOn(__m["b7"], "click", function() {
          const __h = $scope.saveCanvas;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b8"]) {
        const __t2 = __m["b8"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b8(__anchor));
      }
      if (__m["b14"]) {
        const __t2 = __m["b14"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b14(__anchor));
      }
      if (__t["b42"])
        __cleanups.push(hydrateText(__t["b42"], () => $scope.shortId()));
      if (__m["b43"]) {
        const __t2 = __m["b43"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b43(__anchor));
      }
      if (__t["b45"])
        __cleanups.push(hydrateText(__t["b45"], () => $scope.statusText()));
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
