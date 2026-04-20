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
function defineOnce(name, ctor) {
  if (typeof customElements === "undefined") return;
  if (!customElements.get(name)) customElements.define(name, ctor);
}
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
var focusRingStyle = r(`
  outline: none;
  box-shadow:
    0 0 0 var(--ring-offset) var(--background),
    0 0 0 calc(var(--ring-offset) + var(--ring-width))
      color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
`);
var focusRingForcedColors = r(`
  outline: 2px solid CanvasText;
  outline-offset: 2px;
`);
var reducedMotion = i`
  @media (prefers-reduced-motion: reduce) {
    :host, :host *, ::slotted(*) {
      animation-duration: 0.01ms !important;
      animation-iteration-count: 1 !important;
      transition-duration: 0.01ms !important;
      scroll-behavior: auto !important;
    }
  }
`;

// src/web-components/ship-accordion.styles.ts
var shipAccordionStyles = [
  reducedMotion,
  i`
    :host {
      display: block;
      font-family: var(--font-body);
      color: var(--foreground);
    }
    :host([hidden]) { display: none; }
  `
];
var shipAccordionItemStyles = [
  reducedMotion,
  i`
    :host {
      display: block;
      border-bottom: 1px solid var(--border);
    }
    :host([hidden]) { display: none; }

    .trigger {
      display: flex;
      align-items: center;
      justify-content: space-between;
      width: 100%;
      padding: var(--space-4) 0;
      background: none;
      border: none;
      font: inherit;
      font-size: var(--text-base);
      font-weight: 500;
      color: var(--foreground);
      cursor: pointer;
      text-align: left;
      line-height: var(--leading-normal);
      transition: color var(--duration-fast) var(--ease-out);
    }
    .trigger:hover {
      text-decoration: underline;
      text-underline-offset: 4px;
    }

    /* Focus ring */
    .trigger:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
      border-radius: var(--radius-sm);
    }
    @media (forced-colors: active) {
      .trigger:focus-visible {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
    }

    :host([disabled]) .trigger {
      cursor: not-allowed;
      color: var(--muted-foreground);
    }
    :host([disabled]) .trigger:hover {
      text-decoration: none;
    }

    /* Chevron */
    .chevron {
      flex-shrink: 0;
      width: 16px;
      height: 16px;
      color: var(--muted-foreground);
      transition: transform var(--duration-normal, 200ms) var(--ease-out);
    }
    :host([open]) .chevron {
      transform: rotate(180deg);
    }

    /* Content area — CSS grid height animation */
    .content-wrap {
      display: grid;
      grid-template-rows: 0fr;
      transition: grid-template-rows var(--duration-normal, 200ms) var(--ease-out);
    }
    :host([open]) .content-wrap {
      grid-template-rows: 1fr;
    }
    .content-inner {
      overflow: hidden;
    }
    .content-body {
      padding-bottom: var(--space-4);
      font-size: var(--text-sm);
      color: var(--muted-foreground);
      line-height: var(--leading-relaxed);
    }
  `
];

