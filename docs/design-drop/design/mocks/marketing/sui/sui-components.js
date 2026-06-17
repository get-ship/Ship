// packages/components/primitives/src/sui-accordion/sui-accordion.sui
import { adoptSuiSheets, component, prop } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-accordion/sui-accordion.controller.ts
import { controller, effect } from "@ship/sui-runtime";
var sui_accordion_controller_default = controller({
  mount(state, host) {
    const scope = state;
    const getItems = () => Array.from(host.querySelectorAll(":scope > sui-accordion-item"));
    const getEnabledItems = () => getItems().filter((i) => !i.hasAttribute("disabled"));
    const getOpenValues = () => {
      const v = scope.value();
      return new Set(v ? v.split(",").filter(Boolean) : []);
    };
    const syncItems = () => {
      const open = getOpenValues();
      for (const item of getItems()) {
        const val = item.getAttribute("value") || "";
        if (open.has(val)) {
          item.setAttribute("open", "");
        } else {
          item.removeAttribute("open");
        }
      }
    };
    const stopEffect = effect(() => {
      scope.value();
      syncItems();
    });
    const onToggle = (e) => {
      const ev = e;
      ev.stopPropagation();
      const itemVal = ev.detail?.value;
      const open = getOpenValues();
      if (scope.type() === "single") {
        scope.value.set(open.has(itemVal) ? "" : itemVal);
      } else {
        if (open.has(itemVal)) {
          open.delete(itemVal);
        } else {
          open.add(itemVal);
        }
        scope.value.set(Array.from(open).join(","));
      }
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: scope.value() },
        bubbles: true,
        composed: true
      }));
    };
    const onNav = (e) => {
      const ev = e;
      ev.stopPropagation();
      const enabled = getEnabledItems();
      if (!enabled.length)
        return;
      const current = ev.target?.closest("sui-accordion-item");
      const idx = current ? enabled.indexOf(current) : -1;
      let target;
      switch (ev.detail?.direction) {
        case "next":
          target = enabled[(idx + 1) % enabled.length];
          break;
        case "prev":
          target = enabled[(idx - 1 + enabled.length) % enabled.length];
          break;
        case "first":
          target = enabled[0];
          break;
        case "last":
          target = enabled[enabled.length - 1];
          break;
      }
      if (target) {
        const btn = target.shadowRoot?.querySelector("button");
        btn?.focus();
      }
    };
    host.addEventListener("sui-accordion-item-toggle", onToggle);
    host.addEventListener("sui-accordion-nav", onNav);
    return () => {
      stopEffect();
      host.removeEventListener("sui-accordion-item-toggle", onToggle);
      host.removeEventListener("sui-accordion-nav", onNav);
    };
  }
});

// packages/components/primitives/src/sui-accordion/sui-accordion.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_accordion_default = component("sui-accordion", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    type: prop.string({ default: "single", reflect: true }),
    value: prop.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: block; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.type = props.type;
    $scope.value = props.value;
    let __r = null;
    if (sui_accordion_controller_default && typeof sui_accordion_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_accordion_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets(__root);
    const __e0 = document.createElement("slot");
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

// packages/components/primitives/src/sui-accordion/sui-accordion-item.sui
import { attachRef, bindAttr, signal as signal2, view } from "@ship/sui-runtime";
// packages/sui-dom/src/hydrate/text.ts
import { effect as effect2 } from "@ship/sui-runtime";
var TEXT_OPEN_PREFIX = "sui:text:";
var TEXT_CLOSE = "/sui";
function hydrateText(open, getter) {
  const parent = open.parentNode;
  if (!parent)
    return () => {};
  let close = null;
  for (let s2 = open.nextSibling;s2; s2 = s2.nextSibling) {
    if (s2.nodeType === 8 && s2.data === TEXT_CLOSE) {
      close = s2;
      break;
    }
  }
  if (!close)
    return () => {};
  let s = open.nextSibling;
  while (s && s !== close) {
    const next = s.nextSibling;
    parent.removeChild(s);
    s = next;
  }
  const text = (parent.ownerDocument ?? document).createTextNode("");
  parent.insertBefore(text, close);
  return effect2(() => {
    const v = getter();
    text.data = v == null ? "" : String(v);
  });
}
function collectTextOpens(root) {
  const owner = root.ownerDocument ?? document;
  const walker = owner.createTreeWalker(root, 128);
  const out = Object.create(null);
  let n = walker.currentNode;
  while (n = walker.nextNode()) {
    if (n.nodeType !== 8)
      continue;
    const data = n.data;
    if (!data.startsWith(TEXT_OPEN_PREFIX))
      continue;
    out[data.slice(TEXT_OPEN_PREFIX.length)] = n;
  }
  return out;
}
// packages/sui-dom/src/hydrate/markers.ts
var BOUNDARY_TAGS = new Set([
  "sui-if",
  "sui-for",
  "sui-match",
  "sui-await",
  "sui-while-loading",
  "sui-while-error"
]);
function collectElementMarkers(root) {
  const owner = root.ownerDocument ?? document;
  const walker = owner.createTreeWalker(root, 1);
  const out = Object.create(null);
  let n = walker.currentNode;
  while (n = walker.nextNode()) {
    const el = n;
    if (BOUNDARY_TAGS.has(el.tagName.toLowerCase())) {
      const key = el.getAttribute("key");
      if (key)
        out[key] = el;
      continue;
    }
    for (const attr of Array.from(el.attributes)) {
      const name = attr.name;
      if (name.startsWith("on:") || name.startsWith("attr:") || name === "attr-dynamic" || name.startsWith("class:") || name.startsWith("style:") || name.startsWith("bind:") || name === "mutation") {
        out[attr.value] = el;
      }
    }
  }
  return out;
}
// packages/sui-dom/src/hydrate/attr.ts
import { effect as effect3 } from "@ship/sui-runtime";
function hydrateAttr(el, name, getter) {
  return effect3(() => {
    const v = getter();
    if (v == null || v === false) {
      el.removeAttribute(name);
      return;
    }
    el.setAttribute(name, String(v));
  });
}
// packages/sui-dom/src/hydrate/class.ts
import { effect as effect4 } from "@ship/sui-runtime";
// packages/sui-dom/src/hydrate/style.ts
import { effect as effect5 } from "@ship/sui-runtime";
// packages/sui-dom/src/hydrate/prop.ts
import { bindProp } from "@ship/sui-runtime";
// packages/sui-dom/src/hydrate/on.ts
import { on } from "@ship/sui-runtime";
// packages/sui-dom/src/hydrate/bind-value.ts
import { effect as effect6 } from "@ship/sui-runtime";
function hydrateBindChecked(el, sig) {
  const onChange = () => {
    sig.set(el.checked);
  };
  el.addEventListener("change", onChange);
  const dispose = effect6(() => {
    const next = !!sig();
    if (el.checked !== next)
      el.checked = next;
  });
  return () => {
    el.removeEventListener("change", onChange);
    dispose();
  };
}
// packages/sui-dom/src/hydrate/ref.ts
import { signal } from "@ship/sui-runtime";
function hydrateRef(el, scope, varName) {
  const existing = scope[varName];
  let ref;
  if (isWritableSignal(existing)) {
    existing.set(el);
    ref = existing;
  } else {
    ref = signal(el);
    scope[varName] = ref;
  }
  return () => {
    ref.set(null);
  };
}
function isWritableSignal(v) {
  return typeof v === "function" && "set" in v && typeof v.set === "function";
}
// packages/sui-dom/src/hydrate/while.ts
import { effect as effect7 } from "@ship/sui-runtime";
// packages/components/primitives/src/sui-accordion/sui-accordion-item.controller.ts
import { controller as controller2, effect as effect8 } from "@ship/sui-runtime";
var _seq = 0;
var sui_accordion_item_controller_default = controller2({
  mount(state, host) {
    const scope = state;
    const uid = ++_seq;
    const triggerId = `sui-acc-trigger-${uid}`;
    const contentId = `sui-acc-content-${uid}`;
    const stopIds = effect8(() => {
      const btn = scope.triggerBtn();
      const wrap = scope.contentWrap();
      if (btn) {
        btn.id = triggerId;
        btn.setAttribute("aria-controls", contentId);
      }
      if (wrap) {
        wrap.id = contentId;
        wrap.setAttribute("aria-labelledby", triggerId);
      }
    });
    const stopAriaExpanded = effect8(() => {
      const btn = scope.triggerBtn();
      if (btn)
        btn.setAttribute("aria-expanded", scope.open() ? "true" : "false");
    });
    const onClick = () => {
      if (scope.disabled())
        return;
      host.dispatchEvent(new CustomEvent("sui-accordion-item-toggle", {
        detail: { value: scope.value() },
        bubbles: true,
        composed: true
      }));
    };
    const onKeydown = (e) => {
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        host.dispatchEvent(new CustomEvent("sui-accordion-nav", {
          detail: { direction: e.key === "ArrowDown" ? "next" : "prev" },
          bubbles: true,
          composed: true
        }));
      } else if (e.key === "Home" || e.key === "End") {
        e.preventDefault();
        host.dispatchEvent(new CustomEvent("sui-accordion-nav", {
          detail: { direction: e.key === "Home" ? "first" : "last" },
          bubbles: true,
          composed: true
        }));
      }
    };
    const stopBtn = effect8(() => {
      const btn = scope.triggerBtn();
      if (!btn)
        return;
      btn.addEventListener("click", onClick);
      btn.addEventListener("keydown", onKeydown);
      return () => {
        btn.removeEventListener("click", onClick);
        btn.removeEventListener("keydown", onKeydown);
      };
    });
    return () => {
      stopIds();
      stopAriaExpanded();
      stopBtn();
    };
  }
});

// packages/components/primitives/src/sui-accordion/sui-accordion-item.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_accordion_item_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope["contentWrap"] = signal2(null);
  $scope["triggerBtn"] = signal2(null);
  if (sui_accordion_item_controller_default && typeof sui_accordion_item_controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = sui_accordion_item_controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("button");
      __e0.setAttribute("class", "trigger");
      __e0.setAttribute("part", "trigger");
      attachRef($scope, "triggerBtn", __e0);
      __cleanups.push(bindAttr(__e0, "disabled", () => $scope.disabled()));
      const __e1 = document.createElement("span");
      __e1.setAttribute("class", "trigger-label");
      const __e2 = document.createElement("slot");
      __e2.setAttribute("name", "trigger");
      __e1.appendChild(__e2);
      __e0.appendChild(__e1);
      const __e3 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
      __e3.setAttribute("class", "chevron");
      __e3.setAttribute("part", "chevron");
      __e3.setAttribute("viewBox", "0 0 16 16");
      __e3.setAttribute("fill", "none");
      __e3.setAttribute("stroke", "currentColor");
      __e3.setAttribute("stroke-width", "1.75");
      __e3.setAttribute("stroke-linecap", "round");
      __e3.setAttribute("stroke-linejoin", "round");
      __e3.setAttribute("aria-hidden", "true");
      const __e4 = document.createElementNS("http://www.w3.org/2000/svg", "path");
      __e4.setAttribute("d", "M4 6l4 4 4-4");
      __e3.appendChild(__e4);
      __e0.appendChild(__e3);
      container.appendChild(__e0);
      const __e5 = document.createElement("div");
      __e5.setAttribute("class", "content-wrap");
      __e5.setAttribute("part", "content");
      attachRef($scope, "contentWrap", __e5);
      __e5.setAttribute("role", "region");
      const __e6 = document.createElement("div");
      __e6.setAttribute("class", "content-inner");
      const __e7 = document.createElement("div");
      __e7.setAttribute("class", "content-body");
      const __e8 = document.createElement("slot");
      __e7.appendChild(__e8);
      __e6.appendChild(__e7);
      __e5.appendChild(__e6);
      container.appendChild(__e5);
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
        __cleanups.push(hydrateRef(__m["b0"], $scope, "triggerBtn"));
      if (__m["b1"])
        __cleanups.push(hydrateAttr(__m["b1"], "disabled", () => $scope.disabled()));
      if (__m["b2"])
        __cleanups.push(hydrateRef(__m["b2"], $scope, "contentWrap"));
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/components/primitives/src/sui-alert/sui-alert.sui
import { adoptSuiSheets as adoptSuiSheets2, attachRef as attachRef2, bindAttr as bindAttr2, component as component2, prop as prop2, signal as signal3 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-alert/sui-alert.controller.ts
import { controller as controller3, effect as effect9 } from "@ship/sui-runtime";
var sui_alert_controller_default = controller3({
  mount(state, host) {
    const scope = state;
    const stopDismiss = effect9(() => {
      const btn = scope.closeBtn();
      if (!btn)
        return;
      const onClick = () => {
        if (!scope.dismissible())
          return;
        host.dispatchEvent(new CustomEvent("sui-dismiss", { bubbles: true, composed: true }));
        host.remove();
      };
      btn.addEventListener("click", onClick);
      return () => btn.removeEventListener("click", onClick);
    });
    return () => {
      stopDismiss();
    };
  }
});

// packages/components/primitives/src/sui-alert/sui-alert.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_alert_default = component2("sui-alert", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    tone: prop2.union(["info", "success", "warning", "destructive", "muted"], { default: "info", reflect: true }),
    dismissible: prop2.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --tone: var(--info);
    --tone-fg: var(--info);
    --alert-bg: color-mix(in oklch, var(--tone) 12%, var(--background));
    --alert-border: color-mix(in oklch, var(--tone) 30%, transparent);
  }

  :host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  .box {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius);
    border: 1px solid var(--alert-border);
    background: var(--alert-bg);
    color: var(--tone-fg);
    font-family: var(--font-body);
    font-size: 13px;
    line-height: 1.5;
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .title { font-weight: 600; }

  .body { color: var(--foreground); }

  .actions {
    display: flex;
    gap: 6px;
    margin-top: 6px;
  }

  .close {
    display: none;
    background: transparent;
    border: 0;
    color: inherit;
    cursor: pointer;
    font-size: 14px;
    padding: 2px 6px;
    border-radius: 4px;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }
  :host([dismissible]) .close {
    display: inline-flex;
  }
  .close:hover {
    background: color-mix(in oklch, var(--tone-fg) 12%, transparent);
  }

  ::slotted([slot='icon']) { display: inline-flex; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.dismissible = props.dismissible;
    $scope["closeBtn"] = signal3(null);
    let __r = null;
    if (sui_alert_controller_default && typeof sui_alert_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_alert_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets2(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "box");
    __e0.setAttribute("part", "box");
    __cleanups.push(bindAttr2(__e0, "role", () => $scope.tone() === "destructive" || $scope.tone() === "warning" ? "alert" : "status"));
    __cleanups.push(bindAttr2(__e0, "aria-live", () => $scope.tone() === "destructive" || $scope.tone() === "warning" ? "assertive" : "polite"));
    __e0.setAttribute("aria-atomic", "true");
    const __e1 = document.createElement("span");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "icon");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "content");
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "title");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "title");
    __e4.appendChild(__e5);
    __e3.appendChild(__e4);
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "body");
    const __e7 = document.createElement("slot");
    __e6.appendChild(__e7);
    __e3.appendChild(__e6);
    const __e8 = document.createElement("div");
    __e8.setAttribute("class", "actions");
    const __e9 = document.createElement("slot");
    __e9.setAttribute("name", "actions");
    __e8.appendChild(__e9);
    __e3.appendChild(__e8);
    __e0.appendChild(__e3);
    const __e10 = document.createElement("button");
    __e10.setAttribute("class", "close");
    __e10.setAttribute("part", "close");
    __e10.setAttribute("aria-label", "Dismiss");
    attachRef2($scope, "closeBtn", __e10);
    const __t11 = document.createTextNode("×");
    __e10.appendChild(__t11);
    __e0.appendChild(__e10);
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

// packages/components/primitives/src/sui-alert-dialog/sui-alert-dialog.sui
import { adoptSuiSheets as adoptSuiSheets3, attachRef as attachRef3, bindAttr as bindAttr3, bindText, component as component3, on as on2, prop as prop3, signal as signal4 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-alert-dialog/sui-alert-dialog.controller.ts
import { controller as controller4, derived, effect as effect10, ref } from "@ship/sui-runtime";
var sui_alert_dialog_controller_default = controller4({
  scope: (state, host) => {
    const s = state;
    const dialogRef = ref(null);
    const confirmTone = derived(() => s.tone() === "destructive" ? "destructive" : "default");
    function emit(name) {
      host.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }));
      s.open.set(false);
    }
    function onConfirm() {
      emit("sui-confirm");
    }
    function onCancel() {
      emit("sui-cancel");
    }
    return { dialogRef, confirmTone, onConfirm, onCancel };
  },
  mount(state, host) {
    const scope = state;
    const openSig = scope.open;
    const dialogRef = scope.dialogRef;
    const stopOpenSync = effect10(() => {
      const wantOpen = openSig();
      const dlg = dialogRef();
      if (!dlg)
        return;
      if (wantOpen && typeof dlg.show === "function")
        dlg.show();
      else if (!wantOpen && typeof dlg.close === "function")
        dlg.close();
    });
    const onDialogClose = () => {
      if (openSig())
        openSig.set(false);
    };
    host.addEventListener("sui-close", onDialogClose);
    return () => {
      stopOpenSync();
      host.removeEventListener("sui-close", onDialogClose);
    };
  }
});

// packages/components/primitives/src/sui-alert-dialog/sui-alert-dialog.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_alert_dialog_default = component3("sui-alert-dialog", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    open: prop3.bool({ default: false, reflect: true }),
    tone: prop3.union(["default", "destructive"], { default: "default", reflect: true }),
    "confirm-label": prop3.string({ default: "Confirm" }),
    "cancel-label": prop3.string({ default: "Cancel" })
  },
  shadow: true,
  styles: `
  :host { display: contents; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.tone = props.tone;
    $scope.confirmLabel = props.confirmLabel;
    $scope.cancelLabel = props.cancelLabel;
    $scope["dialogRef"] = signal4(null);
    let __r = null;
    if (sui_alert_dialog_controller_default && typeof sui_alert_dialog_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_alert_dialog_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets3(__root);
    const __e0 = document.createElement("sui-dialog");
    __e0.setAttribute("role-override", "alertdialog");
    __e0.setAttribute("size", "sm");
    __e0.setAttribute("no-close-button", "");
    attachRef3($scope, "dialogRef", __e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("slot", "header");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "title");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("slot");
    __e0.appendChild(__e3);
    const __e4 = document.createElement("span");
    __e4.setAttribute("slot", "footer");
    const __e5 = document.createElement("sui-button");
    __e5.setAttribute("variant", "outline");
    __cleanups.push(on2(__e5, "click", function() {
      const __h = $scope.onCancel;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __t6 = document.createTextNode("");
    __e5.appendChild(__t6);
    __cleanups.push(bindText(__t6, () => $scope.cancelLabel()));
    __e4.appendChild(__e5);
    const __e7 = document.createElement("sui-button");
    __e7.setAttribute("variant", "solid");
    __cleanups.push(bindAttr3(__e7, "tone", () => $scope.confirmTone()));
    __cleanups.push(on2(__e7, "click", function() {
      const __h = $scope.onConfirm;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __t8 = document.createTextNode("");
    __e7.appendChild(__t8);
    __cleanups.push(bindText(__t8, () => $scope.confirmLabel()));
    __e4.appendChild(__e7);
    __e0.appendChild(__e4);
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

// packages/components/primitives/src/sui-app-icon/sui-app-icon.sui
import { adoptSuiSheets as adoptSuiSheets4, attachRef as attachRef4, component as component4, prop as prop4, signal as signal5 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-app-icon/sui-app-icon.controller.ts
import { controller as controller5, effect as effect11 } from "@ship/sui-runtime";
var APP_GLYPH = {
  dashboard: "layout-dashboard",
  pipelines: "workflow",
  agents: "bot",
  skills: "sparkles",
  git: "git-branch",
  collections: "database",
  canvas: "frame",
  settings: "settings",
  chat: "messages-square",
  events: "radio",
  specs: "file-text",
  theme: "palette",
  setup: "rocket",
  overview: "compass",
  mesh: "network"
};
var APP_ALIASES = {
  "theme-studio": "theme"
};
var APP_NAMES = new Set([...Object.keys(APP_GLYPH), ...Object.keys(APP_ALIASES)]);
var sui_app_icon_controller_default = controller5({
  mount(state, host) {
    const scope = state;
    const stopEffect = effect11(() => {
      const name = scope.name();
      const label = scope.label();
      const resolved = APP_ALIASES[name] ?? name;
      const known = APP_NAMES.has(name);
      const glyph = known ? APP_GLYPH[resolved] : "dot";
      const icon = scope.glyphEl();
      if (icon)
        icon.setAttribute("name", glyph);
      if (label) {
        host.setAttribute("role", "img");
        host.setAttribute("aria-label", label);
        host.removeAttribute("aria-hidden");
      } else {
        host.setAttribute("aria-hidden", "true");
        host.removeAttribute("role");
        host.removeAttribute("aria-label");
      }
    });
    return () => stopEffect();
  }
});

// packages/components/primitives/src/sui-app-icon/sui-app-icon.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_app_icon_default = component4("sui-app-icon", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    name: prop4.string({ default: "", reflect: true }),
    variant: prop4.union(["glyph", "tile", "solid"], { default: "glyph", reflect: true }),
    size: prop4.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    radius: prop4.string({ default: "", reflect: true }),
    label: prop4.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    vertical-align: middle;

    --app-accent: var(--foreground);
    --tile-size: 20px;
    --glyph-size: 14px;
    --tile-radius: var(--radius-md);

    width: var(--tile-size);
    height: var(--tile-size);
    color: var(--app-accent);
  }

  :host([name='dashboard'])   { --app-accent: var(--app-accent-dashboard); }
  :host([name='pipelines'])   { --app-accent: var(--app-accent-pipelines); }
  :host([name='agents'])      { --app-accent: var(--app-accent-agents); }
  :host([name='skills'])      { --app-accent: var(--app-accent-skills); }
  :host([name='git'])         { --app-accent: var(--app-accent-git); }
  :host([name='collections']) { --app-accent: var(--app-accent-collections); }
  :host([name='canvas'])      { --app-accent: var(--app-accent-canvas); }
  :host([name='settings'])    { --app-accent: var(--app-accent-settings); }
  :host([name='chat'])        { --app-accent: var(--app-accent-chat); }
  :host([name='events'])      { --app-accent: var(--app-accent-events); }
  :host([name='specs'])       { --app-accent: var(--app-accent-specs); }
  :host([name='theme'])       { --app-accent: var(--app-accent-theme); }
  :host([name='setup'])       { --app-accent: var(--app-accent-setup); }
  :host([name='overview'])    { --app-accent: var(--app-accent-overview); }
  :host([name='mesh'])        { --app-accent: var(--app-accent-mesh); }
  :host([name='theme-studio']) { --app-accent: var(--app-accent-theme); }

  :host([size='sm']) { --tile-size: 16px; --glyph-size: 12px; --tile-radius: var(--radius-sm); }
  :host([size='md']) { --tile-size: 20px; --glyph-size: 14px; --tile-radius: var(--radius-md); }
  :host([size='lg']) { --tile-size: 28px; --glyph-size: 18px; --tile-radius: var(--radius-lg); }

  :host([radius='sm'])   { --tile-radius: var(--radius-sm); }
  :host([radius='md'])   { --tile-radius: var(--radius-md); }
  :host([radius='lg'])   { --tile-radius: var(--radius-lg); }
  :host([radius='full']) { --tile-radius: var(--radius-full); }

  .frame {
    width: 100%;
    height: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--tile-radius);
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                box-shadow var(--duration-fast) var(--ease-out);
  }

  :host(:not([variant])) .frame,
  :host([variant='glyph']) .frame {
    background: transparent;
    color: var(--app-accent);
  }

  :host([variant='tile']) .frame {
    background: color-mix(in oklab, var(--app-accent) 16%, transparent);
    color: var(--app-accent);
  }

  :host([variant='solid']) .frame {
    background: var(--app-accent);
    color: var(--background);
    box-shadow: var(--shadow-xs);
  }

  sui-icon {
    width: var(--glyph-size);
    height: var(--glyph-size);
  }
  ::slotted(*) {
    width: var(--glyph-size);
    height: var(--glyph-size);
    display: block;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.name = props.name;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.radius = props.radius;
    $scope.label = props.label;
    $scope["glyphEl"] = signal5(null);
    let __r = null;
    if (sui_app_icon_controller_default && typeof sui_app_icon_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_app_icon_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets4(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "frame");
    __e0.setAttribute("part", "frame");
    const __e1 = document.createElement("slot");
    const __e2 = document.createElement("sui-icon");
    attachRef4($scope, "glyphEl", __e2);
    __e2.setAttribute("part", "glyph");
    __e2.setAttribute("aria-hidden", "true");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
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

// packages/components/primitives/src/sui-avatar/sui-avatar.sui
import { adoptSuiSheets as adoptSuiSheets5, bindAttr as bindAttr4, bindIf, bindText as bindText2, component as component5, prop as prop5 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-avatar/sui-avatar.controller.ts
import { controller as controller6, effect as effect12, signal as signal6 } from "@ship/sui-runtime";
var sui_avatar_controller_default = controller6({
  scope: () => {
    const errored = signal6(false);
    return { errored };
  },
  mount(state, host) {
    const scope = state;
    let lastSrc = "";
    const stopSrcReset = effect12(() => {
      const src = scope.iconSlug() || scope.src();
      if (src !== lastSrc) {
        lastSrc = src;
        scope.errored.set(false);
      }
    });
    const onImgError = (e) => {
      const tag = e.target?.tagName?.toLowerCase();
      if (tag !== "img")
        return;
      scope.errored.set(true);
    };
    host.shadowRoot?.addEventListener("error", onImgError, true);
    return () => {
      stopSrcReset();
      host.shadowRoot?.removeEventListener("error", onImgError, true);
    };
  }
});

// packages/components/primitives/src/sui-avatar/sui-avatar.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_avatar_default = component5("sui-avatar", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    src: prop5.string({ default: "" }),
    name: prop5.string({ default: "" }),
    initials: prop5.string({ default: "" }),
    "icon-slug": prop5.string({ default: "" }),
    size: prop5.union(["xs", "sm", "md", "lg", "xl"], { default: "md", reflect: true }),
    shape: prop5.union(["round", "square"], { default: "round", reflect: true }),
    tone: prop5.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "muted", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    --sz: 32px;
    --fs: 13px;
    --av-bg: linear-gradient(
      135deg,
      color-mix(in oklch, var(--primary) 70%, transparent),
      color-mix(in oklch, var(--accent) 70%, transparent)
    );
    --av-fg: var(--foreground);
  }

  :host([size='xs']) { --sz: 20px; --fs: 10px; }
  :host([size='sm']) { --sz: 28px; --fs: 11px; }
  :host([size='md']) { --sz: 32px; --fs: 13px; }
  :host([size='lg']) { --sz: 40px; --fs: 15px; }
  :host([size='xl']) { --sz: 56px; --fs: 22px; }

  /* Tone channel for initials background */
  :host([tone='primary'])     { --av-bg: var(--primary);     --av-fg: var(--primary-foreground); }
  :host([tone='accent'])      { --av-bg: var(--accent);      --av-fg: var(--accent-foreground); }
  :host([tone='info'])        { --av-bg: color-mix(in oklch, var(--info) 20%, var(--muted));        --av-fg: var(--info); }
  :host([tone='success'])     { --av-bg: color-mix(in oklch, var(--success) 20%, var(--muted));     --av-fg: var(--success); }
  :host([tone='warning'])     { --av-bg: color-mix(in oklch, var(--warning) 20%, var(--muted));     --av-fg: var(--warning); }
  :host([tone='destructive']) { --av-bg: color-mix(in oklch, var(--destructive) 20%, var(--muted)); --av-fg: var(--destructive); }
  :host([tone='muted'])       { --av-bg: linear-gradient(135deg, color-mix(in oklch, var(--primary) 70%, transparent), color-mix(in oklch, var(--accent) 70%, transparent)); --av-fg: var(--foreground); }
  :host([tone='default'])     { --av-bg: linear-gradient(135deg, color-mix(in oklch, var(--primary) 70%, transparent), color-mix(in oklch, var(--accent) 70%, transparent)); --av-fg: var(--foreground); }

  .root {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--sz);
    height: var(--sz);
    flex: none;
    background: var(--av-bg);
    color: var(--av-fg);
    font-family: var(--font-body);
    font-weight: var(--font-weight-bold);
    font-size: var(--fs);
    letter-spacing: 0.02em;
    text-transform: uppercase;
    line-height: 1;
    overflow: hidden;
    user-select: none;
  }
  :host([shape='round'])  .root { border-radius: 50%; }
  :host([shape='square']) .root { border-radius: var(--radius-sm); }

  img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
    padding: 10%;
    box-sizing: border-box;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.src = props.src;
    $scope.name = props.name;
    $scope.initials = props.initials;
    $scope.iconSlug = props.iconSlug;
    $scope.size = props.size;
    $scope.shape = props.shape;
    $scope.tone = props.tone;
    let __r = null;
    if (sui_avatar_controller_default && typeof sui_avatar_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_avatar_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindIf({
      anchor,
      test: () => {
        if (($scope.iconSlug() || $scope.src()) && !$scope.errored())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag2 = document.createDocumentFragment();
          const __c3 = [];
          const __e4 = document.createElement("img");
          __e4.setAttribute("part", "img");
          __c3.push(bindAttr4(__e4, "src", () => $scope.iconSlug() ? "https://cdn.simpleicons.org/" + $scope.iconSlug() : $scope.src()));
          __c3.push(bindAttr4(__e4, "alt", () => $scope.name()));
          __frag2.appendChild(__e4);
          return { nodes: Array.from(__frag2.childNodes), cleanup: () => {
            for (const c of __c3)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("span");
          __e7.setAttribute("part", "initials");
          const __t8 = document.createTextNode("");
          __e7.appendChild(__t8);
          __c6.push(bindText2(__t8, () => $scope.initials() ? $scope.initials().slice(0, 2).toUpperCase() : $scope.name().trim().slice(0, 1).toUpperCase() || "?"));
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets5(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "root");
    __e0.setAttribute("part", "root");
    __e0.setAttribute("role", "img");
    __cleanups.push(bindAttr4(__e0, "aria-label", () => $scope.name() || $scope.initials() || "avatar"));
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

// packages/components/primitives/src/sui-badge/sui-badge.sui
import { adoptSuiSheets as adoptSuiSheets6, component as component6, prop as prop6 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_badge_default = component6("sui-badge", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    tone: prop6.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop6.union(["solid", "soft", "outline"], { default: "soft", reflect: true }),
    size: prop6.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    pill: prop6.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone: var(--foreground);
    --tone-fg: var(--foreground);
    --tone-contrast: var(--background);
    /* Size channel defaults */
    --bdg-h: 22px;
    --bdg-px: 8px;
    --bdg-fs: 10.5px;
    --bdg-gap: 4px;
    --bdg-icon: 10px;
    /* Shape channel */
    --bdg-radius: var(--radius-sm);
  }

  /* Size matrix */
  :host([size='sm']) { --bdg-h: 18px; --bdg-px: 6px;  --bdg-fs: 9.5px;  --bdg-gap: 3px; --bdg-icon: 8px; }
  :host([size='md']) { --bdg-h: 22px; --bdg-px: 8px;  --bdg-fs: 10.5px; --bdg-gap: 4px; --bdg-icon: 10px; }
  :host([size='lg']) { --bdg-h: 26px; --bdg-px: 10px; --bdg-fs: 11.5px; --bdg-gap: 5px; --bdg-icon: 12px; }

  /* Shape — square default, pill when opted-in */
  :host([pill]) { --bdg-radius: var(--radius-full); }

  .badge {
    display: inline-flex;
    align-items: center;
    gap: var(--bdg-gap);
    min-height: var(--bdg-h);
    padding: 0 var(--bdg-px);
    font-family: var(--font-body);
    font-size: var(--bdg-fs);
    font-weight: var(--font-weight-semibold);
    line-height: 1;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    border-radius: var(--bdg-radius);
    /* soft is the default — base .badge renders as soft */
    background: color-mix(in oklch, var(--tone) 16%, transparent);
    border: 1px solid color-mix(in oklch, var(--tone) 24%, transparent);
    color: var(--tone-fg);
    white-space: nowrap;
    -webkit-font-smoothing: antialiased;
    font-variant-numeric: tabular-nums;
  }

  /* Variant matrix */
  :host([variant='soft']) .badge {
    background: color-mix(in oklch, var(--tone) 16%, transparent);
    border-color: color-mix(in oklch, var(--tone) 24%, transparent);
    color: var(--tone-fg);
  }
  :host([variant='solid']) .badge {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
  }
  :host([variant='outline']) .badge {
    background: transparent;
    border-color: color-mix(in oklch, var(--tone) 50%, transparent);
    color: var(--tone-fg);
  }

  /* Leading slot wrapper — hidden when empty */
  .leading {
    display: none;
    align-items: center;
    justify-content: center;
    width: var(--bdg-icon);
    height: var(--bdg-icon);
    flex-shrink: 0;
    color: currentColor;
  }
  .leading:has(::slotted(*)) { display: inline-flex; }
  ::slotted([slot='leading']) {
    display: inline-flex;
    align-items: center;
    color: currentColor;
    width: 100%;
    height: 100%;
  }

  @media (forced-colors: active) {
    .badge { border-color: CanvasText; color: CanvasText; }
    :host([variant='solid']) .badge { background: Highlight; color: HighlightText; border-color: Highlight; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.pill = props.pill;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets6(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "badge");
    __e0.setAttribute("part", "badge");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "leading");
    __e1.setAttribute("part", "leading");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "leading");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("slot");
    __e0.appendChild(__e3);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-breadcrumb/sui-breadcrumb.sui
import { adoptSuiSheets as adoptSuiSheets7, bindAttr as bindAttr5, bindFor, bindIf as bindIf2, bindText as bindText3, component as component7, prop as prop7, signal as signal8 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-breadcrumb/sui-breadcrumb.controller.ts
import { controller as controller7, signal as signal7 } from "@ship/sui-runtime";
var sui_breadcrumb_controller_default = controller7({
  scope: () => {
    const items = signal7([]);
    const lastSlot = signal7("");
    return { items, lastSlot };
  },
  mount(state, host) {
    const sync = () => {
      const children = Array.from(host.querySelectorAll(":scope > sui-breadcrumb-item"));
      const next = children.map((el, i) => {
        const slot = `i${i}`;
        el.setAttribute("slot", slot);
        return { slot };
      });
      state.lastSlot.set(next.length > 0 ? next[next.length - 1].slot : "");
      state.items.set(next);
    };
    sync();
    const observer = new MutationObserver(sync);
    observer.observe(host, { childList: true });
    return () => observer.disconnect();
  }
});

// packages/components/primitives/src/sui-breadcrumb/sui-breadcrumb.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_breadcrumb_default = component7("sui-breadcrumb", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    separator: prop7.string({ default: "/", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: flex; align-items: center; }
  nav { display: inline-flex; align-items: center; flex-wrap: wrap; gap: 2px; }
  .sep {
    display: inline-flex; align-items: center; justify-content: center;
    width: 12px; height: 12px;
    color: var(--muted-foreground);
    opacity: 0.7;
  }
  .sep svg { width: 12px; height: 12px; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.separator = props.separator;
    let __r = null;
    if (sui_breadcrumb_controller_default && typeof sui_breadcrumb_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_breadcrumb_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindFor({
      anchor,
      items: () => $scope.items(),
      key: (item, __i) => {
        const __child = Object.create($scope);
        __child.item = item;
        __child.__i = __i;
        return (($scope2) => item.slot)(__child);
      },
      render: (item, __i) => {
        const __childScope2 = Object.create($scope);
        __childScope2.item = item;
        __childScope2.__i = __i;
        return (($scope2) => {
          const __wireBoundary_b3 = (anchor2) => bindIf2({
            anchor: anchor2,
            test: () => {
              if ($scope2.separator() === "/")
                return 0;
              return 1;
            },
            branches: [
              () => {
                const __frag12 = document.createDocumentFragment();
                const __c13 = [];
                const __e14 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
                __e14.setAttribute("viewBox", "0 0 24 24");
                __e14.setAttribute("fill", "none");
                __e14.setAttribute("stroke", "currentColor");
                __e14.setAttribute("stroke-width", "2");
                __e14.setAttribute("stroke-linecap", "round");
                __e14.setAttribute("stroke-linejoin", "round");
                const __e15 = document.createElementNS("http://www.w3.org/2000/svg", "path");
                __e15.setAttribute("d", "M9 6l6 6-6 6");
                __e14.appendChild(__e15);
                __frag12.appendChild(__e14);
                return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
                  for (const c of __c13)
                    try {
                      c();
                    } catch (e) {}
                } };
              },
              () => {
                const __frag16 = document.createDocumentFragment();
                const __c17 = [];
                const __t18 = document.createTextNode("");
                __frag16.appendChild(__t18);
                __c17.push(bindText3(__t18, () => $scope2.separator()));
                return { nodes: Array.from(__frag16.childNodes), cleanup: () => {
                  for (const c of __c17)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __wireBoundary_b2 = (anchor2) => bindIf2({
            anchor: anchor2,
            test: () => {
              if ($scope2.item.slot !== $scope2.lastSlot())
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag8 = document.createDocumentFragment();
                const __c9 = [];
                const __e10 = document.createElement("span");
                __e10.setAttribute("class", "sep");
                __e10.setAttribute("aria-hidden", "true");
                const __ifAnchor11 = document.createComment("if");
                __e10.appendChild(__ifAnchor11);
                __c9.push(__wireBoundary_b3(__ifAnchor11));
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
          const __frag3 = document.createDocumentFragment();
          const __c4 = [];
          const __e5 = document.createElement("slot");
          {
            const __av6 = $scope2.item.slot;
            if (__av6 == null || __av6 === false || __av6 === "")
              __e5.removeAttribute("name");
            else
              __e5.setAttribute("name", __av6 === true ? "" : String(__av6));
          }
          __frag3.appendChild(__e5);
          const __ifAnchor7 = document.createComment("if");
          __frag3.appendChild(__ifAnchor7);
          __c4.push(__wireBoundary_b2(__ifAnchor7));
          return { nodes: Array.from(__frag3.childNodes), cleanup: () => {
            for (const c of __c4)
              try {
                c();
              } catch (e) {}
          }, update: (__new19, __newI20) => {
            const __old = $scope2.item;
            $scope2.item = __new19;
            $scope2.__i = __newI20;
            if (__old.slot !== $scope2.item.slot) {
              {
                const __av6 = $scope2.item.slot;
                if (__av6 == null || __av6 === false || __av6 === "")
                  __e5.removeAttribute("name");
                else
                  __e5.setAttribute("name", __av6 === true ? "" : String(__av6));
              }
            }
          } };
        })(__childScope2);
      }
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets7(__root);
    const __e0 = document.createElement("nav");
    __e0.setAttribute("aria-label", "Breadcrumb");
    __e0.setAttribute("part", "nav");
    const __forAnchor1 = document.createComment("for");
    __e0.appendChild(__forAnchor1);
    __cleanups.push(__wireBoundary_b0(__forAnchor1));
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

// packages/components/primitives/src/sui-breadcrumb/sui-breadcrumb-item.sui
import { bindAttr as bindAttr6, bindIf as bindIf3, view as view2 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_breadcrumb_item_default = Object.assign(view2((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  const __wireBoundary_b0 = (anchor) => bindIf3({
    anchor,
    test: () => {
      if ($scope.href() && !$scope.active())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag1 = document.createDocumentFragment();
        const __c2 = [];
        const __e3 = document.createElement("a");
        __c2.push(bindAttr6(__e3, "href", () => $scope.href()));
        __e3.setAttribute("part", "link");
        const __e4 = document.createElement("slot");
        __e3.appendChild(__e4);
        __frag1.appendChild(__e3);
        return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
          for (const c of __c2)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag5 = document.createDocumentFragment();
        const __c6 = [];
        const __e7 = document.createElement("span");
        __e7.setAttribute("class", "label");
        __e7.setAttribute("part", "label");
        __c6.push(bindAttr6(__e7, "aria-current", () => $scope.active() ? "page" : "false"));
        const __e8 = document.createElement("slot");
        __e7.appendChild(__e8);
        __frag5.appendChild(__e7);
        return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
          for (const c of __c6)
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
      const __ifAnchor0 = document.createComment("if");
      container.appendChild(__ifAnchor0);
      __cleanups.push(__wireBoundary_b0(__ifAnchor0));
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
      if (__m["b0"]) {
        const __t = __m["b0"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b0(__anchor));
      }
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/components/primitives/src/sui-button/sui-button.sui
import { adoptSuiSheets as adoptSuiSheets8, bindAttr as bindAttr7, bindIf as bindIf4, bindSlotEmpty, component as component8, prop as prop8 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_button_default = component8("sui-button", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    tone: prop8.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop8.union(["solid", "soft", "outline", "ghost", "default"], { default: "default", reflect: true }),
    size: prop8.union(["xs", "sm", "md", "lg"], { default: "md", reflect: true }),
    type: prop8.union(["button", "submit", "reset"], { default: "button", reflect: true }),
    disabled: prop8.bool({ default: false, reflect: true }),
    loading: prop8.bool({ default: false, reflect: true }),
    label: prop8.string({ default: "" }),
    pressed: prop8.string({ default: "", reflect: true }),
    expanded: prop8.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
    :host {
      display: inline-flex;
      vertical-align: middle;
      --tone: var(--foreground);
      --tone-fg: var(--foreground);
      --tone-contrast: var(--background);
      --btn-h: 34px;
      --btn-px: var(--space-3);
      --btn-fs: 13px;
      --btn-fw: var(--font-weight-medium, 500);
      --btn-gap: 8px;
    }

    /* Size matrix */
    :host([size='xs']) { --btn-h: 22px; --btn-fs: 11px; --btn-px: 8px;  --btn-gap: 4px; }
    :host([size='sm']) { --btn-h: 28px; --btn-fs: 12px; --btn-px: 10px; --btn-gap: 6px; }
    :host([size='md']) { --btn-h: 34px; --btn-fs: 13px; --btn-px: 12px; --btn-gap: 8px; }
    :host([size='lg']) { --btn-h: 40px; --btn-fs: 14px; --btn-px: 16px; --btn-gap: 8px; }

    button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: var(--btn-gap);
      min-height: var(--btn-h);
      padding: 0 var(--btn-px);
      font-family: var(--font-body);
      font-weight: var(--btn-fw);
      font-size: var(--btn-fs);
      line-height: 1;
      letter-spacing: -0.005em;
      border-radius: var(--radius-md);
      border: 1px solid transparent;
      background: transparent;
      color: var(--foreground);
      cursor: pointer;
      user-select: none;
      white-space: nowrap;
      isolation: isolate;
      position: relative;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }

    /* Default (no variant attr) — neutral bordered button. */
    :host(:not([variant])) button,
    :host([variant='default']) button {
      background: var(--background);
      border-color: var(--border);
      color: var(--foreground);
      box-shadow: var(--shadow-xs);
    }
    :host(:not([variant])) button:hover:not(:disabled),
    :host([variant='default']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--muted) 70%, var(--background));
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
      transform: translateY(-1px);
    }

    /* Solid */
    :host([variant='solid']) button {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      color: var(--tone-contrast);
      box-shadow: var(--shadow-xs);
    }
    :host([variant='solid']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      transform: translateY(-1px);
      box-shadow: var(--shadow-sm);
    }

    /* Soft */
    :host([variant='soft']) button {
      background: color-mix(in oklch, var(--tone) 14%, transparent);
      border-color: color-mix(in oklch, var(--tone) 22%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='soft']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 22%, transparent);
    }

    /* Outline */
    :host([variant='outline']) button {
      background: transparent;
      border-color: color-mix(in oklch, var(--tone) 55%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='outline']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 10%, transparent);
    }

    /* Ghost */
    :host([variant='ghost']) button {
      background: transparent;
      border-color: transparent;
      color: var(--tone-fg);
    }
    :host([variant='ghost']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 12%, transparent);
    }

    /* Active press — feels clicked across all variants. */
    :host(:not([disabled]):not([loading])) button:active:not(:disabled) {
      transform: translateY(1px) scale(0.98);
    }
    @media (prefers-reduced-motion: reduce) {
      button { transition: none; }
      button:active { transform: none !important; }
    }

    /* Focus ring */
    button:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 2px var(--background),
        0 0 0 4px color-mix(in oklch, var(--tone) 55%, transparent);
    }

    /* Disabled */
    button:disabled {
      cursor: not-allowed;
      color: color-mix(in oklch, var(--muted-foreground) 65%, transparent);
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 80%, transparent);
      box-shadow: none;
      transform: none;
      filter: saturate(0.6);
    }
    :host([variant='ghost']) button:disabled,
    :host([variant='outline']) button:disabled {
      background: transparent;
    }

    /* Loading state — fade the label, overlay a spinner. */
    :host([loading]) button { cursor: progress; }
    :host([loading]) .content { opacity: 0; }
    .content {
      display: inline-flex;
      align-items: center;
      gap: var(--btn-gap);
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    .spinner-wrap {
      position: absolute;
      inset: 0;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      pointer-events: none;
    }
    .spinner {
      width: 14px;
      height: 14px;
      border: 1.5px solid currentColor;
      border-right-color: transparent;
      border-radius: 50%;
      animation: sui-button-spin 0.7s linear infinite;
    }
    @keyframes sui-button-spin { to { transform: rotate(360deg); } }

    /* Icon-only — collapse padding to a square when default slot is empty. */
    :host([data-icon-only]) button {
      padding-inline: 0;
      width: var(--btn-h);
      min-width: var(--btn-h);
    }

    ::slotted([slot='leading']),
    ::slotted([slot='trailing']) {
      display: inline-flex;
      align-items: center;
      flex: 0 0 auto;
    }
  `,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.type = props.type;
    $scope.disabled = props.disabled;
    $scope.loading = props.loading;
    $scope.label = props.label;
    $scope.pressed = props.pressed;
    $scope.expanded = props.expanded;
    const __wireBoundary_b7 = (anchor) => bindIf4({
      anchor,
      test: () => {
        if ($scope.loading())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag6 = document.createDocumentFragment();
          const __c7 = [];
          const __e8 = document.createElement("span");
          __e8.setAttribute("class", "spinner-wrap");
          __e8.setAttribute("aria-hidden", "true");
          const __e9 = document.createElement("span");
          __e9.setAttribute("class", "spinner");
          __e8.appendChild(__e9);
          __frag6.appendChild(__e8);
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
    adoptSuiSheets8(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("part", "button");
    __cleanups.push(bindAttr7(__e0, "type", () => $scope.type()));
    __cleanups.push(bindAttr7(__e0, "disabled", () => $scope.disabled() || $scope.loading()));
    __cleanups.push(bindAttr7(__e0, "aria-busy", () => $scope.loading() ? "true" : "false"));
    __cleanups.push(bindAttr7(__e0, "aria-disabled", () => $scope.disabled() || $scope.loading() ? "true" : "false"));
    __cleanups.push(bindAttr7(__e0, "aria-label", () => $scope.label()));
    __cleanups.push(bindAttr7(__e0, "aria-pressed", () => $scope.pressed()));
    __cleanups.push(bindAttr7(__e0, "aria-expanded", () => $scope.expanded()));
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "content");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "leading");
    __e1.appendChild(__e2);
    const __e3 = document.createElement("slot");
    __cleanups.push(bindSlotEmpty(__e3, host, "data-icon-only"));
    __e1.appendChild(__e3);
    const __e4 = document.createElement("slot");
    __e4.setAttribute("name", "trailing");
    __e1.appendChild(__e4);
    __e0.appendChild(__e1);
    const __ifAnchor5 = document.createComment("if");
    __e0.appendChild(__ifAnchor5);
    __cleanups.push(__wireBoundary_b7(__ifAnchor5));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-card/sui-card.sui
import { adoptSuiSheets as adoptSuiSheets9, bindSlotEmpty as bindSlotEmpty2, component as component9, prop as prop9 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-card/sui-card.controller.ts
import { controller as controller8, effect as effect13 } from "@ship/sui-runtime";
var sui_card_controller_default = controller8({
  mount(state, host) {
    const scope = state;
    const onKeydown = (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        host.click();
      }
    };
    const stopSync = effect13(() => {
      if (scope.interactive()) {
        if (!host.hasAttribute("role"))
          host.setAttribute("role", "button");
        if (!host.hasAttribute("tabindex"))
          host.setAttribute("tabindex", "0");
        host.addEventListener("keydown", onKeydown);
        return () => host.removeEventListener("keydown", onKeydown);
      } else {
        if (host.getAttribute("role") === "button")
          host.removeAttribute("role");
        if (host.getAttribute("tabindex") === "0")
          host.removeAttribute("tabindex");
      }
    });
    return () => {
      stopSync();
    };
  }
});

// packages/components/primitives/src/sui-card/sui-card.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_card_default = component9("sui-card", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    tone: prop9.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop9.union(["default", "outline"], { default: "default", reflect: true }),
    padding: prop9.union(["none", "sm", "md", "lg"], { default: "md", reflect: true }),
    interactive: prop9.bool({ default: false, reflect: true }),
    "accent-bar": prop9.bool({ default: false, reflect: true }),
    "status-rail": prop9.bool({ default: false, reflect: true }),
    fill: prop9.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    position: relative;
    background: var(--card);
    color: var(--card-foreground, var(--foreground));
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-card-rest, var(--shadow-xs));
    --card-px: 20px;
    --card-py: 16px;
    --card-gap: var(--space-3, 12px);
    --tone-fg: var(--foreground);
    --tone-border: var(--border);
    --tone-bg: transparent;
    padding: var(--card-py) var(--card-px);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  :host([tone='primary'])     { --tone-border: color-mix(in oklch, var(--primary) 40%, var(--border));     --tone-bg: color-mix(in oklch, var(--primary) 8%, var(--card)); }
  :host([tone='info'])        { --tone-border: color-mix(in oklch, var(--info) 40%, var(--border));        --tone-bg: color-mix(in oklch, var(--info) 8%, var(--card)); }
  :host([tone='success'])     { --tone-border: color-mix(in oklch, var(--success) 40%, var(--border));     --tone-bg: color-mix(in oklch, var(--success) 8%, var(--card)); }
  :host([tone='warning'])     { --tone-border: color-mix(in oklch, var(--warning) 40%, var(--border));     --tone-bg: color-mix(in oklch, var(--warning) 8%, var(--card)); }
  :host([tone='destructive']) { --tone-border: color-mix(in oklch, var(--destructive) 40%, var(--border)); --tone-bg: color-mix(in oklch, var(--destructive) 8%, var(--card)); }
  :host([tone='accent'])      { --tone-border: color-mix(in oklch, var(--accent) 40%, var(--border));      --tone-bg: color-mix(in oklch, var(--accent) 8%, var(--card)); }
  :host([tone='muted'])       { --tone-border: var(--border); --tone-bg: var(--muted); }

  :host([tone]:not([tone='default'])) { border-color: var(--tone-border); }
  :host([fill]) { background: var(--tone-bg); border-color: var(--tone-border); }

  :host([padding='none']) { --card-px: 0px; --card-py: 0px; --card-gap: 0px; padding: 0; }
  :host([padding='sm'])   { --card-px: 12px; --card-py: 10px; --card-gap: 8px; padding: 10px 12px; }
  :host([padding='md'])   { --card-px: 20px; --card-py: 16px; padding: 16px 20px; }
  :host([padding='lg'])   { --card-px: 28px; --card-py: 24px; --card-gap: 16px; padding: 24px 28px; }

  :host([variant='outline']) { background: transparent; box-shadow: none; }

  :host([status-rail]) {
    border-left: 3px solid var(--tone-fg);
    padding-left: calc(var(--card-px) - 3px);
  }

  :host([accent-bar])::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 2px;
    background: linear-gradient(90deg, var(--primary), var(--accent));
    opacity: 0;
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    transition: opacity var(--duration-medium) var(--ease-out);
    pointer-events: none;
  }
  :host([accent-bar]:hover)::before { opacity: 1; }

  :host([interactive]) { cursor: pointer; -webkit-tap-highlight-color: transparent; }
  :host([interactive]:hover) {
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-hover, var(--shadow-sm));
    transform: translateY(-1px);
  }
  @media (prefers-reduced-motion: reduce) {
    :host([interactive]:hover) { transform: none; }
  }
  :host([interactive]:active) {
    box-shadow: var(--shadow-card-rest, var(--shadow-xs));
    transform: translateY(0);
    transition-duration: 60ms;
  }
  :host([interactive]:focus-visible) {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }

  .header, .footer { display: block; }

  /* Separator — show only when slot has projected content */
  :host(:not([data-no-header])) .header {
    margin: calc(var(--card-py) * -1) calc(var(--card-px) * -1) var(--card-gap);
    padding: var(--card-py) var(--card-px);
    border-bottom: 1px solid var(--border);
    font-family: var(--heading-font);
    font-weight: var(--heading-weight, 600);
  }
  :host(:not([data-no-footer])) .footer {
    margin: var(--card-gap) calc(var(--card-px) * -1) calc(var(--card-py) * -1);
    padding: var(--card-py) var(--card-px);
    border-top: 1px solid var(--border);
  }
  :host([padding='none']:not([data-no-header])) .header {
    margin: 0 0 var(--space-3, 12px);
    padding: 0 0 var(--space-3, 12px);
  }
  :host([padding='none']:not([data-no-footer])) .footer {
    margin: var(--space-3, 12px) 0 0;
    padding: var(--space-3, 12px) 0 0;
  }

  @media (forced-colors: active) {
    :host([interactive]:focus-visible) { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.padding = props.padding;
    $scope.interactive = props.interactive;
    $scope.accentBar = props.accentBar;
    $scope.statusRail = props.statusRail;
    $scope.fill = props.fill;
    let __r = null;
    if (sui_card_controller_default && typeof sui_card_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_card_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets9(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "header");
    __e0.setAttribute("part", "header");
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "header");
    __cleanups.push(bindSlotEmpty2(__e1, host, "data-no-header"));
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "content");
    __e2.setAttribute("part", "content");
    const __e3 = document.createElement("slot");
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "footer");
    __e4.setAttribute("part", "footer");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "footer");
    __cleanups.push(bindSlotEmpty2(__e5, host, "data-no-footer"));
    __e4.appendChild(__e5);
    __root.appendChild(__e4);
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

// packages/components/primitives/src/sui-center/sui-center.sui
import { adoptSuiSheets as adoptSuiSheets10, component as component10, prop as prop10 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_center_default = component10("sui-center", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    max: prop10.union(["prose", "form", "wide", "full"], { default: "prose", reflect: true }),
    padded: prop10.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --center-max: 65ch;
    --center-pad: 0;
  }

  :host([max='prose']) { --center-max: 65ch; }
  :host([max='form'])  { --center-max: 28rem; }
  :host([max='wide'])  { --center-max: 80ch; }
  :host([max='full'])  { --center-max: 100%; }

  :host([padded]) { --center-pad: var(--space-4); }

  .center {
    box-sizing: content-box;
    max-width: var(--center-max);
    margin-inline: auto;
    padding-inline: var(--center-pad);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.max = props.max;
    $scope.padded = props.padded;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets10(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "center");
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-checkbox/sui-checkbox.sui
import { adoptSuiSheets as adoptSuiSheets11, attachRef as attachRef5, bindAttr as bindAttr8, bindChecked, component as component11, prop as prop11, signal as signal9 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-checkbox/sui-checkbox.controller.ts
import { controller as controller9, effect as effect14 } from "@ship/sui-runtime";
var sui_checkbox_controller_default = controller9({
  mount(state, host) {
    const scope = state;
    const stopIndet = effect14(() => {
      const el = scope.inputEl();
      if (!el)
        return;
      el.indeterminate = scope.indeterminate();
    });
    const stopChange = effect14(() => {
      const el = scope.inputEl();
      if (!el)
        return;
      const onNativeChange = () => {
        scope.indeterminate.set(false);
        host.dispatchEvent(new CustomEvent("sui-change", {
          detail: { checked: el.checked, value: scope.value() },
          bubbles: true,
          composed: true
        }));
      };
      el.addEventListener("change", onNativeChange);
      return () => el.removeEventListener("change", onNativeChange);
    });
    return () => {
      stopIndet();
      stopChange();
    };
  }
});

// packages/components/primitives/src/sui-checkbox/sui-checkbox.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_checkbox_default = component11("sui-checkbox", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    checked: prop11.bool({ default: false, reflect: true }),
    indeterminate: prop11.bool({ default: false, reflect: true }),
    disabled: prop11.bool({ default: false, reflect: true }),
    required: prop11.bool({ default: false, reflect: true }),
    invalid: prop11.bool({ default: false, reflect: true }),
    tone: prop11.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop11.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    name: prop11.string({ default: "" }),
    value: prop11.string({ default: "on" }),
    label: prop11.string({ default: "" })
  },
  shadow: true,
  formAssociated: true,
  formValue: "checked",
  formDisabled: "disabled",
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    vertical-align: middle;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);
    -webkit-tap-highlight-color: transparent;

    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }
  :host([tone='info'])        { --tone: var(--info);        --tone-on-fg: var(--info-foreground); }
  :host([tone='success'])     { --tone: var(--success);     --tone-on-fg: var(--success-foreground); }
  :host([tone='warning'])     { --tone: var(--warning);     --tone-on-fg: var(--warning-foreground); }
  :host([tone='destructive']) { --tone: var(--destructive); --tone-on-fg: var(--destructive-foreground); }
  :host([tone='accent'])      { --tone: var(--accent);      --tone-on-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }
  :host([tone='primary'])     { --tone: var(--primary);     --tone-on-fg: var(--primary-foreground); }

  :host(:not([size])),
  :host([size='md']) { --cb-size: 16px; --cb-radius: var(--radius-sm, 4px); --cb-stroke: 2px; }
  :host([size='sm']) { --cb-size: 14px; --cb-radius: 3px;                   --cb-stroke: 1.75px; }
  :host([size='lg']) { --cb-size: 18px; --cb-radius: var(--radius-sm, 4px); --cb-stroke: 2.25px; }

  label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
  }
  :host([disabled]) label { cursor: not-allowed; }

  .box {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: var(--cb-size);
    height: var(--cb-size);
    background: var(--background);
    border: 1px solid color-mix(in oklch, var(--border) 160%, transparent);
    border-radius: var(--cb-radius);
    box-sizing: border-box;
    box-shadow: var(--shadow-xs);
    transition:
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }

  :host(:not([disabled]):not([checked]):not([indeterminate])) label:hover .box {
    border-color: color-mix(in oklch, var(--border) 220%, transparent);
    box-shadow: var(--shadow-sm);
  }

  :host([invalid]) .box {
    border-color: color-mix(in oklch, var(--destructive) 70%, var(--border));
  }
  :host([invalid]:not([disabled])) label:hover .box {
    border-color: color-mix(in oklch, var(--destructive) 85%, var(--border));
  }

  :host([checked]) .box,
  :host([indeterminate]) .box {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 8%);
    box-shadow:
      inset 0 1px 0 0 color-mix(in oklch, white 20%, transparent),
      var(--shadow-xs);
  }
  :host([checked]:not([disabled])) label:hover .box,
  :host([indeterminate]:not([disabled])) label:hover .box {
    background: color-mix(in oklch, var(--tone) 92%, white 8%);
    box-shadow:
      inset 0 1px 0 0 color-mix(in oklch, white 24%, transparent),
      var(--shadow-sm);
  }

  /* Glyphs — CSS-drawn check and indeterminate indicator. */
  .glyph {
    position: absolute;
    inset: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--tone-on-fg);
    opacity: 0;
    transform: scale(0.6);
    transition:
      opacity var(--duration-fast, 150ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 150ms) var(--ease-out, ease-out);
    pointer-events: none;
  }
  :host([checked]:not([indeterminate])) .glyph.check,
  :host([indeterminate]) .glyph.mixed {
    opacity: 1;
    transform: scale(1);
  }

  /* Check mark: rotated L-shape via borders */
  .glyph.check::before {
    content: '';
    display: block;
    width: 45%;
    height: 28%;
    border-left: var(--cb-stroke) solid currentColor;
    border-bottom: var(--cb-stroke) solid currentColor;
    transform: rotate(-45deg) translate(0, -15%);
  }

  /* Indeterminate bar */
  .glyph.mixed::before {
    content: '';
    display: block;
    width: 60%;
    height: var(--cb-stroke);
    background: currentColor;
    border-radius: 1px;
  }

  /* Native input — visually hidden but preserved for a11y + form submission */
  input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    padding: 0;
    opacity: 0;
    cursor: inherit;
    appearance: none;
    -webkit-appearance: none;
  }

  :host(:not([disabled])) label:has(input:focus-visible) .box {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }
  :host([invalid]:not([disabled])) label:has(input:focus-visible) .box {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--destructive) var(--ring-opacity, 50%), transparent);
  }

  @media (forced-colors: active) {
    .box { border-color: CanvasText; }
    :host([checked]) .box,
    :host([indeterminate]) .box { background: Highlight; }
    .glyph { color: HighlightText; }
    :host(:not([disabled])) label:has(input:focus-visible) .box {
      outline: 2px solid CanvasText;
      outline-offset: 2px;
    }
  }


  :host([disabled]) { color: var(--muted-foreground); }
  :host([disabled]) .box {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 70%, transparent);
    filter: saturate(0.6);
    box-shadow: none;
  }
  :host([disabled][checked]) .box,
  :host([disabled][indeterminate]) .box {
    background: color-mix(in oklch, var(--tone) 55%, var(--muted));
    border-color: color-mix(in oklch, var(--tone) 40%, var(--border));
  }

  ::slotted(*) {
    font: inherit;
    color: inherit;
    line-height: var(--leading-tight, 1.25);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.checked = props.checked;
    $scope.indeterminate = props.indeterminate;
    $scope.disabled = props.disabled;
    $scope.required = props.required;
    $scope.invalid = props.invalid;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.name = props.name;
    $scope.value = props.value;
    $scope.label = props.label;
    $scope["inputEl"] = signal9(null);
    let __r = null;
    if (sui_checkbox_controller_default && typeof sui_checkbox_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_checkbox_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets11(__root);
    const __e0 = document.createElement("label");
    __e0.setAttribute("part", "label");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "box");
    __e1.setAttribute("part", "box");
    const __e2 = document.createElement("input");
    __e2.setAttribute("part", "input");
    __e2.setAttribute("type", "checkbox");
    __cleanups.push(bindChecked(__e2, $scope.checked));
    __cleanups.push(bindAttr8(__e2, "disabled", () => $scope.disabled()));
    __cleanups.push(bindAttr8(__e2, "required", () => $scope.required()));
    __cleanups.push(bindAttr8(__e2, "name", () => $scope.name()));
    __cleanups.push(bindAttr8(__e2, "value", () => $scope.value()));
    __cleanups.push(bindAttr8(__e2, "aria-checked", () => $scope.indeterminate() ? "mixed" : $scope.checked() ? "true" : "false"));
    __cleanups.push(bindAttr8(__e2, "aria-invalid", () => $scope.invalid() ? "true" : "false"));
    __cleanups.push(bindAttr8(__e2, "aria-label", () => $scope.label() || $scope.name() || "checkbox"));
    attachRef5($scope, "inputEl", __e2);
    __e1.appendChild(__e2);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "glyph check");
    __e3.setAttribute("aria-hidden", "true");
    __e1.appendChild(__e3);
    const __e4 = document.createElement("span");
    __e4.setAttribute("class", "glyph mixed");
    __e4.setAttribute("aria-hidden", "true");
    __e1.appendChild(__e4);
    __e0.appendChild(__e1);
    const __e5 = document.createElement("slot");
    __e0.appendChild(__e5);
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

// packages/components/primitives/src/sui-chip/sui-chip.sui
import { adoptSuiSheets as adoptSuiSheets12, attachRef as attachRef6, bindIf as bindIf5, component as component12, prop as prop12, signal as signal10 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-chip/sui-chip.controller.ts
import { controller as controller10, effect as effect15 } from "@ship/sui-runtime";
var sui_chip_controller_default = controller10({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "button");
    if (!host.hasAttribute("tabindex") && !scope.disabled()) {
      host.setAttribute("tabindex", "0");
    }
    const stopSync = effect15(() => {
      const sel = scope.selected();
      const dis = scope.disabled();
      host.setAttribute("aria-pressed", sel ? "true" : "false");
      if (dis) {
        host.setAttribute("aria-disabled", "true");
        if (host.getAttribute("tabindex") !== "-1")
          host.setAttribute("tabindex", "-1");
      } else {
        host.removeAttribute("aria-disabled");
        if (host.getAttribute("tabindex") === "-1")
          host.setAttribute("tabindex", "0");
      }
    });
    const onKeydown = (e) => {
      if (scope.disabled())
        return;
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        host.click();
      } else if (scope.removable() && (e.key === "Backspace" || e.key === "Delete")) {
        e.preventDefault();
        host.dispatchEvent(new CustomEvent("ship-remove", { detail: {}, bubbles: true, composed: true }));
      }
    };
    host.addEventListener("keydown", onKeydown);
    const stopRemoveBtn = effect15(() => {
      const btn = scope.removeBtn?.();
      if (!btn)
        return;
      const onClick = (e) => {
        e.stopPropagation();
        if (scope.disabled())
          return;
        host.dispatchEvent(new CustomEvent("ship-remove", { detail: {}, bubbles: true, composed: true }));
      };
      btn.addEventListener("click", onClick);
      return () => btn.removeEventListener("click", onClick);
    });
    return () => {
      stopSync();
      host.removeEventListener("keydown", onKeydown);
      stopRemoveBtn();
    };
  }
});

// packages/components/primitives/src/sui-chip/sui-chip.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_chip_default = component12("sui-chip", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    tone: prop12.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "muted", reflect: true }),
    variant: prop12.union(["solid", "soft", "outline", "ghost"], { default: "soft", reflect: true }),
    size: prop12.union(["sm", "md", "lg"], { default: "sm", reflect: true }),
    selected: prop12.bool({ default: false, reflect: true }),
    removable: prop12.bool({ default: false, reflect: true }),
    disabled: prop12.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    -webkit-tap-highlight-color: transparent;
    --tone: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
    --tone-contrast: var(--background);
    --chip-radius: var(--radius-full);
  }

  :host(:not([size])),
  :host([size='sm']) { --chip-h: 22px; --chip-px: 8px;  --chip-fs: var(--text-xs);   --chip-gap: 4px; --chip-icon: 10px; --chip-remove: 14px; }
  :host([size='md']) { --chip-h: 26px; --chip-px: 10px; --chip-fs: var(--text-sm);   --chip-gap: 6px; --chip-icon: 12px; --chip-remove: 16px; }
  :host([size='lg']) { --chip-h: 30px; --chip-px: 12px; --chip-fs: var(--text-base); --chip-gap: 6px; --chip-icon: 14px; --chip-remove: 18px; }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--chip-gap);
    min-height: var(--chip-h);
    padding: 0 var(--chip-px);
    font-family: var(--font-body);
    font-size: var(--chip-fs);
    font-weight: var(--font-weight-medium);
    line-height: 1;
    color: var(--tone-fg);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--chip-radius);
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }

  :host(:not([variant])) .chip,
  :host([variant='soft']) .chip {
    background: color-mix(in oklch, var(--tone) 14%, transparent);
    border-color: color-mix(in oklch, var(--tone) 22%, transparent);
  }
  :host([variant='soft']:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 22%, transparent);
    border-color: color-mix(in oklch, var(--tone) 32%, transparent);
  }
  :host([variant='outline']) .chip {
    background: transparent;
    border-color: color-mix(in oklch, var(--tone) 45%, transparent);
  }
  :host([variant='outline']:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 10%, transparent);
  }
  :host([variant='solid']) .chip {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
  }
  :host([variant='ghost']) .chip {
    background: transparent;
    border-color: transparent;
  }
  :host([variant='ghost']:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 10%, transparent);
  }

  :host([selected]) .chip {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 10%);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 20%, transparent);
    font-weight: var(--font-weight-semibold);
  }
  :host([selected]:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 92%, white 8%);
  }
  :host([selected]:not([tone])) .chip,
  :host([selected][tone='muted']) .chip {
    background: var(--foreground);
    border-color: var(--foreground);
    color: var(--background);
  }

  /* Keyboard-only focus ring — host carries focus (role=button,
   * tabindex=0 from controller). Outline gives a robust ring across
   * themes; the stacked box-shadow keeps the soft halo for parity
   * with sui-button. */
  :host(:focus-visible) {
    outline: 2px solid color-mix(in oklch, var(--tone) 75%, var(--ring-color, var(--primary)));
    outline-offset: 2px;
    border-radius: var(--chip-radius);
  }
  :host(:focus-visible) .chip {
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
  }
  :host([disabled]:focus-visible) { outline: none; }

  :host([disabled]) .chip {
    cursor: not-allowed;
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 80%, transparent);
    color: color-mix(in oklch, var(--muted-foreground) 70%, transparent);
    filter: saturate(0.65);
    box-shadow: none;
  }

  .leading {
    display: none;
    align-items: center;
    justify-content: center;
    width: var(--chip-icon);
    height: var(--chip-icon);
    flex-shrink: 0;
    color: currentColor;
  }
  .leading:has(::slotted(*)) { display: inline-flex; }
  ::slotted([slot='leading']) {
    display: inline-flex;
    align-items: center;
    color: currentColor;
    width: 100%;
    height: 100%;
  }

  .remove {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--chip-remove);
    height: var(--chip-remove);
    margin-inline: 2px -4px;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    cursor: pointer;
    border-radius: var(--radius-full);
    flex-shrink: 0;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .remove svg {
    width: 70%;
    height: 70%;
    display: block;
    stroke: currentColor;
    fill: none;
    stroke-width: 1.75;
    stroke-linecap: round;
  }
  .remove:hover:not(:disabled) {
    background: color-mix(in oklch, currentColor 18%, transparent);
  }
  :host([disabled]) .remove { cursor: not-allowed; }

  @media (forced-colors: active) {
    .chip { border-color: CanvasText; color: CanvasText; }
    :host([selected]) .chip { background: Highlight; color: HighlightText; border-color: Highlight; }
    :host(:focus-visible) .chip { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.selected = props.selected;
    $scope.removable = props.removable;
    $scope.disabled = props.disabled;
    $scope["removeBtn"] = signal10(null);
    let __r = null;
    if (sui_chip_controller_default && typeof sui_chip_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_chip_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindIf5({
      anchor,
      test: () => {
        if ($scope.removable())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("button");
          __e7.setAttribute("class", "remove");
          __e7.setAttribute("part", "remove");
          __e7.setAttribute("type", "button");
          __e7.setAttribute("aria-label", "Remove");
          __e7.setAttribute("tabindex", "-1");
          attachRef6($scope, "removeBtn", __e7);
          const __e8 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __e8.setAttribute("viewBox", "0 0 10 10");
          __e8.setAttribute("aria-hidden", "true");
          const __e9 = document.createElementNS("http://www.w3.org/2000/svg", "path");
          __e9.setAttribute("d", "M2 2 L8 8 M8 2 L2 8");
          __e8.appendChild(__e9);
          __e7.appendChild(__e8);
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets12(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "chip");
    __e0.setAttribute("part", "chip");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "leading");
    __e1.setAttribute("part", "leading");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "leading");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("slot");
    __e0.appendChild(__e3);
    const __ifAnchor4 = document.createComment("if");
    __e0.appendChild(__ifAnchor4);
    __cleanups.push(__wireBoundary_b0(__ifAnchor4));
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

// packages/components/primitives/src/sui-cluster/sui-cluster.sui
import { adoptSuiSheets as adoptSuiSheets13, component as component13, prop as prop13 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_cluster_default = component13("sui-cluster", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    gap: prop13.union(["0", "1", "2", "3", "4", "5", "6", "7", "8"], { default: "3", reflect: true }),
    align: prop13.union(["start", "center", "end", "baseline"], { default: "center", reflect: true }),
    justify: prop13.union(["start", "center", "end", "between", "around"], { default: "start", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --cluster-gap: var(--space-3);
  }

  :host([gap='0']) { --cluster-gap: 0; }
  :host([gap='1']) { --cluster-gap: var(--space-1); }
  :host([gap='2']) { --cluster-gap: var(--space-2); }
  :host([gap='3']) { --cluster-gap: var(--space-3); }
  :host([gap='4']) { --cluster-gap: var(--space-4); }
  :host([gap='5']) { --cluster-gap: var(--space-5); }
  :host([gap='6']) { --cluster-gap: var(--space-6); }
  :host([gap='7']) { --cluster-gap: var(--space-7); }
  :host([gap='8']) { --cluster-gap: var(--space-8); }

  .cluster {
    display: flex;
    flex-wrap: wrap;
    gap: var(--cluster-gap);
    align-items: center;
    justify-content: flex-start;
  }
  :host([align='start'])    .cluster { align-items: flex-start; }
  :host([align='center'])   .cluster { align-items: center; }
  :host([align='end'])      .cluster { align-items: flex-end; }
  :host([align='baseline']) .cluster { align-items: baseline; }

  :host([justify='start'])   .cluster { justify-content: flex-start; }
  :host([justify='center'])  .cluster { justify-content: center; }
  :host([justify='end'])     .cluster { justify-content: flex-end; }
  :host([justify='between']) .cluster { justify-content: space-between; }
  :host([justify='around'])  .cluster { justify-content: space-around; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.gap = props.gap;
    $scope.align = props.align;
    $scope.justify = props.justify;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets13(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "cluster");
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-code/sui-code.sui
import { adoptSuiSheets as adoptSuiSheets14, bindAttr as bindAttr9, bindIf as bindIf6, bindText as bindText4, component as component14, prop as prop14 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_code_default = component14("sui-code", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    lang: prop14.string({ default: "", reflect: true }),
    block: prop14.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }
  :host([block]) { display: block; }

  .inline {
    padding: 1px 5px;
    border-radius: var(--radius-sm);
    background: var(--muted);
    color: var(--foreground);
    border: 1px solid var(--border);
    font-size: 0.92em;
  }
  .block {
    display: block;
    background: var(--muted);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .block header {
    display: flex;
    justify-content: space-between;
    padding: 4px 10px;
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    background: color-mix(in oklch, var(--muted) 60%, var(--background));
    border-bottom: 1px solid var(--border);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-weight: var(--font-weight-medium);
  }
  pre {
    margin: 0;
    padding: 10px 12px;
    overflow-x: auto;
    line-height: var(--leading-normal);
    font-family: inherit;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.lang = props.lang;
    $scope.block = props.block;
    const __wireBoundary_b2 = (anchor) => bindIf6({
      anchor,
      test: () => {
        if ($scope.lang())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("header");
          __e7.setAttribute("part", "label");
          const __t8 = document.createTextNode("");
          __e7.appendChild(__t8);
          __c6.push(bindText4(__t8, () => $scope.lang()));
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b0 = (anchor) => bindIf6({
      anchor,
      test: () => {
        if ($scope.block())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("div");
          __e3.setAttribute("class", "block");
          __e3.setAttribute("part", "block");
          __e3.setAttribute("role", "region");
          __c2.push(bindAttr9(__e3, "aria-label", () => $scope.lang() ? $scope.lang() + " code block" : "code block"));
          const __ifAnchor4 = document.createComment("if");
          __e3.appendChild(__ifAnchor4);
          __c2.push(__wireBoundary_b2(__ifAnchor4));
          const __e9 = document.createElement("pre");
          __e9.setAttribute("part", "pre");
          const __e10 = document.createElement("slot");
          __e9.appendChild(__e10);
          __e3.appendChild(__e9);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag11 = document.createDocumentFragment();
          const __c12 = [];
          const __e13 = document.createElement("code");
          __e13.setAttribute("class", "inline");
          __e13.setAttribute("part", "inline");
          const __e14 = document.createElement("slot");
          __e13.appendChild(__e14);
          __frag11.appendChild(__e13);
          return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
            for (const c of __c12)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets14(__root);
    const __ifAnchor0 = document.createComment("if");
    __root.appendChild(__ifAnchor0);
    __cleanups.push(__wireBoundary_b0(__ifAnchor0));
    return $scope;
  }
});

// packages/components/primitives/src/sui-collapsible/sui-collapsible.sui
import { adoptSuiSheets as adoptSuiSheets15, attachRef as attachRef7, component as component15, prop as prop15, signal as signal11 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-collapsible/sui-collapsible.controller.ts
import { controller as controller11, effect as effect16 } from "@ship/sui-runtime";
var _seq2 = 0;
var sui_collapsible_controller_default = controller11({
  mount(state, host) {
    const scope = state;
    const contentId = `sui-collapsible-${++_seq2}`;
    const stopContentId = effect16(() => {
      const wrap = scope.contentWrap();
      if (wrap)
        wrap.id = contentId;
    });
    const syncAriaExpanded = () => {
      const slot = scope.triggerSlot();
      if (!slot)
        return;
      const isOpen = scope.open();
      for (const el of slot.assignedElements()) {
        const h = el;
        h.setAttribute("aria-expanded", String(isOpen));
        h.setAttribute("aria-controls", contentId);
        if (!h.hasAttribute("tabindex") && !h.matches("button, a, input, select, textarea")) {
          h.setAttribute("tabindex", "0");
        }
      }
    };
    const stopSlot = effect16(() => {
      const slot = scope.triggerSlot();
      if (!slot)
        return;
      slot.addEventListener("slotchange", syncAriaExpanded);
      syncAriaExpanded();
      return () => slot.removeEventListener("slotchange", syncAriaExpanded);
    });
    const stopOpenSync = effect16(() => {
      scope.open();
      scope.triggerSlot();
      syncAriaExpanded();
    });
    const toggle = () => {
      if (scope.disabled())
        return;
      const next = !scope.open();
      scope.open.set(next);
      host.dispatchEvent(new CustomEvent("sui-toggle", {
        detail: { open: next },
        bubbles: true,
        composed: true
      }));
    };
    const onKeydown = (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        toggle();
      }
    };
    const stopWrap = effect16(() => {
      const wrap = scope.triggerWrap();
      if (!wrap)
        return;
      wrap.addEventListener("click", toggle);
      wrap.addEventListener("keydown", onKeydown);
      return () => {
        wrap.removeEventListener("click", toggle);
        wrap.removeEventListener("keydown", onKeydown);
      };
    });
    return () => {
      stopContentId();
      stopSlot();
      stopOpenSync();
      stopWrap();
    };
  }
});

// packages/components/primitives/src/sui-collapsible/sui-collapsible.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_collapsible_default = component15("sui-collapsible", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    open: prop15.bool({ default: false, reflect: true }),
    disabled: prop15.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    color: var(--foreground);
  }

  .content-wrap {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  :host([open]) .content-wrap {
    grid-template-rows: 1fr;
  }

  .content-inner {
    overflow: hidden;
  }

  ::slotted([slot='trigger']) {
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
  }
  :host([disabled]) ::slotted([slot='trigger']) {
    cursor: not-allowed;
    opacity: 0.5;
  }

  @media (prefers-reduced-motion: reduce) {
    .content-wrap { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.disabled = props.disabled;
    $scope["contentWrap"] = signal11(null);
    $scope["triggerSlot"] = signal11(null);
    $scope["triggerWrap"] = signal11(null);
    let __r = null;
    if (sui_collapsible_controller_default && typeof sui_collapsible_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_collapsible_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets15(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "trigger-wrap");
    attachRef7($scope, "triggerWrap", __e0);
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "trigger");
    attachRef7($scope, "triggerSlot", __e1);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "content-wrap");
    __e2.setAttribute("part", "content");
    attachRef7($scope, "contentWrap", __e2);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "content-inner");
    const __e4 = document.createElement("slot");
    __e3.appendChild(__e4);
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
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

// packages/components/primitives/src/sui-context-menu/sui-context-menu.sui
import { adoptSuiSheets as adoptSuiSheets16, attachRef as attachRef8, component as component16, prop as prop16, signal as signal12 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-context-menu/sui-context-menu.controller.ts
import { controller as controller12, effect as effect17, bindNativeEvent } from "@ship/sui-runtime";
var _seq3 = 0;
var sui_context_menu_controller_default = controller12({
  mount(state, host) {
    const scope = state;
    const panelId = `sui-context-menu-panel-${++_seq3}`;
    const triggerSlot = scope.triggerSlot();
    const panelEl = scope.panelEl();
    if (!panelEl)
      return;
    panelEl.id = panelId;
    let typeaheadBuf = "";
    let typeaheadTimer = null;
    function pushTypeahead(key) {
      if (typeaheadTimer)
        clearTimeout(typeaheadTimer);
      typeaheadBuf += key.toLowerCase();
      typeaheadTimer = setTimeout(() => {
        typeaheadBuf = "";
      }, 350);
      const items = getItems();
      const i = items.indexOf(document.activeElement);
      for (let j = 0;j < items.length; j++) {
        const idx = (i + 1 + j) % items.length;
        const label = (items[idx].textContent ?? "").trim().toLowerCase();
        if (label.startsWith(typeaheadBuf))
          return idx;
      }
      return -1;
    }
    function getItems() {
      return Array.from(host.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((el) => !el.hasAttribute("disabled"));
    }
    function focusItem(index) {
      const items = getItems();
      if (!items.length)
        return;
      const i = (index % items.length + items.length) % items.length;
      for (const it of items)
        it.setAttribute("tabindex", "-1");
      items[i].setAttribute("tabindex", "0");
      items[i].focus();
    }
    function currentIndex() {
      return getItems().indexOf(document.activeElement);
    }
    function setupItems() {
      getItems().forEach((el, idx) => {
        if (!el.hasAttribute("role"))
          el.setAttribute("role", "menuitem");
        if (!el.hasAttribute("tabindex"))
          el.setAttribute("tabindex", idx === 0 ? "0" : "-1");
      });
    }
    function doShow(x, y) {
      if (panelEl) {
        panelEl.style.top = `${y}px`;
        panelEl.style.left = `${x}px`;
      }
      panelEl?.showPopover?.();
    }
    function doClose() {
      panelEl?.hidePopover?.();
    }
    host.expose({ show: doShow, close: doClose });
    const onContextMenu = (e) => {
      e.preventDefault();
      doShow(e.clientX, e.clientY);
    };
    const onToggle = (ev) => {
      const te = ev;
      const nowOpen = te.newState === "open";
      if (nowOpen !== scope.open())
        scope.open.set(nowOpen);
      host.setAttribute("data-state", nowOpen ? "open" : "closed");
      if (nowOpen) {
        host.emit("sui-open");
        queueMicrotask(() => {
          setupItems();
          focusItem(0);
        });
      } else {
        host.emit("sui-close");
      }
    };
    const onItemClick = (e) => {
      const target = e.target.closest("[data-value]");
      if (!target)
        return;
      host.emit("sui-select", { value: target.dataset.value ?? "" });
      doClose();
    };
    const onKeydown = (e) => {
      if (!scope.open())
        return;
      const items = getItems();
      if (!items.length)
        return;
      const i = currentIndex();
      if (e.key === "ArrowDown") {
        e.preventDefault();
        focusItem(i + 1);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        focusItem(i - 1);
      } else if (e.key === "Home") {
        e.preventDefault();
        focusItem(0);
      } else if (e.key === "End") {
        e.preventDefault();
        focusItem(items.length - 1);
      } else if (e.key === "Escape") {
        e.preventDefault();
        doClose();
      } else if (e.key === "Tab") {
        doClose();
      } else if ((e.key === "Enter" || e.key === " ") && i >= 0) {
        e.preventDefault();
        items[i].click();
      } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
        const next = pushTypeahead(e.key);
        if (next >= 0) {
          e.preventDefault();
          focusItem(next);
        }
      }
    };
    const wireTrigger = () => {
      const els = triggerSlot?.assignedElements({ flatten: true }) ?? [];
      for (const el of els) {
        el.addEventListener("contextmenu", onContextMenu);
      }
    };
    if (triggerSlot)
      bindNativeEvent(triggerSlot, "slotchange", wireTrigger);
    wireTrigger();
    bindNativeEvent(panelEl, "toggle", onToggle);
    bindNativeEvent(panelEl, "click", onItemClick);
    bindNativeEvent(host, "keydown", onKeydown);
    const stopSync = effect17(() => {
      host.setAttribute("data-state", scope.open() ? "open" : "closed");
    });
    return () => {
      const els = triggerSlot?.assignedElements({ flatten: true }) ?? [];
      for (const el of els)
        el.removeEventListener("contextmenu", onContextMenu);
      if (typeaheadTimer)
        clearTimeout(typeaheadTimer);
      stopSync();
    };
  }
});

// packages/components/primitives/src/sui-context-menu/sui-context-menu.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_context_menu_default = component16("sui-context-menu", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    open: prop16.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: contents; }

  .panel {
    inset: auto;
    margin: 0;
    min-width: 160px;
    padding: 4px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius, var(--radius-md, 6px));
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    display: flex; flex-direction: column;
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
    opacity: 0;
    transform: scale(0.96);
    transition:
      opacity var(--duration-fast, 120ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: scale(1);
  }
  .panel::backdrop { background: transparent; }

  ::slotted(button), ::slotted([role='menuitem']) {
    text-align: left;
    background: transparent; border: 0;
    padding: 6px 10px;
    border-radius: var(--radius-sm, 4px);
    color: inherit; font: inherit;
    cursor: pointer; width: 100%;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  ::slotted(button:hover), ::slotted([role='menuitem']:hover),
  ::slotted(button:focus-visible), ::slotted([role='menuitem']:focus-visible) {
    background: var(--accent, var(--muted, #f5f5f5));
    outline: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .panel { transition: none; transform: none !important; }
  }
  @media (forced-colors: active) {
    .panel { border: 1px solid CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope["itemSlot"] = signal12(null);
    $scope["panelEl"] = signal12(null);
    $scope["triggerSlot"] = signal12(null);
    let __r = null;
    if (sui_context_menu_controller_default && typeof sui_context_menu_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_context_menu_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets16(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "trigger");
    attachRef8($scope, "triggerSlot", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "panel");
    __e1.setAttribute("part", "panel");
    __e1.setAttribute("popover", "auto");
    __e1.setAttribute("role", "menu");
    __e1.setAttribute("aria-label", "Context menu");
    attachRef8($scope, "panelEl", __e1);
    const __e2 = document.createElement("slot");
    attachRef8($scope, "itemSlot", __e2);
    __e1.appendChild(__e2);
    __root.appendChild(__e1);
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

// packages/components/primitives/src/sui-copy-button/sui-copy-button.sui
import { adoptSuiSheets as adoptSuiSheets17, attachRef as attachRef9, bindAttr as bindAttr10, bindIf as bindIf7, bindSlotEmpty as bindSlotEmpty3, bindText as bindText5, component as component17, prop as prop17, signal as signal13 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-copy-button/sui-copy-button.controller.ts
import { controller as controller13, effect as effect18 } from "@ship/sui-runtime";
var sui_copy_button_controller_default = controller13({
  mount(state, host) {
    const scope = state;
    let resetTimer;
    const handleClick = async () => {
      try {
        await navigator.clipboard.writeText(scope.value());
      } catch {
        const ta = document.createElement("textarea");
        ta.value = scope.value();
        document.body.appendChild(ta);
        ta.select();
        try {
          document.execCommand("copy");
        } catch {}
        ta.remove();
      }
      scope.copied.set(true);
      host.dispatchEvent(new CustomEvent("sui-copy", {
        detail: { value: scope.value() },
        bubbles: true,
        composed: true
      }));
      if (resetTimer)
        clearTimeout(resetTimer);
      resetTimer = setTimeout(() => {
        scope.copied.set(false);
      }, 1200);
    };
    const stopBtn = effect18(() => {
      const btn = scope.buttonEl();
      if (!btn)
        return;
      btn.addEventListener("click", handleClick);
      return () => btn.removeEventListener("click", handleClick);
    });
    return () => {
      stopBtn();
      if (resetTimer)
        clearTimeout(resetTimer);
    };
  }
});

// packages/components/primitives/src/sui-copy-button/sui-copy-button.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_copy_button_default = component17("sui-copy-button", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    value: prop17.string({ default: "", reflect: true }),
    label: prop17.string({ default: "Copy", reflect: true }),
    copied: prop17.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
  }

  button {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-body);
    font-size: var(--text-xs);
    font-weight: var(--font-weight-medium);
    padding: 4px 8px;
    color: var(--muted-foreground);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition:
      color var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out);
  }
  button:hover {
    color: var(--foreground);
    background: var(--muted);
  }
  button:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--ring-color, var(--primary)) var(--ring-opacity, 50%), transparent);
  }
  @media (forced-colors: active) {
    button:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
  :host([copied]) button {
    color: var(--success);
    border-color: var(--success);
  }

  /* Show default label when slot is empty (JSDOM always reports empty, so tests see this) */
  .default-label { display: inline; }
  :host(:not([data-empty])) .default-label { display: none; }

  sui-icon {
    flex: 0 0 auto;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.label = props.label;
    $scope.copied = props.copied;
    $scope["buttonEl"] = signal13(null);
    let __r = null;
    if (sui_copy_button_controller_default && typeof sui_copy_button_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_copy_button_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindIf7({
      anchor,
      test: () => {
        if ($scope.copied())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag2 = document.createDocumentFragment();
          const __c3 = [];
          const __e4 = document.createElement("sui-icon");
          __e4.setAttribute("name", "check");
          __e4.setAttribute("size", "sm");
          __e4.setAttribute("aria-hidden", "true");
          __frag2.appendChild(__e4);
          return { nodes: Array.from(__frag2.childNodes), cleanup: () => {
            for (const c of __c3)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("sui-icon");
          __e7.setAttribute("name", "copy");
          __e7.setAttribute("size", "sm");
          __e7.setAttribute("aria-hidden", "true");
          __frag5.appendChild(__e7);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets17(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("part", "button");
    __e0.setAttribute("type", "button");
    __cleanups.push(bindAttr10(__e0, "aria-label", () => $scope.label()));
    attachRef9($scope, "buttonEl", __e0);
    const __ifAnchor1 = document.createComment("if");
    __e0.appendChild(__ifAnchor1);
    __cleanups.push(__wireBoundary_b2(__ifAnchor1));
    const __e8 = document.createElement("slot");
    __cleanups.push(bindSlotEmpty3(__e8, host, "data-empty"));
    __e0.appendChild(__e8);
    const __e9 = document.createElement("span");
    __e9.setAttribute("class", "default-label");
    const __t10 = document.createTextNode("");
    __e9.appendChild(__t10);
    __cleanups.push(bindText5(__t10, () => $scope.copied() ? "Copied" : $scope.label()));
    __e0.appendChild(__e9);
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

// packages/components/primitives/src/sui-dialog/sui-dialog.sui
import { adoptSuiSheets as adoptSuiSheets18, attachRef as attachRef10, bindIf as bindIf8, component as component18, fade, prop as prop18, registerTransition, signal as signal14 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-dialog/sui-dialog.controller.ts
import { controller as controller14, effect as effect19 } from "@ship/sui-runtime";

// packages/components/primitives/src/overlay-focus.ts
var FOCUSABLE = [
  "a[href]",
  "area[href]",
  "button:not([disabled])",
  'input:not([disabled]):not([type="hidden"])',
  "select:not([disabled])",
  "textarea:not([disabled])",
  "iframe",
  "object",
  "embed",
  "summary",
  '[contenteditable]:not([contenteditable="false"])',
  '[tabindex]:not([tabindex^="-"])'
].join(",");
function focusableWithin(root) {
  return Array.from(root.querySelectorAll(FOCUSABLE)).filter((el) => {
    if (el.hasAttribute("disabled"))
      return false;
    if (el.getAttribute("aria-hidden") === "true")
      return false;
    return true;
  });
}

class FocusReturn {
  saved = null;
  save() {
    this.saved = document.activeElement ?? null;
  }
  restore() {
    const t = this.saved;
    this.saved = null;
    if (t && typeof t.focus === "function")
      t.focus();
  }
}
var _scrollLockCount = 0;
var _savedOverflow = null;
var _savedPadding = null;
function acquireScrollLock() {
  if (typeof document === "undefined")
    return;
  _scrollLockCount += 1;
  if (_scrollLockCount > 1)
    return;
  const body = document.body;
  _savedOverflow = body.style.overflow;
  _savedPadding = body.style.paddingRight;
  const scrollW = typeof window !== "undefined" ? window.innerWidth - document.documentElement.clientWidth : 0;
  if (scrollW > 0) {
    const cur = parseFloat(getComputedStyle(body).paddingRight || "0") || 0;
    body.style.paddingRight = `${cur + scrollW}px`;
  }
  body.style.overflow = "hidden";
}
function releaseScrollLock() {
  if (typeof document === "undefined")
    return;
  if (_scrollLockCount === 0)
    return;
  _scrollLockCount -= 1;
  if (_scrollLockCount > 0)
    return;
  const body = document.body;
  body.style.overflow = _savedOverflow ?? "";
  body.style.paddingRight = _savedPadding ?? "";
  _savedOverflow = null;
  _savedPadding = null;
}

// packages/components/primitives/src/sui-dialog/sui-dialog.controller.ts
var _seq4 = 0;
var sui_dialog_controller_default = controller14({
  mount(state, host) {
    const scope = state;
    const uid = ++_seq4;
    const headerId = `sui-dialog-header-${uid}`;
    const bodyId = `sui-dialog-body-${uid}`;
    const dialogEl = scope.dialogEl();
    const closeBtn = scope.closeBtn();
    const sentinelStart = scope.sentinelStart();
    const sentinelEnd = scope.sentinelEnd();
    if (!dialogEl)
      return;
    const stopHeaderIdEffect = effect19(() => {
      const h = scope.headerEl();
      if (h)
        h.id = headerId;
    });
    const stopBodyIdEffect = effect19(() => {
      const b = scope.bodyEl();
      if (b)
        b.id = bodyId;
    });
    dialogEl.setAttribute("aria-labelledby", headerId);
    dialogEl.setAttribute("aria-describedby", bodyId);
    let hasScrollLock = false;
    const focusReturn = new FocusReturn;
    function updateRole() {
      dialogEl.setAttribute("role", scope.roleOverride() || "dialog");
      dialogEl.setAttribute("aria-modal", "true");
    }
    updateRole();
    function collectFocusables() {
      const shadowFocusables = focusableWithin(dialogEl).filter((el) => !el.classList.contains("sentinel"));
      const lightFocusables = focusableWithin(host);
      return [...lightFocusables, ...shadowFocusables];
    }
    function focusInitial() {
      const autofocus = host.querySelector("[autofocus]") ?? dialogEl.querySelector("[autofocus]");
      if (autofocus) {
        autofocus.focus();
        return;
      }
      const focusables = collectFocusables();
      const primary = focusables.find((el) => el !== closeBtn) ?? focusables[0];
      if (primary)
        primary.focus();
      else
        dialogEl.focus();
    }
    function doOpen() {
      focusReturn.save();
      try {
        dialogEl.showModal();
      } catch {}
      if (!hasScrollLock) {
        acquireScrollLock();
        hasScrollLock = true;
      }
      queueMicrotask(() => focusInitial());
      host.dispatchEvent(new CustomEvent("sui-open", { bubbles: true, composed: true }));
    }
    function doClose() {
      if (dialogEl.open)
        dialogEl.close();
    }
    const onNativeClose = () => {
      if (scope.open())
        scope.open.set(false);
      if (hasScrollLock) {
        releaseScrollLock();
        hasScrollLock = false;
      }
      focusReturn.restore();
      host.dispatchEvent(new CustomEvent("sui-close", { bubbles: true, composed: true }));
    };
    const onBackdropClick = (e) => {
      if (!scope.dismissible())
        return;
      if (e.target === dialogEl)
        scope.open.set(false);
    };
    const onDismiss = () => {
      host.dispatchEvent(new CustomEvent("sui-dismiss", { bubbles: true, composed: true }));
      scope.open.set(false);
    };
    const onSentinelStartFocus = () => {
      const items = collectFocusables();
      if (items.length)
        items[items.length - 1].focus();
    };
    const onSentinelEndFocus = () => {
      const items = collectFocusables();
      if (items.length)
        items[0].focus();
    };
    dialogEl.addEventListener("close", onNativeClose);
    dialogEl.addEventListener("click", onBackdropClick);
    closeBtn?.addEventListener("click", onDismiss);
    sentinelStart?.addEventListener("focus", onSentinelStartFocus);
    sentinelEnd?.addEventListener("focus", onSentinelEndFocus);
    const stopOpenSync = effect19(() => {
      const wantOpen = scope.open();
      host.setAttribute("data-state", wantOpen ? "open" : "closed");
      if (wantOpen && !dialogEl.open)
        doOpen();
      else if (!wantOpen && dialogEl.open)
        doClose();
    });
    const stopRoleSync = effect19(() => {
      updateRole();
    });
    host.show = () => scope.open.set(true);
    host.showModal = () => scope.open.set(true);
    host.close = () => scope.open.set(false);
    return () => {
      if (hasScrollLock)
        releaseScrollLock();
      dialogEl.removeEventListener("close", onNativeClose);
      dialogEl.removeEventListener("click", onBackdropClick);
      closeBtn?.removeEventListener("click", onDismiss);
      sentinelStart?.removeEventListener("focus", onSentinelStartFocus);
      sentinelEnd?.removeEventListener("focus", onSentinelEndFocus);
      stopOpenSync();
      stopRoleSync();
      stopHeaderIdEffect();
      stopBodyIdEffect();
    };
  }
});

// packages/components/primitives/src/sui-dialog/sui-dialog.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_dialog_default = component18("sui-dialog", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    open: prop18.bool({ default: false, reflect: true }),
    size: prop18.union(["sm", "md", "lg", "full"], { default: "md", reflect: true }),
    backdrop: prop18.union(["default", "blur", "solid"], { default: "default", reflect: true }),
    dismissible: prop18.bool({ default: true }),
    "no-close-button": prop18.bool({ default: false, reflect: true }),
    "no-header": prop18.bool({ default: false, reflect: true }),
    "no-footer": prop18.bool({ default: false, reflect: true }),
    "role-override": prop18.union(["dialog", "alertdialog"], { default: "dialog", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: contents; }

  :host(:not([size])), :host([size='md']) { --dlg-max-w: 500px; --dlg-px: 24px; --dlg-hpy: 18px; --dlg-bpy: 16px; --dlg-fpy: 16px; }
  :host([size='sm']) { --dlg-max-w: 400px; --dlg-px: 20px; --dlg-hpy: 16px; --dlg-bpy: 14px; --dlg-fpy: 14px; }
  :host([size='lg']) { --dlg-max-w: 700px; --dlg-px: 28px; --dlg-hpy: 20px; --dlg-bpy: 20px; --dlg-fpy: 18px; }
  :host([size='full']) { --dlg-max-w: 90vw; --dlg-px: 28px; --dlg-hpy: 20px; --dlg-bpy: 20px; --dlg-fpy: 18px; }

  /* \`dialog:not([open])\` MUST stay display:none — overriding to
     display:flex on the bare selector caused a closed dialog to paint
     a 2px artifact at viewport center, visible as a faint horizontal
     line across every view that mounted a sui-dialog. Qualifying with
     \`[open]\` keeps the closed dialog hidden per browser default. */
  dialog:not([open]) { display: none; }
  dialog[open] {
    position: fixed;
    inset: 0;
    margin: auto;
    padding: 0;
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-lg, 8px);
    background: var(--card, #fff);
    color: var(--card-foreground, #111);
    width: calc(100vw - 32px);
    max-width: var(--dlg-max-w, 500px);
    max-height: min(calc(100vh - 32px), 85vh);
    box-shadow: var(--shadow-xl, 0 20px 60px rgba(0,0,0,0.3));
    font-family: var(--font-body, system-ui);
    overflow: visible;
    display: flex;
    flex-direction: column;
  }
  :host([size='full']) dialog[open] { max-height: 90vh; }

  /* Open/close polish is driven by the \`transition:fade\` directive on
     the content wrapper (Web Animations API) and by the native
     ::backdrop fade below — no CSS transition on \`dialog\` itself. */
  .dialog-content {
    display: contents;
  }

  dialog::backdrop {
    background: hsl(0 0% 0% / 0.5);
    opacity: 0;
    transition: opacity var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  dialog[open]::backdrop { opacity: 1; }
  :host([backdrop='blur']) dialog::backdrop { backdrop-filter: blur(6px); -webkit-backdrop-filter: blur(6px); }
  :host([backdrop='solid']) dialog::backdrop { background: var(--background, #fff); }

  .sentinel { position: fixed; width: 1px; height: 1px; pointer-events: none; opacity: 0; }

  .header {
    padding: var(--dlg-hpy, 18px) var(--dlg-px, 24px);
    padding-right: calc(var(--dlg-px, 24px) + 36px);
    flex: 0 0 auto;
    font-size: var(--text-lg, 16px);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--foreground, #111);
  }
  :host([no-header]) .header { display: none; }

  .body {
    padding: var(--dlg-bpy, 16px) var(--dlg-px, 24px);
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    font-size: var(--text-base, 14px);
    line-height: var(--leading-normal, 1.5);
    scrollbar-width: thin;
  }

  .footer {
    padding: var(--dlg-fpy, 16px) var(--dlg-px, 24px);
    border-top: 1px solid var(--border, #e5e5e5);
    display: flex;
    gap: var(--space-2, 8px);
    justify-content: flex-end;
    align-items: center;
    flex: 0 0 auto;
  }
  :host([no-footer]) .footer { display: none; }

  .close {
    position: absolute;
    top: 12px;
    inset-inline-end: 12px;
    width: 28px; height: 28px;
    display: inline-flex; align-items: center; justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md, 6px);
    color: var(--muted-foreground, #888);
    cursor: pointer; padding: 0; font: inherit; z-index: 1;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out),
      color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  .close:hover { background: var(--muted, #f5f5f5); color: var(--foreground, #111); }
  .close svg { width: 14px; height: 14px; display: block; }
  :host([no-close-button]) .close { display: none; }

  @media (prefers-reduced-motion: reduce) {
    dialog::backdrop { transition: none; }
  }
  @media (forced-colors: active) {
    dialog { border: 1px solid CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.size = props.size;
    $scope.backdrop = props.backdrop;
    $scope.dismissible = props.dismissible;
    $scope.noCloseButton = props.noCloseButton;
    $scope.noHeader = props.noHeader;
    $scope.noFooter = props.noFooter;
    $scope.roleOverride = props.roleOverride;
    $scope["bodyEl"] = signal14(null);
    $scope["closeBtn"] = signal14(null);
    $scope["dialogEl"] = signal14(null);
    $scope["headerEl"] = signal14(null);
    $scope["sentinelEnd"] = signal14(null);
    $scope["sentinelStart"] = signal14(null);
    let __r = null;
    if (sui_dialog_controller_default && typeof sui_dialog_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_dialog_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b3 = (anchor) => bindIf8({
      anchor,
      test: () => {
        if ($scope.open())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag6 = document.createDocumentFragment();
          const __c7 = [];
          const __e8 = document.createElement("div");
          __e8.setAttribute("class", "dialog-content");
          registerTransition(__e8, fade, { duration: 150 }, "both");
          const __e9 = document.createElement("div");
          __e9.setAttribute("class", "header");
          __e9.setAttribute("part", "header");
          attachRef10($scope, "headerEl", __e9);
          const __e10 = document.createElement("slot");
          __e10.setAttribute("name", "header");
          __e9.appendChild(__e10);
          __e8.appendChild(__e9);
          const __e11 = document.createElement("div");
          __e11.setAttribute("class", "body");
          __e11.setAttribute("part", "body");
          attachRef10($scope, "bodyEl", __e11);
          const __e12 = document.createElement("slot");
          __e11.appendChild(__e12);
          __e8.appendChild(__e11);
          const __e13 = document.createElement("div");
          __e13.setAttribute("class", "footer");
          __e13.setAttribute("part", "footer");
          const __e14 = document.createElement("slot");
          __e14.setAttribute("name", "footer");
          __e13.appendChild(__e14);
          __e8.appendChild(__e13);
          __frag6.appendChild(__e8);
          return { nodes: Array.from(__frag6.childNodes), cleanup: () => {
            for (const c of __c7)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b7 = (anchor) => bindIf8({
      anchor,
      test: () => {
        if ($scope.open())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag17 = document.createDocumentFragment();
          const __c18 = [];
          const __e19 = document.createElement("sui-portal");
          __e19.setAttribute("target", "body");
          const __e20 = document.createElement("div");
          __e20.setAttribute("class", "sui-dialog-presence");
          __e20.setAttribute("aria-hidden", "true");
          __e20.setAttribute("data-sui-dialog-open", "");
          __e19.appendChild(__e20);
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets18(__root);
    const __e0 = document.createElement("dialog");
    __e0.setAttribute("part", "dialog");
    attachRef10($scope, "dialogEl", __e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("tabindex", "0");
    __e1.setAttribute("class", "sentinel");
    __e1.setAttribute("aria-hidden", "true");
    attachRef10($scope, "sentinelStart", __e1);
    __e0.appendChild(__e1);
    const __e2 = document.createElement("button");
    __e2.setAttribute("class", "close");
    __e2.setAttribute("part", "close");
    __e2.setAttribute("type", "button");
    __e2.setAttribute("aria-label", "Close dialog");
    attachRef10($scope, "closeBtn", __e2);
    const __e3 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    __e3.setAttribute("viewBox", "0 0 16 16");
    __e3.setAttribute("fill", "none");
    __e3.setAttribute("stroke", "currentColor");
    __e3.setAttribute("stroke-width", "1.75");
    __e3.setAttribute("stroke-linecap", "round");
    __e3.setAttribute("stroke-linejoin", "round");
    __e3.setAttribute("aria-hidden", "true");
    __e3.setAttribute("focusable", "false");
    const __e4 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e4.setAttribute("d", "M3.5 3.5l9 9M12.5 3.5l-9 9");
    __e3.appendChild(__e4);
    __e2.appendChild(__e3);
    __e0.appendChild(__e2);
    const __ifAnchor5 = document.createComment("if");
    __e0.appendChild(__ifAnchor5);
    __cleanups.push(__wireBoundary_b3(__ifAnchor5));
    const __e15 = document.createElement("span");
    __e15.setAttribute("tabindex", "0");
    __e15.setAttribute("class", "sentinel");
    __e15.setAttribute("aria-hidden", "true");
    attachRef10($scope, "sentinelEnd", __e15);
    __e0.appendChild(__e15);
    __root.appendChild(__e0);
    const __ifAnchor16 = document.createComment("if");
    __root.appendChild(__ifAnchor16);
    __cleanups.push(__wireBoundary_b7(__ifAnchor16));
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

// packages/components/primitives/src/sui-diff/sui-diff.sui
import { adoptSuiSheets as adoptSuiSheets19, attachRef as attachRef11, component as component19, prop as prop19, signal as signal15 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-diff/sui-diff.controller.ts
import { controller as controller15, bindHtml } from "@ship/sui-runtime";
function escape(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function diffLines(a, b) {
  const A = a.split(`
`);
  const B = b.split(`
`);
  const n = A.length;
  const m = B.length;
  const dp = Array.from({ length: n + 1 }, () => new Array(m + 1).fill(0));
  for (let i2 = n - 1;i2 >= 0; i2--) {
    for (let j2 = m - 1;j2 >= 0; j2--) {
      dp[i2][j2] = A[i2] === B[j2] ? dp[i2 + 1][j2 + 1] + 1 : Math.max(dp[i2 + 1][j2], dp[i2][j2 + 1]);
    }
  }
  const out = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (A[i] === B[j]) {
      out.push({ kind: "ctx", text: A[i] });
      i++;
      j++;
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      out.push({ kind: "del", text: A[i] });
      i++;
    } else {
      out.push({ kind: "add", text: B[j] });
      j++;
    }
  }
  while (i < n)
    out.push({ kind: "del", text: A[i++] });
  while (j < m)
    out.push({ kind: "add", text: B[j++] });
  return out;
}
function parseUnified(value) {
  return value.split(`
`).map((line) => {
    if (line.startsWith("@@"))
      return { kind: "hunk", text: line };
    if (line.startsWith("+"))
      return { kind: "add", text: line.slice(1) };
    if (line.startsWith("-"))
      return { kind: "del", text: line.slice(1) };
    return { kind: "ctx", text: line.startsWith(" ") ? line.slice(1) : line };
  });
}
function renderLines(lines) {
  return lines.map((l) => {
    const sign = l.kind === "add" ? "+" : l.kind === "del" ? "-" : l.kind === "hunk" ? "@" : " ";
    const kindLabel = l.kind === "add" ? "added" : l.kind === "del" ? "removed" : l.kind === "hunk" ? "hunk" : "context";
    return `<div class="line ${l.kind}" part="line" role="listitem" aria-label="${kindLabel}: ${escape(l.text)}"><span class="sign" aria-hidden="true">${sign}</span><span>${escape(l.text)}</span></div>`;
  }).join("");
}
var sui_diff_controller_default = controller15({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "region");
    if (!host.hasAttribute("aria-label"))
      host.setAttribute("aria-label", "diff");
    const el = scope.contentEl();
    if (!el)
      return () => {};
    return bindHtml(el, () => {
      const value = scope.value();
      const from = scope.from();
      const to = scope.to();
      let lines;
      if (value) {
        lines = parseUnified(value);
      } else if (from || to) {
        lines = diffLines(from, to);
      } else {
        lines = [];
      }
      return renderLines(lines);
    });
  }
});

// packages/components/primitives/src/sui-diff/sui-diff.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_diff_default = component19("sui-diff", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    from: prop19.string({ default: "", reflect: true }),
    to: prop19.string({ default: "", reflect: true }),
    value: prop19.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }

  .line {
    display: grid;
    grid-template-columns: 16px 1fr;
    padding: 1px 8px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .line.add { background: var(--tone-success-bg); color: var(--foreground); }
  .line.del { background: var(--tone-destructive-bg); color: var(--foreground); }
  .line.hunk { background: var(--tone-info-bg); color: var(--info); }
  .sign { color: var(--muted-foreground); user-select: none; }
  .line.add .sign { color: var(--success); }
  .line.del .sign { color: var(--destructive); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.from = props.from;
    $scope.to = props.to;
    $scope.value = props.value;
    $scope["contentEl"] = signal15(null);
    let __r = null;
    if (sui_diff_controller_default && typeof sui_diff_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_diff_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets19(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "diff-root");
    attachRef11($scope, "contentEl", __e0);
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

// packages/components/primitives/src/sui-divider/sui-divider.sui
import { adoptSuiSheets as adoptSuiSheets20, bindAttr as bindAttr11, component as component20, prop as prop20 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_divider_default = component20("sui-divider", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    orientation: prop20.union(["horizontal", "vertical"], { default: "horizontal", reflect: true }),
    variant: prop20.union(["solid", "dashed"], { default: "solid", reflect: true }),
    tone: prop20.union(["default", "muted", "primary", "destructive"], { default: "default", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --tone-bg: var(--border);
  }
  :host([tone='muted'])       { --tone-bg: color-mix(in oklch, var(--border) 50%, transparent); }
  :host([tone='primary'])     { --tone-bg: color-mix(in oklch, var(--primary) 50%, var(--border)); }
  :host([tone='destructive']) { --tone-bg: color-mix(in oklch, var(--destructive) 50%, var(--border)); }

  .line {
    border: 0;
  }

  /* Horizontal — full-width 1px line. */
  :host(:not([orientation='vertical'])) .line {
    width: 100%;
    height: 1px;
    margin: var(--space-2) 0;
    background: var(--tone-bg);
  }
  :host(:not([orientation='vertical'])[variant='dashed']) .line {
    background: transparent;
    border-top: 1px dashed var(--tone-bg);
    height: 0;
  }

  /* Vertical — inline 1px line that stretches to parent height. */
  :host([orientation='vertical']) {
    display: inline-block;
    vertical-align: middle;
    align-self: stretch;
  }
  :host([orientation='vertical']) .line {
    width: 1px;
    height: 1em;
    margin: 0 var(--space-2);
    display: inline-block;
    background: var(--tone-bg);
  }
  :host([orientation='vertical'][variant='dashed']) .line {
    background: transparent;
    border-left: 1px dashed var(--tone-bg);
    width: 0;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.orientation = props.orientation;
    $scope.variant = props.variant;
    $scope.tone = props.tone;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets20(__root);
    const __e0 = document.createElement("hr");
    __e0.setAttribute("class", "line");
    __e0.setAttribute("part", "line");
    __e0.setAttribute("role", "separator");
    __cleanups.push(bindAttr11(__e0, "aria-orientation", () => $scope.orientation()));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-drawer/sui-drawer.sui
import { adoptSuiSheets as adoptSuiSheets21, attachRef as attachRef12, component as component21, prop as prop21, signal as signal16 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-drawer/sui-drawer.controller.ts
import { controller as controller16, effect as effect20 } from "@ship/sui-runtime";
var _seq5 = 0;
var sui_drawer_controller_default = controller16({
  mount(state, host) {
    const scope = state;
    const uid = ++_seq5;
    const headerId = `sui-drawer-header-${uid}`;
    const bodyId = `sui-drawer-body-${uid}`;
    const dialogEl = scope.dialogEl();
    const closeBtn = scope.closeBtn();
    const sentinelStart = scope.sentinelStart();
    const sentinelEnd = scope.sentinelEnd();
    const headerEl = scope.headerEl();
    const bodyEl = scope.bodyEl();
    if (!dialogEl)
      return;
    if (headerEl)
      headerEl.id = headerId;
    if (bodyEl)
      bodyEl.id = bodyId;
    dialogEl.setAttribute("aria-labelledby", headerId);
    dialogEl.setAttribute("aria-describedby", bodyId);
    let hasScrollLock = false;
    const focusReturn = new FocusReturn;
    function collectFocusables() {
      const shadowFocusables = focusableWithin(dialogEl).filter((el) => !el.classList.contains("sentinel"));
      const lightFocusables = focusableWithin(host);
      return [...lightFocusables, ...shadowFocusables];
    }
    function focusInitial() {
      const autofocus = host.querySelector("[autofocus]") ?? dialogEl.querySelector("[autofocus]");
      if (autofocus) {
        autofocus.focus();
        return;
      }
      const focusables = collectFocusables();
      const primary = focusables.find((el) => el !== closeBtn) ?? focusables[0];
      if (primary)
        primary.focus();
      else
        dialogEl.focus();
    }
    function doOpen() {
      focusReturn.save();
      try {
        dialogEl.showModal();
      } catch {}
      if (!hasScrollLock) {
        acquireScrollLock();
        hasScrollLock = true;
      }
      queueMicrotask(() => focusInitial());
      host.dispatchEvent(new CustomEvent("sui-open", { bubbles: true, composed: true }));
    }
    function doClose() {
      if (dialogEl.open)
        dialogEl.close();
    }
    const onNativeClose = () => {
      if (scope.open())
        scope.open.set(false);
      if (hasScrollLock) {
        releaseScrollLock();
        hasScrollLock = false;
      }
      focusReturn.restore();
      host.dispatchEvent(new CustomEvent("sui-close", { bubbles: true, composed: true }));
    };
    const onBackdropClick = (e) => {
      if (!scope.dismissible())
        return;
      if (e.target === dialogEl)
        scope.open.set(false);
    };
    const onDismiss = () => {
      host.dispatchEvent(new CustomEvent("sui-dismiss", { bubbles: true, composed: true }));
      scope.open.set(false);
    };
    const onSentinelStartFocus = () => {
      const items = collectFocusables();
      if (items.length)
        items[items.length - 1].focus();
    };
    const onSentinelEndFocus = () => {
      const items = collectFocusables();
      if (items.length)
        items[0].focus();
    };
    dialogEl.addEventListener("close", onNativeClose);
    dialogEl.addEventListener("click", onBackdropClick);
    closeBtn?.addEventListener("click", onDismiss);
    sentinelStart?.addEventListener("focus", onSentinelStartFocus);
    sentinelEnd?.addEventListener("focus", onSentinelEndFocus);
    const stopOpenSync = effect20(() => {
      const wantOpen = scope.open();
      host.setAttribute("data-state", wantOpen ? "open" : "closed");
      if (wantOpen && !dialogEl.open)
        doOpen();
      else if (!wantOpen && dialogEl.open)
        doClose();
    });
    host.show = () => scope.open.set(true);
    host.close = () => scope.open.set(false);
    return () => {
      if (hasScrollLock)
        releaseScrollLock();
      dialogEl.removeEventListener("close", onNativeClose);
      dialogEl.removeEventListener("click", onBackdropClick);
      closeBtn?.removeEventListener("click", onDismiss);
      sentinelStart?.removeEventListener("focus", onSentinelStartFocus);
      sentinelEnd?.removeEventListener("focus", onSentinelEndFocus);
      stopOpenSync();
    };
  }
});

// packages/components/primitives/src/sui-drawer/sui-drawer.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_drawer_default = component21("sui-drawer", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    open: prop21.bool({ default: false, reflect: true }),
    side: prop21.union(["right", "left"], { default: "right", reflect: true }),
    size: prop21.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    dismissible: prop21.bool({ default: true })
  },
  shadow: true,
  styles: `
  :host { display: contents; }

  :host(:not([size])), :host([size='md']) { --dr-w: 420px; --dr-px: 24px; --dr-hpy: 16px; --dr-bpy: 16px; --dr-fpy: 14px; }
  :host([size='sm']) { --dr-w: 320px; --dr-px: 20px; --dr-hpy: 14px; --dr-bpy: 14px; --dr-fpy: 12px; }
  :host([size='lg']) { --dr-w: 560px; --dr-px: 28px; --dr-hpy: 20px; --dr-bpy: 20px; --dr-fpy: 16px; }

  dialog {
    position: fixed;
    top: 0; bottom: 0;
    right: 0; left: auto;
    margin: 0; padding: 0;
    border: none;
    border-left: 1px solid var(--border, #e5e5e5);
    background: var(--card, #fff);
    color: var(--card-foreground, #111);
    width: min(var(--dr-w, 420px), calc(100vw - 32px));
    max-width: 100vw;
    height: 100%;
    max-height: 100%;
    box-shadow: var(--shadow-xl, -4px 0 30px rgba(0,0,0,0.15));
    font-family: var(--font-body, system-ui);
    display: flex;
    flex-direction: column;
    opacity: 0;
    transform: translateX(24px);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  :host([side='left']) dialog {
    right: auto; left: 0;
    border-left: none;
    border-right: 1px solid var(--border, #e5e5e5);
    box-shadow: var(--shadow-xl, 4px 0 30px rgba(0,0,0,0.15));
    transform: translateX(-24px);
  }
  dialog[open] { opacity: 1; transform: translateX(0); }

  dialog::backdrop {
    background: hsl(0 0% 0% / 0.5);
    opacity: 0;
    transition: opacity var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  dialog[open]::backdrop { opacity: 1; }

  .sentinel { position: fixed; width: 1px; height: 1px; pointer-events: none; opacity: 0; }

  .header {
    display: flex; align-items: flex-start; gap: var(--space-2, 8px);
    padding: var(--dr-hpy, 16px) var(--dr-px, 24px);
    padding-right: calc(var(--dr-px, 24px) + 36px);
    border-bottom: 1px solid var(--border, #e5e5e5);
    flex: 0 0 auto; position: relative;
  }
  .header-content { flex: 1 1 0; min-width: 0; }
  .title {
    font-size: var(--text-base, 14px);
    font-weight: var(--font-weight-semibold, 600);
    line-height: var(--leading-tight, 1.25);
    color: var(--foreground, #111);
  }
  .description {
    font-size: var(--text-sm, 12px);
    color: var(--muted-foreground, #888);
    margin-top: 2px;
    line-height: var(--leading-normal, 1.5);
  }

  .body {
    flex: 1 1 auto; min-height: 0;
    overflow-y: auto; overscroll-behavior: contain;
    padding: var(--dr-bpy, 16px) var(--dr-px, 24px);
    font-size: var(--text-base, 14px);
    line-height: var(--leading-normal, 1.5);
    scrollbar-width: thin;
  }

  .footer {
    flex: 0 0 auto;
    padding: var(--dr-fpy, 14px) var(--dr-px, 24px);
    border-top: 1px solid var(--border, #e5e5e5);
    display: flex; gap: var(--space-2, 8px);
    justify-content: flex-end; align-items: center;
  }

  .close {
    position: absolute;
    top: 12px; inset-inline-end: 12px;
    width: 28px; height: 28px;
    display: inline-flex; align-items: center; justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md, 6px);
    color: var(--muted-foreground, #888);
    cursor: pointer; padding: 0; font: inherit; z-index: 1;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out),
      color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  .close:hover { background: var(--muted, #f5f5f5); color: var(--foreground, #111); }
  .close svg { width: 14px; height: 14px; display: block; }

  @media (prefers-reduced-motion: reduce) {
    dialog { transition: none; transform: none !important; opacity: 1; }
    dialog::backdrop { transition: none; }
  }
  @media (forced-colors: active) {
    dialog { border: 1px solid CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.side = props.side;
    $scope.size = props.size;
    $scope.dismissible = props.dismissible;
    $scope["bodyEl"] = signal16(null);
    $scope["closeBtn"] = signal16(null);
    $scope["dialogEl"] = signal16(null);
    $scope["headerEl"] = signal16(null);
    $scope["sentinelEnd"] = signal16(null);
    $scope["sentinelStart"] = signal16(null);
    let __r = null;
    if (sui_drawer_controller_default && typeof sui_drawer_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_drawer_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets21(__root);
    const __e0 = document.createElement("dialog");
    __e0.setAttribute("part", "dialog");
    __e0.setAttribute("role", "dialog");
    __e0.setAttribute("aria-modal", "true");
    attachRef12($scope, "dialogEl", __e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("tabindex", "0");
    __e1.setAttribute("class", "sentinel");
    __e1.setAttribute("aria-hidden", "true");
    attachRef12($scope, "sentinelStart", __e1);
    __e0.appendChild(__e1);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "header");
    __e2.setAttribute("part", "header");
    attachRef12($scope, "headerEl", __e2);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "header-content");
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "title");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "title");
    __e4.appendChild(__e5);
    __e3.appendChild(__e4);
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "description");
    const __e7 = document.createElement("slot");
    __e7.setAttribute("name", "description");
    __e6.appendChild(__e7);
    __e3.appendChild(__e6);
    __e2.appendChild(__e3);
    const __e8 = document.createElement("button");
    __e8.setAttribute("class", "close");
    __e8.setAttribute("part", "close");
    __e8.setAttribute("type", "button");
    __e8.setAttribute("aria-label", "Close drawer");
    attachRef12($scope, "closeBtn", __e8);
    const __e9 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    __e9.setAttribute("viewBox", "0 0 16 16");
    __e9.setAttribute("fill", "none");
    __e9.setAttribute("stroke", "currentColor");
    __e9.setAttribute("stroke-width", "1.75");
    __e9.setAttribute("stroke-linecap", "round");
    __e9.setAttribute("stroke-linejoin", "round");
    __e9.setAttribute("aria-hidden", "true");
    __e9.setAttribute("focusable", "false");
    const __e10 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e10.setAttribute("d", "M3.5 3.5l9 9M12.5 3.5l-9 9");
    __e9.appendChild(__e10);
    __e8.appendChild(__e9);
    __e2.appendChild(__e8);
    __e0.appendChild(__e2);
    const __e11 = document.createElement("div");
    __e11.setAttribute("class", "body");
    __e11.setAttribute("part", "body");
    attachRef12($scope, "bodyEl", __e11);
    const __e12 = document.createElement("slot");
    __e11.appendChild(__e12);
    __e0.appendChild(__e11);
    const __e13 = document.createElement("div");
    __e13.setAttribute("class", "footer");
    __e13.setAttribute("part", "footer");
    const __e14 = document.createElement("slot");
    __e14.setAttribute("name", "footer");
    __e13.appendChild(__e14);
    __e0.appendChild(__e13);
    const __e15 = document.createElement("span");
    __e15.setAttribute("tabindex", "0");
    __e15.setAttribute("class", "sentinel");
    __e15.setAttribute("aria-hidden", "true");
    attachRef12($scope, "sentinelEnd", __e15);
    __e0.appendChild(__e15);
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

// packages/components/primitives/src/sui-dropdown-menu/sui-dropdown-menu.sui
import { adoptSuiSheets as adoptSuiSheets22, attachRef as attachRef13, component as component22, prop as prop22, signal as signal17 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-dropdown-menu/sui-dropdown-menu.controller.ts
import { controller as controller17, effect as effect21, bindNativeEvent as bindNativeEvent2 } from "@ship/sui-runtime";
var _seq6 = 0;
var sui_dropdown_menu_controller_default = controller17({
  mount(state, host) {
    const scope = state;
    const panelId = `sui-dropdown-menu-panel-${++_seq6}`;
    const triggerWrap = scope.triggerWrap();
    const triggerSlot = scope.triggerSlot();
    const panelEl = scope.panelEl();
    if (!panelEl)
      return;
    panelEl.id = panelId;
    let typeaheadBuf = "";
    let typeaheadTimer = null;
    function pushTypeahead(key) {
      if (typeaheadTimer)
        clearTimeout(typeaheadTimer);
      typeaheadBuf += key.toLowerCase();
      typeaheadTimer = setTimeout(() => {
        typeaheadBuf = "";
      }, 350);
      const items = getItems();
      const i = items.indexOf(document.activeElement);
      for (let j = 0;j < items.length; j++) {
        const idx = (i + 1 + j) % items.length;
        const label = (items[idx].textContent ?? "").trim().toLowerCase();
        if (label.startsWith(typeaheadBuf))
          return idx;
      }
      return -1;
    }
    function getItems() {
      return Array.from(host.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((el) => !el.hasAttribute("disabled"));
    }
    function focusItem(index) {
      const items = getItems();
      if (!items.length)
        return;
      const i = (index % items.length + items.length) % items.length;
      for (const it of items)
        it.setAttribute("tabindex", "-1");
      items[i].setAttribute("tabindex", "0");
      items[i].focus();
    }
    function currentIndex() {
      return getItems().indexOf(document.activeElement);
    }
    function setupItems() {
      getItems().forEach((el, idx) => {
        if (!el.hasAttribute("role"))
          el.setAttribute("role", "menuitem");
        if (!el.hasAttribute("tabindex"))
          el.setAttribute("tabindex", idx === 0 ? "0" : "-1");
      });
    }
    function setupTrigger() {
      const els = triggerSlot?.assignedElements({ flatten: true }) ?? [];
      for (const el of els) {
        el.setAttribute("aria-haspopup", "menu");
        el.setAttribute("aria-controls", panelId);
        el.setAttribute("aria-expanded", scope.open() ? "true" : "false");
      }
    }
    function openPanel() {
      const trigger = triggerSlot?.assignedElements({ flatten: true })?.[0];
      if (trigger && panelEl) {
        const r = trigger.getBoundingClientRect();
        panelEl.style.top = `${r.bottom + 4}px`;
        panelEl.style.left = `${r.left}px`;
      }
      panelEl?.showPopover?.();
    }
    function closePanel() {
      panelEl?.hidePopover?.();
    }
    const onTriggerClick = () => {
      if (scope.open())
        closePanel();
      else
        openPanel();
    };
    const onToggle = (ev) => {
      const te = ev;
      const nowOpen = te.newState === "open";
      if (nowOpen !== scope.open())
        scope.open.set(nowOpen);
      host.setAttribute("data-state", nowOpen ? "open" : "closed");
      setupTrigger();
      if (nowOpen)
        queueMicrotask(() => {
          setupItems();
          focusItem(0);
        });
    };
    const onItemClick = (e) => {
      const target = e.target.closest("[data-value]");
      if (!target)
        return;
      host.emit("sui-select", { value: target.dataset.value ?? "" });
      closePanel();
    };
    const onKeydown = (e) => {
      if (!scope.open())
        return;
      const items = getItems();
      if (!items.length)
        return;
      const i = currentIndex();
      if (e.key === "ArrowDown") {
        e.preventDefault();
        focusItem(i + 1);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        focusItem(i - 1);
      } else if (e.key === "Home") {
        e.preventDefault();
        focusItem(0);
      } else if (e.key === "End") {
        e.preventDefault();
        focusItem(items.length - 1);
      } else if (e.key === "Escape") {
        e.preventDefault();
        closePanel();
      } else if (e.key === "Tab") {
        closePanel();
      } else if ((e.key === "Enter" || e.key === " ") && i >= 0) {
        e.preventDefault();
        items[i].click();
      } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
        const next = pushTypeahead(e.key);
        if (next >= 0) {
          e.preventDefault();
          focusItem(next);
        }
      }
    };
    if (triggerWrap)
      bindNativeEvent2(triggerWrap, "click", onTriggerClick);
    bindNativeEvent2(panelEl, "toggle", onToggle);
    bindNativeEvent2(panelEl, "click", onItemClick);
    bindNativeEvent2(host, "keydown", onKeydown);
    if (triggerSlot)
      bindNativeEvent2(triggerSlot, "slotchange", setupTrigger);
    bindNativeEvent2(panelEl, "slotchange", setupItems);
    const stopSync = effect21(() => {
      const wantOpen = scope.open();
      host.setAttribute("data-state", wantOpen ? "open" : "closed");
      setupTrigger();
    });
    return () => {
      if (typeaheadTimer)
        clearTimeout(typeaheadTimer);
      stopSync();
    };
  }
});

// packages/components/primitives/src/sui-dropdown-menu/sui-dropdown-menu.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_dropdown_menu_default = component22("sui-dropdown-menu", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    open: prop22.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host { position: relative; display: inline-flex; }

  .panel {
    inset: auto;
    margin: 0;
    min-width: 160px;
    padding: 4px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius, var(--radius-md, 6px));
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    display: flex; flex-direction: column;
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
    opacity: 0;
    transform: translateY(-4px) scale(0.97);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  .panel::backdrop { background: transparent; }

  ::slotted(button), ::slotted([role='menuitem']) {
    text-align: left;
    background: transparent; border: 0;
    padding: 6px 10px;
    border-radius: var(--radius-sm, 4px);
    color: inherit; font: inherit;
    cursor: pointer; width: 100%;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  ::slotted(button:hover), ::slotted([role='menuitem']:hover),
  ::slotted(button:focus-visible), ::slotted([role='menuitem']:focus-visible) {
    background: var(--accent, var(--muted, #f5f5f5));
    outline: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .panel { transition: none; transform: none !important; }
  }
  @media (forced-colors: active) {
    .panel { border: 1px solid CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope["itemSlot"] = signal17(null);
    $scope["panelEl"] = signal17(null);
    $scope["triggerSlot"] = signal17(null);
    $scope["triggerWrap"] = signal17(null);
    let __r = null;
    if (sui_dropdown_menu_controller_default && typeof sui_dropdown_menu_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_dropdown_menu_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets22(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "trigger-wrap");
    attachRef13($scope, "triggerWrap", __e0);
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "trigger");
    attachRef13($scope, "triggerSlot", __e1);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "panel");
    __e2.setAttribute("part", "panel");
    __e2.setAttribute("popover", "auto");
    __e2.setAttribute("role", "menu");
    attachRef13($scope, "panelEl", __e2);
    const __e3 = document.createElement("slot");
    attachRef13($scope, "itemSlot", __e3);
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
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

// packages/components/primitives/src/sui-empty-state/sui-empty-state.sui
import { adoptSuiSheets as adoptSuiSheets23, bindAttr as bindAttr12, bindSlotEmpty as bindSlotEmpty4, component as component23, prop as prop23 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_empty_state_default = component23("sui-empty-state", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    variant: prop23.union(["default", "inline", "placeholder", "connecting"], { default: "default", reflect: true }),
    size: prop23.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    tone: prop23.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "muted", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    color: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
    --es-pad-y: 64px;
    --es-pad-x: 16px;
    --es-gap: var(--space-3, 12px);
    --es-icon: 22px;
    --es-tile: 48px;
    --es-title-fs: var(--text-md, 16px);
    --es-desc-fs: var(--text-sm, 14px);
    --es-radius: var(--radius-lg);
    --es-border: 1px dashed var(--border);
    --es-bg: var(--card);
  }

  :host([size='sm']) {
    --es-pad-y: var(--space-7, 28px);
    --es-pad-x: var(--space-4, 16px);
    --es-gap: var(--space-2, 8px);
    --es-icon: 16px; --es-tile: 36px;
    --es-title-fs: var(--text-base, 14px);
  }
  :host([size='lg']) {
    --es-pad-y: 96px;
    --es-pad-x: var(--space-6, 24px);
    --es-gap: var(--space-4, 16px);
    --es-icon: 28px; --es-tile: 64px;
    --es-title-fs: var(--text-lg, 18px);
    --es-desc-fs: var(--text-base, 15px);
  }

  :host([variant='placeholder']),
  :host([variant='connecting']) {
    --es-border: 0;
    --es-bg: transparent;
    --es-radius: 0;
    --es-pad-y: var(--space-6, 24px);
  }
  :host([variant='inline']) {
    --es-border: 1px dashed var(--border);
    --es-bg: color-mix(in oklch, var(--muted) 55%, transparent);
    --es-radius: var(--radius-md);
    --es-pad-y: var(--space-3, 12px);
    --es-pad-x: var(--space-4, 16px);
    --es-gap: var(--space-3, 12px);
  }

  .box {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--es-gap);
    padding: var(--es-pad-y) var(--es-pad-x);
    text-align: center;
    background: var(--es-bg);
    border: var(--es-border);
    border-radius: var(--es-radius);
  }
  :host([variant='inline']) .box {
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    text-align: left;
  }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: var(--es-tile);
    height: var(--es-tile);
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--tone-fg) 18%, var(--muted));
    color: var(--tone-fg);
    line-height: 1;
  }
  :host(:not([tone])) .icon,
  :host([tone='default']) .icon,
  :host([tone='muted']) .icon {
    background: var(--muted);
    color: var(--muted-foreground);
  }
  :host([data-no-icon]) .icon { display: none; }
  :host([variant='inline']) .icon {
    width: var(--es-icon);
    height: var(--es-icon);
    background: transparent;
  }
  ::slotted([slot='icon']) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: currentColor;
  }
  ::slotted(svg[slot='icon']) {
    width: var(--es-icon);
    height: var(--es-icon);
  }

  .copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
    min-width: 0;
  }
  :host([variant='inline']) .copy { align-items: flex-start; }

  .title {
    font-family: var(--heading-font, var(--font-body));
    font-size: var(--es-title-fs);
    font-weight: var(--heading-weight, 600);
    color: var(--foreground);
    line-height: var(--leading-tight, 1.25);
  }
  :host([variant='connecting']) .title,
  :host([variant='placeholder']) .title {
    font-weight: var(--font-weight-medium);
    color: var(--muted-foreground);
  }

  .desc {
    font-size: var(--es-desc-fs);
    max-width: 40ch;
    color: var(--muted-foreground);
  }
  :host([variant='inline']) .desc { max-width: 60ch; }

  .actions {
    display: flex;
    gap: var(--space-2, 8px);
    justify-content: center;
    margin-top: 4px;
    flex-wrap: wrap;
  }
  :host([variant='inline']) .actions {
    margin-top: 0;
    margin-left: auto;
    justify-content: flex-end;
  }

  @media (forced-colors: active) {
    .box { border-color: CanvasText; }
    .icon { background: Canvas; color: CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.tone = props.tone;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets23(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "box");
    __e0.setAttribute("part", "box");
    __cleanups.push(bindAttr12(__e0, "role", () => $scope.variant() === "connecting" ? "status" : "note"));
    __cleanups.push(bindAttr12(__e0, "aria-live", () => $scope.variant() === "connecting" ? "polite" : "off"));
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "icon");
    __e1.setAttribute("part", "icon");
    __e1.setAttribute("aria-hidden", "true");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "icon");
    __cleanups.push(bindSlotEmpty4(__e2, host, "data-no-icon"));
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "copy");
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "title");
    __e4.setAttribute("part", "title");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "title");
    __e4.appendChild(__e5);
    __e3.appendChild(__e4);
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "desc");
    __e6.setAttribute("part", "desc");
    const __e7 = document.createElement("slot");
    __e6.appendChild(__e7);
    __e3.appendChild(__e6);
    __e0.appendChild(__e3);
    const __e8 = document.createElement("div");
    __e8.setAttribute("class", "actions");
    __e8.setAttribute("part", "actions");
    const __e9 = document.createElement("slot");
    __e9.setAttribute("name", "action");
    __e8.appendChild(__e9);
    __e0.appendChild(__e8);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-error-boundary/sui-error-boundary.sui
import { adoptSuiSheets as adoptSuiSheets24, bindIf as bindIf9, bindText as bindText6, component as component24, on as on3, prop as prop24 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-error-boundary/sui-error-boundary.controller.ts
import { controller as controller18, signal as signal18, attachBoundary, detachBoundary } from "@ship/sui-runtime";
var sui_error_boundary_controller_default = controller18({
  scope: (_baseScope, host) => {
    const caughtError = signal18(null);
    const hasFallbackSlot = signal18(false);
    const reset = () => {
      caughtError.set(null);
      if (host instanceof HTMLElement)
        host.removeAttribute("data-error-message");
    };
    return { caughtError, hasFallbackSlot, reset };
  },
  mount(state, host) {
    const templates = host.querySelectorAll(':scope > template[slot="fallback"]');
    for (const tpl of Array.from(templates)) {
      const wrap = document.createElement("div");
      wrap.setAttribute("slot", "fallback");
      wrap.appendChild(tpl.content.cloneNode(true));
      tpl.replaceWith(wrap);
    }
    state.hasFallbackSlot.set(!!host.querySelector(':scope > [slot="fallback"]'));
    attachBoundary(host, {
      catch(err, source) {
        if (state.caughtError() !== null)
          return;
        const error = err instanceof Error ? err : new Error(String(err));
        host.setAttribute("data-error-message", error.message);
        state.caughtError.set(error);
        host.dispatchEvent(new CustomEvent("sui-error-boundary:caught", {
          bubbles: true,
          composed: true,
          detail: { error, source }
        }));
      },
      reset: state.reset
    });
    host.reset = state.reset;
    return () => {
      detachBoundary(host);
      delete host.reset;
    };
  }
});

// packages/components/primitives/src/sui-error-boundary/sui-error-boundary.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_error_boundary_default = component24("sui-error-boundary", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    "fallback-text": prop24.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
  }

  .fallback {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-2, 8px);
    padding: var(--space-3, 12px) var(--space-4, 16px);
    border: 1px solid color-mix(in oklch, var(--destructive) 30%, var(--border));
    border-radius: var(--radius);
    background: color-mix(in oklch, var(--destructive) 6%, var(--background));
    color: var(--foreground);
    font-size: var(--text-sm, 13px);
  }

  .message {
    color: var(--muted-foreground);
  }

  .retry {
    appearance: none;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 4px 10px;
    font: inherit;
    font-size: var(--text-xs, 12px);
    color: var(--foreground);
    cursor: pointer;
  }
  .retry:hover {
    background: var(--muted);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.fallbackText = props.fallbackText;
    let __r = null;
    if (sui_error_boundary_controller_default && typeof sui_error_boundary_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_error_boundary_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindIf9({
      anchor,
      test: () => {
        if ($scope.hasFallbackSlot())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag7 = document.createDocumentFragment();
          const __c8 = [];
          const __e9 = document.createElement("slot");
          __e9.setAttribute("name", "fallback");
          __frag7.appendChild(__e9);
          return { nodes: Array.from(__frag7.childNodes), cleanup: () => {
            for (const c of __c8)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag10 = document.createDocumentFragment();
          const __c11 = [];
          const __e12 = document.createElement("div");
          __e12.setAttribute("class", "fallback");
          __e12.setAttribute("part", "fallback");
          __e12.setAttribute("role", "alert");
          const __e13 = document.createElement("div");
          __e13.setAttribute("class", "message");
          __e13.setAttribute("part", "message");
          const __t14 = document.createTextNode("");
          __e13.appendChild(__t14);
          __c11.push(bindText6(__t14, () => $scope.fallbackText() || "Something went wrong"));
          __e12.appendChild(__e13);
          const __e15 = document.createElement("button");
          __e15.setAttribute("class", "retry");
          __e15.setAttribute("part", "retry");
          __e15.setAttribute("type", "button");
          __c11.push(on3(__e15, "click", function() {
            const __h = $scope.reset;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t16 = document.createTextNode("Retry");
          __e15.appendChild(__t16);
          __e12.appendChild(__e15);
          __frag10.appendChild(__e12);
          return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
            for (const c of __c11)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b0 = (anchor) => bindIf9({
      anchor,
      test: () => {
        if ($scope.caughtError() === null)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("slot");
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag4 = document.createDocumentFragment();
          const __c5 = [];
          const __ifAnchor6 = document.createComment("if");
          __frag4.appendChild(__ifAnchor6);
          __c5.push(__wireBoundary_b1(__ifAnchor6));
          return { nodes: Array.from(__frag4.childNodes), cleanup: () => {
            for (const c of __c5)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets24(__root);
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

// packages/components/primitives/src/sui-error-state/sui-error-state.sui
import { adoptSuiSheets as adoptSuiSheets25, attachRef as attachRef14, bindAttr as bindAttr13, bindText as bindText7, component as component25, prop as prop25, signal as signal19 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-error-state/sui-error-state.controller.ts
import { controller as controller19, effect as effect22 } from "@ship/sui-runtime";
var sui_error_state_controller_default = controller19({
  mount(state, host) {
    const scope = state;
    let detailsOpen = false;
    const updateToggleLabel = (btn) => {
      btn.textContent = detailsOpen ? "Hide details" : "Show details";
      btn.setAttribute("aria-expanded", detailsOpen ? "true" : "false");
    };
    const stopDetailsSlot = effect22(() => {
      const slot = scope.detailsSlot();
      if (!slot)
        return;
      const check = () => {
        const hasContent = slot.assignedNodes({ flatten: true }).some((n) => n.nodeType === Node.ELEMENT_NODE || n.nodeType === Node.TEXT_NODE && (n.textContent ?? "").trim().length > 0);
        if (hasContent) {
          host.setAttribute("data-has-details", "");
        } else {
          host.removeAttribute("data-has-details");
          host.removeAttribute("data-details-open");
          detailsOpen = false;
          const btn = scope.detailsToggleBtn();
          if (btn)
            updateToggleLabel(btn);
        }
      };
      slot.addEventListener("slotchange", check);
      check();
      return () => slot.removeEventListener("slotchange", check);
    });
    const stopToggleInit = effect22(() => {
      const btn = scope.detailsToggleBtn();
      if (!btn)
        return;
      updateToggleLabel(btn);
      const onClick = () => {
        detailsOpen = !detailsOpen;
        if (detailsOpen) {
          host.setAttribute("data-details-open", "");
        } else {
          host.removeAttribute("data-details-open");
        }
        updateToggleLabel(btn);
      };
      btn.addEventListener("click", onClick);
      return () => btn.removeEventListener("click", onClick);
    });
    const stopActionSlot = effect22(() => {
      const slot = scope.actionSlot();
      if (!slot)
        return;
      const onSlotChange = () => {
        const retry = slot.assignedElements().find((el) => el.getAttribute("slot") === "retry");
        if (retry)
          requestAnimationFrame(() => retry.focus?.());
      };
      slot.addEventListener("slotchange", onSlotChange);
      return () => slot.removeEventListener("slotchange", onSlotChange);
    });
    return () => {
      stopDetailsSlot();
      stopToggleInit();
      stopActionSlot();
    };
  }
});

// packages/components/primitives/src/sui-error-state/sui-error-state.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_error_state_default = component25("sui-error-state", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    title: prop25.string({ default: "" }),
    body: prop25.string({ default: "" }),
    size: prop25.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    variant: prop25.union(["default", "inline"], { default: "default", reflect: true }),
    icon: prop25.string({ default: "alert-circle" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    --tone-fg: var(--destructive);
    --es-pad-y: 64px;
    --es-pad-x: 16px;
    --es-gap: var(--space-3, 12px);
    --es-icon: 22px;
    --es-tile: 48px;
    --es-title-fs: var(--text-md, 16px);
    --es-desc-fs: var(--text-sm, 14px);
  }

  :host([size='sm']) {
    --es-pad-y: var(--space-7, 28px);
    --es-pad-x: var(--space-4, 16px);
    --es-gap: var(--space-2, 8px);
    --es-icon: 16px; --es-tile: 36px;
    --es-title-fs: var(--text-base, 14px);
  }
  :host([size='lg']) {
    --es-pad-y: 96px;
    --es-pad-x: var(--space-6, 24px);
    --es-gap: var(--space-4, 16px);
    --es-icon: 28px; --es-tile: 64px;
    --es-title-fs: var(--text-lg, 18px);
    --es-desc-fs: var(--text-base, 15px);
  }

  .box {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--es-gap);
    padding: var(--es-pad-y) var(--es-pad-x);
    text-align: center;
    background: var(--card);
    border: 1px dashed var(--border);
    border-radius: var(--radius-lg);
  }
  :host([variant='inline']) .box {
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    text-align: left;
    background: color-mix(in oklch, var(--destructive) 4%, var(--card));
    border: 1px solid color-mix(in oklch, var(--destructive) 24%, var(--border));
    border-radius: var(--radius-md);
    padding: var(--space-3, 12px) var(--space-4, 16px);
  }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: var(--es-tile);
    height: var(--es-tile);
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--destructive) 18%, var(--muted));
    color: var(--destructive);
    line-height: 1;
  }
  :host([variant='inline']) .icon {
    width: var(--es-icon);
    height: var(--es-icon);
    background: transparent;
  }

  .copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
    min-width: 0;
  }
  :host([variant='inline']) .copy { align-items: flex-start; }

  .title {
    font-size: var(--es-title-fs);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--foreground);
    line-height: var(--leading-tight, 1.25);
  }

  .desc {
    font-size: var(--es-desc-fs);
    max-width: 60ch;
    color: var(--muted-foreground);
  }

  /* details-toggle: hidden until controller detects slot content */
  .details-toggle {
    display: none;
    appearance: none;
    background: transparent;
    border: 0;
    font: inherit;
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    cursor: pointer;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    margin-top: var(--space-1);
  }
  :host([data-has-details]) .details-toggle { display: inline-block; }
  .details-toggle:hover { color: var(--foreground); background: var(--muted); }

  /* details panel: always in DOM for slotchange detection, shown when open */
  .details {
    display: none;
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: color-mix(in oklch, var(--destructive) 6%, var(--muted));
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    white-space: pre-wrap;
    word-break: break-word;
    max-width: 60ch;
    text-align: left;
  }
  :host([data-has-details][data-details-open]) .details { display: block; }

  .actions {
    display: flex;
    gap: var(--space-2, 8px);
    justify-content: center;
    margin-top: 4px;
    flex-wrap: wrap;
  }
  :host([variant='inline']) .actions {
    margin-top: 0;
    margin-left: auto;
    justify-content: flex-end;
  }

  @media (forced-colors: active) {
    .box { border-color: CanvasText; }
    .icon { background: Canvas; color: CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.title = props.title;
    $scope.body = props.body;
    $scope.size = props.size;
    $scope.variant = props.variant;
    $scope.icon = props.icon;
    $scope["actionSlot"] = signal19(null);
    $scope["detailsPanel"] = signal19(null);
    $scope["detailsSlot"] = signal19(null);
    $scope["detailsToggleBtn"] = signal19(null);
    let __r = null;
    if (sui_error_state_controller_default && typeof sui_error_state_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_error_state_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets25(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "box");
    __e0.setAttribute("part", "box");
    __e0.setAttribute("role", "alert");
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "icon");
    __e1.setAttribute("part", "icon");
    __e1.setAttribute("aria-hidden", "true");
    const __e2 = document.createElement("sui-icon");
    __cleanups.push(bindAttr13(__e2, "name", () => $scope.icon()));
    __e2.setAttribute("size", "md");
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "copy");
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "title");
    __e4.setAttribute("part", "title");
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "title");
    const __t6 = document.createTextNode("");
    __e5.appendChild(__t6);
    __cleanups.push(bindText7(__t6, () => $scope.title() || "Something went wrong"));
    __e4.appendChild(__e5);
    __e3.appendChild(__e4);
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "desc");
    __e7.setAttribute("part", "desc");
    const __e8 = document.createElement("slot");
    const __t9 = document.createTextNode("");
    __e8.appendChild(__t9);
    __cleanups.push(bindText7(__t9, () => $scope.body()));
    __e7.appendChild(__e8);
    __e3.appendChild(__e7);
    const __e10 = document.createElement("button");
    __e10.setAttribute("class", "details-toggle");
    attachRef14($scope, "detailsToggleBtn", __e10);
    __e10.setAttribute("type", "button");
    __e3.appendChild(__e10);
    const __e11 = document.createElement("div");
    __e11.setAttribute("class", "details");
    attachRef14($scope, "detailsPanel", __e11);
    const __e12 = document.createElement("slot");
    __e12.setAttribute("name", "details");
    attachRef14($scope, "detailsSlot", __e12);
    __e11.appendChild(__e12);
    __e3.appendChild(__e11);
    __e0.appendChild(__e3);
    const __e13 = document.createElement("div");
    __e13.setAttribute("class", "actions");
    __e13.setAttribute("part", "actions");
    const __e14 = document.createElement("slot");
    __e14.setAttribute("name", "action");
    attachRef14($scope, "actionSlot", __e14);
    __e13.appendChild(__e14);
    __e0.appendChild(__e13);
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

// packages/components/primitives/src/sui-field/sui-field.sui
import { adoptSuiSheets as adoptSuiSheets26, attachRef as attachRef15, component as component26, prop as prop26, signal as signal20 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-field/sui-field.controller.ts
import { controller as controller20, effect as effect23 } from "@ship/sui-runtime";
var _idSeq = 0;
var sui_field_controller_default = controller20({
  mount(state, host) {
    const scope = state;
    const hintId = `sui-field-hint-${++_idSeq}`;
    const errorId = `sui-field-error-${++_idSeq}`;
    const stopIds = effect23(() => {
      const hint = scope.hintEl();
      const error = scope.errorEl();
      if (hint)
        hint.id = hintId;
      if (error)
        error.id = errorId;
    });
    const wireControl = () => {
      const slot = scope.controlSlot();
      const first = slot?.assignedElements({ flatten: true })[0];
      if (!first)
        return;
      const inv = scope.invalid();
      const targetId = inv ? errorId : hintId;
      const prior = (first.getAttribute("aria-describedby") || "").split(/\s+/).filter((id) => id && id !== hintId && id !== errorId);
      const final = [...prior, targetId].join(" ").trim();
      if (final)
        first.setAttribute("aria-describedby", final);
      if ("invalid" in first)
        first.invalid = inv;
      else
        first.setAttribute("aria-invalid", inv ? "true" : "false");
    };
    const stopInvalid = effect23(() => {
      scope.invalid();
      wireControl();
    });
    const stopSlot = effect23(() => {
      const slot = scope.controlSlot();
      if (!slot)
        return;
      slot.addEventListener("slotchange", wireControl);
      return () => slot.removeEventListener("slotchange", wireControl);
    });
    return () => {
      stopIds();
      stopInvalid();
      stopSlot();
    };
  }
});

// packages/components/primitives/src/sui-field/sui-field.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_field_default = component26("sui-field", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    invalid: prop26.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .hint,
  .error {
    font-family: var(--font-body);
    font-size: 11px;
    line-height: 1.4;
  }
  .hint { color: var(--muted-foreground); }
  .error { color: var(--destructive); }

  :host(:not([invalid])) .error { display: none; }
  :host([invalid]) .hint { display: none; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.invalid = props.invalid;
    $scope["controlSlot"] = signal20(null);
    $scope["errorEl"] = signal20(null);
    $scope["hintEl"] = signal20(null);
    let __r = null;
    if (sui_field_controller_default && typeof sui_field_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_field_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets26(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "label");
    __root.appendChild(__e0);
    const __e1 = document.createElement("slot");
    attachRef15($scope, "controlSlot", __e1);
    __root.appendChild(__e1);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "hint");
    attachRef15($scope, "hintEl", __e2);
    const __e3 = document.createElement("slot");
    __e3.setAttribute("name", "hint");
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "error");
    __e4.setAttribute("role", "alert");
    __e4.setAttribute("aria-live", "polite");
    attachRef15($scope, "errorEl", __e4);
    const __e5 = document.createElement("slot");
    __e5.setAttribute("name", "error");
    __e4.appendChild(__e5);
    __root.appendChild(__e4);
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

// packages/components/primitives/src/sui-file-upload/sui-file-upload.sui
import { adoptSuiSheets as adoptSuiSheets27, attachRef as attachRef16, bindAttr as bindAttr14, bindClass, bindFor as bindFor2, bindIf as bindIf10, bindText as bindText8, component as component27, prop as prop27, signal as signal22 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-file-upload/sui-file-upload.controller.ts
import { controller as controller21, derived as derived2, ref as ref2, signal as signal21 } from "@ship/sui-runtime";
function fmtSize(bytes) {
  if (bytes < 1024)
    return `${bytes} B`;
  if (bytes < 1048576)
    return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1048576).toFixed(1)} MB`;
}
var sui_file_upload_controller_default = controller21({
  scope: () => {
    const files = signal21([]);
    const over = signal21(false);
    const errorMsg = signal21("");
    const dropzoneEl = ref2(null);
    const inputEl = ref2(null);
    const fileListEl = ref2(null);
    const displayFiles = derived2(() => files().map((file, idx) => ({
      file,
      idx,
      key: `${idx}:${file.name}:${file.size}`,
      name: file.name,
      sizeStr: fmtSize(file.size)
    })));
    return { files, over, errorMsg, displayFiles, dropzoneEl, inputEl, fileListEl };
  },
  mount(state, host) {
    const matchesAccept = (file) => {
      const tokens = state.accept().split(",").map((t) => t.trim().toLowerCase()).filter(Boolean);
      if (tokens.length === 0)
        return true;
      const ext = "." + (file.name.split(".").pop() || "").toLowerCase();
      const type = file.type.toLowerCase();
      return tokens.some((t) => {
        if (t.startsWith("."))
          return ext === t;
        if (t.endsWith("/*"))
          return type.startsWith(t.slice(0, -1));
        return type === t;
      });
    };
    const emitChange = (list) => {
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { files: [...list] },
        bubbles: true,
        composed: true
      }));
    };
    const addFiles = (incoming) => {
      state.errorMsg.set("");
      const valid = [];
      for (const f of incoming) {
        if (f.size > state.maxSize()) {
          state.errorMsg.set(`"${f.name}" exceeds max size (${fmtSize(state.maxSize())}).`);
          continue;
        }
        if (state.accept() && !matchesAccept(f)) {
          state.errorMsg.set(`"${f.name}" is not an accepted file type.`);
          continue;
        }
        valid.push(f);
      }
      const cur = state.files();
      const next = state.multiple() ? [...cur, ...valid] : valid.slice(0, 1);
      state.files.set(next);
      emitChange(next);
    };
    const removeAt = (idx) => {
      const cur = state.files();
      if (idx < 0 || idx >= cur.length)
        return;
      const next = cur.filter((_, i) => i !== idx);
      state.files.set(next);
      state.errorMsg.set("");
      emitChange(next);
    };
    const openPicker = () => {
      if (state.disabled())
        return;
      state.inputEl()?.click();
    };
    const onClick = () => openPicker();
    const onKey = (e) => {
      if (state.disabled())
        return;
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        openPicker();
      }
    };
    const onDragOver = (e) => {
      if (state.disabled())
        return;
      e.preventDefault();
      state.over.set(true);
    };
    const onDragLeave = () => state.over.set(false);
    const onDrop = (e) => {
      if (state.disabled())
        return;
      e.preventDefault();
      state.over.set(false);
      const list = e.dataTransfer?.files;
      if (list && list.length)
        addFiles(Array.from(list));
    };
    const onInputChange = (e) => {
      const inp = e.target;
      if (inp.files && inp.files.length)
        addFiles(Array.from(inp.files));
      inp.value = "";
    };
    const dropzoneEl = state.dropzoneEl();
    const inputEl = state.inputEl();
    dropzoneEl?.addEventListener("click", onClick);
    dropzoneEl?.addEventListener("keydown", onKey);
    dropzoneEl?.addEventListener("dragover", onDragOver);
    dropzoneEl?.addEventListener("dragleave", onDragLeave);
    dropzoneEl?.addEventListener("drop", onDrop);
    inputEl?.addEventListener("change", onInputChange);
    const onShadowClick = (e) => {
      const target = e.target;
      const btn = target?.closest?.("button.remove");
      if (!btn || !host.shadowRoot?.contains(btn))
        return;
      const idx = Number(btn.dataset.idx);
      if (Number.isInteger(idx))
        removeAt(idx);
    };
    host.shadowRoot?.addEventListener("click", onShadowClick);
    return () => {
      dropzoneEl?.removeEventListener("click", onClick);
      dropzoneEl?.removeEventListener("keydown", onKey);
      dropzoneEl?.removeEventListener("dragover", onDragOver);
      dropzoneEl?.removeEventListener("dragleave", onDragLeave);
      dropzoneEl?.removeEventListener("drop", onDrop);
      inputEl?.removeEventListener("change", onInputChange);
      host.shadowRoot?.removeEventListener("click", onShadowClick);
    };
  }
});

// packages/components/primitives/src/sui-file-upload/sui-file-upload.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_file_upload_default = component27("sui-file-upload", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    accept: prop27.string({ default: "" }),
    multiple: prop27.bool({ default: false }),
    "max-size": prop27.number({ default: 10485760 }),
    disabled: prop27.bool({ default: false, reflect: true }),
    tone: prop27.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --tone-border: var(--border);
    --tone-bg: color-mix(in oklch, var(--primary) 8%, transparent);
    --tone-ring: var(--primary);
  }
  :host([disabled]) { opacity: 0.5; pointer-events: none; }

  :host([tone='primary'])     { --tone-ring: var(--primary);     --tone-bg: color-mix(in oklch, var(--primary) 10%, transparent); }
  :host([tone='info'])        { --tone-ring: var(--info);         --tone-bg: color-mix(in oklch, var(--info) 10%, transparent); }
  :host([tone='success'])     { --tone-ring: var(--success);      --tone-bg: color-mix(in oklch, var(--success) 10%, transparent); }
  :host([tone='warning'])     { --tone-ring: var(--warning);      --tone-bg: color-mix(in oklch, var(--warning) 10%, transparent); }
  :host([tone='destructive']) { --tone-ring: var(--destructive);  --tone-bg: color-mix(in oklch, var(--destructive) 10%, transparent); }
  :host([tone='accent'])      { --tone-ring: var(--accent);       --tone-bg: color-mix(in oklch, var(--accent) 10%, transparent); }
  :host([tone='muted'])       { --tone-ring: var(--muted-foreground); --tone-bg: var(--muted); }

  .dropzone {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 24px 16px;
    border: 2px dashed var(--border);
    border-radius: var(--radius-md, 6px);
    background: transparent;
    cursor: pointer;
    transition:
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out);
    color: var(--muted-foreground);
    font-family: var(--font-body);
    font-size: 13px;
    user-select: none;
  }
  .dropzone:focus-visible {
    outline: 2px solid var(--tone-ring);
    outline-offset: 2px;
  }
  .dropzone.over {
    border-style: solid;
    border-color: var(--tone-ring);
    background: var(--tone-bg);
  }

  .icon {
    font-size: 1.5rem;
    line-height: 1;
  }

  input[type='file'] {
    display: none;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 8px;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: var(--radius-sm, 4px);
    background: var(--muted);
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--foreground);
  }

  .file-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-size {
    color: var(--muted-foreground);
    flex-shrink: 0;
  }

  .remove {
    appearance: none;
    border: 0;
    background: transparent;
    cursor: pointer;
    color: var(--muted-foreground);
    padding: 2px;
    line-height: 1;
    border-radius: var(--radius-sm, 4px);
    font-size: 16px;
    font-family: inherit;
  }
  .remove:hover {
    color: var(--foreground);
    background: var(--border);
  }
  .remove:focus-visible {
    outline: 2px solid var(--tone-ring);
    outline-offset: 1px;
  }

  .error {
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--destructive);
    margin-top: 4px;
    padding: 0 2px;
  }

  @media (prefers-reduced-motion: reduce) {
    .dropzone { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.accept = props.accept;
    $scope.multiple = props.multiple;
    $scope.maxSize = props.maxSize;
    $scope.disabled = props.disabled;
    $scope.tone = props.tone;
    $scope["dropzoneEl"] = signal22(null);
    $scope["fileListEl"] = signal22(null);
    $scope["inputEl"] = signal22(null);
    let __r = null;
    if (sui_file_upload_controller_default && typeof sui_file_upload_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_file_upload_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b8 = (anchor) => bindIf10({
      anchor,
      test: () => {
        if ($scope.errorMsg())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag7 = document.createDocumentFragment();
          const __c8 = [];
          const __e9 = document.createElement("div");
          __e9.setAttribute("class", "error");
          __e9.setAttribute("role", "alert");
          __e9.setAttribute("part", "error");
          const __t10 = document.createTextNode("");
          __e9.appendChild(__t10);
          __c8.push(bindText8(__t10, () => $scope.errorMsg()));
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
    const __wireBoundary_b12 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.displayFiles(),
      key: (f, __i) => {
        const __child = Object.create($scope);
        __child.f = f;
        __child.__i = __i;
        return (($scope2) => f.key)(__child);
      },
      render: (f, __i) => {
        const __childScope16 = Object.create($scope);
        __childScope16.f = f;
        __childScope16.__i = __i;
        return (($scope2) => {
          const __frag17 = document.createDocumentFragment();
          const __c18 = [];
          const __e19 = document.createElement("div");
          __e19.setAttribute("class", "file-item");
          __e19.setAttribute("part", "file-item");
          const __e20 = document.createElement("span");
          __e20.setAttribute("class", "file-name");
          const __t21 = document.createTextNode("");
          __e20.appendChild(__t21);
          __t21.textContent = $scope2.f.name;
          __e19.appendChild(__e20);
          const __e22 = document.createElement("span");
          __e22.setAttribute("class", "file-size");
          const __t23 = document.createTextNode("");
          __e22.appendChild(__t23);
          __t23.textContent = $scope2.f.sizeStr;
          __e19.appendChild(__e22);
          const __e24 = document.createElement("button");
          __e24.setAttribute("class", "remove");
          __e24.setAttribute("part", "remove");
          __e24.setAttribute("type", "button");
          {
            const __av25 = $scope2.f.idx;
            if (__av25 == null || __av25 === false || __av25 === "")
              __e24.removeAttribute("data-idx");
            else
              __e24.setAttribute("data-idx", __av25 === true ? "" : String(__av25));
          }
          {
            const __av26 = "Remove " + $scope2.f.name;
            if (__av26 == null || __av26 === false || __av26 === "")
              __e24.removeAttribute("aria-label");
            else
              __e24.setAttribute("aria-label", __av26 === true ? "" : String(__av26));
          }
          const __t27 = document.createTextNode("×");
          __e24.appendChild(__t27);
          __e19.appendChild(__e24);
          __frag17.appendChild(__e19);
          return { nodes: Array.from(__frag17.childNodes), cleanup: () => {
            for (const c of __c18)
              try {
                c();
              } catch (e) {}
          }, update: (__new28, __newI29) => {
            const __old = $scope2.f;
            $scope2.f = __new28;
            $scope2.__i = __newI29;
            if (__old.name !== $scope2.f.name) {
              __t21.textContent = $scope2.f.name;
              {
                const __av26 = "Remove " + $scope2.f.name;
                if (__av26 == null || __av26 === false || __av26 === "")
                  __e24.removeAttribute("aria-label");
                else
                  __e24.setAttribute("aria-label", __av26 === true ? "" : String(__av26));
              }
            }
            if (__old.sizeStr !== $scope2.f.sizeStr) {
              __t23.textContent = $scope2.f.sizeStr;
            }
            if (__old.idx !== $scope2.f.idx) {
              {
                const __av25 = $scope2.f.idx;
                if (__av25 == null || __av25 === false || __av25 === "")
                  __e24.removeAttribute("data-idx");
                else
                  __e24.setAttribute("data-idx", __av25 === true ? "" : String(__av25));
              }
            }
          } };
        })(__childScope16);
      }
    });
    const __wireBoundary_b10 = (anchor) => bindIf10({
      anchor,
      test: () => {
        if ($scope.displayFiles().length > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag12 = document.createDocumentFragment();
          const __c13 = [];
          const __e14 = document.createElement("div");
          __e14.setAttribute("class", "file-list");
          __e14.setAttribute("part", "file-list");
          __e14.setAttribute("aria-live", "polite");
          attachRef16($scope, "fileListEl", __e14);
          const __forAnchor15 = document.createComment("for");
          __e14.appendChild(__forAnchor15);
          __c13.push(__wireBoundary_b12(__forAnchor15));
          __frag12.appendChild(__e14);
          return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
            for (const c of __c13)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets27(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "dropzone");
    __cleanups.push(bindClass(__e0, "over", () => $scope.over()));
    __e0.setAttribute("part", "dropzone");
    __e0.setAttribute("role", "button");
    __cleanups.push(bindAttr14(__e0, "tabindex", () => $scope.disabled() ? "-1" : "0"));
    __cleanups.push(bindAttr14(__e0, "aria-label", () => $scope.accept() ? "Upload files (" + $scope.accept() + ")" : "Upload files"));
    __cleanups.push(bindAttr14(__e0, "aria-disabled", () => $scope.disabled() ? "true" : "false"));
    attachRef16($scope, "dropzoneEl", __e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "icon");
    __e1.setAttribute("aria-hidden", "true");
    const __t2 = document.createTextNode("\uD83D\uDCC4");
    __e1.appendChild(__t2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("span");
    const __t4 = document.createTextNode("Drop files here or click to browse");
    __e3.appendChild(__t4);
    __e0.appendChild(__e3);
    __root.appendChild(__e0);
    const __e5 = document.createElement("input");
    __e5.setAttribute("type", "file");
    __e5.setAttribute("part", "input");
    __cleanups.push(bindAttr14(__e5, "accept", () => $scope.accept()));
    __cleanups.push(bindAttr14(__e5, "multiple", () => $scope.multiple()));
    __e5.setAttribute("aria-hidden", "true");
    __e5.setAttribute("tabindex", "-1");
    attachRef16($scope, "inputEl", __e5);
    __root.appendChild(__e5);
    const __ifAnchor6 = document.createComment("if");
    __root.appendChild(__ifAnchor6);
    __cleanups.push(__wireBoundary_b8(__ifAnchor6));
    const __ifAnchor11 = document.createComment("if");
    __root.appendChild(__ifAnchor11);
    __cleanups.push(__wireBoundary_b10(__ifAnchor11));
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

// packages/components/primitives/src/sui-grid/sui-grid.sui
import { adoptSuiSheets as adoptSuiSheets28, component as component28, effect as effect24, prop as prop28 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_grid_default = component28("sui-grid", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    gap: prop28.union(["0", "1", "2", "3", "4", "5", "6", "7", "8"], { default: "4", reflect: true }),
    min: prop28.string({ default: "14rem", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --grid-gap: var(--space-4);
    --grid-min: 14rem;
  }

  :host([gap='0']) { --grid-gap: 0; }
  :host([gap='1']) { --grid-gap: var(--space-1); }
  :host([gap='2']) { --grid-gap: var(--space-2); }
  :host([gap='3']) { --grid-gap: var(--space-3); }
  :host([gap='4']) { --grid-gap: var(--space-4); }
  :host([gap='5']) { --grid-gap: var(--space-5); }
  :host([gap='6']) { --grid-gap: var(--space-6); }
  :host([gap='7']) { --grid-gap: var(--space-7); }
  :host([gap='8']) { --grid-gap: var(--space-8); }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(min(100%, var(--grid-min)), 1fr));
    gap: var(--grid-gap);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.gap = props.gap;
    $scope.min = props.min;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets28(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "grid");
    __cleanups.push(effect24(() => {
      const __v = $scope.min();
      if (__v == null)
        __e0.style.removeProperty("--grid-min");
      else
        __e0.style.setProperty("--grid-min", String(__v));
    }));
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-history-row/sui-history-row.sui
import { adoptSuiSheets as adoptSuiSheets29, bindAttr as bindAttr15, bindIf as bindIf11, bindText as bindText9, component as component29, on as on4, prop as prop29 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-history-row/sui-history-row.controller.ts
import { controller as controller22, derived as derived3 } from "@ship/sui-runtime";
var SUBJECT_RE = /^([a-z]+(?:\([^)]+\))?!?):\s*(.+)$/;
var sui_history_row_controller_default = controller22({
  scope: (state, host) => {
    const s = state;
    const scopePrefix = derived3(() => {
      const m = s.subject().match(SUBJECT_RE);
      return m ? `${m[1]}:` : "";
    });
    const subjectRest = derived3(() => {
      const m = s.subject().match(SUBJECT_RE);
      return m ? m[2] : s.subject();
    });
    const initial = derived3(() => (s.author() || "?").charAt(0).toUpperCase());
    const dotR = derived3(() => s.active() ? 6 : 4.5);
    function emitSelect() {
      host.dispatchEvent(new CustomEvent("sui-select", {
        detail: { sha: s.sha() },
        bubbles: true,
        composed: true
      }));
    }
    return { scopePrefix, subjectRest, initial, dotR, emitSelect };
  },
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "button");
    if (!host.hasAttribute("tabindex"))
      host.setAttribute("tabindex", "0");
    const onKey = (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        host.dispatchEvent(new CustomEvent("sui-select", {
          detail: { sha: scope.sha() },
          bubbles: true,
          composed: true
        }));
      }
    };
    host.addEventListener("keydown", onKey);
    return () => {
      host.removeEventListener("keydown", onKey);
    };
  }
});

// packages/components/primitives/src/sui-history-row/sui-history-row.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_history_row_default = component29("sui-history-row", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    sha: prop29.string({ default: "", reflect: true }),
    subject: prop29.string({ default: "", reflect: true }),
    author: prop29.string({ default: "", reflect: true }),
    timestamp: prop29.string({ default: "", reflect: true }),
    additions: prop29.number({ default: 0, reflect: true }),
    deletions: prop29.number({ default: 0, reflect: true }),
    active: prop29.bool({ default: false, reflect: true }),
    human: prop29.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    align-items: center;
    gap: 0;
    cursor: pointer;
    border-bottom: 1px solid var(--border-subtle);
    transition: background var(--duration-fast) ease;
    font-family: var(--font-body);
  }
  :host(:hover) { background: color-mix(in oklch, var(--muted) 50%, transparent); }
  :host([active]) { background: color-mix(in oklch, var(--primary) 8%, transparent); }

  .lane {
    width: 44px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    align-self: stretch;
  }
  .lane svg { display: block; }

  .avatar {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    font-size: 9px;
    font-weight: 600;
    color: var(--primary-foreground);
    background: color-mix(in oklch, var(--primary) 70%, var(--muted));
    text-transform: uppercase;
  }
  :host([human]) .avatar {
    background: conic-gradient(from 40deg, var(--status-blue, var(--info)), var(--accent));
  }

  .msg {
    flex: 1;
    min-width: 0;
    padding: 8px 6px 8px 10px;
  }
  .title {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--foreground);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.35;
  }
  :host([active]) .title { color: var(--primary); }
  .scope { color: var(--primary); opacity: 0.85; }
  .meta {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-top: 2px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--muted-foreground);
  }

  .stats {
    display: flex;
    gap: 6px;
    padding-right: 14px;
    flex-shrink: 0;
    font-family: var(--font-mono);
    font-size: 10.5px;
  }
  .add { color: var(--success); }
  .del { color: var(--destructive); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.sha = props.sha;
    $scope.subject = props.subject;
    $scope.author = props.author;
    $scope.timestamp = props.timestamp;
    $scope.additions = props.additions;
    $scope.deletions = props.deletions;
    $scope.active = props.active;
    $scope.human = props.human;
    let __r = null;
    if (sui_history_row_controller_default && typeof sui_history_row_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_history_row_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b5 = (anchor) => bindIf11({
      anchor,
      test: () => {
        if ($scope.scopePrefix())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __e11 = document.createElement("span");
          __e11.setAttribute("class", "scope");
          const __t12 = document.createTextNode("");
          __e11.appendChild(__t12);
          __c10.push(bindText9(__t12, () => $scope.scopePrefix()));
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
    const __wireBoundary_b10 = (anchor) => bindIf11({
      anchor,
      test: () => {
        if ($scope.additions() > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag21 = document.createDocumentFragment();
          const __c22 = [];
          const __e23 = document.createElement("span");
          __e23.setAttribute("class", "add");
          const __t24 = document.createTextNode("+");
          __e23.appendChild(__t24);
          const __t25 = document.createTextNode("");
          __e23.appendChild(__t25);
          __c22.push(bindText9(__t25, () => $scope.additions()));
          __frag21.appendChild(__e23);
          return { nodes: Array.from(__frag21.childNodes), cleanup: () => {
            for (const c of __c22)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b12 = (anchor) => bindIf11({
      anchor,
      test: () => {
        if ($scope.deletions() > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag27 = document.createDocumentFragment();
          const __c28 = [];
          const __e29 = document.createElement("span");
          __e29.setAttribute("class", "del");
          const __t30 = document.createTextNode("−");
          __e29.appendChild(__t30);
          const __t31 = document.createTextNode("");
          __e29.appendChild(__t31);
          __c28.push(bindText9(__t31, () => $scope.deletions()));
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets29(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "lane");
    __cleanups.push(on4(__e0, "click", function() {
      const __h = $scope.emitSelect;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e1 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    __e1.setAttribute("width", "44");
    __e1.setAttribute("height", "46");
    __e1.setAttribute("aria-hidden", "true");
    const __e2 = document.createElementNS("http://www.w3.org/2000/svg", "line");
    __e2.setAttribute("x1", "22");
    __e2.setAttribute("y1", "0");
    __e2.setAttribute("x2", "22");
    __e2.setAttribute("y2", "46");
    __e2.setAttribute("stroke", "var(--primary)");
    __e2.setAttribute("stroke-width", "1.5");
    __e2.setAttribute("opacity", "0.45");
    __e1.appendChild(__e2);
    const __e3 = document.createElementNS("http://www.w3.org/2000/svg", "circle");
    __e3.setAttribute("cx", "22");
    __e3.setAttribute("cy", "23");
    __cleanups.push(bindAttr15(__e3, "r", () => $scope.dotR()));
    __cleanups.push(bindAttr15(__e3, "fill", () => $scope.active() ? "var(--primary)" : "var(--background)"));
    __e3.setAttribute("stroke", "var(--primary)");
    __e3.setAttribute("stroke-width", "2");
    __e1.appendChild(__e3);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "avatar");
    const __t5 = document.createTextNode("");
    __e4.appendChild(__t5);
    __cleanups.push(bindText9(__t5, () => $scope.initial()));
    __root.appendChild(__e4);
    const __e6 = document.createElement("div");
    __e6.setAttribute("class", "msg");
    __cleanups.push(on4(__e6, "click", function() {
      const __h = $scope.emitSelect;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "title");
    const __ifAnchor8 = document.createComment("if");
    __e7.appendChild(__ifAnchor8);
    __cleanups.push(__wireBoundary_b5(__ifAnchor8));
    const __t13 = document.createTextNode("");
    __e7.appendChild(__t13);
    __cleanups.push(bindText9(__t13, () => $scope.subjectRest()));
    __e6.appendChild(__e7);
    const __e14 = document.createElement("div");
    __e14.setAttribute("class", "meta");
    const __e15 = document.createElement("span");
    const __t16 = document.createTextNode("");
    __e15.appendChild(__t16);
    __cleanups.push(bindText9(__t16, () => $scope.sha()));
    __e14.appendChild(__e15);
    const __e17 = document.createElement("span");
    const __t18 = document.createTextNode("");
    __e17.appendChild(__t18);
    __cleanups.push(bindText9(__t18, () => $scope.timestamp()));
    __e14.appendChild(__e17);
    __e6.appendChild(__e14);
    __root.appendChild(__e6);
    const __e19 = document.createElement("div");
    __e19.setAttribute("class", "stats");
    const __ifAnchor20 = document.createComment("if");
    __e19.appendChild(__ifAnchor20);
    __cleanups.push(__wireBoundary_b10(__ifAnchor20));
    const __ifAnchor26 = document.createComment("if");
    __e19.appendChild(__ifAnchor26);
    __cleanups.push(__wireBoundary_b12(__ifAnchor26));
    __root.appendChild(__e19);
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

// packages/components/primitives/src/sui-hover-card/sui-hover-card.sui
import { adoptSuiSheets as adoptSuiSheets30, attachRef as attachRef17, component as component30, prop as prop30, signal as signal23 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-hover-card/sui-hover-card.controller.ts
import { controller as controller23, effect as effect25 } from "@ship/sui-runtime";
var _seq7 = 0;
var GAP = 8;
function positionPanel(trigger, panel, placement) {
  const r = trigger.getBoundingClientRect();
  panel.style.top = "";
  panel.style.left = "";
  panel.style.right = "";
  panel.style.bottom = "";
  switch (placement) {
    case "top":
      panel.style.bottom = `${window.innerHeight - r.top + GAP}px`;
      panel.style.left = `${r.left + r.width / 2}px`;
      panel.style.transform = "translateX(-50%)";
      break;
    case "left":
      panel.style.right = `${window.innerWidth - r.left + GAP}px`;
      panel.style.top = `${r.top}px`;
      break;
    case "right":
      panel.style.left = `${r.right + GAP}px`;
      panel.style.top = `${r.top}px`;
      break;
    case "bottom":
    default:
      panel.style.top = `${r.bottom + GAP}px`;
      panel.style.left = `${r.left + r.width / 2}px`;
      panel.style.transform = "translateX(-50%)";
  }
}
var sui_hover_card_controller_default = controller23({
  mount(state, host) {
    const scope = state;
    const cardId = `sui-hover-card-${++_seq7}`;
    const triggerSlot = scope.triggerSlot();
    const panelEl = scope.panelEl();
    if (!panelEl)
      return;
    panelEl.id = cardId;
    let showTimer = null;
    let hideTimer = null;
    function clearTimers() {
      if (showTimer) {
        clearTimeout(showTimer);
        showTimer = null;
      }
      if (hideTimer) {
        clearTimeout(hideTimer);
        hideTimer = null;
      }
    }
    function wireTrigger() {
      const els = triggerSlot?.assignedElements({ flatten: true }) ?? [];
      for (const el of els) {
        el.setAttribute("aria-describedby", cardId);
      }
    }
    function doShow() {
      clearTimers();
      const trigger = triggerSlot?.assignedElements({ flatten: true })?.[0];
      if (trigger && panelEl) {
        positionPanel(trigger, panelEl, scope.placement() || "bottom");
      }
      panelEl?.showPopover?.();
    }
    function doHide() {
      clearTimers();
      panelEl?.hidePopover?.();
    }
    host.show = doShow;
    host.hide = doHide;
    const onToggle = (ev) => {
      const te = ev;
      const nowOpen = te.newState === "open";
      if (nowOpen !== scope.open())
        scope.open.set(nowOpen);
      host.setAttribute("data-state", nowOpen ? "open" : "closed");
      host.dispatchEvent(new CustomEvent(nowOpen ? "sui-open" : "sui-close", {
        bubbles: true,
        composed: true
      }));
    };
    const onPointerEnter = () => {
      clearTimers();
      showTimer = setTimeout(() => doShow(), scope.showDelay() ?? 300);
    };
    const onPointerLeave = () => {
      clearTimers();
      hideTimer = setTimeout(() => doHide(), scope.hideDelay() ?? 100);
    };
    const onFocusIn = () => {
      clearTimers();
      doShow();
    };
    const onFocusOut = () => {
      clearTimers();
      hideTimer = setTimeout(() => doHide(), scope.hideDelay() ?? 100);
    };
    panelEl.addEventListener("toggle", onToggle);
    triggerSlot?.addEventListener("slotchange", wireTrigger);
    wireTrigger();
    host.addEventListener("pointerenter", onPointerEnter);
    host.addEventListener("pointerleave", onPointerLeave);
    host.addEventListener("focusin", onFocusIn);
    host.addEventListener("focusout", onFocusOut);
    const stopSync = effect25(() => {
      host.setAttribute("data-state", scope.open() ? "open" : "closed");
    });
    return () => {
      clearTimers();
      panelEl.removeEventListener("toggle", onToggle);
      triggerSlot?.removeEventListener("slotchange", wireTrigger);
      host.removeEventListener("pointerenter", onPointerEnter);
      host.removeEventListener("pointerleave", onPointerLeave);
      host.removeEventListener("focusin", onFocusIn);
      host.removeEventListener("focusout", onFocusOut);
      stopSync();
    };
  }
});

// packages/components/primitives/src/sui-hover-card/sui-hover-card.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_hover_card_default = component30("sui-hover-card", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    open: prop30.bool({ default: false, reflect: true }),
    placement: prop30.union(["top", "bottom", "left", "right"], { default: "bottom", reflect: true }),
    "show-delay": prop30.number({ default: 300 }),
    "hide-delay": prop30.number({ default: 100 })
  },
  shadow: true,
  styles: `
  :host { display: inline-flex; position: relative; }

  .panel {
    inset: auto;
    margin: 0;
    min-width: 240px;
    max-width: 340px;
    padding: 16px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-lg, 8px);
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
    line-height: var(--leading-normal, 1.5);
    opacity: 0;
    transform: translateY(-4px);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: translateY(0);
  }
  .panel::backdrop { background: transparent; }

  @media (prefers-reduced-motion: reduce) {
    .panel { transition: none; transform: none !important; }
  }
  @media (forced-colors: active) {
    .panel { border: 1px solid CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.placement = props.placement;
    $scope.showDelay = props.showDelay;
    $scope.hideDelay = props.hideDelay;
    $scope["panelEl"] = signal23(null);
    $scope["triggerSlot"] = signal23(null);
    let __r = null;
    if (sui_hover_card_controller_default && typeof sui_hover_card_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_hover_card_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets30(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "trigger");
    attachRef17($scope, "triggerSlot", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "panel");
    __e1.setAttribute("part", "panel");
    __e1.setAttribute("popover", "auto");
    attachRef17($scope, "panelEl", __e1);
    const __e2 = document.createElement("slot");
    __e1.appendChild(__e2);
    __root.appendChild(__e1);
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

// packages/components/primitives/src/sui-icon/sui-icon.sui
import { adoptSuiSheets as adoptSuiSheets31, component as component31, prop as prop31 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-icon/sui-icon.controller.ts
import { controller as controller24, effect as effect26, signal as signal24 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-icon/registry.ts
var ALIASES = {
  git: "git-branch",
  refresh: "refresh-cw",
  stop: "stop-circle",
  warning: "triangle-alert"
};
var ICON_REGISTRY_URL = "/_sui/lucide-icons.json";
var registry = null;
var pending = null;
function getIconRegistry() {
  return registry;
}
function loadIconRegistry() {
  if (registry)
    return Promise.resolve(registry);
  if (pending)
    return pending;
  pending = fetch(ICON_REGISTRY_URL).then((r) => {
    if (!r.ok)
      throw new Error(`lucide registry fetch ${r.status}`);
    return r.json();
  }).then((map) => {
    registry = map;
    return map;
  }).catch((err) => {
    pending = null;
    throw err;
  });
  return pending;
}
function resolveIconName(name) {
  if (!name || !registry)
    return;
  if (registry[name])
    return name;
  const aliased = ALIASES[name];
  if (aliased && registry[aliased])
    return aliased;
  return;
}
function getIconSvg(resolved) {
  return registry ? registry[resolved] : undefined;
}

// packages/components/primitives/src/sui-icon/sui-icon.controller.ts
var SVG_NS = "http://www.w3.org/2000/svg";
var registryRev = signal24(getIconRegistry() ? 1 : 0);
var kicked = false;
function kickFetch() {
  if (kicked || getIconRegistry())
    return;
  kicked = true;
  loadIconRegistry().then(() => registryRev.set(registryRev() + 1)).catch(() => {
    kicked = false;
  });
}
function makeSvg(doc) {
  const svg = doc.createElementNS(SVG_NS, "svg");
  svg.setAttribute("part", "svg");
  svg.setAttribute("class", "sui-icon-svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-width", "2");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  return svg;
}
function populatePaths(svg, fragment) {
  const parsed = new DOMParser().parseFromString(`<svg xmlns="${SVG_NS}">${fragment}</svg>`, "image/svg+xml").documentElement;
  const adopted = [];
  while (parsed.firstChild) {
    adopted.push(svg.ownerDocument.adoptNode(parsed.firstChild));
  }
  svg.replaceChildren(...adopted);
}
var sui_icon_controller_default = controller24({
  mount(state, host) {
    const scope = state;
    const root = host.shadowRoot;
    let svgEl = null;
    let lastResolved = null;
    const stopAria = effect26(() => {
      const label = scope.label();
      if (label) {
        host.setAttribute("role", "img");
        host.setAttribute("aria-label", label);
        host.removeAttribute("aria-hidden");
      } else {
        host.setAttribute("aria-hidden", "true");
        host.removeAttribute("role");
        host.removeAttribute("aria-label");
      }
    });
    const stopRender = effect26(() => {
      registryRev();
      const requested = scope.name();
      const resolved = resolveIconName(requested) ?? null;
      const label = scope.label();
      if (!resolved) {
        if (svgEl) {
          svgEl.remove();
          svgEl = null;
        }
        lastResolved = null;
        if (requested && !getIconRegistry())
          kickFetch();
        return;
      }
      if (!svgEl) {
        svgEl = makeSvg(host.ownerDocument);
        root.insertBefore(svgEl, root.firstChild);
      }
      if (resolved !== lastResolved) {
        populatePaths(svgEl, getIconSvg(resolved));
        lastResolved = resolved;
      }
      if (label)
        svgEl.removeAttribute("aria-hidden");
      else
        svgEl.setAttribute("aria-hidden", "true");
    });
    return () => {
      stopAria();
      stopRender();
    };
  }
});

// packages/components/primitives/src/sui-icon/sui-icon.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_icon_default = component31("sui-icon", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    name: prop31.string({ default: "", reflect: true }),
    size: prop31.union(["xs", "sm", "md", "lg", "xl"], { default: "md", reflect: true }),
    tone: prop31.union(["default", "info", "success", "warning", "destructive", "accent", "muted", "primary"], { default: "default", reflect: true }),
    label: prop31.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    /* default tone; suiTonesSheet (adopted by compiler) overrides per [tone] */
    --tone-fg: var(--foreground);
    color: var(--tone-fg);
  }

  /* Size matrix — applies to the controller-injected SVG */
  svg.sui-icon-svg {
    display: block;
    width: 16px;
    height: 16px;
    flex: 0 0 auto;
  }
  :host([size='xs']) svg.sui-icon-svg { width: 10px; height: 10px; }
  :host([size='sm']) svg.sui-icon-svg { width: 12px; height: 12px; }
  :host([size='lg']) svg.sui-icon-svg { width: 20px; height: 20px; }
  :host([size='xl']) svg.sui-icon-svg { width: 24px; height: 24px; }

  /* Slot passthrough: user-provided SVG fills the icon box */
  ::slotted(svg) { width: 100%; height: 100%; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.name = props.name;
    $scope.size = props.size;
    $scope.tone = props.tone;
    $scope.label = props.label;
    let __r = null;
    if (sui_icon_controller_default && typeof sui_icon_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_icon_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets31(__root);
    const __e0 = document.createElement("slot");
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

// packages/components/primitives/src/sui-input/sui-input.sui
import { adoptSuiSheets as adoptSuiSheets32, attachRef as attachRef18, bindAttr as bindAttr17, bindIf as bindIf12, bindSlotEmpty as bindSlotEmpty5, bindText as bindText10, bindValue, component as component32, prop as prop32, signal as signal25 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-input/sui-input.controller.ts
import { bindAttr as bindAttr16, bindNativeEvent as bindNativeEvent3, controller as controller25, onCleanup } from "@ship/sui-runtime";
var _idSeq2 = 0;
var sui_input_controller_default = controller25({
  mount(state, host) {
    const scope = state;
    const inputEl = scope.inputEl();
    if (!inputEl)
      return;
    host.expose({
      focus: () => inputEl.focus(),
      blur: () => inputEl.blur(),
      clear: () => {
        inputEl.value = "";
        scope.value.set("");
      },
      reportValidity: () => inputEl.reportValidity()
    });
    bindNativeEvent3(inputEl, "input", () => host.emit("sui-input", { value: inputEl.value }));
    bindNativeEvent3(inputEl, "change", () => host.emit("sui-input-change", { value: inputEl.value }));
    const errId = `sui-input-err-${++_idSeq2}`;
    onCleanup(bindAttr16(inputEl, "aria-describedby", () => {
      const parts = [];
      if (scope.invalid() && scope.errorMessage()) {
        const errRef = scope.errorEl;
        const errorEl = typeof errRef === "function" ? errRef() : null;
        if (errorEl) {
          errorEl.id = errId;
          parts.push(errId);
        }
      }
      const extra = scope.describedBy();
      if (extra)
        parts.push(extra);
      return parts.length > 0 ? parts.join(" ") : null;
    }));
  }
});

// packages/components/primitives/src/sui-input/sui-input.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_input_default = component32("sui-input", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    type: prop32.union(["text", "search", "email", "url", "tel", "password", "number"], { default: "text", reflect: true }),
    value: prop32.string({ default: "" }),
    placeholder: prop32.string({ default: "" }),
    name: prop32.string({ default: "" }),
    autocomplete: prop32.string({ default: "" }),
    required: prop32.bool({ default: false, reflect: true }),
    disabled: prop32.bool({ default: false, reflect: true }),
    readonly: prop32.bool({ default: false, reflect: true }),
    invalid: prop32.bool({ default: false, reflect: true }),
    loading: prop32.bool({ default: false, reflect: true }),
    tone: prop32.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop32.union(["default", "ghost"], { default: "default", reflect: true }),
    size: prop32.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    label: prop32.string({ default: "" }),
    "described-by": prop32.string({ default: "" }),
    "error-message": prop32.string({ default: "" })
  },
  shadow: true,
  formAssociated: true,
  formValue: "value",
  formDisabled: "disabled",
  styles: `
  :host {
    display: inline-flex;
    flex-direction: column;
    gap: 4px;
    vertical-align: middle;
    width: 100%;
    --tone-border: var(--border);
    --tone-ring: var(--primary);
    --h: 34px;
    --fs: 13px;
    --px: 10px;
  }

  :host([tone='primary'])     { --tone-border: var(--primary);     --tone-ring: var(--primary); }
  :host([tone='info'])        { --tone-border: var(--info);         --tone-ring: var(--info); }
  :host([tone='success'])     { --tone-border: var(--success);      --tone-ring: var(--success); }
  :host([tone='warning'])     { --tone-border: var(--warning);      --tone-ring: var(--warning); }
  :host([tone='destructive']),
  :host([invalid])            { --tone-border: var(--destructive);  --tone-ring: var(--destructive); }
  :host([tone='accent'])      { --tone-border: var(--accent);       --tone-ring: var(--accent); }
  :host([tone='muted'])       { --tone-border: var(--muted-foreground); --tone-ring: var(--muted-foreground); }

  :host([size='sm']) { --h: 28px; --fs: 12px; --px: 8px; }
  :host([size='md']) { --h: 34px; --fs: 13px; --px: 10px; }
  :host([size='lg']) { --h: 40px; --fs: 14px; --px: 12px; }

  .wrap {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    min-height: var(--h);
    padding: 0 var(--px);
    background: var(--background);
    border: 1px solid var(--tone-border);
    border-radius: var(--radius-md);
    transition:
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  :host([variant='ghost']) .wrap {
    border-color: transparent;
    background: transparent;
  }

  .wrap:focus-within {
    border-color: var(--tone-ring);
    box-shadow: 0 0 0 3px color-mix(in oklch, var(--tone-ring) 25%, transparent);
  }

  :host([disabled]) .wrap {
    background: var(--muted);
    cursor: not-allowed;
    opacity: 0.65;
  }

  input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    padding: 0;
    font: inherit;
    font-family: var(--font-body);
    font-size: var(--fs);
    color: var(--foreground);
  }
  input::placeholder { color: var(--muted-foreground); }
  input:disabled { cursor: not-allowed; }

  /* Keyboard-only focus ring — drawn on the inner input so it is
   * visible even when the wrap container's :focus-within tint is
   * suppressed by themes that flatten ring tokens. Mirrors the
   * sui-button pattern: 2px outline + 2px offset, tone-tinted. */
  input:focus-visible {
    outline: 2px solid var(--tone-ring, var(--ring-color, var(--primary)));
    outline-offset: 2px;
  }
  @media (forced-colors: active) {
    input:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

  ::slotted([slot='leading']),
  ::slotted([slot='trailing']) {
    display: inline-flex;
    align-items: center;
    flex: 0 0 auto;
    color: var(--muted-foreground);
  }

  .spinner {
    display: inline-block;
    flex: 0 0 auto;
    width: 14px;
    height: 14px;
    border: 2px solid color-mix(in oklch, var(--tone-ring) 30%, transparent);
    border-top-color: var(--tone-ring);
    border-radius: 50%;
    animation: sui-spin 0.6s linear infinite;
  }

  @keyframes sui-spin {
    to { transform: rotate(360deg); }
  }

  .error {
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--destructive);
    line-height: 1.4;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.type = props.type;
    $scope.value = props.value;
    $scope.placeholder = props.placeholder;
    $scope.name = props.name;
    $scope.autocomplete = props.autocomplete;
    $scope.required = props.required;
    $scope.disabled = props.disabled;
    $scope.readonly = props.readonly;
    $scope.invalid = props.invalid;
    $scope.loading = props.loading;
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.label = props.label;
    $scope.describedBy = props.describedBy;
    $scope.errorMessage = props.errorMessage;
    $scope["errorEl"] = signal25(null);
    $scope["inputEl"] = signal25(null);
    let __r = null;
    if (sui_input_controller_default && typeof sui_input_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_input_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b13 = (anchor) => bindIf12({
      anchor,
      test: () => {
        if ($scope.loading())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag4 = document.createDocumentFragment();
          const __c5 = [];
          const __e6 = document.createElement("span");
          __e6.setAttribute("class", "spinner");
          __e6.setAttribute("aria-hidden", "true");
          __frag4.appendChild(__e6);
          return { nodes: Array.from(__frag4.childNodes), cleanup: () => {
            for (const c of __c5)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag7 = document.createDocumentFragment();
          const __c8 = [];
          const __e9 = document.createElement("slot");
          __e9.setAttribute("name", "trailing");
          __c8.push(bindSlotEmpty5(__e9, host, "data-no-trailing"));
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
    const __wireBoundary_b14 = (anchor) => bindIf12({
      anchor,
      test: () => {
        if ($scope.invalid() && $scope.errorMessage())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag11 = document.createDocumentFragment();
          const __c12 = [];
          const __e13 = document.createElement("span");
          __e13.setAttribute("class", "error");
          __e13.setAttribute("role", "alert");
          attachRef18($scope, "errorEl", __e13);
          const __t14 = document.createTextNode("");
          __e13.appendChild(__t14);
          __c12.push(bindText10(__t14, () => $scope.errorMessage()));
          __frag11.appendChild(__e13);
          return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
            for (const c of __c12)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets32(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "wrap");
    __e0.setAttribute("part", "wrap");
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "leading");
    __cleanups.push(bindSlotEmpty5(__e1, host, "data-no-leading"));
    __e0.appendChild(__e1);
    const __e2 = document.createElement("input");
    __e2.setAttribute("part", "input");
    __cleanups.push(bindAttr17(__e2, "type", () => $scope.type()));
    __cleanups.push(bindAttr17(__e2, "placeholder", () => $scope.placeholder()));
    __cleanups.push(bindAttr17(__e2, "name", () => $scope.name()));
    __cleanups.push(bindAttr17(__e2, "autocomplete", () => $scope.autocomplete() || "off"));
    __cleanups.push(bindAttr17(__e2, "disabled", () => $scope.disabled()));
    __cleanups.push(bindAttr17(__e2, "readonly", () => $scope.readonly()));
    __cleanups.push(bindAttr17(__e2, "required", () => $scope.required()));
    __cleanups.push(bindAttr17(__e2, "aria-label", () => $scope.label()));
    __cleanups.push(bindAttr17(__e2, "aria-invalid", () => $scope.invalid() ? "true" : "false"));
    __cleanups.push(bindAttr17(__e2, "aria-required", () => $scope.required() ? "true" : "false"));
    __cleanups.push(bindAttr17(__e2, "aria-busy", () => $scope.loading() ? "true" : "false"));
    __cleanups.push(bindValue(__e2, $scope.value));
    attachRef18($scope, "inputEl", __e2);
    __e0.appendChild(__e2);
    const __ifAnchor3 = document.createComment("if");
    __e0.appendChild(__ifAnchor3);
    __cleanups.push(__wireBoundary_b13(__ifAnchor3));
    __root.appendChild(__e0);
    const __ifAnchor10 = document.createComment("if");
    __root.appendChild(__ifAnchor10);
    __cleanups.push(__wireBoundary_b14(__ifAnchor10));
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

// packages/components/primitives/src/sui-json-view/sui-json-view.sui
import { adoptSuiSheets as adoptSuiSheets33, attachRef as attachRef19, component as component33, prop as prop33, signal as signal26 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-json-view/sui-json-view.controller.ts
import { controller as controller26, bindHtml as bindHtml2 } from "@ship/sui-runtime";
function escape2(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function renderValue(v, key, level, expanded, depth) {
  const keyPrefix = key !== null ? `<span class="key">"${escape2(key)}"</span>: ` : "";
  if (v === null)
    return `${keyPrefix}<span class="null">null</span>`;
  if (typeof v === "string")
    return `${keyPrefix}<span class="s">"${escape2(v)}"</span>`;
  if (typeof v === "number")
    return `${keyPrefix}<span class="n">${v}</span>`;
  if (typeof v === "boolean")
    return `${keyPrefix}<span class="b">${String(v)}</span>`;
  const isArr = Array.isArray(v);
  const entries = isArr ? v.map((x, i) => [String(i), x]) : Object.entries(v);
  const open = expanded || level < depth;
  const open1 = isArr ? "[" : "{";
  const close1 = isArr ? "]" : "}";
  const count = entries.length;
  if (count === 0)
    return `${keyPrefix}${open1}${close1}`;
  const children = entries.map(([k, val]) => `<div class="node">${renderValue(val, isArr ? null : k, level + 1, expanded, depth)}</div>`).join("");
  return `<details${open ? " open" : ""}>
<summary><span class="caret">▶</span>${keyPrefix}${open1}<span class="meta">${count} ${isArr ? "items" : "keys"}</span></summary>
<div class="branch">${children}</div>
${close1}</details>`;
}
var sui_json_view_controller_default = controller26({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "tree");
    if (!host.hasAttribute("aria-label"))
      host.setAttribute("aria-label", "JSON tree");
    const el = scope.contentEl();
    if (!el)
      return () => {};
    return bindHtml2(el, () => {
      const data = scope.data();
      const expanded = scope.expanded();
      const depth = scope.depth();
      if (!data)
        return "";
      let parsed;
      try {
        parsed = JSON.parse(data);
      } catch (e) {
        return `<span class="error" part="error" role="alert">Invalid JSON: ${escape2(e.message)}</span>`;
      }
      return `<div part="root">${renderValue(parsed, null, 0, expanded, depth)}</div>`;
    });
  }
});

// packages/components/primitives/src/sui-json-view/sui-json-view.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_json_view_default = component33("sui-json-view", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    data: prop33.string({ default: "", reflect: true }),
    expanded: prop33.bool({ default: false, reflect: true }),
    depth: prop33.number({ default: 1, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
    color: var(--foreground);
  }

  .node { display: block; }
  .branch {
    margin-left: 12px;
    border-left: 1px dashed var(--border);
    padding-left: 10px;
  }
  details > summary {
    list-style: none;
    cursor: pointer;
    user-select: none;
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
  }
  details > summary::-webkit-details-marker { display: none; }
  .caret {
    display: inline-block;
    width: 10px;
    color: var(--muted-foreground);
    transition: transform var(--duration-fast) var(--ease-out);
  }
  details[open] > summary .caret { transform: rotate(90deg); }
  .key { color: var(--primary); font-weight: var(--font-weight-medium); }
  .s { color: var(--success); }
  .n { color: var(--info); }
  .b { color: var(--accent); font-weight: var(--font-weight-medium); }
  .null { color: var(--muted-foreground); font-style: italic; }
  .meta { color: var(--muted-foreground); font-size: var(--text-xs); margin-left: 4px; }
  .error { color: var(--destructive); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.data = props.data;
    $scope.expanded = props.expanded;
    $scope.depth = props.depth;
    $scope["contentEl"] = signal26(null);
    let __r = null;
    if (sui_json_view_controller_default && typeof sui_json_view_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_json_view_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets33(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "json-root");
    attachRef19($scope, "contentEl", __e0);
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

// packages/components/primitives/src/sui-kbd/sui-kbd.sui
import { adoptSuiSheets as adoptSuiSheets34, component as component34, prop as prop34 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-kbd/sui-kbd.controller.ts
import { controller as controller27, effect as effect27 } from "@ship/sui-runtime";
var sui_kbd_controller_default = controller27({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "group");
    const stopSync = effect27(() => {
      const k = scope.keys();
      if (k)
        host.setAttribute("aria-label", `Keyboard shortcut: ${k}`);
      else
        host.removeAttribute("aria-label");
    });
    return () => stopSync();
  }
});

// packages/components/primitives/src/sui-kbd/sui-kbd.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_kbd_default = component34("sui-kbd", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    keys: prop34.string({ default: "" }),
    size: prop34.union(["sm", "md"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    vertical-align: baseline;
  }

  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    padding: 2px 5px;
    font-family: var(--font-mono);
    font-weight: var(--font-weight-medium, 500);
    font-size: 11px;
    line-height: 1.2;
    color: var(--foreground);
    background: var(--muted);
    border: 1px solid var(--border);
    border-bottom-width: 2px;
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-xs);
    white-space: nowrap;
  }

  :host([size='sm']) kbd { font-size: 10px; padding: 1px 4px; min-width: 16px; }
  :host([size='md']) kbd { font-size: 11px; padding: 2px 5px; min-width: 18px; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.keys = props.keys;
    $scope.size = props.size;
    let __r = null;
    if (sui_kbd_controller_default && typeof sui_kbd_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_kbd_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets34(__root);
    const __e0 = document.createElement("kbd");
    __e0.setAttribute("part", "kbd");
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
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

// packages/components/primitives/src/sui-label/sui-label.sui
import { adoptSuiSheets as adoptSuiSheets35, attachRef as attachRef20, bindIf as bindIf13, component as component35, prop as prop35, signal as signal27 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-label/sui-label.controller.ts
import { controller as controller28, effect as effect28 } from "@ship/sui-runtime";
var sui_label_controller_default = controller28({
  mount(state, _host) {
    const scope = state;
    const stopFor = effect28(() => {
      const labelEl = scope.labelEl();
      if (!labelEl)
        return;
      labelEl.htmlFor = scope["for"]?.() ?? "";
    });
    return () => {
      stopFor();
    };
  }
});

// packages/components/primitives/src/sui-label/sui-label.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_label_default = component35("sui-label", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    for: prop35.string({ default: "" }),
    size: prop35.union(["sm", "md"], { default: "sm", reflect: true }),
    required: prop35.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-block;
  }

  label {
    display: inline;
    font-family: var(--font-body);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    cursor: pointer;
  }

  :host(:not([size])) label,
  :host([size='sm']) label { font-size: 10px; }
  :host([size='md']) label { font-size: 11px; }

  .req {
    color: var(--destructive);
    margin-left: 2px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.for = props.for;
    $scope.size = props.size;
    $scope.required = props.required;
    $scope["labelEl"] = signal27(null);
    let __r = null;
    if (sui_label_controller_default && typeof sui_label_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_label_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindIf13({
      anchor,
      test: () => {
        if ($scope.required())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag3 = document.createDocumentFragment();
          const __c4 = [];
          const __e5 = document.createElement("span");
          __e5.setAttribute("class", "req");
          __e5.setAttribute("aria-hidden", "true");
          const __t6 = document.createTextNode("*");
          __e5.appendChild(__t6);
          __frag3.appendChild(__e5);
          return { nodes: Array.from(__frag3.childNodes), cleanup: () => {
            for (const c of __c4)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets35(__root);
    const __e0 = document.createElement("label");
    __e0.setAttribute("part", "label");
    attachRef20($scope, "labelEl", __e0);
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    const __ifAnchor2 = document.createComment("if");
    __e0.appendChild(__ifAnchor2);
    __cleanups.push(__wireBoundary_b1(__ifAnchor2));
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

// packages/components/primitives/src/sui-link/sui-link.sui
import { adoptSuiSheets as adoptSuiSheets36, bindAttr as bindAttr18, bindIf as bindIf14, component as component36, prop as prop36 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_link_default = component36("sui-link", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    href: prop36.string({ default: "" }),
    external: prop36.bool({ default: false, reflect: true }),
    tone: prop36.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop36.union(["default", "underline", "subtle"], { default: "default", reflect: true }),
    target: prop36.string({ default: "" }),
    rel: prop36.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline;
    font-family: var(--font-body);
    --link-fg: var(--primary);
  }

  :host([tone='default'])     { --link-fg: var(--primary); }
  :host([tone='primary'])     { --link-fg: var(--primary); }
  :host([tone='info'])        { --link-fg: var(--info); }
  :host([tone='success'])     { --link-fg: var(--success); }
  :host([tone='warning'])     { --link-fg: var(--warning); }
  :host([tone='destructive']) { --link-fg: var(--destructive); }
  :host([tone='accent'])      { --link-fg: var(--accent); }
  :host([tone='muted'])       { --link-fg: var(--muted-foreground); }

  a {
    color: var(--link-fg);
    text-decoration: none;
    display: inline;
    border-radius: var(--radius-sm);
    transition:
      color var(--duration-fast),
      text-decoration-color var(--duration-fast);
  }
  a:hover {
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  :host([variant='underline']) a {
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 2px;
    text-decoration-color: color-mix(in oklch, var(--link-fg) 40%, transparent);
  }
  :host([variant='subtle']) a:hover {
    color: color-mix(in oklch, var(--link-fg) 80%, var(--foreground));
  }
  a:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: var(--ring-offset);
  }
  .ext {
    display: inline-block;
    margin-left: 2px;
    font-size: 0.9em;
    opacity: 0.8;
    vertical-align: -0.1em;
  }
  .ext svg {
    width: 0.9em;
    height: 0.9em;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.href = props.href;
    $scope.external = props.external;
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.target = props.target;
    $scope.rel = props.rel;
    const __wireBoundary_b3 = (anchor) => bindIf14({
      anchor,
      test: () => {
        if ($scope.external())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag3 = document.createDocumentFragment();
          const __c4 = [];
          const __e5 = document.createElement("span");
          __e5.setAttribute("class", "sr-only");
          const __t6 = document.createTextNode("(opens in new tab)");
          __e5.appendChild(__t6);
          __frag3.appendChild(__e5);
          const __e7 = document.createElement("span");
          __e7.setAttribute("class", "ext");
          __e7.setAttribute("aria-hidden", "true");
          const __e8 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __e8.setAttribute("viewBox", "0 0 24 24");
          __e8.setAttribute("fill", "none");
          __e8.setAttribute("stroke", "currentColor");
          __e8.setAttribute("stroke-width", "2");
          __e8.setAttribute("stroke-linecap", "round");
          __e8.setAttribute("stroke-linejoin", "round");
          const __e9 = document.createElementNS("http://www.w3.org/2000/svg", "path");
          __e9.setAttribute("d", "M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3");
          __e8.appendChild(__e9);
          __e7.appendChild(__e8);
          __frag3.appendChild(__e7);
          return { nodes: Array.from(__frag3.childNodes), cleanup: () => {
            for (const c of __c4)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets36(__root);
    const __e0 = document.createElement("a");
    __e0.setAttribute("part", "link");
    __cleanups.push(bindAttr18(__e0, "href", () => $scope.href()));
    __cleanups.push(bindAttr18(__e0, "target", () => $scope.external() ? "_blank" : $scope.target()));
    __cleanups.push(bindAttr18(__e0, "rel", () => $scope.external() ? "noopener noreferrer" : $scope.rel()));
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    const __ifAnchor2 = document.createComment("if");
    __e0.appendChild(__ifAnchor2);
    __cleanups.push(__wireBoundary_b3(__ifAnchor2));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-list-tile/sui-list-tile.sui
import { adoptSuiSheets as adoptSuiSheets37, attachRef as attachRef21, bindAttr as bindAttr19, bindIf as bindIf15, component as component37, prop as prop37, signal as signal28 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-list-tile/sui-list-tile.controller.ts
import { controller as controller29, effect as effect29 } from "@ship/sui-runtime";
var DIVIDER_CSS = `
sui-list-tile[variant='row'] + sui-list-tile[variant='row'] {
  border-top: 1px solid var(--border-subtle);
}
sui-list-tile[variant='row'][selected] + sui-list-tile[variant='row'],
sui-list-tile[variant='row'] + sui-list-tile[variant='row'][selected] {
  border-top: none;
}
`;
var _dividerInstalled = new WeakSet;
function installDividerStyles(doc) {
  if (_dividerInstalled.has(doc))
    return;
  _dividerInstalled.add(doc);
  try {
    if (typeof CSSStyleSheet !== "undefined" && "replaceSync" in CSSStyleSheet.prototype) {
      const sheet = new CSSStyleSheet;
      sheet.replaceSync(DIVIDER_CSS);
      doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet];
      return;
    }
  } catch {}
  const style = doc.createElement("style");
  style.setAttribute("data-sui-list-tile-dividers", "");
  style.textContent = DIVIDER_CSS;
  doc.head.appendChild(style);
}
var sui_list_tile_controller_default = controller29({
  mount(state, host) {
    const scope = state;
    installDividerStyles(host.ownerDocument);
    const resolveRole = () => {
      if (scope.navigable())
        return "link";
      if (scope.interactive())
        return "button";
      return "listitem";
    };
    const stopRole = effect29(() => {
      scope.navigable();
      scope.interactive();
      host.setAttribute("role", resolveRole());
    });
    const stopSelected = effect29(() => {
      host.setAttribute("aria-selected", scope.selected() ? "true" : "false");
    });
    const stopDisabled = effect29(() => {
      host.setAttribute("aria-disabled", scope.disabled() ? "true" : "false");
    });
    const stopTabindex = effect29(() => {
      const el = scope.tileEl();
      if (!el)
        return;
      const interactive = scope.interactive();
      const disabled = scope.disabled();
      if (interactive && !disabled) {
        el.setAttribute("tabindex", "0");
      } else {
        el.removeAttribute("tabindex");
      }
    });
    const dispatchSelect = () => {
      if (scope.disabled() || !scope.interactive())
        return false;
      const ev = new CustomEvent("sui-select", {
        detail: {},
        bubbles: true,
        composed: true,
        cancelable: true
      });
      host.dispatchEvent(ev);
      return ev.defaultPrevented;
    };
    const onClick = (e) => {
      if (!scope.interactive())
        return;
      const prevented = dispatchSelect();
      if (prevented && scope.navigable()) {
        e.preventDefault();
      }
    };
    const onKeyDown = (e) => {
      if (!scope.interactive() || scope.disabled())
        return;
      if (scope.navigable() && e.key === "Enter")
        return;
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        dispatchSelect();
      }
    };
    const stopHandlers = effect29(() => {
      const el = scope.tileEl();
      if (!el)
        return;
      el.addEventListener("click", onClick);
      el.addEventListener("keydown", onKeyDown);
      return () => {
        el.removeEventListener("click", onClick);
        el.removeEventListener("keydown", onKeyDown);
      };
    });
    return () => {
      stopRole();
      stopSelected();
      stopDisabled();
      stopTabindex();
      stopHandlers();
    };
  }
});

// packages/components/primitives/src/sui-list-tile/sui-list-tile.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_list_tile_default = component37("sui-list-tile", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    variant: prop37.string({ default: "row", reflect: true }),
    interactive: prop37.bool({ default: false, reflect: true }),
    navigable: prop37.string({ default: "", reflect: true }),
    selected: prop37.bool({ default: false, reflect: true }),
    disabled: prop37.bool({ default: false, reflect: true }),
    density: prop37.string({ default: "normal", reflect: true }),
    tone: prop37.string({ default: "default", reflect: true }),
    target: prop37.string({ default: "", reflect: true }),
    rel: prop37.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --pad-y: 10px;
    --pad-x: 10px;
  }
  :host([density='compact']) { --pad-y: 6px;  --pad-x: 8px; }
  :host([density='relaxed']) { --pad-y: 16px; --pad-x: 14px; }

  .tile {
    width: 100%;
    text-align: left;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    padding: var(--pad-y) var(--pad-x);
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: inherit;
    font: inherit;
    text-decoration: none;
    box-sizing: border-box;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }
  a.tile { display: flex; }

  :host([variant='card']) { height: 100%; box-sizing: border-box; }
  :host([variant='card']) .tile {
    min-height: 100%;
    box-sizing: border-box;
  }
  :host([variant='card']) .row { flex: 1; align-items: stretch; }
  :host([variant='card']) ::slotted([slot='leading']) { align-self: flex-start; }
  :host([variant='card']) .primary {
    display: flex;
    flex-direction: column;
    align-self: stretch;
  }

  :host([variant='row']) .tile:hover,
  :host(:not([variant])) .tile:hover {
    background: var(--muted);
    border-color: var(--border);
  }

  :host([variant='card']) .tile {
    background: var(--card);
    border-color: var(--border-subtle);
    box-shadow: var(--shadow-card-rest);
    backdrop-filter: var(--card-backdrop-filter);
    -webkit-backdrop-filter: var(--card-backdrop-filter);
    position: relative;
  }
  :host([variant='card']) .tile::after {
    content: '';
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: var(--card-texture);
    mix-blend-mode: var(--card-texture-blend);
    opacity: var(--card-texture-opacity);
    border-radius: inherit;
    z-index: 0;
  }
  :host([variant='card']) .tile > * { position: relative; z-index: 1; }
  @media (prefers-reduced-motion: reduce) {
    :host([variant='card']) .tile::after { background: none; }
  }
  :host([variant='card']) .tile:hover {
    border-color: var(--primary);
    box-shadow: var(--shadow-hover);
    transform: translateY(-1px);
  }
  @media (prefers-reduced-motion: reduce) {
    :host([variant='card']) .tile:hover { transform: none; }
  }
  :host([variant='card'][interactive]) .tile:active {
    transform: translateY(0);
    box-shadow: var(--shadow-card-rest);
    transition-duration: 60ms;
  }

  .tile:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }
  @media (forced-colors: active) {
    .tile:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

  :host([selected]) .tile {
    background: var(--muted);
    border-color: var(--primary);
  }

  :host([disabled]) .tile {
    opacity: 0.6;
    cursor: not-allowed;
    pointer-events: none;
  }

  :host([interactive]) .tile { cursor: pointer; }

  :host([tone]:not([tone='default'])) .tile {
    border-left: 3px solid var(--tone-fg);
    padding-left: calc(var(--pad-x) - 2px);
  }

  /* tone-fg mapping */
  :host { --tone-fg: var(--foreground); }
  :host([tone='primary'])     { --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone-fg: var(--info); }
  :host([tone='success'])     { --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }

  .row { display: flex; align-items: center; gap: 8px; }
  .primary {
    flex: 1;
    min-width: 0;
    font-weight: var(--font-weight-medium);
    font-size: var(--text-base);
  }
  .secondary {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 10px;
    color: var(--muted-foreground);
  }
  .secondary:empty { display: none; }

  ::slotted([slot='leading']),
  ::slotted([slot='trailing']) {
    display: inline-flex;
    align-items: center;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.variant = props.variant;
    $scope.interactive = props.interactive;
    $scope.navigable = props.navigable;
    $scope.selected = props.selected;
    $scope.disabled = props.disabled;
    $scope.density = props.density;
    $scope.tone = props.tone;
    $scope.target = props.target;
    $scope.rel = props.rel;
    $scope["tileEl"] = signal28(null);
    let __r = null;
    if (sui_list_tile_controller_default && typeof sui_list_tile_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_list_tile_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindIf15({
      anchor,
      test: () => {
        if ($scope.navigable())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("a");
          __e3.setAttribute("class", "tile");
          __e3.setAttribute("part", "button");
          attachRef21($scope, "tileEl", __e3);
          __c2.push(bindAttr19(__e3, "href", () => $scope.navigable()));
          __c2.push(bindAttr19(__e3, "target", () => $scope.target() || null));
          __c2.push(bindAttr19(__e3, "rel", () => $scope.rel() || null));
          const __e4 = document.createElement("div");
          __e4.setAttribute("class", "row");
          __e4.setAttribute("part", "row");
          const __e5 = document.createElement("slot");
          __e5.setAttribute("name", "leading");
          __e4.appendChild(__e5);
          const __e6 = document.createElement("span");
          __e6.setAttribute("class", "primary");
          __e6.setAttribute("part", "primary");
          const __e7 = document.createElement("slot");
          __e6.appendChild(__e7);
          __e4.appendChild(__e6);
          const __e8 = document.createElement("span");
          __e8.setAttribute("part", "trailing");
          const __e9 = document.createElement("slot");
          __e9.setAttribute("name", "trailing");
          __e8.appendChild(__e9);
          __e4.appendChild(__e8);
          __e3.appendChild(__e4);
          const __e10 = document.createElement("div");
          __e10.setAttribute("class", "secondary");
          __e10.setAttribute("part", "secondary");
          const __e11 = document.createElement("slot");
          __e11.setAttribute("name", "meta");
          __e10.appendChild(__e11);
          __e3.appendChild(__e10);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag12 = document.createDocumentFragment();
          const __c13 = [];
          const __e14 = document.createElement("div");
          __e14.setAttribute("class", "tile");
          __e14.setAttribute("part", "button");
          attachRef21($scope, "tileEl", __e14);
          const __e15 = document.createElement("div");
          __e15.setAttribute("class", "row");
          __e15.setAttribute("part", "row");
          const __e16 = document.createElement("slot");
          __e16.setAttribute("name", "leading");
          __e15.appendChild(__e16);
          const __e17 = document.createElement("span");
          __e17.setAttribute("class", "primary");
          __e17.setAttribute("part", "primary");
          const __e18 = document.createElement("slot");
          __e17.appendChild(__e18);
          __e15.appendChild(__e17);
          const __e19 = document.createElement("span");
          __e19.setAttribute("part", "trailing");
          const __e20 = document.createElement("slot");
          __e20.setAttribute("name", "trailing");
          __e19.appendChild(__e20);
          __e15.appendChild(__e19);
          __e14.appendChild(__e15);
          const __e21 = document.createElement("div");
          __e21.setAttribute("class", "secondary");
          __e21.setAttribute("part", "secondary");
          const __e22 = document.createElement("slot");
          __e22.setAttribute("name", "meta");
          __e21.appendChild(__e22);
          __e14.appendChild(__e21);
          __frag12.appendChild(__e14);
          return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
            for (const c of __c13)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets37(__root);
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

// packages/components/primitives/src/sui-loading/sui-loading.sui
import { adoptSuiSheets as adoptSuiSheets38, attachRef as attachRef22, component as component38, prop as prop38, signal as signal29 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-loading/sui-loading.controller.ts
import { controller as controller30 } from "@ship/sui-runtime";
var DEFAULT_MESSAGES = ["Waking daemon", "Warming routes", "Fetching state", "Hydrating shell"];
function parseMessages(raw) {
  const trimmed = raw.trim();
  if (!trimmed)
    return DEFAULT_MESSAGES;
  return trimmed.split("|").map((s) => s.trim()).filter(Boolean);
}
var sui_loading_controller_default = controller30({
  mount(state, _host) {
    const scope = state;
    const msgEl = scope.msgEl();
    if (!msgEl)
      return;
    let index = 0;
    const showMsg = () => {
      const msgs = parseMessages(scope.messages());
      msgEl.textContent = msgs[index] ?? "";
      msgEl.style.opacity = "1";
    };
    showMsg();
    const timer = setInterval(() => {
      msgEl.style.opacity = "0";
      requestAnimationFrame(() => {
        const msgs = parseMessages(scope.messages());
        index = (index + 1) % msgs.length;
        msgEl.textContent = msgs[index] ?? "";
        msgEl.style.opacity = "1";
      });
    }, 1400);
    return () => clearInterval(timer);
  }
});

// packages/components/primitives/src/sui-loading/sui-loading.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_loading_default = component38("sui-loading", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    messages: prop38.string({ default: "" }),
    compact: prop38.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3, 12px);
    min-height: 240px;
    width: 100%;
    color: var(--muted-foreground);
  }
  :host([compact]) {
    min-height: 80px;
    gap: 6px;
  }

  /* Stage: fixed footprint so bob/flame animations don't shift the message. */
  .stage {
    position: relative;
    width: 56px;
    height: 64px;
    display: grid;
    place-items: center;
  }
  :host([compact]) .stage {
    width: 28px;
    height: 32px;
  }

  .rocket {
    position: relative;
    z-index: 2;
    width: 32px;
    height: 32px;
    color: var(--primary);
    filter: drop-shadow(0 4px 6px color-mix(in oklch, var(--primary) 35%, transparent));
    animation: sui-loading-rocket 2.2s ease-in-out infinite;
    transform-origin: 50% 100%;
  }
  :host([compact]) .rocket {
    width: 18px;
    height: 18px;
    filter: none;
  }
  .rocket svg {
    width: 100%;
    height: 100%;
  }

  .flame {
    position: absolute;
    left: 50%;
    bottom: 6px;
    transform: translateX(-50%);
    width: 14px;
    height: 18px;
    z-index: 1;
    background: radial-gradient(
      ellipse at 50% 30%,
      color-mix(in oklch, var(--primary) 95%, white) 0%,
      color-mix(in oklch, var(--primary) 70%, transparent) 55%,
      transparent 80%
    );
    border-radius: 50% 50% 35% 35% / 60% 60% 35% 35%;
    filter: blur(0.5px);
    transform-origin: 50% 0%;
    animation:
      sui-loading-flame 0.18s ease-in-out infinite alternate,
      sui-loading-flame-pulse 2.2s ease-in-out infinite;
  }
  :host([compact]) .flame {
    width: 8px;
    height: 10px;
    bottom: 4px;
  }

  .particle {
    position: absolute;
    left: 50%;
    top: 75%;
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: color-mix(in oklch, var(--primary) 85%, transparent);
    opacity: 0;
    z-index: 0;
  }
  .p1 { animation: sui-loading-particle 1.4s ease-out 0s    infinite; }
  .p2 { animation: sui-loading-particle 1.7s ease-out 0.4s  infinite; width: 2px; height: 2px; transform: translate(-50%, 0); }
  .p3 { animation: sui-loading-particle 1.5s ease-out 0.8s  infinite; width: 4px; height: 4px; }
  :host([compact]) .particle { display: none; }

  .message {
    font-size: var(--text-sm);
    color: var(--muted-foreground);
    transition: opacity var(--duration-medium, 220ms) var(--ease-out, ease);
    letter-spacing: 0.01em;
  }
  :host([compact]) .message { font-size: var(--text-xs); }

  @keyframes sui-loading-rocket {
    0%   { transform: translateY(0)    rotate(0deg);    }
    40%  { transform: translateY(-6px) rotate(-1.5deg); }
    60%  { transform: translateY(-7px) rotate(1.5deg);  }
    100% { transform: translateY(0)    rotate(0deg);    }
  }
  @keyframes sui-loading-flame {
    0%   { transform: translateX(-50%) scaleY(1)    scaleX(1);    opacity: 0.9; }
    100% { transform: translateX(-50%) scaleY(1.15) scaleX(0.92); opacity: 1;   }
  }
  @keyframes sui-loading-flame-pulse {
    0%, 100% { height: 16px; }
    50%      { height: 22px; }
  }
  @keyframes sui-loading-particle {
    0%   { transform: translate(-50%, 0)    scale(1);   opacity: 0.7; }
    80%  { transform: translate(-50%, 22px) scale(0.5); opacity: 0;   }
    100% { transform: translate(-50%, 26px) scale(0.3); opacity: 0;   }
  }

  @media (prefers-reduced-motion: reduce) {
    .rocket, .flame, .particle { animation: none; }
    .flame { opacity: 0.7; }
    .particle { display: none; }
    .message { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.messages = props.messages;
    $scope.compact = props.compact;
    $scope["msgEl"] = signal29(null);
    let __r = null;
    if (sui_loading_controller_default && typeof sui_loading_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_loading_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets38(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "stage");
    __e0.setAttribute("part", "stage");
    __e0.setAttribute("aria-hidden", "true");
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "rocket");
    const __e2 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    __e2.setAttribute("viewBox", "0 0 32 40");
    __e2.setAttribute("fill", "none");
    __e2.setAttribute("xmlns", "http://www.w3.org/2000/svg");
    const __e3 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e3.setAttribute("d", "M16 2 C20 6 22 11 22 16 H10 C10 11 12 6 16 2 Z");
    __e3.setAttribute("fill", "currentColor");
    __e2.appendChild(__e3);
    const __e4 = document.createElementNS("http://www.w3.org/2000/svg", "rect");
    __e4.setAttribute("x", "10");
    __e4.setAttribute("y", "16");
    __e4.setAttribute("width", "12");
    __e4.setAttribute("height", "14");
    __e4.setAttribute("rx", "2");
    __e4.setAttribute("fill", "currentColor");
    __e4.setAttribute("opacity", "0.9");
    __e2.appendChild(__e4);
    const __e5 = document.createElementNS("http://www.w3.org/2000/svg", "circle");
    __e5.setAttribute("cx", "16");
    __e5.setAttribute("cy", "20");
    __e5.setAttribute("r", "2.5");
    __e5.setAttribute("fill", "var(--background, #fff)");
    __e5.setAttribute("opacity", "0.85");
    __e2.appendChild(__e5);
    const __e6 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e6.setAttribute("d", "M10 24 L4 32 L10 30 Z");
    __e6.setAttribute("fill", "currentColor");
    __e6.setAttribute("opacity", "0.75");
    __e2.appendChild(__e6);
    const __e7 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e7.setAttribute("d", "M22 24 L28 32 L22 30 Z");
    __e7.setAttribute("fill", "currentColor");
    __e7.setAttribute("opacity", "0.75");
    __e2.appendChild(__e7);
    const __e8 = document.createElementNS("http://www.w3.org/2000/svg", "rect");
    __e8.setAttribute("x", "13");
    __e8.setAttribute("y", "29");
    __e8.setAttribute("width", "6");
    __e8.setAttribute("height", "2");
    __e8.setAttribute("rx", "0.5");
    __e8.setAttribute("fill", "var(--background, #000)");
    __e8.setAttribute("opacity", "0.5");
    __e2.appendChild(__e8);
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e9 = document.createElement("div");
    __e9.setAttribute("class", "flame");
    __e0.appendChild(__e9);
    const __e10 = document.createElement("div");
    __e10.setAttribute("class", "particle p1");
    __e0.appendChild(__e10);
    const __e11 = document.createElement("div");
    __e11.setAttribute("class", "particle p2");
    __e0.appendChild(__e11);
    const __e12 = document.createElement("div");
    __e12.setAttribute("class", "particle p3");
    __e0.appendChild(__e12);
    __root.appendChild(__e0);
    const __e13 = document.createElement("div");
    __e13.setAttribute("class", "message");
    __e13.setAttribute("part", "message");
    attachRef22($scope, "msgEl", __e13);
    __root.appendChild(__e13);
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

// packages/components/primitives/src/sui-mark/sui-mark.sui
import { adoptSuiSheets as adoptSuiSheets39, component as component39, prop as prop39 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_mark_default = component39("sui-mark", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    tone: prop39.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "warning", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline;
    --tone: var(--warning);
    --tone-fg: var(--warning);
  }
  mark {
    background: color-mix(in oklch, var(--tone) 20%, transparent);
    color: var(--tone-fg);
    padding: 0 2px;
    border-radius: 2px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets39(__root);
    const __e0 = document.createElement("mark");
    __e0.setAttribute("part", "mark");
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-markdown/sui-markdown.sui
import { adoptSuiSheets as adoptSuiSheets40, attachRef as attachRef23, component as component40, prop as prop40, signal as signal30 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-markdown/sui-markdown.controller.ts
import { controller as controller31, bindHtml as bindHtml3 } from "@ship/sui-runtime";
function escape3(s) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function inline(s) {
  let out = escape3(s);
  out = out.replace(/`([^`]+)`/g, (_, c) => `<code>${c}</code>`);
  out = out.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, "$1<em>$2</em>");
  out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, t, h) => `<a href="${h}" rel="noopener noreferrer">${t}</a>`);
  return out;
}
function renderMarkdown(src) {
  const lines = src.replace(/\r\n/g, `
`).split(`
`);
  const out = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (/^```/.test(line)) {
      const lang = line.slice(3).trim();
      const buf2 = [];
      i++;
      while (i < lines.length && !/^```/.test(lines[i]))
        buf2.push(lines[i++]);
      i++;
      out.push(`<pre data-lang="${escape3(lang)}"><code>${escape3(buf2.join(`
`))}</code></pre>`);
      continue;
    }
    const h = /^(#{1,6})\s+(.*)$/.exec(line);
    if (h) {
      out.push(`<h${h[1].length}>${inline(h[2])}</h${h[1].length}>`);
      i++;
      continue;
    }
    if (/^\s*[-*]\s+/.test(line)) {
      const buf2 = [];
      while (i < lines.length && /^\s*[-*]\s+/.test(lines[i])) {
        buf2.push(`<li>${inline(lines[i].replace(/^\s*[-*]\s+/, ""))}</li>`);
        i++;
      }
      out.push(`<ul>${buf2.join("")}</ul>`);
      continue;
    }
    if (/^\s*\d+\.\s+/.test(line)) {
      const buf2 = [];
      while (i < lines.length && /^\s*\d+\.\s+/.test(lines[i])) {
        buf2.push(`<li>${inline(lines[i].replace(/^\s*\d+\.\s+/, ""))}</li>`);
        i++;
      }
      out.push(`<ol>${buf2.join("")}</ol>`);
      continue;
    }
    if (/^>\s?/.test(line)) {
      const buf2 = [];
      while (i < lines.length && /^>\s?/.test(lines[i])) {
        buf2.push(lines[i].replace(/^>\s?/, ""));
        i++;
      }
      out.push(`<blockquote>${inline(buf2.join(" "))}</blockquote>`);
      continue;
    }
    if (line.trim() === "") {
      i++;
      continue;
    }
    const buf = [];
    while (i < lines.length && lines[i].trim() !== "" && !/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(lines[i])) {
      buf.push(lines[i]);
      i++;
    }
    out.push(`<p>${inline(buf.join(" "))}</p>`);
  }
  return out.join(`
`);
}
var sui_markdown_controller_default = controller31({
  mount(state) {
    const scope = state;
    const el = scope.contentEl();
    if (!el)
      return () => {};
    return bindHtml3(el, () => renderMarkdown(scope.value() ?? ""));
  }
});

// packages/components/primitives/src/sui-markdown/sui-markdown.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_markdown_default = component40("sui-markdown", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    value: prop40.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    color: var(--foreground);
  }

  h1, h2, h3, h4, h5, h6 {
    font-family: var(--heading-font, var(--font-body));
    font-weight: var(--font-weight-semibold);
    line-height: var(--leading-tight);
    margin: 1em 0 0.4em;
  }
  h1 { font-size: var(--text-2xl); }
  h2 { font-size: var(--text-xl); }
  h3 { font-size: var(--text-lg); }
  h4, h5, h6 { font-size: var(--text-md); }
  p { margin: 0.5em 0; }
  a { color: var(--primary); text-decoration: underline; text-underline-offset: 2px; }
  code {
    font-family: var(--font-mono);
    font-size: 0.92em;
    padding: 1px 5px;
    border-radius: var(--radius-sm);
    background: var(--muted);
    border: 1px solid var(--border);
  }
  pre {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 10px 12px;
    overflow-x: auto;
  }
  pre code { background: transparent; border: none; padding: 0; }
  blockquote {
    margin: 0.6em 0;
    padding: 4px 12px;
    border-left: 3px solid var(--border);
    color: var(--muted-foreground);
  }
  ul, ol { margin: 0.4em 0; padding-left: 22px; }
  li { margin: 2px 0; }
  strong { font-weight: var(--font-weight-semibold); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope["contentEl"] = signal30(null);
    let __r = null;
    if (sui_markdown_controller_default && typeof sui_markdown_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_markdown_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets40(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "md-root");
    attachRef23($scope, "contentEl", __e0);
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

// packages/components/primitives/src/sui-metachip/sui-metachip.sui
import { adoptSuiSheets as adoptSuiSheets41, bindText as bindText11, component as component41, prop as prop41 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_metachip_default = component41("sui-metachip", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    label: prop41.string({ default: "", reflect: true }),
    value: prop41.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: baseline;
    gap: 8px;
    padding: 4px 14px;
    border-right: 1px solid color-mix(in oklch, var(--border) 70%, transparent);
    font-size: var(--text-xs);
    line-height: var(--leading-tight);
  }
  :host(:last-of-type) { border-right: 0; padding-right: 0; }
  :host(:first-of-type) { padding-left: 0; }
  .k {
    font-size: 10px;
    font-weight: var(--font-weight-semibold);
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .v {
    font-family: var(--font-mono);
    color: var(--foreground);
    font-size: var(--text-xs);
  }
  ::slotted(*) { color: inherit; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.label = props.label;
    $scope.value = props.value;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets41(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "k");
    __e0.setAttribute("part", "label");
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "label");
    const __t2 = document.createTextNode("");
    __e1.appendChild(__t2);
    __cleanups.push(bindText11(__t2, () => $scope.label()));
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "v");
    __e3.setAttribute("part", "value");
    const __e4 = document.createElement("slot");
    const __t5 = document.createTextNode("");
    __e4.appendChild(__t5);
    __cleanups.push(bindText11(__t5, () => $scope.value()));
    __e3.appendChild(__e4);
    __root.appendChild(__e3);
    return $scope;
  }
});

// packages/components/primitives/src/sui-navigation-menu/sui-navigation-menu.sui
import { adoptSuiSheets as adoptSuiSheets42, attachRef as attachRef24, component as component42, prop as prop42, signal as signal31 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-navigation-menu/sui-navigation-menu.controller.ts
import { controller as controller32, effect as effect30 } from "@ship/sui-runtime";
var sui_navigation_menu_controller_default = controller32({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "navigation");
    const getItems = () => Array.from(host.querySelectorAll(":scope > sui-navigation-item"));
    const enabledItems = () => getItems().filter((i) => !i.hasAttribute("disabled"));
    const syncItems = () => {
      const val = scope.value();
      for (const item of getItems()) {
        item.toggleAttribute("active", item.getAttribute("value") === val);
      }
    };
    const onNavSelect = (e) => {
      const ce = e;
      ce.stopPropagation();
      const val = ce.detail.value;
      scope.value.set(val);
      syncItems();
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: val },
        bubbles: true,
        composed: true
      }));
    };
    const onKeydown = (e) => {
      if (e.key !== "ArrowRight" && e.key !== "ArrowLeft" && e.key !== "Home" && e.key !== "End")
        return;
      const items = enabledItems();
      if (!items.length)
        return;
      const current = e.composedPath().find((n) => items.includes(n));
      const i = current ? items.indexOf(current) : -1;
      let target;
      if (e.key === "ArrowRight")
        target = items[(i + 1) % items.length];
      else if (e.key === "ArrowLeft")
        target = items[(i - 1 + items.length) % items.length];
      else if (e.key === "Home")
        target = items[0];
      else if (e.key === "End")
        target = items[items.length - 1];
      if (target) {
        e.preventDefault();
        target.focusLink?.();
      }
    };
    const stopSync = effect30(() => {
      scope.value();
      syncItems();
    });
    const stopSlot = effect30(() => {
      const slot = scope.defaultSlotEl();
      if (!slot)
        return;
      const onSlotChange = () => syncItems();
      slot.addEventListener("slotchange", onSlotChange);
      return () => slot.removeEventListener("slotchange", onSlotChange);
    });
    host.addEventListener("sui-nav-select", onNavSelect);
    host.addEventListener("keydown", onKeydown);
    return () => {
      stopSync();
      stopSlot();
      host.removeEventListener("sui-nav-select", onNavSelect);
      host.removeEventListener("keydown", onKeydown);
    };
  }
});

// packages/components/primitives/src/sui-navigation-menu/sui-navigation-menu.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_navigation_menu_default = component42("sui-navigation-menu", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    value: prop42.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    align-items: stretch;
    gap: 0;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-body);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope["defaultSlotEl"] = signal31(null);
    let __r = null;
    if (sui_navigation_menu_controller_default && typeof sui_navigation_menu_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_navigation_menu_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets42(__root);
    const __e0 = document.createElement("slot");
    attachRef24($scope, "defaultSlotEl", __e0);
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

// packages/components/primitives/src/sui-navigation-menu/sui-navigation-item.sui
import { bindAttr as bindAttr20, bindIf as bindIf16, view as view3 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-navigation-menu/sui-navigation-item.controller.ts
import { controller as controller33 } from "@ship/sui-runtime";
var sui_navigation_item_controller_default = controller33({
  mount(state, host) {
    const scope = state;
    const onClick = (e) => {
      if (scope.disabled())
        return;
      if (!scope.href())
        e.preventDefault();
      host.dispatchEvent(new CustomEvent("sui-nav-select", {
        detail: { value: scope.value() },
        bubbles: true,
        composed: true
      }));
    };
    host.focusLink = () => {
      const el = host.shadowRoot?.querySelector(".link");
      el?.focus();
    };
    host.addEventListener("click", onClick);
    return () => host.removeEventListener("click", onClick);
  }
});

// packages/components/primitives/src/sui-navigation-menu/sui-navigation-item.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_navigation_item_default = Object.assign(view3((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  if (sui_navigation_item_controller_default && typeof sui_navigation_item_controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = sui_navigation_item_controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b0 = (anchor) => bindIf16({
    anchor,
    test: () => {
      if ($scope.href())
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag1 = document.createDocumentFragment();
        const __c2 = [];
        const __e3 = document.createElement("a");
        __e3.setAttribute("class", "link");
        __e3.setAttribute("part", "link");
        __c2.push(bindAttr20(__e3, "href", () => $scope.href()));
        __c2.push(bindAttr20(__e3, "aria-current", () => $scope.active() ? "page" : "false"));
        const __e4 = document.createElement("slot");
        __e3.appendChild(__e4);
        __frag1.appendChild(__e3);
        return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
          for (const c of __c2)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag5 = document.createDocumentFragment();
        const __c6 = [];
        const __e7 = document.createElement("button");
        __e7.setAttribute("class", "link");
        __e7.setAttribute("part", "link");
        __e7.setAttribute("type", "button");
        __c6.push(bindAttr20(__e7, "aria-current", () => $scope.active() ? "true" : "false"));
        __c6.push(bindAttr20(__e7, "disabled", () => $scope.disabled()));
        const __e8 = document.createElement("slot");
        __e7.appendChild(__e8);
        __frag5.appendChild(__e7);
        return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
          for (const c of __c6)
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
      const __ifAnchor0 = document.createComment("if");
      container.appendChild(__ifAnchor0);
      __cleanups.push(__wireBoundary_b0(__ifAnchor0));
      const __e9 = document.createElement("div");
      __e9.setAttribute("class", "indicator");
      __e9.setAttribute("part", "indicator");
      __e9.setAttribute("aria-hidden", "true");
      container.appendChild(__e9);
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
      if (__m["b0"]) {
        const __t = __m["b0"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b0(__anchor));
      }
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/components/primitives/src/sui-pagination/sui-pagination.sui
import { adoptSuiSheets as adoptSuiSheets43, attachRef as attachRef25, bindAttr as bindAttr21, bindText as bindText12, component as component43, prop as prop43, signal as signal33 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-pagination/sui-pagination.controller.ts
import { controller as controller34, signal as signal32, ref as ref3, effect as effect31 } from "@ship/sui-runtime";
var sui_pagination_controller_default = controller34({
  scope: () => ({
    totalPages: signal32(1),
    info: signal32(""),
    prevBtn: ref3(null),
    nextBtn: ref3(null)
  }),
  mount(state, host) {
    const scope = state;
    const stopEffect = effect31(() => {
      const p = scope.page();
      const t = scope.total();
      const pp = scope.perPage();
      const tp = t > 0 ? Math.ceil(t / pp) : 1;
      scope.totalPages.set(tp);
      if (t > 0) {
        const start = (p - 1) * pp + 1;
        const end = Math.min(p * pp, t);
        scope.info.set(`Page ${p} of ${tp} (${start}–${end} of ${t})`);
      } else {
        scope.info.set(`Page ${p} of ${tp}`);
      }
    });
    const prevBtn = scope.prevBtn();
    const nextBtn = scope.nextBtn();
    const onPrev = () => {
      const p = scope.page();
      if (p > 1) {
        const np = p - 1;
        scope.page.set(np);
        host.dispatchEvent(new CustomEvent("sui-page", {
          detail: { page: np },
          bubbles: true,
          composed: true
        }));
      }
    };
    const onNext = () => {
      const p = scope.page();
      const tp = scope.totalPages();
      if (p < tp) {
        const np = p + 1;
        scope.page.set(np);
        host.dispatchEvent(new CustomEvent("sui-page", {
          detail: { page: np },
          bubbles: true,
          composed: true
        }));
      }
    };
    prevBtn?.addEventListener("click", onPrev);
    nextBtn?.addEventListener("click", onNext);
    return () => {
      stopEffect();
      prevBtn?.removeEventListener("click", onPrev);
      nextBtn?.removeEventListener("click", onNext);
    };
  }
});

// packages/components/primitives/src/sui-pagination/sui-pagination.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_pagination_default = component43("sui-pagination", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    page: prop43.number({ default: 1, reflect: true }),
    total: prop43.number({ default: 0, reflect: true }),
    "per-page": prop43.number({ default: 25, reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: block; }
  nav.bar {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 0.8125rem;
    font-family: var(--font-body);
  }
  .info {
    color: var(--muted-foreground);
    user-select: none;
    white-space: nowrap;
  }
  .spacer { flex: 1; }
  .pg-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 28px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: 0.8125rem;
    cursor: pointer;
    transition: background-color var(--duration-fast);
  }
  .pg-btn:hover:not(:disabled) {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
  }
  .pg-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .pg-btn:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: var(--ring-offset);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.page = props.page;
    $scope.total = props.total;
    $scope.perPage = props.perPage;
    $scope["nextBtn"] = signal33(null);
    $scope["prevBtn"] = signal33(null);
    let __r = null;
    if (sui_pagination_controller_default && typeof sui_pagination_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_pagination_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets43(__root);
    const __e0 = document.createElement("nav");
    __e0.setAttribute("class", "bar");
    __e0.setAttribute("aria-label", "Pagination");
    const __e1 = document.createElement("button");
    attachRef25($scope, "prevBtn", __e1);
    __e1.setAttribute("class", "pg-btn");
    __e1.setAttribute("part", "prev");
    __cleanups.push(bindAttr21(__e1, "disabled", () => $scope.page() <= 1));
    __e1.setAttribute("aria-label", "Previous page");
    __cleanups.push(bindAttr21(__e1, "aria-disabled", () => $scope.page() <= 1 ? "true" : "false"));
    const __t2 = document.createTextNode("Prev");
    __e1.appendChild(__t2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "info");
    __e3.setAttribute("part", "info");
    const __t4 = document.createTextNode("");
    __e3.appendChild(__t4);
    __cleanups.push(bindText12(__t4, () => $scope.info()));
    __e0.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "spacer");
    __e0.appendChild(__e5);
    const __e6 = document.createElement("button");
    attachRef25($scope, "nextBtn", __e6);
    __e6.setAttribute("class", "pg-btn");
    __e6.setAttribute("part", "next");
    __cleanups.push(bindAttr21(__e6, "disabled", () => $scope.page() >= $scope.totalPages()));
    __e6.setAttribute("aria-label", "Next page");
    __cleanups.push(bindAttr21(__e6, "aria-disabled", () => $scope.page() >= $scope.totalPages() ? "true" : "false"));
    const __t7 = document.createTextNode("Next");
    __e6.appendChild(__t7);
    __e0.appendChild(__e6);
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

// packages/components/primitives/src/sui-popover/sui-popover.sui
import { adoptSuiSheets as adoptSuiSheets44, attachRef as attachRef26, bindIf as bindIf17, component as component44, prop as prop44, registerTransition as registerTransition2, scale, signal as signal34 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-popover/sui-popover.controller.ts
import { controller as controller35, effect as effect32, bindNativeEvent as bindNativeEvent4, slotted } from "@ship/sui-runtime";
var anchorSeq = 0;
var anchorSupported = typeof CSS !== "undefined" && typeof CSS.supports === "function" && CSS.supports("anchor-name", "--x");
var sui_popover_controller_default = controller35({
  mount(state, host) {
    const scope = state;
    const id = ++anchorSeq;
    const anchorName = `--sui-popover-${id}`;
    const panelId = `sui-popover-panel-${id}`;
    host.style.setProperty("--sui-popover-anchor", anchorName);
    const triggerSlot = scope.triggerSlot();
    const panelEl = scope.panelEl();
    if (!panelEl)
      return;
    panelEl.id = panelId;
    slotted(scope.triggerSlot, (els) => {
      for (const el of els) {
        el.setAttribute("popovertarget", panelId);
        el.setAttribute("aria-haspopup", "dialog");
        el.setAttribute("aria-controls", panelId);
        el.setAttribute("aria-expanded", scope.open() ? "true" : "false");
      }
    });
    function onToggle(ev) {
      const newState = ev.newState;
      const nowOpen = newState === "open";
      const hasAttr = host.hasAttribute("open");
      if (nowOpen && !hasAttr)
        host.setAttribute("open", "");
      else if (!nowOpen && hasAttr)
        host.removeAttribute("open");
      const slot = triggerSlot;
      const els = slot?.assignedElements({ flatten: true }) ?? [];
      for (const el of els) {
        el.setAttribute("aria-expanded", nowOpen ? "true" : "false");
      }
      host.emit("sui-toggle", { open: nowOpen });
      if (nowOpen && !anchorSupported)
        positionFallback();
    }
    bindNativeEvent4(panelEl, "toggle", onToggle);
    host.expose({
      show: () => panelEl.showPopover?.(),
      hide: () => panelEl.hidePopover?.(),
      toggle: () => typeof panelEl.togglePopover === "function" ? void panelEl.togglePopover() : panelEl.matches?.(":popover-open") ? panelEl.hidePopover?.() : panelEl.showPopover?.()
    });
    const stopOpenSync = effect32(() => {
      const wantOpen = scope.open();
      const isOpen = panelEl.matches?.(":popover-open") ?? false;
      if (wantOpen && !isOpen)
        panelEl.showPopover?.();
      else if (!wantOpen && isOpen)
        panelEl.hidePopover?.();
    });
    function positionFallback() {
      const slot = triggerSlot;
      const els = slot?.assignedElements({ flatten: true }) ?? [];
      const trigger = els[0];
      if (!trigger || !panelEl)
        return;
      const r = trigger.getBoundingClientRect();
      const placement = scope.placement();
      panelEl.style.top = "";
      panelEl.style.left = "";
      panelEl.style.right = "";
      panelEl.style.bottom = "";
      const gap = 6;
      switch (placement) {
        case "top":
          panelEl.style.bottom = `${window.innerHeight - r.top + gap}px`;
          panelEl.style.left = `${r.left}px`;
          break;
        case "top-end":
          panelEl.style.bottom = `${window.innerHeight - r.top + gap}px`;
          panelEl.style.right = `${window.innerWidth - r.right}px`;
          break;
        case "bottom-end":
          panelEl.style.top = `${r.bottom + gap}px`;
          panelEl.style.right = `${window.innerWidth - r.right}px`;
          break;
        case "left":
          panelEl.style.right = `${window.innerWidth - r.left + gap}px`;
          panelEl.style.top = `${r.top}px`;
          break;
        case "left-end":
          panelEl.style.right = `${window.innerWidth - r.left + gap}px`;
          panelEl.style.bottom = `${window.innerHeight - r.bottom}px`;
          break;
        case "right":
          panelEl.style.left = `${r.right + gap}px`;
          panelEl.style.top = `${r.top}px`;
          break;
        case "right-end":
          panelEl.style.left = `${r.right + gap}px`;
          panelEl.style.bottom = `${window.innerHeight - r.bottom}px`;
          break;
        case "bottom":
        default:
          panelEl.style.top = `${r.bottom + gap}px`;
          panelEl.style.left = `${r.left}px`;
      }
    }
    function onWindowChange() {
      if (!scope.open())
        return;
      positionFallback();
    }
    if (!anchorSupported) {
      bindNativeEvent4(window, "resize", onWindowChange);
      bindNativeEvent4(window, "scroll", onWindowChange, { capture: true, passive: true });
    }
    return () => {
      stopOpenSync();
    };
  }
});

// packages/components/primitives/src/sui-popover/sui-popover.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_popover_default = component44("sui-popover", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    open: prop44.bool({ default: false, reflect: true }),
    placement: prop44.union(["top", "top-end", "bottom", "bottom-end", "left", "left-end", "right", "right-end"], { default: "bottom", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    position: relative;
    display: inline-flex;
  }

  /* Native popover defaults are aggressive (position:fixed, inset:0,
     huge default margins). Reset to a layout-neutral container so our
     surface div carries the chrome and anchor positioning drives the
     placement. */
  .panel {
    position: absolute;
    margin: 0;
    border: 0;
    padding: 0;
    background: transparent;
    overflow: visible;
    inset: auto;
  }
  .panel::backdrop { background: transparent; }

  .panel > .surface {
    min-width: 180px;
    padding: 6px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-md, 0 4px 12px rgba(0,0,0,0.1));
    backdrop-filter: var(--card-backdrop-filter, none);
    -webkit-backdrop-filter: var(--card-backdrop-filter, none);
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
  }
  /* Open/close polish is driven by \`transition:scale\` on the
     #if-mounted .surface (see template). No CSS transition here. */

  /* CSS Anchor Positioning. Controller sets a unique anchor-name on the
     SLOTTED trigger element via \`--sui-popover-anchor\`, so each instance
     anchors to its own trigger without any inline-style juggling. */
  @supports (anchor-name: --x) {
    ::slotted([slot='trigger']) {
      anchor-name: var(--sui-popover-anchor, --sui-popover-default);
    }
    .panel {
      position-anchor: var(--sui-popover-anchor, --sui-popover-default);
      /* When the chosen placement would clip the panel against the
         viewport edge, fall back automatically. \`flip-block\` swaps
         top↔bottom, \`flip-inline\` swaps left↔right, and the combined
         entry covers diagonal cases. The browser picks the first
         fallback that fits; if none do, the original placement wins
         (clipped but visible). Modern Chromium/Safari/Firefox all
         honor this — older browsers ignore the property and degrade
         to the chosen placement (the same behavior as today). */
      position-try-fallbacks: flip-block, flip-inline, flip-block flip-inline;
      position-try-order: most-block-size;
    }
    :host(:not([placement])) .panel,
    :host([placement='bottom']) .panel {
      top: calc(anchor(bottom) + 6px);
      left: anchor(left);
    }
    :host([placement='bottom-end']) .panel {
      top: calc(anchor(bottom) + 6px);
      right: calc(100vw - anchor(right));
    }
    :host([placement='top']) .panel {
      bottom: calc(100vh - anchor(top) + 6px);
      left: anchor(left);
    }
    :host([placement='top-end']) .panel {
      bottom: calc(100vh - anchor(top) + 6px);
      right: calc(100vw - anchor(right));
    }
    :host([placement='right']) .panel {
      left: calc(anchor(right) + 6px);
      top: anchor(top);
    }
    :host([placement='right-end']) .panel {
      left: calc(anchor(right) + 6px);
      bottom: calc(100vh - anchor(bottom));
    }
    :host([placement='left']) .panel {
      right: calc(100vw - anchor(left) + 6px);
      top: anchor(top);
    }
    :host([placement='left-end']) .panel {
      right: calc(100vw - anchor(left) + 6px);
      bottom: calc(100vh - anchor(bottom));
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .panel > .surface { transform: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.placement = props.placement;
    $scope["panelEl"] = signal34(null);
    $scope["triggerSlot"] = signal34(null);
    let __r = null;
    if (sui_popover_controller_default && typeof sui_popover_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_popover_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindIf17({
      anchor,
      test: () => {
        if ($scope.open())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag3 = document.createDocumentFragment();
          const __c4 = [];
          const __e5 = document.createElement("div");
          __e5.setAttribute("class", "surface");
          __e5.setAttribute("part", "surface");
          registerTransition2(__e5, scale, { duration: 120 }, "both");
          const __e6 = document.createElement("slot");
          __e5.appendChild(__e6);
          __frag3.appendChild(__e5);
          return { nodes: Array.from(__frag3.childNodes), cleanup: () => {
            for (const c of __c4)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b3 = (anchor) => bindIf17({
      anchor,
      test: () => {
        if ($scope.open())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag8 = document.createDocumentFragment();
          const __c9 = [];
          const __e10 = document.createElement("sui-portal");
          __e10.setAttribute("target", "body");
          const __e11 = document.createElement("div");
          __e11.setAttribute("class", "sui-popover-presence");
          __e11.setAttribute("aria-hidden", "true");
          __e11.setAttribute("data-sui-popover-open", "");
          __e10.appendChild(__e11);
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
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets44(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "trigger");
    attachRef26($scope, "triggerSlot", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "panel");
    __e1.setAttribute("part", "panel");
    __e1.setAttribute("popover", "auto");
    attachRef26($scope, "panelEl", __e1);
    __e1.setAttribute("role", "dialog");
    __e1.setAttribute("aria-modal", "false");
    const __ifAnchor2 = document.createComment("if");
    __e1.appendChild(__ifAnchor2);
    __cleanups.push(__wireBoundary_b2(__ifAnchor2));
    __root.appendChild(__e1);
    const __ifAnchor7 = document.createComment("if");
    __root.appendChild(__ifAnchor7);
    __cleanups.push(__wireBoundary_b3(__ifAnchor7));
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

// packages/components/primitives/src/sui-portal/sui-portal.sui
import { adoptSuiSheets as adoptSuiSheets45, attachRef as attachRef27, component as component45, prop as prop45, signal as signal35 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-portal/sui-portal.controller.ts
import { controller as controller36, effect as effect33 } from "@ship/sui-runtime";
var sui_portal_controller_default = controller36({
  mount(state, host) {
    const scope = state;
    const movedNodes = [];
    let currentTarget = null;
    const resolveTarget = () => {
      const selector = scope.target() || "body";
      const el = document.querySelector(selector);
      if (!el) {
        console.warn(`[sui-portal] target "${selector}" not found in document`);
        return null;
      }
      return el;
    };
    const teleportFromHost = (target) => {
      const slot2 = scope.portalSlot();
      const sources = slot2 ? slot2.assignedNodes() : Array.from(host.childNodes);
      for (const node of sources) {
        target.appendChild(node);
        if (!movedNodes.includes(node))
          movedNodes.push(node);
      }
    };
    const retargetAll = (next) => {
      for (const node of movedNodes)
        next.appendChild(node);
    };
    const removeAll = () => {
      for (const node of movedNodes) {
        if (node.parentNode)
          node.parentNode.removeChild(node);
      }
      movedNodes.length = 0;
    };
    const stopTargetEffect = effect33(() => {
      const next = resolveTarget();
      if (!next) {
        currentTarget = null;
        return;
      }
      if (currentTarget === null) {
        teleportFromHost(next);
      } else if (currentTarget !== next) {
        retargetAll(next);
      }
      currentTarget = next;
    });
    const slot = scope.portalSlot();
    const onSlotChange = () => {
      if (!currentTarget || !slot)
        return;
      for (const node of slot.assignedNodes()) {
        if (!movedNodes.includes(node)) {
          currentTarget.appendChild(node);
          movedNodes.push(node);
        }
      }
    };
    if (slot)
      slot.addEventListener("slotchange", onSlotChange);
    return () => {
      stopTargetEffect();
      if (slot)
        slot.removeEventListener("slotchange", onSlotChange);
      removeAll();
    };
  }
});

// packages/components/primitives/src/sui-portal/sui-portal.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_portal_default = component45("sui-portal", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    target: prop45.string({ default: "body" })
  },
  shadow: true,
  styles: `
  :host { display: none; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.target = props.target;
    $scope["portalSlot"] = signal35(null);
    let __r = null;
    if (sui_portal_controller_default && typeof sui_portal_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_portal_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets45(__root);
    const __e0 = document.createElement("slot");
    attachRef27($scope, "portalSlot", __e0);
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

// packages/components/primitives/src/sui-progress/sui-progress.sui
import { adoptSuiSheets as adoptSuiSheets46, bindAttr as bindAttr22, component as component46, prop as prop46 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_progress_default = component46("sui-progress", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    value: prop46.number({ default: 0 }),
    max: prop46.number({ default: 100 }),
    tone: prop46.union(["default", "primary", "info", "success", "warning", "destructive", "accent"], { default: "primary", reflect: true }),
    size: prop46.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    indeterminate: prop46.bool({ default: false, reflect: true }),
    label: prop46.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --tone: var(--primary);
    --h: 6px;
  }

  :host([size='sm']) { --h: 4px; }
  :host([size='md']) { --h: 6px; }
  :host([size='lg']) { --h: 10px; }

  .track {
    width: 100%;
    height: var(--h);
    background: var(--muted);
    border-radius: var(--radius-full);
    overflow: hidden;
    position: relative;
  }

  .fill {
    height: 100%;
    background: var(--tone);
    border-radius: var(--radius-full);
    transition: width var(--duration-fast) var(--ease-out);
  }

  :host([indeterminate]) .fill {
    width: 30%;
    animation: sui-progress-indet 1.4s ease-in-out infinite;
  }
  @keyframes sui-progress-indet {
    0%   { transform: translateX(-100%); }
    50%  { transform: translateX(150%); }
    100% { transform: translateX(350%); }
  }
  @media (prefers-reduced-motion: reduce) {
    :host([indeterminate]) .fill { animation-duration: 3s; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.max = props.max;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.indeterminate = props.indeterminate;
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets46(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "track");
    __e0.setAttribute("part", "track");
    __e0.setAttribute("role", "progressbar");
    __cleanups.push(bindAttr22(__e0, "aria-valuenow", () => $scope.indeterminate() ? null : $scope.value()));
    __e0.setAttribute("aria-valuemin", "0");
    __cleanups.push(bindAttr22(__e0, "aria-valuemax", () => $scope.max()));
    __cleanups.push(bindAttr22(__e0, "aria-busy", () => $scope.indeterminate() ? "true" : "false"));
    __cleanups.push(bindAttr22(__e0, "aria-label", () => $scope.label()));
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "fill");
    __e1.setAttribute("part", "fill");
    __cleanups.push(bindAttr22(__e1, "style", () => $scope.indeterminate() ? "" : "width: clamp(0%, " + $scope.value() / ($scope.max() || 1) * 100 + "%, 100%)"));
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-progress/sui-progress-ring.sui
import { bindAttr as bindAttr23, view as view4 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_progress_ring_default = Object.assign(view4((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("div");
      __e0.setAttribute("class", "ring");
      __cleanups.push(bindAttr23(__e0, "style", () => "width:" + $scope.size() + "px;height:" + $scope.size() + "px"));
      const __e1 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
      __e1.setAttribute("class", "ring-svg");
      __cleanups.push(bindAttr23(__e1, "viewBox", () => "0 0 " + $scope.size() + " " + $scope.size()));
      __e1.setAttribute("role", "progressbar");
      __e1.setAttribute("aria-valuemin", "0");
      __cleanups.push(bindAttr23(__e1, "aria-valuemax", () => $scope.max()));
      __cleanups.push(bindAttr23(__e1, "aria-valuenow", () => $scope.indeterminate() ? null : $scope.value() < 0 ? 0 : $scope.value() > $scope.max() ? $scope.max() : $scope.value()));
      __cleanups.push(bindAttr23(__e1, "aria-label", () => $scope.label() || null));
      __e1.setAttribute("xmlns", "http://www.w3.org/2000/svg");
      const __e2 = document.createElementNS("http://www.w3.org/2000/svg", "circle");
      __e2.setAttribute("class", "track");
      __e2.setAttribute("part", "track");
      __cleanups.push(bindAttr23(__e2, "cx", () => $scope.size() / 2));
      __cleanups.push(bindAttr23(__e2, "cy", () => $scope.size() / 2));
      __cleanups.push(bindAttr23(__e2, "r", () => ($scope.size() - $scope.trackWidth()) / 2));
      __e2.setAttribute("fill", "none");
      __cleanups.push(bindAttr23(__e2, "stroke-width", () => $scope.trackWidth()));
      __e1.appendChild(__e2);
      const __e3 = document.createElementNS("http://www.w3.org/2000/svg", "circle");
      __e3.setAttribute("class", "bar");
      __e3.setAttribute("part", "ring");
      __cleanups.push(bindAttr23(__e3, "cx", () => $scope.size() / 2));
      __cleanups.push(bindAttr23(__e3, "cy", () => $scope.size() / 2));
      __cleanups.push(bindAttr23(__e3, "r", () => ($scope.size() - $scope.trackWidth()) / 2));
      __e3.setAttribute("fill", "none");
      __cleanups.push(bindAttr23(__e3, "stroke-width", () => $scope.trackWidth()));
      __e3.setAttribute("stroke-linecap", "round");
      __cleanups.push(bindAttr23(__e3, "stroke-dasharray", () => 3.141592653589793 * ($scope.size() - $scope.trackWidth())));
      __cleanups.push(bindAttr23(__e3, "stroke-dashoffset", () => $scope.indeterminate() ? 3.141592653589793 * ($scope.size() - $scope.trackWidth()) * 0.25 : 3.141592653589793 * ($scope.size() - $scope.trackWidth()) * (1 - ($scope.value() < 0 ? 0 : $scope.value() > ($scope.max() || 1) ? 1 : $scope.value() / ($scope.max() || 1)))));
      __e1.appendChild(__e3);
      __e0.appendChild(__e1);
      const __e4 = document.createElement("div");
      __e4.setAttribute("class", "label");
      __e4.setAttribute("aria-hidden", "true");
      const __e5 = document.createElement("slot");
      __e4.appendChild(__e5);
      __e0.appendChild(__e4);
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
        __cleanups.push(hydrateAttr(__m["b0"], "style", () => "width:" + $scope.size() + "px;height:" + $scope.size() + "px"));
      if (__m["b1"])
        __cleanups.push(hydrateAttr(__m["b1"], "viewbox", () => "0 0 " + $scope.size() + " " + $scope.size()));
      if (__m["b2"])
        __cleanups.push(hydrateAttr(__m["b2"], "aria-valuemax", () => $scope.max()));
      if (__m["b3"])
        __cleanups.push(hydrateAttr(__m["b3"], "aria-valuenow", () => $scope.indeterminate() ? null : $scope.value() < 0 ? 0 : $scope.value() > $scope.max() ? $scope.max() : $scope.value()));
      if (__m["b4"])
        __cleanups.push(hydrateAttr(__m["b4"], "aria-label", () => $scope.label() || null));
      if (__m["b5"])
        __cleanups.push(hydrateAttr(__m["b5"], "cx", () => $scope.size() / 2));
      if (__m["b6"])
        __cleanups.push(hydrateAttr(__m["b6"], "cy", () => $scope.size() / 2));
      if (__m["b7"])
        __cleanups.push(hydrateAttr(__m["b7"], "r", () => ($scope.size() - $scope.trackWidth()) / 2));
      if (__m["b8"])
        __cleanups.push(hydrateAttr(__m["b8"], "stroke-width", () => $scope.trackWidth()));
      if (__m["b9"])
        __cleanups.push(hydrateAttr(__m["b9"], "cx", () => $scope.size() / 2));
      if (__m["b10"])
        __cleanups.push(hydrateAttr(__m["b10"], "cy", () => $scope.size() / 2));
      if (__m["b11"])
        __cleanups.push(hydrateAttr(__m["b11"], "r", () => ($scope.size() - $scope.trackWidth()) / 2));
      if (__m["b12"])
        __cleanups.push(hydrateAttr(__m["b12"], "stroke-width", () => $scope.trackWidth()));
      if (__m["b13"])
        __cleanups.push(hydrateAttr(__m["b13"], "stroke-dasharray", () => 3.141592653589793 * ($scope.size() - $scope.trackWidth())));
      if (__m["b14"])
        __cleanups.push(hydrateAttr(__m["b14"], "stroke-dashoffset", () => $scope.indeterminate() ? 3.141592653589793 * ($scope.size() - $scope.trackWidth()) * 0.25 : 3.141592653589793 * ($scope.size() - $scope.trackWidth()) * (1 - ($scope.value() < 0 ? 0 : $scope.value() > ($scope.max() || 1) ? 1 : $scope.value() / ($scope.max() || 1)))));
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/components/primitives/src/sui-radio-group/sui-radio.sui
import { attachRef as attachRef28, bindAttr as bindAttr24, bindChecked as bindChecked2, signal as signal36, view as view5 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-radio-group/sui-radio.controller.ts
import { controller as controller37, effect as effect34 } from "@ship/sui-runtime";
var sui_radio_controller_default = controller37({
  mount(state, host) {
    const scope = state;
    const stop = effect34(() => {
      const input = scope.inputEl();
      if (!input)
        return;
      const onClick = () => {
        if (scope.disabled())
          return;
        host.dispatchEvent(new CustomEvent("sui-radio-select", {
          detail: { value: scope.value() },
          bubbles: true,
          composed: true
        }));
      };
      const onKeydown = (e) => {
        const navKeys = ["ArrowDown", "ArrowRight", "ArrowUp", "ArrowLeft"];
        if (!navKeys.includes(e.key))
          return;
        e.preventDefault();
        const forward = e.key === "ArrowDown" || e.key === "ArrowRight";
        host.dispatchEvent(new CustomEvent("sui-radio-nav", {
          detail: { direction: forward ? "next" : "prev" },
          bubbles: true,
          composed: true
        }));
      };
      input.addEventListener("click", onClick);
      input.addEventListener("keydown", onKeydown);
      return () => {
        input.removeEventListener("click", onClick);
        input.removeEventListener("keydown", onKeydown);
      };
    });
    host.focusInput = () => {
      const input = host.shadowRoot?.querySelector("input");
      input?.focus();
    };
    return () => stop();
  }
});

// packages/components/primitives/src/sui-radio-group/sui-radio.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_radio_default = Object.assign(view5((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope["inputEl"] = signal36(null);
  if (sui_radio_controller_default && typeof sui_radio_controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = sui_radio_controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("label");
      __e0.setAttribute("part", "label");
      const __e1 = document.createElement("span");
      __e1.setAttribute("class", "circle");
      __e1.setAttribute("part", "circle");
      const __e2 = document.createElement("input");
      __e2.setAttribute("part", "input");
      __e2.setAttribute("type", "radio");
      __cleanups.push(bindChecked2(__e2, $scope.checked));
      __cleanups.push(bindAttr24(__e2, "disabled", () => $scope.disabled()));
      __cleanups.push(bindAttr24(__e2, "aria-checked", () => $scope.checked() ? "true" : "false"));
      __cleanups.push(bindAttr24(__e2, "tabindex", () => $scope.checked() ? "0" : "-1"));
      attachRef28($scope, "inputEl", __e2);
      __e1.appendChild(__e2);
      const __e3 = document.createElement("span");
      __e3.setAttribute("class", "dot");
      __e3.setAttribute("part", "dot");
      __e3.setAttribute("aria-hidden", "true");
      __e1.appendChild(__e3);
      __e0.appendChild(__e1);
      const __e4 = document.createElement("slot");
      __e0.appendChild(__e4);
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
        __cleanups.push(hydrateBindChecked(__m["b0"], $scope.checked));
      if (__m["b1"])
        __cleanups.push(hydrateAttr(__m["b1"], "disabled", () => $scope.disabled()));
      if (__m["b2"])
        __cleanups.push(hydrateAttr(__m["b2"], "aria-checked", () => $scope.checked() ? "true" : "false"));
      if (__m["b3"])
        __cleanups.push(hydrateAttr(__m["b3"], "tabindex", () => $scope.checked() ? "0" : "-1"));
      if (__m["b4"])
        __cleanups.push(hydrateRef(__m["b4"], $scope, "inputEl"));
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/components/primitives/src/sui-radio-group/sui-radio-group.sui
import { adoptSuiSheets as adoptSuiSheets47, component as component47, prop as prop47 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-radio-group/sui-radio-group.controller.ts
import { controller as controller38, effect as effect35 } from "@ship/sui-runtime";
var sui_radio_group_controller_default = controller38({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "radiogroup");
    const hiddenInput = document.createElement("input");
    hiddenInput.type = "hidden";
    host.appendChild(hiddenInput);
    const getRadios = () => Array.from(host.querySelectorAll(":scope > sui-radio"));
    const enabledRadios = () => getRadios().filter((r) => !r.hasAttribute("disabled") && !scope.disabled());
    const syncRadios = () => {
      const currentValue = scope.value();
      const isGroupDisabled = scope.disabled();
      const groupTone = scope.tone();
      const groupSize = scope.size();
      for (const radio of getRadios()) {
        radio.toggleAttribute("checked", radio.getAttribute("value") === currentValue);
        if (isGroupDisabled)
          radio.setAttribute("disabled", "");
        if (groupTone && !radio.hasAttribute("tone"))
          radio.setAttribute("tone", groupTone);
        if (groupSize && !radio.hasAttribute("size"))
          radio.setAttribute("size", groupSize);
      }
      hiddenInput.name = scope.name();
      hiddenInput.value = currentValue;
    };
    const stopSync = effect35(() => {
      scope.value();
      scope.disabled();
      scope.tone();
      scope.size();
      syncRadios();
    });
    const stopLabel = effect35(() => {
      const label = scope.label();
      if (label)
        host.setAttribute("aria-label", label);
      else
        host.removeAttribute("aria-label");
    });
    const stopRequired = effect35(() => {
      const required = scope.required();
      if (required)
        host.setAttribute("aria-required", "true");
      else
        host.removeAttribute("aria-required");
    });
    const onSelect = (e) => {
      const ce = e;
      ce.stopPropagation();
      const newVal = ce.detail.value;
      if (newVal === scope.value())
        return;
      scope.value.set(newVal);
      syncRadios();
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: newVal },
        bubbles: true,
        composed: true
      }));
    };
    const onNav = (e) => {
      const ce = e;
      ce.stopPropagation();
      const enabled = enabledRadios();
      if (!enabled.length)
        return;
      const current = ce.composedPath().find((el) => el.tagName?.toLowerCase() === "sui-radio");
      const idx = current ? enabled.indexOf(current) : -1;
      const next = ce.detail.direction === "next" ? enabled[(idx + 1) % enabled.length] : enabled[(idx - 1 + enabled.length) % enabled.length];
      const nextVal = next.getAttribute("value") ?? "";
      scope.value.set(nextVal);
      syncRadios();
      next.focusInput?.();
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: nextVal },
        bubbles: true,
        composed: true
      }));
    };
    host.addEventListener("sui-radio-select", onSelect);
    host.addEventListener("sui-radio-nav", onNav);
    return () => {
      stopSync();
      stopLabel();
      stopRequired();
      host.removeEventListener("sui-radio-select", onSelect);
      host.removeEventListener("sui-radio-nav", onNav);
      hiddenInput.remove();
    };
  }
});

// packages/components/primitives/src/sui-radio-group/sui-radio-group.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_radio_group_default = component47("sui-radio-group", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    value: prop47.string({ default: "", reflect: true }),
    disabled: prop47.bool({ default: false, reflect: true }),
    tone: prop47.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop47.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    name: prop47.string({ default: "" }),
    label: prop47.string({ default: "" }),
    required: prop47.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    gap: var(--space-2, 8px);
    font-family: var(--font-body);
    color: var(--foreground);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.disabled = props.disabled;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.name = props.name;
    $scope.label = props.label;
    $scope.required = props.required;
    let __r = null;
    if (sui_radio_group_controller_default && typeof sui_radio_group_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_radio_group_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets47(__root);
    const __e0 = document.createElement("slot");
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

// packages/components/primitives/src/sui-scroll-area/sui-scroll-area.sui
import { adoptSuiSheets as adoptSuiSheets48, attachRef as attachRef29, component as component48, prop as prop48, signal as signal37 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-scroll-area/sui-scroll-area.controller.ts
import { controller as controller39 } from "@ship/sui-runtime";
var sui_scroll_area_controller_default = controller39({
  mount(state, host) {
    const scope = state;
    let scrollTimer = null;
    const updateThumbs = () => {
      const vp2 = scope.viewportEl();
      if (!vp2)
        return;
      const orientation = scope.orientation();
      const showV = orientation === "vertical" || orientation === "both";
      const showH = orientation === "horizontal" || orientation === "both";
      const vThumb = scope.vThumb();
      const hThumb = scope.hThumb();
      if (showV && vThumb) {
        const ratio = vp2.clientHeight / vp2.scrollHeight;
        if (ratio < 1) {
          const thumbH = Math.max(24, ratio * vp2.clientHeight);
          const scrollRatio = vp2.scrollTop / (vp2.scrollHeight - vp2.clientHeight);
          const thumbTop = scrollRatio * (vp2.clientHeight - thumbH);
          vThumb.style.height = `${thumbH}px`;
          vThumb.style.top = `${thumbTop}px`;
          vThumb.style.display = "";
        } else {
          vThumb.style.display = "none";
        }
      }
      if (showH && hThumb) {
        const ratio = vp2.clientWidth / vp2.scrollWidth;
        if (ratio < 1) {
          const thumbW = Math.max(24, ratio * vp2.clientWidth);
          const scrollRatio = vp2.scrollLeft / (vp2.scrollWidth - vp2.clientWidth);
          const thumbLeft = scrollRatio * (vp2.clientWidth - thumbW);
          hThumb.style.width = `${thumbW}px`;
          hThumb.style.left = `${thumbLeft}px`;
          hThumb.style.display = "";
        } else {
          hThumb.style.display = "none";
        }
      }
    };
    const onScroll = () => {
      updateThumbs();
      const vBar = scope.vScrollbar();
      const hBar = scope.hScrollbar();
      vBar?.classList.add("active");
      hBar?.classList.add("active");
      if (scrollTimer)
        clearTimeout(scrollTimer);
      scrollTimer = setTimeout(() => {
        vBar?.classList.remove("active");
        hBar?.classList.remove("active");
      }, 1200);
    };
    const vp = scope.viewportEl();
    if (vp) {
      vp.addEventListener("scroll", onScroll, { passive: true });
    }
    updateThumbs();
    return () => {
      if (scrollTimer)
        clearTimeout(scrollTimer);
      if (vp)
        vp.removeEventListener("scroll", onScroll);
    };
  }
});

// packages/components/primitives/src/sui-scroll-area/sui-scroll-area.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_scroll_area_default = component48("sui-scroll-area", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    orientation: prop48.union(["vertical", "horizontal", "both"], { default: "vertical", reflect: true }),
    size: prop48.union(["sm", "md", "lg"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    position: relative;
    overflow: hidden;
  }

  :host(:not([size])), :host([size='md']) { --sb-size: 8px; }
  :host([size='sm']) { --sb-size: 6px; }
  :host([size='lg']) { --sb-size: 10px; }

  .viewport {
    width: 100%;
    height: 100%;
    overflow: hidden;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }
  .viewport::-webkit-scrollbar { display: none; }

  :host(:not([orientation])) .viewport,
  :host([orientation='vertical']) .viewport { overflow-y: auto; }
  :host([orientation='horizontal']) .viewport { overflow-x: auto; }
  :host([orientation='both']) .viewport { overflow: auto; }

  /* Overlay scrollbar tracks — absolute-positioned, opacity-fade on hover/scroll */
  .scrollbar {
    position: absolute;
    opacity: 0;
    transition: opacity var(--duration-medium) var(--ease-out);
    z-index: 1;
  }
  :host(:hover) .scrollbar,
  :host(:focus-within) .scrollbar,
  .scrollbar.active { opacity: 1; }

  .scrollbar-v {
    top: 0; right: 2px; bottom: 0;
    width: var(--sb-size);
  }
  .scrollbar-h {
    left: 0; right: 0; bottom: 2px;
    height: var(--sb-size);
  }

  /* Orientation-driven track visibility */
  :host(:not([orientation])) .scrollbar-h,
  :host([orientation='vertical']) .scrollbar-h { display: none; }
  :host([orientation='horizontal']) .scrollbar-v { display: none; }

  .thumb {
    position: absolute;
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--foreground) 30%, transparent);
    transition: background-color var(--duration-fast) var(--ease-out);
  }
  .thumb:hover {
    background: color-mix(in oklch, var(--foreground) 50%, transparent);
  }
  .scrollbar-v .thumb {
    width: 100%;
    min-height: 24px;
  }
  .scrollbar-h .thumb {
    height: 100%;
    min-width: 24px;
  }

  @media (prefers-reduced-motion: reduce) {
    .scrollbar { transition: none; }
    .thumb { transition: none; }
  }

  @media (forced-colors: active) {
    .thumb { background: CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.orientation = props.orientation;
    $scope.size = props.size;
    $scope["hScrollbar"] = signal37(null);
    $scope["hThumb"] = signal37(null);
    $scope["vScrollbar"] = signal37(null);
    $scope["vThumb"] = signal37(null);
    $scope["viewportEl"] = signal37(null);
    let __r = null;
    if (sui_scroll_area_controller_default && typeof sui_scroll_area_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_scroll_area_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets48(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "viewport");
    __e0.setAttribute("part", "viewport");
    __e0.setAttribute("tabindex", "0");
    attachRef29($scope, "viewportEl", __e0);
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "scrollbar scrollbar-v");
    __e2.setAttribute("part", "scrollbar-v");
    __e2.setAttribute("aria-hidden", "true");
    attachRef29($scope, "vScrollbar", __e2);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "thumb");
    attachRef29($scope, "vThumb", __e3);
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "scrollbar scrollbar-h");
    __e4.setAttribute("part", "scrollbar-h");
    __e4.setAttribute("aria-hidden", "true");
    attachRef29($scope, "hScrollbar", __e4);
    const __e5 = document.createElement("div");
    __e5.setAttribute("class", "thumb");
    attachRef29($scope, "hThumb", __e5);
    __e4.appendChild(__e5);
    __root.appendChild(__e4);
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

// packages/components/primitives/src/sui-segmented/sui-segmented.sui
import { adoptSuiSheets as adoptSuiSheets49, attachRef as attachRef30, bindAttr as bindAttr25, bindFor as bindFor3, bindIf as bindIf18, bindProp as bindProp2, bindText as bindText13, component as component49, prop as prop49, signal as signal39 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-segmented/sui-segmented.controller.ts
import { controller as controller40, signal as signal38, effect as effect36, ref as ref4 } from "@ship/sui-runtime";
var cssEscape = (s) => typeof CSS !== "undefined" && CSS.escape ? CSS.escape(s) : s.replace(/[^\w-]/g, "\\$&");
var sui_segmented_controller_default = controller40({
  scope: () => {
    const options = signal38([]);
    const containerEl = ref4(null);
    return { options, containerEl };
  },
  mount(state, host) {
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "radiogroup");
    const hiddenInput = document.createElement("input");
    hiddenInput.type = "hidden";
    host.appendChild(hiddenInput);
    const readOptions = () => {
      const els = Array.from(host.querySelectorAll(":scope > sui-segmented-option"));
      return els.map((el) => ({
        value: el.getAttribute("value") ?? "",
        label: el.textContent?.trim() ?? "",
        icon: el.getAttribute("icon") || null,
        disabled: el.hasAttribute("disabled")
      }));
    };
    const select = (val) => {
      if (state.disabled())
        return;
      const opts = state.options();
      const opt = opts.find((o) => o.value === val);
      if (!opt || opt.disabled)
        return;
      if (state.value() === val)
        return;
      state.value.set(val);
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: val },
        bubbles: true,
        composed: true
      }));
    };
    const onClick = (e) => {
      const btn = e.target?.closest(".btn");
      if (!btn)
        return;
      const val = btn.getAttribute("data-value");
      if (val !== null)
        select(val);
    };
    const onKeyDown = (e) => {
      const opts = state.options();
      const enabled = opts.filter((o) => !o.disabled && !state.disabled());
      if (!enabled.length)
        return;
      const values = enabled.map((o) => o.value);
      const cur = state.value();
      const i = Math.max(0, values.indexOf(cur));
      let next = i;
      if (e.key === "ArrowRight" || e.key === "ArrowDown") {
        e.preventDefault();
        next = (i + 1) % values.length;
      } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
        e.preventDefault();
        next = (i - 1 + values.length) % values.length;
      } else if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        const focused = host.shadowRoot.activeElement;
        const focusedValue = focused?.getAttribute("data-value");
        if (focusedValue)
          select(focusedValue);
        return;
      } else {
        return;
      }
      const targetValue = values[next] ?? "";
      select(targetValue);
      queueMicrotask(() => {
        const btn = host.shadowRoot?.querySelector(`.btn[data-value="${cssEscape(targetValue)}"]`);
        btn?.focus();
      });
    };
    state.options.set(readOptions());
    const observer = new MutationObserver(() => state.options.set(readOptions()));
    observer.observe(host, { childList: true, subtree: false });
    const stopHidden = effect36(() => {
      hiddenInput.name = state.name();
      hiddenInput.value = state.value();
    });
    const stopContainer = effect36(() => {
      const container = state.containerEl();
      if (!container)
        return;
      container.addEventListener("click", onClick);
      container.addEventListener("keydown", onKeyDown);
      return () => {
        container.removeEventListener("click", onClick);
        container.removeEventListener("keydown", onKeyDown);
      };
    });
    return () => {
      observer.disconnect();
      stopHidden();
      stopContainer();
      hiddenInput.remove();
    };
  }
});

// packages/components/primitives/src/sui-segmented/sui-segmented.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_segmented_default = component49("sui-segmented", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    value: prop49.string({ default: "", reflect: true }),
    size: prop49.union(["sm", "md"], { default: "md", reflect: true }),
    disabled: prop49.bool({ default: false, reflect: true }),
    name: prop49.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
  }

  .container {
    display: inline-flex;
    padding: 2px;
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    gap: 1px;
  }

  .btn {
    appearance: none;
    border: 0;
    background: transparent;
    border-radius: var(--radius-sm, 4px);
    font-family: var(--font-body);
    color: var(--muted-foreground);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: var(--space-1, 4px);
    transition:
      background var(--duration-fast, 150ms) var(--ease-out, ease-out),
      color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
    white-space: nowrap;
    line-height: 1;
  }

  :host(:not([size])) .btn,
  :host([size='md']) .btn {
    padding: 5px var(--space-3, 12px);
    font-size: var(--text-sm, 13px);
  }
  :host([size='sm']) .btn {
    padding: 4px var(--space-2, 8px);
    font-size: var(--text-xs, 11px);
  }

  .btn[aria-checked='true'] {
    background: var(--card);
    color: var(--foreground);
    box-shadow: var(--shadow-xs);
  }

  .btn:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 2px var(--background),
      0 0 0 4px color-mix(in oklch, var(--primary) 60%, transparent);
  }

  .btn[disabled],
  :host([disabled]) .btn {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
    width: 14px;
    height: 14px;
  }
  :host(:not([size='sm'])) .btn-icon { width: 16px; height: 16px; }

`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.size = props.size;
    $scope.disabled = props.disabled;
    $scope.name = props.name;
    $scope["containerEl"] = signal39(null);
    let __r = null;
    if (sui_segmented_controller_default && typeof sui_segmented_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_segmented_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindFor3({
      anchor,
      items: () => $scope.options(),
      key: (opt, __i) => {
        const __child = Object.create($scope);
        __child.opt = opt;
        __child.__i = __i;
        return (($scope2) => opt.value)(__child);
      },
      render: (opt, __i) => {
        const __childScope2 = Object.create($scope);
        __childScope2.opt = opt;
        __childScope2.__i = __i;
        return (($scope2) => {
          __childScope2.__v_disabled = signal39(0);
          __childScope2.__v_value = signal39(0);
          const __wireBoundary_b7 = (anchor2) => bindIf18({
            anchor: anchor2,
            test: () => {
              if ($scope2.opt.icon)
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag8 = document.createDocumentFragment();
                const __c9 = [];
                const __e10 = document.createElement("span");
                __e10.setAttribute("class", "btn-icon");
                __e10.setAttribute("aria-hidden", "true");
                const __e11 = document.createElement("sui-icon");
                __e11["name"] = $scope2.opt.icon;
                __e11.setAttribute("size", "sm");
                __e10.appendChild(__e11);
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
          const __frag3 = document.createDocumentFragment();
          const __c4 = [];
          const __e5 = document.createElement("button");
          __e5.setAttribute("class", "btn");
          __e5.setAttribute("role", "radio");
          __e5.setAttribute("type", "button");
          {
            const __av6 = $scope2.opt.value;
            if (__av6 == null || __av6 === false || __av6 === "")
              __e5.removeAttribute("data-value");
            else
              __e5.setAttribute("data-value", __av6 === true ? "" : String(__av6));
          }
          __c4.push(bindAttr25(__e5, "aria-checked", () => (void $scope2.__v_value(), $scope2.opt.value === $scope2.value() ? "true" : "false")));
          __c4.push(bindAttr25(__e5, "aria-disabled", () => (void $scope2.__v_disabled(), $scope2.disabled() || $scope2.opt.disabled ? "true" : "false")));
          __c4.push(bindProp2(__e5, "disabled", () => (void $scope2.__v_disabled(), $scope2.disabled() || $scope2.opt.disabled)));
          __c4.push(bindAttr25(__e5, "tabindex", () => (void $scope2.__v_value(), $scope2.opt.value === $scope2.value() ? "0" : "-1")));
          const __ifAnchor7 = document.createComment("if");
          __e5.appendChild(__ifAnchor7);
          __c4.push(__wireBoundary_b7(__ifAnchor7));
          const __t12 = document.createTextNode("");
          __e5.appendChild(__t12);
          __t12.textContent = $scope2.opt.label;
          __frag3.appendChild(__e5);
          return { nodes: Array.from(__frag3.childNodes), cleanup: () => {
            for (const c of __c4)
              try {
                c();
              } catch (e) {}
          }, update: (__new13, __newI14) => {
            const __old = $scope2.opt;
            $scope2.opt = __new13;
            $scope2.__i = __newI14;
            if (__old.value !== $scope2.opt.value) {
              {
                const __av6 = $scope2.opt.value;
                if (__av6 == null || __av6 === false || __av6 === "")
                  __e5.removeAttribute("data-value");
                else
                  __e5.setAttribute("data-value", __av6 === true ? "" : String(__av6));
              }
            }
            if (__old.label !== $scope2.opt.label) {
              __t12.textContent = $scope2.opt.label;
            }
            if (__old.disabled !== $scope2.opt.disabled) {
              $scope2.__v_disabled.set((c) => c + 1);
            }
            if (__old.value !== $scope2.opt.value) {
              $scope2.__v_value.set((c) => c + 1);
            }
          } };
        })(__childScope2);
      }
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets49(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "container");
    __e0.setAttribute("part", "container");
    attachRef30($scope, "containerEl", __e0);
    const __forAnchor1 = document.createComment("for");
    __e0.appendChild(__forAnchor1);
    __cleanups.push(__wireBoundary_b1(__forAnchor1));
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

// packages/components/primitives/src/sui-segmented/sui-segmented-option.sui
import { view as view6 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_segmented_option_default = Object.assign(view6((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __t0 = document.createTextNode(`

`);
      container.appendChild(__t0);
      const __t1 = document.createTextNode(`
`);
      container.appendChild(__t1);
      const __t2 = document.createTextNode(`
`);
      container.appendChild(__t2);
      const __t3 = document.createTextNode(`

`);
      container.appendChild(__t3);
      const __t4 = document.createTextNode(`
`);
      container.appendChild(__t4);
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/components/primitives/src/sui-select/sui-select.sui
import { adoptSuiSheets as adoptSuiSheets50, attachRef as attachRef31, bindAttr as bindAttr26, component as component50, prop as prop50, signal as signal40 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-select/sui-select.controller.ts
import { controller as controller41, effect as effect37 } from "@ship/sui-runtime";
var sui_select_controller_default = controller41({
  mount(state, host) {
    const scope = state;
    const lightOptions = () => Array.from(host.querySelectorAll(":scope > option, :scope > optgroup > option"));
    const currentLabel = () => {
      const v = scope.value();
      const opts = lightOptions();
      if (opts.length === 0)
        return v;
      const match = opts.find((o) => o.value === v) ?? opts.find((o) => o.hasAttribute("selected")) ?? opts[0];
      const text = (match?.textContent ?? "").trim();
      return text || match?.value || v;
    };
    const updatePlaceholder = () => {
      const sel = scope.selectEl();
      if (!sel) {
        const opts = lightOptions();
        const match = opts.find((o) => o.value === scope.value()) ?? opts.find((o) => o.hasAttribute("selected")) ?? opts[0];
        if (!match || match.value === "")
          host.setAttribute("data-placeholder", "");
        else
          host.removeAttribute("data-placeholder");
        return;
      }
      const opt = sel.selectedOptions?.[0] ?? sel.options[sel.selectedIndex];
      if (!opt || opt.value === "")
        host.setAttribute("data-placeholder", "");
      else
        host.removeAttribute("data-placeholder");
    };
    const syncOptions = () => {
      const sel = scope.selectEl();
      if (!sel)
        return;
      const sourceOpts = host.querySelectorAll(":scope > option, :scope > optgroup");
      sel.replaceChildren(...Array.from(sourceOpts, (o) => o.cloneNode(true)));
      sel.value = scope.value();
    };
    const syncLabel = () => {
      const labelEl = scope.valueLabel();
      if (labelEl)
        labelEl.textContent = currentLabel();
    };
    const stopValueSync = effect37(() => {
      const v = scope.value();
      const sel = scope.selectEl();
      if (sel)
        sel.value = v;
      syncLabel();
      updatePlaceholder();
    });
    const onNativeChange = (e) => {
      const v = e.target.value;
      scope.value.set(v);
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: v },
        bubbles: true,
        composed: true
      }));
    };
    const selectEl = scope.selectEl();
    if (selectEl)
      selectEl.addEventListener("change", onNativeChange);
    const optionsSlot = scope.optionsSlot();
    const onSlotChange = () => {
      syncOptions();
      syncLabel();
      updatePlaceholder();
    };
    if (optionsSlot)
      optionsSlot.addEventListener("slotchange", onSlotChange);
    syncOptions();
    syncLabel();
    updatePlaceholder();
    return () => {
      if (selectEl)
        selectEl.removeEventListener("change", onNativeChange);
      if (optionsSlot)
        optionsSlot.removeEventListener("slotchange", onSlotChange);
      stopValueSync();
    };
  }
});

// packages/components/primitives/src/sui-select/sui-select.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_select_default = component50("sui-select", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    value: prop50.string({ default: "" }),
    name: prop50.string({ default: "" }),
    required: prop50.bool({ default: false, reflect: true }),
    disabled: prop50.bool({ default: false, reflect: true }),
    invalid: prop50.bool({ default: false, reflect: true }),
    tone: prop50.union(["default", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop50.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    label: prop50.string({ default: "" }),
    "described-by": prop50.string({ default: "" })
  },
  shadow: true,
  formAssociated: true,
  formValue: "value",
  formDisabled: "disabled",
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone: var(--foreground);
    --tone-fg: var(--foreground);
  }

  :host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info); }
  :host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) { --sel-h: 34px; --sel-px: 12px; --sel-fs: 13px; --sel-gap: 8px; --sel-chev: 14px; }
  :host([size='sm']) { --sel-h: 28px; --sel-px: 10px; --sel-fs: 12px; --sel-gap: 6px; --sel-chev: 12px; }
  :host([size='lg']) { --sel-h: 40px; --sel-px: 16px; --sel-fs: 14px; --sel-gap: 8px; --sel-chev: 16px; }

  .field {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: var(--sel-gap);
    min-height: var(--sel-h);
    padding-inline: var(--sel-px);
    padding-inline-end: calc(var(--sel-px) + var(--sel-chev) + 6px);
    background: var(--background);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-xs);
    font-family: var(--font-body);
    font-size: var(--sel-fs);
    font-weight: var(--font-weight-medium);
    line-height: 1;
    letter-spacing: -0.005em;
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }

  :host(:not([disabled])) .field:hover {
    background: color-mix(in oklch, var(--muted) 70%, var(--background));
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-sm);
  }

  :host([tone]:not([tone='default'])) .field {
    border-color: color-mix(in oklch, var(--tone) 38%, var(--border));
  }
  :host([tone]:not([tone='default']):not([disabled])) .field:hover {
    background: color-mix(in oklch, var(--tone) 8%, var(--background));
    border-color: color-mix(in oklch, var(--tone) 55%, var(--border));
  }

  select {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    padding-inline: var(--sel-px);
    padding-inline-end: calc(var(--sel-px) + var(--sel-chev) + 6px);
    font: inherit;
    color: transparent;
    background: transparent;
    border: 0;
    outline: 0;
    appearance: none;
    -webkit-appearance: none;
    cursor: inherit;
    border-radius: var(--radius-md);
  }
  select option { color: var(--foreground); background: var(--background); }

  /* Keyboard-only focus ring — drawn on the native <select> so it is
   * visible to AT/keyboard users even when the host's :focus-within
   * tint is suppressed. Mirrors the sui-button pattern: 2px outline +
   * 2px offset, tone-tinted. */
  select:focus-visible {
    outline: 2px solid color-mix(in oklch, var(--tone) 70%, var(--ring-color, var(--primary)));
    outline-offset: 2px;
  }
  :host([invalid]) select:focus-visible {
    outline-color: var(--destructive);
  }

  .value {
    display: inline-block;
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
  }
  :host([data-placeholder]) .value { color: var(--muted-foreground); }

  .chevron {
    position: absolute;
    inset-block: 0;
    inset-inline-end: var(--sel-px);
    margin-block: auto;
    width: var(--sel-chev);
    height: var(--sel-chev);
    color: var(--muted-foreground);
    pointer-events: none;
    transition:
      color var(--duration-fast) var(--ease-out),
      transform var(--duration-medium) var(--ease-out);
  }
  :host(:not([disabled])) .field:hover .chevron { color: var(--foreground); }
  :host([tone]:not([tone='default'])) .chevron {
    color: color-mix(in oklch, var(--tone-fg) 75%, var(--muted-foreground));
  }

  :host(:focus-within) .field {
    outline: none;
    border-color: color-mix(in oklch, var(--tone) 60%, var(--border));
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--tone) 55%, transparent);
  }
  :host(:not([tone]):focus-within) .field,
  :host([tone='default']:focus-within) .field {
    border-color: color-mix(in oklch, var(--ring-color) 55%, var(--border));
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }

  :host([invalid]) .field { border-color: var(--destructive); }
  :host([invalid]:focus-within) .field {
    border-color: var(--destructive);
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
  }

  :host([disabled]) { opacity: 0.6; cursor: not-allowed; }
  :host([disabled]) .field {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 80%, transparent);
    box-shadow: none;
    cursor: not-allowed;
    filter: saturate(0.6);
  }
  :host([disabled]) select { cursor: not-allowed; }

  @media (forced-colors: active) {
    :host(:focus-within) .field { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
  @media (prefers-reduced-motion: reduce) {
    .field, .chevron { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.name = props.name;
    $scope.required = props.required;
    $scope.disabled = props.disabled;
    $scope.invalid = props.invalid;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.label = props.label;
    $scope.describedBy = props.describedBy;
    $scope["optionsSlot"] = signal40(null);
    $scope["selectEl"] = signal40(null);
    $scope["valueLabel"] = signal40(null);
    let __r = null;
    if (sui_select_controller_default && typeof sui_select_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_select_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets50(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "field");
    __e0.setAttribute("part", "field");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "value");
    __e1.setAttribute("aria-hidden", "true");
    attachRef31($scope, "valueLabel", __e1);
    __e0.appendChild(__e1);
    const __e2 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    __e2.setAttribute("class", "chevron");
    __e2.setAttribute("part", "chevron");
    __e2.setAttribute("viewBox", "0 0 16 16");
    __e2.setAttribute("aria-hidden", "true");
    __e2.setAttribute("focusable", "false");
    __e2.setAttribute("fill", "none");
    __e2.setAttribute("stroke", "currentColor");
    __e2.setAttribute("stroke-width", "1.75");
    __e2.setAttribute("stroke-linecap", "round");
    __e2.setAttribute("stroke-linejoin", "round");
    const __e3 = document.createElementNS("http://www.w3.org/2000/svg", "path");
    __e3.setAttribute("d", "M4 6l4 4 4-4");
    __e2.appendChild(__e3);
    __e0.appendChild(__e2);
    const __e4 = document.createElement("select");
    __e4.setAttribute("part", "select");
    __cleanups.push(bindAttr26(__e4, "name", () => $scope.name()));
    __cleanups.push(bindAttr26(__e4, "disabled", () => $scope.disabled()));
    __cleanups.push(bindAttr26(__e4, "required", () => $scope.required()));
    __cleanups.push(bindAttr26(__e4, "aria-invalid", () => $scope.invalid() ? "true" : "false"));
    __cleanups.push(bindAttr26(__e4, "aria-label", () => $scope.label()));
    __cleanups.push(bindAttr26(__e4, "aria-describedby", () => $scope.describedBy()));
    attachRef31($scope, "selectEl", __e4);
    __e0.appendChild(__e4);
    __root.appendChild(__e0);
    const __e5 = document.createElement("span");
    __e5.setAttribute("hidden", "");
    const __e6 = document.createElement("slot");
    attachRef31($scope, "optionsSlot", __e6);
    __e5.appendChild(__e6);
    __root.appendChild(__e5);
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

// packages/components/primitives/src/sui-separator/sui-separator.sui
import { adoptSuiSheets as adoptSuiSheets51, bindAttr as bindAttr27, component as component51, prop as prop51 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_separator_default = component51("sui-separator", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    orientation: prop51.union(["horizontal", "vertical"], { default: "horizontal", reflect: true }),
    variant: prop51.union(["solid", "dashed"], { default: "solid", reflect: true }),
    decorative: prop51.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
  }

  .sep {
    border: 0;
    margin: 0;
    padding: 0;
    flex: 0 0 auto;
  }

  /* Horizontal (default) */
  :host(:not([orientation='vertical'])) .sep {
    display: block;
    width: 100%;
    height: 1px;
    margin: 8px 0;
    background: var(--border);
  }
  :host(:not([orientation='vertical'])[variant='dashed']) .sep {
    background: transparent;
    border-top: 1px dashed var(--border);
    height: 0;
  }

  /* Vertical */
  :host([orientation='vertical']) {
    display: inline-block;
    align-self: stretch;
  }
  :host([orientation='vertical']) .sep {
    display: inline-block;
    width: 1px;
    min-height: 100%;
    margin: 0 8px;
    background: var(--border);
  }
  :host([orientation='vertical'][variant='dashed']) .sep {
    background: transparent;
    border-left: 1px dashed var(--border);
    width: 0;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.orientation = props.orientation;
    $scope.variant = props.variant;
    $scope.decorative = props.decorative;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets51(__root);
    const __e0 = document.createElement("hr");
    __e0.setAttribute("class", "sep");
    __e0.setAttribute("part", "sep");
    __cleanups.push(bindAttr27(__e0, "role", () => $scope.decorative() ? "none" : "separator"));
    __cleanups.push(bindAttr27(__e0, "aria-orientation", () => $scope.decorative() ? null : $scope.orientation()));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-skeleton/sui-skeleton.sui
import { adoptSuiSheets as adoptSuiSheets52, component as component52, prop as prop52 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_skeleton_default = component52("sui-skeleton", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    shape: prop52.union(["text", "line", "block", "circle"], { default: "text", reflect: true }),
    size: prop52.union(["sm", "md", "lg"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: inline-block; vertical-align: middle; }
  :host([shape='block']) { display: block; }

  .sk {
    display: block;
    background: linear-gradient(
      90deg,
      var(--muted) 0%,
      color-mix(in oklch, var(--muted) 75%, var(--background)) 50%,
      var(--muted) 100%
    );
    background-size: 200% 100%;
    animation: sui-skeleton-shimmer 1.5s ease-in-out infinite;
    border-radius: var(--radius-sm);
  }
  :host([shape='circle']) .sk { border-radius: 50%; }

  /* Text / line — narrow heights, full-width default. */
  :host([shape='text']) .sk,
  :host([shape='line']) .sk { width: 100%; }
  :host([size='sm'][shape='text']) .sk,
  :host([size='sm'][shape='line']) .sk { height: 8px; }
  :host([size='md'][shape='text']) .sk,
  :host([size='md'][shape='line']) .sk { height: 12px; }
  :host([size='lg'][shape='text']) .sk,
  :host([size='lg'][shape='line']) .sk { height: 16px; }

  /* Block — fills parent width with a fixed min-height. */
  :host([shape='block']) .sk { width: 100%; min-height: 80px; }

  /* Circle — fixed diameters per size. */
  :host([shape='circle'][size='sm']) .sk { width: 24px; height: 24px; }
  :host([shape='circle'][size='md']) .sk { width: 32px; height: 32px; }
  :host([shape='circle'][size='lg']) .sk { width: 48px; height: 48px; }

  @keyframes sui-skeleton-shimmer {
    0%   { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
  @media (prefers-reduced-motion: reduce) {
    .sk { animation: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.shape = props.shape;
    $scope.size = props.size;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets52(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "sk");
    __e0.setAttribute("part", "sk");
    __e0.setAttribute("role", "status");
    __e0.setAttribute("aria-label", "Loading");
    __e0.setAttribute("aria-live", "polite");
    __e0.setAttribute("aria-busy", "true");
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-slider/sui-slider.sui
import { adoptSuiSheets as adoptSuiSheets53, attachRef as attachRef32, bindAttr as bindAttr28, component as component53, prop as prop53, signal as signal41 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-slider/sui-slider.controller.ts
import { controller as controller42, effect as effect38 } from "@ship/sui-runtime";
var sui_slider_controller_default = controller42({
  mount(state, host) {
    const scope = state;
    const hiddenInput = document.createElement("input");
    hiddenInput.type = "hidden";
    host.appendChild(hiddenInput);
    const clamp = (v) => {
      const step = scope.step();
      const min = scope.min();
      const max = scope.max();
      const stepped = Math.round((v - min) / step) * step + min;
      return Math.max(min, Math.min(max, stepped));
    };
    const setValue = (v) => {
      const clamped = clamp(v);
      if (clamped === scope.value())
        return;
      scope.value.set(clamped);
      hiddenInput.name = scope.name();
      hiddenInput.value = String(clamped);
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: clamped },
        bubbles: true,
        composed: true
      }));
    };
    const valueFromPointer = (clientX) => {
      const track = scope.trackEl();
      if (!track)
        return scope.value();
      const rect = track.getBoundingClientRect();
      const pct = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
      return scope.min() + pct * (scope.max() - scope.min());
    };
    const stopPositionSync = effect38(() => {
      const val = scope.value();
      const min = scope.min();
      const max = scope.max();
      const range = max - min || 1;
      const pct = (val - min) / range * 100;
      const fill = scope.fillEl();
      const thumb = scope.thumbEl();
      if (fill)
        fill.style.width = `${pct}%`;
      if (thumb)
        thumb.style.left = `${pct}%`;
      hiddenInput.name = scope.name();
      hiddenInput.value = String(val);
    });
    let dragging = false;
    const onPointerDown = (e) => {
      if (scope.disabled())
        return;
      e.preventDefault();
      dragging = true;
      setValue(valueFromPointer(e.clientX));
      const thumb = scope.thumbEl();
      thumb?.focus();
      const onMove = (ev) => {
        if (dragging)
          setValue(valueFromPointer(ev.clientX));
      };
      const onUp = () => {
        dragging = false;
        document.removeEventListener("pointermove", onMove);
        document.removeEventListener("pointerup", onUp);
      };
      document.addEventListener("pointermove", onMove);
      document.addEventListener("pointerup", onUp);
    };
    const onKeydown = (e) => {
      if (scope.disabled())
        return;
      const step = scope.step();
      let handled = true;
      switch (e.key) {
        case "ArrowRight":
        case "ArrowUp":
          setValue(scope.value() + step);
          break;
        case "ArrowLeft":
        case "ArrowDown":
          setValue(scope.value() - step);
          break;
        case "Home":
          setValue(scope.min());
          break;
        case "End":
          setValue(scope.max());
          break;
        case "PageUp":
          setValue(scope.value() + step * 10);
          break;
        case "PageDown":
          setValue(scope.value() - step * 10);
          break;
        default:
          handled = false;
      }
      if (handled)
        e.preventDefault();
    };
    const stopPointer = effect38(() => {
      const container = scope.containerEl();
      if (!container)
        return;
      container.addEventListener("pointerdown", onPointerDown);
      return () => container.removeEventListener("pointerdown", onPointerDown);
    });
    const stopKeydown = effect38(() => {
      const thumb = scope.thumbEl();
      if (!thumb)
        return;
      thumb.addEventListener("keydown", onKeydown);
      return () => thumb.removeEventListener("keydown", onKeydown);
    });
    return () => {
      stopPositionSync();
      stopPointer();
      stopKeydown();
      hiddenInput.remove();
    };
  }
});

// packages/components/primitives/src/sui-slider/sui-slider.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_slider_default = component53("sui-slider", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    value: prop53.number({ default: 0, reflect: true }),
    min: prop53.number({ default: 0 }),
    max: prop53.number({ default: 100 }),
    step: prop53.number({ default: 1 }),
    disabled: prop53.bool({ default: false, reflect: true }),
    tone: prop53.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop53.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    name: prop53.string({ default: "" }),
    label: prop53.string({ default: "slider" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    -webkit-tap-highlight-color: transparent;
    --tone: var(--primary);
  }

  :host(:not([tone])),
  :host([tone='default'])     { --tone: var(--primary); }
  :host([tone='primary'])     { --tone: var(--primary); }
  :host([tone='info'])        { --tone: var(--info); }
  :host([tone='success'])     { --tone: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) { --sl-h: 6px; --sl-thumb: 16px; }
  :host([size='sm']) { --sl-h: 4px; --sl-thumb: 12px; }
  :host([size='lg']) { --sl-h: 8px; --sl-thumb: 20px; }

  .container {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
    height: var(--sl-thumb);
    cursor: pointer;
    touch-action: none;
  }
  :host([disabled]) .container { cursor: not-allowed; opacity: 0.5; }

  .track {
    position: relative;
    width: 100%;
    height: var(--sl-h);
    background: var(--muted);
    border-radius: var(--radius-full, 9999px);
    overflow: hidden;
  }

  .fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: var(--tone);
    border-radius: inherit;
  }

  .thumb {
    position: absolute;
    top: 50%;
    width: var(--sl-thumb);
    height: var(--sl-thumb);
    background: var(--background);
    border: 2px solid var(--tone);
    border-radius: 50%;
    box-shadow: var(--shadow-sm);
    transform: translate(-50%, -50%);
    transition:
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 150ms) var(--ease-out, ease-out);
    box-sizing: border-box;
    outline: none;
  }
  .thumb:hover { box-shadow: var(--shadow-md); transform: translate(-50%, -50%) scale(1.05); }
  .thumb:active { transform: translate(-50%, -50%) scale(0.95); }

  :host(:not([disabled])) .thumb:focus-visible {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }

  @media (forced-colors: active) {
    .thumb { border-color: CanvasText; }
    .fill { background: Highlight; }
    .thumb:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.min = props.min;
    $scope.max = props.max;
    $scope.step = props.step;
    $scope.disabled = props.disabled;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.name = props.name;
    $scope.label = props.label;
    $scope["containerEl"] = signal41(null);
    $scope["fillEl"] = signal41(null);
    $scope["thumbEl"] = signal41(null);
    $scope["trackEl"] = signal41(null);
    let __r = null;
    if (sui_slider_controller_default && typeof sui_slider_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_slider_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets53(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "container");
    attachRef32($scope, "containerEl", __e0);
    const __e1 = document.createElement("div");
    __e1.setAttribute("class", "track");
    __e1.setAttribute("part", "track");
    attachRef32($scope, "trackEl", __e1);
    const __e2 = document.createElement("div");
    __e2.setAttribute("class", "fill");
    __e2.setAttribute("part", "fill");
    attachRef32($scope, "fillEl", __e2);
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "thumb");
    __e3.setAttribute("part", "thumb");
    __e3.setAttribute("role", "slider");
    __cleanups.push(bindAttr28(__e3, "tabindex", () => $scope.disabled() ? "-1" : "0"));
    __cleanups.push(bindAttr28(__e3, "aria-valuemin", () => $scope.min()));
    __cleanups.push(bindAttr28(__e3, "aria-valuemax", () => $scope.max()));
    __cleanups.push(bindAttr28(__e3, "aria-valuenow", () => $scope.value()));
    __cleanups.push(bindAttr28(__e3, "aria-label", () => $scope.label() || "slider"));
    __cleanups.push(bindAttr28(__e3, "aria-disabled", () => $scope.disabled() ? "true" : "false"));
    attachRef32($scope, "thumbEl", __e3);
    __e0.appendChild(__e3);
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

// packages/components/primitives/src/sui-spinner/sui-spinner.sui
import { adoptSuiSheets as adoptSuiSheets54, bindAttr as bindAttr29, component as component54, prop as prop54 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_spinner_default = component54("sui-spinner", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    size: prop54.union(["xs", "sm", "md", "lg"], { default: "md", reflect: true }),
    tone: prop54.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    label: prop54.string({ default: "Loading" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone-fg: var(--muted-foreground);
    --sz: 16px;
    --bw: 2px;
  }
  :host([size='xs']) { --sz: 10px; --bw: 1.5px; }
  :host([size='sm']) { --sz: 12px; --bw: 1.5px; }
  :host([size='md']) { --sz: 16px; --bw: 2px; }
  :host([size='lg']) { --sz: 24px; --bw: 2.5px; }

  .ring {
    display: inline-block;
    width: var(--sz);
    height: var(--sz);
    border: var(--bw) solid var(--tone-fg);
    border-right-color: transparent;
    border-radius: 50%;
    animation: sui-spinner-spin 0.7s linear infinite;
  }
  @keyframes sui-spinner-spin { to { transform: rotate(360deg); } }
  @media (prefers-reduced-motion: reduce) { .ring { animation-duration: 1.5s; } }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.size = props.size;
    $scope.tone = props.tone;
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets54(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "ring");
    __e0.setAttribute("part", "ring");
    __cleanups.push(bindAttr29(__e0, "role", () => $scope.label() ? "status" : null));
    __cleanups.push(bindAttr29(__e0, "aria-label", () => $scope.label() || null));
    __cleanups.push(bindAttr29(__e0, "aria-hidden", () => $scope.label() ? null : "true"));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-stack/sui-stack.sui
import { adoptSuiSheets as adoptSuiSheets55, component as component55, prop as prop55 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_stack_default = component55("sui-stack", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    gap: prop55.union(["0", "1", "2", "3", "4", "5", "6", "7", "8"], { default: "4", reflect: true }),
    align: prop55.union(["start", "center", "end", "stretch"], { default: "stretch", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --stack-gap: var(--space-4);
  }

  :host([gap='0']) { --stack-gap: 0; }
  :host([gap='1']) { --stack-gap: var(--space-1); }
  :host([gap='2']) { --stack-gap: var(--space-2); }
  :host([gap='3']) { --stack-gap: var(--space-3); }
  :host([gap='4']) { --stack-gap: var(--space-4); }
  :host([gap='5']) { --stack-gap: var(--space-5); }
  :host([gap='6']) { --stack-gap: var(--space-6); }
  :host([gap='7']) { --stack-gap: var(--space-7); }
  :host([gap='8']) { --stack-gap: var(--space-8); }

  .stack {
    display: flex;
    flex-direction: column;
    gap: var(--stack-gap);
    align-items: stretch;
  }
  :host([align='start'])   .stack { align-items: flex-start; }
  :host([align='center'])  .stack { align-items: center; }
  :host([align='end'])     .stack { align-items: flex-end; }
  :host([align='stretch']) .stack { align-items: stretch; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.gap = props.gap;
    $scope.align = props.align;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets55(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "stack");
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-stat/sui-stat.sui
import { adoptSuiSheets as adoptSuiSheets56, component as component56, prop as prop56 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-stat/sui-stat.controller.ts
import { controller as controller43, effect as effect39 } from "@ship/sui-runtime";
var sui_stat_controller_default = controller43({
  mount(state, host) {
    const scope = state;
    const onKeydown = (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        host.click();
      }
    };
    const stopSync = effect39(() => {
      if (scope.interactive()) {
        if (!host.hasAttribute("role"))
          host.setAttribute("role", "button");
        if (!host.hasAttribute("tabindex"))
          host.setAttribute("tabindex", "0");
        host.addEventListener("keydown", onKeydown);
        return () => host.removeEventListener("keydown", onKeydown);
      } else {
        if (host.getAttribute("role") === "button")
          host.removeAttribute("role");
        if (host.getAttribute("tabindex") === "0")
          host.removeAttribute("tabindex");
      }
    });
    return () => {
      stopSync();
      host.removeEventListener("keydown", onKeydown);
    };
  }
});

// packages/components/primitives/src/sui-stat/sui-stat.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_stat_default = component56("sui-stat", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    tone: prop56.union(["default", "primary", "info", "success", "warning", "destructive", "accent"], { default: "default", reflect: true }),
    interactive: prop56.bool({ default: false, reflect: true }),
    size: prop56.union(["sm", "md", "lg"], { default: "md", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px 18px;
    font-family: var(--font-body);
    transition:
      border-color var(--duration-medium) var(--ease-out),
      transform var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
    --tone-fg: var(--foreground);
  }

  :host([interactive]) { cursor: pointer; }
  :host([interactive]:hover) {
    border-color: color-mix(in oklch, var(--border) 180%, transparent);
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }
  @media (prefers-reduced-motion: reduce) {
    :host([interactive]:hover) { transform: none; }
  }

  .label {
    font-size: var(--text-xs);
    font-weight: var(--font-weight-medium);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .value-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    flex-wrap: wrap;
  }
  .value {
    font-family: var(--heading-font, var(--font-body));
    font-weight: var(--font-weight-bold);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
    color: var(--tone-fg);
  }
  :host([size='sm']) .value { font-size: var(--text-xl, 20px); }
  :host([size='md']) .value { font-size: var(--text-3xl, 30px); }
  :host([size='lg']) .value { font-size: var(--text-4xl, 36px); }

  .trend {
    display: inline-flex;
    align-items: center;
  }
  .hint {
    font-size: 11px;
    color: var(--muted-foreground);
  }
  ::slotted([slot='trend']) { font-size: var(--text-sm); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.interactive = props.interactive;
    $scope.size = props.size;
    let __r = null;
    if (sui_stat_controller_default && typeof sui_stat_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_stat_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets56(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "label");
    __e0.setAttribute("part", "label");
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "label");
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("span");
    __e2.setAttribute("class", "value-row");
    __e2.setAttribute("part", "value-row");
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "value");
    __e3.setAttribute("part", "value");
    const __e4 = document.createElement("slot");
    __e3.appendChild(__e4);
    __e2.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "trend");
    __e5.setAttribute("part", "trend");
    const __e6 = document.createElement("slot");
    __e6.setAttribute("name", "trend");
    __e5.appendChild(__e6);
    __e2.appendChild(__e5);
    __root.appendChild(__e2);
    const __e7 = document.createElement("span");
    __e7.setAttribute("class", "hint");
    __e7.setAttribute("part", "hint");
    const __e8 = document.createElement("slot");
    __e8.setAttribute("name", "hint");
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

// packages/components/primitives/src/sui-step-segments/sui-step-segments.sui
import { adoptSuiSheets as adoptSuiSheets57, bindAttr as bindAttr30, bindFor as bindFor4, component as component57, prop as prop57, signal as signal43 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-step-segments/sui-step-segments.controller.ts
import { controller as controller44, effect as effect40, signal as signal42 } from "@ship/sui-runtime";
var KNOWN = new Set(["done", "failed", "pending", "running"]);
function parseValues(raw) {
  return raw.split(",").map((s) => s.trim().toLowerCase()).filter((s) => s.length > 0).map((s) => KNOWN.has(s) ? s : "pending");
}
var sui_step_segments_controller_default = controller44({
  scope: () => ({
    segments: signal42([]),
    ariaLabel: signal42(null)
  }),
  mount(state, _host) {
    const stop = effect40(() => {
      const states = parseValues(state.values() || "");
      state.segments.set(states.map((s, i) => ({ id: `s${i}`, state: s })));
      state.ariaLabel.set(states.length === 0 ? null : `${states.length} step outcome${states.length === 1 ? "" : "s"}`);
    });
    return () => stop();
  }
});

// packages/components/primitives/src/sui-step-segments/sui-step-segments.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_step_segments_default = component57("sui-step-segments", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    values: prop57.string({ default: "", reflect: true }),
    size: prop57.union(["sm", "md"], { default: "sm", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: inline-flex; vertical-align: middle; }

  .row { display: inline-flex; gap: 3px; align-items: center; }
  .row:empty { display: none; }

  .seg {
    display: inline-block;
    width: 18px;
    height: 6px;
    border-radius: 2px;
    background: color-mix(in oklch, var(--muted-foreground) 28%, transparent);
  }

  :host([size='md']) .seg { width: 24px; height: 8px; border-radius: 3px; }

  .seg[data-state='done']    { background: var(--success); }
  .seg[data-state='failed']  { background: var(--destructive); }
  .seg[data-state='running'] {
    background: var(--warning);
    animation: sui-step-segments-pulse var(--duration-slow, 1200ms) ease-in-out infinite;
  }

  @keyframes sui-step-segments-pulse {
    0%, 100% { opacity: 1; }
    50%      { opacity: 0.55; }
  }

  @media (prefers-reduced-motion: reduce) {
    .seg[data-state='running'] { animation: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.values = props.values;
    $scope.size = props.size;
    let __r = null;
    if (sui_step_segments_controller_default && typeof sui_step_segments_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_step_segments_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindFor4({
      anchor,
      items: () => $scope.segments(),
      key: (seg, __i) => {
        const __child = Object.create($scope);
        __child.seg = seg;
        __child.__i = __i;
        return (($scope2) => seg.id)(__child);
      },
      render: (seg, __i) => {
        const __childScope2 = Object.create($scope);
        __childScope2.seg = seg;
        __childScope2.__i = __i;
        return (($scope2) => {
          const __frag3 = document.createDocumentFragment();
          const __c4 = [];
          const __e5 = document.createElement("span");
          __e5.setAttribute("class", "seg");
          __e5.setAttribute("part", "seg");
          {
            const __av6 = $scope2.seg.state;
            if (__av6 == null || __av6 === false || __av6 === "")
              __e5.removeAttribute("data-state");
            else
              __e5.setAttribute("data-state", __av6 === true ? "" : String(__av6));
          }
          __frag3.appendChild(__e5);
          return { nodes: Array.from(__frag3.childNodes), cleanup: () => {
            for (const c of __c4)
              try {
                c();
              } catch (e) {}
          }, update: (__new7, __newI8) => {
            const __old = $scope2.seg;
            $scope2.seg = __new7;
            $scope2.__i = __newI8;
            if (__old.state !== $scope2.seg.state) {
              {
                const __av6 = $scope2.seg.state;
                if (__av6 == null || __av6 === false || __av6 === "")
                  __e5.removeAttribute("data-state");
                else
                  __e5.setAttribute("data-state", __av6 === true ? "" : String(__av6));
              }
            }
          } };
        })(__childScope2);
      }
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets57(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "row");
    __e0.setAttribute("part", "row");
    __cleanups.push(bindAttr30(__e0, "role", () => $scope.segments().length > 0 ? "img" : null));
    __cleanups.push(bindAttr30(__e0, "aria-label", () => $scope.ariaLabel()));
    const __forAnchor1 = document.createComment("for");
    __e0.appendChild(__forAnchor1);
    __cleanups.push(__wireBoundary_b2(__forAnchor1));
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

// packages/components/primitives/src/sui-stepper/sui-stepper.sui
import { adoptSuiSheets as adoptSuiSheets58, attachRef as attachRef33, component as component58, prop as prop58, signal as signal44 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-stepper/sui-stepper.controller.ts
import { controller as controller45, effect as effect41 } from "@ship/sui-runtime";
var sui_stepper_controller_default = controller45({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "progressbar");
    const stopEffect = effect41(() => {
      const t = Math.max(1, Math.floor(scope.total() || 0));
      const c = Math.max(1, Math.min(t, Math.floor(scope.current() || 1)));
      const label = scope.label();
      host.setAttribute("aria-valuemin", "1");
      host.setAttribute("aria-valuemax", String(t));
      host.setAttribute("aria-valuenow", String(c));
      if (label) {
        host.setAttribute("aria-label", label);
      } else {
        host.setAttribute("aria-label", `Step ${c} of ${t}`);
      }
      const container = scope.dotsEl();
      if (!container)
        return;
      const dots = [];
      for (let i = 1;i <= t; i++) {
        const dot = document.createElement("span");
        dot.className = "dot";
        dot.setAttribute("part", "dot");
        dot.setAttribute("data-state", i === c ? "active" : i < c ? "done" : "pending");
        dot.setAttribute("aria-hidden", "true");
        dots.push(dot);
      }
      container.replaceChildren(...dots);
    });
    return () => {
      stopEffect();
    };
  }
});

// packages/components/primitives/src/sui-stepper/sui-stepper.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_stepper_default = component58("sui-stepper", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    total: prop58.number({ default: 4, reflect: true }),
    current: prop58.number({ default: 1, reflect: true }),
    tone: prop58.string({ default: "", reflect: true }),
    size: prop58.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    label: prop58.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2, 8px);
    --dot-w: 7px;
    --dot-h: 7px;
    --dot-active-w: 22px;
    --tone-fg: var(--primary);
  }

  :host([size='sm']) { --dot-w: 6px; --dot-h: 6px; --dot-active-w: 18px; }
  :host([size='lg']) { --dot-w: 9px; --dot-h: 9px; --dot-active-w: 28px; }

  /* Tone overrides */
  :host([tone='primary'])     { --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone-fg: var(--info); }
  :host([tone='success'])     { --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }

  .dot {
    width: var(--dot-w);
    height: var(--dot-h);
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--muted-foreground) 35%, transparent);
    transition: all var(--duration-medium) var(--ease-out);
    flex-shrink: 0;
  }

  .dot[data-state='active'] {
    width: var(--dot-active-w);
    background: var(--primary);
  }
  :host([tone]) .dot[data-state='active'] {
    background: var(--tone-fg);
  }

  .dot[data-state='done'] {
    background: color-mix(in oklch, var(--primary) 55%, transparent);
  }
  :host([tone]) .dot[data-state='done'] {
    background: color-mix(in oklch, var(--tone-fg) 55%, transparent);
  }

  @media (prefers-reduced-motion: reduce) {
    .dot { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.total = props.total;
    $scope.current = props.current;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.label = props.label;
    $scope["dotsEl"] = signal44(null);
    let __r = null;
    if (sui_stepper_controller_default && typeof sui_stepper_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_stepper_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets58(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "dots");
    attachRef33($scope, "dotsEl", __e0);
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

// packages/components/primitives/src/sui-status-dot/sui-status-dot.sui
import { adoptSuiSheets as adoptSuiSheets59, bindAttr as bindAttr31, component as component59, prop as prop59 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_status_dot_default = component59("sui-status-dot", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    tone: prop59.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "success", reflect: true }),
    pulse: prop59.bool({ default: false, reflect: true }),
    size: prop59.union(["sm", "md"], { default: "md", reflect: true }),
    label: prop59.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone-fg: var(--success);
  }
  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--tone-fg);
    box-shadow: 0 0 0 4px color-mix(in oklch, var(--tone-fg) 20%, transparent);
  }
  :host([size='sm']) .dot {
    width: 6px;
    height: 6px;
    box-shadow: 0 0 0 3px color-mix(in oklch, var(--tone-fg) 20%, transparent);
  }

  :host([pulse]) .dot { animation: sui-status-dot-pulse 2.5s infinite; }
  @keyframes sui-status-dot-pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50%      { opacity: 0.75; transform: scale(0.9); }
  }

  @media (prefers-reduced-motion: reduce) {
    :host([pulse]) .dot { animation: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.pulse = props.pulse;
    $scope.size = props.size;
    $scope.label = props.label;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets59(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "dot");
    __e0.setAttribute("part", "dot");
    __cleanups.push(bindAttr31(__e0, "role", () => $scope.label() ? "img" : null));
    __cleanups.push(bindAttr31(__e0, "aria-label", () => $scope.label() || null));
    __cleanups.push(bindAttr31(__e0, "aria-hidden", () => $scope.label() ? null : "true"));
    __root.appendChild(__e0);
    return $scope;
  }
});

// packages/components/primitives/src/sui-switch/sui-switch.sui
import { adoptSuiSheets as adoptSuiSheets60, attachRef as attachRef34, bindAttr as bindAttr32, bindChecked as bindChecked3, component as component60, prop as prop60, signal as signal45 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-switch/sui-switch.controller.ts
import { controller as controller46, effect as effect42 } from "@ship/sui-runtime";
var sui_switch_controller_default = controller46({
  mount(state, host) {
    const scope = state;
    const stopChange = effect42(() => {
      const el = scope.inputEl();
      if (!el)
        return;
      const onNativeChange = () => {
        host.dispatchEvent(new CustomEvent("sui-change", {
          detail: { checked: el.checked },
          bubbles: true,
          composed: true
        }));
      };
      el.addEventListener("change", onNativeChange);
      return () => el.removeEventListener("change", onNativeChange);
    });
    return () => stopChange();
  }
});

// packages/components/primitives/src/sui-switch/sui-switch.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_switch_default = component60("sui-switch", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    checked: prop60.bool({ default: false, reflect: true }),
    disabled: prop60.bool({ default: false, reflect: true }),
    tone: prop60.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    size: prop60.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    name: prop60.string({ default: "" }),
    value: prop60.string({ default: "on" }),
    label: prop60.string({ default: "" })
  },
  shadow: true,
  formAssociated: true,
  formValue: "checked",
  formDisabled: "disabled",
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    vertical-align: middle;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);
    -webkit-tap-highlight-color: transparent;

    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }
  :host([tone='info'])        { --tone: var(--info);        --tone-on-fg: var(--info-foreground); }
  :host([tone='success'])     { --tone: var(--success);     --tone-on-fg: var(--success-foreground); }
  :host([tone='warning'])     { --tone: var(--warning);     --tone-on-fg: var(--warning-foreground); }
  :host([tone='destructive']) { --tone: var(--destructive); --tone-on-fg: var(--destructive-foreground); }
  :host([tone='accent'])      { --tone: var(--accent);      --tone-on-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }
  :host([tone='primary'])     { --tone: var(--primary);     --tone-on-fg: var(--primary-foreground); }

  :host(:not([size])),
  :host([size='md']) { --sw-w: 28px; --sw-h: 16px; --sw-thumb: 12px; }
  :host([size='sm']) { --sw-w: 24px; --sw-h: 14px; --sw-thumb: 10px; }
  :host([size='lg']) { --sw-w: 32px; --sw-h: 18px; --sw-thumb: 14px; }

  label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
  }
  :host([disabled]) label { cursor: not-allowed; }

  .track {
    position: relative;
    display: inline-block;
    flex-shrink: 0;
    width: var(--sw-w);
    height: var(--sw-h);
    background: var(--muted);
    border: 1px solid color-mix(in oklch, var(--border) 140%, transparent);
    border-radius: var(--radius-full, 9999px);
    box-sizing: border-box;
    transition:
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }

  :host(:not([disabled]):not([checked])) label:hover .track {
    background: color-mix(in oklch, var(--muted) 70%, var(--foreground) 6%);
    border-color: color-mix(in oklch, var(--border) 180%, transparent);
  }

  :host([checked]) .track {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent);
  }
  :host([checked]:not([disabled])) label:hover .track {
    background: color-mix(in oklch, var(--tone) 92%, white 8%);
    box-shadow:
      inset 0 1px 0 0 color-mix(in oklch, white 26%, transparent),
      var(--shadow-xs);
  }

  .thumb {
    position: absolute;
    top: 50%;
    left: 1px;
    width: var(--sw-thumb);
    height: var(--sw-thumb);
    background: var(--background);
    border-radius: 50%;
    box-shadow: var(--shadow-sm);
    transform: translateY(-50%);
    transition:
      left var(--duration-fast, 150ms) var(--ease-out, ease-out),
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
    pointer-events: none;
  }
  :host([checked]) .thumb {
    left: calc(100% - var(--sw-thumb) - 3px);
    background: var(--tone-on-fg);
  }

  /* Native input — visually hidden but preserved for a11y + form submission */
  input {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    padding: 0;
    opacity: 0;
    cursor: inherit;
    appearance: none;
    -webkit-appearance: none;
  }

  :host(:not([disabled])) label:has(input:focus-visible) .track {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }
  :host([checked]:not([disabled])) label:has(input:focus-visible) .track {
    box-shadow:
      inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }

  @media (forced-colors: active) {
    :host(:not([disabled])) label:has(input:focus-visible) .track {
      outline: 2px solid CanvasText;
      outline-offset: 2px;
    }
    .track { border-color: CanvasText; }
    :host([checked]) .track { background: Highlight; }
  }


  :host([disabled]) { color: var(--muted-foreground); }
  :host([disabled]) .track {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 70%, transparent);
    filter: saturate(0.6);
    box-shadow: none;
  }
  :host([disabled][checked]) .track {
    background: color-mix(in oklch, var(--tone) 55%, var(--muted));
    border-color: color-mix(in oklch, var(--tone) 40%, var(--border));
  }
  :host([disabled]) .thumb { box-shadow: var(--shadow-xs); }

  ::slotted(*) {
    font: inherit;
    color: inherit;
    line-height: var(--leading-tight, 1.25);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.checked = props.checked;
    $scope.disabled = props.disabled;
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.name = props.name;
    $scope.value = props.value;
    $scope.label = props.label;
    $scope["inputEl"] = signal45(null);
    let __r = null;
    if (sui_switch_controller_default && typeof sui_switch_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_switch_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets60(__root);
    const __e0 = document.createElement("label");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "track");
    __e1.setAttribute("part", "track");
    const __e2 = document.createElement("input");
    __e2.setAttribute("part", "input");
    __e2.setAttribute("type", "checkbox");
    __e2.setAttribute("role", "switch");
    __cleanups.push(bindChecked3(__e2, $scope.checked));
    __cleanups.push(bindAttr32(__e2, "disabled", () => $scope.disabled()));
    __cleanups.push(bindAttr32(__e2, "name", () => $scope.name()));
    __cleanups.push(bindAttr32(__e2, "value", () => $scope.value()));
    __cleanups.push(bindAttr32(__e2, "aria-checked", () => $scope.checked() ? "true" : "false"));
    __cleanups.push(bindAttr32(__e2, "aria-label", () => $scope.label() || $scope.name() || "toggle"));
    attachRef34($scope, "inputEl", __e2);
    __e1.appendChild(__e2);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "thumb");
    __e3.setAttribute("part", "thumb");
    __e1.appendChild(__e3);
    __e0.appendChild(__e1);
    const __e4 = document.createElement("slot");
    __e0.appendChild(__e4);
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

// packages/components/primitives/src/sui-tab-panel/sui-tab-panel.sui
import { adoptSuiSheets as adoptSuiSheets61, component as component61, prop as prop61 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-tab-panel/sui-tab-panel.controller.ts
import { controller as controller47, effect as effect43 } from "@ship/sui-runtime";
var sui_tab_panel_controller_default = controller47({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "tabpanel");
    if (!host.hasAttribute("tabindex"))
      host.setAttribute("tabindex", "0");
    const stopIdSync = effect43(() => {
      const name = scope.name();
      if (!name)
        return;
      if (!host.id)
        host.id = `sui-tabpanel-${name}`;
      host.setAttribute("aria-labelledby", `sui-tab-${name}`);
    });
    return () => {
      stopIdSync();
    };
  }
});

// packages/components/primitives/src/sui-tab-panel/sui-tab-panel.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_tab_panel_default = component61("sui-tab-panel", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    name: prop61.string({ default: "", reflect: true }),
    active: prop61.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: none; }
  :host([active]) {
    display: block;
    animation: sui-tab-panel-fade var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  @keyframes sui-tab-panel-fade {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  :host(:focus-visible) {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--ring-color, var(--primary)) var(--ring-opacity, 60%), transparent);
    border-radius: var(--radius-md, 6px);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.name = props.name;
    $scope.active = props.active;
    let __r = null;
    if (sui_tab_panel_controller_default && typeof sui_tab_panel_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_tab_panel_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets61(__root);
    const __e0 = document.createElement("slot");
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

// packages/components/primitives/src/sui-table/sui-table.sui
import { adoptSuiSheets as adoptSuiSheets62, bindAttr as bindAttr33, bindClass as bindClass2, bindFor as bindFor5, bindIf as bindIf19, bindText as bindText14, component as component62, on as on5, prop as prop62, signal as signal47 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-table/sui-table.controller.ts
import { controller as controller48, signal as signal46, derived as derived4, bindNativeEvent as bindNativeEvent5 } from "@ship/sui-runtime";
var lightDomInstalled = new WeakSet;
function installLightDomStyles(doc) {
  if (lightDomInstalled.has(doc))
    return;
  lightDomInstalled.add(doc);
  try {
    if (typeof CSSStyleSheet !== "undefined" && "replaceSync" in CSSStyleSheet.prototype) {
      const sheet = new CSSStyleSheet;
      sheet.replaceSync(SUI_TABLE_LIGHT_DOM_CSS);
      doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet];
      return;
    }
  } catch {}
  const style = doc.createElement("style");
  style.setAttribute("data-sui-table", "");
  style.textContent = SUI_TABLE_LIGHT_DOM_CSS;
  doc.head.appendChild(style);
}
var sui_table_controller_default = controller48({
  scope: () => {
    const rows = signal46([]);
    const columns = signal46([]);
    const sortKey = signal46(null);
    const sortDir = signal46("asc");
    const selected = signal46([]);
    const sortedRows = derived4(() => {
      const key = sortKey();
      const dir = sortDir();
      const data = rows();
      if (!key)
        return data;
      return [...data].sort((a, b) => {
        const av = String(a[key] ?? "");
        const bv = String(b[key] ?? "");
        const cmp = av < bv ? -1 : av > bv ? 1 : 0;
        return dir === "asc" ? cmp : -cmp;
      });
    });
    return { rows, columns, sortKey, sortDir, selected, sortedRows };
  },
  on: {
    sortBy(s, field) {
      if (s.sortKey === field) {
        s.sortDir = s.sortDir === "asc" ? "desc" : "asc";
      } else {
        s.sortKey = field;
        s.sortDir = "asc";
      }
    },
    toggleSelect(s, id) {
      const cur = s.selected;
      s.selected = cur.includes(id) ? cur.filter((x) => x !== id) : [...cur, id];
    },
    selectAll(s) {
      s.selected = s.rows.map((r) => r.id);
    },
    clearSelection(s) {
      s.selected = [];
    }
  },
  mount(state, host) {
    const initRows = host.rows;
    if (initRows != null)
      state.rows.set(initRows);
    const initCols = host.columns;
    if (initCols != null)
      state.columns.set(initCols);
    Object.defineProperty(host, "rows", {
      configurable: true,
      get: () => state.rows(),
      set: (v) => state.rows.set(v)
    });
    Object.defineProperty(host, "columns", {
      configurable: true,
      get: () => state.columns(),
      set: (v) => state.columns.set(v)
    });
    installLightDomStyles(host.ownerDocument);
    const paintSortChevrons = () => {
      host.querySelectorAll("th[data-sort-key]").forEach((th) => {
        const dir = th.getAttribute("aria-sort") || "none";
        const ind = th.querySelector(".sui-sort-indicator");
        if (!ind)
          return;
        if (dir === "ascending") {
          ind.style.transform = "rotate(180deg)";
          ind.style.opacity = "1";
        } else if (dir === "descending") {
          ind.style.transform = "rotate(0deg)";
          ind.style.opacity = "1";
        } else {
          ind.style.transform = "rotate(0deg)";
          ind.style.opacity = "0.4";
        }
      });
    };
    const enhanceSortableHeaders = () => {
      host.querySelectorAll("th[data-sort-key]").forEach((th) => {
        if (!th.hasAttribute("aria-sort"))
          th.setAttribute("aria-sort", "none");
        if (!th.hasAttribute("tabindex"))
          th.setAttribute("tabindex", "0");
        if (!th.hasAttribute("role"))
          th.setAttribute("role", "columnheader");
        if (!th.querySelector(".sui-sort-indicator")) {
          const chev = document.createElement("span");
          chev.className = "sui-sort-indicator";
          chev.setAttribute("aria-hidden", "true");
          chev.style.cssText = "margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;";
          const svgNS = "http://www.w3.org/2000/svg";
          const svg = document.createElementNS(svgNS, "svg");
          svg.setAttribute("width", "10");
          svg.setAttribute("height", "10");
          svg.setAttribute("viewBox", "0 0 10 10");
          svg.setAttribute("fill", "none");
          svg.setAttribute("stroke", "currentColor");
          svg.setAttribute("stroke-width", "1.5");
          svg.setAttribute("stroke-linecap", "round");
          svg.setAttribute("stroke-linejoin", "round");
          svg.setAttribute("aria-hidden", "true");
          const path = document.createElementNS(svgNS, "path");
          path.setAttribute("d", "M2 4l3-2 3 2M2 6l3 2 3-2");
          svg.appendChild(path);
          chev.appendChild(svg);
          th.appendChild(chev);
        }
      });
      paintSortChevrons();
    };
    const syncEmptyState = () => {
      const bodyRow = host.querySelector(":scope > table tbody > tr") || host.querySelector(":scope > table > tr:not(:has(th))") || host.querySelector(":scope > tbody > tr") || host.querySelector(':scope > tr:not([slot="head"])');
      const hasEmptySlot = host.querySelector('[slot="empty"]') != null;
      if (!bodyRow && hasEmptySlot)
        host.setAttribute("data-empty", "");
      else
        host.removeAttribute("data-empty");
    };
    const cycleSort = (th) => {
      const key = th.getAttribute("data-sort-key") || "";
      const current = th.getAttribute("aria-sort") || "none";
      const next = current === "none" ? "ascending" : current === "ascending" ? "descending" : "none";
      host.querySelectorAll("th[data-sort-key]").forEach((peer) => {
        if (peer !== th)
          peer.setAttribute("aria-sort", "none");
      });
      th.setAttribute("aria-sort", next);
      paintSortChevrons();
      host.emit("sui-sort", { key, direction: next });
    };
    const onHeaderClick = (e) => {
      const th = e.target.closest("th[data-sort-key]");
      if (!th || !host.contains(th))
        return;
      cycleSort(th);
    };
    const onHeaderKey = (e) => {
      if (e.key !== "Enter" && e.key !== " ")
        return;
      const th = e.target.closest("th[data-sort-key]");
      if (!th || !host.contains(th))
        return;
      e.preventDefault();
      cycleSort(th);
    };
    const onRowClick = (e) => {
      const td = e.target.closest("td");
      if (!td || !host.contains(td))
        return;
      if (td.classList.contains("sui-table-gutter-cell"))
        return;
      const tr = td.closest("tr");
      if (!tr)
        return;
      const rowId = tr.getAttribute("data-row-id");
      if (!rowId)
        return;
      host.emit("sui-row-click", { id: rowId, index: tr.rowIndex });
    };
    bindNativeEvent5(host, "click", onHeaderClick);
    bindNativeEvent5(host, "click", onRowClick);
    bindNativeEvent5(host, "keydown", onHeaderKey);
    enhanceSortableHeaders();
    syncEmptyState();
    const observer = new MutationObserver(() => {
      enhanceSortableHeaders();
      syncEmptyState();
    });
    observer.observe(host, { childList: true, subtree: true });
    return () => {
      observer.disconnect();
    };
  }
});
var SUI_TABLE_LIGHT_DOM_CSS = `
  sui-table {
    --sui-table-border-soft: color-mix(in oklch, var(--border) 80%, transparent);
  }

  sui-table[variant='native'] > table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    table-layout: auto;
    font-variant-numeric: tabular-nums;
  }

  sui-table th,
  sui-table td {
    text-align: left;
    padding: var(--row-py, 10px) var(--row-px, 12px);
    vertical-align: middle;
    border-bottom: 1px solid var(--sui-table-border-soft);
    font-size: var(--text-base, 13px);
    color: var(--foreground);
    box-sizing: border-box;
  }
  sui-table th {
    padding-top: var(--th-py, 10px);
    padding-bottom: var(--th-py, 10px);
    font-weight: var(--font-weight-semibold, 600);
    font-size: var(--th-fs, 10.5px);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
    user-select: none;
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  sui-table thead {
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  sui-table > tr:last-of-type > *,
  sui-table tbody tr:last-child > * {
    border-bottom: 0;
  }

  sui-table th[align='right'], sui-table td[align='right'] { text-align: right; }
  sui-table th[align='center'], sui-table td[align='center'] { text-align: center; }

  sui-table .mono {
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: var(--text-sm, 12px);
    letter-spacing: -0.01em;
  }
  sui-table .muted { color: var(--muted-foreground); }
  sui-table .null { color: var(--muted-foreground); font-style: italic; }

  sui-table > tr:not([slot='head']),
  sui-table tbody tr {
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  sui-table:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  sui-table:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table[striped] > tr:not([slot='head']):nth-of-type(even) > *,
  sui-table[striped] tbody tr:nth-child(even) > * {
    background: color-mix(in oklch, var(--muted) 25%, transparent);
  }
  sui-table[striped]:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  sui-table[striped]:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table tr[data-selected] > *,
  sui-table tr[aria-selected='true'] > * {
    background: var(--tone-primary-bg);
  }
  sui-table tr[data-selected] > *:first-child,
  sui-table tr[aria-selected='true'] > *:first-child {
    box-shadow: inset 3px 0 0 0 var(--primary);
  }

  sui-table[sticky-header] thead th,
  sui-table[sticky-header] tr[slot='head'] > th {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky, 20);
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
    box-shadow: 0 1px 0 0 var(--border);
  }

  sui-table th[data-sort-key],
  sui-table th.sortable {
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    transition: color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  sui-table th[data-sort-key]:hover,
  sui-table th.sortable:hover {
    color: var(--foreground);
  }
  sui-table th[aria-sort='ascending'],
  sui-table th[aria-sort='descending'] {
    color: var(--foreground);
  }
  sui-table th[data-sort-key]:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 var(--ring-width, 3px)
      color-mix(in oklch, var(--ring-color, var(--primary)) var(--ring-opacity, 25%), transparent);
  }

  sui-table[data-empty] > table {
    display: none;
  }

  sui-table[sticky-first-col] th:first-child,
  sui-table[sticky-first-col] td:first-child {
    position: sticky;
    left: 0;
    z-index: 1;
    background: var(--card);
    border-right: 1px solid var(--border);
  }
  sui-table[sticky-first-col] thead th:first-child {
    z-index: 3;
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  sui-table[sticky-first-col] tbody tr:hover td:first-child {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table[gutter] th.sui-table-gutter-cell,
  sui-table[gutter] td.sui-table-gutter-cell {
    width: 40px;
    min-width: 40px;
    max-width: 40px;
    padding: 0;
    text-align: center;
    color: var(--muted-foreground);
    background: color-mix(in oklch, var(--muted-foreground) 5%, var(--background));
    border-right: 1px solid var(--border);
    position: sticky;
    left: 0;
    z-index: 2;
  }
  sui-table[gutter] thead th.sui-table-gutter-cell { z-index: 4; }

  sui-table tr[data-row-id] > td:not(.sui-table-gutter-cell) {
    cursor: pointer;
  }

  @media (forced-colors: active) {
    sui-table th, sui-table td { border-bottom-color: CanvasText; }
  }
`;

// packages/components/primitives/src/sui-table/sui-table.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_table_default = component62("sui-table", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    variant: prop62.union(["slot", "native", "data"], { default: "slot", reflect: true }),
    density: prop62.union(["compact", "default", "comfortable"], { default: "default", reflect: true }),
    dense: prop62.bool({ default: false, reflect: true }),
    hoverable: prop62.bool({ default: true, reflect: true }),
    striped: prop62.bool({ default: false, reflect: true }),
    "sticky-header": prop62.bool({ default: false, reflect: true }),
    "sticky-first-col": prop62.bool({ default: false, reflect: true }),
    gutter: prop62.bool({ default: false, reflect: true }),
    label: prop62.string({ default: "" })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    width: 100%;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);

    --row-py: 10px;
    --row-px: var(--space-3);
    --th-py: 10px;
    --th-fs: var(--text-xs);
    --table-radius: var(--radius-lg);
    --row-hover-mix: 45%;
  }

  :host([density='compact']),
  :host([dense]) {
    --row-py: 6px;
    --row-px: var(--space-2);
    --th-py: 6px;
  }
  :host([density='comfortable']) {
    --row-py: 14px;
    --row-px: var(--space-4);
    --th-py: 14px;
  }

  .wrap {
    position: relative;
    width: 100%;
    border: 1px solid var(--border);
    border-radius: var(--table-radius);
    background: var(--card);
    box-shadow: var(--shadow-xs);
    overflow: hidden;
    overflow-x: auto;
    transition: box-shadow var(--duration-fast) var(--ease-out);
  }

  table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    table-layout: auto;
    font-variant-numeric: tabular-nums;
  }

  :host([sticky-header]) { display: block; }
  :host([sticky-header]) .wrap {
    overflow-y: auto;
    max-height: inherit;
  }

  .empty {
    display: none;
    padding: var(--space-7) var(--space-5);
    text-align: center;
    color: var(--muted-foreground);
    font-size: var(--text-sm);
    background: var(--card);
  }
  :host([data-empty]) .empty { display: block; }
  :host([data-empty]) table { display: none; }

  @media (forced-colors: active) {
    .wrap { border-color: CanvasText; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.variant = props.variant;
    $scope.density = props.density;
    $scope.dense = props.dense;
    $scope.hoverable = props.hoverable;
    $scope.striped = props.striped;
    $scope.stickyHeader = props.stickyHeader;
    $scope.stickyFirstCol = props.stickyFirstCol;
    $scope.gutter = props.gutter;
    $scope.label = props.label;
    let __r = null;
    if (sui_table_controller_default && typeof sui_table_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_table_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindFor5({
      anchor,
      items: () => $scope.columns(),
      key: (c, __i) => {
        const __child = Object.create($scope);
        __child.c = c;
        __child.__i = __i;
        return (($scope2) => c.field)(__child);
      },
      render: (c, __i) => {
        const __childScope8 = Object.create($scope);
        __childScope8.c = c;
        __childScope8.__i = __i;
        return (($scope2) => {
          __childScope8.__v_field = signal47(0);
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __e11 = document.createElement("th");
          __c10.push(on5(__e11, "click", function() {
            const __h = () => $scope2.sortBy($scope2.c.field);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          __c10.push(bindClass2(__e11, "sorted", () => (void $scope2.__v_field(), $scope2.sortKey() === $scope2.c.field)));
          const __t12 = document.createTextNode("");
          __e11.appendChild(__t12);
          __t12.textContent = $scope2.c.label;
          __frag9.appendChild(__e11);
          return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
            for (const c2 of __c10)
              try {
                c2();
              } catch (e) {}
          }, update: (__new13, __newI14) => {
            const __old = $scope2.c;
            $scope2.c = __new13;
            $scope2.__i = __newI14;
            if (__old.label !== $scope2.c.label) {
              __t12.textContent = $scope2.c.label;
            }
            if (__old.field !== $scope2.c.field) {
              $scope2.__v_field.set((c2) => c2 + 1);
            }
          } };
        })(__childScope8);
      }
    });
    const __wireBoundary_b6 = (anchor) => bindFor5({
      anchor,
      items: () => $scope.sortedRows(),
      key: (r, __i) => {
        const __child = Object.create($scope);
        __child.r = r;
        __child.__i = __i;
        return (($scope2) => r.id)(__child);
      },
      render: (r, __i) => {
        const __childScope17 = Object.create($scope);
        __childScope17.r = r;
        __childScope17.__i = __i;
        return (($scope2) => {
          __childScope17.__v_id = signal47(0);
          const __wireBoundary_b9 = (anchor2) => bindFor5({
            anchor: anchor2,
            items: () => $scope2.columns(),
            key: (c, __i2) => {
              const __child = Object.create($scope2);
              __child.c = c;
              __child.__i = __i2;
              return (($scope3) => c.field)(__child);
            },
            render: (c, __i2) => {
              const __childScope22 = Object.create($scope2);
              __childScope22.c = c;
              __childScope22.__i = __i2;
              return (($scope3) => {
                const __frag23 = document.createDocumentFragment();
                const __c24 = [];
                const __e25 = document.createElement("td");
                const __t26 = document.createTextNode("");
                __e25.appendChild(__t26);
                __t26.textContent = $scope3.r[$scope3.c.field];
                __frag23.appendChild(__e25);
                return { nodes: Array.from(__frag23.childNodes), cleanup: () => {
                  for (const c2 of __c24)
                    try {
                      c2();
                    } catch (e) {}
                }, update: (__new27, __newI28) => {
                  const __old = $scope3.c;
                  $scope3.c = __new27;
                  $scope3.__i = __newI28;
                  if (__old.field !== $scope3.c.field) {
                    __t26.textContent = $scope3.r[$scope3.c.field];
                  }
                } };
              })(__childScope22);
            }
          });
          const __frag18 = document.createDocumentFragment();
          const __c19 = [];
          const __e20 = document.createElement("tr");
          __c19.push(bindClass2(__e20, "selected", () => (void $scope2.__v_id(), $scope2.selected().includes($scope2.r.id))));
          __c19.push(on5(__e20, "click", function() {
            const __h = () => $scope2.toggleSelect($scope2.r.id);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __forAnchor21 = document.createComment("for");
          __e20.appendChild(__forAnchor21);
          __c19.push(__wireBoundary_b9(__forAnchor21));
          __frag18.appendChild(__e20);
          return { nodes: Array.from(__frag18.childNodes), cleanup: () => {
            for (const c of __c19)
              try {
                c();
              } catch (e) {}
          }, update: (__new29, __newI30) => {
            const __old = $scope2.r;
            $scope2.r = __new29;
            $scope2.__i = __newI30;
            if (__old.id !== $scope2.r.id) {
              $scope2.__v_id.set((c) => c + 1);
            }
          } };
        })(__childScope17);
      }
    });
    const __wireBoundary_b0 = (anchor) => bindIf19({
      anchor,
      test: () => {
        if ($scope.variant() === "data")
          return 0;
        if ($scope.variant() !== "native")
          return 1;
        return 2;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("div");
          __e3.setAttribute("class", "wrap");
          __e3.setAttribute("part", "wrap");
          const __e4 = document.createElement("table");
          __e4.setAttribute("part", "table");
          __c2.push(bindAttr33(__e4, "aria-label", () => $scope.label() || null));
          const __e5 = document.createElement("thead");
          __e5.setAttribute("part", "thead");
          const __e6 = document.createElement("tr");
          const __forAnchor7 = document.createComment("for");
          __e6.appendChild(__forAnchor7);
          __c2.push(__wireBoundary_b2(__forAnchor7));
          __e5.appendChild(__e6);
          __e4.appendChild(__e5);
          const __e15 = document.createElement("tbody");
          __e15.setAttribute("part", "tbody");
          const __forAnchor16 = document.createComment("for");
          __e15.appendChild(__forAnchor16);
          __c2.push(__wireBoundary_b6(__forAnchor16));
          __e4.appendChild(__e15);
          __e3.appendChild(__e4);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag31 = document.createDocumentFragment();
          const __c32 = [];
          const __e33 = document.createElement("div");
          __e33.setAttribute("class", "wrap");
          __e33.setAttribute("part", "wrap");
          const __e34 = document.createElement("table");
          __e34.setAttribute("part", "table");
          __c32.push(bindAttr33(__e34, "aria-label", () => $scope.label() || null));
          const __e35 = document.createElement("thead");
          __e35.setAttribute("part", "thead");
          const __e36 = document.createElement("slot");
          __e36.setAttribute("name", "head");
          __e35.appendChild(__e36);
          __e34.appendChild(__e35);
          const __e37 = document.createElement("tbody");
          __e37.setAttribute("part", "tbody");
          const __e38 = document.createElement("slot");
          __e37.appendChild(__e38);
          __e34.appendChild(__e37);
          __e33.appendChild(__e34);
          const __e39 = document.createElement("div");
          __e39.setAttribute("class", "empty");
          __e39.setAttribute("part", "empty");
          __e39.setAttribute("role", "status");
          const __e40 = document.createElement("slot");
          __e40.setAttribute("name", "empty");
          __e39.appendChild(__e40);
          __e33.appendChild(__e39);
          __frag31.appendChild(__e33);
          return { nodes: Array.from(__frag31.childNodes), cleanup: () => {
            for (const c of __c32)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag41 = document.createDocumentFragment();
          const __c42 = [];
          const __e43 = document.createElement("div");
          __e43.setAttribute("class", "wrap");
          __e43.setAttribute("part", "wrap");
          const __e44 = document.createElement("slot");
          __e43.appendChild(__e44);
          const __e45 = document.createElement("div");
          __e45.setAttribute("class", "empty");
          __e45.setAttribute("part", "empty");
          __e45.setAttribute("role", "status");
          const __e46 = document.createElement("slot");
          __e46.setAttribute("name", "empty");
          __e45.appendChild(__e46);
          __e43.appendChild(__e45);
          __frag41.appendChild(__e43);
          return { nodes: Array.from(__frag41.childNodes), cleanup: () => {
            for (const c of __c42)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets62(__root);
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

// packages/components/primitives/src/sui-tabs/sui-tabs.sui
import { adoptSuiSheets as adoptSuiSheets63, attachRef as attachRef35, bindAttr as bindAttr34, component as component63, prop as prop63, signal as signal49 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-tabs/sui-tabs.controller.ts
import { controller as controller49, effect as effect44, signal as signal48 } from "@ship/sui-runtime";
var sui_tabs_controller_default = controller49({
  mount(state, host) {
    const scope = state;
    const slotVersion = signal48(0);
    const getTabs = () => {
      const slot = scope.tabSlot();
      if (!slot)
        return Array.from(host.querySelectorAll(":scope > sui-tab"));
      return slot.assignedElements().filter((el) => el.tagName.toLowerCase() === "sui-tab");
    };
    const activate = (name) => {
      if (scope.active() === name)
        return;
      scope.active.set(name);
      host.dispatchEvent(new CustomEvent("sui-tab-change", {
        detail: { name },
        bubbles: true,
        composed: true
      }));
    };
    const stopSync = effect44(() => {
      slotVersion();
      const activeName = scope.active();
      const tabs = getTabs();
      let resolved = activeName;
      if (!resolved && tabs.length > 0) {
        resolved = tabs[0].getAttribute("name") ?? "";
        if (resolved)
          scope.active.set(resolved);
      }
      for (const tab of tabs) {
        const name = tab.getAttribute("name") ?? "";
        if (name === resolved) {
          tab.setAttribute("active", "");
        } else {
          tab.removeAttribute("active");
        }
        tab.id = `sui-tab-${name}`;
        tab.setAttribute("aria-controls", `sui-tabpanel-${name}`);
      }
      for (const panel of Array.from(host.querySelectorAll(':scope > sui-tab-panel, :scope > [role="tabpanel"]'))) {
        const name = panel.getAttribute("name") ?? "";
        if (name === resolved) {
          panel.removeAttribute("hidden");
          panel.setAttribute("active", "");
        } else {
          panel.setAttribute("hidden", "");
          panel.removeAttribute("active");
        }
      }
    });
    const onKeydown = (e) => {
      const vertical = scope.orientation() === "vertical";
      const nextKey = vertical ? "ArrowDown" : "ArrowRight";
      const prevKey = vertical ? "ArrowUp" : "ArrowLeft";
      const navKeys = [nextKey, prevKey, "Home", "End"];
      const activateKeys = ["Enter", " "];
      const isNav = navKeys.includes(e.key);
      const isActivate = activateKeys.includes(e.key);
      if (!isNav && !isActivate)
        return;
      const tabs = getTabs().filter((t) => !t.hasAttribute("disabled"));
      if (!tabs.length)
        return;
      const names = tabs.map((t) => t.getAttribute("name") ?? "");
      const manual = scope.activationMode() === "manual";
      const focusedTab = e.target.closest("sui-tab") ?? tabs.find((t) => t.getAttribute("name") === scope.active()) ?? tabs[0];
      const focusedName = focusedTab?.getAttribute("name") ?? "";
      if (isActivate) {
        e.preventDefault();
        activate(focusedName);
        return;
      }
      e.preventDefault();
      const refName = manual ? focusedName : scope.active();
      const i = Math.max(0, names.indexOf(refName));
      let next = i;
      if (e.key === nextKey)
        next = (i + 1) % names.length;
      else if (e.key === prevKey)
        next = (i - 1 + names.length) % names.length;
      else if (e.key === "Home")
        next = 0;
      else if (e.key === "End")
        next = names.length - 1;
      const targetName = names[next];
      if (!manual)
        activate(targetName);
      queueMicrotask(() => {
        const targetTab = tabs[next];
        const btn = targetTab.shadowRoot?.querySelector("button");
        if (manual && btn) {
          for (const t of tabs) {
            const b = t.shadowRoot?.querySelector("button");
            if (b)
              b.tabIndex = -1;
          }
          btn.tabIndex = 0;
        }
        btn ? btn.focus() : targetTab.focus();
      });
    };
    const onClick = (e) => {
      const target = e.target.closest("sui-tab");
      if (!target || target.hasAttribute("disabled"))
        return;
      const name = target.getAttribute("name") ?? "";
      if (name)
        activate(name);
    };
    const onSlotChange = () => slotVersion.set(slotVersion() + 1);
    const strip = scope.strip();
    const tabSlot = scope.tabSlot();
    if (strip) {
      strip.addEventListener("keydown", onKeydown);
      strip.addEventListener("click", onClick);
    }
    if (tabSlot) {
      tabSlot.addEventListener("slotchange", onSlotChange);
    }
    return () => {
      stopSync();
      if (strip) {
        strip.removeEventListener("keydown", onKeydown);
        strip.removeEventListener("click", onClick);
      }
      if (tabSlot) {
        tabSlot.removeEventListener("slotchange", onSlotChange);
      }
    };
  }
});

// packages/components/primitives/src/sui-tabs/sui-tabs.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_tabs_default = component63("sui-tabs", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    active: prop63.string({ default: "", reflect: true }),
    variant: prop63.union(["underline", "pill", "enclosed"], { default: "underline", reflect: true }),
    size: prop63.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    tone: prop63.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    sticky: prop63.bool({ default: false, reflect: true }),
    orientation: prop63.union(["horizontal", "vertical"], { default: "horizontal", reflect: true }),
    "activation-mode": prop63.union(["automatic", "manual"], { default: "automatic", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --tone: var(--primary);
  }
  :host([sticky]) { position: sticky; top: 0; z-index: 10; background: var(--background); }

  .strip {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
    border-bottom: 1px solid var(--border);
  }

  :host([orientation='vertical']) .strip {
    flex-direction: column;
    align-items: flex-start;
    border-bottom: 0;
    border-right: 1px solid var(--border);
  }

  :host([variant='pill']) .strip {
    gap: 4px;
    border-bottom: 0;
    padding: 4px;
    background: var(--muted);
    border-radius: var(--radius-md);
  }

  :host([variant='enclosed']) .strip {
    gap: 0;
    border-bottom: 1px solid var(--border);
  }

  ::slotted(sui-tab),
  ::slotted([slot='tab']) {
    flex: 0 0 auto;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.active = props.active;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.tone = props.tone;
    $scope.sticky = props.sticky;
    $scope.orientation = props.orientation;
    $scope.activationMode = props.activationMode;
    $scope["strip"] = signal49(null);
    $scope["tabSlot"] = signal49(null);
    let __r = null;
    if (sui_tabs_controller_default && typeof sui_tabs_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_tabs_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets63(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "strip");
    __e0.setAttribute("part", "strip");
    __e0.setAttribute("role", "tablist");
    __cleanups.push(bindAttr34(__e0, "aria-orientation", () => $scope.orientation()));
    attachRef35($scope, "strip", __e0);
    const __e1 = document.createElement("slot");
    attachRef35($scope, "tabSlot", __e1);
    __e0.appendChild(__e1);
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

// packages/components/primitives/src/sui-tabs/sui-tab.sui
import { attachRef as attachRef36, bindAttr as bindAttr35, bindIf as bindIf20, bindText as bindText15, signal as signal50, view as view7 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-tabs/sui-tab.controller.ts
import { controller as controller50, effect as effect45 } from "@ship/sui-runtime";
var sui_tab_controller_default = controller50({
  mount(state, host) {
    const scope = state;
    const stopTabIndex = effect45(() => {
      const btn = scope.btn();
      if (!btn)
        return;
      btn.tabIndex = scope.active() ? 0 : -1;
    });
    return () => {
      stopTabIndex();
    };
  }
});

// packages/components/primitives/src/sui-tabs/sui-tab.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_tab_default = Object.assign(view7((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope["btn"] = signal50(null);
  if (sui_tab_controller_default && typeof sui_tab_controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = sui_tab_controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b5 = (anchor) => bindIf20({
    anchor,
    test: () => {
      if ($scope.count() !== "")
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag5 = document.createDocumentFragment();
        const __c6 = [];
        const __e7 = document.createElement("span");
        __e7.setAttribute("class", "count");
        __e7.setAttribute("aria-hidden", "true");
        const __t8 = document.createTextNode("");
        __e7.appendChild(__t8);
        __c6.push(bindText15(__t8, () => $scope.count()));
        __frag5.appendChild(__e7);
        return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
          for (const c of __c6)
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
      const __e0 = document.createElement("button");
      __e0.setAttribute("part", "button");
      __e0.setAttribute("type", "button");
      __e0.setAttribute("role", "tab");
      __cleanups.push(bindAttr35(__e0, "data-name", () => $scope.name()));
      __cleanups.push(bindAttr35(__e0, "aria-selected", () => $scope.active() ? "true" : "false"));
      __cleanups.push(bindAttr35(__e0, "aria-disabled", () => $scope.disabled() ? "true" : "false"));
      __cleanups.push(bindAttr35(__e0, "disabled", () => $scope.disabled()));
      attachRef36($scope, "btn", __e0);
      const __e1 = document.createElement("span");
      __e1.setAttribute("class", "leading");
      __e1.setAttribute("part", "leading");
      const __e2 = document.createElement("slot");
      __e2.setAttribute("name", "leading");
      __e1.appendChild(__e2);
      __e0.appendChild(__e1);
      const __e3 = document.createElement("slot");
      __e0.appendChild(__e3);
      const __ifAnchor4 = document.createComment("if");
      __e0.appendChild(__ifAnchor4);
      __cleanups.push(__wireBoundary_b5(__ifAnchor4));
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
        __cleanups.push(hydrateAttr(__m["b0"], "data-name", () => $scope.name()));
      if (__m["b1"])
        __cleanups.push(hydrateAttr(__m["b1"], "aria-selected", () => $scope.active() ? "true" : "false"));
      if (__m["b2"])
        __cleanups.push(hydrateAttr(__m["b2"], "aria-disabled", () => $scope.disabled() ? "true" : "false"));
      if (__m["b3"])
        __cleanups.push(hydrateAttr(__m["b3"], "disabled", () => $scope.disabled()));
      if (__m["b4"])
        __cleanups.push(hydrateRef(__m["b4"], $scope, "btn"));
      if (__m["b5"]) {
        const __t = __m["b5"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b5(__anchor));
      }
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/components/primitives/src/sui-textarea/sui-textarea.sui
import { adoptSuiSheets as adoptSuiSheets64, attachRef as attachRef37, bindAttr as bindAttr36, bindValue as bindValue2, component as component64, effect as effect47, prop as prop64, signal as signal51 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-textarea/sui-textarea.controller.ts
import { controller as controller51, effect as effect46 } from "@ship/sui-runtime";
var sui_textarea_controller_default = controller51({
  mount(state, host) {
    const scope = state;
    let blurValue = scope.value();
    const stopListeners = effect46(() => {
      const el = scope.textareaEl();
      if (!el)
        return;
      const onNativeInput = () => {
        host.dispatchEvent(new CustomEvent("sui-input", {
          detail: { value: el.value },
          bubbles: true,
          composed: true
        }));
      };
      const onNativeBlur = () => {
        const current = scope.value();
        if (current !== blurValue) {
          blurValue = current;
          host.dispatchEvent(new CustomEvent("sui-change", {
            detail: { value: current },
            bubbles: true,
            composed: true
          }));
        }
      };
      el.addEventListener("input", onNativeInput);
      el.addEventListener("blur", onNativeBlur);
      return () => {
        el.removeEventListener("input", onNativeInput);
        el.removeEventListener("blur", onNativeBlur);
      };
    });
    return () => stopListeners();
  }
});

// packages/components/primitives/src/sui-textarea/sui-textarea.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_textarea_default = component64("sui-textarea", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    value: prop64.string({ default: "" }),
    placeholder: prop64.string({ default: "" }),
    name: prop64.string({ default: "" }),
    rows: prop64.number({ default: 4 }),
    required: prop64.bool({ default: false, reflect: true }),
    disabled: prop64.bool({ default: false, reflect: true }),
    readonly: prop64.bool({ default: false, reflect: true }),
    invalid: prop64.bool({ default: false, reflect: true }),
    tone: prop64.union(["default", "primary", "info", "success", "warning", "destructive", "accent", "muted"], { default: "default", reflect: true }),
    variant: prop64.union(["default", "code"], { default: "default", reflect: true }),
    size: prop64.union(["sm", "md"], { default: "md", reflect: true }),
    resize: prop64.union(["none", "vertical", "horizontal", "both"], { default: "vertical" }),
    label: prop64.string({ default: "" }),
    "described-by": prop64.string({ default: "" })
  },
  shadow: true,
  formAssociated: true,
  formValue: "value",
  formDisabled: "disabled",
  styles: `
  :host {
    display: block;
    width: 100%;
    --tone: var(--ring-color, var(--primary));
    --tone-fg: var(--foreground);
  }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--ring-color, var(--primary));
    --tone-fg: var(--foreground);
  }
  :host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info); }
  :host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) {
    --ta-px: 12px;
    --ta-py: 10px;
    --ta-fs: var(--text-base, 14px);
    --ta-lh: 1.5;
    --ta-radius: var(--radius-md, 6px);
  }
  :host([size='sm']) {
    --ta-px: 10px;
    --ta-py: 8px;
    --ta-fs: var(--text-sm, 13px);
    --ta-lh: 1.45;
    --ta-radius: var(--radius-sm, 4px);
  }

  textarea {
    display: block;
    width: 100%;
    box-sizing: border-box;
    border: 1px solid var(--border);
    border-radius: var(--ta-radius);
    padding: var(--ta-py) var(--ta-px);
    background: var(--background);
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: var(--ta-fs);
    line-height: var(--ta-lh);
    letter-spacing: -0.005em;
    box-shadow: var(--shadow-xs);
    transition:
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }

  :host(:not([disabled])) textarea:hover {
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-sm);
  }

  textarea:focus-visible,
  textarea:focus {
    outline: none;
    border-color: color-mix(in oklch, var(--tone) 70%, var(--border));
    box-shadow:
      0 0 0 var(--ring-width, 2px)
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }

  :host([invalid]) textarea {
    border-color: color-mix(in oklch, var(--destructive) 65%, var(--border));
  }
  :host([invalid]) textarea:hover {
    border-color: color-mix(in oklch, var(--destructive) 80%, var(--border));
  }
  :host([invalid]) textarea:focus-visible,
  :host([invalid]) textarea:focus {
    border-color: var(--destructive);
    box-shadow:
      0 0 0 var(--ring-width, 2px)
        color-mix(in oklch, var(--destructive) var(--ring-opacity, 50%), transparent);
  }

  textarea::placeholder {
    color: var(--muted-foreground);
    opacity: 1;
  }
  textarea:disabled {
    cursor: not-allowed;
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 70%, transparent);
    color: var(--muted-foreground);
    box-shadow: none;
    opacity: 0.6;
  }
  textarea:read-only { cursor: default; }

  :host([variant='code']) textarea {
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: var(--text-base, 14px);
    line-height: 1.55;
    letter-spacing: 0;
    tab-size: 2;
  }

  @media (forced-colors: active) {
    textarea:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.placeholder = props.placeholder;
    $scope.name = props.name;
    $scope.rows = props.rows;
    $scope.required = props.required;
    $scope.disabled = props.disabled;
    $scope.readonly = props.readonly;
    $scope.invalid = props.invalid;
    $scope.tone = props.tone;
    $scope.variant = props.variant;
    $scope.size = props.size;
    $scope.resize = props.resize;
    $scope.label = props.label;
    $scope.describedBy = props.describedBy;
    $scope["textareaEl"] = signal51(null);
    let __r = null;
    if (sui_textarea_controller_default && typeof sui_textarea_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_textarea_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets64(__root);
    const __e0 = document.createElement("textarea");
    __e0.setAttribute("part", "textarea");
    __cleanups.push(bindAttr36(__e0, "placeholder", () => $scope.placeholder()));
    __cleanups.push(bindAttr36(__e0, "name", () => $scope.name()));
    __cleanups.push(bindAttr36(__e0, "rows", () => $scope.rows()));
    __cleanups.push(bindAttr36(__e0, "disabled", () => $scope.disabled()));
    __cleanups.push(bindAttr36(__e0, "required", () => $scope.required()));
    __cleanups.push(bindAttr36(__e0, "readonly", () => $scope.readonly()));
    __cleanups.push(bindAttr36(__e0, "aria-invalid", () => $scope.invalid() ? "true" : "false"));
    __cleanups.push(bindAttr36(__e0, "aria-required", () => $scope.required() ? "true" : "false"));
    __cleanups.push(bindAttr36(__e0, "aria-label", () => $scope.label()));
    __cleanups.push(bindAttr36(__e0, "aria-describedby", () => $scope.describedBy()));
    __cleanups.push(effect47(() => {
      const __v = $scope.resize();
      if (__v == null)
        __e0.style.removeProperty("resize");
      else
        __e0.style.setProperty("resize", String(__v));
    }));
    __cleanups.push(bindValue2(__e0, $scope.value));
    attachRef37($scope, "textareaEl", __e0);
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

// packages/components/primitives/src/sui-theme-toggle/sui-theme-toggle.sui
import { adoptSuiSheets as adoptSuiSheets65, bindAttr as bindAttr37, bindText as bindText16, component as component65, on as on6, prop as prop65 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-theme-toggle/sui-theme-toggle.controller.ts
import { controller as controller52, derived as derived5 } from "@ship/sui-runtime";
var STORAGE_KEY = "ship-theme";
function resolveInitial(current) {
  if (current === "light" || current === "dark")
    return current;
  if (typeof window === "undefined")
    return "light";
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "light" || saved === "dark")
      return saved;
  } catch {}
  const prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)")?.matches;
  return prefersDark ? "dark" : "light";
}
function applyTheme(theme) {
  if (typeof document === "undefined")
    return;
  document.documentElement.classList.toggle("dark", theme === "dark");
}
var sui_theme_toggle_controller_default = controller52({
  scope: (state, host) => {
    const s = state;
    const isDark = derived5(() => s.theme() === "dark");
    function toggle() {
      const next = s.theme() === "dark" ? "light" : "dark";
      s.theme.set(next);
      try {
        localStorage.setItem(STORAGE_KEY, next);
      } catch {}
      applyTheme(next);
      host.dispatchEvent(new CustomEvent("sui-theme-change", {
        detail: { theme: next },
        bubbles: true,
        composed: true
      }));
    }
    return { isDark, toggle };
  },
  mount(state) {
    const scope = state;
    const resolved = resolveInitial(scope.theme());
    if (resolved !== scope.theme())
      scope.theme.set(resolved);
    applyTheme(resolved);
  }
});

// packages/components/primitives/src/sui-theme-toggle/sui-theme-toggle.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_theme_toggle_default = component65("sui-theme-toggle", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    theme: prop65.union(["light", "dark", "auto"], { default: "auto", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: inline-flex; }
  button {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 4px 8px;
    font: inherit;
    font-size: 12px;
    color: var(--foreground);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  button:hover { background: var(--muted); }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.theme = props.theme;
    let __r = null;
    if (sui_theme_toggle_controller_default && typeof sui_theme_toggle_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_theme_toggle_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets65(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("type", "button");
    __e0.setAttribute("aria-label", "Toggle theme");
    __cleanups.push(bindAttr37(__e0, "aria-pressed", () => $scope.isDark() ? "true" : "false"));
    __e0.setAttribute("title", "Toggle theme");
    __cleanups.push(on6(__e0, "click", function() {
      const __h = $scope.toggle;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __t1 = document.createTextNode("");
    __e0.appendChild(__t1);
    __cleanups.push(bindText16(__t1, () => $scope.isDark() ? "\uD83C\uDF19" : "☀"));
    const __e2 = document.createElement("slot");
    const __t3 = document.createTextNode("");
    __e2.appendChild(__t3);
    __cleanups.push(bindText16(__t3, () => $scope.isDark() ? "Dark" : "Light"));
    __e0.appendChild(__e2);
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

// packages/components/primitives/src/sui-timeline/sui-timeline.sui
import { adoptSuiSheets as adoptSuiSheets66, component as component66 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-timeline/sui-timeline.controller.ts
import { controller as controller53 } from "@ship/sui-runtime";
var sui_timeline_controller_default = controller53({
  mount(_state, host) {
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "list");
  }
});

// packages/components/primitives/src/sui-timeline/sui-timeline.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_timeline_default = component66("sui-timeline", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  shadow: true,
  styles: `
  :host { display: block; position: relative; }
  .list { display: flex; flex-direction: column; gap: 4px; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_timeline_controller_default && typeof sui_timeline_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_timeline_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets66(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "list");
    __e0.setAttribute("part", "list");
    const __e1 = document.createElement("slot");
    __e0.appendChild(__e1);
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

// packages/components/primitives/src/sui-timeline/sui-timeline-item.sui
import { bindIf as bindIf21, bindText as bindText17, view as view8 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-timeline/sui-timeline-item.controller.ts
import { controller as controller54, effect as effect48 } from "@ship/sui-runtime";
var sui_timeline_item_controller_default = controller54({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "listitem");
    const stopRole = effect48(() => {
      if (scope.interactive()) {
        host.setAttribute("role", "button");
        if (!host.hasAttribute("tabindex"))
          host.setAttribute("tabindex", "0");
      } else {
        host.setAttribute("role", "listitem");
        host.removeAttribute("tabindex");
      }
    });
    const emit = () => {
      host.dispatchEvent(new CustomEvent("sui-select", { bubbles: true, composed: true, detail: {} }));
    };
    const onClick = () => {
      if (scope.interactive())
        emit();
    };
    const onKeydown = (e) => {
      if (!scope.interactive())
        return;
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        emit();
      }
    };
    host.addEventListener("click", onClick);
    host.addEventListener("keydown", onKeydown);
    return () => {
      stopRole();
      host.removeEventListener("click", onClick);
      host.removeEventListener("keydown", onKeydown);
    };
  }
});

// packages/components/primitives/src/sui-timeline/sui-timeline-item.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_timeline_item_default = Object.assign(view8((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  if (sui_timeline_item_controller_default && typeof sui_timeline_item_controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = sui_timeline_item_controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b0 = (anchor) => bindIf21({
    anchor,
    test: () => {
      if ($scope.at())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag6 = document.createDocumentFragment();
        const __c7 = [];
        const __e8 = document.createElement("span");
        __e8.setAttribute("class", "at");
        __e8.setAttribute("part", "at");
        const __t9 = document.createTextNode("");
        __e8.appendChild(__t9);
        __c7.push(bindText17(__t9, () => $scope.at()));
        __frag6.appendChild(__e8);
        return { nodes: Array.from(__frag6.childNodes), cleanup: () => {
          for (const c of __c7)
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
      __e0.setAttribute("class", "gutter");
      const __e1 = document.createElement("span");
      __e1.setAttribute("class", "dot");
      __e1.setAttribute("part", "dot");
      __e0.appendChild(__e1);
      container.appendChild(__e0);
      const __e2 = document.createElement("div");
      __e2.setAttribute("class", "body");
      __e2.setAttribute("part", "body");
      const __e3 = document.createElement("div");
      __e3.setAttribute("class", "head");
      const __e4 = document.createElement("slot");
      __e4.setAttribute("name", "title");
      __e3.appendChild(__e4);
      const __ifAnchor5 = document.createComment("if");
      __e3.appendChild(__ifAnchor5);
      __cleanups.push(__wireBoundary_b0(__ifAnchor5));
      __e2.appendChild(__e3);
      const __e10 = document.createElement("div");
      __e10.setAttribute("class", "content");
      const __e11 = document.createElement("slot");
      __e10.appendChild(__e11);
      __e2.appendChild(__e10);
      container.appendChild(__e2);
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
      if (__m["b0"]) {
        const __t = __m["b0"];
        const __doc = __t.ownerDocument ?? document;
        const __anchor = __t.nodeType === 1 ? __doc.createComment("if") : __t;
        if (__t.nodeType === 1 && __t.parentNode)
          __t.parentNode.replaceChild(__anchor, __t);
        __cleanups.push(__wireBoundary_b0(__anchor));
      }
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/components/primitives/src/sui-timestamp/sui-timestamp.sui
import { adoptSuiSheets as adoptSuiSheets67, bindAttr as bindAttr38, bindIf as bindIf22, bindText as bindText18, component as component67, prop as prop66 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-timestamp/sui-timestamp.controller.ts
import { controller as controller55, signal as signal52, derived as derived6, effect as effect49 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-timestamp/sui-timestamp.ts
var MINUTE = 60000;
var HOUR = 3600000;
var DAY = 86400000;
function formatRelative(iso, now = Date.now()) {
  const t = Date.parse(iso);
  if (Number.isNaN(t))
    return iso;
  const diff = now - t;
  const abs = Math.abs(diff);
  const suffix = diff >= 0 ? "ago" : "from now";
  if (abs < MINUTE)
    return "just now";
  if (abs < HOUR)
    return `${Math.floor(abs / MINUTE)}m ${suffix}`;
  if (abs < DAY)
    return `${Math.floor(abs / HOUR)}h ${suffix}`;
  const days = Math.floor(abs / DAY);
  if (days < 30)
    return `${days}d ${suffix}`;
  if (days < 365)
    return `${Math.floor(days / 30)}mo ${suffix}`;
  return `${Math.floor(days / 365)}y ${suffix}`;
}
function absoluteIso(value) {
  const parsed = Date.parse(value);
  return Number.isNaN(parsed) ? value : new Date(parsed).toISOString();
}

// packages/components/primitives/src/sui-timestamp/sui-timestamp.controller.ts
var MINUTE2 = 60000;
var sui_timestamp_controller_default = controller55({
  scope: ({ value }) => {
    const tick = signal52(0);
    const absStr = derived6(() => {
      const v = value();
      return v ? absoluteIso(v) : "";
    });
    const relStr = derived6(() => {
      tick();
      const v = value();
      return v ? formatRelative(v) : "";
    });
    return { tick, absStr, relStr };
  },
  mount(state, host) {
    const stopTitle = effect49(() => {
      const v = state.value();
      if (v)
        host.title = state.absStr();
    });
    const timer = setInterval(() => state.tick.set(state.tick() + 1), MINUTE2);
    return () => {
      stopTitle();
      clearInterval(timer);
    };
  }
});

// packages/components/primitives/src/sui-timestamp/sui-timestamp.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_timestamp_default = component67("sui-timestamp", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    value: prop66.string({ default: "", reflect: true }),
    format: prop66.union(["relative", "absolute", "both"], { default: "relative", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline;
    font-family: var(--font-body);
    font-size: inherit;
    color: inherit;
  }
  .abs { color: var(--muted-foreground); margin-left: 4px; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.value = props.value;
    $scope.format = props.format;
    let __r = null;
    if (sui_timestamp_controller_default && typeof sui_timestamp_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_timestamp_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindIf22({
      anchor,
      test: () => {
        if ($scope.format() === "absolute")
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __t7 = document.createTextNode("");
          __frag5.appendChild(__t7);
          __c6.push(bindText18(__t7, () => $scope.absStr()));
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag8 = document.createDocumentFragment();
          const __c9 = [];
          const __t10 = document.createTextNode("");
          __frag8.appendChild(__t10);
          __c9.push(bindText18(__t10, () => $scope.relStr()));
          return { nodes: Array.from(__frag8.childNodes), cleanup: () => {
            for (const c of __c9)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b5 = (anchor) => bindIf22({
      anchor,
      test: () => {
        if ($scope.format() === "both")
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag12 = document.createDocumentFragment();
          const __c13 = [];
          const __e14 = document.createElement("span");
          __e14.setAttribute("class", "abs");
          __e14.setAttribute("part", "abs");
          const __t15 = document.createTextNode("(");
          __e14.appendChild(__t15);
          const __t16 = document.createTextNode("");
          __e14.appendChild(__t16);
          __c13.push(bindText18(__t16, () => $scope.absStr()));
          const __t17 = document.createTextNode(")");
          __e14.appendChild(__t17);
          __frag12.appendChild(__e14);
          return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
            for (const c of __c13)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b0 = (anchor) => bindIf22({
      anchor,
      test: () => {
        if ($scope.value())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("time");
          __c2.push(bindAttr38(__e3, "datetime", () => $scope.value()));
          __e3.setAttribute("part", "time");
          const __ifAnchor4 = document.createComment("if");
          __e3.appendChild(__ifAnchor4);
          __c2.push(__wireBoundary_b2(__ifAnchor4));
          __frag1.appendChild(__e3);
          const __ifAnchor11 = document.createComment("if");
          __frag1.appendChild(__ifAnchor11);
          __c2.push(__wireBoundary_b5(__ifAnchor11));
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets67(__root);
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

// packages/components/primitives/src/sui-toast/sui-toast.sui
import { adoptSuiSheets as adoptSuiSheets68, attachRef as attachRef38, bindAttr as bindAttr39, bindIf as bindIf23, component as component68, fade as fade2, prop as prop67, registerTransition as registerTransition3, signal as signal54, slide } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-toast/sui-toast.controller.ts
import { controller as controller56, effect as effect50, ref as ref5, signal as signal53 } from "@ship/sui-runtime";
var TONES_WITH_AUTO_ICON = new Set(["info", "success", "warning", "destructive", "accent"]);
var sui_toast_controller_default = controller56({
  scope: () => {
    const hasSlottedIcon = signal53(false);
    const visible = signal53(true);
    const iconSlotEl = ref5(null);
    const titleSlotEl = ref5(null);
    const actionsSlotEl = ref5(null);
    const closeBtn = ref5(null);
    return { hasSlottedIcon, visible, iconSlotEl, titleSlotEl, actionsSlotEl, closeBtn };
  },
  mount(state, host) {
    const scope = state;
    let timer = null;
    let remaining = 0;
    let startedAt = 0;
    let removed = false;
    if (!host.hasAttribute("tabindex"))
      host.tabIndex = 0;
    function clearTimer() {
      if (timer) {
        clearTimeout(timer);
        timer = null;
      }
    }
    function scheduleDismiss() {
      clearTimer();
      const dur = scope.duration();
      if (dur > 0) {
        remaining = dur;
        startedAt = Date.now();
        timer = setTimeout(() => dismiss("timeout"), dur);
        host.style.setProperty("--toast-duration", `${dur}ms`);
      }
    }
    function dismiss(reason = "manual") {
      if (removed)
        return;
      removed = true;
      clearTimer();
      host.dispatchEvent(new CustomEvent("sui-dismiss", {
        detail: { reason },
        bubbles: true,
        composed: true
      }));
      host.remove();
    }
    async function close(reason = "manual") {
      if (removed)
        return;
      const prefersReduce = typeof window !== "undefined" && window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
      if (prefersReduce) {
        dismiss(reason);
        return;
      }
      clearTimer();
      host.setAttribute("closing", "");
      const visSignal = scope.visible;
      visSignal.set(false);
      await new Promise((resolve) => setTimeout(resolve, 200));
      dismiss(reason);
    }
    const onPause = () => {
      if (!timer)
        return;
      const elapsed = Date.now() - startedAt;
      remaining = Math.max(0, remaining - elapsed);
      clearTimer();
      host.toggleAttribute("data-paused", true);
    };
    const onResume = () => {
      if (removed || scope.duration() <= 0)
        return;
      if (remaining <= 0)
        return;
      host.toggleAttribute("data-paused", false);
      startedAt = Date.now();
      timer = setTimeout(() => dismiss("timeout"), remaining);
    };
    const onKeydown = (e) => {
      if (e.key === "Escape" && scope.dismissible()) {
        e.stopPropagation();
        dismiss("manual");
      }
    };
    host.addEventListener("mouseenter", onPause);
    host.addEventListener("mouseleave", onResume);
    host.addEventListener("focusin", onPause);
    host.addEventListener("focusout", onResume);
    host.addEventListener("keydown", onKeydown);
    scheduleDismiss();
    const hasSlottedIconSig = scope.hasSlottedIcon;
    const slotIsFilled = (slot) => slot.assignedNodes({ flatten: true }).some((n) => {
      if (n.nodeType === Node.ELEMENT_NODE)
        return true;
      return (n.textContent?.trim().length ?? 0) > 0;
    });
    const iconSlotEl = scope.iconSlotEl();
    if (iconSlotEl) {
      const onIconSlotChange = () => hasSlottedIconSig.set(slotIsFilled(iconSlotEl));
      iconSlotEl.addEventListener("slotchange", onIconSlotChange);
      hasSlottedIconSig.set(slotIsFilled(iconSlotEl));
    }
    const titleSlotEl = scope.titleSlotEl();
    const actionsSlotEl = scope.actionsSlotEl();
    if (titleSlotEl) {
      const onTitleChange = () => host.toggleAttribute("data-no-title", !slotIsFilled(titleSlotEl));
      titleSlotEl.addEventListener("slotchange", onTitleChange);
      host.toggleAttribute("data-no-title", !slotIsFilled(titleSlotEl));
    }
    if (actionsSlotEl) {
      const onActionsChange = () => host.toggleAttribute("data-no-actions", !slotIsFilled(actionsSlotEl));
      actionsSlotEl.addEventListener("slotchange", onActionsChange);
      host.toggleAttribute("data-no-actions", !slotIsFilled(actionsSlotEl));
    }
    const stopIconAttrEffect = effect50(() => {
      const tone = scope.tone();
      const slotted2 = scope.hasSlottedIcon();
      const hasAuto = TONES_WITH_AUTO_ICON.has(tone);
      host.toggleAttribute("data-no-icon", !hasAuto && !slotted2);
    });
    const stopCloseEffect = effect50(() => {
      const btn = scope.closeBtn();
      if (!btn)
        return;
      const onClick = () => dismiss("manual");
      btn.addEventListener("click", onClick);
      return () => btn.removeEventListener("click", onClick);
    });
    const h = host;
    h.dismiss = dismiss;
    h.close = close;
    return () => {
      removed = true;
      clearTimer();
      host.removeEventListener("mouseenter", onPause);
      host.removeEventListener("mouseleave", onResume);
      host.removeEventListener("focusin", onPause);
      host.removeEventListener("focusout", onResume);
      host.removeEventListener("keydown", onKeydown);
      stopIconAttrEffect();
      stopCloseEffect();
    };
  }
});

// packages/components/primitives/src/sui-toast/sui-toast.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_toast_default = component68("sui-toast", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    tone: prop67.union(["default", "info", "success", "warning", "destructive", "accent", "muted", "primary"], { default: "default", reflect: true }),
    size: prop67.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    duration: prop67.number({ default: 5000 }),
    dismissible: prop67.bool({ default: true, reflect: true }),
    "show-progress": prop67.bool({ default: false, reflect: true }),
    closing: prop67.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    --toast-px: 14px;
    --toast-py: 12px;
    --toast-fs: var(--text-base, 14px);
    --toast-title-fs: var(--text-base, 14px);
    --toast-gap: 10px;
    --toast-min-w: 280px;
    --toast-max-w: 420px;
    --toast-slide-x: 16px;
    --toast-slide-y: 0;
    /* tone channel defaults */
    --tone-bg: transparent;
    --tone-border: var(--border, #e5e5e5);
    pointer-events: auto;
    will-change: transform, opacity;
  }

  :host([size='sm']) {
    --toast-px: 12px;
    --toast-py: 10px;
    --toast-fs: var(--text-sm, 12px);
    --toast-title-fs: var(--text-sm, 12px);
    --toast-gap: 8px;
    --toast-min-w: 240px;
    --toast-max-w: 360px;
  }
  :host([size='lg']) {
    --toast-px: 16px;
    --toast-py: 14px;
    --toast-fs: var(--text-md, 16px);
    --toast-title-fs: var(--text-md, 16px);
    --toast-gap: 12px;
    --toast-min-w: 320px;
    --toast-max-w: 460px;
  }

  /* Tone channel — bg + border per semantic tone */
  :host([tone='info'])        { --tone-bg: var(--tone-info-bg, color-mix(in oklch, var(--info, #3b82f6) 14%, transparent));        --tone-border: var(--tone-info-border, color-mix(in oklch, var(--info, #3b82f6) 30%, transparent)); }
  :host([tone='success'])     { --tone-bg: var(--tone-success-bg, color-mix(in oklch, var(--success, #22c55e) 14%, transparent));   --tone-border: var(--tone-success-border, color-mix(in oklch, var(--success, #22c55e) 30%, transparent)); }
  :host([tone='warning'])     { --tone-bg: var(--tone-warning-bg, color-mix(in oklch, var(--warning, #f59e0b) 14%, transparent));   --tone-border: var(--tone-warning-border, color-mix(in oklch, var(--warning, #f59e0b) 30%, transparent)); }
  :host([tone='destructive']) { --tone-bg: var(--tone-destructive-bg, color-mix(in oklch, var(--destructive, #ef4444) 14%, transparent)); --tone-border: var(--tone-destructive-border, color-mix(in oklch, var(--destructive, #ef4444) 30%, transparent)); }
  :host([tone='accent'])      { --tone-bg: var(--tone-accent-bg, color-mix(in oklch, var(--accent, #a855f7) 14%, transparent));     --tone-border: var(--tone-accent-border, color-mix(in oklch, var(--accent, #a855f7) 30%, transparent)); }

  :host([data-slide-from='right'])  { --toast-slide-x: 16px;  --toast-slide-y: 0; }
  :host([data-slide-from='left'])   { --toast-slide-x: -16px; --toast-slide-y: 0; }
  :host([data-slide-from='top'])    { --toast-slide-x: 0;     --toast-slide-y: -12px; }
  :host([data-slide-from='bottom']) { --toast-slide-x: 0;     --toast-slide-y: 12px; }

  .toast {
    display: grid;
    grid-template-columns: auto 1fr auto;
    column-gap: var(--toast-gap);
    row-gap: 4px;
    align-items: start;
    padding: var(--toast-py) var(--toast-px);
    padding-right: calc(var(--toast-px) + 24px);
    min-width: var(--toast-min-w);
    max-width: var(--toast-max-w);
    border-radius: var(--radius-md, 6px);
    border: 1px solid var(--tone-border);
    background: color-mix(in oklch, var(--tone-bg) 1, var(--card, #fff));
    color: var(--card-foreground, #111);
    box-shadow:
      0 0 0 1px color-mix(in oklch, black 3%, transparent),
      var(--shadow-md, 0 4px 12px rgba(0,0,0,0.1));
    font-family: var(--font-body, system-ui);
    font-size: var(--toast-fs);
    line-height: var(--leading-normal, 1.5);
    position: relative;
    overflow: hidden;
    isolation: isolate;
  }
  /* Enter/exit polish is driven by \`in:slide\` + \`out:fade\` on the
     #if-mounted .toast (see template). */

  :host(:not([tone])) .toast,
  :host([tone='default']) .toast {
    border-color: color-mix(in oklch, var(--border, #e5e5e5) 160%, transparent);
    background: var(--card, #fff);
  }

  :host([tone='info']) .toast,
  :host([tone='success']) .toast,
  :host([tone='warning']) .toast,
  :host([tone='destructive']) .toast,
  :host([tone='accent']) .toast {
    background: color-mix(in oklch, var(--tone-bg) 70%, var(--card, #fff));
    border-color: var(--tone-border);
  }

  .toast::before {
    content: '';
    position: absolute;
    inset-inline-start: 0;
    top: 0; bottom: 0;
    width: 3px;
    background: var(--tone-fg, var(--foreground, #111));
    opacity: 0;
    transition: opacity var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  :host([tone='info']) .toast::before,
  :host([tone='success']) .toast::before,
  :host([tone='warning']) .toast::before,
  :host([tone='destructive']) .toast::before,
  :host([tone='accent']) .toast::before { opacity: 1; }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    flex: 0 0 auto;
    color: var(--tone-fg, var(--muted-foreground, #888));
    margin-top: 1px;
  }
  :host(:not([tone])) .icon,
  :host([tone='default']) .icon { color: var(--muted-foreground, #888); }
  .icon svg { width: 18px; height: 18px; display: block; }
  :host([size='sm']) .icon,
  :host([size='sm']) .icon svg { width: 16px; height: 16px; }
  :host([size='lg']) .icon,
  :host([size='lg']) .icon svg { width: 20px; height: 20px; }

  :host([data-no-icon]) .toast { grid-template-columns: 1fr auto; }
  :host([data-no-icon]) .icon { display: none; }

  .content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    color: var(--card-foreground, #111);
  }
  .title {
    font-weight: var(--font-weight-semibold, 600);
    font-size: var(--toast-title-fs);
    letter-spacing: -0.005em;
    line-height: var(--leading-tight, 1.25);
    color: var(--foreground, #111);
  }
  .body {
    font-size: var(--toast-fs);
    color: var(--muted-foreground, #888);
    line-height: var(--leading-normal, 1.5);
    word-break: break-word;
  }
  :host([data-no-title]) .body { color: var(--card-foreground, #111); }
  :host([data-no-title]) .title { display: none; }

  .actions {
    display: flex;
    gap: var(--space-2, 8px);
    margin-top: 6px;
    grid-column: 2;
  }
  :host([data-no-actions]) .actions { display: none; margin-top: 0; }

  .close {
    position: absolute;
    top: 8px;
    inset-inline-end: 8px;
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    color: var(--muted-foreground, #888);
    cursor: pointer;
    padding: 0;
    font: inherit;
    z-index: 1;
    transition:
      background-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .close:hover {
    background: var(--active-bg, color-mix(in oklch, currentColor 8%, transparent));
    color: var(--foreground, #111);
  }
  .close:active { transform: scale(0.94); }
  .close:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 2px var(--card, #fff),
      0 0 0 4px color-mix(in oklch, var(--tone-fg, var(--primary, #3b82f6)) 55%, transparent);
  }
  .close svg { width: 12px; height: 12px; display: block; }

  .progress {
    position: absolute;
    left: 0; right: 0; bottom: 0;
    height: 2px;
    background: var(--tone-fg, var(--muted-foreground, #888));
    transform: scaleX(1);
    transform-origin: left center;
    animation: sui-toast-progress var(--toast-duration, 5000ms) linear forwards;
  }
  :host(:not([tone])) .progress,
  :host([tone='default']) .progress { background: var(--muted-foreground, #888); }
  :host(:hover) .progress,
  :host([data-paused]) .progress { animation-play-state: paused; }
  @keyframes sui-toast-progress {
    from { transform: scaleX(1); }
    to   { transform: scaleX(0); }
  }

  @media (prefers-reduced-motion: reduce) {
    .progress { animation: none; }
  }

  @media (forced-colors: active) {
    .toast { border: 1px solid CanvasText; }
    .close:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.tone = props.tone;
    $scope.size = props.size;
    $scope.duration = props.duration;
    $scope.dismissible = props.dismissible;
    $scope.showProgress = props.showProgress;
    $scope.closing = props.closing;
    $scope["actionsSlotEl"] = signal54(null);
    $scope["closeBtn"] = signal54(null);
    $scope["iconSlotEl"] = signal54(null);
    $scope["titleSlotEl"] = signal54(null);
    let __r = null;
    if (sui_toast_controller_default && typeof sui_toast_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_toast_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b5 = (anchor) => bindIf23({
      anchor,
      test: () => {
        const __d = $scope.tone();
        if (__d === "info")
          return 0;
        if (__d === "success")
          return 1;
        if (__d === "warning")
          return 2;
        if (__d === "destructive")
          return 3;
        if (__d === "accent")
          return 4;
        return -1;
      },
      branches: [
        () => {
          const __frag10 = document.createDocumentFragment();
          const __c11 = [];
          const __e12 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __e12.setAttribute("viewBox", "0 0 24 24");
          __e12.setAttribute("fill", "none");
          __e12.setAttribute("stroke", "currentColor");
          __e12.setAttribute("stroke-width", "2");
          __e12.setAttribute("stroke-linecap", "round");
          __e12.setAttribute("stroke-linejoin", "round");
          __e12.setAttribute("aria-hidden", "true");
          const __e13 = document.createElementNS("http://www.w3.org/2000/svg", "circle");
          __e13.setAttribute("cx", "12");
          __e13.setAttribute("cy", "12");
          __e13.setAttribute("r", "9");
          __e12.appendChild(__e13);
          const __e14 = document.createElementNS("http://www.w3.org/2000/svg", "path");
          __e14.setAttribute("d", "M12 8h.01");
          __e12.appendChild(__e14);
          const __e15 = document.createElementNS("http://www.w3.org/2000/svg", "path");
          __e15.setAttribute("d", "M11 12h1v4h1");
          __e12.appendChild(__e15);
          __frag10.appendChild(__e12);
          return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
            for (const c of __c11)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag16 = document.createDocumentFragment();
          const __c17 = [];
          const __e18 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __e18.setAttribute("viewBox", "0 0 24 24");
          __e18.setAttribute("fill", "none");
          __e18.setAttribute("stroke", "currentColor");
          __e18.setAttribute("stroke-width", "2");
          __e18.setAttribute("stroke-linecap", "round");
          __e18.setAttribute("stroke-linejoin", "round");
          __e18.setAttribute("aria-hidden", "true");
          const __e19 = document.createElementNS("http://www.w3.org/2000/svg", "circle");
          __e19.setAttribute("cx", "12");
          __e19.setAttribute("cy", "12");
          __e19.setAttribute("r", "9");
          __e18.appendChild(__e19);
          const __e20 = document.createElementNS("http://www.w3.org/2000/svg", "path");
          __e20.setAttribute("d", "m8.5 12.5 2.5 2.5 4.5-5");
          __e18.appendChild(__e20);
          __frag16.appendChild(__e18);
          return { nodes: Array.from(__frag16.childNodes), cleanup: () => {
            for (const c of __c17)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag21 = document.createDocumentFragment();
          const __c22 = [];
          const __e23 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __e23.setAttribute("viewBox", "0 0 24 24");
          __e23.setAttribute("fill", "none");
          __e23.setAttribute("stroke", "currentColor");
          __e23.setAttribute("stroke-width", "2");
          __e23.setAttribute("stroke-linecap", "round");
          __e23.setAttribute("stroke-linejoin", "round");
          __e23.setAttribute("aria-hidden", "true");
          const __e24 = document.createElementNS("http://www.w3.org/2000/svg", "path");
          __e24.setAttribute("d", "M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z");
          __e23.appendChild(__e24);
          const __e25 = document.createElementNS("http://www.w3.org/2000/svg", "line");
          __e25.setAttribute("x1", "12");
          __e25.setAttribute("x2", "12");
          __e25.setAttribute("y1", "9");
          __e25.setAttribute("y2", "13");
          __e23.appendChild(__e25);
          const __e26 = document.createElementNS("http://www.w3.org/2000/svg", "line");
          __e26.setAttribute("x1", "12");
          __e26.setAttribute("x2", "12.01");
          __e26.setAttribute("y1", "17");
          __e26.setAttribute("y2", "17");
          __e23.appendChild(__e26);
          __frag21.appendChild(__e23);
          return { nodes: Array.from(__frag21.childNodes), cleanup: () => {
            for (const c of __c22)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag27 = document.createDocumentFragment();
          const __c28 = [];
          const __e29 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __e29.setAttribute("viewBox", "0 0 24 24");
          __e29.setAttribute("fill", "none");
          __e29.setAttribute("stroke", "currentColor");
          __e29.setAttribute("stroke-width", "2");
          __e29.setAttribute("stroke-linecap", "round");
          __e29.setAttribute("stroke-linejoin", "round");
          __e29.setAttribute("aria-hidden", "true");
          const __e30 = document.createElementNS("http://www.w3.org/2000/svg", "circle");
          __e30.setAttribute("cx", "12");
          __e30.setAttribute("cy", "12");
          __e30.setAttribute("r", "9");
          __e29.appendChild(__e30);
          const __e31 = document.createElementNS("http://www.w3.org/2000/svg", "line");
          __e31.setAttribute("x1", "15");
          __e31.setAttribute("x2", "9");
          __e31.setAttribute("y1", "9");
          __e31.setAttribute("y2", "15");
          __e29.appendChild(__e31);
          const __e32 = document.createElementNS("http://www.w3.org/2000/svg", "line");
          __e32.setAttribute("x1", "9");
          __e32.setAttribute("x2", "15");
          __e32.setAttribute("y1", "9");
          __e32.setAttribute("y2", "15");
          __e29.appendChild(__e32);
          __frag27.appendChild(__e29);
          return { nodes: Array.from(__frag27.childNodes), cleanup: () => {
            for (const c of __c28)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag33 = document.createDocumentFragment();
          const __c34 = [];
          const __e35 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __e35.setAttribute("viewBox", "0 0 24 24");
          __e35.setAttribute("fill", "none");
          __e35.setAttribute("stroke", "currentColor");
          __e35.setAttribute("stroke-width", "2");
          __e35.setAttribute("stroke-linecap", "round");
          __e35.setAttribute("stroke-linejoin", "round");
          __e35.setAttribute("aria-hidden", "true");
          const __e36 = document.createElementNS("http://www.w3.org/2000/svg", "path");
          __e36.setAttribute("d", "M12 2 3 14h7l-1 8 10-12h-7l1-8Z");
          __e35.appendChild(__e36);
          __frag33.appendChild(__e35);
          return { nodes: Array.from(__frag33.childNodes), cleanup: () => {
            for (const c of __c34)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b4 = (anchor) => bindIf23({
      anchor,
      test: () => {
        if (!$scope.hasSlottedIcon())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag7 = document.createDocumentFragment();
          const __c8 = [];
          const __matchAnchor9 = document.createComment("match");
          __frag7.appendChild(__matchAnchor9);
          __c8.push(__wireBoundary_b5(__matchAnchor9));
          return { nodes: Array.from(__frag7.childNodes), cleanup: () => {
            for (const c of __c8)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b8 = (anchor) => bindIf23({
      anchor,
      test: () => {
        if ($scope.dismissible())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag45 = document.createDocumentFragment();
          const __c46 = [];
          const __e47 = document.createElement("button");
          __e47.setAttribute("class", "close");
          __e47.setAttribute("part", "close");
          __e47.setAttribute("type", "button");
          __e47.setAttribute("aria-label", "Dismiss notification");
          attachRef38($scope, "closeBtn", __e47);
          const __e48 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
          __e48.setAttribute("viewBox", "0 0 24 24");
          __e48.setAttribute("fill", "none");
          __e48.setAttribute("stroke", "currentColor");
          __e48.setAttribute("stroke-width", "2");
          __e48.setAttribute("stroke-linecap", "round");
          __e48.setAttribute("stroke-linejoin", "round");
          __e48.setAttribute("aria-hidden", "true");
          const __e49 = document.createElementNS("http://www.w3.org/2000/svg", "line");
          __e49.setAttribute("x1", "18");
          __e49.setAttribute("y1", "6");
          __e49.setAttribute("x2", "6");
          __e49.setAttribute("y2", "18");
          __e48.appendChild(__e49);
          const __e50 = document.createElementNS("http://www.w3.org/2000/svg", "line");
          __e50.setAttribute("x1", "6");
          __e50.setAttribute("y1", "6");
          __e50.setAttribute("x2", "18");
          __e50.setAttribute("y2", "18");
          __e48.appendChild(__e50);
          __e47.appendChild(__e48);
          __frag45.appendChild(__e47);
          return { nodes: Array.from(__frag45.childNodes), cleanup: () => {
            for (const c of __c46)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b10 = (anchor) => bindIf23({
      anchor,
      test: () => {
        if ($scope.showProgress() && $scope.duration() > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag52 = document.createDocumentFragment();
          const __c53 = [];
          const __e54 = document.createElement("span");
          __e54.setAttribute("class", "progress");
          __e54.setAttribute("aria-hidden", "true");
          __frag52.appendChild(__e54);
          return { nodes: Array.from(__frag52.childNodes), cleanup: () => {
            for (const c of __c53)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b0 = (anchor) => bindIf23({
      anchor,
      test: () => {
        if ($scope.visible())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("div");
          __e3.setAttribute("class", "toast");
          registerTransition3(__e3, slide, { axis: "y", duration: 200 }, "in");
          registerTransition3(__e3, fade2, { duration: 180 }, "out");
          __c2.push(bindAttr39(__e3, "role", () => $scope.tone() === "destructive" || $scope.tone() === "warning" ? "alert" : "status"));
          __c2.push(bindAttr39(__e3, "aria-live", () => $scope.tone() === "destructive" || $scope.tone() === "warning" ? "assertive" : "polite"));
          __e3.setAttribute("aria-atomic", "true");
          __e3.setAttribute("part", "toast");
          const __e4 = document.createElement("span");
          __e4.setAttribute("class", "icon");
          const __e5 = document.createElement("slot");
          __e5.setAttribute("name", "icon");
          attachRef38($scope, "iconSlotEl", __e5);
          __e4.appendChild(__e5);
          const __ifAnchor6 = document.createComment("if");
          __e4.appendChild(__ifAnchor6);
          __c2.push(__wireBoundary_b4(__ifAnchor6));
          __e3.appendChild(__e4);
          const __e37 = document.createElement("div");
          __e37.setAttribute("class", "content");
          const __e38 = document.createElement("div");
          __e38.setAttribute("class", "title");
          const __e39 = document.createElement("slot");
          __e39.setAttribute("name", "title");
          attachRef38($scope, "titleSlotEl", __e39);
          __e38.appendChild(__e39);
          __e37.appendChild(__e38);
          const __e40 = document.createElement("div");
          __e40.setAttribute("class", "body");
          const __e41 = document.createElement("slot");
          __e40.appendChild(__e41);
          __e37.appendChild(__e40);
          const __e42 = document.createElement("div");
          __e42.setAttribute("class", "actions");
          const __e43 = document.createElement("slot");
          __e43.setAttribute("name", "actions");
          attachRef38($scope, "actionsSlotEl", __e43);
          __e42.appendChild(__e43);
          __e37.appendChild(__e42);
          __e3.appendChild(__e37);
          const __ifAnchor44 = document.createComment("if");
          __e3.appendChild(__ifAnchor44);
          __c2.push(__wireBoundary_b8(__ifAnchor44));
          const __ifAnchor51 = document.createComment("if");
          __e3.appendChild(__ifAnchor51);
          __c2.push(__wireBoundary_b10(__ifAnchor51));
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets68(__root);
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

// packages/components/primitives/src/sui-toast/sui-toast-container.sui
import { attachRef as attachRef39, signal as signal56, view as view9 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-toast/sui-toast-container.controller.ts
import { bindHtml as bindHtml4, controller as controller57, effect as effect51, signal as signal55 } from "@ship/sui-runtime";
var SLIDE_FROM = {
  "top-left": "top",
  "top-center": "top",
  "top-right": "top",
  "bottom-left": "bottom",
  "bottom-center": "bottom",
  "bottom-right": "bottom"
};
var EDGE = "16px";
var STACK_BASE_STYLE = "position:fixed;display:flex;gap:var(--space-2,8px);" + "pointer-events:none;z-index:var(--z-toast,9999);" + "max-width:min(440px,calc(100vw - 32px));" + "padding:0;margin:0;box-sizing:border-box;";
function stackStyleFor(position) {
  switch (position) {
    case "top-right":
      return `${STACK_BASE_STYLE}top:${EDGE};right:${EDGE};flex-direction:column;align-items:flex-end;`;
    case "top-left":
      return `${STACK_BASE_STYLE}top:${EDGE};left:${EDGE};flex-direction:column;align-items:flex-start;`;
    case "top-center":
      return `${STACK_BASE_STYLE}top:${EDGE};left:50%;transform:translateX(-50%);flex-direction:column;align-items:center;`;
    case "bottom-left":
      return `${STACK_BASE_STYLE}bottom:${EDGE};left:${EDGE};flex-direction:column-reverse;align-items:flex-start;`;
    case "bottom-center":
      return `${STACK_BASE_STYLE}bottom:${EDGE};left:50%;transform:translateX(-50%);flex-direction:column-reverse;align-items:center;`;
    case "bottom-right":
    default:
      return `${STACK_BASE_STYLE}bottom:${EDGE};right:${EDGE};flex-direction:column-reverse;align-items:flex-end;`;
  }
}
var sui_toast_container_controller_default = controller57({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "region");
    if (!host.hasAttribute("aria-label"))
      host.setAttribute("aria-label", "Notifications");
    if (!host.hasAttribute("aria-live"))
      host.setAttribute("aria-live", "polite");
    const slotVersion = signal55(0);
    const defaultSlotEl = scope.defaultSlotEl();
    const stackEl = scope.stackEl();
    const adoptIntoStack = (toast) => {
      if (!stackEl)
        return;
      if (toast.parentElement !== stackEl)
        stackEl.appendChild(toast);
    };
    const getStackToasts = () => {
      if (!stackEl)
        return [];
      return Array.from(stackEl.children).filter((c) => c.tagName.toLowerCase() === "sui-toast");
    };
    const stopStackStyleEffect = effect51(() => {
      if (!stackEl)
        return;
      const pos = scope.position();
      stackEl.style.cssText = stackStyleFor(pos);
      stackEl.setAttribute("data-position", pos);
    });
    const stopSlotEffect = effect51(() => {
      slotVersion();
      const pos = scope.position();
      const maxVis = scope.maxVisible();
      const dir = SLIDE_FROM[pos] ?? "bottom";
      const toasts = getStackToasts();
      for (const t of toasts) {
        t.setAttribute("data-slide-from", dir);
        t.style.pointerEvents = "auto";
      }
      if (maxVis > 0 && toasts.length > maxVis) {
        const overflow = toasts.length - maxVis;
        for (let i = 0;i < overflow; i++) {
          const victim = toasts[i];
          if (typeof victim.close === "function")
            victim.close("programmatic");
          else
            victim.dismiss?.("programmatic");
        }
      }
    });
    const onSlotChange = () => {
      if (defaultSlotEl) {
        for (const node of defaultSlotEl.assignedElements()) {
          if (node.tagName.toLowerCase() === "sui-toast") {
            adoptIntoStack(node);
          }
        }
      }
      slotVersion.set(slotVersion() + 1);
    };
    if (defaultSlotEl) {
      defaultSlotEl.addEventListener("slotchange", onSlotChange);
      onSlotChange();
    }
    const hostObserver = new MutationObserver((records) => {
      let dirty = false;
      for (const r of records) {
        for (const n of r.addedNodes) {
          if (n.nodeType === Node.ELEMENT_NODE) {
            const el = n;
            if (el.tagName.toLowerCase() === "sui-toast") {
              adoptIntoStack(el);
              dirty = true;
            }
          }
        }
        if (r.removedNodes.length)
          dirty = true;
      }
      if (dirty)
        slotVersion.set(slotVersion() + 1);
    });
    hostObserver.observe(host, { childList: true });
    for (const child of Array.from(host.children)) {
      if (child.tagName.toLowerCase() === "sui-toast") {
        adoptIntoStack(child);
      }
    }
    slotVersion.set(slotVersion() + 1);
    const stackObserver = stackEl ? new MutationObserver(() => slotVersion.set(slotVersion() + 1)) : null;
    if (stackEl && stackObserver) {
      stackObserver.observe(stackEl, { childList: true });
    }
    function show(opts = {}) {
      const doc = host.ownerDocument ?? document;
      const t = doc.createElement("sui-toast");
      if (opts.tone)
        t.setAttribute("tone", opts.tone);
      if (opts.size)
        t.setAttribute("size", opts.size);
      if (typeof opts.duration === "number")
        t.setAttribute("duration", String(opts.duration));
      if (opts.dismissible === false)
        t.removeAttribute("dismissible");
      else
        t.setAttribute("dismissible", "");
      if (opts.showProgress)
        t.setAttribute("show-progress", "");
      if (opts.title) {
        const span = doc.createElement("span");
        span.setAttribute("slot", "title");
        span.textContent = opts.title;
        t.appendChild(span);
      }
      if (opts.description) {
        t.appendChild(doc.createTextNode(opts.description));
      }
      if (opts.actionHtml) {
        const wrap = doc.createElement("span");
        wrap.setAttribute("slot", "actions");
        const html = opts.actionHtml;
        bindHtml4(wrap, () => html);
        t.appendChild(wrap);
      }
      if (stackEl)
        stackEl.appendChild(t);
      else
        host.appendChild(t);
      return t;
    }
    host.show = show;
    return () => {
      stopSlotEffect();
      stopStackStyleEffect();
      hostObserver.disconnect();
      stackObserver?.disconnect();
      if (defaultSlotEl) {
        defaultSlotEl.removeEventListener("slotchange", onSlotChange);
      }
    };
  }
});

// packages/components/primitives/src/sui-toast/sui-toast-container.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_toast_container_default = Object.assign(view9((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope["defaultSlotEl"] = signal56(null);
  $scope["stackEl"] = signal56(null);
  if (sui_toast_container_controller_default && typeof sui_toast_container_controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = sui_toast_container_controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("sui-portal");
      __e0.setAttribute("target", "body");
      const __e1 = document.createElement("div");
      __e1.setAttribute("class", "sui-toast-stack");
      __e1.setAttribute("data-sui-toast-stack", "");
      attachRef39($scope, "stackEl", __e1);
      __e0.appendChild(__e1);
      container.appendChild(__e0);
      const __e2 = document.createElement("slot");
      attachRef39($scope, "defaultSlotEl", __e2);
      container.appendChild(__e2);
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
        __cleanups.push(hydrateRef(__m["b0"], $scope, "stackEl"));
      if (__m["b1"])
        __cleanups.push(hydrateRef(__m["b1"], $scope, "defaultSlotEl"));
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/components/primitives/src/sui-toast-host/sui-toast-host.sui
import { adoptSuiSheets as adoptSuiSheets69, attachRef as attachRef40, bindAttr as bindAttr40, component as component69, prop as prop68, signal as signal57 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-toast-host/sui-toast-host.controller.ts
import { controller as controller58 } from "@ship/sui-runtime";
var sui_toast_host_controller_default = controller58({
  mount(state, host) {
    const scope = state;
    const onNotify = (e) => {
      const detail = e.detail ?? {};
      const container = scope.containerEl();
      if (!container || typeof container.show !== "function")
        return;
      container.show({
        tone: detail.tone,
        duration: detail.duration,
        title: detail.message,
        description: detail.description,
        actionHtml: detail.actionHtml
      });
    };
    const target = host.closest("sui-shell, ship-shell") ?? host.ownerDocument?.body ?? host;
    target.addEventListener("ship:notify", onNotify);
    return () => {
      target.removeEventListener("ship:notify", onNotify);
    };
  }
});

// packages/components/primitives/src/sui-toast-host/sui-toast-host.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_toast_host_default = component69("sui-toast-host", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    position: prop68.union(["top-left", "top-center", "top-right", "bottom-left", "bottom-center", "bottom-right"], { default: "bottom-right", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: contents; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.position = props.position;
    $scope["containerEl"] = signal57(null);
    let __r = null;
    if (sui_toast_host_controller_default && typeof sui_toast_host_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_toast_host_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets69(__root);
    const __e0 = document.createElement("sui-toast-container");
    __cleanups.push(bindAttr40(__e0, "position", () => $scope.position()));
    attachRef40($scope, "containerEl", __e0);
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

// packages/components/primitives/src/sui-toggle-group/sui-toggle.sui
import { attachRef as attachRef41, bindAttr as bindAttr41, signal as signal58, view as view10 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-toggle-group/sui-toggle.controller.ts
import { controller as controller59, effect as effect52 } from "@ship/sui-runtime";
var sui_toggle_controller_default = controller59({
  mount(state, host) {
    const scope = state;
    const stop = effect52(() => {
      const btn = scope.buttonEl();
      if (!btn)
        return;
      const onClick = () => {
        if (scope.disabled())
          return;
        scope.pressed.set(!scope.pressed());
        host.dispatchEvent(new CustomEvent("sui-toggle", {
          detail: { pressed: scope.pressed(), value: scope.value() },
          bubbles: true,
          composed: true
        }));
      };
      btn.addEventListener("click", onClick);
      return () => btn.removeEventListener("click", onClick);
    });
    host.focusButton = () => {
      const btn = host.shadowRoot?.querySelector("button");
      btn?.focus();
    };
    return () => stop();
  }
});

// packages/components/primitives/src/sui-toggle-group/sui-toggle.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_toggle_default = Object.assign(view10((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope["buttonEl"] = signal58(null);
  if (sui_toggle_controller_default && typeof sui_toggle_controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = sui_toggle_controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("button");
      __e0.setAttribute("part", "button");
      __e0.setAttribute("type", "button");
      __cleanups.push(bindAttr41(__e0, "aria-pressed", () => $scope.pressed() ? "true" : "false"));
      __cleanups.push(bindAttr41(__e0, "disabled", () => $scope.disabled()));
      attachRef41($scope, "buttonEl", __e0);
      const __e1 = document.createElement("slot");
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
        __cleanups.push(hydrateAttr(__m["b0"], "aria-pressed", () => $scope.pressed() ? "true" : "false"));
      if (__m["b1"])
        __cleanups.push(hydrateAttr(__m["b1"], "disabled", () => $scope.disabled()));
      if (__m["b2"])
        __cleanups.push(hydrateRef(__m["b2"], $scope, "buttonEl"));
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/components/primitives/src/sui-toggle-group/sui-toggle-group.sui
import { adoptSuiSheets as adoptSuiSheets70, attachRef as attachRef42, component as component70, prop as prop69, signal as signal59 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-toggle-group/sui-toggle-group.controller.ts
import { controller as controller60, effect as effect53 } from "@ship/sui-runtime";
var sui_toggle_group_controller_default = controller60({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "group");
    const stopAriaOrientation = effect53(() => {
      host.setAttribute("aria-orientation", scope.orientation());
    });
    const hiddenInput = document.createElement("input");
    hiddenInput.type = "hidden";
    host.appendChild(hiddenInput);
    const getToggles = () => {
      const slot = scope.defaultSlotEl();
      if (!slot)
        return Array.from(host.querySelectorAll(":scope > sui-toggle"));
      return slot.assignedElements().filter((el) => el.tagName.toLowerCase() === "sui-toggle");
    };
    const enabledToggles = () => getToggles().filter((t) => !t.hasAttribute("disabled"));
    const getSelectedValues = () => new Set(scope.value().split(",").filter(Boolean));
    const setSelectedValues = (vals) => {
      const joined = Array.from(vals).join(",");
      scope.value.set(joined);
      syncToggles();
      hiddenInput.name = scope.name();
      hiddenInput.value = joined;
      host.dispatchEvent(new CustomEvent("sui-change", {
        detail: { value: joined },
        bubbles: true,
        composed: true
      }));
    };
    const syncToggles = () => {
      const selected = getSelectedValues();
      const size = scope.size();
      const variant = scope.variant();
      for (const toggle of getToggles()) {
        const val = toggle.getAttribute("value") ?? "";
        toggle.toggleAttribute("pressed", selected.has(val));
        if (size)
          toggle.setAttribute("size", size);
        if (variant)
          toggle.setAttribute("variant", variant);
      }
    };
    const onToggle = (e) => {
      const ce = e;
      ce.stopPropagation();
      const toggle = ce.composedPath().find((el) => el.tagName?.toLowerCase() === "sui-toggle");
      if (!toggle)
        return;
      const val = toggle.getAttribute("value") ?? "";
      const selected = getSelectedValues();
      if (scope.type() === "single") {
        if (selected.has(val))
          setSelectedValues(new Set);
        else
          setSelectedValues(new Set([val]));
      } else {
        if (selected.has(val))
          selected.delete(val);
        else
          selected.add(val);
        setSelectedValues(selected);
      }
    };
    const onKeydown = (e) => {
      const vertical = scope.orientation() === "vertical";
      const nextKey = vertical ? "ArrowDown" : "ArrowRight";
      const prevKey = vertical ? "ArrowUp" : "ArrowLeft";
      if (e.key !== nextKey && e.key !== prevKey)
        return;
      const items = enabledToggles();
      if (!items.length)
        return;
      const current = e.composedPath().find((el) => items.includes(el));
      const i = current ? items.indexOf(current) : -1;
      const next = e.key === nextKey ? (i + 1) % items.length : (i - 1 + items.length) % items.length;
      items[next]?.focusButton?.();
      e.preventDefault();
    };
    const stopSync = effect53(() => {
      scope.value();
      scope.size();
      scope.variant();
      syncToggles();
    });
    const stopSlot = effect53(() => {
      const slot = scope.defaultSlotEl();
      if (!slot)
        return;
      const onSlotChange = () => syncToggles();
      slot.addEventListener("slotchange", onSlotChange);
      return () => slot.removeEventListener("slotchange", onSlotChange);
    });
    const stopHidden = effect53(() => {
      hiddenInput.name = scope.name();
      hiddenInput.value = scope.value();
    });
    host.addEventListener("sui-toggle", onToggle);
    host.addEventListener("keydown", onKeydown);
    return () => {
      stopSync();
      stopSlot();
      stopHidden();
      stopAriaOrientation();
      host.removeEventListener("sui-toggle", onToggle);
      host.removeEventListener("keydown", onKeydown);
      hiddenInput.remove();
    };
  }
});

// packages/components/primitives/src/sui-toggle-group/sui-toggle-group.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_toggle_group_default = component70("sui-toggle-group", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    type: prop69.union(["single", "multiple"], { default: "single", reflect: true }),
    value: prop69.string({ default: "", reflect: true }),
    size: prop69.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    variant: prop69.union(["outline", "ghost"], { default: "outline", reflect: true }),
    name: prop69.string({ default: "" }),
    orientation: prop69.union(["horizontal", "vertical"], { default: "horizontal", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    gap: 0;
  }
  :host([orientation='vertical']) { flex-direction: column; }

  ::slotted(sui-toggle:not(:first-child)) { margin-left: -1px; }
  ::slotted(sui-toggle:first-child) { border-radius: var(--radius, 6px) 0 0 var(--radius, 6px); }
  ::slotted(sui-toggle:last-child) { border-radius: 0 var(--radius, 6px) var(--radius, 6px) 0; }
  ::slotted(sui-toggle:not(:first-child):not(:last-child)) { border-radius: 0; }

  :host([orientation='vertical']) ::slotted(sui-toggle:not(:first-child)) { margin-left: 0; margin-top: -1px; }
  :host([orientation='vertical']) ::slotted(sui-toggle:first-child) { border-radius: var(--radius, 6px) var(--radius, 6px) 0 0; }
  :host([orientation='vertical']) ::slotted(sui-toggle:last-child) { border-radius: 0 0 var(--radius, 6px) var(--radius, 6px); }
  :host([orientation='vertical']) ::slotted(sui-toggle:not(:first-child):not(:last-child)) { border-radius: 0; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.type = props.type;
    $scope.value = props.value;
    $scope.size = props.size;
    $scope.variant = props.variant;
    $scope.name = props.name;
    $scope.orientation = props.orientation;
    $scope["defaultSlotEl"] = signal59(null);
    let __r = null;
    if (sui_toggle_group_controller_default && typeof sui_toggle_group_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_toggle_group_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets70(__root);
    const __e0 = document.createElement("slot");
    attachRef42($scope, "defaultSlotEl", __e0);
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

// packages/components/primitives/src/sui-toolbar/sui-toolbar.sui
import { adoptSuiSheets as adoptSuiSheets71, attachRef as attachRef43, component as component71, prop as prop70, signal as signal60 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-toolbar/sui-toolbar.controller.ts
import { controller as controller61 } from "@ship/sui-runtime";
var sui_toolbar_controller_default = controller61({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "toolbar");
    if (!host.hasAttribute("aria-orientation"))
      host.setAttribute("aria-orientation", "horizontal");
    const focusable = () => {
      const slot = scope.defaultSlot();
      const els = slot?.assignedElements({ flatten: true }) ?? [];
      return els.filter((el) => !el.hasAttribute("disabled"));
    };
    const onKey = (e) => {
      if (e.key !== "ArrowRight" && e.key !== "ArrowLeft")
        return;
      const items = focusable();
      if (!items.length)
        return;
      const active = e.composedPath().find((n) => items.includes(n)) || null;
      const i = active ? items.indexOf(active) : -1;
      const next = e.key === "ArrowRight" ? (i + 1) % items.length : (i - 1 + items.length) % items.length;
      items[next]?.focus();
      e.preventDefault();
    };
    host.addEventListener("keydown", onKey);
    return () => host.removeEventListener("keydown", onKey);
  }
});

// packages/components/primitives/src/sui-toolbar/sui-toolbar.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_toolbar_default = component71("sui-toolbar", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    justify: prop70.union(["start", "center", "end", "between"], { default: "start", reflect: true }),
    align: prop70.union(["center", "start", "end", "baseline"], { default: "center", reflect: true }),
    gap: prop70.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    variant: prop70.union(["default", "bordered"], { default: "default", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  :host([gap='sm']) { gap: 4px; }
  :host([gap='lg']) { gap: 12px; }
  :host([justify='center']) { justify-content: center; }
  :host([justify='end']) { justify-content: flex-end; }
  :host([justify='between']) { justify-content: space-between; }
  :host([align='start']) { align-items: flex-start; }
  :host([align='end']) { align-items: flex-end; }
  :host([align='baseline']) { align-items: baseline; }
  :host([variant='bordered']) {
    padding: 12px 20px;
    border-top: 1px solid var(--border);
  }
  ::slotted([slot='spacer']) { flex: 1; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.justify = props.justify;
    $scope.align = props.align;
    $scope.gap = props.gap;
    $scope.variant = props.variant;
    $scope["defaultSlot"] = signal60(null);
    let __r = null;
    if (sui_toolbar_controller_default && typeof sui_toolbar_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_toolbar_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets71(__root);
    const __e0 = document.createElement("slot");
    __e0.setAttribute("name", "spacer");
    __root.appendChild(__e0);
    const __e1 = document.createElement("slot");
    attachRef43($scope, "defaultSlot", __e1);
    __root.appendChild(__e1);
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

// packages/components/primitives/src/sui-tooltip/sui-tooltip.sui
import { adoptSuiSheets as adoptSuiSheets72, attachRef as attachRef44, component as component72, prop as prop71, signal as signal61 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-tooltip/sui-tooltip.controller.ts
import { controller as controller62, effect as effect54 } from "@ship/sui-runtime";
var _seq8 = 0;
var sui_tooltip_controller_default = controller62({
  mount(state, host) {
    const scope = state;
    const bubbleId = `sui-tooltip-${++_seq8}`;
    const stopBubbleId = effect54(() => {
      const bubble = scope.bubbleEl();
      if (bubble)
        bubble.id = bubbleId;
    });
    const wireTrigger = () => {
      const slot = scope.defaultSlot();
      if (!slot)
        return;
      const first = slot.assignedElements({ flatten: true })[0];
      if (!first)
        return;
      const existing = first.getAttribute("aria-describedby") ?? "";
      if (!existing.split(/\s+/).includes(bubbleId)) {
        first.setAttribute("aria-describedby", existing ? `${existing} ${bubbleId}` : bubbleId);
      }
    };
    const stopSlot = effect54(() => {
      const slot = scope.defaultSlot();
      if (!slot)
        return;
      slot.addEventListener("slotchange", wireTrigger);
      wireTrigger();
      return () => slot.removeEventListener("slotchange", wireTrigger);
    });
    const onKeydown = (e) => {
      if (e.key === "Escape")
        host.blur();
    };
    host.addEventListener("keydown", onKeydown);
    return () => {
      stopBubbleId();
      stopSlot();
      host.removeEventListener("keydown", onKeydown);
    };
  }
});

// packages/components/primitives/src/sui-tooltip/sui-tooltip.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_tooltip_default = component72("sui-tooltip", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    placement: prop71.union(["top", "bottom", "left", "right"], { default: "top", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    position: relative;
    display: inline-flex;
  }

  .bubble {
    position: absolute;
    z-index: var(--z-popover, 50);
    padding: 4px 8px;
    background: var(--foreground);
    color: var(--background);
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: var(--font-weight-medium, 500);
    border-radius: var(--radius-sm);
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transition: opacity var(--duration-fast, 120ms) var(--ease-out, ease-out);
    box-shadow: var(--shadow-sm);
  }

  :host(:hover) .bubble,
  :host(:focus-within) .bubble {
    opacity: 1;
  }

  /* Placement: top (default) */
  :host(:not([placement])) .bubble,
  :host([placement='top']) .bubble {
    bottom: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
  }

  :host([placement='bottom']) .bubble {
    top: calc(100% + 6px);
    left: 50%;
    transform: translateX(-50%);
  }

  :host([placement='left']) .bubble {
    right: calc(100% + 6px);
    top: 50%;
    transform: translateY(-50%);
  }

  :host([placement='right']) .bubble {
    left: calc(100% + 6px);
    top: 50%;
    transform: translateY(-50%);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.placement = props.placement;
    $scope["bubbleEl"] = signal61(null);
    $scope["defaultSlot"] = signal61(null);
    let __r = null;
    if (sui_tooltip_controller_default && typeof sui_tooltip_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_tooltip_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets72(__root);
    const __e0 = document.createElement("slot");
    attachRef44($scope, "defaultSlot", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "bubble");
    __e1.setAttribute("role", "tooltip");
    __e1.setAttribute("part", "bubble");
    attachRef44($scope, "bubbleEl", __e1);
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "content");
    __e1.appendChild(__e2);
    __root.appendChild(__e1);
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

// packages/components/primitives/src/sui-tree-view/sui-tree-view.sui
import { adoptSuiSheets as adoptSuiSheets73, bindFor as bindFor6, bindIf as bindIf24, bindProp as bindProp3, component as component73, signal as signal63 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-tree-view/sui-tree-view.controller.ts
import { controller as controller63, signal as signal62 } from "@ship/sui-runtime";
var sui_tree_view_controller_default = controller63({
  scope: () => {
    const nodes = signal62([]);
    return { nodes };
  },
  mount(state, host) {
    const initNodes = host.nodes;
    if (initNodes != null)
      state.nodes.set(initNodes);
    Object.defineProperty(host, "nodes", {
      configurable: true,
      get: () => state.nodes(),
      set: (v) => state.nodes.set(v)
    });
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "tree");
  }
});

// packages/components/primitives/src/sui-tree-view/sui-tree-view.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_tree_view_default = component73("sui-tree-view", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  shadow: true,
  styles: `
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
  }
  ul { list-style: none; margin: 0; padding: 0; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_tree_view_controller_default && typeof sui_tree_view_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_tree_view_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b1 = (anchor) => bindFor6({
      anchor,
      items: () => $scope.nodes(),
      key: (n, __i) => {
        const __child = Object.create($scope);
        __child.n = n;
        __child.__i = __i;
        return (($scope2) => n.id)(__child);
      },
      render: (n, __i) => {
        const __childScope5 = Object.create($scope);
        __childScope5.n = n;
        __childScope5.__i = __i;
        return (($scope2) => {
          const __frag6 = document.createDocumentFragment();
          const __c7 = [];
          const __e8 = document.createElement("sui-tree-item");
          __e8["node"] = $scope2.n;
          __frag6.appendChild(__e8);
          return { nodes: Array.from(__frag6.childNodes), cleanup: () => {
            for (const c of __c7)
              try {
                c();
              } catch (e) {}
          }, update: (__new9, __newI10) => {
            $scope2.n = __new9;
            $scope2.__i = __newI10;
            __e8["node"] = $scope2.n;
          } };
        })(__childScope5);
      }
    });
    const __wireBoundary_b0 = (anchor) => bindIf24({
      anchor,
      test: () => {
        if ($scope.nodes().length > 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("ul");
          __e3.setAttribute("role", "tree");
          const __forAnchor4 = document.createComment("for");
          __e3.appendChild(__forAnchor4);
          __c2.push(__wireBoundary_b1(__forAnchor4));
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag11 = document.createDocumentFragment();
          const __c12 = [];
          const __e13 = document.createElement("slot");
          __frag11.appendChild(__e13);
          return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
            for (const c of __c12)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets73(__root);
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

// packages/components/primitives/src/sui-tree-view/sui-tree-item.sui
import { attachRef as attachRef45, bindFor as bindFor7, bindIf as bindIf25, bindProp as bindProp4, bindText as bindText19, signal as signal65, view as view11 } from "@ship/sui-runtime";

// packages/components/primitives/src/sui-tree-view/sui-tree-item.controller.ts
import { controller as controller64, signal as signal64, derived as derived7, effect as effect55, ref as ref6 } from "@ship/sui-runtime";
var sui_tree_item_controller_default = controller64({
  scope: ({ label }) => {
    const node = signal64(null);
    const nodeLabel = derived7(() => {
      const n = node();
      return n ? n.label : label();
    });
    const nodeChildren = derived7(() => {
      const n = node();
      return n?.children ?? [];
    });
    const rowEl = ref6(null);
    return { node, nodeLabel, nodeChildren, rowEl };
  },
  mount(state, host) {
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "treeitem");
    const initNode = host.node;
    if (initNode != null)
      state.node.set(initNode);
    Object.defineProperty(host, "node", {
      configurable: true,
      get: () => state.node(),
      set: (v) => state.node.set(v)
    });
    const stopLeafSync = effect55(() => {
      const n = state.node();
      if (n !== null) {
        const isLeaf = !n.children || n.children.length === 0;
        if (isLeaf)
          host.setAttribute("leaf", "");
        else
          host.removeAttribute("leaf");
      }
    });
    const syncAria = effect55(() => {
      const row = state.rowEl();
      if (!row)
        return;
      if (!state.leaf()) {
        row.setAttribute("aria-expanded", String(state.expanded()));
      } else {
        row.removeAttribute("aria-expanded");
      }
      row.setAttribute("aria-selected", String(state.selected()));
    });
    const toggle = (e) => {
      if (state.leaf())
        return;
      e.stopPropagation();
      const next = !state.expanded();
      state.expanded.set(next);
      host.dispatchEvent(new CustomEvent("sui-toggle", {
        detail: { expanded: next },
        bubbles: true,
        composed: true
      }));
    };
    const select = () => {
      host.dispatchEvent(new CustomEvent("sui-select", {
        bubbles: true,
        composed: true,
        detail: {}
      }));
    };
    const onRowClick = (e) => {
      select();
      toggle(e);
    };
    const onKey = (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        select();
      } else if (e.key === "ArrowRight" && !state.leaf() && !state.expanded()) {
        e.preventDefault();
        toggle(e);
      } else if (e.key === "ArrowLeft" && state.expanded()) {
        e.preventDefault();
        toggle(e);
      }
    };
    const stopRow = effect55(() => {
      const row = state.rowEl();
      if (!row)
        return;
      row.addEventListener("click", onRowClick);
      row.addEventListener("keydown", onKey);
      return () => {
        row.removeEventListener("click", onRowClick);
        row.removeEventListener("keydown", onKey);
      };
    });
    return () => {
      stopLeafSync();
      syncAria();
      stopRow();
    };
  }
});

// packages/components/primitives/src/sui-tree-view/sui-tree-item.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_tree_item_default = Object.assign(view11((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  $scope["rowEl"] = signal65(null);
  if (sui_tree_item_controller_default && typeof sui_tree_item_controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = sui_tree_item_controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b3 = (anchor) => bindFor7({
    anchor,
    items: () => $scope.nodeChildren(),
    key: (c, __i) => {
      const __child = Object.create($scope);
      __child.c = c;
      __child.__i = __i;
      return (($scope2) => c.id)(__child);
    },
    render: (c, __i) => {
      const __childScope14 = Object.create($scope);
      __childScope14.c = c;
      __childScope14.__i = __i;
      return (($scope2) => {
        const __frag15 = document.createDocumentFragment();
        const __c16 = [];
        const __e17 = document.createElement("sui-tree-item");
        __e17["node"] = $scope2.c;
        __frag15.appendChild(__e17);
        return { nodes: Array.from(__frag15.childNodes), cleanup: () => {
          for (const c2 of __c16)
            try {
              c2();
            } catch (e) {}
        }, update: (__new18, __newI19) => {
          $scope2.c = __new18;
          $scope2.__i = __newI19;
          __e17["node"] = $scope2.c;
        } };
      })(__childScope14);
    }
  });
  const __wireBoundary_b2 = (anchor) => bindIf25({
    anchor,
    test: () => {
      if ($scope.nodeChildren().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag11 = document.createDocumentFragment();
        const __c12 = [];
        const __forAnchor13 = document.createComment("for");
        __frag11.appendChild(__forAnchor13);
        __c12.push(__wireBoundary_b3(__forAnchor13));
        return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
          for (const c of __c12)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag20 = document.createDocumentFragment();
        const __c21 = [];
        const __e22 = document.createElement("slot");
        __frag20.appendChild(__e22);
        return { nodes: Array.from(__frag20.childNodes), cleanup: () => {
          for (const c of __c21)
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
      __e0.setAttribute("class", "row");
      __e0.setAttribute("part", "row");
      __e0.setAttribute("tabindex", "0");
      attachRef45($scope, "rowEl", __e0);
      const __e1 = document.createElement("span");
      __e1.setAttribute("class", "caret");
      __e1.setAttribute("part", "caret");
      __e1.setAttribute("aria-hidden", "true");
      const __e2 = document.createElementNS("http://www.w3.org/2000/svg", "svg");
      __e2.setAttribute("viewBox", "0 0 12 12");
      __e2.setAttribute("fill", "none");
      __e2.setAttribute("stroke", "currentColor");
      __e2.setAttribute("stroke-width", "1.5");
      const __e3 = document.createElementNS("http://www.w3.org/2000/svg", "polyline");
      __e3.setAttribute("points", "4 3 8 6 4 9");
      __e2.appendChild(__e3);
      __e1.appendChild(__e2);
      __e0.appendChild(__e1);
      const __e4 = document.createElement("slot");
      __e4.setAttribute("name", "icon");
      __e0.appendChild(__e4);
      const __e5 = document.createElement("span");
      __e5.setAttribute("class", "label");
      __e5.setAttribute("part", "label");
      const __t6 = document.createTextNode("");
      __e5.appendChild(__t6);
      __cleanups.push(bindText19(__t6, () => $scope.nodeLabel()));
      __e0.appendChild(__e5);
      const __e7 = document.createElement("span");
      __e7.setAttribute("class", "trailing");
      const __e8 = document.createElement("slot");
      __e8.setAttribute("name", "trailing");
      __e7.appendChild(__e8);
      __e0.appendChild(__e7);
      container.appendChild(__e0);
      const __e9 = document.createElement("div");
      __e9.setAttribute("class", "children");
      __e9.setAttribute("part", "children");
      __e9.setAttribute("role", "group");
      const __ifAnchor10 = document.createComment("if");
      __e9.appendChild(__ifAnchor10);
      __cleanups.push(__wireBoundary_b2(__ifAnchor10));
      container.appendChild(__e9);
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
        __cleanups.push(hydrateRef(__m["b0"], $scope, "rowEl"));
      if (__t["b1"])
        __cleanups.push(hydrateText(__t["b1"], () => $scope.nodeLabel()));
      if (__m["b2"]) {
        const __t2 = __m["b2"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b2(__anchor));
      }
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" } });

// packages/ui-components/src/sui-graph-canvas/sui-graph-canvas.sui
import { adoptSuiSheets as adoptSuiSheets74, attachRef as attachRef46, component as component74, prop as prop72, signal as signal66 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-graph-canvas/sui-graph-canvas.controller.ts
import { controller as controller65, bindHtml as bindHtml5 } from "@ship/sui-runtime";
function esc(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}
function coerce(v, fallback) {
  if (Array.isArray(v))
    return v;
  if (typeof v === "string" && v.trim()) {
    try {
      const parsed = JSON.parse(v);
      return Array.isArray(parsed) ? parsed : fallback;
    } catch {
      return fallback;
    }
  }
  return fallback;
}
function computePositions(nodes, width, height, layout) {
  const map = new Map;
  const missing = nodes.some((n) => typeof n.x !== "number" || typeof n.y !== "number");
  if (layout === "positioned" && !missing) {
    for (const n of nodes)
      map.set(n.id, { x: n.x, y: n.y });
    return map;
  }
  const cx = width * 0.22;
  const cy = height * 0.5;
  const centers = nodes.filter((n) => n.kind === "agent");
  const satellites = nodes.filter((n) => n.kind !== "agent");
  centers.forEach((n, i) => {
    const span = Math.max(1, centers.length);
    const y = cy + (i - (span - 1) / 2) * 80;
    map.set(n.id, { x: cx, y });
  });
  const byKind = new Map;
  for (const n of satellites) {
    const list = byKind.get(n.kind) ?? [];
    list.push(n);
    byKind.set(n.kind, list);
  }
  const kinds = Array.from(byKind.keys());
  kinds.forEach((kind, bandIdx) => {
    const list = byKind.get(kind);
    const bandX = width * (0.5 + (bandIdx + 1) / (kinds.length + 1) * 0.45);
    list.forEach((n, i) => {
      const y = height * (i + 1) / (list.length + 1);
      map.set(n.id, { x: bandX, y });
    });
  });
  return map;
}
function bezierPath(a, b) {
  const dx = Math.max(40, Math.abs(b.x - a.x) * 0.4);
  return `M ${a.x} ${a.y} C ${a.x + dx} ${a.y}, ${b.x - dx} ${b.y}, ${b.x} ${b.y}`;
}
function kindTemplateFor(host, kind) {
  const tpls = host.querySelectorAll('template[slot="node"]');
  for (const t of Array.from(tpls)) {
    if (t.dataset.kind === kind)
      return t;
  }
  return null;
}
function nodeBodyHtml(host, n) {
  const tpl = kindTemplateFor(host, n.kind);
  if (!tpl) {
    const sub = n.sub ? `<div class="node-sub">${esc(n.sub)}</div>` : "";
    return `<div class="node-h"><span class="tag">${esc(n.kind)}</span></div><div class="node-title">${esc(n.title ?? n.id)}</div>${sub}`;
  }
  return tpl.innerHTML.replace(/\$\{title\}/g, esc(n.title ?? "")).replace(/\$\{sub\}/g, esc(n.sub ?? "")).replace(/\$\{id\}/g, esc(n.id));
}
function renderGraphHtml(host, nodesRaw, edgesRaw, width, height, layout, selectedId) {
  const nodes = coerce(nodesRaw, []);
  const edges = coerce(edgesRaw, []);
  const pos = computePositions(nodes, width, height, layout);
  const edgePaths = edges.map((e) => {
    const a = pos.get(e.from);
    const b = pos.get(e.to);
    if (!a || !b)
      return "";
    const d = esc(bezierPath(a, b));
    const dashed = e.dashed ? " data-dashed" : "";
    return `<path d="${d}" data-kind="${esc(e.kind ?? "default")}"${dashed}></path>`;
  }).join("");
  const nodeDivs = nodes.map((n) => {
    const p = pos.get(n.id);
    if (!p)
      return "";
    const left = p.x / width * 100;
    const top = p.y / height * 100;
    const sel = selectedId && selectedId === n.id ? " data-selected" : "";
    const wStyle = n.w ? `--node-w: ${n.w}px;` : "";
    const label = esc(`${n.kind}: ${n.title ?? n.id}`);
    const body = nodeBodyHtml(host, n);
    return `<div class="node" part="node" data-kind="${esc(n.kind)}" data-id="${esc(n.id)}"${sel} tabindex="0" role="button" aria-label="${label}" style="left: ${left}%; top: ${top}%;${wStyle}">${body}</div>`;
  }).join("");
  return `<svg class="edges" part="edges" viewBox="0 0 ${width} ${height}" preserveAspectRatio="none" aria-hidden="true">${edgePaths}</svg>` + nodeDivs;
}
var sui_graph_canvas_controller_default = controller65({
  mount(state, host) {
    const scope = state;
    const canvas = scope.canvasEl();
    if (!canvas)
      return;
    const inner = canvas.ownerDocument.createElement("div");
    canvas.appendChild(inner);
    const stopRender = bindHtml5(inner, () => {
      const nodesRaw = scope.nodes();
      const edgesRaw = scope.edges();
      const width = scope.width() || 1000;
      const height = scope.height() || 600;
      const layout = scope.layout() || "positioned";
      const selectedId = scope.selectedId();
      return renderGraphHtml(host, nodesRaw, edgesRaw, width, height, layout, selectedId);
    });
    let lastHoveredId = "";
    const onClick = (e) => {
      const node = e.target.closest(".node[data-id]");
      if (!node)
        return;
      const id = node.getAttribute("data-id") ?? "";
      const kind = node.getAttribute("data-kind") ?? "";
      host.setAttribute("selected-id", id);
      host.dispatchEvent(new CustomEvent("sui-node-select", {
        detail: { id, kind },
        bubbles: true,
        composed: true
      }));
    };
    const onKeyDown = (e) => {
      if (e.key !== "Enter" && e.key !== " ")
        return;
      const node = e.target.closest(".node[data-id]");
      if (!node)
        return;
      e.preventDefault();
      const id = node.getAttribute("data-id") ?? "";
      const kind = node.getAttribute("data-kind") ?? "";
      host.setAttribute("selected-id", id);
      host.dispatchEvent(new CustomEvent("sui-node-select", {
        detail: { id, kind },
        bubbles: true,
        composed: true
      }));
    };
    const onMouseOver = (e) => {
      const node = e.target.closest(".node[data-id]");
      if (!node) {
        lastHoveredId = "";
        return;
      }
      const id = node.getAttribute("data-id") ?? "";
      if (id === lastHoveredId)
        return;
      lastHoveredId = id;
      const kind = node.getAttribute("data-kind") ?? "";
      host.dispatchEvent(new CustomEvent("sui-node-hover", {
        detail: { id, kind },
        bubbles: true,
        composed: true
      }));
    };
    canvas.addEventListener("click", onClick);
    canvas.addEventListener("keydown", onKeyDown);
    canvas.addEventListener("mouseover", onMouseOver);
    return () => {
      stopRender();
      canvas.removeEventListener("click", onClick);
      canvas.removeEventListener("keydown", onKeyDown);
      canvas.removeEventListener("mouseover", onMouseOver);
    };
  }
});

// packages/ui-components/src/sui-graph-canvas/sui-graph-canvas.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_graph_canvas_default = component74("sui-graph-canvas", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    nodes: prop72.string({ default: "" }),
    edges: prop72.string({ default: "" }),
    width: prop72.number({ default: 1000, reflect: true }),
    height: prop72.number({ default: 600, reflect: true }),
    layout: prop72.union(["positioned", "radial"], { default: "positioned", reflect: true }),
    "selected-id": prop72.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    position: relative;
    width: 100%;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
    /* Component knobs — consumers may override on the host. */
    --graph-aspect: 5 / 3;
    --node-w: 180px;
    --edge-default: var(--border);
    --edge-agent: var(--tone-primary-fg, var(--foreground));
    --edge-skill: var(--tone-success-fg);
    --edge-mcp: var(--tone-info-fg);
    --edge-rule: var(--tone-warning-fg);
    --edge-perm: var(--tone-destructive-fg);
    --edge-harness: var(--tone-accent-fg);
  }

  .canvas {
    position: relative;
    width: 100%;
    aspect-ratio: var(--graph-aspect, 5 / 3);
  }

  svg.edges {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
  svg.edges path {
    fill: none;
    stroke: var(--edge-default);
    stroke-width: 1.5;
    opacity: 0.85;
  }
  svg.edges path[data-kind='agent']    { stroke: var(--edge-agent); }
  svg.edges path[data-kind='skill']    { stroke: var(--edge-skill); }
  svg.edges path[data-kind='mcp']      { stroke: var(--edge-mcp); }
  svg.edges path[data-kind='rule']     { stroke: var(--edge-rule); }
  svg.edges path[data-kind='perm']     { stroke: var(--edge-perm); }
  svg.edges path[data-kind='harness'] { stroke: var(--edge-harness); }
  svg.edges path[data-dashed] { stroke-dasharray: 4 3; }

  .node {
    position: absolute;
    transform: translate(-50%, -50%);
    width: var(--node-w, 180px);
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-2) var(--space-3);
    box-shadow: var(--shadow-xs);
    cursor: pointer;
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    transition: box-shadow var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }
  .node:hover { box-shadow: var(--shadow-sm); }
  .node:focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: 2px;
  }
  .node[data-selected] {
    border-color: var(--ring);
    box-shadow: var(--shadow-sm);
  }
  .node[data-kind='agent']    { border-left: 3px solid var(--edge-agent); }
  .node[data-kind='skill']    { border-left: 3px solid var(--edge-skill); }
  .node[data-kind='mcp']      { border-left: 3px solid var(--edge-mcp); }
  .node[data-kind='rule']     { border-left: 3px solid var(--edge-rule); }
  .node[data-kind='perm']     { border-left: 3px solid var(--edge-perm); }
  .node[data-kind='harness'] { border-left: 3px solid var(--edge-harness); }

  .node-h {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }
  .tag {
    font-size: var(--text-xs);
    font-weight: var(--font-weight-semibold);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .node-title {
    font-weight: var(--font-weight-semibold);
    color: var(--foreground);
  }
  .node-sub {
    font-size: var(--text-xs);
    color: var(--muted-foreground);
  }

  ::slotted(template) { display: none; }

  @media (prefers-reduced-motion: reduce) {
    .node { transition: none; }
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.nodes = props.nodes;
    $scope.edges = props.edges;
    $scope.width = props.width;
    $scope.height = props.height;
    $scope.layout = props.layout;
    $scope.selectedId = props.selectedId;
    $scope["canvasEl"] = signal66(null);
    let __r = null;
    if (sui_graph_canvas_controller_default && typeof sui_graph_canvas_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_graph_canvas_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets74(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "canvas");
    __e0.setAttribute("part", "canvas");
    attachRef46($scope, "canvasEl", __e0);
    __root.appendChild(__e0);
    const __e1 = document.createElement("slot");
    __e1.setAttribute("name", "node");
    __e1.setAttribute("hidden", "");
    __root.appendChild(__e1);
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

// packages/ui-components/src/sui-pipeline-dag/sui-pipeline-dag.sui
import { adoptSuiSheets as adoptSuiSheets75, component as component75 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-pipeline-dag/sui-pipeline-dag.controller.ts
import { controller as controller66 } from "@ship/sui-runtime";
var sui_pipeline_dag_controller_default = controller66({
  mount(_state, host) {
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "list");
    if (!host.hasAttribute("aria-label")) {
      host.setAttribute("aria-label", "pipeline steps");
    }
  }
});

// packages/ui-components/src/sui-pipeline-dag/sui-pipeline-dag.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_pipeline_dag_default = component75("sui-pipeline-dag", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  shadow: true,
  styles: `
  :host {
    display: flex;
    align-items: stretch;
    gap: 0;
    overflow-x: auto;
    padding: 4px 0 2px;
  }
  ::slotted(.arrow) {
    display: flex;
    align-items: center;
    padding: 0 6px;
    color: var(--border);
    font-size: 14px;
    flex-shrink: 0;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_pipeline_dag_controller_default && typeof sui_pipeline_dag_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_pipeline_dag_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets75(__root);
    const __e0 = document.createElement("slot");
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

// packages/ui-components/src/sui-pipeline-step/sui-pipeline-step.sui
import { adoptSuiSheets as adoptSuiSheets76, bindIf as bindIf26, bindText as bindText20, component as component76, prop as prop73 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-pipeline-step/sui-pipeline-step.controller.ts
import { controller as controller67, effect as effect56 } from "@ship/sui-runtime";
var sui_pipeline_step_controller_default = controller67({
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "listitem");
    const stopLabel = effect56(() => {
      const name = scope.name();
      const type = scope.stepType();
      const agent = scope.agent();
      const s = scope.state();
      const summary = `${type} step ${name}${agent ? ` (${agent})` : ""}: ${s}`;
      host.setAttribute("aria-label", summary);
    });
    return () => {
      stopLabel();
    };
  }
});

// packages/ui-components/src/sui-pipeline-step/sui-pipeline-step.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_pipeline_step_default = component76("sui-pipeline-step", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    name: prop73.string({ default: "", reflect: true }),
    "step-type": prop73.union(["agent", "gate", "merge", "run", "human"], { default: "agent", reflect: true }),
    state: prop73.union(["pending", "running", "completed", "failed"], { default: "pending", reflect: true }),
    agent: prop73.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    gap: 4px;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 14px;
    min-width: 130px;
    background: var(--background);
    position: relative;
    font-family: var(--font-body);
  }
  .icon {
    position: absolute;
    top: 10px;
    right: 12px;
    font-size: 10px;
    color: var(--muted-foreground);
    line-height: 1;
  }
  :host([state='completed']) {
    border-color: var(--tone-success-border);
    background: var(--tone-success-bg);
  }
  :host([state='completed']) .icon { color: var(--success); }
  :host([state='running']) {
    border-color: var(--tone-info-border);
    background: var(--tone-info-bg);
    box-shadow: 0 0 0 3px var(--tone-info-ring);
  }
  :host([state='running']) .icon { color: var(--info); }
  :host([state='failed']) {
    border-color: var(--tone-destructive-border);
    background: var(--tone-destructive-bg);
  }
  :host([state='failed']) .icon { color: var(--destructive); }

  .name {
    font-family: var(--font-mono);
    font-size: 11.5px;
    font-weight: 600;
    padding-right: 16px;
    color: var(--foreground);
  }
  .type {
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--muted-foreground);
  }
  .status {
    font-size: 10px;
    color: var(--muted-foreground);
  }
  sui-spinner {
    position: absolute;
    top: 10px;
    right: 12px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.name = props.name;
    $scope.stepType = props.stepType;
    $scope.state = props.state;
    $scope.agent = props.agent;
    let __r = null;
    if (sui_pipeline_step_controller_default && typeof sui_pipeline_step_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_pipeline_step_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b0 = (anchor) => bindIf26({
      anchor,
      test: () => {
        if ($scope.state() === "running")
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("sui-spinner");
          __e3.setAttribute("size", "sm");
          __e3.setAttribute("tone", "info");
          __e3.setAttribute("label", "Running");
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag4 = document.createDocumentFragment();
          const __c5 = [];
          const __e6 = document.createElement("span");
          __e6.setAttribute("class", "icon");
          __e6.setAttribute("aria-hidden", "true");
          const __t7 = document.createTextNode("");
          __e6.appendChild(__t7);
          __c5.push(bindText20(__t7, () => $scope.state() === "completed" ? "✓" : $scope.state() === "failed" ? "✗" : "○"));
          __frag4.appendChild(__e6);
          return { nodes: Array.from(__frag4.childNodes), cleanup: () => {
            for (const c of __c5)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b4 = (anchor) => bindIf26({
      anchor,
      test: () => {
        if ($scope.agent())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag13 = document.createDocumentFragment();
          const __c14 = [];
          const __t15 = document.createTextNode(" · ");
          __frag13.appendChild(__t15);
          const __t16 = document.createTextNode("");
          __frag13.appendChild(__t16);
          __c14.push(bindText20(__t16, () => $scope.agent()));
          return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
            for (const c of __c14)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets76(__root);
    const __ifAnchor0 = document.createComment("if");
    __root.appendChild(__ifAnchor0);
    __cleanups.push(__wireBoundary_b0(__ifAnchor0));
    const __e8 = document.createElement("span");
    __e8.setAttribute("class", "name");
    const __t9 = document.createTextNode("");
    __e8.appendChild(__t9);
    __cleanups.push(bindText20(__t9, () => $scope.name()));
    __root.appendChild(__e8);
    const __e10 = document.createElement("span");
    __e10.setAttribute("class", "type");
    const __t11 = document.createTextNode("");
    __e10.appendChild(__t11);
    __cleanups.push(bindText20(__t11, () => $scope.stepType()));
    const __ifAnchor12 = document.createComment("if");
    __e10.appendChild(__ifAnchor12);
    __cleanups.push(__wireBoundary_b4(__ifAnchor12));
    __root.appendChild(__e10);
    const __e17 = document.createElement("span");
    __e17.setAttribute("class", "status");
    const __t18 = document.createTextNode("");
    __e17.appendChild(__t18);
    __cleanups.push(bindText20(__t18, () => $scope.state()));
    __root.appendChild(__e17);
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

// packages/ui-components/src/sui-pipeline-run/sui-pipeline-run.sui
import { adoptSuiSheets as adoptSuiSheets77, bindAttr as bindAttr42, bindIf as bindIf27, bindText as bindText21, component as component77, prop as prop74 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-pipeline-run/sui-pipeline-run.controller.ts
import { controller as controller68, derived as derived8, effect as effect57 } from "@ship/sui-runtime";
var STATE_TO_TONE = {
  dispatched: "info",
  running: "info",
  gatepending: "warning",
  completed: "success",
  failed: "destructive",
  merged: "accent",
  pending: "muted"
};
var sui_pipeline_run_controller_default = controller68({
  scope: (state) => {
    const s = state;
    const tone = derived8(() => STATE_TO_TONE[s.state()] ?? "muted");
    return { tone };
  },
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "article");
    const stopLabel = effect57(() => {
      const slug = scope.slug();
      const agent = scope.agent();
      const s = scope.state();
      host.setAttribute("aria-label", `Pipeline run ${slug || ""}${agent ? ` on ${agent}` : ""}: ${s}`);
    });
    return () => {
      stopLabel();
    };
  }
});

// packages/ui-components/src/sui-pipeline-run/sui-pipeline-run.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_pipeline_run_default = component77("sui-pipeline-run", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    state: prop74.union(["dispatched", "gatepending", "running", "completed", "failed", "merged", "pending"], { default: "pending", reflect: true }),
    slug: prop74.string({ default: "", reflect: true }),
    agent: prop74.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: block; }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    gap: 12px;
    flex-wrap: wrap;
  }
  .left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }
  .slug {
    font-family: var(--font-mono);
    font-size: 12.5px;
    font-weight: 600;
    color: var(--foreground);
  }
  .agent { font-size: 11px; color: var(--muted-foreground); }
  .agent code {
    background: var(--muted);
    padding: 1px 6px;
    border-radius: 4px;
    font-size: 10.5px;
    font-family: var(--font-mono);
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.state = props.state;
    $scope.slug = props.slug;
    $scope.agent = props.agent;
    let __r = null;
    if (sui_pipeline_run_controller_default && typeof sui_pipeline_run_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_pipeline_run_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindIf27({
      anchor,
      test: () => {
        if ($scope.agent())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag7 = document.createDocumentFragment();
          const __c8 = [];
          const __e9 = document.createElement("span");
          __e9.setAttribute("class", "agent");
          const __e10 = document.createElement("code");
          const __t11 = document.createTextNode("");
          __e10.appendChild(__t11);
          __c8.push(bindText21(__t11, () => $scope.agent()));
          __e9.appendChild(__e10);
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
    adoptSuiSheets77(__root);
    const __e0 = document.createElement("sui-card");
    __e0.setAttribute("status-rail", "");
    __cleanups.push(bindAttr42(__e0, "tone", () => $scope.tone()));
    const __e1 = document.createElement("div");
    __e1.setAttribute("slot", "header");
    const __e2 = document.createElement("header");
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "left");
    const __e4 = document.createElement("span");
    __e4.setAttribute("class", "slug");
    const __t5 = document.createTextNode("");
    __e4.appendChild(__t5);
    __cleanups.push(bindText21(__t5, () => $scope.slug()));
    __e3.appendChild(__e4);
    const __ifAnchor6 = document.createComment("if");
    __e3.appendChild(__ifAnchor6);
    __cleanups.push(__wireBoundary_b2(__ifAnchor6));
    const __e12 = document.createElement("slot");
    __e12.setAttribute("name", "actions");
    __e3.appendChild(__e12);
    __e2.appendChild(__e3);
    const __e13 = document.createElement("sui-badge");
    __cleanups.push(bindAttr42(__e13, "tone", () => $scope.tone()));
    const __t14 = document.createTextNode("");
    __e13.appendChild(__t14);
    __cleanups.push(bindText21(__t14, () => $scope.state()));
    __e2.appendChild(__e13);
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e15 = document.createElement("slot");
    __e0.appendChild(__e15);
    const __e16 = document.createElement("div");
    __e16.setAttribute("slot", "footer");
    const __e17 = document.createElement("slot");
    __e17.setAttribute("name", "footer");
    __e16.appendChild(__e17);
    __e0.appendChild(__e16);
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

// packages/ui-components/src/sui-pipeline-step-def/sui-pipeline-step-def.sui
import { adoptSuiSheets as adoptSuiSheets78, bindAttr as bindAttr43, bindFor as bindFor8, bindIf as bindIf28, bindText as bindText22, component as component78, prop as prop75, signal as signal67 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-pipeline-step-def/sui-pipeline-step-def.controller.ts
import { controller as controller69, derived as derived9 } from "@ship/sui-runtime";
var TYPE_TONE = {
  run: "muted",
  agent: "info",
  gate: "warning",
  merge: "accent",
  human: "warning"
};
var sui_pipeline_step_def_controller_default = controller69({
  scope: (state) => {
    const s = state;
    const kindTone = derived9(() => TYPE_TONE[s.stepType()] ?? "muted");
    const afterList = derived9(() => (s.after() || "").split(",").map((x) => x.trim()).filter(Boolean));
    const commandTrim = derived9(() => (s.command() || "").trim());
    return { kindTone, afterList, commandTrim };
  }
});

// packages/ui-components/src/sui-pipeline-step-def/sui-pipeline-step-def.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_pipeline_step_def_default = component78("sui-pipeline-step-def", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    name: prop75.string({ default: "", reflect: true }),
    "step-type": prop75.union(["agent", "gate", "run", "merge", "human"], { default: "agent", reflect: true }),
    agent: prop75.string({ default: "", reflect: true }),
    harness: prop75.string({ default: "", reflect: true }),
    model: prop75.string({ default: "", reflect: true }),
    after: prop75.string({ default: "", reflect: true }),
    command: prop75.string({ default: "" }),
    index: prop75.number({ default: 0, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    padding: 16px 0;
    border-top: 1px solid var(--border-subtle);
  }
  :host(:first-of-type), :host([first]) { border-top: none; padding-top: 0; }
  :host(:last-of-type) { padding-bottom: 0; }

  [part='row'] {
    display: grid;
    grid-template-columns: 2ch 1fr;
    gap: 16px;
  }
  .index {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    padding-top: 3px;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }
  [part='header'] {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .name {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--foreground);
  }
  .after {
    font-size: var(--text-xs);
    color: var(--muted-foreground);
  }
  .after code {
    font-family: var(--font-mono);
    background: color-mix(in oklch, var(--muted) 40%, transparent);
    padding: 1px 5px;
    border-radius: 4px;
    font-size: var(--text-xs);
  }
  .chips { display: flex; gap: 6px; flex-wrap: wrap; }
  [part='command'] {
    margin: 0;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    background: color-mix(in oklch, var(--muted) 35%, transparent);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.55;
    color: var(--foreground);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-x: auto;
  }
  .trailing { margin-left: auto; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.name = props.name;
    $scope.stepType = props.stepType;
    $scope.agent = props.agent;
    $scope.harness = props.harness;
    $scope.model = props.model;
    $scope.after = props.after;
    $scope.command = props.command;
    $scope.index = props.index;
    let __r = null;
    if (sui_pipeline_step_def_controller_default && typeof sui_pipeline_step_def_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_pipeline_step_def_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b5 = (anchor) => bindFor8({
      anchor,
      items: () => $scope.afterList(),
      key: (a, idx) => {
        const __child = Object.create($scope);
        __child.a = a;
        __child.idx = idx;
        return (($scope2) => a)(__child);
      },
      render: (a, idx) => {
        const __childScope15 = Object.create($scope);
        __childScope15.a = a;
        __childScope15.idx = idx;
        return (($scope2) => {
          const __wireBoundary_b6 = (anchor2) => bindIf28({
            anchor: anchor2,
            test: () => {
              if ($scope2.idx > 0)
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag19 = document.createDocumentFragment();
                const __c20 = [];
                const __t21 = document.createTextNode(", ");
                __frag19.appendChild(__t21);
                return { nodes: Array.from(__frag19.childNodes), cleanup: () => {
                  for (const c of __c20)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __frag16 = document.createDocumentFragment();
          const __c17 = [];
          const __ifAnchor18 = document.createComment("if");
          __frag16.appendChild(__ifAnchor18);
          __c17.push(__wireBoundary_b6(__ifAnchor18));
          const __e22 = document.createElement("code");
          const __t23 = document.createTextNode("");
          __e22.appendChild(__t23);
          __t23.textContent = $scope2.a;
          __frag16.appendChild(__e22);
          return { nodes: Array.from(__frag16.childNodes), cleanup: () => {
            for (const c of __c17)
              try {
                c();
              } catch (e) {}
          }, update: (__new24, __newI25) => {
            $scope2.a = __new24;
            $scope2.idx = __newI25;
            __t23.textContent = $scope2.a;
          } };
        })(__childScope15);
      }
    });
    const __wireBoundary_b4 = (anchor) => bindIf28({
      anchor,
      test: () => {
        if ($scope.afterList().length > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag10 = document.createDocumentFragment();
          const __c11 = [];
          const __e12 = document.createElement("span");
          __e12.setAttribute("class", "after");
          const __t13 = document.createTextNode("after ");
          __e12.appendChild(__t13);
          const __forAnchor14 = document.createComment("for");
          __e12.appendChild(__forAnchor14);
          __c11.push(__wireBoundary_b5(__forAnchor14));
          __frag10.appendChild(__e12);
          return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
            for (const c of __c11)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b9 = (anchor) => bindIf28({
      anchor,
      test: () => {
        if ($scope.agent())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag33 = document.createDocumentFragment();
          const __c34 = [];
          const __e35 = document.createElement("sui-chip");
          __e35.setAttribute("size", "sm");
          const __t36 = document.createTextNode("");
          __e35.appendChild(__t36);
          __c34.push(bindText22(__t36, () => $scope.agent()));
          __frag33.appendChild(__e35);
          return { nodes: Array.from(__frag33.childNodes), cleanup: () => {
            for (const c of __c34)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b11 = (anchor) => bindIf28({
      anchor,
      test: () => {
        if ($scope.harness())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag38 = document.createDocumentFragment();
          const __c39 = [];
          const __e40 = document.createElement("sui-chip");
          __e40.setAttribute("size", "sm");
          const __t41 = document.createTextNode("");
          __e40.appendChild(__t41);
          __c39.push(bindText22(__t41, () => $scope.harness()));
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
    const __wireBoundary_b13 = (anchor) => bindIf28({
      anchor,
      test: () => {
        if ($scope.model())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag43 = document.createDocumentFragment();
          const __c44 = [];
          const __e45 = document.createElement("sui-chip");
          __e45.setAttribute("size", "sm");
          const __t46 = document.createTextNode("");
          __e45.appendChild(__t46);
          __c44.push(bindText22(__t46, () => $scope.model()));
          __frag43.appendChild(__e45);
          return { nodes: Array.from(__frag43.childNodes), cleanup: () => {
            for (const c of __c44)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b8 = (anchor) => bindIf28({
      anchor,
      test: () => {
        if ($scope.agent() || $scope.harness() || $scope.model())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag29 = document.createDocumentFragment();
          const __c30 = [];
          const __e31 = document.createElement("div");
          __e31.setAttribute("class", "chips");
          const __ifAnchor32 = document.createComment("if");
          __e31.appendChild(__ifAnchor32);
          __c30.push(__wireBoundary_b9(__ifAnchor32));
          const __ifAnchor37 = document.createComment("if");
          __e31.appendChild(__ifAnchor37);
          __c30.push(__wireBoundary_b11(__ifAnchor37));
          const __ifAnchor42 = document.createComment("if");
          __e31.appendChild(__ifAnchor42);
          __c30.push(__wireBoundary_b13(__ifAnchor42));
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
    const __wireBoundary_b15 = (anchor) => bindIf28({
      anchor,
      test: () => {
        if ($scope.commandTrim())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag48 = document.createDocumentFragment();
          const __c49 = [];
          const __e50 = document.createElement("pre");
          __e50.setAttribute("part", "command");
          const __t51 = document.createTextNode("");
          __e50.appendChild(__t51);
          __c49.push(bindText22(__t51, () => $scope.commandTrim()));
          __frag48.appendChild(__e50);
          return { nodes: Array.from(__frag48.childNodes), cleanup: () => {
            for (const c of __c49)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets78(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("part", "row");
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "index");
    const __t2 = document.createTextNode("");
    __e1.appendChild(__t2);
    __cleanups.push(bindText22(__t2, () => $scope.index() || ""));
    __e0.appendChild(__e1);
    const __e3 = document.createElement("div");
    __e3.setAttribute("class", "body");
    const __e4 = document.createElement("div");
    __e4.setAttribute("part", "header");
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "name");
    const __t6 = document.createTextNode("");
    __e5.appendChild(__t6);
    __cleanups.push(bindText22(__t6, () => $scope.name()));
    __e4.appendChild(__e5);
    const __e7 = document.createElement("sui-chip");
    __e7.setAttribute("size", "sm");
    __cleanups.push(bindAttr43(__e7, "tone", () => $scope.kindTone()));
    const __t8 = document.createTextNode("");
    __e7.appendChild(__t8);
    __cleanups.push(bindText22(__t8, () => $scope.stepType()));
    __e4.appendChild(__e7);
    const __ifAnchor9 = document.createComment("if");
    __e4.appendChild(__ifAnchor9);
    __cleanups.push(__wireBoundary_b4(__ifAnchor9));
    const __e26 = document.createElement("span");
    __e26.setAttribute("class", "trailing");
    const __e27 = document.createElement("slot");
    __e27.setAttribute("name", "trailing");
    __e26.appendChild(__e27);
    __e4.appendChild(__e26);
    __e3.appendChild(__e4);
    const __ifAnchor28 = document.createComment("if");
    __e3.appendChild(__ifAnchor28);
    __cleanups.push(__wireBoundary_b8(__ifAnchor28));
    const __ifAnchor47 = document.createComment("if");
    __e3.appendChild(__ifAnchor47);
    __cleanups.push(__wireBoundary_b15(__ifAnchor47));
    const __e52 = document.createElement("slot");
    __e52.setAttribute("name", "gate");
    __e3.appendChild(__e52);
    __e0.appendChild(__e3);
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

// packages/ui-components/src/sui-pipeline-definition/sui-pipeline-definition.sui
import { adoptSuiSheets as adoptSuiSheets79, bindIf as bindIf29, bindText as bindText23, component as component79, prop as prop76 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-pipeline-definition/sui-pipeline-definition.controller.ts
import { controller as controller70, derived as derived10, effect as effect58 } from "@ship/sui-runtime";
var sui_pipeline_definition_controller_default = controller70({
  scope: (state) => {
    const s = state;
    const stepsLabel = derived10(() => {
      const n = s.steps();
      return `${n} step${n === 1 ? "" : "s"}`;
    });
    return { stepsLabel };
  },
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "article");
    const stopLabel = effect58(() => {
      const name = scope.name();
      const steps = scope.steps();
      const desc = scope.description();
      const suffix = steps === 1 ? "" : "s";
      const tail = desc ? `: ${desc}` : "";
      host.setAttribute("aria-label", `Pipeline ${name}, ${steps} step${suffix}${tail}`);
    });
    return () => {
      stopLabel();
    };
  }
});

// packages/ui-components/src/sui-pipeline-definition/sui-pipeline-definition.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_pipeline_definition_default = component79("sui-pipeline-definition", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    name: prop76.string({ default: "", reflect: true }),
    steps: prop76.number({ default: 0, reflect: true }),
    description: prop76.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: block; }
  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 12px;
  }
  .name {
    font-family: var(--heading-font, var(--font-body));
    font-size: 15px;
    font-weight: 600;
    letter-spacing: -0.01em;
    color: var(--foreground);
  }
  .steps {
    font-size: 10.5px;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .desc {
    font-size: 12.5px;
    color: var(--muted-foreground);
    line-height: 1.5;
    min-height: 1.5em;
    margin: 6px 0 0;
  }
  .no-desc { opacity: 0.5; }
  sui-card { display: block; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.name = props.name;
    $scope.steps = props.steps;
    $scope.description = props.description;
    let __r = null;
    if (sui_pipeline_definition_controller_default && typeof sui_pipeline_definition_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_pipeline_definition_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindIf29({
      anchor,
      test: () => {
        if ($scope.description())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag9 = document.createDocumentFragment();
          const __c10 = [];
          const __e11 = document.createElement("p");
          __e11.setAttribute("class", "desc");
          const __t12 = document.createTextNode("");
          __e11.appendChild(__t12);
          __c10.push(bindText23(__t12, () => $scope.description()));
          __frag9.appendChild(__e11);
          return { nodes: Array.from(__frag9.childNodes), cleanup: () => {
            for (const c of __c10)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag13 = document.createDocumentFragment();
          const __c14 = [];
          const __e15 = document.createElement("p");
          __e15.setAttribute("class", "desc no-desc");
          const __t16 = document.createTextNode("No description");
          __e15.appendChild(__t16);
          __frag13.appendChild(__e15);
          return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
            for (const c of __c14)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets79(__root);
    const __e0 = document.createElement("sui-card");
    __e0.setAttribute("interactive", "");
    __e0.setAttribute("accent-bar", "");
    __e0.setAttribute("padding", "md");
    const __e1 = document.createElement("div");
    __e1.setAttribute("slot", "header");
    const __e2 = document.createElement("header");
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "name");
    const __t4 = document.createTextNode("");
    __e3.appendChild(__t4);
    __cleanups.push(bindText23(__t4, () => $scope.name()));
    __e2.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "steps");
    const __t6 = document.createTextNode("");
    __e5.appendChild(__t6);
    __cleanups.push(bindText23(__t6, () => $scope.stepsLabel()));
    __e2.appendChild(__e5);
    const __e7 = document.createElement("slot");
    __e7.setAttribute("name", "actions");
    __e2.appendChild(__e7);
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __ifAnchor8 = document.createComment("if");
    __e0.appendChild(__ifAnchor8);
    __cleanups.push(__wireBoundary_b2(__ifAnchor8));
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

// packages/ui-components/src/sui-log-stream/sui-log-stream.sui
import { adoptSuiSheets as adoptSuiSheets80, attachRef as attachRef47, component as component80, prop as prop77, signal as signal68 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-log-stream/sui-log-stream.controller.ts
import { controller as controller71, effect as effect59, ref as ref7 } from "@ship/sui-runtime";
var sui_log_stream_controller_default = controller71({
  scope: () => {
    const preEl = ref7(null);
    return { preEl };
  },
  mount(state, host) {
    const scope = state;
    let source = null;
    let buffer = "";
    let stickyBottom = true;
    const render = () => {
      const pre = scope.preEl();
      if (!pre)
        return;
      const empty = buffer.length === 0;
      if (empty) {
        pre.textContent = "(no output yet)";
        pre.setAttribute("data-empty", "");
      } else {
        pre.textContent = buffer;
        pre.removeAttribute("data-empty");
      }
      if (scope.autoscroll() && stickyBottom) {
        pre.scrollTop = pre.scrollHeight;
      }
    };
    const append = (chunk) => {
      buffer += chunk;
      const cap = scope.maxBytes();
      if (buffer.length > cap) {
        const overflow = buffer.length - cap;
        const cut = buffer.indexOf(`
`, overflow);
        buffer = buffer.slice(cut >= 0 ? cut + 1 : overflow);
      }
      render();
    };
    const close = () => {
      if (source) {
        source.close();
        source = null;
        host.dispatchEvent(new CustomEvent("sui-stream-close", { bubbles: true, composed: true }));
      }
    };
    const open = () => {
      const url = scope.url();
      if (!url || source)
        return;
      try {
        source = new EventSource(url, { withCredentials: true });
      } catch {
        host.dispatchEvent(new CustomEvent("sui-stream-error", { bubbles: true, composed: true }));
        return;
      }
      source.addEventListener(scope.event(), (ev) => {
        const chunk = ev.data ?? "";
        append(chunk);
        host.dispatchEvent(new CustomEvent("sui-log-tick", {
          detail: { bytes: chunk.length },
          bubbles: true,
          composed: true
        }));
      });
      source.onopen = () => {
        host.dispatchEvent(new CustomEvent("sui-stream-open", { bubbles: true, composed: true }));
      };
      source.onerror = () => {
        host.dispatchEvent(new CustomEvent("sui-stream-error", { bubbles: true, composed: true }));
      };
    };
    const onScroll = (e) => {
      const pre = e.currentTarget;
      stickyBottom = pre.scrollHeight - pre.clientHeight <= pre.scrollTop + 4;
    };
    const stopPreBind = effect59(() => {
      const pre = scope.preEl();
      if (!pre)
        return;
      pre.addEventListener("scroll", onScroll);
      render();
      return () => pre.removeEventListener("scroll", onScroll);
    });
    const stopActive = effect59(() => {
      scope.url();
      scope.event();
      const wantActive = scope.active();
      if (wantActive) {
        if (source) {
          close();
        }
        open();
      } else {
        close();
      }
    });
    host.clear = () => {
      buffer = "";
      render();
    };
    return () => {
      stopPreBind();
      stopActive();
      close();
    };
  }
});

// packages/ui-components/src/sui-log-stream/sui-log-stream.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_log_stream_default = component80("sui-log-stream", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    url: prop77.string({ default: "", reflect: true }),
    event: prop77.string({ default: "log", reflect: true }),
    autoscroll: prop77.bool({ default: false, reflect: true }),
    active: prop77.bool({ default: false, reflect: true }),
    "max-bytes": prop77.number({ default: 1e6 })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    min-height: 8rem;
    max-height: 24rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    background: color-mix(in oklch, var(--background) 95%, black 5%);
    color: var(--foreground);
    overflow: hidden;
  }
  [part='scroll'] {
    height: 100%;
    max-height: inherit;
    margin: 0;
    padding: var(--space-3) var(--space-4);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
    overflow-y: auto;
  }
  [part='scroll'][data-empty] {
    color: var(--muted-foreground);
    font-style: italic;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.url = props.url;
    $scope.event = props.event;
    $scope.autoscroll = props.autoscroll;
    $scope.active = props.active;
    $scope.maxBytes = props.maxBytes;
    $scope["preEl"] = signal68(null);
    let __r = null;
    if (sui_log_stream_controller_default && typeof sui_log_stream_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_log_stream_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets80(__root);
    const __e0 = document.createElement("pre");
    __e0.setAttribute("part", "scroll");
    __e0.setAttribute("data-empty", "");
    attachRef47($scope, "preEl", __e0);
    const __t1 = document.createTextNode("(no output yet)");
    __e0.appendChild(__t1);
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

// packages/ui-components/src/sui-harness-blob/sui-harness-blob.sui
import { adoptSuiSheets as adoptSuiSheets81, attachRef as attachRef48, bindAttr as bindAttr44, component as component81, prop as prop78, signal as signal69 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-harness-blob/sui-harness-blob.controller.ts
import { bindHtml as bindHtml6, controller as controller72, derived as derived11, effect as effect60, ref as ref8 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-harness-blob/harness-logos.ts
var CLAUDE_SVG = '<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="#D97757"><title>Claude</title><path d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></svg>';
var CURSOR_SVG = '<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><title>Cursor</title><path d="M11.925.0078 24 6.9702v10.0562L11.925 24 0 17.0288V6.9702ZM2.64 8.529 12 13.924v9.6448l9.36-5.3962V7.529l-9.36 5.4ZM10.677 11.11 2.64 6.4702l9.36-5.4038 8.01 4.6212Z"/></svg>';
var GEMINI_SVG = '<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="#8E75B2"><title>Google Gemini</title><path d="M11.04 19.32Q12 21.51 12 24q0-2.49.93-4.68.96-2.19 2.58-3.81t3.81-2.55Q21.51 12 24 12q-2.49 0-4.68-.93a12.3 12.3 0 0 1-3.81-2.58 12.3 12.3 0 0 1-2.58-3.81Q12 2.49 12 0q0 2.49-.96 4.68-.93 2.19-2.55 3.81a12.3 12.3 0 0 1-3.81 2.58Q2.49 12 0 12q2.49 0 4.68.96 2.19.93 3.81 2.55t2.55 3.81"/></svg>';
var OPENAI_BLACK_SVG = '<svg width="721" height="721" viewBox="0 0 721 721" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1637_2934)"><g clip-path="url(#clip1_1637_2934)"><path d="M304.246 294.611V249.028C304.246 245.189 305.687 242.309 309.044 240.392L400.692 187.612C413.167 180.415 428.042 177.058 443.394 177.058C500.971 177.058 537.44 221.682 537.44 269.182C537.44 272.54 537.44 276.379 536.959 280.218L441.954 224.558C436.197 221.201 430.437 221.201 424.68 224.558L304.246 294.611ZM518.245 472.145V363.224C518.245 356.505 515.364 351.707 509.608 348.349L389.174 278.296L428.519 255.743C431.877 253.826 434.757 253.826 438.115 255.743L529.762 308.523C556.154 323.879 573.905 356.505 573.905 388.171C573.905 424.636 552.315 458.225 518.245 472.141V472.145ZM275.937 376.182L236.592 353.152C233.235 351.235 231.794 348.354 231.794 344.515V238.956C231.794 187.617 271.139 148.749 324.4 148.749C344.555 148.749 363.264 155.468 379.102 167.463L284.578 222.164C278.822 225.521 275.942 230.319 275.942 237.039V376.186L275.937 376.182ZM360.626 425.122L304.246 393.455V326.283L360.626 294.616L417.002 326.283V393.455L360.626 425.122ZM396.852 570.989C376.698 570.989 357.989 564.27 342.151 552.276L436.674 497.574C442.431 494.217 445.311 489.419 445.311 482.699V343.552L485.138 366.582C488.495 368.499 489.936 371.379 489.936 375.219V480.778C489.936 532.117 450.109 570.985 396.852 570.985V570.989ZM283.134 463.99L191.486 411.211C165.094 395.854 147.343 363.229 147.343 331.562C147.343 294.616 169.415 261.509 203.48 247.593V356.991C203.48 363.71 206.361 368.508 212.117 371.866L332.074 441.437L292.729 463.99C289.372 465.907 286.491 465.907 283.134 463.99ZM277.859 542.68C223.639 542.68 183.813 501.895 183.813 451.514C183.813 447.675 184.294 443.836 184.771 439.997L279.295 494.698C285.051 498.056 290.812 498.056 296.568 494.698L417.002 425.127V470.71C417.002 474.549 415.562 477.429 412.204 479.346L320.557 532.126C308.081 539.323 293.206 542.68 277.854 542.68H277.859ZM396.852 599.776C454.911 599.776 503.37 558.513 514.41 503.812C568.149 489.896 602.696 439.515 602.696 388.176C602.696 354.587 588.303 321.962 562.392 298.45C564.791 288.373 566.231 278.296 566.231 268.224C566.231 199.611 510.571 148.267 446.274 148.267C433.322 148.267 420.846 150.184 408.37 154.505C386.775 133.392 357.026 119.958 324.4 119.958C266.342 119.958 217.883 161.22 206.843 215.921C153.104 229.837 118.557 280.218 118.557 331.557C118.557 365.146 132.95 397.771 158.861 421.283C156.462 431.36 155.022 441.437 155.022 451.51C155.022 520.123 210.682 571.466 274.978 571.466C287.931 571.466 300.407 569.549 312.883 565.228C334.473 586.341 364.222 599.776 396.852 599.776Z" fill="black"/></g></g><defs><clipPath id="clip0_1637_2934"><rect width="720" height="720" fill="white" transform="translate(0.606934 0.0999756)"/></clipPath><clipPath id="clip1_1637_2934"><rect width="484.139" height="479.818" fill="white" transform="translate(118.557 119.958)"/></clipPath></defs></svg>';
var OPENAI_WHITE_SVG = '<svg width="721" height="721" viewBox="0 0 721 721" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1637_2935)"><g clip-path="url(#clip1_1637_2935)"><path d="M304.246 295.411V249.828C304.246 245.989 305.687 243.109 309.044 241.191L400.692 188.412C413.167 181.215 428.042 177.858 443.394 177.858C500.971 177.858 537.44 222.482 537.44 269.982C537.44 273.34 537.44 277.179 536.959 281.018L441.954 225.358C436.197 222 430.437 222 424.68 225.358L304.246 295.411ZM518.245 472.945V364.024C518.245 357.304 515.364 352.507 509.608 349.149L389.174 279.096L428.519 256.543C431.877 254.626 434.757 254.626 438.115 256.543L529.762 309.323C556.154 324.679 573.905 357.304 573.905 388.971C573.905 425.436 552.315 459.024 518.245 472.941V472.945ZM275.937 376.982L236.592 353.952C233.235 352.034 231.794 349.154 231.794 345.315V239.756C231.794 188.416 271.139 149.548 324.4 149.548C344.555 149.548 363.264 156.268 379.102 168.262L284.578 222.964C278.822 226.321 275.942 231.119 275.942 237.838V376.986L275.937 376.982ZM360.626 425.922L304.246 394.255V327.083L360.626 295.416L417.002 327.083V394.255L360.626 425.922ZM396.852 571.789C376.698 571.789 357.989 565.07 342.151 553.075L436.674 498.374C442.431 495.017 445.311 490.219 445.311 483.499V344.352L485.138 367.382C488.495 369.299 489.936 372.179 489.936 376.018V481.577C489.936 532.917 450.109 571.785 396.852 571.785V571.789ZM283.134 464.79L191.486 412.01C165.094 396.654 147.343 364.029 147.343 332.362C147.343 295.416 169.415 262.309 203.48 248.393V357.791C203.48 364.51 206.361 369.308 212.117 372.665L332.074 442.237L292.729 464.79C289.372 466.707 286.491 466.707 283.134 464.79ZM277.859 543.48C223.639 543.48 183.813 502.695 183.813 452.314C183.813 448.475 184.294 444.636 184.771 440.797L279.295 495.498C285.051 498.856 290.812 498.856 296.568 495.498L417.002 425.927V471.509C417.002 475.349 415.562 478.229 412.204 480.146L320.557 532.926C308.081 540.122 293.206 543.48 277.854 543.48H277.859ZM396.852 600.576C454.911 600.576 503.37 559.313 514.41 504.612C568.149 490.696 602.696 440.315 602.696 388.976C602.696 355.387 588.303 322.762 562.392 299.25C564.791 289.173 566.231 279.096 566.231 269.024C566.231 200.411 510.571 149.067 446.274 149.067C433.322 149.067 420.846 150.984 408.37 155.305C386.775 134.192 357.026 120.758 324.4 120.758C266.342 120.758 217.883 162.02 206.843 216.721C153.104 230.637 118.557 281.018 118.557 332.357C118.557 365.946 132.95 398.571 158.861 422.083C156.462 432.16 155.022 442.237 155.022 452.309C155.022 520.922 210.682 572.266 274.978 572.266C287.931 572.266 300.407 570.349 312.883 566.028C334.473 587.141 364.222 600.576 396.852 600.576Z" fill="white"/></g></g><defs><clipPath id="clip0_1637_2935"><rect width="720" height="720" fill="white" transform="translate(0.606934 0.899902)"/></clipPath><clipPath id="clip1_1637_2935"><rect width="484.139" height="479.818" fill="white" transform="translate(118.557 120.758)"/></clipPath></defs></svg>';
var OPENCODE_LIGHT_SVG = '<svg width="240" height="300" viewBox="0 0 240 300" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1401_86274)"><mask id="mask0_1401_86274" style="mask-type:luminance" maskUnits="userSpaceOnUse" x="0" y="0" width="240" height="300"><path d="M240 0H0V300H240V0Z" fill="white"/></mask><g mask="url(#mask0_1401_86274)"><path d="M180 240H60V120H180V240Z" fill="#CFCECD"/><path d="M180 60H60V240H180V60ZM240 300H0V0H240V300Z" fill="#211E1E"/></g></g><defs><clipPath id="clip0_1401_86274"><rect width="240" height="300" fill="white"/></clipPath></defs></svg>';
var OPENCODE_DARK_SVG = '<svg width="240" height="300" viewBox="0 0 240 300" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1401_86283)"><mask id="mask0_1401_86283" style="mask-type:luminance" maskUnits="userSpaceOnUse" x="0" y="0" width="240" height="300"><path d="M240 0H0V300H240V0Z" fill="white"/></mask><g mask="url(#mask0_1401_86283)"><path d="M180 240H60V120H180V240Z" fill="#4B4646"/><path d="M180 60H60V240H180V60ZM240 300H0V0H240V300Z" fill="#F1ECEC"/></g></g><defs><clipPath id="clip0_1401_86283"><rect width="240" height="300" fill="white"/></clipPath></defs></svg>';
var HARNESSES = {
  claude: {
    label: "Claude",
    hue: "oklch(0.70 0.16 35)",
    svg: CLAUDE_SVG
  },
  codex: {
    label: "Codex",
    hue: "oklch(0.22 0.02 220)",
    svg: OPENAI_BLACK_SVG,
    svgDark: OPENAI_WHITE_SVG
  },
  cursor: {
    label: "Cursor",
    hue: "oklch(0.45 0.02 250)",
    svg: CURSOR_SVG
  },
  gemini: {
    label: "Gemini",
    hue: "oklch(0.62 0.14 285)",
    svg: GEMINI_SVG
  },
  opencode: {
    label: "OpenCode",
    hue: "oklch(0.60 0.18 35)",
    svg: OPENCODE_LIGHT_SVG,
    svgDark: OPENCODE_DARK_SVG
  }
};

// packages/ui-components/src/sui-harness-blob/sui-harness-blob.controller.ts
var sui_harness_blob_controller_default = controller72({
  scope: (state) => {
    const s = state;
    const tileEl = ref8(null);
    const tileLabel = derived11(() => (HARNESSES[s.harness()] ?? HARNESSES.claude).label);
    return { tileEl, tileLabel };
  },
  mount(state, host) {
    const scope = state;
    const stopHue = effect60(() => {
      const meta = HARNESSES[scope.harness()] ?? HARNESSES.claude;
      host.style.setProperty("--blob-hue", meta.hue);
    });
    const stopSvg = effect60(() => {
      const tile = scope.tileEl();
      if (!tile)
        return;
      return bindHtml6(tile, () => {
        const meta = HARNESSES[scope.harness()] ?? HARNESSES.claude;
        return meta.svg;
      });
    });
    return () => {
      stopHue();
      stopSvg();
    };
  }
});

// packages/ui-components/src/sui-harness-blob/sui-harness-blob.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_harness_blob_default = component81("sui-harness-blob", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    harness: prop78.union(["claude", "codex", "cursor", "gemini", "opencode"], { default: "claude", reflect: true }),
    size: prop78.union(["sm", "md", "lg"], { default: "md", reflect: true }),
    variant: prop78.union(["tile", "glyph"], { default: "tile", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    vertical-align: middle;
    font-family: var(--font-body);
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    --blob-size: 20px;
    --blob-hue: var(--muted);
  }
  :host([size='sm']) { --blob-size: 14px; font-size: 11px; }
  :host([size='md']) { --blob-size: 20px; }
  :host([size='lg']) { --blob-size: 28px; font-size: var(--text-sm); }
  .tile {
    width: var(--blob-size);
    height: var(--blob-size);
    flex-shrink: 0;
    border-radius: calc(var(--blob-size) * 0.22);
    display: grid;
    place-items: center;
    background: color-mix(in oklch, var(--blob-hue) 14%, var(--surface-1));
    box-shadow: inset 0 0 0 1px color-mix(in oklch, var(--blob-hue) 20%, transparent);
    overflow: hidden;
  }
  :host([variant='glyph']) .tile {
    background: transparent;
    box-shadow: none;
  }
  .tile-inner {
    display: contents;
  }
  .tile svg {
    width: 72%;
    height: 72%;
    display: block;
  }
  :host([variant='glyph']) .tile svg {
    width: 100%;
    height: 100%;
  }
  .label {
    white-space: nowrap;
    text-transform: capitalize;
  }
  .label:empty { display: none; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.harness = props.harness;
    $scope.size = props.size;
    $scope.variant = props.variant;
    $scope["tileEl"] = signal69(null);
    let __r = null;
    if (sui_harness_blob_controller_default && typeof sui_harness_blob_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_harness_blob_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets81(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "tile");
    __e0.setAttribute("role", "img");
    __cleanups.push(bindAttr44(__e0, "aria-label", () => $scope.tileLabel()));
    const __e1 = document.createElement("span");
    __e1.setAttribute("class", "tile-inner");
    attachRef48($scope, "tileEl", __e1);
    __e0.appendChild(__e1);
    __root.appendChild(__e0);
    const __e2 = document.createElement("span");
    __e2.setAttribute("class", "label");
    const __e3 = document.createElement("slot");
    __e2.appendChild(__e3);
    __root.appendChild(__e2);
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

// packages/ui-components/src/sui-auth-dialog/sui-auth-dialog.sui
import { adoptSuiSheets as adoptSuiSheets82, attachRef as attachRef49, bindAttr as bindAttr45, bindIf as bindIf30, bindText as bindText24, component as component82, on as on7, prop as prop79, signal as signal71 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-auth-dialog/sui-auth-dialog.controller.ts
import { controller as controller73, derived as derived12, effect as effect61, ref as ref9, signal as signal70 } from "@ship/sui-runtime";
var sui_auth_dialog_controller_default = controller73({
  scope: (state, host) => {
    const s = state;
    const dialogRef = ref9(null);
    const formRef = ref9(null);
    const providersSlot = ref9(null);
    const hasProviders = signal70(false);
    const defaultTitle = derived12(() => s.mode() === "sign-up" ? "Create your account" : "Welcome back");
    const submitLabel = derived12(() => {
      if (s.loading()) {
        return s.mode() === "sign-up" ? "Creating account…" : "Signing in…";
      }
      return s.mode() === "sign-up" ? "Create account" : "Sign in";
    });
    const otherMode = derived12(() => s.mode() === "sign-up" ? "sign-in" : "sign-up");
    const switchLabel = derived12(() => otherMode() === "sign-up" ? "Create an account" : "Sign in instead");
    const switchPrompt = derived12(() => s.mode() === "sign-up" ? "Already have an account?" : "New here?");
    const passwordAutocomplete = derived12(() => s.mode() === "sign-up" ? "new-password" : "current-password");
    function setMode(next) {
      if (next === s.mode())
        return;
      s.mode.set(next);
      host.dispatchEvent(new CustomEvent("sui-mode-change", {
        detail: { mode: next },
        bubbles: true,
        composed: true
      }));
    }
    function onTabChange(e) {
      const detail = e.detail;
      if (!detail || typeof detail.name !== "string")
        return;
      const next = detail.name === "sign-up" ? "sign-up" : "sign-in";
      setMode(next);
    }
    function onSubmit(e) {
      e.preventDefault();
      const form = e.currentTarget ?? formRef();
      if (!form)
        return;
      const email = form.elements.namedItem("email")?.value ?? "";
      const password = form.elements.namedItem("password")?.value ?? "";
      host.dispatchEvent(new CustomEvent("sui-auth-submit", {
        detail: { mode: s.mode(), email, password },
        bubbles: true,
        composed: true
      }));
    }
    function onSwitchClick() {
      setMode(otherMode());
    }
    function onDialogClose() {
      if (s.open())
        s.open.set(false);
    }
    return {
      dialogRef,
      formRef,
      providersSlot,
      hasProviders,
      defaultTitle,
      submitLabel,
      otherMode,
      switchLabel,
      switchPrompt,
      passwordAutocomplete,
      onTabChange,
      onSubmit,
      onSwitchClick,
      onDialogClose
    };
  },
  mount(state, host) {
    const scope = state;
    const recomputeProviders = () => {
      const slot = scope.providersSlot();
      if (!slot) {
        scope.hasProviders.set(false);
        return;
      }
      const filled = slot.assignedNodes({ flatten: true }).some((n) => {
        if (n.nodeType === Node.TEXT_NODE) {
          return (n.textContent || "").trim().length > 0;
        }
        return n.nodeType === Node.ELEMENT_NODE;
      });
      scope.hasProviders.set(filled);
    };
    const stopSlotBind = effect61(() => {
      const slot = scope.providersSlot();
      if (!slot)
        return;
      slot.addEventListener("slotchange", recomputeProviders);
      recomputeProviders();
      return () => slot.removeEventListener("slotchange", recomputeProviders);
    });
    const stopOpenSync = effect61(() => {
      const wantOpen = scope.open();
      const dlg = scope.dialogRef();
      if (!dlg)
        return;
      if (wantOpen && typeof dlg.showModal === "function") {
        if (!dlg.hasAttribute("open"))
          dlg.showModal();
      } else if (!wantOpen && typeof dlg.close === "function") {
        if (dlg.hasAttribute("open"))
          dlg.close();
      }
    });
    const target = host;
    target.show = () => scope.open.set(true);
    target.showModal = () => scope.open.set(true);
    target.close = () => scope.open.set(false);
    return () => {
      stopSlotBind();
      stopOpenSync();
    };
  }
});

// packages/ui-components/src/sui-auth-dialog/sui-auth-dialog.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_auth_dialog_default = component82("sui-auth-dialog", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    open: prop79.bool({ default: false, reflect: true }),
    mode: prop79.union(["sign-in", "sign-up"], { default: "sign-in", reflect: true }),
    loading: prop79.bool({ default: false, reflect: true }),
    backdrop: prop79.union(["default", "blur", "solid"], { default: "blur", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: none; }
  :host([open]) { display: contents; }

  .body {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 4px 0 0;
    font-family: var(--font-body);
    color: var(--foreground);
  }

  .harnesses {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  ::slotted(*) { width: 100%; }

  .or {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--muted-foreground);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .or::before, .or::after {
    content: '';
    flex: 1;
    height: 1px;
    background: color-mix(in oklch, var(--border) 55%, transparent);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 500;
  }
  input {
    height: 38px;
    padding: 0 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--card);
    color: var(--foreground);
    font: inherit;
    font-size: 14px;
    transition: border-color 120ms ease, box-shadow 120ms ease;
  }
  input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--tone-accent-ring, var(--ring));
  }

  .submit-row {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 4px;
  }
  .submit-row sui-button { width: 100%; }

  .terms {
    color: var(--muted-foreground);
    font-size: 11.5px;
    text-align: center;
    line-height: 1.5;
    margin: 0;
  }

  .switch {
    color: var(--muted-foreground);
    font-size: 12.5px;
    text-align: center;
    margin: 0;
  }
  .switch button {
    appearance: none;
    background: transparent;
    border: 0;
    padding: 0;
    color: var(--accent);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.mode = props.mode;
    $scope.loading = props.loading;
    $scope.backdrop = props.backdrop;
    $scope["dialogRef"] = signal71(null);
    $scope["formRef"] = signal71(null);
    $scope["providersSlot"] = signal71(null);
    let __r = null;
    if (sui_auth_dialog_controller_default && typeof sui_auth_dialog_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_auth_dialog_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b7 = (anchor) => bindIf30({
      anchor,
      test: () => {
        if ($scope.hasProviders())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag11 = document.createDocumentFragment();
          const __c12 = [];
          const __e13 = document.createElement("div");
          __e13.setAttribute("class", "or");
          const __t14 = document.createTextNode("or");
          __e13.appendChild(__t14);
          __frag11.appendChild(__e13);
          return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
            for (const c of __c12)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets82(__root);
    const __e0 = document.createElement("sui-dialog");
    __e0.setAttribute("part", "dialog");
    __e0.setAttribute("size", "sm");
    __cleanups.push(bindAttr45(__e0, "backdrop", () => $scope.backdrop()));
    attachRef49($scope, "dialogRef", __e0);
    __cleanups.push(on7(__e0, "sui-close", function() {
      const __h = $scope.onDialogClose;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e1 = document.createElement("div");
    __e1.setAttribute("slot", "header");
    const __e2 = document.createElement("slot");
    __e2.setAttribute("name", "header");
    const __t3 = document.createTextNode("");
    __e2.appendChild(__t3);
    __cleanups.push(bindText24(__t3, () => $scope.defaultTitle()));
    __e1.appendChild(__e2);
    __e0.appendChild(__e1);
    const __e4 = document.createElement("div");
    __e4.setAttribute("class", "body");
    const __e5 = document.createElement("sui-tabs");
    __e5.setAttribute("variant", "pill");
    __cleanups.push(bindAttr45(__e5, "active", () => $scope.mode()));
    __cleanups.push(on7(__e5, "sui-tab-change", function() {
      const __h = $scope.onTabChange;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e6 = document.createElement("sui-tab");
    __e6.setAttribute("name", "sign-in");
    __e6.setAttribute("label", "Sign in");
    __e5.appendChild(__e6);
    const __e7 = document.createElement("sui-tab");
    __e7.setAttribute("name", "sign-up");
    __e7.setAttribute("label", "Sign up");
    __e5.appendChild(__e7);
    __e4.appendChild(__e5);
    const __e8 = document.createElement("div");
    __e8.setAttribute("class", "harnesses");
    const __e9 = document.createElement("slot");
    __e9.setAttribute("name", "harnesses");
    attachRef49($scope, "providersSlot", __e9);
    __e8.appendChild(__e9);
    __e4.appendChild(__e8);
    const __ifAnchor10 = document.createComment("if");
    __e4.appendChild(__ifAnchor10);
    __cleanups.push(__wireBoundary_b7(__ifAnchor10));
    const __e15 = document.createElement("form");
    __e15.setAttribute("part", "form");
    __cleanups.push(on7(__e15, "submit", function() {
      const __h = $scope.onSubmit;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    __e15.setAttribute("novalidate", "");
    attachRef49($scope, "formRef", __e15);
    const __e16 = document.createElement("label");
    const __t17 = document.createTextNode(`
        Email
        `);
    __e16.appendChild(__t17);
    const __e18 = document.createElement("input");
    __e18.setAttribute("name", "email");
    __e18.setAttribute("type", "email");
    __e18.setAttribute("autocomplete", "email");
    __e18.setAttribute("inputmode", "email");
    __e18.setAttribute("required", "");
    __e18.setAttribute("placeholder", "you@company.com");
    __e16.appendChild(__e18);
    __e15.appendChild(__e16);
    const __e19 = document.createElement("label");
    const __t20 = document.createTextNode(`
        Password
        `);
    __e19.appendChild(__t20);
    const __e21 = document.createElement("input");
    __e21.setAttribute("name", "password");
    __e21.setAttribute("type", "password");
    __cleanups.push(bindAttr45(__e21, "autocomplete", () => $scope.passwordAutocomplete()));
    __e21.setAttribute("required", "");
    __e21.setAttribute("minlength", "8");
    __e19.appendChild(__e21);
    __e15.appendChild(__e19);
    const __e22 = document.createElement("div");
    __e22.setAttribute("class", "submit-row");
    const __e23 = document.createElement("sui-button");
    __e23.setAttribute("part", "submit");
    __e23.setAttribute("tone", "accent");
    __e23.setAttribute("variant", "solid");
    __e23.setAttribute("size", "md");
    __e23.setAttribute("type", "submit");
    __cleanups.push(bindAttr45(__e23, "loading", () => $scope.loading()));
    __cleanups.push(bindAttr45(__e23, "disabled", () => $scope.loading()));
    const __t24 = document.createTextNode("");
    __e23.appendChild(__t24);
    __cleanups.push(bindText24(__t24, () => $scope.submitLabel()));
    __e22.appendChild(__e23);
    const __e25 = document.createElement("p");
    __e25.setAttribute("class", "terms");
    const __e26 = document.createElement("slot");
    __e26.setAttribute("name", "terms");
    __e25.appendChild(__e26);
    __e22.appendChild(__e25);
    __e15.appendChild(__e22);
    __e4.appendChild(__e15);
    const __e27 = document.createElement("p");
    __e27.setAttribute("class", "switch");
    const __t28 = document.createTextNode("");
    __e27.appendChild(__t28);
    __cleanups.push(bindText24(__t28, () => $scope.switchPrompt()));
    const __e29 = document.createElement("button");
    __e29.setAttribute("type", "button");
    __cleanups.push(on7(__e29, "click", function() {
      const __h = $scope.onSwitchClick;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __t30 = document.createTextNode("");
    __e29.appendChild(__t30);
    __cleanups.push(bindText24(__t30, () => $scope.switchLabel()));
    __e27.appendChild(__e29);
    __e4.appendChild(__e27);
    __e0.appendChild(__e4);
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

// packages/ui-components/src/sui-git-diff/sui-git-diff.sui
import { adoptSuiSheets as adoptSuiSheets83, bindAttr as bindAttr46, bindIf as bindIf31, bindText as bindText25, component as component83, on as on8, prop as prop80 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-git-diff/sui-git-diff.controller.ts
import { controller as controller74, derived as derived13, effect as effect62, signal as signal72 } from "@ship/sui-runtime";
var sui_git_diff_controller_default = controller74({
  scope: (state, host) => {
    const s = state;
    const allCollapsed = signal72(false);
    const showEmpty = derived13(() => s.empty() || s.files() === 0);
    const filesLabel = derived13(() => {
      const n = s.files();
      return `${n} file${n === 1 ? "" : "s"} changed`;
    });
    const toggleIcon = derived13(() => allCollapsed() ? "chevrons-up-down" : "chevrons-down-up");
    const toggleLabel = derived13(() => allCollapsed() ? "Expand all" : "Collapse all");
    const toggleTitle = derived13(() => allCollapsed() ? "Expand all files" : "Collapse all files");
    function toggleAll() {
      const next = !allCollapsed();
      allCollapsed.set(next);
      const kids = host.querySelectorAll("sui-git-diff-file");
      kids.forEach((f) => {
        if (next)
          f.setAttribute("collapsed", "");
        else
          f.removeAttribute("collapsed");
      });
    }
    return {
      allCollapsed,
      showEmpty,
      filesLabel,
      toggleIcon,
      toggleLabel,
      toggleTitle,
      toggleAll
    };
  },
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "region");
    const stopLabel = effect62(() => {
      const f = scope.files();
      const a = scope.additions();
      const d = scope.deletions();
      host.setAttribute("aria-label", `Diff: ${f} file${f === 1 ? "" : "s"} changed, ${a} additions, ${d} deletions`);
    });
    return () => {
      stopLabel();
    };
  }
});

// packages/ui-components/src/sui-git-diff/sui-git-diff.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_git_diff_default = component83("sui-git-diff", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    files: prop80.number({ default: 0, reflect: true }),
    additions: prop80.number({ default: 0, reflect: true }),
    deletions: prop80.number({ default: 0, reflect: true }),
    empty: prop80.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    flex-direction: column;
    min-height: 0;
    font-family: var(--font-body);
    background: var(--background);
    color: var(--foreground);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 16px;
    border-bottom: 1px solid var(--border-subtle);
    background: color-mix(in oklch, var(--card) 50%, transparent);
    flex-shrink: 0;
  }
  .count { font-size: 12px; color: var(--muted-foreground); }
  .adds { font-size: 11px; color: var(--success); }
  .dels { font-size: 11px; color: var(--destructive); }
  .spacer { flex: 1; }
  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: var(--radius-md);
    background: transparent;
    border: 0;
    font-family: inherit;
    font-size: 11px;
    color: var(--muted-foreground);
    cursor: pointer;
    transition: background var(--duration-fast) ease, color var(--duration-fast) ease;
  }
  .toggle:hover { background: var(--muted); color: var(--foreground); }
  .body { flex: 1; overflow: auto; }
  :host([empty]) .toolbar { display: none; }
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 48px 16px;
    color: var(--muted-foreground);
    font-size: 13px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.files = props.files;
    $scope.additions = props.additions;
    $scope.deletions = props.deletions;
    $scope.empty = props.empty;
    let __r = null;
    if (sui_git_diff_controller_default && typeof sui_git_diff_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_git_diff_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b2 = (anchor) => bindIf31({
      anchor,
      test: () => {
        if ($scope.additions() > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag11 = document.createDocumentFragment();
          const __c12 = [];
          const __e13 = document.createElement("span");
          __e13.setAttribute("class", "adds");
          const __t14 = document.createTextNode("+");
          __e13.appendChild(__t14);
          const __t15 = document.createTextNode("");
          __e13.appendChild(__t15);
          __c12.push(bindText25(__t15, () => $scope.additions()));
          __frag11.appendChild(__e13);
          return { nodes: Array.from(__frag11.childNodes), cleanup: () => {
            for (const c of __c12)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b4 = (anchor) => bindIf31({
      anchor,
      test: () => {
        if ($scope.deletions() > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag17 = document.createDocumentFragment();
          const __c18 = [];
          const __e19 = document.createElement("span");
          __e19.setAttribute("class", "dels");
          const __t20 = document.createTextNode("-");
          __e19.appendChild(__t20);
          const __t21 = document.createTextNode("");
          __e19.appendChild(__t21);
          __c18.push(bindText25(__t21, () => $scope.deletions()));
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
    const __wireBoundary_b0 = (anchor) => bindIf31({
      anchor,
      test: () => {
        if ($scope.showEmpty())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag1 = document.createDocumentFragment();
          const __c2 = [];
          const __e3 = document.createElement("div");
          __e3.setAttribute("class", "empty");
          const __t4 = document.createTextNode("No changes to display");
          __e3.appendChild(__t4);
          __frag1.appendChild(__e3);
          return { nodes: Array.from(__frag1.childNodes), cleanup: () => {
            for (const c of __c2)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag5 = document.createDocumentFragment();
          const __c6 = [];
          const __e7 = document.createElement("div");
          __e7.setAttribute("class", "toolbar");
          const __e8 = document.createElement("span");
          __e8.setAttribute("class", "count");
          const __t9 = document.createTextNode("");
          __e8.appendChild(__t9);
          __c6.push(bindText25(__t9, () => $scope.filesLabel()));
          __e7.appendChild(__e8);
          const __ifAnchor10 = document.createComment("if");
          __e7.appendChild(__ifAnchor10);
          __c6.push(__wireBoundary_b2(__ifAnchor10));
          const __ifAnchor16 = document.createComment("if");
          __e7.appendChild(__ifAnchor16);
          __c6.push(__wireBoundary_b4(__ifAnchor16));
          const __e22 = document.createElement("span");
          __e22.setAttribute("class", "spacer");
          __e7.appendChild(__e22);
          const __e23 = document.createElement("button");
          __e23.setAttribute("class", "toggle");
          __e23.setAttribute("type", "button");
          __c6.push(on8(__e23, "click", function() {
            const __h = $scope.toggleAll;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          __c6.push(bindAttr46(__e23, "title", () => $scope.toggleTitle()));
          const __e24 = document.createElement("sui-icon");
          __e24.setAttribute("size", "sm");
          __c6.push(bindAttr46(__e24, "name", () => $scope.toggleIcon()));
          __e23.appendChild(__e24);
          const __t25 = document.createTextNode("");
          __e23.appendChild(__t25);
          __c6.push(bindText25(__t25, () => $scope.toggleLabel()));
          __e7.appendChild(__e23);
          __frag5.appendChild(__e7);
          const __e26 = document.createElement("div");
          __e26.setAttribute("class", "body");
          const __e27 = document.createElement("slot");
          __e26.appendChild(__e27);
          __frag5.appendChild(__e26);
          return { nodes: Array.from(__frag5.childNodes), cleanup: () => {
            for (const c of __c6)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets83(__root);
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

// packages/ui-components/src/sui-git-diff/sui-git-diff-file.sui
import { adoptSuiSheets as adoptSuiSheets84, bindAttr as bindAttr47, bindIf as bindIf32, bindText as bindText26, component as component84, on as on9, prop as prop81 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-git-diff/sui-git-diff-file.controller.ts
import { controller as controller75, derived as derived14, effect as effect63 } from "@ship/sui-runtime";
var sui_git_diff_file_controller_default = controller75({
  scope: (state) => {
    const s = state;
    const chevIcon = derived14(() => s.collapsed() ? "chevron-right" : "chevron-down");
    const expandedLabel = derived14(() => s.collapsed() ? "false" : "true");
    function toggle() {
      s.collapsed.set(!s.collapsed());
    }
    return { chevIcon, expandedLabel, toggle };
  },
  mount(state, host) {
    const scope = state;
    if (!host.hasAttribute("role"))
      host.setAttribute("role", "group");
    const stopLabel = effect63(() => {
      const path = scope.path();
      const a = scope.additions();
      const d = scope.deletions();
      host.setAttribute("aria-label", `${path}: +${a} -${d}`);
    });
    return () => {
      stopLabel();
    };
  }
});

// packages/ui-components/src/sui-git-diff/sui-git-diff-file.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_git_diff_file_default = component84("sui-git-diff-file", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    path: prop81.string({ default: "", reflect: true }),
    additions: prop81.number({ default: 0, reflect: true }),
    deletions: prop81.number({ default: 0, reflect: true }),
    collapsed: prop81.bool({ default: false, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    border-bottom: 1px solid color-mix(in oklch, var(--border) 35%, transparent);
  }
  .header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--border-subtle);
    text-align: left;
    cursor: pointer;
    color: var(--foreground);
    transition: background var(--duration-fast) ease;
    font-family: inherit;
  }
  .header:hover { background: color-mix(in oklch, var(--muted) 40%, transparent); }
  .chev { color: var(--muted-foreground); flex-shrink: 0; }
  .fileicon { color: var(--muted-foreground); flex-shrink: 0; }
  .path {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--foreground);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .stats { display: inline-flex; align-items: center; gap: 8px; flex-shrink: 0; }
  .adds, .dels {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-size: 10px;
    font-family: var(--font-mono);
  }
  .adds { color: var(--success); }
  .dels { color: var(--destructive); }
  .body { display: block; }
  :host([collapsed]) .body { display: none; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.path = props.path;
    $scope.additions = props.additions;
    $scope.deletions = props.deletions;
    $scope.collapsed = props.collapsed;
    let __r = null;
    if (sui_git_diff_file_controller_default && typeof sui_git_diff_file_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_git_diff_file_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b4 = (anchor) => bindIf32({
      anchor,
      test: () => {
        if ($scope.additions() > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag7 = document.createDocumentFragment();
          const __c8 = [];
          const __e9 = document.createElement("span");
          __e9.setAttribute("class", "adds");
          const __e10 = document.createElement("sui-icon");
          __e10.setAttribute("size", "sm");
          __e10.setAttribute("name", "plus");
          __e9.appendChild(__e10);
          const __t11 = document.createTextNode("");
          __e9.appendChild(__t11);
          __c8.push(bindText26(__t11, () => $scope.additions()));
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
    const __wireBoundary_b6 = (anchor) => bindIf32({
      anchor,
      test: () => {
        if ($scope.deletions() > 0)
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag13 = document.createDocumentFragment();
          const __c14 = [];
          const __e15 = document.createElement("span");
          __e15.setAttribute("class", "dels");
          const __e16 = document.createElement("sui-icon");
          __e16.setAttribute("size", "sm");
          __e16.setAttribute("name", "minus");
          __e15.appendChild(__e16);
          const __t17 = document.createTextNode("");
          __e15.appendChild(__t17);
          __c14.push(bindText26(__t17, () => $scope.deletions()));
          __frag13.appendChild(__e15);
          return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
            for (const c of __c14)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets84(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("class", "header");
    __e0.setAttribute("type", "button");
    __cleanups.push(on9(__e0, "click", function() {
      const __h = $scope.toggle;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    __cleanups.push(bindAttr47(__e0, "aria-expanded", () => $scope.expandedLabel()));
    const __e1 = document.createElement("sui-icon");
    __e1.setAttribute("class", "chev");
    __e1.setAttribute("size", "sm");
    __cleanups.push(bindAttr47(__e1, "name", () => $scope.chevIcon()));
    __e0.appendChild(__e1);
    const __e2 = document.createElement("sui-icon");
    __e2.setAttribute("class", "fileicon");
    __e2.setAttribute("size", "sm");
    __e2.setAttribute("name", "file-code");
    __e0.appendChild(__e2);
    const __e3 = document.createElement("span");
    __e3.setAttribute("class", "path");
    const __t4 = document.createTextNode("");
    __e3.appendChild(__t4);
    __cleanups.push(bindText26(__t4, () => $scope.path()));
    __e0.appendChild(__e3);
    const __e5 = document.createElement("span");
    __e5.setAttribute("class", "stats");
    const __ifAnchor6 = document.createComment("if");
    __e5.appendChild(__ifAnchor6);
    __cleanups.push(__wireBoundary_b4(__ifAnchor6));
    const __ifAnchor12 = document.createComment("if");
    __e5.appendChild(__ifAnchor12);
    __cleanups.push(__wireBoundary_b6(__ifAnchor12));
    __e0.appendChild(__e5);
    __root.appendChild(__e0);
    const __e18 = document.createElement("div");
    __e18.setAttribute("class", "body");
    const __e19 = document.createElement("slot");
    __e18.appendChild(__e19);
    __root.appendChild(__e18);
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

// packages/ui-components/src/sui-git-diff/sui-git-diff-hunk.sui
import { adoptSuiSheets as adoptSuiSheets85, bindText as bindText27, component as component85, prop as prop82 } from "@ship/sui-runtime";
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_git_diff_hunk_default = component85("sui-git-diff-hunk", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    header: prop82.string({ default: "", reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: block;
    padding: 2px 12px;
    background: var(--tone-info-bg);
    border-top: 1px solid var(--tone-info-bg);
    border-bottom: 1px solid var(--tone-info-bg);
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--info);
    user-select: none;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.header = props.header;
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets85(__root);
    const __t0 = document.createTextNode(`

`);
    __root.appendChild(__t0);
    const __t1 = document.createTextNode(`

`);
    __root.appendChild(__t1);
    const __t2 = document.createTextNode("");
    __root.appendChild(__t2);
    __cleanups.push(bindText27(__t2, () => $scope.header()));
    return $scope;
  }
});

// packages/ui-components/src/sui-git-diff/sui-git-diff-line.sui
import { adoptSuiSheets as adoptSuiSheets86, bindText as bindText28, component as component86, on as on10, prop as prop83 } from "@ship/sui-runtime";

// packages/ui-components/src/sui-git-diff/sui-git-diff-line.controller.ts
import { controller as controller76, derived as derived15, effect as effect64 } from "@ship/sui-runtime";
var sui_git_diff_line_controller_default = controller76({
  scope: (state, host) => {
    const propBag = state;
    const newSig = propBag["new"];
    const oldSig = propBag["old"];
    const typeSig = propBag["type"];
    const oldCol = derived15(() => oldSig?.() ?? "");
    const newCol = derived15(() => newSig?.() ?? "");
    const sigChar = derived15(() => {
      const t = typeSig?.();
      return t === "add" ? "+" : t === "del" ? "-" : " ";
    });
    function emitComment(e) {
      e.stopPropagation();
      host.dispatchEvent(new CustomEvent("sui-comment", {
        detail: { old: oldSig?.() ?? "", new: newSig?.() ?? "" },
        bubbles: true,
        composed: true
      }));
    }
    return { oldCol, newCol, sigChar, emitComment };
  },
  mount(state, host) {
    const propBag = state;
    const newSig = propBag["new"];
    const oldSig = propBag["old"];
    const typeSig = propBag["type"];
    const stopLabel = effect64(() => {
      const t = typeSig?.() ?? "context";
      const n = newSig?.() ?? "";
      const o = oldSig?.() ?? "";
      const verb = t === "add" ? "added" : t === "del" ? "removed" : "context";
      host.setAttribute("aria-label", `line ${n || o} ${verb}`);
    });
    return () => {
      stopLabel();
    };
  }
});

// packages/ui-components/src/sui-git-diff/sui-git-diff-line.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
var sui_git_diff_line_default = component86("sui-git-diff-line", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    type: prop83.union(["add", "del", "context"], { default: "context", reflect: true }),
    old: prop83.string({ default: "", reflect: true }),
    new: prop83.string({ default: "", reflect: true }),
    "comment-count": prop83.number({ default: 0, reflect: true })
  },
  shadow: true,
  styles: `
  :host {
    display: flex;
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.45;
    position: relative;
  }
  :host([type='add']) { background: var(--tone-success-bg); }
  :host([type='del']) { background: var(--tone-destructive-bg); }
  :host([comment-count]) { border-left: 2px solid var(--accent); }
  .num {
    width: 42px;
    flex-shrink: 0;
    padding-right: 8px;
    text-align: right;
    color: color-mix(in oklch, var(--muted-foreground) 60%, transparent);
    user-select: none;
    border-right: 1px solid color-mix(in oklch, var(--border) 20%, transparent);
  }
  .sig {
    width: 16px;
    flex-shrink: 0;
    text-align: center;
    user-select: none;
    color: var(--muted-foreground);
  }
  .content {
    flex: 1;
    padding-right: 28px;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--muted-foreground);
  }
  :host([type='add']) .sig,
  :host([type='add']) .content { color: var(--success); }
  :host([type='del']) .sig,
  :host([type='del']) .content { color: var(--destructive); }

  .comment-btn {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
    width: 18px;
    height: 18px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: var(--accent-foreground);
    border: 0;
    display: grid;
    place-items: center;
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--duration-fast) ease;
    font-size: 13px;
    line-height: 1;
    font-family: var(--font-body);
  }
  :host(:hover) .comment-btn { opacity: 1; }
  :host([comment-count]) .comment-btn { opacity: 0.7; }
  :host([comment-count]:hover) .comment-btn { opacity: 1; }
  .comment-btn:focus-visible {
    outline: 2px solid var(--ring-color, var(--ring));
    outline-offset: 1px;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.type = props.type;
    $scope.old = props.old;
    $scope.new = props.new;
    $scope.commentCount = props.commentCount;
    let __r = null;
    if (sui_git_diff_line_controller_default && typeof sui_git_diff_line_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_git_diff_line_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets86(__root);
    const __e0 = document.createElement("span");
    __e0.setAttribute("class", "num");
    const __t1 = document.createTextNode("");
    __e0.appendChild(__t1);
    __cleanups.push(bindText28(__t1, () => $scope.oldCol()));
    __root.appendChild(__e0);
    const __e2 = document.createElement("span");
    __e2.setAttribute("class", "num");
    const __t3 = document.createTextNode("");
    __e2.appendChild(__t3);
    __cleanups.push(bindText28(__t3, () => $scope.newCol()));
    __root.appendChild(__e2);
    const __e4 = document.createElement("span");
    __e4.setAttribute("class", "sig");
    const __t5 = document.createTextNode("");
    __e4.appendChild(__t5);
    __cleanups.push(bindText28(__t5, () => $scope.sigChar()));
    __root.appendChild(__e4);
    const __e6 = document.createElement("span");
    __e6.setAttribute("class", "content");
    const __e7 = document.createElement("slot");
    __e6.appendChild(__e7);
    __root.appendChild(__e6);
    const __e8 = document.createElement("button");
    __e8.setAttribute("class", "comment-btn");
    __e8.setAttribute("type", "button");
    __e8.setAttribute("aria-label", "Comment on this line");
    __e8.setAttribute("title", "Comment on this line");
    __cleanups.push(on10(__e8, "click", function() {
      const __h = $scope.emitComment;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __t9 = document.createTextNode("+");
    __e8.appendChild(__t9);
    __root.appendChild(__e8);
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
