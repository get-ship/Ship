// <ship-item-editor> — schema-generated form, client-side validation,
// Create/Update RPC. Form state lives as a single keyed Record so the
// template can read it via `formValues()[f.name]` — the renderable
// expression subset forbids function calls in templates, so per-field
// signals accessed by dynamic name don't work; a keyed Record does.

import { controller, derived, effect, live, signal, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-item-editor.sui'
import { validateFieldValue, type FieldDef } from '../schema'

type FormValue = string | number | boolean | null

interface FieldDefWithReq extends FieldDef {
  required?: boolean
}

interface SchemaShape {
  name: string
  fields: FieldDefWithReq[]
}

interface StatePayload {
  schema: SchemaShape
  total_items: number
}

const EMPTY_STATE: StatePayload = { schema: { name: '', fields: [] }, total_items: 0 }

// Coerce a string from a text/number input into the matching type
// the schema field expects. Mirrors the server-side parser so
// `validateFieldValue()` doesn't have to handle string vs number
// ambiguity. Returns the raw string for json (caller parses it).
function coerceFromString(text: string, type: string): FormValue {
  if (text === '') return null
  if (type === 'integer') {
    const n = Number(text)
    return Number.isFinite(n) ? Math.trunc(n) : null
  }
  if (type === 'real') {
    const n = Number(text)
    return Number.isFinite(n) ? n : null
  }
  if (type === 'bool') return text === 'true'
  return text
}

export default controller<Scope>({
  scope: ({ collection, id, mode }: ViewScope<Scope>, host) => {
    const state = live<unknown>(() => ship.collections.state({ active: collection() || undefined }))

    // Pre-existing item (edit mode) for prefilling.
    const existing = live<{ ok: true; data: { id: string; data: Record<string, unknown> } } | null>(() => {
      if (mode() !== 'edit') return Promise.resolve(null)
      const c = collection()
      const i = id()
      if (!c || !i) return Promise.resolve(null)
      return ship.collections.get_item({ collection: c, item_id: i })
    })

    const schema = derived<SchemaShape | null>(() => {
      const s = unwrapOr(state, EMPTY_STATE) as { schema?: SchemaShape }
      return s.schema ?? null
    })

    const fieldDefs = derived<FieldDefWithReq[]>(() => schema()?.fields ?? [])

    // Form state as a single keyed Record so the template can read
    // `formValues()[f.name]` (signal-read + property-access — both
    // allowed by the renderable expression subset).
    const formState = signal<Record<string, FormValue>>({})

    // Hydrate the form whenever the schema or the existing item settles.
    effect(() => {
      const s = schema()
      const ex = unwrapOr(existing, null) as { data: { data: Record<string, unknown> } } | null
      const next: Record<string, FormValue> = {}
      if (s) {
        for (const f of s.fields) {
          if (ex?.data && Object.prototype.hasOwnProperty.call(ex.data.data ?? {}, f.name)) {
            const raw = ex.data.data[f.name]
            if (raw === null || raw === undefined) next[f.name] = ''
            else if (f.type === 'bool') next[f.name] = Boolean(raw)
            else if (typeof raw === 'string' || typeof raw === 'number' || typeof raw === 'boolean') next[f.name] = raw
            else next[f.name] = JSON.stringify(raw)
          } else {
            next[f.name] = f.default === undefined ? '' : (f.default as FormValue)
          }
        }
      }
      formState.set(next)
    })

    // JSON-mode fields want a stringified form for the textarea. The
    // template reads `jsonValues()[f.name]` instead of calling jsonFor().
    const jsonValues = derived<Record<string, string>>(() => {
      const out: Record<string, string> = {}
      const vs = formState()
      for (const k of Object.keys(vs)) {
        const v = vs[k]
        if (v === null || v === '') { out[k] = ''; continue }
        out[k] = typeof v === 'string' ? v : JSON.stringify(v, null, 2)
      }
      return out
    })

    // Field flags as a derived Record so the template reads via
    // `requiredFlags()[f.name]` — Boolean() calls in templates are
    // forbidden by the renderable expression subset.
    const requiredFlags = derived<Record<string, boolean>>(() => {
      const out: Record<string, boolean> = {}
      for (const f of fieldDefs()) out[f.name] = f.required === true
      return out
    })

    // Per-type type coercion for the default case (number inputs etc.).
    const numberTypes = derived<Record<string, boolean>>(() => {
      const out: Record<string, boolean> = {}
      for (const f of fieldDefs()) out[f.name] = f.type === 'integer' || f.type === 'real'
      return out
    })

    function setField(name: string, value: FormValue): void {
      const cur = formState()
      formState.set({ ...cur, [name]: value })
    }

    function onTextInput(name: string, e: Event): void {
      const detail = (e as CustomEvent<{ value: string }>).detail
      setField(name, detail?.value ?? '')
    }

    function onChecked(name: string, e: Event): void {
      const detail = (e as CustomEvent<{ checked: boolean }>).detail
      setField(name, detail?.checked === true)
    }

    const errors = derived<string[]>(() => {
      const defs = fieldDefs()
      const values = formState()
      const out: string[] = []
      for (const f of defs) {
        const v = values[f.name]
        const valid = validateFieldValue(f, v)
        if (valid === 'required') out.push(`${f.name} is required`)
        else if (valid === 'wrong-type') out.push(`${f.name} expects ${f.type}`)
      }
      return out
    })

    const saving = signal(false)

    function submit(e: Event): void {
      e.preventDefault()
      const defs = fieldDefs()
      const values = formState()
      for (const f of defs) {
        const v = values[f.name]
        const valid = validateFieldValue(f, v)
        if (valid !== true) return
      }
      const data: Record<string, unknown> = {}
      for (const f of defs) {
        const v = values[f.name]
        if (f.type === 'json' && typeof v === 'string' && v.length > 0) {
          try { data[f.name] = JSON.parse(v) } catch { data[f.name] = v }
        } else {
          data[f.name] = coerceFromString(String(v ?? ''), f.type)
        }
      }
      saving.set(true)
      void (async (): Promise<void> => {
        try {
          const c = collection()
          if (mode() === 'create') {
            await ship.collections.create_item({ collection: c, data })
          } else {
            await ship.collections.update_item({ collection: c, item_id: id(), data })
          }
          host?.dispatchEvent(
            new CustomEvent('ship-item-save', {
              detail: { collection: c, data, id: mode() === 'edit' ? id() : undefined },
              bubbles: true,
              composed: true,
            }),
          )
        } finally {
          saving.set(false)
        }
      })()
    }

    function cancel(): void {
      host?.dispatchEvent(
        new CustomEvent('ship-item-cancel', {
          detail: { collection: collection() },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return { state, schema, fieldDefs, formState, jsonValues, requiredFlags, numberTypes, errors, saving, submit, cancel, onTextInput, onChecked }
  },
})
