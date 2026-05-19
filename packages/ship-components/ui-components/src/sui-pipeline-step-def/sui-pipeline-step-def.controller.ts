import { controller, derived } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-pipeline-step-def.sui'

type Tone = 'muted' | 'info' | 'warning' | 'accent'

const TYPE_TONE: Record<string, Tone> = {
  run: 'muted',
  agent: 'info',
  gate: 'warning',
  merge: 'accent',
  human: 'warning',
}

export default controller<Scope>({
  scope: (state) => {
    const s = state as unknown as {
      stepType: Signal<string>
      after: Signal<string>
      command: Signal<string>
    }
    const kindTone = derived<Tone>(() => TYPE_TONE[s.stepType()] ?? 'muted')
    const afterList = derived<string[]>(() =>
      (s.after() || '')
        .split(',')
        .map((x) => x.trim())
        .filter(Boolean),
    )
    const commandTrim = derived<string>(() => (s.command() || '').trim())
    return { kindTone, afterList, commandTrim }
  },
})
