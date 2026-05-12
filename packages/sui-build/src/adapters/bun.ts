import { join } from 'node:path'
import type { SuiBundler, SuiBuildConfig, BuildResult, AdapterContext } from '../config.js'
import { resolveEntrypoints, defaultSourcemap } from './util.js'

export const bunAdapter: SuiBundler = {
  name: 'bun',

  async build(config: SuiBuildConfig, ctx: AdapterContext): Promise<BuildResult> {
    if (!process.versions.bun) {
      throw new Error(
        'bun adapter requires the Bun runtime; install bun or use --bundler esbuild',
      )
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

    for (const [key, entrypoint] of Object.entries(entries)) {
      const result = await Bun.build({
        entrypoints: [entrypoint],
        outdir: ctx.outdir,
        target: 'browser',
        format: 'esm',
        minify: config.mode === 'production',
        sourcemap: sourcemap ? 'inline' : 'none',
        naming: { entry: `${key}.js` },
      })

      if (!result.success) {
        const errs = result.logs.map((l) => l.message).join('\n')
        throw new Error(`bun adapter build failed for entry "${key}": ${errs}`)
      }

      outputs.push(join(ctx.outdir, `${key}.js`))

      for (const log of result.logs) {
        if (log.level === 'warning') warnings.push(`[${key}] ${log.message}`)
      }
    }

    return { outputs, warnings, durationMs: Date.now() - t0 }
  },
}
