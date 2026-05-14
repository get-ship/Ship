// views/settings/page.sui
import { bindAttr, bindClass, bindFor, bindIf, bindProp, bindText, bindWhileError, bindWhileLoading, on, signal as signal2, view, wireFormMutation } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/settings/controller.ts
import { controller, signal, live } from "@ship/sui-runtime";
function sseSource(expr) {
  return {
    subscribe(sig) {
      const es = new EventSource(`/_sui/data?expr=${encodeURIComponent(expr)}`);
      const onPatch = (ev) => {
        try {
          const patch = JSON.parse(ev.data);
          if (patch.op === "set")
            sig.set({ kind: "ok", value: patch.value });
        } catch (err) {
          sig.set({ kind: "error", error: err instanceof Error ? err : new Error(String(err)) });
        }
      };
      es.addEventListener("patch", onPatch);
      return () => {
        es.close();
      };
    }
  };
}
function flashSaved(sig, ms = 1600) {
  sig.set(true);
  const w = window;
  const key = `__shipSetSavedTimer_${sig.__id ?? Math.random()}`;
  const existing = w[key];
  if (typeof existing === "number")
    clearTimeout(existing);
  w[key] = setTimeout(() => sig.set(false), ms);
}
async function settingsUpdate(formData) {
  const key = String(formData.get("key") ?? "");
  const value = String(formData.get("value") ?? "");
  if (!key)
    throw new Error('settings.update: missing "key" field');
  const res = await fetch("/api/v1/settings", {
    method: "PUT",
    credentials: "same-origin",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ key, value })
  });
  if (!res.ok) {
    const text = await res.text().catch(() => res.statusText);
    throw new Error(`settings.update failed (${res.status}): ${text}`);
  }
  return { key, value };
}
var controller_default = controller({
  scope: () => ({
    advancedOpen: signal(false),
    savedProvider: signal(false),
    savedModel: signal(false),
    savedTerminal: signal(false),
    savedWorktree: signal(false),
    savedRegistry: signal(false),
    flashSaved,
    ship: { settings: { update: settingsUpdate } },
    settings: live(() => sseSource("ship.settings.state()"))
  })
});

