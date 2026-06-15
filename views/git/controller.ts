// Git view controller — read-only. Git is PARKED: the write path (stage /
// commit / push / pull / checkout / branch_* / init) went out with GitService.
// Reads come over the SDK's `ship.git.*` live sources, which route through the
// daemon's mux resolver (`git.overview`), not a Connect service. Overview is
// signal-backed; diff is lazy on the selected commit/file.

import { controller, signal, live, derived, unwrapOr, useFilteredList } from '@ship/sui-runtime'
import type { LiveSource, ViewScope } from '@ship/sui-runtime'
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
// until a commit is actually selected.
const NEVER: LiveSource<GitDiff> = { subscribe: () => () => {} }

const SUBJECT_RE = /^([a-z]+(?:\([^)]+\))?!?):\s*(.+)$/

/** A commit enriched with its presentation fields for the history row. */
type HistoryRow = GitCommit & { scopePrefix: string; subjectRest: string; initial: string }

function parseCommitSubject(subject: string): { prefix: string; rest: string } {
  const m = subject.match(SUBJECT_RE)
  return m ? { prefix: `${m[1]}:`, rest: m[2] ?? subject } : { prefix: '', rest: subject }
}

function getAuthorInitial(author: string): string {
  return (author || '?').charAt(0).toUpperCase()
}

export default controller({
  scope: ({ ship }: ViewScope<Scope>) => {
    const tab = signal<GitTab>('changes')
    const selectedCommit = signal<string>('')
    const selectedFile = signal<string>('')
    const diffMode = signal<DiffMode>('unified')
    const activeBranchName = signal<string>('')
    const refreshNonce = signal<number>(0)

    const overview = live<GitOverview>(() => {
      refreshNonce()
      return ship.git.overview()
    })
    const diff = live<GitDiff>(() => {
      refreshNonce()
      const sha = selectedCommit()
      if (!sha) return NEVER
      return ship.git.diff({ commit: sha, workspace_id: null })
    })
    const branch         = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).branch ?? '')
    const headShort      = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).head_short ?? '')
    const clean          = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).clean)
    const totalChanged   = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).total_changed)
    const ahead          = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).ahead)
    const behind         = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).behind)
    const staged         = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).staged)
    const modified       = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).modified)
    const untracked      = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).untracked)
    const commits        = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).commits)
    const branches       = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).branches)
    const commitsCount   = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).commits.length)
    const branchesCount  = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).branches.length)
    const recentCommits  = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).commits.slice(0, 5))
    const recentBranches = derived(() => unwrapOr(overview, EMPTY_OVERVIEW).branches.slice(0, 6))
    const fileGroups     = derived(() => {
      const o = unwrapOr(overview, EMPTY_OVERVIEW)
      return [
        { key: 'staged' as const,    label: 'Staged',    glyph: 'S', tone: 'success', variant: 'soft',    items: o.staged    },
        { key: 'modified' as const,  label: 'Modified',  glyph: '',  tone: 'warning', variant: 'soft',    items: o.modified  },
        { key: 'untracked' as const, label: 'Untracked', glyph: '?', tone: 'muted',   variant: 'outline', items: o.untracked },
      ].filter((g) => g.items.length > 0)
    })
    const activeBranch   = derived(() => {
      const list = unwrapOr(overview, EMPTY_OVERVIEW).branches
      const name = activeBranchName()
      const picked = name ? list.find((b) => b.name === name) : undefined
      return picked ?? list.find((b) => b.active) ?? list[0] ?? null
    })
    const diffFiles      = derived(() => {
      const v = (diff as unknown as () => { kind: string; value?: GitDiff })()
      if (v.kind !== 'ok' || !v.value) return []
      const file = selectedFile()
      const all = v.value.parsed?.files ?? []
      return file ? all.filter((f) => f.path === file) : all
    })
    const diffReady      = derived(() =>
      (diff as unknown as () => { kind: string })().kind === 'ok')
    const diffIsWorking  = derived(() => {
      const v = (diff as unknown as () => { kind: string; value?: GitDiff })()
      return v.kind === 'ok' && !!v.value?.is_working
    })
    const isNoRepo       = derived(() => {
      const v = (overview as unknown as () => { kind: string; error?: { message?: string } })()
      const msg = v.kind === 'error' ? (v.error?.message ?? '') : ''
      return /no.repo|repo unavailable|not a git/i.test(msg)
    })
    return {
      tab, selectedCommit, selectedFile, diffMode, activeBranchName,
      refreshNonce, overview, diff,
      branch, headShort, clean, totalChanged, ahead, behind,
      staged, modified, untracked, commits, branches,
      commitsCount, branchesCount, recentCommits, recentBranches,
      fileGroups, activeBranch, diffFiles, diffReady, diffIsWorking,
      isNoRepo,
    }
  },

  use: (s) => ({
    // Source is enriched BEFORE filtering so each row carries its presentation
    // fields (scope prefix, subject remainder, author initial) as plain data.
    // Templates can only do zero-arg reads — no call-with-args — so this parsing
    // can't live in page.sui; the history row reads c.scopePrefix / c.subjectRest
    // / c.initial directly off the filtered list.
    commitFilter: useFilteredList<HistoryRow>(
      () =>
        unwrapOr(s.overview, EMPTY_OVERVIEW).commits.map((c) => {
          const parsed = parseCommitSubject(c.subject)
          return { ...c, scopePrefix: parsed.prefix, subjectRest: parsed.rest, initial: getAuthorInitial(c.author) }
        }),
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
      s.refreshNonce = (cur + 1)
    },
    selectFile: (s, file: GitFileEntry) => {
      s.selectedFile = file.path
      s.selectedCommit = 'WORKING'
      s.tab = 'changes'
    },
    selectCommit: (s, sha: string) => {
      s.selectedCommit = sha
      s.selectedFile = ''
      s.tab = 'changes'
    },
    setDiffMode: (s, mode: DiffMode) => { s.diffMode = mode },
    selectBranch: (s, name: string) => { s.activeBranchName = name },
  },
})

export type { GitBranch, GitCommit, GitDiff, GitFileEntry, GitOverview }
