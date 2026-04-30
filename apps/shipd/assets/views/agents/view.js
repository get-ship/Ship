// views/agents/view.ts
import { controller, unwrap } from "@ship-set/runtime";
var view_default = controller({
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    total: ({ items }) => items().length
  },
  on: {
    newAgent: ({ navigate }) => navigate("/agents/create")
  }
});
export {
  view_default as default
};
