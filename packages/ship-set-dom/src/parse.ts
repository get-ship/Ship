// Top-level parser. Splits a `.html.set` source string into the
// directive declarations (`<data>`, `<state>`, `<component>`) and the
// remaining template body (a list of nodes to be compiled and
// hydrated).
//
// We use the browser DOMParser (jsdom in tests) because the HTML side
// of `.html.set` is plain HTML. Template grammar — `{expr}`, `{#if}`,
// etc. — survives parsing as text-node content; the compiler walks
// the resulting DOM and processes those text-segment markers.

import { compileExpr, compileSource, type CompiledExpr } from './template/expr'
import { preprocess } from './preprocess'

/** Declaration extracted from a top-level `<data>` element. */
export interface DataDecl {
  name: string
  /** Compiled source thunk; called with (ship, route). */
  source: (ship: unknown, route: Record<string, string>) => unknown
  live: boolean
}

/** Declaration extracted from a top-level `<state>` element. */
export interface StateDecl {
  name: string
  /** Initial-value expression, compiled against the current scope. */
  value: CompiledExpr
}

/** A single `<prop>` declaration inside a `<component>` element. */
export interface PropDecl {
  name: string
  type: 'string' | 'bool' | 'number' | 'union'
  values?: readonly string[]
  default?: string | number | boolean | undefined
  reflect: boolean
  bindable: boolean
}

/** Declaration extracted from a top-level `<component>` element. */
export interface ComponentDecl {
  tag: string
  props: PropDecl[]
}

/** Result of parsing a `.html.set` source. */
export interface ParsedSource {
  data: DataDecl[]
  state: StateDecl[]
  component: ComponentDecl | null
  /** Top-level template nodes — the body of the file. */
  templateNodes: Node[]
  /** The host document used to construct templateNodes (jsdom in tests). */
  doc: Document
}

const TOP_LEVEL_TAGS = new Set(['data', 'state', 'component'])

/**
 * Parse a `.html.set` source string and split it into directive
 * declarations and a template body.
 *
 * The directive elements (`<data>`, `<state>`, `<component>`) are
 * removed from the template body so that only the renderable content
 * remains. Order is preserved within each declaration list — useful
 * because later `<state>` declarations can reference earlier ones in
 * their `value=` expressions.
 *
 * @param source Raw `.html.set` file contents.
 * @returns A `ParsedSource` describing the directives and template.
 */
export function parseSetSource(source: string): ParsedSource {
  const prepared = preprocess(source)
  const wrapped = `<!doctype html><html><body>${prepared}</body></html>`
  const doc = new DOMParser().parseFromString(wrapped, 'text/html')
  const body = doc.body

  const data: DataDecl[] = []
  const state: StateDecl[] = []
  let component: ComponentDecl | null = null

  // Walk children once; pluck top-level directives and remove them.
  const toRemove: Element[] = []
  for (const child of Array.from(body.children)) {
    const tag = child.tagName.toLowerCase()
    if (!TOP_LEVEL_TAGS.has(tag)) continue
    toRemove.push(child)
    if (tag === 'data') data.push(parseData(child))
    else if (tag === 'state') state.push(parseState(child))
    else if (tag === 'component') {
      if (component) {
        throw new Error(
          '[ship-set-dom] only one <component> declaration allowed per file',
        )
      }
      component = parseComponent(child)
    }
  }
  for (const el of toRemove) el.remove()

  // Trim leading/trailing whitespace text nodes; they're noise.
  trimEdges(body)

  return {
    data,
    state,
    component,
    templateNodes: Array.from(body.childNodes),
    doc,
  }
}

const parseData = (el: Element): DataDecl => {
  const name = required(el, 'name', '<data>')
  const source = required(el, 'source', '<data>')
  return {
    name,
    source: compileSource(source, ['ship', 'route'], {
      location: `<data name="${name}">`,
    }) as DataDecl['source'],
    live: el.hasAttribute('live'),
  }
}

const parseState = (el: Element): StateDecl => {
  const name = required(el, 'name', '<state>')
  const raw = required(el, 'value', '<state>')
  // Authors may write `value={0}` (brace form) or `value="0"` (literal
  // expression string). Both compile to the same code.
  const expr = stripExprBraces(raw)
  return {
    name,
    value: compileExpr(expr, { location: `<state name="${name}">` }),
  }
}

const stripExprBraces = (s: string): string => {
  const trimmed = s.trim()
  if (trimmed.startsWith('{') && trimmed.endsWith('}')) {
    return trimmed.slice(1, -1).trim()
  }
  return trimmed
}

const parseComponent = (el: Element): ComponentDecl => {
  const tag = required(el, 'name', '<component>')
  const props: PropDecl[] = []
  for (const child of Array.from(el.children)) {
    if (child.tagName.toLowerCase() !== 'prop') continue
    props.push(parseProp(child))
  }
  return { tag, props }
}

const parseProp = (el: Element): PropDecl => {
  const name = required(el, 'name', '<prop>')
  const valuesAttr = el.getAttribute('values')
  const typeAttr = el.getAttribute('type')
  const defaultAttr = el.getAttribute('default') ?? undefined
  const reflect = el.hasAttribute('reflect')
  const bindable = el.hasAttribute('bindable')

  if (valuesAttr) {
    const values = valuesAttr.split('|').map((v) => v.trim()).filter(Boolean)
    return {
      name,
      type: 'union',
      values,
      default: defaultAttr,
      reflect,
      bindable,
    }
  }
  switch (typeAttr) {
    case 'bool':
      return {
        name,
        type: 'bool',
        default: defaultAttr === 'true',
        reflect,
        bindable,
      }
    case 'number':
      return {
        name,
        type: 'number',
        default: defaultAttr === undefined ? undefined : Number(defaultAttr),
        reflect,
        bindable,
      }
    case 'string':
    case null:
    case undefined:
      return {
        name,
        type: 'string',
        default: defaultAttr,
        reflect,
        bindable,
      }
    default:
      throw new Error(
        `[ship-set-dom] <prop name="${name}"> unknown type: ${typeAttr}`,
      )
  }
}

const required = (el: Element, attr: string, label: string): string => {
  const v = el.getAttribute(attr)
  if (v === null) {
    throw new Error(`[ship-set-dom] ${label} missing required attribute: ${attr}`)
  }
  return v
}

const trimEdges = (parent: Element): void => {
  while (parent.firstChild && isEmptyText(parent.firstChild)) {
    parent.firstChild.remove()
  }
  while (parent.lastChild && isEmptyText(parent.lastChild)) {
    parent.lastChild.remove()
  }
}

const isEmptyText = (n: Node): boolean =>
  n.nodeType === 3 /* TEXT_NODE */ && (n.textContent ?? '').trim() === ''
