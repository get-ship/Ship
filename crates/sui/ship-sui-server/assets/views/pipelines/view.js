// views/pipelines/page.sui
import { bindAttr, bindChecked, bindFor, bindIf, bindProp, bindText, bindValue, bindWhileError, bindWhileLoading, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/pipelines/controller.ts
import { controller, signal, derived, live } from "@ship/sui-runtime";
var EMPTY = { stats_subtitle: "", defs: [], active_runs: [], providers: [] };
var read = (envelope) => {
  const env = envelope;
  return env?.kind === "ok" && env.value ? env.value : EMPTY;
};
var controller_default = controller({
  scope: ({ ship, route }) => {
    const s = ship;
    const q = derived(() => String(route.query.q() ?? ""));
    const agent = derived(() => String(route.query.agent() ?? ""));
    const pipeline = derived(() => String(route.query.pipeline() ?? ""));
    const status = derived(() => String(route.query.status() ?? ""));
    const tab = derived(() => String(route.query.tab() ?? ""));
    const ctx = live(() => s.pipelines.summaries({
      q: q(),
      agent: agent(),
      pipeline: pipeline(),
      status: status(),
      tab: tab()
    }));
    const defs = derived(() => read(ctx()).defs.map((d) => ({
      ...d,
      success_pct: Math.floor(d.success_rate * 100)
    })));
    const activeRuns = derived(() => read(ctx()).active_runs.map((r) => ({
      ...r,
      state_attr: r.state.replace("-", ""),
      steps: r.steps?.map((step, i, arr) => ({ ...step, isLast: i === arr.length - 1 }))
    })));
    const providers = derived(() => read(ctx()).providers);
    const workspaceId = derived(() => read(ctx()).workspace_id ?? null);
    const statsSubtitle = derived(() => read(ctx()).stats_subtitle);
    const dispatchOpen = signal(false);
    const dispatchSpec = signal("");
    const dispatchPipeline = signal("");
    const dispatchProvider = signal("");
    const dispatchHeadless = signal(true);
    const newPipeOpen = signal(false);
    const newPipeName = signal("");
    const newPipeError = signal("");
    const newPipeCreating = signal(false);
    return {
      ctx,
      q,
      agent,
      pipeline,
      status,
      tab,
      defs,
      activeRuns,
      providers,
      workspaceId,
      statsSubtitle,
      dispatchOpen,
      dispatchSpec,
      dispatchPipeline,
      dispatchProvider,
      dispatchHeadless,
      newPipeOpen,
      newPipeName,
      newPipeError,
      newPipeCreating
    };
  },
  on: {
    openDispatch: (s) => {
      const ds = s;
      if (!ds.dispatchPipeline && ds.defs.length > 0) {
        ds.dispatchPipeline = ds.defs[0].name;
      }
      ds.dispatchOpen = true;
    },
    closeDispatch: (s) => {
      s.dispatchOpen = false;
    },
    dispatchRun: async (s) => {
      const ds = s;
      const name = String(ds.dispatchPipeline);
      if (!name)
        return;
      await fetch(`/api/v1/pipelines/${encodeURIComponent(name)}/dispatch`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ spec: ds.dispatchSpec, provider: ds.dispatchProvider || null, headless: ds.dispatchHeadless })
      });
      ds.dispatchOpen = false;
    },
    openNewPipe: (s) => {
      const ds = s;
      ds.newPipeOpen = true;
      ds.newPipeName = "";
      ds.newPipeError = "";
    },
    closeNewPipe: (s) => {
      s.newPipeOpen = false;
    },
    createPipeline: async (s) => {
      const ds = s;
      const ws = ds.workspaceId;
      if (!ws)
        return;
      const name = String(ds.newPipeName).trim();
      if (!/^[a-z0-9][a-z0-9-]*$/.test(name)) {
        ds.newPipeError = "Name must be lowercase kebab-case (a–z, 0–9, -).";
        return;
      }
      ds.newPipeCreating = true;
      ds.newPipeError = "";
      const content = `{
  "name": "${name}",
  "description": "",
  "steps": [
    { "id": "one", "run": "echo hello" }
  ]
}
`;
      try {
        const resp = await fetch(`/api/v1/workspaces/${encodeURIComponent(ws)}/pipelines/${encodeURIComponent(name)}`, { method: "PUT", credentials: "include", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ content }) });
        if (resp.ok) {
          window.location.href = `/pipelines/focus/${encodeURIComponent(name)}?edit=1`;
          return;
        }
        const parsed = await resp.json().catch(() => ({}));
        ds.newPipeError = parsed?.data?.error ?? `HTTP ${resp.status}`;
      } catch (err) {
        ds.newPipeError = "network: " + (err?.message ?? String(err));
      } finally {
        ds.newPipeCreating = false;
      }
    },
    stopRun: async (_s, jobId) => {
      await fetch("/api/v1/agent_sessions/end", {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ job_id: jobId })
      });
    }
  }
});

