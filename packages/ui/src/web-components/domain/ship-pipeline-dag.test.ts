import { afterEach, describe, expect, it } from 'vitest'
import './ship-pipeline-dag'
import './ship-pipeline-step'
import type { ShipPipelineDag } from './ship-pipeline-dag'
import { cleanupTestDom } from '../util'

describe('ship-pipeline-dag', () => {
  afterEach(cleanupTestDom)

  it('renders projected step children', async () => {
    document.body.insertAdjacentHTML('beforeend', `
      <ship-pipeline-dag>
        <ship-pipeline-step name="a" state="completed"></ship-pipeline-step>
        <span class="arrow">&rarr;</span>
        <ship-pipeline-step name="b" state="running"></ship-pipeline-step>
      </ship-pipeline-dag>
    `)
    const el = document.body.lastElementChild as ShipPipelineDag
    await el.updateComplete
    expect(el.querySelectorAll('ship-pipeline-step').length).toBe(2)
    expect(el.querySelectorAll('.arrow').length).toBe(1)
  })
})
