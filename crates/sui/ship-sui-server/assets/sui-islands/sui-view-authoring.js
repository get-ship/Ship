// packages/ui-components/src/islands/sui-view-authoring/sui-va-activity.sui
import { adoptSuiSheets, bindAttr, bindFor, bindIf, bindText, component, prop, signal as signal2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-view-authoring/sui-va-activity.controller.ts
import { controller, signal } from "@ship/sui-runtime";
var MAX_ENTRIES = 50;
function formatTs(ms) {
  const d = new Date(ms);
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  const ss = String(d.getSeconds()).padStart(2, "0");
  return `${hh}:${mm}:${ss}`;
}
var sui_va_activity_controller_default = controller({
  scope: () => {
    const entries = signal([]);
    return { entries };
  },
  mount(state, host) {
    const s = state;
    let counter = 0;
    function push(entry) {
      counter += 1;
      const e = {
        ...entry,
        id: `va-${counter}`,
        tsFmt: formatTs(entry.ts)
      };
      const next = [e, ...s.entries()].slice(0, MAX_ENTRIES);
      s.entries.set(next);
    }
    function matchesView(detailView) {
      const v = s.view();
      if (!v)
        return true;
      if (!detailView)
        return true;
      return detailView === v;
    }
    function onAnnotation(e) {
      const d = e.detail;
      if (!d || !matchesView(d.view))
        return;
      push({
        kind: "annotation",
        label: d.label ?? "Annotation",
        detail: `selector: ${d.selector ?? "?"}`,
        ts: d.ts ?? Date.now()
      });
    }
    function onCompiled(e) {
      const d = e.detail;
      if (!d || !matchesView(d.view))
        return;
      push({
        kind: "compiled",
        label: "View compiled",
        detail: d.ms != null ? `${d.ms}ms` : "",
        ts: d.ts ?? Date.now()
      });
    }
    function onErrored(e) {
      const d = e.detail;
      if (!d || !matchesView(d.view))
        return;
      push({
        kind: "errored",
        label: "Compile failed",
        detail: (d.errors ?? []).map((er) => er.message).join(`
`),
        ts: d.ts ?? Date.now()
      });
    }
    document.addEventListener("sui:view:annotation", onAnnotation);
    document.addEventListener("sui:view:compiled", onCompiled);
    document.addEventListener("sui:view:errored", onErrored);
    Object.defineProperty(host, "clear", {
      configurable: true,
      value: () => s.entries.set([])
    });
    return () => {
      document.removeEventListener("sui:view:annotation", onAnnotation);
      document.removeEventListener("sui:view:compiled", onCompiled);
      document.removeEventListener("sui:view:errored", onErrored);
    };
  }
});

// packages/ui-components/src/islands/sui-view-authoring/sui-va-activity.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_va_activity_default = component("sui-va-activity", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  props: {
    view: prop.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: block; height: 100%; overflow: auto; }
  .wrap { display: flex; flex-direction: column; padding: 6px 0; }
  details {
    border-bottom: 1px solid color-mix(in oklab, var(--border) 40%, transparent);
    font-size: 12px;
  }
  summary {
    list-style: none;
    cursor: pointer;
    padding: 6px 14px;
    display: grid;
    grid-template-columns: 20px 1fr auto;
    align-items: center;
    gap: 8px;
    color: var(--foreground);
  }
  summary::-webkit-details-marker { display: none; }
  .dot {
    width: 8px; height: 8px; border-radius: 50%;
    background: var(--muted-foreground);
    justify-self: center;
  }
  .dot[data-kind="annotation"] { background: var(--accent, #f97316); }
  .dot[data-kind="compiled"]   { background: var(--success, #22c55e); }
  .dot[data-kind="errored"]    { background: var(--destructive, #ef4444); }
  .label { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .ts {
    color: var(--muted-foreground);
    font-family: var(--font-mono);
    font-size: 10.5px;
    font-variant-numeric: tabular-nums;
  }
  .detail {
    padding: 4px 16px 10px 42px;
    font-size: 11px;
    color: var(--muted-foreground);
    font-family: var(--font-mono);
    white-space: pre-wrap;
  }
  .empty {
    padding: 18px;
    text-align: center;
    color: var(--muted-foreground);
    font-size: 11.5px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.view = props.view;
    let __r = null;
    if (sui_va_activity_controller_default && typeof sui_va_activity_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_va_activity_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindFor({
      anchor,
      items: () => $scope.entries(),
      key: (entry, __i) => {
        const __child = Object.create($scope);
        __child.entry = entry;
        __child.__i = __i;
        return (($scope2) => entry.id)(__child);
      },
      render: (entry, __i) => {
        const __childScope9 = Object.create($scope);
        __childScope9.entry = entry;
        __childScope9.__i = __i;
        __childScope9.__v = signal2(0);
        return (($scope2) => {
          const __frag10 = document.createDocumentFragment();
          const __c11 = [];
          const __e12 = document.createElement("details");
          {
            const __av13 = $scope2.entry.id;
            if (__av13 == null || __av13 === false || __av13 === "")
              __e12.removeAttribute("data-entry-id");
            else
              __e12.setAttribute("data-entry-id", __av13 === true ? "" : String(__av13));
          }
          {
            const __av14 = $scope2.entry.kind;
            if (__av14 == null || __av14 === false || __av14 === "")
              __e12.removeAttribute("data-kind");
            else
              __e12.setAttribute("data-kind", __av14 === true ? "" : String(__av14));
          }
          const __e15 = document.createElement("summary");
          const __e16 = document.createElement("span");
          __e16.setAttribute("class", "dot");
          {
            const __av17 = $scope2.entry.kind;
            if (__av17 == null || __av17 === false || __av17 === "")
              __e16.removeAttribute("data-kind");
            else
              __e16.setAttribute("data-kind", __av17 === true ? "" : String(__av17));
          }
          __e15.appendChild(__e16);
          const __e18 = document.createElement("span");
          __e18.setAttribute("class", "label");
          const __t19 = document.createTextNode("");
          __e18.appendChild(__t19);
          __t19.textContent = $scope2.entry.label;
          __e15.appendChild(__e18);
          const __e20 = document.createElement("span");
          __e20.setAttribute("class", "ts");
          const __t21 = document.createTextNode("");
          __e20.appendChild(__t21);
          __t21.textContent = $scope2.entry.tsFmt;
          __e15.appendChild(__e20);
          __e12.appendChild(__e15);
          const __e22 = document.createElement("div");
          __e22.setAttribute("class", "detail");
          const __t23 = document.createTextNode("");
          __e22.appendChild(__t23);
          __t23.textContent = $scope2.entry.detail;
          __e12.appendChild(__e22);
          __frag10.appendChild(__e12);
          return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
            for (const c of __c11)
              try {
                c();
              } catch (e) {}
          }, update: (__new24, __newI25) => {
            $scope2.entry = __new24;
            $scope2.__i = __newI25;
            {
              const __av13 = $scope2.entry.id;
              if (__av13 == null || __av13 === false || __av13 === "")
                __e12.removeAttribute("data-entry-id");
              else
                __e12.setAttribute("data-entry-id", __av13 === true ? "" : String(__av13));
            }
            {
              const __av14 = $scope2.entry.kind;
              if (__av14 == null || __av14 === false || __av14 === "")
                __e12.removeAttribute("data-kind");
              else
                __e12.setAttribute("data-kind", __av14 === true ? "" : String(__av14));
            }
            {
              const __av17 = $scope2.entry.kind;
              if (__av17 == null || __av17 === false || __av17 === "")
                __e16.removeAttribute("data-kind");
              else
                __e16.setAttribute("data-kind", __av17 === true ? "" : String(__av17));
            }
            __t19.textContent = $scope2.entry.label;
            __t21.textContent = $scope2.entry.tsFmt;
            __t23.textContent = $scope2.entry.detail;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope9);
      }
    });
    const __wireBoundary_b1 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.entries().length === 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag2 = document.createDocumentFragment();
          const __c3 = [];
          const __e4 = document.createElement("div");
          __e4.setAttribute("class", "empty");
          const __t5 = document.createTextNode("No activity yet. Agent turns will appear here.");
          __e4.appendChild(__t5);
          __frag2.appendChild(__e4);
          return { nodes: Array.from(__frag2.childNodes), cleanup: () => {
            for (const c of __c3)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag6 = document.createDocumentFragment();
          const __c7 = [];
          const __forAnchor8 = document.createComment("for");
          __frag6.appendChild(__forAnchor8);
          __c7.push(__wireBoundary_b2(__forAnchor8));
          return { nodes: Array.from(__frag6.childNodes), cleanup: () => {
            for (const c of __c7)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "wrap");
    __cleanups.push(bindAttr(__e0, "data-entry-count", () => $scope.entries().length));
    const __ifAnchor1 = document.createComment("if");
    __e0.appendChild(__ifAnchor1);
    __cleanups.push(__wireBoundary_b1(__ifAnchor1));
    __root.appendChild(__e0);
    if (__r != null) {
      const __lc = Object.getOwnPropertyDescriptor(__r, "__shipSetLifecycle")?.value;
      if (__lc) {
        const __sp = new Proxy($scope, {
          get(t, k) {
            return t[k];
          },
          set(t, k, v) {
            const s = t[k];
            if (s != null && typeof s.set === "function") {
              if (typeof v === "function") {
                s.set(() => v);
              } else {
                s.set(v);
              }
              return true;
            }
            return false;
          }
        });
        if (__lc.mount) {
          const __mc = __lc.mount(__sp, host);
          if (typeof __mc === "function")
            __cleanups.push(__mc);
        }
        if (__lc.unmount)
          __cleanups.push(() => __lc.unmount(__sp));
      }
    }
    return $scope;
  }
});

// packages/ui-components/src/islands/sui-view-authoring/sui-va-annotation-pins.sui
import { adoptSuiSheets as adoptSuiSheets2, bindAttr as bindAttr2, bindFor as bindFor2, bindText as bindText2, component as component2, on, signal as signal4 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-view-authoring/sui-va-annotation-pins.controller.ts
import { controller as controller2, signal as signal3, chrome } from "@ship/sui-runtime";
var AUTO_CLEAR_MS = 30000;
var sui_va_annotation_pins_controller_default = controller2({
  scope: () => {
    const pins = signal3([]);
    function onPinClick(pin) {
      chrome.dock.show();
      chrome.dock.activate("va-activity");
      document.dispatchEvent(new CustomEvent("sui:va:scroll-activity", {
        detail: { pinId: pin.id, selector: pin.selector },
        bubbles: true
      }));
    }
    return { pins, onPinClick };
  },
  mount(state, host) {
    const s = state;
    let counter = 0;
    let lastTurn = 0;
    const timers = new Map;
    function locate(selector) {
      try {
        const slot = document.getElementById("ship-content");
        if (!slot)
          return null;
        const el = slot.querySelector(selector);
        if (!el)
          return null;
        const r = el.getBoundingClientRect();
        if (r.width === 0 && r.height === 0)
          return null;
        return { x: r.left + r.width / 2, y: r.top };
      } catch {
        return null;
      }
    }
    function removePin(id) {
      const next = s.pins().filter((p) => p.id !== id);
      s.pins.set(next);
      const tid = timers.get(id);
      if (tid != null) {
        window.clearTimeout(tid);
        timers.delete(id);
      }
    }
    function startFade(id) {
      const next = s.pins().map((p) => p.id === id ? { ...p, fading: true } : p);
      s.pins.set(next);
      window.setTimeout(() => removePin(id), 260);
    }
    function clearAllExceptTurn(turn) {
      for (const p of s.pins()) {
        if (p.turn < turn)
          startFade(p.id);
      }
    }
    function onAnnotation(e) {
      const d = e.detail;
      if (!d || !d.selector)
        return;
      const turn = d.turn ?? lastTurn + 1;
      if (turn > lastTurn) {
        clearAllExceptTurn(turn);
        lastTurn = turn;
      }
      const at = locate(d.selector);
      if (!at)
        return;
      counter += 1;
      const id = d.id ?? `pin-${counter}`;
      const pin = {
        id,
        number: counter,
        selector: d.selector,
        label: d.label ?? d.selector,
        turn,
        x: at.x,
        y: at.y,
        fading: false
      };
      s.pins.set([...s.pins().filter((p) => p.id !== id), pin]);
      const tid = window.setTimeout(() => startFade(id), AUTO_CLEAR_MS);
      timers.set(id, tid);
    }
    function reposition() {
      const cur = s.pins();
      if (cur.length === 0)
        return;
      const next = [];
      for (const p of cur) {
        const at = locate(p.selector);
        if (!at)
          continue;
        next.push({ ...p, x: at.x, y: at.y });
      }
      s.pins.set(next);
    }
    document.addEventListener("sui:view:annotation", onAnnotation);
    window.addEventListener("resize", reposition);
    window.addEventListener("scroll", reposition, true);
    document.addEventListener("sui:scope:change", reposition);
    Object.defineProperty(host, "pinCount", {
      configurable: true,
      get: () => s.pins().filter((p) => !p.fading).length
    });
    Object.defineProperty(host, "clear", {
      configurable: true,
      value: () => {
        for (const p of s.pins())
          startFade(p.id);
      }
    });
    return () => {
      document.removeEventListener("sui:view:annotation", onAnnotation);
      window.removeEventListener("resize", reposition);
      window.removeEventListener("scroll", reposition, true);
      document.removeEventListener("sui:scope:change", reposition);
      for (const tid of timers.values())
        window.clearTimeout(tid);
      timers.clear();
    };
  }
});

// packages/ui-components/src/islands/sui-view-authoring/sui-va-annotation-pins.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_va_annotation_pins_default = component2("sui-va-annotation-pins", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  shadow: true,
  styles: `
  :host {
    position: fixed;
    inset: 0;
    pointer-events: none;
    z-index: 7500;
  }

  .pin {
    position: absolute;
    transform: translate(-50%, -100%);
    background: var(--accent, #f97316);
    color: #fff;
    font: 600 11px/1 var(--font-body, system-ui);
    border-radius: 999px;
    min-width: 18px;
    height: 18px;
    padding: 0 6px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px hsl(0 0% 0% / 0.25);
    cursor: pointer;
    pointer-events: auto;
    animation: pin-in 320ms var(--ease-out, ease-out) both;
  }
  .pin[data-fading="true"] {
    animation: pin-out 240ms var(--ease-in, ease-in) both;
  }
  .pin::after {
    content: '';
    position: absolute;
    bottom: -3px;
    left: 50%;
    transform: translateX(-50%) rotate(45deg);
    width: 6px; height: 6px;
    background: inherit;
  }
  @keyframes pin-in {
    0%   { opacity: 0; transform: translate(-50%, -90%) scale(0.6); }
    60%  { opacity: 1; transform: translate(-50%, -104%) scale(1.06); }
    100% { opacity: 1; transform: translate(-50%, -100%) scale(1); }
  }
  @keyframes pin-out {
    to { opacity: 0; transform: translate(-50%, -90%) scale(0.85); }
  }
  @media (prefers-reduced-motion: reduce) {
    .pin, .pin[data-fading="true"] { animation: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_va_annotation_pins_controller_default && typeof sui_va_annotation_pins_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_va_annotation_pins_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.pins(),
      key: (pin, __i) => {
        const __child = Object.create($scope);
        __child.pin = pin;
        __child.__i = __i;
        return (($scope2) => pin.id)(__child);
      },
      render: (pin, __i) => {
        const __childScope1 = Object.create($scope);
        __childScope1.pin = pin;
        __childScope1.__i = __i;
        __childScope1.__v = signal4(0);
        return (($scope2) => {
          const __frag2 = document.createDocumentFragment();
          const __c3 = [];
          const __e4 = document.createElement("button");
          __e4.setAttribute("class", "pin");
          __e4.setAttribute("type", "button");
          {
            const __av5 = $scope2.pin.id;
            if (__av5 == null || __av5 === false || __av5 === "")
              __e4.removeAttribute("data-pin-id");
            else
              __e4.setAttribute("data-pin-id", __av5 === true ? "" : String(__av5));
          }
          {
            const __av6 = $scope2.pin.selector;
            if (__av6 == null || __av6 === false || __av6 === "")
              __e4.removeAttribute("data-selector");
            else
              __e4.setAttribute("data-selector", __av6 === true ? "" : String(__av6));
          }
          {
            const __av7 = $scope2.pin.fading ? "true" : "false";
            if (__av7 == null || __av7 === false || __av7 === "")
              __e4.removeAttribute("data-fading");
            else
              __e4.setAttribute("data-fading", __av7 === true ? "" : String(__av7));
          }
          {
            const __av8 = "left:" + $scope2.pin.x + "px;top:" + $scope2.pin.y + "px";
            if (__av8 == null || __av8 === false || __av8 === "")
              __e4.removeAttribute("style");
            else
              __e4.setAttribute("style", __av8 === true ? "" : String(__av8));
          }
          {
            const __av9 = $scope2.pin.label;
            if (__av9 == null || __av9 === false || __av9 === "")
              __e4.removeAttribute("title");
            else
              __e4.setAttribute("title", __av9 === true ? "" : String(__av9));
          }
          __c3.push(on(__e4, "click", function() {
            const __h = () => $scope2.onPinClick($scope2.pin);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t10 = document.createTextNode("");
          __e4.appendChild(__t10);
          __t10.textContent = $scope2.pin.number;
          __frag2.appendChild(__e4);
          return { nodes: Array.from(__frag2.childNodes), cleanup: () => {
            for (const c of __c3)
              try {
                c();
              } catch (e) {}
          }, update: (__new11, __newI12) => {
            $scope2.pin = __new11;
            $scope2.__i = __newI12;
            {
              const __av5 = $scope2.pin.id;
              if (__av5 == null || __av5 === false || __av5 === "")
                __e4.removeAttribute("data-pin-id");
              else
                __e4.setAttribute("data-pin-id", __av5 === true ? "" : String(__av5));
            }
            {
              const __av6 = $scope2.pin.selector;
              if (__av6 == null || __av6 === false || __av6 === "")
                __e4.removeAttribute("data-selector");
              else
                __e4.setAttribute("data-selector", __av6 === true ? "" : String(__av6));
            }
            {
              const __av7 = $scope2.pin.fading ? "true" : "false";
              if (__av7 == null || __av7 === false || __av7 === "")
                __e4.removeAttribute("data-fading");
              else
                __e4.setAttribute("data-fading", __av7 === true ? "" : String(__av7));
            }
            {
              const __av8 = "left:" + $scope2.pin.x + "px;top:" + $scope2.pin.y + "px";
              if (__av8 == null || __av8 === false || __av8 === "")
                __e4.removeAttribute("style");
              else
                __e4.setAttribute("style", __av8 === true ? "" : String(__av8));
            }
            {
              const __av9 = $scope2.pin.label;
              if (__av9 == null || __av9 === false || __av9 === "")
                __e4.removeAttribute("title");
              else
                __e4.setAttribute("title", __av9 === true ? "" : String(__av9));
            }
            __t10.textContent = $scope2.pin.number;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope1);
      }
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets2(__root);
    const __forAnchor0 = document.createComment("for");
    __root.appendChild(__forAnchor0);
    __cleanups.push(__wireBoundary_b0(__forAnchor0));
    if (__r != null) {
      const __lc = Object.getOwnPropertyDescriptor(__r, "__shipSetLifecycle")?.value;
      if (__lc) {
        const __sp = new Proxy($scope, {
          get(t, k) {
            return t[k];
          },
          set(t, k, v) {
            const s = t[k];
            if (s != null && typeof s.set === "function") {
              if (typeof v === "function") {
                s.set(() => v);
              } else {
                s.set(v);
              }
              return true;
            }
            return false;
          }
        });
        if (__lc.mount) {
          const __mc = __lc.mount(__sp, host);
          if (typeof __mc === "function")
            __cleanups.push(__mc);
        }
        if (__lc.unmount)
          __cleanups.push(() => __lc.unmount(__sp));
      }
    }
    return $scope;
  }
});

// packages/ui-components/src/islands/sui-view-authoring/sui-va-compile.sui
import { adoptSuiSheets as adoptSuiSheets3, bindAttr as bindAttr3, bindFor as bindFor3, bindIf as bindIf2, bindText as bindText3, component as component3, on as on2, prop as prop2, signal as signal6 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-view-authoring/sui-va-compile.controller.ts
import { controller as controller3, signal as signal5, chrome as chrome2 } from "@ship/sui-runtime";
var sui_va_compile_controller_default = controller3({
  scope: (state) => {
    const errors = signal5([]);
    const s = state;
    function onClickRow(err) {
      chrome2.dock.activate("va-source");
      document.dispatchEvent(new CustomEvent("sui:va:goto-line", {
        detail: { view: s.view(), line: err.line, col: err.col },
        bubbles: true
      }));
    }
    return { errors, onClickRow };
  },
  mount(state) {
    const s = state;
    let counter = 0;
    function id() {
      counter += 1;
      return `err-${counter}`;
    }
    function matches(detailView) {
      const v = s.view();
      if (!v)
        return true;
      if (!detailView)
        return true;
      return detailView === v;
    }
    function onErrored(e) {
      const d = e.detail;
      if (!d || !matches(d.view))
        return;
      s.errors.set((d.errors ?? []).map((er) => ({
        id: id(),
        line: er.line ?? 1,
        col: er.col ?? 1,
        severity: er.severity ?? "error",
        message: er.message
      })));
    }
    function onCompiled(e) {
      const d = e.detail;
      if (!matches(d?.view))
        return;
      s.errors.set([]);
    }
    document.addEventListener("sui:view:errored", onErrored);
    document.addEventListener("sui:view:compiled", onCompiled);
    return () => {
      document.removeEventListener("sui:view:errored", onErrored);
      document.removeEventListener("sui:view:compiled", onCompiled);
    };
  }
});

// packages/ui-components/src/islands/sui-view-authoring/sui-va-compile.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_va_compile_default = component3("sui-va-compile", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  props: {
    view: prop2.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: block; height: 100%; overflow: auto; }
  .empty {
    padding: 24px;
    text-align: center;
    color: var(--muted-foreground);
    font-size: 12px;
  }
  .empty .ok {
    color: var(--success, #16a34a);
    font-weight: var(--font-weight-medium, 500);
    display: block;
    margin-bottom: 4px;
  }
  .list { display: flex; flex-direction: column; }
  .row {
    display: grid;
    grid-template-columns: 56px 1fr;
    gap: 8px;
    align-items: baseline;
    padding: 8px 12px;
    border-bottom: 1px solid color-mix(in oklab, var(--border) 40%, transparent);
    cursor: pointer;
    font-size: 12px;
  }
  .row:hover { background: var(--hover-bg, var(--sidebar-accent)); }
  .row .loc {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--muted-foreground);
    font-variant-numeric: tabular-nums;
  }
  .row .msg {
    color: var(--foreground);
    font-family: var(--font-mono);
    white-space: pre-wrap;
  }
  .row[data-severity="warning"] .loc { color: var(--warning, #ca8a04); }
  .row[data-severity="error"]   .loc { color: var(--destructive, #ef4444); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.view = props.view;
    let __r = null;
    if (sui_va_compile_controller_default && typeof sui_va_compile_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_va_compile_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindFor3({
      anchor,
      items: () => $scope.errors(),
      key: (err, __i) => {
        const __child = Object.create($scope);
        __child.err = err;
        __child.__i = __i;
        return (($scope2) => err.id)(__child);
      },
      render: (err, __i) => {
        const __childScope11 = Object.create($scope);
        __childScope11.err = err;
        __childScope11.__i = __i;
        __childScope11.__v = signal6(0);
        return (($scope2) => {
          const __frag12 = document.createDocumentFragment();
          const __c13 = [];
          const __e14 = document.createElement("div");
          __e14.setAttribute("class", "row");
          {
            const __av15 = $scope2.err.severity;
            if (__av15 == null || __av15 === false || __av15 === "")
              __e14.removeAttribute("data-severity");
            else
              __e14.setAttribute("data-severity", __av15 === true ? "" : String(__av15));
          }
          {
            const __av16 = $scope2.err.line;
            if (__av16 == null || __av16 === false || __av16 === "")
              __e14.removeAttribute("data-line");
            else
              __e14.setAttribute("data-line", __av16 === true ? "" : String(__av16));
          }
          __c13.push(on2(__e14, "click", function() {
            const __h = () => $scope2.onClickRow($scope2.err);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e17 = document.createElement("span");
          __e17.setAttribute("class", "loc");
          const __t18 = document.createTextNode("");
          __e17.appendChild(__t18);
          __t18.textContent = $scope2.err.line;
          const __t19 = document.createTextNode(":");
          __e17.appendChild(__t19);
          const __t20 = document.createTextNode("");
          __e17.appendChild(__t20);
          __t20.textContent = $scope2.err.col;
          __e14.appendChild(__e17);
          const __e21 = document.createElement("span");
          __e21.setAttribute("class", "msg");
          const __t22 = document.createTextNode("");
          __e21.appendChild(__t22);
          __t22.textContent = $scope2.err.message;
          __e14.appendChild(__e21);
          __frag12.appendChild(__e14);
          return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
            for (const c of __c13)
              try {
                c();
              } catch (e) {}
          }, update: (__new23, __newI24) => {
            $scope2.err = __new23;
            $scope2.__i = __newI24;
            {
              const __av15 = $scope2.err.severity;
              if (__av15 == null || __av15 === false || __av15 === "")
                __e14.removeAttribute("data-severity");
              else
                __e14.setAttribute("data-severity", __av15 === true ? "" : String(__av15));
            }
            {
              const __av16 = $scope2.err.line;
              if (__av16 == null || __av16 === false || __av16 === "")
                __e14.removeAttribute("data-line");
              else
                __e14.setAttribute("data-line", __av16 === true ? "" : String(__av16));
            }
            __t18.textContent = $scope2.err.line;
            __t20.textContent = $scope2.err.col;
            __t22.textContent = $scope2.err.message;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope11);
      }
    });
    const __wireBoundary_b0 = (anchor) => bindIf2({
      anchor,
      test: () => {
        if ($scope.errors().length === 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("div");
          __e3.setAttribute("class", "empty");
          const __e4 = document.createElement("span");
          __e4.setAttribute("class", "ok");
          const __t5 = document.createTextNode("All clear");
          __e4.appendChild(__t5);
          __e3.appendChild(__e4);
          const __t6 = document.createTextNode(`
    No compile errors for this view.
  `);
          __e3.appendChild(__t6);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag7 = document.createDocumentFragment();
          const __c8 = [];
          const __e9 = document.createElement("div");
          __e9.setAttribute("class", "list");
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
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets3(__root);
    const __ifAnchor0 = document.createComment("if");
    __root.appendChild(__ifAnchor0);
    __cleanups.push(__wireBoundary_b0(__ifAnchor0));
    if (__r != null) {
      const __lc = Object.getOwnPropertyDescriptor(__r, "__shipSetLifecycle")?.value;
      if (__lc) {
        const __sp = new Proxy($scope, {
          get(t, k) {
            return t[k];
          },
          set(t, k, v) {
            const s = t[k];
            if (s != null && typeof s.set === "function") {
              if (typeof v === "function") {
                s.set(() => v);
              } else {
                s.set(v);
              }
              return true;
            }
            return false;
          }
        });
        if (__lc.mount) {
          const __mc = __lc.mount(__sp, host);
          if (typeof __mc === "function")
            __cleanups.push(__mc);
        }
        if (__lc.unmount)
          __cleanups.push(() => __lc.unmount(__sp));
      }
    }
    return $scope;
  }
});

// packages/ui-components/src/islands/sui-view-authoring/sui-va-components.sui
import { adoptSuiSheets as adoptSuiSheets4, bindAttr as bindAttr4, bindFor as bindFor4, bindIf as bindIf3, bindText as bindText4, component as component4, on as on3, prop as prop3, signal as signal8 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-view-authoring/sui-va-components.controller.ts
import { controller as controller4, signal as signal7, effect, chrome as chrome3 } from "@ship/sui-runtime";
function scanComponents(source) {
  const counts = new Map;
  const re = /<(sui-[a-z][a-z0-9-]*)\b/g;
  let m;
  while ((m = re.exec(source)) !== null) {
    const tag = m[1].toLowerCase();
    counts.set(tag, (counts.get(tag) ?? 0) + 1);
  }
  return Array.from(counts.entries()).map(([tag, count]) => ({ tag, count })).sort((a, b) => a.tag.localeCompare(b.tag));
}
function highlightInMount(tag) {
  const slot = document.getElementById("ship-content");
  if (!slot)
    return;
  const matches = slot.querySelectorAll(tag);
  for (const el of matches) {
    el.setAttribute("data-va-highlight", "");
  }
  window.setTimeout(() => {
    for (const el of matches)
      el.removeAttribute("data-va-highlight");
  }, 1400);
}
var sui_va_components_controller_default = controller4({
  scope: () => {
    const inView = signal7([]);
    const highlighted = signal7("");
    function onChipClick(tag) {
      highlighted.set(tag);
      highlightInMount(tag);
      window.setTimeout(() => {
        if (highlighted() === tag)
          highlighted.set("");
      }, 1500);
    }
    function onPicked(e) {
      const tags = e.detail?.tags ?? [];
      if (tags.length === 0)
        return;
      const list = tags.map((t) => `<${t}>`).join(", ");
      chrome3.bar.open("ai", `Add ${list} to the current view.`);
    }
    return { inView, highlighted, onChipClick, onPicked };
  },
  mount(state) {
    const s = state;
    let loadedFor = null;
    async function refresh() {
      const name = s.view();
      if (!name) {
        s.inView.set([]);
        return;
      }
      if (loadedFor === name)
        return;
      loadedFor = name;
      const mcp = window.__viewMcp;
      if (!mcp)
        return;
      try {
        const res = await mcp.readSource(name);
        s.inView.set(scanComponents(res.source));
      } catch {
        s.inView.set([]);
      }
    }
    const stopEffect = effect(() => {
      s.view();
      refresh();
    });
    return () => stopEffect();
  }
});

// packages/ui-components/src/islands/sui-view-authoring/sui-va-components.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_va_components_default = component4("sui-va-components", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  props: {
    view: prop3.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: block; height: 100%; overflow: auto; }
  .wrap { display: flex; flex-direction: column; gap: 14px; padding: 12px; }
  h3 {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin: 0 0 6px 0;
    color: var(--muted-foreground);
    font-weight: var(--font-weight-medium, 500);
  }
  .in-view {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: 8px;
  }
  .chip {
    appearance: none;
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 6px 8px;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    font: inherit;
    font-size: 11px;
    color: var(--foreground);
    text-align: left;
  }
  .chip:hover {
    border-color: color-mix(in oklab, var(--primary) 60%, var(--border));
  }
  .chip[data-active="true"] {
    background: color-mix(in oklab, var(--primary) 14%, var(--background));
    border-color: var(--primary);
  }
  .chip .tag {
    font-family: var(--font-mono);
    color: var(--primary);
    flex: 1;
  }
  .chip .count {
    font-family: var(--font-mono);
    font-size: 10px;
    background: var(--muted);
    color: var(--muted-foreground);
    border-radius: var(--radius-sm);
    padding: 0 5px;
    font-variant-numeric: tabular-nums;
  }
  .empty {
    font-size: 11px;
    color: var(--muted-foreground);
    padding: 8px;
    border: 1px dashed var(--border);
    border-radius: var(--radius-md);
    text-align: center;
  }
  .picker-host {
    height: 320px;
    display: flex;
  }
  sui-component-picker { flex: 1; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.view = props.view;
    let __r = null;
    if (sui_va_components_controller_default && typeof sui_va_components_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_va_components_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindFor4({
      anchor,
      items: () => $scope.inView(),
      key: (entry, __i) => {
        const __child = Object.create($scope);
        __child.entry = entry;
        __child.__i = __i;
        return (($scope2) => entry.tag)(__child);
      },
      render: (entry, __i) => {
        const __childScope13 = Object.create($scope);
        __childScope13.entry = entry;
        __childScope13.__i = __i;
        __childScope13.__v = signal8(0);
        return (($scope2) => {
          const __frag14 = document.createDocumentFragment();
          const __c15 = [];
          const __e16 = document.createElement("button");
          __e16.setAttribute("class", "chip");
          __e16.setAttribute("type", "button");
          {
            const __av17 = $scope2.entry.tag;
            if (__av17 == null || __av17 === false || __av17 === "")
              __e16.removeAttribute("data-tag");
            else
              __e16.setAttribute("data-tag", __av17 === true ? "" : String(__av17));
          }
          __c15.push(bindAttr4(__e16, "data-active", () => (void $scope2.__v(), $scope2.highlighted() === $scope2.entry.tag ? "true" : "false")));
          __c15.push(on3(__e16, "click", function() {
            const __h = () => $scope2.onChipClick($scope2.entry.tag);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e18 = document.createElement("sui-icon");
          __e18.setAttribute("name", "package");
          __e18.setAttribute("size", "sm");
          __e16.appendChild(__e18);
          const __e19 = document.createElement("span");
          __e19.setAttribute("class", "tag");
          const __t20 = document.createTextNode("<");
          __e19.appendChild(__t20);
          const __t21 = document.createTextNode("");
          __e19.appendChild(__t21);
          __t21.textContent = $scope2.entry.tag;
          const __t22 = document.createTextNode(">");
          __e19.appendChild(__t22);
          __e16.appendChild(__e19);
          const __e23 = document.createElement("span");
          __e23.setAttribute("class", "count");
          const __t24 = document.createTextNode("");
          __e23.appendChild(__t24);
          __t24.textContent = $scope2.entry.count;
          __e16.appendChild(__e23);
          __frag14.appendChild(__e16);
          return { nodes: Array.from(__frag14.childNodes), cleanup: () => {
            for (const c of __c15)
              try {
                c();
              } catch (e) {}
          }, update: (__new25, __newI26) => {
            $scope2.entry = __new25;
            $scope2.__i = __newI26;
            {
              const __av17 = $scope2.entry.tag;
              if (__av17 == null || __av17 === false || __av17 === "")
                __e16.removeAttribute("data-tag");
              else
                __e16.setAttribute("data-tag", __av17 === true ? "" : String(__av17));
            }
            __t21.textContent = $scope2.entry.tag;
            __t24.textContent = $scope2.entry.count;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope13);
      }
    });
    const __wireBoundary_b0 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.inView().length === 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("div");
          __e7.setAttribute("class", "empty");
          const __t8 = document.createTextNode("No SUI components detected. Open Source and add one, or pick from below.");
          __e7.appendChild(__t8);
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __e11 = document.createElement("div");
          __e11.setAttribute("class", "in-view");
          const __forAnchor12 = document.createComment("for");
          __e11.appendChild(__forAnchor12);
          __c10.push(__wireBoundary_b1(__forAnchor12));
          __frag9.appendChild(__e11);
          return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
            for (const c of __c10)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets4(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "wrap");
    const __e1 = document.createElement("section");
    const __e2 = document.createElement("h3");
    const __t3 = document.createTextNode("In this view");
    __e2.appendChild(__t3);
    __e1.appendChild(__e2);
    const __ifAnchor4 = document.createComment("if");
    __e1.appendChild(__ifAnchor4);
    __cleanups.push(__wireBoundary_b0(__ifAnchor4));
    __e0.appendChild(__e1);
    const __e27 = document.createElement("section");
    const __e28 = document.createElement("h3");
    const __t29 = document.createTextNode("Add a component");
    __e28.appendChild(__t29);
    __e27.appendChild(__e28);
    const __e30 = document.createElement("div");
    __e30.setAttribute("class", "picker-host");
    const __e31 = document.createElement("sui-component-picker");
    __e31.setAttribute("mode", "inline");
    __e31.setAttribute("select-mode", "single");
    __cleanups.push(on3(__e31, "sui:component:picked", function() {
      const __h = $scope.onPicked;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __t32 = document.createTextNode(`
      `);
    __e31.appendChild(__t32);
    __e30.appendChild(__e31);
    __e27.appendChild(__e30);
    __e0.appendChild(__e27);
    __root.appendChild(__e0);
    if (__r != null) {
      const __lc = Object.getOwnPropertyDescriptor(__r, "__shipSetLifecycle")?.value;
      if (__lc) {
        const __sp = new Proxy($scope, {
          get(t, k) {
            return t[k];
          },
          set(t, k, v) {
            const s = t[k];
            if (s != null && typeof s.set === "function") {
              if (typeof v === "function") {
                s.set(() => v);
              } else {
                s.set(v);
              }
              return true;
            }
            return false;
          }
        });
        if (__lc.mount) {
          const __mc = __lc.mount(__sp, host);
          if (typeof __mc === "function")
            __cleanups.push(__mc);
        }
        if (__lc.unmount)
          __cleanups.push(() => __lc.unmount(__sp));
      }
    }
    return $scope;
  }
});

// packages/ui-components/src/islands/sui-view-authoring/sui-va-source.sui
import { adoptSuiSheets as adoptSuiSheets5, attachRef, bindAttr as bindAttr5, bindProp, bindText as bindText5, component as component5, on as on4, prop as prop4, signal as signal10 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-view-authoring/sui-va-source.controller.ts
import { controller as controller5, signal as signal9, ref, effect as effect2 } from "@ship/sui-runtime";
var sui_va_source_controller_default = controller5({
  scope: (state) => {
    const source = signal9("");
    const dirty = signal9(false);
    const saved = signal9(false);
    const editorEl = ref(null);
    const s = state;
    function onChange(e) {
      const ce = e;
      const next = ce.detail?.value ?? (editorEl()?.value ?? "");
      source.set(next);
      dirty.set(true);
      saved.set(false);
    }
    async function onSave() {
      const name = s.view();
      if (!name)
        return;
      const mcp = window.__viewMcp;
      if (!mcp)
        return;
      const value = editorEl()?.value ?? source();
      try {
        await mcp.writeSource(name, value);
        source.set(value);
        dirty.set(false);
        saved.set(true);
        window.setTimeout(() => saved.set(false), 1500);
      } catch {}
    }
    return { source, dirty, saved, editorEl, onChange, onSave };
  },
  mount(state, host) {
    const s = state;
    let loadedFor = null;
    async function loadSource(name) {
      if (!name)
        return;
      if (loadedFor === name)
        return;
      loadedFor = name;
      const mcp = window.__viewMcp;
      if (!mcp)
        return;
      try {
        const res = await mcp.readSource(name);
        s.source.set(res.source);
        s.dirty.set(false);
        s.saved.set(false);
      } catch {}
    }
    const stopLoadEffect = effect2(() => {
      const v = s.view();
      if (v)
        loadSource(v);
    });
    function onErrored(e) {
      const detail = e.detail;
      if (!detail || detail.view !== s.view())
        return;
      const ed = s.editorEl();
      if (!ed?.setDiagnostics)
        return;
      ed.setDiagnostics((detail.errors ?? []).map((e2) => ({
        line: e2.line,
        message: e2.message,
        severity: e2.severity ?? "error"
      })));
    }
    document.addEventListener("sui:view:errored", onErrored);
    Object.defineProperty(host, "reload", {
      configurable: true,
      value: () => {
        loadedFor = null;
        return loadSource(s.view());
      }
    });
    return () => {
      stopLoadEffect();
      document.removeEventListener("sui:view:errored", onErrored);
    };
  }
});

// packages/ui-components/src/islands/sui-view-authoring/sui-va-source.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_va_source_default = component5("sui-va-source", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  props: {
    view: prop4.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .header {
    display: flex; align-items: center; gap: 8px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border-subtle, var(--border));
    background: var(--sidebar, var(--card));
    font-size: 11px;
    color: var(--muted-foreground);
  }
  .header .path { font-family: var(--font-mono); color: var(--foreground); }
  .header .state {
    margin-left: auto;
    font-family: var(--font-mono);
    font-size: 10.5px;
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    background: var(--muted);
    color: var(--muted-foreground);
  }
  .header .state[data-dirty="true"] {
    background: color-mix(in oklab, var(--warning, #ca8a04) 24%, var(--muted));
    color: var(--foreground);
  }
  .header .state[data-saved="true"] {
    background: color-mix(in oklab, var(--success, #16a34a) 22%, var(--muted));
    color: var(--foreground);
  }
  .editor-host { flex: 1; min-height: 0; display: flex; }
  sui-code-editor { flex: 1; min-height: 0; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.view = props.view;
    $scope["editorEl"] = signal10(null);
    let __r = null;
    if (sui_va_source_controller_default && typeof sui_va_source_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_va_source_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets5(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "header");
    const __e1 = document.createElement("span");
    const __t2 = document.createTextNode("view:");
    __e1.appendChild(__t2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "path");
    const __t4 = document.createTextNode("");
    __e3.appendChild(__t4);
    __cleanups.push(bindText5(__t4, () => $scope.view() || "(no view active)"));
    __e0.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "state");
    __cleanups.push(bindAttr5(__e5, "data-dirty", () => $scope.dirty() ? "true" : "false"));
    __cleanups.push(bindAttr5(__e5, "data-saved", () => $scope.saved() ? "true" : "false"));
    const __t6 = document.createTextNode("");
    __e5.appendChild(__t6);
    __cleanups.push(bindText5(__t6, () => $scope.dirty() ? "unsaved" : $scope.saved() ? "saved" : "clean"));
    __e0.appendChild(__e5);
    __root.appendChild(__e0);
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "editor-host");
    const __e8 = document.createElement("sui-code-editor");
    attachRef($scope, "editorEl", __e8);
    __e8.setAttribute("language", "html");
    __cleanups.push(bindProp(__e8, "value", () => $scope.source()));
    __cleanups.push(on4(__e8, "sui-change", function() {
      const __h = $scope.onChange;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    __cleanups.push(on4(__e8, "sui-save", function() {
      const __h = $scope.onSave;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __t9 = document.createTextNode(`
  `);
    __e8.appendChild(__t9);
    __e7.appendChild(__e8);
    __root.appendChild(__e7);
    if (__r != null) {
      const __lc = Object.getOwnPropertyDescriptor(__r, "__shipSetLifecycle")?.value;
      if (__lc) {
        const __sp = new Proxy($scope, {
          get(t, k) {
            return t[k];
          },
          set(t, k, v) {
            const s = t[k];
            if (s != null && typeof s.set === "function") {
              if (typeof v === "function") {
                s.set(() => v);
              } else {
                s.set(v);
              }
              return true;
            }
            return false;
          }
        });
        if (__lc.mount) {
          const __mc = __lc.mount(__sp, host);
          if (typeof __mc === "function")
            __cleanups.push(__mc);
        }
        if (__lc.unmount)
          __cleanups.push(() => __lc.unmount(__sp));
      }
    }
    return $scope;
  }
});
