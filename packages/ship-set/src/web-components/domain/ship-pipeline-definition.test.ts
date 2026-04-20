import { afterEach, describe, expect, it } from 'vitest'
import './ship-pipeline-definition'
import type { ShipPipelineDefinition } from './ship-pipeline-definition'
import { cleanupTestDom } from '../core/util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipPipelineDefinition
  await el.updateComplete
  return el
}

describe('ship-pipeline-definition', () => {
  afterEach(cleanupTestDom)

  it('renders name + step count + description', async () => {
    const el = await mount('<ship-pipeline-definition name="tdd" steps="3" description="Test-first"></ship-pipeline-definition>')
    expect(el.shadowRoot!.querySelector('.name')!.textContent).toBe('tdd')
    expect(el.shadowRoot!.querySelector('.steps')!.textContent).toContain('3 step')
    expect(el.shadowRoot!.querySelector('.desc')!.textContent).toBe('Test-first')
  })

  it('pluralizes steps', async () => {
    const el = await mount('<ship-pipeline-definition name="x" steps="1"></ship-pipeline-definition>')
    expect(el.shadowRoot!.querySelector('.steps')!.textContent).toBe('1 step')
  })

  it('falls back to muted No description', async () => {
    const el = await mount('<ship-pipeline-definition name="x" steps="2"></ship-pipeline-definition>')
    expect(el.shadowRoot!.querySelector('.desc')!.classList.contains('no-desc')).toBe(true)
  })
})
