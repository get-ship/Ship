var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);

// packages/ship-set-runtime/src/signal.ts
var CURRENT_OBSERVER = null;
var BATCH_DEPTH = 0;
var PENDING = /* @__PURE__ */ new Set();
function signal(initial) {
  let value = initial;
  const subs = /* @__PURE__ */ new Set();
  const sig = (() => {
    if (CURRENT_OBSERVER !== null) subs.add(CURRENT_OBSERVER);
    return value;
  });
  sig.set = (next) => {
    const v = typeof next === "function" ? next(value) : next;
    if (Object.is(v, value)) return;
    value = v;
    if (BATCH_DEPTH > 0) {
      for (const s of subs) PENDING.add(s);
    } else {
      for (const s of [...subs]) s();
    }
  };
  sig.__subs = subs;
  sig.__peek = () => value;
  return sig;
}
function derived(fn) {
  let value;
  let dirty = true;
  const subs = /* @__PURE__ */ new Set();
  let trackedSigs = [];
  const recompute = () => {
    const prev = CURRENT_OBSERVER;
    CURRENT_OBSERVER = invalidate;
    const oldTracked = trackedSigs;
    const newTracked = [];
    const collector = () => invalidate();
    CURRENT_OBSERVER = collector;
    try {
      value = fn();
    } finally {
      CURRENT_OBSERVER = prev;
    }
    for (const t of oldTracked) t.__subs.delete(invalidate);
    trackedSigs = newTracked;
    dirty = false;
  };
  const invalidate = () => {
    if (dirty) return;
    dirty = true;
    for (const s of [...subs]) s();
  };
  const reader = (() => {
    if (CURRENT_OBSERVER !== null) subs.add(CURRENT_OBSERVER);
    if (dirty) recompute();
    return value;
  });
  reader.set = () => {
    throw new Error("cannot set on a derived; mutate its dependencies instead");
  };
  reader.__subs = subs;
  reader.__peek = () => {
    if (dirty) recompute();
    return value;
  };
  return reader;
}
function effect(fn) {
  let cleanup = void 0;
  let active = true;
  const run = () => {
    if (!active) return;
    if (cleanup) {
      try {
        cleanup();
      } catch (e) {
        console.error("[ship-set] effect cleanup threw:", e);
      }
      cleanup = void 0;
    }
    const prev = CURRENT_OBSERVER;
    CURRENT_OBSERVER = run;
    try {
      cleanup = fn();
    } catch (e) {
      console.error("[ship-set] effect threw:", e);
    } finally {
      CURRENT_OBSERVER = prev;
    }
  };
  run();
  return () => {
    active = false;
    if (cleanup) {
      try {
        cleanup();
      } catch (e) {
        console.error("[ship-set] effect cleanup threw:", e);
      }
    }
  };
}
function batch(fn) {
  BATCH_DEPTH++;
  try {
    fn();
  } finally {
    BATCH_DEPTH--;
    if (BATCH_DEPTH === 0) {
      const subs = [...PENDING];
      PENDING.clear();
      for (const s of subs) s();
    }
  }
}
function untrack(fn) {
  const prev = CURRENT_OBSERVER;
  CURRENT_OBSERVER = null;
  try {
    return fn();
  } finally {
    CURRENT_OBSERVER = prev;
  }
}

// packages/ship-set-runtime/src/bind.ts
function bindText(node, get) {
  return effect(() => {
    const v = get();
    node.textContent = v == null ? "" : String(v);
  });
}
function bindShow(el, get) {
  let prev = null;
  return effect(() => {
    if (get()) {
      if (prev !== null) el.style.display = prev;
      prev = null;
    } else {
      if (prev === null) prev = el.style.display;
      el.style.display = "none";
    }
  });
}
function bindAttr(el, name, get) {
  return effect(() => {
    const v = get();
    if (v == null || v === false || v === "") {
      el.removeAttribute(name);
    } else if (v === true) {
      el.setAttribute(name, "");
    } else {
      el.setAttribute(name, String(v));
    }
  });
}
function bindClass(el, name, get) {
  return effect(() => {
    if (get()) el.classList.add(name);
    else el.classList.remove(name);
  });
}
function bindProp(el, prop2, get) {
  return effect(() => {
    ;
    el[prop2] = get();
  });
}
function bindValue(el, sig, onWrite) {
  const dispose = effect(() => {
    const v = sig();
    if (el.value !== v) el.value = v;
  });
  const onInput = () => {
    const v = el.value;
    sig.set(v);
    if (onWrite) onWrite(v);
  };
  el.addEventListener("input", onInput);
  return () => {
    dispose();
    el.removeEventListener("input", onInput);
  };
}
function bindChecked(el, sig) {
  const dispose = effect(() => {
    const v = sig();
    if (el.checked !== v) el.checked = v;
  });
  const onChange = () => sig.set(el.checked);
  el.addEventListener("change", onChange);
  return () => {
    dispose();
    el.removeEventListener("change", onChange);
  };
}