// views/pipelines/page.sui
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
  const __renderWhileLoading_0 = () => {
    const __frag0 = document.createDocumentFragment();
    const __c1 = [];
    const __e2 = document.createElement("sui-spinner");
    __e2.setAttribute("size", "md");
    __e2.setAttribute("centered", "");
    __frag0.appendChild(__e2);
    return { nodes: Array.from(__frag0.childNodes), cleanup: () => {
      for (const c of __c1)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope4 = Object.create($scope);
    __childScope4.err = err;
    return (($scope2) => {
      const __frag5 = document.createDocumentFragment();
      const __c6 = [];
      const __e7 = document.createElement("sui-alert");
      __e7.setAttribute("tone", "destructive");
      const __t8 = document.createTextNode("Failed to load pipelines: ");
      __e7.appendChild(__t8);
      const __t9 = document.createTextNode("");
      __e7.appendChild(__t9);
      __c6.push(bindText(__t9, () => $scope2.err.message));
      __frag5.appendChild(__e7);
      return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
        for (const c of __c6)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope4);
  };
  const __wireBoundary_b7 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.workspaceId())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag30 = document.createDocumentFragment();
        const __c31 = [];
        const __e32 = document.createElement("sui-button");
        __e32.setAttribute("variant", "outline");
        __e32.setAttribute("size", "sm");
        __c31.push(on(__e32, "click", function() {
          const __h = $scope.openNewPipe;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e33 = document.createElement("sui-icon");
        __e33.setAttribute("slot", "leading");
        __e33.setAttribute("name", "plus");
        __e33.setAttribute("size", "sm");
        __e32.appendChild(__e33);
        const __t34 = document.createTextNode(`New pipeline
        `);
        __e32.appendChild(__t34);
        __frag30.appendChild(__e32);
        return { nodes: Array.from(__frag30.childNodes), cleanup: () => {
          for (const c of __c31)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b12 = (anchor) => bindFor({
    anchor,
    items: () => $scope.activeRuns(),
    key: (run, __i) => {
      const __child = Object.create($scope);
      __child.run = run;
      __child.__i = __i;
      return (($scope2) => run.slug)(__child);
    },
    render: (run, __i) => {
      const __childScope50 = Object.create($scope);
      __childScope50.run = run;
      __childScope50.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b19 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.run.agent)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag67 = document.createDocumentFragment();
              const __c68 = [];
              const __e69 = document.createElement("sui-avatar");
              __e69.setAttribute("size", "sm");
              __e69.setAttribute("shape", "round");
              {
                const __av70 = $scope2.run.agent;
                if (__av70 == null || __av70 === false || __av70 === "")
                  __e69.removeAttribute("identity-hue");
                else
                  __e69.setAttribute("identity-hue", __av70 === true ? "" : String(__av70));
              }
              {
                const __av71 = $scope2.run.agent;
                if (__av71 == null || __av71 === false || __av71 === "")
                  __e69.removeAttribute("name");
                else
                  __e69.setAttribute("name", __av71 === true ? "" : String(__av71));
              }
              __frag67.appendChild(__e69);
              return { nodes: Array.from(__frag67.childNodes), cleanup: () => {
                for (const c of __c68)
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
            if ($scope2.run.branch)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag73 = document.createDocumentFragment();
              const __c74 = [];
              const __e75 = document.createElement("code");
              const __t76 = document.createTextNode("");
              __e75.appendChild(__t76);
              __t76.textContent = $scope2.run.branch;
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
        const __wireBoundary_b24 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.run.provider)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag78 = document.createDocumentFragment();
              const __c79 = [];
              const __e80 = document.createElement("sui-provider-blob");
              {
                const __av81 = $scope2.run.provider;
                if (__av81 == null || __av81 === false || __av81 === "")
                  __e80.removeAttribute("provider");
                else
                  __e80.setAttribute("provider", __av81 === true ? "" : String(__av81));
              }
              __e80.setAttribute("size", "sm");
              __e80.setAttribute("variant", "glyph");
              __frag78.appendChild(__e80);
              return { nodes: Array.from(__frag78.childNodes), cleanup: () => {
                for (const c of __c79)
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
            if ($scope2.run.model)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag83 = document.createDocumentFragment();
              const __c84 = [];
              const __e85 = document.createElement("sui-chip");
              __e85.setAttribute("size", "sm");
              const __t86 = document.createTextNode("");
              __e85.appendChild(__t86);
              __t86.textContent = $scope2.run.model;
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
        const __wireBoundary_b28 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.run.blocker)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag88 = document.createDocumentFragment();
              const __c89 = [];
              const __e90 = document.createElement("sui-alert");
              __e90.setAttribute("tone", "warning");
              __e90.setAttribute("compact", "");
              const __t91 = document.createTextNode("Blocked: ");
              __e90.appendChild(__t91);
              const __t92 = document.createTextNode("");
              __e90.appendChild(__t92);
              __t92.textContent = $scope2.run.blocker;
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
        const __wireBoundary_b31 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.run.steps,
          key: (step, __i2) => {
            const __child = Object.create($scope2);
            __child.step = step;
            __child.__i = __i2;
            return (($scope3) => step.id)(__child);
          },
          render: (step, __i2) => {
            const __childScope98 = Object.create($scope2);
            __childScope98.step = step;
            __childScope98.__i = __i2;
            return (($scope3) => {
              const __wireBoundary_b36 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if (!$scope3.step.isLast)
                    return 0;
                  return -1;
                },
                branches: [
                  () => {
                    const __frag107 = document.createDocumentFragment();
                    const __c108 = [];
                    const __e109 = document.createElement("span");
                    __e109.setAttribute("class", "arrow");
                    __e109.setAttribute("aria-hidden", "true");
                    const __t110 = document.createTextNode("→");
                    __e109.appendChild(__t110);
                    __frag107.appendChild(__e109);
                    return { nodes: Array.from(__frag107.childNodes), cleanup: () => {
                      for (const c of __c108)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __frag99 = document.createDocumentFragment();
              const __c100 = [];
              const __e101 = document.createElement("sui-pipeline-step");
              {
                const __av102 = $scope3.step.id;
                if (__av102 == null || __av102 === false || __av102 === "")
                  __e101.removeAttribute("name");
                else
                  __e101.setAttribute("name", __av102 === true ? "" : String(__av102));
              }
              {
                const __av103 = $scope3.step.state;
                if (__av103 == null || __av103 === false || __av103 === "")
                  __e101.removeAttribute("state");
                else
                  __e101.setAttribute("state", __av103 === true ? "" : String(__av103));
              }
              {
                const __av104 = $scope3.step.step_type;
                if (__av104 == null || __av104 === false || __av104 === "")
                  __e101.removeAttribute("step-type");
                else
                  __e101.setAttribute("step-type", __av104 === true ? "" : String(__av104));
              }
              {
                const __av105 = $scope3.step.agent;
                if (__av105 == null || __av105 === false || __av105 === "")
                  __e101.removeAttribute("agent");
                else
                  __e101.setAttribute("agent", __av105 === true ? "" : String(__av105));
              }
              __frag99.appendChild(__e101);
              const __ifAnchor106 = document.createComment("if");
              __frag99.appendChild(__ifAnchor106);
              __c100.push(__wireBoundary_b36(__ifAnchor106));
              return { nodes: Array.from(__frag99.childNodes), cleanup: () => {
                for (const c of __c100)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new111, __newI112) => {
                const __old = $scope3.step;
                $scope3.step = __new111;
                $scope3.__i = __newI112;
                if (__old.id !== $scope3.step.id) {
                  {
                    const __av102 = $scope3.step.id;
                    if (__av102 == null || __av102 === false || __av102 === "")
                      __e101.removeAttribute("name");
                    else
                      __e101.setAttribute("name", __av102 === true ? "" : String(__av102));
                  }
                }
                if (__old.state !== $scope3.step.state) {
                  {
                    const __av103 = $scope3.step.state;
                    if (__av103 == null || __av103 === false || __av103 === "")
                      __e101.removeAttribute("state");
                    else
                      __e101.setAttribute("state", __av103 === true ? "" : String(__av103));
                  }
                }
                if (__old.step_type !== $scope3.step.step_type) {
                  {
                    const __av104 = $scope3.step.step_type;
                    if (__av104 == null || __av104 === false || __av104 === "")
                      __e101.removeAttribute("step-type");
                    else
                      __e101.setAttribute("step-type", __av104 === true ? "" : String(__av104));
                  }
                }
                if (__old.agent !== $scope3.step.agent) {
                  {
                    const __av105 = $scope3.step.agent;
                    if (__av105 == null || __av105 === false || __av105 === "")
                      __e101.removeAttribute("agent");
                    else
                      __e101.setAttribute("agent", __av105 === true ? "" : String(__av105));
                  }
                }
              } };
            })(__childScope98);
          }
        });
        const __wireBoundary_b30 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.run.steps && $scope2.run.steps.length > 0)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag94 = document.createDocumentFragment();
              const __c95 = [];
              const __e96 = document.createElement("sui-pipeline-dag");
              const __forAnchor97 = document.createComment("for");
              __e96.appendChild(__forAnchor97);
              __c95.push(__wireBoundary_b31(__forAnchor97));
              __frag94.appendChild(__e96);
              return { nodes: Array.from(__frag94.childNodes), cleanup: () => {
                for (const c of __c95)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag51 = document.createDocumentFragment();
        const __c52 = [];
        const __e53 = document.createElement("sui-pipeline-run");
        {
          const __av54 = $scope2.run.state_attr;
          if (__av54 == null || __av54 === false || __av54 === "")
            __e53.removeAttribute("state");
          else
            __e53.setAttribute("state", __av54 === true ? "" : String(__av54));
        }
        {
          const __av55 = $scope2.run.slug;
          if (__av55 == null || __av55 === false || __av55 === "")
            __e53.removeAttribute("slug");
          else
            __e53.setAttribute("slug", __av55 === true ? "" : String(__av55));
        }
        {
          const __av56 = $scope2.run.agent;
          if (__av56 == null || __av56 === false || __av56 === "")
            __e53.removeAttribute("agent");
          else
            __e53.setAttribute("agent", __av56 === true ? "" : String(__av56));
        }
        const __e57 = document.createElement("sui-timestamp");
        __e57.setAttribute("slot", "actions");
        {
          const __av58 = $scope2.run.updated_at;
          if (__av58 == null || __av58 === false || __av58 === "")
            __e57.removeAttribute("value");
          else
            __e57.setAttribute("value", __av58 === true ? "" : String(__av58));
        }
        __e57.setAttribute("relative", "");
        __e53.appendChild(__e57);
        const __e59 = document.createElement("sui-button");
        __e59.setAttribute("slot", "actions");
        __e59.setAttribute("variant", "ghost");
        __e59.setAttribute("size", "sm");
        __c52.push(on(__e59, "click", function() {
          const __h = () => $scope2.navigate("/pipelines/run/" + $scope2.run.slug);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e60 = document.createElement("sui-icon");
        __e60.setAttribute("slot", "leading");
        __e60.setAttribute("name", "chevron-right");
        __e59.appendChild(__e60);
        const __t61 = document.createTextNode(`Open
            `);
        __e59.appendChild(__t61);
        __e53.appendChild(__e59);
        const __e62 = document.createElement("sui-button");
        __e62.setAttribute("slot", "actions");
        __e62.setAttribute("variant", "outline");
        __e62.setAttribute("size", "sm");
        __e62.setAttribute("tone", "destructive");
        __c52.push(on(__e62, "click", function() {
          const __h = () => $scope2.stopRun($scope2.run.job_id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e63 = document.createElement("sui-icon");
        __e63.setAttribute("slot", "leading");
        __e63.setAttribute("name", "stop");
        __e62.appendChild(__e63);
        const __t64 = document.createTextNode(`Stop
            `);
        __e62.appendChild(__t64);
        __e53.appendChild(__e62);
        const __e65 = document.createElement("div");
        __e65.setAttribute("class", "ship-run-detail");
        const __ifAnchor66 = document.createComment("if");
        __e65.appendChild(__ifAnchor66);
        __c52.push(__wireBoundary_b19(__ifAnchor66));
        const __ifAnchor72 = document.createComment("if");
        __e65.appendChild(__ifAnchor72);
        __c52.push(__wireBoundary_b22(__ifAnchor72));
        const __ifAnchor77 = document.createComment("if");
        __e65.appendChild(__ifAnchor77);
        __c52.push(__wireBoundary_b24(__ifAnchor77));
        const __ifAnchor82 = document.createComment("if");
        __e65.appendChild(__ifAnchor82);
        __c52.push(__wireBoundary_b26(__ifAnchor82));
        __e53.appendChild(__e65);
        const __ifAnchor87 = document.createComment("if");
        __e53.appendChild(__ifAnchor87);
        __c52.push(__wireBoundary_b28(__ifAnchor87));
        const __ifAnchor93 = document.createComment("if");
        __e53.appendChild(__ifAnchor93);
        __c52.push(__wireBoundary_b30(__ifAnchor93));
        const __e113 = document.createElement("sui-log-stream");
        __e113.setAttribute("slot", "footer");
        {
          const __av114 = "logs-" + $scope2.run.slug;
          if (__av114 == null || __av114 === false || __av114 === "")
            __e113.removeAttribute("id");
          else
            __e113.setAttribute("id", __av114 === true ? "" : String(__av114));
        }
        {
          const __av115 = "/api/v1/pipelines/runs/" + $scope2.run.slug + "/log/stream";
          if (__av115 == null || __av115 === false || __av115 === "")
            __e113.removeAttribute("url");
          else
            __e113.setAttribute("url", __av115 === true ? "" : String(__av115));
        }
        __e113.setAttribute("autoscroll", "");
        __e53.appendChild(__e113);
        __frag51.appendChild(__e53);
        return { nodes: Array.from(__frag51.childNodes), cleanup: () => {
          for (const c of __c52)
            try {
              c();
            } catch (e) {}
        }, update: (__new116, __newI117) => {
          const __old = $scope2.run;
          $scope2.run = __new116;
          $scope2.__i = __newI117;
          if (__old.state_attr !== $scope2.run.state_attr) {
            {
              const __av54 = $scope2.run.state_attr;
              if (__av54 == null || __av54 === false || __av54 === "")
                __e53.removeAttribute("state");
              else
                __e53.setAttribute("state", __av54 === true ? "" : String(__av54));
            }
          }
          if (__old.slug !== $scope2.run.slug) {
            {
              const __av55 = $scope2.run.slug;
              if (__av55 == null || __av55 === false || __av55 === "")
                __e53.removeAttribute("slug");
              else
                __e53.setAttribute("slug", __av55 === true ? "" : String(__av55));
            }
            {
              const __av114 = "logs-" + $scope2.run.slug;
              if (__av114 == null || __av114 === false || __av114 === "")
                __e113.removeAttribute("id");
              else
                __e113.setAttribute("id", __av114 === true ? "" : String(__av114));
            }
            {
              const __av115 = "/api/v1/pipelines/runs/" + $scope2.run.slug + "/log/stream";
              if (__av115 == null || __av115 === false || __av115 === "")
                __e113.removeAttribute("url");
              else
                __e113.setAttribute("url", __av115 === true ? "" : String(__av115));
            }
          }
          if (__old.agent !== $scope2.run.agent) {
            {
              const __av56 = $scope2.run.agent;
              if (__av56 == null || __av56 === false || __av56 === "")
                __e53.removeAttribute("agent");
              else
                __e53.setAttribute("agent", __av56 === true ? "" : String(__av56));
            }
          }
          if (__old.updated_at !== $scope2.run.updated_at) {
            {
              const __av58 = $scope2.run.updated_at;
              if (__av58 == null || __av58 === false || __av58 === "")
                __e57.removeAttribute("value");
              else
                __e57.setAttribute("value", __av58 === true ? "" : String(__av58));
            }
          }
        } };
      })(__childScope50);
    }
  });
  const __wireBoundary_b10 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.activeRuns().length > 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag39 = document.createDocumentFragment();
        const __c40 = [];
        const __e41 = document.createElement("div");
        __e41.setAttribute("class", "ship-pipelines-section");
        const __e42 = document.createElement("div");
        __e42.setAttribute("class", "ship-pipelines-section-head");
        const __e43 = document.createElement("h2");
        __e43.setAttribute("class", "ship-pipelines-section-title");
        const __e44 = document.createElement("sui-icon");
        __e44.setAttribute("name", "play");
        __e44.setAttribute("size", "sm");
        __e44.setAttribute("tone", "info");
        __e43.appendChild(__e44);
        const __t45 = document.createTextNode(`Active runs
        `);
        __e43.appendChild(__t45);
        __e42.appendChild(__e43);
        const __e46 = document.createElement("sui-badge");
        __e46.setAttribute("tone", "info");
        const __t47 = document.createTextNode("");
        __e46.appendChild(__t47);
        __c40.push(bindText(__t47, () => $scope.activeRuns().length));
        __e42.appendChild(__e46);
        __e41.appendChild(__e42);
        const __e48 = document.createElement("sui-stack");
        __e48.setAttribute("gap", "3");
        const __forAnchor49 = document.createComment("for");
        __e48.appendChild(__forAnchor49);
        __c40.push(__wireBoundary_b12(__forAnchor49));
        __e41.appendChild(__e48);
        __frag39.appendChild(__e41);
        return { nodes: Array.from(__frag39.childNodes), cleanup: () => {
          for (const c of __c40)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b41 = (anchor) => bindFor({
    anchor,
    items: () => $scope.defs(),
    key: (def, __i) => {
      const __child = Object.create($scope);
      __child.def = def;
      __child.__i = __i;
      return (($scope2) => def.name)(__child);
    },
    render: (def, __i) => {
      const __childScope130 = Object.create($scope);
      __childScope130.def = def;
      __childScope130.__i = __i;
      return (($scope2) => {
        __childScope130.__v_active_runs = signal2(0);
        __childScope130.__v_last_run_tone = signal2(0);
        const __wireBoundary_b45 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.def.active_runs > 0)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag138 = document.createDocumentFragment();
              const __c139 = [];
              const __e140 = document.createElement("sui-badge");
              __e140.setAttribute("tone", "info");
              const __t141 = document.createTextNode("");
              __e140.appendChild(__t141);
              __t141.textContent = $scope2.def.active_runs;
              const __t142 = document.createTextNode(" running");
              __e140.appendChild(__t142);
              __frag138.appendChild(__e140);
              return { nodes: Array.from(__frag138.childNodes), cleanup: () => {
                for (const c of __c139)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b47 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.def.description)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag144 = document.createDocumentFragment();
              const __c145 = [];
              const __e146 = document.createElement("p");
              __e146.setAttribute("class", "ship-def-desc");
              const __t147 = document.createTextNode("");
              __e146.appendChild(__t147);
              __t147.textContent = $scope2.def.description;
              __frag144.appendChild(__e146);
              return { nodes: Array.from(__frag144.childNodes), cleanup: () => {
                for (const c of __c145)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b51 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.def.last_run_status)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag154 = document.createDocumentFragment();
              const __c155 = [];
              const __e156 = document.createElement("sui-chip");
              __e156.setAttribute("size", "sm");
              __c155.push(bindAttr(__e156, "tone", () => (void $scope2.__v_last_run_tone(), $scope2.def.last_run_tone ?? "muted")));
              const __t157 = document.createTextNode("");
              __e156.appendChild(__t157);
              __t157.textContent = $scope2.def.last_run_status;
              __frag154.appendChild(__e156);
              return { nodes: Array.from(__frag154.childNodes), cleanup: () => {
                for (const c of __c155)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b54 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.def.recent_outcomes)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag159 = document.createDocumentFragment();
              const __c160 = [];
              const __e161 = document.createElement("span");
              __e161.setAttribute("class", "ship-def-spark");
              {
                const __av162 = "Last " + $scope2.def.recent_outcomes.length + " run outcomes";
                if (__av162 == null || __av162 === false || __av162 === "")
                  __e161.removeAttribute("aria-label");
                else
                  __e161.setAttribute("aria-label", __av162 === true ? "" : String(__av162));
              }
              __frag159.appendChild(__e161);
              return { nodes: Array.from(__frag159.childNodes), cleanup: () => {
                for (const c of __c160)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b59 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.def.last_run_at)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag180 = document.createDocumentFragment();
              const __c181 = [];
              const __e182 = document.createElement("span");
              __e182.setAttribute("class", "ship-def-stat ship-def-stat-grow");
              const __e183 = document.createElement("span");
              __e183.setAttribute("class", "ship-def-stat-value ship-def-stat-time");
              const __e184 = document.createElement("sui-timestamp");
              {
                const __av185 = $scope2.def.last_run_at;
                if (__av185 == null || __av185 === false || __av185 === "")
                  __e184.removeAttribute("value");
                else
                  __e184.setAttribute("value", __av185 === true ? "" : String(__av185));
              }
              __e184.setAttribute("relative", "");
              __e183.appendChild(__e184);
              __e182.appendChild(__e183);
              const __e186 = document.createElement("span");
              __e186.setAttribute("class", "ship-def-stat-label");
              const __t187 = document.createTextNode("last run");
              __e186.appendChild(__t187);
              __e182.appendChild(__e186);
              __frag180.appendChild(__e182);
              return { nodes: Array.from(__frag180.childNodes), cleanup: () => {
                for (const c of __c181)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b56 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.def.total_runs > 0)
              return 0;
            return 1;
          },
          branches: [
            () => {
              const __frag165 = document.createDocumentFragment();
              const __c166 = [];
              const __e167 = document.createElement("span");
              __e167.setAttribute("class", "ship-def-stat");
              const __e168 = document.createElement("span");
              __e168.setAttribute("class", "ship-def-stat-value");
              const __t169 = document.createTextNode("");
              __e168.appendChild(__t169);
              __t169.textContent = $scope2.def.total_runs;
              __e167.appendChild(__e168);
              const __e170 = document.createElement("span");
              __e170.setAttribute("class", "ship-def-stat-label");
              const __t171 = document.createTextNode("runs");
              __e170.appendChild(__t171);
              __e167.appendChild(__e170);
              __frag165.appendChild(__e167);
              const __e172 = document.createElement("span");
              __e172.setAttribute("class", "ship-def-stat");
              const __e173 = document.createElement("span");
              __e173.setAttribute("class", "ship-def-stat-value");
              const __t174 = document.createTextNode("");
              __e173.appendChild(__t174);
              __t174.textContent = $scope2.def.success_pct;
              const __e175 = document.createElement("span");
              __e175.setAttribute("class", "ship-def-stat-unit");
              const __t176 = document.createTextNode("%");
              __e175.appendChild(__t176);
              __e173.appendChild(__e175);
              __e172.appendChild(__e173);
              const __e177 = document.createElement("span");
              __e177.setAttribute("class", "ship-def-stat-label");
              const __t178 = document.createTextNode("success");
              __e177.appendChild(__t178);
              __e172.appendChild(__e177);
              __frag165.appendChild(__e172);
              const __ifAnchor179 = document.createComment("if");
              __frag165.appendChild(__ifAnchor179);
              __c166.push(__wireBoundary_b59(__ifAnchor179));
              return { nodes: Array.from(__frag165.childNodes), cleanup: () => {
                for (const c of __c166)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag188 = document.createDocumentFragment();
              const __c189 = [];
              const __e190 = document.createElement("span");
              __e190.setAttribute("class", "ship-def-stat-empty");
              const __t191 = document.createTextNode("No runs yet — click to dispatch");
              __e190.appendChild(__t191);
              __frag188.appendChild(__e190);
              return { nodes: Array.from(__frag188.childNodes), cleanup: () => {
                for (const c of __c189)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag131 = document.createDocumentFragment();
        const __c132 = [];
        const __e133 = document.createElement("sui-card");
        __e133.setAttribute("interactive", "");
        __e133.setAttribute("status-rail", "");
        __c132.push(bindAttr(__e133, "tone", () => (void $scope2.__v_active_runs(), void $scope2.__v_last_run_tone(), $scope2.def.active_runs > 0 ? "info" : $scope2.def.last_run_tone ?? "muted")));
        __c132.push(on(__e133, "click", function() {
          const __h = () => $scope2.navigate("/pipelines/focus/" + $scope2.def.name);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e134 = document.createElement("div");
        __e134.setAttribute("class", "ship-def-title");
        const __e135 = document.createElement("span");
        __e135.setAttribute("class", "ship-def-name");
        const __t136 = document.createTextNode("");
        __e135.appendChild(__t136);
        __t136.textContent = $scope2.def.name;
        __e134.appendChild(__e135);
        const __ifAnchor137 = document.createComment("if");
        __e134.appendChild(__ifAnchor137);
        __c132.push(__wireBoundary_b45(__ifAnchor137));
        __e133.appendChild(__e134);
        const __ifAnchor143 = document.createComment("if");
        __e133.appendChild(__ifAnchor143);
        __c132.push(__wireBoundary_b47(__ifAnchor143));
        const __e148 = document.createElement("div");
        __e148.setAttribute("class", "ship-def-chips");
        const __e149 = document.createElement("sui-chip");
        __e149.setAttribute("size", "sm");
        __e149.setAttribute("tone", "muted");
        const __t150 = document.createTextNode("");
        __e149.appendChild(__t150);
        __t150.textContent = $scope2.def.step_count;
        const __t151 = document.createTextNode(" step");
        __e149.appendChild(__t151);
        const __t152 = document.createTextNode("");
        __e149.appendChild(__t152);
        __t152.textContent = $scope2.def.step_count !== 1 ? "s" : "";
        __e148.appendChild(__e149);
        const __ifAnchor153 = document.createComment("if");
        __e148.appendChild(__ifAnchor153);
        __c132.push(__wireBoundary_b51(__ifAnchor153));
        const __ifAnchor158 = document.createComment("if");
        __e148.appendChild(__ifAnchor158);
        __c132.push(__wireBoundary_b54(__ifAnchor158));
        __e133.appendChild(__e148);
        const __e163 = document.createElement("sui-cluster");
        __e163.setAttribute("gap", "5");
        __e163.setAttribute("align", "baseline");
        __e163.setAttribute("slot", "footer");
        const __ifAnchor164 = document.createComment("if");
        __e163.appendChild(__ifAnchor164);
        __c132.push(__wireBoundary_b56(__ifAnchor164));
        __e133.appendChild(__e163);
        __frag131.appendChild(__e133);
        return { nodes: Array.from(__frag131.childNodes), cleanup: () => {
          for (const c of __c132)
            try {
              c();
            } catch (e) {}
        }, update: (__new192, __newI193) => {
          const __old = $scope2.def;
          $scope2.def = __new192;
          $scope2.__i = __newI193;
          if (__old.name !== $scope2.def.name) {
            __t136.textContent = $scope2.def.name;
          }
          if (__old.step_count !== $scope2.def.step_count) {
            __t150.textContent = $scope2.def.step_count;
            __t152.textContent = $scope2.def.step_count !== 1 ? "s" : "";
          }
          if (__old.active_runs !== $scope2.def.active_runs) {
            $scope2.__v_active_runs.set((c) => c + 1);
          }
          if (__old.last_run_tone !== $scope2.def.last_run_tone) {
            $scope2.__v_last_run_tone.set((c) => c + 1);
          }
        } };
      })(__childScope130);
    }
  });
  const __wireBoundary_b40 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.defs().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag126 = document.createDocumentFragment();
        const __c127 = [];
        const __e128 = document.createElement("div");
        __e128.setAttribute("class", "ship-pipelines-def-list");
        const __forAnchor129 = document.createComment("for");
        __e128.appendChild(__forAnchor129);
        __c127.push(__wireBoundary_b41(__forAnchor129));
        __frag126.appendChild(__e128);
        return { nodes: Array.from(__frag126.childNodes), cleanup: () => {
          for (const c of __c127)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag194 = document.createDocumentFragment();
        const __c195 = [];
        const __e196 = document.createElement("sui-empty-state");
        __e196.setAttribute("icon", "git-merge");
        __e196.setAttribute("heading", "No pipelines");
        __e196.setAttribute("description", "Define a pipeline in .ship/pipelines/ to see it here.");
        __frag194.appendChild(__e196);
        return { nodes: Array.from(__frag194.childNodes), cleanup: () => {
          for (const c of __c195)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b64 = (anchor) => bindFor({
    anchor,
    items: () => $scope.defs(),
    key: (def, __i) => {
      const __child = Object.create($scope);
      __child.def = def;
      __child.__i = __i;
      return (($scope2) => def.name)(__child);
    },
    render: (def, __i) => {
      const __childScope207 = Object.create($scope);
      __childScope207.def = def;
      __childScope207.__i = __i;
      return (($scope2) => {
        const __frag208 = document.createDocumentFragment();
        const __c209 = [];
        const __e210 = document.createElement("option");
        {
          const __av211 = $scope2.def.name;
          if (__av211 == null || __av211 === false || __av211 === "")
            __e210.removeAttribute("value");
          else
            __e210.setAttribute("value", __av211 === true ? "" : String(__av211));
        }
        const __t212 = document.createTextNode("");
        __e210.appendChild(__t212);
        __t212.textContent = $scope2.def.name;
        const __t213 = document.createTextNode("");
        __e210.appendChild(__t213);
        __t213.textContent = $scope2.def.description ? " — " + $scope2.def.description : "";
        __frag208.appendChild(__e210);
        return { nodes: Array.from(__frag208.childNodes), cleanup: () => {
          for (const c of __c209)
            try {
              c();
            } catch (e) {}
        }, update: (__new214, __newI215) => {
          const __old = $scope2.def;
          $scope2.def = __new214;
          $scope2.__i = __newI215;
          if (__old.name !== $scope2.def.name) {
            {
              const __av211 = $scope2.def.name;
              if (__av211 == null || __av211 === false || __av211 === "")
                __e210.removeAttribute("value");
              else
                __e210.setAttribute("value", __av211 === true ? "" : String(__av211));
            }
            __t212.textContent = $scope2.def.name;
          }
          if (__old.description !== $scope2.def.description) {
            __t213.textContent = $scope2.def.description ? " — " + $scope2.def.description : "";
          }
        } };
      })(__childScope207);
    }
  });
  const __wireBoundary_b70 = (anchor) => bindFor({
    anchor,
    items: () => $scope.providers(),
    key: (p, __i) => {
      const __child = Object.create($scope);
      __child.p = p;
      __child.__i = __i;
      return (($scope2) => p)(__child);
    },
    render: (p, __i) => {
      const __childScope223 = Object.create($scope);
      __childScope223.p = p;
      __childScope223.__i = __i;
      return (($scope2) => {
        const __frag224 = document.createDocumentFragment();
        const __c225 = [];
        const __e226 = document.createElement("option");
        {
          const __av227 = $scope2.p;
          if (__av227 == null || __av227 === false || __av227 === "")
            __e226.removeAttribute("value");
          else
            __e226.setAttribute("value", __av227 === true ? "" : String(__av227));
        }
        const __t228 = document.createTextNode("");
        __e226.appendChild(__t228);
        __t228.textContent = $scope2.p;
        __frag224.appendChild(__e226);
        return { nodes: Array.from(__frag224.childNodes), cleanup: () => {
          for (const c of __c225)
            try {
              c();
            } catch (e) {}
        }, update: (__new229, __newI230) => {
          $scope2.p = __new229;
          $scope2.__i = __newI230;
          {
            const __av227 = $scope2.p;
            if (__av227 == null || __av227 === false || __av227 === "")
              __e226.removeAttribute("value");
            else
              __e226.setAttribute("value", __av227 === true ? "" : String(__av227));
          }
          __t228.textContent = $scope2.p;
        } };
      })(__childScope223);
    }
  });
  const __wireBoundary_b80 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.newPipeError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag249 = document.createDocumentFragment();
        const __c250 = [];
        const __e251 = document.createElement("sui-alert");
        __e251.setAttribute("tone", "destructive");
        __e251.setAttribute("compact", "");
        const __t252 = document.createTextNode("");
        __e251.appendChild(__t252);
        __c250.push(bindText(__t252, () => $scope.newPipeError()));
        __frag249.appendChild(__e251);
        return { nodes: Array.from(__frag249.childNodes), cleanup: () => {
          for (const c of __c250)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b76 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.workspaceId())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag241 = document.createDocumentFragment();
        const __c242 = [];
        const __e243 = document.createElement("sui-dialog");
        __c242.push(bindProp(__e243, "open", () => $scope.newPipeOpen()));
        __e243.setAttribute("size", "sm");
        __c242.push(on(__e243, "sui-close", function() {
          const __h = $scope.closeNewPipe;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e244 = document.createElement("span");
        __e244.setAttribute("slot", "header");
        const __t245 = document.createTextNode("New pipeline");
        __e244.appendChild(__t245);
        __e243.appendChild(__e244);
        const __e246 = document.createElement("sui-field");
        __e246.setAttribute("label", "Name");
        __e246.setAttribute("hint", "Lowercase, kebab-case. Becomes the filename.");
        const __e247 = document.createElement("sui-input");
        __c242.push(bindValue(__e247, $scope.newPipeName));
        __e247.setAttribute("placeholder", "my-pipeline");
        __e247.setAttribute("autofocus", "");
        __e246.appendChild(__e247);
        __e243.appendChild(__e246);
        const __ifAnchor248 = document.createComment("if");
        __e243.appendChild(__ifAnchor248);
        __c242.push(__wireBoundary_b80(__ifAnchor248));
        const __e253 = document.createElement("sui-toolbar");
        __e253.setAttribute("slot", "footer");
        const __e254 = document.createElement("sui-button");
        __e254.setAttribute("variant", "outline");
        __c242.push(on(__e254, "click", function() {
          const __h = $scope.closeNewPipe;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t255 = document.createTextNode("Cancel");
        __e254.appendChild(__t255);
        __e253.appendChild(__e254);
        const __e256 = document.createElement("sui-button");
        __e256.setAttribute("tone", "primary");
        __e256.setAttribute("variant", "solid");
        __c242.push(bindAttr(__e256, "loading", () => $scope.newPipeCreating() ? "" : null));
        __c242.push(bindAttr(__e256, "disabled", () => $scope.newPipeCreating() ? "" : null));
        __c242.push(on(__e256, "click", function() {
          const __h = $scope.createPipeline;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e257 = document.createElement("sui-icon");
        __e257.setAttribute("slot", "leading");
        __e257.setAttribute("name", "plus");
        __e257.setAttribute("size", "sm");
        __e256.appendChild(__e257);
        const __t258 = document.createTextNode(`Create
      `);
        __e256.appendChild(__t258);
        __e253.appendChild(__e256);
        __e243.appendChild(__e253);
        __frag241.appendChild(__e243);
        return { nodes: Array.from(__frag241.childNodes), cleanup: () => {
          for (const c of __c242)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b4 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.defs() !== undefined)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag15 = document.createDocumentFragment();
        const __c16 = [];
        const __e17 = document.createElement("div");
        __e17.setAttribute("class", "ship-view ship-view-pipelines");
        const __e18 = document.createElement("div");
        __e18.setAttribute("class", "ship-pipelines-wrap");
        const __e19 = document.createElement("header");
        __e19.setAttribute("class", "ship-pipelines-head");
        const __e20 = document.createElement("div");
        const __e21 = document.createElement("h1");
        __e21.setAttribute("class", "ship-pipelines-title");
        const __t22 = document.createTextNode("Pipelines");
        __e21.appendChild(__t22);
        __e20.appendChild(__e21);
        const __e23 = document.createElement("p");
        __e23.setAttribute("class", "ship-pipelines-sub");
        const __t24 = document.createTextNode("");
        __e23.appendChild(__t24);
        __c16.push(bindText(__t24, () => $scope.statsSubtitle()));
        __e20.appendChild(__e23);
        __e19.appendChild(__e20);
        const __e25 = document.createElement("sui-cluster");
        __e25.setAttribute("gap", "2");
        const __e26 = document.createElement("sui-button");
        __e26.setAttribute("variant", "outline");
        __e26.setAttribute("size", "sm");
        __c16.push(on(__e26, "click", function() {
          const __h = () => $scope.navigate("/pipelines/history");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e27 = document.createElement("sui-icon");
        __e27.setAttribute("slot", "leading");
        __e27.setAttribute("name", "list");
        __e27.setAttribute("size", "sm");
        __e26.appendChild(__e27);
        const __t28 = document.createTextNode(`History
        `);
        __e26.appendChild(__t28);
        __e25.appendChild(__e26);
        const __ifAnchor29 = document.createComment("if");
        __e25.appendChild(__ifAnchor29);
        __c16.push(__wireBoundary_b7(__ifAnchor29));
        const __e35 = document.createElement("sui-button");
        __e35.setAttribute("tone", "primary");
        __e35.setAttribute("variant", "solid");
        __e35.setAttribute("size", "sm");
        __c16.push(on(__e35, "click", function() {
          const __h = $scope.openDispatch;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e36 = document.createElement("sui-icon");
        __e36.setAttribute("slot", "leading");
        __e36.setAttribute("name", "play");
        __e36.setAttribute("size", "sm");
        __e35.appendChild(__e36);
        const __t37 = document.createTextNode(`Dispatch
        `);
        __e35.appendChild(__t37);
        __e25.appendChild(__e35);
        __e19.appendChild(__e25);
        __e18.appendChild(__e19);
        const __ifAnchor38 = document.createComment("if");
        __e18.appendChild(__ifAnchor38);
        __c16.push(__wireBoundary_b10(__ifAnchor38));
        const __e118 = document.createElement("div");
        __e118.setAttribute("class", "ship-pipelines-section");
        const __e119 = document.createElement("div");
        __e119.setAttribute("class", "ship-pipelines-section-head");
        const __e120 = document.createElement("h2");
        __e120.setAttribute("class", "ship-pipelines-section-title");
        const __e121 = document.createElement("sui-icon");
        __e121.setAttribute("name", "git-merge");
        __e121.setAttribute("size", "sm");
        __e120.appendChild(__e121);
        const __t122 = document.createTextNode(`Pipelines
        `);
        __e120.appendChild(__t122);
        __e119.appendChild(__e120);
        const __e123 = document.createElement("sui-badge");
        __e123.setAttribute("tone", "muted");
        const __t124 = document.createTextNode("");
        __e123.appendChild(__t124);
        __c16.push(bindText(__t124, () => $scope.defs().length));
        __e119.appendChild(__e123);
        __e118.appendChild(__e119);
        const __ifAnchor125 = document.createComment("if");
        __e118.appendChild(__ifAnchor125);
        __c16.push(__wireBoundary_b40(__ifAnchor125));
        __e18.appendChild(__e118);
        __e17.appendChild(__e18);
        const __e197 = document.createElement("div");
        __e197.setAttribute("class", "ship-mobile-gate");
        const __e198 = document.createElement("sui-icon");
        __e198.setAttribute("name", "monitor");
        __e198.setAttribute("size", "xl");
        __e198.setAttribute("tone", "muted");
        __e197.appendChild(__e198);
        const __e199 = document.createElement("span");
        const __t200 = document.createTextNode("Best experienced on desktop");
        __e199.appendChild(__t200);
        __e197.appendChild(__e199);
        __e17.appendChild(__e197);
        const __e201 = document.createElement("sui-dialog");
        __c16.push(bindProp(__e201, "open", () => $scope.dispatchOpen()));
        __e201.setAttribute("size", "lg");
        __c16.push(on(__e201, "sui-close", function() {
          const __h = $scope.closeDispatch;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e202 = document.createElement("span");
        __e202.setAttribute("slot", "header");
        const __t203 = document.createTextNode("Dispatch pipeline run");
        __e202.appendChild(__t203);
        __e201.appendChild(__e202);
        const __e204 = document.createElement("sui-field");
        __e204.setAttribute("label", "Pipeline");
        const __e205 = document.createElement("sui-select");
        __c16.push(bindValue(__e205, $scope.dispatchPipeline));
        const __forAnchor206 = document.createComment("for");
        __e205.appendChild(__forAnchor206);
        __c16.push(__wireBoundary_b64(__forAnchor206));
        __e204.appendChild(__e205);
        __e201.appendChild(__e204);
        const __e216 = document.createElement("sui-field");
        __e216.setAttribute("label", "Spec");
        __e216.setAttribute("hint", "Plain text or markdown — what the run should do.");
        const __e217 = document.createElement("sui-textarea");
        __c16.push(bindValue(__e217, $scope.dispatchSpec));
        __e217.setAttribute("rows", "10");
        __e217.setAttribute("placeholder", "Describe the work in one or two paragraphs...");
        __e216.appendChild(__e217);
        __e201.appendChild(__e216);
        const __e218 = document.createElement("sui-field");
        __e218.setAttribute("label", "Provider override");
        __e218.setAttribute("hint", "Optional — leave blank to use the pipeline's declared providers.");
        const __e219 = document.createElement("sui-select");
        __c16.push(bindValue(__e219, $scope.dispatchProvider));
        const __e220 = document.createElement("option");
        __e220.setAttribute("value", "");
        const __t221 = document.createTextNode("(from pipeline)");
        __e220.appendChild(__t221);
        __e219.appendChild(__e220);
        const __forAnchor222 = document.createComment("for");
        __e219.appendChild(__forAnchor222);
        __c16.push(__wireBoundary_b70(__forAnchor222));
        __e218.appendChild(__e219);
        __e201.appendChild(__e218);
        const __e231 = document.createElement("sui-field");
        const __e232 = document.createElement("sui-checkbox");
        __c16.push(bindChecked(__e232, $scope.dispatchHeadless));
        const __t233 = document.createTextNode("Headless (run without tmux)");
        __e232.appendChild(__t233);
        __e231.appendChild(__e232);
        __e201.appendChild(__e231);
        const __e234 = document.createElement("sui-toolbar");
        __e234.setAttribute("slot", "footer");
        const __e235 = document.createElement("sui-button");
        __e235.setAttribute("variant", "outline");
        __c16.push(on(__e235, "click", function() {
          const __h = $scope.closeDispatch;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t236 = document.createTextNode("Cancel");
        __e235.appendChild(__t236);
        __e234.appendChild(__e235);
        const __e237 = document.createElement("sui-button");
        __e237.setAttribute("tone", "primary");
        __e237.setAttribute("variant", "solid");
        __c16.push(on(__e237, "click", function() {
          const __h = $scope.dispatchRun;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e238 = document.createElement("sui-icon");
        __e238.setAttribute("slot", "leading");
        __e238.setAttribute("name", "play");
        __e238.setAttribute("size", "sm");
        __e237.appendChild(__e238);
        const __t239 = document.createTextNode(`Dispatch
      `);
        __e237.appendChild(__t239);
        __e234.appendChild(__e237);
        __e201.appendChild(__e234);
        __e17.appendChild(__e201);
        const __ifAnchor240 = document.createComment("if");
        __e17.appendChild(__ifAnchor240);
        __c16.push(__wireBoundary_b76(__ifAnchor240));
        __frag15.appendChild(__e17);
        return { nodes: Array.from(__frag15.childNodes), cleanup: () => {
          for (const c of __c16)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b3 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (ctx().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag12 = document.createDocumentFragment();
        const __c13 = [];
        const __ifAnchor14 = document.createComment("if");
        __frag12.appendChild(__ifAnchor14);
        __c13.push(__wireBoundary_b4(__ifAnchor14));
        return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
          for (const c of __c13)
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
      const __wlAnchor3 = document.createComment("while-loading");
      container.appendChild(__wlAnchor3);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor3, signal: $scope.ctx, render: __renderWhileLoading_0 }));
      const __weAnchor10 = document.createComment("while-error");
      container.appendChild(__weAnchor10);
      __cleanups.push(bindWhileError({ anchor: __weAnchor10, signal: $scope.ctx, render: __renderWhileError_0 }));
      const __ifAnchor11 = document.createComment("if");
      container.appendChild(__ifAnchor11);
      __cleanups.push(__wireBoundary_b3(__ifAnchor11));
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
        __cleanups.push(hydrateWhileLoading(__m["b0"], "b0", () => $scope.ctx(), __renderWhileLoading_0));
      if (__m["b1"])
        __cleanups.push(hydrateWhileError(__m["b1"], "b1", () => $scope.ctx(), __renderWhileError_0));
      if (__m["b3"]) {
        const __t = __m["b3"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b3(__anchor));
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
