// views/settings/page.sui
import { bindAttr, bindClass, bindFor, bindIf, bindProp, bindText, bindWhileError, bindWhileLoading, data, live, on, signal as signal2, view, wireFormMutation } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateWhileError, hydrateWhileLoading } from "@ship/sui-dom";

// views/settings/controller.ts
import { controller, signal, derived, unwrapOr } from "@ship/sui-runtime";
import {
  accountDisplayName,
  accountInitials,
  accountPlanLabel,
  accountSignIn,
  accountSignOut,
  openAccountBilling,
  readAccountIdentity
} from "@ship/sdk-client";
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
  scope: ({ account }) => {
    const accountBusy = signal(false);
    const accountError = signal("");
    const accountIdentity = derived(() => readAccountIdentity(unwrapOr(account, null)));
    const accountName = derived(() => accountDisplayName(accountIdentity()));
    const accountInitialsSig = derived(() => accountInitials(accountIdentity()) || "··");
    const accountPlan = derived(() => accountPlanLabel(accountIdentity()));
    async function onAccountSignIn() {
      if (accountBusy())
        return;
      accountBusy.set(true);
      accountError.set("");
      try {
        await accountSignIn();
      } catch (e) {
        accountError.set(e instanceof Error ? e.message : String(e));
      } finally {
        accountBusy.set(false);
      }
    }
    async function onAccountSignOut() {
      if (accountBusy())
        return;
      accountBusy.set(true);
      accountError.set("");
      try {
        await accountSignOut();
      } catch (e) {
        accountError.set(e instanceof Error ? e.message : String(e));
      } finally {
        accountBusy.set(false);
      }
    }
    return {
      advancedOpen: signal(false),
      savedProvider: signal(false),
      savedModel: signal(false),
      savedTerminal: signal(false),
      savedWorktree: signal(false),
      savedRegistry: signal(false),
      savedHarnessDefault: signal(false),
      savedHarnessModel: signal(false),
      savedAutoResume: signal(false),
      savedWatcherDebounce: signal(false),
      savedGitLogLimit: signal(false),
      savedHooksMaxDepth: signal(false),
      flashSaved,
      ship: { settings: { update: settingsUpdate } },
      accountBusy,
      accountError,
      accountIdentity,
      accountName,
      accountInitials: accountInitialsSig,
      accountPlan,
      onAccountSignIn,
      onAccountSignOut,
      openBilling: openAccountBilling
    };
  },
  on: {
    rerunSetup: () => {
      try {
        localStorage.removeItem("ship-setup-completed");
        localStorage.removeItem("ship-setup-state");
      } catch {}
      window.dispatchEvent(new CustomEvent("ship:setup:rerun"));
      if (!window.location.pathname.startsWith("/setup")) {
        window.location.href = "/setup";
      }
    }
  }
});