// src/web-components/ship-accordion.ts
var _ShipAccordionItem = class _ShipAccordionItem extends i4 {
  constructor() {
    super(...arguments);
    this.value = "";
    this.disabled = false;
    this.open = false;
    this.uid = ++_ShipAccordionItem._idSeq;
    this.onTriggerClick = () => {
      if (this.disabled) return;
      this.dispatchEvent(new CustomEvent("ship-accordion-item-toggle", {
        detail: { value: this.value },
        bubbles: true,
        composed: true
      }));
    };
    this.onKeydown = (e6) => {
      if (e6.key === "ArrowDown" || e6.key === "ArrowUp") {
        e6.preventDefault();
        this.dispatchEvent(new CustomEvent("ship-accordion-nav", {
          detail: { direction: e6.key === "ArrowDown" ? "next" : "prev" },
          bubbles: true,
          composed: true
        }));
      } else if (e6.key === "Home" || e6.key === "End") {
        e6.preventDefault();
        this.dispatchEvent(new CustomEvent("ship-accordion-nav", {
          detail: { direction: e6.key === "Home" ? "first" : "last" },
          bubbles: true,
          composed: true
        }));
      }
    };
  }
  get triggerId() {
    return `ship-acc-trigger-${this.uid}`;
  }
  get contentId() {
    return `ship-acc-content-${this.uid}`;
  }
  /** Focus the trigger button from the parent accordion's keyboard handler. */
  focusTrigger() {
    const btn = this.renderRoot.querySelector("button");
    btn?.focus();
  }
  render() {
    return b2`
      <button
        class="trigger"
        part="trigger"
        id=${this.triggerId}
        aria-expanded=${this.open ? "true" : "false"}
        aria-controls=${this.contentId}
        ?disabled=${this.disabled}
        @click=${this.onTriggerClick}
        @keydown=${this.onKeydown}
      >
        <slot name="trigger"></slot>
        <svg class="chevron" part="chevron" viewBox="0 0 16 16" fill="none"
          stroke="currentColor" stroke-width="1.75" stroke-linecap="round"
          stroke-linejoin="round" aria-hidden="true">
          <path d="M4 6l4 4 4-4"></path>
        </svg>
      </button>
      <div class="content-wrap" part="content"
        id=${this.contentId}
        role="region"
        aria-labelledby=${this.triggerId}
      >
        <div class="content-inner">
          <div class="content-body">
            <slot></slot>
          </div>
        </div>
      </div>
    `;
  }
};
_ShipAccordionItem.styles = shipAccordionItemStyles;
_ShipAccordionItem.properties = {
  value: { type: String, reflect: true },
  disabled: { type: Boolean, reflect: true },
  open: { type: Boolean, reflect: true }
};
_ShipAccordionItem._idSeq = 0;
var ShipAccordionItem = _ShipAccordionItem;
var ShipAccordion = class extends i4 {
  constructor() {
    super(...arguments);
    this.type = "single";
    this.value = "";
    this.onItemToggle = (e6) => {
      e6.stopPropagation();
      const itemVal = e6.detail.value;
      const open = this.getOpenValues();
      if (this.type === "single") {
        if (open.has(itemVal)) {
          this.setOpenValues(/* @__PURE__ */ new Set());
        } else {
          this.setOpenValues(/* @__PURE__ */ new Set([itemVal]));
        }
      } else {
        if (open.has(itemVal)) {
          open.delete(itemVal);
        } else {
          open.add(itemVal);
        }
        this.setOpenValues(open);
      }
    };
    this.onItemNav = (e6) => {
      e6.stopPropagation();
      const enabled = this.enabledItems;
      if (!enabled.length) return;
      const current = e6.target.closest("ship-accordion-item");
      const idx = current ? enabled.indexOf(current) : -1;
      let target;
      switch (e6.detail.direction) {
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
      target?.focusTrigger();
    };
  }
  connectedCallback() {
    super.connectedCallback();
    this.addEventListener("ship-accordion-item-toggle", this.onItemToggle);
    this.addEventListener("ship-accordion-nav", this.onItemNav);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("ship-accordion-item-toggle", this.onItemToggle);
    this.removeEventListener("ship-accordion-nav", this.onItemNav);
  }
  get items() {
    return Array.from(
      this.querySelectorAll(":scope > ship-accordion-item")
    );
  }
  get enabledItems() {
    return this.items.filter((i6) => !i6.disabled);
  }
  getOpenValues() {
    return new Set(this.value ? this.value.split(",").filter(Boolean) : []);
  }
  setOpenValues(vals) {
    this.value = Array.from(vals).join(",");
    this.syncItems();
    this.dispatchEvent(new CustomEvent("ship-change", {
      detail: { value: this.value },
      bubbles: true,
      composed: true
    }));
  }
  updated() {
    this.syncItems();
  }
  syncItems() {
    const open = this.getOpenValues();
    for (const item of this.items) {
      item.open = open.has(item.value);
    }
  }
  render() {
    return b2`<slot></slot>`;
  }
};
ShipAccordion.styles = shipAccordionStyles;
ShipAccordion.properties = {
  type: { type: String, reflect: true },
  value: { type: String, reflect: true }
};
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-accordion-item")) {
    customElements.define("ship-accordion-item", ShipAccordionItem);
  }
  if (!customElements.get("ship-accordion")) {
    customElements.define("ship-accordion", ShipAccordion);
  }
}

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
    const assertive = this.tone === "destructive" || this.tone === "warning";
    const role = assertive ? "alert" : "status";
    const live = assertive ? "assertive" : "polite";
    return b2`
      <div class="box" role=${role} aria-live=${live} aria-atomic="true">
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

// src/web-components/ship-dialog.styles.ts
var shipDialogStyles = [reducedMotion, i`
  :host { display: contents; }
  :host([hidden]) { display: none; }

  /* Size channel — one source of truth for max-width + padding. Adding a
   * size = one :host([size=…]) block below. */
  :host(:not([size])),
  :host([size='md']) {
    --dlg-max-w: 500px;
    --dlg-px: 24px;
    --dlg-header-py: 18px;
    --dlg-body-py: 16px;
    --dlg-footer-py: 16px;
    --dlg-title-fs: var(--text-lg);
    --dlg-desc-fs: var(--text-base);
  }
  :host([size='sm']) {
    --dlg-max-w: 400px;
    --dlg-px: 20px;
    --dlg-header-py: 16px;
    --dlg-body-py: 14px;
    --dlg-footer-py: 14px;
    --dlg-title-fs: var(--text-md);
    --dlg-desc-fs: var(--text-sm);
  }
  :host([size='lg']) {
    --dlg-max-w: 700px;
    --dlg-px: 28px;
    --dlg-header-py: 20px;
    --dlg-body-py: 20px;
    --dlg-footer-py: 18px;
    --dlg-title-fs: var(--text-xl);
    --dlg-desc-fs: var(--text-base);
  }
  :host([size='full']) {
    --dlg-max-w: 90vw;
    --dlg-px: 28px;
    --dlg-header-py: 20px;
    --dlg-body-py: 20px;
    --dlg-footer-py: 18px;
    --dlg-title-fs: var(--text-xl);
    --dlg-desc-fs: var(--text-base);
  }

  /* The native <dialog>. Centered via dialog's default modal positioning
   * (margin:auto in UA sheet). We control the visual: surface, shadow,
   * rounded corners, and an entrance animation that respects reduced-motion
   * via the reducedMotion stylesheet above. */
  dialog {
    position: fixed;
    inset: 0;
    margin: auto;
    padding: 0;
    border: 1px solid color-mix(in oklch, var(--border) 140%, transparent);
    border-radius: var(--radius-lg);
    background: var(--card);
    color: var(--card-foreground);
    width: calc(100vw - 32px);
    max-width: var(--dlg-max-w);
    max-height: min(calc(100vh - 32px), 85vh);
    box-shadow:
      0 0 0 1px color-mix(in oklch, black 4%, transparent),
      var(--shadow-xl);
    font-family: var(--font-body);
    color-scheme: light dark;
    overflow: visible;
    /* Start state — entrance runs on [open]. */
    opacity: 0;
    transform: translateY(8px) scale(0.96);
    transition:
      opacity var(--duration-medium) var(--ease-out),
      transform var(--duration-medium) var(--ease-out);
  }
  :host([size='full']) dialog {
    max-height: 90vh;
  }
  dialog[open] {
    opacity: 1;
    transform: translateY(0) scale(1);
  }

  /* Backdrop — semi-transparent overlay. Default variant has a subtle tint,
   * the 'blur' variant adds a backdrop-filter for a premium feel. Both fade
   * in with the dialog. */
  dialog::backdrop {
    background: oklch(0 0 0 / 42%);
    opacity: 0;
    transition: opacity var(--duration-medium) var(--ease-out);
  }
  dialog[open]::backdrop {
    opacity: 1;
  }
  :host([backdrop='blur']) dialog::backdrop {
    background: oklch(0 0 0 / 32%);
    backdrop-filter: blur(6px) saturate(1.1);
    -webkit-backdrop-filter: blur(6px) saturate(1.1);
  }
  :host([backdrop='solid']) dialog::backdrop {
    background: oklch(0 0 0 / 62%);
  }

  /* Focus-trap sentinels — invisible, zero-size, but keyboard-tabbable.
   * Placed before and after .wrap so Tab at either edge bounces to the
   * other end via the sentinel focus handler. */
  .sentinel {
    position: fixed;
    width: 1px;
    height: 1px;
    pointer-events: none;
    opacity: 0;
  }

  /* Structural wrapper — flex column so the body area can scroll while the
   * header/footer stay pinned. */
  .wrap {
    display: flex;
    flex-direction: column;
    max-height: inherit;
    position: relative;
    isolation: isolate;
  }

  /* Header — title + optional description. Bordered from body so long
   * content below gets a visible scroll anchor. */
  .header {
    padding: var(--dlg-header-py) var(--dlg-px);
    padding-right: calc(var(--dlg-px) + 28px);
    border-bottom: 1px solid color-mix(in oklch, var(--border) 120%, transparent);
    font-size: var(--dlg-title-fs);
    font-weight: var(--font-weight-semibold);
    letter-spacing: -0.01em;
    line-height: var(--leading-tight);
    color: var(--foreground);
    flex: 0 0 auto;
  }
  :host([no-header]) .header { display: none; }

  /* Optional description — consumers wrap a second line in
   * <span class="ship-dialog-description">. Because slotted descendants
   * live in light DOM, the class is global and needs no ::slotted plumbing;
   * callers can style it directly or let the tokens.css surface a default.
   * Exposed custom properties let hosts theme from outside. */
  :host {
    --ship-dialog-description-color: var(--muted-foreground);
    --ship-dialog-description-size: var(--dlg-desc-fs);
  }

  .body {
    padding: var(--dlg-body-py) var(--dlg-px);
    overflow-y: auto;
    overscroll-behavior: contain;
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    color: var(--card-foreground);
    flex: 1 1 auto;
    min-height: 0;
    /* Subtle scrollbar — matches shadcn's scroll-area feel. */
    scrollbar-width: thin;
    scrollbar-color: color-mix(in oklch, var(--border) 160%, transparent) transparent;
  }
  .body::-webkit-scrollbar {
    width: 8px;
    height: 8px;
  }
  .body::-webkit-scrollbar-thumb {
    background: color-mix(in oklch, var(--border) 160%, transparent);
    border-radius: var(--radius-full);
  }
  .body::-webkit-scrollbar-track { background: transparent; }

  /* Footer — action row. Hidden when no footer slot content to avoid a
   * dangling empty band at the bottom. */
  .footer {
    padding: var(--dlg-footer-py) var(--dlg-px);
    border-top: 1px solid color-mix(in oklch, var(--border) 120%, transparent);
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
    align-items: center;
    flex: 0 0 auto;
    background: color-mix(in oklch, var(--muted) 25%, var(--card));
    border-bottom-left-radius: var(--radius-lg);
    border-bottom-right-radius: var(--radius-lg);
  }
  :host([no-footer]) .footer { display: none; }

  /* Close button — top-right, ghost styling that picks up tone on hover.
   * Sized so the target is comfortably 28px while the glyph stays crisp. */
  .close {
    position: absolute;
    top: 12px;
    inset-inline-end: 12px;
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    color: var(--muted-foreground);
    cursor: pointer;
    padding: 0;
    font: inherit;
    z-index: 1;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }
  .close:hover {
    background: var(--muted);
    color: var(--foreground);
    transform: scale(1.04);
  }
  .close:active {
    transform: scale(0.96);
  }
  .close:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--card),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }
  .close svg {
    width: 14px;
    height: 14px;
    display: block;
  }
  :host([no-close-button]) .close { display: none; }

  /* Forced-colors — ring + border legibility in high-contrast mode. */
  @media (forced-colors: active) {
    dialog { border: 1px solid CanvasText; }
    .close:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`];

// src/web-components/focus.ts
var FOCUSABLE_SELECTOR = [
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
  "audio[controls]",
  "video[controls]",
  '[contenteditable]:not([contenteditable="false"])',
  '[tabindex]:not([tabindex^="-"])'
].join(",");
function focusableWithin(root) {
  const nodes = Array.from(root.querySelectorAll(FOCUSABLE_SELECTOR));
  return nodes.filter((el) => {
    if (el.hasAttribute("disabled")) return false;
    if (el.getAttribute("aria-hidden") === "true") return false;
    if (el instanceof HTMLInputElement && el.type === "hidden") return false;
    return true;
  });
}
var FocusReturn = class {
  constructor() {
    this.saved = null;
  }
  save() {
    this.saved = document.activeElement ?? null;
  }
  restore() {
    const target = this.saved;
    this.saved = null;
    if (target && typeof target.focus === "function") target.focus();
  }
};

// src/web-components/ship-dialog.ts
var scrollLockCount = 0;
var savedBodyOverflow = null;
var savedBodyPaddingRight = null;
function acquireScrollLock() {
  if (typeof document === "undefined") return;
  scrollLockCount += 1;
  if (scrollLockCount > 1) return;
  const body = document.body;
  savedBodyOverflow = body.style.overflow;
  savedBodyPaddingRight = body.style.paddingRight;
  const scrollbarW = window.innerWidth - document.documentElement.clientWidth;
  if (scrollbarW > 0) {
    const current = parseFloat(getComputedStyle(body).paddingRight || "0") || 0;
    body.style.paddingRight = `${current + scrollbarW}px`;
  }
  body.style.overflow = "hidden";
}
function releaseScrollLock() {
  if (typeof document === "undefined") return;
  if (scrollLockCount === 0) return;
  scrollLockCount -= 1;
  if (scrollLockCount > 0) return;
  const body = document.body;
  body.style.overflow = savedBodyOverflow ?? "";
  body.style.paddingRight = savedBodyPaddingRight ?? "";
  savedBodyOverflow = null;
  savedBodyPaddingRight = null;
}
var _ShipDialog = class _ShipDialog extends i4 {
  constructor() {
    super(...arguments);
    this.open = false;
    this.size = "md";
    this.backdrop = "default";
    this.dismissible = true;
    this.noCloseButton = false;
    this.noHeader = false;
    this.noFooter = false;
    this.dialogRole = "dialog";
    this.uid = ++_ShipDialog._idSeq;
    this.focusReturn = new FocusReturn();
    this.hasScrollLock = false;
    this.onClose = () => {
      this.open = false;
      if (this.hasScrollLock) {
        releaseScrollLock();
        this.hasScrollLock = false;
      }
      this.focusReturn.restore();
      this.dispatchEvent(new CustomEvent("ship-close", { bubbles: true, composed: true }));
    };
    this.onBackdropClick = (e6) => {
      if (!this.dismissible) return;
      if (e6.target === this.dialogEl) this.close();
    };
    /** Sentinel-based focus trap. Two invisible focusable divs bracket the
     * dialog contents — if Tab lands on either sentinel we bounce focus to
     * the opposite end of the ring. Works across shadow boundaries without
     * requiring us to enumerate cross-boundary focusables. */
    this.onSentinelFocus = (which) => (_e) => {
      const items = this.collectFocusables();
      if (items.length === 0) return;
      const target = which === "start" ? items[items.length - 1] : items[0];
      this.focusWithFallback(target);
    };
    this.onDismiss = () => {
      this.dispatchEvent(new CustomEvent("ship-dismiss", { bubbles: true, composed: true }));
      this.close();
    };
  }
  get headerId() {
    return `ship-dialog-header-${this.uid}`;
  }
  get bodyId() {
    return `ship-dialog-body-${this.uid}`;
  }
  get dialogEl() {
    return this.renderRoot.querySelector("dialog");
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    if (this.hasScrollLock) {
      releaseScrollLock();
      this.hasScrollLock = false;
    }
  }
  updated(changed) {
    this.setAttribute("data-state", this.open ? "open" : "closed");
    if (!changed.has("open") || !this.dialogEl) return;
    if (this.open && !this.dialogEl.open) {
      this.focusReturn.save();
      if (typeof this.dialogEl.showModal === "function") {
        try {
          this.dialogEl.showModal();
        } catch {
        }
      }
      if (!this.hasScrollLock) {
        acquireScrollLock();
        this.hasScrollLock = true;
      }
      queueMicrotask(() => this.focusInitial());
      this.dispatchEvent(new CustomEvent("ship-open", { bubbles: true, composed: true }));
    } else if (!this.open && this.dialogEl.open) {
      this.dialogEl.close();
    }
  }
  show() {
    this.open = true;
  }
  showModal() {
    this.open = true;
  }
  close() {
    this.open = false;
  }
  /** Collect focusables from both the shadow dialog (close button) and the
   * host's light-DOM children (slotted body/footer controls). Sentinels
   * are excluded. Order: light-DOM (natural tab order for body/footer)
   * first, then the shadow close button. */
  collectFocusables() {
    const shadowRoot = this.renderRoot.querySelector("dialog");
    const shadowFocusables = shadowRoot ? focusableWithin(shadowRoot).filter((el) => !el.classList.contains("sentinel")) : [];
    const lightFocusables = focusableWithin(this);
    return [...lightFocusables, ...shadowFocusables];
  }
  focusInitial() {
    const root = this.dialogEl;
    if (!root) return;
    const lightAutofocus = this.querySelector("[autofocus]");
    const shadowAutofocus = root.querySelector("[autofocus]");
    const autofocus = lightAutofocus ?? shadowAutofocus;
    if (autofocus) {
      autofocus.focus();
      return;
    }
    const focusables = this.collectFocusables();
    const primary = focusables.find((el) => !el.classList.contains("close")) ?? focusables[0];
    if (primary) primary.focus();
    else root.focus();
  }
  focusWithFallback(el) {
    el.focus();
    if (document.activeElement === document.body || document.activeElement === el) {
      const leaf = el.shadowRoot?.querySelector(
        'button, input, select, textarea, [tabindex]:not([tabindex^="-"])'
      ) ?? el.querySelector(
        'button, input, select, textarea, [tabindex]:not([tabindex^="-"])'
      );
      leaf?.focus();
    }
  }
  render() {
    return b2`
      <dialog
        part="dialog"
        role=${this.dialogRole}
        aria-modal="true"
        aria-labelledby=${this.headerId}
        aria-describedby=${this.bodyId}
        @close=${this.onClose}
        @click=${this.onBackdropClick}
      >
        <span tabindex="0" class="sentinel" aria-hidden="true"
          @focus=${this.onSentinelFocus("start")}></span>
        <div class="wrap">
          <button
            class="close"
            part="close"
            type="button"
            aria-label="Close dialog"
            @click=${this.onDismiss}
          >
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor"
              stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round"
              aria-hidden="true" focusable="false">
              <path d="M3.5 3.5l9 9M12.5 3.5l-9 9"></path>
            </svg>
          </button>
          <div class="header" part="header" id=${this.headerId}>
            <slot name="header"></slot>
          </div>
          <div class="body" part="body" id=${this.bodyId}>
            <slot></slot>
          </div>
          <div class="footer" part="footer">
            <slot name="footer"></slot>
          </div>
        </div>
        <span tabindex="0" class="sentinel" aria-hidden="true"
          @focus=${this.onSentinelFocus("end")}></span>
      </dialog>
    `;
  }
};
_ShipDialog.styles = shipDialogStyles;
_ShipDialog.properties = {
  open: { type: Boolean, reflect: true },
  size: { type: String, reflect: true },
  backdrop: { type: String, reflect: true },
  dismissible: { type: Boolean },
  noCloseButton: { type: Boolean, reflect: true, attribute: "no-close-button" },
  noHeader: { type: Boolean, reflect: true, attribute: "no-header" },
  noFooter: { type: Boolean, reflect: true, attribute: "no-footer" },
  dialogRole: { type: String, attribute: "role-override" }
};
_ShipDialog._idSeq = 0;
var ShipDialog = _ShipDialog;
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
    this.label = "";
    this.pressed = "";
    this.expanded = "";
    /** Detect an empty default slot so we can render a square icon-only button
     * without requiring callers to opt in with a new attribute. */
    this.handleDefaultSlotChange = (e6) => {
      const slot = e6.target;
      const nodes = slot.assignedNodes({ flatten: true });
      const hasText = nodes.some((n4) => {
        if (n4.nodeType === Node.TEXT_NODE) return (n4.textContent || "").trim().length > 0;
        if (n4.nodeType === Node.ELEMENT_NODE) {
          const el = n4;
          return (el.textContent || "").trim().length > 0 || el.childElementCount > 0;
        }
        return false;
      });
      if (hasText) this.removeAttribute("data-icon-only");
      else this.setAttribute("data-icon-only", "");
    };
  }
  render() {
    return b2`
      <button
        part="button"
        type=${this.type}
        ?disabled=${this.disabled || this.loading}
        aria-busy=${this.loading ? "true" : "false"}
        aria-disabled=${this.disabled || this.loading ? "true" : "false"}
        aria-label=${this.label || A}
        aria-pressed=${this.pressed || A}
        aria-expanded=${this.expanded || A}
      >
        <span class="content">
          <slot name="leading"></slot>
          <slot @slotchange=${this.handleDefaultSlotChange}></slot>
          <slot name="trailing"></slot>
        </span>
        ${this.loading ? b2`<span class="spinner-wrap" aria-hidden="true"><span class="spinner"></span></span>` : A}
      </button>
    `;
  }
};
ShipButton.styles = [reducedMotion, i`
    :host {
      display: inline-flex;
      vertical-align: middle;
      /* Per-host tone channel. Overridden per :host([tone=...]) below so
       * the variant selectors can stay tone-agnostic and compose freely. */
      --tone: var(--primary);
      --tone-fg: var(--primary-foreground);
      --tone-contrast: oklch(0.99 0.02 95);
    }
    :host([hidden]) { display: none; }

    /* Tone channel — each tone supplies its own accent color + a readable
     * contrast foreground for solid fills. The rest of the styling is driven
     * purely off --tone / --tone-contrast so variants stay DRY. */
    :host(:not([tone])),
    :host([tone='default']) {
      --tone: var(--foreground);
      --tone-fg: var(--foreground);
      --tone-contrast: var(--background);
    }
    :host([tone='info']) {
      --tone: var(--info);
      --tone-fg: var(--info);
      --tone-contrast: oklch(0.99 0.02 260);
    }
    :host([tone='success']) {
      --tone: var(--success);
      --tone-fg: var(--success);
      --tone-contrast: oklch(0.99 0.02 150);
    }
    :host([tone='warning']) {
      --tone: var(--warning);
      --tone-fg: var(--warning);
      --tone-contrast: oklch(0.2 0.04 75);
    }
    :host([tone='destructive']) {
      --tone: var(--destructive);
      --tone-fg: var(--destructive);
      --tone-contrast: oklch(0.99 0.02 25);
    }
    :host([tone='accent']) {
      --tone: var(--accent);
      --tone-fg: var(--accent);
      --tone-contrast: var(--accent-foreground);
    }
    :host([tone='muted']) {
      --tone: var(--muted-foreground);
      --tone-fg: var(--muted-foreground);
      --tone-contrast: var(--background);
    }

    button {
      position: relative;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: var(--space-2);
      font-family: var(--font-body);
      font-weight: var(--font-weight-medium);
      letter-spacing: -0.005em;
      line-height: 1;
      border-radius: var(--radius-md);
      cursor: pointer;
      border: 1px solid transparent;
      background: transparent;
      color: var(--foreground);
      isolation: isolate;
      user-select: none;
      -webkit-user-select: none;
      -webkit-tap-highlight-color: transparent;
      /* Optical alignment for g/y descenders — nudge content up by 0.5px so
       * the label sits visually centered between cap-height and baseline. */
      padding-block-start: calc(var(--btn-py) - 0.5px);
      padding-block-end: calc(var(--btn-py) + 0.5px);
      padding-inline: var(--btn-px);
      transform: translateZ(0);
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out),
        filter var(--duration-fast) var(--ease-out);
      white-space: nowrap;
    }

    /* Size tokens — drive padding/height/typography from a single --btn-*
     * channel so the matrix below never needs to repeat sizing logic. */
    :host(:not([size])),
    :host([size='md']) {
      --btn-h: 34px;
      --btn-px: var(--space-3);
      --btn-py: 0px;
      --btn-fs: var(--text-base);
      --btn-fw: var(--font-weight-semibold);
      --btn-gap: var(--space-2);
      --btn-icon: 14px;
    }
    :host([size='sm']) {
      --btn-h: 28px;
      --btn-px: var(--space-3);
      --btn-py: 0px;
      --btn-fs: var(--text-sm);
      --btn-fw: var(--font-weight-medium);
      --btn-gap: 6px;
      --btn-icon: 12px;
    }
    :host([size='lg']) {
      --btn-h: 40px;
      --btn-px: var(--space-4);
      --btn-py: 0px;
      --btn-fs: var(--text-md);
      --btn-fw: var(--font-weight-semibold);
      --btn-gap: var(--space-2);
      --btn-icon: 16px;
    }

    button {
      min-height: var(--btn-h);
      font-size: var(--btn-fs);
      font-weight: var(--btn-fw);
      gap: var(--btn-gap);
    }

    /* Icon-only — detected via the default slot being empty of visible text.
     * When active, collapse horizontal padding so the button renders as a
     * perfectly square affordance for icon adornments. */
    :host([data-icon-only]) button {
      padding-inline: 0;
      inline-size: var(--btn-h);
      min-inline-size: var(--btn-h);
    }

    /* ──────────────────────────────────────────────────────────────────
     * Variant matrix — each rule reads only from --tone/--tone-contrast,
     * so adding a tone is a single :host([tone=…]) channel override above.
     * ────────────────────────────────────────────────────────────────── */

    /* solid — tone fill, contrast text, subtle top-highlight for premium feel */
    :host([variant='solid']) button {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      color: var(--tone-contrast);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent),
        var(--shadow-xs);
    }
    :host([variant='solid']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
        var(--shadow-sm);
      transform: translateY(-1px);
    }
    :host([variant='solid']) button:active:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 92%, black 8%);
      box-shadow:
        inset 0 1px 2px 0 color-mix(in oklch, black 18%, transparent),
        var(--shadow-xs);
      transform: translateY(0);
    }

    /* Solid default tone: neutral surface that still feels like a filled
     * button (matches shadcn's default variant). */
    :host(:not([tone])[variant='solid']) button,
    :host([tone='default'][variant='solid']) button {
      background: var(--foreground);
      border-color: var(--foreground);
      color: var(--background);
    }
    :host(:not([tone])[variant='solid']) button:hover:not(:disabled),
    :host([tone='default'][variant='solid']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--foreground) 88%, var(--background));
    }

    /* Default (no variant attr) is the neutral bordered button — matches
     * the existing view styling and shadcn's outline feel. */
    :host(:not([variant])) button {
      background: var(--background);
      border-color: var(--border);
      color: var(--foreground);
      box-shadow: var(--shadow-xs);
    }
    :host(:not([variant])) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--muted) 70%, var(--background));
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
      box-shadow: var(--shadow-sm);
      transform: translateY(-1px);
    }
    :host(:not([variant])) button:active:not(:disabled) {
      background: var(--muted);
      box-shadow: var(--shadow-xs);
      transform: translateY(0);
    }
    /* Default button with a tone: colorize text + border subtly so tone is
     * legible without overpowering the neutral surface. */
    :host(:not([variant])[tone]:not([tone='default']):not([tone='muted'])) button {
      color: var(--tone-fg);
      border-color: color-mix(in oklch, var(--tone) 38%, var(--border));
    }
    :host(:not([variant])[tone]:not([tone='default']):not([tone='muted'])) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 8%, var(--background));
      border-color: color-mix(in oklch, var(--tone) 55%, var(--border));
    }

    /* soft — tone-tinted surface with a faint border for definition */
    :host([variant='soft']) button {
      background: color-mix(in oklch, var(--tone) 14%, transparent);
      border-color: color-mix(in oklch, var(--tone) 22%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='soft']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 22%, transparent);
      border-color: color-mix(in oklch, var(--tone) 32%, transparent);
      transform: translateY(-1px);
      box-shadow: var(--shadow-xs);
    }
    :host([variant='soft']) button:active:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 28%, transparent);
      transform: translateY(0);
      box-shadow: none;
    }

    /* outline — transparent with tone border + text */
    :host([variant='outline']) button {
      background: transparent;
      border-color: color-mix(in oklch, var(--tone) 55%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='outline']:not([tone])) button,
    :host([variant='outline'][tone='default']) button {
      border-color: var(--border);
      color: var(--foreground);
    }
    :host([variant='outline']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 10%, transparent);
      border-color: color-mix(in oklch, var(--tone) 75%, transparent);
      transform: translateY(-1px);
      box-shadow: var(--shadow-xs);
    }
    :host([variant='outline']:not([tone])) button:hover:not(:disabled),
    :host([variant='outline'][tone='default']) button:hover:not(:disabled) {
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
    }
    :host([variant='outline']) button:active:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 16%, transparent);
      transform: translateY(0);
      box-shadow: none;
    }

    /* ghost — no chrome, tone tint on hover */
    :host([variant='ghost']) button {
      background: transparent;
      border-color: transparent;
      color: var(--tone-fg);
    }
    :host([variant='ghost']:not([tone])) button,
    :host([variant='ghost'][tone='default']) button {
      color: var(--muted-foreground);
    }
    :host([variant='ghost']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 12%, transparent);
    }
    :host([variant='ghost']:not([tone])) button:hover:not(:disabled),
    :host([variant='ghost'][tone='default']) button:hover:not(:disabled) {
      background: var(--muted);
      color: var(--foreground);
    }
    :host([variant='ghost']) button:active:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 18%, transparent);
    }

    /* Premium treatment for the primary CTA (solid + accent): a subtle
     * diagonal gradient + faint colored glow on hover. Keeps the rest of
     * the matrix honest while giving the hero button some presence. */
    :host([variant='solid'][tone='accent']) button {
      background:
        linear-gradient(
          180deg,
          color-mix(in oklch, var(--accent) 100%, white 6%) 0%,
          var(--accent) 100%
        );
    }
    :host([variant='solid'][tone='accent']) button:hover:not(:disabled) {
      background:
        linear-gradient(
          180deg,
          color-mix(in oklch, var(--accent) 100%, white 10%) 0%,
          color-mix(in oklch, var(--accent) 95%, white 5%) 100%
        );
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 24%, transparent),
        var(--shadow-sm),
        0 0 0 1px color-mix(in oklch, var(--accent) 40%, transparent),
        0 6px 20px -8px color-mix(in oklch, var(--accent) 70%, transparent);
    }

    /* Destructive solid: bump contrast so it visibly demands attention. */
    :host([variant='solid'][tone='destructive']) button {
      font-weight: var(--font-weight-semibold);
    }
    :host([variant='solid'][tone='destructive']) button:hover:not(:disabled) {
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 24%, transparent),
        var(--shadow-sm),
        0 4px 14px -4px color-mix(in oklch, var(--destructive) 55%, transparent);
    }

    /* Focus ring — :focus-visible only, rendered via stacked box-shadows
     * so the ring sits outside the border without reserving layout space. */
    button:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) 55%, transparent);
    }
    :host(:not([tone])) button:focus-visible,
    :host([tone='default']) button:focus-visible {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    @media (forced-colors: active) {
      button:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
    }

    /* Disabled — muted, no pointer, no motion. Distinct enough from hover
     * state that users can tell it's unreachable, not just loading. */
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
    :host([variant='outline']) button:disabled {
      border-color: color-mix(in oklch, var(--border) 60%, transparent);
    }

    /* Loading — fade the label out while preserving layout, overlay spinner
     * in the same color as the text would be (currentColor on the button). */
    :host([loading]) button { cursor: progress; }
    :host([loading]) .content {
      opacity: 0;
      transform: scale(0.96);
    }
    .content {
      display: inline-flex;
      align-items: center;
      gap: var(--btn-gap);
      transition:
        opacity var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
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
      width: var(--btn-icon);
      height: var(--btn-icon);
      border: 1.5px solid currentColor;
      border-right-color: transparent;
      border-radius: 50%;
      animation: spin 0.7s linear infinite;
      display: inline-block;
    }
    @keyframes spin { to { transform: rotate(360deg); } }

    /* Slotted icons — constrain to the size channel so icons visually match
     * the typographic scale without each caller having to set a size. */
    ::slotted([slot='leading']),
    ::slotted([slot='trailing']) {
      display: inline-flex;
      align-items: center;
      flex: 0 0 auto;
    }
    ::slotted(svg) {
      width: var(--btn-icon);
      height: var(--btn-icon);
      display: block;
    }
  `];
ShipButton.properties = {
  tone: { type: String, reflect: true },
  variant: { type: String, reflect: true },
  size: { type: String, reflect: true },
  disabled: { type: Boolean, reflect: true },
  loading: { type: Boolean, reflect: true },
  type: { type: String, reflect: true },
  label: { type: String },
  pressed: { type: String, reflect: true },
  expanded: { type: String, reflect: true }
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
      <ship-dialog role-override="alertdialog" size="sm" no-close-button ?open=${this.open}
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

// src/web-components/ship-badge.styles.ts
var shipBadgeStyles = [
  reducedMotion,
  i`
    :host {
      display: inline-flex;
      vertical-align: middle;
      /* Tone channel — overridden per :host([tone=…]). Default reads as
       * muted so a bare <ship-badge> sits quietly until given meaning. */
      --tone: var(--muted-foreground);
      --tone-fg: var(--muted-foreground);
      --tone-contrast: var(--background);
    }
    :host([hidden]) { display: none; }

    /* ------------------------------------------------------------------
     * Tone channel. Contrast colors picked so solid fills stay legible
     * in both light and dark themes (matches ship-button's tone table).
     * ------------------------------------------------------------------ */
    :host(:not([tone])),
    :host([tone='muted']) {
      --tone: var(--muted-foreground);
      --tone-fg: var(--muted-foreground);
      --tone-contrast: var(--background);
    }
    :host([tone='default']) {
      --tone: var(--foreground);
      --tone-fg: var(--foreground);
      --tone-contrast: var(--background);
    }
    :host([tone='info']) {
      --tone: var(--info);
      --tone-fg: var(--info);
      --tone-contrast: oklch(0.99 0.02 260);
    }
    :host([tone='success']) {
      --tone: var(--success);
      --tone-fg: var(--success);
      --tone-contrast: oklch(0.99 0.02 150);
    }
    :host([tone='warning']) {
      --tone: var(--warning);
      --tone-fg: var(--warning);
      --tone-contrast: oklch(0.2 0.04 75);
    }
    :host([tone='destructive']) {
      --tone: var(--destructive);
      --tone-fg: var(--destructive);
      --tone-contrast: oklch(0.99 0.02 25);
    }
    :host([tone='accent']) {
      --tone: var(--accent);
      --tone-fg: var(--accent);
      --tone-contrast: var(--accent-foreground);
    }

    /* ------------------------------------------------------------------
     * Size channel. Heights chosen to slot cleanly into the common
     * contexts: sm inside dense table cells next to a monospace cell,
     * md for inline status after a title, lg for hero/summary cards.
     * Uppercase + 0.04em tracking matches the pre-polish pipelines
     * status pill so table rows don't re-flow.
     * ------------------------------------------------------------------ */
    :host(:not([size])),
    :host([size='md']) {
      --bdg-h: 22px;
      --bdg-px: 8px;
      --bdg-fs: 10.5px;
      --bdg-gap: 4px;
      --bdg-icon: 10px;
    }
    :host([size='sm']) {
      --bdg-h: 18px;
      --bdg-px: 6px;
      --bdg-fs: 9.5px;
      --bdg-gap: 3px;
      --bdg-icon: 8px;
    }
    :host([size='lg']) {
      --bdg-h: 26px;
      --bdg-px: 10px;
      --bdg-fs: 11.5px;
      --bdg-gap: 5px;
      --bdg-icon: 12px;
    }

    /* Shape channel — square by default (visual parity with pre-polish
     * badge), pill when opted-in. Both shapes share the same height &
     * padding so swapping shapes doesn't re-layout surrounding text. */
    :host { --bdg-radius: var(--radius-sm); }
    :host([pill]) { --bdg-radius: var(--radius-full); }

    /* ------------------------------------------------------------------
     * Core badge chrome. Non-interactive — no cursor, no hover, no
     * focus ring. Padding block is 0 so the explicit min-height owns
     * the vertical rhythm; inline padding follows the size channel.
     * ------------------------------------------------------------------ */
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
      /* Base styling — acts as the soft variant so any unknown or missing
       * variant attribute still renders a legible pill. Known variants
       * below re-declare these three properties to override. */
      background: color-mix(in oklch, var(--tone) 16%, transparent);
      border: 1px solid color-mix(in oklch, var(--tone) 24%, transparent);
      color: var(--tone-fg);
      white-space: nowrap;
      -webkit-font-smoothing: antialiased;
      font-variant-numeric: tabular-nums;
    }

    /* ------------------------------------------------------------------
     * Variant matrix. Reads only from --tone / --tone-contrast so the
     * matrix stays DRY across tones. Default (no variant attr) is the
     * soft look — matches the pre-polish badge and is the context
     * where badges sit most often (inline row metadata).
     * ------------------------------------------------------------------ */

    /* soft — tone-tinted surface, faint border, tone text. Default is
     * already applied by .badge above; this explicit selector exists
     * so callers that pass variant=soft target the same rules as
     * variant-less badges. */
    :host([variant='soft']) .badge {
      background: color-mix(in oklch, var(--tone) 16%, transparent);
      border-color: color-mix(in oklch, var(--tone) 24%, transparent);
      color: var(--tone-fg);
    }

    /* solid — filled tone, contrast text, subtle inset highlight for
     * the premium feel (same recipe as ship-button solid). */
    :host([variant='solid']) .badge {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      color: var(--tone-contrast);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
    }
    :host([variant='solid'][tone='muted']) .badge,
    :host([variant='solid']:not([tone])) .badge {
      background: var(--muted-foreground);
      border-color: color-mix(in oklch, var(--muted-foreground) 100%, black 6%);
      color: var(--background);
    }

    /* outline — transparent with tone border + tone text. The slightly
     * deeper border keeps the chip legible against textured backgrounds. */
    :host([variant='outline']) .badge {
      background: transparent;
      border-color: color-mix(in oklch, var(--tone) 50%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='outline']:not([tone])) .badge,
    :host([variant='outline'][tone='muted']) .badge {
      border-color: var(--border);
      color: var(--muted-foreground);
    }

    /* ------------------------------------------------------------------
     * Leading slot — icon / status dot. Sized from the icon channel so
     * the glyph always matches the label's cap-height optically. Uses
     * currentColor so an <ship-status-dot> or <ship-icon> tints with
     * the badge's tone-fg without extra wiring.
     * ------------------------------------------------------------------ */
    .leading {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: var(--bdg-icon);
      height: var(--bdg-icon);
      flex-shrink: 0;
      color: currentColor;
    }
    /* Hide leading wrapper when no content is slotted into it.
     * :empty doesn't work because <slot> is always a child. Use
     * :not(:has(::slotted(*))) for modern browsers. */
    .leading { display: none; }
    .leading:has(::slotted(*)) { display: inline-flex; }
    ::slotted([slot='leading']) {
      display: inline-flex;
      align-items: center;
      color: currentColor;
      width: 100%;
      height: 100%;
    }

    /* Forced-colors — fall back to system colors so high-contrast
     * modes render the badge border reliably. */
    @media (forced-colors: active) {
      .badge { border-color: CanvasText; color: CanvasText; }
      :host([variant='solid']) .badge {
        background: Highlight;
        color: HighlightText;
        border-color: Highlight;
      }
    }
  `
];

// src/web-components/ship-badge.ts
var ShipBadge = class extends i4 {
  constructor() {
    super();
    this.tone = "muted";
    this.variant = null;
    this.size = "md";
    this.pill = false;
  }
  render() {
    return b2`
      <span class="badge" part="badge">
        <span class="leading" part="leading"><slot name="leading"></slot></span>
        <slot></slot>
      </span>
    `;
  }
};
ShipBadge.styles = shipBadgeStyles;
ShipBadge.properties = {
  tone: { type: String, reflect: true },
  variant: { type: String, reflect: true },
  size: { type: String, reflect: true },
  pill: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-badge")) {
  customElements.define("ship-badge", ShipBadge);
}

// src/web-components/ship-card.styles.ts
var shipCardStyles = [
  reducedMotion,
  toneVars,
  i`
    :host {
      display: block;
      position: relative;
      background: var(--card);
      color: var(--card-foreground);
      border: 1px solid var(--border);
      border-radius: var(--card-radius);
      box-shadow: var(--shadow-xs);
      /* Density channel — overridden per :host([padding=…]) below.
       * Defaults match pre-polish 16px/20px so existing composites
       * (ship-pipeline-definition, ship-pipeline-run, settings cards)
       * keep their visual weight. */
      --card-px: 20px;
      --card-py: 16px;
      --card-gap: var(--space-3);
      --card-radius: var(--radius-lg);
      padding: var(--card-py) var(--card-px);
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }
    :host([hidden]) { display: none; }

    /* Density channel. 'none' is useful for nested composites that want
     * total control over internal spacing. */
    :host([padding='none']) {
      --card-px: 0px;
      --card-py: 0px;
      --card-gap: 0px;
    }
    :host([padding='sm']) {
      --card-px: var(--space-3);
      --card-py: var(--space-2);
      --card-gap: var(--space-2);
    }
    :host([padding='lg']) {
      --card-px: var(--space-6);
      --card-py: var(--space-5);
      --card-gap: var(--space-4);
    }

    /* Variant: outline drops the solid fill in favor of the border alone —
     * used for secondary surfaces that sit inside another card/panel. */
    :host([variant='outline']) {
      background: transparent;
      box-shadow: none;
    }

    /* Toned border when a tone is set (but not default). Keeps the surface
     * neutral; the tone lives on the edge. Applied on the non-fill path so
     * it doesn't fight the wash. */
    :host([tone]:not([tone='default']):not([fill])) {
      border-color: var(--tone-border);
    }

    /* fill — tone-tinted background wash. Pairs with status-rail or tone
     * for an alert-style card. Border picks up tone-border so the edge
     * follows the wash instead of clashing with the neutral token. */
    :host([fill]) {
      background: var(--tone-bg);
      border-color: var(--tone-border);
    }

    /* status-rail — 3px left border tinted by tone. Preserves padding by
     * offsetting the inline-start inset so inner text doesn't jump when
     * the rail toggles on. */
    :host([status-rail]) {
      border-left: 3px solid var(--tone-fg);
      padding-left: calc(var(--card-px) - 3px);
    }

    /* accent-bar — 2-stop gradient top-bar that fades in on hover.
     * Rendered as a pseudo-element so the bar scrolls with the card and
     * follows the radius curve without needing clip-path. */
    :host([accent-bar])::before {
      content: '';
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      height: 2px;
      background: linear-gradient(90deg, var(--primary), var(--accent));
      opacity: 0;
      border-radius: var(--card-radius) var(--card-radius) 0 0;
      transition: opacity var(--duration-medium) var(--ease-out);
      pointer-events: none;
    }
    :host([accent-bar]:hover)::before,
    :host([accent-bar][interactive]:focus-visible)::before {
      opacity: 1;
    }

    /* ------------------------------------------------------------------
     * Interactive — hover lift + active press + focus-visible ring.
     * Non-interactive cards stay static; this block is the only place the
     * card picks up pointer affordances. Mirrors ship-button's motion so
     * the two read as the same design system when sitting side-by-side.
     * ------------------------------------------------------------------ */
    :host([interactive]) {
      cursor: pointer;
      -webkit-tap-highlight-color: transparent;
    }
    :host([interactive]:hover) {
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
      box-shadow: var(--shadow-sm);
      transform: translateY(-1px);
    }
    :host([interactive][tone]:not([tone='default']):hover) {
      border-color: color-mix(in oklch, var(--tone-fg) 55%, var(--border));
    }
    :host([interactive]:active) {
      box-shadow: var(--shadow-xs);
      transform: translateY(0);
      transition-duration: 60ms;
    }

    /* Focus ring — only for keyboard focus on interactive cards. Stacked
     * box-shadow so the ring sits outside the border without reserving
     * layout space (matches ship-button / ship-select / ship-input). */
    :host([interactive]:focus-visible) {
      outline: none;
      border-color: color-mix(in oklch, var(--ring-color) 55%, var(--border));
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    :host([interactive][tone]:not([tone='default']):focus-visible) {
      border-color: color-mix(in oklch, var(--tone-fg) 60%, var(--border));
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone-fg) var(--ring-opacity), transparent);
    }

    /* ------------------------------------------------------------------
     * Slots — header/content/footer with subtle separators. Separators
     * render only when the corresponding slot has projected content,
     * detected via the [data-filled] attribute set from slotchange.
     * ------------------------------------------------------------------ */
    .header,
    .footer {
      display: block;
    }
    .header[data-filled] {
      margin: calc(var(--card-py) * -1) calc(var(--card-px) * -1) var(--card-gap);
      padding: var(--card-py) var(--card-px);
      border-bottom: 1px solid
        color-mix(in oklch, var(--border) 80%, transparent);
    }
    .footer[data-filled] {
      margin: var(--card-gap) calc(var(--card-px) * -1) calc(var(--card-py) * -1);
      padding: var(--card-py) var(--card-px);
      border-top: 1px solid
        color-mix(in oklch, var(--border) 80%, transparent);
    }
    /* When there's no inner padding there's nothing to negative-margin
     * against; the separator becomes a plain rule instead. */
    :host([padding='none']) .header[data-filled] {
      margin: 0 0 var(--space-3);
      padding: 0 0 var(--space-3);
    }
    :host([padding='none']) .footer[data-filled] {
      margin: var(--space-3) 0 0;
      padding: var(--space-3) 0 0;
    }

    /* status-rail narrows the left margin by the 3px rail so the header
     * separator doesn't sit over the colored rail. */
    :host([status-rail]) .header[data-filled] {
      margin-left: calc(var(--card-px) * -1 + 3px);
    }
    :host([status-rail]) .footer[data-filled] {
      margin-left: calc(var(--card-px) * -1 + 3px);
    }

    /* Forced-colors — keep the focus ring visible in high-contrast mode
     * where color-mix'd shadows collapse to the system palette. */
    @media (forced-colors: active) {
      :host([interactive]:focus-visible) {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
    }
  `
];

// src/web-components/ship-card.ts
var ShipCard = class extends i4 {
  constructor() {
    super(...arguments);
    this.variant = "default";
    this.padding = "md";
    this.interactive = false;
    this.accentBar = false;
    this.statusRail = false;
    this.fill = false;
    this.onKey = (e6) => {
      if (!this.interactive) return;
      if (e6.key === "Enter" || e6.key === " ") {
        e6.preventDefault();
        this.click();
      }
    };
    /** Reflect projected content onto the wrapper so CSS can render a
     * separator only when the header/footer slot actually has content. */
    this.syncFilled = (e6) => {
      const slot = e6.target;
      const wrapper = slot.parentElement;
      if (!wrapper) return;
      const filled = slot.assignedNodes({ flatten: true }).some((n4) => {
        if (n4.nodeType === Node.TEXT_NODE) return (n4.textContent || "").trim().length > 0;
        return n4.nodeType === Node.ELEMENT_NODE;
      });
      if (filled) wrapper.setAttribute("data-filled", "");
      else wrapper.removeAttribute("data-filled");
    };
  }
  connectedCallback() {
    super.connectedCallback();
    if (this.interactive) this.activateInteractiveRole();
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("keydown", this.onKey);
  }
  updated(changed) {
    if (changed.has("interactive")) {
      if (this.interactive) this.activateInteractiveRole();
      else this.deactivateInteractiveRole();
    }
  }
  activateInteractiveRole() {
    if (!this.hasAttribute("role")) this.setAttribute("role", "button");
    if (!this.hasAttribute("tabindex")) this.setAttribute("tabindex", "0");
    this.addEventListener("keydown", this.onKey);
  }
  deactivateInteractiveRole() {
    if (this.getAttribute("role") === "button") this.removeAttribute("role");
    if (this.getAttribute("tabindex") === "0") this.removeAttribute("tabindex");
    this.removeEventListener("keydown", this.onKey);
  }
  render() {
    return b2`
      <div class="header" part="header">
        <slot name="header" @slotchange=${this.syncFilled}></slot>
      </div>
      <div class="content" part="content">
        <slot></slot>
      </div>
      <div class="footer" part="footer">
        <slot name="footer" @slotchange=${this.syncFilled}></slot>
      </div>
    `;
  }
};
ShipCard.styles = shipCardStyles;
ShipCard.properties = {
  tone: { type: String, reflect: true },
  variant: { type: String, reflect: true },
  padding: { type: String, reflect: true },
  interactive: { type: Boolean, reflect: true },
  accentBar: { type: Boolean, reflect: true, attribute: "accent-bar" },
  statusRail: { type: Boolean, reflect: true, attribute: "status-rail" },
  fill: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-card")) {
  customElements.define("ship-card", ShipCard);
}

// src/web-components/ship-checkbox.styles.ts
var shipCheckboxStyles = [
  reducedMotion,
  i`
    :host {
      display: inline-flex;
      align-items: center;
      gap: var(--space-2);
      vertical-align: middle;
      font-family: var(--font-body);
      font-size: var(--text-base);
      color: var(--foreground);
      -webkit-tap-highlight-color: transparent;

      /* Tone channel — default is primary so an attribute-less checkbox
       * reads as the brand fill (matches shadcn/Radix defaults). */
      --tone: var(--primary);
      --tone-on-fg: var(--primary-foreground);
    }
    :host([hidden]) { display: none; }

    :host(:not([tone])),
    :host([tone='default']) {
      --tone: var(--primary);
      --tone-on-fg: var(--primary-foreground);
    }
    :host([tone='info'])        { --tone: var(--info);        --tone-on-fg: oklch(0.99 0.02 260); }
    :host([tone='success'])     { --tone: var(--success);     --tone-on-fg: oklch(0.99 0.02 150); }
    :host([tone='warning'])     { --tone: var(--warning);     --tone-on-fg: oklch(0.2 0.04 75); }
    :host([tone='destructive']) { --tone: var(--destructive); --tone-on-fg: oklch(0.99 0.02 25); }
    :host([tone='accent'])      { --tone: var(--accent);      --tone-on-fg: var(--accent-foreground); }
    :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }

    /* Size channel. Stroke width scales with size so the check glyph
     * maintains the same optical weight from sm through lg. */
    :host(:not([size])),
    :host([size='md']) {
      --cb-size: 16px;
      --cb-radius: var(--radius-sm);
      --cb-stroke: 2px;
    }
    :host([size='sm']) {
      --cb-size: 14px;
      --cb-radius: 3px;
      --cb-stroke: 1.75px;
    }
    :host([size='lg']) {
      --cb-size: 18px;
      --cb-radius: var(--radius-sm);
      --cb-stroke: 2.25px;
    }

    label {
      display: inline-flex;
      align-items: center;
      gap: var(--space-2);
      cursor: pointer;
      user-select: none;
      -webkit-user-select: none;
    }
    :host([disabled]) label { cursor: not-allowed; }

    /* ------------------------------------------------------------------
     * Box — the square. Border adds a crisp edge; box-sizing absorbs the
     * border so the visible square reads at the advertised size.
     * ------------------------------------------------------------------ */
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
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
    }

    /* Hover — quiet border darkening + subtle shadow bump. Matches
     * ship-input's at-rest-to-hover language so a form row reads as
     * one cohesive surface. */
    :host(:not([disabled]):not([checked]):not([indeterminate])) label:hover .box {
      border-color: color-mix(in oklch, var(--border) 220%, transparent);
      box-shadow: var(--shadow-sm);
    }

    /* Invalid — destructive border regardless of tone, so an error on
     * an info/success-toned checkbox is unmissable. Mirrors ship-input. */
    :host([invalid]) .box {
      border-color: color-mix(in oklch, var(--destructive) 70%, var(--border));
    }
    :host([invalid]:not([disabled])) label:hover .box {
      border-color: color-mix(in oklch, var(--destructive) 85%, var(--border));
    }

    /* Checked / indeterminate — fill with tone, top-highlight inset for
     * the premium feel (same recipe as ship-button solid). Border picks
     * up a slightly darker edge so the square still reads crisply. */
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

    /* ------------------------------------------------------------------
     * Glyphs — check + indeterminate. Rendered in currentColor on
     * --tone-on-fg so the glyph is always legible against the fill.
     * Opacity/scale transitions give a tiny snap when toggling — the
     * kind of detail that sells the component as production-tier.
     * ------------------------------------------------------------------ */
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
        opacity var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
      pointer-events: none;
    }
    :host([checked]:not([indeterminate])) .glyph.check,
    :host([indeterminate]) .glyph.mixed {
      opacity: 1;
      transform: scale(1);
    }

    .glyph svg {
      width: 100%;
      height: 100%;
      display: block;
    }
    .glyph svg path,
    .glyph svg line {
      stroke: currentColor;
      stroke-width: var(--cb-stroke);
      stroke-linecap: round;
      stroke-linejoin: round;
      fill: none;
    }

    /* ------------------------------------------------------------------
     * Native input — visually hidden but preserved for form submission,
     * keyboard toggle (Space), and a11y tools. Stretches over the box so
     * hit-testing matches the visible affordance.
     * ------------------------------------------------------------------ */
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

    /* Focus ring — stacked box-shadow on the box so the ring sits
     * outside the square without reserving layout. Uses :has() so the
     * ring travels with the native input's :focus-visible, keeping
     * keyboard semantics identical to ship-button / ship-switch. */
    :host(:not([disabled])) label:has(input:focus-visible) .box {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }
    :host([invalid]:not([disabled])) label:has(input:focus-visible) .box {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
    }

    /* Forced colors — drop the color-mix ring in favor of a CanvasText
     * outline that high-contrast modes can render reliably. */
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

    /* ------------------------------------------------------------------
     * Disabled — muted + desaturated, no interactions. Distinct from
     * invalid/loading so the unreachable state is unambiguous.
     * ------------------------------------------------------------------ */
    :host([disabled]) {
      color: var(--muted-foreground);
    }
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

    /* Slotted label — inherits host color + size so callers don't need
     * to restyle. Wraps naturally when the row gets tight. */
    ::slotted(*) {
      font: inherit;
      color: inherit;
      line-height: var(--leading-tight);
    }
  `
];

// src/web-components/ship-checkbox.ts
var ShipCheckbox = class extends i4 {
  constructor() {
    super();
    this.defaultChecked = false;
    this.checked = false;
    this.indeterminate = false;
    this.disabled = false;
    this.required = false;
    this.invalid = false;
    this.size = "md";
    this.name = "";
    this.value = "on";
    this.label = "";
    this.onChange = (e6) => {
      const c4 = e6.target.checked;
      this.checked = c4;
      this.indeterminate = false;
      this.setFormValue(c4 ? this.value : null);
      this.dispatchEvent(new CustomEvent("ship-change", {
        detail: { checked: c4 },
        bubbles: true,
        composed: true
      }));
    };
    this.internals = this.attachInternals();
  }
  get input() {
    return this.renderRoot.querySelector("input");
  }
  connectedCallback() {
    super.connectedCallback();
    this.defaultChecked = this.checked;
  }
  updated() {
    if (this.input) this.input.indeterminate = this.indeterminate;
    this.setFormValue(this.checked ? this.value : null);
  }
  formResetCallback() {
    this.checked = this.defaultChecked;
    this.indeterminate = false;
  }
  formStateRestoreCallback(state) {
    this.checked = state === this.value;
  }
  setFormValue(v2) {
    if (typeof this.internals.setFormValue === "function") {
      this.internals.setFormValue(v2);
    }
  }
  render() {
    const ariaChecked = this.indeterminate ? "mixed" : this.checked ? "true" : "false";
    return b2`
      <label part="label">
        <span class="box" part="box">
          <input
            part="input"
            type="checkbox"
            .checked=${this.checked}
            ?disabled=${this.disabled}
            ?required=${this.required}
            name=${this.name}
            value=${this.value}
            aria-checked=${ariaChecked}
            aria-invalid=${this.invalid ? "true" : "false"}
            aria-label=${this.label || this.name || "checkbox"}
            @change=${this.onChange}
          />
          <span class="glyph check" aria-hidden="true">
            <svg viewBox="0 0 16 16" fill="none">
              <path d="M3.5 8.5 L6.75 11.75 L12.5 4.75"></path>
            </svg>
          </span>
          <span class="glyph mixed" aria-hidden="true">
            <svg viewBox="0 0 16 16" fill="none">
              <line x1="4" y1="8" x2="12" y2="8"></line>
            </svg>
          </span>
        </span>
        <slot></slot>
      </label>
    `;
  }
};
ShipCheckbox.formAssociated = true;
ShipCheckbox.styles = shipCheckboxStyles;
ShipCheckbox.properties = {
  checked: { type: Boolean, reflect: true },
  indeterminate: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true },
  required: { type: Boolean, reflect: true },
  invalid: { type: Boolean, reflect: true },
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true },
  name: { type: String },
  value: { type: String },
  label: { type: String }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-checkbox")) {
  customElements.define("ship-checkbox", ShipCheckbox);
}

// src/web-components/ship-collapsible.styles.ts
var shipCollapsibleStyles = [
  reducedMotion,
  i`
    :host {
      display: block;
      font-family: var(--font-body);
      color: var(--foreground);
    }
    :host([hidden]) { display: none; }

    .content-wrap {
      display: grid;
      grid-template-rows: 0fr;
      transition: grid-template-rows var(--duration-normal, 200ms) var(--ease-out);
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
  `
];

// src/web-components/ship-collapsible.ts
var _ShipCollapsible = class _ShipCollapsible extends i4 {
  constructor() {
    super(...arguments);
    this.open = false;
    this.disabled = false;
    this.uid = ++_ShipCollapsible._idSeq;
    this.toggle = () => {
      if (this.disabled) return;
      this.open = !this.open;
      this.dispatchEvent(new CustomEvent("ship-toggle", {
        detail: { open: this.open },
        bubbles: true,
        composed: true
      }));
    };
    this.onTriggerKeydown = (e6) => {
      if (e6.key === "Enter" || e6.key === " ") {
        e6.preventDefault();
        this.toggle();
      }
    };
    this.onTriggerSlotChange = (e6) => {
      const slot = e6.target;
      for (const el of slot.assignedElements()) {
        const h3 = el;
        h3.setAttribute("aria-expanded", String(this.open));
        h3.setAttribute("aria-controls", this.contentId);
        if (!h3.hasAttribute("tabindex") && !h3.matches("button, a, input, select, textarea")) {
          h3.setAttribute("tabindex", "0");
        }
      }
    };
  }
  get contentId() {
    return `ship-collapsible-${this.uid}`;
  }
  updated(changed) {
    if (changed.has("open")) {
      const slot = this.renderRoot.querySelector("slot[name=trigger]");
      if (slot) {
        for (const el of slot.assignedElements()) {
          el.setAttribute("aria-expanded", String(this.open));
        }
      }
    }
  }
  render() {
    return b2`
      <div @click=${this.toggle} @keydown=${this.onTriggerKeydown}>
        <slot name="trigger" @slotchange=${this.onTriggerSlotChange}></slot>
      </div>
      <div class="content-wrap" part="content"
        id=${this.contentId}
        role="region"
      >
        <div class="content-inner">
          <slot></slot>
        </div>
      </div>
    `;
  }
};
_ShipCollapsible.styles = shipCollapsibleStyles;
_ShipCollapsible.properties = {
  open: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true }
};
_ShipCollapsible._idSeq = 0;
var ShipCollapsible = _ShipCollapsible;
if (typeof customElements !== "undefined" && !customElements.get("ship-collapsible")) {
  customElements.define("ship-collapsible", ShipCollapsible);
}

// src/web-components/typeahead.ts
var Typeahead = class {
  constructor(timeout = 500) {
    this.timeout = timeout;
    this.buffer = "";
    this.lastAt = 0;
  }
  push(key) {
    const now = Date.now();
    if (now - this.lastAt > this.timeout) this.buffer = "";
    this.lastAt = now;
    const k2 = key.toLowerCase();
    if (this.buffer === k2) return this.buffer;
    this.buffer += k2;
    return this.buffer;
  }
  reset() {
    this.buffer = "";
    this.lastAt = 0;
  }
  /**
   * Find the next matching item index. Items provide their search label.
   * `fromIndex` is the currently-active index (matches start strictly
   * after it). Buffer of length 1 cycles; longer buffers prefix-match.
   */
  match(labels, fromIndex) {
    const n4 = labels.length;
    if (!this.buffer || n4 === 0) return -1;
    const cycling = this.buffer.length === 1;
    const start = cycling ? fromIndex + 1 : 0;
    for (let i6 = 0; i6 < n4; i6++) {
      const idx = (start + i6) % n4;
      const label = (labels[idx] ?? "").toLowerCase();
      if (label.startsWith(this.buffer)) return idx;
    }
    return -1;
  }
};
function isTypeaheadKey(e6) {
  if (e6.ctrlKey || e6.altKey || e6.metaKey) return false;
  return e6.key.length === 1 && /\S/.test(e6.key);
}

// src/web-components/ship-context-menu.ts
var ShipContextMenu = class extends i4 {
  constructor() {
    super(...arguments);
    this.open = false;
    this.typeahead = new Typeahead();
    this.menuX = 0;
    this.menuY = 0;
    this.onContextMenu = (e6) => {
      e6.preventDefault();
      this.show(e6.clientX, e6.clientY);
    };
    this.onItemClick = (e6) => {
      const target = e6.target.closest("[data-value]");
      if (!target) return;
      this.dispatchEvent(new CustomEvent("ship-select", {
        detail: { value: target.dataset.value || "" },
        bubbles: true,
        composed: true
      }));
      this.close();
    };
    this.onKeydown = (e6) => {
      if (!this.open) return;
      const items = this.items();
      if (!items.length) return;
      const i6 = this.currentIndex();
      if (e6.key === "ArrowDown") {
        e6.preventDefault();
        this.focusItem(i6 + 1);
      } else if (e6.key === "ArrowUp") {
        e6.preventDefault();
        this.focusItem(i6 - 1);
      } else if (e6.key === "Home") {
        e6.preventDefault();
        this.focusItem(0);
      } else if (e6.key === "End") {
        e6.preventDefault();
        this.focusItem(items.length - 1);
      } else if (e6.key === "Escape") {
        e6.preventDefault();
        this.close();
      } else if (e6.key === "Tab") {
        this.close();
      } else if ((e6.key === "Enter" || e6.key === " ") && i6 >= 0) {
        e6.preventDefault();
        items[i6].click();
      } else if (isTypeaheadKey(e6)) {
        this.typeahead.push(e6.key);
        const labels = items.map((el) => (el.textContent ?? "").trim());
        const next = this.typeahead.match(labels, i6);
        if (next >= 0) {
          e6.preventDefault();
          this.focusItem(next);
        }
      }
    };
    this.onDocClick = (e6) => {
      if (!this.open) return;
      const target = e6.composedPath()[0];
      if (!this.contains(target) && target !== this) this.close();
    };
  }
  items() {
    return Array.from(
      this.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')
    ).filter((el) => !el.hasAttribute("disabled"));
  }
  focusItem(index) {
    const items = this.items();
    if (!items.length) return;
    const i6 = (index + items.length) % items.length;
    for (const it of items) it.setAttribute("tabindex", "-1");
    items[i6].setAttribute("tabindex", "0");
    items[i6].focus();
  }
  currentIndex() {
    const items = this.items();
    return items.indexOf(document.activeElement);
  }
  setupItems() {
    const items = this.items();
    items.forEach((el, idx) => {
      if (!el.hasAttribute("role")) el.setAttribute("role", "menuitem");
      if (!el.hasAttribute("tabindex")) el.setAttribute("tabindex", idx === 0 ? "0" : "-1");
    });
  }
  show(x2, y3) {
    this.menuX = x2;
    this.menuY = y3;
    this.open = true;
    this.emit("ship-open");
    queueMicrotask(() => {
      this.setupItems();
      this.focusItem(0);
    });
  }
  close() {
    this.open = false;
    this.emit("ship-close");
  }
  emit(name) {
    this.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }));
  }
  connectedCallback() {
    super.connectedCallback();
    document.addEventListener("click", this.onDocClick);
    this.addEventListener("keydown", this.onKeydown);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    document.removeEventListener("click", this.onDocClick);
    this.removeEventListener("keydown", this.onKeydown);
  }
  updated() {
    this.setAttribute("data-state", this.open ? "open" : "closed");
    const menu = this.renderRoot.querySelector(".menu");
    if (menu && this.open) {
      menu.style.left = `${this.menuX}px`;
      menu.style.top = `${this.menuY}px`;
    }
    this.setupItems();
  }
  render() {
    return b2`
      <span @contextmenu=${this.onContextMenu}><slot name="trigger"></slot></span>
      <div class="menu" part="menu" role="menu"
        aria-label="Context menu"
        @click=${this.onItemClick}>
        <slot @slotchange=${() => this.setupItems()}></slot>
      </div>
    `;
  }
};
ShipContextMenu.styles = [reducedMotion, i`
    :host { display: contents; }
    .menu {
      position: fixed;
      z-index: var(--z-dropdown);
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
      transform: scale(0.95);
      pointer-events: none;
      transition:
        opacity var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }
    :host([open]) .menu {
      opacity: 1;
      transform: scale(1);
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
    @media (forced-colors: active) {
      .menu { border: 1px solid CanvasText; }
    }
  `];
ShipContextMenu.properties = {
  open: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-context-menu")) {
  customElements.define("ship-context-menu", ShipContextMenu);
}

// src/web-components/ship-dropdown-menu.ts
var ShipDropdownMenu = class extends i4 {
  constructor() {
    super(...arguments);
    this.open = false;
    this.typeahead = new Typeahead();
    this.onTriggerClick = () => {
      this.open = !this.open;
      if (this.open) queueMicrotask(() => this.focusItem(0));
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
      this.close();
    };
    this.onKeydown = (e6) => {
      if (!this.open) return;
      const items = this.items();
      if (!items.length) return;
      const i6 = this.currentIndex();
      if (e6.key === "ArrowDown") {
        e6.preventDefault();
        this.focusItem(i6 + 1);
      } else if (e6.key === "ArrowUp") {
        e6.preventDefault();
        this.focusItem(i6 - 1);
      } else if (e6.key === "Home") {
        e6.preventDefault();
        this.focusItem(0);
      } else if (e6.key === "End") {
        e6.preventDefault();
        this.focusItem(items.length - 1);
      } else if (e6.key === "Escape") {
        e6.preventDefault();
        this.close();
      } else if (e6.key === "Tab") {
        this.open = false;
      } else if ((e6.key === "Enter" || e6.key === " ") && i6 >= 0) {
        e6.preventDefault();
        items[i6].click();
      } else if (isTypeaheadKey(e6)) {
        this.typeahead.push(e6.key);
        const labels = items.map((el) => (el.textContent ?? "").trim());
        const next = this.typeahead.match(labels, i6);
        if (next >= 0) {
          e6.preventDefault();
          this.focusItem(next);
        }
      }
    };
    this.onDocClick = (e6) => {
      if (!this.open) return;
      const target = e6.composedPath()[0];
      if (!this.contains(target) && target !== this) this.open = false;
    };
  }
  items() {
    return Array.from(this.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((el) => !el.hasAttribute("disabled"));
  }
  focusItem(index) {
    const items = this.items();
    if (!items.length) return;
    const i6 = (index + items.length) % items.length;
    for (const it of items) it.setAttribute("tabindex", "-1");
    items[i6].setAttribute("tabindex", "0");
    items[i6].focus();
  }
  currentIndex() {
    const items = this.items();
    return items.indexOf(document.activeElement);
  }
  triggerEl() {
    const slot = this.renderRoot.querySelector('slot[name="trigger"]');
    const assigned = slot?.assignedElements({ flatten: true });
    return assigned?.[0] ?? null;
  }
  close() {
    this.open = false;
    this.triggerEl()?.focus();
  }
  setupTrigger() {
    const t4 = this.triggerEl();
    if (!t4) return;
    t4.setAttribute("aria-haspopup", "menu");
    t4.setAttribute("aria-expanded", this.open ? "true" : "false");
  }
  setupItems() {
    const items = this.items();
    items.forEach((el, idx) => {
      if (!el.hasAttribute("role")) el.setAttribute("role", "menuitem");
      if (!el.hasAttribute("tabindex")) el.setAttribute("tabindex", idx === 0 ? "0" : "-1");
    });
  }
  updated() {
    this.setAttribute("data-state", this.open ? "open" : "closed");
    this.setupTrigger();
    this.setupItems();
  }
  connectedCallback() {
    super.connectedCallback();
    document.addEventListener("click", this.onDocClick);
    this.addEventListener("keydown", this.onKeydown);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    document.removeEventListener("click", this.onDocClick);
    this.removeEventListener("keydown", this.onKeydown);
  }
  render() {
    return b2`
      <span @click=${this.onTriggerClick}><slot name="trigger" @slotchange=${() => this.setupTrigger()}></slot></span>
      <div class="menu" role="menu" @click=${this.onItemClick}>
        <slot @slotchange=${() => this.setupItems()}></slot>
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

// src/web-components/ship-empty-state.styles.ts
var shipEmptyStateStyles = [
  reducedMotion,
  toneVars,
  i`
    :host {
      display: block;
      font-family: var(--font-body);
      color: var(--muted-foreground);

      /* Size channel — overridden per :host([size=…]) below. md is the
       * default and matches the pre-polish 64px/16px card so every existing
       * caller keeps the same visual weight. --es-icon sizes the inner
       * glyph; --es-tile sizes the circular wrapper around it. Keeping
       * those on separate channels means the tile can stay generous while
       * the glyph itself reads at the same optical size as the copy. */
      --es-pad-y: 64px;
      --es-pad-x: 16px;
      --es-gap: var(--space-3);
      --es-icon: 22px;
      --es-tile: 48px;
      --es-title-fs: var(--text-md);
      --es-desc-fs: var(--text-sm);
      --es-title-weight: var(--font-weight-semibold);

      /* Variant channel — default is the dashed card; inline/placeholder/
       * connecting override below. */
      --es-radius: var(--radius-lg);
      --es-border: 1px dashed var(--border);
      --es-bg: var(--card);
    }
    :host([hidden]) { display: none; }

    /* ------------------------------------------------------------------
     * Size channel. Padding scales roughly 1.4× between steps so lg feels
     * hero-sized without dwarfing its container; sm tightens to fit list
     * sub-sections. Icon + type sizes travel together so the composition
     * stays optically balanced across sizes.
     * ------------------------------------------------------------------ */
    :host([size='sm']) {
      --es-pad-y: var(--space-7);
      --es-pad-x: var(--space-4);
      --es-gap: var(--space-2);
      --es-icon: 16px;
      --es-tile: 36px;
      --es-title-fs: var(--text-base);
      --es-desc-fs: var(--text-sm);
    }
    :host([size='lg']) {
      --es-pad-y: 96px;
      --es-pad-x: var(--space-6);
      --es-gap: var(--space-4);
      --es-icon: 28px;
      --es-tile: 64px;
      --es-title-fs: var(--text-lg);
      --es-desc-fs: var(--text-base);
    }

    /* ------------------------------------------------------------------
     * Variant channel. Every variant only touches surface + radius +
     * border + internal padding density. Typography and icon sizing stay
     * controlled by the size channel, so size=lg variant=placeholder and
     * size=lg variant=default share the exact same type scale.
     * ------------------------------------------------------------------ */
    :host([variant='placeholder']),
    :host([variant='connecting']) {
      --es-border: 0;
      --es-bg: transparent;
      --es-radius: 0;
      --es-pad-y: var(--space-6);
    }
    :host([variant='placeholder'][size='sm']),
    :host([variant='connecting'][size='sm']) {
      --es-pad-y: var(--space-4);
    }
    :host([variant='placeholder'][size='lg']),
    :host([variant='connecting'][size='lg']) {
      --es-pad-y: var(--space-8);
    }

    :host([variant='inline']) {
      --es-border: 1px dashed var(--border);
      --es-bg: color-mix(in oklch, var(--muted) 55%, transparent);
      --es-radius: var(--radius-md);
      --es-pad-y: var(--space-3);
      --es-pad-x: var(--space-4);
      --es-gap: var(--space-3);
    }

    /* ------------------------------------------------------------------
     * Box — the outer chrome. Flex column by default; inline flips to a
     * horizontal row with left-aligned text so it slots cleanly into list
     * sub-sections without forcing a full-bleed empty card.
     * ------------------------------------------------------------------ */
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
      animation: es-in var(--duration-medium) var(--ease-out) both;
    }
    :host([variant='inline']) .box {
      flex-direction: row;
      align-items: center;
      justify-content: flex-start;
      text-align: left;
      gap: var(--space-3);
    }

    @keyframes es-in {
      from { opacity: 0; transform: translateY(4px); }
      to   { opacity: 1; transform: translateY(0); }
    }

    /* ------------------------------------------------------------------
     * Icon tile. The tile is a muted circle wrapping the glyph — reads
     * as deliberate, not just "missing content". sm uses a 36px tile,
     * md 48px, lg 64px. Inline variant drops the tile so the icon sits
     * bare beside the title in dense row chrome.
     * ------------------------------------------------------------------ */
    .icon {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      width: var(--es-tile);
      height: var(--es-tile);
      border-radius: var(--radius-full);
      /* Tile wash reads deeper than a plain muted surface so the tone is
       * legible without overpowering the surrounding card. Neutral tones
       * fall back to plain muted via the explicit selector below. */
      background: color-mix(in oklch, var(--tone-fg) 18%, var(--muted));
      color: var(--tone-fg);
      line-height: 1;
    }
    :host(:not([tone])) .icon,
    :host([tone='default']) .icon {
      background: var(--muted);
      color: var(--muted-foreground);
    }
    .icon:empty,
    .icon[data-empty] { display: none; }
    /* Inline compact row drops the tile — icon sits bare at the icon-channel
     * size so dense row chrome doesn't get a full-height circle. */
    :host([variant='inline']) .icon {
      width: var(--es-icon);
      height: var(--es-icon);
      background: transparent;
    }
    /* Built-in glyph — inline SVG rendered by the component when the
     * icon attr is set and no slot content is present. Sized off the
     * icon channel so it scales with size=[sm|md|lg]. */
    .glyph {
      width: var(--es-icon);
      height: var(--es-icon);
      display: block;
    }
    /* Slotted glyphs get a display + color nudge. We can't reliably set
     * width/height on a slotted <ship-icon> (its internal SVG is pinned
     * to ship-icon's own size attr), so callers using ship-icon as a
     * slot should set size="lg" when pairing with an md/lg empty state.
     * Raw <svg slot="icon"> scales to the icon channel via the direct
     * selector below. */
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

    /* ------------------------------------------------------------------
     * Typography. Title sits foreground-weight, description picks up
     * muted-foreground + an ~40ch max-width so long copy wraps at a
     * comfortable reading measure rather than stretching to the card.
     * ------------------------------------------------------------------ */
    .copy {
      display: flex;
      flex-direction: column;
      gap: 4px;
      align-items: center;
      min-width: 0;
    }
    :host([variant='inline']) .copy {
      align-items: flex-start;
    }
    .title {
      font-size: var(--es-title-fs);
      font-weight: var(--es-title-weight);
      color: var(--foreground);
      line-height: var(--leading-tight);
      letter-spacing: -0.005em;
    }
    .title:empty { display: none; }

    .desc {
      font-size: var(--es-desc-fs);
      line-height: var(--leading-normal);
      max-width: 40ch;
      color: var(--muted-foreground);
    }
    :host([variant='inline']) .desc {
      max-width: 60ch;
    }
    .desc:empty { display: none; }

    /* connecting / placeholder soften the title weight slightly so these
     * quieter states don't compete with actual content headers. */
    :host([variant='connecting']) .title,
    :host([variant='placeholder']) .title {
      font-weight: var(--font-weight-medium);
      color: var(--muted-foreground);
    }

    /* ------------------------------------------------------------------
     * Actions row. Centered below the copy; on inline, pushed to the
     * far right of the row so a CTA slots naturally on the trailing edge.
     * ------------------------------------------------------------------ */
    .actions {
      display: flex;
      gap: var(--space-2);
      justify-content: center;
      margin-top: 4px;
      flex-wrap: wrap;
    }
    .actions:empty { display: none; }
    :host([variant='inline']) .actions {
      margin-top: 0;
      margin-left: auto;
      justify-content: flex-end;
    }

    /* Forced-colors — keep the dashed border visible in high-contrast
     * modes where color-mix'd tokens collapse. */
    @media (forced-colors: active) {
      .box { border-color: CanvasText; }
      .icon { background: Canvas; color: CanvasText; }
    }
  `
];

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
  "message-square-plus": "M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h9M18 3v6M15 6h6",
  moon: "M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z",
  sun: "M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42M12 6a6 6 0 100 12 6 6 0 000-12z",
  menu: "M3 12h18M3 6h18M3 18h18",
  settings: "M12.22 2h-.44a2 2 0 00-2 2v.18a2 2 0 01-1 1.73l-.43.25a2 2 0 01-2 0l-.15-.08a2 2 0 00-2.73.73l-.22.38a2 2 0 00.73 2.73l.15.1a2 2 0 011 1.72v.51a2 2 0 01-1 1.74l-.15.09a2 2 0 00-.73 2.73l.22.38a2 2 0 002.73.73l.15-.08a2 2 0 012 0l.43.25a2 2 0 011 1.73V20a2 2 0 002 2h.44a2 2 0 002-2v-.18a2 2 0 011-1.73l.43-.25a2 2 0 012 0l.15.08a2 2 0 002.73-.73l.22-.39a2 2 0 00-.73-2.73l-.15-.08a2 2 0 01-1-1.74v-.5a2 2 0 011-1.74l.15-.09a2 2 0 00.73-2.73l-.22-.38a2 2 0 00-2.73-.73l-.15.08a2 2 0 01-2 0l-.43-.25a2 2 0 01-1-1.73V4a2 2 0 00-2-2z",
  image: "M3 5a2 2 0 012-2h14a2 2 0 012 2v14a2 2 0 01-2 2H5a2 2 0 01-2-2V5zM8.5 10a1.5 1.5 0 100-3 1.5 1.5 0 000 3zM21 15l-5-5L5 21",
  upload: "M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M17 8l-5-5-5 5M12 3v12",
  download: "M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3",
  refresh: "M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15",
  eye: "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8zM12 9a3 3 0 100 6 3 3 0 000-6z",
  "eye-off": "M17.94 17.94A10.07 10.07 0 0112 20c-7 0-11-8-11-8a18.45 18.45 0 015.06-5.94M9.9 4.24A9.12 9.12 0 0112 4c7 0 11 8 11 8a18.5 18.5 0 01-2.16 3.19M14.12 14.12a3 3 0 11-4.24-4.24M1 1l22 22",
  lock: "M19 11H5a2 2 0 00-2 2v7a2 2 0 002 2h14a2 2 0 002-2v-7a2 2 0 00-2-2zM7 11V7a5 5 0 0110 0v4",
  "log-out": "M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5-5-5M21 12H9",
  users: "M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2M9 7a4 4 0 100 8 4 4 0 000-8zM23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75",
  terminal: "M4 17l6-5-6-5M12 19h8",
  // git-branch uses circles + lines (can't be a single path d)
  git: "M6 3v12",
  "git-branch": "M6 3v12",
  play: "M5 3l14 9-14 9V3z",
  pause: "M6 4h4v16H6zM14 4h4v16h-4z",
  stop: "M6 4h12a2 2 0 012 2v12a2 2 0 01-2 2H6a2 2 0 01-2-2V6a2 2 0 012-2z",
  maximize: "M8 3H5a2 2 0 00-2 2v3M21 8V5a2 2 0 00-2-2h-3M3 16v3a2 2 0 002 2h3M16 21h3a2 2 0 002-2v-3",
  minimize: "M4 14h6v6M20 10h-6V4M14 10l7-7M3 21l7-7",
  zap: "M13 2L3 14h9l-1 8 10-12h-9l1-8z"
};
var COMPLEX_ICONS = {
  git: w`<line x1="6" y1="3" x2="6" y2="15"></line><circle cx="18" cy="6" r="3"></circle><circle cx="6" cy="18" r="3"></circle><path d="M18 9a9 9 0 01-9 9"></path>`,
  "git-branch": w`<line x1="6" y1="3" x2="6" y2="15"></line><circle cx="18" cy="6" r="3"></circle><circle cx="6" cy="18" r="3"></circle><path d="M18 9a9 9 0 01-9 9"></path>`
};
var ShipIcon = class extends i4 {
  constructor() {
    super(...arguments);
    this.name = "";
    this.size = "md";
    this.tone = "default";
    this.label = "";
  }
  updated() {
    if (this.label) {
      this.setAttribute("role", "img");
      this.setAttribute("aria-label", this.label);
      this.removeAttribute("aria-hidden");
    } else {
      this.setAttribute("aria-hidden", "true");
      this.removeAttribute("role");
      this.removeAttribute("aria-label");
    }
  }
  render() {
    const d3 = this.name ? ICONS[this.name] : void 0;
    const complex = this.name ? COMPLEX_ICONS[this.name] : void 0;
    const hidden = this.label ? A : "true";
    return b2`
      <slot>
        ${complex ? b2`<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
              stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              aria-hidden=${hidden} part="svg">${complex}</svg>` : d3 ? b2`<svg viewBox="0 0 24 24" fill="none" stroke="currentColor"
              stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              aria-hidden=${hidden} part="svg">
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
  tone: { type: String, reflect: true },
  label: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-icon")) {
  customElements.define("ship-icon", ShipIcon);
}

// src/web-components/ship-empty-state.ts
var ShipEmptyState = class extends i4 {
  constructor() {
    super(...arguments);
    this.variant = "default";
    this.size = "md";
    this.icon = "";
    /** Track whether the icon slot has projected content so the `icon` attr
     * fallback only renders when the slot is genuinely empty. Slot always
     * wins — matches the charter rule "slots over attrs for content". */
    this.iconSlotFilled = false;
    this.onIconSlotChange = (e6) => {
      const slot = e6.target;
      const filled = slot.assignedNodes({ flatten: true }).some((n4) => {
        if (n4.nodeType === Node.TEXT_NODE) {
          return (n4.textContent || "").trim().length > 0;
        }
        return n4.nodeType === Node.ELEMENT_NODE;
      });
      if (filled !== this.iconSlotFilled) {
        this.iconSlotFilled = filled;
        this.requestUpdate();
      }
    };
  }
  render() {
    const live = this.variant === "connecting" ? "polite" : "off";
    const role = this.variant === "connecting" ? "status" : "note";
    const fallbackPath = this.icon ? ICONS[this.icon] : void 0;
    const hasFallback = !this.iconSlotFilled && !!fallbackPath;
    const tileHasContent = this.iconSlotFilled || hasFallback;
    return b2`
      <div class="box" part="box" role=${role} aria-live=${live}>
        <div
          class="icon"
          part="icon"
          aria-hidden="true"
          ?data-empty=${!tileHasContent}
        >
          <slot name="icon" @slotchange=${this.onIconSlotChange}>
            ${hasFallback ? b2`<svg
                  class="glyph"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.75"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <path d=${fallbackPath}></path>
                </svg>` : A}
          </slot>
        </div>

        <div class="copy">
          <div class="title" part="title">
            <slot name="title"></slot>
          </div>
          <div class="desc" part="desc">
            <slot></slot>
          </div>
        </div>

        <div class="actions" part="actions">
          <slot name="action"></slot>
        </div>
      </div>
    `;
  }
};
ShipEmptyState.styles = shipEmptyStateStyles;
ShipEmptyState.properties = {
  variant: { type: String, reflect: true },
  size: { type: String, reflect: true },
  tone: { type: String, reflect: true },
  icon: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-empty-state")) {
  customElements.define("ship-empty-state", ShipEmptyState);
}

// src/web-components/ship-field.ts
var _ShipField = class _ShipField extends i4 {
  constructor() {
    super(...arguments);
    this.invalid = false;
    this.uid = ++_ShipField._idSeq;
    this.hintId = `ship-field-hint-${this.uid}`;
    this.errorId = `ship-field-error-${this.uid}`;
    this.wireControl = () => {
      const slot = this.renderRoot.querySelector("slot:not([name])");
      const first = slot?.assignedElements({ flatten: true })[0];
      if (!first) return;
      const ids = [];
      if (this.invalid) ids.push(this.errorId);
      else ids.push(this.hintId);
      const prior = (first.getAttribute("aria-describedby") || "").split(/\s+/).filter((id) => id && id !== this.hintId && id !== this.errorId);
      const final = [...prior, ...ids].join(" ").trim();
      if (final) first.setAttribute("aria-describedby", final);
      if ("invalid" in first) first.invalid = this.invalid;
      else first.setAttribute("aria-invalid", this.invalid ? "true" : "false");
    };
  }
  updated() {
    this.wireControl();
  }
  render() {
    return b2`
      <slot name="label"></slot>
      <slot @slotchange=${this.wireControl}></slot>
      <div id=${this.hintId} class="hint"><slot name="hint"></slot></div>
      <div id=${this.errorId} class="error" role="alert" aria-live="polite"><slot name="error"></slot></div>
    `;
  }
};
_ShipField.styles = i`
    :host { display: flex; flex-direction: column; gap: 6px; }
    .hint, .error { font-size: 11px; line-height: 1.4; }
    .hint { color: var(--muted-foreground); }
    .error { color: var(--destructive); }
    :host(:not([invalid])) .error { display: none; }
    :host([invalid]) .hint { display: none; }
  `;
_ShipField.properties = {
  invalid: { type: Boolean, reflect: true }
};
_ShipField._idSeq = 0;
var ShipField = _ShipField;
if (typeof customElements !== "undefined" && !customElements.get("ship-field")) {
  customElements.define("ship-field", ShipField);
}

// src/web-components/ship-file-upload.ts
function fmtSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1048576).toFixed(1)} MB`;
}
var ShipFileUpload = class extends i4 {
  constructor() {
    super(...arguments);
    this.accept = "";
    this.multiple = false;
    this.maxSize = 10485760;
    this.disabled = false;
    this._files = [];
    this._over = false;
    this._error = "";
  }
  get _inputEl() {
    return this.renderRoot.querySelector('input[type="file"]');
  }
  _open() {
    if (this.disabled) return;
    this._inputEl?.click();
  }
  _onKey(e6) {
    if (e6.key === "Enter" || e6.key === " ") {
      e6.preventDefault();
      this._open();
    }
  }
  _onDragOver(e6) {
    e6.preventDefault();
    this._over = true;
  }
  _onDragLeave() {
    this._over = false;
  }
  _onDrop(e6) {
    e6.preventDefault();
    this._over = false;
    if (e6.dataTransfer?.files) this._addFiles(Array.from(e6.dataTransfer.files));
  }
  _onInputChange(e6) {
    const input = e6.target;
    if (input.files) this._addFiles(Array.from(input.files));
    input.value = "";
  }
  _addFiles(incoming) {
    this._error = "";
    const valid = [];
    for (const f3 of incoming) {
      if (f3.size > this.maxSize) {
        this._error = `"${f3.name}" exceeds max size (${fmtSize(this.maxSize)}).`;
        continue;
      }
      if (this.accept && !this._matchesAccept(f3)) {
        this._error = `"${f3.name}" is not an accepted file type.`;
        continue;
      }
      valid.push(f3);
    }
    this._files = this.multiple ? [...this._files, ...valid] : valid.slice(0, 1);
    this._emit();
  }
  _matchesAccept(file) {
    const tokens = this.accept.split(",").map((t4) => t4.trim().toLowerCase());
    const ext = "." + file.name.split(".").pop()?.toLowerCase();
    return tokens.some((t4) => {
      if (t4.startsWith(".")) return ext === t4;
      if (t4.endsWith("/*")) return file.type.startsWith(t4.replace("/*", "/"));
      return file.type === t4;
    });
  }
  _remove(idx) {
    this._files = this._files.filter((_2, i6) => i6 !== idx);
    this._error = "";
    this._emit();
  }
  _emit() {
    this.dispatchEvent(new CustomEvent("ship-change", {
      detail: { files: [...this._files] },
      bubbles: true,
      composed: true
    }));
  }
  render() {
    return b2`
      <div
        class="dropzone ${this._over ? "over" : ""}"
        role="button"
        tabindex=${this.disabled ? "-1" : "0"}
        aria-label=${this.accept ? `Upload files (${this.accept})` : "Upload files"}
        @click=${this._open}
        @keydown=${this._onKey}
        @dragover=${this._onDragOver}
        @dragleave=${this._onDragLeave}
        @drop=${this._onDrop}
      >
        <span class="icon" aria-hidden="true">\u{1F4C4}</span>
        <span>Drop files here or click to browse</span>
      </div>
      <input
        type="file"
        accept=${this.accept || A}
        ?multiple=${this.multiple}
        @change=${this._onInputChange}
      />
      ${this._error ? b2`<div class="error" role="alert">${this._error}</div>` : A}
      ${this._files.length ? b2`<div class="file-list" aria-live="polite">
            ${this._files.map(
      (f3, i6) => b2`
                <div class="file-item">
                  <span class="file-name">${f3.name}</span>
                  <span class="file-size">${fmtSize(f3.size)}</span>
                  <button
                    class="remove"
                    aria-label="Remove ${f3.name}"
                    @click=${() => this._remove(i6)}
                  >&times;</button>
                </div>`
    )}
          </div>` : A}
    `;
  }
};
ShipFileUpload.styles = [
  toneVars,
  reducedMotion,
  i`
      :host { display: block; }
      :host([disabled]) { opacity: 0.5; pointer-events: none; }

      .dropzone {
        display:flex;flex-direction:column;align-items:center;justify-content:center;
        gap:8px;padding:24px 16px;border:2px dashed var(--border);
        border-radius:var(--radius);background:transparent;cursor:pointer;
        transition:border-color .15s,background .15s;
        color:var(--muted-foreground);font-size:.875rem;
      }
      .dropzone:focus-visible { outline:2px solid var(--ring);outline-offset:2px; }
      .dropzone.over { border-style:solid;border-color:var(--tone-border);background:var(--tone-bg); }
      .icon { font-size:1.5rem;line-height:1; }
      input[type="file"] { display:none; }
      .file-list { display:flex;flex-direction:column;gap:4px;margin-top:8px; }
      .file-item {
        display:flex;align-items:center;gap:8px;padding:6px 10px;
        border-radius:var(--radius-sm);background:var(--muted);
        font-size:.8125rem;color:var(--foreground);
      }
      .file-name { flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
      .file-size { color:var(--muted-foreground);flex-shrink:0; }
      .remove {
        appearance:none;border:none;background:none;cursor:pointer;
        color:var(--muted-foreground);padding:2px;line-height:1;
        border-radius:var(--radius-sm);font-size:1rem;
      }
      .remove:hover { color:var(--foreground);background:var(--border); }
      .error { font-size:.75rem;color:var(--destructive);margin-top:4px;padding:0 2px; }
    `
];
ShipFileUpload.properties = {
  accept: { type: String },
  multiple: { type: Boolean },
  maxSize: { type: Number, attribute: "max-size" },
  disabled: { type: Boolean, reflect: true },
  tone: { type: String, reflect: true },
  _files: { state: true },
  _over: { state: true },
  _error: { state: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-file-upload")) {
  customElements.define("ship-file-upload", ShipFileUpload);
}

// src/web-components/ship-hover-card.ts
var _ShipHoverCard = class _ShipHoverCard extends i4 {
  constructor() {
    super(...arguments);
    this.open = false;
    this.placement = "bottom";
    this.showDelay = 300;
    this.hideDelay = 100;
    this.cardId = `ship-hover-card-${++_ShipHoverCard._idSeq}`;
    this.showTimer = null;
    this.hideTimer = null;
    this.onMouseEnter = () => {
      this.clearTimers();
      this.showTimer = setTimeout(() => this.show(), this.showDelay);
    };
    this.onMouseLeave = () => {
      this.clearTimers();
      this.hideTimer = setTimeout(() => this.hide(), this.hideDelay);
    };
    this.onFocusIn = () => {
      this.clearTimers();
      this.show();
    };
    this.onFocusOut = () => {
      this.clearTimers();
      this.hideTimer = setTimeout(() => this.hide(), this.hideDelay);
    };
    this.wireTrigger = () => {
      const slot = this.renderRoot.querySelector('slot[name="trigger"]');
      const first = slot?.assignedElements({ flatten: true })[0];
      if (!first) return;
      first.setAttribute("aria-describedby", this.cardId);
    };
  }
  show() {
    this.clearTimers();
    this.open = true;
    this.emit("ship-open");
  }
  hide() {
    this.clearTimers();
    this.open = false;
    this.emit("ship-close");
  }
  emit(name) {
    this.dispatchEvent(new CustomEvent(name, { bubbles: true, composed: true }));
  }
  clearTimers() {
    if (this.showTimer) {
      clearTimeout(this.showTimer);
      this.showTimer = null;
    }
    if (this.hideTimer) {
      clearTimeout(this.hideTimer);
      this.hideTimer = null;
    }
  }
  updated() {
    this.setAttribute("data-state", this.open ? "open" : "closed");
  }
  connectedCallback() {
    super.connectedCallback();
    this.addEventListener("mouseenter", this.onMouseEnter);
    this.addEventListener("mouseleave", this.onMouseLeave);
    this.addEventListener("focusin", this.onFocusIn);
    this.addEventListener("focusout", this.onFocusOut);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.clearTimers();
    this.removeEventListener("mouseenter", this.onMouseEnter);
    this.removeEventListener("mouseleave", this.onMouseLeave);
    this.removeEventListener("focusin", this.onFocusIn);
    this.removeEventListener("focusout", this.onFocusOut);
  }
  render() {
    return b2`
      <slot name="trigger" @slotchange=${this.wireTrigger}></slot>
      <div id=${this.cardId} class="card" part="card" role="tooltip"
        aria-hidden=${this.open ? "false" : "true"}>
        <div class="arrow"></div>
        <slot></slot>
      </div>
    `;
  }
};
_ShipHoverCard.styles = [reducedMotion, i`
    :host { position: relative; display: inline-flex; }
    .card {
      position: absolute;
      z-index: var(--z-popover);
      min-width: 240px;
      max-width: 340px;
      padding: 16px;
      background: var(--popover);
      color: var(--popover-foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius-lg);
      box-shadow: var(--shadow-lg);
      font-family: var(--font-body);
      font-size: var(--text-base);
      line-height: var(--leading-normal);
      opacity: 0;
      transform: translateY(-4px);
      pointer-events: none;
      transition:
        opacity var(--duration-medium) var(--ease-out),
        transform var(--duration-medium) var(--ease-out);
    }
    :host([open]) .card {
      opacity: 1;
      transform: translateY(0);
      pointer-events: auto;
    }
    /* Arrow */
    .arrow {
      position: absolute;
      width: 10px;
      height: 10px;
      background: var(--popover);
      border: 1px solid var(--border);
      transform: rotate(45deg);
      z-index: -1;
    }
    /* Placement: bottom (default) */
    :host([placement='bottom']) .card,
    :host(:not([placement])) .card { top: calc(100% + 8px); left: 50%; transform: translateX(-50%); }
    :host([placement='bottom'][open]) .card,
    :host(:not([placement])[open]) .card { transform: translateX(-50%) translateY(0); }
    :host([placement='bottom']) .arrow,
    :host(:not([placement])) .arrow {
      top: -6px; left: 50%; transform: translateX(-50%) rotate(45deg);
      border-bottom: none; border-right: none;
    }
    /* Placement: top */
    :host([placement='top']) .card { bottom: calc(100% + 8px); left: 50%; transform: translateX(-50%); }
    :host([placement='top'][open]) .card { transform: translateX(-50%) translateY(0); }
    :host([placement='top']) .arrow {
      bottom: -6px; left: 50%; transform: translateX(-50%) rotate(45deg);
      border-top: none; border-left: none;
    }
    /* Placement: right */
    :host([placement='right']) .card { left: calc(100% + 8px); top: 50%; transform: translateY(-50%); }
    :host([placement='right'][open]) .card { transform: translateY(-50%); }
    :host([placement='right']) .arrow {
      left: -6px; top: 50%; transform: translateY(-50%) rotate(45deg);
      border-top: none; border-right: none;
    }
    /* Placement: left */
    :host([placement='left']) .card { right: calc(100% + 8px); top: 50%; transform: translateY(-50%); }
    :host([placement='left'][open]) .card { transform: translateY(-50%); }
    :host([placement='left']) .arrow {
      right: -6px; top: 50%; transform: translateY(-50%) rotate(45deg);
      border-bottom: none; border-left: none;
    }
    @media (forced-colors: active) {
      .card { border: 1px solid CanvasText; }
    }
  `];
_ShipHoverCard.properties = {
  open: { type: Boolean, reflect: true },
  placement: { type: String, reflect: true },
  showDelay: { type: Number, attribute: "show-delay" },
  hideDelay: { type: Number, attribute: "hide-delay" }
};
_ShipHoverCard._idSeq = 0;
var ShipHoverCard = _ShipHoverCard;
if (typeof customElements !== "undefined" && !customElements.get("ship-hover-card")) {
  customElements.define("ship-hover-card", ShipHoverCard);
}

// src/web-components/ship-input.styles.ts
var shipInputStyles = [reducedMotion, i`
  :host {
    display: inline-flex;
    width: 100%;
    --tone: var(--primary);
    --tone-fg: var(--primary);
  }
  :host([hidden]) { display: none; }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--ring-color);
    --tone-fg: var(--foreground);
  }
  :host([tone='info'])        { --tone: var(--info);        --tone-fg: var(--info); }
  :host([tone='success'])     { --tone: var(--success);     --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);     --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent);      --tone-fg: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) {
    --in-h: 34px;
    --in-px: 10px;
    --in-gap: var(--space-2);
    --in-fs: var(--text-base);
    --in-lh: 22px;
    --in-fw: var(--font-weight-normal);
    --in-icon: 14px;
    --in-radius: var(--radius-md);
  }
  :host([size='sm']) {
    --in-h: 28px;
    --in-px: 8px;
    --in-gap: 6px;
    --in-fs: var(--text-sm);
    --in-lh: 18px;
    --in-fw: var(--font-weight-normal);
    --in-icon: 12px;
    --in-radius: var(--radius-sm);
  }
  :host([size='lg']) {
    --in-h: 40px;
    --in-px: var(--space-3);
    --in-gap: var(--space-2);
    --in-fs: var(--text-lg);
    --in-lh: 24px;
    --in-fw: var(--font-weight-medium);
    --in-icon: 16px;
    --in-radius: var(--radius-md);
  }

  .field {
    display: inline-flex;
    align-items: center;
    gap: var(--in-gap);
    width: 100%;
    min-height: var(--in-h);
    background: var(--background);
    color: var(--foreground);
    border: 1px solid var(--border);
    border-radius: var(--in-radius);
    padding: 0 var(--in-px);
    box-shadow: var(--shadow-xs);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  /* Hover — subtle border darkening + surface lift. Mirrors shadcn's
   * quiet-input feel: the field acknowledges the pointer without shouting. */
  :host(:not([disabled])) .field:hover {
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-sm);
  }

  /* Focus ring — stacked box-shadow so the ring sits outside the border
   * without reserving layout. Matches ship-button's technique. */
  .field:focus-within {
    outline: none;
    border-color: color-mix(in oklch, var(--tone) 70%, var(--border));
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
  }

  /* Invalid — destructive tint on border + ring. Wins over tone so users
   * can't miss an error state even on info/success-toned inputs. */
  :host([invalid]) .field {
    border-color: color-mix(in oklch, var(--destructive) 65%, var(--border));
  }
  :host([invalid]) .field:hover {
    border-color: color-mix(in oklch, var(--destructive) 80%, var(--border));
  }
  :host([invalid]) .field:focus-within {
    border-color: var(--destructive);
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
  }

  /* Ghost — no chrome at rest, materialize on focus. Keeps the title-input
   * pattern (click-to-edit) without the old border-jump. */
  :host([variant='ghost']) .field {
    background: transparent;
    border-color: transparent;
    box-shadow: none;
  }
  :host([variant='ghost']:not([disabled])) .field:hover {
    background: color-mix(in oklch, var(--muted) 50%, transparent);
    border-color: transparent;
    box-shadow: none;
  }
  :host([variant='ghost']) .field:focus-within {
    background: var(--background);
    border-color: color-mix(in oklch, var(--tone) 70%, var(--border));
  }

  /* Native element — transparent so the field chrome is the surface of
   * truth. font-size is driven off the size channel, never hardcoded. */
  input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: inherit;
    font-family: var(--font-body);
    font-size: var(--in-fs);
    font-weight: var(--in-fw);
    line-height: var(--in-lh);
    letter-spacing: -0.005em;
    padding: 0;
  }
  input::placeholder {
    color: var(--muted-foreground);
    opacity: 1;
  }
  input:-webkit-autofill {
    -webkit-box-shadow: 0 0 0 1000px var(--background) inset;
    -webkit-text-fill-color: var(--foreground);
    caret-color: var(--foreground);
  }

  /* Disabled — dim the whole field and kill pointer affordances. */
  :host([disabled]) .field,
  :host([readonly]) .field { cursor: not-allowed; }
  :host([disabled]) .field {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 70%, transparent);
    box-shadow: none;
    opacity: 0.6;
  }
  input:disabled {
    cursor: not-allowed;
    color: var(--muted-foreground);
  }
  input:read-only { cursor: default; }

  /* Loading spinner — sized off the adornment channel so it sits on the
   * same optical line as a slotted icon. */
  .spinner {
    width: var(--in-icon);
    height: var(--in-icon);
    border: 1.5px solid currentColor;
    border-right-color: transparent;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
    display: inline-block;
    flex-shrink: 0;
    color: var(--muted-foreground);
  }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* Slotted adornments — constrain icons to the size channel so a bare
   * <svg> drops in at the right visual weight without caller intervention. */
  ::slotted([slot='leading']),
  ::slotted([slot='trailing']) {
    display: inline-flex;
    align-items: center;
    flex: 0 0 auto;
    color: var(--muted-foreground);
    font-size: var(--in-fs);
    line-height: 1;
  }
  ::slotted(svg) {
    width: var(--in-icon);
    height: var(--in-icon);
    display: block;
  }

  @media (forced-colors: active) {
    .field:focus-within { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`];

// src/web-components/ship-input.ts
var _ShipInput = class _ShipInput extends i4 {
  constructor() {
    super();
    this.defaultValue = "";
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
    this.label = "";
    this.describedBy = "";
    this.errorMessage = "";
    this.errId = `ship-input-err-${++_ShipInput._idSeq}`;
    this.internals = this.attachInternals();
  }
  connectedCallback() {
    super.connectedCallback();
    this.defaultValue = this.value;
  }
  updated() {
    this.setFormValue(this.value);
  }
  formResetCallback() {
    this.value = this.defaultValue;
  }
  formStateRestoreCallback(state) {
    this.value = state ?? "";
  }
  setFormValue(v2) {
    if (typeof this.internals.setFormValue === "function") this.internals.setFormValue(v2);
  }
  onInput(e6) {
    const target = e6.target;
    this.value = target.value;
    this.setFormValue(this.value);
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
          aria-required=${this.required ? "true" : "false"}
          aria-busy=${this.loading ? "true" : "false"}
          aria-label=${this.label || A}
          aria-describedby=${this.invalid && this.errorMessage ? `${this.errId}${this.describedBy ? " " + this.describedBy : ""}` : this.describedBy || A}
          @input=${this.onInput}
        />
        ${this.loading ? b2`<span class="spinner" aria-hidden="true"></span>` : b2`<slot name="trailing"></slot>`}
      </div>
      ${this.invalid && this.errorMessage ? b2`<span id=${this.errId} role="alert" style="display:none">${this.errorMessage}</span>` : A}
    `;
  }
};
_ShipInput.formAssociated = true;
_ShipInput.styles = shipInputStyles;
_ShipInput.properties = {
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
  tone: { type: String, reflect: true },
  variant: { type: String, reflect: true },
  size: { type: String, reflect: true },
  label: { type: String },
  describedBy: { type: String, attribute: "described-by" },
  errorMessage: { type: String, attribute: "error-message" }
};
_ShipInput._idSeq = 0;
var ShipInput = _ShipInput;
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
        aria-disabled=${this.disabled ? "true" : "false"}
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
    @media (forced-colors: active) {
      button:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
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

// src/web-components/ship-navigation-menu.ts
var ShipNavigationItem = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = "";
    this.active = false;
    this.disabled = false;
    this.href = "";
    this.onClick = (e6) => {
      if (this.disabled) return;
      if (!this.href) e6.preventDefault();
      this.dispatchEvent(new CustomEvent("ship-nav-select", {
        detail: { value: this.value },
        bubbles: true,
        composed: true
      }));
    };
  }
  /** Focus the interactive element — used by parent nav for keyboard nav. */
  focusLink() {
    const el = this.renderRoot.querySelector(".link");
    el?.focus();
  }
  render() {
    if (this.href) {
      return b2`
        <a class="link" part="link"
          href=${this.href}
          aria-current=${this.active ? "page" : "false"}
          @click=${this.onClick}
        ><slot></slot></a>
        <div class="indicator" part="indicator" aria-hidden="true"></div>
      `;
    }
    return b2`
      <button class="link" part="link" type="button"
        aria-current=${this.active ? "true" : "false"}
        ?disabled=${this.disabled}
        @click=${this.onClick}
      ><slot></slot></button>
      <div class="indicator" part="indicator" aria-hidden="true"></div>
    `;
  }
};
ShipNavigationItem.styles = [reducedMotion, i`
    :host { display: inline-flex; position: relative; }
    :host([hidden]) { display: none; }

    .link {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 8px 14px;
      font-family: var(--font-body);
      font-size: var(--text-base);
      font-weight: var(--font-weight-medium);
      color: var(--muted-foreground);
      text-decoration: none;
      background: transparent;
      border: none;
      cursor: pointer;
      transition:
        color var(--duration-fast) var(--ease-out),
        background-color var(--duration-fast) var(--ease-out);
      line-height: var(--leading-normal);
      border-radius: var(--radius);
    }
    .link:hover {
      color: var(--foreground);
      background: color-mix(in oklch, var(--muted) 50%, transparent);
    }
    :host([active]) .link {
      color: var(--foreground);
      font-weight: var(--font-weight-semibold);
    }
    :host([disabled]) .link {
      cursor: not-allowed;
      opacity: 0.5;
      pointer-events: none;
    }
    .link:focus-visible {
      ${focusRingStyle}
    }

    /* Active indicator line */
    .indicator {
      position: absolute;
      bottom: 0;
      left: 14px;
      right: 14px;
      height: 2px;
      background: var(--foreground);
      border-radius: var(--radius-full);
      opacity: 0;
      transform: scaleX(0);
      transition:
        opacity var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }
    :host([active]) .indicator {
      opacity: 1;
      transform: scaleX(1);
    }

    @media (forced-colors: active) {
      .link:focus-visible {
        ${focusRingForcedColors}
      }
      :host([active]) .indicator {
        background: CanvasText;
      }
    }
  `];
