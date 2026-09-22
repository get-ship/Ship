<!--
  <ship-git-diff-file> — per-file collapsible header + hunk/line slot.
-->
<scope>
  props: {
    path = ''
    additions = 0
    deletions = 0
    collapsed = false
  }
  style head = { size({ w: max, min-w: 0 }).pad({ x: 3, y: 2 }).bg(surface).border({ width: 1, color: border-subtle, edge: b }) }
  style pathLab = { type({ size: sm, family: mono, wrap: nowrap }).size({ min-w: 0 }).overflow(hidden) }
  style add = { type({ family: mono, size: xs }).fg(success) }
  style del = { type({ family: mono, size: xs }).fg(destructive) }
  style spacer = { size(grow) }
  style body = { size({ w: max, min-w: 0 }) }
</scope>

<controller>
import type { Scope } from './ship-git-diff-file.sail'

export default controller((ctx: Scope) => {
    const chevIcon = derived<string>(() =>
      ctx.collapsed() ? 'chevron-right' : 'chevron-down',
    )
    const hasAdds = derived<boolean>(() => ctx.additions() > 0)
    const hasDels = derived<boolean>(() => ctx.deletions() > 0)
    const isOpen = derived<boolean>(() => !ctx.collapsed())

    function toggle(): void {
      ctx.collapsed.set(!ctx.collapsed())
    }

    return { chevIcon, hasAdds, hasDels, isOpen, toggle }
})
</controller>

<column gap="0">
  <row nowrap gap="2" align="center" style(head) on:click={() => toggle()}>
    <icon size="sm" name={chevIcon()}></icon>
    <icon size="sm" name="file-code"></icon>
    <span style(pathLab)>{path()}</span>
    <row style(spacer) />
    <if {hasAdds()}>
      <span style(add)>+{additions()}</span>
    </if>
    <if {hasDels()}>
      <span style(del)>-{deletions()}</span>
    </if>
  </row>
  <if {isOpen()}>
    <column gap="0" style(body)>
      <slot></slot>
    </column>
  </if>
</column>
