// Git view controller. Reads come over the SDK's `ship.git.*` live sources
// (overview is signal-backed; diff is lazy on selectedCommit). Mutations
// go through `ship.git.stage/unstage/commit/push/pull/checkout/branch_*/init`
// and bump `refreshNonce` so the live overview re-subscribes and the
// daemon re-runs the signal resolver.

import { controller, signal, live, unwrapOr, useFilteredList } from '@ship/sui-runtime'
import type { LiveSource } from '@ship/sui-runtime'
import type {
  GitBranch,
  GitCommit,
  GitDiff,
  GitFileEntry,
  GitOverview,
  ShipSDK,
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

function errMessage(err: unknown): string {
  if (err instanceof Error) return err.message
  if (typeof err === 'string') return err
  try { return JSON.stringify(err) } catch { return String(err) }
}

export default controller<Scope>({
  scope: ({ ship }: { ship: ShipSDK }) => {
    // Mutation closures captured here so `on:` handlers can fire daemon
    // ops through the typed SDK surface. The runtime strips raw object
    // refs from scope on serialization; only function values survive.
    const stagePaths   = (paths: string[]) => ship.git.stage({ paths })
    const unstagePaths = (paths: string[]) => ship.git.unstage({ paths })
    const commitNow    = (message: string) => ship.git.commit({ message })
    const pushNow      = () => ship.git.push({})
    const pullNow      = () => ship.git.pull({})
    const fetchNow     = () => ship.git.fetch({})
    const checkoutRef  = (ref: string) => ship.git.checkout({ ref })
    const createBranch = (name: string, checkout: boolean) =>
      ship.git.branch_create({ name, checkout })
    const deleteBranch = (name: string, force: boolean) =>
      ship.git.branch_delete({ name, force })
    const initNow      = () => ship.git.init()

    const tab = signal<GitTab>('changes')
    const selectedCommit = signal<string>('')
    const selectedFile = signal<string>('')
    const diffMode = signal<DiffMode>('unified')
    const activeBranchName = signal<string>('')
    const refreshNonce = signal<number>(0)

    // Status banner — surfaces every op error as `<sui-alert>`. Cleared
    // on the next successful op or by explicit dismiss.
    const opError = signal<string>('')
    const opInfo = signal<string>('')
    const busy = signal<string>('') // op label while a mutation is in flight

    // Commit dialog state.
    const commitOpen = signal(false)
    const commitSubject = signal<string>('')
    const commitBody = signal<string>('')
    const commitError = signal<string>('')

    // Branch dialogs.
    const branchCreateOpen = signal(false)
    const branchCreateName = signal<string>('')
    const branchCreateCheckout = signal(true)
    const branchCreateError = signal<string>('')
    const branchDeleteOpen = signal(false)
    const branchDeleteTarget = signal<string>('')
    const branchDeleteForce = signal(false)
    const branchDeleteError = signal<string>('')

    const overview = live<GitOverview>(() => {
      refreshNonce()
      return ship.git.overview()
    })
    const diff = live<GitDiff>(() => {
      refreshNonce()
      const sha = selectedCommit()
      if (!sha) return NEVER
      return ship.git.diff({ commit: sha })
    })
    return {
      tab, selectedCommit, selectedFile, diffMode, activeBranchName,
      refreshNonce, overview, diff,
      opError, opInfo, busy,
      commitOpen, commitSubject, commitBody, commitError,
      branchCreateOpen, branchCreateName, branchCreateCheckout, branchCreateError,
      branchDeleteOpen, branchDeleteTarget, branchDeleteForce, branchDeleteError,
      stagePaths, unstagePaths, commitNow, pushNow, pullNow, fetchNow,
      checkoutRef, createBranch, deleteBranch, initNow,
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
    stagedCount:    (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).staged.length,
    unstagedCount:  (s) => {
      const o = unwrapOr(s.overview, EMPTY_OVERVIEW)
      return o.modified.length + o.untracked.length
    },
    canCommit:      (s) => unwrapOr(s.overview, EMPTY_OVERVIEW).staged.length > 0,
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
    overviewErrMsg: (s) => {
      const v = (s.overview as unknown as () => { kind: string; error?: { message?: string } })()
      return v.kind === 'error' ? (v.error?.message ?? 'Unknown error') : ''
    },
    isNoRepo: (s) => {
      const v = (s.overview as unknown as () => { kind: string; error?: { message?: string } })()
      const msg = v.kind === 'error' ? (v.error?.message ?? '') : ''
      return /no.repo|repo unavailable|not a git/i.test(msg)
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
    dismissError: (s) => { s.opError = '' as never; s.opInfo = '' as never },

    // ── stage / unstage ───────────────────────────────────────────────
    stageFile: async (s, file: GitFileEntry) => {
      await runOp(s, 'stage', () => callFn(s, 'stagePaths', [file.path]))
    },
    unstageFile: async (s, file: GitFileEntry) => {
      await runOp(s, 'unstage', () => callFn(s, 'unstagePaths', [file.path]))
    },
    stageAll: async (s) => {
      const o = unwrapOv(s)
      const paths = [...o.modified, ...o.untracked].map((f) => f.path)
      if (paths.length === 0) return
      await runOp(s, 'stage all', () => callFn(s, 'stagePaths', paths))
    },
    unstageAll: async (s) => {
      const o = unwrapOv(s)
      const paths = o.staged.map((f) => f.path)
      if (paths.length === 0) return
      await runOp(s, 'unstage all', () => callFn(s, 'unstagePaths', paths))
    },

    // ── commit dialog ─────────────────────────────────────────────────
    openCommit: (s) => {
      s.commitOpen = true as never
      s.commitSubject = '' as never
      s.commitBody = '' as never
      s.commitError = '' as never
    },
    closeCommit: (s) => { s.commitOpen = false as never },
    submitCommit: async (s) => {
      const subject = String(s.commitSubject).trim()
      if (!subject) { s.commitError = 'Subject is required.' as never; return }
      const bodyText = String(s.commitBody).trim()
      const message = bodyText ? `${subject}\n\n${bodyText}` : subject
      s.commitError = '' as never
      const ok = await runOp(s, 'commit', () => callFn(s, 'commitNow', message))
      if (ok) {
        s.commitOpen = false as never
        s.opInfo = `Committed “${subject}”.` as never
      } else {
        s.commitError = String(s.opError) as never
      }
    },

    // ── push / pull / fetch ───────────────────────────────────────────
    pushCurrent: async (s) => {
      await runOp(s, 'push', async () => {
        const r = await callFn<{ remote: string }>(s, 'pushNow')
        s.opInfo = `Pushed to ${r.remote}.` as never
      })
    },
    pullCurrent: async (s) => {
      await runOp(s, 'pull', async () => {
        const r = await callFn<{ head: string }>(s, 'pullNow')
        s.opInfo = `Pulled. HEAD is now ${r.head.slice(0, 7)}.` as never
      })
    },
    fetchOrigin: async (s) => {
      await runOp(s, 'fetch', async () => {
        const r = await callFn<{ remote: string }>(s, 'fetchNow')
        s.opInfo = `Fetched from ${r.remote}.` as never
      })
    },

    // ── branches ──────────────────────────────────────────────────────
    checkoutBranch: async (s, name: string) => {
      await runOp(s, 'checkout', async () => {
        await callFn(s, 'checkoutRef', name)
        s.opInfo = `Switched to ${name}.` as never
      })
    },
    openBranchCreate: (s) => {
      s.branchCreateOpen = true as never
      s.branchCreateName = '' as never
      s.branchCreateError = '' as never
    },
    closeBranchCreate: (s) => { s.branchCreateOpen = false as never },
    submitBranchCreate: async (s) => {
      const name = String(s.branchCreateName).trim()
      if (!/^[a-zA-Z0-9._/-]+$/.test(name)) {
        s.branchCreateError = 'Branch name must be alphanumeric with -_./.' as never
        return
      }
      const checkout = !!s.branchCreateCheckout
      const ok = await runOp(s, 'branch create', () => callFn(s, 'createBranch', name, checkout))
      if (ok) {
        s.branchCreateOpen = false as never
        s.opInfo = `Branch ${name} created${checkout ? ' and checked out' : ''}.` as never
      } else {
        s.branchCreateError = String(s.opError) as never
      }
    },
    openBranchDelete: (s, name: string) => {
      s.branchDeleteTarget = name as never
      s.branchDeleteForce = false as never
      s.branchDeleteError = '' as never
      s.branchDeleteOpen = true as never
    },
    closeBranchDelete: (s) => { s.branchDeleteOpen = false as never },
    submitBranchDelete: async (s) => {
      const name = String(s.branchDeleteTarget)
      const force = !!s.branchDeleteForce
      const ok = await runOp(s, 'branch delete', () => callFn(s, 'deleteBranch', name, force))
      if (ok) {
        s.branchDeleteOpen = false as never
        s.opInfo = `Branch ${name} deleted.` as never
      } else {
        s.branchDeleteError = String(s.opError) as never
      }
    },

    // ── init ──────────────────────────────────────────────────────────
    initRepo: async (s) => {
      await runOp(s, 'init', async () => {
        await callFn(s, 'initNow')
        s.opInfo = 'Initialized git repository.' as never
      })
    },
  },
})

// Shared mutation runner — sets busy, surfaces errors as opError, and
// bumps the refresh nonce on success so the overview re-subscribes.
async function runOp(
  s: Record<string, unknown>,
  label: string,
  fn: () => Promise<unknown>,
): Promise<boolean> {
  s.busy = label as never
  s.opError = '' as never
  s.opInfo = '' as never
  try {
    await fn()
    const cur = s.refreshNonce as number
    s.refreshNonce = (cur + 1) as never
    return true
  } catch (err) {
    s.opError = `${label}: ${errMessage(err)}` as never
    return false
  } finally {
    s.busy = '' as never
  }
}

// Call a scope-captured closure by name. The runtime stores controller-
// closures as plain function references on the scope; cast to call.
function callFn<T = unknown>(
  s: Record<string, unknown>,
  name: string,
  ...args: unknown[]
): Promise<T> {
  const fn = s[name] as (...a: unknown[]) => Promise<T>
  if (typeof fn !== 'function') {
    throw new Error(`controller closure ${name} is not a function`)
  }
  return fn(...args)
}

function unwrapOv(s: Record<string, unknown>): GitOverview {
  const v = (s.overview as () => { kind: string; value?: GitOverview })()
  return v.kind === 'ok' && v.value ? v.value : EMPTY_OVERVIEW
}

export type { GitBranch, GitCommit, GitDiff, GitFileEntry, GitOverview }
