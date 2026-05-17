var __defProp = Object.defineProperty;
var __defNormalProp = (obj, key, value) => key in obj ? __defProp(obj, key, { enumerable: true, configurable: true, writable: true, value }) : obj[key] = value;
var __publicField = (obj, key, value) => __defNormalProp(obj, typeof key !== "symbol" ? key + "" : key, value);

// ../../packages/sui-runtime/src/error-boundary.ts
var BOUNDARY_KEY = Symbol.for("sui:error-boundary");
function findNearestBoundary(node) {
  let cur = node;
  while (cur) {
    if (cur instanceof Element) {
      const handle = cur[BOUNDARY_KEY];
      if (handle) return handle;
    }
    const parent = cur.parentNode;
    if (parent) {
      cur = parent;
      continue;
    }
    const root = cur;
    if (root.host) {
      cur = root.host;
      continue;
    }
    cur = null;
  }
  return null;
}
function runProtected(node, source, fn) {
  try {
    return fn();
  } catch (err) {
    const handle = findNearestBoundary(node);
    if (handle) {
      handle.catch(err, source);
      return void 0;
    }
    console.error("[sui] uncaught throw (no error boundary):", err);
    return void 0;
  }
}

// ../../packages/sui-runtime/src/signal.ts
var CURRENT_OBSERVER = null;
var CURRENT_UNTRACKS = null;
var BATCH_DEPTH = 0;
var PENDING = /* @__PURE__ */ new Set();
var HOST_STACK = [];
function withHost(host, fn) {
  HOST_STACK.push(host);
  try {
    return fn();
  } finally {
    HOST_STACK.pop();
  }
}
function currentHost() {
  return HOST_STACK.length > 0 ? HOST_STACK[HOST_STACK.length - 1] : null;
}
function track(subs) {
  if (CURRENT_OBSERVER === null) return;
  if (subs.has(CURRENT_OBSERVER)) return;
  const sub = CURRENT_OBSERVER;
  subs.add(sub);
  if (CURRENT_UNTRACKS !== null) {
    CURRENT_UNTRACKS.push(() => subs.delete(sub));
  }
}
function signal(initial) {
  let value = initial;
  const subs = /* @__PURE__ */ new Set();
  const sig = () => {
    track(subs);
    return value;
  };
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
  let untracks = [];
  const invalidate = () => {
    if (dirty) return;
    dirty = true;
    if (subs.size === 0) return;
    const prev = value;
    recompute();
    if (Object.is(prev, value)) return;
    for (const s of [...subs]) s();
  };
  const recompute = () => {
    for (const u of untracks) u();
    untracks = [];
    const prevObs = CURRENT_OBSERVER;
    const prevUnt = CURRENT_UNTRACKS;
    CURRENT_OBSERVER = invalidate;
    CURRENT_UNTRACKS = untracks;
    try {
      value = fn();
    } finally {
      CURRENT_OBSERVER = prevObs;
      CURRENT_UNTRACKS = prevUnt;
    }
    dirty = false;
  };
  const reader = () => {
    track(subs);
    if (dirty) recompute();
    return value;
  };
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
  let untracks = [];
  const host = currentHost();
  const run = () => {
    if (!active) return;
    if (cleanup) {
      try {
        cleanup();
      } catch (e) {
        console.error("[sui] effect cleanup threw:", e);
      }
      cleanup = void 0;
    }
    for (const u of untracks) u();
    untracks = [];
    const prevObs = CURRENT_OBSERVER;
    const prevUnt = CURRENT_UNTRACKS;
    CURRENT_OBSERVER = run;
    CURRENT_UNTRACKS = untracks;
    try {
      cleanup = fn();
    } catch (e) {
      const boundary = host ? findNearestBoundary(host) : null;
      if (boundary) {
        boundary.catch(e, "effect");
      } else {
        console.error("[sui] effect threw:", e);
      }
    } finally {
      CURRENT_OBSERVER = prevObs;
      CURRENT_UNTRACKS = prevUnt;
    }
  };
  run();
  return () => {
    active = false;
    for (const u of untracks) u();
    untracks = [];
    if (cleanup) {
      try {
        cleanup();
      } catch (e) {
        console.error("[sui] effect cleanup threw:", e);
      }
      cleanup = void 0;
    }
  };
}

// ../../packages/sui-runtime/src/effect-scope.ts
var CURRENT_SCOPE = null;
function withScope(scope, body) {
  const prev = CURRENT_SCOPE;
  CURRENT_SCOPE = scope;
  try {
    return body();
  } finally {
    CURRENT_SCOPE = prev;
  }
}
function onCleanup(fn) {
  const scope = CURRENT_SCOPE;
  if (!scope) return;
  scope.cleanup(fn);
}
function createScope() {
  const cleanups = [];
  let disposed = false;
  const scope = {
    cleanup(fn) {
      if (disposed) {
        try {
          fn();
        } catch (e) {
          console.error("[sui/scope] late cleanup threw:", e);
        }
        return;
      }
      cleanups.push(fn);
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      for (let i = cleanups.length - 1; i >= 0; i--) {
        try {
          cleanups[i]();
        } catch (e) {
          console.error("[sui/scope] cleanup threw:", e);
        }
      }
      cleanups.length = 0;
    },
    get disposed() {
      return disposed;
    }
  };
  return scope;
}

// ../../packages/sui-runtime/src/context.ts
var SHARED = Symbol("sui.shared");
function isShared(sig) {
  return !!sig?.[SHARED];
}
function provide(host, key, value) {
  ;
  host[`__sui_ctx_${key}`] = value;
}
function pickShared(scope) {
  const out = {};
  for (const k in scope) {
    if (Object.prototype.hasOwnProperty.call(scope, k) && isShared(scope[k])) {
      out[k] = scope[k];
    }
  }
  return out;
}

// ../../packages/sui-runtime/src/bind.ts
function bindText(node, get) {
  let prev;
  return effect(() => {
    const v = get();
    const next = v == null ? "" : String(v);
    if (next === prev) return;
    if (prev === void 0 && node.textContent === next) {
      prev = next;
      return;
    }
    node.textContent = next;
    prev = next;
  });
}
function bindClass(el, name, get) {
  let prev;
  return effect(() => {
    const v = !!get();
    if (v === prev) return;
    if (prev === void 0 && v === el.classList.contains(name)) {
      prev = v;
      return;
    }
    if (v) el.classList.add(name);
    else el.classList.remove(name);
    prev = v;
  });
}

// ../../packages/sui-runtime/src/on.ts
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

// ../../packages/sui-runtime/src/host-api.ts
function emit(host, name, detail) {
  host.dispatchEvent(new CustomEvent(name, { detail, bubbles: true, composed: true }));
}
function expose(host, methods) {
  const target = host;
  const names = Object.keys(methods);
  for (const name of names) target[name] = methods[name];
  onCleanup(() => {
    for (const name of names) {
      if (target[name] === methods[name]) {
        try {
          delete target[name];
        } catch {
          target[name] = void 0;
        }
      }
    }
  });
}
function attachHostApi(host) {
  const target = host;
  target.emit = function(name, detail) {
    emit(host, name, detail);
  };
  target.expose = function(methods) {
    expose(host, methods);
  };
}

