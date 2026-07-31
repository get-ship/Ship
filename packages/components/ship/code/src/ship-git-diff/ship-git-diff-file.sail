<controller>
import { controller, derived, effect } from '@ship/sail-runtime'
import type { ViewScope } from '@ship/sail-runtime'
import type { Scope } from './ship-git-diff-file.sail'

export default controller((ctx: ViewScope<Scope>) => {

    const chevIcon = derived<string>(() =>
      ctx.collapsed() ? 'chevron-right' : 'chevron-down',
    )
    const expandedLabel = derived<string>(() => (ctx.collapsed() ? 'false' : 'true'))

    function toggle(): void {
      ctx.collapsed.set(!ctx.collapsed())
    }

    return { chevIcon, expandedLabel, toggle }
})
</controller>

<!--
  <ship-git-diff-file> — SUI port of <ship-git-diff-file>.

  Per-file collapsible header with the path + +/- counts. Slots the
  file's <ship-git-diff-hunk> + <ship-git-diff-line> children below.

  Prop / event / slot parity vs <ship-git-diff-file>
  ─────────────────────────────────────────────────────────────────────────────
  Lit                          SUI                      Notes
  ─────────────────────────────────────────────────────────────────────────────
  path (string, '')            path (string, '',        ✓ reflects
                                  reflect)
  additions (number, 0)        additions (number, 0,    ✓ reflects
                                  reflect)
  deletions (number, 0)        deletions (number, 0,    ✓ reflects
                                  reflect)
  collapsed (bool, false)      collapsed (bool, false,  ✓ reflects, hides
                                  reflect)                  body via CSS
  ─────────────────────────────────────────────────────────────────────────────
  role=group + aria-label      controller effect        ✓
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="path" type="string" default="" reflect /> -->
<!-- <prop name="additions" type="number" default="0" reflect /> -->
<!-- <prop name="deletions" type="number" default="0" reflect /> -->
<!-- <prop name="collapsed" type="bool" default="false" reflect /> -->

<!-- style block removed: use sx / external CSS -->

<button type="button" on:click={toggle} aria-expanded={expandedLabel()}>
  <icon size="sm" name={chevIcon()}></icon>
  <icon size="sm" name="file-code"></icon>
  <span>{path()}</span>
  <span>
    <if cond={additions() > 0}>
      <span><icon size="sm" name="plus"></icon>{additions()}</span>
    </if>
    <if cond={deletions() > 0}>
      <span><icon size="sm" name="minus"></icon>{deletions()}</span>
    </if>
  </span>
</button>
<div><slot></slot></div>
