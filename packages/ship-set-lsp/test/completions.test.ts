import { describe, expect, it } from 'vitest';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { parse } from '../src/parse';
import { complete } from '../src/completions';
import { loadSetup } from '../src/setup';

const FIXTURES = path.join(__dirname, 'fixtures');

describe('completions', () => {
  it('suggests primitive tags after `<ship-`', () => {
    const src = `<ship-`;
    const items = complete({
      source: src, offset: src.length, nodes: parse(src).nodes, setup: null,
      primitiveTags: ['ship-button', 'ship-card', 'ship-icon'],
    });
    const ship = items.filter(i => i.label.startsWith('ship-'));
    expect(ship.length).toBeGreaterThan(0);
    expect(ship.find(i => i.label === 'ship-button')).toBeDefined();
  });

  it('suggests directive elements among tags', () => {
    const src = `<d`;
    const items = complete({ source: src, offset: src.length, nodes: parse(src).nodes, setup: null });
    expect(items.find(i => i.label === 'data')).toBeDefined();
  });

  it('suggests event names after `on:`', () => {
    const src = `<button on:c`;
    const items = complete({ source: src, offset: src.length, nodes: parse(src).nodes, setup: null });
    expect(items.find(i => i.label === 'click')).toBeDefined();
    expect(items.find(i => i.label === 'change')).toBeDefined();
  });

  it('suggests in-scope identifiers from paired setup inside `{…}`', () => {
    const tplPath = path.join(FIXTURES, 'profile.html.set');
    const src = fs.readFileSync(tplPath, 'utf8');
    const setup = loadSetup(tplPath);
    // Position inside `{age()}`
    const idx = src.indexOf('{age()}') + 1; // just after the `{`
    const items = complete({ source: src, offset: idx + 'ag'.length, nodes: parse(src).nodes, setup });
    expect(items.find(i => i.label === 'age')).toBeDefined();
    expect(items.find(i => i.label === 'filter')).toBeDefined();
    expect(items.find(i => i.label === 'name')).toBeDefined();
  });

  it('suggests event modifiers after `on:click|`', () => {
    const src = `<button on:click|`;
    const items = complete({ source: src, offset: src.length, nodes: parse(src).nodes, setup: null });
    expect(items.find(i => i.label === 'preventDefault')).toBeDefined();
    expect(items.find(i => i.label === 'stop')).toBeDefined();
  });
});
