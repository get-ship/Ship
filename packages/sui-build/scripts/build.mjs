#!/usr/bin/env node
import { build } from 'esbuild'
import { writeFileSync, chmodSync, mkdirSync } from 'node:fs'

mkdirSync('dist', { recursive: true })

const result = await build({
  entryPoints: ['src/cli.ts'],
  bundle: true,
  platform: 'node',
  format: 'esm',
  external: ['esbuild', 'chokidar'],
  write: false,
})

const shebang = '#!/usr/bin/env bun\n'
writeFileSync('dist/cli.js', shebang + result.outputFiles[0].text)
chmodSync('dist/cli.js', 0o755)
console.log('built dist/cli.js')
