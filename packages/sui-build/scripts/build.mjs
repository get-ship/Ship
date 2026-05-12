#!/usr/bin/env node
import { build } from 'esbuild'
import { writeFileSync, chmodSync, mkdirSync } from 'node:fs'

mkdirSync('dist', { recursive: true })

const result = await build({
  entryPoints: ['src/cli.ts'],
  bundle: true,
  platform: 'node',
  format: 'esm',
  external: ['esbuild', 'chokidar', 'bun:ffi'],
  write: false,
})

const shebang = '#!/usr/bin/env node\n'
writeFileSync('dist/cli.js', shebang + result.outputFiles[0].text)
chmodSync('dist/cli.js', 0o755)
console.log('built dist/cli.js')
