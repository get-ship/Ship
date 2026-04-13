// ../../node_modules/.pnpm/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/css-tag.js
var t = globalThis;
var e = t.ShadowRoot && (void 0 === t.ShadyCSS || t.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype;
var s = /* @__PURE__ */ Symbol();
var o = /* @__PURE__ */ new WeakMap();
var n = class {
  constructor(t4, e6, o6) {
    if (this._$cssResult$ = true, o6 !== s) throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");
    this.cssText = t4, this.t = e6;
  }
  get styleSheet() {
    let t4 = this.o;
    const s4 = this.t;
    if (e && void 0 === t4) {
      const e6 = void 0 !== s4 && 1 === s4.length;
      e6 && (t4 = o.get(s4)), void 0 === t4 && ((this.o = t4 = new CSSStyleSheet()).replaceSync(this.cssText), e6 && o.set(s4, t4));
    }
    return t4;
  }
  toString() {
    return this.cssText;
  }
};
var r = (t4) => new n("string" == typeof t4 ? t4 : t4 + "", void 0, s);
var i = (t4, ...e6) => {
  const o6 = 1 === t4.length ? t4[0] : e6.reduce((e7, s4, o7) => e7 + ((t5) => {
    if (true === t5._$cssResult$) return t5.cssText;
    if ("number" == typeof t5) return t5;
    throw Error("Value passed to 'css' function must be a 'css' function result: " + t5 + ". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.");
  })(s4) + t4[o7 + 1], t4[0]);
  return new n(o6, t4, s);
};
var S = (s4, o6) => {
  if (e) s4.adoptedStyleSheets = o6.map((t4) => t4 instanceof CSSStyleSheet ? t4 : t4.styleSheet);
  else for (const e6 of o6) {
    const o7 = document.createElement("style"), n4 = t.litNonce;
    void 0 !== n4 && o7.setAttribute("nonce", n4), o7.textContent = e6.cssText, s4.appendChild(o7);
  }
};
var c = e ? (t4) => t4 : (t4) => t4 instanceof CSSStyleSheet ? ((t5) => {
  let e6 = "";
  for (const s4 of t5.cssRules) e6 += s4.cssText;
  return r(e6);
})(t4) : t4;

// ../../node_modules/.pnpm/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/reactive-element.js
var { is: i2, defineProperty: e2, getOwnPropertyDescriptor: h, getOwnPropertyNames: r2, getOwnPropertySymbols: o2, getPrototypeOf: n2 } = Object;
var a = globalThis;
var c2 = a.trustedTypes;
var l = c2 ? c2.emptyScript : "";
var p = a.reactiveElementPolyfillSupport;
var d = (t4, s4) => t4;
var u = { toAttribute(t4, s4) {
  switch (s4) {
    case Boolean:
      t4 = t4 ? l : null;
      break;
    case Object:
    case Array:
      t4 = null == t4 ? t4 : JSON.stringify(t4);
  }
  return t4;
}, fromAttribute(t4, s4) {
  let i6 = t4;
  switch (s4) {
    case Boolean:
      i6 = null !== t4;
      break;
    case Number:
      i6 = null === t4 ? null : Number(t4);
      break;
    case Object:
    case Array:
      try {
        i6 = JSON.parse(t4);
      } catch (t5) {
        i6 = null;
      }
  }
  return i6;
} };
var f = (t4, s4) => !i2(t4, s4);
var b = { attribute: true, type: String, converter: u, reflect: false, useDefault: false, hasChanged: f };
Symbol.metadata ?? (Symbol.metadata = /* @__PURE__ */ Symbol("metadata")), a.litPropertyMetadata ?? (a.litPropertyMetadata = /* @__PURE__ */ new WeakMap());
var y = class extends HTMLElement {
  static addInitializer(t4) {
    this._$Ei(), (this.l ?? (this.l = [])).push(t4);
  }
  static get observedAttributes() {
    return this.finalize(), this._$Eh && [...this._$Eh.keys()];
  }
  static createProperty(t4, s4 = b) {
    if (s4.state && (s4.attribute = false), this._$Ei(), this.prototype.hasOwnProperty(t4) && ((s4 = Object.create(s4)).wrapped = true), this.elementProperties.set(t4, s4), !s4.noAccessor) {
      const i6 = /* @__PURE__ */ Symbol(), h3 = this.getPropertyDescriptor(t4, i6, s4);
      void 0 !== h3 && e2(this.prototype, t4, h3);
    }
  }
  static getPropertyDescriptor(t4, s4, i6) {
    const { get: e6, set: r4 } = h(this.prototype, t4) ?? { get() {
      return this[s4];
    }, set(t5) {
      this[s4] = t5;
    } };
    return { get: e6, set(s5) {
      const h3 = e6?.call(this);
      r4?.call(this, s5), this.requestUpdate(t4, h3, i6);
    }, configurable: true, enumerable: true };
  }
  static getPropertyOptions(t4) {
    return this.elementProperties.get(t4) ?? b;
  }
  static _$Ei() {
    if (this.hasOwnProperty(d("elementProperties"))) return;
    const t4 = n2(this);
    t4.finalize(), void 0 !== t4.l && (this.l = [...t4.l]), this.elementProperties = new Map(t4.elementProperties);
  }
  static finalize() {
    if (this.hasOwnProperty(d("finalized"))) return;
    if (this.finalized = true, this._$Ei(), this.hasOwnProperty(d("properties"))) {
      const t5 = this.properties, s4 = [...r2(t5), ...o2(t5)];
      for (const i6 of s4) this.createProperty(i6, t5[i6]);
    }
    const t4 = this[Symbol.metadata];
    if (null !== t4) {
      const s4 = litPropertyMetadata.get(t4);
      if (void 0 !== s4) for (const [t5, i6] of s4) this.elementProperties.set(t5, i6);
    }
    this._$Eh = /* @__PURE__ */ new Map();
    for (const [t5, s4] of this.elementProperties) {
      const i6 = this._$Eu(t5, s4);
      void 0 !== i6 && this._$Eh.set(i6, t5);
    }
    this.elementStyles = this.finalizeStyles(this.styles);
  }
  static finalizeStyles(s4) {
    const i6 = [];
    if (Array.isArray(s4)) {
      const e6 = new Set(s4.flat(1 / 0).reverse());
      for (const s5 of e6) i6.unshift(c(s5));
    } else void 0 !== s4 && i6.push(c(s4));
    return i6;
  }
  static _$Eu(t4, s4) {
    const i6 = s4.attribute;
    return false === i6 ? void 0 : "string" == typeof i6 ? i6 : "string" == typeof t4 ? t4.toLowerCase() : void 0;
  }
  constructor() {
    super(), this._$Ep = void 0, this.isUpdatePending = false, this.hasUpdated = false, this._$Em = null, this._$Ev();
  }
  _$Ev() {
    this._$ES = new Promise((t4) => this.enableUpdating = t4), this._$AL = /* @__PURE__ */ new Map(), this._$E_(), this.requestUpdate(), this.constructor.l?.forEach((t4) => t4(this));
  }
  addController(t4) {
    (this._$EO ?? (this._$EO = /* @__PURE__ */ new Set())).add(t4), void 0 !== this.renderRoot && this.isConnected && t4.hostConnected?.();
  }
  removeController(t4) {
    this._$EO?.delete(t4);
  }
  _$E_() {
    const t4 = /* @__PURE__ */ new Map(), s4 = this.constructor.elementProperties;
    for (const i6 of s4.keys()) this.hasOwnProperty(i6) && (t4.set(i6, this[i6]), delete this[i6]);
    t4.size > 0 && (this._$Ep = t4);
  }
  createRenderRoot() {
    const t4 = this.shadowRoot ?? this.attachShadow(this.constructor.shadowRootOptions);
    return S(t4, this.constructor.elementStyles), t4;
  }
  connectedCallback() {
    this.renderRoot ?? (this.renderRoot = this.createRenderRoot()), this.enableUpdating(true), this._$EO?.forEach((t4) => t4.hostConnected?.());
  }
  enableUpdating(t4) {
  }
  disconnectedCallback() {
    this._$EO?.forEach((t4) => t4.hostDisconnected?.());
  }
  attributeChangedCallback(t4, s4, i6) {
    this._$AK(t4, i6);
  }
  _$ET(t4, s4) {
    const i6 = this.constructor.elementProperties.get(t4), e6 = this.constructor._$Eu(t4, i6);
    if (void 0 !== e6 && true === i6.reflect) {
      const h3 = (void 0 !== i6.converter?.toAttribute ? i6.converter : u).toAttribute(s4, i6.type);
      this._$Em = t4, null == h3 ? this.removeAttribute(e6) : this.setAttribute(e6, h3), this._$Em = null;
    }
  }
  _$AK(t4, s4) {
    const i6 = this.constructor, e6 = i6._$Eh.get(t4);
    if (void 0 !== e6 && this._$Em !== e6) {
      const t5 = i6.getPropertyOptions(e6), h3 = "function" == typeof t5.converter ? { fromAttribute: t5.converter } : void 0 !== t5.converter?.fromAttribute ? t5.converter : u;
      this._$Em = e6;
      const r4 = h3.fromAttribute(s4, t5.type);
      this[e6] = r4 ?? this._$Ej?.get(e6) ?? r4, this._$Em = null;
    }
  }
  requestUpdate(t4, s4, i6, e6 = false, h3) {
    if (void 0 !== t4) {
      const r4 = this.constructor;
      if (false === e6 && (h3 = this[t4]), i6 ?? (i6 = r4.getPropertyOptions(t4)), !((i6.hasChanged ?? f)(h3, s4) || i6.useDefault && i6.reflect && h3 === this._$Ej?.get(t4) && !this.hasAttribute(r4._$Eu(t4, i6)))) return;
      this.C(t4, s4, i6);
    }
    false === this.isUpdatePending && (this._$ES = this._$EP());
  }
  C(t4, s4, { useDefault: i6, reflect: e6, wrapped: h3 }, r4) {
    i6 && !(this._$Ej ?? (this._$Ej = /* @__PURE__ */ new Map())).has(t4) && (this._$Ej.set(t4, r4 ?? s4 ?? this[t4]), true !== h3 || void 0 !== r4) || (this._$AL.has(t4) || (this.hasUpdated || i6 || (s4 = void 0), this._$AL.set(t4, s4)), true === e6 && this._$Em !== t4 && (this._$Eq ?? (this._$Eq = /* @__PURE__ */ new Set())).add(t4));
  }
  async _$EP() {
    this.isUpdatePending = true;
    try {
      await this._$ES;
    } catch (t5) {
      Promise.reject(t5);
    }
    const t4 = this.scheduleUpdate();
    return null != t4 && await t4, !this.isUpdatePending;
  }
  scheduleUpdate() {
    return this.performUpdate();
  }
  performUpdate() {
    if (!this.isUpdatePending) return;
    if (!this.hasUpdated) {
      if (this.renderRoot ?? (this.renderRoot = this.createRenderRoot()), this._$Ep) {
        for (const [t6, s5] of this._$Ep) this[t6] = s5;
        this._$Ep = void 0;
      }
      const t5 = this.constructor.elementProperties;
      if (t5.size > 0) for (const [s5, i6] of t5) {
        const { wrapped: t6 } = i6, e6 = this[s5];
        true !== t6 || this._$AL.has(s5) || void 0 === e6 || this.C(s5, void 0, i6, e6);
      }
    }
    let t4 = false;
    const s4 = this._$AL;
    try {
      t4 = this.shouldUpdate(s4), t4 ? (this.willUpdate(s4), this._$EO?.forEach((t5) => t5.hostUpdate?.()), this.update(s4)) : this._$EM();
    } catch (s5) {
      throw t4 = false, this._$EM(), s5;
    }
    t4 && this._$AE(s4);
  }
  willUpdate(t4) {
  }
  _$AE(t4) {
    this._$EO?.forEach((t5) => t5.hostUpdated?.()), this.hasUpdated || (this.hasUpdated = true, this.firstUpdated(t4)), this.updated(t4);
  }
  _$EM() {
    this._$AL = /* @__PURE__ */ new Map(), this.isUpdatePending = false;
  }
  get updateComplete() {
    return this.getUpdateComplete();
  }
  getUpdateComplete() {
    return this._$ES;
  }
  shouldUpdate(t4) {
    return true;
  }
  update(t4) {
    this._$Eq && (this._$Eq = this._$Eq.forEach((t5) => this._$ET(t5, this[t5]))), this._$EM();
  }
  updated(t4) {
  }
  firstUpdated(t4) {
  }
};
y.elementStyles = [], y.shadowRootOptions = { mode: "open" }, y[d("elementProperties")] = /* @__PURE__ */ new Map(), y[d("finalized")] = /* @__PURE__ */ new Map(), p?.({ ReactiveElement: y }), (a.reactiveElementVersions ?? (a.reactiveElementVersions = [])).push("2.1.2");

// ../../node_modules/.pnpm/lit-html@3.3.2/node_modules/lit-html/lit-html.js
var t2 = globalThis;
var i3 = (t4) => t4;
var s2 = t2.trustedTypes;
var e3 = s2 ? s2.createPolicy("lit-html", { createHTML: (t4) => t4 }) : void 0;
var h2 = "$lit$";
var o3 = `lit$${Math.random().toFixed(9).slice(2)}$`;
var n3 = "?" + o3;
var r3 = `<${n3}>`;
var l2 = document;
var c3 = () => l2.createComment("");
var a2 = (t4) => null === t4 || "object" != typeof t4 && "function" != typeof t4;
var u2 = Array.isArray;
var d2 = (t4) => u2(t4) || "function" == typeof t4?.[Symbol.iterator];
var f2 = "[ 	\n\f\r]";
var v = /<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g;
var _ = /-->/g;
var m = />/g;
var p2 = RegExp(`>|${f2}(?:([^\\s"'>=/]+)(${f2}*=${f2}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`, "g");
var g = /'/g;
var $ = /"/g;
var y2 = /^(?:script|style|textarea|title)$/i;
var x = (t4) => (i6, ...s4) => ({ _$litType$: t4, strings: i6, values: s4 });
var b2 = x(1);
var w = x(2);
var T = x(3);
var E = /* @__PURE__ */ Symbol.for("lit-noChange");
var A = /* @__PURE__ */ Symbol.for("lit-nothing");
var C = /* @__PURE__ */ new WeakMap();
var P = l2.createTreeWalker(l2, 129);
function V(t4, i6) {
  if (!u2(t4) || !t4.hasOwnProperty("raw")) throw Error("invalid template strings array");
  return void 0 !== e3 ? e3.createHTML(i6) : i6;
}
var N = (t4, i6) => {
  const s4 = t4.length - 1, e6 = [];
  let n4, l3 = 2 === i6 ? "<svg>" : 3 === i6 ? "<math>" : "", c4 = v;
  for (let i7 = 0; i7 < s4; i7++) {
    const s5 = t4[i7];
    let a3, u3, d3 = -1, f3 = 0;
    for (; f3 < s5.length && (c4.lastIndex = f3, u3 = c4.exec(s5), null !== u3); ) f3 = c4.lastIndex, c4 === v ? "!--" === u3[1] ? c4 = _ : void 0 !== u3[1] ? c4 = m : void 0 !== u3[2] ? (y2.test(u3[2]) && (n4 = RegExp("</" + u3[2], "g")), c4 = p2) : void 0 !== u3[3] && (c4 = p2) : c4 === p2 ? ">" === u3[0] ? (c4 = n4 ?? v, d3 = -1) : void 0 === u3[1] ? d3 = -2 : (d3 = c4.lastIndex - u3[2].length, a3 = u3[1], c4 = void 0 === u3[3] ? p2 : '"' === u3[3] ? $ : g) : c4 === $ || c4 === g ? c4 = p2 : c4 === _ || c4 === m ? c4 = v : (c4 = p2, n4 = void 0);
    const x2 = c4 === p2 && t4[i7 + 1].startsWith("/>") ? " " : "";
    l3 += c4 === v ? s5 + r3 : d3 >= 0 ? (e6.push(a3), s5.slice(0, d3) + h2 + s5.slice(d3) + o3 + x2) : s5 + o3 + (-2 === d3 ? i7 : x2);
  }
  return [V(t4, l3 + (t4[s4] || "<?>") + (2 === i6 ? "</svg>" : 3 === i6 ? "</math>" : "")), e6];
};
var S2 = class _S {
  constructor({ strings: t4, _$litType$: i6 }, e6) {
    let r4;
    this.parts = [];
    let l3 = 0, a3 = 0;
    const u3 = t4.length - 1, d3 = this.parts, [f3, v2] = N(t4, i6);
    if (this.el = _S.createElement(f3, e6), P.currentNode = this.el.content, 2 === i6 || 3 === i6) {
      const t5 = this.el.content.firstChild;
      t5.replaceWith(...t5.childNodes);
    }
    for (; null !== (r4 = P.nextNode()) && d3.length < u3; ) {
      if (1 === r4.nodeType) {
        if (r4.hasAttributes()) for (const t5 of r4.getAttributeNames()) if (t5.endsWith(h2)) {
          const i7 = v2[a3++], s4 = r4.getAttribute(t5).split(o3), e7 = /([.?@])?(.*)/.exec(i7);
          d3.push({ type: 1, index: l3, name: e7[2], strings: s4, ctor: "." === e7[1] ? I : "?" === e7[1] ? L : "@" === e7[1] ? z : H }), r4.removeAttribute(t5);
        } else t5.startsWith(o3) && (d3.push({ type: 6, index: l3 }), r4.removeAttribute(t5));
        if (y2.test(r4.tagName)) {
          const t5 = r4.textContent.split(o3), i7 = t5.length - 1;
          if (i7 > 0) {
            r4.textContent = s2 ? s2.emptyScript : "";
            for (let s4 = 0; s4 < i7; s4++) r4.append(t5[s4], c3()), P.nextNode(), d3.push({ type: 2, index: ++l3 });
            r4.append(t5[i7], c3());
          }
        }
      } else if (8 === r4.nodeType) if (r4.data === n3) d3.push({ type: 2, index: l3 });
      else {
        let t5 = -1;
        for (; -1 !== (t5 = r4.data.indexOf(o3, t5 + 1)); ) d3.push({ type: 7, index: l3 }), t5 += o3.length - 1;
      }
      l3++;
    }
  }
  static createElement(t4, i6) {
    const s4 = l2.createElement("template");
    return s4.innerHTML = t4, s4;
  }
};
function M(t4, i6, s4 = t4, e6) {
  if (i6 === E) return i6;
  let h3 = void 0 !== e6 ? s4._$Co?.[e6] : s4._$Cl;
  const o6 = a2(i6) ? void 0 : i6._$litDirective$;
  return h3?.constructor !== o6 && (h3?._$AO?.(false), void 0 === o6 ? h3 = void 0 : (h3 = new o6(t4), h3._$AT(t4, s4, e6)), void 0 !== e6 ? (s4._$Co ?? (s4._$Co = []))[e6] = h3 : s4._$Cl = h3), void 0 !== h3 && (i6 = M(t4, h3._$AS(t4, i6.values), h3, e6)), i6;
}
var R = class {
  constructor(t4, i6) {
    this._$AV = [], this._$AN = void 0, this._$AD = t4, this._$AM = i6;
  }
  get parentNode() {
    return this._$AM.parentNode;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  u(t4) {
    const { el: { content: i6 }, parts: s4 } = this._$AD, e6 = (t4?.creationScope ?? l2).importNode(i6, true);
    P.currentNode = e6;
    let h3 = P.nextNode(), o6 = 0, n4 = 0, r4 = s4[0];
    for (; void 0 !== r4; ) {
      if (o6 === r4.index) {
        let i7;
        2 === r4.type ? i7 = new k(h3, h3.nextSibling, this, t4) : 1 === r4.type ? i7 = new r4.ctor(h3, r4.name, r4.strings, this, t4) : 6 === r4.type && (i7 = new Z(h3, this, t4)), this._$AV.push(i7), r4 = s4[++n4];
      }
      o6 !== r4?.index && (h3 = P.nextNode(), o6++);
    }
    return P.currentNode = l2, e6;
  }
  p(t4) {
    let i6 = 0;
    for (const s4 of this._$AV) void 0 !== s4 && (void 0 !== s4.strings ? (s4._$AI(t4, s4, i6), i6 += s4.strings.length - 2) : s4._$AI(t4[i6])), i6++;
  }
};
var k = class _k {
  get _$AU() {
    return this._$AM?._$AU ?? this._$Cv;
  }
  constructor(t4, i6, s4, e6) {
    this.type = 2, this._$AH = A, this._$AN = void 0, this._$AA = t4, this._$AB = i6, this._$AM = s4, this.options = e6, this._$Cv = e6?.isConnected ?? true;
  }
  get parentNode() {
    let t4 = this._$AA.parentNode;
    const i6 = this._$AM;
    return void 0 !== i6 && 11 === t4?.nodeType && (t4 = i6.parentNode), t4;
  }
  get startNode() {
    return this._$AA;
  }
  get endNode() {
    return this._$AB;
  }
  _$AI(t4, i6 = this) {
    t4 = M(this, t4, i6), a2(t4) ? t4 === A || null == t4 || "" === t4 ? (this._$AH !== A && this._$AR(), this._$AH = A) : t4 !== this._$AH && t4 !== E && this._(t4) : void 0 !== t4._$litType$ ? this.$(t4) : void 0 !== t4.nodeType ? this.T(t4) : d2(t4) ? this.k(t4) : this._(t4);
  }
  O(t4) {
    return this._$AA.parentNode.insertBefore(t4, this._$AB);
  }
  T(t4) {
    this._$AH !== t4 && (this._$AR(), this._$AH = this.O(t4));
  }
  _(t4) {
    this._$AH !== A && a2(this._$AH) ? this._$AA.nextSibling.data = t4 : this.T(l2.createTextNode(t4)), this._$AH = t4;
  }
  $(t4) {
    const { values: i6, _$litType$: s4 } = t4, e6 = "number" == typeof s4 ? this._$AC(t4) : (void 0 === s4.el && (s4.el = S2.createElement(V(s4.h, s4.h[0]), this.options)), s4);
    if (this._$AH?._$AD === e6) this._$AH.p(i6);
    else {
      const t5 = new R(e6, this), s5 = t5.u(this.options);
      t5.p(i6), this.T(s5), this._$AH = t5;
    }
  }
  _$AC(t4) {
    let i6 = C.get(t4.strings);
    return void 0 === i6 && C.set(t4.strings, i6 = new S2(t4)), i6;
  }
  k(t4) {
    u2(this._$AH) || (this._$AH = [], this._$AR());
    const i6 = this._$AH;
    let s4, e6 = 0;
    for (const h3 of t4) e6 === i6.length ? i6.push(s4 = new _k(this.O(c3()), this.O(c3()), this, this.options)) : s4 = i6[e6], s4._$AI(h3), e6++;
    e6 < i6.length && (this._$AR(s4 && s4._$AB.nextSibling, e6), i6.length = e6);
  }
  _$AR(t4 = this._$AA.nextSibling, s4) {
    for (this._$AP?.(false, true, s4); t4 !== this._$AB; ) {
      const s5 = i3(t4).nextSibling;
      i3(t4).remove(), t4 = s5;
    }
  }
  setConnected(t4) {
    void 0 === this._$AM && (this._$Cv = t4, this._$AP?.(t4));
  }
};
var H = class {
  get tagName() {
    return this.element.tagName;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  constructor(t4, i6, s4, e6, h3) {
    this.type = 1, this._$AH = A, this._$AN = void 0, this.element = t4, this.name = i6, this._$AM = e6, this.options = h3, s4.length > 2 || "" !== s4[0] || "" !== s4[1] ? (this._$AH = Array(s4.length - 1).fill(new String()), this.strings = s4) : this._$AH = A;
  }
  _$AI(t4, i6 = this, s4, e6) {
    const h3 = this.strings;
    let o6 = false;
    if (void 0 === h3) t4 = M(this, t4, i6, 0), o6 = !a2(t4) || t4 !== this._$AH && t4 !== E, o6 && (this._$AH = t4);
    else {
      const e7 = t4;
      let n4, r4;
      for (t4 = h3[0], n4 = 0; n4 < h3.length - 1; n4++) r4 = M(this, e7[s4 + n4], i6, n4), r4 === E && (r4 = this._$AH[n4]), o6 || (o6 = !a2(r4) || r4 !== this._$AH[n4]), r4 === A ? t4 = A : t4 !== A && (t4 += (r4 ?? "") + h3[n4 + 1]), this._$AH[n4] = r4;
    }
    o6 && !e6 && this.j(t4);
  }
  j(t4) {
    t4 === A ? this.element.removeAttribute(this.name) : this.element.setAttribute(this.name, t4 ?? "");
  }
};
var I = class extends H {
  constructor() {
    super(...arguments), this.type = 3;
  }
  j(t4) {
    this.element[this.name] = t4 === A ? void 0 : t4;
  }
};
var L = class extends H {
  constructor() {
    super(...arguments), this.type = 4;
  }
  j(t4) {
    this.element.toggleAttribute(this.name, !!t4 && t4 !== A);
  }
};
var z = class extends H {
  constructor(t4, i6, s4, e6, h3) {
    super(t4, i6, s4, e6, h3), this.type = 5;
  }
  _$AI(t4, i6 = this) {
    if ((t4 = M(this, t4, i6, 0) ?? A) === E) return;
    const s4 = this._$AH, e6 = t4 === A && s4 !== A || t4.capture !== s4.capture || t4.once !== s4.once || t4.passive !== s4.passive, h3 = t4 !== A && (s4 === A || e6);
    e6 && this.element.removeEventListener(this.name, this, s4), h3 && this.element.addEventListener(this.name, this, t4), this._$AH = t4;
  }
  handleEvent(t4) {
    "function" == typeof this._$AH ? this._$AH.call(this.options?.host ?? this.element, t4) : this._$AH.handleEvent(t4);
  }
};
var Z = class {
  constructor(t4, i6, s4) {
    this.element = t4, this.type = 6, this._$AN = void 0, this._$AM = i6, this.options = s4;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AI(t4) {
    M(this, t4);
  }
};
var B = t2.litHtmlPolyfillSupport;
B?.(S2, k), (t2.litHtmlVersions ?? (t2.litHtmlVersions = [])).push("3.3.2");
var D = (t4, i6, s4) => {
  const e6 = s4?.renderBefore ?? i6;
  let h3 = e6._$litPart$;
  if (void 0 === h3) {
    const t5 = s4?.renderBefore ?? null;
    e6._$litPart$ = h3 = new k(i6.insertBefore(c3(), t5), t5, void 0, s4 ?? {});
  }
  return h3._$AI(t4), h3;
};

// ../../node_modules/.pnpm/lit-element@4.2.2/node_modules/lit-element/lit-element.js
var s3 = globalThis;
var i4 = class extends y {
  constructor() {
    super(...arguments), this.renderOptions = { host: this }, this._$Do = void 0;
  }
  createRenderRoot() {
    var _a;
    const t4 = super.createRenderRoot();
    return (_a = this.renderOptions).renderBefore ?? (_a.renderBefore = t4.firstChild), t4;
  }
  update(t4) {
    const r4 = this.render();
    this.hasUpdated || (this.renderOptions.isConnected = this.isConnected), super.update(t4), this._$Do = D(r4, this.renderRoot, this.renderOptions);
  }
  connectedCallback() {
    super.connectedCallback(), this._$Do?.setConnected(true);
  }
  disconnectedCallback() {
    super.disconnectedCallback(), this._$Do?.setConnected(false);
  }
  render() {
    return E;
  }
};
i4._$litElement$ = true, i4["finalized"] = true, s3.litElementHydrateSupport?.({ LitElement: i4 });
var o4 = s3.litElementPolyfillSupport;
o4?.({ LitElement: i4 });
(s3.litElementVersions ?? (s3.litElementVersions = [])).push("4.2.2");

// src/web-components/util.ts
var MONO_STACK = "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace";
var MONO = r(MONO_STACK);
var toneVars = i`
  :host {
    --tone-fg: var(--foreground);
    --tone-bg: var(--muted);
    --tone-border: var(--border);
  }
  :host([tone='info']) {
    --tone-fg: var(--info);
    --tone-bg: color-mix(in oklch, var(--info) 16%, transparent);
    --tone-border: color-mix(in oklch, var(--info) 35%, transparent);
  }
  :host([tone='success']) {
    --tone-fg: var(--success);
    --tone-bg: color-mix(in oklch, var(--success) 16%, transparent);
    --tone-border: color-mix(in oklch, var(--success) 35%, transparent);
  }
  :host([tone='warning']) {
    --tone-fg: var(--warning);
    --tone-bg: color-mix(in oklch, var(--warning) 18%, transparent);
    --tone-border: color-mix(in oklch, var(--warning) 40%, transparent);
  }
  :host([tone='destructive']) {
    --tone-fg: var(--destructive);
    --tone-bg: color-mix(in oklch, var(--destructive) 16%, transparent);
    --tone-border: color-mix(in oklch, var(--destructive) 35%, transparent);
  }
  :host([tone='accent']) {
    --tone-fg: var(--accent);
    --tone-bg: color-mix(in oklch, var(--accent) 16%, transparent);
    --tone-border: color-mix(in oklch, var(--accent) 35%, transparent);
  }
  :host([tone='muted']) {
    --tone-fg: var(--muted-foreground);
    --tone-bg: var(--muted);
    --tone-border: var(--border);
  }
`;

// src/web-components/ship-alert.ts
var ShipAlert = class extends i4 {
  constructor() {
    super();
    this.tone = "info";
    this.dismissible = false;
  }
  onDismiss() {
    this.dispatchEvent(new CustomEvent("ship-dismiss", {
      bubbles: true,
      composed: true
    }));
    this.remove();
  }
  render() {
    return b2`
      <div class="box" role="status">
        <span><slot name="icon"></slot></span>
        <div class="content">
          <div class="title"><slot name="title"></slot></div>
          <div class="body"><slot></slot></div>
          <div class="actions"><slot name="actions"></slot></div>
        </div>
        ${this.dismissible ? b2`<button class="close" aria-label="Dismiss" @click=${this.onDismiss}>×</button>` : b2`<span></span>`}
      </div>
    `;
  }
};
ShipAlert.styles = [
  toneVars,
  i`
      :host { display: block; }
      .box {
        display: grid;
        grid-template-columns: auto 1fr auto;
        align-items: start;
        gap: 10px;
        padding: 12px 14px;
        border-radius: var(--radius);
        border: 1px solid var(--tone-border);
        background: var(--tone-bg);
        color: var(--tone-fg);
        font-family: var(--font-body);
        font-size: 13px;
        line-height: 1.5;
      }
      .content { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
      .title { font-weight: 600; }
      .body { color: var(--foreground); }
      .close {
        background: transparent; border: 0; color: inherit;
        cursor: pointer; font-size: 14px; padding: 2px 6px;
        border-radius: 4px;
      }
      .close:hover { background: color-mix(in oklch, var(--tone-fg) 12%, transparent); }
      .actions { display: flex; gap: 6px; margin-top: 6px; }
      ::slotted([slot='icon']) { display: inline-flex; }
    `
];
ShipAlert.properties = {
  tone: { type: String, reflect: true },
  dismissible: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-alert")) {
  customElements.define("ship-alert", ShipAlert);
}

// src/web-components/ship-dialog.ts
var ShipDialog = class extends i4 {
  constructor() {
    super(...arguments);
    this.open = false;
    this.size = "md";
    this.dismissible = true;
    this.noCloseButton = false;
    this.onClose = () => {
      this.open = false;
      this.dispatchEvent(new CustomEvent("ship-close", { bubbles: true, composed: true }));
    };
    this.onBackdropClick = (e6) => {
      if (!this.dismissible) return;
      if (e6.target === this.dialogEl) this.close();
    };
    this.onDismiss = () => {
      this.dispatchEvent(new CustomEvent("ship-dismiss", { bubbles: true, composed: true }));
      this.close();
    };
  }
  get dialogEl() {
    return this.renderRoot.querySelector("dialog");
  }
  updated(changed) {
    if (changed.has("open") && this.dialogEl) {
      if (this.open && !this.dialogEl.open) {
        if (typeof this.dialogEl.showModal === "function") {
          try {
            this.dialogEl.showModal();
          } catch {
          }
        }
        this.dispatchEvent(new CustomEvent("ship-open", { bubbles: true, composed: true }));
      } else if (!this.open && this.dialogEl.open) {
        this.dialogEl.close();
      }
    }
  }
  show() {
    this.open = true;
  }
  close() {
    this.open = false;
  }
  render() {
    return b2`
      <dialog @close=${this.onClose} @click=${this.onBackdropClick}>
        <div class="wrap">
          <button
            class="close"
            type="button"
            aria-label="Close dialog"
            @click=${this.onDismiss}
          >×</button>
          <div class="header"><slot name="header"></slot></div>
          <div class="body"><slot></slot></div>
          <div class="footer"><slot name="footer"></slot></div>
        </div>
      </dialog>
    `;
  }
};
ShipDialog.styles = i`
    :host { display: contents; }
    dialog {
      padding: 0;
      border: 1px solid var(--border);
      border-radius: var(--radius-lg);
      background: var(--card);
      color: var(--card-foreground);
      max-width: 560px;
      width: 100%;
      max-height: calc(100vh - 64px);
      box-shadow: var(--shadow-xl);
      font-family: var(--font-body);
      opacity: 0;
      transform: translateY(4px);
      transition:
        opacity var(--duration-medium) var(--ease-out),
        transform var(--duration-medium) var(--ease-out);
    }
    dialog[open] {
      opacity: 1;
      transform: translateY(0);
    }
    dialog::backdrop {
      background: oklch(0 0 0 / 40%);
      backdrop-filter: blur(2px);
      -webkit-backdrop-filter: blur(2px);
    }
    :host([size='sm']) dialog { max-width: 400px; }
    :host([size='lg']) dialog { max-width: 800px; }

    .wrap { display: flex; flex-direction: column; max-height: inherit; position: relative; }
    .header {
      padding: 14px 20px;
      padding-right: 44px;
      border-bottom: 1px solid var(--border);
      font-size: var(--text-md);
      font-weight: var(--font-weight-semibold);
    }
    .body {
      padding: 16px 20px;
      overflow-y: auto;
      font-size: var(--text-base);
      line-height: var(--leading-normal);
    }
    .footer {
      padding: 12px 20px;
      border-top: 1px solid var(--border);
      display: flex; gap: 8px; justify-content: flex-end;
    }
    .close {
      position: absolute;
      top: 10px; right: 10px;
      width: 28px; height: 28px;
      display: inline-flex; align-items: center; justify-content: center;
      background: transparent;
      border: 1px solid transparent;
      border-radius: var(--radius-md);
      color: var(--muted-foreground);
      cursor: pointer;
      font: inherit;
      font-size: 18px; line-height: 1;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out);
    }
    .close:hover { background: var(--muted); color: var(--foreground); }
    .close:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--card),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    :host([no-close-button]) .close { display: none; }
  `;
ShipDialog.properties = {
  open: { type: Boolean, reflect: true },
  size: { type: String, reflect: true },
  dismissible: { type: Boolean },
  noCloseButton: { type: Boolean, reflect: true, attribute: "no-close-button" }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-dialog")) {
  customElements.define("ship-dialog", ShipDialog);
}

// src/web-components/ship-button.ts
var ShipButton = class extends i4 {
  constructor() {
    super(...arguments);
    this.variant = "solid";
    this.size = "md";
    this.disabled = false;
    this.loading = false;
    this.type = "button";
  }
  render() {
    return b2`
      <button
        part="button"
        type=${this.type}
        ?disabled=${this.disabled || this.loading}
        aria-busy=${this.loading ? "true" : "false"}
      >
        ${this.loading ? b2`<span class="spinner" aria-hidden="true"></span>` : b2`<slot name="leading"></slot>`}
        <slot></slot>
        <slot name="trailing"></slot>
      </button>
    `;
  }
};
ShipButton.styles = i`
    :host {
      display: inline-flex;
      vertical-align: middle;
    }
    :host([hidden]) { display: none; }

    button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: var(--radius-sm);
      font-family: var(--font-body);
      font-weight: var(--font-weight-medium);
      line-height: 1;
      border-radius: var(--radius-md);
      cursor: pointer;
      border: 1px solid transparent;
      background: transparent;
      color: var(--foreground);
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out),
        filter var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out);
      white-space: nowrap;
    }
    :host([loading]) button { min-width: var(--ship-button-min-width, 2.5em); }
    button:hover:not(:disabled) { box-shadow: var(--shadow-sm); }
    button:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    button:disabled { cursor: not-allowed; opacity: 0.6; }

    /* sizes — match specs .btn (sm) and a slightly larger md/lg for Studio use */
    :host([size='sm']) button { font-size: 11px; padding: 4px 8px; }
    :host(:not([size])) button,
    :host([size='md']) button { font-size: var(--text-sm); padding: 5px 10px; }
    :host([size='lg']) button { font-size: var(--text-base); padding: 8px 14px; }

    /* default (tone=default): neutral bordered button matching specs .btn */
    :host(:not([tone])) button,
    :host([tone='default']) button {
      background: var(--background);
      border-color: var(--border);
      color: var(--foreground);
    }
    :host(:not([tone])) button:hover:not(:disabled),
    :host([tone='default']) button:hover:not(:disabled) {
      background: var(--muted);
    }

    /* solid variant — primary colored fill. Matches specs .btn-primary. */
    :host([variant='solid'][tone='default']) button,
    :host([variant='solid']:not([tone])) button {
      background: var(--primary);
      border-color: var(--primary);
      color: var(--primary-foreground);
    }
    :host([variant='solid'][tone='default']) button:hover:not(:disabled),
    :host([variant='solid']:not([tone])) button:hover:not(:disabled) {
      filter: brightness(1.05);
    }
    :host([variant='solid'][tone='destructive']) button {
      background: var(--destructive);
      border-color: var(--destructive);
      color: oklch(0.99 0.02 95);
    }
    :host([variant='solid'][tone='destructive']) button:hover:not(:disabled) {
      filter: brightness(1.05);
    }
    :host([variant='solid'][tone='success']) button {
      background: var(--success); border-color: var(--success); color: oklch(0.99 0.02 95);
    }
    :host([variant='solid'][tone='accent']) button {
      background: var(--accent); border-color: var(--accent); color: var(--accent-foreground);
    }

    /* outline variant — matches specs .btn-danger */
    :host([variant='outline']) button {
      background: transparent;
      border-color: var(--border);
      color: var(--foreground);
    }
    :host([variant='outline'][tone='destructive']) button {
      color: var(--destructive); border-color: var(--destructive);
    }
    :host([variant='outline']) button:hover:not(:disabled) { background: var(--muted); }

    /* ghost variant — transparent, hover tint */
    :host([variant='ghost']) button {
      background: transparent; border-color: transparent; color: var(--muted-foreground);
    }
    :host([variant='ghost']) button:hover:not(:disabled) {
      background: var(--muted); color: var(--foreground);
    }
    :host([variant='ghost'][tone='destructive']) button { color: var(--destructive); }

    /* soft variant — tinted surface, used for pill-like affirm buttons */
    :host([variant='soft']) button {
      background: color-mix(in oklch, var(--primary) 12%, transparent);
      border-color: color-mix(in oklch, var(--primary) 25%, transparent);
      color: var(--primary);
    }
    :host([variant='soft'][tone='destructive']) button {
      background: color-mix(in oklch, var(--destructive) 12%, transparent);
      border-color: color-mix(in oklch, var(--destructive) 28%, transparent);
      color: var(--destructive);
    }

    .spinner {
      width: 12px; height: 12px;
      border: 1.5px solid currentColor;
      border-right-color: transparent;
      border-radius: 50%;
      animation: spin 0.8s linear infinite;
      display: inline-block;
    }
    @keyframes spin { to { transform: rotate(360deg); } }

    ::slotted([slot='leading']), ::slotted([slot='trailing']) {
      display: inline-flex; align-items: center;
    }
  `;
ShipButton.properties = {
  tone: { type: String, reflect: true },
  variant: { type: String, reflect: true },
  size: { type: String, reflect: true },
  disabled: { type: Boolean, reflect: true },
  loading: { type: Boolean, reflect: true },
  type: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-button")) {
  customElements.define("ship-button", ShipButton);
}

// src/web-components/ship-alert-dialog.ts
var ShipAlertDialog = class extends i4 {
  constructor() {
    super(...arguments);
    this.open = false;
    this.tone = "default";
    this.confirmLabel = "Confirm";
    this.cancelLabel = "Cancel";
  }
  emit(name) {
    this.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }));
    this.open = false;
  }
  render() {
    return b2`
      <ship-dialog size="sm" no-close-button ?open=${this.open}
        @ship-close=${() => {
      this.open = false;
    }}
      >
        <span slot="header"><slot name="title"></slot></span>
        <slot></slot>
        <span slot="footer">
          <ship-button variant="outline" @click=${() => this.emit("ship-cancel")}>
            ${this.cancelLabel}
          </ship-button>
          <ship-button
            variant="solid"
            tone=${this.tone === "destructive" ? "destructive" : "default"}
            @click=${() => this.emit("ship-confirm")}
          >
            ${this.confirmLabel}
          </ship-button>
        </span>
      </ship-dialog>
    `;
  }
};
ShipAlertDialog.styles = i`
    :host { display: contents; }
  `;
ShipAlertDialog.properties = {
  open: { type: Boolean, reflect: true },
  tone: { type: String, reflect: true },
  confirmLabel: { type: String, attribute: "confirm-label" },
  cancelLabel: { type: String, attribute: "cancel-label" }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-alert-dialog")) {
  customElements.define("ship-alert-dialog", ShipAlertDialog);
}

// src/web-components/ship-badge.ts
var ShipBadge = class extends i4 {
  constructor() {
    super();
    this.tone = "muted";
    this.size = "md";
  }
  render() {
    return b2`<span class="badge" part="badge"><slot></slot></span>`;
  }
};
ShipBadge.styles = [
  toneVars,
  i`
      :host {
        display: inline-flex;
        vertical-align: middle;
      }
      .badge {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        padding: 2px 8px;
        font-family: var(--font-body);
        font-size: 10px;
        font-weight: 600;
        line-height: 1.4;
        letter-spacing: 0.04em;
        text-transform: uppercase;
        border-radius: 4px;
        background: var(--tone-bg);
        color: var(--tone-fg);
        border: 1px solid transparent;
        white-space: nowrap;
      }
      :host([size='sm']) .badge {
        padding: 1px 6px;
        font-size: 9px;
      }
    `
];
ShipBadge.properties = {
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-badge")) {
  customElements.define("ship-badge", ShipBadge);
}

// src/web-components/ship-card.ts
var ShipCard = class extends i4 {
  constructor() {
    super();
    this.variant = "default";
    this.interactive = false;
    this.accentBar = false;
    this.statusRail = false;
    this.fill = false;
  }
  render() {
    return b2`
      <div class="header"><slot name="header"></slot></div>
      <slot></slot>
      <div class="footer"><slot name="footer"></slot></div>
    `;
  }
};
ShipCard.styles = [
  toneVars,
  i`
      :host {
        display: block;
        background: var(--card);
        color: var(--card-foreground);
        border: 1px solid var(--border);
        border-radius: var(--radius);
        padding: 16px 20px;
        position: relative;
        transition:
          border-color var(--duration-medium) var(--ease-out),
          transform var(--duration-fast) var(--ease-out),
          box-shadow var(--duration-fast) var(--ease-out);
      }
      :host([interactive]) { cursor: pointer; }
      :host([interactive]:hover) {
        border-color: color-mix(in oklch, var(--border) 180%, transparent);
        transform: translateY(-1px);
        box-shadow: var(--shadow-md);
      }
      :host([accent-bar])::before {
        content: '';
        position: absolute; top: 0; left: 0; right: 0; height: 2px;
        background: linear-gradient(90deg, var(--primary), var(--accent));
        opacity: 0;
        transition: opacity var(--duration-medium) var(--ease-out);
        border-radius: var(--radius) var(--radius) 0 0;
      }
      :host([accent-bar]:hover)::before { opacity: 1; }

      /* status-rail — 3px left border tinted by tone */
      :host([status-rail]) {
        border-left: 3px solid var(--tone-fg);
        padding-left: 17px;
      }

      /* fill — tone-tinted background wash. Pairs well with status-rail. */
      :host([fill]) { background: var(--tone-bg); }

      :host([variant='outline']) { background: transparent; }

      .header { margin-bottom: 10px; }
      .footer { margin-top: 10px; }
      ::slotted([slot='header']) { display: block; }
    `
];
ShipCard.properties = {
  tone: { type: String, reflect: true },
  variant: { type: String, reflect: true },
  interactive: { type: Boolean, reflect: true },
  accentBar: { type: Boolean, reflect: true, attribute: "accent-bar" },
  statusRail: { type: Boolean, reflect: true, attribute: "status-rail" },
  fill: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-card")) {
  customElements.define("ship-card", ShipCard);
}

// src/web-components/ship-checkbox.ts
var ShipCheckbox = class extends i4 {
  constructor() {
    super(...arguments);
    this.checked = false;
    this.indeterminate = false;
    this.name = "";
    this.value = "on";
    this.disabled = false;
    this.required = false;
  }
  get input() {
    return this.renderRoot.querySelector("input");
  }
  updated() {
    if (this.input) this.input.indeterminate = this.indeterminate;
  }
  onChange(e6) {
    const c4 = e6.target.checked;
    this.checked = c4;
    this.indeterminate = false;
    this.dispatchEvent(new CustomEvent("ship-change", {
      detail: { checked: c4 },
      bubbles: true,
      composed: true
    }));
  }
  render() {
    return b2`
      <label part="label">
        <input
          part="input"
          type="checkbox"
          .checked=${this.checked}
          name=${this.name}
          value=${this.value}
          ?disabled=${this.disabled}
          ?required=${this.required}
          @change=${this.onChange}
        />
        <slot></slot>
      </label>
    `;
  }
};
ShipCheckbox.styles = i`
    :host { display: inline-flex; align-items: center; gap: 8px; cursor: pointer; font: inherit; font-size: 13px; color: var(--foreground); }
    :host([disabled]) { cursor: not-allowed; opacity: 0.6; }
    label { display: inline-flex; align-items: center; gap: 8px; cursor: inherit; }
    input { margin: 0; accent-color: var(--primary); width: 14px; height: 14px; }
    input:focus-visible { outline: 2px solid var(--primary); outline-offset: 2px; }
  `;
ShipCheckbox.properties = {
  checked: { type: Boolean, reflect: true },
  indeterminate: { type: Boolean },
  name: { type: String },
  value: { type: String },
  disabled: { type: Boolean, reflect: true },
  required: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-checkbox")) {
  customElements.define("ship-checkbox", ShipCheckbox);
}

// src/web-components/ship-dropdown-menu.ts
var ShipDropdownMenu = class extends i4 {
  constructor() {
    super(...arguments);
    this.open = false;
    this.onTriggerClick = () => {
      this.open = !this.open;
    };
    this.onItemClick = (e6) => {
      const target = e6.target.closest("[data-value]");
      if (!target) return;
      const value = target.dataset.value || "";
      this.dispatchEvent(new CustomEvent("ship-select", {
        detail: { value },
        bubbles: true,
        composed: true
      }));
      this.open = false;
    };
    this.onDocClick = (e6) => {
      if (!this.open) return;
      const target = e6.composedPath()[0];
      if (!this.contains(target) && target !== this) this.open = false;
    };
  }
  connectedCallback() {
    super.connectedCallback();
    document.addEventListener("click", this.onDocClick);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    document.removeEventListener("click", this.onDocClick);
  }
  render() {
    return b2`
      <span @click=${this.onTriggerClick}><slot name="trigger"></slot></span>
      <div class="menu" role="menu" @click=${this.onItemClick}>
        <slot></slot>
      </div>
    `;
  }
};
ShipDropdownMenu.styles = i`
    :host { position: relative; display: inline-flex; }
    .menu {
      position: absolute;
      z-index: var(--z-dropdown);
      top: calc(100% + 4px);
      left: 0;
      min-width: 160px;
      padding: 4px;
      background: var(--popover);
      color: var(--popover-foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      box-shadow: var(--shadow-lg);
      display: flex; flex-direction: column;
      font-family: var(--font-body);
      font-size: var(--text-base);
      opacity: 0;
      transform: translateY(-4px);
      pointer-events: none;
      transition:
        opacity var(--duration-medium) var(--ease-out),
        transform var(--duration-medium) var(--ease-out);
    }
    :host([open]) .menu {
      opacity: 1;
      transform: translateY(0);
      pointer-events: auto;
    }
    ::slotted(button), ::slotted([role='menuitem']) {
      text-align: left;
      background: transparent; border: 0;
      padding: 6px 10px;
      border-radius: var(--radius-sm);
      color: inherit; font: inherit;
      cursor: pointer;
      transition: background-color var(--duration-fast) var(--ease-out);
    }
    ::slotted(button:hover), ::slotted([role='menuitem']:hover) {
      background: var(--muted);
    }
  `;
ShipDropdownMenu.properties = {
  open: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-dropdown-menu")) {
  customElements.define("ship-dropdown-menu", ShipDropdownMenu);
}

// src/web-components/ship-empty-state.ts
var ShipEmptyState = class extends i4 {
  constructor() {
    super(...arguments);
    this.variant = "default";
  }
  render() {
    return b2`
      <div class="box">
        <div class="icon"><slot name="icon"></slot></div>
        <div class="title"><slot name="title"></slot></div>
        <div class="desc"><slot></slot></div>
        <div class="actions"><slot name="action"></slot></div>
      </div>
    `;
  }
};
ShipEmptyState.styles = i`
    :host { display: block; }
    .box {
      display: flex; flex-direction: column;
      align-items: center; justify-content: center;
      gap: 10px;
      padding: 64px 16px;
      text-align: center;
      font-family: var(--font-body);
      color: var(--muted-foreground);
      background: var(--card);
      border: 1px dashed var(--border);
      border-radius: var(--radius);
    }
    :host([variant='connecting']) .box {
      padding: 48px 0;
      background: transparent;
      border: 0;
      font-size: 13px;
    }
    :host([variant='placeholder']) .box {
      padding: 24px 16px;
      background: transparent;
      border: 0;
      font-size: 13px;
    }
    .icon {
      font-size: 36px; opacity: 0.35;
      display: inline-flex; align-items: center; justify-content: center;
    }
    .title {
      font-size: 14px; font-weight: 600;
      color: var(--foreground);
    }
    .desc {
      font-size: 12px;
      max-width: 420px;
      margin: 0 auto;
      line-height: 1.55;
    }
    .actions {
      display: flex; gap: 8px; justify-content: center;
      margin-top: 4px;
    }
    ::slotted([slot='icon']) { display: inline-flex; }
  `;
ShipEmptyState.properties = {
  variant: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-empty-state")) {
  customElements.define("ship-empty-state", ShipEmptyState);
}

// src/web-components/ship-field.ts
var ShipField = class extends i4 {
  constructor() {
    super(...arguments);
    this.invalid = false;
  }
  render() {
    return b2`
      <slot name="label"></slot>
      <slot></slot>
      <div class="hint"><slot name="hint"></slot></div>
      <div class="error"><slot name="error"></slot></div>
    `;
  }
};
ShipField.styles = i`
    :host { display: flex; flex-direction: column; gap: 6px; }
    .hint, .error { font-size: 11px; line-height: 1.4; }
    .hint { color: var(--muted-foreground); }
    .error { color: var(--destructive); }
    :host(:not([invalid])) .error { display: none; }
    :host([invalid]) .hint { display: none; }
  `;
ShipField.properties = {
  invalid: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-field")) {
  customElements.define("ship-field", ShipField);
}

// src/web-components/ship-input.ts
var ShipInput = class extends i4 {
  constructor() {
    super(...arguments);
    this.type = "text";
    this.value = "";
    this.placeholder = "";
    this.name = "";
    this.autocomplete = "";
    this.required = false;
    this.disabled = false;
    this.readonly = false;
    this.invalid = false;
    this.loading = false;
    this.variant = "default";
    this.size = "md";
  }
  onInput(e6) {
    const target = e6.target;
    this.value = target.value;
    this.dispatchEvent(new CustomEvent("ship-input", {
      detail: { value: this.value },
      bubbles: true,
      composed: true
    }));
  }
  render() {
    return b2`
      <div class="field" part="field">
        <slot name="leading"></slot>
        <input
          part="input"
          .value=${this.value}
          type=${this.type}
          placeholder=${this.placeholder}
          name=${this.name}
          autocomplete=${this.autocomplete || "off"}
          ?required=${this.required}
          ?disabled=${this.disabled}
          ?readonly=${this.readonly}
          aria-invalid=${this.invalid ? "true" : "false"}
          @input=${this.onInput}
        />
        ${this.loading ? b2`<span class="spinner" aria-hidden="true"></span>` : b2`<slot name="trailing"></slot>`}
      </div>
    `;
  }
};
ShipInput.styles = i`
    :host { display: inline-flex; width: 100%; }
    .field {
      display: inline-flex; align-items: center; gap: var(--radius-sm);
      width: 100%;
      background: var(--background);
      color: var(--foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      padding: 0 10px;
      transition:
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
    }
    .field:focus-within {
      outline: none;
      border-color: var(--primary);
      box-shadow:
        0 0 0 var(--ring-width)
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    :host([invalid]) .field:focus-within {
      border-color: var(--destructive);
      box-shadow:
        0 0 0 var(--ring-width)
          color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
    }
    .spinner {
      width: 12px; height: 12px;
      border: 1.5px solid var(--muted-foreground);
      border-right-color: transparent;
      border-radius: 50%;
      animation: spin 0.8s linear infinite;
      display: inline-block;
      flex-shrink: 0;
    }
    @keyframes spin { to { transform: rotate(360deg); } }
    :host([variant='ghost']) .field {
      background: transparent;
      border-color: transparent;
    }
    :host([variant='ghost']) .field:focus-within {
      background: var(--background);
      border-color: var(--border);
      outline: none;
    }
    :host([invalid]) .field { border-color: var(--destructive); }

    input {
      flex: 1;
      border: 0; outline: 0; background: transparent; color: inherit;
      font: inherit;
      padding: 0;
      min-width: 0;
    }
    input::placeholder { color: var(--muted-foreground); }
    input:disabled { cursor: not-allowed; opacity: 0.6; }

    :host([size='sm']) .field { padding: 2px 8px; }
    :host([size='sm']) input { font-size: 12px; line-height: 20px; }
    :host(:not([size])) .field,
    :host([size='md']) .field { padding: 4px 10px; }
    :host(:not([size])) input,
    :host([size='md']) input { font-size: 13px; line-height: 22px; }
    :host([size='lg']) .field { padding: 6px 12px; }
    :host([size='lg']) input { font-size: 16px; line-height: 28px; font-weight: 600; }
  `;
ShipInput.properties = {
  type: { type: String },
  value: { type: String },
  placeholder: { type: String },
  name: { type: String },
  autocomplete: { type: String },
  required: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true },
  readonly: { type: Boolean, reflect: true },
  invalid: { type: Boolean, reflect: true },
  loading: { type: Boolean, reflect: true },
  variant: { type: String, reflect: true },
  size: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-input")) {
  customElements.define("ship-input", ShipInput);
}

