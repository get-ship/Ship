import { dirname, resolve } from 'node:path'
import { existsSync, mkdirSync } from 'node:fs'
import { watch } from 'chokidar'
import { bunAdapter } from './adapters/bun.js'
import { run } from './runner.js'
import type { SuiBuildConfig, AdapterContext } from './config.js'

const HELP = `
sui-build — SUI bundling pipeline

Usage:
  sui-build [options]

Options:
  --config <path>        Path to sui.config.ts (default: sui.config.ts in cwd)
  --watch                Re-build on source changes
  --mode development|production  Build mode (default: production)
  --help                 Show this message
`.trim()

interface CliArgs {
  config: string
  watch: boolean
  mode: 'development' | 'production'
}

function parseArgs(argv: string[]): CliArgs {
  const args: CliArgs = {
    config: resolve(process.cwd(), 'sui.config.ts'),
    watch: false,
    mode: 'production',
  }
  const envMode = process.env['SUI_BUILD_MODE']
  if (envMode === 'development' || envMode === 'production') args.mode = envMode
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]
    if (arg === '--help' || arg === '-h') {
      console.log(HELP)
      process.exit(0)
    } else if (arg === '--config') {
      args.config = resolve(argv[++i] ?? '')
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

async function loadConfig(configPath: string): Promise<SuiBuildConfig> {
  if (!process.versions.bun) {
    console.error('sui-build requires the Bun runtime — run via `bun sui-build` or invoke bin/sui-build.mjs')
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

async function buildOnce(config: SuiBuildConfig, ctx: AdapterContext): Promise<void> {
  const result = await run(config, ctx, bunAdapter)
  for (const w of result.warnings) console.warn(`[sui-build] warn: ${w}`)
  console.log(
    `[sui-build] built ${result.outputs.length} output(s) in ${result.durationMs}ms via ${bunAdapter.name}`,
  )
  for (const o of result.outputs) console.log(`  ${o}`)
}

async function watchMode(config: SuiBuildConfig, ctx: AdapterContext): Promise<void> {
  console.log('[sui-build] watching...')
  await buildOnce(config, ctx)

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
        await buildOnce(config, ctx)
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

  if (args.watch) {
    config.mode = 'development'
    await watchMode(config, ctx)
  } else {
    await buildOnce(config, ctx)
  }
}

main().catch((err) => {
  console.error('[sui-build] fatal:', err)
  process.exit(1)
})
