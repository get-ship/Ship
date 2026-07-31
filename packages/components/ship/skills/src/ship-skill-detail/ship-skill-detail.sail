<controller>
// <ship-skill-detail> — @ship/skills · the hub `/studio/skills/:id` viewer.
//
// Reads `ship.skills.detail({ id })` (Connect SDK LiveSource) and
// renders the skill's frontmatter, body, and files. The `:id` route
// param flows through the hub route controller as the `id` prop.
//
// The detail payload shape (Rust `SkillDetail` from
// crates/core/ship-platform/crates/ship-agents/src/services/skills/
// state.rs) is `{ id, scope, frontmatter, body, files }`. The SDK
// types the op as `LiveSource<unknown>`, so the controller narrows
// `unknown` to that shape defensively (matching the list's pattern).
//
// The shell's detail page is an editor (markdown body + metadata
// section panels + save). The hub surface is read-only — the spec
// narrows `/studio/skills/[id]` to "renders the detail" and the
// shell already owns the write path. Adding an editor here is YAGNI
// for this pass.
//
// Empty/missing state: an `ok` envelope whose payload is null OR
// whose id doesn't match the requested id renders the
// "no such skill" empty-state. Loading renders the spinner. Error
// renders the error-state.

import { controller, derived, live } from '@ship/sail-runtime'
import type { LiveSource, Signal, ViewScope } from '@ship/sail-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-skill-detail.sail'

/** Wire shape of `ship.skills.detail({ id })`.
 *  Mirrors the Rust `SkillDetail` struct from
 *  ship_agents::services::skills::state::SkillDetail.
 *  `frontmatter` is a free-form object (the SKILL.md YAML frontmatter,
 *  parsed by the daemon). */
export interface SkillDetail {
  id: string
  scope: string
  /** Free-form frontmatter object. */
  frontmatter: Record<string, unknown>
  /** Markdown body (the SKILL.md content under the frontmatter). */
  body: string
  /** Companion files relative to the skill dir (e.g. `assets/foo.png`). */
  files: string[]
}

/** Stable display order + labels for the known scope values. */
const SCOPE_LABELS: Record<string, string> = {
  project: 'Project',
  user: 'User',
  library: 'Library',
}

/** Narrow `unknown` to the `SkillDetail` shape. Returns `null` when
 *  the payload is missing or wrong-shaped. A payload whose id does
 *  NOT match the requested id also returns `null` — the daemon
 *  returns null/404 for missing ids, but a stale id race could
 *  surface a different entry; we treat any mismatch as missing. */
export function readDetail(value: unknown, requestedId: string): SkillDetail | null {
  if (!value || typeof value !== 'object') return null
  const v = value as Record<string, unknown>
  const id = typeof v.id === 'string' ? v.id : ''
  if (id !== requestedId) return null
  const scope = typeof v.scope === 'string' ? v.scope : 'project'
  const frontmatter =
    v.frontmatter && typeof v.frontmatter === 'object' && !Array.isArray(v.frontmatter)
      ? (v.frontmatter as Record<string, unknown>)
      : {}
  const body = typeof v.body === 'string' ? v.body : ''
  const files: string[] = []
  if (Array.isArray(v.files)) {
    for (const f of v.files) {
      if (typeof f === 'string') files.push(f)
    }
  }
  return { id, scope, frontmatter, body, files }
}

/** A frontmatter row for the meta panel — scalars render as their
 *  string form; objects/arrays get a JSON-encoded display fallback
 *  so they don't render as `[object Object]`. */
export interface FrontmatterRow {
  key: string
  value: string
}

/** State the template reads.
 *  - `'loading'`  — initial / reconnecting; render the spinner.
 *  - `'error'`    — live() reported an error; render the error-state.
 *  - `'missing'`  — ok envelope, no skill with the requested id;
 *                   render the "no such skill" empty-state.
 *  - `'ready'`    — ok envelope, skill present; render the detail. */
export type DetailState = 'loading' | 'error' | 'missing' | 'ready'

