var __defProp = Object.defineProperty;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __esm = (fn3, res) => function __init() {
  return fn3 && (res = (0, fn3[__getOwnPropNames(fn3)[0]])(fn3 = 0)), res;
};
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};

// node_modules/.pnpm/@xterm+xterm@6.0.0/node_modules/@xterm/xterm/lib/xterm.mjs
var xterm_exports = {};
__export(xterm_exports, {
  Terminal: () => Dl
});
function Al(s18) {
  return s18.replace(/\r?\n/g, "\r");
}
function kl(s18, t3) {
  return t3 ? "\x1B[200~" + s18 + "\x1B[201~" : s18;
}
function Vs(s18, t3) {
  s18.clipboardData && s18.clipboardData.setData("text/plain", t3.selectionText), s18.preventDefault();
}
function qs(s18, t3, e4, i12) {
  if (s18.stopPropagation(), s18.clipboardData) {
    let r7 = s18.clipboardData.getData("text/plain");
    Cn(r7, t3, e4, i12);
  }
}
function Cn(s18, t3, e4, i12) {
  s18 = Al(s18), s18 = kl(s18, e4.decPrivateModes.bracketedPasteMode && i12.rawOptions.ignoreBracketedPasteMode !== true), e4.triggerDataEvent(s18, true), t3.value = "";
}
function Mn(s18, t3, e4) {
  let i12 = e4.getBoundingClientRect(), r7 = s18.clientX - i12.left - 10, n4 = s18.clientY - i12.top - 10;
  t3.style.width = "20px", t3.style.height = "20px", t3.style.left = `${r7}px`, t3.style.top = `${n4}px`, t3.style.zIndex = "1000", t3.focus();
}
function Pn(s18, t3, e4, i12, r7) {
  Mn(s18, t3, e4), r7 && i12.rightClickSelect(s18), t3.value = i12.selectionText, t3.select();
}
function Ce(s18) {
  return s18 > 65535 ? (s18 -= 65536, String.fromCharCode((s18 >> 10) + 55296) + String.fromCharCode(s18 % 1024 + 56320)) : String.fromCharCode(s18);
}
function It(s18, t3 = 0, e4 = s18.length) {
  let i12 = "";
  for (let r7 = t3; r7 < e4; ++r7) {
    let n4 = s18[r7];
    n4 > 65535 ? (n4 -= 65536, i12 += String.fromCharCode((n4 >> 10) + 55296) + String.fromCharCode(n4 % 1024 + 56320)) : i12 += String.fromCharCode(n4);
  }
  return i12;
}
function Xs(s18) {
  return s18[Hn] || [];
}
function ie(s18) {
  if (Fn.has(s18)) return Fn.get(s18);
  let t3 = function(e4, i12, r7) {
    if (arguments.length !== 3) throw new Error("@IServiceName-decorator can only be used to decorate a parameter");
    Pl(t3, e4, r7);
  };
  return t3._id = s18, Fn.set(s18, t3), t3;
}
function Pl(s18, t3, e4) {
  t3[js] === t3 ? t3[Hn].push({ id: s18, index: e4 }) : (t3[Hn] = [{ id: s18, index: e4 }], t3[js] = t3);
}
function Ol(s18, t3) {
  if (confirm(`Do you want to navigate to ${t3}?

WARNING: This link could potentially be dangerous`)) {
    let i12 = window.open();
    if (i12) {
      try {
        i12.opener = null;
      } catch {
      }
      i12.location.href = t3;
    } else console.warn("Opening link blocked as opener could not be cleared");
  }
}
function Lt(s18) {
  Nl(s18) || Bl.onUnexpectedError(s18);
}
function Nl(s18) {
  return s18 instanceof bi ? true : s18 instanceof Error && s18.name === Un && s18.message === Un;
}
function eo(s18) {
  return s18 ? new Error(`Illegal argument: ${s18}`) : new Error("Illegal argument");
}
function Fl(s18, t3, e4 = 0, i12 = s18.length) {
  let r7 = e4, n4 = i12;
  for (; r7 < n4; ) {
    let o6 = Math.floor((r7 + n4) / 2);
    t3(s18[o6]) ? r7 = o6 + 1 : n4 = o6;
  }
  return r7 - 1;
}
function Se(s18, t3 = 0) {
  return s18[s18.length - (1 + t3)];
}
function no(s18, t3) {
  return (e4, i12) => t3(s18(e4), s18(i12));
}
function co(s18, t3) {
  let e4 = /* @__PURE__ */ Object.create(null);
  for (let i12 of s18) {
    let r7 = t3(i12), n4 = e4[r7];
    n4 || (n4 = e4[r7] = []), n4.push(i12);
  }
  return e4;
}
function Kn(s18, t3) {
  let e4 = this, i12 = false, r7;
  return function() {
    if (i12) return r7;
    if (i12 = true, t3) try {
      r7 = s18.apply(e4, arguments);
    } finally {
      t3();
    }
    else r7 = s18.apply(e4, arguments);
    return r7;
  };
}
function Ul(s18) {
  dt = s18;
}
function fr(s18) {
  return dt?.trackDisposable(s18), s18;
}
function pr(s18) {
  dt?.markAsDisposed(s18);
}
function vi(s18, t3) {
  dt?.setParent(s18, t3);
}
function Kl(s18, t3) {
  if (dt) for (let e4 of s18) dt.setParent(e4, t3);
}
function Gn(s18) {
  return dt?.markAsSingleton(s18), s18;
}
function Ne(s18) {
  if (zn.is(s18)) {
    let t3 = [];
    for (let e4 of s18) if (e4) try {
      e4.dispose();
    } catch (i12) {
      t3.push(i12);
    }
    if (t3.length === 1) throw t3[0];
    if (t3.length > 1) throw new AggregateError(t3, "Encountered errors while disposing of store");
    return Array.isArray(s18) ? [] : s18;
  } else if (s18) return s18.dispose(), s18;
}
function ho(...s18) {
  let t3 = C2(() => Ne(s18));
  return Kl(s18, t3), t3;
}
function C2(s18) {
  let t3 = fr({ dispose: Kn(() => {
    pr(t3), s18();
  }) });
  return t3;
}
function Xl(s18, t3, e4) {
  typeof t3 == "string" && (t3 = s18.matchMedia(t3)), t3.addEventListener("change", e4);
}
function mo(s18) {
  return Si.INSTANCE.getZoomFactor(s18);
}
function _o() {
  return vr;
}
function ca(s18) {
  if (s18.charCode) {
    let e4 = String.fromCharCode(s18.charCode).toUpperCase();
    return Qn.fromString(e4);
  }
  let t3 = s18.keyCode;
  if (t3 === 3) return 7;
  if (Ei) switch (t3) {
    case 59:
      return 85;
    case 60:
      if (Zn) return 97;
      break;
    case 61:
      return 86;
    case 107:
      return 109;
    case 109:
      return 111;
    case 173:
      return 88;
    case 224:
      if (Te) return 57;
      break;
  }
  else if (Bt) {
    if (Te && t3 === 93) return 57;
    if (!Te && t3 === 92) return 57;
  }
  return yo[t3] || 0;
}
function pa(s18) {
  if (!s18.parent || s18.parent === s18) return null;
  try {
    let t3 = s18.location, e4 = s18.parent.location;
    if (t3.origin !== "null" && e4.origin !== "null" && t3.origin !== e4.origin) return null;
  } catch {
    return null;
  }
  return s18.parent;
}
function Lo(s18) {
  return 55296 <= s18 && s18 <= 56319;
}
function is(s18) {
  return 56320 <= s18 && s18 <= 57343;
}
function Ao(s18, t3) {
  return (s18 - 55296 << 10) + (t3 - 56320) + 65536;
}
function Mo(s18) {
  return ns(s18, 0);
}
function ns(s18, t3) {
  switch (typeof s18) {
    case "object":
      return s18 === null ? je(349, t3) : Array.isArray(s18) ? Ea(s18, t3) : Ta(s18, t3);
    case "string":
      return Po(s18, t3);
    case "boolean":
      return Sa(s18, t3);
    case "number":
      return je(s18, t3);
    case "undefined":
      return je(937, t3);
    default:
      return je(617, t3);
  }
}
function je(s18, t3) {
  return (t3 << 5) - t3 + s18 | 0;
}
function Sa(s18, t3) {
  return je(s18 ? 433 : 863, t3);
}
function Po(s18, t3) {
  t3 = je(149417, t3);
  for (let e4 = 0, i12 = s18.length; e4 < i12; e4++) t3 = je(s18.charCodeAt(e4), t3);
  return t3;
}
function Ea(s18, t3) {
  return t3 = je(104579, t3), s18.reduce((e4, i12) => ns(i12, e4), t3);
}
function Ta(s18, t3) {
  return t3 = je(181387, t3), Object.keys(s18).sort().reduce((e4, i12) => (e4 = Po(i12, e4), ns(s18[i12], e4)), t3);
}
function rs(s18, t3, e4 = 32) {
  let i12 = e4 - t3, r7 = ~((1 << i12) - 1);
  return (s18 << t3 | (r7 & s18) >>> i12) >>> 0;
}
function ko(s18, t3 = 0, e4 = s18.byteLength, i12 = 0) {
  for (let r7 = 0; r7 < e4; r7++) s18[t3 + r7] = i12;
}
function Ia(s18, t3, e4 = "0") {
  for (; s18.length < t3; ) s18 = e4 + s18;
  return s18;
}
function wi(s18, t3 = 32) {
  return s18 instanceof ArrayBuffer ? Array.from(new Uint8Array(s18)).map((e4) => e4.toString(16).padStart(2, "0")).join("") : Ia((s18 >>> 0).toString(16), t3 / 4);
}
function L2(s18, t3, e4, i12) {
  return new ss(s18, t3, e4, i12);
}
function ya(s18, t3) {
  return function(e4) {
    return t3(new qe(s18, e4));
  };
}
function xa(s18) {
  return function(t3) {
    return s18(new ft(t3));
  };
}
function Fo(s18) {
  let t3 = s18.getBoundingClientRect(), e4 = be(s18);
  return { left: t3.left + e4.scrollX, top: t3.top + e4.scrollY, width: t3.width, height: t3.height };
}
function Ho(s18, t3, e4, ...i12) {
  let r7 = Da.exec(t3);
  if (!r7) throw new Error("Bad use of emmet");
  let n4 = r7[1] || "div", o6;
  return s18 !== "http://www.w3.org/1999/xhtml" ? o6 = document.createElementNS(s18, n4) : o6 = document.createElement(n4), r7[3] && (o6.id = r7[3]), r7[4] && (o6.className = r7[4].replace(/\./g, " ").trim()), e4 && Object.entries(e4).forEach(([l4, a3]) => {
    typeof a3 > "u" || (/^on\w+$/.test(l4) ? o6[l4] = a3 : l4 === "selected" ? a3 && o6.setAttribute(l4, "true") : o6.setAttribute(l4, a3));
  }), o6.append(...i12), o6;
}
function Ra(s18, t3, ...e4) {
  return Ho("http://www.w3.org/1999/xhtml", s18, t3, ...e4);
}
function Ie(s18) {
  return typeof s18 == "number" ? `${s18}px` : s18;
}
function _t(s18) {
  return new ls(s18);
}
function Wo(s18, t3, e4) {
  let i12 = null, r7 = null;
  if (typeof e4.value == "function" ? (i12 = "value", r7 = e4.value, r7.length !== 0 && console.warn("Memoize should only be used in functions with zero parameters")) : typeof e4.get == "function" && (i12 = "get", r7 = e4.get), !r7) throw new Error("not supported");
  let n4 = `$memoize$${t3}`;
  e4[i12] = function(...o6) {
    return this.hasOwnProperty(n4) || Object.defineProperty(this, n4, { configurable: false, enumerable: false, writable: false, value: r7.apply(this, o6) }), this[n4];
  };
}
function as(s18, t3) {
  let e4 = t3 - s18;
  return function(i12) {
    return s18 + e4 * ka(i12);
  };
}
function La(s18, t3, e4) {
  return function(i12) {
    return i12 < e4 ? s18(i12 / e4) : t3((i12 - e4) / (1 - e4));
  };
}
function Aa(s18) {
  return Math.pow(s18, 3);
}
function ka(s18) {
  return 1 - Aa(1 - s18);
}
function Pa(s18) {
  let t3 = { lazyRender: typeof s18.lazyRender < "u" ? s18.lazyRender : false, className: typeof s18.className < "u" ? s18.className : "", useShadows: typeof s18.useShadows < "u" ? s18.useShadows : true, handleMouseWheel: typeof s18.handleMouseWheel < "u" ? s18.handleMouseWheel : true, flipAxes: typeof s18.flipAxes < "u" ? s18.flipAxes : false, consumeMouseWheelIfScrollbarIsNeeded: typeof s18.consumeMouseWheelIfScrollbarIsNeeded < "u" ? s18.consumeMouseWheelIfScrollbarIsNeeded : false, alwaysConsumeMouseWheel: typeof s18.alwaysConsumeMouseWheel < "u" ? s18.alwaysConsumeMouseWheel : false, scrollYToX: typeof s18.scrollYToX < "u" ? s18.scrollYToX : false, mouseWheelScrollSensitivity: typeof s18.mouseWheelScrollSensitivity < "u" ? s18.mouseWheelScrollSensitivity : 1, fastScrollSensitivity: typeof s18.fastScrollSensitivity < "u" ? s18.fastScrollSensitivity : 5, scrollPredominantAxis: typeof s18.scrollPredominantAxis < "u" ? s18.scrollPredominantAxis : true, mouseWheelSmoothScroll: typeof s18.mouseWheelSmoothScroll < "u" ? s18.mouseWheelSmoothScroll : true, arrowSize: typeof s18.arrowSize < "u" ? s18.arrowSize : 11, listenOnDomNode: typeof s18.listenOnDomNode < "u" ? s18.listenOnDomNode : null, horizontal: typeof s18.horizontal < "u" ? s18.horizontal : 1, horizontalScrollbarSize: typeof s18.horizontalScrollbarSize < "u" ? s18.horizontalScrollbarSize : 10, horizontalSliderSize: typeof s18.horizontalSliderSize < "u" ? s18.horizontalSliderSize : 0, horizontalHasArrows: typeof s18.horizontalHasArrows < "u" ? s18.horizontalHasArrows : false, vertical: typeof s18.vertical < "u" ? s18.vertical : 1, verticalScrollbarSize: typeof s18.verticalScrollbarSize < "u" ? s18.verticalScrollbarSize : 10, verticalHasArrows: typeof s18.verticalHasArrows < "u" ? s18.verticalHasArrows : false, verticalSliderSize: typeof s18.verticalSliderSize < "u" ? s18.verticalSliderSize : 0, scrollByPage: typeof s18.scrollByPage < "u" ? s18.scrollByPage : false };
  return t3.horizontalSliderSize = typeof s18.horizontalSliderSize < "u" ? s18.horizontalSliderSize : t3.horizontalScrollbarSize, t3.verticalSliderSize = typeof s18.verticalSliderSize < "u" ? s18.verticalSliderSize : t3.verticalScrollbarSize, Te && (t3.className += " mac"), t3;
}
function vt(s18) {
  let t3 = s18.toString(16);
  return t3.length < 2 ? "0" + t3 : t3;
}
function Xe(s18, t3) {
  return s18 < t3 ? (t3 + 0.05) / (s18 + 0.05) : (s18 + 0.05) / (t3 + 0.05);
}
function Oa(s18) {
  return 57508 <= s18 && s18 <= 57558;
}
function Ba(s18) {
  return 9472 <= s18 && s18 <= 9631;
}
function $o(s18) {
  return Oa(s18) || Ba(s18);
}
function Vo() {
  return { css: { canvas: qr(), cell: qr() }, device: { canvas: qr(), cell: qr(), char: { width: 0, height: 0, left: 0, top: 0 } } };
}
function qr() {
  return { width: 0, height: 0 };
}
function qo(s18, t3, e4) {
  for (; s18.length < e4; ) s18 = t3 + s18;
  return s18;
}
function Yo() {
  return new ms();
}
function Ci(s18, t3, e4) {
  let i12 = e4.getBoundingClientRect(), r7 = s18.getComputedStyle(e4), n4 = parseInt(r7.getPropertyValue("padding-left")), o6 = parseInt(r7.getPropertyValue("padding-top"));
  return [t3.clientX - i12.left - n4, t3.clientY - i12.top - o6];
}
function Xo(s18, t3, e4, i12, r7, n4, o6, l4, a3) {
  if (!n4) return;
  let u3 = Ci(s18, t3, e4);
  if (u3) return u3[0] = Math.ceil((u3[0] + (a3 ? o6 / 2 : 0)) / o6), u3[1] = Math.ceil(u3[1] / l4), u3[0] = Math.min(Math.max(u3[0], 1), i12 + (a3 ? 1 : 0)), u3[1] = Math.min(Math.max(u3[1], 1), r7), u3;
}
function Ha() {
  if (!Zo) return 0;
  let s18 = Pi.match(/Version\/(\d+)/);
  return s18 === null || s18.length < 2 ? 0 : parseInt(s18[1]);
}
function Jo(s18, t3, e4, i12) {
  let r7 = e4.buffer.x, n4 = e4.buffer.y;
  if (!e4.buffer.hasScrollback) return Ga(r7, n4, s18, t3, e4, i12) + sn(n4, t3, e4, i12) + $a(r7, n4, s18, t3, e4, i12);
  let o6;
  if (n4 === t3) return o6 = r7 > s18 ? "D" : "C", Fi(Math.abs(r7 - s18), Ni(o6, i12));
  o6 = n4 > t3 ? "D" : "C";
  let l4 = Math.abs(n4 - t3), a3 = za(n4 > t3 ? s18 : r7, e4) + (l4 - 1) * e4.cols + 1 + Ka(n4 > t3 ? r7 : s18, e4);
  return Fi(a3, Ni(o6, i12));
}
function Ka(s18, t3) {
  return s18 - 1;
}
function za(s18, t3) {
  return t3.cols - s18;
}
function Ga(s18, t3, e4, i12, r7, n4) {
  return sn(t3, i12, r7, n4).length === 0 ? "" : Fi(el(s18, t3, s18, t3 - gt(t3, r7), false, r7).length, Ni("D", n4));
}
function sn(s18, t3, e4, i12) {
  let r7 = s18 - gt(s18, e4), n4 = t3 - gt(t3, e4), o6 = Math.abs(r7 - n4) - Va(s18, t3, e4);
  return Fi(o6, Ni(Qo(s18, t3), i12));
}
function $a(s18, t3, e4, i12, r7, n4) {
  let o6;
  sn(t3, i12, r7, n4).length > 0 ? o6 = i12 - gt(i12, r7) : o6 = t3;
  let l4 = i12, a3 = qa(s18, t3, e4, i12, r7, n4);
  return Fi(el(s18, o6, e4, l4, a3 === "C", r7).length, Ni(a3, n4));
}
function Va(s18, t3, e4) {
  let i12 = 0, r7 = s18 - gt(s18, e4), n4 = t3 - gt(t3, e4);
  for (let o6 = 0; o6 < Math.abs(r7 - n4); o6++) {
    let l4 = Qo(s18, t3) === "A" ? -1 : 1;
    e4.buffer.lines.get(r7 + l4 * o6)?.isWrapped && i12++;
  }
  return i12;
}
function gt(s18, t3) {
  let e4 = 0, i12 = t3.buffer.lines.get(s18), r7 = i12?.isWrapped;
  for (; r7 && s18 >= 0 && s18 < t3.rows; ) e4++, i12 = t3.buffer.lines.get(--s18), r7 = i12?.isWrapped;
  return e4;
}
function qa(s18, t3, e4, i12, r7, n4) {
  let o6;
  return sn(e4, i12, r7, n4).length > 0 ? o6 = i12 - gt(i12, r7) : o6 = t3, s18 < e4 && o6 <= i12 || s18 >= e4 && o6 < i12 ? "C" : "D";
}
function Qo(s18, t3) {
  return s18 > t3 ? "A" : "B";
}
function el(s18, t3, e4, i12, r7, n4) {
  let o6 = s18, l4 = t3, a3 = "";
  for (; (o6 !== e4 || l4 !== i12) && l4 >= 0 && l4 < n4.buffer.lines.length; ) o6 += r7 ? 1 : -1, r7 && o6 > n4.cols - 1 ? (a3 += n4.buffer.translateBufferLineToString(l4, false, s18, o6), o6 = 0, s18 = 0, l4++) : !r7 && o6 < 0 && (a3 += n4.buffer.translateBufferLineToString(l4, false, 0, s18 + 1), o6 = n4.cols - 1, s18 = o6, l4--);
  return a3 + n4.buffer.translateBufferLineToString(l4, false, s18, o6);
}
function Ni(s18, t3) {
  let e4 = t3 ? "O" : "[";
  return b3.ESC + e4 + s18;
}
function Fi(s18, t3) {
  s18 = Math.floor(s18);
  let e4 = "";
  for (let i12 = 0; i12 < s18; i12++) e4 += t3;
  return e4;
}
function ws(s18, t3) {
  if (s18.start.y > s18.end.y) throw new Error(`Buffer range end (${s18.end.x}, ${s18.end.y}) cannot be before start (${s18.start.x}, ${s18.start.y})`);
  return t3 * (s18.end.y - s18.start.y) + (s18.end.x - s18.start.x + 1);
}
function K(s18, t3) {
  if (s18 !== void 0) try {
    return z2.toColor(s18);
  } catch {
  }
  return t3;
}
function sl(s18, t3, e4, i12, r7, n4) {
  let o6 = [];
  for (let l4 = 0; l4 < s18.length - 1; l4++) {
    let a3 = l4, u3 = s18.get(++a3);
    if (!u3.isWrapped) continue;
    let h4 = [s18.get(l4)];
    for (; a3 < s18.length && u3.isWrapped; ) h4.push(u3), u3 = s18.get(++a3);
    if (!n4 && i12 >= l4 && i12 < a3) {
      l4 += h4.length - 1;
      continue;
    }
    let c4 = 0, d3 = ri(h4, c4, t3), _4 = 1, p3 = 0;
    for (; _4 < h4.length; ) {
      let f3 = ri(h4, _4, t3), A3 = f3 - p3, R2 = e4 - d3, O3 = Math.min(A3, R2);
      h4[c4].copyCellsFrom(h4[_4], p3, d3, O3, false), d3 += O3, d3 === e4 && (c4++, d3 = 0), p3 += O3, p3 === f3 && (_4++, p3 = 0), d3 === 0 && c4 !== 0 && h4[c4 - 1].getWidth(e4 - 1) === 2 && (h4[c4].copyCellsFrom(h4[c4 - 1], e4 - 1, d3++, 1, false), h4[c4 - 1].setCell(e4 - 1, r7));
    }
    h4[c4].replaceCells(d3, e4, r7);
    let m2 = 0;
    for (let f3 = h4.length - 1; f3 > 0 && (f3 > c4 || h4[f3].getTrimmedLength() === 0); f3--) m2++;
    m2 > 0 && (o6.push(l4 + h4.length - m2), o6.push(m2)), l4 += h4.length - 1;
  }
  return o6;
}
function ol(s18, t3) {
  let e4 = [], i12 = 0, r7 = t3[i12], n4 = 0;
  for (let o6 = 0; o6 < s18.length; o6++) if (r7 === o6) {
    let l4 = t3[++i12];
    s18.onDeleteEmitter.fire({ index: o6 - n4, amount: l4 }), o6 += l4 - 1, n4 += l4, r7 = t3[++i12];
  } else e4.push(o6);
  return { layout: e4, countRemoved: n4 };
}
function ll(s18, t3) {
  let e4 = [];
  for (let i12 = 0; i12 < t3.length; i12++) e4.push(s18.get(t3[i12]));
  for (let i12 = 0; i12 < e4.length; i12++) s18.set(i12, e4[i12]);
  s18.length = t3.length;
}
function al(s18, t3, e4) {
  let i12 = [], r7 = s18.map((a3, u3) => ri(s18, u3, t3)).reduce((a3, u3) => a3 + u3), n4 = 0, o6 = 0, l4 = 0;
  for (; l4 < r7; ) {
    if (r7 - l4 < e4) {
      i12.push(r7 - l4);
      break;
    }
    n4 += e4;
    let a3 = ri(s18, o6, t3);
    n4 > a3 && (n4 -= a3, o6++);
    let u3 = s18[o6].getWidth(n4 - 1) === 2;
    u3 && n4--;
    let h4 = u3 ? e4 - 1 : e4;
    i12.push(h4), l4 += h4;
  }
  return i12;
}
function ri(s18, t3, e4) {
  if (t3 === s18.length - 1) return s18[t3].getTrimmedLength();
  let i12 = !s18[t3].hasContent(e4 - 1) && s18[t3].getWidth(e4 - 1) === 1, r7 = s18[t3 + 1].getWidth(0) === 2;
  return i12 && r7 ? e4 - 1 : e4;
}
function sc(s18) {
  return s18 === "block" || s18 === "underline" || s18 === "bar";
}
function oi(s18, t3 = 5) {
  if (typeof s18 != "object") return s18;
  let e4 = Array.isArray(s18) ? [] : {};
  for (let i12 in s18) e4[i12] = t3 <= 1 ? s18[i12] : s18[i12] && oi(s18[i12], t3 - 1);
  return e4;
}
function Ms(s18, t3) {
  let e4 = (s18.ctrl ? 16 : 0) | (s18.shift ? 4 : 0) | (s18.alt ? 8 : 0);
  return s18.button === 4 ? (e4 |= 64, e4 |= s18.action) : (e4 |= s18.button & 3, s18.button & 4 && (e4 |= 64), s18.button & 8 && (e4 |= 128), s18.action === 32 ? e4 |= 32 : s18.action === 0 && !t3 && (e4 |= 3)), e4;
}
function cc(s18, t3) {
  let e4 = 0, i12 = t3.length - 1, r7;
  if (s18 < t3[0][0] || s18 > t3[i12][1]) return false;
  for (; i12 >= e4; ) if (r7 = e4 + i12 >> 1, s18 > t3[r7][1]) e4 = r7 + 1;
  else if (s18 < t3[r7][0]) i12 = r7 - 1;
  else return true;
  return false;
}
function Bs(s18) {
  let e4 = s18.buffer.lines.get(s18.buffer.ybase + s18.buffer.y - 1)?.get(s18.cols - 1), i12 = s18.buffer.lines.get(s18.buffer.ybase + s18.buffer.y);
  i12 && e4 && (i12.isWrapped = e4[3] !== 0 && e4[3] !== 32);
}
function Ws(s18) {
  if (!s18) return;
  let t3 = s18.toLowerCase();
  if (t3.indexOf("rgb:") === 0) {
    t3 = t3.slice(4);
    let e4 = dc.exec(t3);
    if (e4) {
      let i12 = e4[1] ? 15 : e4[4] ? 255 : e4[7] ? 4095 : 65535;
      return [Math.round(parseInt(e4[1] || e4[4] || e4[7] || e4[10], 16) / i12 * 255), Math.round(parseInt(e4[2] || e4[5] || e4[8] || e4[11], 16) / i12 * 255), Math.round(parseInt(e4[3] || e4[6] || e4[9] || e4[12], 16) / i12 * 255)];
    }
  } else if (t3.indexOf("#") === 0 && (t3 = t3.slice(1), fc.exec(t3) && [3, 6, 9, 12].includes(t3.length))) {
    let e4 = t3.length / 3, i12 = [0, 0, 0];
    for (let r7 = 0; r7 < 3; ++r7) {
      let n4 = parseInt(t3.slice(e4 * r7, e4 * r7 + e4), 16);
      i12[r7] = e4 === 1 ? n4 << 4 : e4 === 2 ? n4 : e4 === 3 ? n4 >> 4 : n4 >> 8;
    }
    return i12;
  }
}
function Hs(s18, t3) {
  let e4 = s18.toString(16), i12 = e4.length < 2 ? "0" + e4 : e4;
  switch (t3) {
    case 4:
      return e4[0];
    case 8:
      return i12;
    case 12:
      return (i12 + i12).slice(0, 3);
    default:
      return i12 + i12;
  }
}
function ml(s18, t3 = 16) {
  let [e4, i12, r7] = s18;
  return `rgb:${Hs(e4, t3)}/${Hs(i12, t3)}/${Hs(r7, t3)}`;
}
function bl(s18, t3) {
  if (s18 > 24) return t3.setWinLines || false;
  switch (s18) {
    case 1:
      return !!t3.restoreWin;
    case 2:
      return !!t3.minimizeWin;
    case 3:
      return !!t3.setWinPosition;
    case 4:
      return !!t3.setWinSizePixels;
    case 5:
      return !!t3.raiseWin;
    case 6:
      return !!t3.lowerWin;
    case 7:
      return !!t3.refreshWin;
    case 8:
      return !!t3.setWinSizeChars;
    case 9:
      return !!t3.maximizeWin;
    case 10:
      return !!t3.fullscreenWin;
    case 11:
      return !!t3.getWinState;
    case 13:
      return !!t3.getWinPosition;
    case 14:
      return !!t3.getWinSizePixels;
    case 15:
      return !!t3.getScreenSizePixels;
    case 16:
      return !!t3.getCellSizePixels;
    case 18:
      return !!t3.getWinSizeChars;
    case 19:
      return !!t3.getScreenSizeChars;
    case 20:
      return !!t3.getIconTitle;
    case 21:
      return !!t3.getWinTitle;
    case 22:
      return !!t3.pushTitle;
    case 23:
      return !!t3.popTitle;
    case 24:
      return !!t3.setWinLines;
  }
  return false;
}
function Sl(s18) {
  return 0 <= s18 && s18 < 256;
}
function Il(s18, t3, e4, i12) {
  let r7 = { type: 0, cancel: false, key: void 0 }, n4 = (s18.shiftKey ? 1 : 0) | (s18.altKey ? 2 : 0) | (s18.ctrlKey ? 4 : 0) | (s18.metaKey ? 8 : 0);
  switch (s18.keyCode) {
    case 0:
      s18.key === "UIKeyInputUpArrow" ? t3 ? r7.key = b3.ESC + "OA" : r7.key = b3.ESC + "[A" : s18.key === "UIKeyInputLeftArrow" ? t3 ? r7.key = b3.ESC + "OD" : r7.key = b3.ESC + "[D" : s18.key === "UIKeyInputRightArrow" ? t3 ? r7.key = b3.ESC + "OC" : r7.key = b3.ESC + "[C" : s18.key === "UIKeyInputDownArrow" && (t3 ? r7.key = b3.ESC + "OB" : r7.key = b3.ESC + "[B");
      break;
    case 8:
      r7.key = s18.ctrlKey ? "\b" : b3.DEL, s18.altKey && (r7.key = b3.ESC + r7.key);
      break;
    case 9:
      if (s18.shiftKey) {
        r7.key = b3.ESC + "[Z";
        break;
      }
      r7.key = b3.HT, r7.cancel = true;
      break;
    case 13:
      r7.key = s18.altKey ? b3.ESC + b3.CR : b3.CR, r7.cancel = true;
      break;
    case 27:
      r7.key = b3.ESC, s18.altKey && (r7.key = b3.ESC + b3.ESC), r7.cancel = true;
      break;
    case 37:
      if (s18.metaKey) break;
      n4 ? r7.key = b3.ESC + "[1;" + (n4 + 1) + "D" : t3 ? r7.key = b3.ESC + "OD" : r7.key = b3.ESC + "[D";
      break;
    case 39:
      if (s18.metaKey) break;
      n4 ? r7.key = b3.ESC + "[1;" + (n4 + 1) + "C" : t3 ? r7.key = b3.ESC + "OC" : r7.key = b3.ESC + "[C";
      break;
    case 38:
      if (s18.metaKey) break;
      n4 ? r7.key = b3.ESC + "[1;" + (n4 + 1) + "A" : t3 ? r7.key = b3.ESC + "OA" : r7.key = b3.ESC + "[A";
      break;
    case 40:
      if (s18.metaKey) break;
      n4 ? r7.key = b3.ESC + "[1;" + (n4 + 1) + "B" : t3 ? r7.key = b3.ESC + "OB" : r7.key = b3.ESC + "[B";
      break;
    case 45:
      !s18.shiftKey && !s18.ctrlKey && (r7.key = b3.ESC + "[2~");
      break;
    case 46:
      n4 ? r7.key = b3.ESC + "[3;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[3~";
      break;
    case 36:
      n4 ? r7.key = b3.ESC + "[1;" + (n4 + 1) + "H" : t3 ? r7.key = b3.ESC + "OH" : r7.key = b3.ESC + "[H";
      break;
    case 35:
      n4 ? r7.key = b3.ESC + "[1;" + (n4 + 1) + "F" : t3 ? r7.key = b3.ESC + "OF" : r7.key = b3.ESC + "[F";
      break;
    case 33:
      s18.shiftKey ? r7.type = 2 : s18.ctrlKey ? r7.key = b3.ESC + "[5;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[5~";
      break;
    case 34:
      s18.shiftKey ? r7.type = 3 : s18.ctrlKey ? r7.key = b3.ESC + "[6;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[6~";
      break;
    case 112:
      n4 ? r7.key = b3.ESC + "[1;" + (n4 + 1) + "P" : r7.key = b3.ESC + "OP";
      break;
    case 113:
      n4 ? r7.key = b3.ESC + "[1;" + (n4 + 1) + "Q" : r7.key = b3.ESC + "OQ";
      break;
    case 114:
      n4 ? r7.key = b3.ESC + "[1;" + (n4 + 1) + "R" : r7.key = b3.ESC + "OR";
      break;
    case 115:
      n4 ? r7.key = b3.ESC + "[1;" + (n4 + 1) + "S" : r7.key = b3.ESC + "OS";
      break;
    case 116:
      n4 ? r7.key = b3.ESC + "[15;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[15~";
      break;
    case 117:
      n4 ? r7.key = b3.ESC + "[17;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[17~";
      break;
    case 118:
      n4 ? r7.key = b3.ESC + "[18;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[18~";
      break;
    case 119:
      n4 ? r7.key = b3.ESC + "[19;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[19~";
      break;
    case 120:
      n4 ? r7.key = b3.ESC + "[20;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[20~";
      break;
    case 121:
      n4 ? r7.key = b3.ESC + "[21;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[21~";
      break;
    case 122:
      n4 ? r7.key = b3.ESC + "[23;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[23~";
      break;
    case 123:
      n4 ? r7.key = b3.ESC + "[24;" + (n4 + 1) + "~" : r7.key = b3.ESC + "[24~";
      break;
    default:
      if (s18.ctrlKey && !s18.shiftKey && !s18.altKey && !s18.metaKey) s18.keyCode >= 65 && s18.keyCode <= 90 ? r7.key = String.fromCharCode(s18.keyCode - 64) : s18.keyCode === 32 ? r7.key = b3.NUL : s18.keyCode >= 51 && s18.keyCode <= 55 ? r7.key = String.fromCharCode(s18.keyCode - 51 + 27) : s18.keyCode === 56 ? r7.key = b3.DEL : s18.keyCode === 219 ? r7.key = b3.ESC : s18.keyCode === 220 ? r7.key = b3.FS : s18.keyCode === 221 && (r7.key = b3.GS);
      else if ((!e4 || i12) && s18.altKey && !s18.metaKey) {
        let l4 = gc[s18.keyCode]?.[s18.shiftKey ? 1 : 0];
        if (l4) r7.key = b3.ESC + l4;
        else if (s18.keyCode >= 65 && s18.keyCode <= 90) {
          let a3 = s18.ctrlKey ? s18.keyCode - 64 : s18.keyCode + 32, u3 = String.fromCharCode(a3);
          s18.shiftKey && (u3 = u3.toUpperCase()), r7.key = b3.ESC + u3;
        } else if (s18.keyCode === 32) r7.key = b3.ESC + (s18.ctrlKey ? b3.NUL : " ");
        else if (s18.key === "Dead" && s18.code.startsWith("Key")) {
          let a3 = s18.code.slice(3, 4);
          s18.shiftKey || (a3 = a3.toLowerCase()), r7.key = b3.ESC + a3, r7.cancel = true;
        }
      } else e4 && !s18.altKey && !s18.ctrlKey && !s18.shiftKey && s18.metaKey ? s18.keyCode === 65 && (r7.type = 1) : s18.key && !s18.ctrlKey && !s18.altKey && !s18.metaKey && s18.keyCode >= 48 && s18.key.length === 1 ? r7.key = s18.key : s18.key && s18.ctrlKey && (s18.key === "_" && (r7.key = b3.US), s18.key === "@" && (r7.key = b3.NUL));
      break;
  }
  return r7;
}
function Ec(s18, t3) {
  return s18.text === t3.text && s18.range.start.x === t3.range.start.x && s18.range.start.y === t3.range.start.y && s18.range.end.x === t3.range.end.x && s18.range.end.y === t3.range.end.y;
}
function Tc(s18) {
  return s18.keyCode === 16 || s18.keyCode === 17 || s18.keyCode === 18;
}
var zs, Rl, Ll, M2, S3, Gs, mi, $s, _i, er, tr, ir, we, De, rt, q, js, Hn, Fn, F, rr, ge, Zs, xt, nr, H2, sr, Js, Be, wt, nt, ae, Dt, ce, Qs, or, Re, lr, Wn, Bl, Un, bi, ar, Rt, cr, ro, so, At, lo, ao, oo, ur, zn, Wl, dt, hr, dr, Ee, D2, ye, fe, kt, G, Ct, zl, mr, Gl, fo, $l, $2, Mt, $n, po, br, Vn, gi, qn, Yn, Vl, Pt, ql, Yl, _r, v2, jn, gr, Si, Eu, Tu, Ot, Ei, Bt, Ti, Sr, Iu, yu, vr, Nt, yr, xr, Ii, Zl, vo, go, Jl, Ql, ea, ta, Tr, Ir, bo, ia, $e, Ve, xe, So, ra, Xn, wr, Te, Zn, Dr, na, xu, Fe, st, sa, oa, Eo, la, wu, Du, Ru, Lu, ot, aa, yi, Jn, To, Io, yo, Qn, Rr, es, ua, ha, da, fa, ft, wo, Lr, qe, xi, Do, ma, ts, _a, Ye, kr, ba, Ar, va, _e, Cr, Bh, be, Nh, Fh, Hh, Oo, Wh, Uh, No, Kh, zh, ss, os, wa, mt, Mr, Di, pt, Gh, Y, Da, ls, Wt, He, Q, Pr, lt, Uo, Or, cs, Ri, Br, Nr, Fr, Ca, Ut, Kt, Wr, Ur, Ma, Ko, zo, us, zr, hs, ds, Kr, zt, Gt, Gr, We, at, Li, bt, b3, Ai, fs, $t, ue, he, de, J, ps, j, U, z2, ve, $r, Vr, ct, Vt, Yr, ms, _s, Le, jr, jo, ki, Xr, Na, Yt, jt, Zr, bs, vs, Jr, gs, Qr, Xt, en, tn, Mi, Pi, Oi, Ss, Fa, Zo, Zt, Wa, Ua, Es, Bi, Ts, rn, Is, ys, Jt, nn, Qt, xs, on, Ds, Ya, ja, Xa, Za, Ja, ei, Hi, Wi, re, St, Ki, tl, il, Ui, Qa, ti, Rs, ln, ec, tc, ii, ic, zi, B2, X, an, Ls, Ze, un, cn, ne, Je, cl, $i, hn, ks, Cs, ni, si, nc, dn, ul, hl, li, dl, Ps, fl, ai, Os, ac, se, fn, Ae, pn, Vi, uc, ci, qi, mn, pe, Yi, _n, ji, Xi, Fs, ke, hc, bn, dc, fc, mc, ut, _l, vl, gl, vn, Zi, _c, El, bc, gn, ui, Tl, Sn, gc, ee, En, Us, yl, Tn, Ks, Sc, In, xl, wl, Tt, hi, yn, xn, wn, Ji, Dn, Rn, Ln, Ic, Ue, Dl;
var init_xterm = __esm({
  "node_modules/.pnpm/@xterm+xterm@6.0.0/node_modules/@xterm/xterm/lib/xterm.mjs"() {
    zs = Object.defineProperty;
    Rl = Object.getOwnPropertyDescriptor;
    Ll = (s18, t3) => {
      for (var e4 in t3) zs(s18, e4, { get: t3[e4], enumerable: true });
    };
    M2 = (s18, t3, e4, i12) => {
      for (var r7 = i12 > 1 ? void 0 : i12 ? Rl(t3, e4) : t3, n4 = s18.length - 1, o6; n4 >= 0; n4--) (o6 = s18[n4]) && (r7 = (i12 ? o6(t3, e4, r7) : o6(r7)) || r7);
      return i12 && r7 && zs(t3, e4, r7), r7;
    };
    S3 = (s18, t3) => (e4, i12) => t3(e4, i12, s18);
    Gs = "Terminal input";
    mi = { get: () => Gs, set: (s18) => Gs = s18 };
    $s = "Too much output to announce, navigate to rows manually to read";
    _i = { get: () => $s, set: (s18) => $s = s18 };
    er = class {
      constructor() {
        this._interim = 0;
      }
      clear() {
        this._interim = 0;
      }
      decode(t3, e4) {
        let i12 = t3.length;
        if (!i12) return 0;
        let r7 = 0, n4 = 0;
        if (this._interim) {
          let o6 = t3.charCodeAt(n4++);
          56320 <= o6 && o6 <= 57343 ? e4[r7++] = (this._interim - 55296) * 1024 + o6 - 56320 + 65536 : (e4[r7++] = this._interim, e4[r7++] = o6), this._interim = 0;
        }
        for (let o6 = n4; o6 < i12; ++o6) {
          let l4 = t3.charCodeAt(o6);
          if (55296 <= l4 && l4 <= 56319) {
            if (++o6 >= i12) return this._interim = l4, r7;
            let a3 = t3.charCodeAt(o6);
            56320 <= a3 && a3 <= 57343 ? e4[r7++] = (l4 - 55296) * 1024 + a3 - 56320 + 65536 : (e4[r7++] = l4, e4[r7++] = a3);
            continue;
          }
          l4 !== 65279 && (e4[r7++] = l4);
        }
        return r7;
      }
    };
    tr = class {
      constructor() {
        this.interim = new Uint8Array(3);
      }
      clear() {
        this.interim.fill(0);
      }
      decode(t3, e4) {
        let i12 = t3.length;
        if (!i12) return 0;
        let r7 = 0, n4, o6, l4, a3, u3 = 0, h4 = 0;
        if (this.interim[0]) {
          let _4 = false, p3 = this.interim[0];
          p3 &= (p3 & 224) === 192 ? 31 : (p3 & 240) === 224 ? 15 : 7;
          let m2 = 0, f3;
          for (; (f3 = this.interim[++m2] & 63) && m2 < 4; ) p3 <<= 6, p3 |= f3;
          let A3 = (this.interim[0] & 224) === 192 ? 2 : (this.interim[0] & 240) === 224 ? 3 : 4, R2 = A3 - m2;
          for (; h4 < R2; ) {
            if (h4 >= i12) return 0;
            if (f3 = t3[h4++], (f3 & 192) !== 128) {
              h4--, _4 = true;
              break;
            } else this.interim[m2++] = f3, p3 <<= 6, p3 |= f3 & 63;
          }
          _4 || (A3 === 2 ? p3 < 128 ? h4-- : e4[r7++] = p3 : A3 === 3 ? p3 < 2048 || p3 >= 55296 && p3 <= 57343 || p3 === 65279 || (e4[r7++] = p3) : p3 < 65536 || p3 > 1114111 || (e4[r7++] = p3)), this.interim.fill(0);
        }
        let c4 = i12 - 4, d3 = h4;
        for (; d3 < i12; ) {
          for (; d3 < c4 && !((n4 = t3[d3]) & 128) && !((o6 = t3[d3 + 1]) & 128) && !((l4 = t3[d3 + 2]) & 128) && !((a3 = t3[d3 + 3]) & 128); ) e4[r7++] = n4, e4[r7++] = o6, e4[r7++] = l4, e4[r7++] = a3, d3 += 4;
          if (n4 = t3[d3++], n4 < 128) e4[r7++] = n4;
          else if ((n4 & 224) === 192) {
            if (d3 >= i12) return this.interim[0] = n4, r7;
            if (o6 = t3[d3++], (o6 & 192) !== 128) {
              d3--;
              continue;
            }
            if (u3 = (n4 & 31) << 6 | o6 & 63, u3 < 128) {
              d3--;
              continue;
            }
            e4[r7++] = u3;
          } else if ((n4 & 240) === 224) {
            if (d3 >= i12) return this.interim[0] = n4, r7;
            if (o6 = t3[d3++], (o6 & 192) !== 128) {
              d3--;
              continue;
            }
            if (d3 >= i12) return this.interim[0] = n4, this.interim[1] = o6, r7;
            if (l4 = t3[d3++], (l4 & 192) !== 128) {
              d3--;
              continue;
            }
            if (u3 = (n4 & 15) << 12 | (o6 & 63) << 6 | l4 & 63, u3 < 2048 || u3 >= 55296 && u3 <= 57343 || u3 === 65279) continue;
            e4[r7++] = u3;
          } else if ((n4 & 248) === 240) {
            if (d3 >= i12) return this.interim[0] = n4, r7;
            if (o6 = t3[d3++], (o6 & 192) !== 128) {
              d3--;
              continue;
            }
            if (d3 >= i12) return this.interim[0] = n4, this.interim[1] = o6, r7;
            if (l4 = t3[d3++], (l4 & 192) !== 128) {
              d3--;
              continue;
            }
            if (d3 >= i12) return this.interim[0] = n4, this.interim[1] = o6, this.interim[2] = l4, r7;
            if (a3 = t3[d3++], (a3 & 192) !== 128) {
              d3--;
              continue;
            }
            if (u3 = (n4 & 7) << 18 | (o6 & 63) << 12 | (l4 & 63) << 6 | a3 & 63, u3 < 65536 || u3 > 1114111) continue;
            e4[r7++] = u3;
          }
        }
        return r7;
      }
    };
    ir = "";
    we = " ";
    De = class s4 {
      constructor() {
        this.fg = 0;
        this.bg = 0;
        this.extended = new rt();
      }
      static toColorRGB(t3) {
        return [t3 >>> 16 & 255, t3 >>> 8 & 255, t3 & 255];
      }
      static fromColorRGB(t3) {
        return (t3[0] & 255) << 16 | (t3[1] & 255) << 8 | t3[2] & 255;
      }
      clone() {
        let t3 = new s4();
        return t3.fg = this.fg, t3.bg = this.bg, t3.extended = this.extended.clone(), t3;
      }
      isInverse() {
        return this.fg & 67108864;
      }
      isBold() {
        return this.fg & 134217728;
      }
      isUnderline() {
        return this.hasExtendedAttrs() && this.extended.underlineStyle !== 0 ? 1 : this.fg & 268435456;
      }
      isBlink() {
        return this.fg & 536870912;
      }
      isInvisible() {
        return this.fg & 1073741824;
      }
      isItalic() {
        return this.bg & 67108864;
      }
      isDim() {
        return this.bg & 134217728;
      }
      isStrikethrough() {
        return this.fg & 2147483648;
      }
      isProtected() {
        return this.bg & 536870912;
      }
      isOverline() {
        return this.bg & 1073741824;
      }
      getFgColorMode() {
        return this.fg & 50331648;
      }
      getBgColorMode() {
        return this.bg & 50331648;
      }
      isFgRGB() {
        return (this.fg & 50331648) === 50331648;
      }
      isBgRGB() {
        return (this.bg & 50331648) === 50331648;
      }
      isFgPalette() {
        return (this.fg & 50331648) === 16777216 || (this.fg & 50331648) === 33554432;
      }
      isBgPalette() {
        return (this.bg & 50331648) === 16777216 || (this.bg & 50331648) === 33554432;
      }
      isFgDefault() {
        return (this.fg & 50331648) === 0;
      }
      isBgDefault() {
        return (this.bg & 50331648) === 0;
      }
      isAttributeDefault() {
        return this.fg === 0 && this.bg === 0;
      }
      getFgColor() {
        switch (this.fg & 50331648) {
          case 16777216:
          case 33554432:
            return this.fg & 255;
          case 50331648:
            return this.fg & 16777215;
          default:
            return -1;
        }
      }
      getBgColor() {
        switch (this.bg & 50331648) {
          case 16777216:
          case 33554432:
            return this.bg & 255;
          case 50331648:
            return this.bg & 16777215;
          default:
            return -1;
        }
      }
      hasExtendedAttrs() {
        return this.bg & 268435456;
      }
      updateExtended() {
        this.extended.isEmpty() ? this.bg &= -268435457 : this.bg |= 268435456;
      }
      getUnderlineColor() {
        if (this.bg & 268435456 && ~this.extended.underlineColor) switch (this.extended.underlineColor & 50331648) {
          case 16777216:
          case 33554432:
            return this.extended.underlineColor & 255;
          case 50331648:
            return this.extended.underlineColor & 16777215;
          default:
            return this.getFgColor();
        }
        return this.getFgColor();
      }
      getUnderlineColorMode() {
        return this.bg & 268435456 && ~this.extended.underlineColor ? this.extended.underlineColor & 50331648 : this.getFgColorMode();
      }
      isUnderlineColorRGB() {
        return this.bg & 268435456 && ~this.extended.underlineColor ? (this.extended.underlineColor & 50331648) === 50331648 : this.isFgRGB();
      }
      isUnderlineColorPalette() {
        return this.bg & 268435456 && ~this.extended.underlineColor ? (this.extended.underlineColor & 50331648) === 16777216 || (this.extended.underlineColor & 50331648) === 33554432 : this.isFgPalette();
      }
      isUnderlineColorDefault() {
        return this.bg & 268435456 && ~this.extended.underlineColor ? (this.extended.underlineColor & 50331648) === 0 : this.isFgDefault();
      }
      getUnderlineStyle() {
        return this.fg & 268435456 ? this.bg & 268435456 ? this.extended.underlineStyle : 1 : 0;
      }
      getUnderlineVariantOffset() {
        return this.extended.underlineVariantOffset;
      }
    };
    rt = class s5 {
      constructor(t3 = 0, e4 = 0) {
        this._ext = 0;
        this._urlId = 0;
        this._ext = t3, this._urlId = e4;
      }
      get ext() {
        return this._urlId ? this._ext & -469762049 | this.underlineStyle << 26 : this._ext;
      }
      set ext(t3) {
        this._ext = t3;
      }
      get underlineStyle() {
        return this._urlId ? 5 : (this._ext & 469762048) >> 26;
      }
      set underlineStyle(t3) {
        this._ext &= -469762049, this._ext |= t3 << 26 & 469762048;
      }
      get underlineColor() {
        return this._ext & 67108863;
      }
      set underlineColor(t3) {
        this._ext &= -67108864, this._ext |= t3 & 67108863;
      }
      get urlId() {
        return this._urlId;
      }
      set urlId(t3) {
        this._urlId = t3;
      }
      get underlineVariantOffset() {
        let t3 = (this._ext & 3758096384) >> 29;
        return t3 < 0 ? t3 ^ 4294967288 : t3;
      }
      set underlineVariantOffset(t3) {
        this._ext &= 536870911, this._ext |= t3 << 29 & 3758096384;
      }
      clone() {
        return new s5(this._ext, this._urlId);
      }
      isEmpty() {
        return this.underlineStyle === 0 && this._urlId === 0;
      }
    };
    q = class s6 extends De {
      constructor() {
        super(...arguments);
        this.content = 0;
        this.fg = 0;
        this.bg = 0;
        this.extended = new rt();
        this.combinedData = "";
      }
      static fromCharData(e4) {
        let i12 = new s6();
        return i12.setFromCharData(e4), i12;
      }
      isCombined() {
        return this.content & 2097152;
      }
      getWidth() {
        return this.content >> 22;
      }
      getChars() {
        return this.content & 2097152 ? this.combinedData : this.content & 2097151 ? Ce(this.content & 2097151) : "";
      }
      getCode() {
        return this.isCombined() ? this.combinedData.charCodeAt(this.combinedData.length - 1) : this.content & 2097151;
      }
      setFromCharData(e4) {
        this.fg = e4[0], this.bg = 0;
        let i12 = false;
        if (e4[1].length > 2) i12 = true;
        else if (e4[1].length === 2) {
          let r7 = e4[1].charCodeAt(0);
          if (55296 <= r7 && r7 <= 56319) {
            let n4 = e4[1].charCodeAt(1);
            56320 <= n4 && n4 <= 57343 ? this.content = (r7 - 55296) * 1024 + n4 - 56320 + 65536 | e4[2] << 22 : i12 = true;
          } else i12 = true;
        } else this.content = e4[1].charCodeAt(0) | e4[2] << 22;
        i12 && (this.combinedData = e4[1], this.content = 2097152 | e4[2] << 22);
      }
      getAsCharData() {
        return [this.fg, this.getChars(), this.getWidth(), this.getCode()];
      }
    };
    js = "di$target";
    Hn = "di$dependencies";
    Fn = /* @__PURE__ */ new Map();
    F = ie("BufferService");
    rr = ie("CoreMouseService");
    ge = ie("CoreService");
    Zs = ie("CharsetService");
    xt = ie("InstantiationService");
    nr = ie("LogService");
    H2 = ie("OptionsService");
    sr = ie("OscLinkService");
    Js = ie("UnicodeService");
    Be = ie("DecorationService");
    wt = class {
      constructor(t3, e4, i12) {
        this._bufferService = t3;
        this._optionsService = e4;
        this._oscLinkService = i12;
      }
      provideLinks(t3, e4) {
        let i12 = this._bufferService.buffer.lines.get(t3 - 1);
        if (!i12) {
          e4(void 0);
          return;
        }
        let r7 = [], n4 = this._optionsService.rawOptions.linkHandler, o6 = new q(), l4 = i12.getTrimmedLength(), a3 = -1, u3 = -1, h4 = false;
        for (let c4 = 0; c4 < l4; c4++) if (!(u3 === -1 && !i12.hasContent(c4))) {
          if (i12.loadCell(c4, o6), o6.hasExtendedAttrs() && o6.extended.urlId) if (u3 === -1) {
            u3 = c4, a3 = o6.extended.urlId;
            continue;
          } else h4 = o6.extended.urlId !== a3;
          else u3 !== -1 && (h4 = true);
          if (h4 || u3 !== -1 && c4 === l4 - 1) {
            let d3 = this._oscLinkService.getLinkData(a3)?.uri;
            if (d3) {
              let _4 = { start: { x: u3 + 1, y: t3 }, end: { x: c4 + (!h4 && c4 === l4 - 1 ? 1 : 0), y: t3 } }, p3 = false;
              if (!n4?.allowNonHttpProtocols) try {
                let m2 = new URL(d3);
                ["http:", "https:"].includes(m2.protocol) || (p3 = true);
              } catch {
                p3 = true;
              }
              p3 || r7.push({ text: d3, range: _4, activate: (m2, f3) => n4 ? n4.activate(m2, f3, _4) : Ol(m2, f3), hover: (m2, f3) => n4?.hover?.(m2, f3, _4), leave: (m2, f3) => n4?.leave?.(m2, f3, _4) });
            }
            h4 = false, o6.hasExtendedAttrs() && o6.extended.urlId ? (u3 = c4, a3 = o6.extended.urlId) : (u3 = -1, a3 = -1);
          }
        }
        e4(r7);
      }
    };
    wt = M2([S3(0, F), S3(1, H2), S3(2, sr)], wt);
    nt = ie("CharSizeService");
    ae = ie("CoreBrowserService");
    Dt = ie("MouseService");
    ce = ie("RenderService");
    Qs = ie("SelectionService");
    or = ie("CharacterJoinerService");
    Re = ie("ThemeService");
    lr = ie("LinkProviderService");
    Wn = class {
      constructor() {
        this.listeners = [], this.unexpectedErrorHandler = function(t3) {
          setTimeout(() => {
            throw t3.stack ? ar.isErrorNoTelemetry(t3) ? new ar(t3.message + `

` + t3.stack) : new Error(t3.message + `

` + t3.stack) : t3;
          }, 0);
        };
      }
      addListener(t3) {
        return this.listeners.push(t3), () => {
          this._removeListener(t3);
        };
      }
      emit(t3) {
        this.listeners.forEach((e4) => {
          e4(t3);
        });
      }
      _removeListener(t3) {
        this.listeners.splice(this.listeners.indexOf(t3), 1);
      }
      setUnexpectedErrorHandler(t3) {
        this.unexpectedErrorHandler = t3;
      }
      getUnexpectedErrorHandler() {
        return this.unexpectedErrorHandler;
      }
      onUnexpectedError(t3) {
        this.unexpectedErrorHandler(t3), this.emit(t3);
      }
      onUnexpectedExternalError(t3) {
        this.unexpectedErrorHandler(t3);
      }
    };
    Bl = new Wn();
    Un = "Canceled";
    bi = class extends Error {
      constructor() {
        super(Un), this.name = this.message;
      }
    };
    ar = class s7 extends Error {
      constructor(t3) {
        super(t3), this.name = "CodeExpectedError";
      }
      static fromError(t3) {
        if (t3 instanceof s7) return t3;
        let e4 = new s7();
        return e4.message = t3.message, e4.stack = t3.stack, e4;
      }
      static isErrorNoTelemetry(t3) {
        return t3.name === "CodeExpectedError";
      }
    };
    Rt = class s8 extends Error {
      constructor(t3) {
        super(t3 || "An unexpected bug occurred."), Object.setPrototypeOf(this, s8.prototype);
      }
    };
    cr = class cr2 {
      constructor(t3) {
        this._array = t3;
        this._findLastMonotonousLastIdx = 0;
      }
      findLastMonotonous(t3) {
        if (cr2.assertInvariants) {
          if (this._prevFindLastPredicate) {
            for (let i12 of this._array) if (this._prevFindLastPredicate(i12) && !t3(i12)) throw new Error("MonotonousArray: current predicate must be weaker than (or equal to) the previous predicate.");
          }
          this._prevFindLastPredicate = t3;
        }
        let e4 = Fl(this._array, t3, this._findLastMonotonousLastIdx);
        return this._findLastMonotonousLastIdx = e4 + 1, e4 === -1 ? void 0 : this._array[e4];
      }
    };
    cr.assertInvariants = false;
    ((l4) => {
      function s18(a3) {
        return a3 < 0;
      }
      l4.isLessThan = s18;
      function t3(a3) {
        return a3 <= 0;
      }
      l4.isLessThanOrEqual = t3;
      function e4(a3) {
        return a3 > 0;
      }
      l4.isGreaterThan = e4;
      function i12(a3) {
        return a3 === 0;
      }
      l4.isNeitherLessOrGreaterThan = i12, l4.greaterThan = 1, l4.lessThan = -1, l4.neitherLessOrGreaterThan = 0;
    })(ro || (ro = {}));
    so = (s18, t3) => s18 - t3;
    At = class At2 {
      constructor(t3) {
        this.iterate = t3;
      }
      forEach(t3) {
        this.iterate((e4) => (t3(e4), true));
      }
      toArray() {
        let t3 = [];
        return this.iterate((e4) => (t3.push(e4), true)), t3;
      }
      filter(t3) {
        return new At2((e4) => this.iterate((i12) => t3(i12) ? e4(i12) : true));
      }
      map(t3) {
        return new At2((e4) => this.iterate((i12) => e4(t3(i12))));
      }
      some(t3) {
        let e4 = false;
        return this.iterate((i12) => (e4 = t3(i12), !e4)), e4;
      }
      findFirst(t3) {
        let e4;
        return this.iterate((i12) => t3(i12) ? (e4 = i12, false) : true), e4;
      }
      findLast(t3) {
        let e4;
        return this.iterate((i12) => (t3(i12) && (e4 = i12), true)), e4;
      }
      findLastMaxBy(t3) {
        let e4, i12 = true;
        return this.iterate((r7) => ((i12 || ro.isGreaterThan(t3(r7, e4))) && (i12 = false, e4 = r7), true)), e4;
      }
    };
    At.empty = new At((t3) => {
    });
    oo = class {
      constructor(t3, e4) {
        this.toKey = e4;
        this._map = /* @__PURE__ */ new Map();
        this[lo] = "SetWithKey";
        for (let i12 of t3) this.add(i12);
      }
      get size() {
        return this._map.size;
      }
      add(t3) {
        let e4 = this.toKey(t3);
        return this._map.set(e4, t3), this;
      }
      delete(t3) {
        return this._map.delete(this.toKey(t3));
      }
      has(t3) {
        return this._map.has(this.toKey(t3));
      }
      *entries() {
        for (let t3 of this._map.values()) yield [t3, t3];
      }
      keys() {
        return this.values();
      }
      *values() {
        for (let t3 of this._map.values()) yield t3;
      }
      clear() {
        this._map.clear();
      }
      forEach(t3, e4) {
        this._map.forEach((i12) => t3.call(e4, i12, i12, this));
      }
      [(ao = Symbol.iterator, lo = Symbol.toStringTag, ao)]() {
        return this.values();
      }
    };
    ur = class {
      constructor() {
        this.map = /* @__PURE__ */ new Map();
      }
      add(t3, e4) {
        let i12 = this.map.get(t3);
        i12 || (i12 = /* @__PURE__ */ new Set(), this.map.set(t3, i12)), i12.add(e4);
      }
      delete(t3, e4) {
        let i12 = this.map.get(t3);
        i12 && (i12.delete(e4), i12.size === 0 && this.map.delete(t3));
      }
      forEach(t3, e4) {
        let i12 = this.map.get(t3);
        i12 && i12.forEach(e4);
      }
      get(t3) {
        let e4 = this.map.get(t3);
        return e4 || /* @__PURE__ */ new Set();
      }
    };
    ((O3) => {
      function s18(I2) {
        return I2 && typeof I2 == "object" && typeof I2[Symbol.iterator] == "function";
      }
      O3.is = s18;
      let t3 = Object.freeze([]);
      function e4() {
        return t3;
      }
      O3.empty = e4;
      function* i12(I2) {
        yield I2;
      }
      O3.single = i12;
      function r7(I2) {
        return s18(I2) ? I2 : i12(I2);
      }
      O3.wrap = r7;
      function n4(I2) {
        return I2 || t3;
      }
      O3.from = n4;
      function* o6(I2) {
        for (let k5 = I2.length - 1; k5 >= 0; k5--) yield I2[k5];
      }
      O3.reverse = o6;
      function l4(I2) {
        return !I2 || I2[Symbol.iterator]().next().done === true;
      }
      O3.isEmpty = l4;
      function a3(I2) {
        return I2[Symbol.iterator]().next().value;
      }
      O3.first = a3;
      function u3(I2, k5) {
        let P4 = 0;
        for (let oe of I2) if (k5(oe, P4++)) return true;
        return false;
      }
      O3.some = u3;
      function h4(I2, k5) {
        for (let P4 of I2) if (k5(P4)) return P4;
      }
      O3.find = h4;
      function* c4(I2, k5) {
        for (let P4 of I2) k5(P4) && (yield P4);
      }
      O3.filter = c4;
      function* d3(I2, k5) {
        let P4 = 0;
        for (let oe of I2) yield k5(oe, P4++);
      }
      O3.map = d3;
      function* _4(I2, k5) {
        let P4 = 0;
        for (let oe of I2) yield* k5(oe, P4++);
      }
      O3.flatMap = _4;
      function* p3(...I2) {
        for (let k5 of I2) yield* k5;
      }
      O3.concat = p3;
      function m2(I2, k5, P4) {
        let oe = P4;
        for (let Me3 of I2) oe = k5(oe, Me3);
        return oe;
      }
      O3.reduce = m2;
      function* f3(I2, k5, P4 = I2.length) {
        for (k5 < 0 && (k5 += I2.length), P4 < 0 ? P4 += I2.length : P4 > I2.length && (P4 = I2.length); k5 < P4; k5++) yield I2[k5];
      }
      O3.slice = f3;
      function A3(I2, k5 = Number.POSITIVE_INFINITY) {
        let P4 = [];
        if (k5 === 0) return [P4, I2];
        let oe = I2[Symbol.iterator]();
        for (let Me3 = 0; Me3 < k5; Me3++) {
          let Pe3 = oe.next();
          if (Pe3.done) return [P4, O3.empty()];
          P4.push(Pe3.value);
        }
        return [P4, { [Symbol.iterator]() {
          return oe;
        } }];
      }
      O3.consume = A3;
      async function R2(I2) {
        let k5 = [];
        for await (let P4 of I2) k5.push(P4);
        return Promise.resolve(k5);
      }
      O3.asyncToArray = R2;
    })(zn || (zn = {}));
    Wl = false;
    dt = null;
    hr = class hr2 {
      constructor() {
        this.livingDisposables = /* @__PURE__ */ new Map();
      }
      getDisposableData(t3) {
        let e4 = this.livingDisposables.get(t3);
        return e4 || (e4 = { parent: null, source: null, isSingleton: false, value: t3, idx: hr2.idx++ }, this.livingDisposables.set(t3, e4)), e4;
      }
      trackDisposable(t3) {
        let e4 = this.getDisposableData(t3);
        e4.source || (e4.source = new Error().stack);
      }
      setParent(t3, e4) {
        let i12 = this.getDisposableData(t3);
        i12.parent = e4;
      }
      markAsDisposed(t3) {
        this.livingDisposables.delete(t3);
      }
      markAsSingleton(t3) {
        this.getDisposableData(t3).isSingleton = true;
      }
      getRootParent(t3, e4) {
        let i12 = e4.get(t3);
        if (i12) return i12;
        let r7 = t3.parent ? this.getRootParent(this.getDisposableData(t3.parent), e4) : t3;
        return e4.set(t3, r7), r7;
      }
      getTrackedDisposables() {
        let t3 = /* @__PURE__ */ new Map();
        return [...this.livingDisposables.entries()].filter(([, i12]) => i12.source !== null && !this.getRootParent(i12, t3).isSingleton).flatMap(([i12]) => i12);
      }
      computeLeakingDisposables(t3 = 10, e4) {
        let i12;
        if (e4) i12 = e4;
        else {
          let a3 = /* @__PURE__ */ new Map(), u3 = [...this.livingDisposables.values()].filter((c4) => c4.source !== null && !this.getRootParent(c4, a3).isSingleton);
          if (u3.length === 0) return;
          let h4 = new Set(u3.map((c4) => c4.value));
          if (i12 = u3.filter((c4) => !(c4.parent && h4.has(c4.parent))), i12.length === 0) throw new Error("There are cyclic diposable chains!");
        }
        if (!i12) return;
        function r7(a3) {
          function u3(c4, d3) {
            for (; c4.length > 0 && d3.some((_4) => typeof _4 == "string" ? _4 === c4[0] : c4[0].match(_4)); ) c4.shift();
          }
          let h4 = a3.source.split(`
`).map((c4) => c4.trim().replace("at ", "")).filter((c4) => c4 !== "");
          return u3(h4, ["Error", /^trackDisposable \(.*\)$/, /^DisposableTracker.trackDisposable \(.*\)$/]), h4.reverse();
        }
        let n4 = new ur();
        for (let a3 of i12) {
          let u3 = r7(a3);
          for (let h4 = 0; h4 <= u3.length; h4++) n4.add(u3.slice(0, h4).join(`
`), a3);
        }
        i12.sort(no((a3) => a3.idx, so));
        let o6 = "", l4 = 0;
        for (let a3 of i12.slice(0, t3)) {
          l4++;
          let u3 = r7(a3), h4 = [];
          for (let c4 = 0; c4 < u3.length; c4++) {
            let d3 = u3[c4];
            d3 = `(shared with ${n4.get(u3.slice(0, c4 + 1).join(`
`)).size}/${i12.length} leaks) at ${d3}`;
            let p3 = n4.get(u3.slice(0, c4).join(`
`)), m2 = co([...p3].map((f3) => r7(f3)[c4]), (f3) => f3);
            delete m2[u3[c4]];
            for (let [f3, A3] of Object.entries(m2)) h4.unshift(`    - stacktraces of ${A3.length} other leaks continue with ${f3}`);
            h4.unshift(d3);
          }
          o6 += `


==================== Leaking disposable ${l4}/${i12.length}: ${a3.value.constructor.name} ====================
${h4.join(`
`)}
============================================================

`;
        }
        return i12.length > t3 && (o6 += `


... and ${i12.length - t3} more leaking disposables

`), { leaks: i12, details: o6 };
      }
    };
    hr.idx = 0;
    if (Wl) {
      let s18 = "__is_disposable_tracked__";
      Ul(new class {
        trackDisposable(t3) {
          let e4 = new Error("Potentially leaked disposable").stack;
          setTimeout(() => {
            t3[s18] || console.log(e4);
          }, 3e3);
        }
        setParent(t3, e4) {
          if (t3 && t3 !== D2.None) try {
            t3[s18] = true;
          } catch {
          }
        }
        markAsDisposed(t3) {
          if (t3 && t3 !== D2.None) try {
            t3[s18] = true;
          } catch {
          }
        }
        markAsSingleton(t3) {
        }
      }());
    }
    dr = class dr2 {
      constructor() {
        this._toDispose = /* @__PURE__ */ new Set();
        this._isDisposed = false;
        fr(this);
      }
      dispose() {
        this._isDisposed || (pr(this), this._isDisposed = true, this.clear());
      }
      get isDisposed() {
        return this._isDisposed;
      }
      clear() {
        if (this._toDispose.size !== 0) try {
          Ne(this._toDispose);
        } finally {
          this._toDispose.clear();
        }
      }
      add(t3) {
        if (!t3) return t3;
        if (t3 === this) throw new Error("Cannot register a disposable on itself!");
        return vi(t3, this), this._isDisposed ? dr2.DISABLE_DISPOSED_WARNING || console.warn(new Error("Trying to add a disposable to a DisposableStore that has already been disposed of. The added object will be leaked!").stack) : this._toDispose.add(t3), t3;
      }
      delete(t3) {
        if (t3) {
          if (t3 === this) throw new Error("Cannot dispose a disposable on itself!");
          this._toDispose.delete(t3), t3.dispose();
        }
      }
      deleteAndLeak(t3) {
        t3 && this._toDispose.has(t3) && (this._toDispose.delete(t3), vi(t3, null));
      }
    };
    dr.DISABLE_DISPOSED_WARNING = false;
    Ee = dr;
    D2 = class {
      constructor() {
        this._store = new Ee();
        fr(this), vi(this._store, this);
      }
      dispose() {
        pr(this), this._store.dispose();
      }
      _register(t3) {
        if (t3 === this) throw new Error("Cannot register a disposable on itself!");
        return this._store.add(t3);
      }
    };
    D2.None = Object.freeze({ dispose() {
    } });
    ye = class {
      constructor() {
        this._isDisposed = false;
        fr(this);
      }
      get value() {
        return this._isDisposed ? void 0 : this._value;
      }
      set value(t3) {
        this._isDisposed || t3 === this._value || (this._value?.dispose(), t3 && vi(t3, this), this._value = t3);
      }
      clear() {
        this.value = void 0;
      }
      dispose() {
        this._isDisposed = true, pr(this), this._value?.dispose(), this._value = void 0;
      }
      clearAndLeak() {
        let t3 = this._value;
        return this._value = void 0, t3 && vi(t3, null), t3;
      }
    };
    fe = typeof window == "object" ? window : globalThis;
    kt = class kt2 {
      constructor(t3) {
        this.element = t3, this.next = kt2.Undefined, this.prev = kt2.Undefined;
      }
    };
    kt.Undefined = new kt(void 0);
    G = kt;
    Ct = class {
      constructor() {
        this._first = G.Undefined;
        this._last = G.Undefined;
        this._size = 0;
      }
      get size() {
        return this._size;
      }
      isEmpty() {
        return this._first === G.Undefined;
      }
      clear() {
        let t3 = this._first;
        for (; t3 !== G.Undefined; ) {
          let e4 = t3.next;
          t3.prev = G.Undefined, t3.next = G.Undefined, t3 = e4;
        }
        this._first = G.Undefined, this._last = G.Undefined, this._size = 0;
      }
      unshift(t3) {
        return this._insert(t3, false);
      }
      push(t3) {
        return this._insert(t3, true);
      }
      _insert(t3, e4) {
        let i12 = new G(t3);
        if (this._first === G.Undefined) this._first = i12, this._last = i12;
        else if (e4) {
          let n4 = this._last;
          this._last = i12, i12.prev = n4, n4.next = i12;
        } else {
          let n4 = this._first;
          this._first = i12, i12.next = n4, n4.prev = i12;
        }
        this._size += 1;
        let r7 = false;
        return () => {
          r7 || (r7 = true, this._remove(i12));
        };
      }
      shift() {
        if (this._first !== G.Undefined) {
          let t3 = this._first.element;
          return this._remove(this._first), t3;
        }
      }
      pop() {
        if (this._last !== G.Undefined) {
          let t3 = this._last.element;
          return this._remove(this._last), t3;
        }
      }
      _remove(t3) {
        if (t3.prev !== G.Undefined && t3.next !== G.Undefined) {
          let e4 = t3.prev;
          e4.next = t3.next, t3.next.prev = e4;
        } else t3.prev === G.Undefined && t3.next === G.Undefined ? (this._first = G.Undefined, this._last = G.Undefined) : t3.next === G.Undefined ? (this._last = this._last.prev, this._last.next = G.Undefined) : t3.prev === G.Undefined && (this._first = this._first.next, this._first.prev = G.Undefined);
        this._size -= 1;
      }
      *[Symbol.iterator]() {
        let t3 = this._first;
        for (; t3 !== G.Undefined; ) yield t3.element, t3 = t3.next;
      }
    };
    zl = globalThis.performance && typeof globalThis.performance.now == "function";
    mr = class s9 {
      static create(t3) {
        return new s9(t3);
      }
      constructor(t3) {
        this._now = zl && t3 === false ? Date.now : globalThis.performance.now.bind(globalThis.performance), this._startTime = this._now(), this._stopTime = -1;
      }
      stop() {
        this._stopTime = this._now();
      }
      reset() {
        this._startTime = this._now(), this._stopTime = -1;
      }
      elapsed() {
        return this._stopTime !== -1 ? this._stopTime - this._startTime : this._now() - this._startTime;
      }
    };
    Gl = false;
    fo = false;
    $l = false;
    ((Qe3) => {
      Qe3.None = () => D2.None;
      function t3(y3) {
        if ($l) {
          let { onDidAddListener: T2 } = y3, g3 = gi.create(), w3 = 0;
          y3.onDidAddListener = () => {
            ++w3 === 2 && (console.warn("snapshotted emitter LIKELY used public and SHOULD HAVE BEEN created with DisposableStore. snapshotted here"), g3.print()), T2?.();
          };
        }
      }
      function e4(y3, T2) {
        return d3(y3, () => {
        }, 0, void 0, true, void 0, T2);
      }
      Qe3.defer = e4;
      function i12(y3) {
        return (T2, g3 = null, w3) => {
          let E2 = false, x2;
          return x2 = y3((N3) => {
            if (!E2) return x2 ? x2.dispose() : E2 = true, T2.call(g3, N3);
          }, null, w3), E2 && x2.dispose(), x2;
        };
      }
      Qe3.once = i12;
      function r7(y3, T2, g3) {
        return h4((w3, E2 = null, x2) => y3((N3) => w3.call(E2, T2(N3)), null, x2), g3);
      }
      Qe3.map = r7;
      function n4(y3, T2, g3) {
        return h4((w3, E2 = null, x2) => y3((N3) => {
          T2(N3), w3.call(E2, N3);
        }, null, x2), g3);
      }
      Qe3.forEach = n4;
      function o6(y3, T2, g3) {
        return h4((w3, E2 = null, x2) => y3((N3) => T2(N3) && w3.call(E2, N3), null, x2), g3);
      }
      Qe3.filter = o6;
      function l4(y3) {
        return y3;
      }
      Qe3.signal = l4;
      function a3(...y3) {
        return (T2, g3 = null, w3) => {
          let E2 = ho(...y3.map((x2) => x2((N3) => T2.call(g3, N3))));
          return c4(E2, w3);
        };
      }
      Qe3.any = a3;
      function u3(y3, T2, g3, w3) {
        let E2 = g3;
        return r7(y3, (x2) => (E2 = T2(E2, x2), E2), w3);
      }
      Qe3.reduce = u3;
      function h4(y3, T2) {
        let g3, w3 = { onWillAddFirstListener() {
          g3 = y3(E2.fire, E2);
        }, onDidRemoveLastListener() {
          g3?.dispose();
        } };
        T2 || t3(w3);
        let E2 = new v2(w3);
        return T2?.add(E2), E2.event;
      }
      function c4(y3, T2) {
        return T2 instanceof Array ? T2.push(y3) : T2 && T2.add(y3), y3;
      }
      function d3(y3, T2, g3 = 100, w3 = false, E2 = false, x2, N3) {
        let Z3, te3, Oe3, ze3 = 0, le3, et2 = { leakWarningThreshold: x2, onWillAddFirstListener() {
          Z3 = y3((ht2) => {
            ze3++, te3 = T2(te3, ht2), w3 && !Oe3 && (me3.fire(te3), te3 = void 0), le3 = () => {
              let fi2 = te3;
              te3 = void 0, Oe3 = void 0, (!w3 || ze3 > 1) && me3.fire(fi2), ze3 = 0;
            }, typeof g3 == "number" ? (clearTimeout(Oe3), Oe3 = setTimeout(le3, g3)) : Oe3 === void 0 && (Oe3 = 0, queueMicrotask(le3));
          });
        }, onWillRemoveListener() {
          E2 && ze3 > 0 && le3?.();
        }, onDidRemoveLastListener() {
          le3 = void 0, Z3.dispose();
        } };
        N3 || t3(et2);
        let me3 = new v2(et2);
        return N3?.add(me3), me3.event;
      }
      Qe3.debounce = d3;
      function _4(y3, T2 = 0, g3) {
        return Qe3.debounce(y3, (w3, E2) => w3 ? (w3.push(E2), w3) : [E2], T2, void 0, true, void 0, g3);
      }
      Qe3.accumulate = _4;
      function p3(y3, T2 = (w3, E2) => w3 === E2, g3) {
        let w3 = true, E2;
        return o6(y3, (x2) => {
          let N3 = w3 || !T2(x2, E2);
          return w3 = false, E2 = x2, N3;
        }, g3);
      }
      Qe3.latch = p3;
      function m2(y3, T2, g3) {
        return [Qe3.filter(y3, T2, g3), Qe3.filter(y3, (w3) => !T2(w3), g3)];
      }
      Qe3.split = m2;
      function f3(y3, T2 = false, g3 = [], w3) {
        let E2 = g3.slice(), x2 = y3((te3) => {
          E2 ? E2.push(te3) : Z3.fire(te3);
        });
        w3 && w3.add(x2);
        let N3 = () => {
          E2?.forEach((te3) => Z3.fire(te3)), E2 = null;
        }, Z3 = new v2({ onWillAddFirstListener() {
          x2 || (x2 = y3((te3) => Z3.fire(te3)), w3 && w3.add(x2));
        }, onDidAddFirstListener() {
          E2 && (T2 ? setTimeout(N3) : N3());
        }, onDidRemoveLastListener() {
          x2 && x2.dispose(), x2 = null;
        } });
        return w3 && w3.add(Z3), Z3.event;
      }
      Qe3.buffer = f3;
      function A3(y3, T2) {
        return (w3, E2, x2) => {
          let N3 = T2(new O3());
          return y3(function(Z3) {
            let te3 = N3.evaluate(Z3);
            te3 !== R2 && w3.call(E2, te3);
          }, void 0, x2);
        };
      }
      Qe3.chain = A3;
      let R2 = Symbol("HaltChainable");
      class O3 {
        constructor() {
          this.steps = [];
        }
        map(T2) {
          return this.steps.push(T2), this;
        }
        forEach(T2) {
          return this.steps.push((g3) => (T2(g3), g3)), this;
        }
        filter(T2) {
          return this.steps.push((g3) => T2(g3) ? g3 : R2), this;
        }
        reduce(T2, g3) {
          let w3 = g3;
          return this.steps.push((E2) => (w3 = T2(w3, E2), w3)), this;
        }
        latch(T2 = (g3, w3) => g3 === w3) {
          let g3 = true, w3;
          return this.steps.push((E2) => {
            let x2 = g3 || !T2(E2, w3);
            return g3 = false, w3 = E2, x2 ? E2 : R2;
          }), this;
        }
        evaluate(T2) {
          for (let g3 of this.steps) if (T2 = g3(T2), T2 === R2) break;
          return T2;
        }
      }
      function I2(y3, T2, g3 = (w3) => w3) {
        let w3 = (...Z3) => N3.fire(g3(...Z3)), E2 = () => y3.on(T2, w3), x2 = () => y3.removeListener(T2, w3), N3 = new v2({ onWillAddFirstListener: E2, onDidRemoveLastListener: x2 });
        return N3.event;
      }
      Qe3.fromNodeEventEmitter = I2;
      function k5(y3, T2, g3 = (w3) => w3) {
        let w3 = (...Z3) => N3.fire(g3(...Z3)), E2 = () => y3.addEventListener(T2, w3), x2 = () => y3.removeEventListener(T2, w3), N3 = new v2({ onWillAddFirstListener: E2, onDidRemoveLastListener: x2 });
        return N3.event;
      }
      Qe3.fromDOMEventEmitter = k5;
      function P4(y3) {
        return new Promise((T2) => i12(y3)(T2));
      }
      Qe3.toPromise = P4;
      function oe(y3) {
        let T2 = new v2();
        return y3.then((g3) => {
          T2.fire(g3);
        }, () => {
          T2.fire(void 0);
        }).finally(() => {
          T2.dispose();
        }), T2.event;
      }
      Qe3.fromPromise = oe;
      function Me3(y3, T2) {
        return y3((g3) => T2.fire(g3));
      }
      Qe3.forward = Me3;
      function Pe3(y3, T2, g3) {
        return T2(g3), y3((w3) => T2(w3));
      }
      Qe3.runAndSubscribe = Pe3;
      class Ke {
        constructor(T2, g3) {
          this._observable = T2;
          this._counter = 0;
          this._hasChanged = false;
          let w3 = { onWillAddFirstListener: () => {
            T2.addObserver(this);
          }, onDidRemoveLastListener: () => {
            T2.removeObserver(this);
          } };
          g3 || t3(w3), this.emitter = new v2(w3), g3 && g3.add(this.emitter);
        }
        beginUpdate(T2) {
          this._counter++;
        }
        handlePossibleChange(T2) {
        }
        handleChange(T2, g3) {
          this._hasChanged = true;
        }
        endUpdate(T2) {
          this._counter--, this._counter === 0 && (this._observable.reportChanges(), this._hasChanged && (this._hasChanged = false, this.emitter.fire(this._observable.get())));
        }
      }
      function di(y3, T2) {
        return new Ke(y3, T2).emitter.event;
      }
      Qe3.fromObservable = di;
      function V5(y3) {
        return (T2, g3, w3) => {
          let E2 = 0, x2 = false, N3 = { beginUpdate() {
            E2++;
          }, endUpdate() {
            E2--, E2 === 0 && (y3.reportChanges(), x2 && (x2 = false, T2.call(g3)));
          }, handlePossibleChange() {
          }, handleChange() {
            x2 = true;
          } };
          y3.addObserver(N3), y3.reportChanges();
          let Z3 = { dispose() {
            y3.removeObserver(N3);
          } };
          return w3 instanceof Ee ? w3.add(Z3) : Array.isArray(w3) && w3.push(Z3), Z3;
        };
      }
      Qe3.fromObservableLight = V5;
    })($2 || ($2 = {}));
    Mt = class Mt2 {
      constructor(t3) {
        this.listenerCount = 0;
        this.invocationCount = 0;
        this.elapsedOverall = 0;
        this.durations = [];
        this.name = `${t3}_${Mt2._idPool++}`, Mt2.all.add(this);
      }
      start(t3) {
        this._stopWatch = new mr(), this.listenerCount = t3;
      }
      stop() {
        if (this._stopWatch) {
          let t3 = this._stopWatch.elapsed();
          this.durations.push(t3), this.elapsedOverall += t3, this.invocationCount += 1, this._stopWatch = void 0;
        }
      }
    };
    Mt.all = /* @__PURE__ */ new Set(), Mt._idPool = 0;
    $n = Mt;
    po = -1;
    br = class br2 {
      constructor(t3, e4, i12 = (br2._idPool++).toString(16).padStart(3, "0")) {
        this._errorHandler = t3;
        this.threshold = e4;
        this.name = i12;
        this._warnCountdown = 0;
      }
      dispose() {
        this._stacks?.clear();
      }
      check(t3, e4) {
        let i12 = this.threshold;
        if (i12 <= 0 || e4 < i12) return;
        this._stacks || (this._stacks = /* @__PURE__ */ new Map());
        let r7 = this._stacks.get(t3.value) || 0;
        if (this._stacks.set(t3.value, r7 + 1), this._warnCountdown -= 1, this._warnCountdown <= 0) {
          this._warnCountdown = i12 * 0.5;
          let [n4, o6] = this.getMostFrequentStack(), l4 = `[${this.name}] potential listener LEAK detected, having ${e4} listeners already. MOST frequent listener (${o6}):`;
          console.warn(l4), console.warn(n4);
          let a3 = new qn(l4, n4);
          this._errorHandler(a3);
        }
        return () => {
          let n4 = this._stacks.get(t3.value) || 0;
          this._stacks.set(t3.value, n4 - 1);
        };
      }
      getMostFrequentStack() {
        if (!this._stacks) return;
        let t3, e4 = 0;
        for (let [i12, r7] of this._stacks) (!t3 || e4 < r7) && (t3 = [i12, r7], e4 = r7);
        return t3;
      }
    };
    br._idPool = 1;
    Vn = br;
    gi = class s10 {
      constructor(t3) {
        this.value = t3;
      }
      static create() {
        let t3 = new Error();
        return new s10(t3.stack ?? "");
      }
      print() {
        console.warn(this.value.split(`
`).slice(2).join(`
`));
      }
    };
    qn = class extends Error {
      constructor(t3, e4) {
        super(t3), this.name = "ListenerLeakError", this.stack = e4;
      }
    };
    Yn = class extends Error {
      constructor(t3, e4) {
        super(t3), this.name = "ListenerRefusalError", this.stack = e4;
      }
    };
    Vl = 0;
    Pt = class {
      constructor(t3) {
        this.value = t3;
        this.id = Vl++;
      }
    };
    ql = 2;
    Yl = (s18, t3) => {
      if (s18 instanceof Pt) t3(s18);
      else for (let e4 = 0; e4 < s18.length; e4++) {
        let i12 = s18[e4];
        i12 && t3(i12);
      }
    };
    if (Gl) {
      let s18 = [];
      setInterval(() => {
        s18.length !== 0 && (console.warn("[LEAKING LISTENERS] GC'ed these listeners that were NOT yet disposed:"), console.warn(s18.join(`
`)), s18.length = 0);
      }, 3e3), _r = new FinalizationRegistry((t3) => {
        typeof t3 == "string" && s18.push(t3);
      });
    }
    v2 = class {
      constructor(t3) {
        this._size = 0;
        this._options = t3, this._leakageMon = po > 0 || this._options?.leakWarningThreshold ? new Vn(t3?.onListenerError ?? Lt, this._options?.leakWarningThreshold ?? po) : void 0, this._perfMon = this._options?._profName ? new $n(this._options._profName) : void 0, this._deliveryQueue = this._options?.deliveryQueue;
      }
      dispose() {
        if (!this._disposed) {
          if (this._disposed = true, this._deliveryQueue?.current === this && this._deliveryQueue.reset(), this._listeners) {
            if (fo) {
              let t3 = this._listeners;
              queueMicrotask(() => {
                Yl(t3, (e4) => e4.stack?.print());
              });
            }
            this._listeners = void 0, this._size = 0;
          }
          this._options?.onDidRemoveLastListener?.(), this._leakageMon?.dispose();
        }
      }
      get event() {
        return this._event ?? (this._event = (t3, e4, i12) => {
          if (this._leakageMon && this._size > this._leakageMon.threshold ** 2) {
            let a3 = `[${this._leakageMon.name}] REFUSES to accept new listeners because it exceeded its threshold by far (${this._size} vs ${this._leakageMon.threshold})`;
            console.warn(a3);
            let u3 = this._leakageMon.getMostFrequentStack() ?? ["UNKNOWN stack", -1], h4 = new Yn(`${a3}. HINT: Stack shows most frequent listener (${u3[1]}-times)`, u3[0]);
            return (this._options?.onListenerError || Lt)(h4), D2.None;
          }
          if (this._disposed) return D2.None;
          e4 && (t3 = t3.bind(e4));
          let r7 = new Pt(t3), n4, o6;
          this._leakageMon && this._size >= Math.ceil(this._leakageMon.threshold * 0.2) && (r7.stack = gi.create(), n4 = this._leakageMon.check(r7.stack, this._size + 1)), fo && (r7.stack = o6 ?? gi.create()), this._listeners ? this._listeners instanceof Pt ? (this._deliveryQueue ?? (this._deliveryQueue = new jn()), this._listeners = [this._listeners, r7]) : this._listeners.push(r7) : (this._options?.onWillAddFirstListener?.(this), this._listeners = r7, this._options?.onDidAddFirstListener?.(this)), this._size++;
          let l4 = C2(() => {
            _r?.unregister(l4), n4?.(), this._removeListener(r7);
          });
          if (i12 instanceof Ee ? i12.add(l4) : Array.isArray(i12) && i12.push(l4), _r) {
            let a3 = new Error().stack.split(`
`).slice(2, 3).join(`
`).trim(), u3 = /(file:|vscode-file:\/\/vscode-app)?(\/[^:]*:\d+:\d+)/.exec(a3);
            _r.register(l4, u3?.[2] ?? a3, l4);
          }
          return l4;
        }), this._event;
      }
      _removeListener(t3) {
        if (this._options?.onWillRemoveListener?.(this), !this._listeners) return;
        if (this._size === 1) {
          this._listeners = void 0, this._options?.onDidRemoveLastListener?.(this), this._size = 0;
          return;
        }
        let e4 = this._listeners, i12 = e4.indexOf(t3);
        if (i12 === -1) throw console.log("disposed?", this._disposed), console.log("size?", this._size), console.log("arr?", JSON.stringify(this._listeners)), new Error("Attempted to dispose unknown listener");
        this._size--, e4[i12] = void 0;
        let r7 = this._deliveryQueue.current === this;
        if (this._size * ql <= e4.length) {
          let n4 = 0;
          for (let o6 = 0; o6 < e4.length; o6++) e4[o6] ? e4[n4++] = e4[o6] : r7 && (this._deliveryQueue.end--, n4 < this._deliveryQueue.i && this._deliveryQueue.i--);
          e4.length = n4;
        }
      }
      _deliver(t3, e4) {
        if (!t3) return;
        let i12 = this._options?.onListenerError || Lt;
        if (!i12) {
          t3.value(e4);
          return;
        }
        try {
          t3.value(e4);
        } catch (r7) {
          i12(r7);
        }
      }
      _deliverQueue(t3) {
        let e4 = t3.current._listeners;
        for (; t3.i < t3.end; ) this._deliver(e4[t3.i++], t3.value);
        t3.reset();
      }
      fire(t3) {
        if (this._deliveryQueue?.current && (this._deliverQueue(this._deliveryQueue), this._perfMon?.stop()), this._perfMon?.start(this._size), this._listeners) if (this._listeners instanceof Pt) this._deliver(this._listeners, t3);
        else {
          let e4 = this._deliveryQueue;
          e4.enqueue(this, t3, this._listeners.length), this._deliverQueue(e4);
        }
        this._perfMon?.stop();
      }
      hasListeners() {
        return this._size > 0;
      }
    };
    jn = class {
      constructor() {
        this.i = -1;
        this.end = 0;
      }
      enqueue(t3, e4, i12) {
        this.i = 0, this.end = i12, this.current = t3, this.value = e4;
      }
      reset() {
        this.i = this.end, this.current = void 0, this.value = void 0;
      }
    };
    gr = class gr2 {
      constructor() {
        this.mapWindowIdToZoomLevel = /* @__PURE__ */ new Map();
        this._onDidChangeZoomLevel = new v2();
        this.onDidChangeZoomLevel = this._onDidChangeZoomLevel.event;
        this.mapWindowIdToZoomFactor = /* @__PURE__ */ new Map();
        this._onDidChangeFullscreen = new v2();
        this.onDidChangeFullscreen = this._onDidChangeFullscreen.event;
        this.mapWindowIdToFullScreen = /* @__PURE__ */ new Map();
      }
      getZoomLevel(t3) {
        return this.mapWindowIdToZoomLevel.get(this.getWindowId(t3)) ?? 0;
      }
      setZoomLevel(t3, e4) {
        if (this.getZoomLevel(e4) === t3) return;
        let i12 = this.getWindowId(e4);
        this.mapWindowIdToZoomLevel.set(i12, t3), this._onDidChangeZoomLevel.fire(i12);
      }
      getZoomFactor(t3) {
        return this.mapWindowIdToZoomFactor.get(this.getWindowId(t3)) ?? 1;
      }
      setZoomFactor(t3, e4) {
        this.mapWindowIdToZoomFactor.set(this.getWindowId(e4), t3);
      }
      setFullscreen(t3, e4) {
        if (this.isFullscreen(e4) === t3) return;
        let i12 = this.getWindowId(e4);
        this.mapWindowIdToFullScreen.set(i12, t3), this._onDidChangeFullscreen.fire(i12);
      }
      isFullscreen(t3) {
        return !!this.mapWindowIdToFullScreen.get(this.getWindowId(t3));
      }
      getWindowId(t3) {
        return t3.vscodeWindowId;
      }
    };
    gr.INSTANCE = new gr();
    Si = gr;
    Eu = Si.INSTANCE.onDidChangeZoomLevel;
    Tu = Si.INSTANCE.onDidChangeFullscreen;
    Ot = typeof navigator == "object" ? navigator.userAgent : "";
    Ei = Ot.indexOf("Firefox") >= 0;
    Bt = Ot.indexOf("AppleWebKit") >= 0;
    Ti = Ot.indexOf("Chrome") >= 0;
    Sr = !Ti && Ot.indexOf("Safari") >= 0;
    Iu = Ot.indexOf("Electron/") >= 0;
    yu = Ot.indexOf("Android") >= 0;
    vr = false;
    if (typeof fe.matchMedia == "function") {
      let s18 = fe.matchMedia("(display-mode: standalone) or (display-mode: window-controls-overlay)"), t3 = fe.matchMedia("(display-mode: fullscreen)");
      vr = s18.matches, Xl(fe, s18, ({ matches: e4 }) => {
        vr && t3.matches || (vr = e4);
      });
    }
    Nt = "en";
    yr = false;
    xr = false;
    Ii = false;
    Zl = false;
    vo = false;
    go = false;
    Jl = false;
    Ql = false;
    ea = false;
    ta = false;
    Ir = Nt;
    bo = Nt;
    Ve = globalThis;
    typeof Ve.vscode < "u" && typeof Ve.vscode.process < "u" ? xe = Ve.vscode.process : typeof process < "u" && typeof process?.versions?.node == "string" && (xe = process);
    So = typeof xe?.versions?.electron == "string";
    ra = So && xe?.type === "renderer";
    if (typeof xe == "object") {
      yr = xe.platform === "win32", xr = xe.platform === "darwin", Ii = xe.platform === "linux", Zl = Ii && !!xe.env.SNAP && !!xe.env.SNAP_REVISION, Jl = So, ea = !!xe.env.CI || !!xe.env.BUILD_ARTIFACTSTAGINGDIRECTORY, Tr = Nt, Ir = Nt;
      let s18 = xe.env.VSCODE_NLS_CONFIG;
      if (s18) try {
        let t3 = JSON.parse(s18);
        Tr = t3.userLocale, bo = t3.osLocale, Ir = t3.resolvedLanguage || Nt, ia = t3.languagePack?.translationsConfigFile;
      } catch {
      }
      vo = true;
    } else typeof navigator == "object" && !ra ? ($e = navigator.userAgent, yr = $e.indexOf("Windows") >= 0, xr = $e.indexOf("Macintosh") >= 0, Ql = ($e.indexOf("Macintosh") >= 0 || $e.indexOf("iPad") >= 0 || $e.indexOf("iPhone") >= 0) && !!navigator.maxTouchPoints && navigator.maxTouchPoints > 0, Ii = $e.indexOf("Linux") >= 0, ta = $e?.indexOf("Mobi") >= 0, go = true, Ir = globalThis._VSCODE_NLS_LANGUAGE || Nt, Tr = navigator.language.toLowerCase(), bo = Tr) : console.error("Unable to resolve platform.");
    Xn = 0;
    xr ? Xn = 1 : yr ? Xn = 3 : Ii && (Xn = 2);
    wr = yr;
    Te = xr;
    Zn = Ii;
    Dr = vo;
    na = go && typeof Ve.importScripts == "function";
    xu = na ? Ve.origin : void 0;
    Fe = $e;
    st = Ir;
    ((i12) => {
      function s18() {
        return st;
      }
      i12.value = s18;
      function t3() {
        return st.length === 2 ? st === "en" : st.length >= 3 ? st[0] === "e" && st[1] === "n" && st[2] === "-" : false;
      }
      i12.isDefaultVariant = t3;
      function e4() {
        return st === "en";
      }
      i12.isDefault = e4;
    })(sa || (sa = {}));
    oa = typeof Ve.postMessage == "function" && !Ve.importScripts;
    Eo = (() => {
      if (oa) {
        let s18 = [];
        Ve.addEventListener("message", (e4) => {
          if (e4.data && e4.data.vscodeScheduleAsyncWork) for (let i12 = 0, r7 = s18.length; i12 < r7; i12++) {
            let n4 = s18[i12];
            if (n4.id === e4.data.vscodeScheduleAsyncWork) {
              s18.splice(i12, 1), n4.callback();
              return;
            }
          }
        });
        let t3 = 0;
        return (e4) => {
          let i12 = ++t3;
          s18.push({ id: i12, callback: e4 }), Ve.postMessage({ vscodeScheduleAsyncWork: i12 }, "*");
        };
      }
      return (s18) => setTimeout(s18);
    })();
    la = !!(Fe && Fe.indexOf("Chrome") >= 0);
    wu = !!(Fe && Fe.indexOf("Firefox") >= 0);
    Du = !!(!la && Fe && Fe.indexOf("Safari") >= 0);
    Ru = !!(Fe && Fe.indexOf("Edg/") >= 0);
    Lu = !!(Fe && Fe.indexOf("Android") >= 0);
    ot = typeof navigator == "object" ? navigator : {};
    aa = { clipboard: { writeText: Dr || document.queryCommandSupported && document.queryCommandSupported("copy") || !!(ot && ot.clipboard && ot.clipboard.writeText), readText: Dr || !!(ot && ot.clipboard && ot.clipboard.readText) }, keyboard: Dr || _o() ? 0 : ot.keyboard || Sr ? 1 : 2, touch: "ontouchstart" in fe || ot.maxTouchPoints > 0, pointerEvents: fe.PointerEvent && ("ontouchstart" in fe || navigator.maxTouchPoints > 0) };
    yi = class {
      constructor() {
        this._keyCodeToStr = [], this._strToKeyCode = /* @__PURE__ */ Object.create(null);
      }
      define(t3, e4) {
        this._keyCodeToStr[t3] = e4, this._strToKeyCode[e4.toLowerCase()] = t3;
      }
      keyCodeToStr(t3) {
        return this._keyCodeToStr[t3];
      }
      strToKeyCode(t3) {
        return this._strToKeyCode[t3.toLowerCase()] || 0;
      }
    };
    Jn = new yi();
    To = new yi();
    Io = new yi();
    yo = new Array(230);
    ((o6) => {
      function s18(l4) {
        return Jn.keyCodeToStr(l4);
      }
      o6.toString = s18;
      function t3(l4) {
        return Jn.strToKeyCode(l4);
      }
      o6.fromString = t3;
      function e4(l4) {
        return To.keyCodeToStr(l4);
      }
      o6.toUserSettingsUS = e4;
      function i12(l4) {
        return Io.keyCodeToStr(l4);
      }
      o6.toUserSettingsGeneral = i12;
      function r7(l4) {
        return To.strToKeyCode(l4) || Io.strToKeyCode(l4);
      }
      o6.fromUserSettings = r7;
      function n4(l4) {
        if (l4 >= 98 && l4 <= 113) return null;
        switch (l4) {
          case 16:
            return "Up";
          case 18:
            return "Down";
          case 15:
            return "Left";
          case 17:
            return "Right";
        }
        return Jn.keyCodeToStr(l4);
      }
      o6.toElectronAccelerator = n4;
    })(Qn || (Qn = {}));
    Rr = class s11 {
      constructor(t3, e4, i12, r7, n4) {
        this.ctrlKey = t3;
        this.shiftKey = e4;
        this.altKey = i12;
        this.metaKey = r7;
        this.keyCode = n4;
      }
      equals(t3) {
        return t3 instanceof s11 && this.ctrlKey === t3.ctrlKey && this.shiftKey === t3.shiftKey && this.altKey === t3.altKey && this.metaKey === t3.metaKey && this.keyCode === t3.keyCode;
      }
      getHashCode() {
        let t3 = this.ctrlKey ? "1" : "0", e4 = this.shiftKey ? "1" : "0", i12 = this.altKey ? "1" : "0", r7 = this.metaKey ? "1" : "0";
        return `K${t3}${e4}${i12}${r7}${this.keyCode}`;
      }
      isModifierKey() {
        return this.keyCode === 0 || this.keyCode === 5 || this.keyCode === 57 || this.keyCode === 6 || this.keyCode === 4;
      }
      toKeybinding() {
        return new es([this]);
      }
      isDuplicateModifierCase() {
        return this.ctrlKey && this.keyCode === 5 || this.shiftKey && this.keyCode === 4 || this.altKey && this.keyCode === 6 || this.metaKey && this.keyCode === 57;
      }
    };
    es = class {
      constructor(t3) {
        if (t3.length === 0) throw eo("chords");
        this.chords = t3;
      }
      getHashCode() {
        let t3 = "";
        for (let e4 = 0, i12 = this.chords.length; e4 < i12; e4++) e4 !== 0 && (t3 += ";"), t3 += this.chords[e4].getHashCode();
        return t3;
      }
      equals(t3) {
        if (t3 === null || this.chords.length !== t3.chords.length) return false;
        for (let e4 = 0; e4 < this.chords.length; e4++) if (!this.chords[e4].equals(t3.chords[e4])) return false;
        return true;
      }
    };
    ua = Te ? 256 : 2048;
    ha = 512;
    da = 1024;
    fa = Te ? 2048 : 256;
    ft = class {
      constructor(t3) {
        this._standardKeyboardEventBrand = true;
        let e4 = t3;
        this.browserEvent = e4, this.target = e4.target, this.ctrlKey = e4.ctrlKey, this.shiftKey = e4.shiftKey, this.altKey = e4.altKey, this.metaKey = e4.metaKey, this.altGraphKey = e4.getModifierState?.("AltGraph"), this.keyCode = ca(e4), this.code = e4.code, this.ctrlKey = this.ctrlKey || this.keyCode === 5, this.altKey = this.altKey || this.keyCode === 6, this.shiftKey = this.shiftKey || this.keyCode === 4, this.metaKey = this.metaKey || this.keyCode === 57, this._asKeybinding = this._computeKeybinding(), this._asKeyCodeChord = this._computeKeyCodeChord();
      }
      preventDefault() {
        this.browserEvent && this.browserEvent.preventDefault && this.browserEvent.preventDefault();
      }
      stopPropagation() {
        this.browserEvent && this.browserEvent.stopPropagation && this.browserEvent.stopPropagation();
      }
      toKeyCodeChord() {
        return this._asKeyCodeChord;
      }
      equals(t3) {
        return this._asKeybinding === t3;
      }
      _computeKeybinding() {
        let t3 = 0;
        this.keyCode !== 5 && this.keyCode !== 4 && this.keyCode !== 6 && this.keyCode !== 57 && (t3 = this.keyCode);
        let e4 = 0;
        return this.ctrlKey && (e4 |= ua), this.altKey && (e4 |= ha), this.shiftKey && (e4 |= da), this.metaKey && (e4 |= fa), e4 |= t3, e4;
      }
      _computeKeyCodeChord() {
        let t3 = 0;
        return this.keyCode !== 5 && this.keyCode !== 4 && this.keyCode !== 6 && this.keyCode !== 57 && (t3 = this.keyCode), new Rr(this.ctrlKey, this.shiftKey, this.altKey, this.metaKey, t3);
      }
    };
    wo = /* @__PURE__ */ new WeakMap();
    Lr = class {
      static getSameOriginWindowChain(t3) {
        let e4 = wo.get(t3);
        if (!e4) {
          e4 = [], wo.set(t3, e4);
          let i12 = t3, r7;
          do
            r7 = pa(i12), r7 ? e4.push({ window: new WeakRef(i12), iframeElement: i12.frameElement || null }) : e4.push({ window: new WeakRef(i12), iframeElement: null }), i12 = r7;
          while (i12);
        }
        return e4.slice(0);
      }
      static getPositionOfChildWindowRelativeToAncestorWindow(t3, e4) {
        if (!e4 || t3 === e4) return { top: 0, left: 0 };
        let i12 = 0, r7 = 0, n4 = this.getSameOriginWindowChain(t3);
        for (let o6 of n4) {
          let l4 = o6.window.deref();
          if (i12 += l4?.scrollY ?? 0, r7 += l4?.scrollX ?? 0, l4 === e4 || !o6.iframeElement) break;
          let a3 = o6.iframeElement.getBoundingClientRect();
          i12 += a3.top, r7 += a3.left;
        }
        return { top: i12, left: r7 };
      }
    };
    qe = class {
      constructor(t3, e4) {
        this.timestamp = Date.now(), this.browserEvent = e4, this.leftButton = e4.button === 0, this.middleButton = e4.button === 1, this.rightButton = e4.button === 2, this.buttons = e4.buttons, this.target = e4.target, this.detail = e4.detail || 1, e4.type === "dblclick" && (this.detail = 2), this.ctrlKey = e4.ctrlKey, this.shiftKey = e4.shiftKey, this.altKey = e4.altKey, this.metaKey = e4.metaKey, typeof e4.pageX == "number" ? (this.posx = e4.pageX, this.posy = e4.pageY) : (this.posx = e4.clientX + this.target.ownerDocument.body.scrollLeft + this.target.ownerDocument.documentElement.scrollLeft, this.posy = e4.clientY + this.target.ownerDocument.body.scrollTop + this.target.ownerDocument.documentElement.scrollTop);
        let i12 = Lr.getPositionOfChildWindowRelativeToAncestorWindow(t3, e4.view);
        this.posx -= i12.left, this.posy -= i12.top;
      }
      preventDefault() {
        this.browserEvent.preventDefault();
      }
      stopPropagation() {
        this.browserEvent.stopPropagation();
      }
    };
    xi = class {
      constructor(t3, e4 = 0, i12 = 0) {
        this.browserEvent = t3 || null, this.target = t3 ? t3.target || t3.targetNode || t3.srcElement : null, this.deltaY = i12, this.deltaX = e4;
        let r7 = false;
        if (Ti) {
          let n4 = navigator.userAgent.match(/Chrome\/(\d+)/);
          r7 = (n4 ? parseInt(n4[1]) : 123) <= 122;
        }
        if (t3) {
          let n4 = t3, o6 = t3, l4 = t3.view?.devicePixelRatio || 1;
          if (typeof n4.wheelDeltaY < "u") r7 ? this.deltaY = n4.wheelDeltaY / (120 * l4) : this.deltaY = n4.wheelDeltaY / 120;
          else if (typeof o6.VERTICAL_AXIS < "u" && o6.axis === o6.VERTICAL_AXIS) this.deltaY = -o6.detail / 3;
          else if (t3.type === "wheel") {
            let a3 = t3;
            a3.deltaMode === a3.DOM_DELTA_LINE ? Ei && !Te ? this.deltaY = -t3.deltaY / 3 : this.deltaY = -t3.deltaY : this.deltaY = -t3.deltaY / 40;
          }
          if (typeof n4.wheelDeltaX < "u") Sr && wr ? this.deltaX = -(n4.wheelDeltaX / 120) : r7 ? this.deltaX = n4.wheelDeltaX / (120 * l4) : this.deltaX = n4.wheelDeltaX / 120;
          else if (typeof o6.HORIZONTAL_AXIS < "u" && o6.axis === o6.HORIZONTAL_AXIS) this.deltaX = -t3.detail / 3;
          else if (t3.type === "wheel") {
            let a3 = t3;
            a3.deltaMode === a3.DOM_DELTA_LINE ? Ei && !Te ? this.deltaX = -t3.deltaX / 3 : this.deltaX = -t3.deltaX : this.deltaX = -t3.deltaX / 40;
          }
          this.deltaY === 0 && this.deltaX === 0 && t3.wheelDelta && (r7 ? this.deltaY = t3.wheelDelta / (120 * l4) : this.deltaY = t3.wheelDelta / 120);
        }
      }
      preventDefault() {
        this.browserEvent?.preventDefault();
      }
      stopPropagation() {
        this.browserEvent?.stopPropagation();
      }
    };
    Do = Object.freeze(function(s18, t3) {
      let e4 = setTimeout(s18.bind(t3), 0);
      return { dispose() {
        clearTimeout(e4);
      } };
    });
    ((i12) => {
      function s18(r7) {
        return r7 === i12.None || r7 === i12.Cancelled || r7 instanceof ts ? true : !r7 || typeof r7 != "object" ? false : typeof r7.isCancellationRequested == "boolean" && typeof r7.onCancellationRequested == "function";
      }
      i12.isCancellationToken = s18, i12.None = Object.freeze({ isCancellationRequested: false, onCancellationRequested: $2.None }), i12.Cancelled = Object.freeze({ isCancellationRequested: true, onCancellationRequested: Do });
    })(ma || (ma = {}));
    ts = class {
      constructor() {
        this._isCancelled = false;
        this._emitter = null;
      }
      cancel() {
        this._isCancelled || (this._isCancelled = true, this._emitter && (this._emitter.fire(void 0), this.dispose()));
      }
      get isCancellationRequested() {
        return this._isCancelled;
      }
      get onCancellationRequested() {
        return this._isCancelled ? Do : (this._emitter || (this._emitter = new v2()), this._emitter.event);
      }
      dispose() {
        this._emitter && (this._emitter.dispose(), this._emitter = null);
      }
    };
    _a = Symbol("MicrotaskDelay");
    Ye = class {
      constructor(t3, e4) {
        this._isDisposed = false;
        this._token = -1, typeof t3 == "function" && typeof e4 == "number" && this.setIfNotSet(t3, e4);
      }
      dispose() {
        this.cancel(), this._isDisposed = true;
      }
      cancel() {
        this._token !== -1 && (clearTimeout(this._token), this._token = -1);
      }
      cancelAndSet(t3, e4) {
        if (this._isDisposed) throw new Rt("Calling 'cancelAndSet' on a disposed TimeoutTimer");
        this.cancel(), this._token = setTimeout(() => {
          this._token = -1, t3();
        }, e4);
      }
      setIfNotSet(t3, e4) {
        if (this._isDisposed) throw new Rt("Calling 'setIfNotSet' on a disposed TimeoutTimer");
        this._token === -1 && (this._token = setTimeout(() => {
          this._token = -1, t3();
        }, e4));
      }
    };
    kr = class {
      constructor() {
        this.disposable = void 0;
        this.isDisposed = false;
      }
      cancel() {
        this.disposable?.dispose(), this.disposable = void 0;
      }
      cancelAndSet(t3, e4, i12 = globalThis) {
        if (this.isDisposed) throw new Rt("Calling 'cancelAndSet' on a disposed IntervalTimer");
        this.cancel();
        let r7 = i12.setInterval(() => {
          t3();
        }, e4);
        this.disposable = C2(() => {
          i12.clearInterval(r7), this.disposable = void 0;
        });
      }
      dispose() {
        this.cancel(), this.isDisposed = true;
      }
    };
    (function() {
      typeof globalThis.requestIdleCallback != "function" || typeof globalThis.cancelIdleCallback != "function" ? Ar = (s18, t3) => {
        Eo(() => {
          if (e4) return;
          let i12 = Date.now() + 15;
          t3(Object.freeze({ didTimeout: true, timeRemaining() {
            return Math.max(0, i12 - Date.now());
          } }));
        });
        let e4 = false;
        return { dispose() {
          e4 || (e4 = true);
        } };
      } : Ar = (s18, t3, e4) => {
        let i12 = s18.requestIdleCallback(t3, typeof e4 == "number" ? { timeout: e4 } : void 0), r7 = false;
        return { dispose() {
          r7 || (r7 = true, s18.cancelIdleCallback(i12));
        } };
      }, ba = (s18) => Ar(globalThis, s18);
    })();
    ((e4) => {
      async function s18(i12) {
        let r7, n4 = await Promise.all(i12.map((o6) => o6.then((l4) => l4, (l4) => {
          r7 || (r7 = l4);
        })));
        if (typeof r7 < "u") throw r7;
        return n4;
      }
      e4.settled = s18;
      function t3(i12) {
        return new Promise(async (r7, n4) => {
          try {
            await i12(r7, n4);
          } catch (o6) {
            n4(o6);
          }
        });
      }
      e4.withAsyncBody = t3;
    })(va || (va = {}));
    _e = class _e2 {
      static fromArray(t3) {
        return new _e2((e4) => {
          e4.emitMany(t3);
        });
      }
      static fromPromise(t3) {
        return new _e2(async (e4) => {
          e4.emitMany(await t3);
        });
      }
      static fromPromises(t3) {
        return new _e2(async (e4) => {
          await Promise.all(t3.map(async (i12) => e4.emitOne(await i12)));
        });
      }
      static merge(t3) {
        return new _e2(async (e4) => {
          await Promise.all(t3.map(async (i12) => {
            for await (let r7 of i12) e4.emitOne(r7);
          }));
        });
      }
      constructor(t3, e4) {
        this._state = 0, this._results = [], this._error = null, this._onReturn = e4, this._onStateChanged = new v2(), queueMicrotask(async () => {
          let i12 = { emitOne: (r7) => this.emitOne(r7), emitMany: (r7) => this.emitMany(r7), reject: (r7) => this.reject(r7) };
          try {
            await Promise.resolve(t3(i12)), this.resolve();
          } catch (r7) {
            this.reject(r7);
          } finally {
            i12.emitOne = void 0, i12.emitMany = void 0, i12.reject = void 0;
          }
        });
      }
      [Symbol.asyncIterator]() {
        let t3 = 0;
        return { next: async () => {
          do {
            if (this._state === 2) throw this._error;
            if (t3 < this._results.length) return { done: false, value: this._results[t3++] };
            if (this._state === 1) return { done: true, value: void 0 };
            await $2.toPromise(this._onStateChanged.event);
          } while (true);
        }, return: async () => (this._onReturn?.(), { done: true, value: void 0 }) };
      }
      static map(t3, e4) {
        return new _e2(async (i12) => {
          for await (let r7 of t3) i12.emitOne(e4(r7));
        });
      }
      map(t3) {
        return _e2.map(this, t3);
      }
      static filter(t3, e4) {
        return new _e2(async (i12) => {
          for await (let r7 of t3) e4(r7) && i12.emitOne(r7);
        });
      }
      filter(t3) {
        return _e2.filter(this, t3);
      }
      static coalesce(t3) {
        return _e2.filter(t3, (e4) => !!e4);
      }
      coalesce() {
        return _e2.coalesce(this);
      }
      static async toPromise(t3) {
        let e4 = [];
        for await (let i12 of t3) e4.push(i12);
        return e4;
      }
      toPromise() {
        return _e2.toPromise(this);
      }
      emitOne(t3) {
        this._state === 0 && (this._results.push(t3), this._onStateChanged.fire());
      }
      emitMany(t3) {
        this._state === 0 && (this._results = this._results.concat(t3), this._onStateChanged.fire());
      }
      resolve() {
        this._state === 0 && (this._state = 1, this._onStateChanged.fire());
      }
      reject(t3) {
        this._state === 0 && (this._state = 2, this._error = t3, this._onStateChanged.fire());
      }
    };
    _e.EMPTY = _e.fromArray([]);
    Cr = class Cr2 {
      constructor() {
        this._h0 = 1732584193;
        this._h1 = 4023233417;
        this._h2 = 2562383102;
        this._h3 = 271733878;
        this._h4 = 3285377520;
        this._buff = new Uint8Array(67), this._buffDV = new DataView(this._buff.buffer), this._buffLen = 0, this._totalLen = 0, this._leftoverHighSurrogate = 0, this._finished = false;
      }
      update(t3) {
        let e4 = t3.length;
        if (e4 === 0) return;
        let i12 = this._buff, r7 = this._buffLen, n4 = this._leftoverHighSurrogate, o6, l4;
        for (n4 !== 0 ? (o6 = n4, l4 = -1, n4 = 0) : (o6 = t3.charCodeAt(0), l4 = 0); ; ) {
          let a3 = o6;
          if (Lo(o6)) if (l4 + 1 < e4) {
            let u3 = t3.charCodeAt(l4 + 1);
            is(u3) ? (l4++, a3 = Ao(o6, u3)) : a3 = 65533;
          } else {
            n4 = o6;
            break;
          }
          else is(o6) && (a3 = 65533);
          if (r7 = this._push(i12, r7, a3), l4++, l4 < e4) o6 = t3.charCodeAt(l4);
          else break;
        }
        this._buffLen = r7, this._leftoverHighSurrogate = n4;
      }
      _push(t3, e4, i12) {
        return i12 < 128 ? t3[e4++] = i12 : i12 < 2048 ? (t3[e4++] = 192 | (i12 & 1984) >>> 6, t3[e4++] = 128 | (i12 & 63) >>> 0) : i12 < 65536 ? (t3[e4++] = 224 | (i12 & 61440) >>> 12, t3[e4++] = 128 | (i12 & 4032) >>> 6, t3[e4++] = 128 | (i12 & 63) >>> 0) : (t3[e4++] = 240 | (i12 & 1835008) >>> 18, t3[e4++] = 128 | (i12 & 258048) >>> 12, t3[e4++] = 128 | (i12 & 4032) >>> 6, t3[e4++] = 128 | (i12 & 63) >>> 0), e4 >= 64 && (this._step(), e4 -= 64, this._totalLen += 64, t3[0] = t3[64], t3[1] = t3[65], t3[2] = t3[66]), e4;
      }
      digest() {
        return this._finished || (this._finished = true, this._leftoverHighSurrogate && (this._leftoverHighSurrogate = 0, this._buffLen = this._push(this._buff, this._buffLen, 65533)), this._totalLen += this._buffLen, this._wrapUp()), wi(this._h0) + wi(this._h1) + wi(this._h2) + wi(this._h3) + wi(this._h4);
      }
      _wrapUp() {
        this._buff[this._buffLen++] = 128, ko(this._buff, this._buffLen), this._buffLen > 56 && (this._step(), ko(this._buff));
        let t3 = 8 * this._totalLen;
        this._buffDV.setUint32(56, Math.floor(t3 / 4294967296), false), this._buffDV.setUint32(60, t3 % 4294967296, false), this._step();
      }
      _step() {
        let t3 = Cr2._bigBlock32, e4 = this._buffDV;
        for (let c4 = 0; c4 < 64; c4 += 4) t3.setUint32(c4, e4.getUint32(c4, false), false);
        for (let c4 = 64; c4 < 320; c4 += 4) t3.setUint32(c4, rs(t3.getUint32(c4 - 12, false) ^ t3.getUint32(c4 - 32, false) ^ t3.getUint32(c4 - 56, false) ^ t3.getUint32(c4 - 64, false), 1), false);
        let i12 = this._h0, r7 = this._h1, n4 = this._h2, o6 = this._h3, l4 = this._h4, a3, u3, h4;
        for (let c4 = 0; c4 < 80; c4++) c4 < 20 ? (a3 = r7 & n4 | ~r7 & o6, u3 = 1518500249) : c4 < 40 ? (a3 = r7 ^ n4 ^ o6, u3 = 1859775393) : c4 < 60 ? (a3 = r7 & n4 | r7 & o6 | n4 & o6, u3 = 2400959708) : (a3 = r7 ^ n4 ^ o6, u3 = 3395469782), h4 = rs(i12, 5) + a3 + l4 + u3 + t3.getUint32(c4 * 4, false) & 4294967295, l4 = o6, o6 = n4, n4 = rs(r7, 30), r7 = i12, i12 = h4;
        this._h0 = this._h0 + i12 & 4294967295, this._h1 = this._h1 + r7 & 4294967295, this._h2 = this._h2 + n4 & 4294967295, this._h3 = this._h3 + o6 & 4294967295, this._h4 = this._h4 + l4 & 4294967295;
      }
    };
    Cr._bigBlock32 = new DataView(new ArrayBuffer(320));
    ({ registerWindow: Bh, getWindow: be, getDocument: Nh, getWindows: Fh, getWindowsCount: Hh, getWindowId: Oo, getWindowById: Wh, hasWindow: Uh, onDidRegisterWindow: No, onWillUnregisterWindow: Kh, onDidUnregisterWindow: zh } = (function() {
      let s18 = /* @__PURE__ */ new Map();
      fe;
      let t3 = { window: fe, disposables: new Ee() };
      s18.set(fe.vscodeWindowId, t3);
      let e4 = new v2(), i12 = new v2(), r7 = new v2();
      function n4(o6, l4) {
        return (typeof o6 == "number" ? s18.get(o6) : void 0) ?? (l4 ? t3 : void 0);
      }
      return { onDidRegisterWindow: e4.event, onWillUnregisterWindow: r7.event, onDidUnregisterWindow: i12.event, registerWindow(o6) {
        if (s18.has(o6.vscodeWindowId)) return D2.None;
        let l4 = new Ee(), a3 = { window: o6, disposables: l4.add(new Ee()) };
        return s18.set(o6.vscodeWindowId, a3), l4.add(C2(() => {
          s18.delete(o6.vscodeWindowId), i12.fire(o6);
        })), l4.add(L2(o6, Y.BEFORE_UNLOAD, () => {
          r7.fire(o6);
        })), e4.fire(a3), l4;
      }, getWindows() {
        return s18.values();
      }, getWindowsCount() {
        return s18.size;
      }, getWindowId(o6) {
        return o6.vscodeWindowId;
      }, hasWindow(o6) {
        return s18.has(o6);
      }, getWindowById: n4, getWindow(o6) {
        let l4 = o6;
        if (l4?.ownerDocument?.defaultView) return l4.ownerDocument.defaultView.window;
        let a3 = o6;
        return a3?.view ? a3.view.window : fe;
      }, getDocument(o6) {
        return be(o6).document;
      } };
    })());
    ss = class {
      constructor(t3, e4, i12, r7) {
        this._node = t3, this._type = e4, this._handler = i12, this._options = r7 || false, this._node.addEventListener(this._type, this._handler, this._options);
      }
      dispose() {
        this._handler && (this._node.removeEventListener(this._type, this._handler, this._options), this._node = null, this._handler = null);
      }
    };
    os = function(t3, e4, i12, r7) {
      let n4 = i12;
      return e4 === "click" || e4 === "mousedown" || e4 === "contextmenu" ? n4 = ya(be(t3), i12) : (e4 === "keydown" || e4 === "keypress" || e4 === "keyup") && (n4 = xa(i12)), L2(t3, e4, n4, r7);
    };
    Mr = class extends kr {
      constructor(t3) {
        super(), this.defaultTarget = t3 && be(t3);
      }
      cancelAndSet(t3, e4, i12) {
        return super.cancelAndSet(t3, e4, i12 ?? this.defaultTarget);
      }
    };
    Di = class {
      constructor(t3, e4 = 0) {
        this._runner = t3, this.priority = e4, this._canceled = false;
      }
      dispose() {
        this._canceled = true;
      }
      execute() {
        if (!this._canceled) try {
          this._runner();
        } catch (t3) {
          Lt(t3);
        }
      }
      static sort(t3, e4) {
        return e4.priority - t3.priority;
      }
    };
    (function() {
      let s18 = /* @__PURE__ */ new Map(), t3 = /* @__PURE__ */ new Map(), e4 = /* @__PURE__ */ new Map(), i12 = /* @__PURE__ */ new Map(), r7 = (n4) => {
        e4.set(n4, false);
        let o6 = s18.get(n4) ?? [];
        for (t3.set(n4, o6), s18.set(n4, []), i12.set(n4, true); o6.length > 0; ) o6.sort(Di.sort), o6.shift().execute();
        i12.set(n4, false);
      };
      mt = (n4, o6, l4 = 0) => {
        let a3 = Oo(n4), u3 = new Di(o6, l4), h4 = s18.get(a3);
        return h4 || (h4 = [], s18.set(a3, h4)), h4.push(u3), e4.get(a3) || (e4.set(a3, true), n4.requestAnimationFrame(() => r7(a3))), u3;
      }, wa = (n4, o6, l4) => {
        let a3 = Oo(n4);
        if (i12.get(a3)) {
          let u3 = new Di(o6, l4), h4 = t3.get(a3);
          return h4 || (h4 = [], t3.set(a3, h4)), h4.push(u3), u3;
        } else return mt(n4, o6, l4);
      };
    })();
    pt = class pt2 {
      constructor(t3, e4) {
        this.width = t3;
        this.height = e4;
      }
      with(t3 = this.width, e4 = this.height) {
        return t3 !== this.width || e4 !== this.height ? new pt2(t3, e4) : this;
      }
      static is(t3) {
        return typeof t3 == "object" && typeof t3.height == "number" && typeof t3.width == "number";
      }
      static lift(t3) {
        return t3 instanceof pt2 ? t3 : new pt2(t3.width, t3.height);
      }
      static equals(t3, e4) {
        return t3 === e4 ? true : !t3 || !e4 ? false : t3.width === e4.width && t3.height === e4.height;
      }
    };
    pt.None = new pt(0, 0);
    Gh = new class {
      constructor() {
        this.mutationObservers = /* @__PURE__ */ new Map();
      }
      observe(s18, t3, e4) {
        let i12 = this.mutationObservers.get(s18);
        i12 || (i12 = /* @__PURE__ */ new Map(), this.mutationObservers.set(s18, i12));
        let r7 = Mo(e4), n4 = i12.get(r7);
        if (n4) n4.users += 1;
        else {
          let o6 = new v2(), l4 = new MutationObserver((u3) => o6.fire(u3));
          l4.observe(s18, e4);
          let a3 = n4 = { users: 1, observer: l4, onDidMutate: o6.event };
          t3.add(C2(() => {
            a3.users -= 1, a3.users === 0 && (o6.dispose(), l4.disconnect(), i12?.delete(r7), i12?.size === 0 && this.mutationObservers.delete(s18));
          })), i12.set(r7, n4);
        }
        return n4.onDidMutate;
      }
    }();
    Y = { CLICK: "click", AUXCLICK: "auxclick", DBLCLICK: "dblclick", MOUSE_UP: "mouseup", MOUSE_DOWN: "mousedown", MOUSE_OVER: "mouseover", MOUSE_MOVE: "mousemove", MOUSE_OUT: "mouseout", MOUSE_ENTER: "mouseenter", MOUSE_LEAVE: "mouseleave", MOUSE_WHEEL: "wheel", POINTER_UP: "pointerup", POINTER_DOWN: "pointerdown", POINTER_MOVE: "pointermove", POINTER_LEAVE: "pointerleave", CONTEXT_MENU: "contextmenu", WHEEL: "wheel", KEY_DOWN: "keydown", KEY_PRESS: "keypress", KEY_UP: "keyup", LOAD: "load", BEFORE_UNLOAD: "beforeunload", UNLOAD: "unload", PAGE_SHOW: "pageshow", PAGE_HIDE: "pagehide", PASTE: "paste", ABORT: "abort", ERROR: "error", RESIZE: "resize", SCROLL: "scroll", FULLSCREEN_CHANGE: "fullscreenchange", WK_FULLSCREEN_CHANGE: "webkitfullscreenchange", SELECT: "select", CHANGE: "change", SUBMIT: "submit", RESET: "reset", FOCUS: "focus", FOCUS_IN: "focusin", FOCUS_OUT: "focusout", BLUR: "blur", INPUT: "input", STORAGE: "storage", DRAG_START: "dragstart", DRAG: "drag", DRAG_ENTER: "dragenter", DRAG_LEAVE: "dragleave", DRAG_OVER: "dragover", DROP: "drop", DRAG_END: "dragend", ANIMATION_START: Bt ? "webkitAnimationStart" : "animationstart", ANIMATION_END: Bt ? "webkitAnimationEnd" : "animationend", ANIMATION_ITERATION: Bt ? "webkitAnimationIteration" : "animationiteration" };
    Da = /([\w\-]+)?(#([\w\-]+))?((\.([\w\-]+))*)/;
    Ra.SVG = function(s18, t3, ...e4) {
      return Ho("http://www.w3.org/2000/svg", s18, t3, ...e4);
    };
    ls = class {
      constructor(t3) {
        this.domNode = t3;
        this._maxWidth = "";
        this._width = "";
        this._height = "";
        this._top = "";
        this._left = "";
        this._bottom = "";
        this._right = "";
        this._paddingTop = "";
        this._paddingLeft = "";
        this._paddingBottom = "";
        this._paddingRight = "";
        this._fontFamily = "";
        this._fontWeight = "";
        this._fontSize = "";
        this._fontStyle = "";
        this._fontFeatureSettings = "";
        this._fontVariationSettings = "";
        this._textDecoration = "";
        this._lineHeight = "";
        this._letterSpacing = "";
        this._className = "";
        this._display = "";
        this._position = "";
        this._visibility = "";
        this._color = "";
        this._backgroundColor = "";
        this._layerHint = false;
        this._contain = "none";
        this._boxShadow = "";
      }
      setMaxWidth(t3) {
        let e4 = Ie(t3);
        this._maxWidth !== e4 && (this._maxWidth = e4, this.domNode.style.maxWidth = this._maxWidth);
      }
      setWidth(t3) {
        let e4 = Ie(t3);
        this._width !== e4 && (this._width = e4, this.domNode.style.width = this._width);
      }
      setHeight(t3) {
        let e4 = Ie(t3);
        this._height !== e4 && (this._height = e4, this.domNode.style.height = this._height);
      }
      setTop(t3) {
        let e4 = Ie(t3);
        this._top !== e4 && (this._top = e4, this.domNode.style.top = this._top);
      }
      setLeft(t3) {
        let e4 = Ie(t3);
        this._left !== e4 && (this._left = e4, this.domNode.style.left = this._left);
      }
      setBottom(t3) {
        let e4 = Ie(t3);
        this._bottom !== e4 && (this._bottom = e4, this.domNode.style.bottom = this._bottom);
      }
      setRight(t3) {
        let e4 = Ie(t3);
        this._right !== e4 && (this._right = e4, this.domNode.style.right = this._right);
      }
      setPaddingTop(t3) {
        let e4 = Ie(t3);
        this._paddingTop !== e4 && (this._paddingTop = e4, this.domNode.style.paddingTop = this._paddingTop);
      }
      setPaddingLeft(t3) {
        let e4 = Ie(t3);
        this._paddingLeft !== e4 && (this._paddingLeft = e4, this.domNode.style.paddingLeft = this._paddingLeft);
      }
      setPaddingBottom(t3) {
        let e4 = Ie(t3);
        this._paddingBottom !== e4 && (this._paddingBottom = e4, this.domNode.style.paddingBottom = this._paddingBottom);
      }
      setPaddingRight(t3) {
        let e4 = Ie(t3);
        this._paddingRight !== e4 && (this._paddingRight = e4, this.domNode.style.paddingRight = this._paddingRight);
      }
      setFontFamily(t3) {
        this._fontFamily !== t3 && (this._fontFamily = t3, this.domNode.style.fontFamily = this._fontFamily);
      }
      setFontWeight(t3) {
        this._fontWeight !== t3 && (this._fontWeight = t3, this.domNode.style.fontWeight = this._fontWeight);
      }
      setFontSize(t3) {
        let e4 = Ie(t3);
        this._fontSize !== e4 && (this._fontSize = e4, this.domNode.style.fontSize = this._fontSize);
      }
      setFontStyle(t3) {
        this._fontStyle !== t3 && (this._fontStyle = t3, this.domNode.style.fontStyle = this._fontStyle);
      }
      setFontFeatureSettings(t3) {
        this._fontFeatureSettings !== t3 && (this._fontFeatureSettings = t3, this.domNode.style.fontFeatureSettings = this._fontFeatureSettings);
      }
      setFontVariationSettings(t3) {
        this._fontVariationSettings !== t3 && (this._fontVariationSettings = t3, this.domNode.style.fontVariationSettings = this._fontVariationSettings);
      }
      setTextDecoration(t3) {
        this._textDecoration !== t3 && (this._textDecoration = t3, this.domNode.style.textDecoration = this._textDecoration);
      }
      setLineHeight(t3) {
        let e4 = Ie(t3);
        this._lineHeight !== e4 && (this._lineHeight = e4, this.domNode.style.lineHeight = this._lineHeight);
      }
      setLetterSpacing(t3) {
        let e4 = Ie(t3);
        this._letterSpacing !== e4 && (this._letterSpacing = e4, this.domNode.style.letterSpacing = this._letterSpacing);
      }
      setClassName(t3) {
        this._className !== t3 && (this._className = t3, this.domNode.className = this._className);
      }
      toggleClassName(t3, e4) {
        this.domNode.classList.toggle(t3, e4), this._className = this.domNode.className;
      }
      setDisplay(t3) {
        this._display !== t3 && (this._display = t3, this.domNode.style.display = this._display);
      }
      setPosition(t3) {
        this._position !== t3 && (this._position = t3, this.domNode.style.position = this._position);
      }
      setVisibility(t3) {
        this._visibility !== t3 && (this._visibility = t3, this.domNode.style.visibility = this._visibility);
      }
      setColor(t3) {
        this._color !== t3 && (this._color = t3, this.domNode.style.color = this._color);
      }
      setBackgroundColor(t3) {
        this._backgroundColor !== t3 && (this._backgroundColor = t3, this.domNode.style.backgroundColor = this._backgroundColor);
      }
      setLayerHinting(t3) {
        this._layerHint !== t3 && (this._layerHint = t3, this.domNode.style.transform = this._layerHint ? "translate3d(0px, 0px, 0px)" : "");
      }
      setBoxShadow(t3) {
        this._boxShadow !== t3 && (this._boxShadow = t3, this.domNode.style.boxShadow = t3);
      }
      setContain(t3) {
        this._contain !== t3 && (this._contain = t3, this.domNode.style.contain = this._contain);
      }
      setAttribute(t3, e4) {
        this.domNode.setAttribute(t3, e4);
      }
      removeAttribute(t3) {
        this.domNode.removeAttribute(t3);
      }
      appendChild(t3) {
        this.domNode.appendChild(t3.domNode);
      }
      removeChild(t3) {
        this.domNode.removeChild(t3.domNode);
      }
    };
    Wt = class {
      constructor() {
        this._hooks = new Ee();
        this._pointerMoveCallback = null;
        this._onStopCallback = null;
      }
      dispose() {
        this.stopMonitoring(false), this._hooks.dispose();
      }
      stopMonitoring(t3, e4) {
        if (!this.isMonitoring()) return;
        this._hooks.clear(), this._pointerMoveCallback = null;
        let i12 = this._onStopCallback;
        this._onStopCallback = null, t3 && i12 && i12(e4);
      }
      isMonitoring() {
        return !!this._pointerMoveCallback;
      }
      startMonitoring(t3, e4, i12, r7, n4) {
        this.isMonitoring() && this.stopMonitoring(false), this._pointerMoveCallback = r7, this._onStopCallback = n4;
        let o6 = t3;
        try {
          t3.setPointerCapture(e4), this._hooks.add(C2(() => {
            try {
              t3.releasePointerCapture(e4);
            } catch {
            }
          }));
        } catch {
          o6 = be(t3);
        }
        this._hooks.add(L2(o6, Y.POINTER_MOVE, (l4) => {
          if (l4.buttons !== i12) {
            this.stopMonitoring(true);
            return;
          }
          l4.preventDefault(), this._pointerMoveCallback(l4);
        })), this._hooks.add(L2(o6, Y.POINTER_UP, (l4) => this.stopMonitoring(true)));
      }
    };
    ((n4) => (n4.Tap = "-xterm-gesturetap", n4.Change = "-xterm-gesturechange", n4.Start = "-xterm-gesturestart", n4.End = "-xterm-gesturesend", n4.Contextmenu = "-xterm-gesturecontextmenu"))(He || (He = {}));
    Q = class Q2 extends D2 {
      constructor() {
        super();
        this.dispatched = false;
        this.targets = new Ct();
        this.ignoreTargets = new Ct();
        this.activeTouches = {}, this.handle = null, this._lastSetTapCountTime = 0, this._register($2.runAndSubscribe(No, ({ window: e4, disposables: i12 }) => {
          i12.add(L2(e4.document, "touchstart", (r7) => this.onTouchStart(r7), { passive: false })), i12.add(L2(e4.document, "touchend", (r7) => this.onTouchEnd(e4, r7))), i12.add(L2(e4.document, "touchmove", (r7) => this.onTouchMove(r7), { passive: false }));
        }, { window: fe, disposables: this._store }));
      }
      static addTarget(e4) {
        if (!Q2.isTouchDevice()) return D2.None;
        Q2.INSTANCE || (Q2.INSTANCE = Gn(new Q2()));
        let i12 = Q2.INSTANCE.targets.push(e4);
        return C2(i12);
      }
      static ignoreTarget(e4) {
        if (!Q2.isTouchDevice()) return D2.None;
        Q2.INSTANCE || (Q2.INSTANCE = Gn(new Q2()));
        let i12 = Q2.INSTANCE.ignoreTargets.push(e4);
        return C2(i12);
      }
      static isTouchDevice() {
        return "ontouchstart" in fe || navigator.maxTouchPoints > 0;
      }
      dispose() {
        this.handle && (this.handle.dispose(), this.handle = null), super.dispose();
      }
      onTouchStart(e4) {
        let i12 = Date.now();
        this.handle && (this.handle.dispose(), this.handle = null);
        for (let r7 = 0, n4 = e4.targetTouches.length; r7 < n4; r7++) {
          let o6 = e4.targetTouches.item(r7);
          this.activeTouches[o6.identifier] = { id: o6.identifier, initialTarget: o6.target, initialTimeStamp: i12, initialPageX: o6.pageX, initialPageY: o6.pageY, rollingTimestamps: [i12], rollingPageX: [o6.pageX], rollingPageY: [o6.pageY] };
          let l4 = this.newGestureEvent(He.Start, o6.target);
          l4.pageX = o6.pageX, l4.pageY = o6.pageY, this.dispatchEvent(l4);
        }
        this.dispatched && (e4.preventDefault(), e4.stopPropagation(), this.dispatched = false);
      }
      onTouchEnd(e4, i12) {
        let r7 = Date.now(), n4 = Object.keys(this.activeTouches).length;
        for (let o6 = 0, l4 = i12.changedTouches.length; o6 < l4; o6++) {
          let a3 = i12.changedTouches.item(o6);
          if (!this.activeTouches.hasOwnProperty(String(a3.identifier))) {
            console.warn("move of an UNKNOWN touch", a3);
            continue;
          }
          let u3 = this.activeTouches[a3.identifier], h4 = Date.now() - u3.initialTimeStamp;
          if (h4 < Q2.HOLD_DELAY && Math.abs(u3.initialPageX - Se(u3.rollingPageX)) < 30 && Math.abs(u3.initialPageY - Se(u3.rollingPageY)) < 30) {
            let c4 = this.newGestureEvent(He.Tap, u3.initialTarget);
            c4.pageX = Se(u3.rollingPageX), c4.pageY = Se(u3.rollingPageY), this.dispatchEvent(c4);
          } else if (h4 >= Q2.HOLD_DELAY && Math.abs(u3.initialPageX - Se(u3.rollingPageX)) < 30 && Math.abs(u3.initialPageY - Se(u3.rollingPageY)) < 30) {
            let c4 = this.newGestureEvent(He.Contextmenu, u3.initialTarget);
            c4.pageX = Se(u3.rollingPageX), c4.pageY = Se(u3.rollingPageY), this.dispatchEvent(c4);
          } else if (n4 === 1) {
            let c4 = Se(u3.rollingPageX), d3 = Se(u3.rollingPageY), _4 = Se(u3.rollingTimestamps) - u3.rollingTimestamps[0], p3 = c4 - u3.rollingPageX[0], m2 = d3 - u3.rollingPageY[0], f3 = [...this.targets].filter((A3) => u3.initialTarget instanceof Node && A3.contains(u3.initialTarget));
            this.inertia(e4, f3, r7, Math.abs(p3) / _4, p3 > 0 ? 1 : -1, c4, Math.abs(m2) / _4, m2 > 0 ? 1 : -1, d3);
          }
          this.dispatchEvent(this.newGestureEvent(He.End, u3.initialTarget)), delete this.activeTouches[a3.identifier];
        }
        this.dispatched && (i12.preventDefault(), i12.stopPropagation(), this.dispatched = false);
      }
      newGestureEvent(e4, i12) {
        let r7 = document.createEvent("CustomEvent");
        return r7.initEvent(e4, false, true), r7.initialTarget = i12, r7.tapCount = 0, r7;
      }
      dispatchEvent(e4) {
        if (e4.type === He.Tap) {
          let i12 = (/* @__PURE__ */ new Date()).getTime(), r7 = 0;
          i12 - this._lastSetTapCountTime > Q2.CLEAR_TAP_COUNT_TIME ? r7 = 1 : r7 = 2, this._lastSetTapCountTime = i12, e4.tapCount = r7;
        } else (e4.type === He.Change || e4.type === He.Contextmenu) && (this._lastSetTapCountTime = 0);
        if (e4.initialTarget instanceof Node) {
          for (let r7 of this.ignoreTargets) if (r7.contains(e4.initialTarget)) return;
          let i12 = [];
          for (let r7 of this.targets) if (r7.contains(e4.initialTarget)) {
            let n4 = 0, o6 = e4.initialTarget;
            for (; o6 && o6 !== r7; ) n4++, o6 = o6.parentElement;
            i12.push([n4, r7]);
          }
          i12.sort((r7, n4) => r7[0] - n4[0]);
          for (let [r7, n4] of i12) n4.dispatchEvent(e4), this.dispatched = true;
        }
      }
      inertia(e4, i12, r7, n4, o6, l4, a3, u3, h4) {
        this.handle = mt(e4, () => {
          let c4 = Date.now(), d3 = c4 - r7, _4 = 0, p3 = 0, m2 = true;
          n4 += Q2.SCROLL_FRICTION * d3, a3 += Q2.SCROLL_FRICTION * d3, n4 > 0 && (m2 = false, _4 = o6 * n4 * d3), a3 > 0 && (m2 = false, p3 = u3 * a3 * d3);
          let f3 = this.newGestureEvent(He.Change);
          f3.translationX = _4, f3.translationY = p3, i12.forEach((A3) => A3.dispatchEvent(f3)), m2 || this.inertia(e4, i12, c4, n4, o6, l4 + _4, a3, u3, h4 + p3);
        });
      }
      onTouchMove(e4) {
        let i12 = Date.now();
        for (let r7 = 0, n4 = e4.changedTouches.length; r7 < n4; r7++) {
          let o6 = e4.changedTouches.item(r7);
          if (!this.activeTouches.hasOwnProperty(String(o6.identifier))) {
            console.warn("end of an UNKNOWN touch", o6);
            continue;
          }
          let l4 = this.activeTouches[o6.identifier], a3 = this.newGestureEvent(He.Change, l4.initialTarget);
          a3.translationX = o6.pageX - Se(l4.rollingPageX), a3.translationY = o6.pageY - Se(l4.rollingPageY), a3.pageX = o6.pageX, a3.pageY = o6.pageY, this.dispatchEvent(a3), l4.rollingPageX.length > 3 && (l4.rollingPageX.shift(), l4.rollingPageY.shift(), l4.rollingTimestamps.shift()), l4.rollingPageX.push(o6.pageX), l4.rollingPageY.push(o6.pageY), l4.rollingTimestamps.push(i12);
        }
        this.dispatched && (e4.preventDefault(), e4.stopPropagation(), this.dispatched = false);
      }
    };
    Q.SCROLL_FRICTION = -5e-3, Q.HOLD_DELAY = 700, Q.CLEAR_TAP_COUNT_TIME = 400, M2([Wo], Q, "isTouchDevice", 1);
    Pr = Q;
    lt = class extends D2 {
      onclick(t3, e4) {
        this._register(L2(t3, Y.CLICK, (i12) => e4(new qe(be(t3), i12))));
      }
      onmousedown(t3, e4) {
        this._register(L2(t3, Y.MOUSE_DOWN, (i12) => e4(new qe(be(t3), i12))));
      }
      onmouseover(t3, e4) {
        this._register(L2(t3, Y.MOUSE_OVER, (i12) => e4(new qe(be(t3), i12))));
      }
      onmouseleave(t3, e4) {
        this._register(L2(t3, Y.MOUSE_LEAVE, (i12) => e4(new qe(be(t3), i12))));
      }
      onkeydown(t3, e4) {
        this._register(L2(t3, Y.KEY_DOWN, (i12) => e4(new ft(i12))));
      }
      onkeyup(t3, e4) {
        this._register(L2(t3, Y.KEY_UP, (i12) => e4(new ft(i12))));
      }
      oninput(t3, e4) {
        this._register(L2(t3, Y.INPUT, e4));
      }
      onblur(t3, e4) {
        this._register(L2(t3, Y.BLUR, e4));
      }
      onfocus(t3, e4) {
        this._register(L2(t3, Y.FOCUS, e4));
      }
      onchange(t3, e4) {
        this._register(L2(t3, Y.CHANGE, e4));
      }
      ignoreGesture(t3) {
        return Pr.ignoreTarget(t3);
      }
    };
    Uo = 11;
    Or = class extends lt {
      constructor(t3) {
        super(), this._onActivate = t3.onActivate, this.bgDomNode = document.createElement("div"), this.bgDomNode.className = "arrow-background", this.bgDomNode.style.position = "absolute", this.bgDomNode.style.width = t3.bgWidth + "px", this.bgDomNode.style.height = t3.bgHeight + "px", typeof t3.top < "u" && (this.bgDomNode.style.top = "0px"), typeof t3.left < "u" && (this.bgDomNode.style.left = "0px"), typeof t3.bottom < "u" && (this.bgDomNode.style.bottom = "0px"), typeof t3.right < "u" && (this.bgDomNode.style.right = "0px"), this.domNode = document.createElement("div"), this.domNode.className = t3.className, this.domNode.style.position = "absolute", this.domNode.style.width = Uo + "px", this.domNode.style.height = Uo + "px", typeof t3.top < "u" && (this.domNode.style.top = t3.top + "px"), typeof t3.left < "u" && (this.domNode.style.left = t3.left + "px"), typeof t3.bottom < "u" && (this.domNode.style.bottom = t3.bottom + "px"), typeof t3.right < "u" && (this.domNode.style.right = t3.right + "px"), this._pointerMoveMonitor = this._register(new Wt()), this._register(os(this.bgDomNode, Y.POINTER_DOWN, (e4) => this._arrowPointerDown(e4))), this._register(os(this.domNode, Y.POINTER_DOWN, (e4) => this._arrowPointerDown(e4))), this._pointerdownRepeatTimer = this._register(new Mr()), this._pointerdownScheduleRepeatTimer = this._register(new Ye());
      }
      _arrowPointerDown(t3) {
        if (!t3.target || !(t3.target instanceof Element)) return;
        let e4 = () => {
          this._pointerdownRepeatTimer.cancelAndSet(() => this._onActivate(), 1e3 / 24, be(t3));
        };
        this._onActivate(), this._pointerdownRepeatTimer.cancel(), this._pointerdownScheduleRepeatTimer.cancelAndSet(e4, 200), this._pointerMoveMonitor.startMonitoring(t3.target, t3.pointerId, t3.buttons, (i12) => {
        }, () => {
          this._pointerdownRepeatTimer.cancel(), this._pointerdownScheduleRepeatTimer.cancel();
        }), t3.preventDefault();
      }
    };
    cs = class s12 {
      constructor(t3, e4, i12, r7, n4, o6, l4) {
        this._forceIntegerValues = t3;
        this._scrollStateBrand = void 0;
        this._forceIntegerValues && (e4 = e4 | 0, i12 = i12 | 0, r7 = r7 | 0, n4 = n4 | 0, o6 = o6 | 0, l4 = l4 | 0), this.rawScrollLeft = r7, this.rawScrollTop = l4, e4 < 0 && (e4 = 0), r7 + e4 > i12 && (r7 = i12 - e4), r7 < 0 && (r7 = 0), n4 < 0 && (n4 = 0), l4 + n4 > o6 && (l4 = o6 - n4), l4 < 0 && (l4 = 0), this.width = e4, this.scrollWidth = i12, this.scrollLeft = r7, this.height = n4, this.scrollHeight = o6, this.scrollTop = l4;
      }
      equals(t3) {
        return this.rawScrollLeft === t3.rawScrollLeft && this.rawScrollTop === t3.rawScrollTop && this.width === t3.width && this.scrollWidth === t3.scrollWidth && this.scrollLeft === t3.scrollLeft && this.height === t3.height && this.scrollHeight === t3.scrollHeight && this.scrollTop === t3.scrollTop;
      }
      withScrollDimensions(t3, e4) {
        return new s12(this._forceIntegerValues, typeof t3.width < "u" ? t3.width : this.width, typeof t3.scrollWidth < "u" ? t3.scrollWidth : this.scrollWidth, e4 ? this.rawScrollLeft : this.scrollLeft, typeof t3.height < "u" ? t3.height : this.height, typeof t3.scrollHeight < "u" ? t3.scrollHeight : this.scrollHeight, e4 ? this.rawScrollTop : this.scrollTop);
      }
      withScrollPosition(t3) {
        return new s12(this._forceIntegerValues, this.width, this.scrollWidth, typeof t3.scrollLeft < "u" ? t3.scrollLeft : this.rawScrollLeft, this.height, this.scrollHeight, typeof t3.scrollTop < "u" ? t3.scrollTop : this.rawScrollTop);
      }
      createScrollEvent(t3, e4) {
        let i12 = this.width !== t3.width, r7 = this.scrollWidth !== t3.scrollWidth, n4 = this.scrollLeft !== t3.scrollLeft, o6 = this.height !== t3.height, l4 = this.scrollHeight !== t3.scrollHeight, a3 = this.scrollTop !== t3.scrollTop;
        return { inSmoothScrolling: e4, oldWidth: t3.width, oldScrollWidth: t3.scrollWidth, oldScrollLeft: t3.scrollLeft, width: this.width, scrollWidth: this.scrollWidth, scrollLeft: this.scrollLeft, oldHeight: t3.height, oldScrollHeight: t3.scrollHeight, oldScrollTop: t3.scrollTop, height: this.height, scrollHeight: this.scrollHeight, scrollTop: this.scrollTop, widthChanged: i12, scrollWidthChanged: r7, scrollLeftChanged: n4, heightChanged: o6, scrollHeightChanged: l4, scrollTopChanged: a3 };
      }
    };
    Ri = class extends D2 {
      constructor(e4) {
        super();
        this._scrollableBrand = void 0;
        this._onScroll = this._register(new v2());
        this.onScroll = this._onScroll.event;
        this._smoothScrollDuration = e4.smoothScrollDuration, this._scheduleAtNextAnimationFrame = e4.scheduleAtNextAnimationFrame, this._state = new cs(e4.forceIntegerValues, 0, 0, 0, 0, 0, 0), this._smoothScrolling = null;
      }
      dispose() {
        this._smoothScrolling && (this._smoothScrolling.dispose(), this._smoothScrolling = null), super.dispose();
      }
      setSmoothScrollDuration(e4) {
        this._smoothScrollDuration = e4;
      }
      validateScrollPosition(e4) {
        return this._state.withScrollPosition(e4);
      }
      getScrollDimensions() {
        return this._state;
      }
      setScrollDimensions(e4, i12) {
        let r7 = this._state.withScrollDimensions(e4, i12);
        this._setState(r7, !!this._smoothScrolling), this._smoothScrolling?.acceptScrollDimensions(this._state);
      }
      getFutureScrollPosition() {
        return this._smoothScrolling ? this._smoothScrolling.to : this._state;
      }
      getCurrentScrollPosition() {
        return this._state;
      }
      setScrollPositionNow(e4) {
        let i12 = this._state.withScrollPosition(e4);
        this._smoothScrolling && (this._smoothScrolling.dispose(), this._smoothScrolling = null), this._setState(i12, false);
      }
      setScrollPositionSmooth(e4, i12) {
        if (this._smoothScrollDuration === 0) return this.setScrollPositionNow(e4);
        if (this._smoothScrolling) {
          e4 = { scrollLeft: typeof e4.scrollLeft > "u" ? this._smoothScrolling.to.scrollLeft : e4.scrollLeft, scrollTop: typeof e4.scrollTop > "u" ? this._smoothScrolling.to.scrollTop : e4.scrollTop };
          let r7 = this._state.withScrollPosition(e4);
          if (this._smoothScrolling.to.scrollLeft === r7.scrollLeft && this._smoothScrolling.to.scrollTop === r7.scrollTop) return;
          let n4;
          i12 ? n4 = new Nr(this._smoothScrolling.from, r7, this._smoothScrolling.startTime, this._smoothScrolling.duration) : n4 = this._smoothScrolling.combine(this._state, r7, this._smoothScrollDuration), this._smoothScrolling.dispose(), this._smoothScrolling = n4;
        } else {
          let r7 = this._state.withScrollPosition(e4);
          this._smoothScrolling = Nr.start(this._state, r7, this._smoothScrollDuration);
        }
        this._smoothScrolling.animationFrameDisposable = this._scheduleAtNextAnimationFrame(() => {
          this._smoothScrolling && (this._smoothScrolling.animationFrameDisposable = null, this._performSmoothScrolling());
        });
      }
      hasPendingScrollAnimation() {
        return !!this._smoothScrolling;
      }
      _performSmoothScrolling() {
        if (!this._smoothScrolling) return;
        let e4 = this._smoothScrolling.tick(), i12 = this._state.withScrollPosition(e4);
        if (this._setState(i12, true), !!this._smoothScrolling) {
          if (e4.isDone) {
            this._smoothScrolling.dispose(), this._smoothScrolling = null;
            return;
          }
          this._smoothScrolling.animationFrameDisposable = this._scheduleAtNextAnimationFrame(() => {
            this._smoothScrolling && (this._smoothScrolling.animationFrameDisposable = null, this._performSmoothScrolling());
          });
        }
      }
      _setState(e4, i12) {
        let r7 = this._state;
        r7.equals(e4) || (this._state = e4, this._onScroll.fire(this._state.createScrollEvent(r7, i12)));
      }
    };
    Br = class {
      constructor(t3, e4, i12) {
        this.scrollLeft = t3, this.scrollTop = e4, this.isDone = i12;
      }
    };
    Nr = class s13 {
      constructor(t3, e4, i12, r7) {
        this.from = t3, this.to = e4, this.duration = r7, this.startTime = i12, this.animationFrameDisposable = null, this._initAnimations();
      }
      _initAnimations() {
        this.scrollLeft = this._initAnimation(this.from.scrollLeft, this.to.scrollLeft, this.to.width), this.scrollTop = this._initAnimation(this.from.scrollTop, this.to.scrollTop, this.to.height);
      }
      _initAnimation(t3, e4, i12) {
        if (Math.abs(t3 - e4) > 2.5 * i12) {
          let n4, o6;
          return t3 < e4 ? (n4 = t3 + 0.75 * i12, o6 = e4 - 0.75 * i12) : (n4 = t3 - 0.75 * i12, o6 = e4 + 0.75 * i12), La(as(t3, n4), as(o6, e4), 0.33);
        }
        return as(t3, e4);
      }
      dispose() {
        this.animationFrameDisposable !== null && (this.animationFrameDisposable.dispose(), this.animationFrameDisposable = null);
      }
      acceptScrollDimensions(t3) {
        this.to = t3.withScrollPosition(this.to), this._initAnimations();
      }
      tick() {
        return this._tick(Date.now());
      }
      _tick(t3) {
        let e4 = (t3 - this.startTime) / this.duration;
        if (e4 < 1) {
          let i12 = this.scrollLeft(e4), r7 = this.scrollTop(e4);
          return new Br(i12, r7, false);
        }
        return new Br(this.to.scrollLeft, this.to.scrollTop, true);
      }
      combine(t3, e4, i12) {
        return s13.start(t3, e4, i12);
      }
      static start(t3, e4, i12) {
        i12 = i12 + 10;
        let r7 = Date.now() - 10;
        return new s13(t3, e4, r7, i12);
      }
    };
    Fr = class extends D2 {
      constructor(t3, e4, i12) {
        super(), this._visibility = t3, this._visibleClassName = e4, this._invisibleClassName = i12, this._domNode = null, this._isVisible = false, this._isNeeded = false, this._rawShouldBeVisible = false, this._shouldBeVisible = false, this._revealTimer = this._register(new Ye());
      }
      setVisibility(t3) {
        this._visibility !== t3 && (this._visibility = t3, this._updateShouldBeVisible());
      }
      setShouldBeVisible(t3) {
        this._rawShouldBeVisible = t3, this._updateShouldBeVisible();
      }
      _applyVisibilitySetting() {
        return this._visibility === 2 ? false : this._visibility === 3 ? true : this._rawShouldBeVisible;
      }
      _updateShouldBeVisible() {
        let t3 = this._applyVisibilitySetting();
        this._shouldBeVisible !== t3 && (this._shouldBeVisible = t3, this.ensureVisibility());
      }
      setIsNeeded(t3) {
        this._isNeeded !== t3 && (this._isNeeded = t3, this.ensureVisibility());
      }
      setDomNode(t3) {
        this._domNode = t3, this._domNode.setClassName(this._invisibleClassName), this.setShouldBeVisible(false);
      }
      ensureVisibility() {
        if (!this._isNeeded) {
          this._hide(false);
          return;
        }
        this._shouldBeVisible ? this._reveal() : this._hide(true);
      }
      _reveal() {
        this._isVisible || (this._isVisible = true, this._revealTimer.setIfNotSet(() => {
          this._domNode?.setClassName(this._visibleClassName);
        }, 0));
      }
      _hide(t3) {
        this._revealTimer.cancel(), this._isVisible && (this._isVisible = false, this._domNode?.setClassName(this._invisibleClassName + (t3 ? " fade" : "")));
      }
    };
    Ca = 140;
    Ut = class extends lt {
      constructor(t3) {
        super(), this._lazyRender = t3.lazyRender, this._host = t3.host, this._scrollable = t3.scrollable, this._scrollByPage = t3.scrollByPage, this._scrollbarState = t3.scrollbarState, this._visibilityController = this._register(new Fr(t3.visibility, "visible scrollbar " + t3.extraScrollbarClassName, "invisible scrollbar " + t3.extraScrollbarClassName)), this._visibilityController.setIsNeeded(this._scrollbarState.isNeeded()), this._pointerMoveMonitor = this._register(new Wt()), this._shouldRender = true, this.domNode = _t(document.createElement("div")), this.domNode.setAttribute("role", "presentation"), this.domNode.setAttribute("aria-hidden", "true"), this._visibilityController.setDomNode(this.domNode), this.domNode.setPosition("absolute"), this._register(L2(this.domNode.domNode, Y.POINTER_DOWN, (e4) => this._domNodePointerDown(e4)));
      }
      _createArrow(t3) {
        let e4 = this._register(new Or(t3));
        this.domNode.domNode.appendChild(e4.bgDomNode), this.domNode.domNode.appendChild(e4.domNode);
      }
      _createSlider(t3, e4, i12, r7) {
        this.slider = _t(document.createElement("div")), this.slider.setClassName("slider"), this.slider.setPosition("absolute"), this.slider.setTop(t3), this.slider.setLeft(e4), typeof i12 == "number" && this.slider.setWidth(i12), typeof r7 == "number" && this.slider.setHeight(r7), this.slider.setLayerHinting(true), this.slider.setContain("strict"), this.domNode.domNode.appendChild(this.slider.domNode), this._register(L2(this.slider.domNode, Y.POINTER_DOWN, (n4) => {
          n4.button === 0 && (n4.preventDefault(), this._sliderPointerDown(n4));
        })), this.onclick(this.slider.domNode, (n4) => {
          n4.leftButton && n4.stopPropagation();
        });
      }
      _onElementSize(t3) {
        return this._scrollbarState.setVisibleSize(t3) && (this._visibilityController.setIsNeeded(this._scrollbarState.isNeeded()), this._shouldRender = true, this._lazyRender || this.render()), this._shouldRender;
      }
      _onElementScrollSize(t3) {
        return this._scrollbarState.setScrollSize(t3) && (this._visibilityController.setIsNeeded(this._scrollbarState.isNeeded()), this._shouldRender = true, this._lazyRender || this.render()), this._shouldRender;
      }
      _onElementScrollPosition(t3) {
        return this._scrollbarState.setScrollPosition(t3) && (this._visibilityController.setIsNeeded(this._scrollbarState.isNeeded()), this._shouldRender = true, this._lazyRender || this.render()), this._shouldRender;
      }
      beginReveal() {
        this._visibilityController.setShouldBeVisible(true);
      }
      beginHide() {
        this._visibilityController.setShouldBeVisible(false);
      }
      render() {
        this._shouldRender && (this._shouldRender = false, this._renderDomNode(this._scrollbarState.getRectangleLargeSize(), this._scrollbarState.getRectangleSmallSize()), this._updateSlider(this._scrollbarState.getSliderSize(), this._scrollbarState.getArrowSize() + this._scrollbarState.getSliderPosition()));
      }
      _domNodePointerDown(t3) {
        t3.target === this.domNode.domNode && this._onPointerDown(t3);
      }
      delegatePointerDown(t3) {
        let e4 = this.domNode.domNode.getClientRects()[0].top, i12 = e4 + this._scrollbarState.getSliderPosition(), r7 = e4 + this._scrollbarState.getSliderPosition() + this._scrollbarState.getSliderSize(), n4 = this._sliderPointerPosition(t3);
        i12 <= n4 && n4 <= r7 ? t3.button === 0 && (t3.preventDefault(), this._sliderPointerDown(t3)) : this._onPointerDown(t3);
      }
      _onPointerDown(t3) {
        let e4, i12;
        if (t3.target === this.domNode.domNode && typeof t3.offsetX == "number" && typeof t3.offsetY == "number") e4 = t3.offsetX, i12 = t3.offsetY;
        else {
          let n4 = Fo(this.domNode.domNode);
          e4 = t3.pageX - n4.left, i12 = t3.pageY - n4.top;
        }
        let r7 = this._pointerDownRelativePosition(e4, i12);
        this._setDesiredScrollPositionNow(this._scrollByPage ? this._scrollbarState.getDesiredScrollPositionFromOffsetPaged(r7) : this._scrollbarState.getDesiredScrollPositionFromOffset(r7)), t3.button === 0 && (t3.preventDefault(), this._sliderPointerDown(t3));
      }
      _sliderPointerDown(t3) {
        if (!t3.target || !(t3.target instanceof Element)) return;
        let e4 = this._sliderPointerPosition(t3), i12 = this._sliderOrthogonalPointerPosition(t3), r7 = this._scrollbarState.clone();
        this.slider.toggleClassName("active", true), this._pointerMoveMonitor.startMonitoring(t3.target, t3.pointerId, t3.buttons, (n4) => {
          let o6 = this._sliderOrthogonalPointerPosition(n4), l4 = Math.abs(o6 - i12);
          if (wr && l4 > Ca) {
            this._setDesiredScrollPositionNow(r7.getScrollPosition());
            return;
          }
          let u3 = this._sliderPointerPosition(n4) - e4;
          this._setDesiredScrollPositionNow(r7.getDesiredScrollPositionFromDelta(u3));
        }, () => {
          this.slider.toggleClassName("active", false), this._host.onDragEnd();
        }), this._host.onDragStart();
      }
      _setDesiredScrollPositionNow(t3) {
        let e4 = {};
        this.writeScrollPosition(e4, t3), this._scrollable.setScrollPositionNow(e4);
      }
      updateScrollbarSize(t3) {
        this._updateScrollbarSize(t3), this._scrollbarState.setScrollbarSize(t3), this._shouldRender = true, this._lazyRender || this.render();
      }
      isNeeded() {
        return this._scrollbarState.isNeeded();
      }
    };
    Kt = class s14 {
      constructor(t3, e4, i12, r7, n4, o6) {
        this._scrollbarSize = Math.round(e4), this._oppositeScrollbarSize = Math.round(i12), this._arrowSize = Math.round(t3), this._visibleSize = r7, this._scrollSize = n4, this._scrollPosition = o6, this._computedAvailableSize = 0, this._computedIsNeeded = false, this._computedSliderSize = 0, this._computedSliderRatio = 0, this._computedSliderPosition = 0, this._refreshComputedValues();
      }
      clone() {
        return new s14(this._arrowSize, this._scrollbarSize, this._oppositeScrollbarSize, this._visibleSize, this._scrollSize, this._scrollPosition);
      }
      setVisibleSize(t3) {
        let e4 = Math.round(t3);
        return this._visibleSize !== e4 ? (this._visibleSize = e4, this._refreshComputedValues(), true) : false;
      }
      setScrollSize(t3) {
        let e4 = Math.round(t3);
        return this._scrollSize !== e4 ? (this._scrollSize = e4, this._refreshComputedValues(), true) : false;
      }
      setScrollPosition(t3) {
        let e4 = Math.round(t3);
        return this._scrollPosition !== e4 ? (this._scrollPosition = e4, this._refreshComputedValues(), true) : false;
      }
      setScrollbarSize(t3) {
        this._scrollbarSize = Math.round(t3);
      }
      setOppositeScrollbarSize(t3) {
        this._oppositeScrollbarSize = Math.round(t3);
      }
      static _computeValues(t3, e4, i12, r7, n4) {
        let o6 = Math.max(0, i12 - t3), l4 = Math.max(0, o6 - 2 * e4), a3 = r7 > 0 && r7 > i12;
        if (!a3) return { computedAvailableSize: Math.round(o6), computedIsNeeded: a3, computedSliderSize: Math.round(l4), computedSliderRatio: 0, computedSliderPosition: 0 };
        let u3 = Math.round(Math.max(20, Math.floor(i12 * l4 / r7))), h4 = (l4 - u3) / (r7 - i12), c4 = n4 * h4;
        return { computedAvailableSize: Math.round(o6), computedIsNeeded: a3, computedSliderSize: Math.round(u3), computedSliderRatio: h4, computedSliderPosition: Math.round(c4) };
      }
      _refreshComputedValues() {
        let t3 = s14._computeValues(this._oppositeScrollbarSize, this._arrowSize, this._visibleSize, this._scrollSize, this._scrollPosition);
        this._computedAvailableSize = t3.computedAvailableSize, this._computedIsNeeded = t3.computedIsNeeded, this._computedSliderSize = t3.computedSliderSize, this._computedSliderRatio = t3.computedSliderRatio, this._computedSliderPosition = t3.computedSliderPosition;
      }
      getArrowSize() {
        return this._arrowSize;
      }
      getScrollPosition() {
        return this._scrollPosition;
      }
      getRectangleLargeSize() {
        return this._computedAvailableSize;
      }
      getRectangleSmallSize() {
        return this._scrollbarSize;
      }
      isNeeded() {
        return this._computedIsNeeded;
      }
      getSliderSize() {
        return this._computedSliderSize;
      }
      getSliderPosition() {
        return this._computedSliderPosition;
      }
      getDesiredScrollPositionFromOffset(t3) {
        if (!this._computedIsNeeded) return 0;
        let e4 = t3 - this._arrowSize - this._computedSliderSize / 2;
        return Math.round(e4 / this._computedSliderRatio);
      }
      getDesiredScrollPositionFromOffsetPaged(t3) {
        if (!this._computedIsNeeded) return 0;
        let e4 = t3 - this._arrowSize, i12 = this._scrollPosition;
        return e4 < this._computedSliderPosition ? i12 -= this._visibleSize : i12 += this._visibleSize, i12;
      }
      getDesiredScrollPositionFromDelta(t3) {
        if (!this._computedIsNeeded) return 0;
        let e4 = this._computedSliderPosition + t3;
        return Math.round(e4 / this._computedSliderRatio);
      }
    };
    Wr = class extends Ut {
      constructor(t3, e4, i12) {
        let r7 = t3.getScrollDimensions(), n4 = t3.getCurrentScrollPosition();
        if (super({ lazyRender: e4.lazyRender, host: i12, scrollbarState: new Kt(e4.horizontalHasArrows ? e4.arrowSize : 0, e4.horizontal === 2 ? 0 : e4.horizontalScrollbarSize, e4.vertical === 2 ? 0 : e4.verticalScrollbarSize, r7.width, r7.scrollWidth, n4.scrollLeft), visibility: e4.horizontal, extraScrollbarClassName: "horizontal", scrollable: t3, scrollByPage: e4.scrollByPage }), e4.horizontalHasArrows) throw new Error("horizontalHasArrows is not supported in xterm.js");
        this._createSlider(Math.floor((e4.horizontalScrollbarSize - e4.horizontalSliderSize) / 2), 0, void 0, e4.horizontalSliderSize);
      }
      _updateSlider(t3, e4) {
        this.slider.setWidth(t3), this.slider.setLeft(e4);
      }
      _renderDomNode(t3, e4) {
        this.domNode.setWidth(t3), this.domNode.setHeight(e4), this.domNode.setLeft(0), this.domNode.setBottom(0);
      }
      onDidScroll(t3) {
        return this._shouldRender = this._onElementScrollSize(t3.scrollWidth) || this._shouldRender, this._shouldRender = this._onElementScrollPosition(t3.scrollLeft) || this._shouldRender, this._shouldRender = this._onElementSize(t3.width) || this._shouldRender, this._shouldRender;
      }
      _pointerDownRelativePosition(t3, e4) {
        return t3;
      }
      _sliderPointerPosition(t3) {
        return t3.pageX;
      }
      _sliderOrthogonalPointerPosition(t3) {
        return t3.pageY;
      }
      _updateScrollbarSize(t3) {
        this.slider.setHeight(t3);
      }
      writeScrollPosition(t3, e4) {
        t3.scrollLeft = e4;
      }
      updateOptions(t3) {
        this.updateScrollbarSize(t3.horizontal === 2 ? 0 : t3.horizontalScrollbarSize), this._scrollbarState.setOppositeScrollbarSize(t3.vertical === 2 ? 0 : t3.verticalScrollbarSize), this._visibilityController.setVisibility(t3.horizontal), this._scrollByPage = t3.scrollByPage;
      }
    };
    Ur = class extends Ut {
      constructor(t3, e4, i12) {
        let r7 = t3.getScrollDimensions(), n4 = t3.getCurrentScrollPosition();
        if (super({ lazyRender: e4.lazyRender, host: i12, scrollbarState: new Kt(e4.verticalHasArrows ? e4.arrowSize : 0, e4.vertical === 2 ? 0 : e4.verticalScrollbarSize, 0, r7.height, r7.scrollHeight, n4.scrollTop), visibility: e4.vertical, extraScrollbarClassName: "vertical", scrollable: t3, scrollByPage: e4.scrollByPage }), e4.verticalHasArrows) throw new Error("horizontalHasArrows is not supported in xterm.js");
        this._createSlider(0, Math.floor((e4.verticalScrollbarSize - e4.verticalSliderSize) / 2), e4.verticalSliderSize, void 0);
      }
      _updateSlider(t3, e4) {
        this.slider.setHeight(t3), this.slider.setTop(e4);
      }
      _renderDomNode(t3, e4) {
        this.domNode.setWidth(e4), this.domNode.setHeight(t3), this.domNode.setRight(0), this.domNode.setTop(0);
      }
      onDidScroll(t3) {
        return this._shouldRender = this._onElementScrollSize(t3.scrollHeight) || this._shouldRender, this._shouldRender = this._onElementScrollPosition(t3.scrollTop) || this._shouldRender, this._shouldRender = this._onElementSize(t3.height) || this._shouldRender, this._shouldRender;
      }
      _pointerDownRelativePosition(t3, e4) {
        return e4;
      }
      _sliderPointerPosition(t3) {
        return t3.pageY;
      }
      _sliderOrthogonalPointerPosition(t3) {
        return t3.pageX;
      }
      _updateScrollbarSize(t3) {
        this.slider.setWidth(t3);
      }
      writeScrollPosition(t3, e4) {
        t3.scrollTop = e4;
      }
      updateOptions(t3) {
        this.updateScrollbarSize(t3.vertical === 2 ? 0 : t3.verticalScrollbarSize), this._scrollbarState.setOppositeScrollbarSize(0), this._visibilityController.setVisibility(t3.vertical), this._scrollByPage = t3.scrollByPage;
      }
    };
    Ma = 500;
    Ko = 50;
    zo = true;
    us = class {
      constructor(t3, e4, i12) {
        this.timestamp = t3, this.deltaX = e4, this.deltaY = i12, this.score = 0;
      }
    };
    zr = class zr2 {
      constructor() {
        this._capacity = 5, this._memory = [], this._front = -1, this._rear = -1;
      }
      isPhysicalMouseWheel() {
        if (this._front === -1 && this._rear === -1) return false;
        let t3 = 1, e4 = 0, i12 = 1, r7 = this._rear;
        do {
          let n4 = r7 === this._front ? t3 : Math.pow(2, -i12);
          if (t3 -= n4, e4 += this._memory[r7].score * n4, r7 === this._front) break;
          r7 = (this._capacity + r7 - 1) % this._capacity, i12++;
        } while (true);
        return e4 <= 0.5;
      }
      acceptStandardWheelEvent(t3) {
        if (Ti) {
          let e4 = be(t3.browserEvent), i12 = mo(e4);
          this.accept(Date.now(), t3.deltaX * i12, t3.deltaY * i12);
        } else this.accept(Date.now(), t3.deltaX, t3.deltaY);
      }
      accept(t3, e4, i12) {
        let r7 = null, n4 = new us(t3, e4, i12);
        this._front === -1 && this._rear === -1 ? (this._memory[0] = n4, this._front = 0, this._rear = 0) : (r7 = this._memory[this._rear], this._rear = (this._rear + 1) % this._capacity, this._rear === this._front && (this._front = (this._front + 1) % this._capacity), this._memory[this._rear] = n4), n4.score = this._computeScore(n4, r7);
      }
      _computeScore(t3, e4) {
        if (Math.abs(t3.deltaX) > 0 && Math.abs(t3.deltaY) > 0) return 1;
        let i12 = 0.5;
        if ((!this._isAlmostInt(t3.deltaX) || !this._isAlmostInt(t3.deltaY)) && (i12 += 0.25), e4) {
          let r7 = Math.abs(t3.deltaX), n4 = Math.abs(t3.deltaY), o6 = Math.abs(e4.deltaX), l4 = Math.abs(e4.deltaY), a3 = Math.max(Math.min(r7, o6), 1), u3 = Math.max(Math.min(n4, l4), 1), h4 = Math.max(r7, o6), c4 = Math.max(n4, l4);
          h4 % a3 === 0 && c4 % u3 === 0 && (i12 -= 0.5);
        }
        return Math.min(Math.max(i12, 0), 1);
      }
      _isAlmostInt(t3) {
        return Math.abs(Math.round(t3) - t3) < 0.01;
      }
    };
    zr.INSTANCE = new zr();
    hs = zr;
    ds = class extends lt {
      constructor(e4, i12, r7) {
        super();
        this._onScroll = this._register(new v2());
        this.onScroll = this._onScroll.event;
        this._onWillScroll = this._register(new v2());
        this.onWillScroll = this._onWillScroll.event;
        this._options = Pa(i12), this._scrollable = r7, this._register(this._scrollable.onScroll((o6) => {
          this._onWillScroll.fire(o6), this._onDidScroll(o6), this._onScroll.fire(o6);
        }));
        let n4 = { onMouseWheel: (o6) => this._onMouseWheel(o6), onDragStart: () => this._onDragStart(), onDragEnd: () => this._onDragEnd() };
        this._verticalScrollbar = this._register(new Ur(this._scrollable, this._options, n4)), this._horizontalScrollbar = this._register(new Wr(this._scrollable, this._options, n4)), this._domNode = document.createElement("div"), this._domNode.className = "xterm-scrollable-element " + this._options.className, this._domNode.setAttribute("role", "presentation"), this._domNode.style.position = "relative", this._domNode.appendChild(e4), this._domNode.appendChild(this._horizontalScrollbar.domNode.domNode), this._domNode.appendChild(this._verticalScrollbar.domNode.domNode), this._options.useShadows ? (this._leftShadowDomNode = _t(document.createElement("div")), this._leftShadowDomNode.setClassName("shadow"), this._domNode.appendChild(this._leftShadowDomNode.domNode), this._topShadowDomNode = _t(document.createElement("div")), this._topShadowDomNode.setClassName("shadow"), this._domNode.appendChild(this._topShadowDomNode.domNode), this._topLeftShadowDomNode = _t(document.createElement("div")), this._topLeftShadowDomNode.setClassName("shadow"), this._domNode.appendChild(this._topLeftShadowDomNode.domNode)) : (this._leftShadowDomNode = null, this._topShadowDomNode = null, this._topLeftShadowDomNode = null), this._listenOnDomNode = this._options.listenOnDomNode || this._domNode, this._mouseWheelToDispose = [], this._setListeningToMouseWheel(this._options.handleMouseWheel), this.onmouseover(this._listenOnDomNode, (o6) => this._onMouseOver(o6)), this.onmouseleave(this._listenOnDomNode, (o6) => this._onMouseLeave(o6)), this._hideTimeout = this._register(new Ye()), this._isDragging = false, this._mouseIsOver = false, this._shouldRender = true, this._revealOnScroll = true;
      }
      get options() {
        return this._options;
      }
      dispose() {
        this._mouseWheelToDispose = Ne(this._mouseWheelToDispose), super.dispose();
      }
      getDomNode() {
        return this._domNode;
      }
      getOverviewRulerLayoutInfo() {
        return { parent: this._domNode, insertBefore: this._verticalScrollbar.domNode.domNode };
      }
      delegateVerticalScrollbarPointerDown(e4) {
        this._verticalScrollbar.delegatePointerDown(e4);
      }
      getScrollDimensions() {
        return this._scrollable.getScrollDimensions();
      }
      setScrollDimensions(e4) {
        this._scrollable.setScrollDimensions(e4, false);
      }
      updateClassName(e4) {
        this._options.className = e4, Te && (this._options.className += " mac"), this._domNode.className = "xterm-scrollable-element " + this._options.className;
      }
      updateOptions(e4) {
        typeof e4.handleMouseWheel < "u" && (this._options.handleMouseWheel = e4.handleMouseWheel, this._setListeningToMouseWheel(this._options.handleMouseWheel)), typeof e4.mouseWheelScrollSensitivity < "u" && (this._options.mouseWheelScrollSensitivity = e4.mouseWheelScrollSensitivity), typeof e4.fastScrollSensitivity < "u" && (this._options.fastScrollSensitivity = e4.fastScrollSensitivity), typeof e4.scrollPredominantAxis < "u" && (this._options.scrollPredominantAxis = e4.scrollPredominantAxis), typeof e4.horizontal < "u" && (this._options.horizontal = e4.horizontal), typeof e4.vertical < "u" && (this._options.vertical = e4.vertical), typeof e4.horizontalScrollbarSize < "u" && (this._options.horizontalScrollbarSize = e4.horizontalScrollbarSize), typeof e4.verticalScrollbarSize < "u" && (this._options.verticalScrollbarSize = e4.verticalScrollbarSize), typeof e4.scrollByPage < "u" && (this._options.scrollByPage = e4.scrollByPage), this._horizontalScrollbar.updateOptions(this._options), this._verticalScrollbar.updateOptions(this._options), this._options.lazyRender || this._render();
      }
      setRevealOnScroll(e4) {
        this._revealOnScroll = e4;
      }
      delegateScrollFromMouseWheelEvent(e4) {
        this._onMouseWheel(new xi(e4));
      }
      _setListeningToMouseWheel(e4) {
        if (this._mouseWheelToDispose.length > 0 !== e4 && (this._mouseWheelToDispose = Ne(this._mouseWheelToDispose), e4)) {
          let r7 = (n4) => {
            this._onMouseWheel(new xi(n4));
          };
          this._mouseWheelToDispose.push(L2(this._listenOnDomNode, Y.MOUSE_WHEEL, r7, { passive: false }));
        }
      }
      _onMouseWheel(e4) {
        if (e4.browserEvent?.defaultPrevented) return;
        let i12 = hs.INSTANCE;
        zo && i12.acceptStandardWheelEvent(e4);
        let r7 = false;
        if (e4.deltaY || e4.deltaX) {
          let o6 = e4.deltaY * this._options.mouseWheelScrollSensitivity, l4 = e4.deltaX * this._options.mouseWheelScrollSensitivity;
          this._options.scrollPredominantAxis && (this._options.scrollYToX && l4 + o6 === 0 ? l4 = o6 = 0 : Math.abs(o6) >= Math.abs(l4) ? l4 = 0 : o6 = 0), this._options.flipAxes && ([o6, l4] = [l4, o6]);
          let a3 = !Te && e4.browserEvent && e4.browserEvent.shiftKey;
          (this._options.scrollYToX || a3) && !l4 && (l4 = o6, o6 = 0), e4.browserEvent && e4.browserEvent.altKey && (l4 = l4 * this._options.fastScrollSensitivity, o6 = o6 * this._options.fastScrollSensitivity);
          let u3 = this._scrollable.getFutureScrollPosition(), h4 = {};
          if (o6) {
            let c4 = Ko * o6, d3 = u3.scrollTop - (c4 < 0 ? Math.floor(c4) : Math.ceil(c4));
            this._verticalScrollbar.writeScrollPosition(h4, d3);
          }
          if (l4) {
            let c4 = Ko * l4, d3 = u3.scrollLeft - (c4 < 0 ? Math.floor(c4) : Math.ceil(c4));
            this._horizontalScrollbar.writeScrollPosition(h4, d3);
          }
          h4 = this._scrollable.validateScrollPosition(h4), (u3.scrollLeft !== h4.scrollLeft || u3.scrollTop !== h4.scrollTop) && (zo && this._options.mouseWheelSmoothScroll && i12.isPhysicalMouseWheel() ? this._scrollable.setScrollPositionSmooth(h4) : this._scrollable.setScrollPositionNow(h4), r7 = true);
        }
        let n4 = r7;
        !n4 && this._options.alwaysConsumeMouseWheel && (n4 = true), !n4 && this._options.consumeMouseWheelIfScrollbarIsNeeded && (this._verticalScrollbar.isNeeded() || this._horizontalScrollbar.isNeeded()) && (n4 = true), n4 && (e4.preventDefault(), e4.stopPropagation());
      }
      _onDidScroll(e4) {
        this._shouldRender = this._horizontalScrollbar.onDidScroll(e4) || this._shouldRender, this._shouldRender = this._verticalScrollbar.onDidScroll(e4) || this._shouldRender, this._options.useShadows && (this._shouldRender = true), this._revealOnScroll && this._reveal(), this._options.lazyRender || this._render();
      }
      renderNow() {
        if (!this._options.lazyRender) throw new Error("Please use `lazyRender` together with `renderNow`!");
        this._render();
      }
      _render() {
        if (this._shouldRender && (this._shouldRender = false, this._horizontalScrollbar.render(), this._verticalScrollbar.render(), this._options.useShadows)) {
          let e4 = this._scrollable.getCurrentScrollPosition(), i12 = e4.scrollTop > 0, r7 = e4.scrollLeft > 0, n4 = r7 ? " left" : "", o6 = i12 ? " top" : "", l4 = r7 || i12 ? " top-left-corner" : "";
          this._leftShadowDomNode.setClassName(`shadow${n4}`), this._topShadowDomNode.setClassName(`shadow${o6}`), this._topLeftShadowDomNode.setClassName(`shadow${l4}${o6}${n4}`);
        }
      }
      _onDragStart() {
        this._isDragging = true, this._reveal();
      }
      _onDragEnd() {
        this._isDragging = false, this._hide();
      }
      _onMouseLeave(e4) {
        this._mouseIsOver = false, this._hide();
      }
      _onMouseOver(e4) {
        this._mouseIsOver = true, this._reveal();
      }
      _reveal() {
        this._verticalScrollbar.beginReveal(), this._horizontalScrollbar.beginReveal(), this._scheduleHide();
      }
      _hide() {
        !this._mouseIsOver && !this._isDragging && (this._verticalScrollbar.beginHide(), this._horizontalScrollbar.beginHide());
      }
      _scheduleHide() {
        !this._mouseIsOver && !this._isDragging && this._hideTimeout.cancelAndSet(() => this._hide(), Ma);
      }
    };
    Kr = class extends ds {
      constructor(t3, e4, i12) {
        super(t3, e4, i12);
      }
      setScrollPosition(t3) {
        t3.reuseAnimation ? this._scrollable.setScrollPositionSmooth(t3, t3.reuseAnimation) : this._scrollable.setScrollPositionNow(t3);
      }
      getScrollPosition() {
        return this._scrollable.getCurrentScrollPosition();
      }
    };
    zt = class extends D2 {
      constructor(e4, i12, r7, n4, o6, l4, a3, u3) {
        super();
        this._bufferService = r7;
        this._optionsService = a3;
        this._renderService = u3;
        this._onRequestScrollLines = this._register(new v2());
        this.onRequestScrollLines = this._onRequestScrollLines.event;
        this._isSyncing = false;
        this._isHandlingScroll = false;
        this._suppressOnScrollHandler = false;
        let h4 = this._register(new Ri({ forceIntegerValues: false, smoothScrollDuration: this._optionsService.rawOptions.smoothScrollDuration, scheduleAtNextAnimationFrame: (c4) => mt(n4.window, c4) }));
        this._register(this._optionsService.onSpecificOptionChange("smoothScrollDuration", () => {
          h4.setSmoothScrollDuration(this._optionsService.rawOptions.smoothScrollDuration);
        })), this._scrollableElement = this._register(new Kr(i12, { vertical: 1, horizontal: 2, useShadows: false, mouseWheelSmoothScroll: true, ...this._getChangeOptions() }, h4)), this._register(this._optionsService.onMultipleOptionChange(["scrollSensitivity", "fastScrollSensitivity", "overviewRuler"], () => this._scrollableElement.updateOptions(this._getChangeOptions()))), this._register(o6.onProtocolChange((c4) => {
          this._scrollableElement.updateOptions({ handleMouseWheel: !(c4 & 16) });
        })), this._scrollableElement.setScrollDimensions({ height: 0, scrollHeight: 0 }), this._register($2.runAndSubscribe(l4.onChangeColors, () => {
          this._scrollableElement.getDomNode().style.backgroundColor = l4.colors.background.css;
        })), e4.appendChild(this._scrollableElement.getDomNode()), this._register(C2(() => this._scrollableElement.getDomNode().remove())), this._styleElement = n4.mainDocument.createElement("style"), i12.appendChild(this._styleElement), this._register(C2(() => this._styleElement.remove())), this._register($2.runAndSubscribe(l4.onChangeColors, () => {
          this._styleElement.textContent = [".xterm .xterm-scrollable-element > .scrollbar > .slider {", `  background: ${l4.colors.scrollbarSliderBackground.css};`, "}", ".xterm .xterm-scrollable-element > .scrollbar > .slider:hover {", `  background: ${l4.colors.scrollbarSliderHoverBackground.css};`, "}", ".xterm .xterm-scrollable-element > .scrollbar > .slider.active {", `  background: ${l4.colors.scrollbarSliderActiveBackground.css};`, "}"].join(`
`);
        })), this._register(this._bufferService.onResize(() => this.queueSync())), this._register(this._bufferService.buffers.onBufferActivate(() => {
          this._latestYDisp = void 0, this.queueSync();
        })), this._register(this._bufferService.onScroll(() => this._sync())), this._register(this._scrollableElement.onScroll((c4) => this._handleScroll(c4)));
      }
      scrollLines(e4) {
        let i12 = this._scrollableElement.getScrollPosition();
        this._scrollableElement.setScrollPosition({ reuseAnimation: true, scrollTop: i12.scrollTop + e4 * this._renderService.dimensions.css.cell.height });
      }
      scrollToLine(e4, i12) {
        i12 && (this._latestYDisp = e4), this._scrollableElement.setScrollPosition({ reuseAnimation: !i12, scrollTop: e4 * this._renderService.dimensions.css.cell.height });
      }
      _getChangeOptions() {
        return { mouseWheelScrollSensitivity: this._optionsService.rawOptions.scrollSensitivity, fastScrollSensitivity: this._optionsService.rawOptions.fastScrollSensitivity, verticalScrollbarSize: this._optionsService.rawOptions.overviewRuler?.width || 14 };
      }
      queueSync(e4) {
        e4 !== void 0 && (this._latestYDisp = e4), this._queuedAnimationFrame === void 0 && (this._queuedAnimationFrame = this._renderService.addRefreshCallback(() => {
          this._queuedAnimationFrame = void 0, this._sync(this._latestYDisp);
        }));
      }
      _sync(e4 = this._bufferService.buffer.ydisp) {
        !this._renderService || this._isSyncing || (this._isSyncing = true, this._suppressOnScrollHandler = true, this._scrollableElement.setScrollDimensions({ height: this._renderService.dimensions.css.canvas.height, scrollHeight: this._renderService.dimensions.css.cell.height * this._bufferService.buffer.lines.length }), this._suppressOnScrollHandler = false, e4 !== this._latestYDisp && this._scrollableElement.setScrollPosition({ scrollTop: e4 * this._renderService.dimensions.css.cell.height }), this._isSyncing = false);
      }
      _handleScroll(e4) {
        if (!this._renderService || this._isHandlingScroll || this._suppressOnScrollHandler) return;
        this._isHandlingScroll = true;
        let i12 = Math.round(e4.scrollTop / this._renderService.dimensions.css.cell.height), r7 = i12 - this._bufferService.buffer.ydisp;
        r7 !== 0 && (this._latestYDisp = i12, this._onRequestScrollLines.fire(r7)), this._isHandlingScroll = false;
      }
    };
    zt = M2([S3(2, F), S3(3, ae), S3(4, rr), S3(5, Re), S3(6, H2), S3(7, ce)], zt);
    Gt = class extends D2 {
      constructor(e4, i12, r7, n4, o6) {
        super();
        this._screenElement = e4;
        this._bufferService = i12;
        this._coreBrowserService = r7;
        this._decorationService = n4;
        this._renderService = o6;
        this._decorationElements = /* @__PURE__ */ new Map();
        this._altBufferIsActive = false;
        this._dimensionsChanged = false;
        this._container = document.createElement("div"), this._container.classList.add("xterm-decoration-container"), this._screenElement.appendChild(this._container), this._register(this._renderService.onRenderedViewportChange(() => this._doRefreshDecorations())), this._register(this._renderService.onDimensionsChange(() => {
          this._dimensionsChanged = true, this._queueRefresh();
        })), this._register(this._coreBrowserService.onDprChange(() => this._queueRefresh())), this._register(this._bufferService.buffers.onBufferActivate(() => {
          this._altBufferIsActive = this._bufferService.buffer === this._bufferService.buffers.alt;
        })), this._register(this._decorationService.onDecorationRegistered(() => this._queueRefresh())), this._register(this._decorationService.onDecorationRemoved((l4) => this._removeDecoration(l4))), this._register(C2(() => {
          this._container.remove(), this._decorationElements.clear();
        }));
      }
      _queueRefresh() {
        this._animationFrame === void 0 && (this._animationFrame = this._renderService.addRefreshCallback(() => {
          this._doRefreshDecorations(), this._animationFrame = void 0;
        }));
      }
      _doRefreshDecorations() {
        for (let e4 of this._decorationService.decorations) this._renderDecoration(e4);
        this._dimensionsChanged = false;
      }
      _renderDecoration(e4) {
        this._refreshStyle(e4), this._dimensionsChanged && this._refreshXPosition(e4);
      }
      _createElement(e4) {
        let i12 = this._coreBrowserService.mainDocument.createElement("div");
        i12.classList.add("xterm-decoration"), i12.classList.toggle("xterm-decoration-top-layer", e4?.options?.layer === "top"), i12.style.width = `${Math.round((e4.options.width || 1) * this._renderService.dimensions.css.cell.width)}px`, i12.style.height = `${(e4.options.height || 1) * this._renderService.dimensions.css.cell.height}px`, i12.style.top = `${(e4.marker.line - this._bufferService.buffers.active.ydisp) * this._renderService.dimensions.css.cell.height}px`, i12.style.lineHeight = `${this._renderService.dimensions.css.cell.height}px`;
        let r7 = e4.options.x ?? 0;
        return r7 && r7 > this._bufferService.cols && (i12.style.display = "none"), this._refreshXPosition(e4, i12), i12;
      }
      _refreshStyle(e4) {
        let i12 = e4.marker.line - this._bufferService.buffers.active.ydisp;
        if (i12 < 0 || i12 >= this._bufferService.rows) e4.element && (e4.element.style.display = "none", e4.onRenderEmitter.fire(e4.element));
        else {
          let r7 = this._decorationElements.get(e4);
          r7 || (r7 = this._createElement(e4), e4.element = r7, this._decorationElements.set(e4, r7), this._container.appendChild(r7), e4.onDispose(() => {
            this._decorationElements.delete(e4), r7.remove();
          })), r7.style.display = this._altBufferIsActive ? "none" : "block", this._altBufferIsActive || (r7.style.width = `${Math.round((e4.options.width || 1) * this._renderService.dimensions.css.cell.width)}px`, r7.style.height = `${(e4.options.height || 1) * this._renderService.dimensions.css.cell.height}px`, r7.style.top = `${i12 * this._renderService.dimensions.css.cell.height}px`, r7.style.lineHeight = `${this._renderService.dimensions.css.cell.height}px`), e4.onRenderEmitter.fire(r7);
        }
      }
      _refreshXPosition(e4, i12 = e4.element) {
        if (!i12) return;
        let r7 = e4.options.x ?? 0;
        (e4.options.anchor || "left") === "right" ? i12.style.right = r7 ? `${r7 * this._renderService.dimensions.css.cell.width}px` : "" : i12.style.left = r7 ? `${r7 * this._renderService.dimensions.css.cell.width}px` : "";
      }
      _removeDecoration(e4) {
        this._decorationElements.get(e4)?.remove(), this._decorationElements.delete(e4), e4.dispose();
      }
    };
    Gt = M2([S3(1, F), S3(2, ae), S3(3, Be), S3(4, ce)], Gt);
    Gr = class {
      constructor() {
        this._zones = [];
        this._zonePool = [];
        this._zonePoolIndex = 0;
        this._linePadding = { full: 0, left: 0, center: 0, right: 0 };
      }
      get zones() {
        return this._zonePool.length = Math.min(this._zonePool.length, this._zones.length), this._zones;
      }
      clear() {
        this._zones.length = 0, this._zonePoolIndex = 0;
      }
      addDecoration(t3) {
        if (t3.options.overviewRulerOptions) {
          for (let e4 of this._zones) if (e4.color === t3.options.overviewRulerOptions.color && e4.position === t3.options.overviewRulerOptions.position) {
            if (this._lineIntersectsZone(e4, t3.marker.line)) return;
            if (this._lineAdjacentToZone(e4, t3.marker.line, t3.options.overviewRulerOptions.position)) {
              this._addLineToZone(e4, t3.marker.line);
              return;
            }
          }
          if (this._zonePoolIndex < this._zonePool.length) {
            this._zonePool[this._zonePoolIndex].color = t3.options.overviewRulerOptions.color, this._zonePool[this._zonePoolIndex].position = t3.options.overviewRulerOptions.position, this._zonePool[this._zonePoolIndex].startBufferLine = t3.marker.line, this._zonePool[this._zonePoolIndex].endBufferLine = t3.marker.line, this._zones.push(this._zonePool[this._zonePoolIndex++]);
            return;
          }
          this._zones.push({ color: t3.options.overviewRulerOptions.color, position: t3.options.overviewRulerOptions.position, startBufferLine: t3.marker.line, endBufferLine: t3.marker.line }), this._zonePool.push(this._zones[this._zones.length - 1]), this._zonePoolIndex++;
        }
      }
      setPadding(t3) {
        this._linePadding = t3;
      }
      _lineIntersectsZone(t3, e4) {
        return e4 >= t3.startBufferLine && e4 <= t3.endBufferLine;
      }
      _lineAdjacentToZone(t3, e4, i12) {
        return e4 >= t3.startBufferLine - this._linePadding[i12 || "full"] && e4 <= t3.endBufferLine + this._linePadding[i12 || "full"];
      }
      _addLineToZone(t3, e4) {
        t3.startBufferLine = Math.min(t3.startBufferLine, e4), t3.endBufferLine = Math.max(t3.endBufferLine, e4);
      }
    };
    We = { full: 0, left: 0, center: 0, right: 0 };
    at = { full: 0, left: 0, center: 0, right: 0 };
    Li = { full: 0, left: 0, center: 0, right: 0 };
    bt = class extends D2 {
      constructor(e4, i12, r7, n4, o6, l4, a3, u3) {
        super();
        this._viewportElement = e4;
        this._screenElement = i12;
        this._bufferService = r7;
        this._decorationService = n4;
        this._renderService = o6;
        this._optionsService = l4;
        this._themeService = a3;
        this._coreBrowserService = u3;
        this._colorZoneStore = new Gr();
        this._shouldUpdateDimensions = true;
        this._shouldUpdateAnchor = true;
        this._lastKnownBufferLength = 0;
        this._canvas = this._coreBrowserService.mainDocument.createElement("canvas"), this._canvas.classList.add("xterm-decoration-overview-ruler"), this._refreshCanvasDimensions(), this._viewportElement.parentElement?.insertBefore(this._canvas, this._viewportElement), this._register(C2(() => this._canvas?.remove()));
        let h4 = this._canvas.getContext("2d");
        if (h4) this._ctx = h4;
        else throw new Error("Ctx cannot be null");
        this._register(this._decorationService.onDecorationRegistered(() => this._queueRefresh(void 0, true))), this._register(this._decorationService.onDecorationRemoved(() => this._queueRefresh(void 0, true))), this._register(this._renderService.onRenderedViewportChange(() => this._queueRefresh())), this._register(this._bufferService.buffers.onBufferActivate(() => {
          this._canvas.style.display = this._bufferService.buffer === this._bufferService.buffers.alt ? "none" : "block";
        })), this._register(this._bufferService.onScroll(() => {
          this._lastKnownBufferLength !== this._bufferService.buffers.normal.lines.length && (this._refreshDrawHeightConstants(), this._refreshColorZonePadding());
        })), this._register(this._renderService.onRender(() => {
          (!this._containerHeight || this._containerHeight !== this._screenElement.clientHeight) && (this._queueRefresh(true), this._containerHeight = this._screenElement.clientHeight);
        })), this._register(this._coreBrowserService.onDprChange(() => this._queueRefresh(true))), this._register(this._optionsService.onSpecificOptionChange("overviewRuler", () => this._queueRefresh(true))), this._register(this._themeService.onChangeColors(() => this._queueRefresh())), this._queueRefresh(true);
      }
      get _width() {
        return this._optionsService.options.overviewRuler?.width || 0;
      }
      _refreshDrawConstants() {
        let e4 = Math.floor((this._canvas.width - 1) / 3), i12 = Math.ceil((this._canvas.width - 1) / 3);
        at.full = this._canvas.width, at.left = e4, at.center = i12, at.right = e4, this._refreshDrawHeightConstants(), Li.full = 1, Li.left = 1, Li.center = 1 + at.left, Li.right = 1 + at.left + at.center;
      }
      _refreshDrawHeightConstants() {
        We.full = Math.round(2 * this._coreBrowserService.dpr);
        let e4 = this._canvas.height / this._bufferService.buffer.lines.length, i12 = Math.round(Math.max(Math.min(e4, 12), 6) * this._coreBrowserService.dpr);
        We.left = i12, We.center = i12, We.right = i12;
      }
      _refreshColorZonePadding() {
        this._colorZoneStore.setPadding({ full: Math.floor(this._bufferService.buffers.active.lines.length / (this._canvas.height - 1) * We.full), left: Math.floor(this._bufferService.buffers.active.lines.length / (this._canvas.height - 1) * We.left), center: Math.floor(this._bufferService.buffers.active.lines.length / (this._canvas.height - 1) * We.center), right: Math.floor(this._bufferService.buffers.active.lines.length / (this._canvas.height - 1) * We.right) }), this._lastKnownBufferLength = this._bufferService.buffers.normal.lines.length;
      }
      _refreshCanvasDimensions() {
        this._canvas.style.width = `${this._width}px`, this._canvas.width = Math.round(this._width * this._coreBrowserService.dpr), this._canvas.style.height = `${this._screenElement.clientHeight}px`, this._canvas.height = Math.round(this._screenElement.clientHeight * this._coreBrowserService.dpr), this._refreshDrawConstants(), this._refreshColorZonePadding();
      }
      _refreshDecorations() {
        this._shouldUpdateDimensions && this._refreshCanvasDimensions(), this._ctx.clearRect(0, 0, this._canvas.width, this._canvas.height), this._colorZoneStore.clear();
        for (let i12 of this._decorationService.decorations) this._colorZoneStore.addDecoration(i12);
        this._ctx.lineWidth = 1, this._renderRulerOutline();
        let e4 = this._colorZoneStore.zones;
        for (let i12 of e4) i12.position !== "full" && this._renderColorZone(i12);
        for (let i12 of e4) i12.position === "full" && this._renderColorZone(i12);
        this._shouldUpdateDimensions = false, this._shouldUpdateAnchor = false;
      }
      _renderRulerOutline() {
        this._ctx.fillStyle = this._themeService.colors.overviewRulerBorder.css, this._ctx.fillRect(0, 0, 1, this._canvas.height), this._optionsService.rawOptions.overviewRuler.showTopBorder && this._ctx.fillRect(1, 0, this._canvas.width - 1, 1), this._optionsService.rawOptions.overviewRuler.showBottomBorder && this._ctx.fillRect(1, this._canvas.height - 1, this._canvas.width - 1, this._canvas.height);
      }
      _renderColorZone(e4) {
        this._ctx.fillStyle = e4.color, this._ctx.fillRect(Li[e4.position || "full"], Math.round((this._canvas.height - 1) * (e4.startBufferLine / this._bufferService.buffers.active.lines.length) - We[e4.position || "full"] / 2), at[e4.position || "full"], Math.round((this._canvas.height - 1) * ((e4.endBufferLine - e4.startBufferLine) / this._bufferService.buffers.active.lines.length) + We[e4.position || "full"]));
      }
      _queueRefresh(e4, i12) {
        this._shouldUpdateDimensions = e4 || this._shouldUpdateDimensions, this._shouldUpdateAnchor = i12 || this._shouldUpdateAnchor, this._animationFrame === void 0 && (this._animationFrame = this._coreBrowserService.window.requestAnimationFrame(() => {
          this._refreshDecorations(), this._animationFrame = void 0;
        }));
      }
    };
    bt = M2([S3(2, F), S3(3, Be), S3(4, ce), S3(5, H2), S3(6, Re), S3(7, ae)], bt);
    ((E2) => (E2.NUL = "\0", E2.SOH = "", E2.STX = "", E2.ETX = "", E2.EOT = "", E2.ENQ = "", E2.ACK = "", E2.BEL = "\x07", E2.BS = "\b", E2.HT = "	", E2.LF = `
`, E2.VT = "\v", E2.FF = "\f", E2.CR = "\r", E2.SO = "", E2.SI = "", E2.DLE = "", E2.DC1 = "", E2.DC2 = "", E2.DC3 = "", E2.DC4 = "", E2.NAK = "", E2.SYN = "", E2.ETB = "", E2.CAN = "", E2.EM = "", E2.SUB = "", E2.ESC = "\x1B", E2.FS = "", E2.GS = "", E2.RS = "", E2.US = "", E2.SP = " ", E2.DEL = "\x7F"))(b3 || (b3 = {}));
    ((g3) => (g3.PAD = "\x80", g3.HOP = "\x81", g3.BPH = "\x82", g3.NBH = "\x83", g3.IND = "\x84", g3.NEL = "\x85", g3.SSA = "\x86", g3.ESA = "\x87", g3.HTS = "\x88", g3.HTJ = "\x89", g3.VTS = "\x8A", g3.PLD = "\x8B", g3.PLU = "\x8C", g3.RI = "\x8D", g3.SS2 = "\x8E", g3.SS3 = "\x8F", g3.DCS = "\x90", g3.PU1 = "\x91", g3.PU2 = "\x92", g3.STS = "\x93", g3.CCH = "\x94", g3.MW = "\x95", g3.SPA = "\x96", g3.EPA = "\x97", g3.SOS = "\x98", g3.SGCI = "\x99", g3.SCI = "\x9A", g3.CSI = "\x9B", g3.ST = "\x9C", g3.OSC = "\x9D", g3.PM = "\x9E", g3.APC = "\x9F"))(Ai || (Ai = {}));
    ((t3) => t3.ST = `${b3.ESC}\\`)(fs || (fs = {}));
    $t = class {
      constructor(t3, e4, i12, r7, n4, o6) {
        this._textarea = t3;
        this._compositionView = e4;
        this._bufferService = i12;
        this._optionsService = r7;
        this._coreService = n4;
        this._renderService = o6;
        this._isComposing = false, this._isSendingComposition = false, this._compositionPosition = { start: 0, end: 0 }, this._dataAlreadySent = "";
      }
      get isComposing() {
        return this._isComposing;
      }
      compositionstart() {
        this._isComposing = true, this._compositionPosition.start = this._textarea.value.length, this._compositionView.textContent = "", this._dataAlreadySent = "", this._compositionView.classList.add("active");
      }
      compositionupdate(t3) {
        this._compositionView.textContent = t3.data, this.updateCompositionElements(), setTimeout(() => {
          this._compositionPosition.end = this._textarea.value.length;
        }, 0);
      }
      compositionend() {
        this._finalizeComposition(true);
      }
      keydown(t3) {
        if (this._isComposing || this._isSendingComposition) {
          if (t3.keyCode === 20 || t3.keyCode === 229 || t3.keyCode === 16 || t3.keyCode === 17 || t3.keyCode === 18) return false;
          this._finalizeComposition(false);
        }
        return t3.keyCode === 229 ? (this._handleAnyTextareaChanges(), false) : true;
      }
      _finalizeComposition(t3) {
        if (this._compositionView.classList.remove("active"), this._isComposing = false, t3) {
          let e4 = { start: this._compositionPosition.start, end: this._compositionPosition.end };
          this._isSendingComposition = true, setTimeout(() => {
            if (this._isSendingComposition) {
              this._isSendingComposition = false;
              let i12;
              e4.start += this._dataAlreadySent.length, this._isComposing ? i12 = this._textarea.value.substring(e4.start, this._compositionPosition.start) : i12 = this._textarea.value.substring(e4.start), i12.length > 0 && this._coreService.triggerDataEvent(i12, true);
            }
          }, 0);
        } else {
          this._isSendingComposition = false;
          let e4 = this._textarea.value.substring(this._compositionPosition.start, this._compositionPosition.end);
          this._coreService.triggerDataEvent(e4, true);
        }
      }
      _handleAnyTextareaChanges() {
        let t3 = this._textarea.value;
        setTimeout(() => {
          if (!this._isComposing) {
            let e4 = this._textarea.value, i12 = e4.replace(t3, "");
            this._dataAlreadySent = i12, e4.length > t3.length ? this._coreService.triggerDataEvent(i12, true) : e4.length < t3.length ? this._coreService.triggerDataEvent(`${b3.DEL}`, true) : e4.length === t3.length && e4 !== t3 && this._coreService.triggerDataEvent(e4, true);
          }
        }, 0);
      }
      updateCompositionElements(t3) {
        if (this._isComposing) {
          if (this._bufferService.buffer.isCursorInViewport) {
            let e4 = Math.min(this._bufferService.buffer.x, this._bufferService.cols - 1), i12 = this._renderService.dimensions.css.cell.height, r7 = this._bufferService.buffer.y * this._renderService.dimensions.css.cell.height, n4 = e4 * this._renderService.dimensions.css.cell.width;
            this._compositionView.style.left = n4 + "px", this._compositionView.style.top = r7 + "px", this._compositionView.style.height = i12 + "px", this._compositionView.style.lineHeight = i12 + "px", this._compositionView.style.fontFamily = this._optionsService.rawOptions.fontFamily, this._compositionView.style.fontSize = this._optionsService.rawOptions.fontSize + "px";
            let o6 = this._compositionView.getBoundingClientRect();
            this._textarea.style.left = n4 + "px", this._textarea.style.top = r7 + "px", this._textarea.style.width = Math.max(o6.width, 1) + "px", this._textarea.style.height = Math.max(o6.height, 1) + "px", this._textarea.style.lineHeight = o6.height + "px";
          }
          t3 || setTimeout(() => this.updateCompositionElements(true), 0);
        }
      }
    };
    $t = M2([S3(2, F), S3(3, H2), S3(4, ge), S3(5, ce)], $t);
    ue = 0;
    he = 0;
    de = 0;
    J = 0;
    ps = { css: "#00000000", rgba: 0 };
    ((i12) => {
      function s18(r7, n4, o6, l4) {
        return l4 !== void 0 ? `#${vt(r7)}${vt(n4)}${vt(o6)}${vt(l4)}` : `#${vt(r7)}${vt(n4)}${vt(o6)}`;
      }
      i12.toCss = s18;
      function t3(r7, n4, o6, l4 = 255) {
        return (r7 << 24 | n4 << 16 | o6 << 8 | l4) >>> 0;
      }
      i12.toRgba = t3;
      function e4(r7, n4, o6, l4) {
        return { css: i12.toCss(r7, n4, o6, l4), rgba: i12.toRgba(r7, n4, o6, l4) };
      }
      i12.toColor = e4;
    })(j || (j = {}));
    ((l4) => {
      function s18(a3, u3) {
        if (J = (u3.rgba & 255) / 255, J === 1) return { css: u3.css, rgba: u3.rgba };
        let h4 = u3.rgba >> 24 & 255, c4 = u3.rgba >> 16 & 255, d3 = u3.rgba >> 8 & 255, _4 = a3.rgba >> 24 & 255, p3 = a3.rgba >> 16 & 255, m2 = a3.rgba >> 8 & 255;
        ue = _4 + Math.round((h4 - _4) * J), he = p3 + Math.round((c4 - p3) * J), de = m2 + Math.round((d3 - m2) * J);
        let f3 = j.toCss(ue, he, de), A3 = j.toRgba(ue, he, de);
        return { css: f3, rgba: A3 };
      }
      l4.blend = s18;
      function t3(a3) {
        return (a3.rgba & 255) === 255;
      }
      l4.isOpaque = t3;
      function e4(a3, u3, h4) {
        let c4 = $r.ensureContrastRatio(a3.rgba, u3.rgba, h4);
        if (c4) return j.toColor(c4 >> 24 & 255, c4 >> 16 & 255, c4 >> 8 & 255);
      }
      l4.ensureContrastRatio = e4;
      function i12(a3) {
        let u3 = (a3.rgba | 255) >>> 0;
        return [ue, he, de] = $r.toChannels(u3), { css: j.toCss(ue, he, de), rgba: u3 };
      }
      l4.opaque = i12;
      function r7(a3, u3) {
        return J = Math.round(u3 * 255), [ue, he, de] = $r.toChannels(a3.rgba), { css: j.toCss(ue, he, de, J), rgba: j.toRgba(ue, he, de, J) };
      }
      l4.opacity = r7;
      function n4(a3, u3) {
        return J = a3.rgba & 255, r7(a3, J * u3 / 255);
      }
      l4.multiplyOpacity = n4;
      function o6(a3) {
        return [a3.rgba >> 24 & 255, a3.rgba >> 16 & 255, a3.rgba >> 8 & 255];
      }
      l4.toColorRGB = o6;
    })(U || (U = {}));
    ((i12) => {
      let s18, t3;
      try {
        let r7 = document.createElement("canvas");
        r7.width = 1, r7.height = 1;
        let n4 = r7.getContext("2d", { willReadFrequently: true });
        n4 && (s18 = n4, s18.globalCompositeOperation = "copy", t3 = s18.createLinearGradient(0, 0, 1, 1));
      } catch {
      }
      function e4(r7) {
        if (r7.match(/#[\da-f]{3,8}/i)) switch (r7.length) {
          case 4:
            return ue = parseInt(r7.slice(1, 2).repeat(2), 16), he = parseInt(r7.slice(2, 3).repeat(2), 16), de = parseInt(r7.slice(3, 4).repeat(2), 16), j.toColor(ue, he, de);
          case 5:
            return ue = parseInt(r7.slice(1, 2).repeat(2), 16), he = parseInt(r7.slice(2, 3).repeat(2), 16), de = parseInt(r7.slice(3, 4).repeat(2), 16), J = parseInt(r7.slice(4, 5).repeat(2), 16), j.toColor(ue, he, de, J);
          case 7:
            return { css: r7, rgba: (parseInt(r7.slice(1), 16) << 8 | 255) >>> 0 };
          case 9:
            return { css: r7, rgba: parseInt(r7.slice(1), 16) >>> 0 };
        }
        let n4 = r7.match(/rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*(,\s*(0|1|\d?\.(\d+))\s*)?\)/);
        if (n4) return ue = parseInt(n4[1]), he = parseInt(n4[2]), de = parseInt(n4[3]), J = Math.round((n4[5] === void 0 ? 1 : parseFloat(n4[5])) * 255), j.toColor(ue, he, de, J);
        if (!s18 || !t3) throw new Error("css.toColor: Unsupported css format");
        if (s18.fillStyle = t3, s18.fillStyle = r7, typeof s18.fillStyle != "string") throw new Error("css.toColor: Unsupported css format");
        if (s18.fillRect(0, 0, 1, 1), [ue, he, de, J] = s18.getImageData(0, 0, 1, 1).data, J !== 255) throw new Error("css.toColor: Unsupported css format");
        return { rgba: j.toRgba(ue, he, de, J), css: r7 };
      }
      i12.toColor = e4;
    })(z2 || (z2 = {}));
    ((e4) => {
      function s18(i12) {
        return t3(i12 >> 16 & 255, i12 >> 8 & 255, i12 & 255);
      }
      e4.relativeLuminance = s18;
      function t3(i12, r7, n4) {
        let o6 = i12 / 255, l4 = r7 / 255, a3 = n4 / 255, u3 = o6 <= 0.03928 ? o6 / 12.92 : Math.pow((o6 + 0.055) / 1.055, 2.4), h4 = l4 <= 0.03928 ? l4 / 12.92 : Math.pow((l4 + 0.055) / 1.055, 2.4), c4 = a3 <= 0.03928 ? a3 / 12.92 : Math.pow((a3 + 0.055) / 1.055, 2.4);
        return u3 * 0.2126 + h4 * 0.7152 + c4 * 0.0722;
      }
      e4.relativeLuminance2 = t3;
    })(ve || (ve = {}));
    ((n4) => {
      function s18(o6, l4) {
        if (J = (l4 & 255) / 255, J === 1) return l4;
        let a3 = l4 >> 24 & 255, u3 = l4 >> 16 & 255, h4 = l4 >> 8 & 255, c4 = o6 >> 24 & 255, d3 = o6 >> 16 & 255, _4 = o6 >> 8 & 255;
        return ue = c4 + Math.round((a3 - c4) * J), he = d3 + Math.round((u3 - d3) * J), de = _4 + Math.round((h4 - _4) * J), j.toRgba(ue, he, de);
      }
      n4.blend = s18;
      function t3(o6, l4, a3) {
        let u3 = ve.relativeLuminance(o6 >> 8), h4 = ve.relativeLuminance(l4 >> 8);
        if (Xe(u3, h4) < a3) {
          if (h4 < u3) {
            let p3 = e4(o6, l4, a3), m2 = Xe(u3, ve.relativeLuminance(p3 >> 8));
            if (m2 < a3) {
              let f3 = i12(o6, l4, a3), A3 = Xe(u3, ve.relativeLuminance(f3 >> 8));
              return m2 > A3 ? p3 : f3;
            }
            return p3;
          }
          let d3 = i12(o6, l4, a3), _4 = Xe(u3, ve.relativeLuminance(d3 >> 8));
          if (_4 < a3) {
            let p3 = e4(o6, l4, a3), m2 = Xe(u3, ve.relativeLuminance(p3 >> 8));
            return _4 > m2 ? d3 : p3;
          }
          return d3;
        }
      }
      n4.ensureContrastRatio = t3;
      function e4(o6, l4, a3) {
        let u3 = o6 >> 24 & 255, h4 = o6 >> 16 & 255, c4 = o6 >> 8 & 255, d3 = l4 >> 24 & 255, _4 = l4 >> 16 & 255, p3 = l4 >> 8 & 255, m2 = Xe(ve.relativeLuminance2(d3, _4, p3), ve.relativeLuminance2(u3, h4, c4));
        for (; m2 < a3 && (d3 > 0 || _4 > 0 || p3 > 0); ) d3 -= Math.max(0, Math.ceil(d3 * 0.1)), _4 -= Math.max(0, Math.ceil(_4 * 0.1)), p3 -= Math.max(0, Math.ceil(p3 * 0.1)), m2 = Xe(ve.relativeLuminance2(d3, _4, p3), ve.relativeLuminance2(u3, h4, c4));
        return (d3 << 24 | _4 << 16 | p3 << 8 | 255) >>> 0;
      }
      n4.reduceLuminance = e4;
      function i12(o6, l4, a3) {
        let u3 = o6 >> 24 & 255, h4 = o6 >> 16 & 255, c4 = o6 >> 8 & 255, d3 = l4 >> 24 & 255, _4 = l4 >> 16 & 255, p3 = l4 >> 8 & 255, m2 = Xe(ve.relativeLuminance2(d3, _4, p3), ve.relativeLuminance2(u3, h4, c4));
        for (; m2 < a3 && (d3 < 255 || _4 < 255 || p3 < 255); ) d3 = Math.min(255, d3 + Math.ceil((255 - d3) * 0.1)), _4 = Math.min(255, _4 + Math.ceil((255 - _4) * 0.1)), p3 = Math.min(255, p3 + Math.ceil((255 - p3) * 0.1)), m2 = Xe(ve.relativeLuminance2(d3, _4, p3), ve.relativeLuminance2(u3, h4, c4));
        return (d3 << 24 | _4 << 16 | p3 << 8 | 255) >>> 0;
      }
      n4.increaseLuminance = i12;
      function r7(o6) {
        return [o6 >> 24 & 255, o6 >> 16 & 255, o6 >> 8 & 255, o6 & 255];
      }
      n4.toChannels = r7;
    })($r || ($r = {}));
    Vr = class extends De {
      constructor(e4, i12, r7) {
        super();
        this.content = 0;
        this.combinedData = "";
        this.fg = e4.fg, this.bg = e4.bg, this.combinedData = i12, this._width = r7;
      }
      isCombined() {
        return 2097152;
      }
      getWidth() {
        return this._width;
      }
      getChars() {
        return this.combinedData;
      }
      getCode() {
        return 2097151;
      }
      setFromCharData(e4) {
        throw new Error("not implemented");
      }
      getAsCharData() {
        return [this.fg, this.getChars(), this.getWidth(), this.getCode()];
      }
    };
    ct = class {
      constructor(t3) {
        this._bufferService = t3;
        this._characterJoiners = [];
        this._nextCharacterJoinerId = 0;
        this._workCell = new q();
      }
      register(t3) {
        let e4 = { id: this._nextCharacterJoinerId++, handler: t3 };
        return this._characterJoiners.push(e4), e4.id;
      }
      deregister(t3) {
        for (let e4 = 0; e4 < this._characterJoiners.length; e4++) if (this._characterJoiners[e4].id === t3) return this._characterJoiners.splice(e4, 1), true;
        return false;
      }
      getJoinedCharacters(t3) {
        if (this._characterJoiners.length === 0) return [];
        let e4 = this._bufferService.buffer.lines.get(t3);
        if (!e4 || e4.length === 0) return [];
        let i12 = [], r7 = e4.translateToString(true), n4 = 0, o6 = 0, l4 = 0, a3 = e4.getFg(0), u3 = e4.getBg(0);
        for (let h4 = 0; h4 < e4.getTrimmedLength(); h4++) if (e4.loadCell(h4, this._workCell), this._workCell.getWidth() !== 0) {
          if (this._workCell.fg !== a3 || this._workCell.bg !== u3) {
            if (h4 - n4 > 1) {
              let c4 = this._getJoinedRanges(r7, l4, o6, e4, n4);
              for (let d3 = 0; d3 < c4.length; d3++) i12.push(c4[d3]);
            }
            n4 = h4, l4 = o6, a3 = this._workCell.fg, u3 = this._workCell.bg;
          }
          o6 += this._workCell.getChars().length || we.length;
        }
        if (this._bufferService.cols - n4 > 1) {
          let h4 = this._getJoinedRanges(r7, l4, o6, e4, n4);
          for (let c4 = 0; c4 < h4.length; c4++) i12.push(h4[c4]);
        }
        return i12;
      }
      _getJoinedRanges(t3, e4, i12, r7, n4) {
        let o6 = t3.substring(e4, i12), l4 = [];
        try {
          l4 = this._characterJoiners[0].handler(o6);
        } catch (a3) {
          console.error(a3);
        }
        for (let a3 = 1; a3 < this._characterJoiners.length; a3++) try {
          let u3 = this._characterJoiners[a3].handler(o6);
          for (let h4 = 0; h4 < u3.length; h4++) ct._mergeRanges(l4, u3[h4]);
        } catch (u3) {
          console.error(u3);
        }
        return this._stringRangesToCellRanges(l4, r7, n4), l4;
      }
      _stringRangesToCellRanges(t3, e4, i12) {
        let r7 = 0, n4 = false, o6 = 0, l4 = t3[r7];
        if (l4) {
          for (let a3 = i12; a3 < this._bufferService.cols; a3++) {
            let u3 = e4.getWidth(a3), h4 = e4.getString(a3).length || we.length;
            if (u3 !== 0) {
              if (!n4 && l4[0] <= o6 && (l4[0] = a3, n4 = true), l4[1] <= o6) {
                if (l4[1] = a3, l4 = t3[++r7], !l4) break;
                l4[0] <= o6 ? (l4[0] = a3, n4 = true) : n4 = false;
              }
              o6 += h4;
            }
          }
          l4 && (l4[1] = this._bufferService.cols);
        }
      }
      static _mergeRanges(t3, e4) {
        let i12 = false;
        for (let r7 = 0; r7 < t3.length; r7++) {
          let n4 = t3[r7];
          if (i12) {
            if (e4[1] <= n4[0]) return t3[r7 - 1][1] = e4[1], t3;
            if (e4[1] <= n4[1]) return t3[r7 - 1][1] = Math.max(e4[1], n4[1]), t3.splice(r7, 1), t3;
            t3.splice(r7, 1), r7--;
          } else {
            if (e4[1] <= n4[0]) return t3.splice(r7, 0, e4), t3;
            if (e4[1] <= n4[1]) return n4[0] = Math.min(e4[0], n4[0]), t3;
            e4[0] < n4[1] && (n4[0] = Math.min(e4[0], n4[0]), i12 = true);
            continue;
          }
        }
        return i12 ? t3[t3.length - 1][1] = e4[1] : t3.push(e4), t3;
      }
    };
    ct = M2([S3(0, F)], ct);
    Vt = class {
      constructor(t3, e4, i12, r7, n4, o6, l4) {
        this._document = t3;
        this._characterJoinerService = e4;
        this._optionsService = i12;
        this._coreBrowserService = r7;
        this._coreService = n4;
        this._decorationService = o6;
        this._themeService = l4;
        this._workCell = new q();
        this._columnSelectMode = false;
        this.defaultSpacing = 0;
      }
      handleSelectionChanged(t3, e4, i12) {
        this._selectionStart = t3, this._selectionEnd = e4, this._columnSelectMode = i12;
      }
      createRow(t3, e4, i12, r7, n4, o6, l4, a3, u3, h4, c4) {
        let d3 = [], _4 = this._characterJoinerService.getJoinedCharacters(e4), p3 = this._themeService.colors, m2 = t3.getNoBgTrimmedLength();
        i12 && m2 < o6 + 1 && (m2 = o6 + 1);
        let f3, A3 = 0, R2 = "", O3 = 0, I2 = 0, k5 = 0, P4 = 0, oe = false, Me3 = 0, Pe3 = false, Ke = 0, di = 0, V5 = [], Qe3 = h4 !== -1 && c4 !== -1;
        for (let y3 = 0; y3 < m2; y3++) {
          t3.loadCell(y3, this._workCell);
          let T2 = this._workCell.getWidth();
          if (T2 === 0) continue;
          let g3 = false, w3 = y3 >= di, E2 = y3, x2 = this._workCell;
          if (_4.length > 0 && y3 === _4[0][0] && w3) {
            let W2 = _4.shift(), An2 = this._isCellInSelection(W2[0], e4);
            for (O3 = W2[0] + 1; O3 < W2[1]; O3++) w3 && (w3 = An2 === this._isCellInSelection(O3, e4));
            w3 && (w3 = !i12 || o6 < W2[0] || o6 >= W2[1]), w3 ? (g3 = true, x2 = new Vr(this._workCell, t3.translateToString(true, W2[0], W2[1]), W2[1] - W2[0]), E2 = W2[1] - 1, T2 = x2.getWidth()) : di = W2[1];
          }
          let N3 = this._isCellInSelection(y3, e4), Z3 = i12 && y3 === o6, te3 = Qe3 && y3 >= h4 && y3 <= c4, Oe3 = false;
          this._decorationService.forEachDecorationAtCell(y3, e4, void 0, (W2) => {
            Oe3 = true;
          });
          let ze3 = x2.getChars() || we;
          if (ze3 === " " && (x2.isUnderline() || x2.isOverline()) && (ze3 = "\xA0"), Ke = T2 * a3 - u3.get(ze3, x2.isBold(), x2.isItalic()), !f3) f3 = this._document.createElement("span");
          else if (A3 && (N3 && Pe3 || !N3 && !Pe3 && x2.bg === I2) && (N3 && Pe3 && p3.selectionForeground || x2.fg === k5) && x2.extended.ext === P4 && te3 === oe && Ke === Me3 && !Z3 && !g3 && !Oe3 && w3) {
            x2.isInvisible() ? R2 += we : R2 += ze3, A3++;
            continue;
          } else A3 && (f3.textContent = R2), f3 = this._document.createElement("span"), A3 = 0, R2 = "";
          if (I2 = x2.bg, k5 = x2.fg, P4 = x2.extended.ext, oe = te3, Me3 = Ke, Pe3 = N3, g3 && o6 >= y3 && o6 <= E2 && (o6 = y3), !this._coreService.isCursorHidden && Z3 && this._coreService.isCursorInitialized) {
            if (V5.push("xterm-cursor"), this._coreBrowserService.isFocused) l4 && V5.push("xterm-cursor-blink"), V5.push(r7 === "bar" ? "xterm-cursor-bar" : r7 === "underline" ? "xterm-cursor-underline" : "xterm-cursor-block");
            else if (n4) switch (n4) {
              case "outline":
                V5.push("xterm-cursor-outline");
                break;
              case "block":
                V5.push("xterm-cursor-block");
                break;
              case "bar":
                V5.push("xterm-cursor-bar");
                break;
              case "underline":
                V5.push("xterm-cursor-underline");
                break;
              default:
                break;
            }
          }
          if (x2.isBold() && V5.push("xterm-bold"), x2.isItalic() && V5.push("xterm-italic"), x2.isDim() && V5.push("xterm-dim"), x2.isInvisible() ? R2 = we : R2 = x2.getChars() || we, x2.isUnderline() && (V5.push(`xterm-underline-${x2.extended.underlineStyle}`), R2 === " " && (R2 = "\xA0"), !x2.isUnderlineColorDefault())) if (x2.isUnderlineColorRGB()) f3.style.textDecorationColor = `rgb(${De.toColorRGB(x2.getUnderlineColor()).join(",")})`;
          else {
            let W2 = x2.getUnderlineColor();
            this._optionsService.rawOptions.drawBoldTextInBrightColors && x2.isBold() && W2 < 8 && (W2 += 8), f3.style.textDecorationColor = p3.ansi[W2].css;
          }
          x2.isOverline() && (V5.push("xterm-overline"), R2 === " " && (R2 = "\xA0")), x2.isStrikethrough() && V5.push("xterm-strikethrough"), te3 && (f3.style.textDecoration = "underline");
          let le3 = x2.getFgColor(), et2 = x2.getFgColorMode(), me3 = x2.getBgColor(), ht2 = x2.getBgColorMode(), fi2 = !!x2.isInverse();
          if (fi2) {
            let W2 = le3;
            le3 = me3, me3 = W2;
            let An2 = et2;
            et2 = ht2, ht2 = An2;
          }
          let tt3, Qi2, pi2 = false;
          this._decorationService.forEachDecorationAtCell(y3, e4, void 0, (W2) => {
            W2.options.layer !== "top" && pi2 || (W2.backgroundColorRGB && (ht2 = 50331648, me3 = W2.backgroundColorRGB.rgba >> 8 & 16777215, tt3 = W2.backgroundColorRGB), W2.foregroundColorRGB && (et2 = 50331648, le3 = W2.foregroundColorRGB.rgba >> 8 & 16777215, Qi2 = W2.foregroundColorRGB), pi2 = W2.options.layer === "top");
          }), !pi2 && N3 && (tt3 = this._coreBrowserService.isFocused ? p3.selectionBackgroundOpaque : p3.selectionInactiveBackgroundOpaque, me3 = tt3.rgba >> 8 & 16777215, ht2 = 50331648, pi2 = true, p3.selectionForeground && (et2 = 50331648, le3 = p3.selectionForeground.rgba >> 8 & 16777215, Qi2 = p3.selectionForeground)), pi2 && V5.push("xterm-decoration-top");
          let it3;
          switch (ht2) {
            case 16777216:
            case 33554432:
              it3 = p3.ansi[me3], V5.push(`xterm-bg-${me3}`);
              break;
            case 50331648:
              it3 = j.toColor(me3 >> 16, me3 >> 8 & 255, me3 & 255), this._addStyle(f3, `background-color:#${qo((me3 >>> 0).toString(16), "0", 6)}`);
              break;
            case 0:
            default:
              fi2 ? (it3 = p3.foreground, V5.push(`xterm-bg-${257}`)) : it3 = p3.background;
          }
          switch (tt3 || x2.isDim() && (tt3 = U.multiplyOpacity(it3, 0.5)), et2) {
            case 16777216:
            case 33554432:
              x2.isBold() && le3 < 8 && this._optionsService.rawOptions.drawBoldTextInBrightColors && (le3 += 8), this._applyMinimumContrast(f3, it3, p3.ansi[le3], x2, tt3, void 0) || V5.push(`xterm-fg-${le3}`);
              break;
            case 50331648:
              let W2 = j.toColor(le3 >> 16 & 255, le3 >> 8 & 255, le3 & 255);
              this._applyMinimumContrast(f3, it3, W2, x2, tt3, Qi2) || this._addStyle(f3, `color:#${qo(le3.toString(16), "0", 6)}`);
              break;
            case 0:
            default:
              this._applyMinimumContrast(f3, it3, p3.foreground, x2, tt3, Qi2) || fi2 && V5.push(`xterm-fg-${257}`);
          }
          V5.length && (f3.className = V5.join(" "), V5.length = 0), !Z3 && !g3 && !Oe3 && w3 ? A3++ : f3.textContent = R2, Ke !== this.defaultSpacing && (f3.style.letterSpacing = `${Ke}px`), d3.push(f3), y3 = E2;
        }
        return f3 && A3 && (f3.textContent = R2), d3;
      }
      _applyMinimumContrast(t3, e4, i12, r7, n4, o6) {
        if (this._optionsService.rawOptions.minimumContrastRatio === 1 || $o(r7.getCode())) return false;
        let l4 = this._getContrastCache(r7), a3;
        if (!n4 && !o6 && (a3 = l4.getColor(e4.rgba, i12.rgba)), a3 === void 0) {
          let u3 = this._optionsService.rawOptions.minimumContrastRatio / (r7.isDim() ? 2 : 1);
          a3 = U.ensureContrastRatio(n4 || e4, o6 || i12, u3), l4.setColor((n4 || e4).rgba, (o6 || i12).rgba, a3 ?? null);
        }
        return a3 ? (this._addStyle(t3, `color:${a3.css}`), true) : false;
      }
      _getContrastCache(t3) {
        return t3.isDim() ? this._themeService.colors.halfContrastCache : this._themeService.colors.contrastCache;
      }
      _addStyle(t3, e4) {
        t3.setAttribute("style", `${t3.getAttribute("style") || ""}${e4};`);
      }
      _isCellInSelection(t3, e4) {
        let i12 = this._selectionStart, r7 = this._selectionEnd;
        return !i12 || !r7 ? false : this._columnSelectMode ? i12[0] <= r7[0] ? t3 >= i12[0] && e4 >= i12[1] && t3 < r7[0] && e4 <= r7[1] : t3 < i12[0] && e4 >= i12[1] && t3 >= r7[0] && e4 <= r7[1] : e4 > i12[1] && e4 < r7[1] || i12[1] === r7[1] && e4 === i12[1] && t3 >= i12[0] && t3 < r7[0] || i12[1] < r7[1] && e4 === r7[1] && t3 < r7[0] || i12[1] < r7[1] && e4 === i12[1] && t3 >= i12[0];
      }
    };
    Vt = M2([S3(1, or), S3(2, H2), S3(3, ae), S3(4, ge), S3(5, Be), S3(6, Re)], Vt);
    Yr = class {
      constructor(t3, e4) {
        this._flat = new Float32Array(256);
        this._font = "";
        this._fontSize = 0;
        this._weight = "normal";
        this._weightBold = "bold";
        this._measureElements = [];
        this._container = t3.createElement("div"), this._container.classList.add("xterm-width-cache-measure-container"), this._container.setAttribute("aria-hidden", "true"), this._container.style.whiteSpace = "pre", this._container.style.fontKerning = "none";
        let i12 = t3.createElement("span");
        i12.classList.add("xterm-char-measure-element");
        let r7 = t3.createElement("span");
        r7.classList.add("xterm-char-measure-element"), r7.style.fontWeight = "bold";
        let n4 = t3.createElement("span");
        n4.classList.add("xterm-char-measure-element"), n4.style.fontStyle = "italic";
        let o6 = t3.createElement("span");
        o6.classList.add("xterm-char-measure-element"), o6.style.fontWeight = "bold", o6.style.fontStyle = "italic", this._measureElements = [i12, r7, n4, o6], this._container.appendChild(i12), this._container.appendChild(r7), this._container.appendChild(n4), this._container.appendChild(o6), e4.appendChild(this._container), this.clear();
      }
      dispose() {
        this._container.remove(), this._measureElements.length = 0, this._holey = void 0;
      }
      clear() {
        this._flat.fill(-9999), this._holey = /* @__PURE__ */ new Map();
      }
      setFont(t3, e4, i12, r7) {
        t3 === this._font && e4 === this._fontSize && i12 === this._weight && r7 === this._weightBold || (this._font = t3, this._fontSize = e4, this._weight = i12, this._weightBold = r7, this._container.style.fontFamily = this._font, this._container.style.fontSize = `${this._fontSize}px`, this._measureElements[0].style.fontWeight = `${i12}`, this._measureElements[1].style.fontWeight = `${r7}`, this._measureElements[2].style.fontWeight = `${i12}`, this._measureElements[3].style.fontWeight = `${r7}`, this.clear());
      }
      get(t3, e4, i12) {
        let r7 = 0;
        if (!e4 && !i12 && t3.length === 1 && (r7 = t3.charCodeAt(0)) < 256) {
          if (this._flat[r7] !== -9999) return this._flat[r7];
          let l4 = this._measure(t3, 0);
          return l4 > 0 && (this._flat[r7] = l4), l4;
        }
        let n4 = t3;
        e4 && (n4 += "B"), i12 && (n4 += "I");
        let o6 = this._holey.get(n4);
        if (o6 === void 0) {
          let l4 = 0;
          e4 && (l4 |= 1), i12 && (l4 |= 2), o6 = this._measure(t3, l4), o6 > 0 && this._holey.set(n4, o6);
        }
        return o6;
      }
      _measure(t3, e4) {
        let i12 = this._measureElements[e4];
        return i12.textContent = t3.repeat(32), i12.offsetWidth / 32;
      }
    };
    ms = class {
      constructor() {
        this.clear();
      }
      clear() {
        this.hasSelection = false, this.columnSelectMode = false, this.viewportStartRow = 0, this.viewportEndRow = 0, this.viewportCappedStartRow = 0, this.viewportCappedEndRow = 0, this.startCol = 0, this.endCol = 0, this.selectionStart = void 0, this.selectionEnd = void 0;
      }
      update(t3, e4, i12, r7 = false) {
        if (this.selectionStart = e4, this.selectionEnd = i12, !e4 || !i12 || e4[0] === i12[0] && e4[1] === i12[1]) {
          this.clear();
          return;
        }
        let n4 = t3.buffers.active.ydisp, o6 = e4[1] - n4, l4 = i12[1] - n4, a3 = Math.max(o6, 0), u3 = Math.min(l4, t3.rows - 1);
        if (a3 >= t3.rows || u3 < 0) {
          this.clear();
          return;
        }
        this.hasSelection = true, this.columnSelectMode = r7, this.viewportStartRow = o6, this.viewportEndRow = l4, this.viewportCappedStartRow = a3, this.viewportCappedEndRow = u3, this.startCol = e4[0], this.endCol = i12[0];
      }
      isCellSelected(t3, e4, i12) {
        return this.hasSelection ? (i12 -= t3.buffer.active.viewportY, this.columnSelectMode ? this.startCol <= this.endCol ? e4 >= this.startCol && i12 >= this.viewportCappedStartRow && e4 < this.endCol && i12 <= this.viewportCappedEndRow : e4 < this.startCol && i12 >= this.viewportCappedStartRow && e4 >= this.endCol && i12 <= this.viewportCappedEndRow : i12 > this.viewportStartRow && i12 < this.viewportEndRow || this.viewportStartRow === this.viewportEndRow && i12 === this.viewportStartRow && e4 >= this.startCol && e4 < this.endCol || this.viewportStartRow < this.viewportEndRow && i12 === this.viewportEndRow && e4 < this.endCol || this.viewportStartRow < this.viewportEndRow && i12 === this.viewportStartRow && e4 >= this.startCol) : false;
      }
    };
    _s = "xterm-dom-renderer-owner-";
    Le = "xterm-rows";
    jr = "xterm-fg-";
    jo = "xterm-bg-";
    ki = "xterm-focus";
    Xr = "xterm-selection";
    Na = 1;
    Yt = class extends D2 {
      constructor(e4, i12, r7, n4, o6, l4, a3, u3, h4, c4, d3, _4, p3, m2) {
        super();
        this._terminal = e4;
        this._document = i12;
        this._element = r7;
        this._screenElement = n4;
        this._viewportElement = o6;
        this._helperContainer = l4;
        this._linkifier2 = a3;
        this._charSizeService = h4;
        this._optionsService = c4;
        this._bufferService = d3;
        this._coreService = _4;
        this._coreBrowserService = p3;
        this._themeService = m2;
        this._terminalClass = Na++;
        this._rowElements = [];
        this._selectionRenderModel = Yo();
        this.onRequestRedraw = this._register(new v2()).event;
        this._rowContainer = this._document.createElement("div"), this._rowContainer.classList.add(Le), this._rowContainer.style.lineHeight = "normal", this._rowContainer.setAttribute("aria-hidden", "true"), this._refreshRowElements(this._bufferService.cols, this._bufferService.rows), this._selectionContainer = this._document.createElement("div"), this._selectionContainer.classList.add(Xr), this._selectionContainer.setAttribute("aria-hidden", "true"), this.dimensions = Vo(), this._updateDimensions(), this._register(this._optionsService.onOptionChange(() => this._handleOptionsChanged())), this._register(this._themeService.onChangeColors((f3) => this._injectCss(f3))), this._injectCss(this._themeService.colors), this._rowFactory = u3.createInstance(Vt, document), this._element.classList.add(_s + this._terminalClass), this._screenElement.appendChild(this._rowContainer), this._screenElement.appendChild(this._selectionContainer), this._register(this._linkifier2.onShowLinkUnderline((f3) => this._handleLinkHover(f3))), this._register(this._linkifier2.onHideLinkUnderline((f3) => this._handleLinkLeave(f3))), this._register(C2(() => {
          this._element.classList.remove(_s + this._terminalClass), this._rowContainer.remove(), this._selectionContainer.remove(), this._widthCache.dispose(), this._themeStyleElement.remove(), this._dimensionsStyleElement.remove();
        })), this._widthCache = new Yr(this._document, this._helperContainer), this._widthCache.setFont(this._optionsService.rawOptions.fontFamily, this._optionsService.rawOptions.fontSize, this._optionsService.rawOptions.fontWeight, this._optionsService.rawOptions.fontWeightBold), this._setDefaultSpacing();
      }
      _updateDimensions() {
        let e4 = this._coreBrowserService.dpr;
        this.dimensions.device.char.width = this._charSizeService.width * e4, this.dimensions.device.char.height = Math.ceil(this._charSizeService.height * e4), this.dimensions.device.cell.width = this.dimensions.device.char.width + Math.round(this._optionsService.rawOptions.letterSpacing), this.dimensions.device.cell.height = Math.floor(this.dimensions.device.char.height * this._optionsService.rawOptions.lineHeight), this.dimensions.device.char.left = 0, this.dimensions.device.char.top = 0, this.dimensions.device.canvas.width = this.dimensions.device.cell.width * this._bufferService.cols, this.dimensions.device.canvas.height = this.dimensions.device.cell.height * this._bufferService.rows, this.dimensions.css.canvas.width = Math.round(this.dimensions.device.canvas.width / e4), this.dimensions.css.canvas.height = Math.round(this.dimensions.device.canvas.height / e4), this.dimensions.css.cell.width = this.dimensions.css.canvas.width / this._bufferService.cols, this.dimensions.css.cell.height = this.dimensions.css.canvas.height / this._bufferService.rows;
        for (let r7 of this._rowElements) r7.style.width = `${this.dimensions.css.canvas.width}px`, r7.style.height = `${this.dimensions.css.cell.height}px`, r7.style.lineHeight = `${this.dimensions.css.cell.height}px`, r7.style.overflow = "hidden";
        this._dimensionsStyleElement || (this._dimensionsStyleElement = this._document.createElement("style"), this._screenElement.appendChild(this._dimensionsStyleElement));
        let i12 = `${this._terminalSelector} .${Le} span { display: inline-block; height: 100%; vertical-align: top;}`;
        this._dimensionsStyleElement.textContent = i12, this._selectionContainer.style.height = this._viewportElement.style.height, this._screenElement.style.width = `${this.dimensions.css.canvas.width}px`, this._screenElement.style.height = `${this.dimensions.css.canvas.height}px`;
      }
      _injectCss(e4) {
        this._themeStyleElement || (this._themeStyleElement = this._document.createElement("style"), this._screenElement.appendChild(this._themeStyleElement));
        let i12 = `${this._terminalSelector} .${Le} { pointer-events: none; color: ${e4.foreground.css}; font-family: ${this._optionsService.rawOptions.fontFamily}; font-size: ${this._optionsService.rawOptions.fontSize}px; font-kerning: none; white-space: pre}`;
        i12 += `${this._terminalSelector} .${Le} .xterm-dim { color: ${U.multiplyOpacity(e4.foreground, 0.5).css};}`, i12 += `${this._terminalSelector} span:not(.xterm-bold) { font-weight: ${this._optionsService.rawOptions.fontWeight};}${this._terminalSelector} span.xterm-bold { font-weight: ${this._optionsService.rawOptions.fontWeightBold};}${this._terminalSelector} span.xterm-italic { font-style: italic;}`;
        let r7 = `blink_underline_${this._terminalClass}`, n4 = `blink_bar_${this._terminalClass}`, o6 = `blink_block_${this._terminalClass}`;
        i12 += `@keyframes ${r7} { 50% {  border-bottom-style: hidden; }}`, i12 += `@keyframes ${n4} { 50% {  box-shadow: none; }}`, i12 += `@keyframes ${o6} { 0% {  background-color: ${e4.cursor.css};  color: ${e4.cursorAccent.css}; } 50% {  background-color: inherit;  color: ${e4.cursor.css}; }}`, i12 += `${this._terminalSelector} .${Le}.${ki} .xterm-cursor.xterm-cursor-blink.xterm-cursor-underline { animation: ${r7} 1s step-end infinite;}${this._terminalSelector} .${Le}.${ki} .xterm-cursor.xterm-cursor-blink.xterm-cursor-bar { animation: ${n4} 1s step-end infinite;}${this._terminalSelector} .${Le}.${ki} .xterm-cursor.xterm-cursor-blink.xterm-cursor-block { animation: ${o6} 1s step-end infinite;}${this._terminalSelector} .${Le} .xterm-cursor.xterm-cursor-block { background-color: ${e4.cursor.css}; color: ${e4.cursorAccent.css};}${this._terminalSelector} .${Le} .xterm-cursor.xterm-cursor-block:not(.xterm-cursor-blink) { background-color: ${e4.cursor.css} !important; color: ${e4.cursorAccent.css} !important;}${this._terminalSelector} .${Le} .xterm-cursor.xterm-cursor-outline { outline: 1px solid ${e4.cursor.css}; outline-offset: -1px;}${this._terminalSelector} .${Le} .xterm-cursor.xterm-cursor-bar { box-shadow: ${this._optionsService.rawOptions.cursorWidth}px 0 0 ${e4.cursor.css} inset;}${this._terminalSelector} .${Le} .xterm-cursor.xterm-cursor-underline { border-bottom: 1px ${e4.cursor.css}; border-bottom-style: solid; height: calc(100% - 1px);}`, i12 += `${this._terminalSelector} .${Xr} { position: absolute; top: 0; left: 0; z-index: 1; pointer-events: none;}${this._terminalSelector}.focus .${Xr} div { position: absolute; background-color: ${e4.selectionBackgroundOpaque.css};}${this._terminalSelector} .${Xr} div { position: absolute; background-color: ${e4.selectionInactiveBackgroundOpaque.css};}`;
        for (let [l4, a3] of e4.ansi.entries()) i12 += `${this._terminalSelector} .${jr}${l4} { color: ${a3.css}; }${this._terminalSelector} .${jr}${l4}.xterm-dim { color: ${U.multiplyOpacity(a3, 0.5).css}; }${this._terminalSelector} .${jo}${l4} { background-color: ${a3.css}; }`;
        i12 += `${this._terminalSelector} .${jr}${257} { color: ${U.opaque(e4.background).css}; }${this._terminalSelector} .${jr}${257}.xterm-dim { color: ${U.multiplyOpacity(U.opaque(e4.background), 0.5).css}; }${this._terminalSelector} .${jo}${257} { background-color: ${e4.foreground.css}; }`, this._themeStyleElement.textContent = i12;
      }
      _setDefaultSpacing() {
        let e4 = this.dimensions.css.cell.width - this._widthCache.get("W", false, false);
        this._rowContainer.style.letterSpacing = `${e4}px`, this._rowFactory.defaultSpacing = e4;
      }
      handleDevicePixelRatioChange() {
        this._updateDimensions(), this._widthCache.clear(), this._setDefaultSpacing();
      }
      _refreshRowElements(e4, i12) {
        for (let r7 = this._rowElements.length; r7 <= i12; r7++) {
          let n4 = this._document.createElement("div");
          this._rowContainer.appendChild(n4), this._rowElements.push(n4);
        }
        for (; this._rowElements.length > i12; ) this._rowContainer.removeChild(this._rowElements.pop());
      }
      handleResize(e4, i12) {
        this._refreshRowElements(e4, i12), this._updateDimensions(), this.handleSelectionChanged(this._selectionRenderModel.selectionStart, this._selectionRenderModel.selectionEnd, this._selectionRenderModel.columnSelectMode);
      }
      handleCharSizeChanged() {
        this._updateDimensions(), this._widthCache.clear(), this._setDefaultSpacing();
      }
      handleBlur() {
        this._rowContainer.classList.remove(ki), this.renderRows(0, this._bufferService.rows - 1);
      }
      handleFocus() {
        this._rowContainer.classList.add(ki), this.renderRows(this._bufferService.buffer.y, this._bufferService.buffer.y);
      }
      handleSelectionChanged(e4, i12, r7) {
        if (this._selectionContainer.replaceChildren(), this._rowFactory.handleSelectionChanged(e4, i12, r7), this.renderRows(0, this._bufferService.rows - 1), !e4 || !i12 || (this._selectionRenderModel.update(this._terminal, e4, i12, r7), !this._selectionRenderModel.hasSelection)) return;
        let n4 = this._selectionRenderModel.viewportStartRow, o6 = this._selectionRenderModel.viewportEndRow, l4 = this._selectionRenderModel.viewportCappedStartRow, a3 = this._selectionRenderModel.viewportCappedEndRow, u3 = this._document.createDocumentFragment();
        if (r7) {
          let h4 = e4[0] > i12[0];
          u3.appendChild(this._createSelectionElement(l4, h4 ? i12[0] : e4[0], h4 ? e4[0] : i12[0], a3 - l4 + 1));
        } else {
          let h4 = n4 === l4 ? e4[0] : 0, c4 = l4 === o6 ? i12[0] : this._bufferService.cols;
          u3.appendChild(this._createSelectionElement(l4, h4, c4));
          let d3 = a3 - l4 - 1;
          if (u3.appendChild(this._createSelectionElement(l4 + 1, 0, this._bufferService.cols, d3)), l4 !== a3) {
            let _4 = o6 === a3 ? i12[0] : this._bufferService.cols;
            u3.appendChild(this._createSelectionElement(a3, 0, _4));
          }
        }
        this._selectionContainer.appendChild(u3);
      }
      _createSelectionElement(e4, i12, r7, n4 = 1) {
        let o6 = this._document.createElement("div"), l4 = i12 * this.dimensions.css.cell.width, a3 = this.dimensions.css.cell.width * (r7 - i12);
        return l4 + a3 > this.dimensions.css.canvas.width && (a3 = this.dimensions.css.canvas.width - l4), o6.style.height = `${n4 * this.dimensions.css.cell.height}px`, o6.style.top = `${e4 * this.dimensions.css.cell.height}px`, o6.style.left = `${l4}px`, o6.style.width = `${a3}px`, o6;
      }
      handleCursorMove() {
      }
      _handleOptionsChanged() {
        this._updateDimensions(), this._injectCss(this._themeService.colors), this._widthCache.setFont(this._optionsService.rawOptions.fontFamily, this._optionsService.rawOptions.fontSize, this._optionsService.rawOptions.fontWeight, this._optionsService.rawOptions.fontWeightBold), this._setDefaultSpacing();
      }
      clear() {
        for (let e4 of this._rowElements) e4.replaceChildren();
      }
      renderRows(e4, i12) {
        let r7 = this._bufferService.buffer, n4 = r7.ybase + r7.y, o6 = Math.min(r7.x, this._bufferService.cols - 1), l4 = this._coreService.decPrivateModes.cursorBlink ?? this._optionsService.rawOptions.cursorBlink, a3 = this._coreService.decPrivateModes.cursorStyle ?? this._optionsService.rawOptions.cursorStyle, u3 = this._optionsService.rawOptions.cursorInactiveStyle;
        for (let h4 = e4; h4 <= i12; h4++) {
          let c4 = h4 + r7.ydisp, d3 = this._rowElements[h4], _4 = r7.lines.get(c4);
          if (!d3 || !_4) break;
          d3.replaceChildren(...this._rowFactory.createRow(_4, c4, c4 === n4, a3, u3, o6, l4, this.dimensions.css.cell.width, this._widthCache, -1, -1));
        }
      }
      get _terminalSelector() {
        return `.${_s}${this._terminalClass}`;
      }
      _handleLinkHover(e4) {
        this._setCellUnderline(e4.x1, e4.x2, e4.y1, e4.y2, e4.cols, true);
      }
      _handleLinkLeave(e4) {
        this._setCellUnderline(e4.x1, e4.x2, e4.y1, e4.y2, e4.cols, false);
      }
      _setCellUnderline(e4, i12, r7, n4, o6, l4) {
        r7 < 0 && (e4 = 0), n4 < 0 && (i12 = 0);
        let a3 = this._bufferService.rows - 1;
        r7 = Math.max(Math.min(r7, a3), 0), n4 = Math.max(Math.min(n4, a3), 0), o6 = Math.min(o6, this._bufferService.cols);
        let u3 = this._bufferService.buffer, h4 = u3.ybase + u3.y, c4 = Math.min(u3.x, o6 - 1), d3 = this._optionsService.rawOptions.cursorBlink, _4 = this._optionsService.rawOptions.cursorStyle, p3 = this._optionsService.rawOptions.cursorInactiveStyle;
        for (let m2 = r7; m2 <= n4; ++m2) {
          let f3 = m2 + u3.ydisp, A3 = this._rowElements[m2], R2 = u3.lines.get(f3);
          if (!A3 || !R2) break;
          A3.replaceChildren(...this._rowFactory.createRow(R2, f3, f3 === h4, _4, p3, c4, d3, this.dimensions.css.cell.width, this._widthCache, l4 ? m2 === r7 ? e4 : 0 : -1, l4 ? (m2 === n4 ? i12 : o6) - 1 : -1));
        }
      }
    };
    Yt = M2([S3(7, xt), S3(8, nt), S3(9, H2), S3(10, F), S3(11, ge), S3(12, ae), S3(13, Re)], Yt);
    jt = class extends D2 {
      constructor(e4, i12, r7) {
        super();
        this._optionsService = r7;
        this.width = 0;
        this.height = 0;
        this._onCharSizeChange = this._register(new v2());
        this.onCharSizeChange = this._onCharSizeChange.event;
        try {
          this._measureStrategy = this._register(new vs(this._optionsService));
        } catch {
          this._measureStrategy = this._register(new bs(e4, i12, this._optionsService));
        }
        this._register(this._optionsService.onMultipleOptionChange(["fontFamily", "fontSize"], () => this.measure()));
      }
      get hasValidSize() {
        return this.width > 0 && this.height > 0;
      }
      measure() {
        let e4 = this._measureStrategy.measure();
        (e4.width !== this.width || e4.height !== this.height) && (this.width = e4.width, this.height = e4.height, this._onCharSizeChange.fire());
      }
    };
    jt = M2([S3(2, H2)], jt);
    Zr = class extends D2 {
      constructor() {
        super(...arguments);
        this._result = { width: 0, height: 0 };
      }
      _validateAndSet(e4, i12) {
        e4 !== void 0 && e4 > 0 && i12 !== void 0 && i12 > 0 && (this._result.width = e4, this._result.height = i12);
      }
    };
    bs = class extends Zr {
      constructor(e4, i12, r7) {
        super();
        this._document = e4;
        this._parentElement = i12;
        this._optionsService = r7;
        this._measureElement = this._document.createElement("span"), this._measureElement.classList.add("xterm-char-measure-element"), this._measureElement.textContent = "W".repeat(32), this._measureElement.setAttribute("aria-hidden", "true"), this._measureElement.style.whiteSpace = "pre", this._measureElement.style.fontKerning = "none", this._parentElement.appendChild(this._measureElement);
      }
      measure() {
        return this._measureElement.style.fontFamily = this._optionsService.rawOptions.fontFamily, this._measureElement.style.fontSize = `${this._optionsService.rawOptions.fontSize}px`, this._validateAndSet(Number(this._measureElement.offsetWidth) / 32, Number(this._measureElement.offsetHeight)), this._result;
      }
    };
    vs = class extends Zr {
      constructor(e4) {
        super();
        this._optionsService = e4;
        this._canvas = new OffscreenCanvas(100, 100), this._ctx = this._canvas.getContext("2d");
        let i12 = this._ctx.measureText("W");
        if (!("width" in i12 && "fontBoundingBoxAscent" in i12 && "fontBoundingBoxDescent" in i12)) throw new Error("Required font metrics not supported");
      }
      measure() {
        this._ctx.font = `${this._optionsService.rawOptions.fontSize}px ${this._optionsService.rawOptions.fontFamily}`;
        let e4 = this._ctx.measureText("W");
        return this._validateAndSet(e4.width, e4.fontBoundingBoxAscent + e4.fontBoundingBoxDescent), this._result;
      }
    };
    Jr = class extends D2 {
      constructor(e4, i12, r7) {
        super();
        this._textarea = e4;
        this._window = i12;
        this.mainDocument = r7;
        this._isFocused = false;
        this._cachedIsFocused = void 0;
        this._screenDprMonitor = this._register(new gs(this._window));
        this._onDprChange = this._register(new v2());
        this.onDprChange = this._onDprChange.event;
        this._onWindowChange = this._register(new v2());
        this.onWindowChange = this._onWindowChange.event;
        this._register(this.onWindowChange((n4) => this._screenDprMonitor.setWindow(n4))), this._register($2.forward(this._screenDprMonitor.onDprChange, this._onDprChange)), this._register(L2(this._textarea, "focus", () => this._isFocused = true)), this._register(L2(this._textarea, "blur", () => this._isFocused = false));
      }
      get window() {
        return this._window;
      }
      set window(e4) {
        this._window !== e4 && (this._window = e4, this._onWindowChange.fire(this._window));
      }
      get dpr() {
        return this.window.devicePixelRatio;
      }
      get isFocused() {
        return this._cachedIsFocused === void 0 && (this._cachedIsFocused = this._isFocused && this._textarea.ownerDocument.hasFocus(), queueMicrotask(() => this._cachedIsFocused = void 0)), this._cachedIsFocused;
      }
    };
    gs = class extends D2 {
      constructor(e4) {
        super();
        this._parentWindow = e4;
        this._windowResizeListener = this._register(new ye());
        this._onDprChange = this._register(new v2());
        this.onDprChange = this._onDprChange.event;
        this._outerListener = () => this._setDprAndFireIfDiffers(), this._currentDevicePixelRatio = this._parentWindow.devicePixelRatio, this._updateDpr(), this._setWindowResizeListener(), this._register(C2(() => this.clearListener()));
      }
      setWindow(e4) {
        this._parentWindow = e4, this._setWindowResizeListener(), this._setDprAndFireIfDiffers();
      }
      _setWindowResizeListener() {
        this._windowResizeListener.value = L2(this._parentWindow, "resize", () => this._setDprAndFireIfDiffers());
      }
      _setDprAndFireIfDiffers() {
        this._parentWindow.devicePixelRatio !== this._currentDevicePixelRatio && this._onDprChange.fire(this._parentWindow.devicePixelRatio), this._updateDpr();
      }
      _updateDpr() {
        this._outerListener && (this._resolutionMediaMatchList?.removeListener(this._outerListener), this._currentDevicePixelRatio = this._parentWindow.devicePixelRatio, this._resolutionMediaMatchList = this._parentWindow.matchMedia(`screen and (resolution: ${this._parentWindow.devicePixelRatio}dppx)`), this._resolutionMediaMatchList.addListener(this._outerListener));
      }
      clearListener() {
        !this._resolutionMediaMatchList || !this._outerListener || (this._resolutionMediaMatchList.removeListener(this._outerListener), this._resolutionMediaMatchList = void 0, this._outerListener = void 0);
      }
    };
    Qr = class extends D2 {
      constructor() {
        super();
        this.linkProviders = [];
        this._register(C2(() => this.linkProviders.length = 0));
      }
      registerLinkProvider(e4) {
        return this.linkProviders.push(e4), { dispose: () => {
          let i12 = this.linkProviders.indexOf(e4);
          i12 !== -1 && this.linkProviders.splice(i12, 1);
        } };
      }
    };
    Xt = class {
      constructor(t3, e4) {
        this._renderService = t3;
        this._charSizeService = e4;
      }
      getCoords(t3, e4, i12, r7, n4) {
        return Xo(window, t3, e4, i12, r7, this._charSizeService.hasValidSize, this._renderService.dimensions.css.cell.width, this._renderService.dimensions.css.cell.height, n4);
      }
      getMouseReportCoords(t3, e4) {
        let i12 = Ci(window, t3, e4);
        if (this._charSizeService.hasValidSize) return i12[0] = Math.min(Math.max(i12[0], 0), this._renderService.dimensions.css.canvas.width - 1), i12[1] = Math.min(Math.max(i12[1], 0), this._renderService.dimensions.css.canvas.height - 1), { col: Math.floor(i12[0] / this._renderService.dimensions.css.cell.width), row: Math.floor(i12[1] / this._renderService.dimensions.css.cell.height), x: Math.floor(i12[0]), y: Math.floor(i12[1]) };
      }
    };
    Xt = M2([S3(0, ce), S3(1, nt)], Xt);
    en = class {
      constructor(t3, e4) {
        this._renderCallback = t3;
        this._coreBrowserService = e4;
        this._refreshCallbacks = [];
      }
      dispose() {
        this._animationFrame && (this._coreBrowserService.window.cancelAnimationFrame(this._animationFrame), this._animationFrame = void 0);
      }
      addRefreshCallback(t3) {
        return this._refreshCallbacks.push(t3), this._animationFrame || (this._animationFrame = this._coreBrowserService.window.requestAnimationFrame(() => this._innerRefresh())), this._animationFrame;
      }
      refresh(t3, e4, i12) {
        this._rowCount = i12, t3 = t3 !== void 0 ? t3 : 0, e4 = e4 !== void 0 ? e4 : this._rowCount - 1, this._rowStart = this._rowStart !== void 0 ? Math.min(this._rowStart, t3) : t3, this._rowEnd = this._rowEnd !== void 0 ? Math.max(this._rowEnd, e4) : e4, !this._animationFrame && (this._animationFrame = this._coreBrowserService.window.requestAnimationFrame(() => this._innerRefresh()));
      }
      _innerRefresh() {
        if (this._animationFrame = void 0, this._rowStart === void 0 || this._rowEnd === void 0 || this._rowCount === void 0) {
          this._runRefreshCallbacks();
          return;
        }
        let t3 = Math.max(this._rowStart, 0), e4 = Math.min(this._rowEnd, this._rowCount - 1);
        this._rowStart = void 0, this._rowEnd = void 0, this._renderCallback(t3, e4), this._runRefreshCallbacks();
      }
      _runRefreshCallbacks() {
        for (let t3 of this._refreshCallbacks) t3(0);
        this._refreshCallbacks = [];
      }
    };
    tn = {};
    Ll(tn, { getSafariVersion: () => Ha, isChromeOS: () => Ts, isFirefox: () => Ss, isIpad: () => Wa, isIphone: () => Ua, isLegacyEdge: () => Fa, isLinux: () => Bi, isMac: () => Zt, isNode: () => Mi, isSafari: () => Zo, isWindows: () => Es });
    Mi = typeof process < "u" && "title" in process;
    Pi = Mi ? "node" : navigator.userAgent;
    Oi = Mi ? "node" : navigator.platform;
    Ss = Pi.includes("Firefox");
    Fa = Pi.includes("Edge");
    Zo = /^((?!chrome|android).)*safari/i.test(Pi);
    Zt = ["Macintosh", "MacIntel", "MacPPC", "Mac68K"].includes(Oi);
    Wa = Oi === "iPad";
    Ua = Oi === "iPhone";
    Es = ["Windows", "Win16", "Win32", "WinCE"].includes(Oi);
    Bi = Oi.indexOf("Linux") >= 0;
    Ts = /\bCrOS\b/.test(Pi);
    rn = class {
      constructor() {
        this._tasks = [];
        this._i = 0;
      }
      enqueue(t3) {
        this._tasks.push(t3), this._start();
      }
      flush() {
        for (; this._i < this._tasks.length; ) this._tasks[this._i]() || this._i++;
        this.clear();
      }
      clear() {
        this._idleCallback && (this._cancelCallback(this._idleCallback), this._idleCallback = void 0), this._i = 0, this._tasks.length = 0;
      }
      _start() {
        this._idleCallback || (this._idleCallback = this._requestCallback(this._process.bind(this)));
      }
      _process(t3) {
        this._idleCallback = void 0;
        let e4 = 0, i12 = 0, r7 = t3.timeRemaining(), n4 = 0;
        for (; this._i < this._tasks.length; ) {
          if (e4 = performance.now(), this._tasks[this._i]() || this._i++, e4 = Math.max(1, performance.now() - e4), i12 = Math.max(e4, i12), n4 = t3.timeRemaining(), i12 * 1.5 > n4) {
            r7 - e4 < -20 && console.warn(`task queue exceeded allotted deadline by ${Math.abs(Math.round(r7 - e4))}ms`), this._start();
            return;
          }
          r7 = n4;
        }
        this.clear();
      }
    };
    Is = class extends rn {
      _requestCallback(t3) {
        return setTimeout(() => t3(this._createDeadline(16)));
      }
      _cancelCallback(t3) {
        clearTimeout(t3);
      }
      _createDeadline(t3) {
        let e4 = performance.now() + t3;
        return { timeRemaining: () => Math.max(0, e4 - performance.now()) };
      }
    };
    ys = class extends rn {
      _requestCallback(t3) {
        return requestIdleCallback(t3);
      }
      _cancelCallback(t3) {
        cancelIdleCallback(t3);
      }
    };
    Jt = !Mi && "requestIdleCallback" in window ? ys : Is;
    nn = class {
      constructor() {
        this._queue = new Jt();
      }
      set(t3) {
        this._queue.clear(), this._queue.enqueue(t3);
      }
      flush() {
        this._queue.flush();
      }
    };
    Qt = class extends D2 {
      constructor(e4, i12, r7, n4, o6, l4, a3, u3, h4) {
        super();
        this._rowCount = e4;
        this._optionsService = r7;
        this._charSizeService = n4;
        this._coreService = o6;
        this._coreBrowserService = u3;
        this._renderer = this._register(new ye());
        this._pausedResizeTask = new nn();
        this._observerDisposable = this._register(new ye());
        this._isPaused = false;
        this._needsFullRefresh = false;
        this._isNextRenderRedrawOnly = true;
        this._needsSelectionRefresh = false;
        this._canvasWidth = 0;
        this._canvasHeight = 0;
        this._selectionState = { start: void 0, end: void 0, columnSelectMode: false };
        this._onDimensionsChange = this._register(new v2());
        this.onDimensionsChange = this._onDimensionsChange.event;
        this._onRenderedViewportChange = this._register(new v2());
        this.onRenderedViewportChange = this._onRenderedViewportChange.event;
        this._onRender = this._register(new v2());
        this.onRender = this._onRender.event;
        this._onRefreshRequest = this._register(new v2());
        this.onRefreshRequest = this._onRefreshRequest.event;
        this._renderDebouncer = new en((c4, d3) => this._renderRows(c4, d3), this._coreBrowserService), this._register(this._renderDebouncer), this._syncOutputHandler = new xs(this._coreBrowserService, this._coreService, () => this._fullRefresh()), this._register(C2(() => this._syncOutputHandler.dispose())), this._register(this._coreBrowserService.onDprChange(() => this.handleDevicePixelRatioChange())), this._register(a3.onResize(() => this._fullRefresh())), this._register(a3.buffers.onBufferActivate(() => this._renderer.value?.clear())), this._register(this._optionsService.onOptionChange(() => this._handleOptionsChanged())), this._register(this._charSizeService.onCharSizeChange(() => this.handleCharSizeChanged())), this._register(l4.onDecorationRegistered(() => this._fullRefresh())), this._register(l4.onDecorationRemoved(() => this._fullRefresh())), this._register(this._optionsService.onMultipleOptionChange(["customGlyphs", "drawBoldTextInBrightColors", "letterSpacing", "lineHeight", "fontFamily", "fontSize", "fontWeight", "fontWeightBold", "minimumContrastRatio", "rescaleOverlappingGlyphs"], () => {
          this.clear(), this.handleResize(a3.cols, a3.rows), this._fullRefresh();
        })), this._register(this._optionsService.onMultipleOptionChange(["cursorBlink", "cursorStyle"], () => this.refreshRows(a3.buffer.y, a3.buffer.y, true))), this._register(h4.onChangeColors(() => this._fullRefresh())), this._registerIntersectionObserver(this._coreBrowserService.window, i12), this._register(this._coreBrowserService.onWindowChange((c4) => this._registerIntersectionObserver(c4, i12)));
      }
      get dimensions() {
        return this._renderer.value.dimensions;
      }
      _registerIntersectionObserver(e4, i12) {
        if ("IntersectionObserver" in e4) {
          let r7 = new e4.IntersectionObserver((n4) => this._handleIntersectionChange(n4[n4.length - 1]), { threshold: 0 });
          r7.observe(i12), this._observerDisposable.value = C2(() => r7.disconnect());
        }
      }
      _handleIntersectionChange(e4) {
        this._isPaused = e4.isIntersecting === void 0 ? e4.intersectionRatio === 0 : !e4.isIntersecting, !this._isPaused && !this._charSizeService.hasValidSize && this._charSizeService.measure(), !this._isPaused && this._needsFullRefresh && (this._pausedResizeTask.flush(), this.refreshRows(0, this._rowCount - 1), this._needsFullRefresh = false);
      }
      refreshRows(e4, i12, r7 = false) {
        if (this._isPaused) {
          this._needsFullRefresh = true;
          return;
        }
        if (this._coreService.decPrivateModes.synchronizedOutput) {
          this._syncOutputHandler.bufferRows(e4, i12);
          return;
        }
        let n4 = this._syncOutputHandler.flush();
        n4 && (e4 = Math.min(e4, n4.start), i12 = Math.max(i12, n4.end)), r7 || (this._isNextRenderRedrawOnly = false), this._renderDebouncer.refresh(e4, i12, this._rowCount);
      }
      _renderRows(e4, i12) {
        if (this._renderer.value) {
          if (this._coreService.decPrivateModes.synchronizedOutput) {
            this._syncOutputHandler.bufferRows(e4, i12);
            return;
          }
          e4 = Math.min(e4, this._rowCount - 1), i12 = Math.min(i12, this._rowCount - 1), this._renderer.value.renderRows(e4, i12), this._needsSelectionRefresh && (this._renderer.value.handleSelectionChanged(this._selectionState.start, this._selectionState.end, this._selectionState.columnSelectMode), this._needsSelectionRefresh = false), this._isNextRenderRedrawOnly || this._onRenderedViewportChange.fire({ start: e4, end: i12 }), this._onRender.fire({ start: e4, end: i12 }), this._isNextRenderRedrawOnly = true;
        }
      }
      resize(e4, i12) {
        this._rowCount = i12, this._fireOnCanvasResize();
      }
      _handleOptionsChanged() {
        this._renderer.value && (this.refreshRows(0, this._rowCount - 1), this._fireOnCanvasResize());
      }
      _fireOnCanvasResize() {
        this._renderer.value && (this._renderer.value.dimensions.css.canvas.width === this._canvasWidth && this._renderer.value.dimensions.css.canvas.height === this._canvasHeight || this._onDimensionsChange.fire(this._renderer.value.dimensions));
      }
      hasRenderer() {
        return !!this._renderer.value;
      }
      setRenderer(e4) {
        this._renderer.value = e4, this._renderer.value && (this._renderer.value.onRequestRedraw((i12) => this.refreshRows(i12.start, i12.end, true)), this._needsSelectionRefresh = true, this._fullRefresh());
      }
      addRefreshCallback(e4) {
        return this._renderDebouncer.addRefreshCallback(e4);
      }
      _fullRefresh() {
        this._isPaused ? this._needsFullRefresh = true : this.refreshRows(0, this._rowCount - 1);
      }
      clearTextureAtlas() {
        this._renderer.value && (this._renderer.value.clearTextureAtlas?.(), this._fullRefresh());
      }
      handleDevicePixelRatioChange() {
        this._charSizeService.measure(), this._renderer.value && (this._renderer.value.handleDevicePixelRatioChange(), this.refreshRows(0, this._rowCount - 1));
      }
      handleResize(e4, i12) {
        this._renderer.value && (this._isPaused ? this._pausedResizeTask.set(() => this._renderer.value?.handleResize(e4, i12)) : this._renderer.value.handleResize(e4, i12), this._fullRefresh());
      }
      handleCharSizeChanged() {
        this._renderer.value?.handleCharSizeChanged();
      }
      handleBlur() {
        this._renderer.value?.handleBlur();
      }
      handleFocus() {
        this._renderer.value?.handleFocus();
      }
      handleSelectionChanged(e4, i12, r7) {
        this._selectionState.start = e4, this._selectionState.end = i12, this._selectionState.columnSelectMode = r7, this._renderer.value?.handleSelectionChanged(e4, i12, r7);
      }
      handleCursorMove() {
        this._renderer.value?.handleCursorMove();
      }
      clear() {
        this._renderer.value?.clear();
      }
    };
    Qt = M2([S3(2, H2), S3(3, nt), S3(4, ge), S3(5, Be), S3(6, F), S3(7, ae), S3(8, Re)], Qt);
    xs = class {
      constructor(t3, e4, i12) {
        this._coreBrowserService = t3;
        this._coreService = e4;
        this._onTimeout = i12;
        this._start = 0;
        this._end = 0;
        this._isBuffering = false;
      }
      bufferRows(t3, e4) {
        this._isBuffering ? (this._start = Math.min(this._start, t3), this._end = Math.max(this._end, e4)) : (this._start = t3, this._end = e4, this._isBuffering = true), this._timeout === void 0 && (this._timeout = this._coreBrowserService.window.setTimeout(() => {
          this._timeout = void 0, this._coreService.decPrivateModes.synchronizedOutput = false, this._onTimeout();
        }, 1e3));
      }
      flush() {
        if (this._timeout !== void 0 && (this._coreBrowserService.window.clearTimeout(this._timeout), this._timeout = void 0), !this._isBuffering) return;
        let t3 = { start: this._start, end: this._end };
        return this._isBuffering = false, t3;
      }
      dispose() {
        this._timeout !== void 0 && (this._coreBrowserService.window.clearTimeout(this._timeout), this._timeout = void 0);
      }
    };
    on = class {
      constructor(t3) {
        this._bufferService = t3;
        this.isSelectAllActive = false;
        this.selectionStartLength = 0;
      }
      clearSelection() {
        this.selectionStart = void 0, this.selectionEnd = void 0, this.isSelectAllActive = false, this.selectionStartLength = 0;
      }
      get finalSelectionStart() {
        return this.isSelectAllActive ? [0, 0] : !this.selectionEnd || !this.selectionStart ? this.selectionStart : this.areSelectionValuesReversed() ? this.selectionEnd : this.selectionStart;
      }
      get finalSelectionEnd() {
        if (this.isSelectAllActive) return [this._bufferService.cols, this._bufferService.buffer.ybase + this._bufferService.rows - 1];
        if (this.selectionStart) {
          if (!this.selectionEnd || this.areSelectionValuesReversed()) {
            let t3 = this.selectionStart[0] + this.selectionStartLength;
            return t3 > this._bufferService.cols ? t3 % this._bufferService.cols === 0 ? [this._bufferService.cols, this.selectionStart[1] + Math.floor(t3 / this._bufferService.cols) - 1] : [t3 % this._bufferService.cols, this.selectionStart[1] + Math.floor(t3 / this._bufferService.cols)] : [t3, this.selectionStart[1]];
          }
          if (this.selectionStartLength && this.selectionEnd[1] === this.selectionStart[1]) {
            let t3 = this.selectionStart[0] + this.selectionStartLength;
            return t3 > this._bufferService.cols ? [t3 % this._bufferService.cols, this.selectionStart[1] + Math.floor(t3 / this._bufferService.cols)] : [Math.max(t3, this.selectionEnd[0]), this.selectionEnd[1]];
          }
          return this.selectionEnd;
        }
      }
      areSelectionValuesReversed() {
        let t3 = this.selectionStart, e4 = this.selectionEnd;
        return !t3 || !e4 ? false : t3[1] > e4[1] || t3[1] === e4[1] && t3[0] > e4[0];
      }
      handleTrim(t3) {
        return this.selectionStart && (this.selectionStart[1] -= t3), this.selectionEnd && (this.selectionEnd[1] -= t3), this.selectionEnd && this.selectionEnd[1] < 0 ? (this.clearSelection(), true) : (this.selectionStart && this.selectionStart[1] < 0 && (this.selectionStart[1] = 0), false);
      }
    };
    Ds = 50;
    Ya = 15;
    ja = 50;
    Xa = 500;
    Za = "\xA0";
    Ja = new RegExp(Za, "g");
    ei = class extends D2 {
      constructor(e4, i12, r7, n4, o6, l4, a3, u3, h4) {
        super();
        this._element = e4;
        this._screenElement = i12;
        this._linkifier = r7;
        this._bufferService = n4;
        this._coreService = o6;
        this._mouseService = l4;
        this._optionsService = a3;
        this._renderService = u3;
        this._coreBrowserService = h4;
        this._dragScrollAmount = 0;
        this._enabled = true;
        this._workCell = new q();
        this._mouseDownTimeStamp = 0;
        this._oldHasSelection = false;
        this._oldSelectionStart = void 0;
        this._oldSelectionEnd = void 0;
        this._onLinuxMouseSelection = this._register(new v2());
        this.onLinuxMouseSelection = this._onLinuxMouseSelection.event;
        this._onRedrawRequest = this._register(new v2());
        this.onRequestRedraw = this._onRedrawRequest.event;
        this._onSelectionChange = this._register(new v2());
        this.onSelectionChange = this._onSelectionChange.event;
        this._onRequestScrollLines = this._register(new v2());
        this.onRequestScrollLines = this._onRequestScrollLines.event;
        this._mouseMoveListener = (c4) => this._handleMouseMove(c4), this._mouseUpListener = (c4) => this._handleMouseUp(c4), this._coreService.onUserInput(() => {
          this.hasSelection && this.clearSelection();
        }), this._trimListener = this._bufferService.buffer.lines.onTrim((c4) => this._handleTrim(c4)), this._register(this._bufferService.buffers.onBufferActivate((c4) => this._handleBufferActivate(c4))), this.enable(), this._model = new on(this._bufferService), this._activeSelectionMode = 0, this._register(C2(() => {
          this._removeMouseDownListeners();
        })), this._register(this._bufferService.onResize((c4) => {
          c4.rowsChanged && this.clearSelection();
        }));
      }
      reset() {
        this.clearSelection();
      }
      disable() {
        this.clearSelection(), this._enabled = false;
      }
      enable() {
        this._enabled = true;
      }
      get selectionStart() {
        return this._model.finalSelectionStart;
      }
      get selectionEnd() {
        return this._model.finalSelectionEnd;
      }
      get hasSelection() {
        let e4 = this._model.finalSelectionStart, i12 = this._model.finalSelectionEnd;
        return !e4 || !i12 ? false : e4[0] !== i12[0] || e4[1] !== i12[1];
      }
      get selectionText() {
        let e4 = this._model.finalSelectionStart, i12 = this._model.finalSelectionEnd;
        if (!e4 || !i12) return "";
        let r7 = this._bufferService.buffer, n4 = [];
        if (this._activeSelectionMode === 3) {
          if (e4[0] === i12[0]) return "";
          let l4 = e4[0] < i12[0] ? e4[0] : i12[0], a3 = e4[0] < i12[0] ? i12[0] : e4[0];
          for (let u3 = e4[1]; u3 <= i12[1]; u3++) {
            let h4 = r7.translateBufferLineToString(u3, true, l4, a3);
            n4.push(h4);
          }
        } else {
          let l4 = e4[1] === i12[1] ? i12[0] : void 0;
          n4.push(r7.translateBufferLineToString(e4[1], true, e4[0], l4));
          for (let a3 = e4[1] + 1; a3 <= i12[1] - 1; a3++) {
            let u3 = r7.lines.get(a3), h4 = r7.translateBufferLineToString(a3, true);
            u3?.isWrapped ? n4[n4.length - 1] += h4 : n4.push(h4);
          }
          if (e4[1] !== i12[1]) {
            let a3 = r7.lines.get(i12[1]), u3 = r7.translateBufferLineToString(i12[1], true, 0, i12[0]);
            a3 && a3.isWrapped ? n4[n4.length - 1] += u3 : n4.push(u3);
          }
        }
        return n4.map((l4) => l4.replace(Ja, " ")).join(Es ? `\r
` : `
`);
      }
      clearSelection() {
        this._model.clearSelection(), this._removeMouseDownListeners(), this.refresh(), this._onSelectionChange.fire();
      }
      refresh(e4) {
        this._refreshAnimationFrame || (this._refreshAnimationFrame = this._coreBrowserService.window.requestAnimationFrame(() => this._refresh())), Bi && e4 && this.selectionText.length && this._onLinuxMouseSelection.fire(this.selectionText);
      }
      _refresh() {
        this._refreshAnimationFrame = void 0, this._onRedrawRequest.fire({ start: this._model.finalSelectionStart, end: this._model.finalSelectionEnd, columnSelectMode: this._activeSelectionMode === 3 });
      }
      _isClickInSelection(e4) {
        let i12 = this._getMouseBufferCoords(e4), r7 = this._model.finalSelectionStart, n4 = this._model.finalSelectionEnd;
        return !r7 || !n4 || !i12 ? false : this._areCoordsInSelection(i12, r7, n4);
      }
      isCellInSelection(e4, i12) {
        let r7 = this._model.finalSelectionStart, n4 = this._model.finalSelectionEnd;
        return !r7 || !n4 ? false : this._areCoordsInSelection([e4, i12], r7, n4);
      }
      _areCoordsInSelection(e4, i12, r7) {
        return e4[1] > i12[1] && e4[1] < r7[1] || i12[1] === r7[1] && e4[1] === i12[1] && e4[0] >= i12[0] && e4[0] < r7[0] || i12[1] < r7[1] && e4[1] === r7[1] && e4[0] < r7[0] || i12[1] < r7[1] && e4[1] === i12[1] && e4[0] >= i12[0];
      }
      _selectWordAtCursor(e4, i12) {
        let r7 = this._linkifier.currentLink?.link?.range;
        if (r7) return this._model.selectionStart = [r7.start.x - 1, r7.start.y - 1], this._model.selectionStartLength = ws(r7, this._bufferService.cols), this._model.selectionEnd = void 0, true;
        let n4 = this._getMouseBufferCoords(e4);
        return n4 ? (this._selectWordAt(n4, i12), this._model.selectionEnd = void 0, true) : false;
      }
      selectAll() {
        this._model.isSelectAllActive = true, this.refresh(), this._onSelectionChange.fire();
      }
      selectLines(e4, i12) {
        this._model.clearSelection(), e4 = Math.max(e4, 0), i12 = Math.min(i12, this._bufferService.buffer.lines.length - 1), this._model.selectionStart = [0, e4], this._model.selectionEnd = [this._bufferService.cols, i12], this.refresh(), this._onSelectionChange.fire();
      }
      _handleTrim(e4) {
        this._model.handleTrim(e4) && this.refresh();
      }
      _getMouseBufferCoords(e4) {
        let i12 = this._mouseService.getCoords(e4, this._screenElement, this._bufferService.cols, this._bufferService.rows, true);
        if (i12) return i12[0]--, i12[1]--, i12[1] += this._bufferService.buffer.ydisp, i12;
      }
      _getMouseEventScrollAmount(e4) {
        let i12 = Ci(this._coreBrowserService.window, e4, this._screenElement)[1], r7 = this._renderService.dimensions.css.canvas.height;
        return i12 >= 0 && i12 <= r7 ? 0 : (i12 > r7 && (i12 -= r7), i12 = Math.min(Math.max(i12, -Ds), Ds), i12 /= Ds, i12 / Math.abs(i12) + Math.round(i12 * (Ya - 1)));
      }
      shouldForceSelection(e4) {
        return Zt ? e4.altKey && this._optionsService.rawOptions.macOptionClickForcesSelection : e4.shiftKey;
      }
      handleMouseDown(e4) {
        if (this._mouseDownTimeStamp = e4.timeStamp, !(e4.button === 2 && this.hasSelection) && e4.button === 0) {
          if (!this._enabled) {
            if (!this.shouldForceSelection(e4)) return;
            e4.stopPropagation();
          }
          e4.preventDefault(), this._dragScrollAmount = 0, this._enabled && e4.shiftKey ? this._handleIncrementalClick(e4) : e4.detail === 1 ? this._handleSingleClick(e4) : e4.detail === 2 ? this._handleDoubleClick(e4) : e4.detail === 3 && this._handleTripleClick(e4), this._addMouseDownListeners(), this.refresh(true);
        }
      }
      _addMouseDownListeners() {
        this._screenElement.ownerDocument && (this._screenElement.ownerDocument.addEventListener("mousemove", this._mouseMoveListener), this._screenElement.ownerDocument.addEventListener("mouseup", this._mouseUpListener)), this._dragScrollIntervalTimer = this._coreBrowserService.window.setInterval(() => this._dragScroll(), ja);
      }
      _removeMouseDownListeners() {
        this._screenElement.ownerDocument && (this._screenElement.ownerDocument.removeEventListener("mousemove", this._mouseMoveListener), this._screenElement.ownerDocument.removeEventListener("mouseup", this._mouseUpListener)), this._coreBrowserService.window.clearInterval(this._dragScrollIntervalTimer), this._dragScrollIntervalTimer = void 0;
      }
      _handleIncrementalClick(e4) {
        this._model.selectionStart && (this._model.selectionEnd = this._getMouseBufferCoords(e4));
      }
      _handleSingleClick(e4) {
        if (this._model.selectionStartLength = 0, this._model.isSelectAllActive = false, this._activeSelectionMode = this.shouldColumnSelect(e4) ? 3 : 0, this._model.selectionStart = this._getMouseBufferCoords(e4), !this._model.selectionStart) return;
        this._model.selectionEnd = void 0;
        let i12 = this._bufferService.buffer.lines.get(this._model.selectionStart[1]);
        i12 && i12.length !== this._model.selectionStart[0] && i12.hasWidth(this._model.selectionStart[0]) === 0 && this._model.selectionStart[0]++;
      }
      _handleDoubleClick(e4) {
        this._selectWordAtCursor(e4, true) && (this._activeSelectionMode = 1);
      }
      _handleTripleClick(e4) {
        let i12 = this._getMouseBufferCoords(e4);
        i12 && (this._activeSelectionMode = 2, this._selectLineAt(i12[1]));
      }
      shouldColumnSelect(e4) {
        return e4.altKey && !(Zt && this._optionsService.rawOptions.macOptionClickForcesSelection);
      }
      _handleMouseMove(e4) {
        if (e4.stopImmediatePropagation(), !this._model.selectionStart) return;
        let i12 = this._model.selectionEnd ? [this._model.selectionEnd[0], this._model.selectionEnd[1]] : null;
        if (this._model.selectionEnd = this._getMouseBufferCoords(e4), !this._model.selectionEnd) {
          this.refresh(true);
          return;
        }
        this._activeSelectionMode === 2 ? this._model.selectionEnd[1] < this._model.selectionStart[1] ? this._model.selectionEnd[0] = 0 : this._model.selectionEnd[0] = this._bufferService.cols : this._activeSelectionMode === 1 && this._selectToWordAt(this._model.selectionEnd), this._dragScrollAmount = this._getMouseEventScrollAmount(e4), this._activeSelectionMode !== 3 && (this._dragScrollAmount > 0 ? this._model.selectionEnd[0] = this._bufferService.cols : this._dragScrollAmount < 0 && (this._model.selectionEnd[0] = 0));
        let r7 = this._bufferService.buffer;
        if (this._model.selectionEnd[1] < r7.lines.length) {
          let n4 = r7.lines.get(this._model.selectionEnd[1]);
          n4 && n4.hasWidth(this._model.selectionEnd[0]) === 0 && this._model.selectionEnd[0] < this._bufferService.cols && this._model.selectionEnd[0]++;
        }
        (!i12 || i12[0] !== this._model.selectionEnd[0] || i12[1] !== this._model.selectionEnd[1]) && this.refresh(true);
      }
      _dragScroll() {
        if (!(!this._model.selectionEnd || !this._model.selectionStart) && this._dragScrollAmount) {
          this._onRequestScrollLines.fire({ amount: this._dragScrollAmount, suppressScrollEvent: false });
          let e4 = this._bufferService.buffer;
          this._dragScrollAmount > 0 ? (this._activeSelectionMode !== 3 && (this._model.selectionEnd[0] = this._bufferService.cols), this._model.selectionEnd[1] = Math.min(e4.ydisp + this._bufferService.rows, e4.lines.length - 1)) : (this._activeSelectionMode !== 3 && (this._model.selectionEnd[0] = 0), this._model.selectionEnd[1] = e4.ydisp), this.refresh();
        }
      }
      _handleMouseUp(e4) {
        let i12 = e4.timeStamp - this._mouseDownTimeStamp;
        if (this._removeMouseDownListeners(), this.selectionText.length <= 1 && i12 < Xa && e4.altKey && this._optionsService.rawOptions.altClickMovesCursor) {
          if (this._bufferService.buffer.ybase === this._bufferService.buffer.ydisp) {
            let r7 = this._mouseService.getCoords(e4, this._element, this._bufferService.cols, this._bufferService.rows, false);
            if (r7 && r7[0] !== void 0 && r7[1] !== void 0) {
              let n4 = Jo(r7[0] - 1, r7[1] - 1, this._bufferService, this._coreService.decPrivateModes.applicationCursorKeys);
              this._coreService.triggerDataEvent(n4, true);
            }
          }
        } else this._fireEventIfSelectionChanged();
      }
      _fireEventIfSelectionChanged() {
        let e4 = this._model.finalSelectionStart, i12 = this._model.finalSelectionEnd, r7 = !!e4 && !!i12 && (e4[0] !== i12[0] || e4[1] !== i12[1]);
        if (!r7) {
          this._oldHasSelection && this._fireOnSelectionChange(e4, i12, r7);
          return;
        }
        !e4 || !i12 || (!this._oldSelectionStart || !this._oldSelectionEnd || e4[0] !== this._oldSelectionStart[0] || e4[1] !== this._oldSelectionStart[1] || i12[0] !== this._oldSelectionEnd[0] || i12[1] !== this._oldSelectionEnd[1]) && this._fireOnSelectionChange(e4, i12, r7);
      }
      _fireOnSelectionChange(e4, i12, r7) {
        this._oldSelectionStart = e4, this._oldSelectionEnd = i12, this._oldHasSelection = r7, this._onSelectionChange.fire();
      }
      _handleBufferActivate(e4) {
        this.clearSelection(), this._trimListener.dispose(), this._trimListener = e4.activeBuffer.lines.onTrim((i12) => this._handleTrim(i12));
      }
      _convertViewportColToCharacterIndex(e4, i12) {
        let r7 = i12;
        for (let n4 = 0; i12 >= n4; n4++) {
          let o6 = e4.loadCell(n4, this._workCell).getChars().length;
          this._workCell.getWidth() === 0 ? r7-- : o6 > 1 && i12 !== n4 && (r7 += o6 - 1);
        }
        return r7;
      }
      setSelection(e4, i12, r7) {
        this._model.clearSelection(), this._removeMouseDownListeners(), this._model.selectionStart = [e4, i12], this._model.selectionStartLength = r7, this.refresh(), this._fireEventIfSelectionChanged();
      }
      rightClickSelect(e4) {
        this._isClickInSelection(e4) || (this._selectWordAtCursor(e4, false) && this.refresh(true), this._fireEventIfSelectionChanged());
      }
      _getWordAt(e4, i12, r7 = true, n4 = true) {
        if (e4[0] >= this._bufferService.cols) return;
        let o6 = this._bufferService.buffer, l4 = o6.lines.get(e4[1]);
        if (!l4) return;
        let a3 = o6.translateBufferLineToString(e4[1], false), u3 = this._convertViewportColToCharacterIndex(l4, e4[0]), h4 = u3, c4 = e4[0] - u3, d3 = 0, _4 = 0, p3 = 0, m2 = 0;
        if (a3.charAt(u3) === " ") {
          for (; u3 > 0 && a3.charAt(u3 - 1) === " "; ) u3--;
          for (; h4 < a3.length && a3.charAt(h4 + 1) === " "; ) h4++;
        } else {
          let R2 = e4[0], O3 = e4[0];
          l4.getWidth(R2) === 0 && (d3++, R2--), l4.getWidth(O3) === 2 && (_4++, O3++);
          let I2 = l4.getString(O3).length;
          for (I2 > 1 && (m2 += I2 - 1, h4 += I2 - 1); R2 > 0 && u3 > 0 && !this._isCharWordSeparator(l4.loadCell(R2 - 1, this._workCell)); ) {
            l4.loadCell(R2 - 1, this._workCell);
            let k5 = this._workCell.getChars().length;
            this._workCell.getWidth() === 0 ? (d3++, R2--) : k5 > 1 && (p3 += k5 - 1, u3 -= k5 - 1), u3--, R2--;
          }
          for (; O3 < l4.length && h4 + 1 < a3.length && !this._isCharWordSeparator(l4.loadCell(O3 + 1, this._workCell)); ) {
            l4.loadCell(O3 + 1, this._workCell);
            let k5 = this._workCell.getChars().length;
            this._workCell.getWidth() === 2 ? (_4++, O3++) : k5 > 1 && (m2 += k5 - 1, h4 += k5 - 1), h4++, O3++;
          }
        }
        h4++;
        let f3 = u3 + c4 - d3 + p3, A3 = Math.min(this._bufferService.cols, h4 - u3 + d3 + _4 - p3 - m2);
        if (!(!i12 && a3.slice(u3, h4).trim() === "")) {
          if (r7 && f3 === 0 && l4.getCodePoint(0) !== 32) {
            let R2 = o6.lines.get(e4[1] - 1);
            if (R2 && l4.isWrapped && R2.getCodePoint(this._bufferService.cols - 1) !== 32) {
              let O3 = this._getWordAt([this._bufferService.cols - 1, e4[1] - 1], false, true, false);
              if (O3) {
                let I2 = this._bufferService.cols - O3.start;
                f3 -= I2, A3 += I2;
              }
            }
          }
          if (n4 && f3 + A3 === this._bufferService.cols && l4.getCodePoint(this._bufferService.cols - 1) !== 32) {
            let R2 = o6.lines.get(e4[1] + 1);
            if (R2?.isWrapped && R2.getCodePoint(0) !== 32) {
              let O3 = this._getWordAt([0, e4[1] + 1], false, false, true);
              O3 && (A3 += O3.length);
            }
          }
          return { start: f3, length: A3 };
        }
      }
      _selectWordAt(e4, i12) {
        let r7 = this._getWordAt(e4, i12);
        if (r7) {
          for (; r7.start < 0; ) r7.start += this._bufferService.cols, e4[1]--;
          this._model.selectionStart = [r7.start, e4[1]], this._model.selectionStartLength = r7.length;
        }
      }
      _selectToWordAt(e4) {
        let i12 = this._getWordAt(e4, true);
        if (i12) {
          let r7 = e4[1];
          for (; i12.start < 0; ) i12.start += this._bufferService.cols, r7--;
          if (!this._model.areSelectionValuesReversed()) for (; i12.start + i12.length > this._bufferService.cols; ) i12.length -= this._bufferService.cols, r7++;
          this._model.selectionEnd = [this._model.areSelectionValuesReversed() ? i12.start : i12.start + i12.length, r7];
        }
      }
      _isCharWordSeparator(e4) {
        return e4.getWidth() === 0 ? false : this._optionsService.rawOptions.wordSeparator.indexOf(e4.getChars()) >= 0;
      }
      _selectLineAt(e4) {
        let i12 = this._bufferService.buffer.getWrappedRangeForLine(e4), r7 = { start: { x: 0, y: i12.first }, end: { x: this._bufferService.cols - 1, y: i12.last } };
        this._model.selectionStart = [0, i12.first], this._model.selectionEnd = void 0, this._model.selectionStartLength = ws(r7, this._bufferService.cols);
      }
    };
    ei = M2([S3(3, F), S3(4, ge), S3(5, Dt), S3(6, H2), S3(7, ce), S3(8, ae)], ei);
    Hi = class {
      constructor() {
        this._data = {};
      }
      set(t3, e4, i12) {
        this._data[t3] || (this._data[t3] = {}), this._data[t3][e4] = i12;
      }
      get(t3, e4) {
        return this._data[t3] ? this._data[t3][e4] : void 0;
      }
      clear() {
        this._data = {};
      }
    };
    Wi = class {
      constructor() {
        this._color = new Hi();
        this._css = new Hi();
      }
      setCss(t3, e4, i12) {
        this._css.set(t3, e4, i12);
      }
      getCss(t3, e4) {
        return this._css.get(t3, e4);
      }
      setColor(t3, e4, i12) {
        this._color.set(t3, e4, i12);
      }
      getColor(t3, e4) {
        return this._color.get(t3, e4);
      }
      clear() {
        this._color.clear(), this._css.clear();
      }
    };
    re = Object.freeze((() => {
      let s18 = [z2.toColor("#2e3436"), z2.toColor("#cc0000"), z2.toColor("#4e9a06"), z2.toColor("#c4a000"), z2.toColor("#3465a4"), z2.toColor("#75507b"), z2.toColor("#06989a"), z2.toColor("#d3d7cf"), z2.toColor("#555753"), z2.toColor("#ef2929"), z2.toColor("#8ae234"), z2.toColor("#fce94f"), z2.toColor("#729fcf"), z2.toColor("#ad7fa8"), z2.toColor("#34e2e2"), z2.toColor("#eeeeec")], t3 = [0, 95, 135, 175, 215, 255];
      for (let e4 = 0; e4 < 216; e4++) {
        let i12 = t3[e4 / 36 % 6 | 0], r7 = t3[e4 / 6 % 6 | 0], n4 = t3[e4 % 6];
        s18.push({ css: j.toCss(i12, r7, n4), rgba: j.toRgba(i12, r7, n4) });
      }
      for (let e4 = 0; e4 < 24; e4++) {
        let i12 = 8 + e4 * 10;
        s18.push({ css: j.toCss(i12, i12, i12), rgba: j.toRgba(i12, i12, i12) });
      }
      return s18;
    })());
    St = z2.toColor("#ffffff");
    Ki = z2.toColor("#000000");
    tl = z2.toColor("#ffffff");
    il = Ki;
    Ui = { css: "rgba(255, 255, 255, 0.3)", rgba: 4294967117 };
    Qa = St;
    ti = class extends D2 {
      constructor(e4) {
        super();
        this._optionsService = e4;
        this._contrastCache = new Wi();
        this._halfContrastCache = new Wi();
        this._onChangeColors = this._register(new v2());
        this.onChangeColors = this._onChangeColors.event;
        this._colors = { foreground: St, background: Ki, cursor: tl, cursorAccent: il, selectionForeground: void 0, selectionBackgroundTransparent: Ui, selectionBackgroundOpaque: U.blend(Ki, Ui), selectionInactiveBackgroundTransparent: Ui, selectionInactiveBackgroundOpaque: U.blend(Ki, Ui), scrollbarSliderBackground: U.opacity(St, 0.2), scrollbarSliderHoverBackground: U.opacity(St, 0.4), scrollbarSliderActiveBackground: U.opacity(St, 0.5), overviewRulerBorder: St, ansi: re.slice(), contrastCache: this._contrastCache, halfContrastCache: this._halfContrastCache }, this._updateRestoreColors(), this._setTheme(this._optionsService.rawOptions.theme), this._register(this._optionsService.onSpecificOptionChange("minimumContrastRatio", () => this._contrastCache.clear())), this._register(this._optionsService.onSpecificOptionChange("theme", () => this._setTheme(this._optionsService.rawOptions.theme)));
      }
      get colors() {
        return this._colors;
      }
      _setTheme(e4 = {}) {
        let i12 = this._colors;
        if (i12.foreground = K(e4.foreground, St), i12.background = K(e4.background, Ki), i12.cursor = U.blend(i12.background, K(e4.cursor, tl)), i12.cursorAccent = U.blend(i12.background, K(e4.cursorAccent, il)), i12.selectionBackgroundTransparent = K(e4.selectionBackground, Ui), i12.selectionBackgroundOpaque = U.blend(i12.background, i12.selectionBackgroundTransparent), i12.selectionInactiveBackgroundTransparent = K(e4.selectionInactiveBackground, i12.selectionBackgroundTransparent), i12.selectionInactiveBackgroundOpaque = U.blend(i12.background, i12.selectionInactiveBackgroundTransparent), i12.selectionForeground = e4.selectionForeground ? K(e4.selectionForeground, ps) : void 0, i12.selectionForeground === ps && (i12.selectionForeground = void 0), U.isOpaque(i12.selectionBackgroundTransparent) && (i12.selectionBackgroundTransparent = U.opacity(i12.selectionBackgroundTransparent, 0.3)), U.isOpaque(i12.selectionInactiveBackgroundTransparent) && (i12.selectionInactiveBackgroundTransparent = U.opacity(i12.selectionInactiveBackgroundTransparent, 0.3)), i12.scrollbarSliderBackground = K(e4.scrollbarSliderBackground, U.opacity(i12.foreground, 0.2)), i12.scrollbarSliderHoverBackground = K(e4.scrollbarSliderHoverBackground, U.opacity(i12.foreground, 0.4)), i12.scrollbarSliderActiveBackground = K(e4.scrollbarSliderActiveBackground, U.opacity(i12.foreground, 0.5)), i12.overviewRulerBorder = K(e4.overviewRulerBorder, Qa), i12.ansi = re.slice(), i12.ansi[0] = K(e4.black, re[0]), i12.ansi[1] = K(e4.red, re[1]), i12.ansi[2] = K(e4.green, re[2]), i12.ansi[3] = K(e4.yellow, re[3]), i12.ansi[4] = K(e4.blue, re[4]), i12.ansi[5] = K(e4.magenta, re[5]), i12.ansi[6] = K(e4.cyan, re[6]), i12.ansi[7] = K(e4.white, re[7]), i12.ansi[8] = K(e4.brightBlack, re[8]), i12.ansi[9] = K(e4.brightRed, re[9]), i12.ansi[10] = K(e4.brightGreen, re[10]), i12.ansi[11] = K(e4.brightYellow, re[11]), i12.ansi[12] = K(e4.brightBlue, re[12]), i12.ansi[13] = K(e4.brightMagenta, re[13]), i12.ansi[14] = K(e4.brightCyan, re[14]), i12.ansi[15] = K(e4.brightWhite, re[15]), e4.extendedAnsi) {
          let r7 = Math.min(i12.ansi.length - 16, e4.extendedAnsi.length);
          for (let n4 = 0; n4 < r7; n4++) i12.ansi[n4 + 16] = K(e4.extendedAnsi[n4], re[n4 + 16]);
        }
        this._contrastCache.clear(), this._halfContrastCache.clear(), this._updateRestoreColors(), this._onChangeColors.fire(this.colors);
      }
      restoreColor(e4) {
        this._restoreColor(e4), this._onChangeColors.fire(this.colors);
      }
      _restoreColor(e4) {
        if (e4 === void 0) {
          for (let i12 = 0; i12 < this._restoreColors.ansi.length; ++i12) this._colors.ansi[i12] = this._restoreColors.ansi[i12];
          return;
        }
        switch (e4) {
          case 256:
            this._colors.foreground = this._restoreColors.foreground;
            break;
          case 257:
            this._colors.background = this._restoreColors.background;
            break;
          case 258:
            this._colors.cursor = this._restoreColors.cursor;
            break;
          default:
            this._colors.ansi[e4] = this._restoreColors.ansi[e4];
        }
      }
      modifyColors(e4) {
        e4(this._colors), this._onChangeColors.fire(this.colors);
      }
      _updateRestoreColors() {
        this._restoreColors = { foreground: this._colors.foreground, background: this._colors.background, cursor: this._colors.cursor, ansi: this._colors.ansi.slice() };
      }
    };
    ti = M2([S3(0, H2)], ti);
    Rs = class {
      constructor(...t3) {
        this._entries = /* @__PURE__ */ new Map();
        for (let [e4, i12] of t3) this.set(e4, i12);
      }
      set(t3, e4) {
        let i12 = this._entries.get(t3);
        return this._entries.set(t3, e4), i12;
      }
      forEach(t3) {
        for (let [e4, i12] of this._entries.entries()) t3(e4, i12);
      }
      has(t3) {
        return this._entries.has(t3);
      }
      get(t3) {
        return this._entries.get(t3);
      }
    };
    ln = class {
      constructor() {
        this._services = new Rs();
        this._services.set(xt, this);
      }
      setService(t3, e4) {
        this._services.set(t3, e4);
      }
      getService(t3) {
        return this._services.get(t3);
      }
      createInstance(t3, ...e4) {
        let i12 = Xs(t3).sort((o6, l4) => o6.index - l4.index), r7 = [];
        for (let o6 of i12) {
          let l4 = this._services.get(o6.id);
          if (!l4) throw new Error(`[createInstance] ${t3.name} depends on UNKNOWN service ${o6.id._id}.`);
          r7.push(l4);
        }
        let n4 = i12.length > 0 ? i12[0].index : e4.length;
        if (e4.length !== n4) throw new Error(`[createInstance] First service dependency of ${t3.name} at position ${n4 + 1} conflicts with ${e4.length} static arguments`);
        return new t3(...e4, ...r7);
      }
    };
    ec = { trace: 0, debug: 1, info: 2, warn: 3, error: 4, off: 5 };
    tc = "xterm.js: ";
    ii = class extends D2 {
      constructor(e4) {
        super();
        this._optionsService = e4;
        this._logLevel = 5;
        this._updateLogLevel(), this._register(this._optionsService.onSpecificOptionChange("logLevel", () => this._updateLogLevel())), ic = this;
      }
      get logLevel() {
        return this._logLevel;
      }
      _updateLogLevel() {
        this._logLevel = ec[this._optionsService.rawOptions.logLevel];
      }
      _evalLazyOptionalParams(e4) {
        for (let i12 = 0; i12 < e4.length; i12++) typeof e4[i12] == "function" && (e4[i12] = e4[i12]());
      }
      _log(e4, i12, r7) {
        this._evalLazyOptionalParams(r7), e4.call(console, (this._optionsService.options.logger ? "" : tc) + i12, ...r7);
      }
      trace(e4, ...i12) {
        this._logLevel <= 0 && this._log(this._optionsService.options.logger?.trace.bind(this._optionsService.options.logger) ?? console.log, e4, i12);
      }
      debug(e4, ...i12) {
        this._logLevel <= 1 && this._log(this._optionsService.options.logger?.debug.bind(this._optionsService.options.logger) ?? console.log, e4, i12);
      }
      info(e4, ...i12) {
        this._logLevel <= 2 && this._log(this._optionsService.options.logger?.info.bind(this._optionsService.options.logger) ?? console.info, e4, i12);
      }
      warn(e4, ...i12) {
        this._logLevel <= 3 && this._log(this._optionsService.options.logger?.warn.bind(this._optionsService.options.logger) ?? console.warn, e4, i12);
      }
      error(e4, ...i12) {
        this._logLevel <= 4 && this._log(this._optionsService.options.logger?.error.bind(this._optionsService.options.logger) ?? console.error, e4, i12);
      }
    };
    ii = M2([S3(0, H2)], ii);
    zi = class extends D2 {
      constructor(e4) {
        super();
        this._maxLength = e4;
        this.onDeleteEmitter = this._register(new v2());
        this.onDelete = this.onDeleteEmitter.event;
        this.onInsertEmitter = this._register(new v2());
        this.onInsert = this.onInsertEmitter.event;
        this.onTrimEmitter = this._register(new v2());
        this.onTrim = this.onTrimEmitter.event;
        this._array = new Array(this._maxLength), this._startIndex = 0, this._length = 0;
      }
      get maxLength() {
        return this._maxLength;
      }
      set maxLength(e4) {
        if (this._maxLength === e4) return;
        let i12 = new Array(e4);
        for (let r7 = 0; r7 < Math.min(e4, this.length); r7++) i12[r7] = this._array[this._getCyclicIndex(r7)];
        this._array = i12, this._maxLength = e4, this._startIndex = 0;
      }
      get length() {
        return this._length;
      }
      set length(e4) {
        if (e4 > this._length) for (let i12 = this._length; i12 < e4; i12++) this._array[i12] = void 0;
        this._length = e4;
      }
      get(e4) {
        return this._array[this._getCyclicIndex(e4)];
      }
      set(e4, i12) {
        this._array[this._getCyclicIndex(e4)] = i12;
      }
      push(e4) {
        this._array[this._getCyclicIndex(this._length)] = e4, this._length === this._maxLength ? (this._startIndex = ++this._startIndex % this._maxLength, this.onTrimEmitter.fire(1)) : this._length++;
      }
      recycle() {
        if (this._length !== this._maxLength) throw new Error("Can only recycle when the buffer is full");
        return this._startIndex = ++this._startIndex % this._maxLength, this.onTrimEmitter.fire(1), this._array[this._getCyclicIndex(this._length - 1)];
      }
      get isFull() {
        return this._length === this._maxLength;
      }
      pop() {
        return this._array[this._getCyclicIndex(this._length-- - 1)];
      }
      splice(e4, i12, ...r7) {
        if (i12) {
          for (let n4 = e4; n4 < this._length - i12; n4++) this._array[this._getCyclicIndex(n4)] = this._array[this._getCyclicIndex(n4 + i12)];
          this._length -= i12, this.onDeleteEmitter.fire({ index: e4, amount: i12 });
        }
        for (let n4 = this._length - 1; n4 >= e4; n4--) this._array[this._getCyclicIndex(n4 + r7.length)] = this._array[this._getCyclicIndex(n4)];
        for (let n4 = 0; n4 < r7.length; n4++) this._array[this._getCyclicIndex(e4 + n4)] = r7[n4];
        if (r7.length && this.onInsertEmitter.fire({ index: e4, amount: r7.length }), this._length + r7.length > this._maxLength) {
          let n4 = this._length + r7.length - this._maxLength;
          this._startIndex += n4, this._length = this._maxLength, this.onTrimEmitter.fire(n4);
        } else this._length += r7.length;
      }
      trimStart(e4) {
        e4 > this._length && (e4 = this._length), this._startIndex += e4, this._length -= e4, this.onTrimEmitter.fire(e4);
      }
      shiftElements(e4, i12, r7) {
        if (!(i12 <= 0)) {
          if (e4 < 0 || e4 >= this._length) throw new Error("start argument out of range");
          if (e4 + r7 < 0) throw new Error("Cannot shift elements in list beyond index 0");
          if (r7 > 0) {
            for (let o6 = i12 - 1; o6 >= 0; o6--) this.set(e4 + o6 + r7, this.get(e4 + o6));
            let n4 = e4 + i12 + r7 - this._length;
            if (n4 > 0) for (this._length += n4; this._length > this._maxLength; ) this._length--, this._startIndex++, this.onTrimEmitter.fire(1);
          } else for (let n4 = 0; n4 < i12; n4++) this.set(e4 + n4 + r7, this.get(e4 + n4));
        }
      }
      _getCyclicIndex(e4) {
        return (this._startIndex + e4) % this._maxLength;
      }
    };
    B2 = 3;
    X = Object.freeze(new De());
    an = 0;
    Ls = 2;
    Ze = class s15 {
      constructor(t3, e4, i12 = false) {
        this.isWrapped = i12;
        this._combined = {};
        this._extendedAttrs = {};
        this._data = new Uint32Array(t3 * B2);
        let r7 = e4 || q.fromCharData([0, ir, 1, 0]);
        for (let n4 = 0; n4 < t3; ++n4) this.setCell(n4, r7);
        this.length = t3;
      }
      get(t3) {
        let e4 = this._data[t3 * B2 + 0], i12 = e4 & 2097151;
        return [this._data[t3 * B2 + 1], e4 & 2097152 ? this._combined[t3] : i12 ? Ce(i12) : "", e4 >> 22, e4 & 2097152 ? this._combined[t3].charCodeAt(this._combined[t3].length - 1) : i12];
      }
      set(t3, e4) {
        this._data[t3 * B2 + 1] = e4[0], e4[1].length > 1 ? (this._combined[t3] = e4[1], this._data[t3 * B2 + 0] = t3 | 2097152 | e4[2] << 22) : this._data[t3 * B2 + 0] = e4[1].charCodeAt(0) | e4[2] << 22;
      }
      getWidth(t3) {
        return this._data[t3 * B2 + 0] >> 22;
      }
      hasWidth(t3) {
        return this._data[t3 * B2 + 0] & 12582912;
      }
      getFg(t3) {
        return this._data[t3 * B2 + 1];
      }
      getBg(t3) {
        return this._data[t3 * B2 + 2];
      }
      hasContent(t3) {
        return this._data[t3 * B2 + 0] & 4194303;
      }
      getCodePoint(t3) {
        let e4 = this._data[t3 * B2 + 0];
        return e4 & 2097152 ? this._combined[t3].charCodeAt(this._combined[t3].length - 1) : e4 & 2097151;
      }
      isCombined(t3) {
        return this._data[t3 * B2 + 0] & 2097152;
      }
      getString(t3) {
        let e4 = this._data[t3 * B2 + 0];
        return e4 & 2097152 ? this._combined[t3] : e4 & 2097151 ? Ce(e4 & 2097151) : "";
      }
      isProtected(t3) {
        return this._data[t3 * B2 + 2] & 536870912;
      }
      loadCell(t3, e4) {
        return an = t3 * B2, e4.content = this._data[an + 0], e4.fg = this._data[an + 1], e4.bg = this._data[an + 2], e4.content & 2097152 && (e4.combinedData = this._combined[t3]), e4.bg & 268435456 && (e4.extended = this._extendedAttrs[t3]), e4;
      }
      setCell(t3, e4) {
        e4.content & 2097152 && (this._combined[t3] = e4.combinedData), e4.bg & 268435456 && (this._extendedAttrs[t3] = e4.extended), this._data[t3 * B2 + 0] = e4.content, this._data[t3 * B2 + 1] = e4.fg, this._data[t3 * B2 + 2] = e4.bg;
      }
      setCellFromCodepoint(t3, e4, i12, r7) {
        r7.bg & 268435456 && (this._extendedAttrs[t3] = r7.extended), this._data[t3 * B2 + 0] = e4 | i12 << 22, this._data[t3 * B2 + 1] = r7.fg, this._data[t3 * B2 + 2] = r7.bg;
      }
      addCodepointToCell(t3, e4, i12) {
        let r7 = this._data[t3 * B2 + 0];
        r7 & 2097152 ? this._combined[t3] += Ce(e4) : r7 & 2097151 ? (this._combined[t3] = Ce(r7 & 2097151) + Ce(e4), r7 &= -2097152, r7 |= 2097152) : r7 = e4 | 1 << 22, i12 && (r7 &= -12582913, r7 |= i12 << 22), this._data[t3 * B2 + 0] = r7;
      }
      insertCells(t3, e4, i12) {
        if (t3 %= this.length, t3 && this.getWidth(t3 - 1) === 2 && this.setCellFromCodepoint(t3 - 1, 0, 1, i12), e4 < this.length - t3) {
          let r7 = new q();
          for (let n4 = this.length - t3 - e4 - 1; n4 >= 0; --n4) this.setCell(t3 + e4 + n4, this.loadCell(t3 + n4, r7));
          for (let n4 = 0; n4 < e4; ++n4) this.setCell(t3 + n4, i12);
        } else for (let r7 = t3; r7 < this.length; ++r7) this.setCell(r7, i12);
        this.getWidth(this.length - 1) === 2 && this.setCellFromCodepoint(this.length - 1, 0, 1, i12);
      }
      deleteCells(t3, e4, i12) {
        if (t3 %= this.length, e4 < this.length - t3) {
          let r7 = new q();
          for (let n4 = 0; n4 < this.length - t3 - e4; ++n4) this.setCell(t3 + n4, this.loadCell(t3 + e4 + n4, r7));
          for (let n4 = this.length - e4; n4 < this.length; ++n4) this.setCell(n4, i12);
        } else for (let r7 = t3; r7 < this.length; ++r7) this.setCell(r7, i12);
        t3 && this.getWidth(t3 - 1) === 2 && this.setCellFromCodepoint(t3 - 1, 0, 1, i12), this.getWidth(t3) === 0 && !this.hasContent(t3) && this.setCellFromCodepoint(t3, 0, 1, i12);
      }
      replaceCells(t3, e4, i12, r7 = false) {
        if (r7) {
          for (t3 && this.getWidth(t3 - 1) === 2 && !this.isProtected(t3 - 1) && this.setCellFromCodepoint(t3 - 1, 0, 1, i12), e4 < this.length && this.getWidth(e4 - 1) === 2 && !this.isProtected(e4) && this.setCellFromCodepoint(e4, 0, 1, i12); t3 < e4 && t3 < this.length; ) this.isProtected(t3) || this.setCell(t3, i12), t3++;
          return;
        }
        for (t3 && this.getWidth(t3 - 1) === 2 && this.setCellFromCodepoint(t3 - 1, 0, 1, i12), e4 < this.length && this.getWidth(e4 - 1) === 2 && this.setCellFromCodepoint(e4, 0, 1, i12); t3 < e4 && t3 < this.length; ) this.setCell(t3++, i12);
      }
      resize(t3, e4) {
        if (t3 === this.length) return this._data.length * 4 * Ls < this._data.buffer.byteLength;
        let i12 = t3 * B2;
        if (t3 > this.length) {
          if (this._data.buffer.byteLength >= i12 * 4) this._data = new Uint32Array(this._data.buffer, 0, i12);
          else {
            let r7 = new Uint32Array(i12);
            r7.set(this._data), this._data = r7;
          }
          for (let r7 = this.length; r7 < t3; ++r7) this.setCell(r7, e4);
        } else {
          this._data = this._data.subarray(0, i12);
          let r7 = Object.keys(this._combined);
          for (let o6 = 0; o6 < r7.length; o6++) {
            let l4 = parseInt(r7[o6], 10);
            l4 >= t3 && delete this._combined[l4];
          }
          let n4 = Object.keys(this._extendedAttrs);
          for (let o6 = 0; o6 < n4.length; o6++) {
            let l4 = parseInt(n4[o6], 10);
            l4 >= t3 && delete this._extendedAttrs[l4];
          }
        }
        return this.length = t3, i12 * 4 * Ls < this._data.buffer.byteLength;
      }
      cleanupMemory() {
        if (this._data.length * 4 * Ls < this._data.buffer.byteLength) {
          let t3 = new Uint32Array(this._data.length);
          return t3.set(this._data), this._data = t3, 1;
        }
        return 0;
      }
      fill(t3, e4 = false) {
        if (e4) {
          for (let i12 = 0; i12 < this.length; ++i12) this.isProtected(i12) || this.setCell(i12, t3);
          return;
        }
        this._combined = {}, this._extendedAttrs = {};
        for (let i12 = 0; i12 < this.length; ++i12) this.setCell(i12, t3);
      }
      copyFrom(t3) {
        this.length !== t3.length ? this._data = new Uint32Array(t3._data) : this._data.set(t3._data), this.length = t3.length, this._combined = {};
        for (let e4 in t3._combined) this._combined[e4] = t3._combined[e4];
        this._extendedAttrs = {};
        for (let e4 in t3._extendedAttrs) this._extendedAttrs[e4] = t3._extendedAttrs[e4];
        this.isWrapped = t3.isWrapped;
      }
      clone() {
        let t3 = new s15(0);
        t3._data = new Uint32Array(this._data), t3.length = this.length;
        for (let e4 in this._combined) t3._combined[e4] = this._combined[e4];
        for (let e4 in this._extendedAttrs) t3._extendedAttrs[e4] = this._extendedAttrs[e4];
        return t3.isWrapped = this.isWrapped, t3;
      }
      getTrimmedLength() {
        for (let t3 = this.length - 1; t3 >= 0; --t3) if (this._data[t3 * B2 + 0] & 4194303) return t3 + (this._data[t3 * B2 + 0] >> 22);
        return 0;
      }
      getNoBgTrimmedLength() {
        for (let t3 = this.length - 1; t3 >= 0; --t3) if (this._data[t3 * B2 + 0] & 4194303 || this._data[t3 * B2 + 2] & 50331648) return t3 + (this._data[t3 * B2 + 0] >> 22);
        return 0;
      }
      copyCellsFrom(t3, e4, i12, r7, n4) {
        let o6 = t3._data;
        if (n4) for (let a3 = r7 - 1; a3 >= 0; a3--) {
          for (let u3 = 0; u3 < B2; u3++) this._data[(i12 + a3) * B2 + u3] = o6[(e4 + a3) * B2 + u3];
          o6[(e4 + a3) * B2 + 2] & 268435456 && (this._extendedAttrs[i12 + a3] = t3._extendedAttrs[e4 + a3]);
        }
        else for (let a3 = 0; a3 < r7; a3++) {
          for (let u3 = 0; u3 < B2; u3++) this._data[(i12 + a3) * B2 + u3] = o6[(e4 + a3) * B2 + u3];
          o6[(e4 + a3) * B2 + 2] & 268435456 && (this._extendedAttrs[i12 + a3] = t3._extendedAttrs[e4 + a3]);
        }
        let l4 = Object.keys(t3._combined);
        for (let a3 = 0; a3 < l4.length; a3++) {
          let u3 = parseInt(l4[a3], 10);
          u3 >= e4 && (this._combined[u3 - e4 + i12] = t3._combined[u3]);
        }
      }
      translateToString(t3, e4, i12, r7) {
        e4 = e4 ?? 0, i12 = i12 ?? this.length, t3 && (i12 = Math.min(i12, this.getTrimmedLength())), r7 && (r7.length = 0);
        let n4 = "";
        for (; e4 < i12; ) {
          let o6 = this._data[e4 * B2 + 0], l4 = o6 & 2097151, a3 = o6 & 2097152 ? this._combined[e4] : l4 ? Ce(l4) : we;
          if (n4 += a3, r7) for (let u3 = 0; u3 < a3.length; ++u3) r7.push(e4);
          e4 += o6 >> 22 || 1;
        }
        return r7 && r7.push(e4), n4;
      }
    };
    un = class un2 {
      constructor(t3) {
        this.line = t3;
        this.isDisposed = false;
        this._disposables = [];
        this._id = un2._nextId++;
        this._onDispose = this.register(new v2());
        this.onDispose = this._onDispose.event;
      }
      get id() {
        return this._id;
      }
      dispose() {
        this.isDisposed || (this.isDisposed = true, this.line = -1, this._onDispose.fire(), Ne(this._disposables), this._disposables.length = 0);
      }
      register(t3) {
        return this._disposables.push(t3), t3;
      }
    };
    un._nextId = 1;
    cn = un;
    ne = {};
    Je = ne.B;
    ne[0] = { "`": "\u25C6", a: "\u2592", b: "\u2409", c: "\u240C", d: "\u240D", e: "\u240A", f: "\xB0", g: "\xB1", h: "\u2424", i: "\u240B", j: "\u2518", k: "\u2510", l: "\u250C", m: "\u2514", n: "\u253C", o: "\u23BA", p: "\u23BB", q: "\u2500", r: "\u23BC", s: "\u23BD", t: "\u251C", u: "\u2524", v: "\u2534", w: "\u252C", x: "\u2502", y: "\u2264", z: "\u2265", "{": "\u03C0", "|": "\u2260", "}": "\xA3", "~": "\xB7" };
    ne.A = { "#": "\xA3" };
    ne.B = void 0;
    ne[4] = { "#": "\xA3", "@": "\xBE", "[": "ij", "\\": "\xBD", "]": "|", "{": "\xA8", "|": "f", "}": "\xBC", "~": "\xB4" };
    ne.C = ne[5] = { "[": "\xC4", "\\": "\xD6", "]": "\xC5", "^": "\xDC", "`": "\xE9", "{": "\xE4", "|": "\xF6", "}": "\xE5", "~": "\xFC" };
    ne.R = { "#": "\xA3", "@": "\xE0", "[": "\xB0", "\\": "\xE7", "]": "\xA7", "{": "\xE9", "|": "\xF9", "}": "\xE8", "~": "\xA8" };
    ne.Q = { "@": "\xE0", "[": "\xE2", "\\": "\xE7", "]": "\xEA", "^": "\xEE", "`": "\xF4", "{": "\xE9", "|": "\xF9", "}": "\xE8", "~": "\xFB" };
    ne.K = { "@": "\xA7", "[": "\xC4", "\\": "\xD6", "]": "\xDC", "{": "\xE4", "|": "\xF6", "}": "\xFC", "~": "\xDF" };
    ne.Y = { "#": "\xA3", "@": "\xA7", "[": "\xB0", "\\": "\xE7", "]": "\xE9", "`": "\xF9", "{": "\xE0", "|": "\xF2", "}": "\xE8", "~": "\xEC" };
    ne.E = ne[6] = { "@": "\xC4", "[": "\xC6", "\\": "\xD8", "]": "\xC5", "^": "\xDC", "`": "\xE4", "{": "\xE6", "|": "\xF8", "}": "\xE5", "~": "\xFC" };
    ne.Z = { "#": "\xA3", "@": "\xA7", "[": "\xA1", "\\": "\xD1", "]": "\xBF", "{": "\xB0", "|": "\xF1", "}": "\xE7" };
    ne.H = ne[7] = { "@": "\xC9", "[": "\xC4", "\\": "\xD6", "]": "\xC5", "^": "\xDC", "`": "\xE9", "{": "\xE4", "|": "\xF6", "}": "\xE5", "~": "\xFC" };
    ne["="] = { "#": "\xF9", "@": "\xE0", "[": "\xE9", "\\": "\xE7", "]": "\xEA", "^": "\xEE", _: "\xE8", "`": "\xF4", "{": "\xE4", "|": "\xF6", "}": "\xFC", "~": "\xFB" };
    cl = 4294967295;
    $i = class {
      constructor(t3, e4, i12) {
        this._hasScrollback = t3;
        this._optionsService = e4;
        this._bufferService = i12;
        this.ydisp = 0;
        this.ybase = 0;
        this.y = 0;
        this.x = 0;
        this.tabs = {};
        this.savedY = 0;
        this.savedX = 0;
        this.savedCurAttrData = X.clone();
        this.savedCharset = Je;
        this.markers = [];
        this._nullCell = q.fromCharData([0, ir, 1, 0]);
        this._whitespaceCell = q.fromCharData([0, we, 1, 32]);
        this._isClearing = false;
        this._memoryCleanupQueue = new Jt();
        this._memoryCleanupPosition = 0;
        this._cols = this._bufferService.cols, this._rows = this._bufferService.rows, this.lines = new zi(this._getCorrectBufferLength(this._rows)), this.scrollTop = 0, this.scrollBottom = this._rows - 1, this.setupTabStops();
      }
      getNullCell(t3) {
        return t3 ? (this._nullCell.fg = t3.fg, this._nullCell.bg = t3.bg, this._nullCell.extended = t3.extended) : (this._nullCell.fg = 0, this._nullCell.bg = 0, this._nullCell.extended = new rt()), this._nullCell;
      }
      getWhitespaceCell(t3) {
        return t3 ? (this._whitespaceCell.fg = t3.fg, this._whitespaceCell.bg = t3.bg, this._whitespaceCell.extended = t3.extended) : (this._whitespaceCell.fg = 0, this._whitespaceCell.bg = 0, this._whitespaceCell.extended = new rt()), this._whitespaceCell;
      }
      getBlankLine(t3, e4) {
        return new Ze(this._bufferService.cols, this.getNullCell(t3), e4);
      }
      get hasScrollback() {
        return this._hasScrollback && this.lines.maxLength > this._rows;
      }
      get isCursorInViewport() {
        let e4 = this.ybase + this.y - this.ydisp;
        return e4 >= 0 && e4 < this._rows;
      }
      _getCorrectBufferLength(t3) {
        if (!this._hasScrollback) return t3;
        let e4 = t3 + this._optionsService.rawOptions.scrollback;
        return e4 > cl ? cl : e4;
      }
      fillViewportRows(t3) {
        if (this.lines.length === 0) {
          t3 === void 0 && (t3 = X);
          let e4 = this._rows;
          for (; e4--; ) this.lines.push(this.getBlankLine(t3));
        }
      }
      clear() {
        this.ydisp = 0, this.ybase = 0, this.y = 0, this.x = 0, this.lines = new zi(this._getCorrectBufferLength(this._rows)), this.scrollTop = 0, this.scrollBottom = this._rows - 1, this.setupTabStops();
      }
      resize(t3, e4) {
        let i12 = this.getNullCell(X), r7 = 0, n4 = this._getCorrectBufferLength(e4);
        if (n4 > this.lines.maxLength && (this.lines.maxLength = n4), this.lines.length > 0) {
          if (this._cols < t3) for (let l4 = 0; l4 < this.lines.length; l4++) r7 += +this.lines.get(l4).resize(t3, i12);
          let o6 = 0;
          if (this._rows < e4) for (let l4 = this._rows; l4 < e4; l4++) this.lines.length < e4 + this.ybase && (this._optionsService.rawOptions.windowsMode || this._optionsService.rawOptions.windowsPty.backend !== void 0 || this._optionsService.rawOptions.windowsPty.buildNumber !== void 0 ? this.lines.push(new Ze(t3, i12)) : this.ybase > 0 && this.lines.length <= this.ybase + this.y + o6 + 1 ? (this.ybase--, o6++, this.ydisp > 0 && this.ydisp--) : this.lines.push(new Ze(t3, i12)));
          else for (let l4 = this._rows; l4 > e4; l4--) this.lines.length > e4 + this.ybase && (this.lines.length > this.ybase + this.y + 1 ? this.lines.pop() : (this.ybase++, this.ydisp++));
          if (n4 < this.lines.maxLength) {
            let l4 = this.lines.length - n4;
            l4 > 0 && (this.lines.trimStart(l4), this.ybase = Math.max(this.ybase - l4, 0), this.ydisp = Math.max(this.ydisp - l4, 0), this.savedY = Math.max(this.savedY - l4, 0)), this.lines.maxLength = n4;
          }
          this.x = Math.min(this.x, t3 - 1), this.y = Math.min(this.y, e4 - 1), o6 && (this.y += o6), this.savedX = Math.min(this.savedX, t3 - 1), this.scrollTop = 0;
        }
        if (this.scrollBottom = e4 - 1, this._isReflowEnabled && (this._reflow(t3, e4), this._cols > t3)) for (let o6 = 0; o6 < this.lines.length; o6++) r7 += +this.lines.get(o6).resize(t3, i12);
        this._cols = t3, this._rows = e4, this._memoryCleanupQueue.clear(), r7 > 0.1 * this.lines.length && (this._memoryCleanupPosition = 0, this._memoryCleanupQueue.enqueue(() => this._batchedMemoryCleanup()));
      }
      _batchedMemoryCleanup() {
        let t3 = true;
        this._memoryCleanupPosition >= this.lines.length && (this._memoryCleanupPosition = 0, t3 = false);
        let e4 = 0;
        for (; this._memoryCleanupPosition < this.lines.length; ) if (e4 += this.lines.get(this._memoryCleanupPosition++).cleanupMemory(), e4 > 100) return true;
        return t3;
      }
      get _isReflowEnabled() {
        let t3 = this._optionsService.rawOptions.windowsPty;
        return t3 && t3.buildNumber ? this._hasScrollback && t3.backend === "conpty" && t3.buildNumber >= 21376 : this._hasScrollback && !this._optionsService.rawOptions.windowsMode;
      }
      _reflow(t3, e4) {
        this._cols !== t3 && (t3 > this._cols ? this._reflowLarger(t3, e4) : this._reflowSmaller(t3, e4));
      }
      _reflowLarger(t3, e4) {
        let i12 = this._optionsService.rawOptions.reflowCursorLine, r7 = sl(this.lines, this._cols, t3, this.ybase + this.y, this.getNullCell(X), i12);
        if (r7.length > 0) {
          let n4 = ol(this.lines, r7);
          ll(this.lines, n4.layout), this._reflowLargerAdjustViewport(t3, e4, n4.countRemoved);
        }
      }
      _reflowLargerAdjustViewport(t3, e4, i12) {
        let r7 = this.getNullCell(X), n4 = i12;
        for (; n4-- > 0; ) this.ybase === 0 ? (this.y > 0 && this.y--, this.lines.length < e4 && this.lines.push(new Ze(t3, r7))) : (this.ydisp === this.ybase && this.ydisp--, this.ybase--);
        this.savedY = Math.max(this.savedY - i12, 0);
      }
      _reflowSmaller(t3, e4) {
        let i12 = this._optionsService.rawOptions.reflowCursorLine, r7 = this.getNullCell(X), n4 = [], o6 = 0;
        for (let l4 = this.lines.length - 1; l4 >= 0; l4--) {
          let a3 = this.lines.get(l4);
          if (!a3 || !a3.isWrapped && a3.getTrimmedLength() <= t3) continue;
          let u3 = [a3];
          for (; a3.isWrapped && l4 > 0; ) a3 = this.lines.get(--l4), u3.unshift(a3);
          if (!i12) {
            let I2 = this.ybase + this.y;
            if (I2 >= l4 && I2 < l4 + u3.length) continue;
          }
          let h4 = u3[u3.length - 1].getTrimmedLength(), c4 = al(u3, this._cols, t3), d3 = c4.length - u3.length, _4;
          this.ybase === 0 && this.y !== this.lines.length - 1 ? _4 = Math.max(0, this.y - this.lines.maxLength + d3) : _4 = Math.max(0, this.lines.length - this.lines.maxLength + d3);
          let p3 = [];
          for (let I2 = 0; I2 < d3; I2++) {
            let k5 = this.getBlankLine(X, true);
            p3.push(k5);
          }
          p3.length > 0 && (n4.push({ start: l4 + u3.length + o6, newLines: p3 }), o6 += p3.length), u3.push(...p3);
          let m2 = c4.length - 1, f3 = c4[m2];
          f3 === 0 && (m2--, f3 = c4[m2]);
          let A3 = u3.length - d3 - 1, R2 = h4;
          for (; A3 >= 0; ) {
            let I2 = Math.min(R2, f3);
            if (u3[m2] === void 0) break;
            if (u3[m2].copyCellsFrom(u3[A3], R2 - I2, f3 - I2, I2, true), f3 -= I2, f3 === 0 && (m2--, f3 = c4[m2]), R2 -= I2, R2 === 0) {
              A3--;
              let k5 = Math.max(A3, 0);
              R2 = ri(u3, k5, this._cols);
            }
          }
          for (let I2 = 0; I2 < u3.length; I2++) c4[I2] < t3 && u3[I2].setCell(c4[I2], r7);
          let O3 = d3 - _4;
          for (; O3-- > 0; ) this.ybase === 0 ? this.y < e4 - 1 ? (this.y++, this.lines.pop()) : (this.ybase++, this.ydisp++) : this.ybase < Math.min(this.lines.maxLength, this.lines.length + o6) - e4 && (this.ybase === this.ydisp && this.ydisp++, this.ybase++);
          this.savedY = Math.min(this.savedY + d3, this.ybase + e4 - 1);
        }
        if (n4.length > 0) {
          let l4 = [], a3 = [];
          for (let f3 = 0; f3 < this.lines.length; f3++) a3.push(this.lines.get(f3));
          let u3 = this.lines.length, h4 = u3 - 1, c4 = 0, d3 = n4[c4];
          this.lines.length = Math.min(this.lines.maxLength, this.lines.length + o6);
          let _4 = 0;
          for (let f3 = Math.min(this.lines.maxLength - 1, u3 + o6 - 1); f3 >= 0; f3--) if (d3 && d3.start > h4 + _4) {
            for (let A3 = d3.newLines.length - 1; A3 >= 0; A3--) this.lines.set(f3--, d3.newLines[A3]);
            f3++, l4.push({ index: h4 + 1, amount: d3.newLines.length }), _4 += d3.newLines.length, d3 = n4[++c4];
          } else this.lines.set(f3, a3[h4--]);
          let p3 = 0;
          for (let f3 = l4.length - 1; f3 >= 0; f3--) l4[f3].index += p3, this.lines.onInsertEmitter.fire(l4[f3]), p3 += l4[f3].amount;
          let m2 = Math.max(0, u3 + o6 - this.lines.maxLength);
          m2 > 0 && this.lines.onTrimEmitter.fire(m2);
        }
      }
      translateBufferLineToString(t3, e4, i12 = 0, r7) {
        let n4 = this.lines.get(t3);
        return n4 ? n4.translateToString(e4, i12, r7) : "";
      }
      getWrappedRangeForLine(t3) {
        let e4 = t3, i12 = t3;
        for (; e4 > 0 && this.lines.get(e4).isWrapped; ) e4--;
        for (; i12 + 1 < this.lines.length && this.lines.get(i12 + 1).isWrapped; ) i12++;
        return { first: e4, last: i12 };
      }
      setupTabStops(t3) {
        for (t3 != null ? this.tabs[t3] || (t3 = this.prevStop(t3)) : (this.tabs = {}, t3 = 0); t3 < this._cols; t3 += this._optionsService.rawOptions.tabStopWidth) this.tabs[t3] = true;
      }
      prevStop(t3) {
        for (t3 == null && (t3 = this.x); !this.tabs[--t3] && t3 > 0; ) ;
        return t3 >= this._cols ? this._cols - 1 : t3 < 0 ? 0 : t3;
      }
      nextStop(t3) {
        for (t3 == null && (t3 = this.x); !this.tabs[++t3] && t3 < this._cols; ) ;
        return t3 >= this._cols ? this._cols - 1 : t3 < 0 ? 0 : t3;
      }
      clearMarkers(t3) {
        this._isClearing = true;
        for (let e4 = 0; e4 < this.markers.length; e4++) this.markers[e4].line === t3 && (this.markers[e4].dispose(), this.markers.splice(e4--, 1));
        this._isClearing = false;
      }
      clearAllMarkers() {
        this._isClearing = true;
        for (let t3 = 0; t3 < this.markers.length; t3++) this.markers[t3].dispose();
        this.markers.length = 0, this._isClearing = false;
      }
      addMarker(t3) {
        let e4 = new cn(t3);
        return this.markers.push(e4), e4.register(this.lines.onTrim((i12) => {
          e4.line -= i12, e4.line < 0 && e4.dispose();
        })), e4.register(this.lines.onInsert((i12) => {
          e4.line >= i12.index && (e4.line += i12.amount);
        })), e4.register(this.lines.onDelete((i12) => {
          e4.line >= i12.index && e4.line < i12.index + i12.amount && e4.dispose(), e4.line > i12.index && (e4.line -= i12.amount);
        })), e4.register(e4.onDispose(() => this._removeMarker(e4))), e4;
      }
      _removeMarker(t3) {
        this._isClearing || this.markers.splice(this.markers.indexOf(t3), 1);
      }
    };
    hn = class extends D2 {
      constructor(e4, i12) {
        super();
        this._optionsService = e4;
        this._bufferService = i12;
        this._onBufferActivate = this._register(new v2());
        this.onBufferActivate = this._onBufferActivate.event;
        this.reset(), this._register(this._optionsService.onSpecificOptionChange("scrollback", () => this.resize(this._bufferService.cols, this._bufferService.rows))), this._register(this._optionsService.onSpecificOptionChange("tabStopWidth", () => this.setupTabStops()));
      }
      reset() {
        this._normal = new $i(true, this._optionsService, this._bufferService), this._normal.fillViewportRows(), this._alt = new $i(false, this._optionsService, this._bufferService), this._activeBuffer = this._normal, this._onBufferActivate.fire({ activeBuffer: this._normal, inactiveBuffer: this._alt }), this.setupTabStops();
      }
      get alt() {
        return this._alt;
      }
      get active() {
        return this._activeBuffer;
      }
      get normal() {
        return this._normal;
      }
      activateNormalBuffer() {
        this._activeBuffer !== this._normal && (this._normal.x = this._alt.x, this._normal.y = this._alt.y, this._alt.clearAllMarkers(), this._alt.clear(), this._activeBuffer = this._normal, this._onBufferActivate.fire({ activeBuffer: this._normal, inactiveBuffer: this._alt }));
      }
      activateAltBuffer(e4) {
        this._activeBuffer !== this._alt && (this._alt.fillViewportRows(e4), this._alt.x = this._normal.x, this._alt.y = this._normal.y, this._activeBuffer = this._alt, this._onBufferActivate.fire({ activeBuffer: this._alt, inactiveBuffer: this._normal }));
      }
      resize(e4, i12) {
        this._normal.resize(e4, i12), this._alt.resize(e4, i12), this.setupTabStops(e4);
      }
      setupTabStops(e4) {
        this._normal.setupTabStops(e4), this._alt.setupTabStops(e4);
      }
    };
    ks = 2;
    Cs = 1;
    ni = class extends D2 {
      constructor(e4) {
        super();
        this.isUserScrolling = false;
        this._onResize = this._register(new v2());
        this.onResize = this._onResize.event;
        this._onScroll = this._register(new v2());
        this.onScroll = this._onScroll.event;
        this.cols = Math.max(e4.rawOptions.cols || 0, ks), this.rows = Math.max(e4.rawOptions.rows || 0, Cs), this.buffers = this._register(new hn(e4, this)), this._register(this.buffers.onBufferActivate((i12) => {
          this._onScroll.fire(i12.activeBuffer.ydisp);
        }));
      }
      get buffer() {
        return this.buffers.active;
      }
      resize(e4, i12) {
        let r7 = this.cols !== e4, n4 = this.rows !== i12;
        this.cols = e4, this.rows = i12, this.buffers.resize(e4, i12), this._onResize.fire({ cols: e4, rows: i12, colsChanged: r7, rowsChanged: n4 });
      }
      reset() {
        this.buffers.reset(), this.isUserScrolling = false;
      }
      scroll(e4, i12 = false) {
        let r7 = this.buffer, n4;
        n4 = this._cachedBlankLine, (!n4 || n4.length !== this.cols || n4.getFg(0) !== e4.fg || n4.getBg(0) !== e4.bg) && (n4 = r7.getBlankLine(e4, i12), this._cachedBlankLine = n4), n4.isWrapped = i12;
        let o6 = r7.ybase + r7.scrollTop, l4 = r7.ybase + r7.scrollBottom;
        if (r7.scrollTop === 0) {
          let a3 = r7.lines.isFull;
          l4 === r7.lines.length - 1 ? a3 ? r7.lines.recycle().copyFrom(n4) : r7.lines.push(n4.clone()) : r7.lines.splice(l4 + 1, 0, n4.clone()), a3 ? this.isUserScrolling && (r7.ydisp = Math.max(r7.ydisp - 1, 0)) : (r7.ybase++, this.isUserScrolling || r7.ydisp++);
        } else {
          let a3 = l4 - o6 + 1;
          r7.lines.shiftElements(o6 + 1, a3 - 1, -1), r7.lines.set(l4, n4.clone());
        }
        this.isUserScrolling || (r7.ydisp = r7.ybase), this._onScroll.fire(r7.ydisp);
      }
      scrollLines(e4, i12) {
        let r7 = this.buffer;
        if (e4 < 0) {
          if (r7.ydisp === 0) return;
          this.isUserScrolling = true;
        } else e4 + r7.ydisp >= r7.ybase && (this.isUserScrolling = false);
        let n4 = r7.ydisp;
        r7.ydisp = Math.max(Math.min(r7.ydisp + e4, r7.ybase), 0), n4 !== r7.ydisp && (i12 || this._onScroll.fire(r7.ydisp));
      }
    };
    ni = M2([S3(0, H2)], ni);
    si = { cols: 80, rows: 24, cursorBlink: false, cursorStyle: "block", cursorWidth: 1, cursorInactiveStyle: "outline", customGlyphs: true, drawBoldTextInBrightColors: true, documentOverride: null, fastScrollModifier: "alt", fastScrollSensitivity: 5, fontFamily: "monospace", fontSize: 15, fontWeight: "normal", fontWeightBold: "bold", ignoreBracketedPasteMode: false, lineHeight: 1, letterSpacing: 0, linkHandler: null, logLevel: "info", logger: null, scrollback: 1e3, scrollOnEraseInDisplay: false, scrollOnUserInput: true, scrollSensitivity: 1, screenReaderMode: false, smoothScrollDuration: 0, macOptionIsMeta: false, macOptionClickForcesSelection: false, minimumContrastRatio: 1, disableStdin: false, allowProposedApi: false, allowTransparency: false, tabStopWidth: 8, theme: {}, reflowCursorLine: false, rescaleOverlappingGlyphs: false, rightClickSelectsWord: Zt, windowOptions: {}, windowsMode: false, windowsPty: {}, wordSeparator: " ()[]{}',\"`", altClickMovesCursor: true, convertEol: false, termName: "xterm", cancelEvents: false, overviewRuler: {} };
    nc = ["normal", "bold", "100", "200", "300", "400", "500", "600", "700", "800", "900"];
    dn = class extends D2 {
      constructor(e4) {
        super();
        this._onOptionChange = this._register(new v2());
        this.onOptionChange = this._onOptionChange.event;
        let i12 = { ...si };
        for (let r7 in e4) if (r7 in i12) try {
          let n4 = e4[r7];
          i12[r7] = this._sanitizeAndValidateOption(r7, n4);
        } catch (n4) {
          console.error(n4);
        }
        this.rawOptions = i12, this.options = { ...i12 }, this._setupOptions(), this._register(C2(() => {
          this.rawOptions.linkHandler = null, this.rawOptions.documentOverride = null;
        }));
      }
      onSpecificOptionChange(e4, i12) {
        return this.onOptionChange((r7) => {
          r7 === e4 && i12(this.rawOptions[e4]);
        });
      }
      onMultipleOptionChange(e4, i12) {
        return this.onOptionChange((r7) => {
          e4.indexOf(r7) !== -1 && i12();
        });
      }
      _setupOptions() {
        let e4 = (r7) => {
          if (!(r7 in si)) throw new Error(`No option with key "${r7}"`);
          return this.rawOptions[r7];
        }, i12 = (r7, n4) => {
          if (!(r7 in si)) throw new Error(`No option with key "${r7}"`);
          n4 = this._sanitizeAndValidateOption(r7, n4), this.rawOptions[r7] !== n4 && (this.rawOptions[r7] = n4, this._onOptionChange.fire(r7));
        };
        for (let r7 in this.rawOptions) {
          let n4 = { get: e4.bind(this, r7), set: i12.bind(this, r7) };
          Object.defineProperty(this.options, r7, n4);
        }
      }
      _sanitizeAndValidateOption(e4, i12) {
        switch (e4) {
          case "cursorStyle":
            if (i12 || (i12 = si[e4]), !sc(i12)) throw new Error(`"${i12}" is not a valid value for ${e4}`);
            break;
          case "wordSeparator":
            i12 || (i12 = si[e4]);
            break;
          case "fontWeight":
          case "fontWeightBold":
            if (typeof i12 == "number" && 1 <= i12 && i12 <= 1e3) break;
            i12 = nc.includes(i12) ? i12 : si[e4];
            break;
          case "cursorWidth":
            i12 = Math.floor(i12);
          case "lineHeight":
          case "tabStopWidth":
            if (i12 < 1) throw new Error(`${e4} cannot be less than 1, value: ${i12}`);
            break;
          case "minimumContrastRatio":
            i12 = Math.max(1, Math.min(21, Math.round(i12 * 10) / 10));
            break;
          case "scrollback":
            if (i12 = Math.min(i12, 4294967295), i12 < 0) throw new Error(`${e4} cannot be less than 0, value: ${i12}`);
            break;
          case "fastScrollSensitivity":
          case "scrollSensitivity":
            if (i12 <= 0) throw new Error(`${e4} cannot be less than or equal to 0, value: ${i12}`);
            break;
          case "rows":
          case "cols":
            if (!i12 && i12 !== 0) throw new Error(`${e4} must be numeric, value: ${i12}`);
            break;
          case "windowsPty":
            i12 = i12 ?? {};
            break;
        }
        return i12;
      }
    };
    ul = Object.freeze({ insertMode: false });
    hl = Object.freeze({ applicationCursorKeys: false, applicationKeypad: false, bracketedPasteMode: false, cursorBlink: void 0, cursorStyle: void 0, origin: false, reverseWraparound: false, sendFocus: false, synchronizedOutput: false, wraparound: true });
    li = class extends D2 {
      constructor(e4, i12, r7) {
        super();
        this._bufferService = e4;
        this._logService = i12;
        this._optionsService = r7;
        this.isCursorInitialized = false;
        this.isCursorHidden = false;
        this._onData = this._register(new v2());
        this.onData = this._onData.event;
        this._onUserInput = this._register(new v2());
        this.onUserInput = this._onUserInput.event;
        this._onBinary = this._register(new v2());
        this.onBinary = this._onBinary.event;
        this._onRequestScrollToBottom = this._register(new v2());
        this.onRequestScrollToBottom = this._onRequestScrollToBottom.event;
        this.modes = oi(ul), this.decPrivateModes = oi(hl);
      }
      reset() {
        this.modes = oi(ul), this.decPrivateModes = oi(hl);
      }
      triggerDataEvent(e4, i12 = false) {
        if (this._optionsService.rawOptions.disableStdin) return;
        let r7 = this._bufferService.buffer;
        i12 && this._optionsService.rawOptions.scrollOnUserInput && r7.ybase !== r7.ydisp && this._onRequestScrollToBottom.fire(), i12 && this._onUserInput.fire(), this._logService.debug(`sending data "${e4}"`), this._logService.trace("sending data (codes)", () => e4.split("").map((n4) => n4.charCodeAt(0))), this._onData.fire(e4);
      }
      triggerBinaryEvent(e4) {
        this._optionsService.rawOptions.disableStdin || (this._logService.debug(`sending binary "${e4}"`), this._logService.trace("sending binary (codes)", () => e4.split("").map((i12) => i12.charCodeAt(0))), this._onBinary.fire(e4));
      }
    };
    li = M2([S3(0, F), S3(1, nr), S3(2, H2)], li);
    dl = { NONE: { events: 0, restrict: () => false }, X10: { events: 1, restrict: (s18) => s18.button === 4 || s18.action !== 1 ? false : (s18.ctrl = false, s18.alt = false, s18.shift = false, true) }, VT200: { events: 19, restrict: (s18) => s18.action !== 32 }, DRAG: { events: 23, restrict: (s18) => !(s18.action === 32 && s18.button === 3) }, ANY: { events: 31, restrict: (s18) => true } };
    Ps = String.fromCharCode;
    fl = { DEFAULT: (s18) => {
      let t3 = [Ms(s18, false) + 32, s18.col + 32, s18.row + 32];
      return t3[0] > 255 || t3[1] > 255 || t3[2] > 255 ? "" : `\x1B[M${Ps(t3[0])}${Ps(t3[1])}${Ps(t3[2])}`;
    }, SGR: (s18) => {
      let t3 = s18.action === 0 && s18.button !== 4 ? "m" : "M";
      return `\x1B[<${Ms(s18, true)};${s18.col};${s18.row}${t3}`;
    }, SGR_PIXELS: (s18) => {
      let t3 = s18.action === 0 && s18.button !== 4 ? "m" : "M";
      return `\x1B[<${Ms(s18, true)};${s18.x};${s18.y}${t3}`;
    } };
    ai = class extends D2 {
      constructor(e4, i12, r7) {
        super();
        this._bufferService = e4;
        this._coreService = i12;
        this._optionsService = r7;
        this._protocols = {};
        this._encodings = {};
        this._activeProtocol = "";
        this._activeEncoding = "";
        this._lastEvent = null;
        this._wheelPartialScroll = 0;
        this._onProtocolChange = this._register(new v2());
        this.onProtocolChange = this._onProtocolChange.event;
        for (let n4 of Object.keys(dl)) this.addProtocol(n4, dl[n4]);
        for (let n4 of Object.keys(fl)) this.addEncoding(n4, fl[n4]);
        this.reset();
      }
      addProtocol(e4, i12) {
        this._protocols[e4] = i12;
      }
      addEncoding(e4, i12) {
        this._encodings[e4] = i12;
      }
      get activeProtocol() {
        return this._activeProtocol;
      }
      get areMouseEventsActive() {
        return this._protocols[this._activeProtocol].events !== 0;
      }
      set activeProtocol(e4) {
        if (!this._protocols[e4]) throw new Error(`unknown protocol "${e4}"`);
        this._activeProtocol = e4, this._onProtocolChange.fire(this._protocols[e4].events);
      }
      get activeEncoding() {
        return this._activeEncoding;
      }
      set activeEncoding(e4) {
        if (!this._encodings[e4]) throw new Error(`unknown encoding "${e4}"`);
        this._activeEncoding = e4;
      }
      reset() {
        this.activeProtocol = "NONE", this.activeEncoding = "DEFAULT", this._lastEvent = null, this._wheelPartialScroll = 0;
      }
      consumeWheelEvent(e4, i12, r7) {
        if (e4.deltaY === 0 || e4.shiftKey || i12 === void 0 || r7 === void 0) return 0;
        let n4 = i12 / r7, o6 = this._applyScrollModifier(e4.deltaY, e4);
        return e4.deltaMode === WheelEvent.DOM_DELTA_PIXEL ? (o6 /= n4 + 0, Math.abs(e4.deltaY) < 50 && (o6 *= 0.3), this._wheelPartialScroll += o6, o6 = Math.floor(Math.abs(this._wheelPartialScroll)) * (this._wheelPartialScroll > 0 ? 1 : -1), this._wheelPartialScroll %= 1) : e4.deltaMode === WheelEvent.DOM_DELTA_PAGE && (o6 *= this._bufferService.rows), o6;
      }
      _applyScrollModifier(e4, i12) {
        return i12.altKey || i12.ctrlKey || i12.shiftKey ? e4 * this._optionsService.rawOptions.fastScrollSensitivity * this._optionsService.rawOptions.scrollSensitivity : e4 * this._optionsService.rawOptions.scrollSensitivity;
      }
      triggerMouseEvent(e4) {
        if (e4.col < 0 || e4.col >= this._bufferService.cols || e4.row < 0 || e4.row >= this._bufferService.rows || e4.button === 4 && e4.action === 32 || e4.button === 3 && e4.action !== 32 || e4.button !== 4 && (e4.action === 2 || e4.action === 3) || (e4.col++, e4.row++, e4.action === 32 && this._lastEvent && this._equalEvents(this._lastEvent, e4, this._activeEncoding === "SGR_PIXELS")) || !this._protocols[this._activeProtocol].restrict(e4)) return false;
        let i12 = this._encodings[this._activeEncoding](e4);
        return i12 && (this._activeEncoding === "DEFAULT" ? this._coreService.triggerBinaryEvent(i12) : this._coreService.triggerDataEvent(i12, true)), this._lastEvent = e4, true;
      }
      explainEvents(e4) {
        return { down: !!(e4 & 1), up: !!(e4 & 2), drag: !!(e4 & 4), move: !!(e4 & 8), wheel: !!(e4 & 16) };
      }
      _equalEvents(e4, i12, r7) {
        if (r7) {
          if (e4.x !== i12.x || e4.y !== i12.y) return false;
        } else if (e4.col !== i12.col || e4.row !== i12.row) return false;
        return !(e4.button !== i12.button || e4.action !== i12.action || e4.ctrl !== i12.ctrl || e4.alt !== i12.alt || e4.shift !== i12.shift);
      }
    };
    ai = M2([S3(0, F), S3(1, ge), S3(2, H2)], ai);
    Os = [[768, 879], [1155, 1158], [1160, 1161], [1425, 1469], [1471, 1471], [1473, 1474], [1476, 1477], [1479, 1479], [1536, 1539], [1552, 1557], [1611, 1630], [1648, 1648], [1750, 1764], [1767, 1768], [1770, 1773], [1807, 1807], [1809, 1809], [1840, 1866], [1958, 1968], [2027, 2035], [2305, 2306], [2364, 2364], [2369, 2376], [2381, 2381], [2385, 2388], [2402, 2403], [2433, 2433], [2492, 2492], [2497, 2500], [2509, 2509], [2530, 2531], [2561, 2562], [2620, 2620], [2625, 2626], [2631, 2632], [2635, 2637], [2672, 2673], [2689, 2690], [2748, 2748], [2753, 2757], [2759, 2760], [2765, 2765], [2786, 2787], [2817, 2817], [2876, 2876], [2879, 2879], [2881, 2883], [2893, 2893], [2902, 2902], [2946, 2946], [3008, 3008], [3021, 3021], [3134, 3136], [3142, 3144], [3146, 3149], [3157, 3158], [3260, 3260], [3263, 3263], [3270, 3270], [3276, 3277], [3298, 3299], [3393, 3395], [3405, 3405], [3530, 3530], [3538, 3540], [3542, 3542], [3633, 3633], [3636, 3642], [3655, 3662], [3761, 3761], [3764, 3769], [3771, 3772], [3784, 3789], [3864, 3865], [3893, 3893], [3895, 3895], [3897, 3897], [3953, 3966], [3968, 3972], [3974, 3975], [3984, 3991], [3993, 4028], [4038, 4038], [4141, 4144], [4146, 4146], [4150, 4151], [4153, 4153], [4184, 4185], [4448, 4607], [4959, 4959], [5906, 5908], [5938, 5940], [5970, 5971], [6002, 6003], [6068, 6069], [6071, 6077], [6086, 6086], [6089, 6099], [6109, 6109], [6155, 6157], [6313, 6313], [6432, 6434], [6439, 6440], [6450, 6450], [6457, 6459], [6679, 6680], [6912, 6915], [6964, 6964], [6966, 6970], [6972, 6972], [6978, 6978], [7019, 7027], [7616, 7626], [7678, 7679], [8203, 8207], [8234, 8238], [8288, 8291], [8298, 8303], [8400, 8431], [12330, 12335], [12441, 12442], [43014, 43014], [43019, 43019], [43045, 43046], [64286, 64286], [65024, 65039], [65056, 65059], [65279, 65279], [65529, 65531]];
    ac = [[68097, 68099], [68101, 68102], [68108, 68111], [68152, 68154], [68159, 68159], [119143, 119145], [119155, 119170], [119173, 119179], [119210, 119213], [119362, 119364], [917505, 917505], [917536, 917631], [917760, 917999]];
    fn = class {
      constructor() {
        this.version = "6";
        if (!se) {
          se = new Uint8Array(65536), se.fill(1), se[0] = 0, se.fill(0, 1, 32), se.fill(0, 127, 160), se.fill(2, 4352, 4448), se[9001] = 2, se[9002] = 2, se.fill(2, 11904, 42192), se[12351] = 1, se.fill(2, 44032, 55204), se.fill(2, 63744, 64256), se.fill(2, 65040, 65050), se.fill(2, 65072, 65136), se.fill(2, 65280, 65377), se.fill(2, 65504, 65511);
          for (let t3 = 0; t3 < Os.length; ++t3) se.fill(0, Os[t3][0], Os[t3][1] + 1);
        }
      }
      wcwidth(t3) {
        return t3 < 32 ? 0 : t3 < 127 ? 1 : t3 < 65536 ? se[t3] : cc(t3, ac) ? 0 : t3 >= 131072 && t3 <= 196605 || t3 >= 196608 && t3 <= 262141 ? 2 : 1;
      }
      charProperties(t3, e4) {
        let i12 = this.wcwidth(t3), r7 = i12 === 0 && e4 !== 0;
        if (r7) {
          let n4 = Ae.extractWidth(e4);
          n4 === 0 ? r7 = false : n4 > i12 && (i12 = n4);
        }
        return Ae.createPropertyValue(0, i12, r7);
      }
    };
    Ae = class s16 {
      constructor() {
        this._providers = /* @__PURE__ */ Object.create(null);
        this._active = "";
        this._onChange = new v2();
        this.onChange = this._onChange.event;
        let t3 = new fn();
        this.register(t3), this._active = t3.version, this._activeProvider = t3;
      }
      static extractShouldJoin(t3) {
        return (t3 & 1) !== 0;
      }
      static extractWidth(t3) {
        return t3 >> 1 & 3;
      }
      static extractCharKind(t3) {
        return t3 >> 3;
      }
      static createPropertyValue(t3, e4, i12 = false) {
        return (t3 & 16777215) << 3 | (e4 & 3) << 1 | (i12 ? 1 : 0);
      }
      dispose() {
        this._onChange.dispose();
      }
      get versions() {
        return Object.keys(this._providers);
      }
      get activeVersion() {
        return this._active;
      }
      set activeVersion(t3) {
        if (!this._providers[t3]) throw new Error(`unknown Unicode version "${t3}"`);
        this._active = t3, this._activeProvider = this._providers[t3], this._onChange.fire(t3);
      }
      register(t3) {
        this._providers[t3.version] = t3;
      }
      wcwidth(t3) {
        return this._activeProvider.wcwidth(t3);
      }
      getStringCellWidth(t3) {
        let e4 = 0, i12 = 0, r7 = t3.length;
        for (let n4 = 0; n4 < r7; ++n4) {
          let o6 = t3.charCodeAt(n4);
          if (55296 <= o6 && o6 <= 56319) {
            if (++n4 >= r7) return e4 + this.wcwidth(o6);
            let u3 = t3.charCodeAt(n4);
            56320 <= u3 && u3 <= 57343 ? o6 = (o6 - 55296) * 1024 + u3 - 56320 + 65536 : e4 += this.wcwidth(u3);
          }
          let l4 = this.charProperties(o6, i12), a3 = s16.extractWidth(l4);
          s16.extractShouldJoin(l4) && (a3 -= s16.extractWidth(i12)), e4 += a3, i12 = l4;
        }
        return e4;
      }
      charProperties(t3, e4) {
        return this._activeProvider.charProperties(t3, e4);
      }
    };
    pn = class {
      constructor() {
        this.glevel = 0;
        this._charsets = [];
      }
      reset() {
        this.charset = void 0, this._charsets = [], this.glevel = 0;
      }
      setgLevel(t3) {
        this.glevel = t3, this.charset = this._charsets[t3];
      }
      setgCharset(t3, e4) {
        this._charsets[t3] = e4, this.glevel === t3 && (this.charset = e4);
      }
    };
    Vi = 2147483647;
    uc = 256;
    ci = class s17 {
      constructor(t3 = 32, e4 = 32) {
        this.maxLength = t3;
        this.maxSubParamsLength = e4;
        if (e4 > uc) throw new Error("maxSubParamsLength must not be greater than 256");
        this.params = new Int32Array(t3), this.length = 0, this._subParams = new Int32Array(e4), this._subParamsLength = 0, this._subParamsIdx = new Uint16Array(t3), this._rejectDigits = false, this._rejectSubDigits = false, this._digitIsSub = false;
      }
      static fromArray(t3) {
        let e4 = new s17();
        if (!t3.length) return e4;
        for (let i12 = Array.isArray(t3[0]) ? 1 : 0; i12 < t3.length; ++i12) {
          let r7 = t3[i12];
          if (Array.isArray(r7)) for (let n4 = 0; n4 < r7.length; ++n4) e4.addSubParam(r7[n4]);
          else e4.addParam(r7);
        }
        return e4;
      }
      clone() {
        let t3 = new s17(this.maxLength, this.maxSubParamsLength);
        return t3.params.set(this.params), t3.length = this.length, t3._subParams.set(this._subParams), t3._subParamsLength = this._subParamsLength, t3._subParamsIdx.set(this._subParamsIdx), t3._rejectDigits = this._rejectDigits, t3._rejectSubDigits = this._rejectSubDigits, t3._digitIsSub = this._digitIsSub, t3;
      }
      toArray() {
        let t3 = [];
        for (let e4 = 0; e4 < this.length; ++e4) {
          t3.push(this.params[e4]);
          let i12 = this._subParamsIdx[e4] >> 8, r7 = this._subParamsIdx[e4] & 255;
          r7 - i12 > 0 && t3.push(Array.prototype.slice.call(this._subParams, i12, r7));
        }
        return t3;
      }
      reset() {
        this.length = 0, this._subParamsLength = 0, this._rejectDigits = false, this._rejectSubDigits = false, this._digitIsSub = false;
      }
      addParam(t3) {
        if (this._digitIsSub = false, this.length >= this.maxLength) {
          this._rejectDigits = true;
          return;
        }
        if (t3 < -1) throw new Error("values lesser than -1 are not allowed");
        this._subParamsIdx[this.length] = this._subParamsLength << 8 | this._subParamsLength, this.params[this.length++] = t3 > Vi ? Vi : t3;
      }
      addSubParam(t3) {
        if (this._digitIsSub = true, !!this.length) {
          if (this._rejectDigits || this._subParamsLength >= this.maxSubParamsLength) {
            this._rejectSubDigits = true;
            return;
          }
          if (t3 < -1) throw new Error("values lesser than -1 are not allowed");
          this._subParams[this._subParamsLength++] = t3 > Vi ? Vi : t3, this._subParamsIdx[this.length - 1]++;
        }
      }
      hasSubParams(t3) {
        return (this._subParamsIdx[t3] & 255) - (this._subParamsIdx[t3] >> 8) > 0;
      }
      getSubParams(t3) {
        let e4 = this._subParamsIdx[t3] >> 8, i12 = this._subParamsIdx[t3] & 255;
        return i12 - e4 > 0 ? this._subParams.subarray(e4, i12) : null;
      }
      getSubParamsAll() {
        let t3 = {};
        for (let e4 = 0; e4 < this.length; ++e4) {
          let i12 = this._subParamsIdx[e4] >> 8, r7 = this._subParamsIdx[e4] & 255;
          r7 - i12 > 0 && (t3[e4] = this._subParams.slice(i12, r7));
        }
        return t3;
      }
      addDigit(t3) {
        let e4;
        if (this._rejectDigits || !(e4 = this._digitIsSub ? this._subParamsLength : this.length) || this._digitIsSub && this._rejectSubDigits) return;
        let i12 = this._digitIsSub ? this._subParams : this.params, r7 = i12[e4 - 1];
        i12[e4 - 1] = ~r7 ? Math.min(r7 * 10 + t3, Vi) : t3;
      }
    };
    qi = [];
    mn = class {
      constructor() {
        this._state = 0;
        this._active = qi;
        this._id = -1;
        this._handlers = /* @__PURE__ */ Object.create(null);
        this._handlerFb = () => {
        };
        this._stack = { paused: false, loopPosition: 0, fallThrough: false };
      }
      registerHandler(t3, e4) {
        this._handlers[t3] === void 0 && (this._handlers[t3] = []);
        let i12 = this._handlers[t3];
        return i12.push(e4), { dispose: () => {
          let r7 = i12.indexOf(e4);
          r7 !== -1 && i12.splice(r7, 1);
        } };
      }
      clearHandler(t3) {
        this._handlers[t3] && delete this._handlers[t3];
      }
      setHandlerFallback(t3) {
        this._handlerFb = t3;
      }
      dispose() {
        this._handlers = /* @__PURE__ */ Object.create(null), this._handlerFb = () => {
        }, this._active = qi;
      }
      reset() {
        if (this._state === 2) for (let t3 = this._stack.paused ? this._stack.loopPosition - 1 : this._active.length - 1; t3 >= 0; --t3) this._active[t3].end(false);
        this._stack.paused = false, this._active = qi, this._id = -1, this._state = 0;
      }
      _start() {
        if (this._active = this._handlers[this._id] || qi, !this._active.length) this._handlerFb(this._id, "START");
        else for (let t3 = this._active.length - 1; t3 >= 0; t3--) this._active[t3].start();
      }
      _put(t3, e4, i12) {
        if (!this._active.length) this._handlerFb(this._id, "PUT", It(t3, e4, i12));
        else for (let r7 = this._active.length - 1; r7 >= 0; r7--) this._active[r7].put(t3, e4, i12);
      }
      start() {
        this.reset(), this._state = 1;
      }
      put(t3, e4, i12) {
        if (this._state !== 3) {
          if (this._state === 1) for (; e4 < i12; ) {
            let r7 = t3[e4++];
            if (r7 === 59) {
              this._state = 2, this._start();
              break;
            }
            if (r7 < 48 || 57 < r7) {
              this._state = 3;
              return;
            }
            this._id === -1 && (this._id = 0), this._id = this._id * 10 + r7 - 48;
          }
          this._state === 2 && i12 - e4 > 0 && this._put(t3, e4, i12);
        }
      }
      end(t3, e4 = true) {
        if (this._state !== 0) {
          if (this._state !== 3) if (this._state === 1 && this._start(), !this._active.length) this._handlerFb(this._id, "END", t3);
          else {
            let i12 = false, r7 = this._active.length - 1, n4 = false;
            if (this._stack.paused && (r7 = this._stack.loopPosition - 1, i12 = e4, n4 = this._stack.fallThrough, this._stack.paused = false), !n4 && i12 === false) {
              for (; r7 >= 0 && (i12 = this._active[r7].end(t3), i12 !== true); r7--) if (i12 instanceof Promise) return this._stack.paused = true, this._stack.loopPosition = r7, this._stack.fallThrough = false, i12;
              r7--;
            }
            for (; r7 >= 0; r7--) if (i12 = this._active[r7].end(false), i12 instanceof Promise) return this._stack.paused = true, this._stack.loopPosition = r7, this._stack.fallThrough = true, i12;
          }
          this._active = qi, this._id = -1, this._state = 0;
        }
      }
    };
    pe = class {
      constructor(t3) {
        this._handler = t3;
        this._data = "";
        this._hitLimit = false;
      }
      start() {
        this._data = "", this._hitLimit = false;
      }
      put(t3, e4, i12) {
        this._hitLimit || (this._data += It(t3, e4, i12), this._data.length > 1e7 && (this._data = "", this._hitLimit = true));
      }
      end(t3) {
        let e4 = false;
        if (this._hitLimit) e4 = false;
        else if (t3 && (e4 = this._handler(this._data), e4 instanceof Promise)) return e4.then((i12) => (this._data = "", this._hitLimit = false, i12));
        return this._data = "", this._hitLimit = false, e4;
      }
    };
    Yi = [];
    _n = class {
      constructor() {
        this._handlers = /* @__PURE__ */ Object.create(null);
        this._active = Yi;
        this._ident = 0;
        this._handlerFb = () => {
        };
        this._stack = { paused: false, loopPosition: 0, fallThrough: false };
      }
      dispose() {
        this._handlers = /* @__PURE__ */ Object.create(null), this._handlerFb = () => {
        }, this._active = Yi;
      }
      registerHandler(t3, e4) {
        this._handlers[t3] === void 0 && (this._handlers[t3] = []);
        let i12 = this._handlers[t3];
        return i12.push(e4), { dispose: () => {
          let r7 = i12.indexOf(e4);
          r7 !== -1 && i12.splice(r7, 1);
        } };
      }
      clearHandler(t3) {
        this._handlers[t3] && delete this._handlers[t3];
      }
      setHandlerFallback(t3) {
        this._handlerFb = t3;
      }
      reset() {
        if (this._active.length) for (let t3 = this._stack.paused ? this._stack.loopPosition - 1 : this._active.length - 1; t3 >= 0; --t3) this._active[t3].unhook(false);
        this._stack.paused = false, this._active = Yi, this._ident = 0;
      }
      hook(t3, e4) {
        if (this.reset(), this._ident = t3, this._active = this._handlers[t3] || Yi, !this._active.length) this._handlerFb(this._ident, "HOOK", e4);
        else for (let i12 = this._active.length - 1; i12 >= 0; i12--) this._active[i12].hook(e4);
      }
      put(t3, e4, i12) {
        if (!this._active.length) this._handlerFb(this._ident, "PUT", It(t3, e4, i12));
        else for (let r7 = this._active.length - 1; r7 >= 0; r7--) this._active[r7].put(t3, e4, i12);
      }
      unhook(t3, e4 = true) {
        if (!this._active.length) this._handlerFb(this._ident, "UNHOOK", t3);
        else {
          let i12 = false, r7 = this._active.length - 1, n4 = false;
          if (this._stack.paused && (r7 = this._stack.loopPosition - 1, i12 = e4, n4 = this._stack.fallThrough, this._stack.paused = false), !n4 && i12 === false) {
            for (; r7 >= 0 && (i12 = this._active[r7].unhook(t3), i12 !== true); r7--) if (i12 instanceof Promise) return this._stack.paused = true, this._stack.loopPosition = r7, this._stack.fallThrough = false, i12;
            r7--;
          }
          for (; r7 >= 0; r7--) if (i12 = this._active[r7].unhook(false), i12 instanceof Promise) return this._stack.paused = true, this._stack.loopPosition = r7, this._stack.fallThrough = true, i12;
        }
        this._active = Yi, this._ident = 0;
      }
    };
    ji = new ci();
    ji.addParam(0);
    Xi = class {
      constructor(t3) {
        this._handler = t3;
        this._data = "";
        this._params = ji;
        this._hitLimit = false;
      }
      hook(t3) {
        this._params = t3.length > 1 || t3.params[0] ? t3.clone() : ji, this._data = "", this._hitLimit = false;
      }
      put(t3, e4, i12) {
        this._hitLimit || (this._data += It(t3, e4, i12), this._data.length > 1e7 && (this._data = "", this._hitLimit = true));
      }
      unhook(t3) {
        let e4 = false;
        if (this._hitLimit) e4 = false;
        else if (t3 && (e4 = this._handler(this._data, this._params), e4 instanceof Promise)) return e4.then((i12) => (this._params = ji, this._data = "", this._hitLimit = false, i12));
        return this._params = ji, this._data = "", this._hitLimit = false, e4;
      }
    };
    Fs = class {
      constructor(t3) {
        this.table = new Uint8Array(t3);
      }
      setDefault(t3, e4) {
        this.table.fill(t3 << 4 | e4);
      }
      add(t3, e4, i12, r7) {
        this.table[e4 << 8 | t3] = i12 << 4 | r7;
      }
      addMany(t3, e4, i12, r7) {
        for (let n4 = 0; n4 < t3.length; n4++) this.table[e4 << 8 | t3[n4]] = i12 << 4 | r7;
      }
    };
    ke = 160;
    hc = (function() {
      let s18 = new Fs(4095), e4 = Array.apply(null, Array(256)).map((a3, u3) => u3), i12 = (a3, u3) => e4.slice(a3, u3), r7 = i12(32, 127), n4 = i12(0, 24);
      n4.push(25), n4.push.apply(n4, i12(28, 32));
      let o6 = i12(0, 14), l4;
      s18.setDefault(1, 0), s18.addMany(r7, 0, 2, 0);
      for (l4 in o6) s18.addMany([24, 26, 153, 154], l4, 3, 0), s18.addMany(i12(128, 144), l4, 3, 0), s18.addMany(i12(144, 152), l4, 3, 0), s18.add(156, l4, 0, 0), s18.add(27, l4, 11, 1), s18.add(157, l4, 4, 8), s18.addMany([152, 158, 159], l4, 0, 7), s18.add(155, l4, 11, 3), s18.add(144, l4, 11, 9);
      return s18.addMany(n4, 0, 3, 0), s18.addMany(n4, 1, 3, 1), s18.add(127, 1, 0, 1), s18.addMany(n4, 8, 0, 8), s18.addMany(n4, 3, 3, 3), s18.add(127, 3, 0, 3), s18.addMany(n4, 4, 3, 4), s18.add(127, 4, 0, 4), s18.addMany(n4, 6, 3, 6), s18.addMany(n4, 5, 3, 5), s18.add(127, 5, 0, 5), s18.addMany(n4, 2, 3, 2), s18.add(127, 2, 0, 2), s18.add(93, 1, 4, 8), s18.addMany(r7, 8, 5, 8), s18.add(127, 8, 5, 8), s18.addMany([156, 27, 24, 26, 7], 8, 6, 0), s18.addMany(i12(28, 32), 8, 0, 8), s18.addMany([88, 94, 95], 1, 0, 7), s18.addMany(r7, 7, 0, 7), s18.addMany(n4, 7, 0, 7), s18.add(156, 7, 0, 0), s18.add(127, 7, 0, 7), s18.add(91, 1, 11, 3), s18.addMany(i12(64, 127), 3, 7, 0), s18.addMany(i12(48, 60), 3, 8, 4), s18.addMany([60, 61, 62, 63], 3, 9, 4), s18.addMany(i12(48, 60), 4, 8, 4), s18.addMany(i12(64, 127), 4, 7, 0), s18.addMany([60, 61, 62, 63], 4, 0, 6), s18.addMany(i12(32, 64), 6, 0, 6), s18.add(127, 6, 0, 6), s18.addMany(i12(64, 127), 6, 0, 0), s18.addMany(i12(32, 48), 3, 9, 5), s18.addMany(i12(32, 48), 5, 9, 5), s18.addMany(i12(48, 64), 5, 0, 6), s18.addMany(i12(64, 127), 5, 7, 0), s18.addMany(i12(32, 48), 4, 9, 5), s18.addMany(i12(32, 48), 1, 9, 2), s18.addMany(i12(32, 48), 2, 9, 2), s18.addMany(i12(48, 127), 2, 10, 0), s18.addMany(i12(48, 80), 1, 10, 0), s18.addMany(i12(81, 88), 1, 10, 0), s18.addMany([89, 90, 92], 1, 10, 0), s18.addMany(i12(96, 127), 1, 10, 0), s18.add(80, 1, 11, 9), s18.addMany(n4, 9, 0, 9), s18.add(127, 9, 0, 9), s18.addMany(i12(28, 32), 9, 0, 9), s18.addMany(i12(32, 48), 9, 9, 12), s18.addMany(i12(48, 60), 9, 8, 10), s18.addMany([60, 61, 62, 63], 9, 9, 10), s18.addMany(n4, 11, 0, 11), s18.addMany(i12(32, 128), 11, 0, 11), s18.addMany(i12(28, 32), 11, 0, 11), s18.addMany(n4, 10, 0, 10), s18.add(127, 10, 0, 10), s18.addMany(i12(28, 32), 10, 0, 10), s18.addMany(i12(48, 60), 10, 8, 10), s18.addMany([60, 61, 62, 63], 10, 0, 11), s18.addMany(i12(32, 48), 10, 9, 12), s18.addMany(n4, 12, 0, 12), s18.add(127, 12, 0, 12), s18.addMany(i12(28, 32), 12, 0, 12), s18.addMany(i12(32, 48), 12, 9, 12), s18.addMany(i12(48, 64), 12, 0, 11), s18.addMany(i12(64, 127), 12, 12, 13), s18.addMany(i12(64, 127), 10, 12, 13), s18.addMany(i12(64, 127), 9, 12, 13), s18.addMany(n4, 13, 13, 13), s18.addMany(r7, 13, 13, 13), s18.add(127, 13, 0, 13), s18.addMany([27, 156, 24, 26], 13, 14, 0), s18.add(ke, 0, 2, 0), s18.add(ke, 8, 5, 8), s18.add(ke, 6, 0, 6), s18.add(ke, 11, 0, 11), s18.add(ke, 13, 13, 13), s18;
    })();
    bn = class extends D2 {
      constructor(e4 = hc) {
        super();
        this._transitions = e4;
        this._parseStack = { state: 0, handlers: [], handlerPos: 0, transition: 0, chunkPos: 0 };
        this.initialState = 0, this.currentState = this.initialState, this._params = new ci(), this._params.addParam(0), this._collect = 0, this.precedingJoinState = 0, this._printHandlerFb = (i12, r7, n4) => {
        }, this._executeHandlerFb = (i12) => {
        }, this._csiHandlerFb = (i12, r7) => {
        }, this._escHandlerFb = (i12) => {
        }, this._errorHandlerFb = (i12) => i12, this._printHandler = this._printHandlerFb, this._executeHandlers = /* @__PURE__ */ Object.create(null), this._csiHandlers = /* @__PURE__ */ Object.create(null), this._escHandlers = /* @__PURE__ */ Object.create(null), this._register(C2(() => {
          this._csiHandlers = /* @__PURE__ */ Object.create(null), this._executeHandlers = /* @__PURE__ */ Object.create(null), this._escHandlers = /* @__PURE__ */ Object.create(null);
        })), this._oscParser = this._register(new mn()), this._dcsParser = this._register(new _n()), this._errorHandler = this._errorHandlerFb, this.registerEscHandler({ final: "\\" }, () => true);
      }
      _identifier(e4, i12 = [64, 126]) {
        let r7 = 0;
        if (e4.prefix) {
          if (e4.prefix.length > 1) throw new Error("only one byte as prefix supported");
          if (r7 = e4.prefix.charCodeAt(0), r7 && 60 > r7 || r7 > 63) throw new Error("prefix must be in range 0x3c .. 0x3f");
        }
        if (e4.intermediates) {
          if (e4.intermediates.length > 2) throw new Error("only two bytes as intermediates are supported");
          for (let o6 = 0; o6 < e4.intermediates.length; ++o6) {
            let l4 = e4.intermediates.charCodeAt(o6);
            if (32 > l4 || l4 > 47) throw new Error("intermediate must be in range 0x20 .. 0x2f");
            r7 <<= 8, r7 |= l4;
          }
        }
        if (e4.final.length !== 1) throw new Error("final must be a single byte");
        let n4 = e4.final.charCodeAt(0);
        if (i12[0] > n4 || n4 > i12[1]) throw new Error(`final must be in range ${i12[0]} .. ${i12[1]}`);
        return r7 <<= 8, r7 |= n4, r7;
      }
      identToString(e4) {
        let i12 = [];
        for (; e4; ) i12.push(String.fromCharCode(e4 & 255)), e4 >>= 8;
        return i12.reverse().join("");
      }
      setPrintHandler(e4) {
        this._printHandler = e4;
      }
      clearPrintHandler() {
        this._printHandler = this._printHandlerFb;
      }
      registerEscHandler(e4, i12) {
        let r7 = this._identifier(e4, [48, 126]);
        this._escHandlers[r7] === void 0 && (this._escHandlers[r7] = []);
        let n4 = this._escHandlers[r7];
        return n4.push(i12), { dispose: () => {
          let o6 = n4.indexOf(i12);
          o6 !== -1 && n4.splice(o6, 1);
        } };
      }
      clearEscHandler(e4) {
        this._escHandlers[this._identifier(e4, [48, 126])] && delete this._escHandlers[this._identifier(e4, [48, 126])];
      }
      setEscHandlerFallback(e4) {
        this._escHandlerFb = e4;
      }
      setExecuteHandler(e4, i12) {
        this._executeHandlers[e4.charCodeAt(0)] = i12;
      }
      clearExecuteHandler(e4) {
        this._executeHandlers[e4.charCodeAt(0)] && delete this._executeHandlers[e4.charCodeAt(0)];
      }
      setExecuteHandlerFallback(e4) {
        this._executeHandlerFb = e4;
      }
      registerCsiHandler(e4, i12) {
        let r7 = this._identifier(e4);
        this._csiHandlers[r7] === void 0 && (this._csiHandlers[r7] = []);
        let n4 = this._csiHandlers[r7];
        return n4.push(i12), { dispose: () => {
          let o6 = n4.indexOf(i12);
          o6 !== -1 && n4.splice(o6, 1);
        } };
      }
      clearCsiHandler(e4) {
        this._csiHandlers[this._identifier(e4)] && delete this._csiHandlers[this._identifier(e4)];
      }
      setCsiHandlerFallback(e4) {
        this._csiHandlerFb = e4;
      }
      registerDcsHandler(e4, i12) {
        return this._dcsParser.registerHandler(this._identifier(e4), i12);
      }
      clearDcsHandler(e4) {
        this._dcsParser.clearHandler(this._identifier(e4));
      }
      setDcsHandlerFallback(e4) {
        this._dcsParser.setHandlerFallback(e4);
      }
      registerOscHandler(e4, i12) {
        return this._oscParser.registerHandler(e4, i12);
      }
      clearOscHandler(e4) {
        this._oscParser.clearHandler(e4);
      }
      setOscHandlerFallback(e4) {
        this._oscParser.setHandlerFallback(e4);
      }
      setErrorHandler(e4) {
        this._errorHandler = e4;
      }
      clearErrorHandler() {
        this._errorHandler = this._errorHandlerFb;
      }
      reset() {
        this.currentState = this.initialState, this._oscParser.reset(), this._dcsParser.reset(), this._params.reset(), this._params.addParam(0), this._collect = 0, this.precedingJoinState = 0, this._parseStack.state !== 0 && (this._parseStack.state = 2, this._parseStack.handlers = []);
      }
      _preserveStack(e4, i12, r7, n4, o6) {
        this._parseStack.state = e4, this._parseStack.handlers = i12, this._parseStack.handlerPos = r7, this._parseStack.transition = n4, this._parseStack.chunkPos = o6;
      }
      parse(e4, i12, r7) {
        let n4 = 0, o6 = 0, l4 = 0, a3;
        if (this._parseStack.state) if (this._parseStack.state === 2) this._parseStack.state = 0, l4 = this._parseStack.chunkPos + 1;
        else {
          if (r7 === void 0 || this._parseStack.state === 1) throw this._parseStack.state = 1, new Error("improper continuation due to previous async handler, giving up parsing");
          let u3 = this._parseStack.handlers, h4 = this._parseStack.handlerPos - 1;
          switch (this._parseStack.state) {
            case 3:
              if (r7 === false && h4 > -1) {
                for (; h4 >= 0 && (a3 = u3[h4](this._params), a3 !== true); h4--) if (a3 instanceof Promise) return this._parseStack.handlerPos = h4, a3;
              }
              this._parseStack.handlers = [];
              break;
            case 4:
              if (r7 === false && h4 > -1) {
                for (; h4 >= 0 && (a3 = u3[h4](), a3 !== true); h4--) if (a3 instanceof Promise) return this._parseStack.handlerPos = h4, a3;
              }
              this._parseStack.handlers = [];
              break;
            case 6:
              if (n4 = e4[this._parseStack.chunkPos], a3 = this._dcsParser.unhook(n4 !== 24 && n4 !== 26, r7), a3) return a3;
              n4 === 27 && (this._parseStack.transition |= 1), this._params.reset(), this._params.addParam(0), this._collect = 0;
              break;
            case 5:
              if (n4 = e4[this._parseStack.chunkPos], a3 = this._oscParser.end(n4 !== 24 && n4 !== 26, r7), a3) return a3;
              n4 === 27 && (this._parseStack.transition |= 1), this._params.reset(), this._params.addParam(0), this._collect = 0;
              break;
          }
          this._parseStack.state = 0, l4 = this._parseStack.chunkPos + 1, this.precedingJoinState = 0, this.currentState = this._parseStack.transition & 15;
        }
        for (let u3 = l4; u3 < i12; ++u3) {
          switch (n4 = e4[u3], o6 = this._transitions.table[this.currentState << 8 | (n4 < 160 ? n4 : ke)], o6 >> 4) {
            case 2:
              for (let m2 = u3 + 1; ; ++m2) {
                if (m2 >= i12 || (n4 = e4[m2]) < 32 || n4 > 126 && n4 < ke) {
                  this._printHandler(e4, u3, m2), u3 = m2 - 1;
                  break;
                }
                if (++m2 >= i12 || (n4 = e4[m2]) < 32 || n4 > 126 && n4 < ke) {
                  this._printHandler(e4, u3, m2), u3 = m2 - 1;
                  break;
                }
                if (++m2 >= i12 || (n4 = e4[m2]) < 32 || n4 > 126 && n4 < ke) {
                  this._printHandler(e4, u3, m2), u3 = m2 - 1;
                  break;
                }
                if (++m2 >= i12 || (n4 = e4[m2]) < 32 || n4 > 126 && n4 < ke) {
                  this._printHandler(e4, u3, m2), u3 = m2 - 1;
                  break;
                }
              }
              break;
            case 3:
              this._executeHandlers[n4] ? this._executeHandlers[n4]() : this._executeHandlerFb(n4), this.precedingJoinState = 0;
              break;
            case 0:
              break;
            case 1:
              if (this._errorHandler({ position: u3, code: n4, currentState: this.currentState, collect: this._collect, params: this._params, abort: false }).abort) return;
              break;
            case 7:
              let c4 = this._csiHandlers[this._collect << 8 | n4], d3 = c4 ? c4.length - 1 : -1;
              for (; d3 >= 0 && (a3 = c4[d3](this._params), a3 !== true); d3--) if (a3 instanceof Promise) return this._preserveStack(3, c4, d3, o6, u3), a3;
              d3 < 0 && this._csiHandlerFb(this._collect << 8 | n4, this._params), this.precedingJoinState = 0;
              break;
            case 8:
              do
                switch (n4) {
                  case 59:
                    this._params.addParam(0);
                    break;
                  case 58:
                    this._params.addSubParam(-1);
                    break;
                  default:
                    this._params.addDigit(n4 - 48);
                }
              while (++u3 < i12 && (n4 = e4[u3]) > 47 && n4 < 60);
              u3--;
              break;
            case 9:
              this._collect <<= 8, this._collect |= n4;
              break;
            case 10:
              let _4 = this._escHandlers[this._collect << 8 | n4], p3 = _4 ? _4.length - 1 : -1;
              for (; p3 >= 0 && (a3 = _4[p3](), a3 !== true); p3--) if (a3 instanceof Promise) return this._preserveStack(4, _4, p3, o6, u3), a3;
              p3 < 0 && this._escHandlerFb(this._collect << 8 | n4), this.precedingJoinState = 0;
              break;
            case 11:
              this._params.reset(), this._params.addParam(0), this._collect = 0;
              break;
            case 12:
              this._dcsParser.hook(this._collect << 8 | n4, this._params);
              break;
            case 13:
              for (let m2 = u3 + 1; ; ++m2) if (m2 >= i12 || (n4 = e4[m2]) === 24 || n4 === 26 || n4 === 27 || n4 > 127 && n4 < ke) {
                this._dcsParser.put(e4, u3, m2), u3 = m2 - 1;
                break;
              }
              break;
            case 14:
              if (a3 = this._dcsParser.unhook(n4 !== 24 && n4 !== 26), a3) return this._preserveStack(6, [], 0, o6, u3), a3;
              n4 === 27 && (o6 |= 1), this._params.reset(), this._params.addParam(0), this._collect = 0, this.precedingJoinState = 0;
              break;
            case 4:
              this._oscParser.start();
              break;
            case 5:
              for (let m2 = u3 + 1; ; m2++) if (m2 >= i12 || (n4 = e4[m2]) < 32 || n4 > 127 && n4 < ke) {
                this._oscParser.put(e4, u3, m2), u3 = m2 - 1;
                break;
              }
              break;
            case 6:
              if (a3 = this._oscParser.end(n4 !== 24 && n4 !== 26), a3) return this._preserveStack(5, [], 0, o6, u3), a3;
              n4 === 27 && (o6 |= 1), this._params.reset(), this._params.addParam(0), this._collect = 0, this.precedingJoinState = 0;
              break;
          }
          this.currentState = o6 & 15;
        }
      }
    };
    dc = /^([\da-f])\/([\da-f])\/([\da-f])$|^([\da-f]{2})\/([\da-f]{2})\/([\da-f]{2})$|^([\da-f]{3})\/([\da-f]{3})\/([\da-f]{3})$|^([\da-f]{4})\/([\da-f]{4})\/([\da-f]{4})$/;
    fc = /^[\da-f]+$/;
    mc = { "(": 0, ")": 1, "*": 2, "+": 3, "-": 1, ".": 2 };
    ut = 131072;
    _l = 10;
    vl = 5e3;
    gl = 0;
    vn = class extends D2 {
      constructor(e4, i12, r7, n4, o6, l4, a3, u3, h4 = new bn()) {
        super();
        this._bufferService = e4;
        this._charsetService = i12;
        this._coreService = r7;
        this._logService = n4;
        this._optionsService = o6;
        this._oscLinkService = l4;
        this._coreMouseService = a3;
        this._unicodeService = u3;
        this._parser = h4;
        this._parseBuffer = new Uint32Array(4096);
        this._stringDecoder = new er();
        this._utf8Decoder = new tr();
        this._windowTitle = "";
        this._iconName = "";
        this._windowTitleStack = [];
        this._iconNameStack = [];
        this._curAttrData = X.clone();
        this._eraseAttrDataInternal = X.clone();
        this._onRequestBell = this._register(new v2());
        this.onRequestBell = this._onRequestBell.event;
        this._onRequestRefreshRows = this._register(new v2());
        this.onRequestRefreshRows = this._onRequestRefreshRows.event;
        this._onRequestReset = this._register(new v2());
        this.onRequestReset = this._onRequestReset.event;
        this._onRequestSendFocus = this._register(new v2());
        this.onRequestSendFocus = this._onRequestSendFocus.event;
        this._onRequestSyncScrollBar = this._register(new v2());
        this.onRequestSyncScrollBar = this._onRequestSyncScrollBar.event;
        this._onRequestWindowsOptionsReport = this._register(new v2());
        this.onRequestWindowsOptionsReport = this._onRequestWindowsOptionsReport.event;
        this._onA11yChar = this._register(new v2());
        this.onA11yChar = this._onA11yChar.event;
        this._onA11yTab = this._register(new v2());
        this.onA11yTab = this._onA11yTab.event;
        this._onCursorMove = this._register(new v2());
        this.onCursorMove = this._onCursorMove.event;
        this._onLineFeed = this._register(new v2());
        this.onLineFeed = this._onLineFeed.event;
        this._onScroll = this._register(new v2());
        this.onScroll = this._onScroll.event;
        this._onTitleChange = this._register(new v2());
        this.onTitleChange = this._onTitleChange.event;
        this._onColor = this._register(new v2());
        this.onColor = this._onColor.event;
        this._parseStack = { paused: false, cursorStartX: 0, cursorStartY: 0, decodedLength: 0, position: 0 };
        this._specialColors = [256, 257, 258];
        this._register(this._parser), this._dirtyRowTracker = new Zi(this._bufferService), this._activeBuffer = this._bufferService.buffer, this._register(this._bufferService.buffers.onBufferActivate((c4) => this._activeBuffer = c4.activeBuffer)), this._parser.setCsiHandlerFallback((c4, d3) => {
          this._logService.debug("Unknown CSI code: ", { identifier: this._parser.identToString(c4), params: d3.toArray() });
        }), this._parser.setEscHandlerFallback((c4) => {
          this._logService.debug("Unknown ESC code: ", { identifier: this._parser.identToString(c4) });
        }), this._parser.setExecuteHandlerFallback((c4) => {
          this._logService.debug("Unknown EXECUTE code: ", { code: c4 });
        }), this._parser.setOscHandlerFallback((c4, d3, _4) => {
          this._logService.debug("Unknown OSC code: ", { identifier: c4, action: d3, data: _4 });
        }), this._parser.setDcsHandlerFallback((c4, d3, _4) => {
          d3 === "HOOK" && (_4 = _4.toArray()), this._logService.debug("Unknown DCS code: ", { identifier: this._parser.identToString(c4), action: d3, payload: _4 });
        }), this._parser.setPrintHandler((c4, d3, _4) => this.print(c4, d3, _4)), this._parser.registerCsiHandler({ final: "@" }, (c4) => this.insertChars(c4)), this._parser.registerCsiHandler({ intermediates: " ", final: "@" }, (c4) => this.scrollLeft(c4)), this._parser.registerCsiHandler({ final: "A" }, (c4) => this.cursorUp(c4)), this._parser.registerCsiHandler({ intermediates: " ", final: "A" }, (c4) => this.scrollRight(c4)), this._parser.registerCsiHandler({ final: "B" }, (c4) => this.cursorDown(c4)), this._parser.registerCsiHandler({ final: "C" }, (c4) => this.cursorForward(c4)), this._parser.registerCsiHandler({ final: "D" }, (c4) => this.cursorBackward(c4)), this._parser.registerCsiHandler({ final: "E" }, (c4) => this.cursorNextLine(c4)), this._parser.registerCsiHandler({ final: "F" }, (c4) => this.cursorPrecedingLine(c4)), this._parser.registerCsiHandler({ final: "G" }, (c4) => this.cursorCharAbsolute(c4)), this._parser.registerCsiHandler({ final: "H" }, (c4) => this.cursorPosition(c4)), this._parser.registerCsiHandler({ final: "I" }, (c4) => this.cursorForwardTab(c4)), this._parser.registerCsiHandler({ final: "J" }, (c4) => this.eraseInDisplay(c4, false)), this._parser.registerCsiHandler({ prefix: "?", final: "J" }, (c4) => this.eraseInDisplay(c4, true)), this._parser.registerCsiHandler({ final: "K" }, (c4) => this.eraseInLine(c4, false)), this._parser.registerCsiHandler({ prefix: "?", final: "K" }, (c4) => this.eraseInLine(c4, true)), this._parser.registerCsiHandler({ final: "L" }, (c4) => this.insertLines(c4)), this._parser.registerCsiHandler({ final: "M" }, (c4) => this.deleteLines(c4)), this._parser.registerCsiHandler({ final: "P" }, (c4) => this.deleteChars(c4)), this._parser.registerCsiHandler({ final: "S" }, (c4) => this.scrollUp(c4)), this._parser.registerCsiHandler({ final: "T" }, (c4) => this.scrollDown(c4)), this._parser.registerCsiHandler({ final: "X" }, (c4) => this.eraseChars(c4)), this._parser.registerCsiHandler({ final: "Z" }, (c4) => this.cursorBackwardTab(c4)), this._parser.registerCsiHandler({ final: "`" }, (c4) => this.charPosAbsolute(c4)), this._parser.registerCsiHandler({ final: "a" }, (c4) => this.hPositionRelative(c4)), this._parser.registerCsiHandler({ final: "b" }, (c4) => this.repeatPrecedingCharacter(c4)), this._parser.registerCsiHandler({ final: "c" }, (c4) => this.sendDeviceAttributesPrimary(c4)), this._parser.registerCsiHandler({ prefix: ">", final: "c" }, (c4) => this.sendDeviceAttributesSecondary(c4)), this._parser.registerCsiHandler({ final: "d" }, (c4) => this.linePosAbsolute(c4)), this._parser.registerCsiHandler({ final: "e" }, (c4) => this.vPositionRelative(c4)), this._parser.registerCsiHandler({ final: "f" }, (c4) => this.hVPosition(c4)), this._parser.registerCsiHandler({ final: "g" }, (c4) => this.tabClear(c4)), this._parser.registerCsiHandler({ final: "h" }, (c4) => this.setMode(c4)), this._parser.registerCsiHandler({ prefix: "?", final: "h" }, (c4) => this.setModePrivate(c4)), this._parser.registerCsiHandler({ final: "l" }, (c4) => this.resetMode(c4)), this._parser.registerCsiHandler({ prefix: "?", final: "l" }, (c4) => this.resetModePrivate(c4)), this._parser.registerCsiHandler({ final: "m" }, (c4) => this.charAttributes(c4)), this._parser.registerCsiHandler({ final: "n" }, (c4) => this.deviceStatus(c4)), this._parser.registerCsiHandler({ prefix: "?", final: "n" }, (c4) => this.deviceStatusPrivate(c4)), this._parser.registerCsiHandler({ intermediates: "!", final: "p" }, (c4) => this.softReset(c4)), this._parser.registerCsiHandler({ intermediates: " ", final: "q" }, (c4) => this.setCursorStyle(c4)), this._parser.registerCsiHandler({ final: "r" }, (c4) => this.setScrollRegion(c4)), this._parser.registerCsiHandler({ final: "s" }, (c4) => this.saveCursor(c4)), this._parser.registerCsiHandler({ final: "t" }, (c4) => this.windowOptions(c4)), this._parser.registerCsiHandler({ final: "u" }, (c4) => this.restoreCursor(c4)), this._parser.registerCsiHandler({ intermediates: "'", final: "}" }, (c4) => this.insertColumns(c4)), this._parser.registerCsiHandler({ intermediates: "'", final: "~" }, (c4) => this.deleteColumns(c4)), this._parser.registerCsiHandler({ intermediates: '"', final: "q" }, (c4) => this.selectProtected(c4)), this._parser.registerCsiHandler({ intermediates: "$", final: "p" }, (c4) => this.requestMode(c4, true)), this._parser.registerCsiHandler({ prefix: "?", intermediates: "$", final: "p" }, (c4) => this.requestMode(c4, false)), this._parser.setExecuteHandler(b3.BEL, () => this.bell()), this._parser.setExecuteHandler(b3.LF, () => this.lineFeed()), this._parser.setExecuteHandler(b3.VT, () => this.lineFeed()), this._parser.setExecuteHandler(b3.FF, () => this.lineFeed()), this._parser.setExecuteHandler(b3.CR, () => this.carriageReturn()), this._parser.setExecuteHandler(b3.BS, () => this.backspace()), this._parser.setExecuteHandler(b3.HT, () => this.tab()), this._parser.setExecuteHandler(b3.SO, () => this.shiftOut()), this._parser.setExecuteHandler(b3.SI, () => this.shiftIn()), this._parser.setExecuteHandler(Ai.IND, () => this.index()), this._parser.setExecuteHandler(Ai.NEL, () => this.nextLine()), this._parser.setExecuteHandler(Ai.HTS, () => this.tabSet()), this._parser.registerOscHandler(0, new pe((c4) => (this.setTitle(c4), this.setIconName(c4), true))), this._parser.registerOscHandler(1, new pe((c4) => this.setIconName(c4))), this._parser.registerOscHandler(2, new pe((c4) => this.setTitle(c4))), this._parser.registerOscHandler(4, new pe((c4) => this.setOrReportIndexedColor(c4))), this._parser.registerOscHandler(8, new pe((c4) => this.setHyperlink(c4))), this._parser.registerOscHandler(10, new pe((c4) => this.setOrReportFgColor(c4))), this._parser.registerOscHandler(11, new pe((c4) => this.setOrReportBgColor(c4))), this._parser.registerOscHandler(12, new pe((c4) => this.setOrReportCursorColor(c4))), this._parser.registerOscHandler(104, new pe((c4) => this.restoreIndexedColor(c4))), this._parser.registerOscHandler(110, new pe((c4) => this.restoreFgColor(c4))), this._parser.registerOscHandler(111, new pe((c4) => this.restoreBgColor(c4))), this._parser.registerOscHandler(112, new pe((c4) => this.restoreCursorColor(c4))), this._parser.registerEscHandler({ final: "7" }, () => this.saveCursor()), this._parser.registerEscHandler({ final: "8" }, () => this.restoreCursor()), this._parser.registerEscHandler({ final: "D" }, () => this.index()), this._parser.registerEscHandler({ final: "E" }, () => this.nextLine()), this._parser.registerEscHandler({ final: "H" }, () => this.tabSet()), this._parser.registerEscHandler({ final: "M" }, () => this.reverseIndex()), this._parser.registerEscHandler({ final: "=" }, () => this.keypadApplicationMode()), this._parser.registerEscHandler({ final: ">" }, () => this.keypadNumericMode()), this._parser.registerEscHandler({ final: "c" }, () => this.fullReset()), this._parser.registerEscHandler({ final: "n" }, () => this.setgLevel(2)), this._parser.registerEscHandler({ final: "o" }, () => this.setgLevel(3)), this._parser.registerEscHandler({ final: "|" }, () => this.setgLevel(3)), this._parser.registerEscHandler({ final: "}" }, () => this.setgLevel(2)), this._parser.registerEscHandler({ final: "~" }, () => this.setgLevel(1)), this._parser.registerEscHandler({ intermediates: "%", final: "@" }, () => this.selectDefaultCharset()), this._parser.registerEscHandler({ intermediates: "%", final: "G" }, () => this.selectDefaultCharset());
        for (let c4 in ne) this._parser.registerEscHandler({ intermediates: "(", final: c4 }, () => this.selectCharset("(" + c4)), this._parser.registerEscHandler({ intermediates: ")", final: c4 }, () => this.selectCharset(")" + c4)), this._parser.registerEscHandler({ intermediates: "*", final: c4 }, () => this.selectCharset("*" + c4)), this._parser.registerEscHandler({ intermediates: "+", final: c4 }, () => this.selectCharset("+" + c4)), this._parser.registerEscHandler({ intermediates: "-", final: c4 }, () => this.selectCharset("-" + c4)), this._parser.registerEscHandler({ intermediates: ".", final: c4 }, () => this.selectCharset("." + c4)), this._parser.registerEscHandler({ intermediates: "/", final: c4 }, () => this.selectCharset("/" + c4));
        this._parser.registerEscHandler({ intermediates: "#", final: "8" }, () => this.screenAlignmentPattern()), this._parser.setErrorHandler((c4) => (this._logService.error("Parsing error: ", c4), c4)), this._parser.registerDcsHandler({ intermediates: "$", final: "q" }, new Xi((c4, d3) => this.requestStatusString(c4, d3)));
      }
      getAttrData() {
        return this._curAttrData;
      }
      _preserveStack(e4, i12, r7, n4) {
        this._parseStack.paused = true, this._parseStack.cursorStartX = e4, this._parseStack.cursorStartY = i12, this._parseStack.decodedLength = r7, this._parseStack.position = n4;
      }
      _logSlowResolvingAsync(e4) {
        this._logService.logLevel <= 3 && Promise.race([e4, new Promise((i12, r7) => setTimeout(() => r7("#SLOW_TIMEOUT"), vl))]).catch((i12) => {
          if (i12 !== "#SLOW_TIMEOUT") throw i12;
          console.warn(`async parser handler taking longer than ${vl} ms`);
        });
      }
      _getCurrentLinkId() {
        return this._curAttrData.extended.urlId;
      }
      parse(e4, i12) {
        let r7, n4 = this._activeBuffer.x, o6 = this._activeBuffer.y, l4 = 0, a3 = this._parseStack.paused;
        if (a3) {
          if (r7 = this._parser.parse(this._parseBuffer, this._parseStack.decodedLength, i12)) return this._logSlowResolvingAsync(r7), r7;
          n4 = this._parseStack.cursorStartX, o6 = this._parseStack.cursorStartY, this._parseStack.paused = false, e4.length > ut && (l4 = this._parseStack.position + ut);
        }
        if (this._logService.logLevel <= 1 && this._logService.debug(`parsing data ${typeof e4 == "string" ? ` "${e4}"` : ` "${Array.prototype.map.call(e4, (c4) => String.fromCharCode(c4)).join("")}"`}`), this._logService.logLevel === 0 && this._logService.trace("parsing data (codes)", typeof e4 == "string" ? e4.split("").map((c4) => c4.charCodeAt(0)) : e4), this._parseBuffer.length < e4.length && this._parseBuffer.length < ut && (this._parseBuffer = new Uint32Array(Math.min(e4.length, ut))), a3 || this._dirtyRowTracker.clearRange(), e4.length > ut) for (let c4 = l4; c4 < e4.length; c4 += ut) {
          let d3 = c4 + ut < e4.length ? c4 + ut : e4.length, _4 = typeof e4 == "string" ? this._stringDecoder.decode(e4.substring(c4, d3), this._parseBuffer) : this._utf8Decoder.decode(e4.subarray(c4, d3), this._parseBuffer);
          if (r7 = this._parser.parse(this._parseBuffer, _4)) return this._preserveStack(n4, o6, _4, c4), this._logSlowResolvingAsync(r7), r7;
        }
        else if (!a3) {
          let c4 = typeof e4 == "string" ? this._stringDecoder.decode(e4, this._parseBuffer) : this._utf8Decoder.decode(e4, this._parseBuffer);
          if (r7 = this._parser.parse(this._parseBuffer, c4)) return this._preserveStack(n4, o6, c4, 0), this._logSlowResolvingAsync(r7), r7;
        }
        (this._activeBuffer.x !== n4 || this._activeBuffer.y !== o6) && this._onCursorMove.fire();
        let u3 = this._dirtyRowTracker.end + (this._bufferService.buffer.ybase - this._bufferService.buffer.ydisp), h4 = this._dirtyRowTracker.start + (this._bufferService.buffer.ybase - this._bufferService.buffer.ydisp);
        h4 < this._bufferService.rows && this._onRequestRefreshRows.fire({ start: Math.min(h4, this._bufferService.rows - 1), end: Math.min(u3, this._bufferService.rows - 1) });
      }
      print(e4, i12, r7) {
        let n4, o6, l4 = this._charsetService.charset, a3 = this._optionsService.rawOptions.screenReaderMode, u3 = this._bufferService.cols, h4 = this._coreService.decPrivateModes.wraparound, c4 = this._coreService.modes.insertMode, d3 = this._curAttrData, _4 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y);
        this._dirtyRowTracker.markDirty(this._activeBuffer.y), this._activeBuffer.x && r7 - i12 > 0 && _4.getWidth(this._activeBuffer.x - 1) === 2 && _4.setCellFromCodepoint(this._activeBuffer.x - 1, 0, 1, d3);
        let p3 = this._parser.precedingJoinState;
        for (let m2 = i12; m2 < r7; ++m2) {
          if (n4 = e4[m2], n4 < 127 && l4) {
            let O3 = l4[String.fromCharCode(n4)];
            O3 && (n4 = O3.charCodeAt(0));
          }
          let f3 = this._unicodeService.charProperties(n4, p3);
          o6 = Ae.extractWidth(f3);
          let A3 = Ae.extractShouldJoin(f3), R2 = A3 ? Ae.extractWidth(p3) : 0;
          if (p3 = f3, a3 && this._onA11yChar.fire(Ce(n4)), this._getCurrentLinkId() && this._oscLinkService.addLineToLink(this._getCurrentLinkId(), this._activeBuffer.ybase + this._activeBuffer.y), this._activeBuffer.x + o6 - R2 > u3) {
            if (h4) {
              let O3 = _4, I2 = this._activeBuffer.x - R2;
              for (this._activeBuffer.x = R2, this._activeBuffer.y++, this._activeBuffer.y === this._activeBuffer.scrollBottom + 1 ? (this._activeBuffer.y--, this._bufferService.scroll(this._eraseAttrData(), true)) : (this._activeBuffer.y >= this._bufferService.rows && (this._activeBuffer.y = this._bufferService.rows - 1), this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y).isWrapped = true), _4 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y), R2 > 0 && _4 instanceof Ze && _4.copyCellsFrom(O3, I2, 0, R2, false); I2 < u3; ) O3.setCellFromCodepoint(I2++, 0, 1, d3);
            } else if (this._activeBuffer.x = u3 - 1, o6 === 2) continue;
          }
          if (A3 && this._activeBuffer.x) {
            let O3 = _4.getWidth(this._activeBuffer.x - 1) ? 1 : 2;
            _4.addCodepointToCell(this._activeBuffer.x - O3, n4, o6);
            for (let I2 = o6 - R2; --I2 >= 0; ) _4.setCellFromCodepoint(this._activeBuffer.x++, 0, 0, d3);
            continue;
          }
          if (c4 && (_4.insertCells(this._activeBuffer.x, o6 - R2, this._activeBuffer.getNullCell(d3)), _4.getWidth(u3 - 1) === 2 && _4.setCellFromCodepoint(u3 - 1, 0, 1, d3)), _4.setCellFromCodepoint(this._activeBuffer.x++, n4, o6, d3), o6 > 0) for (; --o6; ) _4.setCellFromCodepoint(this._activeBuffer.x++, 0, 0, d3);
        }
        this._parser.precedingJoinState = p3, this._activeBuffer.x < u3 && r7 - i12 > 0 && _4.getWidth(this._activeBuffer.x) === 0 && !_4.hasContent(this._activeBuffer.x) && _4.setCellFromCodepoint(this._activeBuffer.x, 0, 1, d3), this._dirtyRowTracker.markDirty(this._activeBuffer.y);
      }
      registerCsiHandler(e4, i12) {
        return e4.final === "t" && !e4.prefix && !e4.intermediates ? this._parser.registerCsiHandler(e4, (r7) => bl(r7.params[0], this._optionsService.rawOptions.windowOptions) ? i12(r7) : true) : this._parser.registerCsiHandler(e4, i12);
      }
      registerDcsHandler(e4, i12) {
        return this._parser.registerDcsHandler(e4, new Xi(i12));
      }
      registerEscHandler(e4, i12) {
        return this._parser.registerEscHandler(e4, i12);
      }
      registerOscHandler(e4, i12) {
        return this._parser.registerOscHandler(e4, new pe(i12));
      }
      bell() {
        return this._onRequestBell.fire(), true;
      }
      lineFeed() {
        return this._dirtyRowTracker.markDirty(this._activeBuffer.y), this._optionsService.rawOptions.convertEol && (this._activeBuffer.x = 0), this._activeBuffer.y++, this._activeBuffer.y === this._activeBuffer.scrollBottom + 1 ? (this._activeBuffer.y--, this._bufferService.scroll(this._eraseAttrData())) : this._activeBuffer.y >= this._bufferService.rows ? this._activeBuffer.y = this._bufferService.rows - 1 : this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y).isWrapped = false, this._activeBuffer.x >= this._bufferService.cols && this._activeBuffer.x--, this._dirtyRowTracker.markDirty(this._activeBuffer.y), this._onLineFeed.fire(), true;
      }
      carriageReturn() {
        return this._activeBuffer.x = 0, true;
      }
      backspace() {
        if (!this._coreService.decPrivateModes.reverseWraparound) return this._restrictCursor(), this._activeBuffer.x > 0 && this._activeBuffer.x--, true;
        if (this._restrictCursor(this._bufferService.cols), this._activeBuffer.x > 0) this._activeBuffer.x--;
        else if (this._activeBuffer.x === 0 && this._activeBuffer.y > this._activeBuffer.scrollTop && this._activeBuffer.y <= this._activeBuffer.scrollBottom && this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y)?.isWrapped) {
          this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y).isWrapped = false, this._activeBuffer.y--, this._activeBuffer.x = this._bufferService.cols - 1;
          let e4 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y);
          e4.hasWidth(this._activeBuffer.x) && !e4.hasContent(this._activeBuffer.x) && this._activeBuffer.x--;
        }
        return this._restrictCursor(), true;
      }
      tab() {
        if (this._activeBuffer.x >= this._bufferService.cols) return true;
        let e4 = this._activeBuffer.x;
        return this._activeBuffer.x = this._activeBuffer.nextStop(), this._optionsService.rawOptions.screenReaderMode && this._onA11yTab.fire(this._activeBuffer.x - e4), true;
      }
      shiftOut() {
        return this._charsetService.setgLevel(1), true;
      }
      shiftIn() {
        return this._charsetService.setgLevel(0), true;
      }
      _restrictCursor(e4 = this._bufferService.cols - 1) {
        this._activeBuffer.x = Math.min(e4, Math.max(0, this._activeBuffer.x)), this._activeBuffer.y = this._coreService.decPrivateModes.origin ? Math.min(this._activeBuffer.scrollBottom, Math.max(this._activeBuffer.scrollTop, this._activeBuffer.y)) : Math.min(this._bufferService.rows - 1, Math.max(0, this._activeBuffer.y)), this._dirtyRowTracker.markDirty(this._activeBuffer.y);
      }
      _setCursor(e4, i12) {
        this._dirtyRowTracker.markDirty(this._activeBuffer.y), this._coreService.decPrivateModes.origin ? (this._activeBuffer.x = e4, this._activeBuffer.y = this._activeBuffer.scrollTop + i12) : (this._activeBuffer.x = e4, this._activeBuffer.y = i12), this._restrictCursor(), this._dirtyRowTracker.markDirty(this._activeBuffer.y);
      }
      _moveCursor(e4, i12) {
        this._restrictCursor(), this._setCursor(this._activeBuffer.x + e4, this._activeBuffer.y + i12);
      }
      cursorUp(e4) {
        let i12 = this._activeBuffer.y - this._activeBuffer.scrollTop;
        return i12 >= 0 ? this._moveCursor(0, -Math.min(i12, e4.params[0] || 1)) : this._moveCursor(0, -(e4.params[0] || 1)), true;
      }
      cursorDown(e4) {
        let i12 = this._activeBuffer.scrollBottom - this._activeBuffer.y;
        return i12 >= 0 ? this._moveCursor(0, Math.min(i12, e4.params[0] || 1)) : this._moveCursor(0, e4.params[0] || 1), true;
      }
      cursorForward(e4) {
        return this._moveCursor(e4.params[0] || 1, 0), true;
      }
      cursorBackward(e4) {
        return this._moveCursor(-(e4.params[0] || 1), 0), true;
      }
      cursorNextLine(e4) {
        return this.cursorDown(e4), this._activeBuffer.x = 0, true;
      }
      cursorPrecedingLine(e4) {
        return this.cursorUp(e4), this._activeBuffer.x = 0, true;
      }
      cursorCharAbsolute(e4) {
        return this._setCursor((e4.params[0] || 1) - 1, this._activeBuffer.y), true;
      }
      cursorPosition(e4) {
        return this._setCursor(e4.length >= 2 ? (e4.params[1] || 1) - 1 : 0, (e4.params[0] || 1) - 1), true;
      }
      charPosAbsolute(e4) {
        return this._setCursor((e4.params[0] || 1) - 1, this._activeBuffer.y), true;
      }
      hPositionRelative(e4) {
        return this._moveCursor(e4.params[0] || 1, 0), true;
      }
      linePosAbsolute(e4) {
        return this._setCursor(this._activeBuffer.x, (e4.params[0] || 1) - 1), true;
      }
      vPositionRelative(e4) {
        return this._moveCursor(0, e4.params[0] || 1), true;
      }
      hVPosition(e4) {
        return this.cursorPosition(e4), true;
      }
      tabClear(e4) {
        let i12 = e4.params[0];
        return i12 === 0 ? delete this._activeBuffer.tabs[this._activeBuffer.x] : i12 === 3 && (this._activeBuffer.tabs = {}), true;
      }
      cursorForwardTab(e4) {
        if (this._activeBuffer.x >= this._bufferService.cols) return true;
        let i12 = e4.params[0] || 1;
        for (; i12--; ) this._activeBuffer.x = this._activeBuffer.nextStop();
        return true;
      }
      cursorBackwardTab(e4) {
        if (this._activeBuffer.x >= this._bufferService.cols) return true;
        let i12 = e4.params[0] || 1;
        for (; i12--; ) this._activeBuffer.x = this._activeBuffer.prevStop();
        return true;
      }
      selectProtected(e4) {
        let i12 = e4.params[0];
        return i12 === 1 && (this._curAttrData.bg |= 536870912), (i12 === 2 || i12 === 0) && (this._curAttrData.bg &= -536870913), true;
      }
      _eraseInBufferLine(e4, i12, r7, n4 = false, o6 = false) {
        let l4 = this._activeBuffer.lines.get(this._activeBuffer.ybase + e4);
        l4.replaceCells(i12, r7, this._activeBuffer.getNullCell(this._eraseAttrData()), o6), n4 && (l4.isWrapped = false);
      }
      _resetBufferLine(e4, i12 = false) {
        let r7 = this._activeBuffer.lines.get(this._activeBuffer.ybase + e4);
        r7 && (r7.fill(this._activeBuffer.getNullCell(this._eraseAttrData()), i12), this._bufferService.buffer.clearMarkers(this._activeBuffer.ybase + e4), r7.isWrapped = false);
      }
      eraseInDisplay(e4, i12 = false) {
        this._restrictCursor(this._bufferService.cols);
        let r7;
        switch (e4.params[0]) {
          case 0:
            for (r7 = this._activeBuffer.y, this._dirtyRowTracker.markDirty(r7), this._eraseInBufferLine(r7++, this._activeBuffer.x, this._bufferService.cols, this._activeBuffer.x === 0, i12); r7 < this._bufferService.rows; r7++) this._resetBufferLine(r7, i12);
            this._dirtyRowTracker.markDirty(r7);
            break;
          case 1:
            for (r7 = this._activeBuffer.y, this._dirtyRowTracker.markDirty(r7), this._eraseInBufferLine(r7, 0, this._activeBuffer.x + 1, true, i12), this._activeBuffer.x + 1 >= this._bufferService.cols && (this._activeBuffer.lines.get(r7 + 1).isWrapped = false); r7--; ) this._resetBufferLine(r7, i12);
            this._dirtyRowTracker.markDirty(0);
            break;
          case 2:
            if (this._optionsService.rawOptions.scrollOnEraseInDisplay) {
              for (r7 = this._bufferService.rows, this._dirtyRowTracker.markRangeDirty(0, r7 - 1); r7-- && !this._activeBuffer.lines.get(this._activeBuffer.ybase + r7)?.getTrimmedLength(); ) ;
              for (; r7 >= 0; r7--) this._bufferService.scroll(this._eraseAttrData());
            } else {
              for (r7 = this._bufferService.rows, this._dirtyRowTracker.markDirty(r7 - 1); r7--; ) this._resetBufferLine(r7, i12);
              this._dirtyRowTracker.markDirty(0);
            }
            break;
          case 3:
            let n4 = this._activeBuffer.lines.length - this._bufferService.rows;
            n4 > 0 && (this._activeBuffer.lines.trimStart(n4), this._activeBuffer.ybase = Math.max(this._activeBuffer.ybase - n4, 0), this._activeBuffer.ydisp = Math.max(this._activeBuffer.ydisp - n4, 0), this._onScroll.fire(0));
            break;
        }
        return true;
      }
      eraseInLine(e4, i12 = false) {
        switch (this._restrictCursor(this._bufferService.cols), e4.params[0]) {
          case 0:
            this._eraseInBufferLine(this._activeBuffer.y, this._activeBuffer.x, this._bufferService.cols, this._activeBuffer.x === 0, i12);
            break;
          case 1:
            this._eraseInBufferLine(this._activeBuffer.y, 0, this._activeBuffer.x + 1, false, i12);
            break;
          case 2:
            this._eraseInBufferLine(this._activeBuffer.y, 0, this._bufferService.cols, true, i12);
            break;
        }
        return this._dirtyRowTracker.markDirty(this._activeBuffer.y), true;
      }
      insertLines(e4) {
        this._restrictCursor();
        let i12 = e4.params[0] || 1;
        if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
        let r7 = this._activeBuffer.ybase + this._activeBuffer.y, n4 = this._bufferService.rows - 1 - this._activeBuffer.scrollBottom, o6 = this._bufferService.rows - 1 + this._activeBuffer.ybase - n4 + 1;
        for (; i12--; ) this._activeBuffer.lines.splice(o6 - 1, 1), this._activeBuffer.lines.splice(r7, 0, this._activeBuffer.getBlankLine(this._eraseAttrData()));
        return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.y, this._activeBuffer.scrollBottom), this._activeBuffer.x = 0, true;
      }
      deleteLines(e4) {
        this._restrictCursor();
        let i12 = e4.params[0] || 1;
        if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
        let r7 = this._activeBuffer.ybase + this._activeBuffer.y, n4;
        for (n4 = this._bufferService.rows - 1 - this._activeBuffer.scrollBottom, n4 = this._bufferService.rows - 1 + this._activeBuffer.ybase - n4; i12--; ) this._activeBuffer.lines.splice(r7, 1), this._activeBuffer.lines.splice(n4, 0, this._activeBuffer.getBlankLine(this._eraseAttrData()));
        return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.y, this._activeBuffer.scrollBottom), this._activeBuffer.x = 0, true;
      }
      insertChars(e4) {
        this._restrictCursor();
        let i12 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y);
        return i12 && (i12.insertCells(this._activeBuffer.x, e4.params[0] || 1, this._activeBuffer.getNullCell(this._eraseAttrData())), this._dirtyRowTracker.markDirty(this._activeBuffer.y)), true;
      }
      deleteChars(e4) {
        this._restrictCursor();
        let i12 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y);
        return i12 && (i12.deleteCells(this._activeBuffer.x, e4.params[0] || 1, this._activeBuffer.getNullCell(this._eraseAttrData())), this._dirtyRowTracker.markDirty(this._activeBuffer.y)), true;
      }
      scrollUp(e4) {
        let i12 = e4.params[0] || 1;
        for (; i12--; ) this._activeBuffer.lines.splice(this._activeBuffer.ybase + this._activeBuffer.scrollTop, 1), this._activeBuffer.lines.splice(this._activeBuffer.ybase + this._activeBuffer.scrollBottom, 0, this._activeBuffer.getBlankLine(this._eraseAttrData()));
        return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
      }
      scrollDown(e4) {
        let i12 = e4.params[0] || 1;
        for (; i12--; ) this._activeBuffer.lines.splice(this._activeBuffer.ybase + this._activeBuffer.scrollBottom, 1), this._activeBuffer.lines.splice(this._activeBuffer.ybase + this._activeBuffer.scrollTop, 0, this._activeBuffer.getBlankLine(X));
        return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
      }
      scrollLeft(e4) {
        if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
        let i12 = e4.params[0] || 1;
        for (let r7 = this._activeBuffer.scrollTop; r7 <= this._activeBuffer.scrollBottom; ++r7) {
          let n4 = this._activeBuffer.lines.get(this._activeBuffer.ybase + r7);
          n4.deleteCells(0, i12, this._activeBuffer.getNullCell(this._eraseAttrData())), n4.isWrapped = false;
        }
        return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
      }
      scrollRight(e4) {
        if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
        let i12 = e4.params[0] || 1;
        for (let r7 = this._activeBuffer.scrollTop; r7 <= this._activeBuffer.scrollBottom; ++r7) {
          let n4 = this._activeBuffer.lines.get(this._activeBuffer.ybase + r7);
          n4.insertCells(0, i12, this._activeBuffer.getNullCell(this._eraseAttrData())), n4.isWrapped = false;
        }
        return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
      }
      insertColumns(e4) {
        if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
        let i12 = e4.params[0] || 1;
        for (let r7 = this._activeBuffer.scrollTop; r7 <= this._activeBuffer.scrollBottom; ++r7) {
          let n4 = this._activeBuffer.lines.get(this._activeBuffer.ybase + r7);
          n4.insertCells(this._activeBuffer.x, i12, this._activeBuffer.getNullCell(this._eraseAttrData())), n4.isWrapped = false;
        }
        return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
      }
      deleteColumns(e4) {
        if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
        let i12 = e4.params[0] || 1;
        for (let r7 = this._activeBuffer.scrollTop; r7 <= this._activeBuffer.scrollBottom; ++r7) {
          let n4 = this._activeBuffer.lines.get(this._activeBuffer.ybase + r7);
          n4.deleteCells(this._activeBuffer.x, i12, this._activeBuffer.getNullCell(this._eraseAttrData())), n4.isWrapped = false;
        }
        return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
      }
      eraseChars(e4) {
        this._restrictCursor();
        let i12 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y);
        return i12 && (i12.replaceCells(this._activeBuffer.x, this._activeBuffer.x + (e4.params[0] || 1), this._activeBuffer.getNullCell(this._eraseAttrData())), this._dirtyRowTracker.markDirty(this._activeBuffer.y)), true;
      }
      repeatPrecedingCharacter(e4) {
        let i12 = this._parser.precedingJoinState;
        if (!i12) return true;
        let r7 = e4.params[0] || 1, n4 = Ae.extractWidth(i12), o6 = this._activeBuffer.x - n4, a3 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y).getString(o6), u3 = new Uint32Array(a3.length * r7), h4 = 0;
        for (let d3 = 0; d3 < a3.length; ) {
          let _4 = a3.codePointAt(d3) || 0;
          u3[h4++] = _4, d3 += _4 > 65535 ? 2 : 1;
        }
        let c4 = h4;
        for (let d3 = 1; d3 < r7; ++d3) u3.copyWithin(c4, 0, h4), c4 += h4;
        return this.print(u3, 0, c4), true;
      }
      sendDeviceAttributesPrimary(e4) {
        return e4.params[0] > 0 || (this._is("xterm") || this._is("rxvt-unicode") || this._is("screen") ? this._coreService.triggerDataEvent(b3.ESC + "[?1;2c") : this._is("linux") && this._coreService.triggerDataEvent(b3.ESC + "[?6c")), true;
      }
      sendDeviceAttributesSecondary(e4) {
        return e4.params[0] > 0 || (this._is("xterm") ? this._coreService.triggerDataEvent(b3.ESC + "[>0;276;0c") : this._is("rxvt-unicode") ? this._coreService.triggerDataEvent(b3.ESC + "[>85;95;0c") : this._is("linux") ? this._coreService.triggerDataEvent(e4.params[0] + "c") : this._is("screen") && this._coreService.triggerDataEvent(b3.ESC + "[>83;40003;0c")), true;
      }
      _is(e4) {
        return (this._optionsService.rawOptions.termName + "").indexOf(e4) === 0;
      }
      setMode(e4) {
        for (let i12 = 0; i12 < e4.length; i12++) switch (e4.params[i12]) {
          case 4:
            this._coreService.modes.insertMode = true;
            break;
          case 20:
            this._optionsService.options.convertEol = true;
            break;
        }
        return true;
      }
      setModePrivate(e4) {
        for (let i12 = 0; i12 < e4.length; i12++) switch (e4.params[i12]) {
          case 1:
            this._coreService.decPrivateModes.applicationCursorKeys = true;
            break;
          case 2:
            this._charsetService.setgCharset(0, Je), this._charsetService.setgCharset(1, Je), this._charsetService.setgCharset(2, Je), this._charsetService.setgCharset(3, Je);
            break;
          case 3:
            this._optionsService.rawOptions.windowOptions.setWinLines && (this._bufferService.resize(132, this._bufferService.rows), this._onRequestReset.fire());
            break;
          case 6:
            this._coreService.decPrivateModes.origin = true, this._setCursor(0, 0);
            break;
          case 7:
            this._coreService.decPrivateModes.wraparound = true;
            break;
          case 12:
            this._optionsService.options.cursorBlink = true;
            break;
          case 45:
            this._coreService.decPrivateModes.reverseWraparound = true;
            break;
          case 66:
            this._logService.debug("Serial port requested application keypad."), this._coreService.decPrivateModes.applicationKeypad = true, this._onRequestSyncScrollBar.fire();
            break;
          case 9:
            this._coreMouseService.activeProtocol = "X10";
            break;
          case 1e3:
            this._coreMouseService.activeProtocol = "VT200";
            break;
          case 1002:
            this._coreMouseService.activeProtocol = "DRAG";
            break;
          case 1003:
            this._coreMouseService.activeProtocol = "ANY";
            break;
          case 1004:
            this._coreService.decPrivateModes.sendFocus = true, this._onRequestSendFocus.fire();
            break;
          case 1005:
            this._logService.debug("DECSET 1005 not supported (see #2507)");
            break;
          case 1006:
            this._coreMouseService.activeEncoding = "SGR";
            break;
          case 1015:
            this._logService.debug("DECSET 1015 not supported (see #2507)");
            break;
          case 1016:
            this._coreMouseService.activeEncoding = "SGR_PIXELS";
            break;
          case 25:
            this._coreService.isCursorHidden = false;
            break;
          case 1048:
            this.saveCursor();
            break;
          case 1049:
            this.saveCursor();
          case 47:
          case 1047:
            this._bufferService.buffers.activateAltBuffer(this._eraseAttrData()), this._coreService.isCursorInitialized = true, this._onRequestRefreshRows.fire(void 0), this._onRequestSyncScrollBar.fire();
            break;
          case 2004:
            this._coreService.decPrivateModes.bracketedPasteMode = true;
            break;
          case 2026:
            this._coreService.decPrivateModes.synchronizedOutput = true;
            break;
        }
        return true;
      }
      resetMode(e4) {
        for (let i12 = 0; i12 < e4.length; i12++) switch (e4.params[i12]) {
          case 4:
            this._coreService.modes.insertMode = false;
            break;
          case 20:
            this._optionsService.options.convertEol = false;
            break;
        }
        return true;
      }
      resetModePrivate(e4) {
        for (let i12 = 0; i12 < e4.length; i12++) switch (e4.params[i12]) {
          case 1:
            this._coreService.decPrivateModes.applicationCursorKeys = false;
            break;
          case 3:
            this._optionsService.rawOptions.windowOptions.setWinLines && (this._bufferService.resize(80, this._bufferService.rows), this._onRequestReset.fire());
            break;
          case 6:
            this._coreService.decPrivateModes.origin = false, this._setCursor(0, 0);
            break;
          case 7:
            this._coreService.decPrivateModes.wraparound = false;
            break;
          case 12:
            this._optionsService.options.cursorBlink = false;
            break;
          case 45:
            this._coreService.decPrivateModes.reverseWraparound = false;
            break;
          case 66:
            this._logService.debug("Switching back to normal keypad."), this._coreService.decPrivateModes.applicationKeypad = false, this._onRequestSyncScrollBar.fire();
            break;
          case 9:
          case 1e3:
          case 1002:
          case 1003:
            this._coreMouseService.activeProtocol = "NONE";
            break;
          case 1004:
            this._coreService.decPrivateModes.sendFocus = false;
            break;
          case 1005:
            this._logService.debug("DECRST 1005 not supported (see #2507)");
            break;
          case 1006:
            this._coreMouseService.activeEncoding = "DEFAULT";
            break;
          case 1015:
            this._logService.debug("DECRST 1015 not supported (see #2507)");
            break;
          case 1016:
            this._coreMouseService.activeEncoding = "DEFAULT";
            break;
          case 25:
            this._coreService.isCursorHidden = true;
            break;
          case 1048:
            this.restoreCursor();
            break;
          case 1049:
          case 47:
          case 1047:
            this._bufferService.buffers.activateNormalBuffer(), e4.params[i12] === 1049 && this.restoreCursor(), this._coreService.isCursorInitialized = true, this._onRequestRefreshRows.fire(void 0), this._onRequestSyncScrollBar.fire();
            break;
          case 2004:
            this._coreService.decPrivateModes.bracketedPasteMode = false;
            break;
          case 2026:
            this._coreService.decPrivateModes.synchronizedOutput = false, this._onRequestRefreshRows.fire(void 0);
            break;
        }
        return true;
      }
      requestMode(e4, i12) {
        let r7;
        ((P4) => (P4[P4.NOT_RECOGNIZED = 0] = "NOT_RECOGNIZED", P4[P4.SET = 1] = "SET", P4[P4.RESET = 2] = "RESET", P4[P4.PERMANENTLY_SET = 3] = "PERMANENTLY_SET", P4[P4.PERMANENTLY_RESET = 4] = "PERMANENTLY_RESET"))(r7 || (r7 = {}));
        let n4 = this._coreService.decPrivateModes, { activeProtocol: o6, activeEncoding: l4 } = this._coreMouseService, a3 = this._coreService, { buffers: u3, cols: h4 } = this._bufferService, { active: c4, alt: d3 } = u3, _4 = this._optionsService.rawOptions, p3 = (A3, R2) => (a3.triggerDataEvent(`${b3.ESC}[${i12 ? "" : "?"}${A3};${R2}$y`), true), m2 = (A3) => A3 ? 1 : 2, f3 = e4.params[0];
        return i12 ? f3 === 2 ? p3(f3, 4) : f3 === 4 ? p3(f3, m2(a3.modes.insertMode)) : f3 === 12 ? p3(f3, 3) : f3 === 20 ? p3(f3, m2(_4.convertEol)) : p3(f3, 0) : f3 === 1 ? p3(f3, m2(n4.applicationCursorKeys)) : f3 === 3 ? p3(f3, _4.windowOptions.setWinLines ? h4 === 80 ? 2 : h4 === 132 ? 1 : 0 : 0) : f3 === 6 ? p3(f3, m2(n4.origin)) : f3 === 7 ? p3(f3, m2(n4.wraparound)) : f3 === 8 ? p3(f3, 3) : f3 === 9 ? p3(f3, m2(o6 === "X10")) : f3 === 12 ? p3(f3, m2(_4.cursorBlink)) : f3 === 25 ? p3(f3, m2(!a3.isCursorHidden)) : f3 === 45 ? p3(f3, m2(n4.reverseWraparound)) : f3 === 66 ? p3(f3, m2(n4.applicationKeypad)) : f3 === 67 ? p3(f3, 4) : f3 === 1e3 ? p3(f3, m2(o6 === "VT200")) : f3 === 1002 ? p3(f3, m2(o6 === "DRAG")) : f3 === 1003 ? p3(f3, m2(o6 === "ANY")) : f3 === 1004 ? p3(f3, m2(n4.sendFocus)) : f3 === 1005 ? p3(f3, 4) : f3 === 1006 ? p3(f3, m2(l4 === "SGR")) : f3 === 1015 ? p3(f3, 4) : f3 === 1016 ? p3(f3, m2(l4 === "SGR_PIXELS")) : f3 === 1048 ? p3(f3, 1) : f3 === 47 || f3 === 1047 || f3 === 1049 ? p3(f3, m2(c4 === d3)) : f3 === 2004 ? p3(f3, m2(n4.bracketedPasteMode)) : f3 === 2026 ? p3(f3, m2(n4.synchronizedOutput)) : p3(f3, 0);
      }
      _updateAttrColor(e4, i12, r7, n4, o6) {
        return i12 === 2 ? (e4 |= 50331648, e4 &= -16777216, e4 |= De.fromColorRGB([r7, n4, o6])) : i12 === 5 && (e4 &= -50331904, e4 |= 33554432 | r7 & 255), e4;
      }
      _extractColor(e4, i12, r7) {
        let n4 = [0, 0, -1, 0, 0, 0], o6 = 0, l4 = 0;
        do {
          if (n4[l4 + o6] = e4.params[i12 + l4], e4.hasSubParams(i12 + l4)) {
            let a3 = e4.getSubParams(i12 + l4), u3 = 0;
            do
              n4[1] === 5 && (o6 = 1), n4[l4 + u3 + 1 + o6] = a3[u3];
            while (++u3 < a3.length && u3 + l4 + 1 + o6 < n4.length);
            break;
          }
          if (n4[1] === 5 && l4 + o6 >= 2 || n4[1] === 2 && l4 + o6 >= 5) break;
          n4[1] && (o6 = 1);
        } while (++l4 + i12 < e4.length && l4 + o6 < n4.length);
        for (let a3 = 2; a3 < n4.length; ++a3) n4[a3] === -1 && (n4[a3] = 0);
        switch (n4[0]) {
          case 38:
            r7.fg = this._updateAttrColor(r7.fg, n4[1], n4[3], n4[4], n4[5]);
            break;
          case 48:
            r7.bg = this._updateAttrColor(r7.bg, n4[1], n4[3], n4[4], n4[5]);
            break;
          case 58:
            r7.extended = r7.extended.clone(), r7.extended.underlineColor = this._updateAttrColor(r7.extended.underlineColor, n4[1], n4[3], n4[4], n4[5]);
        }
        return l4;
      }
      _processUnderline(e4, i12) {
        i12.extended = i12.extended.clone(), (!~e4 || e4 > 5) && (e4 = 1), i12.extended.underlineStyle = e4, i12.fg |= 268435456, e4 === 0 && (i12.fg &= -268435457), i12.updateExtended();
      }
      _processSGR0(e4) {
        e4.fg = X.fg, e4.bg = X.bg, e4.extended = e4.extended.clone(), e4.extended.underlineStyle = 0, e4.extended.underlineColor &= -67108864, e4.updateExtended();
      }
      charAttributes(e4) {
        if (e4.length === 1 && e4.params[0] === 0) return this._processSGR0(this._curAttrData), true;
        let i12 = e4.length, r7, n4 = this._curAttrData;
        for (let o6 = 0; o6 < i12; o6++) r7 = e4.params[o6], r7 >= 30 && r7 <= 37 ? (n4.fg &= -50331904, n4.fg |= 16777216 | r7 - 30) : r7 >= 40 && r7 <= 47 ? (n4.bg &= -50331904, n4.bg |= 16777216 | r7 - 40) : r7 >= 90 && r7 <= 97 ? (n4.fg &= -50331904, n4.fg |= 16777216 | r7 - 90 | 8) : r7 >= 100 && r7 <= 107 ? (n4.bg &= -50331904, n4.bg |= 16777216 | r7 - 100 | 8) : r7 === 0 ? this._processSGR0(n4) : r7 === 1 ? n4.fg |= 134217728 : r7 === 3 ? n4.bg |= 67108864 : r7 === 4 ? (n4.fg |= 268435456, this._processUnderline(e4.hasSubParams(o6) ? e4.getSubParams(o6)[0] : 1, n4)) : r7 === 5 ? n4.fg |= 536870912 : r7 === 7 ? n4.fg |= 67108864 : r7 === 8 ? n4.fg |= 1073741824 : r7 === 9 ? n4.fg |= 2147483648 : r7 === 2 ? n4.bg |= 134217728 : r7 === 21 ? this._processUnderline(2, n4) : r7 === 22 ? (n4.fg &= -134217729, n4.bg &= -134217729) : r7 === 23 ? n4.bg &= -67108865 : r7 === 24 ? (n4.fg &= -268435457, this._processUnderline(0, n4)) : r7 === 25 ? n4.fg &= -536870913 : r7 === 27 ? n4.fg &= -67108865 : r7 === 28 ? n4.fg &= -1073741825 : r7 === 29 ? n4.fg &= 2147483647 : r7 === 39 ? (n4.fg &= -67108864, n4.fg |= X.fg & 16777215) : r7 === 49 ? (n4.bg &= -67108864, n4.bg |= X.bg & 16777215) : r7 === 38 || r7 === 48 || r7 === 58 ? o6 += this._extractColor(e4, o6, n4) : r7 === 53 ? n4.bg |= 1073741824 : r7 === 55 ? n4.bg &= -1073741825 : r7 === 59 ? (n4.extended = n4.extended.clone(), n4.extended.underlineColor = -1, n4.updateExtended()) : r7 === 100 ? (n4.fg &= -67108864, n4.fg |= X.fg & 16777215, n4.bg &= -67108864, n4.bg |= X.bg & 16777215) : this._logService.debug("Unknown SGR attribute: %d.", r7);
        return true;
      }
      deviceStatus(e4) {
        switch (e4.params[0]) {
          case 5:
            this._coreService.triggerDataEvent(`${b3.ESC}[0n`);
            break;
          case 6:
            let i12 = this._activeBuffer.y + 1, r7 = this._activeBuffer.x + 1;
            this._coreService.triggerDataEvent(`${b3.ESC}[${i12};${r7}R`);
            break;
        }
        return true;
      }
      deviceStatusPrivate(e4) {
        switch (e4.params[0]) {
          case 6:
            let i12 = this._activeBuffer.y + 1, r7 = this._activeBuffer.x + 1;
            this._coreService.triggerDataEvent(`${b3.ESC}[?${i12};${r7}R`);
            break;
          case 15:
            break;
          case 25:
            break;
          case 26:
            break;
          case 53:
            break;
        }
        return true;
      }
      softReset(e4) {
        return this._coreService.isCursorHidden = false, this._onRequestSyncScrollBar.fire(), this._activeBuffer.scrollTop = 0, this._activeBuffer.scrollBottom = this._bufferService.rows - 1, this._curAttrData = X.clone(), this._coreService.reset(), this._charsetService.reset(), this._activeBuffer.savedX = 0, this._activeBuffer.savedY = this._activeBuffer.ybase, this._activeBuffer.savedCurAttrData.fg = this._curAttrData.fg, this._activeBuffer.savedCurAttrData.bg = this._curAttrData.bg, this._activeBuffer.savedCharset = this._charsetService.charset, this._coreService.decPrivateModes.origin = false, true;
      }
      setCursorStyle(e4) {
        let i12 = e4.length === 0 ? 1 : e4.params[0];
        if (i12 === 0) this._coreService.decPrivateModes.cursorStyle = void 0, this._coreService.decPrivateModes.cursorBlink = void 0;
        else {
          switch (i12) {
            case 1:
            case 2:
              this._coreService.decPrivateModes.cursorStyle = "block";
              break;
            case 3:
            case 4:
              this._coreService.decPrivateModes.cursorStyle = "underline";
              break;
            case 5:
            case 6:
              this._coreService.decPrivateModes.cursorStyle = "bar";
              break;
          }
          let r7 = i12 % 2 === 1;
          this._coreService.decPrivateModes.cursorBlink = r7;
        }
        return true;
      }
      setScrollRegion(e4) {
        let i12 = e4.params[0] || 1, r7;
        return (e4.length < 2 || (r7 = e4.params[1]) > this._bufferService.rows || r7 === 0) && (r7 = this._bufferService.rows), r7 > i12 && (this._activeBuffer.scrollTop = i12 - 1, this._activeBuffer.scrollBottom = r7 - 1, this._setCursor(0, 0)), true;
      }
      windowOptions(e4) {
        if (!bl(e4.params[0], this._optionsService.rawOptions.windowOptions)) return true;
        let i12 = e4.length > 1 ? e4.params[1] : 0;
        switch (e4.params[0]) {
          case 14:
            i12 !== 2 && this._onRequestWindowsOptionsReport.fire(0);
            break;
          case 16:
            this._onRequestWindowsOptionsReport.fire(1);
            break;
          case 18:
            this._bufferService && this._coreService.triggerDataEvent(`${b3.ESC}[8;${this._bufferService.rows};${this._bufferService.cols}t`);
            break;
          case 22:
            (i12 === 0 || i12 === 2) && (this._windowTitleStack.push(this._windowTitle), this._windowTitleStack.length > _l && this._windowTitleStack.shift()), (i12 === 0 || i12 === 1) && (this._iconNameStack.push(this._iconName), this._iconNameStack.length > _l && this._iconNameStack.shift());
            break;
          case 23:
            (i12 === 0 || i12 === 2) && this._windowTitleStack.length && this.setTitle(this._windowTitleStack.pop()), (i12 === 0 || i12 === 1) && this._iconNameStack.length && this.setIconName(this._iconNameStack.pop());
            break;
        }
        return true;
      }
      saveCursor(e4) {
        return this._activeBuffer.savedX = this._activeBuffer.x, this._activeBuffer.savedY = this._activeBuffer.ybase + this._activeBuffer.y, this._activeBuffer.savedCurAttrData.fg = this._curAttrData.fg, this._activeBuffer.savedCurAttrData.bg = this._curAttrData.bg, this._activeBuffer.savedCharset = this._charsetService.charset, true;
      }
      restoreCursor(e4) {
        return this._activeBuffer.x = this._activeBuffer.savedX || 0, this._activeBuffer.y = Math.max(this._activeBuffer.savedY - this._activeBuffer.ybase, 0), this._curAttrData.fg = this._activeBuffer.savedCurAttrData.fg, this._curAttrData.bg = this._activeBuffer.savedCurAttrData.bg, this._charsetService.charset = this._savedCharset, this._activeBuffer.savedCharset && (this._charsetService.charset = this._activeBuffer.savedCharset), this._restrictCursor(), true;
      }
      setTitle(e4) {
        return this._windowTitle = e4, this._onTitleChange.fire(e4), true;
      }
      setIconName(e4) {
        return this._iconName = e4, true;
      }
      setOrReportIndexedColor(e4) {
        let i12 = [], r7 = e4.split(";");
        for (; r7.length > 1; ) {
          let n4 = r7.shift(), o6 = r7.shift();
          if (/^\d+$/.exec(n4)) {
            let l4 = parseInt(n4);
            if (Sl(l4)) if (o6 === "?") i12.push({ type: 0, index: l4 });
            else {
              let a3 = Ws(o6);
              a3 && i12.push({ type: 1, index: l4, color: a3 });
            }
          }
        }
        return i12.length && this._onColor.fire(i12), true;
      }
      setHyperlink(e4) {
        let i12 = e4.indexOf(";");
        if (i12 === -1) return true;
        let r7 = e4.slice(0, i12).trim(), n4 = e4.slice(i12 + 1);
        return n4 ? this._createHyperlink(r7, n4) : r7.trim() ? false : this._finishHyperlink();
      }
      _createHyperlink(e4, i12) {
        this._getCurrentLinkId() && this._finishHyperlink();
        let r7 = e4.split(":"), n4, o6 = r7.findIndex((l4) => l4.startsWith("id="));
        return o6 !== -1 && (n4 = r7[o6].slice(3) || void 0), this._curAttrData.extended = this._curAttrData.extended.clone(), this._curAttrData.extended.urlId = this._oscLinkService.registerLink({ id: n4, uri: i12 }), this._curAttrData.updateExtended(), true;
      }
      _finishHyperlink() {
        return this._curAttrData.extended = this._curAttrData.extended.clone(), this._curAttrData.extended.urlId = 0, this._curAttrData.updateExtended(), true;
      }
      _setOrReportSpecialColor(e4, i12) {
        let r7 = e4.split(";");
        for (let n4 = 0; n4 < r7.length && !(i12 >= this._specialColors.length); ++n4, ++i12) if (r7[n4] === "?") this._onColor.fire([{ type: 0, index: this._specialColors[i12] }]);
        else {
          let o6 = Ws(r7[n4]);
          o6 && this._onColor.fire([{ type: 1, index: this._specialColors[i12], color: o6 }]);
        }
        return true;
      }
      setOrReportFgColor(e4) {
        return this._setOrReportSpecialColor(e4, 0);
      }
      setOrReportBgColor(e4) {
        return this._setOrReportSpecialColor(e4, 1);
      }
      setOrReportCursorColor(e4) {
        return this._setOrReportSpecialColor(e4, 2);
      }
      restoreIndexedColor(e4) {
        if (!e4) return this._onColor.fire([{ type: 2 }]), true;
        let i12 = [], r7 = e4.split(";");
        for (let n4 = 0; n4 < r7.length; ++n4) if (/^\d+$/.exec(r7[n4])) {
          let o6 = parseInt(r7[n4]);
          Sl(o6) && i12.push({ type: 2, index: o6 });
        }
        return i12.length && this._onColor.fire(i12), true;
      }
      restoreFgColor(e4) {
        return this._onColor.fire([{ type: 2, index: 256 }]), true;
      }
      restoreBgColor(e4) {
        return this._onColor.fire([{ type: 2, index: 257 }]), true;
      }
      restoreCursorColor(e4) {
        return this._onColor.fire([{ type: 2, index: 258 }]), true;
      }
      nextLine() {
        return this._activeBuffer.x = 0, this.index(), true;
      }
      keypadApplicationMode() {
        return this._logService.debug("Serial port requested application keypad."), this._coreService.decPrivateModes.applicationKeypad = true, this._onRequestSyncScrollBar.fire(), true;
      }
      keypadNumericMode() {
        return this._logService.debug("Switching back to normal keypad."), this._coreService.decPrivateModes.applicationKeypad = false, this._onRequestSyncScrollBar.fire(), true;
      }
      selectDefaultCharset() {
        return this._charsetService.setgLevel(0), this._charsetService.setgCharset(0, Je), true;
      }
      selectCharset(e4) {
        return e4.length !== 2 ? (this.selectDefaultCharset(), true) : (e4[0] === "/" || this._charsetService.setgCharset(mc[e4[0]], ne[e4[1]] || Je), true);
      }
      index() {
        return this._restrictCursor(), this._activeBuffer.y++, this._activeBuffer.y === this._activeBuffer.scrollBottom + 1 ? (this._activeBuffer.y--, this._bufferService.scroll(this._eraseAttrData())) : this._activeBuffer.y >= this._bufferService.rows && (this._activeBuffer.y = this._bufferService.rows - 1), this._restrictCursor(), true;
      }
      tabSet() {
        return this._activeBuffer.tabs[this._activeBuffer.x] = true, true;
      }
      reverseIndex() {
        if (this._restrictCursor(), this._activeBuffer.y === this._activeBuffer.scrollTop) {
          let e4 = this._activeBuffer.scrollBottom - this._activeBuffer.scrollTop;
          this._activeBuffer.lines.shiftElements(this._activeBuffer.ybase + this._activeBuffer.y, e4, 1), this._activeBuffer.lines.set(this._activeBuffer.ybase + this._activeBuffer.y, this._activeBuffer.getBlankLine(this._eraseAttrData())), this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom);
        } else this._activeBuffer.y--, this._restrictCursor();
        return true;
      }
      fullReset() {
        return this._parser.reset(), this._onRequestReset.fire(), true;
      }
      reset() {
        this._curAttrData = X.clone(), this._eraseAttrDataInternal = X.clone();
      }
      _eraseAttrData() {
        return this._eraseAttrDataInternal.bg &= -67108864, this._eraseAttrDataInternal.bg |= this._curAttrData.bg & 67108863, this._eraseAttrDataInternal;
      }
      setgLevel(e4) {
        return this._charsetService.setgLevel(e4), true;
      }
      screenAlignmentPattern() {
        let e4 = new q();
        e4.content = 1 << 22 | 69, e4.fg = this._curAttrData.fg, e4.bg = this._curAttrData.bg, this._setCursor(0, 0);
        for (let i12 = 0; i12 < this._bufferService.rows; ++i12) {
          let r7 = this._activeBuffer.ybase + this._activeBuffer.y + i12, n4 = this._activeBuffer.lines.get(r7);
          n4 && (n4.fill(e4), n4.isWrapped = false);
        }
        return this._dirtyRowTracker.markAllDirty(), this._setCursor(0, 0), true;
      }
      requestStatusString(e4, i12) {
        let r7 = (a3) => (this._coreService.triggerDataEvent(`${b3.ESC}${a3}${b3.ESC}\\`), true), n4 = this._bufferService.buffer, o6 = this._optionsService.rawOptions, l4 = { block: 2, underline: 4, bar: 6 };
        return r7(e4 === '"q' ? `P1$r${this._curAttrData.isProtected() ? 1 : 0}"q` : e4 === '"p' ? 'P1$r61;1"p' : e4 === "r" ? `P1$r${n4.scrollTop + 1};${n4.scrollBottom + 1}r` : e4 === "m" ? "P1$r0m" : e4 === " q" ? `P1$r${l4[o6.cursorStyle] - (o6.cursorBlink ? 1 : 0)} q` : "P0$r");
      }
      markRangeDirty(e4, i12) {
        this._dirtyRowTracker.markRangeDirty(e4, i12);
      }
    };
    Zi = class {
      constructor(t3) {
        this._bufferService = t3;
        this.clearRange();
      }
      clearRange() {
        this.start = this._bufferService.buffer.y, this.end = this._bufferService.buffer.y;
      }
      markDirty(t3) {
        t3 < this.start ? this.start = t3 : t3 > this.end && (this.end = t3);
      }
      markRangeDirty(t3, e4) {
        t3 > e4 && (gl = t3, t3 = e4, e4 = gl), t3 < this.start && (this.start = t3), e4 > this.end && (this.end = e4);
      }
      markAllDirty() {
        this.markRangeDirty(0, this._bufferService.rows - 1);
      }
    };
    Zi = M2([S3(0, F)], Zi);
    _c = 5e7;
    El = 12;
    bc = 50;
    gn = class extends D2 {
      constructor(e4) {
        super();
        this._action = e4;
        this._writeBuffer = [];
        this._callbacks = [];
        this._pendingData = 0;
        this._bufferOffset = 0;
        this._isSyncWriting = false;
        this._syncCalls = 0;
        this._didUserInput = false;
        this._onWriteParsed = this._register(new v2());
        this.onWriteParsed = this._onWriteParsed.event;
      }
      handleUserInput() {
        this._didUserInput = true;
      }
      writeSync(e4, i12) {
        if (i12 !== void 0 && this._syncCalls > i12) {
          this._syncCalls = 0;
          return;
        }
        if (this._pendingData += e4.length, this._writeBuffer.push(e4), this._callbacks.push(void 0), this._syncCalls++, this._isSyncWriting) return;
        this._isSyncWriting = true;
        let r7;
        for (; r7 = this._writeBuffer.shift(); ) {
          this._action(r7);
          let n4 = this._callbacks.shift();
          n4 && n4();
        }
        this._pendingData = 0, this._bufferOffset = 2147483647, this._isSyncWriting = false, this._syncCalls = 0;
      }
      write(e4, i12) {
        if (this._pendingData > _c) throw new Error("write data discarded, use flow control to avoid losing data");
        if (!this._writeBuffer.length) {
          if (this._bufferOffset = 0, this._didUserInput) {
            this._didUserInput = false, this._pendingData += e4.length, this._writeBuffer.push(e4), this._callbacks.push(i12), this._innerWrite();
            return;
          }
          setTimeout(() => this._innerWrite());
        }
        this._pendingData += e4.length, this._writeBuffer.push(e4), this._callbacks.push(i12);
      }
      _innerWrite(e4 = 0, i12 = true) {
        let r7 = e4 || performance.now();
        for (; this._writeBuffer.length > this._bufferOffset; ) {
          let n4 = this._writeBuffer[this._bufferOffset], o6 = this._action(n4, i12);
          if (o6) {
            let a3 = (u3) => performance.now() - r7 >= El ? setTimeout(() => this._innerWrite(0, u3)) : this._innerWrite(r7, u3);
            o6.catch((u3) => (queueMicrotask(() => {
              throw u3;
            }), Promise.resolve(false))).then(a3);
            return;
          }
          let l4 = this._callbacks[this._bufferOffset];
          if (l4 && l4(), this._bufferOffset++, this._pendingData -= n4.length, performance.now() - r7 >= El) break;
        }
        this._writeBuffer.length > this._bufferOffset ? (this._bufferOffset > bc && (this._writeBuffer = this._writeBuffer.slice(this._bufferOffset), this._callbacks = this._callbacks.slice(this._bufferOffset), this._bufferOffset = 0), setTimeout(() => this._innerWrite())) : (this._writeBuffer.length = 0, this._callbacks.length = 0, this._pendingData = 0, this._bufferOffset = 0), this._onWriteParsed.fire();
      }
    };
    ui = class {
      constructor(t3) {
        this._bufferService = t3;
        this._nextId = 1;
        this._entriesWithId = /* @__PURE__ */ new Map();
        this._dataByLinkId = /* @__PURE__ */ new Map();
      }
      registerLink(t3) {
        let e4 = this._bufferService.buffer;
        if (t3.id === void 0) {
          let a3 = e4.addMarker(e4.ybase + e4.y), u3 = { data: t3, id: this._nextId++, lines: [a3] };
          return a3.onDispose(() => this._removeMarkerFromLink(u3, a3)), this._dataByLinkId.set(u3.id, u3), u3.id;
        }
        let i12 = t3, r7 = this._getEntryIdKey(i12), n4 = this._entriesWithId.get(r7);
        if (n4) return this.addLineToLink(n4.id, e4.ybase + e4.y), n4.id;
        let o6 = e4.addMarker(e4.ybase + e4.y), l4 = { id: this._nextId++, key: this._getEntryIdKey(i12), data: i12, lines: [o6] };
        return o6.onDispose(() => this._removeMarkerFromLink(l4, o6)), this._entriesWithId.set(l4.key, l4), this._dataByLinkId.set(l4.id, l4), l4.id;
      }
      addLineToLink(t3, e4) {
        let i12 = this._dataByLinkId.get(t3);
        if (i12 && i12.lines.every((r7) => r7.line !== e4)) {
          let r7 = this._bufferService.buffer.addMarker(e4);
          i12.lines.push(r7), r7.onDispose(() => this._removeMarkerFromLink(i12, r7));
        }
      }
      getLinkData(t3) {
        return this._dataByLinkId.get(t3)?.data;
      }
      _getEntryIdKey(t3) {
        return `${t3.id};;${t3.uri}`;
      }
      _removeMarkerFromLink(t3, e4) {
        let i12 = t3.lines.indexOf(e4);
        i12 !== -1 && (t3.lines.splice(i12, 1), t3.lines.length === 0 && (t3.data.id !== void 0 && this._entriesWithId.delete(t3.key), this._dataByLinkId.delete(t3.id)));
      }
    };
    ui = M2([S3(0, F)], ui);
    Tl = false;
    Sn = class extends D2 {
      constructor(e4) {
        super();
        this._windowsWrappingHeuristics = this._register(new ye());
        this._onBinary = this._register(new v2());
        this.onBinary = this._onBinary.event;
        this._onData = this._register(new v2());
        this.onData = this._onData.event;
        this._onLineFeed = this._register(new v2());
        this.onLineFeed = this._onLineFeed.event;
        this._onResize = this._register(new v2());
        this.onResize = this._onResize.event;
        this._onWriteParsed = this._register(new v2());
        this.onWriteParsed = this._onWriteParsed.event;
        this._onScroll = this._register(new v2());
        this._instantiationService = new ln(), this.optionsService = this._register(new dn(e4)), this._instantiationService.setService(H2, this.optionsService), this._bufferService = this._register(this._instantiationService.createInstance(ni)), this._instantiationService.setService(F, this._bufferService), this._logService = this._register(this._instantiationService.createInstance(ii)), this._instantiationService.setService(nr, this._logService), this.coreService = this._register(this._instantiationService.createInstance(li)), this._instantiationService.setService(ge, this.coreService), this.coreMouseService = this._register(this._instantiationService.createInstance(ai)), this._instantiationService.setService(rr, this.coreMouseService), this.unicodeService = this._register(this._instantiationService.createInstance(Ae)), this._instantiationService.setService(Js, this.unicodeService), this._charsetService = this._instantiationService.createInstance(pn), this._instantiationService.setService(Zs, this._charsetService), this._oscLinkService = this._instantiationService.createInstance(ui), this._instantiationService.setService(sr, this._oscLinkService), this._inputHandler = this._register(new vn(this._bufferService, this._charsetService, this.coreService, this._logService, this.optionsService, this._oscLinkService, this.coreMouseService, this.unicodeService)), this._register($2.forward(this._inputHandler.onLineFeed, this._onLineFeed)), this._register(this._inputHandler), this._register($2.forward(this._bufferService.onResize, this._onResize)), this._register($2.forward(this.coreService.onData, this._onData)), this._register($2.forward(this.coreService.onBinary, this._onBinary)), this._register(this.coreService.onRequestScrollToBottom(() => this.scrollToBottom(true))), this._register(this.coreService.onUserInput(() => this._writeBuffer.handleUserInput())), this._register(this.optionsService.onMultipleOptionChange(["windowsMode", "windowsPty"], () => this._handleWindowsPtyOptionChange())), this._register(this._bufferService.onScroll(() => {
          this._onScroll.fire({ position: this._bufferService.buffer.ydisp }), this._inputHandler.markRangeDirty(this._bufferService.buffer.scrollTop, this._bufferService.buffer.scrollBottom);
        })), this._writeBuffer = this._register(new gn((i12, r7) => this._inputHandler.parse(i12, r7))), this._register($2.forward(this._writeBuffer.onWriteParsed, this._onWriteParsed));
      }
      get onScroll() {
        return this._onScrollApi || (this._onScrollApi = this._register(new v2()), this._onScroll.event((e4) => {
          this._onScrollApi?.fire(e4.position);
        })), this._onScrollApi.event;
      }
      get cols() {
        return this._bufferService.cols;
      }
      get rows() {
        return this._bufferService.rows;
      }
      get buffers() {
        return this._bufferService.buffers;
      }
      get options() {
        return this.optionsService.options;
      }
      set options(e4) {
        for (let i12 in e4) this.optionsService.options[i12] = e4[i12];
      }
      write(e4, i12) {
        this._writeBuffer.write(e4, i12);
      }
      writeSync(e4, i12) {
        this._logService.logLevel <= 3 && !Tl && (this._logService.warn("writeSync is unreliable and will be removed soon."), Tl = true), this._writeBuffer.writeSync(e4, i12);
      }
      input(e4, i12 = true) {
        this.coreService.triggerDataEvent(e4, i12);
      }
      resize(e4, i12) {
        isNaN(e4) || isNaN(i12) || (e4 = Math.max(e4, ks), i12 = Math.max(i12, Cs), this._bufferService.resize(e4, i12));
      }
      scroll(e4, i12 = false) {
        this._bufferService.scroll(e4, i12);
      }
      scrollLines(e4, i12) {
        this._bufferService.scrollLines(e4, i12);
      }
      scrollPages(e4) {
        this.scrollLines(e4 * (this.rows - 1));
      }
      scrollToTop() {
        this.scrollLines(-this._bufferService.buffer.ydisp);
      }
      scrollToBottom(e4) {
        this.scrollLines(this._bufferService.buffer.ybase - this._bufferService.buffer.ydisp);
      }
      scrollToLine(e4) {
        let i12 = e4 - this._bufferService.buffer.ydisp;
        i12 !== 0 && this.scrollLines(i12);
      }
      registerEscHandler(e4, i12) {
        return this._inputHandler.registerEscHandler(e4, i12);
      }
      registerDcsHandler(e4, i12) {
        return this._inputHandler.registerDcsHandler(e4, i12);
      }
      registerCsiHandler(e4, i12) {
        return this._inputHandler.registerCsiHandler(e4, i12);
      }
      registerOscHandler(e4, i12) {
        return this._inputHandler.registerOscHandler(e4, i12);
      }
      _setup() {
        this._handleWindowsPtyOptionChange();
      }
      reset() {
        this._inputHandler.reset(), this._bufferService.reset(), this._charsetService.reset(), this.coreService.reset(), this.coreMouseService.reset();
      }
      _handleWindowsPtyOptionChange() {
        let e4 = false, i12 = this.optionsService.rawOptions.windowsPty;
        i12 && i12.buildNumber !== void 0 && i12.buildNumber !== void 0 ? e4 = i12.backend === "conpty" && i12.buildNumber < 21376 : this.optionsService.rawOptions.windowsMode && (e4 = true), e4 ? this._enableWindowsWrappingHeuristics() : this._windowsWrappingHeuristics.clear();
      }
      _enableWindowsWrappingHeuristics() {
        if (!this._windowsWrappingHeuristics.value) {
          let e4 = [];
          e4.push(this.onLineFeed(Bs.bind(null, this._bufferService))), e4.push(this.registerCsiHandler({ final: "H" }, () => (Bs(this._bufferService), false))), this._windowsWrappingHeuristics.value = C2(() => {
            for (let i12 of e4) i12.dispose();
          });
        }
      }
    };
    gc = { 48: ["0", ")"], 49: ["1", "!"], 50: ["2", "@"], 51: ["3", "#"], 52: ["4", "$"], 53: ["5", "%"], 54: ["6", "^"], 55: ["7", "&"], 56: ["8", "*"], 57: ["9", "("], 186: [";", ":"], 187: ["=", "+"], 188: [",", "<"], 189: ["-", "_"], 190: [".", ">"], 191: ["/", "?"], 192: ["`", "~"], 219: ["[", "{"], 220: ["\\", "|"], 221: ["]", "}"], 222: ["'", '"'] };
    ee = 0;
    En = class {
      constructor(t3) {
        this._getKey = t3;
        this._array = [];
        this._insertedValues = [];
        this._flushInsertedTask = new Jt();
        this._isFlushingInserted = false;
        this._deletedIndices = [];
        this._flushDeletedTask = new Jt();
        this._isFlushingDeleted = false;
      }
      clear() {
        this._array.length = 0, this._insertedValues.length = 0, this._flushInsertedTask.clear(), this._isFlushingInserted = false, this._deletedIndices.length = 0, this._flushDeletedTask.clear(), this._isFlushingDeleted = false;
      }
      insert(t3) {
        this._flushCleanupDeleted(), this._insertedValues.length === 0 && this._flushInsertedTask.enqueue(() => this._flushInserted()), this._insertedValues.push(t3);
      }
      _flushInserted() {
        let t3 = this._insertedValues.sort((n4, o6) => this._getKey(n4) - this._getKey(o6)), e4 = 0, i12 = 0, r7 = new Array(this._array.length + this._insertedValues.length);
        for (let n4 = 0; n4 < r7.length; n4++) i12 >= this._array.length || this._getKey(t3[e4]) <= this._getKey(this._array[i12]) ? (r7[n4] = t3[e4], e4++) : r7[n4] = this._array[i12++];
        this._array = r7, this._insertedValues.length = 0;
      }
      _flushCleanupInserted() {
        !this._isFlushingInserted && this._insertedValues.length > 0 && this._flushInsertedTask.flush();
      }
      delete(t3) {
        if (this._flushCleanupInserted(), this._array.length === 0) return false;
        let e4 = this._getKey(t3);
        if (e4 === void 0 || (ee = this._search(e4), ee === -1) || this._getKey(this._array[ee]) !== e4) return false;
        do
          if (this._array[ee] === t3) return this._deletedIndices.length === 0 && this._flushDeletedTask.enqueue(() => this._flushDeleted()), this._deletedIndices.push(ee), true;
        while (++ee < this._array.length && this._getKey(this._array[ee]) === e4);
        return false;
      }
      _flushDeleted() {
        this._isFlushingDeleted = true;
        let t3 = this._deletedIndices.sort((n4, o6) => n4 - o6), e4 = 0, i12 = new Array(this._array.length - t3.length), r7 = 0;
        for (let n4 = 0; n4 < this._array.length; n4++) t3[e4] === n4 ? e4++ : i12[r7++] = this._array[n4];
        this._array = i12, this._deletedIndices.length = 0, this._isFlushingDeleted = false;
      }
      _flushCleanupDeleted() {
        !this._isFlushingDeleted && this._deletedIndices.length > 0 && this._flushDeletedTask.flush();
      }
      *getKeyIterator(t3) {
        if (this._flushCleanupInserted(), this._flushCleanupDeleted(), this._array.length !== 0 && (ee = this._search(t3), !(ee < 0 || ee >= this._array.length) && this._getKey(this._array[ee]) === t3)) do
          yield this._array[ee];
        while (++ee < this._array.length && this._getKey(this._array[ee]) === t3);
      }
      forEachByKey(t3, e4) {
        if (this._flushCleanupInserted(), this._flushCleanupDeleted(), this._array.length !== 0 && (ee = this._search(t3), !(ee < 0 || ee >= this._array.length) && this._getKey(this._array[ee]) === t3)) do
          e4(this._array[ee]);
        while (++ee < this._array.length && this._getKey(this._array[ee]) === t3);
      }
      values() {
        return this._flushCleanupInserted(), this._flushCleanupDeleted(), [...this._array].values();
      }
      _search(t3) {
        let e4 = 0, i12 = this._array.length - 1;
        for (; i12 >= e4; ) {
          let r7 = e4 + i12 >> 1, n4 = this._getKey(this._array[r7]);
          if (n4 > t3) i12 = r7 - 1;
          else if (n4 < t3) e4 = r7 + 1;
          else {
            for (; r7 > 0 && this._getKey(this._array[r7 - 1]) === t3; ) r7--;
            return r7;
          }
        }
        return e4;
      }
    };
    Us = 0;
    yl = 0;
    Tn = class extends D2 {
      constructor() {
        super();
        this._decorations = new En((e4) => e4?.marker.line);
        this._onDecorationRegistered = this._register(new v2());
        this.onDecorationRegistered = this._onDecorationRegistered.event;
        this._onDecorationRemoved = this._register(new v2());
        this.onDecorationRemoved = this._onDecorationRemoved.event;
        this._register(C2(() => this.reset()));
      }
      get decorations() {
        return this._decorations.values();
      }
      registerDecoration(e4) {
        if (e4.marker.isDisposed) return;
        let i12 = new Ks(e4);
        if (i12) {
          let r7 = i12.marker.onDispose(() => i12.dispose()), n4 = i12.onDispose(() => {
            n4.dispose(), i12 && (this._decorations.delete(i12) && this._onDecorationRemoved.fire(i12), r7.dispose());
          });
          this._decorations.insert(i12), this._onDecorationRegistered.fire(i12);
        }
        return i12;
      }
      reset() {
        for (let e4 of this._decorations.values()) e4.dispose();
        this._decorations.clear();
      }
      *getDecorationsAtCell(e4, i12, r7) {
        let n4 = 0, o6 = 0;
        for (let l4 of this._decorations.getKeyIterator(i12)) n4 = l4.options.x ?? 0, o6 = n4 + (l4.options.width ?? 1), e4 >= n4 && e4 < o6 && (!r7 || (l4.options.layer ?? "bottom") === r7) && (yield l4);
      }
      forEachDecorationAtCell(e4, i12, r7, n4) {
        this._decorations.forEachByKey(i12, (o6) => {
          Us = o6.options.x ?? 0, yl = Us + (o6.options.width ?? 1), e4 >= Us && e4 < yl && (!r7 || (o6.options.layer ?? "bottom") === r7) && n4(o6);
        });
      }
    };
    Ks = class extends Ee {
      constructor(e4) {
        super();
        this.options = e4;
        this.onRenderEmitter = this.add(new v2());
        this.onRender = this.onRenderEmitter.event;
        this._onDispose = this.add(new v2());
        this.onDispose = this._onDispose.event;
        this._cachedBg = null;
        this._cachedFg = null;
        this.marker = e4.marker, this.options.overviewRulerOptions && !this.options.overviewRulerOptions.position && (this.options.overviewRulerOptions.position = "full");
      }
      get backgroundColorRGB() {
        return this._cachedBg === null && (this.options.backgroundColor ? this._cachedBg = z2.toColor(this.options.backgroundColor) : this._cachedBg = void 0), this._cachedBg;
      }
      get foregroundColorRGB() {
        return this._cachedFg === null && (this.options.foregroundColor ? this._cachedFg = z2.toColor(this.options.foregroundColor) : this._cachedFg = void 0), this._cachedFg;
      }
      dispose() {
        this._onDispose.fire(), super.dispose();
      }
    };
    Sc = 1e3;
    In = class {
      constructor(t3, e4 = Sc) {
        this._renderCallback = t3;
        this._debounceThresholdMS = e4;
        this._lastRefreshMs = 0;
        this._additionalRefreshRequested = false;
      }
      dispose() {
        this._refreshTimeoutID && clearTimeout(this._refreshTimeoutID);
      }
      refresh(t3, e4, i12) {
        this._rowCount = i12, t3 = t3 !== void 0 ? t3 : 0, e4 = e4 !== void 0 ? e4 : this._rowCount - 1, this._rowStart = this._rowStart !== void 0 ? Math.min(this._rowStart, t3) : t3, this._rowEnd = this._rowEnd !== void 0 ? Math.max(this._rowEnd, e4) : e4;
        let r7 = performance.now();
        if (r7 - this._lastRefreshMs >= this._debounceThresholdMS) this._lastRefreshMs = r7, this._innerRefresh();
        else if (!this._additionalRefreshRequested) {
          let n4 = r7 - this._lastRefreshMs, o6 = this._debounceThresholdMS - n4;
          this._additionalRefreshRequested = true, this._refreshTimeoutID = window.setTimeout(() => {
            this._lastRefreshMs = performance.now(), this._innerRefresh(), this._additionalRefreshRequested = false, this._refreshTimeoutID = void 0;
          }, o6);
        }
      }
      _innerRefresh() {
        if (this._rowStart === void 0 || this._rowEnd === void 0 || this._rowCount === void 0) return;
        let t3 = Math.max(this._rowStart, 0), e4 = Math.min(this._rowEnd, this._rowCount - 1);
        this._rowStart = void 0, this._rowEnd = void 0, this._renderCallback(t3, e4);
      }
    };
    xl = 20;
    wl = false;
    Tt = class extends D2 {
      constructor(e4, i12, r7, n4) {
        super();
        this._terminal = e4;
        this._coreBrowserService = r7;
        this._renderService = n4;
        this._rowColumns = /* @__PURE__ */ new WeakMap();
        this._liveRegionLineCount = 0;
        this._charsToConsume = [];
        this._charsToAnnounce = "";
        let o6 = this._coreBrowserService.mainDocument;
        this._accessibilityContainer = o6.createElement("div"), this._accessibilityContainer.classList.add("xterm-accessibility"), this._rowContainer = o6.createElement("div"), this._rowContainer.setAttribute("role", "list"), this._rowContainer.classList.add("xterm-accessibility-tree"), this._rowElements = [];
        for (let l4 = 0; l4 < this._terminal.rows; l4++) this._rowElements[l4] = this._createAccessibilityTreeNode(), this._rowContainer.appendChild(this._rowElements[l4]);
        if (this._topBoundaryFocusListener = (l4) => this._handleBoundaryFocus(l4, 0), this._bottomBoundaryFocusListener = (l4) => this._handleBoundaryFocus(l4, 1), this._rowElements[0].addEventListener("focus", this._topBoundaryFocusListener), this._rowElements[this._rowElements.length - 1].addEventListener("focus", this._bottomBoundaryFocusListener), this._accessibilityContainer.appendChild(this._rowContainer), this._liveRegion = o6.createElement("div"), this._liveRegion.classList.add("live-region"), this._liveRegion.setAttribute("aria-live", "assertive"), this._accessibilityContainer.appendChild(this._liveRegion), this._liveRegionDebouncer = this._register(new In(this._renderRows.bind(this))), !this._terminal.element) throw new Error("Cannot enable accessibility before Terminal.open");
        wl ? (this._accessibilityContainer.classList.add("debug"), this._rowContainer.classList.add("debug"), this._debugRootContainer = o6.createElement("div"), this._debugRootContainer.classList.add("xterm"), this._debugRootContainer.appendChild(o6.createTextNode("------start a11y------")), this._debugRootContainer.appendChild(this._accessibilityContainer), this._debugRootContainer.appendChild(o6.createTextNode("------end a11y------")), this._terminal.element.insertAdjacentElement("afterend", this._debugRootContainer)) : this._terminal.element.insertAdjacentElement("afterbegin", this._accessibilityContainer), this._register(this._terminal.onResize((l4) => this._handleResize(l4.rows))), this._register(this._terminal.onRender((l4) => this._refreshRows(l4.start, l4.end))), this._register(this._terminal.onScroll(() => this._refreshRows())), this._register(this._terminal.onA11yChar((l4) => this._handleChar(l4))), this._register(this._terminal.onLineFeed(() => this._handleChar(`
`))), this._register(this._terminal.onA11yTab((l4) => this._handleTab(l4))), this._register(this._terminal.onKey((l4) => this._handleKey(l4.key))), this._register(this._terminal.onBlur(() => this._clearLiveRegion())), this._register(this._renderService.onDimensionsChange(() => this._refreshRowsDimensions())), this._register(L2(o6, "selectionchange", () => this._handleSelectionChange())), this._register(this._coreBrowserService.onDprChange(() => this._refreshRowsDimensions())), this._refreshRowsDimensions(), this._refreshRows(), this._register(C2(() => {
          wl ? this._debugRootContainer.remove() : this._accessibilityContainer.remove(), this._rowElements.length = 0;
        }));
      }
      _handleTab(e4) {
        for (let i12 = 0; i12 < e4; i12++) this._handleChar(" ");
      }
      _handleChar(e4) {
        this._liveRegionLineCount < xl + 1 && (this._charsToConsume.length > 0 ? this._charsToConsume.shift() !== e4 && (this._charsToAnnounce += e4) : this._charsToAnnounce += e4, e4 === `
` && (this._liveRegionLineCount++, this._liveRegionLineCount === xl + 1 && (this._liveRegion.textContent += _i.get())));
      }
      _clearLiveRegion() {
        this._liveRegion.textContent = "", this._liveRegionLineCount = 0;
      }
      _handleKey(e4) {
        this._clearLiveRegion(), /\p{Control}/u.test(e4) || this._charsToConsume.push(e4);
      }
      _refreshRows(e4, i12) {
        this._liveRegionDebouncer.refresh(e4, i12, this._terminal.rows);
      }
      _renderRows(e4, i12) {
        let r7 = this._terminal.buffer, n4 = r7.lines.length.toString();
        for (let o6 = e4; o6 <= i12; o6++) {
          let l4 = r7.lines.get(r7.ydisp + o6), a3 = [], u3 = l4?.translateToString(true, void 0, void 0, a3) || "", h4 = (r7.ydisp + o6 + 1).toString(), c4 = this._rowElements[o6];
          c4 && (u3.length === 0 ? (c4.textContent = "\xA0", this._rowColumns.set(c4, [0, 1])) : (c4.textContent = u3, this._rowColumns.set(c4, a3)), c4.setAttribute("aria-posinset", h4), c4.setAttribute("aria-setsize", n4), this._alignRowWidth(c4));
        }
        this._announceCharacters();
      }
      _announceCharacters() {
        this._charsToAnnounce.length !== 0 && (this._liveRegion.textContent += this._charsToAnnounce, this._charsToAnnounce = "");
      }
      _handleBoundaryFocus(e4, i12) {
        let r7 = e4.target, n4 = this._rowElements[i12 === 0 ? 1 : this._rowElements.length - 2], o6 = r7.getAttribute("aria-posinset"), l4 = i12 === 0 ? "1" : `${this._terminal.buffer.lines.length}`;
        if (o6 === l4 || e4.relatedTarget !== n4) return;
        let a3, u3;
        if (i12 === 0 ? (a3 = r7, u3 = this._rowElements.pop(), this._rowContainer.removeChild(u3)) : (a3 = this._rowElements.shift(), u3 = r7, this._rowContainer.removeChild(a3)), a3.removeEventListener("focus", this._topBoundaryFocusListener), u3.removeEventListener("focus", this._bottomBoundaryFocusListener), i12 === 0) {
          let h4 = this._createAccessibilityTreeNode();
          this._rowElements.unshift(h4), this._rowContainer.insertAdjacentElement("afterbegin", h4);
        } else {
          let h4 = this._createAccessibilityTreeNode();
          this._rowElements.push(h4), this._rowContainer.appendChild(h4);
        }
        this._rowElements[0].addEventListener("focus", this._topBoundaryFocusListener), this._rowElements[this._rowElements.length - 1].addEventListener("focus", this._bottomBoundaryFocusListener), this._terminal.scrollLines(i12 === 0 ? -1 : 1), this._rowElements[i12 === 0 ? 1 : this._rowElements.length - 2].focus(), e4.preventDefault(), e4.stopImmediatePropagation();
      }
      _handleSelectionChange() {
        if (this._rowElements.length === 0) return;
        let e4 = this._coreBrowserService.mainDocument.getSelection();
        if (!e4) return;
        if (e4.isCollapsed) {
          this._rowContainer.contains(e4.anchorNode) && this._terminal.clearSelection();
          return;
        }
        if (!e4.anchorNode || !e4.focusNode) {
          console.error("anchorNode and/or focusNode are null");
          return;
        }
        let i12 = { node: e4.anchorNode, offset: e4.anchorOffset }, r7 = { node: e4.focusNode, offset: e4.focusOffset };
        if ((i12.node.compareDocumentPosition(r7.node) & Node.DOCUMENT_POSITION_PRECEDING || i12.node === r7.node && i12.offset > r7.offset) && ([i12, r7] = [r7, i12]), i12.node.compareDocumentPosition(this._rowElements[0]) & (Node.DOCUMENT_POSITION_CONTAINED_BY | Node.DOCUMENT_POSITION_FOLLOWING) && (i12 = { node: this._rowElements[0].childNodes[0], offset: 0 }), !this._rowContainer.contains(i12.node)) return;
        let n4 = this._rowElements.slice(-1)[0];
        if (r7.node.compareDocumentPosition(n4) & (Node.DOCUMENT_POSITION_CONTAINED_BY | Node.DOCUMENT_POSITION_PRECEDING) && (r7 = { node: n4, offset: n4.textContent?.length ?? 0 }), !this._rowContainer.contains(r7.node)) return;
        let o6 = ({ node: u3, offset: h4 }) => {
          let c4 = u3 instanceof Text ? u3.parentNode : u3, d3 = parseInt(c4?.getAttribute("aria-posinset"), 10) - 1;
          if (isNaN(d3)) return console.warn("row is invalid. Race condition?"), null;
          let _4 = this._rowColumns.get(c4);
          if (!_4) return console.warn("columns is null. Race condition?"), null;
          let p3 = h4 < _4.length ? _4[h4] : _4.slice(-1)[0] + 1;
          return p3 >= this._terminal.cols && (++d3, p3 = 0), { row: d3, column: p3 };
        }, l4 = o6(i12), a3 = o6(r7);
        if (!(!l4 || !a3)) {
          if (l4.row > a3.row || l4.row === a3.row && l4.column >= a3.column) throw new Error("invalid range");
          this._terminal.select(l4.column, l4.row, (a3.row - l4.row) * this._terminal.cols - l4.column + a3.column);
        }
      }
      _handleResize(e4) {
        this._rowElements[this._rowElements.length - 1].removeEventListener("focus", this._bottomBoundaryFocusListener);
        for (let i12 = this._rowContainer.children.length; i12 < this._terminal.rows; i12++) this._rowElements[i12] = this._createAccessibilityTreeNode(), this._rowContainer.appendChild(this._rowElements[i12]);
        for (; this._rowElements.length > e4; ) this._rowContainer.removeChild(this._rowElements.pop());
        this._rowElements[this._rowElements.length - 1].addEventListener("focus", this._bottomBoundaryFocusListener), this._refreshRowsDimensions();
      }
      _createAccessibilityTreeNode() {
        let e4 = this._coreBrowserService.mainDocument.createElement("div");
        return e4.setAttribute("role", "listitem"), e4.tabIndex = -1, this._refreshRowDimensions(e4), e4;
      }
      _refreshRowsDimensions() {
        if (this._renderService.dimensions.css.cell.height) {
          Object.assign(this._accessibilityContainer.style, { width: `${this._renderService.dimensions.css.canvas.width}px`, fontSize: `${this._terminal.options.fontSize}px` }), this._rowElements.length !== this._terminal.rows && this._handleResize(this._terminal.rows);
          for (let e4 = 0; e4 < this._terminal.rows; e4++) this._refreshRowDimensions(this._rowElements[e4]), this._alignRowWidth(this._rowElements[e4]);
        }
      }
      _refreshRowDimensions(e4) {
        e4.style.height = `${this._renderService.dimensions.css.cell.height}px`;
      }
      _alignRowWidth(e4) {
        e4.style.transform = "";
        let i12 = e4.getBoundingClientRect().width, r7 = this._rowColumns.get(e4)?.slice(-1)?.[0];
        if (!r7) return;
        let n4 = r7 * this._renderService.dimensions.css.cell.width;
        e4.style.transform = `scaleX(${n4 / i12})`;
      }
    };
    Tt = M2([S3(1, xt), S3(2, ae), S3(3, ce)], Tt);
    hi = class extends D2 {
      constructor(e4, i12, r7, n4, o6) {
        super();
        this._element = e4;
        this._mouseService = i12;
        this._renderService = r7;
        this._bufferService = n4;
        this._linkProviderService = o6;
        this._linkCacheDisposables = [];
        this._isMouseOut = true;
        this._wasResized = false;
        this._activeLine = -1;
        this._onShowLinkUnderline = this._register(new v2());
        this.onShowLinkUnderline = this._onShowLinkUnderline.event;
        this._onHideLinkUnderline = this._register(new v2());
        this.onHideLinkUnderline = this._onHideLinkUnderline.event;
        this._register(C2(() => {
          Ne(this._linkCacheDisposables), this._linkCacheDisposables.length = 0, this._lastMouseEvent = void 0, this._activeProviderReplies?.clear();
        })), this._register(this._bufferService.onResize(() => {
          this._clearCurrentLink(), this._wasResized = true;
        })), this._register(L2(this._element, "mouseleave", () => {
          this._isMouseOut = true, this._clearCurrentLink();
        })), this._register(L2(this._element, "mousemove", this._handleMouseMove.bind(this))), this._register(L2(this._element, "mousedown", this._handleMouseDown.bind(this))), this._register(L2(this._element, "mouseup", this._handleMouseUp.bind(this)));
      }
      get currentLink() {
        return this._currentLink;
      }
      _handleMouseMove(e4) {
        this._lastMouseEvent = e4;
        let i12 = this._positionFromMouseEvent(e4, this._element, this._mouseService);
        if (!i12) return;
        this._isMouseOut = false;
        let r7 = e4.composedPath();
        for (let n4 = 0; n4 < r7.length; n4++) {
          let o6 = r7[n4];
          if (o6.classList.contains("xterm")) break;
          if (o6.classList.contains("xterm-hover")) return;
        }
        (!this._lastBufferCell || i12.x !== this._lastBufferCell.x || i12.y !== this._lastBufferCell.y) && (this._handleHover(i12), this._lastBufferCell = i12);
      }
      _handleHover(e4) {
        if (this._activeLine !== e4.y || this._wasResized) {
          this._clearCurrentLink(), this._askForLink(e4, false), this._wasResized = false;
          return;
        }
        this._currentLink && this._linkAtPosition(this._currentLink.link, e4) || (this._clearCurrentLink(), this._askForLink(e4, true));
      }
      _askForLink(e4, i12) {
        (!this._activeProviderReplies || !i12) && (this._activeProviderReplies?.forEach((n4) => {
          n4?.forEach((o6) => {
            o6.link.dispose && o6.link.dispose();
          });
        }), this._activeProviderReplies = /* @__PURE__ */ new Map(), this._activeLine = e4.y);
        let r7 = false;
        for (let [n4, o6] of this._linkProviderService.linkProviders.entries()) i12 ? this._activeProviderReplies?.get(n4) && (r7 = this._checkLinkProviderResult(n4, e4, r7)) : o6.provideLinks(e4.y, (l4) => {
          if (this._isMouseOut) return;
          let a3 = l4?.map((u3) => ({ link: u3 }));
          this._activeProviderReplies?.set(n4, a3), r7 = this._checkLinkProviderResult(n4, e4, r7), this._activeProviderReplies?.size === this._linkProviderService.linkProviders.length && this._removeIntersectingLinks(e4.y, this._activeProviderReplies);
        });
      }
      _removeIntersectingLinks(e4, i12) {
        let r7 = /* @__PURE__ */ new Set();
        for (let n4 = 0; n4 < i12.size; n4++) {
          let o6 = i12.get(n4);
          if (o6) for (let l4 = 0; l4 < o6.length; l4++) {
            let a3 = o6[l4], u3 = a3.link.range.start.y < e4 ? 0 : a3.link.range.start.x, h4 = a3.link.range.end.y > e4 ? this._bufferService.cols : a3.link.range.end.x;
            for (let c4 = u3; c4 <= h4; c4++) {
              if (r7.has(c4)) {
                o6.splice(l4--, 1);
                break;
              }
              r7.add(c4);
            }
          }
        }
      }
      _checkLinkProviderResult(e4, i12, r7) {
        if (!this._activeProviderReplies) return r7;
        let n4 = this._activeProviderReplies.get(e4), o6 = false;
        for (let l4 = 0; l4 < e4; l4++) (!this._activeProviderReplies.has(l4) || this._activeProviderReplies.get(l4)) && (o6 = true);
        if (!o6 && n4) {
          let l4 = n4.find((a3) => this._linkAtPosition(a3.link, i12));
          l4 && (r7 = true, this._handleNewLink(l4));
        }
        if (this._activeProviderReplies.size === this._linkProviderService.linkProviders.length && !r7) for (let l4 = 0; l4 < this._activeProviderReplies.size; l4++) {
          let a3 = this._activeProviderReplies.get(l4)?.find((u3) => this._linkAtPosition(u3.link, i12));
          if (a3) {
            r7 = true, this._handleNewLink(a3);
            break;
          }
        }
        return r7;
      }
      _handleMouseDown() {
        this._mouseDownLink = this._currentLink;
      }
      _handleMouseUp(e4) {
        if (!this._currentLink) return;
        let i12 = this._positionFromMouseEvent(e4, this._element, this._mouseService);
        i12 && this._mouseDownLink && Ec(this._mouseDownLink.link, this._currentLink.link) && this._linkAtPosition(this._currentLink.link, i12) && this._currentLink.link.activate(e4, this._currentLink.link.text);
      }
      _clearCurrentLink(e4, i12) {
        !this._currentLink || !this._lastMouseEvent || (!e4 || !i12 || this._currentLink.link.range.start.y >= e4 && this._currentLink.link.range.end.y <= i12) && (this._linkLeave(this._element, this._currentLink.link, this._lastMouseEvent), this._currentLink = void 0, Ne(this._linkCacheDisposables), this._linkCacheDisposables.length = 0);
      }
      _handleNewLink(e4) {
        if (!this._lastMouseEvent) return;
        let i12 = this._positionFromMouseEvent(this._lastMouseEvent, this._element, this._mouseService);
        i12 && this._linkAtPosition(e4.link, i12) && (this._currentLink = e4, this._currentLink.state = { decorations: { underline: e4.link.decorations === void 0 ? true : e4.link.decorations.underline, pointerCursor: e4.link.decorations === void 0 ? true : e4.link.decorations.pointerCursor }, isHovered: true }, this._linkHover(this._element, e4.link, this._lastMouseEvent), e4.link.decorations = {}, Object.defineProperties(e4.link.decorations, { pointerCursor: { get: () => this._currentLink?.state?.decorations.pointerCursor, set: (r7) => {
          this._currentLink?.state && this._currentLink.state.decorations.pointerCursor !== r7 && (this._currentLink.state.decorations.pointerCursor = r7, this._currentLink.state.isHovered && this._element.classList.toggle("xterm-cursor-pointer", r7));
        } }, underline: { get: () => this._currentLink?.state?.decorations.underline, set: (r7) => {
          this._currentLink?.state && this._currentLink?.state?.decorations.underline !== r7 && (this._currentLink.state.decorations.underline = r7, this._currentLink.state.isHovered && this._fireUnderlineEvent(e4.link, r7));
        } } }), this._linkCacheDisposables.push(this._renderService.onRenderedViewportChange((r7) => {
          if (!this._currentLink) return;
          let n4 = r7.start === 0 ? 0 : r7.start + 1 + this._bufferService.buffer.ydisp, o6 = this._bufferService.buffer.ydisp + 1 + r7.end;
          if (this._currentLink.link.range.start.y >= n4 && this._currentLink.link.range.end.y <= o6 && (this._clearCurrentLink(n4, o6), this._lastMouseEvent)) {
            let l4 = this._positionFromMouseEvent(this._lastMouseEvent, this._element, this._mouseService);
            l4 && this._askForLink(l4, false);
          }
        })));
      }
      _linkHover(e4, i12, r7) {
        this._currentLink?.state && (this._currentLink.state.isHovered = true, this._currentLink.state.decorations.underline && this._fireUnderlineEvent(i12, true), this._currentLink.state.decorations.pointerCursor && e4.classList.add("xterm-cursor-pointer")), i12.hover && i12.hover(r7, i12.text);
      }
      _fireUnderlineEvent(e4, i12) {
        let r7 = e4.range, n4 = this._bufferService.buffer.ydisp, o6 = this._createLinkUnderlineEvent(r7.start.x - 1, r7.start.y - n4 - 1, r7.end.x, r7.end.y - n4 - 1, void 0);
        (i12 ? this._onShowLinkUnderline : this._onHideLinkUnderline).fire(o6);
      }
      _linkLeave(e4, i12, r7) {
        this._currentLink?.state && (this._currentLink.state.isHovered = false, this._currentLink.state.decorations.underline && this._fireUnderlineEvent(i12, false), this._currentLink.state.decorations.pointerCursor && e4.classList.remove("xterm-cursor-pointer")), i12.leave && i12.leave(r7, i12.text);
      }
      _linkAtPosition(e4, i12) {
        let r7 = e4.range.start.y * this._bufferService.cols + e4.range.start.x, n4 = e4.range.end.y * this._bufferService.cols + e4.range.end.x, o6 = i12.y * this._bufferService.cols + i12.x;
        return r7 <= o6 && o6 <= n4;
      }
      _positionFromMouseEvent(e4, i12, r7) {
        let n4 = r7.getCoords(e4, i12, this._bufferService.cols, this._bufferService.rows);
        if (n4) return { x: n4[0], y: n4[1] + this._bufferService.buffer.ydisp };
      }
      _createLinkUnderlineEvent(e4, i12, r7, n4, o6) {
        return { x1: e4, y1: i12, x2: r7, y2: n4, cols: this._bufferService.cols, fg: o6 };
      }
    };
    hi = M2([S3(1, Dt), S3(2, ce), S3(3, F), S3(4, lr)], hi);
    yn = class extends Sn {
      constructor(e4 = {}) {
        super(e4);
        this._linkifier = this._register(new ye());
        this.browser = tn;
        this._keyDownHandled = false;
        this._keyDownSeen = false;
        this._keyPressHandled = false;
        this._unprocessedDeadKey = false;
        this._accessibilityManager = this._register(new ye());
        this._onCursorMove = this._register(new v2());
        this.onCursorMove = this._onCursorMove.event;
        this._onKey = this._register(new v2());
        this.onKey = this._onKey.event;
        this._onRender = this._register(new v2());
        this.onRender = this._onRender.event;
        this._onSelectionChange = this._register(new v2());
        this.onSelectionChange = this._onSelectionChange.event;
        this._onTitleChange = this._register(new v2());
        this.onTitleChange = this._onTitleChange.event;
        this._onBell = this._register(new v2());
        this.onBell = this._onBell.event;
        this._onFocus = this._register(new v2());
        this._onBlur = this._register(new v2());
        this._onA11yCharEmitter = this._register(new v2());
        this._onA11yTabEmitter = this._register(new v2());
        this._onWillOpen = this._register(new v2());
        this._setup(), this._decorationService = this._instantiationService.createInstance(Tn), this._instantiationService.setService(Be, this._decorationService), this._linkProviderService = this._instantiationService.createInstance(Qr), this._instantiationService.setService(lr, this._linkProviderService), this._linkProviderService.registerLinkProvider(this._instantiationService.createInstance(wt)), this._register(this._inputHandler.onRequestBell(() => this._onBell.fire())), this._register(this._inputHandler.onRequestRefreshRows((i12) => this.refresh(i12?.start ?? 0, i12?.end ?? this.rows - 1))), this._register(this._inputHandler.onRequestSendFocus(() => this._reportFocus())), this._register(this._inputHandler.onRequestReset(() => this.reset())), this._register(this._inputHandler.onRequestWindowsOptionsReport((i12) => this._reportWindowsOptions(i12))), this._register(this._inputHandler.onColor((i12) => this._handleColorEvent(i12))), this._register($2.forward(this._inputHandler.onCursorMove, this._onCursorMove)), this._register($2.forward(this._inputHandler.onTitleChange, this._onTitleChange)), this._register($2.forward(this._inputHandler.onA11yChar, this._onA11yCharEmitter)), this._register($2.forward(this._inputHandler.onA11yTab, this._onA11yTabEmitter)), this._register(this._bufferService.onResize((i12) => this._afterResize(i12.cols, i12.rows))), this._register(C2(() => {
          this._customKeyEventHandler = void 0, this.element?.parentNode?.removeChild(this.element);
        }));
      }
      get linkifier() {
        return this._linkifier.value;
      }
      get onFocus() {
        return this._onFocus.event;
      }
      get onBlur() {
        return this._onBlur.event;
      }
      get onA11yChar() {
        return this._onA11yCharEmitter.event;
      }
      get onA11yTab() {
        return this._onA11yTabEmitter.event;
      }
      get onWillOpen() {
        return this._onWillOpen.event;
      }
      _handleColorEvent(e4) {
        if (this._themeService) for (let i12 of e4) {
          let r7, n4 = "";
          switch (i12.index) {
            case 256:
              r7 = "foreground", n4 = "10";
              break;
            case 257:
              r7 = "background", n4 = "11";
              break;
            case 258:
              r7 = "cursor", n4 = "12";
              break;
            default:
              r7 = "ansi", n4 = "4;" + i12.index;
          }
          switch (i12.type) {
            case 0:
              let o6 = U.toColorRGB(r7 === "ansi" ? this._themeService.colors.ansi[i12.index] : this._themeService.colors[r7]);
              this.coreService.triggerDataEvent(`${b3.ESC}]${n4};${ml(o6)}${fs.ST}`);
              break;
            case 1:
              if (r7 === "ansi") this._themeService.modifyColors((l4) => l4.ansi[i12.index] = j.toColor(...i12.color));
              else {
                let l4 = r7;
                this._themeService.modifyColors((a3) => a3[l4] = j.toColor(...i12.color));
              }
              break;
            case 2:
              this._themeService.restoreColor(i12.index);
              break;
          }
        }
      }
      _setup() {
        super._setup(), this._customKeyEventHandler = void 0;
      }
      get buffer() {
        return this.buffers.active;
      }
      focus() {
        this.textarea && this.textarea.focus({ preventScroll: true });
      }
      _handleScreenReaderModeOptionChange(e4) {
        e4 ? !this._accessibilityManager.value && this._renderService && (this._accessibilityManager.value = this._instantiationService.createInstance(Tt, this)) : this._accessibilityManager.clear();
      }
      _handleTextAreaFocus(e4) {
        this.coreService.decPrivateModes.sendFocus && this.coreService.triggerDataEvent(b3.ESC + "[I"), this.element.classList.add("focus"), this._showCursor(), this._onFocus.fire();
      }
      blur() {
        return this.textarea?.blur();
      }
      _handleTextAreaBlur() {
        this.textarea.value = "", this.refresh(this.buffer.y, this.buffer.y), this.coreService.decPrivateModes.sendFocus && this.coreService.triggerDataEvent(b3.ESC + "[O"), this.element.classList.remove("focus"), this._onBlur.fire();
      }
      _syncTextArea() {
        if (!this.textarea || !this.buffer.isCursorInViewport || this._compositionHelper.isComposing || !this._renderService) return;
        let e4 = this.buffer.ybase + this.buffer.y, i12 = this.buffer.lines.get(e4);
        if (!i12) return;
        let r7 = Math.min(this.buffer.x, this.cols - 1), n4 = this._renderService.dimensions.css.cell.height, o6 = i12.getWidth(r7), l4 = this._renderService.dimensions.css.cell.width * o6, a3 = this.buffer.y * this._renderService.dimensions.css.cell.height, u3 = r7 * this._renderService.dimensions.css.cell.width;
        this.textarea.style.left = u3 + "px", this.textarea.style.top = a3 + "px", this.textarea.style.width = l4 + "px", this.textarea.style.height = n4 + "px", this.textarea.style.lineHeight = n4 + "px", this.textarea.style.zIndex = "-5";
      }
      _initGlobal() {
        this._bindKeys(), this._register(L2(this.element, "copy", (i12) => {
          this.hasSelection() && Vs(i12, this._selectionService);
        }));
        let e4 = (i12) => qs(i12, this.textarea, this.coreService, this.optionsService);
        this._register(L2(this.textarea, "paste", e4)), this._register(L2(this.element, "paste", e4)), Ss ? this._register(L2(this.element, "mousedown", (i12) => {
          i12.button === 2 && Pn(i12, this.textarea, this.screenElement, this._selectionService, this.options.rightClickSelectsWord);
        })) : this._register(L2(this.element, "contextmenu", (i12) => {
          Pn(i12, this.textarea, this.screenElement, this._selectionService, this.options.rightClickSelectsWord);
        })), Bi && this._register(L2(this.element, "auxclick", (i12) => {
          i12.button === 1 && Mn(i12, this.textarea, this.screenElement);
        }));
      }
      _bindKeys() {
        this._register(L2(this.textarea, "keyup", (e4) => this._keyUp(e4), true)), this._register(L2(this.textarea, "keydown", (e4) => this._keyDown(e4), true)), this._register(L2(this.textarea, "keypress", (e4) => this._keyPress(e4), true)), this._register(L2(this.textarea, "compositionstart", () => this._compositionHelper.compositionstart())), this._register(L2(this.textarea, "compositionupdate", (e4) => this._compositionHelper.compositionupdate(e4))), this._register(L2(this.textarea, "compositionend", () => this._compositionHelper.compositionend())), this._register(L2(this.textarea, "input", (e4) => this._inputEvent(e4), true)), this._register(this.onRender(() => this._compositionHelper.updateCompositionElements()));
      }
      open(e4) {
        if (!e4) throw new Error("Terminal requires a parent element.");
        if (e4.isConnected || this._logService.debug("Terminal.open was called on an element that was not attached to the DOM"), this.element?.ownerDocument.defaultView && this._coreBrowserService) {
          this.element.ownerDocument.defaultView !== this._coreBrowserService.window && (this._coreBrowserService.window = this.element.ownerDocument.defaultView);
          return;
        }
        this._document = e4.ownerDocument, this.options.documentOverride && this.options.documentOverride instanceof Document && (this._document = this.optionsService.rawOptions.documentOverride), this.element = this._document.createElement("div"), this.element.dir = "ltr", this.element.classList.add("terminal"), this.element.classList.add("xterm"), e4.appendChild(this.element);
        let i12 = this._document.createDocumentFragment();
        this._viewportElement = this._document.createElement("div"), this._viewportElement.classList.add("xterm-viewport"), i12.appendChild(this._viewportElement), this.screenElement = this._document.createElement("div"), this.screenElement.classList.add("xterm-screen"), this._register(L2(this.screenElement, "mousemove", (o6) => this.updateCursorStyle(o6))), this._helperContainer = this._document.createElement("div"), this._helperContainer.classList.add("xterm-helpers"), this.screenElement.appendChild(this._helperContainer), i12.appendChild(this.screenElement);
        let r7 = this.textarea = this._document.createElement("textarea");
        this.textarea.classList.add("xterm-helper-textarea"), this.textarea.setAttribute("aria-label", mi.get()), Ts || this.textarea.setAttribute("aria-multiline", "false"), this.textarea.setAttribute("autocorrect", "off"), this.textarea.setAttribute("autocapitalize", "off"), this.textarea.setAttribute("spellcheck", "false"), this.textarea.tabIndex = 0, this._register(this.optionsService.onSpecificOptionChange("disableStdin", () => r7.readOnly = this.optionsService.rawOptions.disableStdin)), this.textarea.readOnly = this.optionsService.rawOptions.disableStdin, this._coreBrowserService = this._register(this._instantiationService.createInstance(Jr, this.textarea, e4.ownerDocument.defaultView ?? window, this._document ?? typeof window < "u" ? window.document : null)), this._instantiationService.setService(ae, this._coreBrowserService), this._register(L2(this.textarea, "focus", (o6) => this._handleTextAreaFocus(o6))), this._register(L2(this.textarea, "blur", () => this._handleTextAreaBlur())), this._helperContainer.appendChild(this.textarea), this._charSizeService = this._instantiationService.createInstance(jt, this._document, this._helperContainer), this._instantiationService.setService(nt, this._charSizeService), this._themeService = this._instantiationService.createInstance(ti), this._instantiationService.setService(Re, this._themeService), this._characterJoinerService = this._instantiationService.createInstance(ct), this._instantiationService.setService(or, this._characterJoinerService), this._renderService = this._register(this._instantiationService.createInstance(Qt, this.rows, this.screenElement)), this._instantiationService.setService(ce, this._renderService), this._register(this._renderService.onRenderedViewportChange((o6) => this._onRender.fire(o6))), this.onResize((o6) => this._renderService.resize(o6.cols, o6.rows)), this._compositionView = this._document.createElement("div"), this._compositionView.classList.add("composition-view"), this._compositionHelper = this._instantiationService.createInstance($t, this.textarea, this._compositionView), this._helperContainer.appendChild(this._compositionView), this._mouseService = this._instantiationService.createInstance(Xt), this._instantiationService.setService(Dt, this._mouseService);
        let n4 = this._linkifier.value = this._register(this._instantiationService.createInstance(hi, this.screenElement));
        this.element.appendChild(i12);
        try {
          this._onWillOpen.fire(this.element);
        } catch {
        }
        this._renderService.hasRenderer() || this._renderService.setRenderer(this._createRenderer()), this._register(this.onCursorMove(() => {
          this._renderService.handleCursorMove(), this._syncTextArea();
        })), this._register(this.onResize(() => this._renderService.handleResize(this.cols, this.rows))), this._register(this.onBlur(() => this._renderService.handleBlur())), this._register(this.onFocus(() => this._renderService.handleFocus())), this._viewport = this._register(this._instantiationService.createInstance(zt, this.element, this.screenElement)), this._register(this._viewport.onRequestScrollLines((o6) => {
          super.scrollLines(o6, false), this.refresh(0, this.rows - 1);
        })), this._selectionService = this._register(this._instantiationService.createInstance(ei, this.element, this.screenElement, n4)), this._instantiationService.setService(Qs, this._selectionService), this._register(this._selectionService.onRequestScrollLines((o6) => this.scrollLines(o6.amount, o6.suppressScrollEvent))), this._register(this._selectionService.onSelectionChange(() => this._onSelectionChange.fire())), this._register(this._selectionService.onRequestRedraw((o6) => this._renderService.handleSelectionChanged(o6.start, o6.end, o6.columnSelectMode))), this._register(this._selectionService.onLinuxMouseSelection((o6) => {
          this.textarea.value = o6, this.textarea.focus(), this.textarea.select();
        })), this._register($2.any(this._onScroll.event, this._inputHandler.onScroll)(() => {
          this._selectionService.refresh(), this._viewport?.queueSync();
        })), this._register(this._instantiationService.createInstance(Gt, this.screenElement)), this._register(L2(this.element, "mousedown", (o6) => this._selectionService.handleMouseDown(o6))), this.coreMouseService.areMouseEventsActive ? (this._selectionService.disable(), this.element.classList.add("enable-mouse-events")) : this._selectionService.enable(), this.options.screenReaderMode && (this._accessibilityManager.value = this._instantiationService.createInstance(Tt, this)), this._register(this.optionsService.onSpecificOptionChange("screenReaderMode", (o6) => this._handleScreenReaderModeOptionChange(o6))), this.options.overviewRuler.width && (this._overviewRulerRenderer = this._register(this._instantiationService.createInstance(bt, this._viewportElement, this.screenElement))), this.optionsService.onSpecificOptionChange("overviewRuler", (o6) => {
          !this._overviewRulerRenderer && o6 && this._viewportElement && this.screenElement && (this._overviewRulerRenderer = this._register(this._instantiationService.createInstance(bt, this._viewportElement, this.screenElement)));
        }), this._charSizeService.measure(), this.refresh(0, this.rows - 1), this._initGlobal(), this.bindMouse();
      }
      _createRenderer() {
        return this._instantiationService.createInstance(Yt, this, this._document, this.element, this.screenElement, this._viewportElement, this._helperContainer, this.linkifier);
      }
      bindMouse() {
        let e4 = this, i12 = this.element;
        function r7(l4) {
          let a3 = e4._mouseService.getMouseReportCoords(l4, e4.screenElement);
          if (!a3) return false;
          let u3, h4;
          switch (l4.overrideType || l4.type) {
            case "mousemove":
              h4 = 32, l4.buttons === void 0 ? (u3 = 3, l4.button !== void 0 && (u3 = l4.button < 3 ? l4.button : 3)) : u3 = l4.buttons & 1 ? 0 : l4.buttons & 4 ? 1 : l4.buttons & 2 ? 2 : 3;
              break;
            case "mouseup":
              h4 = 0, u3 = l4.button < 3 ? l4.button : 3;
              break;
            case "mousedown":
              h4 = 1, u3 = l4.button < 3 ? l4.button : 3;
              break;
            case "wheel":
              if (e4._customWheelEventHandler && e4._customWheelEventHandler(l4) === false) return false;
              let c4 = l4.deltaY;
              if (c4 === 0 || e4.coreMouseService.consumeWheelEvent(l4, e4._renderService?.dimensions?.device?.cell?.height, e4._coreBrowserService?.dpr) === 0) return false;
              h4 = c4 < 0 ? 0 : 1, u3 = 4;
              break;
            default:
              return false;
          }
          return h4 === void 0 || u3 === void 0 || u3 > 4 ? false : e4.coreMouseService.triggerMouseEvent({ col: a3.col, row: a3.row, x: a3.x, y: a3.y, button: u3, action: h4, ctrl: l4.ctrlKey, alt: l4.altKey, shift: l4.shiftKey });
        }
        let n4 = { mouseup: null, wheel: null, mousedrag: null, mousemove: null }, o6 = { mouseup: (l4) => (r7(l4), l4.buttons || (this._document.removeEventListener("mouseup", n4.mouseup), n4.mousedrag && this._document.removeEventListener("mousemove", n4.mousedrag)), this.cancel(l4)), wheel: (l4) => (r7(l4), this.cancel(l4, true)), mousedrag: (l4) => {
          l4.buttons && r7(l4);
        }, mousemove: (l4) => {
          l4.buttons || r7(l4);
        } };
        this._register(this.coreMouseService.onProtocolChange((l4) => {
          l4 ? (this.optionsService.rawOptions.logLevel === "debug" && this._logService.debug("Binding to mouse events:", this.coreMouseService.explainEvents(l4)), this.element.classList.add("enable-mouse-events"), this._selectionService.disable()) : (this._logService.debug("Unbinding from mouse events."), this.element.classList.remove("enable-mouse-events"), this._selectionService.enable()), l4 & 8 ? n4.mousemove || (i12.addEventListener("mousemove", o6.mousemove), n4.mousemove = o6.mousemove) : (i12.removeEventListener("mousemove", n4.mousemove), n4.mousemove = null), l4 & 16 ? n4.wheel || (i12.addEventListener("wheel", o6.wheel, { passive: false }), n4.wheel = o6.wheel) : (i12.removeEventListener("wheel", n4.wheel), n4.wheel = null), l4 & 2 ? n4.mouseup || (n4.mouseup = o6.mouseup) : (this._document.removeEventListener("mouseup", n4.mouseup), n4.mouseup = null), l4 & 4 ? n4.mousedrag || (n4.mousedrag = o6.mousedrag) : (this._document.removeEventListener("mousemove", n4.mousedrag), n4.mousedrag = null);
        })), this.coreMouseService.activeProtocol = this.coreMouseService.activeProtocol, this._register(L2(i12, "mousedown", (l4) => {
          if (l4.preventDefault(), this.focus(), !(!this.coreMouseService.areMouseEventsActive || this._selectionService.shouldForceSelection(l4))) return r7(l4), n4.mouseup && this._document.addEventListener("mouseup", n4.mouseup), n4.mousedrag && this._document.addEventListener("mousemove", n4.mousedrag), this.cancel(l4);
        })), this._register(L2(i12, "wheel", (l4) => {
          if (!n4.wheel) {
            if (this._customWheelEventHandler && this._customWheelEventHandler(l4) === false) return false;
            if (!this.buffer.hasScrollback) {
              if (l4.deltaY === 0) return false;
              if (e4.coreMouseService.consumeWheelEvent(l4, e4._renderService?.dimensions?.device?.cell?.height, e4._coreBrowserService?.dpr) === 0) return this.cancel(l4, true);
              let h4 = b3.ESC + (this.coreService.decPrivateModes.applicationCursorKeys ? "O" : "[") + (l4.deltaY < 0 ? "A" : "B");
              return this.coreService.triggerDataEvent(h4, true), this.cancel(l4, true);
            }
          }
        }, { passive: false }));
      }
      refresh(e4, i12) {
        this._renderService?.refreshRows(e4, i12);
      }
      updateCursorStyle(e4) {
        this._selectionService?.shouldColumnSelect(e4) ? this.element.classList.add("column-select") : this.element.classList.remove("column-select");
      }
      _showCursor() {
        this.coreService.isCursorInitialized || (this.coreService.isCursorInitialized = true, this.refresh(this.buffer.y, this.buffer.y));
      }
      scrollLines(e4, i12) {
        this._viewport ? this._viewport.scrollLines(e4) : super.scrollLines(e4, i12), this.refresh(0, this.rows - 1);
      }
      scrollPages(e4) {
        this.scrollLines(e4 * (this.rows - 1));
      }
      scrollToTop() {
        this.scrollLines(-this._bufferService.buffer.ydisp);
      }
      scrollToBottom(e4) {
        e4 && this._viewport ? this._viewport.scrollToLine(this.buffer.ybase, true) : this.scrollLines(this._bufferService.buffer.ybase - this._bufferService.buffer.ydisp);
      }
      scrollToLine(e4) {
        let i12 = e4 - this._bufferService.buffer.ydisp;
        i12 !== 0 && this.scrollLines(i12);
      }
      paste(e4) {
        Cn(e4, this.textarea, this.coreService, this.optionsService);
      }
      attachCustomKeyEventHandler(e4) {
        this._customKeyEventHandler = e4;
      }
      attachCustomWheelEventHandler(e4) {
        this._customWheelEventHandler = e4;
      }
      registerLinkProvider(e4) {
        return this._linkProviderService.registerLinkProvider(e4);
      }
      registerCharacterJoiner(e4) {
        if (!this._characterJoinerService) throw new Error("Terminal must be opened first");
        let i12 = this._characterJoinerService.register(e4);
        return this.refresh(0, this.rows - 1), i12;
      }
      deregisterCharacterJoiner(e4) {
        if (!this._characterJoinerService) throw new Error("Terminal must be opened first");
        this._characterJoinerService.deregister(e4) && this.refresh(0, this.rows - 1);
      }
      get markers() {
        return this.buffer.markers;
      }
      registerMarker(e4) {
        return this.buffer.addMarker(this.buffer.ybase + this.buffer.y + e4);
      }
      registerDecoration(e4) {
        return this._decorationService.registerDecoration(e4);
      }
      hasSelection() {
        return this._selectionService ? this._selectionService.hasSelection : false;
      }
      select(e4, i12, r7) {
        this._selectionService.setSelection(e4, i12, r7);
      }
      getSelection() {
        return this._selectionService ? this._selectionService.selectionText : "";
      }
      getSelectionPosition() {
        if (!(!this._selectionService || !this._selectionService.hasSelection)) return { start: { x: this._selectionService.selectionStart[0], y: this._selectionService.selectionStart[1] }, end: { x: this._selectionService.selectionEnd[0], y: this._selectionService.selectionEnd[1] } };
      }
      clearSelection() {
        this._selectionService?.clearSelection();
      }
      selectAll() {
        this._selectionService?.selectAll();
      }
      selectLines(e4, i12) {
        this._selectionService?.selectLines(e4, i12);
      }
      _keyDown(e4) {
        if (this._keyDownHandled = false, this._keyDownSeen = true, this._customKeyEventHandler && this._customKeyEventHandler(e4) === false) return false;
        let i12 = this.browser.isMac && this.options.macOptionIsMeta && e4.altKey;
        if (!i12 && !this._compositionHelper.keydown(e4)) return this.options.scrollOnUserInput && this.buffer.ybase !== this.buffer.ydisp && this.scrollToBottom(true), false;
        !i12 && (e4.key === "Dead" || e4.key === "AltGraph") && (this._unprocessedDeadKey = true);
        let r7 = Il(e4, this.coreService.decPrivateModes.applicationCursorKeys, this.browser.isMac, this.options.macOptionIsMeta);
        if (this.updateCursorStyle(e4), r7.type === 3 || r7.type === 2) {
          let n4 = this.rows - 1;
          return this.scrollLines(r7.type === 2 ? -n4 : n4), this.cancel(e4, true);
        }
        if (r7.type === 1 && this.selectAll(), this._isThirdLevelShift(this.browser, e4) || (r7.cancel && this.cancel(e4, true), !r7.key) || e4.key && !e4.ctrlKey && !e4.altKey && !e4.metaKey && e4.key.length === 1 && e4.key.charCodeAt(0) >= 65 && e4.key.charCodeAt(0) <= 90) return true;
        if (this._unprocessedDeadKey) return this._unprocessedDeadKey = false, true;
        if ((r7.key === b3.ETX || r7.key === b3.CR) && (this.textarea.value = ""), this._onKey.fire({ key: r7.key, domEvent: e4 }), this._showCursor(), this.coreService.triggerDataEvent(r7.key, true), !this.optionsService.rawOptions.screenReaderMode || e4.altKey || e4.ctrlKey) return this.cancel(e4, true);
        this._keyDownHandled = true;
      }
      _isThirdLevelShift(e4, i12) {
        let r7 = e4.isMac && !this.options.macOptionIsMeta && i12.altKey && !i12.ctrlKey && !i12.metaKey || e4.isWindows && i12.altKey && i12.ctrlKey && !i12.metaKey || e4.isWindows && i12.getModifierState("AltGraph");
        return i12.type === "keypress" ? r7 : r7 && (!i12.keyCode || i12.keyCode > 47);
      }
      _keyUp(e4) {
        this._keyDownSeen = false, !(this._customKeyEventHandler && this._customKeyEventHandler(e4) === false) && (Tc(e4) || this.focus(), this.updateCursorStyle(e4), this._keyPressHandled = false);
      }
      _keyPress(e4) {
        let i12;
        if (this._keyPressHandled = false, this._keyDownHandled || this._customKeyEventHandler && this._customKeyEventHandler(e4) === false) return false;
        if (this.cancel(e4), e4.charCode) i12 = e4.charCode;
        else if (e4.which === null || e4.which === void 0) i12 = e4.keyCode;
        else if (e4.which !== 0 && e4.charCode !== 0) i12 = e4.which;
        else return false;
        return !i12 || (e4.altKey || e4.ctrlKey || e4.metaKey) && !this._isThirdLevelShift(this.browser, e4) ? false : (i12 = String.fromCharCode(i12), this._onKey.fire({ key: i12, domEvent: e4 }), this._showCursor(), this.coreService.triggerDataEvent(i12, true), this._keyPressHandled = true, this._unprocessedDeadKey = false, true);
      }
      _inputEvent(e4) {
        if (e4.data && e4.inputType === "insertText" && (!e4.composed || !this._keyDownSeen) && !this.optionsService.rawOptions.screenReaderMode) {
          if (this._keyPressHandled) return false;
          this._unprocessedDeadKey = false;
          let i12 = e4.data;
          return this.coreService.triggerDataEvent(i12, true), this.cancel(e4), true;
        }
        return false;
      }
      resize(e4, i12) {
        if (e4 === this.cols && i12 === this.rows) {
          this._charSizeService && !this._charSizeService.hasValidSize && this._charSizeService.measure();
          return;
        }
        super.resize(e4, i12);
      }
      _afterResize(e4, i12) {
        this._charSizeService?.measure();
      }
      clear() {
        if (!(this.buffer.ybase === 0 && this.buffer.y === 0)) {
          this.buffer.clearAllMarkers(), this.buffer.lines.set(0, this.buffer.lines.get(this.buffer.ybase + this.buffer.y)), this.buffer.lines.length = 1, this.buffer.ydisp = 0, this.buffer.ybase = 0, this.buffer.y = 0;
          for (let e4 = 1; e4 < this.rows; e4++) this.buffer.lines.push(this.buffer.getBlankLine(X));
          this._onScroll.fire({ position: this.buffer.ydisp }), this.refresh(0, this.rows - 1);
        }
      }
      reset() {
        this.options.rows = this.rows, this.options.cols = this.cols;
        let e4 = this._customKeyEventHandler;
        this._setup(), super.reset(), this._selectionService?.reset(), this._decorationService.reset(), this._customKeyEventHandler = e4, this.refresh(0, this.rows - 1);
      }
      clearTextureAtlas() {
        this._renderService?.clearTextureAtlas();
      }
      _reportFocus() {
        this.element?.classList.contains("focus") ? this.coreService.triggerDataEvent(b3.ESC + "[I") : this.coreService.triggerDataEvent(b3.ESC + "[O");
      }
      _reportWindowsOptions(e4) {
        if (this._renderService) switch (e4) {
          case 0:
            let i12 = this._renderService.dimensions.css.canvas.width.toFixed(0), r7 = this._renderService.dimensions.css.canvas.height.toFixed(0);
            this.coreService.triggerDataEvent(`${b3.ESC}[4;${r7};${i12}t`);
            break;
          case 1:
            let n4 = this._renderService.dimensions.css.cell.width.toFixed(0), o6 = this._renderService.dimensions.css.cell.height.toFixed(0);
            this.coreService.triggerDataEvent(`${b3.ESC}[6;${o6};${n4}t`);
            break;
        }
      }
      cancel(e4, i12) {
        if (!(!this.options.cancelEvents && !i12)) return e4.preventDefault(), e4.stopPropagation(), false;
      }
    };
    xn = class {
      constructor() {
        this._addons = [];
      }
      dispose() {
        for (let t3 = this._addons.length - 1; t3 >= 0; t3--) this._addons[t3].instance.dispose();
      }
      loadAddon(t3, e4) {
        let i12 = { instance: e4, dispose: e4.dispose, isDisposed: false };
        this._addons.push(i12), e4.dispose = () => this._wrappedAddonDispose(i12), e4.activate(t3);
      }
      _wrappedAddonDispose(t3) {
        if (t3.isDisposed) return;
        let e4 = -1;
        for (let i12 = 0; i12 < this._addons.length; i12++) if (this._addons[i12] === t3) {
          e4 = i12;
          break;
        }
        if (e4 === -1) throw new Error("Could not dispose an addon that has not been loaded");
        t3.isDisposed = true, t3.dispose.apply(t3.instance), this._addons.splice(e4, 1);
      }
    };
    wn = class {
      constructor(t3) {
        this._line = t3;
      }
      get isWrapped() {
        return this._line.isWrapped;
      }
      get length() {
        return this._line.length;
      }
      getCell(t3, e4) {
        if (!(t3 < 0 || t3 >= this._line.length)) return e4 ? (this._line.loadCell(t3, e4), e4) : this._line.loadCell(t3, new q());
      }
      translateToString(t3, e4, i12) {
        return this._line.translateToString(t3, e4, i12);
      }
    };
    Ji = class {
      constructor(t3, e4) {
        this._buffer = t3;
        this.type = e4;
      }
      init(t3) {
        return this._buffer = t3, this;
      }
      get cursorY() {
        return this._buffer.y;
      }
      get cursorX() {
        return this._buffer.x;
      }
      get viewportY() {
        return this._buffer.ydisp;
      }
      get baseY() {
        return this._buffer.ybase;
      }
      get length() {
        return this._buffer.lines.length;
      }
      getLine(t3) {
        let e4 = this._buffer.lines.get(t3);
        if (e4) return new wn(e4);
      }
      getNullCell() {
        return new q();
      }
    };
    Dn = class extends D2 {
      constructor(e4) {
        super();
        this._core = e4;
        this._onBufferChange = this._register(new v2());
        this.onBufferChange = this._onBufferChange.event;
        this._normal = new Ji(this._core.buffers.normal, "normal"), this._alternate = new Ji(this._core.buffers.alt, "alternate"), this._core.buffers.onBufferActivate(() => this._onBufferChange.fire(this.active));
      }
      get active() {
        if (this._core.buffers.active === this._core.buffers.normal) return this.normal;
        if (this._core.buffers.active === this._core.buffers.alt) return this.alternate;
        throw new Error("Active buffer is neither normal nor alternate");
      }
      get normal() {
        return this._normal.init(this._core.buffers.normal);
      }
      get alternate() {
        return this._alternate.init(this._core.buffers.alt);
      }
    };
    Rn = class {
      constructor(t3) {
        this._core = t3;
      }
      registerCsiHandler(t3, e4) {
        return this._core.registerCsiHandler(t3, (i12) => e4(i12.toArray()));
      }
      addCsiHandler(t3, e4) {
        return this.registerCsiHandler(t3, e4);
      }
      registerDcsHandler(t3, e4) {
        return this._core.registerDcsHandler(t3, (i12, r7) => e4(i12, r7.toArray()));
      }
      addDcsHandler(t3, e4) {
        return this.registerDcsHandler(t3, e4);
      }
      registerEscHandler(t3, e4) {
        return this._core.registerEscHandler(t3, e4);
      }
      addEscHandler(t3, e4) {
        return this.registerEscHandler(t3, e4);
      }
      registerOscHandler(t3, e4) {
        return this._core.registerOscHandler(t3, e4);
      }
      addOscHandler(t3, e4) {
        return this.registerOscHandler(t3, e4);
      }
    };
    Ln = class {
      constructor(t3) {
        this._core = t3;
      }
      register(t3) {
        this._core.unicodeService.register(t3);
      }
      get versions() {
        return this._core.unicodeService.versions;
      }
      get activeVersion() {
        return this._core.unicodeService.activeVersion;
      }
      set activeVersion(t3) {
        this._core.unicodeService.activeVersion = t3;
      }
    };
    Ic = ["cols", "rows"];
    Ue = 0;
    Dl = class extends D2 {
      constructor(t3) {
        super(), this._core = this._register(new yn(t3)), this._addonManager = this._register(new xn()), this._publicOptions = { ...this._core.options };
        let e4 = (r7) => this._core.options[r7], i12 = (r7, n4) => {
          this._checkReadonlyOptions(r7), this._core.options[r7] = n4;
        };
        for (let r7 in this._core.options) {
          let n4 = { get: e4.bind(this, r7), set: i12.bind(this, r7) };
          Object.defineProperty(this._publicOptions, r7, n4);
        }
      }
      _checkReadonlyOptions(t3) {
        if (Ic.includes(t3)) throw new Error(`Option "${t3}" can only be set in the constructor`);
      }
      _checkProposedApi() {
        if (!this._core.optionsService.rawOptions.allowProposedApi) throw new Error("You must set the allowProposedApi option to true to use proposed API");
      }
      get onBell() {
        return this._core.onBell;
      }
      get onBinary() {
        return this._core.onBinary;
      }
      get onCursorMove() {
        return this._core.onCursorMove;
      }
      get onData() {
        return this._core.onData;
      }
      get onKey() {
        return this._core.onKey;
      }
      get onLineFeed() {
        return this._core.onLineFeed;
      }
      get onRender() {
        return this._core.onRender;
      }
      get onResize() {
        return this._core.onResize;
      }
      get onScroll() {
        return this._core.onScroll;
      }
      get onSelectionChange() {
        return this._core.onSelectionChange;
      }
      get onTitleChange() {
        return this._core.onTitleChange;
      }
      get onWriteParsed() {
        return this._core.onWriteParsed;
      }
      get element() {
        return this._core.element;
      }
      get parser() {
        return this._parser || (this._parser = new Rn(this._core)), this._parser;
      }
      get unicode() {
        return this._checkProposedApi(), new Ln(this._core);
      }
      get textarea() {
        return this._core.textarea;
      }
      get rows() {
        return this._core.rows;
      }
      get cols() {
        return this._core.cols;
      }
      get buffer() {
        return this._buffer || (this._buffer = this._register(new Dn(this._core))), this._buffer;
      }
      get markers() {
        return this._checkProposedApi(), this._core.markers;
      }
      get modes() {
        let t3 = this._core.coreService.decPrivateModes, e4 = "none";
        switch (this._core.coreMouseService.activeProtocol) {
          case "X10":
            e4 = "x10";
            break;
          case "VT200":
            e4 = "vt200";
            break;
          case "DRAG":
            e4 = "drag";
            break;
          case "ANY":
            e4 = "any";
            break;
        }
        return { applicationCursorKeysMode: t3.applicationCursorKeys, applicationKeypadMode: t3.applicationKeypad, bracketedPasteMode: t3.bracketedPasteMode, insertMode: this._core.coreService.modes.insertMode, mouseTrackingMode: e4, originMode: t3.origin, reverseWraparoundMode: t3.reverseWraparound, sendFocusMode: t3.sendFocus, synchronizedOutputMode: t3.synchronizedOutput, wraparoundMode: t3.wraparound };
      }
      get options() {
        return this._publicOptions;
      }
      set options(t3) {
        for (let e4 in t3) this._publicOptions[e4] = t3[e4];
      }
      blur() {
        this._core.blur();
      }
      focus() {
        this._core.focus();
      }
      input(t3, e4 = true) {
        this._core.input(t3, e4);
      }
      resize(t3, e4) {
        this._verifyIntegers(t3, e4), this._core.resize(t3, e4);
      }
      open(t3) {
        this._core.open(t3);
      }
      attachCustomKeyEventHandler(t3) {
        this._core.attachCustomKeyEventHandler(t3);
      }
      attachCustomWheelEventHandler(t3) {
        this._core.attachCustomWheelEventHandler(t3);
      }
      registerLinkProvider(t3) {
        return this._core.registerLinkProvider(t3);
      }
      registerCharacterJoiner(t3) {
        return this._checkProposedApi(), this._core.registerCharacterJoiner(t3);
      }
      deregisterCharacterJoiner(t3) {
        this._checkProposedApi(), this._core.deregisterCharacterJoiner(t3);
      }
      registerMarker(t3 = 0) {
        return this._verifyIntegers(t3), this._core.registerMarker(t3);
      }
      registerDecoration(t3) {
        return this._checkProposedApi(), this._verifyPositiveIntegers(t3.x ?? 0, t3.width ?? 0, t3.height ?? 0), this._core.registerDecoration(t3);
      }
      hasSelection() {
        return this._core.hasSelection();
      }
      select(t3, e4, i12) {
        this._verifyIntegers(t3, e4, i12), this._core.select(t3, e4, i12);
      }
      getSelection() {
        return this._core.getSelection();
      }
      getSelectionPosition() {
        return this._core.getSelectionPosition();
      }
      clearSelection() {
        this._core.clearSelection();
      }
      selectAll() {
        this._core.selectAll();
      }
      selectLines(t3, e4) {
        this._verifyIntegers(t3, e4), this._core.selectLines(t3, e4);
      }
      dispose() {
        super.dispose();
      }
      scrollLines(t3) {
        this._verifyIntegers(t3), this._core.scrollLines(t3);
      }
      scrollPages(t3) {
        this._verifyIntegers(t3), this._core.scrollPages(t3);
      }
      scrollToTop() {
        this._core.scrollToTop();
      }
      scrollToBottom() {
        this._core.scrollToBottom();
      }
      scrollToLine(t3) {
        this._verifyIntegers(t3), this._core.scrollToLine(t3);
      }
      clear() {
        this._core.clear();
      }
      write(t3, e4) {
        this._core.write(t3, e4);
      }
      writeln(t3, e4) {
        this._core.write(t3), this._core.write(`\r
`, e4);
      }
      paste(t3) {
        this._core.paste(t3);
      }
      refresh(t3, e4) {
        this._verifyIntegers(t3, e4), this._core.refresh(t3, e4);
      }
      reset() {
        this._core.reset();
      }
      clearTextureAtlas() {
        this._core.clearTextureAtlas();
      }
      loadAddon(t3) {
        this._addonManager.loadAddon(this, t3);
      }
      static get strings() {
        return { get promptLabel() {
          return mi.get();
        }, set promptLabel(t3) {
          mi.set(t3);
        }, get tooMuchOutput() {
          return _i.get();
        }, set tooMuchOutput(t3) {
          _i.set(t3);
        } };
      }
      _verifyIntegers(...t3) {
        for (Ue of t3) if (Ue === 1 / 0 || isNaN(Ue) || Ue % 1 !== 0) throw new Error("This API only accepts integers");
      }
      _verifyPositiveIntegers(...t3) {
        for (Ue of t3) if (Ue && (Ue === 1 / 0 || isNaN(Ue) || Ue % 1 !== 0 || Ue < 0)) throw new Error("This API only accepts positive integers");
      }
    };
  }
});

// node_modules/.pnpm/@xterm+addon-fit@0.11.0/node_modules/@xterm/addon-fit/lib/addon-fit.mjs
var addon_fit_exports = {};
__export(addon_fit_exports, {
  FitAddon: () => o5
});
var h3, _2, o5;
var init_addon_fit = __esm({
  "node_modules/.pnpm/@xterm+addon-fit@0.11.0/node_modules/@xterm/addon-fit/lib/addon-fit.mjs"() {
    h3 = 2;
    _2 = 1;
    o5 = class {
      activate(e4) {
        this._terminal = e4;
      }
      dispose() {
      }
      fit() {
        let e4 = this.proposeDimensions();
        if (!e4 || !this._terminal || isNaN(e4.cols) || isNaN(e4.rows)) return;
        let t3 = this._terminal._core;
        (this._terminal.rows !== e4.rows || this._terminal.cols !== e4.cols) && (t3._renderService.clear(), this._terminal.resize(e4.cols, e4.rows));
      }
      proposeDimensions() {
        if (!this._terminal || !this._terminal.element || !this._terminal.element.parentElement) return;
        let t3 = this._terminal._core._renderService.dimensions;
        if (t3.css.cell.width === 0 || t3.css.cell.height === 0) return;
        let s18 = this._terminal.options.scrollback === 0 ? 0 : this._terminal.options.overviewRuler?.width || 14, r7 = window.getComputedStyle(this._terminal.element.parentElement), l4 = parseInt(r7.getPropertyValue("height")), a3 = Math.max(0, parseInt(r7.getPropertyValue("width"))), i12 = window.getComputedStyle(this._terminal.element), n4 = { top: parseInt(i12.getPropertyValue("padding-top")), bottom: parseInt(i12.getPropertyValue("padding-bottom")), right: parseInt(i12.getPropertyValue("padding-right")), left: parseInt(i12.getPropertyValue("padding-left")) }, m2 = n4.top + n4.bottom, d3 = n4.right + n4.left, c4 = l4 - m2, p3 = a3 - d3 - s18;
        return { cols: Math.max(h3, Math.floor(p3 / t3.css.cell.width)), rows: Math.max(_2, Math.floor(c4 / t3.css.cell.height)) };
      }
    };
  }
});

// node_modules/.pnpm/@xterm+addon-web-links@0.12.0/node_modules/@xterm/addon-web-links/lib/addon-web-links.mjs
var addon_web_links_exports = {};
__export(addon_web_links_exports, {
  WebLinksAddon: () => L3
});
function k2(l4) {
  try {
    let e4 = new URL(l4), t3 = e4.password && e4.username ? `${e4.protocol}//${e4.username}:${e4.password}@${e4.host}` : e4.username ? `${e4.protocol}//${e4.username}@${e4.host}` : `${e4.protocol}//${e4.host}`;
    return l4.toLocaleLowerCase().startsWith(t3.toLocaleLowerCase());
  } catch {
    return false;
  }
}
function w2(l4, e4) {
  let t3 = window.open();
  if (t3) {
    try {
      t3.opener = null;
    } catch {
    }
    t3.location.href = e4;
  } else console.warn("Opening link blocked as opener could not be cleared");
}
var v3, g2, _3, L3;
var init_addon_web_links = __esm({
  "node_modules/.pnpm/@xterm+addon-web-links@0.12.0/node_modules/@xterm/addon-web-links/lib/addon-web-links.mjs"() {
    v3 = class {
      constructor(e4, t3, n4, o6 = {}) {
        this._terminal = e4;
        this._regex = t3;
        this._handler = n4;
        this._options = o6;
      }
      provideLinks(e4, t3) {
        let n4 = g2.computeLink(e4, this._regex, this._terminal, this._handler);
        t3(this._addCallbacks(n4));
      }
      _addCallbacks(e4) {
        return e4.map((t3) => (t3.leave = this._options.leave, t3.hover = (n4, o6) => {
          if (this._options.hover) {
            let { range: p3 } = t3;
            this._options.hover(n4, o6, p3);
          }
        }, t3));
      }
    };
    g2 = class l3 {
      static computeLink(e4, t3, n4, o6) {
        let p3 = new RegExp(t3.source, (t3.flags || "") + "g"), [i12, r7] = l3._getWindowedLineStrings(e4 - 1, n4), s18 = i12.join(""), a3, d3 = [];
        for (; a3 = p3.exec(s18); ) {
          let u3 = a3[0];
          if (!k2(u3)) continue;
          let [c4, h4] = l3._mapStrIdx(n4, r7, 0, a3.index), [m2, f3] = l3._mapStrIdx(n4, c4, h4, u3.length);
          if (c4 === -1 || h4 === -1 || m2 === -1 || f3 === -1) continue;
          let b4 = { start: { x: h4 + 1, y: c4 + 1 }, end: { x: f3, y: m2 + 1 } };
          d3.push({ range: b4, text: u3, activate: o6 });
        }
        return d3;
      }
      static _getWindowedLineStrings(e4, t3) {
        let n4, o6 = e4, p3 = e4, i12 = 0, r7 = "", s18 = [];
        if (n4 = t3.buffer.active.getLine(e4)) {
          let a3 = n4.translateToString(true);
          if (n4.isWrapped && a3[0] !== " ") {
            for (i12 = 0; (n4 = t3.buffer.active.getLine(--o6)) && i12 < 2048 && (r7 = n4.translateToString(true), i12 += r7.length, s18.push(r7), !(!n4.isWrapped || r7.indexOf(" ") !== -1)); ) ;
            s18.reverse();
          }
          for (s18.push(a3), i12 = 0; (n4 = t3.buffer.active.getLine(++p3)) && n4.isWrapped && i12 < 2048 && (r7 = n4.translateToString(true), i12 += r7.length, s18.push(r7), r7.indexOf(" ") === -1); ) ;
        }
        return [s18, o6];
      }
      static _mapStrIdx(e4, t3, n4, o6) {
        let p3 = e4.buffer.active, i12 = p3.getNullCell(), r7 = n4;
        for (; o6; ) {
          let s18 = p3.getLine(t3);
          if (!s18) return [-1, -1];
          for (let a3 = r7; a3 < s18.length; ++a3) {
            s18.getCell(a3, i12);
            let d3 = i12.getChars();
            if (i12.getWidth() && (o6 -= d3.length || 1, a3 === s18.length - 1 && d3 === "")) {
              let c4 = p3.getLine(t3 + 1);
              c4 && c4.isWrapped && (c4.getCell(0, i12), i12.getWidth() === 2 && (o6 += 1));
            }
            if (o6 < 0) return [t3, a3];
          }
          t3++, r7 = 0;
        }
        return [t3, r7];
      }
    };
    _3 = /(https?|HTTPS?):[/]{2}[^\s"'!*(){}|\\\^<>`]*[^\s"':,.!?{}|\\\^~\[\]`()<>]/;
    L3 = class {
      constructor(e4 = w2, t3 = {}) {
        this._handler = e4;
        this._options = t3;
      }
      activate(e4) {
        this._terminal = e4;
        let t3 = this._options, n4 = t3.urlRegex || _3;
        this._linkProvider = this._terminal.registerLinkProvider(new v3(this._terminal, n4, this._handler, t3));
      }
      dispose() {
        this._linkProvider?.dispose();
      }
    };
  }
});

// node_modules/.pnpm/@xterm+addon-search@0.16.0/node_modules/@xterm/addon-search/lib/addon-search.mjs
var addon_search_exports = {};
__export(addon_search_exports, {
  SearchAddon: () => ut2
});
function le(r7) {
  ft2(r7) || pt3.onUnexpectedError(r7);
}
function ft2(r7) {
  return r7 instanceof ee2 ? true : r7 instanceof Error && r7.name === Ce2 && r7.message === Ce2;
}
function mt2(r7, e4, t3 = 0, n4 = r7.length) {
  let i12 = t3, s18 = n4;
  for (; i12 < s18; ) {
    let a3 = Math.floor((i12 + s18) / 2);
    e4(r7[a3]) ? i12 = a3 + 1 : s18 = a3;
  }
  return i12 - 1;
}
function $e2(r7, e4) {
  return (t3, n4) => e4(r7(t3), r7(n4));
}
function Xe2(r7, e4) {
  let t3 = /* @__PURE__ */ Object.create(null);
  for (let n4 of r7) {
    let i12 = e4(n4), s18 = t3[i12];
    s18 || (s18 = t3[i12] = []), s18.push(n4);
  }
  return t3;
}
function Pe(r7, e4) {
  let t3 = this, n4 = false, i12;
  return function() {
    if (n4) return i12;
    if (n4 = true, e4) try {
      i12 = r7.apply(t3, arguments);
    } finally {
      e4();
    }
    else i12 = r7.apply(t3, arguments);
    return i12;
  };
}
function vt2(r7) {
  K2 = r7;
}
function pe2(r7) {
  return K2?.trackDisposable(r7), r7;
}
function fe2(r7) {
  K2?.markAsDisposed(r7);
}
function te(r7, e4) {
  K2?.setParent(r7, e4);
}
function bt2(r7, e4) {
  if (K2) for (let t3 of r7) K2.setParent(t3, e4);
}
function Q3(r7) {
  if (Le2.is(r7)) {
    let e4 = [];
    for (let t3 of r7) if (t3) try {
      t3.dispose();
    } catch (n4) {
      e4.push(n4);
    }
    if (e4.length === 1) throw e4[0];
    if (e4.length > 1) throw new AggregateError(e4, "Encountered errors while disposing of store");
    return Array.isArray(r7) ? [] : r7;
  } else if (r7) return r7.dispose(), r7;
}
function me(...r7) {
  let e4 = A2(() => Q3(r7));
  return bt2(r7, e4), e4;
}
function A2(r7) {
  let e4 = pe2({ dispose: Pe(() => {
    fe2(e4), r7();
  }) });
  return e4;
}
function xe2(r7, e4 = 0, t3) {
  let n4 = setTimeout(() => {
    r7(), t3 && i12.dispose();
  }, e4), i12 = A2(() => {
    clearTimeout(n4), t3?.deleteAndLeak(i12);
  });
  return t3?.add(i12), i12;
}
var Re2, pt3, Ce2, ee2, ae2, ue2, Qe, Be2, V2, Ye2, Je2, Ge, ce2, Le2, Tt2, K2, de2, he2, H3, k3, F2, $3, _t2, Te2, gt2, tt, yt, ie2, B3, Oe, nt2, be2, Ae2, ne2, Me, Fe2, xt2, G2, It2, Dt2, ve2, C3, Ne2, it, Et, We2, Y2, qe2, ze, ge2, wt2, kt3, st2, St2, Rt2, Ct2, Pt2, _e3, ye2, rt2, Lt2, N2, W, O, ot2, Ot2, je2, At3, gn2, M3, j2, Mt3, Ft, at2, Nt2, yn2, xn2, In2, Dn2, Wt2, jt2, Ue2, qt, P2, Ie2, De2, Ee2, we2, ke2, ut2;
var init_addon_search = __esm({
  "node_modules/.pnpm/@xterm+addon-search@0.16.0/node_modules/@xterm/addon-search/lib/addon-search.mjs"() {
    Re2 = class {
      constructor() {
        this.listeners = [], this.unexpectedErrorHandler = function(e4) {
          setTimeout(() => {
            throw e4.stack ? ae2.isErrorNoTelemetry(e4) ? new ae2(e4.message + `

` + e4.stack) : new Error(e4.message + `

` + e4.stack) : e4;
          }, 0);
        };
      }
      addListener(e4) {
        return this.listeners.push(e4), () => {
          this._removeListener(e4);
        };
      }
      emit(e4) {
        this.listeners.forEach((t3) => {
          t3(e4);
        });
      }
      _removeListener(e4) {
        this.listeners.splice(this.listeners.indexOf(e4), 1);
      }
      setUnexpectedErrorHandler(e4) {
        this.unexpectedErrorHandler = e4;
      }
      getUnexpectedErrorHandler() {
        return this.unexpectedErrorHandler;
      }
      onUnexpectedError(e4) {
        this.unexpectedErrorHandler(e4), this.emit(e4);
      }
      onUnexpectedExternalError(e4) {
        this.unexpectedErrorHandler(e4);
      }
    };
    pt3 = new Re2();
    Ce2 = "Canceled";
    ee2 = class extends Error {
      constructor() {
        super(Ce2), this.name = this.message;
      }
    };
    ae2 = class r4 extends Error {
      constructor(e4) {
        super(e4), this.name = "CodeExpectedError";
      }
      static fromError(e4) {
        if (e4 instanceof r4) return e4;
        let t3 = new r4();
        return t3.message = e4.message, t3.stack = e4.stack, t3;
      }
      static isErrorNoTelemetry(e4) {
        return e4.name === "CodeExpectedError";
      }
    };
    ue2 = class ue3 {
      constructor(e4) {
        this._array = e4;
        this._findLastMonotonousLastIdx = 0;
      }
      findLastMonotonous(e4) {
        if (ue3.assertInvariants) {
          if (this._prevFindLastPredicate) {
            for (let n4 of this._array) if (this._prevFindLastPredicate(n4) && !e4(n4)) throw new Error("MonotonousArray: current predicate must be weaker than (or equal to) the previous predicate.");
          }
          this._prevFindLastPredicate = e4;
        }
        let t3 = mt2(this._array, e4, this._findLastMonotonousLastIdx);
        return this._findLastMonotonousLastIdx = t3 + 1, t3 === -1 ? void 0 : this._array[t3];
      }
    };
    ue2.assertInvariants = false;
    ((h4) => {
      function r7(u3) {
        return u3 < 0;
      }
      h4.isLessThan = r7;
      function e4(u3) {
        return u3 <= 0;
      }
      h4.isLessThanOrEqual = e4;
      function t3(u3) {
        return u3 > 0;
      }
      h4.isGreaterThan = t3;
      function n4(u3) {
        return u3 === 0;
      }
      h4.isNeitherLessOrGreaterThan = n4, h4.greaterThan = 1, h4.lessThan = -1, h4.neitherLessOrGreaterThan = 0;
    })(Qe || (Qe = {}));
    Be2 = (r7, e4) => r7 - e4;
    V2 = class V3 {
      constructor(e4) {
        this.iterate = e4;
      }
      forEach(e4) {
        this.iterate((t3) => (e4(t3), true));
      }
      toArray() {
        let e4 = [];
        return this.iterate((t3) => (e4.push(t3), true)), e4;
      }
      filter(e4) {
        return new V3((t3) => this.iterate((n4) => e4(n4) ? t3(n4) : true));
      }
      map(e4) {
        return new V3((t3) => this.iterate((n4) => t3(e4(n4))));
      }
      some(e4) {
        let t3 = false;
        return this.iterate((n4) => (t3 = e4(n4), !t3)), t3;
      }
      findFirst(e4) {
        let t3;
        return this.iterate((n4) => e4(n4) ? (t3 = n4, false) : true), t3;
      }
      findLast(e4) {
        let t3;
        return this.iterate((n4) => (e4(n4) && (t3 = n4), true)), t3;
      }
      findLastMaxBy(e4) {
        let t3, n4 = true;
        return this.iterate((i12) => ((n4 || Qe.isGreaterThan(e4(i12, t3))) && (n4 = false, t3 = i12), true)), t3;
      }
    };
    V2.empty = new V2((e4) => {
    });
    Ge = class {
      constructor(e4, t3) {
        this.toKey = t3;
        this._map = /* @__PURE__ */ new Map();
        this[Ye2] = "SetWithKey";
        for (let n4 of e4) this.add(n4);
      }
      get size() {
        return this._map.size;
      }
      add(e4) {
        let t3 = this.toKey(e4);
        return this._map.set(t3, e4), this;
      }
      delete(e4) {
        return this._map.delete(this.toKey(e4));
      }
      has(e4) {
        return this._map.has(this.toKey(e4));
      }
      *entries() {
        for (let e4 of this._map.values()) yield [e4, e4];
      }
      keys() {
        return this.values();
      }
      *values() {
        for (let e4 of this._map.values()) yield e4;
      }
      clear() {
        this._map.clear();
      }
      forEach(e4, t3) {
        this._map.forEach((n4) => e4.call(t3, n4, n4, this));
      }
      [(Je2 = Symbol.iterator, Ye2 = Symbol.toStringTag, Je2)]() {
        return this.values();
      }
    };
    ce2 = class {
      constructor() {
        this.map = /* @__PURE__ */ new Map();
      }
      add(e4, t3) {
        let n4 = this.map.get(e4);
        n4 || (n4 = /* @__PURE__ */ new Set(), this.map.set(e4, n4)), n4.add(t3);
      }
      delete(e4, t3) {
        let n4 = this.map.get(e4);
        n4 && (n4.delete(t3), n4.size === 0 && this.map.delete(e4));
      }
      forEach(e4, t3) {
        let n4 = this.map.get(e4);
        n4 && n4.forEach(t3);
      }
      get(e4) {
        let t3 = this.map.get(e4);
        return t3 || /* @__PURE__ */ new Set();
      }
    };
    ((z4) => {
      function r7(m2) {
        return m2 && typeof m2 == "object" && typeof m2[Symbol.iterator] == "function";
      }
      z4.is = r7;
      let e4 = Object.freeze([]);
      function t3() {
        return e4;
      }
      z4.empty = t3;
      function* n4(m2) {
        yield m2;
      }
      z4.single = n4;
      function i12(m2) {
        return r7(m2) ? m2 : n4(m2);
      }
      z4.wrap = i12;
      function s18(m2) {
        return m2 || e4;
      }
      z4.from = s18;
      function* a3(m2) {
        for (let _4 = m2.length - 1; _4 >= 0; _4--) yield m2[_4];
      }
      z4.reverse = a3;
      function h4(m2) {
        return !m2 || m2[Symbol.iterator]().next().done === true;
      }
      z4.isEmpty = h4;
      function u3(m2) {
        return m2[Symbol.iterator]().next().value;
      }
      z4.first = u3;
      function p3(m2, _4) {
        let y3 = 0;
        for (let L4 of m2) if (_4(L4, y3++)) return true;
        return false;
      }
      z4.some = p3;
      function T2(m2, _4) {
        for (let y3 of m2) if (_4(y3)) return y3;
      }
      z4.find = T2;
      function* v4(m2, _4) {
        for (let y3 of m2) _4(y3) && (yield y3);
      }
      z4.filter = v4;
      function* I2(m2, _4) {
        let y3 = 0;
        for (let L4 of m2) yield _4(L4, y3++);
      }
      z4.map = I2;
      function* E2(m2, _4) {
        let y3 = 0;
        for (let L4 of m2) yield* _4(L4, y3++);
      }
      z4.flatMap = E2;
      function* S4(...m2) {
        for (let _4 of m2) yield* _4;
      }
      z4.concat = S4;
      function D4(m2, _4, y3) {
        let L4 = y3;
        for (let X3 of m2) L4 = _4(L4, X3);
        return L4;
      }
      z4.reduce = D4;
      function* x2(m2, _4, y3 = m2.length) {
        for (_4 < 0 && (_4 += m2.length), y3 < 0 ? y3 += m2.length : y3 > m2.length && (y3 = m2.length); _4 < y3; _4++) yield m2[_4];
      }
      z4.slice = x2;
      function J3(m2, _4 = Number.POSITIVE_INFINITY) {
        let y3 = [];
        if (_4 === 0) return [y3, m2];
        let L4 = m2[Symbol.iterator]();
        for (let X3 = 0; X3 < _4; X3++) {
          let Se2 = L4.next();
          if (Se2.done) return [y3, z4.empty()];
          y3.push(Se2.value);
        }
        return [y3, { [Symbol.iterator]() {
          return L4;
        } }];
      }
      z4.consume = J3;
      async function q2(m2) {
        let _4 = [];
        for await (let y3 of m2) _4.push(y3);
        return Promise.resolve(_4);
      }
      z4.asyncToArray = q2;
    })(Le2 || (Le2 = {}));
    Tt2 = false;
    K2 = null;
    de2 = class de3 {
      constructor() {
        this.livingDisposables = /* @__PURE__ */ new Map();
      }
      getDisposableData(e4) {
        let t3 = this.livingDisposables.get(e4);
        return t3 || (t3 = { parent: null, source: null, isSingleton: false, value: e4, idx: de3.idx++ }, this.livingDisposables.set(e4, t3)), t3;
      }
      trackDisposable(e4) {
        let t3 = this.getDisposableData(e4);
        t3.source || (t3.source = new Error().stack);
      }
      setParent(e4, t3) {
        let n4 = this.getDisposableData(e4);
        n4.parent = t3;
      }
      markAsDisposed(e4) {
        this.livingDisposables.delete(e4);
      }
      markAsSingleton(e4) {
        this.getDisposableData(e4).isSingleton = true;
      }
      getRootParent(e4, t3) {
        let n4 = t3.get(e4);
        if (n4) return n4;
        let i12 = e4.parent ? this.getRootParent(this.getDisposableData(e4.parent), t3) : e4;
        return t3.set(e4, i12), i12;
      }
      getTrackedDisposables() {
        let e4 = /* @__PURE__ */ new Map();
        return [...this.livingDisposables.entries()].filter(([, n4]) => n4.source !== null && !this.getRootParent(n4, e4).isSingleton).flatMap(([n4]) => n4);
      }
      computeLeakingDisposables(e4 = 10, t3) {
        let n4;
        if (t3) n4 = t3;
        else {
          let u3 = /* @__PURE__ */ new Map(), p3 = [...this.livingDisposables.values()].filter((v4) => v4.source !== null && !this.getRootParent(v4, u3).isSingleton);
          if (p3.length === 0) return;
          let T2 = new Set(p3.map((v4) => v4.value));
          if (n4 = p3.filter((v4) => !(v4.parent && T2.has(v4.parent))), n4.length === 0) throw new Error("There are cyclic diposable chains!");
        }
        if (!n4) return;
        function i12(u3) {
          function p3(v4, I2) {
            for (; v4.length > 0 && I2.some((E2) => typeof E2 == "string" ? E2 === v4[0] : v4[0].match(E2)); ) v4.shift();
          }
          let T2 = u3.source.split(`
`).map((v4) => v4.trim().replace("at ", "")).filter((v4) => v4 !== "");
          return p3(T2, ["Error", /^trackDisposable \(.*\)$/, /^DisposableTracker.trackDisposable \(.*\)$/]), T2.reverse();
        }
        let s18 = new ce2();
        for (let u3 of n4) {
          let p3 = i12(u3);
          for (let T2 = 0; T2 <= p3.length; T2++) s18.add(p3.slice(0, T2).join(`
`), u3);
        }
        n4.sort($e2((u3) => u3.idx, Be2));
        let a3 = "", h4 = 0;
        for (let u3 of n4.slice(0, e4)) {
          h4++;
          let p3 = i12(u3), T2 = [];
          for (let v4 = 0; v4 < p3.length; v4++) {
            let I2 = p3[v4];
            I2 = `(shared with ${s18.get(p3.slice(0, v4 + 1).join(`
`)).size}/${n4.length} leaks) at ${I2}`;
            let S4 = s18.get(p3.slice(0, v4).join(`
`)), D4 = Xe2([...S4].map((x2) => i12(x2)[v4]), (x2) => x2);
            delete D4[p3[v4]];
            for (let [x2, J3] of Object.entries(D4)) T2.unshift(`    - stacktraces of ${J3.length} other leaks continue with ${x2}`);
            T2.unshift(I2);
          }
          a3 += `


==================== Leaking disposable ${h4}/${n4.length}: ${u3.value.constructor.name} ====================
${T2.join(`
`)}
============================================================

`;
        }
        return n4.length > e4 && (a3 += `


... and ${n4.length - e4} more leaking disposables

`), { leaks: n4, details: a3 };
      }
    };
    de2.idx = 0;
    if (Tt2) {
      let r7 = "__is_disposable_tracked__";
      vt2(new class {
        trackDisposable(e4) {
          let t3 = new Error("Potentially leaked disposable").stack;
          setTimeout(() => {
            e4[r7] || console.log(t3);
          }, 3e3);
        }
        setParent(e4, t3) {
          if (e4 && e4 !== k3.None) try {
            e4[r7] = true;
          } catch {
          }
        }
        markAsDisposed(e4) {
          if (e4 && e4 !== k3.None) try {
            e4[r7] = true;
          } catch {
          }
        }
        markAsSingleton(e4) {
        }
      }());
    }
    he2 = class he3 {
      constructor() {
        this._toDispose = /* @__PURE__ */ new Set();
        this._isDisposed = false;
        pe2(this);
      }
      dispose() {
        this._isDisposed || (fe2(this), this._isDisposed = true, this.clear());
      }
      get isDisposed() {
        return this._isDisposed;
      }
      clear() {
        if (this._toDispose.size !== 0) try {
          Q3(this._toDispose);
        } finally {
          this._toDispose.clear();
        }
      }
      add(e4) {
        if (!e4) return e4;
        if (e4 === this) throw new Error("Cannot register a disposable on itself!");
        return te(e4, this), this._isDisposed ? he3.DISABLE_DISPOSED_WARNING || console.warn(new Error("Trying to add a disposable to a DisposableStore that has already been disposed of. The added object will be leaked!").stack) : this._toDispose.add(e4), e4;
      }
      delete(e4) {
        if (e4) {
          if (e4 === this) throw new Error("Cannot dispose a disposable on itself!");
          this._toDispose.delete(e4), e4.dispose();
        }
      }
      deleteAndLeak(e4) {
        e4 && this._toDispose.has(e4) && (this._toDispose.delete(e4), te(e4, null));
      }
    };
    he2.DISABLE_DISPOSED_WARNING = false;
    H3 = he2;
    k3 = class {
      constructor() {
        this._store = new H3();
        pe2(this), te(this._store, this);
      }
      dispose() {
        fe2(this), this._store.dispose();
      }
      _register(e4) {
        if (e4 === this) throw new Error("Cannot register a disposable on itself!");
        return this._store.add(e4);
      }
    };
    k3.None = Object.freeze({ dispose() {
    } });
    F2 = class {
      constructor() {
        this._isDisposed = false;
        pe2(this);
      }
      get value() {
        return this._isDisposed ? void 0 : this._value;
      }
      set value(e4) {
        this._isDisposed || e4 === this._value || (this._value?.dispose(), e4 && te(e4, this), this._value = e4);
      }
      clear() {
        this.value = void 0;
      }
      dispose() {
        this._isDisposed = true, fe2(this), this._value?.dispose(), this._value = void 0;
      }
      clearAndLeak() {
        let e4 = this._value;
        return this._value = void 0, e4 && te(e4, null), e4;
      }
    };
    $3 = class $4 {
      constructor(e4) {
        this.element = e4, this.next = $4.Undefined, this.prev = $4.Undefined;
      }
    };
    $3.Undefined = new $3(void 0);
    _t2 = globalThis.performance && typeof globalThis.performance.now == "function";
    Te2 = class r5 {
      static create(e4) {
        return new r5(e4);
      }
      constructor(e4) {
        this._now = _t2 && e4 === false ? Date.now : globalThis.performance.now.bind(globalThis.performance), this._startTime = this._now(), this._stopTime = -1;
      }
      stop() {
        this._stopTime = this._now();
      }
      reset() {
        this._startTime = this._now(), this._stopTime = -1;
      }
      elapsed() {
        return this._stopTime !== -1 ? this._stopTime - this._startTime : this._now() - this._startTime;
      }
    };
    gt2 = false;
    tt = false;
    yt = false;
    ((re3) => {
      re3.None = () => k3.None;
      function e4(d3) {
        if (yt) {
          let { onDidAddListener: o6 } = d3, c4 = ne2.create(), l4 = 0;
          d3.onDidAddListener = () => {
            ++l4 === 2 && (console.warn("snapshotted emitter LIKELY used public and SHOULD HAVE BEEN created with DisposableStore. snapshotted here"), c4.print()), o6?.();
          };
        }
      }
      function t3(d3, o6) {
        return I2(d3, () => {
        }, 0, void 0, true, void 0, o6);
      }
      re3.defer = t3;
      function n4(d3) {
        return (o6, c4 = null, l4) => {
          let f3 = false, b4;
          return b4 = d3((g3) => {
            if (!f3) return b4 ? b4.dispose() : f3 = true, o6.call(c4, g3);
          }, null, l4), f3 && b4.dispose(), b4;
        };
      }
      re3.once = n4;
      function i12(d3, o6, c4) {
        return T2((l4, f3 = null, b4) => d3((g3) => l4.call(f3, o6(g3)), null, b4), c4);
      }
      re3.map = i12;
      function s18(d3, o6, c4) {
        return T2((l4, f3 = null, b4) => d3((g3) => {
          o6(g3), l4.call(f3, g3);
        }, null, b4), c4);
      }
      re3.forEach = s18;
      function a3(d3, o6, c4) {
        return T2((l4, f3 = null, b4) => d3((g3) => o6(g3) && l4.call(f3, g3), null, b4), c4);
      }
      re3.filter = a3;
      function h4(d3) {
        return d3;
      }
      re3.signal = h4;
      function u3(...d3) {
        return (o6, c4 = null, l4) => {
          let f3 = me(...d3.map((b4) => b4((g3) => o6.call(c4, g3))));
          return v4(f3, l4);
        };
      }
      re3.any = u3;
      function p3(d3, o6, c4, l4) {
        let f3 = c4;
        return i12(d3, (b4) => (f3 = o6(f3, b4), f3), l4);
      }
      re3.reduce = p3;
      function T2(d3, o6) {
        let c4, l4 = { onWillAddFirstListener() {
          c4 = d3(f3.fire, f3);
        }, onDidRemoveLastListener() {
          c4?.dispose();
        } };
        o6 || e4(l4);
        let f3 = new C3(l4);
        return o6?.add(f3), f3.event;
      }
      function v4(d3, o6) {
        return o6 instanceof Array ? o6.push(d3) : o6 && o6.add(d3), d3;
      }
      function I2(d3, o6, c4 = 100, l4 = false, f3 = false, b4, g3) {
        let w3, R2, U3, se2 = 0, Z3, Ve3 = { leakWarningThreshold: b4, onWillAddFirstListener() {
          w3 = d3((dt3) => {
            se2++, R2 = o6(R2, dt3), l4 && !U3 && (oe.fire(R2), R2 = void 0), Z3 = () => {
              let ht2 = R2;
              R2 = void 0, U3 = void 0, (!l4 || se2 > 1) && oe.fire(ht2), se2 = 0;
            }, typeof c4 == "number" ? (clearTimeout(U3), U3 = setTimeout(Z3, c4)) : U3 === void 0 && (U3 = 0, queueMicrotask(Z3));
          });
        }, onWillRemoveListener() {
          f3 && se2 > 0 && Z3?.();
        }, onDidRemoveLastListener() {
          Z3 = void 0, w3.dispose();
        } };
        g3 || e4(Ve3);
        let oe = new C3(Ve3);
        return g3?.add(oe), oe.event;
      }
      re3.debounce = I2;
      function E2(d3, o6 = 0, c4) {
        return re3.debounce(d3, (l4, f3) => l4 ? (l4.push(f3), l4) : [f3], o6, void 0, true, void 0, c4);
      }
      re3.accumulate = E2;
      function S4(d3, o6 = (l4, f3) => l4 === f3, c4) {
        let l4 = true, f3;
        return a3(d3, (b4) => {
          let g3 = l4 || !o6(b4, f3);
          return l4 = false, f3 = b4, g3;
        }, c4);
      }
      re3.latch = S4;
      function D4(d3, o6, c4) {
        return [re3.filter(d3, o6, c4), re3.filter(d3, (l4) => !o6(l4), c4)];
      }
      re3.split = D4;
      function x2(d3, o6 = false, c4 = [], l4) {
        let f3 = c4.slice(), b4 = d3((R2) => {
          f3 ? f3.push(R2) : w3.fire(R2);
        });
        l4 && l4.add(b4);
        let g3 = () => {
          f3?.forEach((R2) => w3.fire(R2)), f3 = null;
        }, w3 = new C3({ onWillAddFirstListener() {
          b4 || (b4 = d3((R2) => w3.fire(R2)), l4 && l4.add(b4));
        }, onDidAddFirstListener() {
          f3 && (o6 ? setTimeout(g3) : g3());
        }, onDidRemoveLastListener() {
          b4 && b4.dispose(), b4 = null;
        } });
        return l4 && l4.add(w3), w3.event;
      }
      re3.buffer = x2;
      function J3(d3, o6) {
        return (l4, f3, b4) => {
          let g3 = o6(new z4());
          return d3(function(w3) {
            let R2 = g3.evaluate(w3);
            R2 !== q2 && l4.call(f3, R2);
          }, void 0, b4);
        };
      }
      re3.chain = J3;
      let q2 = Symbol("HaltChainable");
      class z4 {
        constructor() {
          this.steps = [];
        }
        map(o6) {
          return this.steps.push(o6), this;
        }
        forEach(o6) {
          return this.steps.push((c4) => (o6(c4), c4)), this;
        }
        filter(o6) {
          return this.steps.push((c4) => o6(c4) ? c4 : q2), this;
        }
        reduce(o6, c4) {
          let l4 = c4;
          return this.steps.push((f3) => (l4 = o6(l4, f3), l4)), this;
        }
        latch(o6 = (c4, l4) => c4 === l4) {
          let c4 = true, l4;
          return this.steps.push((f3) => {
            let b4 = c4 || !o6(f3, l4);
            return c4 = false, l4 = f3, b4 ? f3 : q2;
          }), this;
        }
        evaluate(o6) {
          for (let c4 of this.steps) if (o6 = c4(o6), o6 === q2) break;
          return o6;
        }
      }
      function m2(d3, o6, c4 = (l4) => l4) {
        let l4 = (...w3) => g3.fire(c4(...w3)), f3 = () => d3.on(o6, l4), b4 = () => d3.removeListener(o6, l4), g3 = new C3({ onWillAddFirstListener: f3, onDidRemoveLastListener: b4 });
        return g3.event;
      }
      re3.fromNodeEventEmitter = m2;
      function _4(d3, o6, c4 = (l4) => l4) {
        let l4 = (...w3) => g3.fire(c4(...w3)), f3 = () => d3.addEventListener(o6, l4), b4 = () => d3.removeEventListener(o6, l4), g3 = new C3({ onWillAddFirstListener: f3, onDidRemoveLastListener: b4 });
        return g3.event;
      }
      re3.fromDOMEventEmitter = _4;
      function y3(d3) {
        return new Promise((o6) => n4(d3)(o6));
      }
      re3.toPromise = y3;
      function L4(d3) {
        let o6 = new C3();
        return d3.then((c4) => {
          o6.fire(c4);
        }, () => {
          o6.fire(void 0);
        }).finally(() => {
          o6.dispose();
        }), o6.event;
      }
      re3.fromPromise = L4;
      function X3(d3, o6) {
        return d3((c4) => o6.fire(c4));
      }
      re3.forward = X3;
      function Se2(d3, o6, c4) {
        return o6(c4), d3((l4) => o6(l4));
      }
      re3.runAndSubscribe = Se2;
      class ct2 {
        constructor(o6, c4) {
          this._observable = o6;
          this._counter = 0;
          this._hasChanged = false;
          let l4 = { onWillAddFirstListener: () => {
            o6.addObserver(this);
          }, onDidRemoveLastListener: () => {
            o6.removeObserver(this);
          } };
          c4 || e4(l4), this.emitter = new C3(l4), c4 && c4.add(this.emitter);
        }
        beginUpdate(o6) {
          this._counter++;
        }
        handlePossibleChange(o6) {
        }
        handleChange(o6, c4) {
          this._hasChanged = true;
        }
        endUpdate(o6) {
          this._counter--, this._counter === 0 && (this._observable.reportChanges(), this._hasChanged && (this._hasChanged = false, this.emitter.fire(this._observable.get())));
        }
      }
      function zt3(d3, o6) {
        return new ct2(d3, o6).emitter.event;
      }
      re3.fromObservable = zt3;
      function Ut3(d3) {
        return (o6, c4, l4) => {
          let f3 = 0, b4 = false, g3 = { beginUpdate() {
            f3++;
          }, endUpdate() {
            f3--, f3 === 0 && (d3.reportChanges(), b4 && (b4 = false, o6.call(c4)));
          }, handlePossibleChange() {
          }, handleChange() {
            b4 = true;
          } };
          d3.addObserver(g3), d3.reportChanges();
          let w3 = { dispose() {
            d3.removeObserver(g3);
          } };
          return l4 instanceof H3 ? l4.add(w3) : Array.isArray(l4) && l4.push(w3), w3;
        };
      }
      re3.fromObservableLight = Ut3;
    })(ie2 || (ie2 = {}));
    B3 = class B4 {
      constructor(e4) {
        this.listenerCount = 0;
        this.invocationCount = 0;
        this.elapsedOverall = 0;
        this.durations = [];
        this.name = `${e4}_${B4._idPool++}`, B4.all.add(this);
      }
      start(e4) {
        this._stopWatch = new Te2(), this.listenerCount = e4;
      }
      stop() {
        if (this._stopWatch) {
          let e4 = this._stopWatch.elapsed();
          this.durations.push(e4), this.elapsedOverall += e4, this.invocationCount += 1, this._stopWatch = void 0;
        }
      }
    };
    B3.all = /* @__PURE__ */ new Set(), B3._idPool = 0;
    Oe = B3;
    nt2 = -1;
    be2 = class be3 {
      constructor(e4, t3, n4 = (be3._idPool++).toString(16).padStart(3, "0")) {
        this._errorHandler = e4;
        this.threshold = t3;
        this.name = n4;
        this._warnCountdown = 0;
      }
      dispose() {
        this._stacks?.clear();
      }
      check(e4, t3) {
        let n4 = this.threshold;
        if (n4 <= 0 || t3 < n4) return;
        this._stacks || (this._stacks = /* @__PURE__ */ new Map());
        let i12 = this._stacks.get(e4.value) || 0;
        if (this._stacks.set(e4.value, i12 + 1), this._warnCountdown -= 1, this._warnCountdown <= 0) {
          this._warnCountdown = n4 * 0.5;
          let [s18, a3] = this.getMostFrequentStack(), h4 = `[${this.name}] potential listener LEAK detected, having ${t3} listeners already. MOST frequent listener (${a3}):`;
          console.warn(h4), console.warn(s18);
          let u3 = new Me(h4, s18);
          this._errorHandler(u3);
        }
        return () => {
          let s18 = this._stacks.get(e4.value) || 0;
          this._stacks.set(e4.value, s18 - 1);
        };
      }
      getMostFrequentStack() {
        if (!this._stacks) return;
        let e4, t3 = 0;
        for (let [n4, i12] of this._stacks) (!e4 || t3 < i12) && (e4 = [n4, i12], t3 = i12);
        return e4;
      }
    };
    be2._idPool = 1;
    Ae2 = be2;
    ne2 = class r6 {
      constructor(e4) {
        this.value = e4;
      }
      static create() {
        let e4 = new Error();
        return new r6(e4.stack ?? "");
      }
      print() {
        console.warn(this.value.split(`
`).slice(2).join(`
`));
      }
    };
    Me = class extends Error {
      constructor(e4, t3) {
        super(e4), this.name = "ListenerLeakError", this.stack = t3;
      }
    };
    Fe2 = class extends Error {
      constructor(e4, t3) {
        super(e4), this.name = "ListenerRefusalError", this.stack = t3;
      }
    };
    xt2 = 0;
    G2 = class {
      constructor(e4) {
        this.value = e4;
        this.id = xt2++;
      }
    };
    It2 = 2;
    Dt2 = (r7, e4) => {
      if (r7 instanceof G2) e4(r7);
      else for (let t3 = 0; t3 < r7.length; t3++) {
        let n4 = r7[t3];
        n4 && e4(n4);
      }
    };
    if (gt2) {
      let r7 = [];
      setInterval(() => {
        r7.length !== 0 && (console.warn("[LEAKING LISTENERS] GC'ed these listeners that were NOT yet disposed:"), console.warn(r7.join(`
`)), r7.length = 0);
      }, 3e3), ve2 = new FinalizationRegistry((e4) => {
        typeof e4 == "string" && r7.push(e4);
      });
    }
    C3 = class {
      constructor(e4) {
        this._size = 0;
        this._options = e4, this._leakageMon = nt2 > 0 || this._options?.leakWarningThreshold ? new Ae2(e4?.onListenerError ?? le, this._options?.leakWarningThreshold ?? nt2) : void 0, this._perfMon = this._options?._profName ? new Oe(this._options._profName) : void 0, this._deliveryQueue = this._options?.deliveryQueue;
      }
      dispose() {
        if (!this._disposed) {
          if (this._disposed = true, this._deliveryQueue?.current === this && this._deliveryQueue.reset(), this._listeners) {
            if (tt) {
              let e4 = this._listeners;
              queueMicrotask(() => {
                Dt2(e4, (t3) => t3.stack?.print());
              });
            }
            this._listeners = void 0, this._size = 0;
          }
          this._options?.onDidRemoveLastListener?.(), this._leakageMon?.dispose();
        }
      }
      get event() {
        return this._event ?? (this._event = (e4, t3, n4) => {
          if (this._leakageMon && this._size > this._leakageMon.threshold ** 2) {
            let u3 = `[${this._leakageMon.name}] REFUSES to accept new listeners because it exceeded its threshold by far (${this._size} vs ${this._leakageMon.threshold})`;
            console.warn(u3);
            let p3 = this._leakageMon.getMostFrequentStack() ?? ["UNKNOWN stack", -1], T2 = new Fe2(`${u3}. HINT: Stack shows most frequent listener (${p3[1]}-times)`, p3[0]);
            return (this._options?.onListenerError || le)(T2), k3.None;
          }
          if (this._disposed) return k3.None;
          t3 && (e4 = e4.bind(t3));
          let i12 = new G2(e4), s18, a3;
          this._leakageMon && this._size >= Math.ceil(this._leakageMon.threshold * 0.2) && (i12.stack = ne2.create(), s18 = this._leakageMon.check(i12.stack, this._size + 1)), tt && (i12.stack = a3 ?? ne2.create()), this._listeners ? this._listeners instanceof G2 ? (this._deliveryQueue ?? (this._deliveryQueue = new Ne2()), this._listeners = [this._listeners, i12]) : this._listeners.push(i12) : (this._options?.onWillAddFirstListener?.(this), this._listeners = i12, this._options?.onDidAddFirstListener?.(this)), this._size++;
          let h4 = A2(() => {
            ve2?.unregister(h4), s18?.(), this._removeListener(i12);
          });
          if (n4 instanceof H3 ? n4.add(h4) : Array.isArray(n4) && n4.push(h4), ve2) {
            let u3 = new Error().stack.split(`
`).slice(2, 3).join(`
`).trim(), p3 = /(file:|vscode-file:\/\/vscode-app)?(\/[^:]*:\d+:\d+)/.exec(u3);
            ve2.register(h4, p3?.[2] ?? u3, h4);
          }
          return h4;
        }), this._event;
      }
      _removeListener(e4) {
        if (this._options?.onWillRemoveListener?.(this), !this._listeners) return;
        if (this._size === 1) {
          this._listeners = void 0, this._options?.onDidRemoveLastListener?.(this), this._size = 0;
          return;
        }
        let t3 = this._listeners, n4 = t3.indexOf(e4);
        if (n4 === -1) throw console.log("disposed?", this._disposed), console.log("size?", this._size), console.log("arr?", JSON.stringify(this._listeners)), new Error("Attempted to dispose unknown listener");
        this._size--, t3[n4] = void 0;
        let i12 = this._deliveryQueue.current === this;
        if (this._size * It2 <= t3.length) {
          let s18 = 0;
          for (let a3 = 0; a3 < t3.length; a3++) t3[a3] ? t3[s18++] = t3[a3] : i12 && (this._deliveryQueue.end--, s18 < this._deliveryQueue.i && this._deliveryQueue.i--);
          t3.length = s18;
        }
      }
      _deliver(e4, t3) {
        if (!e4) return;
        let n4 = this._options?.onListenerError || le;
        if (!n4) {
          e4.value(t3);
          return;
        }
        try {
          e4.value(t3);
        } catch (i12) {
          n4(i12);
        }
      }
      _deliverQueue(e4) {
        let t3 = e4.current._listeners;
        for (; e4.i < e4.end; ) this._deliver(t3[e4.i++], e4.value);
        e4.reset();
      }
      fire(e4) {
        if (this._deliveryQueue?.current && (this._deliverQueue(this._deliveryQueue), this._perfMon?.stop()), this._perfMon?.start(this._size), this._listeners) if (this._listeners instanceof G2) this._deliver(this._listeners, e4);
        else {
          let t3 = this._deliveryQueue;
          t3.enqueue(this, e4, this._listeners.length), this._deliverQueue(t3);
        }
        this._perfMon?.stop();
      }
      hasListeners() {
        return this._size > 0;
      }
    };
    Ne2 = class {
      constructor() {
        this.i = -1;
        this.end = 0;
      }
      enqueue(e4, t3, n4) {
        this.i = 0, this.end = n4, this.current = e4, this.value = t3;
      }
      reset() {
        this.i = this.end, this.current = void 0, this.value = void 0;
      }
    };
    it = Object.freeze(function(r7, e4) {
      let t3 = setTimeout(r7.bind(e4), 0);
      return { dispose() {
        clearTimeout(t3);
      } };
    });
    ((n4) => {
      function r7(i12) {
        return i12 === n4.None || i12 === n4.Cancelled || i12 instanceof We2 ? true : !i12 || typeof i12 != "object" ? false : typeof i12.isCancellationRequested == "boolean" && typeof i12.onCancellationRequested == "function";
      }
      n4.isCancellationToken = r7, n4.None = Object.freeze({ isCancellationRequested: false, onCancellationRequested: ie2.None }), n4.Cancelled = Object.freeze({ isCancellationRequested: true, onCancellationRequested: it });
    })(Et || (Et = {}));
    We2 = class {
      constructor() {
        this._isCancelled = false;
        this._emitter = null;
      }
      cancel() {
        this._isCancelled || (this._isCancelled = true, this._emitter && (this._emitter.fire(void 0), this.dispose()));
      }
      get isCancellationRequested() {
        return this._isCancelled;
      }
      get onCancellationRequested() {
        return this._isCancelled ? it : (this._emitter || (this._emitter = new C3()), this._emitter.event);
      }
      dispose() {
        this._emitter && (this._emitter.dispose(), this._emitter = null);
      }
    };
    Y2 = "en";
    qe2 = false;
    ze = false;
    ge2 = false;
    wt2 = false;
    kt3 = false;
    st2 = false;
    St2 = false;
    Rt2 = false;
    Ct2 = false;
    Pt2 = false;
    ye2 = Y2;
    rt2 = Y2;
    W = globalThis;
    typeof W.vscode < "u" && typeof W.vscode.process < "u" ? O = W.vscode.process : typeof process < "u" && typeof process?.versions?.node == "string" && (O = process);
    ot2 = typeof O?.versions?.electron == "string";
    Ot2 = ot2 && O?.type === "renderer";
    if (typeof O == "object") {
      qe2 = O.platform === "win32", ze = O.platform === "darwin", ge2 = O.platform === "linux", wt2 = ge2 && !!O.env.SNAP && !!O.env.SNAP_REVISION, St2 = ot2, Ct2 = !!O.env.CI || !!O.env.BUILD_ARTIFACTSTAGINGDIRECTORY, _e3 = Y2, ye2 = Y2;
      let r7 = O.env.VSCODE_NLS_CONFIG;
      if (r7) try {
        let e4 = JSON.parse(r7);
        _e3 = e4.userLocale, rt2 = e4.osLocale, ye2 = e4.resolvedLanguage || Y2, Lt2 = e4.languagePack?.translationsConfigFile;
      } catch {
      }
      kt3 = true;
    } else typeof navigator == "object" && !Ot2 ? (N2 = navigator.userAgent, qe2 = N2.indexOf("Windows") >= 0, ze = N2.indexOf("Macintosh") >= 0, Rt2 = (N2.indexOf("Macintosh") >= 0 || N2.indexOf("iPad") >= 0 || N2.indexOf("iPhone") >= 0) && !!navigator.maxTouchPoints && navigator.maxTouchPoints > 0, ge2 = N2.indexOf("Linux") >= 0, Pt2 = N2?.indexOf("Mobi") >= 0, st2 = true, ye2 = globalThis._VSCODE_NLS_LANGUAGE || Y2, _e3 = navigator.language.toLowerCase(), rt2 = _e3) : console.error("Unable to resolve platform.");
    je2 = 0;
    ze ? je2 = 1 : qe2 ? je2 = 3 : ge2 && (je2 = 2);
    At3 = st2 && typeof W.importScripts == "function";
    gn2 = At3 ? W.origin : void 0;
    M3 = N2;
    j2 = ye2;
    ((n4) => {
      function r7() {
        return j2;
      }
      n4.value = r7;
      function e4() {
        return j2.length === 2 ? j2 === "en" : j2.length >= 3 ? j2[0] === "e" && j2[1] === "n" && j2[2] === "-" : false;
      }
      n4.isDefaultVariant = e4;
      function t3() {
        return j2 === "en";
      }
      n4.isDefault = t3;
    })(Mt3 || (Mt3 = {}));
    Ft = typeof W.postMessage == "function" && !W.importScripts;
    at2 = (() => {
      if (Ft) {
        let r7 = [];
        W.addEventListener("message", (t3) => {
          if (t3.data && t3.data.vscodeScheduleAsyncWork) for (let n4 = 0, i12 = r7.length; n4 < i12; n4++) {
            let s18 = r7[n4];
            if (s18.id === t3.data.vscodeScheduleAsyncWork) {
              r7.splice(n4, 1), s18.callback();
              return;
            }
          }
        });
        let e4 = 0;
        return (t3) => {
          let n4 = ++e4;
          r7.push({ id: n4, callback: t3 }), W.postMessage({ vscodeScheduleAsyncWork: n4 }, "*");
        };
      }
      return (r7) => setTimeout(r7);
    })();
    Nt2 = !!(M3 && M3.indexOf("Chrome") >= 0);
    yn2 = !!(M3 && M3.indexOf("Firefox") >= 0);
    xn2 = !!(!Nt2 && M3 && M3.indexOf("Safari") >= 0);
    In2 = !!(M3 && M3.indexOf("Edg/") >= 0);
    Dn2 = !!(M3 && M3.indexOf("Android") >= 0);
    Wt2 = Symbol("MicrotaskDelay");
    (function() {
      typeof globalThis.requestIdleCallback != "function" || typeof globalThis.cancelIdleCallback != "function" ? Ue2 = (r7, e4) => {
        at2(() => {
          if (t3) return;
          let n4 = Date.now() + 15;
          e4(Object.freeze({ didTimeout: true, timeRemaining() {
            return Math.max(0, n4 - Date.now());
          } }));
        });
        let t3 = false;
        return { dispose() {
          t3 || (t3 = true);
        } };
      } : Ue2 = (r7, e4, t3) => {
        let n4 = r7.requestIdleCallback(e4, typeof t3 == "number" ? { timeout: t3 } : void 0), i12 = false;
        return { dispose() {
          i12 || (i12 = true, r7.cancelIdleCallback(n4));
        } };
      }, jt2 = (r7) => Ue2(globalThis, r7);
    })();
    ((t3) => {
      async function r7(n4) {
        let i12, s18 = await Promise.all(n4.map((a3) => a3.then((h4) => h4, (h4) => {
          i12 || (i12 = h4);
        })));
        if (typeof i12 < "u") throw i12;
        return s18;
      }
      t3.settled = r7;
      function e4(n4) {
        return new Promise(async (i12, s18) => {
          try {
            await n4(i12, s18);
          } catch (a3) {
            s18(a3);
          }
        });
      }
      t3.withAsyncBody = e4;
    })(qt || (qt = {}));
    P2 = class P3 {
      static fromArray(e4) {
        return new P3((t3) => {
          t3.emitMany(e4);
        });
      }
      static fromPromise(e4) {
        return new P3(async (t3) => {
          t3.emitMany(await e4);
        });
      }
      static fromPromises(e4) {
        return new P3(async (t3) => {
          await Promise.all(e4.map(async (n4) => t3.emitOne(await n4)));
        });
      }
      static merge(e4) {
        return new P3(async (t3) => {
          await Promise.all(e4.map(async (n4) => {
            for await (let i12 of n4) t3.emitOne(i12);
          }));
        });
      }
      constructor(e4, t3) {
        this._state = 0, this._results = [], this._error = null, this._onReturn = t3, this._onStateChanged = new C3(), queueMicrotask(async () => {
          let n4 = { emitOne: (i12) => this.emitOne(i12), emitMany: (i12) => this.emitMany(i12), reject: (i12) => this.reject(i12) };
          try {
            await Promise.resolve(e4(n4)), this.resolve();
          } catch (i12) {
            this.reject(i12);
          } finally {
            n4.emitOne = void 0, n4.emitMany = void 0, n4.reject = void 0;
          }
        });
      }
      [Symbol.asyncIterator]() {
        let e4 = 0;
        return { next: async () => {
          do {
            if (this._state === 2) throw this._error;
            if (e4 < this._results.length) return { done: false, value: this._results[e4++] };
            if (this._state === 1) return { done: true, value: void 0 };
            await ie2.toPromise(this._onStateChanged.event);
          } while (true);
        }, return: async () => (this._onReturn?.(), { done: true, value: void 0 }) };
      }
      static map(e4, t3) {
        return new P3(async (n4) => {
          for await (let i12 of e4) n4.emitOne(t3(i12));
        });
      }
      map(e4) {
        return P3.map(this, e4);
      }
      static filter(e4, t3) {
        return new P3(async (n4) => {
          for await (let i12 of e4) t3(i12) && n4.emitOne(i12);
        });
      }
      filter(e4) {
        return P3.filter(this, e4);
      }
      static coalesce(e4) {
        return P3.filter(e4, (t3) => !!t3);
      }
      coalesce() {
        return P3.coalesce(this);
      }
      static async toPromise(e4) {
        let t3 = [];
        for await (let n4 of e4) t3.push(n4);
        return t3;
      }
      toPromise() {
        return P3.toPromise(this);
      }
      emitOne(e4) {
        this._state === 0 && (this._results.push(e4), this._onStateChanged.fire());
      }
      emitMany(e4) {
        this._state === 0 && (this._results = this._results.concat(e4), this._onStateChanged.fire());
      }
      resolve() {
        this._state === 0 && (this._state = 1, this._onStateChanged.fire());
      }
      reject(e4) {
        this._state === 0 && (this._state = 2, this._error = e4, this._onStateChanged.fire());
      }
    };
    P2.EMPTY = P2.fromArray([]);
    Ie2 = class extends k3 {
      constructor(t3) {
        super();
        this._terminal = t3;
        this._linesCacheTimeout = this._register(new F2());
        this._linesCacheDisposables = this._register(new F2());
        this._register(A2(() => this._destroyLinesCache()));
      }
      initLinesCache() {
        this._linesCache || (this._linesCache = new Array(this._terminal.buffer.active.length), this._linesCacheDisposables.value = me(this._terminal.onLineFeed(() => this._destroyLinesCache()), this._terminal.onCursorMove(() => this._destroyLinesCache()), this._terminal.onResize(() => this._destroyLinesCache()))), this._linesCacheTimeout.value = xe2(() => this._destroyLinesCache(), 15e3);
      }
      _destroyLinesCache() {
        this._linesCache = void 0, this._linesCacheDisposables.clear(), this._linesCacheTimeout.clear();
      }
      getLineFromCache(t3) {
        return this._linesCache?.[t3];
      }
      setLineInCache(t3, n4) {
        this._linesCache && (this._linesCache[t3] = n4);
      }
      translateBufferLineToStringWithWrap(t3, n4) {
        let i12 = [], s18 = [0], a3 = this._terminal.buffer.active.getLine(t3);
        for (; a3; ) {
          let h4 = this._terminal.buffer.active.getLine(t3 + 1), u3 = h4 ? h4.isWrapped : false, p3 = a3.translateToString(!u3 && n4);
          if (u3 && h4) {
            let T2 = a3.getCell(a3.length - 1);
            T2 && T2.getCode() === 0 && T2.getWidth() === 1 && h4.getCell(0)?.getWidth() === 2 && (p3 = p3.slice(0, -1));
          }
          if (i12.push(p3), u3) s18.push(s18[s18.length - 1] + p3.length);
          else break;
          t3++, a3 = h4;
        }
        return [i12.join(""), s18];
      }
    };
    De2 = class {
      get cachedSearchTerm() {
        return this._cachedSearchTerm;
      }
      set cachedSearchTerm(e4) {
        this._cachedSearchTerm = e4;
      }
      get lastSearchOptions() {
        return this._lastSearchOptions;
      }
      set lastSearchOptions(e4) {
        this._lastSearchOptions = e4;
      }
      isValidSearchTerm(e4) {
        return !!(e4 && e4.length > 0);
      }
      didOptionsChange(e4) {
        return this._lastSearchOptions ? e4 ? this._lastSearchOptions.caseSensitive !== e4.caseSensitive || this._lastSearchOptions.regex !== e4.regex || this._lastSearchOptions.wholeWord !== e4.wholeWord : false : true;
      }
      shouldUpdateHighlighting(e4, t3) {
        return t3?.decorations ? this._cachedSearchTerm === void 0 || e4 !== this._cachedSearchTerm || this.didOptionsChange(t3) : false;
      }
      clearCachedTerm() {
        this._cachedSearchTerm = void 0;
      }
      reset() {
        this._cachedSearchTerm = void 0, this._lastSearchOptions = void 0;
      }
    };
    Ee2 = class {
      constructor(e4, t3) {
        this._terminal = e4;
        this._lineCache = t3;
      }
      find(e4, t3, n4, i12) {
        if (!e4 || e4.length === 0) {
          this._terminal.clearSelection();
          return;
        }
        if (n4 > this._terminal.cols) throw new Error(`Invalid col: ${n4} to search in terminal of ${this._terminal.cols} cols`);
        this._lineCache.initLinesCache();
        let s18 = { startRow: t3, startCol: n4 }, a3 = this._findInLine(e4, s18, i12);
        if (!a3) for (let h4 = t3 + 1; h4 < this._terminal.buffer.active.baseY + this._terminal.rows && (s18.startRow = h4, s18.startCol = 0, a3 = this._findInLine(e4, s18, i12), !a3); h4++) ;
        return a3;
      }
      findNextWithSelection(e4, t3, n4) {
        if (!e4 || e4.length === 0) {
          this._terminal.clearSelection();
          return;
        }
        let i12 = this._terminal.getSelectionPosition();
        this._terminal.clearSelection();
        let s18 = 0, a3 = 0;
        i12 && (n4 === e4 ? (s18 = i12.end.x, a3 = i12.end.y) : (s18 = i12.start.x, a3 = i12.start.y)), this._lineCache.initLinesCache();
        let h4 = { startRow: a3, startCol: s18 }, u3 = this._findInLine(e4, h4, t3);
        if (!u3) for (let p3 = a3 + 1; p3 < this._terminal.buffer.active.baseY + this._terminal.rows && (h4.startRow = p3, h4.startCol = 0, u3 = this._findInLine(e4, h4, t3), !u3); p3++) ;
        if (!u3 && a3 !== 0) for (let p3 = 0; p3 < a3 && (h4.startRow = p3, h4.startCol = 0, u3 = this._findInLine(e4, h4, t3), !u3); p3++) ;
        return !u3 && i12 && (h4.startRow = i12.start.y, h4.startCol = 0, u3 = this._findInLine(e4, h4, t3)), u3;
      }
      findPreviousWithSelection(e4, t3, n4) {
        if (!e4 || e4.length === 0) {
          this._terminal.clearSelection();
          return;
        }
        let i12 = this._terminal.getSelectionPosition();
        this._terminal.clearSelection();
        let s18 = this._terminal.buffer.active.baseY + this._terminal.rows - 1, a3 = this._terminal.cols, h4 = true;
        this._lineCache.initLinesCache();
        let u3 = { startRow: s18, startCol: a3 }, p3;
        if (i12 && (u3.startRow = s18 = i12.start.y, u3.startCol = a3 = i12.start.x, n4 !== e4 && (p3 = this._findInLine(e4, u3, t3, false), p3 || (u3.startRow = s18 = i12.end.y, u3.startCol = a3 = i12.end.x))), p3 || (p3 = this._findInLine(e4, u3, t3, h4)), !p3) {
          u3.startCol = Math.max(u3.startCol, this._terminal.cols);
          for (let T2 = s18 - 1; T2 >= 0 && (u3.startRow = T2, p3 = this._findInLine(e4, u3, t3, h4), !p3); T2--) ;
        }
        if (!p3 && s18 !== this._terminal.buffer.active.baseY + this._terminal.rows - 1) for (let T2 = this._terminal.buffer.active.baseY + this._terminal.rows - 1; T2 >= s18 && (u3.startRow = T2, p3 = this._findInLine(e4, u3, t3, h4), !p3); T2--) ;
        return p3;
      }
      _isWholeWord(e4, t3, n4) {
        return (e4 === 0 || " ~!@#$%^&*()+`-=[]{}|\\;:\"',./<>?".includes(t3[e4 - 1])) && (e4 + n4.length === t3.length || " ~!@#$%^&*()+`-=[]{}|\\;:\"',./<>?".includes(t3[e4 + n4.length]));
      }
      _findInLine(e4, t3, n4 = {}, i12 = false) {
        let s18 = t3.startRow, a3 = t3.startCol;
        if (this._terminal.buffer.active.getLine(s18)?.isWrapped) {
          if (i12) {
            t3.startCol += this._terminal.cols;
            return;
          }
          return t3.startRow--, t3.startCol += this._terminal.cols, this._findInLine(e4, t3, n4);
        }
        let u3 = this._lineCache.getLineFromCache(s18);
        u3 || (u3 = this._lineCache.translateBufferLineToStringWithWrap(s18, true), this._lineCache.setLineInCache(s18, u3));
        let [p3, T2] = u3, v4 = this._bufferColsToStringOffset(s18, a3), I2 = e4, E2 = p3;
        n4.regex || (I2 = n4.caseSensitive ? e4 : e4.toLowerCase(), E2 = n4.caseSensitive ? p3 : p3.toLowerCase());
        let S4 = -1;
        if (n4.regex) {
          let D4 = RegExp(I2, n4.caseSensitive ? "g" : "gi"), x2;
          if (i12) for (; x2 = D4.exec(E2.slice(0, v4)); ) S4 = D4.lastIndex - x2[0].length, e4 = x2[0], D4.lastIndex -= e4.length - 1;
          else x2 = D4.exec(E2.slice(v4)), x2 && x2[0].length > 0 && (S4 = v4 + (D4.lastIndex - x2[0].length), e4 = x2[0]);
        } else i12 ? v4 - I2.length >= 0 && (S4 = E2.lastIndexOf(I2, v4 - I2.length)) : S4 = E2.indexOf(I2, v4);
        if (S4 >= 0) {
          if (n4.wholeWord && !this._isWholeWord(S4, E2, e4)) return;
          let D4 = 0;
          for (; D4 < T2.length - 1 && S4 >= T2[D4 + 1]; ) D4++;
          let x2 = D4;
          for (; x2 < T2.length - 1 && S4 + e4.length >= T2[x2 + 1]; ) x2++;
          let J3 = S4 - T2[D4], q2 = S4 + e4.length - T2[x2], z4 = this._stringLengthToBufferSize(s18 + D4, J3), _4 = this._stringLengthToBufferSize(s18 + x2, q2) - z4 + this._terminal.cols * (x2 - D4);
          return { term: e4, col: z4, row: s18 + D4, size: _4 };
        }
      }
      _stringLengthToBufferSize(e4, t3) {
        let n4 = this._terminal.buffer.active.getLine(e4);
        if (!n4) return 0;
        for (let i12 = 0; i12 < t3; i12++) {
          let s18 = n4.getCell(i12);
          if (!s18) break;
          let a3 = s18.getChars();
          a3.length > 1 && (t3 -= a3.length - 1);
          let h4 = n4.getCell(i12 + 1);
          h4 && h4.getWidth() === 0 && t3++;
        }
        return t3;
      }
      _bufferColsToStringOffset(e4, t3) {
        let n4 = e4, i12 = 0, s18 = this._terminal.buffer.active.getLine(n4);
        for (; t3 > 0 && s18; ) {
          for (let a3 = 0; a3 < t3 && a3 < this._terminal.cols; a3++) {
            let h4 = s18.getCell(a3);
            if (!h4) break;
            h4.getWidth() && (i12 += h4.getCode() === 0 ? 1 : h4.getChars().length);
          }
          if (n4++, s18 = this._terminal.buffer.active.getLine(n4), s18 && !s18.isWrapped) break;
          t3 -= this._terminal.cols;
        }
        return i12;
      }
    };
    we2 = class extends k3 {
      constructor(t3) {
        super();
        this._terminal = t3;
        this._highlightDecorations = [];
        this._highlightedLines = /* @__PURE__ */ new Set();
        this._register(A2(() => this.clearHighlightDecorations()));
      }
      createHighlightDecorations(t3, n4) {
        this.clearHighlightDecorations();
        for (let i12 of t3) {
          let s18 = this._createResultDecorations(i12, n4, false);
          if (s18) for (let a3 of s18) this._storeDecoration(a3, i12);
        }
      }
      createActiveDecoration(t3, n4) {
        let i12 = this._createResultDecorations(t3, n4, true);
        if (i12) return { decorations: i12, match: t3, dispose() {
          Q3(i12);
        } };
      }
      clearHighlightDecorations() {
        Q3(this._highlightDecorations), this._highlightDecorations = [], this._highlightedLines.clear();
      }
      _storeDecoration(t3, n4) {
        this._highlightedLines.add(t3.marker.line), this._highlightDecorations.push({ decoration: t3, match: n4, dispose() {
          t3.dispose();
        } });
      }
      _applyStyles(t3, n4, i12) {
        t3.classList.contains("xterm-find-result-decoration") || (t3.classList.add("xterm-find-result-decoration"), n4 && (t3.style.outline = `1px solid ${n4}`)), i12 && t3.classList.add("xterm-find-active-result-decoration");
      }
      _createResultDecorations(t3, n4, i12) {
        let s18 = [], a3 = t3.col, h4 = t3.size, u3 = -this._terminal.buffer.active.baseY - this._terminal.buffer.active.cursorY + t3.row;
        for (; h4 > 0; ) {
          let T2 = Math.min(this._terminal.cols - a3, h4);
          s18.push([u3, a3, T2]), a3 = 0, h4 -= T2, u3++;
        }
        let p3 = [];
        for (let T2 of s18) {
          let v4 = this._terminal.registerMarker(T2[0]), I2 = this._terminal.registerDecoration({ marker: v4, x: T2[1], width: T2[2], backgroundColor: i12 ? n4.activeMatchBackground : n4.matchBackground, overviewRulerOptions: this._highlightedLines.has(v4.line) ? void 0 : { color: i12 ? n4.activeMatchColorOverviewRuler : n4.matchOverviewRuler, position: "center" } });
          if (I2) {
            let E2 = [];
            E2.push(v4), E2.push(I2.onRender((S4) => this._applyStyles(S4, i12 ? n4.activeMatchBorder : n4.matchBorder, false))), E2.push(I2.onDispose(() => Q3(E2))), p3.push(I2);
          }
        }
        return p3.length === 0 ? void 0 : p3;
      }
    };
    ke2 = class extends k3 {
      constructor() {
        super(...arguments);
        this._searchResults = [];
        this._onDidChangeResults = this._register(new C3());
      }
      get onDidChangeResults() {
        return this._onDidChangeResults.event;
      }
      get searchResults() {
        return this._searchResults;
      }
      get selectedDecoration() {
        return this._selectedDecoration;
      }
      set selectedDecoration(t3) {
        this._selectedDecoration = t3;
      }
      updateResults(t3, n4) {
        this._searchResults = t3.slice(0, n4);
      }
      clearResults() {
        this._searchResults = [];
      }
      clearSelectedDecoration() {
        this._selectedDecoration && (this._selectedDecoration.dispose(), this._selectedDecoration = void 0);
      }
      findResultIndex(t3) {
        for (let n4 = 0; n4 < this._searchResults.length; n4++) {
          let i12 = this._searchResults[n4];
          if (i12.row === t3.row && i12.col === t3.col && i12.size === t3.size) return n4;
        }
        return -1;
      }
      fireResultsChanged(t3) {
        if (!t3) return;
        let n4 = -1;
        this._selectedDecoration && (n4 = this.findResultIndex(this._selectedDecoration.match)), this._onDidChangeResults.fire({ resultIndex: n4, resultCount: this._searchResults.length });
      }
      reset() {
        this.clearSelectedDecoration(), this.clearResults();
      }
    };
    ut2 = class extends k3 {
      constructor(t3) {
        super();
        this._highlightTimeout = this._register(new F2());
        this._lineCache = this._register(new F2());
        this._state = new De2();
        this._resultTracker = this._register(new ke2());
        this._highlightLimit = t3?.highlightLimit ?? 1e3;
      }
      get onDidChangeResults() {
        return this._resultTracker.onDidChangeResults;
      }
      activate(t3) {
        this._terminal = t3, this._lineCache.value = new Ie2(t3), this._engine = new Ee2(t3, this._lineCache.value), this._decorationManager = new we2(t3), this._register(this._terminal.onWriteParsed(() => this._updateMatches())), this._register(this._terminal.onResize(() => this._updateMatches())), this._register(A2(() => this.clearDecorations()));
      }
      _updateMatches() {
        this._highlightTimeout.clear(), this._state.cachedSearchTerm && this._state.lastSearchOptions?.decorations && (this._highlightTimeout.value = xe2(() => {
          let t3 = this._state.cachedSearchTerm;
          this._state.clearCachedTerm(), this.findPrevious(t3, { ...this._state.lastSearchOptions, incremental: true }, { noScroll: true });
        }, 200));
      }
      clearDecorations(t3) {
        this._resultTracker.clearSelectedDecoration(), this._decorationManager?.clearHighlightDecorations(), this._resultTracker.clearResults(), t3 || this._state.clearCachedTerm();
      }
      clearActiveDecoration() {
        this._resultTracker.clearSelectedDecoration();
      }
      findNext(t3, n4, i12) {
        if (!this._terminal || !this._engine) throw new Error("Cannot use addon until it has been loaded");
        this._state.lastSearchOptions = n4, this._state.shouldUpdateHighlighting(t3, n4) && this._highlightAllMatches(t3, n4);
        let s18 = this._findNextAndSelect(t3, n4, i12);
        return this._fireResults(n4), this._state.cachedSearchTerm = t3, s18;
      }
      _highlightAllMatches(t3, n4) {
        if (!this._terminal || !this._engine || !this._decorationManager) throw new Error("Cannot use addon until it has been loaded");
        if (!this._state.isValidSearchTerm(t3)) {
          this.clearDecorations();
          return;
        }
        this.clearDecorations(true);
        let i12 = [], s18, a3 = this._engine.find(t3, 0, 0, n4);
        for (; a3 && (s18?.row !== a3.row || s18?.col !== a3.col) && !(i12.length >= this._highlightLimit); ) s18 = a3, i12.push(s18), a3 = this._engine.find(t3, s18.col + s18.term.length >= this._terminal.cols ? s18.row + 1 : s18.row, s18.col + s18.term.length >= this._terminal.cols ? 0 : s18.col + 1, n4);
        this._resultTracker.updateResults(i12, this._highlightLimit), n4.decorations && this._decorationManager.createHighlightDecorations(i12, n4.decorations);
      }
      _findNextAndSelect(t3, n4, i12) {
        if (!this._terminal || !this._engine) return false;
        if (!this._state.isValidSearchTerm(t3)) return this._terminal.clearSelection(), this.clearDecorations(), false;
        let s18 = this._engine.findNextWithSelection(t3, n4, this._state.cachedSearchTerm);
        return this._selectResult(s18, n4?.decorations, i12?.noScroll);
      }
      findPrevious(t3, n4, i12) {
        if (!this._terminal || !this._engine) throw new Error("Cannot use addon until it has been loaded");
        this._state.lastSearchOptions = n4, this._state.shouldUpdateHighlighting(t3, n4) && this._highlightAllMatches(t3, n4);
        let s18 = this._findPreviousAndSelect(t3, n4, i12);
        return this._fireResults(n4), this._state.cachedSearchTerm = t3, s18;
      }
      _fireResults(t3) {
        this._resultTracker.fireResultsChanged(!!t3?.decorations);
      }
      _findPreviousAndSelect(t3, n4, i12) {
        if (!this._terminal || !this._engine) return false;
        if (!this._state.isValidSearchTerm(t3)) return this._terminal.clearSelection(), this.clearDecorations(), false;
        let s18 = this._engine.findPreviousWithSelection(t3, n4, this._state.cachedSearchTerm);
        return this._selectResult(s18, n4?.decorations, i12?.noScroll);
      }
      _selectResult(t3, n4, i12) {
        if (!this._terminal || !this._decorationManager) return false;
        if (this._resultTracker.clearSelectedDecoration(), !t3) return this._terminal.clearSelection(), false;
        if (this._terminal.select(t3.col, t3.row, t3.size), n4) {
          let s18 = this._decorationManager.createActiveDecoration(t3, n4);
          s18 && (this._resultTracker.selectedDecoration = s18);
        }
        if (!i12 && (t3.row >= this._terminal.buffer.active.viewportY + this._terminal.rows || t3.row < this._terminal.buffer.active.viewportY)) {
          let s18 = t3.row - this._terminal.buffer.active.viewportY;
          s18 -= Math.floor(this._terminal.rows / 2), this._terminal.scrollLines(s18);
        }
        return true;
      }
    };
  }
});

// node_modules/.pnpm/@xterm+addon-webgl@0.19.0/node_modules/@xterm/addon-webgl/lib/addon-webgl.mjs
var addon_webgl_exports = {};
__export(addon_webgl_exports, {
  WebglAddon: () => xr2
});
function Pe2(i12) {
  Dr2(i12) || Rr2.onUnexpectedError(i12);
}
function Dr2(i12) {
  return i12 instanceof Ye3 ? true : i12 instanceof Error && i12.name === fi && i12.message === fi;
}
function Mr2(i12, e4, t3 = 0, n4 = i12.length) {
  let s18 = t3, o6 = n4;
  for (; s18 < o6; ) {
    let r7 = Math.floor((s18 + o6) / 2);
    e4(i12[r7]) ? s18 = r7 + 1 : o6 = r7;
  }
  return s18 - 1;
}
function tn2(i12, e4) {
  return (t3, n4) => e4(i12(t3), i12(n4));
}
function an2(i12, e4) {
  let t3 = /* @__PURE__ */ Object.create(null);
  for (let n4 of i12) {
    let s18 = e4(n4), o6 = t3[s18];
    o6 || (o6 = t3[s18] = []), o6.push(n4);
  }
  return t3;
}
function mi2(i12, e4) {
  let t3 = this, n4 = false, s18;
  return function() {
    if (n4) return s18;
    if (n4 = true, e4) try {
      s18 = i12.apply(t3, arguments);
    } finally {
      e4();
    }
    else s18 = i12.apply(t3, arguments);
    return s18;
  };
}
function Sr2(i12) {
  Ne3 = i12;
}
function Et2(i12) {
  return Ne3?.trackDisposable(i12), i12;
}
function yt2(i12) {
  Ne3?.markAsDisposed(i12);
}
function Qe2(i12, e4) {
  Ne3?.setParent(i12, e4);
}
function Or2(i12, e4) {
  if (Ne3) for (let t3 of i12) Ne3.setParent(t3, e4);
}
function un3(i12) {
  if (_i2.is(i12)) {
    let e4 = [];
    for (let t3 of i12) if (t3) try {
      t3.dispose();
    } catch (n4) {
      e4.push(n4);
    }
    if (e4.length === 1) throw e4[0];
    if (e4.length > 1) throw new AggregateError(e4, "Encountered errors while disposing of store");
    return Array.isArray(i12) ? [] : i12;
  } else if (i12) return i12.dispose(), i12;
}
function It3(...i12) {
  let e4 = O2(() => un3(i12));
  return Or2(i12, e4), e4;
}
function O2(i12) {
  let e4 = Et2({ dispose: mi2(() => {
    yt2(e4), i12();
  }) });
  return e4;
}
function hn2() {
  if (!vi2) return 0;
  let i12 = Ze2.match(/Version\/(\d+)/);
  return i12 === null || i12.length < 2 ? 0 : parseInt(i12[1]);
}
function Oe2(i12) {
  let e4 = i12.toString(16);
  return e4.length < 2 ? "0" + e4 : e4;
}
function ve3(i12, e4) {
  return i12 < e4 ? (e4 + 0.05) / (i12 + 0.05) : (i12 + 0.05) / (e4 + 0.05);
}
function F3(i12) {
  if (!i12) throw new Error("value must not be falsy");
  return i12;
}
function Rt3(i12) {
  return 57508 <= i12 && i12 <= 57558;
}
function fn2(i12) {
  return 57520 <= i12 && i12 <= 57527;
}
function kr2(i12) {
  return 57344 <= i12 && i12 <= 63743;
}
function Pr2(i12) {
  return 9472 <= i12 && i12 <= 9631;
}
function Br2(i12) {
  return i12 >= 128512 && i12 <= 128591 || i12 >= 127744 && i12 <= 128511 || i12 >= 128640 && i12 <= 128767 || i12 >= 9728 && i12 <= 9983 || i12 >= 9984 && i12 <= 10175 || i12 >= 65024 && i12 <= 65039 || i12 >= 129280 && i12 <= 129535 || i12 >= 127462 && i12 <= 127487;
}
function mn2(i12, e4, t3, n4) {
  return e4 === 1 && t3 > Math.ceil(n4 * 1.5) && i12 !== void 0 && i12 > 255 && !Br2(i12) && !Rt3(i12) && !kr2(i12);
}
function Dt3(i12) {
  return Rt3(i12) || Pr2(i12);
}
function _n2() {
  return { css: { canvas: wt3(), cell: wt3() }, device: { canvas: wt3(), cell: wt3(), char: { width: 0, height: 0, left: 0, top: 0 } } };
}
function wt3() {
  return { width: 0, height: 0 };
}
function bn2(i12, e4, t3 = 0) {
  return (i12 - (Math.round(e4) * 2 - t3)) % (Math.round(e4) * 2);
}
function yn3(i12, e4, t3, n4, s18, o6, r7, a3) {
  let l4 = Hr[e4];
  if (l4) return $r2(i12, l4, t3, n4, s18, o6), true;
  let u3 = Wr2[e4];
  if (u3) return Kr2(i12, u3, t3, n4, s18, o6), true;
  let c4 = Gr2[e4];
  if (c4) return Vr2(i12, c4, t3, n4, s18, o6, a3), true;
  let d3 = et[e4];
  return d3 ? (Cr3(i12, d3, t3, n4, s18, o6, r7, a3), true) : false;
}
function $r2(i12, e4, t3, n4, s18, o6) {
  for (let r7 = 0; r7 < e4.length; r7++) {
    let a3 = e4[r7], l4 = s18 / 8, u3 = o6 / 8;
    i12.fillRect(t3 + a3.x * l4, n4 + a3.y * u3, a3.w * l4, a3.h * u3);
  }
}
function Kr2(i12, e4, t3, n4, s18, o6) {
  let r7 = xn3.get(e4);
  r7 || (r7 = /* @__PURE__ */ new Map(), xn3.set(e4, r7));
  let a3 = i12.fillStyle;
  if (typeof a3 != "string") throw new Error(`Unexpected fillStyle type "${a3}"`);
  let l4 = r7.get(a3);
  if (!l4) {
    let u3 = e4[0].length, c4 = e4.length, d3 = i12.canvas.ownerDocument.createElement("canvas");
    d3.width = u3, d3.height = c4;
    let h4 = F3(d3.getContext("2d")), f3 = new ImageData(u3, c4), I2, L4, M4, q2;
    if (a3.startsWith("#")) I2 = parseInt(a3.slice(1, 3), 16), L4 = parseInt(a3.slice(3, 5), 16), M4 = parseInt(a3.slice(5, 7), 16), q2 = a3.length > 7 && parseInt(a3.slice(7, 9), 16) || 1;
    else if (a3.startsWith("rgba")) [I2, L4, M4, q2] = a3.substring(5, a3.length - 1).split(",").map((S4) => parseFloat(S4));
    else throw new Error(`Unexpected fillStyle color format "${a3}" when drawing pattern glyph`);
    for (let S4 = 0; S4 < c4; S4++) for (let W2 = 0; W2 < u3; W2++) f3.data[(S4 * u3 + W2) * 4] = I2, f3.data[(S4 * u3 + W2) * 4 + 1] = L4, f3.data[(S4 * u3 + W2) * 4 + 2] = M4, f3.data[(S4 * u3 + W2) * 4 + 3] = e4[S4][W2] * (q2 * 255);
    h4.putImageData(f3, 0, 0), l4 = F3(i12.createPattern(d3, null)), r7.set(a3, l4);
  }
  i12.fillStyle = l4, i12.fillRect(t3, n4, s18, o6);
}
function Vr2(i12, e4, t3, n4, s18, o6, r7) {
  i12.strokeStyle = i12.fillStyle;
  for (let [a3, l4] of Object.entries(e4)) {
    i12.beginPath(), i12.lineWidth = r7 * Number.parseInt(a3);
    let u3;
    if (typeof l4 == "function") {
      let d3 = 0.15 / o6 * s18;
      u3 = l4(0.15, d3);
    } else u3 = l4;
    for (let c4 of u3.split(" ")) {
      let d3 = c4[0], h4 = In3[d3];
      if (!h4) {
        console.error(`Could not find drawing instructions for "${d3}"`);
        continue;
      }
      let f3 = c4.substring(1).split(",");
      !f3[0] || !f3[1] || h4(i12, Ln2(f3, s18, o6, t3, n4, true, r7));
    }
    i12.stroke(), i12.closePath();
  }
}
function Cr3(i12, e4, t3, n4, s18, o6, r7, a3) {
  let l4 = new Path2D();
  l4.rect(t3, n4, s18, o6), i12.clip(l4), i12.beginPath();
  let u3 = r7 / 12;
  i12.lineWidth = a3 * u3;
  for (let c4 of e4.d.split(" ")) {
    let d3 = c4[0], h4 = In3[d3];
    if (!h4) {
      console.error(`Could not find drawing instructions for "${d3}"`);
      continue;
    }
    let f3 = c4.substring(1).split(",");
    !f3[0] || !f3[1] || h4(i12, Ln2(f3, s18, o6, t3, n4, false, a3, (e4.leftPadding ?? 0) * (u3 / 2), (e4.rightPadding ?? 0) * (u3 / 2)));
  }
  e4.type === 1 ? (i12.strokeStyle = i12.fillStyle, i12.stroke()) : i12.fill(), i12.closePath();
}
function En2(i12, e4, t3 = 0) {
  return Math.max(Math.min(i12, e4), t3);
}
function Ln2(i12, e4, t3, n4, s18, o6, r7, a3 = 0, l4 = 0) {
  let u3 = i12.map((c4) => parseFloat(c4) || parseInt(c4));
  if (u3.length < 2) throw new Error("Too few arguments for instruction");
  for (let c4 = 0; c4 < u3.length; c4 += 2) u3[c4] *= e4 - a3 * r7 - l4 * r7, o6 && u3[c4] !== 0 && (u3[c4] = En2(Math.round(u3[c4] + 0.5) - 0.5, e4, 0)), u3[c4] += n4 + a3 * r7;
  for (let c4 = 1; c4 < u3.length; c4 += 2) u3[c4] *= t3, o6 && u3[c4] !== 0 && (u3[c4] = En2(Math.round(u3[c4] + 0.5) - 0.5, t3, 0)), u3[c4] += s18;
  return u3;
}
function Di2(i12, e4, t3, n4) {
  let s18 = e4.rgba >>> 24, o6 = e4.rgba >>> 16 & 255, r7 = e4.rgba >>> 8 & 255, a3 = t3.rgba >>> 24, l4 = t3.rgba >>> 16 & 255, u3 = t3.rgba >>> 8 & 255, c4 = Math.floor((Math.abs(s18 - a3) + Math.abs(o6 - l4) + Math.abs(r7 - u3)) / 12), d3 = true;
  for (let h4 = 0; h4 < i12.data.length; h4 += 4) i12.data[h4] === s18 && i12.data[h4 + 1] === o6 && i12.data[h4 + 2] === r7 || n4 && Math.abs(i12.data[h4] - s18) + Math.abs(i12.data[h4 + 1] - o6) + Math.abs(i12.data[h4 + 2] - r7) < c4 ? i12.data[h4 + 3] = 0 : d3 = false;
  return d3;
}
function Jr2(i12) {
  for (let e4 = 0; e4 < i12.data.length; e4 += 4) if (i12.data[e4 + 3] > 0) return false;
  return true;
}
function Sn2(i12, e4, t3) {
  let n4 = i12.createElement("canvas");
  return n4.width = e4, n4.height = t3, n4;
}
function On(i12, e4, t3, n4, s18, o6, r7, a3) {
  let l4 = { foreground: o6.foreground, background: o6.background, cursor: Z2, cursorAccent: Z2, selectionForeground: Z2, selectionBackgroundTransparent: Z2, selectionBackgroundOpaque: Z2, selectionInactiveBackgroundTransparent: Z2, selectionInactiveBackgroundOpaque: Z2, overviewRulerBorder: Z2, scrollbarSliderBackground: Z2, scrollbarSliderHoverBackground: Z2, scrollbarSliderActiveBackground: Z2, ansi: o6.ansi.slice(), contrastCache: o6.contrastCache, halfContrastCache: o6.halfContrastCache };
  return { customGlyphs: s18.customGlyphs, devicePixelRatio: r7, deviceMaxTextureSize: a3, letterSpacing: s18.letterSpacing, lineHeight: s18.lineHeight, deviceCellWidth: i12, deviceCellHeight: e4, deviceCharWidth: t3, deviceCharHeight: n4, fontFamily: s18.fontFamily, fontSize: s18.fontSize, fontWeight: s18.fontWeight, fontWeightBold: s18.fontWeightBold, allowTransparency: s18.allowTransparency, drawBoldTextInBrightColors: s18.drawBoldTextInBrightColors, minimumContrastRatio: s18.minimumContrastRatio, colors: l4 };
}
function Mi2(i12, e4) {
  for (let t3 = 0; t3 < i12.colors.ansi.length; t3++) if (i12.colors.ansi[t3].rgba !== e4.colors.ansi[t3].rgba) return false;
  return i12.devicePixelRatio === e4.devicePixelRatio && i12.customGlyphs === e4.customGlyphs && i12.lineHeight === e4.lineHeight && i12.letterSpacing === e4.letterSpacing && i12.fontFamily === e4.fontFamily && i12.fontSize === e4.fontSize && i12.fontWeight === e4.fontWeight && i12.fontWeightBold === e4.fontWeightBold && i12.allowTransparency === e4.allowTransparency && i12.deviceCharWidth === e4.deviceCharWidth && i12.deviceCharHeight === e4.deviceCharHeight && i12.drawBoldTextInBrightColors === e4.drawBoldTextInBrightColors && i12.minimumContrastRatio === e4.minimumContrastRatio && i12.colors.foreground.rgba === e4.colors.foreground.rgba && i12.colors.background.rgba === e4.colors.background.rgba;
}
function Fn2(i12) {
  return (i12 & 50331648) === 16777216 || (i12 & 50331648) === 33554432;
}
function Nt3(i12, e4, t3, n4, s18, o6, r7, a3, l4) {
  let u3 = On(n4, s18, o6, r7, e4, t3, a3, l4);
  for (let h4 = 0; h4 < le2.length; h4++) {
    let f3 = le2[h4], I2 = f3.ownedBy.indexOf(i12);
    if (I2 >= 0) {
      if (Mi2(f3.config, u3)) return f3.atlas;
      f3.ownedBy.length === 1 ? (f3.atlas.dispose(), le2.splice(h4, 1)) : f3.ownedBy.splice(I2, 1);
      break;
    }
  }
  for (let h4 = 0; h4 < le2.length; h4++) {
    let f3 = le2[h4];
    if (Mi2(f3.config, u3)) return f3.ownedBy.push(i12), f3.atlas;
  }
  let c4 = i12._core, d3 = { atlas: new ae3(document, u3, c4.unicodeService), config: u3, ownedBy: [i12] };
  return le2.push(d3), d3.atlas;
}
function Ai2(i12) {
  for (let e4 = 0; e4 < le2.length; e4++) {
    let t3 = le2[e4].ownedBy.indexOf(i12);
    if (t3 !== -1) {
      le2[e4].ownedBy.length === 1 ? (le2[e4].atlas.dispose(), le2.splice(e4, 1)) : le2[e4].ownedBy.splice(t3, 1);
      break;
    }
  }
}
function Si2(i12, e4, t3) {
  let n4 = new e4.ResizeObserver((s18) => {
    let o6 = s18.find((l4) => l4.target === i12);
    if (!o6) return;
    if (!("devicePixelContentBoxSize" in o6)) {
      n4?.disconnect(), n4 = void 0;
      return;
    }
    let r7 = o6.devicePixelContentBoxSize[0].inlineSize, a3 = o6.devicePixelContentBoxSize[0].blockSize;
    r7 > 0 && a3 > 0 && t3(r7, a3);
  });
  try {
    n4.observe(i12, { box: ["device-pixel-content-box"] });
  } catch {
    n4.disconnect(), n4 = void 0;
  }
  return O2(() => n4?.disconnect());
}
function kn(i12) {
  return i12 > 65535 ? (i12 -= 65536, String.fromCharCode((i12 >> 10) + 55296) + String.fromCharCode(i12 % 1024 + 56320)) : String.fromCharCode(i12);
}
function $t2(i12, e4, t3) {
  let n4 = F3(i12.createProgram());
  if (i12.attachShader(n4, F3(Pn2(i12, i12.VERTEX_SHADER, e4))), i12.attachShader(n4, F3(Pn2(i12, i12.FRAGMENT_SHADER, t3))), i12.linkProgram(n4), i12.getProgramParameter(n4, i12.LINK_STATUS)) return n4;
  console.error(i12.getProgramInfoLog(n4)), i12.deleteProgram(n4);
}
function Pn2(i12, e4, t3) {
  let n4 = F3(i12.createShader(e4));
  if (i12.shaderSource(n4, t3), i12.compileShader(n4), i12.getShaderParameter(n4, i12.COMPILE_STATUS)) return n4;
  console.error(i12.getShaderInfoLog(n4)), i12.deleteShader(n4);
}
function Bn(i12, e4) {
  let t3 = Math.min(i12.length * 2, e4), n4 = new Float32Array(t3);
  for (let s18 = 0; s18 < i12.length; s18++) n4[s18] = i12[s18];
  return n4;
}
function ns2(i12) {
  let e4 = "";
  for (let t3 = 1; t3 < i12; t3++) e4 += ` else if (v_texpage == ${t3}) { outColor = texture(u_texture[${t3}], v_texcoord); }`;
  return `#version 300 es
precision lowp float;

in vec2 v_texcoord;
flat in int v_texpage;

uniform sampler2D u_texture[${i12}];

out vec4 outColor;

void main() {
  if (v_texpage == 0) {
    outColor = texture(u_texture[0], v_texcoord);
  } ${e4}
}`;
}
function Nn() {
  return new ki2();
}
function us2(i12, e4, t3) {
  typeof e4 == "string" && (e4 = i12.matchMedia(e4)), e4.addEventListener("change", t3);
}
function qn2() {
  return Yt2;
}
function sr2(i12) {
  return 55296 <= i12 && i12 <= 56319;
}
function Gi(i12) {
  return 56320 <= i12 && i12 <= 57343;
}
function or2(i12, e4) {
  return (i12 - 55296 << 10) + (e4 - 56320) + 65536;
}
function ur2(i12) {
  return Ki2(i12, 0);
}
function Ki2(i12, e4) {
  switch (typeof i12) {
    case "object":
      return i12 === null ? Le3(349, e4) : Array.isArray(i12) ? As(i12, e4) : Ss2(i12, e4);
    case "string":
      return cr3(i12, e4);
    case "boolean":
      return Ms2(i12, e4);
    case "number":
      return Le3(i12, e4);
    case "undefined":
      return Le3(937, e4);
    default:
      return Le3(617, e4);
  }
}
function Le3(i12, e4) {
  return (e4 << 5) - e4 + i12 | 0;
}
function Ms2(i12, e4) {
  return Le3(i12 ? 433 : 863, e4);
}
function cr3(i12, e4) {
  e4 = Le3(149417, e4);
  for (let t3 = 0, n4 = i12.length; t3 < n4; t3++) e4 = Le3(i12.charCodeAt(t3), e4);
  return e4;
}
function As(i12, e4) {
  return e4 = Le3(104579, e4), i12.reduce((t3, n4) => Ki2(n4, t3), e4);
}
function Ss2(i12, e4) {
  return e4 = Le3(181387, e4), Object.keys(i12).sort().reduce((t3, n4) => (t3 = cr3(n4, t3), Ki2(i12[n4], t3)), e4);
}
function $i2(i12, e4, t3 = 32) {
  let n4 = t3 - e4, s18 = ~((1 << n4) - 1);
  return (i12 << e4 | (s18 & i12) >>> n4) >>> 0;
}
function ar2(i12, e4 = 0, t3 = i12.byteLength, n4 = 0) {
  for (let s18 = 0; s18 < t3; s18++) i12[e4 + s18] = n4;
}
function Os2(i12, e4, t3 = "0") {
  for (; i12.length < e4; ) i12 = t3 + i12;
  return i12;
}
function ht(i12, e4 = 32) {
  return i12 instanceof ArrayBuffer ? Array.from(new Uint8Array(i12)).map((t3) => t3.toString(16).padStart(2, "0")).join("") : Os2((i12 >>> 0).toString(16), e4 / 4);
}
function li2(i12, e4, t3, n4) {
  return new Vi2(i12, e4, t3, n4);
}
function fr2(i12, e4, t3, ...n4) {
  let s18 = Bs2.exec(e4);
  if (!s18) throw new Error("Bad use of emmet");
  let o6 = s18[1] || "div", r7;
  return i12 !== "http://www.w3.org/1999/xhtml" ? r7 = document.createElementNS(i12, o6) : r7 = document.createElement(o6), s18[3] && (r7.id = s18[3]), s18[4] && (r7.className = s18[4].replace(/\./g, " ").trim()), t3 && Object.entries(t3).forEach(([a3, l4]) => {
    typeof l4 > "u" || (/^on\w+$/.test(a3) ? r7[a3] = l4 : a3 === "selected" ? l4 && r7.setAttribute(a3, "true") : r7.setAttribute(a3, l4));
  }), r7.append(...n4), r7;
}
function Ns(i12, e4, ...t3) {
  return fr2("http://www.w3.org/1999/xhtml", i12, e4, ...t3);
}
function mr2(i12, e4, t3 = 0) {
  return Math.max(Math.min(i12, e4), t3);
}
function pe3(i12) {
  if (zi2.has(i12)) return zi2.get(i12);
  let e4 = function(t3, n4, s18) {
    if (arguments.length !== 3) throw new Error("@IServiceName-decorator can only be used to decorate a parameter");
    Us2(e4, t3, s18);
  };
  return e4._id = i12, zi2.set(i12, e4), e4;
}
function Us2(i12, e4, t3) {
  e4[_r2] === e4 ? e4[br3].push({ id: i12, index: t3 }) : (e4[br3] = [{ id: i12, index: t3 }], e4[_r2] = e4);
}
function gr3(i12) {
  Tr2 = i12;
}
var Lr2, wr2, Yi2, Qi, pi, Rr2, fi, Ye3, bt3, vt3, en2, nn2, Be3, sn2, on2, rn2, Tt3, _i2, Ar2, Ne3, gt3, xt3, fe3, B5, be4, Lt3, Ze2, bi2, cn2, dn2, vi2, oo2, ao2, lo2, uo, pn2, K3, V4, C4, U2, Z2, X2, Ue3, Fr2, Y3, Te3, j3, z3, me2, ge3, Mt4, J2, Ti2, At4, gn3, St3, Hr, Wr2, Gr2, et, xn3, In3, Ot3, tt2, Ft2, gi2, xi2, wn2, he4, it2, He2, zr3, kt4, qr2, Dn3, jr2, ee3, We3, Ei2, Mn2, Bt2, yi2, nt3, Ii2, Li2, Xr2, Ge2, Yr2, Qr2, Pt3, D3, wi2, An, rt3, st3, ae3, ot3, le2, Ut2, Ht, at3, Gt2, Wt3, is2, De3, Ve2, rs2, H4, k4, Fi2, lt2, Kt2, ki2, Ce3, ze2, qe3, Ct3, Un2, Vt2, ss2, os2, Ee3, Pi2, as2, zt2, xe3, Hn2, Wn2, Gn2, $n2, Kn2, Vn2, qt2, jt3, Xt2, te2, Zt2, Qt2, Wa2, Ga2, je3, Cn2, ut3, zn2, Bi2, $a2, Ka2, Yt2, Xe3, Ui2, ni2, ti2, cs2, Xn2, Yn2, ds2, hs2, ps2, fs2, ei2, ii2, jn2, ms2, ye3, Ie3, re2, Qn2, _s2, Ni2, ri2, bs2, Va2, _e4, Me2, vs2, Ts2, Zn2, gs2, Ca2, za2, qa2, ja2, Ae3, xs2, dt2, Hi2, Jn2, er2, Es2, tr2, nr2, Is2, Wi2, Ls2, ws2, oi2, Rs2, Q4, ai2, fu, Fs2, mu, _u, bu, dr3, vu, Tu2, gu, xu2, Eu2, Vi2, ks2, hr3, pt4, ke3, yu2, Ps2, Bs2, ui2, Ci2, _r2, br3, zi2, Vu, Cu, zu, qu, ju, Xu, vr2, Yu, Qu, Zu, Hs2, Ws2, ci2, Tr2, xr2;
var init_addon_webgl = __esm({
  "node_modules/.pnpm/@xterm+addon-webgl@0.19.0/node_modules/@xterm/addon-webgl/lib/addon-webgl.mjs"() {
    Lr2 = Object.defineProperty;
    wr2 = Object.getOwnPropertyDescriptor;
    Yi2 = (i12, e4, t3, n4) => {
      for (var s18 = n4 > 1 ? void 0 : n4 ? wr2(e4, t3) : e4, o6 = i12.length - 1, r7; o6 >= 0; o6--) (r7 = i12[o6]) && (s18 = (n4 ? r7(e4, t3, s18) : r7(s18)) || s18);
      return n4 && s18 && Lr2(e4, t3, s18), s18;
    };
    Qi = (i12, e4) => (t3, n4) => e4(t3, n4, i12);
    pi = class {
      constructor() {
        this.listeners = [], this.unexpectedErrorHandler = function(e4) {
          setTimeout(() => {
            throw e4.stack ? bt3.isErrorNoTelemetry(e4) ? new bt3(e4.message + `

` + e4.stack) : new Error(e4.message + `

` + e4.stack) : e4;
          }, 0);
        };
      }
      addListener(e4) {
        return this.listeners.push(e4), () => {
          this._removeListener(e4);
        };
      }
      emit(e4) {
        this.listeners.forEach((t3) => {
          t3(e4);
        });
      }
      _removeListener(e4) {
        this.listeners.splice(this.listeners.indexOf(e4), 1);
      }
      setUnexpectedErrorHandler(e4) {
        this.unexpectedErrorHandler = e4;
      }
      getUnexpectedErrorHandler() {
        return this.unexpectedErrorHandler;
      }
      onUnexpectedError(e4) {
        this.unexpectedErrorHandler(e4), this.emit(e4);
      }
      onUnexpectedExternalError(e4) {
        this.unexpectedErrorHandler(e4);
      }
    };
    Rr2 = new pi();
    fi = "Canceled";
    Ye3 = class extends Error {
      constructor() {
        super(fi), this.name = this.message;
      }
    };
    bt3 = class i5 extends Error {
      constructor(e4) {
        super(e4), this.name = "CodeExpectedError";
      }
      static fromError(e4) {
        if (e4 instanceof i5) return e4;
        let t3 = new i5();
        return t3.message = e4.message, t3.stack = e4.stack, t3;
      }
      static isErrorNoTelemetry(e4) {
        return e4.name === "CodeExpectedError";
      }
    };
    vt3 = class vt4 {
      constructor(e4) {
        this._array = e4;
        this._findLastMonotonousLastIdx = 0;
      }
      findLastMonotonous(e4) {
        if (vt4.assertInvariants) {
          if (this._prevFindLastPredicate) {
            for (let n4 of this._array) if (this._prevFindLastPredicate(n4) && !e4(n4)) throw new Error("MonotonousArray: current predicate must be weaker than (or equal to) the previous predicate.");
          }
          this._prevFindLastPredicate = e4;
        }
        let t3 = Mr2(this._array, e4, this._findLastMonotonousLastIdx);
        return this._findLastMonotonousLastIdx = t3 + 1, t3 === -1 ? void 0 : this._array[t3];
      }
    };
    vt3.assertInvariants = false;
    ((a3) => {
      function i12(l4) {
        return l4 < 0;
      }
      a3.isLessThan = i12;
      function e4(l4) {
        return l4 <= 0;
      }
      a3.isLessThanOrEqual = e4;
      function t3(l4) {
        return l4 > 0;
      }
      a3.isGreaterThan = t3;
      function n4(l4) {
        return l4 === 0;
      }
      a3.isNeitherLessOrGreaterThan = n4, a3.greaterThan = 1, a3.lessThan = -1, a3.neitherLessOrGreaterThan = 0;
    })(en2 || (en2 = {}));
    nn2 = (i12, e4) => i12 - e4;
    Be3 = class Be4 {
      constructor(e4) {
        this.iterate = e4;
      }
      forEach(e4) {
        this.iterate((t3) => (e4(t3), true));
      }
      toArray() {
        let e4 = [];
        return this.iterate((t3) => (e4.push(t3), true)), e4;
      }
      filter(e4) {
        return new Be4((t3) => this.iterate((n4) => e4(n4) ? t3(n4) : true));
      }
      map(e4) {
        return new Be4((t3) => this.iterate((n4) => t3(e4(n4))));
      }
      some(e4) {
        let t3 = false;
        return this.iterate((n4) => (t3 = e4(n4), !t3)), t3;
      }
      findFirst(e4) {
        let t3;
        return this.iterate((n4) => e4(n4) ? (t3 = n4, false) : true), t3;
      }
      findLast(e4) {
        let t3;
        return this.iterate((n4) => (e4(n4) && (t3 = n4), true)), t3;
      }
      findLastMaxBy(e4) {
        let t3, n4 = true;
        return this.iterate((s18) => ((n4 || en2.isGreaterThan(e4(s18, t3))) && (n4 = false, t3 = s18), true)), t3;
      }
    };
    Be3.empty = new Be3((e4) => {
    });
    rn2 = class {
      constructor(e4, t3) {
        this.toKey = t3;
        this._map = /* @__PURE__ */ new Map();
        this[sn2] = "SetWithKey";
        for (let n4 of e4) this.add(n4);
      }
      get size() {
        return this._map.size;
      }
      add(e4) {
        let t3 = this.toKey(e4);
        return this._map.set(t3, e4), this;
      }
      delete(e4) {
        return this._map.delete(this.toKey(e4));
      }
      has(e4) {
        return this._map.has(this.toKey(e4));
      }
      *entries() {
        for (let e4 of this._map.values()) yield [e4, e4];
      }
      keys() {
        return this.values();
      }
      *values() {
        for (let e4 of this._map.values()) yield e4;
      }
      clear() {
        this._map.clear();
      }
      forEach(e4, t3) {
        this._map.forEach((n4) => e4.call(t3, n4, n4, this));
      }
      [(on2 = Symbol.iterator, sn2 = Symbol.toStringTag, on2)]() {
        return this.values();
      }
    };
    Tt3 = class {
      constructor() {
        this.map = /* @__PURE__ */ new Map();
      }
      add(e4, t3) {
        let n4 = this.map.get(e4);
        n4 || (n4 = /* @__PURE__ */ new Set(), this.map.set(e4, n4)), n4.add(t3);
      }
      delete(e4, t3) {
        let n4 = this.map.get(e4);
        n4 && (n4.delete(t3), n4.size === 0 && this.map.delete(e4));
      }
      forEach(e4, t3) {
        let n4 = this.map.get(e4);
        n4 && n4.forEach(t3);
      }
      get(e4) {
        let t3 = this.map.get(e4);
        return t3 || /* @__PURE__ */ new Set();
      }
    };
    ((W2) => {
      function i12(E2) {
        return E2 && typeof E2 == "object" && typeof E2[Symbol.iterator] == "function";
      }
      W2.is = i12;
      let e4 = Object.freeze([]);
      function t3() {
        return e4;
      }
      W2.empty = t3;
      function* n4(E2) {
        yield E2;
      }
      W2.single = n4;
      function s18(E2) {
        return i12(E2) ? E2 : n4(E2);
      }
      W2.wrap = s18;
      function o6(E2) {
        return E2 || e4;
      }
      W2.from = o6;
      function* r7(E2) {
        for (let y3 = E2.length - 1; y3 >= 0; y3--) yield E2[y3];
      }
      W2.reverse = r7;
      function a3(E2) {
        return !E2 || E2[Symbol.iterator]().next().done === true;
      }
      W2.isEmpty = a3;
      function l4(E2) {
        return E2[Symbol.iterator]().next().value;
      }
      W2.first = l4;
      function u3(E2, y3) {
        let w3 = 0;
        for (let G3 of E2) if (y3(G3, w3++)) return true;
        return false;
      }
      W2.some = u3;
      function c4(E2, y3) {
        for (let w3 of E2) if (y3(w3)) return w3;
      }
      W2.find = c4;
      function* d3(E2, y3) {
        for (let w3 of E2) y3(w3) && (yield w3);
      }
      W2.filter = d3;
      function* h4(E2, y3) {
        let w3 = 0;
        for (let G3 of E2) yield y3(G3, w3++);
      }
      W2.map = h4;
      function* f3(E2, y3) {
        let w3 = 0;
        for (let G3 of E2) yield* y3(G3, w3++);
      }
      W2.flatMap = f3;
      function* I2(...E2) {
        for (let y3 of E2) yield* y3;
      }
      W2.concat = I2;
      function L4(E2, y3, w3) {
        let G3 = w3;
        for (let ue4 of E2) G3 = y3(G3, ue4);
        return G3;
      }
      W2.reduce = L4;
      function* M4(E2, y3, w3 = E2.length) {
        for (y3 < 0 && (y3 += E2.length), w3 < 0 ? w3 += E2.length : w3 > E2.length && (w3 = E2.length); y3 < w3; y3++) yield E2[y3];
      }
      W2.slice = M4;
      function q2(E2, y3 = Number.POSITIVE_INFINITY) {
        let w3 = [];
        if (y3 === 0) return [w3, E2];
        let G3 = E2[Symbol.iterator]();
        for (let ue4 = 0; ue4 < y3; ue4++) {
          let Se2 = G3.next();
          if (Se2.done) return [w3, W2.empty()];
          w3.push(Se2.value);
        }
        return [w3, { [Symbol.iterator]() {
          return G3;
        } }];
      }
      W2.consume = q2;
      async function S4(E2) {
        let y3 = [];
        for await (let w3 of E2) y3.push(w3);
        return Promise.resolve(y3);
      }
      W2.asyncToArray = S4;
    })(_i2 || (_i2 = {}));
    Ar2 = false;
    Ne3 = null;
    gt3 = class gt4 {
      constructor() {
        this.livingDisposables = /* @__PURE__ */ new Map();
      }
      getDisposableData(e4) {
        let t3 = this.livingDisposables.get(e4);
        return t3 || (t3 = { parent: null, source: null, isSingleton: false, value: e4, idx: gt4.idx++ }, this.livingDisposables.set(e4, t3)), t3;
      }
      trackDisposable(e4) {
        let t3 = this.getDisposableData(e4);
        t3.source || (t3.source = new Error().stack);
      }
      setParent(e4, t3) {
        let n4 = this.getDisposableData(e4);
        n4.parent = t3;
      }
      markAsDisposed(e4) {
        this.livingDisposables.delete(e4);
      }
      markAsSingleton(e4) {
        this.getDisposableData(e4).isSingleton = true;
      }
      getRootParent(e4, t3) {
        let n4 = t3.get(e4);
        if (n4) return n4;
        let s18 = e4.parent ? this.getRootParent(this.getDisposableData(e4.parent), t3) : e4;
        return t3.set(e4, s18), s18;
      }
      getTrackedDisposables() {
        let e4 = /* @__PURE__ */ new Map();
        return [...this.livingDisposables.entries()].filter(([, n4]) => n4.source !== null && !this.getRootParent(n4, e4).isSingleton).flatMap(([n4]) => n4);
      }
      computeLeakingDisposables(e4 = 10, t3) {
        let n4;
        if (t3) n4 = t3;
        else {
          let l4 = /* @__PURE__ */ new Map(), u3 = [...this.livingDisposables.values()].filter((d3) => d3.source !== null && !this.getRootParent(d3, l4).isSingleton);
          if (u3.length === 0) return;
          let c4 = new Set(u3.map((d3) => d3.value));
          if (n4 = u3.filter((d3) => !(d3.parent && c4.has(d3.parent))), n4.length === 0) throw new Error("There are cyclic diposable chains!");
        }
        if (!n4) return;
        function s18(l4) {
          function u3(d3, h4) {
            for (; d3.length > 0 && h4.some((f3) => typeof f3 == "string" ? f3 === d3[0] : d3[0].match(f3)); ) d3.shift();
          }
          let c4 = l4.source.split(`
`).map((d3) => d3.trim().replace("at ", "")).filter((d3) => d3 !== "");
          return u3(c4, ["Error", /^trackDisposable \(.*\)$/, /^DisposableTracker.trackDisposable \(.*\)$/]), c4.reverse();
        }
        let o6 = new Tt3();
        for (let l4 of n4) {
          let u3 = s18(l4);
          for (let c4 = 0; c4 <= u3.length; c4++) o6.add(u3.slice(0, c4).join(`
`), l4);
        }
        n4.sort(tn2((l4) => l4.idx, nn2));
        let r7 = "", a3 = 0;
        for (let l4 of n4.slice(0, e4)) {
          a3++;
          let u3 = s18(l4), c4 = [];
          for (let d3 = 0; d3 < u3.length; d3++) {
            let h4 = u3[d3];
            h4 = `(shared with ${o6.get(u3.slice(0, d3 + 1).join(`
`)).size}/${n4.length} leaks) at ${h4}`;
            let I2 = o6.get(u3.slice(0, d3).join(`
`)), L4 = an2([...I2].map((M4) => s18(M4)[d3]), (M4) => M4);
            delete L4[u3[d3]];
            for (let [M4, q2] of Object.entries(L4)) c4.unshift(`    - stacktraces of ${q2.length} other leaks continue with ${M4}`);
            c4.unshift(h4);
          }
          r7 += `


==================== Leaking disposable ${a3}/${n4.length}: ${l4.value.constructor.name} ====================
${c4.join(`
`)}
============================================================

`;
        }
        return n4.length > e4 && (r7 += `


... and ${n4.length - e4} more leaking disposables

`), { leaks: n4, details: r7 };
      }
    };
    gt3.idx = 0;
    if (Ar2) {
      let i12 = "__is_disposable_tracked__";
      Sr2(new class {
        trackDisposable(e4) {
          let t3 = new Error("Potentially leaked disposable").stack;
          setTimeout(() => {
            e4[i12] || console.log(t3);
          }, 3e3);
        }
        setParent(e4, t3) {
          if (e4 && e4 !== B5.None) try {
            e4[i12] = true;
          } catch {
          }
        }
        markAsDisposed(e4) {
          if (e4 && e4 !== B5.None) try {
            e4[i12] = true;
          } catch {
          }
        }
        markAsSingleton(e4) {
        }
      }());
    }
    xt3 = class xt4 {
      constructor() {
        this._toDispose = /* @__PURE__ */ new Set();
        this._isDisposed = false;
        Et2(this);
      }
      dispose() {
        this._isDisposed || (yt2(this), this._isDisposed = true, this.clear());
      }
      get isDisposed() {
        return this._isDisposed;
      }
      clear() {
        if (this._toDispose.size !== 0) try {
          un3(this._toDispose);
        } finally {
          this._toDispose.clear();
        }
      }
      add(e4) {
        if (!e4) return e4;
        if (e4 === this) throw new Error("Cannot register a disposable on itself!");
        return Qe2(e4, this), this._isDisposed ? xt4.DISABLE_DISPOSED_WARNING || console.warn(new Error("Trying to add a disposable to a DisposableStore that has already been disposed of. The added object will be leaked!").stack) : this._toDispose.add(e4), e4;
      }
      delete(e4) {
        if (e4) {
          if (e4 === this) throw new Error("Cannot dispose a disposable on itself!");
          this._toDispose.delete(e4), e4.dispose();
        }
      }
      deleteAndLeak(e4) {
        e4 && this._toDispose.has(e4) && (this._toDispose.delete(e4), Qe2(e4, null));
      }
    };
    xt3.DISABLE_DISPOSED_WARNING = false;
    fe3 = xt3;
    B5 = class {
      constructor() {
        this._store = new fe3();
        Et2(this), Qe2(this._store, this);
      }
      dispose() {
        yt2(this), this._store.dispose();
      }
      _register(e4) {
        if (e4 === this) throw new Error("Cannot register a disposable on itself!");
        return this._store.add(e4);
      }
    };
    B5.None = Object.freeze({ dispose() {
    } });
    be4 = class {
      constructor() {
        this._isDisposed = false;
        Et2(this);
      }
      get value() {
        return this._isDisposed ? void 0 : this._value;
      }
      set value(e4) {
        this._isDisposed || e4 === this._value || (this._value?.dispose(), e4 && Qe2(e4, this), this._value = e4);
      }
      clear() {
        this.value = void 0;
      }
      dispose() {
        this._isDisposed = true, yt2(this), this._value?.dispose(), this._value = void 0;
      }
      clearAndLeak() {
        let e4 = this._value;
        return this._value = void 0, e4 && Qe2(e4, null), e4;
      }
    };
    Lt3 = typeof process < "u" && "title" in process;
    Ze2 = Lt3 ? "node" : navigator.userAgent;
    bi2 = Lt3 ? "node" : navigator.platform;
    cn2 = Ze2.includes("Firefox");
    dn2 = Ze2.includes("Edge");
    vi2 = /^((?!chrome|android).)*safari/i.test(Ze2);
    oo2 = ["Macintosh", "MacIntel", "MacPPC", "Mac68K"].includes(bi2);
    ao2 = ["Windows", "Win16", "Win32", "WinCE"].includes(bi2);
    lo2 = bi2.indexOf("Linux") >= 0;
    uo = /\bCrOS\b/.test(Ze2);
    pn2 = "";
    K3 = 0;
    V4 = 0;
    C4 = 0;
    U2 = 0;
    Z2 = { css: "#00000000", rgba: 0 };
    ((n4) => {
      function i12(s18, o6, r7, a3) {
        return a3 !== void 0 ? `#${Oe2(s18)}${Oe2(o6)}${Oe2(r7)}${Oe2(a3)}` : `#${Oe2(s18)}${Oe2(o6)}${Oe2(r7)}`;
      }
      n4.toCss = i12;
      function e4(s18, o6, r7, a3 = 255) {
        return (s18 << 24 | o6 << 16 | r7 << 8 | a3) >>> 0;
      }
      n4.toRgba = e4;
      function t3(s18, o6, r7, a3) {
        return { css: n4.toCss(s18, o6, r7, a3), rgba: n4.toRgba(s18, o6, r7, a3) };
      }
      n4.toColor = t3;
    })(X2 || (X2 = {}));
    ((a3) => {
      function i12(l4, u3) {
        if (U2 = (u3.rgba & 255) / 255, U2 === 1) return { css: u3.css, rgba: u3.rgba };
        let c4 = u3.rgba >> 24 & 255, d3 = u3.rgba >> 16 & 255, h4 = u3.rgba >> 8 & 255, f3 = l4.rgba >> 24 & 255, I2 = l4.rgba >> 16 & 255, L4 = l4.rgba >> 8 & 255;
        K3 = f3 + Math.round((c4 - f3) * U2), V4 = I2 + Math.round((d3 - I2) * U2), C4 = L4 + Math.round((h4 - L4) * U2);
        let M4 = X2.toCss(K3, V4, C4), q2 = X2.toRgba(K3, V4, C4);
        return { css: M4, rgba: q2 };
      }
      a3.blend = i12;
      function e4(l4) {
        return (l4.rgba & 255) === 255;
      }
      a3.isOpaque = e4;
      function t3(l4, u3, c4) {
        let d3 = Te3.ensureContrastRatio(l4.rgba, u3.rgba, c4);
        if (d3) return X2.toColor(d3 >> 24 & 255, d3 >> 16 & 255, d3 >> 8 & 255);
      }
      a3.ensureContrastRatio = t3;
      function n4(l4) {
        let u3 = (l4.rgba | 255) >>> 0;
        return [K3, V4, C4] = Te3.toChannels(u3), { css: X2.toCss(K3, V4, C4), rgba: u3 };
      }
      a3.opaque = n4;
      function s18(l4, u3) {
        return U2 = Math.round(u3 * 255), [K3, V4, C4] = Te3.toChannels(l4.rgba), { css: X2.toCss(K3, V4, C4, U2), rgba: X2.toRgba(K3, V4, C4, U2) };
      }
      a3.opacity = s18;
      function o6(l4, u3) {
        return U2 = l4.rgba & 255, s18(l4, U2 * u3 / 255);
      }
      a3.multiplyOpacity = o6;
      function r7(l4) {
        return [l4.rgba >> 24 & 255, l4.rgba >> 16 & 255, l4.rgba >> 8 & 255];
      }
      a3.toColorRGB = r7;
    })(Ue3 || (Ue3 = {}));
    ((n4) => {
      let i12, e4;
      try {
        let s18 = document.createElement("canvas");
        s18.width = 1, s18.height = 1;
        let o6 = s18.getContext("2d", { willReadFrequently: true });
        o6 && (i12 = o6, i12.globalCompositeOperation = "copy", e4 = i12.createLinearGradient(0, 0, 1, 1));
      } catch {
      }
      function t3(s18) {
        if (s18.match(/#[\da-f]{3,8}/i)) switch (s18.length) {
          case 4:
            return K3 = parseInt(s18.slice(1, 2).repeat(2), 16), V4 = parseInt(s18.slice(2, 3).repeat(2), 16), C4 = parseInt(s18.slice(3, 4).repeat(2), 16), X2.toColor(K3, V4, C4);
          case 5:
            return K3 = parseInt(s18.slice(1, 2).repeat(2), 16), V4 = parseInt(s18.slice(2, 3).repeat(2), 16), C4 = parseInt(s18.slice(3, 4).repeat(2), 16), U2 = parseInt(s18.slice(4, 5).repeat(2), 16), X2.toColor(K3, V4, C4, U2);
          case 7:
            return { css: s18, rgba: (parseInt(s18.slice(1), 16) << 8 | 255) >>> 0 };
          case 9:
            return { css: s18, rgba: parseInt(s18.slice(1), 16) >>> 0 };
        }
        let o6 = s18.match(/rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*(,\s*(0|1|\d?\.(\d+))\s*)?\)/);
        if (o6) return K3 = parseInt(o6[1]), V4 = parseInt(o6[2]), C4 = parseInt(o6[3]), U2 = Math.round((o6[5] === void 0 ? 1 : parseFloat(o6[5])) * 255), X2.toColor(K3, V4, C4, U2);
        if (!i12 || !e4) throw new Error("css.toColor: Unsupported css format");
        if (i12.fillStyle = e4, i12.fillStyle = s18, typeof i12.fillStyle != "string") throw new Error("css.toColor: Unsupported css format");
        if (i12.fillRect(0, 0, 1, 1), [K3, V4, C4, U2] = i12.getImageData(0, 0, 1, 1).data, U2 !== 255) throw new Error("css.toColor: Unsupported css format");
        return { rgba: X2.toRgba(K3, V4, C4, U2), css: s18 };
      }
      n4.toColor = t3;
    })(Fr2 || (Fr2 = {}));
    ((t3) => {
      function i12(n4) {
        return e4(n4 >> 16 & 255, n4 >> 8 & 255, n4 & 255);
      }
      t3.relativeLuminance = i12;
      function e4(n4, s18, o6) {
        let r7 = n4 / 255, a3 = s18 / 255, l4 = o6 / 255, u3 = r7 <= 0.03928 ? r7 / 12.92 : Math.pow((r7 + 0.055) / 1.055, 2.4), c4 = a3 <= 0.03928 ? a3 / 12.92 : Math.pow((a3 + 0.055) / 1.055, 2.4), d3 = l4 <= 0.03928 ? l4 / 12.92 : Math.pow((l4 + 0.055) / 1.055, 2.4);
        return u3 * 0.2126 + c4 * 0.7152 + d3 * 0.0722;
      }
      t3.relativeLuminance2 = e4;
    })(Y3 || (Y3 = {}));
    ((o6) => {
      function i12(r7, a3) {
        if (U2 = (a3 & 255) / 255, U2 === 1) return a3;
        let l4 = a3 >> 24 & 255, u3 = a3 >> 16 & 255, c4 = a3 >> 8 & 255, d3 = r7 >> 24 & 255, h4 = r7 >> 16 & 255, f3 = r7 >> 8 & 255;
        return K3 = d3 + Math.round((l4 - d3) * U2), V4 = h4 + Math.round((u3 - h4) * U2), C4 = f3 + Math.round((c4 - f3) * U2), X2.toRgba(K3, V4, C4);
      }
      o6.blend = i12;
      function e4(r7, a3, l4) {
        let u3 = Y3.relativeLuminance(r7 >> 8), c4 = Y3.relativeLuminance(a3 >> 8);
        if (ve3(u3, c4) < l4) {
          if (c4 < u3) {
            let I2 = t3(r7, a3, l4), L4 = ve3(u3, Y3.relativeLuminance(I2 >> 8));
            if (L4 < l4) {
              let M4 = n4(r7, a3, l4), q2 = ve3(u3, Y3.relativeLuminance(M4 >> 8));
              return L4 > q2 ? I2 : M4;
            }
            return I2;
          }
          let h4 = n4(r7, a3, l4), f3 = ve3(u3, Y3.relativeLuminance(h4 >> 8));
          if (f3 < l4) {
            let I2 = t3(r7, a3, l4), L4 = ve3(u3, Y3.relativeLuminance(I2 >> 8));
            return f3 > L4 ? h4 : I2;
          }
          return h4;
        }
      }
      o6.ensureContrastRatio = e4;
      function t3(r7, a3, l4) {
        let u3 = r7 >> 24 & 255, c4 = r7 >> 16 & 255, d3 = r7 >> 8 & 255, h4 = a3 >> 24 & 255, f3 = a3 >> 16 & 255, I2 = a3 >> 8 & 255, L4 = ve3(Y3.relativeLuminance2(h4, f3, I2), Y3.relativeLuminance2(u3, c4, d3));
        for (; L4 < l4 && (h4 > 0 || f3 > 0 || I2 > 0); ) h4 -= Math.max(0, Math.ceil(h4 * 0.1)), f3 -= Math.max(0, Math.ceil(f3 * 0.1)), I2 -= Math.max(0, Math.ceil(I2 * 0.1)), L4 = ve3(Y3.relativeLuminance2(h4, f3, I2), Y3.relativeLuminance2(u3, c4, d3));
        return (h4 << 24 | f3 << 16 | I2 << 8 | 255) >>> 0;
      }
      o6.reduceLuminance = t3;
      function n4(r7, a3, l4) {
        let u3 = r7 >> 24 & 255, c4 = r7 >> 16 & 255, d3 = r7 >> 8 & 255, h4 = a3 >> 24 & 255, f3 = a3 >> 16 & 255, I2 = a3 >> 8 & 255, L4 = ve3(Y3.relativeLuminance2(h4, f3, I2), Y3.relativeLuminance2(u3, c4, d3));
        for (; L4 < l4 && (h4 < 255 || f3 < 255 || I2 < 255); ) h4 = Math.min(255, h4 + Math.ceil((255 - h4) * 0.1)), f3 = Math.min(255, f3 + Math.ceil((255 - f3) * 0.1)), I2 = Math.min(255, I2 + Math.ceil((255 - I2) * 0.1)), L4 = ve3(Y3.relativeLuminance2(h4, f3, I2), Y3.relativeLuminance2(u3, c4, d3));
        return (h4 << 24 | f3 << 16 | I2 << 8 | 255) >>> 0;
      }
      o6.increaseLuminance = n4;
      function s18(r7) {
        return [r7 >> 24 & 255, r7 >> 16 & 255, r7 >> 8 & 255, r7 & 255];
      }
      o6.toChannels = s18;
    })(Te3 || (Te3 = {}));
    j3 = 0;
    z3 = 0;
    me2 = false;
    ge3 = false;
    Mt4 = false;
    Ti2 = 0;
    At4 = class {
      constructor(e4, t3, n4, s18, o6, r7) {
        this._terminal = e4;
        this._optionService = t3;
        this._selectionRenderModel = n4;
        this._decorationService = s18;
        this._coreBrowserService = o6;
        this._themeService = r7;
        this.result = { fg: 0, bg: 0, ext: 0 };
      }
      resolve(e4, t3, n4, s18) {
        if (this.result.bg = e4.bg, this.result.fg = e4.fg, this.result.ext = e4.bg & 268435456 ? e4.extended.ext : 0, z3 = 0, j3 = 0, ge3 = false, me2 = false, Mt4 = false, J2 = this._themeService.colors, Ti2 = 0, e4.getCode() !== 0 && e4.extended.underlineStyle === 4) {
          let r7 = Math.max(1, Math.floor(this._optionService.rawOptions.fontSize * this._coreBrowserService.dpr / 15));
          Ti2 = t3 * s18 % (Math.round(r7) * 2);
        }
        if (this._decorationService.forEachDecorationAtCell(t3, n4, "bottom", (r7) => {
          r7.backgroundColorRGB && (z3 = r7.backgroundColorRGB.rgba >> 8 & 16777215, ge3 = true), r7.foregroundColorRGB && (j3 = r7.foregroundColorRGB.rgba >> 8 & 16777215, me2 = true);
        }), Mt4 = this._selectionRenderModel.isCellSelected(this._terminal, t3, n4), Mt4) {
          if (this.result.fg & 67108864 || (this.result.bg & 50331648) !== 0) {
            if (this.result.fg & 67108864) switch (this.result.fg & 50331648) {
              case 16777216:
              case 33554432:
                z3 = this._themeService.colors.ansi[this.result.fg & 255].rgba;
                break;
              case 50331648:
                z3 = (this.result.fg & 16777215) << 8 | 255;
                break;
              case 0:
              default:
                z3 = this._themeService.colors.foreground.rgba;
            }
            else switch (this.result.bg & 50331648) {
              case 16777216:
              case 33554432:
                z3 = this._themeService.colors.ansi[this.result.bg & 255].rgba;
                break;
              case 50331648:
                z3 = (this.result.bg & 16777215) << 8 | 255;
                break;
            }
            z3 = Te3.blend(z3, (this._coreBrowserService.isFocused ? J2.selectionBackgroundOpaque : J2.selectionInactiveBackgroundOpaque).rgba & 4294967040 | 128) >> 8 & 16777215;
          } else z3 = (this._coreBrowserService.isFocused ? J2.selectionBackgroundOpaque : J2.selectionInactiveBackgroundOpaque).rgba >> 8 & 16777215;
          if (ge3 = true, J2.selectionForeground && (j3 = J2.selectionForeground.rgba >> 8 & 16777215, me2 = true), Dt3(e4.getCode())) {
            if (this.result.fg & 67108864 && (this.result.bg & 50331648) === 0) j3 = (this._coreBrowserService.isFocused ? J2.selectionBackgroundOpaque : J2.selectionInactiveBackgroundOpaque).rgba >> 8 & 16777215;
            else {
              if (this.result.fg & 67108864) switch (this.result.bg & 50331648) {
                case 16777216:
                case 33554432:
                  j3 = this._themeService.colors.ansi[this.result.bg & 255].rgba;
                  break;
                case 50331648:
                  j3 = (this.result.bg & 16777215) << 8 | 255;
                  break;
              }
              else switch (this.result.fg & 50331648) {
                case 16777216:
                case 33554432:
                  j3 = this._themeService.colors.ansi[this.result.fg & 255].rgba;
                  break;
                case 50331648:
                  j3 = (this.result.fg & 16777215) << 8 | 255;
                  break;
                case 0:
                default:
                  j3 = this._themeService.colors.foreground.rgba;
              }
              j3 = Te3.blend(j3, (this._coreBrowserService.isFocused ? J2.selectionBackgroundOpaque : J2.selectionInactiveBackgroundOpaque).rgba & 4294967040 | 128) >> 8 & 16777215;
            }
            me2 = true;
          }
        }
        this._decorationService.forEachDecorationAtCell(t3, n4, "top", (r7) => {
          r7.backgroundColorRGB && (z3 = r7.backgroundColorRGB.rgba >> 8 & 16777215, ge3 = true), r7.foregroundColorRGB && (j3 = r7.foregroundColorRGB.rgba >> 8 & 16777215, me2 = true);
        }), ge3 && (Mt4 ? z3 = e4.bg & -16777216 & -134217729 | z3 | 50331648 : z3 = e4.bg & -16777216 | z3 | 50331648), me2 && (j3 = e4.fg & -16777216 & -67108865 | j3 | 50331648), this.result.fg & 67108864 && (ge3 && !me2 && ((this.result.bg & 50331648) === 0 ? j3 = this.result.fg & -134217728 | J2.background.rgba >> 8 & 16777215 & 16777215 | 50331648 : j3 = this.result.fg & -134217728 | this.result.bg & 67108863, me2 = true), !ge3 && me2 && ((this.result.fg & 50331648) === 0 ? z3 = this.result.bg & -67108864 | J2.foreground.rgba >> 8 & 16777215 & 16777215 | 50331648 : z3 = this.result.bg & -67108864 | this.result.fg & 67108863, ge3 = true)), J2 = void 0, this.result.bg = ge3 ? z3 : this.result.bg, this.result.fg = me2 ? j3 : this.result.fg, this.result.ext &= 536870911, this.result.ext |= Ti2 << 29 & 3758096384;
      }
    };
    gn3 = 0.5;
    St3 = cn2 || dn2 ? "bottom" : "ideographic";
    Hr = { "\u2580": [{ x: 0, y: 0, w: 8, h: 4 }], "\u2581": [{ x: 0, y: 7, w: 8, h: 1 }], "\u2582": [{ x: 0, y: 6, w: 8, h: 2 }], "\u2583": [{ x: 0, y: 5, w: 8, h: 3 }], "\u2584": [{ x: 0, y: 4, w: 8, h: 4 }], "\u2585": [{ x: 0, y: 3, w: 8, h: 5 }], "\u2586": [{ x: 0, y: 2, w: 8, h: 6 }], "\u2587": [{ x: 0, y: 1, w: 8, h: 7 }], "\u2588": [{ x: 0, y: 0, w: 8, h: 8 }], "\u2589": [{ x: 0, y: 0, w: 7, h: 8 }], "\u258A": [{ x: 0, y: 0, w: 6, h: 8 }], "\u258B": [{ x: 0, y: 0, w: 5, h: 8 }], "\u258C": [{ x: 0, y: 0, w: 4, h: 8 }], "\u258D": [{ x: 0, y: 0, w: 3, h: 8 }], "\u258E": [{ x: 0, y: 0, w: 2, h: 8 }], "\u258F": [{ x: 0, y: 0, w: 1, h: 8 }], "\u2590": [{ x: 4, y: 0, w: 4, h: 8 }], "\u2594": [{ x: 0, y: 0, w: 8, h: 1 }], "\u2595": [{ x: 7, y: 0, w: 1, h: 8 }], "\u2596": [{ x: 0, y: 4, w: 4, h: 4 }], "\u2597": [{ x: 4, y: 4, w: 4, h: 4 }], "\u2598": [{ x: 0, y: 0, w: 4, h: 4 }], "\u2599": [{ x: 0, y: 0, w: 4, h: 8 }, { x: 0, y: 4, w: 8, h: 4 }], "\u259A": [{ x: 0, y: 0, w: 4, h: 4 }, { x: 4, y: 4, w: 4, h: 4 }], "\u259B": [{ x: 0, y: 0, w: 4, h: 8 }, { x: 4, y: 0, w: 4, h: 4 }], "\u259C": [{ x: 0, y: 0, w: 8, h: 4 }, { x: 4, y: 0, w: 4, h: 8 }], "\u259D": [{ x: 4, y: 0, w: 4, h: 4 }], "\u259E": [{ x: 4, y: 0, w: 4, h: 4 }, { x: 0, y: 4, w: 4, h: 4 }], "\u259F": [{ x: 4, y: 0, w: 4, h: 8 }, { x: 0, y: 4, w: 8, h: 4 }], "\u{1FB70}": [{ x: 1, y: 0, w: 1, h: 8 }], "\u{1FB71}": [{ x: 2, y: 0, w: 1, h: 8 }], "\u{1FB72}": [{ x: 3, y: 0, w: 1, h: 8 }], "\u{1FB73}": [{ x: 4, y: 0, w: 1, h: 8 }], "\u{1FB74}": [{ x: 5, y: 0, w: 1, h: 8 }], "\u{1FB75}": [{ x: 6, y: 0, w: 1, h: 8 }], "\u{1FB76}": [{ x: 0, y: 1, w: 8, h: 1 }], "\u{1FB77}": [{ x: 0, y: 2, w: 8, h: 1 }], "\u{1FB78}": [{ x: 0, y: 3, w: 8, h: 1 }], "\u{1FB79}": [{ x: 0, y: 4, w: 8, h: 1 }], "\u{1FB7A}": [{ x: 0, y: 5, w: 8, h: 1 }], "\u{1FB7B}": [{ x: 0, y: 6, w: 8, h: 1 }], "\u{1FB7C}": [{ x: 0, y: 0, w: 1, h: 8 }, { x: 0, y: 7, w: 8, h: 1 }], "\u{1FB7D}": [{ x: 0, y: 0, w: 1, h: 8 }, { x: 0, y: 0, w: 8, h: 1 }], "\u{1FB7E}": [{ x: 7, y: 0, w: 1, h: 8 }, { x: 0, y: 0, w: 8, h: 1 }], "\u{1FB7F}": [{ x: 7, y: 0, w: 1, h: 8 }, { x: 0, y: 7, w: 8, h: 1 }], "\u{1FB80}": [{ x: 0, y: 0, w: 8, h: 1 }, { x: 0, y: 7, w: 8, h: 1 }], "\u{1FB81}": [{ x: 0, y: 0, w: 8, h: 1 }, { x: 0, y: 2, w: 8, h: 1 }, { x: 0, y: 4, w: 8, h: 1 }, { x: 0, y: 7, w: 8, h: 1 }], "\u{1FB82}": [{ x: 0, y: 0, w: 8, h: 2 }], "\u{1FB83}": [{ x: 0, y: 0, w: 8, h: 3 }], "\u{1FB84}": [{ x: 0, y: 0, w: 8, h: 5 }], "\u{1FB85}": [{ x: 0, y: 0, w: 8, h: 6 }], "\u{1FB86}": [{ x: 0, y: 0, w: 8, h: 7 }], "\u{1FB87}": [{ x: 6, y: 0, w: 2, h: 8 }], "\u{1FB88}": [{ x: 5, y: 0, w: 3, h: 8 }], "\u{1FB89}": [{ x: 3, y: 0, w: 5, h: 8 }], "\u{1FB8A}": [{ x: 2, y: 0, w: 6, h: 8 }], "\u{1FB8B}": [{ x: 1, y: 0, w: 7, h: 8 }], "\u{1FB95}": [{ x: 0, y: 0, w: 2, h: 2 }, { x: 4, y: 0, w: 2, h: 2 }, { x: 2, y: 2, w: 2, h: 2 }, { x: 6, y: 2, w: 2, h: 2 }, { x: 0, y: 4, w: 2, h: 2 }, { x: 4, y: 4, w: 2, h: 2 }, { x: 2, y: 6, w: 2, h: 2 }, { x: 6, y: 6, w: 2, h: 2 }], "\u{1FB96}": [{ x: 2, y: 0, w: 2, h: 2 }, { x: 6, y: 0, w: 2, h: 2 }, { x: 0, y: 2, w: 2, h: 2 }, { x: 4, y: 2, w: 2, h: 2 }, { x: 2, y: 4, w: 2, h: 2 }, { x: 6, y: 4, w: 2, h: 2 }, { x: 0, y: 6, w: 2, h: 2 }, { x: 4, y: 6, w: 2, h: 2 }], "\u{1FB97}": [{ x: 0, y: 2, w: 8, h: 2 }, { x: 0, y: 6, w: 8, h: 2 }] };
    Wr2 = { "\u2591": [[1, 0, 0, 0], [0, 0, 0, 0], [0, 0, 1, 0], [0, 0, 0, 0]], "\u2592": [[1, 0], [0, 0], [0, 1], [0, 0]], "\u2593": [[0, 1], [1, 1], [1, 0], [1, 1]] };
    Gr2 = { "\u2500": { 1: "M0,.5 L1,.5" }, "\u2501": { 3: "M0,.5 L1,.5" }, "\u2502": { 1: "M.5,0 L.5,1" }, "\u2503": { 3: "M.5,0 L.5,1" }, "\u250C": { 1: "M0.5,1 L.5,.5 L1,.5" }, "\u250F": { 3: "M0.5,1 L.5,.5 L1,.5" }, "\u2510": { 1: "M0,.5 L.5,.5 L.5,1" }, "\u2513": { 3: "M0,.5 L.5,.5 L.5,1" }, "\u2514": { 1: "M.5,0 L.5,.5 L1,.5" }, "\u2517": { 3: "M.5,0 L.5,.5 L1,.5" }, "\u2518": { 1: "M.5,0 L.5,.5 L0,.5" }, "\u251B": { 3: "M.5,0 L.5,.5 L0,.5" }, "\u251C": { 1: "M.5,0 L.5,1 M.5,.5 L1,.5" }, "\u2523": { 3: "M.5,0 L.5,1 M.5,.5 L1,.5" }, "\u2524": { 1: "M.5,0 L.5,1 M.5,.5 L0,.5" }, "\u252B": { 3: "M.5,0 L.5,1 M.5,.5 L0,.5" }, "\u252C": { 1: "M0,.5 L1,.5 M.5,.5 L.5,1" }, "\u2533": { 3: "M0,.5 L1,.5 M.5,.5 L.5,1" }, "\u2534": { 1: "M0,.5 L1,.5 M.5,.5 L.5,0" }, "\u253B": { 3: "M0,.5 L1,.5 M.5,.5 L.5,0" }, "\u253C": { 1: "M0,.5 L1,.5 M.5,0 L.5,1" }, "\u254B": { 3: "M0,.5 L1,.5 M.5,0 L.5,1" }, "\u2574": { 1: "M.5,.5 L0,.5" }, "\u2578": { 3: "M.5,.5 L0,.5" }, "\u2575": { 1: "M.5,.5 L.5,0" }, "\u2579": { 3: "M.5,.5 L.5,0" }, "\u2576": { 1: "M.5,.5 L1,.5" }, "\u257A": { 3: "M.5,.5 L1,.5" }, "\u2577": { 1: "M.5,.5 L.5,1" }, "\u257B": { 3: "M.5,.5 L.5,1" }, "\u2550": { 1: (i12, e4) => `M0,${0.5 - e4} L1,${0.5 - e4} M0,${0.5 + e4} L1,${0.5 + e4}` }, "\u2551": { 1: (i12, e4) => `M${0.5 - i12},0 L${0.5 - i12},1 M${0.5 + i12},0 L${0.5 + i12},1` }, "\u2552": { 1: (i12, e4) => `M.5,1 L.5,${0.5 - e4} L1,${0.5 - e4} M.5,${0.5 + e4} L1,${0.5 + e4}` }, "\u2553": { 1: (i12, e4) => `M${0.5 - i12},1 L${0.5 - i12},.5 L1,.5 M${0.5 + i12},.5 L${0.5 + i12},1` }, "\u2554": { 1: (i12, e4) => `M1,${0.5 - e4} L${0.5 - i12},${0.5 - e4} L${0.5 - i12},1 M1,${0.5 + e4} L${0.5 + i12},${0.5 + e4} L${0.5 + i12},1` }, "\u2555": { 1: (i12, e4) => `M0,${0.5 - e4} L.5,${0.5 - e4} L.5,1 M0,${0.5 + e4} L.5,${0.5 + e4}` }, "\u2556": { 1: (i12, e4) => `M${0.5 + i12},1 L${0.5 + i12},.5 L0,.5 M${0.5 - i12},.5 L${0.5 - i12},1` }, "\u2557": { 1: (i12, e4) => `M0,${0.5 + e4} L${0.5 - i12},${0.5 + e4} L${0.5 - i12},1 M0,${0.5 - e4} L${0.5 + i12},${0.5 - e4} L${0.5 + i12},1` }, "\u2558": { 1: (i12, e4) => `M.5,0 L.5,${0.5 + e4} L1,${0.5 + e4} M.5,${0.5 - e4} L1,${0.5 - e4}` }, "\u2559": { 1: (i12, e4) => `M1,.5 L${0.5 - i12},.5 L${0.5 - i12},0 M${0.5 + i12},.5 L${0.5 + i12},0` }, "\u255A": { 1: (i12, e4) => `M1,${0.5 - e4} L${0.5 + i12},${0.5 - e4} L${0.5 + i12},0 M1,${0.5 + e4} L${0.5 - i12},${0.5 + e4} L${0.5 - i12},0` }, "\u255B": { 1: (i12, e4) => `M0,${0.5 + e4} L.5,${0.5 + e4} L.5,0 M0,${0.5 - e4} L.5,${0.5 - e4}` }, "\u255C": { 1: (i12, e4) => `M0,.5 L${0.5 + i12},.5 L${0.5 + i12},0 M${0.5 - i12},.5 L${0.5 - i12},0` }, "\u255D": { 1: (i12, e4) => `M0,${0.5 - e4} L${0.5 - i12},${0.5 - e4} L${0.5 - i12},0 M0,${0.5 + e4} L${0.5 + i12},${0.5 + e4} L${0.5 + i12},0` }, "\u255E": { 1: (i12, e4) => `M.5,0 L.5,1 M.5,${0.5 - e4} L1,${0.5 - e4} M.5,${0.5 + e4} L1,${0.5 + e4}` }, "\u255F": { 1: (i12, e4) => `M${0.5 - i12},0 L${0.5 - i12},1 M${0.5 + i12},0 L${0.5 + i12},1 M${0.5 + i12},.5 L1,.5` }, "\u2560": { 1: (i12, e4) => `M${0.5 - i12},0 L${0.5 - i12},1 M1,${0.5 + e4} L${0.5 + i12},${0.5 + e4} L${0.5 + i12},1 M1,${0.5 - e4} L${0.5 + i12},${0.5 - e4} L${0.5 + i12},0` }, "\u2561": { 1: (i12, e4) => `M.5,0 L.5,1 M0,${0.5 - e4} L.5,${0.5 - e4} M0,${0.5 + e4} L.5,${0.5 + e4}` }, "\u2562": { 1: (i12, e4) => `M0,.5 L${0.5 - i12},.5 M${0.5 - i12},0 L${0.5 - i12},1 M${0.5 + i12},0 L${0.5 + i12},1` }, "\u2563": { 1: (i12, e4) => `M${0.5 + i12},0 L${0.5 + i12},1 M0,${0.5 + e4} L${0.5 - i12},${0.5 + e4} L${0.5 - i12},1 M0,${0.5 - e4} L${0.5 - i12},${0.5 - e4} L${0.5 - i12},0` }, "\u2564": { 1: (i12, e4) => `M0,${0.5 - e4} L1,${0.5 - e4} M0,${0.5 + e4} L1,${0.5 + e4} M.5,${0.5 + e4} L.5,1` }, "\u2565": { 1: (i12, e4) => `M0,.5 L1,.5 M${0.5 - i12},.5 L${0.5 - i12},1 M${0.5 + i12},.5 L${0.5 + i12},1` }, "\u2566": { 1: (i12, e4) => `M0,${0.5 - e4} L1,${0.5 - e4} M0,${0.5 + e4} L${0.5 - i12},${0.5 + e4} L${0.5 - i12},1 M1,${0.5 + e4} L${0.5 + i12},${0.5 + e4} L${0.5 + i12},1` }, "\u2567": { 1: (i12, e4) => `M.5,0 L.5,${0.5 - e4} M0,${0.5 - e4} L1,${0.5 - e4} M0,${0.5 + e4} L1,${0.5 + e4}` }, "\u2568": { 1: (i12, e4) => `M0,.5 L1,.5 M${0.5 - i12},.5 L${0.5 - i12},0 M${0.5 + i12},.5 L${0.5 + i12},0` }, "\u2569": { 1: (i12, e4) => `M0,${0.5 + e4} L1,${0.5 + e4} M0,${0.5 - e4} L${0.5 - i12},${0.5 - e4} L${0.5 - i12},0 M1,${0.5 - e4} L${0.5 + i12},${0.5 - e4} L${0.5 + i12},0` }, "\u256A": { 1: (i12, e4) => `M.5,0 L.5,1 M0,${0.5 - e4} L1,${0.5 - e4} M0,${0.5 + e4} L1,${0.5 + e4}` }, "\u256B": { 1: (i12, e4) => `M0,.5 L1,.5 M${0.5 - i12},0 L${0.5 - i12},1 M${0.5 + i12},0 L${0.5 + i12},1` }, "\u256C": { 1: (i12, e4) => `M0,${0.5 + e4} L${0.5 - i12},${0.5 + e4} L${0.5 - i12},1 M1,${0.5 + e4} L${0.5 + i12},${0.5 + e4} L${0.5 + i12},1 M0,${0.5 - e4} L${0.5 - i12},${0.5 - e4} L${0.5 - i12},0 M1,${0.5 - e4} L${0.5 + i12},${0.5 - e4} L${0.5 + i12},0` }, "\u2571": { 1: "M1,0 L0,1" }, "\u2572": { 1: "M0,0 L1,1" }, "\u2573": { 1: "M1,0 L0,1 M0,0 L1,1" }, "\u257C": { 1: "M.5,.5 L0,.5", 3: "M.5,.5 L1,.5" }, "\u257D": { 1: "M.5,.5 L.5,0", 3: "M.5,.5 L.5,1" }, "\u257E": { 1: "M.5,.5 L1,.5", 3: "M.5,.5 L0,.5" }, "\u257F": { 1: "M.5,.5 L.5,1", 3: "M.5,.5 L.5,0" }, "\u250D": { 1: "M.5,.5 L.5,1", 3: "M.5,.5 L1,.5" }, "\u250E": { 1: "M.5,.5 L1,.5", 3: "M.5,.5 L.5,1" }, "\u2511": { 1: "M.5,.5 L.5,1", 3: "M.5,.5 L0,.5" }, "\u2512": { 1: "M.5,.5 L0,.5", 3: "M.5,.5 L.5,1" }, "\u2515": { 1: "M.5,.5 L.5,0", 3: "M.5,.5 L1,.5" }, "\u2516": { 1: "M.5,.5 L1,.5", 3: "M.5,.5 L.5,0" }, "\u2519": { 1: "M.5,.5 L.5,0", 3: "M.5,.5 L0,.5" }, "\u251A": { 1: "M.5,.5 L0,.5", 3: "M.5,.5 L.5,0" }, "\u251D": { 1: "M.5,0 L.5,1", 3: "M.5,.5 L1,.5" }, "\u251E": { 1: "M0.5,1 L.5,.5 L1,.5", 3: "M.5,.5 L.5,0" }, "\u251F": { 1: "M.5,0 L.5,.5 L1,.5", 3: "M.5,.5 L.5,1" }, "\u2520": { 1: "M.5,.5 L1,.5", 3: "M.5,0 L.5,1" }, "\u2521": { 1: "M.5,.5 L.5,1", 3: "M.5,0 L.5,.5 L1,.5" }, "\u2522": { 1: "M.5,.5 L.5,0", 3: "M0.5,1 L.5,.5 L1,.5" }, "\u2525": { 1: "M.5,0 L.5,1", 3: "M.5,.5 L0,.5" }, "\u2526": { 1: "M0,.5 L.5,.5 L.5,1", 3: "M.5,.5 L.5,0" }, "\u2527": { 1: "M.5,0 L.5,.5 L0,.5", 3: "M.5,.5 L.5,1" }, "\u2528": { 1: "M.5,.5 L0,.5", 3: "M.5,0 L.5,1" }, "\u2529": { 1: "M.5,.5 L.5,1", 3: "M.5,0 L.5,.5 L0,.5" }, "\u252A": { 1: "M.5,.5 L.5,0", 3: "M0,.5 L.5,.5 L.5,1" }, "\u252D": { 1: "M0.5,1 L.5,.5 L1,.5", 3: "M.5,.5 L0,.5" }, "\u252E": { 1: "M0,.5 L.5,.5 L.5,1", 3: "M.5,.5 L1,.5" }, "\u252F": { 1: "M.5,.5 L.5,1", 3: "M0,.5 L1,.5" }, "\u2530": { 1: "M0,.5 L1,.5", 3: "M.5,.5 L.5,1" }, "\u2531": { 1: "M.5,.5 L1,.5", 3: "M0,.5 L.5,.5 L.5,1" }, "\u2532": { 1: "M.5,.5 L0,.5", 3: "M0.5,1 L.5,.5 L1,.5" }, "\u2535": { 1: "M.5,0 L.5,.5 L1,.5", 3: "M.5,.5 L0,.5" }, "\u2536": { 1: "M.5,0 L.5,.5 L0,.5", 3: "M.5,.5 L1,.5" }, "\u2537": { 1: "M.5,.5 L.5,0", 3: "M0,.5 L1,.5" }, "\u2538": { 1: "M0,.5 L1,.5", 3: "M.5,.5 L.5,0" }, "\u2539": { 1: "M.5,.5 L1,.5", 3: "M.5,0 L.5,.5 L0,.5" }, "\u253A": { 1: "M.5,.5 L0,.5", 3: "M.5,0 L.5,.5 L1,.5" }, "\u253D": { 1: "M.5,0 L.5,1 M.5,.5 L1,.5", 3: "M.5,.5 L0,.5" }, "\u253E": { 1: "M.5,0 L.5,1 M.5,.5 L0,.5", 3: "M.5,.5 L1,.5" }, "\u253F": { 1: "M.5,0 L.5,1", 3: "M0,.5 L1,.5" }, "\u2540": { 1: "M0,.5 L1,.5 M.5,.5 L.5,1", 3: "M.5,.5 L.5,0" }, "\u2541": { 1: "M.5,.5 L.5,0 M0,.5 L1,.5", 3: "M.5,.5 L.5,1" }, "\u2542": { 1: "M0,.5 L1,.5", 3: "M.5,0 L.5,1" }, "\u2543": { 1: "M0.5,1 L.5,.5 L1,.5", 3: "M.5,0 L.5,.5 L0,.5" }, "\u2544": { 1: "M0,.5 L.5,.5 L.5,1", 3: "M.5,0 L.5,.5 L1,.5" }, "\u2545": { 1: "M.5,0 L.5,.5 L1,.5", 3: "M0,.5 L.5,.5 L.5,1" }, "\u2546": { 1: "M.5,0 L.5,.5 L0,.5", 3: "M0.5,1 L.5,.5 L1,.5" }, "\u2547": { 1: "M.5,.5 L.5,1", 3: "M.5,.5 L.5,0 M0,.5 L1,.5" }, "\u2548": { 1: "M.5,.5 L.5,0", 3: "M0,.5 L1,.5 M.5,.5 L.5,1" }, "\u2549": { 1: "M.5,.5 L1,.5", 3: "M.5,0 L.5,1 M.5,.5 L0,.5" }, "\u254A": { 1: "M.5,.5 L0,.5", 3: "M.5,0 L.5,1 M.5,.5 L1,.5" }, "\u254C": { 1: "M.1,.5 L.4,.5 M.6,.5 L.9,.5" }, "\u254D": { 3: "M.1,.5 L.4,.5 M.6,.5 L.9,.5" }, "\u2504": { 1: "M.0667,.5 L.2667,.5 M.4,.5 L.6,.5 M.7333,.5 L.9333,.5" }, "\u2505": { 3: "M.0667,.5 L.2667,.5 M.4,.5 L.6,.5 M.7333,.5 L.9333,.5" }, "\u2508": { 1: "M.05,.5 L.2,.5 M.3,.5 L.45,.5 M.55,.5 L.7,.5 M.8,.5 L.95,.5" }, "\u2509": { 3: "M.05,.5 L.2,.5 M.3,.5 L.45,.5 M.55,.5 L.7,.5 M.8,.5 L.95,.5" }, "\u254E": { 1: "M.5,.1 L.5,.4 M.5,.6 L.5,.9" }, "\u254F": { 3: "M.5,.1 L.5,.4 M.5,.6 L.5,.9" }, "\u2506": { 1: "M.5,.0667 L.5,.2667 M.5,.4 L.5,.6 M.5,.7333 L.5,.9333" }, "\u2507": { 3: "M.5,.0667 L.5,.2667 M.5,.4 L.5,.6 M.5,.7333 L.5,.9333" }, "\u250A": { 1: "M.5,.05 L.5,.2 M.5,.3 L.5,.45 L.5,.55 M.5,.7 L.5,.95" }, "\u250B": { 3: "M.5,.05 L.5,.2 M.5,.3 L.5,.45 L.5,.55 M.5,.7 L.5,.95" }, "\u256D": { 1: (i12, e4) => `M.5,1 L.5,${0.5 + e4 / 0.15 * 0.5} C.5,${0.5 + e4 / 0.15 * 0.5},.5,.5,1,.5` }, "\u256E": { 1: (i12, e4) => `M.5,1 L.5,${0.5 + e4 / 0.15 * 0.5} C.5,${0.5 + e4 / 0.15 * 0.5},.5,.5,0,.5` }, "\u256F": { 1: (i12, e4) => `M.5,0 L.5,${0.5 - e4 / 0.15 * 0.5} C.5,${0.5 - e4 / 0.15 * 0.5},.5,.5,0,.5` }, "\u2570": { 1: (i12, e4) => `M.5,0 L.5,${0.5 - e4 / 0.15 * 0.5} C.5,${0.5 - e4 / 0.15 * 0.5},.5,.5,1,.5` } };
    et = { "\uE0A0": { d: "M.3,1 L.03,1 L.03,.88 C.03,.82,.06,.78,.11,.73 C.15,.7,.2,.68,.28,.65 L.43,.6 C.49,.58,.53,.56,.56,.53 C.59,.5,.6,.47,.6,.43 L.6,.27 L.4,.27 L.69,.1 L.98,.27 L.78,.27 L.78,.46 C.78,.52,.76,.56,.72,.61 C.68,.66,.63,.67,.56,.7 L.48,.72 C.42,.74,.38,.76,.35,.78 C.32,.8,.31,.84,.31,.88 L.31,1 M.3,.5 L.03,.59 L.03,.09 L.3,.09 L.3,.655", type: 0 }, "\uE0A1": { d: "M.7,.4 L.7,.47 L.2,.47 L.2,.03 L.355,.03 L.355,.4 L.705,.4 M.7,.5 L.86,.5 L.86,.95 L.69,.95 L.44,.66 L.46,.86 L.46,.95 L.3,.95 L.3,.49 L.46,.49 L.71,.78 L.69,.565 L.69,.5", type: 0 }, "\uE0A2": { d: "M.25,.94 C.16,.94,.11,.92,.11,.87 L.11,.53 C.11,.48,.15,.455,.23,.45 L.23,.3 C.23,.25,.26,.22,.31,.19 C.36,.16,.43,.15,.51,.15 C.59,.15,.66,.16,.71,.19 C.77,.22,.79,.26,.79,.3 L.79,.45 C.87,.45,.91,.48,.91,.53 L.91,.87 C.91,.92,.86,.94,.77,.94 L.24,.94 M.53,.2 C.49,.2,.45,.21,.42,.23 C.39,.25,.38,.27,.38,.3 L.38,.45 L.68,.45 L.68,.3 C.68,.27,.67,.25,.64,.23 C.61,.21,.58,.2,.53,.2 M.58,.82 L.58,.66 C.63,.65,.65,.63,.65,.6 C.65,.58,.64,.57,.61,.56 C.58,.55,.56,.54,.52,.54 C.48,.54,.46,.55,.43,.56 C.4,.57,.39,.59,.39,.6 C.39,.63,.41,.64,.46,.66 L.46,.82 L.57,.82", type: 0 }, "\uE0B0": { d: "M0,0 L1,.5 L0,1", type: 0, rightPadding: 2 }, "\uE0B1": { d: "M-1,-.5 L1,.5 L-1,1.5", type: 1, leftPadding: 1, rightPadding: 1 }, "\uE0B2": { d: "M1,0 L0,.5 L1,1", type: 0, leftPadding: 2 }, "\uE0B3": { d: "M2,-.5 L0,.5 L2,1.5", type: 1, leftPadding: 1, rightPadding: 1 }, "\uE0B4": { d: "M0,0 L0,1 C0.552,1,1,0.776,1,.5 C1,0.224,0.552,0,0,0", type: 0, rightPadding: 1 }, "\uE0B5": { d: "M.2,1 C.422,1,.8,.826,.78,.5 C.8,.174,0.422,0,.2,0", type: 1, rightPadding: 1 }, "\uE0B6": { d: "M1,0 L1,1 C0.448,1,0,0.776,0,.5 C0,0.224,0.448,0,1,0", type: 0, leftPadding: 1 }, "\uE0B7": { d: "M.8,1 C0.578,1,0.2,.826,.22,.5 C0.2,0.174,0.578,0,0.8,0", type: 1, leftPadding: 1 }, "\uE0B8": { d: "M-.5,-.5 L1.5,1.5 L-.5,1.5", type: 0 }, "\uE0B9": { d: "M-.5,-.5 L1.5,1.5", type: 1, leftPadding: 1, rightPadding: 1 }, "\uE0BA": { d: "M1.5,-.5 L-.5,1.5 L1.5,1.5", type: 0 }, "\uE0BC": { d: "M1.5,-.5 L-.5,1.5 L-.5,-.5", type: 0 }, "\uE0BD": { d: "M1.5,-.5 L-.5,1.5", type: 1, leftPadding: 1, rightPadding: 1 }, "\uE0BE": { d: "M-.5,-.5 L1.5,1.5 L1.5,-.5", type: 0 } };
    et["\uE0BB"] = et["\uE0BD"];
    et["\uE0BF"] = et["\uE0B9"];
    xn3 = /* @__PURE__ */ new Map();
    In3 = { C: (i12, e4) => i12.bezierCurveTo(e4[0], e4[1], e4[2], e4[3], e4[4], e4[5]), L: (i12, e4) => i12.lineTo(e4[0], e4[1]), M: (i12, e4) => i12.moveTo(e4[0], e4[1]) };
    Ot3 = class {
      constructor() {
        this._data = {};
      }
      set(e4, t3, n4) {
        this._data[e4] || (this._data[e4] = {}), this._data[e4][t3] = n4;
      }
      get(e4, t3) {
        return this._data[e4] ? this._data[e4][t3] : void 0;
      }
      clear() {
        this._data = {};
      }
    };
    tt2 = class {
      constructor() {
        this._data = new Ot3();
      }
      set(e4, t3, n4, s18, o6) {
        this._data.get(e4, t3) || this._data.set(e4, t3, new Ot3()), this._data.get(e4, t3).set(n4, s18, o6);
      }
      get(e4, t3, n4, s18) {
        return this._data.get(e4, t3)?.get(n4, s18);
      }
      clear() {
        this._data.clear();
      }
    };
    Ft2 = class {
      constructor() {
        this._tasks = [];
        this._i = 0;
      }
      enqueue(e4) {
        this._tasks.push(e4), this._start();
      }
      flush() {
        for (; this._i < this._tasks.length; ) this._tasks[this._i]() || this._i++;
        this.clear();
      }
      clear() {
        this._idleCallback && (this._cancelCallback(this._idleCallback), this._idleCallback = void 0), this._i = 0, this._tasks.length = 0;
      }
      _start() {
        this._idleCallback || (this._idleCallback = this._requestCallback(this._process.bind(this)));
      }
      _process(e4) {
        this._idleCallback = void 0;
        let t3 = 0, n4 = 0, s18 = e4.timeRemaining(), o6 = 0;
        for (; this._i < this._tasks.length; ) {
          if (t3 = performance.now(), this._tasks[this._i]() || this._i++, t3 = Math.max(1, performance.now() - t3), n4 = Math.max(t3, n4), o6 = e4.timeRemaining(), n4 * 1.5 > o6) {
            s18 - t3 < -20 && console.warn(`task queue exceeded allotted deadline by ${Math.abs(Math.round(s18 - t3))}ms`), this._start();
            return;
          }
          s18 = o6;
        }
        this.clear();
      }
    };
    gi2 = class extends Ft2 {
      _requestCallback(e4) {
        return setTimeout(() => e4(this._createDeadline(16)));
      }
      _cancelCallback(e4) {
        clearTimeout(e4);
      }
      _createDeadline(e4) {
        let t3 = performance.now() + e4;
        return { timeRemaining: () => Math.max(0, t3 - performance.now()) };
      }
    };
    xi2 = class extends Ft2 {
      _requestCallback(e4) {
        return requestIdleCallback(e4);
      }
      _cancelCallback(e4) {
        cancelIdleCallback(e4);
      }
    };
    wn2 = !Lt3 && "requestIdleCallback" in window ? xi2 : gi2;
    he4 = class i6 {
      constructor() {
        this.fg = 0;
        this.bg = 0;
        this.extended = new it2();
      }
      static toColorRGB(e4) {
        return [e4 >>> 16 & 255, e4 >>> 8 & 255, e4 & 255];
      }
      static fromColorRGB(e4) {
        return (e4[0] & 255) << 16 | (e4[1] & 255) << 8 | e4[2] & 255;
      }
      clone() {
        let e4 = new i6();
        return e4.fg = this.fg, e4.bg = this.bg, e4.extended = this.extended.clone(), e4;
      }
      isInverse() {
        return this.fg & 67108864;
      }
      isBold() {
        return this.fg & 134217728;
      }
      isUnderline() {
        return this.hasExtendedAttrs() && this.extended.underlineStyle !== 0 ? 1 : this.fg & 268435456;
      }
      isBlink() {
        return this.fg & 536870912;
      }
      isInvisible() {
        return this.fg & 1073741824;
      }
      isItalic() {
        return this.bg & 67108864;
      }
      isDim() {
        return this.bg & 134217728;
      }
      isStrikethrough() {
        return this.fg & 2147483648;
      }
      isProtected() {
        return this.bg & 536870912;
      }
      isOverline() {
        return this.bg & 1073741824;
      }
      getFgColorMode() {
        return this.fg & 50331648;
      }
      getBgColorMode() {
        return this.bg & 50331648;
      }
      isFgRGB() {
        return (this.fg & 50331648) === 50331648;
      }
      isBgRGB() {
        return (this.bg & 50331648) === 50331648;
      }
      isFgPalette() {
        return (this.fg & 50331648) === 16777216 || (this.fg & 50331648) === 33554432;
      }
      isBgPalette() {
        return (this.bg & 50331648) === 16777216 || (this.bg & 50331648) === 33554432;
      }
      isFgDefault() {
        return (this.fg & 50331648) === 0;
      }
      isBgDefault() {
        return (this.bg & 50331648) === 0;
      }
      isAttributeDefault() {
        return this.fg === 0 && this.bg === 0;
      }
      getFgColor() {
        switch (this.fg & 50331648) {
          case 16777216:
          case 33554432:
            return this.fg & 255;
          case 50331648:
            return this.fg & 16777215;
          default:
            return -1;
        }
      }
      getBgColor() {
        switch (this.bg & 50331648) {
          case 16777216:
          case 33554432:
            return this.bg & 255;
          case 50331648:
            return this.bg & 16777215;
          default:
            return -1;
        }
      }
      hasExtendedAttrs() {
        return this.bg & 268435456;
      }
      updateExtended() {
        this.extended.isEmpty() ? this.bg &= -268435457 : this.bg |= 268435456;
      }
      getUnderlineColor() {
        if (this.bg & 268435456 && ~this.extended.underlineColor) switch (this.extended.underlineColor & 50331648) {
          case 16777216:
          case 33554432:
            return this.extended.underlineColor & 255;
          case 50331648:
            return this.extended.underlineColor & 16777215;
          default:
            return this.getFgColor();
        }
        return this.getFgColor();
      }
      getUnderlineColorMode() {
        return this.bg & 268435456 && ~this.extended.underlineColor ? this.extended.underlineColor & 50331648 : this.getFgColorMode();
      }
      isUnderlineColorRGB() {
        return this.bg & 268435456 && ~this.extended.underlineColor ? (this.extended.underlineColor & 50331648) === 50331648 : this.isFgRGB();
      }
      isUnderlineColorPalette() {
        return this.bg & 268435456 && ~this.extended.underlineColor ? (this.extended.underlineColor & 50331648) === 16777216 || (this.extended.underlineColor & 50331648) === 33554432 : this.isFgPalette();
      }
      isUnderlineColorDefault() {
        return this.bg & 268435456 && ~this.extended.underlineColor ? (this.extended.underlineColor & 50331648) === 0 : this.isFgDefault();
      }
      getUnderlineStyle() {
        return this.fg & 268435456 ? this.bg & 268435456 ? this.extended.underlineStyle : 1 : 0;
      }
      getUnderlineVariantOffset() {
        return this.extended.underlineVariantOffset;
      }
    };
    it2 = class i7 {
      constructor(e4 = 0, t3 = 0) {
        this._ext = 0;
        this._urlId = 0;
        this._ext = e4, this._urlId = t3;
      }
      get ext() {
        return this._urlId ? this._ext & -469762049 | this.underlineStyle << 26 : this._ext;
      }
      set ext(e4) {
        this._ext = e4;
      }
      get underlineStyle() {
        return this._urlId ? 5 : (this._ext & 469762048) >> 26;
      }
      set underlineStyle(e4) {
        this._ext &= -469762049, this._ext |= e4 << 26 & 469762048;
      }
      get underlineColor() {
        return this._ext & 67108863;
      }
      set underlineColor(e4) {
        this._ext &= -67108864, this._ext |= e4 & 67108863;
      }
      get urlId() {
        return this._urlId;
      }
      set urlId(e4) {
        this._urlId = e4;
      }
      get underlineVariantOffset() {
        let e4 = (this._ext & 3758096384) >> 29;
        return e4 < 0 ? e4 ^ 4294967288 : e4;
      }
      set underlineVariantOffset(e4) {
        this._ext &= 536870911, this._ext |= e4 << 29 & 3758096384;
      }
      clone() {
        return new i7(this._ext, this._urlId);
      }
      isEmpty() {
        return this.underlineStyle === 0 && this._urlId === 0;
      }
    };
    He2 = class He3 {
      constructor(e4) {
        this.element = e4, this.next = He3.Undefined, this.prev = He3.Undefined;
      }
    };
    He2.Undefined = new He2(void 0);
    zr3 = globalThis.performance && typeof globalThis.performance.now == "function";
    kt4 = class i8 {
      static create(e4) {
        return new i8(e4);
      }
      constructor(e4) {
        this._now = zr3 && e4 === false ? Date.now : globalThis.performance.now.bind(globalThis.performance), this._startTime = this._now(), this._stopTime = -1;
      }
      stop() {
        this._stopTime = this._now();
      }
      reset() {
        this._startTime = this._now(), this._stopTime = -1;
      }
      elapsed() {
        return this._stopTime !== -1 ? this._stopTime - this._startTime : this._now() - this._startTime;
      }
    };
    qr2 = false;
    Dn3 = false;
    jr2 = false;
    ((se2) => {
      se2.None = () => B5.None;
      function e4(v4) {
        if (jr2) {
          let { onDidAddListener: p3 } = v4, g3 = nt3.create(), b4 = 0;
          v4.onDidAddListener = () => {
            ++b4 === 2 && (console.warn("snapshotted emitter LIKELY used public and SHOULD HAVE BEEN created with DisposableStore. snapshotted here"), g3.print()), p3?.();
          };
        }
      }
      function t3(v4, p3) {
        return h4(v4, () => {
        }, 0, void 0, true, void 0, p3);
      }
      se2.defer = t3;
      function n4(v4) {
        return (p3, g3 = null, b4) => {
          let m2 = false, _4;
          return _4 = v4((T2) => {
            if (!m2) return _4 ? _4.dispose() : m2 = true, p3.call(g3, T2);
          }, null, b4), m2 && _4.dispose(), _4;
        };
      }
      se2.once = n4;
      function s18(v4, p3, g3) {
        return c4((b4, m2 = null, _4) => v4((T2) => b4.call(m2, p3(T2)), null, _4), g3);
      }
      se2.map = s18;
      function o6(v4, p3, g3) {
        return c4((b4, m2 = null, _4) => v4((T2) => {
          p3(T2), b4.call(m2, T2);
        }, null, _4), g3);
      }
      se2.forEach = o6;
      function r7(v4, p3, g3) {
        return c4((b4, m2 = null, _4) => v4((T2) => p3(T2) && b4.call(m2, T2), null, _4), g3);
      }
      se2.filter = r7;
      function a3(v4) {
        return v4;
      }
      se2.signal = a3;
      function l4(...v4) {
        return (p3, g3 = null, b4) => {
          let m2 = It3(...v4.map((_4) => _4((T2) => p3.call(g3, T2))));
          return d3(m2, b4);
        };
      }
      se2.any = l4;
      function u3(v4, p3, g3, b4) {
        let m2 = g3;
        return s18(v4, (_4) => (m2 = p3(m2, _4), m2), b4);
      }
      se2.reduce = u3;
      function c4(v4, p3) {
        let g3, b4 = { onWillAddFirstListener() {
          g3 = v4(m2.fire, m2);
        }, onDidRemoveLastListener() {
          g3?.dispose();
        } };
        p3 || e4(b4);
        let m2 = new D3(b4);
        return p3?.add(m2), m2.event;
      }
      function d3(v4, p3) {
        return p3 instanceof Array ? p3.push(v4) : p3 && p3.add(v4), v4;
      }
      function h4(v4, p3, g3 = 100, b4 = false, m2 = false, _4, T2) {
        let x2, R2, $5, P4 = 0, de4, Re3 = { leakWarningThreshold: _4, onWillAddFirstListener() {
          x2 = v4((ie3) => {
            P4++, R2 = p3(R2, ie3), b4 && !$5 && (oe.fire(R2), R2 = void 0), de4 = () => {
              let N3 = R2;
              R2 = void 0, $5 = void 0, (!b4 || P4 > 1) && oe.fire(N3), P4 = 0;
            }, typeof g3 == "number" ? (clearTimeout($5), $5 = setTimeout(de4, g3)) : $5 === void 0 && ($5 = 0, queueMicrotask(de4));
          });
        }, onWillRemoveListener() {
          m2 && P4 > 0 && de4?.();
        }, onDidRemoveLastListener() {
          de4 = void 0, x2.dispose();
        } };
        T2 || e4(Re3);
        let oe = new D3(Re3);
        return T2?.add(oe), oe.event;
      }
      se2.debounce = h4;
      function f3(v4, p3 = 0, g3) {
        return se2.debounce(v4, (b4, m2) => b4 ? (b4.push(m2), b4) : [m2], p3, void 0, true, void 0, g3);
      }
      se2.accumulate = f3;
      function I2(v4, p3 = (b4, m2) => b4 === m2, g3) {
        let b4 = true, m2;
        return r7(v4, (_4) => {
          let T2 = b4 || !p3(_4, m2);
          return b4 = false, m2 = _4, T2;
        }, g3);
      }
      se2.latch = I2;
      function L4(v4, p3, g3) {
        return [se2.filter(v4, p3, g3), se2.filter(v4, (b4) => !p3(b4), g3)];
      }
      se2.split = L4;
      function M4(v4, p3 = false, g3 = [], b4) {
        let m2 = g3.slice(), _4 = v4((R2) => {
          m2 ? m2.push(R2) : x2.fire(R2);
        });
        b4 && b4.add(_4);
        let T2 = () => {
          m2?.forEach((R2) => x2.fire(R2)), m2 = null;
        }, x2 = new D3({ onWillAddFirstListener() {
          _4 || (_4 = v4((R2) => x2.fire(R2)), b4 && b4.add(_4));
        }, onDidAddFirstListener() {
          m2 && (p3 ? setTimeout(T2) : T2());
        }, onDidRemoveLastListener() {
          _4 && _4.dispose(), _4 = null;
        } });
        return b4 && b4.add(x2), x2.event;
      }
      se2.buffer = M4;
      function q2(v4, p3) {
        return (b4, m2, _4) => {
          let T2 = p3(new W2());
          return v4(function(x2) {
            let R2 = T2.evaluate(x2);
            R2 !== S4 && b4.call(m2, R2);
          }, void 0, _4);
        };
      }
      se2.chain = q2;
      let S4 = Symbol("HaltChainable");
      class W2 {
        constructor() {
          this.steps = [];
        }
        map(p3) {
          return this.steps.push(p3), this;
        }
        forEach(p3) {
          return this.steps.push((g3) => (p3(g3), g3)), this;
        }
        filter(p3) {
          return this.steps.push((g3) => p3(g3) ? g3 : S4), this;
        }
        reduce(p3, g3) {
          let b4 = g3;
          return this.steps.push((m2) => (b4 = p3(b4, m2), b4)), this;
        }
        latch(p3 = (g3, b4) => g3 === b4) {
          let g3 = true, b4;
          return this.steps.push((m2) => {
            let _4 = g3 || !p3(m2, b4);
            return g3 = false, b4 = m2, _4 ? m2 : S4;
          }), this;
        }
        evaluate(p3) {
          for (let g3 of this.steps) if (p3 = g3(p3), p3 === S4) break;
          return p3;
        }
      }
      function E2(v4, p3, g3 = (b4) => b4) {
        let b4 = (...x2) => T2.fire(g3(...x2)), m2 = () => v4.on(p3, b4), _4 = () => v4.removeListener(p3, b4), T2 = new D3({ onWillAddFirstListener: m2, onDidRemoveLastListener: _4 });
        return T2.event;
      }
      se2.fromNodeEventEmitter = E2;
      function y3(v4, p3, g3 = (b4) => b4) {
        let b4 = (...x2) => T2.fire(g3(...x2)), m2 = () => v4.addEventListener(p3, b4), _4 = () => v4.removeEventListener(p3, b4), T2 = new D3({ onWillAddFirstListener: m2, onDidRemoveLastListener: _4 });
        return T2.event;
      }
      se2.fromDOMEventEmitter = y3;
      function w3(v4) {
        return new Promise((p3) => n4(v4)(p3));
      }
      se2.toPromise = w3;
      function G3(v4) {
        let p3 = new D3();
        return v4.then((g3) => {
          p3.fire(g3);
        }, () => {
          p3.fire(void 0);
        }).finally(() => {
          p3.dispose();
        }), p3.event;
      }
      se2.fromPromise = G3;
      function ue4(v4, p3) {
        return v4((g3) => p3.fire(g3));
      }
      se2.forward = ue4;
      function Se2(v4, p3, g3) {
        return p3(g3), v4((b4) => p3(b4));
      }
      se2.runAndSubscribe = Se2;
      class ce3 {
        constructor(p3, g3) {
          this._observable = p3;
          this._counter = 0;
          this._hasChanged = false;
          let b4 = { onWillAddFirstListener: () => {
            p3.addObserver(this);
          }, onDidRemoveLastListener: () => {
            p3.removeObserver(this);
          } };
          g3 || e4(b4), this.emitter = new D3(b4), g3 && g3.add(this.emitter);
        }
        beginUpdate(p3) {
          this._counter++;
        }
        handlePossibleChange(p3) {
        }
        handleChange(p3, g3) {
          this._hasChanged = true;
        }
        endUpdate(p3) {
          this._counter--, this._counter === 0 && (this._observable.reportChanges(), this._hasChanged && (this._hasChanged = false, this.emitter.fire(this._observable.get())));
        }
      }
      function we3(v4, p3) {
        return new ce3(v4, p3).emitter.event;
      }
      se2.fromObservable = we3;
      function A3(v4) {
        return (p3, g3, b4) => {
          let m2 = 0, _4 = false, T2 = { beginUpdate() {
            m2++;
          }, endUpdate() {
            m2--, m2 === 0 && (v4.reportChanges(), _4 && (_4 = false, p3.call(g3)));
          }, handlePossibleChange() {
          }, handleChange() {
            _4 = true;
          } };
          v4.addObserver(T2), v4.reportChanges();
          let x2 = { dispose() {
            v4.removeObserver(T2);
          } };
          return b4 instanceof fe3 ? b4.add(x2) : Array.isArray(b4) && b4.push(x2), x2;
        };
      }
      se2.fromObservableLight = A3;
    })(ee3 || (ee3 = {}));
    We3 = class We4 {
      constructor(e4) {
        this.listenerCount = 0;
        this.invocationCount = 0;
        this.elapsedOverall = 0;
        this.durations = [];
        this.name = `${e4}_${We4._idPool++}`, We4.all.add(this);
      }
      start(e4) {
        this._stopWatch = new kt4(), this.listenerCount = e4;
      }
      stop() {
        if (this._stopWatch) {
          let e4 = this._stopWatch.elapsed();
          this.durations.push(e4), this.elapsedOverall += e4, this.invocationCount += 1, this._stopWatch = void 0;
        }
      }
    };
    We3.all = /* @__PURE__ */ new Set(), We3._idPool = 0;
    Ei2 = We3;
    Mn2 = -1;
    Bt2 = class Bt3 {
      constructor(e4, t3, n4 = (Bt3._idPool++).toString(16).padStart(3, "0")) {
        this._errorHandler = e4;
        this.threshold = t3;
        this.name = n4;
        this._warnCountdown = 0;
      }
      dispose() {
        this._stacks?.clear();
      }
      check(e4, t3) {
        let n4 = this.threshold;
        if (n4 <= 0 || t3 < n4) return;
        this._stacks || (this._stacks = /* @__PURE__ */ new Map());
        let s18 = this._stacks.get(e4.value) || 0;
        if (this._stacks.set(e4.value, s18 + 1), this._warnCountdown -= 1, this._warnCountdown <= 0) {
          this._warnCountdown = n4 * 0.5;
          let [o6, r7] = this.getMostFrequentStack(), a3 = `[${this.name}] potential listener LEAK detected, having ${t3} listeners already. MOST frequent listener (${r7}):`;
          console.warn(a3), console.warn(o6);
          let l4 = new Ii2(a3, o6);
          this._errorHandler(l4);
        }
        return () => {
          let o6 = this._stacks.get(e4.value) || 0;
          this._stacks.set(e4.value, o6 - 1);
        };
      }
      getMostFrequentStack() {
        if (!this._stacks) return;
        let e4, t3 = 0;
        for (let [n4, s18] of this._stacks) (!e4 || t3 < s18) && (e4 = [n4, s18], t3 = s18);
        return e4;
      }
    };
    Bt2._idPool = 1;
    yi2 = Bt2;
    nt3 = class i9 {
      constructor(e4) {
        this.value = e4;
      }
      static create() {
        let e4 = new Error();
        return new i9(e4.stack ?? "");
      }
      print() {
        console.warn(this.value.split(`
`).slice(2).join(`
`));
      }
    };
    Ii2 = class extends Error {
      constructor(e4, t3) {
        super(e4), this.name = "ListenerLeakError", this.stack = t3;
      }
    };
    Li2 = class extends Error {
      constructor(e4, t3) {
        super(e4), this.name = "ListenerRefusalError", this.stack = t3;
      }
    };
    Xr2 = 0;
    Ge2 = class {
      constructor(e4) {
        this.value = e4;
        this.id = Xr2++;
      }
    };
    Yr2 = 2;
    Qr2 = (i12, e4) => {
      if (i12 instanceof Ge2) e4(i12);
      else for (let t3 = 0; t3 < i12.length; t3++) {
        let n4 = i12[t3];
        n4 && e4(n4);
      }
    };
    if (qr2) {
      let i12 = [];
      setInterval(() => {
        i12.length !== 0 && (console.warn("[LEAKING LISTENERS] GC'ed these listeners that were NOT yet disposed:"), console.warn(i12.join(`
`)), i12.length = 0);
      }, 3e3), Pt3 = new FinalizationRegistry((e4) => {
        typeof e4 == "string" && i12.push(e4);
      });
    }
    D3 = class {
      constructor(e4) {
        this._size = 0;
        this._options = e4, this._leakageMon = Mn2 > 0 || this._options?.leakWarningThreshold ? new yi2(e4?.onListenerError ?? Pe2, this._options?.leakWarningThreshold ?? Mn2) : void 0, this._perfMon = this._options?._profName ? new Ei2(this._options._profName) : void 0, this._deliveryQueue = this._options?.deliveryQueue;
      }
      dispose() {
        if (!this._disposed) {
          if (this._disposed = true, this._deliveryQueue?.current === this && this._deliveryQueue.reset(), this._listeners) {
            if (Dn3) {
              let e4 = this._listeners;
              queueMicrotask(() => {
                Qr2(e4, (t3) => t3.stack?.print());
              });
            }
            this._listeners = void 0, this._size = 0;
          }
          this._options?.onDidRemoveLastListener?.(), this._leakageMon?.dispose();
        }
      }
      get event() {
        return this._event ?? (this._event = (e4, t3, n4) => {
          if (this._leakageMon && this._size > this._leakageMon.threshold ** 2) {
            let l4 = `[${this._leakageMon.name}] REFUSES to accept new listeners because it exceeded its threshold by far (${this._size} vs ${this._leakageMon.threshold})`;
            console.warn(l4);
            let u3 = this._leakageMon.getMostFrequentStack() ?? ["UNKNOWN stack", -1], c4 = new Li2(`${l4}. HINT: Stack shows most frequent listener (${u3[1]}-times)`, u3[0]);
            return (this._options?.onListenerError || Pe2)(c4), B5.None;
          }
          if (this._disposed) return B5.None;
          t3 && (e4 = e4.bind(t3));
          let s18 = new Ge2(e4), o6, r7;
          this._leakageMon && this._size >= Math.ceil(this._leakageMon.threshold * 0.2) && (s18.stack = nt3.create(), o6 = this._leakageMon.check(s18.stack, this._size + 1)), Dn3 && (s18.stack = r7 ?? nt3.create()), this._listeners ? this._listeners instanceof Ge2 ? (this._deliveryQueue ?? (this._deliveryQueue = new wi2()), this._listeners = [this._listeners, s18]) : this._listeners.push(s18) : (this._options?.onWillAddFirstListener?.(this), this._listeners = s18, this._options?.onDidAddFirstListener?.(this)), this._size++;
          let a3 = O2(() => {
            Pt3?.unregister(a3), o6?.(), this._removeListener(s18);
          });
          if (n4 instanceof fe3 ? n4.add(a3) : Array.isArray(n4) && n4.push(a3), Pt3) {
            let l4 = new Error().stack.split(`
`).slice(2, 3).join(`
`).trim(), u3 = /(file:|vscode-file:\/\/vscode-app)?(\/[^:]*:\d+:\d+)/.exec(l4);
            Pt3.register(a3, u3?.[2] ?? l4, a3);
          }
          return a3;
        }), this._event;
      }
      _removeListener(e4) {
        if (this._options?.onWillRemoveListener?.(this), !this._listeners) return;
        if (this._size === 1) {
          this._listeners = void 0, this._options?.onDidRemoveLastListener?.(this), this._size = 0;
          return;
        }
        let t3 = this._listeners, n4 = t3.indexOf(e4);
        if (n4 === -1) throw console.log("disposed?", this._disposed), console.log("size?", this._size), console.log("arr?", JSON.stringify(this._listeners)), new Error("Attempted to dispose unknown listener");
        this._size--, t3[n4] = void 0;
        let s18 = this._deliveryQueue.current === this;
        if (this._size * Yr2 <= t3.length) {
          let o6 = 0;
          for (let r7 = 0; r7 < t3.length; r7++) t3[r7] ? t3[o6++] = t3[r7] : s18 && (this._deliveryQueue.end--, o6 < this._deliveryQueue.i && this._deliveryQueue.i--);
          t3.length = o6;
        }
      }
      _deliver(e4, t3) {
        if (!e4) return;
        let n4 = this._options?.onListenerError || Pe2;
        if (!n4) {
          e4.value(t3);
          return;
        }
        try {
          e4.value(t3);
        } catch (s18) {
          n4(s18);
        }
      }
      _deliverQueue(e4) {
        let t3 = e4.current._listeners;
        for (; e4.i < e4.end; ) this._deliver(t3[e4.i++], e4.value);
        e4.reset();
      }
      fire(e4) {
        if (this._deliveryQueue?.current && (this._deliverQueue(this._deliveryQueue), this._perfMon?.stop()), this._perfMon?.start(this._size), this._listeners) if (this._listeners instanceof Ge2) this._deliver(this._listeners, e4);
        else {
          let t3 = this._deliveryQueue;
          t3.enqueue(this, e4, this._listeners.length), this._deliverQueue(t3);
        }
        this._perfMon?.stop();
      }
      hasListeners() {
        return this._size > 0;
      }
    };
    wi2 = class {
      constructor() {
        this.i = -1;
        this.end = 0;
      }
      enqueue(e4, t3, n4) {
        this.i = 0, this.end = n4, this.current = e4, this.value = t3;
      }
      reset() {
        this.i = this.end, this.current = void 0, this.value = void 0;
      }
    };
    An = { texturePage: 0, texturePosition: { x: 0, y: 0 }, texturePositionClipSpace: { x: 0, y: 0 }, offset: { x: 0, y: 0 }, size: { x: 0, y: 0 }, sizeClipSpace: { x: 0, y: 0 } };
    rt3 = 2;
    ae3 = class i10 {
      constructor(e4, t3, n4) {
        this._document = e4;
        this._config = t3;
        this._unicodeService = n4;
        this._didWarmUp = false;
        this._cacheMap = new tt2();
        this._cacheMapCombined = new tt2();
        this._pages = [];
        this._activePages = [];
        this._workBoundingBox = { top: 0, left: 0, bottom: 0, right: 0 };
        this._workAttributeData = new he4();
        this._textureSize = 512;
        this._onAddTextureAtlasCanvas = new D3();
        this.onAddTextureAtlasCanvas = this._onAddTextureAtlasCanvas.event;
        this._onRemoveTextureAtlasCanvas = new D3();
        this.onRemoveTextureAtlasCanvas = this._onRemoveTextureAtlasCanvas.event;
        this._requestClearModel = false;
        this._createNewPage(), this._tmpCanvas = Sn2(e4, this._config.deviceCellWidth * 4 + rt3 * 2, this._config.deviceCellHeight + rt3 * 2), this._tmpCtx = F3(this._tmpCanvas.getContext("2d", { alpha: this._config.allowTransparency, willReadFrequently: true }));
      }
      get pages() {
        return this._pages;
      }
      dispose() {
        this._tmpCanvas.remove();
        for (let e4 of this.pages) e4.canvas.remove();
        this._onAddTextureAtlasCanvas.dispose();
      }
      warmUp() {
        this._didWarmUp || (this._doWarmUp(), this._didWarmUp = true);
      }
      _doWarmUp() {
        let e4 = new wn2();
        for (let t3 = 33; t3 < 126; t3++) e4.enqueue(() => {
          if (!this._cacheMap.get(t3, 0, 0, 0)) {
            let n4 = this._drawToCache(t3, 0, 0, 0, false, void 0);
            this._cacheMap.set(t3, 0, 0, 0, n4);
          }
        });
      }
      beginFrame() {
        return this._requestClearModel;
      }
      clearTexture() {
        if (!(this._pages[0].currentRow.x === 0 && this._pages[0].currentRow.y === 0)) {
          for (let e4 of this._pages) e4.clear();
          this._cacheMap.clear(), this._cacheMapCombined.clear(), this._didWarmUp = false;
        }
      }
      _createNewPage() {
        if (i10.maxAtlasPages && this._pages.length >= Math.max(4, i10.maxAtlasPages)) {
          let t3 = this._pages.filter((u3) => u3.canvas.width * 2 <= (i10.maxTextureSize || 4096)).sort((u3, c4) => c4.canvas.width !== u3.canvas.width ? c4.canvas.width - u3.canvas.width : c4.percentageUsed - u3.percentageUsed), n4 = -1, s18 = 0;
          for (let u3 = 0; u3 < t3.length; u3++) if (t3[u3].canvas.width !== s18) n4 = u3, s18 = t3[u3].canvas.width;
          else if (u3 - n4 === 3) break;
          let o6 = t3.slice(n4, n4 + 4), r7 = o6.map((u3) => u3.glyphs[0].texturePage).sort((u3, c4) => u3 > c4 ? 1 : -1), a3 = this.pages.length - o6.length, l4 = this._mergePages(o6, a3);
          l4.version++;
          for (let u3 = r7.length - 1; u3 >= 0; u3--) this._deletePage(r7[u3]);
          this.pages.push(l4), this._requestClearModel = true, this._onAddTextureAtlasCanvas.fire(l4.canvas);
        }
        let e4 = new ot3(this._document, this._textureSize);
        return this._pages.push(e4), this._activePages.push(e4), this._onAddTextureAtlasCanvas.fire(e4.canvas), e4;
      }
      _mergePages(e4, t3) {
        let n4 = e4[0].canvas.width * 2, s18 = new ot3(this._document, n4, e4);
        for (let [o6, r7] of e4.entries()) {
          let a3 = o6 * r7.canvas.width % n4, l4 = Math.floor(o6 / 2) * r7.canvas.height;
          s18.ctx.drawImage(r7.canvas, a3, l4);
          for (let c4 of r7.glyphs) c4.texturePage = t3, c4.sizeClipSpace.x = c4.size.x / n4, c4.sizeClipSpace.y = c4.size.y / n4, c4.texturePosition.x += a3, c4.texturePosition.y += l4, c4.texturePositionClipSpace.x = c4.texturePosition.x / n4, c4.texturePositionClipSpace.y = c4.texturePosition.y / n4;
          this._onRemoveTextureAtlasCanvas.fire(r7.canvas);
          let u3 = this._activePages.indexOf(r7);
          u3 !== -1 && this._activePages.splice(u3, 1);
        }
        return s18;
      }
      _deletePage(e4) {
        this._pages.splice(e4, 1);
        for (let t3 = e4; t3 < this._pages.length; t3++) {
          let n4 = this._pages[t3];
          for (let s18 of n4.glyphs) s18.texturePage--;
          n4.version++;
        }
      }
      getRasterizedGlyphCombinedChar(e4, t3, n4, s18, o6, r7) {
        return this._getFromCacheMap(this._cacheMapCombined, e4, t3, n4, s18, o6, r7);
      }
      getRasterizedGlyph(e4, t3, n4, s18, o6, r7) {
        return this._getFromCacheMap(this._cacheMap, e4, t3, n4, s18, o6, r7);
      }
      _getFromCacheMap(e4, t3, n4, s18, o6, r7, a3) {
        return st3 = e4.get(t3, n4, s18, o6), st3 || (st3 = this._drawToCache(t3, n4, s18, o6, r7, a3), e4.set(t3, n4, s18, o6, st3)), st3;
      }
      _getColorFromAnsiIndex(e4) {
        if (e4 >= this._config.colors.ansi.length) throw new Error("No color found for idx " + e4);
        return this._config.colors.ansi[e4];
      }
      _getBackgroundColor(e4, t3, n4, s18) {
        if (this._config.allowTransparency) return Z2;
        let o6;
        switch (e4) {
          case 16777216:
          case 33554432:
            o6 = this._getColorFromAnsiIndex(t3);
            break;
          case 50331648:
            let r7 = he4.toColorRGB(t3);
            o6 = X2.toColor(r7[0], r7[1], r7[2]);
            break;
          case 0:
          default:
            n4 ? o6 = Ue3.opaque(this._config.colors.foreground) : o6 = this._config.colors.background;
            break;
        }
        return this._config.allowTransparency || (o6 = Ue3.opaque(o6)), o6;
      }
      _getForegroundColor(e4, t3, n4, s18, o6, r7, a3, l4, u3, c4) {
        let d3 = this._getMinimumContrastColor(e4, t3, n4, s18, o6, r7, a3, u3, l4, c4);
        if (d3) return d3;
        let h4;
        switch (o6) {
          case 16777216:
          case 33554432:
            this._config.drawBoldTextInBrightColors && u3 && r7 < 8 && (r7 += 8), h4 = this._getColorFromAnsiIndex(r7);
            break;
          case 50331648:
            let f3 = he4.toColorRGB(r7);
            h4 = X2.toColor(f3[0], f3[1], f3[2]);
            break;
          case 0:
          default:
            a3 ? h4 = this._config.colors.background : h4 = this._config.colors.foreground;
        }
        return this._config.allowTransparency && (h4 = Ue3.opaque(h4)), l4 && (h4 = Ue3.multiplyOpacity(h4, gn3)), h4;
      }
      _resolveBackgroundRgba(e4, t3, n4) {
        switch (e4) {
          case 16777216:
          case 33554432:
            return this._getColorFromAnsiIndex(t3).rgba;
          case 50331648:
            return t3 << 8;
          case 0:
          default:
            return n4 ? this._config.colors.foreground.rgba : this._config.colors.background.rgba;
        }
      }
      _resolveForegroundRgba(e4, t3, n4, s18) {
        switch (e4) {
          case 16777216:
          case 33554432:
            return this._config.drawBoldTextInBrightColors && s18 && t3 < 8 && (t3 += 8), this._getColorFromAnsiIndex(t3).rgba;
          case 50331648:
            return t3 << 8;
          case 0:
          default:
            return n4 ? this._config.colors.background.rgba : this._config.colors.foreground.rgba;
        }
      }
      _getMinimumContrastColor(e4, t3, n4, s18, o6, r7, a3, l4, u3, c4) {
        if (this._config.minimumContrastRatio === 1 || c4) return;
        let d3 = this._getContrastCache(u3), h4 = d3.getColor(e4, s18);
        if (h4 !== void 0) return h4 || void 0;
        let f3 = this._resolveBackgroundRgba(t3, n4, a3), I2 = this._resolveForegroundRgba(o6, r7, a3, l4), L4 = Te3.ensureContrastRatio(f3, I2, this._config.minimumContrastRatio / (u3 ? 2 : 1));
        if (!L4) {
          d3.setColor(e4, s18, null);
          return;
        }
        let M4 = X2.toColor(L4 >> 24 & 255, L4 >> 16 & 255, L4 >> 8 & 255);
        return d3.setColor(e4, s18, M4), M4;
      }
      _getContrastCache(e4) {
        return e4 ? this._config.colors.halfContrastCache : this._config.colors.contrastCache;
      }
      _drawToCache(e4, t3, n4, s18, o6, r7) {
        let a3 = typeof e4 == "number" ? String.fromCharCode(e4) : e4;
        r7 && this._tmpCanvas.parentElement !== r7 && (this._tmpCanvas.style.display = "none", r7.append(this._tmpCanvas));
        let l4 = Math.min(this._config.deviceCellWidth * Math.max(a3.length, 2) + rt3 * 2, this._config.deviceMaxTextureSize);
        this._tmpCanvas.width < l4 && (this._tmpCanvas.width = l4);
        let u3 = Math.min(this._config.deviceCellHeight + rt3 * 4, this._textureSize);
        if (this._tmpCanvas.height < u3 && (this._tmpCanvas.height = u3), this._tmpCtx.save(), this._workAttributeData.fg = n4, this._workAttributeData.bg = t3, this._workAttributeData.extended.ext = s18, !!this._workAttributeData.isInvisible()) return An;
        let d3 = !!this._workAttributeData.isBold(), h4 = !!this._workAttributeData.isInverse(), f3 = !!this._workAttributeData.isDim(), I2 = !!this._workAttributeData.isItalic(), L4 = !!this._workAttributeData.isUnderline(), M4 = !!this._workAttributeData.isStrikethrough(), q2 = !!this._workAttributeData.isOverline(), S4 = this._workAttributeData.getFgColor(), W2 = this._workAttributeData.getFgColorMode(), E2 = this._workAttributeData.getBgColor(), y3 = this._workAttributeData.getBgColorMode();
        if (h4) {
          let x2 = S4;
          S4 = E2, E2 = x2;
          let R2 = W2;
          W2 = y3, y3 = R2;
        }
        let w3 = this._getBackgroundColor(y3, E2, h4, f3);
        this._tmpCtx.globalCompositeOperation = "copy", this._tmpCtx.fillStyle = w3.css, this._tmpCtx.fillRect(0, 0, this._tmpCanvas.width, this._tmpCanvas.height), this._tmpCtx.globalCompositeOperation = "source-over";
        let G3 = d3 ? this._config.fontWeightBold : this._config.fontWeight, ue4 = I2 ? "italic" : "";
        this._tmpCtx.font = `${ue4} ${G3} ${this._config.fontSize * this._config.devicePixelRatio}px ${this._config.fontFamily}`, this._tmpCtx.textBaseline = St3;
        let Se2 = a3.length === 1 && Rt3(a3.charCodeAt(0)), ce3 = a3.length === 1 && fn2(a3.charCodeAt(0)), we3 = this._getForegroundColor(t3, y3, E2, n4, W2, S4, h4, f3, d3, Dt3(a3.charCodeAt(0)));
        this._tmpCtx.fillStyle = we3.css;
        let A3 = ce3 ? 0 : rt3 * 2, se2 = false;
        this._config.customGlyphs !== false && (se2 = yn3(this._tmpCtx, a3, A3, A3, this._config.deviceCellWidth, this._config.deviceCellHeight, this._config.fontSize, this._config.devicePixelRatio));
        let v4 = !Se2, p3;
        if (typeof e4 == "number" ? p3 = this._unicodeService.wcwidth(e4) : p3 = this._unicodeService.getStringCellWidth(e4), L4) {
          this._tmpCtx.save();
          let x2 = Math.max(1, Math.floor(this._config.fontSize * this._config.devicePixelRatio / 15)), R2 = x2 % 2 === 1 ? 0.5 : 0;
          if (this._tmpCtx.lineWidth = x2, this._workAttributeData.isUnderlineColorDefault()) this._tmpCtx.strokeStyle = this._tmpCtx.fillStyle;
          else if (this._workAttributeData.isUnderlineColorRGB()) v4 = false, this._tmpCtx.strokeStyle = `rgb(${he4.toColorRGB(this._workAttributeData.getUnderlineColor()).join(",")})`;
          else {
            v4 = false;
            let ie3 = this._workAttributeData.getUnderlineColor();
            this._config.drawBoldTextInBrightColors && this._workAttributeData.isBold() && ie3 < 8 && (ie3 += 8), this._tmpCtx.strokeStyle = this._getColorFromAnsiIndex(ie3).css;
          }
          this._tmpCtx.beginPath();
          let $5 = A3, P4 = Math.ceil(A3 + this._config.deviceCharHeight) - R2 - (o6 ? x2 * 2 : 0), de4 = P4 + x2, Re3 = P4 + x2 * 2, oe = this._workAttributeData.getUnderlineVariantOffset();
          for (let ie3 = 0; ie3 < p3; ie3++) {
            this._tmpCtx.save();
            let N3 = $5 + ie3 * this._config.deviceCellWidth, ne3 = $5 + (ie3 + 1) * this._config.deviceCellWidth, di = N3 + this._config.deviceCellWidth / 2;
            switch (this._workAttributeData.extended.underlineStyle) {
              case 2:
                this._tmpCtx.moveTo(N3, P4), this._tmpCtx.lineTo(ne3, P4), this._tmpCtx.moveTo(N3, Re3), this._tmpCtx.lineTo(ne3, Re3);
                break;
              case 3:
                let ft3 = x2 <= 1 ? Re3 : Math.ceil(A3 + this._config.deviceCharHeight - x2 / 2) - R2, mt3 = x2 <= 1 ? P4 : Math.ceil(A3 + this._config.deviceCharHeight + x2 / 2) - R2, qi2 = new Path2D();
                qi2.rect(N3, P4, this._config.deviceCellWidth, Re3 - P4), this._tmpCtx.clip(qi2), this._tmpCtx.moveTo(N3 - this._config.deviceCellWidth / 2, de4), this._tmpCtx.bezierCurveTo(N3 - this._config.deviceCellWidth / 2, mt3, N3, mt3, N3, de4), this._tmpCtx.bezierCurveTo(N3, ft3, di, ft3, di, de4), this._tmpCtx.bezierCurveTo(di, mt3, ne3, mt3, ne3, de4), this._tmpCtx.bezierCurveTo(ne3, ft3, ne3 + this._config.deviceCellWidth / 2, ft3, ne3 + this._config.deviceCellWidth / 2, de4);
                break;
              case 4:
                let _t3 = oe === 0 ? 0 : oe >= x2 ? x2 * 2 - oe : x2 - oe;
                !(oe >= x2) === false || _t3 === 0 ? (this._tmpCtx.setLineDash([Math.round(x2), Math.round(x2)]), this._tmpCtx.moveTo(N3 + _t3, P4), this._tmpCtx.lineTo(ne3, P4)) : (this._tmpCtx.setLineDash([Math.round(x2), Math.round(x2)]), this._tmpCtx.moveTo(N3, P4), this._tmpCtx.lineTo(N3 + _t3, P4), this._tmpCtx.moveTo(N3 + _t3 + x2, P4), this._tmpCtx.lineTo(ne3, P4)), oe = bn2(ne3 - N3, x2, oe);
                break;
              case 5:
                let Er = 0.6, yr2 = 0.3, hi2 = ne3 - N3, ji2 = Math.floor(Er * hi2), Xi2 = Math.floor(yr2 * hi2), Ir2 = hi2 - ji2 - Xi2;
                this._tmpCtx.setLineDash([ji2, Xi2, Ir2]), this._tmpCtx.moveTo(N3, P4), this._tmpCtx.lineTo(ne3, P4);
                break;
              case 1:
              default:
                this._tmpCtx.moveTo(N3, P4), this._tmpCtx.lineTo(ne3, P4);
                break;
            }
            this._tmpCtx.stroke(), this._tmpCtx.restore();
          }
          if (this._tmpCtx.restore(), !se2 && this._config.fontSize >= 12 && !this._config.allowTransparency && a3 !== " ") {
            this._tmpCtx.save(), this._tmpCtx.textBaseline = "alphabetic";
            let ie3 = this._tmpCtx.measureText(a3);
            if (this._tmpCtx.restore(), "actualBoundingBoxDescent" in ie3 && ie3.actualBoundingBoxDescent > 0) {
              this._tmpCtx.save();
              let N3 = new Path2D();
              N3.rect($5, P4 - Math.ceil(x2 / 2), this._config.deviceCellWidth * p3, Re3 - P4 + Math.ceil(x2 / 2)), this._tmpCtx.clip(N3), this._tmpCtx.lineWidth = this._config.devicePixelRatio * 3, this._tmpCtx.strokeStyle = w3.css, this._tmpCtx.strokeText(a3, A3, A3 + this._config.deviceCharHeight), this._tmpCtx.restore();
            }
          }
        }
        if (q2) {
          let x2 = Math.max(1, Math.floor(this._config.fontSize * this._config.devicePixelRatio / 15)), R2 = x2 % 2 === 1 ? 0.5 : 0;
          this._tmpCtx.lineWidth = x2, this._tmpCtx.strokeStyle = this._tmpCtx.fillStyle, this._tmpCtx.beginPath(), this._tmpCtx.moveTo(A3, A3 + R2), this._tmpCtx.lineTo(A3 + this._config.deviceCharWidth * p3, A3 + R2), this._tmpCtx.stroke();
        }
        if (se2 || this._tmpCtx.fillText(a3, A3, A3 + this._config.deviceCharHeight), a3 === "_" && !this._config.allowTransparency) {
          let x2 = Di2(this._tmpCtx.getImageData(A3, A3, this._config.deviceCellWidth, this._config.deviceCellHeight), w3, we3, v4);
          if (x2) for (let R2 = 1; R2 <= 5 && (this._tmpCtx.save(), this._tmpCtx.fillStyle = w3.css, this._tmpCtx.fillRect(0, 0, this._tmpCanvas.width, this._tmpCanvas.height), this._tmpCtx.restore(), this._tmpCtx.fillText(a3, A3, A3 + this._config.deviceCharHeight - R2), x2 = Di2(this._tmpCtx.getImageData(A3, A3, this._config.deviceCellWidth, this._config.deviceCellHeight), w3, we3, v4), !!x2); R2++) ;
        }
        if (M4) {
          let x2 = Math.max(1, Math.floor(this._config.fontSize * this._config.devicePixelRatio / 10)), R2 = this._tmpCtx.lineWidth % 2 === 1 ? 0.5 : 0;
          this._tmpCtx.lineWidth = x2, this._tmpCtx.strokeStyle = this._tmpCtx.fillStyle, this._tmpCtx.beginPath(), this._tmpCtx.moveTo(A3, A3 + Math.floor(this._config.deviceCharHeight / 2) - R2), this._tmpCtx.lineTo(A3 + this._config.deviceCharWidth * p3, A3 + Math.floor(this._config.deviceCharHeight / 2) - R2), this._tmpCtx.stroke();
        }
        this._tmpCtx.restore();
        let g3 = this._tmpCtx.getImageData(0, 0, this._tmpCanvas.width, this._tmpCanvas.height), b4;
        if (this._config.allowTransparency ? b4 = Jr2(g3) : b4 = Di2(g3, w3, we3, v4), b4) return An;
        let m2 = this._findGlyphBoundingBox(g3, this._workBoundingBox, l4, ce3, se2, A3), _4, T2;
        for (; ; ) {
          if (this._activePages.length === 0) {
            let x2 = this._createNewPage();
            _4 = x2, T2 = x2.currentRow, T2.height = m2.size.y;
            break;
          }
          _4 = this._activePages[this._activePages.length - 1], T2 = _4.currentRow;
          for (let x2 of this._activePages) m2.size.y <= x2.currentRow.height && (_4 = x2, T2 = x2.currentRow);
          for (let x2 = this._activePages.length - 1; x2 >= 0; x2--) for (let R2 of this._activePages[x2].fixedRows) R2.height <= T2.height && m2.size.y <= R2.height && (_4 = this._activePages[x2], T2 = R2);
          if (m2.size.x > this._textureSize) {
            this._overflowSizePage || (this._overflowSizePage = new ot3(this._document, this._config.deviceMaxTextureSize), this.pages.push(this._overflowSizePage), this._requestClearModel = true, this._onAddTextureAtlasCanvas.fire(this._overflowSizePage.canvas)), _4 = this._overflowSizePage, T2 = this._overflowSizePage.currentRow, T2.x + m2.size.x >= _4.canvas.width && (T2.x = 0, T2.y += T2.height, T2.height = 0);
            break;
          }
          if (T2.y + m2.size.y >= _4.canvas.height || T2.height > m2.size.y + 2) {
            let x2 = false;
            if (_4.currentRow.y + _4.currentRow.height + m2.size.y >= _4.canvas.height) {
              let R2;
              for (let $5 of this._activePages) if ($5.currentRow.y + $5.currentRow.height + m2.size.y < $5.canvas.height) {
                R2 = $5;
                break;
              }
              if (R2) _4 = R2;
              else if (i10.maxAtlasPages && this._pages.length >= i10.maxAtlasPages && T2.y + m2.size.y <= _4.canvas.height && T2.height >= m2.size.y && T2.x + m2.size.x <= _4.canvas.width) x2 = true;
              else {
                let $5 = this._createNewPage();
                _4 = $5, T2 = $5.currentRow, T2.height = m2.size.y, x2 = true;
              }
            }
            x2 || (_4.currentRow.height > 0 && _4.fixedRows.push(_4.currentRow), T2 = { x: 0, y: _4.currentRow.y + _4.currentRow.height, height: m2.size.y }, _4.fixedRows.push(T2), _4.currentRow = { x: 0, y: T2.y + T2.height, height: 0 });
          }
          if (T2.x + m2.size.x <= _4.canvas.width) break;
          T2 === _4.currentRow ? (T2.x = 0, T2.y += T2.height, T2.height = 0) : _4.fixedRows.splice(_4.fixedRows.indexOf(T2), 1);
        }
        return m2.texturePage = this._pages.indexOf(_4), m2.texturePosition.x = T2.x, m2.texturePosition.y = T2.y, m2.texturePositionClipSpace.x = T2.x / _4.canvas.width, m2.texturePositionClipSpace.y = T2.y / _4.canvas.height, m2.sizeClipSpace.x /= _4.canvas.width, m2.sizeClipSpace.y /= _4.canvas.height, T2.height = Math.max(T2.height, m2.size.y), T2.x += m2.size.x, _4.ctx.putImageData(g3, m2.texturePosition.x - this._workBoundingBox.left, m2.texturePosition.y - this._workBoundingBox.top, this._workBoundingBox.left, this._workBoundingBox.top, m2.size.x, m2.size.y), _4.addGlyph(m2), _4.version++, m2;
      }
      _findGlyphBoundingBox(e4, t3, n4, s18, o6, r7) {
        t3.top = 0;
        let a3 = s18 ? this._config.deviceCellHeight : this._tmpCanvas.height, l4 = s18 ? this._config.deviceCellWidth : n4, u3 = false;
        for (let c4 = 0; c4 < a3; c4++) {
          for (let d3 = 0; d3 < l4; d3++) {
            let h4 = c4 * this._tmpCanvas.width * 4 + d3 * 4 + 3;
            if (e4.data[h4] !== 0) {
              t3.top = c4, u3 = true;
              break;
            }
          }
          if (u3) break;
        }
        t3.left = 0, u3 = false;
        for (let c4 = 0; c4 < r7 + l4; c4++) {
          for (let d3 = 0; d3 < a3; d3++) {
            let h4 = d3 * this._tmpCanvas.width * 4 + c4 * 4 + 3;
            if (e4.data[h4] !== 0) {
              t3.left = c4, u3 = true;
              break;
            }
          }
          if (u3) break;
        }
        t3.right = l4, u3 = false;
        for (let c4 = r7 + l4 - 1; c4 >= r7; c4--) {
          for (let d3 = 0; d3 < a3; d3++) {
            let h4 = d3 * this._tmpCanvas.width * 4 + c4 * 4 + 3;
            if (e4.data[h4] !== 0) {
              t3.right = c4, u3 = true;
              break;
            }
          }
          if (u3) break;
        }
        t3.bottom = a3, u3 = false;
        for (let c4 = a3 - 1; c4 >= 0; c4--) {
          for (let d3 = 0; d3 < l4; d3++) {
            let h4 = c4 * this._tmpCanvas.width * 4 + d3 * 4 + 3;
            if (e4.data[h4] !== 0) {
              t3.bottom = c4, u3 = true;
              break;
            }
          }
          if (u3) break;
        }
        return { texturePage: 0, texturePosition: { x: 0, y: 0 }, texturePositionClipSpace: { x: 0, y: 0 }, size: { x: t3.right - t3.left + 1, y: t3.bottom - t3.top + 1 }, sizeClipSpace: { x: t3.right - t3.left + 1, y: t3.bottom - t3.top + 1 }, offset: { x: -t3.left + r7 + (s18 || o6 ? Math.floor((this._config.deviceCellWidth - this._config.deviceCharWidth) / 2) : 0), y: -t3.top + r7 + (s18 || o6 ? this._config.lineHeight === 1 ? 0 : Math.round((this._config.deviceCellHeight - this._config.deviceCharHeight) / 2) : 0) } };
      }
    };
    ot3 = class {
      constructor(e4, t3, n4) {
        this._usedPixels = 0;
        this._glyphs = [];
        this.version = 0;
        this.currentRow = { x: 0, y: 0, height: 0 };
        this.fixedRows = [];
        if (n4) for (let s18 of n4) this._glyphs.push(...s18.glyphs), this._usedPixels += s18._usedPixels;
        this.canvas = Sn2(e4, t3, t3), this.ctx = F3(this.canvas.getContext("2d", { alpha: true }));
      }
      get percentageUsed() {
        return this._usedPixels / (this.canvas.width * this.canvas.height);
      }
      get glyphs() {
        return this._glyphs;
      }
      addGlyph(e4) {
        this._glyphs.push(e4), this._usedPixels += e4.size.x * e4.size.y;
      }
      clear() {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height), this.currentRow.x = 0, this.currentRow.y = 0, this.currentRow.height = 0, this.fixedRows.length = 0, this.version++;
      }
    };
    le2 = [];
    Ut2 = 600;
    Ht = class {
      constructor(e4, t3) {
        this._renderCallback = e4;
        this._coreBrowserService = t3;
        this.isCursorVisible = true, this._coreBrowserService.isFocused && this._restartInterval();
      }
      get isPaused() {
        return !(this._blinkStartTimeout || this._blinkInterval);
      }
      dispose() {
        this._blinkInterval && (this._coreBrowserService.window.clearInterval(this._blinkInterval), this._blinkInterval = void 0), this._blinkStartTimeout && (this._coreBrowserService.window.clearTimeout(this._blinkStartTimeout), this._blinkStartTimeout = void 0), this._animationFrame && (this._coreBrowserService.window.cancelAnimationFrame(this._animationFrame), this._animationFrame = void 0);
      }
      restartBlinkAnimation() {
        this.isPaused || (this._animationTimeRestarted = Date.now(), this.isCursorVisible = true, this._animationFrame || (this._animationFrame = this._coreBrowserService.window.requestAnimationFrame(() => {
          this._renderCallback(), this._animationFrame = void 0;
        })));
      }
      _restartInterval(e4 = Ut2) {
        this._blinkInterval && (this._coreBrowserService.window.clearInterval(this._blinkInterval), this._blinkInterval = void 0), this._blinkStartTimeout = this._coreBrowserService.window.setTimeout(() => {
          if (this._animationTimeRestarted) {
            let t3 = Ut2 - (Date.now() - this._animationTimeRestarted);
            if (this._animationTimeRestarted = void 0, t3 > 0) {
              this._restartInterval(t3);
              return;
            }
          }
          this.isCursorVisible = false, this._animationFrame = this._coreBrowserService.window.requestAnimationFrame(() => {
            this._renderCallback(), this._animationFrame = void 0;
          }), this._blinkInterval = this._coreBrowserService.window.setInterval(() => {
            if (this._animationTimeRestarted) {
              let t3 = Ut2 - (Date.now() - this._animationTimeRestarted);
              this._animationTimeRestarted = void 0, this._restartInterval(t3);
              return;
            }
            this.isCursorVisible = !this.isCursorVisible, this._animationFrame = this._coreBrowserService.window.requestAnimationFrame(() => {
              this._renderCallback(), this._animationFrame = void 0;
            });
          }, Ut2);
        }, e4);
      }
      pause() {
        this.isCursorVisible = true, this._blinkInterval && (this._coreBrowserService.window.clearInterval(this._blinkInterval), this._blinkInterval = void 0), this._blinkStartTimeout && (this._coreBrowserService.window.clearTimeout(this._blinkStartTimeout), this._blinkStartTimeout = void 0), this._animationFrame && (this._coreBrowserService.window.cancelAnimationFrame(this._animationFrame), this._animationFrame = void 0);
      }
      resume() {
        this.pause(), this._animationTimeRestarted = void 0, this._restartInterval(), this.restartBlinkAnimation();
      }
    };
    at3 = class i11 extends he4 {
      constructor() {
        super(...arguments);
        this.content = 0;
        this.fg = 0;
        this.bg = 0;
        this.extended = new it2();
        this.combinedData = "";
      }
      static fromCharData(t3) {
        let n4 = new i11();
        return n4.setFromCharData(t3), n4;
      }
      isCombined() {
        return this.content & 2097152;
      }
      getWidth() {
        return this.content >> 22;
      }
      getChars() {
        return this.content & 2097152 ? this.combinedData : this.content & 2097151 ? kn(this.content & 2097151) : "";
      }
      getCode() {
        return this.isCombined() ? this.combinedData.charCodeAt(this.combinedData.length - 1) : this.content & 2097151;
      }
      setFromCharData(t3) {
        this.fg = t3[0], this.bg = 0;
        let n4 = false;
        if (t3[1].length > 2) n4 = true;
        else if (t3[1].length === 2) {
          let s18 = t3[1].charCodeAt(0);
          if (55296 <= s18 && s18 <= 56319) {
            let o6 = t3[1].charCodeAt(1);
            56320 <= o6 && o6 <= 57343 ? this.content = (s18 - 55296) * 1024 + o6 - 56320 + 65536 | t3[2] << 22 : n4 = true;
          } else n4 = true;
        } else this.content = t3[1].charCodeAt(0) | t3[2] << 22;
        n4 && (this.combinedData = t3[1], this.content = 2097152 | t3[2] << 22);
      }
      getAsCharData() {
        return [this.fg, this.getChars(), this.getWidth(), this.getCode()];
      }
    };
    Gt2 = new Float32Array([2, 0, 0, 0, 0, -2, 0, 0, 0, 0, 1, 0, -1, 1, 0, 1]);
    Wt3 = class {
      constructor(e4) {
        this.texture = e4, this.version = -1;
      }
    };
    is2 = `#version 300 es
layout (location = 0) in vec2 a_unitquad;
layout (location = 1) in vec2 a_cellpos;
layout (location = 2) in vec2 a_offset;
layout (location = 3) in vec2 a_size;
layout (location = 4) in float a_texpage;
layout (location = 5) in vec2 a_texcoord;
layout (location = 6) in vec2 a_texsize;

uniform mat4 u_projection;
uniform vec2 u_resolution;

out vec2 v_texcoord;
flat out int v_texpage;

void main() {
  vec2 zeroToOne = (a_offset / u_resolution) + a_cellpos + (a_unitquad * a_size);
  gl_Position = u_projection * vec4(zeroToOne, 0.0, 1.0);
  v_texpage = int(a_texpage);
  v_texcoord = a_texcoord + a_unitquad * a_texsize;
}`;
    De3 = 11;
    Ve2 = De3 * Float32Array.BYTES_PER_ELEMENT;
    rs2 = 2;
    H4 = 0;
    Fi2 = 0;
    lt2 = 0;
    Kt2 = class extends B5 {
      constructor(t3, n4, s18, o6) {
        super();
        this._terminal = t3;
        this._gl = n4;
        this._dimensions = s18;
        this._optionsService = o6;
        this._activeBuffer = 0;
        this._vertices = { count: 0, attributes: new Float32Array(0), attributesBuffers: [new Float32Array(0), new Float32Array(0)] };
        let r7 = this._gl;
        ae3.maxAtlasPages === void 0 && (ae3.maxAtlasPages = Math.min(32, F3(r7.getParameter(r7.MAX_TEXTURE_IMAGE_UNITS))), ae3.maxTextureSize = F3(r7.getParameter(r7.MAX_TEXTURE_SIZE))), this._program = F3($t2(r7, is2, ns2(ae3.maxAtlasPages))), this._register(O2(() => r7.deleteProgram(this._program))), this._projectionLocation = F3(r7.getUniformLocation(this._program, "u_projection")), this._resolutionLocation = F3(r7.getUniformLocation(this._program, "u_resolution")), this._textureLocation = F3(r7.getUniformLocation(this._program, "u_texture")), this._vertexArrayObject = r7.createVertexArray(), r7.bindVertexArray(this._vertexArrayObject);
        let a3 = new Float32Array([0, 0, 1, 0, 0, 1, 1, 1]), l4 = r7.createBuffer();
        this._register(O2(() => r7.deleteBuffer(l4))), r7.bindBuffer(r7.ARRAY_BUFFER, l4), r7.bufferData(r7.ARRAY_BUFFER, a3, r7.STATIC_DRAW), r7.enableVertexAttribArray(0), r7.vertexAttribPointer(0, 2, this._gl.FLOAT, false, 0, 0);
        let u3 = new Uint8Array([0, 1, 2, 3]), c4 = r7.createBuffer();
        this._register(O2(() => r7.deleteBuffer(c4))), r7.bindBuffer(r7.ELEMENT_ARRAY_BUFFER, c4), r7.bufferData(r7.ELEMENT_ARRAY_BUFFER, u3, r7.STATIC_DRAW), this._attributesBuffer = F3(r7.createBuffer()), this._register(O2(() => r7.deleteBuffer(this._attributesBuffer))), r7.bindBuffer(r7.ARRAY_BUFFER, this._attributesBuffer), r7.enableVertexAttribArray(2), r7.vertexAttribPointer(2, 2, r7.FLOAT, false, Ve2, 0), r7.vertexAttribDivisor(2, 1), r7.enableVertexAttribArray(3), r7.vertexAttribPointer(3, 2, r7.FLOAT, false, Ve2, 2 * Float32Array.BYTES_PER_ELEMENT), r7.vertexAttribDivisor(3, 1), r7.enableVertexAttribArray(4), r7.vertexAttribPointer(4, 1, r7.FLOAT, false, Ve2, 4 * Float32Array.BYTES_PER_ELEMENT), r7.vertexAttribDivisor(4, 1), r7.enableVertexAttribArray(5), r7.vertexAttribPointer(5, 2, r7.FLOAT, false, Ve2, 5 * Float32Array.BYTES_PER_ELEMENT), r7.vertexAttribDivisor(5, 1), r7.enableVertexAttribArray(6), r7.vertexAttribPointer(6, 2, r7.FLOAT, false, Ve2, 7 * Float32Array.BYTES_PER_ELEMENT), r7.vertexAttribDivisor(6, 1), r7.enableVertexAttribArray(1), r7.vertexAttribPointer(1, 2, r7.FLOAT, false, Ve2, 9 * Float32Array.BYTES_PER_ELEMENT), r7.vertexAttribDivisor(1, 1), r7.useProgram(this._program);
        let d3 = new Int32Array(ae3.maxAtlasPages);
        for (let h4 = 0; h4 < ae3.maxAtlasPages; h4++) d3[h4] = h4;
        r7.uniform1iv(this._textureLocation, d3), r7.uniformMatrix4fv(this._projectionLocation, false, Gt2), this._atlasTextures = [];
        for (let h4 = 0; h4 < ae3.maxAtlasPages; h4++) {
          let f3 = new Wt3(F3(r7.createTexture()));
          this._register(O2(() => r7.deleteTexture(f3.texture))), r7.activeTexture(r7.TEXTURE0 + h4), r7.bindTexture(r7.TEXTURE_2D, f3.texture), r7.texParameteri(r7.TEXTURE_2D, r7.TEXTURE_WRAP_S, r7.CLAMP_TO_EDGE), r7.texParameteri(r7.TEXTURE_2D, r7.TEXTURE_WRAP_T, r7.CLAMP_TO_EDGE), r7.texImage2D(r7.TEXTURE_2D, 0, r7.RGBA, 1, 1, 0, r7.RGBA, r7.UNSIGNED_BYTE, new Uint8Array([255, 0, 0, 255])), this._atlasTextures[h4] = f3;
        }
        r7.enable(r7.BLEND), r7.blendFunc(r7.SRC_ALPHA, r7.ONE_MINUS_SRC_ALPHA), this.handleResize();
      }
      beginFrame() {
        return this._atlas ? this._atlas.beginFrame() : true;
      }
      updateCell(t3, n4, s18, o6, r7, a3, l4, u3, c4) {
        this._updateCell(this._vertices.attributes, t3, n4, s18, o6, r7, a3, l4, u3, c4);
      }
      _updateCell(t3, n4, s18, o6, r7, a3, l4, u3, c4, d3) {
        if (H4 = (s18 * this._terminal.cols + n4) * De3, o6 === 0 || o6 === void 0) {
          t3.fill(0, H4, H4 + De3 - 1 - rs2);
          return;
        }
        this._atlas && (u3 && u3.length > 1 ? k4 = this._atlas.getRasterizedGlyphCombinedChar(u3, r7, a3, l4, false, this._terminal.element) : k4 = this._atlas.getRasterizedGlyph(o6, r7, a3, l4, false, this._terminal.element), Fi2 = Math.floor((this._dimensions.device.cell.width - this._dimensions.device.char.width) / 2), r7 !== d3 && k4.offset.x > Fi2 ? (lt2 = k4.offset.x - Fi2, t3[H4] = -(k4.offset.x - lt2) + this._dimensions.device.char.left, t3[H4 + 1] = -k4.offset.y + this._dimensions.device.char.top, t3[H4 + 2] = (k4.size.x - lt2) / this._dimensions.device.canvas.width, t3[H4 + 3] = k4.size.y / this._dimensions.device.canvas.height, t3[H4 + 4] = k4.texturePage, t3[H4 + 5] = k4.texturePositionClipSpace.x + lt2 / this._atlas.pages[k4.texturePage].canvas.width, t3[H4 + 6] = k4.texturePositionClipSpace.y, t3[H4 + 7] = k4.sizeClipSpace.x - lt2 / this._atlas.pages[k4.texturePage].canvas.width, t3[H4 + 8] = k4.sizeClipSpace.y) : (t3[H4] = -k4.offset.x + this._dimensions.device.char.left, t3[H4 + 1] = -k4.offset.y + this._dimensions.device.char.top, t3[H4 + 2] = k4.size.x / this._dimensions.device.canvas.width, t3[H4 + 3] = k4.size.y / this._dimensions.device.canvas.height, t3[H4 + 4] = k4.texturePage, t3[H4 + 5] = k4.texturePositionClipSpace.x, t3[H4 + 6] = k4.texturePositionClipSpace.y, t3[H4 + 7] = k4.sizeClipSpace.x, t3[H4 + 8] = k4.sizeClipSpace.y), this._optionsService.rawOptions.rescaleOverlappingGlyphs && mn2(o6, c4, k4.size.x, this._dimensions.device.cell.width) && (t3[H4 + 2] = (this._dimensions.device.cell.width - 1) / this._dimensions.device.canvas.width));
      }
      clear() {
        let t3 = this._terminal, n4 = t3.cols * t3.rows * De3;
        this._vertices.count !== n4 ? this._vertices.attributes = new Float32Array(n4) : this._vertices.attributes.fill(0);
        let s18 = 0;
        for (; s18 < this._vertices.attributesBuffers.length; s18++) this._vertices.count !== n4 ? this._vertices.attributesBuffers[s18] = new Float32Array(n4) : this._vertices.attributesBuffers[s18].fill(0);
        this._vertices.count = n4, s18 = 0;
        for (let o6 = 0; o6 < t3.rows; o6++) for (let r7 = 0; r7 < t3.cols; r7++) this._vertices.attributes[s18 + 9] = r7 / t3.cols, this._vertices.attributes[s18 + 10] = o6 / t3.rows, s18 += De3;
      }
      handleResize() {
        let t3 = this._gl;
        t3.useProgram(this._program), t3.viewport(0, 0, t3.canvas.width, t3.canvas.height), t3.uniform2f(this._resolutionLocation, t3.canvas.width, t3.canvas.height), this.clear();
      }
      render(t3) {
        if (!this._atlas) return;
        let n4 = this._gl;
        n4.useProgram(this._program), n4.bindVertexArray(this._vertexArrayObject), this._activeBuffer = (this._activeBuffer + 1) % 2;
        let s18 = this._vertices.attributesBuffers[this._activeBuffer], o6 = 0;
        for (let r7 = 0; r7 < t3.lineLengths.length; r7++) {
          let a3 = r7 * this._terminal.cols * De3, l4 = this._vertices.attributes.subarray(a3, a3 + t3.lineLengths[r7] * De3);
          s18.set(l4, o6), o6 += l4.length;
        }
        n4.bindBuffer(n4.ARRAY_BUFFER, this._attributesBuffer), n4.bufferData(n4.ARRAY_BUFFER, s18.subarray(0, o6), n4.STREAM_DRAW);
        for (let r7 = 0; r7 < this._atlas.pages.length; r7++) this._atlas.pages[r7].version !== this._atlasTextures[r7].version && this._bindAtlasPageTexture(n4, this._atlas, r7);
        n4.drawElementsInstanced(n4.TRIANGLE_STRIP, 4, n4.UNSIGNED_BYTE, 0, o6 / De3);
      }
      setAtlas(t3) {
        this._atlas = t3;
        for (let n4 of this._atlasTextures) n4.version = -1;
      }
      _bindAtlasPageTexture(t3, n4, s18) {
        t3.activeTexture(t3.TEXTURE0 + s18), t3.bindTexture(t3.TEXTURE_2D, this._atlasTextures[s18].texture), t3.texParameteri(t3.TEXTURE_2D, t3.TEXTURE_WRAP_S, t3.CLAMP_TO_EDGE), t3.texParameteri(t3.TEXTURE_2D, t3.TEXTURE_WRAP_T, t3.CLAMP_TO_EDGE), t3.texImage2D(t3.TEXTURE_2D, 0, t3.RGBA, t3.RGBA, t3.UNSIGNED_BYTE, n4.pages[s18].canvas), t3.generateMipmap(t3.TEXTURE_2D), this._atlasTextures[s18].version = n4.pages[s18].version;
      }
      setDimensions(t3) {
        this._dimensions = t3;
      }
    };
    ki2 = class {
      constructor() {
        this.clear();
      }
      clear() {
        this.hasSelection = false, this.columnSelectMode = false, this.viewportStartRow = 0, this.viewportEndRow = 0, this.viewportCappedStartRow = 0, this.viewportCappedEndRow = 0, this.startCol = 0, this.endCol = 0, this.selectionStart = void 0, this.selectionEnd = void 0;
      }
      update(e4, t3, n4, s18 = false) {
        if (this.selectionStart = t3, this.selectionEnd = n4, !t3 || !n4 || t3[0] === n4[0] && t3[1] === n4[1]) {
          this.clear();
          return;
        }
        let o6 = e4.buffers.active.ydisp, r7 = t3[1] - o6, a3 = n4[1] - o6, l4 = Math.max(r7, 0), u3 = Math.min(a3, e4.rows - 1);
        if (l4 >= e4.rows || u3 < 0) {
          this.clear();
          return;
        }
        this.hasSelection = true, this.columnSelectMode = s18, this.viewportStartRow = r7, this.viewportEndRow = a3, this.viewportCappedStartRow = l4, this.viewportCappedEndRow = u3, this.startCol = t3[0], this.endCol = n4[0];
      }
      isCellSelected(e4, t3, n4) {
        return this.hasSelection ? (n4 -= e4.buffer.active.viewportY, this.columnSelectMode ? this.startCol <= this.endCol ? t3 >= this.startCol && n4 >= this.viewportCappedStartRow && t3 < this.endCol && n4 <= this.viewportCappedEndRow : t3 < this.startCol && n4 >= this.viewportCappedStartRow && t3 >= this.endCol && n4 <= this.viewportCappedEndRow : n4 > this.viewportStartRow && n4 < this.viewportEndRow || this.viewportStartRow === this.viewportEndRow && n4 === this.viewportStartRow && t3 >= this.startCol && t3 < this.endCol || this.viewportStartRow < this.viewportEndRow && n4 === this.viewportEndRow && t3 < this.endCol || this.viewportStartRow < this.viewportEndRow && n4 === this.viewportStartRow && t3 >= this.startCol) : false;
      }
    };
    Ce3 = 4;
    ze2 = 1;
    qe3 = 2;
    Ct3 = 3;
    Un2 = 2147483648;
    Vt2 = class {
      constructor() {
        this.cells = new Uint32Array(0), this.lineLengths = new Uint32Array(0), this.selection = Nn();
      }
      resize(e4, t3) {
        let n4 = e4 * t3 * Ce3;
        n4 !== this.cells.length && (this.cells = new Uint32Array(n4), this.lineLengths = new Uint32Array(t3));
      }
      clear() {
        this.cells.fill(0, 0), this.lineLengths.fill(0, 0);
      }
    };
    ss2 = `#version 300 es
layout (location = 0) in vec2 a_position;
layout (location = 1) in vec2 a_size;
layout (location = 2) in vec4 a_color;
layout (location = 3) in vec2 a_unitquad;

uniform mat4 u_projection;

out vec4 v_color;

void main() {
  vec2 zeroToOne = a_position + (a_unitquad * a_size);
  gl_Position = u_projection * vec4(zeroToOne, 0.0, 1.0);
  v_color = a_color;
}`;
    os2 = `#version 300 es
precision lowp float;

in vec4 v_color;

out vec4 outColor;

void main() {
  outColor = v_color;
}`;
    Ee3 = 8;
    Pi2 = Ee3 * Float32Array.BYTES_PER_ELEMENT;
    as2 = 20 * Ee3;
    zt2 = class {
      constructor() {
        this.attributes = new Float32Array(as2), this.count = 0;
      }
    };
    xe3 = 0;
    Hn2 = 0;
    Wn2 = 0;
    Gn2 = 0;
    $n2 = 0;
    Kn2 = 0;
    Vn2 = 0;
    qt2 = class extends B5 {
      constructor(t3, n4, s18, o6) {
        super();
        this._terminal = t3;
        this._gl = n4;
        this._dimensions = s18;
        this._themeService = o6;
        this._vertices = new zt2();
        this._verticesCursor = new zt2();
        let r7 = this._gl;
        this._program = F3($t2(r7, ss2, os2)), this._register(O2(() => r7.deleteProgram(this._program))), this._projectionLocation = F3(r7.getUniformLocation(this._program, "u_projection")), this._vertexArrayObject = r7.createVertexArray(), r7.bindVertexArray(this._vertexArrayObject);
        let a3 = new Float32Array([0, 0, 1, 0, 0, 1, 1, 1]), l4 = r7.createBuffer();
        this._register(O2(() => r7.deleteBuffer(l4))), r7.bindBuffer(r7.ARRAY_BUFFER, l4), r7.bufferData(r7.ARRAY_BUFFER, a3, r7.STATIC_DRAW), r7.enableVertexAttribArray(3), r7.vertexAttribPointer(3, 2, this._gl.FLOAT, false, 0, 0);
        let u3 = new Uint8Array([0, 1, 2, 3]), c4 = r7.createBuffer();
        this._register(O2(() => r7.deleteBuffer(c4))), r7.bindBuffer(r7.ELEMENT_ARRAY_BUFFER, c4), r7.bufferData(r7.ELEMENT_ARRAY_BUFFER, u3, r7.STATIC_DRAW), this._attributesBuffer = F3(r7.createBuffer()), this._register(O2(() => r7.deleteBuffer(this._attributesBuffer))), r7.bindBuffer(r7.ARRAY_BUFFER, this._attributesBuffer), r7.enableVertexAttribArray(0), r7.vertexAttribPointer(0, 2, r7.FLOAT, false, Pi2, 0), r7.vertexAttribDivisor(0, 1), r7.enableVertexAttribArray(1), r7.vertexAttribPointer(1, 2, r7.FLOAT, false, Pi2, 2 * Float32Array.BYTES_PER_ELEMENT), r7.vertexAttribDivisor(1, 1), r7.enableVertexAttribArray(2), r7.vertexAttribPointer(2, 4, r7.FLOAT, false, Pi2, 4 * Float32Array.BYTES_PER_ELEMENT), r7.vertexAttribDivisor(2, 1), this._updateCachedColors(o6.colors), this._register(this._themeService.onChangeColors((d3) => {
          this._updateCachedColors(d3), this._updateViewportRectangle();
        }));
      }
      renderBackgrounds() {
        this._renderVertices(this._vertices);
      }
      renderCursor() {
        this._renderVertices(this._verticesCursor);
      }
      _renderVertices(t3) {
        let n4 = this._gl;
        n4.useProgram(this._program), n4.bindVertexArray(this._vertexArrayObject), n4.uniformMatrix4fv(this._projectionLocation, false, Gt2), n4.bindBuffer(n4.ARRAY_BUFFER, this._attributesBuffer), n4.bufferData(n4.ARRAY_BUFFER, t3.attributes, n4.DYNAMIC_DRAW), n4.drawElementsInstanced(this._gl.TRIANGLE_STRIP, 4, n4.UNSIGNED_BYTE, 0, t3.count);
      }
      handleResize() {
        this._updateViewportRectangle();
      }
      setDimensions(t3) {
        this._dimensions = t3;
      }
      _updateCachedColors(t3) {
        this._bgFloat = this._colorToFloat32Array(t3.background), this._cursorFloat = this._colorToFloat32Array(t3.cursor);
      }
      _updateViewportRectangle() {
        this._addRectangleFloat(this._vertices.attributes, 0, 0, 0, this._terminal.cols * this._dimensions.device.cell.width, this._terminal.rows * this._dimensions.device.cell.height, this._bgFloat);
      }
      updateBackgrounds(t3) {
        let n4 = this._terminal, s18 = this._vertices, o6 = 1, r7, a3, l4, u3, c4, d3, h4, f3, I2, L4, M4;
        for (r7 = 0; r7 < n4.rows; r7++) {
          for (l4 = -1, u3 = 0, c4 = 0, d3 = false, a3 = 0; a3 < n4.cols; a3++) h4 = (r7 * n4.cols + a3) * Ce3, f3 = t3.cells[h4 + ze2], I2 = t3.cells[h4 + qe3], L4 = !!(I2 & 67108864), (f3 !== u3 || I2 !== c4 && (d3 || L4)) && ((u3 !== 0 || d3 && c4 !== 0) && (M4 = o6++ * Ee3, this._updateRectangle(s18, M4, c4, u3, l4, a3, r7)), l4 = a3, u3 = f3, c4 = I2, d3 = L4);
          (u3 !== 0 || d3 && c4 !== 0) && (M4 = o6++ * Ee3, this._updateRectangle(s18, M4, c4, u3, l4, n4.cols, r7));
        }
        s18.count = o6;
      }
      updateCursor(t3) {
        let n4 = this._verticesCursor, s18 = t3.cursor;
        if (!s18 || s18.style === "block") {
          n4.count = 0;
          return;
        }
        let o6, r7 = 0;
        (s18.style === "bar" || s18.style === "outline") && (o6 = r7++ * Ee3, this._addRectangleFloat(n4.attributes, o6, s18.x * this._dimensions.device.cell.width, s18.y * this._dimensions.device.cell.height, s18.style === "bar" ? s18.dpr * s18.cursorWidth : s18.dpr, this._dimensions.device.cell.height, this._cursorFloat)), (s18.style === "underline" || s18.style === "outline") && (o6 = r7++ * Ee3, this._addRectangleFloat(n4.attributes, o6, s18.x * this._dimensions.device.cell.width, (s18.y + 1) * this._dimensions.device.cell.height - s18.dpr, s18.width * this._dimensions.device.cell.width, s18.dpr, this._cursorFloat)), s18.style === "outline" && (o6 = r7++ * Ee3, this._addRectangleFloat(n4.attributes, o6, s18.x * this._dimensions.device.cell.width, s18.y * this._dimensions.device.cell.height, s18.width * this._dimensions.device.cell.width, s18.dpr, this._cursorFloat), o6 = r7++ * Ee3, this._addRectangleFloat(n4.attributes, o6, (s18.x + s18.width) * this._dimensions.device.cell.width - s18.dpr, s18.y * this._dimensions.device.cell.height, s18.dpr, this._dimensions.device.cell.height, this._cursorFloat)), n4.count = r7;
      }
      _updateRectangle(t3, n4, s18, o6, r7, a3, l4) {
        if (s18 & 67108864) switch (s18 & 50331648) {
          case 16777216:
          case 33554432:
            xe3 = this._themeService.colors.ansi[s18 & 255].rgba;
            break;
          case 50331648:
            xe3 = (s18 & 16777215) << 8;
            break;
          case 0:
          default:
            xe3 = this._themeService.colors.foreground.rgba;
        }
        else switch (o6 & 50331648) {
          case 16777216:
          case 33554432:
            xe3 = this._themeService.colors.ansi[o6 & 255].rgba;
            break;
          case 50331648:
            xe3 = (o6 & 16777215) << 8;
            break;
          case 0:
          default:
            xe3 = this._themeService.colors.background.rgba;
        }
        t3.attributes.length < n4 + 4 && (t3.attributes = Bn(t3.attributes, this._terminal.rows * this._terminal.cols * Ee3)), Hn2 = r7 * this._dimensions.device.cell.width, Wn2 = l4 * this._dimensions.device.cell.height, Gn2 = (xe3 >> 24 & 255) / 255, $n2 = (xe3 >> 16 & 255) / 255, Kn2 = (xe3 >> 8 & 255) / 255, Vn2 = 1, this._addRectangle(t3.attributes, n4, Hn2, Wn2, (a3 - r7) * this._dimensions.device.cell.width, this._dimensions.device.cell.height, Gn2, $n2, Kn2, Vn2);
      }
      _addRectangle(t3, n4, s18, o6, r7, a3, l4, u3, c4, d3) {
        t3[n4] = s18 / this._dimensions.device.canvas.width, t3[n4 + 1] = o6 / this._dimensions.device.canvas.height, t3[n4 + 2] = r7 / this._dimensions.device.canvas.width, t3[n4 + 3] = a3 / this._dimensions.device.canvas.height, t3[n4 + 4] = l4, t3[n4 + 5] = u3, t3[n4 + 6] = c4, t3[n4 + 7] = d3;
      }
      _addRectangleFloat(t3, n4, s18, o6, r7, a3, l4) {
        t3[n4] = s18 / this._dimensions.device.canvas.width, t3[n4 + 1] = o6 / this._dimensions.device.canvas.height, t3[n4 + 2] = r7 / this._dimensions.device.canvas.width, t3[n4 + 3] = a3 / this._dimensions.device.canvas.height, t3[n4 + 4] = l4[0], t3[n4 + 5] = l4[1], t3[n4 + 6] = l4[2], t3[n4 + 7] = l4[3];
      }
      _colorToFloat32Array(t3) {
        return new Float32Array([(t3.rgba >> 24 & 255) / 255, (t3.rgba >> 16 & 255) / 255, (t3.rgba >> 8 & 255) / 255, (t3.rgba & 255) / 255]);
      }
    };
    jt3 = class extends B5 {
      constructor(t3, n4, s18, o6, r7, a3, l4, u3) {
        super();
        this._container = n4;
        this._alpha = r7;
        this._coreBrowserService = a3;
        this._optionsService = l4;
        this._themeService = u3;
        this._deviceCharWidth = 0;
        this._deviceCharHeight = 0;
        this._deviceCellWidth = 0;
        this._deviceCellHeight = 0;
        this._deviceCharLeft = 0;
        this._deviceCharTop = 0;
        this._canvas = this._coreBrowserService.mainDocument.createElement("canvas"), this._canvas.classList.add(`xterm-${s18}-layer`), this._canvas.style.zIndex = o6.toString(), this._initCanvas(), this._container.appendChild(this._canvas), this._register(this._themeService.onChangeColors((c4) => {
          this._refreshCharAtlas(t3, c4), this.reset(t3);
        })), this._register(O2(() => {
          this._canvas.remove();
        }));
      }
      _initCanvas() {
        this._ctx = F3(this._canvas.getContext("2d", { alpha: this._alpha })), this._alpha || this._clearAll();
      }
      handleBlur(t3) {
      }
      handleFocus(t3) {
      }
      handleCursorMove(t3) {
      }
      handleGridChanged(t3, n4, s18) {
      }
      handleSelectionChanged(t3, n4, s18, o6 = false) {
      }
      _setTransparency(t3, n4) {
        if (n4 === this._alpha) return;
        let s18 = this._canvas;
        this._alpha = n4, this._canvas = this._canvas.cloneNode(), this._initCanvas(), this._container.replaceChild(this._canvas, s18), this._refreshCharAtlas(t3, this._themeService.colors), this.handleGridChanged(t3, 0, t3.rows - 1);
      }
      _refreshCharAtlas(t3, n4) {
        this._deviceCharWidth <= 0 && this._deviceCharHeight <= 0 || (this._charAtlas = Nt3(t3, this._optionsService.rawOptions, n4, this._deviceCellWidth, this._deviceCellHeight, this._deviceCharWidth, this._deviceCharHeight, this._coreBrowserService.dpr, 2048), this._charAtlas.warmUp());
      }
      resize(t3, n4) {
        this._deviceCellWidth = n4.device.cell.width, this._deviceCellHeight = n4.device.cell.height, this._deviceCharWidth = n4.device.char.width, this._deviceCharHeight = n4.device.char.height, this._deviceCharLeft = n4.device.char.left, this._deviceCharTop = n4.device.char.top, this._canvas.width = n4.device.canvas.width, this._canvas.height = n4.device.canvas.height, this._canvas.style.width = `${n4.css.canvas.width}px`, this._canvas.style.height = `${n4.css.canvas.height}px`, this._alpha || this._clearAll(), this._refreshCharAtlas(t3, this._themeService.colors);
      }
      _fillBottomLineAtCells(t3, n4, s18 = 1) {
        this._ctx.fillRect(t3 * this._deviceCellWidth, (n4 + 1) * this._deviceCellHeight - this._coreBrowserService.dpr - 1, s18 * this._deviceCellWidth, this._coreBrowserService.dpr);
      }
      _clearAll() {
        this._alpha ? this._ctx.clearRect(0, 0, this._canvas.width, this._canvas.height) : (this._ctx.fillStyle = this._themeService.colors.background.css, this._ctx.fillRect(0, 0, this._canvas.width, this._canvas.height));
      }
      _clearCells(t3, n4, s18, o6) {
        this._alpha ? this._ctx.clearRect(t3 * this._deviceCellWidth, n4 * this._deviceCellHeight, s18 * this._deviceCellWidth, o6 * this._deviceCellHeight) : (this._ctx.fillStyle = this._themeService.colors.background.css, this._ctx.fillRect(t3 * this._deviceCellWidth, n4 * this._deviceCellHeight, s18 * this._deviceCellWidth, o6 * this._deviceCellHeight));
      }
      _fillCharTrueColor(t3, n4, s18, o6) {
        this._ctx.font = this._getFont(t3, false, false), this._ctx.textBaseline = St3, this._clipCell(s18, o6, n4.getWidth()), this._ctx.fillText(n4.getChars(), s18 * this._deviceCellWidth + this._deviceCharLeft, o6 * this._deviceCellHeight + this._deviceCharTop + this._deviceCharHeight);
      }
      _clipCell(t3, n4, s18) {
        this._ctx.beginPath(), this._ctx.rect(t3 * this._deviceCellWidth, n4 * this._deviceCellHeight, s18 * this._deviceCellWidth, this._deviceCellHeight), this._ctx.clip();
      }
      _getFont(t3, n4, s18) {
        let o6 = n4 ? t3.options.fontWeightBold : t3.options.fontWeight;
        return `${s18 ? "italic" : ""} ${o6} ${t3.options.fontSize * this._coreBrowserService.dpr}px ${t3.options.fontFamily}`;
      }
    };
    Xt2 = class extends jt3 {
      constructor(e4, t3, n4, s18, o6, r7, a3) {
        super(n4, e4, "link", t3, true, o6, r7, a3), this._register(s18.onShowLinkUnderline((l4) => this._handleShowLinkUnderline(l4))), this._register(s18.onHideLinkUnderline((l4) => this._handleHideLinkUnderline(l4)));
      }
      resize(e4, t3) {
        super.resize(e4, t3), this._state = void 0;
      }
      reset(e4) {
        this._clearCurrentLink();
      }
      _clearCurrentLink() {
        if (this._state) {
          this._clearCells(this._state.x1, this._state.y1, this._state.cols - this._state.x1, 1);
          let e4 = this._state.y2 - this._state.y1 - 1;
          e4 > 0 && this._clearCells(0, this._state.y1 + 1, this._state.cols, e4), this._clearCells(0, this._state.y2, this._state.x2, 1), this._state = void 0;
        }
      }
      _handleShowLinkUnderline(e4) {
        if (e4.fg === 257 ? this._ctx.fillStyle = this._themeService.colors.background.css : e4.fg !== void 0 && Fn2(e4.fg) ? this._ctx.fillStyle = this._themeService.colors.ansi[e4.fg].css : this._ctx.fillStyle = this._themeService.colors.foreground.css, e4.y1 === e4.y2) this._fillBottomLineAtCells(e4.x1, e4.y1, e4.x2 - e4.x1);
        else {
          this._fillBottomLineAtCells(e4.x1, e4.y1, e4.cols - e4.x1);
          for (let t3 = e4.y1 + 1; t3 < e4.y2; t3++) this._fillBottomLineAtCells(0, t3, e4.cols);
          this._fillBottomLineAtCells(0, e4.y2, e4.x2);
        }
        this._state = e4;
      }
      _handleHideLinkUnderline(e4) {
        this._clearCurrentLink();
      }
    };
    te2 = typeof window == "object" ? window : globalThis;
    Zt2 = class Zt3 {
      constructor() {
        this.mapWindowIdToZoomLevel = /* @__PURE__ */ new Map();
        this._onDidChangeZoomLevel = new D3();
        this.onDidChangeZoomLevel = this._onDidChangeZoomLevel.event;
        this.mapWindowIdToZoomFactor = /* @__PURE__ */ new Map();
        this._onDidChangeFullscreen = new D3();
        this.onDidChangeFullscreen = this._onDidChangeFullscreen.event;
        this.mapWindowIdToFullScreen = /* @__PURE__ */ new Map();
      }
      getZoomLevel(e4) {
        return this.mapWindowIdToZoomLevel.get(this.getWindowId(e4)) ?? 0;
      }
      setZoomLevel(e4, t3) {
        if (this.getZoomLevel(t3) === e4) return;
        let n4 = this.getWindowId(t3);
        this.mapWindowIdToZoomLevel.set(n4, e4), this._onDidChangeZoomLevel.fire(n4);
      }
      getZoomFactor(e4) {
        return this.mapWindowIdToZoomFactor.get(this.getWindowId(e4)) ?? 1;
      }
      setZoomFactor(e4, t3) {
        this.mapWindowIdToZoomFactor.set(this.getWindowId(t3), e4);
      }
      setFullscreen(e4, t3) {
        if (this.isFullscreen(t3) === e4) return;
        let n4 = this.getWindowId(t3);
        this.mapWindowIdToFullScreen.set(n4, e4), this._onDidChangeFullscreen.fire(n4);
      }
      isFullscreen(e4) {
        return !!this.mapWindowIdToFullScreen.get(this.getWindowId(e4));
      }
      getWindowId(e4) {
        return e4.vscodeWindowId;
      }
    };
    Zt2.INSTANCE = new Zt2();
    Qt2 = Zt2;
    Wa2 = Qt2.INSTANCE.onDidChangeZoomLevel;
    Ga2 = Qt2.INSTANCE.onDidChangeFullscreen;
    je3 = typeof navigator == "object" ? navigator.userAgent : "";
    Cn2 = je3.indexOf("Firefox") >= 0;
    ut3 = je3.indexOf("AppleWebKit") >= 0;
    zn2 = je3.indexOf("Chrome") >= 0;
    Bi2 = !zn2 && je3.indexOf("Safari") >= 0;
    $a2 = je3.indexOf("Electron/") >= 0;
    Ka2 = je3.indexOf("Android") >= 0;
    Yt2 = false;
    if (typeof te2.matchMedia == "function") {
      let i12 = te2.matchMedia("(display-mode: standalone) or (display-mode: window-controls-overlay)"), e4 = te2.matchMedia("(display-mode: fullscreen)");
      Yt2 = i12.matches, us2(te2, i12, ({ matches: t3 }) => {
        Yt2 && e4.matches || (Yt2 = t3);
      });
    }
    Xe3 = "en";
    Ui2 = false;
    ni2 = false;
    ti2 = false;
    cs2 = false;
    Xn2 = false;
    Yn2 = false;
    ds2 = false;
    hs2 = false;
    ps2 = false;
    fs2 = false;
    ii2 = Xe3;
    jn2 = Xe3;
    Ie3 = globalThis;
    typeof Ie3.vscode < "u" && typeof Ie3.vscode.process < "u" ? re2 = Ie3.vscode.process : typeof process < "u" && typeof process?.versions?.node == "string" && (re2 = process);
    Qn2 = typeof re2?.versions?.electron == "string";
    _s2 = Qn2 && re2?.type === "renderer";
    if (typeof re2 == "object") {
      Ui2 = re2.platform === "win32", ni2 = re2.platform === "darwin", ti2 = re2.platform === "linux", cs2 = ti2 && !!re2.env.SNAP && !!re2.env.SNAP_REVISION, ds2 = Qn2, ps2 = !!re2.env.CI || !!re2.env.BUILD_ARTIFACTSTAGINGDIRECTORY, ei2 = Xe3, ii2 = Xe3;
      let i12 = re2.env.VSCODE_NLS_CONFIG;
      if (i12) try {
        let e4 = JSON.parse(i12);
        ei2 = e4.userLocale, jn2 = e4.osLocale, ii2 = e4.resolvedLanguage || Xe3, ms2 = e4.languagePack?.translationsConfigFile;
      } catch {
      }
      Xn2 = true;
    } else typeof navigator == "object" && !_s2 ? (ye3 = navigator.userAgent, Ui2 = ye3.indexOf("Windows") >= 0, ni2 = ye3.indexOf("Macintosh") >= 0, hs2 = (ye3.indexOf("Macintosh") >= 0 || ye3.indexOf("iPad") >= 0 || ye3.indexOf("iPhone") >= 0) && !!navigator.maxTouchPoints && navigator.maxTouchPoints > 0, ti2 = ye3.indexOf("Linux") >= 0, fs2 = ye3?.indexOf("Mobi") >= 0, Yn2 = true, ii2 = globalThis._VSCODE_NLS_LANGUAGE || Xe3, ei2 = navigator.language.toLowerCase(), jn2 = ei2) : console.error("Unable to resolve platform.");
    Ni2 = 0;
    ni2 ? Ni2 = 1 : Ui2 ? Ni2 = 3 : ti2 && (Ni2 = 2);
    ri2 = Xn2;
    bs2 = Yn2 && typeof Ie3.importScripts == "function";
    Va2 = bs2 ? Ie3.origin : void 0;
    _e4 = ye3;
    Me2 = ii2;
    ((n4) => {
      function i12() {
        return Me2;
      }
      n4.value = i12;
      function e4() {
        return Me2.length === 2 ? Me2 === "en" : Me2.length >= 3 ? Me2[0] === "e" && Me2[1] === "n" && Me2[2] === "-" : false;
      }
      n4.isDefaultVariant = e4;
      function t3() {
        return Me2 === "en";
      }
      n4.isDefault = t3;
    })(vs2 || (vs2 = {}));
    Ts2 = typeof Ie3.postMessage == "function" && !Ie3.importScripts;
    Zn2 = (() => {
      if (Ts2) {
        let i12 = [];
        Ie3.addEventListener("message", (t3) => {
          if (t3.data && t3.data.vscodeScheduleAsyncWork) for (let n4 = 0, s18 = i12.length; n4 < s18; n4++) {
            let o6 = i12[n4];
            if (o6.id === t3.data.vscodeScheduleAsyncWork) {
              i12.splice(n4, 1), o6.callback();
              return;
            }
          }
        });
        let e4 = 0;
        return (t3) => {
          let n4 = ++e4;
          i12.push({ id: n4, callback: t3 }), Ie3.postMessage({ vscodeScheduleAsyncWork: n4 }, "*");
        };
      }
      return (i12) => setTimeout(i12);
    })();
    gs2 = !!(_e4 && _e4.indexOf("Chrome") >= 0);
    Ca2 = !!(_e4 && _e4.indexOf("Firefox") >= 0);
    za2 = !!(!gs2 && _e4 && _e4.indexOf("Safari") >= 0);
    qa2 = !!(_e4 && _e4.indexOf("Edg/") >= 0);
    ja2 = !!(_e4 && _e4.indexOf("Android") >= 0);
    Ae3 = typeof navigator == "object" ? navigator : {};
    xs2 = { clipboard: { writeText: ri2 || document.queryCommandSupported && document.queryCommandSupported("copy") || !!(Ae3 && Ae3.clipboard && Ae3.clipboard.writeText), readText: ri2 || !!(Ae3 && Ae3.clipboard && Ae3.clipboard.readText) }, keyboard: ri2 || qn2() ? 0 : Ae3.keyboard || Bi2 ? 1 : 2, touch: "ontouchstart" in te2 || Ae3.maxTouchPoints > 0, pointerEvents: te2.PointerEvent && ("ontouchstart" in te2 || navigator.maxTouchPoints > 0) };
    dt2 = class {
      constructor() {
        this._keyCodeToStr = [], this._strToKeyCode = /* @__PURE__ */ Object.create(null);
      }
      define(e4, t3) {
        this._keyCodeToStr[e4] = t3, this._strToKeyCode[t3.toLowerCase()] = e4;
      }
      keyCodeToStr(e4) {
        return this._keyCodeToStr[e4];
      }
      strToKeyCode(e4) {
        return this._strToKeyCode[e4.toLowerCase()] || 0;
      }
    };
    Hi2 = new dt2();
    Jn2 = new dt2();
    er2 = new dt2();
    Es2 = new Array(230);
    ((r7) => {
      function i12(a3) {
        return Hi2.keyCodeToStr(a3);
      }
      r7.toString = i12;
      function e4(a3) {
        return Hi2.strToKeyCode(a3);
      }
      r7.fromString = e4;
      function t3(a3) {
        return Jn2.keyCodeToStr(a3);
      }
      r7.toUserSettingsUS = t3;
      function n4(a3) {
        return er2.keyCodeToStr(a3);
      }
      r7.toUserSettingsGeneral = n4;
      function s18(a3) {
        return Jn2.strToKeyCode(a3) || er2.strToKeyCode(a3);
      }
      r7.fromUserSettings = s18;
      function o6(a3) {
        if (a3 >= 98 && a3 <= 113) return null;
        switch (a3) {
          case 16:
            return "Up";
          case 18:
            return "Down";
          case 15:
            return "Left";
          case 17:
            return "Right";
        }
        return Hi2.keyCodeToStr(a3);
      }
      r7.toElectronAccelerator = o6;
    })(tr2 || (tr2 = {}));
    nr2 = Object.freeze(function(i12, e4) {
      let t3 = setTimeout(i12.bind(e4), 0);
      return { dispose() {
        clearTimeout(t3);
      } };
    });
    ((n4) => {
      function i12(s18) {
        return s18 === n4.None || s18 === n4.Cancelled || s18 instanceof Wi2 ? true : !s18 || typeof s18 != "object" ? false : typeof s18.isCancellationRequested == "boolean" && typeof s18.onCancellationRequested == "function";
      }
      n4.isCancellationToken = i12, n4.None = Object.freeze({ isCancellationRequested: false, onCancellationRequested: ee3.None }), n4.Cancelled = Object.freeze({ isCancellationRequested: true, onCancellationRequested: nr2 });
    })(Is2 || (Is2 = {}));
    Wi2 = class {
      constructor() {
        this._isCancelled = false;
        this._emitter = null;
      }
      cancel() {
        this._isCancelled || (this._isCancelled = true, this._emitter && (this._emitter.fire(void 0), this.dispose()));
      }
      get isCancellationRequested() {
        return this._isCancelled;
      }
      get onCancellationRequested() {
        return this._isCancelled ? nr2 : (this._emitter || (this._emitter = new D3()), this._emitter.event);
      }
      dispose() {
        this._emitter && (this._emitter.dispose(), this._emitter = null);
      }
    };
    Ls2 = Symbol("MicrotaskDelay");
    (function() {
      typeof globalThis.requestIdleCallback != "function" || typeof globalThis.cancelIdleCallback != "function" ? oi2 = (i12, e4) => {
        Zn2(() => {
          if (t3) return;
          let n4 = Date.now() + 15;
          e4(Object.freeze({ didTimeout: true, timeRemaining() {
            return Math.max(0, n4 - Date.now());
          } }));
        });
        let t3 = false;
        return { dispose() {
          t3 || (t3 = true);
        } };
      } : oi2 = (i12, e4, t3) => {
        let n4 = i12.requestIdleCallback(e4, typeof t3 == "number" ? { timeout: t3 } : void 0), s18 = false;
        return { dispose() {
          s18 || (s18 = true, i12.cancelIdleCallback(n4));
        } };
      }, ws2 = (i12) => oi2(globalThis, i12);
    })();
    ((t3) => {
      async function i12(n4) {
        let s18, o6 = await Promise.all(n4.map((r7) => r7.then((a3) => a3, (a3) => {
          s18 || (s18 = a3);
        })));
        if (typeof s18 < "u") throw s18;
        return o6;
      }
      t3.settled = i12;
      function e4(n4) {
        return new Promise(async (s18, o6) => {
          try {
            await n4(s18, o6);
          } catch (r7) {
            o6(r7);
          }
        });
      }
      t3.withAsyncBody = e4;
    })(Rs2 || (Rs2 = {}));
    Q4 = class Q5 {
      static fromArray(e4) {
        return new Q5((t3) => {
          t3.emitMany(e4);
        });
      }
      static fromPromise(e4) {
        return new Q5(async (t3) => {
          t3.emitMany(await e4);
        });
      }
      static fromPromises(e4) {
        return new Q5(async (t3) => {
          await Promise.all(e4.map(async (n4) => t3.emitOne(await n4)));
        });
      }
      static merge(e4) {
        return new Q5(async (t3) => {
          await Promise.all(e4.map(async (n4) => {
            for await (let s18 of n4) t3.emitOne(s18);
          }));
        });
      }
      constructor(e4, t3) {
        this._state = 0, this._results = [], this._error = null, this._onReturn = t3, this._onStateChanged = new D3(), queueMicrotask(async () => {
          let n4 = { emitOne: (s18) => this.emitOne(s18), emitMany: (s18) => this.emitMany(s18), reject: (s18) => this.reject(s18) };
          try {
            await Promise.resolve(e4(n4)), this.resolve();
          } catch (s18) {
            this.reject(s18);
          } finally {
            n4.emitOne = void 0, n4.emitMany = void 0, n4.reject = void 0;
          }
        });
      }
      [Symbol.asyncIterator]() {
        let e4 = 0;
        return { next: async () => {
          do {
            if (this._state === 2) throw this._error;
            if (e4 < this._results.length) return { done: false, value: this._results[e4++] };
            if (this._state === 1) return { done: true, value: void 0 };
            await ee3.toPromise(this._onStateChanged.event);
          } while (true);
        }, return: async () => (this._onReturn?.(), { done: true, value: void 0 }) };
      }
      static map(e4, t3) {
        return new Q5(async (n4) => {
          for await (let s18 of e4) n4.emitOne(t3(s18));
        });
      }
      map(e4) {
        return Q5.map(this, e4);
      }
      static filter(e4, t3) {
        return new Q5(async (n4) => {
          for await (let s18 of e4) t3(s18) && n4.emitOne(s18);
        });
      }
      filter(e4) {
        return Q5.filter(this, e4);
      }
      static coalesce(e4) {
        return Q5.filter(e4, (t3) => !!t3);
      }
      coalesce() {
        return Q5.coalesce(this);
      }
      static async toPromise(e4) {
        let t3 = [];
        for await (let n4 of e4) t3.push(n4);
        return t3;
      }
      toPromise() {
        return Q5.toPromise(this);
      }
      emitOne(e4) {
        this._state === 0 && (this._results.push(e4), this._onStateChanged.fire());
      }
      emitMany(e4) {
        this._state === 0 && (this._results = this._results.concat(e4), this._onStateChanged.fire());
      }
      resolve() {
        this._state === 0 && (this._state = 1, this._onStateChanged.fire());
      }
      reject(e4) {
        this._state === 0 && (this._state = 2, this._error = e4, this._onStateChanged.fire());
      }
    };
    Q4.EMPTY = Q4.fromArray([]);
    ai2 = class ai3 {
      constructor() {
        this._h0 = 1732584193;
        this._h1 = 4023233417;
        this._h2 = 2562383102;
        this._h3 = 271733878;
        this._h4 = 3285377520;
        this._buff = new Uint8Array(67), this._buffDV = new DataView(this._buff.buffer), this._buffLen = 0, this._totalLen = 0, this._leftoverHighSurrogate = 0, this._finished = false;
      }
      update(e4) {
        let t3 = e4.length;
        if (t3 === 0) return;
        let n4 = this._buff, s18 = this._buffLen, o6 = this._leftoverHighSurrogate, r7, a3;
        for (o6 !== 0 ? (r7 = o6, a3 = -1, o6 = 0) : (r7 = e4.charCodeAt(0), a3 = 0); ; ) {
          let l4 = r7;
          if (sr2(r7)) if (a3 + 1 < t3) {
            let u3 = e4.charCodeAt(a3 + 1);
            Gi(u3) ? (a3++, l4 = or2(r7, u3)) : l4 = 65533;
          } else {
            o6 = r7;
            break;
          }
          else Gi(r7) && (l4 = 65533);
          if (s18 = this._push(n4, s18, l4), a3++, a3 < t3) r7 = e4.charCodeAt(a3);
          else break;
        }
        this._buffLen = s18, this._leftoverHighSurrogate = o6;
      }
      _push(e4, t3, n4) {
        return n4 < 128 ? e4[t3++] = n4 : n4 < 2048 ? (e4[t3++] = 192 | (n4 & 1984) >>> 6, e4[t3++] = 128 | (n4 & 63) >>> 0) : n4 < 65536 ? (e4[t3++] = 224 | (n4 & 61440) >>> 12, e4[t3++] = 128 | (n4 & 4032) >>> 6, e4[t3++] = 128 | (n4 & 63) >>> 0) : (e4[t3++] = 240 | (n4 & 1835008) >>> 18, e4[t3++] = 128 | (n4 & 258048) >>> 12, e4[t3++] = 128 | (n4 & 4032) >>> 6, e4[t3++] = 128 | (n4 & 63) >>> 0), t3 >= 64 && (this._step(), t3 -= 64, this._totalLen += 64, e4[0] = e4[64], e4[1] = e4[65], e4[2] = e4[66]), t3;
      }
      digest() {
        return this._finished || (this._finished = true, this._leftoverHighSurrogate && (this._leftoverHighSurrogate = 0, this._buffLen = this._push(this._buff, this._buffLen, 65533)), this._totalLen += this._buffLen, this._wrapUp()), ht(this._h0) + ht(this._h1) + ht(this._h2) + ht(this._h3) + ht(this._h4);
      }
      _wrapUp() {
        this._buff[this._buffLen++] = 128, ar2(this._buff, this._buffLen), this._buffLen > 56 && (this._step(), ar2(this._buff));
        let e4 = 8 * this._totalLen;
        this._buffDV.setUint32(56, Math.floor(e4 / 4294967296), false), this._buffDV.setUint32(60, e4 % 4294967296, false), this._step();
      }
      _step() {
        let e4 = ai3._bigBlock32, t3 = this._buffDV;
        for (let d3 = 0; d3 < 64; d3 += 4) e4.setUint32(d3, t3.getUint32(d3, false), false);
        for (let d3 = 64; d3 < 320; d3 += 4) e4.setUint32(d3, $i2(e4.getUint32(d3 - 12, false) ^ e4.getUint32(d3 - 32, false) ^ e4.getUint32(d3 - 56, false) ^ e4.getUint32(d3 - 64, false), 1), false);
        let n4 = this._h0, s18 = this._h1, o6 = this._h2, r7 = this._h3, a3 = this._h4, l4, u3, c4;
        for (let d3 = 0; d3 < 80; d3++) d3 < 20 ? (l4 = s18 & o6 | ~s18 & r7, u3 = 1518500249) : d3 < 40 ? (l4 = s18 ^ o6 ^ r7, u3 = 1859775393) : d3 < 60 ? (l4 = s18 & o6 | s18 & r7 | o6 & r7, u3 = 2400959708) : (l4 = s18 ^ o6 ^ r7, u3 = 3395469782), c4 = $i2(n4, 5) + l4 + a3 + u3 + e4.getUint32(d3 * 4, false) & 4294967295, a3 = r7, r7 = o6, o6 = $i2(s18, 30), s18 = n4, n4 = c4;
        this._h0 = this._h0 + n4 & 4294967295, this._h1 = this._h1 + s18 & 4294967295, this._h2 = this._h2 + o6 & 4294967295, this._h3 = this._h3 + r7 & 4294967295, this._h4 = this._h4 + a3 & 4294967295;
      }
    };
    ai2._bigBlock32 = new DataView(new ArrayBuffer(320));
    ({ registerWindow: fu, getWindow: Fs2, getDocument: mu, getWindows: _u, getWindowsCount: bu, getWindowId: dr3, getWindowById: vu, hasWindow: Tu2, onDidRegisterWindow: gu, onWillUnregisterWindow: xu2, onDidUnregisterWindow: Eu2 } = (function() {
      let i12 = /* @__PURE__ */ new Map();
      te2;
      let e4 = { window: te2, disposables: new fe3() };
      i12.set(te2.vscodeWindowId, e4);
      let t3 = new D3(), n4 = new D3(), s18 = new D3();
      function o6(r7, a3) {
        return (typeof r7 == "number" ? i12.get(r7) : void 0) ?? (a3 ? e4 : void 0);
      }
      return { onDidRegisterWindow: t3.event, onWillUnregisterWindow: s18.event, onDidUnregisterWindow: n4.event, registerWindow(r7) {
        if (i12.has(r7.vscodeWindowId)) return B5.None;
        let a3 = new fe3(), l4 = { window: r7, disposables: a3.add(new fe3()) };
        return i12.set(r7.vscodeWindowId, l4), a3.add(O2(() => {
          i12.delete(r7.vscodeWindowId), n4.fire(r7);
        })), a3.add(li2(r7, Ps2.BEFORE_UNLOAD, () => {
          s18.fire(r7);
        })), t3.fire(l4), a3;
      }, getWindows() {
        return i12.values();
      }, getWindowsCount() {
        return i12.size;
      }, getWindowId(r7) {
        return r7.vscodeWindowId;
      }, hasWindow(r7) {
        return i12.has(r7);
      }, getWindowById: o6, getWindow(r7) {
        let a3 = r7;
        if (a3?.ownerDocument?.defaultView) return a3.ownerDocument.defaultView.window;
        let l4 = r7;
        return l4?.view ? l4.view.window : te2;
      }, getDocument(r7) {
        return Fs2(r7).document;
      } };
    })());
    Vi2 = class {
      constructor(e4, t3, n4, s18) {
        this._node = e4, this._type = t3, this._handler = n4, this._options = s18 || false, this._node.addEventListener(this._type, this._handler, this._options);
      }
      dispose() {
        this._handler && (this._node.removeEventListener(this._type, this._handler, this._options), this._node = null, this._handler = null);
      }
    };
    pt4 = class {
      constructor(e4, t3 = 0) {
        this._runner = e4, this.priority = t3, this._canceled = false;
      }
      dispose() {
        this._canceled = true;
      }
      execute() {
        if (!this._canceled) try {
          this._runner();
        } catch (e4) {
          Pe2(e4);
        }
      }
      static sort(e4, t3) {
        return t3.priority - e4.priority;
      }
    };
    (function() {
      let i12 = /* @__PURE__ */ new Map(), e4 = /* @__PURE__ */ new Map(), t3 = /* @__PURE__ */ new Map(), n4 = /* @__PURE__ */ new Map(), s18 = (o6) => {
        t3.set(o6, false);
        let r7 = i12.get(o6) ?? [];
        for (e4.set(o6, r7), i12.set(o6, []), n4.set(o6, true); r7.length > 0; ) r7.sort(pt4.sort), r7.shift().execute();
        n4.set(o6, false);
      };
      hr3 = (o6, r7, a3 = 0) => {
        let l4 = dr3(o6), u3 = new pt4(r7, a3), c4 = i12.get(l4);
        return c4 || (c4 = [], i12.set(l4, c4)), c4.push(u3), t3.get(l4) || (t3.set(l4, true), o6.requestAnimationFrame(() => s18(l4))), u3;
      }, ks2 = (o6, r7, a3) => {
        let l4 = dr3(o6);
        if (n4.get(l4)) {
          let u3 = new pt4(r7, a3), c4 = e4.get(l4);
          return c4 || (c4 = [], e4.set(l4, c4)), c4.push(u3), u3;
        } else return hr3(o6, r7, a3);
      };
    })();
    ke3 = class ke4 {
      constructor(e4, t3) {
        this.width = e4;
        this.height = t3;
      }
      with(e4 = this.width, t3 = this.height) {
        return e4 !== this.width || t3 !== this.height ? new ke4(e4, t3) : this;
      }
      static is(e4) {
        return typeof e4 == "object" && typeof e4.height == "number" && typeof e4.width == "number";
      }
      static lift(e4) {
        return e4 instanceof ke4 ? e4 : new ke4(e4.width, e4.height);
      }
      static equals(e4, t3) {
        return e4 === t3 ? true : !e4 || !t3 ? false : e4.width === t3.width && e4.height === t3.height;
      }
    };
    ke3.None = new ke3(0, 0);
    yu2 = new class {
      constructor() {
        this.mutationObservers = /* @__PURE__ */ new Map();
      }
      observe(i12, e4, t3) {
        let n4 = this.mutationObservers.get(i12);
        n4 || (n4 = /* @__PURE__ */ new Map(), this.mutationObservers.set(i12, n4));
        let s18 = ur2(t3), o6 = n4.get(s18);
        if (o6) o6.users += 1;
        else {
          let r7 = new D3(), a3 = new MutationObserver((u3) => r7.fire(u3));
          a3.observe(i12, t3);
          let l4 = o6 = { users: 1, observer: a3, onDidMutate: r7.event };
          e4.add(O2(() => {
            l4.users -= 1, l4.users === 0 && (r7.dispose(), a3.disconnect(), n4?.delete(s18), n4?.size === 0 && this.mutationObservers.delete(i12));
          })), n4.set(s18, o6);
        }
        return o6.onDidMutate;
      }
    }();
    Ps2 = { CLICK: "click", AUXCLICK: "auxclick", DBLCLICK: "dblclick", MOUSE_UP: "mouseup", MOUSE_DOWN: "mousedown", MOUSE_OVER: "mouseover", MOUSE_MOVE: "mousemove", MOUSE_OUT: "mouseout", MOUSE_ENTER: "mouseenter", MOUSE_LEAVE: "mouseleave", MOUSE_WHEEL: "wheel", POINTER_UP: "pointerup", POINTER_DOWN: "pointerdown", POINTER_MOVE: "pointermove", POINTER_LEAVE: "pointerleave", CONTEXT_MENU: "contextmenu", WHEEL: "wheel", KEY_DOWN: "keydown", KEY_PRESS: "keypress", KEY_UP: "keyup", LOAD: "load", BEFORE_UNLOAD: "beforeunload", UNLOAD: "unload", PAGE_SHOW: "pageshow", PAGE_HIDE: "pagehide", PASTE: "paste", ABORT: "abort", ERROR: "error", RESIZE: "resize", SCROLL: "scroll", FULLSCREEN_CHANGE: "fullscreenchange", WK_FULLSCREEN_CHANGE: "webkitfullscreenchange", SELECT: "select", CHANGE: "change", SUBMIT: "submit", RESET: "reset", FOCUS: "focus", FOCUS_IN: "focusin", FOCUS_OUT: "focusout", BLUR: "blur", INPUT: "input", STORAGE: "storage", DRAG_START: "dragstart", DRAG: "drag", DRAG_ENTER: "dragenter", DRAG_LEAVE: "dragleave", DRAG_OVER: "dragover", DROP: "drop", DRAG_END: "dragend", ANIMATION_START: ut3 ? "webkitAnimationStart" : "animationstart", ANIMATION_END: ut3 ? "webkitAnimationEnd" : "animationend", ANIMATION_ITERATION: ut3 ? "webkitAnimationIteration" : "animationiteration" };
    Bs2 = /([\w\-]+)?(#([\w\-]+))?((\.([\w\-]+))*)/;
    Ns.SVG = function(i12, e4, ...t3) {
      return fr2("http://www.w3.org/2000/svg", i12, e4, ...t3);
    };
    ui2 = class extends B5 {
      constructor(t3, n4, s18, o6, r7, a3, l4, u3, c4) {
        super();
        this._terminal = t3;
        this._characterJoinerService = n4;
        this._charSizeService = s18;
        this._coreBrowserService = o6;
        this._coreService = r7;
        this._decorationService = a3;
        this._optionsService = l4;
        this._themeService = u3;
        this._cursorBlinkStateManager = new be4();
        this._charAtlasDisposable = this._register(new be4());
        this._observerDisposable = this._register(new be4());
        this._model = new Vt2();
        this._workCell = new at3();
        this._workCell2 = new at3();
        this._rectangleRenderer = this._register(new be4());
        this._glyphRenderer = this._register(new be4());
        this._onChangeTextureAtlas = this._register(new D3());
        this.onChangeTextureAtlas = this._onChangeTextureAtlas.event;
        this._onAddTextureAtlasCanvas = this._register(new D3());
        this.onAddTextureAtlasCanvas = this._onAddTextureAtlasCanvas.event;
        this._onRemoveTextureAtlasCanvas = this._register(new D3());
        this.onRemoveTextureAtlasCanvas = this._onRemoveTextureAtlasCanvas.event;
        this._onRequestRedraw = this._register(new D3());
        this.onRequestRedraw = this._onRequestRedraw.event;
        this._onContextLoss = this._register(new D3());
        this.onContextLoss = this._onContextLoss.event;
        this._canvas = this._coreBrowserService.mainDocument.createElement("canvas");
        let d3 = { antialias: false, depth: false, preserveDrawingBuffer: c4 };
        if (this._gl = this._canvas.getContext("webgl2", d3), !this._gl) throw new Error("WebGL2 not supported " + this._gl);
        this._register(this._themeService.onChangeColors(() => this._handleColorChange())), this._cellColorResolver = new At4(this._terminal, this._optionsService, this._model.selection, this._decorationService, this._coreBrowserService, this._themeService), this._core = this._terminal._core, this._renderLayers = [new Xt2(this._core.screenElement, 2, this._terminal, this._core.linkifier, this._coreBrowserService, l4, this._themeService)], this.dimensions = _n2(), this._devicePixelRatio = this._coreBrowserService.dpr, this._updateDimensions(), this._updateCursorBlink(), this._register(l4.onOptionChange(() => this._handleOptionsChanged())), this._deviceMaxTextureSize = this._gl.getParameter(this._gl.MAX_TEXTURE_SIZE), this._register(li2(this._canvas, "webglcontextlost", (h4) => {
          console.log("webglcontextlost event received"), h4.preventDefault(), this._contextRestorationTimeout = setTimeout(() => {
            this._contextRestorationTimeout = void 0, console.warn("webgl context not restored; firing onContextLoss"), this._onContextLoss.fire(h4);
          }, 3e3);
        })), this._register(li2(this._canvas, "webglcontextrestored", (h4) => {
          console.warn("webglcontextrestored event received"), clearTimeout(this._contextRestorationTimeout), this._contextRestorationTimeout = void 0, Ai2(this._terminal), this._initializeWebGLState(), this._requestRedrawViewport();
        })), this._observerDisposable.value = Si2(this._canvas, this._coreBrowserService.window, (h4, f3) => this._setCanvasDevicePixelDimensions(h4, f3)), this._register(this._coreBrowserService.onWindowChange((h4) => {
          this._observerDisposable.value = Si2(this._canvas, h4, (f3, I2) => this._setCanvasDevicePixelDimensions(f3, I2));
        })), this._core.screenElement.appendChild(this._canvas), [this._rectangleRenderer.value, this._glyphRenderer.value] = this._initializeWebGLState(), this._isAttached = this._core.screenElement.isConnected, this._register(O2(() => {
          for (let h4 of this._renderLayers) h4.dispose();
          this._canvas.parentElement?.removeChild(this._canvas), Ai2(this._terminal);
        }));
      }
      get textureAtlas() {
        return this._charAtlas?.pages[0].canvas;
      }
      _handleColorChange() {
        this._refreshCharAtlas(), this._clearModel(true);
      }
      handleDevicePixelRatioChange() {
        this._devicePixelRatio !== this._coreBrowserService.dpr && (this._devicePixelRatio = this._coreBrowserService.dpr, this.handleResize(this._terminal.cols, this._terminal.rows));
      }
      handleResize(t3, n4) {
        this._updateDimensions(), this._model.resize(this._terminal.cols, this._terminal.rows);
        for (let s18 of this._renderLayers) s18.resize(this._terminal, this.dimensions);
        this._canvas.width = this.dimensions.device.canvas.width, this._canvas.height = this.dimensions.device.canvas.height, this._canvas.style.width = `${this.dimensions.css.canvas.width}px`, this._canvas.style.height = `${this.dimensions.css.canvas.height}px`, this._core.screenElement.style.width = `${this.dimensions.css.canvas.width}px`, this._core.screenElement.style.height = `${this.dimensions.css.canvas.height}px`, this._rectangleRenderer.value?.setDimensions(this.dimensions), this._rectangleRenderer.value?.handleResize(), this._glyphRenderer.value?.setDimensions(this.dimensions), this._glyphRenderer.value?.handleResize(), this._refreshCharAtlas(), this._clearModel(false);
      }
      handleCharSizeChanged() {
        this.handleResize(this._terminal.cols, this._terminal.rows);
      }
      handleBlur() {
        for (let t3 of this._renderLayers) t3.handleBlur(this._terminal);
        this._cursorBlinkStateManager.value?.pause(), this._requestRedrawViewport();
      }
      handleFocus() {
        for (let t3 of this._renderLayers) t3.handleFocus(this._terminal);
        this._cursorBlinkStateManager.value?.resume(), this._requestRedrawViewport();
      }
      handleSelectionChanged(t3, n4, s18) {
        for (let o6 of this._renderLayers) o6.handleSelectionChanged(this._terminal, t3, n4, s18);
        this._model.selection.update(this._core, t3, n4, s18), this._requestRedrawViewport();
      }
      handleCursorMove() {
        for (let t3 of this._renderLayers) t3.handleCursorMove(this._terminal);
        this._cursorBlinkStateManager.value?.restartBlinkAnimation();
      }
      _handleOptionsChanged() {
        this._updateDimensions(), this._refreshCharAtlas(), this._updateCursorBlink();
      }
      _initializeWebGLState() {
        return this._rectangleRenderer.value = new qt2(this._terminal, this._gl, this.dimensions, this._themeService), this._glyphRenderer.value = new Kt2(this._terminal, this._gl, this.dimensions, this._optionsService), this.handleCharSizeChanged(), [this._rectangleRenderer.value, this._glyphRenderer.value];
      }
      _refreshCharAtlas() {
        if (this.dimensions.device.char.width <= 0 && this.dimensions.device.char.height <= 0) {
          this._isAttached = false;
          return;
        }
        let t3 = Nt3(this._terminal, this._optionsService.rawOptions, this._themeService.colors, this.dimensions.device.cell.width, this.dimensions.device.cell.height, this.dimensions.device.char.width, this.dimensions.device.char.height, this._coreBrowserService.dpr, this._deviceMaxTextureSize);
        this._charAtlas !== t3 && (this._onChangeTextureAtlas.fire(t3.pages[0].canvas), this._charAtlasDisposable.value = It3(ee3.forward(t3.onAddTextureAtlasCanvas, this._onAddTextureAtlasCanvas), ee3.forward(t3.onRemoveTextureAtlasCanvas, this._onRemoveTextureAtlasCanvas))), this._charAtlas = t3, this._charAtlas.warmUp(), this._glyphRenderer.value?.setAtlas(this._charAtlas);
      }
      _clearModel(t3) {
        this._model.clear(), t3 && this._glyphRenderer.value?.clear();
      }
      clearTextureAtlas() {
        this._charAtlas?.clearTexture(), this._clearModel(true), this._requestRedrawViewport();
      }
      clear() {
        this._clearModel(true);
        for (let t3 of this._renderLayers) t3.reset(this._terminal);
        this._cursorBlinkStateManager.value?.restartBlinkAnimation(), this._updateCursorBlink();
      }
      renderRows(t3, n4) {
        if (!this._isAttached) if (this._core.screenElement?.isConnected && this._charSizeService.width && this._charSizeService.height) this._updateDimensions(), this._refreshCharAtlas(), this._isAttached = true;
        else return;
        for (let s18 of this._renderLayers) s18.handleGridChanged(this._terminal, t3, n4);
        !this._glyphRenderer.value || !this._rectangleRenderer.value || (this._glyphRenderer.value.beginFrame() ? (this._clearModel(true), this._updateModel(0, this._terminal.rows - 1)) : this._updateModel(t3, n4), this._rectangleRenderer.value.renderBackgrounds(), this._glyphRenderer.value.render(this._model), (!this._cursorBlinkStateManager.value || this._cursorBlinkStateManager.value.isCursorVisible) && this._rectangleRenderer.value.renderCursor());
      }
      _updateCursorBlink() {
        this._coreService.decPrivateModes.cursorBlink ?? this._terminal.options.cursorBlink ? this._cursorBlinkStateManager.value = new Ht(() => {
          this._requestRedrawCursor();
        }, this._coreBrowserService) : this._cursorBlinkStateManager.clear(), this._requestRedrawCursor();
      }
      _updateModel(t3, n4) {
        let s18 = this._core, o6 = this._workCell, r7, a3, l4, u3, c4, d3, h4 = 0, f3 = true, I2, L4, M4, q2, S4, W2, E2, y3, w3;
        t3 = mr2(t3, s18.rows - 1, 0), n4 = mr2(n4, s18.rows - 1, 0);
        let G3 = this._coreService.decPrivateModes.cursorStyle ?? s18.options.cursorStyle ?? "block", ue4 = this._terminal.buffer.active.baseY + this._terminal.buffer.active.cursorY, Se2 = ue4 - s18.buffer.ydisp, ce3 = Math.min(this._terminal.buffer.active.cursorX, s18.cols - 1), we3 = -1, A3 = this._coreService.isCursorInitialized && !this._coreService.isCursorHidden && (!this._cursorBlinkStateManager.value || this._cursorBlinkStateManager.value.isCursorVisible);
        this._model.cursor = void 0;
        let se2 = false;
        for (a3 = t3; a3 <= n4; a3++) for (l4 = a3 + s18.buffer.ydisp, u3 = s18.buffer.lines.get(l4), this._model.lineLengths[a3] = 0, M4 = ue4 === l4, h4 = 0, c4 = this._characterJoinerService.getJoinedCharacters(l4), y3 = 0; y3 < s18.cols; y3++) {
          if (r7 = this._cellColorResolver.result.bg, u3.loadCell(y3, o6), y3 === 0 && (r7 = this._cellColorResolver.result.bg), d3 = false, f3 = y3 >= h4, I2 = y3, c4.length > 0 && y3 === c4[0][0] && f3) {
            L4 = c4.shift();
            let v4 = this._model.selection.isCellSelected(this._terminal, L4[0], l4);
            for (E2 = L4[0] + 1; E2 < L4[1]; E2++) f3 && (f3 = v4 === this._model.selection.isCellSelected(this._terminal, E2, l4));
            f3 && (f3 = !M4 || ce3 < L4[0] || ce3 >= L4[1]), f3 ? (d3 = true, o6 = new Ci2(o6, u3.translateToString(true, L4[0], L4[1]), L4[1] - L4[0]), I2 = L4[1] - 1) : h4 = L4[1];
          }
          if (q2 = o6.getChars(), S4 = o6.getCode(), E2 = (a3 * s18.cols + y3) * Ce3, this._cellColorResolver.resolve(o6, y3, l4, this.dimensions.device.cell.width), A3 && l4 === ue4 && (y3 === ce3 && (this._model.cursor = { x: ce3, y: Se2, width: o6.getWidth(), style: this._coreBrowserService.isFocused ? G3 : s18.options.cursorInactiveStyle, cursorWidth: s18.options.cursorWidth, dpr: this._devicePixelRatio }, we3 = ce3 + o6.getWidth() - 1), y3 >= ce3 && y3 <= we3 && (this._coreBrowserService.isFocused && G3 === "block" || this._coreBrowserService.isFocused === false && s18.options.cursorInactiveStyle === "block") && (this._cellColorResolver.result.fg = 50331648 | this._themeService.colors.cursorAccent.rgba >> 8 & 16777215, this._cellColorResolver.result.bg = 50331648 | this._themeService.colors.cursor.rgba >> 8 & 16777215)), S4 !== 0 && (this._model.lineLengths[a3] = y3 + 1), !(this._model.cells[E2] === S4 && this._model.cells[E2 + ze2] === this._cellColorResolver.result.bg && this._model.cells[E2 + qe3] === this._cellColorResolver.result.fg && this._model.cells[E2 + Ct3] === this._cellColorResolver.result.ext) && (se2 = true, q2.length > 1 && (S4 |= Un2), this._model.cells[E2] = S4, this._model.cells[E2 + ze2] = this._cellColorResolver.result.bg, this._model.cells[E2 + qe3] = this._cellColorResolver.result.fg, this._model.cells[E2 + Ct3] = this._cellColorResolver.result.ext, W2 = o6.getWidth(), this._glyphRenderer.value.updateCell(y3, a3, S4, this._cellColorResolver.result.bg, this._cellColorResolver.result.fg, this._cellColorResolver.result.ext, q2, W2, r7), d3)) {
            for (o6 = this._workCell, y3++; y3 <= I2; y3++) w3 = (a3 * s18.cols + y3) * Ce3, this._glyphRenderer.value.updateCell(y3, a3, 0, 0, 0, 0, pn2, 0, 0), this._model.cells[w3] = 0, this._model.cells[w3 + ze2] = this._cellColorResolver.result.bg, this._model.cells[w3 + qe3] = this._cellColorResolver.result.fg, this._model.cells[w3 + Ct3] = this._cellColorResolver.result.ext;
            y3--;
          }
        }
        se2 && this._rectangleRenderer.value.updateBackgrounds(this._model), this._rectangleRenderer.value.updateCursor(this._model);
      }
      _updateDimensions() {
        !this._charSizeService.width || !this._charSizeService.height || (this.dimensions.device.char.width = Math.floor(this._charSizeService.width * this._devicePixelRatio), this.dimensions.device.char.height = Math.ceil(this._charSizeService.height * this._devicePixelRatio), this.dimensions.device.cell.height = Math.floor(this.dimensions.device.char.height * this._optionsService.rawOptions.lineHeight), this.dimensions.device.char.top = this._optionsService.rawOptions.lineHeight === 1 ? 0 : Math.round((this.dimensions.device.cell.height - this.dimensions.device.char.height) / 2), this.dimensions.device.cell.width = this.dimensions.device.char.width + Math.round(this._optionsService.rawOptions.letterSpacing), this.dimensions.device.char.left = Math.floor(this._optionsService.rawOptions.letterSpacing / 2), this.dimensions.device.canvas.height = this._terminal.rows * this.dimensions.device.cell.height, this.dimensions.device.canvas.width = this._terminal.cols * this.dimensions.device.cell.width, this.dimensions.css.canvas.height = Math.round(this.dimensions.device.canvas.height / this._devicePixelRatio), this.dimensions.css.canvas.width = Math.round(this.dimensions.device.canvas.width / this._devicePixelRatio), this.dimensions.css.cell.height = this.dimensions.device.cell.height / this._devicePixelRatio, this.dimensions.css.cell.width = this.dimensions.device.cell.width / this._devicePixelRatio);
      }
      _setCanvasDevicePixelDimensions(t3, n4) {
        this._canvas.width === t3 && this._canvas.height === n4 || (this._canvas.width = t3, this._canvas.height = n4, this._requestRedrawViewport());
      }
      _requestRedrawViewport() {
        this._onRequestRedraw.fire({ start: 0, end: this._terminal.rows - 1 });
      }
      _requestRedrawCursor() {
        let t3 = this._terminal.buffer.active.cursorY;
        this._onRequestRedraw.fire({ start: t3, end: t3 });
      }
    };
    Ci2 = class extends he4 {
      constructor(t3, n4, s18) {
        super();
        this.content = 0;
        this.combinedData = "";
        this.fg = t3.fg, this.bg = t3.bg, this.combinedData = n4, this._width = s18;
      }
      isCombined() {
        return 2097152;
      }
      getWidth() {
        return this._width;
      }
      getChars() {
        return this.combinedData;
      }
      getCode() {
        return 2097151;
      }
      setFromCharData(t3) {
        throw new Error("not implemented");
      }
      getAsCharData() {
        return [this.fg, this.getChars(), this.getWidth(), this.getCode()];
      }
    };
    _r2 = "di$target";
    br3 = "di$dependencies";
    zi2 = /* @__PURE__ */ new Map();
    Vu = pe3("BufferService");
    Cu = pe3("CoreMouseService");
    zu = pe3("CoreService");
    qu = pe3("CharsetService");
    ju = pe3("InstantiationService");
    Xu = pe3("LogService");
    vr2 = pe3("OptionsService");
    Yu = pe3("OscLinkService");
    Qu = pe3("UnicodeService");
    Zu = pe3("DecorationService");
    Hs2 = { trace: 0, debug: 1, info: 2, warn: 3, error: 4, off: 5 };
    Ws2 = "xterm.js: ";
    ci2 = class extends B5 {
      constructor(t3) {
        super();
        this._optionsService = t3;
        this._logLevel = 5;
        this._updateLogLevel(), this._register(this._optionsService.onSpecificOptionChange("logLevel", () => this._updateLogLevel())), Tr2 = this;
      }
      get logLevel() {
        return this._logLevel;
      }
      _updateLogLevel() {
        this._logLevel = Hs2[this._optionsService.rawOptions.logLevel];
      }
      _evalLazyOptionalParams(t3) {
        for (let n4 = 0; n4 < t3.length; n4++) typeof t3[n4] == "function" && (t3[n4] = t3[n4]());
      }
      _log(t3, n4, s18) {
        this._evalLazyOptionalParams(s18), t3.call(console, (this._optionsService.options.logger ? "" : Ws2) + n4, ...s18);
      }
      trace(t3, ...n4) {
        this._logLevel <= 0 && this._log(this._optionsService.options.logger?.trace.bind(this._optionsService.options.logger) ?? console.log, t3, n4);
      }
      debug(t3, ...n4) {
        this._logLevel <= 1 && this._log(this._optionsService.options.logger?.debug.bind(this._optionsService.options.logger) ?? console.log, t3, n4);
      }
      info(t3, ...n4) {
        this._logLevel <= 2 && this._log(this._optionsService.options.logger?.info.bind(this._optionsService.options.logger) ?? console.info, t3, n4);
      }
      warn(t3, ...n4) {
        this._logLevel <= 3 && this._log(this._optionsService.options.logger?.warn.bind(this._optionsService.options.logger) ?? console.warn, t3, n4);
      }
      error(t3, ...n4) {
        this._logLevel <= 4 && this._log(this._optionsService.options.logger?.error.bind(this._optionsService.options.logger) ?? console.error, t3, n4);
      }
    };
    ci2 = Yi2([Qi(0, vr2)], ci2);
    xr2 = class extends B5 {
      constructor(t3) {
        if (vi2 && hn2() < 16) {
          let n4 = { antialias: false, depth: false, preserveDrawingBuffer: true };
          if (!document.createElement("canvas").getContext("webgl2", n4)) throw new Error("Webgl2 is only supported on Safari 16 and above");
        }
        super();
        this._preserveDrawingBuffer = t3;
        this._onChangeTextureAtlas = this._register(new D3());
        this.onChangeTextureAtlas = this._onChangeTextureAtlas.event;
        this._onAddTextureAtlasCanvas = this._register(new D3());
        this.onAddTextureAtlasCanvas = this._onAddTextureAtlasCanvas.event;
        this._onRemoveTextureAtlasCanvas = this._register(new D3());
        this.onRemoveTextureAtlasCanvas = this._onRemoveTextureAtlasCanvas.event;
        this._onContextLoss = this._register(new D3());
        this.onContextLoss = this._onContextLoss.event;
      }
      activate(t3) {
        let n4 = t3._core;
        if (!t3.element) {
          this._register(n4.onWillOpen(() => this.activate(t3)));
          return;
        }
        this._terminal = t3;
        let s18 = n4.coreService, o6 = n4.optionsService, r7 = n4, a3 = r7._renderService, l4 = r7._characterJoinerService, u3 = r7._charSizeService, c4 = r7._coreBrowserService, d3 = r7._decorationService, h4 = r7._logService, f3 = r7._themeService;
        gr3(h4), this._renderer = this._register(new ui2(t3, l4, u3, c4, s18, d3, o6, f3, this._preserveDrawingBuffer)), this._register(ee3.forward(this._renderer.onContextLoss, this._onContextLoss)), this._register(ee3.forward(this._renderer.onChangeTextureAtlas, this._onChangeTextureAtlas)), this._register(ee3.forward(this._renderer.onAddTextureAtlasCanvas, this._onAddTextureAtlasCanvas)), this._register(ee3.forward(this._renderer.onRemoveTextureAtlasCanvas, this._onRemoveTextureAtlasCanvas)), a3.setRenderer(this._renderer), this._register(O2(() => {
          if (this._terminal._core._store._isDisposed) return;
          let I2 = this._terminal._core._renderService;
          I2.setRenderer(this._terminal._core._createRenderer()), I2.handleResize(t3.cols, t3.rows);
        }));
      }
      get textureAtlas() {
        return this._renderer?.textureAtlas;
      }
      clearTextureAtlas() {
        this._renderer?.clearTextureAtlas();
      }
    };
  }
});

// node_modules/.pnpm/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/css-tag.js
var t = globalThis;
var e = t.ShadowRoot && (void 0 === t.ShadyCSS || t.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype;
var s = Symbol();
var o = /* @__PURE__ */ new WeakMap();
var n = class {
  constructor(t3, e4, o6) {
    if (this._$cssResult$ = true, o6 !== s) throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");
    this.cssText = t3, this.t = e4;
  }
  get styleSheet() {
    let t3 = this.o;
    const s18 = this.t;
    if (e && void 0 === t3) {
      const e4 = void 0 !== s18 && 1 === s18.length;
      e4 && (t3 = o.get(s18)), void 0 === t3 && ((this.o = t3 = new CSSStyleSheet()).replaceSync(this.cssText), e4 && o.set(s18, t3));
    }
    return t3;
  }
  toString() {
    return this.cssText;
  }
};
var r = (t3) => new n("string" == typeof t3 ? t3 : t3 + "", void 0, s);
var i = (t3, ...e4) => {
  const o6 = 1 === t3.length ? t3[0] : e4.reduce((e5, s18, o7) => e5 + ((t4) => {
    if (true === t4._$cssResult$) return t4.cssText;
    if ("number" == typeof t4) return t4;
    throw Error("Value passed to 'css' function must be a 'css' function result: " + t4 + ". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.");
  })(s18) + t3[o7 + 1], t3[0]);
  return new n(o6, t3, s);
};
var S = (s18, o6) => {
  if (e) s18.adoptedStyleSheets = o6.map((t3) => t3 instanceof CSSStyleSheet ? t3 : t3.styleSheet);
  else for (const e4 of o6) {
    const o7 = document.createElement("style"), n4 = t.litNonce;
    void 0 !== n4 && o7.setAttribute("nonce", n4), o7.textContent = e4.cssText, s18.appendChild(o7);
  }
};
var c = e ? (t3) => t3 : (t3) => t3 instanceof CSSStyleSheet ? ((t4) => {
  let e4 = "";
  for (const s18 of t4.cssRules) e4 += s18.cssText;
  return r(e4);
})(t3) : t3;

// node_modules/.pnpm/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/reactive-element.js
var { is: i2, defineProperty: e2, getOwnPropertyDescriptor: h, getOwnPropertyNames: r2, getOwnPropertySymbols: o2, getPrototypeOf: n2 } = Object;
var a = globalThis;
var c2 = a.trustedTypes;
var l = c2 ? c2.emptyScript : "";
var p = a.reactiveElementPolyfillSupport;
var d = (t3, s18) => t3;
var u = { toAttribute(t3, s18) {
  switch (s18) {
    case Boolean:
      t3 = t3 ? l : null;
      break;
    case Object:
    case Array:
      t3 = null == t3 ? t3 : JSON.stringify(t3);
  }
  return t3;
}, fromAttribute(t3, s18) {
  let i12 = t3;
  switch (s18) {
    case Boolean:
      i12 = null !== t3;
      break;
    case Number:
      i12 = null === t3 ? null : Number(t3);
      break;
    case Object:
    case Array:
      try {
        i12 = JSON.parse(t3);
      } catch (t4) {
        i12 = null;
      }
  }
  return i12;
} };
var f = (t3, s18) => !i2(t3, s18);
var b = { attribute: true, type: String, converter: u, reflect: false, useDefault: false, hasChanged: f };
Symbol.metadata ?? (Symbol.metadata = Symbol("metadata")), a.litPropertyMetadata ?? (a.litPropertyMetadata = /* @__PURE__ */ new WeakMap());
var y = class extends HTMLElement {
  static addInitializer(t3) {
    this._$Ei(), (this.l ?? (this.l = [])).push(t3);
  }
  static get observedAttributes() {
    return this.finalize(), this._$Eh && [...this._$Eh.keys()];
  }
  static createProperty(t3, s18 = b) {
    if (s18.state && (s18.attribute = false), this._$Ei(), this.prototype.hasOwnProperty(t3) && ((s18 = Object.create(s18)).wrapped = true), this.elementProperties.set(t3, s18), !s18.noAccessor) {
      const i12 = Symbol(), h4 = this.getPropertyDescriptor(t3, i12, s18);
      void 0 !== h4 && e2(this.prototype, t3, h4);
    }
  }
  static getPropertyDescriptor(t3, s18, i12) {
    const { get: e4, set: r7 } = h(this.prototype, t3) ?? { get() {
      return this[s18];
    }, set(t4) {
      this[s18] = t4;
    } };
    return { get: e4, set(s19) {
      const h4 = e4?.call(this);
      r7?.call(this, s19), this.requestUpdate(t3, h4, i12);
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
      const t4 = this.properties, s18 = [...r2(t4), ...o2(t4)];
      for (const i12 of s18) this.createProperty(i12, t4[i12]);
    }
    const t3 = this[Symbol.metadata];
    if (null !== t3) {
      const s18 = litPropertyMetadata.get(t3);
      if (void 0 !== s18) for (const [t4, i12] of s18) this.elementProperties.set(t4, i12);
    }
    this._$Eh = /* @__PURE__ */ new Map();
    for (const [t4, s18] of this.elementProperties) {
      const i12 = this._$Eu(t4, s18);
      void 0 !== i12 && this._$Eh.set(i12, t4);
    }
    this.elementStyles = this.finalizeStyles(this.styles);
  }
  static finalizeStyles(s18) {
    const i12 = [];
    if (Array.isArray(s18)) {
      const e4 = new Set(s18.flat(1 / 0).reverse());
      for (const s19 of e4) i12.unshift(c(s19));
    } else void 0 !== s18 && i12.push(c(s18));
    return i12;
  }
  static _$Eu(t3, s18) {
    const i12 = s18.attribute;
    return false === i12 ? void 0 : "string" == typeof i12 ? i12 : "string" == typeof t3 ? t3.toLowerCase() : void 0;
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
    const t3 = /* @__PURE__ */ new Map(), s18 = this.constructor.elementProperties;
    for (const i12 of s18.keys()) this.hasOwnProperty(i12) && (t3.set(i12, this[i12]), delete this[i12]);
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
  attributeChangedCallback(t3, s18, i12) {
    this._$AK(t3, i12);
  }
  _$ET(t3, s18) {
    const i12 = this.constructor.elementProperties.get(t3), e4 = this.constructor._$Eu(t3, i12);
    if (void 0 !== e4 && true === i12.reflect) {
      const h4 = (void 0 !== i12.converter?.toAttribute ? i12.converter : u).toAttribute(s18, i12.type);
      this._$Em = t3, null == h4 ? this.removeAttribute(e4) : this.setAttribute(e4, h4), this._$Em = null;
    }
  }
  _$AK(t3, s18) {
    const i12 = this.constructor, e4 = i12._$Eh.get(t3);
    if (void 0 !== e4 && this._$Em !== e4) {
      const t4 = i12.getPropertyOptions(e4), h4 = "function" == typeof t4.converter ? { fromAttribute: t4.converter } : void 0 !== t4.converter?.fromAttribute ? t4.converter : u;
      this._$Em = e4;
      const r7 = h4.fromAttribute(s18, t4.type);
      this[e4] = r7 ?? this._$Ej?.get(e4) ?? r7, this._$Em = null;
    }
  }
  requestUpdate(t3, s18, i12, e4 = false, h4) {
    if (void 0 !== t3) {
      const r7 = this.constructor;
      if (false === e4 && (h4 = this[t3]), i12 ?? (i12 = r7.getPropertyOptions(t3)), !((i12.hasChanged ?? f)(h4, s18) || i12.useDefault && i12.reflect && h4 === this._$Ej?.get(t3) && !this.hasAttribute(r7._$Eu(t3, i12)))) return;
      this.C(t3, s18, i12);
    }
    false === this.isUpdatePending && (this._$ES = this._$EP());
  }
  C(t3, s18, { useDefault: i12, reflect: e4, wrapped: h4 }, r7) {
    i12 && !(this._$Ej ?? (this._$Ej = /* @__PURE__ */ new Map())).has(t3) && (this._$Ej.set(t3, r7 ?? s18 ?? this[t3]), true !== h4 || void 0 !== r7) || (this._$AL.has(t3) || (this.hasUpdated || i12 || (s18 = void 0), this._$AL.set(t3, s18)), true === e4 && this._$Em !== t3 && (this._$Eq ?? (this._$Eq = /* @__PURE__ */ new Set())).add(t3));
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
        for (const [t5, s19] of this._$Ep) this[t5] = s19;
        this._$Ep = void 0;
      }
      const t4 = this.constructor.elementProperties;
      if (t4.size > 0) for (const [s19, i12] of t4) {
        const { wrapped: t5 } = i12, e4 = this[s19];
        true !== t5 || this._$AL.has(s19) || void 0 === e4 || this.C(s19, void 0, i12, e4);
      }
    }
    let t3 = false;
    const s18 = this._$AL;
    try {
      t3 = this.shouldUpdate(s18), t3 ? (this.willUpdate(s18), this._$EO?.forEach((t4) => t4.hostUpdate?.()), this.update(s18)) : this._$EM();
    } catch (s19) {
      throw t3 = false, this._$EM(), s19;
    }
    t3 && this._$AE(s18);
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

// node_modules/.pnpm/lit-html@3.3.2/node_modules/lit-html/lit-html.js
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
var x = (t3) => (i12, ...s18) => ({ _$litType$: t3, strings: i12, values: s18 });
var b2 = x(1);
var w = x(2);
var T = x(3);
var E = Symbol.for("lit-noChange");
var A = Symbol.for("lit-nothing");
var C = /* @__PURE__ */ new WeakMap();
var P = l2.createTreeWalker(l2, 129);
function V(t3, i12) {
  if (!u2(t3) || !t3.hasOwnProperty("raw")) throw Error("invalid template strings array");
  return void 0 !== e3 ? e3.createHTML(i12) : i12;
}
var N = (t3, i12) => {
  const s18 = t3.length - 1, e4 = [];
  let n4, l4 = 2 === i12 ? "<svg>" : 3 === i12 ? "<math>" : "", c4 = v;
  for (let i13 = 0; i13 < s18; i13++) {
    const s19 = t3[i13];
    let a3, u3, d3 = -1, f3 = 0;
    for (; f3 < s19.length && (c4.lastIndex = f3, u3 = c4.exec(s19), null !== u3); ) f3 = c4.lastIndex, c4 === v ? "!--" === u3[1] ? c4 = _ : void 0 !== u3[1] ? c4 = m : void 0 !== u3[2] ? (y2.test(u3[2]) && (n4 = RegExp("</" + u3[2], "g")), c4 = p2) : void 0 !== u3[3] && (c4 = p2) : c4 === p2 ? ">" === u3[0] ? (c4 = n4 ?? v, d3 = -1) : void 0 === u3[1] ? d3 = -2 : (d3 = c4.lastIndex - u3[2].length, a3 = u3[1], c4 = void 0 === u3[3] ? p2 : '"' === u3[3] ? $ : g) : c4 === $ || c4 === g ? c4 = p2 : c4 === _ || c4 === m ? c4 = v : (c4 = p2, n4 = void 0);
    const x2 = c4 === p2 && t3[i13 + 1].startsWith("/>") ? " " : "";
    l4 += c4 === v ? s19 + r3 : d3 >= 0 ? (e4.push(a3), s19.slice(0, d3) + h2 + s19.slice(d3) + o3 + x2) : s19 + o3 + (-2 === d3 ? i13 : x2);
  }
  return [V(t3, l4 + (t3[s18] || "<?>") + (2 === i12 ? "</svg>" : 3 === i12 ? "</math>" : "")), e4];
};
var S2 = class _S {
  constructor({ strings: t3, _$litType$: i12 }, e4) {
    let r7;
    this.parts = [];
    let l4 = 0, a3 = 0;
    const u3 = t3.length - 1, d3 = this.parts, [f3, v4] = N(t3, i12);
    if (this.el = _S.createElement(f3, e4), P.currentNode = this.el.content, 2 === i12 || 3 === i12) {
      const t4 = this.el.content.firstChild;
      t4.replaceWith(...t4.childNodes);
    }
    for (; null !== (r7 = P.nextNode()) && d3.length < u3; ) {
      if (1 === r7.nodeType) {
        if (r7.hasAttributes()) for (const t4 of r7.getAttributeNames()) if (t4.endsWith(h2)) {
          const i13 = v4[a3++], s18 = r7.getAttribute(t4).split(o3), e5 = /([.?@])?(.*)/.exec(i13);
          d3.push({ type: 1, index: l4, name: e5[2], strings: s18, ctor: "." === e5[1] ? I : "?" === e5[1] ? L : "@" === e5[1] ? z : H }), r7.removeAttribute(t4);
        } else t4.startsWith(o3) && (d3.push({ type: 6, index: l4 }), r7.removeAttribute(t4));
        if (y2.test(r7.tagName)) {
          const t4 = r7.textContent.split(o3), i13 = t4.length - 1;
          if (i13 > 0) {
            r7.textContent = s2 ? s2.emptyScript : "";
            for (let s18 = 0; s18 < i13; s18++) r7.append(t4[s18], c3()), P.nextNode(), d3.push({ type: 2, index: ++l4 });
            r7.append(t4[i13], c3());
          }
        }
      } else if (8 === r7.nodeType) if (r7.data === n3) d3.push({ type: 2, index: l4 });
      else {
        let t4 = -1;
        for (; -1 !== (t4 = r7.data.indexOf(o3, t4 + 1)); ) d3.push({ type: 7, index: l4 }), t4 += o3.length - 1;
      }
      l4++;
    }
  }
  static createElement(t3, i12) {
    const s18 = l2.createElement("template");
    return s18.innerHTML = t3, s18;
  }
};
function M(t3, i12, s18 = t3, e4) {
  if (i12 === E) return i12;
  let h4 = void 0 !== e4 ? s18._$Co?.[e4] : s18._$Cl;
  const o6 = a2(i12) ? void 0 : i12._$litDirective$;
  return h4?.constructor !== o6 && (h4?._$AO?.(false), void 0 === o6 ? h4 = void 0 : (h4 = new o6(t3), h4._$AT(t3, s18, e4)), void 0 !== e4 ? (s18._$Co ?? (s18._$Co = []))[e4] = h4 : s18._$Cl = h4), void 0 !== h4 && (i12 = M(t3, h4._$AS(t3, i12.values), h4, e4)), i12;
}
var R = class {
  constructor(t3, i12) {
    this._$AV = [], this._$AN = void 0, this._$AD = t3, this._$AM = i12;
  }
  get parentNode() {
    return this._$AM.parentNode;
  }
  get _$AU() {
    return this._$AM._$AU;
  }
  u(t3) {
    const { el: { content: i12 }, parts: s18 } = this._$AD, e4 = (t3?.creationScope ?? l2).importNode(i12, true);
    P.currentNode = e4;
    let h4 = P.nextNode(), o6 = 0, n4 = 0, r7 = s18[0];
    for (; void 0 !== r7; ) {
      if (o6 === r7.index) {
        let i13;
        2 === r7.type ? i13 = new k(h4, h4.nextSibling, this, t3) : 1 === r7.type ? i13 = new r7.ctor(h4, r7.name, r7.strings, this, t3) : 6 === r7.type && (i13 = new Z(h4, this, t3)), this._$AV.push(i13), r7 = s18[++n4];
      }
      o6 !== r7?.index && (h4 = P.nextNode(), o6++);
    }
    return P.currentNode = l2, e4;
  }
  p(t3) {
    let i12 = 0;
    for (const s18 of this._$AV) void 0 !== s18 && (void 0 !== s18.strings ? (s18._$AI(t3, s18, i12), i12 += s18.strings.length - 2) : s18._$AI(t3[i12])), i12++;
  }
};
var k = class _k {
  get _$AU() {
    return this._$AM?._$AU ?? this._$Cv;
  }
  constructor(t3, i12, s18, e4) {
    this.type = 2, this._$AH = A, this._$AN = void 0, this._$AA = t3, this._$AB = i12, this._$AM = s18, this.options = e4, this._$Cv = e4?.isConnected ?? true;
  }
  get parentNode() {
    let t3 = this._$AA.parentNode;
    const i12 = this._$AM;
    return void 0 !== i12 && 11 === t3?.nodeType && (t3 = i12.parentNode), t3;
  }
  get startNode() {
    return this._$AA;
  }
  get endNode() {
    return this._$AB;
  }
  _$AI(t3, i12 = this) {
    t3 = M(this, t3, i12), a2(t3) ? t3 === A || null == t3 || "" === t3 ? (this._$AH !== A && this._$AR(), this._$AH = A) : t3 !== this._$AH && t3 !== E && this._(t3) : void 0 !== t3._$litType$ ? this.$(t3) : void 0 !== t3.nodeType ? this.T(t3) : d2(t3) ? this.k(t3) : this._(t3);
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
    const { values: i12, _$litType$: s18 } = t3, e4 = "number" == typeof s18 ? this._$AC(t3) : (void 0 === s18.el && (s18.el = S2.createElement(V(s18.h, s18.h[0]), this.options)), s18);
    if (this._$AH?._$AD === e4) this._$AH.p(i12);
    else {
      const t4 = new R(e4, this), s19 = t4.u(this.options);
      t4.p(i12), this.T(s19), this._$AH = t4;
    }
  }
  _$AC(t3) {
    let i12 = C.get(t3.strings);
    return void 0 === i12 && C.set(t3.strings, i12 = new S2(t3)), i12;
  }
  k(t3) {
    u2(this._$AH) || (this._$AH = [], this._$AR());
    const i12 = this._$AH;
    let s18, e4 = 0;
    for (const h4 of t3) e4 === i12.length ? i12.push(s18 = new _k(this.O(c3()), this.O(c3()), this, this.options)) : s18 = i12[e4], s18._$AI(h4), e4++;
    e4 < i12.length && (this._$AR(s18 && s18._$AB.nextSibling, e4), i12.length = e4);
  }
  _$AR(t3 = this._$AA.nextSibling, s18) {
    for (this._$AP?.(false, true, s18); t3 !== this._$AB; ) {
      const s19 = i3(t3).nextSibling;
      i3(t3).remove(), t3 = s19;
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
  constructor(t3, i12, s18, e4, h4) {
    this.type = 1, this._$AH = A, this._$AN = void 0, this.element = t3, this.name = i12, this._$AM = e4, this.options = h4, s18.length > 2 || "" !== s18[0] || "" !== s18[1] ? (this._$AH = Array(s18.length - 1).fill(new String()), this.strings = s18) : this._$AH = A;
  }
  _$AI(t3, i12 = this, s18, e4) {
    const h4 = this.strings;
    let o6 = false;
    if (void 0 === h4) t3 = M(this, t3, i12, 0), o6 = !a2(t3) || t3 !== this._$AH && t3 !== E, o6 && (this._$AH = t3);
    else {
      const e5 = t3;
      let n4, r7;
      for (t3 = h4[0], n4 = 0; n4 < h4.length - 1; n4++) r7 = M(this, e5[s18 + n4], i12, n4), r7 === E && (r7 = this._$AH[n4]), o6 || (o6 = !a2(r7) || r7 !== this._$AH[n4]), r7 === A ? t3 = A : t3 !== A && (t3 += (r7 ?? "") + h4[n4 + 1]), this._$AH[n4] = r7;
    }
    o6 && !e4 && this.j(t3);
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
  constructor(t3, i12, s18, e4, h4) {
    super(t3, i12, s18, e4, h4), this.type = 5;
  }
  _$AI(t3, i12 = this) {
    if ((t3 = M(this, t3, i12, 0) ?? A) === E) return;
    const s18 = this._$AH, e4 = t3 === A && s18 !== A || t3.capture !== s18.capture || t3.once !== s18.once || t3.passive !== s18.passive, h4 = t3 !== A && (s18 === A || e4);
    e4 && this.element.removeEventListener(this.name, this, s18), h4 && this.element.addEventListener(this.name, this, t3), this._$AH = t3;
  }
  handleEvent(t3) {
    "function" == typeof this._$AH ? this._$AH.call(this.options?.host ?? this.element, t3) : this._$AH.handleEvent(t3);
  }
};
var Z = class {
  constructor(t3, i12, s18) {
    this.element = t3, this.type = 6, this._$AN = void 0, this._$AM = i12, this.options = s18;
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
var D = (t3, i12, s18) => {
  const e4 = s18?.renderBefore ?? i12;
  let h4 = e4._$litPart$;
  if (void 0 === h4) {
    const t4 = s18?.renderBefore ?? null;
    e4._$litPart$ = h4 = new k(i12.insertBefore(c3(), t4), t4, void 0, s18 ?? {});
  }
  return h4._$AI(t3), h4;
};

// node_modules/.pnpm/lit-element@4.2.2/node_modules/lit-element/lit-element.js
var s3 = globalThis;
var i4 = class extends y {
  constructor() {
    super(...arguments), this.renderOptions = { host: this }, this._$Do = void 0;
  }
  createRenderRoot() {
    var _a2;
    const t3 = super.createRenderRoot();
    return (_a2 = this.renderOptions).renderBefore ?? (_a2.renderBefore = t3.firstChild), t3;
  }
  update(t3) {
    const r7 = this.render();
    this.hasUpdated || (this.renderOptions.isConnected = this.isConnected), super.update(t3), this._$Do = D(r7, this.renderRoot, this.renderOptions);
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

// packages/ui/src/web-components/util.ts
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

// packages/ui/src/web-components/ship-terminal.styles.ts
var shipTerminalStyles = [reducedMotion, i`
  :host {
    display: block;
    width: 100%;
    position: relative;
    overflow: hidden;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--background);
  }
  :host([hidden]) { display: none; }

  .terminal-wrap {
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
  }

  /* xterm canvas renderer corrupts row 0 glyphs on first write (shadow DOM bug).
     A sacrificial writeln('') in _init absorbs the corruption. On canvas
     fallback, clip the corrupted row and pad to compensate. */
  :host(.canvas-fallback) .terminal-wrap {
    clip-path: inset(var(--xterm-row-h, 20px) 0 0 0);
    margin-top: calc(-1 * var(--xterm-row-h, 20px));
  }

  /* xterm.js injects its own styles on the container; we ensure the
     wrapper fills the host and remove any extra padding xterm adds. */
  .terminal-wrap .xterm {
    padding: var(--space-2) var(--space-3);
  }

  .terminal-wrap .xterm-viewport {
    overflow-y: auto;
  }

  /* Search overlay — positioned top-right inside the terminal */
  .search-bar {
    position: absolute;
    top: var(--space-2);
    right: var(--space-3);
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-sm);
    z-index: var(--z-elevated);
    opacity: 0;
    pointer-events: none;
    transform: translateY(-4px);
    transition:
      opacity var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }
  .search-bar[data-open='true'] {
    opacity: 1;
    pointer-events: auto;
    transform: translateY(0);
  }
  .search-bar input {
    font-family: inherit;
    font-size: var(--text-sm);
    padding: 2px var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--background);
    color: var(--foreground);
    outline: none;
    width: 180px;
  }
  .search-bar input:focus {
    border-color: var(--primary);
    box-shadow:
      0 0 0 var(--ring-width)
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }
  .search-bar button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--muted-foreground);
    border-radius: var(--radius-sm);
    cursor: pointer;
    font-size: var(--text-sm);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }
  .search-bar button:hover {
    background: var(--muted);
    color: var(--foreground);
  }

  @media (forced-colors: active) {
    :host {
      outline: 1px solid CanvasText;
    }
    .search-bar input:focus {
      outline: 2px solid CanvasText;
      outline-offset: 2px;
    }
  }
`];

// packages/ui/src/web-components/ship-terminal.theme.ts
var FALLBACKS = {
  background: "#fafaf9",
  foreground: "#1c1917",
  cursor: "#c2410c",
  cursorAccent: "#fafaf9",
  selectionBackground: "rgba(194, 65, 12, 0.3)",
  black: "#78716c",
  red: "#dc2626",
  green: "#16a34a",
  yellow: "#ca8a04",
  blue: "#2563eb",
  magenta: "#9333ea",
  cyan: "#0891b2",
  white: "#1c1917",
  brightBlack: "#a8a29e",
  brightRed: "#ef4444",
  brightGreen: "#22c55e",
  brightYellow: "#eab308",
  brightBlue: "#3b82f6",
  brightMagenta: "#a855f7",
  brightCyan: "#06b6d4",
  brightWhite: "#fafaf9"
};
var TOKEN_MAP = {
  background: "--background",
  foreground: "--foreground",
  cursor: "--primary",
  cursorAccent: "--background",
  black: "--muted-foreground",
  red: "--destructive",
  green: "--success",
  yellow: "--warning",
  blue: "--info",
  magenta: "--accent",
  white: "--foreground"
};
function resolve(el2, prop) {
  return getComputedStyle(el2).getPropertyValue(prop).trim();
}
function buildTheme(host) {
  const theme = { ...FALLBACKS };
  for (const [key, cssVar] of Object.entries(TOKEN_MAP)) {
    const val = resolve(host, cssVar);
    if (val) theme[key] = val;
  }
  const primary = theme.cursor || FALLBACKS.cursor;
  theme.selectionBackground = `color-mix(in oklch, ${primary} 30%, transparent)`;
  theme.cyan = "oklch(0.7 0.15 200)";
  theme.brightBlack = theme.white;
  theme.brightRed = theme.red;
  theme.brightGreen = theme.green;
  theme.brightYellow = theme.yellow;
  theme.brightBlue = theme.blue;
  theme.brightMagenta = theme.magenta;
  theme.brightCyan = theme.cyan;
  theme.brightWhite = theme.background;
  return theme;
}

// packages/ui/src/web-components/ship-terminal.init.ts
async function injectXtermCSS(root) {
  const sheet = new CSSStyleSheet();
  try {
    await sheet.replace(`
      .xterm { position: relative; user-select: none; }
      .xterm.focus, .xterm:focus { outline: none; }
      .xterm .xterm-helpers { position: absolute; top: 0; z-index: 5; }
      .xterm .xterm-helper-textarea {
        padding: 0; border: 0; margin: 0;
        position: absolute; opacity: 0; left: -9999em; top: 0;
        width: 0; height: 0; z-index: -5;
        white-space: nowrap; overflow: hidden; resize: none;
      }
      .xterm .composition-view {
        display: none; position: absolute;
        white-space: nowrap; z-index: 1;
      }
      .xterm .xterm-viewport {
        background-color: transparent !important;
        overflow-y: scroll; cursor: default;
        position: absolute; right: 0; left: 0; top: 0; bottom: 0;
      }
      .xterm .xterm-screen { position: relative; }
      .xterm .xterm-screen canvas { position: absolute; left: 0; top: 0; }
      .xterm .xterm-decoration-container {
        position: absolute; top: 0; left: 0; right: 0;
      }
    `);
    root.adoptedStyleSheets = [...root.adoptedStyleSheets, sheet];
  } catch {
  }
}
async function createTerminal(opts) {
  const [
    { Terminal },
    { FitAddon },
    { WebLinksAddon },
    { SearchAddon },
    webglMod
  ] = await Promise.all([
    Promise.resolve().then(() => (init_xterm(), xterm_exports)),
    Promise.resolve().then(() => (init_addon_fit(), addon_fit_exports)),
    Promise.resolve().then(() => (init_addon_web_links(), addon_web_links_exports)),
    Promise.resolve().then(() => (init_addon_search(), addon_search_exports)),
    Promise.resolve().then(() => (init_addon_webgl(), addon_webgl_exports)).catch(() => null)
  ]);
  const term = new Terminal({
    rows: opts.rows,
    cols: opts.cols,
    fontSize: opts.fontSize,
    fontFamily: MONO_STACK,
    theme: buildTheme(opts.host),
    cursorBlink: !opts.readonly,
    disableStdin: opts.readonly,
    allowProposedApi: true,
    convertEol: true
  });
  const fitAddon = new FitAddon();
  const searchAddon = new SearchAddon();
  term.loadAddon(fitAddon);
  term.loadAddon(new WebLinksAddon());
  term.loadAddon(searchAddon);
  if (webglMod) {
    try {
      const webgl = new webglMod.WebglAddon();
      webgl.onContextLoss(() => {
        webgl.dispose();
      });
      term.loadAddon(webgl);
    } catch {
    }
  }
  await new Promise((resolve2) => requestAnimationFrame(() => {
    term.open(opts.container);
    resolve2();
  }));
  if (!opts.readonly) {
    term.onData(opts.onData);
  }
  term.attachCustomKeyEventHandler((ev) => {
    if ((ev.ctrlKey || ev.metaKey) && ev.key === "f" && ev.type === "keydown") {
      ev.preventDefault();
      opts.onSearchToggle();
      return false;
    }
    return true;
  });
  return { terminal: term, fitAddon, searchAddon };
}

// packages/ui/src/web-components/ship-terminal.ts
var ShipTerminal = class extends i4 {
  constructor() {
    super(...arguments);
    this.rows = 24;
    this.cols = 80;
    this.fontSize = 0;
    this.readonly = false;
    this.theme = "auto";
    this.fit = false;
    this._terminal = null;
    this._fitAddon = null;
    this._searchAddon = null;
    this._resizeObserver = null;
    this._searchOpen = false;
    this._searchQuery = "";
    this._onSearchInput = (e4) => {
      this._searchQuery = e4.target.value;
      if (this._searchQuery) this._searchAddon?.findNext(this._searchQuery);
    };
    this._onSearchKeydown = (e4) => {
      if (e4.key === "Escape") {
        e4.preventDefault();
        this._searchOpen = false;
        this._searchQuery = "";
        this.requestUpdate();
        this._terminal?.focus();
      } else if (e4.key === "Enter") {
        e4.preventDefault();
        if (e4.shiftKey) {
          this._searchAddon?.findPrevious(this._searchQuery);
        } else {
          this._searchAddon?.findNext(this._searchQuery);
        }
      }
    };
    this._searchNext = () => {
      this._searchAddon?.findNext(this._searchQuery);
    };
    this._searchPrev = () => {
      this._searchAddon?.findPrevious(this._searchQuery);
    };
  }
  /** Access the raw xterm.js Terminal instance. */
  get terminal() {
    return this._terminal;
  }
  /** Write text to the terminal. */
  write(data) {
    this._terminal?.write(data);
  }
  /** Write text + newline to the terminal. */
  writeln(data) {
    this._terminal?.writeln(data);
  }
  /** Clear the terminal screen. */
  clear() {
    this._terminal?.clear();
  }
  disconnectedCallback() {
    super.disconnectedCallback();
    this._resizeObserver?.disconnect();
    this._resizeObserver = null;
    if (this._terminal) {
      try {
        this._terminal.dispose();
      } catch {
      }
    }
    this._terminal = null;
    this._fitAddon = null;
    this._searchAddon = null;
  }
  firstUpdated() {
    void this._init();
  }
  updated(changed) {
    if (!this._terminal) return;
    if (changed.has("theme")) {
      this._terminal.options.theme = buildTheme(this);
    }
    if (changed.has("fontSize") && this.fontSize > 0) {
      this._terminal.options.fontSize = this.fontSize;
      this._fitAddon?.fit();
    }
    if (changed.has("fit")) {
      if (this.fit) {
        this._fitAddon?.fit();
        this._attachResizeObserver();
      } else {
        this._resizeObserver?.disconnect();
        this._resizeObserver = null;
      }
    }
  }
  async _init() {
    const wrap = this.renderRoot.querySelector("[data-terminal]");
    if (!wrap) return;
    await injectXtermCSS(this.renderRoot);
    const resolvedFontSize = this.fontSize > 0 ? this.fontSize : this._resolveTokenFontSize();
    let bundle;
    try {
      bundle = await createTerminal({
        container: wrap,
        host: this,
        rows: this.rows,
        cols: this.cols,
        fontSize: resolvedFontSize,
        readonly: this.readonly,
        onData: (data) => {
          this.dispatchEvent(new CustomEvent("ship-data", {
            detail: { data },
            bubbles: true,
            composed: true
          }));
        },
        onSearchToggle: () => this._toggleSearch()
      });
    } catch {
      return;
    }
    this._terminal = bundle.terminal;
    this._fitAddon = bundle.fitAddon;
    this._searchAddon = bundle.searchAddon;
    if (this.fit) {
      bundle.fitAddon.fit();
      this._attachResizeObserver();
    }
    const isWebGL = !!bundle.terminal._core?._renderService?._renderer?.value?.constructor?.name?.includes("Webgl");
    if (!isWebGL) {
      bundle.terminal.writeln("");
      const wrap2 = this.renderRoot.querySelector("[data-terminal]");
      const xtermRows = wrap2?.querySelector(".xterm-rows");
      const firstRow = xtermRows?.children[0];
      const secondRow = xtermRows?.children[1];
      const rowH = firstRow && secondRow ? secondRow.offsetTop - firstRow.offsetTop + 8 : Math.ceil(resolvedFontSize * 1.6);
      this.style.setProperty("--xterm-row-h", `${rowH}px`);
      this.classList.add("canvas-fallback");
    }
    this.dispatchEvent(new CustomEvent("ship-ready", {
      bubbles: true,
      composed: true
    }));
  }
  _resolveTokenFontSize() {
    const raw = getComputedStyle(this).getPropertyValue("--text-sm").trim();
    const parsed = parseFloat(raw);
    return parsed > 0 ? parsed : 12;
  }
  _attachResizeObserver() {
    if (this._resizeObserver) return;
    this._resizeObserver = new ResizeObserver(() => {
      this._fitAddon?.fit();
    });
    this._resizeObserver.observe(this);
  }
  _toggleSearch() {
    this._searchOpen = !this._searchOpen;
    this.requestUpdate();
    if (this._searchOpen) {
      requestAnimationFrame(() => {
        const input = this.renderRoot.querySelector(".search-bar input");
        input?.focus();
      });
    }
  }
  render() {
    return b2`
      <div class="terminal-wrap" part="wrap" data-terminal
        role=${this.readonly ? "log" : "terminal"}
        aria-label=${this.readonly ? "Log output" : "Terminal"}
        aria-readonly=${this.readonly ? "true" : A}
      ></div>
      <div class="search-bar" data-open=${this._searchOpen ? "true" : "false"}>
        <input
          type="text"
          placeholder="Search..."
          aria-label="Search terminal"
          .value=${this._searchQuery}
          @input=${this._onSearchInput}
          @keydown=${this._onSearchKeydown}
        />
        <button @click=${this._searchPrev} aria-label="Previous match"
          title="Previous (Shift+Enter)">&#x2191;</button>
        <button @click=${this._searchNext} aria-label="Next match"
          title="Next (Enter)">&#x2193;</button>
      </div>
    `;
  }
};
ShipTerminal.styles = shipTerminalStyles;
ShipTerminal.properties = {
  rows: { type: Number },
  cols: { type: Number },
  fontSize: { type: Number, attribute: "font-size" },
  readonly: { type: Boolean, reflect: true },
  theme: { type: String, reflect: true },
  fit: { type: Boolean, reflect: true }
};
defineOnce("ship-terminal", ShipTerminal);
export {
  ShipTerminal
};
/*! Bundled license information:

@xterm/xterm/lib/xterm.mjs:
@xterm/addon-fit/lib/addon-fit.mjs:
@xterm/addon-web-links/lib/addon-web-links.mjs:
@xterm/addon-search/lib/addon-search.mjs:
@xterm/addon-webgl/lib/addon-webgl.mjs:
  (**
   * Copyright (c) 2014-2024 The xterm.js authors. All rights reserved.
   * @license MIT
   *
   * Copyright (c) 2012-2013, Christopher Jeffrey (MIT License)
   * @license MIT
   *
   * Originally forked from (with the author's permission):
   *   Fabrice Bellard's javascript vt100 for jslinux:
   *   http://bellard.org/jslinux/
   *   Copyright (c) 2011 Fabrice Bellard
   *)

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