ShipNavigationItem.properties = {
  value: { type: String, reflect: true },
  active: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true },
  href: { type: String }
};
var ShipNavigationMenu = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = "";
    this.onNavSelect = (e6) => {
      e6.stopPropagation();
      const val = e6.detail.value;
      this.value = val;
      this.syncItems();
      this.dispatchEvent(new CustomEvent("ship-change", {
        detail: { value: val },
        bubbles: true,
        composed: true
      }));
    };
    this.onKeydown = (e6) => {
      if (e6.key !== "ArrowRight" && e6.key !== "ArrowLeft" && e6.key !== "Home" && e6.key !== "End") return;
      const items = this.enabledItems;
      if (!items.length) return;
      const current = e6.composedPath().find(
        (n4) => items.includes(n4)
      ) || null;
      const i6 = current ? items.indexOf(current) : -1;
      let target;
      if (e6.key === "ArrowRight") target = items[(i6 + 1) % items.length];
      else if (e6.key === "ArrowLeft") target = items[(i6 - 1 + items.length) % items.length];
      else if (e6.key === "Home") target = items[0];
      else if (e6.key === "End") target = items[items.length - 1];
      if (target) {
        e6.preventDefault();
        target.focusLink();
      }
    };
  }
  get navItems() {
    return Array.from(
      this.querySelectorAll(":scope > ship-navigation-item")
    );
  }
  get enabledItems() {
    return this.navItems.filter((i6) => !i6.disabled);
  }
  syncItems() {
    for (const item of this.navItems) {
      item.active = item.value === this.value;
    }
  }
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "navigation");
    this.addEventListener("ship-nav-select", this.onNavSelect);
    this.addEventListener("keydown", this.onKeydown);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("ship-nav-select", this.onNavSelect);
    this.removeEventListener("keydown", this.onKeydown);
  }
  updated() {
    this.syncItems();
  }
  render() {
    return b2`<slot @slotchange=${() => this.syncItems()}></slot>`;
  }
};
ShipNavigationMenu.styles = [reducedMotion, i`
    :host {
      display: flex;
      align-items: stretch;
      gap: 0;
      border-bottom: 1px solid var(--border);
      font-family: var(--font-body);
    }
    :host([hidden]) { display: none; }
  `];
