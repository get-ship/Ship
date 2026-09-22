<controller>
// Hub version of `apps/shell/routes/studio/skills/[id]/controller.ts` —
// the surface is byte-identical to the shell (the studio's skill-detail
// view is one of the surfaces this job ports verbatim) EXCEPT the
// client seam: the shell pulls `ship` from its `../../../../ops`
// wrapper, the hub pulls `ops` from the AMBIENT `ctx.ops`
// injected by the host's `entry.gen.ts` (deck-runtime maps
// `clients.<name>` into view scope). The relative `src/lib/client`
// import is gone — the client lives in the view mount context, not a
// per-view module path.
//
// skills.* has no live mux resolver (see live_resolvers.rs), so BOTH
// the detail read and the frontmatter/body writes go through `ops`
// (createShipConnectClient); the read decodes the `{ json }` envelope.
// (The hub's live SSE client `ship` is reserved for ops that have a
// resolver — agents.summaries, collections.list, …)
import { controller, signal, derived, live, effect } from '@ship/sail-runtime'
import type { LiveSource, ViewScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'

interface SkillDetail {
  id: string
  name: string
  description: string
  kind: string
  source: string
  tags: string[]
  authors: string[]
  variables: string[]
  files: string[]
  body: string
}
type LiveResult = { kind: string; value?: SkillDetail }
type Banner = { ok: boolean; msg: string } | null
type Section = 'identity' | 'description' | 'tags' | 'variables' | 'files'

const SOURCE_LABELS: Record<string, string> = {
  'PROJECT SKILLS': 'Project',
  'USER SKILLS': 'User',
  LIBRARY: 'Library',
}
const sourceLabel = (s: string): string => SOURCE_LABELS[s] ?? s

// Parse the comma/whitespace tag input into a deduped, appended list.
const appendTags = (input: string, current: string[]): string[] => {
  const raw = input.trim().replace(/,+$/, '').trim()
  if (!raw) return current
  const next = [...current]
  for (const p of raw.split(',').map((t) => t.trim()).filter(Boolean)) if (!next.includes(p)) next.push(p)
  return next
}
export default controller((ctx: ViewScope<Scope>) => {

    // AMBIENT client (host-owned): the entry.gen.ts bootstrap constructs
    // `ops` from `src/lib/client.ts` and injects it as `clients.ops`.
    const ops = ctx.ops
    const { route } = ctx
    const skillId = derived(() => route.params.id() ?? '')
    const skillRes = live(() =>
      ops.skills.detail({ id: skillId() || '__none__' }) as unknown as LiveSource<SkillDetail>,
    )
    const sk = derived<SkillDetail | null>(() => {
      const r = skillRes() as LiveResult
      // Unary op → `{ json }` envelope; decode to the SkillDetail payload.
      const v = r.value as SkillDetail & { json?: string }
      if (typeof v.json === 'string') {
        try { return JSON.parse(v.json) as SkillDetail } catch { return null }
      }
      return v
    })
    const loading = derived(() => (skillRes() as LiveResult).kind !== 'ok')

    const skName = derived(() => sk()?.name ?? '')
    const skSourceLabel = derived(() => sourceLabel(sk()?.source ?? ''))
    const skVariables = derived<string[]>(() => sk()?.variables ?? [])
    const skFiles = derived<string[]>(() => sk()?.files ?? [])
    const skAuthorsText = derived(() => (sk()?.authors ?? []).join(', '))
    const hasBody = derived(() => (sk()?.body ?? '').length > 0)

    // Editor + per-section edit state.
    const draft = signal('')
    const baseline = signal('')
    const dirty = derived(() => draft() !== baseline())
    const saving = signal(false)
    const saveBanner = signal<Banner>(null)

    const descDraft = signal('')
    const descBaseline = signal('')
    const descDirty = derived(() => descDraft() !== descBaseline())
    const descSaving = signal(false)
    const descBanner = signal<Banner>(null)

    const tagsList = signal<string[]>([])
    const tagsBaseline = signal<string[]>([])
    const tagsInput = signal('')
    const tagsDirty = derived(() => {
      const a = tagsList(), b = tagsBaseline()
      return a.length !== b.length || a.some((t, i) => t !== b[i])
    })
    const tagsSaving = signal(false)
    const tagsBanner = signal<Banner>(null)

    // Seed editor + metadata drafts whenever a new skill loads.
    let seededId = ''
    effect(() => {
      const s = sk()
      if (s && s.id !== seededId) {
        seededId = s.id
        draft.set(s.body ?? ''); baseline.set(s.body ?? '')
        descDraft.set(s.description ?? ''); descBaseline.set(s.description ?? '')
        tagsList.set([...(s.tags ?? [])]); tagsBaseline.set([...(s.tags ?? [])])
      }
    })

    // Collapsible open state (in-memory; description + tags open by default).
    const open = signal<Record<Section, boolean>>({
      identity: true, description: true, tags: true, variables: false, files: false,
    })
    const openAttr = (s: Section) => derived(() => (open()[s] ? true : null))

    async function putFrontmatter(p: { description?: string; tags?: string[] }): Promise<Banner> {
      const id = skillId(); const s = sk()
      if (!id || !s) return { ok: false, msg: 'Skill not loaded.' }
      const fm: Record<string, unknown> = { name: s.name, description: p.description ?? s.description ?? '' }
      const tags = p.tags ?? s.tags ?? []
      if (tags.length) fm.tags = tags
      if ((s.authors ?? []).length) fm.authors = s.authors
      try {
        await ops.skills.writeFrontmatter({ id, frontmatterJson: JSON.stringify(fm), scope: '' })
        return { ok: true, msg: 'Saved.' }
      } catch (e) {
        return { ok: false, msg: (e as Error).message ?? 'Save failed.' }
      }
    }

    
    const toggleSection = (name: string, isOpen: boolean) => {
      open = { ...open, [name]: isOpen }
    }
    const onEditorInput = (e: Event) => {
      const d = (e as CustomEvent<{ value?: string }>).detail
      const v = d && typeof d.value === 'string' ? d.value : (e.target as HTMLElement & { value?: string }).value
      if (typeof v === 'string') draft = v
    }
    const save = async (s) => {
      const id = s.skillId
      if (!id || s.saving) return
      s.saving = true; s.saveBanner = null
      try {
        await ops.skills.writeBody({ id, body: s.draft, scope: '' })
        s.baseline = s.draft
        s.saveBanner = { ok: true, msg: 'Saved.' }
      } catch (e) {
        s.saveBanner = { ok: false, msg: (e as Error).message ?? 'Save failed.' }
      } finally {
        s.saving = false
      }
    }
    const onDescInput = (e: Event) => { descDraft = (e.target as HTMLTextAreaElement).value ?? '' }
    const resetDesc = (s) => { s.descDraft = s.descBaseline; s.descBanner = null }
    const saveDesc = async (s) => {
      if (s.descSaving || !s.descDirty) return
      s.descSaving = true; s.descBanner = null
      const r = await s.putFrontmatter({ description: s.descDraft, tags: s.tagsBaseline })
      if (r && r.ok) { s.descBaseline = s.descDraft; s.descBanner = { ok: true, msg: 'Saved.' } }
      else s.descBanner = r
      s.descSaving = false
    }
    const onTagsInput = (e: Event) => { tagsInput = (e.target as HTMLInputElement).value }
    const addTag = (s) => {
      const next = appendTags(s.tagsInput, s.tagsList)
      if (next !== s.tagsList) { s.tagsList = next; s.tagsInput = '' }
    }
    const onTagsKeydown = (e: KeyboardEvent) => {
      if (e.key !== 'Enter' && e.key !== ',') return
      e.preventDefault()
      const next = appendTags(tagsInput, tagsList)
      if (next !== tagsList) { tagsList = next; tagsInput = '' }
    }
    const removeTag = (t: string) => { tagsList = tagsList.filter((x: string) => x !== t) }
    const resetTags = (s) => { s.tagsList = [...s.tagsBaseline]; s.tagsInput = ''; s.tagsBanner = null }
    const saveTags = async (s) => {
      if (s.tagsSaving || !s.tagsDirty) return
      if (s.tagsInput.trim()) {
        const raw = s.tagsInput.trim().replace(/,+$/, '').trim()
        const next = [...s.tagsList]
        for (const p of raw.split(',').map((t: string) => t.trim()).filter(Boolean)) if (!next.includes(p)) next.push(p)
        s.tagsList = next; s.tagsInput = ''
      }
      s.tagsSaving = true; s.tagsBanner = null
      const next = s.tagsList
      const r = await s.putFrontmatter({ description: s.descBaseline, tags: next })
      if (r && r.ok) { s.tagsBaseline = [...next]; s.tagsBanner = { ok: true, msg: 'Saved.' } }
      else s.tagsBanner = r
      s.tagsSaving = false
    }
    return {
      skillId, sk, loading, skName, skSourceLabel, skVariables, skFiles, skAuthorsText, hasBody,
      draft, baseline, dirty, saving, saveBanner,
      descDraft, descBaseline, descDirty, descSaving, descBanner,
      tagsList, tagsBaseline, tagsInput, tagsDirty, tagsSaving, tagsBanner,
      open, putFrontmatter,
      openIdentityAttr: openAttr('identity'),
      openDescriptionAttr: openAttr('description'),
      openTagsAttr: openAttr('tags'),
      openVariablesAttr: openAttr('variables'),
      openFilesAttr: openAttr('files'), toggleSection, onEditorInput, save, onDescInput, resetDesc, saveDesc, onTagsInput, addTag, onTagsKeydown, removeTag, resetTags, saveTags }
})
</controller>

<div data-test="studio-view">
  <ship-studio-rail></ship-studio-rail>
  <main data-test="studio-content">
    <if cond={loading()}>
      <div><spinner size="md" centered></spinner></div>
    <else-if cond={!sk()}>
      <div>
        <empty-state icon="search-x" variant="default" size="md" data-test="skill-not-found">
          <span slot="title">No such skill</span>
          <span>{'No skill with id "' + skillId() + '" was found.'}</span>
        </empty-state>
      </div>
    </else-if><else>
      <div data-test="skill-detail">

        <!-- Editor pane -->
        <section>
          <div>
            <h1>{skName()}</h1>
            <div>
              <if cond={saveBanner()}>
                <span data-ok={saveBanner().ok}>{saveBanner().msg}</span>
              </if>
              <button tone="primary" variant="solid" size="sm" data-test="skill-save"
                disabled={!dirty() || saving()} on:click={save}>
                <icon slot="leading" name="save" size="xs"></icon>
                {saving() ? 'Saving…' : dirty() ? 'Save' : 'Saved'}
              </button>
            </div>
          </div>
          <div>
            <if cond={hasBody() || dirty()}>
              <ship-markdown-editor data-test="skill-editor" full-height
                value={draft()}
                on:sail-change={onEditorInput}
                on:input={onEditorInput}></ship-markdown-editor>
            <else>
              <div>
                <icon name="file" size="xl" tone="muted"></icon>
                <span>This skill has no SKILL.md content yet — start typing to add it.</span>
                <ship-markdown-editor data-test="skill-editor" full-height
                  value={draft()}
                  on:sail-change={onEditorInput}
                  on:input={onEditorInput}></ship-markdown-editor>
              </div>
            </else></if>
          </div>
        </section>

        <!-- Metadata panel -->
        <aside aria-label="Skill metadata">
          <collapsible open={openIdentityAttr()}
            on:sail-toggle={(e) => toggleSection('identity', e.detail.open)}>
            <div slot="trigger">
              <icon name="id-card" size="xs" tone="muted"></icon><span>Identity</span>
              <icon name="chevron-down" size="xs" tone="muted"></icon>
            </div>
            <div>
              <div><span>id</span><code>{skillId()}</code></div>
              <div><span>kind</span><span>{sk().kind}</span></div>
              <div><span>source</span><span>{skSourceLabel()}</span></div>
              <if cond={(sk().authors || []).length > 0}>
                <div><span>authors</span><span>{skAuthorsText()}</span></div>
              </if>
            </div>
          </collapsible>

          <collapsible open={openDescriptionAttr()}
            on:sail-toggle={(e) => toggleSection('description', e.detail.open)}>
            <div slot="trigger">
              <icon name="info" size="xs" tone="muted"></icon><span>Description</span>
              <icon name="chevron-down" size="xs" tone="muted"></icon>
            </div>
            <div>
              <textarea data-test="desc-edit" rows="4"
                placeholder="What this skill does (one-liner shown in the list)…"
                disabled={descSaving()} value={descDraft()} on:input={onDescInput}></textarea>
              <div>
                <span>{descDirty() ? 'Unsaved' : 'Synced'}</span>
                <button variant="ghost" size="sm" disabled={!descDirty() || descSaving()} on:click={resetDesc}>Cancel</button>
                <button variant="solid" tone="primary" size="sm" data-test="desc-save"
                  disabled={!descDirty() || descSaving()} on:click={saveDesc}>{descSaving() ? 'Saving…' : 'Save'}</button>
              </div>
              <if cond={descBanner()}>
                <alert tone={descBanner().ok ? 'success' : 'destructive'}>{descBanner().msg}</alert>
              </if>
            </div>
          </collapsible>

          <collapsible open={openTagsAttr()}
            on:sail-toggle={(e) => toggleSection('tags', e.detail.open)}>
            <div slot="trigger">
              <icon name="tag" size="xs" tone="muted"></icon><span>Tags</span>
              <span>{tagsList().length}</span>
              <icon name="chevron-down" size="xs" tone="muted"></icon>
            </div>
            <div>
              <if cond={tagsList().length > 0}>
                <div data-test="tag-chips">
                  <for as="t" of={tagsList()} key={t}>
                    <span>{t}<button type="button" aria-label="Remove tag" on:click={() => removeTag(t)}><icon name="x" size="xs"></icon></button></span>
                  </for>
                </div>
              </if>
              <input type="text" placeholder="Add tag, Enter to commit…"
                value={tagsInput()} on:input={onTagsInput} on:keydown={onTagsKeydown} />
              <div>
                <span>{tagsDirty() ? 'Unsaved' : 'Synced'}</span>
                <button variant="ghost" size="sm" disabled={!tagsDirty() || tagsSaving()} on:click={resetTags}>Cancel</button>
                <button variant="solid" tone="primary" size="sm" data-test="tags-save"
                  disabled={!tagsDirty() || tagsSaving()} on:click={saveTags}>{tagsSaving() ? 'Saving…' : 'Save'}</button>
              </div>
              <if cond={tagsBanner()}>
                <alert tone={tagsBanner().ok ? 'success' : 'destructive'}>{tagsBanner().msg}</alert>
              </if>
            </div>
          </collapsible>

          <if cond={skVariables().length > 0}>
            <collapsible open={openVariablesAttr()}
              on:sail-toggle={(e) => toggleSection('variables', e.detail.open)}>
              <div slot="trigger">
                <icon name="braces" size="xs" tone="muted"></icon><span>Variables</span>
                <span>{skVariables().length}</span>
                <icon name="chevron-down" size="xs" tone="muted"></icon>
              </div>
              <div><div><for as="v" of={skVariables()} key={v}><code>{v}</code></for></div></div>
            </collapsible>
          </if>

          <if cond={skFiles().length > 0}>
            <collapsible open={openFilesAttr()}
              on:sail-toggle={(e) => toggleSection('files', e.detail.open)}>
              <div slot="trigger">
                <icon name="file" size="xs" tone="muted"></icon><span>Files</span>
                <span>{skFiles().length}</span>
                <icon name="chevron-down" size="xs" tone="muted"></icon>
              </div>
              <div><div><for as="f" of={skFiles()} key={f}><code>{f}</code></for></div></div>
            </collapsible>
          </if>
        </aside>

      </div>
    </else></if>
  </main>
</div>
