import { describe, it, expect, vi, beforeAll, afterAll } from 'vitest'
import { mkdtemp, writeFile, mkdir, readFile, unlink } from 'node:fs/promises'
import { rmSync, existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { basename, dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { run } from '../src/runner.js'
import { makeFfiSpawn } from '../src/adapters/bun.js'
import type { SuiBuildConfig, AdapterContext, SuiBundler } from '../src/config.js'

const thisDir = dirname(fileURLToPath(import.meta.url))

function findWorkspaceRoot(start: string): string {
  let dir = start
  while (true) {
    if (existsSync(join(dir, 'Cargo.toml'))) return dir
    const parent = dirname(dir)
    if (parent === dir) return start
    dir = parent
  }
}

const wsRoot = findWorkspaceRoot(thisDir)
const ext = process.platform === 'darwin' ? 'dylib' : process.platform === 'win32' ? 'dll' : 'so'
const prefix = process.platform === 'win32' ? '' : 'lib'
const cdylibPath = join(wsRoot, 'target', 'debug', `${prefix}sui_compiler.${ext}`)
const isBun = !!process.versions.bun
const hasCdylib = existsSync(cdylibPath)

function mockAdapter(warnings: string[] = []): SuiBundler {
  return {
    name: 'bun',
    async build(_config, ctx) {
      return { outputs: [join(ctx.outdir, 'app.js')], warnings, durationMs: 1 }
    },
  }
}

/**
 * Mock spawn that mimics sui-compiler's out-of-repo fallback: write a stub
 * `<stem>.compiled.js` into `<src>/dist/`. The runner's cache check requires
 * the artifact to exist; using a writing spawn lets tests exercise the real
 * cache-hit path.
 */
async function writingSpawn(suiPath: string, stem: string): Promise<void> {
  const distDir = join(dirname(suiPath), 'dist')
  await mkdir(distDir, { recursive: true })
  await writeFile(join(distDir, `${stem}.compiled.js`), `// stub for ${basename(suiPath)}`)
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
    const spawn = async (suiPath: string, stem: string) => {
      spawned.push(stem)
      await writingSpawn(suiPath, stem)
    }
    // First run populates cache
    await run(config, ctx, mockAdapter(), spawn)
    const afterFirst = spawned.length
    // Second run — same mtimes, artifact still present — should not spawn again
    await run(config, ctx, mockAdapter(), spawn)
    expect(spawned.length).toBe(afterFirst)
  })

  it('re-compiles when the cached artifact is missing (regression: stale cache vs wiped mirror)', async () => {
    const spawned: string[] = []
    const spawn = async (suiPath: string, stem: string) => {
      spawned.push(stem)
      await writingSpawn(suiPath, stem)
    }
    // Populate the cache + artifact
    await run(config, ctx, mockAdapter(), spawn)
    const afterFirst = spawned.length
    // Wipe the artifact (simulates `rm -rf .sui-out` while
    // `.sui-build-cache.json` still records the source's mtime)
    const artifact = join(tmpDir, 'src', 'dist', 'hello.compiled.js')
    expect(existsSync(artifact)).toBe(true)
    await unlink(artifact)
    // Same mtime in cache, but artifact missing → must re-spawn
    await run(config, ctx, mockAdapter(), spawn)
    expect(spawned.length).toBeGreaterThan(afterFirst)
    expect(existsSync(artifact)).toBe(true)
  })

  it('re-compiles a file whose content has changed (mtime updated)', async () => {
    const suiFile = join(tmpDir, 'src', 'hello.sui')
    // Overwrite to update mtime
    await writeFile(suiFile, `<span>updated</span>`)
    const spawned: string[] = []
    await run(config, ctx, mockAdapter(), async (suiPath, stem) => {
      spawned.push(stem)
      await writingSpawn(suiPath, stem)
    })
    expect(spawned).toContain('hello')
  })

  it('calls the adapter after the compile phase', async () => {
    const buildFn = vi.fn().mockResolvedValue({ outputs: [], warnings: [], durationMs: 0 })
    const adapter: SuiBundler = { name: 'bun', build: buildFn }
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
    await run(lazyConfig, ctx, mockAdapter(), async (suiPath, stem) => {
      spawned.push(stem)
      await writingSpawn(suiPath, stem)
    })
    expect(spawned).toContain('widget')
  })
})

