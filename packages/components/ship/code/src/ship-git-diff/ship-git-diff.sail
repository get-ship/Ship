<controller>
import { controller, derived, effect, signal } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-git-diff.sail'

type GitDiffFile = HTMLElement & { collapsed: boolean }

export default controller((ctx: ViewScope<Scope>, host) => {

    const allCollapsed = signal<boolean>(false)

    const showEmpty = derived<boolean>(() => ctx.empty() || ctx.files() === 0)
    const filesLabel = derived<string>(() => {
      const n = ctx.files()
      return `${n} file${n === 1 ? '' : 's'} changed`
    })
    const toggleIcon = derived<string>(() =>
      allCollapsed() ? 'chevrons-up-down' : 'chevrons-down-up',
    )
    const toggleLabel = derived<string>(() =>
      allCollapsed() ? 'Expand all' : 'Collapse all',
    )
    const toggleTitle = derived<string>(() =>
      allCollapsed() ? 'Expand all files' : 'Collapse all files',
    )

    function toggleAll(): void {
      const next = !allCollapsed()
      allCollapsed.set(next)
      const kids = host?.querySelectorAll<GitDiffFile>('ship-git-diff-file')
      kids?.forEach((f) => {
        if (next) f.setAttribute('collapsed', '')
        else f.removeAttribute('collapsed')
      })
    }

    return {
      allCollapsed,
      showEmpty,
      filesLabel,
      toggleIcon,
      toggleLabel,
      toggleTitle,
      toggleAll,
    }
})
</controller>

<!--
  <ship-git-diff> — SUI port of <ship-git-diff>.

  Container for a unified-diff tree. Renders a toolbar (file count + add/
  remove totals + "expand all / collapse all" toggle) and slots the
  <ship-git-diff-file> children below. The daemon parses the diff and
  emits the tree as light DOM; this primitive supplies the chrome.

  Prop / event / slot parity vs <ship-git-diff>
  ─────────────────────────────────────────────────────────────────────────────
  Lit                          SUI                      Notes
  ─────────────────────────────────────────────────────────────────────────────
  files (number, 0)            files (number, 0,        ✓ reflects
                                  reflect)
  additions (number, 0)        additions (number, 0,    ✓
                                  reflect)
  deletions (number, 0)        deletions (number, 0,    ✓
                                  reflect)
  empty (bool, false)          empty (bool, false,      ✓ replaces the
                                  reflect)                 toolbar+body with
                                                           an empty-state row
  allCollapsed (state)         allCollapsed (signal,    ✓ controller-local
                                  controller)              UI state
  ─────────────────────────────────────────────────────────────────────────────
  role=region + aria-label     controller mount + eff   ✓
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="files" type="number" default="0" reflect /> -->
<!-- <prop name="additions" type="number" default="0" reflect /> -->
<!-- <prop name="deletions" type="number" default="0" reflect /> -->
<!-- <prop name="empty" type="bool" default="false" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<if cond={showEmpty()}>
  <div>No changes to display</div>
<else>
  <div>
    <span>{filesLabel()}</span>
    <if cond={additions() > 0}><span>+{additions()}</span></if>
    <if cond={deletions() > 0}><span>-{deletions()}</span></if>
    <span></span>
    <button type="button" on:click={toggleAll} title={toggleTitle()}>
      <icon size="sm" name={toggleIcon()}></icon>
      {toggleLabel()}
    </button>
  </div>
  <div><slot></slot></div>
</else></if>
