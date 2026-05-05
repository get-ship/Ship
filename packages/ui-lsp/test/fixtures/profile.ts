// Test fixture exercising the LSP's Form C (`controller(def)`).
// `name` and `age` come from the controller's `scope:` block (signals),
// `filter` from `derived`, `save` from `on`. All four end up in
// template scope.

declare function controller<T>(def: T): T;
declare function signal<T>(initial: T): { (): T; set(v: T): void };

export default controller({
  scope: () => ({ name: signal('Alice'), age: signal(30) }),
  derived: { filter: () => 'all' },
  on: { save: () => {} },
});
