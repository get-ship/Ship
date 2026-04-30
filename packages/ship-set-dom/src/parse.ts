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
  /**
   * Compiled source thunk. Receives `(ship, route, $scope)`. `$scope`
   * carries every identifier already in view scope at the time the
   * `<data>` mounts — ship, route, navigate, and any earlier `<state>`
   * declarations or controller bindings (when the resolver pre-mounts
   * state ahead of data).
   *
   * Reading a signal inside this thunk is tracked: `live()` re-fires
   * the thunk when any tracked dependency changes, disposing the prior
   * SSE subscription and opening a new one. That is the mechanism for
   * reactive `<data>` source args (e.g.
   * `source="ship.agents.get({id: $scope.selectedId()})"`).
   */
  source: (
    ship: unknown,
    route: Record<string, string>,
    $scope: Record<string, unknown>,
  ) => unknown
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
  /**
   * Whether the component instances render into a shadow root.
   * Default `false` (light DOM). Opt in via `<component shadow>`.
   *
   * Shadow components get `:host(...)` selectors, scoped CSS, and
   * `::slotted(...)`-aware children. Light components share the
   * page's CSS scope; the framework auto-prefixes `[data-component=...]`
   * to selectors but doesn't enforce isolation.
   */
  shadow: boolean
  /**
   * Styles to attach inside the shadow root (shadow components only),
   * captured from a `<style>` child of `<component>`. Concatenated in
   * source order across multiple `<style>` tags. Empty string when
   * no `<style>` was authored.
   */
  styles: string
}

/** A single attribute-value carrier on a `<use-*>` element. */
export interface UseAttrValue {
  /** `'literal'` — `name="value"`. The hook receives `value` verbatim. */
  /** `'identifier'` — `name="ident"` (no quotes around braces). The hook
   *    receives the runtime value bound to `ident` in scope. */
  /** `'binding'` — `bind:name={ident}`. Same as identifier in v1; the
   *    binding marker remains for future static-checking distinctions. */
  kind: 'literal' | 'identifier' | 'binding'
  /** For `literal`: the raw string. For `identifier`/`binding`: the
   *  scope-key the hook factory should resolve. */
  raw: string
}

/** A `<on key="..." action="..." />`-style child of a `<use-*>` element.
 * Tag name + a flat record of literal attribute strings. */
export interface UseChildDecl {
  tag: string
  attrs: Record<string, string>
}

/** A `<use-*>` element lifted out of the template at parse time. */
export interface UseDecl {
  /** The tag suffix — `<use-kbd>` → `'kbd'`. */
  name: string
  /** Parsed attributes by name. Keys come from the `<use-*>` element. */
  attrs: Record<string, UseAttrValue>
  /** Parsed child config records. */
  children: UseChildDecl[]
}

/** Result of parsing a `.html.set` source. */
export interface ParsedSource {
  data: DataDecl[]
  state: StateDecl[]
  component: ComponentDecl | null
  /** `<use-*>` template-hook elements lifted from the body. */
  uses: UseDecl[]
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
  const uses: UseDecl[] = []

  // Walk children once; pluck top-level directives and `<use-*>` hooks
  // and remove them. Both `<data>`/`<state>`/`<component>` and
  // `<use-*>` elements live at the body's top level only — nested
  // `<use-foo>` deeper in the template is reserved (planned but not
  // yet supported) and lifted via the same path here.
  const toRemove: Element[] = []
  for (const child of Array.from(body.children)) {
    const tag = child.tagName.toLowerCase()
    if (TOP_LEVEL_TAGS.has(tag)) {
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
    } else if (tag.startsWith('use-')) {
      toRemove.push(child)
      uses.push(parseUse(child))
    }
  }
  for (const el of toRemove) el.remove()

  // Trim leading/trailing whitespace text nodes; they're noise.
  trimEdges(body)

  return {
    data,
    state,
    component,
    uses,
    templateNodes: Array.from(body.childNodes),
    doc,
  }
}

const parseUse = (el: Element): UseDecl => {
  const name = el.tagName.toLowerCase().slice('use-'.length)
  const attrs: Record<string, UseAttrValue> = {}
  for (const a of Array.from(el.attributes)) {
    if (a.name.startsWith('bind:')) {
      // bind:foo={ident} — preprocess wraps brace values in quotes,
      // strip the leading/trailing braces if present so the hook
      // factory receives just the identifier name.
      const ident = a.value.replace(/^\{|\}$/g, '').trim()
      attrs[a.name.slice('bind:'.length)] = { kind: 'binding', raw: ident }
    } else if (a.value.startsWith('{') && a.value.endsWith('}')) {
      // Brace value treated as an identifier reference (resolved at
      // hydrate from view scope).
      attrs[a.name] = {
        kind: 'identifier',
        raw: a.value.slice(1, -1).trim(),
      }
    } else {
      attrs[a.name] = { kind: 'literal', raw: a.value }
    }
  }
  const children: UseChildDecl[] = []
  for (const c of Array.from(el.children)) {
    const childAttrs: Record<string, string> = {}
    for (const ca of Array.from(c.attributes)) childAttrs[ca.name] = ca.value
    children.push({ tag: c.tagName.toLowerCase(), attrs: childAttrs })
  }
  return { name, attrs, children }
}

const parseData = (el: Element): DataDecl => {
  const name = required(el, 'name', '<data>')
  const source = required(el, 'source', '<data>')
  return {
    name,
    source: compileSource(source, ['ship', 'route', '$scope'], {
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
  const shadow = el.hasAttribute('shadow')
  const props: PropDecl[] = []
  const styleParts: string[] = []
  for (const child of Array.from(el.children)) {
    const childTag = child.tagName.toLowerCase()
    if (childTag === 'prop') props.push(parseProp(child))
    else if (childTag === 'style') styleParts.push(child.textContent ?? '')
  }
  return { tag, props, shadow, styles: styleParts.join('\n') }
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
