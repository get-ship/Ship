import { describe, expect, it } from 'vitest';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { parse } from '../src/parse';
import { diagnose } from '../src/diagnostics';
import { loadSetup } from '../src/setup';

const FIXTURES = path.join(__dirname, 'fixtures');
const VIEWS = path.join(__dirname, '..', '..', '..', 'views');

describe('diagnostics', () => {
  it('flags missing required directive attributes', () => {
    const src = `<data name="x" />`;
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, null);
    expect(diags.find(d => d.code === 'directive.data')).toBeDefined();
  });

  it('flags assignment in expression', () => {
    const src = `<input bind:value={x = 1} />`;
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, null);
    const assign = diags.find(d => d.code?.startsWith('expr.assign'));
    expect(assign).toBeDefined();
    // Span sits inside the brace expression.
    const idx = src.indexOf('x = 1');
    expect(assign!.range.start).toBeGreaterThanOrEqual(idx);
    expect(assign!.range.end).toBeLessThanOrEqual(idx + 'x = 1'.length);
  });

  it('allows arrow functions only in `on:` handlers', () => {
    const yes = parse(`<button on:click={() => save()} />`);
    expect(diagnose(yes.source, yes.nodes, yes.errors, null).find(d => d.code?.includes('arrow'))).toBeUndefined();
    const no = parse(`<p>{() => save()}</p>`);
    expect(diagnose(no.source, no.nodes, no.errors, null).find(d => d.code?.includes('arrow'))).toBeDefined();
  });

  it('flags unbound identifier when no setup', () => {
    const src = `<p>{undeclared()}</p>`;
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, null);
    expect(diags.find(d => d.code === 'unbound' && d.message.includes('undeclared'))).toBeDefined();
  });

  it('resolves identifier from paired setup', () => {
    const src = fs.readFileSync(path.join(FIXTURES, 'profile.html.set'), 'utf8');
    const setup = loadSetup(path.join(FIXTURES, 'profile.html.set'));
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, setup);
    expect(diags.find(d => d.code === 'unbound' && d.message.includes('age'))).toBeUndefined();
    expect(diags.find(d => d.code === 'unbound' && d.message.includes('filter'))).toBeUndefined();
    expect(diags.find(d => d.code === 'unbound' && d.message.includes('save'))).toBeUndefined();
  });

  it('agents view fixture parses with no diagnostics', () => {
    const fp = path.join(VIEWS, 'agents', 'view.html.set');
    if (!fs.existsSync(fp)) return; // guard if scope changes
    const src = fs.readFileSync(fp, 'utf8');
    const setup = loadSetup(fp);
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, setup);
    const real = diags.filter(d => d.severity === 'error');
    expect(real, JSON.stringify(real, null, 2)).toEqual([]);
  });
});
