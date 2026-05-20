// views/git/page.sui
import { bindAttr, bindChecked, bindFor, bindIf, bindProp, bindText, bindValue, bindWhileError, bindWhileLoading, data, getTemplateHook, live as live2, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";
import"@ship/sui-runtime/template-hooks";

// views/git/controller.ts
import { controller, signal, live, unwrapOr, useFilteredList } from "@ship/sui-runtime";
var EMPTY_OVERVIEW = {
  workspace_id: null,
  branch: null,
  head_short: null,
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
var NEVER = { subscribe: () => () => {} };
function errMessage(err) {
  if (err instanceof Error)
    return err.message;
  if (typeof err === "string")
    return err;
  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}
var controller_default = controller({
  scope: ({ ship }) => {
    const stagePaths = (paths) => ship.git.stage({ paths });
    const unstagePaths = (paths) => ship.git.unstage({ paths });
    const commitNow = (message) => ship.git.commit({ message });
    const pushNow = () => ship.git.push({});
    const pullNow = () => ship.git.pull({});
    const fetchNow = () => ship.git.fetch({});
    const checkoutRef = (ref) => ship.git.checkout({ ref });
    const createBranch = (name, checkout) => ship.git.branch_create({ name, checkout });
    const deleteBranch = (name, force) => ship.git.branch_delete({ name, force });
    const initNow = () => ship.git.init();
    const tab = signal("changes");
    const selectedCommit = signal("");
    const selectedFile = signal("");
    const diffMode = signal("unified");
    const activeBranchName = signal("");
    const refreshNonce = signal(0);
    const opError = signal("");
    const opInfo = signal("");
    const busy = signal("");
    const commitOpen = signal(false);
    const commitSubject = signal("");
    const commitBody = signal("");
    const commitError = signal("");
    const branchCreateOpen = signal(false);
    const branchCreateName = signal("");
    const branchCreateCheckout = signal(true);
    const branchCreateError = signal("");
    const branchDeleteOpen = signal(false);
    const branchDeleteTarget = signal("");
    const branchDeleteForce = signal(false);
    const branchDeleteError = signal("");
    const overview = live(() => {
      refreshNonce();
      return ship.git.overview();
    });
    const diff = live(() => {
      refreshNonce();
      const sha = selectedCommit();
      if (!sha)
        return NEVER;
      return ship.git.diff({ commit: sha });
    });
    return {
      tab,
      selectedCommit,
      selectedFile,
      diffMode,
      activeBranchName,
      refreshNonce,
      overview,
      diff,
      opError,
      opInfo,
      busy,
      commitOpen,
      commitSubject,
      commitBody,
      commitError,
      branchCreateOpen,
      branchCreateName,
      branchCreateCheckout,
      branchCreateError,
      branchDeleteOpen,
      branchDeleteTarget,
      branchDeleteForce,
      branchDeleteError,
      stagePaths,
      unstagePaths,
      commitNow,
      pushNow,
      pullNow,
      fetchNow,
      checkoutRef,
      createBranch,
      deleteBranch,
      initNow
    };
  },
  derived: {
    branch: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branch ?? "",
    headShort: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).head_short ?? "",
    clean: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).clean,
    totalChanged: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).total_changed,
    ahead: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).ahead,
    behind: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).behind,
    staged: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).staged,
    modified: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).modified,
    untracked: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).untracked,
    commits: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).commits,
    branches: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branches,
    commitsCount: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).commits.length,
    branchesCount: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branches.length,
    recentCommits: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).commits.slice(0, 5),
    recentBranches: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branches.slice(0, 6),
    stagedCount: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).staged.length,
    unstagedCount: (s) => {
      const o = unwrapOr(s.overview, EMPTY_OVERVIEW);
      return o.modified.length + o.untracked.length;
    },
    canCommit: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).staged.length > 0,
    fileGroups: (s) => {
      const o = unwrapOr(s.overview, EMPTY_OVERVIEW);
      return [
        { key: "staged", label: "Staged", glyph: "S", tone: "success", variant: "soft", items: o.staged },
        { key: "modified", label: "Modified", glyph: "", tone: "warning", variant: "soft", items: o.modified },
        { key: "untracked", label: "Untracked", glyph: "?", tone: "muted", variant: "outline", items: o.untracked }
      ].filter((g) => g.items.length > 0);
    },
    activeBranch: (s) => {
      const list = unwrapOr(s.overview, EMPTY_OVERVIEW).branches;
      const name = s.activeBranchName();
      const picked = name ? list.find((b) => b.name === name) : undefined;
      return picked ?? list.find((b) => b.active) ?? list[0] ?? null;
    },
    diffFiles: (s) => {
      const v = s.diff();
      if (v.kind !== "ok" || !v.value)
        return [];
      const file = s.selectedFile();
      const all = v.value.parsed?.files ?? [];
      return file ? all.filter((f) => f.path === file) : all;
    },
    diffReady: (s) => s.diff().kind === "ok",
    diffIsWorking: (s) => {
      const v = s.diff();
      return v.kind === "ok" && !!v.value?.is_working;
    },
    overviewErrMsg: (s) => {
      const v = s.overview();
      return v.kind === "error" ? v.error?.message ?? "Unknown error" : "";
    },
    isNoRepo: (s) => {
      const v = s.overview();
      const msg = v.kind === "error" ? v.error?.message ?? "" : "";
      return /no.repo|repo unavailable|not a git/i.test(msg);
    }
  },
  use: (s) => ({
    commitFilter: useFilteredList(() => unwrapOr(s.overview, EMPTY_OVERVIEW).commits, (c, q) => {
      if (!q)
        return true;
      const needle = q.toLowerCase();
      return c.subject.toLowerCase().includes(needle) || c.short_sha.toLowerCase().includes(needle) || c.author.toLowerCase().includes(needle);
    })
  }),
  on: {
    showChanges: (s) => {
      s.tab = "changes";
    },
    showHistory: (s) => {
      s.tab = "history";
    },
    showBranches: (s) => {
      s.tab = "branches";
    },
    refresh: (s) => {
      const cur = s.refreshNonce;
      s.refreshNonce = cur + 1;
    },
    selectFile: (s, file) => {
      s.selectedFile = file.path;
      s.selectedCommit = "WORKING";
      s.tab = "changes";
    },
    selectCommit: (s, sha) => {
      s.selectedCommit = sha;
      s.selectedFile = "";
      s.tab = "changes";
    },
    setDiffMode: (s, mode) => {
      s.diffMode = mode;
    },
    selectBranch: (s, name) => {
      s.activeBranchName = name;
    },
    dismissError: (s) => {
      s.opError = "";
      s.opInfo = "";
    },
    stageFile: async (s, file) => {
      await runOp(s, "stage", () => callFn(s, "stagePaths", [file.path]));
    },
    unstageFile: async (s, file) => {
      await runOp(s, "unstage", () => callFn(s, "unstagePaths", [file.path]));
    },
    stageAll: async (s) => {
      const o = unwrapOv(s);
      const paths = [...o.modified, ...o.untracked].map((f) => f.path);
      if (paths.length === 0)
        return;
      await runOp(s, "stage all", () => callFn(s, "stagePaths", paths));
    },
    unstageAll: async (s) => {
      const o = unwrapOv(s);
      const paths = o.staged.map((f) => f.path);
      if (paths.length === 0)
        return;
      await runOp(s, "unstage all", () => callFn(s, "unstagePaths", paths));
    },
    openCommit: (s) => {
      s.commitOpen = true;
      s.commitSubject = "";
      s.commitBody = "";
      s.commitError = "";
    },
    closeCommit: (s) => {
      s.commitOpen = false;
    },
    submitCommit: async (s) => {
      const subject = String(s.commitSubject).trim();
      if (!subject) {
        s.commitError = "Subject is required.";
        return;
      }
      const bodyText = String(s.commitBody).trim();
      const message = bodyText ? `${subject}

${bodyText}` : subject;
      s.commitError = "";
      const ok = await runOp(s, "commit", () => callFn(s, "commitNow", message));
      if (ok) {
        s.commitOpen = false;
        s.opInfo = `Committed “${subject}”.`;
      } else {
        s.commitError = String(s.opError);
      }
    },
    pushCurrent: async (s) => {
      await runOp(s, "push", async () => {
        const r = await callFn(s, "pushNow");
        s.opInfo = `Pushed to ${r.remote}.`;
      });
    },
    pullCurrent: async (s) => {
      await runOp(s, "pull", async () => {
        const r = await callFn(s, "pullNow");
        s.opInfo = `Pulled. HEAD is now ${r.head.slice(0, 7)}.`;
      });
    },
    fetchOrigin: async (s) => {
      await runOp(s, "fetch", async () => {
        const r = await callFn(s, "fetchNow");
        s.opInfo = `Fetched from ${r.remote}.`;
      });
    },
    checkoutBranch: async (s, name) => {
      await runOp(s, "checkout", async () => {
        await callFn(s, "checkoutRef", name);
        s.opInfo = `Switched to ${name}.`;
      });
    },
    openBranchCreate: (s) => {
      s.branchCreateOpen = true;
      s.branchCreateName = "";
      s.branchCreateError = "";
    },
    closeBranchCreate: (s) => {
      s.branchCreateOpen = false;
    },
    submitBranchCreate: async (s) => {
      const name = String(s.branchCreateName).trim();
      if (!/^[a-zA-Z0-9._/-]+$/.test(name)) {
        s.branchCreateError = "Branch name must be alphanumeric with -_./.";
        return;
      }
      const checkout = !!s.branchCreateCheckout;
      const ok = await runOp(s, "branch create", () => callFn(s, "createBranch", name, checkout));
      if (ok) {
        s.branchCreateOpen = false;
        s.opInfo = `Branch ${name} created${checkout ? " and checked out" : ""}.`;
      } else {
        s.branchCreateError = String(s.opError);
      }
    },
    openBranchDelete: (s, name) => {
      s.branchDeleteTarget = name;
      s.branchDeleteForce = false;
      s.branchDeleteError = "";
      s.branchDeleteOpen = true;
    },
    closeBranchDelete: (s) => {
      s.branchDeleteOpen = false;
    },
    submitBranchDelete: async (s) => {
      const name = String(s.branchDeleteTarget);
      const force = !!s.branchDeleteForce;
      const ok = await runOp(s, "branch delete", () => callFn(s, "deleteBranch", name, force));
      if (ok) {
        s.branchDeleteOpen = false;
        s.opInfo = `Branch ${name} deleted.`;
      } else {
        s.branchDeleteError = String(s.opError);
      }
    },
    initRepo: async (s) => {
      await runOp(s, "init", async () => {
        await callFn(s, "initNow");
        s.opInfo = "Initialized git repository.";
      });
    }
  }
});
async function runOp(s, label, fn) {
  s.busy = label;
  s.opError = "";
  s.opInfo = "";
  try {
    await fn();
    const cur = s.refreshNonce;
    s.refreshNonce = cur + 1;
    return true;
  } catch (err) {
    s.opError = `${label}: ${errMessage(err)}`;
    return false;
  } finally {
    s.busy = "";
  }
}
function callFn(s, name, ...args) {
  const fn = s[name];
  if (typeof fn !== "function") {
    throw new Error(`controller closure ${name} is not a function`);
  }
  return fn(...args);
}
function unwrapOv(s) {
  const v = s.overview();
  return v.kind === "ok" && v.value ? v.value : EMPTY_OVERVIEW;
}