ShipNavigationMenu.properties = {
  value: { type: String, reflect: true }
};
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-navigation-item")) {
    customElements.define("ship-navigation-item", ShipNavigationItem);
  }
  if (!customElements.get("ship-navigation-menu")) {
    customElements.define("ship-navigation-menu", ShipNavigationMenu);
  }
}

// src/web-components/ship-pagination.ts
var ShipPagination = class extends i4 {
  constructor() {
    super(...arguments);
    this.page = 1;
    this.total = 0;
    this.perPage = 25;
  }
  get _totalPages() {
    return this.total > 0 ? Math.ceil(this.total / this.perPage) : 1;
  }
  _emitChange(newPage) {
    this.page = newPage;
    this.dispatchEvent(
      new CustomEvent("ship-change", {
        detail: { page: newPage },
        bubbles: true,
        composed: true
      })
    );
  }
  _prev() {
    if (this.page > 1) this._emitChange(this.page - 1);
  }
  _next() {
    if (this.page < this._totalPages) this._emitChange(this.page + 1);
  }
  render() {
    const tp = this._totalPages;
    const start = (this.page - 1) * this.perPage + 1;
    const end = Math.min(this.page * this.perPage, this.total);
    const toneAttr = this.tone && this.tone !== "default" ? this.tone : void 0;
    return b2`
      <nav class="bar" aria-label="Pagination">
        <ship-button
          variant="outline"
          size="sm"
          tone=${toneAttr ?? ""}
          ?disabled=${this.page <= 1}
          @click=${this._prev}
          aria-label="Previous page"
        >Prev</ship-button>
        <span class="info">
          Page ${this.page} of ${tp}${this.total > 0 ? b2` <span>(${start}\u2013${end} of ${this.total})</span>` : ""}
        </span>
        <span class="spacer"></span>
        <ship-button
          variant="outline"
          size="sm"
          tone=${toneAttr ?? ""}
          ?disabled=${this.page >= tp}
          @click=${this._next}
          aria-label="Next page"
        >Next</ship-button>
      </nav>
    `;
  }
};
ShipPagination.styles = [
  toneVars,
  i`
      :host { display: block; }
      .bar {
        display: flex;
        align-items: center;
        gap: 12px;
        font-size: 0.8125rem;
        font-family: var(--font-sans, sans-serif);
      }
      .info {
        color: var(--muted-foreground);
        user-select: none;
        white-space: nowrap;
      }
      .spacer { flex: 1; }
    `
];
ShipPagination.properties = {
  page: { type: Number, reflect: true },
  total: { type: Number, reflect: true },
  perPage: { type: Number, attribute: "per-page", reflect: true },
  tone: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-pagination")) {
  customElements.define("ship-pagination", ShipPagination);
}

