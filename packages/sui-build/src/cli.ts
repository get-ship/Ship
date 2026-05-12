import { dirname, resolve } from 'node:path'
import { existsSync, mkdirSync } from 'node:fs'
import { watch } from 'chokidar'
import { bunAdapter } from './adapters/bun.js'
import { esbuildAdapter } from './adapters/esbuild.js'
import { run, collectSuiFiles } from './runner.js'
import type { SuiBuildConfig, SuiBundler, AdapterContext } from './config.js'

const HELP = `
sui-build — SUI bundling pipeline

Usage:
  sui-build [options]

Options:
  --config <path>        Path to sui.config.ts (default: sui.config.ts in cwd)
  --bundler bun|esbuild  Bundler adapter to use (default: bun if available)
  --watch                Re-build on source changes
  --mode development|production  Build mode (default: production)
  --help                 Show this message
`.trim()

interface CliArgs {
  config: string
  bundler: 'bun' | 'esbuild' | null
  watch: boolean
  mode: 'development' | 'production'
}

function parseArgs(argv: string[]): CliArgs {
  const args: CliArgs = {
    config: resolve(process.cwd(), 'sui.config.ts'),
    bundler: null,
    watch: false,
    mode: 'production',
  }
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]
    if (arg === '--help' || arg === '-h') {
      console.log(HELP)
      process.exit(0)
    } else if (arg === '--config') {
      args.config = resolve(argv[++i] ?? '')
    } else if (arg === '--bundler') {
      const b = argv[++i]
      if (b !== 'bun' && b !== 'esbuild') {
        console.error(`sui-build: --bundler must be "bun" or "esbuild", got: ${b}`)
        process.exit(1)
      }
      args.bundler = b
    } else if (arg === '--watch') {
      args.watch = true
    } else if (arg === '--mode') {
      const m = argv[++i]
      if (m !== 'development' && m !== 'production') {
        console.error(`sui-build: --mode must be "development" or "production", got: ${m}`)
        process.exit(1)
      }
      args.mode = m
    }
  }
  return args
}

function pickAdapter(bundler: 'bun' | 'esbuild' | null): SuiBundler {
  if (bundler === 'esbuild') return esbuildAdapter
  if (bundler === 'bun') return bunAdapter

  // Auto-select: prefer bun when running under bun runtime.
  const envBundler = process.env['SUI_BUILD_BUNDLER']
  if (envBundler === 'esbuild') return esbuildAdapter
  if (envBundler === 'bun') return bunAdapter

  if (process.versions.bun) return bunAdapter

  console.warn('sui-build: bun not found, falling back to esbuild bundler')
  return esbuildAdapter
}

async function loadConfig(configPath: string): Promise<SuiBuildConfig> {
  if (!process.versions.bun) {
    console.error(
      'sui-build CLI requires Bun runtime to load sui.config.ts — run with `bun sui-build`',
    )
    process.exit(1)
  }
  if (!existsSync(configPath)) {
    console.error(`sui-build: config not found: ${configPath}`)
    process.exit(1)
  }
  const mod = await import(`file://${configPath}`)
  const config: SuiBuildConfig = mod.default ?? mod
  return config
}

async function buildOnce(
  config: SuiBuildConfig,
  ctx: AdapterContext,
  adapter: SuiBundler,
): Promise<void> {
  const result = await run(config, ctx, adapter)
  for (const w of result.warnings) console.warn(`[sui-build] warn: ${w}`)
  console.log(
    `[sui-build] built ${result.outputs.length} output(s) in ${result.durationMs}ms via ${adapter.name}`,
  )
  for (const o of result.outputs) console.log(`  ${o}`)
}

async function watchMode(
  config: SuiBuildConfig,
  ctx: AdapterContext,
  adapter: SuiBundler,
): Promise<void> {
  console.log('[sui-build] watching...')
  await buildOnce(config, ctx, adapter)

  const srcDirs = [
    ...new Set(
      Object.values({ ...config.entries, ...(config.lazyBundles ?? {}) }).map(
        (src) => dirname(resolve(ctx.packageRoot, src)),
      ),
    ),
  ]

  const watcher = watch(srcDirs, { ignoreInitial: true, persistent: true })
  let debounce: ReturnType<typeof setTimeout> | null = null

  watcher.on('all', () => {
    if (debounce) clearTimeout(debounce)
    debounce = setTimeout(async () => {
      try {
        await buildOnce(config, ctx, adapter)
      } catch (err) {
        console.error('[sui-build] build error:', err)
      }
    }, 100)
  })
}

async function main() {
  const args = parseArgs(process.argv.slice(2))
  const config = await loadConfig(args.config)
  config.mode ??= args.mode

  const packageRoot = dirname(args.config)
  const outdir = resolve(packageRoot, config.outdir)
  mkdirSync(outdir, { recursive: true })

  const ctx: AdapterContext = { packageRoot, outdir }
  const adapter = pickAdapter(args.bundler)

  if (args.watch) {
    config.mode = 'development'
    await watchMode(config, ctx, adapter)
  } else {
    await buildOnce(config, ctx, adapter)
  }
}

main().catch((err) => {
  console.error('[sui-build] fatal:', err)
  process.exit(1)
})