// src/web-components/ship-label.ts
var ShipLabel = class extends i4 {
  constructor() {
    super(...arguments);
    this.htmlFor = "";
    this.size = "sm";
    this.required = false;
  }
  render() {
    return b2`
      <label for=${this.htmlFor}>
        <slot></slot>${this.required ? b2`<span class="req" aria-hidden="true">*</span>` : null}
      </label>
    `;
  }
};
ShipLabel.styles = i`
    :host { display: inline-block; }
    label {
      font-family: var(--font-body);
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      color: var(--muted-foreground);
    }
    :host([size='sm']) label,
    :host(:not([size])) label { font-size: 10px; }
    :host([size='md']) label { font-size: 11px; }
    .req { color: var(--destructive); margin-left: 2px; }
  `;
ShipLabel.properties = {
  htmlFor: { type: String, attribute: "for" },
  size: { type: String, reflect: true },
  required: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-label")) {
  customElements.define("ship-label", ShipLabel);
}

// src/web-components/ship-list-item.ts
var ShipListItem = class extends i4 {
  constructor() {
    super(...arguments);
    this.selected = false;
    this.disabled = false;
  }
  onClick(e6) {
    if (this.disabled) return;
    this.dispatchEvent(new CustomEvent("ship-select", {
      detail: {},
      bubbles: true,
      composed: true
    }));
    void e6;
  }
  render() {
    return b2`
      <button
        part="button"
        role="option"
        aria-selected=${this.selected ? "true" : "false"}
        ?disabled=${this.disabled}
        @click=${this.onClick}
      >
        <div class="row">
          <slot name="leading"></slot>
          <span class="primary"><slot></slot></span>
          <slot name="trailing"></slot>
        </div>
        <div class="meta"><slot name="meta"></slot></div>
      </button>
    `;
  }
};
ShipListItem.styles = i`
    :host { display: block; }
    button {
      width: 100%;
      text-align: left;
      background: transparent;
      border: 1px solid transparent;
      border-radius: var(--radius-md);
      padding: 8px 10px;
      cursor: pointer;
      display: flex;
      flex-direction: column;
      gap: 4px;
      color: inherit;
      font: inherit;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out);
    }
    button:hover:not(:disabled) { background: var(--muted); border-color: var(--border); }
    button:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    :host([selected]) button {
      background: var(--muted);
      border-color: var(--primary);
    }
    button:disabled { opacity: 0.6; cursor: not-allowed; }

    .row { display: flex; align-items: center; gap: 8px; }
    .primary { flex: 1; min-width: 0; font-weight: var(--font-weight-medium); font-size: var(--text-base); }
    .meta {
      display: flex; gap: 6px; align-items: center;
      font-size: 10px; color: var(--muted-foreground);
    }
    ::slotted([slot='leading']), ::slotted([slot='trailing']) {
      display: inline-flex; align-items: center;
    }
  `;
