// Components Studio section — detail-surface logic tests.
//
// Run under the shell's vitest config:
//   cd apps/shell && bunx vitest run "src/studio/components/[id]/page.test.ts"
//
// Targets the storybook's deterministic core: matrix-axis derivation
// (criterion 3), knob-control selection, and the knob → live-render +
// snippet pipeline (criterion 4). The single live preview and matrix
// hosts are imperative DOM, exercised here via applyProps under jsdom.

import { describe, it, expect } from 'vitest'
import type { PropEntry } from '../catalog'
import { deriveAxes, buildCells } from '../matrix'
import { buildSnippet } from '../snippet'
import { seedDefaults, applyProps } from '../render'
import { knobFor } from '../sui-component-detail/sui-component-detail.controller'

// sui-button's published prop schema (subset that drives the storybook).
const BUTTON_PROPS: PropEntry[] = [
  { name: 'variant', type: "'solid' | 'soft' | 'outline' | 'ghost' | 'default'", default: 'default',
    values: ['solid', 'soft', 'outline', 'ghost', 'default'] },
  { name: 'tone', type: 'string', default: 'default',
    values: ['default', 'primary', 'info', 'success', 'warning', 'destructive'] },
  { name: 'size', type: 'string', default: 'md', values: ['xs', 'sm', 'md', 'lg'] },
  { name: 'disabled', type: 'boolean', default: 'false' },
  { name: 'label', type: 'string', default: '' },
]

describe('components/[id]: matrix axes', () => {
  it('derives variant × tone for sui-button (criterion 3)', () => {
    const axes = deriveAxes(BUTTON_PROPS)
    expect(axes.map((a) => a.name)).toEqual(['variant', 'tone'])
    const cells = buildCells(axes)
    expect(cells).toHaveLength(5 * 6)
    expect(cells[0].props).toEqual({ variant: 'solid', tone: 'default' })
  })

  it('falls back to a single tone/size axis when variant is absent', () => {
    const axes = deriveAxes([{ name: 'size', type: 'string', values: ['sm', 'md', 'lg'] }])
    expect(axes).toHaveLength(1)
    expect(axes[0].name).toBe('size')
    expect(buildCells(axes)).toHaveLength(3)
  })

  it('yields no axes for a component without enum props', () => {
    expect(deriveAxes([{ name: 'label', type: 'string' }])).toEqual([])
  })
})

describe('components/[id]: knob controls', () => {
  it('maps prop types to the right knob control', () => {
    expect(knobFor(BUTTON_PROPS[0]).kind).toBe('enum')   // variant
    expect(knobFor(BUTTON_PROPS[3]).kind).toBe('bool')   // disabled
    expect(knobFor(BUTTON_PROPS[4]).kind).toBe('text')   // label
  })

  it('carries enum values onto the knob for the segmented control', () => {
    expect(knobFor(BUTTON_PROPS[0]).values).toEqual(
      ['solid', 'soft', 'outline', 'ghost', 'default'],
    )
  })
})

describe('components/[id]: seed defaults', () => {
  it('prefers the largest size variant and the declared defaults', () => {
    const seed = seedDefaults(BUTTON_PROPS)
    expect(seed.size).toBe('lg')
    expect(seed.variant).toBe('default')
    expect(seed.disabled).toBe('false')
  })
})

describe('components/[id]: knob change updates render + snippet (criterion 4)', () => {
  it('reflects a changed enum value in both the DOM node and the snippet', () => {
    const node = document.createElement('sui-button')
    const vals = { ...seedDefaults(BUTTON_PROPS), variant: 'solid', tone: 'primary' }

    applyProps(node, vals, BUTTON_PROPS)
    expect(node.getAttribute('variant')).toBe('solid')
    expect(node.getAttribute('tone')).toBe('primary')

    const snippet = buildSnippet('sui-button', vals, BUTTON_PROPS, undefined)
    expect(snippet).toContain('variant="solid"')
    expect(snippet).toContain('tone="primary"')
    // size=lg differs from the declared default (md) so it lands too.
    expect(snippet).toContain('size="lg"')
    // label is empty + matches default → omitted.
    expect(snippet).not.toContain('label=')
  })

  it('strips a boolean attribute when toggled false, sets it when true', () => {
    const node = document.createElement('sui-button')
    applyProps(node, { disabled: 'true' }, BUTTON_PROPS)
    expect(node.hasAttribute('disabled')).toBe(true)
    expect(buildSnippet('sui-button', { disabled: 'true' }, BUTTON_PROPS, undefined))
      .toContain('disabled')

    applyProps(node, { disabled: 'false' }, BUTTON_PROPS)
    expect(node.hasAttribute('disabled')).toBe(false)
    expect(buildSnippet('sui-button', { disabled: 'false' }, BUTTON_PROPS, undefined))
      .not.toContain('disabled')
  })

  it('uses a slot composition as snippet inner content for compounds', () => {
    const snippet = buildSnippet('sui-card', {}, [], '<span slot="header">Card title</span>')
    expect(snippet).toContain('<sui-card>')
    expect(snippet).toContain('slot="header"')
    expect(snippet.endsWith('</sui-card>')).toBe(true)
  })
})
