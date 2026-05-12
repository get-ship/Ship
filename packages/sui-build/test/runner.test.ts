import { describe, it, expect, vi, beforeAll, afterAll } from 'vitest'
import { mkdtemp, writeFile, mkdir } from 'node:fs/promises'
import { rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { run } from '../src/runner.js'
import type { SuiBuildConfig, AdapterContext, SuiBundler } from '../src/config.js'

function mockAdapter(warnings: string[] = []): SuiBundler {
  return {
    name: 'esbuild',
    async build(_config, ctx) {
      return { outputs: [join(ctx.outdir, 'app.js')], warnings, durationMs: 1 }
    },
  }
}

describe('runner', () => {
  let tmpDir: string
  let config: SuiBuildConfig
  let ctx: AdapterContext

  beforeAll(async () => {
    tmpDir = await mkdtemp(join(tmpdir(), 'sui-build-runner-'))
    await mkdir(join(tmpDir, 'src'))
    await mkdir(join(tmpDir, 'dist'), { recursive: true })
    await writeFile(join(tmpDir, 'src', 'entry.ts'), `export const app = 'hello'`)
    await writeFile(join(tmpDir, 'src', 'hello.sui'), `<span>hello</span>`)

    config = { entries: { app: 'src/entry.ts' }, outdir: 'dist' }
    ctx = { packageRoot: tmpDir, outdir: join(tmpDir, 'dist') }
  })

  afterAll(() => {
    rmSync(tmpDir, { recursive: true, force: true })
  })

  it('invokes spawn for each .sui file found in source dirs', async () => {
    const spawned: Array<[string, string]> = []
    await run(config, ctx, mockAdapter(), async (suiPath, stem) => {
      spawned.push([suiPath, stem])
    })
    expect(spawned).toHaveLength(1)
    expect(spawned[0][1]).toBe('hello')
    expect(spawned[0][0]).toContain('hello.sui')
  })

  it('skips unchanged files on a second run (mtime cache hit)', async () => {
    const spawned: string[] = []
    const spawn = async (_path: string, stem: string) => { spawned.push(stem) }
    // First run populates cache
    await run(config, ctx, mockAdapter(), spawn)
    const afterFirst = spawned.length
    // Second run — same mtimes — should not spawn again
    await run(config, ctx, mockAdapter(), spawn)
    expect(spawned.length).toBe(afterFirst)
  })

  it('re-compiles a file whose content has changed (mtime updated)', async () => {
    const suiFile = join(tmpDir, 'src', 'hello.sui')
    // Overwrite to update mtime
    await writeFile(suiFile, `<span>updated</span>`)
    const spawned: string[] = []
    await run(config, ctx, mockAdapter(), async (_p, stem) => spawned.push(stem))
    expect(spawned).toContain('hello')
  })

  it('calls the adapter after the compile phase', async () => {
    const buildFn = vi.fn().mockResolvedValue({ outputs: [], warnings: [], durationMs: 0 })
    const adapter: SuiBundler = { name: 'esbuild', build: buildFn }
    await run(config, ctx, adapter, async () => {})
    expect(buildFn).toHaveBeenCalledOnce()
    expect(buildFn).toHaveBeenCalledWith(config, ctx)
  })

  it('merges adapter warnings into the returned result', async () => {
    const result = await run(config, ctx, mockAdapter(['adapter-warn']), async () => {})
    expect(result.warnings).toContain('adapter-warn')
  })

  it('returns durationMs that is a non-negative number', async () => {
    const result = await run(config, ctx, mockAdapter(), async () => {})
    expect(result.durationMs).toBeGreaterThanOrEqual(0)
  })

  it('handles lazyBundles directories the same as entries', async () => {
    await mkdir(join(tmpDir, 'lazy'), { recursive: true })
    await writeFile(join(tmpDir, 'lazy', 'widget.ts'), `export const w = 1`)
    await writeFile(join(tmpDir, 'lazy', 'widget.sui'), `<div>widget</div>`)

    const lazyConfig: SuiBuildConfig = {
      entries: { app: 'src/entry.ts' },
      lazyBundles: { widget: 'lazy/widget.ts' },
      outdir: 'dist',
    }
    const spawned: string[] = []
    await run(lazyConfig, ctx, mockAdapter(), async (_p, stem) => spawned.push(stem))
    expect(spawned).toContain('widget')
  })
})
