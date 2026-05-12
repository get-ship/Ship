import { execFile } from 'node:child_process'
import { readdirSync, statSync, existsSync } from 'node:fs'
import { readFile, writeFile } from 'node:fs/promises'
import { basename, dirname, join, resolve } from 'node:path'
import type { SuiBuildConfig, SuiBundler, BuildResult, AdapterContext } from './config.js'
import { makeFfiSpawn } from './adapters/bun.js'

/** Injected spawn function — lets tests avoid needing the real binary. */
export type CompilerSpawn = (suiPath: string, stem: string) => Promise<void>

type MtimeCache = Record<string, number>

function findWorkspaceRoot(startDir: string): string {
  let dir = startDir
  while (true) {
    if (existsSync(join(dir, 'Cargo.toml'))) return dir
    const parent = dirname(dir)
    if (parent === dir) return startDir
    dir = parent
  }
}

function resolveCompilerBin(packageRoot: string): string {
  const envPath = process.env['SUI_COMPILER_PATH']
  if (envPath) return envPath
  return join(findWorkspaceRoot(packageRoot), 'target', 'debug', 'sui-compiler')
}

function resolveCompilerLib(packageRoot: string): string {
  const envPath = process.env['SUI_COMPILER_LIB_PATH']
  if (envPath) return envPath
  const wsRoot = findWorkspaceRoot(packageRoot)
  const ext = process.platform === 'darwin' ? 'dylib' : process.platform === 'win32' ? 'dll' : 'so'
  const prefix = process.platform === 'win32' ? '' : 'lib'
  return join(wsRoot, 'target', 'debug', `${prefix}sui_compiler.${ext}`)
}

function* walkSui(dir: string): Generator<string> {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, entry.name)
    if (entry.isDirectory()) yield* walkSui(p)
    else if (entry.isFile() && entry.name.endsWith('.sui')) yield p
  }
}

function collectSuiFiles(config: SuiBuildConfig, ctx: AdapterContext): string[] {
  const dirs = new Set<string>()
  for (const src of Object.values({ ...config.entries, ...(config.lazyBundles ?? {}) })) {
    dirs.add(dirname(resolve(ctx.packageRoot, src)))
  }
  const files: string[] = []
  for (const dir of dirs) {
    if (existsSync(dir)) files.push(...walkSui(dir))
  }
  return [...new Set(files)]
}

async function loadCache(cacheFile: string): Promise<MtimeCache> {
  try {
    return JSON.parse(await readFile(cacheFile, 'utf8')) as MtimeCache
  } catch {
    return {}
  }
}

function makeProcessSpawn(compilerBin: string): CompilerSpawn {
  return (suiPath, stem) =>
    new Promise<void>((res, rej) => {
      const hasBin = existsSync(compilerBin)
      const cmd = hasBin ? compilerBin : 'cargo'
      const args = hasBin
        ? ['build', suiPath, '--component-name', stem, '--no-dts']
        : ['run', '--quiet', '-p', 'sui-compiler', '--', 'build', suiPath, '--component-name', stem, '--no-dts']
      execFile(cmd, args, (err, _out, stderr) => {
        if (err) rej(new Error(`sui-compiler failed for ${suiPath}: ${stderr || err.message}`))
        else res()
      })
    })
}

async function makeDefaultSpawn(compilerBin: string, libPath: string): Promise<CompilerSpawn> {
  const ffi = await makeFfiSpawn(libPath)
  return ffi ?? makeProcessSpawn(compilerBin)
}

export async function run(
  config: SuiBuildConfig,
  ctx: AdapterContext,
  adapter: SuiBundler,
  spawn?: CompilerSpawn,
): Promise<BuildResult> {
  const t0 = Date.now()
  const cacheFile = join(ctx.packageRoot, '.sui-build-cache.json')
  const cache = await loadCache(cacheFile)
  const doSpawn = spawn ?? await makeDefaultSpawn(
    resolveCompilerBin(ctx.packageRoot),
    resolveCompilerLib(ctx.packageRoot),
  )

  const suiFiles = collectSuiFiles(config, ctx)
  const warnings: string[] = []

  for (const suiPath of suiFiles) {
    let mtime: number
    try {
      mtime = statSync(suiPath).mtimeMs
    } catch {
      warnings.push(`skipped missing file: ${suiPath}`)
      continue
    }
    if (cache[suiPath] === mtime) continue
    const stem = basename(suiPath, '.sui')
    await doSpawn(suiPath, stem)
    cache[suiPath] = mtime
  }

  await writeFile(cacheFile, JSON.stringify(cache), 'utf8')

  const result = await adapter.build(config, ctx)
  return {
    outputs: result.outputs,
    warnings: [...warnings, ...result.warnings],
    durationMs: Date.now() - t0,
  }
}

// Re-export for consumers who want to scan source dirs (e.g. the CLI watch loop).
export { collectSuiFiles }
