// Test fixture exercising the LSP's Form C (`controller(def)`).
// `filter` and `save` end up in template scope — `filter` from
// `derived`, `save` from `on`. `name` and `age` are template-declared
// `<state>` directives in profile.html.set, so they don't show up
// from controller-def extraction (the template parser surfaces them
// separately).

declare function controller<T>(def: T): T;

export default controller({
  derived: { filter: () => 'all' },
  on: { save: () => {} },
});
