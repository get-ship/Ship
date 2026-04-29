import { describe, expect, it } from 'vitest';
import * as path from 'node:path';
import { loadSetup, analyzeSetup, pairedSetupPath } from '../src/setup';

const FIXTURES = path.join(__dirname, 'fixtures');

describe('setup', () => {
  it('resolves paired ts path', () => {
    expect(pairedSetupPath('/x/y/view.html.set')).toBe('/x/y/view.ts');
    expect(pairedSetupPath('/x/y/foo.set')).toBe('/x/y/foo.ts');
    expect(pairedSetupPath('/x/y/foo.unrelated')).toBeNull();
  });

  it('extracts members via type argument', () => {
    const info = loadSetup(path.join(FIXTURES, 'profile.html.set'));
    expect(info).not.toBeNull();
    const names = info!.members.map(m => m.name).sort();
    expect(names).toEqual(['age', 'filter', 'name', 'save']);
    const filter = info!.members.find(m => m.name === 'filter');
    expect(filter!.type).toBe('() => string');
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
});
