import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { basename, join } from 'node:path'

import type { ViteFileMode } from './types.ts'

export interface CompileResult {
  /** Emitted JS module text. */
  code: string
  /** Optional source map. The current sui-compiler doesn't emit position
   *  info per AST node, so we synthesize a low-resolution map that just
   *  points DevTools back at the .sui source file (no line precision).
   *  Once the compiler grows real source-position emit, this gets replaced. */
  map: SourceMap
}

export interface SourceMap {
  version: 3
  sources: string[]
  sourcesContent: string[]
  names: string[]
  mappings: string
}

export interface CompileArgs {
  compilerPath: string
  filePath: string
  source: string
  mode: ViteFileMode
  noDts: boolean
}

/**
 * Compile a single `.sui` source by shelling to `sui-compiler build`.
 *
 * Subprocess spawn uses `Bun.spawn`; emitted JS is read via `Bun.file`.
 * Both are first-class on the runtime we ship in `~/.ship/runtime/bun`.
 *
 * The compiler reads from disk (not stdin) so we route the file path
 * through directly rather than streaming the content. Output lands in a
 * per-call temp dir that we clean up after reading the JS. Vite caches
 * transform results keyed on file content, so this fires once per import
 * and again on edits.
 */
export async function compileSui(args: CompileArgs): Promise<CompileResult> {
  const out = await mkdtemp(join(tmpdir(), 'sui-vite-'))
  try {
    const cliArgs = ['build', args.filePath, '--out', out]
    if (args.noDts) cliArgs.push('--no-dts')
    if (args.mode.kind === 'component') {
      cliArgs.push('--component-name', args.mode.meta.name)
    }
    await runChild(args.compilerPath, cliArgs)
    const stem = basename(args.filePath).replace(/\.sui$/, '')
    const jsPath = join(out, `${stem}.js`)
    const code = await Bun.file(jsPath).text()
    return { code, map: stubSourceMap(args.filePath, args.source) }
  } finally {
    await rm(out, { recursive: true, force: true })
  }
}

async function runChild(bin: string, args: string[]): Promise<void> {
  const proc = Bun.spawn({
    cmd: [bin, ...args],
    stdout: 'pipe',
    stderr: 'pipe',
    stdin: 'ignore',
  })
  const [stderr, exitCode] = await Promise.all([
    new Response(proc.stderr).text(),
    proc.exited,
  ])
  if (exitCode !== 0) {
    const target = args[args.indexOf('build') + 1] ?? '?'
    throw new Error(
      `sui-compiler exit ${exitCode} for ${target}\n${stderr.trim()}`,
    )
  }
}

/**
 * Low-resolution source map: every line of emitted JS maps to the start
 * of the .sui file. Lets DevTools jump from a stack frame back to the
 * source file even though we can't yet track exact lines. The compiler
 * will eventually emit real source-position info; when it does, this
 * function gets replaced wholesale.
 */
function stubSourceMap(filePath: string, source: string): SourceMap {
  return {
    version: 3,
    sources: [filePath],
    sourcesContent: [source],
    names: [],
    mappings: 'AAAA',
  }
}
