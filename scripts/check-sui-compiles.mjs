#!/usr/bin/env node
// SUI production-compile gate. Runs the REAL native compiler
// (`cargo run -q -p sui-compiler -- build <file>`) over EVERY first-party
// `.sui` source and FAILS (exit 1) listing every file the compiler rejects.
//
// This is the production gate — the same `check_all` pass the daemon bundler
// runs, with honest exit codes (#255): a hard parse/type error OR any
// Error-severity lint finding (css-literal, prop-value, attr-bool-prop,
// controller-purity, …) makes `sui-compiler build` exit non-zero. jsdom unit
// tests do NOT exercise this path, so a `.sui` can be green in vitest and still
// fail the real compiler (#263). This gate closes that hole.
//
// Scope: first-party sources only —
//   packages/components/**, views/**, .ship/views/**
// Excludes: node_modules, dist, .deck, sui-out, .artifacts, tests, fixtures.
//
// Usage:  node scripts/check-sui-compiles.mjs
// Exit 0  = every first-party .sui compiles clean.
// Exit 1  = at least one file failed (each printed with the compiler's output).
// Exit 2  = no first-party .sui files found (scope drift — fail loud).

import { spawnSync } from 'node:child_process';
import { mkdtempSync, readdirSync, rmSync, statSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, basename, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), '..');

// Roots to walk for first-party `.sui` sources. `apps/shell/src` carries the
// shell's own chrome components + studio view sources; without it a broken
// shell `.sui` (e.g. a malformed `{#while-error}`) only surfaces at app-build
// time, never in this gate — exactly how sui-account-menu shipped a real
// compile error past this gate.
const ROOTS = ['packages/components', 'views', '.ship/views', 'apps/shell/src'];

// Directory names that never hold authored first-party sources: build output,
// generated mirrors, vendored deps, scratch, and test fixtures.
const EXCLUDE_DIRS = new Set([
    'node_modules',
    'dist',
    '.deck',
    'sui-out',
    '.artifacts',
    'tests',
    'fixtures',
]);

function isSuiSource(name) {
    return name.endsWith('.sui') || name.endsWith('.html.set');
}

function walk(dir, acc) {
    let entries;
    try {
        entries = readdirSync(dir, { withFileTypes: true });
    } catch {
        return; // root absent in this worktree — skip silently.
    }
    for (const entry of entries) {
        if (entry.isDirectory()) {
            if (EXCLUDE_DIRS.has(entry.name)) continue;
            walk(join(dir, entry.name), acc);
        } else if (entry.isFile() && isSuiSource(entry.name)) {
            acc.push(join(dir, entry.name));
        }
    }
}

// `<tag>/<tag>.sui` is the component convention — the parent dir name IS the
// tag. The compiler refuses to compile such a path WITHOUT --component-name
// (it would otherwise silently emit view-mode output that registers no custom
// element, #176). Mirror that detection here so the gate passes the right
// flag instead of tripping the guardrail on every component.
function componentTag(file) {
    const name = basename(file);
    const stem = name.endsWith('.sui')
        ? name.slice(0, -'.sui'.length)
        : name.endsWith('.html.set')
          ? name.slice(0, -'.html.set'.length)
          : name;
    const parent = basename(dirname(file));
    return stem === parent ? stem : null;
}

const files = [];
for (const root of ROOTS) walk(join(repoRoot, root), files);
files.sort();

if (files.length === 0) {
    console.error(
        'check-sui-compiles: no first-party .sui files found under ' +
            ROOTS.join(', ') +
            ' — scope drift, refusing to pass vacuously.',
    );
    process.exit(2);
}

// Emit sidecar artifacts into a throwaway dir so the gate never litters the
// tree (.ship/sui-out is the default and would pollute a clean checkout).
const outDir = mkdtempSync(join(tmpdir(), 'sui-compile-gate-'));
const failures = [];

console.log(`check-sui-compiles: ${files.length} first-party .sui file(s)\n`);

for (const file of files) {
    const rel = file.startsWith(repoRoot + sep)
        ? file.slice(repoRoot.length + 1)
        : file;
    const tag = componentTag(file);
    const args = ['run', '-q', '-p', 'sui-compiler', '--', 'build', file, '--out', outDir];
    if (tag) args.push('--component-name', tag);
    const res = spawnSync('cargo', args, { cwd: repoRoot, encoding: 'utf8' });
    if (res.status === 0) {
        console.log(`  ok    ${rel}`);
    } else {
        const out = `${res.stdout ?? ''}${res.stderr ?? ''}`.trim();
        console.log(`  FAIL  ${rel}`);
        failures.push({ rel, out });
    }
}

rmSync(outDir, { recursive: true, force: true });

if (failures.length > 0) {
    console.error(`\ncheck-sui-compiles: ${failures.length} of ${files.length} file(s) FAILED to compile:\n`);
    for (const { rel, out } of failures) {
        console.error(`── ${rel} ──`);
        console.error(out || '(no compiler output)');
        console.error('');
    }
    process.exit(1);
}

console.log(`\ncheck-sui-compiles: all ${files.length} first-party .sui file(s) compile clean.`);
