// sui-catalog CLI — emits `.ship/sui-out/catalogs/{workspace,published}.json`
// for the current repo.
//
// Usage:
//   sui-catalog                    # run from repo root (default)
//   sui-catalog --root <dir>       # explicit repo root
//   sui-catalog --out <dir>        # output dir (default: <root>/.ship/sui-out/catalogs)
//
// No watch mode — this is a one-shot post-build step that the bundler
// invokes after every full sui-build pass.

import { mkdir, writeFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { buildCatalogs, stringifyCatalog } from './aggregator.js'

interface CliArgs {
  root: string
  out: string
  quiet: boolean
}

function parseArgs(argv: string[]): CliArgs {
  let root = process.cwd()
  let outOverride: string | null = null
  let quiet = false
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === '--root') root = resolve(argv[++i] ?? '.')
    else if (a === '--out') outOverride = resolve(argv[++i] ?? '.')
    else if (a === '--quiet' || a === '-q') quiet = true
    else if (a === '--help' || a === '-h') {
      process.stdout.write(
        'sui-catalog — emit .ship/sui-out/catalogs/{workspace,published}.json\n' +
          '\n  --root <dir>   repo root (default: cwd)\n' +
          '  --out  <dir>   output directory (default: <root>/.ship/sui-out/catalogs)\n' +
          '  --quiet        suppress progress output\n',
      )
      process.exit(0)
    }
  }
  const out = outOverride ?? resolve(root, '.ship/sui-out/catalogs')
  return { root, out, quiet }
}

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2))
  const t0 = Date.now()
  const { workspace, published } = await buildCatalogs({ repoRoot: args.root })
  await mkdir(args.out, { recursive: true })
  const wsPath = resolve(args.out, 'workspace.json')
  const pubPath = resolve(args.out, 'published.json')
  await writeFile(wsPath, stringifyCatalog(workspace), 'utf8')
  await writeFile(pubPath, stringifyCatalog(published), 'utf8')
  if (!args.quiet) {
    const dt = Date.now() - t0
    process.stdout.write(
      `sui-catalog: workspace=${workspace.count} published=${published.count} (${dt} ms)\n` +
        `  ${wsPath}\n  ${pubPath}\n`,
    )
  }
}

main().catch((err) => {
  process.stderr.write(`sui-catalog: ${err instanceof Error ? err.message : String(err)}\n`)
  process.exit(1)
})
