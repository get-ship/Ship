// views/git/view.js
import { bindAttr, bindFor, bindIf, bindText, bindValue, bindWhileError, bindWhileLoading, data, live, on, signal, view } from "@ship/ui-runtime";

// views/git/view.ts
import { controller, unwrapOr, useFilteredList } from "@ship/ui-runtime";
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
var view_default = controller({
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

// views/git/view.js
var __scopeEval = (src) => new Function("$scope", "with ($scope) { return (" + src + "); }");
var __E = [];
__E[0] = __scopeEval("err.message");
__E[1] = __scopeEval("overview().kind === 'ok'");
__E[2] = __scopeEval("branch()");
__E[3] = __scopeEval("ahead() > 0 ? ' \xE2\x86\x91' + ahead() : ''");
__E[4] = __scopeEval("behind() > 0 ? ' \xE2\x86\x93' + behind() : ''");
__E[5] = __scopeEval("headShort()");
__E[6] = __scopeEval("clean() ? 'success' : 'warning'");
__E[7] = __scopeEval("clean() ? 'clean' : totalChanged() + ' changed'");
__E[8] = __scopeEval("tab() === 'changes'");
__E[9] = __scopeEval("totalChanged() + ''");
__E[10] = __scopeEval("() => tab.set('changes')");
__E[11] = __scopeEval("tab() === 'history'");
__E[12] = __scopeEval("commitsCount() + ''");
__E[13] = __scopeEval("() => tab.set('history')");
__E[14] = __scopeEval("tab() === 'branches'");
__E[15] = __scopeEval("branchesCount() + ''");
__E[16] = __scopeEval("() => tab.set('branches')");
__E[17] = __scopeEval("clean()");
__E[18] = __scopeEval(`ahead() === 0 && behind() === 0 ? "You're up to date with the upstream." : ''`);
__E[19] = __scopeEval("ahead() > 0 ? ahead() + ' commit' + (ahead() === 1 ? '' : 's') + ' ahead of upstream.' : ''");
__E[20] = __scopeEval("behind() > 0 ? ' ' + behind() + ' commit' + (behind() === 1 ? '' : 's') + ' behind upstream.' : ''");
__E[21] = __scopeEval("recentCommits().length === 0");
__E[22] = __scopeEval("recentCommits()");
__E[23] = __scopeEval("c.sha");
__E[24] = __scopeEval("c.author");
__E[25] = __scopeEval("c.subject");
__E[26] = __scopeEval("c.short_sha");
__E[27] = __scopeEval("c.date");
__E[28] = __scopeEval("c.additions");
__E[29] = __scopeEval("c.deletions");
__E[30] = __scopeEval("recentBranches().length === 0");
__E[31] = __scopeEval("recentBranches()");
__E[32] = __scopeEval("b.name");
__E[33] = __scopeEval("b.active ? 'git-icon-active' : 'git-icon-muted'");
__E[34] = __scopeEval("b.upstream ? b.upstream : 'local only'");
__E[35] = __scopeEval("b.ahead > 0 ? '\xE2\x86\x91' + b.ahead : ''");
__E[36] = __scopeEval("b.behind > 0 ? ' \xE2\x86\x93' + b.behind : ''");
__E[37] = __scopeEval("staged().length > 0");
__E[38] = __scopeEval("staged().length");
__E[39] = __scopeEval("staged()");
__E[40] = __scopeEval("f.path");
__E[41] = __scopeEval("modified().length > 0");
__E[42] = __scopeEval("modified().length");
__E[43] = __scopeEval("modified()");
__E[44] = __scopeEval("untracked().length > 0");
__E[45] = __scopeEval("untracked().length");
__E[46] = __scopeEval("untracked()");
__E[47] = __scopeEval("commitFilter.query");
__E[48] = __scopeEval("commitFilter.filtered().length");
__E[49] = __scopeEval("commitsCount()");
__E[50] = __scopeEval("commitFilter.filtered().length === 0");
__E[51] = __scopeEval("commitsCount() === 0 ? 'No commits' : 'No matching commits'");
__E[52] = __scopeEval("commitFilter.filtered()");
__E[53] = __scopeEval("branches().length === 0");
__E[54] = __scopeEval("branches()");
__E[55] = __scopeEval("b.active ? ' (HEAD)' : ''");
var view_default2 = view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = /* @__PURE__ */ Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.overview = live(({ ship: ship2, route: route2 }) => ship2.git.overview());
  $scope.tab = signal("changes");
  if (view_default && typeof view_default.setup === "function") {
    const __r = view_default.setup(ctx);
    if (__r && typeof __r === "object") Object.assign($scope, __r);
  }
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __t0 = document.createTextNode("\n");
      container.appendChild(__t0);
      const __t1 = document.createTextNode("\n\n");
      container.appendChild(__t1);
      const __e2 = document.createElement("use-kbd");
      const __t3 = document.createTextNode("\n  ");
      __e2.appendChild(__t3);
      const __e4 = document.createElement("on");
      __e4.setAttribute("key", "1");
      __e4.setAttribute("action", "showChanges");
      __e2.appendChild(__e4);
      const __t5 = document.createTextNode("\n  ");
      __e2.appendChild(__t5);
      const __e6 = document.createElement("on");
      __e6.setAttribute("key", "2");
      __e6.setAttribute("action", "showHistory");
      __e2.appendChild(__e6);
      const __t7 = document.createTextNode("\n  ");
      __e2.appendChild(__t7);
      const __e8 = document.createElement("on");
      __e8.setAttribute("key", "3");
      __e8.setAttribute("action", "showBranches");
      __e2.appendChild(__e8);
      const __t9 = document.createTextNode("\n");
      __e2.appendChild(__t9);
      container.appendChild(__e2);
      const __t10 = document.createTextNode("\n\n");
      container.appendChild(__t10);
      const __e11 = document.createElement("div");
      __e11.setAttribute("class", "ship-view git-view");
      const __t12 = document.createTextNode("\n  ");
      __e11.appendChild(__t12);
      const __wlAnchor19 = document.createComment("while-loading");
      __e11.appendChild(__wlAnchor19);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor19, signal: $scope.overview, render: () => {
        const __frag20 = document.createDocumentFragment();
        const __c21 = [];
        const __t22 = document.createTextNode("\n    ");
        __frag20.appendChild(__t22);
        const __e23 = document.createElement("p");
        __e23.setAttribute("class", "git-loading");
        __e23.setAttribute("data-test", "loading");
        const __t24 = document.createTextNode("Loading repo overview&hellip;");
        __e23.appendChild(__t24);
        __frag20.appendChild(__e23);
        const __t25 = document.createTextNode("\n  ");
        __frag20.appendChild(__t25);
        return { nodes: Array.from(__frag20.childNodes), cleanup: () => {
          for (const c of __c21) try {
            c();
          } catch (e) {
          }
        } };
      } }));
      const __t26 = document.createTextNode("\n\n  ");
      __e11.appendChild(__t26);
      const __weAnchor35 = document.createComment("while-error");
      __e11.appendChild(__weAnchor35);
      __cleanups.push(bindWhileError({ anchor: __weAnchor35, signal: $scope.overview, render: (err) => {
        const __childScope38 = Object.create($scope2);
        __childScope38.err = err;
        const __frag36 = document.createDocumentFragment();
        const __c37 = [];
        const $scope2 = __childScope38;
        const __t39 = document.createTextNode("\n    ");
        __frag36.appendChild(__t39);
        const __e40 = document.createElement("p");
        __e40.setAttribute("class", "git-error");
        __e40.setAttribute("data-test", "error");
        const __t41 = document.createTextNode("Failed to load: ");
        __e40.appendChild(__t41);
        const __t42 = document.createTextNode("");
        __e40.appendChild(__t42);
        __c37.push(bindText(__t42, () => __E[0]($scope2)));
        __frag36.appendChild(__e40);
        const __t43 = document.createTextNode("\n  ");
        __frag36.appendChild(__t43);
        return { nodes: Array.from(__frag36.childNodes), cleanup: () => {
          for (const c of __c37) try {
            c();
          } catch (e) {
          }
        } };
      } }));
      const __t44 = document.createTextNode("\n\n  ");
      __e11.appendChild(__t44);
      const __ifAnchor45 = document.createComment("if");
      __e11.appendChild(__ifAnchor45);
      __cleanups.push(bindIf({
        anchor: __ifAnchor45,
        test: () => {
          if (__E[1]($scope)) return 0;
          return -1;
        },
        branches: [
          () => {
            const __frag46 = document.createDocumentFragment();
            const __c47 = [];
            const __t48 = document.createTextNode("\n    ");
            __frag46.appendChild(__t48);
            const __e49 = document.createElement("header");
            __e49.setAttribute("class", "git-head");
            const __t50 = document.createTextNode("\n      ");
            __e49.appendChild(__t50);
            const __e51 = document.createElement("div");
            __e51.setAttribute("class", "git-head-titles");
            const __t52 = document.createTextNode("\n        ");
            __e51.appendChild(__t52);
            const __e53 = document.createElement("h1");
            __e53.setAttribute("class", "git-title");
            const __t54 = document.createTextNode("Git");
            __e53.appendChild(__t54);
            __e51.appendChild(__e53);
            const __t55 = document.createTextNode("\n        ");
            __e51.appendChild(__t55);
            const __e56 = document.createElement("p");
            __e56.setAttribute("class", "git-sub");
            const __t57 = document.createTextNode("\n          ");
            __e56.appendChild(__t57);
            const __e58 = document.createElement("span");
            __e58.setAttribute("class", "git-mono");
            const __t59 = document.createTextNode("");
            __e58.appendChild(__t59);
            __c47.push(bindText(__t59, () => __E[2]($scope)));
            __e56.appendChild(__e58);
            const __t60 = document.createTextNode("\n          ");
            __e56.appendChild(__t60);
            const __t61 = document.createTextNode("");
            __e56.appendChild(__t61);
            __c47.push(bindText(__t61, () => __E[3]($scope)));
            const __t62 = document.createTextNode("\n          ");
            __e56.appendChild(__t62);
            const __t63 = document.createTextNode("");
            __e56.appendChild(__t63);
            __c47.push(bindText(__t63, () => __E[4]($scope)));
            const __t64 = document.createTextNode("\n          &middot;\n          ");
            __e56.appendChild(__t64);
            const __e65 = document.createElement("span");
            __e65.setAttribute("class", "git-mono");
            const __t66 = document.createTextNode("");
            __e65.appendChild(__t66);
            __c47.push(bindText(__t66, () => __E[5]($scope)));
            __e56.appendChild(__e65);
            const __t67 = document.createTextNode("\n        ");
            __e56.appendChild(__t67);
            __e51.appendChild(__e56);
            const __t68 = document.createTextNode("\n      ");
            __e51.appendChild(__t68);
            __e49.appendChild(__e51);
            const __t69 = document.createTextNode("\n      ");
            __e49.appendChild(__t69);
            const __e70 = document.createElement("div");
            __e70.setAttribute("class", "git-head-actions");
            const __t71 = document.createTextNode("\n        ");
            __e70.appendChild(__t71);
            const __e72 = document.createElement("set-chip");
            __c47.push(bindAttr(__e72, "tone", () => __E[6]($scope)));
            __e72.setAttribute("variant", "soft");
            __e72.setAttribute("size", "sm");
            const __t73 = document.createTextNode("\n          ");
            __e72.appendChild(__t73);
            const __t74 = document.createTextNode("");
            __e72.appendChild(__t74);
            __c47.push(bindText(__t74, () => __E[7]($scope)));
            const __t75 = document.createTextNode("\n        ");
            __e72.appendChild(__t75);
            __e70.appendChild(__e72);
            const __t76 = document.createTextNode("\n      ");
            __e70.appendChild(__t76);
            __e49.appendChild(__e70);
            const __t77 = document.createTextNode("\n    ");
            __e49.appendChild(__t77);
            __frag46.appendChild(__e49);
            const __t78 = document.createTextNode("\n\n    ");
            __frag46.appendChild(__t78);
            const __e79 = document.createElement("div");
            __e79.setAttribute("class", "git-tabs-row");
            const __t80 = document.createTextNode("\n      ");
            __e79.appendChild(__t80);
            const __e81 = document.createElement("set-tabs");
            __e81.setAttribute("variant", "underline");
            const __t82 = document.createTextNode("\n        ");
            __e81.appendChild(__t82);
            const __e83 = document.createElement("set-tab");
            __e83.setAttribute("name", "changes");
            __c47.push(bindAttr(__e83, "active", () => __E[8]($scope)));
            __c47.push(bindAttr(__e83, "count", () => __E[9]($scope)));
            __c47.push(on(__e83, "click", (function() {
              const __h = __E[10]($scope);
              return typeof __h === "function" ? __h.bind(void 0) : (() => {
              });
            })(), { modifiers: [] }));
            const __t84 = document.createTextNode("Changes");
            __e83.appendChild(__t84);
            __e81.appendChild(__e83);
            const __t85 = document.createTextNode("\n        ");
            __e81.appendChild(__t85);
            const __e86 = document.createElement("set-tab");
            __e86.setAttribute("name", "history");
            __c47.push(bindAttr(__e86, "active", () => __E[11]($scope)));
            __c47.push(bindAttr(__e86, "count", () => __E[12]($scope)));
            __c47.push(on(__e86, "click", (function() {
              const __h = __E[13]($scope);
              return typeof __h === "function" ? __h.bind(void 0) : (() => {
              });
            })(), { modifiers: [] }));
            const __t87 = document.createTextNode("History");
            __e86.appendChild(__t87);
            __e81.appendChild(__e86);
            const __t88 = document.createTextNode("\n        ");
            __e81.appendChild(__t88);
            const __e89 = document.createElement("set-tab");
            __e89.setAttribute("name", "branches");
            __c47.push(bindAttr(__e89, "active", () => __E[14]($scope)));
            __c47.push(bindAttr(__e89, "count", () => __E[15]($scope)));
            __c47.push(on(__e89, "click", (function() {
              const __h = __E[16]($scope);
              return typeof __h === "function" ? __h.bind(void 0) : (() => {
              });
            })(), { modifiers: [] }));
            const __t90 = document.createTextNode("Branches");
            __e89.appendChild(__t90);
            __e81.appendChild(__e89);
            const __t91 = document.createTextNode("\n      ");
            __e81.appendChild(__t91);
            __e79.appendChild(__e81);
            const __t92 = document.createTextNode("\n    ");
            __e79.appendChild(__t92);
            __frag46.appendChild(__e79);
            const __t93 = document.createTextNode("\n\n    ");
            __frag46.appendChild(__t93);
            const __e94 = document.createElement("div");
            __e94.setAttribute("class", "git-body");
            const __t95 = document.createTextNode("\n      ");
            __e94.appendChild(__t95);
            const __ifAnchor96 = document.createComment("if");
            __e94.appendChild(__ifAnchor96);
            __c47.push(bindIf({
              anchor: __ifAnchor96,
              test: () => {
                if (__E[8]($scope)) return 0;
                return -1;
              },
              branches: [
                () => {
                  const __frag97 = document.createDocumentFragment();
                  const __c98 = [];
                  const __t99 = document.createTextNode("\n        ");
                  __frag97.appendChild(__t99);
                  const __ifAnchor100 = document.createComment("if");
                  __frag97.appendChild(__ifAnchor100);
                  __c98.push(bindIf({
                    anchor: __ifAnchor100,
                    test: () => {
                      if (__E[17]($scope)) return 0;
                      return 1;
                    },
                    branches: [
                      () => {
                        const __frag101 = document.createDocumentFragment();
                        const __c102 = [];
                        const __t103 = document.createTextNode("\n          ");
                        __frag101.appendChild(__t103);
                        const __e104 = document.createElement("div");
                        __e104.setAttribute("class", "git-clean");
                        const __t105 = document.createTextNode("\n            ");
                        __e104.appendChild(__t105);
                        const __e106 = document.createElement("div");
                        __e106.setAttribute("class", "git-hero");
                        const __t107 = document.createTextNode("\n              ");
                        __e106.appendChild(__t107);
                        const __e108 = document.createElement("div");
                        __e108.setAttribute("class", "git-halo");
                        const __t109 = document.createTextNode("\n                ");
                        __e108.appendChild(__t109);
                        const __e110 = document.createElement("ship-icon");
                        __e110.setAttribute("name", "check");
                        __e110.setAttribute("size", "md");
                        __e108.appendChild(__e110);
                        const __t111 = document.createTextNode("\n              ");
                        __e108.appendChild(__t111);
                        __e106.appendChild(__e108);
                        const __t112 = document.createTextNode("\n              ");
                        __e106.appendChild(__t112);
                        const __e113 = document.createElement("div");
                        const __t114 = document.createTextNode("\n                ");
                        __e113.appendChild(__t114);
                        const __e115 = document.createElement("h2");
                        __e115.setAttribute("class", "git-hero-title");
                        const __t116 = document.createTextNode("Working tree is clean.");
                        __e115.appendChild(__t116);
                        __e113.appendChild(__e115);
                        const __t117 = document.createTextNode("\n                ");
                        __e113.appendChild(__t117);
                        const __e118 = document.createElement("p");
                        __e118.setAttribute("class", "git-hero-sub");
                        const __t119 = document.createTextNode("\n                  Nothing to commit on ");
                        __e118.appendChild(__t119);
                        const __e120 = document.createElement("span");
                        __e120.setAttribute("class", "git-mono");
                        const __t121 = document.createTextNode("");
                        __e120.appendChild(__t121);
                        __c102.push(bindText(__t121, () => __E[2]($scope)));
                        __e118.appendChild(__e120);
                        const __t122 = document.createTextNode(".\n                  ");
                        __e118.appendChild(__t122);
                        const __t123 = document.createTextNode("");
                        __e118.appendChild(__t123);
                        __c102.push(bindText(__t123, () => __E[18]($scope)));
                        const __t124 = document.createTextNode("\n                  ");
                        __e118.appendChild(__t124);
                        const __t125 = document.createTextNode("");
                        __e118.appendChild(__t125);
                        __c102.push(bindText(__t125, () => __E[19]($scope)));
                        const __t126 = document.createTextNode("\n                  ");
                        __e118.appendChild(__t126);
                        const __t127 = document.createTextNode("");
                        __e118.appendChild(__t127);
                        __c102.push(bindText(__t127, () => __E[20]($scope)));
                        const __t128 = document.createTextNode("\n                ");
                        __e118.appendChild(__t128);
                        __e113.appendChild(__e118);
                        const __t129 = document.createTextNode("\n              ");
                        __e113.appendChild(__t129);
                        __e106.appendChild(__e113);
                        const __t130 = document.createTextNode("\n            ");
                        __e106.appendChild(__t130);
                        __e104.appendChild(__e106);
                        const __t131 = document.createTextNode("\n\n            ");
                        __e104.appendChild(__t131);
                        const __e132 = document.createElement("div");
                        __e132.setAttribute("class", "git-two-col");
                        const __t133 = document.createTextNode("\n              ");
                        __e132.appendChild(__t133);
                        const __e134 = document.createElement("section");
                        __e134.setAttribute("class", "git-sec");
                        const __t135 = document.createTextNode("\n                ");
                        __e134.appendChild(__t135);
                        const __e136 = document.createElement("h3");
                        const __t137 = document.createTextNode("Recent on this branch");
                        __e136.appendChild(__t137);
                        __e134.appendChild(__e136);
                        const __t138 = document.createTextNode("\n                ");
                        __e134.appendChild(__t138);
                        const __ifAnchor139 = document.createComment("if");
                        __e134.appendChild(__ifAnchor139);
                        __c102.push(bindIf({
                          anchor: __ifAnchor139,
                          test: () => {
                            if (__E[21]($scope)) return 0;
                            return 1;
                          },
                          branches: [
                            () => {
                              const __frag140 = document.createDocumentFragment();
                              const __c141 = [];
                              const __t142 = document.createTextNode("\n                  ");
                              __frag140.appendChild(__t142);
                              const __e143 = document.createElement("p");
                              __e143.setAttribute("class", "git-empty");
                              const __t144 = document.createTextNode("No commits yet.");
                              __e143.appendChild(__t144);
                              __frag140.appendChild(__e143);
                              const __t145 = document.createTextNode("\n                ");
                              __frag140.appendChild(__t145);
                              return { nodes: Array.from(__frag140.childNodes), cleanup: () => {
                                for (const c of __c141) try {
                                  c();
                                } catch (e) {
                                }
                              } };
                            },
                            () => {
                              const __frag146 = document.createDocumentFragment();
                              const __c147 = [];
                              const __t148 = document.createTextNode("\n                  ");
                              __frag146.appendChild(__t148);
                              const __forAnchor149 = document.createComment("for");
                              __frag146.appendChild(__forAnchor149);
                              __c147.push(bindFor({
                                anchor: __forAnchor149,
                                source: () => __E[22]($scope),
                                key: (c, __i) => {
                                  const __child = Object.create($scope);
                                  __child.c = c;
                                  __child.__i = __i;
                                  return __E[23](__child);
                                },
                                render: (c, __i) => {
                                  const __childScope152 = Object.create($scope2);
                                  __childScope152.c = c;
                                  __childScope152.__i = __i;
                                  const __frag150 = document.createDocumentFragment();
                                  const __c151 = [];
                                  const $scope2 = __childScope152;
                                  const __t153 = document.createTextNode("\n                    ");
                                  __frag150.appendChild(__t153);
                                  const __e154 = document.createElement("div");
                                  __e154.setAttribute("class", "git-row");
                                  const __t155 = document.createTextNode("\n                      ");
                                  __e154.appendChild(__t155);
                                  const __e156 = document.createElement("set-avatar");
                                  __e156.setAttribute("size", "sm");
                                  __e156.setAttribute("shape", "round");
                                  __c151.push(bindAttr(__e156, "name", () => __E[24]($scope2)));
                                  __e154.appendChild(__e156);
                                  const __t157 = document.createTextNode("\n                      ");
                                  __e154.appendChild(__t157);
                                  const __e158 = document.createElement("div");
                                  __e158.setAttribute("class", "git-row-body");
                                  const __t159 = document.createTextNode("\n                        ");
                                  __e158.appendChild(__t159);
                                  const __e160 = document.createElement("div");
                                  __e160.setAttribute("class", "git-row-title");
                                  const __t161 = document.createTextNode("");
                                  __e160.appendChild(__t161);
                                  __c151.push(bindText(__t161, () => __E[25]($scope2)));
                                  __e158.appendChild(__e160);
                                  const __t162 = document.createTextNode("\n                        ");
                                  __e158.appendChild(__t162);
                                  const __e163 = document.createElement("div");
                                  __e163.setAttribute("class", "git-row-sub");
                                  const __t164 = document.createTextNode("");
                                  __e163.appendChild(__t164);
                                  __c151.push(bindText(__t164, () => __E[26]($scope2)));
                                  const __t165 = document.createTextNode(" &middot; ");
                                  __e163.appendChild(__t165);
                                  const __t166 = document.createTextNode("");
                                  __e163.appendChild(__t166);
                                  __c151.push(bindText(__t166, () => __E[24]($scope2)));
                                  const __t167 = document.createTextNode(" &middot; ");
                                  __e163.appendChild(__t167);
                                  const __t168 = document.createTextNode("");
                                  __e163.appendChild(__t168);
                                  __c151.push(bindText(__t168, () => __E[27]($scope2)));
                                  __e158.appendChild(__e163);
                                  const __t169 = document.createTextNode("\n                      ");
                                  __e158.appendChild(__t169);
                                  __e154.appendChild(__e158);
                                  const __t170 = document.createTextNode("\n                      ");
                                  __e154.appendChild(__t170);
                                  const __e171 = document.createElement("div");
                                  __e171.setAttribute("class", "git-deltas");
                                  const __t172 = document.createTextNode("\n                        ");
                                  __e171.appendChild(__t172);
                                  const __e173 = document.createElement("span");
                                  __e173.setAttribute("class", "git-add");
                                  const __t174 = document.createTextNode("+");
                                  __e173.appendChild(__t174);
                                  const __t175 = document.createTextNode("");
                                  __e173.appendChild(__t175);
                                  __c151.push(bindText(__t175, () => __E[28]($scope2)));
                                  __e171.appendChild(__e173);
                                  const __t176 = document.createTextNode("\n                        ");
                                  __e171.appendChild(__t176);
                                  const __e177 = document.createElement("span");
                                  __e177.setAttribute("class", "git-del");
                                  const __t178 = document.createTextNode("&minus;");
                                  __e177.appendChild(__t178);
                                  const __t179 = document.createTextNode("");
                                  __e177.appendChild(__t179);
                                  __c151.push(bindText(__t179, () => __E[29]($scope2)));
                                  __e171.appendChild(__e177);
                                  const __t180 = document.createTextNode("\n                      ");
                                  __e171.appendChild(__t180);
                                  __e154.appendChild(__e171);
                                  const __t181 = document.createTextNode("\n                    ");
                                  __e154.appendChild(__t181);
                                  __frag150.appendChild(__e154);
                                  const __t182 = document.createTextNode("\n                  ");
                                  __frag150.appendChild(__t182);
                                  return { nodes: Array.from(__frag150.childNodes), cleanup: () => {
                                    for (const c2 of __c151) try {
                                      c2();
                                    } catch (e) {
                                    }
                                  } };
                                }
                              }));
                              const __t183 = document.createTextNode("\n                ");
                              __frag146.appendChild(__t183);
                              return { nodes: Array.from(__frag146.childNodes), cleanup: () => {
                                for (const c of __c147) try {
                                  c();
                                } catch (e) {
                                }
                              } };
                            }
                          ]
                        }));
                        const __t184 = document.createTextNode("\n              ");
                        __e134.appendChild(__t184);
                        __e132.appendChild(__e134);
                        const __t185 = document.createTextNode("\n\n              ");
                        __e132.appendChild(__t185);
                        const __e186 = document.createElement("section");
                        __e186.setAttribute("class", "git-sec");
                        const __t187 = document.createTextNode("\n                ");
                        __e186.appendChild(__t187);
                        const __e188 = document.createElement("h3");
                        const __t189 = document.createTextNode("Branches");
                        __e188.appendChild(__t189);
                        __e186.appendChild(__e188);
                        const __t190 = document.createTextNode("\n                ");
                        __e186.appendChild(__t190);
                        const __ifAnchor191 = document.createComment("if");
                        __e186.appendChild(__ifAnchor191);
                        __c102.push(bindIf({
                          anchor: __ifAnchor191,
                          test: () => {
                            if (__E[30]($scope)) return 0;
                            return 1;
                          },
                          branches: [
                            () => {
                              const __frag192 = document.createDocumentFragment();
                              const __c193 = [];
                              const __t194 = document.createTextNode("\n                  ");
                              __frag192.appendChild(__t194);
                              const __e195 = document.createElement("p");
                              __e195.setAttribute("class", "git-empty");
                              const __t196 = document.createTextNode("No branches.");
                              __e195.appendChild(__t196);
                              __frag192.appendChild(__e195);
                              const __t197 = document.createTextNode("\n                ");
                              __frag192.appendChild(__t197);
                              return { nodes: Array.from(__frag192.childNodes), cleanup: () => {
                                for (const c of __c193) try {
                                  c();
                                } catch (e) {
                                }
                              } };
                            },
                            () => {
                              const __frag198 = document.createDocumentFragment();
                              const __c199 = [];
                              const __t200 = document.createTextNode("\n                  ");
                              __frag198.appendChild(__t200);
                              const __forAnchor201 = document.createComment("for");
                              __frag198.appendChild(__forAnchor201);
                              __c199.push(bindFor({
                                anchor: __forAnchor201,
                                source: () => __E[31]($scope),
                                key: (b, __i) => {
                                  const __child = Object.create($scope);
                                  __child.b = b;
                                  __child.__i = __i;
                                  return __E[32](__child);
                                },
                                render: (b, __i) => {
                                  const __childScope204 = Object.create($scope2);
                                  __childScope204.b = b;
                                  __childScope204.__i = __i;
                                  const __frag202 = document.createDocumentFragment();
                                  const __c203 = [];
                                  const $scope2 = __childScope204;
                                  const __t205 = document.createTextNode("\n                    ");
                                  __frag202.appendChild(__t205);
                                  const __e206 = document.createElement("div");
                                  __e206.setAttribute("class", "git-row");
                                  const __t207 = document.createTextNode("\n                      ");
                                  __e206.appendChild(__t207);
                                  const __e208 = document.createElement("ship-icon");
                                  __e208.setAttribute("name", "git-branch");
                                  __e208.setAttribute("size", "xs");
                                  __c203.push(bindAttr(__e208, "class", () => __E[33]($scope2)));
                                  __e206.appendChild(__e208);
                                  const __t209 = document.createTextNode("\n                      ");
                                  __e206.appendChild(__t209);
                                  const __e210 = document.createElement("div");
                                  __e210.setAttribute("class", "git-row-body");
                                  const __t211 = document.createTextNode("\n                        ");
                                  __e210.appendChild(__t211);
                                  const __e212 = document.createElement("div");
                                  __e212.setAttribute("class", "git-row-title git-mono");
                                  const __t213 = document.createTextNode("");
                                  __e212.appendChild(__t213);
                                  __c203.push(bindText(__t213, () => __E[32]($scope2)));
                                  __e210.appendChild(__e212);
                                  const __t214 = document.createTextNode("\n                        ");
                                  __e210.appendChild(__t214);
                                  const __e215 = document.createElement("div");
                                  __e215.setAttribute("class", "git-row-sub");
                                  const __t216 = document.createTextNode("");
                                  __e215.appendChild(__t216);
                                  __c203.push(bindText(__t216, () => __E[34]($scope2)));
                                  __e210.appendChild(__e215);
                                  const __t217 = document.createTextNode("\n                      ");
                                  __e210.appendChild(__t217);
                                  __e206.appendChild(__e210);
                                  const __t218 = document.createTextNode("\n                      ");
                                  __e206.appendChild(__t218);
                                  const __e219 = document.createElement("div");
                                  __e219.setAttribute("class", "git-deltas");
                                  const __t220 = document.createTextNode("\n                        ");
                                  __e219.appendChild(__t220);
                                  const __t221 = document.createTextNode("");
                                  __e219.appendChild(__t221);
                                  __c203.push(bindText(__t221, () => __E[35]($scope2)));
                                  const __t222 = document.createTextNode("\n                        ");
                                  __e219.appendChild(__t222);
                                  const __t223 = document.createTextNode("");
                                  __e219.appendChild(__t223);
                                  __c203.push(bindText(__t223, () => __E[36]($scope2)));
                                  const __t224 = document.createTextNode("\n                      ");
                                  __e219.appendChild(__t224);
                                  __e206.appendChild(__e219);
                                  const __t225 = document.createTextNode("\n                    ");
                                  __e206.appendChild(__t225);
                                  __frag202.appendChild(__e206);
                                  const __t226 = document.createTextNode("\n                  ");
                                  __frag202.appendChild(__t226);
                                  return { nodes: Array.from(__frag202.childNodes), cleanup: () => {
                                    for (const c of __c203) try {
                                      c();
                                    } catch (e) {
                                    }
                                  } };
                                }
                              }));
                              const __t227 = document.createTextNode("\n                ");
                              __frag198.appendChild(__t227);
                              return { nodes: Array.from(__frag198.childNodes), cleanup: () => {
                                for (const c of __c199) try {
                                  c();
                                } catch (e) {
                                }
                              } };
                            }
                          ]
                        }));
                        const __t228 = document.createTextNode("\n              ");
                        __e186.appendChild(__t228);
                        __e132.appendChild(__e186);
                        const __t229 = document.createTextNode("\n            ");
                        __e132.appendChild(__t229);
                        __e104.appendChild(__e132);
                        const __t230 = document.createTextNode("\n          ");
                        __e104.appendChild(__t230);
                        __frag101.appendChild(__e104);
                        const __t231 = document.createTextNode("\n        ");
                        __frag101.appendChild(__t231);
                        return { nodes: Array.from(__frag101.childNodes), cleanup: () => {
                          for (const c of __c102) try {
                            c();
                          } catch (e) {
                          }
                        } };
                      },
                      () => {
                        const __frag232 = document.createDocumentFragment();
                        const __c233 = [];
                        const __t234 = document.createTextNode("\n          ");
                        __frag232.appendChild(__t234);
                        const __e235 = document.createElement("div");
                        __e235.setAttribute("class", "git-changes");
                        const __t236 = document.createTextNode("\n            ");
                        __e235.appendChild(__t236);
                        const __ifAnchor237 = document.createComment("if");
                        __e235.appendChild(__ifAnchor237);
                        __c233.push(bindIf({
                          anchor: __ifAnchor237,
                          test: () => {
                            if (__E[37]($scope)) return 0;
                            return -1;
                          },
                          branches: [
                            () => {
                              const __frag238 = document.createDocumentFragment();
                              const __c239 = [];
                              const __t240 = document.createTextNode("\n              ");
                              __frag238.appendChild(__t240);
                              const __e241 = document.createElement("section");
                              __e241.setAttribute("class", "git-filegroup");
                              const __t242 = document.createTextNode("\n                ");
                              __e241.appendChild(__t242);
                              const __e243 = document.createElement("h3");
                              const __t244 = document.createTextNode("Staged (");
                              __e243.appendChild(__t244);
                              const __t245 = document.createTextNode("");
                              __e243.appendChild(__t245);
                              __c239.push(bindText(__t245, () => __E[38]($scope)));
                              const __t246 = document.createTextNode(")");
                              __e243.appendChild(__t246);
                              __e241.appendChild(__e243);
                              const __t247 = document.createTextNode("\n                ");
                              __e241.appendChild(__t247);
                              const __forAnchor248 = document.createComment("for");
                              __e241.appendChild(__forAnchor248);
                              __c239.push(bindFor({
                                anchor: __forAnchor248,
                                source: () => __E[39]($scope),
                                key: (f, __i) => {
                                  const __child = Object.create($scope);
                                  __child.f = f;
                                  __child.__i = __i;
                                  return __E[40](__child);
                                },
                                render: (f, __i) => {
                                  const __childScope251 = Object.create($scope2);
                                  __childScope251.f = f;
                                  __childScope251.__i = __i;
                                  const __frag249 = document.createDocumentFragment();
                                  const __c250 = [];
                                  const $scope2 = __childScope251;
                                  const __t252 = document.createTextNode("\n                  ");
                                  __frag249.appendChild(__t252);
                                  const __e253 = document.createElement("div");
                                  __e253.setAttribute("class", "git-fileline git-mono");
                                  const __t254 = document.createTextNode("");
                                  __e253.appendChild(__t254);
                                  __c250.push(bindText(__t254, () => __E[40]($scope2)));
                                  __frag249.appendChild(__e253);
                                  const __t255 = document.createTextNode("\n                ");
                                  __frag249.appendChild(__t255);
                                  return { nodes: Array.from(__frag249.childNodes), cleanup: () => {
                                    for (const c of __c250) try {
                                      c();
                                    } catch (e) {
                                    }
                                  } };
                                }
                              }));
                              const __t256 = document.createTextNode("\n              ");
                              __e241.appendChild(__t256);
                              __frag238.appendChild(__e241);
                              const __t257 = document.createTextNode("\n            ");
                              __frag238.appendChild(__t257);
                              return { nodes: Array.from(__frag238.childNodes), cleanup: () => {
                                for (const c of __c239) try {
                                  c();
                                } catch (e) {
                                }
                              } };
                            }
                          ]
                        }));
                        const __t258 = document.createTextNode("\n            ");
                        __e235.appendChild(__t258);
                        const __ifAnchor259 = document.createComment("if");
                        __e235.appendChild(__ifAnchor259);
                        __c233.push(bindIf({
                          anchor: __ifAnchor259,
                          test: () => {
                            if (__E[41]($scope)) return 0;
                            return -1;
                          },
                          branches: [
                            () => {
                              const __frag260 = document.createDocumentFragment();
                              const __c261 = [];
                              const __t262 = document.createTextNode("\n              ");
                              __frag260.appendChild(__t262);
                              const __e263 = document.createElement("section");
                              __e263.setAttribute("class", "git-filegroup");
                              const __t264 = document.createTextNode("\n                ");
                              __e263.appendChild(__t264);
                              const __e265 = document.createElement("h3");
                              const __t266 = document.createTextNode("Modified (");
                              __e265.appendChild(__t266);
                              const __t267 = document.createTextNode("");
                              __e265.appendChild(__t267);
                              __c261.push(bindText(__t267, () => __E[42]($scope)));
                              const __t268 = document.createTextNode(")");
                              __e265.appendChild(__t268);
                              __e263.appendChild(__e265);
                              const __t269 = document.createTextNode("\n                ");
                              __e263.appendChild(__t269);
                              const __forAnchor270 = document.createComment("for");
                              __e263.appendChild(__forAnchor270);
                              __c261.push(bindFor({
                                anchor: __forAnchor270,
                                source: () => __E[43]($scope),
                                key: (f, __i) => {
                                  const __child = Object.create($scope);
                                  __child.f = f;
                                  __child.__i = __i;
                                  return __E[40](__child);
                                },
                                render: (f, __i) => {
                                  const __childScope273 = Object.create($scope2);
                                  __childScope273.f = f;
                                  __childScope273.__i = __i;
                                  const __frag271 = document.createDocumentFragment();
                                  const __c272 = [];
                                  const $scope2 = __childScope273;
                                  const __t274 = document.createTextNode("\n                  ");
                                  __frag271.appendChild(__t274);
                                  const __e275 = document.createElement("div");
                                  __e275.setAttribute("class", "git-fileline git-mono");
                                  const __t276 = document.createTextNode("");
                                  __e275.appendChild(__t276);
                                  __c272.push(bindText(__t276, () => __E[40]($scope2)));
                                  __frag271.appendChild(__e275);
                                  const __t277 = document.createTextNode("\n                ");
                                  __frag271.appendChild(__t277);
                                  return { nodes: Array.from(__frag271.childNodes), cleanup: () => {
                                    for (const c of __c272) try {
                                      c();
                                    } catch (e) {
                                    }
                                  } };
                                }
                              }));
                              const __t278 = document.createTextNode("\n              ");
                              __e263.appendChild(__t278);
                              __frag260.appendChild(__e263);
                              const __t279 = document.createTextNode("\n            ");
                              __frag260.appendChild(__t279);
                              return { nodes: Array.from(__frag260.childNodes), cleanup: () => {
                                for (const c of __c261) try {
                                  c();
                                } catch (e) {
                                }
                              } };
                            }
                          ]
                        }));
                        const __t280 = document.createTextNode("\n            ");
                        __e235.appendChild(__t280);
                        const __ifAnchor281 = document.createComment("if");
                        __e235.appendChild(__ifAnchor281);
                        __c233.push(bindIf({
                          anchor: __ifAnchor281,
                          test: () => {
                            if (__E[44]($scope)) return 0;
                            return -1;
                          },
                          branches: [
                            () => {
                              const __frag282 = document.createDocumentFragment();
                              const __c283 = [];
                              const __t284 = document.createTextNode("\n              ");
                              __frag282.appendChild(__t284);
                              const __e285 = document.createElement("section");
                              __e285.setAttribute("class", "git-filegroup");
                              const __t286 = document.createTextNode("\n                ");
                              __e285.appendChild(__t286);
                              const __e287 = document.createElement("h3");
                              const __t288 = document.createTextNode("Untracked (");
                              __e287.appendChild(__t288);
                              const __t289 = document.createTextNode("");
                              __e287.appendChild(__t289);
                              __c283.push(bindText(__t289, () => __E[45]($scope)));
                              const __t290 = document.createTextNode(")");
                              __e287.appendChild(__t290);
                              __e285.appendChild(__e287);
                              const __t291 = document.createTextNode("\n                ");
                              __e285.appendChild(__t291);
                              const __forAnchor292 = document.createComment("for");
                              __e285.appendChild(__forAnchor292);
                              __c283.push(bindFor({
                                anchor: __forAnchor292,
                                source: () => __E[46]($scope),
                                key: (f, __i) => {
                                  const __child = Object.create($scope);
                                  __child.f = f;
                                  __child.__i = __i;
                                  return __E[40](__child);
                                },
                                render: (f, __i) => {
                                  const __childScope295 = Object.create($scope2);
                                  __childScope295.f = f;
                                  __childScope295.__i = __i;
                                  const __frag293 = document.createDocumentFragment();
                                  const __c294 = [];
                                  const $scope2 = __childScope295;
                                  const __t296 = document.createTextNode("\n                  ");
                                  __frag293.appendChild(__t296);
                                  const __e297 = document.createElement("div");
                                  __e297.setAttribute("class", "git-fileline git-mono");
                                  const __t298 = document.createTextNode("");
                                  __e297.appendChild(__t298);
                                  __c294.push(bindText(__t298, () => __E[40]($scope2)));
                                  __frag293.appendChild(__e297);
                                  const __t299 = document.createTextNode("\n                ");
                                  __frag293.appendChild(__t299);
                                  return { nodes: Array.from(__frag293.childNodes), cleanup: () => {
                                    for (const c of __c294) try {
                                      c();
                                    } catch (e) {
                                    }
                                  } };
                                }
                              }));
                              const __t300 = document.createTextNode("\n              ");
                              __e285.appendChild(__t300);
                              __frag282.appendChild(__e285);
                              const __t301 = document.createTextNode("\n            ");
                              __frag282.appendChild(__t301);
                              return { nodes: Array.from(__frag282.childNodes), cleanup: () => {
                                for (const c of __c283) try {
                                  c();
                                } catch (e) {
                                }
                              } };
                            }
                          ]
                        }));
                        const __t302 = document.createTextNode("\n          ");
                        __e235.appendChild(__t302);
                        __frag232.appendChild(__e235);
                        const __t303 = document.createTextNode("\n        ");
                        __frag232.appendChild(__t303);
                        return { nodes: Array.from(__frag232.childNodes), cleanup: () => {
                          for (const c of __c233) try {
                            c();
                          } catch (e) {
                          }
                        } };
                      }
                    ]
                  }));
                  const __t304 = document.createTextNode("\n      ");
                  __frag97.appendChild(__t304);
                  return { nodes: Array.from(__frag97.childNodes), cleanup: () => {
                    for (const c of __c98) try {
                      c();
                    } catch (e) {
                    }
                  } };
                }
              ]
            }));
            const __t305 = document.createTextNode("\n\n      ");
            __e94.appendChild(__t305);
            const __ifAnchor306 = document.createComment("if");
            __e94.appendChild(__ifAnchor306);
            __c47.push(bindIf({
              anchor: __ifAnchor306,
              test: () => {
                if (__E[11]($scope)) return 0;
                return -1;
              },
              branches: [
                () => {
                  const __frag307 = document.createDocumentFragment();
                  const __c308 = [];
                  const __t309 = document.createTextNode("\n        ");
                  __frag307.appendChild(__t309);
                  const __e310 = document.createElement("div");
                  __e310.setAttribute("class", "git-history");
                  const __t311 = document.createTextNode("\n          ");
                  __e310.appendChild(__t311);
                  const __e312 = document.createElement("div");
                  __e312.setAttribute("class", "git-history-search");
                  const __t313 = document.createTextNode("\n            ");
                  __e312.appendChild(__t313);
                  const __e314 = document.createElement("input");
                  __e314.setAttribute("class", "git-search");
                  __e314.setAttribute("placeholder", "Filter by subject, sha, or author");
                  __c308.push(bindValue(__e314, __E[47]($scope)));
                  __e312.appendChild(__e314);
                  const __t315 = document.createTextNode("\n            ");
                  __e312.appendChild(__t315);
                  const __e316 = document.createElement("span");
                  __e316.setAttribute("class", "git-history-count");
                  const __t317 = document.createTextNode("");
                  __e316.appendChild(__t317);
                  __c308.push(bindText(__t317, () => __E[48]($scope)));
                  const __t318 = document.createTextNode(" of ");
                  __e316.appendChild(__t318);
                  const __t319 = document.createTextNode("");
                  __e316.appendChild(__t319);
                  __c308.push(bindText(__t319, () => __E[49]($scope)));
                  __e312.appendChild(__e316);
                  const __t320 = document.createTextNode("\n          ");
                  __e312.appendChild(__t320);
                  __e310.appendChild(__e312);
                  const __t321 = document.createTextNode("\n          ");
                  __e310.appendChild(__t321);
                  const __ifAnchor322 = document.createComment("if");
                  __e310.appendChild(__ifAnchor322);
                  __c308.push(bindIf({
                    anchor: __ifAnchor322,
                    test: () => {
                      if (__E[50]($scope)) return 0;
                      return 1;
                    },
                    branches: [
                      () => {
                        const __frag323 = document.createDocumentFragment();
                        const __c324 = [];
                        const __t325 = document.createTextNode("\n            ");
                        __frag323.appendChild(__t325);
                        const __e326 = document.createElement("set-empty-state");
                        __e326.setAttribute("tone", "muted");
                        __e326.setAttribute("size", "md");
                        const __t327 = document.createTextNode("\n              ");
                        __e326.appendChild(__t327);
                        const __e328 = document.createElement("span");
                        __e328.setAttribute("slot", "title");
                        const __t329 = document.createTextNode("");
                        __e328.appendChild(__t329);
                        __c324.push(bindText(__t329, () => __E[51]($scope)));
                        __e326.appendChild(__e328);
                        const __t330 = document.createTextNode("\n            ");
                        __e326.appendChild(__t330);
                        __frag323.appendChild(__e326);
                        const __t331 = document.createTextNode("\n          ");
                        __frag323.appendChild(__t331);
                        return { nodes: Array.from(__frag323.childNodes), cleanup: () => {
                          for (const c of __c324) try {
                            c();
                          } catch (e) {
                          }
                        } };
                      },
                      () => {
                        const __frag332 = document.createDocumentFragment();
                        const __c333 = [];
                        const __t334 = document.createTextNode("\n            ");
                        __frag332.appendChild(__t334);
                        const __e335 = document.createElement("div");
                        __e335.setAttribute("class", "git-history-rows");
                        const __t336 = document.createTextNode("\n              ");
                        __e335.appendChild(__t336);
                        const __forAnchor337 = document.createComment("for");
                        __e335.appendChild(__forAnchor337);
                        __c333.push(bindFor({
                          anchor: __forAnchor337,
                          source: () => __E[52]($scope),
                          key: (c, __i) => {
                            const __child = Object.create($scope);
                            __child.c = c;
                            __child.__i = __i;
                            return __E[23](__child);
                          },
                          render: (c, __i) => {
                            const __childScope340 = Object.create($scope2);
                            __childScope340.c = c;
                            __childScope340.__i = __i;
                            const __frag338 = document.createDocumentFragment();
                            const __c339 = [];
                            const $scope2 = __childScope340;
                            const __t341 = document.createTextNode("\n                ");
                            __frag338.appendChild(__t341);
                            const __e342 = document.createElement("div");
                            __e342.setAttribute("class", "git-history-row");
                            const __t343 = document.createTextNode("\n                  ");
                            __e342.appendChild(__t343);
                            const __e344 = document.createElement("set-avatar");
                            __e344.setAttribute("size", "sm");
                            __e344.setAttribute("shape", "round");
                            __c339.push(bindAttr(__e344, "name", () => __E[24]($scope2)));
                            __e342.appendChild(__e344);
                            const __t345 = document.createTextNode("\n                  ");
                            __e342.appendChild(__t345);
                            const __e346 = document.createElement("div");
                            __e346.setAttribute("class", "git-row-body");
                            const __t347 = document.createTextNode("\n                    ");
                            __e346.appendChild(__t347);
                            const __e348 = document.createElement("div");
                            __e348.setAttribute("class", "git-row-title");
                            const __t349 = document.createTextNode("");
                            __e348.appendChild(__t349);
                            __c339.push(bindText(__t349, () => __E[25]($scope2)));
                            __e346.appendChild(__e348);
                            const __t350 = document.createTextNode("\n                    ");
                            __e346.appendChild(__t350);
                            const __e351 = document.createElement("div");
                            __e351.setAttribute("class", "git-row-sub git-mono");
                            const __t352 = document.createTextNode("");
                            __e351.appendChild(__t352);
                            __c339.push(bindText(__t352, () => __E[26]($scope2)));
                            const __t353 = document.createTextNode(" &middot; ");
                            __e351.appendChild(__t353);
                            const __t354 = document.createTextNode("");
                            __e351.appendChild(__t354);
                            __c339.push(bindText(__t354, () => __E[24]($scope2)));
                            const __t355 = document.createTextNode(" &middot; ");
                            __e351.appendChild(__t355);
                            const __t356 = document.createTextNode("");
                            __e351.appendChild(__t356);
                            __c339.push(bindText(__t356, () => __E[27]($scope2)));
                            __e346.appendChild(__e351);
                            const __t357 = document.createTextNode("\n                  ");
                            __e346.appendChild(__t357);
                            __e342.appendChild(__e346);
                            const __t358 = document.createTextNode("\n                  ");
                            __e342.appendChild(__t358);
                            const __e359 = document.createElement("div");
                            __e359.setAttribute("class", "git-deltas");
                            const __t360 = document.createTextNode("\n                    ");
                            __e359.appendChild(__t360);
                            const __e361 = document.createElement("span");
                            __e361.setAttribute("class", "git-add");
                            const __t362 = document.createTextNode("+");
                            __e361.appendChild(__t362);
                            const __t363 = document.createTextNode("");
                            __e361.appendChild(__t363);
                            __c339.push(bindText(__t363, () => __E[28]($scope2)));
                            __e359.appendChild(__e361);
                            const __t364 = document.createTextNode("\n                    ");
                            __e359.appendChild(__t364);
                            const __e365 = document.createElement("span");
                            __e365.setAttribute("class", "git-del");
                            const __t366 = document.createTextNode("&minus;");
                            __e365.appendChild(__t366);
                            const __t367 = document.createTextNode("");
                            __e365.appendChild(__t367);
                            __c339.push(bindText(__t367, () => __E[29]($scope2)));
                            __e359.appendChild(__e365);
                            const __t368 = document.createTextNode("\n                  ");
                            __e359.appendChild(__t368);
                            __e342.appendChild(__e359);
                            const __t369 = document.createTextNode("\n                ");
                            __e342.appendChild(__t369);
                            __frag338.appendChild(__e342);
                            const __t370 = document.createTextNode("\n              ");
                            __frag338.appendChild(__t370);
                            return { nodes: Array.from(__frag338.childNodes), cleanup: () => {
                              for (const c2 of __c339) try {
                                c2();
                              } catch (e) {
                              }
                            } };
                          }
                        }));
                        const __t371 = document.createTextNode("\n            ");
                        __e335.appendChild(__t371);
                        __frag332.appendChild(__e335);
                        const __t372 = document.createTextNode("\n          ");
                        __frag332.appendChild(__t372);
                        return { nodes: Array.from(__frag332.childNodes), cleanup: () => {
                          for (const c of __c333) try {
                            c();
                          } catch (e) {
                          }
                        } };
                      }
                    ]
                  }));
                  const __t373 = document.createTextNode("\n        ");
                  __e310.appendChild(__t373);
                  __frag307.appendChild(__e310);
                  const __t374 = document.createTextNode("\n      ");
                  __frag307.appendChild(__t374);
                  return { nodes: Array.from(__frag307.childNodes), cleanup: () => {
                    for (const c of __c308) try {
                      c();
                    } catch (e) {
                    }
                  } };
                }
              ]
            }));
            const __t375 = document.createTextNode("\n\n      ");
            __e94.appendChild(__t375);
            const __ifAnchor376 = document.createComment("if");
            __e94.appendChild(__ifAnchor376);
            __c47.push(bindIf({
              anchor: __ifAnchor376,
              test: () => {
                if (__E[14]($scope)) return 0;
                return -1;
              },
              branches: [
                () => {
                  const __frag377 = document.createDocumentFragment();
                  const __c378 = [];
                  const __t379 = document.createTextNode("\n        ");
                  __frag377.appendChild(__t379);
                  const __e380 = document.createElement("div");
                  __e380.setAttribute("class", "git-branches");
                  const __t381 = document.createTextNode("\n          ");
                  __e380.appendChild(__t381);
                  const __ifAnchor382 = document.createComment("if");
                  __e380.appendChild(__ifAnchor382);
                  __c378.push(bindIf({
                    anchor: __ifAnchor382,
                    test: () => {
                      if (__E[53]($scope)) return 0;
                      return 1;
                    },
                    branches: [
                      () => {
                        const __frag383 = document.createDocumentFragment();
                        const __c384 = [];
                        const __t385 = document.createTextNode("\n            ");
                        __frag383.appendChild(__t385);
                        const __e386 = document.createElement("set-empty-state");
                        __e386.setAttribute("tone", "muted");
                        __e386.setAttribute("size", "md");
                        const __t387 = document.createTextNode("\n              ");
                        __e386.appendChild(__t387);
                        const __e388 = document.createElement("span");
                        __e388.setAttribute("slot", "title");
                        const __t389 = document.createTextNode("No branches");
                        __e388.appendChild(__t389);
                        __e386.appendChild(__e388);
                        const __t390 = document.createTextNode("\n            ");
                        __e386.appendChild(__t390);
                        __frag383.appendChild(__e386);
                        const __t391 = document.createTextNode("\n          ");
                        __frag383.appendChild(__t391);
                        return { nodes: Array.from(__frag383.childNodes), cleanup: () => {
                          for (const c of __c384) try {
                            c();
                          } catch (e) {
                          }
                        } };
                      },
                      () => {
                        const __frag392 = document.createDocumentFragment();
                        const __c393 = [];
                        const __t394 = document.createTextNode("\n            ");
                        __frag392.appendChild(__t394);
                        const __e395 = document.createElement("div");
                        __e395.setAttribute("class", "git-history-rows");
                        const __t396 = document.createTextNode("\n              ");
                        __e395.appendChild(__t396);
                        const __forAnchor397 = document.createComment("for");
                        __e395.appendChild(__forAnchor397);
                        __c393.push(bindFor({
                          anchor: __forAnchor397,
                          source: () => __E[54]($scope),
                          key: (b, __i) => {
                            const __child = Object.create($scope);
                            __child.b = b;
                            __child.__i = __i;
                            return __E[32](__child);
                          },
                          render: (b, __i) => {
                            const __childScope400 = Object.create($scope2);
                            __childScope400.b = b;
                            __childScope400.__i = __i;
                            const __frag398 = document.createDocumentFragment();
                            const __c399 = [];
                            const $scope2 = __childScope400;
                            const __t401 = document.createTextNode("\n                ");
                            __frag398.appendChild(__t401);
                            const __e402 = document.createElement("div");
                            __e402.setAttribute("class", "git-history-row");
                            const __t403 = document.createTextNode("\n                  ");
                            __e402.appendChild(__t403);
                            const __e404 = document.createElement("ship-icon");
                            __e404.setAttribute("name", "git-branch");
                            __e404.setAttribute("size", "sm");
                            __c399.push(bindAttr(__e404, "class", () => __E[33]($scope2)));
                            __e402.appendChild(__e404);
                            const __t405 = document.createTextNode("\n                  ");
                            __e402.appendChild(__t405);
                            const __e406 = document.createElement("div");
                            __e406.setAttribute("class", "git-row-body");
                            const __t407 = document.createTextNode("\n                    ");
                            __e406.appendChild(__t407);
                            const __e408 = document.createElement("div");
                            __e408.setAttribute("class", "git-row-title git-mono");
                            const __t409 = document.createTextNode("");
                            __e408.appendChild(__t409);
                            __c399.push(bindText(__t409, () => __E[32]($scope2)));
                            const __t410 = document.createTextNode("");
                            __e408.appendChild(__t410);
                            __c399.push(bindText(__t410, () => __E[55]($scope2)));
                            __e406.appendChild(__e408);
                            const __t411 = document.createTextNode("\n                    ");
                            __e406.appendChild(__t411);
                            const __e412 = document.createElement("div");
                            __e412.setAttribute("class", "git-row-sub");
                            const __t413 = document.createTextNode("");
                            __e412.appendChild(__t413);
                            __c399.push(bindText(__t413, () => __E[34]($scope2)));
                            __e406.appendChild(__e412);
                            const __t414 = document.createTextNode("\n                  ");
                            __e406.appendChild(__t414);
                            __e402.appendChild(__e406);
                            const __t415 = document.createTextNode("\n                  ");
                            __e402.appendChild(__t415);
                            const __e416 = document.createElement("div");
                            __e416.setAttribute("class", "git-deltas");
                            const __t417 = document.createTextNode("\n                    ");
                            __e416.appendChild(__t417);
                            const __t418 = document.createTextNode("");
                            __e416.appendChild(__t418);
                            __c399.push(bindText(__t418, () => __E[35]($scope2)));
                            const __t419 = document.createTextNode("\n                    ");
                            __e416.appendChild(__t419);
                            const __t420 = document.createTextNode("");
                            __e416.appendChild(__t420);
                            __c399.push(bindText(__t420, () => __E[36]($scope2)));
                            const __t421 = document.createTextNode("\n                  ");
                            __e416.appendChild(__t421);
                            __e402.appendChild(__e416);
                            const __t422 = document.createTextNode("\n                ");
                            __e402.appendChild(__t422);
                            __frag398.appendChild(__e402);
                            const __t423 = document.createTextNode("\n              ");
                            __frag398.appendChild(__t423);
                            return { nodes: Array.from(__frag398.childNodes), cleanup: () => {
                              for (const c of __c399) try {
                                c();
                              } catch (e) {
                              }
                            } };
                          }
                        }));
                        const __t424 = document.createTextNode("\n            ");
                        __e395.appendChild(__t424);
                        __frag392.appendChild(__e395);
                        const __t425 = document.createTextNode("\n          ");
                        __frag392.appendChild(__t425);
                        return { nodes: Array.from(__frag392.childNodes), cleanup: () => {
                          for (const c of __c393) try {
                            c();
                          } catch (e) {
                          }
                        } };
                      }
                    ]
                  }));
                  const __t426 = document.createTextNode("\n        ");
                  __e380.appendChild(__t426);
                  __frag377.appendChild(__e380);
                  const __t427 = document.createTextNode("\n      ");
                  __frag377.appendChild(__t427);
                  return { nodes: Array.from(__frag377.childNodes), cleanup: () => {
                    for (const c of __c378) try {
                      c();
                    } catch (e) {
                    }
                  } };
                }
              ]
            }));
            const __t428 = document.createTextNode("\n    ");
            __e94.appendChild(__t428);
            __frag46.appendChild(__e94);
            const __t429 = document.createTextNode("\n  ");
            __frag46.appendChild(__t429);
            return { nodes: Array.from(__frag46.childNodes), cleanup: () => {
              for (const c of __c47) try {
                c();
              } catch (e) {
              }
            } };
          }
        ]
      }));
      const __t430 = document.createTextNode("\n");
      __e11.appendChild(__t430);
      container.appendChild(__e11);
      const __t431 = document.createTextNode("\n");
      container.appendChild(__t431);
      return () => {
        for (const c of __cleanups) try {
          c();
        } catch (e) {
        }
      };
    },
    "__renderWhileLoading_0": () => {
      const __frag13 = document.createDocumentFragment();
      const __c14 = [];
      const __t15 = document.createTextNode("\n    ");
      __frag13.appendChild(__t15);
      const __e16 = document.createElement("p");
      __e16.setAttribute("class", "git-loading");
      __e16.setAttribute("data-test", "loading");
      const __t17 = document.createTextNode("Loading repo overview&hellip;");
      __e16.appendChild(__t17);
      __frag13.appendChild(__e16);
      const __t18 = document.createTextNode("\n  ");
      __frag13.appendChild(__t18);
      return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
        for (const c of __c14) try {
          c();
        } catch (e) {
        }
      } };
    },
    "__renderWhileError_0": (err) => {
      const __childScope29 = Object.create($scope2);
      __childScope29.err = err;
      const __frag27 = document.createDocumentFragment();
      const __c28 = [];
      const $scope2 = __childScope29;
      const __t30 = document.createTextNode("\n    ");
      __frag27.appendChild(__t30);
      const __e31 = document.createElement("p");
      __e31.setAttribute("class", "git-error");
      __e31.setAttribute("data-test", "error");
      const __t32 = document.createTextNode("Failed to load: ");
      __e31.appendChild(__t32);
      const __t33 = document.createTextNode("");
      __e31.appendChild(__t33);
      __c28.push(bindText(__t33, () => __E[0]($scope2)));
      __frag27.appendChild(__e31);
      const __t34 = document.createTextNode("\n  ");
      __frag27.appendChild(__t34);
      return { nodes: Array.from(__frag27.childNodes), cleanup: () => {
        for (const c of __c28) try {
          c();
        } catch (e) {
        }
      } };
    }
  });
});
var bindings = [
  { key: "b0", kind: "while-loading", expr: "overview", deps: ["overview"], renderKey: "__renderWhileLoading_0" },
  { key: "b1", kind: "while-error", expr: "overview", deps: ["overview"], renderKey: "__renderWhileError_0" },
  { key: "b3", kind: "if", expr: "overview().kind === 'ok'", deps: ["overview"] },
  { key: "b4", kind: "text", expr: "branch()", deps: ["branch"] },
  { key: "b5", kind: "text", expr: "ahead() > 0 ? ' \xE2\x86\x91' + ahead() : ''", deps: ["ahead"] },
  { key: "b6", kind: "text", expr: "behind() > 0 ? ' \xE2\x86\x93' + behind() : ''", deps: ["behind"] },
  { key: "b7", kind: "text", expr: "headShort()", deps: ["headShort"] },
  { key: "b8", kind: "attr:tone", expr: "clean() ? 'success' : 'warning'", deps: ["clean"] },
  { key: "b9", kind: "text", expr: "clean() ? 'clean' : totalChanged() + ' changed'", deps: ["clean", "totalChanged"] },
  { key: "b10", kind: "attr:active", expr: "tab() === 'changes'", deps: ["tab"] },
  { key: "b11", kind: "attr:count", expr: "totalChanged() + ''", deps: ["totalChanged"] },
  { key: "b12", kind: "on:click", expr: "() => tab.set('changes')", deps: ["tab", "changes"] },
  { key: "b13", kind: "attr:active", expr: "tab() === 'history'", deps: ["tab"] },
  { key: "b14", kind: "attr:count", expr: "commitsCount() + ''", deps: ["commitsCount"] },
  { key: "b15", kind: "on:click", expr: "() => tab.set('history')", deps: ["tab", "history"] },
  { key: "b16", kind: "attr:active", expr: "tab() === 'branches'", deps: ["tab"] },
  { key: "b17", kind: "attr:count", expr: "branchesCount() + ''", deps: ["branchesCount"] },
  { key: "b18", kind: "on:click", expr: "() => tab.set('branches')", deps: ["tab", "branches"] },
  { key: "b19", kind: "if", expr: "tab() === 'changes'", deps: ["tab"] },
  { key: "b20", kind: "if", expr: "clean()", deps: ["clean"] },
  { key: "b21", kind: "text", expr: "branch()", deps: ["branch"] },
  { key: "b22", kind: "text", expr: `ahead() === 0 && behind() === 0 ? "You're up to date with the upstream." : ''`, deps: ["ahead", "behind"] },
  { key: "b23", kind: "text", expr: "ahead() > 0 ? ahead() + ' commit' + (ahead() === 1 ? '' : 's') + ' ahead of upstream.' : ''", deps: ["ahead"] },
  { key: "b24", kind: "text", expr: "behind() > 0 ? ' ' + behind() + ' commit' + (behind() === 1 ? '' : 's') + ' behind upstream.' : ''", deps: ["behind"] },
  { key: "b25", kind: "if", expr: "recentCommits().length === 0", deps: ["recentCommits"] },
  { key: "b26", kind: "for", expr: "recentCommits()", deps: ["recentCommits"] },
  { key: "b27", kind: "attr:name", expr: "c.author", deps: ["c"] },
  { key: "b28", kind: "text", expr: "c.subject", deps: ["c"] },
  { key: "b29", kind: "text", expr: "c.short_sha", deps: ["c"] },
  { key: "b30", kind: "text", expr: "c.author", deps: ["c"] },
  { key: "b31", kind: "text", expr: "c.date", deps: ["c"] },
  { key: "b32", kind: "text", expr: "c.additions", deps: ["c"] },
  { key: "b33", kind: "text", expr: "c.deletions", deps: ["c"] },
  { key: "b34", kind: "if", expr: "recentBranches().length === 0", deps: ["recentBranches"] },
  { key: "b35", kind: "for", expr: "recentBranches()", deps: ["recentBranches"] },
  { key: "b36", kind: "attr:class", expr: "b.active ? 'git-icon-active' : 'git-icon-muted'", deps: ["b"] },
  { key: "b37", kind: "text", expr: "b.name", deps: ["b"] },
  { key: "b38", kind: "text", expr: "b.upstream ? b.upstream : 'local only'", deps: ["b"] },
  { key: "b39", kind: "text", expr: "b.ahead > 0 ? '\xE2\x86\x91' + b.ahead : ''", deps: ["b"] },
  { key: "b40", kind: "text", expr: "b.behind > 0 ? ' \xE2\x86\x93' + b.behind : ''", deps: ["b"] },
  { key: "b41", kind: "if", expr: "staged().length > 0", deps: ["staged"] },
  { key: "b42", kind: "text", expr: "staged().length", deps: ["staged"] },
  { key: "b43", kind: "for", expr: "staged()", deps: ["staged"] },
  { key: "b44", kind: "text", expr: "f.path", deps: ["f"] },
  { key: "b45", kind: "if", expr: "modified().length > 0", deps: ["modified"] },
  { key: "b46", kind: "text", expr: "modified().length", deps: ["modified"] },
  { key: "b47", kind: "for", expr: "modified()", deps: ["modified"] },
  { key: "b48", kind: "text", expr: "f.path", deps: ["f"] },
  { key: "b49", kind: "if", expr: "untracked().length > 0", deps: ["untracked"] },
  { key: "b50", kind: "text", expr: "untracked().length", deps: ["untracked"] },
  { key: "b51", kind: "for", expr: "untracked()", deps: ["untracked"] },
  { key: "b52", kind: "text", expr: "f.path", deps: ["f"] },
  { key: "b53", kind: "if", expr: "tab() === 'history'", deps: ["tab"] },
  { key: "b54", kind: "bind:value", expr: "commitFilter.query", deps: ["commitFilter"] },
  { key: "b55", kind: "text", expr: "commitFilter.filtered().length", deps: ["commitFilter"] },
  { key: "b56", kind: "text", expr: "commitsCount()", deps: ["commitsCount"] },
  { key: "b57", kind: "if", expr: "commitFilter.filtered().length === 0", deps: ["commitFilter"] },
  { key: "b58", kind: "text", expr: "commitsCount() === 0 ? 'No commits' : 'No matching commits'", deps: ["commitsCount"] },
  { key: "b59", kind: "for", expr: "commitFilter.filtered()", deps: ["commitFilter"] },
  { key: "b60", kind: "attr:name", expr: "c.author", deps: ["c"] },
  { key: "b61", kind: "text", expr: "c.subject", deps: ["c"] },
  { key: "b62", kind: "text", expr: "c.short_sha", deps: ["c"] },
  { key: "b63", kind: "text", expr: "c.author", deps: ["c"] },
  { key: "b64", kind: "text", expr: "c.date", deps: ["c"] },
  { key: "b65", kind: "text", expr: "c.additions", deps: ["c"] },
  { key: "b66", kind: "text", expr: "c.deletions", deps: ["c"] },
  { key: "b67", kind: "if", expr: "tab() === 'branches'", deps: ["tab"] },
  { key: "b68", kind: "if", expr: "branches().length === 0", deps: ["branches"] },
  { key: "b69", kind: "for", expr: "branches()", deps: ["branches"] },
  { key: "b70", kind: "attr:class", expr: "b.active ? 'git-icon-active' : 'git-icon-muted'", deps: ["b"] },
  { key: "b71", kind: "text", expr: "b.name", deps: ["b"] },
  { key: "b72", kind: "text", expr: "b.active ? ' (HEAD)' : ''", deps: ["b"] },
  { key: "b73", kind: "text", expr: "b.upstream ? b.upstream : 'local only'", deps: ["b"] },
  { key: "b74", kind: "text", expr: "b.ahead > 0 ? '\xE2\x86\x91' + b.ahead : ''", deps: ["b"] },
  { key: "b75", kind: "text", expr: "b.behind > 0 ? ' \xE2\x86\x93' + b.behind : ''", deps: ["b"] }
];
export {
  bindings,
  view_default2 as default
};
