// views/theme-studio/page.sui
import { bindAttr, bindFor, bindIf, bindText, effect, on, signal as signal2, view } from "@ship/sui-runtime";
import { collectElementMarkers, hydrateOn } from "@ship/sui-dom";

// views/theme-studio/controller.ts
import { controller, signal, derived, chrome } from "@ship/sui-runtime";

// packages/components/ui-components/src/tokens/themes.ts
var THEMES = [
  {
    id: "ship-light",
    label: "SHIP Light",
    dark: false,
    swatches: [
      "oklch(0.99 0.001 106.423)",
      "oklch(1 0 0)",
      "oklch(0.67 0.16 58)",
      "oklch(0.61 0.23 303)"
    ]
  },
  {
    id: "ship-dark",
    label: "SHIP Dark",
    dark: true,
    swatches: [
      "oklch(0.18 0.01 250)",
      "oklch(0.18 0.01 50)",
      "oklch(0.77 0.16 70)",
      "oklch(0.3 0.05 300)"
    ]
  },
  {
    id: "paper",
    label: "Paper",
    dark: false,
    swatches: [
      "oklch(0.97 0.014 80)",
      "oklch(0.99 0.010 80)",
      "oklch(0.55 0.14 35)",
      "oklch(0.48 0.09 55)"
    ]
  },
  {
    id: "halo",
    label: "Halo (Soft)",
    dark: false,
    swatches: [
      "oklch(0.92 0.008 230)",
      "oklch(0.92 0.008 230)",
      "oklch(0.62 0.14 50)",
      "oklch(0.55 0.05 230)"
    ]
  },
  {
    id: "console",
    label: "Console",
    dark: true,
    swatches: [
      "oklch(0.13 0.008 220)",
      "oklch(0.16 0.010 220)",
      "oklch(0.78 0.20 145)",
      "oklch(0.78 0.16 195)"
    ]
  },
  {
    id: "contrast",
    label: "High Contrast",
    dark: true,
    swatches: [
      "oklch(0.09 0 0)",
      "oklch(0.15 0 0)",
      "oklch(0.87 0.18 85)",
      "oklch(0.99 0 0)"
    ]
  },
  {
    id: "glass",
    label: "Glass",
    dark: false,
    swatches: [
      "oklch(0.95 0.04 230)",
      "oklch(0.99 0.005 240)",
      "oklch(0.62 0.16 50)",
      "oklch(0.62 0.14 280)"
    ]
  }
];
var THEME_BY_ID = Object.fromEntries(THEMES.map((t) => [t.id, t]));
var DEFAULT_THEME = "ship-light";
var THEME_STORAGE_KEY = "ship-site-theme";
var isValidTheme = (value) => typeof value === "string" && (value in THEME_BY_ID);
var isDarkTheme = (id) => THEME_BY_ID[id]?.dark ?? false;
function applyTheme(id, root = document.documentElement) {
  const next = isValidTheme(id) ? id : DEFAULT_THEME;
  const dark = isDarkTheme(next);
  root.dataset.theme = next;
  root.classList.toggle("dark", dark);
  if (typeof document !== "undefined" && typeof CustomEvent !== "undefined") {
    document.dispatchEvent(new CustomEvent("ship:theme:change", { detail: { id: next, dark } }));
  }
  return next;
}
function readStoredTheme() {
  if (typeof localStorage === "undefined")
    return DEFAULT_THEME;
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY);
    return isValidTheme(raw) ? raw : DEFAULT_THEME;
  } catch {
    return DEFAULT_THEME;
  }
}
function persistTheme(id) {
  if (typeof localStorage === "undefined")
    return;
  try {
    localStorage.setItem(THEME_STORAGE_KEY, id);
  } catch {}
  try {
    fetch("/api/settings", {
      method: "PUT",
      credentials: "same-origin",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ key: "shell.theme", value: id })
    }).catch(() => {});
  } catch {}
}

