// <component> directive — registers the file's body as a custom
// element. `<prop>` children declare reactive props; the runtime's
// `component()` factory hands per-instance prop signals to a setup
// callback we wire to render the parsed template body.

import { component, prop, type PropDef } from '@ship/ui-runtime'
import type { ComponentDecl } from '../parse'
import type { Instr } from '../template/compile'
import { extendScope, render, type Scope } from '../template/render'

const camelOf = (s: string): string =>
  s.replace(/-([a-z])/g, (_, c: string) => c.toUpperCase())

const buildPropDef = (
  decl: ComponentDecl['props'][number],
): PropDef<unknown> => {
  switch (decl.type) {
    case 'union':
      return prop.union(decl.values ?? [], {
        ...(decl.default !== undefined ? { default: decl.default as string } : {}),
        reflect: decl.reflect,
        bindable: decl.bindable,
      }) as PropDef<unknown>
    case 'bool':
      return prop.bool({
        ...(decl.default !== undefined
          ? { default: Boolean(decl.default) }
          : {}),
        reflect: decl.reflect,
        bindable: decl.bindable,
      }) as PropDef<unknown>
    case 'number':
      return prop.number({
        ...(decl.default !== undefined
          ? { default: Number(decl.default) }
          : {}),
        reflect: decl.reflect,
        bindable: decl.bindable,
      }) as PropDef<unknown>
    case 'string':
      return prop.string({
        ...(decl.default !== undefined ? { default: String(decl.default) } : {}),
        reflect: decl.reflect,
        bindable: decl.bindable,
      }) as PropDef<unknown>
  }
}

/**
 * Register a `<component>`-declared element. The body's compiled
 * instructions are rendered into each instance against a per-instance
 * scope that exposes the props (under both kebab and camelCase names)
 * plus whatever the optional user setup returns.
 */
export function registerComponent(
  decl: ComponentDecl,
  body: Instr[],
  doc: Document,
  userSetup?: () => unknown,
): void {
  const props: Record<string, PropDef<unknown>> = {}
  for (const p of decl.props) props[p.name] = buildPropDef(p)

  component(decl.tag, {
    props,
    ...(decl.shadow ? { shadow: true as const } : {}),
    ...(decl.styles ? { styles: decl.styles } : {}),
    setup: ({ props: propSignals, host, cleanups }) => {
      const scope: Scope = {}
      for (const [attr, sig] of Object.entries(propSignals)) {
        scope[attr] = sig
        const camel = camelOf(attr)
        if (camel !== attr) scope[camel] = sig
      }
      let extras: Record<string, unknown> | undefined
      if (userSetup) {
        const ret = userSetup.call(undefined)
        if (ret && typeof ret === 'object') {
          extras = ret as Record<string, unknown>
        }
      }
      const finalScope = extras ? extendScope(scope, extras) : scope
      const r = render(body, finalScope, doc)
      // Shadow components render into the attached shadow root; light
      // components render into the host itself. The factory attaches
      // the shadow root before `setup()` runs, so checking for it
      // here is the simple discriminator.
      const target: ParentNode = host.shadowRoot ?? host
      for (const n of r.nodes) target.appendChild(n)
      const cleanup = r.mount()
      cleanups.push(() => {
        for (const n of r.nodes) {
          try {
            n.parentNode?.removeChild(n)
          } catch {
            /* detached */
          }
        }
        cleanup()
      })
    },
  })
}
