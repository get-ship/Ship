// views/setup/page.sui
import { bindAttr, bindChecked, bindFor, bindIf, bindProp, bindText, bindValue, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateAttr, hydrateOn, hydrateProp } from "@ship/sui-dom";

// views/setup/controller.ts
import { controller, signal } from "@ship/sui-runtime";
var STORAGE_KEY = "ship-setup-state";
var TOTAL_STEPS = 6;
var PROVIDERS = [
  { id: "claude", name: "Claude", sub: "Anthropic · claude" },
  { id: "codex", name: "Codex", sub: "OpenAI · codex" },
  { id: "gemini", name: "Gemini", sub: "Google · gemini" },
  { id: "opencode", name: "OpenCode", sub: "Open source · opencode" },
  { id: "cursor", name: "Cursor", sub: "Cursor · cursor-agent" }
];
function defaultProjectPath(name) {
  const trimmed = name.trim() || "<name>";
  return `~/Documents/ShipProjects/${trimmed}`;
}
function loadPersisted() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw)
      return {};
    const parsed = JSON.parse(raw);
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    return {};
  }
}
function savePersisted(state) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {}
}
function readEntry(s) {
  const sig = (k) => s[k]();
  return {
    step: sig("step"),
    identityName: sig("identityName"),
    provider: sig("provider"),
    providerValidated: sig("providerTestOk"),
    projectName: sig("projectName"),
    projectPath: sig("projectPath"),
    sampleApp: sig("sampleApp"),
    completed: false
  };
}
async function fetchInstalledProviders() {
  try {
    const res = await fetch("/api/v1/providers/installed", { credentials: "same-origin" });
    if (!res.ok)
      return [];
    const json = await res.json();
    const data = Array.isArray(json?.data) ? json.data : json;
    return Array.isArray(data?.providers) ? data.providers : [];
  } catch {
    return [];
  }
}
async function postProject(name, path) {
  try {
    const res = await fetch("/api/v1/projects", {
      method: "POST",
      credentials: "same-origin",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ path, label: name })
    });
    const body = await res.json().catch(() => ({}));
    if (res.ok) {
      return { ok: true };
    }
    const msg = body?.error?.message ?? body?.message ?? `HTTP ${res.status}`;
    if (/not a directory|exists/i.test(msg)) {
      return { ok: false, warning: msg };
    }
    return { ok: false, error: msg };
  } catch (e) {
    return { ok: false, error: e.message };
  }
}
function persistFromState(s, completed = false) {
  const entry = readEntry(s);
  entry.completed = completed;
  savePersisted(entry);
}
var controller_default = controller({
  scope: () => {
    const persisted = loadPersisted();
    return {
      step: signal(persisted.completed ? 1 : persisted.step ?? 1),
      open: signal(true),
      identityName: signal(persisted.identityName ?? ""),
      identityError: signal(""),
      provider: signal(persisted.provider ?? ""),
      providerTestOk: signal(persisted.providerValidated ?? false),
      providerTesting: signal(false),
      providerError: signal(""),
      providerOkLabel: signal(persisted.providerValidated ? "Ready" : ""),
      projectName: signal(persisted.projectName ?? ""),
      projectPath: signal(persisted.projectPath ?? ""),
      projectError: signal(""),
      projectWarning: signal(""),
      projectSubmitting: signal(false),
      sampleApp: signal(persisted.sampleApp ?? true),
      confirmingExit: signal(false),
      providerList: PROVIDERS
    };
  },
  on: {
    goNext: (s) => {
      const next = Math.min(TOTAL_STEPS, s.step + 1);
      s.step = next;
      persistFromState(s);
    },
    goBack: (s) => {
      const prev = Math.max(1, s.step - 1);
      s.step = prev;
      persistFromState(s);
    },
    submitIdentity: (s) => {
      const name = String(s.identityName ?? "").trim();
      if (!name) {
        s.identityError = "Please enter a display name.";
        return;
      }
      s.identityError = "";
      s.step = 3;
      persistFromState(s);
    },
    selectProvider: (s, id) => {
      s.provider = id;
      s.providerTestOk = false;
      s.providerError = "";
      s.providerOkLabel = "";
      (async () => {
        s.providerTesting = true;
        const installed = await fetchInstalledProviders();
        s.providerTesting = false;
        if (installed.includes(id)) {
          s.providerTestOk = true;
          s.providerOkLabel = `${id} CLI detected`;
          s.providerError = "";
        } else {
          s.providerTestOk = false;
          s.providerError = `${id} CLI not found on PATH. Install it, then retry — or pick another provider.`;
        }
        persistFromState(s);
      })();
    },
    validateProvider: (s) => {
      const id = String(s.provider ?? "");
      if (!id)
        return;
      s.providerTestOk = false;
      s.providerError = "";
      (async () => {
        s.providerTesting = true;
        const installed = await fetchInstalledProviders();
        s.providerTesting = false;
        if (installed.includes(id)) {
          s.providerTestOk = true;
          s.providerOkLabel = `${id} CLI detected`;
        } else {
          s.providerError = `${id} CLI not found on PATH. Install it, then retry — or pick another provider.`;
        }
        persistFromState(s);
      })();
    },
    submitProvider: (s) => {
      s.step = 4;
      persistFromState(s);
    },
    submitProject: (s) => {
      const name = String(s.projectName ?? "").trim();
      if (!name)
        return;
      const path = String(s.projectPath ?? "").trim() || defaultProjectPath(name);
      s.projectPath = path;
      s.projectError = "";
      s.projectWarning = "";
      s.projectSubmitting = true;
      (async () => {
        const result = await postProject(name, path);
        s.projectSubmitting = false;
        if (result.ok) {
          s.step = 5;
          persistFromState(s);
          return;
        }
        if (result.warning) {
          s.projectWarning = result.warning;
        } else if (result.error) {
          s.projectError = result.error;
        }
        s.step = 5;
        persistFromState(s);
      })();
    },
    submitSample: (s) => {
      s.step = 6;
      persistFromState(s);
    },
    finish: (s) => {
      persistFromState(s, true);
      try {
        localStorage.setItem("ship-setup-completed", "1");
      } catch {}
      window.location.href = "/";
    },
    requestExit: (s) => {
      s.confirmingExit = true;
    },
    cancelExit: (s) => {
      s.confirmingExit = false;
    },
    confirmExit: (s) => {
      s.confirmingExit = false;
      persistFromState(s, true);
      try {
        localStorage.setItem("ship-setup-completed", "1");
      } catch {}
      window.location.href = "/";
    },
    onFieldKeydown: (s, e) => {
      if (e.key === "Escape") {
        const target = e.target;
        target?.blur();
        e.stopPropagation();
        return;
      }
      if (e.key === "Enter") {
        e.preventDefault();
        const step = s.step;
        if (step === 2)
          s.submitIdentity?.();
        else if (step === 4)
          s.submitProject?.();
      }
    }
  },
  mount: (state, _el) => {
    const onKey = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === ".") {
        e.preventDefault();
        state.confirmingExit = true;
        return;
      }
      if (e.key === "Enter" && !(e.target instanceof HTMLInputElement)) {
        const step = state.step;
        if (step === 1)
          state.goNext?.();
        else if (step === 5)
          state.submitSample?.();
        else if (step === 6)
          state.finish?.();
      }
    };
    window.addEventListener("keydown", onKey, true);
    state._cleanup = () => {
      window.removeEventListener("keydown", onKey, true);
    };
  },
  unmount: (state) => {
    const cleanup = state._cleanup;
    if (typeof cleanup === "function")
      cleanup();
  }
});

