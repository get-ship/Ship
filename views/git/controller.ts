// Git view controller — wires `ship.git.overview()` (always-on) and
// `ship.git.diff()` (lazy, gated on `selectedCommit`) over the daemon's
// `/_sui/data` SSE bridge. View-host (`/sui/<name>`) mounts views with
// no `ship` client in ctx, so `liveExpr` is the sanctioned path.
//
// Tab + selection + diff-mode state is local to the controller; tabs
// switch via on:click + kbd 1/2/3. Refresh bumps `refreshNonce`,
// invalidating both thunks so `live()` re-subscribes their SSE channels.

import { controller, signal, live, unwrapOr, useFilteredList } from '@ship/sui-runtime'
import type { LiveSource } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type {
  GitBranch,
  GitCommit,
  GitDiff,
  GitFileEntry,
  GitOverview,
} from '@ship/sdk-client'
import type { Scope } from './page.sui'

type GitTab = 'changes' | 'history' | 'branches'
type DiffMode = 'unified' | 'split'

const EMPTY_OVERVIEW: GitOverview = {
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
  behind: 0,
}

// A LiveSource that never resolves — keeps `diff()` parked in `loading`
// until a commit is actually selected, instead of opening an EventSource
// we won't read.
const NEVER: LiveSource<GitDiff> = { subscribe: () => () => {} }

export default controller<Scope>({
  scope: () => {
    const tab = signal<GitTab>('changes')
    const selectedCommit = signal<string>('')
    const selectedFile = signal<string>('')
    const diffMode = signal<DiffMode>('unified')
    const activeBranchName = signal<string>('')
    const refreshNonce = signal<number>(0)
    const overview = live<GitOverview>(() => {
      refreshNonce() // dep — refresh re-subscribes
      return liveExpr<GitOverview>('ship.git.overview()')
    })
    const diff = live<GitDiff>(() => {
      refreshNonce()
      const sha = selectedCommit()
      if (!sha) return NEVER
      return liveExpr<GitDiff>(`ship.git.diff(${JSON.stringify({ commit: sha })})`)
    })
    return {
      tab, selectedCommit, selectedFile, diffMode, activeBranchName,
      refreshNonce, overview, diff,
    }
  },

  derived: {
    branch:         (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branch ?? '',
    headShort:      (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).head_short ?? '',
    clean:          (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).clean,
    totalChanged:   (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).total_changed,
    ahead:          (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).ahead,
    behind:         (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).behind,
    staged:         (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).staged,
    modified:       (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).modified,
    untracked:      (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).untracked,
    commits:        (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).commits,
    branches:       (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branches,
    commitsCount:   (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).commits.length,
    branchesCount:  (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branches.length,
    recentCommits:  (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).commits.slice(0, 5),
    recentBranches: (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).branches.slice(0, 6),
    // Flattened file list for the split-Changes pane. Each entry
    // carries the porcelain status label the row renders. Order:
    // staged, modified, untracked — matches the j2 partial grouping.
    fileGroups: (s) => {
      const o = unwrapOr(s.overview, EMPTY_OVERVIEW)
      return [
        { key: 'staged' as const,    label: 'Staged',    glyph: 'S', tone: 'success', variant: 'soft',    items: o.staged    },
        { key: 'modified' as const,  label: 'Modified',  glyph: '',  tone: 'warning', variant: 'soft',    items: o.modified  },
        { key: 'untracked' as const, label: 'Untracked', glyph: '?', tone: 'muted',   variant: 'outline', items: o.untracked },
      ].filter((g) => g.items.length > 0)
    },
    activeBranch:   (s) => {
      const list = unwrapOr(s.overview, EMPTY_OVERVIEW).branches
      const name = (s.activeBranchName as unknown as () => string)()
      const picked = name ? list.find((b) => b.name === name) : undefined
      return picked ?? list.find((b) => b.active) ?? list[0] ?? null
    },
    diffFiles: (s) => {
      const v = (s.diff as unknown as () => { kind: string; value?: GitDiff })()
      if (v.kind !== 'ok' || !v.value) return []
      const file = (s.selectedFile as unknown as () => string)()
      const all = v.value.parsed?.files ?? []
      return file ? all.filter((f) => f.path === file) : all
    },
    diffReady: (s) =>
      (s.diff as unknown as () => { kind: string })().kind === 'ok',
    diffIsWorking: (s) => {
      const v = (s.diff as unknown as () => { kind: string; value?: GitDiff })()
      return v.kind === 'ok' && !!v.value?.is_working
    },
  },

  use: (s) => ({
    commitFilter: useFilteredList<GitCommit>(
      () => unwrapOr(s.overview, EMPTY_OVERVIEW).commits,
      (c, q) => {
        if (!q) return true
        const needle = q.toLowerCase()
        return (
          c.subject.toLowerCase().includes(needle) ||
          c.short_sha.toLowerCase().includes(needle) ||
          c.author.toLowerCase().includes(needle)
        )
      },
    ),
  }),

  on: {
    showChanges:  (s) => { s.tab = 'changes' },
    showHistory:  (s) => { s.tab = 'history' },
    showBranches: (s) => { s.tab = 'branches' },
    refresh: (s) => {
      const cur = (s.refreshNonce as unknown as number)
      s.refreshNonce = (cur + 1) as never
    },
    selectFile: (s, file: GitFileEntry) => {
      s.selectedFile = file.path as never
      s.selectedCommit = 'WORKING' as never
      s.tab = 'changes'
    },
    selectCommit: (s, sha: string) => {
      s.selectedCommit = sha as never
      s.selectedFile = '' as never
      s.tab = 'changes'
    },
    setDiffMode: (s, mode: DiffMode) => { s.diffMode = mode as never },
    selectBranch: (s, name: string) => { s.activeBranchName = name as never },
  },
})

export type { GitBranch, GitCommit, GitDiff, GitFileEntry, GitOverview }
