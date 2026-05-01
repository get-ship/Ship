// D-2 — `use:` namespacing.
//
// `controller({ use: { list: useFilteredList(...) } })` registers
// `list` as a namespace whose members are the contributions returned
// by `useFilteredList`. Templates referencing `{list.filtered()}`
// must validate against that namespace.

import { describe, expect, it } from 'vitest';
import { parse } from '../src/parse';
import { complete } from '../src/completions';
import { hover } from '../src/hover';
import { diagnose } from '../src/diagnostics';
import { analyzeSetup } from '../src/setup';

const HOOK_FACTORY_SRC = `
function useFilteredList(source, predicate) {
  return {
    state: { query: '' },
    derived: {
      filtered: (scope): string[] =>
        source().filter((i) => predicate(i, scope.query())),
    },
    on: {
      setQuery: (s, q: string) => { s.query = q },
      clear: (s) => { s.query = '' },
    },
  };
}
`;

const SETUP_SRC = `${HOOK_FACTORY_SRC}
declare function controller<T>(def: T): T;
export default controller({
  use: { list: useFilteredList([], (a, q) => a.includes(q)) },
});
`;

describe('use: namespacing (D-2)', () => {
  it('records namespace + contribution shape', () => {
    const info = analyzeSetup('/tmp/view.ts', SETUP_SRC);
    const ns = info.namespaces?.find(n => n.alias === 'list');
    expect(ns).toBeDefined();
    expect(ns!.hookFactory).toBe('useFilteredList');
    const names = ns!.members.map(m => m.name).sort();
    expect(names).toEqual(['clear', 'filtered', 'query', 'setQuery']);
    const filtered = ns!.members.find(m => m.name === 'filtered')!;
    expect(filtered.kind).toBe('derived');
    expect(filtered.signature).toContain('string[]');
  });

  it('hover on `filtered` shows the contribution type', () => {
    const tpl = `<p>{list.filtered()}</p>`;
    const info = analyzeSetup('/tmp/view.ts', SETUP_SRC);
    const idx = tpl.indexOf('filtered') + 2; // inside `filtered`
    const h = hover(tpl, idx, parse(tpl).nodes, info);
    expect(h).not.toBeNull();
    expect(h!.contents).toContain('list.filtered');
    expect(h!.contents).toContain('string[]');
    expect(h!.contents).toContain('derived');
  });

  it('completion at `list.` lists the namespace members', () => {
    const tpl = `<p>{list.}</p>`;
    const info = analyzeSetup('/tmp/view.ts', SETUP_SRC);
    const idx = tpl.indexOf('list.') + 'list.'.length;
    const items = complete({ source: tpl, offset: idx, nodes: parse(tpl).nodes, setup: info });
    const labels = items.map(i => i.label).sort();
    expect(labels).toEqual(['clear', 'filtered', 'query', 'setQuery']);
  });

  it('flags references to unknown contributions as diagnostics', () => {
    const tpl = `<p>{list.bogus()}</p>`;
    const info = analyzeSetup('/tmp/view.ts', SETUP_SRC);
    const { nodes, errors } = parse(tpl);
    const diags = diagnose(tpl, nodes, errors, info);
    const bad = diags.find(d => d.code === 'use.unknown-member');
    expect(bad).toBeDefined();
    expect(bad!.message).toContain('list');
    expect(bad!.message).toContain('bogus');
    const idx = tpl.indexOf('bogus');
    expect(bad!.range.start).toBe(idx);
    expect(bad!.range.end).toBe(idx + 'bogus'.length);
  });

  it('does not flag known members', () => {
    const tpl = `<p>{list.filtered()}</p>`;
    const info = analyzeSetup('/tmp/view.ts', SETUP_SRC);
    const { nodes, errors } = parse(tpl);
    const diags = diagnose(tpl, nodes, errors, info);
    expect(diags.find(d => d.code === 'use.unknown-member')).toBeUndefined();
    expect(diags.find(d => d.code === 'unbound')).toBeUndefined();
  });
});