// views/theme-studio/token-catalog.ts
var LAYERS = [
  {
    id: "palette",
    label: "Palette",
    blurb: "Brand colors. Tone ramps rebuild from these bases.",
    tokens: [
      { name: "--background", type: "color", intent: "App background" },
      { name: "--foreground", type: "color", intent: "App text" },
      { name: "--card", type: "color", intent: "Card background" },
      { name: "--card-foreground", type: "color", intent: "Card text" },
      { name: "--primary", type: "color", intent: "Brand primary" },
      { name: "--primary-foreground", type: "color", intent: "On primary" },
      { name: "--accent", type: "color", intent: "Brand accent" },
      { name: "--accent-foreground", type: "color", intent: "On accent" },
      { name: "--muted", type: "color", intent: "Muted surface" },
      { name: "--muted-foreground", type: "color", intent: "Muted text" },
      { name: "--border", type: "color", intent: "Borders" },
      { name: "--ring", type: "color", intent: "Focus ring" },
      { name: "--success", type: "color", intent: "Success base" },
      { name: "--warning", type: "color", intent: "Warning base" },
      { name: "--info", type: "color", intent: "Info base" },
      { name: "--destructive", type: "color", intent: "Destructive base" }
    ]
  },
  {
    id: "surface",
    label: "Semantic surfaces & intents",
    blurb: "Depth and interaction surfaces. Cards read --surface-1, popovers --surface-3.",
    tokens: [
      { name: "--surface-0", type: "color", intent: "App-level surface" },
      { name: "--surface-1", type: "color", intent: "Resting card surface" },
      { name: "--surface-2", type: "color", intent: "Nested panel surface" },
      { name: "--surface-3", type: "color", intent: "Floating/popover surface" },
      { name: "--backdrop", type: "color", intent: "Modal backdrop scrim" },
      { name: "--scrim", type: "color", intent: "Lighter scrim overlay" },
      { name: "--hover-bg", type: "color", intent: "Hover tint" },
      { name: "--selected-bg", type: "color", intent: "Selected tint" },
      { name: "--divider", type: "color", intent: "Divider line" }
    ]
  },
  {
    id: "tone",
    label: "Tone ramps",
    blurb: "Status tones. Read by chips, badges, alerts. Ramps derive from palette bases above.",
    tokens: [
      { name: "--tone-success-fg", type: "color", intent: "Success fg" },
      { name: "--tone-success-bg", type: "color", intent: "Success soft bg" },
      { name: "--tone-success-border", type: "color", intent: "Success border" },
      { name: "--tone-warning-fg", type: "color", intent: "Warning fg" },
      { name: "--tone-warning-bg", type: "color", intent: "Warning soft bg" },
      { name: "--tone-destructive-fg", type: "color", intent: "Destructive fg" },
      { name: "--tone-destructive-bg", type: "color", intent: "Destructive soft bg" },
      { name: "--tone-info-fg", type: "color", intent: "Info fg" },
      { name: "--tone-info-bg", type: "color", intent: "Info soft bg" }
    ]
  },
  {
    id: "role",
    label: "Role tokens",
    blurb: "Typography, motion, density, shadow intent.",
    tokens: [
      { name: "--heading-font", type: "font", intent: "Heading font stack" },
      { name: "--heading-weight", type: "number", intent: "Heading weight" },
      { name: "--body-font", type: "font", intent: "Body font stack" },
      { name: "--label-font", type: "font", intent: "Label font stack" },
      { name: "--ease-out", type: "easing", intent: "Default ease" },
      { name: "--ease-out-soft", type: "easing", intent: "Soft ease" },
      { name: "--density-row-py", type: "length", intent: "Row vertical padding" },
      { name: "--density-row-px", type: "length", intent: "Row horizontal padding" },
      { name: "--shadow-hover", type: "shadow", intent: "Hover shadow" },
      { name: "--shadow-accent", type: "shadow", intent: "Primary-tinted focus glow" }
    ]
  },
  {
    id: "primitive",
    label: "Primitives",
    blurb: "Raw scale. Rarely the right override point.",
    tokens: [
      { name: "--space-1", type: "length", intent: "4px" },
      { name: "--space-2", type: "length", intent: "8px" },
      { name: "--space-3", type: "length", intent: "12px" },
      { name: "--space-4", type: "length", intent: "16px" },
      { name: "--radius-sm", type: "length", intent: "Small radius" },
      { name: "--radius-md", type: "length", intent: "Medium radius" },
      { name: "--radius-lg", type: "length", intent: "Large radius" },
      { name: "--radius-xl", type: "length", intent: "XL radius" },
      { name: "--duration-fast", type: "duration", intent: "Fast duration" },
      { name: "--duration-medium", type: "duration", intent: "Medium duration" },
      { name: "--duration-slow", type: "duration", intent: "Slow duration" }
    ]
  }
];
var THEME_PACKS = [
  { id: "ship-light", label: "SHIP Light" },
  { id: "ship-dark", label: "SHIP Dark" },
  { id: "paper", label: "Paper" },
  { id: "halo", label: "Halo (Soft)" },
  { id: "console", label: "Console" },
  { id: "contrast", label: "High Contrast" }
];
var ALL_TOKENS = LAYERS.flatMap((l) => l.tokens);

