import { describe, expect, it } from 'vitest'
import { compileExpr } from '../src/template/expr'
import { tokenizeChildren, tokenizeText, type Token } from '../src/template/tokenize'

describe('compileExpr — happy path', () => {
  it('reads identifiers from scope', () => {
    const fn = compileExpr('count + 1')
    expect(fn({ count: 7 })).toBe(8)
  })

  it('calls scope-visible functions', () => {
    const fn = compileExpr('getName()')
    expect(fn({ getName: () => 'ada' })).toBe('ada')
  })

  it('supports member access and ternary', () => {
    const fn = compileExpr('user.active ? user.name : "anon"')
    expect(fn({ user: { active: true, name: 'ada' } })).toBe('ada')
    expect(fn({ user: { active: false, name: 'x' } })).toBe('anon')
  })

  it('supports template literals', () => {
    const fn = compileExpr('`hello ${name}`')
    expect(fn({ name: 'world' })).toBe('hello world')
  })
})

describe('compileExpr — guarded forms', () => {
  it('rejects assignments', () => {
    expect(() => compileExpr('x = 5')).toThrow(/assignments are not allowed/)
  })

  it('rejects statements', () => {
    expect(() => compileExpr('a; b')).toThrow(/statements are not allowed/)
  })

  it('rejects arrows by default', () => {
    expect(() => compileExpr('() => 1')).toThrow(/arrow functions are not allowed/)
  })

  it('allows arrows when opts.allowArrow is set', () => {
    const fn = compileExpr('() => 42', { allowArrow: true })
    const arrow = fn({}) as () => number
    expect(arrow()).toBe(42)
  })

  it('reports a clear error with location', () => {
    try {
      compileExpr('x = 1', { location: 'attr foo' })
    } catch (e) {
      expect((e as Error).message).toMatch(/attr foo/)
    }
  })

  it('does not flag = inside string literals as assignment', () => {
    const fn = compileExpr('"a=b"')
    expect(fn({})).toBe('a=b')
  })
})

describe('tokenizeText', () => {
  it('splits text into static segments and {expr}', () => {
    const out: Token[] = []
    tokenizeText('hello {name}!', out)
    expect(out.map((t) => t.kind)).toEqual(['text', 'expr', 'text'])
    if (out[0]!.kind === 'text') expect(out[0]!.value).toBe('hello ')
    if (out[1]!.kind === 'expr') expect(out[1]!.src).toBe('name')
    if (out[2]!.kind === 'text') expect(out[2]!.value).toBe('!')
  })

  it('recognizes {#if}/{:else}/{/if} markers', () => {
    const out: Token[] = []
    tokenizeText('{#if cond} {:else} {/if}', out)
    const kinds = out.map((t) => t.kind)
    expect(kinds).toContain('open-if')
    expect(kinds).toContain('else')
    expect(kinds).toContain('close-if')
  })

  it('handles nested braces inside expressions', () => {
    const out: Token[] = []
    tokenizeText('{user.opts || {a: 1}}', out)
    expect(out).toHaveLength(1)
    expect(out[0]!.kind).toBe('expr')
  })
})

describe('tokenizeChildren', () => {
  it('walks DOM children producing element + text tokens', () => {
    const doc = new DOMParser().parseFromString(
      '<!doctype html><html><body><div>foo {x} <span>bar</span></div></body></html>',
      'text/html',
    )
    const div = doc.querySelector('div')!
    const tokens = tokenizeChildren(div)
    expect(tokens.map((t) => t.kind)).toEqual(['text', 'expr', 'text', 'element'])
  })
})
