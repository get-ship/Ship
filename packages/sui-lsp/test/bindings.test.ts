import { describe, expect, it } from 'vitest';
import { parse } from '../src/parse';
import { buildScope } from '../src/bindings';

describe('bindings', () => {
  it('extracts data declarations', () => {
    const src = `<data name="foo" source={ship.x()} /><p>{foo}</p>`;
    const { nodes } = parse(src);
    const scope = buildScope(nodes, src);
    expect(scope.declarations.find(d => d.name === 'foo' && d.source === 'data')).toBeDefined();
    expect(scope.references.some(r => r.name === 'foo')).toBe(true);
  });

  it('extracts state declarations', () => {
    const src = `<state name="count" value={0} />{count()}`;
    const { nodes } = parse(src);
    const scope = buildScope(nodes, src);
    expect(scope.declarations.find(d => d.name === 'count' && d.source === 'state')).toBeDefined();
  });

  it('extracts let: bindings scoped to element body', () => {
    const src = `<while-error bind="agents" let:err><p>{err.message}</p></while-error>`;
    const { nodes } = parse(src);
    const scope = buildScope(nodes, src);
    const errDecl = scope.declarations.find(d => d.name === 'err' && d.source === 'let');
    expect(errDecl).toBeDefined();
    const errRef = scope.references.find(r => r.name === 'err');
    expect(errRef).toBeDefined();
    expect(errRef!.range.start).toBeGreaterThanOrEqual(errDecl!.extent.start);
    expect(errRef!.range.end).toBeLessThanOrEqual(errDecl!.extent.end);
  });

  it('introduces for-loop pattern bindings', () => {
    const src = `{#for item of items() key={item.id}}<p>{item.name}</p>{/for}`;
    const { nodes } = parse(src);
    const scope = buildScope(nodes, src);
    const decl = scope.declarations.find(d => d.name === 'item' && d.source === 'for');
    expect(decl).toBeDefined();
    const ref = scope.references.find(r => r.name === 'item');
    expect(ref).toBeDefined();
  });

  it('skips property accessors for reference extraction', () => {
    const src = `<p>{a.message}</p>`;
    const { nodes } = parse(src);
    const scope = buildScope(nodes, src);
    const refs = scope.references.filter(r => r.name === 'message');
    expect(refs.length).toBe(0);
  });
});