ShipListItem.properties = {
  selected: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-list-item")) {
  customElements.define("ship-list-item", ShipListItem);
}

// src/web-components/ship-popover.ts
var ShipPopover = class extends i4 {
  constructor() {
    super(...arguments);
    this.open = false;
    this.placement = "bottom";
    this.onDocClick = (e6) => {
      if (!this.open) return;
      const target = e6.composedPath()[0];
      if (!this.contains(target) && target !== this) this.hide();
    };
  }
  show() {
    this.open = true;
    this.emit("ship-open");
  }
  hide() {
    this.open = false;
    this.emit("ship-close");
  }
  toggle() {
    this.open ? this.hide() : this.show();
  }
  emit(name) {
    this.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }));
  }
  connectedCallback() {
    super.connectedCallback();
    document.addEventListener("click", this.onDocClick);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    document.removeEventListener("click", this.onDocClick);
  }
  render() {
    return b2`
      <span @click=${() => this.toggle()}><slot name="trigger"></slot></span>
      <div class="panel" role="dialog"><slot></slot></div>
    `;
  }
};
ShipPopover.styles = i`
    :host { position: relative; display: inline-flex; }
    .panel {
      position: absolute;
      z-index: var(--z-popover);
      min-width: 180px;
      padding: 6px;
      background: var(--popover);
      color: var(--popover-foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      box-shadow: var(--shadow-lg);
      font-family: var(--font-body);
      font-size: var(--text-base);
      opacity: 0;
      transform: translateY(-4px);
      pointer-events: none;
      transition:
        opacity var(--duration-medium) var(--ease-out),
        transform var(--duration-medium) var(--ease-out);
    }
    :host([open]) .panel {
      opacity: 1;
      transform: translateY(0);
      pointer-events: auto;
    }
    :host([placement='bottom']) .panel,
    :host(:not([placement])) .panel { top: calc(100% + 6px); left: 0; }
    :host([placement='top']) .panel { bottom: calc(100% + 6px); left: 0; }
    :host([placement='right']) .panel { left: calc(100% + 6px); top: 0; }
    :host([placement='left']) .panel { right: calc(100% + 6px); top: 0; }
  `;
ShipPopover.properties = {
  open: { type: Boolean, reflect: true },
  placement: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-popover")) {
  customElements.define("ship-popover", ShipPopover);
}

// src/web-components/ship-progress.ts
var ShipProgress = class extends i4 {
  constructor() {
    super();
    this.value = null;
    this.max = 100;
    this.tone = "info";
    this.size = "md";
  }
  render() {
    const pct = this.value == null ? null : Math.max(0, Math.min(100, this.value / this.max * 100));
    return b2`
      <div class="track" role="progressbar"
        aria-valuemin="0"
        aria-valuemax=${this.max}
        aria-valuenow=${this.value ?? ""}
      >
        <div class="bar" style=${pct != null ? `width: ${pct}%` : ""}></div>
      </div>
    `;
  }
};
ShipProgress.styles = [
  toneVars,
  i`
      :host { display: block; }
      .track {
        background: var(--muted);
        border-radius: 999px;
        overflow: hidden;
        height: 6px;
      }
      :host([size='sm']) .track { height: 3px; }
      .bar {
        height: 100%;
        background: var(--tone-fg);
        border-radius: inherit;
        transition: width 180ms ease;
      }
      :host(:not([value])) .bar {
        width: 35%;
        animation: indet 1.4s cubic-bezier(0.4, 0, 0.2, 1) infinite;
      }
      @keyframes indet {
        0% { transform: translateX(-100%); }
        60% { transform: translateX(200%); }
        100% { transform: translateX(200%); }
      }
    `
];
ShipProgress.properties = {
  value: { type: Number, reflect: true },
  max: { type: Number },
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-progress")) {
  customElements.define("ship-progress", ShipProgress);
}

// src/web-components/ship-select.ts
var ShipSelect = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = "";
    this.name = "";
    this.required = false;
    this.disabled = false;
    this.size = "md";
  }
  get selectEl() {
    return this.renderRoot.querySelector("select");
  }
  firstUpdated() {
    this.syncOptions();
  }
  updated() {
    this.syncOptions();
  }
  syncOptions() {
    if (!this.selectEl) return;
    const opts = Array.from(this.querySelectorAll(":scope > option, :scope > optgroup"));
    const fragment = document.createDocumentFragment();
    for (const o6 of opts) fragment.appendChild(o6.cloneNode(true));
    this.selectEl.innerHTML = "";
    this.selectEl.appendChild(fragment);
    this.selectEl.value = this.value;
  }
  onChange(e6) {
    this.value = e6.target.value;
    this.dispatchEvent(new CustomEvent("ship-change", {
      detail: { value: this.value },
      bubbles: true,
      composed: true
    }));
  }
  render() {
    return b2`
      <select part="select"
        name=${this.name}
        ?required=${this.required}
        ?disabled=${this.disabled}
        @change=${this.onChange}
      ></select>
      <span hidden><slot @slotchange=${this.syncOptions}></slot></span>
    `;
  }
};
ShipSelect.styles = i`
    :host { display: inline-flex; }
    select {
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 5px 8px;
      font: inherit; font-size: 12px;
      background: var(--background);
      color: var(--foreground);
      cursor: pointer;
    }
    select:focus-visible { outline: 2px solid var(--primary); outline-offset: 1px; }
    select:disabled { cursor: not-allowed; opacity: 0.6; }
    :host([size='sm']) select { font-size: 11px; padding: 3px 6px; }
  `;
ShipSelect.properties = {
  value: { type: String },
  name: { type: String },
  required: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true },
  size: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-select")) {
  customElements.define("ship-select", ShipSelect);
}

// src/web-components/ship-separator.ts
var ShipSeparator = class extends i4 {
  constructor() {
    super(...arguments);
    this.orientation = "horizontal";
    this.variant = "solid";
  }
  render() {
    this.setAttribute("role", "separator");
    this.setAttribute("aria-orientation", this.orientation);
    return b2``;
  }
};
ShipSeparator.styles = i`
    :host { display: block; background: var(--border); }
    :host(:not([orientation='vertical'])) { height: 1px; width: 100%; margin: 8px 0; }
    :host([orientation='vertical']) { width: 1px; min-height: 100%; display: inline-block; margin: 0 8px; }
    :host([variant='dashed']) {
      background: transparent;
      border-top: 1px dashed var(--border);
      height: 0;
    }
    :host([variant='dashed'][orientation='vertical']) {
      border-top: 0;
      border-left: 1px dashed var(--border);
      width: 0;
    }
  `;
ShipSeparator.properties = {
  orientation: { type: String, reflect: true },
  variant: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-separator")) {
  customElements.define("ship-separator", ShipSeparator);
}

