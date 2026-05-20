// views/home/page.sui
import { bindAttr, bindFor, bindIf, bindProp, bindText, bindWhileError, bindWhileLoading, data, live, on, signal, view } from "@ship/sui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateText, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/home/controller.ts
import { controller, unwrapOr, chrome } from "@ship/sui-runtime";
var EMPTY_DASH = {
  stats: { active: 0, history: 0 },
  agents: [],
  recent_runs: [],
  events: [],
  collections: [],
  total_events: 0
};
var EMPTY_GIT = {
  workspace_id: null,
  branch: "",
  head_short: "",
  clean: true,
  total_changed: 0,
  staged: [],
  modified: [],
  untracked: [],
  commits: [],
  branches: [],
  ahead: 0,
  behind: 0
};
var LINKS = [
  { key: "workflows", label: "Workflows", icon: "workflow", href: "/pipelines" },
  { key: "agents", label: "Agents", icon: "users", href: "/agents" },
  { key: "skills", label: "Skills", icon: "book", href: "/skills" },
  { key: "data", label: "Data", icon: "database", href: "/collections" },
  { key: "events", label: "Events", icon: "activity", href: "/events" },
  { key: "settings", label: "Settings", icon: "settings", href: "/settings" }
];
var initialOf = (agent) => (agent.letter ?? (agent.name ?? agent.id).slice(0, 1) ?? "?").toString().toUpperCase();
var readDash = ({ dash }) => unwrapOr(dash, EMPTY_DASH);
var readGit = ({ git }) => unwrapOr(git, EMPTY_GIT);
var controller_default = controller({
  scope: () => ({
    navigate: (path) => chrome.navigate(path),
    goNav: (event, path) => {
      event.preventDefault();
      chrome.navigate(path);
    }
  }),
  derived: {
    workspaceLabel: (s) => readDash(s).settings?.workspace_name ?? "workspace",
    totalJobs: (s) => readDash(s).stats.active + readDash(s).stats.history,
    activeJobs: (s) => readDash(s).stats.active,
    agentCount: (s) => readDash(s).agents.length,
    totalEvents: (s) => readDash(s).total_events,
    collectionsCount: (s) => readDash(s).collections.length,
    recentJobs: (s) => readDash(s).recent_runs,
    recentEvents: (s) => readDash(s).events,
    agentSlice: (s) => readDash(s).agents.slice(0, 6).map((a) => ({
      ...a,
      initials: initialOf(a)
    })),
    quickLinks: () => LINKS,
    isFirstRun: (s) => {
      const d = readDash(s);
      return d.stats.active + d.stats.history === 0 && d.agents.length === 0 && d.events.length === 0 && d.collections.length === 0;
    },
    branchName: (s) => readGit(s).branch ?? "",
    headShort: (s) => readGit(s).head_short ?? "",
    isClean: (s) => readGit(s).clean,
    changedSummary: (s) => `${readGit(s).total_changed} changed`
  }
});

