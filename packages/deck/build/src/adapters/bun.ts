import { basename, dirname, join } from 'node:path'
import { existsSync } from 'node:fs'
import { mkdir } from 'node:fs/promises'
import type { SuiBundler, SuiBuildConfig, BuildResult, AdapterContext } from '../config.js'
import { resolveEntrypoints, defaultSourcemap } from './util.js'
import { mirrorResolverPlugin } from './import-rewrite.js'

type CompileSpawn = (suiPath: string, stem: string) => Promise<void>

/**
 * Create an FFI-backed compiler spawn using Bun.dlopen.
 * Returns null when not under Bun, when the cdylib is absent, or when
 * SUI_BUILD_DISABLE_FFI=1 is set.
 */
export async function makeFfiSpawn(libPath: string): Promise<CompileSpawn | null> {
  if (!process.versions.bun || !existsSync(libPath) || process.env['SUI_BUILD_DISABLE_FFI'] === '1') {
    return null
  }
  // @ts-expect-error bun:ffi is a Bun-only module; guarded by the runtime check above
  const { dlopen, FFIType, CString } = await import('bun:ffi')
  const lib = dlopen(libPath, {
    sui_compiler_compile: {
      args: [FFIType.cstring, FFIType.cstring, FFIType.cstring],
      returns: FFIType.ptr,
    },
    sui_compiler_free_string: {
      args: [FFIType.ptr],
      returns: FFIType.void,
    },
  })
  // Bun FFI's FFIType.cstring args want a null-terminated UTF-8 buffer,
  // not a JS string. Encode each arg before the call.
  const encoder = new TextEncoder()
  const toCString = (s: string): Uint8Array => {
    const bytes = encoder.encode(s)
    const buf = new Uint8Array(bytes.length + 1)
    buf.set(bytes)
    buf[bytes.length] = 0
    return buf
  }
  return async (suiPath: string, stem: string): Promise<void> => {
    // Empty out_dir tells the compiler to use its canonical mirror tree
    // (`<repo-root>/.sui-out/<src-rel>/`). Resolution happens inside
    // the Rust crate so the FFI surface stays minimal.
    const rawPtr = lib.symbols.sui_compiler_compile(
      toCString(suiPath),
      toCString(stem),
      toCString(''),
    )
    if (!rawPtr) throw new Error(`sui-compiler FFI returned null for ${suiPath}`)
    const jsonStr = new CString(rawPtr).toString()
    lib.symbols.sui_compiler_free_string(rawPtr)
    const parsed = JSON.parse(jsonStr) as { ok: boolean; error?: string }
    if (!parsed.ok) {
      throw new Error(`sui-compiler failed for ${suiPath}: ${parsed.error ?? 'unknown error'}`)
    }
  }
}

const BUN_LOADERS = new Set<string>([
  'base64', 'binary', 'css', 'dataurl', 'js', 'json', 'jsx', 'text', 'toml', 'ts', 'tsx',
])

export const bunAdapter: SuiBundler = {
  name: 'bun',

  async build(config: SuiBuildConfig, ctx: AdapterContext): Promise<BuildResult> {
    if (!process.versions.bun) {
      throw new Error(
        'bun adapter requires the Bun runtime; install bun or use --bundler esbuild',
      )
    }

    if (config.loaders) {
      for (const [ext, loader] of Object.entries(config.loaders)) {
        if (!BUN_LOADERS.has(loader)) {
          throw new Error(
            `sui-build bun adapter: unsupported loader "${loader}" for "${ext}". Valid loaders: ${[...BUN_LOADERS].sort().join(', ')}`,
          )
        }
      }
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const Bun = (globalThis as any).Bun as {
      build(opts: Record<string, unknown>): Promise<{
        success: boolean
        outputs: Array<{ path: string }>
        logs: Array<{ level: string; message: string }>
      }>
    }

    const t0 = Date.now()
    const entries = resolveEntrypoints(config, ctx)
    const sourcemap = defaultSourcemap(config)
    const outputs: string[] = []
    const warnings: string[] = []
    // Bun selects Lit's 'development' export condition unless process.env.NODE_ENV
    // is defined as 'production' in the build options. Pass it via `define` so
    // Bun resolves the production path without mutating the host process env.
    const nodeEnvDef = JSON.stringify(config.mode === 'production' ? 'production' : 'development')

    for (const [key, entrypoint] of Object.entries(entries)) {
      // Keys with a `/` denote subdirectory placement (e.g. lazy bundles
      // routed via `lazyOutSubdir`). Compute per-entry outdir + naming so
      // we don't depend on Bun's naming template honoring a literal slash.
      const subDir = dirname(key)
      const baseName = basename(key)
      const entryOutdir = subDir === '.' ? ctx.outdir : join(ctx.outdir, subDir)
      if (subDir !== '.') await mkdir(entryOutdir, { recursive: true })

      const result = await Bun.build({
        entrypoints: [entrypoint],
        outdir: entryOutdir,
        target: 'browser',
        format: 'esm',
        minify: config.mode === 'production',
        sourcemap: sourcemap ? 'inline' : 'none',
        naming: { entry: `${baseName}.js` },
        define: { 'process.env.NODE_ENV': nodeEnvDef },
        plugins: [mirrorResolverPlugin(ctx.packageRoot)],
        ...(config.loaders ? { loader: config.loaders } : {}),
      })

      if (!result.success) {
        const errs = result.logs.map((l) => l.message).join('\n')
        throw new Error(`bun adapter build failed for entry "${key}": ${errs}`)
      }

      outputs.push(join(entryOutdir, `${baseName}.js`))

      for (const log of result.logs) {
        if (log.level === 'warning') warnings.push(`[${key}] ${log.message}`)
      }
    }

    return { outputs, warnings, durationMs: Date.now() - t0 }
  },
}
