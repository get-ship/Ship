import { afterEach, describe, expect, it } from 'vitest'
import './ship-git-diff'
import type { ShipGitDiff, ShipGitDiffFile, ShipGitDiffLine, ShipGitDiffHunk } from './ship-git-diff'
import { cleanupTestDom } from '../util'

async function mount<T extends HTMLElement>(markup: string): Promise<T> {
  document.body.insertAdjacentHTML('beforeend', markup)
  const el = document.body.lastElementChild as T
  // @ts-expect-error — updateComplete exists on Lit elements
  if (typeof el.updateComplete?.then === 'function') await el.updateComplete
  for (const child of Array.from(el.querySelectorAll('*'))) {
    // @ts-expect-error — nested Lit elements
    if (typeof child.updateComplete?.then === 'function') await child.updateComplete
  }
  return el
}

describe('ship-git-diff suite', () => {
  afterEach(cleanupTestDom)

  it('renders toolbar with file count and totals', async () => {
    const el = await mount<ShipGitDiff>(
      '<ship-git-diff files="2" additions="5" deletions="3"></ship-git-diff>',
    )
    const text = el.shadowRoot!.textContent || ''
    expect(text).toContain('2 files changed')
    expect(text).toContain('+5')
    expect(text).toContain('-3')
  })

  it('renders empty state with no files', async () => {
    const el = await mount<ShipGitDiff>('<ship-git-diff files="0" empty></ship-git-diff>')
    expect(el.shadowRoot!.textContent).toContain('No changes')
  })

  it('collapse-all toggles every child file', async () => {
    const el = await mount<ShipGitDiff>(`
      <ship-git-diff files="2" additions="1" deletions="0">
        <ship-git-diff-file path="a.ts" additions="1" deletions="0"></ship-git-diff-file>
        <ship-git-diff-file path="b.ts" additions="0" deletions="0"></ship-git-diff-file>
      </ship-git-diff>
    `)
    const btn = el.shadowRoot!.querySelector('.toggle') as HTMLButtonElement
    btn.click()
    await el.updateComplete
    const files = el.querySelectorAll('ship-git-diff-file') as NodeListOf<ShipGitDiffFile>
    expect(files[0].collapsed).toBe(true)
    expect(files[1].collapsed).toBe(true)
  })

  it('file header shows path and stats', async () => {
    const el = await mount<ShipGitDiffFile>(
      '<ship-git-diff-file path="src/foo.rs" additions="3" deletions="1"></ship-git-diff-file>',
    )
    const sr = el.shadowRoot!
    expect(sr.querySelector('.path')!.textContent).toBe('src/foo.rs')
    expect(sr.textContent).toContain('3')
    expect(sr.textContent).toContain('1')
  })

  it('file header click toggles collapsed state', async () => {
    const el = await mount<ShipGitDiffFile>('<ship-git-diff-file path="a.ts"></ship-git-diff-file>')
    const btn = el.shadowRoot!.querySelector('.header') as HTMLButtonElement
    btn.click()
    await el.updateComplete
    expect(el.collapsed).toBe(true)
  })

  it('hunk renders header text', async () => {
    const el = await mount<ShipGitDiffHunk>(
      '<ship-git-diff-hunk header="@@ -1,5 +1,7 @@"></ship-git-diff-hunk>',
    )
    expect(el.shadowRoot!.textContent).toContain('@@ -1,5 +1,7 @@')
  })

  it('line renders old/new numbers and signature', async () => {
    const el = await mount<ShipGitDiffLine>(
      '<ship-git-diff-line type="add" old="" new="42">hello</ship-git-diff-line>',
    )
    const sr = el.shadowRoot!
    const nums = sr.querySelectorAll('.num')
    expect(nums[0].textContent).toBe('')
    expect(nums[1].textContent).toBe('42')
    expect(sr.querySelector('.sig')!.textContent).toBe('+')
  })

  it('del line has destructive signature', async () => {
    const el = await mount<ShipGitDiffLine>(
      '<ship-git-diff-line type="del" old="7" new="">gone</ship-git-diff-line>',
    )
    expect(el.shadowRoot!.querySelector('.sig')!.textContent).toBe('-')
  })
})
