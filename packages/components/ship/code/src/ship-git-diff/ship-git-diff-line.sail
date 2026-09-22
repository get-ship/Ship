<!--
  <ship-git-diff-line> — one unified-diff row (gutters + signature + text).
-->
<scope>
  props: {
    type: 'add' | 'del' | 'context' = 'context'
    old = ''
    'new' = ''
    'comment-count' = 0
  }
  style gutter = { type({ family: mono, size: xs, nums: tabular, align: end }).fg(muted-foreground).size({ min-w: 8 }) }
  style sig = { type({ family: mono, size: xs }) }
  style text = { type({ family: mono, size: xs, wrap: nowrap }).size({ min-w: 0 }).overflow(hidden) }
  style lineAdd = { type({ family: mono, size: xs }).fg(success).pad({ x: 2, y: px }).size({ w: max, min-w: 0 }).bg(surface) }
  style lineDel = { type({ family: mono, size: xs }).fg(destructive).pad({ x: 2, y: px }).size({ w: max, min-w: 0 }).bg(surface) }
  style lineCtx = { type({ family: mono, size: xs }).pad({ x: 2, y: px }).size({ w: max, min-w: 0 }) }
</scope>

<controller>
import type { Scope } from './ship-git-diff-line.sail'

export default controller((ctx: Scope) => {
    // `new` is a reserved word — `ctx.new()` is a syntax error.
    const oldCol = derived<string>(() => ctx.old() ?? '')
    const newCol = derived<string>(() => ctx['new']() ?? '')
    const sigChar = derived<string>(() => {
      const t = ctx.type()
      return t === 'add' ? '+' : t === 'del' ? '-' : ' '
    })
    const isAdd = derived<boolean>(() => ctx.type() === 'add')
    const isDel = derived<boolean>(() => ctx.type() === 'del')

    return { oldCol, newCol, sigChar, isAdd, isDel }
})
</controller>

<if {isAdd()}>
  <row nowrap gap="2" align="center" style(lineAdd)>
    <span style(gutter)>{oldCol()}</span>
    <span style(gutter)>{newCol()}</span>
    <span style(sig)>{sigChar()}</span>
    <span style(text)><slot></slot></span>
  </row>
{else-if {isDel()} =>
  <row nowrap gap="2" align="center" style(lineDel)>
    <span style(gutter)>{oldCol()}</span>
    <span style(gutter)>{newCol()}</span>
    <span style(sig)>{sigChar()}</span>
    <span style(text)><slot></slot></span>
  </row>
}
{else =>
  <row nowrap gap="2" align="center" style(lineCtx)>
    <span style(gutter)>{oldCol()}</span>
    <span style(gutter)>{newCol()}</span>
    <span style(sig)>{sigChar()}</span>
    <span style(text)><slot></slot></span>
  </row>
}</if>