// packages/ship-set-runtime/src/on.ts
function on(el, event, handler, opts = {}) {
  const modifiers = opts.modifiers ?? [];
  let timer;
  let throttleArmed = true;
  const wrapped = (e) => {
    for (const m of modifiers) {
      switch (m.kind) {
        case "preventDefault":
          e.preventDefault();
          break;
        case "stopPropagation":
          e.stopPropagation();
          break;
        case "stopImmediatePropagation":
          e.stopImmediatePropagation();
          break;
        case "self":
          if (e.target !== el) return;
          break;
        case "trusted":
          if (!e.isTrusted) return;
          break;
        case "key": {
          if (!matchesKey(e, m.arg ?? "")) return;
          break;
        }
      }
    }
    const debounceMod = modifiers.find((m) => m.kind === "debounce");
    const throttleMod = modifiers.find((m) => m.kind === "throttle");
    if (debounceMod) {
      const ms = Number(debounceMod.arg) || 0;
      if (timer !== void 0) clearTimeout(timer);
      timer = setTimeout(() => {
        timer = void 0;
        handler(e);
      }, ms);
      return;
    }
    if (throttleMod) {
      const ms = Number(throttleMod.arg) || 0;
      if (!throttleArmed) return;
      throttleArmed = false;
      handler(e);
      timer = setTimeout(() => {
        throttleArmed = true;
      }, ms);
      return;
    }
    handler(e);
  };
  const listenerOpts = {};
  if (opts.capture) listenerOpts.capture = true;
  if (opts.passive) listenerOpts.passive = true;
  if (opts.once) listenerOpts.once = true;
  el.addEventListener(event, wrapped, listenerOpts);
  return () => {
    el.removeEventListener(event, wrapped, listenerOpts);
    if (timer !== void 0) clearTimeout(timer);
  };
}
function matchesKey(e, spec) {
  const parts = spec.split("+").map((p) => p.toLowerCase());
  const key = parts.pop() ?? "";
  const wantMeta = parts.includes("meta") || parts.includes("cmd");
  const wantCtrl = parts.includes("ctrl") || parts.includes("control");
  const wantShift = parts.includes("shift");
  const wantAlt = parts.includes("alt") || parts.includes("option");
  const wantMod = parts.includes("mod");
  const isMac = typeof navigator !== "undefined" && /Mac|iPhone|iPad|iPod/i.test(navigator.platform || navigator.userAgent || "");
  if (wantMeta && !e.metaKey) return false;
  if (wantCtrl && !e.ctrlKey) return false;
  if (wantShift && !e.shiftKey) return false;
  if (wantAlt && !e.altKey) return false;
  if (wantMod && !(isMac ? e.metaKey : e.ctrlKey)) return false;
  return e.key.toLowerCase() === key;
}

// packages/ship-set-runtime/src/patch.ts
function applyPatch(sig, patch) {
  const next = applyOp(sig.__peek(), patch);
  sig.set(next);
}
function applyPatches(sig, patches) {
  if (patches.length === 0) return;
  batch(() => {
    for (const p of patches) applyPatch(sig, p);
  });
}
function tokenizePath(path) {
  if (!path) return [];
  const tokens = [];
  const re = /([A-Za-z_$][\w$]*)|\[(\d+)\]/g;
  let m;
  let lastEnd = 0;
  while ((m = re.exec(path)) !== null) {
    if (m.index !== lastEnd && lastEnd !== 0 && path[lastEnd] !== ".") {
      throw new Error(`malformed path at byte ${lastEnd}: ${path}`);
    }
    if (m[1] !== void 0) tokens.push({ kind: "field", name: m[1] });
    else if (m[2] !== void 0) tokens.push({ kind: "index", i: Number(m[2]) });
    lastEnd = m.index + m[0].length;
  }
  if (lastEnd !== path.length) {
    throw new Error(`malformed path tail: ${path}`);
  }
  return tokens;
}
function applyOp(value, p) {
  const path = tokenizePath(p.path ?? "");
  switch (p.op) {
    case "set":
      return setAt(value, path, p.value);
    case "merge":
      return mergeAt(value, path, p.value);
    case "append":
      return arrayMod(value, path, (arr) => [...arr, p.value]);
    case "prepend":
      return arrayMod(value, path, (arr) => [p.value, ...arr]);
    case "insert": {
      const i = p.index ?? 0;
      return arrayMod(value, path, (arr) => {
        const next = arr.slice();
        next.splice(i, 0, p.value);
        return next;
      });
    }
    case "remove":
      return removeAt(value, path);
    case "move": {
      const from = p.from ?? 0;
      const to = p.to ?? 0;
      return arrayMod(value, path, (arr) => {
        if (from < 0 || from >= arr.length) return arr;
        const next = arr.slice();
        const [item] = next.splice(from, 1);
        next.splice(to, 0, item);
        return next;
      });
    }
    case "splice": {
      const i = p.index ?? 0;
      const count = p.count ?? 0;
      const inserts = Array.isArray(p.value) ? p.value : [];
      return arrayMod(value, path, (arr) => {
        const next = arr.slice();
        next.splice(i, count, ...inserts);
        return next;
      });
    }
    case "inc": {
      const by = p.by ?? 1;
      return mapAt(value, path, (v) => typeof v === "number" ? v + by : v);
    }
    default:
      throw new Error(`unknown patch op: ${p.op}`);
  }
}
function setAt(root, path, value) {
  if (path.length === 0) return value;
  return rebuild(root, path, () => value);
}
function mergeAt(root, path, partial) {
  return rebuild(root, path, (current) => {
    if (current === null || typeof current !== "object" || Array.isArray(current)) {
      throw new Error("merge requires an object at path");
    }
    if (partial === null || typeof partial !== "object" || Array.isArray(partial)) {
      throw new Error("merge value must be an object");
    }
    return { ...current, ...partial };
  });
}
function removeAt(root, path) {
  if (path.length === 0) return void 0;
  const parentPath = path.slice(0, -1);
  const last = path[path.length - 1];
  return rebuild(root, parentPath, (parent) => {
    if (last.kind === "index") {
      if (!Array.isArray(parent)) throw new Error("remove[index] requires array");
      const next = parent.slice();
      next.splice(last.i, 1);
      return next;
    } else {
      if (parent === null || typeof parent !== "object" || Array.isArray(parent)) {
        throw new Error("remove field requires an object");
      }
      const next = { ...parent };
      delete next[last.name];
      return next;
    }
  });
}
function arrayMod(root, path, mod) {
  return rebuild(root, path, (current) => {
    if (!Array.isArray(current)) throw new Error("expected array at path");
    return mod(current);
  });
}
function mapAt(root, path, fn) {
  return rebuild(root, path, fn);
}
function rebuild(root, path, mut) {
  if (path.length === 0) return mut(root);
  const head = path[0];
  const tail = path.slice(1);
  if (head.kind === "index") {
    if (!Array.isArray(root)) throw new Error("expected array at path");
    const arr = root;
    const next = arr.slice();
    next[head.i] = rebuild(arr[head.i], tail, mut);
    return next;
  } else {
    if (root === null || typeof root !== "object" || Array.isArray(root)) {
      throw new Error("expected object at path");
    }
    const obj = root;
    return { ...obj, [head.name]: rebuild(obj[head.name], tail, mut) };
  }
}