// src/web-components/ship-popover.ts
var _ShipPopover = class _ShipPopover extends i4 {
  constructor() {
    super(...arguments);
    this.open = false;
    this.placement = "bottom";
    this.panelId = `ship-popover-panel-${++_ShipPopover._idSeq}`;
    this.setupTrigger = () => {
      const t4 = this.triggerEl();
      if (!t4) return;
      t4.setAttribute("aria-haspopup", "dialog");
      t4.setAttribute("aria-expanded", this.open ? "true" : "false");
      t4.setAttribute("aria-controls", this.panelId);
    };
    this.onDocClick = (e6) => {
      if (!this.open) return;
      const target = e6.composedPath()[0];
      if (!this.contains(target) && target !== this) this.hide();
    };
    this.onKeydown = (e6) => {
      if (this.open && e6.key === "Escape") {
        e6.preventDefault();
        this.hide();
        this.triggerEl()?.focus();
      }
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
  triggerEl() {
    const slot = this.renderRoot.querySelector('slot[name="trigger"]');
    const assigned = slot?.assignedElements({ flatten: true });
    return assigned?.[0] ?? null;
  }
  updated() {
    this.setAttribute("data-state", this.open ? "open" : "closed");
    this.setupTrigger();
  }
  connectedCallback() {
    super.connectedCallback();
    document.addEventListener("click", this.onDocClick);
    this.addEventListener("keydown", this.onKeydown);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    document.removeEventListener("click", this.onDocClick);
    this.removeEventListener("keydown", this.onKeydown);
  }
  render() {
    return b2`
      <span @click=${() => this.toggle()}><slot name="trigger" @slotchange=${this.setupTrigger}></slot></span>
      <div
        id=${this.panelId}
        class="panel"
        role="dialog"
        aria-modal="false"
        aria-hidden=${this.open ? "false" : "true"}
      ><slot></slot></div>
    `;
  }
};
_ShipPopover.styles = i`
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
_ShipPopover.properties = {
  open: { type: Boolean, reflect: true },
  placement: { type: String, reflect: true }
};
_ShipPopover._idSeq = 0;
var ShipPopover = _ShipPopover;
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
    this.label = "";
  }
  render() {
    const pct = this.value == null ? null : Math.max(0, Math.min(100, this.value / this.max * 100));
    return b2`
      <div class="track" role="progressbar"
        aria-valuemin="0"
        aria-valuemax=${this.max}
        aria-valuenow=${this.value != null ? this.value : A}
        aria-busy=${this.value == null ? "true" : "false"}
        aria-label=${this.label || A}
      >
        <div class="bar" style=${pct != null ? `width: ${pct}%` : ""}></div>
      </div>
    `;
  }
};
ShipProgress.styles = [
  reducedMotion,
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
  size: { type: String, reflect: true },
  label: { type: String }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-progress")) {
  customElements.define("ship-progress", ShipProgress);
}

// src/web-components/ship-radio-group.styles.ts
var shipRadioGroupStyles = [
  reducedMotion,
  i`
    :host {
      display: flex;
      flex-direction: column;
      gap: var(--space-2);
      font-family: var(--font-body);
      color: var(--foreground);
    }
    :host([hidden]) { display: none; }
  `
];
var shipRadioStyles = [
  reducedMotion,
  i`
    :host {
      display: inline-flex;
      align-items: center;
      gap: var(--space-2);
      vertical-align: middle;
      font-family: var(--font-body);
      font-size: var(--text-base);
      color: var(--foreground);
      -webkit-tap-highlight-color: transparent;

      --tone: var(--primary);
    }
    :host([hidden]) { display: none; }

    /* Tone channel — inherited from group or set directly */
    :host(:not([tone])),
    :host([tone='default']) { --tone: var(--primary); }
    :host([tone='info'])        { --tone: var(--info); }
    :host([tone='success'])     { --tone: var(--success); }
    :host([tone='warning'])     { --tone: var(--warning); }
    :host([tone='destructive']) { --tone: var(--destructive); }
    :host([tone='accent'])      { --tone: var(--accent); }
    :host([tone='muted'])       { --tone: var(--muted-foreground); }

    /* Size channel */
    :host(:not([size])),
    :host([size='md']) {
      --rd-size: 16px;
      --rd-dot: 8px;
    }
    :host([size='sm']) {
      --rd-size: 14px;
      --rd-dot: 6px;
    }
    :host([size='lg']) {
      --rd-size: 20px;
      --rd-dot: 10px;
    }

    label {
      display: inline-flex;
      align-items: center;
      gap: var(--space-2);
      cursor: pointer;
      user-select: none;
      -webkit-user-select: none;
    }
    :host([disabled]) label { cursor: not-allowed; }

    /* Circle */
    .circle {
      position: relative;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      width: var(--rd-size);
      height: var(--rd-size);
      background: var(--background);
      border: 1px solid color-mix(in oklch, var(--border) 160%, transparent);
      border-radius: 50%;
      box-sizing: border-box;
      transition:
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
    }

    /* Hover */
    :host(:not([disabled]):not([checked])) label:hover .circle {
      border-color: color-mix(in oklch, var(--border) 220%, transparent);
    }

    /* Checked — border takes tone color */
    :host([checked]) .circle {
      border-color: var(--tone);
    }

    /* Dot */
    .dot {
      width: var(--rd-dot);
      height: var(--rd-dot);
      border-radius: 50%;
      background: var(--tone);
      opacity: 0;
      transform: scale(0);
      transition:
        opacity var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }
    :host([checked]) .dot {
      opacity: 1;
      transform: scale(1);
    }

    /* Native input — visually hidden */
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

    /* Focus ring */
    :host(:not([disabled])) label:has(input:focus-visible) .circle {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }
    @media (forced-colors: active) {
      .circle { border-color: CanvasText; }
      :host([checked]) .dot { background: Highlight; }
      :host(:not([disabled])) label:has(input:focus-visible) .circle {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
    }

    /* Disabled */
    :host([disabled]) {
      color: var(--muted-foreground);
    }
    :host([disabled]) .circle {
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 70%, transparent);
      filter: saturate(0.6);
    }
    :host([disabled][checked]) .circle {
      border-color: color-mix(in oklch, var(--tone) 50%, var(--border));
    }
    :host([disabled][checked]) .dot {
      background: color-mix(in oklch, var(--tone) 55%, var(--muted));
    }

    ::slotted(*) {
      font: inherit;
      color: inherit;
      line-height: var(--leading-tight);
    }
  `
];

// src/web-components/ship-radio-group.ts
var ShipRadio = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = "";
    this.disabled = false;
    this.checked = false;
    this.onClick = () => {
      if (this.disabled) return;
      this.dispatchEvent(new CustomEvent("ship-radio-select", {
        detail: { value: this.value },
        bubbles: true,
        composed: true
      }));
    };
    this.onKeydown = (e6) => {
      const nav = ["ArrowDown", "ArrowRight", "ArrowUp", "ArrowLeft"];
      if (nav.includes(e6.key)) {
        e6.preventDefault();
        const forward = e6.key === "ArrowDown" || e6.key === "ArrowRight";
        this.dispatchEvent(new CustomEvent("ship-radio-nav", {
          detail: { direction: forward ? "next" : "prev" },
          bubbles: true,
          composed: true
        }));
      }
    };
  }
  /** Focus the native input from the group's keyboard handler. */
  focusInput() {
    const input = this.renderRoot.querySelector("input");
    input?.focus();
  }
  render() {
    return b2`
      <label>
        <span class="circle" part="circle">
          <input
            part="input"
            type="radio"
            role="radio"
            .checked=${this.checked}
            ?disabled=${this.disabled}
            aria-checked=${this.checked ? "true" : "false"}
            aria-label=${this.value}
            tabindex=${this.checked ? "0" : "-1"}
            @click=${this.onClick}
            @keydown=${this.onKeydown}
          />
          <span class="dot" part="dot" aria-hidden="true"></span>
        </span>
        <slot></slot>
      </label>
    `;
  }
};
ShipRadio.styles = shipRadioStyles;
ShipRadio.properties = {
  value: { type: String, reflect: true },
  disabled: { type: Boolean, reflect: true },
  checked: { type: Boolean, reflect: true },
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true }
};
var ShipRadioGroup = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = "";
    this.disabled = false;
    this.name = "";
    this.label = "";
    this.onRadioSelect = (e6) => {
      e6.stopPropagation();
      const newValue = e6.detail.value;
      if (newValue === this.value) return;
      this.value = newValue;
      this.syncRadios();
      this.dispatchEvent(new CustomEvent("ship-change", {
        detail: { value: this.value },
        bubbles: true,
        composed: true
      }));
    };
    this.onRadioNav = (e6) => {
      e6.stopPropagation();
      const enabled = this.enabledRadios;
      if (!enabled.length) return;
      const current = e6.target.closest("ship-radio");
      const idx = current ? enabled.indexOf(current) : -1;
      let next;
      if (e6.detail.direction === "next") {
        next = enabled[(idx + 1) % enabled.length];
      } else {
        next = enabled[(idx - 1 + enabled.length) % enabled.length];
      }
      this.value = next.value;
      this.syncRadios();
      next.focusInput();
      this.dispatchEvent(new CustomEvent("ship-change", {
        detail: { value: this.value },
        bubbles: true,
        composed: true
      }));
    };
  }
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "radiogroup");
    if (this.label) this.setAttribute("aria-label", this.label);
    this.addEventListener("ship-radio-select", this.onRadioSelect);
    this.addEventListener("ship-radio-nav", this.onRadioNav);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("ship-radio-select", this.onRadioSelect);
    this.removeEventListener("ship-radio-nav", this.onRadioNav);
  }
  get radios() {
    return Array.from(this.querySelectorAll(":scope > ship-radio"));
  }
  get enabledRadios() {
    return this.radios.filter((r4) => !r4.disabled && !this.disabled);
  }
  updated() {
    this.syncRadios();
  }
  syncRadios() {
    for (const radio of this.radios) {
      radio.checked = radio.value === this.value;
      if (this.tone && !radio.hasAttribute("tone")) {
        radio.tone = this.tone;
      }
      if (this.size && !radio.hasAttribute("size")) {
        radio.size = this.size;
      }
      if (this.disabled) {
        radio.disabled = true;
      }
    }
  }
  render() {
    return b2`<slot></slot>`;
  }
};
ShipRadioGroup.styles = shipRadioGroupStyles;
ShipRadioGroup.properties = {
  value: { type: String, reflect: true },
  disabled: { type: Boolean, reflect: true },
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true },
  name: { type: String },
  label: { type: String }
};
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-radio")) {
    customElements.define("ship-radio", ShipRadio);
  }
  if (!customElements.get("ship-radio-group")) {
    customElements.define("ship-radio-group", ShipRadioGroup);
  }
}

// src/web-components/ship-scroll-area.ts
var ShipScrollArea = class extends i4 {
  constructor() {
    super(...arguments);
    this.orientation = "vertical";
    this.size = "md";
    this.vThumbEl = null;
    this.hThumbEl = null;
    this.scrollTimer = null;
    this.updateThumbs = () => {
      const vp = this.viewportEl;
      if (!vp) return;
      if (this.showV && this.vThumbEl) {
        const ratio = vp.clientHeight / vp.scrollHeight;
        if (ratio < 1) {
          const thumbH = Math.max(24, ratio * vp.clientHeight);
          const scrollRatio = vp.scrollTop / (vp.scrollHeight - vp.clientHeight);
          const thumbTop = scrollRatio * (vp.clientHeight - thumbH);
          this.vThumbEl.style.height = `${thumbH}px`;
          this.vThumbEl.style.top = `${thumbTop}px`;
          this.vThumbEl.style.display = "";
        } else {
          this.vThumbEl.style.display = "none";
        }
      }
      if (this.showH && this.hThumbEl) {
        const ratio = vp.clientWidth / vp.scrollWidth;
        if (ratio < 1) {
          const thumbW = Math.max(24, ratio * vp.clientWidth);
          const scrollRatio = vp.scrollLeft / (vp.scrollWidth - vp.clientWidth);
          const thumbLeft = scrollRatio * (vp.clientWidth - thumbW);
          this.hThumbEl.style.width = `${thumbW}px`;
          this.hThumbEl.style.left = `${thumbLeft}px`;
          this.hThumbEl.style.display = "";
        } else {
          this.hThumbEl.style.display = "none";
        }
      }
    };
    this.onScroll = () => {
      this.updateThumbs();
      const vBar = this.renderRoot.querySelector(".scrollbar-v");
      const hBar = this.renderRoot.querySelector(".scrollbar-h");
      vBar?.classList.add("active");
      hBar?.classList.add("active");
      if (this.scrollTimer) clearTimeout(this.scrollTimer);
      this.scrollTimer = setTimeout(() => {
        vBar?.classList.remove("active");
        hBar?.classList.remove("active");
      }, 1200);
    };
  }
  get viewportEl() {
    return this.renderRoot.querySelector(".viewport");
  }
  get showV() {
    return this.orientation === "vertical" || this.orientation === "both";
  }
  get showH() {
    return this.orientation === "horizontal" || this.orientation === "both";
  }
  firstUpdated() {
    this.vThumbEl = this.renderRoot.querySelector(".scrollbar-v .thumb");
    this.hThumbEl = this.renderRoot.querySelector(".scrollbar-h .thumb");
    const vp = this.viewportEl;
    if (vp) {
      vp.addEventListener("scroll", this.onScroll, { passive: true });
    }
    this.updateThumbs();
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    if (this.scrollTimer) clearTimeout(this.scrollTimer);
  }
  render() {
    return b2`
      <div class="viewport" part="viewport" tabindex="0">
        <slot></slot>
      </div>
      ${this.showV ? b2`
        <div class="scrollbar scrollbar-v" part="scrollbar-v" aria-hidden="true">
          <div class="thumb"></div>
        </div>` : ""}
      ${this.showH ? b2`
        <div class="scrollbar scrollbar-h" part="scrollbar-h" aria-hidden="true">
          <div class="thumb"></div>
        </div>` : ""}
    `;
  }
};
ShipScrollArea.styles = [reducedMotion, i`
    :host {
      display: block;
      position: relative;
      overflow: hidden;
    }
    :host([hidden]) { display: none; }

    /* Size channel for scrollbar thumb width */
    :host(:not([size])), :host([size='md']) { --sb-size: 8px; }
    :host([size='sm']) { --sb-size: 6px; }
    :host([size='lg']) { --sb-size: 10px; }

    .viewport {
      width: 100%;
      height: 100%;
      overflow: hidden;
      scrollbar-width: none;         /* Firefox */
      -ms-overflow-style: none;       /* IE / Edge */
    }
    .viewport::-webkit-scrollbar { display: none; }

    :host([orientation='vertical']) .viewport,
    :host(:not([orientation])) .viewport {
      overflow-y: auto;
    }
    :host([orientation='horizontal']) .viewport {
      overflow-x: auto;
    }
    :host([orientation='both']) .viewport {
      overflow: auto;
    }

    /* Scrollbar tracks — absolute positioned overlays */
    .scrollbar {
      position: absolute;
      opacity: 0;
      transition: opacity var(--duration-medium) var(--ease-out);
      z-index: 1;
    }
    :host(:hover) .scrollbar,
    :host(:focus-within) .scrollbar,
    .scrollbar.active {
      opacity: 1;
    }

    .scrollbar-v {
      top: 0; right: 2px; bottom: 0;
      width: var(--sb-size);
    }
    .scrollbar-h {
      left: 0; right: 0; bottom: 2px;
      height: var(--sb-size);
    }

    /* Thumb */
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

    @media (forced-colors: active) {
      .thumb { background: CanvasText; }
    }
  `];
ShipScrollArea.properties = {
  orientation: { type: String, reflect: true },
  size: { type: String, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-scroll-area")) {
  customElements.define("ship-scroll-area", ShipScrollArea);
}

// src/web-components/ship-select.styles.ts
var shipSelectStyles = [reducedMotion, i`
  :host {
    display: inline-flex;
    vertical-align: middle;
    /* Per-host tone channel — mirrors ship-button's approach so the chrome
     * below can stay tone-agnostic and compose freely. */
    --tone: var(--foreground);
    --tone-fg: var(--foreground);
  }
  :host([hidden]) { display: none; }

  /* Tone channel. Default tone leaves the control neutral; explicit tones
   * colorize the border + focus ring. Foreground text stays readable. */
  :host([tone='info']) {
    --tone: var(--info);
    --tone-fg: var(--info);
  }
  :host([tone='success']) {
    --tone: var(--success);
    --tone-fg: var(--success);
  }
  :host([tone='warning']) {
    --tone: var(--warning);
    --tone-fg: var(--warning);
  }
  :host([tone='destructive']) {
    --tone: var(--destructive);
    --tone-fg: var(--destructive);
  }
  :host([tone='accent']) {
    --tone: var(--accent);
    --tone-fg: var(--accent);
  }
  :host([tone='muted']) {
    --tone: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
  }

  /* Size channel — one source of truth for height/padding/type scale.
   * Matches ship-button visual weight. Values are hardcoded (not pulled
   * from --space-* tokens) because some runtime hosts ship a tokens.css
   * without the spacing scale, and a select that loses its padding is
   * worse than a select that ignores a token the host didn't define. */
  :host(:not([size])),
  :host([size='md']) {
    --sel-h: 34px;
    --sel-px: 12px;
    --sel-fs: 13px;
    --sel-gap: 8px;
    --sel-chev: 14px;
  }
  :host([size='sm']) {
    --sel-h: 28px;
    --sel-px: 10px;
    --sel-fs: 12px;
    --sel-gap: 6px;
    --sel-chev: 12px;
  }
  :host([size='lg']) {
    --sel-h: 40px;
    --sel-px: 16px;
    --sel-fs: 14px;
    --sel-gap: 8px;
    --sel-chev: 16px;
  }

  /* Field wrapper — this is the visible chrome. The real <select> sits
   * on top, fully transparent, so clicks/keyboard/native popup still work
   * without us re-implementing a menu. */
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

  /* Hover — subtle muted wash + border crispening, matching ship-button's
   * default (no-variant) treatment so the two sit next to each other
   * without either looking out of place. */
  :host(:not([disabled])) .field:hover {
    background: color-mix(in oklch, var(--muted) 70%, var(--background));
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-sm);
  }

  /* Toned border when a tone is set (except default). Keeps the surface
   * neutral while still signaling intent on the edge. */
  :host([tone]:not([tone='default'])) .field {
    border-color: color-mix(in oklch, var(--tone) 38%, var(--border));
  }
  :host([tone]:not([tone='default']):not([disabled])) .field:hover {
    background: color-mix(in oklch, var(--tone) 8%, var(--background));
    border-color: color-mix(in oklch, var(--tone) 55%, var(--border));
  }

  /* The real <select> — layered invisibly over the chrome. We use
   * appearance:none to strip OS chrome, color:transparent to hide the
   * rendered glyph, and pointer events stay enabled so the native popup
   * fires. */
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
  }
  /* Keep the chosen option legible in the native popup (OS-chromed). */
  select option { color: var(--foreground); background: var(--background); }

  /* Visible label layer — mirrors the current value on top of the chrome.
   * This is what the user actually reads; the native <select> underneath
   * provides the menu. Sized to content so the host's intrinsic width
   * tracks the current value (matches shadcn's trigger-hugs-label feel). */
  .value {
    display: inline-block;
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
  }
  :host([data-placeholder]) .value {
    color: var(--muted-foreground);
  }

  /* Chevron — SVG in shadow so it always matches currentColor + size.
   * Positioned absolutely so it never contends with the text layout. */
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
  :host(:not([disabled])) .field:hover .chevron {
    color: var(--foreground);
  }
  :host([tone]:not([tone='default'])) .chevron {
    color: color-mix(in oklch, var(--tone-fg) 75%, var(--muted-foreground));
  }

  /* Focus ring — rendered on the wrapper via stacked box-shadows. The
   * native select sits inside, so :focus-within on the host flips this on
   * reliably whether the user tabs in or clicks. */
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

  /* Invalid — destructive border + destructive ring, overrides tone so
   * validation always reads first. */
  :host([invalid]) .field {
    border-color: var(--destructive);
  }
  :host([invalid]:focus-within) .field {
    border-color: var(--destructive);
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
  }

  /* Disabled — muted surface, desaturated, not-allowed. 60% opacity on
   * the whole host matches ship-button's disabled affordance. */
  :host([disabled]) {
    opacity: 0.6;
    cursor: not-allowed;
  }
  :host([disabled]) .field {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 80%, transparent);
    box-shadow: none;
    cursor: not-allowed;
    filter: saturate(0.6);
  }
  :host([disabled]) select {
    cursor: not-allowed;
  }

  /* Forced-colors — make sure the ring is still visible. */
  @media (forced-colors: active) {
    :host(:focus-within) .field { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`];

// src/web-components/ship-select.ts
var ShipSelect = class extends i4 {
  constructor() {
    super();
    this.defaultValue = "";
    this.value = "";
    this.name = "";
    this.required = false;
    this.disabled = false;
    this.invalid = false;
    this.size = "md";
    this.label = "";
    this.describedBy = "";
    this.internals = this.attachInternals();
  }
  get selectEl() {
    return this.renderRoot.querySelector("select");
  }
  connectedCallback() {
    super.connectedCallback();
    this.defaultValue = this.value;
  }
  firstUpdated() {
    this.syncOptions();
  }
  updated() {
    this.syncOptions();
    this.setFormValue(this.value);
    this.updatePlaceholderState();
  }
  formResetCallback() {
    this.value = this.defaultValue;
  }
  formStateRestoreCallback(state) {
    this.value = state ?? "";
  }
  setFormValue(v2) {
    if (typeof this.internals.setFormValue === "function") this.internals.setFormValue(v2);
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
  /** Pick the label for the current value. We always read from light-DOM
   * options (the source of truth) so the label is available on every render,
   * including the first one before `syncOptions()` has cloned them into
   * shadow. Falls back to the raw value so the chrome never renders empty. */
  currentLabel() {
    const lightOpts = Array.from(
      this.querySelectorAll(":scope > option, :scope > optgroup > option")
    );
    if (lightOpts.length === 0) return this.value;
    const match = lightOpts.find((o6) => o6.value === this.value) ?? lightOpts.find((o6) => o6.hasAttribute("selected")) ?? lightOpts[0];
    const text = (match?.textContent || "").trim();
    return text || match?.value || this.value;
  }
  /** Mirror `data-placeholder` on the host when the current option has an
   * empty value — lets the visible label dim to `--muted-foreground`. */
  updatePlaceholderState() {
    const sel = this.selectEl;
    if (!sel) {
      const lightOpts = Array.from(
        this.querySelectorAll(":scope > option, :scope > optgroup > option")
      );
      const match = lightOpts.find((o6) => o6.value === this.value) ?? lightOpts.find((o6) => o6.hasAttribute("selected")) ?? lightOpts[0];
      if (!match || match.value === "") this.setAttribute("data-placeholder", "");
      else this.removeAttribute("data-placeholder");
      return;
    }
    const opt = sel.selectedOptions?.[0] ?? sel.options[sel.selectedIndex];
    const isPlaceholder = !opt || opt.value === "";
    if (isPlaceholder) this.setAttribute("data-placeholder", "");
    else this.removeAttribute("data-placeholder");
  }
  onChange(e6) {
    this.value = e6.target.value;
    this.setFormValue(this.value);
    this.updatePlaceholderState();
    this.dispatchEvent(new CustomEvent("ship-change", {
      detail: { value: this.value },
      bubbles: true,
      composed: true
    }));
  }
  render() {
    return b2`
      <div class="field" part="field">
        <span class="value" aria-hidden="true">${this.currentLabel()}</span>
        <svg class="chevron" part="chevron" viewBox="0 0 16 16"
          aria-hidden="true" focusable="false" fill="none"
          stroke="currentColor" stroke-width="1.75"
          stroke-linecap="round" stroke-linejoin="round">
          <path d="M4 6l4 4 4-4"></path>
        </svg>
        <select part="select"
          name=${this.name}
          ?required=${this.required}
          ?disabled=${this.disabled}
          aria-invalid=${this.invalid ? "true" : "false"}
          aria-label=${this.label || A}
          aria-describedby=${this.describedBy || A}
          @change=${this.onChange}
        ></select>
      </div>
      <span hidden><slot @slotchange=${this.syncOptions}></slot></span>
    `;
  }
};
ShipSelect.formAssociated = true;
ShipSelect.styles = shipSelectStyles;
ShipSelect.properties = {
  value: { type: String },
  name: { type: String },
  required: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true },
  invalid: { type: Boolean, reflect: true },
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true },
  label: { type: String },
  describedBy: { type: String, attribute: "described-by" }
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

// src/web-components/ship-slider.styles.ts
var shipSliderStyles = [
  reducedMotion,
  i`
    :host {
      display: block;
      font-family: var(--font-body);
      color: var(--foreground);
      -webkit-tap-highlight-color: transparent;
      --tone: var(--primary);
    }
    :host([hidden]) { display: none; }

    :host(:not([tone])),
    :host([tone='default']) { --tone: var(--primary); }
    :host([tone='info'])        { --tone: var(--info); }
    :host([tone='success'])     { --tone: var(--success); }
    :host([tone='warning'])     { --tone: var(--warning); }
    :host([tone='destructive']) { --tone: var(--destructive); }
    :host([tone='accent'])      { --tone: var(--accent); }
    :host([tone='muted'])       { --tone: var(--muted-foreground); }

    /* Size channel */
    :host(:not([size])),
    :host([size='md']) {
      --sl-h: 6px;
      --sl-thumb: 16px;
    }
    :host([size='sm']) {
      --sl-h: 4px;
      --sl-thumb: 12px;
    }
    :host([size='lg']) {
      --sl-h: 8px;
      --sl-thumb: 20px;
    }

    .container {
      position: relative;
      display: flex;
      align-items: center;
      width: 100%;
      height: var(--sl-thumb);
      cursor: pointer;
      touch-action: none;
    }
    :host([disabled]) .container {
      cursor: not-allowed;
      opacity: 0.5;
    }

    /* Track */
    .track {
      position: relative;
      width: 100%;
      height: var(--sl-h);
      background: var(--muted);
      border-radius: var(--radius-full);
      overflow: hidden;
    }

    /* Filled portion */
    .fill {
      position: absolute;
      top: 0;
      left: 0;
      height: 100%;
      background: var(--tone);
      border-radius: inherit;
      transition: width 0ms;
    }

    /* Thumb */
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
        box-shadow var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
      box-sizing: border-box;
    }
    .thumb:hover {
      box-shadow: var(--shadow-md);
      transform: translate(-50%, -50%) scale(1.05);
    }
    .thumb:active {
      transform: translate(-50%, -50%) scale(0.95);
    }

    /* Focus ring */
    :host(:not([disabled])) .thumb:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }
    @media (forced-colors: active) {
      .thumb:focus-visible {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
      .thumb { border-color: CanvasText; }
      .fill { background: Highlight; }
    }
  `
];

// src/web-components/ship-slider.ts
var ShipSlider = class extends i4 {
  constructor() {
    super(...arguments);
    this.value = 0;
    this.min = 0;
    this.max = 100;
    this.step = 1;
    this.disabled = false;
    this.size = "md";
    this.label = "";
    this.dragging = false;
    this.onPointerDown = (e6) => {
      if (this.disabled) return;
      e6.preventDefault();
      this.dragging = true;
      this.setValue(this.valueFromPointer(e6.clientX));
      const thumb = this.renderRoot.querySelector(".thumb");
      thumb?.focus();
      const onMove = (ev) => {
        if (!this.dragging) return;
        this.setValue(this.valueFromPointer(ev.clientX));
      };
      const onUp = () => {
        this.dragging = false;
        document.removeEventListener("pointermove", onMove);
        document.removeEventListener("pointerup", onUp);
      };
      document.addEventListener("pointermove", onMove);
      document.addEventListener("pointerup", onUp);
    };
    this.onKeydown = (e6) => {
      if (this.disabled) return;
      let handled = true;
      switch (e6.key) {
        case "ArrowRight":
        case "ArrowUp":
          this.setValue(this.value + this.step);
          break;
        case "ArrowLeft":
        case "ArrowDown":
          this.setValue(this.value - this.step);
          break;
        case "Home":
          this.setValue(this.min);
          break;
        case "End":
          this.setValue(this.max);
          break;
        case "PageUp":
          this.setValue(this.value + this.step * 10);
          break;
        case "PageDown":
          this.setValue(this.value - this.step * 10);
          break;
        default:
          handled = false;
      }
      if (handled) e6.preventDefault();
    };
  }
  get pct() {
    const range = this.max - this.min;
    if (range <= 0) return 0;
    return (this.value - this.min) / range * 100;
  }
  clamp(v2) {
    const stepped = Math.round((v2 - this.min) / this.step) * this.step + this.min;
    return Math.max(this.min, Math.min(this.max, stepped));
  }
  setValue(v2) {
    const clamped = this.clamp(v2);
    if (clamped === this.value) return;
    this.value = clamped;
    this.dispatchEvent(new CustomEvent("ship-change", {
      detail: { value: this.value },
      bubbles: true,
      composed: true
    }));
  }
  valueFromPointer(clientX) {
    const track = this.renderRoot.querySelector(".track");
    if (!track) return this.value;
    const rect = track.getBoundingClientRect();
    const pct = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    return this.min + pct * (this.max - this.min);
  }
  render() {
    const pct = this.pct;
    return b2`
      <div class="container" @pointerdown=${this.onPointerDown}>
        <div class="track" part="track">
          <div class="fill" part="fill" style="width: ${pct}%"></div>
        </div>
        <div
          class="thumb"
          part="thumb"
          role="slider"
          tabindex=${this.disabled ? "-1" : "0"}
          aria-valuemin=${this.min}
          aria-valuemax=${this.max}
          aria-valuenow=${this.value}
          aria-label=${this.label || "slider"}
          aria-disabled=${this.disabled ? "true" : "false"}
          style="left: ${pct}%"
          @keydown=${this.onKeydown}
        ></div>
      </div>
    `;
  }
};
ShipSlider.styles = shipSliderStyles;
ShipSlider.properties = {
  value: { type: Number, reflect: true },
  min: { type: Number },
  max: { type: Number },
  step: { type: Number },
  disabled: { type: Boolean, reflect: true },
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true },
  label: { type: String }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-slider")) {
  customElements.define("ship-slider", ShipSlider);
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
  reducedMotion,
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
    this.onKey = (e6) => {
      if (!this.interactive) return;
      if (e6.key === "Enter" || e6.key === " ") {
        e6.preventDefault();
        this.click();
      }
    };
  }
  connectedCallback() {
    super.connectedCallback();
    if (this.interactive) {
      if (!this.hasAttribute("role")) this.setAttribute("role", "button");
      if (!this.hasAttribute("tabindex")) this.setAttribute("tabindex", "0");
      this.addEventListener("keydown", this.onKey);
    }
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("keydown", this.onKey);
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
    this.label = "";
  }
  render() {
    if (this.label) {
      return b2`<span class="dot" part="dot" role="img" aria-label=${this.label}></span>`;
    }
    return b2`<span class="dot" part="dot" aria-hidden="true"></span>`;
  }
};
ShipStatusDot.styles = [
  reducedMotion,
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
  size: { type: String, reflect: true },
  label: { type: String }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-status-dot")) {
  customElements.define("ship-status-dot", ShipStatusDot);
}

// src/web-components/ship-switch.styles.ts
var shipSwitchStyles = [
  reducedMotion,
  i`
    :host {
      display: inline-flex;
      align-items: center;
      gap: var(--space-2);
      vertical-align: middle;
      font-family: var(--font-body);
      font-size: var(--text-base);
      color: var(--foreground);
      -webkit-tap-highlight-color: transparent;

      /* Tone channel — overridden per :host([tone=…]) below. Default is
       * primary so an attribute-less switch reads as the brand toggle. */
      --tone: var(--primary);
      --tone-on-fg: var(--primary-foreground);
    }
    :host([hidden]) { display: none; }

    :host(:not([tone])),
    :host([tone='default']) {
      --tone: var(--primary);
      --tone-on-fg: var(--primary-foreground);
    }
    :host([tone='info'])        { --tone: var(--info);        --tone-on-fg: oklch(0.99 0.02 260); }
    :host([tone='success'])     { --tone: var(--success);     --tone-on-fg: oklch(0.99 0.02 150); }
    :host([tone='warning'])     { --tone: var(--warning);     --tone-on-fg: oklch(0.2 0.04 75); }
    :host([tone='destructive']) { --tone: var(--destructive); --tone-on-fg: oklch(0.99 0.02 25); }
    :host([tone='accent'])      { --tone: var(--accent);      --tone-on-fg: var(--accent-foreground); }
    :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }

    /* Size channel. Each pair is (track W × H, thumb diameter). The
     * 1px border is absorbed via box-sizing so the visible pill reads
     * at the advertised dimensions. */
    :host(:not([size])),
    :host([size='md']) { --sw-w: 28px; --sw-h: 16px; --sw-thumb: 12px; }
    :host([size='sm']) { --sw-w: 24px; --sw-h: 14px; --sw-thumb: 10px; }
    :host([size='lg']) { --sw-w: 32px; --sw-h: 18px; --sw-thumb: 14px; }

    label {
      display: inline-flex;
      align-items: center;
      gap: var(--space-2);
      cursor: pointer;
      user-select: none;
      -webkit-user-select: none;
    }
    :host([disabled]) label { cursor: not-allowed; }

    /* ------------------------------------------------------------------
     * Track — the pill. Border adds a crisp 1px edge that keeps the off
     * state legible against neutral backgrounds (shadcn does the same).
     * Transitions are token-driven so reduced-motion collapses them.
     * ------------------------------------------------------------------ */
    .track {
      position: relative;
      display: inline-block;
      flex-shrink: 0;
      width: var(--sw-w);
      height: var(--sw-h);
      background: var(--muted);
      border: 1px solid color-mix(in oklch, var(--border) 140%, transparent);
      border-radius: var(--radius-full);
      box-sizing: border-box;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
    }

    /* Hover — deepen the border + subtle shadow, matching ship-button's
     * quiet hover language. Skipped on disabled so users can tell when
     * the control is unreachable. */
    :host(:not([disabled]):not([checked])) label:hover .track {
      background: color-mix(in oklch, var(--muted) 70%, var(--foreground) 6%);
      border-color: color-mix(in oklch, var(--border) 180%, transparent);
    }

    /* Checked — fill with tone, top-highlight inset for the premium feel
     * (same recipe as ship-button solid). Border picks up a slightly
     * darker edge so the pill still reads crisply over light surfaces. */
    :host([checked]) .track {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent);
    }
    :host([checked]:not([disabled])) label:hover .track {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 26%, transparent),
        var(--shadow-xs);
    }

    /* ------------------------------------------------------------------
     * Thumb — sliding pellet. Sits 1px inside the border in both states;
     * the left calc flips between those anchors so sizes stay in sync.
     * ------------------------------------------------------------------ */
    .thumb {
      position: absolute;
      top: 50%;
      left: 1px;
      width: var(--sw-thumb);
      height: var(--sw-thumb);
      background: var(--background);
      border-radius: 50%;
      box-shadow:
        0 1px 2px 0 oklch(0 0 0 / 18%),
        0 1px 1px 0 oklch(0 0 0 / 10%);
      transform: translateY(-50%);
      transition:
        left var(--duration-fast) var(--ease-out),
        background-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out);
      pointer-events: none;
    }
    :host([checked]) .thumb {
      left: calc(100% - var(--sw-thumb) - 3px);
      background: var(--tone-on-fg);
    }

    /* ------------------------------------------------------------------
     * Native input — visually hidden but preserved for form submission,
     * keyboard toggle (Space), and a11y tools. Stretches over the track
     * so hit-testing matches the visible affordance.
     * ------------------------------------------------------------------ */
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

    /* Focus ring — stacked box-shadow on the track so the ring sits
     * outside the pill without reserving layout. Uses :has() so the
     * ring travels with the native input's :focus-visible, keeping
     * keyboard semantics identical to ship-button. */
    :host(:not([disabled])) label:has(input:focus-visible) .track {
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }
    :host([checked]:not([disabled])) label:has(input:focus-visible) .track {
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }

    /* Forced colors — drop the color-mix ring in favor of a CanvasText
     * outline that high-contrast modes can render reliably. */
    @media (forced-colors: active) {
      :host(:not([disabled])) label:has(input:focus-visible) .track {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
      .track { border-color: CanvasText; }
      :host([checked]) .track { background: Highlight; }
    }

    /* ------------------------------------------------------------------
     * Disabled — muted + desaturated, no interactions. Distinct from
     * loading (which this component doesn't have) so the unreachable
     * state is unambiguous.
     * ------------------------------------------------------------------ */
    :host([disabled]) {
      color: var(--muted-foreground);
    }
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
    :host([disabled]) .thumb {
      box-shadow: 0 1px 1px 0 oklch(0 0 0 / 10%);
    }

    /* Slotted label — inherits host color + size so callers don't need
     * to restyle. Wraps naturally when the row gets tight. */
    ::slotted(*) {
      font: inherit;
      color: inherit;
      line-height: var(--leading-tight);
    }
  `
];

// src/web-components/ship-switch.ts
var ShipSwitch = class extends i4 {
  constructor() {
    super();
    this.defaultChecked = false;
    this.checked = false;
    this.disabled = false;
    this.size = "md";
    this.name = "";
    this.value = "on";
    this.label = "";
    this.onChange = (e6) => {
      const c4 = e6.target.checked;
      this.checked = c4;
      this.setFormValue(c4 ? this.value : null);
      this.dispatchEvent(new CustomEvent("ship-change", {
        detail: { checked: c4 },
        bubbles: true,
        composed: true
      }));
    };
    this.internals = this.attachInternals();
  }
  connectedCallback() {
    super.connectedCallback();
    this.defaultChecked = this.checked;
  }
  updated() {
    this.setFormValue(this.checked ? this.value : null);
  }
  formResetCallback() {
    this.checked = this.defaultChecked;
  }
  formStateRestoreCallback(state) {
    this.checked = state === this.value;
  }
  setFormValue(v2) {
    if (typeof this.internals.setFormValue === "function") {
      this.internals.setFormValue(v2);
    }
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
            aria-label=${this.label || this.name || "toggle"}
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
ShipSwitch.formAssociated = true;
ShipSwitch.styles = shipSwitchStyles;
ShipSwitch.properties = {
  checked: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true },
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true },
  name: { type: String },
  value: { type: String },
  label: { type: String }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-switch")) {
  customElements.define("ship-switch", ShipSwitch);
}

// src/web-components/ship-tabs.styles.ts
var shipTabsStyles = [reducedMotion, i`
  :host {
    display: block;
    font-family: var(--font-body);
    --tone: var(--primary);
    --tone-fg: var(--foreground);
  }
  :host([hidden]) { display: none; }

  :host([tone='info'])        { --tone: var(--info); }
  :host([tone='success'])     { --tone: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); }

  /* Size channel — matches ship-button's sm/md/lg height scale so a tab
   * sitting next to a button lines up optically. */
  :host(:not([size])),
  :host([size='md']) {
    --tab-h: 36px;
    --tab-px: 14px;
    --tab-fs: var(--text-base);
    --tab-fw: var(--font-weight-medium);
    --tab-active-fw: var(--font-weight-semibold);
    --tab-gap: var(--space-2);
    --tab-radius: var(--radius-md);
    --tab-count-fs: 10.5px;
    --tab-count-px: 7px;
  }
  :host([size='sm']) {
    --tab-h: 30px;
    --tab-px: 10px;
    --tab-fs: var(--text-sm);
    --tab-gap: 6px;
    --tab-radius: var(--radius-sm);
    --tab-count-fs: 9.5px;
    --tab-count-px: 6px;
  }
  :host([size='lg']) {
    --tab-h: 42px;
    --tab-px: var(--space-4);
    --tab-fs: var(--text-md);
    --tab-gap: var(--space-2);
    --tab-radius: var(--radius-md);
    --tab-count-fs: 11px;
    --tab-count-px: 8px;
  }

  /* Shared tablist + tab chrome. Variants reshape from here via their
   * class hook (.underline / .segmented) — no per-variant duplication. */
  .tablist {
    position: relative;
    display: flex;
    align-items: stretch;
    gap: 2px;
  }
  .tab {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--tab-gap);
    height: var(--tab-h);
    padding: 0 var(--tab-px);
    background: transparent;
    border: 0;
    color: var(--muted-foreground);
    font-family: inherit;
    font-size: var(--tab-fs);
    font-weight: var(--tab-fw, var(--font-weight-medium));
    line-height: 1;
    cursor: pointer;
    white-space: nowrap;
    user-select: none;
    -webkit-user-select: none;
    -webkit-tap-highlight-color: transparent;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }
  .tab:hover:not(:disabled):not([aria-selected='true']) {
    color: var(--foreground);
  }
  .tab[aria-selected='true'] {
    color: var(--tone-fg);
    font-weight: var(--tab-active-fw, var(--font-weight-semibold));
  }
  .tab:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    border-radius: var(--tab-radius);
    z-index: 1;
  }
  .tab:disabled {
    cursor: not-allowed;
    color: color-mix(in oklch, var(--muted-foreground) 55%, transparent);
    filter: saturate(0.6);
  }

  /* Underline variant — indicator is an ::after pseudo on .tablist
   * positioned via --underline-x/--underline-w; width transition drives
   * the tab-to-tab slide. Pre-measurement opacity hides the indicator
   * until the component writes the geometry on first update. */
  .tablist.underline {
    border-bottom: 1px solid var(--border);
    margin-bottom: 20px;
    gap: 0;
  }
  .tablist.underline::after {
    content: '';
    position: absolute;
    left: 0;
    bottom: -1px;
    height: 2px;
    width: var(--underline-w, 0px);
    transform: translateX(var(--underline-x, 0px));
    background: var(--tone);
    border-radius: 2px 2px 0 0;
    pointer-events: none;
    transition:
      transform var(--duration-medium) var(--ease-out),
      width var(--duration-medium) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }
  :host(:not([data-underline-ready])) .tablist.underline::after {
    opacity: 0;
  }
  .tablist.underline .tab { border-radius: 0; margin-bottom: -1px; }
  .tablist.underline .tab:hover:not(:disabled):not([aria-selected='true']) {
    background: color-mix(in oklch, var(--muted) 60%, transparent);
  }
  :host([sticky]) .tablist.underline {
    position: sticky;
    top: 0;
    background: var(--background);
    z-index: var(--z-sticky);
    padding-top: 4px;
    margin-top: -4px;
  }

  /* Segmented variant — pill group. Active tab elevates onto --background
   * with a soft shadow so it reads as a chip lifted off the muted track. */
  .tablist.segmented {
    display: inline-flex;
    gap: 2px;
    padding: 3px;
    background: var(--muted);
    border: 1px solid color-mix(in oklch, var(--border) 80%, transparent);
    border-radius: calc(var(--tab-radius) + 3px);
  }
  .tablist.segmented .tab {
    flex: 1;
    height: calc(var(--tab-h) - 6px);
    border-radius: var(--tab-radius);
  }
  .tablist.segmented .tab:hover:not(:disabled):not([aria-selected='true']) {
    color: var(--foreground);
    background: color-mix(in oklch, var(--background) 60%, transparent);
  }
  .tablist.segmented .tab[aria-selected='true'] {
    background: var(--background);
    color: var(--foreground);
    box-shadow:
      inset 0 0 0 1px color-mix(in oklch, var(--border) 60%, transparent),
      var(--shadow-xs);
  }
  .tablist.segmented::after { display: none; }

  /* Count chip — tabular-nums so digit-width changes don't shift the
   * label. Active state re-colors to the tone channel. */
  .count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: calc(var(--tab-count-fs) + 8px);
    padding: 1px var(--tab-count-px);
    background: color-mix(in oklch, var(--muted-foreground) 14%, transparent);
    color: var(--muted-foreground);
    border-radius: var(--radius-full);
    font-size: var(--tab-count-fs);
    font-weight: var(--font-weight-semibold);
    font-variant-numeric: tabular-nums;
    line-height: 1.4;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .tab[aria-selected='true'] .count {
    background: color-mix(in oklch, var(--tone) 18%, transparent);
    color: var(--tone);
  }

  .panels { display: block; }

  /* Vertical orientation — tablist on the left, panels on the right.
   * Underline indicator flips to a right-edge rail; --underline-y/h drive
   * the same transform-based animation on the Y axis. */
  :host([orientation='vertical']) {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: var(--space-6);
  }
  :host([orientation='vertical']) .tablist.underline {
    flex-direction: column;
    align-items: stretch;
    border-bottom: 0;
    border-right: 1px solid var(--border);
    margin-bottom: 0;
    gap: 2px;
  }
  :host([orientation='vertical']) .tablist.underline .tab {
    justify-content: flex-start;
    margin-bottom: 0;
    margin-right: -1px;
    border-radius: var(--tab-radius) 0 0 var(--tab-radius);
  }
  :host([orientation='vertical']) .tablist.underline::after {
    left: auto;
    right: -1px;
    bottom: auto;
    top: 0;
    width: 2px;
    height: var(--underline-h, 0px);
    transform: translateY(var(--underline-y, 0px));
    border-radius: 2px 0 0 2px;
    transition:
      transform var(--duration-medium) var(--ease-out),
      height var(--duration-medium) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
  }
  :host([orientation='vertical']) .tablist.segmented {
    flex-direction: column;
    padding: 3px;
  }
  :host([orientation='vertical']) .tablist.segmented .tab {
    flex: none;
    justify-content: flex-start;
  }

  @media (forced-colors: active) {
    .tab:focus-visible { outline: 2px solid CanvasText; outline-offset: -2px; }
    .tablist.underline::after { background: CanvasText; }
    .tab[aria-selected='true'] { color: CanvasText; }
  }
