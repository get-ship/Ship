<controller>
// <ship-skill-list> — @ship/skills · the hub `/studio/skills` index.
//
// Reads `ship.skills.list()` (Connect SDK LiveSource) and renders the
// registered skills, grouped by `SkillScopeTag` (project vs user). The
// per-scope labels are stable public facts (matches the daemon's
// ScopeTag enum, projected to display strings); the count sub-line
// mirrors the shell's "N skills" pattern.
//
// The shell's controller reads `ship.skills.summaries()` (a
// pre-aggregated tree); the hub uses `ship.skills.list()` directly so
// the controller owns the grouping + filtering and the template owns
// only the layout. The list payload is typed (`{ items:
// SkillIndexEntry[] }` per ship-sdk.ts); the controller narrows
// `unknown` to that shape defensively (the items field is `unknown`
// to the runtime's LiveSignal envelope until narrowed, and each
// entry's string fields are coalesced against missing values).
//
// On loading + error + zero skills the component renders the
// respective empty-state. The shell's filter (q + kind) is not in
// this surface — the job spec narrows `/studio/skills` to "list the
// live skill list" + `/studio/skills/[id]` for detail; filter is
// YAGNI for this pass.

import { controller, derived, live } from '@ship/sail-runtime'
import type { LiveSource, ViewScope } from '@ship/sail-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-skill-list.sail'

/** Wire shape of a single skill index entry.
 *  Mirrors the Rust `ship_agents::services::skills::state::IndexEntry`
 *  (see crates/xtask/src/types.rs — specta-registered as
 *  `SkillIndexEntry`). The controller narrows `unknown` to this shape
 *  defensively. */
export interface SkillIndexEntry {
  id: string
  /** 'project' | 'user' | 'library' — matches ScopeTag. */
  scope: string
  name: string
  description: string
}

/** Wire shape of `ship.skills.list()`. */
interface ListPayload {
  items: SkillIndexEntry[]
}

/** Stable display order + labels for the known scope values.
 *  Any scope the daemon adds in the future still renders (appended
 *  after the known order), but the listed scopes get stable,
 *  human-friendly labels. */
export const SCOPE_LABELS: Record<string, string> = {
  project: 'Project',
  user: 'User',
  library: 'Library',
}

export const SCOPE_ORDER: readonly string[] = ['project', 'user', 'library']

/** Narrow `unknown` (the LiveSignal envelope's ok.value) into the
 *  `ListPayload` shape. Returns `null` for missing / wrong-shape
 *  payloads so the template renders the empty-state without
 *  throwing. */
export function readList(value: unknown): ListPayload | null {
  if (!value || typeof value !== 'object') return null
  const items = (value as { items?: unknown }).items
  if (!Array.isArray(items)) return null
  const out: SkillIndexEntry[] = []
  for (const entry of items) {
    if (!entry || typeof entry !== 'object') continue
    const e = entry as Record<string, unknown>
    out.push({
      id: typeof e.id === 'string' ? e.id : '',
      scope: typeof e.scope === 'string' ? e.scope : 'project',
      name:
        typeof e.name === 'string'
          ? e.name
          : typeof e.id === 'string'
            ? e.id
            : '',
      description: typeof e.description === 'string' ? e.description : '',
    })
  }
  return { items: out }
}

/** A single grouped bucket the template renders as a section. */
export interface SkillGroup {
  /** Scope key (e.g. 'project') — drives the section label + a
   *  data-scope attribute for test selectors. */
  id: string
  /** Display label (e.g. 'Project'). */
  label: string
  /** The skills in this group, in the order returned by the daemon. */
  skills: SkillIndexEntry[]
}

/** State the template reads.
 *  - `'loading'`  — initial / reconnecting; render the spinner.
 *  - `'error'`    — live() reported an error; render the error-state.
 *  - `'empty'`    — ok envelope, zero skills; render the empty-state.
 *  - `'ready'`    — ok envelope, one or more skills; render the list. */
export type ListState = 'loading' | 'error' | 'empty' | 'ready'

