/**
 * Merged sample registry. Topic files contribute their own maps; this
 * file combines them into the single SAMPLES record the renderer reads.
 */

import { ACTIONS } from './actions'
import { DATA } from './data'
import { DOMAIN } from './domain'
import { FORMS } from './forms'
import { LAYOUT } from './layout'
import { OVERLAYS } from './overlays'
import { STATUS } from './status'
import { TYPOGRAPHY } from './typography'
import { UTILITY } from './utility'
import type { Sample } from './types'

export const SAMPLES: Record<string, Sample[]> = {
  ...TYPOGRAPHY,
  ...ACTIONS,
  ...FORMS,
  ...STATUS,
  ...DATA,
  ...LAYOUT,
  ...UTILITY,
  ...OVERLAYS,
  ...DOMAIN,
}

export {
  FULL_WIDTH,
  GROUPS,
  allGroupedTags,
  isTrigger,
  type Group,
  type PlainSample,
  type Sample,
  type TriggerSample,
} from './types'