// views/home/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var page_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.dash = live(() => ship.dashboard.state());
  $scope.git = live(() => ship.git.overview());
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b2 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1, 2, 3],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope18 = Object.create($scope);
      __childScope18.n = n;
      __childScope18.__i = __i;
      return (($scope2) => {
        const __frag19 = document.createDocumentFragment();
        const __c20 = [];
        const __e21 = document.createElement("sui-skeleton");
        __e21.setAttribute("shape", "block");
        __e21.setAttribute("size", "md");
        __frag19.appendChild(__e21);
        return { nodes: Array.from(__frag19.childNodes), cleanup: () => {
          for (const c of __c20)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope18);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag14 = document.createDocumentFragment();
    const __c15 = [];
    const __e16 = document.createElement("div");
    __e16.setAttribute("class", "home-stats");
    __e16.setAttribute("data-test", "home-loading");
    __e16.setAttribute("aria-busy", "true");
    __e16.setAttribute("aria-label", "Loading workspace");
    const __forAnchor17 = document.createComment("for");
    __e16.appendChild(__forAnchor17);
    __c15.push(__wireBoundary_b2(__forAnchor17));
    __frag14.appendChild(__e16);
    return { nodes: Array.from(__frag14.childNodes), cleanup: () => {
      for (const c of __c15)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope23 = Object.create($scope);
    __childScope23.err = err;
    return (($scope2) => {
      const __frag24 = document.createDocumentFragment();
      const __c25 = [];
      const __e26 = document.createElement("sui-error-state");
      __e26.setAttribute("title", "Couldn't load workspace");
      __e26.setAttribute("data-test", "home-error");
      __e26.setAttribute("role", "alert");
      __e26.setAttribute("aria-live", "assertive");
      const __t27 = document.createTextNode("");
      __e26.appendChild(__t27);
      __c25.push(bindText(__t27, () => $scope2.err.message));
      const __e28 = document.createElement("sui-button");
      __e28.setAttribute("slot", "action");
      __e28.setAttribute("tone", "primary");
      __e28.setAttribute("variant", "solid");
      __c25.push(on(__e28, "click", function() {
        const __h = () => window.location.reload();
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t29 = document.createTextNode("Retry");
      __e28.appendChild(__t29);
      __e26.appendChild(__e28);
      __frag24.appendChild(__e26);
      return { nodes: Array.from(__frag24.childNodes), cleanup: () => {
        for (const c of __c25)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope23);
  };
  const __renderWhileLoading_1 = () => {
    const __frag93 = document.createDocumentFragment();
    const __c94 = [];
    const __e95 = document.createElement("span");
    __e95.setAttribute("class", "home-glance-skel");
    __e95.setAttribute("data-test", "git-loading");
    const __t96 = document.createTextNode("Loading git status…");
    __e95.appendChild(__t96);
    __frag93.appendChild(__e95);
    return { nodes: Array.from(__frag93.childNodes), cleanup: () => {
      for (const c of __c94)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_1 = (err) => {
    const __childScope98 = Object.create($scope);
    __childScope98.err = err;
    return (($scope2) => {
      const __frag99 = document.createDocumentFragment();
      const __c100 = [];
      const __e101 = document.createElement("span");
      __e101.setAttribute("class", "home-glance-err");
      __e101.setAttribute("data-test", "git-error");
      const __t102 = document.createTextNode("Git stream offline: ");
      __e101.appendChild(__t102);
      const __t103 = document.createTextNode("");
      __e101.appendChild(__t103);
      __c100.push(bindText(__t103, () => $scope2.err.message));
      __frag99.appendChild(__e101);
      return { nodes: Array.from(__frag99.childNodes), cleanup: () => {
        for (const c of __c100)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope98);
  };
  const __wireBoundary_b20 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.git().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag106 = document.createDocumentFragment();
        const __c107 = [];
        const __e108 = document.createElement("span");
        __e108.setAttribute("class", "home-glance-line");
        __e108.setAttribute("data-test", "git-summary");
        const __e109 = document.createElement("span");
        __e109.setAttribute("class", "home-glance-dot");
        __c107.push(bindAttr(__e109, "data-clean", () => $scope.isClean() ? "1" : "0"));
        __e109.setAttribute("aria-hidden", "true");
        __e108.appendChild(__e109);
        const __e110 = document.createElement("span");
        const __t111 = document.createTextNode("Branch ");
        __e110.appendChild(__t111);
        const __e112 = document.createElement("strong");
        __e112.setAttribute("class", "home-mono");
        __e112.setAttribute("data-test", "git-branch");
        const __t113 = document.createTextNode("");
        __e112.appendChild(__t113);
        __c107.push(bindText(__t113, () => $scope.branchName()));
        __e110.appendChild(__e112);
        __e108.appendChild(__e110);
        const __e114 = document.createElement("span");
        __e114.setAttribute("class", "home-glance-sep");
        const __t115 = document.createTextNode("·");
        __e114.appendChild(__t115);
        __e108.appendChild(__e114);
        const __e116 = document.createElement("span");
        __e116.setAttribute("class", "home-mono");
        __e116.setAttribute("data-test", "git-head");
        const __t117 = document.createTextNode("");
        __e116.appendChild(__t117);
        __c107.push(bindText(__t117, () => $scope.headShort()));
        __e108.appendChild(__e116);
        const __e118 = document.createElement("span");
        __e118.setAttribute("class", "home-glance-sep");
        const __t119 = document.createTextNode("·");
        __e118.appendChild(__t119);
        __e108.appendChild(__e118);
        const __e120 = document.createElement("span");
        __e120.setAttribute("data-test", "git-status");
        const __t121 = document.createTextNode("");
        __e120.appendChild(__t121);
        __c107.push(bindText(__t121, () => $scope.isClean() ? "clean" : $scope.changedSummary()));
        __e108.appendChild(__e120);
        __frag106.appendChild(__e108);
        return { nodes: Array.from(__frag106.childNodes), cleanup: () => {
          for (const c of __c107)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b27 = (anchor) => bindFor({
    anchor,
    items: () => $scope.recentJobs(),
    key: (run, __i) => {
      const __child = Object.create($scope);
      __child.run = run;
      __child.__i = __i;
      return (($scope2) => run.id)(__child);
    },
    render: (run, __i) => {
      const __childScope135 = Object.create($scope);
      __childScope135.run = run;
      __childScope135.__i = __i;
      return (($scope2) => {
        __childScope135.__v_agent = signal(0);
        __childScope135.__v_created_at = signal(0);
        __childScope135.__v_id = signal(0);
        __childScope135.__v_slug = signal(0);
        __childScope135.__v_started_at = signal(0);
        __childScope135.__v_tone = signal(0);
        const __wireBoundary_b30 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.run.agent)
              return 0;
            return 1;
          },
          branches: [
            () => {
              const __frag144 = document.createDocumentFragment();
              const __c145 = [];
              const __e146 = document.createElement("sui-avatar");
              __e146.setAttribute("size", "sm");
              {
                const __av147 = $scope2.run.agent;
                if (__av147 == null || __av147 === false || __av147 === "")
                  __e146.removeAttribute("name");
                else
                  __e146.setAttribute("name", __av147 === true ? "" : String(__av147));
              }
              __c145.push(bindAttr(__e146, "initials", () => (void $scope2.__v_agent(), $scope2.run.agent.slice(0, 1).toUpperCase())));
              __frag144.appendChild(__e146);
              const __e148 = document.createElement("span");
              __e148.setAttribute("class", "home-job-agent");
              const __t149 = document.createTextNode("");
              __e148.appendChild(__t149);
              __t149.textContent = $scope2.run.agent;
              __frag144.appendChild(__e148);
              return { nodes: Array.from(__frag144.childNodes), cleanup: () => {
                for (const c of __c145)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag150 = document.createDocumentFragment();
              const __c151 = [];
              const __e152 = document.createElement("span");
              __e152.setAttribute("class", "home-job-agent");
              const __t153 = document.createTextNode("—");
              __e152.appendChild(__t153);
              __frag150.appendChild(__e152);
              return { nodes: Array.from(__frag150.childNodes), cleanup: () => {
                for (const c of __c151)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __wireBoundary_b34 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.run.started_at ?? $scope2.run.created_at)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag155 = document.createDocumentFragment();
              const __c156 = [];
              const __e157 = document.createElement("sui-timestamp");
              __c156.push(bindProp(__e157, "value", () => (void $scope2.__v_started_at(), void $scope2.__v_created_at(), $scope2.run.started_at ?? $scope2.run.created_at)));
              __e157.setAttribute("class", "home-job-time");
              __frag155.appendChild(__e157);
              return { nodes: Array.from(__frag155.childNodes), cleanup: () => {
                for (const c of __c156)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag136 = document.createDocumentFragment();
        const __c137 = [];
        const __e138 = document.createElement("sui-list-tile");
        __e138.setAttribute("density", "compact");
        __e138.setAttribute("class", "home-job-tile");
        __e138.setAttribute("data-test", "home-job");
        const __e139 = document.createElement("sui-status-dot");
        __e139.setAttribute("slot", "leading");
        __c137.push(bindProp(__e139, "tone", () => (void $scope2.__v_tone(), $scope2.run.tone ?? "muted")));
        __e138.appendChild(__e139);
        const __e140 = document.createElement("span");
        __e140.setAttribute("class", "home-job-name");
        const __t141 = document.createTextNode("");
        __e140.appendChild(__t141);
        __c137.push(bindText(__t141, () => (void $scope2.__v_slug(), void $scope2.__v_id(), $scope2.run.slug ?? $scope2.run.id)));
        __e138.appendChild(__e140);
        const __e142 = document.createElement("span");
        __e142.setAttribute("slot", "trailing");
        __e142.setAttribute("class", "home-job-trailing");
        const __ifAnchor143 = document.createComment("if");
        __e142.appendChild(__ifAnchor143);
        __c137.push(__wireBoundary_b30(__ifAnchor143));
        const __ifAnchor154 = document.createComment("if");
        __e142.appendChild(__ifAnchor154);
        __c137.push(__wireBoundary_b34(__ifAnchor154));
        __e138.appendChild(__e142);
        __frag136.appendChild(__e138);
        return { nodes: Array.from(__frag136.childNodes), cleanup: () => {
          for (const c of __c137)
            try {
              c();
            } catch (e) {}
        }, update: (__new158, __newI159) => {
          const __old = $scope2.run;
          $scope2.run = __new158;
          $scope2.__i = __newI159;
          if (__old.agent !== $scope2.run.agent) {
            $scope2.__v_agent.set((c) => c + 1);
          }
          if (__old.created_at !== $scope2.run.created_at) {
            $scope2.__v_created_at.set((c) => c + 1);
          }
          if (__old.id !== $scope2.run.id) {
            $scope2.__v_id.set((c) => c + 1);
          }
          if (__old.slug !== $scope2.run.slug) {
            $scope2.__v_slug.set((c) => c + 1);
          }
          if (__old.started_at !== $scope2.run.started_at) {
            $scope2.__v_started_at.set((c) => c + 1);
          }
          if (__old.tone !== $scope2.run.tone) {
            $scope2.__v_tone.set((c) => c + 1);
          }
        } };
      })(__childScope135);
    }
  });
  const __wireBoundary_b26 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.recentJobs().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag132 = document.createDocumentFragment();
        const __c133 = [];
        const __forAnchor134 = document.createComment("for");
        __frag132.appendChild(__forAnchor134);
        __c133.push(__wireBoundary_b27(__forAnchor134));
        return { nodes: Array.from(__frag132.childNodes), cleanup: () => {
          for (const c of __c133)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag160 = document.createDocumentFragment();
        const __c161 = [];
        const __e162 = document.createElement("sui-empty-state");
        __e162.setAttribute("variant", "placeholder");
        __e162.setAttribute("size", "sm");
        __e162.setAttribute("icon", "play");
        __e162.setAttribute("data-test", "home-jobs-empty");
        const __e163 = document.createElement("span");
        __e163.setAttribute("slot", "title");
        const __t164 = document.createTextNode("No jobs yet");
        __e163.appendChild(__t164);
        __e162.appendChild(__e163);
        const __t165 = document.createTextNode(`
                  Start a workflow to see runs here.
                `);
        __e162.appendChild(__t165);
        __frag160.appendChild(__e162);
        return { nodes: Array.from(__frag160.childNodes), cleanup: () => {
          for (const c of __c161)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b39 = (anchor) => bindFor({
    anchor,
    items: () => $scope.agentSlice(),
    key: (agent, __i) => {
      const __child = Object.create($scope);
      __child.agent = agent;
      __child.__i = __i;
      return (($scope2) => agent.id)(__child);
    },
    render: (agent, __i) => {
      const __childScope182 = Object.create($scope);
      __childScope182.agent = agent;
      __childScope182.__i = __i;
      return (($scope2) => {
        __childScope182.__v_id = signal(0);
        __childScope182.__v_name = signal(0);
        const __wireBoundary_b44 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.agent.description)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag193 = document.createDocumentFragment();
              const __c194 = [];
              const __e195 = document.createElement("p");
              __e195.setAttribute("class", "home-agent-desc");
              const __t196 = document.createTextNode("");
              __e195.appendChild(__t196);
              __t196.textContent = $scope2.agent.description;
              __frag193.appendChild(__e195);
              return { nodes: Array.from(__frag193.childNodes), cleanup: () => {
                for (const c of __c194)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag183 = document.createDocumentFragment();
        const __c184 = [];
        const __e185 = document.createElement("sui-list-tile");
        __e185.setAttribute("variant", "card");
        __e185.setAttribute("density", "compact");
        __e185.setAttribute("interactive", "");
        {
          const __av186 = "/agents/" + $scope2.agent.id;
          if (__av186 == null || __av186 === false || __av186 === "")
            __e185.removeAttribute("navigable");
          else
            __e185.setAttribute("navigable", __av186 === true ? "" : String(__av186));
        }
        __e185.setAttribute("class", "home-agent-tile");
        __e185.setAttribute("data-test", "home-agent");
        const __e187 = document.createElement("sui-avatar");
        __e187.setAttribute("slot", "leading");
        __e187.setAttribute("size", "md");
        __e187.setAttribute("shape", "square");
        __c184.push(bindAttr(__e187, "name", () => (void $scope2.__v_name(), void $scope2.__v_id(), $scope2.agent.name ?? $scope2.agent.id)));
        {
          const __av188 = $scope2.agent.initials;
          if (__av188 == null || __av188 === false || __av188 === "")
            __e187.removeAttribute("initials");
          else
            __e187.setAttribute("initials", __av188 === true ? "" : String(__av188));
        }
        __e185.appendChild(__e187);
        const __e189 = document.createElement("div");
        __e189.setAttribute("class", "home-agent-body");
        const __e190 = document.createElement("span");
        __e190.setAttribute("class", "home-agent-name");
        const __t191 = document.createTextNode("");
        __e190.appendChild(__t191);
        __c184.push(bindText(__t191, () => (void $scope2.__v_name(), void $scope2.__v_id(), $scope2.agent.name ?? $scope2.agent.id)));
        __e189.appendChild(__e190);
        const __ifAnchor192 = document.createComment("if");
        __e189.appendChild(__ifAnchor192);
        __c184.push(__wireBoundary_b44(__ifAnchor192));
        __e185.appendChild(__e189);
        __frag183.appendChild(__e185);
        return { nodes: Array.from(__frag183.childNodes), cleanup: () => {
          for (const c of __c184)
            try {
              c();
            } catch (e) {}
        }, update: (__new197, __newI198) => {
          const __old = $scope2.agent;
          $scope2.agent = __new197;
          $scope2.__i = __newI198;
          if (__old.id !== $scope2.agent.id) {
            {
              const __av186 = "/agents/" + $scope2.agent.id;
              if (__av186 == null || __av186 === false || __av186 === "")
                __e185.removeAttribute("navigable");
              else
                __e185.setAttribute("navigable", __av186 === true ? "" : String(__av186));
            }
          }
          if (__old.initials !== $scope2.agent.initials) {
            {
              const __av188 = $scope2.agent.initials;
              if (__av188 == null || __av188 === false || __av188 === "")
                __e187.removeAttribute("initials");
              else
                __e187.setAttribute("initials", __av188 === true ? "" : String(__av188));
            }
          }
          if (__old.id !== $scope2.agent.id) {
            $scope2.__v_id.set((c) => c + 1);
          }
          if (__old.name !== $scope2.agent.name) {
            $scope2.__v_name.set((c) => c + 1);
          }
        } };
      })(__childScope182);
    }
  });
  const __wireBoundary_b38 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.agentSlice().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag178 = document.createDocumentFragment();
        const __c179 = [];
        const __e180 = document.createElement("div");
        __e180.setAttribute("class", "home-agents-grid");
        __e180.setAttribute("data-view-enter", "");
        const __forAnchor181 = document.createComment("for");
        __e180.appendChild(__forAnchor181);
        __c179.push(__wireBoundary_b39(__forAnchor181));
        __frag178.appendChild(__e180);
        return { nodes: Array.from(__frag178.childNodes), cleanup: () => {
          for (const c of __c179)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag199 = document.createDocumentFragment();
        const __c200 = [];
        const __e201 = document.createElement("sui-empty-state");
        __e201.setAttribute("variant", "placeholder");
        __e201.setAttribute("size", "sm");
        __e201.setAttribute("icon", "user-plus");
        __e201.setAttribute("data-test", "home-agents-empty");
        const __e202 = document.createElement("span");
        __e202.setAttribute("slot", "title");
        const __t203 = document.createTextNode("No agents configured");
        __e202.appendChild(__t203);
        __e201.appendChild(__e202);
        const __e204 = document.createElement("sui-button");
        __e204.setAttribute("slot", "action");
        __e204.setAttribute("size", "sm");
        __e204.setAttribute("tone", "primary");
        __e204.setAttribute("variant", "solid");
        __c200.push(on(__e204, "click", function() {
          const __h = () => $scope.navigate("/agents/create");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e205 = document.createElement("sui-icon");
        __e205.setAttribute("slot", "leading");
        __e205.setAttribute("name", "plus");
        __e204.appendChild(__e205);
        const __t206 = document.createTextNode(`
                    Add an agent
                  `);
        __e204.appendChild(__t206);
        __e201.appendChild(__e204);
        __frag199.appendChild(__e201);
        return { nodes: Array.from(__frag199.childNodes), cleanup: () => {
          for (const c of __c200)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b50 = (anchor) => bindFor({
    anchor,
    items: () => $scope.recentEvents(),
    key: (r, __i) => {
      const __child = Object.create($scope);
      __child.r = r;
      __child.__i = __i;
      return (($scope2) => r.id ?? r.timestamp)(__child);
    },
    render: (r, __i) => {
      const __childScope223 = Object.create($scope);
      __childScope223.r = r;
      __childScope223.__i = __i;
      return (($scope2) => {
        __childScope223.__v_actor = signal(0);
        const __wireBoundary_b53 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.r.actor)
              return 0;
            return 1;
          },
          branches: [
            () => {
              const __frag234 = document.createDocumentFragment();
              const __c235 = [];
              const __e236 = document.createElement("sui-avatar");
              __e236.setAttribute("size", "sm");
              {
                const __av237 = $scope2.r.actor;
                if (__av237 == null || __av237 === false || __av237 === "")
                  __e236.removeAttribute("name");
                else
                  __e236.setAttribute("name", __av237 === true ? "" : String(__av237));
              }
              __c235.push(bindAttr(__e236, "initials", () => (void $scope2.__v_actor(), $scope2.r.actor.slice(0, 1).toUpperCase())));
              __frag234.appendChild(__e236);
              const __e238 = document.createElement("span");
              const __t239 = document.createTextNode("");
              __e238.appendChild(__t239);
              __t239.textContent = $scope2.r.actor;
              __frag234.appendChild(__e238);
              return { nodes: Array.from(__frag234.childNodes), cleanup: () => {
                for (const c of __c235)
                  try {
                    c();
                  } catch (e) {}
              } };
            },
            () => {
              const __frag240 = document.createDocumentFragment();
              const __c241 = [];
              const __e242 = document.createElement("span");
              const __t243 = document.createTextNode("—");
              __e242.appendChild(__t243);
              __frag240.appendChild(__e242);
              return { nodes: Array.from(__frag240.childNodes), cleanup: () => {
                for (const c of __c241)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag224 = document.createDocumentFragment();
        const __c225 = [];
        const __e226 = document.createElement("li");
        __e226.setAttribute("class", "home-activity-row");
        __e226.setAttribute("data-test", "home-activity-row");
        const __e227 = document.createElement("span");
        __e227.setAttribute("class", "home-activity-rail");
        __e227.setAttribute("aria-hidden", "true");
        __e226.appendChild(__e227);
        const __e228 = document.createElement("span");
        __e228.setAttribute("class", "home-event-ts");
        const __t229 = document.createTextNode("");
        __e228.appendChild(__t229);
        __t229.textContent = $scope2.r.timestamp;
        __e226.appendChild(__e228);
        const __e230 = document.createElement("span");
        __e230.setAttribute("class", "home-event-type");
        const __t231 = document.createTextNode("");
        __e230.appendChild(__t231);
        __t231.textContent = $scope2.r.event_type;
        __e226.appendChild(__e230);
        const __e232 = document.createElement("span");
        __e232.setAttribute("class", "home-event-actor");
        const __ifAnchor233 = document.createComment("if");
        __e232.appendChild(__ifAnchor233);
        __c225.push(__wireBoundary_b53(__ifAnchor233));
        __e226.appendChild(__e232);
        __frag224.appendChild(__e226);
        return { nodes: Array.from(__frag224.childNodes), cleanup: () => {
          for (const c of __c225)
            try {
              c();
            } catch (e) {}
        }, update: (__new244, __newI245) => {
          const __old = $scope2.r;
          $scope2.r = __new244;
          $scope2.__i = __newI245;
          if (__old.timestamp !== $scope2.r.timestamp) {
            __t229.textContent = $scope2.r.timestamp;
          }
          if (__old.event_type !== $scope2.r.event_type) {
            __t231.textContent = $scope2.r.event_type;
          }
          if (__old.actor !== $scope2.r.actor) {
            $scope2.__v_actor.set((c) => c + 1);
          }
        } };
      })(__childScope223);
    }
  });
  const __wireBoundary_b49 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.recentEvents().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag219 = document.createDocumentFragment();
        const __c220 = [];
        const __e221 = document.createElement("ol");
        __e221.setAttribute("class", "home-activity-stream");
        __e221.setAttribute("data-view-enter", "");
        const __forAnchor222 = document.createComment("for");
        __e221.appendChild(__forAnchor222);
        __c220.push(__wireBoundary_b50(__forAnchor222));
        __frag219.appendChild(__e221);
        return { nodes: Array.from(__frag219.childNodes), cleanup: () => {
          for (const c of __c220)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag246 = document.createDocumentFragment();
        const __c247 = [];
        const __e248 = document.createElement("sui-empty-state");
        __e248.setAttribute("variant", "placeholder");
        __e248.setAttribute("size", "sm");
        __e248.setAttribute("icon", "activity");
        __e248.setAttribute("data-test", "home-activity-empty");
        const __e249 = document.createElement("span");
        __e249.setAttribute("slot", "title");
        const __t250 = document.createTextNode("Waiting for activity");
        __e249.appendChild(__t250);
        __e248.appendChild(__e249);
        const __t251 = document.createTextNode(`
                  The stream will fill as the daemon emits events.
                `);
        __e248.appendChild(__t251);
        __frag246.appendChild(__e248);
        return { nodes: Array.from(__frag246.childNodes), cleanup: () => {
          for (const c of __c247)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b57 = (anchor) => bindFor({
    anchor,
    items: () => $scope.quickLinks(),
    key: (link, __i) => {
      const __child = Object.create($scope);
      __child.link = link;
      __child.__i = __i;
      return (($scope2) => link.href)(__child);
    },
    render: (link, __i) => {
      const __childScope259 = Object.create($scope);
      __childScope259.link = link;
      __childScope259.__i = __i;
      return (($scope2) => {
        const __frag260 = document.createDocumentFragment();
        const __c261 = [];
        const __e262 = document.createElement("a");
        __e262.setAttribute("class", "home-link-btn");
        {
          const __av263 = $scope2.link.href;
          if (__av263 == null || __av263 === false || __av263 === "")
            __e262.removeAttribute("href");
          else
            __e262.setAttribute("href", __av263 === true ? "" : String(__av263));
        }
        __c261.push(on(__e262, "click", function() {
          const __h = (e) => $scope2.goNav(e, $scope2.link.href);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e262.setAttribute("data-test", "home-link");
        {
          const __av264 = $scope2.link.key;
          if (__av264 == null || __av264 === false || __av264 === "")
            __e262.removeAttribute("data-link");
          else
            __e262.setAttribute("data-link", __av264 === true ? "" : String(__av264));
        }
        const __e265 = document.createElement("div");
        __e265.setAttribute("class", "home-link-icon");
        {
          const __av266 = $scope2.link.key;
          if (__av266 == null || __av266 === false || __av266 === "")
            __e265.removeAttribute("data-tone");
          else
            __e265.setAttribute("data-tone", __av266 === true ? "" : String(__av266));
        }
        __e265.setAttribute("aria-hidden", "true");
        const __e267 = document.createElement("sui-icon");
        {
          const __av268 = $scope2.link.icon;
          if (__av268 == null || __av268 === false || __av268 === "")
            __e267.removeAttribute("name");
          else
            __e267.setAttribute("name", __av268 === true ? "" : String(__av268));
        }
        __e265.appendChild(__e267);
        __e262.appendChild(__e265);
        const __t269 = document.createTextNode("");
        __e262.appendChild(__t269);
        __t269.textContent = $scope2.link.label;
        __frag260.appendChild(__e262);
        return { nodes: Array.from(__frag260.childNodes), cleanup: () => {
          for (const c of __c261)
            try {
              c();
            } catch (e) {}
        }, update: (__new270, __newI271) => {
          const __old = $scope2.link;
          $scope2.link = __new270;
          $scope2.__i = __newI271;
          if (__old.href !== $scope2.link.href) {
            {
              const __av263 = $scope2.link.href;
              if (__av263 == null || __av263 === false || __av263 === "")
                __e262.removeAttribute("href");
              else
                __e262.setAttribute("href", __av263 === true ? "" : String(__av263));
            }
          }
          if (__old.key !== $scope2.link.key) {
            {
              const __av264 = $scope2.link.key;
              if (__av264 == null || __av264 === false || __av264 === "")
                __e262.removeAttribute("data-link");
              else
                __e262.setAttribute("data-link", __av264 === true ? "" : String(__av264));
            }
            {
              const __av266 = $scope2.link.key;
              if (__av266 == null || __av266 === false || __av266 === "")
                __e265.removeAttribute("data-tone");
              else
                __e265.setAttribute("data-tone", __av266 === true ? "" : String(__av266));
            }
          }
          if (__old.icon !== $scope2.link.icon) {
            {
              const __av268 = $scope2.link.icon;
              if (__av268 == null || __av268 === false || __av268 === "")
                __e267.removeAttribute("name");
              else
                __e267.setAttribute("name", __av268 === true ? "" : String(__av268));
            }
          }
          if (__old.label !== $scope2.link.label) {
            __t269.textContent = $scope2.link.label;
          }
        } };
      })(__childScope259);
    }
  });
  const __wireBoundary_b7 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.isFirstRun())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag35 = document.createDocumentFragment();
        const __c36 = [];
        const __e37 = document.createElement("sui-empty-state");
        __e37.setAttribute("icon", "sparkles");
        __e37.setAttribute("variant", "default");
        __e37.setAttribute("size", "lg");
        __e37.setAttribute("data-test", "home-empty");
        __e37.setAttribute("role", "status");
        __e37.setAttribute("class", "home-empty");
        const __e38 = document.createElement("span");
        __e38.setAttribute("slot", "title");
        const __t39 = document.createTextNode("Welcome to Ship");
        __e38.appendChild(__t39);
        __e37.appendChild(__e38);
        const __t40 = document.createTextNode(`
          This workspace is empty. Start by adding an agent or scaffolding a view —
          everything else fills in as you work.
          `);
        __e37.appendChild(__t40);
        const __e41 = document.createElement("sui-cluster");
        __e41.setAttribute("slot", "action");
        __e41.setAttribute("gap", "2");
        __e41.setAttribute("justify", "center");
        __e41.setAttribute("class", "home-empty-actions");
        const __e42 = document.createElement("sui-button");
        __e42.setAttribute("tone", "primary");
        __e42.setAttribute("variant", "solid");
        __c36.push(on(__e42, "click", function() {
          const __h = () => $scope.navigate("/agents/create");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e42.setAttribute("data-test", "home-empty-agent");
        const __e43 = document.createElement("sui-icon");
        __e43.setAttribute("slot", "leading");
        __e43.setAttribute("name", "user-plus");
        __e42.appendChild(__e43);
        const __t44 = document.createTextNode(`
              Add an agent
            `);
        __e42.appendChild(__t44);
        __e41.appendChild(__e42);
        const __e45 = document.createElement("sui-button");
        __e45.setAttribute("variant", "outline");
        __c36.push(on(__e45, "click", function() {
          const __h = () => $scope.navigate("/skills");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e45.setAttribute("data-test", "home-empty-skill");
        const __e46 = document.createElement("sui-icon");
        __e46.setAttribute("slot", "leading");
        __e46.setAttribute("name", "book");
        __e45.appendChild(__e46);
        const __t47 = document.createTextNode(`
              Browse skills
            `);
        __e45.appendChild(__t47);
        __e41.appendChild(__e45);
        const __e48 = document.createElement("sui-button");
        __e48.setAttribute("variant", "ghost");
        __c36.push(on(__e48, "click", function() {
          const __h = () => $scope.navigate("/ship-ui-studio");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e48.setAttribute("data-test", "home-empty-studio");
        const __e49 = document.createElement("sui-icon");
        __e49.setAttribute("slot", "leading");
        __e49.setAttribute("name", "sparkles");
        __e48.appendChild(__e49);
        const __t50 = document.createTextNode(`
              Open Studio
            `);
        __e48.appendChild(__t50);
        __e41.appendChild(__e48);
        __e37.appendChild(__e41);
        __frag35.appendChild(__e37);
        return { nodes: Array.from(__frag35.childNodes), cleanup: () => {
          for (const c of __c36)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag51 = document.createDocumentFragment();
        const __c52 = [];
        const __e53 = document.createElement("div");
        __e53.setAttribute("class", "home-stats");
        __e53.setAttribute("role", "list");
        __e53.setAttribute("aria-label", "Workspace statistics");
        __e53.setAttribute("data-test", "home-stats");
        __e53.setAttribute("data-view-enter", "");
        const __e54 = document.createElement("div");
        __e54.setAttribute("class", "home-stat-wrap home-stat-wrap--jobs");
        __e54.setAttribute("role", "listitem");
        const __e55 = document.createElement("sui-stat");
        const __e56 = document.createElement("span");
        __e56.setAttribute("slot", "label");
        const __t57 = document.createTextNode("Jobs");
        __e56.appendChild(__t57);
        __e55.appendChild(__e56);
        const __t58 = document.createTextNode("");
        __e55.appendChild(__t58);
        __c52.push(bindText(__t58, () => $scope.totalJobs()));
        const __e59 = document.createElement("span");
        __e59.setAttribute("slot", "hint");
        const __t60 = document.createTextNode("");
        __e59.appendChild(__t60);
        __c52.push(bindText(__t60, () => $scope.activeJobs()));
        const __t61 = document.createTextNode(" active");
        __e59.appendChild(__t61);
        __e55.appendChild(__e59);
        __e54.appendChild(__e55);
        __e53.appendChild(__e54);
        const __e62 = document.createElement("div");
        __e62.setAttribute("class", "home-stat-wrap home-stat-wrap--agents");
        __e62.setAttribute("role", "listitem");
        const __e63 = document.createElement("sui-stat");
        const __e64 = document.createElement("span");
        __e64.setAttribute("slot", "label");
        const __t65 = document.createTextNode("Agents");
        __e64.appendChild(__t65);
        __e63.appendChild(__e64);
        const __t66 = document.createTextNode("");
        __e63.appendChild(__t66);
        __c52.push(bindText(__t66, () => $scope.agentCount()));
        const __e67 = document.createElement("span");
        __e67.setAttribute("slot", "hint");
        const __t68 = document.createTextNode("configured");
        __e67.appendChild(__t68);
        __e63.appendChild(__e67);
        __e62.appendChild(__e63);
        __e53.appendChild(__e62);
        const __e69 = document.createElement("div");
        __e69.setAttribute("class", "home-stat-wrap home-stat-wrap--events");
        __e69.setAttribute("role", "listitem");
        const __e70 = document.createElement("sui-stat");
        const __e71 = document.createElement("span");
        __e71.setAttribute("slot", "label");
        const __t72 = document.createTextNode("Events");
        __e71.appendChild(__t72);
        __e70.appendChild(__e71);
        const __t73 = document.createTextNode("");
        __e70.appendChild(__t73);
        __c52.push(bindText(__t73, () => $scope.totalEvents()));
        const __e74 = document.createElement("span");
        __e74.setAttribute("slot", "hint");
        const __t75 = document.createTextNode("in stream");
        __e74.appendChild(__t75);
        __e70.appendChild(__e74);
        __e69.appendChild(__e70);
        __e53.appendChild(__e69);
        const __e76 = document.createElement("div");
        __e76.setAttribute("class", "home-stat-wrap home-stat-wrap--collections");
        __e76.setAttribute("role", "listitem");
        const __e77 = document.createElement("sui-stat");
        const __e78 = document.createElement("span");
        __e78.setAttribute("slot", "label");
        const __t79 = document.createTextNode("Collections");
        __e78.appendChild(__t79);
        __e77.appendChild(__e78);
        const __t80 = document.createTextNode("");
        __e77.appendChild(__t80);
        __c52.push(bindText(__t80, () => $scope.collectionsCount()));
        const __e81 = document.createElement("span");
        __e81.setAttribute("slot", "hint");
        const __t82 = document.createTextNode("schemas defined");
        __e81.appendChild(__t82);
        __e77.appendChild(__e81);
        __e76.appendChild(__e77);
        __e53.appendChild(__e76);
        __frag51.appendChild(__e53);
        const __e83 = document.createElement("div");
        __e83.setAttribute("class", "home-body");
        const __e84 = document.createElement("sui-card");
        __e84.setAttribute("padding", "none");
        __e84.setAttribute("class", "home-section home-body--wide");
        __e84.setAttribute("aria-label", "Project at-a-glance");
        __e84.setAttribute("data-test", "home-glance");
        const __e85 = document.createElement("div");
        __e85.setAttribute("slot", "header");
        __e85.setAttribute("class", "home-section-head");
        const __e86 = document.createElement("h2");
        __e86.setAttribute("class", "home-section-title");
        const __e87 = document.createElement("sui-icon");
        __e87.setAttribute("class", "home-section-icon");
        __e87.setAttribute("name", "git-branch");
        __e87.setAttribute("aria-hidden", "true");
        __e86.appendChild(__e87);
        const __t88 = document.createTextNode(`
                Project at-a-glance
              `);
        __e86.appendChild(__t88);
        __e85.appendChild(__e86);
        const __e89 = document.createElement("a");
        __e89.setAttribute("class", "home-section-link");
        __e89.setAttribute("href", "/git");
        __c52.push(on(__e89, "click", function() {
          const __h = (e) => $scope.goNav(e, "/git");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t90 = document.createTextNode(`
                Git view `);
        __e89.appendChild(__t90);
        const __e91 = document.createElement("sui-icon");
        __e91.setAttribute("name", "chevron-right");
        __e91.setAttribute("size", "xs");
        __e91.setAttribute("aria-hidden", "true");
        __e89.appendChild(__e91);
        __e85.appendChild(__e89);
        __e84.appendChild(__e85);
        const __e92 = document.createElement("div");
        __e92.setAttribute("class", "home-glance");
        const __wlAnchor97 = document.createComment("while-loading");
        __e92.appendChild(__wlAnchor97);
        __c52.push(bindWhileLoading({ anchor: __wlAnchor97, signal: $scope.git, render: __renderWhileLoading_1 }));
        const __weAnchor104 = document.createComment("while-error");
        __e92.appendChild(__weAnchor104);
        __c52.push(bindWhileError({ anchor: __weAnchor104, signal: $scope.git, render: __renderWhileError_1 }));
        const __ifAnchor105 = document.createComment("if");
        __e92.appendChild(__ifAnchor105);
        __c52.push(__wireBoundary_b20(__ifAnchor105));
        __e84.appendChild(__e92);
        __e83.appendChild(__e84);
        const __e122 = document.createElement("sui-card");
        __e122.setAttribute("padding", "none");
        __e122.setAttribute("class", "home-section");
        __e122.setAttribute("aria-label", "Recent jobs");
        __e122.setAttribute("data-test", "home-jobs");
        const __e123 = document.createElement("div");
        __e123.setAttribute("slot", "header");
        __e123.setAttribute("class", "home-section-head");
        const __e124 = document.createElement("h2");
        __e124.setAttribute("class", "home-section-title");
        const __e125 = document.createElement("sui-icon");
        __e125.setAttribute("class", "home-section-icon");
        __e125.setAttribute("name", "workflow");
        __e125.setAttribute("aria-hidden", "true");
        __e124.appendChild(__e125);
        const __t126 = document.createTextNode(`
                Recent jobs
              `);
        __e124.appendChild(__t126);
        __e123.appendChild(__e124);
        const __e127 = document.createElement("a");
        __e127.setAttribute("class", "home-section-link");
        __e127.setAttribute("href", "/pipelines");
        __c52.push(on(__e127, "click", function() {
          const __h = (e) => $scope.goNav(e, "/pipelines");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t128 = document.createTextNode(`
                View all `);
        __e127.appendChild(__t128);
        const __e129 = document.createElement("sui-icon");
        __e129.setAttribute("name", "chevron-right");
        __e129.setAttribute("size", "xs");
        __e129.setAttribute("aria-hidden", "true");
        __e127.appendChild(__e129);
        __e123.appendChild(__e127);
        __e122.appendChild(__e123);
        const __e130 = document.createElement("sui-stack");
        const __ifAnchor131 = document.createComment("if");
        __e130.appendChild(__ifAnchor131);
        __c52.push(__wireBoundary_b26(__ifAnchor131));
        __e122.appendChild(__e130);
        __e83.appendChild(__e122);
        const __e166 = document.createElement("sui-card");
        __e166.setAttribute("padding", "none");
        __e166.setAttribute("class", "home-section");
        __e166.setAttribute("aria-label", "Agents");
        __e166.setAttribute("data-test", "home-agents");
        const __e167 = document.createElement("div");
        __e167.setAttribute("slot", "header");
        __e167.setAttribute("class", "home-section-head");
        const __e168 = document.createElement("h2");
        __e168.setAttribute("class", "home-section-title");
        const __e169 = document.createElement("sui-icon");
        __e169.setAttribute("class", "home-section-icon");
        __e169.setAttribute("name", "users");
        __e169.setAttribute("aria-hidden", "true");
        __e168.appendChild(__e169);
        const __t170 = document.createTextNode(`
                Agents
                `);
        __e168.appendChild(__t170);
        const __e171 = document.createElement("sui-badge");
        __e171.setAttribute("tone", "muted");
        __e171.setAttribute("variant", "soft");
        __e171.setAttribute("size", "sm");
        const __t172 = document.createTextNode("");
        __e171.appendChild(__t172);
        __c52.push(bindText(__t172, () => $scope.agentCount()));
        __e168.appendChild(__e171);
        __e167.appendChild(__e168);
        const __e173 = document.createElement("a");
        __e173.setAttribute("class", "home-section-link");
        __e173.setAttribute("href", "/agents");
        __c52.push(on(__e173, "click", function() {
          const __h = (e) => $scope.goNav(e, "/agents");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t174 = document.createTextNode(`
                Manage `);
        __e173.appendChild(__t174);
        const __e175 = document.createElement("sui-icon");
        __e175.setAttribute("name", "chevron-right");
        __e175.setAttribute("size", "xs");
        __e175.setAttribute("aria-hidden", "true");
        __e173.appendChild(__e175);
        __e167.appendChild(__e173);
        __e166.appendChild(__e167);
        const __e176 = document.createElement("div");
        __e176.setAttribute("class", "home-section-body");
        const __ifAnchor177 = document.createComment("if");
        __e176.appendChild(__ifAnchor177);
        __c52.push(__wireBoundary_b38(__ifAnchor177));
        __e166.appendChild(__e176);
        __e83.appendChild(__e166);
        const __e207 = document.createElement("sui-card");
        __e207.setAttribute("padding", "none");
        __e207.setAttribute("class", "home-section home-body--wide");
        __e207.setAttribute("aria-label", "Recent activity");
        __e207.setAttribute("data-test", "home-activity");
        const __e208 = document.createElement("div");
        __e208.setAttribute("slot", "header");
        __e208.setAttribute("class", "home-section-head");
        const __e209 = document.createElement("h2");
        __e209.setAttribute("class", "home-section-title");
        const __e210 = document.createElement("sui-icon");
        __e210.setAttribute("class", "home-section-icon");
        __e210.setAttribute("name", "activity");
        __e210.setAttribute("aria-hidden", "true");
        __e209.appendChild(__e210);
        const __t211 = document.createTextNode(`
                Recent activity
                `);
        __e209.appendChild(__t211);
        const __e212 = document.createElement("sui-badge");
        __e212.setAttribute("tone", "muted");
        __e212.setAttribute("variant", "soft");
        __e212.setAttribute("size", "sm");
        const __t213 = document.createTextNode("");
        __e212.appendChild(__t213);
        __c52.push(bindText(__t213, () => $scope.recentEvents().length));
        __e209.appendChild(__e212);
        __e208.appendChild(__e209);
        const __e214 = document.createElement("a");
        __e214.setAttribute("class", "home-section-link");
        __e214.setAttribute("href", "/events");
        __c52.push(on(__e214, "click", function() {
          const __h = (e) => $scope.goNav(e, "/events");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t215 = document.createTextNode(`
                Full stream `);
        __e214.appendChild(__t215);
        const __e216 = document.createElement("sui-icon");
        __e216.setAttribute("name", "chevron-right");
        __e216.setAttribute("size", "xs");
        __e216.setAttribute("aria-hidden", "true");
        __e214.appendChild(__e216);
        __e208.appendChild(__e214);
        __e207.appendChild(__e208);
        const __e217 = document.createElement("div");
        __e217.setAttribute("class", "home-section-body");
        const __ifAnchor218 = document.createComment("if");
        __e217.appendChild(__ifAnchor218);
        __c52.push(__wireBoundary_b49(__ifAnchor218));
        __e207.appendChild(__e217);
        __e83.appendChild(__e207);
        const __e252 = document.createElement("sui-card");
        __e252.setAttribute("padding", "none");
        __e252.setAttribute("class", "home-section home-body--wide");
        __e252.setAttribute("aria-label", "Quick links");
        __e252.setAttribute("data-test", "home-links");
        const __e253 = document.createElement("div");
        __e253.setAttribute("slot", "header");
        __e253.setAttribute("class", "home-section-head");
        const __e254 = document.createElement("h2");
        __e254.setAttribute("class", "home-section-title");
        const __e255 = document.createElement("sui-icon");
        __e255.setAttribute("class", "home-section-icon");
        __e255.setAttribute("name", "external-link");
        __e255.setAttribute("aria-hidden", "true");
        __e254.appendChild(__e255);
        const __t256 = document.createTextNode(`
                Quick links
              `);
        __e254.appendChild(__t256);
        __e253.appendChild(__e254);
        __e252.appendChild(__e253);
        const __e257 = document.createElement("div");
        __e257.setAttribute("class", "home-links");
        const __forAnchor258 = document.createComment("for");
        __e257.appendChild(__forAnchor258);
        __c52.push(__wireBoundary_b57(__forAnchor258));
        __e252.appendChild(__e257);
        __e83.appendChild(__e252);
        __frag51.appendChild(__e83);
        return { nodes: Array.from(__frag51.childNodes), cleanup: () => {
          for (const c of __c52)
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
      if ($scope.dash().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag32 = document.createDocumentFragment();
        const __c33 = [];
        const __ifAnchor34 = document.createComment("if");
        __frag32.appendChild(__ifAnchor34);
        __c33.push(__wireBoundary_b7(__ifAnchor34));
        return { nodes: Array.from(__frag32.childNodes), cleanup: () => {
          for (const c of __c33)
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
      __e0.setAttribute("class", "ship-view home-view");
      const __e1 = document.createElement("div");
      __e1.setAttribute("class", "home-wrap");
      const __e2 = document.createElement("header");
      __e2.setAttribute("class", "home-head");
      const __e3 = document.createElement("sui-cluster");
      __e3.setAttribute("gap", "3");
      __e3.setAttribute("align", "center");
      __e3.setAttribute("class", "home-wordmark");
      const __e4 = document.createElement("sui-app-icon");
      __e4.setAttribute("class", "home-logo");
      __e4.setAttribute("name", "dashboard");
      __e4.setAttribute("variant", "solid");
      __e4.setAttribute("size", "lg");
      __e4.setAttribute("radius", "md");
      __e4.setAttribute("aria-hidden", "true");
      __e3.appendChild(__e4);
      const __e5 = document.createElement("div");
      const __e6 = document.createElement("h1");
      __e6.setAttribute("class", "home-title");
      __e6.setAttribute("data-test", "home-title");
      const __t7 = document.createTextNode("SHIP");
      __e6.appendChild(__t7);
      __e5.appendChild(__e6);
      const __e8 = document.createElement("p");
      __e8.setAttribute("class", "home-workspace");
      __e8.setAttribute("data-test", "home-workspace");
      const __t9 = document.createTextNode("");
      __e8.appendChild(__t9);
      __cleanups.push(bindText(__t9, () => $scope.workspaceLabel()));
      __e5.appendChild(__e8);
      __e3.appendChild(__e5);
      __e2.appendChild(__e3);
      const __e10 = document.createElement("sui-cluster");
      __e10.setAttribute("gap", "2");
      __e10.setAttribute("align", "center");
      __e10.setAttribute("class", "home-head-side");
      const __e11 = document.createElement("div");
      __e11.setAttribute("class", "home-live-badge");
      __e11.setAttribute("role", "status");
      __e11.setAttribute("aria-label", "Live updates active");
      const __e12 = document.createElement("span");
      __e12.setAttribute("class", "home-live-dot");
      __e11.appendChild(__e12);
      const __t13 = document.createTextNode(`live
        `);
      __e11.appendChild(__t13);
      __e10.appendChild(__e11);
      __e2.appendChild(__e10);
      __e1.appendChild(__e2);
      const __wlAnchor22 = document.createComment("while-loading");
      __e1.appendChild(__wlAnchor22);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor22, signal: $scope.dash, render: __renderWhileLoading_0 }));
      const __weAnchor30 = document.createComment("while-error");
      __e1.appendChild(__weAnchor30);
      __cleanups.push(bindWhileError({ anchor: __weAnchor30, signal: $scope.dash, render: __renderWhileError_0 }));
      const __ifAnchor31 = document.createComment("if");
      __e1.appendChild(__ifAnchor31);
      __cleanups.push(__wireBoundary_b6(__ifAnchor31));
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
        __cleanups.push(hydrateText(__t["b0"], () => $scope.workspaceLabel()));
      if (__m["b1"])
        __cleanups.push(hydrateWhileLoading(__m["b1"], "b1", () => $scope.dash(), __renderWhileLoading_0));
      if (__m["b3"])
        __cleanups.push(hydrateWhileError(__m["b3"], "b3", () => $scope.dash(), __renderWhileError_0));
      if (__m["b6"]) {
        const __t2 = __m["b6"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
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
