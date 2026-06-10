#!/usr/bin/env node
// License audit gate. Walks the RESOLVED dependency tree (the on-disk
// `node_modules/.bun/` isolated store Bun produces) and reads each package's
// own `package.json` `license`/`licenses` field — the source of truth for what
// actually got installed. EXITS NON-ZERO if any resolved license is copyleft
// (a deny-list: GPL / AGPL / LGPL / EPL / weak-MPL families).
//
// This replaces the old practice of hand-copying a vendor LICENSE file next to
// committed vendored bytes. We no longer commit vendored source — third-party
// engines are declared deps and inlined at build by the bundler — so license
// control moves here, run in CI (see .github/workflows/ci.yml).
//
// Usage:  node scripts/check-licenses.mjs        (or: just license-check)
// Exit 0  = all resolved licenses permissive.
// Exit 1  = at least one copyleft license found (printed with the offending pkg).
// Exit 2  = the resolved store is missing (run `bun install` first).

import { readFileSync, readdirSync, existsSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
const store = join(repoRoot, 'node_modules', '.bun');

// Copyleft / reciprocal deny-list. SPDX identifier *fragments* matched
// case-insensitively against the declared license string. Permissive families
// (MIT, ISC, BSD-*, Apache-2.0, Unlicense, 0BSD, CC0, BlueOak, Python, Zlib)
// are NOT on this list and pass.
const DENY = [
  'GPL',       // GPL-2.0, GPL-3.0 (also catches AGPL/LGPL substrings, kept explicit below)
  'AGPL',
  'LGPL',
  'EPL',       // Eclipse Public License
  'EUPL',      // European Union Public License (copyleft)
  'MPL',       // Mozilla Public License — weak copyleft, denied per task spec
  'CDDL',      // Common Development and Distribution License (weak copyleft)
  'CECILL',    // French copyleft
  'OSL',       // Open Software License (copyleft)
  'SLEEPYCAT', // Berkeley DB copyleft
];

// SPDX exception: a permissive license carrying a "GPL-compatible" note or a
// dual-license that INCLUDES a permissive option. We only treat a string as
// copyleft if a denied token appears AND no permissive alternative is offered
// via an SPDX `OR`. e.g. "(MPL-2.0 OR Apache-2.0)" is acceptable (take Apache).
const PERMISSIVE_HINT =
  /\b(MIT|ISC|BSD(-[0-9A-Za-z-]+)?|Apache-2\.0|Unlicense|0BSD|CC0-1\.0|BlueOak-1\.0\.0|Python-2\.0|Zlib|WTFPL)\b/i;

// Pre-existing accepted copyleft exceptions, keyed by bare package name (any
// version). Each is a build-time-only tool whose copyleft does NOT reach the
// shipped product (MPL/LGPL are file-level/dynamic-link copyleft; we neither
// modify their source nor statically link them into a distributed binary).
// New copyleft deps are still denied — add here only with a written reason.
const ALLOW = {
  // CSS minifier (build tool). MPL-2.0 is file-level copyleft; we consume the
  // compiled binary unmodified. Already in package.json trustedDependencies.
  'lightningcss': 'build tool (CSS minify); MPL-2.0 file-level, consumed unmodified',
  'lightningcss-linux-x64-gnu': 'lightningcss native binary (build tool)',
  'lightningcss-linux-x64-musl': 'lightningcss native binary (build tool)',
  // Accessibility test engine, devDependency only — never shipped to users.
  'axe-core': 'devDependency (a11y test engine); MPL-2.0, test-time only',
  // sharp's libvips native binary (image build tooling). LGPL-3.0 is dynamic-
  // link copyleft; consumed as an unmodified prebuilt. In trustedDependencies.
  '@img/sharp-libvips-linux-x64': 'sharp/libvips prebuilt (image build tool); LGPL dynamic-link, unmodified',
  '@img/sharp-libvips-linuxmusl-x64': 'sharp/libvips prebuilt (image build tool); LGPL dynamic-link, unmodified',
};

function readLicense(pkgJsonPath) {
  try {
    const pkg = JSON.parse(readFileSync(pkgJsonPath, 'utf8'));
    if (typeof pkg.license === 'string') return pkg.license;
    if (pkg.license && typeof pkg.license === 'object' && pkg.license.type) {
      return pkg.license.type;
    }
    if (Array.isArray(pkg.licenses)) {
      return pkg.licenses.map((l) => (typeof l === 'string' ? l : l.type)).join(' OR ');
    }
    return null; // unknown — reported, not denied
  } catch {
    return null;
  }
}

function isDenied(license) {
  if (!license) return false;
  const upper = license.toUpperCase();
  const hitsDeny = DENY.some((tok) => upper.includes(tok));
  if (!hitsDeny) return false;
  // Dual-license escape hatch: "(MPL-2.0 OR MIT)" — a permissive option exists.
  if (/\bOR\b/i.test(license) && PERMISSIVE_HINT.test(license)) return false;
  return true;
}

// Each `.bun/<pkg>@<ver>` dir nests the package under `node_modules/<name>`.
// Collect every package.json one level into that nested node_modules.
function collectPackageJsons(storeDir) {
  const found = [];
  if (!existsSync(storeDir)) return found;
  for (const entry of readdirSync(storeDir)) {
    if (entry === 'node_modules') continue;
    const nm = join(storeDir, entry, 'node_modules');
    if (!existsSync(nm)) continue;
    for (const name of readdirSync(nm)) {
      const dir = join(nm, name);
      if (name.startsWith('@')) {
        // scoped: one more level
        if (!statSafe(dir)) continue;
        for (const sub of readdirSync(dir)) {
          const pj = join(dir, sub, 'package.json');
          if (existsSync(pj)) found.push(pj);
        }
      } else {
        const pj = join(dir, 'package.json');
        if (existsSync(pj)) found.push(pj);
      }
    }
  }
  return found;
}

function statSafe(p) {
  try {
    return statSync(p).isDirectory();
  } catch {
    return false;
  }
}

if (!existsSync(store)) {
  console.error(
    `[check-licenses] resolved store not found at ${store}\n` +
      `  Run \`bun install\` first — this gate audits the resolved tree.`,
  );
  process.exit(2);
}

const pkgJsons = collectPackageJsons(store);
const violations = [];
const allowed = [];
const unknown = [];
const seen = new Set();

for (const pj of pkgJsons) {
  let name, version;
  try {
    const pkg = JSON.parse(readFileSync(pj, 'utf8'));
    name = pkg.name;
    version = pkg.version;
  } catch {
    continue;
  }
  const id = `${name}@${version}`;
  if (seen.has(id)) continue;
  seen.add(id);
  const license = readLicense(pj);
  if (isDenied(license)) {
    if (Object.prototype.hasOwnProperty.call(ALLOW, name)) {
      allowed.push({ id, license, reason: ALLOW[name] });
    } else {
      violations.push({ id, license });
    }
  } else if (!license) {
    unknown.push(id);
  }
}

console.log(`[check-licenses] audited ${seen.size} resolved packages from ${store.replace(repoRoot + '/', '')}`);

if (allowed.length) {
  console.log(`[check-licenses] ${allowed.length} pre-approved copyleft exception(s) (build-time-only, see ALLOW):`);
  for (const a of allowed) console.log(`    ~ ${a.id}  →  ${a.license}  (${a.reason})`);
}

if (unknown.length) {
  console.warn(
    `[check-licenses] ${unknown.length} package(s) declare no license field (reported, not denied):`,
  );
  for (const id of unknown.slice(0, 20)) console.warn(`    - ${id}`);
  if (unknown.length > 20) console.warn(`    ... and ${unknown.length - 20} more`);
}

if (violations.length) {
  console.error(`\n[check-licenses] FAIL — ${violations.length} copyleft license(s) found:`);
  for (const v of violations) console.error(`    ✗ ${v.id}  →  ${v.license}`);
  console.error(
    `\nCopyleft deny-list: ${DENY.join(', ')}. ` +
      `Remove or replace the offending dependency, or (if dual-licensed with a ` +
      `permissive option) confirm the SPDX 'OR' expression is correct.`,
  );
  process.exit(1);
}

console.log('[check-licenses] PASS — all resolved licenses are permissive.');
process.exit(0);
