<!--
  <ship-git-diff> — unified-diff chrome.

  Toolbar (file count + +/- totals + expand/collapse) and a slot for
  <ship-git-diff-file> children. Presentational — the host walks the
  parsed Diff tree into those children.
-->
<scope>
  props: {
    files = 0
    additions = 0
    deletions = 0
    empty = false
  }
  style shell = { size({ w: max, min-w: 0 }).bg(background) }
  style bar = { pad({ x: 3, y: 2 }).border({ width: 1, color: border-subtle, edge: b }).size({ w: max, min-w: 0 }) }
  style filesLab = { type({ size: sm, weight: medium }) }
  style add = { type({ family: mono, size: xs }).fg(success) }
  style del = { type({ family: mono, size: xs }).fg(destructive) }
  style spacer = { size(grow) }
  style emptyPad = { pad(6) }
  style body = { size({ w: max, min-w: 0 }) }
</scope>

<controller>
import type { Scope } from './ship-git-diff.sail'
import { setCollapsedOnFiles } from './host'

export default controller((ctx: Scope, host?) => {
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
    const hasAdds = derived<boolean>(() => ctx.additions() > 0)
    const hasDels = derived<boolean>(() => ctx.deletions() > 0)

    function toggleAll(): void {
      const next = !allCollapsed()
      allCollapsed.set(next)
      setCollapsedOnFiles(host, next)
    }

    return {
      allCollapsed,
      showEmpty,
      filesLabel,
      toggleIcon,
      toggleLabel,
      toggleTitle,
      hasAdds,
      hasDels,
      toggleAll,
    }
})
</controller>

<if {showEmpty()}>
  <center style(emptyPad)>
    <empty-state size="sm">No changes to display</empty-state>
  </center>
{else =>
  <column gap="0" style(shell)>
    <row nowrap gap="2" align="center" style(bar)>
      <span style(filesLab)>{filesLabel()}</span>
      <if {hasAdds()}>
        <span style(add)>+{additions()}</span>
      </if>
      <if {hasDels()}>
        <span style(del)>-{deletions()}</span>
      </if>
      <row style(spacer) />
      <button type="button" variant="ghost" size="sm" on:click={toggleAll} title={toggleTitle()}>
        <icon size="sm" name={toggleIcon()}></icon>
        {toggleLabel()}
      </button>
    </row>
    <column gap="0" style(body)>
      <slot></slot>
    </column>
  </column>
}</if>
