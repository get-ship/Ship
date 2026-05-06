// views/demos/page.sui
import { view } from "@ship/ui-runtime";

// views/demos/controller.ts
var controller_default = null;

// views/demos/page.sui
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
      __e0.setAttribute("class", "demos-index");
      const __e1 = document.createElement("h1");
      const __t2 = document.createTextNode("SUI demos");
      __e1.appendChild(__t2);
      __e0.appendChild(__e1);
      const __e3 = document.createElement("p");
      __e3.setAttribute("class", "caption");
      const __t4 = document.createTextNode(`
    Small, single-feature views exercising one piece of the SUI surface each.
    Each demo lives at `);
      __e3.appendChild(__t4);
      const __e5 = document.createElement("code");
      const __t6 = document.createTextNode("views/demos/<name>/");
      __e5.appendChild(__t6);
      __e3.appendChild(__e5);
      const __t7 = document.createTextNode(` and is
    50–100 lines of `);
      __e3.appendChild(__t7);
      const __e8 = document.createElement("code");
      const __t9 = document.createTextNode("page.sui");
      __e8.appendChild(__t9);
      __e3.appendChild(__e8);
      const __t10 = document.createTextNode(" + ");
      __e3.appendChild(__t10);
      const __e11 = document.createElement("code");
      const __t12 = document.createTextNode("controller.ts");
      __e11.appendChild(__t12);
      __e3.appendChild(__e11);
      const __t13 = document.createTextNode(`.
  `);
      __e3.appendChild(__t13);
      __e0.appendChild(__e3);
      const __e14 = document.createElement("ul");
      __e14.setAttribute("class", "demos-list");
      const __e15 = document.createElement("li");
      const __e16 = document.createElement("a");
      __e16.setAttribute("href", "/demos/signals-and-derived");
      __e16.setAttribute("data-test", "link-signals");
      const __t17 = document.createTextNode("signals-and-derived");
      __e16.appendChild(__t17);
      __e15.appendChild(__e16);
      const __e18 = document.createElement("span");
      const __t19 = document.createTextNode(" — counter, derived double, history list. ");
      __e18.appendChild(__t19);
      const __e20 = document.createElement("code");
      const __t21 = document.createTextNode("scope:");
      __e20.appendChild(__t21);
      __e18.appendChild(__e20);
      const __t22 = document.createTextNode(" + ");
      __e18.appendChild(__t22);
      const __e23 = document.createElement("code");
      const __t24 = document.createTextNode("derived:");
      __e23.appendChild(__t24);
      __e18.appendChild(__e23);
      const __t25 = document.createTextNode(".");
      __e18.appendChild(__t25);
      __e15.appendChild(__e18);
      __e14.appendChild(__e15);
      const __e26 = document.createElement("li");
      const __e27 = document.createElement("a");
      __e27.setAttribute("href", "/demos/conditional-render");
      __e27.setAttribute("data-test", "link-conditional");
      const __t28 = document.createTextNode("conditional-render");
      __e27.appendChild(__t28);
      __e26.appendChild(__e27);
      const __e29 = document.createElement("span");
      const __t30 = document.createTextNode(" — checkbox toggles a panel. Block ");
      __e29.appendChild(__t30);
      const __e31 = document.createElement("code");
      const __t32 = document.createTextNode("if/else");
      __e31.appendChild(__t32);
      __e29.appendChild(__e31);
      const __t33 = document.createTextNode(".");
      __e29.appendChild(__t33);
      __e26.appendChild(__e29);
      __e14.appendChild(__e26);
      const __e34 = document.createElement("li");
      const __e35 = document.createElement("a");
      __e35.setAttribute("href", "/demos/iteration");
      __e35.setAttribute("data-test", "link-iteration");
      const __t36 = document.createTextNode("iteration");
      __e35.appendChild(__t36);
      __e34.appendChild(__e35);
      const __e37 = document.createElement("span");
      const __t38 = document.createTextNode(" — todo list. Block ");
      __e37.appendChild(__t38);
      const __e39 = document.createElement("code");
      const __t40 = document.createTextNode("for");
      __e39.appendChild(__t40);
      __e37.appendChild(__e39);
      const __t41 = document.createTextNode(" with ");
      __e37.appendChild(__t41);
      const __e42 = document.createElement("code");
      const __t43 = document.createTextNode("key=");
      __e42.appendChild(__t43);
      __e37.appendChild(__e42);
      const __t44 = document.createTextNode(".");
      __e37.appendChild(__t44);
      __e34.appendChild(__e37);
      __e14.appendChild(__e34);
      const __e45 = document.createElement("li");
      const __e46 = document.createElement("a");
      __e46.setAttribute("href", "/demos/match");
      __e46.setAttribute("data-test", "link-match");
      const __t47 = document.createTextNode("match");
      __e46.appendChild(__t47);
      __e45.appendChild(__e46);
      const __e48 = document.createElement("span");
      const __t49 = document.createTextNode(" — alert tone selector. Block ");
      __e48.appendChild(__t49);
      const __e50 = document.createElement("code");
      const __t51 = document.createTextNode("match");
      __e50.appendChild(__t51);
      __e48.appendChild(__e50);
      const __t52 = document.createTextNode(".");
      __e48.appendChild(__t52);
      __e45.appendChild(__e48);
      __e14.appendChild(__e45);
      const __e53 = document.createElement("li");
      const __e54 = document.createElement("a");
      __e54.setAttribute("href", "/demos/lifecycle");
      __e54.setAttribute("data-test", "link-lifecycle");
      const __t55 = document.createTextNode("lifecycle");
      __e54.appendChild(__t55);
      __e53.appendChild(__e54);
      const __e56 = document.createElement("span");
      const __t57 = document.createTextNode(" — mount tick log. ");
      __e56.appendChild(__t57);
      const __e58 = document.createElement("code");
      const __t59 = document.createTextNode("mount:");
      __e58.appendChild(__t59);
      __e56.appendChild(__e58);
      const __t60 = document.createTextNode(" returns cleanup.");
      __e56.appendChild(__t60);
      __e53.appendChild(__e56);
      __e14.appendChild(__e53);
      __e0.appendChild(__e14);
      container.appendChild(__e0);
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