export default controller(({ id }: ViewScope<Scope> & { id: Signal<string> }, host?: Element) => {

    const rail = live(
      () =>
        ship.skills.detail({ id: id() }) as unknown as LiveSource<SkillDetail | null>,
    )

    const detail = derived<SkillDetail | null>(() => {
      const r = rail()
      if (r.kind !== 'ok') return null
      return readDetail(r.value, id())
    })

    const scopeLabel = derived<string>(() => {
      const d = detail()
      if (!d) return ''
      return SCOPE_LABELS[d.scope] ?? d.scope
    })

    /** Frontmatter rows the meta panel renders. Stable key order
     *  (object insertion order); scalars render as their string
     *  form, non-scalars JSON-encode. */
    const frontmatterRows = derived<FrontmatterRow[]>(() => {
      const d = detail()
      if (!d) return []
      const out: FrontmatterRow[] = []
      for (const [key, raw] of Object.entries(d.frontmatter)) {
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
    })

    const state = derived<DetailState>(() => {
      const r = rail()
      if (r.kind === 'loading') return 'loading'
      if (r.kind === 'error') return 'error'
      if (!detail()) return 'missing'
      return 'ready'
    })

    const onBack = (e: MouseEvent): void => {
      e.preventDefault()
      host?.dispatchEvent(
        new CustomEvent('ship-skill-back', { bubbles: true, composed: true }),
      )
    }

    return { id, rail, detail, scopeLabel, frontmatterRows, state, onBack }
})
</controller>

<!--
  <ship-skill-detail> — @ship/skills · the hub `/studio/skills/:id` viewer.

  Reads `ship.skills.detail({ id })` (Connect SDK LiveSource) and
  renders the skill's frontmatter, body, and files. The `:id` route
  param flows through the hub route controller as the `id` prop.

  Loading/missing/error states are first-class. The shell's detail
  page is a full editor (markdown body + metadata section panels +
  save). The hub surface is read-only — the spec narrows
  `/studio/skills/[id]` to "renders the detail"; the shell already
  owns the write path, so adding an editor here is a YAGNI for this
  pass.

  Composes
    <column>         section stack
    <row>       meta row
    <icon>          affordance glyphs
    <badge>         scope pill
    <sail-markdown>      body renderer (read-only)
    <empty-state>   missing / error fallback
    <spinner>       loading indicator

  Prop / event
  ─────────────────────────────────────────────────────────────────────
  id   (string, '')   skill id from the route (the `:id` param)

  Emits:
    ship-skill-back   {}   click on the back link
-->

<!-- <prop name="id" type="string" default="" /> -->

<!-- style block removed: use sx / external CSS -->

<a href="/studio/skills" data-test="skill-detail-back" on:click={onBack}>
  <icon name="chevron-left" size="xs"></icon>
  Skills
</a>

<if cond={state() === 'loading'}>
  <div data-test="skill-detail-loading">
    <spinner size="md" centered></spinner>
  </div>
<else-if cond={state() === 'error'}>
  <empty-state variant="default" size="md" tone="muted" data-test="skill-detail-error">
    <icon slot="icon" name="alert-triangle"></icon>
    <span slot="title">Couldn't load skill</span>
    The skill detail couldn't be read right now. Reload to try again.
  </empty-state>
</else-if><else-if cond={state() === 'missing'}>
  <empty-state variant="default" size="md" tone="muted" data-test="skill-detail-missing">
    <icon slot="icon" name="search-x"></icon>
    <span slot="title">No such skill</span>
    <span>{'No skill with id "' + id() + '" was found.'}</span>
  </empty-state>
</else-if><else>
  <header data-test="skill-detail-head">
    <span aria-hidden="true"><icon name="sparkles" size="md"></icon></span>
    <div>
      <h1 data-test="skill-detail-title">{detail().name || detail().id}</h1>
      <div data-test="skill-detail-meta">
        <code>{detail().id}</code>
        <span aria-hidden="true">·</span>
        <badge tone="muted" variant="outline" size="sm">{scopeLabel()}</badge>
      </div>
      <div aria-hidden="true"></div>
    </div>
  </header>

  <column data-test="skill-detail-body" gap="5">
    <if cond={frontmatterRows().length > 0}>
      <section data-test="skill-detail-frontmatter">
        <h2>Frontmatter · {frontmatterRows().length}</h2>
        <column gap="0">
          <for as="row" of={frontmatterRows()} key={row.key}>
            <div>
              <span>{row.key}</span>
              <span>{row.value}</span>
            </div>
          </for>
        </column>
      </section>
    </if>

    <section data-test="skill-detail-content">
      <if cond={detail().body}>
        <sail-markdown value={detail().body}></sail-markdown>
      <else>
        <empty-state variant="inline" size="sm" tone="muted" data-test="skill-detail-no-body">
          <icon slot="icon" name="file"></icon>
          <span slot="title">No body</span>
          This skill has no SKILL.md content yet.
        </empty-state>
      </else></if>
    </section>

    <if cond={detail().files.length > 0}>
      <section data-test="skill-detail-files">
        <h2>Files · {detail().files.length}</h2>
        <div>
          <for as="f" of={detail().files} key={f}>
            <code>{f}</code>
          </for>
        </div>
      </section>
    </if>
  </column>
</else></if>
