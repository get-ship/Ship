var F=globalThis,G=F.ShadowRoot&&(F.ShadyCSS===void 0||F.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,N=Symbol(),Z=new WeakMap;class T{constructor(e,n,a){if(this._$cssResult$=!0,a!==N)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=e,this.t=n}get styleSheet(){let e=this.o,n=this.t;if(G&&e===void 0){let a=n!==void 0&&n.length===1;a&&(e=Z.get(n)),e===void 0&&((this.o=e=new CSSStyleSheet).replaceSync(this.cssText),a&&Z.set(n,e))}return e}toString(){return this.cssText}}var P=(e)=>new T(typeof e=="string"?e:e+"",void 0,N),C=(e,...n)=>{let a=e.length===1?e[0]:n.reduce((r,o,c)=>r+((l)=>{if(l._$cssResult$===!0)return l.cssText;if(typeof l=="number")return l;throw Error("Value passed to 'css' function must be a 'css' function result: "+l+". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.")})(o)+e[c+1],e[0]);return new T(a,e,N)},U=(e,n)=>{if(G)e.adoptedStyleSheets=n.map((a)=>a instanceof CSSStyleSheet?a:a.styleSheet);else for(let a of n){let r=document.createElement("style"),o=F.litNonce;o!==void 0&&r.setAttribute("nonce",o),r.textContent=a.cssText,e.appendChild(r)}},O=G?(e)=>e:(e)=>e instanceof CSSStyleSheet?((n)=>{let a="";for(let r of n.cssRules)a+=r.cssText;return P(a)})(e):e;var{is:fe,defineProperty:me,getOwnPropertyDescriptor:ve,getOwnPropertyNames:ge,getOwnPropertySymbols:ke,getPrototypeOf:xe}=Object,q=globalThis,M=q.trustedTypes,we=M?M.emptyScript:"",ye=q.reactiveElementPolyfillSupport,_=(e,n)=>e,Q={toAttribute(e,n){switch(n){case Boolean:e=e?we:null;break;case Object:case Array:e=e==null?e:JSON.stringify(e)}return e},fromAttribute(e,n){let a=e;switch(n){case Boolean:a=e!==null;break;case Number:a=e===null?null:Number(e);break;case Object:case Array:try{a=JSON.parse(e)}catch(r){a=null}}return a}},j=(e,n)=>!fe(e,n),A={attribute:!0,type:String,converter:Q,reflect:!1,useDefault:!1,hasChanged:j};Symbol.metadata??=Symbol("metadata"),q.litPropertyMetadata??=new WeakMap;class f extends HTMLElement{static addInitializer(e){this._$Ei(),(this.l??=[]).push(e)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(e,n=A){if(n.state&&(n.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(e)&&((n=Object.create(n)).wrapped=!0),this.elementProperties.set(e,n),!n.noAccessor){let a=Symbol(),r=this.getPropertyDescriptor(e,a,n);r!==void 0&&me(this.prototype,e,r)}}static getPropertyDescriptor(e,n,a){let{get:r,set:o}=ve(this.prototype,e)??{get(){return this[n]},set(c){this[n]=c}};return{get:r,set(c){let l=r?.call(this);o?.call(this,c),this.requestUpdate(e,l,a)},configurable:!0,enumerable:!0}}static getPropertyOptions(e){return this.elementProperties.get(e)??A}static _$Ei(){if(this.hasOwnProperty(_("elementProperties")))return;let e=xe(this);e.finalize(),e.l!==void 0&&(this.l=[...e.l]),this.elementProperties=new Map(e.elementProperties)}static finalize(){if(this.hasOwnProperty(_("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(_("properties"))){let n=this.properties,a=[...ge(n),...ke(n)];for(let r of a)this.createProperty(r,n[r])}let e=this[Symbol.metadata];if(e!==null){let n=litPropertyMetadata.get(e);if(n!==void 0)for(let[a,r]of n)this.elementProperties.set(a,r)}this._$Eh=new Map;for(let[n,a]of this.elementProperties){let r=this._$Eu(n,a);r!==void 0&&this._$Eh.set(r,n)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(e){let n=[];if(Array.isArray(e)){let a=new Set(e.flat(1/0).reverse());for(let r of a)n.unshift(O(r))}else e!==void 0&&n.push(O(e));return n}static _$Eu(e,n){let a=n.attribute;return a===!1?void 0:typeof a=="string"?a:typeof e=="string"?e.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise((e)=>this.enableUpdating=e),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach((e)=>e(this))}addController(e){(this._$EO??=new Set).add(e),this.renderRoot!==void 0&&this.isConnected&&e.hostConnected?.()}removeController(e){this._$EO?.delete(e)}_$E_(){let e=new Map,n=this.constructor.elementProperties;for(let a of n.keys())this.hasOwnProperty(a)&&(e.set(a,this[a]),delete this[a]);e.size>0&&(this._$Ep=e)}createRenderRoot(){let e=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return U(e,this.constructor.elementStyles),e}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach((e)=>e.hostConnected?.())}enableUpdating(e){}disconnectedCallback(){this._$EO?.forEach((e)=>e.hostDisconnected?.())}attributeChangedCallback(e,n,a){this._$AK(e,a)}_$ET(e,n){let a=this.constructor.elementProperties.get(e),r=this.constructor._$Eu(e,a);if(r!==void 0&&a.reflect===!0){let o=(a.converter?.toAttribute!==void 0?a.converter:Q).toAttribute(n,a.type);this._$Em=e,o==null?this.removeAttribute(r):this.setAttribute(r,o),this._$Em=null}}_$AK(e,n){let a=this.constructor,r=a._$Eh.get(e);if(r!==void 0&&this._$Em!==r){let o=a.getPropertyOptions(r),c=typeof o.converter=="function"?{fromAttribute:o.converter}:o.converter?.fromAttribute!==void 0?o.converter:Q;this._$Em=r;let l=c.fromAttribute(n,o.type);this[r]=l??this._$Ej?.get(r)??l,this._$Em=null}}requestUpdate(e,n,a,r=!1,o){if(e!==void 0){let c=this.constructor;if(r===!1&&(o=this[e]),a??=c.getPropertyOptions(e),!((a.hasChanged??j)(o,n)||a.useDefault&&a.reflect&&o===this._$Ej?.get(e)&&!this.hasAttribute(c._$Eu(e,a))))return;this.C(e,n,a)}this.isUpdatePending===!1&&(this._$ES=this._$EP())}C(e,n,{useDefault:a,reflect:r,wrapped:o},c){a&&!(this._$Ej??=new Map).has(e)&&(this._$Ej.set(e,c??n??this[e]),o!==!0||c!==void 0)||(this._$AL.has(e)||(this.hasUpdated||a||(n=void 0),this._$AL.set(e,n)),r===!0&&this._$Em!==e&&(this._$Eq??=new Set).add(e))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(n){Promise.reject(n)}let e=this.scheduleUpdate();return e!=null&&await e,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(let[r,o]of this._$Ep)this[r]=o;this._$Ep=void 0}let a=this.constructor.elementProperties;if(a.size>0)for(let[r,o]of a){let{wrapped:c}=o,l=this[r];c!==!0||this._$AL.has(r)||l===void 0||this.C(r,void 0,o,l)}}let e=!1,n=this._$AL;try{e=this.shouldUpdate(n),e?(this.willUpdate(n),this._$EO?.forEach((a)=>a.hostUpdate?.()),this.update(n)):this._$EM()}catch(a){throw e=!1,this._$EM(),a}e&&this._$AE(n)}willUpdate(e){}_$AE(e){this._$EO?.forEach((n)=>n.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(e)),this.updated(e)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(e){return!0}update(e){this._$Eq&&=this._$Eq.forEach((n)=>this._$ET(n,this[n])),this._$EM()}updated(e){}firstUpdated(e){}}f.elementStyles=[],f.shadowRootOptions={mode:"open"},f[_("elementProperties")]=new Map,f[_("finalized")]=new Map,ye?.({ReactiveElement:f}),(q.reactiveElementVersions??=[]).push("2.1.2");var W=globalThis,H=(e)=>e,J=W.trustedTypes,t=J?J.createPolicy("lit-html",{createHTML:(e)=>e}):void 0;var m=`lit$${Math.random().toFixed(9).slice(2)}$`,le="?"+m,Ee=`<${le}>`,x=document,K=()=>x.createComment(""),D=(e)=>e===null||typeof e!="object"&&typeof e!="function",X=Array.isArray,Ce=(e)=>X(e)||typeof e?.[Symbol.iterator]=="function";var $=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,ee=/-->/g,ne=/>/g,g=RegExp(`>|[ 	
\f\r](?:([^\\s"'>=/]+)([ 	
\f\r]*=[ 	
\f\r]*(?:[^ 	
\f\r"'\`<>=]|("|')|))|$)`,"g"),ae=/'/g,re=/"/g,ce=/^(?:script|style|textarea|title)$/i,Y=(e)=>(n,...a)=>({_$litType$:e,strings:n,values:a}),y=Y(1),Fe=Y(2),Ge=Y(3),w=Symbol.for("lit-noChange"),h=Symbol.for("lit-nothing"),oe=new WeakMap,k=x.createTreeWalker(x,129);function de(e,n){if(!X(e)||!e.hasOwnProperty("raw"))throw Error("invalid template strings array");return t!==void 0?t.createHTML(n):n}var _e=(e,n)=>{let a=e.length-1,r=[],o,c=n===2?"<svg>":n===3?"<math>":"",l=$;for(let d=0;d<a;d++){let p=e[d],S,i,s=-1,u=0;for(;u<p.length&&(l.lastIndex=u,i=l.exec(p),i!==null);)u=l.lastIndex,l===$?i[1]==="!--"?l=ee:i[1]!==void 0?l=ne:i[2]!==void 0?(ce.test(i[2])&&(o=RegExp("</"+i[2],"g")),l=g):i[3]!==void 0&&(l=g):l===g?i[0]===">"?(l=o??$,s=-1):i[1]===void 0?s=-2:(s=l.lastIndex-i[2].length,S=i[1],l=i[3]===void 0?g:i[3]==='"'?re:ae):l===re||l===ae?l=g:l===ee||l===ne?l=$:(l=g,o=void 0);let v=l===g&&e[d+1].startsWith("/>")?" ":"";c+=l===$?p+Ee:s>=0?(r.push(S),p.slice(0,s)+"$lit$"+p.slice(s)+m+v):p+m+(s===-2?d:v)}return[de(e,c+(e[a]||"<?>")+(n===2?"</svg>":n===3?"</math>":"")),r]};class z{constructor({strings:e,_$litType$:n},a){let r;this.parts=[];let o=0,c=0,l=e.length-1,d=this.parts,[p,S]=_e(e,n);if(this.el=z.createElement(p,a),k.currentNode=this.el.content,n===2||n===3){let i=this.el.content.firstChild;i.replaceWith(...i.childNodes)}for(;(r=k.nextNode())!==null&&d.length<l;){if(r.nodeType===1){if(r.hasAttributes())for(let i of r.getAttributeNames())if(i.endsWith("$lit$")){let s=S[c++],u=r.getAttribute(i).split(m),v=/([.?@])?(.*)/.exec(s);d.push({type:1,index:o,name:v[2],strings:u,ctor:v[1]==="."?pe:v[1]==="?"?he:v[1]==="@"?se:L}),r.removeAttribute(i)}else i.startsWith(m)&&(d.push({type:6,index:o}),r.removeAttribute(i));if(ce.test(r.tagName)){let i=r.textContent.split(m),s=i.length-1;if(s>0){r.textContent=J?J.emptyScript:"";for(let u=0;u<s;u++)r.append(i[u],K()),k.nextNode(),d.push({type:2,index:++o});r.append(i[s],K())}}}else if(r.nodeType===8)if(r.data===le)d.push({type:2,index:o});else{let i=-1;for(;(i=r.data.indexOf(m,i+1))!==-1;)d.push({type:7,index:o}),i+=m.length-1}o++}}static createElement(e,n){let a=x.createElement("template");return a.innerHTML=e,a}}function E(e,n,a=e,r){if(n===w)return n;let o=r!==void 0?a._$Co?.[r]:a._$Cl,c=D(n)?void 0:n._$litDirective$;return o?.constructor!==c&&(o?._$AO?.(!1),c===void 0?o=void 0:(o=new c(e),o._$AT(e,a,r)),r!==void 0?(a._$Co??=[])[r]=o:a._$Cl=o),o!==void 0&&(n=E(e,o._$AS(e,n.values),o,r)),n}class ie{constructor(e,n){this._$AV=[],this._$AN=void 0,this._$AD=e,this._$AM=n}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(e){let{el:{content:n},parts:a}=this._$AD,r=(e?.creationScope??x).importNode(n,!0);k.currentNode=r;let o=k.nextNode(),c=0,l=0,d=a[0];for(;d!==void 0;){if(c===d.index){let p;d.type===2?p=new B(o,o.nextSibling,this,e):d.type===1?p=new d.ctor(o,d.name,d.strings,this,e):d.type===6&&(p=new ue(o,this,e)),this._$AV.push(p),d=a[++l]}c!==d?.index&&(o=k.nextNode(),c++)}return k.currentNode=x,r}p(e){let n=0;for(let a of this._$AV)a!==void 0&&(a.strings!==void 0?(a._$AI(e,a,n),n+=a.strings.length-2):a._$AI(e[n])),n++}}class B{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(e,n,a,r){this.type=2,this._$AH=h,this._$AN=void 0,this._$AA=e,this._$AB=n,this._$AM=a,this.options=r,this._$Cv=r?.isConnected??!0}get parentNode(){let e=this._$AA.parentNode,n=this._$AM;return n!==void 0&&e?.nodeType===11&&(e=n.parentNode),e}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(e,n=this){e=E(this,e,n),D(e)?e===h||e==null||e===""?(this._$AH!==h&&this._$AR(),this._$AH=h):e!==this._$AH&&e!==w&&this._(e):e._$litType$!==void 0?this.$(e):e.nodeType!==void 0?this.T(e):Ce(e)?this.k(e):this._(e)}O(e){return this._$AA.parentNode.insertBefore(e,this._$AB)}T(e){this._$AH!==e&&(this._$AR(),this._$AH=this.O(e))}_(e){this._$AH!==h&&D(this._$AH)?this._$AA.nextSibling.data=e:this.T(x.createTextNode(e)),this._$AH=e}$(e){let{values:n,_$litType$:a}=e,r=typeof a=="number"?this._$AC(e):(a.el===void 0&&(a.el=z.createElement(de(a.h,a.h[0]),this.options)),a);if(this._$AH?._$AD===r)this._$AH.p(n);else{let o=new ie(r,this),c=o.u(this.options);o.p(n),this.T(c),this._$AH=o}}_$AC(e){let n=oe.get(e.strings);return n===void 0&&oe.set(e.strings,n=new z(e)),n}k(e){X(this._$AH)||(this._$AH=[],this._$AR());let n=this._$AH,a,r=0;for(let o of e)r===n.length?n.push(a=new B(this.O(K()),this.O(K()),this,this.options)):a=n[r],a._$AI(o),r++;r<n.length&&(this._$AR(a&&a._$AB.nextSibling,r),n.length=r)}_$AR(e=this._$AA.nextSibling,n){for(this._$AP?.(!1,!0,n);e!==this._$AB;){let a=H(e).nextSibling;H(e).remove(),e=a}}setConnected(e){this._$AM===void 0&&(this._$Cv=e,this._$AP?.(e))}}class L{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(e,n,a,r,o){this.type=1,this._$AH=h,this._$AN=void 0,this.element=e,this.name=n,this._$AM=r,this.options=o,a.length>2||a[0]!==""||a[1]!==""?(this._$AH=Array(a.length-1).fill(new String),this.strings=a):this._$AH=h}_$AI(e,n=this,a,r){let o=this.strings,c=!1;if(o===void 0)e=E(this,e,n,0),c=!D(e)||e!==this._$AH&&e!==w,c&&(this._$AH=e);else{let l=e,d,p;for(e=o[0],d=0;d<o.length-1;d++)p=E(this,l[a+d],n,d),p===w&&(p=this._$AH[d]),c||=!D(p)||p!==this._$AH[d],p===h?e=h:e!==h&&(e+=(p??"")+o[d+1]),this._$AH[d]=p}c&&!r&&this.j(e)}j(e){e===h?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,e??"")}}class pe extends L{constructor(){super(...arguments),this.type=3}j(e){this.element[this.name]=e===h?void 0:e}}class he extends L{constructor(){super(...arguments),this.type=4}j(e){this.element.toggleAttribute(this.name,!!e&&e!==h)}}class se extends L{constructor(e,n,a,r,o){super(e,n,a,r,o),this.type=5}_$AI(e,n=this){if((e=E(this,e,n,0)??h)===w)return;let a=this._$AH,r=e===h&&a!==h||e.capture!==a.capture||e.once!==a.once||e.passive!==a.passive,o=e!==h&&(a===h||r);r&&this.element.removeEventListener(this.name,this,a),o&&this.element.addEventListener(this.name,this,e),this._$AH=e}handleEvent(e){typeof this._$AH=="function"?this._$AH.call(this.options?.host??this.element,e):this._$AH.handleEvent(e)}}class ue{constructor(e,n,a){this.element=e,this.type=6,this._$AN=void 0,this._$AM=n,this.options=a}get _$AU(){return this._$AM._$AU}_$AI(e){E(this,e)}}var $e=W.litHtmlPolyfillSupport;$e?.(z,B),(W.litHtmlVersions??=[]).push("3.3.2");var be=(e,n,a)=>{let r=a?.renderBefore??n,o=r._$litPart$;if(o===void 0){let c=a?.renderBefore??null;r._$litPart$=o=new B(n.insertBefore(K(),c),c,void 0,a??{})}return o._$AI(e),o};var V=globalThis;class b extends f{constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){let e=super.createRenderRoot();return this.renderOptions.renderBefore??=e.firstChild,e}update(e){let n=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(e),this._$Do=be(n,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return w}}b._$litElement$=!0,b.finalized=!0,V.litElementHydrateSupport?.({LitElement:b});var Ke=V.litElementPolyfillSupport;Ke?.({LitElement:b});(V.litElementVersions??=[]).push("4.2.2");var De={M:"M",A:"A",D:"D",R:"R","?":"?"};class R extends b{static properties={path:{type:String,reflect:!0},basename:{type:String,reflect:!0},status:{type:String,reflect:!0},active:{type:Boolean,reflect:!0}};path="";basename="";status="M";active=!1;static styles=C`
    :host {
      display: flex;
      align-items: center;
      gap: 6px;
      width: 100%;
      padding: 5px 12px;
      background: transparent;
      border: 0;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 25%, transparent);
      font-family: var(--font-mono);
      font-size: 11px;
      color: var(--muted-foreground);
      cursor: pointer;
      text-align: left;
      box-sizing: border-box;
      transition: background var(--duration-fast) ease, color var(--duration-fast) ease;
    }
    :host(:hover) {
      background: color-mix(in oklch, var(--muted) 40%, transparent);
      color: var(--foreground);
    }
    :host([active]) {
      background: color-mix(in oklch, var(--primary) 8%, transparent);
      color: var(--foreground);
    }
    .badge {
      display: inline-block;
      font-family: var(--font-mono);
      font-size: 9px;
      line-height: 1;
      padding: 2px 4px;
      border-radius: var(--radius-sm);
      font-weight: 700;
      flex-shrink: 0;
      min-width: 16px;
      text-align: center;
    }
    :host([status="M"]) .badge,
    :host([status="S"]) .badge { background: color-mix(in oklch, var(--warning) 15%, transparent); color: var(--warning); }
    :host([status="A"]) .badge { background: color-mix(in oklch, var(--success) 15%, transparent); color: var(--success); }
    :host([status="D"]) .badge { background: color-mix(in oklch, var(--destructive) 15%, transparent); color: var(--destructive); }
    :host([status="?"]) .badge { background: color-mix(in oklch, var(--muted-foreground) 15%, transparent); color: var(--muted-foreground); }
    :host([status="R"]) .badge { background: color-mix(in oklch, var(--info) 15%, transparent); color: var(--info); }
    .name {
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
      flex: 1;
    }
  `;connectedCallback(){super.connectedCallback(),this.setAttribute("role","button"),this.setAttribute("tabindex","0"),this.addEventListener("click",this._onClick),this.addEventListener("keydown",this._onKey)}disconnectedCallback(){super.disconnectedCallback(),this.removeEventListener("click",this._onClick),this.removeEventListener("keydown",this._onKey)}_onClick=()=>{this.dispatchEvent(new CustomEvent("ship-select",{detail:{path:this.path},bubbles:!0,composed:!0}))};_onKey=(e)=>{if(e.key==="Enter"||e.key===" ")e.preventDefault(),this._onClick()};render(){return y`
      <span class="badge">${De[this.status]??this.status}</span>
      <span class="name" title="${this.path}">${this.basename||this.path}</span>
    `}}customElements.define("ship-file-row",R);class I extends b{static properties={name:{type:String,reflect:!0},active:{type:Boolean,reflect:!0},upstream:{type:String,reflect:!0},ahead:{type:Number,reflect:!0},behind:{type:Number,reflect:!0},selected:{type:Boolean,reflect:!0}};name="";active=!1;upstream=null;ahead=0;behind=0;selected=!1;static styles=C`
    :host {
      display: flex;
      align-items: center;
      gap: 8px;
      width: 100%;
      padding: 7px 12px;
      background: transparent;
      border: 0;
      border-bottom: 1px solid color-mix(in oklch, var(--border) 25%, transparent);
      font-size: 12px;
      color: var(--foreground);
      cursor: pointer;
      text-align: left;
      box-sizing: border-box;
      transition: background var(--duration-fast) ease;
    }
    :host(:hover) { background: color-mix(in oklch, var(--muted) 40%, transparent); }
    :host([selected]) { background: color-mix(in oklch, var(--primary) 8%, transparent); }
    .dot {
      width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
      background: var(--muted-foreground);
    }
    :host([active]) .dot { background: var(--success); }
    .info { flex: 1; min-width: 0; }
    .branch-name {
      font-family: var(--font-mono);
      font-size: 11.5px;
      font-weight: 500;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .upstream {
      font-size: 10px;
      color: var(--muted-foreground);
      margin-top: 1px;
    }
    .counters {
      display: flex;
      flex-direction: column;
      align-items: flex-end;
      gap: 1px;
      font-family: var(--font-mono);
      font-size: 10px;
      flex-shrink: 0;
    }
    .up { color: var(--success); }
    .down { color: var(--warning); }
  `;connectedCallback(){super.connectedCallback(),this.setAttribute("role","button"),this.setAttribute("tabindex","0"),this.addEventListener("click",this._onClick),this.addEventListener("keydown",this._onKey)}disconnectedCallback(){super.disconnectedCallback(),this.removeEventListener("click",this._onClick),this.removeEventListener("keydown",this._onKey)}_onClick=()=>{this.dispatchEvent(new CustomEvent("ship-select",{detail:{name:this.name},bubbles:!0,composed:!0}))};_onKey=(e)=>{if(e.key==="Enter"||e.key===" ")e.preventDefault(),this._onClick()};render(){return y`
      <span class="dot"></span>
      <div class="info">
        <div class="branch-name" title="${this.name}">${this.name}</div>
        ${this.upstream?y`<div class="upstream">${this.upstream}</div>`:""}
      </div>
      <div class="counters">
        ${this.ahead>0?y`<span class="up">↑${this.ahead}</span>`:""}
        ${this.behind>0?y`<span class="down">↓${this.behind}</span>`:""}
      </div>
    `}}customElements.define("ship-branch-row",I);export{R as ShipFileRow,I as ShipBranchRow};
