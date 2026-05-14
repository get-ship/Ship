// views/agents/page.sui
import { bindAttr, bindFor, bindIf, bindText, bindWhileError, bindWhileLoading, data, live as live2, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateAttr, hydrateOn, hydrateText, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/agents/controller.ts
import { controller, signal, unwrap, live, derived } from "@ship/sui-runtime";
var PROVIDER_MODELS = {
  claude: ["claude-opus-4-7", "claude-sonnet-4-6", "claude-haiku-4-5-20251001"],
  codex: ["o4-mini", "o4"],
  cursor: ["gpt-5", "gpt-5-mini"],
  gemini: ["gemini-2.5-pro", "gemini-2.5-flash"],
  opencode: ["opencode-default"]
};
var slugify = (s) => s.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
var controller_default = controller({
  scope: ({ ship }) => {
    const agentEvents = live(() => ship.events.list({ event_type: "agent.", limit: 50 }));
    const agentStatus = derived(() => {
      const env = agentEvents();
      if (env.kind !== "ok")
        return {};
      const v = env.value;
      const rows = (v?.groups ?? []).flatMap((g) => g.rows ?? []);
      rows.sort((a, b) => a.timestamp.localeCompare(b.timestamp));
      const map = {};
      for (const r of rows) {
        if (!r.actor)
          continue;
        if (r.event_type === "agent.started")
          map[r.actor] = "running";
        else if (r.event_type === "agent.completed")
          map[r.actor] = "completed";
        else if (r.event_type === "agent.error" || r.event_type === "agent.errored" || r.event_type === "agent.failed")
          map[r.actor] = "errored";
      }
      return map;
    });
    const dialogOpen = signal(false);
    const formName = signal("");
    const formId = signal("");
    const formIdTouched = signal(false);
    const formProvider = signal("claude");
    const formModel = signal("");
    const submitting = signal(false);
    const formErr = signal("");
    const modelOptions = derived(() => PROVIDER_MODELS[formProvider()] ?? []);
    const canSubmit = derived(() => !submitting() && formName().trim().length > 0 && /^[a-z][a-z0-9-]*$/.test(formId()) && formProvider().length > 0);
    return {
      agentEvents,
      agentStatus,
      dialogOpen,
      formName,
      formId,
      formIdTouched,
      formProvider,
      formModel,
      submitting,
      formErr,
      modelOptions,
      canSubmit
    };
  },
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    total: ({ agents }) => unwrap(agents)?.items.length ?? 0
  },
  on: {
    openNew: (s) => {
      s.dialogOpen = true;
      s.formName = "";
      s.formId = "";
      s.formIdTouched = false;
      s.formProvider = "claude";
      s.formModel = "";
      s.formErr = "";
    },
    closeNew: (s) => {
      s.dialogOpen = false;
    },
    nameInput: (s, e) => {
      const v = e.detail.value;
      s.formName = v;
      if (!s.formIdTouched)
        s.formId = slugify(v);
    },
    idInput: (s, e) => {
      s.formId = e.detail.value;
      s.formIdTouched = true;
    },
    providerChange: (s, e) => {
      s.formProvider = e.detail.value;
      s.formModel = "";
    },
    modelChange: (s, e) => {
      s.formModel = e.detail.value;
    },
    submitNew: async (s) => {
      if (!s.canSubmit)
        return;
      s.submitting = true;
      s.formErr = "";
      const id = String(s.formId);
      const name = String(s.formName).trim();
      const provider = String(s.formProvider);
      const model = String(s.formModel).trim();
      const doc = {
        $schema: "../../schemas/agent.schema.json",
        schema_version: "1",
        agent: { id, name, providers: [provider], ...model ? { model } : {} },
        permissions: { preset: "standard" }
      };
      try {
        const resp = await fetch("/api/v1/agents", {
          method: "POST",
          credentials: "include",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ id, doc })
        });
        if (!resp.ok) {
          const body = await resp.json().catch(() => ({}));
          s.formErr = body.message ?? `HTTP ${resp.status}`;
          s.submitting = false;
          return;
        }
        s.dialogOpen = false;
        window.location.reload();
      } catch (err) {
        s.formErr = "Network: " + (err instanceof Error ? err.message : String(err));
        s.submitting = false;
      }
    },
    openSession: async (_s, agentId, provider, evt) => {
      if (evt) {
        evt.preventDefault();
        evt.stopPropagation();
      }
      try {
        const resp = await fetch("/api/v1/sessions/spawn", {
          method: "POST",
          credentials: "include",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ workspace_id: "main", kind: "agent", agent_id: agentId, provider })
        });
        if (!resp.ok)
          return;
        window.location.href = "/sessions";
      } catch {}
    }
  }
});

