// Skills detail — the skill's content + metadata. Data via
// ship.skills.detail({id}) through liveExpr (the detail payload carries the
// SKILL.md body, variables, files, authors). Read-focused; editing the body
// (the old views/skills draft/save flow) is a follow-up.
import { controller, derived, live } from '@ship/sui-runtime'
import { liveExpr } from '@ship/sdk-client'
import type { Scope } from './page.sui'

interface SkillDetail {
  id: string
  name: string
  description: string
  kind: string
  source: string
  tags: string[]
  authors: string[]
  variables: string[]
  artifacts: string[]
  files: string[]
  body: string
}
type LiveResult = { kind: string; value?: SkillDetail }

export default controller<Scope>({
  scope: ({ route }) => {
    const skillId = derived(() => route.params.id() ?? '')
    const detail = live(() =>
      liveExpr<SkillDetail>(
        `ship.skills.detail(${JSON.stringify({ id: skillId() || '__none__' })})`,
      ),
    )
    const skill = derived(() => {
      const r = detail() as LiveResult
      return r.kind === 'ok' && r.value ? r.value : null
    })
    const loading = derived(() => (detail() as LiveResult).kind !== 'ok')
    // .join() isn't in the template expression allowlist — precompute here.
    const authorsLabel = derived(() => ((skill()?.authors ?? []) as string[]).join(', '))
    return { skillId, detail, skill, loading, authorsLabel }
  },

  on: {
    goBack: ({ navigate }) => navigate('/studio/skills'),
  },
})
