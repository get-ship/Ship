// Test helpers for @ship/skills — pure-derived projections the
// controllers own. The production code keeps these inline (the
// controllers' private narrowing + grouping). The helpers re-export
// the same logic with stable signatures so the test suite can lock
// the contract without reaching into controller-internal locals.
//
// Mirrors `readList` (list controller), the grouping loop, the
// frontmatter-row projection (detail controller), and the shared
// SCOPE_LABELS table. The controllers themselves are thin wrappers
// around the live SDK; the helpers expose the algorithmic core the
// templates depend on.

export {
  SCOPE_LABELS,
  SCOPE_ORDER,
  type SkillIndexEntry,
  type SkillGroup,
} from '../src/ship-skill-list/ship-skill-list.controller'

export { type SkillDetail, type FrontmatterRow } from '../src/ship-skill-detail/ship-skill-detail.controller'

import {
  SCOPE_LABELS,
  SCOPE_ORDER,
  type SkillGroup,
  type SkillIndexEntry,
} from '../src/ship-skill-list/ship-skill-list.controller'
import type { FrontmatterRow } from '../src/ship-skill-detail/ship-skill-detail.controller'

/** Group a flat skill list into the stable-order SkillGroup[] the
 *  template renders. Mirrors the controller's `groups` derived —
 *  pulled out as a pure function so the test suite can assert the
 *  grouping rules without wiring up the live SDK. */
export function buildGroupsFromList(items: SkillIndexEntry[]): SkillGroup[] {
  const byScope = new Map<string, SkillIndexEntry[]>()
  for (const s of items) {
    const bucket = byScope.get(s.scope) ?? []
    bucket.push(s)
    byScope.set(s.scope, bucket)
  }
  const out: SkillGroup[] = []
  for (const id of SCOPE_ORDER) {
    const skills = byScope.get(id)
    if (!skills || skills.length === 0) continue
    out.push({ id, label: SCOPE_LABELS[id] ?? id, skills })
  }
  for (const [id, skills] of byScope) {
    if (SCOPE_ORDER.includes(id)) continue
    out.push({ id, label: SCOPE_LABELS[id] ?? id, skills })
  }
  return out
}

/** Project a free-form frontmatter object into the stable-row form
 *  the meta panel renders. Mirrors the controller's
 *  `frontmatterRows` derived. */
export function frontmatterRowsFrom(
  frontmatter: Record<string, unknown>,
): FrontmatterRow[] {
  const out: FrontmatterRow[] = []
  for (const [key, raw] of Object.entries(frontmatter)) {
    let value: string
    if (raw == null) value = ''
    else if (typeof raw === 'string') value = raw
    else if (
      typeof raw === 'number' ||
      typeof raw === 'boolean' ||
      typeof raw === 'bigint'
    )
      value = String(raw)
    else value = JSON.stringify(raw)
    out.push({ key, value })
  }
  return out
}