// views/theme-studio/controller.ts
var STORAGE_KEY = "theme.overrides";
function readResolved(token) {
  if (typeof window === "undefined")
    return "";
  return getComputedStyle(document.documentElement).getPropertyValue(token).trim();
}
function snapshotPack() {
  const out = {};
  for (const t of ALL_TOKENS)
    out[t.name] = readResolved(t.name);
  return out;
}
async function loadOverrides() {
  try {
    const res = await fetch(`/api/v1/settings/${encodeURIComponent(STORAGE_KEY)}`, {
      credentials: "same-origin"
    });
    if (!res.ok)
      return null;
    const body = await res.json();
    const raw = "data" in body ? body.data?.value : body.value;
    if (typeof raw !== "string")
      return null;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object" || !parsed.values)
      return null;
    const pack = isValidTheme(parsed.pack) ? parsed.pack : readStoredTheme();
    return { pack, values: parsed.values };
  } catch {
    return null;
  }
}
async function saveOverrides(payload) {
  await fetch("/api/v1/settings", {
    method: "PUT",
    credentials: "same-origin",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ key: STORAGE_KEY, value: JSON.stringify(payload) })
  });
}
var controller_default = controller({
  scope: () => {
    const activePack = signal(readStoredTheme());
    const packDefaults = signal({});
    const overrides = signal({});
    const savedSnapshot = signal(null);
    const status = signal({ tone: "idle", msg: "" });
    const layers = derived(() => LAYERS);
    const packs = derived(() => THEME_PACKS);
    const values = derived(() => {
      const o = overrides();
      const d = packDefaults();
      const out = {};
      for (const t of ALL_TOKENS) {
        if (t.name in o)
          out[t.name] = o[t.name] ?? "";
        else if (t.name in d)
          out[t.name] = d[t.name] ?? "";
        else
          out[t.name] = readResolved(t.name);
      }
      return out;
    });
    function setToken(token, value) {
      const next = { ...overrides(), [token]: value };
      overrides.set(next);
      chrome.theme.set(token, value);
    }
    function applyAllOverrides(values2) {
      for (const [k, v] of Object.entries(values2)) {
        chrome.theme.set(k, v);
      }
    }
    function clearAllOverrides(values2) {
      for (const k of Object.keys(values2))
        chrome.theme.clear(k);
    }
    function selectPack(idRaw) {
      if (!isValidTheme(idRaw))
        return;
      const id = idRaw;
      clearAllOverrides(overrides());
      const next = applyTheme(id);
      persistTheme(next);
      activePack.set(next);
      overrides.set({});
      requestAnimationFrame(() => packDefaults.set(snapshotPack()));
    }
    function resetToPack() {
      clearAllOverrides(overrides());
      overrides.set({});
      status.set({ tone: "idle", msg: "Reset to pack defaults." });
    }
    function resetToSaved() {
      const saved = savedSnapshot();
      clearAllOverrides(overrides());
      if (!saved) {
        overrides.set({});
        status.set({ tone: "idle", msg: "Reverted (no saved overrides)." });
        return;
      }
      overrides.set({ ...saved.values });
      applyAllOverrides(saved.values);
      status.set({ tone: "idle", msg: "Reverted to last saved." });
    }
    async function savePersist() {
      const payload = { pack: activePack(), values: { ...overrides() } };
      try {
        await saveOverrides(payload);
        savedSnapshot.set(payload);
        status.set({ tone: "saved", msg: "Saved." });
        setTimeout(() => {
          if (status().tone === "saved")
            status.set({ tone: "idle", msg: "" });
        }, 1600);
      } catch (err) {
        status.set({ tone: "error", msg: `Save failed: ${err.message}` });
      }
    }
    function exportJson() {
      const payload = { pack: activePack(), values: { ...overrides() } };
      const blob = new Blob([JSON.stringify(payload, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `theme-${payload.pack}-overrides.json`;
      a.click();
      URL.revokeObjectURL(url);
    }
    async function importFiles(files) {
      if (!files || files.length === 0)
        return;
      const file = files[0];
      if (!file)
        return;
      try {
        const text = await file.text();
        const parsed = JSON.parse(text);
        if (!parsed.values || typeof parsed.values !== "object") {
          throw new Error('missing "values" object');
        }
        const pack = isValidTheme(parsed.pack) ? parsed.pack : activePack();
        if (pack !== activePack())
          selectPack(pack);
        clearAllOverrides(overrides());
        const values2 = parsed.values;
        overrides.set({ ...values2 });
        applyAllOverrides(values2);
        status.set({ tone: "saved", msg: `Imported ${Object.keys(values2).length} overrides.` });
      } catch (err) {
        status.set({ tone: "error", msg: `Import failed: ${err.message}` });
      }
    }
    return {
      activePack,
      packDefaults,
      overrides,
      savedSnapshot,
      status,
      layers,
      packs,
      values,
      setToken,
      selectPack,
      resetToPack,
      resetToSaved,
      savePersist,
      exportJson,
      importFiles
    };
  },
  mount(state) {
    const scope = state;
    scope.packDefaults = snapshotPack();
    loadOverrides().then((saved) => {
      if (!saved)
        return;
      scope.savedSnapshot = saved;
      if (saved.pack !== scope.activePack) {
        applyTheme(saved.pack);
        scope.activePack = saved.pack;
        requestAnimationFrame(() => {
          scope.packDefaults = snapshotPack();
        });
      }
      scope.overrides = { ...saved.values };
      for (const [k, v] of Object.entries(saved.values))
        chrome.theme.set(k, v);
    });
    const onThemeChange = () => {
      requestAnimationFrame(() => {
        scope.packDefaults = snapshotPack();
      });
    };
    document.addEventListener("ship:theme:change", onThemeChange);
    return () => {
      document.removeEventListener("ship:theme:change", onThemeChange);
    };
  }
});

// views/theme-studio/page.sui
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
  const __wireBoundary_b1 = (anchor) => bindFor({
    anchor,
    items: () => $scope.packs(),
    key: (p, __i) => {
      const __child = Object.create($scope);
      __child.p = p;
      __child.__i = __i;
      return (($scope2) => p.id)(__child);
    },
    render: (p, __i) => {
      const __childScope7 = Object.create($scope);
      __childScope7.p = p;
      __childScope7.__i = __i;
      return (($scope2) => {
        __childScope7.__v_id = signal2(0);
        const __frag8 = document.createDocumentFragment();
        const __c9 = [];
        const __e10 = document.createElement("option");
        {
          const __av11 = $scope2.p.id;
          if (__av11 == null || __av11 === false || __av11 === "")
            __e10.removeAttribute("value");
          else
            __e10.setAttribute("value", __av11 === true ? "" : String(__av11));
        }
        __c9.push(bindAttr(__e10, "selected", () => (void $scope2.__v_id(), $scope2.activePack() === $scope2.p.id ? "" : null)));
        const __t12 = document.createTextNode("");
        __e10.appendChild(__t12);
        __t12.textContent = $scope2.p.label;
        __frag8.appendChild(__e10);
        return { nodes: Array.from(__frag8.childNodes), cleanup: () => {
          for (const c of __c9)
            try {
              c();
            } catch (e) {}
        }, update: (__new13, __newI14) => {
          const __old = $scope2.p;
          $scope2.p = __new13;
          $scope2.__i = __newI14;
          if (__old.id !== $scope2.p.id) {
            {
              const __av11 = $scope2.p.id;
              if (__av11 == null || __av11 === false || __av11 === "")
                __e10.removeAttribute("value");
              else
                __e10.setAttribute("value", __av11 === true ? "" : String(__av11));
            }
          }
          if (__old.label !== $scope2.p.label) {
            __t12.textContent = $scope2.p.label;
          }
          if (__old.id !== $scope2.p.id) {
            $scope2.__v_id.set((c) => c + 1);
          }
        } };
      })(__childScope7);
    }
  });
  const __wireBoundary_b5 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.status().tone === "saved")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag17 = document.createDocumentFragment();
        const __c18 = [];
        const __e19 = document.createElement("span");
        __e19.setAttribute("class", "ts-pill ts-pill-ok");
        __e19.setAttribute("data-test", "theme-studio-status-saved");
        const __t20 = document.createTextNode("");
        __e19.appendChild(__t20);
        __c18.push(bindText(__t20, () => $scope.status().msg));
        __frag17.appendChild(__e19);
        return { nodes: Array.from(__frag17.childNodes), cleanup: () => {
          for (const c of __c18)
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
      if ($scope.status().tone === "error")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag22 = document.createDocumentFragment();
        const __c23 = [];
        const __e24 = document.createElement("span");
        __e24.setAttribute("class", "ts-pill ts-pill-err");
        const __t25 = document.createTextNode("");
        __e24.appendChild(__t25);
        __c23.push(bindText(__t25, () => $scope.status().msg));
        __frag22.appendChild(__e24);
        return { nodes: Array.from(__frag22.childNodes), cleanup: () => {
          for (const c of __c23)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b9 = (anchor) => bindIf({
    anchor,
    test: () => {
      if ($scope.status().tone === "idle" && $scope.status().msg)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag27 = document.createDocumentFragment();
        const __c28 = [];
        const __e29 = document.createElement("span");
        __e29.setAttribute("class", "ts-pill");
        const __t30 = document.createTextNode("");
        __e29.appendChild(__t30);
        __c28.push(bindText(__t30, () => $scope.status().msg));
        __frag27.appendChild(__e29);
        return { nodes: Array.from(__frag27.childNodes), cleanup: () => {
          for (const c of __c28)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b16 = (anchor) => bindFor({
    anchor,
    items: () => $scope.layers(),
    key: (layer, __i) => {
      const __child = Object.create($scope);
      __child.layer = layer;
      __child.__i = __i;
      return (($scope2) => layer.id)(__child);
    },
    render: (layer, __i) => {
      const __childScope45 = Object.create($scope);
      __childScope45.layer = layer;
      __childScope45.__i = __i;
      return (($scope2) => {
        const __wireBoundary_b20 = (anchor2) => bindFor({
          anchor: anchor2,
          items: () => $scope2.layer.tokens,
          key: (tok, __i2) => {
            const __child = Object.create($scope2);
            __child.tok = tok;
            __child.__i = __i2;
            return (($scope3) => tok.name)(__child);
          },
          render: (tok, __i2) => {
            const __childScope57 = Object.create($scope2);
            __childScope57.tok = tok;
            __childScope57.__i = __i2;
            return (($scope3) => {
              __childScope57.__v_name = signal2(0);
              const __wireBoundary_b26 = (anchor3) => bindIf({
                anchor: anchor3,
                test: () => {
                  if ($scope3.tok.type === "color")
                    return 0;
                  return -1;
                },
                branches: [
                  () => {
                    const __frag69 = document.createDocumentFragment();
                    const __c70 = [];
                    const __e71 = document.createElement("span");
                    __e71.setAttribute("class", "ts-swatch");
                    __c70.push(effect(() => {
                      $scope3.__v_name();
                      const __v = $scope3.values()[$scope3.tok.name];
                      if (__v == null)
                        __e71.style.removeProperty("background");
                      else
                        __e71.style.setProperty("background", String(__v));
                    }));
                    __e71.setAttribute("aria-hidden", "true");
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
              const __frag58 = document.createDocumentFragment();
              const __c59 = [];
              const __e60 = document.createElement("div");
              __e60.setAttribute("class", "ts-row");
              {
                const __av61 = $scope3.tok.name;
                if (__av61 == null || __av61 === false || __av61 === "")
                  __e60.removeAttribute("data-token");
                else
                  __e60.setAttribute("data-token", __av61 === true ? "" : String(__av61));
              }
              {
                const __av62 = $scope3.tok.type;
                if (__av62 == null || __av62 === false || __av62 === "")
                  __e60.removeAttribute("data-token-type");
                else
                  __e60.setAttribute("data-token-type", __av62 === true ? "" : String(__av62));
              }
              const __e63 = document.createElement("label");
              __e63.setAttribute("class", "ts-row-name");
              {
                const __av64 = "ts-in-" + $scope3.tok.name;
                if (__av64 == null || __av64 === false || __av64 === "")
                  __e63.removeAttribute("for");
                else
                  __e63.setAttribute("for", __av64 === true ? "" : String(__av64));
              }
              const __t65 = document.createTextNode("");
              __e63.appendChild(__t65);
              __t65.textContent = $scope3.tok.name;
              const __e66 = document.createElement("span");
              __e66.setAttribute("class", "ts-row-intent");
              const __t67 = document.createTextNode("");
              __e66.appendChild(__t67);
              __t67.textContent = $scope3.tok.intent;
              __e63.appendChild(__e66);
              __e60.appendChild(__e63);
              const __ifAnchor68 = document.createComment("if");
              __e60.appendChild(__ifAnchor68);
              __c59.push(__wireBoundary_b26(__ifAnchor68));
              const __e72 = document.createElement("input");
              __e72.setAttribute("class", "ts-input");
              __e72.setAttribute("type", "text");
              {
                const __av73 = "ts-in-" + $scope3.tok.name;
                if (__av73 == null || __av73 === false || __av73 === "")
                  __e72.removeAttribute("id");
                else
                  __e72.setAttribute("id", __av73 === true ? "" : String(__av73));
              }
              {
                const __av74 = $scope3.tok.name;
                if (__av74 == null || __av74 === false || __av74 === "")
                  __e72.removeAttribute("aria-label");
                else
                  __e72.setAttribute("aria-label", __av74 === true ? "" : String(__av74));
              }
              __c59.push(bindAttr(__e72, "value", () => (void $scope3.__v_name(), $scope3.values()[$scope3.tok.name])));
              __c59.push(on(__e72, "input", function() {
                const __h = (e) => $scope3.setToken($scope3.tok.name, e.target.value);
                return typeof __h === "function" ? __h.bind(undefined) : () => {};
              }(), { modifiers: [] }));
              __e60.appendChild(__e72);
              __frag58.appendChild(__e60);
              return { nodes: Array.from(__frag58.childNodes), cleanup: () => {
                for (const c of __c59)
                  try {
                    c();
                  } catch (e) {}
              }, update: (__new75, __newI76) => {
                const __old = $scope3.tok;
                $scope3.tok = __new75;
                $scope3.__i = __newI76;
                if (__old.name !== $scope3.tok.name) {
                  {
                    const __av61 = $scope3.tok.name;
                    if (__av61 == null || __av61 === false || __av61 === "")
                      __e60.removeAttribute("data-token");
                    else
                      __e60.setAttribute("data-token", __av61 === true ? "" : String(__av61));
                  }
                  {
                    const __av64 = "ts-in-" + $scope3.tok.name;
                    if (__av64 == null || __av64 === false || __av64 === "")
                      __e63.removeAttribute("for");
                    else
                      __e63.setAttribute("for", __av64 === true ? "" : String(__av64));
                  }
                  __t65.textContent = $scope3.tok.name;
                  {
                    const __av73 = "ts-in-" + $scope3.tok.name;
                    if (__av73 == null || __av73 === false || __av73 === "")
                      __e72.removeAttribute("id");
                    else
                      __e72.setAttribute("id", __av73 === true ? "" : String(__av73));
                  }
                  {
                    const __av74 = $scope3.tok.name;
                    if (__av74 == null || __av74 === false || __av74 === "")
                      __e72.removeAttribute("aria-label");
                    else
                      __e72.setAttribute("aria-label", __av74 === true ? "" : String(__av74));
                  }
                }
                if (__old.type !== $scope3.tok.type) {
                  {
                    const __av62 = $scope3.tok.type;
                    if (__av62 == null || __av62 === false || __av62 === "")
                      __e60.removeAttribute("data-token-type");
                    else
                      __e60.setAttribute("data-token-type", __av62 === true ? "" : String(__av62));
                  }
                }
                if (__old.intent !== $scope3.tok.intent) {
                  __t67.textContent = $scope3.tok.intent;
                }
                if (__old.name !== $scope3.tok.name) {
                  $scope3.__v_name.set((c) => c + 1);
                }
              } };
            })(__childScope57);
          }
        });
        const __frag46 = document.createDocumentFragment();
        const __c47 = [];
        const __e48 = document.createElement("section");
        __e48.setAttribute("class", "ts-layer");
        {
          const __av49 = $scope2.layer.id;
          if (__av49 == null || __av49 === false || __av49 === "")
            __e48.removeAttribute("data-layer");
          else
            __e48.setAttribute("data-layer", __av49 === true ? "" : String(__av49));
        }
        const __e50 = document.createElement("header");
        __e50.setAttribute("class", "ts-layer-head");
        const __e51 = document.createElement("h2");
        __e51.setAttribute("class", "ts-layer-title");
        const __t52 = document.createTextNode("");
        __e51.appendChild(__t52);
        __t52.textContent = $scope2.layer.label;
        __e50.appendChild(__e51);
        const __e53 = document.createElement("p");
        __e53.setAttribute("class", "ts-layer-blurb");
        const __t54 = document.createTextNode("");
        __e53.appendChild(__t54);
        __t54.textContent = $scope2.layer.blurb;
        __e50.appendChild(__e53);
        __e48.appendChild(__e50);
        const __e55 = document.createElement("sui-grid");
        const __forAnchor56 = document.createComment("for");
        __e55.appendChild(__forAnchor56);
        __c47.push(__wireBoundary_b20(__forAnchor56));
        __e48.appendChild(__e55);
        __frag46.appendChild(__e48);
        return { nodes: Array.from(__frag46.childNodes), cleanup: () => {
          for (const c of __c47)
            try {
              c();
            } catch (e) {}
        }, update: (__new77, __newI78) => {
          const __old = $scope2.layer;
          $scope2.layer = __new77;
          $scope2.__i = __newI78;
          if (__old.id !== $scope2.layer.id) {
            {
              const __av49 = $scope2.layer.id;
              if (__av49 == null || __av49 === false || __av49 === "")
                __e48.removeAttribute("data-layer");
              else
                __e48.setAttribute("data-layer", __av49 === true ? "" : String(__av49));
            }
          }
          if (__old.label !== $scope2.layer.label) {
            __t52.textContent = $scope2.layer.label;
          }
          if (__old.blurb !== $scope2.layer.blurb) {
            __t54.textContent = $scope2.layer.blurb;
          }
        } };
      })(__childScope45);
    }
  });
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("div");
      __e0.setAttribute("class", "ship-view ship-theme-studio");
      const __e1 = document.createElement("header");
      __e1.setAttribute("class", "ts-top");
      __e1.setAttribute("data-test", "theme-studio-top");
      const __e2 = document.createElement("div");
      __e2.setAttribute("class", "ts-pack");
      const __e3 = document.createElement("label");
      __e3.setAttribute("for", "ts-pack-select");
      __e3.setAttribute("class", "ts-top-label");
      const __t4 = document.createTextNode("Theme pack");
      __e3.appendChild(__t4);
      __e2.appendChild(__e3);
      const __e5 = document.createElement("select");
      __e5.setAttribute("id", "ts-pack-select");
      __e5.setAttribute("class", "ts-select");
      __e5.setAttribute("data-test", "theme-studio-pack-select");
      __cleanups.push(on(__e5, "change", function() {
        const __h = (e) => $scope.selectPack(e.target.value);
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __forAnchor6 = document.createComment("for");
      __e5.appendChild(__forAnchor6);
      __cleanups.push(__wireBoundary_b1(__forAnchor6));
      __e2.appendChild(__e5);
      __e1.appendChild(__e2);
      const __e15 = document.createElement("div");
      __e15.setAttribute("class", "ts-status");
      __e15.setAttribute("aria-live", "polite");
      const __ifAnchor16 = document.createComment("if");
      __e15.appendChild(__ifAnchor16);
      __cleanups.push(__wireBoundary_b5(__ifAnchor16));
      const __ifAnchor21 = document.createComment("if");
      __e15.appendChild(__ifAnchor21);
      __cleanups.push(__wireBoundary_b7(__ifAnchor21));
      const __ifAnchor26 = document.createComment("if");
      __e15.appendChild(__ifAnchor26);
      __cleanups.push(__wireBoundary_b9(__ifAnchor26));
      __e1.appendChild(__e15);
      const __e31 = document.createElement("div");
      __e31.setAttribute("class", "ts-actions");
      const __e32 = document.createElement("button");
      __e32.setAttribute("type", "button");
      __e32.setAttribute("class", "ts-btn ts-btn-primary");
      __e32.setAttribute("data-test", "theme-studio-save");
      __cleanups.push(on(__e32, "click", function() {
        const __h = $scope.savePersist;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t33 = document.createTextNode("Save");
      __e32.appendChild(__t33);
      __e31.appendChild(__e32);
      const __e34 = document.createElement("button");
      __e34.setAttribute("type", "button");
      __e34.setAttribute("class", "ts-btn");
      __e34.setAttribute("data-test", "theme-studio-reset");
      __cleanups.push(on(__e34, "click", function() {
        const __h = $scope.resetToSaved;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t35 = document.createTextNode("Reset");
      __e34.appendChild(__t35);
      __e31.appendChild(__e34);
      const __e36 = document.createElement("button");
      __e36.setAttribute("type", "button");
      __e36.setAttribute("class", "ts-btn");
      __e36.setAttribute("data-test", "theme-studio-reset-pack");
      __cleanups.push(on(__e36, "click", function() {
        const __h = $scope.resetToPack;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t37 = document.createTextNode("Reset to pack");
      __e36.appendChild(__t37);
      __e31.appendChild(__e36);
      const __e38 = document.createElement("button");
      __e38.setAttribute("type", "button");
      __e38.setAttribute("class", "ts-btn");
      __e38.setAttribute("data-test", "theme-studio-export");
      __cleanups.push(on(__e38, "click", function() {
        const __h = $scope.exportJson;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t39 = document.createTextNode("Export");
      __e38.appendChild(__t39);
      __e31.appendChild(__e38);
      const __e40 = document.createElement("label");
      __e40.setAttribute("class", "ts-btn ts-btn-file");
      __e40.setAttribute("for", "ts-file-input");
      const __t41 = document.createTextNode(`
        Import
        `);
      __e40.appendChild(__t41);
      const __e42 = document.createElement("input");
      __e42.setAttribute("id", "ts-file-input");
      __e42.setAttribute("class", "ts-file");
      __e42.setAttribute("type", "file");
      __e42.setAttribute("accept", "application/json");
      __e42.setAttribute("data-test", "theme-studio-import");
      __cleanups.push(on(__e42, "change", function() {
        const __h = (e) => $scope.importFiles(e.target.files);
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e40.appendChild(__e42);
      __e31.appendChild(__e40);
      __e1.appendChild(__e31);
      __e0.appendChild(__e1);
      const __e43 = document.createElement("aside");
      __e43.setAttribute("class", "ts-editor");
      __e43.setAttribute("data-test", "theme-studio-editor");
      const __forAnchor44 = document.createComment("for");
      __e43.appendChild(__forAnchor44);
      __cleanups.push(__wireBoundary_b16(__forAnchor44));
      __e0.appendChild(__e43);
      const __e79 = document.createElement("section");
      __e79.setAttribute("class", "ts-preview");
      __e79.setAttribute("id", "ts-preview");
      __e79.setAttribute("data-test", "theme-studio-preview");
      const __e80 = document.createElement("header");
      __e80.setAttribute("class", "ts-pv-head");
      const __e81 = document.createElement("h2");
      const __t82 = document.createTextNode("Live preview");
      __e81.appendChild(__t82);
      __e80.appendChild(__e81);
      const __e83 = document.createElement("p");
      const __t84 = document.createTextNode("Reflects the active pack + your draft overrides in real time.");
      __e83.appendChild(__t84);
      __e80.appendChild(__e83);
      __e79.appendChild(__e80);
      const __e85 = document.createElement("div");
      __e85.setAttribute("class", "ts-pv-grid");
      const __e86 = document.createElement("article");
      __e86.setAttribute("class", "ts-pv-card");
      const __e87 = document.createElement("h3");
      __e87.setAttribute("class", "ts-pv-h");
      const __t88 = document.createTextNode("Buttons");
      __e87.appendChild(__t88);
      __e86.appendChild(__e87);
      const __e89 = document.createElement("sui-cluster");
      const __e90 = document.createElement("button");
      __e90.setAttribute("type", "button");
      __e90.setAttribute("class", "ts-pv-btn ts-tone-primary");
      const __t91 = document.createTextNode("Primary");
      __e90.appendChild(__t91);
      __e89.appendChild(__e90);
      const __e92 = document.createElement("button");
      __e92.setAttribute("type", "button");
      __e92.setAttribute("class", "ts-pv-btn ts-tone-accent");
      const __t93 = document.createTextNode("Accent");
      __e92.appendChild(__t93);
      __e89.appendChild(__e92);
      const __e94 = document.createElement("button");
      __e94.setAttribute("type", "button");
      __e94.setAttribute("class", "ts-pv-btn ts-tone-destructive");
      const __t95 = document.createTextNode("Destructive");
      __e94.appendChild(__t95);
      __e89.appendChild(__e94);
      __e86.appendChild(__e89);
      __e85.appendChild(__e86);
      const __e96 = document.createElement("article");
      __e96.setAttribute("class", "ts-pv-card");
      const __e97 = document.createElement("h3");
      __e97.setAttribute("class", "ts-pv-h");
      const __t98 = document.createTextNode("Card");
      __e97.appendChild(__t98);
      __e96.appendChild(__e97);
      const __e99 = document.createElement("div");
      __e99.setAttribute("class", "ts-pv-card-body");
      const __e100 = document.createElement("strong");
      const __t101 = document.createTextNode("Card title");
      __e100.appendChild(__t101);
      __e99.appendChild(__e100);
      const __e102 = document.createElement("p");
      __e102.setAttribute("class", "ts-pv-muted");
      const __t103 = document.createTextNode("Content sits on --card, text reads --card-foreground.");
      __e102.appendChild(__t103);
      __e99.appendChild(__e102);
      __e96.appendChild(__e99);
      __e85.appendChild(__e96);
      const __e104 = document.createElement("article");
      __e104.setAttribute("class", "ts-pv-card");
      const __e105 = document.createElement("h3");
      __e105.setAttribute("class", "ts-pv-h");
      const __t106 = document.createTextNode("Input");
      __e105.appendChild(__t106);
      __e104.appendChild(__e105);
      const __e107 = document.createElement("input");
      __e107.setAttribute("type", "text");
      __e107.setAttribute("class", "ts-pv-input");
      __e107.setAttribute("placeholder", "Type here…");
      __e104.appendChild(__e107);
      __e85.appendChild(__e104);
      const __e108 = document.createElement("article");
      __e108.setAttribute("class", "ts-pv-card");
      const __e109 = document.createElement("h3");
      __e109.setAttribute("class", "ts-pv-h");
      const __t110 = document.createTextNode("Dialog");
      __e109.appendChild(__t110);
      __e108.appendChild(__e109);
      const __e111 = document.createElement("div");
      __e111.setAttribute("class", "ts-pv-dialog");
      __e111.setAttribute("role", "dialog");
      __e111.setAttribute("aria-label", "Preview dialog");
      const __e112 = document.createElement("strong");
      const __t113 = document.createTextNode("Heads up");
      __e112.appendChild(__t113);
      __e111.appendChild(__e112);
      const __e114 = document.createElement("p");
      __e114.setAttribute("class", "ts-pv-muted");
      const __t115 = document.createTextNode("Modal sits on --popover with --backdrop scrim behind.");
      __e114.appendChild(__t115);
      __e111.appendChild(__e114);
      const __e116 = document.createElement("div");
      __e116.setAttribute("class", "ts-pv-dialog-actions");
      const __e117 = document.createElement("button");
      __e117.setAttribute("type", "button");
      __e117.setAttribute("class", "ts-pv-btn ts-pv-btn-ghost");
      const __t118 = document.createTextNode("Cancel");
      __e117.appendChild(__t118);
      __e116.appendChild(__e117);
      const __e119 = document.createElement("button");
      __e119.setAttribute("type", "button");
      __e119.setAttribute("class", "ts-pv-btn ts-tone-primary");
      const __t120 = document.createTextNode("Confirm");
      __e119.appendChild(__t120);
      __e116.appendChild(__e119);
      __e111.appendChild(__e116);
      __e108.appendChild(__e111);
      __e85.appendChild(__e108);
      const __e121 = document.createElement("article");
      __e121.setAttribute("class", "ts-pv-card ts-pv-wide");
      const __e122 = document.createElement("h3");
      __e122.setAttribute("class", "ts-pv-h");
      const __t123 = document.createTextNode("Navigation strip");
      __e122.appendChild(__t123);
      __e121.appendChild(__e122);
      const __e124 = document.createElement("nav");
      __e124.setAttribute("class", "ts-pv-nav");
      __e124.setAttribute("aria-label", "Preview nav");
      const __e125 = document.createElement("a");
      __e125.setAttribute("class", "ts-pv-nav-item ts-pv-nav-active");
      const __t126 = document.createTextNode("Dashboard");
      __e125.appendChild(__t126);
      __e124.appendChild(__e125);
      const __e127 = document.createElement("a");
      __e127.setAttribute("class", "ts-pv-nav-item");
      const __t128 = document.createTextNode("Pipelines");
      __e127.appendChild(__t128);
      __e124.appendChild(__e127);
      const __e129 = document.createElement("a");
      __e129.setAttribute("class", "ts-pv-nav-item");
      const __t130 = document.createTextNode("Agents");
      __e129.appendChild(__t130);
      __e124.appendChild(__e129);
      const __e131 = document.createElement("a");
      __e131.setAttribute("class", "ts-pv-nav-item");
      const __t132 = document.createTextNode("Settings");
      __e131.appendChild(__t132);
      __e124.appendChild(__e131);
      __e121.appendChild(__e124);
      __e85.appendChild(__e121);
      const __e133 = document.createElement("article");
      __e133.setAttribute("class", "ts-pv-card");
      const __e134 = document.createElement("h3");
      __e134.setAttribute("class", "ts-pv-h");
      const __t135 = document.createTextNode("Command bar trigger");
      __e134.appendChild(__t135);
      __e133.appendChild(__e134);
      const __e136 = document.createElement("button");
      __e136.setAttribute("type", "button");
      __e136.setAttribute("class", "ts-pv-cmdk");
      const __e137 = document.createElement("span");
      __e137.setAttribute("class", "ts-pv-cmdk-dot");
      __e137.setAttribute("aria-hidden", "true");
      __e136.appendChild(__e137);
      const __e138 = document.createElement("span");
      __e138.setAttribute("class", "ts-pv-cmdk-text");
      const __t139 = document.createTextNode("Search, run, navigate…");
      __e138.appendChild(__t139);
      __e136.appendChild(__e138);
      const __e140 = document.createElement("kbd");
      __e140.setAttribute("class", "ts-pv-kbd");
      const __t141 = document.createTextNode("⌘K");
      __e140.appendChild(__t141);
      __e136.appendChild(__e140);
      __e133.appendChild(__e136);
      __e85.appendChild(__e133);
      const __e142 = document.createElement("article");
      __e142.setAttribute("class", "ts-pv-card ts-pv-wide");
      const __e143 = document.createElement("h3");
      __e143.setAttribute("class", "ts-pv-h");
      const __t144 = document.createTextNode("Toast tones");
      __e143.appendChild(__t144);
      __e142.appendChild(__e143);
      const __e145 = document.createElement("sui-grid");
      const __e146 = document.createElement("div");
      __e146.setAttribute("class", "ts-pv-toast ts-tone-info");
      const __t147 = document.createTextNode("Info — preview tones reflect tone-ramp tokens.");
      __e146.appendChild(__t147);
      __e145.appendChild(__e146);
      const __e148 = document.createElement("div");
      __e148.setAttribute("class", "ts-pv-toast ts-tone-success");
      const __t149 = document.createTextNode("Success — saved.");
      __e148.appendChild(__t149);
      __e145.appendChild(__e148);
      const __e150 = document.createElement("div");
      __e150.setAttribute("class", "ts-pv-toast ts-tone-warning");
      const __t151 = document.createTextNode("Warning — check your inputs.");
      __e150.appendChild(__t151);
      __e145.appendChild(__e150);
      const __e152 = document.createElement("div");
      __e152.setAttribute("class", "ts-pv-toast ts-tone-destructive");
      const __t153 = document.createTextNode("Error — something went wrong.");
      __e152.appendChild(__t153);
      __e145.appendChild(__e152);
      __e142.appendChild(__e145);
      __e85.appendChild(__e142);
      __e79.appendChild(__e85);
      __e0.appendChild(__e79);
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
        __cleanups.push(hydrateOn(__m["b0"], "change", function() {
          const __h = (e) => $scope.selectPack(e.target.value);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b1"]) {
        const __t = __m["b1"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("for") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b1(__anchor));
      }
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
      if (__m["b9"]) {
        const __t = __m["b9"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b9(__anchor));
      }
      if (__m["b11"])
        __cleanups.push(hydrateOn(__m["b11"], "click", function() {
          const __h = $scope.savePersist;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b12"])
        __cleanups.push(hydrateOn(__m["b12"], "click", function() {
          const __h = $scope.resetToSaved;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b13"])
        __cleanups.push(hydrateOn(__m["b13"], "click", function() {
          const __h = $scope.resetToPack;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b14"])
        __cleanups.push(hydrateOn(__m["b14"], "click", function() {
          const __h = $scope.exportJson;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b15"])
        __cleanups.push(hydrateOn(__m["b15"], "change", function() {
          const __h = (e) => $scope.importFiles(e.target.files);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b16"]) {
        const __t = __m["b16"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("for") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b16(__anchor));
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