// views/agents/page.sui
var page_default = view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.agents = live2(() => ship.agents.summaries());
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b4 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1, 2, 3, 4, 5],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope17 = Object.create($scope);
      __childScope17.n = n;
      __childScope17.__i = __i;
      return (($scope2) => {
        const __frag18 = document.createDocumentFragment();
        const __c19 = [];
        const __e20 = document.createElement("div");
        __e20.setAttribute("class", "agents-skel-card");
        __e20.setAttribute("aria-hidden", "true");
        const __e21 = document.createElement("sui-skeleton");
        __e21.setAttribute("shape", "block");
        __e21.setAttribute("size", "md");
        __e21.setAttribute("class", "agents-skel-icon");
        __e20.appendChild(__e21);
        const __e22 = document.createElement("div");
        __e22.setAttribute("class", "agents-skel-body");
        const __e23 = document.createElement("sui-skeleton");
        __e23.setAttribute("shape", "line");
        __e23.setAttribute("size", "md");
        __e23.setAttribute("style", "width:70%");
        __e22.appendChild(__e23);
        const __e24 = document.createElement("sui-skeleton");
        __e24.setAttribute("shape", "line");
        __e24.setAttribute("size", "sm");
        __e24.setAttribute("style", "width:95%");
        __e22.appendChild(__e24);
        const __e25 = document.createElement("sui-skeleton");
        __e25.setAttribute("shape", "line");
        __e25.setAttribute("size", "sm");
        __e25.setAttribute("style", "width:55%");
        __e22.appendChild(__e25);
        __e20.appendChild(__e22);
        __frag18.appendChild(__e20);
        return { nodes: Array.from(__frag18.childNodes), cleanup: () => {
          for (const c of __c19)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope17);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag13 = document.createDocumentFragment();
    const __c14 = [];
    const __e15 = document.createElement("div");
    __e15.setAttribute("class", "agents-grid");
    __e15.setAttribute("data-test", "loading");
    __e15.setAttribute("aria-label", "Loading agents");
    const __forAnchor16 = document.createComment("for");
    __e15.appendChild(__forAnchor16);
    __c14.push(__wireBoundary_b4(__forAnchor16));
    __frag13.appendChild(__e15);
    return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
      for (const c of __c14)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope27 = Object.create($scope);
    __childScope27.err = err;
    return (($scope2) => {
      const __frag28 = document.createDocumentFragment();
      const __c29 = [];
      const __e30 = document.createElement("sui-error-state");
      __e30.setAttribute("title", "Couldn't load agents");
      __e30.setAttribute("data-test", "error");
      const __t31 = document.createTextNode("");
      __e30.appendChild(__t31);
      __c29.push(bindText(__t31, () => $scope2.err.message));
      const __e32 = document.createElement("sui-button");
      __e32.setAttribute("slot", "action");
      __e32.setAttribute("tone", "primary");
      __e32.setAttribute("variant", "solid");
      __c29.push(on(__e32, "click", function() {
        const __h = () => window.location.reload();
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t33 = document.createTextNode("Retry");
      __e32.appendChild(__t33);
      __e30.appendChild(__e32);
      __frag28.appendChild(__e30);
      return { nodes: Array.from(__frag28.childNodes), cleanup: () => {
        for (const c of __c29)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope27);
  };
  const __wireBoundary_b11 = (anchor) => bindFor({
    anchor,
    items: () => $scope.items(),
    key: (a, __i) => {
      const __child = Object.create($scope);
      __child.a = a;
      __child.__i = __i;
      return (($scope2) => a.id)(__child);
    },
    render: (a, __i) => {
      const __childScope55 = Object.create($scope);
      __childScope55.a = a;
      __childScope55.__i = __i;
      __childScope55.__v = signal2(0);
      return (($scope2) => {
        const __wireBoundary_b22 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.a.providers.slice(0, 4),
          render: (p, __i2) => {
            const __childScope90 = Object.create($scope2);
            __childScope90.p = p;
            __childScope90.__i = __i2;
            __childScope90.__v = signal2(0);
            return (($scope3) => {
              const __frag91 = document.createDocumentFragment();
              const __c92 = [];
              const __e93 = document.createElement("sui-chip");
              __e93.setAttribute("tone", "muted");
              __e93.setAttribute("variant", "outline");
              __e93.setAttribute("size", "sm");
              {
                const __av94 = $scope3.p;
                if (__av94 == null || __av94 === false || __av94 === "")
                  __e93.removeAttribute("title");
                else
                  __e93.setAttribute("title", __av94 === true ? "" : String(__av94));
              }
              const __t95 = document.createTextNode("");
              __e93.appendChild(__t95);
              __t95.textContent = $scope3.p;
              __frag91.appendChild(__e93);
              return { nodes: Array.from(__frag91.childNodes), cleanup: () => {
                for (const c of __c92)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new96, __newI97) => {
                $scope3.p = __new96;
                $scope3.__i = __newI97;
                {
                  const __av94 = $scope3.p;
                  if (__av94 == null || __av94 === false || __av94 === "")
                    __e93.removeAttribute("title");
                  else
                    __e93.setAttribute("title", __av94 === true ? "" : String(__av94));
                }
                __t95.textContent = $scope3.p;
                $scope3.__v.set((c) => c + 1);
              } };
            })(__childScope90);
          }
        });
        const __wireBoundary_b25 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.providers.length > 4)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag99 = document.createDocumentFragment();
              const __c100 = [];
              const __e101 = document.createElement("sui-chip");
              __e101.setAttribute("tone", "muted");
              __e101.setAttribute("variant", "outline");
              __e101.setAttribute("size", "sm");
              const __t102 = document.createTextNode("+");
              __e101.appendChild(__t102);
              const __t103 = document.createTextNode("");
              __e101.appendChild(__t103);
              __t103.textContent = $scope2.a.providers.length - 4;
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
        const __wireBoundary_b21 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.providers && $scope2.a.providers.length > 0)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag86 = document.createDocumentFragment();
              const __c87 = [];
              const __e88 = document.createElement("div");
              __e88.setAttribute("class", "agents-card-providers");
              __e88.setAttribute("aria-label", "Compiles to");
              const __forAnchor89 = document.createComment("for");
              __e88.appendChild(__forAnchor89);
              __c87.push(__wireBoundary_b22(__forAnchor89));
              const __ifAnchor98 = document.createComment("if");
              __e88.appendChild(__ifAnchor98);
              __c87.push(__wireBoundary_b25(__ifAnchor98));
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
        const __wireBoundary_b35 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.a.active)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag116 = document.createDocumentFragment();
              const __c117 = [];
              const __e118 = document.createElement("sui-chip");
              __e118.setAttribute("tone", "primary");
              __e118.setAttribute("variant", "soft");
              __e118.setAttribute("size", "sm");
              const __e119 = document.createElement("sui-status-dot");
              __e119.setAttribute("slot", "leading");
              __e119.setAttribute("tone", "primary");
              __e119.setAttribute("size", "sm");
              __e119.setAttribute("pulse", "");
              __e119.setAttribute("label", "Active session");
              __e118.appendChild(__e119);
              const __t120 = document.createTextNode(`
                        active session
                      `);
              __e118.appendChild(__t120);
              __frag116.appendChild(__e118);
              return { nodes: Array.from(__frag116.childNodes), cleanup: () => {
                for (const c of __c117)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b28 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.agentStatus()[$scope2.a.id])
              return 0;
            return 1;
          },
          branches: [
            () => {
              const __frag108 = document.createDocumentFragment();
              const __c109 = [];
              const __e110 = document.createElement("sui-chip");
              __c109.push(bindAttr(__e110, "tone", () => (void $scope2.__v(), $scope2.agentStatus()[$scope2.a.id] === "errored" ? "destructive" : $scope2.agentStatus()[$scope2.a.id] === "completed" ? "success" : "primary")));
              __e110.setAttribute("variant", "soft");
              __e110.setAttribute("size", "sm");
              __e110.setAttribute("data-test", "live-status");
              __c109.push(bindAttr(__e110, "data-status", () => (void $scope2.__v(), $scope2.agentStatus()[$scope2.a.id])));
              const __e111 = document.createElement("sui-status-dot");
              __e111.setAttribute("slot", "leading");
              __c109.push(bindAttr(__e111, "tone", () => (void $scope2.__v(), $scope2.agentStatus()[$scope2.a.id] === "errored" ? "destructive" : $scope2.agentStatus()[$scope2.a.id] === "completed" ? "success" : "primary")));
              __e111.setAttribute("size", "sm");
              __c109.push(bindAttr(__e111, "pulse", () => (void $scope2.__v(), $scope2.agentStatus()[$scope2.a.id] === "running" ? "" : null)));
              __c109.push(bindAttr(__e111, "label", () => (void $scope2.__v(), $scope2.agentStatus()[$scope2.a.id])));
              __e110.appendChild(__e111);
              const __t112 = document.createTextNode("");
              __e110.appendChild(__t112);
              __c109.push(bindText(__t112, () => (void $scope2.__v(), $scope2.agentStatus()[$scope2.a.id])));
              __frag108.appendChild(__e110);
              return { nodes: Array.from(__frag108.childNodes), cleanup: () => {
                for (const c of __c109)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag113 = document.createDocumentFragment();
              const __c114 = [];
              const __ifAnchor115 = document.createComment("if");
              __frag113.appendChild(__ifAnchor115);
              __c114.push(__wireBoundary_b35(__ifAnchor115));
              return { nodes: Array.from(__frag113.childNodes), cleanup: () => {
                for (const c of __c114)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b36 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => ($scope2.a.tags || []).slice(0, 2),
          render: (t, __i2) => {
            const __childScope122 = Object.create($scope2);
            __childScope122.t = t;
            __childScope122.__i = __i2;
            __childScope122.__v = signal2(0);
            return (($scope3) => {
              const __frag123 = document.createDocumentFragment();
              const __c124 = [];
              const __e125 = document.createElement("sui-chip");
              __e125.setAttribute("tone", "muted");
              __e125.setAttribute("variant", "soft");
              __e125.setAttribute("size", "sm");
              const __t126 = document.createTextNode("");
              __e125.appendChild(__t126);
              __t126.textContent = $scope3.t;
              __frag123.appendChild(__e125);
              return { nodes: Array.from(__frag123.childNodes), cleanup: () => {
                for (const c of __c124)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new127, __newI128) => {
                $scope3.t = __new127;
                $scope3.__i = __newI128;
                __t126.textContent = $scope3.t;
                $scope3.__v.set((c) => c + 1);
              } };
            })(__childScope122);
          }
        });
        const __wireBoundary_b38 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if (($scope2.a.tags || []).length > 2)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag130 = document.createDocumentFragment();
              const __c131 = [];
              const __e132 = document.createElement("sui-chip");
              __e132.setAttribute("tone", "muted");
              __e132.setAttribute("variant", "outline");
              __e132.setAttribute("size", "sm");
              const __t133 = document.createTextNode("+");
              __e132.appendChild(__t133);
              const __t134 = document.createTextNode("");
              __e132.appendChild(__t134);
              __c131.push(bindText(__t134, () => (void $scope2.__v(), ($scope2.a.tags || []).length - 2)));
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
        const __frag56 = document.createDocumentFragment();
        const __c57 = [];
        const __e58 = document.createElement("sui-list-tile");
        __e58.setAttribute("variant", "card");
        __e58.setAttribute("density", "normal");
        __e58.setAttribute("interactive", "");
        {
          const __av59 = "/agents/" + $scope2.a.id;
          if (__av59 == null || __av59 === false || __av59 === "")
            __e58.removeAttribute("navigable");
          else
            __e58.setAttribute("navigable", __av59 === true ? "" : String(__av59));
        }
        __e58.setAttribute("class", "agents-card-tile");
        __e58.setAttribute("data-test", "agent");
        const __e60 = document.createElement("div");
        __e60.setAttribute("class", "agents-card-icon-tile");
        __e60.setAttribute("slot", "leading");
        const __e61 = document.createElement("span");
        __e61.setAttribute("class", "agents-card-letter");
        const __t62 = document.createTextNode("");
        __e61.appendChild(__t62);
        __t62.textContent = $scope2.a.letter;
        __e60.appendChild(__e61);
        __e58.appendChild(__e60);
        const __e63 = document.createElement("div");
        __e63.setAttribute("class", "agents-card-body");
        const __e64 = document.createElement("div");
        __e64.setAttribute("class", "agents-card-head");
        const __e65 = document.createElement("div");
        __e65.setAttribute("class", "agents-card-title");
        const __t66 = document.createTextNode("");
        __e65.appendChild(__t66);
        __t66.textContent = $scope2.a.name;
        __e64.appendChild(__e65);
        const __e67 = document.createElement("p");
        __e67.setAttribute("class", "agents-card-desc");
        const __t68 = document.createTextNode("");
        __e67.appendChild(__t68);
        __t68.textContent = $scope2.a.description || "No description";
        __e64.appendChild(__e67);
        __e63.appendChild(__e64);
        const __e69 = document.createElement("div");
        __e69.setAttribute("class", "agents-card-stats");
        const __e70 = document.createElement("span");
        const __t71 = document.createTextNode("");
        __e70.appendChild(__t71);
        __t71.textContent = $scope2.a.skills_count;
        const __t72 = document.createTextNode(" skill");
        __e70.appendChild(__t72);
        const __t73 = document.createTextNode("");
        __e70.appendChild(__t73);
        __t73.textContent = $scope2.a.skills_count === 1 ? "" : "s";
        __e69.appendChild(__e70);
        const __e74 = document.createElement("span");
        __e74.setAttribute("class", "agents-card-sep");
        const __t75 = document.createTextNode("·");
        __e74.appendChild(__t75);
        __e69.appendChild(__e74);
        const __e76 = document.createElement("span");
        const __t77 = document.createTextNode("");
        __e76.appendChild(__t77);
        __t77.textContent = $scope2.a.mcp_count;
        const __t78 = document.createTextNode(" MCP");
        __e76.appendChild(__t78);
        __e69.appendChild(__e76);
        const __e79 = document.createElement("span");
        __e79.setAttribute("class", "agents-card-sep");
        const __t80 = document.createTextNode("·");
        __e79.appendChild(__t80);
        __e69.appendChild(__e79);
        const __e81 = document.createElement("span");
        const __t82 = document.createTextNode("");
        __e81.appendChild(__t82);
        __t82.textContent = $scope2.a.rules_count;
        const __t83 = document.createTextNode(" rule");
        __e81.appendChild(__t83);
        const __t84 = document.createTextNode("");
        __e81.appendChild(__t84);
        __t84.textContent = $scope2.a.rules_count === 1 ? "" : "s";
        __e69.appendChild(__e81);
        __e63.appendChild(__e69);
        const __ifAnchor85 = document.createComment("if");
        __e63.appendChild(__ifAnchor85);
        __c57.push(__wireBoundary_b21(__ifAnchor85));
        const __e104 = document.createElement("div");
        __e104.setAttribute("class", "agents-card-badges");
        const __e105 = document.createElement("sui-badge");
        __e105.setAttribute("tone", "muted");
        __e105.setAttribute("variant", "outline");
        __e105.setAttribute("size", "sm");
        __e105.setAttribute("class", "agents-card-badge-source");
        const __t106 = document.createTextNode("");
        __e105.appendChild(__t106);
        __t106.textContent = $scope2.a.source;
        __e104.appendChild(__e105);
        const __ifAnchor107 = document.createComment("if");
        __e104.appendChild(__ifAnchor107);
        __c57.push(__wireBoundary_b28(__ifAnchor107));
        const __forAnchor121 = document.createComment("for");
        __e104.appendChild(__forAnchor121);
        __c57.push(__wireBoundary_b36(__forAnchor121));
        const __ifAnchor129 = document.createComment("if");
        __e104.appendChild(__ifAnchor129);
        __c57.push(__wireBoundary_b38(__ifAnchor129));
        __e63.appendChild(__e104);
        __e58.appendChild(__e63);
        const __e135 = document.createElement("sui-button");
        __e135.setAttribute("slot", "trailing");
        __e135.setAttribute("variant", "ghost");
        __e135.setAttribute("size", "sm");
        __e135.setAttribute("data-test", "open-session");
        {
          const __av136 = $scope2.a.id;
          if (__av136 == null || __av136 === false || __av136 === "")
            __e135.removeAttribute("data-agent");
          else
            __e135.setAttribute("data-agent", __av136 === true ? "" : String(__av136));
        }
        __c57.push(on(__e135, "click", function() {
          const __h = (e) => $scope2.openSession($scope2.a.id, $scope2.a.providers && $scope2.a.providers[0] || "claude", e);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e137 = document.createElement("sui-icon");
        __e137.setAttribute("slot", "leading");
        __e137.setAttribute("name", "play");
        __e137.setAttribute("size", "xs");
        __e135.appendChild(__e137);
        const __t138 = document.createTextNode(`
                Open in session
              `);
        __e135.appendChild(__t138);
        __e58.appendChild(__e135);
        __frag56.appendChild(__e58);
        return { nodes: Array.from(__frag56.childNodes), cleanup: () => {
          for (const c of __c57)
            try {
              c();
            } catch (e) {}
        }, update: (__new139, __newI140) => {
          $scope2.a = __new139;
          $scope2.__i = __newI140;
          {
            const __av59 = "/agents/" + $scope2.a.id;
            if (__av59 == null || __av59 === false || __av59 === "")
              __e58.removeAttribute("navigable");
            else
              __e58.setAttribute("navigable", __av59 === true ? "" : String(__av59));
          }
          __t62.textContent = $scope2.a.letter;
          __t66.textContent = $scope2.a.name;
          __t68.textContent = $scope2.a.description || "No description";
          __t71.textContent = $scope2.a.skills_count;
          __t73.textContent = $scope2.a.skills_count === 1 ? "" : "s";
          __t77.textContent = $scope2.a.mcp_count;
          __t82.textContent = $scope2.a.rules_count;
          __t84.textContent = $scope2.a.rules_count === 1 ? "" : "s";
          __t106.textContent = $scope2.a.source;
          {
            const __av136 = $scope2.a.id;
            if (__av136 == null || __av136 === false || __av136 === "")
              __e135.removeAttribute("data-agent");
            else
              __e135.setAttribute("data-agent", __av136 === true ? "" : String(__av136));
          }
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope55);
    }
  });
  const __wireBoundary_b9 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.items().length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag39 = document.createDocumentFragment();
        const __c40 = [];
        const __e41 = document.createElement("sui-empty-state");
        __e41.setAttribute("icon", "sparkles");
        __e41.setAttribute("variant", "default");
        __e41.setAttribute("size", "sm");
        __e41.setAttribute("data-test", "empty");
        const __e42 = document.createElement("span");
        __e42.setAttribute("slot", "title");
        const __t43 = document.createTextNode("No agents yet");
        __e42.appendChild(__t43);
        __e41.appendChild(__e42);
        const __t44 = document.createTextNode(`
          Drop a JSONC file under
          `);
        __e41.appendChild(__t44);
        const __e45 = document.createElement("code");
        __e45.setAttribute("class", "agents-empty-code");
        const __t46 = document.createTextNode(".ship/agents/");
        __e45.appendChild(__t46);
        __e41.appendChild(__e45);
        const __t47 = document.createTextNode(`
          or create one from the Studio.
          `);
        __e41.appendChild(__t47);
        const __e48 = document.createElement("sui-button");
        __e48.setAttribute("slot", "action");
        __e48.setAttribute("tone", "primary");
        __e48.setAttribute("variant", "solid");
        __c40.push(on(__e48, "click", function() {
          const __h = $scope.openNew;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e48.setAttribute("data-test", "cta-create-first");
        const __e49 = document.createElement("sui-icon");
        __e49.setAttribute("slot", "leading");
        __e49.setAttribute("name", "plus");
        __e48.appendChild(__e49);
        const __t50 = document.createTextNode(`
            Create your first agent
          `);
        __e48.appendChild(__t50);
        __e41.appendChild(__e48);
        __frag39.appendChild(__e41);
        return { nodes: Array.from(__frag39.childNodes), cleanup: () => {
          for (const c of __c40)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag51 = document.createDocumentFragment();
        const __c52 = [];
        const __e53 = document.createElement("div");
        __e53.setAttribute("class", "agents-grid");
        __e53.setAttribute("data-view-enter", "");
        __e53.setAttribute("data-test", "grid");
        const __forAnchor54 = document.createComment("for");
        __e53.appendChild(__forAnchor54);
        __c52.push(__wireBoundary_b11(__forAnchor54));
        __frag51.appendChild(__e53);
        return { nodes: Array.from(__frag51.childNodes), cleanup: () => {
          for (const c of __c52)
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
      if ($scope.agents().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag36 = document.createDocumentFragment();
        const __c37 = [];
        const __ifAnchor38 = document.createComment("if");
        __frag36.appendChild(__ifAnchor38);
        __c37.push(__wireBoundary_b9(__ifAnchor38));
        return { nodes: Array.from(__frag36.childNodes), cleanup: () => {
          for (const c of __c37)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b52 = (anchor) => bindFor({
    anchor,
    items: () => $scope.modelOptions(),
    key: (m, __i) => {
      const __child = Object.create($scope);
      __child.m = m;
      __child.__i = __i;
      return (($scope2) => m)(__child);
    },
    render: (m, __i) => {
      const __childScope165 = Object.create($scope);
      __childScope165.m = m;
      __childScope165.__i = __i;
      __childScope165.__v = signal2(0);
      return (($scope2) => {
        const __frag166 = document.createDocumentFragment();
        const __c167 = [];
        const __e168 = document.createElement("option");
        {
          const __av169 = $scope2.m;
          if (__av169 == null || __av169 === false || __av169 === "")
            __e168.removeAttribute("value");
          else
            __e168.setAttribute("value", __av169 === true ? "" : String(__av169));
        }
        const __t170 = document.createTextNode("");
        __e168.appendChild(__t170);
        __t170.textContent = $scope2.m;
        __frag166.appendChild(__e168);
        return { nodes: Array.from(__frag166.childNodes), cleanup: () => {
          for (const c of __c167)
            try {
              c();
            } catch (e) {}
        }, update: (__new171, __newI172) => {
          $scope2.m = __new171;
          $scope2.__i = __newI172;
          {
            const __av169 = $scope2.m;
            if (__av169 == null || __av169 === false || __av169 === "")
              __e168.removeAttribute("value");
            else
              __e168.setAttribute("value", __av169 === true ? "" : String(__av169));
          }
          __t170.textContent = $scope2.m;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope165);
    }
  });
  const __wireBoundary_b55 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.formErr())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag174 = document.createDocumentFragment();
        const __c175 = [];
        const __e176 = document.createElement("sui-alert");
        __e176.setAttribute("tone", "destructive");
        __e176.setAttribute("compact", "");
        const __t177 = document.createTextNode("");
        __e176.appendChild(__t177);
        __c175.push(bindText(__t177, () => $scope.formErr()));
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
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("div");
      __e0.setAttribute("class", "ship-view agents-view");
      const __e1 = document.createElement("div");
      __e1.setAttribute("class", "agents-wrap");
      const __e2 = document.createElement("header");
      __e2.setAttribute("class", "agents-head");
      const __e3 = document.createElement("div");
      __e3.setAttribute("class", "agents-head-titles");
      const __e4 = document.createElement("h1");
      __e4.setAttribute("class", "agents-title");
      const __t5 = document.createTextNode("Agents");
      __e4.appendChild(__t5);
      __e3.appendChild(__e4);
      const __e6 = document.createElement("p");
      __e6.setAttribute("class", "agents-sub");
      const __t7 = document.createTextNode("");
      __e6.appendChild(__t7);
      __cleanups.push(bindText(__t7, () => $scope.total()));
      const __t8 = document.createTextNode(" agent");
      __e6.appendChild(__t8);
      const __t9 = document.createTextNode("");
      __e6.appendChild(__t9);
      __cleanups.push(bindText(__t9, () => $scope.total() === 1 ? "" : "s"));
      __e3.appendChild(__e6);
      __e2.appendChild(__e3);
      const __e10 = document.createElement("sui-button");
      __e10.setAttribute("tone", "primary");
      __e10.setAttribute("variant", "solid");
      __cleanups.push(on(__e10, "click", function() {
        const __h = $scope.openNew;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e11 = document.createElement("sui-icon");
      __e11.setAttribute("slot", "leading");
      __e11.setAttribute("name", "plus");
      __e10.appendChild(__e11);
      const __t12 = document.createTextNode(`
        New agent
      `);
      __e10.appendChild(__t12);
      __e2.appendChild(__e10);
      __e1.appendChild(__e2);
      const __wlAnchor26 = document.createComment("while-loading");
      __e1.appendChild(__wlAnchor26);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor26, signal: $scope.agents, render: __renderWhileLoading_0 }));
      const __weAnchor34 = document.createComment("while-error");
      __e1.appendChild(__weAnchor34);
      __cleanups.push(bindWhileError({ anchor: __weAnchor34, signal: $scope.agents, render: __renderWhileError_0 }));
      const __ifAnchor35 = document.createComment("if");
      __e1.appendChild(__ifAnchor35);
      __cleanups.push(__wireBoundary_b8(__ifAnchor35));
      __e0.appendChild(__e1);
      const __e141 = document.createElement("sui-dialog");
      __cleanups.push(bindAttr(__e141, "open", () => $scope.dialogOpen() ? "" : null));
      __e141.setAttribute("size", "sm");
      __cleanups.push(on(__e141, "sui-close", function() {
        const __h = $scope.closeNew;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e141.setAttribute("data-test", "new-agent-dialog");
      const __e142 = document.createElement("span");
      __e142.setAttribute("slot", "header");
      const __t143 = document.createTextNode("New agent");
      __e142.appendChild(__t143);
      __e141.appendChild(__e142);
      const __e144 = document.createElement("sui-field");
      __e144.setAttribute("label", "Name");
      __e144.setAttribute("hint", "Human-readable label.");
      const __e145 = document.createElement("sui-input");
      __cleanups.push(bindAttr(__e145, "value", () => $scope.formName()));
      __cleanups.push(on(__e145, "sui-input", function() {
        const __h = $scope.nameInput;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e145.setAttribute("placeholder", "e.g. Rust Compiler");
      __e145.setAttribute("autofocus", "");
      __e144.appendChild(__e145);
      __e141.appendChild(__e144);
      const __e146 = document.createElement("sui-field");
      __e146.setAttribute("label", "ID");
      __e146.setAttribute("hint", "Lowercase kebab-case — becomes the filename stem.");
      const __e147 = document.createElement("sui-input");
      __cleanups.push(bindAttr(__e147, "value", () => $scope.formId()));
      __cleanups.push(on(__e147, "sui-input", function() {
        const __h = $scope.idInput;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e147.setAttribute("placeholder", "rust-compiler");
      __e146.appendChild(__e147);
      __e141.appendChild(__e146);
      const __e148 = document.createElement("sui-field");
      __e148.setAttribute("label", "Provider");
      const __e149 = document.createElement("sui-select");
      __cleanups.push(bindAttr(__e149, "value", () => $scope.formProvider()));
      __cleanups.push(on(__e149, "sui-change", function() {
        const __h = $scope.providerChange;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e150 = document.createElement("option");
      __e150.setAttribute("value", "claude");
      const __t151 = document.createTextNode("claude");
      __e150.appendChild(__t151);
      __e149.appendChild(__e150);
      const __e152 = document.createElement("option");
      __e152.setAttribute("value", "codex");
      const __t153 = document.createTextNode("codex");
      __e152.appendChild(__t153);
      __e149.appendChild(__e152);
      const __e154 = document.createElement("option");
      __e154.setAttribute("value", "cursor");
      const __t155 = document.createTextNode("cursor");
      __e154.appendChild(__t155);
      __e149.appendChild(__e154);
      const __e156 = document.createElement("option");
      __e156.setAttribute("value", "gemini");
      const __t157 = document.createTextNode("gemini");
      __e156.appendChild(__t157);
      __e149.appendChild(__e156);
      const __e158 = document.createElement("option");
      __e158.setAttribute("value", "opencode");
      const __t159 = document.createTextNode("opencode");
      __e158.appendChild(__t159);
      __e149.appendChild(__e158);
      __e148.appendChild(__e149);
      __e141.appendChild(__e148);
      const __e160 = document.createElement("sui-field");
      __e160.setAttribute("label", "Model");
      __e160.setAttribute("hint", "Optional override — leave blank for the provider default.");
      const __e161 = document.createElement("sui-select");
      __cleanups.push(bindAttr(__e161, "value", () => $scope.formModel()));
      __cleanups.push(on(__e161, "sui-change", function() {
        const __h = $scope.modelChange;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e162 = document.createElement("option");
      __e162.setAttribute("value", "");
      const __t163 = document.createTextNode("(provider default)");
      __e162.appendChild(__t163);
      __e161.appendChild(__e162);
      const __forAnchor164 = document.createComment("for");
      __e161.appendChild(__forAnchor164);
      __cleanups.push(__wireBoundary_b52(__forAnchor164));
      __e160.appendChild(__e161);
      __e141.appendChild(__e160);
      const __ifAnchor173 = document.createComment("if");
      __e141.appendChild(__ifAnchor173);
      __cleanups.push(__wireBoundary_b55(__ifAnchor173));
      const __e178 = document.createElement("sui-toolbar");
      __e178.setAttribute("slot", "footer");
      const __e179 = document.createElement("sui-button");
      __e179.setAttribute("variant", "outline");
      __cleanups.push(on(__e179, "click", function() {
        const __h = $scope.closeNew;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t180 = document.createTextNode("Cancel");
      __e179.appendChild(__t180);
      __e178.appendChild(__e179);
      const __e181 = document.createElement("sui-button");
      __e181.setAttribute("tone", "primary");
      __e181.setAttribute("variant", "solid");
      __e181.setAttribute("data-test", "submit-new-agent");
      __cleanups.push(bindAttr(__e181, "loading", () => $scope.submitting() ? "" : null));
      __cleanups.push(bindAttr(__e181, "disabled", () => $scope.canSubmit() ? null : ""));
      __cleanups.push(on(__e181, "click", function() {
        const __h = $scope.submitNew;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e182 = document.createElement("sui-icon");
      __e182.setAttribute("slot", "leading");
      __e182.setAttribute("name", "plus");
      __e182.setAttribute("size", "sm");
      __e181.appendChild(__e182);
      const __t183 = document.createTextNode(`
        Create
      `);
      __e181.appendChild(__t183);
      __e178.appendChild(__e181);
      __e141.appendChild(__e178);
      __e0.appendChild(__e141);
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
        __cleanups.push(hydrateOn(__m["b2"], "click", function() {
          const __h = $scope.openNew;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b3"])
        __cleanups.push(hydrateWhileLoading(__m["b3"], "b3", () => $scope.agents(), __renderWhileLoading_0));
      if (__m["b5"])
        __cleanups.push(hydrateWhileError(__m["b5"], "b5", () => $scope.agents(), __renderWhileError_0));
      if (__m["b8"]) {
        const __t2 = __m["b8"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b8(__anchor));
      }
      if (__m["b42"])
        __cleanups.push(hydrateAttr(__m["b42"], "open", () => $scope.dialogOpen() ? "" : null));
      if (__m["b43"])
        __cleanups.push(hydrateOn(__m["b43"], "sui-close", function() {
          const __h = $scope.closeNew;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b44"])
        __cleanups.push(hydrateAttr(__m["b44"], "value", () => $scope.formName()));
      if (__m["b45"])
        __cleanups.push(hydrateOn(__m["b45"], "sui-input", function() {
          const __h = $scope.nameInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b46"])
        __cleanups.push(hydrateAttr(__m["b46"], "value", () => $scope.formId()));
      if (__m["b47"])
        __cleanups.push(hydrateOn(__m["b47"], "sui-input", function() {
          const __h = $scope.idInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b48"])
        __cleanups.push(hydrateAttr(__m["b48"], "value", () => $scope.formProvider()));
      if (__m["b49"])
        __cleanups.push(hydrateOn(__m["b49"], "sui-change", function() {
          const __h = $scope.providerChange;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b50"])
        __cleanups.push(hydrateAttr(__m["b50"], "value", () => $scope.formModel()));
      if (__m["b51"])
        __cleanups.push(hydrateOn(__m["b51"], "sui-change", function() {
          const __h = $scope.modelChange;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b52"]) {
        const __t2 = __m["b52"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("for") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b52(__anchor));
      }
      if (__m["b55"]) {
        const __t2 = __m["b55"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b55(__anchor));
      }
      if (__m["b57"])
        __cleanups.push(hydrateOn(__m["b57"], "click", function() {
          const __h = $scope.closeNew;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b58"])
        __cleanups.push(hydrateAttr(__m["b58"], "loading", () => $scope.submitting() ? "" : null));
      if (__m["b59"])
        __cleanups.push(hydrateAttr(__m["b59"], "disabled", () => $scope.canSubmit() ? null : ""));
      if (__m["b60"])
        __cleanups.push(hydrateOn(__m["b60"], "click", function() {
          const __h = $scope.submitNew;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
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
