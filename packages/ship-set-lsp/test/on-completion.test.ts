// D-4 — handler completion + signature.
//
// `on:click={…}` braces should complete against the controller's `on`
// section keys, with each item carrying the handler's typed signature
// for the IDE's completion-detail / signature-hint surface.

import { describe, expect, it } from 'vitest';
import { parse } from '../src/parse';
import { complete } from '../src/completions';
import { hover } from '../src/hover';
import { analyzeSetup } from '../src/setup';

const SETUP_SRC = `
declare function controller<T>(def: T): T;
type MutableState<T> = T;
type Scope = { count: number };
export default controller({
  on: {
    increment: (s: MutableState<Scope>, e: MouseEvent): void => { s.count++ },
    reset: (s: MutableState<Scope>): void => { s.count = 0 },
  },
});
`;

describe('on:event handler completion (D-4)', () => {
  it('extracts on-handlers with typed signatures', () => {
    const info = analyzeSetup('/tmp/view.ts', SETUP_SRC);
    const incr = info.onHandlers?.find(h => h.name === 'increment');
    expect(incr).toBeDefined();
    expect(incr!.signature).toBe('(s: MutableState<Scope>, e: MouseEvent) => void');
  });

  it('completes `on:click={incr|}` against controller `on` keys', () => {
    const tpl = `<button on:click={incr}></button>`;
    const info = analyzeSetup('/tmp/view.ts', SETUP_SRC);
    const idx = tpl.indexOf('incr') + 'incr'.length;
    const items = complete({ source: tpl, offset: idx, nodes: parse(tpl).nodes, setup: info });
    const labels = items.map(i => i.label).sort();
    expect(labels).toEqual(['increment', 'reset']);
    const incrItem = items.find(i => i.label === 'increment')!;
    expect(incrItem.detail).toBe('(s: MutableState<Scope>, e: MouseEvent) => void');
  });

  it('completes inside an empty `on:click={}` brace', () => {
    const tpl = `<button on:click={}></button>`;
    const info = analyzeSetup('/tmp/view.ts', SETUP_SRC);
    const idx = tpl.indexOf('={') + 2; // just after `{`
    const items = complete({ source: tpl, offset: idx, nodes: parse(tpl).nodes, setup: info });
    expect(items.find(i => i.label === 'increment')).toBeDefined();
    expect(items.find(i => i.label === 'reset')).toBeDefined();
  });

  it('hover on an on-handler reference shows its signature', () => {
    const tpl = `<button on:click={increment}></button>`;
    const info = analyzeSetup('/tmp/view.ts', SETUP_SRC);
    const idx = tpl.indexOf('increment') + 2;
    const h = hover(tpl, idx, parse(tpl).nodes, info);
    expect(h).not.toBeNull();
    expect(h!.contents).toContain('increment');
    expect(h!.contents).toContain('(s: MutableState<Scope>, e: MouseEvent) => void');
  });

  it('inside non-on braces, controller `on` completion is not the only menu', () => {
    // In a regular `{…}` interpolation the menu is the in-scope identifier list,
    // not the `on:` short menu. Sanity-check that the regular path runs.
    const tpl = `<p>{}</p>`;
    const info = analyzeSetup('/tmp/view.ts', SETUP_SRC);
    const idx = tpl.indexOf('{') + 1;
    const items = complete({ source: tpl, offset: idx, nodes: parse(tpl).nodes, setup: info });
    // Should include globals (e.g. `Math`) — the identifier-completion menu.
    expect(items.find(i => i.label === 'Math')).toBeDefined();
  });
});
