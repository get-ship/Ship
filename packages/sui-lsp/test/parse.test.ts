import { describe, expect, it } from 'vitest';
import { parse } from '../src/parse';

describe('parse', () => {
  it('parses element with attributes', () => {
    const r = parse(`<ship-button tone="primary" on:click={save()}>x</ship-button>`);
    expect(r.errors).toEqual([]);
    const open = r.nodes[0];
    expect(open.kind).toBe('element-open');
    if (open.kind !== 'element-open') throw new Error('bad');
    expect(open.tag).toBe('ship-button');
    expect(open.attrs.length).toBe(2);
    expect(open.attrs[0]!.kind).toBe('static');
    expect(open.attrs[0]!.name).toBe('tone');
    expect(open.attrs[1]!.kind).toBe('on');
    expect(open.attrs[1]!.name).toBe('click');
    expect(open.attrs[1]!.value?.text).toBe('save()');
  });

  it('parses self-closing element', () => {
    const r = parse(`<ship-icon name="plus" />`);
    expect(r.errors).toEqual([]);
    const open = r.nodes[0];
    if (open.kind !== 'element-open') throw new Error('bad');
    expect(open.selfClosing).toBe(true);
  });

  it('parses interpolation with nested braces', () => {
    const r = parse(`<p>{a + {b: 1}.b}</p>`);
    expect(r.errors).toEqual([]);
    const interp = r.nodes.find(n => n.kind === 'interpolation');
    expect(interp).toBeDefined();
    if (interp?.kind === 'interpolation') {
      expect(interp.text).toBe('a + {b: 1}.b');
    }
  });

  it('parses for-block with key', () => {
    const r = parse(`{#for a of items() key={a.id}}<x />{/for}`);
    expect(r.errors).toEqual([]);
    const open = r.nodes[0];
    if (open.kind !== 'block-open') throw new Error('bad');
    expect(open.block).toBe('for');
    expect(open.pattern?.idents).toEqual(['a']);
    expect(open.iter?.text).toBe('items()');
    expect(open.key?.text).toBe('a.id');
  });

  it('parses if/else', () => {
    const r = parse(`{#if x()}<a />{:else}<b />{/if}`);
    expect(r.errors).toEqual([]);
    expect(r.nodes.find(n => n.kind === 'block-tag' && n.tag === 'else')).toBeDefined();
  });

  it('parses bind: with modifiers on event', () => {
    const r = parse(`<input on:keydown|preventDefault|debounce:200={k()} />`);
    expect(r.errors).toEqual([]);
    const open = r.nodes[0];
    if (open.kind !== 'element-open') throw new Error('bad');
    const ev = open.attrs[0]!;
    expect(ev.kind).toBe('on');
    expect(ev.modifiers).toEqual(['preventDefault', 'debounce']);
  });
});
