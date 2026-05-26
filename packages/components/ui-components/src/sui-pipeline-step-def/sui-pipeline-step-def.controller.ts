import { controller, derived } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-step-def.sui'

type Tone = 'muted' | 'info' | 'warning' | 'accent'

const TYPE_TONE: Record<string, Tone> = {
  run: 'muted',
  agent: 'info',
  gate: 'warning',
  merge: 'accent',
  human: 'warning',
}

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const kindTone = derived<Tone>(() => TYPE_TONE[ctx.stepType()] ?? 'muted')
    const afterList = derived<string[]>(() =>
      String(ctx.after() || '')
        .split(',')
        .map((x: string) => x.trim())
        .filter(Boolean),
    )
    const commandTrim = derived<string>(() => (ctx.command() || '').trim())
    return { kindTone, afterList, commandTrim }
  },
})
