// Attribute classifier: takes a parsed Element's attributes and turns
// them into AttrInstr nodes that the renderer applies at hydrate time.
//
// Recognises the prefixed forms documented in the Phase-2 spec:
//
//   on:event[|mod[:arg]…]={handler}
//   bind:value={signal} | bind:checked={signal}
//   class:name={cond}
//   style:--var={value}
//   attr:{name}={value}     (dynamic-name)
//   plain  attr={value} or attr="literal"

import { compileExpr, type CompiledExpr } from '../template/expr'
import type { Modifier } from '@ship/ui-runtime'

export type AttrInstr =
  | { kind: 'static'; name: string; value: string }
  | { kind: 'attr'; name: string; expr: CompiledExpr }
  | { kind: 'attr-dynamic'; nameExpr: CompiledExpr; expr: CompiledExpr }
  | { kind: 'bind-value'; expr: CompiledExpr }
  | { kind: 'bind-checked'; expr: CompiledExpr }
  | { kind: 'on'; event: string; handler: CompiledExpr; modifiers: Modifier[] }
  | { kind: 'class'; name: string; expr: CompiledExpr }
  | { kind: 'style'; name: string; expr: CompiledExpr }

const EXPR_RE = /^\{(.*)\}$/s

const isExprValue = (v: string): boolean => EXPR_RE.test(v.trim())
const stripBraces = (v: string): string => {
  const m = v.trim().match(EXPR_RE)
  return m ? (m[1] ?? '') : v
}

/**
 * Compile every attribute on `el` into an `AttrInstr`. The element's
 * own attribute list is left intact — the renderer's element clone
 * will also strip the recognized prefix attributes before insertion so
 * they don't leak into the produced DOM.
 */
export function compileAttrs(el: Element, locPrefix: string): AttrInstr[] {
  const out: AttrInstr[] = []
  for (const { name, value } of Array.from(el.attributes)) {
    out.push(compileAttr(name, value, `${locPrefix} ${name}`))
  }
  return out
}

const compileAttr = (
  name: string,
  value: string,
  loc: string,
): AttrInstr => {
  if (name.startsWith('on:')) return compileOn(name, value, loc)
  if (name.startsWith('bind:')) return compileBind(name, value, loc)
  if (name.startsWith('class:')) return compileClass(name, value, loc)
  if (name.startsWith('style:')) return compileStyle(name, value, loc)
  if (name.startsWith('attr:')) return compileDynamicAttr(name, value, loc)

  if (isExprValue(value)) {
    return {
      kind: 'attr',
      name,
      expr: compileExpr(stripBraces(value), { location: loc }),
    }
  }
  return { kind: 'static', name, value }
}

const compileOn = (name: string, value: string, loc: string): AttrInstr => {
  const parts = name.slice(3).split('|')
  const event = parts[0] ?? ''
  if (!event) {
    throw new Error(`[ship-set-dom] empty event name in on: binding (at ${loc})`)
  }
  const modifiers: Modifier[] = []
  for (let i = 1; i < parts.length; i++) {
    const m = parts[i] ?? ''
    if (!m) continue
    modifiers.push(parseModifier(m, loc))
  }
  if (!isExprValue(value)) {
    throw new Error(
      `[ship-set-dom] on: handler must be {expression} (at ${loc})`,
    )
  }
  return {
    kind: 'on',
    event,
    handler: compileExpr(stripBraces(value), {
      allowArrow: true,
      location: loc,
    }),
    modifiers,
  }
}

const parseModifier = (raw: string, loc: string): Modifier => {
  const colonAt = raw.indexOf(':')
  const head = colonAt === -1 ? raw : raw.slice(0, colonAt)
  const arg = colonAt === -1 ? undefined : raw.slice(colonAt + 1)
  // HTML attribute names are lowercased by the parser, so accept both
  // camelCase (compiler-friendly) and lowercase forms.
  switch (head.toLowerCase()) {
    case 'preventdefault':
    case 'prevent':
      return { kind: 'preventDefault' }
    case 'stoppropagation':
    case 'stop':
      return { kind: 'stopPropagation' }
    case 'stopimmediatepropagation':
      return { kind: 'stopImmediatePropagation' }
    case 'self':
      return { kind: 'self' }
    case 'trusted':
      return { kind: 'trusted' }
    case 'key':
      return arg !== undefined ? { kind: 'key', arg } : { kind: 'key' }
    case 'debounce':
      return arg !== undefined
        ? { kind: 'debounce', arg }
        : { kind: 'debounce' }
    case 'throttle':
      return arg !== undefined
        ? { kind: 'throttle', arg }
        : { kind: 'throttle' }
    default:
      throw new Error(
        `[ship-set-dom] unknown modifier "${head}" (at ${loc})`,
      )
  }
}

const compileBind = (name: string, value: string, loc: string): AttrInstr => {
  const target = name.slice(5)
  if (!isExprValue(value)) {
    throw new Error(
      `[ship-set-dom] bind:${target} expects {signal} (at ${loc})`,
    )
  }
  const expr = compileExpr(stripBraces(value), { location: loc })
  if (target === 'value') return { kind: 'bind-value', expr }
  if (target === 'checked') return { kind: 'bind-checked', expr }
  throw new Error(`[ship-set-dom] unknown bind:${target} (at ${loc})`)
}

const compileClass = (name: string, value: string, loc: string): AttrInstr => {
  const className = name.slice(6)
  if (!className) {
    throw new Error(`[ship-set-dom] class: requires a class name (at ${loc})`)
  }
  if (!isExprValue(value)) {
    throw new Error(
      `[ship-set-dom] class:${className} expects {expr} (at ${loc})`,
    )
  }
  return {
    kind: 'class',
    name: className,
    expr: compileExpr(stripBraces(value), { location: loc }),
  }
}

const compileStyle = (name: string, value: string, loc: string): AttrInstr => {
  const prop = name.slice(6)
  if (!prop) {
    throw new Error(`[ship-set-dom] style: requires a property name (at ${loc})`)
  }
  if (!isExprValue(value)) {
    throw new Error(
      `[ship-set-dom] style:${prop} expects {expr} (at ${loc})`,
    )
  }
  return {
    kind: 'style',
    name: prop,
    expr: compileExpr(stripBraces(value), { location: loc }),
  }
}

const compileDynamicAttr = (
  name: string,
  value: string,
  loc: string,
): AttrInstr => {
  // `attr:{name-expr}={value-expr}` — the part after `attr:` is the
  // dynamic-name expression, surrounded by literal braces in source.
  const rawNameExpr = name.slice(5)
  if (!isExprValue(rawNameExpr)) {
    throw new Error(
      `[ship-set-dom] attr: expects attr:{nameExpr} (at ${loc})`,
    )
  }
  if (!isExprValue(value)) {
    throw new Error(
      `[ship-set-dom] attr:{...} expects ={valueExpr} (at ${loc})`,
    )
  }
  return {
    kind: 'attr-dynamic',
    nameExpr: compileExpr(stripBraces(rawNameExpr), { location: loc }),
    expr: compileExpr(stripBraces(value), { location: loc }),
  }
}
