// views/git/dist/view.compiled.js
import { bindAttr, bindFor, bindIf, bindText, bindValue, bindWhileError, bindWhileLoading, data, live, on, view } from "@ship/ui-runtime";
import { collectElementMarkers, hydrateWhileError, hydrateWhileLoading } from "@ship/ui-dom";

// views/git/view.controller.ts
import { controller, signal, unwrapOr, useFilteredList } from "@ship/ui-runtime";
var EMPTY_OVERVIEW = {
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
var read = ({ overview }) => unwrapOr(overview, EMPTY_OVERVIEW);
var view_controller_default = controller({
  scope: () => ({ tab: signal("changes") }),
  derived: {
    branch: (s) => read(s).branch,
    headShort: (s) => read(s).head_short,
    clean: (s) => read(s).clean,
    totalChanged: (s) => read(s).total_changed,
    ahead: (s) => read(s).ahead,
    behind: (s) => read(s).behind,
    staged: (s) => read(s).staged,
    modified: (s) => read(s).modified,
    untracked: (s) => read(s).untracked,
    commits: (s) => read(s).commits,
    branches: (s) => read(s).branches,
    commitsCount: (s) => read(s).commits.length,
    branchesCount: (s) => read(s).branches.length,
    recentCommits: (s) => read(s).commits.slice(0, 5),
    recentBranches: (s) => read(s).branches.slice(0, 6)
  },
  use: (s) => ({
    // Filter on subject, short sha, or author — three needles, one query.
    commitFilter: useFilteredList(() => read(s).commits, (c, q) => {
      if (!q) return true;
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
    }
  }
});

// views/git/dist/view.compiled.js
var view_compiled_default = view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = /* @__PURE__ */ Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.overview = live(({ ship: ship2, route: route2 }) => ship2.git.overview());
  if (view_controller_default && typeof view_controller_default.setup === "function") {
    const __r = view_controller_default.setup(ctx);
    if (__r && typeof __r === "object") Object.assign($scope, __r);
  }
  const __renderWhileLoading_0 = () => {
    const __frag5 = document.createDocumentFragment();
    const __c6 = [];
    const __e7 = document.createElement("p");
    __e7.setAttribute("class", "git-loading");
    __e7.setAttribute("data-test", "loading");
    const __t8 = document.createTextNode("Loading repo overview&hellip;");
    __e7.appendChild(__t8);
    __frag5.appendChild(__e7);
    return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
      for (const c of __c6) try {
        c();
      } catch (e) {
      }
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope10 = Object.create($scope);
    __childScope10.err = err;
    return (($scope2) => {
      const __frag11 = document.createDocumentFragment();
      const __c12 = [];
      const __e13 = document.createElement("p");
      __e13.setAttribute("class", "git-error");
      __e13.setAttribute("data-test", "error");
      const __t14 = document.createTextNode("Failed to load: ");
      __e13.appendChild(__t14);
      const __t15 = document.createTextNode("");
      __e13.appendChild(__t15);
      __c12.push(bindText(__t15, () => $scope2.err.message));
      __frag11.appendChild(__e13);
      return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
        for (const c of __c12) try {
          c();
        } catch (e) {
        }
      } };
    })(__childScope10);
  };
  const __wireBoundary_b26 = (anchor) => bindFor({
    anchor,
    items: () => $scope.recentCommits(),
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return (($scope2) => c.sha)(__child);
    },
    render: (c, __i) => {
      const __childScope77 = Object.create($scope);
      __childScope77.c = c;
      __childScope77.__i = __i;
      return (($scope2) => {
        const __frag78 = document.createDocumentFragment();
        const __c79 = [];
        const __e80 = document.createElement("div");
        __e80.setAttribute("class", "git-row");
        const __e81 = document.createElement("sui-avatar");
        __e81.setAttribute("size", "sm");
        __e81.setAttribute("shape", "round");
        __c79.push(bindAttr(__e81, "name", () => $scope2.c.author));
        __e80.appendChild(__e81);
        const __e82 = document.createElement("div");
        __e82.setAttribute("class", "git-row-body");
        const __e83 = document.createElement("div");
        __e83.setAttribute("class", "git-row-title");
        const __t84 = document.createTextNode("");
        __e83.appendChild(__t84);
        __c79.push(bindText(__t84, () => $scope2.c.subject));
        __e82.appendChild(__e83);
        const __e85 = document.createElement("div");
        __e85.setAttribute("class", "git-row-sub");
        const __t86 = document.createTextNode("");
        __e85.appendChild(__t86);
        __c79.push(bindText(__t86, () => $scope2.c.short_sha));
        const __t87 = document.createTextNode(" &middot; ");
        __e85.appendChild(__t87);
        const __t88 = document.createTextNode("");
        __e85.appendChild(__t88);
        __c79.push(bindText(__t88, () => $scope2.c.author));
        const __t89 = document.createTextNode(" &middot; ");
        __e85.appendChild(__t89);
        const __t90 = document.createTextNode("");
        __e85.appendChild(__t90);
        __c79.push(bindText(__t90, () => $scope2.c.date));
        __e82.appendChild(__e85);
        __e80.appendChild(__e82);
        const __e91 = document.createElement("div");
        __e91.setAttribute("class", "git-deltas");
        const __e92 = document.createElement("span");
        __e92.setAttribute("class", "git-add");
        const __t93 = document.createTextNode("+");
        __e92.appendChild(__t93);
        const __t94 = document.createTextNode("");
        __e92.appendChild(__t94);
        __c79.push(bindText(__t94, () => $scope2.c.additions));
        __e91.appendChild(__e92);
        const __e95 = document.createElement("span");
        __e95.setAttribute("class", "git-del");
        const __t96 = document.createTextNode("&minus;");
        __e95.appendChild(__t96);
        const __t97 = document.createTextNode("");
        __e95.appendChild(__t97);
        __c79.push(bindText(__t97, () => $scope2.c.deletions));
        __e91.appendChild(__e95);
        __e80.appendChild(__e91);
        __frag78.appendChild(__e80);
        return { nodes: Array.from(__frag78.childNodes), cleanup: () => {
          for (const c2 of __c79) try {
            c2();
          } catch (e) {
          }
        } };
      })(__childScope77);
    }
  });
  const __wireBoundary_b25 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.recentCommits().length === 0) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag70 = document.createDocumentFragment();
        const __c71 = [];
        const __e72 = document.createElement("p");
        __e72.setAttribute("class", "git-empty");
        const __t73 = document.createTextNode("No commits yet.");
        __e72.appendChild(__t73);
        __frag70.appendChild(__e72);
        return { nodes: Array.from(__frag70.childNodes), cleanup: () => {
          for (const c of __c71) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag74 = document.createDocumentFragment();
        const __c75 = [];
        const __forAnchor76 = document.createComment("for");
        __frag74.appendChild(__forAnchor76);
        __c75.push(__wireBoundary_b26(__forAnchor76));
        return { nodes: Array.from(__frag74.childNodes), cleanup: () => {
          for (const c of __c75) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b35 = (anchor) => bindFor({
    anchor,
    items: () => $scope.recentBranches(),
    key: (b, __i) => {
      const __child = Object.create($scope);
      __child.b = b;
      __child.__i = __i;
      return (($scope2) => b.name)(__child);
    },
    render: (b, __i) => {
      const __childScope109 = Object.create($scope);
      __childScope109.b = b;
      __childScope109.__i = __i;
      return (($scope2) => {
        const __frag110 = document.createDocumentFragment();
        const __c111 = [];
        const __e112 = document.createElement("div");
        __e112.setAttribute("class", "git-row");
        const __e113 = document.createElement("ship-icon");
        __e113.setAttribute("name", "git-branch");
        __e113.setAttribute("size", "xs");
        __c111.push(bindAttr(__e113, "class", () => $scope2.b.active ? "git-icon-active" : "git-icon-muted"));
        __e112.appendChild(__e113);
        const __e114 = document.createElement("div");
        __e114.setAttribute("class", "git-row-body");
        const __e115 = document.createElement("div");
        __e115.setAttribute("class", "git-row-title git-mono");
        const __t116 = document.createTextNode("");
        __e115.appendChild(__t116);
        __c111.push(bindText(__t116, () => $scope2.b.name));
        __e114.appendChild(__e115);
        const __e117 = document.createElement("div");
        __e117.setAttribute("class", "git-row-sub");
        const __t118 = document.createTextNode("");
        __e117.appendChild(__t118);
        __c111.push(bindText(__t118, () => $scope2.b.upstream ? $scope2.b.upstream : "local only"));
        __e114.appendChild(__e117);
        __e112.appendChild(__e114);
        const __e119 = document.createElement("div");
        __e119.setAttribute("class", "git-deltas");
        const __t120 = document.createTextNode("");
        __e119.appendChild(__t120);
        __c111.push(bindText(__t120, () => $scope2.b.ahead > 0 ? "\xE2\x86\x91" + $scope2.b.ahead : ""));
        const __t121 = document.createTextNode("");
        __e119.appendChild(__t121);
        __c111.push(bindText(__t121, () => $scope2.b.behind > 0 ? " \xE2\x86\x93" + $scope2.b.behind : ""));
        __e112.appendChild(__e119);
        __frag110.appendChild(__e112);
        return { nodes: Array.from(__frag110.childNodes), cleanup: () => {
          for (const c of __c111) try {
            c();
          } catch (e) {
          }
        } };
      })(__childScope109);
    }
  });
  const __wireBoundary_b34 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.recentBranches().length === 0) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag102 = document.createDocumentFragment();
        const __c103 = [];
        const __e104 = document.createElement("p");
        __e104.setAttribute("class", "git-empty");
        const __t105 = document.createTextNode("No branches.");
        __e104.appendChild(__t105);
        __frag102.appendChild(__e104);
        return { nodes: Array.from(__frag102.childNodes), cleanup: () => {
          for (const c of __c103) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag106 = document.createDocumentFragment();
        const __c107 = [];
        const __forAnchor108 = document.createComment("for");
        __frag106.appendChild(__forAnchor108);
        __c107.push(__wireBoundary_b35(__forAnchor108));
        return { nodes: Array.from(__frag106.childNodes), cleanup: () => {
          for (const c of __c107) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b43 = (anchor) => bindFor({
    anchor,
    items: () => $scope.staged(),
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return (($scope2) => f.path)(__child);
    },
    render: (f, __i) => {
      const __childScope134 = Object.create($scope);
      __childScope134.f = f;
      __childScope134.__i = __i;
      return (($scope2) => {
        const __frag135 = document.createDocumentFragment();
        const __c136 = [];
        const __e137 = document.createElement("div");
        __e137.setAttribute("class", "git-fileline git-mono");
        const __t138 = document.createTextNode("");
        __e137.appendChild(__t138);
        __c136.push(bindText(__t138, () => $scope2.f.path));
        __frag135.appendChild(__e137);
        return { nodes: Array.from(__frag135.childNodes), cleanup: () => {
          for (const c of __c136) try {
            c();
          } catch (e) {
          }
        } };
      })(__childScope134);
    }
  });
  const __wireBoundary_b41 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.staged().length > 0) return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag126 = document.createDocumentFragment();
        const __c127 = [];
        const __e128 = document.createElement("section");
        __e128.setAttribute("class", "git-filegroup");
        const __e129 = document.createElement("h3");
        const __t130 = document.createTextNode("Staged (");
        __e129.appendChild(__t130);
        const __t131 = document.createTextNode("");
        __e129.appendChild(__t131);
        __c127.push(bindText(__t131, () => $scope.staged().length));
        const __t132 = document.createTextNode(")");
        __e129.appendChild(__t132);
        __e128.appendChild(__e129);
        const __forAnchor133 = document.createComment("for");
        __e128.appendChild(__forAnchor133);
        __c127.push(__wireBoundary_b43(__forAnchor133));
        __frag126.appendChild(__e128);
        return { nodes: Array.from(__frag126.childNodes), cleanup: () => {
          for (const c of __c127) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b47 = (anchor) => bindFor({
    anchor,
    items: () => $scope.modified(),
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return (($scope2) => f.path)(__child);
    },
    render: (f, __i) => {
      const __childScope148 = Object.create($scope);
      __childScope148.f = f;
      __childScope148.__i = __i;
      return (($scope2) => {
        const __frag149 = document.createDocumentFragment();
        const __c150 = [];
        const __e151 = document.createElement("div");
        __e151.setAttribute("class", "git-fileline git-mono");
        const __t152 = document.createTextNode("");
        __e151.appendChild(__t152);
        __c150.push(bindText(__t152, () => $scope2.f.path));
        __frag149.appendChild(__e151);
        return { nodes: Array.from(__frag149.childNodes), cleanup: () => {
          for (const c of __c150) try {
            c();
          } catch (e) {
          }
        } };
      })(__childScope148);
    }
  });
  const __wireBoundary_b45 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.modified().length > 0) return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag140 = document.createDocumentFragment();
        const __c141 = [];
        const __e142 = document.createElement("section");
        __e142.setAttribute("class", "git-filegroup");
        const __e143 = document.createElement("h3");
        const __t144 = document.createTextNode("Modified (");
        __e143.appendChild(__t144);
        const __t145 = document.createTextNode("");
        __e143.appendChild(__t145);
        __c141.push(bindText(__t145, () => $scope.modified().length));
        const __t146 = document.createTextNode(")");
        __e143.appendChild(__t146);
        __e142.appendChild(__e143);
        const __forAnchor147 = document.createComment("for");
        __e142.appendChild(__forAnchor147);
        __c141.push(__wireBoundary_b47(__forAnchor147));
        __frag140.appendChild(__e142);
        return { nodes: Array.from(__frag140.childNodes), cleanup: () => {
          for (const c of __c141) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b51 = (anchor) => bindFor({
    anchor,
    items: () => $scope.untracked(),
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return (($scope2) => f.path)(__child);
    },
    render: (f, __i) => {
      const __childScope162 = Object.create($scope);
      __childScope162.f = f;
      __childScope162.__i = __i;
      return (($scope2) => {
        const __frag163 = document.createDocumentFragment();
        const __c164 = [];
        const __e165 = document.createElement("div");
        __e165.setAttribute("class", "git-fileline git-mono");
        const __t166 = document.createTextNode("");
        __e165.appendChild(__t166);
        __c164.push(bindText(__t166, () => $scope2.f.path));
        __frag163.appendChild(__e165);
        return { nodes: Array.from(__frag163.childNodes), cleanup: () => {
          for (const c of __c164) try {
            c();
          } catch (e) {
          }
        } };
      })(__childScope162);
    }
  });
  const __wireBoundary_b49 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.untracked().length > 0) return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag154 = document.createDocumentFragment();
        const __c155 = [];
        const __e156 = document.createElement("section");
        __e156.setAttribute("class", "git-filegroup");
        const __e157 = document.createElement("h3");
        const __t158 = document.createTextNode("Untracked (");
        __e157.appendChild(__t158);
        const __t159 = document.createTextNode("");
        __e157.appendChild(__t159);
        __c155.push(bindText(__t159, () => $scope.untracked().length));
        const __t160 = document.createTextNode(")");
        __e157.appendChild(__t160);
        __e156.appendChild(__e157);
        const __forAnchor161 = document.createComment("for");
        __e156.appendChild(__forAnchor161);
        __c155.push(__wireBoundary_b51(__forAnchor161));
        __frag154.appendChild(__e156);
        return { nodes: Array.from(__frag154.childNodes), cleanup: () => {
          for (const c of __c155) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b20 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.clean()) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag48 = document.createDocumentFragment();
        const __c49 = [];
        const __e50 = document.createElement("div");
        __e50.setAttribute("class", "git-clean");
        const __e51 = document.createElement("div");
        __e51.setAttribute("class", "git-hero");
        const __e52 = document.createElement("div");
        __e52.setAttribute("class", "git-halo");
        const __e53 = document.createElement("ship-icon");
        __e53.setAttribute("name", "check");
        __e53.setAttribute("size", "md");
        __e52.appendChild(__e53);
        __e51.appendChild(__e52);
        const __e54 = document.createElement("div");
        const __e55 = document.createElement("h2");
        __e55.setAttribute("class", "git-hero-title");
        const __t56 = document.createTextNode("Working tree is clean.");
        __e55.appendChild(__t56);
        __e54.appendChild(__e55);
        const __e57 = document.createElement("p");
        __e57.setAttribute("class", "git-hero-sub");
        const __t58 = document.createTextNode("\n                  Nothing to commit on ");
        __e57.appendChild(__t58);
        const __e59 = document.createElement("span");
        __e59.setAttribute("class", "git-mono");
        const __t60 = document.createTextNode("");
        __e59.appendChild(__t60);
        __c49.push(bindText(__t60, () => $scope.branch()));
        __e57.appendChild(__e59);
        const __t61 = document.createTextNode(".\n                  ");
        __e57.appendChild(__t61);
        const __t62 = document.createTextNode("");
        __e57.appendChild(__t62);
        __c49.push(bindText(__t62, () => $scope.ahead() === 0 && $scope.behind() === 0 ? "You're up to date with the upstream." : ""));
        const __t63 = document.createTextNode("");
        __e57.appendChild(__t63);
        __c49.push(bindText(__t63, () => $scope.ahead() > 0 ? $scope.ahead() + " commit" + ($scope.ahead() === 1 ? "" : "s") + " ahead of upstream." : ""));
        const __t64 = document.createTextNode("");
        __e57.appendChild(__t64);
        __c49.push(bindText(__t64, () => $scope.behind() > 0 ? " " + $scope.behind() + " commit" + ($scope.behind() === 1 ? "" : "s") + " behind upstream." : ""));
        __e54.appendChild(__e57);
        __e51.appendChild(__e54);
        __e50.appendChild(__e51);
        const __e65 = document.createElement("div");
        __e65.setAttribute("class", "git-two-col");
        const __e66 = document.createElement("section");
        __e66.setAttribute("class", "git-sec");
        const __e67 = document.createElement("h3");
        const __t68 = document.createTextNode("Recent on this branch");
        __e67.appendChild(__t68);
        __e66.appendChild(__e67);
        const __ifAnchor69 = document.createComment("if");
        __e66.appendChild(__ifAnchor69);
        __c49.push(__wireBoundary_b25(__ifAnchor69));
        __e65.appendChild(__e66);
        const __e98 = document.createElement("section");
        __e98.setAttribute("class", "git-sec");
        const __e99 = document.createElement("h3");
        const __t100 = document.createTextNode("Branches");
        __e99.appendChild(__t100);
        __e98.appendChild(__e99);
        const __ifAnchor101 = document.createComment("if");
        __e98.appendChild(__ifAnchor101);
        __c49.push(__wireBoundary_b34(__ifAnchor101));
        __e65.appendChild(__e98);
        __e50.appendChild(__e65);
        __frag48.appendChild(__e50);
        return { nodes: Array.from(__frag48.childNodes), cleanup: () => {
          for (const c of __c49) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag122 = document.createDocumentFragment();
        const __c123 = [];
        const __e124 = document.createElement("div");
        __e124.setAttribute("class", "git-changes");
        const __ifAnchor125 = document.createComment("if");
        __e124.appendChild(__ifAnchor125);
        __c123.push(__wireBoundary_b41(__ifAnchor125));
        const __ifAnchor139 = document.createComment("if");
        __e124.appendChild(__ifAnchor139);
        __c123.push(__wireBoundary_b45(__ifAnchor139));
        const __ifAnchor153 = document.createComment("if");
        __e124.appendChild(__ifAnchor153);
        __c123.push(__wireBoundary_b49(__ifAnchor153));
        __frag122.appendChild(__e124);
        return { nodes: Array.from(__frag122.childNodes), cleanup: () => {
          for (const c of __c123) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b19 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tab() === "changes") return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag45 = document.createDocumentFragment();
        const __c46 = [];
        const __ifAnchor47 = document.createComment("if");
        __frag45.appendChild(__ifAnchor47);
        __c46.push(__wireBoundary_b20(__ifAnchor47));
        return { nodes: Array.from(__frag45.childNodes), cleanup: () => {
          for (const c of __c46) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b59 = (anchor) => bindFor({
    anchor,
    items: () => $scope.commitFilter.filtered(),
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return (($scope2) => c.sha)(__child);
    },
    render: (c, __i) => {
      const __childScope187 = Object.create($scope);
      __childScope187.c = c;
      __childScope187.__i = __i;
      return (($scope2) => {
        const __frag188 = document.createDocumentFragment();
        const __c189 = [];
        const __e190 = document.createElement("div");
        __e190.setAttribute("class", "git-history-row");
        const __e191 = document.createElement("sui-avatar");
        __e191.setAttribute("size", "sm");
        __e191.setAttribute("shape", "round");
        __c189.push(bindAttr(__e191, "name", () => $scope2.c.author));
        __e190.appendChild(__e191);
        const __e192 = document.createElement("div");
        __e192.setAttribute("class", "git-row-body");
        const __e193 = document.createElement("div");
        __e193.setAttribute("class", "git-row-title");
        const __t194 = document.createTextNode("");
        __e193.appendChild(__t194);
        __c189.push(bindText(__t194, () => $scope2.c.subject));
        __e192.appendChild(__e193);
        const __e195 = document.createElement("div");
        __e195.setAttribute("class", "git-row-sub git-mono");
        const __t196 = document.createTextNode("");
        __e195.appendChild(__t196);
        __c189.push(bindText(__t196, () => $scope2.c.short_sha));
        const __t197 = document.createTextNode(" &middot; ");
        __e195.appendChild(__t197);
        const __t198 = document.createTextNode("");
        __e195.appendChild(__t198);
        __c189.push(bindText(__t198, () => $scope2.c.author));
        const __t199 = document.createTextNode(" &middot; ");
        __e195.appendChild(__t199);
        const __t200 = document.createTextNode("");
        __e195.appendChild(__t200);
        __c189.push(bindText(__t200, () => $scope2.c.date));
        __e192.appendChild(__e195);
        __e190.appendChild(__e192);
        const __e201 = document.createElement("div");
        __e201.setAttribute("class", "git-deltas");
        const __e202 = document.createElement("span");
        __e202.setAttribute("class", "git-add");
        const __t203 = document.createTextNode("+");
        __e202.appendChild(__t203);
        const __t204 = document.createTextNode("");
        __e202.appendChild(__t204);
        __c189.push(bindText(__t204, () => $scope2.c.additions));
        __e201.appendChild(__e202);
        const __e205 = document.createElement("span");
        __e205.setAttribute("class", "git-del");
        const __t206 = document.createTextNode("&minus;");
        __e205.appendChild(__t206);
        const __t207 = document.createTextNode("");
        __e205.appendChild(__t207);
        __c189.push(bindText(__t207, () => $scope2.c.deletions));
        __e201.appendChild(__e205);
        __e190.appendChild(__e201);
        __frag188.appendChild(__e190);
        return { nodes: Array.from(__frag188.childNodes), cleanup: () => {
          for (const c2 of __c189) try {
            c2();
          } catch (e) {
          }
        } };
      })(__childScope187);
    }
  });
  const __wireBoundary_b57 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.commitFilter.filtered().length === 0) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag178 = document.createDocumentFragment();
        const __c179 = [];
        const __e180 = document.createElement("sui-empty-state");
        __e180.setAttribute("tone", "muted");
        __e180.setAttribute("size", "md");
        const __e181 = document.createElement("span");
        __e181.setAttribute("slot", "title");
        const __t182 = document.createTextNode("");
        __e181.appendChild(__t182);
        __c179.push(bindText(__t182, () => $scope.commitsCount() === 0 ? "No commits" : "No matching commits"));
        __e180.appendChild(__e181);
        __frag178.appendChild(__e180);
        return { nodes: Array.from(__frag178.childNodes), cleanup: () => {
          for (const c of __c179) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag183 = document.createDocumentFragment();
        const __c184 = [];
        const __e185 = document.createElement("div");
        __e185.setAttribute("class", "git-history-rows");
        const __forAnchor186 = document.createComment("for");
        __e185.appendChild(__forAnchor186);
        __c184.push(__wireBoundary_b59(__forAnchor186));
        __frag183.appendChild(__e185);
        return { nodes: Array.from(__frag183.childNodes), cleanup: () => {
          for (const c of __c184) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b53 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tab() === "history") return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag168 = document.createDocumentFragment();
        const __c169 = [];
        const __e170 = document.createElement("div");
        __e170.setAttribute("class", "git-history");
        const __e171 = document.createElement("div");
        __e171.setAttribute("class", "git-history-search");
        const __e172 = document.createElement("input");
        __e172.setAttribute("class", "git-search");
        __e172.setAttribute("placeholder", "Filter by subject, sha, or author");
        __c169.push(bindValue(__e172, $scope.commitFilter.query));
        __e171.appendChild(__e172);
        const __e173 = document.createElement("span");
        __e173.setAttribute("class", "git-history-count");
        const __t174 = document.createTextNode("");
        __e173.appendChild(__t174);
        __c169.push(bindText(__t174, () => $scope.commitFilter.filtered().length));
        const __t175 = document.createTextNode(" of ");
        __e173.appendChild(__t175);
        const __t176 = document.createTextNode("");
        __e173.appendChild(__t176);
        __c169.push(bindText(__t176, () => $scope.commitsCount()));
        __e171.appendChild(__e173);
        __e170.appendChild(__e171);
        const __ifAnchor177 = document.createComment("if");
        __e170.appendChild(__ifAnchor177);
        __c169.push(__wireBoundary_b57(__ifAnchor177));
        __frag168.appendChild(__e170);
        return { nodes: Array.from(__frag168.childNodes), cleanup: () => {
          for (const c of __c169) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b69 = (anchor) => bindFor({
    anchor,
    items: () => $scope.branches(),
    key: (b, __i) => {
      const __child = Object.create($scope);
      __child.b = b;
      __child.__i = __i;
      return (($scope2) => b.name)(__child);
    },
    render: (b, __i) => {
      const __childScope222 = Object.create($scope);
      __childScope222.b = b;
      __childScope222.__i = __i;
      return (($scope2) => {
        const __frag223 = document.createDocumentFragment();
        const __c224 = [];
        const __e225 = document.createElement("div");
        __e225.setAttribute("class", "git-history-row");
        const __e226 = document.createElement("ship-icon");
        __e226.setAttribute("name", "git-branch");
        __e226.setAttribute("size", "sm");
        __c224.push(bindAttr(__e226, "class", () => $scope2.b.active ? "git-icon-active" : "git-icon-muted"));
        __e225.appendChild(__e226);
        const __e227 = document.createElement("div");
        __e227.setAttribute("class", "git-row-body");
        const __e228 = document.createElement("div");
        __e228.setAttribute("class", "git-row-title git-mono");
        const __t229 = document.createTextNode("");
        __e228.appendChild(__t229);
        __c224.push(bindText(__t229, () => $scope2.b.name));
        const __t230 = document.createTextNode("");
        __e228.appendChild(__t230);
        __c224.push(bindText(__t230, () => $scope2.b.active ? " (HEAD)" : ""));
        __e227.appendChild(__e228);
        const __e231 = document.createElement("div");
        __e231.setAttribute("class", "git-row-sub");
        const __t232 = document.createTextNode("");
        __e231.appendChild(__t232);
        __c224.push(bindText(__t232, () => $scope2.b.upstream ? $scope2.b.upstream : "local only"));
        __e227.appendChild(__e231);
        __e225.appendChild(__e227);
        const __e233 = document.createElement("div");
        __e233.setAttribute("class", "git-deltas");
        const __t234 = document.createTextNode("");
        __e233.appendChild(__t234);
        __c224.push(bindText(__t234, () => $scope2.b.ahead > 0 ? "\xE2\x86\x91" + $scope2.b.ahead : ""));
        const __t235 = document.createTextNode("");
        __e233.appendChild(__t235);
        __c224.push(bindText(__t235, () => $scope2.b.behind > 0 ? " \xE2\x86\x93" + $scope2.b.behind : ""));
        __e225.appendChild(__e233);
        __frag223.appendChild(__e225);
        return { nodes: Array.from(__frag223.childNodes), cleanup: () => {
          for (const c of __c224) try {
            c();
          } catch (e) {
          }
        } };
      })(__childScope222);
    }
  });
  const __wireBoundary_b68 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.branches().length === 0) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag213 = document.createDocumentFragment();
        const __c214 = [];
        const __e215 = document.createElement("sui-empty-state");
        __e215.setAttribute("tone", "muted");
        __e215.setAttribute("size", "md");
        const __e216 = document.createElement("span");
        __e216.setAttribute("slot", "title");
        const __t217 = document.createTextNode("No branches");
        __e216.appendChild(__t217);
        __e215.appendChild(__e216);
        __frag213.appendChild(__e215);
        return { nodes: Array.from(__frag213.childNodes), cleanup: () => {
          for (const c of __c214) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag218 = document.createDocumentFragment();
        const __c219 = [];
        const __e220 = document.createElement("div");
        __e220.setAttribute("class", "git-history-rows");
        const __forAnchor221 = document.createComment("for");
        __e220.appendChild(__forAnchor221);
        __c219.push(__wireBoundary_b69(__forAnchor221));
        __frag218.appendChild(__e220);
        return { nodes: Array.from(__frag218.childNodes), cleanup: () => {
          for (const c of __c219) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b67 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.tab() === "branches") return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag209 = document.createDocumentFragment();
        const __c210 = [];
        const __e211 = document.createElement("div");
        __e211.setAttribute("class", "git-branches");
        const __ifAnchor212 = document.createComment("if");
        __e211.appendChild(__ifAnchor212);
        __c210.push(__wireBoundary_b68(__ifAnchor212));
        __frag209.appendChild(__e211);
        return { nodes: Array.from(__frag209.childNodes), cleanup: () => {
          for (const c of __c210) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b3 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.overview().kind === "ok") return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag18 = document.createDocumentFragment();
        const __c19 = [];
        const __e20 = document.createElement("header");
        __e20.setAttribute("class", "git-head");
        const __e21 = document.createElement("div");
        __e21.setAttribute("class", "git-head-titles");
        const __e22 = document.createElement("h1");
        __e22.setAttribute("class", "git-title");
        const __t23 = document.createTextNode("Git");
        __e22.appendChild(__t23);
        __e21.appendChild(__e22);
        const __e24 = document.createElement("p");
        __e24.setAttribute("class", "git-sub");
        const __e25 = document.createElement("span");
        __e25.setAttribute("class", "git-mono");
        const __t26 = document.createTextNode("");
        __e25.appendChild(__t26);
        __c19.push(bindText(__t26, () => $scope.branch()));
        __e24.appendChild(__e25);
        const __t27 = document.createTextNode("");
        __e24.appendChild(__t27);
        __c19.push(bindText(__t27, () => $scope.ahead() > 0 ? " \xE2\x86\x91" + $scope.ahead() : ""));
        const __t28 = document.createTextNode("");
        __e24.appendChild(__t28);
        __c19.push(bindText(__t28, () => $scope.behind() > 0 ? " \xE2\x86\x93" + $scope.behind() : ""));
        const __t29 = document.createTextNode("\n          &middot;\n          ");
        __e24.appendChild(__t29);
        const __e30 = document.createElement("span");
        __e30.setAttribute("class", "git-mono");
        const __t31 = document.createTextNode("");
        __e30.appendChild(__t31);
        __c19.push(bindText(__t31, () => $scope.headShort()));
        __e24.appendChild(__e30);
        __e21.appendChild(__e24);
        __e20.appendChild(__e21);
        const __e32 = document.createElement("div");
        __e32.setAttribute("class", "git-head-actions");
        const __e33 = document.createElement("sui-chip");
        __c19.push(bindAttr(__e33, "tone", () => $scope.clean() ? "success" : "warning"));
        __e33.setAttribute("variant", "soft");
        __e33.setAttribute("size", "sm");
        const __t34 = document.createTextNode("");
        __e33.appendChild(__t34);
        __c19.push(bindText(__t34, () => $scope.clean() ? "clean" : $scope.totalChanged() + " changed"));
        __e32.appendChild(__e33);
        __e20.appendChild(__e32);
        __frag18.appendChild(__e20);
        const __e35 = document.createElement("div");
        __e35.setAttribute("class", "git-tabs-row");
        const __e36 = document.createElement("sui-tabs");
        __e36.setAttribute("variant", "underline");
        const __e37 = document.createElement("sui-tab");
        __e37.setAttribute("name", "changes");
        __c19.push(bindAttr(__e37, "active", () => $scope.tab() === "changes"));
        __c19.push(bindAttr(__e37, "count", () => $scope.totalChanged() + ""));
        __c19.push(on(__e37, "click", (function() {
          const __h = (() => $scope.tab.set("changes"));
          return typeof __h === "function" ? __h.bind(void 0) : (() => {
          });
        })(), { modifiers: [] }));
        const __t38 = document.createTextNode("Changes");
        __e37.appendChild(__t38);
        __e36.appendChild(__e37);
        const __e39 = document.createElement("sui-tab");
        __e39.setAttribute("name", "history");
        __c19.push(bindAttr(__e39, "active", () => $scope.tab() === "history"));
        __c19.push(bindAttr(__e39, "count", () => $scope.commitsCount() + ""));
        __c19.push(on(__e39, "click", (function() {
          const __h = (() => $scope.tab.set("history"));
          return typeof __h === "function" ? __h.bind(void 0) : (() => {
          });
        })(), { modifiers: [] }));
        const __t40 = document.createTextNode("History");
        __e39.appendChild(__t40);
        __e36.appendChild(__e39);
        const __e41 = document.createElement("sui-tab");
        __e41.setAttribute("name", "branches");
        __c19.push(bindAttr(__e41, "active", () => $scope.tab() === "branches"));
        __c19.push(bindAttr(__e41, "count", () => $scope.branchesCount() + ""));
        __c19.push(on(__e41, "click", (function() {
          const __h = (() => $scope.tab.set("branches"));
          return typeof __h === "function" ? __h.bind(void 0) : (() => {
          });
        })(), { modifiers: [] }));
        const __t42 = document.createTextNode("Branches");
        __e41.appendChild(__t42);
        __e36.appendChild(__e41);
        __e35.appendChild(__e36);
        __frag18.appendChild(__e35);
        const __e43 = document.createElement("div");
        __e43.setAttribute("class", "git-body");
        const __ifAnchor44 = document.createComment("if");
        __e43.appendChild(__ifAnchor44);
        __c19.push(__wireBoundary_b19(__ifAnchor44));
        const __ifAnchor167 = document.createComment("if");
        __e43.appendChild(__ifAnchor167);
        __c19.push(__wireBoundary_b53(__ifAnchor167));
        const __ifAnchor208 = document.createComment("if");
        __e43.appendChild(__ifAnchor208);
        __c19.push(__wireBoundary_b67(__ifAnchor208));
        __frag18.appendChild(__e43);
        return { nodes: Array.from(__frag18.childNodes), cleanup: () => {
          for (const c of __c19) try {
            c();
          } catch (e) {
          }
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
      const __e4 = document.createElement("div");
      __e4.setAttribute("class", "ship-view git-view");
      const __wlAnchor9 = document.createComment("while-loading");
      __e4.appendChild(__wlAnchor9);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor9, signal: $scope.overview, render: __renderWhileLoading_0 }));
      const __weAnchor16 = document.createComment("while-error");
      __e4.appendChild(__weAnchor16);
      __cleanups.push(bindWhileError({ anchor: __weAnchor16, signal: $scope.overview, render: __renderWhileError_0 }));
      const __ifAnchor17 = document.createComment("if");
      __e4.appendChild(__ifAnchor17);
      __cleanups.push(__wireBoundary_b3(__ifAnchor17));
      container.appendChild(__e4);
      return () => {
        for (const c of __cleanups) try {
          c();
        } catch (e) {
        }
      };
    },
    hydrate(root, ctx2) {
      const __cleanups = [];
      const __m = collectElementMarkers(root);
      if (__m["b0"]) __cleanups.push(hydrateWhileLoading(__m["b0"], "b0", () => $scope.overview(), __renderWhileLoading_0));
      if (__m["b1"]) __cleanups.push(hydrateWhileError(__m["b1"], "b1", () => $scope.overview(), __renderWhileError_0));
      if (__m["b3"]) {
        const __t = __m["b3"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode) __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b3(__anchor));
      }
      return () => {
        for (const c of __cleanups) try {
          c();
        } catch (e) {
        }
      };
    }
  });
});
export {
  view_compiled_default as default
};
