<scope>
  props: {
    id = ''
    view: 'guided' | 'raw' = 'guided'
  }
</scope>
<controller>
import { controller, derived, onCleanup, query, signal, unwrapOr } from '@ship/sail-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-schema-editor.sail'
import type { FieldDef, FieldType } from '../schema'

// <ship-schema-editor> — author/edit a collection schema, guided +
// raw surfaces, with the reproject warning the spec requires.
//
// State is a list of FieldDefWithUid (uid is the local key for the
// field loop, not a schema field). On save:
//   1. If the proposed field set drops or renames a name vs the
//      original, the reproject warning flips on and save bails —
//      the user clicks save a second time to acknowledge.
//   2. The CreateSchema / UpdateSchema RPC fires; the host can
//      listen for ship-schema-save { id, fields }.
//
// Field mutations route through a single delegated listener on the
// host (matches the @ship/spaces switcher's pattern). The listener
// walks composedPath() to find the right input/button by data-* and
// the per-row uid; that keeps the templates free of `$event` (which
// isn't part of the SUI runtime's documented handler-arg surface).

interface FieldDefWithUid extends FieldDef {
  uid: string
}

interface DetailShape {
  schema: {
    name: string
    fields: FieldDef[]
    builtin?: boolean
  }
  items: unknown[]
  total_items: number
}

const FIELD_TYPES: readonly FieldType[] = ['text', 'integer', 'real', 'bool', 'timestamp', 'json']

function isFieldType(v: unknown): v is FieldType {
  return typeof v === 'string' && (FIELD_TYPES as readonly string[]).includes(v)
}

let _uid = 0
const nextUid = (): string => `f${++_uid}`

export default controller(({ id }: Scope, host?: Element) => {

    const detail = query<DetailShape | null>(() => {
      const cid = id()
      if (!cid) return Promise.resolve(null)
      return ship.collections.detail({ id: cid })
    })

    const fields = signal<FieldDefWithUid[]>([])
    const initialized = signal(false)
    const saving = signal(false)
    const acknowledgedReproject = signal(false)

    const detailValue = derived<DetailShape | null>(() => {
      const v = unwrapOr(detail, null) as DetailShape | null
      if (v && !initialized()) {
        fields.set(
          (v.schema?.fields ?? []).map((f) => ({ ...f, uid: nextUid() })),
        )
        initialized.set(true)
        acknowledgedReproject.set(false)
      }
      return v
    })

    const needsReproject = derived<boolean>(() => {
      if (!initialized()) return false
      const original = new Set((detailValue()?.schema?.fields ?? []).map((f) => f.name))
      const proposed = new Set(fields().map((f) => f.name).filter((n) => n.length > 0))
      for (const name of original) if (!proposed.has(name)) return true
      return false
    })

    const reprojectWarning = derived<boolean>(() => needsReproject() && !acknowledgedReproject())

    const rawJson = derived<string>(() => JSON.stringify({
      title: detailValue()?.schema?.name ?? id(),
      fields: fields().map(({ uid: _u, ...rest }) => rest),
    }, null, 2))

    function addField(): void {
      fields.set([...fields(), { uid: nextUid(), name: '', type: 'text' }])
    }
    function removeField(uid: string): void {
      fields.set(fields().filter((f) => f.uid !== uid))
    }

    // Event delegation. Each input carries data-test="name"|"type"|... and
    // a data-uid="<field.uid>"; the delegated handler routes by test name.
    const onHostEvent = (e: Event): void => {
      if (!host) return
      for (const node of e.composedPath()) {
        if (!(node instanceof HTMLElement)) continue
        const el = node as HTMLElement
        if (el.dataset.test === 'add' && e.type === 'click') { addField(); return }
        if (el.dataset.test === 'remove' && e.type === 'click') {
          const row = el.closest('[data-test="field"]') as HTMLElement | null
          const uid = row?.dataset.uid
          if (uid) removeField(uid)
          return
        }
        if (el.dataset.test === 'save' && e.type === 'click') { void save(); return }
        if (el.dataset.test === 'view-guided' && e.type === 'click') { host.setAttribute('view', 'guided'); return }
        if (el.dataset.test === 'view-raw' && e.type === 'click') { host.setAttribute('view', 'raw'); return }
        if (el.dataset.test === 'name' && (e.type === 'input' || e.type === 'change')) {
          const row = el.closest('[data-test="field"]') as HTMLElement | null
          const uid = row?.dataset.uid
          if (uid) {
            const value = (el as HTMLInputElement).value
            fields.set(fields().map((f) => f.uid === uid ? { ...f, name: value } : f))
          }
          return
        }
        if (el.dataset.test === 'type' && e.type === 'change') {
          const row = el.closest('[data-test="field"]') as HTMLElement | null
          const uid = row?.dataset.uid
          if (uid) {
            const value = (el as HTMLSelectElement).value
            const next: FieldType = isFieldType(value) ? value : 'text'
            fields.set(fields().map((f) => f.uid === uid ? { ...f, type: next } : f))
          }
          return
        }
        if (el.dataset.test === 'required' && e.type === 'change') {
          const row = el.closest('[data-test="field"]') as HTMLElement | null
          const uid = row?.dataset.uid
          const input = el.matches('input') ? el : el.querySelector('input')
          if (uid && input) fields.set(fields().map((f) => f.uid === uid ? { ...f, required: (input as HTMLInputElement).checked } : f))
          return
        }
        if (el.dataset.test === 'indexed' && e.type === 'change') {
          const row = el.closest('[data-test="field"]') as HTMLElement | null
          const uid = row?.dataset.uid
          const input = el.matches('input') ? el : el.querySelector('input')
          if (uid && input) fields.set(fields().map((f) => f.uid === uid ? { ...f, indexed: (input as HTMLInputElement).checked } : f))
          return
        }
        if (el.dataset.test === 'unique' && e.type === 'change') {
          const row = el.closest('[data-test="field"]') as HTMLElement | null
          const uid = row?.dataset.uid
          const input = el.matches('input') ? el : el.querySelector('input')
          if (uid && input) fields.set(fields().map((f) => f.uid === uid ? { ...f, unique: (input as HTMLInputElement).checked } : f))
          return
        }
      }
    }
    host?.addEventListener('click', onHostEvent)
    host?.addEventListener('change', onHostEvent)
    host?.addEventListener('input', onHostEvent)
    onCleanup(() => {
      host?.removeEventListener('click', onHostEvent)
      host?.removeEventListener('change', onHostEvent)
      host?.removeEventListener('input', onHostEvent)
    })

    async function save(): Promise<void> {
      if (needsReproject() && !acknowledgedReproject()) {
        acknowledgedReproject.set(true)
        return
      }
      const cleaned: FieldDef[] = fields()
        .filter((f) => f.name.length > 0)
        .map((f) => {
          const out: FieldDef = { name: f.name, type: f.type }
          if (f.required) out.required = true
          if (f.indexed) out.indexed = true
          if (f.unique) out.unique = true
          if (f.default !== undefined) out.default = f.default
          return out
        })
      const title = detailValue()?.schema?.name ?? id()
      saving.set(true)
      try {
        if (id()) {
          await ship.collections.update_schema({ id: id(), fields: cleaned as Array<{ name: string; type: string; required?: boolean; values?: string[] }> })
        } else {
          await ship.collections.create({
            id: id() || title.toLowerCase().replace(/[^a-z0-9]+/g, '-'),
            name: title,
            fields: cleaned as Record<string, unknown>,
          })
        }
        host?.dispatchEvent(
          new CustomEvent('ship-schema-save', {
            detail: { id: id() || title, fields: cleaned },
            bubbles: true,
            composed: true,
          }),
        )
        acknowledgedReproject.set(false)
      } finally {
        saving.set(false)
      }
    }

    return {
      detail, detailValue, fields, needsReproject, reprojectWarning, rawJson, saving,
      addField, removeField, save,
    }
})
</controller>