// src/web-components/ship-spinner.ts
var ShipSpinner = class extends i4 {
  constructor() {
    super();
    this.tone = "info";
    this.size = "md";
    this.label = "Loading";
  }
  render() {
    return b2`<span class="spin" part="spinner" role="status" aria-label=${this.label}></span>`;
  }
};
ShipSpinner.styles = [
  toneVars,
  i`
      :host { display: inline-flex; vertical-align: middle; }
      .spin {
        width: 14px; height: 14px;
        border: 2px solid color-mix(in oklch, var(--tone-fg) 25%, transparent);
        border-top-color: var(--tone-fg);
        border-radius: 50%;
        animation: spin 0.9s linear infinite;
      }
      :host([size='sm']) .spin { width: 10px; height: 10px; border-width: 1.5px; }
      :host([size='lg']) .spin { width: 22px; height: 22px; border-width: 2.5px; }
      @keyframes spin { to { transform: rotate(360deg); } }
    `
];
ShipSpinner.properties = {
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true },
  label: { type: String }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-spinner")) {
  customElements.define("ship-spinner", ShipSpinner);
}

// src/web-components/ship-stat.ts
var ShipStat = class extends i4 {
  constructor() {
    super(...arguments);
    this.accent = "default";
    this.interactive = false;
  }
  render() {
    return b2`
      <span class="label"><slot name="label"></slot></span>
      <span class="value-row">
        <span class="value"><slot></slot></span>
        <span class="trend"><slot name="trend"></slot></span>
      </span>
      <span class="hint"><slot name="hint"></slot></span>
    `;
  }
};
ShipStat.styles = i`
    :host {
      display: flex;
      flex-direction: column;
      gap: 4px;
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 16px 18px;
      transition:
        border-color var(--duration-medium) var(--ease-out),
        transform var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
    }
    :host([interactive]) { cursor: pointer; }
    :host([interactive]:hover) {
      border-color: color-mix(in oklch, var(--border) 180%, transparent);
      transform: translateY(-1px);
      box-shadow: var(--shadow-sm);
    }
    .label {
      font-size: var(--text-xs);
      font-weight: var(--font-weight-medium);
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--muted-foreground);
      font-family: var(--font-body);
    }
    .value-row {
      display: flex;
      align-items: baseline;
      gap: 8px;
      flex-wrap: wrap;
    }
    .value {
      font-family: var(--font-display);
      font-size: var(--text-3xl);
      font-weight: var(--font-weight-semibold);
      font-variant-numeric: tabular-nums;
      letter-spacing: -0.02em;
      color: var(--foreground);
    }
    .trend {
      display: inline-flex;
      align-items: center;
    }
    :host([accent='primary']) .value { color: var(--primary); }
    :host([accent='info']) .value { color: var(--info); }
    :host([accent='success']) .value { color: var(--success); }
    :host([accent='destructive']) .value { color: var(--destructive); }
    :host([accent='warning']) .value { color: var(--warning); }
    :host([accent='accent']) .value { color: var(--accent); }

    .hint {
      font-size: 11px;
      color: var(--muted-foreground);
    }
    ::slotted([slot='trend']) { font-size: var(--text-sm); }
  `;
ShipStat.properties = {
  accent: { type: String, reflect: true },
  interactive: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-stat")) {
  customElements.define("ship-stat", ShipStat);
}

// src/web-components/ship-status-dot.ts
var ShipStatusDot = class extends i4 {
  constructor() {
    super();
    this.tone = "success";
    this.pulse = false;
    this.size = "md";
  }
  render() {
    return b2`<span class="dot" part="dot" aria-hidden="true"></span>`;
  }
};
ShipStatusDot.styles = [
  toneVars,
  i`
      :host { display: inline-flex; vertical-align: middle; }
      .dot {
        display: inline-block;
        width: 8px; height: 8px;
        border-radius: 50%;
        background: var(--tone-fg);
        box-shadow: 0 0 0 4px color-mix(in oklch, var(--tone-fg) 20%, transparent);
      }
      :host([size='sm']) .dot { width: 6px; height: 6px; box-shadow: 0 0 0 3px color-mix(in oklch, var(--tone-fg) 20%, transparent); }
      :host([pulse]) .dot { animation: pulse 2.5s infinite; }
      @keyframes pulse {
        0%, 100% { opacity: 1; transform: scale(1); }
        50% { opacity: 0.75; transform: scale(0.9); }
      }
    `
];
ShipStatusDot.properties = {
  tone: { type: String, reflect: true },
  pulse: { type: Boolean, reflect: true },
  size: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-status-dot")) {
  customElements.define("ship-status-dot", ShipStatusDot);
}

// src/web-components/ship-switch.ts
var ShipSwitch = class extends i4 {
  constructor() {
    super(...arguments);
    this.checked = false;
    this.disabled = false;
    this.name = "";
    this.value = "on";
  }
  onChange(e6) {
    const c4 = e6.target.checked;
    this.checked = c4;
    this.dispatchEvent(new CustomEvent("ship-change", {
      detail: { checked: c4 },
      bubbles: true,
      composed: true
    }));
  }
  render() {
    return b2`
      <label>
        <span class="track" part="track">
          <input
            part="input"
            type="checkbox"
            role="switch"
            aria-checked=${this.checked ? "true" : "false"}
            .checked=${this.checked}
            ?disabled=${this.disabled}
            name=${this.name}
            value=${this.value}
            @change=${this.onChange}
          />
          <span class="thumb" part="thumb"></span>
        </span>
        <slot></slot>
      </label>
    `;
  }
};
ShipSwitch.styles = i`
    :host { display: inline-flex; align-items: center; gap: 8px; font-size: 13px; color: var(--foreground); }
    :host([disabled]) { opacity: 0.6; cursor: not-allowed; }
    label { display: inline-flex; align-items: center; gap: 8px; cursor: inherit; }
    .track {
      position: relative;
      display: inline-block;
      width: 30px; height: 18px;
      background: var(--muted);
      border: 1px solid var(--border);
      border-radius: 999px;
      transition: background 120ms, border-color 120ms;
    }
    .thumb {
      position: absolute;
      top: 1px; left: 1px;
      width: 14px; height: 14px;
      background: var(--background);
      border-radius: 50%;
      box-shadow: 0 1px 2px oklch(0.147 0.004 49.25 / 25%);
      transition: transform 140ms ease;
    }
    :host([checked]) .track {
      background: var(--primary);
      border-color: var(--primary);
    }
    :host([checked]) .thumb { transform: translateX(12px); }
    input {
      position: absolute; opacity: 0; width: 100%; height: 100%;
      margin: 0; cursor: inherit;
      inset: 0;
    }
    input:focus-visible + .thumb { outline: 2px solid var(--primary); outline-offset: 2px; }
  `;
ShipSwitch.properties = {
  checked: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true },
  name: { type: String },
  value: { type: String }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-switch")) {
  customElements.define("ship-switch", ShipSwitch);
}

// src/web-components/ship-tabs.ts
var ShipTab = class extends i4 {
  constructor() {
    super(...arguments);
    this.name = "";
    this.label = "";
    this.count = null;
    this.disabled = false;
  }
  render() {
    return b2``;
  }
};
ShipTab.styles = i`:host { display: none; }`;
ShipTab.properties = {
  name: { type: String },
  label: { type: String },
  count: { type: Number },
  disabled: { type: Boolean }
};
var ShipTabPanel = class extends i4 {
  constructor() {
    super(...arguments);
    this.name = "";
    this.active = false;
  }
  render() {
    return b2`<slot></slot>`;
  }
};
ShipTabPanel.styles = i`
    :host { display: none; }
    :host([active]) { display: block; animation: fade 180ms ease; }
    @keyframes fade {
      from { opacity: 0; transform: translateY(4px); }
      to { opacity: 1; transform: translateY(0); }
    }
  `;
ShipTabPanel.properties = {
  name: { type: String },
  active: { type: Boolean, reflect: true }
};
var ShipTabs = class extends i4 {
  constructor() {
    super(...arguments);
    this.active = "";
    this.variant = "underline";
    this.sticky = false;
  }
  get tabs() {
    return Array.from(this.querySelectorAll(":scope > ship-tab"));
  }
  get panels() {
    return Array.from(this.querySelectorAll(":scope > ship-tab-panel"));
  }
  firstUpdated() {
    this.syncPanels();
  }
  updated() {
    this.syncPanels();
  }
  syncPanels() {
    for (const p3 of this.panels) p3.active = p3.name === this.active;
  }
  activate(name) {
    this.active = name;
    this.dispatchEvent(new CustomEvent("ship-tab-change", {
      detail: { name },
      bubbles: true,
      composed: true
    }));
  }
  onKeydown(e6) {
    if (e6.key !== "ArrowRight" && e6.key !== "ArrowLeft") return;
    e6.preventDefault();
    const tabs = this.tabs.filter((t4) => !t4.disabled);
    const names = tabs.map((t4) => t4.name);
    const i6 = names.indexOf(this.active);
    const next = e6.key === "ArrowRight" ? (i6 + 1) % names.length : (i6 - 1 + names.length) % names.length;
    this.activate(names[next]);
  }
  render() {
    const tabs = this.tabs.map((t4) => ({
      name: t4.name,
      label: t4.label || t4.name,
      count: t4.count,
      disabled: t4.disabled
    }));
    return b2`
      <div class="tablist ${this.variant}" role="tablist" @keydown=${this.onKeydown}>
        ${tabs.map((t4) => b2`
          <button
            role="tab"
            aria-selected=${t4.name === this.active ? "true" : "false"}
            tabindex=${t4.name === this.active ? "0" : "-1"}
            ?disabled=${t4.disabled}
            @click=${() => this.activate(t4.name)}
          >
            ${t4.label}
            ${t4.count != null ? b2`<span class="count">${t4.count}</span>` : null}
          </button>
        `)}
      </div>
      <div class="panels"><slot></slot></div>
    `;
  }
};
ShipTabs.styles = i`
    :host { display: block; font-family: var(--font-body); }

    /* Underline variant (default) — matches pipelines .tabs-bar */
    .tablist.underline {
      display: flex; gap: 2px;
      border-bottom: 1px solid var(--border);
      margin-bottom: 24px;
    }
    :host([sticky]) .tablist.underline {
      position: sticky; top: 0; background: var(--background); z-index: 5;
      padding-top: 4px;
    }
    .tablist.underline button {
      background: transparent; border: 0;
      color: var(--muted-foreground);
      padding: 10px 16px;
      font: inherit;
      font-size: 13px; font-weight: 500;
      cursor: pointer;
      border-bottom: 2px solid transparent;
      margin-bottom: -1px;
      display: inline-flex; align-items: center; gap: 8px;
      transition: color 120ms;
    }
    .tablist.underline button:hover:not(:disabled) { color: var(--foreground); }
    .tablist.underline button[aria-selected='true'] {
      color: var(--foreground);
      border-bottom-color: var(--primary);
    }

    /* Segmented variant — matches specs .tabs */
    .tablist.segmented {
      display: flex; gap: 2px;
      padding: 8px 10px;
      border-bottom: 1px solid var(--border);
      background: var(--muted);
    }
    .tablist.segmented button {
      flex: 1;
      padding: 6px 8px;
      border: none;
      background: transparent;
      color: var(--muted-foreground);
      font: inherit;
      font-size: 11px; font-weight: 500;
      cursor: pointer;
      border-radius: 4px;
      text-transform: uppercase;
      letter-spacing: 0.04em;
    }
    .tablist.segmented button[aria-selected='true'] {
      background: var(--background);
      color: var(--foreground);
      box-shadow: 0 1px 2px rgba(0,0,0,0.05);
    }

    button:disabled { opacity: 0.5; cursor: not-allowed; }
    button:focus-visible {
      outline: 2px solid var(--primary);
      outline-offset: -2px;
      border-radius: 4px;
    }

    .count {
      background: var(--muted);
      color: var(--muted-foreground);
      padding: 1px 7px;
      border-radius: 999px;
      font-size: 10px; font-weight: 600;
      font-variant-numeric: tabular-nums;
      min-width: 18px;
      text-align: center;
    }
    .tablist.underline button[aria-selected='true'] .count {
      background: color-mix(in oklch, var(--primary) 15%, transparent);
      color: var(--primary);
    }

    .panels { display: block; }
  `;
ShipTabs.properties = {
  active: { type: String, reflect: true },
  variant: { type: String, reflect: true },
  sticky: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-tab")) customElements.define("ship-tab", ShipTab);
  if (!customElements.get("ship-tab-panel")) customElements.define("ship-tab-panel", ShipTabPanel);
  if (!customElements.get("ship-tabs")) customElements.define("ship-tabs", ShipTabs);
}

// src/web-components/ship-textarea.ts
var ShipTextarea = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = "";
    this.placeholder = "";
    this.name = "";
    this.rows = 4;
    this.required = false;
    this.disabled = false;
    this.readonly = false;
    this.variant = "default";
    this.size = "md";
    this.resize = "vertical";
  }
  onInput(e6) {
    this.value = e6.target.value;
    this.dispatchEvent(new CustomEvent("ship-input", {
      detail: { value: this.value },
      bubbles: true,
      composed: true
    }));
  }
  render() {
    return b2`
      <textarea
        part="textarea"
        .value=${this.value}
        placeholder=${this.placeholder}
        name=${this.name}
        rows=${this.rows}
        ?required=${this.required}
        ?disabled=${this.disabled}
        ?readonly=${this.readonly}
        style="resize: ${this.resize}"
        @input=${this.onInput}
      ></textarea>
    `;
  }
};
ShipTextarea.styles = i`
    :host { display: block; }
    textarea {
      width: 100%;
      box-sizing: border-box;
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 12px;
      background: var(--background);
      color: var(--foreground);
      font: inherit;
      font-family: var(--font-body);
      line-height: 1.5;
    }
    textarea:focus {
      outline: 2px solid var(--primary);
      outline-offset: -1px;
      border-color: transparent;
    }
    textarea::placeholder { color: var(--muted-foreground); }
    textarea:disabled { cursor: not-allowed; opacity: 0.6; }

    :host([variant='code']) textarea {
      font-family: ${MONO};
      font-size: 13px;
    }
    :host([size='sm']) textarea {
      padding: 4px 8px;
      font-size: 12px;
    }
  `;
ShipTextarea.properties = {
  value: { type: String },
  placeholder: { type: String },
  name: { type: String },
  rows: { type: Number },
  required: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true },
  readonly: { type: Boolean, reflect: true },
  variant: { type: String, reflect: true },
  size: { type: String, reflect: true },
  resize: { type: String }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-textarea")) {
  customElements.define("ship-textarea", ShipTextarea);
}

// src/web-components/ship-theme-toggle.ts
var STORAGE_KEY = "ship-theme";
var ShipThemeToggle = class extends i4 {
  constructor() {
    super(...arguments);
    this.theme = "auto";
    this.toggle = () => {
      this.theme = this.theme === "dark" ? "light" : "dark";
      try {
        localStorage.setItem(STORAGE_KEY, this.theme);
      } catch {
      }
      this.apply();
      this.dispatchEvent(new CustomEvent("ship-theme-change", {
        detail: { theme: this.theme },
        bubbles: true,
        composed: true
      }));
    };
  }
  connectedCallback() {
    super.connectedCallback();
    if (typeof window === "undefined") return;
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "light" || saved === "dark") {
      this.theme = saved;
    } else {
      const prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)")?.matches;
      this.theme = prefersDark ? "dark" : "light";
    }
    this.apply();
  }
  apply() {
    const dark = this.theme === "dark";
    document.documentElement.classList.toggle("dark", dark);
  }
  render() {
    const isDark = this.theme === "dark";
    return b2`
      <button @click=${this.toggle} aria-label="Toggle theme" title="Toggle theme">
        ${isDark ? "\u{1F319}" : "\u2600"} <slot>${isDark ? "Dark" : "Light"}</slot>
      </button>
    `;
  }
};
ShipThemeToggle.styles = i`
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
      display: inline-flex; align-items: center; gap: 6px;
    }
    button:hover { background: var(--muted); }
  `;
ShipThemeToggle.properties = {
  theme: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-theme-toggle")) {
  customElements.define("ship-theme-toggle", ShipThemeToggle);
}

// src/web-components/ship-toolbar.ts
var ShipToolbar = class extends i4 {
  constructor() {
    super(...arguments);
    this.justify = "start";
    this.align = "center";
    this.gap = "md";
    this.variant = "default";
  }
  render() {
    return b2`<slot name="spacer"></slot><slot></slot>`;
  }
};
ShipToolbar.styles = i`
    :host { display: flex; align-items: center; gap: 8px; }
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
  `;
ShipToolbar.properties = {
  justify: { type: String, reflect: true },
  align: { type: String, reflect: true },
  gap: { type: String, reflect: true },
  variant: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-toolbar")) {
  customElements.define("ship-toolbar", ShipToolbar);
}

// src/web-components/ship-tooltip.ts
var ShipTooltip = class extends i4 {
  constructor() {
    super(...arguments);
    this.placement = "top";
  }
  render() {
    return b2`
      <slot></slot>
      <span class="bubble" role="tooltip"><slot name="content"></slot></span>
    `;
  }
};
ShipTooltip.styles = i`
    :host { position: relative; display: inline-flex; }
    .bubble {
      position: absolute;
      z-index: var(--z-popover);
      padding: 4px 8px;
      background: var(--foreground);
      color: var(--background);
      font-family: var(--font-body);
      font-size: 11px;
      font-weight: var(--font-weight-medium);
      border-radius: var(--radius-sm);
      white-space: nowrap;
      pointer-events: none;
      opacity: 0;
      transition: opacity var(--duration-fast) var(--ease-out) var(--tooltip-hide-delay);
      box-shadow: var(--shadow-sm);
    }
    :host(:hover) .bubble,
    :host(:focus-within) .bubble {
      opacity: 1;
      transition-delay: var(--tooltip-show-delay);
    }
    :host([placement='top']) .bubble,
    :host(:not([placement])) .bubble {
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
  `;
ShipTooltip.properties = {
  placement: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-tooltip")) {
  customElements.define("ship-tooltip", ShipTooltip);
}

// src/web-components/domain/ship-pipeline-step.ts
var ShipPipelineStep = class extends i4 {
  constructor() {
    super(...arguments);
    this.name = "";
    this.stepType = "agent";
    this.state = "pending";
    this.agent = "";
  }
  glyphFor(state) {
    switch (state) {
      case "completed":
        return "\u2713";
      // ✓
      case "failed":
        return "\u2717";
      // ✗
      case "running":
        return "";
      // spinner
      default:
        return "\u25CB";
    }
  }
  render() {
    const isRunning = this.state === "running";
    return b2`
      ${isRunning ? b2`<ship-spinner size="sm" tone="info" label="Running"></ship-spinner>` : b2`<span class="icon" aria-hidden="true">${this.glyphFor(this.state)}</span>`}
      <span class="name">${this.name}</span>
      <span class="type">
        ${this.stepType}${this.agent ? b2` &middot; ${this.agent}` : ""}
      </span>
      <span class="status">${this.state}</span>
    `;
  }
};
ShipPipelineStep.styles = i`
    :host {
      display: flex; flex-direction: column; gap: 4px;
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
      top: 10px; right: 12px;
      font-size: 10px;
      color: var(--muted-foreground);
      line-height: 1;
    }
    :host([state='completed']) {
      border-color: oklch(0.62 0.17 150 / 50%);
      background: oklch(0.62 0.17 150 / 5%);
    }
    :host([state='completed']) .icon { color: var(--success); }
    :host([state='running']) {
      border-color: oklch(0.65 0.15 240 / 60%);
      background: oklch(0.65 0.15 240 / 7%);
      box-shadow: 0 0 0 3px oklch(0.65 0.15 240 / 12%);
    }
    :host([state='running']) .icon { color: var(--info); }
    :host([state='failed']) {
      border-color: oklch(0.58 0.22 27 / 55%);
      background: oklch(0.58 0.22 27 / 6%);
    }
    :host([state='failed']) .icon { color: var(--destructive); }

    .name {
      font-family: ${MONO};
      font-size: 11.5px; font-weight: 600;
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
    ship-spinner {
      /* same size + placement as the glyph icon it replaces */
      position: absolute; top: 10px; right: 12px;
    }
  `;
ShipPipelineStep.properties = {
  name: { type: String, reflect: true },
  stepType: { type: String, reflect: true, attribute: "step-type" },
  state: { type: String, reflect: true },
  agent: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-pipeline-step")) {
  customElements.define("ship-pipeline-step", ShipPipelineStep);
}

// src/web-components/domain/ship-pipeline-run.ts
var STATE_TO_TONE = {
  dispatched: "info",
  running: "info",
  gatepending: "warning",
  completed: "success",
  failed: "destructive",
  merged: "accent",
  pending: "muted"
};
var ShipPipelineRun = class extends i4 {
  constructor() {
    super(...arguments);
    this.state = "pending";
    this.slug = "";
    this.agent = "";
  }
  render() {
    const tone = STATE_TO_TONE[this.state] ?? "muted";
    return b2`
      <ship-card status-rail tone=${tone}>
        <div slot="header">
          <header>
            <div class="left">
              <span class="slug">${this.slug}</span>
              ${this.agent ? b2`<span class="agent"><code>${this.agent}</code></span>` : null}
              <slot name="actions"></slot>
            </div>
            <ship-badge tone=${tone}>${this.state}</ship-badge>
          </header>
        </div>
        <slot></slot>
        <div slot="footer"><slot name="footer"></slot></div>
      </ship-card>
    `;
  }
};
ShipPipelineRun.styles = i`
    :host { display: block; }
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 12px;
      gap: 12px;
      flex-wrap: wrap;
    }
    .left { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
    .slug {
      font-family: ${MONO};
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
      font-family: ${MONO};
    }
  `;
ShipPipelineRun.properties = {
  state: { type: String, reflect: true },
  slug: { type: String, reflect: true },
  agent: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-pipeline-run")) {
  customElements.define("ship-pipeline-run", ShipPipelineRun);
}

