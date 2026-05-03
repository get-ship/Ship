// views/counter/view.js
import { bindText, on, signal, view } from "@ship/ui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateOn, hydrateText } from "@ship/ui-dom";

// views/counter/view.ts
var view_default = null;

// views/counter/view.js
var view_default2 = view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = /* @__PURE__ */ Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.count = signal(0);
  if (view_default && typeof view_default.setup === "function") {
    const __r = view_default.setup(ctx);
    if (__r && typeof __r === "object") Object.assign($scope, __r);
  }
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __t0 = document.createTextNode("\n\n");
      container.appendChild(__t0);
      const __e1 = document.createElement("div");
      __e1.setAttribute("class", "counter");
      const __t2 = document.createTextNode("\n  ");
      __e1.appendChild(__t2);
      const __e3 = document.createElement("h1");
      const __t4 = document.createTextNode("Counter");
      __e3.appendChild(__t4);
      __e1.appendChild(__e3);
      const __t5 = document.createTextNode("\n  ");
      __e1.appendChild(__t5);
      const __e6 = document.createElement("p");
      const __t7 = document.createTextNode("Count: ");
      __e6.appendChild(__t7);
      const __e8 = document.createElement("span");
      __e8.setAttribute("data-test", "value");
      const __t9 = document.createTextNode("");
      __e8.appendChild(__t9);
      __cleanups.push(bindText(__t9, () => $scope.count()));
      __e6.appendChild(__e8);
      __e1.appendChild(__e6);
      const __t10 = document.createTextNode("\n  ");
      __e1.appendChild(__t10);
      const __e11 = document.createElement("button");
      __e11.setAttribute("data-test", "dec");
      __cleanups.push(on(__e11, "click", (function() {
        const __h = (() => $scope.count.set($scope.count() - 1));
        return typeof __h === "function" ? __h.bind(void 0) : (() => {
        });
      })(), { modifiers: [] }));
      const __t12 = document.createTextNode("-");
      __e11.appendChild(__t12);
      __e1.appendChild(__e11);
      const __t13 = document.createTextNode("\n  ");
      __e1.appendChild(__t13);
      const __e14 = document.createElement("button");
      __e14.setAttribute("data-test", "inc");
      __cleanups.push(on(__e14, "click", (function() {
        const __h = (() => $scope.count.set($scope.count() + 1));
        return typeof __h === "function" ? __h.bind(void 0) : (() => {
        });
      })(), { modifiers: [] }));
      const __t15 = document.createTextNode("+");
      __e14.appendChild(__t15);
      __e1.appendChild(__e14);
      const __t16 = document.createTextNode("\n  ");
      __e1.appendChild(__t16);
      const __e17 = document.createElement("button");
      __e17.setAttribute("data-test", "reset");
      __cleanups.push(on(__e17, "click", (function() {
        const __h = (() => $scope.count.set(0));
        return typeof __h === "function" ? __h.bind(void 0) : (() => {
        });
      })(), { modifiers: [] }));
      const __t18 = document.createTextNode("reset");
      __e17.appendChild(__t18);
      __e1.appendChild(__e17);
      const __t19 = document.createTextNode("\n  ");
      __e1.appendChild(__t19);
      const __e20 = document.createElement("p");
      __e20.setAttribute("class", "caption");
      const __t21 = document.createTextNode("Tier 1: no controller, declarative state, the simplest possible Ship Set view.");
      __e20.appendChild(__t21);
      __e1.appendChild(__e20);
      const __t22 = document.createTextNode("\n");
      __e1.appendChild(__t22);
      container.appendChild(__e1);
      const __t23 = document.createTextNode("\n");
      container.appendChild(__t23);
      return () => {
        for (const c of __cleanups) try {
          c();
        } catch (e) {
        }
      };
    },
    hydrate(root, ctx2) {
      const __cleanups = [];
      const __t = collectTextOpens(root);
      const __m = collectElementMarkers(root);
      if (__t["b0"]) __cleanups.push(hydrateText(__t["b0"], () => $scope.count()));
      if (__m["b1"]) __cleanups.push(hydrateOn(__m["b1"], "click", (function() {
        const __h = (() => $scope.count.set($scope.count() - 1));
        return typeof __h === "function" ? __h.bind(void 0) : (() => {
        });
      })(), []));
      if (__m["b2"]) __cleanups.push(hydrateOn(__m["b2"], "click", (function() {
        const __h = (() => $scope.count.set($scope.count() + 1));
        return typeof __h === "function" ? __h.bind(void 0) : (() => {
        });
      })(), []));
      if (__m["b3"]) __cleanups.push(hydrateOn(__m["b3"], "click", (function() {
        const __h = (() => $scope.count.set(0));
        return typeof __h === "function" ? __h.bind(void 0) : (() => {
        });
      })(), []));
      return () => {
        for (const c of __cleanups) try {
          c();
        } catch (e) {
        }
      };
    }
  });
});
var bindings = [];
export {
  bindings,
  view_default2 as default
};