// packages/ship-set-runtime/src/for.ts
function bindFor(opts) {
  const { anchor, items, key, render } = opts;
  let prev = [];
  let active = true;
  const dispose = effect(() => {
    if (!active) return;
    const next = items();
    const nextKeys = /* @__PURE__ */ new Map();
    for (let i = 0; i < next.length; i++) {
      const item = next[i];
      const k = key(item, i);
      nextKeys.set(k, { item, index: i });
    }
    const prevByKey = /* @__PURE__ */ new Map();
    for (const r of prev) prevByKey.set(r.key, r);
    for (const r of prev) {
      if (!nextKeys.has(r.key)) {
        for (const n of r.nodes) {
          try {
            n.parentNode?.removeChild(n);
          } catch {
          }
        }
        r.cleanup?.();
      }
    }
    let cursor = anchor;
    const newRecords = [];
    for (let i = 0; i < next.length; i++) {
      const item = next[i];
      const k = key(item, i);
      const existing = prevByKey.get(k);
      let record;
      if (existing) {
        existing.item = item;
        existing.update?.(item, i);
        record = existing;
      } else {
        const r = render(item, i);
        record = {
          key: k,
          item,
          nodes: r.nodes,
          ...r.cleanup && { cleanup: r.cleanup },
          ...r.update && { update: r.update }
        };
      }
      const parent = anchor.parentNode;
      if (!parent) {
        continue;
      }
      for (const n of record.nodes) {
        const desired = cursor.nextSibling;
        if (n !== desired) {
          parent.insertBefore(n, desired);
        }
        cursor = n;
      }
      newRecords.push(record);
    }
    prev = newRecords;
  });
  return () => {
    active = false;
    dispose();
    for (const r of prev) {
      for (const n of r.nodes) {
        try {
          n.parentNode?.removeChild(n);
        } catch {
        }
      }
      r.cleanup?.();
    }
    prev = [];
  };
}
function bindForSignal(anchor, sig, key, render) {
  return bindFor({ anchor, items: () => sig(), key, render });
}

// packages/ship-set-runtime/src/if.ts
function bindIf(opts) {
  const { anchor } = opts;
  const { test, branches } = isMulti(opts) ? { test: opts.test, branches: opts.branches } : twoBranchToMulti(opts);
  let active = null;
  let lastIdx = -2;
  const tearDown = () => {
    if (!active) return;
    for (const n of active.nodes) {
      try {
        n.parentNode?.removeChild(n);
      } catch {
      }
    }
    active.cleanup?.();
    active = null;
  };
  const dispose = effect(() => {
    const idx = test();
    if (idx === lastIdx) return;
    tearDown();
    lastIdx = idx;
    if (idx < 0 || idx >= branches.length) {
      active = null;
      return;
    }
    active = branches[idx]();
    const parent = anchor.parentNode;
    if (!parent) return;
    for (const n of active.nodes) {
      parent.insertBefore(n, anchor);
    }
  });
  return () => {
    dispose();
    tearDown();
  };
}
function isMulti(o) {
  return typeof o.test === "function";
}
function twoBranchToMulti(o) {
  const { cond, then_, else_ } = o;
  const branches = else_ ? [then_, else_] : [then_];
  return {
    test: () => cond() ? 0 : else_ ? 1 : -1,
    branches
  };
}

// packages/ship-set-runtime/src/await.ts
function bindAwait(opts) {
  const { anchor, promise, loading, then_, catch_ } = opts;
  let active = null;
  let token = 0;
  const tearDown = () => {
    if (!active) return;
    for (const n of active.nodes) {
      try {
        n.parentNode?.removeChild(n);
      } catch {
      }
    }
    active.cleanup?.();
    active = null;
  };
  const insert = (br) => {
    active = br;
    const parent = anchor.parentNode;
    if (!parent) return;
    for (const n of br.nodes) parent.insertBefore(n, anchor);
  };
  const dispose = effect(() => {
    const myToken = ++token;
    const p = promise();
    tearDown();
    insert(loading());
    p.then(
      (v) => {
        if (myToken !== token) return;
        if (then_) {
          tearDown();
          insert(then_(v));
        }
      },
      (e) => {
        if (myToken !== token) return;
        if (catch_) {
          tearDown();
          insert(catch_(e));
        }
      }
    );
  });
  return () => {
    token++;
    dispose();
    tearDown();
  };
}

// packages/ship-set-runtime/src/raw-html.ts
function bindRawHtml(el, get, deps = []) {
  return effect(() => {
    for (const d of deps) d();
    el.innerHTML = String(get() ?? "");
  });
}