// src/web-components/domain/ship-pipeline-definition.ts
var ShipPipelineDefinition = class extends i4 {
  constructor() {
    super(...arguments);
    this.name = "";
    this.steps = 0;
    this.description = "";
  }
  render() {
    const s4 = this.steps;
    const suffix = s4 === 1 ? "" : "s";
    return b2`
      <ship-card interactive accent-bar>
        <div slot="header">
          <header>
            <span class="name">${this.name}</span>
            <span class="steps">${s4} step${suffix}</span>
            <slot name="actions"></slot>
          </header>
        </div>
        ${this.description ? b2`<p class="desc">${this.description}</p>` : b2`<p class="desc no-desc">No description</p>`}
      </ship-card>
    `;
  }
};
ShipPipelineDefinition.styles = i`
    :host { display: block; }
    header {
      display: flex; justify-content: space-between;
      align-items: baseline; gap: 12px;
    }
    .name {
      font-family: var(--font-display);
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
      margin-top: 6px;
    }
    .no-desc { opacity: 0.5; }
    ship-card { display: block; padding: 18px 20px; }
  `;
ShipPipelineDefinition.properties = {
  name: { type: String, reflect: true },
  steps: { type: Number, reflect: true },
  description: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-pipeline-definition")) {
  customElements.define("ship-pipeline-definition", ShipPipelineDefinition);
}

// src/web-components/domain/ship-pipeline-dag.ts
var ShipPipelineDag = class extends i4 {
  render() {
    return b2`<slot></slot>`;
  }
};
ShipPipelineDag.styles = i`
    :host {
      display: flex;
      align-items: stretch;
      gap: 0;
      overflow-x: auto;
      padding: 4px 0 2px;
    }
    ::slotted(.arrow) {
      display: flex; align-items: center;
      padding: 0 6px;
      color: var(--border);
      font-size: 14px;
      flex-shrink: 0;
    }
  `;
if (typeof customElements !== "undefined" && !customElements.get("ship-pipeline-dag")) {
  customElements.define("ship-pipeline-dag", ShipPipelineDag);
}

// src/web-components/domain/ship-history-row.ts
var ShipHistoryRow = class extends i4 {
  constructor() {
    super(...arguments);
    this.state = "";
  }
  render() {
    return b2`
      <span class="cell"><slot name="slug"></slot></span>
      <span class="cell"><slot name="pipeline"></slot></span>
      <span class="cell"><slot name="status"></slot></span>
      <span class="cell"><slot name="duration"></slot></span>
      <span class="cell"><slot name="completed"></slot></span>
    `;
  }
};
ShipHistoryRow.styles = i`
    :host {
      display: table-row;
      border-bottom: 1px solid var(--border);
      font-family: var(--font-body);
      font-size: 12.5px;
      font-variant-numeric: tabular-nums;
      vertical-align: middle;
    }
    :host(:last-child) { border-bottom: 0; }
    :host(:hover) { background: var(--muted); }
    .cell { display: table-cell; padding: 12px 16px; }
  `;
ShipHistoryRow.properties = {
  state: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-history-row")) {
  customElements.define("ship-history-row", ShipHistoryRow);
}

// src/web-components/domain/ship-spec-row.ts
var STATUS_TO_TONE = {
  draft: "muted",
  ready: "success",
  launched: "warning",
  completed: "success",
  failed: "destructive"
};
var ShipSpecRow = class extends i4 {
  constructor() {
    super(...arguments);
    this.itemTitle = "";
    this.status = "draft";
    this.pipeline = "";
    this.selected = false;
  }
  render() {
    const tone = STATUS_TO_TONE[this.status] ?? "muted";
    return b2`
      <ship-list-item ?selected=${this.selected}>
        ${this.itemTitle || "(untitled)"}
        <span slot="meta">
          <ship-badge tone=${tone} size="sm">${this.status}</ship-badge>
          ${this.pipeline ? b2`<span class="pipeline">${this.pipeline}</span>` : null}
        </span>
      </ship-list-item>
    `;
  }
};
ShipSpecRow.styles = i`
    :host { display: block; }
    .pipeline {
      font-family: ${MONO};
      color: var(--accent);
      font-size: 10px;
    }
  `;
ShipSpecRow.properties = {
  itemTitle: { type: String, reflect: true, attribute: "item-title" },
  status: { type: String, reflect: true },
  pipeline: { type: String, reflect: true },
  selected: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-spec-row")) {
  customElements.define("ship-spec-row", ShipSpecRow);
}

// src/web-components/utility/ship-icon.ts
var ICONS = {
  "chevron-down": "M6 9l6 6 6-6",
  "chevron-right": "M9 6l6 6-6 6",
  "chevron-left": "M15 6l-6 6 6 6",
  "chevron-up": "M6 15l6-6 6 6",
  check: "M20 6L9 17l-5-5",
  x: "M18 6L6 18M6 6l12 12",
  plus: "M12 5v14M5 12h14",
  minus: "M5 12h14",
  search: "M11 4a7 7 0 100 14 7 7 0 000-14zM21 21l-4.3-4.3",
  "alert-circle": "M12 2a10 10 0 100 20 10 10 0 000-20zM12 8v4M12 16h.01",
  info: "M12 2a10 10 0 100 20 10 10 0 000-20zM12 16v-4M12 8h.01",
  star: "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z",
  folder: "M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V7z",
  file: "M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8l-6-6zM14 2v6h6",
  trash: "M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M6 6l1 14a2 2 0 002 2h6a2 2 0 002-2l1-14",
  edit: "M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7M18.5 2.5a2.12 2.12 0 113 3L12 15l-4 1 1-4 9.5-9.5z",
  copy: "M9 9h10a2 2 0 012 2v10a2 2 0 01-2 2H9a2 2 0 01-2-2V11a2 2 0 012-2zM5 15H4a2 2 0 01-2-2V3a2 2 0 012-2h10a2 2 0 012 2v1",
  "external-link": "M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3",
  "arrow-up": "M12 19V5M5 12l7-7 7 7",
  "arrow-down": "M12 5v14M19 12l-7 7-7-7",
  "arrow-left": "M19 12H5M12 19l-7-7 7-7",
  "arrow-right": "M5 12h14M12 5l7 7-7 7",
  sparkles: "M12 3l2 5 5 2-5 2-2 5-2-5-5-2 5-2 2-5zM19 14l1 2 2 1-2 1-1 2-1-2-2-1 2-1 1-2z",
  "file-code": "M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8l-6-6zM14 2v6h6M10 13l-2 2 2 2M14 13l2 2-2 2",
  "chevrons-down-up": "M7 20l5-5 5 5M7 4l5 5 5-5",
  "chevrons-up-down": "M7 15l5 5 5-5M7 9l5-5 5 5",
  "message-square-plus": "M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h9M18 3v6M15 6h6"
};
var ShipIcon = class extends i4 {
  constructor() {
    super(...arguments);
    this.name = "";
    this.size = "md";
    this.tone = "default";
  }
  render() {
    const d3 = this.name ? ICONS[this.name] : void 0;
    return b2`
      <slot>
        ${d3 ? b2`<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
              stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              aria-hidden="true" part="svg">
              <path d=${d3}></path>
            </svg>` : null}
      </slot>
    `;
  }
};
ShipIcon.styles = [
  toneVars,
  i`
      :host { display: inline-flex; align-items: center; justify-content: center; color: var(--tone-fg); }
      svg { width: 16px; height: 16px; display: block; }
      :host([size='sm']) svg { width: 12px; height: 12px; }
      :host([size='lg']) svg { width: 20px; height: 20px; }
      ::slotted(svg) { width: 100%; height: 100%; }
    `
];
ShipIcon.properties = {
  name: { type: String, reflect: true },
  size: { type: String, reflect: true },
  tone: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-icon")) {
  customElements.define("ship-icon", ShipIcon);
}

// src/web-components/domain/ship-git-diff.ts
var ShipGitDiff = class extends i4 {
  constructor() {
    super(...arguments);
    this.files = 0;
    this.additions = 0;
    this.deletions = 0;
    this.empty = false;
    this.allCollapsed = false;
  }
  toggleAll() {
    const next = !this.allCollapsed;
    this.allCollapsed = next;
    const kids = this.querySelectorAll("ship-git-diff-file");
    kids.forEach((f3) => {
      f3.collapsed = next;
    });
  }
  render() {
    if (this.empty || this.files === 0) {
      return b2`<div class="empty">No changes to display</div>`;
    }
    return b2`
      <div class="toolbar">
        <span class="count">${this.files} file${this.files !== 1 ? "s" : ""} changed</span>
        ${this.additions > 0 ? b2`<span class="adds">+${this.additions}</span>` : null}
        ${this.deletions > 0 ? b2`<span class="dels">-${this.deletions}</span>` : null}
        <span class="spacer"></span>
        <button class="toggle" type="button" @click=${this.toggleAll}
          title=${this.allCollapsed ? "Expand all files" : "Collapse all files"}>
          <ship-icon size="sm" name=${this.allCollapsed ? "chevrons-up-down" : "chevrons-down-up"}></ship-icon>
          ${this.allCollapsed ? "Expand all" : "Collapse all"}
        </button>
      </div>
      <div class="body"><slot></slot></div>
    `;
  }
};
ShipGitDiff.styles = i`
    :host {
      display: flex;
      flex-direction: column;
      min-height: 0;
      font-family: var(--font-body);
      background: var(--background);
      color: var(--foreground);
      border: 1px solid color-mix(in oklch, var(--border) 60%, transparent);
      border-radius: var(--radius-lg);
      overflow: hidden;
    }
    .toolbar {
      display: flex; align-items: center; gap: 12px;
      padding: 6px 16px;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 60%, transparent);
      background: color-mix(in oklch, var(--card) 50%, transparent);
      flex-shrink: 0;
    }
    .count { font-size: 12px; color: var(--muted-foreground); }
    .adds { font-size: 11px; color: var(--success); }
    .dels { font-size: 11px; color: var(--destructive); }
    .spacer { flex: 1; }
    .toggle {
      display: inline-flex; align-items: center; gap: 4px;
      padding: 2px 8px;
      border-radius: var(--radius-md);
      background: transparent; border: 0;
      font-family: inherit; font-size: 11px;
      color: var(--muted-foreground);
      cursor: pointer;
      transition: background var(--duration-fast) ease, color var(--duration-fast) ease;
    }
    .toggle:hover { background: var(--muted); color: var(--foreground); }
    .body { flex: 1; overflow: auto; }
    :host([empty]) .toolbar { display: none; }
    .empty {
      display: flex; align-items: center; justify-content: center;
      padding: 48px 16px;
      color: var(--muted-foreground);
      font-size: 13px;
    }
  `;
ShipGitDiff.properties = {
  files: { type: Number, reflect: true },
  additions: { type: Number, reflect: true },
  deletions: { type: Number, reflect: true },
  empty: { type: Boolean, reflect: true },
  allCollapsed: { type: Boolean, state: true }
};
var ShipGitDiffFile = class extends i4 {
  constructor() {
    super(...arguments);
    this.path = "";
    this.additions = 0;
    this.deletions = 0;
    this.collapsed = false;
  }
  toggle() {
    this.collapsed = !this.collapsed;
  }
  render() {
    return b2`
      <button class="header" type="button" @click=${this.toggle}
        aria-expanded=${this.collapsed ? "false" : "true"}>
        <ship-icon class="chev" size="sm" name=${this.collapsed ? "chevron-right" : "chevron-down"}></ship-icon>
        <ship-icon class="fileicon" size="sm" name="file-code"></ship-icon>
        <span class="path">${this.path}</span>
        <span class="stats">
          ${this.additions > 0 ? b2`<span class="adds"><ship-icon size="sm" name="plus"></ship-icon>${this.additions}</span>` : null}
          ${this.deletions > 0 ? b2`<span class="dels"><ship-icon size="sm" name="minus"></ship-icon>${this.deletions}</span>` : null}
        </span>
      </button>
      <div class="body"><slot></slot></div>
    `;
  }
};
ShipGitDiffFile.styles = i`
    :host {
      display: block;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 35%, transparent);
    }
    .header {
      display: flex; align-items: center; gap: 8px;
      width: 100%;
      padding: 6px 12px;
      background: transparent;
      border: 0;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 40%, transparent);
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
      font-family: ${MONO};
      font-size: 12px;
      color: var(--foreground);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    }
    .stats { display: inline-flex; align-items: center; gap: 8px; flex-shrink: 0; }
    .adds, .dels {
      display: inline-flex; align-items: center; gap: 2px;
      font-size: 10px; font-family: ${MONO};
    }
    .adds { color: var(--success); }
    .dels { color: var(--destructive); }
    .body { display: block; }
    :host([collapsed]) .body { display: none; }
  `;
ShipGitDiffFile.properties = {
  path: { type: String, reflect: true },
  additions: { type: Number, reflect: true },
  deletions: { type: Number, reflect: true },
  collapsed: { type: Boolean, reflect: true }
};
var ShipGitDiffHunk = class extends i4 {
  constructor() {
    super(...arguments);
    this.header = "";
  }
  render() {
    return b2`${this.header}`;
  }
};
ShipGitDiffHunk.styles = i`
    :host {
      display: block;
      padding: 2px 12px;
      background: color-mix(in oklch, var(--info) 10%, transparent);
      border-top: 1px solid color-mix(in oklch, var(--info) 20%, transparent);
      border-bottom: 1px solid color-mix(in oklch, var(--info) 20%, transparent);
      font-family: ${MONO};
      font-size: 11px;
      color: var(--info);
      user-select: none;
    }
  `;
ShipGitDiffHunk.properties = { header: { type: String, reflect: true } };
var ShipGitDiffLine = class extends i4 {
  constructor() {
    super(...arguments);
    this.type = "context";
    this.old = "";
    this.new = "";
  }
  sig() {
    return this.type === "add" ? "+" : this.type === "del" ? "-" : " ";
  }
  render() {
    return b2`
      <span class="num">${this.old}</span>
      <span class="num">${this.new}</span>
      <span class="sig">${this.sig()}</span>
      <span class="content"><slot></slot></span>
    `;
  }
};
ShipGitDiffLine.styles = i`
    :host {
      display: flex;
      font-family: ${MONO};
      font-size: 11px;
      line-height: 1.45;
    }
    :host([type='add']) { background: color-mix(in oklch, var(--success) 12%, transparent); }
    :host([type='del']) { background: color-mix(in oklch, var(--destructive) 12%, transparent); }
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
      padding-right: 12px;
      white-space: pre-wrap;
      word-break: break-all;
      color: var(--muted-foreground);
    }
    :host([type='add']) .sig,
    :host([type='add']) .content { color: var(--success); }
    :host([type='del']) .sig,
    :host([type='del']) .content { color: var(--destructive); }
  `;
ShipGitDiffLine.properties = {
  type: { type: String, reflect: true },
  old: { type: String, reflect: true },
  new: { type: String, reflect: true }
};
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-git-diff")) customElements.define("ship-git-diff", ShipGitDiff);
  if (!customElements.get("ship-git-diff-file")) customElements.define("ship-git-diff-file", ShipGitDiffFile);
  if (!customElements.get("ship-git-diff-hunk")) customElements.define("ship-git-diff-hunk", ShipGitDiffHunk);
  if (!customElements.get("ship-git-diff-line")) customElements.define("ship-git-diff-line", ShipGitDiffLine);
}

// src/web-components/observability/ship-log-line.ts
var ShipLogLine = class extends i4 {
  constructor() {
    super(...arguments);
    this.level = "info";
  }
  render() {
    return b2`
      ${this.timestamp ? b2`<span class="ts" part="timestamp">${this.timestamp}</span>` : b2`<span></span>`}
      ${this.source ? b2`<span class="src" part="source">${this.source}</span>` : b2`<span></span>`}
      <span class="msg" part="message"><slot></slot></span>
    `;
  }
};
ShipLogLine.styles = i`
    :host {
      display: grid;
      grid-template-columns: auto auto 1fr;
      gap: 8px;
      align-items: baseline;
      font-family: ${MONO};
      font-size: var(--text-sm);
      line-height: var(--leading-normal);
      padding: 2px 8px;
      border-left: 2px solid transparent;
      color: var(--foreground);
    }
    :host([level='debug']) { border-left-color: var(--muted-foreground); color: var(--muted-foreground); }
    :host([level='info']) { border-left-color: var(--info); }
    :host([level='warn']) { border-left-color: var(--warning); }
    :host([level='error']) {
      border-left-color: var(--destructive);
      background: color-mix(in oklch, var(--destructive) 6%, transparent);
    }
    .ts { color: var(--muted-foreground); font-size: var(--text-xs); white-space: nowrap; }
    .src {
      color: var(--muted-foreground);
      font-size: var(--text-xs);
      font-weight: var(--font-weight-medium);
      text-transform: uppercase;
      letter-spacing: 0.04em;
      white-space: nowrap;
    }
    .msg { white-space: pre-wrap; word-break: break-word; min-width: 0; }
  `;
ShipLogLine.properties = {
  level: { type: String, reflect: true },
  timestamp: { type: String, reflect: true },
  source: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-log-line")) {
  customElements.define("ship-log-line", ShipLogLine);
}

// src/web-components/observability/ship-event-row.ts
var ShipEventRow = class extends i4 {
  render() {
    return b2`
      <div class="head" part="head">
        ${this.actor ? b2`<span class="actor" part="actor">${this.actor}</span>` : null}
        ${this.verb ? b2`<span class="verb" part="verb">${this.verb}</span>` : null}
        ${this.target ? b2`<span class="target" part="target">${this.target}</span>` : null}
        ${this.at ? b2`<span class="at" part="at">${this.at}</span>` : null}
      </div>
      <div class="payload" part="payload"><slot></slot></div>
    `;
  }
};
ShipEventRow.styles = i`
    :host {
      display: block;
      padding: 8px 12px;
      border-bottom: 1px solid var(--border);
      font-family: var(--font-body);
      font-size: var(--text-sm);
      line-height: var(--leading-normal);
    }
    .head { display: flex; flex-wrap: wrap; align-items: baseline; gap: 6px; }
    .actor { font-weight: var(--font-weight-semibold); color: var(--foreground); }
    .verb { color: var(--muted-foreground); }
    .target { font-weight: var(--font-weight-medium); color: var(--accent); }
    .at {
      margin-left: auto;
      color: var(--muted-foreground);
      font-size: var(--text-xs);
      white-space: nowrap;
    }
    .payload { margin-top: 6px; }
    .payload:empty { display: none; }
  `;
ShipEventRow.properties = {
  actor: { type: String, reflect: true },
  verb: { type: String, reflect: true },
  target: { type: String, reflect: true },
  at: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-event-row")) {
  customElements.define("ship-event-row", ShipEventRow);
}

// src/web-components/observability/ship-timestamp.ts
var MINUTE = 6e4;
var HOUR = 36e5;
var DAY = 864e5;
function formatRelative(iso, now = Date.now()) {
  const t4 = Date.parse(iso);
  if (Number.isNaN(t4)) return iso;
  const diff = now - t4;
  const abs = Math.abs(diff);
  const suffix = diff >= 0 ? "ago" : "from now";
  if (abs < MINUTE) return `just now`;
  if (abs < HOUR) return `${Math.floor(abs / MINUTE)}m ${suffix}`;
  if (abs < DAY) return `${Math.floor(abs / HOUR)}h ${suffix}`;
  const days = Math.floor(abs / DAY);
  if (days < 30) return `${days}d ${suffix}`;
  if (days < 365) return `${Math.floor(days / 30)}mo ${suffix}`;
  return `${Math.floor(days / 365)}y ${suffix}`;
}
var ShipTimestamp = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = "";
    this.format = "relative";
  }
  connectedCallback() {
    super.connectedCallback();
    this._timer = setInterval(() => this.requestUpdate(), MINUTE);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    if (this._timer) clearInterval(this._timer);
  }
  render() {
    if (!this.value) return b2`<span></span>`;
    const iso = this.value;
    const rel = formatRelative(iso);
    const parsed = Date.parse(iso);
    const abs = Number.isNaN(parsed) ? iso : new Date(parsed).toISOString();
    this.title = abs;
    if (this.format === "absolute") return b2`<time datetime=${iso}>${abs}</time>`;
    if (this.format === "both")
      return b2`<time datetime=${iso}>${rel}</time><span class="abs">(${abs})</span>`;
    return b2`<time datetime=${iso}>${rel}</time>`;
  }
};
ShipTimestamp.styles = i`
    :host {
      display: inline;
      font-family: var(--font-body);
      font-size: inherit;
      color: inherit;
    }
    .abs { color: var(--muted-foreground); margin-left: 4px; }
  `;
ShipTimestamp.properties = {
  value: { type: String, reflect: true },
  format: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-timestamp")) {
  customElements.define("ship-timestamp", ShipTimestamp);
}