`];
var shipTabPanelStyles = [reducedMotion, i`
  :host { display: none; }
  :host([active]) {
    display: block;
    animation: ship-tab-panel-fade var(--duration-medium) var(--ease-out);
  }
  @keyframes ship-tab-panel-fade {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  :host(:focus-visible) {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    border-radius: var(--radius-md);
  }
`];

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
  name: { type: String, reflect: true },
  label: { type: String, reflect: true },
  count: { type: Number, reflect: true },
  disabled: { type: Boolean, reflect: true }
};
var ShipTabPanel = class extends i4 {
  constructor() {
    super(...arguments);
    this.name = "";
    this.active = false;
  }
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "tabpanel");
    if (!this.hasAttribute("tabindex")) this.setAttribute("tabindex", "0");
    if (this.name && !this.id) this.id = `ship-tabpanel-${this.name}`;
    if (this.name) this.setAttribute("aria-labelledby", `ship-tab-${this.name}`);
  }
  render() {
    return b2`<slot></slot>`;
  }
};
ShipTabPanel.styles = shipTabPanelStyles;
ShipTabPanel.properties = {
  name: { type: String, reflect: true },
  active: { type: Boolean, reflect: true }
};
var ShipTabs = class extends i4 {
  constructor() {
    super(...arguments);
    this.active = "";
    this.variant = "underline";
    this.size = "md";
    this.sticky = false;
    this.orientation = "horizontal";
    /** Underline-indicator measurement runs through rAF so we never write
     * custom properties more than once per frame even under event storms. */
    this.rafId = 0;
  }
  connectedCallback() {
    super.connectedCallback();
    this.mutationObserver = new MutationObserver(() => this.requestUpdate());
    this.mutationObserver.observe(this, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["name", "label", "count", "disabled"]
    });
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.mutationObserver?.disconnect();
    this.resizeObserver?.disconnect();
    if (this.rafId) cancelAnimationFrame(this.rafId);
  }
  get tabs() {
    return Array.from(this.querySelectorAll(":scope > ship-tab"));
  }
  get panels() {
    return Array.from(this.querySelectorAll(":scope > ship-tab-panel"));
  }
  firstUpdated() {
    this.syncPanels();
    this.attachResizeObserver();
    this.scheduleIndicatorSync();
  }
  updated(changed) {
    this.syncPanels();
    if (changed.has("active") || changed.has("variant") || changed.has("orientation") || changed.has("size")) {
      this.scheduleIndicatorSync();
    }
  }
  syncPanels() {
    for (const p3 of this.panels) p3.active = p3.name === this.active;
  }
  attachResizeObserver() {
    const tablist = this.renderRoot.querySelector(".tablist");
    if (!tablist || typeof ResizeObserver === "undefined") return;
    this.resizeObserver = new ResizeObserver(() => this.scheduleIndicatorSync());
    this.resizeObserver.observe(tablist);
  }
  /** Coalesce indicator updates through rAF. We measure the active tab's
   * offset within the tablist and write the geometry into CSS custom
   * properties; the pseudo-element transition animates the move. */
  scheduleIndicatorSync() {
    if (this.variant === "segmented") return;
    if (this.rafId) cancelAnimationFrame(this.rafId);
    this.rafId = requestAnimationFrame(() => this.syncIndicator());
  }
  syncIndicator() {
    this.rafId = 0;
    const tablist = this.renderRoot.querySelector(".tablist");
    if (!tablist) return;
    const activeBtn = tablist.querySelector(
      'button[aria-selected="true"]'
    );
    if (!activeBtn) {
      this.removeAttribute("data-underline-ready");
      return;
    }
    if (this.orientation === "vertical") {
      tablist.style.setProperty("--underline-y", `${activeBtn.offsetTop}px`);
      tablist.style.setProperty("--underline-h", `${activeBtn.offsetHeight}px`);
    } else {
      tablist.style.setProperty("--underline-x", `${activeBtn.offsetLeft}px`);
      tablist.style.setProperty("--underline-w", `${activeBtn.offsetWidth}px`);
    }
    this.setAttribute("data-underline-ready", "");
  }
  activate(name) {
    if (this.active === name) return;
    this.active = name;
    this.dispatchEvent(new CustomEvent("ship-tab-change", {
      detail: { name },
      bubbles: true,
      composed: true
    }));
  }
  onKeydown(e6) {
    const vertical = this.orientation === "vertical";
    const nextKey = vertical ? "ArrowDown" : "ArrowRight";
    const prevKey = vertical ? "ArrowUp" : "ArrowLeft";
    const nav = [nextKey, prevKey, "Home", "End"];
    const activate = ["Enter", " "];
    if (!nav.includes(e6.key) && !activate.includes(e6.key)) return;
    e6.preventDefault();
    const enabled = this.tabs.filter((t4) => !t4.disabled);
    if (!enabled.length) return;
    const names = enabled.map((t4) => t4.name);
    const i6 = Math.max(0, names.indexOf(this.active));
    let next = i6;
    if (e6.key === nextKey) next = (i6 + 1) % names.length;
    else if (e6.key === prevKey) next = (i6 - 1 + names.length) % names.length;
    else if (e6.key === "Home") next = 0;
    else if (e6.key === "End") next = names.length - 1;
    const targetName = names[next];
    this.activate(targetName);
    queueMicrotask(() => {
      const btn = this.renderRoot.querySelector(
        `button[data-name="${targetName}"]`
      );
      btn?.focus();
    });
  }
  render() {
    const tabs = this.tabs.map((t4, idx) => ({
      index: idx,
      name: t4.name,
      label: t4.label || t4.name,
      count: t4.count,
      disabled: t4.disabled
    }));
    return b2`
      <div class="tablist ${this.variant}" part="tablist" role="tablist"
           aria-orientation=${this.orientation}
           @keydown=${this.onKeydown}>
        ${tabs.map((t4) => b2`
          <button
            class="tab"
            part="tab"
            role="tab"
            id=${`ship-tab-${t4.name}`}
            data-name=${t4.name}
            aria-selected=${t4.name === this.active ? "true" : "false"}
            aria-controls=${`ship-tabpanel-${t4.name}`}
            aria-disabled=${t4.disabled ? "true" : "false"}
            tabindex=${t4.name === this.active ? "0" : "-1"}
            ?disabled=${t4.disabled}
            @click=${() => !t4.disabled && this.activate(t4.name)}
          >
            <span class="label"><slot name=${`label-${t4.name}`}>${t4.label}</slot></span>
            ${t4.count != null ? b2`<span class="count" aria-hidden="true">${t4.count}</span>` : null}
          </button>
        `)}
      </div>
      <div class="panels" part="panels"><slot></slot></div>
    `;
  }
};
ShipTabs.styles = shipTabsStyles;
ShipTabs.properties = {
  active: { type: String, reflect: true },
  variant: { type: String, reflect: true },
  size: { type: String, reflect: true },
  tone: { type: String, reflect: true },
  sticky: { type: Boolean, reflect: true },
  orientation: { type: String, reflect: true }
};
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-tab")) customElements.define("ship-tab", ShipTab);
  if (!customElements.get("ship-tab-panel")) customElements.define("ship-tab-panel", ShipTabPanel);
  if (!customElements.get("ship-tabs")) customElements.define("ship-tabs", ShipTabs);
}

// src/web-components/ship-textarea.styles.ts
var shipTextareaStyles = [reducedMotion, i`
  :host {
    display: block;
    width: 100%;
    --tone: var(--ring-color);
    --tone-fg: var(--foreground);
  }
  :host([hidden]) { display: none; }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--ring-color);
    --tone-fg: var(--foreground);
  }
  :host([tone='info'])        { --tone: var(--info);        --tone-fg: var(--info); }
  :host([tone='success'])     { --tone: var(--success);     --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);     --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent);      --tone-fg: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) {
    --ta-px: 12px;
    --ta-py: 10px;
    --ta-fs: var(--text-base);
    --ta-lh: 1.5;
    --ta-radius: var(--radius-md);
  }
  :host([size='sm']) {
    --ta-px: 10px;
    --ta-py: 8px;
    --ta-fs: var(--text-sm);
    --ta-lh: 1.45;
    --ta-radius: var(--radius-sm);
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
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
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
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
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
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
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
    font-family: ${MONO};
    font-size: var(--text-base);
    line-height: 1.55;
    letter-spacing: 0;
    tab-size: 2;
  }

  @media (forced-colors: active) {
    textarea:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`];

// src/web-components/ship-textarea.ts
var ShipTextarea = class extends i4 {
  constructor() {
    super();
    this.defaultValue = "";
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
    this.invalid = false;
    this.label = "";
    this.describedBy = "";
    this.internals = this.attachInternals();
  }
  connectedCallback() {
    super.connectedCallback();
    this.defaultValue = this.value;
  }
  updated() {
    this.setFormValue(this.value);
  }
  formResetCallback() {
    this.value = this.defaultValue;
  }
  formStateRestoreCallback(state) {
    this.value = state ?? "";
  }
  setFormValue(v2) {
    if (typeof this.internals.setFormValue === "function") this.internals.setFormValue(v2);
  }
  onInput(e6) {
    this.value = e6.target.value;
    this.setFormValue(this.value);
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
        aria-invalid=${this.invalid ? "true" : "false"}
        aria-required=${this.required ? "true" : "false"}
        aria-label=${this.label || A}
        aria-describedby=${this.describedBy || A}
        style="resize: ${this.resize}"
        @input=${this.onInput}
      ></textarea>
    `;
  }
};
ShipTextarea.formAssociated = true;
ShipTextarea.styles = shipTextareaStyles;
ShipTextarea.properties = {
  value: { type: String },
  placeholder: { type: String },
  name: { type: String },
  rows: { type: Number },
  required: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true },
  readonly: { type: Boolean, reflect: true },
  tone: { type: String, reflect: true },
  variant: { type: String, reflect: true },
  size: { type: String, reflect: true },
  resize: { type: String },
  invalid: { type: Boolean, reflect: true },
  label: { type: String },
  describedBy: { type: String, attribute: "described-by" }
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
      <button
        @click=${this.toggle}
        aria-label="Toggle theme"
        aria-pressed=${isDark ? "true" : "false"}
        title="Toggle theme"
      >
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

// src/web-components/ship-toggle-group.ts
var ShipToggle = class extends i4 {
  constructor() {
    super(...arguments);
    this.pressed = false;
    this.disabled = false;
    this.value = "";
    this.size = "md";
    this.variant = "outline";
    this.onClick = () => {
      if (this.disabled) return;
      this.pressed = !this.pressed;
      this.dispatchEvent(new CustomEvent("ship-toggle", {
        detail: { pressed: this.pressed, value: this.value },
        bubbles: true,
        composed: true
      }));
    };
  }
  /** Focus the button — used by parent group for keyboard nav. */
  focusButton() {
    const btn = this.renderRoot.querySelector("button");
    btn?.focus();
  }
  render() {
    return b2`
      <button
        part="button"
        type="button"
        aria-pressed=${this.pressed ? "true" : "false"}
        ?disabled=${this.disabled}
        @click=${this.onClick}
      ><slot></slot></button>
    `;
  }
};
ShipToggle.styles = [reducedMotion, i`
    :host { display: inline-flex; }
    :host([hidden]) { display: none; }

    /* Size channel */
    :host(:not([size])), :host([size='md']) {
      --tgl-h: 36px; --tgl-px: 12px; --tgl-fs: var(--text-base);
    }
    :host([size='sm']) {
      --tgl-h: 28px; --tgl-px: 8px; --tgl-fs: var(--text-sm);
    }
    :host([size='lg']) {
      --tgl-h: 42px; --tgl-px: 16px; --tgl-fs: var(--text-base);
    }

    button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: 6px;
      height: var(--tgl-h);
      padding: 0 var(--tgl-px);
      font-family: var(--font-body);
      font-size: var(--tgl-fs);
      font-weight: var(--font-weight-medium);
      line-height: 1;
      color: var(--muted-foreground);
      background: transparent;
      border: 1px solid var(--border);
      border-radius: var(--radius);
      cursor: pointer;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out);
    }
    :host([variant='ghost']) button {
      border-color: transparent;
    }
    button:hover {
      background: var(--muted);
      color: var(--foreground);
    }
    :host([pressed]) button {
      background: var(--muted);
      color: var(--foreground);
      border-color: var(--border);
    }
    :host([disabled]) button {
      cursor: not-allowed;
      opacity: 0.5;
      pointer-events: none;
    }
    button:focus-visible {
      ${focusRingStyle}
    }
    @media (forced-colors: active) {
      button:focus-visible {
        ${focusRingForcedColors}
      }
      :host([pressed]) button {
        outline: 2px solid CanvasText;
      }
    }
  `];
ShipToggle.properties = {
  pressed: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true },
  value: { type: String, reflect: true },
  size: { type: String, reflect: true },
  variant: { type: String, reflect: true }
};
var ShipToggleGroup = class extends i4 {
  constructor() {
    super(...arguments);
    this.type = "single";
    this.value = "";
    this.size = "md";
    this.variant = "outline";
    this.onToggle = (e6) => {
      e6.stopPropagation();
      const toggle = e6.target.closest("ship-toggle");
      if (!toggle) return;
      const val = toggle.value;
      const selected = this.getSelectedValues();
      if (this.type === "single") {
        if (selected.has(val)) {
          this.setSelectedValues(/* @__PURE__ */ new Set());
        } else {
          this.setSelectedValues(/* @__PURE__ */ new Set([val]));
        }
      } else {
        if (selected.has(val)) {
          selected.delete(val);
        } else {
          selected.add(val);
        }
        this.setSelectedValues(selected);
      }
    };
    this.onKeydown = (e6) => {
      if (e6.key !== "ArrowRight" && e6.key !== "ArrowLeft") return;
      const items = this.enabledToggles;
      if (!items.length) return;
      const current = e6.composedPath().find(
        (n4) => items.includes(n4)
      ) || null;
      const i6 = current ? items.indexOf(current) : -1;
      const next = e6.key === "ArrowRight" ? (i6 + 1) % items.length : (i6 - 1 + items.length) % items.length;
      items[next].focusButton();
      e6.preventDefault();
    };
  }
  get toggles() {
    return Array.from(this.querySelectorAll(":scope > ship-toggle"));
  }
  get enabledToggles() {
    return this.toggles.filter((t4) => !t4.disabled);
  }
  getSelectedValues() {
    return new Set(this.value ? this.value.split(",").filter(Boolean) : []);
  }
  setSelectedValues(vals) {
    this.value = Array.from(vals).join(",");
    this.syncToggles();
    this.dispatchEvent(new CustomEvent("ship-change", {
      detail: { value: this.value },
      bubbles: true,
      composed: true
    }));
  }
  syncToggles() {
    const selected = this.getSelectedValues();
    for (const toggle of this.toggles) {
      toggle.pressed = selected.has(toggle.value);
      if (this.size) toggle.size = this.size;
      if (this.variant) toggle.variant = this.variant;
    }
  }
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "group");
    this.addEventListener("ship-toggle", this.onToggle);
    this.addEventListener("keydown", this.onKeydown);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("ship-toggle", this.onToggle);
    this.removeEventListener("keydown", this.onKeydown);
  }
  updated() {
    this.syncToggles();
  }
  render() {
    return b2`<slot @slotchange=${() => this.syncToggles()}></slot>`;
  }
};
ShipToggleGroup.styles = [reducedMotion, i`
    :host {
      display: inline-flex;
      gap: 0;
    }
    :host([hidden]) { display: none; }
    /* Merge borders between children */
    ::slotted(ship-toggle:not(:first-child)) {
      margin-left: -1px;
    }
    ::slotted(ship-toggle:first-child) {
      border-radius: var(--radius) 0 0 var(--radius);
    }
    ::slotted(ship-toggle:last-child) {
      border-radius: 0 var(--radius) var(--radius) 0;
    }
    ::slotted(ship-toggle:not(:first-child):not(:last-child)) {
      border-radius: 0;
    }
  `];
ShipToggleGroup.properties = {
  type: { type: String, reflect: true },
  value: { type: String, reflect: true },
  size: { type: String, reflect: true },
  variant: { type: String, reflect: true }
};
if (typeof customElements !== "undefined") {
  if (!customElements.get("ship-toggle")) {
    customElements.define("ship-toggle", ShipToggle);
  }
  if (!customElements.get("ship-toggle-group")) {
    customElements.define("ship-toggle-group", ShipToggleGroup);
  }
}

