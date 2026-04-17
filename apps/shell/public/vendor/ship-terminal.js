var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __commonJS = (cb, mod) => function __require() {
  return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", { value: mod, enumerable: true }) : target,
  mod
));

// ../../node_modules/.pnpm/@xterm+xterm@5.5.0/node_modules/@xterm/xterm/lib/xterm.js
var require_xterm = __commonJS({
  "../../node_modules/.pnpm/@xterm+xterm@5.5.0/node_modules/@xterm/xterm/lib/xterm.js"(exports, module) {
    !(function(e4, t3) {
      if ("object" == typeof exports && "object" == typeof module) module.exports = t3();
      else if ("function" == typeof define && define.amd) define([], t3);
      else {
        var i5 = t3();
        for (var s4 in i5) ("object" == typeof exports ? exports : e4)[s4] = i5[s4];
      }
    })(globalThis, (() => (() => {
      "use strict";
      var e4 = { 4567: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.AccessibilityManager = void 0;
        const n4 = i6(9042), o5 = i6(9924), a3 = i6(844), h3 = i6(4725), c4 = i6(2585), l3 = i6(3656);
        let d3 = t4.AccessibilityManager = class extends a3.Disposable {
          constructor(e6, t5, i7, s6) {
            super(), this._terminal = e6, this._coreBrowserService = i7, this._renderService = s6, this._rowColumns = /* @__PURE__ */ new WeakMap(), this._liveRegionLineCount = 0, this._charsToConsume = [], this._charsToAnnounce = "", this._accessibilityContainer = this._coreBrowserService.mainDocument.createElement("div"), this._accessibilityContainer.classList.add("xterm-accessibility"), this._rowContainer = this._coreBrowserService.mainDocument.createElement("div"), this._rowContainer.setAttribute("role", "list"), this._rowContainer.classList.add("xterm-accessibility-tree"), this._rowElements = [];
            for (let e7 = 0; e7 < this._terminal.rows; e7++) this._rowElements[e7] = this._createAccessibilityTreeNode(), this._rowContainer.appendChild(this._rowElements[e7]);
            if (this._topBoundaryFocusListener = (e7) => this._handleBoundaryFocus(e7, 0), this._bottomBoundaryFocusListener = (e7) => this._handleBoundaryFocus(e7, 1), this._rowElements[0].addEventListener("focus", this._topBoundaryFocusListener), this._rowElements[this._rowElements.length - 1].addEventListener("focus", this._bottomBoundaryFocusListener), this._refreshRowsDimensions(), this._accessibilityContainer.appendChild(this._rowContainer), this._liveRegion = this._coreBrowserService.mainDocument.createElement("div"), this._liveRegion.classList.add("live-region"), this._liveRegion.setAttribute("aria-live", "assertive"), this._accessibilityContainer.appendChild(this._liveRegion), this._liveRegionDebouncer = this.register(new o5.TimeBasedDebouncer(this._renderRows.bind(this))), !this._terminal.element) throw new Error("Cannot enable accessibility before Terminal.open");
            this._terminal.element.insertAdjacentElement("afterbegin", this._accessibilityContainer), this.register(this._terminal.onResize(((e7) => this._handleResize(e7.rows)))), this.register(this._terminal.onRender(((e7) => this._refreshRows(e7.start, e7.end)))), this.register(this._terminal.onScroll((() => this._refreshRows()))), this.register(this._terminal.onA11yChar(((e7) => this._handleChar(e7)))), this.register(this._terminal.onLineFeed((() => this._handleChar("\n")))), this.register(this._terminal.onA11yTab(((e7) => this._handleTab(e7)))), this.register(this._terminal.onKey(((e7) => this._handleKey(e7.key)))), this.register(this._terminal.onBlur((() => this._clearLiveRegion()))), this.register(this._renderService.onDimensionsChange((() => this._refreshRowsDimensions()))), this.register((0, l3.addDisposableDomListener)(document, "selectionchange", (() => this._handleSelectionChange()))), this.register(this._coreBrowserService.onDprChange((() => this._refreshRowsDimensions()))), this._refreshRows(), this.register((0, a3.toDisposable)((() => {
              this._accessibilityContainer.remove(), this._rowElements.length = 0;
            })));
          }
          _handleTab(e6) {
            for (let t5 = 0; t5 < e6; t5++) this._handleChar(" ");
          }
          _handleChar(e6) {
            this._liveRegionLineCount < 21 && (this._charsToConsume.length > 0 ? this._charsToConsume.shift() !== e6 && (this._charsToAnnounce += e6) : this._charsToAnnounce += e6, "\n" === e6 && (this._liveRegionLineCount++, 21 === this._liveRegionLineCount && (this._liveRegion.textContent += n4.tooMuchOutput)));
          }
          _clearLiveRegion() {
            this._liveRegion.textContent = "", this._liveRegionLineCount = 0;
          }
          _handleKey(e6) {
            this._clearLiveRegion(), /\p{Control}/u.test(e6) || this._charsToConsume.push(e6);
          }
          _refreshRows(e6, t5) {
            this._liveRegionDebouncer.refresh(e6, t5, this._terminal.rows);
          }
          _renderRows(e6, t5) {
            const i7 = this._terminal.buffer, s6 = i7.lines.length.toString();
            for (let r5 = e6; r5 <= t5; r5++) {
              const e7 = i7.lines.get(i7.ydisp + r5), t6 = [], n5 = e7?.translateToString(true, void 0, void 0, t6) || "", o6 = (i7.ydisp + r5 + 1).toString(), a4 = this._rowElements[r5];
              a4 && (0 === n5.length ? (a4.innerText = "\xA0", this._rowColumns.set(a4, [0, 1])) : (a4.textContent = n5, this._rowColumns.set(a4, t6)), a4.setAttribute("aria-posinset", o6), a4.setAttribute("aria-setsize", s6));
            }
            this._announceCharacters();
          }
          _announceCharacters() {
            0 !== this._charsToAnnounce.length && (this._liveRegion.textContent += this._charsToAnnounce, this._charsToAnnounce = "");
          }
          _handleBoundaryFocus(e6, t5) {
            const i7 = e6.target, s6 = this._rowElements[0 === t5 ? 1 : this._rowElements.length - 2];
            if (i7.getAttribute("aria-posinset") === (0 === t5 ? "1" : `${this._terminal.buffer.lines.length}`)) return;
            if (e6.relatedTarget !== s6) return;
            let r5, n5;
            if (0 === t5 ? (r5 = i7, n5 = this._rowElements.pop(), this._rowContainer.removeChild(n5)) : (r5 = this._rowElements.shift(), n5 = i7, this._rowContainer.removeChild(r5)), r5.removeEventListener("focus", this._topBoundaryFocusListener), n5.removeEventListener("focus", this._bottomBoundaryFocusListener), 0 === t5) {
              const e7 = this._createAccessibilityTreeNode();
              this._rowElements.unshift(e7), this._rowContainer.insertAdjacentElement("afterbegin", e7);
            } else {
              const e7 = this._createAccessibilityTreeNode();
              this._rowElements.push(e7), this._rowContainer.appendChild(e7);
            }
            this._rowElements[0].addEventListener("focus", this._topBoundaryFocusListener), this._rowElements[this._rowElements.length - 1].addEventListener("focus", this._bottomBoundaryFocusListener), this._terminal.scrollLines(0 === t5 ? -1 : 1), this._rowElements[0 === t5 ? 1 : this._rowElements.length - 2].focus(), e6.preventDefault(), e6.stopImmediatePropagation();
          }
          _handleSelectionChange() {
            if (0 === this._rowElements.length) return;
            const e6 = document.getSelection();
            if (!e6) return;
            if (e6.isCollapsed) return void (this._rowContainer.contains(e6.anchorNode) && this._terminal.clearSelection());
            if (!e6.anchorNode || !e6.focusNode) return void console.error("anchorNode and/or focusNode are null");
            let t5 = { node: e6.anchorNode, offset: e6.anchorOffset }, i7 = { node: e6.focusNode, offset: e6.focusOffset };
            if ((t5.node.compareDocumentPosition(i7.node) & Node.DOCUMENT_POSITION_PRECEDING || t5.node === i7.node && t5.offset > i7.offset) && ([t5, i7] = [i7, t5]), t5.node.compareDocumentPosition(this._rowElements[0]) & (Node.DOCUMENT_POSITION_CONTAINED_BY | Node.DOCUMENT_POSITION_FOLLOWING) && (t5 = { node: this._rowElements[0].childNodes[0], offset: 0 }), !this._rowContainer.contains(t5.node)) return;
            const s6 = this._rowElements.slice(-1)[0];
            if (i7.node.compareDocumentPosition(s6) & (Node.DOCUMENT_POSITION_CONTAINED_BY | Node.DOCUMENT_POSITION_PRECEDING) && (i7 = { node: s6, offset: s6.textContent?.length ?? 0 }), !this._rowContainer.contains(i7.node)) return;
            const r5 = ({ node: e7, offset: t6 }) => {
              const i8 = e7 instanceof Text ? e7.parentNode : e7;
              let s7 = parseInt(i8?.getAttribute("aria-posinset"), 10) - 1;
              if (isNaN(s7)) return console.warn("row is invalid. Race condition?"), null;
              const r6 = this._rowColumns.get(i8);
              if (!r6) return console.warn("columns is null. Race condition?"), null;
              let n6 = t6 < r6.length ? r6[t6] : r6.slice(-1)[0] + 1;
              return n6 >= this._terminal.cols && (++s7, n6 = 0), { row: s7, column: n6 };
            }, n5 = r5(t5), o6 = r5(i7);
            if (n5 && o6) {
              if (n5.row > o6.row || n5.row === o6.row && n5.column >= o6.column) throw new Error("invalid range");
              this._terminal.select(n5.column, n5.row, (o6.row - n5.row) * this._terminal.cols - n5.column + o6.column);
            }
          }
          _handleResize(e6) {
            this._rowElements[this._rowElements.length - 1].removeEventListener("focus", this._bottomBoundaryFocusListener);
            for (let e7 = this._rowContainer.children.length; e7 < this._terminal.rows; e7++) this._rowElements[e7] = this._createAccessibilityTreeNode(), this._rowContainer.appendChild(this._rowElements[e7]);
            for (; this._rowElements.length > e6; ) this._rowContainer.removeChild(this._rowElements.pop());
            this._rowElements[this._rowElements.length - 1].addEventListener("focus", this._bottomBoundaryFocusListener), this._refreshRowsDimensions();
          }
          _createAccessibilityTreeNode() {
            const e6 = this._coreBrowserService.mainDocument.createElement("div");
            return e6.setAttribute("role", "listitem"), e6.tabIndex = -1, this._refreshRowDimensions(e6), e6;
          }
          _refreshRowsDimensions() {
            if (this._renderService.dimensions.css.cell.height) {
              this._accessibilityContainer.style.width = `${this._renderService.dimensions.css.canvas.width}px`, this._rowElements.length !== this._terminal.rows && this._handleResize(this._terminal.rows);
              for (let e6 = 0; e6 < this._terminal.rows; e6++) this._refreshRowDimensions(this._rowElements[e6]);
            }
          }
          _refreshRowDimensions(e6) {
            e6.style.height = `${this._renderService.dimensions.css.cell.height}px`;
          }
        };
        t4.AccessibilityManager = d3 = s5([r4(1, c4.IInstantiationService), r4(2, h3.ICoreBrowserService), r4(3, h3.IRenderService)], d3);
      }, 3614: (e5, t4) => {
        function i6(e6) {
          return e6.replace(/\r?\n/g, "\r");
        }
        function s5(e6, t5) {
          return t5 ? "\x1B[200~" + e6 + "\x1B[201~" : e6;
        }
        function r4(e6, t5, r5, n5) {
          e6 = s5(e6 = i6(e6), r5.decPrivateModes.bracketedPasteMode && true !== n5.rawOptions.ignoreBracketedPasteMode), r5.triggerDataEvent(e6, true), t5.value = "";
        }
        function n4(e6, t5, i7) {
          const s6 = i7.getBoundingClientRect(), r5 = e6.clientX - s6.left - 10, n5 = e6.clientY - s6.top - 10;
          t5.style.width = "20px", t5.style.height = "20px", t5.style.left = `${r5}px`, t5.style.top = `${n5}px`, t5.style.zIndex = "1000", t5.focus();
        }
        Object.defineProperty(t4, "__esModule", { value: true }), t4.rightClickHandler = t4.moveTextAreaUnderMouseCursor = t4.paste = t4.handlePasteEvent = t4.copyHandler = t4.bracketTextForPaste = t4.prepareTextForTerminal = void 0, t4.prepareTextForTerminal = i6, t4.bracketTextForPaste = s5, t4.copyHandler = function(e6, t5) {
          e6.clipboardData && e6.clipboardData.setData("text/plain", t5.selectionText), e6.preventDefault();
        }, t4.handlePasteEvent = function(e6, t5, i7, s6) {
          e6.stopPropagation(), e6.clipboardData && r4(e6.clipboardData.getData("text/plain"), t5, i7, s6);
        }, t4.paste = r4, t4.moveTextAreaUnderMouseCursor = n4, t4.rightClickHandler = function(e6, t5, i7, s6, r5) {
          n4(e6, t5, i7), r5 && s6.rightClickSelect(e6), t5.value = s6.selectionText, t5.select();
        };
      }, 7239: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.ColorContrastCache = void 0;
        const s5 = i6(1505);
        t4.ColorContrastCache = class {
          constructor() {
            this._color = new s5.TwoKeyMap(), this._css = new s5.TwoKeyMap();
          }
          setCss(e6, t5, i7) {
            this._css.set(e6, t5, i7);
          }
          getCss(e6, t5) {
            return this._css.get(e6, t5);
          }
          setColor(e6, t5, i7) {
            this._color.set(e6, t5, i7);
          }
          getColor(e6, t5) {
            return this._color.get(e6, t5);
          }
          clear() {
            this._color.clear(), this._css.clear();
          }
        };
      }, 3656: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.addDisposableDomListener = void 0, t4.addDisposableDomListener = function(e6, t5, i6, s5) {
          e6.addEventListener(t5, i6, s5);
          let r4 = false;
          return { dispose: () => {
            r4 || (r4 = true, e6.removeEventListener(t5, i6, s5));
          } };
        };
      }, 3551: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.Linkifier = void 0;
        const n4 = i6(3656), o5 = i6(8460), a3 = i6(844), h3 = i6(2585), c4 = i6(4725);
        let l3 = t4.Linkifier = class extends a3.Disposable {
          get currentLink() {
            return this._currentLink;
          }
          constructor(e6, t5, i7, s6, r5) {
            super(), this._element = e6, this._mouseService = t5, this._renderService = i7, this._bufferService = s6, this._linkProviderService = r5, this._linkCacheDisposables = [], this._isMouseOut = true, this._wasResized = false, this._activeLine = -1, this._onShowLinkUnderline = this.register(new o5.EventEmitter()), this.onShowLinkUnderline = this._onShowLinkUnderline.event, this._onHideLinkUnderline = this.register(new o5.EventEmitter()), this.onHideLinkUnderline = this._onHideLinkUnderline.event, this.register((0, a3.getDisposeArrayDisposable)(this._linkCacheDisposables)), this.register((0, a3.toDisposable)((() => {
              this._lastMouseEvent = void 0, this._activeProviderReplies?.clear();
            }))), this.register(this._bufferService.onResize((() => {
              this._clearCurrentLink(), this._wasResized = true;
            }))), this.register((0, n4.addDisposableDomListener)(this._element, "mouseleave", (() => {
              this._isMouseOut = true, this._clearCurrentLink();
            }))), this.register((0, n4.addDisposableDomListener)(this._element, "mousemove", this._handleMouseMove.bind(this))), this.register((0, n4.addDisposableDomListener)(this._element, "mousedown", this._handleMouseDown.bind(this))), this.register((0, n4.addDisposableDomListener)(this._element, "mouseup", this._handleMouseUp.bind(this)));
          }
          _handleMouseMove(e6) {
            this._lastMouseEvent = e6;
            const t5 = this._positionFromMouseEvent(e6, this._element, this._mouseService);
            if (!t5) return;
            this._isMouseOut = false;
            const i7 = e6.composedPath();
            for (let e7 = 0; e7 < i7.length; e7++) {
              const t6 = i7[e7];
              if (t6.classList.contains("xterm")) break;
              if (t6.classList.contains("xterm-hover")) return;
            }
            this._lastBufferCell && t5.x === this._lastBufferCell.x && t5.y === this._lastBufferCell.y || (this._handleHover(t5), this._lastBufferCell = t5);
          }
          _handleHover(e6) {
            if (this._activeLine !== e6.y || this._wasResized) return this._clearCurrentLink(), this._askForLink(e6, false), void (this._wasResized = false);
            this._currentLink && this._linkAtPosition(this._currentLink.link, e6) || (this._clearCurrentLink(), this._askForLink(e6, true));
          }
          _askForLink(e6, t5) {
            this._activeProviderReplies && t5 || (this._activeProviderReplies?.forEach(((e7) => {
              e7?.forEach(((e8) => {
                e8.link.dispose && e8.link.dispose();
              }));
            })), this._activeProviderReplies = /* @__PURE__ */ new Map(), this._activeLine = e6.y);
            let i7 = false;
            for (const [s6, r5] of this._linkProviderService.linkProviders.entries()) if (t5) {
              const t6 = this._activeProviderReplies?.get(s6);
              t6 && (i7 = this._checkLinkProviderResult(s6, e6, i7));
            } else r5.provideLinks(e6.y, ((t6) => {
              if (this._isMouseOut) return;
              const r6 = t6?.map(((e7) => ({ link: e7 })));
              this._activeProviderReplies?.set(s6, r6), i7 = this._checkLinkProviderResult(s6, e6, i7), this._activeProviderReplies?.size === this._linkProviderService.linkProviders.length && this._removeIntersectingLinks(e6.y, this._activeProviderReplies);
            }));
          }
          _removeIntersectingLinks(e6, t5) {
            const i7 = /* @__PURE__ */ new Set();
            for (let s6 = 0; s6 < t5.size; s6++) {
              const r5 = t5.get(s6);
              if (r5) for (let t6 = 0; t6 < r5.length; t6++) {
                const s7 = r5[t6], n5 = s7.link.range.start.y < e6 ? 0 : s7.link.range.start.x, o6 = s7.link.range.end.y > e6 ? this._bufferService.cols : s7.link.range.end.x;
                for (let e7 = n5; e7 <= o6; e7++) {
                  if (i7.has(e7)) {
                    r5.splice(t6--, 1);
                    break;
                  }
                  i7.add(e7);
                }
              }
            }
          }
          _checkLinkProviderResult(e6, t5, i7) {
            if (!this._activeProviderReplies) return i7;
            const s6 = this._activeProviderReplies.get(e6);
            let r5 = false;
            for (let t6 = 0; t6 < e6; t6++) this._activeProviderReplies.has(t6) && !this._activeProviderReplies.get(t6) || (r5 = true);
            if (!r5 && s6) {
              const e7 = s6.find(((e8) => this._linkAtPosition(e8.link, t5)));
              e7 && (i7 = true, this._handleNewLink(e7));
            }
            if (this._activeProviderReplies.size === this._linkProviderService.linkProviders.length && !i7) for (let e7 = 0; e7 < this._activeProviderReplies.size; e7++) {
              const s7 = this._activeProviderReplies.get(e7)?.find(((e8) => this._linkAtPosition(e8.link, t5)));
              if (s7) {
                i7 = true, this._handleNewLink(s7);
                break;
              }
            }
            return i7;
          }
          _handleMouseDown() {
            this._mouseDownLink = this._currentLink;
          }
          _handleMouseUp(e6) {
            if (!this._currentLink) return;
            const t5 = this._positionFromMouseEvent(e6, this._element, this._mouseService);
            t5 && this._mouseDownLink === this._currentLink && this._linkAtPosition(this._currentLink.link, t5) && this._currentLink.link.activate(e6, this._currentLink.link.text);
          }
          _clearCurrentLink(e6, t5) {
            this._currentLink && this._lastMouseEvent && (!e6 || !t5 || this._currentLink.link.range.start.y >= e6 && this._currentLink.link.range.end.y <= t5) && (this._linkLeave(this._element, this._currentLink.link, this._lastMouseEvent), this._currentLink = void 0, (0, a3.disposeArray)(this._linkCacheDisposables));
          }
          _handleNewLink(e6) {
            if (!this._lastMouseEvent) return;
            const t5 = this._positionFromMouseEvent(this._lastMouseEvent, this._element, this._mouseService);
            t5 && this._linkAtPosition(e6.link, t5) && (this._currentLink = e6, this._currentLink.state = { decorations: { underline: void 0 === e6.link.decorations || e6.link.decorations.underline, pointerCursor: void 0 === e6.link.decorations || e6.link.decorations.pointerCursor }, isHovered: true }, this._linkHover(this._element, e6.link, this._lastMouseEvent), e6.link.decorations = {}, Object.defineProperties(e6.link.decorations, { pointerCursor: { get: () => this._currentLink?.state?.decorations.pointerCursor, set: (e7) => {
              this._currentLink?.state && this._currentLink.state.decorations.pointerCursor !== e7 && (this._currentLink.state.decorations.pointerCursor = e7, this._currentLink.state.isHovered && this._element.classList.toggle("xterm-cursor-pointer", e7));
            } }, underline: { get: () => this._currentLink?.state?.decorations.underline, set: (t6) => {
              this._currentLink?.state && this._currentLink?.state?.decorations.underline !== t6 && (this._currentLink.state.decorations.underline = t6, this._currentLink.state.isHovered && this._fireUnderlineEvent(e6.link, t6));
            } } }), this._linkCacheDisposables.push(this._renderService.onRenderedViewportChange(((e7) => {
              if (!this._currentLink) return;
              const t6 = 0 === e7.start ? 0 : e7.start + 1 + this._bufferService.buffer.ydisp, i7 = this._bufferService.buffer.ydisp + 1 + e7.end;
              if (this._currentLink.link.range.start.y >= t6 && this._currentLink.link.range.end.y <= i7 && (this._clearCurrentLink(t6, i7), this._lastMouseEvent)) {
                const e8 = this._positionFromMouseEvent(this._lastMouseEvent, this._element, this._mouseService);
                e8 && this._askForLink(e8, false);
              }
            }))));
          }
          _linkHover(e6, t5, i7) {
            this._currentLink?.state && (this._currentLink.state.isHovered = true, this._currentLink.state.decorations.underline && this._fireUnderlineEvent(t5, true), this._currentLink.state.decorations.pointerCursor && e6.classList.add("xterm-cursor-pointer")), t5.hover && t5.hover(i7, t5.text);
          }
          _fireUnderlineEvent(e6, t5) {
            const i7 = e6.range, s6 = this._bufferService.buffer.ydisp, r5 = this._createLinkUnderlineEvent(i7.start.x - 1, i7.start.y - s6 - 1, i7.end.x, i7.end.y - s6 - 1, void 0);
            (t5 ? this._onShowLinkUnderline : this._onHideLinkUnderline).fire(r5);
          }
          _linkLeave(e6, t5, i7) {
            this._currentLink?.state && (this._currentLink.state.isHovered = false, this._currentLink.state.decorations.underline && this._fireUnderlineEvent(t5, false), this._currentLink.state.decorations.pointerCursor && e6.classList.remove("xterm-cursor-pointer")), t5.leave && t5.leave(i7, t5.text);
          }
          _linkAtPosition(e6, t5) {
            const i7 = e6.range.start.y * this._bufferService.cols + e6.range.start.x, s6 = e6.range.end.y * this._bufferService.cols + e6.range.end.x, r5 = t5.y * this._bufferService.cols + t5.x;
            return i7 <= r5 && r5 <= s6;
          }
          _positionFromMouseEvent(e6, t5, i7) {
            const s6 = i7.getCoords(e6, t5, this._bufferService.cols, this._bufferService.rows);
            if (s6) return { x: s6[0], y: s6[1] + this._bufferService.buffer.ydisp };
          }
          _createLinkUnderlineEvent(e6, t5, i7, s6, r5) {
            return { x1: e6, y1: t5, x2: i7, y2: s6, cols: this._bufferService.cols, fg: r5 };
          }
        };
        t4.Linkifier = l3 = s5([r4(1, c4.IMouseService), r4(2, c4.IRenderService), r4(3, h3.IBufferService), r4(4, c4.ILinkProviderService)], l3);
      }, 9042: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.tooMuchOutput = t4.promptLabel = void 0, t4.promptLabel = "Terminal input", t4.tooMuchOutput = "Too much output to announce, navigate to rows manually to read";
      }, 3730: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.OscLinkProvider = void 0;
        const n4 = i6(511), o5 = i6(2585);
        let a3 = t4.OscLinkProvider = class {
          constructor(e6, t5, i7) {
            this._bufferService = e6, this._optionsService = t5, this._oscLinkService = i7;
          }
          provideLinks(e6, t5) {
            const i7 = this._bufferService.buffer.lines.get(e6 - 1);
            if (!i7) return void t5(void 0);
            const s6 = [], r5 = this._optionsService.rawOptions.linkHandler, o6 = new n4.CellData(), a4 = i7.getTrimmedLength();
            let c4 = -1, l3 = -1, d3 = false;
            for (let t6 = 0; t6 < a4; t6++) if (-1 !== l3 || i7.hasContent(t6)) {
              if (i7.loadCell(t6, o6), o6.hasExtendedAttrs() && o6.extended.urlId) {
                if (-1 === l3) {
                  l3 = t6, c4 = o6.extended.urlId;
                  continue;
                }
                d3 = o6.extended.urlId !== c4;
              } else -1 !== l3 && (d3 = true);
              if (d3 || -1 !== l3 && t6 === a4 - 1) {
                const i8 = this._oscLinkService.getLinkData(c4)?.uri;
                if (i8) {
                  const n5 = { start: { x: l3 + 1, y: e6 }, end: { x: t6 + (d3 || t6 !== a4 - 1 ? 0 : 1), y: e6 } };
                  let o7 = false;
                  if (!r5?.allowNonHttpProtocols) try {
                    const e7 = new URL(i8);
                    ["http:", "https:"].includes(e7.protocol) || (o7 = true);
                  } catch (e7) {
                    o7 = true;
                  }
                  o7 || s6.push({ text: i8, range: n5, activate: (e7, t7) => r5 ? r5.activate(e7, t7, n5) : h3(0, t7), hover: (e7, t7) => r5?.hover?.(e7, t7, n5), leave: (e7, t7) => r5?.leave?.(e7, t7, n5) });
                }
                d3 = false, o6.hasExtendedAttrs() && o6.extended.urlId ? (l3 = t6, c4 = o6.extended.urlId) : (l3 = -1, c4 = -1);
              }
            }
            t5(s6);
          }
        };
        function h3(e6, t5) {
          if (confirm(`Do you want to navigate to ${t5}?

WARNING: This link could potentially be dangerous`)) {
            const e7 = window.open();
            if (e7) {
              try {
                e7.opener = null;
              } catch {
              }
              e7.location.href = t5;
            } else console.warn("Opening link blocked as opener could not be cleared");
          }
        }
        t4.OscLinkProvider = a3 = s5([r4(0, o5.IBufferService), r4(1, o5.IOptionsService), r4(2, o5.IOscLinkService)], a3);
      }, 6193: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.RenderDebouncer = void 0, t4.RenderDebouncer = class {
          constructor(e6, t5) {
            this._renderCallback = e6, this._coreBrowserService = t5, this._refreshCallbacks = [];
          }
          dispose() {
            this._animationFrame && (this._coreBrowserService.window.cancelAnimationFrame(this._animationFrame), this._animationFrame = void 0);
          }
          addRefreshCallback(e6) {
            return this._refreshCallbacks.push(e6), this._animationFrame || (this._animationFrame = this._coreBrowserService.window.requestAnimationFrame((() => this._innerRefresh()))), this._animationFrame;
          }
          refresh(e6, t5, i6) {
            this._rowCount = i6, e6 = void 0 !== e6 ? e6 : 0, t5 = void 0 !== t5 ? t5 : this._rowCount - 1, this._rowStart = void 0 !== this._rowStart ? Math.min(this._rowStart, e6) : e6, this._rowEnd = void 0 !== this._rowEnd ? Math.max(this._rowEnd, t5) : t5, this._animationFrame || (this._animationFrame = this._coreBrowserService.window.requestAnimationFrame((() => this._innerRefresh())));
          }
          _innerRefresh() {
            if (this._animationFrame = void 0, void 0 === this._rowStart || void 0 === this._rowEnd || void 0 === this._rowCount) return void this._runRefreshCallbacks();
            const e6 = Math.max(this._rowStart, 0), t5 = Math.min(this._rowEnd, this._rowCount - 1);
            this._rowStart = void 0, this._rowEnd = void 0, this._renderCallback(e6, t5), this._runRefreshCallbacks();
          }
          _runRefreshCallbacks() {
            for (const e6 of this._refreshCallbacks) e6(0);
            this._refreshCallbacks = [];
          }
        };
      }, 3236: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.Terminal = void 0;
        const s5 = i6(3614), r4 = i6(3656), n4 = i6(3551), o5 = i6(9042), a3 = i6(3730), h3 = i6(1680), c4 = i6(3107), l3 = i6(5744), d3 = i6(2950), _2 = i6(1296), u3 = i6(428), f3 = i6(4269), v2 = i6(5114), p3 = i6(8934), g2 = i6(3230), m2 = i6(9312), S3 = i6(4725), C2 = i6(6731), b3 = i6(8055), w2 = i6(8969), y3 = i6(8460), E2 = i6(844), k2 = i6(6114), L2 = i6(8437), D2 = i6(2584), R2 = i6(7399), x2 = i6(5941), A2 = i6(9074), B2 = i6(2585), T2 = i6(5435), M2 = i6(4567), O = i6(779);
        class P2 extends w2.CoreTerminal {
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
          constructor(e6 = {}) {
            super(e6), this.browser = k2, this._keyDownHandled = false, this._keyDownSeen = false, this._keyPressHandled = false, this._unprocessedDeadKey = false, this._accessibilityManager = this.register(new E2.MutableDisposable()), this._onCursorMove = this.register(new y3.EventEmitter()), this.onCursorMove = this._onCursorMove.event, this._onKey = this.register(new y3.EventEmitter()), this.onKey = this._onKey.event, this._onRender = this.register(new y3.EventEmitter()), this.onRender = this._onRender.event, this._onSelectionChange = this.register(new y3.EventEmitter()), this.onSelectionChange = this._onSelectionChange.event, this._onTitleChange = this.register(new y3.EventEmitter()), this.onTitleChange = this._onTitleChange.event, this._onBell = this.register(new y3.EventEmitter()), this.onBell = this._onBell.event, this._onFocus = this.register(new y3.EventEmitter()), this._onBlur = this.register(new y3.EventEmitter()), this._onA11yCharEmitter = this.register(new y3.EventEmitter()), this._onA11yTabEmitter = this.register(new y3.EventEmitter()), this._onWillOpen = this.register(new y3.EventEmitter()), this._setup(), this._decorationService = this._instantiationService.createInstance(A2.DecorationService), this._instantiationService.setService(B2.IDecorationService, this._decorationService), this._linkProviderService = this._instantiationService.createInstance(O.LinkProviderService), this._instantiationService.setService(S3.ILinkProviderService, this._linkProviderService), this._linkProviderService.registerLinkProvider(this._instantiationService.createInstance(a3.OscLinkProvider)), this.register(this._inputHandler.onRequestBell((() => this._onBell.fire()))), this.register(this._inputHandler.onRequestRefreshRows(((e7, t5) => this.refresh(e7, t5)))), this.register(this._inputHandler.onRequestSendFocus((() => this._reportFocus()))), this.register(this._inputHandler.onRequestReset((() => this.reset()))), this.register(this._inputHandler.onRequestWindowsOptionsReport(((e7) => this._reportWindowsOptions(e7)))), this.register(this._inputHandler.onColor(((e7) => this._handleColorEvent(e7)))), this.register((0, y3.forwardEvent)(this._inputHandler.onCursorMove, this._onCursorMove)), this.register((0, y3.forwardEvent)(this._inputHandler.onTitleChange, this._onTitleChange)), this.register((0, y3.forwardEvent)(this._inputHandler.onA11yChar, this._onA11yCharEmitter)), this.register((0, y3.forwardEvent)(this._inputHandler.onA11yTab, this._onA11yTabEmitter)), this.register(this._bufferService.onResize(((e7) => this._afterResize(e7.cols, e7.rows)))), this.register((0, E2.toDisposable)((() => {
              this._customKeyEventHandler = void 0, this.element?.parentNode?.removeChild(this.element);
            })));
          }
          _handleColorEvent(e6) {
            if (this._themeService) for (const t5 of e6) {
              let e7, i7 = "";
              switch (t5.index) {
                case 256:
                  e7 = "foreground", i7 = "10";
                  break;
                case 257:
                  e7 = "background", i7 = "11";
                  break;
                case 258:
                  e7 = "cursor", i7 = "12";
                  break;
                default:
                  e7 = "ansi", i7 = "4;" + t5.index;
              }
              switch (t5.type) {
                case 0:
                  const s6 = b3.color.toColorRGB("ansi" === e7 ? this._themeService.colors.ansi[t5.index] : this._themeService.colors[e7]);
                  this.coreService.triggerDataEvent(`${D2.C0.ESC}]${i7};${(0, x2.toRgbString)(s6)}${D2.C1_ESCAPED.ST}`);
                  break;
                case 1:
                  if ("ansi" === e7) this._themeService.modifyColors(((e8) => e8.ansi[t5.index] = b3.channels.toColor(...t5.color)));
                  else {
                    const i8 = e7;
                    this._themeService.modifyColors(((e8) => e8[i8] = b3.channels.toColor(...t5.color)));
                  }
                  break;
                case 2:
                  this._themeService.restoreColor(t5.index);
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
          _handleScreenReaderModeOptionChange(e6) {
            e6 ? !this._accessibilityManager.value && this._renderService && (this._accessibilityManager.value = this._instantiationService.createInstance(M2.AccessibilityManager, this)) : this._accessibilityManager.clear();
          }
          _handleTextAreaFocus(e6) {
            this.coreService.decPrivateModes.sendFocus && this.coreService.triggerDataEvent(D2.C0.ESC + "[I"), this.element.classList.add("focus"), this._showCursor(), this._onFocus.fire();
          }
          blur() {
            return this.textarea?.blur();
          }
          _handleTextAreaBlur() {
            this.textarea.value = "", this.refresh(this.buffer.y, this.buffer.y), this.coreService.decPrivateModes.sendFocus && this.coreService.triggerDataEvent(D2.C0.ESC + "[O"), this.element.classList.remove("focus"), this._onBlur.fire();
          }
          _syncTextArea() {
            if (!this.textarea || !this.buffer.isCursorInViewport || this._compositionHelper.isComposing || !this._renderService) return;
            const e6 = this.buffer.ybase + this.buffer.y, t5 = this.buffer.lines.get(e6);
            if (!t5) return;
            const i7 = Math.min(this.buffer.x, this.cols - 1), s6 = this._renderService.dimensions.css.cell.height, r5 = t5.getWidth(i7), n5 = this._renderService.dimensions.css.cell.width * r5, o6 = this.buffer.y * this._renderService.dimensions.css.cell.height, a4 = i7 * this._renderService.dimensions.css.cell.width;
            this.textarea.style.left = a4 + "px", this.textarea.style.top = o6 + "px", this.textarea.style.width = n5 + "px", this.textarea.style.height = s6 + "px", this.textarea.style.lineHeight = s6 + "px", this.textarea.style.zIndex = "-5";
          }
          _initGlobal() {
            this._bindKeys(), this.register((0, r4.addDisposableDomListener)(this.element, "copy", ((e7) => {
              this.hasSelection() && (0, s5.copyHandler)(e7, this._selectionService);
            })));
            const e6 = (e7) => (0, s5.handlePasteEvent)(e7, this.textarea, this.coreService, this.optionsService);
            this.register((0, r4.addDisposableDomListener)(this.textarea, "paste", e6)), this.register((0, r4.addDisposableDomListener)(this.element, "paste", e6)), k2.isFirefox ? this.register((0, r4.addDisposableDomListener)(this.element, "mousedown", ((e7) => {
              2 === e7.button && (0, s5.rightClickHandler)(e7, this.textarea, this.screenElement, this._selectionService, this.options.rightClickSelectsWord);
            }))) : this.register((0, r4.addDisposableDomListener)(this.element, "contextmenu", ((e7) => {
              (0, s5.rightClickHandler)(e7, this.textarea, this.screenElement, this._selectionService, this.options.rightClickSelectsWord);
            }))), k2.isLinux && this.register((0, r4.addDisposableDomListener)(this.element, "auxclick", ((e7) => {
              1 === e7.button && (0, s5.moveTextAreaUnderMouseCursor)(e7, this.textarea, this.screenElement);
            })));
          }
          _bindKeys() {
            this.register((0, r4.addDisposableDomListener)(this.textarea, "keyup", ((e6) => this._keyUp(e6)), true)), this.register((0, r4.addDisposableDomListener)(this.textarea, "keydown", ((e6) => this._keyDown(e6)), true)), this.register((0, r4.addDisposableDomListener)(this.textarea, "keypress", ((e6) => this._keyPress(e6)), true)), this.register((0, r4.addDisposableDomListener)(this.textarea, "compositionstart", (() => this._compositionHelper.compositionstart()))), this.register((0, r4.addDisposableDomListener)(this.textarea, "compositionupdate", ((e6) => this._compositionHelper.compositionupdate(e6)))), this.register((0, r4.addDisposableDomListener)(this.textarea, "compositionend", (() => this._compositionHelper.compositionend()))), this.register((0, r4.addDisposableDomListener)(this.textarea, "input", ((e6) => this._inputEvent(e6)), true)), this.register(this.onRender((() => this._compositionHelper.updateCompositionElements())));
          }
          open(e6) {
            if (!e6) throw new Error("Terminal requires a parent element.");
            if (e6.isConnected || this._logService.debug("Terminal.open was called on an element that was not attached to the DOM"), this.element?.ownerDocument.defaultView && this._coreBrowserService) return void (this.element.ownerDocument.defaultView !== this._coreBrowserService.window && (this._coreBrowserService.window = this.element.ownerDocument.defaultView));
            this._document = e6.ownerDocument, this.options.documentOverride && this.options.documentOverride instanceof Document && (this._document = this.optionsService.rawOptions.documentOverride), this.element = this._document.createElement("div"), this.element.dir = "ltr", this.element.classList.add("terminal"), this.element.classList.add("xterm"), e6.appendChild(this.element);
            const t5 = this._document.createDocumentFragment();
            this._viewportElement = this._document.createElement("div"), this._viewportElement.classList.add("xterm-viewport"), t5.appendChild(this._viewportElement), this._viewportScrollArea = this._document.createElement("div"), this._viewportScrollArea.classList.add("xterm-scroll-area"), this._viewportElement.appendChild(this._viewportScrollArea), this.screenElement = this._document.createElement("div"), this.screenElement.classList.add("xterm-screen"), this.register((0, r4.addDisposableDomListener)(this.screenElement, "mousemove", ((e7) => this.updateCursorStyle(e7)))), this._helperContainer = this._document.createElement("div"), this._helperContainer.classList.add("xterm-helpers"), this.screenElement.appendChild(this._helperContainer), t5.appendChild(this.screenElement), this.textarea = this._document.createElement("textarea"), this.textarea.classList.add("xterm-helper-textarea"), this.textarea.setAttribute("aria-label", o5.promptLabel), k2.isChromeOS || this.textarea.setAttribute("aria-multiline", "false"), this.textarea.setAttribute("autocorrect", "off"), this.textarea.setAttribute("autocapitalize", "off"), this.textarea.setAttribute("spellcheck", "false"), this.textarea.tabIndex = 0, this._coreBrowserService = this.register(this._instantiationService.createInstance(v2.CoreBrowserService, this.textarea, e6.ownerDocument.defaultView ?? window, this._document ?? "undefined" != typeof window ? window.document : null)), this._instantiationService.setService(S3.ICoreBrowserService, this._coreBrowserService), this.register((0, r4.addDisposableDomListener)(this.textarea, "focus", ((e7) => this._handleTextAreaFocus(e7)))), this.register((0, r4.addDisposableDomListener)(this.textarea, "blur", (() => this._handleTextAreaBlur()))), this._helperContainer.appendChild(this.textarea), this._charSizeService = this._instantiationService.createInstance(u3.CharSizeService, this._document, this._helperContainer), this._instantiationService.setService(S3.ICharSizeService, this._charSizeService), this._themeService = this._instantiationService.createInstance(C2.ThemeService), this._instantiationService.setService(S3.IThemeService, this._themeService), this._characterJoinerService = this._instantiationService.createInstance(f3.CharacterJoinerService), this._instantiationService.setService(S3.ICharacterJoinerService, this._characterJoinerService), this._renderService = this.register(this._instantiationService.createInstance(g2.RenderService, this.rows, this.screenElement)), this._instantiationService.setService(S3.IRenderService, this._renderService), this.register(this._renderService.onRenderedViewportChange(((e7) => this._onRender.fire(e7)))), this.onResize(((e7) => this._renderService.resize(e7.cols, e7.rows))), this._compositionView = this._document.createElement("div"), this._compositionView.classList.add("composition-view"), this._compositionHelper = this._instantiationService.createInstance(d3.CompositionHelper, this.textarea, this._compositionView), this._helperContainer.appendChild(this._compositionView), this._mouseService = this._instantiationService.createInstance(p3.MouseService), this._instantiationService.setService(S3.IMouseService, this._mouseService), this.linkifier = this.register(this._instantiationService.createInstance(n4.Linkifier, this.screenElement)), this.element.appendChild(t5);
            try {
              this._onWillOpen.fire(this.element);
            } catch {
            }
            this._renderService.hasRenderer() || this._renderService.setRenderer(this._createRenderer()), this.viewport = this._instantiationService.createInstance(h3.Viewport, this._viewportElement, this._viewportScrollArea), this.viewport.onRequestScrollLines(((e7) => this.scrollLines(e7.amount, e7.suppressScrollEvent, 1))), this.register(this._inputHandler.onRequestSyncScrollBar((() => this.viewport.syncScrollArea()))), this.register(this.viewport), this.register(this.onCursorMove((() => {
              this._renderService.handleCursorMove(), this._syncTextArea();
            }))), this.register(this.onResize((() => this._renderService.handleResize(this.cols, this.rows)))), this.register(this.onBlur((() => this._renderService.handleBlur()))), this.register(this.onFocus((() => this._renderService.handleFocus()))), this.register(this._renderService.onDimensionsChange((() => this.viewport.syncScrollArea()))), this._selectionService = this.register(this._instantiationService.createInstance(m2.SelectionService, this.element, this.screenElement, this.linkifier)), this._instantiationService.setService(S3.ISelectionService, this._selectionService), this.register(this._selectionService.onRequestScrollLines(((e7) => this.scrollLines(e7.amount, e7.suppressScrollEvent)))), this.register(this._selectionService.onSelectionChange((() => this._onSelectionChange.fire()))), this.register(this._selectionService.onRequestRedraw(((e7) => this._renderService.handleSelectionChanged(e7.start, e7.end, e7.columnSelectMode)))), this.register(this._selectionService.onLinuxMouseSelection(((e7) => {
              this.textarea.value = e7, this.textarea.focus(), this.textarea.select();
            }))), this.register(this._onScroll.event(((e7) => {
              this.viewport.syncScrollArea(), this._selectionService.refresh();
            }))), this.register((0, r4.addDisposableDomListener)(this._viewportElement, "scroll", (() => this._selectionService.refresh()))), this.register(this._instantiationService.createInstance(c4.BufferDecorationRenderer, this.screenElement)), this.register((0, r4.addDisposableDomListener)(this.element, "mousedown", ((e7) => this._selectionService.handleMouseDown(e7)))), this.coreMouseService.areMouseEventsActive ? (this._selectionService.disable(), this.element.classList.add("enable-mouse-events")) : this._selectionService.enable(), this.options.screenReaderMode && (this._accessibilityManager.value = this._instantiationService.createInstance(M2.AccessibilityManager, this)), this.register(this.optionsService.onSpecificOptionChange("screenReaderMode", ((e7) => this._handleScreenReaderModeOptionChange(e7)))), this.options.overviewRulerWidth && (this._overviewRulerRenderer = this.register(this._instantiationService.createInstance(l3.OverviewRulerRenderer, this._viewportElement, this.screenElement))), this.optionsService.onSpecificOptionChange("overviewRulerWidth", ((e7) => {
              !this._overviewRulerRenderer && e7 && this._viewportElement && this.screenElement && (this._overviewRulerRenderer = this.register(this._instantiationService.createInstance(l3.OverviewRulerRenderer, this._viewportElement, this.screenElement)));
            })), this._charSizeService.measure(), this.refresh(0, this.rows - 1), this._initGlobal(), this.bindMouse();
          }
          _createRenderer() {
            return this._instantiationService.createInstance(_2.DomRenderer, this, this._document, this.element, this.screenElement, this._viewportElement, this._helperContainer, this.linkifier);
          }
          bindMouse() {
            const e6 = this, t5 = this.element;
            function i7(t6) {
              const i8 = e6._mouseService.getMouseReportCoords(t6, e6.screenElement);
              if (!i8) return false;
              let s7, r5;
              switch (t6.overrideType || t6.type) {
                case "mousemove":
                  r5 = 32, void 0 === t6.buttons ? (s7 = 3, void 0 !== t6.button && (s7 = t6.button < 3 ? t6.button : 3)) : s7 = 1 & t6.buttons ? 0 : 4 & t6.buttons ? 1 : 2 & t6.buttons ? 2 : 3;
                  break;
                case "mouseup":
                  r5 = 0, s7 = t6.button < 3 ? t6.button : 3;
                  break;
                case "mousedown":
                  r5 = 1, s7 = t6.button < 3 ? t6.button : 3;
                  break;
                case "wheel":
                  if (e6._customWheelEventHandler && false === e6._customWheelEventHandler(t6)) return false;
                  if (0 === e6.viewport.getLinesScrolled(t6)) return false;
                  r5 = t6.deltaY < 0 ? 0 : 1, s7 = 4;
                  break;
                default:
                  return false;
              }
              return !(void 0 === r5 || void 0 === s7 || s7 > 4) && e6.coreMouseService.triggerMouseEvent({ col: i8.col, row: i8.row, x: i8.x, y: i8.y, button: s7, action: r5, ctrl: t6.ctrlKey, alt: t6.altKey, shift: t6.shiftKey });
            }
            const s6 = { mouseup: null, wheel: null, mousedrag: null, mousemove: null }, n5 = { mouseup: (e7) => (i7(e7), e7.buttons || (this._document.removeEventListener("mouseup", s6.mouseup), s6.mousedrag && this._document.removeEventListener("mousemove", s6.mousedrag)), this.cancel(e7)), wheel: (e7) => (i7(e7), this.cancel(e7, true)), mousedrag: (e7) => {
              e7.buttons && i7(e7);
            }, mousemove: (e7) => {
              e7.buttons || i7(e7);
            } };
            this.register(this.coreMouseService.onProtocolChange(((e7) => {
              e7 ? ("debug" === this.optionsService.rawOptions.logLevel && this._logService.debug("Binding to mouse events:", this.coreMouseService.explainEvents(e7)), this.element.classList.add("enable-mouse-events"), this._selectionService.disable()) : (this._logService.debug("Unbinding from mouse events."), this.element.classList.remove("enable-mouse-events"), this._selectionService.enable()), 8 & e7 ? s6.mousemove || (t5.addEventListener("mousemove", n5.mousemove), s6.mousemove = n5.mousemove) : (t5.removeEventListener("mousemove", s6.mousemove), s6.mousemove = null), 16 & e7 ? s6.wheel || (t5.addEventListener("wheel", n5.wheel, { passive: false }), s6.wheel = n5.wheel) : (t5.removeEventListener("wheel", s6.wheel), s6.wheel = null), 2 & e7 ? s6.mouseup || (s6.mouseup = n5.mouseup) : (this._document.removeEventListener("mouseup", s6.mouseup), s6.mouseup = null), 4 & e7 ? s6.mousedrag || (s6.mousedrag = n5.mousedrag) : (this._document.removeEventListener("mousemove", s6.mousedrag), s6.mousedrag = null);
            }))), this.coreMouseService.activeProtocol = this.coreMouseService.activeProtocol, this.register((0, r4.addDisposableDomListener)(t5, "mousedown", ((e7) => {
              if (e7.preventDefault(), this.focus(), this.coreMouseService.areMouseEventsActive && !this._selectionService.shouldForceSelection(e7)) return i7(e7), s6.mouseup && this._document.addEventListener("mouseup", s6.mouseup), s6.mousedrag && this._document.addEventListener("mousemove", s6.mousedrag), this.cancel(e7);
            }))), this.register((0, r4.addDisposableDomListener)(t5, "wheel", ((e7) => {
              if (!s6.wheel) {
                if (this._customWheelEventHandler && false === this._customWheelEventHandler(e7)) return false;
                if (!this.buffer.hasScrollback) {
                  const t6 = this.viewport.getLinesScrolled(e7);
                  if (0 === t6) return;
                  const i8 = D2.C0.ESC + (this.coreService.decPrivateModes.applicationCursorKeys ? "O" : "[") + (e7.deltaY < 0 ? "A" : "B");
                  let s7 = "";
                  for (let e8 = 0; e8 < Math.abs(t6); e8++) s7 += i8;
                  return this.coreService.triggerDataEvent(s7, true), this.cancel(e7, true);
                }
                return this.viewport.handleWheel(e7) ? this.cancel(e7) : void 0;
              }
            }), { passive: false })), this.register((0, r4.addDisposableDomListener)(t5, "touchstart", ((e7) => {
              if (!this.coreMouseService.areMouseEventsActive) return this.viewport.handleTouchStart(e7), this.cancel(e7);
            }), { passive: true })), this.register((0, r4.addDisposableDomListener)(t5, "touchmove", ((e7) => {
              if (!this.coreMouseService.areMouseEventsActive) return this.viewport.handleTouchMove(e7) ? void 0 : this.cancel(e7);
            }), { passive: false }));
          }
          refresh(e6, t5) {
            this._renderService?.refreshRows(e6, t5);
          }
          updateCursorStyle(e6) {
            this._selectionService?.shouldColumnSelect(e6) ? this.element.classList.add("column-select") : this.element.classList.remove("column-select");
          }
          _showCursor() {
            this.coreService.isCursorInitialized || (this.coreService.isCursorInitialized = true, this.refresh(this.buffer.y, this.buffer.y));
          }
          scrollLines(e6, t5, i7 = 0) {
            1 === i7 ? (super.scrollLines(e6, t5, i7), this.refresh(0, this.rows - 1)) : this.viewport?.scrollLines(e6);
          }
          paste(e6) {
            (0, s5.paste)(e6, this.textarea, this.coreService, this.optionsService);
          }
          attachCustomKeyEventHandler(e6) {
            this._customKeyEventHandler = e6;
          }
          attachCustomWheelEventHandler(e6) {
            this._customWheelEventHandler = e6;
          }
          registerLinkProvider(e6) {
            return this._linkProviderService.registerLinkProvider(e6);
          }
          registerCharacterJoiner(e6) {
            if (!this._characterJoinerService) throw new Error("Terminal must be opened first");
            const t5 = this._characterJoinerService.register(e6);
            return this.refresh(0, this.rows - 1), t5;
          }
          deregisterCharacterJoiner(e6) {
            if (!this._characterJoinerService) throw new Error("Terminal must be opened first");
            this._characterJoinerService.deregister(e6) && this.refresh(0, this.rows - 1);
          }
          get markers() {
            return this.buffer.markers;
          }
          registerMarker(e6) {
            return this.buffer.addMarker(this.buffer.ybase + this.buffer.y + e6);
          }
          registerDecoration(e6) {
            return this._decorationService.registerDecoration(e6);
          }
          hasSelection() {
            return !!this._selectionService && this._selectionService.hasSelection;
          }
          select(e6, t5, i7) {
            this._selectionService.setSelection(e6, t5, i7);
          }
          getSelection() {
            return this._selectionService ? this._selectionService.selectionText : "";
          }
          getSelectionPosition() {
            if (this._selectionService && this._selectionService.hasSelection) return { start: { x: this._selectionService.selectionStart[0], y: this._selectionService.selectionStart[1] }, end: { x: this._selectionService.selectionEnd[0], y: this._selectionService.selectionEnd[1] } };
          }
          clearSelection() {
            this._selectionService?.clearSelection();
          }
          selectAll() {
            this._selectionService?.selectAll();
          }
          selectLines(e6, t5) {
            this._selectionService?.selectLines(e6, t5);
          }
          _keyDown(e6) {
            if (this._keyDownHandled = false, this._keyDownSeen = true, this._customKeyEventHandler && false === this._customKeyEventHandler(e6)) return false;
            const t5 = this.browser.isMac && this.options.macOptionIsMeta && e6.altKey;
            if (!t5 && !this._compositionHelper.keydown(e6)) return this.options.scrollOnUserInput && this.buffer.ybase !== this.buffer.ydisp && this.scrollToBottom(), false;
            t5 || "Dead" !== e6.key && "AltGraph" !== e6.key || (this._unprocessedDeadKey = true);
            const i7 = (0, R2.evaluateKeyboardEvent)(e6, this.coreService.decPrivateModes.applicationCursorKeys, this.browser.isMac, this.options.macOptionIsMeta);
            if (this.updateCursorStyle(e6), 3 === i7.type || 2 === i7.type) {
              const t6 = this.rows - 1;
              return this.scrollLines(2 === i7.type ? -t6 : t6), this.cancel(e6, true);
            }
            return 1 === i7.type && this.selectAll(), !!this._isThirdLevelShift(this.browser, e6) || (i7.cancel && this.cancel(e6, true), !i7.key || !!(e6.key && !e6.ctrlKey && !e6.altKey && !e6.metaKey && 1 === e6.key.length && e6.key.charCodeAt(0) >= 65 && e6.key.charCodeAt(0) <= 90) || (this._unprocessedDeadKey ? (this._unprocessedDeadKey = false, true) : (i7.key !== D2.C0.ETX && i7.key !== D2.C0.CR || (this.textarea.value = ""), this._onKey.fire({ key: i7.key, domEvent: e6 }), this._showCursor(), this.coreService.triggerDataEvent(i7.key, true), !this.optionsService.rawOptions.screenReaderMode || e6.altKey || e6.ctrlKey ? this.cancel(e6, true) : void (this._keyDownHandled = true))));
          }
          _isThirdLevelShift(e6, t5) {
            const i7 = e6.isMac && !this.options.macOptionIsMeta && t5.altKey && !t5.ctrlKey && !t5.metaKey || e6.isWindows && t5.altKey && t5.ctrlKey && !t5.metaKey || e6.isWindows && t5.getModifierState("AltGraph");
            return "keypress" === t5.type ? i7 : i7 && (!t5.keyCode || t5.keyCode > 47);
          }
          _keyUp(e6) {
            this._keyDownSeen = false, this._customKeyEventHandler && false === this._customKeyEventHandler(e6) || ((function(e7) {
              return 16 === e7.keyCode || 17 === e7.keyCode || 18 === e7.keyCode;
            })(e6) || this.focus(), this.updateCursorStyle(e6), this._keyPressHandled = false);
          }
          _keyPress(e6) {
            let t5;
            if (this._keyPressHandled = false, this._keyDownHandled) return false;
            if (this._customKeyEventHandler && false === this._customKeyEventHandler(e6)) return false;
            if (this.cancel(e6), e6.charCode) t5 = e6.charCode;
            else if (null === e6.which || void 0 === e6.which) t5 = e6.keyCode;
            else {
              if (0 === e6.which || 0 === e6.charCode) return false;
              t5 = e6.which;
            }
            return !(!t5 || (e6.altKey || e6.ctrlKey || e6.metaKey) && !this._isThirdLevelShift(this.browser, e6) || (t5 = String.fromCharCode(t5), this._onKey.fire({ key: t5, domEvent: e6 }), this._showCursor(), this.coreService.triggerDataEvent(t5, true), this._keyPressHandled = true, this._unprocessedDeadKey = false, 0));
          }
          _inputEvent(e6) {
            if (e6.data && "insertText" === e6.inputType && (!e6.composed || !this._keyDownSeen) && !this.optionsService.rawOptions.screenReaderMode) {
              if (this._keyPressHandled) return false;
              this._unprocessedDeadKey = false;
              const t5 = e6.data;
              return this.coreService.triggerDataEvent(t5, true), this.cancel(e6), true;
            }
            return false;
          }
          resize(e6, t5) {
            e6 !== this.cols || t5 !== this.rows ? super.resize(e6, t5) : this._charSizeService && !this._charSizeService.hasValidSize && this._charSizeService.measure();
          }
          _afterResize(e6, t5) {
            this._charSizeService?.measure(), this.viewport?.syncScrollArea(true);
          }
          clear() {
            if (0 !== this.buffer.ybase || 0 !== this.buffer.y) {
              this.buffer.clearAllMarkers(), this.buffer.lines.set(0, this.buffer.lines.get(this.buffer.ybase + this.buffer.y)), this.buffer.lines.length = 1, this.buffer.ydisp = 0, this.buffer.ybase = 0, this.buffer.y = 0;
              for (let e6 = 1; e6 < this.rows; e6++) this.buffer.lines.push(this.buffer.getBlankLine(L2.DEFAULT_ATTR_DATA));
              this._onScroll.fire({ position: this.buffer.ydisp, source: 0 }), this.viewport?.reset(), this.refresh(0, this.rows - 1);
            }
          }
          reset() {
            this.options.rows = this.rows, this.options.cols = this.cols;
            const e6 = this._customKeyEventHandler;
            this._setup(), super.reset(), this._selectionService?.reset(), this._decorationService.reset(), this.viewport?.reset(), this._customKeyEventHandler = e6, this.refresh(0, this.rows - 1);
          }
          clearTextureAtlas() {
            this._renderService?.clearTextureAtlas();
          }
          _reportFocus() {
            this.element?.classList.contains("focus") ? this.coreService.triggerDataEvent(D2.C0.ESC + "[I") : this.coreService.triggerDataEvent(D2.C0.ESC + "[O");
          }
          _reportWindowsOptions(e6) {
            if (this._renderService) switch (e6) {
              case T2.WindowsOptionsReportType.GET_WIN_SIZE_PIXELS:
                const e7 = this._renderService.dimensions.css.canvas.width.toFixed(0), t5 = this._renderService.dimensions.css.canvas.height.toFixed(0);
                this.coreService.triggerDataEvent(`${D2.C0.ESC}[4;${t5};${e7}t`);
                break;
              case T2.WindowsOptionsReportType.GET_CELL_SIZE_PIXELS:
                const i7 = this._renderService.dimensions.css.cell.width.toFixed(0), s6 = this._renderService.dimensions.css.cell.height.toFixed(0);
                this.coreService.triggerDataEvent(`${D2.C0.ESC}[6;${s6};${i7}t`);
            }
          }
          cancel(e6, t5) {
            if (this.options.cancelEvents || t5) return e6.preventDefault(), e6.stopPropagation(), false;
          }
        }
        t4.Terminal = P2;
      }, 9924: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.TimeBasedDebouncer = void 0, t4.TimeBasedDebouncer = class {
          constructor(e6, t5 = 1e3) {
            this._renderCallback = e6, this._debounceThresholdMS = t5, this._lastRefreshMs = 0, this._additionalRefreshRequested = false;
          }
          dispose() {
            this._refreshTimeoutID && clearTimeout(this._refreshTimeoutID);
          }
          refresh(e6, t5, i6) {
            this._rowCount = i6, e6 = void 0 !== e6 ? e6 : 0, t5 = void 0 !== t5 ? t5 : this._rowCount - 1, this._rowStart = void 0 !== this._rowStart ? Math.min(this._rowStart, e6) : e6, this._rowEnd = void 0 !== this._rowEnd ? Math.max(this._rowEnd, t5) : t5;
            const s5 = Date.now();
            if (s5 - this._lastRefreshMs >= this._debounceThresholdMS) this._lastRefreshMs = s5, this._innerRefresh();
            else if (!this._additionalRefreshRequested) {
              const e7 = s5 - this._lastRefreshMs, t6 = this._debounceThresholdMS - e7;
              this._additionalRefreshRequested = true, this._refreshTimeoutID = window.setTimeout((() => {
                this._lastRefreshMs = Date.now(), this._innerRefresh(), this._additionalRefreshRequested = false, this._refreshTimeoutID = void 0;
              }), t6);
            }
          }
          _innerRefresh() {
            if (void 0 === this._rowStart || void 0 === this._rowEnd || void 0 === this._rowCount) return;
            const e6 = Math.max(this._rowStart, 0), t5 = Math.min(this._rowEnd, this._rowCount - 1);
            this._rowStart = void 0, this._rowEnd = void 0, this._renderCallback(e6, t5);
          }
        };
      }, 1680: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.Viewport = void 0;
        const n4 = i6(3656), o5 = i6(4725), a3 = i6(8460), h3 = i6(844), c4 = i6(2585);
        let l3 = t4.Viewport = class extends h3.Disposable {
          constructor(e6, t5, i7, s6, r5, o6, h4, c5) {
            super(), this._viewportElement = e6, this._scrollArea = t5, this._bufferService = i7, this._optionsService = s6, this._charSizeService = r5, this._renderService = o6, this._coreBrowserService = h4, this.scrollBarWidth = 0, this._currentRowHeight = 0, this._currentDeviceCellHeight = 0, this._lastRecordedBufferLength = 0, this._lastRecordedViewportHeight = 0, this._lastRecordedBufferHeight = 0, this._lastTouchY = 0, this._lastScrollTop = 0, this._wheelPartialScroll = 0, this._refreshAnimationFrame = null, this._ignoreNextScrollEvent = false, this._smoothScrollState = { startTime: 0, origin: -1, target: -1 }, this._onRequestScrollLines = this.register(new a3.EventEmitter()), this.onRequestScrollLines = this._onRequestScrollLines.event, this.scrollBarWidth = this._viewportElement.offsetWidth - this._scrollArea.offsetWidth || 15, this.register((0, n4.addDisposableDomListener)(this._viewportElement, "scroll", this._handleScroll.bind(this))), this._activeBuffer = this._bufferService.buffer, this.register(this._bufferService.buffers.onBufferActivate(((e7) => this._activeBuffer = e7.activeBuffer))), this._renderDimensions = this._renderService.dimensions, this.register(this._renderService.onDimensionsChange(((e7) => this._renderDimensions = e7))), this._handleThemeChange(c5.colors), this.register(c5.onChangeColors(((e7) => this._handleThemeChange(e7)))), this.register(this._optionsService.onSpecificOptionChange("scrollback", (() => this.syncScrollArea()))), setTimeout((() => this.syncScrollArea()));
          }
          _handleThemeChange(e6) {
            this._viewportElement.style.backgroundColor = e6.background.css;
          }
          reset() {
            this._currentRowHeight = 0, this._currentDeviceCellHeight = 0, this._lastRecordedBufferLength = 0, this._lastRecordedViewportHeight = 0, this._lastRecordedBufferHeight = 0, this._lastTouchY = 0, this._lastScrollTop = 0, this._coreBrowserService.window.requestAnimationFrame((() => this.syncScrollArea()));
          }
          _refresh(e6) {
            if (e6) return this._innerRefresh(), void (null !== this._refreshAnimationFrame && this._coreBrowserService.window.cancelAnimationFrame(this._refreshAnimationFrame));
            null === this._refreshAnimationFrame && (this._refreshAnimationFrame = this._coreBrowserService.window.requestAnimationFrame((() => this._innerRefresh())));
          }
          _innerRefresh() {
            if (this._charSizeService.height > 0) {
              this._currentRowHeight = this._renderDimensions.device.cell.height / this._coreBrowserService.dpr, this._currentDeviceCellHeight = this._renderDimensions.device.cell.height, this._lastRecordedViewportHeight = this._viewportElement.offsetHeight;
              const e7 = Math.round(this._currentRowHeight * this._lastRecordedBufferLength) + (this._lastRecordedViewportHeight - this._renderDimensions.css.canvas.height);
              this._lastRecordedBufferHeight !== e7 && (this._lastRecordedBufferHeight = e7, this._scrollArea.style.height = this._lastRecordedBufferHeight + "px");
            }
            const e6 = this._bufferService.buffer.ydisp * this._currentRowHeight;
            this._viewportElement.scrollTop !== e6 && (this._ignoreNextScrollEvent = true, this._viewportElement.scrollTop = e6), this._refreshAnimationFrame = null;
          }
          syncScrollArea(e6 = false) {
            if (this._lastRecordedBufferLength !== this._bufferService.buffer.lines.length) return this._lastRecordedBufferLength = this._bufferService.buffer.lines.length, void this._refresh(e6);
            this._lastRecordedViewportHeight === this._renderService.dimensions.css.canvas.height && this._lastScrollTop === this._activeBuffer.ydisp * this._currentRowHeight && this._renderDimensions.device.cell.height === this._currentDeviceCellHeight || this._refresh(e6);
          }
          _handleScroll(e6) {
            if (this._lastScrollTop = this._viewportElement.scrollTop, !this._viewportElement.offsetParent) return;
            if (this._ignoreNextScrollEvent) return this._ignoreNextScrollEvent = false, void this._onRequestScrollLines.fire({ amount: 0, suppressScrollEvent: true });
            const t5 = Math.round(this._lastScrollTop / this._currentRowHeight) - this._bufferService.buffer.ydisp;
            this._onRequestScrollLines.fire({ amount: t5, suppressScrollEvent: true });
          }
          _smoothScroll() {
            if (this._isDisposed || -1 === this._smoothScrollState.origin || -1 === this._smoothScrollState.target) return;
            const e6 = this._smoothScrollPercent();
            this._viewportElement.scrollTop = this._smoothScrollState.origin + Math.round(e6 * (this._smoothScrollState.target - this._smoothScrollState.origin)), e6 < 1 ? this._coreBrowserService.window.requestAnimationFrame((() => this._smoothScroll())) : this._clearSmoothScrollState();
          }
          _smoothScrollPercent() {
            return this._optionsService.rawOptions.smoothScrollDuration && this._smoothScrollState.startTime ? Math.max(Math.min((Date.now() - this._smoothScrollState.startTime) / this._optionsService.rawOptions.smoothScrollDuration, 1), 0) : 1;
          }
          _clearSmoothScrollState() {
            this._smoothScrollState.startTime = 0, this._smoothScrollState.origin = -1, this._smoothScrollState.target = -1;
          }
          _bubbleScroll(e6, t5) {
            const i7 = this._viewportElement.scrollTop + this._lastRecordedViewportHeight;
            return !(t5 < 0 && 0 !== this._viewportElement.scrollTop || t5 > 0 && i7 < this._lastRecordedBufferHeight) || (e6.cancelable && e6.preventDefault(), false);
          }
          handleWheel(e6) {
            const t5 = this._getPixelsScrolled(e6);
            return 0 !== t5 && (this._optionsService.rawOptions.smoothScrollDuration ? (this._smoothScrollState.startTime = Date.now(), this._smoothScrollPercent() < 1 ? (this._smoothScrollState.origin = this._viewportElement.scrollTop, -1 === this._smoothScrollState.target ? this._smoothScrollState.target = this._viewportElement.scrollTop + t5 : this._smoothScrollState.target += t5, this._smoothScrollState.target = Math.max(Math.min(this._smoothScrollState.target, this._viewportElement.scrollHeight), 0), this._smoothScroll()) : this._clearSmoothScrollState()) : this._viewportElement.scrollTop += t5, this._bubbleScroll(e6, t5));
          }
          scrollLines(e6) {
            if (0 !== e6) if (this._optionsService.rawOptions.smoothScrollDuration) {
              const t5 = e6 * this._currentRowHeight;
              this._smoothScrollState.startTime = Date.now(), this._smoothScrollPercent() < 1 ? (this._smoothScrollState.origin = this._viewportElement.scrollTop, this._smoothScrollState.target = this._smoothScrollState.origin + t5, this._smoothScrollState.target = Math.max(Math.min(this._smoothScrollState.target, this._viewportElement.scrollHeight), 0), this._smoothScroll()) : this._clearSmoothScrollState();
            } else this._onRequestScrollLines.fire({ amount: e6, suppressScrollEvent: false });
          }
          _getPixelsScrolled(e6) {
            if (0 === e6.deltaY || e6.shiftKey) return 0;
            let t5 = this._applyScrollModifier(e6.deltaY, e6);
            return e6.deltaMode === WheelEvent.DOM_DELTA_LINE ? t5 *= this._currentRowHeight : e6.deltaMode === WheelEvent.DOM_DELTA_PAGE && (t5 *= this._currentRowHeight * this._bufferService.rows), t5;
          }
          getBufferElements(e6, t5) {
            let i7, s6 = "";
            const r5 = [], n5 = t5 ?? this._bufferService.buffer.lines.length, o6 = this._bufferService.buffer.lines;
            for (let t6 = e6; t6 < n5; t6++) {
              const e7 = o6.get(t6);
              if (!e7) continue;
              const n6 = o6.get(t6 + 1)?.isWrapped;
              if (s6 += e7.translateToString(!n6), !n6 || t6 === o6.length - 1) {
                const e8 = document.createElement("div");
                e8.textContent = s6, r5.push(e8), s6.length > 0 && (i7 = e8), s6 = "";
              }
            }
            return { bufferElements: r5, cursorElement: i7 };
          }
          getLinesScrolled(e6) {
            if (0 === e6.deltaY || e6.shiftKey) return 0;
            let t5 = this._applyScrollModifier(e6.deltaY, e6);
            return e6.deltaMode === WheelEvent.DOM_DELTA_PIXEL ? (t5 /= this._currentRowHeight + 0, this._wheelPartialScroll += t5, t5 = Math.floor(Math.abs(this._wheelPartialScroll)) * (this._wheelPartialScroll > 0 ? 1 : -1), this._wheelPartialScroll %= 1) : e6.deltaMode === WheelEvent.DOM_DELTA_PAGE && (t5 *= this._bufferService.rows), t5;
          }
          _applyScrollModifier(e6, t5) {
            const i7 = this._optionsService.rawOptions.fastScrollModifier;
            return "alt" === i7 && t5.altKey || "ctrl" === i7 && t5.ctrlKey || "shift" === i7 && t5.shiftKey ? e6 * this._optionsService.rawOptions.fastScrollSensitivity * this._optionsService.rawOptions.scrollSensitivity : e6 * this._optionsService.rawOptions.scrollSensitivity;
          }
          handleTouchStart(e6) {
            this._lastTouchY = e6.touches[0].pageY;
          }
          handleTouchMove(e6) {
            const t5 = this._lastTouchY - e6.touches[0].pageY;
            return this._lastTouchY = e6.touches[0].pageY, 0 !== t5 && (this._viewportElement.scrollTop += t5, this._bubbleScroll(e6, t5));
          }
        };
        t4.Viewport = l3 = s5([r4(2, c4.IBufferService), r4(3, c4.IOptionsService), r4(4, o5.ICharSizeService), r4(5, o5.IRenderService), r4(6, o5.ICoreBrowserService), r4(7, o5.IThemeService)], l3);
      }, 3107: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.BufferDecorationRenderer = void 0;
        const n4 = i6(4725), o5 = i6(844), a3 = i6(2585);
        let h3 = t4.BufferDecorationRenderer = class extends o5.Disposable {
          constructor(e6, t5, i7, s6, r5) {
            super(), this._screenElement = e6, this._bufferService = t5, this._coreBrowserService = i7, this._decorationService = s6, this._renderService = r5, this._decorationElements = /* @__PURE__ */ new Map(), this._altBufferIsActive = false, this._dimensionsChanged = false, this._container = document.createElement("div"), this._container.classList.add("xterm-decoration-container"), this._screenElement.appendChild(this._container), this.register(this._renderService.onRenderedViewportChange((() => this._doRefreshDecorations()))), this.register(this._renderService.onDimensionsChange((() => {
              this._dimensionsChanged = true, this._queueRefresh();
            }))), this.register(this._coreBrowserService.onDprChange((() => this._queueRefresh()))), this.register(this._bufferService.buffers.onBufferActivate((() => {
              this._altBufferIsActive = this._bufferService.buffer === this._bufferService.buffers.alt;
            }))), this.register(this._decorationService.onDecorationRegistered((() => this._queueRefresh()))), this.register(this._decorationService.onDecorationRemoved(((e7) => this._removeDecoration(e7)))), this.register((0, o5.toDisposable)((() => {
              this._container.remove(), this._decorationElements.clear();
            })));
          }
          _queueRefresh() {
            void 0 === this._animationFrame && (this._animationFrame = this._renderService.addRefreshCallback((() => {
              this._doRefreshDecorations(), this._animationFrame = void 0;
            })));
          }
          _doRefreshDecorations() {
            for (const e6 of this._decorationService.decorations) this._renderDecoration(e6);
            this._dimensionsChanged = false;
          }
          _renderDecoration(e6) {
            this._refreshStyle(e6), this._dimensionsChanged && this._refreshXPosition(e6);
          }
          _createElement(e6) {
            const t5 = this._coreBrowserService.mainDocument.createElement("div");
            t5.classList.add("xterm-decoration"), t5.classList.toggle("xterm-decoration-top-layer", "top" === e6?.options?.layer), t5.style.width = `${Math.round((e6.options.width || 1) * this._renderService.dimensions.css.cell.width)}px`, t5.style.height = (e6.options.height || 1) * this._renderService.dimensions.css.cell.height + "px", t5.style.top = (e6.marker.line - this._bufferService.buffers.active.ydisp) * this._renderService.dimensions.css.cell.height + "px", t5.style.lineHeight = `${this._renderService.dimensions.css.cell.height}px`;
            const i7 = e6.options.x ?? 0;
            return i7 && i7 > this._bufferService.cols && (t5.style.display = "none"), this._refreshXPosition(e6, t5), t5;
          }
          _refreshStyle(e6) {
            const t5 = e6.marker.line - this._bufferService.buffers.active.ydisp;
            if (t5 < 0 || t5 >= this._bufferService.rows) e6.element && (e6.element.style.display = "none", e6.onRenderEmitter.fire(e6.element));
            else {
              let i7 = this._decorationElements.get(e6);
              i7 || (i7 = this._createElement(e6), e6.element = i7, this._decorationElements.set(e6, i7), this._container.appendChild(i7), e6.onDispose((() => {
                this._decorationElements.delete(e6), i7.remove();
              }))), i7.style.top = t5 * this._renderService.dimensions.css.cell.height + "px", i7.style.display = this._altBufferIsActive ? "none" : "block", e6.onRenderEmitter.fire(i7);
            }
          }
          _refreshXPosition(e6, t5 = e6.element) {
            if (!t5) return;
            const i7 = e6.options.x ?? 0;
            "right" === (e6.options.anchor || "left") ? t5.style.right = i7 ? i7 * this._renderService.dimensions.css.cell.width + "px" : "" : t5.style.left = i7 ? i7 * this._renderService.dimensions.css.cell.width + "px" : "";
          }
          _removeDecoration(e6) {
            this._decorationElements.get(e6)?.remove(), this._decorationElements.delete(e6), e6.dispose();
          }
        };
        t4.BufferDecorationRenderer = h3 = s5([r4(1, a3.IBufferService), r4(2, n4.ICoreBrowserService), r4(3, a3.IDecorationService), r4(4, n4.IRenderService)], h3);
      }, 5871: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.ColorZoneStore = void 0, t4.ColorZoneStore = class {
          constructor() {
            this._zones = [], this._zonePool = [], this._zonePoolIndex = 0, this._linePadding = { full: 0, left: 0, center: 0, right: 0 };
          }
          get zones() {
            return this._zonePool.length = Math.min(this._zonePool.length, this._zones.length), this._zones;
          }
          clear() {
            this._zones.length = 0, this._zonePoolIndex = 0;
          }
          addDecoration(e6) {
            if (e6.options.overviewRulerOptions) {
              for (const t5 of this._zones) if (t5.color === e6.options.overviewRulerOptions.color && t5.position === e6.options.overviewRulerOptions.position) {
                if (this._lineIntersectsZone(t5, e6.marker.line)) return;
                if (this._lineAdjacentToZone(t5, e6.marker.line, e6.options.overviewRulerOptions.position)) return void this._addLineToZone(t5, e6.marker.line);
              }
              if (this._zonePoolIndex < this._zonePool.length) return this._zonePool[this._zonePoolIndex].color = e6.options.overviewRulerOptions.color, this._zonePool[this._zonePoolIndex].position = e6.options.overviewRulerOptions.position, this._zonePool[this._zonePoolIndex].startBufferLine = e6.marker.line, this._zonePool[this._zonePoolIndex].endBufferLine = e6.marker.line, void this._zones.push(this._zonePool[this._zonePoolIndex++]);
              this._zones.push({ color: e6.options.overviewRulerOptions.color, position: e6.options.overviewRulerOptions.position, startBufferLine: e6.marker.line, endBufferLine: e6.marker.line }), this._zonePool.push(this._zones[this._zones.length - 1]), this._zonePoolIndex++;
            }
          }
          setPadding(e6) {
            this._linePadding = e6;
          }
          _lineIntersectsZone(e6, t5) {
            return t5 >= e6.startBufferLine && t5 <= e6.endBufferLine;
          }
          _lineAdjacentToZone(e6, t5, i6) {
            return t5 >= e6.startBufferLine - this._linePadding[i6 || "full"] && t5 <= e6.endBufferLine + this._linePadding[i6 || "full"];
          }
          _addLineToZone(e6, t5) {
            e6.startBufferLine = Math.min(e6.startBufferLine, t5), e6.endBufferLine = Math.max(e6.endBufferLine, t5);
          }
        };
      }, 5744: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.OverviewRulerRenderer = void 0;
        const n4 = i6(5871), o5 = i6(4725), a3 = i6(844), h3 = i6(2585), c4 = { full: 0, left: 0, center: 0, right: 0 }, l3 = { full: 0, left: 0, center: 0, right: 0 }, d3 = { full: 0, left: 0, center: 0, right: 0 };
        let _2 = t4.OverviewRulerRenderer = class extends a3.Disposable {
          get _width() {
            return this._optionsService.options.overviewRulerWidth || 0;
          }
          constructor(e6, t5, i7, s6, r5, o6, h4) {
            super(), this._viewportElement = e6, this._screenElement = t5, this._bufferService = i7, this._decorationService = s6, this._renderService = r5, this._optionsService = o6, this._coreBrowserService = h4, this._colorZoneStore = new n4.ColorZoneStore(), this._shouldUpdateDimensions = true, this._shouldUpdateAnchor = true, this._lastKnownBufferLength = 0, this._canvas = this._coreBrowserService.mainDocument.createElement("canvas"), this._canvas.classList.add("xterm-decoration-overview-ruler"), this._refreshCanvasDimensions(), this._viewportElement.parentElement?.insertBefore(this._canvas, this._viewportElement);
            const c5 = this._canvas.getContext("2d");
            if (!c5) throw new Error("Ctx cannot be null");
            this._ctx = c5, this._registerDecorationListeners(), this._registerBufferChangeListeners(), this._registerDimensionChangeListeners(), this.register((0, a3.toDisposable)((() => {
              this._canvas?.remove();
            })));
          }
          _registerDecorationListeners() {
            this.register(this._decorationService.onDecorationRegistered((() => this._queueRefresh(void 0, true)))), this.register(this._decorationService.onDecorationRemoved((() => this._queueRefresh(void 0, true))));
          }
          _registerBufferChangeListeners() {
            this.register(this._renderService.onRenderedViewportChange((() => this._queueRefresh()))), this.register(this._bufferService.buffers.onBufferActivate((() => {
              this._canvas.style.display = this._bufferService.buffer === this._bufferService.buffers.alt ? "none" : "block";
            }))), this.register(this._bufferService.onScroll((() => {
              this._lastKnownBufferLength !== this._bufferService.buffers.normal.lines.length && (this._refreshDrawHeightConstants(), this._refreshColorZonePadding());
            })));
          }
          _registerDimensionChangeListeners() {
            this.register(this._renderService.onRender((() => {
              this._containerHeight && this._containerHeight === this._screenElement.clientHeight || (this._queueRefresh(true), this._containerHeight = this._screenElement.clientHeight);
            }))), this.register(this._optionsService.onSpecificOptionChange("overviewRulerWidth", (() => this._queueRefresh(true)))), this.register(this._coreBrowserService.onDprChange((() => this._queueRefresh(true)))), this._queueRefresh(true);
          }
          _refreshDrawConstants() {
            const e6 = Math.floor(this._canvas.width / 3), t5 = Math.ceil(this._canvas.width / 3);
            l3.full = this._canvas.width, l3.left = e6, l3.center = t5, l3.right = e6, this._refreshDrawHeightConstants(), d3.full = 0, d3.left = 0, d3.center = l3.left, d3.right = l3.left + l3.center;
          }
          _refreshDrawHeightConstants() {
            c4.full = Math.round(2 * this._coreBrowserService.dpr);
            const e6 = this._canvas.height / this._bufferService.buffer.lines.length, t5 = Math.round(Math.max(Math.min(e6, 12), 6) * this._coreBrowserService.dpr);
            c4.left = t5, c4.center = t5, c4.right = t5;
          }
          _refreshColorZonePadding() {
            this._colorZoneStore.setPadding({ full: Math.floor(this._bufferService.buffers.active.lines.length / (this._canvas.height - 1) * c4.full), left: Math.floor(this._bufferService.buffers.active.lines.length / (this._canvas.height - 1) * c4.left), center: Math.floor(this._bufferService.buffers.active.lines.length / (this._canvas.height - 1) * c4.center), right: Math.floor(this._bufferService.buffers.active.lines.length / (this._canvas.height - 1) * c4.right) }), this._lastKnownBufferLength = this._bufferService.buffers.normal.lines.length;
          }
          _refreshCanvasDimensions() {
            this._canvas.style.width = `${this._width}px`, this._canvas.width = Math.round(this._width * this._coreBrowserService.dpr), this._canvas.style.height = `${this._screenElement.clientHeight}px`, this._canvas.height = Math.round(this._screenElement.clientHeight * this._coreBrowserService.dpr), this._refreshDrawConstants(), this._refreshColorZonePadding();
          }
          _refreshDecorations() {
            this._shouldUpdateDimensions && this._refreshCanvasDimensions(), this._ctx.clearRect(0, 0, this._canvas.width, this._canvas.height), this._colorZoneStore.clear();
            for (const e7 of this._decorationService.decorations) this._colorZoneStore.addDecoration(e7);
            this._ctx.lineWidth = 1;
            const e6 = this._colorZoneStore.zones;
            for (const t5 of e6) "full" !== t5.position && this._renderColorZone(t5);
            for (const t5 of e6) "full" === t5.position && this._renderColorZone(t5);
            this._shouldUpdateDimensions = false, this._shouldUpdateAnchor = false;
          }
          _renderColorZone(e6) {
            this._ctx.fillStyle = e6.color, this._ctx.fillRect(d3[e6.position || "full"], Math.round((this._canvas.height - 1) * (e6.startBufferLine / this._bufferService.buffers.active.lines.length) - c4[e6.position || "full"] / 2), l3[e6.position || "full"], Math.round((this._canvas.height - 1) * ((e6.endBufferLine - e6.startBufferLine) / this._bufferService.buffers.active.lines.length) + c4[e6.position || "full"]));
          }
          _queueRefresh(e6, t5) {
            this._shouldUpdateDimensions = e6 || this._shouldUpdateDimensions, this._shouldUpdateAnchor = t5 || this._shouldUpdateAnchor, void 0 === this._animationFrame && (this._animationFrame = this._coreBrowserService.window.requestAnimationFrame((() => {
              this._refreshDecorations(), this._animationFrame = void 0;
            })));
          }
        };
        t4.OverviewRulerRenderer = _2 = s5([r4(2, h3.IBufferService), r4(3, h3.IDecorationService), r4(4, o5.IRenderService), r4(5, h3.IOptionsService), r4(6, o5.ICoreBrowserService)], _2);
      }, 2950: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.CompositionHelper = void 0;
        const n4 = i6(4725), o5 = i6(2585), a3 = i6(2584);
        let h3 = t4.CompositionHelper = class {
          get isComposing() {
            return this._isComposing;
          }
          constructor(e6, t5, i7, s6, r5, n5) {
            this._textarea = e6, this._compositionView = t5, this._bufferService = i7, this._optionsService = s6, this._coreService = r5, this._renderService = n5, this._isComposing = false, this._isSendingComposition = false, this._compositionPosition = { start: 0, end: 0 }, this._dataAlreadySent = "";
          }
          compositionstart() {
            this._isComposing = true, this._compositionPosition.start = this._textarea.value.length, this._compositionView.textContent = "", this._dataAlreadySent = "", this._compositionView.classList.add("active");
          }
          compositionupdate(e6) {
            this._compositionView.textContent = e6.data, this.updateCompositionElements(), setTimeout((() => {
              this._compositionPosition.end = this._textarea.value.length;
            }), 0);
          }
          compositionend() {
            this._finalizeComposition(true);
          }
          keydown(e6) {
            if (this._isComposing || this._isSendingComposition) {
              if (229 === e6.keyCode) return false;
              if (16 === e6.keyCode || 17 === e6.keyCode || 18 === e6.keyCode) return false;
              this._finalizeComposition(false);
            }
            return 229 !== e6.keyCode || (this._handleAnyTextareaChanges(), false);
          }
          _finalizeComposition(e6) {
            if (this._compositionView.classList.remove("active"), this._isComposing = false, e6) {
              const e7 = { start: this._compositionPosition.start, end: this._compositionPosition.end };
              this._isSendingComposition = true, setTimeout((() => {
                if (this._isSendingComposition) {
                  let t5;
                  this._isSendingComposition = false, e7.start += this._dataAlreadySent.length, t5 = this._isComposing ? this._textarea.value.substring(e7.start, e7.end) : this._textarea.value.substring(e7.start), t5.length > 0 && this._coreService.triggerDataEvent(t5, true);
                }
              }), 0);
            } else {
              this._isSendingComposition = false;
              const e7 = this._textarea.value.substring(this._compositionPosition.start, this._compositionPosition.end);
              this._coreService.triggerDataEvent(e7, true);
            }
          }
          _handleAnyTextareaChanges() {
            const e6 = this._textarea.value;
            setTimeout((() => {
              if (!this._isComposing) {
                const t5 = this._textarea.value, i7 = t5.replace(e6, "");
                this._dataAlreadySent = i7, t5.length > e6.length ? this._coreService.triggerDataEvent(i7, true) : t5.length < e6.length ? this._coreService.triggerDataEvent(`${a3.C0.DEL}`, true) : t5.length === e6.length && t5 !== e6 && this._coreService.triggerDataEvent(t5, true);
              }
            }), 0);
          }
          updateCompositionElements(e6) {
            if (this._isComposing) {
              if (this._bufferService.buffer.isCursorInViewport) {
                const e7 = Math.min(this._bufferService.buffer.x, this._bufferService.cols - 1), t5 = this._renderService.dimensions.css.cell.height, i7 = this._bufferService.buffer.y * this._renderService.dimensions.css.cell.height, s6 = e7 * this._renderService.dimensions.css.cell.width;
                this._compositionView.style.left = s6 + "px", this._compositionView.style.top = i7 + "px", this._compositionView.style.height = t5 + "px", this._compositionView.style.lineHeight = t5 + "px", this._compositionView.style.fontFamily = this._optionsService.rawOptions.fontFamily, this._compositionView.style.fontSize = this._optionsService.rawOptions.fontSize + "px";
                const r5 = this._compositionView.getBoundingClientRect();
                this._textarea.style.left = s6 + "px", this._textarea.style.top = i7 + "px", this._textarea.style.width = Math.max(r5.width, 1) + "px", this._textarea.style.height = Math.max(r5.height, 1) + "px", this._textarea.style.lineHeight = r5.height + "px";
              }
              e6 || setTimeout((() => this.updateCompositionElements(true)), 0);
            }
          }
        };
        t4.CompositionHelper = h3 = s5([r4(2, o5.IBufferService), r4(3, o5.IOptionsService), r4(4, o5.ICoreService), r4(5, n4.IRenderService)], h3);
      }, 9806: (e5, t4) => {
        function i6(e6, t5, i7) {
          const s5 = i7.getBoundingClientRect(), r4 = e6.getComputedStyle(i7), n4 = parseInt(r4.getPropertyValue("padding-left")), o5 = parseInt(r4.getPropertyValue("padding-top"));
          return [t5.clientX - s5.left - n4, t5.clientY - s5.top - o5];
        }
        Object.defineProperty(t4, "__esModule", { value: true }), t4.getCoords = t4.getCoordsRelativeToElement = void 0, t4.getCoordsRelativeToElement = i6, t4.getCoords = function(e6, t5, s5, r4, n4, o5, a3, h3, c4) {
          if (!o5) return;
          const l3 = i6(e6, t5, s5);
          return l3 ? (l3[0] = Math.ceil((l3[0] + (c4 ? a3 / 2 : 0)) / a3), l3[1] = Math.ceil(l3[1] / h3), l3[0] = Math.min(Math.max(l3[0], 1), r4 + (c4 ? 1 : 0)), l3[1] = Math.min(Math.max(l3[1], 1), n4), l3) : void 0;
        };
      }, 9504: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.moveToCellSequence = void 0;
        const s5 = i6(2584);
        function r4(e6, t5, i7, s6) {
          const r5 = e6 - n4(e6, i7), a4 = t5 - n4(t5, i7), l3 = Math.abs(r5 - a4) - (function(e7, t6, i8) {
            let s7 = 0;
            const r6 = e7 - n4(e7, i8), a5 = t6 - n4(t6, i8);
            for (let n5 = 0; n5 < Math.abs(r6 - a5); n5++) {
              const a6 = "A" === o5(e7, t6) ? -1 : 1, h4 = i8.buffer.lines.get(r6 + a6 * n5);
              h4?.isWrapped && s7++;
            }
            return s7;
          })(e6, t5, i7);
          return c4(l3, h3(o5(e6, t5), s6));
        }
        function n4(e6, t5) {
          let i7 = 0, s6 = t5.buffer.lines.get(e6), r5 = s6?.isWrapped;
          for (; r5 && e6 >= 0 && e6 < t5.rows; ) i7++, s6 = t5.buffer.lines.get(--e6), r5 = s6?.isWrapped;
          return i7;
        }
        function o5(e6, t5) {
          return e6 > t5 ? "A" : "B";
        }
        function a3(e6, t5, i7, s6, r5, n5) {
          let o6 = e6, a4 = t5, h4 = "";
          for (; o6 !== i7 || a4 !== s6; ) o6 += r5 ? 1 : -1, r5 && o6 > n5.cols - 1 ? (h4 += n5.buffer.translateBufferLineToString(a4, false, e6, o6), o6 = 0, e6 = 0, a4++) : !r5 && o6 < 0 && (h4 += n5.buffer.translateBufferLineToString(a4, false, 0, e6 + 1), o6 = n5.cols - 1, e6 = o6, a4--);
          return h4 + n5.buffer.translateBufferLineToString(a4, false, e6, o6);
        }
        function h3(e6, t5) {
          const i7 = t5 ? "O" : "[";
          return s5.C0.ESC + i7 + e6;
        }
        function c4(e6, t5) {
          e6 = Math.floor(e6);
          let i7 = "";
          for (let s6 = 0; s6 < e6; s6++) i7 += t5;
          return i7;
        }
        t4.moveToCellSequence = function(e6, t5, i7, s6) {
          const o6 = i7.buffer.x, l3 = i7.buffer.y;
          if (!i7.buffer.hasScrollback) return (function(e7, t6, i8, s7, o7, l4) {
            return 0 === r4(t6, s7, o7, l4).length ? "" : c4(a3(e7, t6, e7, t6 - n4(t6, o7), false, o7).length, h3("D", l4));
          })(o6, l3, 0, t5, i7, s6) + r4(l3, t5, i7, s6) + (function(e7, t6, i8, s7, o7, l4) {
            let d4;
            d4 = r4(t6, s7, o7, l4).length > 0 ? s7 - n4(s7, o7) : t6;
            const _3 = s7, u3 = (function(e8, t7, i9, s8, o8, a4) {
              let h4;
              return h4 = r4(i9, s8, o8, a4).length > 0 ? s8 - n4(s8, o8) : t7, e8 < i9 && h4 <= s8 || e8 >= i9 && h4 < s8 ? "C" : "D";
            })(e7, t6, i8, s7, o7, l4);
            return c4(a3(e7, d4, i8, _3, "C" === u3, o7).length, h3(u3, l4));
          })(o6, l3, e6, t5, i7, s6);
          let d3;
          if (l3 === t5) return d3 = o6 > e6 ? "D" : "C", c4(Math.abs(o6 - e6), h3(d3, s6));
          d3 = l3 > t5 ? "D" : "C";
          const _2 = Math.abs(l3 - t5);
          return c4((function(e7, t6) {
            return t6.cols - e7;
          })(l3 > t5 ? e6 : o6, i7) + (_2 - 1) * i7.cols + 1 + ((l3 > t5 ? o6 : e6) - 1), h3(d3, s6));
        };
      }, 1296: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.DomRenderer = void 0;
        const n4 = i6(3787), o5 = i6(2550), a3 = i6(2223), h3 = i6(6171), c4 = i6(6052), l3 = i6(4725), d3 = i6(8055), _2 = i6(8460), u3 = i6(844), f3 = i6(2585), v2 = "xterm-dom-renderer-owner-", p3 = "xterm-rows", g2 = "xterm-fg-", m2 = "xterm-bg-", S3 = "xterm-focus", C2 = "xterm-selection";
        let b3 = 1, w2 = t4.DomRenderer = class extends u3.Disposable {
          constructor(e6, t5, i7, s6, r5, a4, l4, d4, f4, g3, m3, S4, w3) {
            super(), this._terminal = e6, this._document = t5, this._element = i7, this._screenElement = s6, this._viewportElement = r5, this._helperContainer = a4, this._linkifier2 = l4, this._charSizeService = f4, this._optionsService = g3, this._bufferService = m3, this._coreBrowserService = S4, this._themeService = w3, this._terminalClass = b3++, this._rowElements = [], this._selectionRenderModel = (0, c4.createSelectionRenderModel)(), this.onRequestRedraw = this.register(new _2.EventEmitter()).event, this._rowContainer = this._document.createElement("div"), this._rowContainer.classList.add(p3), this._rowContainer.style.lineHeight = "normal", this._rowContainer.setAttribute("aria-hidden", "true"), this._refreshRowElements(this._bufferService.cols, this._bufferService.rows), this._selectionContainer = this._document.createElement("div"), this._selectionContainer.classList.add(C2), this._selectionContainer.setAttribute("aria-hidden", "true"), this.dimensions = (0, h3.createRenderDimensions)(), this._updateDimensions(), this.register(this._optionsService.onOptionChange((() => this._handleOptionsChanged()))), this.register(this._themeService.onChangeColors(((e7) => this._injectCss(e7)))), this._injectCss(this._themeService.colors), this._rowFactory = d4.createInstance(n4.DomRendererRowFactory, document), this._element.classList.add(v2 + this._terminalClass), this._screenElement.appendChild(this._rowContainer), this._screenElement.appendChild(this._selectionContainer), this.register(this._linkifier2.onShowLinkUnderline(((e7) => this._handleLinkHover(e7)))), this.register(this._linkifier2.onHideLinkUnderline(((e7) => this._handleLinkLeave(e7)))), this.register((0, u3.toDisposable)((() => {
              this._element.classList.remove(v2 + this._terminalClass), this._rowContainer.remove(), this._selectionContainer.remove(), this._widthCache.dispose(), this._themeStyleElement.remove(), this._dimensionsStyleElement.remove();
            }))), this._widthCache = new o5.WidthCache(this._document, this._helperContainer), this._widthCache.setFont(this._optionsService.rawOptions.fontFamily, this._optionsService.rawOptions.fontSize, this._optionsService.rawOptions.fontWeight, this._optionsService.rawOptions.fontWeightBold), this._setDefaultSpacing();
          }
          _updateDimensions() {
            const e6 = this._coreBrowserService.dpr;
            this.dimensions.device.char.width = this._charSizeService.width * e6, this.dimensions.device.char.height = Math.ceil(this._charSizeService.height * e6), this.dimensions.device.cell.width = this.dimensions.device.char.width + Math.round(this._optionsService.rawOptions.letterSpacing), this.dimensions.device.cell.height = Math.floor(this.dimensions.device.char.height * this._optionsService.rawOptions.lineHeight), this.dimensions.device.char.left = 0, this.dimensions.device.char.top = 0, this.dimensions.device.canvas.width = this.dimensions.device.cell.width * this._bufferService.cols, this.dimensions.device.canvas.height = this.dimensions.device.cell.height * this._bufferService.rows, this.dimensions.css.canvas.width = Math.round(this.dimensions.device.canvas.width / e6), this.dimensions.css.canvas.height = Math.round(this.dimensions.device.canvas.height / e6), this.dimensions.css.cell.width = this.dimensions.css.canvas.width / this._bufferService.cols, this.dimensions.css.cell.height = this.dimensions.css.canvas.height / this._bufferService.rows;
            for (const e7 of this._rowElements) e7.style.width = `${this.dimensions.css.canvas.width}px`, e7.style.height = `${this.dimensions.css.cell.height}px`, e7.style.lineHeight = `${this.dimensions.css.cell.height}px`, e7.style.overflow = "hidden";
            this._dimensionsStyleElement || (this._dimensionsStyleElement = this._document.createElement("style"), this._screenElement.appendChild(this._dimensionsStyleElement));
            const t5 = `${this._terminalSelector} .${p3} span { display: inline-block; height: 100%; vertical-align: top;}`;
            this._dimensionsStyleElement.textContent = t5, this._selectionContainer.style.height = this._viewportElement.style.height, this._screenElement.style.width = `${this.dimensions.css.canvas.width}px`, this._screenElement.style.height = `${this.dimensions.css.canvas.height}px`;
          }
          _injectCss(e6) {
            this._themeStyleElement || (this._themeStyleElement = this._document.createElement("style"), this._screenElement.appendChild(this._themeStyleElement));
            let t5 = `${this._terminalSelector} .${p3} { color: ${e6.foreground.css}; font-family: ${this._optionsService.rawOptions.fontFamily}; font-size: ${this._optionsService.rawOptions.fontSize}px; font-kerning: none; white-space: pre}`;
            t5 += `${this._terminalSelector} .${p3} .xterm-dim { color: ${d3.color.multiplyOpacity(e6.foreground, 0.5).css};}`, t5 += `${this._terminalSelector} span:not(.xterm-bold) { font-weight: ${this._optionsService.rawOptions.fontWeight};}${this._terminalSelector} span.xterm-bold { font-weight: ${this._optionsService.rawOptions.fontWeightBold};}${this._terminalSelector} span.xterm-italic { font-style: italic;}`;
            const i7 = `blink_underline_${this._terminalClass}`, s6 = `blink_bar_${this._terminalClass}`, r5 = `blink_block_${this._terminalClass}`;
            t5 += `@keyframes ${i7} { 50% {  border-bottom-style: hidden; }}`, t5 += `@keyframes ${s6} { 50% {  box-shadow: none; }}`, t5 += `@keyframes ${r5} { 0% {  background-color: ${e6.cursor.css};  color: ${e6.cursorAccent.css}; } 50% {  background-color: inherit;  color: ${e6.cursor.css}; }}`, t5 += `${this._terminalSelector} .${p3}.${S3} .xterm-cursor.xterm-cursor-blink.xterm-cursor-underline { animation: ${i7} 1s step-end infinite;}${this._terminalSelector} .${p3}.${S3} .xterm-cursor.xterm-cursor-blink.xterm-cursor-bar { animation: ${s6} 1s step-end infinite;}${this._terminalSelector} .${p3}.${S3} .xterm-cursor.xterm-cursor-blink.xterm-cursor-block { animation: ${r5} 1s step-end infinite;}${this._terminalSelector} .${p3} .xterm-cursor.xterm-cursor-block { background-color: ${e6.cursor.css}; color: ${e6.cursorAccent.css};}${this._terminalSelector} .${p3} .xterm-cursor.xterm-cursor-block:not(.xterm-cursor-blink) { background-color: ${e6.cursor.css} !important; color: ${e6.cursorAccent.css} !important;}${this._terminalSelector} .${p3} .xterm-cursor.xterm-cursor-outline { outline: 1px solid ${e6.cursor.css}; outline-offset: -1px;}${this._terminalSelector} .${p3} .xterm-cursor.xterm-cursor-bar { box-shadow: ${this._optionsService.rawOptions.cursorWidth}px 0 0 ${e6.cursor.css} inset;}${this._terminalSelector} .${p3} .xterm-cursor.xterm-cursor-underline { border-bottom: 1px ${e6.cursor.css}; border-bottom-style: solid; height: calc(100% - 1px);}`, t5 += `${this._terminalSelector} .${C2} { position: absolute; top: 0; left: 0; z-index: 1; pointer-events: none;}${this._terminalSelector}.focus .${C2} div { position: absolute; background-color: ${e6.selectionBackgroundOpaque.css};}${this._terminalSelector} .${C2} div { position: absolute; background-color: ${e6.selectionInactiveBackgroundOpaque.css};}`;
            for (const [i8, s7] of e6.ansi.entries()) t5 += `${this._terminalSelector} .${g2}${i8} { color: ${s7.css}; }${this._terminalSelector} .${g2}${i8}.xterm-dim { color: ${d3.color.multiplyOpacity(s7, 0.5).css}; }${this._terminalSelector} .${m2}${i8} { background-color: ${s7.css}; }`;
            t5 += `${this._terminalSelector} .${g2}${a3.INVERTED_DEFAULT_COLOR} { color: ${d3.color.opaque(e6.background).css}; }${this._terminalSelector} .${g2}${a3.INVERTED_DEFAULT_COLOR}.xterm-dim { color: ${d3.color.multiplyOpacity(d3.color.opaque(e6.background), 0.5).css}; }${this._terminalSelector} .${m2}${a3.INVERTED_DEFAULT_COLOR} { background-color: ${e6.foreground.css}; }`, this._themeStyleElement.textContent = t5;
          }
          _setDefaultSpacing() {
            const e6 = this.dimensions.css.cell.width - this._widthCache.get("W", false, false);
            this._rowContainer.style.letterSpacing = `${e6}px`, this._rowFactory.defaultSpacing = e6;
          }
          handleDevicePixelRatioChange() {
            this._updateDimensions(), this._widthCache.clear(), this._setDefaultSpacing();
          }
          _refreshRowElements(e6, t5) {
            for (let e7 = this._rowElements.length; e7 <= t5; e7++) {
              const e8 = this._document.createElement("div");
              this._rowContainer.appendChild(e8), this._rowElements.push(e8);
            }
            for (; this._rowElements.length > t5; ) this._rowContainer.removeChild(this._rowElements.pop());
          }
          handleResize(e6, t5) {
            this._refreshRowElements(e6, t5), this._updateDimensions(), this.handleSelectionChanged(this._selectionRenderModel.selectionStart, this._selectionRenderModel.selectionEnd, this._selectionRenderModel.columnSelectMode);
          }
          handleCharSizeChanged() {
            this._updateDimensions(), this._widthCache.clear(), this._setDefaultSpacing();
          }
          handleBlur() {
            this._rowContainer.classList.remove(S3), this.renderRows(0, this._bufferService.rows - 1);
          }
          handleFocus() {
            this._rowContainer.classList.add(S3), this.renderRows(this._bufferService.buffer.y, this._bufferService.buffer.y);
          }
          handleSelectionChanged(e6, t5, i7) {
            if (this._selectionContainer.replaceChildren(), this._rowFactory.handleSelectionChanged(e6, t5, i7), this.renderRows(0, this._bufferService.rows - 1), !e6 || !t5) return;
            this._selectionRenderModel.update(this._terminal, e6, t5, i7);
            const s6 = this._selectionRenderModel.viewportStartRow, r5 = this._selectionRenderModel.viewportEndRow, n5 = this._selectionRenderModel.viewportCappedStartRow, o6 = this._selectionRenderModel.viewportCappedEndRow;
            if (n5 >= this._bufferService.rows || o6 < 0) return;
            const a4 = this._document.createDocumentFragment();
            if (i7) {
              const i8 = e6[0] > t5[0];
              a4.appendChild(this._createSelectionElement(n5, i8 ? t5[0] : e6[0], i8 ? e6[0] : t5[0], o6 - n5 + 1));
            } else {
              const i8 = s6 === n5 ? e6[0] : 0, h4 = n5 === r5 ? t5[0] : this._bufferService.cols;
              a4.appendChild(this._createSelectionElement(n5, i8, h4));
              const c5 = o6 - n5 - 1;
              if (a4.appendChild(this._createSelectionElement(n5 + 1, 0, this._bufferService.cols, c5)), n5 !== o6) {
                const e7 = r5 === o6 ? t5[0] : this._bufferService.cols;
                a4.appendChild(this._createSelectionElement(o6, 0, e7));
              }
            }
            this._selectionContainer.appendChild(a4);
          }
          _createSelectionElement(e6, t5, i7, s6 = 1) {
            const r5 = this._document.createElement("div"), n5 = t5 * this.dimensions.css.cell.width;
            let o6 = this.dimensions.css.cell.width * (i7 - t5);
            return n5 + o6 > this.dimensions.css.canvas.width && (o6 = this.dimensions.css.canvas.width - n5), r5.style.height = s6 * this.dimensions.css.cell.height + "px", r5.style.top = e6 * this.dimensions.css.cell.height + "px", r5.style.left = `${n5}px`, r5.style.width = `${o6}px`, r5;
          }
          handleCursorMove() {
          }
          _handleOptionsChanged() {
            this._updateDimensions(), this._injectCss(this._themeService.colors), this._widthCache.setFont(this._optionsService.rawOptions.fontFamily, this._optionsService.rawOptions.fontSize, this._optionsService.rawOptions.fontWeight, this._optionsService.rawOptions.fontWeightBold), this._setDefaultSpacing();
          }
          clear() {
            for (const e6 of this._rowElements) e6.replaceChildren();
          }
          renderRows(e6, t5) {
            const i7 = this._bufferService.buffer, s6 = i7.ybase + i7.y, r5 = Math.min(i7.x, this._bufferService.cols - 1), n5 = this._optionsService.rawOptions.cursorBlink, o6 = this._optionsService.rawOptions.cursorStyle, a4 = this._optionsService.rawOptions.cursorInactiveStyle;
            for (let h4 = e6; h4 <= t5; h4++) {
              const e7 = h4 + i7.ydisp, t6 = this._rowElements[h4], c5 = i7.lines.get(e7);
              if (!t6 || !c5) break;
              t6.replaceChildren(...this._rowFactory.createRow(c5, e7, e7 === s6, o6, a4, r5, n5, this.dimensions.css.cell.width, this._widthCache, -1, -1));
            }
          }
          get _terminalSelector() {
            return `.${v2}${this._terminalClass}`;
          }
          _handleLinkHover(e6) {
            this._setCellUnderline(e6.x1, e6.x2, e6.y1, e6.y2, e6.cols, true);
          }
          _handleLinkLeave(e6) {
            this._setCellUnderline(e6.x1, e6.x2, e6.y1, e6.y2, e6.cols, false);
          }
          _setCellUnderline(e6, t5, i7, s6, r5, n5) {
            i7 < 0 && (e6 = 0), s6 < 0 && (t5 = 0);
            const o6 = this._bufferService.rows - 1;
            i7 = Math.max(Math.min(i7, o6), 0), s6 = Math.max(Math.min(s6, o6), 0), r5 = Math.min(r5, this._bufferService.cols);
            const a4 = this._bufferService.buffer, h4 = a4.ybase + a4.y, c5 = Math.min(a4.x, r5 - 1), l4 = this._optionsService.rawOptions.cursorBlink, d4 = this._optionsService.rawOptions.cursorStyle, _3 = this._optionsService.rawOptions.cursorInactiveStyle;
            for (let o7 = i7; o7 <= s6; ++o7) {
              const u4 = o7 + a4.ydisp, f4 = this._rowElements[o7], v3 = a4.lines.get(u4);
              if (!f4 || !v3) break;
              f4.replaceChildren(...this._rowFactory.createRow(v3, u4, u4 === h4, d4, _3, c5, l4, this.dimensions.css.cell.width, this._widthCache, n5 ? o7 === i7 ? e6 : 0 : -1, n5 ? (o7 === s6 ? t5 : r5) - 1 : -1));
            }
          }
        };
        t4.DomRenderer = w2 = s5([r4(7, f3.IInstantiationService), r4(8, l3.ICharSizeService), r4(9, f3.IOptionsService), r4(10, f3.IBufferService), r4(11, l3.ICoreBrowserService), r4(12, l3.IThemeService)], w2);
      }, 3787: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.DomRendererRowFactory = void 0;
        const n4 = i6(2223), o5 = i6(643), a3 = i6(511), h3 = i6(2585), c4 = i6(8055), l3 = i6(4725), d3 = i6(4269), _2 = i6(6171), u3 = i6(3734);
        let f3 = t4.DomRendererRowFactory = class {
          constructor(e6, t5, i7, s6, r5, n5, o6) {
            this._document = e6, this._characterJoinerService = t5, this._optionsService = i7, this._coreBrowserService = s6, this._coreService = r5, this._decorationService = n5, this._themeService = o6, this._workCell = new a3.CellData(), this._columnSelectMode = false, this.defaultSpacing = 0;
          }
          handleSelectionChanged(e6, t5, i7) {
            this._selectionStart = e6, this._selectionEnd = t5, this._columnSelectMode = i7;
          }
          createRow(e6, t5, i7, s6, r5, a4, h4, l4, _3, f4, p3) {
            const g2 = [], m2 = this._characterJoinerService.getJoinedCharacters(t5), S3 = this._themeService.colors;
            let C2, b3 = e6.getNoBgTrimmedLength();
            i7 && b3 < a4 + 1 && (b3 = a4 + 1);
            let w2 = 0, y3 = "", E2 = 0, k2 = 0, L2 = 0, D2 = false, R2 = 0, x2 = false, A2 = 0;
            const B2 = [], T2 = -1 !== f4 && -1 !== p3;
            for (let M2 = 0; M2 < b3; M2++) {
              e6.loadCell(M2, this._workCell);
              let b4 = this._workCell.getWidth();
              if (0 === b4) continue;
              let O = false, P2 = M2, I2 = this._workCell;
              if (m2.length > 0 && M2 === m2[0][0]) {
                O = true;
                const t6 = m2.shift();
                I2 = new d3.JoinedCellData(this._workCell, e6.translateToString(true, t6[0], t6[1]), t6[1] - t6[0]), P2 = t6[1] - 1, b4 = I2.getWidth();
              }
              const H2 = this._isCellInSelection(M2, t5), F = i7 && M2 === a4, W = T2 && M2 >= f4 && M2 <= p3;
              let U = false;
              this._decorationService.forEachDecorationAtCell(M2, t5, void 0, ((e7) => {
                U = true;
              }));
              let N2 = I2.getChars() || o5.WHITESPACE_CELL_CHAR;
              if (" " === N2 && (I2.isUnderline() || I2.isOverline()) && (N2 = "\xA0"), A2 = b4 * l4 - _3.get(N2, I2.isBold(), I2.isItalic()), C2) {
                if (w2 && (H2 && x2 || !H2 && !x2 && I2.bg === E2) && (H2 && x2 && S3.selectionForeground || I2.fg === k2) && I2.extended.ext === L2 && W === D2 && A2 === R2 && !F && !O && !U) {
                  I2.isInvisible() ? y3 += o5.WHITESPACE_CELL_CHAR : y3 += N2, w2++;
                  continue;
                }
                w2 && (C2.textContent = y3), C2 = this._document.createElement("span"), w2 = 0, y3 = "";
              } else C2 = this._document.createElement("span");
              if (E2 = I2.bg, k2 = I2.fg, L2 = I2.extended.ext, D2 = W, R2 = A2, x2 = H2, O && a4 >= M2 && a4 <= P2 && (a4 = M2), !this._coreService.isCursorHidden && F && this._coreService.isCursorInitialized) {
                if (B2.push("xterm-cursor"), this._coreBrowserService.isFocused) h4 && B2.push("xterm-cursor-blink"), B2.push("bar" === s6 ? "xterm-cursor-bar" : "underline" === s6 ? "xterm-cursor-underline" : "xterm-cursor-block");
                else if (r5) switch (r5) {
                  case "outline":
                    B2.push("xterm-cursor-outline");
                    break;
                  case "block":
                    B2.push("xterm-cursor-block");
                    break;
                  case "bar":
                    B2.push("xterm-cursor-bar");
                    break;
                  case "underline":
                    B2.push("xterm-cursor-underline");
                }
              }
              if (I2.isBold() && B2.push("xterm-bold"), I2.isItalic() && B2.push("xterm-italic"), I2.isDim() && B2.push("xterm-dim"), y3 = I2.isInvisible() ? o5.WHITESPACE_CELL_CHAR : I2.getChars() || o5.WHITESPACE_CELL_CHAR, I2.isUnderline() && (B2.push(`xterm-underline-${I2.extended.underlineStyle}`), " " === y3 && (y3 = "\xA0"), !I2.isUnderlineColorDefault())) if (I2.isUnderlineColorRGB()) C2.style.textDecorationColor = `rgb(${u3.AttributeData.toColorRGB(I2.getUnderlineColor()).join(",")})`;
              else {
                let e7 = I2.getUnderlineColor();
                this._optionsService.rawOptions.drawBoldTextInBrightColors && I2.isBold() && e7 < 8 && (e7 += 8), C2.style.textDecorationColor = S3.ansi[e7].css;
              }
              I2.isOverline() && (B2.push("xterm-overline"), " " === y3 && (y3 = "\xA0")), I2.isStrikethrough() && B2.push("xterm-strikethrough"), W && (C2.style.textDecoration = "underline");
              let $2 = I2.getFgColor(), j = I2.getFgColorMode(), z2 = I2.getBgColor(), K = I2.getBgColorMode();
              const q = !!I2.isInverse();
              if (q) {
                const e7 = $2;
                $2 = z2, z2 = e7;
                const t6 = j;
                j = K, K = t6;
              }
              let V2, G, X, J = false;
              switch (this._decorationService.forEachDecorationAtCell(M2, t5, void 0, ((e7) => {
                "top" !== e7.options.layer && J || (e7.backgroundColorRGB && (K = 50331648, z2 = e7.backgroundColorRGB.rgba >> 8 & 16777215, V2 = e7.backgroundColorRGB), e7.foregroundColorRGB && (j = 50331648, $2 = e7.foregroundColorRGB.rgba >> 8 & 16777215, G = e7.foregroundColorRGB), J = "top" === e7.options.layer);
              })), !J && H2 && (V2 = this._coreBrowserService.isFocused ? S3.selectionBackgroundOpaque : S3.selectionInactiveBackgroundOpaque, z2 = V2.rgba >> 8 & 16777215, K = 50331648, J = true, S3.selectionForeground && (j = 50331648, $2 = S3.selectionForeground.rgba >> 8 & 16777215, G = S3.selectionForeground)), J && B2.push("xterm-decoration-top"), K) {
                case 16777216:
                case 33554432:
                  X = S3.ansi[z2], B2.push(`xterm-bg-${z2}`);
                  break;
                case 50331648:
                  X = c4.channels.toColor(z2 >> 16, z2 >> 8 & 255, 255 & z2), this._addStyle(C2, `background-color:#${v2((z2 >>> 0).toString(16), "0", 6)}`);
                  break;
                default:
                  q ? (X = S3.foreground, B2.push(`xterm-bg-${n4.INVERTED_DEFAULT_COLOR}`)) : X = S3.background;
              }
              switch (V2 || I2.isDim() && (V2 = c4.color.multiplyOpacity(X, 0.5)), j) {
                case 16777216:
                case 33554432:
                  I2.isBold() && $2 < 8 && this._optionsService.rawOptions.drawBoldTextInBrightColors && ($2 += 8), this._applyMinimumContrast(C2, X, S3.ansi[$2], I2, V2, void 0) || B2.push(`xterm-fg-${$2}`);
                  break;
                case 50331648:
                  const e7 = c4.channels.toColor($2 >> 16 & 255, $2 >> 8 & 255, 255 & $2);
                  this._applyMinimumContrast(C2, X, e7, I2, V2, G) || this._addStyle(C2, `color:#${v2($2.toString(16), "0", 6)}`);
                  break;
                default:
                  this._applyMinimumContrast(C2, X, S3.foreground, I2, V2, G) || q && B2.push(`xterm-fg-${n4.INVERTED_DEFAULT_COLOR}`);
              }
              B2.length && (C2.className = B2.join(" "), B2.length = 0), F || O || U ? C2.textContent = y3 : w2++, A2 !== this.defaultSpacing && (C2.style.letterSpacing = `${A2}px`), g2.push(C2), M2 = P2;
            }
            return C2 && w2 && (C2.textContent = y3), g2;
          }
          _applyMinimumContrast(e6, t5, i7, s6, r5, n5) {
            if (1 === this._optionsService.rawOptions.minimumContrastRatio || (0, _2.treatGlyphAsBackgroundColor)(s6.getCode())) return false;
            const o6 = this._getContrastCache(s6);
            let a4;
            if (r5 || n5 || (a4 = o6.getColor(t5.rgba, i7.rgba)), void 0 === a4) {
              const e7 = this._optionsService.rawOptions.minimumContrastRatio / (s6.isDim() ? 2 : 1);
              a4 = c4.color.ensureContrastRatio(r5 || t5, n5 || i7, e7), o6.setColor((r5 || t5).rgba, (n5 || i7).rgba, a4 ?? null);
            }
            return !!a4 && (this._addStyle(e6, `color:${a4.css}`), true);
          }
          _getContrastCache(e6) {
            return e6.isDim() ? this._themeService.colors.halfContrastCache : this._themeService.colors.contrastCache;
          }
          _addStyle(e6, t5) {
            e6.setAttribute("style", `${e6.getAttribute("style") || ""}${t5};`);
          }
          _isCellInSelection(e6, t5) {
            const i7 = this._selectionStart, s6 = this._selectionEnd;
            return !(!i7 || !s6) && (this._columnSelectMode ? i7[0] <= s6[0] ? e6 >= i7[0] && t5 >= i7[1] && e6 < s6[0] && t5 <= s6[1] : e6 < i7[0] && t5 >= i7[1] && e6 >= s6[0] && t5 <= s6[1] : t5 > i7[1] && t5 < s6[1] || i7[1] === s6[1] && t5 === i7[1] && e6 >= i7[0] && e6 < s6[0] || i7[1] < s6[1] && t5 === s6[1] && e6 < s6[0] || i7[1] < s6[1] && t5 === i7[1] && e6 >= i7[0]);
          }
        };
        function v2(e6, t5, i7) {
          for (; e6.length < i7; ) e6 = t5 + e6;
          return e6;
        }
        t4.DomRendererRowFactory = f3 = s5([r4(1, l3.ICharacterJoinerService), r4(2, h3.IOptionsService), r4(3, l3.ICoreBrowserService), r4(4, h3.ICoreService), r4(5, h3.IDecorationService), r4(6, l3.IThemeService)], f3);
      }, 2550: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.WidthCache = void 0, t4.WidthCache = class {
          constructor(e6, t5) {
            this._flat = new Float32Array(256), this._font = "", this._fontSize = 0, this._weight = "normal", this._weightBold = "bold", this._measureElements = [], this._container = e6.createElement("div"), this._container.classList.add("xterm-width-cache-measure-container"), this._container.setAttribute("aria-hidden", "true"), this._container.style.whiteSpace = "pre", this._container.style.fontKerning = "none";
            const i6 = e6.createElement("span");
            i6.classList.add("xterm-char-measure-element");
            const s5 = e6.createElement("span");
            s5.classList.add("xterm-char-measure-element"), s5.style.fontWeight = "bold";
            const r4 = e6.createElement("span");
            r4.classList.add("xterm-char-measure-element"), r4.style.fontStyle = "italic";
            const n4 = e6.createElement("span");
            n4.classList.add("xterm-char-measure-element"), n4.style.fontWeight = "bold", n4.style.fontStyle = "italic", this._measureElements = [i6, s5, r4, n4], this._container.appendChild(i6), this._container.appendChild(s5), this._container.appendChild(r4), this._container.appendChild(n4), t5.appendChild(this._container), this.clear();
          }
          dispose() {
            this._container.remove(), this._measureElements.length = 0, this._holey = void 0;
          }
          clear() {
            this._flat.fill(-9999), this._holey = /* @__PURE__ */ new Map();
          }
          setFont(e6, t5, i6, s5) {
            e6 === this._font && t5 === this._fontSize && i6 === this._weight && s5 === this._weightBold || (this._font = e6, this._fontSize = t5, this._weight = i6, this._weightBold = s5, this._container.style.fontFamily = this._font, this._container.style.fontSize = `${this._fontSize}px`, this._measureElements[0].style.fontWeight = `${i6}`, this._measureElements[1].style.fontWeight = `${s5}`, this._measureElements[2].style.fontWeight = `${i6}`, this._measureElements[3].style.fontWeight = `${s5}`, this.clear());
          }
          get(e6, t5, i6) {
            let s5 = 0;
            if (!t5 && !i6 && 1 === e6.length && (s5 = e6.charCodeAt(0)) < 256) {
              if (-9999 !== this._flat[s5]) return this._flat[s5];
              const t6 = this._measure(e6, 0);
              return t6 > 0 && (this._flat[s5] = t6), t6;
            }
            let r4 = e6;
            t5 && (r4 += "B"), i6 && (r4 += "I");
            let n4 = this._holey.get(r4);
            if (void 0 === n4) {
              let s6 = 0;
              t5 && (s6 |= 1), i6 && (s6 |= 2), n4 = this._measure(e6, s6), n4 > 0 && this._holey.set(r4, n4);
            }
            return n4;
          }
          _measure(e6, t5) {
            const i6 = this._measureElements[t5];
            return i6.textContent = e6.repeat(32), i6.offsetWidth / 32;
          }
        };
      }, 2223: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.TEXT_BASELINE = t4.DIM_OPACITY = t4.INVERTED_DEFAULT_COLOR = void 0;
        const s5 = i6(6114);
        t4.INVERTED_DEFAULT_COLOR = 257, t4.DIM_OPACITY = 0.5, t4.TEXT_BASELINE = s5.isFirefox || s5.isLegacyEdge ? "bottom" : "ideographic";
      }, 6171: (e5, t4) => {
        function i6(e6) {
          return 57508 <= e6 && e6 <= 57558;
        }
        function s5(e6) {
          return e6 >= 128512 && e6 <= 128591 || e6 >= 127744 && e6 <= 128511 || e6 >= 128640 && e6 <= 128767 || e6 >= 9728 && e6 <= 9983 || e6 >= 9984 && e6 <= 10175 || e6 >= 65024 && e6 <= 65039 || e6 >= 129280 && e6 <= 129535 || e6 >= 127462 && e6 <= 127487;
        }
        Object.defineProperty(t4, "__esModule", { value: true }), t4.computeNextVariantOffset = t4.createRenderDimensions = t4.treatGlyphAsBackgroundColor = t4.allowRescaling = t4.isEmoji = t4.isRestrictedPowerlineGlyph = t4.isPowerlineGlyph = t4.throwIfFalsy = void 0, t4.throwIfFalsy = function(e6) {
          if (!e6) throw new Error("value must not be falsy");
          return e6;
        }, t4.isPowerlineGlyph = i6, t4.isRestrictedPowerlineGlyph = function(e6) {
          return 57520 <= e6 && e6 <= 57527;
        }, t4.isEmoji = s5, t4.allowRescaling = function(e6, t5, r4, n4) {
          return 1 === t5 && r4 > Math.ceil(1.5 * n4) && void 0 !== e6 && e6 > 255 && !s5(e6) && !i6(e6) && !(function(e7) {
            return 57344 <= e7 && e7 <= 63743;
          })(e6);
        }, t4.treatGlyphAsBackgroundColor = function(e6) {
          return i6(e6) || (function(e7) {
            return 9472 <= e7 && e7 <= 9631;
          })(e6);
        }, t4.createRenderDimensions = function() {
          return { css: { canvas: { width: 0, height: 0 }, cell: { width: 0, height: 0 } }, device: { canvas: { width: 0, height: 0 }, cell: { width: 0, height: 0 }, char: { width: 0, height: 0, left: 0, top: 0 } } };
        }, t4.computeNextVariantOffset = function(e6, t5, i7 = 0) {
          return (e6 - (2 * Math.round(t5) - i7)) % (2 * Math.round(t5));
        };
      }, 6052: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.createSelectionRenderModel = void 0;
        class i6 {
          constructor() {
            this.clear();
          }
          clear() {
            this.hasSelection = false, this.columnSelectMode = false, this.viewportStartRow = 0, this.viewportEndRow = 0, this.viewportCappedStartRow = 0, this.viewportCappedEndRow = 0, this.startCol = 0, this.endCol = 0, this.selectionStart = void 0, this.selectionEnd = void 0;
          }
          update(e6, t5, i7, s5 = false) {
            if (this.selectionStart = t5, this.selectionEnd = i7, !t5 || !i7 || t5[0] === i7[0] && t5[1] === i7[1]) return void this.clear();
            const r4 = e6.buffers.active.ydisp, n4 = t5[1] - r4, o5 = i7[1] - r4, a3 = Math.max(n4, 0), h3 = Math.min(o5, e6.rows - 1);
            a3 >= e6.rows || h3 < 0 ? this.clear() : (this.hasSelection = true, this.columnSelectMode = s5, this.viewportStartRow = n4, this.viewportEndRow = o5, this.viewportCappedStartRow = a3, this.viewportCappedEndRow = h3, this.startCol = t5[0], this.endCol = i7[0]);
          }
          isCellSelected(e6, t5, i7) {
            return !!this.hasSelection && (i7 -= e6.buffer.active.viewportY, this.columnSelectMode ? this.startCol <= this.endCol ? t5 >= this.startCol && i7 >= this.viewportCappedStartRow && t5 < this.endCol && i7 <= this.viewportCappedEndRow : t5 < this.startCol && i7 >= this.viewportCappedStartRow && t5 >= this.endCol && i7 <= this.viewportCappedEndRow : i7 > this.viewportStartRow && i7 < this.viewportEndRow || this.viewportStartRow === this.viewportEndRow && i7 === this.viewportStartRow && t5 >= this.startCol && t5 < this.endCol || this.viewportStartRow < this.viewportEndRow && i7 === this.viewportEndRow && t5 < this.endCol || this.viewportStartRow < this.viewportEndRow && i7 === this.viewportStartRow && t5 >= this.startCol);
          }
        }
        t4.createSelectionRenderModel = function() {
          return new i6();
        };
      }, 456: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.SelectionModel = void 0, t4.SelectionModel = class {
          constructor(e6) {
            this._bufferService = e6, this.isSelectAllActive = false, this.selectionStartLength = 0;
          }
          clearSelection() {
            this.selectionStart = void 0, this.selectionEnd = void 0, this.isSelectAllActive = false, this.selectionStartLength = 0;
          }
          get finalSelectionStart() {
            return this.isSelectAllActive ? [0, 0] : this.selectionEnd && this.selectionStart && this.areSelectionValuesReversed() ? this.selectionEnd : this.selectionStart;
          }
          get finalSelectionEnd() {
            if (this.isSelectAllActive) return [this._bufferService.cols, this._bufferService.buffer.ybase + this._bufferService.rows - 1];
            if (this.selectionStart) {
              if (!this.selectionEnd || this.areSelectionValuesReversed()) {
                const e6 = this.selectionStart[0] + this.selectionStartLength;
                return e6 > this._bufferService.cols ? e6 % this._bufferService.cols == 0 ? [this._bufferService.cols, this.selectionStart[1] + Math.floor(e6 / this._bufferService.cols) - 1] : [e6 % this._bufferService.cols, this.selectionStart[1] + Math.floor(e6 / this._bufferService.cols)] : [e6, this.selectionStart[1]];
              }
              if (this.selectionStartLength && this.selectionEnd[1] === this.selectionStart[1]) {
                const e6 = this.selectionStart[0] + this.selectionStartLength;
                return e6 > this._bufferService.cols ? [e6 % this._bufferService.cols, this.selectionStart[1] + Math.floor(e6 / this._bufferService.cols)] : [Math.max(e6, this.selectionEnd[0]), this.selectionEnd[1]];
              }
              return this.selectionEnd;
            }
          }
          areSelectionValuesReversed() {
            const e6 = this.selectionStart, t5 = this.selectionEnd;
            return !(!e6 || !t5) && (e6[1] > t5[1] || e6[1] === t5[1] && e6[0] > t5[0]);
          }
          handleTrim(e6) {
            return this.selectionStart && (this.selectionStart[1] -= e6), this.selectionEnd && (this.selectionEnd[1] -= e6), this.selectionEnd && this.selectionEnd[1] < 0 ? (this.clearSelection(), true) : (this.selectionStart && this.selectionStart[1] < 0 && (this.selectionStart[1] = 0), false);
          }
        };
      }, 428: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.CharSizeService = void 0;
        const n4 = i6(2585), o5 = i6(8460), a3 = i6(844);
        let h3 = t4.CharSizeService = class extends a3.Disposable {
          get hasValidSize() {
            return this.width > 0 && this.height > 0;
          }
          constructor(e6, t5, i7) {
            super(), this._optionsService = i7, this.width = 0, this.height = 0, this._onCharSizeChange = this.register(new o5.EventEmitter()), this.onCharSizeChange = this._onCharSizeChange.event;
            try {
              this._measureStrategy = this.register(new d3(this._optionsService));
            } catch {
              this._measureStrategy = this.register(new l3(e6, t5, this._optionsService));
            }
            this.register(this._optionsService.onMultipleOptionChange(["fontFamily", "fontSize"], (() => this.measure())));
          }
          measure() {
            const e6 = this._measureStrategy.measure();
            e6.width === this.width && e6.height === this.height || (this.width = e6.width, this.height = e6.height, this._onCharSizeChange.fire());
          }
        };
        t4.CharSizeService = h3 = s5([r4(2, n4.IOptionsService)], h3);
        class c4 extends a3.Disposable {
          constructor() {
            super(...arguments), this._result = { width: 0, height: 0 };
          }
          _validateAndSet(e6, t5) {
            void 0 !== e6 && e6 > 0 && void 0 !== t5 && t5 > 0 && (this._result.width = e6, this._result.height = t5);
          }
        }
        class l3 extends c4 {
          constructor(e6, t5, i7) {
            super(), this._document = e6, this._parentElement = t5, this._optionsService = i7, this._measureElement = this._document.createElement("span"), this._measureElement.classList.add("xterm-char-measure-element"), this._measureElement.textContent = "W".repeat(32), this._measureElement.setAttribute("aria-hidden", "true"), this._measureElement.style.whiteSpace = "pre", this._measureElement.style.fontKerning = "none", this._parentElement.appendChild(this._measureElement);
          }
          measure() {
            return this._measureElement.style.fontFamily = this._optionsService.rawOptions.fontFamily, this._measureElement.style.fontSize = `${this._optionsService.rawOptions.fontSize}px`, this._validateAndSet(Number(this._measureElement.offsetWidth) / 32, Number(this._measureElement.offsetHeight)), this._result;
          }
        }
        class d3 extends c4 {
          constructor(e6) {
            super(), this._optionsService = e6, this._canvas = new OffscreenCanvas(100, 100), this._ctx = this._canvas.getContext("2d");
            const t5 = this._ctx.measureText("W");
            if (!("width" in t5 && "fontBoundingBoxAscent" in t5 && "fontBoundingBoxDescent" in t5)) throw new Error("Required font metrics not supported");
          }
          measure() {
            this._ctx.font = `${this._optionsService.rawOptions.fontSize}px ${this._optionsService.rawOptions.fontFamily}`;
            const e6 = this._ctx.measureText("W");
            return this._validateAndSet(e6.width, e6.fontBoundingBoxAscent + e6.fontBoundingBoxDescent), this._result;
          }
        }
      }, 4269: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.CharacterJoinerService = t4.JoinedCellData = void 0;
        const n4 = i6(3734), o5 = i6(643), a3 = i6(511), h3 = i6(2585);
        class c4 extends n4.AttributeData {
          constructor(e6, t5, i7) {
            super(), this.content = 0, this.combinedData = "", this.fg = e6.fg, this.bg = e6.bg, this.combinedData = t5, this._width = i7;
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
          setFromCharData(e6) {
            throw new Error("not implemented");
          }
          getAsCharData() {
            return [this.fg, this.getChars(), this.getWidth(), this.getCode()];
          }
        }
        t4.JoinedCellData = c4;
        let l3 = t4.CharacterJoinerService = class e6 {
          constructor(e7) {
            this._bufferService = e7, this._characterJoiners = [], this._nextCharacterJoinerId = 0, this._workCell = new a3.CellData();
          }
          register(e7) {
            const t5 = { id: this._nextCharacterJoinerId++, handler: e7 };
            return this._characterJoiners.push(t5), t5.id;
          }
          deregister(e7) {
            for (let t5 = 0; t5 < this._characterJoiners.length; t5++) if (this._characterJoiners[t5].id === e7) return this._characterJoiners.splice(t5, 1), true;
            return false;
          }
          getJoinedCharacters(e7) {
            if (0 === this._characterJoiners.length) return [];
            const t5 = this._bufferService.buffer.lines.get(e7);
            if (!t5 || 0 === t5.length) return [];
            const i7 = [], s6 = t5.translateToString(true);
            let r5 = 0, n5 = 0, a4 = 0, h4 = t5.getFg(0), c5 = t5.getBg(0);
            for (let e8 = 0; e8 < t5.getTrimmedLength(); e8++) if (t5.loadCell(e8, this._workCell), 0 !== this._workCell.getWidth()) {
              if (this._workCell.fg !== h4 || this._workCell.bg !== c5) {
                if (e8 - r5 > 1) {
                  const e9 = this._getJoinedRanges(s6, a4, n5, t5, r5);
                  for (let t6 = 0; t6 < e9.length; t6++) i7.push(e9[t6]);
                }
                r5 = e8, a4 = n5, h4 = this._workCell.fg, c5 = this._workCell.bg;
              }
              n5 += this._workCell.getChars().length || o5.WHITESPACE_CELL_CHAR.length;
            }
            if (this._bufferService.cols - r5 > 1) {
              const e8 = this._getJoinedRanges(s6, a4, n5, t5, r5);
              for (let t6 = 0; t6 < e8.length; t6++) i7.push(e8[t6]);
            }
            return i7;
          }
          _getJoinedRanges(t5, i7, s6, r5, n5) {
            const o6 = t5.substring(i7, s6);
            let a4 = [];
            try {
              a4 = this._characterJoiners[0].handler(o6);
            } catch (e7) {
              console.error(e7);
            }
            for (let t6 = 1; t6 < this._characterJoiners.length; t6++) try {
              const i8 = this._characterJoiners[t6].handler(o6);
              for (let t7 = 0; t7 < i8.length; t7++) e6._mergeRanges(a4, i8[t7]);
            } catch (e7) {
              console.error(e7);
            }
            return this._stringRangesToCellRanges(a4, r5, n5), a4;
          }
          _stringRangesToCellRanges(e7, t5, i7) {
            let s6 = 0, r5 = false, n5 = 0, a4 = e7[s6];
            if (a4) {
              for (let h4 = i7; h4 < this._bufferService.cols; h4++) {
                const i8 = t5.getWidth(h4), c5 = t5.getString(h4).length || o5.WHITESPACE_CELL_CHAR.length;
                if (0 !== i8) {
                  if (!r5 && a4[0] <= n5 && (a4[0] = h4, r5 = true), a4[1] <= n5) {
                    if (a4[1] = h4, a4 = e7[++s6], !a4) break;
                    a4[0] <= n5 ? (a4[0] = h4, r5 = true) : r5 = false;
                  }
                  n5 += c5;
                }
              }
              a4 && (a4[1] = this._bufferService.cols);
            }
          }
          static _mergeRanges(e7, t5) {
            let i7 = false;
            for (let s6 = 0; s6 < e7.length; s6++) {
              const r5 = e7[s6];
              if (i7) {
                if (t5[1] <= r5[0]) return e7[s6 - 1][1] = t5[1], e7;
                if (t5[1] <= r5[1]) return e7[s6 - 1][1] = Math.max(t5[1], r5[1]), e7.splice(s6, 1), e7;
                e7.splice(s6, 1), s6--;
              } else {
                if (t5[1] <= r5[0]) return e7.splice(s6, 0, t5), e7;
                if (t5[1] <= r5[1]) return r5[0] = Math.min(t5[0], r5[0]), e7;
                t5[0] < r5[1] && (r5[0] = Math.min(t5[0], r5[0]), i7 = true);
              }
            }
            return i7 ? e7[e7.length - 1][1] = t5[1] : e7.push(t5), e7;
          }
        };
        t4.CharacterJoinerService = l3 = s5([r4(0, h3.IBufferService)], l3);
      }, 5114: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.CoreBrowserService = void 0;
        const s5 = i6(844), r4 = i6(8460), n4 = i6(3656);
        class o5 extends s5.Disposable {
          constructor(e6, t5, i7) {
            super(), this._textarea = e6, this._window = t5, this.mainDocument = i7, this._isFocused = false, this._cachedIsFocused = void 0, this._screenDprMonitor = new a3(this._window), this._onDprChange = this.register(new r4.EventEmitter()), this.onDprChange = this._onDprChange.event, this._onWindowChange = this.register(new r4.EventEmitter()), this.onWindowChange = this._onWindowChange.event, this.register(this.onWindowChange(((e7) => this._screenDprMonitor.setWindow(e7)))), this.register((0, r4.forwardEvent)(this._screenDprMonitor.onDprChange, this._onDprChange)), this._textarea.addEventListener("focus", (() => this._isFocused = true)), this._textarea.addEventListener("blur", (() => this._isFocused = false));
          }
          get window() {
            return this._window;
          }
          set window(e6) {
            this._window !== e6 && (this._window = e6, this._onWindowChange.fire(this._window));
          }
          get dpr() {
            return this.window.devicePixelRatio;
          }
          get isFocused() {
            return void 0 === this._cachedIsFocused && (this._cachedIsFocused = this._isFocused && this._textarea.ownerDocument.hasFocus(), queueMicrotask((() => this._cachedIsFocused = void 0))), this._cachedIsFocused;
          }
        }
        t4.CoreBrowserService = o5;
        class a3 extends s5.Disposable {
          constructor(e6) {
            super(), this._parentWindow = e6, this._windowResizeListener = this.register(new s5.MutableDisposable()), this._onDprChange = this.register(new r4.EventEmitter()), this.onDprChange = this._onDprChange.event, this._outerListener = () => this._setDprAndFireIfDiffers(), this._currentDevicePixelRatio = this._parentWindow.devicePixelRatio, this._updateDpr(), this._setWindowResizeListener(), this.register((0, s5.toDisposable)((() => this.clearListener())));
          }
          setWindow(e6) {
            this._parentWindow = e6, this._setWindowResizeListener(), this._setDprAndFireIfDiffers();
          }
          _setWindowResizeListener() {
            this._windowResizeListener.value = (0, n4.addDisposableDomListener)(this._parentWindow, "resize", (() => this._setDprAndFireIfDiffers()));
          }
          _setDprAndFireIfDiffers() {
            this._parentWindow.devicePixelRatio !== this._currentDevicePixelRatio && this._onDprChange.fire(this._parentWindow.devicePixelRatio), this._updateDpr();
          }
          _updateDpr() {
            this._outerListener && (this._resolutionMediaMatchList?.removeListener(this._outerListener), this._currentDevicePixelRatio = this._parentWindow.devicePixelRatio, this._resolutionMediaMatchList = this._parentWindow.matchMedia(`screen and (resolution: ${this._parentWindow.devicePixelRatio}dppx)`), this._resolutionMediaMatchList.addListener(this._outerListener));
          }
          clearListener() {
            this._resolutionMediaMatchList && this._outerListener && (this._resolutionMediaMatchList.removeListener(this._outerListener), this._resolutionMediaMatchList = void 0, this._outerListener = void 0);
          }
        }
      }, 779: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.LinkProviderService = void 0;
        const s5 = i6(844);
        class r4 extends s5.Disposable {
          constructor() {
            super(), this.linkProviders = [], this.register((0, s5.toDisposable)((() => this.linkProviders.length = 0)));
          }
          registerLinkProvider(e6) {
            return this.linkProviders.push(e6), { dispose: () => {
              const t5 = this.linkProviders.indexOf(e6);
              -1 !== t5 && this.linkProviders.splice(t5, 1);
            } };
          }
        }
        t4.LinkProviderService = r4;
      }, 8934: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.MouseService = void 0;
        const n4 = i6(4725), o5 = i6(9806);
        let a3 = t4.MouseService = class {
          constructor(e6, t5) {
            this._renderService = e6, this._charSizeService = t5;
          }
          getCoords(e6, t5, i7, s6, r5) {
            return (0, o5.getCoords)(window, e6, t5, i7, s6, this._charSizeService.hasValidSize, this._renderService.dimensions.css.cell.width, this._renderService.dimensions.css.cell.height, r5);
          }
          getMouseReportCoords(e6, t5) {
            const i7 = (0, o5.getCoordsRelativeToElement)(window, e6, t5);
            if (this._charSizeService.hasValidSize) return i7[0] = Math.min(Math.max(i7[0], 0), this._renderService.dimensions.css.canvas.width - 1), i7[1] = Math.min(Math.max(i7[1], 0), this._renderService.dimensions.css.canvas.height - 1), { col: Math.floor(i7[0] / this._renderService.dimensions.css.cell.width), row: Math.floor(i7[1] / this._renderService.dimensions.css.cell.height), x: Math.floor(i7[0]), y: Math.floor(i7[1]) };
          }
        };
        t4.MouseService = a3 = s5([r4(0, n4.IRenderService), r4(1, n4.ICharSizeService)], a3);
      }, 3230: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.RenderService = void 0;
        const n4 = i6(6193), o5 = i6(4725), a3 = i6(8460), h3 = i6(844), c4 = i6(7226), l3 = i6(2585);
        let d3 = t4.RenderService = class extends h3.Disposable {
          get dimensions() {
            return this._renderer.value.dimensions;
          }
          constructor(e6, t5, i7, s6, r5, o6, l4, d4) {
            super(), this._rowCount = e6, this._charSizeService = s6, this._renderer = this.register(new h3.MutableDisposable()), this._pausedResizeTask = new c4.DebouncedIdleTask(), this._observerDisposable = this.register(new h3.MutableDisposable()), this._isPaused = false, this._needsFullRefresh = false, this._isNextRenderRedrawOnly = true, this._needsSelectionRefresh = false, this._canvasWidth = 0, this._canvasHeight = 0, this._selectionState = { start: void 0, end: void 0, columnSelectMode: false }, this._onDimensionsChange = this.register(new a3.EventEmitter()), this.onDimensionsChange = this._onDimensionsChange.event, this._onRenderedViewportChange = this.register(new a3.EventEmitter()), this.onRenderedViewportChange = this._onRenderedViewportChange.event, this._onRender = this.register(new a3.EventEmitter()), this.onRender = this._onRender.event, this._onRefreshRequest = this.register(new a3.EventEmitter()), this.onRefreshRequest = this._onRefreshRequest.event, this._renderDebouncer = new n4.RenderDebouncer(((e7, t6) => this._renderRows(e7, t6)), l4), this.register(this._renderDebouncer), this.register(l4.onDprChange((() => this.handleDevicePixelRatioChange()))), this.register(o6.onResize((() => this._fullRefresh()))), this.register(o6.buffers.onBufferActivate((() => this._renderer.value?.clear()))), this.register(i7.onOptionChange((() => this._handleOptionsChanged()))), this.register(this._charSizeService.onCharSizeChange((() => this.handleCharSizeChanged()))), this.register(r5.onDecorationRegistered((() => this._fullRefresh()))), this.register(r5.onDecorationRemoved((() => this._fullRefresh()))), this.register(i7.onMultipleOptionChange(["customGlyphs", "drawBoldTextInBrightColors", "letterSpacing", "lineHeight", "fontFamily", "fontSize", "fontWeight", "fontWeightBold", "minimumContrastRatio", "rescaleOverlappingGlyphs"], (() => {
              this.clear(), this.handleResize(o6.cols, o6.rows), this._fullRefresh();
            }))), this.register(i7.onMultipleOptionChange(["cursorBlink", "cursorStyle"], (() => this.refreshRows(o6.buffer.y, o6.buffer.y, true)))), this.register(d4.onChangeColors((() => this._fullRefresh()))), this._registerIntersectionObserver(l4.window, t5), this.register(l4.onWindowChange(((e7) => this._registerIntersectionObserver(e7, t5))));
          }
          _registerIntersectionObserver(e6, t5) {
            if ("IntersectionObserver" in e6) {
              const i7 = new e6.IntersectionObserver(((e7) => this._handleIntersectionChange(e7[e7.length - 1])), { threshold: 0 });
              i7.observe(t5), this._observerDisposable.value = (0, h3.toDisposable)((() => i7.disconnect()));
            }
          }
          _handleIntersectionChange(e6) {
            this._isPaused = void 0 === e6.isIntersecting ? 0 === e6.intersectionRatio : !e6.isIntersecting, this._isPaused || this._charSizeService.hasValidSize || this._charSizeService.measure(), !this._isPaused && this._needsFullRefresh && (this._pausedResizeTask.flush(), this.refreshRows(0, this._rowCount - 1), this._needsFullRefresh = false);
          }
          refreshRows(e6, t5, i7 = false) {
            this._isPaused ? this._needsFullRefresh = true : (i7 || (this._isNextRenderRedrawOnly = false), this._renderDebouncer.refresh(e6, t5, this._rowCount));
          }
          _renderRows(e6, t5) {
            this._renderer.value && (e6 = Math.min(e6, this._rowCount - 1), t5 = Math.min(t5, this._rowCount - 1), this._renderer.value.renderRows(e6, t5), this._needsSelectionRefresh && (this._renderer.value.handleSelectionChanged(this._selectionState.start, this._selectionState.end, this._selectionState.columnSelectMode), this._needsSelectionRefresh = false), this._isNextRenderRedrawOnly || this._onRenderedViewportChange.fire({ start: e6, end: t5 }), this._onRender.fire({ start: e6, end: t5 }), this._isNextRenderRedrawOnly = true);
          }
          resize(e6, t5) {
            this._rowCount = t5, this._fireOnCanvasResize();
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
          setRenderer(e6) {
            this._renderer.value = e6, this._renderer.value && (this._renderer.value.onRequestRedraw(((e7) => this.refreshRows(e7.start, e7.end, true))), this._needsSelectionRefresh = true, this._fullRefresh());
          }
          addRefreshCallback(e6) {
            return this._renderDebouncer.addRefreshCallback(e6);
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
          handleResize(e6, t5) {
            this._renderer.value && (this._isPaused ? this._pausedResizeTask.set((() => this._renderer.value?.handleResize(e6, t5))) : this._renderer.value.handleResize(e6, t5), this._fullRefresh());
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
          handleSelectionChanged(e6, t5, i7) {
            this._selectionState.start = e6, this._selectionState.end = t5, this._selectionState.columnSelectMode = i7, this._renderer.value?.handleSelectionChanged(e6, t5, i7);
          }
          handleCursorMove() {
            this._renderer.value?.handleCursorMove();
          }
          clear() {
            this._renderer.value?.clear();
          }
        };
        t4.RenderService = d3 = s5([r4(2, l3.IOptionsService), r4(3, o5.ICharSizeService), r4(4, l3.IDecorationService), r4(5, l3.IBufferService), r4(6, o5.ICoreBrowserService), r4(7, o5.IThemeService)], d3);
      }, 9312: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.SelectionService = void 0;
        const n4 = i6(9806), o5 = i6(9504), a3 = i6(456), h3 = i6(4725), c4 = i6(8460), l3 = i6(844), d3 = i6(6114), _2 = i6(4841), u3 = i6(511), f3 = i6(2585), v2 = String.fromCharCode(160), p3 = new RegExp(v2, "g");
        let g2 = t4.SelectionService = class extends l3.Disposable {
          constructor(e6, t5, i7, s6, r5, n5, o6, h4, d4) {
            super(), this._element = e6, this._screenElement = t5, this._linkifier = i7, this._bufferService = s6, this._coreService = r5, this._mouseService = n5, this._optionsService = o6, this._renderService = h4, this._coreBrowserService = d4, this._dragScrollAmount = 0, this._enabled = true, this._workCell = new u3.CellData(), this._mouseDownTimeStamp = 0, this._oldHasSelection = false, this._oldSelectionStart = void 0, this._oldSelectionEnd = void 0, this._onLinuxMouseSelection = this.register(new c4.EventEmitter()), this.onLinuxMouseSelection = this._onLinuxMouseSelection.event, this._onRedrawRequest = this.register(new c4.EventEmitter()), this.onRequestRedraw = this._onRedrawRequest.event, this._onSelectionChange = this.register(new c4.EventEmitter()), this.onSelectionChange = this._onSelectionChange.event, this._onRequestScrollLines = this.register(new c4.EventEmitter()), this.onRequestScrollLines = this._onRequestScrollLines.event, this._mouseMoveListener = (e7) => this._handleMouseMove(e7), this._mouseUpListener = (e7) => this._handleMouseUp(e7), this._coreService.onUserInput((() => {
              this.hasSelection && this.clearSelection();
            })), this._trimListener = this._bufferService.buffer.lines.onTrim(((e7) => this._handleTrim(e7))), this.register(this._bufferService.buffers.onBufferActivate(((e7) => this._handleBufferActivate(e7)))), this.enable(), this._model = new a3.SelectionModel(this._bufferService), this._activeSelectionMode = 0, this.register((0, l3.toDisposable)((() => {
              this._removeMouseDownListeners();
            })));
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
            const e6 = this._model.finalSelectionStart, t5 = this._model.finalSelectionEnd;
            return !(!e6 || !t5 || e6[0] === t5[0] && e6[1] === t5[1]);
          }
          get selectionText() {
            const e6 = this._model.finalSelectionStart, t5 = this._model.finalSelectionEnd;
            if (!e6 || !t5) return "";
            const i7 = this._bufferService.buffer, s6 = [];
            if (3 === this._activeSelectionMode) {
              if (e6[0] === t5[0]) return "";
              const r5 = e6[0] < t5[0] ? e6[0] : t5[0], n5 = e6[0] < t5[0] ? t5[0] : e6[0];
              for (let o6 = e6[1]; o6 <= t5[1]; o6++) {
                const e7 = i7.translateBufferLineToString(o6, true, r5, n5);
                s6.push(e7);
              }
            } else {
              const r5 = e6[1] === t5[1] ? t5[0] : void 0;
              s6.push(i7.translateBufferLineToString(e6[1], true, e6[0], r5));
              for (let r6 = e6[1] + 1; r6 <= t5[1] - 1; r6++) {
                const e7 = i7.lines.get(r6), t6 = i7.translateBufferLineToString(r6, true);
                e7?.isWrapped ? s6[s6.length - 1] += t6 : s6.push(t6);
              }
              if (e6[1] !== t5[1]) {
                const e7 = i7.lines.get(t5[1]), r6 = i7.translateBufferLineToString(t5[1], true, 0, t5[0]);
                e7 && e7.isWrapped ? s6[s6.length - 1] += r6 : s6.push(r6);
              }
            }
            return s6.map(((e7) => e7.replace(p3, " "))).join(d3.isWindows ? "\r\n" : "\n");
          }
          clearSelection() {
            this._model.clearSelection(), this._removeMouseDownListeners(), this.refresh(), this._onSelectionChange.fire();
          }
          refresh(e6) {
            this._refreshAnimationFrame || (this._refreshAnimationFrame = this._coreBrowserService.window.requestAnimationFrame((() => this._refresh()))), d3.isLinux && e6 && this.selectionText.length && this._onLinuxMouseSelection.fire(this.selectionText);
          }
          _refresh() {
            this._refreshAnimationFrame = void 0, this._onRedrawRequest.fire({ start: this._model.finalSelectionStart, end: this._model.finalSelectionEnd, columnSelectMode: 3 === this._activeSelectionMode });
          }
          _isClickInSelection(e6) {
            const t5 = this._getMouseBufferCoords(e6), i7 = this._model.finalSelectionStart, s6 = this._model.finalSelectionEnd;
            return !!(i7 && s6 && t5) && this._areCoordsInSelection(t5, i7, s6);
          }
          isCellInSelection(e6, t5) {
            const i7 = this._model.finalSelectionStart, s6 = this._model.finalSelectionEnd;
            return !(!i7 || !s6) && this._areCoordsInSelection([e6, t5], i7, s6);
          }
          _areCoordsInSelection(e6, t5, i7) {
            return e6[1] > t5[1] && e6[1] < i7[1] || t5[1] === i7[1] && e6[1] === t5[1] && e6[0] >= t5[0] && e6[0] < i7[0] || t5[1] < i7[1] && e6[1] === i7[1] && e6[0] < i7[0] || t5[1] < i7[1] && e6[1] === t5[1] && e6[0] >= t5[0];
          }
          _selectWordAtCursor(e6, t5) {
            const i7 = this._linkifier.currentLink?.link?.range;
            if (i7) return this._model.selectionStart = [i7.start.x - 1, i7.start.y - 1], this._model.selectionStartLength = (0, _2.getRangeLength)(i7, this._bufferService.cols), this._model.selectionEnd = void 0, true;
            const s6 = this._getMouseBufferCoords(e6);
            return !!s6 && (this._selectWordAt(s6, t5), this._model.selectionEnd = void 0, true);
          }
          selectAll() {
            this._model.isSelectAllActive = true, this.refresh(), this._onSelectionChange.fire();
          }
          selectLines(e6, t5) {
            this._model.clearSelection(), e6 = Math.max(e6, 0), t5 = Math.min(t5, this._bufferService.buffer.lines.length - 1), this._model.selectionStart = [0, e6], this._model.selectionEnd = [this._bufferService.cols, t5], this.refresh(), this._onSelectionChange.fire();
          }
          _handleTrim(e6) {
            this._model.handleTrim(e6) && this.refresh();
          }
          _getMouseBufferCoords(e6) {
            const t5 = this._mouseService.getCoords(e6, this._screenElement, this._bufferService.cols, this._bufferService.rows, true);
            if (t5) return t5[0]--, t5[1]--, t5[1] += this._bufferService.buffer.ydisp, t5;
          }
          _getMouseEventScrollAmount(e6) {
            let t5 = (0, n4.getCoordsRelativeToElement)(this._coreBrowserService.window, e6, this._screenElement)[1];
            const i7 = this._renderService.dimensions.css.canvas.height;
            return t5 >= 0 && t5 <= i7 ? 0 : (t5 > i7 && (t5 -= i7), t5 = Math.min(Math.max(t5, -50), 50), t5 /= 50, t5 / Math.abs(t5) + Math.round(14 * t5));
          }
          shouldForceSelection(e6) {
            return d3.isMac ? e6.altKey && this._optionsService.rawOptions.macOptionClickForcesSelection : e6.shiftKey;
          }
          handleMouseDown(e6) {
            if (this._mouseDownTimeStamp = e6.timeStamp, (2 !== e6.button || !this.hasSelection) && 0 === e6.button) {
              if (!this._enabled) {
                if (!this.shouldForceSelection(e6)) return;
                e6.stopPropagation();
              }
              e6.preventDefault(), this._dragScrollAmount = 0, this._enabled && e6.shiftKey ? this._handleIncrementalClick(e6) : 1 === e6.detail ? this._handleSingleClick(e6) : 2 === e6.detail ? this._handleDoubleClick(e6) : 3 === e6.detail && this._handleTripleClick(e6), this._addMouseDownListeners(), this.refresh(true);
            }
          }
          _addMouseDownListeners() {
            this._screenElement.ownerDocument && (this._screenElement.ownerDocument.addEventListener("mousemove", this._mouseMoveListener), this._screenElement.ownerDocument.addEventListener("mouseup", this._mouseUpListener)), this._dragScrollIntervalTimer = this._coreBrowserService.window.setInterval((() => this._dragScroll()), 50);
          }
          _removeMouseDownListeners() {
            this._screenElement.ownerDocument && (this._screenElement.ownerDocument.removeEventListener("mousemove", this._mouseMoveListener), this._screenElement.ownerDocument.removeEventListener("mouseup", this._mouseUpListener)), this._coreBrowserService.window.clearInterval(this._dragScrollIntervalTimer), this._dragScrollIntervalTimer = void 0;
          }
          _handleIncrementalClick(e6) {
            this._model.selectionStart && (this._model.selectionEnd = this._getMouseBufferCoords(e6));
          }
          _handleSingleClick(e6) {
            if (this._model.selectionStartLength = 0, this._model.isSelectAllActive = false, this._activeSelectionMode = this.shouldColumnSelect(e6) ? 3 : 0, this._model.selectionStart = this._getMouseBufferCoords(e6), !this._model.selectionStart) return;
            this._model.selectionEnd = void 0;
            const t5 = this._bufferService.buffer.lines.get(this._model.selectionStart[1]);
            t5 && t5.length !== this._model.selectionStart[0] && 0 === t5.hasWidth(this._model.selectionStart[0]) && this._model.selectionStart[0]++;
          }
          _handleDoubleClick(e6) {
            this._selectWordAtCursor(e6, true) && (this._activeSelectionMode = 1);
          }
          _handleTripleClick(e6) {
            const t5 = this._getMouseBufferCoords(e6);
            t5 && (this._activeSelectionMode = 2, this._selectLineAt(t5[1]));
          }
          shouldColumnSelect(e6) {
            return e6.altKey && !(d3.isMac && this._optionsService.rawOptions.macOptionClickForcesSelection);
          }
          _handleMouseMove(e6) {
            if (e6.stopImmediatePropagation(), !this._model.selectionStart) return;
            const t5 = this._model.selectionEnd ? [this._model.selectionEnd[0], this._model.selectionEnd[1]] : null;
            if (this._model.selectionEnd = this._getMouseBufferCoords(e6), !this._model.selectionEnd) return void this.refresh(true);
            2 === this._activeSelectionMode ? this._model.selectionEnd[1] < this._model.selectionStart[1] ? this._model.selectionEnd[0] = 0 : this._model.selectionEnd[0] = this._bufferService.cols : 1 === this._activeSelectionMode && this._selectToWordAt(this._model.selectionEnd), this._dragScrollAmount = this._getMouseEventScrollAmount(e6), 3 !== this._activeSelectionMode && (this._dragScrollAmount > 0 ? this._model.selectionEnd[0] = this._bufferService.cols : this._dragScrollAmount < 0 && (this._model.selectionEnd[0] = 0));
            const i7 = this._bufferService.buffer;
            if (this._model.selectionEnd[1] < i7.lines.length) {
              const e7 = i7.lines.get(this._model.selectionEnd[1]);
              e7 && 0 === e7.hasWidth(this._model.selectionEnd[0]) && this._model.selectionEnd[0] < this._bufferService.cols && this._model.selectionEnd[0]++;
            }
            t5 && t5[0] === this._model.selectionEnd[0] && t5[1] === this._model.selectionEnd[1] || this.refresh(true);
          }
          _dragScroll() {
            if (this._model.selectionEnd && this._model.selectionStart && this._dragScrollAmount) {
              this._onRequestScrollLines.fire({ amount: this._dragScrollAmount, suppressScrollEvent: false });
              const e6 = this._bufferService.buffer;
              this._dragScrollAmount > 0 ? (3 !== this._activeSelectionMode && (this._model.selectionEnd[0] = this._bufferService.cols), this._model.selectionEnd[1] = Math.min(e6.ydisp + this._bufferService.rows, e6.lines.length - 1)) : (3 !== this._activeSelectionMode && (this._model.selectionEnd[0] = 0), this._model.selectionEnd[1] = e6.ydisp), this.refresh();
            }
          }
          _handleMouseUp(e6) {
            const t5 = e6.timeStamp - this._mouseDownTimeStamp;
            if (this._removeMouseDownListeners(), this.selectionText.length <= 1 && t5 < 500 && e6.altKey && this._optionsService.rawOptions.altClickMovesCursor) {
              if (this._bufferService.buffer.ybase === this._bufferService.buffer.ydisp) {
                const t6 = this._mouseService.getCoords(e6, this._element, this._bufferService.cols, this._bufferService.rows, false);
                if (t6 && void 0 !== t6[0] && void 0 !== t6[1]) {
                  const e7 = (0, o5.moveToCellSequence)(t6[0] - 1, t6[1] - 1, this._bufferService, this._coreService.decPrivateModes.applicationCursorKeys);
                  this._coreService.triggerDataEvent(e7, true);
                }
              }
            } else this._fireEventIfSelectionChanged();
          }
          _fireEventIfSelectionChanged() {
            const e6 = this._model.finalSelectionStart, t5 = this._model.finalSelectionEnd, i7 = !(!e6 || !t5 || e6[0] === t5[0] && e6[1] === t5[1]);
            i7 ? e6 && t5 && (this._oldSelectionStart && this._oldSelectionEnd && e6[0] === this._oldSelectionStart[0] && e6[1] === this._oldSelectionStart[1] && t5[0] === this._oldSelectionEnd[0] && t5[1] === this._oldSelectionEnd[1] || this._fireOnSelectionChange(e6, t5, i7)) : this._oldHasSelection && this._fireOnSelectionChange(e6, t5, i7);
          }
          _fireOnSelectionChange(e6, t5, i7) {
            this._oldSelectionStart = e6, this._oldSelectionEnd = t5, this._oldHasSelection = i7, this._onSelectionChange.fire();
          }
          _handleBufferActivate(e6) {
            this.clearSelection(), this._trimListener.dispose(), this._trimListener = e6.activeBuffer.lines.onTrim(((e7) => this._handleTrim(e7)));
          }
          _convertViewportColToCharacterIndex(e6, t5) {
            let i7 = t5;
            for (let s6 = 0; t5 >= s6; s6++) {
              const r5 = e6.loadCell(s6, this._workCell).getChars().length;
              0 === this._workCell.getWidth() ? i7-- : r5 > 1 && t5 !== s6 && (i7 += r5 - 1);
            }
            return i7;
          }
          setSelection(e6, t5, i7) {
            this._model.clearSelection(), this._removeMouseDownListeners(), this._model.selectionStart = [e6, t5], this._model.selectionStartLength = i7, this.refresh(), this._fireEventIfSelectionChanged();
          }
          rightClickSelect(e6) {
            this._isClickInSelection(e6) || (this._selectWordAtCursor(e6, false) && this.refresh(true), this._fireEventIfSelectionChanged());
          }
          _getWordAt(e6, t5, i7 = true, s6 = true) {
            if (e6[0] >= this._bufferService.cols) return;
            const r5 = this._bufferService.buffer, n5 = r5.lines.get(e6[1]);
            if (!n5) return;
            const o6 = r5.translateBufferLineToString(e6[1], false);
            let a4 = this._convertViewportColToCharacterIndex(n5, e6[0]), h4 = a4;
            const c5 = e6[0] - a4;
            let l4 = 0, d4 = 0, _3 = 0, u4 = 0;
            if (" " === o6.charAt(a4)) {
              for (; a4 > 0 && " " === o6.charAt(a4 - 1); ) a4--;
              for (; h4 < o6.length && " " === o6.charAt(h4 + 1); ) h4++;
            } else {
              let t6 = e6[0], i8 = e6[0];
              0 === n5.getWidth(t6) && (l4++, t6--), 2 === n5.getWidth(i8) && (d4++, i8++);
              const s7 = n5.getString(i8).length;
              for (s7 > 1 && (u4 += s7 - 1, h4 += s7 - 1); t6 > 0 && a4 > 0 && !this._isCharWordSeparator(n5.loadCell(t6 - 1, this._workCell)); ) {
                n5.loadCell(t6 - 1, this._workCell);
                const e7 = this._workCell.getChars().length;
                0 === this._workCell.getWidth() ? (l4++, t6--) : e7 > 1 && (_3 += e7 - 1, a4 -= e7 - 1), a4--, t6--;
              }
              for (; i8 < n5.length && h4 + 1 < o6.length && !this._isCharWordSeparator(n5.loadCell(i8 + 1, this._workCell)); ) {
                n5.loadCell(i8 + 1, this._workCell);
                const e7 = this._workCell.getChars().length;
                2 === this._workCell.getWidth() ? (d4++, i8++) : e7 > 1 && (u4 += e7 - 1, h4 += e7 - 1), h4++, i8++;
              }
            }
            h4++;
            let f4 = a4 + c5 - l4 + _3, v3 = Math.min(this._bufferService.cols, h4 - a4 + l4 + d4 - _3 - u4);
            if (t5 || "" !== o6.slice(a4, h4).trim()) {
              if (i7 && 0 === f4 && 32 !== n5.getCodePoint(0)) {
                const t6 = r5.lines.get(e6[1] - 1);
                if (t6 && n5.isWrapped && 32 !== t6.getCodePoint(this._bufferService.cols - 1)) {
                  const t7 = this._getWordAt([this._bufferService.cols - 1, e6[1] - 1], false, true, false);
                  if (t7) {
                    const e7 = this._bufferService.cols - t7.start;
                    f4 -= e7, v3 += e7;
                  }
                }
              }
              if (s6 && f4 + v3 === this._bufferService.cols && 32 !== n5.getCodePoint(this._bufferService.cols - 1)) {
                const t6 = r5.lines.get(e6[1] + 1);
                if (t6?.isWrapped && 32 !== t6.getCodePoint(0)) {
                  const t7 = this._getWordAt([0, e6[1] + 1], false, false, true);
                  t7 && (v3 += t7.length);
                }
              }
              return { start: f4, length: v3 };
            }
          }
          _selectWordAt(e6, t5) {
            const i7 = this._getWordAt(e6, t5);
            if (i7) {
              for (; i7.start < 0; ) i7.start += this._bufferService.cols, e6[1]--;
              this._model.selectionStart = [i7.start, e6[1]], this._model.selectionStartLength = i7.length;
            }
          }
          _selectToWordAt(e6) {
            const t5 = this._getWordAt(e6, true);
            if (t5) {
              let i7 = e6[1];
              for (; t5.start < 0; ) t5.start += this._bufferService.cols, i7--;
              if (!this._model.areSelectionValuesReversed()) for (; t5.start + t5.length > this._bufferService.cols; ) t5.length -= this._bufferService.cols, i7++;
              this._model.selectionEnd = [this._model.areSelectionValuesReversed() ? t5.start : t5.start + t5.length, i7];
            }
          }
          _isCharWordSeparator(e6) {
            return 0 !== e6.getWidth() && this._optionsService.rawOptions.wordSeparator.indexOf(e6.getChars()) >= 0;
          }
          _selectLineAt(e6) {
            const t5 = this._bufferService.buffer.getWrappedRangeForLine(e6), i7 = { start: { x: 0, y: t5.first }, end: { x: this._bufferService.cols - 1, y: t5.last } };
            this._model.selectionStart = [0, t5.first], this._model.selectionEnd = void 0, this._model.selectionStartLength = (0, _2.getRangeLength)(i7, this._bufferService.cols);
          }
        };
        t4.SelectionService = g2 = s5([r4(3, f3.IBufferService), r4(4, f3.ICoreService), r4(5, h3.IMouseService), r4(6, f3.IOptionsService), r4(7, h3.IRenderService), r4(8, h3.ICoreBrowserService)], g2);
      }, 4725: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.ILinkProviderService = t4.IThemeService = t4.ICharacterJoinerService = t4.ISelectionService = t4.IRenderService = t4.IMouseService = t4.ICoreBrowserService = t4.ICharSizeService = void 0;
        const s5 = i6(8343);
        t4.ICharSizeService = (0, s5.createDecorator)("CharSizeService"), t4.ICoreBrowserService = (0, s5.createDecorator)("CoreBrowserService"), t4.IMouseService = (0, s5.createDecorator)("MouseService"), t4.IRenderService = (0, s5.createDecorator)("RenderService"), t4.ISelectionService = (0, s5.createDecorator)("SelectionService"), t4.ICharacterJoinerService = (0, s5.createDecorator)("CharacterJoinerService"), t4.IThemeService = (0, s5.createDecorator)("ThemeService"), t4.ILinkProviderService = (0, s5.createDecorator)("LinkProviderService");
      }, 6731: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.ThemeService = t4.DEFAULT_ANSI_COLORS = void 0;
        const n4 = i6(7239), o5 = i6(8055), a3 = i6(8460), h3 = i6(844), c4 = i6(2585), l3 = o5.css.toColor("#ffffff"), d3 = o5.css.toColor("#000000"), _2 = o5.css.toColor("#ffffff"), u3 = o5.css.toColor("#000000"), f3 = { css: "rgba(255, 255, 255, 0.3)", rgba: 4294967117 };
        t4.DEFAULT_ANSI_COLORS = Object.freeze((() => {
          const e6 = [o5.css.toColor("#2e3436"), o5.css.toColor("#cc0000"), o5.css.toColor("#4e9a06"), o5.css.toColor("#c4a000"), o5.css.toColor("#3465a4"), o5.css.toColor("#75507b"), o5.css.toColor("#06989a"), o5.css.toColor("#d3d7cf"), o5.css.toColor("#555753"), o5.css.toColor("#ef2929"), o5.css.toColor("#8ae234"), o5.css.toColor("#fce94f"), o5.css.toColor("#729fcf"), o5.css.toColor("#ad7fa8"), o5.css.toColor("#34e2e2"), o5.css.toColor("#eeeeec")], t5 = [0, 95, 135, 175, 215, 255];
          for (let i7 = 0; i7 < 216; i7++) {
            const s6 = t5[i7 / 36 % 6 | 0], r5 = t5[i7 / 6 % 6 | 0], n5 = t5[i7 % 6];
            e6.push({ css: o5.channels.toCss(s6, r5, n5), rgba: o5.channels.toRgba(s6, r5, n5) });
          }
          for (let t6 = 0; t6 < 24; t6++) {
            const i7 = 8 + 10 * t6;
            e6.push({ css: o5.channels.toCss(i7, i7, i7), rgba: o5.channels.toRgba(i7, i7, i7) });
          }
          return e6;
        })());
        let v2 = t4.ThemeService = class extends h3.Disposable {
          get colors() {
            return this._colors;
          }
          constructor(e6) {
            super(), this._optionsService = e6, this._contrastCache = new n4.ColorContrastCache(), this._halfContrastCache = new n4.ColorContrastCache(), this._onChangeColors = this.register(new a3.EventEmitter()), this.onChangeColors = this._onChangeColors.event, this._colors = { foreground: l3, background: d3, cursor: _2, cursorAccent: u3, selectionForeground: void 0, selectionBackgroundTransparent: f3, selectionBackgroundOpaque: o5.color.blend(d3, f3), selectionInactiveBackgroundTransparent: f3, selectionInactiveBackgroundOpaque: o5.color.blend(d3, f3), ansi: t4.DEFAULT_ANSI_COLORS.slice(), contrastCache: this._contrastCache, halfContrastCache: this._halfContrastCache }, this._updateRestoreColors(), this._setTheme(this._optionsService.rawOptions.theme), this.register(this._optionsService.onSpecificOptionChange("minimumContrastRatio", (() => this._contrastCache.clear()))), this.register(this._optionsService.onSpecificOptionChange("theme", (() => this._setTheme(this._optionsService.rawOptions.theme))));
          }
          _setTheme(e6 = {}) {
            const i7 = this._colors;
            if (i7.foreground = p3(e6.foreground, l3), i7.background = p3(e6.background, d3), i7.cursor = p3(e6.cursor, _2), i7.cursorAccent = p3(e6.cursorAccent, u3), i7.selectionBackgroundTransparent = p3(e6.selectionBackground, f3), i7.selectionBackgroundOpaque = o5.color.blend(i7.background, i7.selectionBackgroundTransparent), i7.selectionInactiveBackgroundTransparent = p3(e6.selectionInactiveBackground, i7.selectionBackgroundTransparent), i7.selectionInactiveBackgroundOpaque = o5.color.blend(i7.background, i7.selectionInactiveBackgroundTransparent), i7.selectionForeground = e6.selectionForeground ? p3(e6.selectionForeground, o5.NULL_COLOR) : void 0, i7.selectionForeground === o5.NULL_COLOR && (i7.selectionForeground = void 0), o5.color.isOpaque(i7.selectionBackgroundTransparent)) {
              const e7 = 0.3;
              i7.selectionBackgroundTransparent = o5.color.opacity(i7.selectionBackgroundTransparent, e7);
            }
            if (o5.color.isOpaque(i7.selectionInactiveBackgroundTransparent)) {
              const e7 = 0.3;
              i7.selectionInactiveBackgroundTransparent = o5.color.opacity(i7.selectionInactiveBackgroundTransparent, e7);
            }
            if (i7.ansi = t4.DEFAULT_ANSI_COLORS.slice(), i7.ansi[0] = p3(e6.black, t4.DEFAULT_ANSI_COLORS[0]), i7.ansi[1] = p3(e6.red, t4.DEFAULT_ANSI_COLORS[1]), i7.ansi[2] = p3(e6.green, t4.DEFAULT_ANSI_COLORS[2]), i7.ansi[3] = p3(e6.yellow, t4.DEFAULT_ANSI_COLORS[3]), i7.ansi[4] = p3(e6.blue, t4.DEFAULT_ANSI_COLORS[4]), i7.ansi[5] = p3(e6.magenta, t4.DEFAULT_ANSI_COLORS[5]), i7.ansi[6] = p3(e6.cyan, t4.DEFAULT_ANSI_COLORS[6]), i7.ansi[7] = p3(e6.white, t4.DEFAULT_ANSI_COLORS[7]), i7.ansi[8] = p3(e6.brightBlack, t4.DEFAULT_ANSI_COLORS[8]), i7.ansi[9] = p3(e6.brightRed, t4.DEFAULT_ANSI_COLORS[9]), i7.ansi[10] = p3(e6.brightGreen, t4.DEFAULT_ANSI_COLORS[10]), i7.ansi[11] = p3(e6.brightYellow, t4.DEFAULT_ANSI_COLORS[11]), i7.ansi[12] = p3(e6.brightBlue, t4.DEFAULT_ANSI_COLORS[12]), i7.ansi[13] = p3(e6.brightMagenta, t4.DEFAULT_ANSI_COLORS[13]), i7.ansi[14] = p3(e6.brightCyan, t4.DEFAULT_ANSI_COLORS[14]), i7.ansi[15] = p3(e6.brightWhite, t4.DEFAULT_ANSI_COLORS[15]), e6.extendedAnsi) {
              const s6 = Math.min(i7.ansi.length - 16, e6.extendedAnsi.length);
              for (let r5 = 0; r5 < s6; r5++) i7.ansi[r5 + 16] = p3(e6.extendedAnsi[r5], t4.DEFAULT_ANSI_COLORS[r5 + 16]);
            }
            this._contrastCache.clear(), this._halfContrastCache.clear(), this._updateRestoreColors(), this._onChangeColors.fire(this.colors);
          }
          restoreColor(e6) {
            this._restoreColor(e6), this._onChangeColors.fire(this.colors);
          }
          _restoreColor(e6) {
            if (void 0 !== e6) switch (e6) {
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
                this._colors.ansi[e6] = this._restoreColors.ansi[e6];
            }
            else for (let e7 = 0; e7 < this._restoreColors.ansi.length; ++e7) this._colors.ansi[e7] = this._restoreColors.ansi[e7];
          }
          modifyColors(e6) {
            e6(this._colors), this._onChangeColors.fire(this.colors);
          }
          _updateRestoreColors() {
            this._restoreColors = { foreground: this._colors.foreground, background: this._colors.background, cursor: this._colors.cursor, ansi: this._colors.ansi.slice() };
          }
        };
        function p3(e6, t5) {
          if (void 0 !== e6) try {
            return o5.css.toColor(e6);
          } catch {
          }
          return t5;
        }
        t4.ThemeService = v2 = s5([r4(0, c4.IOptionsService)], v2);
      }, 6349: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.CircularList = void 0;
        const s5 = i6(8460), r4 = i6(844);
        class n4 extends r4.Disposable {
          constructor(e6) {
            super(), this._maxLength = e6, this.onDeleteEmitter = this.register(new s5.EventEmitter()), this.onDelete = this.onDeleteEmitter.event, this.onInsertEmitter = this.register(new s5.EventEmitter()), this.onInsert = this.onInsertEmitter.event, this.onTrimEmitter = this.register(new s5.EventEmitter()), this.onTrim = this.onTrimEmitter.event, this._array = new Array(this._maxLength), this._startIndex = 0, this._length = 0;
          }
          get maxLength() {
            return this._maxLength;
          }
          set maxLength(e6) {
            if (this._maxLength === e6) return;
            const t5 = new Array(e6);
            for (let i7 = 0; i7 < Math.min(e6, this.length); i7++) t5[i7] = this._array[this._getCyclicIndex(i7)];
            this._array = t5, this._maxLength = e6, this._startIndex = 0;
          }
          get length() {
            return this._length;
          }
          set length(e6) {
            if (e6 > this._length) for (let t5 = this._length; t5 < e6; t5++) this._array[t5] = void 0;
            this._length = e6;
          }
          get(e6) {
            return this._array[this._getCyclicIndex(e6)];
          }
          set(e6, t5) {
            this._array[this._getCyclicIndex(e6)] = t5;
          }
          push(e6) {
            this._array[this._getCyclicIndex(this._length)] = e6, this._length === this._maxLength ? (this._startIndex = ++this._startIndex % this._maxLength, this.onTrimEmitter.fire(1)) : this._length++;
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
          splice(e6, t5, ...i7) {
            if (t5) {
              for (let i8 = e6; i8 < this._length - t5; i8++) this._array[this._getCyclicIndex(i8)] = this._array[this._getCyclicIndex(i8 + t5)];
              this._length -= t5, this.onDeleteEmitter.fire({ index: e6, amount: t5 });
            }
            for (let t6 = this._length - 1; t6 >= e6; t6--) this._array[this._getCyclicIndex(t6 + i7.length)] = this._array[this._getCyclicIndex(t6)];
            for (let t6 = 0; t6 < i7.length; t6++) this._array[this._getCyclicIndex(e6 + t6)] = i7[t6];
            if (i7.length && this.onInsertEmitter.fire({ index: e6, amount: i7.length }), this._length + i7.length > this._maxLength) {
              const e7 = this._length + i7.length - this._maxLength;
              this._startIndex += e7, this._length = this._maxLength, this.onTrimEmitter.fire(e7);
            } else this._length += i7.length;
          }
          trimStart(e6) {
            e6 > this._length && (e6 = this._length), this._startIndex += e6, this._length -= e6, this.onTrimEmitter.fire(e6);
          }
          shiftElements(e6, t5, i7) {
            if (!(t5 <= 0)) {
              if (e6 < 0 || e6 >= this._length) throw new Error("start argument out of range");
              if (e6 + i7 < 0) throw new Error("Cannot shift elements in list beyond index 0");
              if (i7 > 0) {
                for (let s7 = t5 - 1; s7 >= 0; s7--) this.set(e6 + s7 + i7, this.get(e6 + s7));
                const s6 = e6 + t5 + i7 - this._length;
                if (s6 > 0) for (this._length += s6; this._length > this._maxLength; ) this._length--, this._startIndex++, this.onTrimEmitter.fire(1);
              } else for (let s6 = 0; s6 < t5; s6++) this.set(e6 + s6 + i7, this.get(e6 + s6));
            }
          }
          _getCyclicIndex(e6) {
            return (this._startIndex + e6) % this._maxLength;
          }
        }
        t4.CircularList = n4;
      }, 1439: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.clone = void 0, t4.clone = function e6(t5, i6 = 5) {
          if ("object" != typeof t5) return t5;
          const s5 = Array.isArray(t5) ? [] : {};
          for (const r4 in t5) s5[r4] = i6 <= 1 ? t5[r4] : t5[r4] && e6(t5[r4], i6 - 1);
          return s5;
        };
      }, 8055: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.contrastRatio = t4.toPaddedHex = t4.rgba = t4.rgb = t4.css = t4.color = t4.channels = t4.NULL_COLOR = void 0;
        let i6 = 0, s5 = 0, r4 = 0, n4 = 0;
        var o5, a3, h3, c4, l3;
        function d3(e6) {
          const t5 = e6.toString(16);
          return t5.length < 2 ? "0" + t5 : t5;
        }
        function _2(e6, t5) {
          return e6 < t5 ? (t5 + 0.05) / (e6 + 0.05) : (e6 + 0.05) / (t5 + 0.05);
        }
        t4.NULL_COLOR = { css: "#00000000", rgba: 0 }, (function(e6) {
          e6.toCss = function(e7, t5, i7, s6) {
            return void 0 !== s6 ? `#${d3(e7)}${d3(t5)}${d3(i7)}${d3(s6)}` : `#${d3(e7)}${d3(t5)}${d3(i7)}`;
          }, e6.toRgba = function(e7, t5, i7, s6 = 255) {
            return (e7 << 24 | t5 << 16 | i7 << 8 | s6) >>> 0;
          }, e6.toColor = function(t5, i7, s6, r5) {
            return { css: e6.toCss(t5, i7, s6, r5), rgba: e6.toRgba(t5, i7, s6, r5) };
          };
        })(o5 || (t4.channels = o5 = {})), (function(e6) {
          function t5(e7, t6) {
            return n4 = Math.round(255 * t6), [i6, s5, r4] = l3.toChannels(e7.rgba), { css: o5.toCss(i6, s5, r4, n4), rgba: o5.toRgba(i6, s5, r4, n4) };
          }
          e6.blend = function(e7, t6) {
            if (n4 = (255 & t6.rgba) / 255, 1 === n4) return { css: t6.css, rgba: t6.rgba };
            const a4 = t6.rgba >> 24 & 255, h4 = t6.rgba >> 16 & 255, c5 = t6.rgba >> 8 & 255, l4 = e7.rgba >> 24 & 255, d4 = e7.rgba >> 16 & 255, _3 = e7.rgba >> 8 & 255;
            return i6 = l4 + Math.round((a4 - l4) * n4), s5 = d4 + Math.round((h4 - d4) * n4), r4 = _3 + Math.round((c5 - _3) * n4), { css: o5.toCss(i6, s5, r4), rgba: o5.toRgba(i6, s5, r4) };
          }, e6.isOpaque = function(e7) {
            return 255 == (255 & e7.rgba);
          }, e6.ensureContrastRatio = function(e7, t6, i7) {
            const s6 = l3.ensureContrastRatio(e7.rgba, t6.rgba, i7);
            if (s6) return o5.toColor(s6 >> 24 & 255, s6 >> 16 & 255, s6 >> 8 & 255);
          }, e6.opaque = function(e7) {
            const t6 = (255 | e7.rgba) >>> 0;
            return [i6, s5, r4] = l3.toChannels(t6), { css: o5.toCss(i6, s5, r4), rgba: t6 };
          }, e6.opacity = t5, e6.multiplyOpacity = function(e7, i7) {
            return n4 = 255 & e7.rgba, t5(e7, n4 * i7 / 255);
          }, e6.toColorRGB = function(e7) {
            return [e7.rgba >> 24 & 255, e7.rgba >> 16 & 255, e7.rgba >> 8 & 255];
          };
        })(a3 || (t4.color = a3 = {})), (function(e6) {
          let t5, a4;
          try {
            const e7 = document.createElement("canvas");
            e7.width = 1, e7.height = 1;
            const i7 = e7.getContext("2d", { willReadFrequently: true });
            i7 && (t5 = i7, t5.globalCompositeOperation = "copy", a4 = t5.createLinearGradient(0, 0, 1, 1));
          } catch {
          }
          e6.toColor = function(e7) {
            if (e7.match(/#[\da-f]{3,8}/i)) switch (e7.length) {
              case 4:
                return i6 = parseInt(e7.slice(1, 2).repeat(2), 16), s5 = parseInt(e7.slice(2, 3).repeat(2), 16), r4 = parseInt(e7.slice(3, 4).repeat(2), 16), o5.toColor(i6, s5, r4);
              case 5:
                return i6 = parseInt(e7.slice(1, 2).repeat(2), 16), s5 = parseInt(e7.slice(2, 3).repeat(2), 16), r4 = parseInt(e7.slice(3, 4).repeat(2), 16), n4 = parseInt(e7.slice(4, 5).repeat(2), 16), o5.toColor(i6, s5, r4, n4);
              case 7:
                return { css: e7, rgba: (parseInt(e7.slice(1), 16) << 8 | 255) >>> 0 };
              case 9:
                return { css: e7, rgba: parseInt(e7.slice(1), 16) >>> 0 };
            }
            const h4 = e7.match(/rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*(,\s*(0|1|\d?\.(\d+))\s*)?\)/);
            if (h4) return i6 = parseInt(h4[1]), s5 = parseInt(h4[2]), r4 = parseInt(h4[3]), n4 = Math.round(255 * (void 0 === h4[5] ? 1 : parseFloat(h4[5]))), o5.toColor(i6, s5, r4, n4);
            if (!t5 || !a4) throw new Error("css.toColor: Unsupported css format");
            if (t5.fillStyle = a4, t5.fillStyle = e7, "string" != typeof t5.fillStyle) throw new Error("css.toColor: Unsupported css format");
            if (t5.fillRect(0, 0, 1, 1), [i6, s5, r4, n4] = t5.getImageData(0, 0, 1, 1).data, 255 !== n4) throw new Error("css.toColor: Unsupported css format");
            return { rgba: o5.toRgba(i6, s5, r4, n4), css: e7 };
          };
        })(h3 || (t4.css = h3 = {})), (function(e6) {
          function t5(e7, t6, i7) {
            const s6 = e7 / 255, r5 = t6 / 255, n5 = i7 / 255;
            return 0.2126 * (s6 <= 0.03928 ? s6 / 12.92 : Math.pow((s6 + 0.055) / 1.055, 2.4)) + 0.7152 * (r5 <= 0.03928 ? r5 / 12.92 : Math.pow((r5 + 0.055) / 1.055, 2.4)) + 0.0722 * (n5 <= 0.03928 ? n5 / 12.92 : Math.pow((n5 + 0.055) / 1.055, 2.4));
          }
          e6.relativeLuminance = function(e7) {
            return t5(e7 >> 16 & 255, e7 >> 8 & 255, 255 & e7);
          }, e6.relativeLuminance2 = t5;
        })(c4 || (t4.rgb = c4 = {})), (function(e6) {
          function t5(e7, t6, i7) {
            const s6 = e7 >> 24 & 255, r5 = e7 >> 16 & 255, n5 = e7 >> 8 & 255;
            let o6 = t6 >> 24 & 255, a5 = t6 >> 16 & 255, h4 = t6 >> 8 & 255, l4 = _2(c4.relativeLuminance2(o6, a5, h4), c4.relativeLuminance2(s6, r5, n5));
            for (; l4 < i7 && (o6 > 0 || a5 > 0 || h4 > 0); ) o6 -= Math.max(0, Math.ceil(0.1 * o6)), a5 -= Math.max(0, Math.ceil(0.1 * a5)), h4 -= Math.max(0, Math.ceil(0.1 * h4)), l4 = _2(c4.relativeLuminance2(o6, a5, h4), c4.relativeLuminance2(s6, r5, n5));
            return (o6 << 24 | a5 << 16 | h4 << 8 | 255) >>> 0;
          }
          function a4(e7, t6, i7) {
            const s6 = e7 >> 24 & 255, r5 = e7 >> 16 & 255, n5 = e7 >> 8 & 255;
            let o6 = t6 >> 24 & 255, a5 = t6 >> 16 & 255, h4 = t6 >> 8 & 255, l4 = _2(c4.relativeLuminance2(o6, a5, h4), c4.relativeLuminance2(s6, r5, n5));
            for (; l4 < i7 && (o6 < 255 || a5 < 255 || h4 < 255); ) o6 = Math.min(255, o6 + Math.ceil(0.1 * (255 - o6))), a5 = Math.min(255, a5 + Math.ceil(0.1 * (255 - a5))), h4 = Math.min(255, h4 + Math.ceil(0.1 * (255 - h4))), l4 = _2(c4.relativeLuminance2(o6, a5, h4), c4.relativeLuminance2(s6, r5, n5));
            return (o6 << 24 | a5 << 16 | h4 << 8 | 255) >>> 0;
          }
          e6.blend = function(e7, t6) {
            if (n4 = (255 & t6) / 255, 1 === n4) return t6;
            const a5 = t6 >> 24 & 255, h4 = t6 >> 16 & 255, c5 = t6 >> 8 & 255, l4 = e7 >> 24 & 255, d4 = e7 >> 16 & 255, _3 = e7 >> 8 & 255;
            return i6 = l4 + Math.round((a5 - l4) * n4), s5 = d4 + Math.round((h4 - d4) * n4), r4 = _3 + Math.round((c5 - _3) * n4), o5.toRgba(i6, s5, r4);
          }, e6.ensureContrastRatio = function(e7, i7, s6) {
            const r5 = c4.relativeLuminance(e7 >> 8), n5 = c4.relativeLuminance(i7 >> 8);
            if (_2(r5, n5) < s6) {
              if (n5 < r5) {
                const n6 = t5(e7, i7, s6), o7 = _2(r5, c4.relativeLuminance(n6 >> 8));
                if (o7 < s6) {
                  const t6 = a4(e7, i7, s6);
                  return o7 > _2(r5, c4.relativeLuminance(t6 >> 8)) ? n6 : t6;
                }
                return n6;
              }
              const o6 = a4(e7, i7, s6), h4 = _2(r5, c4.relativeLuminance(o6 >> 8));
              if (h4 < s6) {
                const n6 = t5(e7, i7, s6);
                return h4 > _2(r5, c4.relativeLuminance(n6 >> 8)) ? o6 : n6;
              }
              return o6;
            }
          }, e6.reduceLuminance = t5, e6.increaseLuminance = a4, e6.toChannels = function(e7) {
            return [e7 >> 24 & 255, e7 >> 16 & 255, e7 >> 8 & 255, 255 & e7];
          };
        })(l3 || (t4.rgba = l3 = {})), t4.toPaddedHex = d3, t4.contrastRatio = _2;
      }, 8969: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.CoreTerminal = void 0;
        const s5 = i6(844), r4 = i6(2585), n4 = i6(4348), o5 = i6(7866), a3 = i6(744), h3 = i6(7302), c4 = i6(6975), l3 = i6(8460), d3 = i6(1753), _2 = i6(1480), u3 = i6(7994), f3 = i6(9282), v2 = i6(5435), p3 = i6(5981), g2 = i6(2660);
        let m2 = false;
        class S3 extends s5.Disposable {
          get onScroll() {
            return this._onScrollApi || (this._onScrollApi = this.register(new l3.EventEmitter()), this._onScroll.event(((e6) => {
              this._onScrollApi?.fire(e6.position);
            }))), this._onScrollApi.event;
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
          set options(e6) {
            for (const t5 in e6) this.optionsService.options[t5] = e6[t5];
          }
          constructor(e6) {
            super(), this._windowsWrappingHeuristics = this.register(new s5.MutableDisposable()), this._onBinary = this.register(new l3.EventEmitter()), this.onBinary = this._onBinary.event, this._onData = this.register(new l3.EventEmitter()), this.onData = this._onData.event, this._onLineFeed = this.register(new l3.EventEmitter()), this.onLineFeed = this._onLineFeed.event, this._onResize = this.register(new l3.EventEmitter()), this.onResize = this._onResize.event, this._onWriteParsed = this.register(new l3.EventEmitter()), this.onWriteParsed = this._onWriteParsed.event, this._onScroll = this.register(new l3.EventEmitter()), this._instantiationService = new n4.InstantiationService(), this.optionsService = this.register(new h3.OptionsService(e6)), this._instantiationService.setService(r4.IOptionsService, this.optionsService), this._bufferService = this.register(this._instantiationService.createInstance(a3.BufferService)), this._instantiationService.setService(r4.IBufferService, this._bufferService), this._logService = this.register(this._instantiationService.createInstance(o5.LogService)), this._instantiationService.setService(r4.ILogService, this._logService), this.coreService = this.register(this._instantiationService.createInstance(c4.CoreService)), this._instantiationService.setService(r4.ICoreService, this.coreService), this.coreMouseService = this.register(this._instantiationService.createInstance(d3.CoreMouseService)), this._instantiationService.setService(r4.ICoreMouseService, this.coreMouseService), this.unicodeService = this.register(this._instantiationService.createInstance(_2.UnicodeService)), this._instantiationService.setService(r4.IUnicodeService, this.unicodeService), this._charsetService = this._instantiationService.createInstance(u3.CharsetService), this._instantiationService.setService(r4.ICharsetService, this._charsetService), this._oscLinkService = this._instantiationService.createInstance(g2.OscLinkService), this._instantiationService.setService(r4.IOscLinkService, this._oscLinkService), this._inputHandler = this.register(new v2.InputHandler(this._bufferService, this._charsetService, this.coreService, this._logService, this.optionsService, this._oscLinkService, this.coreMouseService, this.unicodeService)), this.register((0, l3.forwardEvent)(this._inputHandler.onLineFeed, this._onLineFeed)), this.register(this._inputHandler), this.register((0, l3.forwardEvent)(this._bufferService.onResize, this._onResize)), this.register((0, l3.forwardEvent)(this.coreService.onData, this._onData)), this.register((0, l3.forwardEvent)(this.coreService.onBinary, this._onBinary)), this.register(this.coreService.onRequestScrollToBottom((() => this.scrollToBottom()))), this.register(this.coreService.onUserInput((() => this._writeBuffer.handleUserInput()))), this.register(this.optionsService.onMultipleOptionChange(["windowsMode", "windowsPty"], (() => this._handleWindowsPtyOptionChange()))), this.register(this._bufferService.onScroll(((e7) => {
              this._onScroll.fire({ position: this._bufferService.buffer.ydisp, source: 0 }), this._inputHandler.markRangeDirty(this._bufferService.buffer.scrollTop, this._bufferService.buffer.scrollBottom);
            }))), this.register(this._inputHandler.onScroll(((e7) => {
              this._onScroll.fire({ position: this._bufferService.buffer.ydisp, source: 0 }), this._inputHandler.markRangeDirty(this._bufferService.buffer.scrollTop, this._bufferService.buffer.scrollBottom);
            }))), this._writeBuffer = this.register(new p3.WriteBuffer(((e7, t5) => this._inputHandler.parse(e7, t5)))), this.register((0, l3.forwardEvent)(this._writeBuffer.onWriteParsed, this._onWriteParsed));
          }
          write(e6, t5) {
            this._writeBuffer.write(e6, t5);
          }
          writeSync(e6, t5) {
            this._logService.logLevel <= r4.LogLevelEnum.WARN && !m2 && (this._logService.warn("writeSync is unreliable and will be removed soon."), m2 = true), this._writeBuffer.writeSync(e6, t5);
          }
          input(e6, t5 = true) {
            this.coreService.triggerDataEvent(e6, t5);
          }
          resize(e6, t5) {
            isNaN(e6) || isNaN(t5) || (e6 = Math.max(e6, a3.MINIMUM_COLS), t5 = Math.max(t5, a3.MINIMUM_ROWS), this._bufferService.resize(e6, t5));
          }
          scroll(e6, t5 = false) {
            this._bufferService.scroll(e6, t5);
          }
          scrollLines(e6, t5, i7) {
            this._bufferService.scrollLines(e6, t5, i7);
          }
          scrollPages(e6) {
            this.scrollLines(e6 * (this.rows - 1));
          }
          scrollToTop() {
            this.scrollLines(-this._bufferService.buffer.ydisp);
          }
          scrollToBottom() {
            this.scrollLines(this._bufferService.buffer.ybase - this._bufferService.buffer.ydisp);
          }
          scrollToLine(e6) {
            const t5 = e6 - this._bufferService.buffer.ydisp;
            0 !== t5 && this.scrollLines(t5);
          }
          registerEscHandler(e6, t5) {
            return this._inputHandler.registerEscHandler(e6, t5);
          }
          registerDcsHandler(e6, t5) {
            return this._inputHandler.registerDcsHandler(e6, t5);
          }
          registerCsiHandler(e6, t5) {
            return this._inputHandler.registerCsiHandler(e6, t5);
          }
          registerOscHandler(e6, t5) {
            return this._inputHandler.registerOscHandler(e6, t5);
          }
          _setup() {
            this._handleWindowsPtyOptionChange();
          }
          reset() {
            this._inputHandler.reset(), this._bufferService.reset(), this._charsetService.reset(), this.coreService.reset(), this.coreMouseService.reset();
          }
          _handleWindowsPtyOptionChange() {
            let e6 = false;
            const t5 = this.optionsService.rawOptions.windowsPty;
            t5 && void 0 !== t5.buildNumber && void 0 !== t5.buildNumber ? e6 = !!("conpty" === t5.backend && t5.buildNumber < 21376) : this.optionsService.rawOptions.windowsMode && (e6 = true), e6 ? this._enableWindowsWrappingHeuristics() : this._windowsWrappingHeuristics.clear();
          }
          _enableWindowsWrappingHeuristics() {
            if (!this._windowsWrappingHeuristics.value) {
              const e6 = [];
              e6.push(this.onLineFeed(f3.updateWindowsModeWrappedState.bind(null, this._bufferService))), e6.push(this.registerCsiHandler({ final: "H" }, (() => ((0, f3.updateWindowsModeWrappedState)(this._bufferService), false)))), this._windowsWrappingHeuristics.value = (0, s5.toDisposable)((() => {
                for (const t5 of e6) t5.dispose();
              }));
            }
          }
        }
        t4.CoreTerminal = S3;
      }, 8460: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.runAndSubscribe = t4.forwardEvent = t4.EventEmitter = void 0, t4.EventEmitter = class {
          constructor() {
            this._listeners = [], this._disposed = false;
          }
          get event() {
            return this._event || (this._event = (e6) => (this._listeners.push(e6), { dispose: () => {
              if (!this._disposed) {
                for (let t5 = 0; t5 < this._listeners.length; t5++) if (this._listeners[t5] === e6) return void this._listeners.splice(t5, 1);
              }
            } })), this._event;
          }
          fire(e6, t5) {
            const i6 = [];
            for (let e7 = 0; e7 < this._listeners.length; e7++) i6.push(this._listeners[e7]);
            for (let s5 = 0; s5 < i6.length; s5++) i6[s5].call(void 0, e6, t5);
          }
          dispose() {
            this.clearListeners(), this._disposed = true;
          }
          clearListeners() {
            this._listeners && (this._listeners.length = 0);
          }
        }, t4.forwardEvent = function(e6, t5) {
          return e6(((e7) => t5.fire(e7)));
        }, t4.runAndSubscribe = function(e6, t5) {
          return t5(void 0), e6(((e7) => t5(e7)));
        };
      }, 5435: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.InputHandler = t4.WindowsOptionsReportType = void 0;
        const n4 = i6(2584), o5 = i6(7116), a3 = i6(2015), h3 = i6(844), c4 = i6(482), l3 = i6(8437), d3 = i6(8460), _2 = i6(643), u3 = i6(511), f3 = i6(3734), v2 = i6(2585), p3 = i6(1480), g2 = i6(6242), m2 = i6(6351), S3 = i6(5941), C2 = { "(": 0, ")": 1, "*": 2, "+": 3, "-": 1, ".": 2 }, b3 = 131072;
        function w2(e6, t5) {
          if (e6 > 24) return t5.setWinLines || false;
          switch (e6) {
            case 1:
              return !!t5.restoreWin;
            case 2:
              return !!t5.minimizeWin;
            case 3:
              return !!t5.setWinPosition;
            case 4:
              return !!t5.setWinSizePixels;
            case 5:
              return !!t5.raiseWin;
            case 6:
              return !!t5.lowerWin;
            case 7:
              return !!t5.refreshWin;
            case 8:
              return !!t5.setWinSizeChars;
            case 9:
              return !!t5.maximizeWin;
            case 10:
              return !!t5.fullscreenWin;
            case 11:
              return !!t5.getWinState;
            case 13:
              return !!t5.getWinPosition;
            case 14:
              return !!t5.getWinSizePixels;
            case 15:
              return !!t5.getScreenSizePixels;
            case 16:
              return !!t5.getCellSizePixels;
            case 18:
              return !!t5.getWinSizeChars;
            case 19:
              return !!t5.getScreenSizeChars;
            case 20:
              return !!t5.getIconTitle;
            case 21:
              return !!t5.getWinTitle;
            case 22:
              return !!t5.pushTitle;
            case 23:
              return !!t5.popTitle;
            case 24:
              return !!t5.setWinLines;
          }
          return false;
        }
        var y3;
        !(function(e6) {
          e6[e6.GET_WIN_SIZE_PIXELS = 0] = "GET_WIN_SIZE_PIXELS", e6[e6.GET_CELL_SIZE_PIXELS = 1] = "GET_CELL_SIZE_PIXELS";
        })(y3 || (t4.WindowsOptionsReportType = y3 = {}));
        let E2 = 0;
        class k2 extends h3.Disposable {
          getAttrData() {
            return this._curAttrData;
          }
          constructor(e6, t5, i7, s6, r5, h4, _3, f4, v3 = new a3.EscapeSequenceParser()) {
            super(), this._bufferService = e6, this._charsetService = t5, this._coreService = i7, this._logService = s6, this._optionsService = r5, this._oscLinkService = h4, this._coreMouseService = _3, this._unicodeService = f4, this._parser = v3, this._parseBuffer = new Uint32Array(4096), this._stringDecoder = new c4.StringToUtf32(), this._utf8Decoder = new c4.Utf8ToUtf32(), this._workCell = new u3.CellData(), this._windowTitle = "", this._iconName = "", this._windowTitleStack = [], this._iconNameStack = [], this._curAttrData = l3.DEFAULT_ATTR_DATA.clone(), this._eraseAttrDataInternal = l3.DEFAULT_ATTR_DATA.clone(), this._onRequestBell = this.register(new d3.EventEmitter()), this.onRequestBell = this._onRequestBell.event, this._onRequestRefreshRows = this.register(new d3.EventEmitter()), this.onRequestRefreshRows = this._onRequestRefreshRows.event, this._onRequestReset = this.register(new d3.EventEmitter()), this.onRequestReset = this._onRequestReset.event, this._onRequestSendFocus = this.register(new d3.EventEmitter()), this.onRequestSendFocus = this._onRequestSendFocus.event, this._onRequestSyncScrollBar = this.register(new d3.EventEmitter()), this.onRequestSyncScrollBar = this._onRequestSyncScrollBar.event, this._onRequestWindowsOptionsReport = this.register(new d3.EventEmitter()), this.onRequestWindowsOptionsReport = this._onRequestWindowsOptionsReport.event, this._onA11yChar = this.register(new d3.EventEmitter()), this.onA11yChar = this._onA11yChar.event, this._onA11yTab = this.register(new d3.EventEmitter()), this.onA11yTab = this._onA11yTab.event, this._onCursorMove = this.register(new d3.EventEmitter()), this.onCursorMove = this._onCursorMove.event, this._onLineFeed = this.register(new d3.EventEmitter()), this.onLineFeed = this._onLineFeed.event, this._onScroll = this.register(new d3.EventEmitter()), this.onScroll = this._onScroll.event, this._onTitleChange = this.register(new d3.EventEmitter()), this.onTitleChange = this._onTitleChange.event, this._onColor = this.register(new d3.EventEmitter()), this.onColor = this._onColor.event, this._parseStack = { paused: false, cursorStartX: 0, cursorStartY: 0, decodedLength: 0, position: 0 }, this._specialColors = [256, 257, 258], this.register(this._parser), this._dirtyRowTracker = new L2(this._bufferService), this._activeBuffer = this._bufferService.buffer, this.register(this._bufferService.buffers.onBufferActivate(((e7) => this._activeBuffer = e7.activeBuffer))), this._parser.setCsiHandlerFallback(((e7, t6) => {
              this._logService.debug("Unknown CSI code: ", { identifier: this._parser.identToString(e7), params: t6.toArray() });
            })), this._parser.setEscHandlerFallback(((e7) => {
              this._logService.debug("Unknown ESC code: ", { identifier: this._parser.identToString(e7) });
            })), this._parser.setExecuteHandlerFallback(((e7) => {
              this._logService.debug("Unknown EXECUTE code: ", { code: e7 });
            })), this._parser.setOscHandlerFallback(((e7, t6, i8) => {
              this._logService.debug("Unknown OSC code: ", { identifier: e7, action: t6, data: i8 });
            })), this._parser.setDcsHandlerFallback(((e7, t6, i8) => {
              "HOOK" === t6 && (i8 = i8.toArray()), this._logService.debug("Unknown DCS code: ", { identifier: this._parser.identToString(e7), action: t6, payload: i8 });
            })), this._parser.setPrintHandler(((e7, t6, i8) => this.print(e7, t6, i8))), this._parser.registerCsiHandler({ final: "@" }, ((e7) => this.insertChars(e7))), this._parser.registerCsiHandler({ intermediates: " ", final: "@" }, ((e7) => this.scrollLeft(e7))), this._parser.registerCsiHandler({ final: "A" }, ((e7) => this.cursorUp(e7))), this._parser.registerCsiHandler({ intermediates: " ", final: "A" }, ((e7) => this.scrollRight(e7))), this._parser.registerCsiHandler({ final: "B" }, ((e7) => this.cursorDown(e7))), this._parser.registerCsiHandler({ final: "C" }, ((e7) => this.cursorForward(e7))), this._parser.registerCsiHandler({ final: "D" }, ((e7) => this.cursorBackward(e7))), this._parser.registerCsiHandler({ final: "E" }, ((e7) => this.cursorNextLine(e7))), this._parser.registerCsiHandler({ final: "F" }, ((e7) => this.cursorPrecedingLine(e7))), this._parser.registerCsiHandler({ final: "G" }, ((e7) => this.cursorCharAbsolute(e7))), this._parser.registerCsiHandler({ final: "H" }, ((e7) => this.cursorPosition(e7))), this._parser.registerCsiHandler({ final: "I" }, ((e7) => this.cursorForwardTab(e7))), this._parser.registerCsiHandler({ final: "J" }, ((e7) => this.eraseInDisplay(e7, false))), this._parser.registerCsiHandler({ prefix: "?", final: "J" }, ((e7) => this.eraseInDisplay(e7, true))), this._parser.registerCsiHandler({ final: "K" }, ((e7) => this.eraseInLine(e7, false))), this._parser.registerCsiHandler({ prefix: "?", final: "K" }, ((e7) => this.eraseInLine(e7, true))), this._parser.registerCsiHandler({ final: "L" }, ((e7) => this.insertLines(e7))), this._parser.registerCsiHandler({ final: "M" }, ((e7) => this.deleteLines(e7))), this._parser.registerCsiHandler({ final: "P" }, ((e7) => this.deleteChars(e7))), this._parser.registerCsiHandler({ final: "S" }, ((e7) => this.scrollUp(e7))), this._parser.registerCsiHandler({ final: "T" }, ((e7) => this.scrollDown(e7))), this._parser.registerCsiHandler({ final: "X" }, ((e7) => this.eraseChars(e7))), this._parser.registerCsiHandler({ final: "Z" }, ((e7) => this.cursorBackwardTab(e7))), this._parser.registerCsiHandler({ final: "`" }, ((e7) => this.charPosAbsolute(e7))), this._parser.registerCsiHandler({ final: "a" }, ((e7) => this.hPositionRelative(e7))), this._parser.registerCsiHandler({ final: "b" }, ((e7) => this.repeatPrecedingCharacter(e7))), this._parser.registerCsiHandler({ final: "c" }, ((e7) => this.sendDeviceAttributesPrimary(e7))), this._parser.registerCsiHandler({ prefix: ">", final: "c" }, ((e7) => this.sendDeviceAttributesSecondary(e7))), this._parser.registerCsiHandler({ final: "d" }, ((e7) => this.linePosAbsolute(e7))), this._parser.registerCsiHandler({ final: "e" }, ((e7) => this.vPositionRelative(e7))), this._parser.registerCsiHandler({ final: "f" }, ((e7) => this.hVPosition(e7))), this._parser.registerCsiHandler({ final: "g" }, ((e7) => this.tabClear(e7))), this._parser.registerCsiHandler({ final: "h" }, ((e7) => this.setMode(e7))), this._parser.registerCsiHandler({ prefix: "?", final: "h" }, ((e7) => this.setModePrivate(e7))), this._parser.registerCsiHandler({ final: "l" }, ((e7) => this.resetMode(e7))), this._parser.registerCsiHandler({ prefix: "?", final: "l" }, ((e7) => this.resetModePrivate(e7))), this._parser.registerCsiHandler({ final: "m" }, ((e7) => this.charAttributes(e7))), this._parser.registerCsiHandler({ final: "n" }, ((e7) => this.deviceStatus(e7))), this._parser.registerCsiHandler({ prefix: "?", final: "n" }, ((e7) => this.deviceStatusPrivate(e7))), this._parser.registerCsiHandler({ intermediates: "!", final: "p" }, ((e7) => this.softReset(e7))), this._parser.registerCsiHandler({ intermediates: " ", final: "q" }, ((e7) => this.setCursorStyle(e7))), this._parser.registerCsiHandler({ final: "r" }, ((e7) => this.setScrollRegion(e7))), this._parser.registerCsiHandler({ final: "s" }, ((e7) => this.saveCursor(e7))), this._parser.registerCsiHandler({ final: "t" }, ((e7) => this.windowOptions(e7))), this._parser.registerCsiHandler({ final: "u" }, ((e7) => this.restoreCursor(e7))), this._parser.registerCsiHandler({ intermediates: "'", final: "}" }, ((e7) => this.insertColumns(e7))), this._parser.registerCsiHandler({ intermediates: "'", final: "~" }, ((e7) => this.deleteColumns(e7))), this._parser.registerCsiHandler({ intermediates: '"', final: "q" }, ((e7) => this.selectProtected(e7))), this._parser.registerCsiHandler({ intermediates: "$", final: "p" }, ((e7) => this.requestMode(e7, true))), this._parser.registerCsiHandler({ prefix: "?", intermediates: "$", final: "p" }, ((e7) => this.requestMode(e7, false))), this._parser.setExecuteHandler(n4.C0.BEL, (() => this.bell())), this._parser.setExecuteHandler(n4.C0.LF, (() => this.lineFeed())), this._parser.setExecuteHandler(n4.C0.VT, (() => this.lineFeed())), this._parser.setExecuteHandler(n4.C0.FF, (() => this.lineFeed())), this._parser.setExecuteHandler(n4.C0.CR, (() => this.carriageReturn())), this._parser.setExecuteHandler(n4.C0.BS, (() => this.backspace())), this._parser.setExecuteHandler(n4.C0.HT, (() => this.tab())), this._parser.setExecuteHandler(n4.C0.SO, (() => this.shiftOut())), this._parser.setExecuteHandler(n4.C0.SI, (() => this.shiftIn())), this._parser.setExecuteHandler(n4.C1.IND, (() => this.index())), this._parser.setExecuteHandler(n4.C1.NEL, (() => this.nextLine())), this._parser.setExecuteHandler(n4.C1.HTS, (() => this.tabSet())), this._parser.registerOscHandler(0, new g2.OscHandler(((e7) => (this.setTitle(e7), this.setIconName(e7), true)))), this._parser.registerOscHandler(1, new g2.OscHandler(((e7) => this.setIconName(e7)))), this._parser.registerOscHandler(2, new g2.OscHandler(((e7) => this.setTitle(e7)))), this._parser.registerOscHandler(4, new g2.OscHandler(((e7) => this.setOrReportIndexedColor(e7)))), this._parser.registerOscHandler(8, new g2.OscHandler(((e7) => this.setHyperlink(e7)))), this._parser.registerOscHandler(10, new g2.OscHandler(((e7) => this.setOrReportFgColor(e7)))), this._parser.registerOscHandler(11, new g2.OscHandler(((e7) => this.setOrReportBgColor(e7)))), this._parser.registerOscHandler(12, new g2.OscHandler(((e7) => this.setOrReportCursorColor(e7)))), this._parser.registerOscHandler(104, new g2.OscHandler(((e7) => this.restoreIndexedColor(e7)))), this._parser.registerOscHandler(110, new g2.OscHandler(((e7) => this.restoreFgColor(e7)))), this._parser.registerOscHandler(111, new g2.OscHandler(((e7) => this.restoreBgColor(e7)))), this._parser.registerOscHandler(112, new g2.OscHandler(((e7) => this.restoreCursorColor(e7)))), this._parser.registerEscHandler({ final: "7" }, (() => this.saveCursor())), this._parser.registerEscHandler({ final: "8" }, (() => this.restoreCursor())), this._parser.registerEscHandler({ final: "D" }, (() => this.index())), this._parser.registerEscHandler({ final: "E" }, (() => this.nextLine())), this._parser.registerEscHandler({ final: "H" }, (() => this.tabSet())), this._parser.registerEscHandler({ final: "M" }, (() => this.reverseIndex())), this._parser.registerEscHandler({ final: "=" }, (() => this.keypadApplicationMode())), this._parser.registerEscHandler({ final: ">" }, (() => this.keypadNumericMode())), this._parser.registerEscHandler({ final: "c" }, (() => this.fullReset())), this._parser.registerEscHandler({ final: "n" }, (() => this.setgLevel(2))), this._parser.registerEscHandler({ final: "o" }, (() => this.setgLevel(3))), this._parser.registerEscHandler({ final: "|" }, (() => this.setgLevel(3))), this._parser.registerEscHandler({ final: "}" }, (() => this.setgLevel(2))), this._parser.registerEscHandler({ final: "~" }, (() => this.setgLevel(1))), this._parser.registerEscHandler({ intermediates: "%", final: "@" }, (() => this.selectDefaultCharset())), this._parser.registerEscHandler({ intermediates: "%", final: "G" }, (() => this.selectDefaultCharset()));
            for (const e7 in o5.CHARSETS) this._parser.registerEscHandler({ intermediates: "(", final: e7 }, (() => this.selectCharset("(" + e7))), this._parser.registerEscHandler({ intermediates: ")", final: e7 }, (() => this.selectCharset(")" + e7))), this._parser.registerEscHandler({ intermediates: "*", final: e7 }, (() => this.selectCharset("*" + e7))), this._parser.registerEscHandler({ intermediates: "+", final: e7 }, (() => this.selectCharset("+" + e7))), this._parser.registerEscHandler({ intermediates: "-", final: e7 }, (() => this.selectCharset("-" + e7))), this._parser.registerEscHandler({ intermediates: ".", final: e7 }, (() => this.selectCharset("." + e7))), this._parser.registerEscHandler({ intermediates: "/", final: e7 }, (() => this.selectCharset("/" + e7)));
            this._parser.registerEscHandler({ intermediates: "#", final: "8" }, (() => this.screenAlignmentPattern())), this._parser.setErrorHandler(((e7) => (this._logService.error("Parsing error: ", e7), e7))), this._parser.registerDcsHandler({ intermediates: "$", final: "q" }, new m2.DcsHandler(((e7, t6) => this.requestStatusString(e7, t6))));
          }
          _preserveStack(e6, t5, i7, s6) {
            this._parseStack.paused = true, this._parseStack.cursorStartX = e6, this._parseStack.cursorStartY = t5, this._parseStack.decodedLength = i7, this._parseStack.position = s6;
          }
          _logSlowResolvingAsync(e6) {
            this._logService.logLevel <= v2.LogLevelEnum.WARN && Promise.race([e6, new Promise(((e7, t5) => setTimeout((() => t5("#SLOW_TIMEOUT")), 5e3)))]).catch(((e7) => {
              if ("#SLOW_TIMEOUT" !== e7) throw e7;
              console.warn("async parser handler taking longer than 5000 ms");
            }));
          }
          _getCurrentLinkId() {
            return this._curAttrData.extended.urlId;
          }
          parse(e6, t5) {
            let i7, s6 = this._activeBuffer.x, r5 = this._activeBuffer.y, n5 = 0;
            const o6 = this._parseStack.paused;
            if (o6) {
              if (i7 = this._parser.parse(this._parseBuffer, this._parseStack.decodedLength, t5)) return this._logSlowResolvingAsync(i7), i7;
              s6 = this._parseStack.cursorStartX, r5 = this._parseStack.cursorStartY, this._parseStack.paused = false, e6.length > b3 && (n5 = this._parseStack.position + b3);
            }
            if (this._logService.logLevel <= v2.LogLevelEnum.DEBUG && this._logService.debug("parsing data" + ("string" == typeof e6 ? ` "${e6}"` : ` "${Array.prototype.map.call(e6, ((e7) => String.fromCharCode(e7))).join("")}"`), "string" == typeof e6 ? e6.split("").map(((e7) => e7.charCodeAt(0))) : e6), this._parseBuffer.length < e6.length && this._parseBuffer.length < b3 && (this._parseBuffer = new Uint32Array(Math.min(e6.length, b3))), o6 || this._dirtyRowTracker.clearRange(), e6.length > b3) for (let t6 = n5; t6 < e6.length; t6 += b3) {
              const n6 = t6 + b3 < e6.length ? t6 + b3 : e6.length, o7 = "string" == typeof e6 ? this._stringDecoder.decode(e6.substring(t6, n6), this._parseBuffer) : this._utf8Decoder.decode(e6.subarray(t6, n6), this._parseBuffer);
              if (i7 = this._parser.parse(this._parseBuffer, o7)) return this._preserveStack(s6, r5, o7, t6), this._logSlowResolvingAsync(i7), i7;
            }
            else if (!o6) {
              const t6 = "string" == typeof e6 ? this._stringDecoder.decode(e6, this._parseBuffer) : this._utf8Decoder.decode(e6, this._parseBuffer);
              if (i7 = this._parser.parse(this._parseBuffer, t6)) return this._preserveStack(s6, r5, t6, 0), this._logSlowResolvingAsync(i7), i7;
            }
            this._activeBuffer.x === s6 && this._activeBuffer.y === r5 || this._onCursorMove.fire();
            const a4 = this._dirtyRowTracker.end + (this._bufferService.buffer.ybase - this._bufferService.buffer.ydisp), h4 = this._dirtyRowTracker.start + (this._bufferService.buffer.ybase - this._bufferService.buffer.ydisp);
            h4 < this._bufferService.rows && this._onRequestRefreshRows.fire(Math.min(h4, this._bufferService.rows - 1), Math.min(a4, this._bufferService.rows - 1));
          }
          print(e6, t5, i7) {
            let s6, r5;
            const n5 = this._charsetService.charset, o6 = this._optionsService.rawOptions.screenReaderMode, a4 = this._bufferService.cols, h4 = this._coreService.decPrivateModes.wraparound, d4 = this._coreService.modes.insertMode, u4 = this._curAttrData;
            let f4 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y);
            this._dirtyRowTracker.markDirty(this._activeBuffer.y), this._activeBuffer.x && i7 - t5 > 0 && 2 === f4.getWidth(this._activeBuffer.x - 1) && f4.setCellFromCodepoint(this._activeBuffer.x - 1, 0, 1, u4);
            let v3 = this._parser.precedingJoinState;
            for (let g3 = t5; g3 < i7; ++g3) {
              if (s6 = e6[g3], s6 < 127 && n5) {
                const e7 = n5[String.fromCharCode(s6)];
                e7 && (s6 = e7.charCodeAt(0));
              }
              const t6 = this._unicodeService.charProperties(s6, v3);
              r5 = p3.UnicodeService.extractWidth(t6);
              const i8 = p3.UnicodeService.extractShouldJoin(t6), m3 = i8 ? p3.UnicodeService.extractWidth(v3) : 0;
              if (v3 = t6, o6 && this._onA11yChar.fire((0, c4.stringFromCodePoint)(s6)), this._getCurrentLinkId() && this._oscLinkService.addLineToLink(this._getCurrentLinkId(), this._activeBuffer.ybase + this._activeBuffer.y), this._activeBuffer.x + r5 - m3 > a4) {
                if (h4) {
                  const e7 = f4;
                  let t7 = this._activeBuffer.x - m3;
                  for (this._activeBuffer.x = m3, this._activeBuffer.y++, this._activeBuffer.y === this._activeBuffer.scrollBottom + 1 ? (this._activeBuffer.y--, this._bufferService.scroll(this._eraseAttrData(), true)) : (this._activeBuffer.y >= this._bufferService.rows && (this._activeBuffer.y = this._bufferService.rows - 1), this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y).isWrapped = true), f4 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y), m3 > 0 && f4 instanceof l3.BufferLine && f4.copyCellsFrom(e7, t7, 0, m3, false); t7 < a4; ) e7.setCellFromCodepoint(t7++, 0, 1, u4);
                } else if (this._activeBuffer.x = a4 - 1, 2 === r5) continue;
              }
              if (i8 && this._activeBuffer.x) {
                const e7 = f4.getWidth(this._activeBuffer.x - 1) ? 1 : 2;
                f4.addCodepointToCell(this._activeBuffer.x - e7, s6, r5);
                for (let e8 = r5 - m3; --e8 >= 0; ) f4.setCellFromCodepoint(this._activeBuffer.x++, 0, 0, u4);
              } else if (d4 && (f4.insertCells(this._activeBuffer.x, r5 - m3, this._activeBuffer.getNullCell(u4)), 2 === f4.getWidth(a4 - 1) && f4.setCellFromCodepoint(a4 - 1, _2.NULL_CELL_CODE, _2.NULL_CELL_WIDTH, u4)), f4.setCellFromCodepoint(this._activeBuffer.x++, s6, r5, u4), r5 > 0) for (; --r5; ) f4.setCellFromCodepoint(this._activeBuffer.x++, 0, 0, u4);
            }
            this._parser.precedingJoinState = v3, this._activeBuffer.x < a4 && i7 - t5 > 0 && 0 === f4.getWidth(this._activeBuffer.x) && !f4.hasContent(this._activeBuffer.x) && f4.setCellFromCodepoint(this._activeBuffer.x, 0, 1, u4), this._dirtyRowTracker.markDirty(this._activeBuffer.y);
          }
          registerCsiHandler(e6, t5) {
            return "t" !== e6.final || e6.prefix || e6.intermediates ? this._parser.registerCsiHandler(e6, t5) : this._parser.registerCsiHandler(e6, ((e7) => !w2(e7.params[0], this._optionsService.rawOptions.windowOptions) || t5(e7)));
          }
          registerDcsHandler(e6, t5) {
            return this._parser.registerDcsHandler(e6, new m2.DcsHandler(t5));
          }
          registerEscHandler(e6, t5) {
            return this._parser.registerEscHandler(e6, t5);
          }
          registerOscHandler(e6, t5) {
            return this._parser.registerOscHandler(e6, new g2.OscHandler(t5));
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
            else if (0 === this._activeBuffer.x && this._activeBuffer.y > this._activeBuffer.scrollTop && this._activeBuffer.y <= this._activeBuffer.scrollBottom && this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y)?.isWrapped) {
              this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y).isWrapped = false, this._activeBuffer.y--, this._activeBuffer.x = this._bufferService.cols - 1;
              const e6 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y);
              e6.hasWidth(this._activeBuffer.x) && !e6.hasContent(this._activeBuffer.x) && this._activeBuffer.x--;
            }
            return this._restrictCursor(), true;
          }
          tab() {
            if (this._activeBuffer.x >= this._bufferService.cols) return true;
            const e6 = this._activeBuffer.x;
            return this._activeBuffer.x = this._activeBuffer.nextStop(), this._optionsService.rawOptions.screenReaderMode && this._onA11yTab.fire(this._activeBuffer.x - e6), true;
          }
          shiftOut() {
            return this._charsetService.setgLevel(1), true;
          }
          shiftIn() {
            return this._charsetService.setgLevel(0), true;
          }
          _restrictCursor(e6 = this._bufferService.cols - 1) {
            this._activeBuffer.x = Math.min(e6, Math.max(0, this._activeBuffer.x)), this._activeBuffer.y = this._coreService.decPrivateModes.origin ? Math.min(this._activeBuffer.scrollBottom, Math.max(this._activeBuffer.scrollTop, this._activeBuffer.y)) : Math.min(this._bufferService.rows - 1, Math.max(0, this._activeBuffer.y)), this._dirtyRowTracker.markDirty(this._activeBuffer.y);
          }
          _setCursor(e6, t5) {
            this._dirtyRowTracker.markDirty(this._activeBuffer.y), this._coreService.decPrivateModes.origin ? (this._activeBuffer.x = e6, this._activeBuffer.y = this._activeBuffer.scrollTop + t5) : (this._activeBuffer.x = e6, this._activeBuffer.y = t5), this._restrictCursor(), this._dirtyRowTracker.markDirty(this._activeBuffer.y);
          }
          _moveCursor(e6, t5) {
            this._restrictCursor(), this._setCursor(this._activeBuffer.x + e6, this._activeBuffer.y + t5);
          }
          cursorUp(e6) {
            const t5 = this._activeBuffer.y - this._activeBuffer.scrollTop;
            return t5 >= 0 ? this._moveCursor(0, -Math.min(t5, e6.params[0] || 1)) : this._moveCursor(0, -(e6.params[0] || 1)), true;
          }
          cursorDown(e6) {
            const t5 = this._activeBuffer.scrollBottom - this._activeBuffer.y;
            return t5 >= 0 ? this._moveCursor(0, Math.min(t5, e6.params[0] || 1)) : this._moveCursor(0, e6.params[0] || 1), true;
          }
          cursorForward(e6) {
            return this._moveCursor(e6.params[0] || 1, 0), true;
          }
          cursorBackward(e6) {
            return this._moveCursor(-(e6.params[0] || 1), 0), true;
          }
          cursorNextLine(e6) {
            return this.cursorDown(e6), this._activeBuffer.x = 0, true;
          }
          cursorPrecedingLine(e6) {
            return this.cursorUp(e6), this._activeBuffer.x = 0, true;
          }
          cursorCharAbsolute(e6) {
            return this._setCursor((e6.params[0] || 1) - 1, this._activeBuffer.y), true;
          }
          cursorPosition(e6) {
            return this._setCursor(e6.length >= 2 ? (e6.params[1] || 1) - 1 : 0, (e6.params[0] || 1) - 1), true;
          }
          charPosAbsolute(e6) {
            return this._setCursor((e6.params[0] || 1) - 1, this._activeBuffer.y), true;
          }
          hPositionRelative(e6) {
            return this._moveCursor(e6.params[0] || 1, 0), true;
          }
          linePosAbsolute(e6) {
            return this._setCursor(this._activeBuffer.x, (e6.params[0] || 1) - 1), true;
          }
          vPositionRelative(e6) {
            return this._moveCursor(0, e6.params[0] || 1), true;
          }
          hVPosition(e6) {
            return this.cursorPosition(e6), true;
          }
          tabClear(e6) {
            const t5 = e6.params[0];
            return 0 === t5 ? delete this._activeBuffer.tabs[this._activeBuffer.x] : 3 === t5 && (this._activeBuffer.tabs = {}), true;
          }
          cursorForwardTab(e6) {
            if (this._activeBuffer.x >= this._bufferService.cols) return true;
            let t5 = e6.params[0] || 1;
            for (; t5--; ) this._activeBuffer.x = this._activeBuffer.nextStop();
            return true;
          }
          cursorBackwardTab(e6) {
            if (this._activeBuffer.x >= this._bufferService.cols) return true;
            let t5 = e6.params[0] || 1;
            for (; t5--; ) this._activeBuffer.x = this._activeBuffer.prevStop();
            return true;
          }
          selectProtected(e6) {
            const t5 = e6.params[0];
            return 1 === t5 && (this._curAttrData.bg |= 536870912), 2 !== t5 && 0 !== t5 || (this._curAttrData.bg &= -536870913), true;
          }
          _eraseInBufferLine(e6, t5, i7, s6 = false, r5 = false) {
            const n5 = this._activeBuffer.lines.get(this._activeBuffer.ybase + e6);
            n5.replaceCells(t5, i7, this._activeBuffer.getNullCell(this._eraseAttrData()), r5), s6 && (n5.isWrapped = false);
          }
          _resetBufferLine(e6, t5 = false) {
            const i7 = this._activeBuffer.lines.get(this._activeBuffer.ybase + e6);
            i7 && (i7.fill(this._activeBuffer.getNullCell(this._eraseAttrData()), t5), this._bufferService.buffer.clearMarkers(this._activeBuffer.ybase + e6), i7.isWrapped = false);
          }
          eraseInDisplay(e6, t5 = false) {
            let i7;
            switch (this._restrictCursor(this._bufferService.cols), e6.params[0]) {
              case 0:
                for (i7 = this._activeBuffer.y, this._dirtyRowTracker.markDirty(i7), this._eraseInBufferLine(i7++, this._activeBuffer.x, this._bufferService.cols, 0 === this._activeBuffer.x, t5); i7 < this._bufferService.rows; i7++) this._resetBufferLine(i7, t5);
                this._dirtyRowTracker.markDirty(i7);
                break;
              case 1:
                for (i7 = this._activeBuffer.y, this._dirtyRowTracker.markDirty(i7), this._eraseInBufferLine(i7, 0, this._activeBuffer.x + 1, true, t5), this._activeBuffer.x + 1 >= this._bufferService.cols && (this._activeBuffer.lines.get(i7 + 1).isWrapped = false); i7--; ) this._resetBufferLine(i7, t5);
                this._dirtyRowTracker.markDirty(0);
                break;
              case 2:
                for (i7 = this._bufferService.rows, this._dirtyRowTracker.markDirty(i7 - 1); i7--; ) this._resetBufferLine(i7, t5);
                this._dirtyRowTracker.markDirty(0);
                break;
              case 3:
                const e7 = this._activeBuffer.lines.length - this._bufferService.rows;
                e7 > 0 && (this._activeBuffer.lines.trimStart(e7), this._activeBuffer.ybase = Math.max(this._activeBuffer.ybase - e7, 0), this._activeBuffer.ydisp = Math.max(this._activeBuffer.ydisp - e7, 0), this._onScroll.fire(0));
            }
            return true;
          }
          eraseInLine(e6, t5 = false) {
            switch (this._restrictCursor(this._bufferService.cols), e6.params[0]) {
              case 0:
                this._eraseInBufferLine(this._activeBuffer.y, this._activeBuffer.x, this._bufferService.cols, 0 === this._activeBuffer.x, t5);
                break;
              case 1:
                this._eraseInBufferLine(this._activeBuffer.y, 0, this._activeBuffer.x + 1, false, t5);
                break;
              case 2:
                this._eraseInBufferLine(this._activeBuffer.y, 0, this._bufferService.cols, true, t5);
            }
            return this._dirtyRowTracker.markDirty(this._activeBuffer.y), true;
          }
          insertLines(e6) {
            this._restrictCursor();
            let t5 = e6.params[0] || 1;
            if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
            const i7 = this._activeBuffer.ybase + this._activeBuffer.y, s6 = this._bufferService.rows - 1 - this._activeBuffer.scrollBottom, r5 = this._bufferService.rows - 1 + this._activeBuffer.ybase - s6 + 1;
            for (; t5--; ) this._activeBuffer.lines.splice(r5 - 1, 1), this._activeBuffer.lines.splice(i7, 0, this._activeBuffer.getBlankLine(this._eraseAttrData()));
            return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.y, this._activeBuffer.scrollBottom), this._activeBuffer.x = 0, true;
          }
          deleteLines(e6) {
            this._restrictCursor();
            let t5 = e6.params[0] || 1;
            if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
            const i7 = this._activeBuffer.ybase + this._activeBuffer.y;
            let s6;
            for (s6 = this._bufferService.rows - 1 - this._activeBuffer.scrollBottom, s6 = this._bufferService.rows - 1 + this._activeBuffer.ybase - s6; t5--; ) this._activeBuffer.lines.splice(i7, 1), this._activeBuffer.lines.splice(s6, 0, this._activeBuffer.getBlankLine(this._eraseAttrData()));
            return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.y, this._activeBuffer.scrollBottom), this._activeBuffer.x = 0, true;
          }
          insertChars(e6) {
            this._restrictCursor();
            const t5 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y);
            return t5 && (t5.insertCells(this._activeBuffer.x, e6.params[0] || 1, this._activeBuffer.getNullCell(this._eraseAttrData())), this._dirtyRowTracker.markDirty(this._activeBuffer.y)), true;
          }
          deleteChars(e6) {
            this._restrictCursor();
            const t5 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y);
            return t5 && (t5.deleteCells(this._activeBuffer.x, e6.params[0] || 1, this._activeBuffer.getNullCell(this._eraseAttrData())), this._dirtyRowTracker.markDirty(this._activeBuffer.y)), true;
          }
          scrollUp(e6) {
            let t5 = e6.params[0] || 1;
            for (; t5--; ) this._activeBuffer.lines.splice(this._activeBuffer.ybase + this._activeBuffer.scrollTop, 1), this._activeBuffer.lines.splice(this._activeBuffer.ybase + this._activeBuffer.scrollBottom, 0, this._activeBuffer.getBlankLine(this._eraseAttrData()));
            return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
          }
          scrollDown(e6) {
            let t5 = e6.params[0] || 1;
            for (; t5--; ) this._activeBuffer.lines.splice(this._activeBuffer.ybase + this._activeBuffer.scrollBottom, 1), this._activeBuffer.lines.splice(this._activeBuffer.ybase + this._activeBuffer.scrollTop, 0, this._activeBuffer.getBlankLine(l3.DEFAULT_ATTR_DATA));
            return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
          }
          scrollLeft(e6) {
            if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
            const t5 = e6.params[0] || 1;
            for (let e7 = this._activeBuffer.scrollTop; e7 <= this._activeBuffer.scrollBottom; ++e7) {
              const i7 = this._activeBuffer.lines.get(this._activeBuffer.ybase + e7);
              i7.deleteCells(0, t5, this._activeBuffer.getNullCell(this._eraseAttrData())), i7.isWrapped = false;
            }
            return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
          }
          scrollRight(e6) {
            if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
            const t5 = e6.params[0] || 1;
            for (let e7 = this._activeBuffer.scrollTop; e7 <= this._activeBuffer.scrollBottom; ++e7) {
              const i7 = this._activeBuffer.lines.get(this._activeBuffer.ybase + e7);
              i7.insertCells(0, t5, this._activeBuffer.getNullCell(this._eraseAttrData())), i7.isWrapped = false;
            }
            return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
          }
          insertColumns(e6) {
            if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
            const t5 = e6.params[0] || 1;
            for (let e7 = this._activeBuffer.scrollTop; e7 <= this._activeBuffer.scrollBottom; ++e7) {
              const i7 = this._activeBuffer.lines.get(this._activeBuffer.ybase + e7);
              i7.insertCells(this._activeBuffer.x, t5, this._activeBuffer.getNullCell(this._eraseAttrData())), i7.isWrapped = false;
            }
            return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
          }
          deleteColumns(e6) {
            if (this._activeBuffer.y > this._activeBuffer.scrollBottom || this._activeBuffer.y < this._activeBuffer.scrollTop) return true;
            const t5 = e6.params[0] || 1;
            for (let e7 = this._activeBuffer.scrollTop; e7 <= this._activeBuffer.scrollBottom; ++e7) {
              const i7 = this._activeBuffer.lines.get(this._activeBuffer.ybase + e7);
              i7.deleteCells(this._activeBuffer.x, t5, this._activeBuffer.getNullCell(this._eraseAttrData())), i7.isWrapped = false;
            }
            return this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom), true;
          }
          eraseChars(e6) {
            this._restrictCursor();
            const t5 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y);
            return t5 && (t5.replaceCells(this._activeBuffer.x, this._activeBuffer.x + (e6.params[0] || 1), this._activeBuffer.getNullCell(this._eraseAttrData())), this._dirtyRowTracker.markDirty(this._activeBuffer.y)), true;
          }
          repeatPrecedingCharacter(e6) {
            const t5 = this._parser.precedingJoinState;
            if (!t5) return true;
            const i7 = e6.params[0] || 1, s6 = p3.UnicodeService.extractWidth(t5), r5 = this._activeBuffer.x - s6, n5 = this._activeBuffer.lines.get(this._activeBuffer.ybase + this._activeBuffer.y).getString(r5), o6 = new Uint32Array(n5.length * i7);
            let a4 = 0;
            for (let e7 = 0; e7 < n5.length; ) {
              const t6 = n5.codePointAt(e7) || 0;
              o6[a4++] = t6, e7 += t6 > 65535 ? 2 : 1;
            }
            let h4 = a4;
            for (let e7 = 1; e7 < i7; ++e7) o6.copyWithin(h4, 0, a4), h4 += a4;
            return this.print(o6, 0, h4), true;
          }
          sendDeviceAttributesPrimary(e6) {
            return e6.params[0] > 0 || (this._is("xterm") || this._is("rxvt-unicode") || this._is("screen") ? this._coreService.triggerDataEvent(n4.C0.ESC + "[?1;2c") : this._is("linux") && this._coreService.triggerDataEvent(n4.C0.ESC + "[?6c")), true;
          }
          sendDeviceAttributesSecondary(e6) {
            return e6.params[0] > 0 || (this._is("xterm") ? this._coreService.triggerDataEvent(n4.C0.ESC + "[>0;276;0c") : this._is("rxvt-unicode") ? this._coreService.triggerDataEvent(n4.C0.ESC + "[>85;95;0c") : this._is("linux") ? this._coreService.triggerDataEvent(e6.params[0] + "c") : this._is("screen") && this._coreService.triggerDataEvent(n4.C0.ESC + "[>83;40003;0c")), true;
          }
          _is(e6) {
            return 0 === (this._optionsService.rawOptions.termName + "").indexOf(e6);
          }
          setMode(e6) {
            for (let t5 = 0; t5 < e6.length; t5++) switch (e6.params[t5]) {
              case 4:
                this._coreService.modes.insertMode = true;
                break;
              case 20:
                this._optionsService.options.convertEol = true;
            }
            return true;
          }
          setModePrivate(e6) {
            for (let t5 = 0; t5 < e6.length; t5++) switch (e6.params[t5]) {
              case 1:
                this._coreService.decPrivateModes.applicationCursorKeys = true;
                break;
              case 2:
                this._charsetService.setgCharset(0, o5.DEFAULT_CHARSET), this._charsetService.setgCharset(1, o5.DEFAULT_CHARSET), this._charsetService.setgCharset(2, o5.DEFAULT_CHARSET), this._charsetService.setgCharset(3, o5.DEFAULT_CHARSET);
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
                this._bufferService.buffers.activateAltBuffer(this._eraseAttrData()), this._coreService.isCursorInitialized = true, this._onRequestRefreshRows.fire(0, this._bufferService.rows - 1), this._onRequestSyncScrollBar.fire();
                break;
              case 2004:
                this._coreService.decPrivateModes.bracketedPasteMode = true;
            }
            return true;
          }
          resetMode(e6) {
            for (let t5 = 0; t5 < e6.length; t5++) switch (e6.params[t5]) {
              case 4:
                this._coreService.modes.insertMode = false;
                break;
              case 20:
                this._optionsService.options.convertEol = false;
            }
            return true;
          }
          resetModePrivate(e6) {
            for (let t5 = 0; t5 < e6.length; t5++) switch (e6.params[t5]) {
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
              case 1016:
                this._coreMouseService.activeEncoding = "DEFAULT";
                break;
              case 1015:
                this._logService.debug("DECRST 1015 not supported (see #2507)");
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
                this._bufferService.buffers.activateNormalBuffer(), 1049 === e6.params[t5] && this.restoreCursor(), this._coreService.isCursorInitialized = true, this._onRequestRefreshRows.fire(0, this._bufferService.rows - 1), this._onRequestSyncScrollBar.fire();
                break;
              case 2004:
                this._coreService.decPrivateModes.bracketedPasteMode = false;
            }
            return true;
          }
          requestMode(e6, t5) {
            const i7 = this._coreService.decPrivateModes, { activeProtocol: s6, activeEncoding: r5 } = this._coreMouseService, o6 = this._coreService, { buffers: a4, cols: h4 } = this._bufferService, { active: c5, alt: l4 } = a4, d4 = this._optionsService.rawOptions, _3 = (e7) => e7 ? 1 : 2, u4 = e6.params[0];
            return f4 = u4, v3 = t5 ? 2 === u4 ? 4 : 4 === u4 ? _3(o6.modes.insertMode) : 12 === u4 ? 3 : 20 === u4 ? _3(d4.convertEol) : 0 : 1 === u4 ? _3(i7.applicationCursorKeys) : 3 === u4 ? d4.windowOptions.setWinLines ? 80 === h4 ? 2 : 132 === h4 ? 1 : 0 : 0 : 6 === u4 ? _3(i7.origin) : 7 === u4 ? _3(i7.wraparound) : 8 === u4 ? 3 : 9 === u4 ? _3("X10" === s6) : 12 === u4 ? _3(d4.cursorBlink) : 25 === u4 ? _3(!o6.isCursorHidden) : 45 === u4 ? _3(i7.reverseWraparound) : 66 === u4 ? _3(i7.applicationKeypad) : 67 === u4 ? 4 : 1e3 === u4 ? _3("VT200" === s6) : 1002 === u4 ? _3("DRAG" === s6) : 1003 === u4 ? _3("ANY" === s6) : 1004 === u4 ? _3(i7.sendFocus) : 1005 === u4 ? 4 : 1006 === u4 ? _3("SGR" === r5) : 1015 === u4 ? 4 : 1016 === u4 ? _3("SGR_PIXELS" === r5) : 1048 === u4 ? 1 : 47 === u4 || 1047 === u4 || 1049 === u4 ? _3(c5 === l4) : 2004 === u4 ? _3(i7.bracketedPasteMode) : 0, o6.triggerDataEvent(`${n4.C0.ESC}[${t5 ? "" : "?"}${f4};${v3}$y`), true;
            var f4, v3;
          }
          _updateAttrColor(e6, t5, i7, s6, r5) {
            return 2 === t5 ? (e6 |= 50331648, e6 &= -16777216, e6 |= f3.AttributeData.fromColorRGB([i7, s6, r5])) : 5 === t5 && (e6 &= -50331904, e6 |= 33554432 | 255 & i7), e6;
          }
          _extractColor(e6, t5, i7) {
            const s6 = [0, 0, -1, 0, 0, 0];
            let r5 = 0, n5 = 0;
            do {
              if (s6[n5 + r5] = e6.params[t5 + n5], e6.hasSubParams(t5 + n5)) {
                const i8 = e6.getSubParams(t5 + n5);
                let o6 = 0;
                do {
                  5 === s6[1] && (r5 = 1), s6[n5 + o6 + 1 + r5] = i8[o6];
                } while (++o6 < i8.length && o6 + n5 + 1 + r5 < s6.length);
                break;
              }
              if (5 === s6[1] && n5 + r5 >= 2 || 2 === s6[1] && n5 + r5 >= 5) break;
              s6[1] && (r5 = 1);
            } while (++n5 + t5 < e6.length && n5 + r5 < s6.length);
            for (let e7 = 2; e7 < s6.length; ++e7) -1 === s6[e7] && (s6[e7] = 0);
            switch (s6[0]) {
              case 38:
                i7.fg = this._updateAttrColor(i7.fg, s6[1], s6[3], s6[4], s6[5]);
                break;
              case 48:
                i7.bg = this._updateAttrColor(i7.bg, s6[1], s6[3], s6[4], s6[5]);
                break;
              case 58:
                i7.extended = i7.extended.clone(), i7.extended.underlineColor = this._updateAttrColor(i7.extended.underlineColor, s6[1], s6[3], s6[4], s6[5]);
            }
            return n5;
          }
          _processUnderline(e6, t5) {
            t5.extended = t5.extended.clone(), (!~e6 || e6 > 5) && (e6 = 1), t5.extended.underlineStyle = e6, t5.fg |= 268435456, 0 === e6 && (t5.fg &= -268435457), t5.updateExtended();
          }
          _processSGR0(e6) {
            e6.fg = l3.DEFAULT_ATTR_DATA.fg, e6.bg = l3.DEFAULT_ATTR_DATA.bg, e6.extended = e6.extended.clone(), e6.extended.underlineStyle = 0, e6.extended.underlineColor &= -67108864, e6.updateExtended();
          }
          charAttributes(e6) {
            if (1 === e6.length && 0 === e6.params[0]) return this._processSGR0(this._curAttrData), true;
            const t5 = e6.length;
            let i7;
            const s6 = this._curAttrData;
            for (let r5 = 0; r5 < t5; r5++) i7 = e6.params[r5], i7 >= 30 && i7 <= 37 ? (s6.fg &= -50331904, s6.fg |= 16777216 | i7 - 30) : i7 >= 40 && i7 <= 47 ? (s6.bg &= -50331904, s6.bg |= 16777216 | i7 - 40) : i7 >= 90 && i7 <= 97 ? (s6.fg &= -50331904, s6.fg |= 16777224 | i7 - 90) : i7 >= 100 && i7 <= 107 ? (s6.bg &= -50331904, s6.bg |= 16777224 | i7 - 100) : 0 === i7 ? this._processSGR0(s6) : 1 === i7 ? s6.fg |= 134217728 : 3 === i7 ? s6.bg |= 67108864 : 4 === i7 ? (s6.fg |= 268435456, this._processUnderline(e6.hasSubParams(r5) ? e6.getSubParams(r5)[0] : 1, s6)) : 5 === i7 ? s6.fg |= 536870912 : 7 === i7 ? s6.fg |= 67108864 : 8 === i7 ? s6.fg |= 1073741824 : 9 === i7 ? s6.fg |= 2147483648 : 2 === i7 ? s6.bg |= 134217728 : 21 === i7 ? this._processUnderline(2, s6) : 22 === i7 ? (s6.fg &= -134217729, s6.bg &= -134217729) : 23 === i7 ? s6.bg &= -67108865 : 24 === i7 ? (s6.fg &= -268435457, this._processUnderline(0, s6)) : 25 === i7 ? s6.fg &= -536870913 : 27 === i7 ? s6.fg &= -67108865 : 28 === i7 ? s6.fg &= -1073741825 : 29 === i7 ? s6.fg &= 2147483647 : 39 === i7 ? (s6.fg &= -67108864, s6.fg |= 16777215 & l3.DEFAULT_ATTR_DATA.fg) : 49 === i7 ? (s6.bg &= -67108864, s6.bg |= 16777215 & l3.DEFAULT_ATTR_DATA.bg) : 38 === i7 || 48 === i7 || 58 === i7 ? r5 += this._extractColor(e6, r5, s6) : 53 === i7 ? s6.bg |= 1073741824 : 55 === i7 ? s6.bg &= -1073741825 : 59 === i7 ? (s6.extended = s6.extended.clone(), s6.extended.underlineColor = -1, s6.updateExtended()) : 100 === i7 ? (s6.fg &= -67108864, s6.fg |= 16777215 & l3.DEFAULT_ATTR_DATA.fg, s6.bg &= -67108864, s6.bg |= 16777215 & l3.DEFAULT_ATTR_DATA.bg) : this._logService.debug("Unknown SGR attribute: %d.", i7);
            return true;
          }
          deviceStatus(e6) {
            switch (e6.params[0]) {
              case 5:
                this._coreService.triggerDataEvent(`${n4.C0.ESC}[0n`);
                break;
              case 6:
                const e7 = this._activeBuffer.y + 1, t5 = this._activeBuffer.x + 1;
                this._coreService.triggerDataEvent(`${n4.C0.ESC}[${e7};${t5}R`);
            }
            return true;
          }
          deviceStatusPrivate(e6) {
            if (6 === e6.params[0]) {
              const e7 = this._activeBuffer.y + 1, t5 = this._activeBuffer.x + 1;
              this._coreService.triggerDataEvent(`${n4.C0.ESC}[?${e7};${t5}R`);
            }
            return true;
          }
          softReset(e6) {
            return this._coreService.isCursorHidden = false, this._onRequestSyncScrollBar.fire(), this._activeBuffer.scrollTop = 0, this._activeBuffer.scrollBottom = this._bufferService.rows - 1, this._curAttrData = l3.DEFAULT_ATTR_DATA.clone(), this._coreService.reset(), this._charsetService.reset(), this._activeBuffer.savedX = 0, this._activeBuffer.savedY = this._activeBuffer.ybase, this._activeBuffer.savedCurAttrData.fg = this._curAttrData.fg, this._activeBuffer.savedCurAttrData.bg = this._curAttrData.bg, this._activeBuffer.savedCharset = this._charsetService.charset, this._coreService.decPrivateModes.origin = false, true;
          }
          setCursorStyle(e6) {
            const t5 = e6.params[0] || 1;
            switch (t5) {
              case 1:
              case 2:
                this._optionsService.options.cursorStyle = "block";
                break;
              case 3:
              case 4:
                this._optionsService.options.cursorStyle = "underline";
                break;
              case 5:
              case 6:
                this._optionsService.options.cursorStyle = "bar";
            }
            const i7 = t5 % 2 == 1;
            return this._optionsService.options.cursorBlink = i7, true;
          }
          setScrollRegion(e6) {
            const t5 = e6.params[0] || 1;
            let i7;
            return (e6.length < 2 || (i7 = e6.params[1]) > this._bufferService.rows || 0 === i7) && (i7 = this._bufferService.rows), i7 > t5 && (this._activeBuffer.scrollTop = t5 - 1, this._activeBuffer.scrollBottom = i7 - 1, this._setCursor(0, 0)), true;
          }
          windowOptions(e6) {
            if (!w2(e6.params[0], this._optionsService.rawOptions.windowOptions)) return true;
            const t5 = e6.length > 1 ? e6.params[1] : 0;
            switch (e6.params[0]) {
              case 14:
                2 !== t5 && this._onRequestWindowsOptionsReport.fire(y3.GET_WIN_SIZE_PIXELS);
                break;
              case 16:
                this._onRequestWindowsOptionsReport.fire(y3.GET_CELL_SIZE_PIXELS);
                break;
              case 18:
                this._bufferService && this._coreService.triggerDataEvent(`${n4.C0.ESC}[8;${this._bufferService.rows};${this._bufferService.cols}t`);
                break;
              case 22:
                0 !== t5 && 2 !== t5 || (this._windowTitleStack.push(this._windowTitle), this._windowTitleStack.length > 10 && this._windowTitleStack.shift()), 0 !== t5 && 1 !== t5 || (this._iconNameStack.push(this._iconName), this._iconNameStack.length > 10 && this._iconNameStack.shift());
                break;
              case 23:
                0 !== t5 && 2 !== t5 || this._windowTitleStack.length && this.setTitle(this._windowTitleStack.pop()), 0 !== t5 && 1 !== t5 || this._iconNameStack.length && this.setIconName(this._iconNameStack.pop());
            }
            return true;
          }
          saveCursor(e6) {
            return this._activeBuffer.savedX = this._activeBuffer.x, this._activeBuffer.savedY = this._activeBuffer.ybase + this._activeBuffer.y, this._activeBuffer.savedCurAttrData.fg = this._curAttrData.fg, this._activeBuffer.savedCurAttrData.bg = this._curAttrData.bg, this._activeBuffer.savedCharset = this._charsetService.charset, true;
          }
          restoreCursor(e6) {
            return this._activeBuffer.x = this._activeBuffer.savedX || 0, this._activeBuffer.y = Math.max(this._activeBuffer.savedY - this._activeBuffer.ybase, 0), this._curAttrData.fg = this._activeBuffer.savedCurAttrData.fg, this._curAttrData.bg = this._activeBuffer.savedCurAttrData.bg, this._charsetService.charset = this._savedCharset, this._activeBuffer.savedCharset && (this._charsetService.charset = this._activeBuffer.savedCharset), this._restrictCursor(), true;
          }
          setTitle(e6) {
            return this._windowTitle = e6, this._onTitleChange.fire(e6), true;
          }
          setIconName(e6) {
            return this._iconName = e6, true;
          }
          setOrReportIndexedColor(e6) {
            const t5 = [], i7 = e6.split(";");
            for (; i7.length > 1; ) {
              const e7 = i7.shift(), s6 = i7.shift();
              if (/^\d+$/.exec(e7)) {
                const i8 = parseInt(e7);
                if (D2(i8)) if ("?" === s6) t5.push({ type: 0, index: i8 });
                else {
                  const e8 = (0, S3.parseColor)(s6);
                  e8 && t5.push({ type: 1, index: i8, color: e8 });
                }
              }
            }
            return t5.length && this._onColor.fire(t5), true;
          }
          setHyperlink(e6) {
            const t5 = e6.split(";");
            return !(t5.length < 2) && (t5[1] ? this._createHyperlink(t5[0], t5[1]) : !t5[0] && this._finishHyperlink());
          }
          _createHyperlink(e6, t5) {
            this._getCurrentLinkId() && this._finishHyperlink();
            const i7 = e6.split(":");
            let s6;
            const r5 = i7.findIndex(((e7) => e7.startsWith("id=")));
            return -1 !== r5 && (s6 = i7[r5].slice(3) || void 0), this._curAttrData.extended = this._curAttrData.extended.clone(), this._curAttrData.extended.urlId = this._oscLinkService.registerLink({ id: s6, uri: t5 }), this._curAttrData.updateExtended(), true;
          }
          _finishHyperlink() {
            return this._curAttrData.extended = this._curAttrData.extended.clone(), this._curAttrData.extended.urlId = 0, this._curAttrData.updateExtended(), true;
          }
          _setOrReportSpecialColor(e6, t5) {
            const i7 = e6.split(";");
            for (let e7 = 0; e7 < i7.length && !(t5 >= this._specialColors.length); ++e7, ++t5) if ("?" === i7[e7]) this._onColor.fire([{ type: 0, index: this._specialColors[t5] }]);
            else {
              const s6 = (0, S3.parseColor)(i7[e7]);
              s6 && this._onColor.fire([{ type: 1, index: this._specialColors[t5], color: s6 }]);
            }
            return true;
          }
          setOrReportFgColor(e6) {
            return this._setOrReportSpecialColor(e6, 0);
          }
          setOrReportBgColor(e6) {
            return this._setOrReportSpecialColor(e6, 1);
          }
          setOrReportCursorColor(e6) {
            return this._setOrReportSpecialColor(e6, 2);
          }
          restoreIndexedColor(e6) {
            if (!e6) return this._onColor.fire([{ type: 2 }]), true;
            const t5 = [], i7 = e6.split(";");
            for (let e7 = 0; e7 < i7.length; ++e7) if (/^\d+$/.exec(i7[e7])) {
              const s6 = parseInt(i7[e7]);
              D2(s6) && t5.push({ type: 2, index: s6 });
            }
            return t5.length && this._onColor.fire(t5), true;
          }
          restoreFgColor(e6) {
            return this._onColor.fire([{ type: 2, index: 256 }]), true;
          }
          restoreBgColor(e6) {
            return this._onColor.fire([{ type: 2, index: 257 }]), true;
          }
          restoreCursorColor(e6) {
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
            return this._charsetService.setgLevel(0), this._charsetService.setgCharset(0, o5.DEFAULT_CHARSET), true;
          }
          selectCharset(e6) {
            return 2 !== e6.length ? (this.selectDefaultCharset(), true) : ("/" === e6[0] || this._charsetService.setgCharset(C2[e6[0]], o5.CHARSETS[e6[1]] || o5.DEFAULT_CHARSET), true);
          }
          index() {
            return this._restrictCursor(), this._activeBuffer.y++, this._activeBuffer.y === this._activeBuffer.scrollBottom + 1 ? (this._activeBuffer.y--, this._bufferService.scroll(this._eraseAttrData())) : this._activeBuffer.y >= this._bufferService.rows && (this._activeBuffer.y = this._bufferService.rows - 1), this._restrictCursor(), true;
          }
          tabSet() {
            return this._activeBuffer.tabs[this._activeBuffer.x] = true, true;
          }
          reverseIndex() {
            if (this._restrictCursor(), this._activeBuffer.y === this._activeBuffer.scrollTop) {
              const e6 = this._activeBuffer.scrollBottom - this._activeBuffer.scrollTop;
              this._activeBuffer.lines.shiftElements(this._activeBuffer.ybase + this._activeBuffer.y, e6, 1), this._activeBuffer.lines.set(this._activeBuffer.ybase + this._activeBuffer.y, this._activeBuffer.getBlankLine(this._eraseAttrData())), this._dirtyRowTracker.markRangeDirty(this._activeBuffer.scrollTop, this._activeBuffer.scrollBottom);
            } else this._activeBuffer.y--, this._restrictCursor();
            return true;
          }
          fullReset() {
            return this._parser.reset(), this._onRequestReset.fire(), true;
          }
          reset() {
            this._curAttrData = l3.DEFAULT_ATTR_DATA.clone(), this._eraseAttrDataInternal = l3.DEFAULT_ATTR_DATA.clone();
          }
          _eraseAttrData() {
            return this._eraseAttrDataInternal.bg &= -67108864, this._eraseAttrDataInternal.bg |= 67108863 & this._curAttrData.bg, this._eraseAttrDataInternal;
          }
          setgLevel(e6) {
            return this._charsetService.setgLevel(e6), true;
          }
          screenAlignmentPattern() {
            const e6 = new u3.CellData();
            e6.content = 1 << 22 | "E".charCodeAt(0), e6.fg = this._curAttrData.fg, e6.bg = this._curAttrData.bg, this._setCursor(0, 0);
            for (let t5 = 0; t5 < this._bufferService.rows; ++t5) {
              const i7 = this._activeBuffer.ybase + this._activeBuffer.y + t5, s6 = this._activeBuffer.lines.get(i7);
              s6 && (s6.fill(e6), s6.isWrapped = false);
            }
            return this._dirtyRowTracker.markAllDirty(), this._setCursor(0, 0), true;
          }
          requestStatusString(e6, t5) {
            const i7 = this._bufferService.buffer, s6 = this._optionsService.rawOptions;
            return ((e7) => (this._coreService.triggerDataEvent(`${n4.C0.ESC}${e7}${n4.C0.ESC}\\`), true))('"q' === e6 ? `P1$r${this._curAttrData.isProtected() ? 1 : 0}"q` : '"p' === e6 ? 'P1$r61;1"p' : "r" === e6 ? `P1$r${i7.scrollTop + 1};${i7.scrollBottom + 1}r` : "m" === e6 ? "P1$r0m" : " q" === e6 ? `P1$r${{ block: 2, underline: 4, bar: 6 }[s6.cursorStyle] - (s6.cursorBlink ? 1 : 0)} q` : "P0$r");
          }
          markRangeDirty(e6, t5) {
            this._dirtyRowTracker.markRangeDirty(e6, t5);
          }
        }
        t4.InputHandler = k2;
        let L2 = class {
          constructor(e6) {
            this._bufferService = e6, this.clearRange();
          }
          clearRange() {
            this.start = this._bufferService.buffer.y, this.end = this._bufferService.buffer.y;
          }
          markDirty(e6) {
            e6 < this.start ? this.start = e6 : e6 > this.end && (this.end = e6);
          }
          markRangeDirty(e6, t5) {
            e6 > t5 && (E2 = e6, e6 = t5, t5 = E2), e6 < this.start && (this.start = e6), t5 > this.end && (this.end = t5);
          }
          markAllDirty() {
            this.markRangeDirty(0, this._bufferService.rows - 1);
          }
        };
        function D2(e6) {
          return 0 <= e6 && e6 < 256;
        }
        L2 = s5([r4(0, v2.IBufferService)], L2);
      }, 844: (e5, t4) => {
        function i6(e6) {
          for (const t5 of e6) t5.dispose();
          e6.length = 0;
        }
        Object.defineProperty(t4, "__esModule", { value: true }), t4.getDisposeArrayDisposable = t4.disposeArray = t4.toDisposable = t4.MutableDisposable = t4.Disposable = void 0, t4.Disposable = class {
          constructor() {
            this._disposables = [], this._isDisposed = false;
          }
          dispose() {
            this._isDisposed = true;
            for (const e6 of this._disposables) e6.dispose();
            this._disposables.length = 0;
          }
          register(e6) {
            return this._disposables.push(e6), e6;
          }
          unregister(e6) {
            const t5 = this._disposables.indexOf(e6);
            -1 !== t5 && this._disposables.splice(t5, 1);
          }
        }, t4.MutableDisposable = class {
          constructor() {
            this._isDisposed = false;
          }
          get value() {
            return this._isDisposed ? void 0 : this._value;
          }
          set value(e6) {
            this._isDisposed || e6 === this._value || (this._value?.dispose(), this._value = e6);
          }
          clear() {
            this.value = void 0;
          }
          dispose() {
            this._isDisposed = true, this._value?.dispose(), this._value = void 0;
          }
        }, t4.toDisposable = function(e6) {
          return { dispose: e6 };
        }, t4.disposeArray = i6, t4.getDisposeArrayDisposable = function(e6) {
          return { dispose: () => i6(e6) };
        };
      }, 1505: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.FourKeyMap = t4.TwoKeyMap = void 0;
        class i6 {
          constructor() {
            this._data = {};
          }
          set(e6, t5, i7) {
            this._data[e6] || (this._data[e6] = {}), this._data[e6][t5] = i7;
          }
          get(e6, t5) {
            return this._data[e6] ? this._data[e6][t5] : void 0;
          }
          clear() {
            this._data = {};
          }
        }
        t4.TwoKeyMap = i6, t4.FourKeyMap = class {
          constructor() {
            this._data = new i6();
          }
          set(e6, t5, s5, r4, n4) {
            this._data.get(e6, t5) || this._data.set(e6, t5, new i6()), this._data.get(e6, t5).set(s5, r4, n4);
          }
          get(e6, t5, i7, s5) {
            return this._data.get(e6, t5)?.get(i7, s5);
          }
          clear() {
            this._data.clear();
          }
        };
      }, 6114: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.isChromeOS = t4.isLinux = t4.isWindows = t4.isIphone = t4.isIpad = t4.isMac = t4.getSafariVersion = t4.isSafari = t4.isLegacyEdge = t4.isFirefox = t4.isNode = void 0, t4.isNode = "undefined" != typeof process && "title" in process;
        const i6 = t4.isNode ? "node" : navigator.userAgent, s5 = t4.isNode ? "node" : navigator.platform;
        t4.isFirefox = i6.includes("Firefox"), t4.isLegacyEdge = i6.includes("Edge"), t4.isSafari = /^((?!chrome|android).)*safari/i.test(i6), t4.getSafariVersion = function() {
          if (!t4.isSafari) return 0;
          const e6 = i6.match(/Version\/(\d+)/);
          return null === e6 || e6.length < 2 ? 0 : parseInt(e6[1]);
        }, t4.isMac = ["Macintosh", "MacIntel", "MacPPC", "Mac68K"].includes(s5), t4.isIpad = "iPad" === s5, t4.isIphone = "iPhone" === s5, t4.isWindows = ["Windows", "Win16", "Win32", "WinCE"].includes(s5), t4.isLinux = s5.indexOf("Linux") >= 0, t4.isChromeOS = /\bCrOS\b/.test(i6);
      }, 6106: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.SortedList = void 0;
        let i6 = 0;
        t4.SortedList = class {
          constructor(e6) {
            this._getKey = e6, this._array = [];
          }
          clear() {
            this._array.length = 0;
          }
          insert(e6) {
            0 !== this._array.length ? (i6 = this._search(this._getKey(e6)), this._array.splice(i6, 0, e6)) : this._array.push(e6);
          }
          delete(e6) {
            if (0 === this._array.length) return false;
            const t5 = this._getKey(e6);
            if (void 0 === t5) return false;
            if (i6 = this._search(t5), -1 === i6) return false;
            if (this._getKey(this._array[i6]) !== t5) return false;
            do {
              if (this._array[i6] === e6) return this._array.splice(i6, 1), true;
            } while (++i6 < this._array.length && this._getKey(this._array[i6]) === t5);
            return false;
          }
          *getKeyIterator(e6) {
            if (0 !== this._array.length && (i6 = this._search(e6), !(i6 < 0 || i6 >= this._array.length) && this._getKey(this._array[i6]) === e6)) do {
              yield this._array[i6];
            } while (++i6 < this._array.length && this._getKey(this._array[i6]) === e6);
          }
          forEachByKey(e6, t5) {
            if (0 !== this._array.length && (i6 = this._search(e6), !(i6 < 0 || i6 >= this._array.length) && this._getKey(this._array[i6]) === e6)) do {
              t5(this._array[i6]);
            } while (++i6 < this._array.length && this._getKey(this._array[i6]) === e6);
          }
          values() {
            return [...this._array].values();
          }
          _search(e6) {
            let t5 = 0, i7 = this._array.length - 1;
            for (; i7 >= t5; ) {
              let s5 = t5 + i7 >> 1;
              const r4 = this._getKey(this._array[s5]);
              if (r4 > e6) i7 = s5 - 1;
              else {
                if (!(r4 < e6)) {
                  for (; s5 > 0 && this._getKey(this._array[s5 - 1]) === e6; ) s5--;
                  return s5;
                }
                t5 = s5 + 1;
              }
            }
            return t5;
          }
        };
      }, 7226: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.DebouncedIdleTask = t4.IdleTaskQueue = t4.PriorityTaskQueue = void 0;
        const s5 = i6(6114);
        class r4 {
          constructor() {
            this._tasks = [], this._i = 0;
          }
          enqueue(e6) {
            this._tasks.push(e6), this._start();
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
          _process(e6) {
            this._idleCallback = void 0;
            let t5 = 0, i7 = 0, s6 = e6.timeRemaining(), r5 = 0;
            for (; this._i < this._tasks.length; ) {
              if (t5 = Date.now(), this._tasks[this._i]() || this._i++, t5 = Math.max(1, Date.now() - t5), i7 = Math.max(t5, i7), r5 = e6.timeRemaining(), 1.5 * i7 > r5) return s6 - t5 < -20 && console.warn(`task queue exceeded allotted deadline by ${Math.abs(Math.round(s6 - t5))}ms`), void this._start();
              s6 = r5;
            }
            this.clear();
          }
        }
        class n4 extends r4 {
          _requestCallback(e6) {
            return setTimeout((() => e6(this._createDeadline(16))));
          }
          _cancelCallback(e6) {
            clearTimeout(e6);
          }
          _createDeadline(e6) {
            const t5 = Date.now() + e6;
            return { timeRemaining: () => Math.max(0, t5 - Date.now()) };
          }
        }
        t4.PriorityTaskQueue = n4, t4.IdleTaskQueue = !s5.isNode && "requestIdleCallback" in window ? class extends r4 {
          _requestCallback(e6) {
            return requestIdleCallback(e6);
          }
          _cancelCallback(e6) {
            cancelIdleCallback(e6);
          }
        } : n4, t4.DebouncedIdleTask = class {
          constructor() {
            this._queue = new t4.IdleTaskQueue();
          }
          set(e6) {
            this._queue.clear(), this._queue.enqueue(e6);
          }
          flush() {
            this._queue.flush();
          }
        };
      }, 9282: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.updateWindowsModeWrappedState = void 0;
        const s5 = i6(643);
        t4.updateWindowsModeWrappedState = function(e6) {
          const t5 = e6.buffer.lines.get(e6.buffer.ybase + e6.buffer.y - 1), i7 = t5?.get(e6.cols - 1), r4 = e6.buffer.lines.get(e6.buffer.ybase + e6.buffer.y);
          r4 && i7 && (r4.isWrapped = i7[s5.CHAR_DATA_CODE_INDEX] !== s5.NULL_CELL_CODE && i7[s5.CHAR_DATA_CODE_INDEX] !== s5.WHITESPACE_CELL_CODE);
        };
      }, 3734: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.ExtendedAttrs = t4.AttributeData = void 0;
        class i6 {
          constructor() {
            this.fg = 0, this.bg = 0, this.extended = new s5();
          }
          static toColorRGB(e6) {
            return [e6 >>> 16 & 255, e6 >>> 8 & 255, 255 & e6];
          }
          static fromColorRGB(e6) {
            return (255 & e6[0]) << 16 | (255 & e6[1]) << 8 | 255 & e6[2];
          }
          clone() {
            const e6 = new i6();
            return e6.fg = this.fg, e6.bg = this.bg, e6.extended = this.extended.clone(), e6;
          }
          isInverse() {
            return 67108864 & this.fg;
          }
          isBold() {
            return 134217728 & this.fg;
          }
          isUnderline() {
            return this.hasExtendedAttrs() && 0 !== this.extended.underlineStyle ? 1 : 268435456 & this.fg;
          }
          isBlink() {
            return 536870912 & this.fg;
          }
          isInvisible() {
            return 1073741824 & this.fg;
          }
          isItalic() {
            return 67108864 & this.bg;
          }
          isDim() {
            return 134217728 & this.bg;
          }
          isStrikethrough() {
            return 2147483648 & this.fg;
          }
          isProtected() {
            return 536870912 & this.bg;
          }
          isOverline() {
            return 1073741824 & this.bg;
          }
          getFgColorMode() {
            return 50331648 & this.fg;
          }
          getBgColorMode() {
            return 50331648 & this.bg;
          }
          isFgRGB() {
            return 50331648 == (50331648 & this.fg);
          }
          isBgRGB() {
            return 50331648 == (50331648 & this.bg);
          }
          isFgPalette() {
            return 16777216 == (50331648 & this.fg) || 33554432 == (50331648 & this.fg);
          }
          isBgPalette() {
            return 16777216 == (50331648 & this.bg) || 33554432 == (50331648 & this.bg);
          }
          isFgDefault() {
            return 0 == (50331648 & this.fg);
          }
          isBgDefault() {
            return 0 == (50331648 & this.bg);
          }
          isAttributeDefault() {
            return 0 === this.fg && 0 === this.bg;
          }
          getFgColor() {
            switch (50331648 & this.fg) {
              case 16777216:
              case 33554432:
                return 255 & this.fg;
              case 50331648:
                return 16777215 & this.fg;
              default:
                return -1;
            }
          }
          getBgColor() {
            switch (50331648 & this.bg) {
              case 16777216:
              case 33554432:
                return 255 & this.bg;
              case 50331648:
                return 16777215 & this.bg;
              default:
                return -1;
            }
          }
          hasExtendedAttrs() {
            return 268435456 & this.bg;
          }
          updateExtended() {
            this.extended.isEmpty() ? this.bg &= -268435457 : this.bg |= 268435456;
          }
          getUnderlineColor() {
            if (268435456 & this.bg && ~this.extended.underlineColor) switch (50331648 & this.extended.underlineColor) {
              case 16777216:
              case 33554432:
                return 255 & this.extended.underlineColor;
              case 50331648:
                return 16777215 & this.extended.underlineColor;
              default:
                return this.getFgColor();
            }
            return this.getFgColor();
          }
          getUnderlineColorMode() {
            return 268435456 & this.bg && ~this.extended.underlineColor ? 50331648 & this.extended.underlineColor : this.getFgColorMode();
          }
          isUnderlineColorRGB() {
            return 268435456 & this.bg && ~this.extended.underlineColor ? 50331648 == (50331648 & this.extended.underlineColor) : this.isFgRGB();
          }
          isUnderlineColorPalette() {
            return 268435456 & this.bg && ~this.extended.underlineColor ? 16777216 == (50331648 & this.extended.underlineColor) || 33554432 == (50331648 & this.extended.underlineColor) : this.isFgPalette();
          }
          isUnderlineColorDefault() {
            return 268435456 & this.bg && ~this.extended.underlineColor ? 0 == (50331648 & this.extended.underlineColor) : this.isFgDefault();
          }
          getUnderlineStyle() {
            return 268435456 & this.fg ? 268435456 & this.bg ? this.extended.underlineStyle : 1 : 0;
          }
          getUnderlineVariantOffset() {
            return this.extended.underlineVariantOffset;
          }
        }
        t4.AttributeData = i6;
        class s5 {
          get ext() {
            return this._urlId ? -469762049 & this._ext | this.underlineStyle << 26 : this._ext;
          }
          set ext(e6) {
            this._ext = e6;
          }
          get underlineStyle() {
            return this._urlId ? 5 : (469762048 & this._ext) >> 26;
          }
          set underlineStyle(e6) {
            this._ext &= -469762049, this._ext |= e6 << 26 & 469762048;
          }
          get underlineColor() {
            return 67108863 & this._ext;
          }
          set underlineColor(e6) {
            this._ext &= -67108864, this._ext |= 67108863 & e6;
          }
          get urlId() {
            return this._urlId;
          }
          set urlId(e6) {
            this._urlId = e6;
          }
          get underlineVariantOffset() {
            const e6 = (3758096384 & this._ext) >> 29;
            return e6 < 0 ? 4294967288 ^ e6 : e6;
          }
          set underlineVariantOffset(e6) {
            this._ext &= 536870911, this._ext |= e6 << 29 & 3758096384;
          }
          constructor(e6 = 0, t5 = 0) {
            this._ext = 0, this._urlId = 0, this._ext = e6, this._urlId = t5;
          }
          clone() {
            return new s5(this._ext, this._urlId);
          }
          isEmpty() {
            return 0 === this.underlineStyle && 0 === this._urlId;
          }
        }
        t4.ExtendedAttrs = s5;
      }, 9092: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.Buffer = t4.MAX_BUFFER_SIZE = void 0;
        const s5 = i6(6349), r4 = i6(7226), n4 = i6(3734), o5 = i6(8437), a3 = i6(4634), h3 = i6(511), c4 = i6(643), l3 = i6(4863), d3 = i6(7116);
        t4.MAX_BUFFER_SIZE = 4294967295, t4.Buffer = class {
          constructor(e6, t5, i7) {
            this._hasScrollback = e6, this._optionsService = t5, this._bufferService = i7, this.ydisp = 0, this.ybase = 0, this.y = 0, this.x = 0, this.tabs = {}, this.savedY = 0, this.savedX = 0, this.savedCurAttrData = o5.DEFAULT_ATTR_DATA.clone(), this.savedCharset = d3.DEFAULT_CHARSET, this.markers = [], this._nullCell = h3.CellData.fromCharData([0, c4.NULL_CELL_CHAR, c4.NULL_CELL_WIDTH, c4.NULL_CELL_CODE]), this._whitespaceCell = h3.CellData.fromCharData([0, c4.WHITESPACE_CELL_CHAR, c4.WHITESPACE_CELL_WIDTH, c4.WHITESPACE_CELL_CODE]), this._isClearing = false, this._memoryCleanupQueue = new r4.IdleTaskQueue(), this._memoryCleanupPosition = 0, this._cols = this._bufferService.cols, this._rows = this._bufferService.rows, this.lines = new s5.CircularList(this._getCorrectBufferLength(this._rows)), this.scrollTop = 0, this.scrollBottom = this._rows - 1, this.setupTabStops();
          }
          getNullCell(e6) {
            return e6 ? (this._nullCell.fg = e6.fg, this._nullCell.bg = e6.bg, this._nullCell.extended = e6.extended) : (this._nullCell.fg = 0, this._nullCell.bg = 0, this._nullCell.extended = new n4.ExtendedAttrs()), this._nullCell;
          }
          getWhitespaceCell(e6) {
            return e6 ? (this._whitespaceCell.fg = e6.fg, this._whitespaceCell.bg = e6.bg, this._whitespaceCell.extended = e6.extended) : (this._whitespaceCell.fg = 0, this._whitespaceCell.bg = 0, this._whitespaceCell.extended = new n4.ExtendedAttrs()), this._whitespaceCell;
          }
          getBlankLine(e6, t5) {
            return new o5.BufferLine(this._bufferService.cols, this.getNullCell(e6), t5);
          }
          get hasScrollback() {
            return this._hasScrollback && this.lines.maxLength > this._rows;
          }
          get isCursorInViewport() {
            const e6 = this.ybase + this.y - this.ydisp;
            return e6 >= 0 && e6 < this._rows;
          }
          _getCorrectBufferLength(e6) {
            if (!this._hasScrollback) return e6;
            const i7 = e6 + this._optionsService.rawOptions.scrollback;
            return i7 > t4.MAX_BUFFER_SIZE ? t4.MAX_BUFFER_SIZE : i7;
          }
          fillViewportRows(e6) {
            if (0 === this.lines.length) {
              void 0 === e6 && (e6 = o5.DEFAULT_ATTR_DATA);
              let t5 = this._rows;
              for (; t5--; ) this.lines.push(this.getBlankLine(e6));
            }
          }
          clear() {
            this.ydisp = 0, this.ybase = 0, this.y = 0, this.x = 0, this.lines = new s5.CircularList(this._getCorrectBufferLength(this._rows)), this.scrollTop = 0, this.scrollBottom = this._rows - 1, this.setupTabStops();
          }
          resize(e6, t5) {
            const i7 = this.getNullCell(o5.DEFAULT_ATTR_DATA);
            let s6 = 0;
            const r5 = this._getCorrectBufferLength(t5);
            if (r5 > this.lines.maxLength && (this.lines.maxLength = r5), this.lines.length > 0) {
              if (this._cols < e6) for (let t6 = 0; t6 < this.lines.length; t6++) s6 += +this.lines.get(t6).resize(e6, i7);
              let n5 = 0;
              if (this._rows < t5) for (let s7 = this._rows; s7 < t5; s7++) this.lines.length < t5 + this.ybase && (this._optionsService.rawOptions.windowsMode || void 0 !== this._optionsService.rawOptions.windowsPty.backend || void 0 !== this._optionsService.rawOptions.windowsPty.buildNumber ? this.lines.push(new o5.BufferLine(e6, i7)) : this.ybase > 0 && this.lines.length <= this.ybase + this.y + n5 + 1 ? (this.ybase--, n5++, this.ydisp > 0 && this.ydisp--) : this.lines.push(new o5.BufferLine(e6, i7)));
              else for (let e7 = this._rows; e7 > t5; e7--) this.lines.length > t5 + this.ybase && (this.lines.length > this.ybase + this.y + 1 ? this.lines.pop() : (this.ybase++, this.ydisp++));
              if (r5 < this.lines.maxLength) {
                const e7 = this.lines.length - r5;
                e7 > 0 && (this.lines.trimStart(e7), this.ybase = Math.max(this.ybase - e7, 0), this.ydisp = Math.max(this.ydisp - e7, 0), this.savedY = Math.max(this.savedY - e7, 0)), this.lines.maxLength = r5;
              }
              this.x = Math.min(this.x, e6 - 1), this.y = Math.min(this.y, t5 - 1), n5 && (this.y += n5), this.savedX = Math.min(this.savedX, e6 - 1), this.scrollTop = 0;
            }
            if (this.scrollBottom = t5 - 1, this._isReflowEnabled && (this._reflow(e6, t5), this._cols > e6)) for (let t6 = 0; t6 < this.lines.length; t6++) s6 += +this.lines.get(t6).resize(e6, i7);
            this._cols = e6, this._rows = t5, this._memoryCleanupQueue.clear(), s6 > 0.1 * this.lines.length && (this._memoryCleanupPosition = 0, this._memoryCleanupQueue.enqueue((() => this._batchedMemoryCleanup())));
          }
          _batchedMemoryCleanup() {
            let e6 = true;
            this._memoryCleanupPosition >= this.lines.length && (this._memoryCleanupPosition = 0, e6 = false);
            let t5 = 0;
            for (; this._memoryCleanupPosition < this.lines.length; ) if (t5 += this.lines.get(this._memoryCleanupPosition++).cleanupMemory(), t5 > 100) return true;
            return e6;
          }
          get _isReflowEnabled() {
            const e6 = this._optionsService.rawOptions.windowsPty;
            return e6 && e6.buildNumber ? this._hasScrollback && "conpty" === e6.backend && e6.buildNumber >= 21376 : this._hasScrollback && !this._optionsService.rawOptions.windowsMode;
          }
          _reflow(e6, t5) {
            this._cols !== e6 && (e6 > this._cols ? this._reflowLarger(e6, t5) : this._reflowSmaller(e6, t5));
          }
          _reflowLarger(e6, t5) {
            const i7 = (0, a3.reflowLargerGetLinesToRemove)(this.lines, this._cols, e6, this.ybase + this.y, this.getNullCell(o5.DEFAULT_ATTR_DATA));
            if (i7.length > 0) {
              const s6 = (0, a3.reflowLargerCreateNewLayout)(this.lines, i7);
              (0, a3.reflowLargerApplyNewLayout)(this.lines, s6.layout), this._reflowLargerAdjustViewport(e6, t5, s6.countRemoved);
            }
          }
          _reflowLargerAdjustViewport(e6, t5, i7) {
            const s6 = this.getNullCell(o5.DEFAULT_ATTR_DATA);
            let r5 = i7;
            for (; r5-- > 0; ) 0 === this.ybase ? (this.y > 0 && this.y--, this.lines.length < t5 && this.lines.push(new o5.BufferLine(e6, s6))) : (this.ydisp === this.ybase && this.ydisp--, this.ybase--);
            this.savedY = Math.max(this.savedY - i7, 0);
          }
          _reflowSmaller(e6, t5) {
            const i7 = this.getNullCell(o5.DEFAULT_ATTR_DATA), s6 = [];
            let r5 = 0;
            for (let n5 = this.lines.length - 1; n5 >= 0; n5--) {
              let h4 = this.lines.get(n5);
              if (!h4 || !h4.isWrapped && h4.getTrimmedLength() <= e6) continue;
              const c5 = [h4];
              for (; h4.isWrapped && n5 > 0; ) h4 = this.lines.get(--n5), c5.unshift(h4);
              const l4 = this.ybase + this.y;
              if (l4 >= n5 && l4 < n5 + c5.length) continue;
              const d4 = c5[c5.length - 1].getTrimmedLength(), _2 = (0, a3.reflowSmallerGetNewLineLengths)(c5, this._cols, e6), u3 = _2.length - c5.length;
              let f3;
              f3 = 0 === this.ybase && this.y !== this.lines.length - 1 ? Math.max(0, this.y - this.lines.maxLength + u3) : Math.max(0, this.lines.length - this.lines.maxLength + u3);
              const v2 = [];
              for (let e7 = 0; e7 < u3; e7++) {
                const e8 = this.getBlankLine(o5.DEFAULT_ATTR_DATA, true);
                v2.push(e8);
              }
              v2.length > 0 && (s6.push({ start: n5 + c5.length + r5, newLines: v2 }), r5 += v2.length), c5.push(...v2);
              let p3 = _2.length - 1, g2 = _2[p3];
              0 === g2 && (p3--, g2 = _2[p3]);
              let m2 = c5.length - u3 - 1, S3 = d4;
              for (; m2 >= 0; ) {
                const e7 = Math.min(S3, g2);
                if (void 0 === c5[p3]) break;
                if (c5[p3].copyCellsFrom(c5[m2], S3 - e7, g2 - e7, e7, true), g2 -= e7, 0 === g2 && (p3--, g2 = _2[p3]), S3 -= e7, 0 === S3) {
                  m2--;
                  const e8 = Math.max(m2, 0);
                  S3 = (0, a3.getWrappedLineTrimmedLength)(c5, e8, this._cols);
                }
              }
              for (let t6 = 0; t6 < c5.length; t6++) _2[t6] < e6 && c5[t6].setCell(_2[t6], i7);
              let C2 = u3 - f3;
              for (; C2-- > 0; ) 0 === this.ybase ? this.y < t5 - 1 ? (this.y++, this.lines.pop()) : (this.ybase++, this.ydisp++) : this.ybase < Math.min(this.lines.maxLength, this.lines.length + r5) - t5 && (this.ybase === this.ydisp && this.ydisp++, this.ybase++);
              this.savedY = Math.min(this.savedY + u3, this.ybase + t5 - 1);
            }
            if (s6.length > 0) {
              const e7 = [], t6 = [];
              for (let e8 = 0; e8 < this.lines.length; e8++) t6.push(this.lines.get(e8));
              const i8 = this.lines.length;
              let n5 = i8 - 1, o6 = 0, a4 = s6[o6];
              this.lines.length = Math.min(this.lines.maxLength, this.lines.length + r5);
              let h4 = 0;
              for (let c6 = Math.min(this.lines.maxLength - 1, i8 + r5 - 1); c6 >= 0; c6--) if (a4 && a4.start > n5 + h4) {
                for (let e8 = a4.newLines.length - 1; e8 >= 0; e8--) this.lines.set(c6--, a4.newLines[e8]);
                c6++, e7.push({ index: n5 + 1, amount: a4.newLines.length }), h4 += a4.newLines.length, a4 = s6[++o6];
              } else this.lines.set(c6, t6[n5--]);
              let c5 = 0;
              for (let t7 = e7.length - 1; t7 >= 0; t7--) e7[t7].index += c5, this.lines.onInsertEmitter.fire(e7[t7]), c5 += e7[t7].amount;
              const l4 = Math.max(0, i8 + r5 - this.lines.maxLength);
              l4 > 0 && this.lines.onTrimEmitter.fire(l4);
            }
          }
          translateBufferLineToString(e6, t5, i7 = 0, s6) {
            const r5 = this.lines.get(e6);
            return r5 ? r5.translateToString(t5, i7, s6) : "";
          }
          getWrappedRangeForLine(e6) {
            let t5 = e6, i7 = e6;
            for (; t5 > 0 && this.lines.get(t5).isWrapped; ) t5--;
            for (; i7 + 1 < this.lines.length && this.lines.get(i7 + 1).isWrapped; ) i7++;
            return { first: t5, last: i7 };
          }
          setupTabStops(e6) {
            for (null != e6 ? this.tabs[e6] || (e6 = this.prevStop(e6)) : (this.tabs = {}, e6 = 0); e6 < this._cols; e6 += this._optionsService.rawOptions.tabStopWidth) this.tabs[e6] = true;
          }
          prevStop(e6) {
            for (null == e6 && (e6 = this.x); !this.tabs[--e6] && e6 > 0; ) ;
            return e6 >= this._cols ? this._cols - 1 : e6 < 0 ? 0 : e6;
          }
          nextStop(e6) {
            for (null == e6 && (e6 = this.x); !this.tabs[++e6] && e6 < this._cols; ) ;
            return e6 >= this._cols ? this._cols - 1 : e6 < 0 ? 0 : e6;
          }
          clearMarkers(e6) {
            this._isClearing = true;
            for (let t5 = 0; t5 < this.markers.length; t5++) this.markers[t5].line === e6 && (this.markers[t5].dispose(), this.markers.splice(t5--, 1));
            this._isClearing = false;
          }
          clearAllMarkers() {
            this._isClearing = true;
            for (let e6 = 0; e6 < this.markers.length; e6++) this.markers[e6].dispose(), this.markers.splice(e6--, 1);
            this._isClearing = false;
          }
          addMarker(e6) {
            const t5 = new l3.Marker(e6);
            return this.markers.push(t5), t5.register(this.lines.onTrim(((e7) => {
              t5.line -= e7, t5.line < 0 && t5.dispose();
            }))), t5.register(this.lines.onInsert(((e7) => {
              t5.line >= e7.index && (t5.line += e7.amount);
            }))), t5.register(this.lines.onDelete(((e7) => {
              t5.line >= e7.index && t5.line < e7.index + e7.amount && t5.dispose(), t5.line > e7.index && (t5.line -= e7.amount);
            }))), t5.register(t5.onDispose((() => this._removeMarker(t5)))), t5;
          }
          _removeMarker(e6) {
            this._isClearing || this.markers.splice(this.markers.indexOf(e6), 1);
          }
        };
      }, 8437: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.BufferLine = t4.DEFAULT_ATTR_DATA = void 0;
        const s5 = i6(3734), r4 = i6(511), n4 = i6(643), o5 = i6(482);
        t4.DEFAULT_ATTR_DATA = Object.freeze(new s5.AttributeData());
        let a3 = 0;
        class h3 {
          constructor(e6, t5, i7 = false) {
            this.isWrapped = i7, this._combined = {}, this._extendedAttrs = {}, this._data = new Uint32Array(3 * e6);
            const s6 = t5 || r4.CellData.fromCharData([0, n4.NULL_CELL_CHAR, n4.NULL_CELL_WIDTH, n4.NULL_CELL_CODE]);
            for (let t6 = 0; t6 < e6; ++t6) this.setCell(t6, s6);
            this.length = e6;
          }
          get(e6) {
            const t5 = this._data[3 * e6 + 0], i7 = 2097151 & t5;
            return [this._data[3 * e6 + 1], 2097152 & t5 ? this._combined[e6] : i7 ? (0, o5.stringFromCodePoint)(i7) : "", t5 >> 22, 2097152 & t5 ? this._combined[e6].charCodeAt(this._combined[e6].length - 1) : i7];
          }
          set(e6, t5) {
            this._data[3 * e6 + 1] = t5[n4.CHAR_DATA_ATTR_INDEX], t5[n4.CHAR_DATA_CHAR_INDEX].length > 1 ? (this._combined[e6] = t5[1], this._data[3 * e6 + 0] = 2097152 | e6 | t5[n4.CHAR_DATA_WIDTH_INDEX] << 22) : this._data[3 * e6 + 0] = t5[n4.CHAR_DATA_CHAR_INDEX].charCodeAt(0) | t5[n4.CHAR_DATA_WIDTH_INDEX] << 22;
          }
          getWidth(e6) {
            return this._data[3 * e6 + 0] >> 22;
          }
          hasWidth(e6) {
            return 12582912 & this._data[3 * e6 + 0];
          }
          getFg(e6) {
            return this._data[3 * e6 + 1];
          }
          getBg(e6) {
            return this._data[3 * e6 + 2];
          }
          hasContent(e6) {
            return 4194303 & this._data[3 * e6 + 0];
          }
          getCodePoint(e6) {
            const t5 = this._data[3 * e6 + 0];
            return 2097152 & t5 ? this._combined[e6].charCodeAt(this._combined[e6].length - 1) : 2097151 & t5;
          }
          isCombined(e6) {
            return 2097152 & this._data[3 * e6 + 0];
          }
          getString(e6) {
            const t5 = this._data[3 * e6 + 0];
            return 2097152 & t5 ? this._combined[e6] : 2097151 & t5 ? (0, o5.stringFromCodePoint)(2097151 & t5) : "";
          }
          isProtected(e6) {
            return 536870912 & this._data[3 * e6 + 2];
          }
          loadCell(e6, t5) {
            return a3 = 3 * e6, t5.content = this._data[a3 + 0], t5.fg = this._data[a3 + 1], t5.bg = this._data[a3 + 2], 2097152 & t5.content && (t5.combinedData = this._combined[e6]), 268435456 & t5.bg && (t5.extended = this._extendedAttrs[e6]), t5;
          }
          setCell(e6, t5) {
            2097152 & t5.content && (this._combined[e6] = t5.combinedData), 268435456 & t5.bg && (this._extendedAttrs[e6] = t5.extended), this._data[3 * e6 + 0] = t5.content, this._data[3 * e6 + 1] = t5.fg, this._data[3 * e6 + 2] = t5.bg;
          }
          setCellFromCodepoint(e6, t5, i7, s6) {
            268435456 & s6.bg && (this._extendedAttrs[e6] = s6.extended), this._data[3 * e6 + 0] = t5 | i7 << 22, this._data[3 * e6 + 1] = s6.fg, this._data[3 * e6 + 2] = s6.bg;
          }
          addCodepointToCell(e6, t5, i7) {
            let s6 = this._data[3 * e6 + 0];
            2097152 & s6 ? this._combined[e6] += (0, o5.stringFromCodePoint)(t5) : 2097151 & s6 ? (this._combined[e6] = (0, o5.stringFromCodePoint)(2097151 & s6) + (0, o5.stringFromCodePoint)(t5), s6 &= -2097152, s6 |= 2097152) : s6 = t5 | 1 << 22, i7 && (s6 &= -12582913, s6 |= i7 << 22), this._data[3 * e6 + 0] = s6;
          }
          insertCells(e6, t5, i7) {
            if ((e6 %= this.length) && 2 === this.getWidth(e6 - 1) && this.setCellFromCodepoint(e6 - 1, 0, 1, i7), t5 < this.length - e6) {
              const s6 = new r4.CellData();
              for (let i8 = this.length - e6 - t5 - 1; i8 >= 0; --i8) this.setCell(e6 + t5 + i8, this.loadCell(e6 + i8, s6));
              for (let s7 = 0; s7 < t5; ++s7) this.setCell(e6 + s7, i7);
            } else for (let t6 = e6; t6 < this.length; ++t6) this.setCell(t6, i7);
            2 === this.getWidth(this.length - 1) && this.setCellFromCodepoint(this.length - 1, 0, 1, i7);
          }
          deleteCells(e6, t5, i7) {
            if (e6 %= this.length, t5 < this.length - e6) {
              const s6 = new r4.CellData();
              for (let i8 = 0; i8 < this.length - e6 - t5; ++i8) this.setCell(e6 + i8, this.loadCell(e6 + t5 + i8, s6));
              for (let e7 = this.length - t5; e7 < this.length; ++e7) this.setCell(e7, i7);
            } else for (let t6 = e6; t6 < this.length; ++t6) this.setCell(t6, i7);
            e6 && 2 === this.getWidth(e6 - 1) && this.setCellFromCodepoint(e6 - 1, 0, 1, i7), 0 !== this.getWidth(e6) || this.hasContent(e6) || this.setCellFromCodepoint(e6, 0, 1, i7);
          }
          replaceCells(e6, t5, i7, s6 = false) {
            if (s6) for (e6 && 2 === this.getWidth(e6 - 1) && !this.isProtected(e6 - 1) && this.setCellFromCodepoint(e6 - 1, 0, 1, i7), t5 < this.length && 2 === this.getWidth(t5 - 1) && !this.isProtected(t5) && this.setCellFromCodepoint(t5, 0, 1, i7); e6 < t5 && e6 < this.length; ) this.isProtected(e6) || this.setCell(e6, i7), e6++;
            else for (e6 && 2 === this.getWidth(e6 - 1) && this.setCellFromCodepoint(e6 - 1, 0, 1, i7), t5 < this.length && 2 === this.getWidth(t5 - 1) && this.setCellFromCodepoint(t5, 0, 1, i7); e6 < t5 && e6 < this.length; ) this.setCell(e6++, i7);
          }
          resize(e6, t5) {
            if (e6 === this.length) return 4 * this._data.length * 2 < this._data.buffer.byteLength;
            const i7 = 3 * e6;
            if (e6 > this.length) {
              if (this._data.buffer.byteLength >= 4 * i7) this._data = new Uint32Array(this._data.buffer, 0, i7);
              else {
                const e7 = new Uint32Array(i7);
                e7.set(this._data), this._data = e7;
              }
              for (let i8 = this.length; i8 < e6; ++i8) this.setCell(i8, t5);
            } else {
              this._data = this._data.subarray(0, i7);
              const t6 = Object.keys(this._combined);
              for (let i8 = 0; i8 < t6.length; i8++) {
                const s7 = parseInt(t6[i8], 10);
                s7 >= e6 && delete this._combined[s7];
              }
              const s6 = Object.keys(this._extendedAttrs);
              for (let t7 = 0; t7 < s6.length; t7++) {
                const i8 = parseInt(s6[t7], 10);
                i8 >= e6 && delete this._extendedAttrs[i8];
              }
            }
            return this.length = e6, 4 * i7 * 2 < this._data.buffer.byteLength;
          }
          cleanupMemory() {
            if (4 * this._data.length * 2 < this._data.buffer.byteLength) {
              const e6 = new Uint32Array(this._data.length);
              return e6.set(this._data), this._data = e6, 1;
            }
            return 0;
          }
          fill(e6, t5 = false) {
            if (t5) for (let t6 = 0; t6 < this.length; ++t6) this.isProtected(t6) || this.setCell(t6, e6);
            else {
              this._combined = {}, this._extendedAttrs = {};
              for (let t6 = 0; t6 < this.length; ++t6) this.setCell(t6, e6);
            }
          }
          copyFrom(e6) {
            this.length !== e6.length ? this._data = new Uint32Array(e6._data) : this._data.set(e6._data), this.length = e6.length, this._combined = {};
            for (const t5 in e6._combined) this._combined[t5] = e6._combined[t5];
            this._extendedAttrs = {};
            for (const t5 in e6._extendedAttrs) this._extendedAttrs[t5] = e6._extendedAttrs[t5];
            this.isWrapped = e6.isWrapped;
          }
          clone() {
            const e6 = new h3(0);
            e6._data = new Uint32Array(this._data), e6.length = this.length;
            for (const t5 in this._combined) e6._combined[t5] = this._combined[t5];
            for (const t5 in this._extendedAttrs) e6._extendedAttrs[t5] = this._extendedAttrs[t5];
            return e6.isWrapped = this.isWrapped, e6;
          }
          getTrimmedLength() {
            for (let e6 = this.length - 1; e6 >= 0; --e6) if (4194303 & this._data[3 * e6 + 0]) return e6 + (this._data[3 * e6 + 0] >> 22);
            return 0;
          }
          getNoBgTrimmedLength() {
            for (let e6 = this.length - 1; e6 >= 0; --e6) if (4194303 & this._data[3 * e6 + 0] || 50331648 & this._data[3 * e6 + 2]) return e6 + (this._data[3 * e6 + 0] >> 22);
            return 0;
          }
          copyCellsFrom(e6, t5, i7, s6, r5) {
            const n5 = e6._data;
            if (r5) for (let r6 = s6 - 1; r6 >= 0; r6--) {
              for (let e7 = 0; e7 < 3; e7++) this._data[3 * (i7 + r6) + e7] = n5[3 * (t5 + r6) + e7];
              268435456 & n5[3 * (t5 + r6) + 2] && (this._extendedAttrs[i7 + r6] = e6._extendedAttrs[t5 + r6]);
            }
            else for (let r6 = 0; r6 < s6; r6++) {
              for (let e7 = 0; e7 < 3; e7++) this._data[3 * (i7 + r6) + e7] = n5[3 * (t5 + r6) + e7];
              268435456 & n5[3 * (t5 + r6) + 2] && (this._extendedAttrs[i7 + r6] = e6._extendedAttrs[t5 + r6]);
            }
            const o6 = Object.keys(e6._combined);
            for (let s7 = 0; s7 < o6.length; s7++) {
              const r6 = parseInt(o6[s7], 10);
              r6 >= t5 && (this._combined[r6 - t5 + i7] = e6._combined[r6]);
            }
          }
          translateToString(e6, t5, i7, s6) {
            t5 = t5 ?? 0, i7 = i7 ?? this.length, e6 && (i7 = Math.min(i7, this.getTrimmedLength())), s6 && (s6.length = 0);
            let r5 = "";
            for (; t5 < i7; ) {
              const e7 = this._data[3 * t5 + 0], i8 = 2097151 & e7, a4 = 2097152 & e7 ? this._combined[t5] : i8 ? (0, o5.stringFromCodePoint)(i8) : n4.WHITESPACE_CELL_CHAR;
              if (r5 += a4, s6) for (let e8 = 0; e8 < a4.length; ++e8) s6.push(t5);
              t5 += e7 >> 22 || 1;
            }
            return s6 && s6.push(t5), r5;
          }
        }
        t4.BufferLine = h3;
      }, 4841: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.getRangeLength = void 0, t4.getRangeLength = function(e6, t5) {
          if (e6.start.y > e6.end.y) throw new Error(`Buffer range end (${e6.end.x}, ${e6.end.y}) cannot be before start (${e6.start.x}, ${e6.start.y})`);
          return t5 * (e6.end.y - e6.start.y) + (e6.end.x - e6.start.x + 1);
        };
      }, 4634: (e5, t4) => {
        function i6(e6, t5, i7) {
          if (t5 === e6.length - 1) return e6[t5].getTrimmedLength();
          const s5 = !e6[t5].hasContent(i7 - 1) && 1 === e6[t5].getWidth(i7 - 1), r4 = 2 === e6[t5 + 1].getWidth(0);
          return s5 && r4 ? i7 - 1 : i7;
        }
        Object.defineProperty(t4, "__esModule", { value: true }), t4.getWrappedLineTrimmedLength = t4.reflowSmallerGetNewLineLengths = t4.reflowLargerApplyNewLayout = t4.reflowLargerCreateNewLayout = t4.reflowLargerGetLinesToRemove = void 0, t4.reflowLargerGetLinesToRemove = function(e6, t5, s5, r4, n4) {
          const o5 = [];
          for (let a3 = 0; a3 < e6.length - 1; a3++) {
            let h3 = a3, c4 = e6.get(++h3);
            if (!c4.isWrapped) continue;
            const l3 = [e6.get(a3)];
            for (; h3 < e6.length && c4.isWrapped; ) l3.push(c4), c4 = e6.get(++h3);
            if (r4 >= a3 && r4 < h3) {
              a3 += l3.length - 1;
              continue;
            }
            let d3 = 0, _2 = i6(l3, d3, t5), u3 = 1, f3 = 0;
            for (; u3 < l3.length; ) {
              const e7 = i6(l3, u3, t5), r5 = e7 - f3, o6 = s5 - _2, a4 = Math.min(r5, o6);
              l3[d3].copyCellsFrom(l3[u3], f3, _2, a4, false), _2 += a4, _2 === s5 && (d3++, _2 = 0), f3 += a4, f3 === e7 && (u3++, f3 = 0), 0 === _2 && 0 !== d3 && 2 === l3[d3 - 1].getWidth(s5 - 1) && (l3[d3].copyCellsFrom(l3[d3 - 1], s5 - 1, _2++, 1, false), l3[d3 - 1].setCell(s5 - 1, n4));
            }
            l3[d3].replaceCells(_2, s5, n4);
            let v2 = 0;
            for (let e7 = l3.length - 1; e7 > 0 && (e7 > d3 || 0 === l3[e7].getTrimmedLength()); e7--) v2++;
            v2 > 0 && (o5.push(a3 + l3.length - v2), o5.push(v2)), a3 += l3.length - 1;
          }
          return o5;
        }, t4.reflowLargerCreateNewLayout = function(e6, t5) {
          const i7 = [];
          let s5 = 0, r4 = t5[s5], n4 = 0;
          for (let o5 = 0; o5 < e6.length; o5++) if (r4 === o5) {
            const i8 = t5[++s5];
            e6.onDeleteEmitter.fire({ index: o5 - n4, amount: i8 }), o5 += i8 - 1, n4 += i8, r4 = t5[++s5];
          } else i7.push(o5);
          return { layout: i7, countRemoved: n4 };
        }, t4.reflowLargerApplyNewLayout = function(e6, t5) {
          const i7 = [];
          for (let s5 = 0; s5 < t5.length; s5++) i7.push(e6.get(t5[s5]));
          for (let t6 = 0; t6 < i7.length; t6++) e6.set(t6, i7[t6]);
          e6.length = t5.length;
        }, t4.reflowSmallerGetNewLineLengths = function(e6, t5, s5) {
          const r4 = [], n4 = e6.map(((s6, r5) => i6(e6, r5, t5))).reduce(((e7, t6) => e7 + t6));
          let o5 = 0, a3 = 0, h3 = 0;
          for (; h3 < n4; ) {
            if (n4 - h3 < s5) {
              r4.push(n4 - h3);
              break;
            }
            o5 += s5;
            const c4 = i6(e6, a3, t5);
            o5 > c4 && (o5 -= c4, a3++);
            const l3 = 2 === e6[a3].getWidth(o5 - 1);
            l3 && o5--;
            const d3 = l3 ? s5 - 1 : s5;
            r4.push(d3), h3 += d3;
          }
          return r4;
        }, t4.getWrappedLineTrimmedLength = i6;
      }, 5295: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.BufferSet = void 0;
        const s5 = i6(8460), r4 = i6(844), n4 = i6(9092);
        class o5 extends r4.Disposable {
          constructor(e6, t5) {
            super(), this._optionsService = e6, this._bufferService = t5, this._onBufferActivate = this.register(new s5.EventEmitter()), this.onBufferActivate = this._onBufferActivate.event, this.reset(), this.register(this._optionsService.onSpecificOptionChange("scrollback", (() => this.resize(this._bufferService.cols, this._bufferService.rows)))), this.register(this._optionsService.onSpecificOptionChange("tabStopWidth", (() => this.setupTabStops())));
          }
          reset() {
            this._normal = new n4.Buffer(true, this._optionsService, this._bufferService), this._normal.fillViewportRows(), this._alt = new n4.Buffer(false, this._optionsService, this._bufferService), this._activeBuffer = this._normal, this._onBufferActivate.fire({ activeBuffer: this._normal, inactiveBuffer: this._alt }), this.setupTabStops();
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
          activateAltBuffer(e6) {
            this._activeBuffer !== this._alt && (this._alt.fillViewportRows(e6), this._alt.x = this._normal.x, this._alt.y = this._normal.y, this._activeBuffer = this._alt, this._onBufferActivate.fire({ activeBuffer: this._alt, inactiveBuffer: this._normal }));
          }
          resize(e6, t5) {
            this._normal.resize(e6, t5), this._alt.resize(e6, t5), this.setupTabStops(e6);
          }
          setupTabStops(e6) {
            this._normal.setupTabStops(e6), this._alt.setupTabStops(e6);
          }
        }
        t4.BufferSet = o5;
      }, 511: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.CellData = void 0;
        const s5 = i6(482), r4 = i6(643), n4 = i6(3734);
        class o5 extends n4.AttributeData {
          constructor() {
            super(...arguments), this.content = 0, this.fg = 0, this.bg = 0, this.extended = new n4.ExtendedAttrs(), this.combinedData = "";
          }
          static fromCharData(e6) {
            const t5 = new o5();
            return t5.setFromCharData(e6), t5;
          }
          isCombined() {
            return 2097152 & this.content;
          }
          getWidth() {
            return this.content >> 22;
          }
          getChars() {
            return 2097152 & this.content ? this.combinedData : 2097151 & this.content ? (0, s5.stringFromCodePoint)(2097151 & this.content) : "";
          }
          getCode() {
            return this.isCombined() ? this.combinedData.charCodeAt(this.combinedData.length - 1) : 2097151 & this.content;
          }
          setFromCharData(e6) {
            this.fg = e6[r4.CHAR_DATA_ATTR_INDEX], this.bg = 0;
            let t5 = false;
            if (e6[r4.CHAR_DATA_CHAR_INDEX].length > 2) t5 = true;
            else if (2 === e6[r4.CHAR_DATA_CHAR_INDEX].length) {
              const i7 = e6[r4.CHAR_DATA_CHAR_INDEX].charCodeAt(0);
              if (55296 <= i7 && i7 <= 56319) {
                const s6 = e6[r4.CHAR_DATA_CHAR_INDEX].charCodeAt(1);
                56320 <= s6 && s6 <= 57343 ? this.content = 1024 * (i7 - 55296) + s6 - 56320 + 65536 | e6[r4.CHAR_DATA_WIDTH_INDEX] << 22 : t5 = true;
              } else t5 = true;
            } else this.content = e6[r4.CHAR_DATA_CHAR_INDEX].charCodeAt(0) | e6[r4.CHAR_DATA_WIDTH_INDEX] << 22;
            t5 && (this.combinedData = e6[r4.CHAR_DATA_CHAR_INDEX], this.content = 2097152 | e6[r4.CHAR_DATA_WIDTH_INDEX] << 22);
          }
          getAsCharData() {
            return [this.fg, this.getChars(), this.getWidth(), this.getCode()];
          }
        }
        t4.CellData = o5;
      }, 643: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.WHITESPACE_CELL_CODE = t4.WHITESPACE_CELL_WIDTH = t4.WHITESPACE_CELL_CHAR = t4.NULL_CELL_CODE = t4.NULL_CELL_WIDTH = t4.NULL_CELL_CHAR = t4.CHAR_DATA_CODE_INDEX = t4.CHAR_DATA_WIDTH_INDEX = t4.CHAR_DATA_CHAR_INDEX = t4.CHAR_DATA_ATTR_INDEX = t4.DEFAULT_EXT = t4.DEFAULT_ATTR = t4.DEFAULT_COLOR = void 0, t4.DEFAULT_COLOR = 0, t4.DEFAULT_ATTR = 256 | t4.DEFAULT_COLOR << 9, t4.DEFAULT_EXT = 0, t4.CHAR_DATA_ATTR_INDEX = 0, t4.CHAR_DATA_CHAR_INDEX = 1, t4.CHAR_DATA_WIDTH_INDEX = 2, t4.CHAR_DATA_CODE_INDEX = 3, t4.NULL_CELL_CHAR = "", t4.NULL_CELL_WIDTH = 1, t4.NULL_CELL_CODE = 0, t4.WHITESPACE_CELL_CHAR = " ", t4.WHITESPACE_CELL_WIDTH = 1, t4.WHITESPACE_CELL_CODE = 32;
      }, 4863: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.Marker = void 0;
        const s5 = i6(8460), r4 = i6(844);
        class n4 {
          get id() {
            return this._id;
          }
          constructor(e6) {
            this.line = e6, this.isDisposed = false, this._disposables = [], this._id = n4._nextId++, this._onDispose = this.register(new s5.EventEmitter()), this.onDispose = this._onDispose.event;
          }
          dispose() {
            this.isDisposed || (this.isDisposed = true, this.line = -1, this._onDispose.fire(), (0, r4.disposeArray)(this._disposables), this._disposables.length = 0);
          }
          register(e6) {
            return this._disposables.push(e6), e6;
          }
        }
        t4.Marker = n4, n4._nextId = 1;
      }, 7116: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.DEFAULT_CHARSET = t4.CHARSETS = void 0, t4.CHARSETS = {}, t4.DEFAULT_CHARSET = t4.CHARSETS.B, t4.CHARSETS[0] = { "`": "\u25C6", a: "\u2592", b: "\u2409", c: "\u240C", d: "\u240D", e: "\u240A", f: "\xB0", g: "\xB1", h: "\u2424", i: "\u240B", j: "\u2518", k: "\u2510", l: "\u250C", m: "\u2514", n: "\u253C", o: "\u23BA", p: "\u23BB", q: "\u2500", r: "\u23BC", s: "\u23BD", t: "\u251C", u: "\u2524", v: "\u2534", w: "\u252C", x: "\u2502", y: "\u2264", z: "\u2265", "{": "\u03C0", "|": "\u2260", "}": "\xA3", "~": "\xB7" }, t4.CHARSETS.A = { "#": "\xA3" }, t4.CHARSETS.B = void 0, t4.CHARSETS[4] = { "#": "\xA3", "@": "\xBE", "[": "ij", "\\": "\xBD", "]": "|", "{": "\xA8", "|": "f", "}": "\xBC", "~": "\xB4" }, t4.CHARSETS.C = t4.CHARSETS[5] = { "[": "\xC4", "\\": "\xD6", "]": "\xC5", "^": "\xDC", "`": "\xE9", "{": "\xE4", "|": "\xF6", "}": "\xE5", "~": "\xFC" }, t4.CHARSETS.R = { "#": "\xA3", "@": "\xE0", "[": "\xB0", "\\": "\xE7", "]": "\xA7", "{": "\xE9", "|": "\xF9", "}": "\xE8", "~": "\xA8" }, t4.CHARSETS.Q = { "@": "\xE0", "[": "\xE2", "\\": "\xE7", "]": "\xEA", "^": "\xEE", "`": "\xF4", "{": "\xE9", "|": "\xF9", "}": "\xE8", "~": "\xFB" }, t4.CHARSETS.K = { "@": "\xA7", "[": "\xC4", "\\": "\xD6", "]": "\xDC", "{": "\xE4", "|": "\xF6", "}": "\xFC", "~": "\xDF" }, t4.CHARSETS.Y = { "#": "\xA3", "@": "\xA7", "[": "\xB0", "\\": "\xE7", "]": "\xE9", "`": "\xF9", "{": "\xE0", "|": "\xF2", "}": "\xE8", "~": "\xEC" }, t4.CHARSETS.E = t4.CHARSETS[6] = { "@": "\xC4", "[": "\xC6", "\\": "\xD8", "]": "\xC5", "^": "\xDC", "`": "\xE4", "{": "\xE6", "|": "\xF8", "}": "\xE5", "~": "\xFC" }, t4.CHARSETS.Z = { "#": "\xA3", "@": "\xA7", "[": "\xA1", "\\": "\xD1", "]": "\xBF", "{": "\xB0", "|": "\xF1", "}": "\xE7" }, t4.CHARSETS.H = t4.CHARSETS[7] = { "@": "\xC9", "[": "\xC4", "\\": "\xD6", "]": "\xC5", "^": "\xDC", "`": "\xE9", "{": "\xE4", "|": "\xF6", "}": "\xE5", "~": "\xFC" }, t4.CHARSETS["="] = { "#": "\xF9", "@": "\xE0", "[": "\xE9", "\\": "\xE7", "]": "\xEA", "^": "\xEE", _: "\xE8", "`": "\xF4", "{": "\xE4", "|": "\xF6", "}": "\xFC", "~": "\xFB" };
      }, 2584: (e5, t4) => {
        var i6, s5, r4;
        Object.defineProperty(t4, "__esModule", { value: true }), t4.C1_ESCAPED = t4.C1 = t4.C0 = void 0, (function(e6) {
          e6.NUL = "\0", e6.SOH = "", e6.STX = "", e6.ETX = "", e6.EOT = "", e6.ENQ = "", e6.ACK = "", e6.BEL = "\x07", e6.BS = "\b", e6.HT = "	", e6.LF = "\n", e6.VT = "\v", e6.FF = "\f", e6.CR = "\r", e6.SO = "", e6.SI = "", e6.DLE = "", e6.DC1 = "", e6.DC2 = "", e6.DC3 = "", e6.DC4 = "", e6.NAK = "", e6.SYN = "", e6.ETB = "", e6.CAN = "", e6.EM = "", e6.SUB = "", e6.ESC = "\x1B", e6.FS = "", e6.GS = "", e6.RS = "", e6.US = "", e6.SP = " ", e6.DEL = "\x7F";
        })(i6 || (t4.C0 = i6 = {})), (function(e6) {
          e6.PAD = "\x80", e6.HOP = "\x81", e6.BPH = "\x82", e6.NBH = "\x83", e6.IND = "\x84", e6.NEL = "\x85", e6.SSA = "\x86", e6.ESA = "\x87", e6.HTS = "\x88", e6.HTJ = "\x89", e6.VTS = "\x8A", e6.PLD = "\x8B", e6.PLU = "\x8C", e6.RI = "\x8D", e6.SS2 = "\x8E", e6.SS3 = "\x8F", e6.DCS = "\x90", e6.PU1 = "\x91", e6.PU2 = "\x92", e6.STS = "\x93", e6.CCH = "\x94", e6.MW = "\x95", e6.SPA = "\x96", e6.EPA = "\x97", e6.SOS = "\x98", e6.SGCI = "\x99", e6.SCI = "\x9A", e6.CSI = "\x9B", e6.ST = "\x9C", e6.OSC = "\x9D", e6.PM = "\x9E", e6.APC = "\x9F";
        })(s5 || (t4.C1 = s5 = {})), (function(e6) {
          e6.ST = `${i6.ESC}\\`;
        })(r4 || (t4.C1_ESCAPED = r4 = {}));
      }, 7399: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.evaluateKeyboardEvent = void 0;
        const s5 = i6(2584), r4 = { 48: ["0", ")"], 49: ["1", "!"], 50: ["2", "@"], 51: ["3", "#"], 52: ["4", "$"], 53: ["5", "%"], 54: ["6", "^"], 55: ["7", "&"], 56: ["8", "*"], 57: ["9", "("], 186: [";", ":"], 187: ["=", "+"], 188: [",", "<"], 189: ["-", "_"], 190: [".", ">"], 191: ["/", "?"], 192: ["`", "~"], 219: ["[", "{"], 220: ["\\", "|"], 221: ["]", "}"], 222: ["'", '"'] };
        t4.evaluateKeyboardEvent = function(e6, t5, i7, n4) {
          const o5 = { type: 0, cancel: false, key: void 0 }, a3 = (e6.shiftKey ? 1 : 0) | (e6.altKey ? 2 : 0) | (e6.ctrlKey ? 4 : 0) | (e6.metaKey ? 8 : 0);
          switch (e6.keyCode) {
            case 0:
              "UIKeyInputUpArrow" === e6.key ? o5.key = t5 ? s5.C0.ESC + "OA" : s5.C0.ESC + "[A" : "UIKeyInputLeftArrow" === e6.key ? o5.key = t5 ? s5.C0.ESC + "OD" : s5.C0.ESC + "[D" : "UIKeyInputRightArrow" === e6.key ? o5.key = t5 ? s5.C0.ESC + "OC" : s5.C0.ESC + "[C" : "UIKeyInputDownArrow" === e6.key && (o5.key = t5 ? s5.C0.ESC + "OB" : s5.C0.ESC + "[B");
              break;
            case 8:
              o5.key = e6.ctrlKey ? "\b" : s5.C0.DEL, e6.altKey && (o5.key = s5.C0.ESC + o5.key);
              break;
            case 9:
              if (e6.shiftKey) {
                o5.key = s5.C0.ESC + "[Z";
                break;
              }
              o5.key = s5.C0.HT, o5.cancel = true;
              break;
            case 13:
              o5.key = e6.altKey ? s5.C0.ESC + s5.C0.CR : s5.C0.CR, o5.cancel = true;
              break;
            case 27:
              o5.key = s5.C0.ESC, e6.altKey && (o5.key = s5.C0.ESC + s5.C0.ESC), o5.cancel = true;
              break;
            case 37:
              if (e6.metaKey) break;
              a3 ? (o5.key = s5.C0.ESC + "[1;" + (a3 + 1) + "D", o5.key === s5.C0.ESC + "[1;3D" && (o5.key = s5.C0.ESC + (i7 ? "b" : "[1;5D"))) : o5.key = t5 ? s5.C0.ESC + "OD" : s5.C0.ESC + "[D";
              break;
            case 39:
              if (e6.metaKey) break;
              a3 ? (o5.key = s5.C0.ESC + "[1;" + (a3 + 1) + "C", o5.key === s5.C0.ESC + "[1;3C" && (o5.key = s5.C0.ESC + (i7 ? "f" : "[1;5C"))) : o5.key = t5 ? s5.C0.ESC + "OC" : s5.C0.ESC + "[C";
              break;
            case 38:
              if (e6.metaKey) break;
              a3 ? (o5.key = s5.C0.ESC + "[1;" + (a3 + 1) + "A", i7 || o5.key !== s5.C0.ESC + "[1;3A" || (o5.key = s5.C0.ESC + "[1;5A")) : o5.key = t5 ? s5.C0.ESC + "OA" : s5.C0.ESC + "[A";
              break;
            case 40:
              if (e6.metaKey) break;
              a3 ? (o5.key = s5.C0.ESC + "[1;" + (a3 + 1) + "B", i7 || o5.key !== s5.C0.ESC + "[1;3B" || (o5.key = s5.C0.ESC + "[1;5B")) : o5.key = t5 ? s5.C0.ESC + "OB" : s5.C0.ESC + "[B";
              break;
            case 45:
              e6.shiftKey || e6.ctrlKey || (o5.key = s5.C0.ESC + "[2~");
              break;
            case 46:
              o5.key = a3 ? s5.C0.ESC + "[3;" + (a3 + 1) + "~" : s5.C0.ESC + "[3~";
              break;
            case 36:
              o5.key = a3 ? s5.C0.ESC + "[1;" + (a3 + 1) + "H" : t5 ? s5.C0.ESC + "OH" : s5.C0.ESC + "[H";
              break;
            case 35:
              o5.key = a3 ? s5.C0.ESC + "[1;" + (a3 + 1) + "F" : t5 ? s5.C0.ESC + "OF" : s5.C0.ESC + "[F";
              break;
            case 33:
              e6.shiftKey ? o5.type = 2 : e6.ctrlKey ? o5.key = s5.C0.ESC + "[5;" + (a3 + 1) + "~" : o5.key = s5.C0.ESC + "[5~";
              break;
            case 34:
              e6.shiftKey ? o5.type = 3 : e6.ctrlKey ? o5.key = s5.C0.ESC + "[6;" + (a3 + 1) + "~" : o5.key = s5.C0.ESC + "[6~";
              break;
            case 112:
              o5.key = a3 ? s5.C0.ESC + "[1;" + (a3 + 1) + "P" : s5.C0.ESC + "OP";
              break;
            case 113:
              o5.key = a3 ? s5.C0.ESC + "[1;" + (a3 + 1) + "Q" : s5.C0.ESC + "OQ";
              break;
            case 114:
              o5.key = a3 ? s5.C0.ESC + "[1;" + (a3 + 1) + "R" : s5.C0.ESC + "OR";
              break;
            case 115:
              o5.key = a3 ? s5.C0.ESC + "[1;" + (a3 + 1) + "S" : s5.C0.ESC + "OS";
              break;
            case 116:
              o5.key = a3 ? s5.C0.ESC + "[15;" + (a3 + 1) + "~" : s5.C0.ESC + "[15~";
              break;
            case 117:
              o5.key = a3 ? s5.C0.ESC + "[17;" + (a3 + 1) + "~" : s5.C0.ESC + "[17~";
              break;
            case 118:
              o5.key = a3 ? s5.C0.ESC + "[18;" + (a3 + 1) + "~" : s5.C0.ESC + "[18~";
              break;
            case 119:
              o5.key = a3 ? s5.C0.ESC + "[19;" + (a3 + 1) + "~" : s5.C0.ESC + "[19~";
              break;
            case 120:
              o5.key = a3 ? s5.C0.ESC + "[20;" + (a3 + 1) + "~" : s5.C0.ESC + "[20~";
              break;
            case 121:
              o5.key = a3 ? s5.C0.ESC + "[21;" + (a3 + 1) + "~" : s5.C0.ESC + "[21~";
              break;
            case 122:
              o5.key = a3 ? s5.C0.ESC + "[23;" + (a3 + 1) + "~" : s5.C0.ESC + "[23~";
              break;
            case 123:
              o5.key = a3 ? s5.C0.ESC + "[24;" + (a3 + 1) + "~" : s5.C0.ESC + "[24~";
              break;
            default:
              if (!e6.ctrlKey || e6.shiftKey || e6.altKey || e6.metaKey) if (i7 && !n4 || !e6.altKey || e6.metaKey) !i7 || e6.altKey || e6.ctrlKey || e6.shiftKey || !e6.metaKey ? e6.key && !e6.ctrlKey && !e6.altKey && !e6.metaKey && e6.keyCode >= 48 && 1 === e6.key.length ? o5.key = e6.key : e6.key && e6.ctrlKey && ("_" === e6.key && (o5.key = s5.C0.US), "@" === e6.key && (o5.key = s5.C0.NUL)) : 65 === e6.keyCode && (o5.type = 1);
              else {
                const t6 = r4[e6.keyCode], i8 = t6?.[e6.shiftKey ? 1 : 0];
                if (i8) o5.key = s5.C0.ESC + i8;
                else if (e6.keyCode >= 65 && e6.keyCode <= 90) {
                  const t7 = e6.ctrlKey ? e6.keyCode - 64 : e6.keyCode + 32;
                  let i9 = String.fromCharCode(t7);
                  e6.shiftKey && (i9 = i9.toUpperCase()), o5.key = s5.C0.ESC + i9;
                } else if (32 === e6.keyCode) o5.key = s5.C0.ESC + (e6.ctrlKey ? s5.C0.NUL : " ");
                else if ("Dead" === e6.key && e6.code.startsWith("Key")) {
                  let t7 = e6.code.slice(3, 4);
                  e6.shiftKey || (t7 = t7.toLowerCase()), o5.key = s5.C0.ESC + t7, o5.cancel = true;
                }
              }
              else e6.keyCode >= 65 && e6.keyCode <= 90 ? o5.key = String.fromCharCode(e6.keyCode - 64) : 32 === e6.keyCode ? o5.key = s5.C0.NUL : e6.keyCode >= 51 && e6.keyCode <= 55 ? o5.key = String.fromCharCode(e6.keyCode - 51 + 27) : 56 === e6.keyCode ? o5.key = s5.C0.DEL : 219 === e6.keyCode ? o5.key = s5.C0.ESC : 220 === e6.keyCode ? o5.key = s5.C0.FS : 221 === e6.keyCode && (o5.key = s5.C0.GS);
          }
          return o5;
        };
      }, 482: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.Utf8ToUtf32 = t4.StringToUtf32 = t4.utf32ToString = t4.stringFromCodePoint = void 0, t4.stringFromCodePoint = function(e6) {
          return e6 > 65535 ? (e6 -= 65536, String.fromCharCode(55296 + (e6 >> 10)) + String.fromCharCode(e6 % 1024 + 56320)) : String.fromCharCode(e6);
        }, t4.utf32ToString = function(e6, t5 = 0, i6 = e6.length) {
          let s5 = "";
          for (let r4 = t5; r4 < i6; ++r4) {
            let t6 = e6[r4];
            t6 > 65535 ? (t6 -= 65536, s5 += String.fromCharCode(55296 + (t6 >> 10)) + String.fromCharCode(t6 % 1024 + 56320)) : s5 += String.fromCharCode(t6);
          }
          return s5;
        }, t4.StringToUtf32 = class {
          constructor() {
            this._interim = 0;
          }
          clear() {
            this._interim = 0;
          }
          decode(e6, t5) {
            const i6 = e6.length;
            if (!i6) return 0;
            let s5 = 0, r4 = 0;
            if (this._interim) {
              const i7 = e6.charCodeAt(r4++);
              56320 <= i7 && i7 <= 57343 ? t5[s5++] = 1024 * (this._interim - 55296) + i7 - 56320 + 65536 : (t5[s5++] = this._interim, t5[s5++] = i7), this._interim = 0;
            }
            for (let n4 = r4; n4 < i6; ++n4) {
              const r5 = e6.charCodeAt(n4);
              if (55296 <= r5 && r5 <= 56319) {
                if (++n4 >= i6) return this._interim = r5, s5;
                const o5 = e6.charCodeAt(n4);
                56320 <= o5 && o5 <= 57343 ? t5[s5++] = 1024 * (r5 - 55296) + o5 - 56320 + 65536 : (t5[s5++] = r5, t5[s5++] = o5);
              } else 65279 !== r5 && (t5[s5++] = r5);
            }
            return s5;
          }
        }, t4.Utf8ToUtf32 = class {
          constructor() {
            this.interim = new Uint8Array(3);
          }
          clear() {
            this.interim.fill(0);
          }
          decode(e6, t5) {
            const i6 = e6.length;
            if (!i6) return 0;
            let s5, r4, n4, o5, a3 = 0, h3 = 0, c4 = 0;
            if (this.interim[0]) {
              let s6 = false, r5 = this.interim[0];
              r5 &= 192 == (224 & r5) ? 31 : 224 == (240 & r5) ? 15 : 7;
              let n5, o6 = 0;
              for (; (n5 = 63 & this.interim[++o6]) && o6 < 4; ) r5 <<= 6, r5 |= n5;
              const h4 = 192 == (224 & this.interim[0]) ? 2 : 224 == (240 & this.interim[0]) ? 3 : 4, l4 = h4 - o6;
              for (; c4 < l4; ) {
                if (c4 >= i6) return 0;
                if (n5 = e6[c4++], 128 != (192 & n5)) {
                  c4--, s6 = true;
                  break;
                }
                this.interim[o6++] = n5, r5 <<= 6, r5 |= 63 & n5;
              }
              s6 || (2 === h4 ? r5 < 128 ? c4-- : t5[a3++] = r5 : 3 === h4 ? r5 < 2048 || r5 >= 55296 && r5 <= 57343 || 65279 === r5 || (t5[a3++] = r5) : r5 < 65536 || r5 > 1114111 || (t5[a3++] = r5)), this.interim.fill(0);
            }
            const l3 = i6 - 4;
            let d3 = c4;
            for (; d3 < i6; ) {
              for (; !(!(d3 < l3) || 128 & (s5 = e6[d3]) || 128 & (r4 = e6[d3 + 1]) || 128 & (n4 = e6[d3 + 2]) || 128 & (o5 = e6[d3 + 3])); ) t5[a3++] = s5, t5[a3++] = r4, t5[a3++] = n4, t5[a3++] = o5, d3 += 4;
              if (s5 = e6[d3++], s5 < 128) t5[a3++] = s5;
              else if (192 == (224 & s5)) {
                if (d3 >= i6) return this.interim[0] = s5, a3;
                if (r4 = e6[d3++], 128 != (192 & r4)) {
                  d3--;
                  continue;
                }
                if (h3 = (31 & s5) << 6 | 63 & r4, h3 < 128) {
                  d3--;
                  continue;
                }
                t5[a3++] = h3;
              } else if (224 == (240 & s5)) {
                if (d3 >= i6) return this.interim[0] = s5, a3;
                if (r4 = e6[d3++], 128 != (192 & r4)) {
                  d3--;
                  continue;
                }
                if (d3 >= i6) return this.interim[0] = s5, this.interim[1] = r4, a3;
                if (n4 = e6[d3++], 128 != (192 & n4)) {
                  d3--;
                  continue;
                }
                if (h3 = (15 & s5) << 12 | (63 & r4) << 6 | 63 & n4, h3 < 2048 || h3 >= 55296 && h3 <= 57343 || 65279 === h3) continue;
                t5[a3++] = h3;
              } else if (240 == (248 & s5)) {
                if (d3 >= i6) return this.interim[0] = s5, a3;
                if (r4 = e6[d3++], 128 != (192 & r4)) {
                  d3--;
                  continue;
                }
                if (d3 >= i6) return this.interim[0] = s5, this.interim[1] = r4, a3;
                if (n4 = e6[d3++], 128 != (192 & n4)) {
                  d3--;
                  continue;
                }
                if (d3 >= i6) return this.interim[0] = s5, this.interim[1] = r4, this.interim[2] = n4, a3;
                if (o5 = e6[d3++], 128 != (192 & o5)) {
                  d3--;
                  continue;
                }
                if (h3 = (7 & s5) << 18 | (63 & r4) << 12 | (63 & n4) << 6 | 63 & o5, h3 < 65536 || h3 > 1114111) continue;
                t5[a3++] = h3;
              }
            }
            return a3;
          }
        };
      }, 225: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.UnicodeV6 = void 0;
        const s5 = i6(1480), r4 = [[768, 879], [1155, 1158], [1160, 1161], [1425, 1469], [1471, 1471], [1473, 1474], [1476, 1477], [1479, 1479], [1536, 1539], [1552, 1557], [1611, 1630], [1648, 1648], [1750, 1764], [1767, 1768], [1770, 1773], [1807, 1807], [1809, 1809], [1840, 1866], [1958, 1968], [2027, 2035], [2305, 2306], [2364, 2364], [2369, 2376], [2381, 2381], [2385, 2388], [2402, 2403], [2433, 2433], [2492, 2492], [2497, 2500], [2509, 2509], [2530, 2531], [2561, 2562], [2620, 2620], [2625, 2626], [2631, 2632], [2635, 2637], [2672, 2673], [2689, 2690], [2748, 2748], [2753, 2757], [2759, 2760], [2765, 2765], [2786, 2787], [2817, 2817], [2876, 2876], [2879, 2879], [2881, 2883], [2893, 2893], [2902, 2902], [2946, 2946], [3008, 3008], [3021, 3021], [3134, 3136], [3142, 3144], [3146, 3149], [3157, 3158], [3260, 3260], [3263, 3263], [3270, 3270], [3276, 3277], [3298, 3299], [3393, 3395], [3405, 3405], [3530, 3530], [3538, 3540], [3542, 3542], [3633, 3633], [3636, 3642], [3655, 3662], [3761, 3761], [3764, 3769], [3771, 3772], [3784, 3789], [3864, 3865], [3893, 3893], [3895, 3895], [3897, 3897], [3953, 3966], [3968, 3972], [3974, 3975], [3984, 3991], [3993, 4028], [4038, 4038], [4141, 4144], [4146, 4146], [4150, 4151], [4153, 4153], [4184, 4185], [4448, 4607], [4959, 4959], [5906, 5908], [5938, 5940], [5970, 5971], [6002, 6003], [6068, 6069], [6071, 6077], [6086, 6086], [6089, 6099], [6109, 6109], [6155, 6157], [6313, 6313], [6432, 6434], [6439, 6440], [6450, 6450], [6457, 6459], [6679, 6680], [6912, 6915], [6964, 6964], [6966, 6970], [6972, 6972], [6978, 6978], [7019, 7027], [7616, 7626], [7678, 7679], [8203, 8207], [8234, 8238], [8288, 8291], [8298, 8303], [8400, 8431], [12330, 12335], [12441, 12442], [43014, 43014], [43019, 43019], [43045, 43046], [64286, 64286], [65024, 65039], [65056, 65059], [65279, 65279], [65529, 65531]], n4 = [[68097, 68099], [68101, 68102], [68108, 68111], [68152, 68154], [68159, 68159], [119143, 119145], [119155, 119170], [119173, 119179], [119210, 119213], [119362, 119364], [917505, 917505], [917536, 917631], [917760, 917999]];
        let o5;
        t4.UnicodeV6 = class {
          constructor() {
            if (this.version = "6", !o5) {
              o5 = new Uint8Array(65536), o5.fill(1), o5[0] = 0, o5.fill(0, 1, 32), o5.fill(0, 127, 160), o5.fill(2, 4352, 4448), o5[9001] = 2, o5[9002] = 2, o5.fill(2, 11904, 42192), o5[12351] = 1, o5.fill(2, 44032, 55204), o5.fill(2, 63744, 64256), o5.fill(2, 65040, 65050), o5.fill(2, 65072, 65136), o5.fill(2, 65280, 65377), o5.fill(2, 65504, 65511);
              for (let e6 = 0; e6 < r4.length; ++e6) o5.fill(0, r4[e6][0], r4[e6][1] + 1);
            }
          }
          wcwidth(e6) {
            return e6 < 32 ? 0 : e6 < 127 ? 1 : e6 < 65536 ? o5[e6] : (function(e7, t5) {
              let i7, s6 = 0, r5 = t5.length - 1;
              if (e7 < t5[0][0] || e7 > t5[r5][1]) return false;
              for (; r5 >= s6; ) if (i7 = s6 + r5 >> 1, e7 > t5[i7][1]) s6 = i7 + 1;
              else {
                if (!(e7 < t5[i7][0])) return true;
                r5 = i7 - 1;
              }
              return false;
            })(e6, n4) ? 0 : e6 >= 131072 && e6 <= 196605 || e6 >= 196608 && e6 <= 262141 ? 2 : 1;
          }
          charProperties(e6, t5) {
            let i7 = this.wcwidth(e6), r5 = 0 === i7 && 0 !== t5;
            if (r5) {
              const e7 = s5.UnicodeService.extractWidth(t5);
              0 === e7 ? r5 = false : e7 > i7 && (i7 = e7);
            }
            return s5.UnicodeService.createPropertyValue(0, i7, r5);
          }
        };
      }, 5981: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.WriteBuffer = void 0;
        const s5 = i6(8460), r4 = i6(844);
        class n4 extends r4.Disposable {
          constructor(e6) {
            super(), this._action = e6, this._writeBuffer = [], this._callbacks = [], this._pendingData = 0, this._bufferOffset = 0, this._isSyncWriting = false, this._syncCalls = 0, this._didUserInput = false, this._onWriteParsed = this.register(new s5.EventEmitter()), this.onWriteParsed = this._onWriteParsed.event;
          }
          handleUserInput() {
            this._didUserInput = true;
          }
          writeSync(e6, t5) {
            if (void 0 !== t5 && this._syncCalls > t5) return void (this._syncCalls = 0);
            if (this._pendingData += e6.length, this._writeBuffer.push(e6), this._callbacks.push(void 0), this._syncCalls++, this._isSyncWriting) return;
            let i7;
            for (this._isSyncWriting = true; i7 = this._writeBuffer.shift(); ) {
              this._action(i7);
              const e7 = this._callbacks.shift();
              e7 && e7();
            }
            this._pendingData = 0, this._bufferOffset = 2147483647, this._isSyncWriting = false, this._syncCalls = 0;
          }
          write(e6, t5) {
            if (this._pendingData > 5e7) throw new Error("write data discarded, use flow control to avoid losing data");
            if (!this._writeBuffer.length) {
              if (this._bufferOffset = 0, this._didUserInput) return this._didUserInput = false, this._pendingData += e6.length, this._writeBuffer.push(e6), this._callbacks.push(t5), void this._innerWrite();
              setTimeout((() => this._innerWrite()));
            }
            this._pendingData += e6.length, this._writeBuffer.push(e6), this._callbacks.push(t5);
          }
          _innerWrite(e6 = 0, t5 = true) {
            const i7 = e6 || Date.now();
            for (; this._writeBuffer.length > this._bufferOffset; ) {
              const e7 = this._writeBuffer[this._bufferOffset], s6 = this._action(e7, t5);
              if (s6) {
                const e8 = (e9) => Date.now() - i7 >= 12 ? setTimeout((() => this._innerWrite(0, e9))) : this._innerWrite(i7, e9);
                return void s6.catch(((e9) => (queueMicrotask((() => {
                  throw e9;
                })), Promise.resolve(false)))).then(e8);
              }
              const r5 = this._callbacks[this._bufferOffset];
              if (r5 && r5(), this._bufferOffset++, this._pendingData -= e7.length, Date.now() - i7 >= 12) break;
            }
            this._writeBuffer.length > this._bufferOffset ? (this._bufferOffset > 50 && (this._writeBuffer = this._writeBuffer.slice(this._bufferOffset), this._callbacks = this._callbacks.slice(this._bufferOffset), this._bufferOffset = 0), setTimeout((() => this._innerWrite()))) : (this._writeBuffer.length = 0, this._callbacks.length = 0, this._pendingData = 0, this._bufferOffset = 0), this._onWriteParsed.fire();
          }
        }
        t4.WriteBuffer = n4;
      }, 5941: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.toRgbString = t4.parseColor = void 0;
        const i6 = /^([\da-f])\/([\da-f])\/([\da-f])$|^([\da-f]{2})\/([\da-f]{2})\/([\da-f]{2})$|^([\da-f]{3})\/([\da-f]{3})\/([\da-f]{3})$|^([\da-f]{4})\/([\da-f]{4})\/([\da-f]{4})$/, s5 = /^[\da-f]+$/;
        function r4(e6, t5) {
          const i7 = e6.toString(16), s6 = i7.length < 2 ? "0" + i7 : i7;
          switch (t5) {
            case 4:
              return i7[0];
            case 8:
              return s6;
            case 12:
              return (s6 + s6).slice(0, 3);
            default:
              return s6 + s6;
          }
        }
        t4.parseColor = function(e6) {
          if (!e6) return;
          let t5 = e6.toLowerCase();
          if (0 === t5.indexOf("rgb:")) {
            t5 = t5.slice(4);
            const e7 = i6.exec(t5);
            if (e7) {
              const t6 = e7[1] ? 15 : e7[4] ? 255 : e7[7] ? 4095 : 65535;
              return [Math.round(parseInt(e7[1] || e7[4] || e7[7] || e7[10], 16) / t6 * 255), Math.round(parseInt(e7[2] || e7[5] || e7[8] || e7[11], 16) / t6 * 255), Math.round(parseInt(e7[3] || e7[6] || e7[9] || e7[12], 16) / t6 * 255)];
            }
          } else if (0 === t5.indexOf("#") && (t5 = t5.slice(1), s5.exec(t5) && [3, 6, 9, 12].includes(t5.length))) {
            const e7 = t5.length / 3, i7 = [0, 0, 0];
            for (let s6 = 0; s6 < 3; ++s6) {
              const r5 = parseInt(t5.slice(e7 * s6, e7 * s6 + e7), 16);
              i7[s6] = 1 === e7 ? r5 << 4 : 2 === e7 ? r5 : 3 === e7 ? r5 >> 4 : r5 >> 8;
            }
            return i7;
          }
        }, t4.toRgbString = function(e6, t5 = 16) {
          const [i7, s6, n4] = e6;
          return `rgb:${r4(i7, t5)}/${r4(s6, t5)}/${r4(n4, t5)}`;
        };
      }, 5770: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.PAYLOAD_LIMIT = void 0, t4.PAYLOAD_LIMIT = 1e7;
      }, 6351: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.DcsHandler = t4.DcsParser = void 0;
        const s5 = i6(482), r4 = i6(8742), n4 = i6(5770), o5 = [];
        t4.DcsParser = class {
          constructor() {
            this._handlers = /* @__PURE__ */ Object.create(null), this._active = o5, this._ident = 0, this._handlerFb = () => {
            }, this._stack = { paused: false, loopPosition: 0, fallThrough: false };
          }
          dispose() {
            this._handlers = /* @__PURE__ */ Object.create(null), this._handlerFb = () => {
            }, this._active = o5;
          }
          registerHandler(e6, t5) {
            void 0 === this._handlers[e6] && (this._handlers[e6] = []);
            const i7 = this._handlers[e6];
            return i7.push(t5), { dispose: () => {
              const e7 = i7.indexOf(t5);
              -1 !== e7 && i7.splice(e7, 1);
            } };
          }
          clearHandler(e6) {
            this._handlers[e6] && delete this._handlers[e6];
          }
          setHandlerFallback(e6) {
            this._handlerFb = e6;
          }
          reset() {
            if (this._active.length) for (let e6 = this._stack.paused ? this._stack.loopPosition - 1 : this._active.length - 1; e6 >= 0; --e6) this._active[e6].unhook(false);
            this._stack.paused = false, this._active = o5, this._ident = 0;
          }
          hook(e6, t5) {
            if (this.reset(), this._ident = e6, this._active = this._handlers[e6] || o5, this._active.length) for (let e7 = this._active.length - 1; e7 >= 0; e7--) this._active[e7].hook(t5);
            else this._handlerFb(this._ident, "HOOK", t5);
          }
          put(e6, t5, i7) {
            if (this._active.length) for (let s6 = this._active.length - 1; s6 >= 0; s6--) this._active[s6].put(e6, t5, i7);
            else this._handlerFb(this._ident, "PUT", (0, s5.utf32ToString)(e6, t5, i7));
          }
          unhook(e6, t5 = true) {
            if (this._active.length) {
              let i7 = false, s6 = this._active.length - 1, r5 = false;
              if (this._stack.paused && (s6 = this._stack.loopPosition - 1, i7 = t5, r5 = this._stack.fallThrough, this._stack.paused = false), !r5 && false === i7) {
                for (; s6 >= 0 && (i7 = this._active[s6].unhook(e6), true !== i7); s6--) if (i7 instanceof Promise) return this._stack.paused = true, this._stack.loopPosition = s6, this._stack.fallThrough = false, i7;
                s6--;
              }
              for (; s6 >= 0; s6--) if (i7 = this._active[s6].unhook(false), i7 instanceof Promise) return this._stack.paused = true, this._stack.loopPosition = s6, this._stack.fallThrough = true, i7;
            } else this._handlerFb(this._ident, "UNHOOK", e6);
            this._active = o5, this._ident = 0;
          }
        };
        const a3 = new r4.Params();
        a3.addParam(0), t4.DcsHandler = class {
          constructor(e6) {
            this._handler = e6, this._data = "", this._params = a3, this._hitLimit = false;
          }
          hook(e6) {
            this._params = e6.length > 1 || e6.params[0] ? e6.clone() : a3, this._data = "", this._hitLimit = false;
          }
          put(e6, t5, i7) {
            this._hitLimit || (this._data += (0, s5.utf32ToString)(e6, t5, i7), this._data.length > n4.PAYLOAD_LIMIT && (this._data = "", this._hitLimit = true));
          }
          unhook(e6) {
            let t5 = false;
            if (this._hitLimit) t5 = false;
            else if (e6 && (t5 = this._handler(this._data, this._params), t5 instanceof Promise)) return t5.then(((e7) => (this._params = a3, this._data = "", this._hitLimit = false, e7)));
            return this._params = a3, this._data = "", this._hitLimit = false, t5;
          }
        };
      }, 2015: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.EscapeSequenceParser = t4.VT500_TRANSITION_TABLE = t4.TransitionTable = void 0;
        const s5 = i6(844), r4 = i6(8742), n4 = i6(6242), o5 = i6(6351);
        class a3 {
          constructor(e6) {
            this.table = new Uint8Array(e6);
          }
          setDefault(e6, t5) {
            this.table.fill(e6 << 4 | t5);
          }
          add(e6, t5, i7, s6) {
            this.table[t5 << 8 | e6] = i7 << 4 | s6;
          }
          addMany(e6, t5, i7, s6) {
            for (let r5 = 0; r5 < e6.length; r5++) this.table[t5 << 8 | e6[r5]] = i7 << 4 | s6;
          }
        }
        t4.TransitionTable = a3;
        const h3 = 160;
        t4.VT500_TRANSITION_TABLE = (function() {
          const e6 = new a3(4095), t5 = Array.apply(null, Array(256)).map(((e7, t6) => t6)), i7 = (e7, i8) => t5.slice(e7, i8), s6 = i7(32, 127), r5 = i7(0, 24);
          r5.push(25), r5.push.apply(r5, i7(28, 32));
          const n5 = i7(0, 14);
          let o6;
          for (o6 in e6.setDefault(1, 0), e6.addMany(s6, 0, 2, 0), n5) e6.addMany([24, 26, 153, 154], o6, 3, 0), e6.addMany(i7(128, 144), o6, 3, 0), e6.addMany(i7(144, 152), o6, 3, 0), e6.add(156, o6, 0, 0), e6.add(27, o6, 11, 1), e6.add(157, o6, 4, 8), e6.addMany([152, 158, 159], o6, 0, 7), e6.add(155, o6, 11, 3), e6.add(144, o6, 11, 9);
          return e6.addMany(r5, 0, 3, 0), e6.addMany(r5, 1, 3, 1), e6.add(127, 1, 0, 1), e6.addMany(r5, 8, 0, 8), e6.addMany(r5, 3, 3, 3), e6.add(127, 3, 0, 3), e6.addMany(r5, 4, 3, 4), e6.add(127, 4, 0, 4), e6.addMany(r5, 6, 3, 6), e6.addMany(r5, 5, 3, 5), e6.add(127, 5, 0, 5), e6.addMany(r5, 2, 3, 2), e6.add(127, 2, 0, 2), e6.add(93, 1, 4, 8), e6.addMany(s6, 8, 5, 8), e6.add(127, 8, 5, 8), e6.addMany([156, 27, 24, 26, 7], 8, 6, 0), e6.addMany(i7(28, 32), 8, 0, 8), e6.addMany([88, 94, 95], 1, 0, 7), e6.addMany(s6, 7, 0, 7), e6.addMany(r5, 7, 0, 7), e6.add(156, 7, 0, 0), e6.add(127, 7, 0, 7), e6.add(91, 1, 11, 3), e6.addMany(i7(64, 127), 3, 7, 0), e6.addMany(i7(48, 60), 3, 8, 4), e6.addMany([60, 61, 62, 63], 3, 9, 4), e6.addMany(i7(48, 60), 4, 8, 4), e6.addMany(i7(64, 127), 4, 7, 0), e6.addMany([60, 61, 62, 63], 4, 0, 6), e6.addMany(i7(32, 64), 6, 0, 6), e6.add(127, 6, 0, 6), e6.addMany(i7(64, 127), 6, 0, 0), e6.addMany(i7(32, 48), 3, 9, 5), e6.addMany(i7(32, 48), 5, 9, 5), e6.addMany(i7(48, 64), 5, 0, 6), e6.addMany(i7(64, 127), 5, 7, 0), e6.addMany(i7(32, 48), 4, 9, 5), e6.addMany(i7(32, 48), 1, 9, 2), e6.addMany(i7(32, 48), 2, 9, 2), e6.addMany(i7(48, 127), 2, 10, 0), e6.addMany(i7(48, 80), 1, 10, 0), e6.addMany(i7(81, 88), 1, 10, 0), e6.addMany([89, 90, 92], 1, 10, 0), e6.addMany(i7(96, 127), 1, 10, 0), e6.add(80, 1, 11, 9), e6.addMany(r5, 9, 0, 9), e6.add(127, 9, 0, 9), e6.addMany(i7(28, 32), 9, 0, 9), e6.addMany(i7(32, 48), 9, 9, 12), e6.addMany(i7(48, 60), 9, 8, 10), e6.addMany([60, 61, 62, 63], 9, 9, 10), e6.addMany(r5, 11, 0, 11), e6.addMany(i7(32, 128), 11, 0, 11), e6.addMany(i7(28, 32), 11, 0, 11), e6.addMany(r5, 10, 0, 10), e6.add(127, 10, 0, 10), e6.addMany(i7(28, 32), 10, 0, 10), e6.addMany(i7(48, 60), 10, 8, 10), e6.addMany([60, 61, 62, 63], 10, 0, 11), e6.addMany(i7(32, 48), 10, 9, 12), e6.addMany(r5, 12, 0, 12), e6.add(127, 12, 0, 12), e6.addMany(i7(28, 32), 12, 0, 12), e6.addMany(i7(32, 48), 12, 9, 12), e6.addMany(i7(48, 64), 12, 0, 11), e6.addMany(i7(64, 127), 12, 12, 13), e6.addMany(i7(64, 127), 10, 12, 13), e6.addMany(i7(64, 127), 9, 12, 13), e6.addMany(r5, 13, 13, 13), e6.addMany(s6, 13, 13, 13), e6.add(127, 13, 0, 13), e6.addMany([27, 156, 24, 26], 13, 14, 0), e6.add(h3, 0, 2, 0), e6.add(h3, 8, 5, 8), e6.add(h3, 6, 0, 6), e6.add(h3, 11, 0, 11), e6.add(h3, 13, 13, 13), e6;
        })();
        class c4 extends s5.Disposable {
          constructor(e6 = t4.VT500_TRANSITION_TABLE) {
            super(), this._transitions = e6, this._parseStack = { state: 0, handlers: [], handlerPos: 0, transition: 0, chunkPos: 0 }, this.initialState = 0, this.currentState = this.initialState, this._params = new r4.Params(), this._params.addParam(0), this._collect = 0, this.precedingJoinState = 0, this._printHandlerFb = (e7, t5, i7) => {
            }, this._executeHandlerFb = (e7) => {
            }, this._csiHandlerFb = (e7, t5) => {
            }, this._escHandlerFb = (e7) => {
            }, this._errorHandlerFb = (e7) => e7, this._printHandler = this._printHandlerFb, this._executeHandlers = /* @__PURE__ */ Object.create(null), this._csiHandlers = /* @__PURE__ */ Object.create(null), this._escHandlers = /* @__PURE__ */ Object.create(null), this.register((0, s5.toDisposable)((() => {
              this._csiHandlers = /* @__PURE__ */ Object.create(null), this._executeHandlers = /* @__PURE__ */ Object.create(null), this._escHandlers = /* @__PURE__ */ Object.create(null);
            }))), this._oscParser = this.register(new n4.OscParser()), this._dcsParser = this.register(new o5.DcsParser()), this._errorHandler = this._errorHandlerFb, this.registerEscHandler({ final: "\\" }, (() => true));
          }
          _identifier(e6, t5 = [64, 126]) {
            let i7 = 0;
            if (e6.prefix) {
              if (e6.prefix.length > 1) throw new Error("only one byte as prefix supported");
              if (i7 = e6.prefix.charCodeAt(0), i7 && 60 > i7 || i7 > 63) throw new Error("prefix must be in range 0x3c .. 0x3f");
            }
            if (e6.intermediates) {
              if (e6.intermediates.length > 2) throw new Error("only two bytes as intermediates are supported");
              for (let t6 = 0; t6 < e6.intermediates.length; ++t6) {
                const s7 = e6.intermediates.charCodeAt(t6);
                if (32 > s7 || s7 > 47) throw new Error("intermediate must be in range 0x20 .. 0x2f");
                i7 <<= 8, i7 |= s7;
              }
            }
            if (1 !== e6.final.length) throw new Error("final must be a single byte");
            const s6 = e6.final.charCodeAt(0);
            if (t5[0] > s6 || s6 > t5[1]) throw new Error(`final must be in range ${t5[0]} .. ${t5[1]}`);
            return i7 <<= 8, i7 |= s6, i7;
          }
          identToString(e6) {
            const t5 = [];
            for (; e6; ) t5.push(String.fromCharCode(255 & e6)), e6 >>= 8;
            return t5.reverse().join("");
          }
          setPrintHandler(e6) {
            this._printHandler = e6;
          }
          clearPrintHandler() {
            this._printHandler = this._printHandlerFb;
          }
          registerEscHandler(e6, t5) {
            const i7 = this._identifier(e6, [48, 126]);
            void 0 === this._escHandlers[i7] && (this._escHandlers[i7] = []);
            const s6 = this._escHandlers[i7];
            return s6.push(t5), { dispose: () => {
              const e7 = s6.indexOf(t5);
              -1 !== e7 && s6.splice(e7, 1);
            } };
          }
          clearEscHandler(e6) {
            this._escHandlers[this._identifier(e6, [48, 126])] && delete this._escHandlers[this._identifier(e6, [48, 126])];
          }
          setEscHandlerFallback(e6) {
            this._escHandlerFb = e6;
          }
          setExecuteHandler(e6, t5) {
            this._executeHandlers[e6.charCodeAt(0)] = t5;
          }
          clearExecuteHandler(e6) {
            this._executeHandlers[e6.charCodeAt(0)] && delete this._executeHandlers[e6.charCodeAt(0)];
          }
          setExecuteHandlerFallback(e6) {
            this._executeHandlerFb = e6;
          }
          registerCsiHandler(e6, t5) {
            const i7 = this._identifier(e6);
            void 0 === this._csiHandlers[i7] && (this._csiHandlers[i7] = []);
            const s6 = this._csiHandlers[i7];
            return s6.push(t5), { dispose: () => {
              const e7 = s6.indexOf(t5);
              -1 !== e7 && s6.splice(e7, 1);
            } };
          }
          clearCsiHandler(e6) {
            this._csiHandlers[this._identifier(e6)] && delete this._csiHandlers[this._identifier(e6)];
          }
          setCsiHandlerFallback(e6) {
            this._csiHandlerFb = e6;
          }
          registerDcsHandler(e6, t5) {
            return this._dcsParser.registerHandler(this._identifier(e6), t5);
          }
          clearDcsHandler(e6) {
            this._dcsParser.clearHandler(this._identifier(e6));
          }
          setDcsHandlerFallback(e6) {
            this._dcsParser.setHandlerFallback(e6);
          }
          registerOscHandler(e6, t5) {
            return this._oscParser.registerHandler(e6, t5);
          }
          clearOscHandler(e6) {
            this._oscParser.clearHandler(e6);
          }
          setOscHandlerFallback(e6) {
            this._oscParser.setHandlerFallback(e6);
          }
          setErrorHandler(e6) {
            this._errorHandler = e6;
          }
          clearErrorHandler() {
            this._errorHandler = this._errorHandlerFb;
          }
          reset() {
            this.currentState = this.initialState, this._oscParser.reset(), this._dcsParser.reset(), this._params.reset(), this._params.addParam(0), this._collect = 0, this.precedingJoinState = 0, 0 !== this._parseStack.state && (this._parseStack.state = 2, this._parseStack.handlers = []);
          }
          _preserveStack(e6, t5, i7, s6, r5) {
            this._parseStack.state = e6, this._parseStack.handlers = t5, this._parseStack.handlerPos = i7, this._parseStack.transition = s6, this._parseStack.chunkPos = r5;
          }
          parse(e6, t5, i7) {
            let s6, r5 = 0, n5 = 0, o6 = 0;
            if (this._parseStack.state) if (2 === this._parseStack.state) this._parseStack.state = 0, o6 = this._parseStack.chunkPos + 1;
            else {
              if (void 0 === i7 || 1 === this._parseStack.state) throw this._parseStack.state = 1, new Error("improper continuation due to previous async handler, giving up parsing");
              const t6 = this._parseStack.handlers;
              let n6 = this._parseStack.handlerPos - 1;
              switch (this._parseStack.state) {
                case 3:
                  if (false === i7 && n6 > -1) {
                    for (; n6 >= 0 && (s6 = t6[n6](this._params), true !== s6); n6--) if (s6 instanceof Promise) return this._parseStack.handlerPos = n6, s6;
                  }
                  this._parseStack.handlers = [];
                  break;
                case 4:
                  if (false === i7 && n6 > -1) {
                    for (; n6 >= 0 && (s6 = t6[n6](), true !== s6); n6--) if (s6 instanceof Promise) return this._parseStack.handlerPos = n6, s6;
                  }
                  this._parseStack.handlers = [];
                  break;
                case 6:
                  if (r5 = e6[this._parseStack.chunkPos], s6 = this._dcsParser.unhook(24 !== r5 && 26 !== r5, i7), s6) return s6;
                  27 === r5 && (this._parseStack.transition |= 1), this._params.reset(), this._params.addParam(0), this._collect = 0;
                  break;
                case 5:
                  if (r5 = e6[this._parseStack.chunkPos], s6 = this._oscParser.end(24 !== r5 && 26 !== r5, i7), s6) return s6;
                  27 === r5 && (this._parseStack.transition |= 1), this._params.reset(), this._params.addParam(0), this._collect = 0;
              }
              this._parseStack.state = 0, o6 = this._parseStack.chunkPos + 1, this.precedingJoinState = 0, this.currentState = 15 & this._parseStack.transition;
            }
            for (let i8 = o6; i8 < t5; ++i8) {
              switch (r5 = e6[i8], n5 = this._transitions.table[this.currentState << 8 | (r5 < 160 ? r5 : h3)], n5 >> 4) {
                case 2:
                  for (let s7 = i8 + 1; ; ++s7) {
                    if (s7 >= t5 || (r5 = e6[s7]) < 32 || r5 > 126 && r5 < h3) {
                      this._printHandler(e6, i8, s7), i8 = s7 - 1;
                      break;
                    }
                    if (++s7 >= t5 || (r5 = e6[s7]) < 32 || r5 > 126 && r5 < h3) {
                      this._printHandler(e6, i8, s7), i8 = s7 - 1;
                      break;
                    }
                    if (++s7 >= t5 || (r5 = e6[s7]) < 32 || r5 > 126 && r5 < h3) {
                      this._printHandler(e6, i8, s7), i8 = s7 - 1;
                      break;
                    }
                    if (++s7 >= t5 || (r5 = e6[s7]) < 32 || r5 > 126 && r5 < h3) {
                      this._printHandler(e6, i8, s7), i8 = s7 - 1;
                      break;
                    }
                  }
                  break;
                case 3:
                  this._executeHandlers[r5] ? this._executeHandlers[r5]() : this._executeHandlerFb(r5), this.precedingJoinState = 0;
                  break;
                case 0:
                  break;
                case 1:
                  if (this._errorHandler({ position: i8, code: r5, currentState: this.currentState, collect: this._collect, params: this._params, abort: false }).abort) return;
                  break;
                case 7:
                  const o7 = this._csiHandlers[this._collect << 8 | r5];
                  let a4 = o7 ? o7.length - 1 : -1;
                  for (; a4 >= 0 && (s6 = o7[a4](this._params), true !== s6); a4--) if (s6 instanceof Promise) return this._preserveStack(3, o7, a4, n5, i8), s6;
                  a4 < 0 && this._csiHandlerFb(this._collect << 8 | r5, this._params), this.precedingJoinState = 0;
                  break;
                case 8:
                  do {
                    switch (r5) {
                      case 59:
                        this._params.addParam(0);
                        break;
                      case 58:
                        this._params.addSubParam(-1);
                        break;
                      default:
                        this._params.addDigit(r5 - 48);
                    }
                  } while (++i8 < t5 && (r5 = e6[i8]) > 47 && r5 < 60);
                  i8--;
                  break;
                case 9:
                  this._collect <<= 8, this._collect |= r5;
                  break;
                case 10:
                  const c5 = this._escHandlers[this._collect << 8 | r5];
                  let l3 = c5 ? c5.length - 1 : -1;
                  for (; l3 >= 0 && (s6 = c5[l3](), true !== s6); l3--) if (s6 instanceof Promise) return this._preserveStack(4, c5, l3, n5, i8), s6;
                  l3 < 0 && this._escHandlerFb(this._collect << 8 | r5), this.precedingJoinState = 0;
                  break;
                case 11:
                  this._params.reset(), this._params.addParam(0), this._collect = 0;
                  break;
                case 12:
                  this._dcsParser.hook(this._collect << 8 | r5, this._params);
                  break;
                case 13:
                  for (let s7 = i8 + 1; ; ++s7) if (s7 >= t5 || 24 === (r5 = e6[s7]) || 26 === r5 || 27 === r5 || r5 > 127 && r5 < h3) {
                    this._dcsParser.put(e6, i8, s7), i8 = s7 - 1;
                    break;
                  }
                  break;
                case 14:
                  if (s6 = this._dcsParser.unhook(24 !== r5 && 26 !== r5), s6) return this._preserveStack(6, [], 0, n5, i8), s6;
                  27 === r5 && (n5 |= 1), this._params.reset(), this._params.addParam(0), this._collect = 0, this.precedingJoinState = 0;
                  break;
                case 4:
                  this._oscParser.start();
                  break;
                case 5:
                  for (let s7 = i8 + 1; ; s7++) if (s7 >= t5 || (r5 = e6[s7]) < 32 || r5 > 127 && r5 < h3) {
                    this._oscParser.put(e6, i8, s7), i8 = s7 - 1;
                    break;
                  }
                  break;
                case 6:
                  if (s6 = this._oscParser.end(24 !== r5 && 26 !== r5), s6) return this._preserveStack(5, [], 0, n5, i8), s6;
                  27 === r5 && (n5 |= 1), this._params.reset(), this._params.addParam(0), this._collect = 0, this.precedingJoinState = 0;
              }
              this.currentState = 15 & n5;
            }
          }
        }
        t4.EscapeSequenceParser = c4;
      }, 6242: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.OscHandler = t4.OscParser = void 0;
        const s5 = i6(5770), r4 = i6(482), n4 = [];
        t4.OscParser = class {
          constructor() {
            this._state = 0, this._active = n4, this._id = -1, this._handlers = /* @__PURE__ */ Object.create(null), this._handlerFb = () => {
            }, this._stack = { paused: false, loopPosition: 0, fallThrough: false };
          }
          registerHandler(e6, t5) {
            void 0 === this._handlers[e6] && (this._handlers[e6] = []);
            const i7 = this._handlers[e6];
            return i7.push(t5), { dispose: () => {
              const e7 = i7.indexOf(t5);
              -1 !== e7 && i7.splice(e7, 1);
            } };
          }
          clearHandler(e6) {
            this._handlers[e6] && delete this._handlers[e6];
          }
          setHandlerFallback(e6) {
            this._handlerFb = e6;
          }
          dispose() {
            this._handlers = /* @__PURE__ */ Object.create(null), this._handlerFb = () => {
            }, this._active = n4;
          }
          reset() {
            if (2 === this._state) for (let e6 = this._stack.paused ? this._stack.loopPosition - 1 : this._active.length - 1; e6 >= 0; --e6) this._active[e6].end(false);
            this._stack.paused = false, this._active = n4, this._id = -1, this._state = 0;
          }
          _start() {
            if (this._active = this._handlers[this._id] || n4, this._active.length) for (let e6 = this._active.length - 1; e6 >= 0; e6--) this._active[e6].start();
            else this._handlerFb(this._id, "START");
          }
          _put(e6, t5, i7) {
            if (this._active.length) for (let s6 = this._active.length - 1; s6 >= 0; s6--) this._active[s6].put(e6, t5, i7);
            else this._handlerFb(this._id, "PUT", (0, r4.utf32ToString)(e6, t5, i7));
          }
          start() {
            this.reset(), this._state = 1;
          }
          put(e6, t5, i7) {
            if (3 !== this._state) {
              if (1 === this._state) for (; t5 < i7; ) {
                const i8 = e6[t5++];
                if (59 === i8) {
                  this._state = 2, this._start();
                  break;
                }
                if (i8 < 48 || 57 < i8) return void (this._state = 3);
                -1 === this._id && (this._id = 0), this._id = 10 * this._id + i8 - 48;
              }
              2 === this._state && i7 - t5 > 0 && this._put(e6, t5, i7);
            }
          }
          end(e6, t5 = true) {
            if (0 !== this._state) {
              if (3 !== this._state) if (1 === this._state && this._start(), this._active.length) {
                let i7 = false, s6 = this._active.length - 1, r5 = false;
                if (this._stack.paused && (s6 = this._stack.loopPosition - 1, i7 = t5, r5 = this._stack.fallThrough, this._stack.paused = false), !r5 && false === i7) {
                  for (; s6 >= 0 && (i7 = this._active[s6].end(e6), true !== i7); s6--) if (i7 instanceof Promise) return this._stack.paused = true, this._stack.loopPosition = s6, this._stack.fallThrough = false, i7;
                  s6--;
                }
                for (; s6 >= 0; s6--) if (i7 = this._active[s6].end(false), i7 instanceof Promise) return this._stack.paused = true, this._stack.loopPosition = s6, this._stack.fallThrough = true, i7;
              } else this._handlerFb(this._id, "END", e6);
              this._active = n4, this._id = -1, this._state = 0;
            }
          }
        }, t4.OscHandler = class {
          constructor(e6) {
            this._handler = e6, this._data = "", this._hitLimit = false;
          }
          start() {
            this._data = "", this._hitLimit = false;
          }
          put(e6, t5, i7) {
            this._hitLimit || (this._data += (0, r4.utf32ToString)(e6, t5, i7), this._data.length > s5.PAYLOAD_LIMIT && (this._data = "", this._hitLimit = true));
          }
          end(e6) {
            let t5 = false;
            if (this._hitLimit) t5 = false;
            else if (e6 && (t5 = this._handler(this._data), t5 instanceof Promise)) return t5.then(((e7) => (this._data = "", this._hitLimit = false, e7)));
            return this._data = "", this._hitLimit = false, t5;
          }
        };
      }, 8742: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.Params = void 0;
        const i6 = 2147483647;
        class s5 {
          static fromArray(e6) {
            const t5 = new s5();
            if (!e6.length) return t5;
            for (let i7 = Array.isArray(e6[0]) ? 1 : 0; i7 < e6.length; ++i7) {
              const s6 = e6[i7];
              if (Array.isArray(s6)) for (let e7 = 0; e7 < s6.length; ++e7) t5.addSubParam(s6[e7]);
              else t5.addParam(s6);
            }
            return t5;
          }
          constructor(e6 = 32, t5 = 32) {
            if (this.maxLength = e6, this.maxSubParamsLength = t5, t5 > 256) throw new Error("maxSubParamsLength must not be greater than 256");
            this.params = new Int32Array(e6), this.length = 0, this._subParams = new Int32Array(t5), this._subParamsLength = 0, this._subParamsIdx = new Uint16Array(e6), this._rejectDigits = false, this._rejectSubDigits = false, this._digitIsSub = false;
          }
          clone() {
            const e6 = new s5(this.maxLength, this.maxSubParamsLength);
            return e6.params.set(this.params), e6.length = this.length, e6._subParams.set(this._subParams), e6._subParamsLength = this._subParamsLength, e6._subParamsIdx.set(this._subParamsIdx), e6._rejectDigits = this._rejectDigits, e6._rejectSubDigits = this._rejectSubDigits, e6._digitIsSub = this._digitIsSub, e6;
          }
          toArray() {
            const e6 = [];
            for (let t5 = 0; t5 < this.length; ++t5) {
              e6.push(this.params[t5]);
              const i7 = this._subParamsIdx[t5] >> 8, s6 = 255 & this._subParamsIdx[t5];
              s6 - i7 > 0 && e6.push(Array.prototype.slice.call(this._subParams, i7, s6));
            }
            return e6;
          }
          reset() {
            this.length = 0, this._subParamsLength = 0, this._rejectDigits = false, this._rejectSubDigits = false, this._digitIsSub = false;
          }
          addParam(e6) {
            if (this._digitIsSub = false, this.length >= this.maxLength) this._rejectDigits = true;
            else {
              if (e6 < -1) throw new Error("values lesser than -1 are not allowed");
              this._subParamsIdx[this.length] = this._subParamsLength << 8 | this._subParamsLength, this.params[this.length++] = e6 > i6 ? i6 : e6;
            }
          }
          addSubParam(e6) {
            if (this._digitIsSub = true, this.length) if (this._rejectDigits || this._subParamsLength >= this.maxSubParamsLength) this._rejectSubDigits = true;
            else {
              if (e6 < -1) throw new Error("values lesser than -1 are not allowed");
              this._subParams[this._subParamsLength++] = e6 > i6 ? i6 : e6, this._subParamsIdx[this.length - 1]++;
            }
          }
          hasSubParams(e6) {
            return (255 & this._subParamsIdx[e6]) - (this._subParamsIdx[e6] >> 8) > 0;
          }
          getSubParams(e6) {
            const t5 = this._subParamsIdx[e6] >> 8, i7 = 255 & this._subParamsIdx[e6];
            return i7 - t5 > 0 ? this._subParams.subarray(t5, i7) : null;
          }
          getSubParamsAll() {
            const e6 = {};
            for (let t5 = 0; t5 < this.length; ++t5) {
              const i7 = this._subParamsIdx[t5] >> 8, s6 = 255 & this._subParamsIdx[t5];
              s6 - i7 > 0 && (e6[t5] = this._subParams.slice(i7, s6));
            }
            return e6;
          }
          addDigit(e6) {
            let t5;
            if (this._rejectDigits || !(t5 = this._digitIsSub ? this._subParamsLength : this.length) || this._digitIsSub && this._rejectSubDigits) return;
            const s6 = this._digitIsSub ? this._subParams : this.params, r4 = s6[t5 - 1];
            s6[t5 - 1] = ~r4 ? Math.min(10 * r4 + e6, i6) : e6;
          }
        }
        t4.Params = s5;
      }, 5741: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.AddonManager = void 0, t4.AddonManager = class {
          constructor() {
            this._addons = [];
          }
          dispose() {
            for (let e6 = this._addons.length - 1; e6 >= 0; e6--) this._addons[e6].instance.dispose();
          }
          loadAddon(e6, t5) {
            const i6 = { instance: t5, dispose: t5.dispose, isDisposed: false };
            this._addons.push(i6), t5.dispose = () => this._wrappedAddonDispose(i6), t5.activate(e6);
          }
          _wrappedAddonDispose(e6) {
            if (e6.isDisposed) return;
            let t5 = -1;
            for (let i6 = 0; i6 < this._addons.length; i6++) if (this._addons[i6] === e6) {
              t5 = i6;
              break;
            }
            if (-1 === t5) throw new Error("Could not dispose an addon that has not been loaded");
            e6.isDisposed = true, e6.dispose.apply(e6.instance), this._addons.splice(t5, 1);
          }
        };
      }, 8771: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.BufferApiView = void 0;
        const s5 = i6(3785), r4 = i6(511);
        t4.BufferApiView = class {
          constructor(e6, t5) {
            this._buffer = e6, this.type = t5;
          }
          init(e6) {
            return this._buffer = e6, this;
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
          getLine(e6) {
            const t5 = this._buffer.lines.get(e6);
            if (t5) return new s5.BufferLineApiView(t5);
          }
          getNullCell() {
            return new r4.CellData();
          }
        };
      }, 3785: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.BufferLineApiView = void 0;
        const s5 = i6(511);
        t4.BufferLineApiView = class {
          constructor(e6) {
            this._line = e6;
          }
          get isWrapped() {
            return this._line.isWrapped;
          }
          get length() {
            return this._line.length;
          }
          getCell(e6, t5) {
            if (!(e6 < 0 || e6 >= this._line.length)) return t5 ? (this._line.loadCell(e6, t5), t5) : this._line.loadCell(e6, new s5.CellData());
          }
          translateToString(e6, t5, i7) {
            return this._line.translateToString(e6, t5, i7);
          }
        };
      }, 8285: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.BufferNamespaceApi = void 0;
        const s5 = i6(8771), r4 = i6(8460), n4 = i6(844);
        class o5 extends n4.Disposable {
          constructor(e6) {
            super(), this._core = e6, this._onBufferChange = this.register(new r4.EventEmitter()), this.onBufferChange = this._onBufferChange.event, this._normal = new s5.BufferApiView(this._core.buffers.normal, "normal"), this._alternate = new s5.BufferApiView(this._core.buffers.alt, "alternate"), this._core.buffers.onBufferActivate((() => this._onBufferChange.fire(this.active)));
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
        }
        t4.BufferNamespaceApi = o5;
      }, 7975: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.ParserApi = void 0, t4.ParserApi = class {
          constructor(e6) {
            this._core = e6;
          }
          registerCsiHandler(e6, t5) {
            return this._core.registerCsiHandler(e6, ((e7) => t5(e7.toArray())));
          }
          addCsiHandler(e6, t5) {
            return this.registerCsiHandler(e6, t5);
          }
          registerDcsHandler(e6, t5) {
            return this._core.registerDcsHandler(e6, ((e7, i6) => t5(e7, i6.toArray())));
          }
          addDcsHandler(e6, t5) {
            return this.registerDcsHandler(e6, t5);
          }
          registerEscHandler(e6, t5) {
            return this._core.registerEscHandler(e6, t5);
          }
          addEscHandler(e6, t5) {
            return this.registerEscHandler(e6, t5);
          }
          registerOscHandler(e6, t5) {
            return this._core.registerOscHandler(e6, t5);
          }
          addOscHandler(e6, t5) {
            return this.registerOscHandler(e6, t5);
          }
        };
      }, 7090: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.UnicodeApi = void 0, t4.UnicodeApi = class {
          constructor(e6) {
            this._core = e6;
          }
          register(e6) {
            this._core.unicodeService.register(e6);
          }
          get versions() {
            return this._core.unicodeService.versions;
          }
          get activeVersion() {
            return this._core.unicodeService.activeVersion;
          }
          set activeVersion(e6) {
            this._core.unicodeService.activeVersion = e6;
          }
        };
      }, 744: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.BufferService = t4.MINIMUM_ROWS = t4.MINIMUM_COLS = void 0;
        const n4 = i6(8460), o5 = i6(844), a3 = i6(5295), h3 = i6(2585);
        t4.MINIMUM_COLS = 2, t4.MINIMUM_ROWS = 1;
        let c4 = t4.BufferService = class extends o5.Disposable {
          get buffer() {
            return this.buffers.active;
          }
          constructor(e6) {
            super(), this.isUserScrolling = false, this._onResize = this.register(new n4.EventEmitter()), this.onResize = this._onResize.event, this._onScroll = this.register(new n4.EventEmitter()), this.onScroll = this._onScroll.event, this.cols = Math.max(e6.rawOptions.cols || 0, t4.MINIMUM_COLS), this.rows = Math.max(e6.rawOptions.rows || 0, t4.MINIMUM_ROWS), this.buffers = this.register(new a3.BufferSet(e6, this));
          }
          resize(e6, t5) {
            this.cols = e6, this.rows = t5, this.buffers.resize(e6, t5), this._onResize.fire({ cols: e6, rows: t5 });
          }
          reset() {
            this.buffers.reset(), this.isUserScrolling = false;
          }
          scroll(e6, t5 = false) {
            const i7 = this.buffer;
            let s6;
            s6 = this._cachedBlankLine, s6 && s6.length === this.cols && s6.getFg(0) === e6.fg && s6.getBg(0) === e6.bg || (s6 = i7.getBlankLine(e6, t5), this._cachedBlankLine = s6), s6.isWrapped = t5;
            const r5 = i7.ybase + i7.scrollTop, n5 = i7.ybase + i7.scrollBottom;
            if (0 === i7.scrollTop) {
              const e7 = i7.lines.isFull;
              n5 === i7.lines.length - 1 ? e7 ? i7.lines.recycle().copyFrom(s6) : i7.lines.push(s6.clone()) : i7.lines.splice(n5 + 1, 0, s6.clone()), e7 ? this.isUserScrolling && (i7.ydisp = Math.max(i7.ydisp - 1, 0)) : (i7.ybase++, this.isUserScrolling || i7.ydisp++);
            } else {
              const e7 = n5 - r5 + 1;
              i7.lines.shiftElements(r5 + 1, e7 - 1, -1), i7.lines.set(n5, s6.clone());
            }
            this.isUserScrolling || (i7.ydisp = i7.ybase), this._onScroll.fire(i7.ydisp);
          }
          scrollLines(e6, t5, i7) {
            const s6 = this.buffer;
            if (e6 < 0) {
              if (0 === s6.ydisp) return;
              this.isUserScrolling = true;
            } else e6 + s6.ydisp >= s6.ybase && (this.isUserScrolling = false);
            const r5 = s6.ydisp;
            s6.ydisp = Math.max(Math.min(s6.ydisp + e6, s6.ybase), 0), r5 !== s6.ydisp && (t5 || this._onScroll.fire(s6.ydisp));
          }
        };
        t4.BufferService = c4 = s5([r4(0, h3.IOptionsService)], c4);
      }, 7994: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.CharsetService = void 0, t4.CharsetService = class {
          constructor() {
            this.glevel = 0, this._charsets = [];
          }
          reset() {
            this.charset = void 0, this._charsets = [], this.glevel = 0;
          }
          setgLevel(e6) {
            this.glevel = e6, this.charset = this._charsets[e6];
          }
          setgCharset(e6, t5) {
            this._charsets[e6] = t5, this.glevel === e6 && (this.charset = t5);
          }
        };
      }, 1753: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.CoreMouseService = void 0;
        const n4 = i6(2585), o5 = i6(8460), a3 = i6(844), h3 = { NONE: { events: 0, restrict: () => false }, X10: { events: 1, restrict: (e6) => 4 !== e6.button && 1 === e6.action && (e6.ctrl = false, e6.alt = false, e6.shift = false, true) }, VT200: { events: 19, restrict: (e6) => 32 !== e6.action }, DRAG: { events: 23, restrict: (e6) => 32 !== e6.action || 3 !== e6.button }, ANY: { events: 31, restrict: (e6) => true } };
        function c4(e6, t5) {
          let i7 = (e6.ctrl ? 16 : 0) | (e6.shift ? 4 : 0) | (e6.alt ? 8 : 0);
          return 4 === e6.button ? (i7 |= 64, i7 |= e6.action) : (i7 |= 3 & e6.button, 4 & e6.button && (i7 |= 64), 8 & e6.button && (i7 |= 128), 32 === e6.action ? i7 |= 32 : 0 !== e6.action || t5 || (i7 |= 3)), i7;
        }
        const l3 = String.fromCharCode, d3 = { DEFAULT: (e6) => {
          const t5 = [c4(e6, false) + 32, e6.col + 32, e6.row + 32];
          return t5[0] > 255 || t5[1] > 255 || t5[2] > 255 ? "" : `\x1B[M${l3(t5[0])}${l3(t5[1])}${l3(t5[2])}`;
        }, SGR: (e6) => {
          const t5 = 0 === e6.action && 4 !== e6.button ? "m" : "M";
          return `\x1B[<${c4(e6, true)};${e6.col};${e6.row}${t5}`;
        }, SGR_PIXELS: (e6) => {
          const t5 = 0 === e6.action && 4 !== e6.button ? "m" : "M";
          return `\x1B[<${c4(e6, true)};${e6.x};${e6.y}${t5}`;
        } };
        let _2 = t4.CoreMouseService = class extends a3.Disposable {
          constructor(e6, t5) {
            super(), this._bufferService = e6, this._coreService = t5, this._protocols = {}, this._encodings = {}, this._activeProtocol = "", this._activeEncoding = "", this._lastEvent = null, this._onProtocolChange = this.register(new o5.EventEmitter()), this.onProtocolChange = this._onProtocolChange.event;
            for (const e7 of Object.keys(h3)) this.addProtocol(e7, h3[e7]);
            for (const e7 of Object.keys(d3)) this.addEncoding(e7, d3[e7]);
            this.reset();
          }
          addProtocol(e6, t5) {
            this._protocols[e6] = t5;
          }
          addEncoding(e6, t5) {
            this._encodings[e6] = t5;
          }
          get activeProtocol() {
            return this._activeProtocol;
          }
          get areMouseEventsActive() {
            return 0 !== this._protocols[this._activeProtocol].events;
          }
          set activeProtocol(e6) {
            if (!this._protocols[e6]) throw new Error(`unknown protocol "${e6}"`);
            this._activeProtocol = e6, this._onProtocolChange.fire(this._protocols[e6].events);
          }
          get activeEncoding() {
            return this._activeEncoding;
          }
          set activeEncoding(e6) {
            if (!this._encodings[e6]) throw new Error(`unknown encoding "${e6}"`);
            this._activeEncoding = e6;
          }
          reset() {
            this.activeProtocol = "NONE", this.activeEncoding = "DEFAULT", this._lastEvent = null;
          }
          triggerMouseEvent(e6) {
            if (e6.col < 0 || e6.col >= this._bufferService.cols || e6.row < 0 || e6.row >= this._bufferService.rows) return false;
            if (4 === e6.button && 32 === e6.action) return false;
            if (3 === e6.button && 32 !== e6.action) return false;
            if (4 !== e6.button && (2 === e6.action || 3 === e6.action)) return false;
            if (e6.col++, e6.row++, 32 === e6.action && this._lastEvent && this._equalEvents(this._lastEvent, e6, "SGR_PIXELS" === this._activeEncoding)) return false;
            if (!this._protocols[this._activeProtocol].restrict(e6)) return false;
            const t5 = this._encodings[this._activeEncoding](e6);
            return t5 && ("DEFAULT" === this._activeEncoding ? this._coreService.triggerBinaryEvent(t5) : this._coreService.triggerDataEvent(t5, true)), this._lastEvent = e6, true;
          }
          explainEvents(e6) {
            return { down: !!(1 & e6), up: !!(2 & e6), drag: !!(4 & e6), move: !!(8 & e6), wheel: !!(16 & e6) };
          }
          _equalEvents(e6, t5, i7) {
            if (i7) {
              if (e6.x !== t5.x) return false;
              if (e6.y !== t5.y) return false;
            } else {
              if (e6.col !== t5.col) return false;
              if (e6.row !== t5.row) return false;
            }
            return e6.button === t5.button && e6.action === t5.action && e6.ctrl === t5.ctrl && e6.alt === t5.alt && e6.shift === t5.shift;
          }
        };
        t4.CoreMouseService = _2 = s5([r4(0, n4.IBufferService), r4(1, n4.ICoreService)], _2);
      }, 6975: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.CoreService = void 0;
        const n4 = i6(1439), o5 = i6(8460), a3 = i6(844), h3 = i6(2585), c4 = Object.freeze({ insertMode: false }), l3 = Object.freeze({ applicationCursorKeys: false, applicationKeypad: false, bracketedPasteMode: false, origin: false, reverseWraparound: false, sendFocus: false, wraparound: true });
        let d3 = t4.CoreService = class extends a3.Disposable {
          constructor(e6, t5, i7) {
            super(), this._bufferService = e6, this._logService = t5, this._optionsService = i7, this.isCursorInitialized = false, this.isCursorHidden = false, this._onData = this.register(new o5.EventEmitter()), this.onData = this._onData.event, this._onUserInput = this.register(new o5.EventEmitter()), this.onUserInput = this._onUserInput.event, this._onBinary = this.register(new o5.EventEmitter()), this.onBinary = this._onBinary.event, this._onRequestScrollToBottom = this.register(new o5.EventEmitter()), this.onRequestScrollToBottom = this._onRequestScrollToBottom.event, this.modes = (0, n4.clone)(c4), this.decPrivateModes = (0, n4.clone)(l3);
          }
          reset() {
            this.modes = (0, n4.clone)(c4), this.decPrivateModes = (0, n4.clone)(l3);
          }
          triggerDataEvent(e6, t5 = false) {
            if (this._optionsService.rawOptions.disableStdin) return;
            const i7 = this._bufferService.buffer;
            t5 && this._optionsService.rawOptions.scrollOnUserInput && i7.ybase !== i7.ydisp && this._onRequestScrollToBottom.fire(), t5 && this._onUserInput.fire(), this._logService.debug(`sending data "${e6}"`, (() => e6.split("").map(((e7) => e7.charCodeAt(0))))), this._onData.fire(e6);
          }
          triggerBinaryEvent(e6) {
            this._optionsService.rawOptions.disableStdin || (this._logService.debug(`sending binary "${e6}"`, (() => e6.split("").map(((e7) => e7.charCodeAt(0))))), this._onBinary.fire(e6));
          }
        };
        t4.CoreService = d3 = s5([r4(0, h3.IBufferService), r4(1, h3.ILogService), r4(2, h3.IOptionsService)], d3);
      }, 9074: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.DecorationService = void 0;
        const s5 = i6(8055), r4 = i6(8460), n4 = i6(844), o5 = i6(6106);
        let a3 = 0, h3 = 0;
        class c4 extends n4.Disposable {
          get decorations() {
            return this._decorations.values();
          }
          constructor() {
            super(), this._decorations = new o5.SortedList(((e6) => e6?.marker.line)), this._onDecorationRegistered = this.register(new r4.EventEmitter()), this.onDecorationRegistered = this._onDecorationRegistered.event, this._onDecorationRemoved = this.register(new r4.EventEmitter()), this.onDecorationRemoved = this._onDecorationRemoved.event, this.register((0, n4.toDisposable)((() => this.reset())));
          }
          registerDecoration(e6) {
            if (e6.marker.isDisposed) return;
            const t5 = new l3(e6);
            if (t5) {
              const e7 = t5.marker.onDispose((() => t5.dispose()));
              t5.onDispose((() => {
                t5 && (this._decorations.delete(t5) && this._onDecorationRemoved.fire(t5), e7.dispose());
              })), this._decorations.insert(t5), this._onDecorationRegistered.fire(t5);
            }
            return t5;
          }
          reset() {
            for (const e6 of this._decorations.values()) e6.dispose();
            this._decorations.clear();
          }
          *getDecorationsAtCell(e6, t5, i7) {
            let s6 = 0, r5 = 0;
            for (const n5 of this._decorations.getKeyIterator(t5)) s6 = n5.options.x ?? 0, r5 = s6 + (n5.options.width ?? 1), e6 >= s6 && e6 < r5 && (!i7 || (n5.options.layer ?? "bottom") === i7) && (yield n5);
          }
          forEachDecorationAtCell(e6, t5, i7, s6) {
            this._decorations.forEachByKey(t5, ((t6) => {
              a3 = t6.options.x ?? 0, h3 = a3 + (t6.options.width ?? 1), e6 >= a3 && e6 < h3 && (!i7 || (t6.options.layer ?? "bottom") === i7) && s6(t6);
            }));
          }
        }
        t4.DecorationService = c4;
        class l3 extends n4.Disposable {
          get isDisposed() {
            return this._isDisposed;
          }
          get backgroundColorRGB() {
            return null === this._cachedBg && (this.options.backgroundColor ? this._cachedBg = s5.css.toColor(this.options.backgroundColor) : this._cachedBg = void 0), this._cachedBg;
          }
          get foregroundColorRGB() {
            return null === this._cachedFg && (this.options.foregroundColor ? this._cachedFg = s5.css.toColor(this.options.foregroundColor) : this._cachedFg = void 0), this._cachedFg;
          }
          constructor(e6) {
            super(), this.options = e6, this.onRenderEmitter = this.register(new r4.EventEmitter()), this.onRender = this.onRenderEmitter.event, this._onDispose = this.register(new r4.EventEmitter()), this.onDispose = this._onDispose.event, this._cachedBg = null, this._cachedFg = null, this.marker = e6.marker, this.options.overviewRulerOptions && !this.options.overviewRulerOptions.position && (this.options.overviewRulerOptions.position = "full");
          }
          dispose() {
            this._onDispose.fire(), super.dispose();
          }
        }
      }, 4348: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.InstantiationService = t4.ServiceCollection = void 0;
        const s5 = i6(2585), r4 = i6(8343);
        class n4 {
          constructor(...e6) {
            this._entries = /* @__PURE__ */ new Map();
            for (const [t5, i7] of e6) this.set(t5, i7);
          }
          set(e6, t5) {
            const i7 = this._entries.get(e6);
            return this._entries.set(e6, t5), i7;
          }
          forEach(e6) {
            for (const [t5, i7] of this._entries.entries()) e6(t5, i7);
          }
          has(e6) {
            return this._entries.has(e6);
          }
          get(e6) {
            return this._entries.get(e6);
          }
        }
        t4.ServiceCollection = n4, t4.InstantiationService = class {
          constructor() {
            this._services = new n4(), this._services.set(s5.IInstantiationService, this);
          }
          setService(e6, t5) {
            this._services.set(e6, t5);
          }
          getService(e6) {
            return this._services.get(e6);
          }
          createInstance(e6, ...t5) {
            const i7 = (0, r4.getServiceDependencies)(e6).sort(((e7, t6) => e7.index - t6.index)), s6 = [];
            for (const t6 of i7) {
              const i8 = this._services.get(t6.id);
              if (!i8) throw new Error(`[createInstance] ${e6.name} depends on UNKNOWN service ${t6.id}.`);
              s6.push(i8);
            }
            const n5 = i7.length > 0 ? i7[0].index : t5.length;
            if (t5.length !== n5) throw new Error(`[createInstance] First service dependency of ${e6.name} at position ${n5 + 1} conflicts with ${t5.length} static arguments`);
            return new e6(...[...t5, ...s6]);
          }
        };
      }, 7866: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a4 = e6.length - 1; a4 >= 0; a4--) (r5 = e6[a4]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.traceCall = t4.setTraceLogger = t4.LogService = void 0;
        const n4 = i6(844), o5 = i6(2585), a3 = { trace: o5.LogLevelEnum.TRACE, debug: o5.LogLevelEnum.DEBUG, info: o5.LogLevelEnum.INFO, warn: o5.LogLevelEnum.WARN, error: o5.LogLevelEnum.ERROR, off: o5.LogLevelEnum.OFF };
        let h3, c4 = t4.LogService = class extends n4.Disposable {
          get logLevel() {
            return this._logLevel;
          }
          constructor(e6) {
            super(), this._optionsService = e6, this._logLevel = o5.LogLevelEnum.OFF, this._updateLogLevel(), this.register(this._optionsService.onSpecificOptionChange("logLevel", (() => this._updateLogLevel()))), h3 = this;
          }
          _updateLogLevel() {
            this._logLevel = a3[this._optionsService.rawOptions.logLevel];
          }
          _evalLazyOptionalParams(e6) {
            for (let t5 = 0; t5 < e6.length; t5++) "function" == typeof e6[t5] && (e6[t5] = e6[t5]());
          }
          _log(e6, t5, i7) {
            this._evalLazyOptionalParams(i7), e6.call(console, (this._optionsService.options.logger ? "" : "xterm.js: ") + t5, ...i7);
          }
          trace(e6, ...t5) {
            this._logLevel <= o5.LogLevelEnum.TRACE && this._log(this._optionsService.options.logger?.trace.bind(this._optionsService.options.logger) ?? console.log, e6, t5);
          }
          debug(e6, ...t5) {
            this._logLevel <= o5.LogLevelEnum.DEBUG && this._log(this._optionsService.options.logger?.debug.bind(this._optionsService.options.logger) ?? console.log, e6, t5);
          }
          info(e6, ...t5) {
            this._logLevel <= o5.LogLevelEnum.INFO && this._log(this._optionsService.options.logger?.info.bind(this._optionsService.options.logger) ?? console.info, e6, t5);
          }
          warn(e6, ...t5) {
            this._logLevel <= o5.LogLevelEnum.WARN && this._log(this._optionsService.options.logger?.warn.bind(this._optionsService.options.logger) ?? console.warn, e6, t5);
          }
          error(e6, ...t5) {
            this._logLevel <= o5.LogLevelEnum.ERROR && this._log(this._optionsService.options.logger?.error.bind(this._optionsService.options.logger) ?? console.error, e6, t5);
          }
        };
        t4.LogService = c4 = s5([r4(0, o5.IOptionsService)], c4), t4.setTraceLogger = function(e6) {
          h3 = e6;
        }, t4.traceCall = function(e6, t5, i7) {
          if ("function" != typeof i7.value) throw new Error("not supported");
          const s6 = i7.value;
          i7.value = function(...e7) {
            if (h3.logLevel !== o5.LogLevelEnum.TRACE) return s6.apply(this, e7);
            h3.trace(`GlyphRenderer#${s6.name}(${e7.map(((e8) => JSON.stringify(e8))).join(", ")})`);
            const t6 = s6.apply(this, e7);
            return h3.trace(`GlyphRenderer#${s6.name} return`, t6), t6;
          };
        };
      }, 7302: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.OptionsService = t4.DEFAULT_OPTIONS = void 0;
        const s5 = i6(8460), r4 = i6(844), n4 = i6(6114);
        t4.DEFAULT_OPTIONS = { cols: 80, rows: 24, cursorBlink: false, cursorStyle: "block", cursorWidth: 1, cursorInactiveStyle: "outline", customGlyphs: true, drawBoldTextInBrightColors: true, documentOverride: null, fastScrollModifier: "alt", fastScrollSensitivity: 5, fontFamily: "courier-new, courier, monospace", fontSize: 15, fontWeight: "normal", fontWeightBold: "bold", ignoreBracketedPasteMode: false, lineHeight: 1, letterSpacing: 0, linkHandler: null, logLevel: "info", logger: null, scrollback: 1e3, scrollOnUserInput: true, scrollSensitivity: 1, screenReaderMode: false, smoothScrollDuration: 0, macOptionIsMeta: false, macOptionClickForcesSelection: false, minimumContrastRatio: 1, disableStdin: false, allowProposedApi: false, allowTransparency: false, tabStopWidth: 8, theme: {}, rescaleOverlappingGlyphs: false, rightClickSelectsWord: n4.isMac, windowOptions: {}, windowsMode: false, windowsPty: {}, wordSeparator: " ()[]{}',\"`", altClickMovesCursor: true, convertEol: false, termName: "xterm", cancelEvents: false, overviewRulerWidth: 0 };
        const o5 = ["normal", "bold", "100", "200", "300", "400", "500", "600", "700", "800", "900"];
        class a3 extends r4.Disposable {
          constructor(e6) {
            super(), this._onOptionChange = this.register(new s5.EventEmitter()), this.onOptionChange = this._onOptionChange.event;
            const i7 = { ...t4.DEFAULT_OPTIONS };
            for (const t5 in e6) if (t5 in i7) try {
              const s6 = e6[t5];
              i7[t5] = this._sanitizeAndValidateOption(t5, s6);
            } catch (e7) {
              console.error(e7);
            }
            this.rawOptions = i7, this.options = { ...i7 }, this._setupOptions(), this.register((0, r4.toDisposable)((() => {
              this.rawOptions.linkHandler = null, this.rawOptions.documentOverride = null;
            })));
          }
          onSpecificOptionChange(e6, t5) {
            return this.onOptionChange(((i7) => {
              i7 === e6 && t5(this.rawOptions[e6]);
            }));
          }
          onMultipleOptionChange(e6, t5) {
            return this.onOptionChange(((i7) => {
              -1 !== e6.indexOf(i7) && t5();
            }));
          }
          _setupOptions() {
            const e6 = (e7) => {
              if (!(e7 in t4.DEFAULT_OPTIONS)) throw new Error(`No option with key "${e7}"`);
              return this.rawOptions[e7];
            }, i7 = (e7, i8) => {
              if (!(e7 in t4.DEFAULT_OPTIONS)) throw new Error(`No option with key "${e7}"`);
              i8 = this._sanitizeAndValidateOption(e7, i8), this.rawOptions[e7] !== i8 && (this.rawOptions[e7] = i8, this._onOptionChange.fire(e7));
            };
            for (const t5 in this.rawOptions) {
              const s6 = { get: e6.bind(this, t5), set: i7.bind(this, t5) };
              Object.defineProperty(this.options, t5, s6);
            }
          }
          _sanitizeAndValidateOption(e6, i7) {
            switch (e6) {
              case "cursorStyle":
                if (i7 || (i7 = t4.DEFAULT_OPTIONS[e6]), !/* @__PURE__ */ (function(e7) {
                  return "block" === e7 || "underline" === e7 || "bar" === e7;
                })(i7)) throw new Error(`"${i7}" is not a valid value for ${e6}`);
                break;
              case "wordSeparator":
                i7 || (i7 = t4.DEFAULT_OPTIONS[e6]);
                break;
              case "fontWeight":
              case "fontWeightBold":
                if ("number" == typeof i7 && 1 <= i7 && i7 <= 1e3) break;
                i7 = o5.includes(i7) ? i7 : t4.DEFAULT_OPTIONS[e6];
                break;
              case "cursorWidth":
                i7 = Math.floor(i7);
              case "lineHeight":
              case "tabStopWidth":
                if (i7 < 1) throw new Error(`${e6} cannot be less than 1, value: ${i7}`);
                break;
              case "minimumContrastRatio":
                i7 = Math.max(1, Math.min(21, Math.round(10 * i7) / 10));
                break;
              case "scrollback":
                if ((i7 = Math.min(i7, 4294967295)) < 0) throw new Error(`${e6} cannot be less than 0, value: ${i7}`);
                break;
              case "fastScrollSensitivity":
              case "scrollSensitivity":
                if (i7 <= 0) throw new Error(`${e6} cannot be less than or equal to 0, value: ${i7}`);
                break;
              case "rows":
              case "cols":
                if (!i7 && 0 !== i7) throw new Error(`${e6} must be numeric, value: ${i7}`);
                break;
              case "windowsPty":
                i7 = i7 ?? {};
            }
            return i7;
          }
        }
        t4.OptionsService = a3;
      }, 2660: function(e5, t4, i6) {
        var s5 = this && this.__decorate || function(e6, t5, i7, s6) {
          var r5, n5 = arguments.length, o6 = n5 < 3 ? t5 : null === s6 ? s6 = Object.getOwnPropertyDescriptor(t5, i7) : s6;
          if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) o6 = Reflect.decorate(e6, t5, i7, s6);
          else for (var a3 = e6.length - 1; a3 >= 0; a3--) (r5 = e6[a3]) && (o6 = (n5 < 3 ? r5(o6) : n5 > 3 ? r5(t5, i7, o6) : r5(t5, i7)) || o6);
          return n5 > 3 && o6 && Object.defineProperty(t5, i7, o6), o6;
        }, r4 = this && this.__param || function(e6, t5) {
          return function(i7, s6) {
            t5(i7, s6, e6);
          };
        };
        Object.defineProperty(t4, "__esModule", { value: true }), t4.OscLinkService = void 0;
        const n4 = i6(2585);
        let o5 = t4.OscLinkService = class {
          constructor(e6) {
            this._bufferService = e6, this._nextId = 1, this._entriesWithId = /* @__PURE__ */ new Map(), this._dataByLinkId = /* @__PURE__ */ new Map();
          }
          registerLink(e6) {
            const t5 = this._bufferService.buffer;
            if (void 0 === e6.id) {
              const i8 = t5.addMarker(t5.ybase + t5.y), s7 = { data: e6, id: this._nextId++, lines: [i8] };
              return i8.onDispose((() => this._removeMarkerFromLink(s7, i8))), this._dataByLinkId.set(s7.id, s7), s7.id;
            }
            const i7 = e6, s6 = this._getEntryIdKey(i7), r5 = this._entriesWithId.get(s6);
            if (r5) return this.addLineToLink(r5.id, t5.ybase + t5.y), r5.id;
            const n5 = t5.addMarker(t5.ybase + t5.y), o6 = { id: this._nextId++, key: this._getEntryIdKey(i7), data: i7, lines: [n5] };
            return n5.onDispose((() => this._removeMarkerFromLink(o6, n5))), this._entriesWithId.set(o6.key, o6), this._dataByLinkId.set(o6.id, o6), o6.id;
          }
          addLineToLink(e6, t5) {
            const i7 = this._dataByLinkId.get(e6);
            if (i7 && i7.lines.every(((e7) => e7.line !== t5))) {
              const e7 = this._bufferService.buffer.addMarker(t5);
              i7.lines.push(e7), e7.onDispose((() => this._removeMarkerFromLink(i7, e7)));
            }
          }
          getLinkData(e6) {
            return this._dataByLinkId.get(e6)?.data;
          }
          _getEntryIdKey(e6) {
            return `${e6.id};;${e6.uri}`;
          }
          _removeMarkerFromLink(e6, t5) {
            const i7 = e6.lines.indexOf(t5);
            -1 !== i7 && (e6.lines.splice(i7, 1), 0 === e6.lines.length && (void 0 !== e6.data.id && this._entriesWithId.delete(e6.key), this._dataByLinkId.delete(e6.id)));
          }
        };
        t4.OscLinkService = o5 = s5([r4(0, n4.IBufferService)], o5);
      }, 8343: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.createDecorator = t4.getServiceDependencies = t4.serviceRegistry = void 0;
        const i6 = "di$target", s5 = "di$dependencies";
        t4.serviceRegistry = /* @__PURE__ */ new Map(), t4.getServiceDependencies = function(e6) {
          return e6[s5] || [];
        }, t4.createDecorator = function(e6) {
          if (t4.serviceRegistry.has(e6)) return t4.serviceRegistry.get(e6);
          const r4 = function(e7, t5, n4) {
            if (3 !== arguments.length) throw new Error("@IServiceName-decorator can only be used to decorate a parameter");
            !(function(e8, t6, r5) {
              t6[i6] === t6 ? t6[s5].push({ id: e8, index: r5 }) : (t6[s5] = [{ id: e8, index: r5 }], t6[i6] = t6);
            })(r4, e7, n4);
          };
          return r4.toString = () => e6, t4.serviceRegistry.set(e6, r4), r4;
        };
      }, 2585: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.IDecorationService = t4.IUnicodeService = t4.IOscLinkService = t4.IOptionsService = t4.ILogService = t4.LogLevelEnum = t4.IInstantiationService = t4.ICharsetService = t4.ICoreService = t4.ICoreMouseService = t4.IBufferService = void 0;
        const s5 = i6(8343);
        var r4;
        t4.IBufferService = (0, s5.createDecorator)("BufferService"), t4.ICoreMouseService = (0, s5.createDecorator)("CoreMouseService"), t4.ICoreService = (0, s5.createDecorator)("CoreService"), t4.ICharsetService = (0, s5.createDecorator)("CharsetService"), t4.IInstantiationService = (0, s5.createDecorator)("InstantiationService"), (function(e6) {
          e6[e6.TRACE = 0] = "TRACE", e6[e6.DEBUG = 1] = "DEBUG", e6[e6.INFO = 2] = "INFO", e6[e6.WARN = 3] = "WARN", e6[e6.ERROR = 4] = "ERROR", e6[e6.OFF = 5] = "OFF";
        })(r4 || (t4.LogLevelEnum = r4 = {})), t4.ILogService = (0, s5.createDecorator)("LogService"), t4.IOptionsService = (0, s5.createDecorator)("OptionsService"), t4.IOscLinkService = (0, s5.createDecorator)("OscLinkService"), t4.IUnicodeService = (0, s5.createDecorator)("UnicodeService"), t4.IDecorationService = (0, s5.createDecorator)("DecorationService");
      }, 1480: (e5, t4, i6) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.UnicodeService = void 0;
        const s5 = i6(8460), r4 = i6(225);
        class n4 {
          static extractShouldJoin(e6) {
            return 0 != (1 & e6);
          }
          static extractWidth(e6) {
            return e6 >> 1 & 3;
          }
          static extractCharKind(e6) {
            return e6 >> 3;
          }
          static createPropertyValue(e6, t5, i7 = false) {
            return (16777215 & e6) << 3 | (3 & t5) << 1 | (i7 ? 1 : 0);
          }
          constructor() {
            this._providers = /* @__PURE__ */ Object.create(null), this._active = "", this._onChange = new s5.EventEmitter(), this.onChange = this._onChange.event;
            const e6 = new r4.UnicodeV6();
            this.register(e6), this._active = e6.version, this._activeProvider = e6;
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
          set activeVersion(e6) {
            if (!this._providers[e6]) throw new Error(`unknown Unicode version "${e6}"`);
            this._active = e6, this._activeProvider = this._providers[e6], this._onChange.fire(e6);
          }
          register(e6) {
            this._providers[e6.version] = e6;
          }
          wcwidth(e6) {
            return this._activeProvider.wcwidth(e6);
          }
          getStringCellWidth(e6) {
            let t5 = 0, i7 = 0;
            const s6 = e6.length;
            for (let r5 = 0; r5 < s6; ++r5) {
              let o5 = e6.charCodeAt(r5);
              if (55296 <= o5 && o5 <= 56319) {
                if (++r5 >= s6) return t5 + this.wcwidth(o5);
                const i8 = e6.charCodeAt(r5);
                56320 <= i8 && i8 <= 57343 ? o5 = 1024 * (o5 - 55296) + i8 - 56320 + 65536 : t5 += this.wcwidth(i8);
              }
              const a3 = this.charProperties(o5, i7);
              let h3 = n4.extractWidth(a3);
              n4.extractShouldJoin(a3) && (h3 -= n4.extractWidth(i7)), t5 += h3, i7 = a3;
            }
            return t5;
          }
          charProperties(e6, t5) {
            return this._activeProvider.charProperties(e6, t5);
          }
        }
        t4.UnicodeService = n4;
      } }, t3 = {};
      function i5(s5) {
        var r4 = t3[s5];
        if (void 0 !== r4) return r4.exports;
        var n4 = t3[s5] = { exports: {} };
        return e4[s5].call(n4.exports, n4, n4.exports, i5), n4.exports;
      }
      var s4 = {};
      return (() => {
        var e5 = s4;
        Object.defineProperty(e5, "__esModule", { value: true }), e5.Terminal = void 0;
        const t4 = i5(9042), r4 = i5(3236), n4 = i5(844), o5 = i5(5741), a3 = i5(8285), h3 = i5(7975), c4 = i5(7090), l3 = ["cols", "rows"];
        class d3 extends n4.Disposable {
          constructor(e6) {
            super(), this._core = this.register(new r4.Terminal(e6)), this._addonManager = this.register(new o5.AddonManager()), this._publicOptions = { ...this._core.options };
            const t5 = (e7) => this._core.options[e7], i6 = (e7, t6) => {
              this._checkReadonlyOptions(e7), this._core.options[e7] = t6;
            };
            for (const e7 in this._core.options) {
              const s5 = { get: t5.bind(this, e7), set: i6.bind(this, e7) };
              Object.defineProperty(this._publicOptions, e7, s5);
            }
          }
          _checkReadonlyOptions(e6) {
            if (l3.includes(e6)) throw new Error(`Option "${e6}" can only be set in the constructor`);
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
            return this._parser || (this._parser = new h3.ParserApi(this._core)), this._parser;
          }
          get unicode() {
            return this._checkProposedApi(), new c4.UnicodeApi(this._core);
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
            return this._buffer || (this._buffer = this.register(new a3.BufferNamespaceApi(this._core))), this._buffer;
          }
          get markers() {
            return this._checkProposedApi(), this._core.markers;
          }
          get modes() {
            const e6 = this._core.coreService.decPrivateModes;
            let t5 = "none";
            switch (this._core.coreMouseService.activeProtocol) {
              case "X10":
                t5 = "x10";
                break;
              case "VT200":
                t5 = "vt200";
                break;
              case "DRAG":
                t5 = "drag";
                break;
              case "ANY":
                t5 = "any";
            }
            return { applicationCursorKeysMode: e6.applicationCursorKeys, applicationKeypadMode: e6.applicationKeypad, bracketedPasteMode: e6.bracketedPasteMode, insertMode: this._core.coreService.modes.insertMode, mouseTrackingMode: t5, originMode: e6.origin, reverseWraparoundMode: e6.reverseWraparound, sendFocusMode: e6.sendFocus, wraparoundMode: e6.wraparound };
          }
          get options() {
            return this._publicOptions;
          }
          set options(e6) {
            for (const t5 in e6) this._publicOptions[t5] = e6[t5];
          }
          blur() {
            this._core.blur();
          }
          focus() {
            this._core.focus();
          }
          input(e6, t5 = true) {
            this._core.input(e6, t5);
          }
          resize(e6, t5) {
            this._verifyIntegers(e6, t5), this._core.resize(e6, t5);
          }
          open(e6) {
            this._core.open(e6);
          }
          attachCustomKeyEventHandler(e6) {
            this._core.attachCustomKeyEventHandler(e6);
          }
          attachCustomWheelEventHandler(e6) {
            this._core.attachCustomWheelEventHandler(e6);
          }
          registerLinkProvider(e6) {
            return this._core.registerLinkProvider(e6);
          }
          registerCharacterJoiner(e6) {
            return this._checkProposedApi(), this._core.registerCharacterJoiner(e6);
          }
          deregisterCharacterJoiner(e6) {
            this._checkProposedApi(), this._core.deregisterCharacterJoiner(e6);
          }
          registerMarker(e6 = 0) {
            return this._verifyIntegers(e6), this._core.registerMarker(e6);
          }
          registerDecoration(e6) {
            return this._checkProposedApi(), this._verifyPositiveIntegers(e6.x ?? 0, e6.width ?? 0, e6.height ?? 0), this._core.registerDecoration(e6);
          }
          hasSelection() {
            return this._core.hasSelection();
          }
          select(e6, t5, i6) {
            this._verifyIntegers(e6, t5, i6), this._core.select(e6, t5, i6);
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
          selectLines(e6, t5) {
            this._verifyIntegers(e6, t5), this._core.selectLines(e6, t5);
          }
          dispose() {
            super.dispose();
          }
          scrollLines(e6) {
            this._verifyIntegers(e6), this._core.scrollLines(e6);
          }
          scrollPages(e6) {
            this._verifyIntegers(e6), this._core.scrollPages(e6);
          }
          scrollToTop() {
            this._core.scrollToTop();
          }
          scrollToBottom() {
            this._core.scrollToBottom();
          }
          scrollToLine(e6) {
            this._verifyIntegers(e6), this._core.scrollToLine(e6);
          }
          clear() {
            this._core.clear();
          }
          write(e6, t5) {
            this._core.write(e6, t5);
          }
          writeln(e6, t5) {
            this._core.write(e6), this._core.write("\r\n", t5);
          }
          paste(e6) {
            this._core.paste(e6);
          }
          refresh(e6, t5) {
            this._verifyIntegers(e6, t5), this._core.refresh(e6, t5);
          }
          reset() {
            this._core.reset();
          }
          clearTextureAtlas() {
            this._core.clearTextureAtlas();
          }
          loadAddon(e6) {
            this._addonManager.loadAddon(this, e6);
          }
          static get strings() {
            return t4;
          }
          _verifyIntegers(...e6) {
            for (const t5 of e6) if (t5 === 1 / 0 || isNaN(t5) || t5 % 1 != 0) throw new Error("This API only accepts integers");
          }
          _verifyPositiveIntegers(...e6) {
            for (const t5 of e6) if (t5 && (t5 === 1 / 0 || isNaN(t5) || t5 % 1 != 0 || t5 < 0)) throw new Error("This API only accepts positive integers");
          }
        }
        e5.Terminal = d3;
      })(), s4;
    })()));
  }
});

// ../../node_modules/.pnpm/@xterm+addon-fit@0.10.0_@xterm+xterm@5.5.0/node_modules/@xterm/addon-fit/lib/addon-fit.js
var require_addon_fit = __commonJS({
  "../../node_modules/.pnpm/@xterm+addon-fit@0.10.0_@xterm+xterm@5.5.0/node_modules/@xterm/addon-fit/lib/addon-fit.js"(exports, module) {
    !(function(e4, t3) {
      "object" == typeof exports && "object" == typeof module ? module.exports = t3() : "function" == typeof define && define.amd ? define([], t3) : "object" == typeof exports ? exports.FitAddon = t3() : e4.FitAddon = t3();
    })(self, (() => (() => {
      "use strict";
      var e4 = {};
      return (() => {
        var t3 = e4;
        Object.defineProperty(t3, "__esModule", { value: true }), t3.FitAddon = void 0, t3.FitAddon = class {
          activate(e5) {
            this._terminal = e5;
          }
          dispose() {
          }
          fit() {
            const e5 = this.proposeDimensions();
            if (!e5 || !this._terminal || isNaN(e5.cols) || isNaN(e5.rows)) return;
            const t4 = this._terminal._core;
            this._terminal.rows === e5.rows && this._terminal.cols === e5.cols || (t4._renderService.clear(), this._terminal.resize(e5.cols, e5.rows));
          }
          proposeDimensions() {
            if (!this._terminal) return;
            if (!this._terminal.element || !this._terminal.element.parentElement) return;
            const e5 = this._terminal._core, t4 = e5._renderService.dimensions;
            if (0 === t4.css.cell.width || 0 === t4.css.cell.height) return;
            const r4 = 0 === this._terminal.options.scrollback ? 0 : e5.viewport.scrollBarWidth, i5 = window.getComputedStyle(this._terminal.element.parentElement), o5 = parseInt(i5.getPropertyValue("height")), s4 = Math.max(0, parseInt(i5.getPropertyValue("width"))), n4 = window.getComputedStyle(this._terminal.element), l3 = o5 - (parseInt(n4.getPropertyValue("padding-top")) + parseInt(n4.getPropertyValue("padding-bottom"))), a3 = s4 - (parseInt(n4.getPropertyValue("padding-right")) + parseInt(n4.getPropertyValue("padding-left"))) - r4;
            return { cols: Math.max(2, Math.floor(a3 / t4.css.cell.width)), rows: Math.max(1, Math.floor(l3 / t4.css.cell.height)) };
          }
        };
      })(), e4;
    })()));
  }
});

// ../../node_modules/.pnpm/@xterm+addon-web-links@0.11.0_@xterm+xterm@5.5.0/node_modules/@xterm/addon-web-links/lib/addon-web-links.js
var require_addon_web_links = __commonJS({
  "../../node_modules/.pnpm/@xterm+addon-web-links@0.11.0_@xterm+xterm@5.5.0/node_modules/@xterm/addon-web-links/lib/addon-web-links.js"(exports, module) {
    !(function(e4, t3) {
      "object" == typeof exports && "object" == typeof module ? module.exports = t3() : "function" == typeof define && define.amd ? define([], t3) : "object" == typeof exports ? exports.WebLinksAddon = t3() : e4.WebLinksAddon = t3();
    })(self, (() => (() => {
      "use strict";
      var e4 = { 6: (e5, t4) => {
        function n5(e6) {
          try {
            const t5 = new URL(e6), n6 = t5.password && t5.username ? `${t5.protocol}//${t5.username}:${t5.password}@${t5.host}` : t5.username ? `${t5.protocol}//${t5.username}@${t5.host}` : `${t5.protocol}//${t5.host}`;
            return e6.toLocaleLowerCase().startsWith(n6.toLocaleLowerCase());
          } catch (e7) {
            return false;
          }
        }
        Object.defineProperty(t4, "__esModule", { value: true }), t4.LinkComputer = t4.WebLinkProvider = void 0, t4.WebLinkProvider = class {
          constructor(e6, t5, n6, o7 = {}) {
            this._terminal = e6, this._regex = t5, this._handler = n6, this._options = o7;
          }
          provideLinks(e6, t5) {
            const n6 = o6.computeLink(e6, this._regex, this._terminal, this._handler);
            t5(this._addCallbacks(n6));
          }
          _addCallbacks(e6) {
            return e6.map(((e7) => (e7.leave = this._options.leave, e7.hover = (t5, n6) => {
              if (this._options.hover) {
                const { range: o7 } = e7;
                this._options.hover(t5, n6, o7);
              }
            }, e7)));
          }
        };
        class o6 {
          static computeLink(e6, t5, r4, i5) {
            const s4 = new RegExp(t5.source, (t5.flags || "") + "g"), [a3, c4] = o6._getWindowedLineStrings(e6 - 1, r4), l3 = a3.join("");
            let d3;
            const p3 = [];
            for (; d3 = s4.exec(l3); ) {
              const e7 = d3[0];
              if (!n5(e7)) continue;
              const [t6, s5] = o6._mapStrIdx(r4, c4, 0, d3.index), [a4, l4] = o6._mapStrIdx(r4, t6, s5, e7.length);
              if (-1 === t6 || -1 === s5 || -1 === a4 || -1 === l4) continue;
              const h3 = { start: { x: s5 + 1, y: t6 + 1 }, end: { x: l4, y: a4 + 1 } };
              p3.push({ range: h3, text: e7, activate: i5 });
            }
            return p3;
          }
          static _getWindowedLineStrings(e6, t5) {
            let n6, o7 = e6, r4 = e6, i5 = 0, s4 = "";
            const a3 = [];
            if (n6 = t5.buffer.active.getLine(e6)) {
              const e7 = n6.translateToString(true);
              if (n6.isWrapped && " " !== e7[0]) {
                for (i5 = 0; (n6 = t5.buffer.active.getLine(--o7)) && i5 < 2048 && (s4 = n6.translateToString(true), i5 += s4.length, a3.push(s4), n6.isWrapped && -1 === s4.indexOf(" ")); ) ;
                a3.reverse();
              }
              for (a3.push(e7), i5 = 0; (n6 = t5.buffer.active.getLine(++r4)) && n6.isWrapped && i5 < 2048 && (s4 = n6.translateToString(true), i5 += s4.length, a3.push(s4), -1 === s4.indexOf(" ")); ) ;
            }
            return [a3, o7];
          }
          static _mapStrIdx(e6, t5, n6, o7) {
            const r4 = e6.buffer.active, i5 = r4.getNullCell();
            let s4 = n6;
            for (; o7; ) {
              const e7 = r4.getLine(t5);
              if (!e7) return [-1, -1];
              for (let n7 = s4; n7 < e7.length; ++n7) {
                e7.getCell(n7, i5);
                const s5 = i5.getChars();
                if (i5.getWidth() && (o7 -= s5.length || 1, n7 === e7.length - 1 && "" === s5)) {
                  const e8 = r4.getLine(t5 + 1);
                  e8 && e8.isWrapped && (e8.getCell(0, i5), 2 === i5.getWidth() && (o7 += 1));
                }
                if (o7 < 0) return [t5, n7];
              }
              t5++, s4 = 0;
            }
            return [t5, s4];
          }
        }
        t4.LinkComputer = o6;
      } }, t3 = {};
      function n4(o6) {
        var r4 = t3[o6];
        if (void 0 !== r4) return r4.exports;
        var i5 = t3[o6] = { exports: {} };
        return e4[o6](i5, i5.exports, n4), i5.exports;
      }
      var o5 = {};
      return (() => {
        var e5 = o5;
        Object.defineProperty(e5, "__esModule", { value: true }), e5.WebLinksAddon = void 0;
        const t4 = n4(6), r4 = /(https?|HTTPS?):[/]{2}[^\s"'!*(){}|\\\^<>`]*[^\s"':,.!?{}|\\\^~\[\]`()<>]/;
        function i5(e6, t5) {
          const n5 = window.open();
          if (n5) {
            try {
              n5.opener = null;
            } catch {
            }
            n5.location.href = t5;
          } else console.warn("Opening link blocked as opener could not be cleared");
        }
        e5.WebLinksAddon = class {
          constructor(e6 = i5, t5 = {}) {
            this._handler = e6, this._options = t5;
          }
          activate(e6) {
            this._terminal = e6;
            const n5 = this._options, o6 = n5.urlRegex || r4;
            this._linkProvider = this._terminal.registerLinkProvider(new t4.WebLinkProvider(this._terminal, o6, this._handler, n5));
          }
          dispose() {
            this._linkProvider?.dispose();
          }
        };
      })(), o5;
    })()));
  }
});

// ../../node_modules/.pnpm/@xterm+addon-search@0.15.0_@xterm+xterm@5.5.0/node_modules/@xterm/addon-search/lib/addon-search.js
var require_addon_search = __commonJS({
  "../../node_modules/.pnpm/@xterm+addon-search@0.15.0_@xterm+xterm@5.5.0/node_modules/@xterm/addon-search/lib/addon-search.js"(exports, module) {
    !(function(e4, t3) {
      "object" == typeof exports && "object" == typeof module ? module.exports = t3() : "function" == typeof define && define.amd ? define([], t3) : "object" == typeof exports ? exports.SearchAddon = t3() : e4.SearchAddon = t3();
    })(self, (() => (() => {
      "use strict";
      var e4 = { 345: (e5, t4) => {
        Object.defineProperty(t4, "__esModule", { value: true }), t4.runAndSubscribe = t4.forwardEvent = t4.EventEmitter = void 0, t4.EventEmitter = class {
          constructor() {
            this._listeners = [], this._disposed = false;
          }
          get event() {
            return this._event || (this._event = (e6) => (this._listeners.push(e6), { dispose: () => {
              if (!this._disposed) {
                for (let t5 = 0; t5 < this._listeners.length; t5++) if (this._listeners[t5] === e6) return void this._listeners.splice(t5, 1);
              }
            } })), this._event;
          }
          fire(e6, t5) {
            const i6 = [];
            for (let e7 = 0; e7 < this._listeners.length; e7++) i6.push(this._listeners[e7]);
            for (let s5 = 0; s5 < i6.length; s5++) i6[s5].call(void 0, e6, t5);
          }
          dispose() {
            this.clearListeners(), this._disposed = true;
          }
          clearListeners() {
            this._listeners && (this._listeners.length = 0);
          }
        }, t4.forwardEvent = function(e6, t5) {
          return e6(((e7) => t5.fire(e7)));
        }, t4.runAndSubscribe = function(e6, t5) {
          return t5(void 0), e6(((e7) => t5(e7)));
        };
      }, 859: (e5, t4) => {
        function i6(e6) {
          for (const t5 of e6) t5.dispose();
          e6.length = 0;
        }
        Object.defineProperty(t4, "__esModule", { value: true }), t4.getDisposeArrayDisposable = t4.disposeArray = t4.toDisposable = t4.MutableDisposable = t4.Disposable = void 0, t4.Disposable = class {
          constructor() {
            this._disposables = [], this._isDisposed = false;
          }
          dispose() {
            this._isDisposed = true;
            for (const e6 of this._disposables) e6.dispose();
            this._disposables.length = 0;
          }
          register(e6) {
            return this._disposables.push(e6), e6;
          }
          unregister(e6) {
            const t5 = this._disposables.indexOf(e6);
            -1 !== t5 && this._disposables.splice(t5, 1);
          }
        }, t4.MutableDisposable = class {
          constructor() {
            this._isDisposed = false;
          }
          get value() {
            return this._isDisposed ? void 0 : this._value;
          }
          set value(e6) {
            this._isDisposed || e6 === this._value || (this._value?.dispose(), this._value = e6);
          }
          clear() {
            this.value = void 0;
          }
          dispose() {
            this._isDisposed = true, this._value?.dispose(), this._value = void 0;
          }
        }, t4.toDisposable = function(e6) {
          return { dispose: e6 };
        }, t4.disposeArray = i6, t4.getDisposeArrayDisposable = function(e6) {
          return { dispose: () => i6(e6) };
        };
      } }, t3 = {};
      function i5(s5) {
        var r4 = t3[s5];
        if (void 0 !== r4) return r4.exports;
        var o5 = t3[s5] = { exports: {} };
        return e4[s5](o5, o5.exports, i5), o5.exports;
      }
      var s4 = {};
      return (() => {
        var e5 = s4;
        Object.defineProperty(e5, "__esModule", { value: true }), e5.SearchAddon = void 0;
        const t4 = i5(345), r4 = i5(859), o5 = " ~!@#$%^&*()+`-=[]{}|\\;:\"',./<>?";
        class n4 extends r4.Disposable {
          constructor(e6) {
            super(), this._highlightedLines = /* @__PURE__ */ new Set(), this._highlightDecorations = [], this._selectedDecoration = this.register(new r4.MutableDisposable()), this._linesCacheTimeoutId = 0, this._linesCacheDisposables = new r4.MutableDisposable(), this._onDidChangeResults = this.register(new t4.EventEmitter()), this.onDidChangeResults = this._onDidChangeResults.event, this._highlightLimit = e6?.highlightLimit ?? 1e3;
          }
          activate(e6) {
            this._terminal = e6, this.register(this._terminal.onWriteParsed((() => this._updateMatches()))), this.register(this._terminal.onResize((() => this._updateMatches()))), this.register((0, r4.toDisposable)((() => this.clearDecorations())));
          }
          _updateMatches() {
            this._highlightTimeout && window.clearTimeout(this._highlightTimeout), this._cachedSearchTerm && this._lastSearchOptions?.decorations && (this._highlightTimeout = setTimeout((() => {
              const e6 = this._cachedSearchTerm;
              this._cachedSearchTerm = void 0, this.findPrevious(e6, { ...this._lastSearchOptions, incremental: true, noScroll: true });
            }), 200));
          }
          clearDecorations(e6) {
            this._selectedDecoration.clear(), (0, r4.disposeArray)(this._highlightDecorations), this._highlightDecorations = [], this._highlightedLines.clear(), e6 || (this._cachedSearchTerm = void 0);
          }
          clearActiveDecoration() {
            this._selectedDecoration.clear();
          }
          findNext(e6, t5) {
            if (!this._terminal) throw new Error("Cannot use addon until it has been loaded");
            const i6 = !this._lastSearchOptions || this._didOptionsChange(this._lastSearchOptions, t5);
            this._lastSearchOptions = t5, t5?.decorations && (void 0 === this._cachedSearchTerm || e6 !== this._cachedSearchTerm || i6) && this._highlightAllMatches(e6, t5);
            const s5 = this._findNextAndSelect(e6, t5);
            return this._fireResults(t5), this._cachedSearchTerm = e6, s5;
          }
          _highlightAllMatches(e6, t5) {
            if (!this._terminal) throw new Error("Cannot use addon until it has been loaded");
            if (!e6 || 0 === e6.length) return void this.clearDecorations();
            t5 = t5 || {}, this.clearDecorations(true);
            const i6 = [];
            let s5, r5 = this._find(e6, 0, 0, t5);
            for (; r5 && (s5?.row !== r5.row || s5?.col !== r5.col) && !(i6.length >= this._highlightLimit); ) s5 = r5, i6.push(s5), r5 = this._find(e6, s5.col + s5.term.length >= this._terminal.cols ? s5.row + 1 : s5.row, s5.col + s5.term.length >= this._terminal.cols ? 0 : s5.col + 1, t5);
            for (const e7 of i6) {
              const i7 = this._createResultDecoration(e7, t5.decorations);
              i7 && (this._highlightedLines.add(i7.marker.line), this._highlightDecorations.push({ decoration: i7, match: e7, dispose() {
                i7.dispose();
              } }));
            }
          }
          _find(e6, t5, i6, s5) {
            if (!this._terminal || !e6 || 0 === e6.length) return this._terminal?.clearSelection(), void this.clearDecorations();
            if (i6 > this._terminal.cols) throw new Error(`Invalid col: ${i6} to search in terminal of ${this._terminal.cols} cols`);
            let r5;
            this._initLinesCache();
            const o6 = { startRow: t5, startCol: i6 };
            if (r5 = this._findInLine(e6, o6, s5), !r5) for (let i7 = t5 + 1; i7 < this._terminal.buffer.active.baseY + this._terminal.rows && (o6.startRow = i7, o6.startCol = 0, r5 = this._findInLine(e6, o6, s5), !r5); i7++) ;
            return r5;
          }
          _findNextAndSelect(e6, t5) {
            if (!this._terminal || !e6 || 0 === e6.length) return this._terminal?.clearSelection(), this.clearDecorations(), false;
            const i6 = this._terminal.getSelectionPosition();
            this._terminal.clearSelection();
            let s5 = 0, r5 = 0;
            i6 && (this._cachedSearchTerm === e6 ? (s5 = i6.end.x, r5 = i6.end.y) : (s5 = i6.start.x, r5 = i6.start.y)), this._initLinesCache();
            const o6 = { startRow: r5, startCol: s5 };
            let n5 = this._findInLine(e6, o6, t5);
            if (!n5) for (let i7 = r5 + 1; i7 < this._terminal.buffer.active.baseY + this._terminal.rows && (o6.startRow = i7, o6.startCol = 0, n5 = this._findInLine(e6, o6, t5), !n5); i7++) ;
            if (!n5 && 0 !== r5) for (let i7 = 0; i7 < r5 && (o6.startRow = i7, o6.startCol = 0, n5 = this._findInLine(e6, o6, t5), !n5); i7++) ;
            return !n5 && i6 && (o6.startRow = i6.start.y, o6.startCol = 0, n5 = this._findInLine(e6, o6, t5)), this._selectResult(n5, t5?.decorations, t5?.noScroll);
          }
          findPrevious(e6, t5) {
            if (!this._terminal) throw new Error("Cannot use addon until it has been loaded");
            const i6 = !this._lastSearchOptions || this._didOptionsChange(this._lastSearchOptions, t5);
            this._lastSearchOptions = t5, t5?.decorations && (void 0 === this._cachedSearchTerm || e6 !== this._cachedSearchTerm || i6) && this._highlightAllMatches(e6, t5);
            const s5 = this._findPreviousAndSelect(e6, t5);
            return this._fireResults(t5), this._cachedSearchTerm = e6, s5;
          }
          _didOptionsChange(e6, t5) {
            return !!t5 && (e6.caseSensitive !== t5.caseSensitive || e6.regex !== t5.regex || e6.wholeWord !== t5.wholeWord);
          }
          _fireResults(e6) {
            if (e6?.decorations) {
              let e7 = -1;
              if (this._selectedDecoration.value) {
                const t5 = this._selectedDecoration.value.match;
                for (let i6 = 0; i6 < this._highlightDecorations.length; i6++) {
                  const s5 = this._highlightDecorations[i6].match;
                  if (s5.row === t5.row && s5.col === t5.col && s5.size === t5.size) {
                    e7 = i6;
                    break;
                  }
                }
              }
              this._onDidChangeResults.fire({ resultIndex: e7, resultCount: this._highlightDecorations.length });
            }
          }
          _findPreviousAndSelect(e6, t5) {
            if (!this._terminal) throw new Error("Cannot use addon until it has been loaded");
            if (!this._terminal || !e6 || 0 === e6.length) return this._terminal?.clearSelection(), this.clearDecorations(), false;
            const i6 = this._terminal.getSelectionPosition();
            this._terminal.clearSelection();
            let s5 = this._terminal.buffer.active.baseY + this._terminal.rows - 1, r5 = this._terminal.cols;
            const o6 = true;
            this._initLinesCache();
            const n5 = { startRow: s5, startCol: r5 };
            let h3;
            if (i6 && (n5.startRow = s5 = i6.start.y, n5.startCol = r5 = i6.start.x, this._cachedSearchTerm !== e6 && (h3 = this._findInLine(e6, n5, t5, false), h3 || (n5.startRow = s5 = i6.end.y, n5.startCol = r5 = i6.end.x))), h3 || (h3 = this._findInLine(e6, n5, t5, o6)), !h3) {
              n5.startCol = Math.max(n5.startCol, this._terminal.cols);
              for (let i7 = s5 - 1; i7 >= 0 && (n5.startRow = i7, h3 = this._findInLine(e6, n5, t5, o6), !h3); i7--) ;
            }
            if (!h3 && s5 !== this._terminal.buffer.active.baseY + this._terminal.rows - 1) for (let i7 = this._terminal.buffer.active.baseY + this._terminal.rows - 1; i7 >= s5 && (n5.startRow = i7, h3 = this._findInLine(e6, n5, t5, o6), !h3); i7--) ;
            return this._selectResult(h3, t5?.decorations, t5?.noScroll);
          }
          _initLinesCache() {
            const e6 = this._terminal;
            this._linesCache || (this._linesCache = new Array(e6.buffer.active.length), this._linesCacheDisposables.value = (0, r4.getDisposeArrayDisposable)([e6.onLineFeed((() => this._destroyLinesCache())), e6.onCursorMove((() => this._destroyLinesCache())), e6.onResize((() => this._destroyLinesCache()))])), window.clearTimeout(this._linesCacheTimeoutId), this._linesCacheTimeoutId = window.setTimeout((() => this._destroyLinesCache()), 15e3);
          }
          _destroyLinesCache() {
            this._linesCache = void 0, this._linesCacheDisposables.clear(), this._linesCacheTimeoutId && (window.clearTimeout(this._linesCacheTimeoutId), this._linesCacheTimeoutId = 0);
          }
          _isWholeWord(e6, t5, i6) {
            return (0 === e6 || o5.includes(t5[e6 - 1])) && (e6 + i6.length === t5.length || o5.includes(t5[e6 + i6.length]));
          }
          _findInLine(e6, t5, i6 = {}, s5 = false) {
            const r5 = this._terminal, o6 = t5.startRow, n5 = t5.startCol, h3 = r5.buffer.active.getLine(o6);
            if (h3?.isWrapped) return s5 ? void (t5.startCol += r5.cols) : (t5.startRow--, t5.startCol += r5.cols, this._findInLine(e6, t5, i6));
            let a3 = this._linesCache?.[o6];
            a3 || (a3 = this._translateBufferLineToStringWithWrap(o6, true), this._linesCache && (this._linesCache[o6] = a3));
            const [l3, c4] = a3, d3 = this._bufferColsToStringOffset(o6, n5), _2 = i6.caseSensitive ? e6 : e6.toLowerCase(), u3 = i6.caseSensitive ? l3 : l3.toLowerCase();
            let f3 = -1;
            if (i6.regex) {
              const t6 = RegExp(_2, "g");
              let i7;
              if (s5) for (; i7 = t6.exec(u3.slice(0, d3)); ) f3 = t6.lastIndex - i7[0].length, e6 = i7[0], t6.lastIndex -= e6.length - 1;
              else i7 = t6.exec(u3.slice(d3)), i7 && i7[0].length > 0 && (f3 = d3 + (t6.lastIndex - i7[0].length), e6 = i7[0]);
            } else s5 ? d3 - _2.length >= 0 && (f3 = u3.lastIndexOf(_2, d3 - _2.length)) : f3 = u3.indexOf(_2, d3);
            if (f3 >= 0) {
              if (i6.wholeWord && !this._isWholeWord(f3, u3, e6)) return;
              let t6 = 0;
              for (; t6 < c4.length - 1 && f3 >= c4[t6 + 1]; ) t6++;
              let s6 = t6;
              for (; s6 < c4.length - 1 && f3 + e6.length >= c4[s6 + 1]; ) s6++;
              const n6 = f3 - c4[t6], h4 = f3 + e6.length - c4[s6], a4 = this._stringLengthToBufferSize(o6 + t6, n6);
              return { term: e6, col: a4, row: o6 + t6, size: this._stringLengthToBufferSize(o6 + s6, h4) - a4 + r5.cols * (s6 - t6) };
            }
          }
          _stringLengthToBufferSize(e6, t5) {
            const i6 = this._terminal.buffer.active.getLine(e6);
            if (!i6) return 0;
            for (let e7 = 0; e7 < t5; e7++) {
              const s5 = i6.getCell(e7);
              if (!s5) break;
              const r5 = s5.getChars();
              r5.length > 1 && (t5 -= r5.length - 1);
              const o6 = i6.getCell(e7 + 1);
              o6 && 0 === o6.getWidth() && t5++;
            }
            return t5;
          }
          _bufferColsToStringOffset(e6, t5) {
            const i6 = this._terminal;
            let s5 = e6, r5 = 0, o6 = i6.buffer.active.getLine(s5);
            for (; t5 > 0 && o6; ) {
              for (let e7 = 0; e7 < t5 && e7 < i6.cols; e7++) {
                const t6 = o6.getCell(e7);
                if (!t6) break;
                t6.getWidth() && (r5 += 0 === t6.getCode() ? 1 : t6.getChars().length);
              }
              if (s5++, o6 = i6.buffer.active.getLine(s5), o6 && !o6.isWrapped) break;
              t5 -= i6.cols;
            }
            return r5;
          }
          _translateBufferLineToStringWithWrap(e6, t5) {
            const i6 = this._terminal, s5 = [], r5 = [0];
            let o6 = i6.buffer.active.getLine(e6);
            for (; o6; ) {
              const n5 = i6.buffer.active.getLine(e6 + 1), h3 = !!n5 && n5.isWrapped;
              let a3 = o6.translateToString(!h3 && t5);
              if (h3 && n5) {
                const e7 = o6.getCell(o6.length - 1);
                e7 && 0 === e7.getCode() && 1 === e7.getWidth() && 2 === n5.getCell(0)?.getWidth() && (a3 = a3.slice(0, -1));
              }
              if (s5.push(a3), !h3) break;
              r5.push(r5[r5.length - 1] + a3.length), e6++, o6 = n5;
            }
            return [s5.join(""), r5];
          }
          _selectResult(e6, t5, i6) {
            const s5 = this._terminal;
            if (this._selectedDecoration.clear(), !e6) return s5.clearSelection(), false;
            if (s5.select(e6.col, e6.row, e6.size), t5) {
              const i7 = s5.registerMarker(-s5.buffer.active.baseY - s5.buffer.active.cursorY + e6.row);
              if (i7) {
                const o6 = s5.registerDecoration({ marker: i7, x: e6.col, width: e6.size, backgroundColor: t5.activeMatchBackground, layer: "top", overviewRulerOptions: { color: t5.activeMatchColorOverviewRuler } });
                if (o6) {
                  const s6 = [];
                  s6.push(i7), s6.push(o6.onRender(((e7) => this._applyStyles(e7, t5.activeMatchBorder, true)))), s6.push(o6.onDispose((() => (0, r4.disposeArray)(s6)))), this._selectedDecoration.value = { decoration: o6, match: e6, dispose() {
                    o6.dispose();
                  } };
                }
              }
            }
            if (!i6 && (e6.row >= s5.buffer.active.viewportY + s5.rows || e6.row < s5.buffer.active.viewportY)) {
              let t6 = e6.row - s5.buffer.active.viewportY;
              t6 -= Math.floor(s5.rows / 2), s5.scrollLines(t6);
            }
            return true;
          }
          _applyStyles(e6, t5, i6) {
            e6.classList.contains("xterm-find-result-decoration") || (e6.classList.add("xterm-find-result-decoration"), t5 && (e6.style.outline = `1px solid ${t5}`)), i6 && e6.classList.add("xterm-find-active-result-decoration");
          }
          _createResultDecoration(e6, t5) {
            const i6 = this._terminal, s5 = i6.registerMarker(-i6.buffer.active.baseY - i6.buffer.active.cursorY + e6.row);
            if (!s5) return;
            const o6 = i6.registerDecoration({ marker: s5, x: e6.col, width: e6.size, backgroundColor: t5.matchBackground, overviewRulerOptions: this._highlightedLines.has(s5.line) ? void 0 : { color: t5.matchOverviewRuler, position: "center" } });
            if (o6) {
              const e7 = [];
              e7.push(s5), e7.push(o6.onRender(((e8) => this._applyStyles(e8, t5.matchBorder, false)))), e7.push(o6.onDispose((() => (0, r4.disposeArray)(e7))));
            }
            return o6;
          }
        }
        e5.SearchAddon = n4;
      })(), s4;
    })()));
  }
});

// ../../node_modules/.pnpm/@lit+reactive-element@2.1.2/node_modules/@lit/reactive-element/css-tag.js
var t = globalThis;
var e = t.ShadowRoot && (void 0 === t.ShadyCSS || t.ShadyCSS.nativeShadow) && "adoptedStyleSheets" in Document.prototype && "replace" in CSSStyleSheet.prototype;
var s = Symbol();
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
Symbol.metadata ?? (Symbol.metadata = Symbol("metadata")), a.litPropertyMetadata ?? (a.litPropertyMetadata = /* @__PURE__ */ new WeakMap());
var y = class extends HTMLElement {
  static addInitializer(t3) {
    this._$Ei(), (this.l ?? (this.l = [])).push(t3);
  }
  static get observedAttributes() {
    return this.finalize(), this._$Eh && [...this._$Eh.keys()];
  }
  static createProperty(t3, s4 = b) {
    if (s4.state && (s4.attribute = false), this._$Ei(), this.prototype.hasOwnProperty(t3) && ((s4 = Object.create(s4)).wrapped = true), this.elementProperties.set(t3, s4), !s4.noAccessor) {
      const i5 = Symbol(), h3 = this.getPropertyDescriptor(t3, i5, s4);
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
var E = Symbol.for("lit-noChange");
var A = Symbol.for("lit-nothing");
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

// src/web-components/ship-terminal.styles.ts
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

// src/web-components/ship-terminal.theme.ts
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
function resolve(el, prop) {
  return getComputedStyle(el).getPropertyValue(prop).trim();
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

// src/web-components/ship-terminal.init.ts
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
    { SearchAddon }
  ] = await Promise.all([
    Promise.resolve().then(() => __toESM(require_xterm(), 1)),
    Promise.resolve().then(() => __toESM(require_addon_fit(), 1)),
    Promise.resolve().then(() => __toESM(require_addon_web_links(), 1)),
    Promise.resolve().then(() => __toESM(require_addon_search(), 1))
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
  term.open(opts.container);
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

// src/web-components/ship-terminal.ts
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