// packages/ship-set-runtime/src/morph.ts
function morph(target, source, opts = {}) {
  morphElement(target, source, opts);
}
function morphElement(target, source, opts) {
  if (opts.skip?.(target)) return;
  if (target.tagName !== source.tagName) {
    target.replaceWith(source.cloneNode(true));
    return;
  }
  morphAttributes(target, source);
  morphChildren(target, source, opts);
}
function morphAttributes(target, source) {
  const tAttrs = Array.from(target.attributes);
  for (const a of tAttrs) {
    if (!source.hasAttribute(a.name)) target.removeAttribute(a.name);
  }
  const sAttrs = Array.from(source.attributes);
  for (const a of sAttrs) {
    if (target.getAttribute(a.name) !== a.value) {
      target.setAttribute(a.name, a.value);
    }
  }
  if (target instanceof HTMLInputElement && source instanceof HTMLInputElement) {
    if (source.hasAttribute("value") && target.value !== source.value) {
      target.value = source.value;
    }
    if (target.checked !== source.checked) {
      target.checked = source.checked;
    }
  }
}
function morphChildren(target, source, opts) {
  const tChildren = Array.from(target.childNodes);
  const sChildren = Array.from(source.childNodes);
  const max = Math.max(tChildren.length, sChildren.length);
  for (let i = 0; i < max; i++) {
    const t = tChildren[i];
    const s = sChildren[i];
    if (!s && t) {
      target.removeChild(t);
      continue;
    }
    if (!t && s) {
      target.appendChild(s.cloneNode(true));
      continue;
    }
    if (!t || !s) continue;
    morphNode(t, s, target, opts);
  }
}
function morphNode(t, s, parent, opts) {
  if (t.nodeType !== s.nodeType) {
    parent.replaceChild(s.cloneNode(true), t);
    return;
  }
  if (t.nodeType === Node.TEXT_NODE || t.nodeType === Node.COMMENT_NODE) {
    if (t.nodeValue !== s.nodeValue) t.nodeValue = s.nodeValue;
    return;
  }
  if (t.nodeType === Node.ELEMENT_NODE) {
    morphElement(t, s, opts);
  }
}

// packages/ship-set-runtime/src/slot.ts
function resolveSlots(host, slots, originalChildren) {
  const fills = /* @__PURE__ */ new Map();
  for (const child of originalChildren) {
    let name = "default";
    if (child instanceof Element) {
      name = child.getAttribute("slot") ?? "default";
      if (child.hasAttribute("slot")) child.removeAttribute("slot");
    }
    const list = fills.get(name) ?? [];
    list.push(child);
    fills.set(name, list);
  }
  const filledNames = /* @__PURE__ */ new Set();
  for (const [name, nodes] of fills) {
    const hasMeaning = nodes.some((n) => {
      if (n.nodeType === 1) return true;
      if (n.nodeType === 3) return (n.textContent || "").trim().length > 0;
      return false;
    });
    if (hasMeaning) filledNames.add(name);
  }
  ;
  host.__shipSetSlotFilled = filledNames;
  for (const slot of slots) {
    if (!slot || !slot.anchor) continue;
    const filled = fills.get(slot.name) ?? [];
    const parent = slot.anchor.parentNode;
    if (!parent) continue;
    if (filled.length > 0) {
      for (const n of filled) parent.insertBefore(n, slot.anchor);
    } else if (slot.fallback) {
      const nodes = slot.fallback();
      for (const n of nodes) parent.insertBefore(n, slot.anchor);
    }
  }
  for (const slot of slots) {
    if (slot && slot.anchor) slot.anchor.remove();
  }
}
function slotFilled(host, name) {
  const set = host.__shipSetSlotFilled;
  return set?.has(name) ?? false;
}
function snapshotChildren(host) {
  return Array.from(host.childNodes);
}

// packages/ship-set-runtime/src/decode.ts
var decodeString = (raw) => raw ?? "";
var decodeNumber = (raw) => {
  if (raw === null || raw === "") return NaN;
  return Number(raw);
};
var decodeBoolean = (raw) => {
  if (raw === null) return false;
  const v = raw.toLowerCase();
  if (v === "false") return false;
  return true;
};
function decodeJson(fallback) {
  return (raw) => {
    if (raw === null || raw === "") return fallback;
    try {
      return JSON.parse(raw);
    } catch {
      return fallback;
    }
  };
}
function decodeEnum(allowed, fallback) {
  const set = new Set(allowed);
  return (raw) => raw !== null && set.has(raw) ? raw : fallback;
}

