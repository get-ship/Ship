import { describe, expect, it, beforeEach, afterEach } from 'vitest'
import { useTitleHook } from '../../src/template-hooks/title'
import { signal } from '../../src/signal'
import type { TemplateHookContext } from '../../src/template-hook'

function makeCtx(
  attrs: Record<string, unknown>,
): { ctx: TemplateHookContext<Record<string, unknown>>; cleanups: (() => void)[] } {
  const cleanups: (() => void)[] = []
  const ctx: TemplateHookContext<Record<string, unknown>> = {
    attrs,
    el: document.body,
    cleanup: (fn) => cleanups.push(fn),
  }
  return { ctx, cleanups }
}

describe('useTitleHook', () => {
  let original: string

  beforeEach(() => {
    original = document.title
  })

  afterEach(() => {
    document.title = original
  })

  it('registers under name "title"', () => {
    expect(useTitleHook.name).toBe('title')
  })

  it('does nothing when text is not provided', () => {
    const { ctx, cleanups } = makeCtx({ text: undefined })
    expect(() => useTitleHook.factory(ctx as never)).not.toThrow()
    expect(cleanups).toHaveLength(0)
  })

  it('sets document.title to a plain string on mount', () => {
    const { ctx, cleanups } = makeCtx({ text: 'My Page' })
    useTitleHook.factory(ctx as never)

    expect(document.title).toBe('My Page')
    for (const fn of cleanups) fn()
  })

  it('sets document.title from a signal accessor', () => {
    const title = signal('Initial Title')
    const { ctx, cleanups } = makeCtx({ text: title })
    useTitleHook.factory(ctx as never)

    expect(document.title).toBe('Initial Title')
    for (const fn of cleanups) fn()
  })

  it('updates document.title reactively when signal changes', () => {
    const title = signal('Page A')
    const { ctx, cleanups } = makeCtx({ text: title })
    useTitleHook.factory(ctx as never)

    expect(document.title).toBe('Page A')

    title.set('Page B')
    expect(document.title).toBe('Page B')

    title.set('Page C')
    expect(document.title).toBe('Page C')

    for (const fn of cleanups) fn()
  })

  it('restores the previous document.title on cleanup', () => {
    document.title = 'Original'
    const { ctx, cleanups } = makeCtx({ text: 'Override' })
    useTitleHook.factory(ctx as never)

    expect(document.title).toBe('Override')
    for (const fn of cleanups) fn()
    expect(document.title).toBe('Original')
  })

  it('stops tracking signal changes after cleanup', () => {
    document.title = 'Before'
    const title = signal('During')
    const { ctx, cleanups } = makeCtx({ text: title })
    useTitleHook.factory(ctx as never)

    for (const fn of cleanups) fn()
    // document.title restored to 'Before'

    title.set('After cleanup')
    expect(document.title).toBe('Before')
  })
})
