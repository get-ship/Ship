import { afterEach, describe, expect, it } from 'vitest'
import './ship-pipeline-run'
import type { ShipPipelineRun } from './ship-pipeline-run'
import { cleanupTestDom } from '../core/util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipPipelineRun
  await el.updateComplete
  return el
}

describe('ship-pipeline-run', () => {
  afterEach(cleanupTestDom)

  it('renders slug + agent + state badge', async () => {
    const el = await mount('<ship-pipeline-run state="running" slug="fix-bug" agent="tester"></ship-pipeline-run>')
    expect(el.shadowRoot!.querySelector('.slug')!.textContent).toBe('fix-bug')
    expect(el.shadowRoot!.querySelector('.agent')!.textContent).toContain('tester')
    const badge = el.shadowRoot!.querySelector('ship-badge')!
    expect(badge.textContent!.trim()).toBe('running')
  })

  it('maps state completed to tone=success', async () => {
    const el = await mount('<ship-pipeline-run state="completed" slug="x"></ship-pipeline-run>')
    const badge = el.shadowRoot!.querySelector('ship-badge')!
    expect(badge.getAttribute('tone')).toBe('success')
  })
})