// packages/ship-set-runtime/src/component-factory.ts
var prop = {
  /** String-valued prop. Default `''`. */
  string(opts = {}) {
    return { type: "string", ...opts };
  },
  /** Boolean prop using HTML attribute-presence semantics. */
  bool(opts = {}) {
    return { type: "bool", ...opts };
  },
  /** Numeric prop. Empty/missing decode to `NaN`. */
  number(opts = {}) {
    return { type: "number", ...opts };
  },
  /**
   * Enum-style prop. Values outside `values` (including missing) decode
   * to `default` if provided, otherwise the first allowed value.
   */
  union(values, opts = {}) {
    return { type: "union", values, ...opts };
  },
  /** JSON-parsed object prop. Bad/missing input falls back to `default`. */
  object(opts = {}) {
    return { type: "object", ...opts };
  },
  /** JSON-parsed array prop. Bad/missing input falls back to `default ?? []`. */
  array(opts = {}) {
    return { type: "array", ...opts };
  }
};
var decoderFor = (def) => {
  switch (def.type) {
    case "string":
      return decodeString;
    case "bool":
      return decodeBoolean;
    case "number":
      return decodeNumber;
    case "union": {
      const values = def.values ?? [];
      const fallback = def.default ?? values[0] ?? "";
      return decodeEnum(values, fallback);
    }
    case "object":
      return decodeJson(def.default);
    case "array":
      return decodeJson(def.default ?? []);
  }
};
var camelOf = (attr) => attr.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
var encodeAttr = (def, value) => {
  if (value === void 0 || value === null) return null;
  switch (def.type) {
    case "string":
      return value === "" ? null : String(value);
    case "number":
      return Number.isNaN(value) ? null : String(value);
    case "bool":
      return value ? "" : null;
    case "union":
      return String(value);
    case "object":
    case "array":
      try {
        return JSON.stringify(value);
      } catch {
        return null;
      }
  }
};
var injectScopedStyles = (tag, css) => {
  const id = `ship-set-style-${tag}`;
  if (document.getElementById(id)) return;
  const style = document.createElement("style");
  style.id = id;
  style.textContent = `[data-component="${tag}"] { ${css} }`;
  document.head.appendChild(style);
};
var seedSignal = (host, attr, def, decode) => {
  const raw = host.getAttribute(attr);
  const initial = raw !== null ? decode(raw) : def.default;
  return signal(initial);
};
var buildSchema = (def) => {
  const observedAttrs = [];
  const props = {};
  for (const [attr, propDef] of Object.entries(def.props)) {
    observedAttrs.push(attr);
    props[attr] = { camel: camelOf(attr), decode: decoderFor(propDef) };
  }
  const schema = { observedAttrs, props };
  if (def.formAssociated) schema.formAssociated = true;
  return schema;
};
function component(tag, def) {
  const schema = buildSchema(def);
  const propEntries = Object.entries(def.props);
  defineComponent(tag, schema, (host, propSignals, internals) => {
    let shadowAttached = null;
    if (def.shadow) {
      shadowAttached = host.shadowRoot ?? host.attachShadow({ mode: "open" });
      if (def.styles) {
        const style = document.createElement("style");
        style.textContent = def.styles;
        shadowAttached.appendChild(style);
      }
    } else if (def.styles) {
      injectScopedStyles(tag, def.styles);
    }
    const cleanups = [];
    for (const [attr, propDef] of propEntries) {
      const camel = camelOf(attr);
      const decode = schema.props[attr].decode;
      const sig = seedSignal(host, attr, propDef, decode);
      propSignals[camel] = sig;
      if (propDef.reflect) {
        let first = true;
        const dispose = effect(() => {
          const v = sig();
          if (first) {
            first = false;
            return;
          }
          const next = encodeAttr(propDef, v);
          if (next === null) host.removeAttribute(attr);
          else host.setAttribute(attr, next);
        });
        cleanups.push(dispose);
      }
    }
    if (def.setup) {
      const ctx = { props: propSignals, host, cleanups };
      if (internals) ctx.internals = internals;
      def.setup(ctx);
    }
    return () => {
      for (const c of cleanups.splice(0)) {
        try {
          c();
        } catch (e) {
          console.error(`[ship-set:${tag}] cleanup threw:`, e);
        }
      }
    };
  });
}

// packages/ship-set-runtime/src/component.ts
function defineComponent(tag, schema, factory) {
  if (customElements.get(tag)) return;
  if (schema.formAssociated) {
    class ShipSetFormElement extends HTMLElement {
      constructor() {
        super();
        __publicField(this, "cleanup");
        __publicField(this, "propSignals", {});
        __publicField(this, "internals");
        this.internals = this.attachInternals();
      }
      static get observedAttributes() {
        return schema.observedAttrs;
      }
      connectedCallback() {
        this.dataset.component = tag;
        this.cleanup = factory(this, this.propSignals, this.internals);
      }
      disconnectedCallback() {
        try {
          this.cleanup?.();
        } catch (e) {
          console.error(`[ship-set:${tag}] cleanup threw:`, e);
        }
        this.cleanup = void 0;
      }
      attributeChangedCallback(attrName, _old, value) {
        const meta = schema.props[attrName];
        if (!meta) return;
        const sig = this.propSignals[meta.camel];
        if (sig) sig.set(meta.decode(value));
      }
      // Form-association lifecycle hooks. The runtime exposes them via
      // user-supplied `internals.*` calls in mount/effect blocks; the
      // browser also dispatches these on the host so authors who want
      // to override (e.g. for restore) can attach listeners directly.
      formResetCallback() {
        this.dispatchEvent(new CustomEvent("ship-set:form-reset"));
      }
      formStateRestoreCallback(state, mode) {
        this.dispatchEvent(
          new CustomEvent("ship-set:form-restore", { detail: { state, mode } })
        );
      }
      formDisabledCallback(disabled) {
        this.dispatchEvent(
          new CustomEvent("ship-set:form-disabled", { detail: { disabled } })
        );
      }
    }
    __publicField(ShipSetFormElement, "formAssociated", true);
    customElements.define(tag, ShipSetFormElement);
    return;
  }
  class ShipSetElement extends HTMLElement {
    constructor() {
      super(...arguments);
      __publicField(this, "cleanup");
      __publicField(this, "propSignals", {});
    }
    static get observedAttributes() {
      return schema.observedAttrs;
    }
    connectedCallback() {
      this.dataset.component = tag;
      this.cleanup = factory(this, this.propSignals);
    }
    disconnectedCallback() {
      try {
        this.cleanup?.();
      } catch (e) {
        console.error(`[ship-set:${tag}] cleanup threw:`, e);
      }
      this.cleanup = void 0;
    }
    attributeChangedCallback(attrName, _old, value) {
      const meta = schema.props[attrName];
      if (!meta) return;
      const sig = this.propSignals[meta.camel];
      if (sig) sig.set(meta.decode(value));
    }
  }
  customElements.define(tag, ShipSetElement);
}
function camelOf2(attr) {
  return attr.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
}