// ../../packages/sui-runtime/src/transition-runner.ts
var _runEnter = () => {
};
var _runLeave = () => null;
var _cancelTransitions = () => {
};
var runEnter = (nodes) => _runEnter(nodes);
var runLeave = (nodes) => _runLeave(nodes);
var cancelTransitions = (node) => _cancelTransitions(node);

// ../../packages/sui-runtime/src/for.ts
function computeLisMaskInto(matched, count, mask, pred, tails) {
  mask.fill(0, 0, count);
  if (count === 0) return;
  let tailsLen = 0;
  for (let i = 0; i < count; i++) {
    const v = matched[2 * i + 1];
    let lo = 0, hi = tailsLen;
    while (lo < hi) {
      const mid = lo + hi >>> 1;
      if (matched[2 * tails[mid] + 1] < v) lo = mid + 1;
      else hi = mid;
    }
    pred[i] = lo > 0 ? tails[lo - 1] : -1;
    tails[lo] = i;
    if (lo === tailsLen) tailsLen++;
  }
  let idx = tails[tailsLen - 1];
  while (idx >= 0) {
    mask[idx] = 1;
    idx = pred[idx];
  }
}
function bindFor(opts) {
  const { anchor, items, key, render } = opts;
  let prev = [];
  let active = true;
  const departing = /* @__PURE__ */ new Set();
  const nextKeys = /* @__PURE__ */ new Map();
  const prevByKey = /* @__PURE__ */ new Map();
  const prevPos = /* @__PURE__ */ new Map();
  const stable = /* @__PURE__ */ new Set();
  const freshNodes = [];
  let nextRecords = [];
  let matched = new Int32Array(0);
  let lisMaskBuf = new Uint8Array(0), lisPred = new Int32Array(0), lisTails = new Int32Array(0);
  const flush = (r) => {
    for (const n of r.nodes) {
      try {
        n.parentNode?.removeChild(n);
      } catch {
      }
    }
    r.cleanup?.();
  };
  let lastItems = null;
  const dispose = effect(() => {
    if (!active) return;
    const next = items();
    if (next === lastItems) return;
    lastItems = next;
    const n = next.length;
    if (matched.length < n * 2) matched = new Int32Array(Math.max(matched.length * 2, n * 2, 16));
    if (lisMaskBuf.length < n) {
      const cap = Math.max(lisMaskBuf.length * 2, n, 8);
      lisMaskBuf = new Uint8Array(cap);
      lisPred = new Int32Array(cap);
      lisTails = new Int32Array(cap);
    }
    for (let i = 0; i < n; i++) {
      const item = next[i];
      const k = key(item, i);
      if (nextKeys.has(k)) {
        console.warn(
          "[sui] {#for}: duplicate key",
          k,
          "at index",
          i,
          "(prior at",
          nextKeys.get(k),
          "); the earlier item will be dropped from the rendered output.",
          "Fix the key function so each item maps to a unique value."
        );
      }
      nextKeys.set(k, i);
    }
    for (const r of prev) prevByKey.set(r.key, r);
    for (const r of prev) {
      if (nextKeys.has(r.key)) continue;
      const leaving = runLeave(r.nodes);
      if (leaving === null) {
        flush(r);
        continue;
      }
      departing.add(r);
      leaving.then(() => {
        if (!departing.delete(r)) return;
        flush(r);
      });
    }
    nextRecords.length = 0;
    freshNodes.length = 0;
    let anyReused = false;
    for (let i = 0; i < n; i++) {
      const item = next[i];
      const k = key(item, i);
      const existing = prevByKey.get(k);
      let record;
      if (existing) {
        anyReused = true;
        if (existing.item !== item || existing.__i !== i) {
          existing.item = item;
          existing.__i = i;
          runProtected(anchor, "render", () => existing.update?.(item, i));
        }
        record = existing;
      } else {
        const r = runProtected(anchor, "render", () => render(item, i));
        if (!r) {
          continue;
        }
        record = {
          key: k,
          item,
          __i: i,
          nodes: r.nodes,
          ...r.cleanup && { cleanup: r.cleanup },
          ...r.update && { update: r.update }
        };
        freshNodes.push(...r.nodes);
      }
      nextRecords.push(record);
    }
    let matchedCount = 0;
    if (anyReused) {
      for (let i = 0; i < prev.length; i++) prevPos.set(prev[i].key, i);
      let monotone = true;
      let lastPrevPos = -1;
      for (let i = 0; i < nextRecords.length; i++) {
        const p = prevPos.get(nextRecords[i].key);
        if (p !== void 0) {
          matched[2 * matchedCount] = i;
          matched[2 * matchedCount + 1] = p;
          matchedCount++;
          if (monotone && p <= lastPrevPos) monotone = false;
          lastPrevPos = p;
        }
      }
      if (monotone) {
        for (let j = 0; j < matchedCount; j++) stable.add(matched[2 * j]);
      } else {
        computeLisMaskInto(matched, matchedCount, lisMaskBuf, lisPred, lisTails);
        for (let j = 0; j < matchedCount; j++) {
          if (lisMaskBuf[j]) stable.add(matched[2 * j]);
        }
      }
    }
    const parent = anchor.parentNode;
    if (parent && matchedCount === 0 && nextRecords.length > 0) {
      const frag = document.createDocumentFragment();
      for (let i = 0; i < nextRecords.length; i++) {
        const nodes = nextRecords[i].nodes;
        for (let j = 0; j < nodes.length; j++) frag.appendChild(nodes[j]);
      }
      parent.insertBefore(frag, anchor.nextSibling);
    } else {
      let cursor = anchor;
      for (let i = 0; i < nextRecords.length; i++) {
        if (!parent) break;
        const record = nextRecords[i];
        if (stable.has(i)) {
          cursor = record.nodes[record.nodes.length - 1] ?? cursor;
        } else {
          for (const node of record.nodes) {
            parent.insertBefore(node, cursor.nextSibling);
            cursor = node;
          }
        }
      }
    }
    const tmp = prev;
    prev = nextRecords;
    nextRecords = tmp;
    if (freshNodes.length > 0) runEnter(freshNodes);
    nextRecords.length = 0;
    nextKeys.clear();
    prevByKey.clear();
    prevPos.clear();
    stable.clear();
    freshNodes.length = 0;
  });
  return () => {
    active = false;
    dispose();
    for (const r of prev) flush(r);
    for (const d of departing) flush(d);
    departing.clear();
    prev = [];
    nextRecords.length = 0;
    lastItems = null;
  };
}

