// Bundle Kysely + a Kysely query script into one self-contained ESM that:
//  - runs in any JS environment with no module loader (suitable for rquickjs)
//  - sets globalThis.__queryAst = JSON.stringify(...toOperationNode())
//
// Run: bun bundle.mjs <query-script-path> > out.bundle.js
// Or with default test query: bun bundle.mjs > out.bundle.js
import { build } from 'bun'
import { writeFileSync, readFileSync } from 'node:fs'
import { resolve, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const queryPath = process.argv[2] || resolve(__dirname, 'query-default.mjs')

const entry = resolve(__dirname, '__entry.mjs')

// Write a small entry that imports the query, then publishes its AST globally.
writeFileSync(
  entry,
  `import { query } from ${JSON.stringify(queryPath)};
globalThis.__queryAst = JSON.stringify(query.toOperationNode());
`,
)

const result = await build({
  entrypoints: [entry],
  format: 'esm',
  target: 'browser',
  // Inline everything; produce one self-contained string.
  minify: false,
  splitting: false,
  external: [],
})

if (!result.success) {
  console.error('bundle failed:', result.logs)
  process.exit(1)
}

const bundle = await result.outputs[0].text()
process.stdout.write(bundle)
