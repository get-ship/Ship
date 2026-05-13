// empty-controller diagnostic.
//
// Flags `controller(...)` factory calls (or `export default null`)
// whose argument carries no behavior. Controllers are optional in
// SUI, so empty files add ceremony with no payoff — the diagnostic
// recommends deleting the file.

import { describe, expect, it } from 'vitest';
import { diagnoseSetup } from '../src/setup-diagnostics';

const findEmpty = (src: string) =>
  diagnoseSetup('/tmp/c.controller.ts', src).find(d => d.code === 'controller.empty');

describe('empty-controller diagnostic', () => {
  it('flags `controller({})`', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({});
    `;
    const diag = findEmpty(src);
    expect(diag).toBeDefined();
    expect(diag!.severity).toBe('warning');
    expect(diag!.message).toMatch(/Empty controller/);
  });

  it('flags `controller<Scope>({})`', () => {
    const src = `
      declare function controller<T>(def: T): T;
      type Scope = { count: () => number };
      export default controller<Scope>({});
    `;
    expect(findEmpty(src)).toBeDefined();
  });

  it('flags `controller<Scope>({ scope: () => ({}) })`', () => {
    const src = `
      declare function controller<T>(def: T): T;
      type Scope = {};
      export default controller<Scope>({ scope: () => ({}) });
    `;
    expect(findEmpty(src)).toBeDefined();
  });

  it('flags `controller({ scope: () => ({}), derived: {}, on: {}, mount: () => {} })`', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({
        scope: () => ({}),
        derived: {},
        on: {},
        mount: () => {},
      });
    `;
    expect(findEmpty(src)).toBeDefined();
  });

  it('flags `controller()` with no argument', () => {
    const src = `
      declare function controller(def?: unknown): unknown;
      export default controller();
    `;
    expect(findEmpty(src)).toBeDefined();
  });

  it('flags `export default null`', () => {
    const src = `export default null;`;
    expect(findEmpty(src)).toBeDefined();
  });

  it('does not flag a controller with a non-empty scope', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({
        scope: () => ({ count: 0 }),
      });
    `;
    expect(findEmpty(src)).toBeUndefined();
  });

  it('does not flag a controller with a non-empty mount body', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({
        mount: (_state, host) => { host.setAttribute('role', 'list'); },
      });
    `;
    expect(findEmpty(src)).toBeUndefined();
  });

  it('does not flag a controller with a `mount` method body', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({
        mount(_state, host) { host.setAttribute('role', 'list'); },
      });
    `;
    expect(findEmpty(src)).toBeUndefined();
  });

  it('does not flag a controller with `derived` entries', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({
        derived: { doubled: (s) => s.count() * 2 },
      });
    `;
    expect(findEmpty(src)).toBeUndefined();
  });

  it('does not flag a controller with `on` handlers', () => {
    const src = `
      declare function controller<T>(def: T): T;
      export default controller({
        on: { click: () => {} },
      });
    `;
    expect(findEmpty(src)).toBeUndefined();
  });

  it('does not flag a non-controller default export', () => {
    const src = `
      export default { hello: 'world' };
    `;
    expect(findEmpty(src)).toBeUndefined();
  });

  it('does not flag a file with no default export', () => {
    const src = `
      export const x = 1;
    `;
    expect(findEmpty(src)).toBeUndefined();
  });
});