// ../../packages/sui-runtime/src/if.ts
function bindIf(opts) {
  const { anchor } = opts;
  const { test, branches } = isMulti(opts) ? { test: opts.test, branches: opts.branches } : twoBranchToMulti(opts);
  let active = null;
  let lastIdx = -2;
  let pendingOut = null;
  const removeNodes = (nodes) => {
    for (const n of nodes) {
      try {
        n.parentNode?.removeChild(n);
      } catch {
      }
    }
  };
  const dispose = effect(() => {
    const idx = test();
    if (active && idx === lastIdx) return;
    if (pendingOut && idx === pendingOut.idx && idx >= 0 && idx < branches.length) {
      const promoted = pendingOut;
      pendingOut = null;
      for (const n of promoted.nodes) cancelTransitions(n);
      active = { nodes: promoted.nodes, ...promoted.cleanup && { cleanup: promoted.cleanup } };
      lastIdx = idx;
      runEnter(active.nodes);
      return;
    }
    if (active) {
      const dep = {
        idx: lastIdx,
        nodes: active.nodes,
        ...active.cleanup && { cleanup: active.cleanup }
      };
      const leaving = runLeave(dep.nodes);
      active = null;
      if (leaving === null) {
        removeNodes(dep.nodes);
        dep.cleanup?.();
      } else {
        pendingOut = dep;
        leaving.then(() => {
          if (pendingOut !== dep) return;
          removeNodes(dep.nodes);
          dep.cleanup?.();
          pendingOut = null;
        });
      }
    }
    lastIdx = idx;
    if (idx < 0 || idx >= branches.length) return;
    const rendered = runProtected(anchor, "render", () => branches[idx]());
    if (!rendered) return;
    active = rendered;
    const parent = anchor.parentNode;
    if (!parent) return;
    for (const n of active.nodes) parent.insertBefore(n, anchor);
    runEnter(active.nodes);
  });
  return () => {
    dispose();
    if (active) {
      removeNodes(active.nodes);
      active.cleanup?.();
      active = null;
    }
    if (pendingOut) {
      removeNodes(pendingOut.nodes);
      pendingOut.cleanup?.();
      pendingOut = null;
    }
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

// ../../packages/sui-runtime/src/transition.ts
var REG = Symbol("sui:transition");
var MARK = "data-sui-transition";
function ensureReg(node) {
  const obj = node;
  let r = obj[REG];
  if (!r) {
    r = { inOpts: {}, outOpts: {} };
    obj[REG] = r;
  }
  return r;
}
function registerTransition(node, transition, opts = {}, dir = "both") {
  const r = ensureReg(node);
  if (dir === "in" || dir === "both") {
    r.inT = transition;
    r.inOpts = opts;
  }
  if (dir === "out" || dir === "both") {
    r.outT = transition;
    r.outOpts = opts;
  }
  if (!node.hasAttribute(MARK)) node.setAttribute(MARK, "");
}
var DEFAULT_DURATION = 200;
var DEFAULT_EASING = "cubic-bezier(0.4, 0, 0.2, 1)";
var DEFAULT_DELAY = 0;
function timing(o) {
  return {
    duration: typeof o.duration === "number" ? o.duration : DEFAULT_DURATION,
    easing: typeof o.easing === "string" ? o.easing : DEFAULT_EASING,
    delay: typeof o.delay === "number" ? o.delay : DEFAULT_DELAY,
    fill: "both"
  };
}
function animateSafe(node, frames, t) {
  const fn = node.animate;
  if (typeof fn !== "function") return void 0;
  return fn.call(node, frames, t);
}
var fade = {
  in: (node, o) => animateSafe(node, [{ opacity: 0 }, { opacity: 1 }], timing(o)),
  out: (node, o) => animateSafe(node, [{ opacity: 1 }, { opacity: 0 }], timing(o))
};
function scaleFrames(from, dir) {
  const off = { opacity: 0, transform: `scale(${from})` };
  const on2 = { opacity: 1, transform: "scale(1)" };
  return dir === "in" ? [off, on2] : [on2, off];
}
var scale = {
  in: (node, o) => {
    const from = typeof o.from === "number" ? o.from : 0;
    return animateSafe(node, scaleFrames(from, "in"), timing(o));
  },
  out: (node, o) => {
    const from = typeof o.from === "number" ? o.from : 0;
    return animateSafe(node, scaleFrames(from, "out"), timing(o));
  }
};

// ../../packages/sui-runtime/src/decode.ts
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

// ../../packages/sui-runtime/src/component-decoders.ts
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

// ../../packages/sui-runtime/src/component-form.ts
function installFormHooks(host, internals, def, propSignals) {
  if (!internals) return void 0;
  const hooks = {};
  let stopValueEffect;
  const valueAttr = def.formValue;
  if (valueAttr !== void 0) {
    const { sig, propDef } = resolveProp(def, propSignals, valueAttr, "formValue");
    const initial = propDef.default;
    const decode = decoderFor(propDef);
    stopValueEffect = effect(() => {
      const v = sig();
      internals.setFormValue(stringifyFormValue(v));
    });
    hooks.reset = () => {
      sig.set(initial);
    };
    hooks.restore = (state) => {
      sig.set(decode(state));
    };
  }
  const disabledAttr = def.formDisabled;
  if (disabledAttr !== void 0) {
    const { sig, propDef } = resolveProp(def, propSignals, disabledAttr, "formDisabled");
    if (propDef.type !== "bool") {
      throw new Error(
        `[sui-runtime] formDisabled: "${disabledAttr}" must be a bool-typed prop`
      );
    }
    hooks.disabled = (d) => {
      sig.set(!!d);
    };
  }
  ;
  host.__shipSetFormHooks = hooks;
  if (stopValueEffect) {
    ;
    host.__shipSetFormValueCleanup = stopValueEffect;
  }
  return stopValueEffect;
}
function clearFormHooks(host) {
  const holder = host;
  delete holder.__shipSetFormHooks;
  delete holder.__shipSetFormValueCleanup;
}
function stringifyFormValue(v) {
  if (v === null || v === void 0) return "";
  if (typeof v === "boolean") return v ? "on" : "";
  return String(v);
}
function resolveProp(def, propSignals, attr, channel) {
  const propDef = def.props[attr];
  if (!propDef) {
    throw new Error(
      `[sui-runtime] ${channel}: "${attr}" \u2014 no matching prop on this component`
    );
  }
  const sig = propSignals[camelOf(attr)];
  if (!sig) {
    throw new Error(
      `[sui-runtime] ${channel}: "${attr}" \u2014 prop signal missing`
    );
  }
  return { sig, propDef };
}

// ../../packages/sui-runtime/src/component-factory.ts
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
  for (const [attr, propDef] of Object.entries(def.props ?? {})) {
    observedAttrs.push(attr);
    props[attr] = { camel: camelOf(attr), decode: decoderFor(propDef) };
  }
  const associated = def.formAssociated === true || def.formValue !== void 0 || def.formDisabled !== void 0;
  const schema = { observedAttrs, props };
  if (associated) schema.formAssociated = true;
  return schema;
};
function component(tag, def) {
  const schema = buildSchema(def);
  const propEntries = Object.entries(def.props ?? {});
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
    if (def.formValue !== void 0 || def.formDisabled !== void 0) {
      const stopValue = installFormHooks(host, internals, def, propSignals);
      if (typeof stopValue === "function") cleanups.push(stopValue);
    }
    if (def.setup) {
      const ctx = { props: propSignals, host, cleanups };
      if (internals) ctx.internals = internals;
      def.setup(ctx);
    }
    return () => {
      clearFormHooks(host);
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

// ../../packages/sui-runtime/src/component.ts
var warnedDuplicateTags = /* @__PURE__ */ new Set();
function defineComponent(tag, schema, factory) {
  if (customElements.get(tag)) {
    if (!warnedDuplicateTags.has(tag)) {
      warnedDuplicateTags.add(tag);
      console.warn(
        `[sui-runtime] customElements.define("${tag}", \u2026) called twice. The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.`
      );
    }
    return;
  }
  if (schema.formAssociated) {
    class ShipSetFormElement extends HTMLElement {
      constructor() {
        super();
        __publicField(this, "cleanup");
        __publicField(this, "scope");
        __publicField(this, "propSignals", {});
        __publicField(this, "internals");
        this.internals = this.attachInternals();
      }
      static get observedAttributes() {
        return schema.observedAttrs;
      }
      connectedCallback() {
        this.dataset.component = tag;
        attachHostApi(this);
        this.scope = createScope();
        try {
          this.cleanup = withScope(
            this.scope,
            () => withHost(this, () => factory(this, this.propSignals, this.internals))
          );
        } catch (err) {
          const boundary = findNearestBoundary(this.parentNode);
          if (boundary) {
            boundary.catch(err, "mount");
          } else {
            console.error(`[ship-set:${tag}] setup threw:`, err);
          }
        }
      }
      disconnectedCallback() {
        try {
          this.cleanup?.();
        } catch (e) {
          console.error(`[ship-set:${tag}] cleanup threw:`, e);
        }
        this.cleanup = void 0;
        try {
          this.scope?.dispose();
        } catch (e) {
          console.error(`[ship-set:${tag}] scope dispose threw:`, e);
        }
        this.scope = void 0;
      }
      attributeChangedCallback(attrName, _old, value) {
        const meta = schema.props[attrName];
        if (!meta) return;
        const sig = this.propSignals[meta.camel];
        if (sig) sig.set(meta.decode(value));
      }
      // Form-association lifecycle hooks. The browser dispatches these
      // on the host. The runtime fans them out two ways:
      //   1. Element-level events (`ship-set:form-*`) for ad-hoc
      //      listeners attached by author code or tests.
      //   2. Factory-installed handlers stored under
      //      `__shipSetFormHooks` (set by component-factory.ts when the
      //      author declared `formValue:` / `formDisabled:` on the
      //      component def). These run BEFORE the event so signal
      //      writes are visible to listeners that read the prop.
      formResetCallback() {
        const hooks = this.__shipSetFormHooks;
        try {
          hooks?.reset?.();
        } catch (e) {
          console.error(`[ship-set:${tag}] formResetCallback threw:`, e);
        }
        this.dispatchEvent(new CustomEvent("ship-set:form-reset"));
      }
      formStateRestoreCallback(state, mode) {
        const hooks = this.__shipSetFormHooks;
        try {
          hooks?.restore?.(state);
        } catch (e) {
          console.error(`[ship-set:${tag}] formStateRestoreCallback threw:`, e);
        }
        this.dispatchEvent(
          new CustomEvent("ship-set:form-restore", { detail: { state, mode } })
        );
      }
      formDisabledCallback(disabled) {
        const hooks = this.__shipSetFormHooks;
        try {
          hooks?.disabled?.(disabled);
        } catch (e) {
          console.error(`[ship-set:${tag}] formDisabledCallback threw:`, e);
        }
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
      __publicField(this, "scope");
      __publicField(this, "propSignals", {});
    }
    static get observedAttributes() {
      return schema.observedAttrs;
    }
    connectedCallback() {
      this.dataset.component = tag;
      attachHostApi(this);
      this.scope = createScope();
      try {
        this.cleanup = withScope(
          this.scope,
          () => withHost(this, () => factory(this, this.propSignals))
        );
      } catch (err) {
        const boundary = findNearestBoundary(this.parentNode);
        if (boundary) {
          boundary.catch(err, "mount");
        } else {
          console.error(`[ship-set:${tag}] setup threw:`, err);
        }
      }
    }
    disconnectedCallback() {
      try {
        this.cleanup?.();
      } catch (e) {
        console.error(`[ship-set:${tag}] cleanup threw:`, e);
      }
      this.cleanup = void 0;
      try {
        this.scope?.dispose();
      } catch (e) {
        console.error(`[ship-set:${tag}] scope dispose threw:`, e);
      }
      this.scope = void 0;
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
function camelOf(attr) {
  return attr.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
}

// ../../packages/sui-runtime/src/controller-hook-resolver.ts
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

// ../../packages/sui-runtime/src/controller-resolver.ts
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
  const scopeAdds = shape.scope ? shape.scope(baseScope, ctx.host) : {};
  Object.assign(baseScope, scopeAdds);
  const stateSignals = pickStateSignals(baseScope, ctx.stateKeys, scopeAdds);
  const writableProxy = buildWritableProxy(stateSignals, baseScope);
  const declScope = { ...baseScope };
  const deriveds = buildDeriveds(shape, declScope, baseScope);
  Object.assign(declScope, deriveds);
  const useResults = buildUseResults(shape, declScope, ctx);
  for (const [k, v] of Object.entries(useResults)) {
    declScope[k] = v;
  }
  const onHandlers = buildOnHandlers(shape, writableProxy);
  const bindings = {
    ...scopeAdds,
    ...deriveds,
    ...onHandlers,
    ...useResults
  };
  if (Object.keys(scopeAdds).length > 0 && ctx.host) {
    const sharedEntries = pickShared(scopeAdds);
    if (Object.keys(sharedEntries).length > 0) {
      const hostEl = ctx.host;
      const tag = hostEl.tagName.toLowerCase();
      provide(hostEl, tag, sharedEntries);
    }
  }
  if (shape.mount || shape.unmount) {
    const lifecycle = {
      state: writableProxy
    };
    if (shape.mount) lifecycle.mount = shape.mount;
    if (shape.unmount) lifecycle.unmount = shape.unmount;
    Object.defineProperty(bindings, LIFECYCLE_KEY, {
      value: lifecycle,
      enumerable: true,
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
function pickStateSignals(scope, stateKeys, scopeAdds) {
  const out = {};
  const haveStateKeys = stateKeys !== void 0;
  const scopeAddKeys = scopeAdds ? Object.keys(scopeAdds) : [];
  if (haveStateKeys || scopeAddKeys.length > 0) {
    if (stateKeys) {
      for (const k of stateKeys) {
        const v = scope[k];
        if (isSignal(v)) out[k] = v;
      }
    }
    for (const k of scopeAddKeys) {
      if (k in out) continue;
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
var SIGNALS_KEY = "$signals";
function buildWritableProxy(stateSignals, baseScope) {
  return new Proxy({}, {
    get(_target, key) {
      if (key === SIGNALS_KEY) return stateSignals;
      if (key in stateSignals) return stateSignals[key]();
      return baseScope[key];
    },
    set(_target, key, value) {
      if (key in stateSignals) {
        if (typeof value === "function") {
          stateSignals[key].set(() => value);
        } else {
          stateSignals[key].set(value);
        }
        return true;
      }
      if (key === SIGNALS_KEY) {
        console.warn(
          `[ship-set] handler tried to write s.${SIGNALS_KEY} = \u2026 but ${SIGNALS_KEY} is the read-only signal-map accessor. The assignment was ignored.`
        );
        return true;
      }
      console.warn(
        `[ship-set] handler tried to write s.${key} = \u2026 but ${key} is not declared as <state name="${key}"> in the template. The assignment was ignored.`
      );
      return true;
    },
    has(_target, key) {
      return key === SIGNALS_KEY || key in stateSignals || key in baseScope;
    },
    ownKeys() {
      return [SIGNALS_KEY, ...Object.keys(stateSignals)];
    },
    getOwnPropertyDescriptor(_target, key) {
      if (key === SIGNALS_KEY) {
        return { configurable: true, enumerable: true, writable: false, value: stateSignals };
      }
      if (key in stateSignals) {
        return { configurable: true, enumerable: true, writable: true, value: stateSignals[key]() };
      }
      return void 0;
    }
  });
}
function buildDeriveds(shape, declScope, baseScope) {
  const section = shape.derived ?? {};
  const out = {};
  for (const [k, fn] of Object.entries(section)) {
    const shadowsScope = k in baseScope;
    const scopeView = shadowsScope ? new Proxy(declScope, {
      get(target, prop2) {
        return prop2 === k ? baseScope[k] : Reflect.get(target, prop2);
      },
      has(target, prop2) {
        return prop2 in baseScope || prop2 in target;
      }
    }) : declScope;
    out[k] = derived(() => fn(scopeView));
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

// ../../packages/sui-runtime/src/boundary-elements.ts
if (typeof customElements !== "undefined") {
  for (const tag of [
    "sui-if",
    "sui-for",
    "sui-match",
    "sui-while-loading",
    "sui-while-error"
  ]) {
    if (!customElements.get(tag)) {
      customElements.define(tag, class extends HTMLElement {
      });
    }
  }
}

// ../../packages/sui-runtime/src/styles/index.ts
var TONES_CSS = ":host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }\n:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }\n:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }\n:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }\n:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }\n:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }\n:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }\n:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }\n";
var BASE_CSS = ":host([hidden]) { display: none !important; }\n@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }\n";
function makeSheet(css) {
  if (typeof CSSStyleSheet === "undefined") return null;
  try {
    const sheet = new CSSStyleSheet();
    sheet.replaceSync(css);
    return sheet;
  } catch {
    return null;
  }
}
var suiTonesSheet = makeSheet(TONES_CSS);
var suiBaseSheet = makeSheet(BASE_CSS);
function adoptSuiSheets(root) {
  if (typeof ShadowRoot === "undefined") return;
  if (!(root instanceof ShadowRoot)) return;
  const sheets = [suiBaseSheet, suiTonesSheet].filter(
    (s) => s != null
  );
  if (sheets.length === 0) return;
  try {
    root.adoptedStyleSheets = [...sheets, ...root.adoptedStyleSheets ?? []];
  } catch {
  }
}

// ../../packages/sui-runtime/src/chrome.ts
var pendingModals = /* @__PURE__ */ new Map();
if (typeof document !== "undefined") {
  document.addEventListener("sui:modal:respond", (e) => {
    const detail = e.detail;
    if (typeof detail?.id !== "number") return;
    const resolve = pendingModals.get(detail.id);
    if (!resolve) return;
    pendingModals.delete(detail.id);
    resolve(detail.action ?? "");
  });
}

// src/sui-app.controller.ts
var OWNERS = ["ada", "beck", "cora", "dax", "ezra"];
var TITLES = [
  "wire sse reconnect",
  "profile bindFor at 10k",
  "audit focus ring",
  "tokenize router segments",
  "split kanban controller",
  "add gauge component",
  "sweep aria-labels",
  "centralize date format",
  "replace findByTag"
];
var PRIORITIES = ["low", "normal", "normal", "normal", "high"];
var TONES = ["var(--info)", "var(--success)", "var(--warning)", "var(--accent)", "var(--primary)"];
var nextId = 0;
var pick = (arr) => arr[Math.floor(Math.random() * arr.length)];
var mkCard = () => ({
  id: ++nextId,
  title: pick(TITLES),
  owner: pick(OWNERS),
  priority: pick(PRIORITIES),
  tone: pick(TONES)
});
var initialColumns = () => [
  { id: "todo", title: "To do", tone: "var(--info)", cards: Array.from({ length: 4 }, mkCard) },
  { id: "doing", title: "In progress", tone: "var(--warning)", cards: Array.from({ length: 3 }, mkCard) },
  { id: "done", title: "Done", tone: "var(--success)", cards: Array.from({ length: 2 }, mkCard) }
];
var moveBetween = (state, cardId, targetCol, beforeId) => {
  const next = state.map((c) => ({ ...c, cards: [...c.cards] }));
  let moving = null;
  for (const c of next) {
    const idx = c.cards.findIndex((x) => x.id === cardId);
    if (idx >= 0) {
      moving = c.cards.splice(idx, 1)[0];
      break;
    }
  }
  if (!moving) return state;
  const target = next.find((c) => c.id === targetCol);
  if (!target) return state;
  if (beforeId == null) target.cards.push(moving);
  else {
    const idx = target.cards.findIndex((c) => c.id === beforeId);
    target.cards.splice(idx < 0 ? target.cards.length : idx, 0, moving);
  }
  return next;
};
var fmt = (d) => `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}:${String(d.getSeconds()).padStart(2, "0")}.${String(d.getMilliseconds()).padStart(3, "0")}`;
var sui_app_controller_default = controller({
  scope: () => {
    const count = signal(0);
    const doubled = derived(() => count() * 2);
    const ticks = signal(0);
    const lastTickWall = signal(Date.now());
    const recentTicks = signal([]);
    const tickRate = signal("0");
    const tickHistory = [];
    const interval = setInterval(() => {
      const n = ticks() + 1;
      const now = Date.now();
      ticks.set(n);
      lastTickWall.set(now);
      const ws = recentTicks().slice(-9);
      ws.push({ id: n, label: fmt(new Date(now)) });
      recentTicks.set(ws);
      tickHistory.push(now);
      while (tickHistory.length > 0 && now - tickHistory[0] > 1e3) tickHistory.shift();
      tickRate.set(tickHistory.length.toString());
    }, 250);
    const cols = signal(initialColumns());
    const draggingId = signal(null);
    effect(() => {
      if (count() >= 10 && count() < 11) {
        console.log("[sui-app] count reached", count());
      }
    });
    return {
      // counter
      count,
      doubled,
      bump: (n) => count.set(count() + n),
      reset: () => count.set(0),
      // ticks
      ticks,
      recentTicks,
      tickRate,
      // kanban
      cols,
      draggingId,
      totalCards: derived(() => cols().reduce((n, c) => n + c.cards.length, 0)),
      columns: derived(() => cols()),
      addRandom: () => {
        const c = cols();
        const col = pick(c);
        cols.set(c.map((x) => x.id === col.id ? { ...x, cards: [...x.cards, mkCard()] } : x));
      },
      chaos: () => {
        let state = cols();
        for (let i = 0; i < 20; i++) {
          const op = Math.random();
          if (op < 0.45 && state.some((x) => x.cards.length > 0)) {
            const filled = state.filter((x) => x.cards.length > 0);
            const from = pick(filled);
            const card = pick(from.cards);
            const to = pick(state);
            const beforeId = to.cards.length > 0 ? pick(to.cards).id : null;
            state = moveBetween(state, card.id, to.id, beforeId);
          } else if (op < 0.8) {
            const col = pick(state);
            state = state.map((x) => x.id === col.id ? { ...x, cards: [...x.cards, mkCard()] } : x);
          } else {
            const all = state.flatMap((x) => x.cards);
            if (all.length === 0) continue;
            const victim = pick(all);
            state = state.map((col) => ({ ...col, cards: col.cards.filter((c) => c.id !== victim.id) }));
          }
        }
        cols.set(state);
      },
      resetBoard: () => {
        nextId = 0;
        cols.set(initialColumns());
      },
      onDragStart: (e, id) => {
        draggingId.set(id);
        if (e.dataTransfer) {
          e.dataTransfer.setData("text/plain", String(id));
          e.dataTransfer.effectAllowed = "move";
        }
      },
      onDragEnd: () => draggingId.set(null),
      onDragOver: (e) => {
        if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
      },
      onDrop: (e, col, beforeId) => {
        const raw = e.dataTransfer?.getData("text/plain") ?? "";
        const id = raw ? Number(raw) : draggingId();
        if (id == null || id === beforeId) {
          draggingId.set(null);
          return;
        }
        cols.set(moveBetween(cols(), id, col, beforeId));
        draggingId.set(null);
      },
      __interval: interval
    };
  }
});

// src/dist/sui-app.js
var sui_app_default = component("sui-app", {
  _compilerVersion: { cargo: "0.1.0", git: "d69f9713", dirty: true, builtAt: "2026-05-16T18:56:56Z" },
  props: {
    "shadow": prop.bool({ default: false })
  },
  shadow: false,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups) ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = /* @__PURE__ */ Object.create(null);
    $scope.shadow = props.shadow;
    let __r = null;
    if (sui_app_controller_default && typeof sui_app_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_app_controller_default.setup(ctx);
      if (__r && typeof __r === "object") Object.assign($scope, __r);
    }
    const __wireBoundary_b8 = (anchor) => bindFor({
      anchor,
      items: () => $scope.recentTicks(),
      key: (row, __i) => {
        const __child = Object.create($scope);
        __child.row = row;
        __child.__i = __i;
        return (($scope2) => row.id)(__child);
      },
      render: (row, __i) => {
        const __childScope40 = Object.create($scope);
        __childScope40.row = row;
        __childScope40.__i = __i;
        return (($scope2) => {
          __childScope40.__v_id = signal(0);
          const __frag41 = document.createDocumentFragment();
          const __c42 = [];
          const __e43 = document.createElement("div");
          __e43.setAttribute("class", "ticks-row");
          __c42.push(bindClass(__e43, "fresh", () => (void $scope2.__v_id(), $scope2.row.id === $scope2.ticks())));
          if ($scope2.row.id % 7 === 0) __e43.classList.add("warn");
          else __e43.classList.remove("warn");
          const __t44 = document.createTextNode("\n          tick ");
          __e43.appendChild(__t44);
          const __t45 = document.createTextNode("");
          __e43.appendChild(__t45);
          __t45.textContent = $scope2.row.id;
          const __t46 = document.createTextNode(" @ ");
          __e43.appendChild(__t46);
          const __t47 = document.createTextNode("");
          __e43.appendChild(__t47);
          __t47.textContent = $scope2.row.label;
          __frag41.appendChild(__e43);
          return { nodes: Array.from(__frag41.childNodes), cleanup: () => {
            for (const c of __c42) try {
              c();
            } catch (e) {
            }
          }, update: (__new48, __newI49) => {
            const __old = $scope2.row;
            $scope2.row = __new48;
            $scope2.__i = __newI49;
            if (__old.id !== $scope2.row.id) {
              if ($scope2.row.id % 7 === 0) __e43.classList.add("warn");
              else __e43.classList.remove("warn");
              __t45.textContent = $scope2.row.id;
            }
            if (__old.label !== $scope2.row.label) {
              __t47.textContent = $scope2.row.label;
            }
            if (__old.id !== $scope2.row.id) {
              $scope2.__v_id.set((c) => c + 1);
            }
          } };
        })(__childScope40);
      }
    });
    const __wireBoundary_b17 = (anchor) => bindFor({
      anchor,
      items: () => $scope.columns(),
      key: (col, __i) => {
        const __child = Object.create($scope);
        __child.col = col;
        __child.__i = __i;
        return (($scope2) => col.id)(__child);
      },
      render: (col, __i) => {
        const __childScope65 = Object.create($scope);
        __childScope65.col = col;
        __childScope65.__i = __i;
        return (($scope2) => {
          const __wireBoundary_b25 = (anchor2) => bindFor({
            anchor: anchor2,
            items: () => $scope2.col.cards,
            key: (card, __i2) => {
              const __child = Object.create($scope2);
              __child.card = card;
              __child.__i = __i2;
              return (($scope3) => card.id)(__child);
            },
            render: (card, __i2) => {
              const __childScope82 = Object.create($scope2);
              __childScope82.card = card;
              __childScope82.__i = __i2;
              return (($scope3) => {
                const __wireBoundary_b32 = (anchor3) => bindIf({
                  anchor: anchor3,
                  test: () => {
                    if ($scope3.card.priority === "high") return 0;
                    if ($scope3.card.priority === "low") return 1;
                    return -1;
                  },
                  branches: [
                    () => {
                      const __frag90 = document.createDocumentFragment();
                      const __c91 = [];
                      const __e92 = document.createElement("span");
                      __e92.setAttribute("class", "pill high");
                      const __t93 = document.createTextNode("high");
                      __e92.appendChild(__t93);
                      __frag90.appendChild(__e92);
                      return { nodes: Array.from(__frag90.childNodes), cleanup: () => {
                        for (const c of __c91) try {
                          c();
                        } catch (e) {
                        }
                      } };
                    },
                    () => {
                      const __frag94 = document.createDocumentFragment();
                      const __c95 = [];
                      const __e96 = document.createElement("span");
                      __e96.setAttribute("class", "pill low");
                      const __t97 = document.createTextNode("low");
                      __e96.appendChild(__t97);
                      __frag94.appendChild(__e96);
                      return { nodes: Array.from(__frag94.childNodes), cleanup: () => {
                        for (const c of __c95) try {
                          c();
                        } catch (e) {
                        }
                      } };
                    }
                  ]
                });
                const __frag83 = document.createDocumentFragment();
                const __c84 = [];
                const __e85 = document.createElement("li");
                __e85.setAttribute("class", "card");
                {
                  const __av86 = $scope3.card.id;
                  if (__av86 == null || __av86 === false || __av86 === "") __e85.removeAttribute("data-card");
                  else __e85.setAttribute("data-card", __av86 === true ? "" : String(__av86));
                }
                __e85.setAttribute("draggable", "true");
                registerTransition(__e85, scale, { from: 0.9, duration: 160 }, "in");
                registerTransition(__e85, fade, { duration: 120 }, "out");
                __c84.push(on(__e85, "dragstart", function() {
                  const __h = (e) => $scope3.onDragStart(e, $scope3.card.id);
                  return typeof __h === "function" ? __h.bind(void 0) : () => {
                  };
                }(), { modifiers: [] }));
                __c84.push(on(__e85, "dragend", function() {
                  const __h = $scope3.onDragEnd;
                  return typeof __h === "function" ? __h.bind(void 0) : () => {
                  };
                }(), { modifiers: [] }));
                __c84.push(on(__e85, "dragover", function() {
                  const __h = (e) => $scope3.onDragOver(e);
                  return typeof __h === "function" ? __h.bind(void 0) : () => {
                  };
                }(), { modifiers: [{ kind: "preventDefault" }, { kind: "stopPropagation" }] }));
                __c84.push(on(__e85, "drop", function() {
                  const __h = (e) => $scope3.onDrop(e, $scope3.col.id, $scope3.card.id);
                  return typeof __h === "function" ? __h.bind(void 0) : () => {
                  };
                }(), { modifiers: [{ kind: "preventDefault" }, { kind: "stopPropagation" }] }));
                const __e87 = document.createElement("div");
                __e87.setAttribute("class", "title");
                const __t88 = document.createTextNode("");
                __e87.appendChild(__t88);
                __t88.textContent = $scope3.card.title;
                const __ifAnchor89 = document.createComment("if");
                __e87.appendChild(__ifAnchor89);
                __c84.push(__wireBoundary_b32(__ifAnchor89));
                __e85.appendChild(__e87);
                const __e98 = document.createElement("div");
                __e98.setAttribute("class", "meta");
                const __t99 = document.createTextNode("owner: ");
                __e98.appendChild(__t99);
                const __t100 = document.createTextNode("");
                __e98.appendChild(__t100);
                __t100.textContent = $scope3.card.owner;
                const __t101 = document.createTextNode(" \xB7 #");
                __e98.appendChild(__t101);
                const __t102 = document.createTextNode("");
                __e98.appendChild(__t102);
                __t102.textContent = $scope3.card.id;
                __e85.appendChild(__e98);
                __frag83.appendChild(__e85);
                return { nodes: Array.from(__frag83.childNodes), cleanup: () => {
                  for (const c of __c84) try {
                    c();
                  } catch (e) {
                  }
                }, update: (__new103, __newI104) => {
                  const __old = $scope3.card;
                  $scope3.card = __new103;
                  $scope3.__i = __newI104;
                  if (__old.id !== $scope3.card.id) {
                    {
                      const __av86 = $scope3.card.id;
                      if (__av86 == null || __av86 === false || __av86 === "") __e85.removeAttribute("data-card");
                      else __e85.setAttribute("data-card", __av86 === true ? "" : String(__av86));
                    }
                    __t102.textContent = $scope3.card.id;
                  }
                  if (__old.title !== $scope3.card.title) {
                    __t88.textContent = $scope3.card.title;
                  }
                  if (__old.owner !== $scope3.card.owner) {
                    __t100.textContent = $scope3.card.owner;
                  }
                } };
              })(__childScope82);
            }
          });
          const __wireBoundary_b35 = (anchor2) => bindIf({
            anchor: anchor2,
            test: () => {
              if ($scope2.col.cards.length === 0) return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag106 = document.createDocumentFragment();
                const __c107 = [];
                const __e108 = document.createElement("li");
                __e108.setAttribute("class", "card");
                __e108.setAttribute("style", "border-style:dashed;cursor:default;text-align:center;color:var(--muted)");
                const __t109 = document.createTextNode("\n                drop a card\n              ");
                __e108.appendChild(__t109);
                __frag106.appendChild(__e108);
                return { nodes: Array.from(__frag106.childNodes), cleanup: () => {
                  for (const c of __c107) try {
                    c();
                  } catch (e) {
                  }
                } };
              }
            ]
          });
          const __frag66 = document.createDocumentFragment();
          const __c67 = [];
          const __e68 = document.createElement("section");
          __e68.setAttribute("class", "col");
          {
            const __av69 = $scope2.col.id;
            if (__av69 == null || __av69 === false || __av69 === "") __e68.removeAttribute("data-col");
            else __e68.setAttribute("data-col", __av69 === true ? "" : String(__av69));
          }
          const __e70 = document.createElement("div");
          __e70.setAttribute("class", "col-head");
          const __e71 = document.createElement("span");
          __e71.setAttribute("class", "dot");
          {
            const __av72 = "background:" + $scope2.col.tone;
            if (__av72 == null || __av72 === false || __av72 === "") __e71.removeAttribute("style");
            else __e71.setAttribute("style", __av72 === true ? "" : String(__av72));
          }
          __e70.appendChild(__e71);
          const __e73 = document.createElement("span");
          const __t74 = document.createTextNode("");
          __e73.appendChild(__t74);
          __t74.textContent = $scope2.col.title;
          __e70.appendChild(__e73);
          const __e75 = document.createElement("span");
          __e75.setAttribute("class", "stat");
          const __t76 = document.createTextNode("(");
          __e75.appendChild(__t76);
          const __t77 = document.createTextNode("");
          __e75.appendChild(__t77);
          __t77.textContent = $scope2.col.cards.length;
          const __t78 = document.createTextNode(")");
          __e75.appendChild(__t78);
          __e70.appendChild(__e75);
          __e68.appendChild(__e70);
          const __e79 = document.createElement("ol");
          __e79.setAttribute("class", "cards");
          {
            const __av80 = $scope2.col.id;
            if (__av80 == null || __av80 === false || __av80 === "") __e79.removeAttribute("data-col-cards");
            else __e79.setAttribute("data-col-cards", __av80 === true ? "" : String(__av80));
          }
          __c67.push(on(__e79, "dragover", function() {
            const __h = (e) => $scope2.onDragOver(e);
            return typeof __h === "function" ? __h.bind(void 0) : () => {
            };
          }(), { modifiers: [{ kind: "preventDefault" }] }));
          __c67.push(on(__e79, "drop", function() {
            const __h = (e) => $scope2.onDrop(e, $scope2.col.id, null);
            return typeof __h === "function" ? __h.bind(void 0) : () => {
            };
          }(), { modifiers: [{ kind: "preventDefault" }] }));
          const __forAnchor81 = document.createComment("for");
          __e79.appendChild(__forAnchor81);
          __c67.push(__wireBoundary_b25(__forAnchor81));
          const __ifAnchor105 = document.createComment("if");
          __e79.appendChild(__ifAnchor105);
          __c67.push(__wireBoundary_b35(__ifAnchor105));
          __e68.appendChild(__e79);
          __frag66.appendChild(__e68);
          return { nodes: Array.from(__frag66.childNodes), cleanup: () => {
            for (const c of __c67) try {
              c();
            } catch (e) {
            }
          }, update: (__new110, __newI111) => {
            const __old = $scope2.col;
            $scope2.col = __new110;
            $scope2.__i = __newI111;
            if (__old.id !== $scope2.col.id) {
              {
                const __av69 = $scope2.col.id;
                if (__av69 == null || __av69 === false || __av69 === "") __e68.removeAttribute("data-col");
                else __e68.setAttribute("data-col", __av69 === true ? "" : String(__av69));
              }
              {
                const __av80 = $scope2.col.id;
                if (__av80 == null || __av80 === false || __av80 === "") __e79.removeAttribute("data-col-cards");
                else __e79.setAttribute("data-col-cards", __av80 === true ? "" : String(__av80));
              }
            }
            if (__old.tone !== $scope2.col.tone) {
              {
                const __av72 = "background:" + $scope2.col.tone;
                if (__av72 == null || __av72 === false || __av72 === "") __e71.removeAttribute("style");
                else __e71.setAttribute("style", __av72 === true ? "" : String(__av72));
              }
            }
            if (__old.title !== $scope2.col.title) {
              __t74.textContent = $scope2.col.title;
            }
            if (__old.cards !== $scope2.col.cards) {
              __t77.textContent = $scope2.col.cards.length;
            }
          } };
        })(__childScope65);
      }
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets(__root);
    const __e0 = document.createElement("div");
    __e0.setAttribute("class", "wrap");
    const __e1 = document.createElement("header");
    const __e2 = document.createElement("div");
    const __e3 = document.createElement("h1");
    const __t4 = document.createTextNode("SUI sanity-check");
    __e3.appendChild(__t4);
    __e2.appendChild(__e3);
    const __e5 = document.createElement("p");
    const __t6 = document.createTextNode("standalone \u2014 no daemon, no shell, no view-host");
    __e5.appendChild(__t6);
    __e2.appendChild(__e5);
    __e1.appendChild(__e2);
    const __e7 = document.createElement("div");
    __e7.setAttribute("class", "row");
    const __e8 = document.createElement("div");
    __e8.setAttribute("class", "counter");
    const __t9 = document.createTextNode("ticks ");
    __e8.appendChild(__t9);
    const __e10 = document.createElement("strong");
    const __t11 = document.createTextNode("");
    __e10.appendChild(__t11);
    __cleanups.push(bindText(__t11, () => $scope.ticks()));
    __e8.appendChild(__e10);
    __e7.appendChild(__e8);
    const __e12 = document.createElement("div");
    __e12.setAttribute("class", "counter");
    const __t13 = document.createTextNode("");
    __e12.appendChild(__t13);
    __cleanups.push(bindText(__t13, () => $scope.tickRate()));
    const __t14 = document.createTextNode("/s");
    __e12.appendChild(__t14);
    __e7.appendChild(__e12);
    __e1.appendChild(__e7);
    __e0.appendChild(__e1);
    const __e15 = document.createElement("section");
    const __e16 = document.createElement("h2");
    const __t17 = document.createTextNode("Counter + derived");
    __e16.appendChild(__t17);
    __e15.appendChild(__e16);
    const __e18 = document.createElement("div");
    __e18.setAttribute("class", "row");
    const __e19 = document.createElement("div");
    __e19.setAttribute("class", "counter");
    const __t20 = document.createTextNode("count ");
    __e19.appendChild(__t20);
    const __e21 = document.createElement("strong");
    const __t22 = document.createTextNode("");
    __e21.appendChild(__t22);
    __cleanups.push(bindText(__t22, () => $scope.count()));
    __e19.appendChild(__e21);
    __e18.appendChild(__e19);
    const __e23 = document.createElement("div");
    __e23.setAttribute("class", "counter");
    const __t24 = document.createTextNode("\xD72 ");
    __e23.appendChild(__t24);
    const __e25 = document.createElement("strong");
    const __t26 = document.createTextNode("");
    __e25.appendChild(__t26);
    __cleanups.push(bindText(__t26, () => $scope.doubled()));
    __e23.appendChild(__e25);
    __e18.appendChild(__e23);
    const __e27 = document.createElement("button");
    __cleanups.push(on(__e27, "click", function() {
      const __h = () => $scope.bump(1);
      return typeof __h === "function" ? __h.bind(void 0) : () => {
      };
    }(), { modifiers: [] }));
    const __t28 = document.createTextNode("+1");
    __e27.appendChild(__t28);
    __e18.appendChild(__e27);
    const __e29 = document.createElement("button");
    __cleanups.push(on(__e29, "click", function() {
      const __h = () => $scope.bump(-1);
      return typeof __h === "function" ? __h.bind(void 0) : () => {
      };
    }(), { modifiers: [] }));
    const __t30 = document.createTextNode("\u22121");
    __e29.appendChild(__t30);
    __e18.appendChild(__e29);
    const __e31 = document.createElement("button");
    __cleanups.push(on(__e31, "click", function() {
      const __h = () => $scope.bump(10);
      return typeof __h === "function" ? __h.bind(void 0) : () => {
      };
    }(), { modifiers: [] }));
    const __t32 = document.createTextNode("+10");
    __e31.appendChild(__t32);
    __e18.appendChild(__e31);
    const __e33 = document.createElement("button");
    __e33.setAttribute("class", "danger");
    __cleanups.push(on(__e33, "click", function() {
      const __h = () => $scope.reset();
      return typeof __h === "function" ? __h.bind(void 0) : () => {
      };
    }(), { modifiers: [] }));
    const __t34 = document.createTextNode("reset");
    __e33.appendChild(__t34);
    __e18.appendChild(__e33);
    __e15.appendChild(__e18);
    __e0.appendChild(__e15);
    const __e35 = document.createElement("section");
    const __e36 = document.createElement("h2");
    const __t37 = document.createTextNode("Live tick log (effect re-fires each tick)");
    __e36.appendChild(__t37);
    __e35.appendChild(__e36);
    const __e38 = document.createElement("div");
    __e38.setAttribute("class", "ticks");
    const __forAnchor39 = document.createComment("for");
    __e38.appendChild(__forAnchor39);
    __cleanups.push(__wireBoundary_b8(__forAnchor39));
    __e35.appendChild(__e38);
    __e0.appendChild(__e35);
    const __e50 = document.createElement("section");
    const __e51 = document.createElement("h2");
    const __t52 = document.createTextNode("Kanban \u2014 bindFor + DnD + transitions");
    __e51.appendChild(__t52);
    __e50.appendChild(__e51);
    const __e53 = document.createElement("div");
    __e53.setAttribute("class", "row");
    __e53.setAttribute("style", "margin-bottom:12px");
    const __e54 = document.createElement("div");
    __e54.setAttribute("class", "stat");
    const __t55 = document.createTextNode("");
    __e54.appendChild(__t55);
    __cleanups.push(bindText(__t55, () => $scope.totalCards()));
    const __t56 = document.createTextNode(" cards");
    __e54.appendChild(__t56);
    __e53.appendChild(__e54);
    const __e57 = document.createElement("button");
    __cleanups.push(on(__e57, "click", function() {
      const __h = () => $scope.addRandom();
      return typeof __h === "function" ? __h.bind(void 0) : () => {
      };
    }(), { modifiers: [] }));
    const __t58 = document.createTextNode("+ add");
    __e57.appendChild(__t58);
    __e53.appendChild(__e57);
    const __e59 = document.createElement("button");
    __e59.setAttribute("class", "primary");
    __cleanups.push(on(__e59, "click", function() {
      const __h = () => $scope.chaos();
      return typeof __h === "function" ? __h.bind(void 0) : () => {
      };
    }(), { modifiers: [] }));
    const __t60 = document.createTextNode("chaos \xD7 20");
    __e59.appendChild(__t60);
    __e53.appendChild(__e59);
    const __e61 = document.createElement("button");
    __e61.setAttribute("class", "danger");
    __cleanups.push(on(__e61, "click", function() {
      const __h = () => $scope.resetBoard();
      return typeof __h === "function" ? __h.bind(void 0) : () => {
      };
    }(), { modifiers: [] }));
    const __t62 = document.createTextNode("reset board");
    __e61.appendChild(__t62);
    __e53.appendChild(__e61);
    __e50.appendChild(__e53);
    const __e63 = document.createElement("div");
    __e63.setAttribute("class", "cols");
    const __forAnchor64 = document.createComment("for");
    __e63.appendChild(__forAnchor64);
    __cleanups.push(__wireBoundary_b17(__forAnchor64));
    __e50.appendChild(__e63);
    __e0.appendChild(__e50);
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
          if (typeof __mc === "function") __cleanups.push(__mc);
        }
        if (__lc.unmount) __cleanups.push(() => __lc.unmount(__sp));
      }
    }
    return $scope;
  }
});
/*! _compilerVersion {"cargo":"0.1.0","git":"d69f9713","dirty":true,"builtAt":"2026-05-16T18:56:56Z"} */
//# sourceMappingURL=main.js.map
