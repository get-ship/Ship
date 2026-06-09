#!/usr/bin/env node
// View smoke: read tests/e2e/visual/views.json and confirm every baselined
// view returns HTTP 200 for both the SSR route (/sui/<name>) and the
// compiled module route (/_deck/views/<name>). Exit 1 on any failure.
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(__dirname, '..', '..')

const args = process.argv.slice(2)
let port = ''
for (let i = 0; i < args.length; i++) {
    if (args[i] === '--port' && args[i + 1]) {
        port = args[i + 1]
        i++
    }
}
if (!port) {
    console.error('usage: smoke-views.mjs --port <port>')
    process.exit(2)
}

const base = `http://localhost:${port}`
const viewsJson = JSON.parse(
    fs.readFileSync(path.join(root, 'tests/e2e/visual/views.json'), 'utf8'),
)
const views = viewsJson.views ?? []
if (views.length === 0) {
    console.error('views.json has no views — nothing to smoke')
    process.exit(2)
}

const RED = '\x1b[31m'
const GREEN = '\x1b[32m'
const RESET = '\x1b[0m'

let failed = 0
for (const name of views) {
    for (const route of [`/sui/${name}`, `/_deck/views/${name}`]) {
        const url = `${base}${route}`
        let status = 0
        let err = ''
        try {
            const res = await fetch(url, { redirect: 'manual' })
            status = res.status
        } catch (e) {
            err = e instanceof Error ? e.message : String(e)
        }
        if (status === 200) {
            console.log(`${GREEN}✓${RESET} ${route} 200`)
        } else {
            failed++
            const detail = err ? `(${err})` : status
            console.log(`${RED}✗${RESET} ${route} ${detail}`)
        }
    }
}

if (failed > 0) {
    console.error(`\n${failed} view check(s) failed`)
    process.exit(1)
}
