import { describe, expect, it } from 'vitest';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { parse } from '../src/parse';
import { hover } from '../src/hover';
import { loadSetup } from '../src/setup';

const FIXTURES = path.join(__dirname, 'fixtures');

describe('hover', () => {
  it('returns setup-return type for an identifier in `{…}`', () => {
    const tplPath = path.join(FIXTURES, 'profile.html.set');
    const src = fs.readFileSync(tplPath, 'utf8');
    const setup = loadSetup(tplPath);
    const idx = src.indexOf('{filter()}') + 2; // inside `filter`
    const h = hover(src, idx, parse(src).nodes, setup);
    expect(h).not.toBeNull();
    expect(h!.contents).toContain('() => string');
  });

  it('returns directive doc on `<data>` tag', () => {
    const src = `<data name="x" source={ship.x()} />`;
    const idx = src.indexOf('data') + 1;
    const h = hover(src, idx, parse(src).nodes, null);
    expect(h?.contents).toContain('directive');
  });

  it('returns CEM TODO for ship-* tag', () => {
    const src = `<ship-button>x</ship-button>`;
    const idx = src.indexOf('ship-button') + 2;
    const h = hover(src, idx, parse(src).nodes, null);
    expect(h?.contents).toContain('ship-button');
  });

  it('returns null on stray text', () => {
    const src = `hello world`;
    const h = hover(src, 3, parse(src).nodes, null);
    expect(h).toBeNull();
  });
});