// views/settings/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var page_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope.settings = live(() => ship.settings.state());
  $scope.account = live(() => ship.account.state());
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
        const __e14 = document.createElement("sui-stack");
        __e14.setAttribute("class", "s-skel-card");
        __e14.setAttribute("gap", "3");
        const __e15 = document.createElement("sui-cluster");
        __e15.setAttribute("gap", "2");
        __e15.setAttribute("align", "center");
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
        const __e18 = document.createElement("sui-stack");
        __e18.setAttribute("gap", "2");
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
    const __e9 = document.createElement("sui-stack");
    __e9.setAttribute("gap", "4");
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
  const __wireBoundary_b6 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.accountIdentity().me?.image)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag55 = document.createDocumentFragment();
        const __c56 = [];
        const __e57 = document.createElement("img");
        __e57.setAttribute("slot", "leading");
        __e57.setAttribute("class", "s-account-avatar");
        __c56.push(bindAttr(__e57, "src", () => $scope.accountIdentity().me?.image ?? ""));
        __e57.setAttribute("alt", "");
        __frag55.appendChild(__e57);
        return { nodes: Array.from(__frag55.childNodes), cleanup: () => {
          for (const c of __c56)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag58 = document.createDocumentFragment();
        const __c59 = [];
        const __e60 = document.createElement("span");
        __e60.setAttribute("slot", "leading");
        __e60.setAttribute("class", "s-account-avatar-fallback");
        const __t61 = document.createTextNode("");
        __e60.appendChild(__t61);
        __c59.push(bindText(__t61, () => $scope.accountInitials()));
        __frag58.appendChild(__e60);
        return { nodes: Array.from(__frag58.childNodes), cleanup: () => {
          for (const c of __c59)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b10 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.accountIdentity().me?.email)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag65 = document.createDocumentFragment();
        const __c66 = [];
        const __e67 = document.createElement("span");
        __e67.setAttribute("slot", "meta");
        const __t68 = document.createTextNode("");
        __e67.appendChild(__t68);
        __c66.push(bindText(__t68, () => $scope.accountIdentity().me?.email));
        __frag65.appendChild(__e67);
        return { nodes: Array.from(__frag65.childNodes), cleanup: () => {
          for (const c of __c66)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag69 = document.createDocumentFragment();
        const __c70 = [];
        const __e71 = document.createElement("span");
        __e71.setAttribute("slot", "meta");
        const __t72 = document.createTextNode("Signed in to Ship Cloud");
        __e71.appendChild(__t72);
        __frag69.appendChild(__e71);
        return { nodes: Array.from(__frag69.childNodes), cleanup: () => {
          for (const c of __c70)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b12 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.accountPlan())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag74 = document.createDocumentFragment();
        const __c75 = [];
        const __e76 = document.createElement("span");
        __e76.setAttribute("slot", "trailing");
        __e76.setAttribute("class", "s-row-trailing");
        const __e77 = document.createElement("sui-badge");
        __e77.setAttribute("tone", "info");
        __e77.setAttribute("variant", "soft");
        __e77.setAttribute("size", "sm");
        const __t78 = document.createTextNode("");
        __e77.appendChild(__t78);
        __c75.push(bindText(__t78, () => $scope.accountPlan()));
        __e76.appendChild(__e77);
        __frag74.appendChild(__e76);
        return { nodes: Array.from(__frag74.childNodes), cleanup: () => {
          for (const c of __c75)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b5 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.account().kind === "loading")
        return 0;
      if ($scope.account().kind === "error")
        return 1;
      if ($scope.accountIdentity().status === "logged_in")
        return 2;
      if ($scope.accountIdentity().status === "pending")
        return 3;
      return 4;
    },
    branches: [
      () => {
        const __frag39 = document.createDocumentFragment();
        const __c40 = [];
        const __e41 = document.createElement("sui-list-tile");
        __e41.setAttribute("density", "compact");
        const __e42 = document.createElement("span");
        __e42.setAttribute("class", "s-row-label muted");
        const __t43 = document.createTextNode("Loading account…");
        __e42.appendChild(__t43);
        __e41.appendChild(__e42);
        __frag39.appendChild(__e41);
        return { nodes: Array.from(__frag39.childNodes), cleanup: () => {
          for (const c of __c40)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag44 = document.createDocumentFragment();
        const __c45 = [];
        const __e46 = document.createElement("sui-list-tile");
        __e46.setAttribute("density", "compact");
        const __e47 = document.createElement("span");
        __e47.setAttribute("class", "s-row-label");
        const __t48 = document.createTextNode("Ship Cloud");
        __e47.appendChild(__t48);
        __e46.appendChild(__e47);
        const __e49 = document.createElement("span");
        __e49.setAttribute("slot", "meta");
        const __t50 = document.createTextNode("Daemon unreachable. Account features unavailable while offline.");
        __e49.appendChild(__t50);
        __e46.appendChild(__e49);
        __frag44.appendChild(__e46);
        return { nodes: Array.from(__frag44.childNodes), cleanup: () => {
          for (const c of __c45)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag51 = document.createDocumentFragment();
        const __c52 = [];
        const __e53 = document.createElement("sui-list-tile");
        __e53.setAttribute("density", "compact");
        const __ifAnchor54 = document.createComment("if");
        __e53.appendChild(__ifAnchor54);
        __c52.push(__wireBoundary_b6(__ifAnchor54));
        const __e62 = document.createElement("span");
        __e62.setAttribute("class", "s-row-label");
        const __t63 = document.createTextNode("");
        __e62.appendChild(__t63);
        __c52.push(bindText(__t63, () => $scope.accountName()));
        __e53.appendChild(__e62);
        const __ifAnchor64 = document.createComment("if");
        __e53.appendChild(__ifAnchor64);
        __c52.push(__wireBoundary_b10(__ifAnchor64));
        const __ifAnchor73 = document.createComment("if");
        __e53.appendChild(__ifAnchor73);
        __c52.push(__wireBoundary_b12(__ifAnchor73));
        __frag51.appendChild(__e53);
        const __e79 = document.createElement("sui-list-tile");
        __e79.setAttribute("density", "compact");
        const __e80 = document.createElement("span");
        __e80.setAttribute("class", "s-row-label");
        const __t81 = document.createTextNode("Billing");
        __e80.appendChild(__t81);
        __e79.appendChild(__e80);
        const __e82 = document.createElement("span");
        __e82.setAttribute("slot", "meta");
        const __t83 = document.createTextNode("Manage your plan and payment method");
        __e82.appendChild(__t83);
        __e79.appendChild(__e82);
        const __e84 = document.createElement("span");
        __e84.setAttribute("slot", "trailing");
        const __e85 = document.createElement("sui-button");
        __e85.setAttribute("variant", "outline");
        __e85.setAttribute("size", "sm");
        __c52.push(on(__e85, "click", function() {
          const __h = () => $scope.openBilling();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e86 = document.createElement("sui-icon");
        __e86.setAttribute("slot", "leading");
        __e86.setAttribute("name", "external-link");
        __e86.setAttribute("size", "sm");
        __e85.appendChild(__e86);
        const __t87 = document.createTextNode(`
                    Open billing
                  `);
        __e85.appendChild(__t87);
        __e84.appendChild(__e85);
        __e79.appendChild(__e84);
        __frag51.appendChild(__e79);
        const __e88 = document.createElement("sui-list-tile");
        __e88.setAttribute("density", "compact");
        const __e89 = document.createElement("span");
        __e89.setAttribute("class", "s-row-label");
        const __t90 = document.createTextNode("Session");
        __e89.appendChild(__t90);
        __e88.appendChild(__e89);
        const __e91 = document.createElement("span");
        __e91.setAttribute("slot", "meta");
        const __t92 = document.createTextNode("Sign out clears the API key stored by this daemon");
        __e91.appendChild(__t92);
        __e88.appendChild(__e91);
        const __e93 = document.createElement("span");
        __e93.setAttribute("slot", "trailing");
        const __e94 = document.createElement("sui-button");
        __e94.setAttribute("variant", "outline");
        __e94.setAttribute("tone", "destructive");
        __e94.setAttribute("size", "sm");
        __c52.push(bindAttr(__e94, "disabled", () => $scope.accountBusy()));
        __c52.push(on(__e94, "click", function() {
          const __h = $scope.onAccountSignOut;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t95 = document.createTextNode(`
                    Sign out
                  `);
        __e94.appendChild(__t95);
        __e93.appendChild(__e94);
        __e88.appendChild(__e93);
        __frag51.appendChild(__e88);
        return { nodes: Array.from(__frag51.childNodes), cleanup: () => {
          for (const c of __c52)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag96 = document.createDocumentFragment();
        const __c97 = [];
        const __e98 = document.createElement("sui-list-tile");
        __e98.setAttribute("density", "compact");
        const __e99 = document.createElement("span");
        __e99.setAttribute("slot", "leading");
        __e99.setAttribute("class", "s-account-avatar-fallback signed-out");
        const __e100 = document.createElement("sui-icon");
        __e100.setAttribute("name", "loader");
        __e100.setAttribute("size", "sm");
        __e99.appendChild(__e100);
        __e98.appendChild(__e99);
        const __e101 = document.createElement("span");
        __e101.setAttribute("class", "s-row-label");
        const __t102 = document.createTextNode("Completing sign-in…");
        __e101.appendChild(__t102);
        __e98.appendChild(__e101);
        const __e103 = document.createElement("span");
        __e103.setAttribute("slot", "meta");
        const __t104 = document.createTextNode("Finish the flow in your browser, then return here.");
        __e103.appendChild(__t104);
        __e98.appendChild(__e103);
        __frag96.appendChild(__e98);
        return { nodes: Array.from(__frag96.childNodes), cleanup: () => {
          for (const c of __c97)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag105 = document.createDocumentFragment();
        const __c106 = [];
        const __e107 = document.createElement("sui-list-tile");
        __e107.setAttribute("density", "compact");
        const __e108 = document.createElement("span");
        __e108.setAttribute("slot", "leading");
        __e108.setAttribute("class", "s-account-avatar-fallback signed-out");
        const __e109 = document.createElement("sui-icon");
        __e109.setAttribute("name", "user");
        __e109.setAttribute("size", "sm");
        __e108.appendChild(__e109);
        __e107.appendChild(__e108);
        const __e110 = document.createElement("span");
        __e110.setAttribute("class", "s-row-label");
        const __t111 = document.createTextNode("Not signed in");
        __e110.appendChild(__t111);
        __e107.appendChild(__e110);
        const __e112 = document.createElement("span");
        __e112.setAttribute("slot", "meta");
        const __t113 = document.createTextNode("Connect this workspace to Ship Cloud to unlock publish + sync.");
        __e112.appendChild(__t113);
        __e107.appendChild(__e112);
        const __e114 = document.createElement("span");
        __e114.setAttribute("slot", "trailing");
        const __e115 = document.createElement("sui-button");
        __e115.setAttribute("variant", "solid");
        __e115.setAttribute("tone", "primary");
        __e115.setAttribute("size", "sm");
        __c106.push(bindAttr(__e115, "disabled", () => $scope.accountBusy()));
        __c106.push(on(__e115, "click", function() {
          const __h = $scope.onAccountSignIn;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t116 = document.createTextNode(`
                    Sign in
                  `);
        __e115.appendChild(__t116);
        __e114.appendChild(__e115);
        __e107.appendChild(__e114);
        __frag105.appendChild(__e107);
        return { nodes: Array.from(__frag105.childNodes), cleanup: () => {
          for (const c of __c106)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b19 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.accountError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag118 = document.createDocumentFragment();
        const __c119 = [];
        const __e120 = document.createElement("sui-list-tile");
        __e120.setAttribute("density", "compact");
        const __e121 = document.createElement("span");
        __e121.setAttribute("class", "s-row-label err");
        const __t122 = document.createTextNode("");
        __e121.appendChild(__t122);
        __c119.push(bindText(__t122, () => $scope.accountError()));
        __e120.appendChild(__e121);
        __frag118.appendChild(__e120);
        return { nodes: Array.from(__frag118.childNodes), cleanup: () => {
          for (const c of __c119)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b26 = (anchor) => bindFor({
    anchor,
    items: () => $scope.settings().value.providers,
    key: (p, __i) => {
      const __child = Object.create($scope);
      __child.p = p;
      __child.__i = __i;
      return (($scope2) => p.id)(__child);
    },
    render: (p, __i) => {
      const __childScope156 = Object.create($scope);
      __childScope156.p = p;
      __childScope156.__i = __i;
      return (($scope2) => {
        const __frag157 = document.createDocumentFragment();
        const __c158 = [];
        const __e159 = document.createElement("option");
        {
          const __av160 = $scope2.p.id;
          if (__av160 == null || __av160 === false || __av160 === "")
            __e159.removeAttribute("value");
          else
            __e159.setAttribute("value", __av160 === true ? "" : String(__av160));
        }
        const __t161 = document.createTextNode("");
        __e159.appendChild(__t161);
        __t161.textContent = $scope2.p.id;
        __frag157.appendChild(__e159);
        return { nodes: Array.from(__frag157.childNodes), cleanup: () => {
          for (const c of __c158)
            try {
              c();
            } catch (e) {}
        }, update: (__new162, __newI163) => {
          const __old = $scope2.p;
          $scope2.p = __new162;
          $scope2.__i = __newI163;
          if (__old.id !== $scope2.p.id) {
            {
              const __av160 = $scope2.p.id;
              if (__av160 == null || __av160 === false || __av160 === "")
                __e159.removeAttribute("value");
              else
                __e159.setAttribute("value", __av160 === true ? "" : String(__av160));
            }
            __t161.textContent = $scope2.p.id;
          }
        } };
      })(__childScope156);
    }
  });
  const __wireBoundary_b36 = (anchor) => bindFor({
    anchor,
    items: () => $scope.settings().value.providers,
    key: (p, __i) => {
      const __child = Object.create($scope);
      __child.p = p;
      __child.__i = __i;
      return (($scope2) => p.id)(__child);
    },
    render: (p, __i) => {
      const __childScope183 = Object.create($scope);
      __childScope183.p = p;
      __childScope183.__i = __i;
      return (($scope2) => {
        __childScope183.__v_available = signal2(0);
        __childScope183.__v_label = signal2(0);
        const __wireBoundary_b41 = (anchor2) => bindIf({
          anchor: anchor2,
          test: () => {
            if ($scope2.p.version)
              return 0;
            return -1;
          },
          branches: [
            () => {
              const __frag195 = document.createDocumentFragment();
              const __c196 = [];
              const __e197 = document.createElement("span");
              __e197.setAttribute("class", "mono");
              const __t198 = document.createTextNode("v");
              __e197.appendChild(__t198);
              const __t199 = document.createTextNode("");
              __e197.appendChild(__t199);
              __t199.textContent = $scope2.p.version;
              const __t200 = document.createTextNode("");
              __e197.appendChild(__t200);
              __c196.push(bindText(__t200, () => (void $scope2.__v_label(), $scope2.p.label ? " (" + $scope2.p.label + ")" : "")));
              __frag195.appendChild(__e197);
              return { nodes: Array.from(__frag195.childNodes), cleanup: () => {
                for (const c of __c196)
                  try {
                    c();
                  } catch (e) {}
              } };
            }
          ]
        });
        const __frag184 = document.createDocumentFragment();
        const __c185 = [];
        const __e186 = document.createElement("sui-list-tile");
        __e186.setAttribute("density", "compact");
        __e186.setAttribute("class", "s-prov-tile");
        const __e187 = document.createElement("sui-provider-blob");
        __e187.setAttribute("slot", "leading");
        {
          const __av188 = $scope2.p.id;
          if (__av188 == null || __av188 === false || __av188 === "")
            __e187.removeAttribute("provider");
          else
            __e187.setAttribute("provider", __av188 === true ? "" : String(__av188));
        }
        __e187.setAttribute("size", "sm");
        __e187.setAttribute("variant", "glyph");
        __c185.push(bindAttr(__e187, "style", () => (void $scope2.__v_available(), $scope2.p.available ? "" : "opacity:0.45;filter:grayscale(1)")));
        __e186.appendChild(__e187);
        const __e189 = document.createElement("span");
        __e189.setAttribute("class", "s-prov-name");
        const __t190 = document.createTextNode("");
        __e189.appendChild(__t190);
        __t190.textContent = $scope2.p.id;
        __e186.appendChild(__e189);
        const __e191 = document.createElement("span");
        __e191.setAttribute("slot", "trailing");
        __e191.setAttribute("class", "s-prov-right");
        const __e192 = document.createElement("span");
        const __t193 = document.createTextNode("");
        __e192.appendChild(__t193);
        __t193.textContent = $scope2.p.available ? "installed" : "not found";
        __e191.appendChild(__e192);
        const __ifAnchor194 = document.createComment("if");
        __e191.appendChild(__ifAnchor194);
        __c185.push(__wireBoundary_b41(__ifAnchor194));
        __e186.appendChild(__e191);
        __frag184.appendChild(__e186);
        return { nodes: Array.from(__frag184.childNodes), cleanup: () => {
          for (const c of __c185)
            try {
              c();
            } catch (e) {}
        }, update: (__new201, __newI202) => {
          const __old = $scope2.p;
          $scope2.p = __new201;
          $scope2.__i = __newI202;
          if (__old.id !== $scope2.p.id) {
            {
              const __av188 = $scope2.p.id;
              if (__av188 == null || __av188 === false || __av188 === "")
                __e187.removeAttribute("provider");
              else
                __e187.setAttribute("provider", __av188 === true ? "" : String(__av188));
            }
            __t190.textContent = $scope2.p.id;
          }
          if (__old.available !== $scope2.p.available) {
            __t193.textContent = $scope2.p.available ? "installed" : "not found";
          }
          if (__old.available !== $scope2.p.available) {
            $scope2.__v_available.set((c) => c + 1);
          }
          if (__old.label !== $scope2.p.label) {
            $scope2.__v_label.set((c) => c + 1);
          }
        } };
      })(__childScope183);
    }
  });
  const __wireBoundary_b35 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.settings().value.providers.length > 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag177 = document.createDocumentFragment();
        const __c178 = [];
        const __e179 = document.createElement("div");
        __e179.setAttribute("class", "s-provider-list");
        const __e180 = document.createElement("div");
        __e180.setAttribute("class", "s-provider-label");
        const __t181 = document.createTextNode("Installed providers");
        __e180.appendChild(__t181);
        __e179.appendChild(__e180);
        const __forAnchor182 = document.createComment("for");
        __e179.appendChild(__forAnchor182);
        __c178.push(__wireBoundary_b36(__forAnchor182));
        __frag177.appendChild(__e179);
        return { nodes: Array.from(__frag177.childNodes), cleanup: () => {
          for (const c of __c178)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b49 = (anchor) => bindFor({
    anchor,
    items: () => $scope.settings().value.terminal.options,
    key: (opt, __i) => {
      const __child = Object.create($scope);
      __child.opt = opt;
      __child.__i = __i;
      return (($scope2) => opt.value)(__child);
    },
    render: (opt, __i) => {
      const __childScope222 = Object.create($scope);
      __childScope222.opt = opt;
      __childScope222.__i = __i;
      return (($scope2) => {
        const __frag223 = document.createDocumentFragment();
        const __c224 = [];
        const __e225 = document.createElement("option");
        {
          const __av226 = $scope2.opt.value;
          if (__av226 == null || __av226 === false || __av226 === "")
            __e225.removeAttribute("value");
          else
            __e225.setAttribute("value", __av226 === true ? "" : String(__av226));
        }
        const __t227 = document.createTextNode("");
        __e225.appendChild(__t227);
        __t227.textContent = $scope2.opt.label;
        __frag223.appendChild(__e225);
        return { nodes: Array.from(__frag223.childNodes), cleanup: () => {
          for (const c of __c224)
            try {
              c();
            } catch (e) {}
        }, update: (__new228, __newI229) => {
          const __old = $scope2.opt;
          $scope2.opt = __new228;
          $scope2.__i = __newI229;
          if (__old.value !== $scope2.opt.value) {
            {
              const __av226 = $scope2.opt.value;
              if (__av226 == null || __av226 === false || __av226 === "")
                __e225.removeAttribute("value");
              else
                __e225.setAttribute("value", __av226 === true ? "" : String(__av226));
            }
          }
          if (__old.label !== $scope2.opt.label) {
            __t227.textContent = $scope2.opt.label;
          }
        } };
      })(__childScope222);
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
        const __e31 = document.createElement("sui-stack");
        __e31.setAttribute("gap", "4");
        const __e32 = document.createElement("sui-card");
        __e32.setAttribute("id", "account");
        __e32.setAttribute("padding", "none");
        const __e33 = document.createElement("div");
        __e33.setAttribute("slot", "header");
        __e33.setAttribute("class", "s-section-head");
        const __e34 = document.createElement("h2");
        __e34.setAttribute("class", "s-section-title");
        const __e35 = document.createElement("sui-icon");
        __e35.setAttribute("name", "user");
        __e34.appendChild(__e35);
        const __t36 = document.createTextNode(`Account
            `);
        __e34.appendChild(__t36);
        __e33.appendChild(__e34);
        __e32.appendChild(__e33);
        const __e37 = document.createElement("div");
        __e37.setAttribute("class", "s-section-body s-account-body");
        const __ifAnchor38 = document.createComment("if");
        __e37.appendChild(__ifAnchor38);
        __c30.push(__wireBoundary_b5(__ifAnchor38));
        const __ifAnchor117 = document.createComment("if");
        __e37.appendChild(__ifAnchor117);
        __c30.push(__wireBoundary_b19(__ifAnchor117));
        __e32.appendChild(__e37);
        __e31.appendChild(__e32);
        const __e123 = document.createElement("sui-card");
        __e123.setAttribute("padding", "none");
        const __e124 = document.createElement("div");
        __e124.setAttribute("slot", "header");
        __e124.setAttribute("class", "s-section-head");
        const __e125 = document.createElement("h2");
        __e125.setAttribute("class", "s-section-title");
        const __e126 = document.createElement("sui-icon");
        __e126.setAttribute("name", "eye");
        __e125.appendChild(__e126);
        const __t127 = document.createTextNode(`Theme
            `);
        __e125.appendChild(__t127);
        __e124.appendChild(__e125);
        __e123.appendChild(__e124);
        const __e128 = document.createElement("div");
        __e128.setAttribute("class", "s-section-body");
        const __e129 = document.createElement("sui-list-tile");
        __e129.setAttribute("density", "compact");
        const __e130 = document.createElement("span");
        __e130.setAttribute("class", "s-row-label");
        const __t131 = document.createTextNode("Customize design tokens");
        __e130.appendChild(__t131);
        __e129.appendChild(__e130);
        const __e132 = document.createElement("span");
        __e132.setAttribute("slot", "meta");
        const __t133 = document.createTextNode("Tweak palette, surfaces, motion, typography — overrides ride on top of the active pack.");
        __e132.appendChild(__t133);
        __e129.appendChild(__e132);
        const __e134 = document.createElement("span");
        __e134.setAttribute("slot", "trailing");
        __e134.setAttribute("class", "s-row-trailing");
        const __e135 = document.createElement("sui-link");
        __e135.setAttribute("href", "/theme-studio");
        __e135.setAttribute("data-test", "settings-theme-studio-link");
        const __t136 = document.createTextNode(`
                  Open Theme Studio
                `);
        __e135.appendChild(__t136);
        __e134.appendChild(__e135);
        __e129.appendChild(__e134);
        __e128.appendChild(__e129);
        __e123.appendChild(__e128);
        __e31.appendChild(__e123);
        const __e137 = document.createElement("sui-card");
        __e137.setAttribute("padding", "none");
        const __e138 = document.createElement("div");
        __e138.setAttribute("slot", "header");
        __e138.setAttribute("class", "s-section-head");
        const __e139 = document.createElement("h2");
        __e139.setAttribute("class", "s-section-title");
        const __e140 = document.createElement("sui-icon");
        __e140.setAttribute("name", "cpu");
        __e139.appendChild(__e140);
        const __t141 = document.createTextNode(`Providers
            `);
        __e139.appendChild(__t141);
        __e138.appendChild(__e139);
        __e137.appendChild(__e138);
        const __e142 = document.createElement("div");
        __e142.setAttribute("class", "s-section-body");
        const __e143 = document.createElement("sui-list-tile");
        __e143.setAttribute("density", "compact");
        const __e144 = document.createElement("span");
        __e144.setAttribute("class", "s-row-label");
        __e144.setAttribute("id", "s-label-provider");
        const __t145 = document.createTextNode("Default provider");
        __e144.appendChild(__t145);
        __e143.appendChild(__e144);
        const __e146 = document.createElement("span");
        __e146.setAttribute("slot", "meta");
        const __t147 = document.createTextNode("Used when no provider is specified");
        __e146.appendChild(__t147);
        __e143.appendChild(__e146);
        const __e148 = document.createElement("span");
        __e148.setAttribute("slot", "trailing");
        __e148.setAttribute("class", "s-row-trailing");
        const __e149 = document.createElement("form");
        __c30.push(wireFormMutation(__e149, "ship.settings.update", $scope));
        __c30.push(on(__e149, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedProvider);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e150 = document.createElement("input");
        __e150.setAttribute("type", "hidden");
        __e150.setAttribute("name", "key");
        __e150.setAttribute("value", "defaults.provider");
        __e149.appendChild(__e150);
        const __e151 = document.createElement("sui-badge");
        __e151.setAttribute("tone", "success");
        __e151.setAttribute("variant", "soft");
        __e151.setAttribute("size", "sm");
        __e151.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e151, "visible", () => $scope.savedProvider()));
        const __e152 = document.createElement("sui-icon");
        __e152.setAttribute("slot", "leading");
        __e152.setAttribute("name", "check");
        __e152.setAttribute("size", "sm");
        __e151.appendChild(__e152);
        const __t153 = document.createTextNode(`Saved
                  `);
        __e151.appendChild(__t153);
        __e149.appendChild(__e151);
        const __e154 = document.createElement("sui-select");
        __e154.setAttribute("name", "value");
        __e154.setAttribute("size", "sm");
        __e154.setAttribute("aria-labelledby", "s-label-provider");
        __c30.push(bindProp(__e154, "value", () => $scope.settings().value.default_provider));
        __c30.push(on(__e154, "sui-change", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __forAnchor155 = document.createComment("for");
        __e154.appendChild(__forAnchor155);
        __c30.push(__wireBoundary_b26(__forAnchor155));
        __e149.appendChild(__e154);
        __e148.appendChild(__e149);
        __e143.appendChild(__e148);
        __e142.appendChild(__e143);
        const __e164 = document.createElement("sui-list-tile");
        __e164.setAttribute("density", "compact");
        const __e165 = document.createElement("span");
        __e165.setAttribute("class", "s-row-label");
        __e165.setAttribute("id", "s-label-model");
        const __t166 = document.createTextNode("Default model");
        __e165.appendChild(__t166);
        __e164.appendChild(__e165);
        const __e167 = document.createElement("span");
        __e167.setAttribute("slot", "meta");
        const __t168 = document.createTextNode("Model name passed to the provider");
        __e167.appendChild(__t168);
        __e164.appendChild(__e167);
        const __e169 = document.createElement("span");
        __e169.setAttribute("slot", "trailing");
        __e169.setAttribute("class", "s-row-trailing");
        const __e170 = document.createElement("form");
        __c30.push(wireFormMutation(__e170, "ship.settings.update", $scope));
        __c30.push(on(__e170, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedModel);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e171 = document.createElement("input");
        __e171.setAttribute("type", "hidden");
        __e171.setAttribute("name", "key");
        __e171.setAttribute("value", "defaults.model");
        __e170.appendChild(__e171);
        const __e172 = document.createElement("sui-badge");
        __e172.setAttribute("tone", "success");
        __e172.setAttribute("variant", "soft");
        __e172.setAttribute("size", "sm");
        __e172.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e172, "visible", () => $scope.savedModel()));
        const __e173 = document.createElement("sui-icon");
        __e173.setAttribute("slot", "leading");
        __e173.setAttribute("name", "check");
        __e173.setAttribute("size", "sm");
        __e172.appendChild(__e173);
        const __t174 = document.createTextNode(`Saved
                  `);
        __e172.appendChild(__t174);
        __e170.appendChild(__e172);
        const __e175 = document.createElement("sui-input");
        __e175.setAttribute("name", "value");
        __e175.setAttribute("size", "sm");
        __e175.setAttribute("aria-labelledby", "s-label-model");
        __c30.push(bindProp(__e175, "value", () => $scope.settings().value.default_model));
        __e175.setAttribute("placeholder", "e.g. sonnet");
        __c30.push(on(__e175, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e175, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e170.appendChild(__e175);
        __e169.appendChild(__e170);
        __e164.appendChild(__e169);
        __e142.appendChild(__e164);
        const __ifAnchor176 = document.createComment("if");
        __e142.appendChild(__ifAnchor176);
        __c30.push(__wireBoundary_b35(__ifAnchor176));
        __e137.appendChild(__e142);
        __e31.appendChild(__e137);
        const __e203 = document.createElement("sui-card");
        __e203.setAttribute("padding", "none");
        const __e204 = document.createElement("div");
        __e204.setAttribute("slot", "header");
        __e204.setAttribute("class", "s-section-head");
        const __e205 = document.createElement("h2");
        __e205.setAttribute("class", "s-section-title");
        const __e206 = document.createElement("sui-icon");
        __e206.setAttribute("name", "terminal");
        __e205.appendChild(__e206);
        const __t207 = document.createTextNode(`Terminal
            `);
        __e205.appendChild(__t207);
        __e204.appendChild(__e205);
        __e203.appendChild(__e204);
        const __e208 = document.createElement("div");
        __e208.setAttribute("class", "s-section-body");
        const __e209 = document.createElement("sui-list-tile");
        __e209.setAttribute("density", "compact");
        const __e210 = document.createElement("span");
        __e210.setAttribute("class", "s-row-label");
        __e210.setAttribute("id", "s-label-term");
        const __t211 = document.createTextNode("Terminal program");
        __e210.appendChild(__t211);
        __e209.appendChild(__e210);
        const __e212 = document.createElement("span");
        __e212.setAttribute("slot", "meta");
        const __t213 = document.createTextNode("Used when launching sessions");
        __e212.appendChild(__t213);
        __e209.appendChild(__e212);
        const __e214 = document.createElement("span");
        __e214.setAttribute("slot", "trailing");
        __e214.setAttribute("class", "s-row-trailing");
        const __e215 = document.createElement("form");
        __c30.push(wireFormMutation(__e215, "ship.settings.update", $scope));
        __c30.push(on(__e215, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedTerminal);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e216 = document.createElement("input");
        __e216.setAttribute("type", "hidden");
        __e216.setAttribute("name", "key");
        __e216.setAttribute("value", "terminal.program");
        __e215.appendChild(__e216);
        const __e217 = document.createElement("sui-badge");
        __e217.setAttribute("tone", "success");
        __e217.setAttribute("variant", "soft");
        __e217.setAttribute("size", "sm");
        __e217.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e217, "visible", () => $scope.savedTerminal()));
        const __e218 = document.createElement("sui-icon");
        __e218.setAttribute("slot", "leading");
        __e218.setAttribute("name", "check");
        __e218.setAttribute("size", "sm");
        __e217.appendChild(__e218);
        const __t219 = document.createTextNode(`Saved
                  `);
        __e217.appendChild(__t219);
        __e215.appendChild(__e217);
        const __e220 = document.createElement("sui-select");
        __e220.setAttribute("name", "value");
        __e220.setAttribute("size", "sm");
        __e220.setAttribute("aria-labelledby", "s-label-term");
        __c30.push(bindProp(__e220, "value", () => $scope.settings().value.terminal.program));
        __c30.push(on(__e220, "sui-change", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __forAnchor221 = document.createComment("for");
        __e220.appendChild(__forAnchor221);
        __c30.push(__wireBoundary_b49(__forAnchor221));
        __e215.appendChild(__e220);
        __e214.appendChild(__e215);
        __e209.appendChild(__e214);
        __e208.appendChild(__e209);
        const __e230 = document.createElement("sui-list-tile");
        __e230.setAttribute("density", "compact");
        const __e231 = document.createElement("span");
        __e231.setAttribute("class", "s-row-label");
        __e231.setAttribute("id", "s-label-worktree");
        const __t232 = document.createTextNode("Worktree base directory");
        __e231.appendChild(__t232);
        __e230.appendChild(__e231);
        const __e233 = document.createElement("span");
        __e233.setAttribute("slot", "meta");
        const __t234 = document.createTextNode("Root for git worktrees");
        __e233.appendChild(__t234);
        __e230.appendChild(__e233);
        const __e235 = document.createElement("span");
        __e235.setAttribute("slot", "trailing");
        __e235.setAttribute("class", "s-row-trailing");
        const __e236 = document.createElement("form");
        __c30.push(wireFormMutation(__e236, "ship.settings.update", $scope));
        __c30.push(on(__e236, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedWorktree);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e237 = document.createElement("input");
        __e237.setAttribute("type", "hidden");
        __e237.setAttribute("name", "key");
        __e237.setAttribute("value", "worktree.base_dir");
        __e236.appendChild(__e237);
        const __e238 = document.createElement("sui-badge");
        __e238.setAttribute("tone", "success");
        __e238.setAttribute("variant", "soft");
        __e238.setAttribute("size", "sm");
        __e238.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e238, "visible", () => $scope.savedWorktree()));
        const __e239 = document.createElement("sui-icon");
        __e239.setAttribute("slot", "leading");
        __e239.setAttribute("name", "check");
        __e239.setAttribute("size", "sm");
        __e238.appendChild(__e239);
        const __t240 = document.createTextNode(`Saved
                  `);
        __e238.appendChild(__t240);
        __e236.appendChild(__e238);
        const __e241 = document.createElement("sui-input");
        __e241.setAttribute("name", "value");
        __e241.setAttribute("size", "sm");
        __e241.setAttribute("aria-labelledby", "s-label-worktree");
        __c30.push(bindProp(__e241, "value", () => $scope.settings().value.terminal.worktree_base));
        __e241.setAttribute("placeholder", "/path/to/worktrees");
        __c30.push(on(__e241, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e241, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e236.appendChild(__e241);
        __e235.appendChild(__e236);
        __e230.appendChild(__e235);
        __e208.appendChild(__e230);
        __e203.appendChild(__e208);
        __e31.appendChild(__e203);
        const __e242 = document.createElement("sui-card");
        __e242.setAttribute("padding", "none");
        const __e243 = document.createElement("div");
        __e243.setAttribute("slot", "header");
        __e243.setAttribute("class", "s-section-head");
        const __e244 = document.createElement("h2");
        __e244.setAttribute("class", "s-section-title");
        const __e245 = document.createElement("sui-icon");
        __e245.setAttribute("name", "cpu");
        __e244.appendChild(__e245);
        const __t246 = document.createTextNode(`Harness defaults
            `);
        __e244.appendChild(__t246);
        __e243.appendChild(__e244);
        __e242.appendChild(__e243);
        const __e247 = document.createElement("div");
        __e247.setAttribute("class", "s-section-body");
        const __e248 = document.createElement("sui-list-tile");
        __e248.setAttribute("density", "compact");
        const __e249 = document.createElement("span");
        __e249.setAttribute("class", "s-row-label");
        __e249.setAttribute("id", "s-label-harness-default");
        const __t250 = document.createTextNode("Default harness");
        __e249.appendChild(__t250);
        __e248.appendChild(__e249);
        const __e251 = document.createElement("span");
        __e251.setAttribute("slot", "meta");
        const __t252 = document.createTextNode("Harness used when `ship use --launch` runs with no `--harness` flag");
        __e251.appendChild(__t252);
        __e248.appendChild(__e251);
        const __e253 = document.createElement("span");
        __e253.setAttribute("slot", "trailing");
        __e253.setAttribute("class", "s-row-trailing");
        const __e254 = document.createElement("form");
        __c30.push(wireFormMutation(__e254, "ship.settings.update", $scope));
        __c30.push(on(__e254, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedHarnessDefault);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e255 = document.createElement("input");
        __e255.setAttribute("type", "hidden");
        __e255.setAttribute("name", "key");
        __e255.setAttribute("value", "harness.default");
        __e254.appendChild(__e255);
        const __e256 = document.createElement("sui-badge");
        __e256.setAttribute("tone", "success");
        __e256.setAttribute("variant", "soft");
        __e256.setAttribute("size", "sm");
        __e256.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e256, "visible", () => $scope.savedHarnessDefault()));
        const __e257 = document.createElement("sui-icon");
        __e257.setAttribute("slot", "leading");
        __e257.setAttribute("name", "check");
        __e257.setAttribute("size", "sm");
        __e256.appendChild(__e257);
        const __t258 = document.createTextNode(`Saved
                  `);
        __e256.appendChild(__t258);
        __e254.appendChild(__e256);
        const __e259 = document.createElement("sui-input");
        __e259.setAttribute("name", "value");
        __e259.setAttribute("size", "sm");
        __e259.setAttribute("aria-labelledby", "s-label-harness-default");
        __c30.push(bindProp(__e259, "value", () => $scope.settings().value.harness_default));
        __e259.setAttribute("placeholder", "e.g. claude, codex, gemini");
        __c30.push(on(__e259, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e259, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e254.appendChild(__e259);
        __e253.appendChild(__e254);
        __e248.appendChild(__e253);
        __e247.appendChild(__e248);
        const __e260 = document.createElement("sui-list-tile");
        __e260.setAttribute("density", "compact");
        const __e261 = document.createElement("span");
        __e261.setAttribute("class", "s-row-label");
        __e261.setAttribute("id", "s-label-harness-default-model");
        const __t262 = document.createTextNode("Default harness model");
        __e261.appendChild(__t262);
        __e260.appendChild(__e261);
        const __e263 = document.createElement("span");
        __e263.setAttribute("slot", "meta");
        const __t264 = document.createTextNode("Model passed to the launched harness when no `--model` flag is set");
        __e263.appendChild(__t264);
        __e260.appendChild(__e263);
        const __e265 = document.createElement("span");
        __e265.setAttribute("slot", "trailing");
        __e265.setAttribute("class", "s-row-trailing");
        const __e266 = document.createElement("form");
        __c30.push(wireFormMutation(__e266, "ship.settings.update", $scope));
        __c30.push(on(__e266, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedHarnessModel);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e267 = document.createElement("input");
        __e267.setAttribute("type", "hidden");
        __e267.setAttribute("name", "key");
        __e267.setAttribute("value", "harness.default_model");
        __e266.appendChild(__e267);
        const __e268 = document.createElement("sui-badge");
        __e268.setAttribute("tone", "success");
        __e268.setAttribute("variant", "soft");
        __e268.setAttribute("size", "sm");
        __e268.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e268, "visible", () => $scope.savedHarnessModel()));
        const __e269 = document.createElement("sui-icon");
        __e269.setAttribute("slot", "leading");
        __e269.setAttribute("name", "check");
        __e269.setAttribute("size", "sm");
        __e268.appendChild(__e269);
        const __t270 = document.createTextNode(`Saved
                  `);
        __e268.appendChild(__t270);
        __e266.appendChild(__e268);
        const __e271 = document.createElement("sui-input");
        __e271.setAttribute("name", "value");
        __e271.setAttribute("size", "sm");
        __e271.setAttribute("aria-labelledby", "s-label-harness-default-model");
        __c30.push(bindProp(__e271, "value", () => $scope.settings().value.harness_default_model));
        __e271.setAttribute("placeholder", "e.g. sonnet, opus");
        __c30.push(on(__e271, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e271, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e266.appendChild(__e271);
        __e265.appendChild(__e266);
        __e260.appendChild(__e265);
        __e247.appendChild(__e260);
        __e242.appendChild(__e247);
        __e31.appendChild(__e242);
        const __e272 = document.createElement("sui-card");
        __e272.setAttribute("padding", "none");
        const __e273 = document.createElement("div");
        __e273.setAttribute("slot", "header");
        __e273.setAttribute("class", "s-section-head");
        const __e274 = document.createElement("h2");
        __e274.setAttribute("class", "s-section-title");
        const __e275 = document.createElement("sui-icon");
        __e275.setAttribute("name", "play");
        __e274.appendChild(__e275);
        const __t276 = document.createTextNode(`Sessions
            `);
        __e274.appendChild(__t276);
        __e273.appendChild(__e274);
        __e272.appendChild(__e273);
        const __e277 = document.createElement("div");
        __e277.setAttribute("class", "s-section-body");
        const __e278 = document.createElement("sui-list-tile");
        __e278.setAttribute("density", "compact");
        const __e279 = document.createElement("span");
        __e279.setAttribute("class", "s-row-label");
        __e279.setAttribute("id", "s-label-auto-resume");
        const __t280 = document.createTextNode("Auto-resume agent sessions");
        __e279.appendChild(__t280);
        __e278.appendChild(__e279);
        const __e281 = document.createElement("span");
        __e281.setAttribute("slot", "meta");
        const __t282 = document.createTextNode("When on, `ship use --launch` re-enters an existing (agent, path) session instead of spawning fresh");
        __e281.appendChild(__t282);
        __e278.appendChild(__e281);
        const __e283 = document.createElement("span");
        __e283.setAttribute("slot", "trailing");
        __e283.setAttribute("class", "s-row-trailing");
        const __e284 = document.createElement("form");
        __c30.push(wireFormMutation(__e284, "ship.settings.update", $scope));
        __c30.push(on(__e284, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedAutoResume);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e285 = document.createElement("input");
        __e285.setAttribute("type", "hidden");
        __e285.setAttribute("name", "key");
        __e285.setAttribute("value", "agent_session.auto_resume");
        __e284.appendChild(__e285);
        const __e286 = document.createElement("sui-badge");
        __e286.setAttribute("tone", "success");
        __e286.setAttribute("variant", "soft");
        __e286.setAttribute("size", "sm");
        __e286.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e286, "visible", () => $scope.savedAutoResume()));
        const __e287 = document.createElement("sui-icon");
        __e287.setAttribute("slot", "leading");
        __e287.setAttribute("name", "check");
        __e287.setAttribute("size", "sm");
        __e286.appendChild(__e287);
        const __t288 = document.createTextNode(`Saved
                  `);
        __e286.appendChild(__t288);
        __e284.appendChild(__e286);
        const __e289 = document.createElement("sui-switch");
        __e289.setAttribute("name", "value");
        __e289.setAttribute("size", "sm");
        __e289.setAttribute("aria-labelledby", "s-label-auto-resume");
        __c30.push(bindProp(__e289, "checked", () => $scope.settings().value.agent_session_auto_resume));
        __c30.push(on(__e289, "sui-change", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e284.appendChild(__e289);
        __e283.appendChild(__e284);
        __e278.appendChild(__e283);
        __e277.appendChild(__e278);
        __e272.appendChild(__e277);
        __e31.appendChild(__e272);
        const __e290 = document.createElement("sui-card");
        __e290.setAttribute("padding", "none");
        const __e291 = document.createElement("sui-collapsible");
        __c30.push(on(__e291, "sui-toggle", function() {
          const __h = () => $scope.advancedOpen.set(!$scope.advancedOpen());
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e292 = document.createElement("div");
        __e292.setAttribute("slot", "trigger");
        __e292.setAttribute("class", "s-collapse-trigger");
        const __e293 = document.createElement("h2");
        __e293.setAttribute("class", "s-section-title");
        const __e294 = document.createElement("sui-icon");
        __e294.setAttribute("name", "settings");
        __e293.appendChild(__e294);
        const __t295 = document.createTextNode(`Advanced
              `);
        __e293.appendChild(__t295);
        __e292.appendChild(__e293);
        const __e296 = document.createElement("sui-icon");
        __e296.setAttribute("name", "chevron-down");
        __e296.setAttribute("class", "s-chevron");
        __c30.push(bindClass(__e296, "open", () => $scope.advancedOpen()));
        __e292.appendChild(__e296);
        __e291.appendChild(__e292);
        const __e297 = document.createElement("div");
        __e297.setAttribute("class", "s-collapse-inner");
        const __e298 = document.createElement("div");
        __e298.setAttribute("class", "s-section-body");
        const __e299 = document.createElement("sui-list-tile");
        __e299.setAttribute("density", "compact");
        const __e300 = document.createElement("span");
        __e300.setAttribute("class", "s-row-label");
        const __t301 = document.createTextNode("Daemon status");
        __e300.appendChild(__t301);
        __e299.appendChild(__e300);
        const __e302 = document.createElement("span");
        __e302.setAttribute("slot", "trailing");
        __e302.setAttribute("class", "s-row-trailing");
        const __e303 = document.createElement("sui-status-dot");
        __c30.push(bindAttr(__e303, "tone", () => $scope.settings().kind === "ok" ? "success" : "destructive"));
        __c30.push(bindAttr(__e303, "label", () => $scope.settings().kind === "ok" ? "Connected" : "Disconnected"));
        __e302.appendChild(__e303);
        const __e304 = document.createElement("span");
        __e304.setAttribute("class", "s-status-text");
        const __t305 = document.createTextNode("");
        __e304.appendChild(__t305);
        __c30.push(bindText(__t305, () => $scope.settings().kind === "ok" ? "Connected" : "Disconnected"));
        __e302.appendChild(__e304);
        __e299.appendChild(__e302);
        __e298.appendChild(__e299);
        const __e306 = document.createElement("sui-list-tile");
        __e306.setAttribute("density", "compact");
        const __e307 = document.createElement("span");
        __e307.setAttribute("class", "s-row-label");
        const __t308 = document.createTextNode("Daemon endpoint");
        __e307.appendChild(__t308);
        __e306.appendChild(__e307);
        const __e309 = document.createElement("span");
        __e309.setAttribute("slot", "trailing");
        __e309.setAttribute("class", "s-mono-val");
        const __t310 = document.createTextNode("");
        __e309.appendChild(__t310);
        __c30.push(bindText(__t310, () => $scope.settings().value.daemon_endpoint));
        __e306.appendChild(__e309);
        __e298.appendChild(__e306);
        const __e311 = document.createElement("sui-list-tile");
        __e311.setAttribute("density", "compact");
        const __e312 = document.createElement("span");
        __e312.setAttribute("class", "s-row-label");
        __e312.setAttribute("id", "s-label-registry");
        const __t313 = document.createTextNode("Registry endpoint");
        __e312.appendChild(__t313);
        __e311.appendChild(__e312);
        const __e314 = document.createElement("span");
        __e314.setAttribute("slot", "meta");
        const __t315 = document.createTextNode("Package registry URL");
        __e314.appendChild(__t315);
        __e311.appendChild(__e314);
        const __e316 = document.createElement("span");
        __e316.setAttribute("slot", "trailing");
        __e316.setAttribute("class", "s-row-trailing");
        const __e317 = document.createElement("form");
        __c30.push(wireFormMutation(__e317, "ship.settings.update", $scope));
        __c30.push(on(__e317, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedRegistry);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e318 = document.createElement("input");
        __e318.setAttribute("type", "hidden");
        __e318.setAttribute("name", "key");
        __e318.setAttribute("value", "registry.endpoint");
        __e317.appendChild(__e318);
        const __e319 = document.createElement("sui-badge");
        __e319.setAttribute("tone", "success");
        __e319.setAttribute("variant", "soft");
        __e319.setAttribute("size", "sm");
        __e319.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e319, "visible", () => $scope.savedRegistry()));
        const __e320 = document.createElement("sui-icon");
        __e320.setAttribute("slot", "leading");
        __e320.setAttribute("name", "check");
        __e320.setAttribute("size", "sm");
        __e319.appendChild(__e320);
        const __t321 = document.createTextNode(`Saved
                      `);
        __e319.appendChild(__t321);
        __e317.appendChild(__e319);
        const __e322 = document.createElement("sui-input");
        __e322.setAttribute("name", "value");
        __e322.setAttribute("size", "sm");
        __e322.setAttribute("aria-labelledby", "s-label-registry");
        __c30.push(bindProp(__e322, "value", () => $scope.settings().value.advanced_registry));
        __e322.setAttribute("placeholder", "https://registry.getship.dev");
        __c30.push(on(__e322, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e322, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e317.appendChild(__e322);
        __e316.appendChild(__e317);
        __e311.appendChild(__e316);
        __e298.appendChild(__e311);
        const __e323 = document.createElement("sui-list-tile");
        __e323.setAttribute("density", "compact");
        const __e324 = document.createElement("span");
        __e324.setAttribute("class", "s-row-label");
        __e324.setAttribute("id", "s-label-watcher-debounce");
        const __t325 = document.createTextNode("Watcher debounce (ms)");
        __e324.appendChild(__t325);
        __e323.appendChild(__e324);
        const __e326 = document.createElement("span");
        __e326.setAttribute("slot", "meta");
        const __t327 = document.createTextNode("Debounce window for the declarative `.ship/` file watcher");
        __e326.appendChild(__t327);
        __e323.appendChild(__e326);
        const __e328 = document.createElement("span");
        __e328.setAttribute("slot", "trailing");
        __e328.setAttribute("class", "s-row-trailing");
        const __e329 = document.createElement("form");
        __c30.push(wireFormMutation(__e329, "ship.settings.update", $scope));
        __c30.push(on(__e329, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedWatcherDebounce);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e330 = document.createElement("input");
        __e330.setAttribute("type", "hidden");
        __e330.setAttribute("name", "key");
        __e330.setAttribute("value", "watcher.debounce_ms");
        __e329.appendChild(__e330);
        const __e331 = document.createElement("sui-badge");
        __e331.setAttribute("tone", "success");
        __e331.setAttribute("variant", "soft");
        __e331.setAttribute("size", "sm");
        __e331.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e331, "visible", () => $scope.savedWatcherDebounce()));
        const __e332 = document.createElement("sui-icon");
        __e332.setAttribute("slot", "leading");
        __e332.setAttribute("name", "check");
        __e332.setAttribute("size", "sm");
        __e331.appendChild(__e332);
        const __t333 = document.createTextNode(`Saved
                      `);
        __e331.appendChild(__t333);
        __e329.appendChild(__e331);
        const __e334 = document.createElement("sui-input");
        __e334.setAttribute("type", "number");
        __e334.setAttribute("name", "value");
        __e334.setAttribute("size", "sm");
        __e334.setAttribute("aria-labelledby", "s-label-watcher-debounce");
        __c30.push(bindProp(__e334, "value", () => $scope.settings().value.watcher_debounce_ms));
        __e334.setAttribute("placeholder", "200");
        __c30.push(on(__e334, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e334, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e329.appendChild(__e334);
        __e328.appendChild(__e329);
        __e323.appendChild(__e328);
        __e298.appendChild(__e323);
        const __e335 = document.createElement("sui-list-tile");
        __e335.setAttribute("density", "compact");
        const __e336 = document.createElement("span");
        __e336.setAttribute("class", "s-row-label");
        __e336.setAttribute("id", "s-label-git-log-limit");
        const __t337 = document.createTextNode("Git log limit");
        __e336.appendChild(__t337);
        __e335.appendChild(__e336);
        const __e338 = document.createElement("span");
        __e338.setAttribute("slot", "meta");
        const __t339 = document.createTextNode("Default row count for `git log` reads when callers omit a limit");
        __e338.appendChild(__t339);
        __e335.appendChild(__e338);
        const __e340 = document.createElement("span");
        __e340.setAttribute("slot", "trailing");
        __e340.setAttribute("class", "s-row-trailing");
        const __e341 = document.createElement("form");
        __c30.push(wireFormMutation(__e341, "ship.settings.update", $scope));
        __c30.push(on(__e341, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedGitLogLimit);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e342 = document.createElement("input");
        __e342.setAttribute("type", "hidden");
        __e342.setAttribute("name", "key");
        __e342.setAttribute("value", "git.log_limit");
        __e341.appendChild(__e342);
        const __e343 = document.createElement("sui-badge");
        __e343.setAttribute("tone", "success");
        __e343.setAttribute("variant", "soft");
        __e343.setAttribute("size", "sm");
        __e343.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e343, "visible", () => $scope.savedGitLogLimit()));
        const __e344 = document.createElement("sui-icon");
        __e344.setAttribute("slot", "leading");
        __e344.setAttribute("name", "check");
        __e344.setAttribute("size", "sm");
        __e343.appendChild(__e344);
        const __t345 = document.createTextNode(`Saved
                      `);
        __e343.appendChild(__t345);
        __e341.appendChild(__e343);
        const __e346 = document.createElement("sui-input");
        __e346.setAttribute("type", "number");
        __e346.setAttribute("name", "value");
        __e346.setAttribute("size", "sm");
        __e346.setAttribute("aria-labelledby", "s-label-git-log-limit");
        __c30.push(bindProp(__e346, "value", () => $scope.settings().value.git_log_limit));
        __e346.setAttribute("placeholder", "50");
        __c30.push(on(__e346, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e346, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e341.appendChild(__e346);
        __e340.appendChild(__e341);
        __e335.appendChild(__e340);
        __e298.appendChild(__e335);
        const __e347 = document.createElement("sui-list-tile");
        __e347.setAttribute("density", "compact");
        const __e348 = document.createElement("span");
        __e348.setAttribute("class", "s-row-label");
        __e348.setAttribute("id", "s-label-hooks-max-depth");
        const __t349 = document.createTextNode("Hooks max depth");
        __e348.appendChild(__t349);
        __e347.appendChild(__e348);
        const __e350 = document.createElement("span");
        __e350.setAttribute("slot", "meta");
        const __t351 = document.createTextNode("Max hook recursion depth before the engine drops further events");
        __e350.appendChild(__t351);
        __e347.appendChild(__e350);
        const __e352 = document.createElement("span");
        __e352.setAttribute("slot", "trailing");
        __e352.setAttribute("class", "s-row-trailing");
        const __e353 = document.createElement("form");
        __c30.push(wireFormMutation(__e353, "ship.settings.update", $scope));
        __c30.push(on(__e353, "success", function() {
          const __h = () => $scope.flashSaved($scope.savedHooksMaxDepth);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e354 = document.createElement("input");
        __e354.setAttribute("type", "hidden");
        __e354.setAttribute("name", "key");
        __e354.setAttribute("value", "hooks.max_depth");
        __e353.appendChild(__e354);
        const __e355 = document.createElement("sui-badge");
        __e355.setAttribute("tone", "success");
        __e355.setAttribute("variant", "soft");
        __e355.setAttribute("size", "sm");
        __e355.setAttribute("class", "s-save-badge");
        __c30.push(bindClass(__e355, "visible", () => $scope.savedHooksMaxDepth()));
        const __e356 = document.createElement("sui-icon");
        __e356.setAttribute("slot", "leading");
        __e356.setAttribute("name", "check");
        __e356.setAttribute("size", "sm");
        __e355.appendChild(__e356);
        const __t357 = document.createTextNode(`Saved
                      `);
        __e355.appendChild(__t357);
        __e353.appendChild(__e355);
        const __e358 = document.createElement("sui-input");
        __e358.setAttribute("type", "number");
        __e358.setAttribute("name", "value");
        __e358.setAttribute("size", "sm");
        __e358.setAttribute("aria-labelledby", "s-label-hooks-max-depth");
        __c30.push(bindProp(__e358, "value", () => $scope.settings().value.hooks_max_depth));
        __e358.setAttribute("placeholder", "3");
        __c30.push(on(__e358, "blur", function() {
          const __h = (e) => e.target.closest("form")?.requestSubmit();
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __c30.push(on(__e358, "keydown", function() {
          const __h = (e) => {
            if (e.key === "Enter")
              e.target.blur();
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e353.appendChild(__e358);
        __e352.appendChild(__e353);
        __e347.appendChild(__e352);
        __e298.appendChild(__e347);
        __e297.appendChild(__e298);
        __e291.appendChild(__e297);
        __e290.appendChild(__e291);
        __e31.appendChild(__e290);
        const __e359 = document.createElement("sui-card");
        __e359.setAttribute("padding", "none");
        const __e360 = document.createElement("div");
        __e360.setAttribute("slot", "header");
        __e360.setAttribute("class", "s-section-head");
        const __e361 = document.createElement("h2");
        __e361.setAttribute("class", "s-section-title");
        const __e362 = document.createElement("sui-icon");
        __e362.setAttribute("name", "info");
        __e361.appendChild(__e362);
        const __t363 = document.createTextNode(`About
            `);
        __e361.appendChild(__t363);
        __e360.appendChild(__e361);
        __e359.appendChild(__e360);
        const __e364 = document.createElement("div");
        __e364.setAttribute("class", "s-section-body");
        const __e365 = document.createElement("sui-list-tile");
        __e365.setAttribute("density", "compact");
        const __e366 = document.createElement("span");
        __e366.setAttribute("class", "s-row-label");
        const __t367 = document.createTextNode("Documentation");
        __e366.appendChild(__t367);
        __e365.appendChild(__e366);
        const __e368 = document.createElement("span");
        __e368.setAttribute("slot", "trailing");
        const __e369 = document.createElement("sui-link");
        __c30.push(bindProp(__e369, "href", () => $scope.settings().value.about.docs));
        __e369.setAttribute("external", "");
        const __t370 = document.createTextNode("getship.dev/docs");
        __e369.appendChild(__t370);
        __e368.appendChild(__e369);
        __e365.appendChild(__e368);
        __e364.appendChild(__e365);
        const __e371 = document.createElement("sui-list-tile");
        __e371.setAttribute("density", "compact");
        const __e372 = document.createElement("span");
        __e372.setAttribute("class", "s-row-label");
        const __t373 = document.createTextNode("Source");
        __e372.appendChild(__t373);
        __e371.appendChild(__e372);
        const __e374 = document.createElement("span");
        __e374.setAttribute("slot", "trailing");
        const __e375 = document.createElement("sui-link");
        __c30.push(bindProp(__e375, "href", () => $scope.settings().value.about.source));
        __e375.setAttribute("external", "");
        const __t376 = document.createTextNode("GitHub");
        __e375.appendChild(__t376);
        __e374.appendChild(__e375);
        __e371.appendChild(__e374);
        __e364.appendChild(__e371);
        const __e377 = document.createElement("sui-list-tile");
        __e377.setAttribute("density", "compact");
        const __e378 = document.createElement("span");
        __e378.setAttribute("class", "s-row-label");
        const __t379 = document.createTextNode("First-run setup");
        __e378.appendChild(__t379);
        __e377.appendChild(__e378);
        const __e380 = document.createElement("span");
        __e380.setAttribute("slot", "meta");
        const __t381 = document.createTextNode("Run the welcome wizard again");
        __e380.appendChild(__t381);
        __e377.appendChild(__e380);
        const __e382 = document.createElement("span");
        __e382.setAttribute("slot", "trailing");
        const __e383 = document.createElement("sui-button");
        __e383.setAttribute("variant", "outline");
        __e383.setAttribute("size", "sm");
        __e383.setAttribute("data-test", "settings-rerun-setup");
        __c30.push(on(__e383, "click", function() {
          const __h = $scope.rerunSetup;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t384 = document.createTextNode(`
                  Run setup again
                `);
        __e383.appendChild(__t384);
        __e382.appendChild(__e383);
        __e377.appendChild(__e382);
        __e364.appendChild(__e377);
        __e359.appendChild(__e364);
        __e31.appendChild(__e359);
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
}), { _compilerVersion: { cargo: "0.1.0", git: "01329411", dirty: true, builtAt: "2026-05-19T16:54:50Z" } });
export {
  page_default as default
};