// src/web-components/observability/ship-json-view.ts
var ShipJsonView = class extends i4 {
  constructor() {
    super(...arguments);
    this.data = "";
    this.expanded = false;
    this.depth = 1;
  }
  _renderValue(v2, key, level) {
    const keyPrefix = key !== null ? b2`<span class="key">"${key}"</span>: ` : A;
    if (v2 === null) return b2`${keyPrefix}<span class="null">null</span>`;
    if (typeof v2 === "string") return b2`${keyPrefix}<span class="s">"${v2}"</span>`;
    if (typeof v2 === "number") return b2`${keyPrefix}<span class="n">${v2}</span>`;
    if (typeof v2 === "boolean") return b2`${keyPrefix}<span class="b">${String(v2)}</span>`;
    const isArray = Array.isArray(v2);
    const entries = isArray ? v2.map((x2, i6) => [String(i6), x2]) : Object.entries(v2);
    const open = this.expanded || level < this.depth;
    const open1 = isArray ? "[" : "{";
    const close1 = isArray ? "]" : "}";
    const count = entries.length;
    if (count === 0) return b2`${keyPrefix}${open1}${close1}`;
    return b2`
      <details ?open=${open}>
        <summary>
          <span class="caret">▶</span>
          ${keyPrefix}${open1}<span class="meta">${count} ${isArray ? "items" : "keys"}</span>
        </summary>
        <div class="branch">
          ${entries.map(
      ([k2, val]) => b2`<div class="node">${this._renderValue(val, isArray ? null : k2, level + 1)}</div>`
    )}
        </div>
        ${close1}
      </details>
    `;
  }
  render() {
    if (!this.data) return b2``;
    let parsed;
    try {
      parsed = JSON.parse(this.data);
    } catch (e6) {
      return b2`<span class="error" part="error">Invalid JSON: ${e6.message}</span>`;
    }
    return b2`<div part="root">${this._renderValue(parsed, null, 0)}</div>`;
  }
};
ShipJsonView.styles = i`
    :host {
      display: block;
      font-family: ${MONO};
      font-size: var(--text-sm);
      line-height: var(--leading-normal);
      color: var(--foreground);
    }
    .node { display: block; }
    .branch { margin-left: 12px; border-left: 1px dashed var(--border); padding-left: 10px; }
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
    .key { color: var(--accent); }
    .s { color: var(--success); }
    .n { color: var(--info); }
    .b { color: var(--accent); font-weight: var(--font-weight-medium); }
    .null { color: var(--muted-foreground); font-style: italic; }
    .meta { color: var(--muted-foreground); font-size: var(--text-xs); margin-left: 4px; }
    .error { color: var(--destructive); }
  `;
ShipJsonView.properties = {
  data: { type: String, reflect: true },
  expanded: { type: Boolean, reflect: true },
  depth: { type: Number, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-json-view")) {
  customElements.define("ship-json-view", ShipJsonView);
}

// src/web-components/observability/ship-diff.ts
function diffLines(a3, b3) {
  const A2 = a3.split("\n");
  const B2 = b3.split("\n");
  const n4 = A2.length;
  const m2 = B2.length;
  const dp = Array.from({ length: n4 + 1 }, () => new Array(m2 + 1).fill(0));
  for (let i7 = n4 - 1; i7 >= 0; i7--) {
    for (let j2 = m2 - 1; j2 >= 0; j2--) {
      dp[i7][j2] = A2[i7] === B2[j2] ? dp[i7 + 1][j2 + 1] + 1 : Math.max(dp[i7 + 1][j2], dp[i7][j2 + 1]);
    }
  }
  const out = [];
  let i6 = 0;
  let j = 0;
  while (i6 < n4 && j < m2) {
    if (A2[i6] === B2[j]) {
      out.push({ kind: "ctx", text: A2[i6] });
      i6++;
      j++;
    } else if (dp[i6 + 1][j] >= dp[i6][j + 1]) {
      out.push({ kind: "del", text: A2[i6] });
      i6++;
    } else {
      out.push({ kind: "add", text: B2[j] });
      j++;
    }
  }
  while (i6 < n4) out.push({ kind: "del", text: A2[i6++] });
  while (j < m2) out.push({ kind: "add", text: B2[j++] });
  return out;
}
function parseUnified(value) {
  return value.split("\n").map((line) => {
    if (line.startsWith("@@")) return { kind: "hunk", text: line };
    if (line.startsWith("+")) return { kind: "add", text: line.slice(1) };
    if (line.startsWith("-")) return { kind: "del", text: line.slice(1) };
    return { kind: "ctx", text: line.startsWith(" ") ? line.slice(1) : line };
  });
}
var ShipDiff = class extends i4 {
  constructor() {
    super(...arguments);
    this.from = "";
    this.to = "";
    this.value = "";
  }
  _lines() {
    if (this.value) return parseUnified(this.value);
    if (this.from || this.to) return diffLines(this.from, this.to);
    return [];
  }
  render() {
    const lines = this._lines();
    return b2`${lines.map((l3) => {
      const sign = l3.kind === "add" ? "+" : l3.kind === "del" ? "-" : l3.kind === "hunk" ? "@" : " ";
      return b2`<div class="line ${l3.kind}" part="line"><span class="sign">${sign}</span><span>${l3.text}</span></div>`;
    })}`;
  }
};
ShipDiff.styles = i`
    :host {
      display: block;
      font-family: ${MONO};
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
    .line.add { background: color-mix(in oklch, var(--success) 12%, transparent); color: var(--foreground); }
    .line.del { background: color-mix(in oklch, var(--destructive) 12%, transparent); color: var(--foreground); }
    .line.hunk { background: color-mix(in oklch, var(--info) 10%, transparent); color: var(--info); }
    .sign { color: var(--muted-foreground); user-select: none; }
    .line.add .sign { color: var(--success); }
    .line.del .sign { color: var(--destructive); }
  `;
ShipDiff.properties = {
  from: { type: String, reflect: true },
  to: { type: String, reflect: true },
  value: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-diff")) {
  customElements.define("ship-diff", ShipDiff);
}

// src/web-components/observability/ship-code.ts
var ShipCode = class extends i4 {
  constructor() {
    super(...arguments);
    this.block = false;
  }
  render() {
    if (this.block) {
      return b2`
        <div class="block" part="block">
          ${this.language ? b2`<header part="label">${this.language}</header>` : null}
          <pre part="pre"><slot></slot></pre>
        </div>
      `;
    }
    return b2`<code class="inline" part="inline"><slot></slot></code>`;
  }
};
ShipCode.styles = i`
    :host {
      display: inline;
      font-family: ${MONO};
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
  `;
ShipCode.properties = {
  language: { type: String, reflect: true, attribute: "lang" },
  block: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-code")) {
  customElements.define("ship-code", ShipCode);
}

// src/web-components/observability/ship-copy-button.ts
var ShipCopyButton = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = "";
    this.label = "Copy";
    this.copied = false;
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    if (this._resetTimer) clearTimeout(this._resetTimer);
  }
  async _copy() {
    try {
      await navigator.clipboard.writeText(this.value);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = this.value;
      document.body.appendChild(ta);
      ta.select();
      try {
        document.execCommand("copy");
      } catch {
      }
      ta.remove();
    }
    this.copied = true;
    this.dispatchEvent(
      new CustomEvent("ship-copy", {
        detail: { value: this.value },
        bubbles: true,
        composed: true
      })
    );
    if (this._resetTimer) clearTimeout(this._resetTimer);
    this._resetTimer = setTimeout(() => {
      this.copied = false;
    }, 1200);
  }
  render() {
    const icon = this.copied ? b2`<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><polyline points="3 8 7 12 13 4"></polyline></svg>` : b2`<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><rect x="4" y="4" width="9" height="9" rx="1"></rect><path d="M11 4V3a1 1 0 0 0-1-1H3a1 1 0 0 0-1 1v7a1 1 0 0 0 1 1h1"></path></svg>`;
    return b2`
      <button
        type="button"
        part="button"
        aria-label=${this.label}
        @click=${this._copy}
      >
        ${icon}
        <slot>${this.copied ? "Copied" : this.label}</slot>
      </button>
    `;
  }
};
ShipCopyButton.styles = i`
    :host { display: inline-flex; }
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
    button:hover { color: var(--foreground); background: var(--muted); }
    button:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    :host([copied]) button { color: var(--success); border-color: var(--success); }
    svg { width: 12px; height: 12px; }
  `;
ShipCopyButton.properties = {
  value: { type: String, reflect: true },
  label: { type: String, reflect: true },
  copied: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-copy-button")) {
  customElements.define("ship-copy-button", ShipCopyButton);
}

