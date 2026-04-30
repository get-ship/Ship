// view.ts
import { controller, unwrapOr, useFilteredList } from "@ship-set/runtime";
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
export {
  view_default as default
};
