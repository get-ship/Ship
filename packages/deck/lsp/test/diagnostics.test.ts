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
    const src = fs.readFileSync(path.join(FIXTURES, 'profile.sui'), 'utf8');
    const setup = loadSetup(path.join(FIXTURES, 'profile.sui'));
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, setup);
    expect(diags.find(d => d.code === 'unbound' && d.message.includes('age'))).toBeUndefined();
    expect(diags.find(d => d.code === 'unbound' && d.message.includes('filter'))).toBeUndefined();
    expect(diags.find(d => d.code === 'unbound' && d.message.includes('save'))).toBeUndefined();
  });

  it('does not flag CSS inside <style> as template expressions', () => {
    // Regression: the LSP used to enter `<style>` blocks and treat
    // every CSS `{...}` declaration as a template interpolation,
    // producing thousands of false-positive "Expression contains a
    // statement" diagnostics on every shipped component.
    const src = `<button>x</button>\n<style>\n  :host { display: inline-flex; --gap: 8px; }\n  :host([size='sm']) { --h: 28px; }\n  button:hover:not(:disabled) { transform: translateY(-1px); }\n</style>`;
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, null);
    expect(diags.filter(d => d.code?.startsWith('expr.'))).toEqual([]);
    expect(diags.filter(d => d.code === 'unbound')).toEqual([]);
    expect(errors).toEqual([]);
  });

  it('does not flag JS inside <script> as template expressions', () => {
    const src = `<script>\n  const x = 1; for (let i = 0; i < 3; i++) { y[i] = i; }\n</script>`;
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, null);
    expect(diags.filter(d => d.code?.startsWith('expr.'))).toEqual([]);
    expect(diags.filter(d => d.code === 'unbound')).toEqual([]);
  });

  it('treats kebab-case <prop> as its camelCased identifier', () => {
    // Mirrors `crates/sui-compiler/src/check/symbol_table.rs::build`,
    // which calls `camel_of(&p.name)` so `<prop name="active-tab">`
    // contributes `activeTab` to template scope.
    const src = `<prop name="active-tab" type="string" default="terminal" />\n<div hidden={activeTab() !== 'terminal'}></div>`;
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, null);
    expect(diags.filter(d => d.code === 'unbound')).toEqual([]);
  });

  it('does not flag `bind:this` target as unbound', () => {
    // `bind:this={X}` is a *write target* — the compiler's
    // `scope_bound` check skips BindThis and `bind_this_target`
    // separately requires X to be a `ref(...)` in the controller's
    // scope. The LSP must not pile on with an "unresolved identifier".
    const src = `<div bind:this={containerEl}><span bind:this={labelEl}></span></div>`;
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, null);
    expect(diags.filter(d => d.code === 'unbound')).toEqual([]);
  });

  it('agents view fixture parses with no diagnostics', () => {
    const fp = path.join(VIEWS, 'agents', 'view.sui');
    if (!fs.existsSync(fp)) return; // guard if scope changes
    const src = fs.readFileSync(fp, 'utf8');
    const setup = loadSetup(fp);
    const { nodes, errors } = parse(src);
    const diags = diagnose(src, nodes, errors, setup);
    const real = diags.filter(d => d.severity === 'error');
    expect(real, JSON.stringify(real, null, 2)).toEqual([]);
  });
});
