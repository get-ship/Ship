// packages/sui/runtime/src/error-boundary.ts
var BOUNDARY_KEY = Symbol.for("sui:error-boundary");
function attachBoundary(host, handle) {
  host[BOUNDARY_KEY] = handle;
}
function detachBoundary(host) {
  delete host[BOUNDARY_KEY];
}
function findNearestBoundary(node) {
  let cur = node;
  while (cur) {
    if (cur instanceof Element) {
      const handle = cur[BOUNDARY_KEY];
      if (handle)
        return handle;
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
      return;
    }
    console.error("[sui] uncaught throw (no error boundary):", err);
    return;
  }
}

// packages/sui/runtime/src/signal.ts
var CURRENT_OBSERVER = null;
var CURRENT_UNTRACKS = null;
var BATCH_DEPTH = 0;
var PENDING = new Set;
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
  if (CURRENT_OBSERVER === null)
    return;
  if (subs.has(CURRENT_OBSERVER))
    return;
  const sub = CURRENT_OBSERVER;
  subs.add(sub);
  if (CURRENT_UNTRACKS !== null) {
    CURRENT_UNTRACKS.push(() => subs.delete(sub));
  }
}
function signal(initial) {
  let value = initial;
  const subs = new Set;
  const sig = () => {
    track(subs);
    return value;
  };
  sig.set = (next) => {
    const v = typeof next === "function" ? next(value) : next;
    if (Object.is(v, value))
      return;
    value = v;
    if (BATCH_DEPTH > 0) {
      for (const s of subs)
        PENDING.add(s);
    } else {
      for (const s of [...subs])
        s();
    }
  };
  sig.__subs = subs;
  sig.__peek = () => value;
  return sig;
}
function derived(fn) {
  let value;
  let dirty = true;
  const subs = new Set;
  let untracks = [];
  const invalidate = () => {
    if (dirty)
      return;
    dirty = true;
    if (subs.size === 0)
      return;
    const prev = value;
    recompute();
    if (Object.is(prev, value))
      return;
    for (const s of [...subs])
      s();
  };
  const recompute = () => {
    for (const u of untracks)
      u();
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
    if (dirty)
      recompute();
    return value;
  };
  reader.set = () => {
    throw new Error("cannot set on a derived; mutate its dependencies instead");
  };
  reader.__subs = subs;
  reader.__peek = () => {
    if (dirty)
      recompute();
    return value;
  };
  return reader;
}
function effect(fn) {
  let cleanup = undefined;
  let active = true;
  let untracks = [];
  const host = currentHost();
  const run = () => {
    if (!active)
      return;
    if (cleanup) {
      try {
        cleanup();
      } catch (e) {
        console.error("[sui] effect cleanup threw:", e);
      }
      cleanup = undefined;
    }
    for (const u of untracks)
      u();
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
    for (const u of untracks)
      u();
    untracks = [];
    if (cleanup) {
      try {
        cleanup();
      } catch (e) {
        console.error("[sui] effect cleanup threw:", e);
      }
      cleanup = undefined;
    }
  };
}
function selector(source) {
  const buckets = new Map;
  let current = source.__peek();
  effect(() => {
    const next = source();
    if (Object.is(next, current))
      return;
    notifyBucket(buckets.get(current));
    notifyBucket(buckets.get(next));
    current = next;
  });
  return (key) => {
    if (CURRENT_OBSERVER !== null) {
      let bucket = buckets.get(key);
      if (!bucket) {
        bucket = new Set;
        buckets.set(key, bucket);
      }
      const obs = CURRENT_OBSERVER;
      if (!bucket.has(obs)) {
        bucket.add(obs);
        if (CURRENT_UNTRACKS !== null) {
          CURRENT_UNTRACKS.push(() => bucket.delete(obs));
        }
      }
    }
    return Object.is(key, current);
  };
}
function notifyBucket(set) {
  if (!set || set.size === 0)
    return;
  if (BATCH_DEPTH > 0) {
    for (const s of set)
      PENDING.add(s);
  } else {
    for (const s of [...set])
      s();
  }
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
      for (const s of subs)
        s();
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
// packages/sui/runtime/src/effect-scope.ts
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
  if (!scope)
    return;
  scope.cleanup(fn);
}
function currentScope() {
  return CURRENT_SCOPE;
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
      if (disposed)
        return;
      disposed = true;
      for (let i = cleanups.length - 1;i >= 0; i--) {
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
function createChildScope(parent) {
  const owner = parent ?? CURRENT_SCOPE;
  const child = createScope();
  if (owner)
    owner.cleanup(() => child.dispose());
  return child;
}
// packages/sui/runtime/src/context.ts
var SHARED = Symbol("sui.shared");
function shared(sig) {
  sig[SHARED] = true;
  return sig;
}
function isShared(sig) {
  return !!sig?.[SHARED];
}
function provide(host, key, value) {
  host[`__sui_ctx_${key}`] = value;
}
function useShared(host, key) {
  let cur = host;
  while (cur) {
    const v = cur[`__sui_ctx_${key}`];
    if (v !== undefined)
      return v;
    const parent = cur.parentElement;
    if (parent) {
      cur = parent;
      continue;
    }
    const root = cur.getRootNode();
    cur = root instanceof ShadowRoot ? root.host : null;
  }
  return null;
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
// packages/sui/runtime/src/ref.ts
function ref(_initial = null) {
  return signal(null);
}
function attachRef(scope, varName, el) {
  const existing = scope[varName];
  if (typeof existing === "function" && typeof existing.set === "function") {
    existing.set(el);
    return existing;
  }
  const slot = signal(el);
  scope[varName] = slot;
  return slot;
}
// packages/sui/runtime/src/slot-empty.ts
function bindSlotEmpty(slot, host, attrName) {
  const sync = () => {
    const nodes = slot.assignedNodes({ flatten: true });
    const hasText = nodes.some(hasVisibleContent);
    if (hasText)
      host.removeAttribute(attrName);
    else
      host.setAttribute(attrName, "");
  };
  slot.addEventListener("slotchange", sync);
  sync();
  return () => {
    slot.removeEventListener("slotchange", sync);
  };
}
function hasVisibleContent(n) {
  if (n.nodeType === Node.TEXT_NODE) {
    return (n.textContent ?? "").trim().length > 0;
  }
  if (n.nodeType === Node.ELEMENT_NODE) {
    const el = n;
    return (el.textContent ?? "").trim().length > 0 || el.childElementCount > 0;
  }
  return false;
}
// packages/sui/runtime/src/morph.ts
function morph(target, source, opts = {}) {
  if (typeof source === "string") {
    const tag = target.tagName.toLowerCase();
    const tmp = target.namespaceURI === "http://www.w3.org/2000/svg" ? target.ownerDocument.createElementNS("http://www.w3.org/2000/svg", tag) : target.ownerDocument.createElement(tag);
    tmp.innerHTML = source;
    morphChildren(target, tmp, opts);
    return;
  }
  morphElement(target, source, opts);
}
function morphElement(target, source, opts) {
  if (opts.skip?.(target))
    return;
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
    if (!source.hasAttribute(a.name))
      target.removeAttribute(a.name);
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
  const targetById = new Map;
  for (const c of Array.from(target.children)) {
    if (c.id)
      targetById.set(c.id, c);
  }
  const sourceIds = new Set;
  for (const c of Array.from(source.children)) {
    if (c.id)
      sourceIds.add(c.id);
  }
  const sChildren = Array.from(source.childNodes);
  let cursor = target.firstChild;
  for (const s of sChildren) {
    while (cursor && cursor.nodeType === Node.ELEMENT_NODE && cursor.id && sourceIds.has(cursor.id) && !(s.nodeType === Node.ELEMENT_NODE && s.id === cursor.id)) {
      cursor = cursor.nextSibling;
    }
    let claim = null;
    if (s.nodeType === Node.ELEMENT_NODE) {
      const sId = s.id;
      if (sId && targetById.has(sId)) {
        claim = targetById.get(sId);
        targetById.delete(sId);
      }
    }
    if (!claim && cursor && nodesAlign(cursor, s)) {
      claim = cursor;
    }
    if (claim) {
      if (claim === cursor) {
        cursor = cursor.nextSibling;
      } else {
        target.insertBefore(claim, cursor);
      }
      morphNode(claim, s, target, opts);
    } else {
      target.insertBefore(s.cloneNode(true), cursor);
    }
  }
  while (cursor) {
    const next = cursor.nextSibling;
    target.removeChild(cursor);
    cursor = next;
  }
}
function nodesAlign(a, b) {
  if (a.nodeType !== b.nodeType)
    return false;
  if (a.nodeType === Node.ELEMENT_NODE) {
    return a.tagName === b.tagName;
  }
  return true;
}
function morphNode(t, s, parent, opts) {
  if (t.nodeType !== s.nodeType) {
    parent.replaceChild(s.cloneNode(true), t);
    return;
  }
  if (t.nodeType === Node.TEXT_NODE || t.nodeType === Node.COMMENT_NODE) {
    if (t.nodeValue !== s.nodeValue)
      t.nodeValue = s.nodeValue;
    return;
  }
  if (t.nodeType === Node.ELEMENT_NODE) {
    morphElement(t, s, opts);
  }
}

// packages/sui/runtime/src/bind.ts
function bindText(node, get) {
  let prev;
  return effect(() => {
    const v = get();
    const next = v == null ? "" : String(v);
    if (next === prev)
      return;
    if (prev === undefined && node.textContent === next) {
      prev = next;
      return;
    }
    node.textContent = next;
    prev = next;
  });
}
function bindShow(el, get) {
  let prev = null;
  return effect(() => {
    if (get()) {
      if (prev !== null)
        el.style.display = prev;
      prev = null;
    } else {
      if (prev === null)
        prev = el.style.display;
      el.style.display = "none";
    }
  });
}
function bindAttr(el, name, get) {
  let prev;
  let init = false;
  return effect(() => {
    const v = get();
    if (init && v === prev)
      return;
    const next = v == null || v === false || v === "" ? null : v === true ? "" : String(v);
    if (init || el.getAttribute(name) !== next) {
      if (next === null)
        el.removeAttribute(name);
      else
        el.setAttribute(name, next);
    }
    prev = v;
    init = true;
  });
}
function bindClass(el, name, get) {
  let prev;
  return effect(() => {
    const v = !!get();
    if (v === prev)
      return;
    if (prev === undefined && v === el.classList.contains(name)) {
      prev = v;
      return;
    }
    if (v)
      el.classList.add(name);
    else
      el.classList.remove(name);
    prev = v;
  });
}
function bindProp(el, prop, get) {
  return effect(() => {
    el[prop] = get();
  });
}
function bindValue(el, sig, onWrite) {
  const dispose = effect(() => {
    const v = sig();
    if (el.value !== v)
      el.value = v;
  });
  const onInput = () => {
    const v = el.value;
    sig.set(v);
    if (onWrite)
      onWrite(v);
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
    if (el.checked !== v)
      el.checked = v;
  });
  const onChange = () => sig.set(el.checked);
  el.addEventListener("change", onChange);
  return () => {
    dispose();
    el.removeEventListener("change", onChange);
  };
}
function bindHtml(el, get) {
  const SVG_NS = "http://www.w3.org/2000/svg";
  const isSvg = el.namespaceURI === SVG_NS;
  const tag = el.tagName.toLowerCase();
  return effect(() => {
    const html = get();
    const tmp = isSvg ? el.ownerDocument.createElementNS(SVG_NS, tag) : el.ownerDocument.createElement(tag);
    tmp.innerHTML = html;
    morph(el, tmp);
  });
}
// packages/sui/runtime/src/on.ts
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
          if (e.target !== el)
            return;
          break;
        case "trusted":
          if (!e.isTrusted)
            return;
          break;
        case "key": {
          if (!matchesKey(e, m.arg ?? ""))
            return;
          break;
        }
      }
    }
    const debounceMod = modifiers.find((m) => m.kind === "debounce");
    const throttleMod = modifiers.find((m) => m.kind === "throttle");
    if (debounceMod) {
      const ms = Number(debounceMod.arg) || 0;
      if (timer !== undefined)
        clearTimeout(timer);
      timer = setTimeout(() => {
        timer = undefined;
        handler(e);
      }, ms);
      return;
    }
    if (throttleMod) {
      const ms = Number(throttleMod.arg) || 0;
      if (!throttleArmed)
        return;
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
  if (opts.capture)
    listenerOpts.capture = true;
  if (opts.passive)
    listenerOpts.passive = true;
  if (opts.once)
    listenerOpts.once = true;
  el.addEventListener(event, wrapped, listenerOpts);
  return () => {
    el.removeEventListener(event, wrapped, listenerOpts);
    if (timer !== undefined)
      clearTimeout(timer);
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
  if (wantMeta && !e.metaKey)
    return false;
  if (wantCtrl && !e.ctrlKey)
    return false;
  if (wantShift && !e.shiftKey)
    return false;
  if (wantAlt && !e.altKey)
    return false;
  if (wantMod && !(isMac ? e.metaKey : e.ctrlKey))
    return false;
  return e.key.toLowerCase() === key;
}
// packages/sui/runtime/src/host-api.ts
function emit(host, name, detail) {
  host.dispatchEvent(new CustomEvent(name, { detail, bubbles: true, composed: true }));
}
function expose(host, methods) {
  const target = host;
  const names = Object.keys(methods);
  for (const name of names)
    target[name] = methods[name];
  onCleanup(() => {
    for (const name of names) {
      if (target[name] === methods[name]) {
        try {
          delete target[name];
        } catch {
          target[name] = undefined;
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
function bindNativeEvent(target, event, handler, opts) {
  target.addEventListener(event, handler, opts);
  onCleanup(() => target.removeEventListener(event, handler, opts));
}
function slotted(slot, render) {
  const stop = effect(() => {
    const s = slot();
    if (!s)
      return;
    const update = () => {
      render(s.assignedElements());
    };
    s.addEventListener("slotchange", update);
    update();
    return () => s.removeEventListener("slotchange", update);
  });
  onCleanup(stop);
}
function bindHostAttr(host, name, get) {
  onCleanup(bindAttr(host, name, get));
}
// packages/sui/runtime/src/transition-runner.ts
var _runEnter = () => {};
var _runLeave = () => null;
var _cancelTransitions = () => {};
var runEnter = (nodes) => _runEnter(nodes);
var runLeave = (nodes) => _runLeave(nodes);
var cancelTransitions = (node) => _cancelTransitions(node);

// packages/sui/runtime/src/for.ts
function computeLisMaskInto(matched, count, mask, pred, tails) {
  mask.fill(0, 0, count);
  if (count === 0)
    return;
  let tailsLen = 0;
  for (let i = 0;i < count; i++) {
    const v = matched[2 * i + 1];
    let lo = 0, hi = tailsLen;
    while (lo < hi) {
      const mid = lo + hi >>> 1;
      if (matched[2 * tails[mid] + 1] < v)
        lo = mid + 1;
      else
        hi = mid;
    }
    pred[i] = lo > 0 ? tails[lo - 1] : -1;
    tails[lo] = i;
    if (lo === tailsLen)
      tailsLen++;
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
  const departing = new Set;
  const nextKeys = new Map;
  const prevByKey = new Map;
  const prevPos = new Map;
  const stable = new Set;
  const freshNodes = [];
  let nextRecords = [];
  let matched = new Int32Array(0);
  let lisMaskBuf = new Uint8Array(0), lisPred = new Int32Array(0), lisTails = new Int32Array(0);
  const flush = (r) => {
    for (const n of r.nodes) {
      try {
        n.parentNode?.removeChild(n);
      } catch {}
    }
    r.cleanup?.();
  };
  let lastItems = null;
  const dispose = effect(() => {
    if (!active)
      return;
    const next = items();
    if (next === lastItems)
      return;
    lastItems = next;
    const n = next.length;
    if (matched.length < n * 2)
      matched = new Int32Array(Math.max(matched.length * 2, n * 2, 16));
    if (lisMaskBuf.length < n) {
      const cap = Math.max(lisMaskBuf.length * 2, n, 8);
      lisMaskBuf = new Uint8Array(cap);
      lisPred = new Int32Array(cap);
      lisTails = new Int32Array(cap);
    }
    for (let i = 0;i < n; i++) {
      const item = next[i];
      const k = key(item, i);
      if (nextKeys.has(k)) {
        console.warn("[sui] {#for}: duplicate key", k, "at index", i, "(prior at", nextKeys.get(k), "); the earlier item will be dropped from the rendered output.", "Fix the key function so each item maps to a unique value.");
      }
      nextKeys.set(k, i);
    }
    for (const r of prev)
      prevByKey.set(r.key, r);
    for (const r of prev) {
      if (nextKeys.has(r.key))
        continue;
      const leaving = runLeave(r.nodes);
      if (leaving === null) {
        flush(r);
        continue;
      }
      departing.add(r);
      leaving.then(() => {
        if (!departing.delete(r))
          return;
        flush(r);
      });
    }
    nextRecords.length = 0;
    freshNodes.length = 0;
    let anyReused = false;
    for (let i = 0;i < n; i++) {
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
      for (let i = 0;i < prev.length; i++)
        prevPos.set(prev[i].key, i);
      let monotone = true;
      let lastPrevPos = -1;
      for (let i = 0;i < nextRecords.length; i++) {
        const p = prevPos.get(nextRecords[i].key);
        if (p !== undefined) {
          matched[2 * matchedCount] = i;
          matched[2 * matchedCount + 1] = p;
          matchedCount++;
          if (monotone && p <= lastPrevPos)
            monotone = false;
          lastPrevPos = p;
        }
      }
      if (monotone) {
        for (let j = 0;j < matchedCount; j++)
          stable.add(matched[2 * j]);
      } else {
        computeLisMaskInto(matched, matchedCount, lisMaskBuf, lisPred, lisTails);
        for (let j = 0;j < matchedCount; j++) {
          if (lisMaskBuf[j])
            stable.add(matched[2 * j]);
        }
      }
    }
    const parent = anchor.parentNode;
    if (parent && matchedCount === 0 && nextRecords.length > 0) {
      const frag = document.createDocumentFragment();
      for (let i = 0;i < nextRecords.length; i++) {
        const nodes = nextRecords[i].nodes;
        for (let j = 0;j < nodes.length; j++)
          frag.appendChild(nodes[j]);
      }
      parent.insertBefore(frag, anchor.nextSibling);
    } else {
      let cursor = anchor;
      for (let i = 0;i < nextRecords.length; i++) {
        if (!parent)
          break;
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
    if (freshNodes.length > 0)
      runEnter(freshNodes);
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
    for (const r of prev)
      flush(r);
    for (const d of departing)
      flush(d);
    departing.clear();
    prev = [];
    nextRecords.length = 0;
    lastItems = null;
  };
}
function bindForSignal(anchor, sig, key, render) {
  return bindFor({ anchor, items: () => sig(), key, render });
}
// packages/sui/runtime/src/if.ts
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
      } catch {}
    }
  };
  const dispose = effect(() => {
    const idx = test();
    if (active && idx === lastIdx)
      return;
    if (pendingOut && idx === pendingOut.idx && idx >= 0 && idx < branches.length) {
      const promoted = pendingOut;
      pendingOut = null;
      for (const n of promoted.nodes)
        cancelTransitions(n);
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
          if (pendingOut !== dep)
            return;
          removeNodes(dep.nodes);
          dep.cleanup?.();
          pendingOut = null;
        });
      }
    }
    lastIdx = idx;
    if (idx < 0 || idx >= branches.length)
      return;
    const rendered = runProtected(anchor, "render", () => branches[idx]());
    if (!rendered)
      return;
    active = rendered;
    const parent = anchor.parentNode;
    if (!parent)
      return;
    for (const n of active.nodes)
      parent.insertBefore(n, anchor);
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
// packages/sui/runtime/src/transition.ts
var REG = Symbol("sui:transition");
var MARK = "data-sui-transition";
function getReg(node) {
  return node[REG];
}
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
  if (!node.hasAttribute(MARK))
    node.setAttribute(MARK, "");
}
function reducedMotion() {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return false;
  }
  try {
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  } catch {
    return false;
  }
}
function applyReduced(o) {
  if (!reducedMotion())
    return o;
  return { ...o, duration: 0, delay: 0 };
}
function runIn(node) {
  const r = getReg(node);
  if (!r?.inT?.in)
    return;
  r.current?.cancel();
  const anim = r.inT.in(node, applyReduced(r.inOpts));
  r.current = anim;
  return anim;
}
function runOut(node) {
  const r = getReg(node);
  if (!r?.outT?.out)
    return;
  r.current?.cancel();
  const anim = r.outT.out(node, applyReduced(r.outOpts));
  r.current = anim;
  return anim;
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
  if (typeof fn !== "function")
    return;
  return fn.call(node, frames, t);
}
var fade = {
  in: (node, o) => animateSafe(node, [{ opacity: 0 }, { opacity: 1 }], timing(o)),
  out: (node, o) => animateSafe(node, [{ opacity: 1 }, { opacity: 0 }], timing(o))
};
function slideFrames(axis, dir) {
  const off = axis === "y" ? "translateY(-100%)" : "translateX(-100%)";
  const on2 = "translate(0, 0)";
  return dir === "in" ? [{ opacity: 0, transform: off }, { opacity: 1, transform: on2 }] : [{ opacity: 1, transform: on2 }, { opacity: 0, transform: off }];
}
var slide = {
  in: (node, o) => {
    const axis = o.axis === "x" ? "x" : "y";
    return animateSafe(node, slideFrames(axis, "in"), timing(o));
  },
  out: (node, o) => {
    const axis = o.axis === "x" ? "x" : "y";
    return animateSafe(node, slideFrames(axis, "out"), timing(o));
  }
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
// packages/sui/runtime/src/await.ts
function bindAwait(opts) {
  const { anchor, promise, loading, then_, catch_ } = opts;
  let active = null;
  let token = 0;
  const tearDown = () => {
    if (!active)
      return;
    for (const n of active.nodes) {
      try {
        n.parentNode?.removeChild(n);
      } catch {}
    }
    active.cleanup?.();
    active = null;
  };
  const insert = (br) => {
    active = br;
    const parent = anchor.parentNode;
    if (!parent)
      return;
    for (const n of br.nodes)
      parent.insertBefore(n, anchor);
  };
  const dispose = effect(() => {
    const myToken = ++token;
    const p = promise();
    tearDown();
    insert(loading());
    p.then((v) => {
      if (myToken !== token)
        return;
      if (then_) {
        tearDown();
        insert(then_(v));
      }
    }, (e) => {
      if (myToken !== token)
        return;
      if (catch_) {
        tearDown();
        insert(catch_(e));
      }
    });
  });
  return () => {
    token++;
    dispose();
    tearDown();
  };
}
// packages/sui/runtime/src/raw-html.ts
function bindRawHtml(el, get, deps = []) {
  return effect(() => {
    for (const d of deps)
      d();
    const next = String(get() ?? "");
    if (el.hasAttribute("morph")) {
      morph(el, next);
    } else {
      el.innerHTML = next;
    }
  });
}
// packages/sui/runtime/src/slot.ts
function resolveSlots(host, slots, originalChildren) {
  const fills = new Map;
  for (const child of originalChildren) {
    let name = "default";
    if (child instanceof Element) {
      name = child.getAttribute("slot") ?? "default";
      if (child.hasAttribute("slot"))
        child.removeAttribute("slot");
    }
    const list = fills.get(name) ?? [];
    list.push(child);
    fills.set(name, list);
  }
  const filledNames = new Set;
  for (const [name, nodes] of fills) {
    const hasMeaning = nodes.some((n) => {
      if (n.nodeType === 1)
        return true;
      if (n.nodeType === 3)
        return (n.textContent || "").trim().length > 0;
      return false;
    });
    if (hasMeaning)
      filledNames.add(name);
  }
  host.__shipSetSlotFilled = filledNames;
  for (const slot of slots) {
    if (!slot || !slot.anchor)
      continue;
    const filled = fills.get(slot.name) ?? [];
    const parent = slot.anchor.parentNode;
    if (!parent)
      continue;
    if (filled.length > 0) {
      for (const n of filled)
        parent.insertBefore(n, slot.anchor);
    } else if (slot.fallback) {
      const nodes = slot.fallback();
      for (const n of nodes)
        parent.insertBefore(n, slot.anchor);
    }
  }
  for (const slot of slots) {
    if (slot && slot.anchor)
      slot.anchor.remove();
  }
}
function slotFilled(host, name) {
  const set = host.__shipSetSlotFilled;
  return set?.has(name) ?? false;
}
function snapshotChildren(host) {
  return Array.from(host.childNodes);
}
// packages/sui/runtime/src/decode.ts
var decodeString = (raw) => raw ?? "";
var decodeNumber = (raw) => {
  if (raw === null || raw === "")
    return NaN;
  return Number(raw);
};
var decodeBoolean = (raw) => {
  if (raw === null)
    return false;
  const v = raw.toLowerCase();
  if (v === "false")
    return false;
  return true;
};
function decodeJson(fallback) {
  return (raw) => {
    if (raw === null || raw === "")
      return fallback;
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

// packages/sui/runtime/src/component-decoders.ts
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

// packages/sui/runtime/src/component-form.ts
function installFormHooks(host, internals, def, propSignals) {
  if (!internals)
    return;
  const hooks = {};
  let stopValueEffect;
  const valueAttr = def.formValue;
  if (valueAttr !== undefined) {
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
  if (disabledAttr !== undefined) {
    const { sig, propDef } = resolveProp(def, propSignals, disabledAttr, "formDisabled");
    if (propDef.type !== "bool") {
      throw new Error(`[sui-runtime] formDisabled: "${disabledAttr}" must be a bool-typed prop`);
    }
    hooks.disabled = (d) => {
      sig.set(!!d);
    };
  }
  host.__shipSetFormHooks = hooks;
  if (stopValueEffect) {
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
  if (v === null || v === undefined)
    return "";
  if (typeof v === "boolean")
    return v ? "on" : "";
  return String(v);
}
function resolveProp(def, propSignals, attr, channel) {
  const propDef = def.props[attr];
  if (!propDef) {
    throw new Error(`[sui-runtime] ${channel}: "${attr}" — no matching prop on this component`);
  }
  const sig = propSignals[camelOf(attr)];
  if (!sig) {
    throw new Error(`[sui-runtime] ${channel}: "${attr}" — prop signal missing`);
  }
  return { sig, propDef };
}

// packages/sui/runtime/src/component-factory.ts
var prop = {
  string(opts = {}) {
    return { type: "string", ...opts };
  },
  bool(opts = {}) {
    return { type: "bool", ...opts };
  },
  number(opts = {}) {
    return { type: "number", ...opts };
  },
  union(values, opts = {}) {
    return { type: "union", values, ...opts };
  },
  object(opts = {}) {
    return { type: "object", ...opts };
  },
  array(opts = {}) {
    return { type: "array", ...opts };
  }
};
var encodeAttr = (def, value) => {
  if (value === undefined || value === null)
    return null;
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
  if (document.getElementById(id))
    return;
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
  const associated = def.formAssociated === true || def.formValue !== undefined || def.formDisabled !== undefined;
  const schema = { observedAttrs, props };
  if (associated)
    schema.formAssociated = true;
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
          if (next === null)
            host.removeAttribute(attr);
          else
            host.setAttribute(attr, next);
        });
        cleanups.push(dispose);
      }
    }
    if (def.formValue !== undefined || def.formDisabled !== undefined) {
      const stopValue = installFormHooks(host, internals, def, propSignals);
      if (typeof stopValue === "function")
        cleanups.push(stopValue);
    }
    if (def.setup) {
      const ctx = { props: propSignals, host, cleanups };
      if (internals)
        ctx.internals = internals;
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

// packages/sui/runtime/src/component.ts
var warnedDuplicateTags = new Set;
function defineComponent(tag, schema, factory) {
  if (customElements.get(tag)) {
    if (!warnedDuplicateTags.has(tag)) {
      warnedDuplicateTags.add(tag);
      console.warn(`[sui-runtime] customElements.define("${tag}", …) called twice. ` + `The first registration wins; this call is a no-op. Likely cause: ` + `two modules independently registering the same tag.`);
    }
    return;
  }
  if (schema.formAssociated) {

    class ShipSetFormElement extends HTMLElement {
      static formAssociated = true;
      static get observedAttributes() {
        return schema.observedAttrs;
      }
      cleanup;
      scope;
      propSignals = {};
      internals;
      constructor() {
        super();
        this.internals = this.attachInternals();
      }
      connectedCallback() {
        this.dataset.component = tag;
        attachHostApi(this);
        this.scope = createScope();
        try {
          this.cleanup = withScope(this.scope, () => withHost(this, () => factory(this, this.propSignals, this.internals)));
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
        this.cleanup = undefined;
        try {
          this.scope?.dispose();
        } catch (e) {
          console.error(`[ship-set:${tag}] scope dispose threw:`, e);
        }
        this.scope = undefined;
      }
      attributeChangedCallback(attrName, _old, value) {
        const meta = schema.props[attrName];
        if (!meta)
          return;
        const sig = this.propSignals[meta.camel];
        if (sig)
          sig.set(meta.decode(value));
      }
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
        this.dispatchEvent(new CustomEvent("ship-set:form-restore", { detail: { state, mode } }));
      }
      formDisabledCallback(disabled) {
        const hooks = this.__shipSetFormHooks;
        try {
          hooks?.disabled?.(disabled);
        } catch (e) {
          console.error(`[ship-set:${tag}] formDisabledCallback threw:`, e);
        }
        this.dispatchEvent(new CustomEvent("ship-set:form-disabled", { detail: { disabled } }));
      }
    }
    customElements.define(tag, ShipSetFormElement);
    return;
  }

  class ShipSetElement extends HTMLElement {
    static get observedAttributes() {
      return schema.observedAttrs;
    }
    cleanup;
    scope;
    propSignals = {};
    connectedCallback() {
      this.dataset.component = tag;
      attachHostApi(this);
      this.scope = createScope();
      try {
        this.cleanup = withScope(this.scope, () => withHost(this, () => factory(this, this.propSignals)));
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
      this.cleanup = undefined;
      try {
        this.scope?.dispose();
      } catch (e) {
        console.error(`[ship-set:${tag}] scope dispose threw:`, e);
      }
      this.scope = undefined;
    }
    attributeChangedCallback(attrName, _old, value) {
      const meta = schema.props[attrName];
      if (!meta)
        return;
      const sig = this.propSignals[meta.camel];
      if (sig)
        sig.set(meta.decode(value));
    }
  }
  customElements.define(tag, ShipSetElement);
}
function camelOf(attr) {
  return attr.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
}
// packages/sui/runtime/src/patch.ts
function applyPatch(sig, patch) {
  const next = applyOp(sig.__peek(), patch);
  sig.set(next);
}
function applyPatches(sig, patches) {
  if (patches.length === 0)
    return;
  batch(() => {
    for (const p of patches)
      applyPatch(sig, p);
  });
}
function tokenizePath(path) {
  if (!path)
    return [];
  const tokens = [];
  const re = /([A-Za-z_$][\w$]*)|\[(\d+)\]/g;
  let m;
  let lastEnd = 0;
  while ((m = re.exec(path)) !== null) {
    if (m.index !== lastEnd && lastEnd !== 0 && path[lastEnd] !== ".") {
      throw new Error(`malformed path at byte ${lastEnd}: ${path}`);
    }
    if (m[1] !== undefined)
      tokens.push({ kind: "field", name: m[1] });
    else if (m[2] !== undefined)
      tokens.push({ kind: "index", i: Number(m[2]) });
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
        if (from < 0 || from >= arr.length)
          return arr;
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
  if (path.length === 0)
    return value;
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
  if (path.length === 0)
    return;
  const parentPath = path.slice(0, -1);
  const last = path[path.length - 1];
  return rebuild(root, parentPath, (parent) => {
    if (last.kind === "index") {
      if (!Array.isArray(parent))
        throw new Error("remove[index] requires array");
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
    if (!Array.isArray(current))
      throw new Error("expected array at path");
    return mod(current);
  });
}
function mapAt(root, path, fn) {
  return rebuild(root, path, fn);
}
function rebuild(root, path, mut) {
  if (path.length === 0)
    return mut(root);
  const head = path[0];
  const tail = path.slice(1);
  if (head.kind === "index") {
    if (!Array.isArray(root))
      throw new Error("expected array at path");
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

// packages/sui/runtime/src/data.ts
function data(opts) {
  const sig = signal(opts.initial);
  const ESCtor = opts.EventSource ?? globalThis.EventSource;
  if (!opts.subscribe || !ESCtor) {
    return { signal: sig, dispose: () => {} };
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
      if (opts.onPatch)
        opts.onPatch(patch, sig);
      else
        applyPatch(sig, patch);
    } catch (err) {
      opts.onError?.(err);
    }
  };
  const onError = (err) => opts.onError?.(err);
  sse.addEventListener("patch", onMessage);
  sse.addEventListener("error", onError);
  let disposed = false;
  const dispose = () => {
    if (disposed)
      return;
    disposed = true;
    sse.removeEventListener("patch", onMessage);
    sse.removeEventListener("error", onError);
    sse.close();
  };
  onCleanup(dispose);
  return { signal: sig, dispose };
}
// packages/sui/runtime/src/state.ts
function debounce(input, ms) {
  const out = signal(untrack(() => input()));
  let timer = null;
  effect(() => {
    const value = input();
    if (timer !== null)
      clearTimeout(timer);
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
// packages/sui/runtime/src/live.ts
var toError = (err) => err instanceof Error ? err : new Error(String(err));
var isLiveSource = (v) => typeof v === "object" && v !== null && typeof v.subscribe === "function";
function live(source, opts) {
  const sig = signal({ kind: "loading" });
  let token = 0;
  const stop = effect(() => {
    const myToken = ++token;
    let result;
    try {
      result = source();
    } catch (err) {
      if (myToken !== token)
        return;
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
        if (disposed)
          return;
        disposed = true;
        try {
          unsub();
        } catch (err) {
          opts?.onError?.(err);
        }
      };
    }
    Promise.resolve(result).then((value) => {
      if (myToken !== token)
        return;
      sig.set({ kind: "ok", value });
    }, (err) => {
      if (myToken !== token)
        return;
      const error = toError(err);
      sig.set({ kind: "error", error });
      opts?.onError?.(err);
    });
    return;
  });
  onCleanup(stop);
  return sig;
}
function unwrap(sig) {
  const s = sig();
  return s.kind === "ok" ? s.value : undefined;
}
function unwrapOr(sig, fallback) {
  const s = sig();
  return s.kind === "ok" ? s.value : fallback;
}
// packages/sui/runtime/src/mutation.ts
var toError2 = (err) => err instanceof Error ? err : new Error(String(err));
function resolveChain(scope, chain) {
  const parts = chain.split(".");
  let cur = scope;
  for (const part of parts) {
    if (cur == null || typeof cur !== "object")
      return;
    cur = cur[part];
  }
  return typeof cur === "function" ? cur : undefined;
}
function wireFormMutation(el, chain, scope) {
  const mut = signal({ kind: "ok", value: undefined });
  scope["$mutation"] = mut;
  const listener = async (e) => {
    e.preventDefault();
    mut.set({ kind: "loading" });
    const fn = resolveChain(scope, chain);
    if (!fn) {
      const error = new Error(`[ship-set] mutation chain "${chain}" not found in scope`);
      console.error(error);
      mut.set({ kind: "error", error });
      el.dispatchEvent(new CustomEvent("error", { detail: { error }, bubbles: true }));
      return;
    }
    const formData = el instanceof HTMLFormElement ? new FormData(el) : new FormData;
    try {
      const value = await Promise.resolve(fn(formData));
      mut.set({ kind: "ok", value });
      el.dispatchEvent(new CustomEvent("success", { detail: { value }, bubbles: true }));
    } catch (err) {
      const error = toError2(err);
      mut.set({ kind: "error", error });
      el.dispatchEvent(new CustomEvent("error", { detail: { error }, bubbles: true }));
    }
  };
  el.addEventListener("submit", listener);
  return () => el.removeEventListener("submit", listener);
}
// packages/sui/runtime/src/while.ts
var tearDownBranch = (active) => {
  if (!active)
    return null;
  for (const n of active.nodes) {
    try {
      n.parentNode?.removeChild(n);
    } catch {}
  }
  active.cleanup?.();
  return null;
};
var insertBranch = (anchor, br) => {
  const parent = anchor.parentNode;
  if (!parent)
    return;
  for (const n of br.nodes)
    parent.insertBefore(n, anchor);
};
function bindWhileLoading(opts) {
  const { anchor, signal: sig, render } = opts;
  let active = null;
  let mounted = false;
  const dispose = effect(() => {
    const want = sig().kind === "loading";
    if (want === mounted)
      return;
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
    if (want === mounted)
      return;
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
// packages/sui/runtime/src/controller-hook-resolver.ts
function resolveHook(hook, ctx, outerScope) {
  const hookState = resolveHookState(hook.state, { ship: ctx.ship, ...outerScope });
  const stateSignals = {};
  for (const [k, v] of Object.entries(hookState)) {
    stateSignals[k] = signal(v);
  }
  const baseScope = {
    ship: ctx.ship,
    route: ctx.route ?? {},
    navigate: ctx.navigate ?? ((_path) => {}),
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
  if (section === undefined)
    return {};
  if (typeof section === "function") {
    return section(scope);
  }
  return section;
}
function buildHookWritableProxy(stateSignals, baseScope) {
  return new Proxy({}, {
    get(_target, key) {
      if (key in stateSignals)
        return stateSignals[key]();
      return baseScope[key];
    },
    set(_target, key, value) {
      if (key in stateSignals) {
        stateSignals[key].set(value);
        return true;
      }
      console.warn(`[ship-set] hook handler tried to write s.${key} = … but ${key} is not a hook state key. The assignment was ignored.`);
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
      return;
    }
  });
}

// packages/sui/runtime/src/controller-resolver.ts
var LIFECYCLE_KEY = "__shipSetLifecycle";
function resolveController(def, ctx) {
  const shape = normalize(def, ctx);
  const baseScope = {
    ship: ctx.ship,
    route: ctx.route ?? {},
    navigate: ctx.navigate ?? ((_path) => {}),
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
    if (shape.mount)
      lifecycle.mount = shape.mount;
    if (shape.unmount)
      lifecycle.unmount = shape.unmount;
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
  const haveStateKeys = stateKeys !== undefined;
  const scopeAddKeys = scopeAdds ? Object.keys(scopeAdds) : [];
  if (haveStateKeys || scopeAddKeys.length > 0) {
    if (stateKeys) {
      for (const k of stateKeys) {
        const v = scope[k];
        if (isSignal(v))
          out[k] = v;
      }
    }
    for (const k of scopeAddKeys) {
      if (k in out)
        continue;
      const v = scope[k];
      if (isSignal(v))
        out[k] = v;
    }
    return out;
  }
  for (const [k, v] of Object.entries(scope)) {
    if (isSignal(v))
      out[k] = v;
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
      if (key === SIGNALS_KEY)
        return stateSignals;
      if (key in stateSignals)
        return stateSignals[key]();
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
        console.warn(`[ship-set] handler tried to write s.${SIGNALS_KEY} = … but ${SIGNALS_KEY} is the read-only signal-map accessor. The assignment was ignored.`);
        return true;
      }
      console.warn(`[ship-set] handler tried to write s.${key} = … but ${key} is not declared as <state name="${key}"> in the template. The assignment was ignored.`);
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
      return;
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
  if (shape.use === undefined)
    return {};
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
// packages/sui/runtime/src/view.ts
function view(setup) {
  return { setup };
}
// packages/sui/runtime/src/hooks/filtered-list.ts
function readItems(source) {
  const v = source();
  if (Array.isArray(v))
    return v;
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
// packages/sui/runtime/src/hooks/selection.ts
function readItems2(source) {
  const v = source();
  if (Array.isArray(v))
    return v;
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
        if (id === null)
          return null;
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
// packages/sui/runtime/src/hooks/form.ts
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
          out[k] = v === undefined || v === null ? null : v;
        }
        return out;
      },
      valid: (s) => {
        const values = snapshotValues(s, fieldKeys);
        if (!opts.validate)
          return true;
        const validated = opts.validate(values);
        for (const k of fieldKeys) {
          const v = validated[k];
          if (v !== undefined && v !== null)
            return false;
        }
        return true;
      },
      dirty: (s) => {
        for (const k of fieldKeys) {
          const v = s[k]();
          if (!Object.is(v, init[k]))
            return true;
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
            if (v !== undefined && v !== null)
              return;
          }
        }
        if (!opts.submit)
          return;
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
// packages/sui/runtime/src/template-hooks/registry.ts
var registry = new Map;
function registerTemplateHook(hook) {
  registry.set(hook.name, hook);
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
// packages/sui/runtime/src/boundary-elements.ts
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
// packages/sui/runtime/src/styles/index.ts
var TONES_CSS = `:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
` + `:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
` + `:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
` + `:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
` + `:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
` + `:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
` + `:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
` + `:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`;
var BASE_CSS = `:host([hidden]) { display: none !important; }
` + `@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;
function makeSheet(css) {
  if (typeof CSSStyleSheet === "undefined")
    return null;
  try {
    const sheet = new CSSStyleSheet;
    sheet.replaceSync(css);
    return sheet;
  } catch {
    return null;
  }
}
var suiTonesSheet = makeSheet(TONES_CSS);
var suiBaseSheet = makeSheet(BASE_CSS);
function adoptSuiSheets(root) {
  if (typeof ShadowRoot === "undefined")
    return;
  if (!(root instanceof ShadowRoot))
    return;
  const sheets = [suiBaseSheet, suiTonesSheet].filter((s) => s != null);
  if (sheets.length === 0)
    return;
  try {
    root.adoptedStyleSheets = [...sheets, ...root.adoptedStyleSheets ?? []];
  } catch {}
}
// packages/sui/runtime/src/mount.ts
var LIFECYCLE_KEY2 = "__shipSetLifecycle";
async function mount(target, payloadOrName, opts = {}) {
  if (!target)
    throw new Error("[sui/mount] target is required");
  const payload = typeof payloadOrName === "string" ? await fetchPayload(payloadOrName) : payloadOrName;
  seedRouteParams(opts.route, payload.params);
  await loadViewIslands(payload.islands);
  if (hasHydratableMarkup(target)) {
    morph(target, payload.html);
  } else {
    const tmpl = (target.ownerDocument ?? document).createElement("template");
    tmpl.innerHTML = payload.html;
    target.replaceChildren(...Array.from(tmpl.content.childNodes));
  }
  const styleEl = injectCss(target, payload.css, opts.cssScope ?? "adjacent");
  const built = buildScopeFromPayload(payload.scope, opts.restoreScope);
  const mod = opts.module ?? await import(payload.moduleUrl);
  if (!mod?.default || typeof mod.default.setup !== "function") {
    throw new Error(`[sui/mount] module ${payload.moduleUrl} missing default.setup()`);
  }
  const route = opts.route;
  const navigate = opts.navigate ?? noop;
  const baseScope = {
    ship: opts.ship,
    route,
    navigate,
    ...built.scope
  };
  const setupCtx = {
    ship: opts.ship,
    route,
    navigate,
    scope: { ...baseScope },
    stateKeys: built.stateKeys
  };
  const scope = createScope();
  const setupRet = withScope(scope, () => mod.default.setup(setupCtx));
  const finalScope = { ...baseScope };
  if (setupRet)
    Object.assign(finalScope, setupRet);
  for (const k of built.stateKeys) {
    if (built.scope[k])
      finalScope[k] = built.scope[k];
  }
  for (const k of built.liveKeys) {
    if (built.scope[k])
      finalScope[k] = built.scope[k];
  }
  const cleanups = [];
  let disposed = false;
  const hydrate = setupRet?.["hydrate"];
  if (typeof hydrate === "function") {
    try {
      const dispose = hydrate(target, setupCtx);
      if (typeof dispose === "function")
        cleanups.push(dispose);
    } catch (err) {
      console.error("[sui/mount] hydrate threw:", err);
    }
  }
  const lifecycle = extractLifecycle(setupRet);
  if (lifecycle?.mount) {
    try {
      const result = lifecycle.mount(lifecycle.state, target);
      if (typeof result === "function") {
        cleanups.push(result);
      } else if (result && typeof result === "object" && typeof result.then === "function") {
        result.then((asyncResult) => {
          if (typeof asyncResult !== "function")
            return;
          const disposer = asyncResult;
          if (disposed) {
            try {
              disposer();
            } catch (err) {
              console.error("[sui/mount] async cleanup threw:", err);
            }
          } else {
            cleanups.push(disposer);
          }
        }, (err) => {
          console.error("[sui/mount] controller mount() promise rejected:", err);
        });
      }
    } catch (err) {
      console.error("[sui/mount] controller mount() threw:", err);
    }
  }
  const subscribeUrl = pluckSubscribe(payload.scope);
  let liveSource = null;
  if (subscribeUrl && built.liveKeys.size > 0) {
    const liveScope = pickLiveSignals(setupRet, built.scope, built.liveKeys);
    liveSource = openLiveChannel(subscribeUrl, liveScope, built.liveKeys, target);
    cleanups.push(() => liveSource?.close());
    await awaitChannelOpen(liveSource, 3000);
  }
  const handle = {
    element: target,
    scope: built.scope,
    dispose() {
      if (disposed)
        return;
      disposed = true;
      if (lifecycle?.unmount) {
        try {
          lifecycle.unmount(lifecycle.state);
        } catch (err) {
          console.error("[sui/mount] controller unmount() threw:", err);
        }
      }
      for (const fn of cleanups) {
        try {
          fn();
        } catch (err) {
          console.error("[sui/mount] cleanup threw:", err);
        }
      }
      scope.dispose();
      styleEl?.remove();
    },
    async reload(next) {
      const restored = {};
      const nextScope = next.scope ?? {};
      for (const k of Object.keys(nextScope)) {
        const sig = built.scope[k];
        if (sig)
          restored[k] = sig();
      }
      handle.dispose();
      const remounted = await mount(target, next, {
        ...opts,
        restoreScope: restored
      });
      handle.scope = remounted.scope;
      handle.dispose = remounted.dispose;
      handle.reload = remounted.reload;
    }
  };
  return handle;
}
function hasHydratableMarkup(el) {
  if (el.children.length > 0)
    return true;
  for (const n of Array.from(el.childNodes)) {
    if (n.nodeType === Node.TEXT_NODE && (n.nodeValue ?? "").trim() !== "") {
      return true;
    }
  }
  return false;
}
function seedRouteParams(route, params) {
  if (!route || !params)
    return;
  for (const [k, v] of Object.entries(params)) {
    if (typeof v !== "string")
      continue;
    route.params[k]?.set(v);
  }
}
async function loadViewIslands(bundles) {
  if (!bundles || bundles.length === 0)
    return;
  if (typeof customElements === "undefined")
    return;
  await Promise.all(bundles.filter((bundle) => !customElements.get(bundle)).map((bundle) => import(`/_sui/sui-components/${bundle}.js`).catch((err) => {
    console.error(`[sui/mount] failed to load component bundle ${bundle}:`, err);
  })));
}
async function fetchPayload(nameOrPath) {
  const path = nameOrPath.startsWith("/_sui/views/") ? nameOrPath : `/_sui/views/${nameOrPath}`;
  const res = await fetch(path, { credentials: "same-origin" });
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`[sui/mount] fetch ${path} ${res.status}: ${body.slice(0, 200)}`);
  }
  return await res.json();
}
function buildScopeFromPayload(payloadScope, restore) {
  const scope = {};
  const stateKeys = new Set;
  const liveKeys = new Set;
  for (const [k, v] of Object.entries(payloadScope)) {
    if (k === "__subscribe__")
      continue;
    if (isLiveStateEnvelope(v)) {
      const init = restore && k in restore ? restore[k] : v;
      scope[k] = signal(init);
      liveKeys.add(k);
    } else {
      const init = restore && k in restore ? restore[k] : v;
      scope[k] = signal(init);
      stateKeys.add(k);
    }
  }
  return { scope, stateKeys, liveKeys };
}
function pluckSubscribe(payloadScope) {
  const v = payloadScope["__subscribe__"];
  return typeof v === "string" ? v : undefined;
}
function awaitChannelOpen(es, timeoutMs) {
  if (es.readyState === 1)
    return Promise.resolve();
  return new Promise((resolve) => {
    let settled = false;
    const settle = () => {
      if (settled)
        return;
      settled = true;
      es.removeEventListener("open", settle);
      es.removeEventListener("error", settle);
      resolve();
    };
    es.addEventListener("open", settle);
    es.addEventListener("error", settle);
    if (timeoutMs > 0) {
      setTimeout(settle, timeoutMs);
    }
  });
}
function pickLiveSignals(setupRet, fallback, liveKeys) {
  const out = {};
  for (const k of liveKeys) {
    const fromSetup = setupRet?.[k];
    if (typeof fromSetup === "function" && "set" in fromSetup && typeof fromSetup.set === "function") {
      out[k] = fromSetup;
    } else if (fallback[k]) {
      out[k] = fallback[k];
    }
  }
  return out;
}
function isLiveStateEnvelope(v) {
  if (!v || typeof v !== "object")
    return false;
  const k = v.kind;
  return k === "loading" || k === "ok" || k === "error";
}
function injectCss(target, css, scope) {
  if (!css || scope === "none")
    return null;
  const doc = target.ownerDocument ?? document;
  const style = doc.createElement("style");
  style.setAttribute("data-sui-css", "1");
  style.textContent = css;
  if (scope === "document-head") {
    doc.head.appendChild(style);
  } else {
    target.parentNode?.insertBefore(style, target);
  }
  return style;
}
function extractLifecycle(ret) {
  if (!ret || typeof ret !== "object")
    return;
  const lc = ret[LIFECYCLE_KEY2];
  if (lc && typeof lc === "object")
    return lc;
  return;
}
function noop(_p) {}
function openLiveChannel(url, scope, liveKeys, root) {
  const es = new EventSource(url);
  es.addEventListener("patch-signals", (event) => {
    let payload;
    try {
      payload = JSON.parse(event.data);
    } catch {
      console.warn("[sui/mount] patch-signals: bad payload", event.data);
      return;
    }
    const { name, value } = payload;
    if (!liveKeys.has(name))
      return;
    const sig = scope[name];
    if (!sig)
      return;
    sig.set(value);
  });
  es.addEventListener("patch-elements", (event) => {
    applyPatchElementsFrame(root, event.data);
  });
  es.addEventListener("error", () => {});
  return es;
}
function applyPatchElementsFrame(root, raw) {
  let payload;
  try {
    payload = JSON.parse(raw);
  } catch {
    console.warn("[sui/mount] patch-elements: malformed payload", raw);
    return;
  }
  const { selector: selector2, mode, html } = payload;
  if (typeof selector2 !== "string" || selector2.trim() === "") {
    console.warn("[sui/mount] patch-elements: empty selector");
    return;
  }
  let target = null;
  try {
    target = root.querySelector(selector2);
  } catch (err) {
    console.warn(`[sui/mount] patch-elements: invalid selector ${selector2}:`, err);
    return;
  }
  if (!target) {
    console.warn(`[sui/mount] patch-elements: selector ${selector2} matched no element inside the view root; dropped`);
    return;
  }
  applyPatchElementsTarget(target, mode, html);
}
function applyPatchElementsTarget(target, mode, html) {
  const doc = target.ownerDocument ?? document;
  switch (mode) {
    case "replace":
      target.outerHTML = html;
      return;
    case "append": {
      const tmpl = doc.createElement("template");
      tmpl.innerHTML = html;
      target.append(...Array.from(tmpl.content.childNodes));
      return;
    }
    case "prepend": {
      const tmpl = doc.createElement("template");
      tmpl.innerHTML = html;
      target.prepend(...Array.from(tmpl.content.childNodes));
      return;
    }
    case "before": {
      const tmpl = doc.createElement("template");
      tmpl.innerHTML = html;
      target.before(...Array.from(tmpl.content.childNodes));
      return;
    }
    case "after": {
      const tmpl = doc.createElement("template");
      tmpl.innerHTML = html;
      target.after(...Array.from(tmpl.content.childNodes));
      return;
    }
    case "morph": {
      const tmpl = doc.createElement("template");
      tmpl.innerHTML = html;
      const source = tmpl.content.firstElementChild;
      if (!source) {
        console.warn("[sui/mount] patch-elements: morph with no source element");
        return;
      }
      if (source.tagName !== target.tagName) {
        target.outerHTML = html;
        return;
      }
      morph(target, source);
      return;
    }
    default:
      console.warn(`[sui/mount] patch-elements: unknown mode ${String(mode)}`);
  }
}
// packages/sui/runtime/src/chrome.ts
function getCurrentViewId() {
  if (typeof window === "undefined")
    return;
  return window.__suiCurrentViewId;
}
function emit2(name, detail) {
  if (typeof document === "undefined")
    return;
  document.dispatchEvent(new CustomEvent(name, { detail, bubbles: true }));
}
function stamp(detail, opts) {
  const viewId = opts?.viewId ?? getCurrentViewId();
  return viewId ? { ...detail, viewId } : detail;
}
var modalSeq = 0;
var pendingModals = new Map;
var modalListenerAttached = false;
function attachModalListener() {
  if (modalListenerAttached)
    return;
  if (typeof document === "undefined")
    return;
  modalListenerAttached = true;
  document.addEventListener("sui:modal:respond", (e) => {
    const detail = e.detail;
    if (typeof detail?.id !== "number")
      return;
    const resolve = pendingModals.get(detail.id);
    if (!resolve)
      return;
    pendingModals.delete(detail.id);
    resolve(detail.action ?? "");
  });
}
function normalizeTone(tone) {
  return tone === "error" ? "destructive" : tone;
}
var chrome = {
  notify(opts) {
    const normalized = normalizeTone(opts.tone);
    const detail = { ...opts };
    if (normalized !== undefined)
      detail["tone"] = normalized;
    else
      delete detail["tone"];
    emit2("sui:notify", detail);
  },
  navigate(path, opts) {
    if (typeof history !== "undefined") {
      if (opts?.replace)
        history.replaceState({}, "", path);
      else
        history.pushState({}, "", path);
    }
    emit2("sui:navigate", { to: path, ...opts?.replace ? { replace: true } : {} });
  },
  dock: {
    toggle() {
      emit2("sui:dock:toggle", {});
    },
    show() {
      emit2("sui:dock:show", {});
    },
    hide() {
      emit2("sui:dock:hide", {});
    },
    activate(tab) {
      emit2("sui:dock:activate", { tab });
    },
    clear(tab) {
      emit2("sui:dock:clear", tab ? { tab } : {});
    },
    popout(tab) {
      emit2("sui:dock:popout", tab ? { tab } : {});
    },
    close() {
      emit2("sui:dock:close", {});
    },
    resize(height) {
      emit2("sui:dock:resize", { height });
    }
  },
  bar: {
    open(mode, prefill) {
      const detail = {};
      if (mode !== undefined)
        detail["mode"] = mode;
      if (prefill !== undefined)
        detail["prefill"] = prefill;
      emit2("ship:bar:open", detail);
    },
    close() {
      emit2("ship:bar:close", {});
    }
  },
  breadcrumb: {
    set(items, opts) {
      emit2("sui:breadcrumb:set", stamp({ items }, opts));
    },
    clear(opts) {
      emit2("sui:breadcrumb:set", stamp({ items: [] }, opts));
    }
  },
  status: {
    set(key, value, opts) {
      emit2("sui:status:set", stamp({ key, value }, opts));
    },
    clear(key, opts) {
      emit2("sui:status:clear", stamp({ key }, opts));
    }
  },
  theme: {
    set(token, value, opts) {
      emit2("sui:theme:set", stamp({ token, value }, opts));
    },
    clear(token, opts) {
      emit2("sui:theme:clear", stamp({ token }, opts));
    }
  },
  focus: {
    enter(opts) {
      emit2("sui:focus:enter", stamp({}, opts));
    },
    exit(opts) {
      emit2("sui:focus:exit", stamp({}, opts));
    }
  },
  modal: {
    request(opts) {
      attachModalListener();
      const id = ++modalSeq;
      return new Promise((resolve) => {
        pendingModals.set(id, resolve);
        emit2("sui:modal:request", { id, ...opts });
      });
    }
  }
};
function notifyViewUnmount(viewId) {
  emit2("sui:view:unmount", { viewId });
}
// packages/sui/runtime/src/edit-mode.ts
var SIGNALS = new Map;
function projectId() {
  if (typeof window === "undefined")
    return "default";
  return window.__shipProjectId ?? "default";
}
var VIEW_MCP_SESSION_KEY = "ship.chrome.view-mcp.session_id";
function sessionId() {
  if (typeof window === "undefined")
    return "no-window";
  const fromWindow = window.__shipSessionId;
  if (fromWindow && fromWindow.length > 0)
    return fromWindow;
  try {
    const stored = sessionStorage.getItem(VIEW_MCP_SESSION_KEY);
    if (stored && stored.length > 0)
      return stored;
    const fresh = typeof crypto !== "undefined" && crypto.randomUUID ? crypto.randomUUID() : `runtime-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    sessionStorage.setItem(VIEW_MCP_SESSION_KEY, fresh);
    return fresh;
  } catch {
    return `runtime-fallback-${Date.now()}`;
  }
}
function signalKey(name) {
  return `${projectId()}.${name}`;
}
var migrated = false;
function purgeLegacyPersistence() {
  if (migrated)
    return;
  migrated = true;
  if (typeof localStorage === "undefined")
    return;
  try {
    const stale = [];
    for (let i = 0;i < localStorage.length; i++) {
      const k = localStorage.key(i);
      if (k && k.startsWith("ship.edit-mode."))
        stale.push(k);
    }
    for (const k of stale)
      localStorage.removeItem(k);
  } catch {}
}
async function postLifecycle(name, action) {
  if (typeof fetch === "undefined")
    return;
  const body = JSON.stringify({ session_id: sessionId() });
  try {
    await fetch(`/api/v1/views/${encodeURIComponent(name)}/edit-mode/${action}`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      credentials: "same-origin",
      body
    });
  } catch {}
}
function editModeForView(name) {
  purgeLegacyPersistence();
  const k = signalKey(name);
  const existing = SIGNALS.get(k);
  if (existing)
    return existing;
  const sig = signal(false);
  SIGNALS.set(k, sig);
  return sig;
}
function toggleEditMode(name) {
  const sig = editModeForView(name);
  const next = !sig();
  setEditMode(name, next);
  return next;
}
function setEditMode(name, on2) {
  const sig = editModeForView(name);
  if (sig() === on2)
    return;
  sig.set(on2);
  postLifecycle(name, on2 ? "enter" : "exit");
  if (typeof document !== "undefined") {
    document.dispatchEvent(new CustomEvent("sui:edit-mode:change", {
      detail: { view: name, on: on2 },
      bubbles: true
    }));
  }
}
var ANNOTATION_SIGNALS = new Map;
function annotationModeForView(name) {
  const k = `ship.annotation-mode.${projectId()}.${name}`;
  const existing = ANNOTATION_SIGNALS.get(k);
  if (existing)
    return existing;
  const sig = signal(false);
  ANNOTATION_SIGNALS.set(k, sig);
  return sig;
}
function toggleAnnotationMode(name) {
  const sig = annotationModeForView(name);
  const next = !sig();
  setAnnotationMode(name, next);
  return next;
}
function setAnnotationMode(name, on2) {
  const sig = annotationModeForView(name);
  if (sig() === on2)
    return;
  if (on2 && !editModeForView(name)())
    return;
  sig.set(on2);
  if (typeof document !== "undefined") {
    document.dispatchEvent(new CustomEvent("sui:annotation-mode:change", {
      detail: { view: name, on: on2 },
      bubbles: true
    }));
  }
}
// packages/sui/runtime/src/route/index.ts
var SHAPES = [];
var INSTANCES = new Set;
var popstateBound = false;
var updaterInstalled = false;
function defineRoutes(shapes) {
  for (const s of shapes)
    SHAPES.push(compilePattern(s));
  globalThis.__shipSetRouteUpdate?.();
}
function route(opts) {
  if (opts?.routes)
    defineRoutes(opts.routes);
  let currentUrl = opts?.url ?? readWindowUrl();
  const pathname = signal(currentUrl.pathname);
  const hash = signal(currentUrl.hash);
  const paramSigs = new Map;
  const querySigs = new Map;
  const seedParam = (key) => matchPathname(currentUrl.pathname)[key];
  const seedQuery = (key) => currentUrl.searchParams.get(key) ?? undefined;
  const params = makeProxy(paramSigs, seedParam, "params");
  const query = makeProxy(querySigs, seedQuery, "query");
  const update = () => {
    currentUrl = readWindowUrl();
    pathname.set(currentUrl.pathname);
    hash.set(currentUrl.hash);
    const matched = matchPathname(currentUrl.pathname);
    for (const [k, s] of paramSigs)
      s.set(matched[k]);
    for (const [k, s] of querySigs) {
      s.set(currentUrl.searchParams.get(k) ?? undefined);
    }
  };
  const instance = { update };
  INSTANCES.add(instance);
  ensureGlobalUpdater();
  ensurePopstateListener();
  return { params, query, pathname, hash };
}
function navigate(path, opts = {}) {
  if (typeof window === "undefined" || !window.history)
    return;
  if (opts.replace) {
    window.history.replaceState({}, "", path);
  } else {
    window.history.pushState({}, "", path);
  }
  globalThis.__shipSetRouteUpdate?.();
}
function readWindowUrl() {
  if (typeof window !== "undefined" && window.location?.href) {
    return new URL(window.location.href);
  }
  return new URL("http://localhost/");
}
function compilePattern(shape) {
  const keys = [];
  const segments = shape.pattern.split("/").map((seg) => {
    if (seg.startsWith(":")) {
      keys.push(seg.slice(1));
      return "([^/]+)";
    }
    return escapeRegExp(seg);
  });
  return {
    ...shape,
    regex: new RegExp("^" + segments.join("/") + "/?$"),
    keys
  };
}
var escapeRegExp = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
function matchPathname(pathname) {
  for (const shape of SHAPES) {
    const m = shape.regex.exec(pathname);
    if (!m)
      continue;
    const out = {};
    shape.keys.forEach((k, i) => {
      out[k] = m[i + 1];
    });
    return out;
  }
  return {};
}
function makeProxy(store, seed, label) {
  const ensure = (key) => {
    let s = store.get(key);
    if (!s) {
      s = signal(seed(key));
      store.set(key, s);
    }
    return s;
  };
  const target = Object.create(null);
  return new Proxy(target, {
    get(_t, key) {
      if (typeof key !== "string")
        return;
      if (key === "toJSON") {
        return () => {
          const out = {};
          for (const [k, s] of store)
            out[k] = s.__peek();
          return out;
        };
      }
      return ensure(key);
    },
    set(_t, key) {
      throw new Error(`[ship-set] route.${label}.${String(key)} is read-only — call navigate(path) to change the URL`);
    },
    deleteProperty(_t, key) {
      throw new Error(`[ship-set] cannot delete route.${label}.${String(key)} — call navigate(path) to change the URL`);
    },
    has(_t, key) {
      return typeof key === "string" && store.has(key);
    },
    ownKeys() {
      return [...store.keys()];
    },
    getOwnPropertyDescriptor(_t, key) {
      if (typeof key !== "string")
        return;
      const s = store.get(key);
      if (!s)
        return;
      return { configurable: true, enumerable: true, writable: false, value: s };
    }
  });
}
function ensureGlobalUpdater() {
  if (updaterInstalled)
    return;
  updaterInstalled = true;
  globalThis.__shipSetRouteUpdate = () => {
    for (const inst of INSTANCES)
      inst.update();
  };
}
function ensurePopstateListener() {
  if (popstateBound)
    return;
  if (typeof window === "undefined" || !window.addEventListener)
    return;
  popstateBound = true;
  window.addEventListener("popstate", () => {
    globalThis.__shipSetRouteUpdate?.();
  });
}

// packages/sui/runtime/src/index.ts
{
  const g = globalThis;
  g.__suiRuntimeInstances = (g.__suiRuntimeInstances ?? 0) + 1;
  if (g.__suiRuntimeInstances > 1) {
    console.warn("[sui-runtime] loaded %d times. In production this means a bundle is not " + "externalizing @ship/sui-runtime (split-brain signals); under vite " + "shell-dev it is the expected @fs-source + /_sui-bundle duplication.", g.__suiRuntimeInstances);
  }
}
export {
  withScope,
  wireFormMutation,
  view,
  useShared,
  useSelection,
  useForm,
  useFilteredList,
  unwrapOr,
  unwrap,
  untrack,
  tokenizePath,
  toggleEditMode,
  toggleAnnotationMode,
  suiTonesSheet,
  suiBaseSheet,
  signal as state,
  snapshotChildren,
  slotted,
  slotFilled,
  slide,
  signal,
  shared,
  setEditMode,
  setAnnotationMode,
  selector,
  scale,
  runProtected,
  runOut,
  runLeave,
  runIn,
  runEnter,
  route,
  resolveSlots,
  resolveController,
  registerTransition,
  registerTemplateHook,
  ref,
  provide,
  prop,
  pickShared,
  onCleanup,
  on,
  notifyViewUnmount,
  navigate,
  mount,
  morph,
  matchesKey,
  live,
  listRegisteredHooks,
  isShared,
  getTemplateHook,
  findNearestBoundary,
  fade,
  expose,
  emit,
  effect,
  editModeForView,
  detachBoundary,
  derived,
  defineRoutes,
  defineComponent,
  decodeString,
  decodeNumber,
  decodeJson,
  decodeEnum,
  decodeBoolean,
  debounce,
  data,
  currentScope,
  createScope,
  createChildScope,
  controller,
  component,
  clearRegistry,
  chrome,
  cancelTransitions,
  camelOf,
  bindWhileLoading,
  bindWhileError,
  bindValue,
  bindText,
  bindSlotEmpty,
  bindShow,
  bindRawHtml,
  bindProp,
  bindNativeEvent,
  bindIf,
  bindHtml,
  bindHostAttr,
  bindForSignal,
  bindFor,
  bindClass,
  bindChecked,
  bindAwait,
  bindAttr,
  batch,
  attachRef,
  attachHostApi,
  attachBoundary,
  applyPatches,
  applyPatch,
  annotationModeForView,
  adoptSuiSheets,
  LIFECYCLE_KEY
};