export default controller({
  scope: (_ctx: ViewScope<Scope>, host?: Element) => {
    const rail = live(
      () => ship.skills.list() as unknown as LiveSource<ListPayload>,
    )

    const groups = derived<SkillGroup[]>(() => {
      const r = rail()
      if (r.kind !== 'ok') return []
      const list = readList(r.value)
      if (!list) return []
      const byScope = new Map<string, SkillIndexEntry[]>()
      for (const s of list.items) {
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
      // Any scope the daemon adds in the future (and the map doesn't
      // name) still renders — appended after the known order.
      for (const [id, skills] of byScope) {
        if (SCOPE_ORDER.includes(id)) continue
        out.push({ id, label: SCOPE_LABELS[id] ?? id, skills })
      }
      return out
    })

    const total = derived<number>(() =>
      groups().reduce((n, g) => n + g.skills.length, 0),
    )

    /** Sub-line under the H1: "N skills" (or "1 skill"). */
    const sub = derived<string>(() => {
      const t = total()
      if (t === 0) return ''
      return t === 1 ? '1 skill' : `${t} skills`
    })

    const state = derived<ListState>(() => {
      const r = rail()
      if (r.kind === 'loading') return 'loading'
      if (r.kind === 'error') return 'error'
      if (total() === 0) return 'empty'
      return 'ready'
    })
    const onOpen = (e: MouseEvent, skillId: string): void => {
      e.preventDefault()
      host?.dispatchEvent(
        new CustomEvent('ship-skill-open', {
          detail: { id: skillId },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { rail, groups, sub, state, total, onOpen }
  },
})
</controller>

<!--
  <ship-skill-list> — @ship/skills · the hub `/studio/skills` index.

  Reads `ship.skills.list()` (Connect SDK LiveSource) and renders the
  registered skills, grouped by SkillScopeTag (project/user/library).
  Empty/loading/error states are first-class; this is the live hub
  surface (no BACKEND-GAP — `ship.skills.list()` exists in the SDK,
  see packages/sdk-client/src/ship-sdk.ts:211).

  Composes
    <column>        section stack
    <row>      meta row
    <card>         per-skill card
    <icon>         affordance glyphs
    <badge>        scope pill
    <empty-state>  no-skills / error fallback
    <spinner>      loading indicator

  Prop / event
  ─────────────────────────────────────────────────────────────────────
  (no props — the pack reads the SDK directly)

  Emits:
    ship-skill-open   { detail: { id: string } }   click on a skill card
-->

<!-- style block removed: use sx / external CSS -->

<section data-test="skills-head">
  <h1>Skills</h1>
  <span>{sub()}</span>
  <span aria-hidden="true"></span>
</section>

<if cond={state() === 'loading'}>
  <div data-test="skills-loading">
    <spinner size="md" centered></spinner>
  </div>
<else-if cond={state() === 'error'}>
  <empty-state variant="default" size="md" tone="muted" data-test="skills-error">
    <icon slot="icon" name="alert-triangle"></icon>
    <span slot="title">Couldn't load skills</span>
    The skill list couldn't be read right now. Reload to try again.
  </empty-state>
</else-if><else-if cond={state() === 'empty'}>
  <empty-state variant="default" size="md" tone="muted" data-test="skills-empty">
    <icon slot="icon" name="sparkles"></icon>
    <span slot="title">No skills registered</span>
    Skills are folders with a <code>SKILL.md</code> file. Create one in
    your project or user skills directory to see it here.
  </empty-state>
</else-if><else>
  <column data-test="skills-groups" gap="5">
    <for as="g" of={groups()} key={g.id}>
      <section data-test="skills-group" data-scope={g.id}>
        <header>
          {g.label} <span>· {g.skills.length}</span>
        </header>
        <column data-test="skills-group-list" gap="2">
          <for as="s" of={g.skills} key={s.id}>
            <a data-test="skill-card" data-id={s.id}
              href={'/studio/skills/' + s.id}
              on:click={(e) => onOpen(e, s.id)}>
              <column gap="1">
                <row gap="2">
                  <span>{s.name}</span>
                  <badge tone="muted" variant="outline" size="sm">{g.label}</badge>
                </row>
                <if cond={s.description}>
                  <p>{s.description}</p>
                </if>
              </column>
              <icon name="chevron-right" size="sm" tone="muted"></icon>
            </a>
          </for>
        </column>
      </section>
    </for>
  </column>
</else></if>
