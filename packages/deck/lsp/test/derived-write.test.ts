// D-3 — derived-write diagnostic.
//
// Mutating state inside a `derived` section is a type error (the
// section receives a `ReadonlyState<…>` view) but the LSP needs to
// surface it as a Warning marker so the editor lights up the line
// even when `tsc --noEmit` hasn't run.

import { describe, expect, it } from 'vitest';
import { diagnoseSetup } from '../src/setup-diagnostics';

describe('derived-write diagnostic (D-3)', () => {
  it('flags assignment to state inside a derived section', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({
        derived: {
          x: (s) => { s.count = 5; return s.count; },
        },
      });
    `;
    const diags = diagnoseSetup('/tmp/view.ts', src);
    const warn = diags.find(d => d.code === 'derived.mutates-state');
    expect(warn).toBeDefined();
    expect(warn!.severity).toBe('warning');
    expect(warn!.message).toContain('derived sections cannot mutate state');
    const idx = src.indexOf('s.count = 5');
    expect(warn!.range.start).toBe(idx);
    expect(warn!.range.end).toBe(idx + 's.count = 5'.length);
  });

  it('also flags compound assignments and `++` operators', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({
        derived: {
          a: (s) => { s.n += 1; return s.n },
          b: (s) => { s.n++; return s.n },
        },
      });
    `;
    const diags = diagnoseSetup('/tmp/view.ts', src).filter(d => d.code === 'derived.mutates-state');
    expect(diags.length).toBe(2);
  });

  it('does not flag pure derived sections', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({
        derived: {
          doubled: ({ count }) => count() * 2,
          items: ({ agents }) => agents()?.items ?? [],
        },
      });
    `;
    const diags = diagnoseSetup('/tmp/view.ts', src).filter(d => d.code === 'derived.mutates-state');
    expect(diags).toEqual([]);
  });

  it('does not flag mutations inside `on` handlers', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({
        on: { incr: (s) => { s.count++ } },
      });
    `;
    const diags = diagnoseSetup('/tmp/view.ts', src).filter(d => d.code === 'derived.mutates-state');
    expect(diags).toEqual([]);
  });
});
