#!/usr/bin/env node
/**
 * Extract macro signatures and docstrings from apps/shipd/templates/_ship.html.j2
 * and emit a machine-readable manifest (JSON) + a grouped human reference (Markdown).
 *
 * Output:
 *   - packages/ship-set/jinja-macros.json (pretty-printed, stable source order)
 *   - packages/ship-set/jinja-macros.md  (TOC + grouped sections)
 *
 * Idempotent: re-running on the same source produces byte-identical output.
 *
 * Usage:
 *   node scripts/build-jinja-docs.mjs
 */

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

import { extractMacros, buildManifest } from './jinja-docs/parse.mjs';
import { renderMarkdown } from './jinja-docs/render.mjs';

const repoRoot = resolve(fileURLToPath(import.meta.url), '..', '..');
const sources = [
  'apps/shipd/templates/_ship.html.j2',
];
const jsonOut = 'packages/ship-set/jinja-macros.json';
const mdOut = 'packages/ship-set/jinja-macros.md';

// Category grouping — edit here to move a macro into a different section.
// Any macro not listed lands in a trailing `Other` section.
const categories = {
  'Page setup': ['head', 'styles'],
  'Signals': ['signal', 'show', 'attr', 'bind', 'toggle_class'],
  'Events': ['on', 'live', 'live_persistent', 'on_fetch_error'],
  'Actions': ['get', 'post', 'put', 'delete', 'navigate', 'nav', 'view', 'submit', 'put_on_change'],
  'Collection actions': ['create', 'update', 'confirm_delete'],
  'Job actions': ['create_job', 'emit', 'feedback'],
  'UI patterns': [
    'section', 'endsection', 'tab', 'tab_panel', 'chip', 'chip_all',
    'search', 'on_change_debounced', 'radio', 'confirm', 'loading', 'spinner',
    'toast_container', 'toast_action', 'confirm_dialog', 'confirm_trigger',
  ],
  'Auto-generators': ['form', 'table'],
  'Layout': ['layout_center', 'layout_sidebar', 'layout_three_col', 'layout_end'],
};

function main() {
  const allMacros = [];
  for (const rel of sources) {
    const abs = resolve(repoRoot, rel);
    const text = readFileSync(abs, 'utf8');
    allMacros.push(...extractMacros(text, rel));
  }

  const entries = buildManifest(allMacros);
  const undocumented = entries.filter((e) => !e.documented);

  const jsonPath = resolve(repoRoot, jsonOut);
  const mdPath = resolve(repoRoot, mdOut);
  mkdirSync(dirname(jsonPath), { recursive: true });
  mkdirSync(dirname(mdPath), { recursive: true });
  writeFileSync(jsonPath, JSON.stringify(entries, null, 2) + '\n');
  writeFileSync(mdPath, renderMarkdown(entries, categories));

  const relJson = relative(repoRoot, jsonPath);
  const relMd = relative(repoRoot, mdPath);
  console.log(`[jinja-docs] wrote ${relJson} (${entries.length} macros)`);
  console.log(`[jinja-docs] wrote ${relMd}`);
  if (undocumented.length > 0) {
    console.log(`[jinja-docs] WARNING: ${undocumented.length} undocumented macros:`);
    for (const e of undocumented) console.log(`  - ${e.name} (${e.source})`);
  }
}

main();
