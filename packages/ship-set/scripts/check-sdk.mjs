#!/usr/bin/env node
/**
 * Sanity-check the generated Ship SDK manifest. Verifies:
 *   - All six expected JSON files exist under packages/ship-set/sdk/
 *   - Each has `_generated`, `version`, `$schema`, and a non-null `entries`
 *   - `entries` is an object (or array for url maps)
 *
 * Exits non-zero with a clear message if anything is missing. Intended to run
 * in CI after `just gen-sdk` (or locally to confirm a fresh regenerate).
 */
import { readFileSync, existsSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const sdkDir = resolve(here, '..', 'sdk');

const expected = [
  'sdk-macros.json',
  'sdk-urls.json',
  'sdk-view-contexts.json',
  'sdk-collections.json',
  'sdk-triggers.json',
  'sdk-views.json',
];

let failed = false;
for (const name of expected) {
  const path = resolve(sdkDir, name);
  if (!existsSync(path)) {
    console.error(`[check-sdk] MISSING: ${path}`);
    failed = true;
    continue;
  }
  let doc;
  try {
    doc = JSON.parse(readFileSync(path, 'utf8'));
  } catch (e) {
    console.error(`[check-sdk] INVALID JSON: ${name}: ${e.message}`);
    failed = true;
    continue;
  }
  for (const key of ['_generated', 'version', '$schema', 'entries']) {
    if (!(key in doc)) {
      console.error(`[check-sdk] ${name}: missing top-level key "${key}"`);
      failed = true;
    }
  }
  if (doc.entries == null || typeof doc.entries !== 'object') {
    console.error(`[check-sdk] ${name}: "entries" must be a non-null object`);
    failed = true;
  }
  const count = Array.isArray(doc.entries)
    ? doc.entries.length
    : Object.keys(doc.entries || {}).length;
  console.log(`[check-sdk] OK: ${name} (${count} entries)`);
}

if (failed) {
  console.error('[check-sdk] FAILED');
  process.exit(1);
}
console.log('[check-sdk] all six files pass');
