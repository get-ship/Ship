#!/usr/bin/env node
// Zero-collection guard. A `*.test.ts` file that collects ZERO tests passes
// vacuously — vitest reports the suite green while the file proves nothing. The
// honesty guard (`expect.hasAssertions()`) catches an empty test BODY; this
// catches an empty test FILE (no `it`/`test` at all, or every case commented
// out). Together with `passWithNoTests:false` in the shared vitest defaults
// (which fails the whole run if it collects nothing), no empty test can hide.
//
// Static check: every first-party `*.test.ts` must declare at least one test
// case — `it(...)`, `test(...)`, or their modifier forms (`it.skipIf`,
// `it.each`, `test.only`, …). Comments and strings are stripped first so a
// `// it('todo')` line cannot satisfy the guard.
//
// Scope: first-party test sources — packages/**, apps/**, tests/**.
// Excludes: node_modules, dist, .deck, sui-out, .artifacts, fixtures.
//
// Usage:  node scripts/check-zero-collection.mjs
// Exit 0  = every test file declares at least one case.
// Exit 1  = at least one file collects zero tests (each printed).
// Exit 2  = no test files found (scope drift — fail loud).

import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const ROOTS = ['packages', 'apps', 'tests'];
const EXCLUDE_DIRS = new Set([
    'node_modules',
    'dist',
    '.deck',
    'sui-out',
    '.artifacts',
    'fixtures',
]);

function walk(dir, acc) {
    let entries;
    try {
        entries = readdirSync(dir, { withFileTypes: true });
    } catch {
        return;
    }
    for (const entry of entries) {
        if (entry.isDirectory()) {
            if (EXCLUDE_DIRS.has(entry.name)) continue;
            walk(join(dir, entry.name), acc);
        } else if (entry.isFile() && entry.name.endsWith('.test.ts')) {
            acc.push(join(dir, entry.name));
        }
    }
}

// Strip line comments, block comments, and string/template literals so a
// `// it(...)` or `"it("` inside a string can't masquerade as a real test.
function stripCommentsAndStrings(src) {
    let out = '';
    let i = 0;
    const n = src.length;
    while (i < n) {
        const c = src[i];
        const next = src[i + 1];
        if (c === '/' && next === '/') {
            while (i < n && src[i] !== '\n') i++;
        } else if (c === '/' && next === '*') {
            i += 2;
            while (i < n && !(src[i] === '*' && src[i + 1] === '/')) i++;
            i += 2;
        } else if (c === '"' || c === "'" || c === '`') {
            const quote = c;
            i++;
            while (i < n && src[i] !== quote) {
                if (src[i] === '\\') i++;
                i++;
            }
            i++;
        } else {
            out += c;
            i++;
        }
    }
    return out;
}

// `it`/`test` as a call or with a chained modifier: it(, test(, it.skip(,
// it.skipIf(...)(, it.each([...])(, test.only(, describe-nested forms included.
const TEST_DECL = /\b(?:it|test)\s*(?:\.\s*[A-Za-z]+\s*(?:\([^)]*\))?)*\s*\(/;

const files = [];
for (const root of ROOTS) walk(join(repoRoot, root), files);
files.sort();

if (files.length === 0) {
    console.error(
        'check-zero-collection: no *.test.ts files found under ' +
            ROOTS.join(', ') +
            ' — scope drift, refusing to pass vacuously.',
    );
    process.exit(2);
}

const empty = [];
for (const file of files) {
    const code = stripCommentsAndStrings(readFileSync(file, 'utf8'));
    if (!TEST_DECL.test(code)) empty.push(file);
}

if (empty.length > 0) {
    console.error(
        `check-zero-collection: ${empty.length} of ${files.length} test file(s) collect ZERO tests:\n`,
    );
    for (const file of empty) {
        const rel = file.startsWith(repoRoot + sep) ? file.slice(repoRoot.length + 1) : file;
        console.error(`  ${rel}`);
    }
    console.error('\nA test file with no it()/test() proves nothing. Add a real case or delete the file.');
    process.exit(1);
}

console.log(`check-zero-collection: all ${files.length} test file(s) declare at least one case.`);
