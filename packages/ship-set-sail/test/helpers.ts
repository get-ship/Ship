// Shared compile + load helper for the sail ports.

import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { join, resolve } from 'node:path'

const REPO_ROOT = resolve(__dirname, '../../../')
const SAIL_BIN = resolve(REPO_ROOT, '../ship-cache/cargo-target/debug/sail')
const COMPONENTS = resolve(__dirname, '../components')
const COMPILE_OUT = resolve(__dirname, '.compiled')

function ensureBinary() {
  if (existsSync(SAIL_BIN)) return
  execFileSync('cargo', ['build', '--bin', 'sail', '--quiet'], {
    cwd: REPO_ROOT,
    stdio: 'inherit',
  })
}

export async function loadSailComponent(file: string): Promise<string> {
  ensureBinary()
  mkdirSync(COMPILE_OUT, { recursive: true })
  execFileSync(SAIL_BIN, ['build', join(COMPONENTS, file), '--out', COMPILE_OUT], {
    stdio: 'pipe',
  })
  const stem = file.replace(/\.sail$/, '')
  // Tag derives via tag_for: stems with `-` keep their name verbatim.
  const tag = stem
  const jsPath = join(COMPILE_OUT, `${tag}.js`)
  const js = readFileSync(jsPath, 'utf8')
  const mjsPath = join(COMPILE_OUT, `${tag}.mjs`)
  writeFileSync(mjsPath, js)
  await import(`${mjsPath}?t=${Date.now()}`)
  return tag
}
