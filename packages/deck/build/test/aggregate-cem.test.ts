import { afterAll, beforeAll, describe, expect, it } from 'vitest'
import { mkdtemp, mkdir, writeFile } from 'node:fs/promises'
import { rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { aggregateCem, aggregateLoadedCem, stringifyCem } from '../src/aggregate-cem.js'

interface ScratchDir {
  root: string
  write: (rel: string, content: string) => Promise<string>
}

async function makeScratch(prefix: string): Promise<ScratchDir> {
  const root = await mkdtemp(join(tmpdir(), `aggregate-cem-${prefix}-`))
  return {
    root,
    async write(rel: string, content: string) {
      const abs = join(root, rel)
      await mkdir(join(abs, '..'), { recursive: true })
      await writeFile(abs, content, 'utf8')
      return abs
    },
  }
}

function moduleFor(tagName: string, className?: string) {
  return {
    kind: 'javascript-module',
    path: `${tagName}.sui`,
    declarations: [
      {
        kind: 'class',
        name: className ?? toClass(tagName),
        tagName,
        customElement: true,
      },
    ],
    exports: [
      {
        kind: 'custom-element-definition',
        name: tagName,
        declaration: { name: className ?? toClass(tagName), module: `${tagName}.sui` },
      },
    ],
  }
}

function toClass(tagName: string): string {
  return tagName
    .split('-')
    .map((s) => s.charAt(0).toUpperCase() + s.slice(1))
    .join('')
}

function cemFile(...modules: ReturnType<typeof moduleFor>[]): string {
  return JSON.stringify({ schemaVersion: '1.0.0', readme: '', modules }, null, 2)
}

describe('aggregateCem', () => {
  let scratches: ScratchDir[] = []

  beforeAll(() => {
    scratches = []
  })

  afterAll(() => {
    for (const s of scratches) {
      rmSync(s.root, { recursive: true, force: true })
    }
  })

  async function mkScratch(prefix: string): Promise<ScratchDir> {
    const s = await makeScratch(prefix)
    scratches.push(s)
    return s
  }

  it('aggregates two per-component CEMs in alphabetic order by path', async () => {
    const s = await mkScratch('happy')
    await s.write('sui-zebra/sui-zebra.custom-elements.json', cemFile(moduleFor('sui-zebra')))
    await s.write('sui-alpha/sui-alpha.custom-elements.json', cemFile(moduleFor('sui-alpha')))

    const root = await aggregateCem({ sources: [s.root] })

    expect(root.schemaVersion).toBe('1.0.0')
    expect(root.readme).toBe('')
    expect(root.modules).toHaveLength(2)
    expect(root.modules.map((m) => m.path)).toEqual(['sui-alpha.sui', 'sui-zebra.sui'])
  })

  it('warns and keeps first-path on duplicate tagName', async () => {
    const s = await mkScratch('dupe')
    await s.write('a-first/sui-dupe.custom-elements.json', cemFile(moduleFor('sui-dupe', 'First')))
    await s.write('z-second/sui-dupe.custom-elements.json', cemFile(moduleFor('sui-dupe', 'Second')))

    const warnings: string[] = []
    const root = await aggregateCem({ sources: [s.root], warn: (m) => warnings.push(m) })

    expect(root.modules).toHaveLength(1)
    expect(root.modules[0]?.declarations?.[0]?.name).toBe('First')
    expect(warnings).toHaveLength(1)
    expect(warnings[0]).toMatch(/duplicate tagName "sui-dupe"/)
    expect(warnings[0]).toContain('z-second')
  })

  it('returns empty modules for empty source dir', async () => {
    const s = await mkScratch('empty')
    const root = await aggregateCem({ sources: [s.root] })
    expect(root.modules).toEqual([])
    expect(root.schemaVersion).toBe('1.0.0')
  })

  it('produces byte-identical output across runs (determinism)', async () => {
    const s = await mkScratch('stable')
    await s.write('sui-c/sui-c.custom-elements.json', cemFile(moduleFor('sui-c')))
    await s.write('sui-a/sui-a.custom-elements.json', cemFile(moduleFor('sui-a')))
    await s.write('sui-b/sui-b.custom-elements.json', cemFile(moduleFor('sui-b')))

    const first = stringifyCem(await aggregateCem({ sources: [s.root] }))
    const second = stringifyCem(await aggregateCem({ sources: [s.root] }))

    expect(first).toBe(second)
    expect(first.endsWith('\n')).toBe(true)
  })

  it('skips node_modules, dist, target', async () => {
    const s = await mkScratch('skip')
    await s.write('sui-real/sui-real.custom-elements.json', cemFile(moduleFor('sui-real')))
    await s.write('node_modules/sui-bad/sui-bad.custom-elements.json', cemFile(moduleFor('sui-bad')))
    await s.write('dist/sui-bad2/sui-bad2.custom-elements.json', cemFile(moduleFor('sui-bad2')))
    await s.write('target/sui-bad3/sui-bad3.custom-elements.json', cemFile(moduleFor('sui-bad3')))

    const root = await aggregateCem({ sources: [s.root] })
    expect(root.modules.map((m) => m.path)).toEqual(['sui-real.sui'])
  })

  it('aggregateLoadedCem matches filesystem-walk output byte-for-byte', async () => {
    const s = await mkScratch('preloaded-parity')
    const aPath = await s.write(
      'sui-a/sui-a.custom-elements.json',
      cemFile(moduleFor('sui-a')),
    )
    const bPath = await s.write(
      'sui-b/sui-b.custom-elements.json',
      cemFile(moduleFor('sui-b')),
    )
    const cPath = await s.write(
      'sui-c/sui-c.custom-elements.json',
      cemFile(moduleFor('sui-c')),
    )

    const fromFs = stringifyCem(await aggregateCem({ sources: [s.root] }))

    // Pre-loaded variant, deliberately unsorted to prove the function sorts
    // before deduping/merging.
    const preloaded = stringifyCem(
      aggregateLoadedCem([
        { filePath: cPath, modules: [moduleFor('sui-c')] },
        { filePath: aPath, modules: [moduleFor('sui-a')] },
        { filePath: bPath, modules: [moduleFor('sui-b')] },
      ]),
    )

    expect(preloaded).toBe(fromFs)
  })

  it('aggregateLoadedCem warns on duplicate tagName and keeps first-path', () => {
    const warnings: string[] = []
    const root = aggregateLoadedCem(
      [
        { filePath: '/tmp/z-second/sui-dupe.sui', modules: [moduleFor('sui-dupe', 'Second')] },
        { filePath: '/tmp/a-first/sui-dupe.sui', modules: [moduleFor('sui-dupe', 'First')] },
      ],
      { warn: (m) => warnings.push(m) },
    )

    expect(root.modules).toHaveLength(1)
    expect(root.modules[0]?.declarations?.[0]?.name).toBe('First')
    expect(warnings).toHaveLength(1)
    expect(warnings[0]).toMatch(/duplicate tagName "sui-dupe"/)
    expect(warnings[0]).toContain('z-second')
  })

  it('honors schemaVersion and readme overrides', async () => {
    const s = await mkScratch('opts')
    await s.write('sui-x/sui-x.custom-elements.json', cemFile(moduleFor('sui-x')))

    const root = await aggregateCem({
      sources: [s.root],
      schemaVersion: '2.0.0',
      readme: '# CEM',
    })

    expect(root.schemaVersion).toBe('2.0.0')
    expect(root.readme).toBe('# CEM')
  })
})