// views/settings/page.sui
var page_default = view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  if (controller_default && typeof controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b1 = (anchor) => bindFor({
    anchor,
    items: () => [0, 1, 2],
    key: (n, __i) => {
      const __child = Object.create($scope);
      __child.n = n;
      __child.__i = __i;
      return (($scope2) => n)(__child);
    },
    render: (n, __i) => {
      const __childScope11 = Object.create($scope);
      __childScope11.n = n;
      __childScope11.__i = __i;
      return (($scope2) => {
        const __frag12 = document.createDocumentFragment();
        const __c13 = [];
        const __e14 = document.createElement("div");
        __e14.setAttribute("class", "s-skel-card");
        const __e15 = document.createElement("div");
        __e15.setAttribute("class", "s-skel-head");
        const __e16 = document.createElement("sui-skeleton");
        __e16.setAttribute("shape", "circle");
        __e16.setAttribute("size", "sm");
        __e15.appendChild(__e16);
        const __e17 = document.createElement("sui-skeleton");
        __e17.setAttribute("shape", "line");
        __e17.setAttribute("size", "md");
        __e17.setAttribute("style", "width:120px");
        __e15.appendChild(__e17);
        __e14.appendChild(__e15);
        const __e18 = document.createElement("div");
        __e18.setAttribute("class", "s-skel-body");
        const __e19 = document.createElement("sui-skeleton");
        __e19.setAttribute("shape", "line");
        __e19.setAttribute("size", "md");
        __e18.appendChild(__e19);
        const __e20 = document.createElement("sui-skeleton");
        __e20.setAttribute("shape", "line");
        __e20.setAttribute("size", "md");
        __e18.appendChild(__e20);
        __e14.appendChild(__e18);
        __frag12.appendChild(__e14);
        return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
          for (const c of __c13)
            try {
              c();
            } catch (e) {}
        } };
      })(__childScope11);
    }
  });
  const __renderWhileLoading_0 = () => {
    const __frag7 = document.createDocumentFragment();
    const __c8 = [];
    const __e9 = document.createElement("div");
    __e9.setAttribute("class", "s-skel");
    __e9.setAttribute("aria-busy", "true");
    __e9.setAttribute("aria-label", "Loading settings");
    const __forAnchor10 = document.createComment("for");
    __e9.appendChild(__forAnchor10);
    __c8.push(__wireBoundary_b1(__forAnchor10));
    __frag7.appendChild(__e9);
    return { nodes: Array.from(__frag7.childNodes), cleanup: () => {
      for (const c of __c8)
        try {
          c();
        } catch (e) {}
    } };
  };
  const __renderWhileError_0 = (err) => {
    const __childScope22 = Object.create($scope);
    __childScope22.err = err;
    return (($scope2) => {
      const __frag23 = document.createDocumentFragment();
      const __c24 = [];
      const __e25 = document.createElement("sui-error-state");
      __e25.setAttribute("variant", "default");
      __e25.setAttribute("size", "md");
      __e25.setAttribute("title", "Couldn't load settings");
      const __t26 = document.createTextNode("");
      __e25.appendChild(__t26);
      __c24.push(bindText(__t26, () => $scope2.err.message));
      __frag23.appendChild(__e25);
      return { nodes: Array.from(__frag23.childNodes), cleanup: () => {
        for (const c of __c24)
          try {
            c();
          } catch (e) {}
      } };
    })(__childScope22);
  };
  const __wireBoundary_b10 = (anchor) => bindFor({
    anchor,
    items: () => $scope.settings().value.providers,
    key: (p, __i) => {
      const __child = Object.create($scope);
      __child.p = p;
      __child.__i = __i;
      return (($scope2) => p.id)(__child);
    },
    render: (p, __i) => {
      const __childScope61 = Object.create($scope);
      __childScope61.p = p;
      __childScope61.__i = __i;
      __childScope61.__v = signal2(0);
      return (($scope2) => {
        const __frag62 = document.createDocumentFragment();
        const __c63 = [];
        const __e64 = document.createElement("option");
        {
          const __av65 = $scope2.p.id;
          if (__av65 == null || __av65 === false || __av65 === "")
            __e64.removeAttribute("value");
          else
            __e64.setAttribute("value", __av65 === true ? "" : String(__av65));
        }
        const __t66 = document.createTextNode("");
        __e64.appendChild(__t66);
        __t66.textContent = $scope2.p.id;
        __frag62.appendChild(__e64);
        return { nodes: Array.from(__frag62.childNodes), cleanup: () => {
          for (const c of __c63)
            try {
              c();
            } catch (e) {}
        }, update: (__new67, __newI68) => {
          $scope2.p = __new67;
          $scope2.__i = __newI68;
          {
            const __av65 = $scope2.p.id;
            if (__av65 == null || __av65 === false || __av65 === "")
              __e64.removeAttribute("value");
            else
              __e64.setAttribute("value", __av65 === true ? "" : String(__av65));
          }
          __t66.textContent = $scope2.p.id;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope61);
    }
  });
  const __wireBoundary_b20 = (anchor) => bindFor({
    anchor,
    items: () => $scope.settings().value.providers,
    key: (p, __i) => {
      const __child = Object.create($scope);
      __child.p = p;
      __child.__i = __i;
      return (($scope2) => p.id)(__child);
    },
    render: (p, __i) => {
      const __childScope88 = Object.create($scope);
      __childScope88.p = p;
      __childScope88.__i = __i;
      __childScope88.__v = signal2(0);
      return (($scope2) => {
        const __wireBoundary_b25 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.p.version)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag100 = document.createDocumentFragment();
              const __c101 = [];
              const __e102 = document.createElement("span");
              __e102.setAttribute("class", "mono");
              const __t103 = document.createTextNode("v");
              __e102.appendChild(__t103);
              const __t104 = document.createTextNode("");
              __e102.appendChild(__t104);
              __t104.textContent = $scope2.p.version;
              const __t105 = document.createTextNode("");
              __e102.appendChild(__t105);
              __c101.push(bindText(__t105, () => (void $scope2.__v(), $scope2.p.label ? " (" + $scope2.p.label + ")" : "")));
              __frag100.appendChild(__e102);
              return { nodes: Array.from(__frag100.childNodes), cleanup: () => {
                for (const c of __c101)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag89 = document.createDocumentFragment();
        const __c90 = [];
        const __e91 = document.createElement("sui-list-tile");
        __e91.setAttribute("density", "compact");
        __e91.setAttribute("class", "s-prov-tile");
        const __e92 = document.createElement("sui-provider-blob");
        __e92.setAttribute("slot", "leading");
        {
          const __av93 = $scope2.p.id;
          if (__av93 == null || __av93 === false || __av93 === "")
            __e92.removeAttribute("provider");
          else
            __e92.setAttribute("provider", __av93 === true ? "" : String(__av93));
        }
        __e92.setAttribute("size", "sm");
        __e92.setAttribute("variant", "glyph");
        __c90.push(bindAttr(__e92, "style", () => (void $scope2.__v(), $scope2.p.available ? "" : "opacity:0.45;filter:grayscale(1)")));
        __e91.appendChild(__e92);
        const __e94 = document.createElement("span");
        __e94.setAttribute("class", "s-prov-name");
        const __t95 = document.createTextNode("");
        __e94.appendChild(__t95);
        __t95.textContent = $scope2.p.id;
        __e91.appendChild(__e94);
        const __e96 = document.createElement("span");
        __e96.setAttribute("slot", "trailing");
        __e96.setAttribute("class", "s-prov-right");
        const __e97 = document.createElement("span");
        const __t98 = document.createTextNode("");
        __e97.appendChild(__t98);
        __t98.textContent = $scope2.p.available ? "installed" : "not found";
        __e96.appendChild(__e97);
        const __ifAnchor99 = document.createComment("if");
        __e96.appendChild(__ifAnchor99);
        __c90.push(__wireBoundary_b25(__ifAnchor99));
        __e91.appendChild(__e96);
        __frag89.appendChild(__e91);
        return { nodes: Array.from(__frag89.childNodes), cleanup: () => {
          for (const c of __c90)
            try {
              c();
            } catch (e) {}
        }, update: (__new106, __newI107) => {
          $scope2.p = __new106;
          $scope2.__i = __newI107;
          {
            const __av93 = $scope2.p.id;
            if (__av93 == null || __av93 === false || __av93 === "")
              __e92.removeAttribute("provider");
            else
              __e92.setAttribute("provider", __av93 === true ? "" : String(__av93));
          }
          __t95.textContent = $scope2.p.id;
          __t98.textContent = $scope2.p.available ? "installed" : "not found";
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope88);
    }
  });
  const __wireBoundary_b19 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.settings().value.providers.length > 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag82 = document.createDocumentFragment();
        const __c83 = [];
        const __e84 = document.createElement("div");
        __e84.setAttribute("class", "s-provider-list");
        const __e85 = document.createElement("div");
        __e85.setAttribute("class", "s-provider-label");
        const __t86 = document.createTextNode("Installed providers");
        __e85.appendChild(__t86);
        __e84.appendChild(__e85);
        const __forAnchor87 = document.createComment("for");
        __e84.appendChild(__forAnchor87);
        __c83.push(__wireBoundary_b20(__forAnchor87));
        __frag82.appendChild(__e84);
        return { nodes: Array.from(__frag82.childNodes), cleanup: () => {
          for (const c of __c83)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b33 = (anchor) => bindFor({
    anchor,
    items: () => $scope.settings().value.terminal.options,
    key: (opt, __i) => {
      const __child = Object.create($scope);
      __child.opt = opt;
      __child.__i = __i;
      return (($scope2) => opt.value)(__child);
    },
    render: (opt, __i) => {
      const __childScope127 = Object.create($scope);
      __childScope127.opt = opt;
      __childScope127.__i = __i;
      __childScope127.__v = signal2(0);
      return (($scope2) => {
        const __frag128 = document.createDocumentFragment();
        const __c129 = [];
        const __e130 = document.createElement("option");
        {
          const __av131 = $scope2.opt.value;
          if (__av131 == null || __av131 === false || __av131 === "")
            __e130.removeAttribute("value");
          else
            __e130.setAttribute("value", __av131 === true ? "" : String(__av131));
        }
        const __t132 = document.createTextNode("");
        __e130.appendChild(__t132);
        __t132.textContent = $scope2.opt.label;
        __frag128.appendChild(__e130);
        return { nodes: Array.from(__frag128.childNodes), cleanup: () => {
          for (const c of __c129)
            try {
              c();
            } catch (e) {}
        }, update: (__new133, __newI134) => {
          $scope2.opt = __new133;
          $scope2.__i = __newI134;
          {
            const __av131 = $scope2.opt.value;
            if (__av131 == null || __av131 === false || __av131 === "")
              __e130.removeAttribute("value");
            else
              __e130.setAttribute("value", __av131 === true ? "" : String(__av131));
          }
          __t132.textContent = $scope2.opt.label;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope127);
    }
  });
  const __wireBoundary_b4 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.settings().kind === "ok")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag29 = document.createDocumentFragment();
        const __c30 = [];
        const __e31 = document.createElement("div");
        __e31.setAttribute("class", "s-sections");
        const __e32 = document.createElement("sui-card");
        __e32.setAttribute("padding", "none");
        const __e33 = document.createElement("div");
        __e33.setAttribute("slot", "header");
        __e33.setAttribute("class", "s-section-head");
        const __e34 = document.createElement("h2");
        __e34.setAttribute("class", "s-section-title");
        const __e35 = document.createElement("sui-icon");
        __e35.setAttribute("name", "eye");
        __e34.appendChild(__e35);
        const __t36 = document.createTextNode(`Theme
            `);
        __e34.appendChild(__t36);
        __e33.appendChild(__e34);
        __e32.appendChild(__e33);
        const __e37 = document.createElement("div");
        __e37.setAttribute("class", "s-section-body");
        const __e38 = document.createElement("sui-empty-state");
        __e38.setAttribute("icon", "palette");
        __e38.setAttribute("tone", "muted");
        __e38.setAttribute("size", "sm");
        __e38.setAttribute("style", "padding: var(--space-4) 0");
        const __e39 = document.createElement("span");
        __e39.setAttribute("slot", "title");
        const __t40 = document.createTextNode("Theme lives in the top bar");
        __e39.appendChild(__t40);
        __e38.appendChild(__e39);
        const __t41 = document.createTextNode(`
              Open the palette icon in the top-right to pick a theme. Every
              Ship theme bundles a full aesthetic — palette, motion, radii,
              shadows, typography — not just color.
            `);
        __e38.appendChild(__t41);
        __e37.appendChild(__e38);
        __e32.appendChild(__e37);
        __e31.appendChild(__e32);
        const __e42 = document.createElement("sui-card");
        __e42.setAttribute("padding", "none");
        const __e43 = document.createElement("div");
        __e43.setAttribute("slot", "header");
        __e43.setAttribute("class", "s-section-head");
        const __e44 = document.createElement("h2");
        __e44.setAttribute("class", "s-section-title");
        const __e45 = document.createElement("sui-icon");
        __e45.setAttribute("name", "cpu");
        __e44.appendChild(__e45);
        const __t46 = document.createTextNode(`Providers
            `);
        __e44.appendChild(__t46);
        __e43.appendChild(__e44);
        __e42.appendChild(__e43);
        const __e47 = document.createElement("div");
        __e47.setAttribute("class", "s-section-body");
        const __e48 = document.createElement("sui-list-tile");
        __e48.setAttribute("density", "compact");
        const __e49 = document.createElement("span");
        __e49.setAttribute("class", "s-row-label");
        __e49.setAttribute("id", "s-label-provider");
        const __t50 = document.createTextNode("Default provider");
        __e49.appendChild(__t50);
        __e48.appendChild(__e49);
        const __e51 = document.createElement("span");
        __e51.setAttribute("slot", "meta");
        const __t52 = document.createTextNode("Used when no provider is specified");
        __e51.appendChild(__t52);
        __e48.appendChild(__e51);
        const __e53 = document.createElement("span");
        __e53.setAttribute("slot", "trailing");
        __e53.setAttribute("class", "s-row-trailing");
        const __e54 = document.createElement("form");
        __c30.push(wireFormMutation(__e54, "ship.settings.update", $scope));
        __c30.push(on(__e54, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedProvider);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e55 = document.createElement("input");
        __e55.setAttribute("type", "hidden");
        __e55.setAttribute("name", "key");
        __e55.setAttribute("value", "defaults.provider");
        __e54.appendChild(__e55);
        const __e56 = document.createElement("sui-badge");
        __e56.setAttribute("tone", "success");
        __e56.setAttribute("variant", "soft");
        __e56.setAttribute("size", "sm");
        __e56.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e56, "visible", () => $scope.savedProvider()));
        const __e57 = document.createElement("sui-icon");
        __e57.setAttribute("slot", "leading");
        __e57.setAttribute("name", "check");
        __e57.setAttribute("size", "sm");
        __e56.appendChild(__e57);
        const __t58 = document.createTextNode(`Saved
                  `);
        __e56.appendChild(__t58);
        __e54.appendChild(__e56);
        const __e59 = document.createElement("sui-select");
        __e59.setAttribute("name", "value");
        __e59.setAttribute("size", "sm");
        __e59.setAttribute("aria-labelledby", "s-label-provider");
        __c30.push(bindProp(__e59, "value", () => $scope.settings().value.default_provider));
        __c30.push(on(__e59, "sui-change", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __forAnchor60 = document.createComment("for");
        __e59.appendChild(__forAnchor60);
        __c30.push(__wireBoundary_b10(__forAnchor60));
        __e54.appendChild(__e59);
        __e53.appendChild(__e54);
        __e48.appendChild(__e53);
        __e47.appendChild(__e48);
        const __e69 = document.createElement("sui-list-tile");
        __e69.setAttribute("density", "compact");
        const __e70 = document.createElement("span");
        __e70.setAttribute("class", "s-row-label");
        __e70.setAttribute("id", "s-label-model");
        const __t71 = document.createTextNode("Default model");
        __e70.appendChild(__t71);
        __e69.appendChild(__e70);
        const __e72 = document.createElement("span");
        __e72.setAttribute("slot", "meta");
        const __t73 = document.createTextNode("Model name passed to the provider");
        __e72.appendChild(__t73);
        __e69.appendChild(__e72);
        const __e74 = document.createElement("span");
        __e74.setAttribute("slot", "trailing");
        __e74.setAttribute("class", "s-row-trailing");
        const __e75 = document.createElement("form");
        __c30.push(wireFormMutation(__e75, "ship.settings.update", $scope));
        __c30.push(on(__e75, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedModel);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e76 = document.createElement("input");
        __e76.setAttribute("type", "hidden");
        __e76.setAttribute("name", "key");
        __e76.setAttribute("value", "defaults.model");
        __e75.appendChild(__e76);
        const __e77 = document.createElement("sui-badge");
        __e77.setAttribute("tone", "success");
        __e77.setAttribute("variant", "soft");
        __e77.setAttribute("size", "sm");
        __e77.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e77, "visible", () => $scope.savedModel()));
        const __e78 = document.createElement("sui-icon");
        __e78.setAttribute("slot", "leading");
        __e78.setAttribute("name", "check");
        __e78.setAttribute("size", "sm");
        __e77.appendChild(__e78);
        const __t79 = document.createTextNode(`Saved
                  `);
        __e77.appendChild(__t79);
        __e75.appendChild(__e77);
        const __e80 = document.createElement("sui-input");
        __e80.setAttribute("name", "value");
        __e80.setAttribute("size", "sm");
        __e80.setAttribute("aria-labelledby", "s-label-model");
        __c30.push(bindProp(__e80, "value", () => $scope.settings().value.default_model));
        __e80.setAttribute("placeholder", "e.g. sonnet");
        __c30.push(on(__e80, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e80, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e75.appendChild(__e80);
        __e74.appendChild(__e75);
        __e69.appendChild(__e74);
        __e47.appendChild(__e69);
        const __ifAnchor81 = document.createComment("if");
        __e47.appendChild(__ifAnchor81);
        __c30.push(__wireBoundary_b19(__ifAnchor81));
        __e42.appendChild(__e47);
        __e31.appendChild(__e42);
        const __e108 = document.createElement("sui-card");
        __e108.setAttribute("padding", "none");
        const __e109 = document.createElement("div");
        __e109.setAttribute("slot", "header");
        __e109.setAttribute("class", "s-section-head");
        const __e110 = document.createElement("h2");
        __e110.setAttribute("class", "s-section-title");
        const __e111 = document.createElement("sui-icon");
        __e111.setAttribute("name", "terminal");
        __e110.appendChild(__e111);
        const __t112 = document.createTextNode(`Terminal
            `);
        __e110.appendChild(__t112);
        __e109.appendChild(__e110);
        __e108.appendChild(__e109);
        const __e113 = document.createElement("div");
        __e113.setAttribute("class", "s-section-body");
        const __e114 = document.createElement("sui-list-tile");
        __e114.setAttribute("density", "compact");
        const __e115 = document.createElement("span");
        __e115.setAttribute("class", "s-row-label");
        __e115.setAttribute("id", "s-label-term");
        const __t116 = document.createTextNode("Terminal program");
        __e115.appendChild(__t116);
        __e114.appendChild(__e115);
        const __e117 = document.createElement("span");
        __e117.setAttribute("slot", "meta");
        const __t118 = document.createTextNode("Used when launching sessions");
        __e117.appendChild(__t118);
        __e114.appendChild(__e117);
        const __e119 = document.createElement("span");
        __e119.setAttribute("slot", "trailing");
        __e119.setAttribute("class", "s-row-trailing");
        const __e120 = document.createElement("form");
        __c30.push(wireFormMutation(__e120, "ship.settings.update", $scope));
        __c30.push(on(__e120, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedTerminal);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e121 = document.createElement("input");
        __e121.setAttribute("type", "hidden");
        __e121.setAttribute("name", "key");
        __e121.setAttribute("value", "terminal.program");
        __e120.appendChild(__e121);
        const __e122 = document.createElement("sui-badge");
        __e122.setAttribute("tone", "success");
        __e122.setAttribute("variant", "soft");
        __e122.setAttribute("size", "sm");
        __e122.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e122, "visible", () => $scope.savedTerminal()));
        const __e123 = document.createElement("sui-icon");
        __e123.setAttribute("slot", "leading");
        __e123.setAttribute("name", "check");
        __e123.setAttribute("size", "sm");
        __e122.appendChild(__e123);
        const __t124 = document.createTextNode(`Saved
                  `);
        __e122.appendChild(__t124);
        __e120.appendChild(__e122);
        const __e125 = document.createElement("sui-select");
        __e125.setAttribute("name", "value");
        __e125.setAttribute("size", "sm");
        __e125.setAttribute("aria-labelledby", "s-label-term");
        __c30.push(bindProp(__e125, "value", () => $scope.settings().value.terminal.program));
        __c30.push(on(__e125, "sui-change", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __forAnchor126 = document.createComment("for");
        __e125.appendChild(__forAnchor126);
        __c30.push(__wireBoundary_b33(__forAnchor126));
        __e120.appendChild(__e125);
        __e119.appendChild(__e120);
        __e114.appendChild(__e119);
        __e113.appendChild(__e114);
        const __e135 = document.createElement("sui-list-tile");
        __e135.setAttribute("density", "compact");
        const __e136 = document.createElement("span");
        __e136.setAttribute("class", "s-row-label");
        __e136.setAttribute("id", "s-label-worktree");
        const __t137 = document.createTextNode("Worktree base directory");
        __e136.appendChild(__t137);
        __e135.appendChild(__e136);
        const __e138 = document.createElement("span");
        __e138.setAttribute("slot", "meta");
        const __t139 = document.createTextNode("Root for git worktrees");
        __e138.appendChild(__t139);
        __e135.appendChild(__e138);
        const __e140 = document.createElement("span");
        __e140.setAttribute("slot", "trailing");
        __e140.setAttribute("class", "s-row-trailing");
        const __e141 = document.createElement("form");
        __c30.push(wireFormMutation(__e141, "ship.settings.update", $scope));
        __c30.push(on(__e141, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedWorktree);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e142 = document.createElement("input");
        __e142.setAttribute("type", "hidden");
        __e142.setAttribute("name", "key");
        __e142.setAttribute("value", "worktree.base_dir");
        __e141.appendChild(__e142);
        const __e143 = document.createElement("sui-badge");
        __e143.setAttribute("tone", "success");
        __e143.setAttribute("variant", "soft");
        __e143.setAttribute("size", "sm");
        __e143.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e143, "visible", () => $scope.savedWorktree()));
        const __e144 = document.createElement("sui-icon");
        __e144.setAttribute("slot", "leading");
        __e144.setAttribute("name", "check");
        __e144.setAttribute("size", "sm");
        __e143.appendChild(__e144);
        const __t145 = document.createTextNode(`Saved
                  `);
        __e143.appendChild(__t145);
        __e141.appendChild(__e143);
        const __e146 = document.createElement("sui-input");
        __e146.setAttribute("name", "value");
        __e146.setAttribute("size", "sm");
        __e146.setAttribute("aria-labelledby", "s-label-worktree");
        __c30.push(bindProp(__e146, "value", () => $scope.settings().value.terminal.worktree_base));
        __e146.setAttribute("placeholder", "/path/to/worktrees");
        __c30.push(on(__e146, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e146, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e141.appendChild(__e146);
        __e140.appendChild(__e141);
        __e135.appendChild(__e140);
        __e113.appendChild(__e135);
        __e108.appendChild(__e113);
        __e31.appendChild(__e108);
        const __e147 = document.createElement("sui-card");
        __e147.setAttribute("padding", "none");
        const __e148 = document.createElement("sui-collapsible");
        __c30.push(on(__e148, "sui-toggle", function() {
          const __h = () => $scope.advancedOpen.set(!$scope.advancedOpen());
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e149 = document.createElement("div");
        __e149.setAttribute("slot", "trigger");
        __e149.setAttribute("class", "s-collapse-trigger");
        const __e150 = document.createElement("h2");
        __e150.setAttribute("class", "s-section-title");
        const __e151 = document.createElement("sui-icon");
        __e151.setAttribute("name", "settings");
        __e150.appendChild(__e151);
        const __t152 = document.createTextNode(`Advanced
              `);
        __e150.appendChild(__t152);
        __e149.appendChild(__e150);
        const __e153 = document.createElement("sui-icon");
        __e153.setAttribute("name", "chevron-down");
        __e153.setAttribute("class", "s-chevron");
        __c30.push(bindClass(__e153, "open", () => $scope.advancedOpen()));
        __e149.appendChild(__e153);
        __e148.appendChild(__e149);
        const __e154 = document.createElement("div");
        __e154.setAttribute("class", "s-collapse-inner");
        const __e155 = document.createElement("div");
        __e155.setAttribute("class", "s-section-body");
        const __e156 = document.createElement("sui-list-tile");
        __e156.setAttribute("density", "compact");
        const __e157 = document.createElement("span");
        __e157.setAttribute("class", "s-row-label");
        const __t158 = document.createTextNode("Daemon status");
        __e157.appendChild(__t158);
        __e156.appendChild(__e157);
        const __e159 = document.createElement("span");
        __e159.setAttribute("slot", "trailing");
        __e159.setAttribute("class", "s-row-trailing");
        const __e160 = document.createElement("sui-status-dot");
        __c30.push(bindAttr(__e160, "tone", () => $scope.settings().kind === "ok" ? "success" : "danger"));
        __c30.push(bindAttr(__e160, "label", () => $scope.settings().kind === "ok" ? "Connected" : "Disconnected"));
        __e159.appendChild(__e160);
        const __e161 = document.createElement("span");
        __e161.setAttribute("class", "s-status-text");
        const __t162 = document.createTextNode("");
        __e161.appendChild(__t162);
        __c30.push(bindText(__t162, () => $scope.settings().kind === "ok" ? "Connected" : "Disconnected"));
        __e159.appendChild(__e161);
        __e156.appendChild(__e159);
        __e155.appendChild(__e156);
        const __e163 = document.createElement("sui-list-tile");
        __e163.setAttribute("density", "compact");
        const __e164 = document.createElement("span");
        __e164.setAttribute("class", "s-row-label");
        const __t165 = document.createTextNode("Daemon endpoint");
        __e164.appendChild(__t165);
        __e163.appendChild(__e164);
        const __e166 = document.createElement("span");
        __e166.setAttribute("slot", "trailing");
        __e166.setAttribute("class", "s-mono-val");
        const __t167 = document.createTextNode("");
        __e166.appendChild(__t167);
        __c30.push(bindText(__t167, () => $scope.settings().value.daemon_endpoint));
        __e163.appendChild(__e166);
        __e155.appendChild(__e163);
        const __e168 = document.createElement("sui-list-tile");
        __e168.setAttribute("density", "compact");
        const __e169 = document.createElement("span");
        __e169.setAttribute("class", "s-row-label");
        __e169.setAttribute("id", "s-label-registry");
        const __t170 = document.createTextNode("Registry endpoint");
        __e169.appendChild(__t170);
        __e168.appendChild(__e169);
        const __e171 = document.createElement("span");
        __e171.setAttribute("slot", "meta");
        const __t172 = document.createTextNode("Package registry URL");
        __e171.appendChild(__t172);
        __e168.appendChild(__e171);
        const __e173 = document.createElement("span");
        __e173.setAttribute("slot", "trailing");
        __e173.setAttribute("class", "s-row-trailing");
        const __e174 = document.createElement("form");
        __c30.push(wireFormMutation(__e174, "ship.settings.update", $scope));
        __c30.push(on(__e174, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedRegistry);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e175 = document.createElement("input");
        __e175.setAttribute("type", "hidden");
        __e175.setAttribute("name", "key");
        __e175.setAttribute("value", "registry.endpoint");
        __e174.appendChild(__e175);
        const __e176 = document.createElement("sui-badge");
        __e176.setAttribute("tone", "success");
        __e176.setAttribute("variant", "soft");
        __e176.setAttribute("size", "sm");
        __e176.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e176, "visible", () => $scope.savedRegistry()));
        const __e177 = document.createElement("sui-icon");
        __e177.setAttribute("slot", "leading");
        __e177.setAttribute("name", "check");
        __e177.setAttribute("size", "sm");
        __e176.appendChild(__e177);
        const __t178 = document.createTextNode(`Saved
                      `);
        __e176.appendChild(__t178);
        __e174.appendChild(__e176);
        const __e179 = document.createElement("sui-input");
        __e179.setAttribute("name", "value");
        __e179.setAttribute("size", "sm");
        __e179.setAttribute("aria-labelledby", "s-label-registry");
        __c30.push(bindProp(__e179, "value", () => $scope.settings().value.advanced_registry));
        __e179.setAttribute("placeholder", "https://registry.getship.dev");
        __c30.push(on(__e179, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e179, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e174.appendChild(__e179);
        __e173.appendChild(__e174);
        __e168.appendChild(__e173);
        __e155.appendChild(__e168);
        __e154.appendChild(__e155);
        __e148.appendChild(__e154);
        __e147.appendChild(__e148);
        __e31.appendChild(__e147);
        const __e180 = document.createElement("sui-card");
        __e180.setAttribute("padding", "none");
        const __e181 = document.createElement("div");
        __e181.setAttribute("slot", "header");
        __e181.setAttribute("class", "s-section-head");
        const __e182 = document.createElement("h2");
        __e182.setAttribute("class", "s-section-title");
        const __e183 = document.createElement("sui-icon");
        __e183.setAttribute("name", "info");
        __e182.appendChild(__e183);
        const __t184 = document.createTextNode(`About
            `);
        __e182.appendChild(__t184);
        __e181.appendChild(__e182);
        __e180.appendChild(__e181);
        const __e185 = document.createElement("div");
        __e185.setAttribute("class", "s-section-body");
        const __e186 = document.createElement("sui-list-tile");
        __e186.setAttribute("density", "compact");
        const __e187 = document.createElement("span");
        __e187.setAttribute("class", "s-row-label");
        const __t188 = document.createTextNode("Documentation");
        __e187.appendChild(__t188);
        __e186.appendChild(__e187);
        const __e189 = document.createElement("span");
        __e189.setAttribute("slot", "trailing");
        const __e190 = document.createElement("sui-link");
        __c30.push(bindProp(__e190, "href", () => $scope.settings().value.about.docs));
        __e190.setAttribute("external", "");
        const __t191 = document.createTextNode("getship.dev/docs");
        __e190.appendChild(__t191);
        __e189.appendChild(__e190);
        __e186.appendChild(__e189);
        __e185.appendChild(__e186);
        const __e192 = document.createElement("sui-list-tile");
        __e192.setAttribute("density", "compact");
        const __e193 = document.createElement("span");
        __e193.setAttribute("class", "s-row-label");
        const __t194 = document.createTextNode("Source");
        __e193.appendChild(__t194);
        __e192.appendChild(__e193);
        const __e195 = document.createElement("span");
        __e195.setAttribute("slot", "trailing");
        const __e196 = document.createElement("sui-link");
        __c30.push(bindProp(__e196, "href", () => $scope.settings().value.about.source));
        __e196.setAttribute("external", "");
        const __t197 = document.createTextNode("GitHub");
        __e196.appendChild(__t197);
        __e195.appendChild(__e196);
        __e192.appendChild(__e195);
        __e185.appendChild(__e192);
        __e180.appendChild(__e185);
        __e31.appendChild(__e180);
        __frag29.appendChild(__e31);
        return { nodes: Array.from(__frag29.childNodes), cleanup: () => {
          for (const c of __c30)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("div");
      __e0.setAttribute("class", "ship-view ship-view-settings");
      const __e1 = document.createElement("main");
      __e1.setAttribute("class", "settings-wrap");
      const __e2 = document.createElement("header");
      __e2.setAttribute("class", "settings-header");
      const __e3 = document.createElement("h1");
      const __t4 = document.createTextNode("Settings");
      __e3.appendChild(__t4);
      __e2.appendChild(__e3);
      const __e5 = document.createElement("p");
      const __t6 = document.createTextNode("CLI and studio configuration");
      __e5.appendChild(__t6);
      __e2.appendChild(__e5);
      __e1.appendChild(__e2);
      const __wlAnchor21 = document.createComment("while-loading");
      __e1.appendChild(__wlAnchor21);
      __cleanups.push(bindWhileLoading({ anchor: __wlAnchor21, signal: $scope.settings, render: __renderWhileLoading_0 }));
      const __weAnchor27 = document.createComment("while-error");
      __e1.appendChild(__weAnchor27);
      __cleanups.push(bindWhileError({ anchor: __weAnchor27, signal: $scope.settings, render: __renderWhileError_0 }));
      const __ifAnchor28 = document.createComment("if");
      __e1.appendChild(__ifAnchor28);
      __cleanups.push(__wireBoundary_b4(__ifAnchor28));
      __e0.appendChild(__e1);
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
      const __m = collectElementMarkers(root);
      if (__m["b0"])
        __cleanups.push(hydrateWhileLoading(__m["b0"], "b0", () => $scope.settings(), __renderWhileLoading_0));
      if (__m["b2"])
        __cleanups.push(hydrateWhileError(__m["b2"], "b2", () => $scope.settings(), __renderWhileError_0));
      if (__m["b4"]) {
        const __t = __m["b4"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b4(__anchor));
      }
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