// src/web-components/ship-toolbar.ts
var ShipToolbar = class extends i4 {
  constructor() {
    super(...arguments);
    this.justify = "start";
    this.align = "center";
    this.gap = "md";
    this.variant = "default";
    this.onKey = (e6) => {
      if (e6.key !== "ArrowRight" && e6.key !== "ArrowLeft") return;
      const items = this.focusable();
      if (!items.length) return;
      const active = e6.composedPath().find((n4) => items.includes(n4)) || null;
      const i6 = active ? items.indexOf(active) : -1;
      const next = e6.key === "ArrowRight" ? (i6 + 1) % items.length : (i6 - 1 + items.length) % items.length;
      items[next].focus();
      e6.preventDefault();
    };
  }
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "toolbar");
    if (!this.hasAttribute("aria-orientation")) this.setAttribute("aria-orientation", "horizontal");
    this.addEventListener("keydown", this.onKey);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("keydown", this.onKey);
  }
  focusable() {
    const slot = this.renderRoot.querySelector("slot:not([name])");
    const els = slot?.assignedElements({ flatten: true }) ?? [];
    return els.filter((el) => !el.hasAttribute("disabled"));
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
var _ShipTooltip = class _ShipTooltip extends i4 {
  constructor() {
    super(...arguments);
    this.placement = "top";
    this.bubbleId = `ship-tooltip-${++_ShipTooltip._idSeq}`;
    this.onKeydown = (e6) => {
      if (e6.key === "Escape") this.blur();
    };
    this.wireTrigger = () => {
      const slot = this.renderRoot.querySelector("slot:not([name])");
      const first = slot?.assignedElements({ flatten: true })[0];
      if (!first) return;
      const existing = first.getAttribute("aria-describedby") || "";
      if (!existing.split(/\s+/).includes(this.bubbleId)) {
        first.setAttribute("aria-describedby", existing ? `${existing} ${this.bubbleId}` : this.bubbleId);
      }
    };
  }
  connectedCallback() {
    super.connectedCallback();
    this.addEventListener("keydown", this.onKeydown);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("keydown", this.onKeydown);
  }
  render() {
    return b2`
      <slot @slotchange=${this.wireTrigger}></slot>
      <span id=${this.bubbleId} class="bubble" role="tooltip"><slot name="content"></slot></span>
    `;
  }
};
_ShipTooltip.styles = i`
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
_ShipTooltip.properties = {
  placement: { type: String, reflect: true }
};
_ShipTooltip._idSeq = 0;
var ShipTooltip = _ShipTooltip;
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
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "listitem");
  }
  updated() {
    const summary = `${this.stepType} step ${this.name}${this.agent ? ` (${this.agent})` : ""}: ${this.state}`;
    this.setAttribute("aria-label", summary);
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
  updated() {
    this.setAttribute("role", "article");
    this.setAttribute("aria-label", `Pipeline run ${this.slug || ""}${this.agent ? ` on ${this.agent}` : ""}: ${this.state}`);
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
  updated() {
    this.setAttribute("role", "article");
    const desc = this.description ? `: ${this.description}` : "";
    this.setAttribute("aria-label", `Pipeline ${this.name}, ${this.steps} step${this.steps === 1 ? "" : "s"}${desc}`);
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
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "list");
    if (!this.hasAttribute("aria-label")) this.setAttribute("aria-label", "pipeline steps");
  }
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
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "row");
  }
  render() {
    return b2`
      <span class="cell" role="cell"><slot name="slug"></slot></span>
      <span class="cell" role="cell"><slot name="pipeline"></slot></span>
      <span class="cell" role="cell"><slot name="status"></slot></span>
      <span class="cell" role="cell"><slot name="duration"></slot></span>
      <span class="cell" role="cell"><slot name="completed"></slot></span>
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
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "region");
  }
  updated() {
    this.setAttribute("aria-label", `Diff: ${this.files} file${this.files === 1 ? "" : "s"} changed, ${this.additions} additions, ${this.deletions} deletions`);
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
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "group");
  }
  updated() {
    this.setAttribute("aria-label", `${this.path}: +${this.additions} -${this.deletions}`);
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
  updated() {
    const verb = this.type === "add" ? "added" : this.type === "del" ? "removed" : "context";
    this.setAttribute("aria-label", `line ${this.new || this.old} ${verb}`);
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
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "listitem");
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

// src/web-components/observability/ship-log-stream.ts
var ShipLogStream = class extends i4 {
  constructor() {
    super(...arguments);
    this.url = "";
    this.event = "log";
    this.autoscroll = false;
    this.active = false;
    this.maxBytes = 1e6;
    this._source = null;
    this._buffer = "";
    this._stickyBottom = true;
    this._onScroll = (e6) => {
      const pre = e6.currentTarget;
      this._stickyBottom = pre.scrollHeight - pre.clientHeight <= pre.scrollTop + 4;
    };
  }
  updated(changed) {
    if (changed.has("url") || changed.has("event")) {
      if (this.active && this._source) {
        this.close();
        this.open();
      }
    }
    if (changed.has("active")) {
      if (this.active) this.open();
      else this.close();
    }
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.close();
  }
  open() {
    if (!this.url || this._source) return;
    try {
      this._source = new EventSource(this.url, { withCredentials: true });
    } catch {
      this.dispatchEvent(new CustomEvent("ship-stream-error"));
      return;
    }
    this._source.addEventListener(this.event, (ev) => {
      this._append(ev.data ?? "");
    });
    this._source.onopen = () => {
      this.dispatchEvent(new CustomEvent("ship-stream-open"));
    };
    this._source.onerror = () => {
      this.dispatchEvent(new CustomEvent("ship-stream-error"));
    };
  }
  close() {
    if (this._source) {
      this._source.close();
      this._source = null;
      this.dispatchEvent(new CustomEvent("ship-stream-close"));
    }
  }
  clear() {
    this._buffer = "";
    this.requestUpdate();
  }
  _append(chunk) {
    this._buffer += chunk;
    if (this._buffer.length > this.maxBytes) {
      const overflow = this._buffer.length - this.maxBytes;
      const cut = this._buffer.indexOf("\n", overflow);
      this._buffer = this._buffer.slice(cut >= 0 ? cut + 1 : overflow);
    }
    this.requestUpdate();
  }
  firstUpdated() {
    if (this.active && !this._source) this.open();
  }
  updated_scroll() {
    if (!this.autoscroll || !this._stickyBottom) return;
    const pre = this.renderRoot.querySelector('[part="scroll"]');
    if (pre) pre.scrollTop = pre.scrollHeight;
  }
  render() {
    queueMicrotask(() => this.updated_scroll());
    const empty = this._buffer.length === 0;
    return b2`<pre
      part="scroll"
      ?data-empty=${empty}
      @scroll=${this._onScroll}
    >${empty ? "(no output yet)" : this._buffer}</pre>`;
  }
};
ShipLogStream.styles = i`
    :host {
      display: block;
      min-height: 8rem;
      max-height: 24rem;
      border-radius: var(--radius-md);
      border: 1px solid color-mix(in oklch, var(--border) 60%, transparent);
      background: color-mix(in oklch, var(--background) 95%, black 5%);
      color: var(--foreground);
      overflow: hidden;
    }
    [part='scroll'] {
      height: 100%;
      max-height: inherit;
      margin: 0;
      padding: var(--space-3) var(--space-4);
      font-family: ${MONO};
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
  `;
ShipLogStream.properties = {
  url: { type: String, reflect: true },
  event: { type: String, reflect: true },
  autoscroll: { type: Boolean, reflect: true },
  active: { type: Boolean, reflect: true },
  maxBytes: { type: Number, attribute: "max-bytes" }
};
defineOnce("ship-log-stream", ShipLogStream);

// src/web-components/observability/ship-event-row.ts
var ShipEventRow = class extends i4 {
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "listitem");
  }
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
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "tree");
    if (!this.hasAttribute("aria-label")) this.setAttribute("aria-label", "JSON tree");
  }
  render() {
    if (!this.data) return b2``;
    let parsed;
    try {
      parsed = JSON.parse(this.data);
    } catch (e6) {
      return b2`<span class="error" part="error" role="alert">Invalid JSON: ${e6.message}</span>`;
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
    .key { color: var(--primary); font-weight: var(--font-weight-medium); }
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
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "region");
    if (!this.hasAttribute("aria-label")) this.setAttribute("aria-label", "diff");
  }
  render() {
    const lines = this._lines();
    return b2`${lines.map((l3) => {
      const sign = l3.kind === "add" ? "+" : l3.kind === "del" ? "-" : l3.kind === "hunk" ? "@" : " ";
      const kindLabel = l3.kind === "add" ? "added" : l3.kind === "del" ? "removed" : l3.kind === "hunk" ? "hunk" : "context";
      return b2`<div class="line ${l3.kind}" part="line" role="listitem" aria-label=${`${kindLabel}: ${l3.text}`}><span class="sign" aria-hidden="true">${sign}</span><span>${l3.text}</span></div>`;
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
        <div class="block" part="block" role="region" aria-label=${this.language ? `${this.language} code block` : "code block"}>
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
    @media (forced-colors: active) {
      button:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
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

// src/web-components/observability/ship-table.styles.ts
var shipTableStyles = [
  reducedMotion,
  i`
    :host {
      display: block;
      width: 100%;
      font-family: var(--font-body);
      font-size: var(--text-base);
      color: var(--foreground);

      /* Density channel — default matches the pre-polish 10px/12px cells
       * the SDK table macro uses, so existing views don't shift a pixel. */
      --row-py: 10px;
      --row-px: var(--space-3);
      --th-py: 10px;
      --th-fs: var(--text-xs);
      --table-radius: var(--radius-lg);
      --row-hover-mix: 45%;
    }
    :host([hidden]) { display: none; }

    /* Density: compact — log-table feel. Legacy 'dense' aliases to this. */
    :host([density='compact']),
    :host([dense]) {
      --row-py: 6px;
      --row-px: var(--space-2);
      --th-py: 6px;
    }
    /* Density: comfortable — settings / admin feel. */
    :host([density='comfortable']) {
      --row-py: 14px;
      --row-px: var(--space-4);
      --th-py: 14px;
    }

    /* ------------------------------------------------------------------
     * Wrapper — source-of-truth surface. Border + radius + overflow
     * live here so the table inside can scroll horizontally on narrow
     * viewports without the border clipping mid-cell. Mirrors shadcn's
     * Table's wrapping <div>.
     * ------------------------------------------------------------------ */
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

    /* Sticky-header — pins <thead> while the body scrolls. The consumer
     * sets max-height on the host (inline style); we mirror it onto the
     * wrapper so the scroll container is the wrapper itself, which keeps
     * the border + radius intact around the scrolled body. */
    :host([sticky-header]) {
      display: block;
    }
    :host([sticky-header]) .wrap {
      overflow-y: auto;
      max-height: inherit;
    }

    /* ------------------------------------------------------------------
     * Empty state — rendered when the consumer slots content into the
     * 'empty' slot AND the tbody has no <tr> children. Visibility is
     * toggled via the 'data-empty' attribute set from the component's
     * slotchange / MutationObserver logic.
     * ------------------------------------------------------------------ */
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
  `
];
var shipTableLightDomCss = `
  ship-table {
    --ship-table-border-soft: color-mix(in oklch, var(--border) 80%, transparent);
  }

  /* In native mode the consumer's <table> is a direct child \u2014 reset
   * its own borders/spacing so the ship-table wrapper chrome owns the
   * surface and nothing double-lines. */
  ship-table[variant='native'] > table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    table-layout: auto;
    font-variant-numeric: tabular-nums;
  }

  /* Cell chrome \u2014 padding, baseline, separator. Reads density from the
   * host custom properties via inheritance. Scoped under ship-table so
   * we only claim cells inside the component, not arbitrary tables. */
  ship-table th,
  ship-table td {
    text-align: left;
    padding: var(--row-py, 10px) var(--row-px, 12px);
    vertical-align: middle;
    border-bottom: 1px solid var(--ship-table-border-soft);
    font-size: var(--text-base, 13px);
    color: var(--foreground);
    box-sizing: border-box;
  }
  ship-table th {
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
  ship-table thead {
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  /* Last body row \u2014 drop separator; the wrapper border closes the table.
   * Works for both the slot pattern (bare <tr> as direct child) and the
   * fragment pattern (<tr> inside <tbody>). */
  ship-table > tr:last-of-type > *,
  ship-table tbody tr:last-child > * {
    border-bottom: 0;
  }

  /* Column alignment. */
  ship-table th[align='right'], ship-table td[align='right'] { text-align: right; }
  ship-table th[align='center'], ship-table td[align='center'] { text-align: center; }

  /* Mono/muted conventions for the data-view SDK table + similar. */
  ship-table .mono {
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: var(--text-sm, 12px);
    letter-spacing: -0.01em;
  }
  ship-table .muted { color: var(--muted-foreground); }
  ship-table .null { color: var(--muted-foreground); font-style: italic; }

  /* Row hover \u2014 on by default, opt-out via hoverable="false". Applies
   * to body rows in both patterns: bare <tr> under ship-table, or <tr>
   * under <tbody>. Excludes the thead row via :not([slot='head']). */
  ship-table > tr:not([slot='head']),
  ship-table tbody tr {
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  ship-table:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  ship-table:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  /* Striped \u2014 zebra on alternate body rows. Both patterns. */
  ship-table[striped] > tr:not([slot='head']):nth-of-type(even) > *,
  ship-table[striped] tbody tr:nth-child(even) > * {
    background: color-mix(in oklch, var(--muted) 25%, transparent);
  }
  ship-table[striped]:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  ship-table[striped]:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  /* Selected row \u2014 data-selected attr or aria-selected="true". */
  ship-table tr[data-selected] > *,
  ship-table tr[aria-selected='true'] > * {
    background: color-mix(in oklch, var(--primary) 14%, transparent);
  }
  ship-table tr[data-selected] > *:first-child,
  ship-table tr[aria-selected='true'] > *:first-child {
    box-shadow: inset 3px 0 0 0 var(--primary);
  }

  /* Sticky-header \u2014 applies position:sticky to the thead cells so the
   * header rides above scrolling body rows. Works on both patterns
   * because the thead cells sit inside the shadow <thead> via slot
   * projection, and the slotted element inherits sticky semantics. */
  ship-table[sticky-header] thead th,
  ship-table[sticky-header] tr[slot='head'] > th {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky, 20);
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
    box-shadow: 0 1px 0 0 var(--border);
  }

  /* Sortable headers \u2014 chevron rendered by the component; these rules
   * paint the cursor + color transitions. */
  ship-table th[data-sort-key],
  ship-table th.sortable {
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    transition: color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  ship-table th[data-sort-key]:hover,
  ship-table th.sortable:hover {
    color: var(--foreground);
  }
  ship-table th[aria-sort='ascending'],
  ship-table th[aria-sort='descending'] {
    color: var(--foreground);
  }
  ship-table th[data-sort-key]:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 var(--ring-width, 3px)
      color-mix(in oklch, var(--ring-color, var(--primary)) var(--ring-opacity, 25%), transparent);
  }

  /* Empty-state \u2014 in native mode the <table> is a light-DOM child so
   * hiding it has to happen from the document stylesheet. The host
   * [data-empty] attribute is the single source of truth either way. */
  ship-table[data-empty] > table {
    display: none;
  }

  /* Forced colors \u2014 keep legibility in high-contrast mode. */
  @media (forced-colors: active) {
    ship-table th, ship-table td { border-bottom-color: CanvasText; }
  }
`;

// src/web-components/observability/ship-table.ts
var lightDomInstalled = /* @__PURE__ */ new WeakSet();
function installLightDomStyles(doc) {
  if (lightDomInstalled.has(doc)) return;
  lightDomInstalled.add(doc);
  try {
    if (typeof CSSStyleSheet !== "undefined" && "replaceSync" in CSSStyleSheet.prototype) {
      const sheet = new CSSStyleSheet();
      sheet.replaceSync(shipTableLightDomCss);
      doc.adoptedStyleSheets = [...doc.adoptedStyleSheets, sheet];
      return;
    }
  } catch {
  }
  const style = doc.createElement("style");
  style.setAttribute("data-ship-table", "");
  style.textContent = shipTableLightDomCss;
  doc.head.appendChild(style);
}
var ShipTable = class extends i4 {
  constructor() {
    super(...arguments);
    this.variant = "slot";
    this.density = "default";
    this.dense = false;
    this.hoverable = true;
    this.striped = false;
    this.stickyHeader = false;
    this.label = "";
    this.onHeaderClick = (e6) => {
      const th = e6.target.closest(
        "th[data-sort-key]"
      );
      if (!th || !this.contains(th)) return;
      this.cycleSort(th);
    };
    this.onHeaderKey = (e6) => {
      if (e6.key !== "Enter" && e6.key !== " ") return;
      const th = e6.target.closest(
        "th[data-sort-key]"
      );
      if (!th || !this.contains(th)) return;
      e6.preventDefault();
      this.cycleSort(th);
    };
  }
  connectedCallback() {
    super.connectedCallback();
    installLightDomStyles(this.ownerDocument);
    this.addEventListener("click", this.onHeaderClick);
    this.addEventListener("keydown", this.onHeaderKey);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener("click", this.onHeaderClick);
    this.removeEventListener("keydown", this.onHeaderKey);
    this.structureObserver?.disconnect();
  }
  firstUpdated() {
    this.enhanceSortableHeaders();
    this.syncEmptyState();
    this.structureObserver = new MutationObserver(() => {
      this.enhanceSortableHeaders();
      this.syncEmptyState();
    });
    this.structureObserver.observe(this, { childList: true, subtree: true });
  }
  /** Give sortable headers a tabindex, role, chevron, and an initial
   * aria-sort so keyboard users can cycle the sort direction. Idempotent
   * across re-runs so Datastar-patched tables stay enhanced. */
  enhanceSortableHeaders() {
    const ths = this.querySelectorAll(
      "th[data-sort-key]"
    );
    ths.forEach((th) => {
      if (!th.hasAttribute("aria-sort")) th.setAttribute("aria-sort", "none");
      if (!th.hasAttribute("tabindex")) th.setAttribute("tabindex", "0");
      if (!th.hasAttribute("role")) th.setAttribute("role", "columnheader");
      if (!th.querySelector(".ship-sort-indicator")) {
        const chev = document.createElement("span");
        chev.className = "ship-sort-indicator";
        chev.setAttribute("aria-hidden", "true");
        chev.style.cssText = "margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;";
        chev.innerHTML = '<svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M2 4l3-2 3 2M2 6l3 2 3-2"/></svg>';
        th.appendChild(chev);
      }
    });
    this.paintSortChevrons();
  }
  paintSortChevrons() {
    this.querySelectorAll("th[data-sort-key]").forEach(
      (th) => {
        const dir = th.getAttribute("aria-sort") || "none";
        const ind = th.querySelector(".ship-sort-indicator");
        if (!ind) return;
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
      }
    );
  }
  /** Toggle `[data-empty]` when the table has no body rows AND an
   * `empty` slot is provided. Covers all three structural patterns:
   *   - slot mode: <tr slot="head"> + bare <tr> default-slotted
   *   - native mode: <table><thead><tr/></thead><tbody><tr/></tbody></table>
   *   - native mode without explicit tbody */
  syncEmptyState() {
    const bodyRow = this.querySelector(":scope > table tbody > tr") || this.querySelector(":scope > table > tr:not(:has(th))") || this.querySelector(":scope > tbody > tr") || this.querySelector(':scope > tr:not([slot="head"])');
    const hasEmptySlot = this.querySelector('[slot="empty"]') != null;
    if (!bodyRow && hasEmptySlot) this.setAttribute("data-empty", "");
    else this.removeAttribute("data-empty");
  }
  /** Cycle: none -> ascending -> descending -> none. Clearing peers keeps
   * the table single-sort; multi-sort consumers can listen for ship-sort
   * and manage aria-sort themselves. */
  cycleSort(th) {
    const key = th.getAttribute("data-sort-key") || "";
    const current = th.getAttribute("aria-sort") || "none";
    const next = current === "none" ? "ascending" : current === "ascending" ? "descending" : "none";
    this.querySelectorAll("th[data-sort-key]").forEach(
      (peer) => {
        if (peer !== th) peer.setAttribute("aria-sort", "none");
      }
    );
    th.setAttribute("aria-sort", next);
    this.paintSortChevrons();
    this.dispatchEvent(
      new CustomEvent("ship-sort", {
        detail: { key, direction: next },
        bubbles: true,
        composed: true
      })
    );
  }
  render() {
    if (this.variant === "native") {
      return b2`
        <div class="wrap" part="wrap">
          <slot></slot>
          <div class="empty" part="empty" role="status">
            <slot name="empty"></slot>
          </div>
        </div>
      `;
    }
    return b2`
      <div class="wrap" part="wrap">
        <table part="table" aria-label=${this.label || A}>
          <thead part="thead"><slot name="head"></slot></thead>
          <tbody part="tbody"><slot></slot></tbody>
        </table>
        <div class="empty" part="empty" role="status">
          <slot name="empty"></slot>
        </div>
      </div>
    `;
  }
};
ShipTable.styles = shipTableStyles;
ShipTable.properties = {
  variant: { type: String, reflect: true },
  density: { type: String, reflect: true },
  dense: { type: Boolean, reflect: true },
  hoverable: { type: Boolean, reflect: true },
  striped: { type: Boolean, reflect: true },
  stickyHeader: { type: Boolean, reflect: true, attribute: "sticky-header" },
  label: { type: String }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-table")) {
  customElements.define("ship-table", ShipTable);
}

// src/web-components/observability/ship-timeline.ts
var ShipTimeline = class extends i4 {
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "list");
  }
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
    if (!this.hasAttribute("role")) this.setAttribute("role", "listitem");
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
      this.setAttribute("role", "button");
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
      @media (forced-colors: active) {
        :host([interactive]:focus-visible) { outline: 2px solid CanvasText; outline-offset: 2px; }
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
    /** Clicking the row both selects AND toggles expand/collapse. */
    this._rowClick = (e6) => {
      this._select();
      this._toggle(e6);
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
        aria-expanded=${this.leaf ? A : String(this.expanded)}
        aria-selected=${String(this.selected)}
        @click=${this._rowClick}
        @keydown=${this._onKey}
      >
        <span class="caret" part="caret" aria-hidden="true">
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
    @media (forced-colors: active) {
      .row:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
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
    this.label = "sparkline";
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
        aria-label=${this.label}
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
  fill: { type: Boolean, reflect: true },
  label: { type: String }
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
    this.label = "bar chart";
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
      <svg viewBox="0 0 ${w2} ${h3}" preserveAspectRatio="none" role="img" aria-label=${this.label}>
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
    svg { display: block; width: 100%; height: 100%; overflow: hidden; }
  `;
ShipBarChart.properties = {
  data: { type: String, reflect: true },
  labels: { type: String },
  tone: { type: String, reflect: true },
  orientation: { type: String, reflect: true },
  width: { type: Number },
  height: { type: Number },
  max: { type: Number },
  label: { type: String }
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
    this.label = "line chart";
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
        <svg viewBox="0 0 ${w2} ${h3}" preserveAspectRatio="none" role="img" aria-label=${this.label}>
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
    svg { display: block; width: 100%; flex: 1; overflow: hidden; }
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
  grid: { type: Boolean, reflect: true },
  label: { type: String }
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
    this.label = "area chart";
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
        <svg viewBox="0 0 ${w2} ${h3}" preserveAspectRatio="none" role="img" aria-label=${this.label}>
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
    svg { display: block; width: 100%; flex: 1; overflow: hidden; }
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
  grid: { type: Boolean, reflect: true },
  label: { type: String }
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
    this.label = "donut chart";
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
      <svg viewBox="0 0 ${s4} ${s4}" role="img" aria-label=${this.label}>
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
  thickness: { type: Number },
  label: { type: String }
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
    this.label = "gauge";
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
      <svg viewBox="0 0 ${s4} ${s4}" role="img" aria-label=${this.label}>
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
  thickness: { type: Number },
  label: { type: String }
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
    this.label = "";
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
        aria-valuemin="0" aria-valuemax="100" aria-valuenow=${pct}
        aria-label=${this.label || A}>
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
  tone: { type: String, reflect: true },
  label: { type: String, reflect: true }
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
    this.label = "heatmap";
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
      <div class="grid" style=${gridStyle} role="table" aria-label=${this.label}>
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
  tone: { type: String, reflect: true },
  label: { type: String }
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
    this.label = "meter";
  }
  render() {
    const segs = parseJson(this.segments, []);
    const sum = segs.reduce((a3, s4) => a3 + Math.max(0, s4.count), 0);
    const denom = this.total ?? sum;
    const safeDenom = denom <= 0 ? 1 : denom;
    return b2`
      <div class="wrap">
        <div class="bar" role="img" aria-label=${this.label}>
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
  total: { type: Number },
  label: { type: String }
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

// src/web-components/utility/ship-chip.styles.ts
var shipChipStyles = [
  reducedMotion,
  i`
    :host {
      display: inline-flex;
      vertical-align: middle;
      -webkit-tap-highlight-color: transparent;
      --tone: var(--muted-foreground);
      --tone-fg: var(--muted-foreground);
      --tone-contrast: var(--background);
    }
    :host([hidden]) { display: none; }

    /* Tone channel — same palette as ship-button so a row of mixed
     * chips + buttons share tone semantics. One-liner per tone keeps
     * additions cheap. */
    :host(:not([tone])),
    :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
    :host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
    :host([tone='info'])        { --tone: var(--info);              --tone-fg: var(--info);              --tone-contrast: oklch(0.99 0.02 260); }
    :host([tone='success'])     { --tone: var(--success);           --tone-fg: var(--success);           --tone-contrast: oklch(0.99 0.02 150); }
    :host([tone='warning'])     { --tone: var(--warning);           --tone-fg: var(--warning);           --tone-contrast: oklch(0.2 0.04 75); }
    :host([tone='destructive']) { --tone: var(--destructive);       --tone-fg: var(--destructive);       --tone-contrast: oklch(0.99 0.02 25); }
    :host([tone='accent'])      { --tone: var(--accent);            --tone-fg: var(--accent);            --tone-contrast: var(--accent-foreground); }

    /* Size channel — sm matches pre-polish filter rails; md/lg step up
     * proportionally. Heights +4px per step so sibling controls align. */
    :host(:not([size])),
    :host([size='md']) { --chip-h: 26px; --chip-px: 10px; --chip-fs: var(--text-sm);   --chip-gap: 6px; --chip-icon: 12px; --chip-remove: 16px; }
    :host([size='sm']) { --chip-h: 22px; --chip-px: 8px;  --chip-fs: var(--text-xs);   --chip-gap: 4px; --chip-icon: 10px; --chip-remove: 14px; }
    :host([size='lg']) { --chip-h: 30px; --chip-px: 12px; --chip-fs: var(--text-base); --chip-gap: 6px; --chip-icon: 14px; --chip-remove: 18px; }

    :host { --chip-radius: var(--radius-full); }

    /* ------------------------------------------------------------------
     * Core chip chrome. The host owns interaction — no inner <button>
     * — so Datastar's data-on:click on the host fires identically
     * whether the user hits the label, padding, or icon. Keyboard
     * activation (Enter / Space) synthesizes a host click in TS.
     * ------------------------------------------------------------------ */
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
      letter-spacing: -0.005em;
      color: var(--tone-fg);
      background: transparent;
      border: 1px solid transparent;
      border-radius: var(--chip-radius);
      cursor: pointer;
      user-select: none;
      -webkit-user-select: none;
      white-space: nowrap;
      isolation: isolate;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out),
        filter var(--duration-fast) var(--ease-out);
    }

    /* ------------------------------------------------------------------
     * Variant matrix — solid | soft | outline. Default (no variant
     * attr) is soft so filter rails stay legible against the page bg.
     * ------------------------------------------------------------------ */

    :host(:not([variant])) .chip,
    :host([variant='soft']) .chip {
      background: color-mix(in oklch, var(--tone) 14%, transparent);
      border-color: color-mix(in oklch, var(--tone) 22%, transparent);
      color: var(--tone-fg);
    }
    :host(:not([variant]):not([disabled])) .chip:hover,
    :host([variant='soft']:not([disabled])) .chip:hover {
      background: color-mix(in oklch, var(--tone) 22%, transparent);
      border-color: color-mix(in oklch, var(--tone) 32%, transparent);
    }
    :host(:not([variant]):not([disabled])) .chip:active,
    :host([variant='soft']:not([disabled])) .chip:active {
      background: color-mix(in oklch, var(--tone) 28%, transparent);
    }

    :host([variant='outline']) .chip {
      background: transparent;
      border-color: color-mix(in oklch, var(--tone) 45%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='outline']:not([tone])) .chip,
    :host([variant='outline'][tone='muted']) .chip {
      border-color: var(--border);
      color: var(--muted-foreground);
    }
    :host([variant='outline']:not([disabled])) .chip:hover {
      background: color-mix(in oklch, var(--tone) 10%, transparent);
      border-color: color-mix(in oklch, var(--tone) 70%, transparent);
    }
    :host([variant='outline']:not([tone]):not([disabled])) .chip:hover,
    :host([variant='outline'][tone='muted']:not([disabled])) .chip:hover {
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
      color: var(--foreground);
    }

    :host([variant='solid']) .chip {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      color: var(--tone-contrast);
      box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
    }
    :host([variant='solid']:not([disabled])) .chip:hover {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
        var(--shadow-xs);
    }

    /* ------------------------------------------------------------------
     * Selected state — Radix Toggle pressed look. Promotes any variant
     * to a solid tone fill so [soft][soft][selected][soft] reads the
     * active one unambiguously. Selected + muted uses --foreground as
     * the fill so the pill asserts neutrally instead of washing out.
     * ------------------------------------------------------------------ */
    :host([selected]) .chip {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 10%);
      color: var(--tone-contrast);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 20%, transparent),
        var(--shadow-xs);
      font-weight: var(--font-weight-semibold);
    }
    :host([selected]:not([disabled])) .chip:hover {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 24%, transparent),
        var(--shadow-sm);
    }
    :host([selected]:not([tone])) .chip,
    :host([selected][tone='muted']) .chip {
      background: var(--foreground);
      border-color: var(--foreground);
      color: var(--background);
    }
    :host([selected]:not([tone]):not([disabled])) .chip:hover,
    :host([selected][tone='muted']:not([disabled])) .chip:hover {
      background: color-mix(in oklch, var(--foreground) 88%, var(--background));
    }

    /* Focus ring — stacked box-shadow so the ring sits outside the pill
     * without reserving layout. Fires on :host(:focus-visible) so it
     * travels with Tab navigation and programmatic focus identically. */
    :host(:focus-visible) { outline: none; }
    :host(:focus-visible) .chip {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }
    :host([selected]:focus-visible) .chip {
      box-shadow:
        inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
        0 0 0 var(--ring-offset) var(--background),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
    }

    /* Disabled — muted, desaturated, uninteractive. Distinct from the
     * unselected look so users can tell when a chip is unreachable. */
    :host([disabled]) .chip {
      cursor: not-allowed;
      background: var(--muted);
      border-color: color-mix(in oklch, var(--border) 80%, transparent);
      color: color-mix(in oklch, var(--muted-foreground) 70%, transparent);
      filter: saturate(0.65);
      box-shadow: none;
    }

    /* ------------------------------------------------------------------
     * Leading slot — icon / status dot / avatar. Sized from the icon
     * channel so glyphs optically match cap-height. currentColor flows
     * so <ship-status-dot> tints with the chip tone for free.
     * ------------------------------------------------------------------ */
    .leading {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: var(--chip-icon);
      height: var(--chip-icon);
      flex-shrink: 0;
      color: currentColor;
    }
    .leading { display: none; }
    .leading:has(::slotted(*)) { display: inline-flex; }
    ::slotted([slot='leading']) {
      display: inline-flex;
      align-items: center;
      color: currentColor;
      width: 100%;
      height: 100%;
    }

    /* ------------------------------------------------------------------
     * Remove button — × affordance on the trailing edge. Not focusable
     * separately from the chip (tabindex=-1 in TS); clicking stops
     * propagation so the chip's click doesn't also fire. Keyboard
     * removal: Backspace / Delete on the focused chip (handled in TS).
     * ------------------------------------------------------------------ */
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
      -webkit-tap-highlight-color: transparent;
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
    :host([selected]) .remove:hover:not(:disabled) {
      background: color-mix(in oklch, white 24%, transparent);
    }
    .remove:focus-visible {
      outline: none;
      box-shadow: 0 0 0 2px color-mix(in oklch, var(--tone) 55%, transparent);
    }
    :host([disabled]) .remove { cursor: not-allowed; }

    /* Forced-colors — system colors for high-contrast mode. */
    @media (forced-colors: active) {
      .chip { border-color: CanvasText; color: CanvasText; }
      :host([selected]) .chip {
        background: Highlight;
        color: HighlightText;
        border-color: Highlight;
      }
      :host(:focus-visible) .chip {
        outline: 2px solid CanvasText;
        outline-offset: 2px;
      }
    }
  `
];

// src/web-components/utility/ship-chip.ts
var ShipChip = class extends i4 {
  constructor() {
    super();
    this.onKeydown = (e6) => {
      if (this.disabled) return;
      if (e6.key === "Enter" || e6.key === " ") {
        e6.preventDefault();
        this.click();
        return;
      }
      if (this.removable && (e6.key === "Backspace" || e6.key === "Delete")) {
        e6.preventDefault();
        this.dispatchRemove();
      }
    };
    this.onRemoveClick = (e6) => {
      e6.stopPropagation();
      if (this.disabled) return;
      this.dispatchRemove();
    };
    this.tone = "muted";
    this.variant = null;
    this.size = "md";
    this.selected = false;
    this.removable = false;
    this.disabled = false;
  }
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("tabindex") && !this.disabled) {
      this.setAttribute("tabindex", "0");
    }
    if (!this.hasAttribute("role")) {
      this.setAttribute("role", "button");
    }
    this.addEventListener("keydown", this.onKeydown);
  }
  disconnectedCallback() {
    this.removeEventListener("keydown", this.onKeydown);
    super.disconnectedCallback();
  }
  updated(changed) {
    if (changed.has("disabled")) {
      if (this.disabled) {
        this.setAttribute("aria-disabled", "true");
        this.setAttribute("tabindex", "-1");
      } else {
        this.removeAttribute("aria-disabled");
        if (this.getAttribute("tabindex") === "-1") {
          this.setAttribute("tabindex", "0");
        }
      }
    }
    if (changed.has("selected")) {
      this.setAttribute("aria-pressed", this.selected ? "true" : "false");
    }
  }
  dispatchRemove() {
    this.dispatchEvent(new CustomEvent("ship-remove", {
      detail: {},
      bubbles: true,
      composed: true
    }));
  }
  render() {
    return b2`
      <span class="chip" part="chip">
        <span class="leading" part="leading"><slot name="leading"></slot></span>
        <slot></slot>
        ${this.removable ? b2`
              <button
                class="remove"
                part="remove"
                type="button"
                aria-label="Remove"
                tabindex="-1"
                ?disabled=${this.disabled}
                @click=${this.onRemoveClick}
              >
                <svg viewBox="0 0 10 10" aria-hidden="true">
                  <path d="M2 2 L8 8 M8 2 L2 8" />
                </svg>
              </button>
            ` : A}
      </span>
    `;
  }
};
ShipChip.styles = shipChipStyles;
ShipChip.properties = {
  tone: { type: String, reflect: true },
  variant: { type: String, reflect: true },
  size: { type: String, reflect: true },
  selected: { type: Boolean, reflect: true },
  removable: { type: Boolean, reflect: true },
  disabled: { type: Boolean, reflect: true }
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
  updated() {
    if (!this.hasAttribute("role")) this.setAttribute("role", "group");
    if (this.keys) this.setAttribute("aria-label", `Keyboard shortcut: ${this.keys}`);
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
var HEIGHTS = {
  text: "16px",
  circle: "40px",
  rect: "100px"
};
var LINE_WIDTHS = ["100%", "80%", "60%"];
var ShipSkeleton = class extends i4 {
  constructor() {
    super(...arguments);
    this.variant = "text";
    this.width = "";
    this.height = "";
    this.lines = 1;
    this.animate = true;
  }
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "status");
    if (!this.hasAttribute("aria-live")) this.setAttribute("aria-live", "polite");
    if (!this.hasAttribute("aria-busy")) this.setAttribute("aria-busy", "true");
    if (!this.hasAttribute("aria-label")) this.setAttribute("aria-label", "Loading");
  }
  barStyle(index, count) {
    const parts = [];
    const w2 = this.width || (this.variant === "text" && count > 1 ? LINE_WIDTHS[Math.min(index, LINE_WIDTHS.length - 1)] : "100%");
    parts.push(`width:${w2}`);
    parts.push(`height:${this.height || HEIGHTS[this.variant]}`);
    return parts.join(";");
  }
  render() {
    const count = this.variant === "text" ? Math.max(1, this.lines || 1) : 1;
    return b2`${Array.from({ length: count }, (_2, i6) => b2`
      <span class="bar" part="bar" style=${this.barStyle(i6, count)}></span>
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
      animation: shimmer 1.8s ease-in-out infinite;
      border-radius: 9999px;
    }
    :host(:not([animate])) .bar,
    :host([animate='false']) .bar { animation: none; }
    :host([variant='circle']) .bar { border-radius: var(--radius-full); }
    :host([variant='rect']) .bar { border-radius: var(--radius-md); }
    :host([variant='text']) .bar + .bar { margin-top: 6px; }
    @keyframes shimmer {
      0% { background-position: 200% 0; }
      100% { background-position: -200% 0; }
    }
    @media (prefers-reduced-motion: reduce) {
      .bar { animation: none; }
    }
  `;
ShipSkeleton.properties = {
  variant: { type: String, reflect: true },
  width: { type: String },
  height: { type: String },
  lines: { type: Number },
  animate: { type: Boolean, reflect: true }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-skeleton")) {
  customElements.define("ship-skeleton", ShipSkeleton);
}

// src/web-components/utility/ship-toast-container.styles.ts
var shipToastContainerStyles = i`
  :host {
    position: fixed;
    z-index: var(--z-toast);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    pointer-events: none;
    max-width: min(440px, calc(100vw - 32px));
    /* Nudge inwards from the viewport edge uniformly; per-position blocks
     * below override which sides are anchored. */
    --toast-edge: 16px;
    padding: 0;
    margin: 0;
    box-sizing: border-box;
  }
  :host([hidden]) { display: none; }

  /* Default (unset) matches the legacy ship-toast-container contract:
   * top-right. Newly-appended toasts land at the bottom of the stack. */
  :host(:not([position])),
  :host([position='top-right']) {
    top: var(--toast-edge);
    right: var(--toast-edge);
    bottom: auto;
    left: auto;
    flex-direction: column;
    align-items: flex-end;
  }
  :host([position='bottom-right']) {
    bottom: var(--toast-edge);
    right: var(--toast-edge);
    top: auto;
    left: auto;
    flex-direction: column-reverse;
    align-items: flex-end;
  }
  :host([position='bottom-left']) {
    bottom: var(--toast-edge);
    left: var(--toast-edge);
    top: auto;
    right: auto;
    flex-direction: column-reverse;
    align-items: flex-start;
  }
  :host([position='bottom-center']) {
    bottom: var(--toast-edge);
    left: 50%;
    transform: translateX(-50%);
    top: auto;
    right: auto;
    flex-direction: column-reverse;
    align-items: center;
  }
  :host([position='top-left']) {
    top: var(--toast-edge);
    left: var(--toast-edge);
    bottom: auto;
    right: auto;
    flex-direction: column;
    align-items: flex-start;
  }
  :host([position='top-center']) {
    top: var(--toast-edge);
    left: 50%;
    transform: translateX(-50%);
    bottom: auto;
    right: auto;
    flex-direction: column;
    align-items: center;
  }

  /* Every child toast manages its own pointer-events, but allow pointer
   * interactions on slotted content so hover-to-pause works across the
   * shadow boundary. */
  ::slotted(ship-toast) {
    pointer-events: auto;
    width: auto;
  }
`;

// src/web-components/utility/ship-toast-container.ts
var SLIDE_FROM = {
  "top-left": "top",
  "top-center": "top",
  "top-right": "top",
  "bottom-left": "bottom",
  "bottom-center": "bottom",
  "bottom-right": "bottom"
};
var ShipToastContainer = class extends i4 {
  constructor() {
    super(...arguments);
    this.position = "top-right";
    this.maxVisible = 5;
    /** Propagate slide-direction hints to every child so animations are
     *  position-aware, and enforce max-visible. */
    this._onSlotChange = (e6) => {
      const slot = e6.target;
      const dir = SLIDE_FROM[this.position] ?? "bottom";
      const children = slot.assignedElements({ flatten: true }).filter((n4) => n4.tagName?.toLowerCase() === "ship-toast");
      for (const t4 of children) {
        t4.setAttribute("data-slide-from", dir);
      }
      this._enforceMaxVisible(children);
    };
  }
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("role")) this.setAttribute("role", "region");
    if (!this.hasAttribute("aria-label"))
      this.setAttribute("aria-label", "Notifications");
    if (!this.hasAttribute("aria-live")) this.setAttribute("aria-live", "polite");
  }
  _enforceMaxVisible(children) {
    if (this.maxVisible <= 0 || children.length <= this.maxVisible) return;
    const overflow = children.length - this.maxVisible;
    for (let i6 = 0; i6 < overflow; i6++) {
      const victim = children[i6];
      if (typeof victim.close === "function") victim.close("programmatic");
      else victim.dismiss?.("programmatic");
    }
  }
  updated(changed) {
    if (changed.has("position")) {
      const dir = SLIDE_FROM[this.position] ?? "bottom";
      for (const child of this.querySelectorAll("ship-toast")) {
        child.setAttribute("data-slide-from", dir);
      }
    }
  }
  /**
   * Programmatic toast API. Creates a <ship-toast>, populates slots from
   * the options, appends to the container, and returns the element so
   * call-sites can `await toast.close()` or subscribe to ship-dismiss.
   *
   * Matches the sonner/shadcn ergonomics while staying framework-free so
   * daemon-rendered views can call it from Datastar-emitted inline scripts.
   */
  show(opts = {}) {
    const doc = this.ownerDocument ?? document;
    const t4 = doc.createElement("ship-toast");
    if (opts.tone) t4.setAttribute("tone", opts.tone);
    if (opts.size) t4.setAttribute("size", opts.size);
    if (typeof opts.duration === "number") t4.duration = opts.duration;
    t4.dismissible = opts.dismissible !== false;
    if (opts.dismissible === false) t4.removeAttribute("dismissible");
    else t4.setAttribute("dismissible", "");
    if (opts.showProgress) t4.setAttribute("show-progress", "");
    if (opts.title) {
      const title = doc.createElement("span");
      title.setAttribute("slot", "title");
      title.textContent = opts.title;
      t4.appendChild(title);
    }
    if (opts.description) {
      t4.appendChild(doc.createTextNode(opts.description));
    }
    if (opts.actionHtml) {
      const wrap = doc.createElement("span");
      wrap.setAttribute("slot", "actions");
      wrap.innerHTML = opts.actionHtml;
      t4.appendChild(wrap);
    }
    this.appendChild(t4);
    return t4;
  }
  render() {
    return b2`<slot @slotchange=${this._onSlotChange}></slot>`;
  }
};
ShipToastContainer.styles = shipToastContainerStyles;
ShipToastContainer.properties = {
  position: { type: String, reflect: true },
  maxVisible: { type: Number, reflect: true, attribute: "max-visible" }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-toast-container")) {
  customElements.define("ship-toast-container", ShipToastContainer);
}