// packages/ship-set-runtime/src/data.ts
function data(opts) {
  const sig = signal(opts.initial);
  const ESCtor = opts.EventSource ?? globalThis.EventSource;
  if (!opts.subscribe || !ESCtor) {
    return { signal: sig, dispose: () => {
    } };
  }
  const sse = new ESCtor(opts.subscribe);
  const onMessage = (ev) => {
    let patch;
    try {
      patch = JSON.parse(ev.data);
    } catch (err) {
      opts.onError?.(err);
      return;
    }
    try {
      if (opts.onPatch) opts.onPatch(patch, sig);
      else applyPatch(sig, patch);
    } catch (err) {
      opts.onError?.(err);
    }
  };
  const onError = (err) => opts.onError?.(err);
  sse.addEventListener("patch", onMessage);
  sse.addEventListener("error", onError);
  return {
    signal: sig,
    dispose: () => {
      sse.removeEventListener("patch", onMessage);
      sse.removeEventListener("error", onError);
      sse.close();
    }
  };
}

// packages/ship-set-runtime/src/state.ts
function debounce(input, ms) {
  const out = signal(untrack(() => input()));
  let timer = null;
  effect(() => {
    const value = input();
    if (timer !== null) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      out.set(value);
    }, ms);
    return () => {
      if (timer !== null) {
        clearTimeout(timer);
        timer = null;
      }
    };
  });
  return out;
}

// packages/ship-set-runtime/src/live.ts
var toError = (err) => err instanceof Error ? err : new Error(String(err));
var isLiveSource = (v) => typeof v === "object" && v !== null && typeof v.subscribe === "function";
function live(source, opts) {
  const sig = signal({ kind: "loading" });
  let token = 0;
  effect(() => {
    const myToken = ++token;
    let result;
    try {
      result = source();
    } catch (err) {
      if (myToken !== token) return;
      const error = toError(err);
      sig.set({ kind: "error", error });
      opts?.onError?.(err);
      return;
    }
    sig.set({ kind: "loading" });
    if (isLiveSource(result)) {
      let disposed = false;
      const unsub = result.subscribe(sig);
      return () => {
        if (disposed) return;
        disposed = true;
        try {
          unsub();
        } catch (err) {
          opts?.onError?.(err);
        }
      };
    }
    Promise.resolve(result).then(
      (value) => {
        if (myToken !== token) return;
        sig.set({ kind: "ok", value });
      },
      (err) => {
        if (myToken !== token) return;
        const error = toError(err);
        sig.set({ kind: "error", error });
        opts?.onError?.(err);
      }
    );
    return void 0;
  });
  return sig;
}
function unwrap(sig) {
  const s = sig();
  return s.kind === "ok" ? s.value : void 0;
}
function unwrapOr(sig, fallback) {
  const s = sig();
  return s.kind === "ok" ? s.value : fallback;
}

// packages/ship-set-runtime/src/while.ts
var tearDownBranch = (active) => {
  if (!active) return null;
  for (const n of active.nodes) {
    try {
      n.parentNode?.removeChild(n);
    } catch {
    }
  }
  active.cleanup?.();
  return null;
};
var insertBranch = (anchor, br) => {
  const parent = anchor.parentNode;
  if (!parent) return;
  for (const n of br.nodes) parent.insertBefore(n, anchor);
};
function bindWhileLoading(opts) {
  const { anchor, signal: sig, render } = opts;
  let active = null;
  let mounted = false;
  const dispose = effect(() => {
    const want = sig().kind === "loading";
    if (want === mounted) return;
    mounted = want;
    if (want) {
      active = render();
      insertBranch(anchor, active);
    } else {
      active = tearDownBranch(active);
    }
  });
  return () => {
    dispose();
    active = tearDownBranch(active);
  };
}
function bindWhileError(opts) {
  const { anchor, signal: sig, render } = opts;
  let active = null;
  let mounted = false;
  const dispose = effect(() => {
    const v = sig();
    const want = v.kind === "error";
    if (want === mounted) return;
    mounted = want;
    if (want && v.kind === "error") {
      active = render(v.error);
      insertBranch(anchor, active);
    } else {
      active = tearDownBranch(active);
    }
  });
  return () => {
    dispose();
    active = tearDownBranch(active);
  };
}

// packages/ship-set-runtime/src/controller-hook-resolver.ts
function resolveHook(hook, ctx, outerScope) {
  const hookState = resolveHookState(hook.state, { ship: ctx.ship, ...outerScope });
  const stateSignals = {};
  for (const [k, v] of Object.entries(hookState)) {
    stateSignals[k] = signal(v);
  }
  const baseScope = {
    ship: ctx.ship,
    route: ctx.route ?? {},
    navigate: ctx.navigate ?? ((_path) => {
    }),
    ...outerScope,
    ...stateSignals
  };
  const writableProxy = buildHookWritableProxy(stateSignals, baseScope);
  const declScope = { ...baseScope };
  const deriveds = {};
  for (const [k, fn] of Object.entries(hook.derived ?? {})) {
    deriveds[k] = derived(() => fn(declScope));
  }
  Object.assign(declScope, deriveds);
  const onHandlers = {};
  for (const [k, fn] of Object.entries(hook.on ?? {})) {
    onHandlers[k] = (...args) => fn(writableProxy, ...args);
  }
  const useResults = {};
  const useSection = typeof hook.use === "function" ? hook.use(declScope) : hook.use ?? {};
  for (const [k, sub] of Object.entries(useSection)) {
    useResults[k] = resolveHook(sub, ctx, declScope);
  }
  return {
    ...stateSignals,
    ...deriveds,
    ...onHandlers,
    ...useResults
  };
}
function resolveHookState(section, scope) {
  if (section === void 0) return {};
  if (typeof section === "function") {
    return section(scope);
  }
  return section;
}
function buildHookWritableProxy(stateSignals, baseScope) {
  return new Proxy({}, {
    get(_target, key) {
      if (key in stateSignals) return stateSignals[key]();
      return baseScope[key];
    },
    set(_target, key, value) {
      if (key in stateSignals) {
        stateSignals[key].set(value);
        return true;
      }
      console.warn(
        `[ship-set] hook handler tried to write s.${key} = \u2026 but ${key} is not a hook state key. The assignment was ignored.`
      );
      return true;
    },
    has(_target, key) {
      return key in stateSignals || key in baseScope;
    },
    ownKeys() {
      return Object.keys(stateSignals);
    },
    getOwnPropertyDescriptor(_target, key) {
      if (key in stateSignals) {
        return { configurable: true, enumerable: true, writable: true, value: stateSignals[key]() };
      }
      return void 0;
    }
  });
}

