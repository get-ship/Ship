// packages/ui-dom/src/setup-loader.ts
async function loadSetup(setUrl, loader = (p) => import(p)) {
  const tsUrl = pairedTsPath(setUrl);
  let mod;
  try {
    mod = await loader(tsUrl);
  } catch {
    return;
  }
  const def = mod?.default;
  if (typeof def === "function")
    return def;
  if (def && typeof def === "object" && "setup" in def) {
    const setup = def.setup;
    if (typeof setup === "function")
      return setup;
  }
  return;
}
function pairedTsPath(setUrl) {
  if (setUrl.endsWith(".sui")) {
    return setUrl.slice(0, -".sui".length) + ".ts";
  }
  if (setUrl.endsWith(".set")) {
    return setUrl.slice(0, -".set".length) + ".ts";
  }
  return setUrl + ".ts";
}
// packages/ui-dom/src/hydrate/walk.ts
function walk(root, path) {
  let n = root;
  for (let i = 0;i < path.length; i++) {
    const next = n.childNodes[path[i]];
    if (!next) {
      throw new Error(`[ship-set/hydrate] walk: path ${JSON.stringify(path)} fell off at step ${i} ` + `(parent <${n.tagName?.toLowerCase?.() ?? n.nodeName}> has ${n.childNodes.length} children)`);
    }
    n = next;
  }
  return n;
}
// packages/ui-dom/src/hydrate/text.ts
import { effect } from "@ship/sui-runtime";
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
  return effect(() => {
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
// packages/ui-dom/src/hydrate/markers.ts
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
// packages/ui-dom/src/hydrate/attr.ts
import { effect as effect2 } from "@ship/sui-runtime";
function hydrateAttr(el, name, getter) {
  return effect2(() => {
    const v = getter();
    if (v == null || v === false) {
      el.removeAttribute(name);
      return;
    }
    el.setAttribute(name, String(v));
  });
}
// packages/ui-dom/src/hydrate/class.ts
import { effect as effect3 } from "@ship/sui-runtime";
function hydrateClass(el, name, getter) {
  return effect3(() => {
    el.classList.toggle(name, !!getter());
  });
}
// packages/ui-dom/src/hydrate/style.ts
import { effect as effect4 } from "@ship/sui-runtime";
function hydrateStyle(el, prop, getter) {
  return effect4(() => {
    const v = getter();
    if (v == null || v === false || v === "") {
      el.style.removeProperty(prop);
      return;
    }
    el.style.setProperty(prop, String(v));
  });
}
// packages/ui-dom/src/hydrate/prop.ts
import { bindProp } from "@ship/sui-runtime";
function hydrateProp(el, name, getter) {
  return bindProp(el, name, getter);
}
// packages/ui-dom/src/hydrate/on.ts
import { on } from "@ship/sui-runtime";
function hydrateOn(el, event, handler, modifiers = []) {
  return on(el, event, handler, { modifiers });
}
// packages/ui-dom/src/hydrate/bind-value.ts
import { effect as effect5 } from "@ship/sui-runtime";
function hydrateBindValue(el, sig) {
  const onInput = () => {
    sig.set(el.value);
  };
  el.addEventListener("input", onInput);
  const dispose = effect5(() => {
    const v = sig();
    const next = v == null ? "" : String(v);
    if (el.value !== next) {
      el.value = next;
    }
  });
  return () => {
    el.removeEventListener("input", onInput);
    dispose();
  };
}
function hydrateBindChecked(el, sig) {
  const onChange = () => {
    sig.set(el.checked);
  };
  el.addEventListener("change", onChange);
  const dispose = effect5(() => {
    const next = !!sig();
    if (el.checked !== next)
      el.checked = next;
  });
  return () => {
    el.removeEventListener("change", onChange);
    dispose();
  };
}
// packages/ui-dom/src/hydrate/ref.ts
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
// packages/ui-dom/src/hydrate/while.ts
import { effect as effect6 } from "@ship/sui-runtime";
function hydrateWhileLoading(el, key, signalGetter, render) {
  let current = el;
  let branchDispose = null;
  let firstRun = true;
  const condDispose = effect6(() => {
    let isLoading = true;
    try {
      const v = signalGetter();
      isLoading = !v || v.kind === "loading";
    } catch {}
    if (firstRun) {
      firstRun = false;
      return;
    }
    applyTransition(isLoading);
  });
  function applyTransition(show) {
    if (branchDispose) {
      try {
        branchDispose();
      } catch {}
      branchDispose = null;
    }
    const parent = current.parentNode;
    if (!parent)
      return;
    const owner = current.ownerDocument ?? document;
    const next = owner.createElement("sui-while-loading");
    next.setAttribute("key", key);
    if (show) {
      next.style.display = "contents";
      const result = render();
      for (const node of result.nodes)
        next.appendChild(node);
      branchDispose = () => {
        try {
          result.cleanup();
        } catch {}
      };
    } else {
      next.setAttribute("hidden", "");
    }
    parent.replaceChild(next, current);
    current = next;
  }
  return () => {
    try {
      condDispose();
    } catch {}
    if (branchDispose) {
      try {
        branchDispose();
      } catch {}
      branchDispose = null;
    }
  };
}
function hydrateWhileError(el, key, signalGetter, render) {
  let current = el;
  let branchDispose = null;
  let firstRun = true;
  const condDispose = effect6(() => {
    let isError = false;
    let errorVal;
    try {
      const v = signalGetter();
      isError = !!v && v.kind === "error";
      errorVal = v?.error;
    } catch {}
    if (firstRun) {
      firstRun = false;
      return;
    }
    applyTransition(isError, errorVal);
  });
  function applyTransition(show, errorVal) {
    if (branchDispose) {
      try {
        branchDispose();
      } catch {}
      branchDispose = null;
    }
    const parent = current.parentNode;
    if (!parent)
      return;
    const owner = current.ownerDocument ?? document;
    const next = owner.createElement("sui-while-error");
    next.setAttribute("key", key);
    if (show) {
      next.style.display = "contents";
      const result = render(errorVal);
      for (const node of result.nodes)
        next.appendChild(node);
      branchDispose = () => {
        try {
          result.cleanup();
        } catch {}
      };
    } else {
      next.setAttribute("hidden", "");
    }
    parent.replaceChild(next, current);
    current = next;
  }
  return () => {
    try {
      condDispose();
    } catch {}
    if (branchDispose) {
      try {
        branchDispose();
      } catch {}
      branchDispose = null;
    }
  };
}
export {
  walk,
  pairedTsPath,
  loadSetup,
  hydrateWhileLoading,
  hydrateWhileError,
  hydrateText,
  hydrateStyle,
  hydrateRef,
  hydrateProp,
  hydrateOn,
  hydrateClass,
  hydrateBindValue,
  hydrateBindChecked,
  hydrateAttr,
  collectTextOpens,
  collectElementMarkers
};
