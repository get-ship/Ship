// Apply a compiled AttrInstr to a live element. Pushes any disposers
// it creates onto the caller's list so the parent render's cleanup
// chain tears them down with the rest.

import {
  bindAttr,
  bindClass,
  bindValue,
  bindChecked,
  effect,
  on,
  type Disposer,
  type Signal,
} from '@ship/ui-runtime'

import type { Scope } from './render'
import type { AttrInstr } from '../bindings/attr-compile'

const assertSignal = (v: unknown, label: string): void => {
  if (typeof v !== 'function') {
    throw new Error(`[ship-set-dom] ${label} requires a Signal`)
  }
}

export function applyAttr(
  el: Element,
  a: AttrInstr,
  scope: Scope,
  disposers: Disposer[],
): void {
  switch (a.kind) {
    case 'static':
      el.setAttribute(a.name, a.value)
      return
    case 'attr':
      disposers.push(bindAttr(el, a.name, () => a.expr(scope)))
      return
    case 'attr-dynamic': {
      let prevName: string | null = null
      const dispose = effect(() => {
        const nameVal = a.nameExpr(scope)
        const valueVal = a.expr(scope)
        const nm = nameVal == null ? null : String(nameVal)
        if (prevName !== null && prevName !== nm) el.removeAttribute(prevName)
        if (nm !== null) {
          if (valueVal == null || valueVal === false || valueVal === '') {
            el.removeAttribute(nm)
          } else {
            el.setAttribute(nm, valueVal === true ? '' : String(valueVal))
          }
        }
        prevName = nm
      })
      disposers.push(dispose)
      return
    }
    case 'bind-value': {
      const sig = a.expr(scope)
      assertSignal(sig, 'bind:value')
      disposers.push(
        bindValue(
          el as HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement,
          sig as Signal<string>,
        ),
      )
      return
    }
    case 'bind-checked': {
      const sig = a.expr(scope)
      assertSignal(sig, 'bind:checked')
      disposers.push(bindChecked(el as HTMLInputElement, sig as Signal<boolean>))
      return
    }
    case 'on': {
      const handler = a.handler(scope)
      if (typeof handler !== 'function') {
        throw new Error(
          `[ship-set-dom] on:${a.event} handler is not a function`,
        )
      }
      const opts =
        a.modifiers.length > 0 ? { modifiers: a.modifiers } : undefined
      disposers.push(
        on(el, a.event, handler as (e: Event) => unknown, opts ?? {}),
      )
      return
    }
    case 'class':
      disposers.push(bindClass(el, a.name, () => a.expr(scope)))
      return
    case 'style': {
      const dispose = effect(() => {
        const v = a.expr(scope)
        const html = el as HTMLElement
        if (v == null || v === false || v === '') {
          html.style.removeProperty(a.name)
        } else {
          html.style.setProperty(a.name, String(v))
        }
      })
      disposers.push(dispose)
      return
    }
  }
}
