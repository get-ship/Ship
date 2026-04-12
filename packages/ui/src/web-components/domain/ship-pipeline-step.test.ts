import { afterEach, describe, expect, it } from 'vitest'
import './ship-pipeline-step'
import type { ShipPipelineStep } from './ship-pipeline-step'
import { cleanupTestDom } from '../util'

async function mount(markup: string) {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as ShipPipelineStep
  await el.updateComplete
  return el
}

describe('ship-pipeline-step', () => {
  afterEach(cleanupTestDom)

  it('renders name + type + state', async () => {
    const el = await mount('<ship-pipeline-step name="impl" step-type="agent" state="completed" agent="coder"></ship-pipeline-step>')
    expect(el.shadowRoot!.querySelector('.name')!.textContent).toBe('impl')
    expect(el.shadowRoot!.querySelector('.type')!.textContent!.toLowerCase()).toContain('agent')
    expect(el.shadowRoot!.querySelector('.type')!.textContent).toContain('coder')
    expect(el.shadowRoot!.querySelector('.status')!.textContent).toBe('completed')
  })

  it('uses a spinner when running', async () => {
    const el = await mount('<ship-pipeline-step name="x" state="running"></ship-pipeline-step>')
    expect(el.shadowRoot!.querySelector('ship-spinner')).toBeTruthy()
  })

  it('shows failed glyph', async () => {
    const el = await mount('<ship-pipeline-step name="x" state="failed"></ship-pipeline-step>')
    expect(el.shadowRoot!.querySelector('.icon')!.textContent).toBe('\u2717')
  })
})
