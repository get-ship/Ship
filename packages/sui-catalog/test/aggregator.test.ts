// Aggregator integration test — synthesizes a tiny fixture monorepo on
// disk, runs `buildCatalogs`, and asserts on the resulting workspace +
// published documents. Covers:
//   - workspace glob expansion (`packages/*` + `packages/components/*`)
//   - published filter (private packages excluded)
//   - origin tagging for non-packaged components (view-local + shell-local)
//   - missing-CEM resilience (package with `customElements` field but no
//     manifest file on disk drops cleanly, no throw)
//   - tagName-stable sort + dedup

import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { buildCatalogs, originForNonPackagedSource } from '../src/aggregator'

let root: string

beforeEach(async () => {
  root = await mkdtemp(join(tmpdir(), 'sui-catalog-test-'))
})

afterEach(async () => {
  if (root) await rm(root, { recursive: true, force: true })
})

async function write(path: string, body: string): Promise<void> {
  await mkdir(join(root, path, '..'), { recursive: true })
  await writeFile(join(root, path), body, 'utf8')
}

function cem(tag: string, klass: string, description = ''): string {
  return JSON.stringify({
    schemaVersion: '1.0.0',
    readme: '',
    modules: [
      {
        kind: 'javascript-module',
        path: `${tag}.sui`,
        declarations: [
          {
            kind: 'class',
            name: klass,
            tagName: tag,
            customElement: true,
            description,
            attributes: [{ name: 'foo', type: { text: 'string' }, default: '' }],
            slots: [{ name: '', description: 'default slot' }],
          },
        ],
      },
    ],
  })
}

describe('buildCatalogs', () => {
  it('walks workspaces, aggregates packaged CEMs, applies published filter', async () => {
    await write(
      'package.json',
      JSON.stringify({
        name: 'fixture-root',
        private: true,
        workspaces: { packages: ['packages/*', 'packages/components/*'] },
      }),
    )
    // Public pack: `@fx/widgets` (not private + customElements field)
    await write(
      'packages/components/widgets/package.json',
      JSON.stringify({
        name: '@fx/widgets',
        customElements: 'dist/custom-elements.json',
      }),
    )
    await write(
      'packages/components/widgets/dist/custom-elements.json',
      cem('fx-button', 'FxButton', 'A button.\nSecond line.'),
    )
    await write('packages/components/widgets/dist/fx-widgets.js', '/* bundle */')
    // Private umbrella: `@fx/umbrella` (private: true + customElements field)
    // It MUST appear in workspace.json but NOT in published.json.
    await write(
      'packages/umbrella/package.json',
      JSON.stringify({
        name: '@fx/umbrella',
        private: true,
        customElements: 'dist/custom-elements.json',
      }),
    )
    await write(
      'packages/umbrella/dist/custom-elements.json',
      cem('fx-card', 'FxCard'),
    )
    await write('packages/umbrella/dist/fx-umbrella.js', '/* bundle */')
    // Package with customElements field but no manifest on disk —
    // must not throw, must not add entries.
    await write(
      'packages/components/empty/package.json',
      JSON.stringify({ name: '@fx/empty', customElements: 'dist/custom-elements.json' }),
    )

    const { workspace, published } = await buildCatalogs({ repoRoot: root })

    const wsTags = workspace.entries.map((e) => e.tagName).sort()
    expect(wsTags).toEqual(['fx-button', 'fx-card'])
    expect(workspace.count).toBe(2)

    const button = workspace.entries.find((e) => e.tagName === 'fx-button')!
    expect(button.package).toBe('@fx/widgets')
    expect(button.private).toBe(false)
    expect(button.bundleUrl).toBe('/_sui/packs/widgets/fx-widgets.js')
    expect(button.manifestUrl).toBe('/_sui/packs/widgets/custom-elements.json')
    expect(button.sourcePath).toBe(
      'packages/components/widgets/src/fx-button/fx-button.sui',
    )
    expect(button.attributes).toHaveLength(1)
    expect(button.firstDesc).toBe('A button.')

    const card = workspace.entries.find((e) => e.tagName === 'fx-card')!
    expect(card.package).toBe('@fx/umbrella')
    expect(card.private).toBe(true)

    const pubTags = published.entries.map((e) => e.tagName)
    expect(pubTags).toEqual(['fx-button'])
    expect(published.count).toBe(1)
  })

  it('tags non-packaged components from .ship/sui-out by origin', async () => {
    await write(
      'package.json',
      JSON.stringify({
        name: 'fixture-root',
        private: true,
        workspaces: { packages: ['packages/*'] },
      }),
    )
    // Shell-local: lives under apps/shell/...
    await write(
      '.ship/sui-out/apps/shell/src/components/sui-rail/sui-rail.custom-elements.json',
      cem('sui-rail', 'SuiRail'),
    )
    // View-local: lives under views/<name>/...
    await write(
      '.ship/sui-out/views/agents/components/agents-row/agents-row.custom-elements.json',
      cem('agents-row', 'AgentsRow'),
    )

    const { workspace } = await buildCatalogs({ repoRoot: root })
    const byTag = Object.fromEntries(workspace.entries.map((e) => [e.tagName, e]))
    expect(byTag['sui-rail']!.package).toBe('app:shell')
    expect(byTag['sui-rail']!.private).toBe(true)
    expect(byTag['sui-rail']!.bundleUrl).toBe('/_sui/sui-components/sui-rail.js')
    expect(byTag['sui-rail']!.sourcePath).toBe(
      'apps/shell/src/components/sui-rail/sui-rail.sui',
    )
    expect(byTag['agents-row']!.package).toBe('view:agents')
  })

  it('originForNonPackagedSource heuristics', () => {
    expect(originForNonPackagedSource('apps/shell/src/foo/foo.sui')).toBe('app:shell')
    expect(originForNonPackagedSource('views/agents/components/x/x.sui')).toBe('view:agents')
    expect(originForNonPackagedSource('crates/foo/assets/views/git/x.sui')).toBe('view:git')
    expect(originForNonPackagedSource('docs/page.sui')).toBe('workspace:docs')
  })

  it('handles missing .ship/sui-out cleanly', async () => {
    await write(
      'package.json',
      JSON.stringify({ name: 'empty', private: true, workspaces: { packages: [] } }),
    )
    const { workspace, published } = await buildCatalogs({ repoRoot: root })
    expect(workspace.count).toBe(0)
    expect(published.count).toBe(0)
  })

  it('drops sui-out entries that duplicate a packaged tag (first-wins)', async () => {
    await write(
      'package.json',
      JSON.stringify({
        name: 'fixture-root',
        private: true,
        workspaces: { packages: ['packages/*'] },
      }),
    )
    await write(
      'packages/widgets/package.json',
      JSON.stringify({ name: '@fx/widgets', customElements: 'dist/custom-elements.json' }),
    )
    await write(
      'packages/widgets/dist/custom-elements.json',
      cem('fx-button', 'FxButton'),
    )
    await write('packages/widgets/dist/fx-widgets.js', '/* bundle */')
    // Stale sui-out CEM for the same tag — must be skipped (under packaged root).
    await write(
      '.ship/sui-out/packages/widgets/src/fx-button/fx-button.custom-elements.json',
      cem('fx-button', 'FxButton'),
    )

    const { workspace } = await buildCatalogs({ repoRoot: root })
    const buttons = workspace.entries.filter((e) => e.tagName === 'fx-button')
    expect(buttons).toHaveLength(1)
    expect(buttons[0]!.package).toBe('@fx/widgets')
  })
})