// views/setup/page.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"01329411","dirty":true,"builtAt":"2026-05-19T16:54:50Z"} */
var page_default = Object.assign(view((ctx) => {
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
  const __wireBoundary_b5 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.step() === 1)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag8 = document.createDocumentFragment();
        const __c9 = [];
        const __e10 = document.createElement("section");
        __e10.setAttribute("class", "setup-step");
        __e10.setAttribute("data-test", "step-welcome");
        const __e11 = document.createElement("p");
        __e11.setAttribute("class", "setup-eyebrow");
        const __t12 = document.createTextNode("Step 1 of 6");
        __e11.appendChild(__t12);
        __e10.appendChild(__e11);
        const __e13 = document.createElement("h1");
        __e13.setAttribute("class", "setup-title");
        const __t14 = document.createTextNode("Welcome to Ship");
        __e13.appendChild(__t14);
        __e10.appendChild(__e13);
        const __e15 = document.createElement("p");
        __e15.setAttribute("class", "setup-lede");
        const __t16 = document.createTextNode(`
            Ship is the workbench for shipping software with agents. It runs
            locally — you own the data, the agents, and the pipelines.
          `);
        __e15.appendChild(__t16);
        __e10.appendChild(__e15);
        const __e17 = document.createElement("p");
        __e17.setAttribute("class", "setup-lede");
        const __t18 = document.createTextNode(`
            The next five steps set up an identity, connect a provider, and
            create your first project. You can change any of it later.
          `);
        __e17.appendChild(__t18);
        __e10.appendChild(__e17);
        const __e19 = document.createElement("div");
        __e19.setAttribute("class", "setup-actions setup-actions--end");
        const __e20 = document.createElement("sui-button");
        __e20.setAttribute("tone", "primary");
        __e20.setAttribute("variant", "solid");
        __e20.setAttribute("size", "md");
        __e20.setAttribute("data-test", "step-welcome-next");
        __c9.push(on(__e20, "click", function() {
          const __h = $scope.goNext;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t21 = document.createTextNode(`
              Let's set it up
              `);
        __e20.appendChild(__t21);
        const __e22 = document.createElement("sui-icon");
        __e22.setAttribute("slot", "trailing");
        __e22.setAttribute("name", "arrow-right");
        __e20.appendChild(__e22);
        __e19.appendChild(__e20);
        __e10.appendChild(__e19);
        __frag8.appendChild(__e10);
        return { nodes: Array.from(__frag8.childNodes), cleanup: () => {
          for (const c of __c9)
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
      if ($scope.identityError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag38 = document.createDocumentFragment();
        const __c39 = [];
        const __e40 = document.createElement("sui-alert");
        __e40.setAttribute("tone", "destructive");
        __e40.setAttribute("data-test", "step-identity-error");
        __e40.setAttribute("style", "margin-top:var(--space-3)");
        const __t41 = document.createTextNode("");
        __e40.appendChild(__t41);
        __c39.push(bindText(__t41, () => $scope.identityError()));
        __frag38.appendChild(__e40);
        return { nodes: Array.from(__frag38.childNodes), cleanup: () => {
          for (const c of __c39)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b7 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.step() === 2)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag24 = document.createDocumentFragment();
        const __c25 = [];
        const __e26 = document.createElement("section");
        __e26.setAttribute("class", "setup-step");
        __e26.setAttribute("data-test", "step-identity");
        const __e27 = document.createElement("p");
        __e27.setAttribute("class", "setup-eyebrow");
        const __t28 = document.createTextNode("Step 2 of 6");
        __e27.appendChild(__t28);
        __e26.appendChild(__e27);
        const __e29 = document.createElement("h2");
        __e29.setAttribute("class", "setup-title");
        const __t30 = document.createTextNode("What should we call you?");
        __e29.appendChild(__t30);
        __e26.appendChild(__e29);
        const __e31 = document.createElement("p");
        __e31.setAttribute("class", "setup-lede");
        const __t32 = document.createTextNode(`
            Your display name shows up next to agent activity, sessions, and
            git commits Ship makes on your behalf.
          `);
        __e31.appendChild(__t32);
        __e26.appendChild(__e31);
        const __e33 = document.createElement("sui-field");
        const __e34 = document.createElement("sui-label");
        __e34.setAttribute("for", "setup-identity-name");
        const __t35 = document.createTextNode("Display name");
        __e34.appendChild(__t35);
        __e33.appendChild(__e34);
        const __e36 = document.createElement("sui-input");
        __e36.setAttribute("id", "setup-identity-name");
        __e36.setAttribute("size", "md");
        __e36.setAttribute("placeholder", "e.g. Ada Lovelace");
        __e36.setAttribute("data-test", "step-identity-name");
        __c25.push(bindValue(__e36, $scope.identityName));
        __c25.push(on(__e36, "keydown", function() {
          const __h = $scope.onFieldKeydown;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e33.appendChild(__e36);
        __e26.appendChild(__e33);
        const __ifAnchor37 = document.createComment("if");
        __e26.appendChild(__ifAnchor37);
        __c25.push(__wireBoundary_b10(__ifAnchor37));
        const __e42 = document.createElement("div");
        __e42.setAttribute("class", "setup-actions");
        const __e43 = document.createElement("sui-button");
        __e43.setAttribute("variant", "ghost");
        __e43.setAttribute("size", "md");
        __c25.push(on(__e43, "click", function() {
          const __h = $scope.goBack;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e44 = document.createElement("sui-icon");
        __e44.setAttribute("slot", "leading");
        __e44.setAttribute("name", "arrow-left");
        __e43.appendChild(__e44);
        const __t45 = document.createTextNode(`Back
            `);
        __e43.appendChild(__t45);
        __e42.appendChild(__e43);
        const __e46 = document.createElement("sui-button");
        __e46.setAttribute("tone", "primary");
        __e46.setAttribute("variant", "solid");
        __e46.setAttribute("size", "md");
        __e46.setAttribute("data-test", "step-identity-next");
        __c25.push(on(__e46, "click", function() {
          const __h = $scope.submitIdentity;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t47 = document.createTextNode(`
              Continue
              `);
        __e46.appendChild(__t47);
        const __e48 = document.createElement("sui-icon");
        __e48.setAttribute("slot", "trailing");
        __e48.setAttribute("name", "arrow-right");
        __e46.appendChild(__e48);
        __e42.appendChild(__e46);
        __e26.appendChild(__e42);
        __frag24.appendChild(__e26);
        return { nodes: Array.from(__frag24.childNodes), cleanup: () => {
          for (const c of __c25)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b15 = (anchor) => bindFor({
    anchor,
    items: () => $scope.providerList,
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
      return (($scope2) => {
        __childScope61.__v_id = signal2(0);
        const __frag62 = document.createDocumentFragment();
        const __c63 = [];
        const __e64 = document.createElement("sui-card");
        __e64.setAttribute("interactive", "");
        __e64.setAttribute("padding", "md");
        __e64.setAttribute("role", "radio");
        {
          const __av65 = "provider-" + $scope2.p.id;
          if (__av65 == null || __av65 === false || __av65 === "")
            __e64.removeAttribute("data-test");
          else
            __e64.setAttribute("data-test", __av65 === true ? "" : String(__av65));
        }
        __c63.push(bindAttr(__e64, "aria-checked", () => (void $scope2.__v_id(), $scope2.provider() === $scope2.p.id ? "true" : "false")));
        __c63.push(bindProp(__e64, "tone", () => (void $scope2.__v_id(), $scope2.provider() === $scope2.p.id ? "primary" : "")));
        __c63.push(bindProp(__e64, "fill", () => (void $scope2.__v_id(), $scope2.provider() === $scope2.p.id)));
        __c63.push(on(__e64, "click", function() {
          const __h = () => $scope2.selectProvider($scope2.p.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e66 = document.createElement("div");
        __e66.setAttribute("class", "setup-provider-row");
        const __e67 = document.createElement("sui-provider-blob");
        __e67["provider"] = $scope2.p.id;
        __e67.setAttribute("size", "md");
        __e67.setAttribute("variant", "glyph");
        __e66.appendChild(__e67);
        const __e68 = document.createElement("div");
        __e68.setAttribute("style", "min-width:0");
        const __e69 = document.createElement("div");
        __e69.setAttribute("class", "setup-provider-name");
        const __t70 = document.createTextNode("");
        __e69.appendChild(__t70);
        __t70.textContent = $scope2.p.name;
        __e68.appendChild(__e69);
        const __e71 = document.createElement("div");
        __e71.setAttribute("class", "setup-provider-sub");
        const __t72 = document.createTextNode("");
        __e71.appendChild(__t72);
        __t72.textContent = $scope2.p.sub;
        __e68.appendChild(__e71);
        __e66.appendChild(__e68);
        __e64.appendChild(__e66);
        __frag62.appendChild(__e64);
        return { nodes: Array.from(__frag62.childNodes), cleanup: () => {
          for (const c of __c63)
            try {
              c();
            } catch (e) {}
        }, update: (__new73, __newI74) => {
          const __old = $scope2.p;
          $scope2.p = __new73;
          $scope2.__i = __newI74;
          if (__old.id !== $scope2.p.id) {
            {
              const __av65 = "provider-" + $scope2.p.id;
              if (__av65 == null || __av65 === false || __av65 === "")
                __e64.removeAttribute("data-test");
              else
                __e64.setAttribute("data-test", __av65 === true ? "" : String(__av65));
            }
            __e67["provider"] = $scope2.p.id;
          }
          if (__old.name !== $scope2.p.name) {
            __t70.textContent = $scope2.p.name;
          }
          if (__old.sub !== $scope2.p.sub) {
            __t72.textContent = $scope2.p.sub;
          }
          if (__old.id !== $scope2.p.id) {
            $scope2.__v_id.set((c) => c + 1);
          }
        } };
      })(__childScope61);
    }
  });
  const __wireBoundary_b24 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.providerTesting())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag77 = document.createDocumentFragment();
        const __c78 = [];
        const __e79 = document.createElement("sui-badge");
        __e79.setAttribute("tone", "info");
        __e79.setAttribute("variant", "soft");
        __e79.setAttribute("size", "sm");
        const __t80 = document.createTextNode("Validating…");
        __e79.appendChild(__t80);
        __frag77.appendChild(__e79);
        return { nodes: Array.from(__frag77.childNodes), cleanup: () => {
          for (const c of __c78)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b25 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.providerTestOk())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag82 = document.createDocumentFragment();
        const __c83 = [];
        const __e84 = document.createElement("sui-badge");
        __e84.setAttribute("tone", "success");
        __e84.setAttribute("variant", "soft");
        __e84.setAttribute("size", "sm");
        __e84.setAttribute("data-test", "provider-ok");
        const __e85 = document.createElement("sui-icon");
        __e85.setAttribute("slot", "leading");
        __e85.setAttribute("name", "check");
        __e84.appendChild(__e85);
        const __t86 = document.createTextNode("");
        __e84.appendChild(__t86);
        __c83.push(bindText(__t86, () => $scope.providerOkLabel()));
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
  const __wireBoundary_b27 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.providerError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag88 = document.createDocumentFragment();
        const __c89 = [];
        const __e90 = document.createElement("sui-alert");
        __e90.setAttribute("tone", "destructive");
        __e90.setAttribute("data-test", "provider-error");
        const __t91 = document.createTextNode("");
        __e90.appendChild(__t91);
        __c89.push(bindText(__t91, () => $scope.providerError()));
        const __e92 = document.createElement("sui-button");
        __e92.setAttribute("slot", "actions");
        __e92.setAttribute("size", "sm");
        __e92.setAttribute("variant", "outline");
        __e92.setAttribute("data-test", "provider-retry");
        __c89.push(on(__e92, "click", function() {
          const __h = $scope.validateProvider;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t93 = document.createTextNode(`
                  Retry
                `);
        __e92.appendChild(__t93);
        __e90.appendChild(__e92);
        __frag88.appendChild(__e90);
        return { nodes: Array.from(__frag88.childNodes), cleanup: () => {
          for (const c of __c89)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b14 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.step() === 3)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag50 = document.createDocumentFragment();
        const __c51 = [];
        const __e52 = document.createElement("section");
        __e52.setAttribute("class", "setup-step");
        __e52.setAttribute("data-test", "step-provider");
        const __e53 = document.createElement("p");
        __e53.setAttribute("class", "setup-eyebrow");
        const __t54 = document.createTextNode("Step 3 of 6");
        __e53.appendChild(__t54);
        __e52.appendChild(__e53);
        const __e55 = document.createElement("h2");
        __e55.setAttribute("class", "setup-title");
        const __t56 = document.createTextNode("Connect a provider");
        __e55.appendChild(__t56);
        __e52.appendChild(__e55);
        const __e57 = document.createElement("p");
        __e57.setAttribute("class", "setup-lede");
        const __t58 = document.createTextNode(`
            Ship drives whichever provider CLIs you have installed. Pick one
            to start with — add more later from Settings.
          `);
        __e57.appendChild(__t58);
        __e52.appendChild(__e57);
        const __e59 = document.createElement("div");
        __e59.setAttribute("class", "setup-grid-2");
        __e59.setAttribute("role", "radiogroup");
        __e59.setAttribute("aria-label", "AI provider");
        const __forAnchor60 = document.createComment("for");
        __e59.appendChild(__forAnchor60);
        __c51.push(__wireBoundary_b15(__forAnchor60));
        __e52.appendChild(__e59);
        const __e75 = document.createElement("div");
        __e75.setAttribute("class", "setup-provider-status");
        const __ifAnchor76 = document.createComment("if");
        __e75.appendChild(__ifAnchor76);
        __c51.push(__wireBoundary_b24(__ifAnchor76));
        const __ifAnchor81 = document.createComment("if");
        __e75.appendChild(__ifAnchor81);
        __c51.push(__wireBoundary_b25(__ifAnchor81));
        const __ifAnchor87 = document.createComment("if");
        __e75.appendChild(__ifAnchor87);
        __c51.push(__wireBoundary_b27(__ifAnchor87));
        __e52.appendChild(__e75);
        const __e94 = document.createElement("div");
        __e94.setAttribute("class", "setup-actions");
        const __e95 = document.createElement("sui-button");
        __e95.setAttribute("variant", "ghost");
        __e95.setAttribute("size", "md");
        __c51.push(on(__e95, "click", function() {
          const __h = $scope.goBack;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e96 = document.createElement("sui-icon");
        __e96.setAttribute("slot", "leading");
        __e96.setAttribute("name", "arrow-left");
        __e95.appendChild(__e96);
        const __t97 = document.createTextNode(`Back
            `);
        __e95.appendChild(__t97);
        __e94.appendChild(__e95);
        const __e98 = document.createElement("sui-button");
        __e98.setAttribute("tone", "primary");
        __e98.setAttribute("variant", "solid");
        __e98.setAttribute("size", "md");
        __e98.setAttribute("data-test", "step-provider-next");
        __c51.push(bindProp(__e98, "disabled", () => !$scope.provider() || $scope.providerTesting()));
        __c51.push(on(__e98, "click", function() {
          const __h = $scope.submitProvider;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t99 = document.createTextNode(`
              Continue
              `);
        __e98.appendChild(__t99);
        const __e100 = document.createElement("sui-icon");
        __e100.setAttribute("slot", "trailing");
        __e100.setAttribute("name", "arrow-right");
        __e98.appendChild(__e100);
        __e94.appendChild(__e98);
        __e52.appendChild(__e94);
        __frag50.appendChild(__e52);
        return { nodes: Array.from(__frag50.childNodes), cleanup: () => {
          for (const c of __c51)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b38 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.projectWarning())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag125 = document.createDocumentFragment();
        const __c126 = [];
        const __e127 = document.createElement("sui-alert");
        __e127.setAttribute("tone", "warning");
        __e127.setAttribute("data-test", "project-warning");
        __e127.setAttribute("style", "margin-top:var(--space-3)");
        const __t128 = document.createTextNode("");
        __e127.appendChild(__t128);
        __c126.push(bindText(__t128, () => $scope.projectWarning()));
        __frag125.appendChild(__e127);
        return { nodes: Array.from(__frag125.childNodes), cleanup: () => {
          for (const c of __c126)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b40 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.projectError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag130 = document.createDocumentFragment();
        const __c131 = [];
        const __e132 = document.createElement("sui-alert");
        __e132.setAttribute("tone", "destructive");
        __e132.setAttribute("data-test", "project-error");
        __e132.setAttribute("style", "margin-top:var(--space-3)");
        const __t133 = document.createTextNode("");
        __e132.appendChild(__t133);
        __c131.push(bindText(__t133, () => $scope.projectError()));
        __frag130.appendChild(__e132);
        return { nodes: Array.from(__frag130.childNodes), cleanup: () => {
          for (const c of __c131)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b33 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.step() === 4)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag102 = document.createDocumentFragment();
        const __c103 = [];
        const __e104 = document.createElement("section");
        __e104.setAttribute("class", "setup-step");
        __e104.setAttribute("data-test", "step-project");
        const __e105 = document.createElement("p");
        __e105.setAttribute("class", "setup-eyebrow");
        const __t106 = document.createTextNode("Step 4 of 6");
        __e105.appendChild(__t106);
        __e104.appendChild(__e105);
        const __e107 = document.createElement("h2");
        __e107.setAttribute("class", "setup-title");
        const __t108 = document.createTextNode("Create your first project");
        __e107.appendChild(__t108);
        __e104.appendChild(__e107);
        const __e109 = document.createElement("p");
        __e109.setAttribute("class", "setup-lede");
        const __t110 = document.createTextNode(`
            A project is a folder Ship watches. Agents and pipelines work
            inside it; you can have many.
          `);
        __e109.appendChild(__t110);
        __e104.appendChild(__e109);
        const __e111 = document.createElement("sui-field");
        const __e112 = document.createElement("sui-label");
        __e112.setAttribute("for", "setup-project-name");
        const __t113 = document.createTextNode("Project name");
        __e112.appendChild(__t113);
        __e111.appendChild(__e112);
        const __e114 = document.createElement("sui-input");
        __e114.setAttribute("id", "setup-project-name");
        __e114.setAttribute("size", "md");
        __e114.setAttribute("placeholder", "my-first-project");
        __e114.setAttribute("data-test", "step-project-name");
        __c103.push(bindValue(__e114, $scope.projectName));
        __c103.push(on(__e114, "keydown", function() {
          const __h = $scope.onFieldKeydown;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e111.appendChild(__e114);
        __e104.appendChild(__e111);
        const __e115 = document.createElement("sui-field");
        const __e116 = document.createElement("sui-label");
        __e116.setAttribute("for", "setup-project-path");
        const __t117 = document.createTextNode("Location");
        __e116.appendChild(__t117);
        __e115.appendChild(__e116);
        const __e118 = document.createElement("sui-input");
        __e118.setAttribute("id", "setup-project-path");
        __e118.setAttribute("size", "md");
        __e118.setAttribute("placeholder", "~/Documents/ShipProjects/<name>");
        __e118.setAttribute("data-test", "step-project-path");
        __c103.push(bindValue(__e118, $scope.projectPath));
        __c103.push(on(__e118, "keydown", function() {
          const __h = $scope.onFieldKeydown;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        __e115.appendChild(__e118);
        const __e119 = document.createElement("span");
        __e119.setAttribute("slot", "hint");
        __e119.setAttribute("class", "setup-hint");
        const __t120 = document.createTextNode(`
              Defaults to `);
        __e119.appendChild(__t120);
        const __e121 = document.createElement("sui-code");
        const __t122 = document.createTextNode("~/Documents/ShipProjects/<name>");
        __e121.appendChild(__t122);
        __e119.appendChild(__e121);
        const __t123 = document.createTextNode(`.
              The folder is created if it doesn't exist.
            `);
        __e119.appendChild(__t123);
        __e115.appendChild(__e119);
        __e104.appendChild(__e115);
        const __ifAnchor124 = document.createComment("if");
        __e104.appendChild(__ifAnchor124);
        __c103.push(__wireBoundary_b38(__ifAnchor124));
        const __ifAnchor129 = document.createComment("if");
        __e104.appendChild(__ifAnchor129);
        __c103.push(__wireBoundary_b40(__ifAnchor129));
        const __e134 = document.createElement("div");
        __e134.setAttribute("class", "setup-actions");
        const __e135 = document.createElement("sui-button");
        __e135.setAttribute("variant", "ghost");
        __e135.setAttribute("size", "md");
        __c103.push(on(__e135, "click", function() {
          const __h = $scope.goBack;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e136 = document.createElement("sui-icon");
        __e136.setAttribute("slot", "leading");
        __e136.setAttribute("name", "arrow-left");
        __e135.appendChild(__e136);
        const __t137 = document.createTextNode(`Back
            `);
        __e135.appendChild(__t137);
        __e134.appendChild(__e135);
        const __e138 = document.createElement("sui-button");
        __e138.setAttribute("tone", "primary");
        __e138.setAttribute("variant", "solid");
        __e138.setAttribute("size", "md");
        __e138.setAttribute("data-test", "step-project-next");
        __c103.push(bindProp(__e138, "loading", () => $scope.projectSubmitting()));
        __c103.push(bindProp(__e138, "disabled", () => !$scope.projectName().length));
        __c103.push(on(__e138, "click", function() {
          const __h = $scope.submitProject;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t139 = document.createTextNode(`
              Create project
              `);
        __e138.appendChild(__t139);
        const __e140 = document.createElement("sui-icon");
        __e140.setAttribute("slot", "trailing");
        __e140.setAttribute("name", "arrow-right");
        __e138.appendChild(__e140);
        __e134.appendChild(__e138);
        __e104.appendChild(__e134);
        __frag102.appendChild(__e104);
        return { nodes: Array.from(__frag102.childNodes), cleanup: () => {
          for (const c of __c103)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b46 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.step() === 5)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag142 = document.createDocumentFragment();
        const __c143 = [];
        const __e144 = document.createElement("section");
        __e144.setAttribute("class", "setup-step");
        __e144.setAttribute("data-test", "step-sample");
        const __e145 = document.createElement("p");
        __e145.setAttribute("class", "setup-eyebrow");
        const __t146 = document.createTextNode("Step 5 of 6");
        __e145.appendChild(__t146);
        __e144.appendChild(__e145);
        const __e147 = document.createElement("h2");
        __e147.setAttribute("class", "setup-title");
        const __t148 = document.createTextNode("Want a sample app to start with?");
        __e147.appendChild(__t148);
        __e144.appendChild(__e147);
        const __e149 = document.createElement("p");
        __e149.setAttribute("class", "setup-lede");
        const __t150 = document.createTextNode(`
            Scaffolds a "Hello, Ship" view in your project so the first time
            you open it isn't empty. Easy to delete later.
          `);
        __e149.appendChild(__t150);
        __e144.appendChild(__e149);
        const __e151 = document.createElement("sui-field");
        const __e152 = document.createElement("sui-checkbox");
        __e152.setAttribute("data-test", "step-sample-toggle");
        __c143.push(bindChecked(__e152, $scope.sampleApp));
        const __t153 = document.createTextNode(`
              Add the Hello, Ship sample app
            `);
        __e152.appendChild(__t153);
        __e151.appendChild(__e152);
        __e144.appendChild(__e151);
        const __e154 = document.createElement("div");
        __e154.setAttribute("class", "setup-actions");
        const __e155 = document.createElement("sui-button");
        __e155.setAttribute("variant", "ghost");
        __e155.setAttribute("size", "md");
        __c143.push(on(__e155, "click", function() {
          const __h = $scope.goBack;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e156 = document.createElement("sui-icon");
        __e156.setAttribute("slot", "leading");
        __e156.setAttribute("name", "arrow-left");
        __e155.appendChild(__e156);
        const __t157 = document.createTextNode(`Back
            `);
        __e155.appendChild(__t157);
        __e154.appendChild(__e155);
        const __e158 = document.createElement("sui-button");
        __e158.setAttribute("tone", "primary");
        __e158.setAttribute("variant", "solid");
        __e158.setAttribute("size", "md");
        __e158.setAttribute("data-test", "step-sample-next");
        __c143.push(on(__e158, "click", function() {
          const __h = $scope.submitSample;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t159 = document.createTextNode(`
              Continue
              `);
        __e158.appendChild(__t159);
        const __e160 = document.createElement("sui-icon");
        __e160.setAttribute("slot", "trailing");
        __e160.setAttribute("name", "arrow-right");
        __e158.appendChild(__e160);
        __e154.appendChild(__e158);
        __e144.appendChild(__e154);
        __frag142.appendChild(__e144);
        return { nodes: Array.from(__frag142.childNodes), cleanup: () => {
          for (const c of __c143)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b53 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.sampleApp())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag177 = document.createDocumentFragment();
        const __c178 = [];
        const __t179 = document.createTextNode("with the Hello, Ship sample app");
        __frag177.appendChild(__t179);
        return { nodes: Array.from(__frag177.childNodes), cleanup: () => {
          for (const c of __c178)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b50 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.step() === 6)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag162 = document.createDocumentFragment();
        const __c163 = [];
        const __e164 = document.createElement("section");
        __e164.setAttribute("class", "setup-step");
        __e164.setAttribute("data-test", "step-done");
        const __e165 = document.createElement("p");
        __e165.setAttribute("class", "setup-eyebrow");
        const __t166 = document.createTextNode("Step 6 of 6");
        __e165.appendChild(__t166);
        __e164.appendChild(__e165);
        const __e167 = document.createElement("h2");
        __e167.setAttribute("class", "setup-title");
        const __t168 = document.createTextNode("Ship is ready");
        __e167.appendChild(__t168);
        __e164.appendChild(__e167);
        const __e169 = document.createElement("p");
        __e169.setAttribute("class", "setup-lede");
        const __t170 = document.createTextNode(`
            Hi `);
        __e169.appendChild(__t170);
        const __t171 = document.createTextNode("");
        __e169.appendChild(__t171);
        __c163.push(bindText(__t171, () => $scope.identityName() || "there"));
        const __t172 = document.createTextNode(`, your project
            `);
        __e169.appendChild(__t172);
        const __e173 = document.createElement("strong");
        const __t174 = document.createTextNode("");
        __e173.appendChild(__t174);
        __c163.push(bindText(__t174, () => $scope.projectName()));
        __e169.appendChild(__e173);
        const __t175 = document.createTextNode(` is set up
            `);
        __e169.appendChild(__t175);
        const __ifAnchor176 = document.createComment("if");
        __e169.appendChild(__ifAnchor176);
        __c163.push(__wireBoundary_b53(__ifAnchor176));
        const __t180 = document.createTextNode(`.
          `);
        __e169.appendChild(__t180);
        __e164.appendChild(__e169);
        const __e181 = document.createElement("p");
        __e181.setAttribute("class", "setup-lede");
        const __t182 = document.createTextNode(`
            Jump in — the rail on the left navigates to Agents, Pipelines,
            and the rest of Ship.
          `);
        __e181.appendChild(__t182);
        __e164.appendChild(__e181);
        const __e183 = document.createElement("div");
        __e183.setAttribute("class", "setup-actions setup-actions--end");
        const __e184 = document.createElement("sui-button");
        __e184.setAttribute("tone", "primary");
        __e184.setAttribute("variant", "solid");
        __e184.setAttribute("size", "lg");
        __e184.setAttribute("data-test", "step-done-finish");
        __c163.push(on(__e184, "click", function() {
          const __h = $scope.finish;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t185 = document.createTextNode(`
              Open Ship
              `);
        __e184.appendChild(__t185);
        const __e186 = document.createElement("sui-icon");
        __e186.setAttribute("slot", "trailing");
        __e186.setAttribute("name", "arrow-right");
        __e184.appendChild(__e186);
        __e183.appendChild(__e184);
        __e164.appendChild(__e183);
        __frag162.appendChild(__e164);
        return { nodes: Array.from(__frag162.childNodes), cleanup: () => {
          for (const c of __c163)
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
      __e0.setAttribute("class", "ship-view setup-view");
      __e0.setAttribute("data-test", "setup-view");
      const __e1 = document.createElement("sui-dialog");
      __cleanups.push(bindAttr(__e1, "open", () => $scope.open() ? "" : null));
      __e1.setAttribute("backdrop", "blur");
      __e1.setAttribute("size", "lg");
      __e1.setAttribute("no-close-button", "");
      __e1.setAttribute("no-header", "");
      __e1.setAttribute("no-footer", "");
      __e1.setAttribute("dismissible", "false");
      const __e2 = document.createElement("div");
      __e2.setAttribute("class", "setup-wizard");
      __e2.setAttribute("data-test", "setup-wizard");
      __cleanups.push(bindAttr(__e2, "data-step", () => $scope.step()));
      const __e3 = document.createElement("header");
      __e3.setAttribute("class", "setup-head");
      const __e4 = document.createElement("sui-stepper");
      __cleanups.push(bindProp(__e4, "total", () => 6));
      __cleanups.push(bindProp(__e4, "current", () => $scope.step()));
      __e4.setAttribute("label", "Setup progress");
      __e3.appendChild(__e4);
      const __e5 = document.createElement("sui-button");
      __e5.setAttribute("variant", "ghost");
      __e5.setAttribute("size", "sm");
      __e5.setAttribute("data-test", "setup-skip");
      __cleanups.push(on(__e5, "click", function() {
        const __h = $scope.requestExit;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t6 = document.createTextNode(`
          Skip setup
        `);
      __e5.appendChild(__t6);
      __e3.appendChild(__e5);
      __e2.appendChild(__e3);
      const __ifAnchor7 = document.createComment("if");
      __e2.appendChild(__ifAnchor7);
      __cleanups.push(__wireBoundary_b5(__ifAnchor7));
      const __ifAnchor23 = document.createComment("if");
      __e2.appendChild(__ifAnchor23);
      __cleanups.push(__wireBoundary_b7(__ifAnchor23));
      const __ifAnchor49 = document.createComment("if");
      __e2.appendChild(__ifAnchor49);
      __cleanups.push(__wireBoundary_b14(__ifAnchor49));
      const __ifAnchor101 = document.createComment("if");
      __e2.appendChild(__ifAnchor101);
      __cleanups.push(__wireBoundary_b33(__ifAnchor101));
      const __ifAnchor141 = document.createComment("if");
      __e2.appendChild(__ifAnchor141);
      __cleanups.push(__wireBoundary_b46(__ifAnchor141));
      const __ifAnchor161 = document.createComment("if");
      __e2.appendChild(__ifAnchor161);
      __cleanups.push(__wireBoundary_b50(__ifAnchor161));
      __e1.appendChild(__e2);
      __e0.appendChild(__e1);
      const __e187 = document.createElement("sui-alert-dialog");
      __cleanups.push(bindAttr(__e187, "open", () => $scope.confirmingExit() ? "" : null));
      __e187.setAttribute("tone", "default");
      __e187.setAttribute("confirm-label", "Skip setup");
      __e187.setAttribute("cancel-label", "Keep going");
      __e187.setAttribute("data-test", "setup-confirm-exit");
      __cleanups.push(on(__e187, "sui-confirm", function() {
        const __h = $scope.confirmExit;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __cleanups.push(on(__e187, "sui-cancel", function() {
        const __h = $scope.cancelExit;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e188 = document.createElement("span");
      __e188.setAttribute("slot", "title");
      const __t189 = document.createTextNode("Skip setup?");
      __e188.appendChild(__t189);
      __e187.appendChild(__e188);
      const __e190 = document.createElement("p");
      const __t191 = document.createTextNode(`
      We'll apply sensible defaults for the steps you haven't filled in.
      You can run setup again from Settings → About.
    `);
      __e190.appendChild(__t191);
      __e187.appendChild(__e190);
      __e0.appendChild(__e187);
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
        __cleanups.push(hydrateAttr(__m["b0"], "open", () => $scope.open() ? "" : null));
      if (__m["b1"])
        __cleanups.push(hydrateAttr(__m["b1"], "data-step", () => $scope.step()));
      if (__m["b2"])
        __cleanups.push(hydrateProp(__m["b2"], "total", () => 6));
      if (__m["b3"])
        __cleanups.push(hydrateProp(__m["b3"], "current", () => $scope.step()));
      if (__m["b4"])
        __cleanups.push(hydrateOn(__m["b4"], "click", function() {
          const __h = $scope.requestExit;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b5"]) {
        const __t = __m["b5"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b5(__anchor));
      }
      if (__m["b7"]) {
        const __t = __m["b7"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b7(__anchor));
      }
      if (__m["b14"]) {
        const __t = __m["b14"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b14(__anchor));
      }
      if (__m["b33"]) {
        const __t = __m["b33"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b33(__anchor));
      }
      if (__m["b46"]) {
        const __t = __m["b46"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b46(__anchor));
      }
      if (__m["b50"]) {
        const __t = __m["b50"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b50(__anchor));
      }
      if (__m["b55"])
        __cleanups.push(hydrateAttr(__m["b55"], "open", () => $scope.confirmingExit() ? "" : null));
      if (__m["b56"])
        __cleanups.push(hydrateOn(__m["b56"], "sui-confirm", function() {
          const __h = $scope.confirmExit;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b57"])
        __cleanups.push(hydrateOn(__m["b57"], "sui-cancel", function() {
          const __h = $scope.cancelExit;
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
}), { _compilerVersion: { cargo: "0.1.0", git: "01329411", dirty: true, builtAt: "2026-05-19T16:54:50Z" } });
export {
  page_default as default
};
