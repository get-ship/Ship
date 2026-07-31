// Tests for the studio viewer's category classifier.
//
// The `categoryFor` helper maps a custom-element tag to its semantic
// group. It backs the viewer meta line (`<code>{category()}</code>`).
// The map is a stable public fact about the tag — not invented data —
// and is the only piece of testable logic in the studio pack (the rest
// of the surface degrades to a documented BACKEND-GAP empty-state).
//
// The test asserts the major shipped tags are covered (so a renamed
// primitive doesn't silently fall through to "Other") and the
// fallthrough behavior for an unknown tag.

import { describe, expect, it } from 'vitest'
import { categoryFor } from '../src/studio/ship-component-viewer/ship-component-viewer.controller'

describe('hub/studio · categoryFor', () => {
  it('classifies the core sui-* primitives', () => {
    expect(categoryFor('sui-button')).toBe('Actions')
    expect(categoryFor('sui-icon-button')).toBe('Actions')
    expect(categoryFor('sail-input')).toBe('Inputs')
    expect(categoryFor('sui-textarea')).toBe('Inputs')
    expect(categoryFor('sui-card')).toBe('Layout')
    expect(categoryFor('sui-stack')).toBe('Layout')
    expect(categoryFor('sui-cluster')).toBe('Layout')
    expect(categoryFor('sui-empty-state')).toBe('Feedback')
    expect(categoryFor('sui-spinner')).toBe('Feedback')
    expect(categoryFor('sui-badge')).toBe('Indicators')
    expect(categoryFor('sui-status-dot')).toBe('Indicators')
    expect(categoryFor('sui-dialog')).toBe('Overlays')
    expect(categoryFor('sui-tooltip')).toBe('Overlays')
    expect(categoryFor('sui-table')).toBe('Data')
    expect(categoryFor('sui-markdown')).toBe('Data')
  })

  it('classifies the ship-* chrome pack tags', () => {
    expect(categoryFor('ship-nav-top')).toBe('Chrome')
    expect(categoryFor('ship-app-chrome')).toBe('Chrome')
    expect(categoryFor('ship-rail')).toBe('Chrome')
    expect(categoryFor('ship-dock')).toBe('Chrome')
    expect(categoryFor('ship-command-bar')).toBe('Chrome')
    expect(categoryFor('ship-status-bar')).toBe('Chrome')
  })

  it('classifies the ship-* domain composites the hub actually mounts', () => {
    // These are the @ship/home + studio tags the hub routes depend on.
    // The viewer renders a "category" pill for them — if any silently
    // fall through to "Other", the hub pages would mis-label them.
    expect(categoryFor('ship-home-greeting')).toBe('Home')
    expect(categoryFor('ship-spaces-hero')).toBe('Home')
    expect(categoryFor('ship-activity-runs')).toBe('Home')
    expect(categoryFor('ship-component-catalog')).toBe('Studio')
    expect(categoryFor('ship-component-viewer')).toBe('Studio')
  })

  it('falls through to "Other" for unknown tags without throwing', () => {
    expect(categoryFor('not-a-real-tag')).toBe('Other')
    expect(categoryFor('')).toBe('Other')
    // The route's :name can be anything — defensive against author
    // typos / shell injection.
    expect(categoryFor('sui-<script>')).toBe('Other')
  })
})
