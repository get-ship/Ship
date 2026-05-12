import { describe, it, expect, beforeAll, afterAll } from 'vitest'
import { mkdtemp, writeFile, mkdir, readFile } from 'node:fs/promises'
import { rmSync, existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { esbuildAdapter } from '../../src/adapters/esbuild.js'
import type { SuiBuildConfig, AdapterContext } from '../../src/config.js'

describe('esbuild adapter', () => {
  let tmpDir: string
  let ctx: AdapterContext

  beforeAll(async () => {
    tmpDir = await mkdtemp(join(tmpdir(), 'sui-build-esbuild-'))
    await mkdir(join(tmpDir, 'src'))
    await mkdir(join(tmpDir, 'dist'), { recursive: true })

    await writeFile(
      join(tmpDir, 'src', 'entry.ts'),
      `export const greeting = 'hello from sui-build'\nexport default function greet() { return greeting }`,
    )
    await writeFile(
      join(tmpDir, 'src', 'secondary.ts'),
      `export const value = 42`,
    )

    ctx = { packageRoot: tmpDir, outdir: join(tmpDir, 'dist') }
  })

  afterAll(() => {
    rmSync(tmpDir, { recursive: true, force: true })
  })

  it('bundles a single entry and writes output to outdir', async () => {
    const config: SuiBuildConfig = {
      entries: { app: 'src/entry.ts' },
      outdir: 'dist',
    }
    const result = await esbuildAdapter.build(config, ctx)
    expect(result.outputs).toContain(join(tmpDir, 'dist', 'app.js'))
    expect(existsSync(join(tmpDir, 'dist', 'app.js'))).toBe(true)
  })

  it('output is valid ESM (contains export keyword)', async () => {
    const config: SuiBuildConfig = {
      entries: { esm: 'src/entry.ts' },
      outdir: 'dist',
    }
    await esbuildAdapter.build(config, ctx)
    const content = await readFile(join(tmpDir, 'dist', 'esm.js'), 'utf8')
    expect(content).toMatch(/export/)
  })

  it('bundles multiple entries into separate output files', async () => {
    const config: SuiBuildConfig = {
      entries: { main: 'src/entry.ts', side: 'src/secondary.ts' },
      outdir: 'dist',
    }
    const result = await esbuildAdapter.build(config, ctx)
    expect(result.outputs).toHaveLength(2)
    expect(existsSync(join(tmpDir, 'dist', 'main.js'))).toBe(true)
    expect(existsSync(join(tmpDir, 'dist', 'side.js'))).toBe(true)
  })

  it('minifies output in production mode', async () => {
    const devConfig: SuiBuildConfig = {
      entries: { dev: 'src/entry.ts' },
      outdir: 'dist',
      mode: 'development',
    }
    const prodConfig: SuiBuildConfig = {
      entries: { prod: 'src/entry.ts' },
      outdir: 'dist',
      mode: 'production',
    }

    await esbuildAdapter.build(devConfig, ctx)
    await esbuildAdapter.build(prodConfig, ctx)

    const devSize = (await readFile(join(tmpDir, 'dist', 'dev.js'))).length
    const prodSize = (await readFile(join(tmpDir, 'dist', 'prod.js'))).length
    expect(prodSize).toBeLessThan(devSize)
  })

  it('inlines sourcemap when sourcemap is enabled', async () => {
    const config: SuiBuildConfig = {
      entries: { mapped: 'src/entry.ts' },
      outdir: 'dist',
      mode: 'development',
      sourcemap: true,
    }
    await esbuildAdapter.build(config, ctx)
    const content = await readFile(join(tmpDir, 'dist', 'mapped.js'), 'utf8')
    expect(content).toContain('sourceMappingURL=data:application/json')
  })

  it('omits sourcemap in production by default', async () => {
    const config: SuiBuildConfig = {
      entries: { nosrc: 'src/entry.ts' },
      outdir: 'dist',
      mode: 'production',
    }
    await esbuildAdapter.build(config, ctx)
    const content = await readFile(join(tmpDir, 'dist', 'nosrc.js'), 'utf8')
    expect(content).not.toContain('sourceMappingURL')
  })

  it('reports durationMs as a positive number', async () => {
    const config: SuiBuildConfig = {
      entries: { timing: 'src/entry.ts' },
      outdir: 'dist',
    }
    const result = await esbuildAdapter.build(config, ctx)
    expect(result.durationMs).toBeGreaterThanOrEqual(0)
  })

  it('adapter name is "esbuild"', () => {
    expect(esbuildAdapter.name).toBe('esbuild')
  })

  it('inlines file content as a string when using the text loader', async () => {
    await mkdir(join(tmpDir, 'src', 'icons'), { recursive: true })
    await writeFile(join(tmpDir, 'src', 'icons', 'star.svg'), '<svg><path d="M12 2l3 7h7l-6 4 2 7-6-4-6 4 2-7-6-4h7z"/></svg>')
    await writeFile(
      join(tmpDir, 'src', 'with-svg.ts'),
      `import star from './icons/star.svg'\nexport default star`,
    )
    const config: SuiBuildConfig = {
      entries: { 'with-svg': 'src/with-svg.ts' },
      outdir: 'dist',
      loaders: { '.svg': 'text' },
    }
    await esbuildAdapter.build(config, ctx)
    const content = await readFile(join(tmpDir, 'dist', 'with-svg.js'), 'utf8')
    expect(content).toContain('<svg>')
  })

  it('throws a clear error for unsupported loader names', async () => {
    const config: SuiBuildConfig = {
      entries: { app: 'src/entry.ts' },
      outdir: 'dist',
      loaders: { '.xyz': 'not-a-real-loader' },
    }
    await expect(esbuildAdapter.build(config, ctx)).rejects.toThrow('unsupported loader')
  })

  it('does not minify when mode is explicitly development', async () => {
    const config: SuiBuildConfig = {
      entries: { 'dev-mode': 'src/entry.ts' },
      outdir: 'dist',
      mode: 'development',
    }
    await esbuildAdapter.build(config, ctx)
    const content = await readFile(join(tmpDir, 'dist', 'dev-mode.js'), 'utf8')
    expect(content).toContain('greeting')
  })
})