// packages/ship-set-runtime/src/controller-resolver.ts
var LIFECYCLE_KEY = "__shipSetLifecycle";
function resolveController(def, ctx) {
  const shape = normalize(def, ctx);
  const baseScope = {
    ship: ctx.ship,
    route: ctx.route ?? {},
    navigate: ctx.navigate ?? ((_path) => {
    }),
    ...ctx.scope ?? {}
  };
  const stateSignals = pickStateSignals(ctx.scope ?? {}, ctx.stateKeys);
  const writableProxy = buildWritableProxy(stateSignals, baseScope);
  const declScope = { ...baseScope };
  const deriveds = buildDeriveds(shape, declScope);
  Object.assign(declScope, deriveds);
  const useResults = buildUseResults(shape, declScope, ctx);
  for (const [k, v] of Object.entries(useResults)) {
    declScope[k] = v;
  }
  const onHandlers = buildOnHandlers(shape, writableProxy);
  const bindings = {
    ...deriveds,
    ...onHandlers,
    ...useResults
  };
  if (shape.mount || shape.unmount) {
    const lifecycle = {
      state: writableProxy
    };
    if (shape.mount) lifecycle.mount = shape.mount;
    if (shape.unmount) lifecycle.unmount = shape.unmount;
    Object.defineProperty(bindings, LIFECYCLE_KEY, {
      value: lifecycle,
      enumerable: false,
      configurable: true,
      writable: false
    });
  }
  return bindings;
}
function normalize(def, ctx) {
  if (typeof def === "function") {
    return def({ ship: ctx.ship });
  }
  return def;
}
function pickStateSignals(scope, stateKeys) {
  const out = {};
  if (stateKeys) {
    for (const k of stateKeys) {
      const v = scope[k];
      if (isSignal(v)) out[k] = v;
    }
    return out;
  }
  for (const [k, v] of Object.entries(scope)) {
    if (isSignal(v)) out[k] = v;
  }
  return out;
}
function isSignal(v) {
  return typeof v === "function" && "set" in v && typeof v.set === "function";
}
function buildWritableProxy(stateSignals, baseScope) {
  return new Proxy({}, {
    get(_target, key) {
      if (key in stateSignals) return stateSignals[key]();
      return baseScope[key];
    },
    set(_target, key, value) {
      if (key in stateSignals) {
        stateSignals[key].set(value);
        return true;
      }
      console.warn(
        `[ship-set] handler tried to write s.${key} = \u2026 but ${key} is not declared as <state name="${key}"> in the template. The assignment was ignored.`
      );
      return true;
    },
    has(_target, key) {
      return key in stateSignals || key in baseScope;
    },
    ownKeys() {
      return Object.keys(stateSignals);
    },
    getOwnPropertyDescriptor(_target, key) {
      if (key in stateSignals) {
        return { configurable: true, enumerable: true, writable: true, value: stateSignals[key]() };
      }
      return void 0;
    }
  });
}
function buildDeriveds(shape, declScope) {
  const section = shape.derived ?? {};
  const out = {};
  for (const [k, fn] of Object.entries(section)) {
    out[k] = derived(() => fn(declScope));
  }
  return out;
}
function buildOnHandlers(shape, writableProxy) {
  const section = shape.on ?? {};
  const out = {};
  for (const [k, fn] of Object.entries(section)) {
    out[k] = (...args) => fn(writableProxy, ...args);
  }
  return out;
}
function buildUseResults(shape, declScope, ctx) {
  if (shape.use === void 0) return {};
  const useSection = typeof shape.use === "function" ? shape.use(declScope) : shape.use;
  const out = {};
  for (const [k, hookShape] of Object.entries(useSection)) {
    out[k] = resolveHook(hookShape, ctx, declScope);
  }
  return out;
}
function controller(def) {
  return {
    setup: (ctx) => resolveController(def, ctx),
    __def: def
  };
}

// packages/ship-set-runtime/src/hooks/filtered-list.ts
function readItems(source) {
  const v = source();
  if (Array.isArray(v)) return v;
  if (v && typeof v === "object" && "kind" in v) {
    const env = v;
    if (env.kind === "ok" && env.value && Array.isArray(env.value.items)) {
      return env.value.items;
    }
    return [];
  }
  return [];
}
function useFilteredList(source, predicate) {
  return {
    state: { query: "" },
    derived: {
      filtered: ({ query }) => {
        const q = query();
        return readItems(source).filter((i) => predicate(i, q));
      }
    },
    on: {
      setQuery: (s, q) => {
        s.query = q;
      },
      clear: (s) => {
        s.query = "";
      }
    }
  };
}

// packages/ship-set-runtime/src/hooks/selection.ts
function readItems2(source) {
  const v = source();
  if (Array.isArray(v)) return v;
  if (v && typeof v === "object" && "kind" in v) {
    const env = v;
    if (env.kind === "ok" && env.value && Array.isArray(env.value.items)) {
      return env.value.items;
    }
    return [];
  }
  return [];
}
function useSelection(source, keyOf) {
  return {
    state: { selectedId: null },
    derived: {
      selected: ({ selectedId }) => {
        const id = selectedId();
        if (id === null) return null;
        return readItems2(source).find((i) => keyOf(i) === id) ?? null;
      }
    },
    on: {
      select: (s, id) => {
        s.selectedId = id;
      },
      clear: (s) => {
        s.selectedId = null;
      },
      toggle: (s, id) => {
        s.selectedId = s.selectedId === id ? null : id;
      }
    }
  };
}