// src/web-components/utility/ship-toast.styles.ts
var shipToastStyles = [
  reducedMotion,
  toneVars,
  i`
    :host {
      display: block;
      /* Size channel defaults (= md). Overridden per :host([size=…]) below. */
      --toast-px: 14px;
      --toast-py: 12px;
      --toast-fs: var(--text-base);
      --toast-title-fs: var(--text-base);
      --toast-gap: 10px;
      --toast-min-w: 280px;
      --toast-max-w: 420px;
      /* Default slide direction (right edge). Container sets
       * data-slide-from so positions on the left/top slide in from the
       * correct edge. */
      --toast-slide-x: 16px;
      --toast-slide-y: 0;
      pointer-events: auto;
      will-change: transform, opacity, max-height, margin;
    }
    :host([hidden]) { display: none; }

    :host([size='sm']) {
      --toast-px: 12px;
      --toast-py: 10px;
      --toast-fs: var(--text-sm);
      --toast-title-fs: var(--text-sm);
      --toast-gap: 8px;
      --toast-min-w: 240px;
      --toast-max-w: 360px;
    }
    :host([size='lg']) {
      --toast-px: 16px;
      --toast-py: 14px;
      --toast-fs: var(--text-md);
      --toast-title-fs: var(--text-md);
      --toast-gap: 12px;
      --toast-min-w: 320px;
      --toast-max-w: 460px;
    }

    /* Container tells us which edge to slide from via data-slide-from.
     * Matches the six container positions with a small set of transforms. */
    :host([data-slide-from='right'])  { --toast-slide-x: 16px;  --toast-slide-y: 0; }
    :host([data-slide-from='left'])   { --toast-slide-x: -16px; --toast-slide-y: 0; }
    :host([data-slide-from='top'])    { --toast-slide-x: 0;     --toast-slide-y: -12px; }
    :host([data-slide-from='bottom']) { --toast-slide-x: 0;     --toast-slide-y: 12px; }

    /* Entrance animation — runs once on mount. Uses the slide channel so
     * position-aware direction comes for free. */
    @keyframes ship-toast-in {
      from {
        opacity: 0;
        transform: translate3d(var(--toast-slide-x), var(--toast-slide-y), 0)
          scale(0.98);
      }
      to {
        opacity: 1;
        transform: translate3d(0, 0, 0) scale(1);
      }
    }

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
      border-radius: var(--radius-md);
      border: 1px solid var(--tone-border);
      background: var(--card);
      color: var(--card-foreground);
      box-shadow:
        0 0 0 1px color-mix(in oklch, black 3%, transparent),
        var(--shadow-md);
      font-family: var(--font-body);
      font-size: var(--toast-fs);
      line-height: var(--leading-normal);
      position: relative;
      overflow: hidden;
      isolation: isolate;
      animation: ship-toast-in var(--duration-medium) var(--ease-out) both;
    }

    /* Default (no tone) keeps the neutral card surface but drops a subtle
     * 2px accent rail on the leading edge so the toast reads as a card,
     * not a sticky note. */
    :host(:not([tone])) .toast,
    :host([tone='default']) .toast {
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
    }

    /* Tone-tinted variants — blend the tone wash with the card surface so
     * text keeps its contrast. The rail (::before) renders the tone at full
     * saturation for a crisp visual anchor. One combined selector covers
     * every semantic tone; the muted/default tones keep the neutral card. */
    :host([tone='info']) .toast, :host([tone='success']) .toast,
    :host([tone='warning']) .toast, :host([tone='destructive']) .toast,
    :host([tone='accent']) .toast {
      background: color-mix(in oklch, var(--tone-bg) 70%, var(--card));
      border-color: var(--tone-border);
    }

    /* Leading accent rail — 3px tinted stripe. Renders as ::before so the
     * rail follows the border-radius automatically. */
    .toast::before {
      content: '';
      position: absolute;
      inset-inline-start: 0;
      top: 0; bottom: 0;
      width: 3px;
      background: var(--tone-fg);
      opacity: 0;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    :host([tone='info']) .toast::before, :host([tone='success']) .toast::before,
    :host([tone='warning']) .toast::before, :host([tone='destructive']) .toast::before,
    :host([tone='accent']) .toast::before { opacity: 1; }

    /* Icon slot — sits top-left, sized to harmonize with the title line. */
    .icon {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 18px;
      height: 18px;
      flex: 0 0 auto;
      color: var(--tone-fg);
      margin-top: 1px;
    }
    :host(:not([tone])) .icon,
    :host([tone='default']) .icon { color: var(--muted-foreground); }
    .icon svg { width: 18px; height: 18px; display: block; }
    :host([size='sm']) .icon,
    :host([size='sm']) .icon svg { width: 16px; height: 16px; }
    :host([size='lg']) .icon,
    :host([size='lg']) .icon svg { width: 20px; height: 20px; }

    /* When the consumer doesn't slot an icon and we have no tone icon to
     * auto-fill (tone=default without data-has-icon), collapse the column
     * so the content flows to the leading edge. */
    :host([data-no-icon]) .toast {
      grid-template-columns: 1fr auto;
    }
    :host([data-no-icon]) .icon { display: none; }

    .content {
      display: flex;
      flex-direction: column;
      gap: 2px;
      min-width: 0;
      color: var(--card-foreground);
    }
    .title {
      font-weight: var(--font-weight-semibold);
      font-size: var(--toast-title-fs);
      letter-spacing: -0.005em;
      line-height: var(--leading-tight);
      color: var(--foreground);
    }
    .body {
      font-size: var(--toast-fs);
      color: var(--muted-foreground);
      line-height: var(--leading-normal);
      word-break: break-word;
    }
    /* When there's no title the body promotes to the stronger color so a
     * single-line toast still reads as the primary message. */
    :host([data-no-title]) .body { color: var(--card-foreground); }
    :host([data-no-title]) .title { display: none; }

    .actions {
      display: flex;
      gap: var(--space-2);
      margin-top: 6px;
      grid-column: 2;
    }
    :host([data-no-actions]) .actions { display: none; margin-top: 0; }

    /* Close button — top-right, ghost affordance that picks up tone on
     * hover. Matches the ship-dialog close in size and motion. */
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
      border-radius: var(--radius-sm);
      color: var(--muted-foreground);
      cursor: pointer;
      padding: 0;
      font: inherit;
      z-index: 1;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }
    .close:hover {
      background: color-mix(in oklch, var(--foreground) 8%, transparent);
      color: var(--foreground);
    }
    .close:active { transform: scale(0.94); }
    .close:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 var(--ring-offset) var(--card),
        0 0 0 calc(var(--ring-offset) + var(--ring-width))
          color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
    }
    .close svg { width: 12px; height: 12px; display: block; }
    :host(:not([dismissible])) .close { display: none; }

    /* Progress bar — 2px strip at the bottom that counts down from full to
     * empty over the configured duration. Opt-in via [show-progress].
     * Paused on host hover (CSS) and host-level data-paused (JS driven). */
    .progress {
      position: absolute;
      left: 0;
      right: 0;
      bottom: 0;
      height: 2px;
      background: var(--tone-fg);
      transform: scaleX(1);
      transform-origin: left center;
      animation: ship-toast-progress var(--toast-duration, 5000ms) linear forwards;
    }
    :host(:not([tone])) .progress,
    :host([tone='default']) .progress { background: var(--muted-foreground); }
    :host(:hover) .progress,
    :host([data-paused]) .progress { animation-play-state: paused; }
    @keyframes ship-toast-progress {
      from { transform: scaleX(1); }
      to   { transform: scaleX(0); }
    }
    :host(:not([show-progress])) .progress { display: none; }

    /* Closing state — opt-in exit animation. Consumers invoke .close() to
     * fade + collapse the toast before final removal. The sync dismiss()
     * path skips this so existing unit tests keep their synchronous
     * isConnected === false expectation. */
    :host([data-closing]) {
      animation: ship-toast-out var(--duration-medium) var(--ease-in) forwards;
    }
    @keyframes ship-toast-out {
      to {
        opacity: 0;
        transform: translate3d(var(--toast-slide-x), var(--toast-slide-y), 0)
          scale(0.96);
      }
    }

    /* Forced-colors — restore a visible edge when color-mix is flattened. */
    @media (forced-colors: active) {
      .toast { border: 1px solid CanvasText; }
      .close:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
    }
  `
];

// src/web-components/utility/ship-toast.ts
var TONE_ICON = {
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 8h.01"/><path d="M11 12h1v4h1"/>',
  success: '<circle cx="12" cy="12" r="9"/><path d="m8.5 12.5 2.5 2.5 4.5-5"/>',
  warning: '<path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"/><line x1="12" x2="12" y1="9" y2="13"/><line x1="12" x2="12.01" y1="17" y2="17"/>',
  destructive: '<circle cx="12" cy="12" r="9"/><line x1="15" x2="9" y1="9" y2="15"/><line x1="9" x2="15" y1="9" y2="15"/>',
  accent: '<path d="M12 2 3 14h7l-1 8 10-12h-7l1-8Z"/>'
};
var ShipToast = class extends i4 {
  constructor() {
    super(...arguments);
    this.size = "md";
    this.duration = 5e3;
    this.dismissible = true;
    this.showProgress = false;
    this._timer = null;
    this._remaining = 0;
    this._startedAt = 0;
    this._removed = false;
    this._onPause = () => {
      if (!this._timer) return;
      const elapsed = Date.now() - this._startedAt;
      this._remaining = Math.max(0, this._remaining - elapsed);
      this._clearTimer();
      this.toggleAttribute("data-paused", true);
    };
    this._onResume = () => {
      if (this._removed || this.duration <= 0) return;
      if (this._remaining <= 0) return;
      this.toggleAttribute("data-paused", false);
      this._startedAt = Date.now();
      this._timer = setTimeout(() => this.dismiss("timeout"), this._remaining);
    };
    this._onKeydown = (e6) => {
      if (e6.key === "Escape" && this.dismissible) {
        e6.stopPropagation();
        this.dismiss("manual");
      }
    };
    this._onCloseClick = () => this.dismiss("manual");
    this._handleSlotChange = (e6) => {
      const slot = e6.target;
      const name = slot.name || "default";
      const filled = slot.assignedNodes({ flatten: true }).some((n4) => {
        if (n4.nodeType === Node.ELEMENT_NODE) return true;
        return n4.textContent?.trim().length ?? 0 > 0;
      });
      if (name === "icon") this.toggleAttribute("data-has-slotted-icon", filled);
      if (name === "title") this.toggleAttribute("data-no-title", !filled);
      if (name === "actions") this.toggleAttribute("data-no-actions", !filled);
    };
  }
  connectedCallback() {
    super.connectedCallback();
    if (!this.hasAttribute("tabindex")) this.tabIndex = 0;
    this._scheduleDismiss();
    this.addEventListener("mouseenter", this._onPause);
    this.addEventListener("mouseleave", this._onResume);
    this.addEventListener("focusin", this._onPause);
    this.addEventListener("focusout", this._onResume);
    this.addEventListener("keydown", this._onKeydown);
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this._clearTimer();
    this.removeEventListener("mouseenter", this._onPause);
    this.removeEventListener("mouseleave", this._onResume);
    this.removeEventListener("focusin", this._onPause);
    this.removeEventListener("focusout", this._onResume);
    this.removeEventListener("keydown", this._onKeydown);
  }
  _scheduleDismiss() {
    this._clearTimer();
    if (this.duration > 0) {
      this._remaining = this.duration;
      this._startedAt = Date.now();
      this._timer = setTimeout(() => this.dismiss("timeout"), this.duration);
      this.style.setProperty("--toast-duration", `${this.duration}ms`);
    }
  }
  _clearTimer() {
    if (this._timer) {
      clearTimeout(this._timer);
      this._timer = null;
    }
  }
  /**
   * Synchronously fire `ship-dismiss` and detach the host. Used by the
   * close button and auto-timeout. Matches the contract tested in
   * ship-toast.test.ts (isConnected → false immediately after click).
   */
  dismiss(reason = "manual") {
    if (this._removed) return;
    this._removed = true;
    this._clearTimer();
    this.dispatchEvent(new CustomEvent("ship-dismiss", {
      detail: { reason },
      bubbles: true,
      composed: true
    }));
    this.remove();
  }
  /**
   * Animated dismissal — plays the exit keyframe, then removes. Useful for
   * galleries and long-lived notifications where the extra 180ms feels
   * more considered than a pop-out. Falls back to sync dismiss when
   * prefers-reduced-motion is set.
   */
  async close(reason = "manual") {
    if (this._removed) return;
    const prefersReduce = typeof window !== "undefined" && window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    if (prefersReduce) {
      this.dismiss(reason);
      return;
    }
    this._clearTimer();
    this.toggleAttribute("data-closing", true);
    await new Promise((resolve) => {
      const onEnd = () => {
        this.removeEventListener("animationend", onEnd);
        resolve();
      };
      this.addEventListener("animationend", onEnd);
      setTimeout(() => resolve(), 260);
    });
    this.dismiss(reason);
  }
  _iconMarkup() {
    const key = this.tone ?? "default";
    const svg2 = TONE_ICON[key];
    if (!svg2) return b2`<slot name="icon"></slot>`;
    return b2`
      <slot name="icon" @slotchange=${this._handleSlotChange}>
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
          .innerHTML=${svg2}
        ></svg>
      </slot>
    `;
  }
  render() {
    const assertive = this.tone === "destructive" || this.tone === "warning";
    const role = assertive ? "alert" : "status";
    const live = assertive ? "assertive" : "polite";
    const hasAutoIcon = !!this.tone && this.tone !== "default" && this.tone !== "muted";
    this.toggleAttribute("data-no-icon", !hasAutoIcon && !this.hasAttribute("data-has-slotted-icon"));
    return b2`
      <div class="toast" role=${role} aria-live=${live} aria-atomic="true" part="toast">
        <span class="icon">${this._iconMarkup()}</span>
        <div class="content">
          <div class="title">
            <slot name="title" @slotchange=${this._handleSlotChange}></slot>
          </div>
          <div class="body"><slot></slot></div>
          <div class="actions">
            <slot name="actions" @slotchange=${this._handleSlotChange}></slot>
          </div>
        </div>
        ${this.dismissible ? b2`
              <button
                class="close"
                part="close"
                type="button"
                aria-label="Dismiss notification"
                @click=${this._onCloseClick}
              >
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            ` : null}
        ${this.showProgress && this.duration > 0 ? b2`<span class="progress" aria-hidden="true"></span>` : null}
      </div>
    `;
  }
};
ShipToast.styles = shipToastStyles;
ShipToast.properties = {
  tone: { type: String, reflect: true },
  size: { type: String, reflect: true },
  duration: { type: Number },
  dismissible: { type: Boolean, reflect: true },
  showProgress: { type: Boolean, reflect: true, attribute: "show-progress" }
};
if (typeof customElements !== "undefined" && !customElements.get("ship-toast")) {
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
        ${this.external ? b2`<span class="sr-only" style="position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0,0,0,0);">(opens in new tab)</span>
        <span class="ext" aria-hidden="true">
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
  ShipAccordion,
  ShipAccordionItem,
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
  ShipCollapsible,
  ShipContextMenu,
  ShipCopyButton,
  ShipDialog,
  ShipDiff,
  ShipDonut,
  ShipDropdownMenu,
  ShipEmptyState,
  ShipEventRow,
  ShipField,
  ShipFileUpload,
  ShipGauge,
  ShipGitDiff,
  ShipGitDiffFile,
  ShipGitDiffHunk,
  ShipGitDiffLine,
  ShipHeatmap,
  ShipHistoryRow,
  ShipHoverCard,
  ShipIcon,
  ShipInput,
  ShipJsonView,
  ShipKbd,
  ShipLabel,
  ShipLineChart,
  ShipLink,
  ShipListItem,
  ShipLogLine,
  ShipLogStream,
  ShipMarkdown,
  ShipMetachip,
  ShipMeter,
  ShipNavigationItem,
  ShipNavigationMenu,
  ShipPagination,
  ShipPipelineDag,
  ShipPipelineDefinition,
  ShipPipelineRun,
  ShipPipelineStep,
  ShipPopover,
  ShipProgress,
  ShipProgressRing,
  ShipRadio,
  ShipRadioGroup,
  ShipScrollArea,
  ShipSelect,
  ShipSeparator,
  ShipSkeleton,
  ShipSlider,
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
  ShipToggle,
  ShipToggleGroup,
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
