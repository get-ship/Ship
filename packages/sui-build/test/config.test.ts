import { describe, it, expect } from 'vitest'
import type { SuiBuildConfig, BuildResult, SuiBundler, AdapterContext } from '../src/config.js'

describe('SuiBuildConfig', () => {
  it('accepts minimal config with entries and outdir', () => {
    const config: SuiBuildConfig = {
      entries: { main: 'src/index.ts' },
      outdir: 'dist',
    }
    expect(config.entries).toEqual({ main: 'src/index.ts' })
    expect(config.outdir).toBe('dist')
    expect(config.mode).toBeUndefined()
    expect(config.sourcemap).toBeUndefined()
    expect(config.lazyBundles).toBeUndefined()
  })

  it('accepts full config with all optional fields', () => {
    const config: SuiBuildConfig = {
      entries: { main: 'src/index.ts', shell: 'src/shell.ts' },
      lazyBundles: { modal: 'src/modal.ts', panel: 'src/panel.ts' },
      outdir: 'dist',
      mode: 'development',
      sourcemap: true,
    }
    expect(config.mode).toBe('development')
    expect(config.sourcemap).toBe(true)
    expect(config.lazyBundles?.modal).toBe('src/modal.ts')
    expect(Object.keys(config.entries)).toHaveLength(2)
  })

  it('accepts production mode with sourcemap disabled', () => {
    const config: SuiBuildConfig = {
      entries: { app: 'src/app.ts' },
      outdir: 'public/assets',
      mode: 'production',
      sourcemap: false,
    }
    expect(config.mode).toBe('production')
    expect(config.sourcemap).toBe(false)
  })

  it('allows multiple lazy bundles', () => {
    const config: SuiBuildConfig = {
      entries: { main: 'src/main.ts' },
      lazyBundles: {
        dialogs: 'src/dialogs.ts',
        charts: 'src/charts.ts',
        admin: 'src/admin.ts',
      },
      outdir: 'dist',
    }
    expect(Object.keys(config.lazyBundles ?? {})).toHaveLength(3)
  })
})

describe('BuildResult', () => {
  it('has outputs, warnings, and durationMs', () => {
    const result: BuildResult = {
      outputs: ['/tmp/dist/main.js', '/tmp/dist/modal.js'],
      warnings: [],
      durationMs: 42,
    }
    expect(result.outputs).toHaveLength(2)
    expect(result.warnings).toHaveLength(0)
    expect(result.durationMs).toBe(42)
  })

  it('carries non-fatal warnings', () => {
    const result: BuildResult = {
      outputs: [],
      warnings: ['unused export in src/foo.ts', 'large bundle: 2MB'],
      durationMs: 100,
    }
    expect(result.warnings).toHaveLength(2)
  })
})

describe('AdapterContext', () => {
  it('has packageRoot and outdir as absolute paths', () => {
    const ctx: AdapterContext = {
      packageRoot: '/home/user/project',
      outdir: '/home/user/project/dist',
    }
    expect(ctx.packageRoot).toBe('/home/user/project')
    expect(ctx.outdir).toBe('/home/user/project/dist')
  })
})

describe('SuiBundler interface', () => {
  it('accepts a bun-named adapter', () => {
    const adapter: SuiBundler = {
      name: 'bun',
      async build() {
        return { outputs: [], warnings: [], durationMs: 0 }
      },
    }
    expect(adapter.name).toBe('bun')
  })

  it('accepts an esbuild-named adapter', () => {
    const adapter: SuiBundler = {
      name: 'esbuild',
      async build() {
        return { outputs: [], warnings: [], durationMs: 0 }
      },
    }
    expect(adapter.name).toBe('esbuild')
  })
})
