import { describe, expect, it } from 'bun:test'
import { resolve } from 'node:path'

import { defaultDetect } from '../src/detect'

describe('defaultDetect', () => {
  it('flags <dir>/<dir>.sui as a component named <dir>', () => {
    const got = defaultDetect(resolve('/repo/packages/ui-components/src/sui-button/sui-button.sui'))
    expect(got).toEqual({ kind: 'component', meta: { name: 'sui-button', shadow: true } })
  })

  it('flags views/<name>/page.sui as a view', () => {
    const got = defaultDetect(resolve('/repo/views/counter/page.sui'))
    expect(got).toEqual({ kind: 'view' })
  })

  it('flags an unrelated .sui file as a view', () => {
    const got = defaultDetect(resolve('/tmp/spike-bun-vite/app.sui'))
    expect(got).toEqual({ kind: 'view' })
  })

  it('survives .sui files at the filesystem root', () => {
    const got = defaultDetect('/page.sui')
    expect(got).toEqual({ kind: 'view' })
  })

  it('returns a view for any non-.sui input (defensive only — plugin guards before calling)', () => {
    const got = defaultDetect('/somewhere/notsui.txt')
    expect(got).toEqual({ kind: 'view' })
  })
})
