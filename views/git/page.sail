<controller>
// Git view controller — read-only. Git is PARKED: the write path (stage /
// commit / push / pull / checkout / branch_* / init) went out with GitService.
// Reads come over the SDK's `ship.git.*` live sources, which route through the
// daemon's mux resolver (`git.overview`), not a Connect service. Overview is
// signal-backed; diff is lazy on the selected commit/file.

import { controller, signal, live, derived, unwrapOr, useFilteredList } from '@ship/sail-runtime'
import type { LiveSignal, LiveSource, ViewScope } from '@ship/sail-runtime'
import type {
  GitBranch,
  GitCommit,
  GitDiff,
  GitFileEntry,
  GitOverview,
} from '@ship/sdk-client'
import type { Scope } from './page.sail'

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
  scope: ({ ship }: ViewScope) => {
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

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  use: (s: any) => ({
    // Source is enriched BEFORE filtering so each row carries its presentation
    // fields (scope prefix, subject remainder, author initial) as plain data.
    // Templates can only do zero-arg reads — no call-with-args — so this parsing
    // can't live in page.sail; the history row reads c.scopePrefix / c.subjectRest
    // / c.initial directly off the filtered list.
    // s is typed `any` (not ViewScope<Scope>) because precise Scope types are
    // generated by deck-cli from page.sail and not yet available in this seed copy.
    commitFilter: useFilteredList<HistoryRow>(
      () =>
        unwrapOr(s.overview as LiveSignal<GitOverview>, EMPTY_OVERVIEW).commits.map((c: GitCommit) => {
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

  // s is typed `any` in on: handlers (not MutableState<Scope>) because precise
  // Scope types are generated by deck-cli from page.sail and not yet available in
  // this seed copy.
  on: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    showChanges:  (s: any) => { s.tab = 'changes' },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    showHistory:  (s: any) => { s.tab = 'history' },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    showBranches: (s: any) => { s.tab = 'branches' },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    refresh: (s: any) => {
      const cur = (s.refreshNonce as unknown as number)
      s.refreshNonce = (cur + 1)
    },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    selectFile: (s: any, file: GitFileEntry) => {
      s.selectedFile = file.path
      s.selectedCommit = 'WORKING'
      s.tab = 'changes'
    },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    selectCommit: (s: any, sha: string) => {
      s.selectedCommit = sha
      s.selectedFile = ''
      s.tab = 'changes'
    },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    setDiffMode: (s: any, mode: DiffMode) => { s.diffMode = mode },
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    selectBranch: (s: any, name: string) => { s.activeBranchName = name },
  },
})

export type { GitBranch, GitCommit, GitDiff, GitFileEntry, GitOverview }
</controller>

<client name="ship" />
<data name="overview" source="ship.git.overview()" live />

<use-kbd>
  <on key="1" action="showChanges" />
  <on key="2" action="showHistory" />
  <on key="3" action="showBranches" />
</use-kbd>

<div>
  
    <while-loading of="overview">
    <div data-test="loading" aria-busy="true">
      <spinner shape="line" size="lg"></spinner>
      <for as="n" of={[0, 1, 2]} key={n}>
        <spinner shape="line" size="md"></spinner>
      </for>
    </div>

    </while-loading><while-error as="err" of="overview">
    <if cond={isNoRepo()}>
      <div data-test="no-repo">
        <empty-state tone="muted" size="lg">
          <icon slot="icon" name="git-branch" size="lg"></icon>
          <span slot="title">Not a git repository</span>
          <span slot="description">{err.message}</span>
        </empty-state>
      </div>
    <else>
      <error-state size="md" title="Couldn't load repo" data-test="error">{err.message}</error-state>
    </else></if>

    </while-error><if cond={overview().kind === 'ok'}>
    <header>
      <div>
        <h1>Git</h1>
        <p>
          <span>{branch()}</span>
          {ahead() > 0 ? ' ↑' + ahead() : ''}
          {behind() > 0 ? ' ↓' + behind() : ''}
          &middot;
          <span>{headShort()}</span>
        </p>
      </div>
      <row align="center">
        <button variant="ghost" size="sm" aria-label="Refresh" data-test="git-refresh" on:click={refresh}>
          <icon name="refresh-cw" size="sm"></icon>
        </button>
        <chip tone={clean() ? 'success' : 'warning'} variant="soft" size="sm">{clean() ? 'clean' : totalChanged() + ' changed'}</chip>
      </row>
    </header>

    <div>
      <tabs variant="underline">
        <tab name="changes"  active={tab() === 'changes'}  count={totalChanged() + ''}  on:click={() => tab.set('changes')}>Changes</tab>
        <tab name="history"  active={tab() === 'history'}  count={commitsCount() + ''}  on:click={() => tab.set('history')}>History</tab>
        <tab name="branches" active={tab() === 'branches'} count={branchesCount() + ''} on:click={() => tab.set('branches')}>Branches</tab>
      </tabs>
      <div>
        <button variant="ghost" size="sm" on:click={() => navigate('/pipelines')}>
          <icon slot="leading" name="play" size="xs"></icon>Run checks
        </button>
      </div>
    </div>

    <div>
      <if cond={tab() === 'changes'}>
        <if cond={clean()}>
          <div>
            <div>
              <div><icon name="check" size="md"></icon></div>
              <div>
                <h2>Working tree is clean.</h2>
                <p>
                  Nothing to commit on <span>{branch()}</span>.
                  {ahead() === 0 && behind() === 0 ? "You're up to date with the upstream." : ''}
                  {ahead() > 0 ? ahead() + ' commit' + (ahead() === 1 ? '' : 's') + ' ahead of upstream.' : ''}
                  {behind() > 0 ? ' ' + behind() + ' commit' + (behind() === 1 ? '' : 's') + ' behind upstream.' : ''}
                </p>
              </div>
            </div>

            <div>
              <section>
                <h3>Recent on this branch</h3>
                <if cond={recentCommits().length === 0}><p>No commits yet.</p></if>
                <for as="c" of={recentCommits()} key={c.sha}>
                  <div>
                    <avatar size="sm" shape="round" name={c.author}></avatar>
                    <div>
                      <div>{c.subject}</div>
                      <div>{c.short_sha} &middot; {c.author} &middot; {c.date}</div>
                    </div>
                    <div>
                      <span>+{c.additions}</span>
                      <span>−{c.deletions}</span>
                    </div>
                  </div>
                </for>
              </section>
              <section>
                <h3>Branches</h3>
                <if cond={recentBranches().length === 0}><p>No branches.</p></if>
                <for as="b" of={recentBranches()} key={b.name}>
                  <div>
                    <icon name="git-branch" size="xs"></icon>
                    <div>
                      <div>{b.name}</div>
                      <div>{b.upstream ? b.upstream : 'local only'}</div>
                    </div>
                    <div>
                      {b.ahead > 0 ? '↑' + b.ahead : ''}
                      {b.behind > 0 ? ' ↓' + b.behind : ''}
                    </div>
                  </div>
                </for>
              </section>
            </div>
          </div>
        <else>
          <div>
            <aside aria-label="Changed files">
              <div>
                <icon name="git-branch" size="xs"></icon>
                <strong>Changes</strong>
                <span style="flex: 1;"></span>
                <span>{totalChanged()} files</span>
              </div>
              <div>
                <for as="g" of={fileGroups()} key={g.key}>
                  <chip tone={g.tone} variant={g.variant} size="sm">{g.items.length} {g.key}</chip>
                </for>
              </div>
              <scroll-area>
                <for as="g" of={fileGroups()} key={g.key}>
                  <div>{g.label}</div>
                  <for as="f" of={g.items} key={f.path}>
                    <div data-active={selectedFile() === f.path}>
                      <button type="button"
                        data-test="git-file-row"
                        on:click={() => selectFile(f)}>
                        <span>{g.glyph || f.status}</span>{f.path}
                      </button>
                    </div>
                  </for>
                </for>
              </scroll-area>
            </aside>

            <section aria-label="Diff">
              <if cond={!selectedCommit()}>
                <empty-state tone="muted" size="md">
                  <span slot="title">Select a file</span>
                  Pick a file on the left to view its diff.
                </empty-state>
              <else-if cond={!diffReady()}>
                <div data-test="diff-loading">Loading diff&hellip;</div>
              </else-if><else-if cond={diffFiles().length === 0}>
                <empty-state tone="muted" size="md">
                  <span slot="title">No changes in this diff</span>
                </empty-state>
              </else-if><else>
                <div>
                  <badge tone="info" variant="soft" size="sm">diff</badge>
                  <span data-test="diff-path">{diffFiles()[0].path}</span>
                  <span style="flex: 1;"></span>
                  <span><span>+{diffFiles()[0].additions}</span> <span>−{diffFiles()[0].deletions}</span></span>
                  <toggle-group variant="segmented" type="single" value={diffMode()} size="sm" on:sail-change={(e) => setDiffMode(e.detail.value)}>
                    <toggle value="unified">Unified</toggle>
                    <toggle value="split">Split</toggle>
                  </toggle-group>
                </div>
                <if cond={diffIsWorking()}>
                  <alert tone="muted"><icon slot="icon" name="git-commit" size="xs"></icon>Viewing working tree changes vs HEAD.</alert>
                </if>
                <scroll-area data-test="diff-body">
                  <for as="file" of={diffFiles()} key={file.path}>
                    <ship-git-diff files="1">
                      <ship-git-diff-file path={file.path}
                        additions={file.additions + ''}
                        deletions={file.deletions + ''}>
                        <for as="hunk" of={file.hunks} key={hunk.header}>
                          <ship-git-diff-hunk header={hunk.header}></ship-git-diff-hunk>
                          <for as="line" of={hunk.lines} key={(line.old_num || 0) + ':' + (line.new_num || 0)}>
                            <ship-git-diff-line
                              type={line.kind}
                              old={line.old_num ? (line.old_num + '') : null}
                              new={line.new_num ? (line.new_num + '') : null}>{line.content}</ship-git-diff-line>
                          </for>
                        </for>
                      </ship-git-diff-file>
                    </ship-git-diff>
                  </for>
                </scroll-area>
              </else></if>
            </section>
          </div>
        </else></if>
      </if>

      <if cond={tab() === 'history'}>
        <div>
          <row align="center">
            <input placeholder="Filter by subject, sha, or author"
              bind:value={commitFilter.query} />
            <span>{commitFilter.filtered().length} of {commitsCount()}</span>
          </row>
          <if cond={commitFilter.filtered().length === 0}>
            <empty-state tone="muted" size="md">
              <if cond={commitsCount() === 0}>
                <span slot="title">No commits</span>
              <else>
                <span slot="title">No matching commits</span>
              </else></if>
            </empty-state>
          <else>
            <div data-test="history-rows">
              <for as="c" of={commitFilter.filtered()} key={c.sha}>
                <div data-test="history-row"
                  role="button" tabindex="0"
                  on:click={() => selectCommit(c.sha)}
                  on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectCommit(c.sha); } }}>
                  <div>
                    <svg width="44" height="46" aria-hidden="true">
                      <line x1="22" y1="0" x2="22" y2="46"
                        stroke="var(--primary)" stroke-width="1.5" opacity="0.45"></line>
                      <circle cx="22" cy="23" r="4.5"
                        fill="var(--background)"
                        stroke="var(--primary)" stroke-width="2"></circle>
                    </svg>
                  </div>
                  <div>{c.initial}</div>
                  <div>
                    <div>
                      <if cond={c.scopePrefix}><span>{c.scopePrefix}</span> </if>{c.subjectRest}
                    </div>
                    <div>
                      <span>{c.short_sha}</span>
                      <span>{c.date}</span>
                    </div>
                  </div>
                  <div>
                    <if cond={c.additions > 0}><span>+{c.additions}</span></if>
                    <if cond={c.deletions > 0}><span>−{c.deletions}</span></if>
                  </div>
                </div>
              </for>
            </div>
          </else></if>
        </div>
      </if>

      <if cond={tab() === 'branches'}>
        <div>
        <if cond={branches().length === 0}>
          <empty-state tone="muted" size="md">
            <span slot="title">No branches</span>
          </empty-state>
        <else>
          <div>
            <aside aria-label="Branches">
              <scroll-area>
                <for as="b" of={branches()} key={b.name}>
                  <div
                    data-test="git-branch-tile"
                    role="button"
                    tabindex="0"
                    style={activeBranchName() === b.name ? 'background:color-mix(in oklch, var(--primary) 10%, var(--muted));border-color:var(--primary)' : ''}
                    on:click={() => selectBranch(b.name)}
                    on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); selectBranch(b.name); } }}>
                    <div>
                      <status-dot
                        tone={b.active ? 'success' : 'muted'}
                        pulse={b.active}></status-dot>
                      <span>{b.name}</span>
                      <span>
                        {b.ahead > 0 ? '↑' + b.ahead : ''}
                        {b.behind > 0 ? ' ↓' + b.behind : ''}
                      </span>
                    </div>
                    <div>{b.upstream ? b.upstream : 'local only'}</div>
                  </div>
                </for>
              </scroll-area>
            </aside>
            <section aria-label="Branch detail">
              <if cond={activeBranch()}>
                <div data-test="git-branch-detail">
                  <div>
                    <div>
                      <div>{activeBranch().name}</div>
                      <div>
                        {activeBranch().upstream ? 'tracking ' + activeBranch().upstream : 'no upstream'}
                      </div>
                    </div>
                  </div>
                  <div>
                    <card>
                      <div slot="header">Status</div>
                      <div><span>Head</span><span>{activeBranch().name}</span></div>
                      <if cond={activeBranch().upstream}>
                        <div><span>Upstream</span><span>{activeBranch().upstream}</span></div>
                      </if>
                      <div><span>Divergence</span><span>{'↑' + activeBranch().ahead + ' ↓' + activeBranch().behind}</span></div>
                    </card>
                  </div>
                </div>
              </if>
            </section>
          </div>
        </else></if>
        </div>
      </if>
    </div>

  </if>
</div>