// views/git/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var page_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.overview = live2(() => ship.git.overview());
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b1 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1, 2],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope10 = Object.create($scope);
      __childScope10.n = n;
      __childScope10.__i = __i;
      return (($scope2) => {
        const __frag11 = document.createDocumentFragment();
        const __c12 = [];
        const __e13 = document.createElement("sui-skeleton");
        __e13.setAttribute("shape", "line");
        __e13.setAttribute("size", "md");
        __frag11.appendChild(__e13);
        return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
          for (const c of __c12)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope10);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag5 = document.createDocumentFragment();
    const __c6 = [];
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "git-skel");
    __e7.setAttribute("data-test", "loading");
    __e7.setAttribute("aria-busy", "true");
    const __e8 = document.createElement("sui-skeleton");
    __e8.setAttribute("shape", "line");
    __e8.setAttribute("size", "lg");
    __e7.appendChild(__e8);
    const __forAnchor9 = document.createComment("for");
    __e7.appendChild(__forAnchor9);
    __c6.push(__wireBoundary_b1(__forAnchor9));
    __frag5.appendChild(__e7);
    return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
      for (const c of __c6)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __wireBoundary_b3 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.isNoRepo())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag19 = document.createDocumentFragment();
        const __c20 = [];
        const __e21 = document.createElement("div");
        __e21.setAttribute("class", "git-no-repo");
        __e21.setAttribute("data-test", "no-repo");
        const __e22 = document.createElement("sui-empty-state");
        __e22.setAttribute("tone", "muted");
        __e22.setAttribute("size", "lg");
        const __e23 = document.createElement("sui-icon");
        __e23.setAttribute("slot", "icon");
        __e23.setAttribute("name", "git-branch");
        __e23.setAttribute("size", "lg");
        __e22.appendChild(__e23);
        const __e24 = document.createElement("span");
        __e24.setAttribute("slot", "title");
        const __t25 = document.createTextNode("Not a git repository");
        __e24.appendChild(__t25);
        __e22.appendChild(__e24);
        const __e26 = document.createElement("span");
        __e26.setAttribute("slot", "description");
        const __t27 = document.createTextNode("");
        __e26.appendChild(__t27);
        __c20.push(bindText(__t27, () => $scope.err.message));
        __e22.appendChild(__e26);
        const __e28 = document.createElement("div");
        __e28.setAttribute("slot", "actions");
        const __e29 = document.createElement("sui-button");
        __e29.setAttribute("tone", "primary");
        __e29.setAttribute("variant", "solid");
        __e29.setAttribute("data-test", "git-init");
        __c20.push(bindAttr(__e29, "loading", () => $scope.busy() === "init" ? "" : null));
        __c20.push(bindAttr(__e29, "disabled", () => $scope.busy() === "init" ? "" : null));
        __c20.push(on(__e29, "click", function() {
          const __h = $scope.initRepo;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e30 = document.createElement("sui-icon");
        __e30.setAttribute("slot", "leading");
        __e30.setAttribute("name", "plus");
        __e30.setAttribute("size", "sm");
        __e29.appendChild(__e30);
        const __t31 = document.createTextNode(`Initialize repository
            `);
        __e29.appendChild(__t31);
        __e28.appendChild(__e29);
        __e22.appendChild(__e28);
        __e21.appendChild(__e22);
        __frag19.appendChild(__e21);
        return { nodes: Array.from(__frag19.childNodes), cleanup: () => {
          for (const c of __c20)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag32 = document.createDocumentFragment();
        const __c33 = [];
        const __e34 = document.createElement("sui-error-state");
        __e34.setAttribute("size", "md");
        __e34.setAttribute("title", "Couldn't load repo");
        __e34.setAttribute("data-test", "error");
        const __t35 = document.createTextNode("");
        __e34.appendChild(__t35);
        __c33.push(bindText(__t35, () => $scope.err.message));
        __frag32.appendChild(__e34);
        return { nodes: Array.from(__frag32.childNodes), cleanup: () => {
          for (const c of __c33)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __renderWhileError_0 = (err) => {
    const __childScope15 = Object.create($scope);
    __childScope15.err = err;
    return (($scope2) => {
      const __frag16 = document.createDocumentFragment();
      const __c17 = [];
      const __ifAnchor18 = document.createComment("if");
      __frag16.appendChild(__ifAnchor18);
      __c17.push(__wireBoundary_b3(__ifAnchor18));
      return { nodes: Array.from(__frag16.childNodes), cleanup: () => {
        for (const c of __c17)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope15);
  };
  const __wireBoundary_b10 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.opError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag41 = document.createDocumentFragment();
        const __c42 = [];
        const __e43 = document.createElement("sui-alert");
        __e43.setAttribute("tone", "destructive");
        __e43.setAttribute("data-test", "git-op-error");
        __e43.setAttribute("dismissible", "");
        __c42.push(on(__e43, "sui-dismiss", function() {
          const __h = $scope.dismissError;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e44 = document.createElement("sui-icon");
        __e44.setAttribute("slot", "icon");
        __e44.setAttribute("name", "alert-circle");
        __e44.setAttribute("size", "sm");
        __e43.appendChild(__e44);
        const __t45 = document.createTextNode("");
        __e43.appendChild(__t45);
        __c42.push(bindText(__t45, () => $scope.opError()));
        __frag41.appendChild(__e43);
        return { nodes: Array.from(__frag41.childNodes), cleanup: () => {
          for (const c of __c42)
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
      if ($scope.opInfo())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag47 = document.createDocumentFragment();
        const __c48 = [];
        const __e49 = document.createElement("sui-alert");
        __e49.setAttribute("tone", "success");
        __e49.setAttribute("data-test", "git-op-info");
        __e49.setAttribute("dismissible", "");
        __c48.push(on(__e49, "sui-dismiss", function() {
          const __h = $scope.dismissError;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e50 = document.createElement("sui-icon");
        __e50.setAttribute("slot", "icon");
        __e50.setAttribute("name", "check-circle");
        __e50.setAttribute("size", "sm");
        __e49.appendChild(__e50);
        const __t51 = document.createTextNode("");
        __e49.appendChild(__t51);
        __c48.push(bindText(__t51, () => $scope.opInfo()));
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
  const __wireBoundary_b48 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.recentCommits().length === 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag117 = document.createDocumentFragment();
        const __c118 = [];
        const __e119 = document.createElement("p");
        __e119.setAttribute("class", "git-empty");
        const __t120 = document.createTextNode("No commits yet.");
        __e119.appendChild(__t120);
        __frag117.appendChild(__e119);
        return { nodes: Array.from(__frag117.childNodes), cleanup: () => {
          for (const c of __c118)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b49 = (anchor) => bindFor({
    anchor,
    items: () => $scope.recentCommits(),
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return (($scope2) => c.sha)(__child);
    },
    render: (c, __i) => {
      const __childScope122 = Object.create($scope);
      __childScope122.c = c;
      __childScope122.__i = __i;
      return (($scope2) => {
        const __frag123 = document.createDocumentFragment();
        const __c124 = [];
        const __e125 = document.createElement("div");
        __e125.setAttribute("class", "git-row");
        const __e126 = document.createElement("sui-avatar");
        __e126.setAttribute("size", "sm");
        __e126.setAttribute("shape", "round");
        {
          const __av127 = $scope2.c.author;
          if (__av127 == null || __av127 === false || __av127 === "")
            __e126.removeAttribute("name");
          else
            __e126.setAttribute("name", __av127 === true ? "" : String(__av127));
        }
        __e125.appendChild(__e126);
        const __e128 = document.createElement("div");
        __e128.setAttribute("class", "git-row-body");
        const __e129 = document.createElement("div");
        __e129.setAttribute("class", "git-row-title");
        const __t130 = document.createTextNode("");
        __e129.appendChild(__t130);
        __t130.textContent = $scope2.c.subject;
        __e128.appendChild(__e129);
        const __e131 = document.createElement("div");
        __e131.setAttribute("class", "git-row-sub");
        const __t132 = document.createTextNode("");
        __e131.appendChild(__t132);
        __t132.textContent = $scope2.c.short_sha;
        const __t133 = document.createTextNode(" · ");
        __e131.appendChild(__t133);
        const __t134 = document.createTextNode("");
        __e131.appendChild(__t134);
        __t134.textContent = $scope2.c.author;
        const __t135 = document.createTextNode(" · ");
        __e131.appendChild(__t135);
        const __t136 = document.createTextNode("");
        __e131.appendChild(__t136);
        __t136.textContent = $scope2.c.date;
        __e128.appendChild(__e131);
        __e125.appendChild(__e128);
        const __e137 = document.createElement("div");
        __e137.setAttribute("class", "git-deltas");
        const __e138 = document.createElement("span");
        __e138.setAttribute("class", "git-add");
        const __t139 = document.createTextNode("+");
        __e138.appendChild(__t139);
        const __t140 = document.createTextNode("");
        __e138.appendChild(__t140);
        __t140.textContent = $scope2.c.additions;
        __e137.appendChild(__e138);
        const __e141 = document.createElement("span");
        __e141.setAttribute("class", "git-del");
        const __t142 = document.createTextNode("−");
        __e141.appendChild(__t142);
        const __t143 = document.createTextNode("");
        __e141.appendChild(__t143);
        __t143.textContent = $scope2.c.deletions;
        __e137.appendChild(__e141);
        __e125.appendChild(__e137);
        __frag123.appendChild(__e125);
        return { nodes: Array.from(__frag123.childNodes), cleanup: () => {
          for (const c2 of __c124)
            try {
              c2();
            } catch (e) {}
        }, update: (__new144, __newI145) => {
          const __old = $scope2.c;
          $scope2.c = __new144;
          $scope2.__i = __newI145;
          if (__old.author !== $scope2.c.author) {
            {
              const __av127 = $scope2.c.author;
              if (__av127 == null || __av127 === false || __av127 === "")
                __e126.removeAttribute("name");
              else
                __e126.setAttribute("name", __av127 === true ? "" : String(__av127));
            }
            __t134.textContent = $scope2.c.author;
          }
          if (__old.subject !== $scope2.c.subject) {
            __t130.textContent = $scope2.c.subject;
          }
          if (__old.short_sha !== $scope2.c.short_sha) {
            __t132.textContent = $scope2.c.short_sha;
          }
          if (__old.date !== $scope2.c.date) {
            __t136.textContent = $scope2.c.date;
          }
          if (__old.additions !== $scope2.c.additions) {
            __t140.textContent = $scope2.c.additions;
          }
          if (__old.deletions !== $scope2.c.deletions) {
            __t143.textContent = $scope2.c.deletions;
          }
        } };
      })(__childScope122);
    }
  });
  const __wireBoundary_b57 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.recentBranches().length === 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag150 = document.createDocumentFragment();
        const __c151 = [];
        const __e152 = document.createElement("p");
        __e152.setAttribute("class", "git-empty");
        const __t153 = document.createTextNode("No branches.");
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
  const __wireBoundary_b58 = (anchor) => bindFor({
    anchor,
    items: () => $scope.recentBranches(),
    key: (b, __i) => {
      const __child = Object.create($scope);
      __child.b = b;
      __child.__i = __i;
      return (($scope2) => b.name)(__child);
    },
    render: (b, __i) => {
      const __childScope155 = Object.create($scope);
      __childScope155.b = b;
      __childScope155.__i = __i;
      return (($scope2) => {
        const __frag156 = document.createDocumentFragment();
        const __c157 = [];
        const __e158 = document.createElement("div");
        __e158.setAttribute("class", "git-row");
        const __e159 = document.createElement("sui-icon");
        __e159.setAttribute("name", "git-branch");
        __e159.setAttribute("size", "xs");
        {
          const __av160 = $scope2.b.active ? "git-icon-active" : "git-icon-muted";
          if (__av160 == null || __av160 === false || __av160 === "")
            __e159.removeAttribute("class");
          else
            __e159.setAttribute("class", __av160 === true ? "" : String(__av160));
        }
        __e158.appendChild(__e159);
        const __e161 = document.createElement("div");
        __e161.setAttribute("class", "git-row-body");
        const __e162 = document.createElement("div");
        __e162.setAttribute("class", "git-row-title git-mono");
        const __t163 = document.createTextNode("");
        __e162.appendChild(__t163);
        __t163.textContent = $scope2.b.name;
        __e161.appendChild(__e162);
        const __e164 = document.createElement("div");
        __e164.setAttribute("class", "git-row-sub");
        const __t165 = document.createTextNode("");
        __e164.appendChild(__t165);
        __t165.textContent = $scope2.b.upstream ? $scope2.b.upstream : "local only";
        __e161.appendChild(__e164);
        __e158.appendChild(__e161);
        const __e166 = document.createElement("div");
        __e166.setAttribute("class", "git-deltas");
        const __t167 = document.createTextNode("");
        __e166.appendChild(__t167);
        __t167.textContent = $scope2.b.ahead > 0 ? "↑" + $scope2.b.ahead : "";
        const __t168 = document.createTextNode("");
        __e166.appendChild(__t168);
        __t168.textContent = $scope2.b.behind > 0 ? " ↓" + $scope2.b.behind : "";
        __e158.appendChild(__e166);
        __frag156.appendChild(__e158);
        return { nodes: Array.from(__frag156.childNodes), cleanup: () => {
          for (const c of __c157)
            try {
              c();
            } catch (e) {}
        }, update: (__new169, __newI170) => {
          const __old = $scope2.b;
          $scope2.b = __new169;
          $scope2.__i = __newI170;
          if (__old.active !== $scope2.b.active) {
            {
              const __av160 = $scope2.b.active ? "git-icon-active" : "git-icon-muted";
              if (__av160 == null || __av160 === false || __av160 === "")
                __e159.removeAttribute("class");
              else
                __e159.setAttribute("class", __av160 === true ? "" : String(__av160));
            }
          }
          if (__old.name !== $scope2.b.name) {
            __t163.textContent = $scope2.b.name;
          }
          if (__old.upstream !== $scope2.b.upstream) {
            __t165.textContent = $scope2.b.upstream ? $scope2.b.upstream : "local only";
          }
          if (__old.ahead !== $scope2.b.ahead) {
            __t167.textContent = $scope2.b.ahead > 0 ? "↑" + $scope2.b.ahead : "";
          }
          if (__old.behind !== $scope2.b.behind) {
            __t168.textContent = $scope2.b.behind > 0 ? " ↓" + $scope2.b.behind : "";
          }
        } };
      })(__childScope155);
    }
  });
  const __wireBoundary_b72 = (anchor) => bindFor({
    anchor,
    items: () => $scope.fileGroups(),
    key: (g, __i) => {
      const __child = Object.create($scope);
      __child.g = g;
      __child.__i = __i;
      return (($scope2) => g.key)(__child);
    },
    render: (g, __i) => {
      const __childScope198 = Object.create($scope);
      __childScope198.g = g;
      __childScope198.__i = __i;
      return (($scope2) => {
        const __frag199 = document.createDocumentFragment();
        const __c200 = [];
        const __e201 = document.createElement("sui-chip");
        {
          const __av202 = $scope2.g.tone;
          if (__av202 == null || __av202 === false || __av202 === "")
            __e201.removeAttribute("tone");
          else
            __e201.setAttribute("tone", __av202 === true ? "" : String(__av202));
        }
        {
          const __av203 = $scope2.g.variant;
          if (__av203 == null || __av203 === false || __av203 === "")
            __e201.removeAttribute("variant");
          else
            __e201.setAttribute("variant", __av203 === true ? "" : String(__av203));
        }
        __e201.setAttribute("size", "sm");
        const __t204 = document.createTextNode("");
        __e201.appendChild(__t204);
        __t204.textContent = $scope2.g.items.length;
        const __t205 = document.createTextNode("");
        __e201.appendChild(__t205);
        __t205.textContent = $scope2.g.key;
        __frag199.appendChild(__e201);
        return { nodes: Array.from(__frag199.childNodes), cleanup: () => {
          for (const c of __c200)
            try {
              c();
            } catch (e) {}
        }, update: (__new206, __newI207) => {
          const __old = $scope2.g;
          $scope2.g = __new206;
          $scope2.__i = __newI207;
          if (__old.tone !== $scope2.g.tone) {
            {
              const __av202 = $scope2.g.tone;
              if (__av202 == null || __av202 === false || __av202 === "")
                __e201.removeAttribute("tone");
              else
                __e201.setAttribute("tone", __av202 === true ? "" : String(__av202));
            }
          }
          if (__old.variant !== $scope2.g.variant) {
            {
              const __av203 = $scope2.g.variant;
              if (__av203 == null || __av203 === false || __av203 === "")
                __e201.removeAttribute("variant");
              else
                __e201.setAttribute("variant", __av203 === true ? "" : String(__av203));
            }
          }
          if (__old.items !== $scope2.g.items) {
            __t204.textContent = $scope2.g.items.length;
          }
          if (__old.key !== $scope2.g.key) {
            __t205.textContent = $scope2.g.key;
          }
        } };
      })(__childScope198);
    }
  });
  const __wireBoundary_b77 = (anchor) => bindFor({
    anchor,
    items: () => $scope.fileGroups(),
    key: (g, __i) => {
      const __child = Object.create($scope);
      __child.g = g;
      __child.__i = __i;
      return (($scope2) => g.key)(__child);
    },
    render: (g, __i) => {
      const __childScope210 = Object.create($scope);
      __childScope210.g = g;
      __childScope210.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b79 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.g.items,
          key: (f, __i2) => {
            const __child = Object.create($scope2);
            __child.f = f;
            __child.__i = __i2;
            return (($scope3) => f.path)(__child);
          },
          render: (f, __i2) => {
            const __childScope216 = Object.create($scope2);
            __childScope216.f = f;
            __childScope216.__i = __i2;
            return (($scope3) => {
              __childScope216.__v_path = signal2(0);
              const __wireBoundary_b84 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.g.key === "staged")
                    return 0;
                  return 1;
                },
                branches: [
                  () => {
                    const __frag225 = document.createDocumentFragment();
                    const __c226 = [];
                    const __e227 = document.createElement("button");
                    __e227.setAttribute("class", "git-file-act");
                    __e227.setAttribute("type", "button");
                    __e227.setAttribute("data-test", "git-unstage-file");
                    __e227.setAttribute("aria-label", "Unstage file");
                    __e227.setAttribute("title", "Unstage");
                    __c226.push(on(__e227, "click", function() {
                      const __h = () => $scope3.unstageFile($scope3.f);
                      return typeof __h === "function" ? __h.bind(undefined) : () => {};
                    }(), { modifiers: [] }));
                    const __t228 = document.createTextNode("−");
                    __e227.appendChild(__t228);
                    __frag225.appendChild(__e227);
                    return { nodes: Array.from(__frag225.childNodes), cleanup: () => {
                      for (const c of __c226)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  },
                  () => {
                    const __frag229 = document.createDocumentFragment();
                    const __c230 = [];
                    const __e231 = document.createElement("button");
                    __e231.setAttribute("class", "git-file-act");
                    __e231.setAttribute("type", "button");
                    __e231.setAttribute("data-test", "git-stage-file");
                    __e231.setAttribute("aria-label", "Stage file");
                    __e231.setAttribute("title", "Stage");
                    __c230.push(on(__e231, "click", function() {
                      const __h = () => $scope3.stageFile($scope3.f);
                      return typeof __h === "function" ? __h.bind(undefined) : () => {};
                    }(), { modifiers: [] }));
                    const __t232 = document.createTextNode("+");
                    __e231.appendChild(__t232);
                    __frag229.appendChild(__e231);
                    return { nodes: Array.from(__frag229.childNodes), cleanup: () => {
                      for (const c of __c230)
                        try {
                          c();
                        } catch (e) {}
                    } };
                  }
                ]
              });
              const __frag217 = document.createDocumentFragment();
              const __c218 = [];
              const __e219 = document.createElement("div");
              __e219.setAttribute("class", "git-file-row-wrap");
              __c218.push(bindAttr(__e219, "data-active", () => (void $scope3.__v_path(), $scope3.selectedFile() === $scope3.f.path ? "" : null)));
              const __e220 = document.createElement("button");
              __e220.setAttribute("class", "git-file-row git-mono");
              __e220.setAttribute("type", "button");
              __e220.setAttribute("data-test", "git-file-row");
              __c218.push(on(__e220, "click", function() {
                const __h = () => $scope3.selectFile($scope3.f);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              const __e221 = document.createElement("span");
              __e221.setAttribute("class", "git-file-status");
              const __t222 = document.createTextNode("");
              __e221.appendChild(__t222);
              __t222.textContent = $scope3.g.glyph || $scope3.f.status;
              __e220.appendChild(__e221);
              const __t223 = document.createTextNode("");
              __e220.appendChild(__t223);
              __t223.textContent = $scope3.f.path;
              __e219.appendChild(__e220);
              const __ifAnchor224 = document.createComment("if");
              __e219.appendChild(__ifAnchor224);
              __c218.push(__wireBoundary_b84(__ifAnchor224));
              __frag217.appendChild(__e219);
              return { nodes: Array.from(__frag217.childNodes), cleanup: () => {
                for (const c of __c218)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new233, __newI234) => {
                const __old = $scope3.f;
                $scope3.f = __new233;
                $scope3.__i = __newI234;
                if (__old.status !== $scope3.f.status) {
                  __t222.textContent = $scope3.g.glyph || $scope3.f.status;
                }
                if (__old.path !== $scope3.f.path) {
                  __t223.textContent = $scope3.f.path;
                }
                if (__old.path !== $scope3.f.path) {
                  $scope3.__v_path.set((c) => c + 1);
                }
              } };
            })(__childScope216);
          }
        });
        const __frag211 = document.createDocumentFragment();
        const __c212 = [];
        const __e213 = document.createElement("div");
        __e213.setAttribute("class", "git-file-group-head");
        const __t214 = document.createTextNode("");
        __e213.appendChild(__t214);
        __t214.textContent = $scope2.g.label;
        __frag211.appendChild(__e213);
        const __forAnchor215 = document.createComment("for");
        __frag211.appendChild(__forAnchor215);
        __c212.push(__wireBoundary_b79(__forAnchor215));
        return { nodes: Array.from(__frag211.childNodes), cleanup: () => {
          for (const c of __c212)
            try {
              c();
            } catch (e) {}
        }, update: (__new235, __newI236) => {
          const __old = $scope2.g;
          $scope2.g = __new235;
          $scope2.__i = __newI236;
          if (__old.label !== $scope2.g.label) {
            __t214.textContent = $scope2.g.label;
          }
        } };
      })(__childScope210);
    }
  });
  const __wireBoundary_b93 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.diffIsWorking())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag275 = document.createDocumentFragment();
        const __c276 = [];
        const __e277 = document.createElement("sui-alert");
        __e277.setAttribute("tone", "muted");
        __e277.setAttribute("class", "git-diff-banner");
        const __e278 = document.createElement("sui-icon");
        __e278.setAttribute("slot", "icon");
        __e278.setAttribute("name", "git-commit");
        __e278.setAttribute("size", "xs");
        __e277.appendChild(__e278);
        const __t279 = document.createTextNode("Viewing working tree changes vs HEAD.");
        __e277.appendChild(__t279);
        __frag275.appendChild(__e277);
        return { nodes: Array.from(__frag275.childNodes), cleanup: () => {
          for (const c of __c276)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b94 = (anchor) => bindFor({
    anchor,
    items: () => $scope.diffFiles(),
    key: (file, __i) => {
      const __child = Object.create($scope);
      __child.file = file;
      __child.__i = __i;
      return (($scope2) => file.path)(__child);
    },
    render: (file, __i) => {
      const __childScope282 = Object.create($scope);
      __childScope282.file = file;
      __childScope282.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b98 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.file.hunks,
          key: (hunk, __i2) => {
            const __child = Object.create($scope2);
            __child.hunk = hunk;
            __child.__i = __i2;
            return (($scope3) => hunk.header)(__child);
          },
          render: (hunk, __i2) => {
            const __childScope291 = Object.create($scope2);
            __childScope291.hunk = hunk;
            __childScope291.__i = __i2;
            return (($scope3) => {
              const __wireBoundary_b100 = (anchor3) => bindFor({
                anchor: anchor3,
                items: () => $scope3.hunk.lines,
                key: (line, __i3) => {
                  const __child = Object.create($scope3);
                  __child.line = line;
                  __child.__i = __i3;
                  return (($scope4) => (line.old_num || 0) + ":" + (line.new_num || 0))(__child);
                },
                render: (line, __i3) => {
                  const __childScope297 = Object.create($scope3);
                  __childScope297.line = line;
                  __childScope297.__i = __i3;
                  return (($scope4) => {
                    __childScope297.__v_new_num = signal2(0);
                    __childScope297.__v_old_num = signal2(0);
                    const __frag298 = document.createDocumentFragment();
                    const __c299 = [];
                    const __e300 = document.createElement("sui-git-diff-line");
                    {
                      const __av301 = $scope4.line.kind;
                      if (__av301 == null || __av301 === false || __av301 === "")
                        __e300.removeAttribute("type");
                      else
                        __e300.setAttribute("type", __av301 === true ? "" : String(__av301));
                    }
                    __c299.push(bindAttr(__e300, "old", () => (void $scope4.__v_old_num(), $scope4.line.old_num ? $scope4.line.old_num + "" : null)));
                    __c299.push(bindAttr(__e300, "new", () => (void $scope4.__v_new_num(), $scope4.line.new_num ? $scope4.line.new_num + "" : null)));
                    const __t302 = document.createTextNode("");
                    __e300.appendChild(__t302);
                    __t302.textContent = $scope4.line.content;
                    __frag298.appendChild(__e300);
                    return { nodes: Array.from(__frag298.childNodes), cleanup: () => {
                      for (const c of __c299)
                        try {
                          c();
                        } catch (e) {}
                    }, update: (__new303, __newI304) => {
                      const __old = $scope4.line;
                      $scope4.line = __new303;
                      $scope4.__i = __newI304;
                      if (__old.kind !== $scope4.line.kind) {
                        {
                          const __av301 = $scope4.line.kind;
                          if (__av301 == null || __av301 === false || __av301 === "")
                            __e300.removeAttribute("type");
                          else
                            __e300.setAttribute("type", __av301 === true ? "" : String(__av301));
                        }
                      }
                      if (__old.content !== $scope4.line.content) {
                        __t302.textContent = $scope4.line.content;
                      }
                      if (__old.new_num !== $scope4.line.new_num) {
                        $scope4.__v_new_num.set((c) => c + 1);
                      }
                      if (__old.old_num !== $scope4.line.old_num) {
                        $scope4.__v_old_num.set((c) => c + 1);
                      }
                    } };
                  })(__childScope297);
                }
              });
              const __frag292 = document.createDocumentFragment();
              const __c293 = [];
              const __e294 = document.createElement("sui-git-diff-hunk");
              {
                const __av295 = $scope3.hunk.header;
                if (__av295 == null || __av295 === false || __av295 === "")
                  __e294.removeAttribute("header");
                else
                  __e294.setAttribute("header", __av295 === true ? "" : String(__av295));
              }
              __frag292.appendChild(__e294);
              const __forAnchor296 = document.createComment("for");
              __frag292.appendChild(__forAnchor296);
              __c293.push(__wireBoundary_b100(__forAnchor296));
              return { nodes: Array.from(__frag292.childNodes), cleanup: () => {
                for (const c of __c293)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new305, __newI306) => {
                const __old = $scope3.hunk;
                $scope3.hunk = __new305;
                $scope3.__i = __newI306;
                if (__old.header !== $scope3.hunk.header) {
                  {
                    const __av295 = $scope3.hunk.header;
                    if (__av295 == null || __av295 === false || __av295 === "")
                      __e294.removeAttribute("header");
                    else
                      __e294.setAttribute("header", __av295 === true ? "" : String(__av295));
                  }
                }
              } };
            })(__childScope291);
          }
        });
        const __frag283 = document.createDocumentFragment();
        const __c284 = [];
        const __e285 = document.createElement("sui-git-diff");
        __e285.setAttribute("files", "1");
        const __e286 = document.createElement("sui-git-diff-file");
        {
          const __av287 = $scope2.file.path;
          if (__av287 == null || __av287 === false || __av287 === "")
            __e286.removeAttribute("path");
          else
            __e286.setAttribute("path", __av287 === true ? "" : String(__av287));
        }
        {
          const __av288 = $scope2.file.additions + "";
          if (__av288 == null || __av288 === false || __av288 === "")
            __e286.removeAttribute("additions");
          else
            __e286.setAttribute("additions", __av288 === true ? "" : String(__av288));
        }
        {
          const __av289 = $scope2.file.deletions + "";
          if (__av289 == null || __av289 === false || __av289 === "")
            __e286.removeAttribute("deletions");
          else
            __e286.setAttribute("deletions", __av289 === true ? "" : String(__av289));
        }
        const __forAnchor290 = document.createComment("for");
        __e286.appendChild(__forAnchor290);
        __c284.push(__wireBoundary_b98(__forAnchor290));
        __e285.appendChild(__e286);
        __frag283.appendChild(__e285);
        return { nodes: Array.from(__frag283.childNodes), cleanup: () => {
          for (const c of __c284)
            try {
              c();
            } catch (e) {}
        }, update: (__new307, __newI308) => {
          const __old = $scope2.file;
          $scope2.file = __new307;
          $scope2.__i = __newI308;
          if (__old.path !== $scope2.file.path) {
            {
              const __av287 = $scope2.file.path;
              if (__av287 == null || __av287 === false || __av287 === "")
                __e286.removeAttribute("path");
              else
                __e286.setAttribute("path", __av287 === true ? "" : String(__av287));
            }
          }
          if (__old.additions !== $scope2.file.additions) {
            {
              const __av288 = $scope2.file.additions + "";
              if (__av288 == null || __av288 === false || __av288 === "")
                __e286.removeAttribute("additions");
              else
                __e286.setAttribute("additions", __av288 === true ? "" : String(__av288));
            }
          }
          if (__old.deletions !== $scope2.file.deletions) {
            {
              const __av289 = $scope2.file.deletions + "";
              if (__av289 == null || __av289 === false || __av289 === "")
                __e286.removeAttribute("deletions");
              else
                __e286.setAttribute("deletions", __av289 === true ? "" : String(__av289));
            }
          }
        } };
      })(__childScope282);
    }
  });
  const __wireBoundary_b87 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (!$scope.selectedCommit())
        return 0;
      if (!$scope.diffReady())
        return 1;
      if ($scope.diffFiles().length === 0)
        return 2;
      return 3;
    },
    branches: [
      () => {
        const __frag239 = document.createDocumentFragment();
        const __c240 = [];
        const __e241 = document.createElement("sui-empty-state");
        __e241.setAttribute("tone", "muted");
        __e241.setAttribute("size", "md");
        __e241.setAttribute("class", "git-diff-empty");
        const __e242 = document.createElement("span");
        __e242.setAttribute("slot", "title");
        const __t243 = document.createTextNode("Select a file");
        __e242.appendChild(__t243);
        __e241.appendChild(__e242);
        const __t244 = document.createTextNode(`
                  Pick a file on the left to view its diff.
                `);
        __e241.appendChild(__t244);
        __frag239.appendChild(__e241);
        return { nodes: Array.from(__frag239.childNodes), cleanup: () => {
          for (const c of __c240)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag245 = document.createDocumentFragment();
        const __c246 = [];
        const __e247 = document.createElement("div");
        __e247.setAttribute("class", "git-diff-loading");
        __e247.setAttribute("data-test", "diff-loading");
        const __t248 = document.createTextNode("Loading diff…");
        __e247.appendChild(__t248);
        __frag245.appendChild(__e247);
        return { nodes: Array.from(__frag245.childNodes), cleanup: () => {
          for (const c of __c246)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag249 = document.createDocumentFragment();
        const __c250 = [];
        const __e251 = document.createElement("sui-empty-state");
        __e251.setAttribute("tone", "muted");
        __e251.setAttribute("size", "md");
        __e251.setAttribute("class", "git-diff-empty");
        const __e252 = document.createElement("span");
        __e252.setAttribute("slot", "title");
        const __t253 = document.createTextNode("No changes in this diff");
        __e252.appendChild(__t253);
        __e251.appendChild(__e252);
        __frag249.appendChild(__e251);
        return { nodes: Array.from(__frag249.childNodes), cleanup: () => {
          for (const c of __c250)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag254 = document.createDocumentFragment();
        const __c255 = [];
        const __e256 = document.createElement("sui-toolbar");
        __e256.setAttribute("variant", "bordered");
        __e256.setAttribute("class", "git-diff-toolbar");
        const __e257 = document.createElement("sui-badge");
        __e257.setAttribute("tone", "info");
        __e257.setAttribute("variant", "soft");
        __e257.setAttribute("size", "sm");
        const __t258 = document.createTextNode("diff");
        __e257.appendChild(__t258);
        __e256.appendChild(__e257);
        const __e259 = document.createElement("span");
        __e259.setAttribute("class", "git-mono git-diff-path");
        __e259.setAttribute("data-test", "diff-path");
        const __t260 = document.createTextNode("");
        __e259.appendChild(__t260);
        __c255.push(bindText(__t260, () => $scope.diffFiles()[0].path));
        __e256.appendChild(__e259);
        const __e261 = document.createElement("span");
        __e261.setAttribute("slot", "spacer");
        __e256.appendChild(__e261);
        const __e262 = document.createElement("span");
        __e262.setAttribute("class", "git-mono");
        const __e263 = document.createElement("span");
        __e263.setAttribute("class", "git-add");
        const __t264 = document.createTextNode("+");
        __e263.appendChild(__t264);
        const __t265 = document.createTextNode("");
        __e263.appendChild(__t265);
        __c255.push(bindText(__t265, () => $scope.diffFiles()[0].additions));
        __e262.appendChild(__e263);
        const __e266 = document.createElement("span");
        __e266.setAttribute("class", "git-del");
        const __t267 = document.createTextNode("−");
        __e266.appendChild(__t267);
        const __t268 = document.createTextNode("");
        __e266.appendChild(__t268);
        __c255.push(bindText(__t268, () => $scope.diffFiles()[0].deletions));
        __e262.appendChild(__e266);
        __e256.appendChild(__e262);
        const __e269 = document.createElement("sui-segmented");
        __c255.push(bindAttr(__e269, "value", () => $scope.diffMode()));
        __e269.setAttribute("size", "sm");
        __c255.push(on(__e269, "sui-change", function() {
          const __h = (e) => $scope.setDiffMode(e.detail.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e270 = document.createElement("sui-segmented-option");
        __e270.setAttribute("value", "unified");
        const __t271 = document.createTextNode("Unified");
        __e270.appendChild(__t271);
        __e269.appendChild(__e270);
        const __e272 = document.createElement("sui-segmented-option");
        __e272.setAttribute("value", "split");
        const __t273 = document.createTextNode("Split");
        __e272.appendChild(__t273);
        __e269.appendChild(__e272);
        __e256.appendChild(__e269);
        __frag254.appendChild(__e256);
        const __ifAnchor274 = document.createComment("if");
        __frag254.appendChild(__ifAnchor274);
        __c255.push(__wireBoundary_b93(__ifAnchor274));
        const __e280 = document.createElement("sui-scroll-area");
        __e280.setAttribute("class", "git-diff-scroll");
        __e280.setAttribute("data-test", "diff-body");
        const __forAnchor281 = document.createComment("for");
        __e280.appendChild(__forAnchor281);
        __c255.push(__wireBoundary_b94(__forAnchor281));
        __frag254.appendChild(__e280);
        return { nodes: Array.from(__frag254.childNodes), cleanup: () => {
          for (const c of __c255)
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
      if ($scope.clean())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag95 = document.createDocumentFragment();
        const __c96 = [];
        const __e97 = document.createElement("div");
        __e97.setAttribute("class", "git-clean");
        const __e98 = document.createElement("div");
        __e98.setAttribute("class", "git-hero");
        const __e99 = document.createElement("div");
        __e99.setAttribute("class", "git-halo");
        const __e100 = document.createElement("sui-icon");
        __e100.setAttribute("name", "check");
        __e100.setAttribute("size", "md");
        __e99.appendChild(__e100);
        __e98.appendChild(__e99);
        const __e101 = document.createElement("div");
        const __e102 = document.createElement("h2");
        __e102.setAttribute("class", "git-hero-title");
        const __t103 = document.createTextNode("Working tree is clean.");
        __e102.appendChild(__t103);
        __e101.appendChild(__e102);
        const __e104 = document.createElement("p");
        __e104.setAttribute("class", "git-hero-sub");
        const __t105 = document.createTextNode(`
                  Nothing to commit on `);
        __e104.appendChild(__t105);
        const __e106 = document.createElement("span");
        __e106.setAttribute("class", "git-mono");
        const __t107 = document.createTextNode("");
        __e106.appendChild(__t107);
        __c96.push(bindText(__t107, () => $scope.branch()));
        __e104.appendChild(__e106);
        const __t108 = document.createTextNode(`.
                  `);
        __e104.appendChild(__t108);
        const __t109 = document.createTextNode("");
        __e104.appendChild(__t109);
        __c96.push(bindText(__t109, () => $scope.ahead() === 0 && $scope.behind() === 0 ? "You're up to date with the upstream." : ""));
        const __t110 = document.createTextNode("");
        __e104.appendChild(__t110);
        __c96.push(bindText(__t110, () => $scope.ahead() > 0 ? $scope.ahead() + " commit" + ($scope.ahead() === 1 ? "" : "s") + " ahead of upstream." : ""));
        const __t111 = document.createTextNode("");
        __e104.appendChild(__t111);
        __c96.push(bindText(__t111, () => $scope.behind() > 0 ? " " + $scope.behind() + " commit" + ($scope.behind() === 1 ? "" : "s") + " behind upstream." : ""));
        __e101.appendChild(__e104);
        __e98.appendChild(__e101);
        __e97.appendChild(__e98);
        const __e112 = document.createElement("div");
        __e112.setAttribute("class", "git-two-col");
        const __e113 = document.createElement("section");
        __e113.setAttribute("class", "git-sec");
        const __e114 = document.createElement("h3");
        const __t115 = document.createTextNode("Recent on this branch");
        __e114.appendChild(__t115);
        __e113.appendChild(__e114);
        const __ifAnchor116 = document.createComment("if");
        __e113.appendChild(__ifAnchor116);
        __c96.push(__wireBoundary_b48(__ifAnchor116));
        const __forAnchor121 = document.createComment("for");
        __e113.appendChild(__forAnchor121);
        __c96.push(__wireBoundary_b49(__forAnchor121));
        __e112.appendChild(__e113);
        const __e146 = document.createElement("section");
        __e146.setAttribute("class", "git-sec");
        const __e147 = document.createElement("h3");
        const __t148 = document.createTextNode("Branches");
        __e147.appendChild(__t148);
        __e146.appendChild(__e147);
        const __ifAnchor149 = document.createComment("if");
        __e146.appendChild(__ifAnchor149);
        __c96.push(__wireBoundary_b57(__ifAnchor149));
        const __forAnchor154 = document.createComment("for");
        __e146.appendChild(__forAnchor154);
        __c96.push(__wireBoundary_b58(__forAnchor154));
        __e112.appendChild(__e146);
        __e97.appendChild(__e112);
        __frag95.appendChild(__e97);
        return { nodes: Array.from(__frag95.childNodes), cleanup: () => {
          for (const c of __c96)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag171 = document.createDocumentFragment();
        const __c172 = [];
        const __e173 = document.createElement("div");
        __e173.setAttribute("class", "git-split");
        const __e174 = document.createElement("aside");
        __e174.setAttribute("class", "git-left");
        __e174.setAttribute("aria-label", "Changed files");
        const __e175 = document.createElement("sui-toolbar");
        __e175.setAttribute("variant", "bordered");
        __e175.setAttribute("class", "git-left-head");
        const __e176 = document.createElement("sui-icon");
        __e176.setAttribute("name", "git-branch");
        __e176.setAttribute("size", "xs");
        __e175.appendChild(__e176);
        const __e177 = document.createElement("strong");
        __e177.setAttribute("class", "git-left-title");
        const __t178 = document.createTextNode("Changes");
        __e177.appendChild(__t178);
        __e175.appendChild(__e177);
        const __e179 = document.createElement("span");
        __e179.setAttribute("slot", "spacer");
        __e175.appendChild(__e179);
        const __e180 = document.createElement("span");
        __e180.setAttribute("class", "git-mono");
        const __t181 = document.createTextNode("");
        __e180.appendChild(__t181);
        __c172.push(bindText(__t181, () => $scope.totalChanged()));
        const __t182 = document.createTextNode(" files");
        __e180.appendChild(__t182);
        __e175.appendChild(__e180);
        __e174.appendChild(__e175);
        const __e183 = document.createElement("div");
        __e183.setAttribute("class", "git-bulk-row");
        const __e184 = document.createElement("sui-button");
        __e184.setAttribute("size", "xs");
        __e184.setAttribute("variant", "outline");
        __e184.setAttribute("data-test", "git-stage-all");
        __c172.push(bindAttr(__e184, "disabled", () => $scope.unstagedCount() === 0 || $scope.busy() ? "" : null));
        __c172.push(on(__e184, "click", function() {
          const __h = $scope.stageAll;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e185 = document.createElement("sui-icon");
        __e185.setAttribute("slot", "leading");
        __e185.setAttribute("name", "plus");
        __e185.setAttribute("size", "xs");
        __e184.appendChild(__e185);
        const __t186 = document.createTextNode(`Stage all
                `);
        __e184.appendChild(__t186);
        __e183.appendChild(__e184);
        const __e187 = document.createElement("sui-button");
        __e187.setAttribute("size", "xs");
        __e187.setAttribute("variant", "outline");
        __e187.setAttribute("data-test", "git-unstage-all");
        __c172.push(bindAttr(__e187, "disabled", () => $scope.stagedCount() === 0 || $scope.busy() ? "" : null));
        __c172.push(on(__e187, "click", function() {
          const __h = $scope.unstageAll;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e188 = document.createElement("sui-icon");
        __e188.setAttribute("slot", "leading");
        __e188.setAttribute("name", "minus");
        __e188.setAttribute("size", "xs");
        __e187.appendChild(__e188);
        const __t189 = document.createTextNode(`Unstage all
                `);
        __e187.appendChild(__t189);
        __e183.appendChild(__e187);
        const __e190 = document.createElement("span");
        __e190.setAttribute("class", "git-bulk-spacer");
        __e183.appendChild(__e190);
        const __e191 = document.createElement("sui-button");
        __e191.setAttribute("size", "xs");
        __e191.setAttribute("variant", "solid");
        __e191.setAttribute("tone", "primary");
        __e191.setAttribute("data-test", "git-open-commit");
        __c172.push(bindAttr(__e191, "disabled", () => !$scope.canCommit() || $scope.busy() ? "" : null));
        __c172.push(on(__e191, "click", function() {
          const __h = $scope.openCommit;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e192 = document.createElement("sui-icon");
        __e192.setAttribute("slot", "leading");
        __e192.setAttribute("name", "git-commit");
        __e192.setAttribute("size", "xs");
        __e191.appendChild(__e192);
        const __t193 = document.createTextNode("Commit (");
        __e191.appendChild(__t193);
        const __t194 = document.createTextNode("");
        __e191.appendChild(__t194);
        __c172.push(bindText(__t194, () => $scope.stagedCount()));
        const __t195 = document.createTextNode(`)
                `);
        __e191.appendChild(__t195);
        __e183.appendChild(__e191);
        __e174.appendChild(__e183);
        const __e196 = document.createElement("div");
        __e196.setAttribute("class", "git-chips-row");
        const __forAnchor197 = document.createComment("for");
        __e196.appendChild(__forAnchor197);
        __c172.push(__wireBoundary_b72(__forAnchor197));
        __e174.appendChild(__e196);
        const __e208 = document.createElement("sui-scroll-area");
        __e208.setAttribute("class", "git-file-list");
        const __forAnchor209 = document.createComment("for");
        __e208.appendChild(__forAnchor209);
        __c172.push(__wireBoundary_b77(__forAnchor209));
        __e174.appendChild(__e208);
        __e173.appendChild(__e174);
        const __e237 = document.createElement("section");
        __e237.setAttribute("class", "git-right");
        __e237.setAttribute("aria-label", "Diff");
        const __ifAnchor238 = document.createComment("if");
        __e237.appendChild(__ifAnchor238);
        __c172.push(__wireBoundary_b87(__ifAnchor238));
        __e173.appendChild(__e237);
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
  const __wireBoundary_b42 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tab() === "changes")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag92 = document.createDocumentFragment();
        const __c93 = [];
        const __ifAnchor94 = document.createComment("if");
        __frag92.appendChild(__ifAnchor94);
        __c93.push(__wireBoundary_b43(__ifAnchor94));
        return { nodes: Array.from(__frag92.childNodes), cleanup: () => {
          for (const c of __c93)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b110 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.commitsCount() === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag324 = document.createDocumentFragment();
        const __c325 = [];
        const __e326 = document.createElement("span");
        __e326.setAttribute("slot", "title");
        const __t327 = document.createTextNode("No commits");
        __e326.appendChild(__t327);
        __frag324.appendChild(__e326);
        return { nodes: Array.from(__frag324.childNodes), cleanup: () => {
          for (const c of __c325)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag328 = document.createDocumentFragment();
        const __c329 = [];
        const __e330 = document.createElement("span");
        __e330.setAttribute("slot", "title");
        const __t331 = document.createTextNode("No matching commits");
        __e330.appendChild(__t331);
        __frag328.appendChild(__e330);
        return { nodes: Array.from(__frag328.childNodes), cleanup: () => {
          for (const c of __c329)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b111 = (anchor) => bindFor({
    anchor,
    items: () => $scope.commitFilter.filtered(),
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return (($scope2) => c.sha)(__child);
    },
    render: (c, __i) => {
      const __childScope336 = Object.create($scope);
      __childScope336.c = c;
      __childScope336.__i = __i;
      return (($scope2) => {
        const __frag337 = document.createDocumentFragment();
        const __c338 = [];
        const __e339 = document.createElement("sui-history-row");
        {
          const __av340 = $scope2.c.short_sha;
          if (__av340 == null || __av340 === false || __av340 === "")
            __e339.removeAttribute("sha");
          else
            __e339.setAttribute("sha", __av340 === true ? "" : String(__av340));
        }
        {
          const __av341 = $scope2.c.subject;
          if (__av341 == null || __av341 === false || __av341 === "")
            __e339.removeAttribute("subject");
          else
            __e339.setAttribute("subject", __av341 === true ? "" : String(__av341));
        }
        {
          const __av342 = $scope2.c.author;
          if (__av342 == null || __av342 === false || __av342 === "")
            __e339.removeAttribute("author");
          else
            __e339.setAttribute("author", __av342 === true ? "" : String(__av342));
        }
        {
          const __av343 = $scope2.c.date;
          if (__av343 == null || __av343 === false || __av343 === "")
            __e339.removeAttribute("timestamp");
          else
            __e339.setAttribute("timestamp", __av343 === true ? "" : String(__av343));
        }
        {
          const __av344 = $scope2.c.additions + "";
          if (__av344 == null || __av344 === false || __av344 === "")
            __e339.removeAttribute("additions");
          else
            __e339.setAttribute("additions", __av344 === true ? "" : String(__av344));
        }
        {
          const __av345 = $scope2.c.deletions + "";
          if (__av345 == null || __av345 === false || __av345 === "")
            __e339.removeAttribute("deletions");
          else
            __e339.setAttribute("deletions", __av345 === true ? "" : String(__av345));
        }
        __c338.push(on(__e339, "sui-select", function() {
          const __h = () => $scope2.selectCommit($scope2.c.sha);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t346 = document.createTextNode(`
                `);
        __e339.appendChild(__t346);
        __frag337.appendChild(__e339);
        return { nodes: Array.from(__frag337.childNodes), cleanup: () => {
          for (const c2 of __c338)
            try {
              c2();
            } catch (e) {}
        }, update: (__new347, __newI348) => {
          const __old = $scope2.c;
          $scope2.c = __new347;
          $scope2.__i = __newI348;
          if (__old.short_sha !== $scope2.c.short_sha) {
            {
              const __av340 = $scope2.c.short_sha;
              if (__av340 == null || __av340 === false || __av340 === "")
                __e339.removeAttribute("sha");
              else
                __e339.setAttribute("sha", __av340 === true ? "" : String(__av340));
            }
          }
          if (__old.subject !== $scope2.c.subject) {
            {
              const __av341 = $scope2.c.subject;
              if (__av341 == null || __av341 === false || __av341 === "")
                __e339.removeAttribute("subject");
              else
                __e339.setAttribute("subject", __av341 === true ? "" : String(__av341));
            }
          }
          if (__old.author !== $scope2.c.author) {
            {
              const __av342 = $scope2.c.author;
              if (__av342 == null || __av342 === false || __av342 === "")
                __e339.removeAttribute("author");
              else
                __e339.setAttribute("author", __av342 === true ? "" : String(__av342));
            }
          }
          if (__old.date !== $scope2.c.date) {
            {
              const __av343 = $scope2.c.date;
              if (__av343 == null || __av343 === false || __av343 === "")
                __e339.removeAttribute("timestamp");
              else
                __e339.setAttribute("timestamp", __av343 === true ? "" : String(__av343));
            }
          }
          if (__old.additions !== $scope2.c.additions) {
            {
              const __av344 = $scope2.c.additions + "";
              if (__av344 == null || __av344 === false || __av344 === "")
                __e339.removeAttribute("additions");
              else
                __e339.setAttribute("additions", __av344 === true ? "" : String(__av344));
            }
          }
          if (__old.deletions !== $scope2.c.deletions) {
            {
              const __av345 = $scope2.c.deletions + "";
              if (__av345 == null || __av345 === false || __av345 === "")
                __e339.removeAttribute("deletions");
              else
                __e339.setAttribute("deletions", __av345 === true ? "" : String(__av345));
            }
          }
        } };
      })(__childScope336);
    }
  });
  const __wireBoundary_b109 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.commitFilter.filtered().length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag320 = document.createDocumentFragment();
        const __c321 = [];
        const __e322 = document.createElement("sui-empty-state");
        __e322.setAttribute("tone", "muted");
        __e322.setAttribute("size", "md");
        const __ifAnchor323 = document.createComment("if");
        __e322.appendChild(__ifAnchor323);
        __c321.push(__wireBoundary_b110(__ifAnchor323));
        __frag320.appendChild(__e322);
        return { nodes: Array.from(__frag320.childNodes), cleanup: () => {
          for (const c of __c321)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag332 = document.createDocumentFragment();
        const __c333 = [];
        const __e334 = document.createElement("div");
        __e334.setAttribute("class", "git-history-rows");
        __e334.setAttribute("data-test", "history-rows");
        const __forAnchor335 = document.createComment("for");
        __e334.appendChild(__forAnchor335);
        __c333.push(__wireBoundary_b111(__forAnchor335));
        __frag332.appendChild(__e334);
        return { nodes: Array.from(__frag332.childNodes), cleanup: () => {
          for (const c of __c333)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b105 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tab() === "history")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag310 = document.createDocumentFragment();
        const __c311 = [];
        const __e312 = document.createElement("div");
        __e312.setAttribute("class", "git-history");
        const __e313 = document.createElement("sui-cluster");
        __e313.setAttribute("align", "center");
        const __e314 = document.createElement("input");
        __e314.setAttribute("class", "git-search");
        __e314.setAttribute("placeholder", "Filter by subject, sha, or author");
        __c311.push(bindValue(__e314, $scope.commitFilter.query));
        __e313.appendChild(__e314);
        const __e315 = document.createElement("span");
        __e315.setAttribute("class", "git-history-count");
        const __t316 = document.createTextNode("");
        __e315.appendChild(__t316);
        __c311.push(bindText(__t316, () => $scope.commitFilter.filtered().length));
        const __t317 = document.createTextNode(" of ");
        __e315.appendChild(__t317);
        const __t318 = document.createTextNode("");
        __e315.appendChild(__t318);
        __c311.push(bindText(__t318, () => $scope.commitsCount()));
        __e313.appendChild(__e315);
        __e312.appendChild(__e313);
        const __ifAnchor319 = document.createComment("if");
        __e312.appendChild(__ifAnchor319);
        __c311.push(__wireBoundary_b109(__ifAnchor319));
        __frag310.appendChild(__e312);
        return { nodes: Array.from(__frag310.childNodes), cleanup: () => {
          for (const c of __c311)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b123 = (anchor) => bindFor({
    anchor,
    items: () => $scope.branches(),
    key: (b, __i) => {
      const __child = Object.create($scope);
      __child.b = b;
      __child.__i = __i;
      return (($scope2) => b.name)(__child);
    },
    render: (b, __i) => {
      const __childScope369 = Object.create($scope);
      __childScope369.b = b;
      __childScope369.__i = __i;
      return (($scope2) => {
        __childScope369.__v_name = signal2(0);
        const __frag370 = document.createDocumentFragment();
        const __c371 = [];
        const __e372 = document.createElement("sui-list-tile");
        __e372.setAttribute("interactive", "");
        __e372.setAttribute("density", "compact");
        __e372.setAttribute("data-test", "git-branch-tile");
        {
          const __av373 = $scope2.b.active ? "primary" : null;
          if (__av373 == null || __av373 === false || __av373 === "")
            __e372.removeAttribute("tone");
          else
            __e372.setAttribute("tone", __av373 === true ? "" : String(__av373));
        }
        __c371.push(bindAttr(__e372, "selected", () => (void $scope2.__v_name(), $scope2.activeBranchName() === $scope2.b.name ? "" : null)));
        __c371.push(on(__e372, "sui-select", function() {
          const __h = () => $scope2.selectBranch($scope2.b.name);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e374 = document.createElement("sui-status-dot");
        __e374.setAttribute("slot", "leading");
        {
          const __av375 = $scope2.b.active ? "success" : "muted";
          if (__av375 == null || __av375 === false || __av375 === "")
            __e374.removeAttribute("tone");
          else
            __e374.setAttribute("tone", __av375 === true ? "" : String(__av375));
        }
        {
          const __av376 = $scope2.b.active ? "" : null;
          if (__av376 == null || __av376 === false || __av376 === "")
            __e374.removeAttribute("pulse");
          else
            __e374.setAttribute("pulse", __av376 === true ? "" : String(__av376));
        }
        __e372.appendChild(__e374);
        const __e377 = document.createElement("span");
        __e377.setAttribute("class", "git-mono git-branch-name");
        const __t378 = document.createTextNode("");
        __e377.appendChild(__t378);
        __t378.textContent = $scope2.b.name;
        __e372.appendChild(__e377);
        const __e379 = document.createElement("span");
        __e379.setAttribute("slot", "meta");
        const __t380 = document.createTextNode("");
        __e379.appendChild(__t380);
        __t380.textContent = $scope2.b.upstream ? $scope2.b.upstream : "local only";
        __e372.appendChild(__e379);
        const __e381 = document.createElement("span");
        __e381.setAttribute("slot", "trailing");
        __e381.setAttribute("class", "git-mono git-branch-deltas");
        const __t382 = document.createTextNode("");
        __e381.appendChild(__t382);
        __t382.textContent = $scope2.b.ahead > 0 ? "↑" + $scope2.b.ahead : "";
        const __t383 = document.createTextNode("");
        __e381.appendChild(__t383);
        __t383.textContent = $scope2.b.behind > 0 ? " ↓" + $scope2.b.behind : "";
        __e372.appendChild(__e381);
        __frag370.appendChild(__e372);
        return { nodes: Array.from(__frag370.childNodes), cleanup: () => {
          for (const c of __c371)
            try {
              c();
            } catch (e) {}
        }, update: (__new384, __newI385) => {
          const __old = $scope2.b;
          $scope2.b = __new384;
          $scope2.__i = __newI385;
          if (__old.active !== $scope2.b.active) {
            {
              const __av373 = $scope2.b.active ? "primary" : null;
              if (__av373 == null || __av373 === false || __av373 === "")
                __e372.removeAttribute("tone");
              else
                __e372.setAttribute("tone", __av373 === true ? "" : String(__av373));
            }
            {
              const __av375 = $scope2.b.active ? "success" : "muted";
              if (__av375 == null || __av375 === false || __av375 === "")
                __e374.removeAttribute("tone");
              else
                __e374.setAttribute("tone", __av375 === true ? "" : String(__av375));
            }
            {
              const __av376 = $scope2.b.active ? "" : null;
              if (__av376 == null || __av376 === false || __av376 === "")
                __e374.removeAttribute("pulse");
              else
                __e374.setAttribute("pulse", __av376 === true ? "" : String(__av376));
            }
          }
          if (__old.name !== $scope2.b.name) {
            __t378.textContent = $scope2.b.name;
          }
          if (__old.upstream !== $scope2.b.upstream) {
            __t380.textContent = $scope2.b.upstream ? $scope2.b.upstream : "local only";
          }
          if (__old.ahead !== $scope2.b.ahead) {
            __t382.textContent = $scope2.b.ahead > 0 ? "↑" + $scope2.b.ahead : "";
          }
          if (__old.behind !== $scope2.b.behind) {
            __t383.textContent = $scope2.b.behind > 0 ? " ↓" + $scope2.b.behind : "";
          }
          if (__old.name !== $scope2.b.name) {
            $scope2.__v_name.set((c) => c + 1);
          }
        } };
      })(__childScope369);
    }
  });
  const __wireBoundary_b136 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (!$scope.activeBranch().active)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag399 = document.createDocumentFragment();
        const __c400 = [];
        const __e401 = document.createElement("sui-button");
        __e401.setAttribute("size", "sm");
        __e401.setAttribute("variant", "solid");
        __e401.setAttribute("tone", "primary");
        __e401.setAttribute("data-test", "git-checkout");
        __c400.push(bindAttr(__e401, "loading", () => $scope.busy() === "checkout" ? "" : null));
        __c400.push(bindAttr(__e401, "disabled", () => $scope.busy() ? "" : null));
        __c400.push(on(__e401, "click", function() {
          const __h = () => $scope.checkoutBranch($scope.activeBranch().name);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e402 = document.createElement("sui-icon");
        __e402.setAttribute("slot", "leading");
        __e402.setAttribute("name", "git-branch");
        __e402.setAttribute("size", "xs");
        __e401.appendChild(__e402);
        const __t403 = document.createTextNode(`Checkout
                        `);
        __e401.appendChild(__t403);
        __frag399.appendChild(__e401);
        const __e404 = document.createElement("sui-button");
        __e404.setAttribute("size", "sm");
        __e404.setAttribute("variant", "ghost");
        __e404.setAttribute("tone", "destructive");
        __e404.setAttribute("data-test", "git-delete-branch");
        __c400.push(bindAttr(__e404, "disabled", () => $scope.busy() ? "" : null));
        __c400.push(on(__e404, "click", function() {
          const __h = () => $scope.openBranchDelete($scope.activeBranch().name);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e405 = document.createElement("sui-icon");
        __e405.setAttribute("slot", "leading");
        __e405.setAttribute("name", "trash");
        __e405.setAttribute("size", "xs");
        __e404.appendChild(__e405);
        const __t406 = document.createTextNode(`Delete
                        `);
        __e404.appendChild(__t406);
        __frag399.appendChild(__e404);
        return { nodes: Array.from(__frag399.childNodes), cleanup: () => {
          for (const c of __c400)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b143 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.activeBranch().upstream)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag413 = document.createDocumentFragment();
        const __c414 = [];
        const __e415 = document.createElement("sui-metachip");
        __e415.setAttribute("label", "Upstream");
        __c414.push(bindAttr(__e415, "value", () => $scope.activeBranch().upstream));
        __frag413.appendChild(__e415);
        return { nodes: Array.from(__frag413.childNodes), cleanup: () => {
          for (const c of __c414)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b133 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.activeBranch())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag388 = document.createDocumentFragment();
        const __c389 = [];
        const __e390 = document.createElement("div");
        __e390.setAttribute("class", "git-branch-detail");
        __e390.setAttribute("data-test", "git-branch-detail");
        const __e391 = document.createElement("sui-toolbar");
        __e391.setAttribute("justify", "between");
        __e391.setAttribute("class", "git-branch-head");
        const __e392 = document.createElement("div");
        const __e393 = document.createElement("div");
        __e393.setAttribute("class", "git-mono git-branch-detail-name");
        const __t394 = document.createTextNode("");
        __e393.appendChild(__t394);
        __c389.push(bindText(__t394, () => $scope.activeBranch().name));
        __e392.appendChild(__e393);
        const __e395 = document.createElement("div");
        __e395.setAttribute("class", "git-branch-detail-sub");
        const __t396 = document.createTextNode("");
        __e395.appendChild(__t396);
        __c389.push(bindText(__t396, () => $scope.activeBranch().upstream ? "tracking " + $scope.activeBranch().upstream : "no upstream"));
        __e392.appendChild(__e395);
        __e391.appendChild(__e392);
        const __e397 = document.createElement("sui-toolbar");
        const __ifAnchor398 = document.createComment("if");
        __e397.appendChild(__ifAnchor398);
        __c389.push(__wireBoundary_b136(__ifAnchor398));
        __e391.appendChild(__e397);
        __e390.appendChild(__e391);
        const __e407 = document.createElement("div");
        __e407.setAttribute("class", "git-branch-grid");
        const __e408 = document.createElement("sui-card");
        const __e409 = document.createElement("div");
        __e409.setAttribute("slot", "header");
        __e409.setAttribute("class", "git-card-head");
        const __t410 = document.createTextNode("Status");
        __e409.appendChild(__t410);
        __e408.appendChild(__e409);
        const __e411 = document.createElement("sui-metachip");
        __e411.setAttribute("label", "Head");
        __c389.push(bindAttr(__e411, "value", () => $scope.activeBranch().name));
        __e408.appendChild(__e411);
        const __ifAnchor412 = document.createComment("if");
        __e408.appendChild(__ifAnchor412);
        __c389.push(__wireBoundary_b143(__ifAnchor412));
        const __e416 = document.createElement("sui-metachip");
        __e416.setAttribute("label", "Divergence");
        __c389.push(bindAttr(__e416, "value", () => "↑" + $scope.activeBranch().ahead + " ↓" + $scope.activeBranch().behind));
        __e408.appendChild(__e416);
        __e407.appendChild(__e408);
        const __e417 = document.createElement("sui-card");
        const __e418 = document.createElement("div");
        __e418.setAttribute("slot", "header");
        __e418.setAttribute("class", "git-card-head");
        const __t419 = document.createTextNode("Actions");
        __e418.appendChild(__t419);
        __e417.appendChild(__e418);
        const __e420 = document.createElement("sui-button");
        __e420.setAttribute("variant", "ghost");
        __e420.setAttribute("size", "sm");
        __c389.push(on(__e420, "click", function() {
          const __h = $scope.pullCurrent;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c389.push(bindAttr(__e420, "loading", () => $scope.busy() === "pull" ? "" : null));
        __c389.push(bindAttr(__e420, "disabled", () => $scope.busy() ? "" : null));
        const __e421 = document.createElement("sui-icon");
        __e421.setAttribute("slot", "leading");
        __e421.setAttribute("name", "arrow-down");
        __e421.setAttribute("size", "xs");
        __e420.appendChild(__e421);
        const __t422 = document.createTextNode(`Pull latest
                      `);
        __e420.appendChild(__t422);
        __e417.appendChild(__e420);
        const __e423 = document.createElement("sui-button");
        __e423.setAttribute("variant", "ghost");
        __e423.setAttribute("size", "sm");
        __c389.push(on(__e423, "click", function() {
          const __h = $scope.pushCurrent;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c389.push(bindAttr(__e423, "loading", () => $scope.busy() === "push" ? "" : null));
        __c389.push(bindAttr(__e423, "disabled", () => $scope.busy() ? "" : null));
        const __e424 = document.createElement("sui-icon");
        __e424.setAttribute("slot", "leading");
        __e424.setAttribute("name", "arrow-up");
        __e424.setAttribute("size", "xs");
        __e423.appendChild(__e424);
        const __t425 = document.createTextNode(`Push changes
                      `);
        __e423.appendChild(__t425);
        __e417.appendChild(__e423);
        const __e426 = document.createElement("sui-button");
        __e426.setAttribute("variant", "ghost");
        __e426.setAttribute("size", "sm");
        __c389.push(on(__e426, "click", function() {
          const __h = $scope.fetchOrigin;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c389.push(bindAttr(__e426, "loading", () => $scope.busy() === "fetch" ? "" : null));
        __c389.push(bindAttr(__e426, "disabled", () => $scope.busy() ? "" : null));
        const __e427 = document.createElement("sui-icon");
        __e427.setAttribute("slot", "leading");
        __e427.setAttribute("name", "download");
        __e427.setAttribute("size", "xs");
        __e426.appendChild(__e427);
        const __t428 = document.createTextNode(`Fetch
                      `);
        __e426.appendChild(__t428);
        __e417.appendChild(__e426);
        __e407.appendChild(__e417);
        __e390.appendChild(__e407);
        __frag388.appendChild(__e390);
        return { nodes: Array.from(__frag388.childNodes), cleanup: () => {
          for (const c of __c389)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b122 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.branches().length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag358 = document.createDocumentFragment();
        const __c359 = [];
        const __e360 = document.createElement("sui-empty-state");
        __e360.setAttribute("tone", "muted");
        __e360.setAttribute("size", "md");
        const __e361 = document.createElement("span");
        __e361.setAttribute("slot", "title");
        const __t362 = document.createTextNode("No branches");
        __e361.appendChild(__t362);
        __e360.appendChild(__e361);
        __frag358.appendChild(__e360);
        return { nodes: Array.from(__frag358.childNodes), cleanup: () => {
          for (const c of __c359)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag363 = document.createDocumentFragment();
        const __c364 = [];
        const __e365 = document.createElement("div");
        __e365.setAttribute("class", "git-branches-split");
        const __e366 = document.createElement("aside");
        __e366.setAttribute("class", "git-bleft");
        __e366.setAttribute("aria-label", "Branches");
        const __e367 = document.createElement("sui-scroll-area");
        __e367.setAttribute("class", "git-branches-rail");
        const __forAnchor368 = document.createComment("for");
        __e367.appendChild(__forAnchor368);
        __c364.push(__wireBoundary_b123(__forAnchor368));
        __e366.appendChild(__e367);
        __e365.appendChild(__e366);
        const __e386 = document.createElement("section");
        __e386.setAttribute("class", "git-bright");
        __e386.setAttribute("aria-label", "Branch detail");
        const __ifAnchor387 = document.createComment("if");
        __e386.appendChild(__ifAnchor387);
        __c364.push(__wireBoundary_b133(__ifAnchor387));
        __e365.appendChild(__e386);
        __frag363.appendChild(__e365);
        return { nodes: Array.from(__frag363.childNodes), cleanup: () => {
          for (const c of __c364)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b119 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tab() === "branches")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag350 = document.createDocumentFragment();
        const __c351 = [];
        const __e352 = document.createElement("div");
        __e352.setAttribute("class", "git-branches");
        const __e353 = document.createElement("div");
        __e353.setAttribute("class", "git-branches-actions");
        const __e354 = document.createElement("sui-button");
        __e354.setAttribute("size", "sm");
        __e354.setAttribute("variant", "solid");
        __e354.setAttribute("tone", "primary");
        __e354.setAttribute("data-test", "git-open-branch-create");
        __c351.push(bindAttr(__e354, "disabled", () => $scope.busy() ? "" : null));
        __c351.push(on(__e354, "click", function() {
          const __h = $scope.openBranchCreate;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e355 = document.createElement("sui-icon");
        __e355.setAttribute("slot", "leading");
        __e355.setAttribute("name", "plus");
        __e355.setAttribute("size", "xs");
        __e354.appendChild(__e355);
        const __t356 = document.createTextNode(`New branch
          `);
        __e354.appendChild(__t356);
        __e353.appendChild(__e354);
        __e352.appendChild(__e353);
        const __ifAnchor357 = document.createComment("if");
        __e352.appendChild(__ifAnchor357);
        __c351.push(__wireBoundary_b122(__ifAnchor357));
        __frag350.appendChild(__e352);
        return { nodes: Array.from(__frag350.childNodes), cleanup: () => {
          for (const c of __c351)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b160 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.commitError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag439 = document.createDocumentFragment();
        const __c440 = [];
        const __e441 = document.createElement("sui-alert");
        __e441.setAttribute("tone", "destructive");
        __e441.setAttribute("compact", "");
        const __t442 = document.createTextNode("");
        __e441.appendChild(__t442);
        __c440.push(bindText(__t442, () => $scope.commitError()));
        __frag439.appendChild(__e441);
        return { nodes: Array.from(__frag439.childNodes), cleanup: () => {
          for (const c of __c440)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b170 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.branchCreateError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag458 = document.createDocumentFragment();
        const __c459 = [];
        const __e460 = document.createElement("sui-alert");
        __e460.setAttribute("tone", "destructive");
        __e460.setAttribute("compact", "");
        const __t461 = document.createTextNode("");
        __e460.appendChild(__t461);
        __c459.push(bindText(__t461, () => $scope.branchCreateError()));
        __frag458.appendChild(__e460);
        return { nodes: Array.from(__frag458.childNodes), cleanup: () => {
          for (const c of __c459)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b180 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.branchDeleteError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag479 = document.createDocumentFragment();
        const __c480 = [];
        const __e481 = document.createElement("sui-alert");
        __e481.setAttribute("tone", "destructive");
        __e481.setAttribute("compact", "");
        const __t482 = document.createTextNode("");
        __e481.appendChild(__t482);
        __c480.push(bindText(__t482, () => $scope.branchDeleteError()));
        __frag479.appendChild(__e481);
        return { nodes: Array.from(__frag479.childNodes), cleanup: () => {
          for (const c of __c480)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b9 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.overview().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag38 = document.createDocumentFragment();
        const __c39 = [];
        const __ifAnchor40 = document.createComment("if");
        __frag38.appendChild(__ifAnchor40);
        __c39.push(__wireBoundary_b10(__ifAnchor40));
        const __ifAnchor46 = document.createComment("if");
        __frag38.appendChild(__ifAnchor46);
        __c39.push(__wireBoundary_b13(__ifAnchor46));
        const __e52 = document.createElement("header");
        __e52.setAttribute("class", "git-head");
        const __e53 = document.createElement("div");
        __e53.setAttribute("class", "git-head-titles");
        const __e54 = document.createElement("h1");
        __e54.setAttribute("class", "git-title");
        const __t55 = document.createTextNode("Git");
        __e54.appendChild(__t55);
        __e53.appendChild(__e54);
        const __e56 = document.createElement("p");
        __e56.setAttribute("class", "git-sub");
        const __e57 = document.createElement("span");
        __e57.setAttribute("class", "git-mono");
        const __t58 = document.createTextNode("");
        __e57.appendChild(__t58);
        __c39.push(bindText(__t58, () => $scope.branch()));
        __e56.appendChild(__e57);
        const __t59 = document.createTextNode("");
        __e56.appendChild(__t59);
        __c39.push(bindText(__t59, () => $scope.ahead() > 0 ? " ↑" + $scope.ahead() : ""));
        const __t60 = document.createTextNode("");
        __e56.appendChild(__t60);
        __c39.push(bindText(__t60, () => $scope.behind() > 0 ? " ↓" + $scope.behind() : ""));
        const __t61 = document.createTextNode(`
          ·
          `);
        __e56.appendChild(__t61);
        const __e62 = document.createElement("span");
        __e62.setAttribute("class", "git-mono");
        const __t63 = document.createTextNode("");
        __e62.appendChild(__t63);
        __c39.push(bindText(__t63, () => $scope.headShort()));
        __e56.appendChild(__e62);
        __e53.appendChild(__e56);
        __e52.appendChild(__e53);
        const __e64 = document.createElement("sui-cluster");
        __e64.setAttribute("align", "center");
        const __e65 = document.createElement("sui-button");
        __e65.setAttribute("variant", "ghost");
        __e65.setAttribute("size", "sm");
        __e65.setAttribute("data-test", "git-fetch");
        __c39.push(bindAttr(__e65, "loading", () => $scope.busy() === "fetch" ? "" : null));
        __c39.push(bindAttr(__e65, "disabled", () => $scope.busy() ? "" : null));
        __c39.push(on(__e65, "click", function() {
          const __h = $scope.fetchOrigin;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e66 = document.createElement("sui-icon");
        __e66.setAttribute("slot", "leading");
        __e66.setAttribute("name", "download");
        __e66.setAttribute("size", "xs");
        __e65.appendChild(__e66);
        const __t67 = document.createTextNode(`Fetch
        `);
        __e65.appendChild(__t67);
        __e64.appendChild(__e65);
        const __e68 = document.createElement("sui-button");
        __e68.setAttribute("variant", "ghost");
        __e68.setAttribute("size", "sm");
        __e68.setAttribute("data-test", "git-pull");
        __c39.push(bindAttr(__e68, "loading", () => $scope.busy() === "pull" ? "" : null));
        __c39.push(bindAttr(__e68, "disabled", () => $scope.busy() ? "" : null));
        __c39.push(on(__e68, "click", function() {
          const __h = $scope.pullCurrent;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e69 = document.createElement("sui-icon");
        __e69.setAttribute("slot", "leading");
        __e69.setAttribute("name", "arrow-down");
        __e69.setAttribute("size", "xs");
        __e68.appendChild(__e69);
        const __t70 = document.createTextNode(`Pull
        `);
        __e68.appendChild(__t70);
        __e64.appendChild(__e68);
        const __e71 = document.createElement("sui-button");
        __e71.setAttribute("variant", "ghost");
        __e71.setAttribute("size", "sm");
        __e71.setAttribute("data-test", "git-push");
        __c39.push(bindAttr(__e71, "loading", () => $scope.busy() === "push" ? "" : null));
        __c39.push(bindAttr(__e71, "disabled", () => $scope.busy() ? "" : null));
        __c39.push(on(__e71, "click", function() {
          const __h = $scope.pushCurrent;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e72 = document.createElement("sui-icon");
        __e72.setAttribute("slot", "leading");
        __e72.setAttribute("name", "arrow-up");
        __e72.setAttribute("size", "xs");
        __e71.appendChild(__e72);
        const __t73 = document.createTextNode(`Push
        `);
        __e71.appendChild(__t73);
        __e64.appendChild(__e71);
        const __e74 = document.createElement("sui-button");
        __e74.setAttribute("variant", "ghost");
        __e74.setAttribute("size", "sm");
        __e74.setAttribute("aria-label", "Refresh");
        __e74.setAttribute("data-test", "git-refresh");
        __c39.push(on(__e74, "click", function() {
          const __h = $scope.refresh;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e75 = document.createElement("sui-icon");
        __e75.setAttribute("name", "refresh-cw");
        __e75.setAttribute("size", "sm");
        __e74.appendChild(__e75);
        __e64.appendChild(__e74);
        const __e76 = document.createElement("sui-chip");
        __c39.push(bindAttr(__e76, "tone", () => $scope.clean() ? "success" : "warning"));
        __e76.setAttribute("variant", "soft");
        __e76.setAttribute("size", "sm");
        const __t77 = document.createTextNode("");
        __e76.appendChild(__t77);
        __c39.push(bindText(__t77, () => $scope.clean() ? "clean" : $scope.totalChanged() + " changed"));
        __e64.appendChild(__e76);
        __e52.appendChild(__e64);
        __frag38.appendChild(__e52);
        const __e78 = document.createElement("div");
        __e78.setAttribute("class", "git-tabs-row");
        const __e79 = document.createElement("sui-tabs");
        __e79.setAttribute("variant", "underline");
        const __e80 = document.createElement("sui-tab");
        __e80.setAttribute("name", "changes");
        __c39.push(bindAttr(__e80, "active", () => $scope.tab() === "changes"));
        __c39.push(bindAttr(__e80, "count", () => $scope.totalChanged() + ""));
        __c39.push(on(__e80, "click", function() {
          const __h = () => $scope.tab.set("changes");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t81 = document.createTextNode("Changes");
        __e80.appendChild(__t81);
        __e79.appendChild(__e80);
        const __e82 = document.createElement("sui-tab");
        __e82.setAttribute("name", "history");
        __c39.push(bindAttr(__e82, "active", () => $scope.tab() === "history"));
        __c39.push(bindAttr(__e82, "count", () => $scope.commitsCount() + ""));
        __c39.push(on(__e82, "click", function() {
          const __h = () => $scope.tab.set("history");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t83 = document.createTextNode("History");
        __e82.appendChild(__t83);
        __e79.appendChild(__e82);
        const __e84 = document.createElement("sui-tab");
        __e84.setAttribute("name", "branches");
        __c39.push(bindAttr(__e84, "active", () => $scope.tab() === "branches"));
        __c39.push(bindAttr(__e84, "count", () => $scope.branchesCount() + ""));
        __c39.push(on(__e84, "click", function() {
          const __h = () => $scope.tab.set("branches");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t85 = document.createTextNode("Branches");
        __e84.appendChild(__t85);
        __e79.appendChild(__e84);
        __e78.appendChild(__e79);
        const __e86 = document.createElement("div");
        __e86.setAttribute("class", "git-tabs-right");
        const __e87 = document.createElement("sui-button");
        __e87.setAttribute("variant", "ghost");
        __e87.setAttribute("size", "sm");
        __c39.push(on(__e87, "click", function() {
          const __h = () => $scope.navigate("/pipelines");
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e88 = document.createElement("sui-icon");
        __e88.setAttribute("slot", "leading");
        __e88.setAttribute("name", "play");
        __e88.setAttribute("size", "xs");
        __e87.appendChild(__e88);
        const __t89 = document.createTextNode(`Run checks
        `);
        __e87.appendChild(__t89);
        __e86.appendChild(__e87);
        __e78.appendChild(__e86);
        __frag38.appendChild(__e78);
        const __e90 = document.createElement("div");
        __e90.setAttribute("class", "git-body");
        const __ifAnchor91 = document.createComment("if");
        __e90.appendChild(__ifAnchor91);
        __c39.push(__wireBoundary_b42(__ifAnchor91));
        const __ifAnchor309 = document.createComment("if");
        __e90.appendChild(__ifAnchor309);
        __c39.push(__wireBoundary_b105(__ifAnchor309));
        const __ifAnchor349 = document.createComment("if");
        __e90.appendChild(__ifAnchor349);
        __c39.push(__wireBoundary_b119(__ifAnchor349));
        __frag38.appendChild(__e90);
        const __e429 = document.createElement("sui-dialog");
        __c39.push(bindProp(__e429, "open", () => $scope.commitOpen()));
        __e429.setAttribute("size", "md");
        __c39.push(on(__e429, "sui-close", function() {
          const __h = $scope.closeCommit;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e429.setAttribute("data-test", "git-commit-dialog");
        const __e430 = document.createElement("span");
        __e430.setAttribute("slot", "header");
        const __t431 = document.createTextNode("Commit (");
        __e430.appendChild(__t431);
        const __t432 = document.createTextNode("");
        __e430.appendChild(__t432);
        __c39.push(bindText(__t432, () => $scope.stagedCount()));
        const __t433 = document.createTextNode(" staged)");
        __e430.appendChild(__t433);
        __e429.appendChild(__e430);
        const __e434 = document.createElement("sui-field");
        __e434.setAttribute("label", "Subject");
        __e434.setAttribute("hint", "One line, imperative mood.");
        const __e435 = document.createElement("sui-input");
        __c39.push(bindValue(__e435, $scope.commitSubject));
        __e435.setAttribute("placeholder", "fix: …");
        __e435.setAttribute("autofocus", "");
        __e434.appendChild(__e435);
        __e429.appendChild(__e434);
        const __e436 = document.createElement("sui-field");
        __e436.setAttribute("label", "Body");
        __e436.setAttribute("hint", "Optional. Explain the why, not the what.");
        const __e437 = document.createElement("sui-textarea");
        __c39.push(bindValue(__e437, $scope.commitBody));
        __e437.setAttribute("rows", "6");
        __e437.setAttribute("placeholder", "Optional description…");
        __e436.appendChild(__e437);
        __e429.appendChild(__e436);
        const __ifAnchor438 = document.createComment("if");
        __e429.appendChild(__ifAnchor438);
        __c39.push(__wireBoundary_b160(__ifAnchor438));
        const __e443 = document.createElement("sui-toolbar");
        __e443.setAttribute("slot", "footer");
        const __e444 = document.createElement("sui-button");
        __e444.setAttribute("variant", "outline");
        __c39.push(on(__e444, "click", function() {
          const __h = $scope.closeCommit;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t445 = document.createTextNode("Cancel");
        __e444.appendChild(__t445);
        __e443.appendChild(__e444);
        const __e446 = document.createElement("sui-button");
        __e446.setAttribute("tone", "primary");
        __e446.setAttribute("variant", "solid");
        __e446.setAttribute("data-test", "git-commit-submit");
        __c39.push(bindAttr(__e446, "loading", () => $scope.busy() === "commit" ? "" : null));
        __c39.push(bindAttr(__e446, "disabled", () => $scope.busy() === "commit" ? "" : null));
        __c39.push(on(__e446, "click", function() {
          const __h = $scope.submitCommit;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e447 = document.createElement("sui-icon");
        __e447.setAttribute("slot", "leading");
        __e447.setAttribute("name", "git-commit");
        __e447.setAttribute("size", "sm");
        __e446.appendChild(__e447);
        const __t448 = document.createTextNode(`Commit
        `);
        __e446.appendChild(__t448);
        __e443.appendChild(__e446);
        __e429.appendChild(__e443);
        __frag38.appendChild(__e429);
        const __e449 = document.createElement("sui-dialog");
        __c39.push(bindProp(__e449, "open", () => $scope.branchCreateOpen()));
        __e449.setAttribute("size", "sm");
        __c39.push(on(__e449, "sui-close", function() {
          const __h = $scope.closeBranchCreate;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e449.setAttribute("data-test", "git-branch-create-dialog");
        const __e450 = document.createElement("span");
        __e450.setAttribute("slot", "header");
        const __t451 = document.createTextNode("New branch");
        __e450.appendChild(__t451);
        __e449.appendChild(__e450);
        const __e452 = document.createElement("sui-field");
        __e452.setAttribute("label", "Name");
        __e452.setAttribute("hint", "Created from current HEAD.");
        const __e453 = document.createElement("sui-input");
        __c39.push(bindValue(__e453, $scope.branchCreateName));
        __e453.setAttribute("placeholder", "feature/my-branch");
        __e453.setAttribute("autofocus", "");
        __e452.appendChild(__e453);
        __e449.appendChild(__e452);
        const __e454 = document.createElement("sui-field");
        const __e455 = document.createElement("sui-checkbox");
        __c39.push(bindChecked(__e455, $scope.branchCreateCheckout));
        const __t456 = document.createTextNode("Switch to this branch after creating");
        __e455.appendChild(__t456);
        __e454.appendChild(__e455);
        __e449.appendChild(__e454);
        const __ifAnchor457 = document.createComment("if");
        __e449.appendChild(__ifAnchor457);
        __c39.push(__wireBoundary_b170(__ifAnchor457));
        const __e462 = document.createElement("sui-toolbar");
        __e462.setAttribute("slot", "footer");
        const __e463 = document.createElement("sui-button");
        __e463.setAttribute("variant", "outline");
        __c39.push(on(__e463, "click", function() {
          const __h = $scope.closeBranchCreate;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t464 = document.createTextNode("Cancel");
        __e463.appendChild(__t464);
        __e462.appendChild(__e463);
        const __e465 = document.createElement("sui-button");
        __e465.setAttribute("tone", "primary");
        __e465.setAttribute("variant", "solid");
        __e465.setAttribute("data-test", "git-branch-create-submit");
        __c39.push(bindAttr(__e465, "loading", () => $scope.busy() === "branch create" ? "" : null));
        __c39.push(bindAttr(__e465, "disabled", () => $scope.busy() === "branch create" ? "" : null));
        __c39.push(on(__e465, "click", function() {
          const __h = $scope.submitBranchCreate;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t466 = document.createTextNode("Create");
        __e465.appendChild(__t466);
        __e462.appendChild(__e465);
        __e449.appendChild(__e462);
        __frag38.appendChild(__e449);
        const __e467 = document.createElement("sui-dialog");
        __c39.push(bindProp(__e467, "open", () => $scope.branchDeleteOpen()));
        __e467.setAttribute("size", "sm");
        __c39.push(on(__e467, "sui-close", function() {
          const __h = $scope.closeBranchDelete;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e467.setAttribute("data-test", "git-branch-delete-dialog");
        const __e468 = document.createElement("span");
        __e468.setAttribute("slot", "header");
        const __t469 = document.createTextNode("Delete branch");
        __e468.appendChild(__t469);
        __e467.appendChild(__e468);
        const __e470 = document.createElement("p");
        __e470.setAttribute("class", "git-confirm-msg");
        const __t471 = document.createTextNode("Delete ");
        __e470.appendChild(__t471);
        const __e472 = document.createElement("span");
        __e472.setAttribute("class", "git-mono");
        const __t473 = document.createTextNode("");
        __e472.appendChild(__t473);
        __c39.push(bindText(__t473, () => $scope.branchDeleteTarget()));
        __e470.appendChild(__e472);
        const __t474 = document.createTextNode("? This cannot be undone.");
        __e470.appendChild(__t474);
        __e467.appendChild(__e470);
        const __e475 = document.createElement("sui-field");
        const __e476 = document.createElement("sui-checkbox");
        __c39.push(bindChecked(__e476, $scope.branchDeleteForce));
        const __t477 = document.createTextNode("Force-delete even if unmerged");
        __e476.appendChild(__t477);
        __e475.appendChild(__e476);
        __e467.appendChild(__e475);
        const __ifAnchor478 = document.createComment("if");
        __e467.appendChild(__ifAnchor478);
        __c39.push(__wireBoundary_b180(__ifAnchor478));
        const __e483 = document.createElement("sui-toolbar");
        __e483.setAttribute("slot", "footer");
        const __e484 = document.createElement("sui-button");
        __e484.setAttribute("variant", "outline");
        __c39.push(on(__e484, "click", function() {
          const __h = $scope.closeBranchDelete;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t485 = document.createTextNode("Cancel");
        __e484.appendChild(__t485);
        __e483.appendChild(__e484);
        const __e486 = document.createElement("sui-button");
        __e486.setAttribute("tone", "destructive");
        __e486.setAttribute("variant", "solid");
        __e486.setAttribute("data-test", "git-branch-delete-submit");
        __c39.push(bindAttr(__e486, "loading", () => $scope.busy() === "branch delete" ? "" : null));
        __c39.push(bindAttr(__e486, "disabled", () => $scope.busy() === "branch delete" ? "" : null));
        __c39.push(on(__e486, "click", function() {
          const __h = $scope.submitBranchDelete;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t487 = document.createTextNode("Delete");
        __e486.appendChild(__t487);
        __e483.appendChild(__e486);
        __e467.appendChild(__e483);
        __frag38.appendChild(__e467);
        return { nodes: Array.from(__frag38.childNodes), cleanup: () => {
          for (const c of __c39)
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
      __e1.setAttribute("key", "1");
      __e1.setAttribute("action", "showChanges");
      __e0.appendChild(__e1);
      const __e2 = document.createElement("on");
      __e2.setAttribute("key", "2");
      __e2.setAttribute("action", "showHistory");
      __e0.appendChild(__e2);
      const __e3 = document.createElement("on");
      __e3.setAttribute("key", "3");
      __e3.setAttribute("action", "showBranches");
      __e0.appendChild(__e3);
      container.appendChild(__e0);
      const __hook_kbd = getTemplateHook("kbd");
      if (__hook_kbd) {
        const __hookAttrs = Object.create(null);
        __hookAttrs.shortcuts = [{ key: "1", action: "showChanges" }, { key: "2", action: "showHistory" }, { key: "3", action: "showBranches" }];
        __hookAttrs.resolve = (__n) => $scope[__n];
        __hook_kbd.factory({ attrs: __hookAttrs, el: __e0, cleanup: (__fn) => __cleanups.push(__fn) });
      }
      const __e4 = document.createElement("div");
      __e4.setAttribute("class", "ship-view git-view");
      const __wlAnchor14 = document.createComment("while-loading");
      __e4.appendChild(__wlAnchor14);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor14, signal: $scope.overview, render: __renderWhileLoading_0 }));
      const __weAnchor36 = document.createComment("while-error");
      __e4.appendChild(__weAnchor36);
      __cleanups.push(bindWhileError({ anchor: __weAnchor36, signal: $scope.overview, render: __renderWhileError_0 }));
      const __ifAnchor37 = document.createComment("if");
      __e4.appendChild(__ifAnchor37);
      __cleanups.push(__wireBoundary_b9(__ifAnchor37));
      container.appendChild(__e4);
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
        __cleanups.push(hydrateWhileLoading(__m["b0"], "b0", () => $scope.overview(), __renderWhileLoading_0));
      if (__m["b2"])
        __cleanups.push(hydrateWhileError(__m["b2"], "b2", () => $scope.overview(), __renderWhileError_0));
      if (__m["b9"]) {
        const __t = __m["b9"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b9(__anchor));
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
