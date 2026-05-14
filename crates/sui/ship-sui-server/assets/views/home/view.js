// views/home/page.sui
import { bindProp, bindText, on, view } from "@ship/sui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateOn, hydrateProp, hydrateText } from "@ship/sui-dom";

// views/home/controller.ts
import { controller, signal } from "@ship/sui-runtime";
var controller_default = controller({
  scope: () => ({ seen: signal(0) })
});

// views/home/page.sui
var page_default = view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  if (controller_default && typeof controller_default.setup === "function") {
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("div");
      __e0.setAttribute("class", "home");
      const __e1 = document.createElement("home-hero");
      __cleanups.push(bindProp(__e1, "title", () => "Welcome to Ship"));
      __e0.appendChild(__e1);
      const __e2 = document.createElement("section");
      __e2.setAttribute("class", "counter");
      const __e3 = document.createElement("p");
      const __t4 = document.createTextNode("You've opened the home page ");
      __e3.appendChild(__t4);
      const __e5 = document.createElement("strong");
      const __t6 = document.createTextNode("");
      __e5.appendChild(__t6);
      __cleanups.push(bindText(__t6, () => $scope.seen()));
      __e3.appendChild(__e5);
      const __t7 = document.createTextNode(" times this session.");
      __e3.appendChild(__t7);
      __e2.appendChild(__e3);
      const __e8 = document.createElement("button");
      __e8.setAttribute("data-test", "bump");
      __cleanups.push(on(__e8, "click", function() {
        const __h = () => $scope.seen.set($scope.seen() + 1);
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t9 = document.createTextNode("Bump");
      __e8.appendChild(__t9);
      __e2.appendChild(__e8);
      __e0.appendChild(__e2);
      const __e10 = document.createElement("p");
      __e10.setAttribute("class", "caption");
      const __t11 = document.createTextNode(`
    Proof view for the new directory convention — page.sui + controller.ts +
    style.css + manifest.jsonc, with a view-private widget under _components/.
  `);
      __e10.appendChild(__t11);
      __e0.appendChild(__e10);
      container.appendChild(__e0);
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    },
    hydrate(root, ctx2) {
      const __cleanups = [];
      const __t = collectTextOpens(root);
      const __m = collectElementMarkers(root);
      if (__m["b0"])
        __cleanups.push(hydrateProp(__m["b0"], "title", () => "Welcome to Ship"));
      if (__t["b1"])
        __cleanups.push(hydrateText(__t["b1"], () => $scope.seen()));
      if (__m["b2"])
        __cleanups.push(hydrateOn(__m["b2"], "click", function() {
          const __h = () => $scope.seen.set($scope.seen() + 1);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
});
export {
  page_default as default
};