describe.skipIf(!isBun || !hasCdylib)('runner FFI path', () => {
  let ffiTmpDir: string
  let ffiCtx: AdapterContext
  let ffiConfig: SuiBuildConfig

  beforeAll(async () => {
    ffiTmpDir = await mkdtemp(join(tmpdir(), 'sui-build-ffi-'))
    await mkdir(join(ffiTmpDir, 'src'))
    await mkdir(join(ffiTmpDir, 'dist'), { recursive: true })
    await writeFile(join(ffiTmpDir, 'src', 'entry.ts'), `export const app = 'hello'`)
    await writeFile(
      join(ffiTmpDir, 'src', 'hello.sui'),
      `<prop name="label" type="string" default="world" />\n<span>{label()}</span>`,
    )
    ffiConfig = { entries: { app: 'src/entry.ts' }, outdir: 'dist' }
    ffiCtx = { packageRoot: ffiTmpDir, outdir: join(ffiTmpDir, 'dist') }
  })

  afterAll(() => {
    rmSync(ffiTmpDir, { recursive: true, force: true })
  })

  it('makeFfiSpawn returns a non-null spawn function', async () => {
    const spawn = await makeFfiSpawn(cdylibPath)
    expect(spawn).not.toBeNull()
  })

  it('FFI spawn compiles a .sui file and writes .compiled.js', async () => {
    const spawn = await makeFfiSpawn(cdylibPath)
    expect(spawn).not.toBeNull()
    const suiPath = join(ffiTmpDir, 'src', 'hello.sui')
    await spawn!(suiPath, 'hello')
    const outPath = join(ffiTmpDir, 'src', 'dist', 'hello.compiled.js')
    expect(existsSync(outPath)).toBe(true)
    const content = await readFile(outPath, 'utf8')
    expect(content).toContain('sui-runtime')
  })

  it('FFI spawn emits a .sui.d.ts into the canonical output dir', async () => {
    const spawn = await makeFfiSpawn(cdylibPath)
    expect(spawn).not.toBeNull()
    const suiPath = join(ffiTmpDir, 'src', 'hello.sui')
    await spawn!(suiPath, 'hello')
    // Outside the workspace, the compiler falls back to a sibling
    // `dist/` directory; in-tree it lands in the `.sui-out/` mirror
    // (see sui-compiler::out_layout::resolve_out_dir).
    const dtsPath = join(ffiTmpDir, 'src', 'dist', 'hello.sui.d.ts')
    expect(existsSync(dtsPath)).toBe(true)
    const dts = await readFile(dtsPath, 'utf8')
    // The emitter walks <prop> declarations into the Scope interface.
    expect(dts).toContain('Scope')
    expect(dts).toContain('label')
  })

  it('FFI and spawn paths produce equivalent JS output', async () => {
    const suiPath = join(ffiTmpDir, 'src', 'hello.sui')

    // FFI path
    const ffiSpawn = await makeFfiSpawn(cdylibPath)
    expect(ffiSpawn).not.toBeNull()
    await ffiSpawn!(suiPath, 'hello')
    const ffiOut = await readFile(join(ffiTmpDir, 'src', 'dist', 'hello.compiled.js'), 'utf8')

    // Spawn path (re-compile after touching mtime)
    await writeFile(suiPath, await readFile(suiPath, 'utf8'))
    const spawnedStems: string[] = []
    await run(ffiConfig, ffiCtx, mockAdapter(), async (_p, stem) => {
      spawnedStems.push(stem)
      await ffiSpawn!(_p, stem)
    })
    const spawnOut = await readFile(join(ffiTmpDir, 'src', 'dist', 'hello.compiled.js'), 'utf8')

    expect(ffiOut).toBe(spawnOut)
    expect(spawnedStems).toContain('hello')
  })

  it('SUI_BUILD_DISABLE_FFI=1 causes makeFfiSpawn to return null', async () => {
    process.env['SUI_BUILD_DISABLE_FFI'] = '1'
    try {
      const spawn = await makeFfiSpawn(cdylibPath)
      expect(spawn).toBeNull()
    } finally {
      delete process.env['SUI_BUILD_DISABLE_FFI']
    }
  })
})