// packages/ship-set-runtime/src/hooks/form.ts
function resolveInitial(init) {
  return typeof init === "function" ? init() : init;
}
function snapshotValues(state, keys) {
  const out = {};
  for (const k of keys) {
    const v = state[k];
    out[k] = typeof v === "function" ? v() : v;
  }
  return out;
}
function useForm(opts) {
  const init = resolveInitial(opts.initial);
  const fieldKeys = Object.keys(init);
  const initialValues = {
    ...init,
    pending: false,
    lastError: null
  };
  return {
    state: initialValues,
    derived: {
      values: (s) => snapshotValues(s, fieldKeys),
      errors: (s) => {
        const values = snapshotValues(s, fieldKeys);
        const validated = opts.validate ? opts.validate(values) : {};
        const out = {};
        for (const k of fieldKeys) {
          const v = validated[k];
          out[k] = v === void 0 || v === null ? null : v;
        }
        return out;
      },
      valid: (s) => {
        const values = snapshotValues(s, fieldKeys);
        if (!opts.validate) return true;
        const validated = opts.validate(values);
        for (const k of fieldKeys) {
          const v = validated[k];
          if (v !== void 0 && v !== null) return false;
        }
        return true;
      },
      dirty: (s) => {
        for (const k of fieldKeys) {
          const v = s[k]();
          if (!Object.is(v, init[k])) return true;
        }
        return false;
      }
    },
    on: {
      reset: (s) => {
        for (const k of fieldKeys) {
          s[k] = init[k];
        }
        s.pending = false;
        s.lastError = null;
      },
      submit: async (s, event) => {
        if (event && typeof event.preventDefault === "function") {
          event.preventDefault();
        }
        const proxy = s;
        const values = snapshotValues(proxy, fieldKeys);
        if (opts.validate) {
          const validated = opts.validate(values);
          for (const k of fieldKeys) {
            const v = validated[k];
            if (v !== void 0 && v !== null) return;
          }
        }
        if (!opts.submit) return;
        proxy.pending = true;
        proxy.lastError = null;
        try {
          await opts.submit(values);
        } catch (err) {
          proxy.lastError = err instanceof Error ? err : new Error(String(err));
        } finally {
          proxy.pending = false;
        }
      }
    }
  };
}

// packages/ship-set-runtime/src/template-hooks/registry.ts
var registry = /* @__PURE__ */ new Map();
function registerTemplateHook(hook) {
  registry.set(
    hook.name,
    hook
  );
}
function getTemplateHook(name) {
  return registry.get(name);
}
function clearRegistry() {
  registry.clear();
}
function listRegisteredHooks() {
  return Array.from(registry.keys()).sort();
}

// packages/ship-set-runtime/src/template-hooks/kbd.ts
var useKbdHook = {
  name: "kbd",
  attrs: { shortcuts: "literal", resolve: "literal" },
  children: true,
  factory: (ctx) => {
    const { shortcuts, resolve } = ctx.attrs;
    if (!shortcuts || shortcuts.length === 0) return;
    const handler = (event) => {
      for (const sc of shortcuts) {
        if (matchesKey(event, sc.key)) {
          const fn = resolve(sc.action);
          if (fn) fn(event);
          return;
        }
      }
    };
    document.addEventListener("keydown", handler);
    ctx.cleanup(() => document.removeEventListener("keydown", handler));
  }
};

// packages/ship-set-runtime/src/template-hooks/click-outside.ts
var useClickOutsideHook = {
  name: "click-outside",
  attrs: { target: "binding", action: "binding" },
  factory: (ctx) => {
    const { target, action } = ctx.attrs;
    if (!target || !action) return;
    const handler = (event) => {
      const t = event.target;
      if (t instanceof Node && target.contains(t)) return;
      action(event);
    };
    document.addEventListener("mousedown", handler, true);
    ctx.cleanup(() => document.removeEventListener("mousedown", handler, true));
  }
};

// packages/ship-set-runtime/src/template-hooks/shell.ts
var useShellHook = {
  name: "shell",
  attrs: { as: "literal" },
  factory: (_ctx) => {
    return typeof window !== "undefined" && window.__shipShell || {};
  }
};

// packages/ship-set-runtime/src/template-hooks/index.ts
registerTemplateHook(useKbdHook);
registerTemplateHook(useClickOutsideHook);
registerTemplateHook(useShellHook);
export {
  LIFECYCLE_KEY,
  applyPatch,
  applyPatches,
  batch,
  bindAttr,
  bindAwait,
  bindChecked,
  bindClass,
  bindFor,
  bindForSignal,
  bindIf,
  bindProp,
  bindRawHtml,
  bindShow,
  bindText,
  bindValue,
  bindWhileError,
  bindWhileLoading,
  camelOf2 as camelOf,
  clearRegistry,
  component,
  controller,
  data,
  debounce,
  decodeBoolean,
  decodeEnum,
  decodeJson,
  decodeNumber,
  decodeString,
  defineComponent,
  derived,
  effect,
  getTemplateHook,
  listRegisteredHooks,
  live,
  matchesKey,
  morph,
  on,
  prop,
  registerTemplateHook,
  resolveController,
  resolveSlots,
  signal,
  slotFilled,
  snapshotChildren,
  signal as state,
  tokenizePath,
  untrack,
  unwrap,
  unwrapOr,
  useFilteredList,
  useForm,
  useSelection
};