<!--
  <ship-schema-editor> — @ship/collections · author/edit a
  collection's schema. Both a guided field-list and a raw editor
  surface (toggle). Calls ship.collections.create or
  ship.collections.update_schema via the controller; surfaces the
  reproject warning when an UpdateSchema change rebuilds the
  projection from the event log (data preserved, column dropped).

  Emits ship-schema-save { id, fields } on confirm. The reproject
  warning is part of the spec's hard requirement — see spec.md
  §"sui-schema-editor" and the errors fixture `reproject_warning`.

  Tag prefix `ship-*` marks this component as depending on the
  Ship runtime: mounting without a host that has called
  setShipClient() will throw on the first SDK access.

  Interactions route through a single delegated listener on the
  host (see controller). Each row carries data-uid and each
  interactive control carries data-test="..." so the listener
  routes by name — the template stays free of `$event`-style
  handler args.
-->
<!-- <prop name="id" type="string" default="" /> -->
<!-- <prop name="view" values="guided|raw" default="guided" reflect /> -->

<match {detail()}>
  {loading => <spinner shape="line" size="lg"></spinner>}
  {error as err => <error-state title="Couldn't load schema" variant="inline" body={err?.message ?? 'Request failed'}></error-state>}
  {ok as _ => }
</match>
<if {detail().kind === 'ok'}>
  <div>
    <h3 data-test="editor-title">{id() || 'New collection'}</h3>
    <div role="tablist">
      <button type="button" role="tab" data-test="view-guided" aria-pressed={view() === 'guided' ? 'true' : 'false'}>guided</button>
      <button type="button" role="tab" data-test="view-raw" aria-pressed={view() === 'raw' ? 'true' : 'false'}>raw</button>
    </div>
  </div>
  <div>
    <if {view() === 'raw'}>
      <textarea
        value={rawJson()}
        variant={'code'}
        rows={14}
        data-test="raw"
      ></textarea>
    {else =>
      <column gap="2" data-test="fields">
        <for {fields() as f} key={f.name || f.uid}>
          <div data-test="field" data-uid={f.uid}>
            <input
              type="text"
              placeholder="name"
              value={f.name}
              data-test="name"
            />
            <select
              value={f.type}
              data-test="type"
            >
              <option value="text">text</option>
              <option value="integer">integer</option>
              <option value="real">real</option>
              <option value="bool">bool</option>
              <option value="timestamp">timestamp</option>
              <option value="json">json</option>
            </select>
            <label data-test="required"><checkbox checked={f.required}></checkbox> req</label>
            <label data-test="indexed"><checkbox checked={f.indexed}></checkbox> idx</label>
            <label data-test="unique"><checkbox checked={f.unique}></checkbox> uniq</label>
            <button type="button" data-test="remove" aria-label="Remove field">×</button>
          </div>
        </for>
      </column>
       <div>
        <button variant="ghost" size="sm" type="button" data-test="add">+ Add field</button>
      </div>
    }</if>
    <if {reprojectWarning()}>
      <div data-test="reproject-warning" role="alert">
        <strong>Reproject</strong>
        Removing or renaming a field rebuilds the projection from the
        event log. Existing item data is preserved; the column is
        dropped from the table. Continue?
      </div>
    </if>
    <div>
      <button variant="solid" tone="primary" type="button" data-test="save" loading={saving() ? true : false}>{id() ? 'Save schema' : 'Create collection'}</button>
    </div>
  </div>
</if>
