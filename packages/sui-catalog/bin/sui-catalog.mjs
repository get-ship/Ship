#!/usr/bin/env bun
// Tiny shim — invokes src/cli.ts. Mirrors @ship/sui-build's bin shape so
// `bun run --filter @ship/sui-catalog ...` and shell-wrapper invocations
// both work without rebuilding.
import { spawn } from 'node:child_process'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const cli = resolve(here, '..', 'src', 'cli.ts')

if (typeof Bun !== 'undefined') {
  await import(cli)
} else {
  const child = spawn('bun', [cli, ...process.argv.slice(2)], { stdio: 'inherit' })
  child.on('exit', (code, signal) => {
    if (signal) process.kill(process.pid, signal)
    else process.exit(code ?? 1)
  })
}
