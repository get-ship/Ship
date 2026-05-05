import { describe, expect, it } from 'vitest';
import * as path from 'node:path';
import { loadSetup, analyzeSetup, pairedSetupPath } from '../src/setup';

const FIXTURES = path.join(__dirname, 'fixtures');

describe('setup', () => {
  it('resolves paired ts path', () => {
    expect(pairedSetupPath('/x/y/view.sui')).toBe('/x/y/view.ts');
    expect(pairedSetupPath('/x/y/foo.unrelated')).toBeNull();
  });

  it('extracts members from controller() fixture', () => {
    const info = loadSetup(path.join(FIXTURES, 'profile.sui'));
    expect(info).not.toBeNull();
    const names = info!.members.map(m => m.name).sort();
    // `name` and `age` are signals declared in the controller's
    // `scope:` factory; `filter` from `derived`; `save` from `on`.
    expect(names).toEqual(['age', 'filter', 'name', 'save']);
  });

  it('falls back to return-statement object literal', () => {
    const src = `
      export default function setup() {
        return { a, b: 1, c() {} };
      }
    `;
    const info = analyzeSetup('/tmp/x.ts', src);
    expect(info.members.map(m => m.name).sort()).toEqual(['a', 'b', 'c']);
  });

  describe('controller(def) — Form C', () => {
    it('extracts scope / derived / on / use keys from object form', () => {
      // Setup analysis surfaces every controller section that
      // contributes to template scope: scope / derived / on / use.
      const src = `
        import { controller } from '@ship/ui-runtime'
        export default controller({
          derived: { doubled: ({ count }) => count() * 2 },
          on: { incr: (s) => { s.count++ }, toggle: (s) => { s.open = !s.open } },
          use: { list: someHook() },
        })
      `;
      const info = analyzeSetup('/tmp/x.ts', src);
      const names = info.members.map((m) => m.name).sort();
      expect(names).toEqual(['doubled', 'incr', 'list', 'toggle']);
    });

    it('extracts keys from function form (arrow returning object)', () => {
      const src = `
        import { controller } from '@ship/ui-runtime'
        export default controller(({ ship }) => ({
          on: { refresh: () => ship.agents.refresh() },
        }))
      `;
      const info = analyzeSetup('/tmp/x.ts', src);
      expect(info.members.map((m) => m.name).sort()).toEqual(['refresh']);
    });

    it('extracts keys from function form (block body with return)', () => {
      const src = `
        import { controller } from '@ship/ui-runtime'
        export default controller(({ ship }) => {
          const stableId = 'x'
          return {
            on: { toggle: (s) => { s.open = !s.open } },
          }
        })
      `;
      const info = analyzeSetup('/tmp/x.ts', src);
      expect(info.members.map((m) => m.name).sort()).toEqual(['toggle']);
    });

    it('handles per-section factory form (derived as function)', () => {
      const src = `
        import { controller } from '@ship/ui-runtime'
        export default controller({
          derived: ({ agents, query }) => ({
            filtered: ({ agents, query }) => agents().filter(a => a.includes(query())),
          }),
        })
      `;
      const info = analyzeSetup('/tmp/x.ts', src);
      expect(info.members.map((m) => m.name).sort()).toEqual(['filtered']);
    });
  });
});
