import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import { mkdtemp, writeFile, mkdir } from 'node:fs/promises'
import { rmSync, existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { bunAdapter } from '../../src/adapters/bun.js'
import type { SuiBuildConfig, AdapterContext } from '../../src/config.js'

const isBun = Boolean(process.versions.bun)

describe.skipIf(!isBun)('bun adapter (Bun runtime only)', () => {
  let tmpDir: string
  let ctx: AdapterContext

  beforeAll(async () => {
    if (!isBun) return
    tmpDir = await mkdtemp(join(tmpdir(), 'sui-build-bun-'))
    await mkdir(join(tmpDir, 'src'))
    await mkdir(join(tmpDir, 'dist'), { recursive: true })
    await writeFile(
      join(tmpDir, 'src', 'entry.ts'),
      `export const greeting = 'hello from bun'\nexport default function greet() { return greeting }`,
    )
    ctx = { packageRoot: tmpDir, outdir: join(tmpDir, 'dist') }
  })

  afterAll(() => {
    if (tmpDir) rmSync(tmpDir, { recursive: true, force: true })
  })

  it('bundles entry and writes to outdir', async () => {
    const config: SuiBuildConfig = {
      entries: { app: 'src/entry.ts' },
      outdir: 'dist',
    }
    const result = await bunAdapter.build(config, ctx)
    expect(result.outputs).toContain(join(tmpDir, 'dist', 'app.js'))
    expect(existsSync(join(tmpDir, 'dist', 'app.js'))).toBe(true)
  })

  it('reports durationMs as a non-negative number', async () => {
    const config: SuiBuildConfig = {
      entries: { timing: 'src/entry.ts' },
      outdir: 'dist',
    }
    const result = await bunAdapter.build(config, ctx)
    expect(result.durationMs).toBeGreaterThanOrEqual(0)
  })

  it('adapter name is "bun"', () => {
    expect(bunAdapter.name).toBe('bun')
  })
})

describe.skipIf(isBun)('bun adapter (Node runtime — error path)', () => {
  it('throws a clear error when not running under Bun', async () => {
    const config: SuiBuildConfig = {
      entries: { app: 'src/entry.ts' },
      outdir: 'dist',
    }
    const ctx: AdapterContext = { packageRoot: '/tmp', outdir: '/tmp/dist' }
    await expect(bunAdapter.build(config, ctx)).rejects.toThrow(
      'bun adapter requires the Bun runtime',
    )
  })
})
