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
  const __renderWhileLoading_0 = () => {
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
  };
  const __renderWhileError_0 = (err) => {
    const __childScope23 = Object.create($scope2);
    __childScope23.err = err;
    const __frag21 = document.createDocumentFragment();
    const __c22 = [];
    const $scope2 = __childScope23;
    const __t24 = document.createTextNode("\n    ");
    __frag21.appendChild(__t24);
    const __e25 = document.createElement("p");
    __e25.setAttribute("class", "git-error");
    __e25.setAttribute("data-test", "error");
    const __t26 = document.createTextNode("Failed to load: ");
    __e25.appendChild(__t26);
    const __t27 = document.createTextNode("");
    __e25.appendChild(__t27);
    __c22.push(bindText(__t27, () => __E[0]($scope2)));
    __frag21.appendChild(__e25);
    const __t28 = document.createTextNode("\n  ");
    __frag21.appendChild(__t28);
    return { nodes: Array.from(__frag21.childNodes), cleanup: () => {
      for (const c of __c22) try {
        c();
      } catch (e) {
      }
    } };
  };
  const __wireBoundary_b26 = (anchor) => bindFor({
    anchor,
    items: () => __E[22]($scope),
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return __E[23](__child);
    },
    render: (c, __i) => {
      const __childScope138 = Object.create($scope2);
      __childScope138.c = c;
      __childScope138.__i = __i;
      const __frag136 = document.createDocumentFragment();
      const __c137 = [];
      const $scope2 = __childScope138;
      const __t139 = document.createTextNode("\n                    ");
      __frag136.appendChild(__t139);
      const __e140 = document.createElement("div");
      __e140.setAttribute("class", "git-row");
      const __t141 = document.createTextNode("\n                      ");
      __e140.appendChild(__t141);
      const __e142 = document.createElement("sui-avatar");
      __e142.setAttribute("size", "sm");
      __e142.setAttribute("shape", "round");
      __c137.push(bindAttr(__e142, "name", () => __E[24]($scope2)));
      __e140.appendChild(__e142);
      const __t143 = document.createTextNode("\n                      ");
      __e140.appendChild(__t143);
      const __e144 = document.createElement("div");
      __e144.setAttribute("class", "git-row-body");
      const __t145 = document.createTextNode("\n                        ");
      __e144.appendChild(__t145);
      const __e146 = document.createElement("div");
      __e146.setAttribute("class", "git-row-title");
      const __t147 = document.createTextNode("");
      __e146.appendChild(__t147);
      __c137.push(bindText(__t147, () => __E[25]($scope2)));
      __e144.appendChild(__e146);
      const __t148 = document.createTextNode("\n                        ");
      __e144.appendChild(__t148);
      const __e149 = document.createElement("div");
      __e149.setAttribute("class", "git-row-sub");
      const __t150 = document.createTextNode("");
      __e149.appendChild(__t150);
      __c137.push(bindText(__t150, () => __E[26]($scope2)));
      const __t151 = document.createTextNode(" &middot; ");
      __e149.appendChild(__t151);
      const __t152 = document.createTextNode("");
      __e149.appendChild(__t152);
      __c137.push(bindText(__t152, () => __E[24]($scope2)));
      const __t153 = document.createTextNode(" &middot; ");
      __e149.appendChild(__t153);
      const __t154 = document.createTextNode("");
      __e149.appendChild(__t154);
      __c137.push(bindText(__t154, () => __E[27]($scope2)));
      __e144.appendChild(__e149);
      const __t155 = document.createTextNode("\n                      ");
      __e144.appendChild(__t155);
      __e140.appendChild(__e144);
      const __t156 = document.createTextNode("\n                      ");
      __e140.appendChild(__t156);
      const __e157 = document.createElement("div");
      __e157.setAttribute("class", "git-deltas");
      const __t158 = document.createTextNode("\n                        ");
      __e157.appendChild(__t158);
      const __e159 = document.createElement("span");
      __e159.setAttribute("class", "git-add");
      const __t160 = document.createTextNode("+");
      __e159.appendChild(__t160);
      const __t161 = document.createTextNode("");
      __e159.appendChild(__t161);
      __c137.push(bindText(__t161, () => __E[28]($scope2)));
      __e157.appendChild(__e159);
      const __t162 = document.createTextNode("\n                        ");
      __e157.appendChild(__t162);
      const __e163 = document.createElement("span");
      __e163.setAttribute("class", "git-del");
      const __t164 = document.createTextNode("&minus;");
      __e163.appendChild(__t164);
      const __t165 = document.createTextNode("");
      __e163.appendChild(__t165);
      __c137.push(bindText(__t165, () => __E[29]($scope2)));
      __e157.appendChild(__e163);
      const __t166 = document.createTextNode("\n                      ");
      __e157.appendChild(__t166);
      __e140.appendChild(__e157);
      const __t167 = document.createTextNode("\n                    ");
      __e140.appendChild(__t167);
      __frag136.appendChild(__e140);
      const __t168 = document.createTextNode("\n                  ");
      __frag136.appendChild(__t168);
      return { nodes: Array.from(__frag136.childNodes), cleanup: () => {
        for (const c2 of __c137) try {
          c2();
        } catch (e) {
        }
      } };
    }
  });
  const __wireBoundary_b25 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (__E[21]($scope)) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag126 = document.createDocumentFragment();
        const __c127 = [];
        const __t128 = document.createTextNode("\n                  ");
        __frag126.appendChild(__t128);
        const __e129 = document.createElement("p");
        __e129.setAttribute("class", "git-empty");
        const __t130 = document.createTextNode("No commits yet.");
        __e129.appendChild(__t130);
        __frag126.appendChild(__e129);
        const __t131 = document.createTextNode("\n                ");
        __frag126.appendChild(__t131);
        return { nodes: Array.from(__frag126.childNodes), cleanup: () => {
          for (const c of __c127) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag132 = document.createDocumentFragment();
        const __c133 = [];
        const __t134 = document.createTextNode("\n                  ");
        __frag132.appendChild(__t134);
        const __forAnchor135 = document.createComment("for");
        __frag132.appendChild(__forAnchor135);
        __c133.push(__wireBoundary_b26(__forAnchor135));
        const __t169 = document.createTextNode("\n                ");
        __frag132.appendChild(__t169);
        return { nodes: Array.from(__frag132.childNodes), cleanup: () => {
          for (const c of __c133) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b35 = (anchor) => bindFor({
    anchor,
    items: () => __E[31]($scope),
    key: (b, __i) => {
      const __child = Object.create($scope);
      __child.b = b;
      __child.__i = __i;
      return __E[32](__child);
    },
    render: (b, __i) => {
      const __childScope190 = Object.create($scope2);
      __childScope190.b = b;
      __childScope190.__i = __i;
      const __frag188 = document.createDocumentFragment();
      const __c189 = [];
      const $scope2 = __childScope190;
      const __t191 = document.createTextNode("\n                    ");
      __frag188.appendChild(__t191);
      const __e192 = document.createElement("div");
      __e192.setAttribute("class", "git-row");
      const __t193 = document.createTextNode("\n                      ");
      __e192.appendChild(__t193);
      const __e194 = document.createElement("ship-icon");
      __e194.setAttribute("name", "git-branch");
      __e194.setAttribute("size", "xs");
      __c189.push(bindAttr(__e194, "class", () => __E[33]($scope2)));
      __e192.appendChild(__e194);
      const __t195 = document.createTextNode("\n                      ");
      __e192.appendChild(__t195);
      const __e196 = document.createElement("div");
      __e196.setAttribute("class", "git-row-body");
      const __t197 = document.createTextNode("\n                        ");
      __e196.appendChild(__t197);
      const __e198 = document.createElement("div");
      __e198.setAttribute("class", "git-row-title git-mono");
      const __t199 = document.createTextNode("");
      __e198.appendChild(__t199);
      __c189.push(bindText(__t199, () => __E[32]($scope2)));
      __e196.appendChild(__e198);
      const __t200 = document.createTextNode("\n                        ");
      __e196.appendChild(__t200);
      const __e201 = document.createElement("div");
      __e201.setAttribute("class", "git-row-sub");
      const __t202 = document.createTextNode("");
      __e201.appendChild(__t202);
      __c189.push(bindText(__t202, () => __E[34]($scope2)));
      __e196.appendChild(__e201);
      const __t203 = document.createTextNode("\n                      ");
      __e196.appendChild(__t203);
      __e192.appendChild(__e196);
      const __t204 = document.createTextNode("\n                      ");
      __e192.appendChild(__t204);
      const __e205 = document.createElement("div");
      __e205.setAttribute("class", "git-deltas");
      const __t206 = document.createTextNode("\n                        ");
      __e205.appendChild(__t206);
      const __t207 = document.createTextNode("");
      __e205.appendChild(__t207);
      __c189.push(bindText(__t207, () => __E[35]($scope2)));
      const __t208 = document.createTextNode("\n                        ");
      __e205.appendChild(__t208);
      const __t209 = document.createTextNode("");
      __e205.appendChild(__t209);
      __c189.push(bindText(__t209, () => __E[36]($scope2)));
      const __t210 = document.createTextNode("\n                      ");
      __e205.appendChild(__t210);
      __e192.appendChild(__e205);
      const __t211 = document.createTextNode("\n                    ");
      __e192.appendChild(__t211);
      __frag188.appendChild(__e192);
      const __t212 = document.createTextNode("\n                  ");
      __frag188.appendChild(__t212);
      return { nodes: Array.from(__frag188.childNodes), cleanup: () => {
        for (const c of __c189) try {
          c();
        } catch (e) {
        }
      } };
    }
  });
  const __wireBoundary_b34 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (__E[30]($scope)) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag178 = document.createDocumentFragment();
        const __c179 = [];
        const __t180 = document.createTextNode("\n                  ");
        __frag178.appendChild(__t180);
        const __e181 = document.createElement("p");
        __e181.setAttribute("class", "git-empty");
        const __t182 = document.createTextNode("No branches.");
        __e181.appendChild(__t182);
        __frag178.appendChild(__e181);
        const __t183 = document.createTextNode("\n                ");
        __frag178.appendChild(__t183);
        return { nodes: Array.from(__frag178.childNodes), cleanup: () => {
          for (const c of __c179) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag184 = document.createDocumentFragment();
        const __c185 = [];
        const __t186 = document.createTextNode("\n                  ");
        __frag184.appendChild(__t186);
        const __forAnchor187 = document.createComment("for");
        __frag184.appendChild(__forAnchor187);
        __c185.push(__wireBoundary_b35(__forAnchor187));
        const __t213 = document.createTextNode("\n                ");
        __frag184.appendChild(__t213);
        return { nodes: Array.from(__frag184.childNodes), cleanup: () => {
          for (const c of __c185) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b43 = (anchor) => bindFor({
    anchor,
    items: () => __E[39]($scope),
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return __E[40](__child);
    },
    render: (f, __i) => {
      const __childScope237 = Object.create($scope2);
      __childScope237.f = f;
      __childScope237.__i = __i;
      const __frag235 = document.createDocumentFragment();
      const __c236 = [];
      const $scope2 = __childScope237;
      const __t238 = document.createTextNode("\n                  ");
      __frag235.appendChild(__t238);
      const __e239 = document.createElement("div");
      __e239.setAttribute("class", "git-fileline git-mono");
      const __t240 = document.createTextNode("");
      __e239.appendChild(__t240);
      __c236.push(bindText(__t240, () => __E[40]($scope2)));
      __frag235.appendChild(__e239);
      const __t241 = document.createTextNode("\n                ");
      __frag235.appendChild(__t241);
      return { nodes: Array.from(__frag235.childNodes), cleanup: () => {
        for (const c of __c236) try {
          c();
        } catch (e) {
        }
      } };
    }
  });
  const __wireBoundary_b41 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (__E[37]($scope)) return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag224 = document.createDocumentFragment();
        const __c225 = [];
        const __t226 = document.createTextNode("\n              ");
        __frag224.appendChild(__t226);
        const __e227 = document.createElement("section");
        __e227.setAttribute("class", "git-filegroup");
        const __t228 = document.createTextNode("\n                ");
        __e227.appendChild(__t228);
        const __e229 = document.createElement("h3");
        const __t230 = document.createTextNode("Staged (");
        __e229.appendChild(__t230);
        const __t231 = document.createTextNode("");
        __e229.appendChild(__t231);
        __c225.push(bindText(__t231, () => __E[38]($scope)));
        const __t232 = document.createTextNode(")");
        __e229.appendChild(__t232);
        __e227.appendChild(__e229);
        const __t233 = document.createTextNode("\n                ");
        __e227.appendChild(__t233);
        const __forAnchor234 = document.createComment("for");
        __e227.appendChild(__forAnchor234);
        __c225.push(__wireBoundary_b43(__forAnchor234));
        const __t242 = document.createTextNode("\n              ");
        __e227.appendChild(__t242);
        __frag224.appendChild(__e227);
        const __t243 = document.createTextNode("\n            ");
        __frag224.appendChild(__t243);
        return { nodes: Array.from(__frag224.childNodes), cleanup: () => {
          for (const c of __c225) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b47 = (anchor) => bindFor({
    anchor,
    items: () => __E[43]($scope),
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return __E[40](__child);
    },
    render: (f, __i) => {
      const __childScope259 = Object.create($scope2);
      __childScope259.f = f;
      __childScope259.__i = __i;
      const __frag257 = document.createDocumentFragment();
      const __c258 = [];
      const $scope2 = __childScope259;
      const __t260 = document.createTextNode("\n                  ");
      __frag257.appendChild(__t260);
      const __e261 = document.createElement("div");
      __e261.setAttribute("class", "git-fileline git-mono");
      const __t262 = document.createTextNode("");
      __e261.appendChild(__t262);
      __c258.push(bindText(__t262, () => __E[40]($scope2)));
      __frag257.appendChild(__e261);
      const __t263 = document.createTextNode("\n                ");
      __frag257.appendChild(__t263);
      return { nodes: Array.from(__frag257.childNodes), cleanup: () => {
        for (const c of __c258) try {
          c();
        } catch (e) {
        }
      } };
    }
  });
  const __wireBoundary_b45 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (__E[41]($scope)) return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag246 = document.createDocumentFragment();
        const __c247 = [];
        const __t248 = document.createTextNode("\n              ");
        __frag246.appendChild(__t248);
        const __e249 = document.createElement("section");
        __e249.setAttribute("class", "git-filegroup");
        const __t250 = document.createTextNode("\n                ");
        __e249.appendChild(__t250);
        const __e251 = document.createElement("h3");
        const __t252 = document.createTextNode("Modified (");
        __e251.appendChild(__t252);
        const __t253 = document.createTextNode("");
        __e251.appendChild(__t253);
        __c247.push(bindText(__t253, () => __E[42]($scope)));
        const __t254 = document.createTextNode(")");
        __e251.appendChild(__t254);
        __e249.appendChild(__e251);
        const __t255 = document.createTextNode("\n                ");
        __e249.appendChild(__t255);
        const __forAnchor256 = document.createComment("for");
        __e249.appendChild(__forAnchor256);
        __c247.push(__wireBoundary_b47(__forAnchor256));
        const __t264 = document.createTextNode("\n              ");
        __e249.appendChild(__t264);
        __frag246.appendChild(__e249);
        const __t265 = document.createTextNode("\n            ");
        __frag246.appendChild(__t265);
        return { nodes: Array.from(__frag246.childNodes), cleanup: () => {
          for (const c of __c247) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b51 = (anchor) => bindFor({
    anchor,
    items: () => __E[46]($scope),
    key: (f, __i) => {
      const __child = Object.create($scope);
      __child.f = f;
      __child.__i = __i;
      return __E[40](__child);
    },
    render: (f, __i) => {
      const __childScope281 = Object.create($scope2);
      __childScope281.f = f;
      __childScope281.__i = __i;
      const __frag279 = document.createDocumentFragment();
      const __c280 = [];
      const $scope2 = __childScope281;
      const __t282 = document.createTextNode("\n                  ");
      __frag279.appendChild(__t282);
      const __e283 = document.createElement("div");
      __e283.setAttribute("class", "git-fileline git-mono");
      const __t284 = document.createTextNode("");
      __e283.appendChild(__t284);
      __c280.push(bindText(__t284, () => __E[40]($scope2)));
      __frag279.appendChild(__e283);
      const __t285 = document.createTextNode("\n                ");
      __frag279.appendChild(__t285);
      return { nodes: Array.from(__frag279.childNodes), cleanup: () => {
        for (const c of __c280) try {
          c();
        } catch (e) {
        }
      } };
    }
  });
  const __wireBoundary_b49 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (__E[44]($scope)) return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag268 = document.createDocumentFragment();
        const __c269 = [];
        const __t270 = document.createTextNode("\n              ");
        __frag268.appendChild(__t270);
        const __e271 = document.createElement("section");
        __e271.setAttribute("class", "git-filegroup");
        const __t272 = document.createTextNode("\n                ");
        __e271.appendChild(__t272);
        const __e273 = document.createElement("h3");
        const __t274 = document.createTextNode("Untracked (");
        __e273.appendChild(__t274);
        const __t275 = document.createTextNode("");
        __e273.appendChild(__t275);
        __c269.push(bindText(__t275, () => __E[45]($scope)));
        const __t276 = document.createTextNode(")");
        __e273.appendChild(__t276);
        __e271.appendChild(__e273);
        const __t277 = document.createTextNode("\n                ");
        __e271.appendChild(__t277);
        const __forAnchor278 = document.createComment("for");
        __e271.appendChild(__forAnchor278);
        __c269.push(__wireBoundary_b51(__forAnchor278));
        const __t286 = document.createTextNode("\n              ");
        __e271.appendChild(__t286);
        __frag268.appendChild(__e271);
        const __t287 = document.createTextNode("\n            ");
        __frag268.appendChild(__t287);
        return { nodes: Array.from(__frag268.childNodes), cleanup: () => {
          for (const c of __c269) try {
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
      if (__E[17]($scope)) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag87 = document.createDocumentFragment();
        const __c88 = [];
        const __t89 = document.createTextNode("\n          ");
        __frag87.appendChild(__t89);
        const __e90 = document.createElement("div");
        __e90.setAttribute("class", "git-clean");
        const __t91 = document.createTextNode("\n            ");
        __e90.appendChild(__t91);
        const __e92 = document.createElement("div");
        __e92.setAttribute("class", "git-hero");
        const __t93 = document.createTextNode("\n              ");
        __e92.appendChild(__t93);
        const __e94 = document.createElement("div");
        __e94.setAttribute("class", "git-halo");
        const __t95 = document.createTextNode("\n                ");
        __e94.appendChild(__t95);
        const __e96 = document.createElement("ship-icon");
        __e96.setAttribute("name", "check");
        __e96.setAttribute("size", "md");
        __e94.appendChild(__e96);
        const __t97 = document.createTextNode("\n              ");
        __e94.appendChild(__t97);
        __e92.appendChild(__e94);
        const __t98 = document.createTextNode("\n              ");
        __e92.appendChild(__t98);
        const __e99 = document.createElement("div");
        const __t100 = document.createTextNode("\n                ");
        __e99.appendChild(__t100);
        const __e101 = document.createElement("h2");
        __e101.setAttribute("class", "git-hero-title");
        const __t102 = document.createTextNode("Working tree is clean.");
        __e101.appendChild(__t102);
        __e99.appendChild(__e101);
        const __t103 = document.createTextNode("\n                ");
        __e99.appendChild(__t103);
        const __e104 = document.createElement("p");
        __e104.setAttribute("class", "git-hero-sub");
        const __t105 = document.createTextNode("\n                  Nothing to commit on ");
        __e104.appendChild(__t105);
        const __e106 = document.createElement("span");
        __e106.setAttribute("class", "git-mono");
        const __t107 = document.createTextNode("");
        __e106.appendChild(__t107);
        __c88.push(bindText(__t107, () => __E[2]($scope)));
        __e104.appendChild(__e106);
        const __t108 = document.createTextNode(".\n                  ");
        __e104.appendChild(__t108);
        const __t109 = document.createTextNode("");
        __e104.appendChild(__t109);
        __c88.push(bindText(__t109, () => __E[18]($scope)));
        const __t110 = document.createTextNode("\n                  ");
        __e104.appendChild(__t110);
        const __t111 = document.createTextNode("");
        __e104.appendChild(__t111);
        __c88.push(bindText(__t111, () => __E[19]($scope)));
        const __t112 = document.createTextNode("\n                  ");
        __e104.appendChild(__t112);
        const __t113 = document.createTextNode("");
        __e104.appendChild(__t113);
        __c88.push(bindText(__t113, () => __E[20]($scope)));
        const __t114 = document.createTextNode("\n                ");
        __e104.appendChild(__t114);
        __e99.appendChild(__e104);
        const __t115 = document.createTextNode("\n              ");
        __e99.appendChild(__t115);
        __e92.appendChild(__e99);
        const __t116 = document.createTextNode("\n            ");
        __e92.appendChild(__t116);
        __e90.appendChild(__e92);
        const __t117 = document.createTextNode("\n\n            ");
        __e90.appendChild(__t117);
        const __e118 = document.createElement("div");
        __e118.setAttribute("class", "git-two-col");
        const __t119 = document.createTextNode("\n              ");
        __e118.appendChild(__t119);
        const __e120 = document.createElement("section");
        __e120.setAttribute("class", "git-sec");
        const __t121 = document.createTextNode("\n                ");
        __e120.appendChild(__t121);
        const __e122 = document.createElement("h3");
        const __t123 = document.createTextNode("Recent on this branch");
        __e122.appendChild(__t123);
        __e120.appendChild(__e122);
        const __t124 = document.createTextNode("\n                ");
        __e120.appendChild(__t124);
        const __ifAnchor125 = document.createComment("if");
        __e120.appendChild(__ifAnchor125);
        __c88.push(__wireBoundary_b25(__ifAnchor125));
        const __t170 = document.createTextNode("\n              ");
        __e120.appendChild(__t170);
        __e118.appendChild(__e120);
        const __t171 = document.createTextNode("\n\n              ");
        __e118.appendChild(__t171);
        const __e172 = document.createElement("section");
        __e172.setAttribute("class", "git-sec");
        const __t173 = document.createTextNode("\n                ");
        __e172.appendChild(__t173);
        const __e174 = document.createElement("h3");
        const __t175 = document.createTextNode("Branches");
        __e174.appendChild(__t175);
        __e172.appendChild(__e174);
        const __t176 = document.createTextNode("\n                ");
        __e172.appendChild(__t176);
        const __ifAnchor177 = document.createComment("if");
        __e172.appendChild(__ifAnchor177);
        __c88.push(__wireBoundary_b34(__ifAnchor177));
        const __t214 = document.createTextNode("\n              ");
        __e172.appendChild(__t214);
        __e118.appendChild(__e172);
        const __t215 = document.createTextNode("\n            ");
        __e118.appendChild(__t215);
        __e90.appendChild(__e118);
        const __t216 = document.createTextNode("\n          ");
        __e90.appendChild(__t216);
        __frag87.appendChild(__e90);
        const __t217 = document.createTextNode("\n        ");
        __frag87.appendChild(__t217);
        return { nodes: Array.from(__frag87.childNodes), cleanup: () => {
          for (const c of __c88) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag218 = document.createDocumentFragment();
        const __c219 = [];
        const __t220 = document.createTextNode("\n          ");
        __frag218.appendChild(__t220);
        const __e221 = document.createElement("div");
        __e221.setAttribute("class", "git-changes");
        const __t222 = document.createTextNode("\n            ");
        __e221.appendChild(__t222);
        const __ifAnchor223 = document.createComment("if");
        __e221.appendChild(__ifAnchor223);
        __c219.push(__wireBoundary_b41(__ifAnchor223));
        const __t244 = document.createTextNode("\n            ");
        __e221.appendChild(__t244);
        const __ifAnchor245 = document.createComment("if");
        __e221.appendChild(__ifAnchor245);
        __c219.push(__wireBoundary_b45(__ifAnchor245));
        const __t266 = document.createTextNode("\n            ");
        __e221.appendChild(__t266);
        const __ifAnchor267 = document.createComment("if");
        __e221.appendChild(__ifAnchor267);
        __c219.push(__wireBoundary_b49(__ifAnchor267));
        const __t288 = document.createTextNode("\n          ");
        __e221.appendChild(__t288);
        __frag218.appendChild(__e221);
        const __t289 = document.createTextNode("\n        ");
        __frag218.appendChild(__t289);
        return { nodes: Array.from(__frag218.childNodes), cleanup: () => {
          for (const c of __c219) try {
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
      if (__E[8]($scope)) return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag83 = document.createDocumentFragment();
        const __c84 = [];
        const __t85 = document.createTextNode("\n        ");
        __frag83.appendChild(__t85);
        const __ifAnchor86 = document.createComment("if");
        __frag83.appendChild(__ifAnchor86);
        __c84.push(__wireBoundary_b20(__ifAnchor86));
        const __t290 = document.createTextNode("\n      ");
        __frag83.appendChild(__t290);
        return { nodes: Array.from(__frag83.childNodes), cleanup: () => {
          for (const c of __c84) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b59 = (anchor) => bindFor({
    anchor,
    items: () => __E[52]($scope),
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return __E[23](__child);
    },
    render: (c, __i) => {
      const __childScope326 = Object.create($scope2);
      __childScope326.c = c;
      __childScope326.__i = __i;
      const __frag324 = document.createDocumentFragment();
      const __c325 = [];
      const $scope2 = __childScope326;
      const __t327 = document.createTextNode("\n                ");
      __frag324.appendChild(__t327);
      const __e328 = document.createElement("div");
      __e328.setAttribute("class", "git-history-row");
      const __t329 = document.createTextNode("\n                  ");
      __e328.appendChild(__t329);
      const __e330 = document.createElement("sui-avatar");
      __e330.setAttribute("size", "sm");
      __e330.setAttribute("shape", "round");
      __c325.push(bindAttr(__e330, "name", () => __E[24]($scope2)));
      __e328.appendChild(__e330);
      const __t331 = document.createTextNode("\n                  ");
      __e328.appendChild(__t331);
      const __e332 = document.createElement("div");
      __e332.setAttribute("class", "git-row-body");
      const __t333 = document.createTextNode("\n                    ");
      __e332.appendChild(__t333);
      const __e334 = document.createElement("div");
      __e334.setAttribute("class", "git-row-title");
      const __t335 = document.createTextNode("");
      __e334.appendChild(__t335);
      __c325.push(bindText(__t335, () => __E[25]($scope2)));
      __e332.appendChild(__e334);
      const __t336 = document.createTextNode("\n                    ");
      __e332.appendChild(__t336);
      const __e337 = document.createElement("div");
      __e337.setAttribute("class", "git-row-sub git-mono");
      const __t338 = document.createTextNode("");
      __e337.appendChild(__t338);
      __c325.push(bindText(__t338, () => __E[26]($scope2)));
      const __t339 = document.createTextNode(" &middot; ");
      __e337.appendChild(__t339);
      const __t340 = document.createTextNode("");
      __e337.appendChild(__t340);
      __c325.push(bindText(__t340, () => __E[24]($scope2)));
      const __t341 = document.createTextNode(" &middot; ");
      __e337.appendChild(__t341);
      const __t342 = document.createTextNode("");
      __e337.appendChild(__t342);
      __c325.push(bindText(__t342, () => __E[27]($scope2)));
      __e332.appendChild(__e337);
      const __t343 = document.createTextNode("\n                  ");
      __e332.appendChild(__t343);
      __e328.appendChild(__e332);
      const __t344 = document.createTextNode("\n                  ");
      __e328.appendChild(__t344);
      const __e345 = document.createElement("div");
      __e345.setAttribute("class", "git-deltas");
      const __t346 = document.createTextNode("\n                    ");
      __e345.appendChild(__t346);
      const __e347 = document.createElement("span");
      __e347.setAttribute("class", "git-add");
      const __t348 = document.createTextNode("+");
      __e347.appendChild(__t348);
      const __t349 = document.createTextNode("");
      __e347.appendChild(__t349);
      __c325.push(bindText(__t349, () => __E[28]($scope2)));
      __e345.appendChild(__e347);
      const __t350 = document.createTextNode("\n                    ");
      __e345.appendChild(__t350);
      const __e351 = document.createElement("span");
      __e351.setAttribute("class", "git-del");
      const __t352 = document.createTextNode("&minus;");
      __e351.appendChild(__t352);
      const __t353 = document.createTextNode("");
      __e351.appendChild(__t353);
      __c325.push(bindText(__t353, () => __E[29]($scope2)));
      __e345.appendChild(__e351);
      const __t354 = document.createTextNode("\n                  ");
      __e345.appendChild(__t354);
      __e328.appendChild(__e345);
      const __t355 = document.createTextNode("\n                ");
      __e328.appendChild(__t355);
      __frag324.appendChild(__e328);
      const __t356 = document.createTextNode("\n              ");
      __frag324.appendChild(__t356);
      return { nodes: Array.from(__frag324.childNodes), cleanup: () => {
        for (const c2 of __c325) try {
          c2();
        } catch (e) {
        }
      } };
    }
  });
  const __wireBoundary_b57 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (__E[50]($scope)) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag309 = document.createDocumentFragment();
        const __c310 = [];
        const __t311 = document.createTextNode("\n            ");
        __frag309.appendChild(__t311);
        const __e312 = document.createElement("sui-empty-state");
        __e312.setAttribute("tone", "muted");
        __e312.setAttribute("size", "md");
        const __t313 = document.createTextNode("\n              ");
        __e312.appendChild(__t313);
        const __e314 = document.createElement("span");
        __e314.setAttribute("slot", "title");
        const __t315 = document.createTextNode("");
        __e314.appendChild(__t315);
        __c310.push(bindText(__t315, () => __E[51]($scope)));
        __e312.appendChild(__e314);
        const __t316 = document.createTextNode("\n            ");
        __e312.appendChild(__t316);
        __frag309.appendChild(__e312);
        const __t317 = document.createTextNode("\n          ");
        __frag309.appendChild(__t317);
        return { nodes: Array.from(__frag309.childNodes), cleanup: () => {
          for (const c of __c310) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag318 = document.createDocumentFragment();
        const __c319 = [];
        const __t320 = document.createTextNode("\n            ");
        __frag318.appendChild(__t320);
        const __e321 = document.createElement("div");
        __e321.setAttribute("class", "git-history-rows");
        const __t322 = document.createTextNode("\n              ");
        __e321.appendChild(__t322);
        const __forAnchor323 = document.createComment("for");
        __e321.appendChild(__forAnchor323);
        __c319.push(__wireBoundary_b59(__forAnchor323));
        const __t357 = document.createTextNode("\n            ");
        __e321.appendChild(__t357);
        __frag318.appendChild(__e321);
        const __t358 = document.createTextNode("\n          ");
        __frag318.appendChild(__t358);
        return { nodes: Array.from(__frag318.childNodes), cleanup: () => {
          for (const c of __c319) try {
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
      if (__E[11]($scope)) return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag293 = document.createDocumentFragment();
        const __c294 = [];
        const __t295 = document.createTextNode("\n        ");
        __frag293.appendChild(__t295);
        const __e296 = document.createElement("div");
        __e296.setAttribute("class", "git-history");
        const __t297 = document.createTextNode("\n          ");
        __e296.appendChild(__t297);
        const __e298 = document.createElement("div");
        __e298.setAttribute("class", "git-history-search");
        const __t299 = document.createTextNode("\n            ");
        __e298.appendChild(__t299);
        const __e300 = document.createElement("input");
        __e300.setAttribute("class", "git-search");
        __e300.setAttribute("placeholder", "Filter by subject, sha, or author");
        __c294.push(bindValue(__e300, __E[47]($scope)));
        __e298.appendChild(__e300);
        const __t301 = document.createTextNode("\n            ");
        __e298.appendChild(__t301);
        const __e302 = document.createElement("span");
        __e302.setAttribute("class", "git-history-count");
        const __t303 = document.createTextNode("");
        __e302.appendChild(__t303);
        __c294.push(bindText(__t303, () => __E[48]($scope)));
        const __t304 = document.createTextNode(" of ");
        __e302.appendChild(__t304);
        const __t305 = document.createTextNode("");
        __e302.appendChild(__t305);
        __c294.push(bindText(__t305, () => __E[49]($scope)));
        __e298.appendChild(__e302);
        const __t306 = document.createTextNode("\n          ");
        __e298.appendChild(__t306);
        __e296.appendChild(__e298);
        const __t307 = document.createTextNode("\n          ");
        __e296.appendChild(__t307);
        const __ifAnchor308 = document.createComment("if");
        __e296.appendChild(__ifAnchor308);
        __c294.push(__wireBoundary_b57(__ifAnchor308));
        const __t359 = document.createTextNode("\n        ");
        __e296.appendChild(__t359);
        __frag293.appendChild(__e296);
        const __t360 = document.createTextNode("\n      ");
        __frag293.appendChild(__t360);
        return { nodes: Array.from(__frag293.childNodes), cleanup: () => {
          for (const c of __c294) try {
            c();
          } catch (e) {
          }
        } };
      }
    ]
  });
  const __wireBoundary_b69 = (anchor) => bindFor({
    anchor,
    items: () => __E[54]($scope),
    key: (b, __i) => {
      const __child = Object.create($scope);
      __child.b = b;
      __child.__i = __i;
      return __E[32](__child);
    },
    render: (b, __i) => {
      const __childScope386 = Object.create($scope2);
      __childScope386.b = b;
      __childScope386.__i = __i;
      const __frag384 = document.createDocumentFragment();
      const __c385 = [];
      const $scope2 = __childScope386;
      const __t387 = document.createTextNode("\n                ");
      __frag384.appendChild(__t387);
      const __e388 = document.createElement("div");
      __e388.setAttribute("class", "git-history-row");
      const __t389 = document.createTextNode("\n                  ");
      __e388.appendChild(__t389);
      const __e390 = document.createElement("ship-icon");
      __e390.setAttribute("name", "git-branch");
      __e390.setAttribute("size", "sm");
      __c385.push(bindAttr(__e390, "class", () => __E[33]($scope2)));
      __e388.appendChild(__e390);
      const __t391 = document.createTextNode("\n                  ");
      __e388.appendChild(__t391);
      const __e392 = document.createElement("div");
      __e392.setAttribute("class", "git-row-body");
      const __t393 = document.createTextNode("\n                    ");
      __e392.appendChild(__t393);
      const __e394 = document.createElement("div");
      __e394.setAttribute("class", "git-row-title git-mono");
      const __t395 = document.createTextNode("");
      __e394.appendChild(__t395);
      __c385.push(bindText(__t395, () => __E[32]($scope2)));
      const __t396 = document.createTextNode("");
      __e394.appendChild(__t396);
      __c385.push(bindText(__t396, () => __E[55]($scope2)));
      __e392.appendChild(__e394);
      const __t397 = document.createTextNode("\n                    ");
      __e392.appendChild(__t397);
      const __e398 = document.createElement("div");
      __e398.setAttribute("class", "git-row-sub");
      const __t399 = document.createTextNode("");
      __e398.appendChild(__t399);
      __c385.push(bindText(__t399, () => __E[34]($scope2)));
      __e392.appendChild(__e398);
      const __t400 = document.createTextNode("\n                  ");
      __e392.appendChild(__t400);
      __e388.appendChild(__e392);
      const __t401 = document.createTextNode("\n                  ");
      __e388.appendChild(__t401);
      const __e402 = document.createElement("div");
      __e402.setAttribute("class", "git-deltas");
      const __t403 = document.createTextNode("\n                    ");
      __e402.appendChild(__t403);
      const __t404 = document.createTextNode("");
      __e402.appendChild(__t404);
      __c385.push(bindText(__t404, () => __E[35]($scope2)));
      const __t405 = document.createTextNode("\n                    ");
      __e402.appendChild(__t405);
      const __t406 = document.createTextNode("");
      __e402.appendChild(__t406);
      __c385.push(bindText(__t406, () => __E[36]($scope2)));
      const __t407 = document.createTextNode("\n                  ");
      __e402.appendChild(__t407);
      __e388.appendChild(__e402);
      const __t408 = document.createTextNode("\n                ");
      __e388.appendChild(__t408);
      __frag384.appendChild(__e388);
      const __t409 = document.createTextNode("\n              ");
      __frag384.appendChild(__t409);
      return { nodes: Array.from(__frag384.childNodes), cleanup: () => {
        for (const c of __c385) try {
          c();
        } catch (e) {
        }
      } };
    }
  });
  const __wireBoundary_b68 = (anchor) => bindIf({
    anchor,
    test: () => {
      if (__E[53]($scope)) return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag369 = document.createDocumentFragment();
        const __c370 = [];
        const __t371 = document.createTextNode("\n            ");
        __frag369.appendChild(__t371);
        const __e372 = document.createElement("sui-empty-state");
        __e372.setAttribute("tone", "muted");
        __e372.setAttribute("size", "md");
        const __t373 = document.createTextNode("\n              ");
        __e372.appendChild(__t373);
        const __e374 = document.createElement("span");
        __e374.setAttribute("slot", "title");
        const __t375 = document.createTextNode("No branches");
        __e374.appendChild(__t375);
        __e372.appendChild(__e374);
        const __t376 = document.createTextNode("\n            ");
        __e372.appendChild(__t376);
        __frag369.appendChild(__e372);
        const __t377 = document.createTextNode("\n          ");
        __frag369.appendChild(__t377);
        return { nodes: Array.from(__frag369.childNodes), cleanup: () => {
          for (const c of __c370) try {
            c();
          } catch (e) {
          }
        } };
      },
      () => {
        const __frag378 = document.createDocumentFragment();
        const __c379 = [];
        const __t380 = document.createTextNode("\n            ");
        __frag378.appendChild(__t380);
        const __e381 = document.createElement("div");
        __e381.setAttribute("class", "git-history-rows");
        const __t382 = document.createTextNode("\n              ");
        __e381.appendChild(__t382);
        const __forAnchor383 = document.createComment("for");
        __e381.appendChild(__forAnchor383);
        __c379.push(__wireBoundary_b69(__forAnchor383));
        const __t410 = document.createTextNode("\n            ");
        __e381.appendChild(__t410);
        __frag378.appendChild(__e381);
        const __t411 = document.createTextNode("\n          ");
        __frag378.appendChild(__t411);
        return { nodes: Array.from(__frag378.childNodes), cleanup: () => {
          for (const c of __c379) try {
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
      if (__E[14]($scope)) return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag363 = document.createDocumentFragment();
        const __c364 = [];
        const __t365 = document.createTextNode("\n        ");
        __frag363.appendChild(__t365);
        const __e366 = document.createElement("div");
        __e366.setAttribute("class", "git-branches");
        const __t367 = document.createTextNode("\n          ");
        __e366.appendChild(__t367);
        const __ifAnchor368 = document.createComment("if");
        __e366.appendChild(__ifAnchor368);
        __c364.push(__wireBoundary_b68(__ifAnchor368));
        const __t412 = document.createTextNode("\n        ");
        __e366.appendChild(__t412);
        __frag363.appendChild(__e366);
        const __t413 = document.createTextNode("\n      ");
        __frag363.appendChild(__t413);
        return { nodes: Array.from(__frag363.childNodes), cleanup: () => {
          for (const c of __c364) try {
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
      if (__E[1]($scope)) return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag32 = document.createDocumentFragment();
        const __c33 = [];
        const __t34 = document.createTextNode("\n    ");
        __frag32.appendChild(__t34);
        const __e35 = document.createElement("header");
        __e35.setAttribute("class", "git-head");
        const __t36 = document.createTextNode("\n      ");
        __e35.appendChild(__t36);
        const __e37 = document.createElement("div");
        __e37.setAttribute("class", "git-head-titles");
        const __t38 = document.createTextNode("\n        ");
        __e37.appendChild(__t38);
        const __e39 = document.createElement("h1");
        __e39.setAttribute("class", "git-title");
        const __t40 = document.createTextNode("Git");
        __e39.appendChild(__t40);
        __e37.appendChild(__e39);
        const __t41 = document.createTextNode("\n        ");
        __e37.appendChild(__t41);
        const __e42 = document.createElement("p");
        __e42.setAttribute("class", "git-sub");
        const __t43 = document.createTextNode("\n          ");
        __e42.appendChild(__t43);
        const __e44 = document.createElement("span");
        __e44.setAttribute("class", "git-mono");
        const __t45 = document.createTextNode("");
        __e44.appendChild(__t45);
        __c33.push(bindText(__t45, () => __E[2]($scope)));
        __e42.appendChild(__e44);
        const __t46 = document.createTextNode("\n          ");
        __e42.appendChild(__t46);
        const __t47 = document.createTextNode("");
        __e42.appendChild(__t47);
        __c33.push(bindText(__t47, () => __E[3]($scope)));
        const __t48 = document.createTextNode("\n          ");
        __e42.appendChild(__t48);
        const __t49 = document.createTextNode("");
        __e42.appendChild(__t49);
        __c33.push(bindText(__t49, () => __E[4]($scope)));
        const __t50 = document.createTextNode("\n          &middot;\n          ");
        __e42.appendChild(__t50);
        const __e51 = document.createElement("span");
        __e51.setAttribute("class", "git-mono");
        const __t52 = document.createTextNode("");
        __e51.appendChild(__t52);
        __c33.push(bindText(__t52, () => __E[5]($scope)));
        __e42.appendChild(__e51);
        const __t53 = document.createTextNode("\n        ");
        __e42.appendChild(__t53);
        __e37.appendChild(__e42);
        const __t54 = document.createTextNode("\n      ");
        __e37.appendChild(__t54);
        __e35.appendChild(__e37);
        const __t55 = document.createTextNode("\n      ");
        __e35.appendChild(__t55);
        const __e56 = document.createElement("div");
        __e56.setAttribute("class", "git-head-actions");
        const __t57 = document.createTextNode("\n        ");
        __e56.appendChild(__t57);
        const __e58 = document.createElement("sui-chip");
        __c33.push(bindAttr(__e58, "tone", () => __E[6]($scope)));
        __e58.setAttribute("variant", "soft");
        __e58.setAttribute("size", "sm");
        const __t59 = document.createTextNode("\n          ");
        __e58.appendChild(__t59);
        const __t60 = document.createTextNode("");
        __e58.appendChild(__t60);
        __c33.push(bindText(__t60, () => __E[7]($scope)));
        const __t61 = document.createTextNode("\n        ");
        __e58.appendChild(__t61);
        __e56.appendChild(__e58);
        const __t62 = document.createTextNode("\n      ");
        __e56.appendChild(__t62);
        __e35.appendChild(__e56);
        const __t63 = document.createTextNode("\n    ");
        __e35.appendChild(__t63);
        __frag32.appendChild(__e35);
        const __t64 = document.createTextNode("\n\n    ");
        __frag32.appendChild(__t64);
        const __e65 = document.createElement("div");
        __e65.setAttribute("class", "git-tabs-row");
        const __t66 = document.createTextNode("\n      ");
        __e65.appendChild(__t66);
        const __e67 = document.createElement("sui-tabs");
        __e67.setAttribute("variant", "underline");
        const __t68 = document.createTextNode("\n        ");
        __e67.appendChild(__t68);
        const __e69 = document.createElement("sui-tab");
        __e69.setAttribute("name", "changes");
        __c33.push(bindAttr(__e69, "active", () => __E[8]($scope)));
        __c33.push(bindAttr(__e69, "count", () => __E[9]($scope)));
        __c33.push(on(__e69, "click", (function() {
          const __h = __E[10]($scope);
          return typeof __h === "function" ? __h.bind(void 0) : (() => {
          });
        })(), { modifiers: [] }));
        const __t70 = document.createTextNode("Changes");
        __e69.appendChild(__t70);
        __e67.appendChild(__e69);
        const __t71 = document.createTextNode("\n        ");
        __e67.appendChild(__t71);
        const __e72 = document.createElement("sui-tab");
        __e72.setAttribute("name", "history");
        __c33.push(bindAttr(__e72, "active", () => __E[11]($scope)));
        __c33.push(bindAttr(__e72, "count", () => __E[12]($scope)));
        __c33.push(on(__e72, "click", (function() {
          const __h = __E[13]($scope);
          return typeof __h === "function" ? __h.bind(void 0) : (() => {
          });
        })(), { modifiers: [] }));
        const __t73 = document.createTextNode("History");
        __e72.appendChild(__t73);
        __e67.appendChild(__e72);
        const __t74 = document.createTextNode("\n        ");
        __e67.appendChild(__t74);
        const __e75 = document.createElement("sui-tab");
        __e75.setAttribute("name", "branches");
        __c33.push(bindAttr(__e75, "active", () => __E[14]($scope)));
        __c33.push(bindAttr(__e75, "count", () => __E[15]($scope)));
        __c33.push(on(__e75, "click", (function() {
          const __h = __E[16]($scope);
          return typeof __h === "function" ? __h.bind(void 0) : (() => {
          });
        })(), { modifiers: [] }));
        const __t76 = document.createTextNode("Branches");
        __e75.appendChild(__t76);
        __e67.appendChild(__e75);
        const __t77 = document.createTextNode("\n      ");
        __e67.appendChild(__t77);
        __e65.appendChild(__e67);
        const __t78 = document.createTextNode("\n    ");
        __e65.appendChild(__t78);
        __frag32.appendChild(__e65);
        const __t79 = document.createTextNode("\n\n    ");
        __frag32.appendChild(__t79);
        const __e80 = document.createElement("div");
        __e80.setAttribute("class", "git-body");
        const __t81 = document.createTextNode("\n      ");
        __e80.appendChild(__t81);
        const __ifAnchor82 = document.createComment("if");
        __e80.appendChild(__ifAnchor82);
        __c33.push(__wireBoundary_b19(__ifAnchor82));
        const __t291 = document.createTextNode("\n\n      ");
        __e80.appendChild(__t291);
        const __ifAnchor292 = document.createComment("if");
        __e80.appendChild(__ifAnchor292);
        __c33.push(__wireBoundary_b53(__ifAnchor292));
        const __t361 = document.createTextNode("\n\n      ");
        __e80.appendChild(__t361);
        const __ifAnchor362 = document.createComment("if");
        __e80.appendChild(__ifAnchor362);
        __c33.push(__wireBoundary_b67(__ifAnchor362));
        const __t414 = document.createTextNode("\n    ");
        __e80.appendChild(__t414);
        __frag32.appendChild(__e80);
        const __t415 = document.createTextNode("\n  ");
        __frag32.appendChild(__t415);
        return { nodes: Array.from(__frag32.childNodes), cleanup: () => {
          for (const c of __c33) try {
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
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor19, signal: $scope.overview, render: __renderWhileLoading_0 }));
      const __t20 = document.createTextNode("\n\n  ");
      __e11.appendChild(__t20);
      const __weAnchor29 = document.createComment("while-error");
      __e11.appendChild(__weAnchor29);
      __cleanups.push(bindWhileError({ anchor: __weAnchor29, signal: $scope.overview, render: __renderWhileError_0 }));
      const __t30 = document.createTextNode("\n\n  ");
      __e11.appendChild(__t30);
      const __ifAnchor31 = document.createComment("if");
      __e11.appendChild(__ifAnchor31);
      __cleanups.push(__wireBoundary_b3(__ifAnchor31));
      const __t416 = document.createTextNode("\n");
      __e11.appendChild(__t416);
      container.appendChild(__e11);
      const __t417 = document.createTextNode("\n");
      container.appendChild(__t417);
      return () => {
        for (const c of __cleanups) try {
          c();
        } catch (e) {
        }
      };
    },
    "__renderWhileLoading_0": __renderWhileLoading_0,
    "__renderWhileError_0": __renderWhileError_0,
    __b26_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("for") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b26(__anchor);
    },
    __b25_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b25(__anchor);
    },
    __b35_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("for") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b35(__anchor);
    },
    __b34_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b34(__anchor);
    },
    __b43_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("for") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b43(__anchor);
    },
    __b41_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b41(__anchor);
    },
    __b47_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("for") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b47(__anchor);
    },
    __b45_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b45(__anchor);
    },
    __b51_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("for") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b51(__anchor);
    },
    __b49_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b49(__anchor);
    },
    __b20_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b20(__anchor);
    },
    __b19_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b19(__anchor);
    },
    __b59_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("for") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b59(__anchor);
    },
    __b57_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b57(__anchor);
    },
    __b53_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b53(__anchor);
    },
    __b69_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("for") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b69(__anchor);
    },
    __b68_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b68(__anchor);
    },
    __b67_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b67(__anchor);
    },
    __b3_attach: (target) => {
      const __doc = target.ownerDocument ?? document;
      const __anchor = target.nodeType === 1 ? __doc.createComment("if") : target;
      if (target.nodeType === 1 && target.parentNode) target.parentNode.replaceChild(__anchor, target);
      return __wireBoundary_b3(__anchor);
    }
  });
});
var bindings = [
  { key: "b0", kind: "while-loading", expr: "overview", deps: ["overview"], renderKey: "__renderWhileLoading_0" },
  { key: "b1", kind: "while-error", expr: "overview", deps: ["overview"], renderKey: "__renderWhileError_0" },
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
  { key: "b21", kind: "text", expr: "branch()", deps: ["branch"] },
  { key: "b22", kind: "text", expr: `ahead() === 0 && behind() === 0 ? "You're up to date with the upstream." : ''`, deps: ["ahead", "behind"] },
  { key: "b23", kind: "text", expr: "ahead() > 0 ? ahead() + ' commit' + (ahead() === 1 ? '' : 's') + ' ahead of upstream.' : ''", deps: ["ahead"] },
  { key: "b24", kind: "text", expr: "behind() > 0 ? ' ' + behind() + ' commit' + (behind() === 1 ? '' : 's') + ' behind upstream.' : ''", deps: ["behind"] },
  { key: "b27", kind: "attr:name", expr: "c.author", deps: ["c"] },
  { key: "b28", kind: "text", expr: "c.subject", deps: ["c"] },
  { key: "b29", kind: "text", expr: "c.short_sha", deps: ["c"] },
  { key: "b30", kind: "text", expr: "c.author", deps: ["c"] },
  { key: "b31", kind: "text", expr: "c.date", deps: ["c"] },
  { key: "b32", kind: "text", expr: "c.additions", deps: ["c"] },
  { key: "b33", kind: "text", expr: "c.deletions", deps: ["c"] },
  { key: "b36", kind: "attr:class", expr: "b.active ? 'git-icon-active' : 'git-icon-muted'", deps: ["b"] },
  { key: "b37", kind: "text", expr: "b.name", deps: ["b"] },
  { key: "b38", kind: "text", expr: "b.upstream ? b.upstream : 'local only'", deps: ["b"] },
  { key: "b39", kind: "text", expr: "b.ahead > 0 ? '\xE2\x86\x91' + b.ahead : ''", deps: ["b"] },
  { key: "b40", kind: "text", expr: "b.behind > 0 ? ' \xE2\x86\x93' + b.behind : ''", deps: ["b"] },
  { key: "b42", kind: "text", expr: "staged().length", deps: ["staged"] },
  { key: "b44", kind: "text", expr: "f.path", deps: ["f"] },
  { key: "b46", kind: "text", expr: "modified().length", deps: ["modified"] },
  { key: "b48", kind: "text", expr: "f.path", deps: ["f"] },
  { key: "b50", kind: "text", expr: "untracked().length", deps: ["untracked"] },
  { key: "b52", kind: "text", expr: "f.path", deps: ["f"] },
  { key: "b54", kind: "bind:value", expr: "commitFilter.query", deps: ["commitFilter"] },
  { key: "b55", kind: "text", expr: "commitFilter.filtered().length", deps: ["commitFilter"] },
  { key: "b56", kind: "text", expr: "commitsCount()", deps: ["commitsCount"] },
  { key: "b58", kind: "text", expr: "commitsCount() === 0 ? 'No commits' : 'No matching commits'", deps: ["commitsCount"] },
  { key: "b60", kind: "attr:name", expr: "c.author", deps: ["c"] },
  { key: "b61", kind: "text", expr: "c.subject", deps: ["c"] },
  { key: "b62", kind: "text", expr: "c.short_sha", deps: ["c"] },
  { key: "b63", kind: "text", expr: "c.author", deps: ["c"] },
  { key: "b64", kind: "text", expr: "c.date", deps: ["c"] },
  { key: "b65", kind: "text", expr: "c.additions", deps: ["c"] },
  { key: "b66", kind: "text", expr: "c.deletions", deps: ["c"] },
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
