// ../../node_modules/.pnpm/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/css-tag.js
var t = globalThis;
var e = t.ShadowRoot && (void 0 === t.ShadyCSS || t.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype;
var s = /* @__PURE__ */ Symbol();
var o = /* @__PURE__ */ new WeakMap();
var n = class {
  constructor(t3, e4, o5) {
    if (this._$cssResult$ = true, o5 !== s) throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");
    this.cssText = t3, this.t = e4;
  }
  get styleSheet() {
    let t3 = this.o;
    const s4 = this.t;
    if (e && void 0 === t3) {
      const e4 = void 0 !== s4 && 1 === s4.length;
      e4 && (t3 = o.get(s4)), void 0 === t3 && ((this.o = t3 = new CSSStyleSheet()).replaceSync(this.cssText), e4 && o.set(s4, t3));
    }
    return t3;
  }
  toString() {
    return this.cssText;
  }
};
var r = (t3) => new n("string" == typeof t3 ? t3 : t3 + "", void 0, s);
var i = (t3, ...e4) => {
  const o5 = 1 === t3.length ? t3[0] : e4.reduce((e5, s4, o6) => e5 + ((t4) => {
    if (true === t4._$cssResult$) return t4.cssText;
    if ("number" == typeof t4) return t4;
    throw Error("Value passed to 'css' function must be a 'css' function result: " + t4 + ". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.");
  })(s4) + t3[o6 + 1], t3[0]);
  return new n(o5, t3, s);
};
var S = (s4, o5) => {
  if (e) s4.adoptedStyleSheets = o5.map((t3) => t3 instanceof CSSStyleSheet ? t3 : t3.styleSheet);
  else for (const e4 of o5) {
    const o6 = document.createElement("style"), n4 = t.litNonce;
    void 0 !== n4 && o6.setAttribute("nonce", n4), o6.textContent = e4.cssText, s4.appendChild(o6);
  }
};
var c = e ? (t3) => t3 : (t3) => t3 instanceof CSSStyleSheet ? ((t4) => {
  let e4 = "";
  for (const s4 of t4.cssRules) e4 += s4.cssText;
  return r(e4);
})(t3) : t3;

// ../../node_modules/.pnpm/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/reactive-element.js
var { is: i2, defineProperty: e2, getOwnPropertyDescriptor: h, getOwnPropertyNames: r2, getOwnPropertySymbols: o2, getPrototypeOf: n2 } = Object;
var a = globalThis;
var c2 = a.trustedTypes;
var l = c2 ? c2.emptyScript : "";
var p = a.reactiveElementPolyfillSupport;
var d = (t3, s4) => t3;
var u = { toAttribute(t3, s4) {
  switch (s4) {
    case Boolean:
      t3 = t3 ? l : null;
      break;
    case Object:
    case Array:
      t3 = null == t3 ? t3 : JSON.stringify(t3);
  }
  return t3;
}, fromAttribute(t3, s4) {
  let i5 = t3;
  switch (s4) {
    case Boolean:
      i5 = null !== t3;
      break;
    case Number:
      i5 = null === t3 ? null : Number(t3);
      break;
    case Object:
    case Array:
      try {
        i5 = JSON.parse(t3);
      } catch (t4) {
        i5 = null;
      }
  }
  return i5;
} };
var f = (t3, s4) => !i2(t3, s4);
var b = { attribute: true, type: String, converter: u, reflect: false, useDefault: false, hasChanged: f };
Symbol.metadata ?? (Symbol.metadata = /* @__PURE__ */ Symbol("metadata")), a.litPropertyMetadata ?? (a.litPropertyMetadata = /* @__PURE__ */ new WeakMap());
var y = class extends HTMLElement {
  static addInitializer(t3) {
    this._$Ei(), (this.l ?? (this.l = [])).push(t3);
  }
  static get observedAttributes() {
    return this.finalize(), this._$Eh && [...this._$Eh.keys()];
  }
  static createProperty(t3, s4 = b) {
    if (s4.state && (s4.attribute = false), this._$Ei(), this.prototype.hasOwnProperty(t3) && ((s4 = Object.create(s4)).wrapped = true), this.elementProperties.set(t3, s4), !s4.noAccessor) {
      const i5 = /* @__PURE__ */ Symbol(), h3 = this.getPropertyDescriptor(t3, i5, s4);
      void 0 !== h3 && e2(this.prototype, t3, h3);
    }
  }
  static getPropertyDescriptor(t3, s4, i5) {
    const { get: e4, set: r4 } = h(this.prototype, t3) ?? { get() {
      return this[s4];
    }, set(t4) {
      this[s4] = t4;
    } };
    return { get: e4, set(s5) {
      const h3 = e4?.call(this);
      r4?.call(this, s5), this.requestUpdate(t3, h3, i5);
    }, configurable: true, enumerable: true };
  }
  static getPropertyOptions(t3) {
    return this.elementProperties.get(t3) ?? b;
  }
  static _$Ei() {
    if (this.hasOwnProperty(d("elementProperties"))) return;
    const t3 = n2(this);
    t3.finalize(), void 0 !== t3.l && (this.l = [...t3.l]), this.elementProperties = new Map(t3.elementProperties);
  }
  static finalize() {
    if (this.hasOwnProperty(d("finalized"))) return;
    if (this.finalized = true, this._$Ei(), this.hasOwnProperty(d("properties"))) {
      const t4 = this.properties, s4 = [...r2(t4), ...o2(t4)];
      for (const i5 of s4) this.createProperty(i5, t4[i5]);
    }
    const t3 = this[Symbol.metadata];
    if (null !== t3) {
      const s4 = litPropertyMetadata.get(t3);
      if (void 0 !== s4) for (const [t4, i5] of s4) this.elementProperties.set(t4, i5);
    }
    this._$Eh = /* @__PURE__ */ new Map();
    for (const [t4, s4] of this.elementProperties) {
      const i5 = this._$Eu(t4, s4);
      void 0 !== i5 && this._$Eh.set(i5, t4);
    }
    this.elementStyles = this.finalizeStyles(this.styles);
  }
  static finalizeStyles(s4) {
    const i5 = [];
    if (Array.isArray(s4)) {
      const e4 = new Set(s4.flat(1 / 0).reverse());
      for (const s5 of e4) i5.unshift(c(s5));
    } else void 0 !== s4 && i5.push(c(s4));
    return i5;
  }
  static _$Eu(t3, s4) {
    const i5 = s4.attribute;
    return false === i5 ? void 0 : "string" == typeof i5 ? i5 : "string" == typeof t3 ? t3.toLowerCase() : void 0;
  }
  constructor() {
    super(), this._$Ep = void 0, this.isUpdatePending = false, this.hasUpdated = false, this._$Em = null, this._$Ev();
  }
  _$Ev() {
    this._$ES = new Promise((t3) => this.enableUpdating = t3), this._$AL = /* @__PURE__ */ new Map(), this._$E_(), this.requestUpdate(), this.constructor.l?.forEach((t3) => t3(this));
  }
  addController(t3) {
    (this._$EO ?? (this._$EO = /* @__PURE__ */ new Set())).add(t3), void 0 !== this.renderRoot && this.isConnected && t3.hostConnected?.();
  }
  removeController(t3) {
    this._$EO?.delete(t3);
  }
  _$E_() {
    const t3 = /* @__PURE__ */ new Map(), s4 = this.constructor.elementProperties;
    for (const i5 of s4.keys()) this.hasOwnProperty(i5) && (t3.set(i5, this[i5]), delete this[i5]);
    t3.size > 0 && (this._$Ep = t3);
  }
  createRenderRoot() {
    const t3 = this.shadowRoot ?? this.attachShadow(this.constructor.shadowRootOptions);
    return S(t3, this.constructor.elementStyles), t3;
  }
  connectedCallback() {
    this.renderRoot ?? (this.renderRoot = this.createRenderRoot()), this.enableUpdating(true), this._$EO?.forEach((t3) => t3.hostConnected?.());
  }
  enableUpdating(t3) {
  }
  disconnectedCallback() {
    this._$EO?.forEach((t3) => t3.hostDisconnected?.());
  }
  attributeChangedCallback(t3, s4, i5) {
    this._$AK(t3, i5);
  }
  _$ET(t3, s4) {
    const i5 = this.constructor.elementProperties.get(t3), e4 = this.constructor._$Eu(t3, i5);
    if (void 0 !== e4 && true === i5.reflect) {
      const h3 = (void 0 !== i5.converter?.toAttribute ? i5.converter : u).toAttribute(s4, i5.type);
      this._$Em = t3, null == h3 ? this.removeAttribute(e4) : this.setAttribute(e4, h3), this._$Em = null;
    }
  }
  _$AK(t3, s4) {
    const i5 = this.constructor, e4 = i5._$Eh.get(t3);
    if (void 0 !== e4 && this._$Em !== e4) {
      const t4 = i5.getPropertyOptions(e4), h3 = "function" == typeof t4.converter ? { fromAttribute: t4.converter } : void 0 !== t4.converter?.fromAttribute ? t4.converter : u;
      this._$Em = e4;
      const r4 = h3.fromAttribute(s4, t4.type);
      this[e4] = r4 ?? this._$Ej?.get(e4) ?? r4, this._$Em = null;
    }
  }
  requestUpdate(t3, s4, i5, e4 = false, h3) {
    if (void 0 !== t3) {
      const r4 = this.constructor;
      if (false === e4 && (h3 = this[t3]), i5 ?? (i5 = r4.getPropertyOptions(t3)), !((i5.hasChanged ?? f)(h3, s4) || i5.useDefault && i5.reflect && h3 === this._$Ej?.get(t3) && !this.hasAttribute(r4._$Eu(t3, i5)))) return;
      this.C(t3, s4, i5);
    }
    false === this.isUpdatePending && (this._$ES = this._$EP());
  }
  C(t3, s4, { useDefault: i5, reflect: e4, wrapped: h3 }, r4) {
    i5 && !(this._$Ej ?? (this._$Ej = /* @__PURE__ */ new Map())).has(t3) && (this._$Ej.set(t3, r4 ?? s4 ?? this[t3]), true !== h3 || void 0 !== r4) || (this._$AL.has(t3) || (this.hasUpdated || i5 || (s4 = void 0), this._$AL.set(t3, s4)), true === e4 && this._$Em !== t3 && (this._$Eq ?? (this._$Eq = /* @__PURE__ */ new Set())).add(t3));
  }
  async _$EP() {
    this.isUpdatePending = true;
    try {
      await this._$ES;
    } catch (t4) {
      Promise.reject(t4);
    }
    const t3 = this.scheduleUpdate();
    return null != t3 && await t3, !this.isUpdatePending;
  }
  scheduleUpdate() {
    return this.performUpdate();
  }
  performUpdate() {
    if (!this.isUpdatePending) return;
    if (!this.hasUpdated) {
      if (this.renderRoot ?? (this.renderRoot = this.createRenderRoot()), this._$Ep) {
        for (const [t5, s5] of this._$Ep) this[t5] = s5;
        this._$Ep = void 0;
      }
      const t4 = this.constructor.elementProperties;
      if (t4.size > 0) for (const [s5, i5] of t4) {
        const { wrapped: t5 } = i5, e4 = this[s5];
        true !== t5 || this._$AL.has(s5) || void 0 === e4 || this.C(s5, void 0, i5, e4);
      }
    }
    let t3 = false;
    const s4 = this._$AL;
    try {
      t3 = this.shouldUpdate(s4), t3 ? (this.willUpdate(s4), this._$EO?.forEach((t4) => t4.hostUpdate?.()), this.update(s4)) : this._$EM();
    } catch (s5) {
      throw t3 = false, this._$EM(), s5;
    }
    t3 && this._$AE(s4);
  }
  willUpdate(t3) {
  }
  _$AE(t3) {
    this._$EO?.forEach((t4) => t4.hostUpdated?.()), this.hasUpdated || (this.hasUpdated = true, this.firstUpdated(t3)), this.updated(t3);
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
  shouldUpdate(t3) {
    return true;
  }
  update(t3) {
    this._$Eq && (this._$Eq = this._$Eq.forEach((t4) => this._$ET(t4, this[t4]))), this._$EM();
  }
  updated(t3) {
  }
  firstUpdated(t3) {
  }
};
y.elementStyles = [], y.shadowRootOptions = { mode: "open" }, y[d("elementProperties")] = /* @__PURE__ */ new Map(), y[d("finalized")] = /* @__PURE__ */ new Map(), p?.({ ReactiveElement: y }), (a.reactiveElementVersions ?? (a.reactiveElementVersions = [])).push("2.1.2");

// ../../node_modules/.pnpm/lit-html@3.3.2/node_modules/lit-html/lit-html.js
var t2 = globalThis;
var i3 = (t3) => t3;
var s2 = t2.trustedTypes;
var e3 = s2 ? s2.createPolicy("lit-html", { createHTML: (t3) => t3 }) : void 0;
var h2 = "$lit$";
var o3 = `lit$${Math.random().toFixed(9).slice(2)}$`;
var n3 = "?" + o3;
var r3 = `<${n3}>`;
var l2 = document;
var c3 = () => l2.createComment("");
var a2 = (t3) => null === t3 || "object" != typeof t3 && "function" != typeof t3;
var u2 = Array.isArray;
var d2 = (t3) => u2(t3) || "function" == typeof t3?.[Symbol.iterator];
var f2 = "[ 	\n\f\r]";
var v = /<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g;
var _ = /-->/g;
var m = />/g;
var p2 = RegExp(`>|${f2}(?:([^\\s"'>=/]+)(${f2}*=${f2}*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`, "g");
var g = /'/g;
var $ = /"/g;
var y2 = /^(?:script|style|textarea|title)$/i;
var x = (t3) => (i5, ...s4) => ({ _$litType$: t3, strings: i5, values: s4 });
var b2 = x(1);
var w = x(2);
var T = x(3);
var E = /* @__PURE__ */ Symbol.for("lit-noChange");
var A = /* @__PURE__ */ Symbol.for("lit-nothing");
var C = /* @__PURE__ */ new WeakMap();
var P = l2.createTreeWalker(l2, 129);
function V(t3, i5) {
  if (!u2(t3) || !t3.hasOwnProperty("raw")) throw Error("invalid template strings array");
  return void 0 !== e3 ? e3.createHTML(i5) : i5;
}
var N = (t3, i5) => {
  const s4 = t3.length - 1, e4 = [];
  let n4, l3 = 2 === i5 ? "<svg>" : 3 === i5 ? "<math>" : "", c4 = v;
  for (let i6 = 0; i6 < s4; i6++) {
    const s5 = t3[i6];
    let a3, u3, d3 = -1, f3 = 0;
    for (; f3 < s5.length && (c4.lastIndex = f3, u3 = c4.exec(s5), null !== u3); ) f3 = c4.lastIndex, c4 === v ? "!--" === u3[1] ? c4 = _ : void 0 !== u3[1] ? c4 = m : void 0 !== u3[2] ? (y2.test(u3[2]) && (n4 = RegExp("</" + u3[2], "g")), c4 = p2) : void 0 !== u3[3] && (c4 = p2) : c4 === p2 ? ">" === u3[0] ? (c4 = n4 ?? v, d3 = -1) : void 0 === u3[1] ? d3 = -2 : (d3 = c4.lastIndex - u3[2].length, a3 = u3[1], c4 = void 0 === u3[3] ? p2 : '"' === u3[3] ? $ : g) : c4 === $ || c4 === g ? c4 = p2 : c4 === _ || c4 === m ? c4 = v : (c4 = p2, n4 = void 0);
    const x2 = c4 === p2 && t3[i6 + 1].startsWith("/>") ? " " : "";
    l3 += c4 === v ? s5 + r3 : d3 >= 0 ? (e4.push(a3), s5.slice(0, d3) + h2 + s5.slice(d3) + o3 + x2) : s5 + o3 + (-2 === d3 ? i6 : x2);
  }
  return [V(t3, l3 + (t3[s4] || "<?>") + (2 === i5 ? "</svg>" : 3 === i5 ? "</math>" : "")), e4];
};
var S2 = class _S {
  constructor({ strings: t3, _$litType$: i5 }, e4) {
    let r4;
    this.parts = [];
    let l3 = 0, a3 = 0;
    const u3 = t3.length - 1, d3 = this.parts, [f3, v2] = N(t3, i5);
    if (this.el = _S.createElement(f3, e4), P.currentNode = this.el.content, 2 === i5 || 3 === i5) {
      const t4 = this.el.content.firstChild;
      t4.replaceWith(...t4.childNodes);
    }
    for (; null !== (r4 = P.nextNode()) && d3.length < u3; ) {
      if (1 === r4.nodeType) {
        if (r4.hasAttributes()) for (const t4 of r4.getAttributeNames()) if (t4.endsWith(h2)) {
          const i6 = v2[a3++], s4 = r4.getAttribute(t4).split(o3), e5 = /([.?@])?(.*)/.exec(i6);
          d3.push({ type: 1, index: l3, name: e5[2], strings: s4, ctor: "." === e5[1] ? I : "?" === e5[1] ? L : "@" === e5[1] ? z : H }), r4.removeAttribute(t4);
        } else t4.startsWith(o3) && (d3.push({ type: 6, index: l3 }), r4.removeAttribute(t4));
        if (y2.test(r4.tagName)) {
          const t4 = r4.textContent.split(o3), i6 = t4.length - 1;
          if (i6 > 0) {
            r4.textContent = s2 ? s2.emptyScript : "";
            for (let s4 = 0; s4 < i6; s4++) r4.append(t4[s4], c3()), P.nextNode(), d3.push({ type: 2, index: ++l3 });
            r4.append(t4[i6], c3());
          }
        }
      } else if (8 === r4.nodeType) if (r4.data === n3) d3.push({ type: 2, index: l3 });
      else {
        let t4 = -1;
        for (; -1 !== (t4 = r4.data.indexOf(o3, t4 + 1)); ) d3.push({ type: 7, index: l3 }), t4 += o3.length - 1;
      }
      l3++;
    }
  }
  static createElement(t3, i5) {
    const s4 = l2.createElement("template");
    return s4.innerHTML = t3, s4;
  }
};
function M(t3, i5, s4 = t3, e4) {
  if (i5 === E) return i5;
  let h3 = void 0 !== e4 ? s4._$Co?.[e4] : s4._$Cl;
  const o5 = a2(i5) ? void 0 : i5._$litDirective$;
  return h3?.constructor !== o5 && (h3?._$AO?.(false), void 0 === o5 ? h3 = void 0 : (h3 = new o5(t3), h3._$AT(t3, s4, e4)), void 0 !== e4 ? (s4._$Co ?? (s4._$Co = []))[e4] = h3 : s4._$Cl = h3), void 0 !== h3 && (i5 = M(t3, h3._$AS(t3, i5.values), h3, e4)), i5;
}
var R = class {
  constructor(t3, i5) {
    this._$AV = [], this._$AN = void 0, this._$AD = t3, this._$AM = i5;
  }
  get parentNode() {
    return this._$AM.parentNode;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  u(t3) {
    const { el: { content: i5 }, parts: s4 } = this._$AD, e4 = (t3?.creationScope ?? l2).importNode(i5, true);
    P.currentNode = e4;
    let h3 = P.nextNode(), o5 = 0, n4 = 0, r4 = s4[0];
    for (; void 0 !== r4; ) {
      if (o5 === r4.index) {
        let i6;
        2 === r4.type ? i6 = new k(h3, h3.nextSibling, this, t3) : 1 === r4.type ? i6 = new r4.ctor(h3, r4.name, r4.strings, this, t3) : 6 === r4.type && (i6 = new Z(h3, this, t3)), this._$AV.push(i6), r4 = s4[++n4];
      }
      o5 !== r4?.index && (h3 = P.nextNode(), o5++);
    }
    return P.currentNode = l2, e4;
  }
  p(t3) {
    let i5 = 0;
    for (const s4 of this._$AV) void 0 !== s4 && (void 0 !== s4.strings ? (s4._$AI(t3, s4, i5), i5 += s4.strings.length - 2) : s4._$AI(t3[i5])), i5++;
  }
};
var k = class _k {
  get _$AU() {
    return this._$AM?._$AU ?? this._$Cv;
  }
  constructor(t3, i5, s4, e4) {
    this.type = 2, this._$AH = A, this._$AN = void 0, this._$AA = t3, this._$AB = i5, this._$AM = s4, this.options = e4, this._$Cv = e4?.isConnected ?? true;
  }
  get parentNode() {
    let t3 = this._$AA.parentNode;
    const i5 = this._$AM;
    return void 0 !== i5 && 11 === t3?.nodeType && (t3 = i5.parentNode), t3;
  }
  get startNode() {
    return this._$AA;
  }
  get endNode() {
    return this._$AB;
  }
  _$AI(t3, i5 = this) {
    t3 = M(this, t3, i5), a2(t3) ? t3 === A || null == t3 || "" === t3 ? (this._$AH !== A && this._$AR(), this._$AH = A) : t3 !== this._$AH && t3 !== E && this._(t3) : void 0 !== t3._$litType$ ? this.$(t3) : void 0 !== t3.nodeType ? this.T(t3) : d2(t3) ? this.k(t3) : this._(t3);
  }
  O(t3) {
    return this._$AA.parentNode.insertBefore(t3, this._$AB);
  }
  T(t3) {
    this._$AH !== t3 && (this._$AR(), this._$AH = this.O(t3));
  }
  _(t3) {
    this._$AH !== A && a2(this._$AH) ? this._$AA.nextSibling.data = t3 : this.T(l2.createTextNode(t3)), this._$AH = t3;
  }
  $(t3) {
    const { values: i5, _$litType$: s4 } = t3, e4 = "number" == typeof s4 ? this._$AC(t3) : (void 0 === s4.el && (s4.el = S2.createElement(V(s4.h, s4.h[0]), this.options)), s4);
    if (this._$AH?._$AD === e4) this._$AH.p(i5);
    else {
      const t4 = new R(e4, this), s5 = t4.u(this.options);
      t4.p(i5), this.T(s5), this._$AH = t4;
    }
  }
  _$AC(t3) {
    let i5 = C.get(t3.strings);
    return void 0 === i5 && C.set(t3.strings, i5 = new S2(t3)), i5;
  }
  k(t3) {
    u2(this._$AH) || (this._$AH = [], this._$AR());
    const i5 = this._$AH;
    let s4, e4 = 0;
    for (const h3 of t3) e4 === i5.length ? i5.push(s4 = new _k(this.O(c3()), this.O(c3()), this, this.options)) : s4 = i5[e4], s4._$AI(h3), e4++;
    e4 < i5.length && (this._$AR(s4 && s4._$AB.nextSibling, e4), i5.length = e4);
  }
  _$AR(t3 = this._$AA.nextSibling, s4) {
    for (this._$AP?.(false, true, s4); t3 !== this._$AB; ) {
      const s5 = i3(t3).nextSibling;
      i3(t3).remove(), t3 = s5;
    }
  }
  setConnected(t3) {
    void 0 === this._$AM && (this._$Cv = t3, this._$AP?.(t3));
  }
};
var H = class {
  get tagName() {
    return this.element.tagName;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  constructor(t3, i5, s4, e4, h3) {
    this.type = 1, this._$AH = A, this._$AN = void 0, this.element = t3, this.name = i5, this._$AM = e4, this.options = h3, s4.length > 2 || "" !== s4[0] || "" !== s4[1] ? (this._$AH = Array(s4.length - 1).fill(new String()), this.strings = s4) : this._$AH = A;
  }
  _$AI(t3, i5 = this, s4, e4) {
    const h3 = this.strings;
    let o5 = false;
    if (void 0 === h3) t3 = M(this, t3, i5, 0), o5 = !a2(t3) || t3 !== this._$AH && t3 !== E, o5 && (this._$AH = t3);
    else {
      const e5 = t3;
      let n4, r4;
      for (t3 = h3[0], n4 = 0; n4 < h3.length - 1; n4++) r4 = M(this, e5[s4 + n4], i5, n4), r4 === E && (r4 = this._$AH[n4]), o5 || (o5 = !a2(r4) || r4 !== this._$AH[n4]), r4 === A ? t3 = A : t3 !== A && (t3 += (r4 ?? "") + h3[n4 + 1]), this._$AH[n4] = r4;
    }
    o5 && !e4 && this.j(t3);
  }
  j(t3) {
    t3 === A ? this.element.removeAttribute(this.name) : this.element.setAttribute(this.name, t3 ?? "");
  }
};
var I = class extends H {
  constructor() {
    super(...arguments), this.type = 3;
  }
  j(t3) {
    this.element[this.name] = t3 === A ? void 0 : t3;
  }
};
var L = class extends H {
  constructor() {
    super(...arguments), this.type = 4;
  }
  j(t3) {
    this.element.toggleAttribute(this.name, !!t3 && t3 !== A);
  }
};
var z = class extends H {
  constructor(t3, i5, s4, e4, h3) {
    super(t3, i5, s4, e4, h3), this.type = 5;
  }
  _$AI(t3, i5 = this) {
    if ((t3 = M(this, t3, i5, 0) ?? A) === E) return;
    const s4 = this._$AH, e4 = t3 === A && s4 !== A || t3.capture !== s4.capture || t3.once !== s4.once || t3.passive !== s4.passive, h3 = t3 !== A && (s4 === A || e4);
    e4 && this.element.removeEventListener(this.name, this, s4), h3 && this.element.addEventListener(this.name, this, t3), this._$AH = t3;
  }
  handleEvent(t3) {
    "function" == typeof this._$AH ? this._$AH.call(this.options?.host ?? this.element, t3) : this._$AH.handleEvent(t3);
  }
};
var Z = class {
  constructor(t3, i5, s4) {
    this.element = t3, this.type = 6, this._$AN = void 0, this._$AM = i5, this.options = s4;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  _$AI(t3) {
    M(this, t3);
  }
};
var B = t2.litHtmlPolyfillSupport;
B?.(S2, k), (t2.litHtmlVersions ?? (t2.litHtmlVersions = [])).push("3.3.2");
var D = (t3, i5, s4) => {
  const e4 = s4?.renderBefore ?? i5;
  let h3 = e4._$litPart$;
  if (void 0 === h3) {
    const t4 = s4?.renderBefore ?? null;
    e4._$litPart$ = h3 = new k(i5.insertBefore(c3(), t4), t4, void 0, s4 ?? {});
  }
  return h3._$AI(t3), h3;
};

// ../../node_modules/.pnpm/lit-element@4.2.2/node_modules/lit-element/lit-element.js
var s3 = globalThis;
var i4 = class extends y {
  constructor() {
    super(...arguments), this.renderOptions = { host: this }, this._$Do = void 0;
  }
  createRenderRoot() {
    var _a;
    const t3 = super.createRenderRoot();
    return (_a = this.renderOptions).renderBefore ?? (_a.renderBefore = t3.firstChild), t3;
  }
  update(t3) {
    const r4 = this.render();
    this.hasUpdated || (this.renderOptions.isConnected = this.isConnected), super.update(t3), this._$Do = D(r4, this.renderRoot, this.renderOptions);
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
    this.onBackdropClick = (e4) => {
      if (!this.dismissible) return;
      if (e4.target === this.dialogEl) this.close();
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
  onChange(e4) {
    const c4 = e4.target.checked;
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
    this.onItemClick = (e4) => {
      const target = e4.target.closest("[data-value]");
      if (!target) return;
      const value = target.dataset.value || "";
      this.dispatchEvent(new CustomEvent("ship-select", {
        detail: { value },
        bubbles: true,
        composed: true
      }));
      this.open = false;
    };
    this.onDocClick = (e4) => {
      if (!this.open) return;
      const target = e4.composedPath()[0];
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
  onInput(e4) {
    const target = e4.target;
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
  onClick(e4) {
    if (this.disabled) return;
    this.dispatchEvent(new CustomEvent("ship-select", {
      detail: {},
      bubbles: true,
      composed: true
    }));
    void e4;
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
    this.onDocClick = (e4) => {
      if (!this.open) return;
      const target = e4.composedPath()[0];
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
    for (const o5 of opts) fragment.appendChild(o5.cloneNode(true));
    this.selectEl.innerHTML = "";
    this.selectEl.appendChild(fragment);
    this.selectEl.value = this.value;
  }
  onChange(e4) {
    this.value = e4.target.value;
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
  onChange(e4) {
    const c4 = e4.target.checked;
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
  onKeydown(e4) {
    if (e4.key !== "ArrowRight" && e4.key !== "ArrowLeft") return;
    e4.preventDefault();
    const tabs = this.tabs.filter((t3) => !t3.disabled);
    const names = tabs.map((t3) => t3.name);
    const i5 = names.indexOf(this.active);
    const next = e4.key === "ArrowRight" ? (i5 + 1) % names.length : (i5 - 1 + names.length) % names.length;
    this.activate(names[next]);
  }
  render() {
    const tabs = this.tabs.map((t3) => ({
      name: t3.name,
      label: t3.label || t3.name,
      count: t3.count,
      disabled: t3.disabled
    }));
    return b2`
      <div class="tablist ${this.variant}" role="tablist" @keydown=${this.onKeydown}>
        ${tabs.map((t3) => b2`
          <button
            role="tab"
            aria-selected=${t3.name === this.active ? "true" : "false"}
            tabindex=${t3.name === this.active ? "0" : "-1"}
            ?disabled=${t3.disabled}
            @click=${() => this.activate(t3.name)}
          >
            ${t3.label}
            ${t3.count != null ? b2`<span class="count">${t3.count}</span>` : null}
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
  onInput(e4) {
    this.value = e4.target.value;
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

// src/web-components/index.ts
function registerShipComponents() {
}
export {
  ShipAlert,
  ShipAlertDialog,
  ShipBadge,
  ShipButton,
  ShipCard,
  ShipCheckbox,
  ShipDialog,
  ShipDropdownMenu,
  ShipEmptyState,
  ShipField,
  ShipHistoryRow,
  ShipInput,
  ShipLabel,
  ShipListItem,
  ShipPipelineDag,
  ShipPipelineDefinition,
  ShipPipelineRun,
  ShipPipelineStep,
  ShipPopover,
  ShipProgress,
  ShipSelect,
  ShipSeparator,
  ShipSpecRow,
  ShipSpinner,
  ShipStat,
  ShipStatusDot,
  ShipSwitch,
  ShipTab,
  ShipTabPanel,
  ShipTabs,
  ShipTextarea,
  ShipThemeToggle,
  ShipToolbar,
  ShipTooltip,
  registerShipComponents
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