// ../../node_modules/.pnpm/lit-html@3.3.2/node_modules/lit-html/directive.js
var t3 = { ATTRIBUTE: 1, CHILD: 2, PROPERTY: 3, BOOLEAN_ATTRIBUTE: 4, EVENT: 5, ELEMENT: 6 };
var e4 = (t4) => (...e6) => ({ _$litDirective$: t4, values: e6 });
var i5 = class {
  constructor(t4) {
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AT(t4, e6, i6) {
    this._$Ct = t4, this._$AM = e6, this._$Ci = i6;
  }
  _$AS(t4, e6) {
    return this.update(t4, e6);
  }
  update(t4, e6) {
    return this.render(...e6);
  }
};

// ../../node_modules/.pnpm/lit-html@3.3.2/node_modules/lit-html/directives/unsafe-html.js
var e5 = class extends i5 {
  constructor(i6) {
    if (super(i6), this.it = A, i6.type !== t3.CHILD) throw Error(this.constructor.directiveName + "() can only be used in child bindings");
  }
  render(r4) {
    if (r4 === A || null == r4) return this._t = void 0, this.it = r4;
    if (r4 === E) return r4;
    if ("string" != typeof r4) throw Error(this.constructor.directiveName + "() called with a non-string value");
    if (r4 === this.it) return this._t;
    this.it = r4;
    const s4 = [r4];
    return s4.raw = s4, this._t = { _$litType$: this.constructor.resultType, strings: s4, values: [] };
  }
};
e5.directiveName = "unsafeHTML", e5.resultType = 1;
var o5 = e4(e5);

// src/web-components/observability/ship-markdown.ts
function escape(s4) {
  return s4.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
function inline(s4) {
  let out = escape(s4);
  out = out.replace(/`([^`]+)`/g, (_2, c4) => `<code>${c4}</code>`);
  out = out.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  out = out.replace(/(^|[^*])\*([^*\n]+)\*/g, "$1<em>$2</em>");
  out = out.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_2, t4, h3) => `<a href="${h3}" rel="noopener noreferrer">${t4}</a>`);
  return out;
}
function renderMarkdown(src) {
  const lines = src.replace(/\r\n/g, "\n").split("\n");
  const out = [];
  let i6 = 0;
  while (i6 < lines.length) {
    const line = lines[i6];
    if (/^```/.test(line)) {
      const lang = line.slice(3).trim();
      const buf2 = [];
      i6++;
      while (i6 < lines.length && !/^```/.test(lines[i6])) buf2.push(lines[i6++]);
      i6++;
      out.push(`<pre data-lang="${escape(lang)}"><code>${escape(buf2.join("\n"))}</code></pre>`);
      continue;
    }
    const h3 = /^(#{1,6})\s+(.*)$/.exec(line);
    if (h3) {
      out.push(`<h${h3[1].length}>${inline(h3[2])}</h${h3[1].length}>`);
      i6++;
      continue;
    }
    if (/^\s*[-*]\s+/.test(line)) {
      const buf2 = [];
      while (i6 < lines.length && /^\s*[-*]\s+/.test(lines[i6])) {
        buf2.push(`<li>${inline(lines[i6].replace(/^\s*[-*]\s+/, ""))}</li>`);
        i6++;
      }
      out.push(`<ul>${buf2.join("")}</ul>`);
      continue;
    }
    if (/^\s*\d+\.\s+/.test(line)) {
      const buf2 = [];
      while (i6 < lines.length && /^\s*\d+\.\s+/.test(lines[i6])) {
        buf2.push(`<li>${inline(lines[i6].replace(/^\s*\d+\.\s+/, ""))}</li>`);
        i6++;
      }
      out.push(`<ol>${buf2.join("")}</ol>`);
      continue;
    }
    if (/^>\s?/.test(line)) {
      const buf2 = [];
      while (i6 < lines.length && /^>\s?/.test(lines[i6])) {
        buf2.push(lines[i6].replace(/^>\s?/, ""));
        i6++;
      }
      out.push(`<blockquote>${inline(buf2.join(" "))}</blockquote>`);
      continue;
    }
    if (line.trim() === "") {
      i6++;
      continue;
    }
    const buf = [];
    while (i6 < lines.length && lines[i6].trim() !== "" && !/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(lines[i6])) {
      buf.push(lines[i6]);
      i6++;
    }
    out.push(`<p>${inline(buf.join(" "))}</p>`);
  }
  return out.join("\n");
}
var ShipMarkdown = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = "";
  }
  render() {
    return b2`${o5(renderMarkdown(this.value ?? ""))}`;
  }
};
ShipMarkdown.styles = i`
    :host {
      display: block;
      font-family: var(--font-body);
      font-size: var(--text-base);
      line-height: var(--leading-normal);
      color: var(--foreground);
    }
    h1, h2, h3, h4, h5, h6 {
      font-family: var(--font-display);
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
      font-family: ${MONO};
      font-size: 0.92em;
      padding: 1px 5px;
      border-radius: var(--radius-sm);
      background: var(--muted);
      border: 1px solid var(--border);
    }
    pre {
      font-family: ${MONO};
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
  `;
ShipMarkdown.properties = {
  value: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-markdown")) {
  customElements.define("ship-markdown", ShipMarkdown);
}

// src/web-components/observability/ship-table.ts
var ShipTable = class extends i4 {
  constructor() {
    super(...arguments);
    this.dense = false;
    this.hoverable = true;
  }
  render() {
    return b2`
      <div class="wrap" part="wrap">
        <table part="table">
          <thead part="thead"><slot name="head"></slot></thead>
          <tbody part="tbody"><slot></slot></tbody>
        </table>
      </div>
    `;
  }
};
ShipTable.styles = i`
    :host {
      display: block;
      width: 100%;
      font-family: var(--font-body);
      font-size: var(--text-sm);
      color: var(--foreground);
    }
    .wrap { overflow-x: auto; border: 1px solid var(--border); border-radius: var(--radius-md); }
    table { width: 100%; border-collapse: collapse; }
    ::slotted(tr) { border-bottom: 1px solid var(--border); }
    ::slotted(tr:last-child) { border-bottom: none; }
    ::slotted(th), ::slotted(td) {
      text-align: left;
      padding: 8px 12px;
      vertical-align: middle;
    }
    ::slotted(th) {
      font-weight: var(--font-weight-semibold);
      font-size: var(--text-xs);
      text-transform: uppercase;
      letter-spacing: 0.04em;
      color: var(--muted-foreground);
      background: color-mix(in oklch, var(--muted) 60%, var(--background));
    }
    :host([dense]) ::slotted(th),
    :host([dense]) ::slotted(td) { padding: 4px 8px; }
    :host([hoverable]) ::slotted(tr:hover) { background: var(--muted); }
    thead { border-bottom: 1px solid var(--border); }
  `;
ShipTable.properties = {
  dense: { type: Boolean, reflect: true },
  hoverable: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-table")) {
  customElements.define("ship-table", ShipTable);
}

// src/web-components/observability/ship-timeline.ts
var ShipTimeline = class extends i4 {
  render() {
    return b2`<div class="list" part="list"><slot></slot></div>`;
  }
};
ShipTimeline.styles = i`
    :host { display: block; position: relative; }
    .list { display: flex; flex-direction: column; gap: 4px; }
  `;
var ShipTimelineItem = class extends i4 {
  constructor() {
    super(...arguments);
    this.tone = "default";
    this.interactive = false;
    this._onClick = () => {
      if (this.interactive) this._emit();
    };
    this._onKey = (e6) => {
      if (!this.interactive) return;
      if (e6.key === "Enter" || e6.key === " ") {
        e6.preventDefault();
        this._emit();
      }
    };
  }
  connectedCallback() {
    super.connectedCallback();
    this.addEventListener("keydown", this._onKey);
    this.addEventListener("click", this._onClick);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("keydown", this._onKey);
    this.removeEventListener("click", this._onClick);
  }
  _emit() {
    this.dispatchEvent(new CustomEvent("ship-select", { bubbles: true, composed: true, detail: {} }));
  }
  updated() {
    if (this.interactive) {
      if (!this.hasAttribute("role")) this.setAttribute("role", "button");
      if (!this.hasAttribute("tabindex")) this.setAttribute("tabindex", "0");
    }
  }
  render() {
    return b2`
      <div class="gutter"><span class="dot" part="dot"></span></div>
      <div class="body" part="body">
        <div class="head">
          <slot name="title"></slot>
          ${this.at ? b2`<span class="at" part="at">${this.at}</span>` : null}
        </div>
        <div class="content"><slot></slot></div>
      </div>
    `;
  }
};
ShipTimelineItem.styles = [
  toneVars,
  i`
      :host {
        display: grid;
        grid-template-columns: 20px 1fr;
        column-gap: 10px;
        padding: 6px 0;
        position: relative;
      }
      .gutter {
        position: relative;
        display: flex;
        justify-content: center;
      }
      .gutter::before {
        content: '';
        position: absolute;
        top: 0; bottom: 0;
        width: 1px;
        background: var(--border);
      }
      :host(:first-of-type) .gutter::before { top: 10px; }
      :host(:last-of-type) .gutter::before { bottom: calc(100% - 10px); }
      .dot {
        position: relative;
        width: 10px; height: 10px;
        border-radius: var(--radius-full);
        background: var(--tone-fg);
        border: 2px solid var(--background);
        margin-top: 4px;
        box-shadow: var(--shadow-xs);
      }
      .body { min-width: 0; }
      .head {
        display: flex;
        align-items: baseline;
        gap: 8px;
        font-family: var(--font-body);
        font-size: var(--text-sm);
        font-weight: var(--font-weight-medium);
        color: var(--foreground);
      }
      .at { color: var(--muted-foreground); font-size: var(--text-xs); margin-left: auto; }
      .content {
        font-size: var(--text-sm);
        color: var(--muted-foreground);
        line-height: var(--leading-normal);
        margin-top: 2px;
      }
      :host([interactive]) { cursor: pointer; border-radius: var(--radius-sm); outline: none; }
      :host([interactive]:hover) { background: var(--muted); }
      :host([interactive]:focus-visible) {
        box-shadow:
          0 0 0 var(--ring-offset) var(--background),
          0 0 0 calc(var(--ring-offset) + var(--ring-width))
            color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
      }
    `
];
ShipTimelineItem.properties = {
  tone: { type: String, reflect: true },
  at: { type: String, reflect: true },
  interactive: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-timeline")) customElements.define("ship-timeline", ShipTimeline);
  if (!customElements.get("ship-timeline-item")) customElements.define("ship-timeline-item", ShipTimelineItem);
}

// src/web-components/observability/ship-tree-view.ts
var ShipTreeView = class extends i4 {
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "tree");
  }
  render() {
    return b2`<slot></slot>`;
  }
};
ShipTreeView.styles = i`
    :host {
      display: block;
      font-family: var(--font-body);
      font-size: var(--text-sm);
      color: var(--foreground);
    }
  `;
var ShipTreeNode = class extends i4 {
  constructor() {
    super(...arguments);
    this.label = "";
    this.expanded = false;
    this.selected = false;
    this.leaf = false;
    this._toggle = (e6) => {
      if (this.leaf) return;
      e6.stopPropagation();
      this.expanded = !this.expanded;
      this.dispatchEvent(
        new CustomEvent("ship-toggle", {
          detail: { expanded: this.expanded },
          bubbles: true,
          composed: true
        })
      );
    };
    this._select = () => {
      this.dispatchEvent(new CustomEvent("ship-select", { bubbles: true, composed: true, detail: {} }));
    };
    this._onKey = (e6) => {
      if (e6.key === "Enter" || e6.key === " ") {
        e6.preventDefault();
        this._select();
      } else if (e6.key === "ArrowRight" && !this.leaf && !this.expanded) {
        e6.preventDefault();
        this._toggle(e6);
      } else if (e6.key === "ArrowLeft" && this.expanded) {
        e6.preventDefault();
        this._toggle(e6);
      }
    };
  }
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "treeitem");
  }
  render() {
    return b2`
      <div
        class="row"
        part="row"
        tabindex="0"
        aria-expanded=${this.leaf ? "false" : String(this.expanded)}
        aria-selected=${String(this.selected)}
        @click=${this._select}
        @keydown=${this._onKey}
      >
        <span class="caret" part="caret" @click=${this._toggle} aria-hidden="true">
          <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="4 3 8 6 4 9"></polyline></svg>
        </span>
        <slot name="icon"></slot>
        <span class="label" part="label">${this.label}</span>
        <span class="trailing"><slot name="trailing"></slot></span>
      </div>
      <div class="children" part="children" role="group"><slot></slot></div>
    `;
  }
};
ShipTreeNode.styles = i`
    :host { display: block; }
    .row {
      display: flex;
      align-items: center;
      gap: 4px;
      padding: 3px 6px;
      border-radius: var(--radius-sm);
      cursor: pointer;
      outline: none;
      color: var(--foreground);
      transition: background-color var(--duration-fast) var(--ease-out);
    }
    .row:hover { background: var(--muted); }
    :host([selected]) .row {
      background: color-mix(in oklch, var(--primary) 14%, transparent);
      color: var(--foreground);
      font-weight: var(--font-weight-medium);
    }
    .row:focus-visible {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    .caret {
      display: inline-flex;
      width: 12px; height: 12px;
      color: var(--muted-foreground);
      transition: transform var(--duration-fast) var(--ease-out);
    }
    :host([expanded]) .caret { transform: rotate(90deg); }
    :host([leaf]) .caret { visibility: hidden; }
    .label { flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .trailing { margin-left: auto; color: var(--muted-foreground); font-size: var(--text-xs); }
    .children { margin-left: 16px; border-left: 1px dashed var(--border); padding-left: 6px; }
    :host(:not([expanded])) .children { display: none; }
  `;
ShipTreeNode.properties = {
  label: { type: String, reflect: true },
  expanded: { type: Boolean, reflect: true },
  selected: { type: Boolean, reflect: true },
  leaf: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-tree-view")) customElements.define("ship-tree-view", ShipTreeView);
  if (!customElements.get("ship-tree-node")) customElements.define("ship-tree-node", ShipTreeNode);
}

// src/web-components/charts/util.ts
var TONE_VARS = {
  default: "var(--foreground)",
  primary: "var(--primary)",
  info: "var(--info)",
  success: "var(--success)",
  warning: "var(--warning)",
  destructive: "var(--destructive)",
  accent: "var(--accent)",
  muted: "var(--muted-foreground)"
};
function toneColor(tone) {
  if (!tone) return TONE_VARS.default;
  return TONE_VARS[tone] ?? TONE_VARS.default;
}
function parseNums(s4) {
  if (!s4) return [];
  return s4.split(",").map((p3) => Number.parseFloat(p3.trim())).filter((n4) => Number.isFinite(n4));
}
function parseLabels(s4) {
  if (!s4) return [];
  return s4.split(",").map((p3) => p3.trim());
}
function parseJson(s4, fallback) {
  if (!s4) return fallback;
  try {
    return JSON.parse(s4);
  } catch {
    return fallback;
  }
}
function linePath(data, w2, h3, pad = 2) {
  if (data.length === 0) return { d: "", points: [] };
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;
  const innerH = h3 - pad * 2;
  const step = data.length === 1 ? 0 : w2 / (data.length - 1);
  const points = data.map((v2, i6) => {
    const x2 = data.length === 1 ? w2 / 2 : i6 * step;
    const y3 = pad + innerH - (v2 - min) / range * innerH;
    return [x2, y3];
  });
  const d3 = points.map(([x2, y3], i6) => `${i6 === 0 ? "M" : "L"}${x2.toFixed(2)},${y3.toFixed(2)}`).join(" ");
  return { d: d3, points };
}
function polar(cx, cy, r4, angleRad) {
  return [cx + r4 * Math.cos(angleRad), cy + r4 * Math.sin(angleRad)];
}
function arcPath(cx, cy, r4, startAngle, endAngle) {
  const [x1, y1] = polar(cx, cy, r4, startAngle);
  const [x2, y22] = polar(cx, cy, r4, endAngle);
  const large = endAngle - startAngle > Math.PI ? 1 : 0;
  const sweep = endAngle > startAngle ? 1 : 0;
  return `M${x1.toFixed(2)},${y1.toFixed(2)} A${r4},${r4} 0 ${large} ${sweep} ${x2.toFixed(2)},${y22.toFixed(2)}`;
}

// src/web-components/charts/ship-sparkline.ts
var ShipSparkline = class extends i4 {
  constructor() {
    super(...arguments);
    this.data = "";
    this.tone = "primary";
    this.width = 80;
    this.height = 20;
    this.fill = false;
  }
  render() {
    const nums = parseNums(this.data);
    const w2 = this.width;
    const h3 = this.height;
    if (nums.length < 2) {
      return b2`<svg viewBox="0 0 ${w2} ${h3}" width=${w2} height=${h3} aria-hidden="true"></svg>`;
    }
    const color = toneColor(this.tone);
    const { d: d3, points } = linePath(nums, w2, h3, 2);
    const last = points[points.length - 1];
    const first = points[0];
    const areaD = this.fill ? `${d3} L${last[0].toFixed(2)},${h3} L${first[0].toFixed(2)},${h3} Z` : "";
    return b2`
      <svg
        viewBox="0 0 ${w2} ${h3}"
        width=${w2}
        height=${h3}
        role="img"
        aria-label="sparkline"
      >
        ${this.fill ? w`<path d=${areaD} fill=${color} fill-opacity="0.18" stroke="none"></path>` : null}
        <path
          d=${d3}
          fill="none"
          stroke=${color}
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        ></path>
      </svg>
    `;
  }
};
ShipSparkline.styles = i`
    :host {
      display: inline-block;
      width: var(--ship-sparkline-w, auto);
      height: var(--ship-sparkline-h, 40px);
      line-height: 0;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    svg { display: block; overflow: visible; }
  `;
ShipSparkline.properties = {
  data: { type: String, reflect: true },
  tone: { type: String, reflect: true },
  width: { type: Number },
  height: { type: Number },
  fill: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-sparkline")) {
  customElements.define("ship-sparkline", ShipSparkline);
}

// src/web-components/charts/ship-bar-chart.ts
var ShipBarChart = class extends i4 {
  constructor() {
    super(...arguments);
    this.data = "";
    this.labels = "";
    this.tone = "primary";
    this.orientation = "vertical";
    this.width = 300;
    this.height = 200;
    this.max = null;
  }
  render() {
    const nums = parseNums(this.data);
    const labels = parseLabels(this.labels);
    const w2 = this.width;
    const h3 = this.height;
    if (nums.length === 0) {
      return b2`<svg viewBox="0 0 ${w2} ${h3}" aria-hidden="true"></svg>`;
    }
    const color = toneColor(this.tone);
    const maxVal = this.max ?? Math.max(...nums, 0);
    const safeMax = maxVal <= 0 ? 1 : maxVal;
    const gap = 4;
    const radius = 3;
    const horizontal = this.orientation === "horizontal";
    const n4 = nums.length;
    const bars = nums.map((v2, i6) => {
      const ratio = Math.max(0, v2) / safeMax;
      if (horizontal) {
        const barH2 = (h3 - gap * (n4 - 1)) / n4;
        const y4 = i6 * (barH2 + gap);
        const barW2 = ratio * w2;
        return { x: 0, y: y4, w: barW2, h: barH2 };
      }
      const barW = (w2 - gap * (n4 - 1)) / n4;
      const x2 = i6 * (barW + gap);
      const barH = ratio * h3;
      const y3 = h3 - barH;
      return { x: x2, y: y3, w: barW, h: barH };
    });
    return b2`
      <svg viewBox="0 0 ${w2} ${h3}" preserveAspectRatio="none" role="img" aria-label="bar chart">
        ${bars.map(
      (b3, i6) => w`
          <rect
            x=${b3.x}
            y=${b3.y}
            width=${Math.max(0, b3.w)}
            height=${Math.max(0, b3.h)}
            rx=${radius}
            ry=${radius}
            fill=${color}
          ><title>${labels[i6] ?? String(nums[i6])}: ${nums[i6]}</title></rect>
        `
    )}
      </svg>
    `;
  }
};
ShipBarChart.styles = i`
    :host {
      display: block;
      width: 100%;
      height: 200px;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    svg { display: block; width: 100%; height: 100%; overflow: visible; }
  `;
ShipBarChart.properties = {
  data: { type: String, reflect: true },
  labels: { type: String },
  tone: { type: String, reflect: true },
  orientation: { type: String, reflect: true },
  width: { type: Number },
  height: { type: Number },
  max: { type: Number }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-bar-chart")) {
  customElements.define("ship-bar-chart", ShipBarChart);
}

// src/web-components/charts/ship-line-chart.ts
var ShipLineChart = class extends i4 {
  constructor() {
    super(...arguments);
    this.series = "";
    this.labels = "";
    this.width = 300;
    this.height = 200;
    this.points = false;
    this.grid = false;
  }
  render() {
    const allSeries = parseJson(this.series, []);
    const w2 = this.width;
    const h3 = this.height;
    const _labels = parseLabels(this.labels);
    const flat = allSeries.flatMap((s4) => s4.data);
    const min = flat.length ? Math.min(...flat) : 0;
    const max = flat.length ? Math.max(...flat) : 1;
    const range = max - min || 1;
    const innerH = h3 - 4;
    const gridLines = this.grid ? [0.25, 0.5, 0.75].map((f3) => 2 + innerH * f3) : [];
    return b2`
      <div class="wrap">
        <svg viewBox="0 0 ${w2} ${h3}" preserveAspectRatio="none" role="img" aria-label="line chart">
          ${gridLines.map(
      (y3) => w`<line class="grid-line" x1="0" x2=${w2} y1=${y3.toFixed(2)} y2=${y3.toFixed(2)}></line>`
    )}
          ${allSeries.map((s4) => {
      const color = toneColor(s4.tone);
      if (s4.data.length < 2) return null;
      const normalized = s4.data.map((v2) => (v2 - min) / range * 100);
      const { d: d3, points } = linePath(normalized, w2, h3, 2);
      return w`
              <path d=${d3} fill="none" stroke=${color} stroke-width="1.75"
                stroke-linecap="round" stroke-linejoin="round"></path>
              ${this.points ? points.map(
        ([x2, y3]) => w`<circle cx=${x2.toFixed(2)} cy=${y3.toFixed(2)} r="2.5" fill=${color}></circle>`
      ) : null}
            `;
    })}
        </svg>
        <div class="legend"><slot></slot></div>
      </div>
    `;
  }
};
ShipLineChart.styles = i`
    :host {
      display: block;
      width: 100%;
      height: 200px;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    .wrap { display: flex; flex-direction: column; gap: 6px; width: 100%; height: 100%; }
    svg { display: block; width: 100%; flex: 1; overflow: visible; }
    .legend {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
      font-family: var(--font-body);
    }
    .legend:empty { display: none; }
    .grid-line { stroke: var(--border); stroke-width: 1; stroke-dasharray: 2 3; }
  `;
ShipLineChart.properties = {
  series: { type: String, reflect: true },
  labels: { type: String },
  width: { type: Number },
  height: { type: Number },
  points: { type: Boolean, reflect: true },
  grid: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-line-chart")) {
  customElements.define("ship-line-chart", ShipLineChart);
}

// src/web-components/charts/ship-area-chart.ts
var ShipAreaChart = class extends i4 {
  constructor() {
    super(...arguments);
    this.series = "";
    this.labels = "";
    this.width = 300;
    this.height = 200;
    this.points = false;
    this.grid = false;
  }
  render() {
    const allSeries = parseJson(this.series, []);
    const w2 = this.width;
    const h3 = this.height;
    const flat = allSeries.flatMap((s4) => s4.data);
    const min = flat.length ? Math.min(...flat) : 0;
    const max = flat.length ? Math.max(...flat) : 1;
    const range = max - min || 1;
    const innerH = h3 - 4;
    const gridLines = this.grid ? [0.25, 0.5, 0.75].map((f3) => 2 + innerH * f3) : [];
    return b2`
      <div class="wrap">
        <svg viewBox="0 0 ${w2} ${h3}" preserveAspectRatio="none" role="img" aria-label="area chart">
          ${gridLines.map(
      (y3) => w`<line class="grid-line" x1="0" x2=${w2} y1=${y3.toFixed(2)} y2=${y3.toFixed(2)}></line>`
    )}
          ${allSeries.map((s4) => {
      const color = toneColor(s4.tone);
      if (s4.data.length < 2) return null;
      const normalized = s4.data.map((v2) => (v2 - min) / range * 100);
      const { d: d3, points } = linePath(normalized, w2, h3, 2);
      const first = points[0];
      const last = points[points.length - 1];
      const areaD = `${d3} L${last[0].toFixed(2)},${h3} L${first[0].toFixed(2)},${h3} Z`;
      return w`
              <path d=${areaD} fill=${color} fill-opacity="0.3" stroke="none"></path>
              <path d=${d3} fill="none" stroke=${color} stroke-width="1.75"
                stroke-linecap="round" stroke-linejoin="round"></path>
              ${this.points ? points.map(
        ([x2, y3]) => w`<circle cx=${x2.toFixed(2)} cy=${y3.toFixed(2)} r="2.5" fill=${color}></circle>`
      ) : null}
            `;
    })}
        </svg>
        <div class="legend"><slot></slot></div>
      </div>
    `;
  }
};
ShipAreaChart.styles = i`
    :host {
      display: block;
      width: 100%;
      height: 200px;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    .wrap { display: flex; flex-direction: column; gap: 6px; width: 100%; height: 100%; }
    svg { display: block; width: 100%; flex: 1; overflow: visible; }
    .legend {
      display: flex; flex-wrap: wrap; gap: 12px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
      font-family: var(--font-body);
    }
    .legend:empty { display: none; }
    .grid-line { stroke: var(--border); stroke-width: 1; stroke-dasharray: 2 3; }
  `;
ShipAreaChart.properties = {
  series: { type: String, reflect: true },
  labels: { type: String },
  width: { type: Number },
  height: { type: Number },
  points: { type: Boolean, reflect: true },
  grid: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-area-chart")) {
  customElements.define("ship-area-chart", ShipAreaChart);
}

// src/web-components/charts/ship-donut.ts
var ShipDonut = class extends i4 {
  constructor() {
    super(...arguments);
    this.data = "";
    this.size = 120;
    this.thickness = 16;
  }
  updated() {
    this.style.setProperty("--ship-donut-size", `${this.size}px`);
  }
  render() {
    const segs = parseJson(this.data, []);
    const s4 = this.size;
    const t4 = this.thickness;
    const r4 = (s4 - t4) / 2;
    const c4 = s4 / 2;
    const circumference = 2 * Math.PI * r4;
    const total = segs.reduce((acc, x2) => acc + Math.max(0, x2.value), 0);
    let offset = 0;
    return b2`
      <svg viewBox="0 0 ${s4} ${s4}" role="img" aria-label="donut chart">
        <circle class="track" cx=${c4} cy=${c4} r=${r4} fill="none" stroke-width=${t4}></circle>
        ${total > 0 ? segs.map((seg) => {
      const len = Math.max(0, seg.value) / total * circumference;
      const dash = `${len.toFixed(2)} ${(circumference - len).toFixed(2)}`;
      const node = w`
                <circle
                  cx=${c4} cy=${c4} r=${r4} fill="none"
                  stroke=${toneColor(seg.tone)}
                  stroke-width=${t4}
                  stroke-dasharray=${dash}
                  stroke-dashoffset=${(-offset).toFixed(2)}
                  stroke-linecap="butt"
                ><title>${seg.label}: ${seg.value}</title></circle>
              `;
      offset += len;
      return node;
    }) : null}
      </svg>
      <div class="center"><slot></slot></div>
    `;
  }
};
ShipDonut.styles = i`
    :host {
      display: inline-block;
      width: var(--ship-donut-size, 120px);
      height: var(--ship-donut-size, 120px);
      position: relative;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    svg { display: block; width: 100%; height: 100%; transform: rotate(-90deg); }
    .center {
      position: absolute;
      inset: 0;
      display: flex;
      align-items: center;
      justify-content: center;
      font-family: var(--font-display);
      font-weight: var(--font-weight-semibold);
      font-size: var(--text-lg);
      color: var(--foreground);
      line-height: var(--leading-tight);
      pointer-events: none;
    }
    .track { stroke: var(--muted); }
  `;
ShipDonut.properties = {
  data: { type: String, reflect: true },
  size: { type: Number },
  thickness: { type: Number }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-donut")) {
  customElements.define("ship-donut", ShipDonut);
}

// src/web-components/charts/ship-gauge.ts
var ShipGauge = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = 0;
    this.min = 0;
    this.max = 100;
    this.tone = null;
    this.lowThreshold = null;
    this.highThreshold = null;
    this.size = 140;
    this.thickness = 12;
  }
  updated() {
    this.style.setProperty("--ship-gauge-size", `${this.size}px`);
  }
  resolveTone() {
    if (this.tone) return this.tone;
    if (this.lowThreshold != null && this.value < this.lowThreshold) return "destructive";
    if (this.highThreshold != null && this.value < this.highThreshold) return "warning";
    if (this.lowThreshold != null || this.highThreshold != null) return "success";
    return "primary";
  }
  render() {
    const s4 = this.size;
    const t4 = this.thickness;
    const r4 = (s4 - t4) / 2 - 2;
    const c4 = s4 / 2;
    const start = 135 * Math.PI / 180;
    const end = 405 * Math.PI / 180;
    const fullSweep = end - start;
    const ratio = Math.max(
      0,
      Math.min(1, (this.value - this.min) / (this.max - this.min || 1))
    );
    const valueEnd = start + fullSweep * ratio;
    const trackD = arcPath(c4, c4, r4, start, end);
    const valueD = ratio > 0 ? arcPath(c4, c4, r4, start, valueEnd) : "";
    const color = toneColor(this.resolveTone());
    return b2`
      <svg viewBox="0 0 ${s4} ${s4}" role="img" aria-label="gauge">
        <path class="track" d=${trackD} fill="none" stroke-width=${t4} stroke-linecap="round"></path>
        ${valueD ? w`<path d=${valueD} fill="none" stroke=${color} stroke-width=${t4} stroke-linecap="round"></path>` : null}
      </svg>
      <div class="label"><slot></slot></div>
    `;
  }
};
ShipGauge.styles = i`
    :host {
      display: inline-block;
      width: var(--ship-gauge-size, 140px);
      height: var(--ship-gauge-size, 140px);
      position: relative;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    svg { display: block; width: 100%; height: 100%; }
    .label {
      position: absolute; inset: 0;
      display: flex; align-items: center; justify-content: center;
      font-family: var(--font-display);
      font-weight: var(--font-weight-semibold);
      font-size: var(--text-xl);
      color: var(--foreground);
    }
    .track { stroke: var(--muted); }
  `;
ShipGauge.properties = {
  value: { type: Number, reflect: true },
  min: { type: Number },
  max: { type: Number },
  tone: { type: String, reflect: true },
  lowThreshold: { type: Number, attribute: "low-threshold" },
  highThreshold: { type: Number, attribute: "high-threshold" },
  size: { type: Number },
  thickness: { type: Number }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-gauge")) {
  customElements.define("ship-gauge", ShipGauge);
}

// src/web-components/charts/ship-progress-ring.ts
var ShipProgressRing = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = 0;
    this.size = 48;
    this.thickness = 4;
    this.tone = "primary";
  }
  updated() {
    this.style.setProperty("--ship-ring-size", `${this.size}px`);
  }
  render() {
    const s4 = this.size;
    const t4 = this.thickness;
    const r4 = (s4 - t4) / 2;
    const c4 = s4 / 2;
    const circ = 2 * Math.PI * r4;
    const pct = Math.max(0, Math.min(100, this.value));
    const offset = circ - pct / 100 * circ;
    const color = toneColor(this.tone);
    return b2`
      <svg viewBox="0 0 ${s4} ${s4}" role="progressbar"
        aria-valuemin="0" aria-valuemax="100" aria-valuenow=${pct}>
        <circle class="track" cx=${c4} cy=${c4} r=${r4}
          fill="none" stroke-width=${t4}></circle>
        <circle class="bar" cx=${c4} cy=${c4} r=${r4}
          fill="none" stroke=${color} stroke-width=${t4}
          stroke-linecap="round"
          stroke-dasharray=${circ.toFixed(2)}
          stroke-dashoffset=${offset.toFixed(2)}></circle>
      </svg>
      <div class="label"><slot></slot></div>
    `;
  }
};
ShipProgressRing.styles = i`
    :host {
      display: inline-block;
      width: var(--ship-ring-size, 48px);
      height: var(--ship-ring-size, 48px);
      position: relative;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    svg { display: block; width: 100%; height: 100%; transform: rotate(-90deg); }
    .label {
      position: absolute; inset: 0;
      display: flex; align-items: center; justify-content: center;
      font-size: var(--text-xs);
      font-weight: var(--font-weight-semibold);
      font-family: var(--font-body);
      color: var(--foreground);
      font-variant-numeric: tabular-nums;
    }
    .track { stroke: var(--muted); }
    .bar { transition: stroke-dashoffset var(--duration-medium) var(--ease-out); }
  `;
ShipProgressRing.properties = {
  value: { type: Number, reflect: true },
  size: { type: Number },
  thickness: { type: Number },
  tone: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-progress-ring")) {
  customElements.define("ship-progress-ring", ShipProgressRing);
}

// src/web-components/charts/ship-heatmap.ts
var TONE_CSS_VAR = {
  default: "--foreground",
  primary: "--primary",
  info: "--info",
  success: "--success",
  warning: "--warning",
  destructive: "--destructive",
  accent: "--accent",
  muted: "--muted-foreground"
};
var ShipHeatmap = class extends i4 {
  constructor() {
    super(...arguments);
    this.data = "";
    this.rows = "";
    this.cols = "";
    this.tone = "primary";
  }
  render() {
    const rows = parseLabels(this.rows);
    const cols = parseLabels(this.cols);
    const cells = parseJson(this.data, []);
    const toneVar = TONE_CSS_VAR[this.tone] ?? TONE_CSS_VAR.primary;
    const lookup = /* @__PURE__ */ new Map();
    for (const c4 of cells) lookup.set(`${c4.y}\0${c4.x}`, c4.value);
    const gridStyle = `grid-template-columns: auto repeat(${cols.length}, 1fr);`;
    return b2`
      <div class="grid" style=${gridStyle} role="table" aria-label="heatmap">
        <div></div>
        ${cols.map((c4) => b2`<div class="col-label">${c4}</div>`)}
        ${rows.map(
      (r4) => b2`
            <div class="row-label">${r4}</div>
            ${cols.map((c4) => {
        const v2 = Math.max(0, Math.min(1, lookup.get(`${r4}\0${c4}`) ?? 0));
        const pct = (v2 * 100).toFixed(1);
        const style = v2 > 0 ? `background: color-mix(in oklch, var(${toneVar}) ${pct}%, transparent);` : "";
        return b2`<div class="cell" style=${style} title="${r4} / ${c4}: ${v2}"></div>`;
      })}
          `
    )}
      </div>
    `;
  }
};
ShipHeatmap.styles = i`
    :host {
      display: block;
      width: 100%;
      font-family: var(--font-body);
    }
    .grid {
      display: grid;
      gap: 3px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    .cell {
      aspect-ratio: 1 / 1;
      border-radius: var(--radius-sm);
      background: var(--muted);
    }
    .col-label, .row-label {
      font-size: 10px;
      color: var(--muted-foreground);
      font-weight: var(--font-weight-medium);
      display: flex;
      align-items: center;
    }
    .col-label { justify-content: center; }
    .row-label { justify-content: flex-end; padding-right: 6px; }
  `;
ShipHeatmap.properties = {
  data: { type: String, reflect: true },
  rows: { type: String },
  cols: { type: String },
  tone: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-heatmap")) {
  customElements.define("ship-heatmap", ShipHeatmap);
}

// src/web-components/charts/ship-meter.ts
var ShipMeter = class extends i4 {
  constructor() {
    super(...arguments);
    this.segments = "";
    this.total = null;
  }
  render() {
    const segs = parseJson(this.segments, []);
    const sum = segs.reduce((a3, s4) => a3 + Math.max(0, s4.count), 0);
    const denom = this.total ?? sum;
    const safeDenom = denom <= 0 ? 1 : denom;
    return b2`
      <div class="wrap">
        <div class="bar" role="img" aria-label="meter">
          ${segs.map((s4) => {
      const pct = Math.max(0, s4.count) / safeDenom * 100;
      if (pct <= 0) return null;
      const style = `width: ${pct.toFixed(3)}%; background: ${toneColor(s4.tone)};`;
      return b2`<div class="seg" style=${style} title="${s4.label ?? s4.tone}: ${s4.count}"></div>`;
    })}
        </div>
        <div class="legend"><slot></slot></div>
      </div>
    `;
  }
};
ShipMeter.styles = i`
    :host {
      display: block;
      width: 100%;
      font-family: var(--font-body);
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    .wrap { display: flex; flex-direction: column; gap: 8px; }
    .bar {
      display: flex;
      gap: 2px;
      height: 10px;
      width: 100%;
      background: var(--muted);
      border-radius: var(--radius-full);
      overflow: hidden;
      padding: 0;
    }
    .seg:first-child {
      border-top-left-radius: var(--radius-full);
      border-bottom-left-radius: var(--radius-full);
    }
    .seg:last-child {
      border-top-right-radius: var(--radius-full);
      border-bottom-right-radius: var(--radius-full);
    }
    .legend {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    .legend:empty { display: none; }
  `;
ShipMeter.properties = {
  segments: { type: String, reflect: true },
  total: { type: Number }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-meter")) {
  customElements.define("ship-meter", ShipMeter);
}

// src/web-components/utility/ship-avatar.ts
function initialsFromName(name) {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "";
  if (parts.length === 1) return parts[0].charAt(0).toUpperCase();
  return (parts[0].charAt(0) + parts[parts.length - 1].charAt(0)).toUpperCase();
}
var ShipAvatar = class extends i4 {
  constructor() {
    super(...arguments);
    this.src = "";
    this.name = "";
    this.size = "md";
    this.tone = "muted";
    this._errored = false;
    this.onError = () => {
      this._errored = true;
    };
  }
  updated(changed) {
    if (changed.has("src")) this._errored = false;
  }
  render() {
    const showImage = this.src && !this._errored;
    const initials = initialsFromName(this.name);
    return b2`
      <span class="wrap" role="img" aria-label=${this.name || "avatar"} part="wrap">
        ${showImage ? b2`<img src=${this.src} alt=${this.name} @error=${this.onError} />` : b2`<span part="initials">${initials}</span>`}
      </span>
    `;
  }
};
ShipAvatar.styles = [
  toneVars,
  i`
      :host {
        display: inline-flex;
        width: 32px; height: 32px;
        border-radius: var(--radius-full);
        background: var(--tone-bg);
        color: var(--tone-fg);
        font-family: var(--font-body);
        font-weight: var(--font-weight-semibold);
        font-size: var(--text-sm);
        overflow: hidden;
        user-select: none;
      }
      :host([size='sm']) { width: 24px; height: 24px; font-size: var(--text-xs); }
      :host([size='lg']) { width: 40px; height: 40px; font-size: var(--text-md); }
      :host([size='xl']) { width: 56px; height: 56px; font-size: var(--text-lg); }
      .wrap {
        width: 100%; height: 100%;
        display: inline-flex; align-items: center; justify-content: center;
      }
      img { width: 100%; height: 100%; object-fit: cover; display: block; }
    `
];
ShipAvatar.properties = {
  src: { type: String },
  name: { type: String },
  size: { type: String, reflect: true },
  tone: { type: String, reflect: true },
  _errored: { state: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-avatar")) {
  customElements.define("ship-avatar", ShipAvatar);
}

// src/web-components/utility/ship-chip.ts
var ShipChip = class extends i4 {
  constructor() {
    super(...arguments);
    this.tone = "muted";
    this.size = "md";
    this.removable = false;
    this.selected = false;
  }
  onRemove(e6) {
    e6.stopPropagation();
    this.dispatchEvent(new CustomEvent("ship-remove", {
      detail: {},
      bubbles: true,
      composed: true
    }));
  }
  render() {
    return b2`
      <span class="chip" part="chip">
        <slot name="leading"></slot>
        <slot></slot>
        ${this.removable ? b2`<button class="remove" part="remove" aria-label="Remove" @click=${this.onRemove}>×</button>` : null}
      </span>
    `;
  }
};
ShipChip.styles = [
  toneVars,
  i`
      :host { display: inline-flex; vertical-align: middle; }
      .chip {
        display: inline-flex; align-items: center; gap: 6px;
        padding: 4px 10px;
        font-family: var(--font-body);
        font-size: var(--text-sm);
        font-weight: var(--font-weight-medium);
        line-height: var(--leading-tight);
        border-radius: var(--radius-full);
        background: var(--tone-bg);
        color: var(--tone-fg);
        border: 1px solid var(--tone-border);
        white-space: nowrap;
      }
      :host([size='sm']) .chip { padding: 2px 8px; font-size: var(--text-xs); gap: 4px; }
      :host([selected]) .chip {
        background: color-mix(in oklch, var(--tone-fg) 22%, transparent);
        border-color: var(--tone-fg);
      }
      .remove {
        display: inline-flex; align-items: center; justify-content: center;
        width: 16px; height: 16px;
        border: 0; padding: 0; margin-right: -4px;
        background: transparent; color: inherit; cursor: pointer;
        border-radius: var(--radius-full);
        font-size: 14px; line-height: 1;
      }
      .remove:hover { background: color-mix(in oklch, var(--tone-fg) 18%, transparent); }
      .remove:focus-visible {
        outline: var(--ring-width) solid var(--ring-color);
        outline-offset: var(--ring-offset);
      }
      ::slotted([slot='leading']) { display: inline-flex; }
    `
];
ShipChip.properties = {
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true },
  removable: { type: Boolean, reflect: true },
  selected: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-chip")) {
  customElements.define("ship-chip", ShipChip);
}

// src/web-components/utility/ship-kbd.ts
var ShipKbd = class extends i4 {
  constructor() {
    super(...arguments);
    this.keys = "";
  }
  render() {
    const tokens = this.keys.split("+").map((s4) => s4.trim()).filter(Boolean);
    return b2`${tokens.map((t4, i6) => b2`
      ${i6 > 0 ? b2`<span class="plus" aria-hidden="true">+</span>` : null}
      <kbd part="key">${t4}</kbd>
    `)}`;
  }
};
ShipKbd.styles = i`
    :host { display: inline-flex; align-items: center; gap: 3px; vertical-align: middle; }
    kbd {
      display: inline-flex; align-items: center; justify-content: center;
      min-width: 18px;
      padding: 1px 5px;
      font-family: ${MONO};
      font-size: var(--text-xs);
      font-weight: var(--font-weight-medium);
      line-height: 1.4;
      color: var(--foreground);
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: var(--radius-sm);
      box-shadow: var(--shadow-xs);
    }
    .plus { color: var(--muted-foreground); font-size: var(--text-xs); }
  `;
ShipKbd.properties = {
  keys: { type: String }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-kbd")) {
  customElements.define("ship-kbd", ShipKbd);
}

// src/web-components/utility/ship-skeleton.ts
var ShipSkeleton = class extends i4 {
  constructor() {
    super(...arguments);
    this.variant = "text";
    this.width = "";
    this.height = "";
    this.lines = 1;
  }
  styleFor() {
    const parts = [];
    if (this.width) parts.push(`width:${this.width}`);
    if (this.height) parts.push(`height:${this.height}`);
    return parts.join(";");
  }
  render() {
    const count = this.variant === "text" ? Math.max(1, this.lines || 1) : 1;
    const s4 = this.styleFor();
    return b2`${Array.from({ length: count }).map(() => b2`
      <span class="bar" part="bar" style=${s4}></span>
    `)}`;
  }
};
ShipSkeleton.styles = i`
    :host { display: block; }
    .bar {
      display: block;
      background: linear-gradient(
        90deg,
        var(--muted) 0%,
        color-mix(in oklch, var(--muted-foreground) 14%, var(--muted)) 50%,
        var(--muted) 100%
      );
      background-size: 200% 100%;
      animation: shimmer var(--duration-slow) linear infinite;
      border-radius: var(--radius-sm);
    }
    :host([variant='circle']) .bar { border-radius: var(--radius-full); }
    :host([variant='rect']) .bar { border-radius: var(--radius-md); }
    :host([variant='text']) .bar { height: 0.9em; }
    :host([variant='text']) .bar + .bar { margin-top: 6px; }
    :host([variant='text']) .bar:last-child { width: 65%; }
    :host([variant='text']) .bar:only-child { width: 100%; }
    @keyframes shimmer {
      0% { background-position: 200% 0; }
      100% { background-position: -200% 0; }
    }
  `;
ShipSkeleton.properties = {
  variant: { type: String, reflect: true },
  width: { type: String },
  height: { type: String },
  lines: { type: Number }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-skeleton")) {
  customElements.define("ship-skeleton", ShipSkeleton);
}

// src/web-components/utility/ship-toast.ts
var ShipToastContainer = class extends i4 {
  constructor() {
    super(...arguments);
    this.position = "top-right";
  }
  render() {
    return b2`<slot></slot>`;
  }
};
ShipToastContainer.styles = i`
    :host {
      position: fixed;
      top: 16px; right: 16px;
      display: flex; flex-direction: column; gap: 8px;
      z-index: var(--z-toast);
      pointer-events: none;
      max-width: min(420px, calc(100vw - 32px));
    }
    :host([position='bottom-right']) { top: auto; bottom: 16px; right: 16px; flex-direction: column-reverse; }
    :host([position='bottom-left']) { top: auto; bottom: 16px; left: 16px; right: auto; flex-direction: column-reverse; }
    :host([position='top-left']) { left: 16px; right: auto; }
    ::slotted(ship-toast) { pointer-events: auto; }
  `;
ShipToastContainer.properties = { position: { type: String, reflect: true } };
var ShipToast = class extends i4 {
  constructor() {
    super(...arguments);
    this.tone = "info";
    this.duration = 5e3;
    this.dismissible = true;
    this._timer = null;
  }
  connectedCallback() {
    super.connectedCallback();
    if (this.duration > 0) {
      this._timer = setTimeout(() => this.dismiss(), this.duration);
    }
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    if (this._timer) clearTimeout(this._timer);
  }
  dismiss() {
    this.dispatchEvent(new CustomEvent("ship-dismiss", {
      bubbles: true,
      composed: true
    }));
    this.remove();
  }
  render() {
    return b2`
      <div class="toast" role="status" part="toast">
        <span><slot name="icon"></slot></span>
        <div class="content">
          <div class="title"><slot name="title"></slot></div>
          <div><slot></slot></div>
          <div><slot name="actions"></slot></div>
        </div>
        ${this.dismissible ? b2`<button class="close" aria-label="Dismiss" @click=${this.dismiss}>×</button>` : b2`<span></span>`}
      </div>
    `;
  }
};
ShipToast.styles = [
  toneVars,
  i`
      :host {
        display: block;
        animation: slide-in var(--duration-medium) var(--ease-out);
      }
      .toast {
        display: grid;
        grid-template-columns: auto 1fr auto;
        gap: 10px;
        align-items: start;
        padding: 12px 14px;
        border-radius: var(--radius);
        border: 1px solid var(--tone-border);
        background: var(--tone-bg);
        color: var(--tone-fg);
        box-shadow: var(--shadow-md);
        font-family: var(--font-body);
        font-size: var(--text-base);
        line-height: var(--leading-normal);
      }
      .content { display: flex; flex-direction: column; gap: 4px; min-width: 0; color: var(--foreground); }
      .title { font-weight: var(--font-weight-semibold); color: var(--tone-fg); }
      .close {
        background: transparent; border: 0; color: inherit;
        cursor: pointer; font-size: 14px; padding: 2px 6px;
        border-radius: var(--radius-sm);
      }
      .close:hover { background: color-mix(in oklch, var(--tone-fg) 14%, transparent); }
      .close:focus-visible {
        outline: var(--ring-width) solid var(--ring-color);
        outline-offset: var(--ring-offset);
      }
      @keyframes slide-in {
        from { opacity: 0; transform: translateY(-8px); }
        to   { opacity: 1; transform: translateY(0); }
      }
    `
];
ShipToast.properties = {
  tone: { type: String, reflect: true },
  duration: { type: Number },
  dismissible: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-toast-container"))
    customElements.define("ship-toast-container", ShipToastContainer);
  if (!customElements.get("ship-toast"))
    customElements.define("ship-toast", ShipToast);
}

// src/web-components/utility/ship-breadcrumb.ts
var ShipBreadcrumbItem = class extends i4 {
  constructor() {
    super(...arguments);
    this.href = "";
    this.active = false;
  }
  render() {
    if (this.href && !this.active) {
      return b2`<a href=${this.href} part="link"><slot></slot></a>`;
    }
    return b2`<span class="label" part="label" aria-current=${this.active ? "page" : "false"}><slot></slot></span>`;
  }
};
ShipBreadcrumbItem.styles = i`
    :host {
      display: inline-flex; align-items: center;
      font-family: var(--font-body);
      font-size: var(--text-base);
      line-height: var(--leading-normal);
      color: var(--muted-foreground);
    }
    :host([active]) { color: var(--foreground); font-weight: var(--font-weight-medium); }
    a, .label {
      display: inline-flex; align-items: center;
      color: inherit;
      text-decoration: none;
      padding: 2px 4px;
      border-radius: var(--radius-sm);
      transition: color var(--duration-fast);
    }
    a:hover { color: var(--foreground); }
    a:focus-visible {
      outline: var(--ring-width) solid var(--ring-color);
      outline-offset: var(--ring-offset);
    }
  `;
ShipBreadcrumbItem.properties = {
  href: { type: String },
  active: { type: Boolean, reflect: true }
};
var ShipBreadcrumb = class extends i4 {
  constructor() {
    super(...arguments);
    this._observer = null;
  }
  get items() {
    return Array.from(this.querySelectorAll(":scope > ship-breadcrumb-item"));
  }
  connectedCallback() {
    super.connectedCallback();
    this._observer = new MutationObserver(() => this.requestUpdate());
    this._observer.observe(this, { childList: true });
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this._observer?.disconnect();
  }
  render() {
    const count = this.items.length;
    const slots = [];
    for (let i6 = 0; i6 < count; i6++) {
      slots.push(b2`<slot name="i${i6}"></slot>`);
      if (i6 < count - 1) {
        slots.push(b2`<span class="sep" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
               stroke-linecap="round" stroke-linejoin="round"><path d="M9 6l6 6-6 6"/></svg>
        </span>`);
      }
    }
    this.items.forEach((el, i6) => el.setAttribute("slot", `i${i6}`));
    return b2`<nav aria-label="Breadcrumb" part="nav">${slots}</nav>`;
  }
};
ShipBreadcrumb.styles = i`
    :host { display: flex; align-items: center; }
    nav { display: inline-flex; align-items: center; flex-wrap: wrap; gap: 2px; }
    .sep {
      display: inline-flex; align-items: center; justify-content: center;
      width: 12px; height: 12px;
      color: var(--muted-foreground);
      opacity: 0.7;
    }
    .sep svg { width: 12px; height: 12px; }
  `;
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-breadcrumb-item"))
    customElements.define("ship-breadcrumb-item", ShipBreadcrumbItem);
  if (!customElements.get("ship-breadcrumb"))
    customElements.define("ship-breadcrumb", ShipBreadcrumb);
}

// src/web-components/utility/ship-link.ts
var ShipLink = class extends i4 {
  constructor() {
    super(...arguments);
    this.href = "";
    this.external = false;
    this.tone = "default";
  }
  render() {
    const target = this.external ? "_blank" : void 0;
    const rel = this.external ? "noopener noreferrer" : void 0;
    return b2`
      <a href=${this.href}
         part="anchor"
         target=${target ?? ""}
         rel=${rel ?? ""}>
        <slot></slot>
        ${this.external ? b2`<span class="ext" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
               stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3"/>
          </svg>
        </span>` : null}
      </a>
    `;
  }
};
ShipLink.styles = i`
    :host {
      display: inline;
      font-family: var(--font-body);
      color: var(--primary);
    }
    :host([tone='muted']) { color: var(--muted-foreground); }
    a {
      color: inherit;
      text-decoration: none;
      border-radius: var(--radius-sm);
      display: inline;
      transition: color var(--duration-fast);
    }
    a:hover { text-decoration: underline; text-underline-offset: 2px; }
    a:focus-visible {
      outline: var(--ring-width) solid var(--ring-color);
      outline-offset: var(--ring-offset);
    }
    .ext {
      display: inline-block;
      margin-left: 2px;
      font-size: 0.9em;
      opacity: 0.8;
    }
    .ext svg { width: 0.9em; height: 0.9em; vertical-align: -0.1em; }
  `;
ShipLink.properties = {
  href: { type: String },
  external: { type: Boolean, reflect: true },
  tone: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-link")) {
  customElements.define("ship-link", ShipLink);
}

// src/web-components/utility/ship-metachip.ts
var ShipMetachip = class extends i4 {
  constructor() {
    super(...arguments);
    this.label = "";
    this.value = "";
  }
  render() {
    return b2`
      <span class="k" part="label"><slot name="label">${this.label}</slot></span>
      <span class="v" part="value"><slot>${this.value}</slot></span>
    `;
  }
};
ShipMetachip.styles = i`
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
  `;
ShipMetachip.properties = {
  label: { type: String, reflect: true },
  value: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-metachip")) {
  customElements.define("ship-metachip", ShipMetachip);
}

// src/web-components/utility/ship-step-segments.ts
var KNOWN = /* @__PURE__ */ new Set(["done", "failed", "pending", "running"]);
var ShipStepSegments = class extends i4 {
  constructor() {
    super(...arguments);
    this.values = "";
    this.size = "sm";
  }
  parsed() {
    return this.values.split(",").map((s4) => s4.trim().toLowerCase()).filter((s4) => s4.length > 0).map((s4) => KNOWN.has(s4) ? s4 : "pending");
  }
  render() {
    const segs = this.parsed();
    if (segs.length === 0) return b2``;
    return b2`
      <span class="row" part="row" role="img"
            aria-label="${segs.length} step outcome${segs.length === 1 ? "" : "s"}">
        ${segs.map(
      (s4) => b2`<span class="seg ${s4}" part="seg" data-state=${s4}></span>`
    )}
      </span>
    `;
  }
};
ShipStepSegments.styles = i`
    :host { display: inline-flex; vertical-align: middle; }
    .row { display: inline-flex; gap: 3px; align-items: center; }
    .seg {
      display: inline-block;
      width: 18px; height: 6px;
      border-radius: 2px;
      background: color-mix(in oklch, var(--muted-foreground) 28%, transparent);
    }
    :host([size='md']) .seg { width: 24px; height: 8px; border-radius: 3px; }
    .seg.done { background: var(--success); }
    .seg.failed { background: var(--destructive); }
    .seg.running {
      background: var(--warning);
      animation: pulse var(--duration-slow, 1200ms) ease-in-out infinite;
    }
    @keyframes pulse {
      0%, 100% { opacity: 1; }
      50% { opacity: 0.55; }
    }
  `;
ShipStepSegments.properties = {
  values: { type: String, reflect: true },
  size: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-step-segments")) {
  customElements.define("ship-step-segments", ShipStepSegments);
}

// src/web-components/index.ts
function registerShipComponents() {
}
export {
  ICONS,
  ShipAlert,
  ShipAlertDialog,
  ShipAreaChart,
  ShipAvatar,
  ShipBadge,
  ShipBarChart,
  ShipBreadcrumb,
  ShipBreadcrumbItem,
  ShipButton,
  ShipCard,
  ShipCheckbox,
  ShipChip,
  ShipCode,
  ShipCopyButton,
  ShipDialog,
  ShipDiff,
  ShipDonut,
  ShipDropdownMenu,
  ShipEmptyState,
  ShipEventRow,
  ShipField,
  ShipGauge,
  ShipGitDiff,
  ShipGitDiffFile,
  ShipGitDiffHunk,
  ShipGitDiffLine,
  ShipHeatmap,
  ShipHistoryRow,
  ShipIcon,
  ShipInput,
  ShipJsonView,
  ShipKbd,
  ShipLabel,
  ShipLineChart,
  ShipLink,
  ShipListItem,
  ShipLogLine,
  ShipMarkdown,
  ShipMetachip,
  ShipMeter,
  ShipPipelineDag,
  ShipPipelineDefinition,
  ShipPipelineRun,
  ShipPipelineStep,
  ShipPopover,
  ShipProgress,
  ShipProgressRing,
  ShipSelect,
  ShipSeparator,
  ShipSkeleton,
  ShipSparkline,
  ShipSpecRow,
  ShipSpinner,
  ShipStat,
  ShipStatusDot,
  ShipStepSegments,
  ShipSwitch,
  ShipTab,
  ShipTabPanel,
  ShipTable,
  ShipTabs,
  ShipTextarea,
  ShipThemeToggle,
  ShipTimeline,
  ShipTimelineItem,
  ShipTimestamp,
  ShipToast,
  ShipToastContainer,
  ShipToolbar,
  ShipTooltip,
  ShipTreeNode,
  ShipTreeView,
  formatRelative,
  initialsFromName,
  registerShipComponents,
  renderMarkdown
};
/*! Bundled license information:

@lit/reactive-element/css-tag.js:
  (**
   * @license
   * Copyright 2019 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

@lit/reactive-element/reactive-element.js:
lit-html/lit-html.js:
lit-element/lit-element.js:
lit-html/directive.js:
lit-html/directives/unsafe-html.js:
  (**
   * @license
   * Copyright 2017 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)

lit-html/is-server.js:
  (**
   * @license
   * Copyright 2022 Google LLC
   * SPDX-License-Identifier: BSD-3-Clause
   *)
*/
