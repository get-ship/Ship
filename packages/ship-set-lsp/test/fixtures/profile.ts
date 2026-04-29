interface Bindings {
  name: () => string;
  age: () => number;
  filter: () => string;
  save: () => void;
}

declare function view<T>(setup: (ctx: { ship: unknown; navigate: (p: string) => void }) => T): unknown;

export default view<Bindings>((_ctx) => {
  return {
    name: () => 'Alice',
    age: () => 30,
    filter: () => 'all',
    save: () => {},
  };
});
