import { controller, signal, onCleanup } from '@ship/sui-runtime'
import type { Scope } from './sui-studio-skills.sui'

import '../sui-skills-list/sui-skills-list.sui'
import '../sui-skill-detail/sui-skill-detail.sui'

function parseSkillId(): string {
  const parts = typeof window !== 'undefined'
    ? window.location.pathname.split('/').filter(Boolean)
    : []
  // /studio/skills/[id] → ['studio', 'skills', 'id']
  return parts.length >= 3 && parts[0] === 'studio' && parts[1] === 'skills'
    ? parts[2]
    : ''
}

export default controller<Scope>({
  scope: () => {
    const detailId = signal(parseSkillId())
    const onNav = () => detailId.set(parseSkillId())
    if (typeof window !== 'undefined') {
      window.addEventListener('popstate', onNav)
      document.addEventListener('sui:navigate', onNav)
      onCleanup(() => {
        window.removeEventListener('popstate', onNav)
        document.removeEventListener('sui:navigate', onNav)
      })
    }
    return { detailId }
  },
})
