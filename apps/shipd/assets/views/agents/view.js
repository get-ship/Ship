// views/agents/view.ts
import { derived, live, view } from "@ship-set/runtime";
var view_default = view((ctx) => {
  const ship = ctx.ship;
  const agents = live(() => ship.agents.summaries());
  const items = derived(() => {
    const s = agents();
    return s.kind === "ok" ? s.value.items : [];
  });
  const total = derived(() => items().length);
  return {
    agents,
    items,
    total,
    navigate: ctx.navigate
  };
});
export {
  view_default as default
};
