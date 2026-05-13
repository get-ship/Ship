var Pe=Symbol.for("sui:error-boundary");function Ne(e,n){e[Pe]=n}function He(e){delete e[Pe]}function ct(e){let n=e;while(n){if(n instanceof Element){let t=n[Pe];if(t)return t}let r=n.parentNode;if(r){n=r;continue}let o=n;if(o.host){n=o.host;continue}n=null}return null}function vt(e,n,r){try{return r()}catch(o){let t=ct(e);if(t){t.catch(o,n);return}console.error("[sui] uncaught throw (no error boundary):",o);return}}var it=null,lt=null,ho=0,vo=new Set,Dt=[];function Se(e,n){Dt.push(e);try{return n()}finally{Dt.pop()}}function go(){return Dt.length>0?Dt[Dt.length-1]:null}function pr(e){if(it===null)return;if(e.has(it))return;let n=it;if(e.add(n),lt!==null)lt.push(()=>e.delete(n))}function P(e){let n=e,r=new Set,o=()=>{return pr(r),n};return o.set=(t)=>{let a=typeof t==="function"?t(n):t;if(Object.is(a,n))return;if(n=a,ho>0)for(let b of r)vo.add(b);else for(let b of[...r])b()},o.__subs=r,o.__peek=()=>n,o}function J(e){let n,r=!0,o=new Set,t=[],a=()=>{if(r)return;r=!0;for(let f of[...o])f()},b=()=>{for(let d of t)d();t=[];let f=it,p=lt;it=a,lt=t;try{n=e()}finally{it=f,lt=p}r=!1},c=()=>{if(pr(o),r)b();return n};return c.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},c.__subs=o,c.__peek=()=>{if(r)b();return n},c}function D(e){let n=void 0,r=!0,o=[],t=go(),a=()=>{if(!r)return;if(n){try{n()}catch(f){console.error("[sui] effect cleanup threw:",f)}n=void 0}for(let f of o)f();o=[];let b=it,c=lt;it=a,lt=o;try{n=e()}catch(f){let p=t?ct(t):null;if(p)p.catch(f,"effect");else console.error("[sui] effect threw:",f)}finally{it=b,lt=c}};return a(),()=>{r=!1;for(let b of o)b();if(o=[],n){try{n()}catch(b){console.error("[sui] effect cleanup threw:",b)}n=void 0}}}var qt=null;function Ot(e,n){let r=qt;qt=e;try{return n()}finally{qt=r}}function st(e){let n=qt;if(!n)return;n.cleanup(e)}function Tt(){let e=[],n=!1;return{cleanup(o){if(n){try{o()}catch(t){console.error("[sui/scope] late cleanup threw:",t)}return}e.push(o)},dispose(){if(n)return;n=!0;for(let o=e.length-1;o>=0;o--)try{e[o]()}catch(t){console.error("[sui/scope] cleanup threw:",t)}e.length=0},get disposed(){return n}}}var yo=Symbol("sui.shared");function fr(e){return!!e?.[yo]}function Fe(e,n,r){e[`__sui_ctx_${n}`]=r}function Be(e){let n={};for(let r in e)if(Object.prototype.hasOwnProperty.call(e,r)&&fr(e[r]))n[r]=e[r];return n}function Y(e=null){return P(null)}function T(e,n,r){let o=e[n];if(typeof o==="function"&&typeof o.set==="function")return o.set(r),o;let t=P(r);return e[n]=t,t}function tt(e,n,r){let o=()=>{if(e.assignedNodes({flatten:!0}).some(xo))n.removeAttribute(r);else n.setAttribute(r,"")};return e.addEventListener("slotchange",o),o(),()=>{e.removeEventListener("slotchange",o)}}function xo(e){if(e.nodeType===Node.TEXT_NODE)return(e.textContent??"").trim().length>0;if(e.nodeType===Node.ELEMENT_NODE){let n=e;return(n.textContent??"").trim().length>0||n.childElementCount>0}return!1}function mr(e,n,r={}){br(e,n,r)}function br(e,n,r){if(r.skip?.(e))return;if(e.tagName!==n.tagName){e.replaceWith(n.cloneNode(!0));return}wo(e,n),Ao(e,n,r)}function wo(e,n){let r=Array.from(e.attributes);for(let t of r)if(!n.hasAttribute(t.name))e.removeAttribute(t.name);let o=Array.from(n.attributes);for(let t of o)if(e.getAttribute(t.name)!==t.value)e.setAttribute(t.name,t.value);if(e instanceof HTMLInputElement&&n instanceof HTMLInputElement){if(n.hasAttribute("value")&&e.value!==n.value)e.value=n.value;if(e.checked!==n.checked)e.checked=n.checked}}function Ao(e,n,r){let o=Array.from(e.childNodes),t=Array.from(n.childNodes),a=Math.max(o.length,t.length);for(let b=0;b<a;b++){let c=o[b],f=t[b];if(!f&&c){e.removeChild(c);continue}if(!c&&f){e.appendChild(f.cloneNode(!0));continue}if(!c||!f)continue;ko(c,f,e,r)}}function ko(e,n,r,o){if(e.nodeType!==n.nodeType){r.replaceChild(n.cloneNode(!0),e);return}if(e.nodeType===Node.TEXT_NODE||e.nodeType===Node.COMMENT_NODE){if(e.nodeValue!==n.nodeValue)e.nodeValue=n.nodeValue;return}if(e.nodeType===Node.ELEMENT_NODE)br(e,n,o)}function F(e,n){return D(()=>{let r=n();e.textContent=r==null?"":String(r)})}function E(e,n,r){return D(()=>{let o=r();if(o==null||o===!1||o==="")e.removeAttribute(n);else if(o===!0)e.setAttribute(n,"");else e.setAttribute(n,String(o))})}function gt(e,n,r){return D(()=>{if(r())e.classList.add(n);else e.classList.remove(n)})}function $(e,n,r){return D(()=>{e[n]=r()})}function pt(e,n,r){let o=D(()=>{let a=n();if(e.value!==a)e.value=a}),t=()=>{let a=e.value;if(n.set(a),r)r(a)};return e.addEventListener("input",t),()=>{o(),e.removeEventListener("input",t)}}function ft(e,n){let r=D(()=>{let t=n();if(e.checked!==t)e.checked=t}),o=()=>n.set(e.checked);return e.addEventListener("change",o),()=>{r(),e.removeEventListener("change",o)}}function nt(e,n){let o=e.namespaceURI==="http://www.w3.org/2000/svg",t=e.tagName.toLowerCase();return D(()=>{let a=n(),b=o?e.ownerDocument.createElementNS("http://www.w3.org/2000/svg",t):e.ownerDocument.createElement(t);b.innerHTML=a,mr(e,b)})}function V(e,n,r,o={}){let t=o.modifiers??[],a,b=!0,c=(p)=>{for(let i of t)switch(i.kind){case"preventDefault":p.preventDefault();break;case"stopPropagation":p.stopPropagation();break;case"stopImmediatePropagation":p.stopImmediatePropagation();break;case"self":if(p.target!==e)return;break;case"trusted":if(!p.isTrusted)return;break;case"key":{if(!hr(p,i.arg??""))return;break}}let d=t.find((i)=>i.kind==="debounce"),l=t.find((i)=>i.kind==="throttle");if(d){let i=Number(d.arg)||0;if(a!==void 0)clearTimeout(a);a=setTimeout(()=>{a=void 0,r(p)},i);return}if(l){let i=Number(l.arg)||0;if(!b)return;b=!1,r(p),a=setTimeout(()=>{b=!0},i);return}r(p)},f={};if(o.capture)f.capture=!0;if(o.passive)f.passive=!0;if(o.once)f.once=!0;return e.addEventListener(n,c,f),()=>{if(e.removeEventListener(n,c,f),a!==void 0)clearTimeout(a)}}function hr(e,n){let r=n.split("+").map((d)=>d.toLowerCase()),o=r.pop()??"",t=r.includes("meta")||r.includes("cmd"),a=r.includes("ctrl")||r.includes("control"),b=r.includes("shift"),c=r.includes("alt")||r.includes("option"),f=r.includes("mod"),p=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(t&&!e.metaKey)return!1;if(a&&!e.ctrlKey)return!1;if(b&&!e.shiftKey)return!1;if(c&&!e.altKey)return!1;if(f&&!(p?e.metaKey:e.ctrlKey))return!1;return e.key.toLowerCase()===o}function vr(e,n,r){e.dispatchEvent(new CustomEvent(n,{detail:r,bubbles:!0,composed:!0}))}function gr(e,n){let r=e,o=Object.keys(n);for(let t of o)r[t]=n[t];st(()=>{for(let t of o)if(r[t]===n[t])try{delete r[t]}catch{r[t]=void 0}})}function Jt(e){let n=e;n.emit=function(r,o){vr(e,r,o)},n.expose=function(r){gr(e,r)}}function W(e,n,r,o){e.addEventListener(n,r,o),st(()=>e.removeEventListener(n,r,o))}function Ge(e,n){let r=D(()=>{let o=e();if(!o)return;let t=()=>{n(o.assignedElements())};return o.addEventListener("slotchange",t),t(),()=>o.removeEventListener("slotchange",t)});st(r)}var Ue=Symbol("sui:transition");function Yt(e){return e[Ue]}function Eo(e){let n=e,r=n[Ue];if(!r)r={inOpts:{},outOpts:{}},n[Ue]=r;return r}function dt(e,n,r={},o="both"){let t=Eo(e);if(o==="in"||o==="both")t.inT=n,t.inOpts=r;if(o==="out"||o==="both")t.outT=n,t.outOpts=r;if(!e.hasAttribute("data-sui-transition"))e.setAttribute("data-sui-transition","")}function Co(){if(typeof window>"u"||typeof window.matchMedia!=="function")return!1;try{return window.matchMedia("(prefers-reduced-motion: reduce)").matches}catch{return!1}}function wr(e){if(!Co())return e;return{...e,duration:0,delay:0}}function Ar(e){let n=Yt(e);if(!n?.inT?.in)return;n.current?.cancel();let r=n.inT.in(e,wr(n.inOpts));return n.current=r,r}function kr(e){let n=Yt(e);if(!n?.outT?.out)return;n.current?.cancel();let r=n.outT.out(e,wr(n.outOpts));return n.current=r,r}function Xe(e){if(e.nodeType!==1)return;let n=e;if(n.hasAttribute("data-sui-transition")){let r=Yt(n);if(r?.current?.cancel(),r)r.current=void 0}for(let r of n.querySelectorAll("[data-sui-transition]")){let o=Yt(r);if(o?.current?.cancel(),o)o.current=void 0}}function Er(e,n){for(let r of e){if(r.nodeType!==1)continue;let o=r;if(o.hasAttribute("data-sui-transition"))n(o);for(let t of o.querySelectorAll("[data-sui-transition]"))n(t)}}function wt(e){Er(e,(n)=>{Ar(n)})}function Mt(e){let n=[];if(Er(e,(r)=>{let o=kr(r);if(o)n.push(o.finished.catch(()=>{}))}),n.length===0)return null;return Promise.all(n).then(()=>{return})}var zo=200,jo="cubic-bezier(0.4, 0, 0.2, 1)",Lo=0;function yt(e){return{duration:typeof e.duration==="number"?e.duration:zo,easing:typeof e.easing==="string"?e.easing:jo,delay:typeof e.delay==="number"?e.delay:Lo,fill:"both"}}function xt(e,n,r){let o=e.animate;if(typeof o!=="function")return;return o.call(e,n,r)}var Pt={in:(e,n)=>xt(e,[{opacity:0},{opacity:1}],yt(n)),out:(e,n)=>xt(e,[{opacity:1},{opacity:0}],yt(n))};function yr(e,n){let r=e==="y"?"translateY(-100%)":"translateX(-100%)",o="translate(0, 0)";return n==="in"?[{opacity:0,transform:r},{opacity:1,transform:"translate(0, 0)"}]:[{opacity:1,transform:"translate(0, 0)"},{opacity:0,transform:r}]}var Re={in:(e,n)=>{let r=n.axis==="x"?"x":"y";return xt(e,yr(r,"in"),yt(n))},out:(e,n)=>{let r=n.axis==="x"?"x":"y";return xt(e,yr(r,"out"),yt(n))}};function xr(e,n){let r={opacity:0,transform:`scale(${e})`},o={opacity:1,transform:"scale(1)"};return n==="in"?[r,o]:[o,r]}var qe={in:(e,n)=>{let r=typeof n.from==="number"?n.from:0;return xt(e,xr(r,"in"),yt(n))},out:(e,n)=>{let r=typeof n.from==="number"?n.from:0;return xt(e,xr(r,"out"),yt(n))}};function Do(e){let n=e.length,r=Array(n).fill(!1);if(n===0)return r;let o=Array(n).fill(-1),t=[];for(let b=0;b<n;b++){let c=e[b],f=0,p=t.length;while(f<p){let d=f+p>>>1;if(e[t[d]]<c)f=d+1;else p=d}if(f>0)o[b]=t[f-1];t[f]=b}let a=t[t.length-1];while(a>=0)r[a]=!0,a=o[a];return r}function X(e){let{anchor:n,items:r,key:o,render:t}=e,a=[],b=!0,c=new Set,f=(d)=>{for(let l of d.nodes)try{l.parentNode?.removeChild(l)}catch{}d.cleanup?.()},p=D(()=>{if(!b)return;let d=r(),l=new Map;for(let x=0;x<d.length;x++){let C=d[x],z=o(C,x);if(l.has(z))console.warn("[sui] {#for}: duplicate key",z,"at index",x,"(prior at",l.get(z).index,"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");l.set(z,{item:C,index:x})}let i=new Map;for(let x of a)i.set(x.key,x);for(let x of a){if(l.has(x.key))continue;let C=Mt(x.nodes);if(C===null){f(x);continue}c.add(x),C.then(()=>{if(!c.delete(x))return;f(x)})}let u=[],s=[];for(let x=0;x<d.length;x++){let C=d[x],z=o(C,x),M=i.get(z),N;if(M)M.item=C,vt(n,"render",()=>M.update?.(C,x)),N=M;else{let S=vt(n,"render",()=>t(C,x));if(!S)continue;N={key:z,item:C,nodes:S.nodes,...S.cleanup&&{cleanup:S.cleanup},...S.update&&{update:S.update}},s.push(...S.nodes)}u.push(N)}let m=new Map;for(let x=0;x<a.length;x++)m.set(a[x].key,x);let h=[],v=[];for(let x=0;x<u.length;x++){let C=m.get(u[x].key);if(C!==void 0)h.push(x),v.push(C)}let y=Do(v),w=new Set;for(let x=0;x<h.length;x++)if(y[x])w.add(h[x]);let k=n.parentNode,A=n;for(let x=0;x<u.length;x++){if(!k)break;let C=u[x];if(w.has(x))A=C.nodes[C.nodes.length-1]??A;else for(let z of C.nodes)k.insertBefore(z,A.nextSibling),A=z}if(a=u,s.length>0)wt(s)});return()=>{b=!1,p();for(let d of a)f(d);for(let d of c)f(d);c.clear(),a=[]}}function H(e){let{anchor:n}=e,{test:r,branches:o}=Oo(e)?{test:e.test,branches:e.branches}:To(e),t=null,a=-2,b=null,c=(p)=>{for(let d of p)try{d.parentNode?.removeChild(d)}catch{}},f=D(()=>{let p=r();if(t&&p===a)return;if(b&&p===b.idx&&p>=0&&p<o.length){let i=b;b=null;for(let u of i.nodes)Xe(u);t={nodes:i.nodes,...i.cleanup&&{cleanup:i.cleanup}},a=p,wt(t.nodes);return}if(t){let i={idx:a,nodes:t.nodes,...t.cleanup&&{cleanup:t.cleanup}},u=Mt(i.nodes);if(t=null,u===null)c(i.nodes),i.cleanup?.();else b=i,u.then(()=>{if(b!==i)return;c(i.nodes),i.cleanup?.(),b=null})}if(a=p,p<0||p>=o.length)return;let d=vt(n,"render",()=>o[p]());if(!d)return;t=d;let l=n.parentNode;if(!l)return;for(let i of t.nodes)l.insertBefore(i,n);wt(t.nodes)});return()=>{if(f(),t)c(t.nodes),t.cleanup?.(),t=null;if(b)c(b.nodes),b.cleanup?.(),b=null}}function Oo(e){return typeof e.test==="function"}function To(e){let{cond:n,then_:r,else_:o}=e;return{test:()=>n()?0:o?1:-1,branches:o?[r,o]:[r]}}var Je=(e)=>e??"",Ye=(e)=>{if(e===null||e==="")return NaN;return Number(e)},We=(e)=>{if(e===null)return!1;if(e.toLowerCase()==="false")return!1;return!0};function Wt(e){return(n)=>{if(n===null||n==="")return e;try{return JSON.parse(n)}catch{return e}}}function Ze(e,n){let r=new Set(e);return(o)=>o!==null&&r.has(o)?o:n}var Zt=(e)=>{switch(e.type){case"string":return Je;case"bool":return We;case"number":return Ye;case"union":{let n=e.values??[],r=e.default??n[0]??"";return Ze(n,r)}case"object":return Wt(e.default);case"array":return Wt(e.default??[])}};function zr(e,n,r,o){if(!n)return;let t={},a,b=r.formValue;if(b!==void 0){let{sig:f,propDef:p}=Cr(r,o,b,"formValue"),d=p.default,l=Zt(p);a=D(()=>{let i=f();n.setFormValue(Mo(i))}),t.reset=()=>{f.set(d)},t.restore=(i)=>{f.set(l(i))}}let c=r.formDisabled;if(c!==void 0){let{sig:f,propDef:p}=Cr(r,o,c,"formDisabled");if(p.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${c}" must be a bool-typed prop`);t.disabled=(d)=>{f.set(!!d)}}if(e.__shipSetFormHooks=t,a)e.__shipSetFormValueCleanup=a;return a}function jr(e){let n=e;delete n.__shipSetFormHooks,delete n.__shipSetFormValueCleanup}function Mo(e){if(e===null||e===void 0)return"";if(typeof e==="boolean")return e?"on":"";return String(e)}function Cr(e,n,r,o){let t=e.props[r];if(!t)throw Error(`[sui-runtime] ${o}: "${r}" — no matching prop on this component`);let a=n[At(r)];if(!a)throw Error(`[sui-runtime] ${o}: "${r}" — prop signal missing`);return{sig:a,propDef:t}}var g={string(e={}){return{type:"string",...e}},bool(e={}){return{type:"bool",...e}},number(e={}){return{type:"number",...e}},union(e,n={}){return{type:"union",values:e,...n}},object(e={}){return{type:"object",...e}},array(e={}){return{type:"array",...e}}},Po=(e,n)=>{if(n===void 0||n===null)return null;switch(e.type){case"string":return n===""?null:String(n);case"number":return Number.isNaN(n)?null:String(n);case"bool":return n?"":null;case"union":return String(n);case"object":case"array":try{return JSON.stringify(n)}catch{return null}}},No=(e,n)=>{let r=`ship-set-style-${e}`;if(document.getElementById(r))return;let o=document.createElement("style");o.id=r,o.textContent=`[data-component="${e}"] { ${n} }`,document.head.appendChild(o)},Ho=(e,n,r,o)=>{let t=e.getAttribute(n),a=t!==null?o(t):r.default;return P(a)},So=(e)=>{let n=[],r={};for(let[a,b]of Object.entries(e.props??{}))n.push(a),r[a]={camel:At(a),decode:Zt(b)};let o=e.formAssociated===!0||e.formValue!==void 0||e.formDisabled!==void 0,t={observedAttrs:n,props:r};if(o)t.formAssociated=!0;return t};function j(e,n){let r=So(n),o=Object.entries(n.props??{});Qe(e,r,(t,a,b)=>{let c=null;if(n.shadow){if(c=t.shadowRoot??t.attachShadow({mode:"open"}),n.styles){let p=document.createElement("style");p.textContent=n.styles,c.appendChild(p)}}else if(n.styles)No(e,n.styles);let f=[];for(let[p,d]of o){let l=At(p),i=r.props[p].decode,u=Ho(t,p,d,i);if(a[l]=u,d.reflect){let s=!0,m=D(()=>{let h=u();if(s){s=!1;return}let v=Po(d,h);if(v===null)t.removeAttribute(p);else t.setAttribute(p,v)});f.push(m)}}if(n.formValue!==void 0||n.formDisabled!==void 0){let p=zr(t,b,n,a);if(typeof p==="function")f.push(p)}if(n.setup){let p={props:a,host:t,cleanups:f};if(b)p.internals=b;n.setup(p)}return()=>{jr(t);for(let p of f.splice(0))try{p()}catch(d){console.error(`[ship-set:${e}] cleanup threw:`,d)}}})}var Lr=new Set;function Qe(e,n,r){if(customElements.get(e)){if(!Lr.has(e))Lr.add(e),console.warn(`[sui-runtime] customElements.define("${e}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(n.formAssociated){class t extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return n.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=e,Jt(this),this.scope=Tt();try{this.cleanup=Ot(this.scope,()=>Se(this,()=>r(this,this.propSignals,this.internals)))}catch(a){let b=ct(this.parentNode);if(b)b.catch(a,"mount");else console.error(`[ship-set:${e}] setup threw:`,a)}}disconnectedCallback(){try{this.cleanup?.()}catch(a){console.error(`[ship-set:${e}] cleanup threw:`,a)}this.cleanup=void 0;try{this.scope?.dispose()}catch(a){console.error(`[ship-set:${e}] scope dispose threw:`,a)}this.scope=void 0}attributeChangedCallback(a,b,c){let f=n.props[a];if(!f)return;let p=this.propSignals[f.camel];if(p)p.set(f.decode(c))}formResetCallback(){let a=this.__shipSetFormHooks;try{a?.reset?.()}catch(b){console.error(`[ship-set:${e}] formResetCallback threw:`,b)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(a,b){let c=this.__shipSetFormHooks;try{c?.restore?.(a)}catch(f){console.error(`[ship-set:${e}] formStateRestoreCallback threw:`,f)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:a,mode:b}}))}formDisabledCallback(a){let b=this.__shipSetFormHooks;try{b?.disabled?.(a)}catch(c){console.error(`[ship-set:${e}] formDisabledCallback threw:`,c)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:a}}))}}customElements.define(e,t);return}class o extends HTMLElement{static get observedAttributes(){return n.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=e,Jt(this),this.scope=Tt();try{this.cleanup=Ot(this.scope,()=>Se(this,()=>r(this,this.propSignals)))}catch(t){let a=ct(this.parentNode);if(a)a.catch(t,"mount");else console.error(`[ship-set:${e}] setup threw:`,t)}}disconnectedCallback(){try{this.cleanup?.()}catch(t){console.error(`[ship-set:${e}] cleanup threw:`,t)}this.cleanup=void 0;try{this.scope?.dispose()}catch(t){console.error(`[ship-set:${e}] scope dispose threw:`,t)}this.scope=void 0}attributeChangedCallback(t,a,b){let c=n.props[t];if(!c)return;let f=this.propSignals[c.camel];if(f)f.set(c.decode(b))}}customElements.define(e,o)}function At(e){return e.replace(/-([a-z])/g,(n,r)=>r.toUpperCase())}function Ve(e,n,r){let o=Go(e.state,{ship:n.ship,...r}),t={};for(let[i,u]of Object.entries(o))t[i]=P(u);let a={ship:n.ship,route:n.route??{},navigate:n.navigate??((i)=>{}),...r,...t},b=Uo(t,a),c={...a},f={};for(let[i,u]of Object.entries(e.derived??{}))f[i]=J(()=>u(c));Object.assign(c,f);let p={};for(let[i,u]of Object.entries(e.on??{}))p[i]=(...s)=>u(b,...s);let d={},l=typeof e.use==="function"?e.use(c):e.use??{};for(let[i,u]of Object.entries(l))d[i]=Ve(u,n,c);return{...t,...f,...p,...d}}function Go(e,n){if(e===void 0)return{};if(typeof e==="function")return e(n);return e}function Uo(e,n){return new Proxy({},{get(r,o){if(o in e)return e[o]();return n[o]},set(r,o,t){if(o in e)return e[o].set(t),!0;return console.warn(`[ship-set] hook handler tried to write s.${o} = … but ${o} is not a hook state key. The assignment was ignored.`),!0},has(r,o){return o in e||o in n},ownKeys(){return Object.keys(e)},getOwnPropertyDescriptor(r,o){if(o in e)return{configurable:!0,enumerable:!0,writable:!0,value:e[o]()};return}})}var Dr="__shipSetLifecycle";function Or(e,n){let r=Xo(e,n),o={ship:n.ship,route:n.route??{},navigate:n.navigate??((i)=>{}),...n.scope??{}},t=r.scope?r.scope(o,n.host):{};Object.assign(o,t);let a=Ro(o,n.stateKeys,t),b=qo(a,o),c={...o},f=Jo(r,c);Object.assign(c,f);let p=Wo(r,c,n);for(let[i,u]of Object.entries(p))c[i]=u;let d=Yo(r,b),l={...t,...f,...d,...p};if(Object.keys(t).length>0&&n.host){let i=Be(t);if(Object.keys(i).length>0){let u=n.host,s=u.tagName.toLowerCase();Fe(u,s,i)}}if(r.mount||r.unmount){let i={state:b};if(r.mount)i.mount=r.mount;if(r.unmount)i.unmount=r.unmount;Object.defineProperty(l,Dr,{value:i,enumerable:!0,configurable:!0,writable:!1})}return l}function Xo(e,n){if(typeof e==="function")return e({ship:n.ship});return e}function Ro(e,n,r){let o={},t=n!==void 0,a=r?Object.keys(r):[];if(t||a.length>0){if(n)for(let b of n){let c=e[b];if(Ke(c))o[b]=c}for(let b of a){if(b in o)continue;let c=e[b];if(Ke(c))o[b]=c}return o}for(let[b,c]of Object.entries(e))if(Ke(c))o[b]=c;return o}function Ke(e){return typeof e==="function"&&"set"in e&&typeof e.set==="function"}function qo(e,n){return new Proxy({},{get(r,o){if(o in e)return e[o]();return n[o]},set(r,o,t){if(o in e){if(typeof t==="function")e[o].set(()=>t);else e[o].set(t);return!0}return console.warn(`[ship-set] handler tried to write s.${o} = … but ${o} is not declared as <state name="${o}"> in the template. The assignment was ignored.`),!0},has(r,o){return o in e||o in n},ownKeys(){return Object.keys(e)},getOwnPropertyDescriptor(r,o){if(o in e)return{configurable:!0,enumerable:!0,writable:!0,value:e[o]()};return}})}function Jo(e,n){let r=e.derived??{},o={};for(let[t,a]of Object.entries(r))o[t]=J(()=>a(n));return o}function Yo(e,n){let r=e.on??{},o={};for(let[t,a]of Object.entries(r))o[t]=(...b)=>a(n,...b);return o}function Wo(e,n,r){if(e.use===void 0)return{};let o=typeof e.use==="function"?e.use(n):e.use,t={};for(let[a,b]of Object.entries(o))t[a]=Ve(b,r,n);return t}function O(e){return{setup:(n)=>Or(e,n),__def:e}}if(typeof customElements<"u"){for(let e of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(e))customElements.define(e,class extends HTMLElement{})}var Ko=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,Io=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function Tr(e){if(typeof CSSStyleSheet>"u")return null;try{let n=new CSSStyleSheet;return n.replaceSync(e),n}catch{return null}}var Mr=Tr(Ko),Pr=Tr(Io);function L(e){if(typeof ShadowRoot>"u")return;if(!(e instanceof ShadowRoot))return;let n=[Pr,Mr].filter((r)=>r!=null);if(n.length===0)return;try{e.adoptedStyleSheets=[...n,...e.adoptedStyleSheets??[]]}catch{}}var wu=j("sui-status-dot",{props:{tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"success",reflect:!0}),pulse:g.bool({default:!1,reflect:!0}),size:g.union(["sm","md"],{default:"md",reflect:!0}),label:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone-fg: var(--success);
  }
  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--tone-fg);
    box-shadow: 0 0 0 4px color-mix(in oklch, var(--tone-fg) 20%, transparent);
  }
  :host([size='sm']) .dot {
    width: 6px;
    height: 6px;
    box-shadow: 0 0 0 3px color-mix(in oklch, var(--tone-fg) 20%, transparent);
  }

  :host([pulse]) .dot { animation: sui-status-dot-pulse 2.5s infinite; }
  @keyframes sui-status-dot-pulse {
    0%, 100% { opacity: 1; transform: scale(1); }
    50%      { opacity: 0.75; transform: scale(0.9); }
  }

  @media (prefers-reduced-motion: reduce) {
    :host([pulse]) .dot { animation: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.pulse=n.pulse,t.size=n.size,t.label=n.label;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");return b.setAttribute("class","dot"),b.setAttribute("part","dot"),o.push(E(b,"role",()=>t.label()?"img":null)),o.push(E(b,"aria-label",()=>t.label()||null)),o.push(E(b,"aria-hidden",()=>t.label()?null:"true")),a.appendChild(b),t}});var Eu=j("sui-spinner",{props:{size:g.union(["xs","sm","md","lg"],{default:"md",reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),label:g.string({default:"Loading"})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone-fg: var(--muted-foreground);
    --sz: 16px;
    --bw: 2px;
  }
  :host([size='xs']) { --sz: 10px; --bw: 1.5px; }
  :host([size='sm']) { --sz: 12px; --bw: 1.5px; }
  :host([size='md']) { --sz: 16px; --bw: 2px; }
  :host([size='lg']) { --sz: 24px; --bw: 2.5px; }

  .ring {
    display: inline-block;
    width: var(--sz);
    height: var(--sz);
    border: var(--bw) solid var(--tone-fg);
    border-right-color: transparent;
    border-radius: 50%;
    animation: sui-spinner-spin 0.7s linear infinite;
  }
  @keyframes sui-spinner-spin { to { transform: rotate(360deg); } }
  @media (prefers-reduced-motion: reduce) { .ring { animation-duration: 1.5s; } }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.size=n.size,t.tone=n.tone,t.label=n.label;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");return b.setAttribute("class","ring"),b.setAttribute("part","ring"),o.push(E(b,"role",()=>t.label()?"status":null)),o.push(E(b,"aria-label",()=>t.label()||null)),o.push(E(b,"aria-hidden",()=>t.label()?null:"true")),a.appendChild(b),t}});var ju=j("sui-badge",{props:{tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:g.union(["solid","soft","outline"],{default:"soft",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),pill:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone: var(--foreground);
    --tone-fg: var(--foreground);
    --tone-contrast: var(--background);
    /* Size channel defaults */
    --bdg-h: 22px;
    --bdg-px: 8px;
    --bdg-fs: 10.5px;
    --bdg-gap: 4px;
    --bdg-icon: 10px;
    /* Shape channel */
    --bdg-radius: var(--radius-sm);
  }

  /* Size matrix */
  :host([size='sm']) { --bdg-h: 18px; --bdg-px: 6px;  --bdg-fs: 9.5px;  --bdg-gap: 3px; --bdg-icon: 8px; }
  :host([size='md']) { --bdg-h: 22px; --bdg-px: 8px;  --bdg-fs: 10.5px; --bdg-gap: 4px; --bdg-icon: 10px; }
  :host([size='lg']) { --bdg-h: 26px; --bdg-px: 10px; --bdg-fs: 11.5px; --bdg-gap: 5px; --bdg-icon: 12px; }

  /* Shape — square default, pill when opted-in */
  :host([pill]) { --bdg-radius: var(--radius-full); }

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
    /* soft is the default — base .badge renders as soft */
    background: color-mix(in oklch, var(--tone) 16%, transparent);
    border: 1px solid color-mix(in oklch, var(--tone) 24%, transparent);
    color: var(--tone-fg);
    white-space: nowrap;
    -webkit-font-smoothing: antialiased;
    font-variant-numeric: tabular-nums;
  }

  /* Variant matrix */
  :host([variant='soft']) .badge {
    background: color-mix(in oklch, var(--tone) 16%, transparent);
    border-color: color-mix(in oklch, var(--tone) 24%, transparent);
    color: var(--tone-fg);
  }
  :host([variant='solid']) .badge {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
  }
  :host([variant='outline']) .badge {
    background: transparent;
    border-color: color-mix(in oklch, var(--tone) 50%, transparent);
    color: var(--tone-fg);
  }

  /* Leading slot wrapper — hidden when empty */
  .leading {
    display: none;
    align-items: center;
    justify-content: center;
    width: var(--bdg-icon);
    height: var(--bdg-icon);
    flex-shrink: 0;
    color: currentColor;
  }
  .leading:has(::slotted(*)) { display: inline-flex; }
  ::slotted([slot='leading']) {
    display: inline-flex;
    align-items: center;
    color: currentColor;
    width: 100%;
    height: 100%;
  }

  @media (forced-colors: active) {
    .badge { border-color: CanvasText; color: CanvasText; }
    :host([variant='solid']) .badge { background: Highlight; color: HighlightText; border-color: Highlight; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.pill=n.pill;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");b.setAttribute("class","badge"),b.setAttribute("part","badge");let c=document.createElement("span");c.setAttribute("class","leading"),c.setAttribute("part","leading");let f=document.createElement("slot");f.setAttribute("name","leading"),c.appendChild(f),b.appendChild(c);let p=document.createElement("slot");return b.appendChild(p),a.appendChild(b),t}});var Qt=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","button");if(!n.hasAttribute("tabindex")&&!r.disabled())n.setAttribute("tabindex","0");let o=D(()=>{let b=r.selected(),c=r.disabled();if(n.setAttribute("aria-pressed",b?"true":"false"),c){if(n.setAttribute("aria-disabled","true"),n.getAttribute("tabindex")!=="-1")n.setAttribute("tabindex","-1")}else if(n.removeAttribute("aria-disabled"),n.getAttribute("tabindex")==="-1")n.setAttribute("tabindex","0")}),t=(b)=>{if(r.disabled())return;if(b.key==="Enter"||b.key===" ")b.preventDefault(),n.click();else if(r.removable()&&(b.key==="Backspace"||b.key==="Delete"))b.preventDefault(),n.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};n.addEventListener("keydown",t);let a=D(()=>{let b=r.removeBtn?.();if(!b)return;let c=(f)=>{if(f.stopPropagation(),r.disabled())return;n.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};return b.addEventListener("click",c),()=>b.removeEventListener("click",c)});return()=>{o(),n.removeEventListener("keydown",t),a()}}});var Pu=j("sui-chip",{props:{tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0}),variant:g.union(["solid","soft","outline","ghost"],{default:"soft",reflect:!0}),size:g.union(["sm","md","lg"],{default:"sm",reflect:!0}),selected:g.bool({default:!1,reflect:!0}),removable:g.bool({default:!1,reflect:!0}),disabled:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    vertical-align: middle;
    -webkit-tap-highlight-color: transparent;
    --tone: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
    --tone-contrast: var(--background);
    --chip-radius: var(--radius-full);
  }

  :host(:not([size])),
  :host([size='sm']) { --chip-h: 22px; --chip-px: 8px;  --chip-fs: var(--text-xs);   --chip-gap: 4px; --chip-icon: 10px; --chip-remove: 14px; }
  :host([size='md']) { --chip-h: 26px; --chip-px: 10px; --chip-fs: var(--text-sm);   --chip-gap: 6px; --chip-icon: 12px; --chip-remove: 16px; }
  :host([size='lg']) { --chip-h: 30px; --chip-px: 12px; --chip-fs: var(--text-base); --chip-gap: 6px; --chip-icon: 14px; --chip-remove: 18px; }

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
    color: var(--tone-fg);
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--chip-radius);
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      color var(--duration-fast) var(--ease-out);
  }

  :host(:not([variant])) .chip,
  :host([variant='soft']) .chip {
    background: color-mix(in oklch, var(--tone) 14%, transparent);
    border-color: color-mix(in oklch, var(--tone) 22%, transparent);
  }
  :host([variant='soft']:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 22%, transparent);
    border-color: color-mix(in oklch, var(--tone) 32%, transparent);
  }
  :host([variant='outline']) .chip {
    background: transparent;
    border-color: color-mix(in oklch, var(--tone) 45%, transparent);
  }
  :host([variant='outline']:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 10%, transparent);
  }
  :host([variant='solid']) .chip {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 18%, transparent);
  }
  :host([variant='ghost']) .chip {
    background: transparent;
    border-color: transparent;
  }
  :host([variant='ghost']:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 10%, transparent);
  }

  :host([selected]) .chip {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 10%);
    color: var(--tone-contrast);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 20%, transparent);
    font-weight: var(--font-weight-semibold);
  }
  :host([selected]:not([disabled])) .chip:hover {
    background: color-mix(in oklch, var(--tone) 92%, white 8%);
  }
  :host([selected]:not([tone])) .chip,
  :host([selected][tone='muted']) .chip {
    background: var(--foreground);
    border-color: var(--foreground);
    color: var(--background);
  }

  :host(:focus-visible) { outline: none; }
  :host(:focus-visible) .chip {
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--tone) var(--ring-opacity), transparent);
  }

  :host([disabled]) .chip {
    cursor: not-allowed;
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 80%, transparent);
    color: color-mix(in oklch, var(--muted-foreground) 70%, transparent);
    filter: saturate(0.65);
    box-shadow: none;
  }

  .leading {
    display: none;
    align-items: center;
    justify-content: center;
    width: var(--chip-icon);
    height: var(--chip-icon);
    flex-shrink: 0;
    color: currentColor;
  }
  .leading:has(::slotted(*)) { display: inline-flex; }
  ::slotted([slot='leading']) {
    display: inline-flex;
    align-items: center;
    color: currentColor;
    width: 100%;
    height: 100%;
  }

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
  :host([disabled]) .remove { cursor: not-allowed; }

  @media (forced-colors: active) {
    .chip { border-color: CanvasText; color: CanvasText; }
    :host([selected]) .chip { background: Highlight; color: HighlightText; border-color: Highlight; }
    :host(:focus-visible) .chip { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.selected=n.selected,t.removable=n.removable,t.disabled=n.disabled;let a=null;if(Qt&&typeof Qt.setup==="function"){if(e.scope=t,a=Qt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>H({anchor:u,test:()=>{if(t.removable())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("button");h.setAttribute("class","remove"),h.setAttribute("part","remove"),h.setAttribute("type","button"),h.setAttribute("aria-label","Remove"),h.setAttribute("tabindex","-1"),T(t,"removeBtn",h);let v=document.createElement("svg");v.setAttribute("viewbox","0 0 10 10"),v.setAttribute("aria-hidden","true");let y=document.createElement("path");return y.setAttribute("d","M2 2 L8 8 M8 2 L2 8"),v.appendChild(y),h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let w of m)try{w()}catch(k){}}}}]}),c=r.shadowRoot??r;L(c);let f=document.createElement("span");f.setAttribute("class","chip"),f.setAttribute("part","chip");let p=document.createElement("span");p.setAttribute("class","leading"),p.setAttribute("part","leading");let d=document.createElement("slot");d.setAttribute("name","leading"),p.appendChild(d),f.appendChild(p);let l=document.createElement("slot");f.appendChild(l);let i=document.createComment("if");if(f.appendChild(i),o.push(b(i)),c.appendChild(f),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(m,h){return m[h]},set(m,h,v){let y=m[h];if(y!=null&&typeof y.set==="function"){if(typeof v==="function")y.set(()=>v);else y.set(v);return!0}return!1}});if(u.mount){let m=u.mount(s,r);if(typeof m==="function")o.push(m)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var Vt=O({mount(e,n){let r=e,o=(a)=>{if(a.key==="Enter"||a.key===" ")a.preventDefault(),n.click()},t=D(()=>{if(r.interactive()){if(!n.hasAttribute("role"))n.setAttribute("role","button");if(!n.hasAttribute("tabindex"))n.setAttribute("tabindex","0");return n.addEventListener("keydown",o),()=>n.removeEventListener("keydown",o)}else{if(n.getAttribute("role")==="button")n.removeAttribute("role");if(n.getAttribute("tabindex")==="0")n.removeAttribute("tabindex")}});return()=>{t()}}});var Gu=j("sui-card",{props:{tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:g.union(["default","outline"],{default:"default",reflect:!0}),padding:g.union(["none","sm","md","lg"],{default:"md",reflect:!0}),interactive:g.bool({default:!1,reflect:!0}),"accent-bar":g.bool({default:!1,reflect:!0}),"status-rail":g.bool({default:!1,reflect:!0}),fill:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    position: relative;
    background: var(--card);
    color: var(--card-foreground, var(--foreground));
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-card-rest, var(--shadow-xs));
    --card-px: 20px;
    --card-py: 16px;
    --card-gap: var(--space-3, 12px);
    --tone-fg: var(--foreground);
    --tone-border: var(--border);
    --tone-bg: transparent;
    padding: var(--card-py) var(--card-px);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  :host([tone='primary'])     { --tone-border: color-mix(in oklch, var(--primary) 40%, var(--border));     --tone-bg: color-mix(in oklch, var(--primary) 8%, var(--card)); }
  :host([tone='info'])        { --tone-border: color-mix(in oklch, var(--info) 40%, var(--border));        --tone-bg: color-mix(in oklch, var(--info) 8%, var(--card)); }
  :host([tone='success'])     { --tone-border: color-mix(in oklch, var(--success) 40%, var(--border));     --tone-bg: color-mix(in oklch, var(--success) 8%, var(--card)); }
  :host([tone='warning'])     { --tone-border: color-mix(in oklch, var(--warning) 40%, var(--border));     --tone-bg: color-mix(in oklch, var(--warning) 8%, var(--card)); }
  :host([tone='destructive']) { --tone-border: color-mix(in oklch, var(--destructive) 40%, var(--border)); --tone-bg: color-mix(in oklch, var(--destructive) 8%, var(--card)); }
  :host([tone='accent'])      { --tone-border: color-mix(in oklch, var(--accent) 40%, var(--border));      --tone-bg: color-mix(in oklch, var(--accent) 8%, var(--card)); }
  :host([tone='muted'])       { --tone-border: var(--border); --tone-bg: var(--muted); }

  :host([tone]:not([tone='default'])) { border-color: var(--tone-border); }
  :host([fill]) { background: var(--tone-bg); border-color: var(--tone-border); }

  :host([padding='none']) { --card-px: 0px; --card-py: 0px; --card-gap: 0px; padding: 0; }
  :host([padding='sm'])   { --card-px: 12px; --card-py: 10px; --card-gap: 8px; padding: 10px 12px; }
  :host([padding='md'])   { --card-px: 20px; --card-py: 16px; padding: 16px 20px; }
  :host([padding='lg'])   { --card-px: 28px; --card-py: 24px; --card-gap: 16px; padding: 24px 28px; }

  :host([variant='outline']) { background: transparent; box-shadow: none; }

  :host([status-rail]) {
    border-left: 3px solid var(--tone-fg);
    padding-left: calc(var(--card-px) - 3px);
  }

  :host([accent-bar])::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 2px;
    background: linear-gradient(90deg, var(--primary), var(--accent));
    opacity: 0;
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    transition: opacity var(--duration-medium) var(--ease-out);
    pointer-events: none;
  }
  :host([accent-bar]:hover)::before { opacity: 1; }

  :host([interactive]) { cursor: pointer; -webkit-tap-highlight-color: transparent; }
  :host([interactive]:hover) {
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-hover, var(--shadow-sm));
    transform: translateY(-1px);
  }
  @media (prefers-reduced-motion: reduce) {
    :host([interactive]:hover) { transform: none; }
  }
  :host([interactive]:active) {
    box-shadow: var(--shadow-card-rest, var(--shadow-xs));
    transform: translateY(0);
    transition-duration: 60ms;
  }
  :host([interactive]:focus-visible) {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }

  .header, .footer { display: block; }

  /* Separator — show only when slot has projected content */
  :host(:not([data-no-header])) .header {
    margin: calc(var(--card-py) * -1) calc(var(--card-px) * -1) var(--card-gap);
    padding: var(--card-py) var(--card-px);
    border-bottom: 1px solid var(--border);
    font-family: var(--heading-font);
    font-weight: var(--heading-weight, 600);
  }
  :host(:not([data-no-footer])) .footer {
    margin: var(--card-gap) calc(var(--card-px) * -1) calc(var(--card-py) * -1);
    padding: var(--card-py) var(--card-px);
    border-top: 1px solid var(--border);
  }
  :host([padding='none']:not([data-no-header])) .header {
    margin: 0 0 var(--space-3, 12px);
    padding: 0 0 var(--space-3, 12px);
  }
  :host([padding='none']:not([data-no-footer])) .footer {
    margin: var(--space-3, 12px) 0 0;
    padding: var(--space-3, 12px) 0 0;
  }

  @media (forced-colors: active) {
    :host([interactive]:focus-visible) { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.padding=n.padding,t.interactive=n.interactive,t.accentBar=n.accentBar,t.statusRail=n.statusRail,t.fill=n.fill;let a=null;if(Vt&&typeof Vt.setup==="function"){if(e.scope=t,a=Vt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","header"),c.setAttribute("part","header");let f=document.createElement("slot");f.setAttribute("name","header"),o.push(tt(f,r,"data-no-header")),c.appendChild(f),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","content"),p.setAttribute("part","content");let d=document.createElement("slot");p.appendChild(d),b.appendChild(p);let l=document.createElement("div");l.setAttribute("class","footer"),l.setAttribute("part","footer");let i=document.createElement("slot");if(i.setAttribute("name","footer"),o.push(tt(i,r,"data-no-footer")),l.appendChild(i),b.appendChild(l),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(m,h){return m[h]},set(m,h,v){let y=m[h];if(y!=null&&typeof y.set==="function"){if(typeof v==="function")y.set(()=>v);else y.set(v);return!0}return!1}});if(u.mount){let m=u.mount(s,r);if(typeof m==="function")o.push(m)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var Ru=j("sui-link",{props:{href:g.string({default:""}),external:g.bool({default:!1,reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:g.union(["default","underline","subtle"],{default:"default",reflect:!0}),target:g.string({default:""}),rel:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline;
    font-family: var(--font-body);
    --link-fg: var(--primary);
  }

  :host([tone='default'])     { --link-fg: var(--primary); }
  :host([tone='primary'])     { --link-fg: var(--primary); }
  :host([tone='info'])        { --link-fg: var(--info); }
  :host([tone='success'])     { --link-fg: var(--success); }
  :host([tone='warning'])     { --link-fg: var(--warning); }
  :host([tone='destructive']) { --link-fg: var(--destructive); }
  :host([tone='accent'])      { --link-fg: var(--accent); }
  :host([tone='muted'])       { --link-fg: var(--muted-foreground); }

  a {
    color: var(--link-fg);
    text-decoration: none;
    display: inline;
    border-radius: var(--radius-sm);
    transition:
      color var(--duration-fast),
      text-decoration-color var(--duration-fast);
  }
  a:hover {
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  :host([variant='underline']) a {
    text-decoration: underline;
    text-decoration-thickness: 1px;
    text-underline-offset: 2px;
    text-decoration-color: color-mix(in oklch, var(--link-fg) 40%, transparent);
  }
  :host([variant='subtle']) a:hover {
    color: color-mix(in oklch, var(--link-fg) 80%, var(--foreground));
  }
  a:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: var(--ring-offset);
  }
  .ext {
    display: inline-block;
    margin-left: 2px;
    font-size: 0.9em;
    opacity: 0.8;
    vertical-align: -0.1em;
  }
  .ext svg {
    width: 0.9em;
    height: 0.9em;
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.href=n.href,t.external=n.external,t.tone=n.tone,t.variant=n.variant,t.target=n.target,t.rel=n.rel;let a=(d)=>H({anchor:d,test:()=>{if(t.external())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("span");u.setAttribute("class","sr-only");let s=document.createTextNode("(opens in new tab)");u.appendChild(s),l.appendChild(u);let m=document.createElement("span");m.setAttribute("class","ext"),m.setAttribute("aria-hidden","true");let h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round");let v=document.createElement("path");return v.setAttribute("d","M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3"),h.appendChild(v),m.appendChild(h),l.appendChild(m),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let y of i)try{y()}catch(w){}}}}]}),b=r.shadowRoot??r;L(b);let c=document.createElement("a");c.setAttribute("part","link"),o.push(E(c,"href",()=>t.href())),o.push(E(c,"target",()=>t.external()?"_blank":t.target())),o.push(E(c,"rel",()=>t.external()?"noopener noreferrer":t.rel()));let f=document.createElement("slot");c.appendChild(f);let p=document.createComment("if");return c.appendChild(p),o.push(a(p)),b.appendChild(c),t}});var Yu=j("sui-divider",{props:{orientation:g.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:g.union(["solid","dashed"],{default:"solid",reflect:!0}),tone:g.union(["default","muted","primary","destructive"],{default:"default",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    --tone-bg: var(--border);
  }
  :host([tone='muted'])       { --tone-bg: color-mix(in oklch, var(--border) 50%, transparent); }
  :host([tone='primary'])     { --tone-bg: color-mix(in oklch, var(--primary) 50%, var(--border)); }
  :host([tone='destructive']) { --tone-bg: color-mix(in oklch, var(--destructive) 50%, var(--border)); }

  .line {
    border: 0;
  }

  /* Horizontal — full-width 1px line. */
  :host(:not([orientation='vertical'])) .line {
    width: 100%;
    height: 1px;
    margin: var(--space-2) 0;
    background: var(--tone-bg);
  }
  :host(:not([orientation='vertical'])[variant='dashed']) .line {
    background: transparent;
    border-top: 1px dashed var(--tone-bg);
    height: 0;
  }

  /* Vertical — inline 1px line that stretches to parent height. */
  :host([orientation='vertical']) {
    display: inline-block;
    vertical-align: middle;
    align-self: stretch;
  }
  :host([orientation='vertical']) .line {
    width: 1px;
    height: 1em;
    margin: 0 var(--space-2);
    display: inline-block;
    background: var(--tone-bg);
  }
  :host([orientation='vertical'][variant='dashed']) .line {
    background: transparent;
    border-left: 1px dashed var(--tone-bg);
    width: 0;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.orientation=n.orientation,t.variant=n.variant,t.tone=n.tone;let a=r.shadowRoot??r;L(a);let b=document.createElement("hr");return b.setAttribute("class","line"),b.setAttribute("part","line"),b.setAttribute("role","separator"),o.push(E(b,"aria-orientation",()=>t.orientation())),a.appendChild(b),t}});var Qu=j("sui-skeleton",{props:{shape:g.union(["text","line","block","circle"],{default:"text",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
  :host { display: inline-block; vertical-align: middle; }
  :host([shape='block']) { display: block; }

  .sk {
    display: block;
    background: linear-gradient(
      90deg,
      var(--muted) 0%,
      color-mix(in oklch, var(--muted) 75%, var(--background)) 50%,
      var(--muted) 100%
    );
    background-size: 200% 100%;
    animation: sui-skeleton-shimmer 1.5s ease-in-out infinite;
    border-radius: var(--radius-sm);
  }
  :host([shape='circle']) .sk { border-radius: 50%; }

  /* Text / line — narrow heights, full-width default. */
  :host([shape='text']) .sk,
  :host([shape='line']) .sk { width: 100%; }
  :host([size='sm'][shape='text']) .sk,
  :host([size='sm'][shape='line']) .sk { height: 8px; }
  :host([size='md'][shape='text']) .sk,
  :host([size='md'][shape='line']) .sk { height: 12px; }
  :host([size='lg'][shape='text']) .sk,
  :host([size='lg'][shape='line']) .sk { height: 16px; }

  /* Block — fills parent width with a fixed min-height. */
  :host([shape='block']) .sk { width: 100%; min-height: 80px; }

  /* Circle — fixed diameters per size. */
  :host([shape='circle'][size='sm']) .sk { width: 24px; height: 24px; }
  :host([shape='circle'][size='md']) .sk { width: 32px; height: 32px; }
  :host([shape='circle'][size='lg']) .sk { width: 48px; height: 48px; }

  @keyframes sui-skeleton-shimmer {
    0%   { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
  @media (prefers-reduced-motion: reduce) {
    .sk { animation: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.shape=n.shape,t.size=n.size;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");return b.setAttribute("class","sk"),b.setAttribute("part","sk"),b.setAttribute("role","status"),b.setAttribute("aria-label","Loading"),b.setAttribute("aria-live","polite"),b.setAttribute("aria-busy","true"),a.appendChild(b),t}});var Kt=O({mount(e,n){let r=e,o=(a)=>{if(a.key==="Enter"||a.key===" ")a.preventDefault(),n.click()},t=D(()=>{if(r.interactive()){if(!n.hasAttribute("role"))n.setAttribute("role","button");if(!n.hasAttribute("tabindex"))n.setAttribute("tabindex","0");return n.addEventListener("keydown",o),()=>n.removeEventListener("keydown",o)}else{if(n.getAttribute("role")==="button")n.removeAttribute("role");if(n.getAttribute("tabindex")==="0")n.removeAttribute("tabindex")}});return()=>{t(),n.removeEventListener("keydown",o)}}});var tl=j("sui-stat",{props:{tone:g.union(["default","primary","info","success","warning","destructive","accent"],{default:"default",reflect:!0}),interactive:g.bool({default:!1,reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 16px 18px;
    font-family: var(--font-body);
    transition:
      border-color var(--duration-medium) var(--ease-out),
      transform var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
    --tone-fg: var(--foreground);
  }

  :host([interactive]) { cursor: pointer; }
  :host([interactive]:hover) {
    border-color: color-mix(in oklch, var(--border) 180%, transparent);
    transform: translateY(-1px);
    box-shadow: var(--shadow-sm);
  }
  @media (prefers-reduced-motion: reduce) {
    :host([interactive]:hover) { transform: none; }
  }

  .label {
    font-size: var(--text-xs);
    font-weight: var(--font-weight-medium);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .value-row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    flex-wrap: wrap;
  }
  .value {
    font-family: var(--heading-font, var(--font-body));
    font-weight: var(--font-weight-bold);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
    color: var(--tone-fg);
  }
  :host([size='sm']) .value { font-size: var(--text-xl, 20px); }
  :host([size='md']) .value { font-size: var(--text-3xl, 30px); }
  :host([size='lg']) .value { font-size: var(--text-4xl, 36px); }

  .trend {
    display: inline-flex;
    align-items: center;
  }
  .hint {
    font-size: 11px;
    color: var(--muted-foreground);
  }
  ::slotted([slot='trend']) { font-size: var(--text-sm); }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.interactive=n.interactive,t.size=n.size;let a=null;if(Kt&&typeof Kt.setup==="function"){if(e.scope=t,a=Kt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("span");c.setAttribute("class","label"),c.setAttribute("part","label");let f=document.createElement("slot");f.setAttribute("name","label"),c.appendChild(f),b.appendChild(c);let p=document.createElement("span");p.setAttribute("class","value-row"),p.setAttribute("part","value-row");let d=document.createElement("span");d.setAttribute("class","value"),d.setAttribute("part","value");let l=document.createElement("slot");d.appendChild(l),p.appendChild(d);let i=document.createElement("span");i.setAttribute("class","trend"),i.setAttribute("part","trend");let u=document.createElement("slot");u.setAttribute("name","trend"),i.appendChild(u),p.appendChild(i),b.appendChild(p);let s=document.createElement("span");s.setAttribute("class","hint"),s.setAttribute("part","hint");let m=document.createElement("slot");if(m.setAttribute("name","hint"),s.appendChild(m),b.appendChild(s),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(y,w){return y[w]},set(y,w,k){let A=y[w];if(A!=null&&typeof A.set==="function"){if(typeof k==="function")A.set(()=>k);else A.set(k);return!0}return!1}});if(h.mount){let y=h.mount(v,r);if(typeof y==="function")o.push(y)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var rl=j("sui-mark",{props:{tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"warning",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline;
    --tone: var(--warning);
    --tone-fg: var(--warning);
  }
  mark {
    background: color-mix(in oklch, var(--tone) 20%, transparent);
    color: var(--tone-fg);
    padding: 0 2px;
    border-radius: 2px;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone;let a=r.shadowRoot??r;L(a);let b=document.createElement("mark");b.setAttribute("part","mark");let c=document.createElement("slot");return b.appendChild(c),a.appendChild(b),t}});var It=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","group");let o=D(()=>{let t=r.keys();if(t)n.setAttribute("aria-label",`Keyboard shortcut: ${t}`);else n.removeAttribute("aria-label")});return()=>o()}});var ll=j("sui-kbd",{props:{keys:g.string({default:""}),size:g.union(["sm","md"],{default:"md",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    align-items: center;
    vertical-align: baseline;
  }

  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    padding: 2px 5px;
    font-family: var(--font-mono);
    font-weight: var(--font-weight-medium, 500);
    font-size: 11px;
    line-height: 1.2;
    color: var(--foreground);
    background: var(--muted);
    border: 1px solid var(--border);
    border-bottom-width: 2px;
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-xs);
    white-space: nowrap;
  }

  :host([size='sm']) kbd { font-size: 10px; padding: 1px 4px; min-width: 16px; }
  :host([size='md']) kbd { font-size: 11px; padding: 2px 5px; min-width: 18px; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.keys=n.keys,t.size=n.size;let a=null;if(It&&typeof It.setup==="function"){if(e.scope=t,a=It.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("kbd");c.setAttribute("part","kbd");let f=document.createElement("slot");if(c.appendChild(f),b.appendChild(c),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var pl=j("sui-avatar",{props:{src:g.string({default:""}),name:g.string({default:""}),initials:g.string({default:""}),"icon-slug":g.string({default:""}),size:g.union(["xs","sm","md","lg","xl"],{default:"md",reflect:!0}),shape:g.union(["round","square"],{default:"round",reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    vertical-align: middle;
    --sz: 32px;
    --fs: 13px;
    --av-bg: linear-gradient(
      135deg,
      color-mix(in oklch, var(--primary) 70%, transparent),
      color-mix(in oklch, var(--accent) 70%, transparent)
    );
    --av-fg: var(--foreground);
  }

  :host([size='xs']) { --sz: 20px; --fs: 10px; }
  :host([size='sm']) { --sz: 28px; --fs: 11px; }
  :host([size='md']) { --sz: 32px; --fs: 13px; }
  :host([size='lg']) { --sz: 40px; --fs: 15px; }
  :host([size='xl']) { --sz: 56px; --fs: 22px; }

  /* Tone channel for initials background */
  :host([tone='primary'])     { --av-bg: var(--primary);     --av-fg: var(--primary-foreground); }
  :host([tone='accent'])      { --av-bg: var(--accent);      --av-fg: var(--accent-foreground); }
  :host([tone='info'])        { --av-bg: color-mix(in oklch, var(--info) 20%, var(--muted));        --av-fg: var(--info); }
  :host([tone='success'])     { --av-bg: color-mix(in oklch, var(--success) 20%, var(--muted));     --av-fg: var(--success); }
  :host([tone='warning'])     { --av-bg: color-mix(in oklch, var(--warning) 20%, var(--muted));     --av-fg: var(--warning); }
  :host([tone='destructive']) { --av-bg: color-mix(in oklch, var(--destructive) 20%, var(--muted)); --av-fg: var(--destructive); }
  :host([tone='muted'])       { --av-bg: linear-gradient(135deg, color-mix(in oklch, var(--primary) 70%, transparent), color-mix(in oklch, var(--accent) 70%, transparent)); --av-fg: var(--foreground); }
  :host([tone='default'])     { --av-bg: linear-gradient(135deg, color-mix(in oklch, var(--primary) 70%, transparent), color-mix(in oklch, var(--accent) 70%, transparent)); --av-fg: var(--foreground); }

  .root {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--sz);
    height: var(--sz);
    flex: none;
    background: var(--av-bg);
    color: var(--av-fg);
    font-family: var(--font-body);
    font-weight: var(--font-weight-bold);
    font-size: var(--fs);
    letter-spacing: 0.02em;
    text-transform: uppercase;
    line-height: 1;
    overflow: hidden;
    user-select: none;
  }
  :host([shape='round'])  .root { border-radius: 50%; }
  :host([shape='square']) .root { border-radius: var(--radius-sm); }

  img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
    padding: 10%;
    box-sizing: border-box;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.src=n.src,t.name=n.name,t.initials=n.initials,t.iconSlug=n.iconSlug,t.size=n.size,t.shape=n.shape,t.tone=n.tone;let a=(p)=>H({anchor:p,test:()=>{if(t.iconSlug()||t.src())return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("img");return i.setAttribute("part","img"),l.push(E(i,"src",()=>t.iconSlug()?"https://cdn.simpleicons.org/"+t.iconSlug():t.src())),l.push(E(i,"alt",()=>t.name())),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let u of l)try{u()}catch(s){}}}},()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("span");i.setAttribute("part","initials");let u=document.createTextNode("");return i.appendChild(u),l.push(F(u,()=>t.initials()?t.initials().slice(0,2).toUpperCase():t.name().trim().slice(0,1).toUpperCase()||"?")),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let s of l)try{s()}catch(m){}}}}]}),b=r.shadowRoot??r;L(b);let c=document.createElement("span");c.setAttribute("class","root"),c.setAttribute("part","root"),c.setAttribute("role","img"),o.push(E(c,"aria-label",()=>t.name()||t.initials()||"avatar"));let f=document.createComment("if");return c.appendChild(f),o.push(a(f)),b.appendChild(c),t}});var bl=j("sui-progress",{props:{value:g.number({default:0}),max:g.number({default:100}),tone:g.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),indeterminate:g.bool({default:!1,reflect:!0}),label:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: block;
    --tone: var(--primary);
    --h: 6px;
  }

  :host([size='sm']) { --h: 4px; }
  :host([size='md']) { --h: 6px; }
  :host([size='lg']) { --h: 10px; }

  .track {
    width: 100%;
    height: var(--h);
    background: var(--muted);
    border-radius: var(--radius-full);
    overflow: hidden;
    position: relative;
  }

  .fill {
    height: 100%;
    background: var(--tone);
    border-radius: var(--radius-full);
    transition: width var(--duration-fast) var(--ease-out);
  }

  :host([indeterminate]) .fill {
    width: 30%;
    animation: sui-progress-indet 1.4s ease-in-out infinite;
  }
  @keyframes sui-progress-indet {
    0%   { transform: translateX(-100%); }
    50%  { transform: translateX(150%); }
    100% { transform: translateX(350%); }
  }
  @media (prefers-reduced-motion: reduce) {
    :host([indeterminate]) .fill { animation-duration: 3s; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.max=n.max,t.tone=n.tone,t.size=n.size,t.indeterminate=n.indeterminate,t.label=n.label;let a=r.shadowRoot??r;L(a);let b=document.createElement("div");b.setAttribute("class","track"),b.setAttribute("part","track"),b.setAttribute("role","progressbar"),o.push(E(b,"aria-valuenow",()=>t.indeterminate()?null:t.value())),b.setAttribute("aria-valuemin","0"),o.push(E(b,"aria-valuemax",()=>t.max())),o.push(E(b,"aria-busy",()=>t.indeterminate()?"true":"false")),o.push(E(b,"aria-label",()=>t.label()));let c=document.createElement("div");return c.setAttribute("class","fill"),c.setAttribute("part","fill"),o.push(E(c,"style",()=>t.indeterminate()?"":"width: clamp(0%, "+t.value()/(t.max()||1)*100+"%, 100%)")),b.appendChild(c),a.appendChild(b),t}});var gl=j("sui-progress-ring",{props:{value:g.number({default:0}),max:g.number({default:100}),size:g.number({default:48}),"track-width":g.number({default:4}),tone:g.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),indeterminate:g.bool({default:!1,reflect:!0}),label:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline-block;
    --tone: var(--primary);
  }

  .ring {
    position: relative;
  }

  .ring-svg {
    display: block;
    width: 100%;
    height: 100%;
    transform: rotate(-90deg);
  }

  :host([indeterminate]) .ring-svg {
    animation: sui-ring-spin 1.4s linear infinite;
  }
  @keyframes sui-ring-spin {
    from { transform: rotate(-90deg); }
    to   { transform: rotate(270deg); }
  }

  .track { stroke: var(--muted); }

  .bar {
    stroke: var(--tone);
    transition: stroke-dashoffset var(--duration-medium, 220ms) var(--ease-out, ease-out);
  }

  .label {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: var(--text-xs);
    font-weight: var(--font-weight-semibold);
    font-family: var(--font-body);
    color: var(--foreground);
    font-variant-numeric: tabular-nums;
    pointer-events: none;
  }

  @media (prefers-reduced-motion: reduce) {
    :host([indeterminate]) .ring-svg { animation-duration: 3s; }
    .bar { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.max=n.max,t.size=n.size,t.trackWidth=n.trackWidth,t.tone=n.tone,t.indeterminate=n.indeterminate,t.label=n.label;let a=r.shadowRoot??r;L(a);let b=document.createElement("div");b.setAttribute("class","ring"),o.push(E(b,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let c=document.createElement("svg");c.setAttribute("class","ring-svg"),o.push(E(c,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),c.setAttribute("role","progressbar"),c.setAttribute("aria-valuemin","0"),o.push(E(c,"aria-valuemax",()=>t.max())),o.push(E(c,"aria-valuenow",()=>t.indeterminate()?null:t.value()<0?0:t.value()>t.max()?t.max():t.value())),o.push(E(c,"aria-label",()=>t.label()||null)),c.setAttribute("xmlns","http://www.w3.org/2000/svg");let f=document.createElement("circle");f.setAttribute("class","track"),f.setAttribute("part","track"),o.push(E(f,"cx",()=>t.size()/2)),o.push(E(f,"cy",()=>t.size()/2)),o.push(E(f,"r",()=>(t.size()-t.trackWidth())/2)),f.setAttribute("fill","none"),o.push(E(f,"stroke-width",()=>t.trackWidth())),c.appendChild(f);let p=document.createElement("circle");p.setAttribute("class","bar"),p.setAttribute("part","ring"),o.push(E(p,"cx",()=>t.size()/2)),o.push(E(p,"cy",()=>t.size()/2)),o.push(E(p,"r",()=>(t.size()-t.trackWidth())/2)),p.setAttribute("fill","none"),o.push(E(p,"stroke-width",()=>t.trackWidth())),p.setAttribute("stroke-linecap","round"),o.push(E(p,"stroke-dasharray",()=>3.141592653589793*(t.size()-t.trackWidth()))),o.push(E(p,"stroke-dashoffset",()=>t.indeterminate()?3.141592653589793*(t.size()-t.trackWidth())*0.25:3.141592653589793*(t.size()-t.trackWidth())*(1-(t.value()<0?0:t.value()>(t.max()||1)?1:t.value()/(t.max()||1))))),c.appendChild(p),b.appendChild(c);let d=document.createElement("div");d.setAttribute("class","label"),d.setAttribute("aria-hidden","true");let l=document.createElement("slot");return d.appendChild(l),b.appendChild(d),a.appendChild(b),t}});var Ie={default:"var(--foreground)",primary:"var(--primary)",info:"var(--info)",success:"var(--success)",warning:"var(--warning)",destructive:"var(--destructive)",accent:"var(--accent)",muted:"var(--muted-foreground)"};function K(e){if(!e)return Ie.default;return Ie[e]??Ie.default}function kt(e){if(!e)return[];return e.split(",").map((n)=>Number.parseFloat(n.trim())).filter((n)=>Number.isFinite(n))}function Nt(e){if(!e)return[];return e.split(",").map((n)=>n.trim())}function rt(e,n){if(!e)return n;try{return JSON.parse(e)}catch{return n}}function Et(e,n,r,o=2){if(e.length===0)return{d:"",points:[]};let t=Math.min(...e),b=Math.max(...e)-t||1,c=r-o*2,f=e.length===1?0:n/(e.length-1),p=e.map((l,i)=>{let u=e.length===1?n/2:i*f,s=o+c-(l-t)/b*c;return[u,s]});return{d:p.map(([l,i],u)=>`${u===0?"M":"L"}${l.toFixed(2)},${i.toFixed(2)}`).join(" "),points:p}}function Nr(e,n,r,o){return[e+r*Math.cos(o),n+r*Math.sin(o)]}function Ht(e,n,r,o,t){let[a,b]=Nr(e,n,r,o),[c,f]=Nr(e,n,r,t),p=t-o>Math.PI?1:0,d=t>o?1:0;return`M${a.toFixed(2)},${b.toFixed(2)} A${r},${r} 0 ${p} ${d} ${c.toFixed(2)},${f.toFixed(2)}`}var $t=O({scope:()=>{let e=P([]),n=P([]);return{seriesPaths:e,gridLines:n}},mount(e,n){let r=e,o=r.seriesPaths,t=r.gridLines,a=D(()=>{let b=rt(r.series(),[]),c=r.width(),f=r.height(),p=b.flatMap((m)=>m.data),d=p.length?Math.min(...p):0,i=(p.length?Math.max(...p):1)-d||1,u=f-4;t.set(r.grid()?[0.25,0.5,0.75].map((m,h)=>({id:`g${h}`,y:2+u*m})):[]);let s=[];for(let m of b){if(m.data.length<2)continue;let h=m.data.map((C)=>(C-d)/i*100),{d:v,points:y}=Et(h,c,f,2);if(!v)continue;let w=K(m.tone),k=y[0],A=y[y.length-1],x=`${v} L${A[0].toFixed(2)},${f} L${k[0].toFixed(2)},${f} Z`;s.push({name:m.name,areaD:x,strokeD:v,color:w,pts:y.map(([C,z],M)=>({id:`${m.name}-${M}`,cx:C,cy:z}))})}o.set(s)});return()=>a()}});var zl=j("sui-area-chart",{props:{series:g.string({default:"",reflect:!0}),labels:g.string({default:""}),width:g.number({default:300}),height:g.number({default:200}),points:g.bool({default:!1,reflect:!0}),grid:g.bool({default:!1,reflect:!0}),label:g.string({default:"area chart"})},shadow:!0,styles:`
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

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.series=n.series,t.labels=n.labels,t.width=n.width,t.height=n.height,t.points=n.points,t.grid=n.grid,t.label=n.label;let a=null;if($t&&typeof $t.setup==="function"){if(e.scope=t,a=$t.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(m)=>X({anchor:m,items:()=>t.gridLines(),key:(h,v)=>{let y=Object.create(t);return y.g=h,y.__i=v,((w)=>h.id)(y)},render:(h,v)=>{let y=Object.create(t);return y.g=h,y.__i=v,y.__v=P(0),((w)=>{let k=document.createDocumentFragment(),A=[],x=document.createElement("line");return x.setAttribute("class","grid-line"),x.setAttribute("x1","0"),A.push(E(x,"x2",()=>(w.__v(),w.width()))),A.push(E(x,"y1",()=>(w.__v(),w.g.y))),A.push(E(x,"y2",()=>(w.__v(),w.g.y))),k.appendChild(x),{nodes:Array.from(k.childNodes),cleanup:()=>{for(let C of A)try{C()}catch(z){}},update:(C,z)=>{w.g=C,w.__i=z,w.__v.set((M)=>M+1)}}})(y)}}),c=(m)=>X({anchor:m,items:()=>t.seriesPaths(),key:(h,v)=>{let y=Object.create(t);return y.s=h,y.__i=v,((w)=>h.name)(y)},render:(h,v)=>{let y=Object.create(t);return y.s=h,y.__i=v,y.__v=P(0),((w)=>{let k=(S)=>X({anchor:S,items:()=>w.s.pts,key:(B,U)=>{let G=Object.create(w);return G.p=B,G.__i=U,((q)=>B.id)(G)},render:(B,U)=>{let G=Object.create(w);return G.p=B,G.__i=U,G.__v=P(0),((q)=>{let Q=document.createDocumentFragment(),Z=[],R=document.createElement("circle");return Z.push(E(R,"cx",()=>(q.__v(),q.p.cx))),Z.push(E(R,"cy",()=>(q.__v(),q.p.cy))),R.setAttribute("r","2.5"),Z.push(E(R,"fill",()=>(q.__v(),q.s.color))),Q.appendChild(R),{nodes:Array.from(Q.childNodes),cleanup:()=>{for(let at of Z)try{at()}catch(I){}},update:(at,I)=>{q.p=at,q.__i=I,q.__v.set((_)=>_+1)}}})(G)}}),A=(S)=>H({anchor:S,test:()=>{if(w.points())return 0;return-1},branches:[()=>{let B=document.createDocumentFragment(),U=[],G=document.createComment("for");return B.appendChild(G),U.push(k(G)),{nodes:Array.from(B.childNodes),cleanup:()=>{for(let q of U)try{q()}catch(Q){}}}}]}),x=document.createDocumentFragment(),C=[],z=document.createElement("path");C.push(E(z,"d",()=>(w.__v(),w.s.areaD))),C.push(E(z,"fill",()=>(w.__v(),w.s.color))),z.setAttribute("fill-opacity","0.3"),z.setAttribute("stroke","none"),x.appendChild(z);let M=document.createElement("path");C.push(E(M,"d",()=>(w.__v(),w.s.strokeD))),M.setAttribute("fill","none"),C.push(E(M,"stroke",()=>(w.__v(),w.s.color))),M.setAttribute("stroke-width","1.75"),M.setAttribute("stroke-linecap","round"),M.setAttribute("stroke-linejoin","round"),x.appendChild(M);let N=document.createComment("if");return x.appendChild(N),C.push(A(N)),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let S of C)try{S()}catch(B){}},update:(S,B)=>{w.s=S,w.__i=B,w.__v.set((U)=>U+1)}}})(y)}}),f=r.shadowRoot??r;L(f);let p=document.createElement("div");p.setAttribute("class","wrap");let d=document.createElement("svg");o.push(E(d,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),d.setAttribute("preserveaspectratio","none"),d.setAttribute("role","img"),o.push(E(d,"aria-label",()=>t.label()));let l=document.createComment("for");d.appendChild(l),o.push(b(l));let i=document.createComment("for");d.appendChild(i),o.push(c(i)),p.appendChild(d);let u=document.createElement("div");u.setAttribute("class","legend");let s=document.createElement("slot");if(u.appendChild(s),p.appendChild(u),f.appendChild(p),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let h=new Proxy(t,{get(v,y){return v[y]},set(v,y,w){let k=v[y];if(k!=null&&typeof k.set==="function"){if(typeof w==="function")k.set(()=>w);else k.set(w);return!0}return!1}});if(m.mount){let v=m.mount(h,r);if(typeof v==="function")o.push(v)}if(m.unmount)o.push(()=>m.unmount(h))}}return t}});var _t=O({scope:()=>{let e=P([]),n=P(!0);return{bars:e,isEmpty:n}},mount(e,n){let r=e,o=r.bars,t=r.isEmpty,a=D(()=>{let b=kt(r.data());if(b.length===0){o.set([]),t.set(!0);return}let c=Nt(r.labels()),f=r.width(),p=r.height(),d=r.max(),l=d>0?d:Math.max(...b,0),i=l<=0?1:l,u=K(r.tone()),s=4,m=3,h=r.orientation()==="horizontal",v=b.length;o.set(b.map((y,w)=>{let k=Math.max(0,y)/i,A,x,C,z;if(h)z=(p-s*(v-1))/v,x=w*(z+s),C=k*f,A=0;else C=(f-s*(v-1))/v,A=w*(C+s),z=k*p,x=p-z;return{id:`b${w}`,x:A,y:x,w:Math.max(0,C),h:Math.max(0,z),rx:m,ry:m,color:u,title:`${c[w]??String(y)}: ${y}`}})),t.set(!1)});return()=>a()}});var Pl=j("sui-bar-chart",{props:{data:g.string({default:"",reflect:!0}),labels:g.string({default:""}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),orientation:g.union(["vertical","horizontal"],{default:"vertical",reflect:!0}),width:g.number({default:300}),height:g.number({default:200}),max:g.number({default:0}),label:g.string({default:"bar chart"})},shadow:!0,styles:`
  :host {
    display: block;
    width: 100%;
    height: 200px;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  svg { display: block; width: 100%; height: 100%; overflow: hidden; }

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.labels=n.labels,t.tone=n.tone,t.orientation=n.orientation,t.width=n.width,t.height=n.height,t.max=n.max,t.label=n.label;let a=null;if(_t&&typeof _t.setup==="function"){if(e.scope=t,a=_t.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>X({anchor:d,items:()=>t.bars(),key:(l,i)=>{let u=Object.create(t);return u.b=l,u.__i=i,((s)=>l.id)(u)},render:(l,i)=>{let u=Object.create(t);return u.b=l,u.__i=i,u.__v=P(0),((s)=>{let m=document.createDocumentFragment(),h=[],v=document.createElement("rect");h.push(E(v,"x",()=>(s.__v(),s.b.x))),h.push(E(v,"y",()=>(s.__v(),s.b.y))),h.push(E(v,"width",()=>(s.__v(),s.b.w))),h.push(E(v,"height",()=>(s.__v(),s.b.h))),h.push(E(v,"rx",()=>(s.__v(),s.b.rx))),h.push(E(v,"ry",()=>(s.__v(),s.b.ry))),h.push(E(v,"fill",()=>(s.__v(),s.b.color)));let y=document.createElement("title"),w=document.createTextNode("");return y.appendChild(w),h.push(F(w,()=>(s.__v(),s.b.title))),v.appendChild(y),m.appendChild(v),{nodes:Array.from(m.childNodes),cleanup:()=>{for(let k of h)try{k()}catch(A){}},update:(k,A)=>{s.b=k,s.__i=A,s.__v.set((x)=>x+1)}}})(u)}}),c=(d)=>H({anchor:d,test:()=>{if(t.isEmpty())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("svg");return i.push(E(u,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),u.setAttribute("aria-hidden","true"),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(m){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("svg");i.push(E(u,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),u.setAttribute("preserveaspectratio","none"),u.setAttribute("role","img"),i.push(E(u,"aria-label",()=>t.label()));let s=document.createComment("for");return u.appendChild(s),i.push(b(s)),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let m of i)try{m()}catch(h){}}}}]}),f=r.shadowRoot??r;L(f);let p=document.createComment("if");if(f.appendChild(p),o.push(c(p)),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var tn=O({scope:()=>{let e=P([]),n=P([]);return{seriesPaths:e,gridLines:n}},mount(e,n){let r=e,o=r.seriesPaths,t=r.gridLines,a=D(()=>{let b=rt(r.series(),[]),c=r.width(),f=r.height(),p=b.flatMap((m)=>m.data),d=p.length?Math.min(...p):0,i=(p.length?Math.max(...p):1)-d||1,u=f-4;t.set(r.grid()?[0.25,0.5,0.75].map((m,h)=>({id:`g${h}`,y:2+u*m})):[]);let s=[];for(let m of b){if(m.data.length<2)continue;let h=m.data.map((k)=>(k-d)/i*100),{d:v,points:y}=Et(h,c,f,2);if(!v)continue;let w=K(m.tone);s.push({name:m.name,strokeD:v,color:w,pts:y.map(([k,A],x)=>({id:`${m.name}-${x}`,cx:k,cy:A}))})}o.set(s)});return()=>a()}});var Ul=j("sui-line-chart",{props:{series:g.string({default:"",reflect:!0}),labels:g.string({default:""}),width:g.number({default:300}),height:g.number({default:200}),points:g.bool({default:!1,reflect:!0}),grid:g.bool({default:!1,reflect:!0}),label:g.string({default:"line chart"})},shadow:!0,styles:`
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

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.series=n.series,t.labels=n.labels,t.width=n.width,t.height=n.height,t.points=n.points,t.grid=n.grid,t.label=n.label;let a=null;if(tn&&typeof tn.setup==="function"){if(e.scope=t,a=tn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(m)=>X({anchor:m,items:()=>t.gridLines(),key:(h,v)=>{let y=Object.create(t);return y.g=h,y.__i=v,((w)=>h.id)(y)},render:(h,v)=>{let y=Object.create(t);return y.g=h,y.__i=v,y.__v=P(0),((w)=>{let k=document.createDocumentFragment(),A=[],x=document.createElement("line");return x.setAttribute("class","grid-line"),x.setAttribute("x1","0"),A.push(E(x,"x2",()=>(w.__v(),w.width()))),A.push(E(x,"y1",()=>(w.__v(),w.g.y))),A.push(E(x,"y2",()=>(w.__v(),w.g.y))),k.appendChild(x),{nodes:Array.from(k.childNodes),cleanup:()=>{for(let C of A)try{C()}catch(z){}},update:(C,z)=>{w.g=C,w.__i=z,w.__v.set((M)=>M+1)}}})(y)}}),c=(m)=>X({anchor:m,items:()=>t.seriesPaths(),key:(h,v)=>{let y=Object.create(t);return y.s=h,y.__i=v,((w)=>h.name)(y)},render:(h,v)=>{let y=Object.create(t);return y.s=h,y.__i=v,y.__v=P(0),((w)=>{let k=(N)=>X({anchor:N,items:()=>w.s.pts,key:(S,B)=>{let U=Object.create(w);return U.p=S,U.__i=B,((G)=>S.id)(U)},render:(S,B)=>{let U=Object.create(w);return U.p=S,U.__i=B,U.__v=P(0),((G)=>{let q=document.createDocumentFragment(),Q=[],Z=document.createElement("circle");return Q.push(E(Z,"cx",()=>(G.__v(),G.p.cx))),Q.push(E(Z,"cy",()=>(G.__v(),G.p.cy))),Z.setAttribute("r","2.5"),Q.push(E(Z,"fill",()=>(G.__v(),G.s.color))),q.appendChild(Z),{nodes:Array.from(q.childNodes),cleanup:()=>{for(let R of Q)try{R()}catch(at){}},update:(R,at)=>{G.p=R,G.__i=at,G.__v.set((I)=>I+1)}}})(U)}}),A=(N)=>H({anchor:N,test:()=>{if(w.points())return 0;return-1},branches:[()=>{let S=document.createDocumentFragment(),B=[],U=document.createComment("for");return S.appendChild(U),B.push(k(U)),{nodes:Array.from(S.childNodes),cleanup:()=>{for(let G of B)try{G()}catch(q){}}}}]}),x=document.createDocumentFragment(),C=[],z=document.createElement("path");C.push(E(z,"d",()=>(w.__v(),w.s.strokeD))),z.setAttribute("fill","none"),C.push(E(z,"stroke",()=>(w.__v(),w.s.color))),z.setAttribute("stroke-width","1.75"),z.setAttribute("stroke-linecap","round"),z.setAttribute("stroke-linejoin","round"),x.appendChild(z);let M=document.createComment("if");return x.appendChild(M),C.push(A(M)),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let N of C)try{N()}catch(S){}},update:(N,S)=>{w.s=N,w.__i=S,w.__v.set((B)=>B+1)}}})(y)}}),f=r.shadowRoot??r;L(f);let p=document.createElement("div");p.setAttribute("class","wrap");let d=document.createElement("svg");o.push(E(d,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),d.setAttribute("preserveaspectratio","none"),d.setAttribute("role","img"),o.push(E(d,"aria-label",()=>t.label()));let l=document.createComment("for");d.appendChild(l),o.push(b(l));let i=document.createComment("for");d.appendChild(i),o.push(c(i)),p.appendChild(d);let u=document.createElement("div");u.setAttribute("class","legend");let s=document.createElement("slot");if(u.appendChild(s),p.appendChild(u),f.appendChild(p),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let h=new Proxy(t,{get(v,y){return v[y]},set(v,y,w){let k=v[y];if(k!=null&&typeof k.set==="function"){if(typeof w==="function")k.set(()=>w);else k.set(w);return!0}return!1}});if(m.mount){let v=m.mount(h,r);if(typeof v==="function")o.push(v)}if(m.unmount)o.push(()=>m.unmount(h))}}return t}});var nn=O({scope:()=>{let e=P(!1),n=P(""),r=P(""),o=P("var(--primary)");return{hasData:e,strokePath:n,areaPath:r,lineColor:o}},mount(e,n){let r=e,o=r.hasData,t=r.strokePath,a=r.areaPath,b=r.lineColor,c=D(()=>{let f=kt(r.data()),p=r.width(),d=r.height(),l=K(r.tone());if(b.set(l),f.length<2){o.set(!1),t.set(""),a.set("");return}let{d:i,points:u}=Et(f,p,d,2);if(t.set(i),r.fill()){let s=u[0],m=u[u.length-1];a.set(`${i} L${m[0].toFixed(2)},${d} L${s[0].toFixed(2)},${d} Z`)}else a.set("");o.set(!0)});return()=>c()}});var Zl=j("sui-sparkline",{props:{data:g.string({default:"",reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),width:g.number({default:80}),height:g.number({default:20}),fill:g.bool({default:!1,reflect:!0}),label:g.string({default:"sparkline"})},shadow:!0,styles:`
  :host {
    --sui-sparkline-w: auto;
    --sui-sparkline-h: 40px;
    display: inline-block;
    width: var(--sui-sparkline-w);
    height: var(--sui-sparkline-h);
    line-height: 0;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  svg { display: block; overflow: visible; }

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.tone=n.tone,t.width=n.width,t.height=n.height,t.fill=n.fill,t.label=n.label;let a=null;if(nn&&typeof nn.setup==="function"){if(e.scope=t,a=nn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>H({anchor:d,test:()=>{if(t.fill())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("path");return i.push(E(u,"d",()=>t.areaPath())),i.push(E(u,"fill",()=>t.lineColor())),u.setAttribute("fill-opacity","0.18"),u.setAttribute("stroke","none"),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(m){}}}}]}),c=(d)=>H({anchor:d,test:()=>{if(t.hasData())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("svg");i.push(E(u,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),i.push(E(u,"width",()=>t.width())),i.push(E(u,"height",()=>t.height())),u.setAttribute("role","img"),i.push(E(u,"aria-label",()=>t.label()));let s=document.createComment("if");u.appendChild(s),i.push(b(s));let m=document.createElement("path");i.push(E(m,"d",()=>t.strokePath())),m.setAttribute("fill","none"),i.push(E(m,"stroke",()=>t.lineColor())),m.setAttribute("stroke-width","1.5"),m.setAttribute("stroke-linecap","round"),m.setAttribute("stroke-linejoin","round");let h=document.createTextNode(`
    `);return m.appendChild(h),u.appendChild(m),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let v of i)try{v()}catch(y){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("svg");i.push(E(u,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),i.push(E(u,"width",()=>t.width())),i.push(E(u,"height",()=>t.height())),u.setAttribute("aria-hidden","true");let s=document.createTextNode(`
  `);return u.appendChild(s),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let m of i)try{m()}catch(h){}}}}]}),f=r.shadowRoot??r;L(f);let p=document.createComment("if");if(f.appendChild(p),o.push(c(p)),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var en=O({scope:()=>({segments:P([])}),mount(e,n){let r=e,o=r.segments,t=D(()=>{let a=rt(r.data(),[]),b=a.reduce((s,m)=>s+Math.max(0,m.value),0);if(b<=0){o.set([]);return}let c=r.size(),f=r.thickness(),p=(c-f)/2,d=c/2,l=c/2,i=2*Math.PI,u=-Math.PI/2;o.set(a.filter((s)=>s.value>0).map((s,m)=>{let h=s.value/b*i,v=h>=i-0.001?i-0.001:h,y=Ht(d,l,p,u,u+v);return u+=h,{id:`s${m}`,d:y,color:K(s.tone),title:`${s.label}: ${s.value}`}}))});return()=>t()}});var td=j("sui-donut",{props:{data:g.string({default:"",reflect:!0}),size:g.number({default:120}),thickness:g.number({default:16}),label:g.string({default:"donut chart"})},shadow:!0,styles:`
  :host {
    display: inline-block;
    position: relative;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  svg { display: block; }
  .track { stroke: var(--muted); }
  .center {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--heading-font);
    font-weight: var(--font-weight-semibold);
    font-size: var(--text-lg);
    color: var(--foreground);
    line-height: var(--leading-tight);
    pointer-events: none;
  }

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.size=n.size,t.thickness=n.thickness,t.label=n.label;let a=null;if(en&&typeof en.setup==="function"){if(e.scope=t,a=en.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(s)=>X({anchor:s,items:()=>t.segments(),key:(m,h)=>{let v=Object.create(t);return v.seg=m,v.__i=h,((y)=>m.id)(v)},render:(m,h)=>{let v=Object.create(t);return v.seg=m,v.__i=h,v.__v=P(0),((y)=>{let w=document.createDocumentFragment(),k=[],A=document.createElement("path");A.setAttribute("part","segment"),k.push(E(A,"d",()=>(y.__v(),y.seg.d))),A.setAttribute("fill","none"),k.push(E(A,"stroke",()=>(y.__v(),y.seg.color))),k.push(E(A,"stroke-width",()=>(y.__v(),y.thickness()))),A.setAttribute("stroke-linecap","butt");let x=document.createElement("title"),C=document.createTextNode("");return x.appendChild(C),k.push(F(C,()=>(y.__v(),y.seg.title))),A.appendChild(x),w.appendChild(A),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let z of k)try{z()}catch(M){}},update:(z,M)=>{y.seg=z,y.__i=M,y.__v.set((N)=>N+1)}}})(v)}}),c=r.shadowRoot??r;L(c);let f=document.createElement("svg");o.push(E(f,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),f.setAttribute("role","img"),o.push(E(f,"aria-label",()=>t.label())),o.push(E(f,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let p=document.createElement("circle");p.setAttribute("class","track"),p.setAttribute("part","track"),o.push(E(p,"cx",()=>t.size()/2)),o.push(E(p,"cy",()=>t.size()/2)),o.push(E(p,"r",()=>(t.size()-t.thickness())/2)),p.setAttribute("fill","none"),o.push(E(p,"stroke-width",()=>t.thickness()));let d=document.createTextNode(`
  `);p.appendChild(d),f.appendChild(p);let l=document.createComment("for");f.appendChild(l),o.push(b(l)),c.appendChild(f);let i=document.createElement("div");i.setAttribute("class","center");let u=document.createElement("slot");if(i.appendChild(u),c.appendChild(i),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let m=new Proxy(t,{get(h,v){return h[v]},set(h,v,y){let w=h[v];if(w!=null&&typeof w.set==="function"){if(typeof y==="function")w.set(()=>y);else w.set(y);return!0}return!1}});if(s.mount){let h=s.mount(m,r);if(typeof h==="function")o.push(h)}if(s.unmount)o.push(()=>s.unmount(m))}}return t}});var Sr=Math.PI/180,rn=135*Sr,Hr=405*Sr,on=O({scope:()=>({trackD:P(""),valueD:P(""),arcColor:P("var(--primary)")}),mount(e,n){let r=e,o=r.trackD,t=r.valueD,a=r.arcColor,b=D(()=>{let c=r.size(),f=r.thickness(),p=(c-f)/2-2,d=c/2,l=c/2;o.set(Ht(d,l,p,rn,Hr));let i=r.value(),u=r.min(),s=r.max(),m=Math.max(0,Math.min(1,(i-u)/(s-u||1))),h=rn+(Hr-rn)*m;t.set(m>0?Ht(d,l,p,rn,h):"");let v=r.tone(),y=r.lowThreshold(),w=r.highThreshold(),k=v;if(!k)if(y>0&&i<y)k="destructive";else if(w>0&&i<w)k="warning";else if(y>0||w>0)k="success";else k="primary";a.set(K(k))});return()=>b()}});var sd=j("sui-gauge",{props:{value:g.number({default:0,reflect:!0}),min:g.number({default:0}),max:g.number({default:100}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"",reflect:!0}),"low-threshold":g.number({default:0}),"high-threshold":g.number({default:0}),size:g.number({default:140}),thickness:g.number({default:12}),label:g.string({default:"gauge"})},shadow:!0,styles:`
  :host {
    display: inline-block;
    position: relative;
    transition: opacity var(--duration-fast) var(--ease-out);
  }
  svg { display: block; }
  .track { stroke: var(--muted); }
  .gauge-label {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--heading-font);
    font-weight: var(--font-weight-semibold);
    font-size: var(--text-xl);
    color: var(--foreground);
  }

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.min=n.min,t.max=n.max,t.tone=n.tone,t.lowThreshold=n.lowThreshold,t.highThreshold=n.highThreshold,t.size=n.size,t.thickness=n.thickness,t.label=n.label;let a=null;if(on&&typeof on.setup==="function"){if(e.scope=t,a=on.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(s)=>H({anchor:s,test:()=>{if(t.valueD())return 0;return-1},branches:[()=>{let m=document.createDocumentFragment(),h=[],v=document.createElement("path");v.setAttribute("part","fill"),h.push(E(v,"d",()=>t.valueD())),v.setAttribute("fill","none"),h.push(E(v,"stroke",()=>t.arcColor())),h.push(E(v,"stroke-width",()=>t.thickness())),v.setAttribute("stroke-linecap","round");let y=document.createTextNode(`
    `);return v.appendChild(y),m.appendChild(v),{nodes:Array.from(m.childNodes),cleanup:()=>{for(let w of h)try{w()}catch(k){}}}}]}),c=r.shadowRoot??r;L(c);let f=document.createElement("svg");o.push(E(f,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),f.setAttribute("role","img"),o.push(E(f,"aria-label",()=>t.label())),o.push(E(f,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let p=document.createElement("path");p.setAttribute("class","track"),p.setAttribute("part","track"),o.push(E(p,"d",()=>t.trackD())),p.setAttribute("fill","none"),o.push(E(p,"stroke-width",()=>t.thickness())),p.setAttribute("stroke-linecap","round");let d=document.createTextNode(`
  `);p.appendChild(d),f.appendChild(p);let l=document.createComment("if");f.appendChild(l),o.push(b(l)),c.appendChild(f);let i=document.createElement("div");i.setAttribute("class","gauge-label");let u=document.createElement("slot");if(i.appendChild(u),c.appendChild(i),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let m=new Proxy(t,{get(h,v){return h[v]},set(h,v,y){let w=h[v];if(w!=null&&typeof w.set==="function"){if(typeof y==="function")w.set(()=>y);else w.set(y);return!0}return!1}});if(s.mount){let h=s.mount(m,r);if(typeof h==="function")o.push(h)}if(s.unmount)o.push(()=>s.unmount(m))}}return t}});var an=O({scope:()=>({bars:P([])}),mount(e,n){let r=e,o=r.bars,t=D(()=>{let a=rt(r.segments(),[]),b=a.reduce((d,l)=>d+Math.max(0,l.count),0),c=r.total(),f=c>0?c:b,p=f<=0?1:f;o.set(a.filter((d)=>d.count>0).map((d,l)=>{let i=Math.max(0,d.count)/p*100;return{id:`b${l}`,style:`width:${i.toFixed(3)}%;background:${K(d.tone)};`,title:`${d.label??d.tone}: ${d.count}`}}))});return()=>t()}});var md=j("sui-meter",{props:{segments:g.string({default:"",reflect:!0}),total:g.number({default:0}),label:g.string({default:"meter"})},shadow:!0,styles:`
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

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.segments=n.segments,t.total=n.total,t.label=n.label;let a=null;if(an&&typeof an.setup==="function"){if(e.scope=t,a=an.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>X({anchor:u,items:()=>t.bars(),key:(s,m)=>{let h=Object.create(t);return h.b=s,h.__i=m,((v)=>s.id)(h)},render:(s,m)=>{let h=Object.create(t);return h.b=s,h.__i=m,h.__v=P(0),((v)=>{let y=document.createDocumentFragment(),w=[],k=document.createElement("div");return k.setAttribute("class","seg"),k.setAttribute("part","segment"),w.push(E(k,"style",()=>(v.__v(),v.b.style))),w.push(E(k,"title",()=>(v.__v(),v.b.title))),y.appendChild(k),{nodes:Array.from(y.childNodes),cleanup:()=>{for(let A of w)try{A()}catch(x){}},update:(A,x)=>{v.b=A,v.__i=x,v.__v.set((C)=>C+1)}}})(h)}}),c=r.shadowRoot??r;L(c);let f=document.createElement("div");f.setAttribute("class","wrap");let p=document.createElement("div");p.setAttribute("class","bar"),p.setAttribute("part","bar"),p.setAttribute("role","img"),o.push(E(p,"aria-label",()=>t.label()));let d=document.createComment("for");p.appendChild(d),o.push(b(d)),f.appendChild(p);let l=document.createElement("div");l.setAttribute("class","legend");let i=document.createElement("slot");if(l.appendChild(i),f.appendChild(l),c.appendChild(f),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(m,h){return m[h]},set(m,h,v){let y=m[h];if(y!=null&&typeof y.set==="function"){if(typeof v==="function")y.set(()=>v);else y.set(v);return!0}return!1}});if(u.mount){let m=u.mount(s,r);if(typeof m==="function")o.push(m)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var sn=O({scope:()=>({cells:P([]),svgW:P(0),svgH:P(0)}),mount(e,n){let r=e,o=r.cells,t=r.svgW,a=r.svgH,b=D(()=>{let c=Nt(r.rows()),f=Nt(r.cols()),p=r.cellSize(),d=r.gap(),l=r.data(),i=rt(l,[]),u=Array.isArray(i)&&i.length>0&&Array.isArray(i[0]),s;if(u)s=i;else{let z=kt(l),M=f.length||1,N=c.length||(z.length>0?Math.ceil(z.length/M):0);s=Array.from({length:N},(S,B)=>z.slice(B*M,(B+1)*M))}let m=s.length,h=m>0?Math.max(...s.map((z)=>z.length)):0,v=s.flatMap((z)=>z.filter((M)=>typeof M==="number")),y=v.length?Math.min(...v):0,k=(v.length?Math.max(...v):1)-y,A=K(r.tone()),x=p+d;t.set(h>0?h*p+(h-1)*d:0),a.set(m>0?m*p+(m-1)*d:0);let C=[];for(let z=0;z<m;z++){let M=s[z]??[];for(let N=0;N<h;N++){let S=M[N]??0,B=k===0?y>0?1:0:(S-y)/k,U=(B*100).toFixed(1),G=B>0?`fill:color-mix(in oklch,${A} ${U}%,transparent)`:"";C.push({id:`c${z}-${N}`,x:N*x,y:z*x,w:p,h:p,fill:G})}}o.set(C)});return()=>b()}});var wd=j("sui-heatmap",{props:{data:g.string({default:"",reflect:!0}),rows:g.string({default:""}),cols:g.string({default:""}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),"cell-size":g.number({default:20}),gap:g.number({default:3}),label:g.string({default:"heatmap"})},shadow:!0,styles:`
  :host {
    display: inline-block;
    font-family: var(--font-body);
  }
  svg { display: block; overflow: visible; }
  .cell { fill: var(--muted); }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.rows=n.rows,t.cols=n.cols,t.tone=n.tone,t.cellSize=n.cellSize,t.gap=n.gap,t.label=n.label;let a=null;if(sn&&typeof sn.setup==="function"){if(e.scope=t,a=sn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>X({anchor:d,items:()=>t.cells(),key:(l,i)=>{let u=Object.create(t);return u.cell=l,u.__i=i,((s)=>l.id)(u)},render:(l,i)=>{let u=Object.create(t);return u.cell=l,u.__i=i,u.__v=P(0),((s)=>{let m=document.createDocumentFragment(),h=[],v=document.createElement("rect");v.setAttribute("class","cell"),v.setAttribute("part","cell"),h.push(E(v,"x",()=>(s.__v(),s.cell.x))),h.push(E(v,"y",()=>(s.__v(),s.cell.y))),h.push(E(v,"width",()=>(s.__v(),s.cell.w))),h.push(E(v,"height",()=>(s.__v(),s.cell.h))),v.setAttribute("rx","2"),h.push(E(v,"style",()=>(s.__v(),s.cell.fill)));let y=document.createTextNode(`
    `);return v.appendChild(y),m.appendChild(v),{nodes:Array.from(m.childNodes),cleanup:()=>{for(let w of h)try{w()}catch(k){}},update:(w,k)=>{s.cell=w,s.__i=k,s.__v.set((A)=>A+1)}}})(u)}}),c=r.shadowRoot??r;L(c);let f=document.createElement("svg");o.push(E(f,"viewbox",()=>"0 0 "+t.svgW()+" "+t.svgH())),f.setAttribute("role","img"),o.push(E(f,"aria-label",()=>t.label()));let p=document.createComment("for");if(f.appendChild(p),o.push(b(p)),c.appendChild(f),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var Ed=j("sui-button",{props:{tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:g.union(["solid","soft","outline","ghost","default"],{default:"default",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),type:g.union(["button","submit","reset"],{default:"button",reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),loading:g.bool({default:!1,reflect:!0}),label:g.string({default:""}),pressed:g.string({default:"",reflect:!0}),expanded:g.string({default:"",reflect:!0})},shadow:!0,styles:`
    :host {
      display: inline-flex;
      vertical-align: middle;
      --tone: var(--foreground);
      --tone-fg: var(--foreground);
      --tone-contrast: var(--background);
      --btn-h: 34px;
      --btn-px: var(--space-3);
      --btn-fs: 13px;
      --btn-fw: var(--font-weight-medium, 500);
      --btn-gap: 8px;
    }

    /* Size matrix */
    :host([size='sm']) { --btn-h: 28px; --btn-fs: 12px; --btn-px: 10px; --btn-gap: 6px; }
    :host([size='md']) { --btn-h: 34px; --btn-fs: 13px; --btn-px: 12px; --btn-gap: 8px; }
    :host([size='lg']) { --btn-h: 40px; --btn-fs: 14px; --btn-px: 16px; --btn-gap: 8px; }

    button {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      gap: var(--btn-gap);
      min-height: var(--btn-h);
      padding: 0 var(--btn-px);
      font-family: var(--font-body);
      font-weight: var(--btn-fw);
      font-size: var(--btn-fs);
      line-height: 1;
      letter-spacing: -0.005em;
      border-radius: var(--radius-md);
      border: 1px solid transparent;
      background: transparent;
      color: var(--foreground);
      cursor: pointer;
      user-select: none;
      white-space: nowrap;
      isolation: isolate;
      position: relative;
      transition:
        background-color var(--duration-fast) var(--ease-out),
        border-color var(--duration-fast) var(--ease-out),
        box-shadow var(--duration-fast) var(--ease-out),
        transform var(--duration-fast) var(--ease-out);
    }

    /* Default (no variant attr) — neutral bordered button. */
    :host(:not([variant])) button,
    :host([variant='default']) button {
      background: var(--background);
      border-color: var(--border);
      color: var(--foreground);
      box-shadow: var(--shadow-xs);
    }
    :host(:not([variant])) button:hover:not(:disabled),
    :host([variant='default']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--muted) 70%, var(--background));
      border-color: color-mix(in oklch, var(--border) 160%, transparent);
      transform: translateY(-1px);
    }

    /* Solid */
    :host([variant='solid']) button {
      background: var(--tone);
      border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
      color: var(--tone-contrast);
      box-shadow: var(--shadow-xs);
    }
    :host([variant='solid']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 92%, white 8%);
      transform: translateY(-1px);
      box-shadow: var(--shadow-sm);
    }

    /* Soft */
    :host([variant='soft']) button {
      background: color-mix(in oklch, var(--tone) 14%, transparent);
      border-color: color-mix(in oklch, var(--tone) 22%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='soft']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 22%, transparent);
    }

    /* Outline */
    :host([variant='outline']) button {
      background: transparent;
      border-color: color-mix(in oklch, var(--tone) 55%, transparent);
      color: var(--tone-fg);
    }
    :host([variant='outline']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 10%, transparent);
    }

    /* Ghost */
    :host([variant='ghost']) button {
      background: transparent;
      border-color: transparent;
      color: var(--tone-fg);
    }
    :host([variant='ghost']) button:hover:not(:disabled) {
      background: color-mix(in oklch, var(--tone) 12%, transparent);
    }

    /* Active press — feels clicked across all variants. */
    :host(:not([disabled]):not([loading])) button:active:not(:disabled) {
      transform: translateY(1px) scale(0.98);
    }
    @media (prefers-reduced-motion: reduce) {
      button { transition: none; }
      button:active { transform: none !important; }
    }

    /* Focus ring */
    button:focus-visible {
      outline: none;
      box-shadow:
        0 0 0 2px var(--background),
        0 0 0 4px color-mix(in oklch, var(--tone) 55%, transparent);
    }

    /* Disabled */
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

    /* Loading state — fade the label, overlay a spinner. */
    :host([loading]) button { cursor: progress; }
    :host([loading]) .content { opacity: 0; }
    .content {
      display: inline-flex;
      align-items: center;
      gap: var(--btn-gap);
      transition: opacity var(--duration-fast) var(--ease-out);
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
      width: 14px;
      height: 14px;
      border: 1.5px solid currentColor;
      border-right-color: transparent;
      border-radius: 50%;
      animation: sui-button-spin 0.7s linear infinite;
    }
    @keyframes sui-button-spin { to { transform: rotate(360deg); } }

    /* Icon-only — collapse padding to a square when default slot is empty. */
    :host([data-icon-only]) button {
      padding-inline: 0;
      width: var(--btn-h);
      min-width: var(--btn-h);
    }

    ::slotted([slot='leading']),
    ::slotted([slot='trailing']) {
      display: inline-flex;
      align-items: center;
      flex: 0 0 auto;
    }
  `,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.type=n.type,t.disabled=n.disabled,t.loading=n.loading,t.label=n.label,t.pressed=n.pressed,t.expanded=n.expanded;let a=(u)=>H({anchor:u,test:()=>{if(t.loading())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("span");h.setAttribute("class","spinner-wrap"),h.setAttribute("aria-hidden","true");let v=document.createElement("span");return v.setAttribute("class","spinner"),h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let y of m)try{y()}catch(w){}}}}]}),b=r.shadowRoot??r;L(b);let c=document.createElement("button");c.setAttribute("part","button"),o.push(E(c,"type",()=>t.type())),o.push(E(c,"disabled",()=>t.disabled()||t.loading())),o.push(E(c,"aria-busy",()=>t.loading()?"true":"false")),o.push(E(c,"aria-disabled",()=>t.disabled()||t.loading()?"true":"false")),o.push(E(c,"aria-label",()=>t.label())),o.push(E(c,"aria-pressed",()=>t.pressed())),o.push(E(c,"aria-expanded",()=>t.expanded()));let f=document.createElement("span");f.setAttribute("class","content");let p=document.createElement("slot");p.setAttribute("name","leading"),f.appendChild(p);let d=document.createElement("slot");o.push(tt(d,r,"data-icon-only")),f.appendChild(d);let l=document.createElement("slot");l.setAttribute("name","trailing"),f.appendChild(l),c.appendChild(f);let i=document.createComment("if");return c.appendChild(i),o.push(a(i)),b.appendChild(c),t}});var un=O({mount(e,n){let r=e,o=P(0),t=()=>{let i=r.tabSlot();if(!i)return Array.from(n.querySelectorAll(":scope > sui-tab"));return i.assignedElements().filter((u)=>u.tagName.toLowerCase()==="sui-tab")},a=(i)=>{if(r.active()===i)return;r.active.set(i),n.dispatchEvent(new CustomEvent("sui-tab-change",{detail:{name:i},bubbles:!0,composed:!0}))},b=D(()=>{o();let i=r.active(),u=t(),s=i;if(!s&&u.length>0){if(s=u[0].getAttribute("name")??"",s)r.active.set(s)}for(let m of u){let h=m.getAttribute("name")??"";if(h===s)m.setAttribute("active","");else m.removeAttribute("active");m.id=`sui-tab-${h}`,m.setAttribute("aria-controls",`sui-tabpanel-${h}`)}for(let m of Array.from(n.querySelectorAll(':scope > sui-tab-panel, :scope > [role="tabpanel"]')))if((m.getAttribute("name")??"")===s)m.removeAttribute("hidden"),m.setAttribute("active","");else m.setAttribute("hidden",""),m.removeAttribute("active")}),c=(i)=>{let u=r.orientation()==="vertical",s=u?"ArrowDown":"ArrowRight",m=u?"ArrowUp":"ArrowLeft",h=[s,m,"Home","End"],v=["Enter"," "];if(!h.includes(i.key)&&!v.includes(i.key))return;i.preventDefault();let y=t().filter((z)=>!z.hasAttribute("disabled"));if(!y.length)return;let w=y.map((z)=>z.getAttribute("name")??""),k=r.active(),A=Math.max(0,w.indexOf(k)),x=A;if(i.key===s)x=(A+1)%w.length;else if(i.key===m)x=(A-1+w.length)%w.length;else if(i.key==="Home")x=0;else if(i.key==="End")x=w.length-1;let C=w[x];a(C),queueMicrotask(()=>{let z=y[x],M=z.shadowRoot?.querySelector("button");M?M.focus():z.focus()})},f=(i)=>{let u=i.target.closest("sui-tab");if(!u||u.hasAttribute("disabled"))return;let s=u.getAttribute("name")??"";if(s)a(s)},p=()=>o.set(o()+1),d=r.strip(),l=r.tabSlot();if(d)d.addEventListener("keydown",c),d.addEventListener("click",f);if(l)l.addEventListener("slotchange",p);return()=>{if(b(),d)d.removeEventListener("keydown",c),d.removeEventListener("click",f);if(l)l.removeEventListener("slotchange",p)}}});var Od=j("sui-tabs",{props:{active:g.string({default:"",reflect:!0}),variant:g.union(["underline","pill","enclosed"],{default:"underline",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),sticky:g.bool({default:!1,reflect:!0}),orientation:g.union(["horizontal","vertical"],{default:"horizontal",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    --tone: var(--primary);
  }
  :host([sticky]) { position: sticky; top: 0; z-index: 10; background: var(--background); }

  .strip {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
    border-bottom: 1px solid var(--border);
  }

  :host([orientation='vertical']) .strip {
    flex-direction: column;
    align-items: flex-start;
    border-bottom: 0;
    border-right: 1px solid var(--border);
  }

  :host([variant='pill']) .strip {
    gap: 4px;
    border-bottom: 0;
    padding: 4px;
    background: var(--muted);
    border-radius: var(--radius-md);
  }

  :host([variant='enclosed']) .strip {
    gap: 0;
    border-bottom: 1px solid var(--border);
  }

  ::slotted(sui-tab),
  ::slotted([slot='tab']) {
    flex: 0 0 auto;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.active=n.active,t.variant=n.variant,t.size=n.size,t.tone=n.tone,t.sticky=n.sticky,t.orientation=n.orientation;let a=null;if(un&&typeof un.setup==="function"){if(e.scope=t,a=un.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","strip"),c.setAttribute("part","strip"),c.setAttribute("role","tablist"),o.push(E(c,"aria-orientation",()=>t.orientation())),T(t,"strip",c);let f=document.createElement("slot");if(T(t,"tabSlot",f),c.appendChild(f),b.appendChild(c),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var ln=O({mount(e,n){let r=e,o=D(()=>{let t=r.btn();if(!t)return;t.tabIndex=r.active()?0:-1});return()=>{o()}}});var Sd=j("sui-tab",{props:{name:g.string({default:"",reflect:!0}),label:g.string({default:"",reflect:!0}),count:g.string({default:"",reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),active:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: inline-flex; }

  button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 14px;
    font: inherit;
    font-family: var(--font-body);
    font-size: 13px;
    font-weight: var(--font-weight-medium, 500);
    color: var(--muted-foreground);
    background: transparent;
    border: 0;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    border-radius: 0;
    white-space: nowrap;
    transition:
      color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out);
  }

  button:hover:not(:disabled) { color: var(--foreground); }

  :host([active]) button {
    color: var(--foreground);
    border-bottom-color: var(--tone, var(--primary));
  }

  button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .leading {
    display: none;
    align-items: center;
    flex: 0 0 auto;
    color: currentColor;
  }
  .leading:has(::slotted(*)) { display: inline-flex; }

  .count {
    font-size: 11px;
    font-weight: var(--font-weight-medium, 500);
    color: var(--muted-foreground);
    padding: 0 6px;
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--muted) 80%, transparent);
  }
  :host([active]) .count {
    color: var(--tone, var(--primary));
    background: color-mix(in oklch, var(--tone, var(--primary)) 12%, transparent);
  }

  button:focus-visible {
    outline: 2px solid color-mix(in oklch, var(--tone, var(--primary)) 55%, transparent);
    outline-offset: -2px;
  }

  :host([variant='pill']) button {
    border-bottom: 0;
    border-radius: var(--radius-sm);
    padding: 6px 12px;
  }
  :host([variant='pill'][active]) button {
    background: var(--background);
    box-shadow: var(--shadow-sm, 0 1px 2px 0 rgb(0 0 0 / 0.05));
    color: var(--foreground);
  }

  :host([variant='enclosed']) button {
    border-bottom: 0;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
    border: 1px solid transparent;
  }
  :host([variant='enclosed'][active]) button {
    background: var(--background);
    border-color: var(--border) var(--border) transparent;
    color: var(--foreground);
    position: relative;
    bottom: -1px;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.name=n.name,t.label=n.label,t.count=n.count,t.disabled=n.disabled,t.active=n.active;let a=null;if(ln&&typeof ln.setup==="function"){if(e.scope=t,a=ln.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>H({anchor:u,test:()=>{if(t.count()!=="")return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("span");h.setAttribute("class","count"),h.setAttribute("aria-hidden","true");let v=document.createTextNode("");return h.appendChild(v),m.push(F(v,()=>t.count())),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let y of m)try{y()}catch(w){}}}}]}),c=r.shadowRoot??r;L(c);let f=document.createElement("button");f.setAttribute("part","button"),f.setAttribute("type","button"),f.setAttribute("role","tab"),o.push(E(f,"data-name",()=>t.name())),o.push(E(f,"aria-selected",()=>t.active()?"true":"false")),o.push(E(f,"aria-disabled",()=>t.disabled()?"true":"false")),o.push(E(f,"disabled",()=>t.disabled())),T(t,"btn",f);let p=document.createElement("span");p.setAttribute("class","leading"),p.setAttribute("part","leading");let d=document.createElement("slot");d.setAttribute("name","leading"),p.appendChild(d),f.appendChild(p);let l=document.createElement("slot");f.appendChild(l);let i=document.createComment("if");if(f.appendChild(i),o.push(b(i)),c.appendChild(f),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(m,h){return m[h]},set(m,h,v){let y=m[h];if(y!=null&&typeof y.set==="function"){if(typeof v==="function")y.set(()=>v);else y.set(v);return!0}return!1}});if(u.mount){let m=u.mount(s,r);if(typeof m==="function")o.push(m)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var Gd=j("sui-empty-state",{props:{variant:g.union(["default","inline","placeholder","connecting"],{default:"default",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    color: var(--muted-foreground);
    --tone-fg: var(--muted-foreground);
    --es-pad-y: 64px;
    --es-pad-x: 16px;
    --es-gap: var(--space-3, 12px);
    --es-icon: 22px;
    --es-tile: 48px;
    --es-title-fs: var(--text-md, 16px);
    --es-desc-fs: var(--text-sm, 14px);
    --es-radius: var(--radius-lg);
    --es-border: 1px dashed var(--border);
    --es-bg: var(--card);
  }

  :host([size='sm']) {
    --es-pad-y: var(--space-7, 28px);
    --es-pad-x: var(--space-4, 16px);
    --es-gap: var(--space-2, 8px);
    --es-icon: 16px; --es-tile: 36px;
    --es-title-fs: var(--text-base, 14px);
  }
  :host([size='lg']) {
    --es-pad-y: 96px;
    --es-pad-x: var(--space-6, 24px);
    --es-gap: var(--space-4, 16px);
    --es-icon: 28px; --es-tile: 64px;
    --es-title-fs: var(--text-lg, 18px);
    --es-desc-fs: var(--text-base, 15px);
  }

  :host([variant='placeholder']),
  :host([variant='connecting']) {
    --es-border: 0;
    --es-bg: transparent;
    --es-radius: 0;
    --es-pad-y: var(--space-6, 24px);
  }
  :host([variant='inline']) {
    --es-border: 1px dashed var(--border);
    --es-bg: color-mix(in oklch, var(--muted) 55%, transparent);
    --es-radius: var(--radius-md);
    --es-pad-y: var(--space-3, 12px);
    --es-pad-x: var(--space-4, 16px);
    --es-gap: var(--space-3, 12px);
  }

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
  }
  :host([variant='inline']) .box {
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    text-align: left;
  }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: var(--es-tile);
    height: var(--es-tile);
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--tone-fg) 18%, var(--muted));
    color: var(--tone-fg);
    line-height: 1;
  }
  :host(:not([tone])) .icon,
  :host([tone='default']) .icon,
  :host([tone='muted']) .icon {
    background: var(--muted);
    color: var(--muted-foreground);
  }
  :host([data-no-icon]) .icon { display: none; }
  :host([variant='inline']) .icon {
    width: var(--es-icon);
    height: var(--es-icon);
    background: transparent;
  }
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

  .copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
    min-width: 0;
  }
  :host([variant='inline']) .copy { align-items: flex-start; }

  .title {
    font-family: var(--heading-font, var(--font-body));
    font-size: var(--es-title-fs);
    font-weight: var(--heading-weight, 600);
    color: var(--foreground);
    line-height: var(--leading-tight, 1.25);
  }
  :host([variant='connecting']) .title,
  :host([variant='placeholder']) .title {
    font-weight: var(--font-weight-medium);
    color: var(--muted-foreground);
  }

  .desc {
    font-size: var(--es-desc-fs);
    max-width: 40ch;
    color: var(--muted-foreground);
  }
  :host([variant='inline']) .desc { max-width: 60ch; }

  .actions {
    display: flex;
    gap: var(--space-2, 8px);
    justify-content: center;
    margin-top: 4px;
    flex-wrap: wrap;
  }
  :host([variant='inline']) .actions {
    margin-top: 0;
    margin-left: auto;
    justify-content: flex-end;
  }

  @media (forced-colors: active) {
    .box { border-color: CanvasText; }
    .icon { background: Canvas; color: CanvasText; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.variant=n.variant,t.size=n.size,t.tone=n.tone;let a=r.shadowRoot??r;L(a);let b=document.createElement("div");b.setAttribute("class","box"),b.setAttribute("part","box"),o.push(E(b,"role",()=>t.variant()==="connecting"?"status":"note")),o.push(E(b,"aria-live",()=>t.variant()==="connecting"?"polite":"off"));let c=document.createElement("div");c.setAttribute("class","icon"),c.setAttribute("part","icon"),c.setAttribute("aria-hidden","true");let f=document.createElement("slot");f.setAttribute("name","icon"),o.push(tt(f,r,"data-no-icon")),c.appendChild(f),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","copy");let d=document.createElement("div");d.setAttribute("class","title"),d.setAttribute("part","title");let l=document.createElement("slot");l.setAttribute("name","title"),d.appendChild(l),p.appendChild(d);let i=document.createElement("div");i.setAttribute("class","desc"),i.setAttribute("part","desc");let u=document.createElement("slot");i.appendChild(u),p.appendChild(i),b.appendChild(p);let s=document.createElement("div");s.setAttribute("class","actions"),s.setAttribute("part","actions");let m=document.createElement("slot");return m.setAttribute("name","action"),s.appendChild(m),b.appendChild(s),a.appendChild(b),t}});var $o=0,dn=O({mount(e,n){let r=e,o=r.inputEl();if(!o)return;n.expose({focus:()=>o.focus(),blur:()=>o.blur(),clear:()=>{o.value="",r.value.set("")},reportValidity:()=>o.reportValidity()}),W(o,"input",()=>n.emit("sui-input",{value:o.value})),W(o,"change",()=>n.emit("sui-input-change",{value:o.value}));let t=`sui-input-err-${++$o}`;st(E(o,"aria-describedby",()=>{let a=[];if(r.invalid()&&r.errorMessage()){let c=r.errorEl,f=typeof c==="function"?c():null;if(f)f.id=t,a.push(t)}let b=r.describedBy();if(b)a.push(b);return a.length>0?a.join(" "):null}))}});var Yd=j("sui-input",{props:{type:g.union(["text","search","email","url","tel","password","number"],{default:"text",reflect:!0}),value:g.string({default:""}),placeholder:g.string({default:""}),name:g.string({default:""}),autocomplete:g.string({default:""}),required:g.bool({default:!1,reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),readonly:g.bool({default:!1,reflect:!0}),invalid:g.bool({default:!1,reflect:!0}),loading:g.bool({default:!1,reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:g.union(["default","ghost"],{default:"default",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),label:g.string({default:""}),"described-by":g.string({default:""}),"error-message":g.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
  :host {
    display: inline-flex;
    flex-direction: column;
    gap: 4px;
    vertical-align: middle;
    width: 100%;
    --tone-border: var(--border);
    --tone-ring: var(--primary);
    --h: 34px;
    --fs: 13px;
    --px: 10px;
  }

  :host([tone='primary'])     { --tone-border: var(--primary);     --tone-ring: var(--primary); }
  :host([tone='info'])        { --tone-border: var(--info);         --tone-ring: var(--info); }
  :host([tone='success'])     { --tone-border: var(--success);      --tone-ring: var(--success); }
  :host([tone='warning'])     { --tone-border: var(--warning);      --tone-ring: var(--warning); }
  :host([tone='destructive']),
  :host([invalid])            { --tone-border: var(--destructive);  --tone-ring: var(--destructive); }
  :host([tone='accent'])      { --tone-border: var(--accent);       --tone-ring: var(--accent); }
  :host([tone='muted'])       { --tone-border: var(--muted-foreground); --tone-ring: var(--muted-foreground); }

  :host([size='sm']) { --h: 28px; --fs: 12px; --px: 8px; }
  :host([size='md']) { --h: 34px; --fs: 13px; --px: 10px; }
  :host([size='lg']) { --h: 40px; --fs: 14px; --px: 12px; }

  .wrap {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    min-height: var(--h);
    padding: 0 var(--px);
    background: var(--background);
    border: 1px solid var(--tone-border);
    border-radius: var(--radius-md);
    transition:
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  :host([variant='ghost']) .wrap {
    border-color: transparent;
    background: transparent;
  }

  .wrap:focus-within {
    border-color: var(--tone-ring);
    box-shadow: 0 0 0 3px color-mix(in oklch, var(--tone-ring) 25%, transparent);
  }

  :host([disabled]) .wrap {
    background: var(--muted);
    cursor: not-allowed;
    opacity: 0.65;
  }

  input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    padding: 0;
    font: inherit;
    font-family: var(--font-body);
    font-size: var(--fs);
    color: var(--foreground);
  }
  input::placeholder { color: var(--muted-foreground); }
  input:disabled { cursor: not-allowed; }

  ::slotted([slot='leading']),
  ::slotted([slot='trailing']) {
    display: inline-flex;
    align-items: center;
    flex: 0 0 auto;
    color: var(--muted-foreground);
  }

  .spinner {
    display: inline-block;
    flex: 0 0 auto;
    width: 14px;
    height: 14px;
    border: 2px solid color-mix(in oklch, var(--tone-ring) 30%, transparent);
    border-top-color: var(--tone-ring);
    border-radius: 50%;
    animation: sui-spin 0.6s linear infinite;
  }

  @keyframes sui-spin {
    to { transform: rotate(360deg); }
  }

  .error {
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--destructive);
    line-height: 1.4;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.type=n.type,t.value=n.value,t.placeholder=n.placeholder,t.name=n.name,t.autocomplete=n.autocomplete,t.required=n.required,t.disabled=n.disabled,t.readonly=n.readonly,t.invalid=n.invalid,t.loading=n.loading,t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.label=n.label,t.describedBy=n.describedBy,t.errorMessage=n.errorMessage;let a=null;if(dn&&typeof dn.setup==="function"){if(e.scope=t,a=dn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(s)=>H({anchor:s,test:()=>{if(t.loading())return 0;return 1},branches:[()=>{let m=document.createDocumentFragment(),h=[],v=document.createElement("span");return v.setAttribute("class","spinner"),v.setAttribute("aria-hidden","true"),m.appendChild(v),{nodes:Array.from(m.childNodes),cleanup:()=>{for(let y of h)try{y()}catch(w){}}}},()=>{let m=document.createDocumentFragment(),h=[],v=document.createElement("slot");return v.setAttribute("name","trailing"),h.push(tt(v,r,"data-no-trailing")),m.appendChild(v),{nodes:Array.from(m.childNodes),cleanup:()=>{for(let y of h)try{y()}catch(w){}}}}]}),c=(s)=>H({anchor:s,test:()=>{if(t.invalid()&&t.errorMessage())return 0;return-1},branches:[()=>{let m=document.createDocumentFragment(),h=[],v=document.createElement("span");v.setAttribute("class","error"),v.setAttribute("role","alert"),T(t,"errorEl",v);let y=document.createTextNode("");return v.appendChild(y),h.push(F(y,()=>t.errorMessage())),m.appendChild(v),{nodes:Array.from(m.childNodes),cleanup:()=>{for(let w of h)try{w()}catch(k){}}}}]}),f=r.shadowRoot??r;L(f);let p=document.createElement("div");p.setAttribute("class","wrap"),p.setAttribute("part","wrap");let d=document.createElement("slot");d.setAttribute("name","leading"),o.push(tt(d,r,"data-no-leading")),p.appendChild(d);let l=document.createElement("input");l.setAttribute("part","input"),o.push(E(l,"type",()=>t.type())),o.push(E(l,"placeholder",()=>t.placeholder())),o.push(E(l,"name",()=>t.name())),o.push(E(l,"autocomplete",()=>t.autocomplete()||"off")),o.push(E(l,"disabled",()=>t.disabled())),o.push(E(l,"readonly",()=>t.readonly())),o.push(E(l,"required",()=>t.required())),o.push(E(l,"aria-label",()=>t.label())),o.push(E(l,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(E(l,"aria-required",()=>t.required()?"true":"false")),o.push(E(l,"aria-busy",()=>t.loading()?"true":"false")),o.push(pt(l,t.value)),T(t,"inputEl",l),p.appendChild(l);let i=document.createComment("if");p.appendChild(i),o.push(b(i)),f.appendChild(p);let u=document.createComment("if");if(f.appendChild(u),o.push(c(u)),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let m=new Proxy(t,{get(h,v){return h[v]},set(h,v,y){let w=h[v];if(w!=null&&typeof w.set==="function"){if(typeof y==="function")w.set(()=>y);else w.set(y);return!0}return!1}});if(s.mount){let h=s.mount(m,r);if(typeof h==="function")o.push(h)}if(s.unmount)o.push(()=>s.unmount(m))}}return t}});var _o={git:"git-branch",refresh:"refresh-cw",stop:"stop-circle",warning:"triangle-alert"};var ut=null,St=null;function cn(){return ut}function Fr(){if(ut)return Promise.resolve(ut);if(St)return St;return St=fetch("/_sui/lucide-icons.json").then((e)=>{if(!e.ok)throw Error(`lucide registry fetch ${e.status}`);return e.json()}).then((e)=>{return ut=e,e}).catch((e)=>{throw St=null,e}),St}function Br(e){if(!e||!ut)return;if(ut[e])return e;let n=_o[e];if(n&&ut[n])return n;return}function Gr(e){return ut?ut[e]:void 0}var Ur="http://www.w3.org/2000/svg",_e=P(cn()?1:0),$e=!1;function ta(){if($e||cn())return;$e=!0,Fr().then(()=>_e.set(_e()+1)).catch(()=>{$e=!1})}function na(e){let n=e.createElementNS(Ur,"svg");return n.setAttribute("part","svg"),n.setAttribute("class","sui-icon-svg"),n.setAttribute("viewBox","0 0 24 24"),n.setAttribute("fill","none"),n.setAttribute("stroke","currentColor"),n.setAttribute("stroke-width","2"),n.setAttribute("stroke-linecap","round"),n.setAttribute("stroke-linejoin","round"),n}function ea(e,n){while(e.firstChild)e.removeChild(e.firstChild);let r=new DOMParser().parseFromString(`<svg xmlns="${Ur}">${n}</svg>`,"image/svg+xml").documentElement;while(r.firstChild)e.appendChild(e.ownerDocument.adoptNode(r.firstChild))}var pn=O({mount(e,n){let r=e,o=n.shadowRoot,t=null,a=null,b=D(()=>{let f=r.label();if(f)n.setAttribute("role","img"),n.setAttribute("aria-label",f),n.removeAttribute("aria-hidden");else n.setAttribute("aria-hidden","true"),n.removeAttribute("role"),n.removeAttribute("aria-label")}),c=D(()=>{_e();let f=r.name(),p=Br(f)??null,d=r.label();if(!p){if(t)t.remove(),t=null;if(a=null,f&&!cn())ta();return}if(!t)t=na(n.ownerDocument),o.insertBefore(t,o.firstChild);if(p!==a)ea(t,Gr(p)),a=p;if(d)t.removeAttribute("aria-hidden");else t.setAttribute("aria-hidden","true")});return()=>{b(),c()}}});var _d=j("sui-icon",{props:{name:g.string({default:"",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:g.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),label:g.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    /* default tone; suiTonesSheet (adopted by compiler) overrides per [tone] */
    --tone-fg: var(--foreground);
    color: var(--tone-fg);
  }

  /* Size matrix — applies to the controller-injected SVG */
  svg.sui-icon-svg {
    display: block;
    width: 16px;
    height: 16px;
    flex: 0 0 auto;
  }
  :host([size='sm']) svg.sui-icon-svg { width: 12px; height: 12px; }
  :host([size='lg']) svg.sui-icon-svg { width: 20px; height: 20px; }

  /* Slot passthrough: user-provided SVG fills the icon box */
  ::slotted(svg) { width: 100%; height: 100%; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.name=n.name,t.size=n.size,t.tone=n.tone,t.label=n.label;let a=null;if(pn&&typeof pn.setup==="function"){if(e.scope=t,a=pn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});var fn=O({mount(e,n){let r=e,o=()=>Array.from(n.querySelectorAll(":scope > option, :scope > optgroup > option")),t=()=>{let u=r.value(),s=o();if(s.length===0)return u;let m=s.find((v)=>v.value===u)??s.find((v)=>v.hasAttribute("selected"))??s[0];return(m?.textContent??"").trim()||m?.value||u},a=()=>{let u=r.selectEl();if(!u){let m=o(),h=m.find((v)=>v.value===r.value())??m.find((v)=>v.hasAttribute("selected"))??m[0];if(!h||h.value==="")n.setAttribute("data-placeholder","");else n.removeAttribute("data-placeholder");return}let s=u.selectedOptions?.[0]??u.options[u.selectedIndex];if(!s||s.value==="")n.setAttribute("data-placeholder","");else n.removeAttribute("data-placeholder")},b=()=>{let u=r.selectEl();if(!u)return;while(u.firstChild)u.removeChild(u.firstChild);let s=document.createDocumentFragment(),m=n.querySelectorAll(":scope > option, :scope > optgroup");for(let h of Array.from(m))s.appendChild(h.cloneNode(!0));u.appendChild(s),u.value=r.value()},c=()=>{let u=r.valueLabel();if(u)u.textContent=t()},f=D(()=>{let u=r.value(),s=r.selectEl();if(s)s.value=u;c(),a()}),p=(u)=>{let s=u.target.value;r.value.set(s),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:s},bubbles:!0,composed:!0}))},d=r.selectEl();if(d)d.addEventListener("change",p);let l=r.optionsSlot(),i=()=>{b(),c(),a()};if(l)l.addEventListener("slotchange",i);return b(),c(),a(),()=>{if(d)d.removeEventListener("change",p);if(l)l.removeEventListener("slotchange",i);f()}}});var ac=j("sui-select",{props:{value:g.string({default:""}),name:g.string({default:""}),required:g.bool({default:!1,reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),invalid:g.bool({default:!1,reflect:!0}),tone:g.union(["default","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),label:g.string({default:""}),"described-by":g.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
  :host {
    display: inline-flex;
    vertical-align: middle;
    --tone: var(--foreground);
    --tone-fg: var(--foreground);
  }

  :host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info); }
  :host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) { --sel-h: 34px; --sel-px: 12px; --sel-fs: 13px; --sel-gap: 8px; --sel-chev: 14px; }
  :host([size='sm']) { --sel-h: 28px; --sel-px: 10px; --sel-fs: 12px; --sel-gap: 6px; --sel-chev: 12px; }
  :host([size='lg']) { --sel-h: 40px; --sel-px: 16px; --sel-fs: 14px; --sel-gap: 8px; --sel-chev: 16px; }

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

  :host(:not([disabled])) .field:hover {
    background: color-mix(in oklch, var(--muted) 70%, var(--background));
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
    box-shadow: var(--shadow-sm);
  }

  :host([tone]:not([tone='default'])) .field {
    border-color: color-mix(in oklch, var(--tone) 38%, var(--border));
  }
  :host([tone]:not([tone='default']):not([disabled])) .field:hover {
    background: color-mix(in oklch, var(--tone) 8%, var(--background));
    border-color: color-mix(in oklch, var(--tone) 55%, var(--border));
  }

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
  select option { color: var(--foreground); background: var(--background); }

  .value {
    display: inline-block;
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
  }
  :host([data-placeholder]) .value { color: var(--muted-foreground); }

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
  :host(:not([disabled])) .field:hover .chevron { color: var(--foreground); }
  :host([tone]:not([tone='default'])) .chevron {
    color: color-mix(in oklch, var(--tone-fg) 75%, var(--muted-foreground));
  }

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

  :host([invalid]) .field { border-color: var(--destructive); }
  :host([invalid]:focus-within) .field {
    border-color: var(--destructive);
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--destructive) var(--ring-opacity), transparent);
  }

  :host([disabled]) { opacity: 0.6; cursor: not-allowed; }
  :host([disabled]) .field {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 80%, transparent);
    box-shadow: none;
    cursor: not-allowed;
    filter: saturate(0.6);
  }
  :host([disabled]) select { cursor: not-allowed; }

  @media (forced-colors: active) {
    :host(:focus-within) .field { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
  @media (prefers-reduced-motion: reduce) {
    .field, .chevron { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.name=n.name,t.required=n.required,t.disabled=n.disabled,t.invalid=n.invalid,t.tone=n.tone,t.size=n.size,t.label=n.label,t.describedBy=n.describedBy;let a=null;if(fn&&typeof fn.setup==="function"){if(e.scope=t,a=fn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","field"),c.setAttribute("part","field");let f=document.createElement("span");f.setAttribute("class","value"),f.setAttribute("aria-hidden","true"),T(t,"valueLabel",f),c.appendChild(f);let p=document.createElement("svg");p.setAttribute("class","chevron"),p.setAttribute("part","chevron"),p.setAttribute("viewbox","0 0 16 16"),p.setAttribute("aria-hidden","true"),p.setAttribute("focusable","false"),p.setAttribute("fill","none"),p.setAttribute("stroke","currentColor"),p.setAttribute("stroke-width","1.75"),p.setAttribute("stroke-linecap","round"),p.setAttribute("stroke-linejoin","round");let d=document.createElement("path");d.setAttribute("d","M4 6l4 4 4-4"),p.appendChild(d),c.appendChild(p);let l=document.createElement("select");l.setAttribute("part","select"),o.push(E(l,"name",()=>t.name())),o.push(E(l,"disabled",()=>t.disabled())),o.push(E(l,"required",()=>t.required())),o.push(E(l,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(E(l,"aria-label",()=>t.label())),o.push(E(l,"aria-describedby",()=>t.describedBy())),T(t,"selectEl",l),c.appendChild(l),b.appendChild(c);let i=document.createElement("span");i.setAttribute("hidden","");let u=document.createElement("slot");if(T(t,"optionsSlot",u),i.appendChild(u),b.appendChild(i),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let m=new Proxy(t,{get(h,v){return h[v]},set(h,v,y){let w=h[v];if(w!=null&&typeof w.set==="function"){if(typeof y==="function")w.set(()=>y);else w.set(y);return!0}return!1}});if(s.mount){let h=s.mount(m,r);if(typeof h==="function")o.push(h)}if(s.unmount)o.push(()=>s.unmount(m))}}return t}});var uc=j("sui-separator",{props:{orientation:g.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:g.union(["solid","dashed"],{default:"solid",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
  }

  .sep {
    border: 0;
    margin: 0;
    padding: 0;
    flex: 0 0 auto;
  }

  /* Horizontal (default) */
  :host(:not([orientation='vertical'])) .sep {
    display: block;
    width: 100%;
    height: 1px;
    margin: 8px 0;
    background: var(--border);
  }
  :host(:not([orientation='vertical'])[variant='dashed']) .sep {
    background: transparent;
    border-top: 1px dashed var(--border);
    height: 0;
  }

  /* Vertical */
  :host([orientation='vertical']) {
    display: inline-block;
    align-self: stretch;
  }
  :host([orientation='vertical']) .sep {
    display: inline-block;
    width: 1px;
    min-height: 100%;
    margin: 0 8px;
    background: var(--border);
  }
  :host([orientation='vertical'][variant='dashed']) .sep {
    background: transparent;
    border-left: 1px dashed var(--border);
    width: 0;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.orientation=n.orientation,t.variant=n.variant;let a=r.shadowRoot??r;L(a);let b=document.createElement("hr");return b.setAttribute("class","sep"),b.setAttribute("part","sep"),b.setAttribute("role","separator"),o.push(E(b,"aria-orientation",()=>t.orientation())),a.appendChild(b),t}});var mn=O({mount(e,n){let r=e,o=D(()=>{let t=r.labelEl();if(!t)return;t.htmlFor=r.for?.()??""});return()=>{o()}}});var mc=j("sui-label",{props:{for:g.string({default:""}),size:g.union(["sm","md"],{default:"sm",reflect:!0}),required:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-block;
  }

  label {
    display: inline;
    font-family: var(--font-body);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    cursor: pointer;
  }

  :host(:not([size])) label,
  :host([size='sm']) label { font-size: 10px; }
  :host([size='md']) label { font-size: 11px; }

  .req {
    color: var(--destructive);
    margin-left: 2px;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.for=n.for,t.size=n.size,t.required=n.required;let a=null;if(mn&&typeof mn.setup==="function"){if(e.scope=t,a=mn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(l)=>H({anchor:l,test:()=>{if(t.required())return 0;return-1},branches:[()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("span");s.setAttribute("class","req"),s.setAttribute("aria-hidden","true");let m=document.createTextNode("*");return s.appendChild(m),i.appendChild(s),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let h of u)try{h()}catch(v){}}}}]}),c=r.shadowRoot??r;L(c);let f=document.createElement("label");f.setAttribute("part","label"),T(t,"labelEl",f);let p=document.createElement("slot");f.appendChild(p);let d=document.createComment("if");if(f.appendChild(d),o.push(b(d)),c.appendChild(f),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(u,s){return u[s]},set(u,s,m){let h=u[s];if(h!=null&&typeof h.set==="function"){if(typeof m==="function")h.set(()=>m);else h.set(m);return!0}return!1}});if(l.mount){let u=l.mount(i,r);if(typeof u==="function")o.push(u)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var bn=O({mount(e,n){let r=e,o=D(()=>{let t=r.closeBtn();if(!t)return;let a=()=>{if(!r.dismissible())return;n.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),n.remove()};return t.addEventListener("click",a),()=>t.removeEventListener("click",a)});return()=>{o()}}});var xc=j("sui-alert",{props:{tone:g.union(["info","success","warning","destructive","muted"],{default:"info",reflect:!0}),dismissible:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    --tone: var(--info);
    --tone-fg: var(--info);
    --alert-bg: color-mix(in oklch, var(--tone) 12%, var(--background));
    --alert-border: color-mix(in oklch, var(--tone) 30%, transparent);
  }

  :host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  .box {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius);
    border: 1px solid var(--alert-border);
    background: var(--alert-bg);
    color: var(--tone-fg);
    font-family: var(--font-body);
    font-size: 13px;
    line-height: 1.5;
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .title { font-weight: 600; }

  .body { color: var(--foreground); }

  .actions {
    display: flex;
    gap: 6px;
    margin-top: 6px;
  }

  .close {
    display: none;
    background: transparent;
    border: 0;
    color: inherit;
    cursor: pointer;
    font-size: 14px;
    padding: 2px 6px;
    border-radius: 4px;
    align-items: center;
    justify-content: center;
    line-height: 1;
  }
  :host([dismissible]) .close {
    display: inline-flex;
  }
  .close:hover {
    background: color-mix(in oklch, var(--tone-fg) 12%, transparent);
  }

  ::slotted([slot='icon']) { display: inline-flex; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.dismissible=n.dismissible;let a=null;if(bn&&typeof bn.setup==="function"){if(e.scope=t,a=bn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","box"),c.setAttribute("part","box"),o.push(E(c,"role",()=>t.tone()==="destructive"||t.tone()==="warning"?"alert":"status")),o.push(E(c,"aria-live",()=>t.tone()==="destructive"||t.tone()==="warning"?"assertive":"polite")),c.setAttribute("aria-atomic","true");let f=document.createElement("span"),p=document.createElement("slot");p.setAttribute("name","icon"),f.appendChild(p),c.appendChild(f);let d=document.createElement("div");d.setAttribute("class","content");let l=document.createElement("div");l.setAttribute("class","title");let i=document.createElement("slot");i.setAttribute("name","title"),l.appendChild(i),d.appendChild(l);let u=document.createElement("div");u.setAttribute("class","body");let s=document.createElement("slot");u.appendChild(s),d.appendChild(u);let m=document.createElement("div");m.setAttribute("class","actions");let h=document.createElement("slot");h.setAttribute("name","actions"),m.appendChild(h),d.appendChild(m),c.appendChild(d);let v=document.createElement("button");v.setAttribute("class","close"),v.setAttribute("part","close"),v.setAttribute("aria-label","Dismiss"),T(t,"closeBtn",v);let y=document.createTextNode("×");if(v.appendChild(y),c.appendChild(v),b.appendChild(c),a!=null){let w=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(w){let k=new Proxy(t,{get(A,x){return A[x]},set(A,x,C){let z=A[x];if(z!=null&&typeof z.set==="function"){if(typeof C==="function")z.set(()=>C);else z.set(C);return!0}return!1}});if(w.mount){let A=w.mount(k,r);if(typeof A==="function")o.push(A)}if(w.unmount)o.push(()=>w.unmount(k))}}return t}});var ra=0,hn=O({mount(e,n){let r=e,o=`sui-tooltip-${++ra}`,t=D(()=>{let f=r.bubbleEl();if(f)f.id=o}),a=()=>{let f=r.defaultSlot();if(!f)return;let p=f.assignedElements({flatten:!0})[0];if(!p)return;let d=p.getAttribute("aria-describedby")??"";if(!d.split(/\s+/).includes(o))p.setAttribute("aria-describedby",d?`${d} ${o}`:o)},b=D(()=>{let f=r.defaultSlot();if(!f)return;return f.addEventListener("slotchange",a),a(),()=>f.removeEventListener("slotchange",a)}),c=(f)=>{if(f.key==="Escape")n.blur()};return n.addEventListener("keydown",c),()=>{t(),b(),n.removeEventListener("keydown",c)}}});var zc=j("sui-tooltip",{props:{placement:g.union(["top","bottom","left","right"],{default:"top",reflect:!0})},shadow:!0,styles:`
  :host {
    position: relative;
    display: inline-flex;
  }

  .bubble {
    position: absolute;
    z-index: var(--z-popover, 50);
    padding: 4px 8px;
    background: var(--foreground);
    color: var(--background);
    font-family: var(--font-body);
    font-size: 11px;
    font-weight: var(--font-weight-medium, 500);
    border-radius: var(--radius-sm);
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transition: opacity var(--duration-fast, 120ms) var(--ease-out, ease-out);
    box-shadow: var(--shadow-sm);
  }

  :host(:hover) .bubble,
  :host(:focus-within) .bubble {
    opacity: 1;
  }

  /* Placement: top (default) */
  :host(:not([placement])) .bubble,
  :host([placement='top']) .bubble {
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.placement=n.placement;let a=null;if(hn&&typeof hn.setup==="function"){if(e.scope=t,a=hn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");T(t,"defaultSlot",c),b.appendChild(c);let f=document.createElement("span");f.setAttribute("class","bubble"),f.setAttribute("role","tooltip"),f.setAttribute("part","bubble"),T(t,"bubbleEl",f);let p=document.createElement("slot");if(p.setAttribute("name","content"),f.appendChild(p),b.appendChild(f),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var oa=0,vn=O({mount(e,n){let r=e,o=`sui-collapsible-${++oa}`,t=D(()=>{let l=r.contentWrap();if(l)l.id=o}),a=()=>{let l=r.triggerSlot();if(!l)return;let i=r.open();for(let u of l.assignedElements()){let s=u;if(s.setAttribute("aria-expanded",String(i)),s.setAttribute("aria-controls",o),!s.hasAttribute("tabindex")&&!s.matches("button, a, input, select, textarea"))s.setAttribute("tabindex","0")}},b=D(()=>{let l=r.triggerSlot();if(!l)return;return l.addEventListener("slotchange",a),a(),()=>l.removeEventListener("slotchange",a)}),c=D(()=>{r.open(),r.triggerSlot(),a()}),f=()=>{if(r.disabled())return;let l=!r.open();r.open.set(l),n.dispatchEvent(new CustomEvent("sui-toggle",{detail:{open:l},bubbles:!0,composed:!0}))},p=(l)=>{if(l.key==="Enter"||l.key===" ")l.preventDefault(),f()},d=D(()=>{let l=r.triggerWrap();if(!l)return;return l.addEventListener("click",f),l.addEventListener("keydown",p),()=>{l.removeEventListener("click",f),l.removeEventListener("keydown",p)}});return()=>{t(),b(),c(),d()}}});var Mc=j("sui-collapsible",{props:{open:g.bool({default:!1,reflect:!0}),disabled:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    color: var(--foreground);
  }

  .content-wrap {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows var(--duration-medium, 200ms) var(--ease-out, ease-out);
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

  @media (prefers-reduced-motion: reduce) {
    .content-wrap { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.disabled=n.disabled;let a=null;if(vn&&typeof vn.setup==="function"){if(e.scope=t,a=vn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","trigger-wrap"),T(t,"triggerWrap",c);let f=document.createElement("slot");f.setAttribute("name","trigger"),T(t,"triggerSlot",f),c.appendChild(f),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","content-wrap"),p.setAttribute("part","content"),T(t,"contentWrap",p);let d=document.createElement("div");d.setAttribute("class","content-inner");let l=document.createElement("slot");if(d.appendChild(l),p.appendChild(d),b.appendChild(p),a!=null){let i=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(i){let u=new Proxy(t,{get(s,m){return s[m]},set(s,m,h){let v=s[m];if(v!=null&&typeof v.set==="function"){if(typeof h==="function")v.set(()=>h);else v.set(h);return!0}return!1}});if(i.mount){let s=i.mount(u,r);if(typeof s==="function")o.push(s)}if(i.unmount)o.push(()=>i.unmount(u))}}return t}});var gn=O({mount(e,n){let r=e,o=D(()=>{let a=r.inputEl();if(!a)return;a.indeterminate=r.indeterminate()}),t=D(()=>{let a=r.inputEl();if(!a)return;let b=()=>{r.indeterminate.set(!1),n.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:a.checked,value:r.value()},bubbles:!0,composed:!0}))};return a.addEventListener("change",b),()=>a.removeEventListener("change",b)});return()=>{o(),t()}}});var Bc=j("sui-checkbox",{props:{checked:g.bool({default:!1,reflect:!0}),indeterminate:g.bool({default:!1,reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),required:g.bool({default:!1,reflect:!0}),invalid:g.bool({default:!1,reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),name:g.string({default:""}),value:g.string({default:"on"}),label:g.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
  :host {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    vertical-align: middle;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);
    -webkit-tap-highlight-color: transparent;

    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }
  :host([tone='info'])        { --tone: var(--info);        --tone-on-fg: var(--info-foreground); }
  :host([tone='success'])     { --tone: var(--success);     --tone-on-fg: var(--success-foreground); }
  :host([tone='warning'])     { --tone: var(--warning);     --tone-on-fg: var(--warning-foreground); }
  :host([tone='destructive']) { --tone: var(--destructive); --tone-on-fg: var(--destructive-foreground); }
  :host([tone='accent'])      { --tone: var(--accent);      --tone-on-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }
  :host([tone='primary'])     { --tone: var(--primary);     --tone-on-fg: var(--primary-foreground); }

  :host(:not([size])),
  :host([size='md']) { --cb-size: 16px; --cb-radius: var(--radius-sm, 4px); --cb-stroke: 2px; }
  :host([size='sm']) { --cb-size: 14px; --cb-radius: 3px;                   --cb-stroke: 1.75px; }
  :host([size='lg']) { --cb-size: 18px; --cb-radius: var(--radius-sm, 4px); --cb-stroke: 2.25px; }

  label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
  }
  :host([disabled]) label { cursor: not-allowed; }

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
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }

  :host(:not([disabled]):not([checked]):not([indeterminate])) label:hover .box {
    border-color: color-mix(in oklch, var(--border) 220%, transparent);
    box-shadow: var(--shadow-sm);
  }

  :host([invalid]) .box {
    border-color: color-mix(in oklch, var(--destructive) 70%, var(--border));
  }
  :host([invalid]:not([disabled])) label:hover .box {
    border-color: color-mix(in oklch, var(--destructive) 85%, var(--border));
  }

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

  /* Glyphs — CSS-drawn check and indeterminate indicator. */
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
      opacity var(--duration-fast, 150ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 150ms) var(--ease-out, ease-out);
    pointer-events: none;
  }
  :host([checked]:not([indeterminate])) .glyph.check,
  :host([indeterminate]) .glyph.mixed {
    opacity: 1;
    transform: scale(1);
  }

  /* Check mark: rotated L-shape via borders */
  .glyph.check::before {
    content: '';
    display: block;
    width: 45%;
    height: 28%;
    border-left: var(--cb-stroke) solid currentColor;
    border-bottom: var(--cb-stroke) solid currentColor;
    transform: rotate(-45deg) translate(0, -15%);
  }

  /* Indeterminate bar */
  .glyph.mixed::before {
    content: '';
    display: block;
    width: 60%;
    height: var(--cb-stroke);
    background: currentColor;
    border-radius: 1px;
  }

  /* Native input — visually hidden but preserved for a11y + form submission */
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

  :host(:not([disabled])) label:has(input:focus-visible) .box {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }
  :host([invalid]:not([disabled])) label:has(input:focus-visible) .box {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--destructive) var(--ring-opacity, 50%), transparent);
  }

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


  :host([disabled]) { color: var(--muted-foreground); }
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

  ::slotted(*) {
    font: inherit;
    color: inherit;
    line-height: var(--leading-tight, 1.25);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.checked=n.checked,t.indeterminate=n.indeterminate,t.disabled=n.disabled,t.required=n.required,t.invalid=n.invalid,t.tone=n.tone,t.size=n.size,t.name=n.name,t.value=n.value,t.label=n.label;let a=null;if(gn&&typeof gn.setup==="function"){if(e.scope=t,a=gn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("label");c.setAttribute("part","label");let f=document.createElement("span");f.setAttribute("class","box"),f.setAttribute("part","box");let p=document.createElement("input");p.setAttribute("part","input"),p.setAttribute("type","checkbox"),o.push(ft(p,t.checked)),o.push(E(p,"disabled",()=>t.disabled())),o.push(E(p,"required",()=>t.required())),o.push(E(p,"name",()=>t.name())),o.push(E(p,"value",()=>t.value())),o.push(E(p,"aria-checked",()=>t.indeterminate()?"mixed":t.checked()?"true":"false")),o.push(E(p,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(E(p,"aria-label",()=>t.label()||t.name()||"checkbox")),T(t,"inputEl",p),f.appendChild(p);let d=document.createElement("span");d.setAttribute("class","glyph check"),d.setAttribute("aria-hidden","true"),f.appendChild(d);let l=document.createElement("span");l.setAttribute("class","glyph mixed"),l.setAttribute("aria-hidden","true"),f.appendChild(l),c.appendChild(f);let i=document.createElement("slot");if(c.appendChild(i),b.appendChild(c),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(m,h){return m[h]},set(m,h,v){let y=m[h];if(y!=null&&typeof y.set==="function"){if(typeof v==="function")y.set(()=>v);else y.set(v);return!0}return!1}});if(u.mount){let m=u.mount(s,r);if(typeof m==="function")o.push(m)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var yn=O({mount(e,n){let r=e,o=D(()=>{let t=r.inputEl();if(!t)return;let a=()=>{n.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:t.checked},bubbles:!0,composed:!0}))};return t.addEventListener("change",a),()=>t.removeEventListener("change",a)});return()=>o()}});var Jc=j("sui-switch",{props:{checked:g.bool({default:!1,reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),name:g.string({default:""}),value:g.string({default:"on"}),label:g.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
  :host {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    vertical-align: middle;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);
    -webkit-tap-highlight-color: transparent;

    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
  }
  :host([tone='info'])        { --tone: var(--info);        --tone-on-fg: var(--info-foreground); }
  :host([tone='success'])     { --tone: var(--success);     --tone-on-fg: var(--success-foreground); }
  :host([tone='warning'])     { --tone: var(--warning);     --tone-on-fg: var(--warning-foreground); }
  :host([tone='destructive']) { --tone: var(--destructive); --tone-on-fg: var(--destructive-foreground); }
  :host([tone='accent'])      { --tone: var(--accent);      --tone-on-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }
  :host([tone='primary'])     { --tone: var(--primary);     --tone-on-fg: var(--primary-foreground); }

  :host(:not([size])),
  :host([size='md']) { --sw-w: 28px; --sw-h: 16px; --sw-thumb: 12px; }
  :host([size='sm']) { --sw-w: 24px; --sw-h: 14px; --sw-thumb: 10px; }
  :host([size='lg']) { --sw-w: 32px; --sw-h: 18px; --sw-thumb: 14px; }

  label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
  }
  :host([disabled]) label { cursor: not-allowed; }

  .track {
    position: relative;
    display: inline-block;
    flex-shrink: 0;
    width: var(--sw-w);
    height: var(--sw-h);
    background: var(--muted);
    border: 1px solid color-mix(in oklch, var(--border) 140%, transparent);
    border-radius: var(--radius-full, 9999px);
    box-sizing: border-box;
    transition:
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }

  :host(:not([disabled]):not([checked])) label:hover .track {
    background: color-mix(in oklch, var(--muted) 70%, var(--foreground) 6%);
    border-color: color-mix(in oklch, var(--border) 180%, transparent);
  }

  :host([checked]) .track {
    background: var(--tone);
    border-color: color-mix(in oklch, var(--tone) 100%, black 6%);
    box-shadow: inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent);
  }
  :host([checked]:not([disabled])) label:hover .track {
    background: color-mix(in oklch, var(--tone) 92%, white 8%);
    box-shadow:
      inset 0 1px 0 0 color-mix(in oklch, white 26%, transparent),
      var(--shadow-xs);
  }

  .thumb {
    position: absolute;
    top: 50%;
    left: 1px;
    width: var(--sw-thumb);
    height: var(--sw-thumb);
    background: var(--background);
    border-radius: 50%;
    box-shadow: var(--shadow-sm);
    transform: translateY(-50%);
    transition:
      left var(--duration-fast, 150ms) var(--ease-out, ease-out),
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
    pointer-events: none;
  }
  :host([checked]) .thumb {
    left: calc(100% - var(--sw-thumb) - 3px);
    background: var(--tone-on-fg);
  }

  /* Native input — visually hidden but preserved for a11y + form submission */
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

  :host(:not([disabled])) label:has(input:focus-visible) .track {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }
  :host([checked]:not([disabled])) label:has(input:focus-visible) .track {
    box-shadow:
      inset 0 1px 0 0 color-mix(in oklch, white 22%, transparent),
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }

  @media (forced-colors: active) {
    :host(:not([disabled])) label:has(input:focus-visible) .track {
      outline: 2px solid CanvasText;
      outline-offset: 2px;
    }
    .track { border-color: CanvasText; }
    :host([checked]) .track { background: Highlight; }
  }


  :host([disabled]) { color: var(--muted-foreground); }
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
  :host([disabled]) .thumb { box-shadow: var(--shadow-xs); }

  ::slotted(*) {
    font: inherit;
    color: inherit;
    line-height: var(--leading-tight, 1.25);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.checked=n.checked,t.disabled=n.disabled,t.tone=n.tone,t.size=n.size,t.name=n.name,t.value=n.value,t.label=n.label;let a=null;if(yn&&typeof yn.setup==="function"){if(e.scope=t,a=yn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("label"),f=document.createElement("span");f.setAttribute("class","track"),f.setAttribute("part","track");let p=document.createElement("input");p.setAttribute("part","input"),p.setAttribute("type","checkbox"),p.setAttribute("role","switch"),o.push(ft(p,t.checked)),o.push(E(p,"disabled",()=>t.disabled())),o.push(E(p,"name",()=>t.name())),o.push(E(p,"value",()=>t.value())),o.push(E(p,"aria-checked",()=>t.checked()?"true":"false")),o.push(E(p,"aria-label",()=>t.label()||t.name()||"toggle")),T(t,"inputEl",p),f.appendChild(p);let d=document.createElement("span");d.setAttribute("class","thumb"),d.setAttribute("part","thumb"),f.appendChild(d),c.appendChild(f);let l=document.createElement("slot");if(c.appendChild(l),b.appendChild(c),a!=null){let i=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(i){let u=new Proxy(t,{get(s,m){return s[m]},set(s,m,h){let v=s[m];if(v!=null&&typeof v.set==="function"){if(typeof h==="function")v.set(()=>h);else v.set(h);return!0}return!1}});if(i.mount){let s=i.mount(u,r);if(typeof s==="function")o.push(s)}if(i.unmount)o.push(()=>i.unmount(u))}}return t}});var xn=O({mount(e,n){let r=e,o=r.value(),t=D(()=>{let a=r.textareaEl();if(!a)return;let b=()=>{n.dispatchEvent(new CustomEvent("sui-input",{detail:{value:a.value},bubbles:!0,composed:!0}))},c=()=>{let f=r.value();if(f!==o)o=f,n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:f},bubbles:!0,composed:!0}))};return a.addEventListener("input",b),a.addEventListener("blur",c),()=>{a.removeEventListener("input",b),a.removeEventListener("blur",c)}});return()=>t()}});var Kc=j("sui-textarea",{props:{value:g.string({default:""}),placeholder:g.string({default:""}),name:g.string({default:""}),rows:g.number({default:4}),required:g.bool({default:!1,reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),readonly:g.bool({default:!1,reflect:!0}),invalid:g.bool({default:!1,reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:g.union(["default","code"],{default:"default",reflect:!0}),size:g.union(["sm","md"],{default:"md",reflect:!0}),resize:g.union(["none","vertical","horizontal","both"],{default:"vertical"}),label:g.string({default:""}),"described-by":g.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
  :host {
    display: block;
    width: 100%;
    --tone: var(--ring-color, var(--primary));
    --tone-fg: var(--foreground);
  }

  :host(:not([tone])),
  :host([tone='default']) {
    --tone: var(--ring-color, var(--primary));
    --tone-fg: var(--foreground);
  }
  :host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info); }
  :host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) {
    --ta-px: 12px;
    --ta-py: 10px;
    --ta-fs: var(--text-base, 14px);
    --ta-lh: 1.5;
    --ta-radius: var(--radius-md, 6px);
  }
  :host([size='sm']) {
    --ta-px: 10px;
    --ta-py: 8px;
    --ta-fs: var(--text-sm, 13px);
    --ta-lh: 1.45;
    --ta-radius: var(--radius-sm, 4px);
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
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
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
      0 0 0 var(--ring-width, 2px)
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
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
      0 0 0 var(--ring-width, 2px)
        color-mix(in oklch, var(--destructive) var(--ring-opacity, 50%), transparent);
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
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: var(--text-base, 14px);
    line-height: 1.55;
    letter-spacing: 0;
    tab-size: 2;
  }

  @media (forced-colors: active) {
    textarea:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.placeholder=n.placeholder,t.name=n.name,t.rows=n.rows,t.required=n.required,t.disabled=n.disabled,t.readonly=n.readonly,t.invalid=n.invalid,t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.resize=n.resize,t.label=n.label,t.describedBy=n.describedBy;let a=null;if(xn&&typeof xn.setup==="function"){if(e.scope=t,a=xn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("textarea");if(c.setAttribute("part","textarea"),o.push(E(c,"placeholder",()=>t.placeholder())),o.push(E(c,"name",()=>t.name())),o.push(E(c,"rows",()=>t.rows())),o.push(E(c,"disabled",()=>t.disabled())),o.push(E(c,"required",()=>t.required())),o.push(E(c,"readonly",()=>t.readonly())),o.push(E(c,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(E(c,"aria-required",()=>t.required()?"true":"false")),o.push(E(c,"aria-label",()=>t.label())),o.push(E(c,"aria-describedby",()=>t.describedBy())),o.push(D(()=>{let f=t.resize();if(f==null)c.style.removeProperty("resize");else c.style.setProperty("resize",String(f))})),o.push(pt(c,t.value)),T(t,"textareaEl",c),b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});var Xr=0,wn=O({mount(e,n){let r=e,o=`sui-field-hint-${++Xr}`,t=`sui-field-error-${++Xr}`,a=D(()=>{let p=r.hintEl(),d=r.errorEl();if(p)p.id=o;if(d)d.id=t}),b=()=>{let d=r.controlSlot()?.assignedElements({flatten:!0})[0];if(!d)return;let l=r.invalid(),i=l?t:o,s=[...(d.getAttribute("aria-describedby")||"").split(/\s+/).filter((m)=>m&&m!==o&&m!==t),i].join(" ").trim();if(s)d.setAttribute("aria-describedby",s);if("invalid"in d)d.invalid=l;else d.setAttribute("aria-invalid",l?"true":"false")},c=D(()=>{r.invalid(),b()}),f=D(()=>{let p=r.controlSlot();if(!p)return;return p.addEventListener("slotchange",b),()=>p.removeEventListener("slotchange",b)});return()=>{a(),c(),f()}}});var ep=j("sui-field",{props:{invalid:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .hint,
  .error {
    font-family: var(--font-body);
    font-size: 11px;
    line-height: 1.4;
  }
  .hint { color: var(--muted-foreground); }
  .error { color: var(--destructive); }

  :host(:not([invalid])) .error { display: none; }
  :host([invalid]) .hint { display: none; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.invalid=n.invalid;let a=null;if(wn&&typeof wn.setup==="function"){if(e.scope=t,a=wn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");c.setAttribute("name","label"),b.appendChild(c);let f=document.createElement("slot");T(t,"controlSlot",f),b.appendChild(f);let p=document.createElement("div");p.setAttribute("class","hint"),T(t,"hintEl",p);let d=document.createElement("slot");d.setAttribute("name","hint"),p.appendChild(d),b.appendChild(p);let l=document.createElement("div");l.setAttribute("class","error"),l.setAttribute("role","alert"),l.setAttribute("aria-live","polite"),T(t,"errorEl",l);let i=document.createElement("slot");if(i.setAttribute("name","error"),l.appendChild(i),b.appendChild(l),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(m,h){return m[h]},set(m,h,v){let y=m[h];if(y!=null&&typeof y.set==="function"){if(typeof v==="function")y.set(()=>v);else y.set(v);return!0}return!1}});if(u.mount){let m=u.mount(s,r);if(typeof m==="function")o.push(m)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var An=O({scope:(e,n)=>{let r=e,o=Y(null),t=J(()=>r.tone()==="destructive"?"destructive":"default");function a(f){n.dispatchEvent(new CustomEvent(f,{bubbles:!0,composed:!0})),r.open.set(!1)}function b(){a("sui-confirm")}function c(){a("sui-cancel")}return{dialogRef:o,confirmTone:t,onConfirm:b,onCancel:c}},mount(e,n){let r=e,o=r.open,t=r.dialogRef,a=D(()=>{let c=o(),f=t();if(!f)return;if(c&&typeof f.show==="function")f.show();else if(!c&&typeof f.close==="function")f.close()}),b=()=>{if(o())o.set(!1)};return n.addEventListener("sui-close",b),()=>{a(),n.removeEventListener("sui-close",b)}}});var up=j("sui-alert-dialog",{props:{open:g.bool({default:!1,reflect:!0}),tone:g.union(["default","destructive"],{default:"default",reflect:!0}),"confirm-label":g.string({default:"Confirm"}),"cancel-label":g.string({default:"Cancel"})},shadow:!0,styles:`
  :host { display: contents; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.tone=n.tone,t.confirmLabel=n.confirmLabel,t.cancelLabel=n.cancelLabel;let a=null;if(An&&typeof An.setup==="function"){if(e.scope=t,a=An.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("sui-dialog");c.setAttribute("role-override","alertdialog"),c.setAttribute("size","sm"),c.setAttribute("no-close-button",""),T(t,"dialogRef",c);let f=document.createElement("span");f.setAttribute("slot","header");let p=document.createElement("slot");p.setAttribute("name","title"),f.appendChild(p),c.appendChild(f);let d=document.createElement("slot");c.appendChild(d);let l=document.createElement("span");l.setAttribute("slot","footer");let i=document.createElement("sui-button");i.setAttribute("variant","outline"),o.push(V(i,"click",function(){let h=t.onCancel;return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let u=document.createTextNode("");i.appendChild(u),o.push(F(u,()=>t.cancelLabel())),l.appendChild(i);let s=document.createElement("sui-button");s.setAttribute("variant","solid"),o.push(E(s,"tone",()=>t.confirmTone())),o.push(V(s,"click",function(){let h=t.onConfirm;return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let m=document.createTextNode("");if(s.appendChild(m),o.push(F(m,()=>t.confirmLabel())),l.appendChild(s),c.appendChild(l),b.appendChild(c),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(y,w){return y[w]},set(y,w,k){let A=y[w];if(A!=null&&typeof A.set==="function"){if(typeof k==="function")A.set(()=>k);else A.set(k);return!0}return!1}});if(h.mount){let y=h.mount(v,r);if(typeof y==="function")o.push(y)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});function Rr(e){if(e<1024)return`${e} B`;if(e<1048576)return`${(e/1024).toFixed(1)} KB`;return`${(e/1048576).toFixed(1)} MB`}var kn=O({scope:()=>{let e=P([]),n=P(!1),r=P(""),o=Y(null),t=Y(null),a=Y(null),b=J(()=>e().map((c,f)=>({file:c,idx:f,key:`${f}:${c.name}:${c.size}`,name:c.name,sizeStr:Rr(c.size)})));return{files:e,over:n,errorMsg:r,displayFiles:b,dropzoneEl:o,inputEl:t,fileListEl:a}},mount(e,n){let r=e,o=r.files,t=r.over,a=r.errorMsg,b=(k)=>{let A=r.accept().split(",").map((z)=>z.trim().toLowerCase()).filter(Boolean);if(A.length===0)return!0;let x="."+(k.name.split(".").pop()||"").toLowerCase(),C=k.type.toLowerCase();return A.some((z)=>{if(z.startsWith("."))return x===z;if(z.endsWith("/*"))return C.startsWith(z.slice(0,-1));return C===z})},c=(k)=>{n.dispatchEvent(new CustomEvent("sui-change",{detail:{files:[...k]},bubbles:!0,composed:!0}))},f=(k)=>{a.set("");let A=[];for(let z of k){if(z.size>r.maxSize()){a.set(`"${z.name}" exceeds max size (${Rr(r.maxSize())}).`);continue}if(r.accept()&&!b(z)){a.set(`"${z.name}" is not an accepted file type.`);continue}A.push(z)}let x=o(),C=r.multiple()?[...x,...A]:A.slice(0,1);o.set(C),c(C)},p=(k)=>{let A=o();if(k<0||k>=A.length)return;let x=A.filter((C,z)=>z!==k);o.set(x),a.set(""),c(x)},d=()=>{if(r.disabled())return;r.inputEl()?.click()},l=()=>d(),i=(k)=>{if(r.disabled())return;if(k.key==="Enter"||k.key===" ")k.preventDefault(),d()},u=(k)=>{if(r.disabled())return;k.preventDefault(),t.set(!0)},s=()=>t.set(!1),m=(k)=>{if(r.disabled())return;k.preventDefault(),t.set(!1);let A=k.dataTransfer?.files;if(A&&A.length)f(Array.from(A))},h=(k)=>{let A=k.target;if(A.files&&A.files.length)f(Array.from(A.files));A.value=""},v=r.dropzoneEl(),y=r.inputEl();v?.addEventListener("click",l),v?.addEventListener("keydown",i),v?.addEventListener("dragover",u),v?.addEventListener("dragleave",s),v?.addEventListener("drop",m),y?.addEventListener("change",h);let w=(k)=>{let x=k.target?.closest?.("button.remove");if(!x||!n.shadowRoot?.contains(x))return;let C=Number(x.dataset.idx);if(Number.isInteger(C))p(C)};return n.shadowRoot?.addEventListener("click",w),()=>{v?.removeEventListener("click",l),v?.removeEventListener("keydown",i),v?.removeEventListener("dragover",u),v?.removeEventListener("dragleave",s),v?.removeEventListener("drop",m),y?.removeEventListener("change",h),n.shadowRoot?.removeEventListener("click",w)}}});var mp=j("sui-file-upload",{props:{accept:g.string({default:""}),multiple:g.bool({default:!1}),"max-size":g.number({default:10485760}),disabled:g.bool({default:!1,reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    --tone-border: var(--border);
    --tone-bg: color-mix(in oklch, var(--primary) 8%, transparent);
    --tone-ring: var(--primary);
  }
  :host([disabled]) { opacity: 0.5; pointer-events: none; }

  :host([tone='primary'])     { --tone-ring: var(--primary);     --tone-bg: color-mix(in oklch, var(--primary) 10%, transparent); }
  :host([tone='info'])        { --tone-ring: var(--info);         --tone-bg: color-mix(in oklch, var(--info) 10%, transparent); }
  :host([tone='success'])     { --tone-ring: var(--success);      --tone-bg: color-mix(in oklch, var(--success) 10%, transparent); }
  :host([tone='warning'])     { --tone-ring: var(--warning);      --tone-bg: color-mix(in oklch, var(--warning) 10%, transparent); }
  :host([tone='destructive']) { --tone-ring: var(--destructive);  --tone-bg: color-mix(in oklch, var(--destructive) 10%, transparent); }
  :host([tone='accent'])      { --tone-ring: var(--accent);       --tone-bg: color-mix(in oklch, var(--accent) 10%, transparent); }
  :host([tone='muted'])       { --tone-ring: var(--muted-foreground); --tone-bg: var(--muted); }

  .dropzone {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 24px 16px;
    border: 2px dashed var(--border);
    border-radius: var(--radius-md, 6px);
    background: transparent;
    cursor: pointer;
    transition:
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out);
    color: var(--muted-foreground);
    font-family: var(--font-body);
    font-size: 13px;
    user-select: none;
  }
  .dropzone:focus-visible {
    outline: 2px solid var(--tone-ring);
    outline-offset: 2px;
  }
  .dropzone.over {
    border-style: solid;
    border-color: var(--tone-ring);
    background: var(--tone-bg);
  }

  .icon {
    font-size: 1.5rem;
    line-height: 1;
  }

  input[type='file'] {
    display: none;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 8px;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: var(--radius-sm, 4px);
    background: var(--muted);
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--foreground);
  }

  .file-name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .file-size {
    color: var(--muted-foreground);
    flex-shrink: 0;
  }

  .remove {
    appearance: none;
    border: 0;
    background: transparent;
    cursor: pointer;
    color: var(--muted-foreground);
    padding: 2px;
    line-height: 1;
    border-radius: var(--radius-sm, 4px);
    font-size: 16px;
    font-family: inherit;
  }
  .remove:hover {
    color: var(--foreground);
    background: var(--border);
  }
  .remove:focus-visible {
    outline: 2px solid var(--tone-ring);
    outline-offset: 1px;
  }

  .error {
    font-family: var(--font-body);
    font-size: 12px;
    color: var(--destructive);
    margin-top: 4px;
    padding: 0 2px;
  }

  @media (prefers-reduced-motion: reduce) {
    .dropzone { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.accept=n.accept,t.multiple=n.multiple,t.maxSize=n.maxSize,t.disabled=n.disabled,t.tone=n.tone;let a=null;if(kn&&typeof kn.setup==="function"){if(e.scope=t,a=kn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(y)=>H({anchor:y,test:()=>{if(t.errorMsg())return 0;return-1},branches:[()=>{let w=document.createDocumentFragment(),k=[],A=document.createElement("div");A.setAttribute("class","error"),A.setAttribute("role","alert"),A.setAttribute("part","error");let x=document.createTextNode("");return A.appendChild(x),k.push(F(x,()=>t.errorMsg())),w.appendChild(A),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of k)try{C()}catch(z){}}}}]}),c=(y)=>X({anchor:y,items:()=>t.displayFiles(),key:(w,k)=>{let A=Object.create(t);return A.f=w,A.__i=k,((x)=>w.key)(A)},render:(w,k)=>{let A=Object.create(t);return A.f=w,A.__i=k,A.__v=P(0),((x)=>{let C=document.createDocumentFragment(),z=[],M=document.createElement("div");M.setAttribute("class","file-item"),M.setAttribute("part","file-item");let N=document.createElement("span");N.setAttribute("class","file-name");let S=document.createTextNode("");N.appendChild(S),z.push(F(S,()=>(x.__v(),x.f.name))),M.appendChild(N);let B=document.createElement("span");B.setAttribute("class","file-size");let U=document.createTextNode("");B.appendChild(U),z.push(F(U,()=>(x.__v(),x.f.sizeStr))),M.appendChild(B);let G=document.createElement("button");G.setAttribute("class","remove"),G.setAttribute("part","remove"),G.setAttribute("type","button"),z.push(E(G,"data-idx",()=>(x.__v(),x.f.idx))),z.push(E(G,"aria-label",()=>(x.__v(),"Remove "+x.f.name)));let q=document.createTextNode("×");return G.appendChild(q),M.appendChild(G),C.appendChild(M),{nodes:Array.from(C.childNodes),cleanup:()=>{for(let Q of z)try{Q()}catch(Z){}},update:(Q,Z)=>{x.f=Q,x.__i=Z,x.__v.set((R)=>R+1)}}})(A)}}),f=(y)=>H({anchor:y,test:()=>{if(t.displayFiles().length>0)return 0;return-1},branches:[()=>{let w=document.createDocumentFragment(),k=[],A=document.createElement("div");A.setAttribute("class","file-list"),A.setAttribute("part","file-list"),A.setAttribute("aria-live","polite"),T(t,"fileListEl",A);let x=document.createComment("for");return A.appendChild(x),k.push(c(x)),w.appendChild(A),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of k)try{C()}catch(z){}}}}]}),p=r.shadowRoot??r;L(p);let d=document.createElement("div");d.setAttribute("class","dropzone"),o.push(gt(d,"over",()=>t.over())),d.setAttribute("part","dropzone"),d.setAttribute("role","button"),o.push(E(d,"tabindex",()=>t.disabled()?"-1":"0")),o.push(E(d,"aria-label",()=>t.accept()?"Upload files ("+t.accept()+")":"Upload files")),o.push(E(d,"aria-disabled",()=>t.disabled()?"true":"false")),T(t,"dropzoneEl",d);let l=document.createElement("span");l.setAttribute("class","icon"),l.setAttribute("aria-hidden","true");let i=document.createTextNode("\uD83D\uDCC4");l.appendChild(i),d.appendChild(l);let u=document.createElement("span"),s=document.createTextNode("Drop files here or click to browse");u.appendChild(s),d.appendChild(u),p.appendChild(d);let m=document.createElement("input");m.setAttribute("type","file"),m.setAttribute("part","input"),o.push(E(m,"accept",()=>t.accept())),o.push(E(m,"multiple",()=>t.multiple())),m.setAttribute("aria-hidden","true"),m.setAttribute("tabindex","-1"),T(t,"inputEl",m),p.appendChild(m);let h=document.createComment("if");p.appendChild(h),o.push(b(h));let v=document.createComment("if");if(p.appendChild(v),o.push(f(v)),a!=null){let y=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(y){let w=new Proxy(t,{get(k,A){return k[A]},set(k,A,x){let C=k[A];if(C!=null&&typeof C.set==="function"){if(typeof x==="function")C.set(()=>x);else C.set(x);return!0}return!1}});if(y.mount){let k=y.mount(w,r);if(typeof k==="function")o.push(k)}if(y.unmount)o.push(()=>y.unmount(w))}}return t}});var aa=new Set(["info","success","warning","destructive","accent"]),En=O({scope:()=>{let e=P(!1),n=P(!0),r=Y(null),o=Y(null),t=Y(null),a=Y(null);return{hasSlottedIcon:e,visible:n,iconSlotEl:r,titleSlotEl:o,actionsSlotEl:t,closeBtn:a}},mount(e,n){let r=e,o=null,t=0,a=0,b=!1;if(!n.hasAttribute("tabindex"))n.tabIndex=0;function c(){if(o)clearTimeout(o),o=null}function f(){c();let x=r.duration();if(x>0)t=x,a=Date.now(),o=setTimeout(()=>p("timeout"),x),n.style.setProperty("--toast-duration",`${x}ms`)}function p(x="manual"){if(b)return;b=!0,c(),n.dispatchEvent(new CustomEvent("sui-dismiss",{detail:{reason:x},bubbles:!0,composed:!0})),n.remove()}async function d(x="manual"){if(b)return;if(typeof window<"u"&&window.matchMedia?.("(prefers-reduced-motion: reduce)").matches){p(x);return}c(),n.setAttribute("closing",""),r.visible.set(!1),await new Promise((M)=>setTimeout(M,200)),p(x)}let l=()=>{if(!o)return;let x=Date.now()-a;t=Math.max(0,t-x),c(),n.toggleAttribute("data-paused",!0)},i=()=>{if(b||r.duration()<=0)return;if(t<=0)return;n.toggleAttribute("data-paused",!1),a=Date.now(),o=setTimeout(()=>p("timeout"),t)},u=(x)=>{if(x.key==="Escape"&&r.dismissible())x.stopPropagation(),p("manual")};n.addEventListener("mouseenter",l),n.addEventListener("mouseleave",i),n.addEventListener("focusin",l),n.addEventListener("focusout",i),n.addEventListener("keydown",u),f();let s=r.hasSlottedIcon,m=(x)=>x.assignedNodes({flatten:!0}).some((C)=>{if(C.nodeType===Node.ELEMENT_NODE)return!0;return(C.textContent?.trim().length??0)>0}),h=r.iconSlotEl();if(h){let x=()=>s.set(m(h));h.addEventListener("slotchange",x),s.set(m(h))}let v=r.titleSlotEl(),y=r.actionsSlotEl();if(v){let x=()=>n.toggleAttribute("data-no-title",!m(v));v.addEventListener("slotchange",x),n.toggleAttribute("data-no-title",!m(v))}if(y){let x=()=>n.toggleAttribute("data-no-actions",!m(y));y.addEventListener("slotchange",x),n.toggleAttribute("data-no-actions",!m(y))}let w=D(()=>{let x=r.tone(),C=r.hasSlottedIcon(),z=aa.has(x);n.toggleAttribute("data-no-icon",!z&&!C)}),k=D(()=>{let x=r.closeBtn();if(!x)return;let C=()=>p("manual");return x.addEventListener("click",C),()=>x.removeEventListener("click",C)}),A=n;return A.dismiss=p,A.close=d,()=>{b=!0,c(),n.removeEventListener("mouseenter",l),n.removeEventListener("mouseleave",i),n.removeEventListener("focusin",l),n.removeEventListener("focusout",i),n.removeEventListener("keydown",u),w(),k()}}});var xp=j("sui-toast",{props:{tone:g.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),duration:g.number({default:5000}),dismissible:g.bool({default:!0,reflect:!0}),"show-progress":g.bool({default:!1,reflect:!0}),closing:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    --toast-px: 14px;
    --toast-py: 12px;
    --toast-fs: var(--text-base, 14px);
    --toast-title-fs: var(--text-base, 14px);
    --toast-gap: 10px;
    --toast-min-w: 280px;
    --toast-max-w: 420px;
    --toast-slide-x: 16px;
    --toast-slide-y: 0;
    /* tone channel defaults */
    --tone-bg: transparent;
    --tone-border: var(--border, #e5e5e5);
    pointer-events: auto;
    will-change: transform, opacity;
  }

  :host([size='sm']) {
    --toast-px: 12px;
    --toast-py: 10px;
    --toast-fs: var(--text-sm, 12px);
    --toast-title-fs: var(--text-sm, 12px);
    --toast-gap: 8px;
    --toast-min-w: 240px;
    --toast-max-w: 360px;
  }
  :host([size='lg']) {
    --toast-px: 16px;
    --toast-py: 14px;
    --toast-fs: var(--text-md, 16px);
    --toast-title-fs: var(--text-md, 16px);
    --toast-gap: 12px;
    --toast-min-w: 320px;
    --toast-max-w: 460px;
  }

  /* Tone channel — bg + border per semantic tone */
  :host([tone='info'])        { --tone-bg: var(--tone-info-bg, color-mix(in oklch, var(--info, #3b82f6) 14%, transparent));        --tone-border: var(--tone-info-border, color-mix(in oklch, var(--info, #3b82f6) 30%, transparent)); }
  :host([tone='success'])     { --tone-bg: var(--tone-success-bg, color-mix(in oklch, var(--success, #22c55e) 14%, transparent));   --tone-border: var(--tone-success-border, color-mix(in oklch, var(--success, #22c55e) 30%, transparent)); }
  :host([tone='warning'])     { --tone-bg: var(--tone-warning-bg, color-mix(in oklch, var(--warning, #f59e0b) 14%, transparent));   --tone-border: var(--tone-warning-border, color-mix(in oklch, var(--warning, #f59e0b) 30%, transparent)); }
  :host([tone='destructive']) { --tone-bg: var(--tone-destructive-bg, color-mix(in oklch, var(--destructive, #ef4444) 14%, transparent)); --tone-border: var(--tone-destructive-border, color-mix(in oklch, var(--destructive, #ef4444) 30%, transparent)); }
  :host([tone='accent'])      { --tone-bg: var(--tone-accent-bg, color-mix(in oklch, var(--accent, #a855f7) 14%, transparent));     --tone-border: var(--tone-accent-border, color-mix(in oklch, var(--accent, #a855f7) 30%, transparent)); }

  :host([data-slide-from='right'])  { --toast-slide-x: 16px;  --toast-slide-y: 0; }
  :host([data-slide-from='left'])   { --toast-slide-x: -16px; --toast-slide-y: 0; }
  :host([data-slide-from='top'])    { --toast-slide-x: 0;     --toast-slide-y: -12px; }
  :host([data-slide-from='bottom']) { --toast-slide-x: 0;     --toast-slide-y: 12px; }

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
    border-radius: var(--radius-md, 6px);
    border: 1px solid var(--tone-border);
    background: color-mix(in oklch, var(--tone-bg) 1, var(--card, #fff));
    color: var(--card-foreground, #111);
    box-shadow:
      0 0 0 1px color-mix(in oklch, black 3%, transparent),
      var(--shadow-md, 0 4px 12px rgba(0,0,0,0.1));
    font-family: var(--font-body, system-ui);
    font-size: var(--toast-fs);
    line-height: var(--leading-normal, 1.5);
    position: relative;
    overflow: hidden;
    isolation: isolate;
  }
  /* Enter/exit polish is driven by \`in:slide\` + \`out:fade\` on the
     #if-mounted .toast (see template). */

  :host(:not([tone])) .toast,
  :host([tone='default']) .toast {
    border-color: color-mix(in oklch, var(--border, #e5e5e5) 160%, transparent);
    background: var(--card, #fff);
  }

  :host([tone='info']) .toast,
  :host([tone='success']) .toast,
  :host([tone='warning']) .toast,
  :host([tone='destructive']) .toast,
  :host([tone='accent']) .toast {
    background: color-mix(in oklch, var(--tone-bg) 70%, var(--card, #fff));
    border-color: var(--tone-border);
  }

  .toast::before {
    content: '';
    position: absolute;
    inset-inline-start: 0;
    top: 0; bottom: 0;
    width: 3px;
    background: var(--tone-fg, var(--foreground, #111));
    opacity: 0;
    transition: opacity var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  :host([tone='info']) .toast::before,
  :host([tone='success']) .toast::before,
  :host([tone='warning']) .toast::before,
  :host([tone='destructive']) .toast::before,
  :host([tone='accent']) .toast::before { opacity: 1; }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    flex: 0 0 auto;
    color: var(--tone-fg, var(--muted-foreground, #888));
    margin-top: 1px;
  }
  :host(:not([tone])) .icon,
  :host([tone='default']) .icon { color: var(--muted-foreground, #888); }
  .icon svg { width: 18px; height: 18px; display: block; }
  :host([size='sm']) .icon,
  :host([size='sm']) .icon svg { width: 16px; height: 16px; }
  :host([size='lg']) .icon,
  :host([size='lg']) .icon svg { width: 20px; height: 20px; }

  :host([data-no-icon]) .toast { grid-template-columns: 1fr auto; }
  :host([data-no-icon]) .icon { display: none; }

  .content {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    color: var(--card-foreground, #111);
  }
  .title {
    font-weight: var(--font-weight-semibold, 600);
    font-size: var(--toast-title-fs);
    letter-spacing: -0.005em;
    line-height: var(--leading-tight, 1.25);
    color: var(--foreground, #111);
  }
  .body {
    font-size: var(--toast-fs);
    color: var(--muted-foreground, #888);
    line-height: var(--leading-normal, 1.5);
    word-break: break-word;
  }
  :host([data-no-title]) .body { color: var(--card-foreground, #111); }
  :host([data-no-title]) .title { display: none; }

  .actions {
    display: flex;
    gap: var(--space-2, 8px);
    margin-top: 6px;
    grid-column: 2;
  }
  :host([data-no-actions]) .actions { display: none; margin-top: 0; }

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
    border-radius: var(--radius-sm, 4px);
    color: var(--muted-foreground, #888);
    cursor: pointer;
    padding: 0;
    font: inherit;
    z-index: 1;
    transition:
      background-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .close:hover {
    background: var(--active-bg, color-mix(in oklch, currentColor 8%, transparent));
    color: var(--foreground, #111);
  }
  .close:active { transform: scale(0.94); }
  .close:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 2px var(--card, #fff),
      0 0 0 4px color-mix(in oklch, var(--tone-fg, var(--primary, #3b82f6)) 55%, transparent);
  }
  .close svg { width: 12px; height: 12px; display: block; }

  .progress {
    position: absolute;
    left: 0; right: 0; bottom: 0;
    height: 2px;
    background: var(--tone-fg, var(--muted-foreground, #888));
    transform: scaleX(1);
    transform-origin: left center;
    animation: sui-toast-progress var(--toast-duration, 5000ms) linear forwards;
  }
  :host(:not([tone])) .progress,
  :host([tone='default']) .progress { background: var(--muted-foreground, #888); }
  :host(:hover) .progress,
  :host([data-paused]) .progress { animation-play-state: paused; }
  @keyframes sui-toast-progress {
    from { transform: scaleX(1); }
    to   { transform: scaleX(0); }
  }

  @media (prefers-reduced-motion: reduce) {
    .progress { animation: none; }
  }

  @media (forced-colors: active) {
    .toast { border: 1px solid CanvasText; }
    .close:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.size=n.size,t.duration=n.duration,t.dismissible=n.dismissible,t.showProgress=n.showProgress,t.closing=n.closing;let a=null;if(En&&typeof En.setup==="function"){if(e.scope=t,a=En.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>H({anchor:u,test:()=>{let s=t.tone();if(s==="info")return 0;if(s==="success")return 1;if(s==="warning")return 2;if(s==="destructive")return 3;if(s==="accent")return 4;return-1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("circle");v.setAttribute("cx","12"),v.setAttribute("cy","12"),v.setAttribute("r","9"),h.appendChild(v);let y=document.createElement("path");y.setAttribute("d","M12 8h.01"),h.appendChild(y);let w=document.createElement("path");return w.setAttribute("d","M11 12h1v4h1"),h.appendChild(w),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let k of m)try{k()}catch(A){}}}},()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("circle");v.setAttribute("cx","12"),v.setAttribute("cy","12"),v.setAttribute("r","9"),h.appendChild(v);let y=document.createElement("path");return y.setAttribute("d","m8.5 12.5 2.5 2.5 4.5-5"),h.appendChild(y),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let w of m)try{w()}catch(k){}}}},()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("path");v.setAttribute("d","M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"),h.appendChild(v);let y=document.createElement("line");y.setAttribute("x1","12"),y.setAttribute("x2","12"),y.setAttribute("y1","9"),y.setAttribute("y2","13"),h.appendChild(y);let w=document.createElement("line");return w.setAttribute("x1","12"),w.setAttribute("x2","12.01"),w.setAttribute("y1","17"),w.setAttribute("y2","17"),h.appendChild(w),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let k of m)try{k()}catch(A){}}}},()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("circle");v.setAttribute("cx","12"),v.setAttribute("cy","12"),v.setAttribute("r","9"),h.appendChild(v);let y=document.createElement("line");y.setAttribute("x1","15"),y.setAttribute("x2","9"),y.setAttribute("y1","9"),y.setAttribute("y2","15"),h.appendChild(y);let w=document.createElement("line");return w.setAttribute("x1","9"),w.setAttribute("x2","15"),w.setAttribute("y1","9"),w.setAttribute("y2","15"),h.appendChild(w),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let k of m)try{k()}catch(A){}}}},()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("path");return v.setAttribute("d","M12 2 3 14h7l-1 8 10-12h-7l1-8Z"),h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let y of m)try{y()}catch(w){}}}}]}),c=(u)=>H({anchor:u,test:()=>{if(!t.hasSlottedIcon())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createComment("match");return s.appendChild(h),m.push(b(h)),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let v of m)try{v()}catch(y){}}}}]}),f=(u)=>H({anchor:u,test:()=>{if(t.dismissible())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("button");h.setAttribute("class","close"),h.setAttribute("part","close"),h.setAttribute("type","button"),h.setAttribute("aria-label","Dismiss notification"),T(t,"closeBtn",h);let v=document.createElement("svg");v.setAttribute("viewbox","0 0 24 24"),v.setAttribute("fill","none"),v.setAttribute("stroke","currentColor"),v.setAttribute("stroke-width","2"),v.setAttribute("stroke-linecap","round"),v.setAttribute("stroke-linejoin","round"),v.setAttribute("aria-hidden","true");let y=document.createElement("line");y.setAttribute("x1","18"),y.setAttribute("y1","6"),y.setAttribute("x2","6"),y.setAttribute("y2","18"),v.appendChild(y);let w=document.createElement("line");return w.setAttribute("x1","6"),w.setAttribute("y1","6"),w.setAttribute("x2","18"),w.setAttribute("y2","18"),v.appendChild(w),h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let k of m)try{k()}catch(A){}}}}]}),p=(u)=>H({anchor:u,test:()=>{if(t.showProgress()&&t.duration()>0)return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("span");return h.setAttribute("class","progress"),h.setAttribute("aria-hidden","true"),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let v of m)try{v()}catch(y){}}}}]}),d=(u)=>H({anchor:u,test:()=>{if(t.visible())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("div");h.setAttribute("class","toast"),dt(h,Re,{axis:"y",duration:200},"in"),dt(h,Pt,{duration:180},"out"),m.push(E(h,"role",()=>t.tone()==="destructive"||t.tone()==="warning"?"alert":"status")),m.push(E(h,"aria-live",()=>t.tone()==="destructive"||t.tone()==="warning"?"assertive":"polite")),h.setAttribute("aria-atomic","true"),h.setAttribute("part","toast");let v=document.createElement("span");v.setAttribute("class","icon");let y=document.createElement("slot");y.setAttribute("name","icon"),T(t,"iconSlotEl",y),v.appendChild(y);let w=document.createComment("if");v.appendChild(w),m.push(c(w)),h.appendChild(v);let k=document.createElement("div");k.setAttribute("class","content");let A=document.createElement("div");A.setAttribute("class","title");let x=document.createElement("slot");x.setAttribute("name","title"),T(t,"titleSlotEl",x),A.appendChild(x),k.appendChild(A);let C=document.createElement("div");C.setAttribute("class","body");let z=document.createElement("slot");C.appendChild(z),k.appendChild(C);let M=document.createElement("div");M.setAttribute("class","actions");let N=document.createElement("slot");N.setAttribute("name","actions"),T(t,"actionsSlotEl",N),M.appendChild(N),k.appendChild(M),h.appendChild(k);let S=document.createComment("if");h.appendChild(S),m.push(f(S));let B=document.createComment("if");return h.appendChild(B),m.push(p(B)),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let U of m)try{U()}catch(G){}}}}]}),l=r.shadowRoot??r;L(l);let i=document.createComment("if");if(l.appendChild(i),o.push(d(i)),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(m,h){return m[h]},set(m,h,v){let y=m[h];if(y!=null&&typeof y.set==="function"){if(typeof v==="function")y.set(()=>v);else y.set(v);return!0}return!1}});if(u.mount){let m=u.mount(s,r);if(typeof m==="function")o.push(m)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var ia={"top-left":"top","top-center":"top","top-right":"top","bottom-left":"bottom","bottom-center":"bottom","bottom-right":"bottom"},ot="16px",Ct="position:fixed;display:flex;gap:var(--space-2,8px);pointer-events:none;z-index:var(--z-toast,9999);max-width:min(440px,calc(100vw - 32px));padding:0;margin:0;box-sizing:border-box;";function sa(e){switch(e){case"top-right":return`${Ct}top:${ot};right:${ot};flex-direction:column;align-items:flex-end;`;case"top-left":return`${Ct}top:${ot};left:${ot};flex-direction:column;align-items:flex-start;`;case"top-center":return`${Ct}top:${ot};left:50%;transform:translateX(-50%);flex-direction:column;align-items:center;`;case"bottom-left":return`${Ct}bottom:${ot};left:${ot};flex-direction:column-reverse;align-items:flex-start;`;case"bottom-center":return`${Ct}bottom:${ot};left:50%;transform:translateX(-50%);flex-direction:column-reverse;align-items:center;`;case"bottom-right":default:return`${Ct}bottom:${ot};right:${ot};flex-direction:column-reverse;align-items:flex-end;`}}var Cn=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","region");if(!n.hasAttribute("aria-label"))n.setAttribute("aria-label","Notifications");if(!n.hasAttribute("aria-live"))n.setAttribute("aria-live","polite");let o=P(0),t=r.defaultSlotEl(),a=r.stackEl(),b=(s)=>{if(!a)return;if(s.parentElement!==a)a.appendChild(s)},c=()=>{if(!a)return[];return Array.from(a.children).filter((s)=>s.tagName.toLowerCase()==="sui-toast")},f=D(()=>{if(!a)return;let s=r.position();a.style.cssText=sa(s),a.setAttribute("data-position",s)}),p=D(()=>{o();let s=r.position(),m=r.maxVisible(),h=ia[s]??"bottom",v=c();for(let y of v)y.setAttribute("data-slide-from",h),y.style.pointerEvents="auto";if(m>0&&v.length>m){let y=v.length-m;for(let w=0;w<y;w++){let k=v[w];if(typeof k.close==="function")k.close("programmatic");else k.dismiss?.("programmatic")}}}),d=()=>{if(t){for(let s of t.assignedElements())if(s.tagName.toLowerCase()==="sui-toast")b(s)}o.set(o()+1)};if(t)t.addEventListener("slotchange",d),d();let l=new MutationObserver((s)=>{let m=!1;for(let h of s){for(let v of h.addedNodes)if(v.nodeType===Node.ELEMENT_NODE){let y=v;if(y.tagName.toLowerCase()==="sui-toast")b(y),m=!0}if(h.removedNodes.length)m=!0}if(m)o.set(o()+1)});l.observe(n,{childList:!0});for(let s of Array.from(n.children))if(s.tagName.toLowerCase()==="sui-toast")b(s);o.set(o()+1);let i=a?new MutationObserver(()=>o.set(o()+1)):null;if(a&&i)i.observe(a,{childList:!0});function u(s={}){let m=n.ownerDocument??document,h=m.createElement("sui-toast");if(s.tone)h.setAttribute("tone",s.tone);if(s.size)h.setAttribute("size",s.size);if(typeof s.duration==="number")h.setAttribute("duration",String(s.duration));if(s.dismissible===!1)h.removeAttribute("dismissible");else h.setAttribute("dismissible","");if(s.showProgress)h.setAttribute("show-progress","");if(s.title){let v=m.createElement("span");v.setAttribute("slot","title"),v.textContent=s.title,h.appendChild(v)}if(s.description)h.appendChild(m.createTextNode(s.description));if(s.actionHtml){let v=m.createElement("span");v.setAttribute("slot","actions");let y=s.actionHtml;nt(v,()=>y),h.appendChild(v)}if(a)a.appendChild(h);else n.appendChild(h);return h}return n.show=u,()=>{if(p(),f(),l.disconnect(),i?.disconnect(),t)t.removeEventListener("slotchange",d)}}});var zp=j("sui-toast-container",{props:{position:g.union(["top-left","top-center","top-right","bottom-left","bottom-center","bottom-right"],{default:"bottom-right",reflect:!0}),"max-visible":g.number({default:5,reflect:!0})},shadow:!0,styles:`
  /* Host collapses — the visible stack lives in body via <sui-portal>
     (see template). The host stays minimal so it can sit anywhere in
     the document without taking layout space. */
  :host { display: contents; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.position=n.position,t.maxVisible=n.maxVisible;let a=null;if(Cn&&typeof Cn.setup==="function"){if(e.scope=t,a=Cn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("sui-portal");c.setAttribute("target","body");let f=document.createElement("div");f.setAttribute("class","sui-toast-stack"),f.setAttribute("data-sui-toast-stack",""),T(t,"stackEl",f),c.appendChild(f),b.appendChild(c);let p=document.createElement("slot");if(T(t,"defaultSlotEl",p),b.appendChild(p),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var zn=O({scope:()=>{let e=P([]),n=P("");return{items:e,lastSlot:n}},mount(e,n){let r=e,o=r.items,t=r.lastSlot,a=()=>{let f=Array.from(n.querySelectorAll(":scope > sui-breadcrumb-item")).map((p,d)=>{let l=`i${d}`;return p.setAttribute("slot",l),{slot:l}});t.set(f.length>0?f[f.length-1].slot:""),o.set(f)};a();let b=new MutationObserver(a);return b.observe(n,{childList:!0}),()=>b.disconnect()}});var Mp=j("sui-breadcrumb",{props:{separator:g.string({default:"/",reflect:!0})},shadow:!0,styles:`
  :host { display: flex; align-items: center; }
  nav { display: inline-flex; align-items: center; flex-wrap: wrap; gap: 2px; }
  .sep {
    display: inline-flex; align-items: center; justify-content: center;
    width: 12px; height: 12px;
    color: var(--muted-foreground);
    opacity: 0.7;
  }
  .sep svg { width: 12px; height: 12px; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.separator=n.separator;let a=null;if(zn&&typeof zn.setup==="function"){if(e.scope=t,a=zn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>X({anchor:d,items:()=>t.items(),key:(l,i)=>{let u=Object.create(t);return u.item=l,u.__i=i,((s)=>l.slot)(u)},render:(l,i)=>{let u=Object.create(t);return u.item=l,u.__i=i,u.__v=P(0),((s)=>{let m=(A)=>H({anchor:A,test:()=>{if(s.separator()==="/")return 0;return 1},branches:[()=>{let x=document.createDocumentFragment(),C=[],z=document.createElement("svg");z.setAttribute("viewbox","0 0 24 24"),z.setAttribute("fill","none"),z.setAttribute("stroke","currentColor"),z.setAttribute("stroke-width","2"),z.setAttribute("stroke-linecap","round"),z.setAttribute("stroke-linejoin","round");let M=document.createElement("path");return M.setAttribute("d","M9 6l6 6-6 6"),z.appendChild(M),x.appendChild(z),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let N of C)try{N()}catch(S){}}}},()=>{let x=document.createDocumentFragment(),C=[],z=document.createTextNode("");return x.appendChild(z),C.push(F(z,()=>(s.__v(),s.separator()))),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let M of C)try{M()}catch(N){}}}}]}),h=(A)=>H({anchor:A,test:()=>{if(s.item.slot!==s.lastSlot())return 0;return-1},branches:[()=>{let x=document.createDocumentFragment(),C=[],z=document.createElement("span");z.setAttribute("class","sep"),z.setAttribute("aria-hidden","true");let M=document.createComment("if");return z.appendChild(M),C.push(m(M)),x.appendChild(z),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let N of C)try{N()}catch(S){}}}}]}),v=document.createDocumentFragment(),y=[],w=document.createElement("slot");y.push(E(w,"name",()=>(s.__v(),s.item.slot))),v.appendChild(w);let k=document.createComment("if");return v.appendChild(k),y.push(h(k)),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let A of y)try{A()}catch(x){}},update:(A,x)=>{s.item=A,s.__i=x,s.__v.set((C)=>C+1)}}})(u)}}),c=r.shadowRoot??r;L(c);let f=document.createElement("nav");f.setAttribute("aria-label","Breadcrumb"),f.setAttribute("part","nav");let p=document.createComment("for");if(f.appendChild(p),o.push(b(p)),c.appendChild(f),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var Hp=j("sui-breadcrumb-item",{props:{href:g.string({default:"",reflect:!0}),active:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    align-items: center;
    font-family: var(--font-body);
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    color: var(--muted-foreground);
  }
  :host([active]) {
    color: var(--foreground);
    font-weight: var(--font-weight-medium);
  }
  a, .label {
    display: inline-flex;
    align-items: center;
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.href=n.href,t.active=n.active;let a=(f)=>H({anchor:f,test:()=>{if(t.href()&&!t.active())return 0;return 1},branches:[()=>{let p=document.createDocumentFragment(),d=[],l=document.createElement("a");d.push(E(l,"href",()=>t.href())),l.setAttribute("part","link");let i=document.createElement("slot");return l.appendChild(i),p.appendChild(l),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let u of d)try{u()}catch(s){}}}},()=>{let p=document.createDocumentFragment(),d=[],l=document.createElement("span");l.setAttribute("class","label"),l.setAttribute("part","label"),d.push(E(l,"aria-current",()=>t.active()?"page":"false"));let i=document.createElement("slot");return l.appendChild(i),p.appendChild(l),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let u of d)try{u()}catch(s){}}}}]}),b=r.shadowRoot??r;L(b);let c=document.createComment("if");return b.appendChild(c),o.push(a(c)),t}});var jn=O({scope:()=>({totalPages:P(1),info:P(""),prevBtn:Y(null),nextBtn:Y(null)}),mount(e,n){let r=e,o=D(()=>{let f=r.page(),p=r.total(),d=r.perPage(),l=p>0?Math.ceil(p/d):1;if(r.totalPages.set(l),p>0){let i=(f-1)*d+1,u=Math.min(f*d,p);r.info.set(`Page ${f} of ${l} (${i}–${u} of ${p})`)}else r.info.set(`Page ${f} of ${l}`)}),t=r.prevBtn(),a=r.nextBtn(),b=()=>{let f=r.page();if(f>1){let p=f-1;r.page.set(p),n.dispatchEvent(new CustomEvent("sui-page",{detail:{page:p},bubbles:!0,composed:!0}))}},c=()=>{let f=r.page(),p=r.totalPages();if(f<p){let d=f+1;r.page.set(d),n.dispatchEvent(new CustomEvent("sui-page",{detail:{page:d},bubbles:!0,composed:!0}))}};return t?.addEventListener("click",b),a?.addEventListener("click",c),()=>{o(),t?.removeEventListener("click",b),a?.removeEventListener("click",c)}}});var Xp=j("sui-pagination",{props:{page:g.number({default:1,reflect:!0}),total:g.number({default:0,reflect:!0}),"per-page":g.number({default:25,reflect:!0})},shadow:!0,styles:`
  :host { display: block; }
  nav.bar {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 0.8125rem;
    font-family: var(--font-body);
  }
  .info {
    color: var(--muted-foreground);
    user-select: none;
    white-space: nowrap;
  }
  .spacer { flex: 1; }
  .pg-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 28px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: 0.8125rem;
    cursor: pointer;
    transition: background-color var(--duration-fast);
  }
  .pg-btn:hover:not(:disabled) {
    background: var(--muted);
    border-color: color-mix(in oklch, var(--border) 160%, transparent);
  }
  .pg-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .pg-btn:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: var(--ring-offset);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.page=n.page,t.total=n.total,t.perPage=n.perPage;let a=null;if(jn&&typeof jn.setup==="function"){if(e.scope=t,a=jn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("nav");c.setAttribute("class","bar"),c.setAttribute("aria-label","Pagination");let f=document.createElement("button");T(t,"prevBtn",f),f.setAttribute("class","pg-btn"),f.setAttribute("part","prev"),o.push(E(f,"disabled",()=>t.page()<=1)),f.setAttribute("aria-label","Previous page"),o.push(E(f,"aria-disabled",()=>t.page()<=1?"true":"false"));let p=document.createTextNode("Prev");f.appendChild(p),c.appendChild(f);let d=document.createElement("span");d.setAttribute("class","info"),d.setAttribute("part","info");let l=document.createTextNode("");d.appendChild(l),o.push(F(l,()=>t.info())),c.appendChild(d);let i=document.createElement("span");i.setAttribute("class","spacer"),c.appendChild(i);let u=document.createElement("button");T(t,"nextBtn",u),u.setAttribute("class","pg-btn"),u.setAttribute("part","next"),o.push(E(u,"disabled",()=>t.page()>=t.totalPages())),u.setAttribute("aria-label","Next page"),o.push(E(u,"aria-disabled",()=>t.page()>=t.totalPages()?"true":"false"));let s=document.createTextNode("Next");if(u.appendChild(s),c.appendChild(u),b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let h=new Proxy(t,{get(v,y){return v[y]},set(v,y,w){let k=v[y];if(k!=null&&typeof k.set==="function"){if(typeof w==="function")k.set(()=>w);else k.set(w);return!0}return!1}});if(m.mount){let v=m.mount(h,r);if(typeof v==="function")o.push(v)}if(m.unmount)o.push(()=>m.unmount(h))}}return t}});function qr(e,n=Date.now()){let r=Date.parse(e);if(Number.isNaN(r))return e;let o=n-r,t=Math.abs(o),a=o>=0?"ago":"from now";if(t<60000)return"just now";if(t<3600000)return`${Math.floor(t/60000)}m ${a}`;if(t<86400000)return`${Math.floor(t/3600000)}h ${a}`;let b=Math.floor(t/86400000);if(b<30)return`${b}d ${a}`;if(b<365)return`${Math.floor(b/30)}mo ${a}`;return`${Math.floor(b/365)}y ${a}`}function Jr(e){let n=Date.parse(e);return Number.isNaN(n)?e:new Date(n).toISOString()}var ua=60000,Ln=O({scope:({value:e})=>{let n=P(0),r=J(()=>{let t=e();return t?Jr(t):""}),o=J(()=>{n();let t=e();return t?qr(t):""});return{tick:n,absStr:r,relStr:o}},mount(e,n){let r=e,o=r.tick,t=D(()=>{if(r.value())n.title=r.absStr()}),a=setInterval(()=>o.set(o()+1),ua);return()=>{t(),clearInterval(a)}}});var Vp=j("sui-timestamp",{props:{value:g.string({default:"",reflect:!0}),format:g.union(["relative","absolute","both"],{default:"relative",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline;
    font-family: var(--font-body);
    font-size: inherit;
    color: inherit;
  }
  .abs { color: var(--muted-foreground); margin-left: 4px; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.format=n.format;let a=null;if(Ln&&typeof Ln.setup==="function"){if(e.scope=t,a=Ln.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(l)=>H({anchor:l,test:()=>{if(t.format()==="absolute")return 0;return 1},branches:[()=>{let i=document.createDocumentFragment(),u=[],s=document.createTextNode("");return i.appendChild(s),u.push(F(s,()=>t.absStr())),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let m of u)try{m()}catch(h){}}}},()=>{let i=document.createDocumentFragment(),u=[],s=document.createTextNode("");return i.appendChild(s),u.push(F(s,()=>t.relStr())),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let m of u)try{m()}catch(h){}}}}]}),c=(l)=>H({anchor:l,test:()=>{if(t.format()==="both")return 0;return-1},branches:[()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("span");s.setAttribute("class","abs"),s.setAttribute("part","abs");let m=document.createTextNode("(");s.appendChild(m);let h=document.createTextNode("");s.appendChild(h),u.push(F(h,()=>t.absStr()));let v=document.createTextNode(")");return s.appendChild(v),i.appendChild(s),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let y of u)try{y()}catch(w){}}}}]}),f=(l)=>H({anchor:l,test:()=>{if(t.value())return 0;return-1},branches:[()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("time");u.push(E(s,"datetime",()=>t.value())),s.setAttribute("part","time");let m=document.createComment("if");s.appendChild(m),u.push(b(m)),i.appendChild(s);let h=document.createComment("if");return i.appendChild(h),u.push(c(h)),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let v of u)try{v()}catch(y){}}}}]}),p=r.shadowRoot??r;L(p);let d=document.createComment("if");if(p.appendChild(d),o.push(f(d)),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(u,s){return u[s]},set(u,s,m){let h=u[s];if(h!=null&&typeof h.set==="function"){if(typeof m==="function")h.set(()=>m);else h.set(m);return!0}return!1}});if(l.mount){let u=l.mount(i,r);if(typeof u==="function")o.push(u)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var $p=j("sui-metachip",{props:{label:g.string({default:"",reflect:!0}),value:g.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.label=n.label,t.value=n.value;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");b.setAttribute("class","k"),b.setAttribute("part","label");let c=document.createElement("slot");c.setAttribute("name","label");let f=document.createTextNode("");c.appendChild(f),o.push(F(f,()=>t.label())),b.appendChild(c),a.appendChild(b);let p=document.createElement("span");p.setAttribute("class","v"),p.setAttribute("part","value");let d=document.createElement("slot"),l=document.createTextNode("");return d.appendChild(l),o.push(F(l,()=>t.value())),p.appendChild(d),a.appendChild(p),t}});var la=0,Yr=typeof CSS<"u"&&typeof CSS.supports==="function"&&CSS.supports("anchor-name","--x"),Dn=O({mount(e,n){let r=e,o=++la,t=`--sui-popover-${o}`,a=`sui-popover-panel-${o}`;n.style.setProperty("--sui-popover-anchor",t);let b=r.triggerSlot(),c=r.panelEl();if(!c)return;c.id=a,Ge(r.triggerSlot,(i)=>{for(let u of i)u.setAttribute("popovertarget",a),u.setAttribute("aria-haspopup","dialog"),u.setAttribute("aria-controls",a),u.setAttribute("aria-expanded",r.open()?"true":"false")});function f(i){let s=i.newState==="open",m=n.hasAttribute("open");if(s&&!m)n.setAttribute("open","");else if(!s&&m)n.removeAttribute("open");let v=b?.assignedElements({flatten:!0})??[];for(let y of v)y.setAttribute("aria-expanded",s?"true":"false");if(n.emit("sui-toggle",{open:s}),s&&!Yr)d()}W(c,"toggle",f),n.expose({show:()=>c.showPopover?.(),hide:()=>c.hidePopover?.(),toggle:()=>typeof c.togglePopover==="function"?void c.togglePopover():c.matches?.(":popover-open")?c.hidePopover?.():c.showPopover?.()});let p=D(()=>{let i=r.open(),u=c.matches?.(":popover-open")??!1;if(i&&!u)c.showPopover?.();else if(!i&&u)c.hidePopover?.()});function d(){let s=(b?.assignedElements({flatten:!0})??[])[0];if(!s||!c)return;let m=s.getBoundingClientRect(),h=r.placement();c.style.top="",c.style.left="",c.style.right="",c.style.bottom="";let v=6;switch(h){case"top":c.style.bottom=`${window.innerHeight-m.top+v}px`,c.style.left=`${m.left}px`;break;case"top-end":c.style.bottom=`${window.innerHeight-m.top+v}px`,c.style.right=`${window.innerWidth-m.right}px`;break;case"bottom-end":c.style.top=`${m.bottom+v}px`,c.style.right=`${window.innerWidth-m.right}px`;break;case"left":c.style.right=`${window.innerWidth-m.left+v}px`,c.style.top=`${m.top}px`;break;case"left-end":c.style.right=`${window.innerWidth-m.left+v}px`,c.style.bottom=`${window.innerHeight-m.bottom}px`;break;case"right":c.style.left=`${m.right+v}px`,c.style.top=`${m.top}px`;break;case"right-end":c.style.left=`${m.right+v}px`,c.style.bottom=`${window.innerHeight-m.bottom}px`;break;case"bottom":default:c.style.top=`${m.bottom+v}px`,c.style.left=`${m.left}px`}}function l(){if(!r.open())return;d()}if(!Yr)W(window,"resize",l),W(window,"scroll",l,{capture:!0,passive:!0});return()=>{p()}}});var of=j("sui-popover",{props:{open:g.bool({default:!1,reflect:!0}),placement:g.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom",reflect:!0})},shadow:!0,styles:`
  :host {
    position: relative;
    display: inline-flex;
  }

  /* Native popover defaults are aggressive (position:fixed, inset:0,
     huge default margins). Reset to a layout-neutral container so our
     surface div carries the chrome and anchor positioning drives the
     placement. */
  .panel {
    position: absolute;
    margin: 0;
    border: 0;
    padding: 0;
    background: transparent;
    overflow: visible;
    inset: auto;
  }
  .panel::backdrop { background: transparent; }

  .panel > .surface {
    min-width: 180px;
    padding: 6px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-md, 0 4px 12px rgba(0,0,0,0.1));
    backdrop-filter: var(--card-backdrop-filter, none);
    -webkit-backdrop-filter: var(--card-backdrop-filter, none);
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
  }
  /* Open/close polish is driven by \`transition:scale\` on the
     #if-mounted .surface (see template). No CSS transition here. */

  /* CSS Anchor Positioning. Controller sets a unique anchor-name on the
     SLOTTED trigger element via \`--sui-popover-anchor\`, so each instance
     anchors to its own trigger without any inline-style juggling. */
  @supports (anchor-name: --x) {
    ::slotted([slot='trigger']) {
      anchor-name: var(--sui-popover-anchor, --sui-popover-default);
    }
    .panel {
      position-anchor: var(--sui-popover-anchor, --sui-popover-default);
    }
    :host(:not([placement])) .panel,
    :host([placement='bottom']) .panel {
      top: calc(anchor(bottom) + 6px);
      left: anchor(left);
    }
    :host([placement='bottom-end']) .panel {
      top: calc(anchor(bottom) + 6px);
      right: calc(100vw - anchor(right));
    }
    :host([placement='top']) .panel {
      bottom: calc(100vh - anchor(top) + 6px);
      left: anchor(left);
    }
    :host([placement='top-end']) .panel {
      bottom: calc(100vh - anchor(top) + 6px);
      right: calc(100vw - anchor(right));
    }
    :host([placement='right']) .panel {
      left: calc(anchor(right) + 6px);
      top: anchor(top);
    }
    :host([placement='right-end']) .panel {
      left: calc(anchor(right) + 6px);
      bottom: calc(100vh - anchor(bottom));
    }
    :host([placement='left']) .panel {
      right: calc(100vw - anchor(left) + 6px);
      top: anchor(top);
    }
    :host([placement='left-end']) .panel {
      right: calc(100vw - anchor(left) + 6px);
      bottom: calc(100vh - anchor(bottom));
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .panel > .surface { transform: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.placement=n.placement;let a=null;if(Dn&&typeof Dn.setup==="function"){if(e.scope=t,a=Dn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>H({anchor:u,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("div");h.setAttribute("class","surface"),dt(h,qe,{duration:120},"both");let v=document.createElement("slot");return h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let y of m)try{y()}catch(w){}}}}]}),c=(u)=>H({anchor:u,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("sui-portal");h.setAttribute("target","body");let v=document.createElement("div");return v.setAttribute("class","sui-popover-presence"),v.setAttribute("aria-hidden","true"),v.setAttribute("data-sui-popover-open",""),h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let y of m)try{y()}catch(w){}}}}]}),f=r.shadowRoot??r;L(f);let p=document.createElement("slot");p.setAttribute("name","trigger"),T(t,"triggerSlot",p),f.appendChild(p);let d=document.createElement("div");d.setAttribute("class","panel"),d.setAttribute("part","panel"),d.setAttribute("popover","auto"),T(t,"panelEl",d),d.setAttribute("role","dialog"),d.setAttribute("aria-modal","false");let l=document.createComment("if");d.appendChild(l),o.push(b(l)),f.appendChild(d);let i=document.createComment("if");if(f.appendChild(i),o.push(c(i)),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(m,h){return m[h]},set(m,h,v){let y=m[h];if(y!=null&&typeof y.set==="function"){if(typeof v==="function")y.set(()=>v);else y.set(v);return!0}return!1}});if(u.mount){let m=u.mount(s,r);if(typeof m==="function")o.push(m)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var tr=[{id:"ship-light",label:"SHIP Light",dark:!1,swatches:["oklch(0.99 0.001 106.423)","oklch(1 0 0)","oklch(0.67 0.16 58)","oklch(0.61 0.23 303)"]},{id:"ship-dark",label:"SHIP Dark",dark:!0,swatches:["oklch(0.18 0.01 250)","oklch(0.18 0.01 50)","oklch(0.77 0.16 70)","oklch(0.3 0.05 300)"]},{id:"paper",label:"Paper",dark:!1,swatches:["oklch(0.97 0.014 80)","oklch(0.99 0.010 80)","oklch(0.55 0.14 35)","oklch(0.48 0.09 55)"]},{id:"halo",label:"Halo (Soft)",dark:!1,swatches:["oklch(0.92 0.008 230)","oklch(0.92 0.008 230)","oklch(0.62 0.14 50)","oklch(0.55 0.05 230)"]},{id:"console",label:"Console",dark:!0,swatches:["oklch(0.13 0.008 220)","oklch(0.16 0.010 220)","oklch(0.78 0.20 145)","oklch(0.78 0.16 195)"]},{id:"contrast",label:"High Contrast",dark:!0,swatches:["oklch(0.09 0 0)","oklch(0.15 0 0)","oklch(0.87 0.18 85)","oklch(0.99 0 0)"]},{id:"glass",label:"Glass",dark:!1,swatches:["oklch(0.95 0.04 230)","oklch(0.99 0.005 240)","oklch(0.62 0.16 50)","oklch(0.62 0.14 280)"]}],Wr=Object.fromEntries(tr.map((e)=>[e.id,e]));var On=(e)=>typeof e==="string"&&(e in Wr),da=(e)=>Wr[e]?.dark??!1;function nr(e,n=document.documentElement){let r=On(e)?e:"ship-light",o=da(r);if(n.dataset.theme=r,n.classList.toggle("dark",o),typeof document<"u"&&typeof CustomEvent<"u")document.dispatchEvent(new CustomEvent("ship:theme:change",{detail:{id:r,dark:o}}));return r}function er(){if(typeof localStorage>"u")return"ship-light";try{let e=localStorage.getItem("ship-site-theme");return On(e)?e:"ship-light"}catch{return"ship-light"}}function Zr(e){if(typeof localStorage>"u")return;try{localStorage.setItem("ship-site-theme",e)}catch{}try{fetch("/api/settings",{method:"PUT",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:"shell.theme",value:e})}).catch(()=>{})}catch{}}var ca=[{label:"Default",ids:["ship-light","ship-dark"]},{label:"Texture",ids:["paper","halo"]},{label:"Effects",ids:["glass"]},{label:"Special",ids:["console","contrast"]}],Tn=new Map(tr.map((e)=>[e.id,e]));function pa(e){let n=e.shadowRoot;if(!n)return;if(n.querySelector("link[data-sui-theme-preview-styles]"))return;let r=document.createElement("link");r.rel="stylesheet",r.href="/api/assets/view-tokens.css",r.setAttribute("data-sui-theme-preview-styles",""),n.appendChild(r)}var Mn=O({scope:(e)=>{let n=e,r=P(er()),o=P(!1),t=P(ca.map((i)=>({label:i.label,themes:i.ids.map((u)=>Tn.get(u)).filter((u)=>!!u)}))),a=J(()=>Tn.get(r())?.swatches[2]??"var(--primary, #3b82f6)"),b=J(()=>Tn.get(r())?.label??"Theme"),c={host:null,panel:null};function f(){let i=c.panel;if(!i)return;if(o())i.hidePopover?.();else i.showPopover?.()}function p(i){let u=nr(i);if(n.persist())Zr(u);r.set(u);let s=c.host;if(s)s.dispatchEvent(new CustomEvent("sui-theme-change",{detail:{id:u,dark:Tn.get(u)?.dark??!1},bubbles:!0,composed:!0}));c.panel?.hidePopover?.()}let d=Y(null),l=Y(null);return{activeTheme:r,panelOpen:o,groups:t,dotColor:a,triggerLabel:b,togglePanel:f,selectTheme:p,triggerBtn:d,panelEl:l,_internals:c}},mount(e,n){let r=e;r._internals.host=n,pa(n),nr(er());let o=D(()=>{let p=r.panelEl();if(r._internals.panel=p,!p)return;let d=(l)=>{let u=l.newState==="open";if(r.panelOpen()!==u)r.panelOpen.set(u);if(u)t()};return p.addEventListener("toggle",d),()=>p.removeEventListener("toggle",d)});function t(){let p=r.panelEl(),d=r.triggerBtn();if(!p||!d)return;let l=d.getBoundingClientRect();p.style.top=`${l.bottom+4}px`,p.style.left=`${Math.max(8,l.right-p.offsetWidth)}px`}function a(p){let d=p.detail?.id;if(!On(d))return;if(r.activeTheme()!==d)r.activeTheme.set(d)}document.addEventListener("ship:theme:change",a);function b(p){let d=r.panelEl();if(!d)return;let l=Array.from(d.querySelectorAll(".card"));if(!l.length)return;let i=d.getRootNode(),u=l.findIndex((s)=>s===i.activeElement);switch(p.key){case"ArrowDown":p.preventDefault(),l[(u+1+l.length)%l.length]?.focus();break;case"ArrowUp":p.preventDefault(),l[(u-1+l.length)%l.length]?.focus();break;case"Home":p.preventDefault(),l[0]?.focus();break;case"End":p.preventDefault(),l[l.length-1]?.focus();break;case"Escape":p.preventDefault(),d.hidePopover?.();break}}let c=D(()=>{let p=r.panelEl();if(!p)return;return p.addEventListener("keydown",b),()=>p.removeEventListener("keydown",b)}),f=D(()=>{if(!r.panelOpen())return;requestAnimationFrame(()=>{let p=r.panelEl();if(!p)return;let d=Array.from(p.querySelectorAll(".card"));(d.find((i)=>i.getAttribute("aria-selected")==="true")??d[0])?.focus()})});return()=>{document.removeEventListener("ship:theme:change",a),o(),c(),f()}}});var ff=j("sui-theme-picker",{props:{placement:g.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom-end",reflect:!0}),"label-style":g.union(["text","compact"],{default:"text",reflect:!0}),persist:g.bool({default:!0,reflect:!0})},shadow:!0,styles:`
  :host {
    position: relative;
    display: inline-flex;
    --tp-trigger-gap: 6px;
  }

  /* Trigger button — uses ghost-button visual: transparent until hover.
     Mirrors the look of the shell's <ship-button variant=ghost size=sm>
     because we don't (yet) want sui-theme-picker to depend on sui-button. */
  .trigger {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    padding: 4px 8px;
    font: inherit;
    font-size: var(--text-base, 14px);
    color: var(--foreground, #111);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: var(--tp-trigger-gap);
    transition:
      background-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .trigger:hover {
    background: var(--hover-bg, var(--muted, color-mix(in oklch, currentColor 6%, transparent)));
  }
  .trigger:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }

  .dot {
    width: 10px; height: 10px;
    border-radius: 999px;
    background: var(--primary, #3b82f6);
    box-shadow: 0 0 0 1px color-mix(in oklch, var(--foreground, #111) 15%, transparent);
    flex: 0 0 auto;
  }

  .label {
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 14ch;
  }
  :host([label-style='compact']) .label { display: none; }

  .caret {
    width: 10px; height: 6px;
    flex: 0 0 auto;
    color: var(--muted-foreground, #888);
  }

  /* Native popover panel. Reset the UA's default centering and let
     the controller place us via getBoundingClientRect on the trigger. */
  .panel {
    position: fixed;
    inset: auto;
    margin: 0;
    border: 0;
    padding: 4px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    opacity: 0;
    transform: translateY(-4px) scale(0.97);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  .panel::backdrop { background: transparent; }

  .grid {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 332px;
    max-height: min(80vh, 720px);
    overflow-y: auto;
    scrollbar-width: thin;
  }

  .group { display: flex; flex-direction: column; gap: 2px; }
  .group-h {
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 9px;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--muted-foreground, #888);
    padding: 0 8px 2px;
    opacity: 0.7;
  }
  .group-rows { display: flex; flex-direction: column; gap: 4px; }

  /* Card — full-width entry per theme. Aria-selected drives a tinted
     background + radio fill. */
  .card {
    appearance: none;
    background: transparent;
    border: 0;
    border-radius: 10px;
    padding: 6px;
    cursor: pointer;
    text-align: left;
    font: inherit;
    display: flex;
    flex-direction: column;
    gap: 6px;
    outline: none;
    transition:
      background var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .card:hover {
    background: var(--hover-bg, var(--muted, color-mix(in oklch, currentColor 6%, transparent)));
  }
  .card:focus-visible {
    background: var(--hover-bg, var(--muted, color-mix(in oklch, currentColor 6%, transparent)));
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }
  .card[aria-selected='true'] {
    background: color-mix(in oklch, var(--primary, #3b82f6) 10%, transparent);
  }

  .row {
    display: flex; align-items: center; gap: 8px;
    padding: 0 4px;
  }
  .name {
    flex: 1;
    font-family: var(--font-body, system-ui);
    font-size: 12.5px;
    font-weight: 500;
    color: var(--foreground, #111);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .radio {
    width: 12px; height: 12px;
    border-radius: 999px;
    border: 1.5px solid color-mix(in oklch, var(--foreground, #111) 40%, transparent);
    background: transparent;
    position: relative;
    flex-shrink: 0;
  }
  .card[aria-selected='true'] .radio { border-color: var(--primary, #3b82f6); }
  .card[aria-selected='true'] .radio::after {
    content: '';
    position: absolute;
    inset: 2px;
    border-radius: 999px;
    background: var(--primary, #3b82f6);
  }

  /* Mini card preview tile. Each .tc-bg carries data-theme-preview="<id>",
     which the per-theme \`[data-theme-preview]\` rules in themes.css scope
     for. The controller links view-tokens.css (which bundles themes.css)
     into the shadow root so those rules reach these elements. */
  .tc-bg {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 132px;
    background: var(--background);
    color: var(--foreground);
    border-radius: 8px;
    overflow: hidden;
    isolation: isolate;
    box-shadow: inset 0 0 0 1px color-mix(in oklch, currentColor 14%, transparent);
  }
  .tc-overlay {
    position: absolute; inset: 0;
    pointer-events: none;
    background: var(--surface-overlay, none);
    mix-blend-mode: var(--surface-overlay-blend, normal);
    opacity: var(--surface-overlay-opacity, 1);
    z-index: 1;
  }
  @media (prefers-reduced-motion: reduce) {
    .tc-overlay { display: none; }
    .panel { transition: none; transform: none !important; }
  }
  .tc-card {
    position: relative;
    z-index: 2;
    display: flex; flex-direction: column; gap: 7px;
    width: calc(100% - 24px);
    padding: 10px 12px 11px;
    background: var(--card);
    color: var(--card-foreground);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-card-rest, var(--shadow-sm, 0 1px 2px oklch(0 0 0 / 8%)));
    backdrop-filter: var(--card-backdrop-filter, none);
    -webkit-backdrop-filter: var(--card-backdrop-filter, none);
  }
  .tc-card-head { display: flex; flex-direction: column; gap: 2px; }
  .tc-eyebrow {
    font-family: var(--label-font, var(--font-mono));
    font-weight: var(--label-weight, 600);
    letter-spacing: var(--label-tracking, 0.08em);
    font-size: 8px; line-height: 1;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .tc-heading {
    font-family: var(--heading-font, var(--font-body));
    font-weight: var(--heading-weight, 600);
    font-size: 13px; line-height: 1.05;
    color: var(--card-foreground);
  }
  .tc-typo {
    display: flex; align-items: baseline; gap: 6px;
    font-family: var(--body-font, var(--font-body));
    font-size: 9.5px; line-height: 1;
    color: var(--muted-foreground);
  }
  .tc-body { color: var(--card-foreground); opacity: 0.85; }
  .tc-mono {
    font-family: var(--font-mono);
    font-size: 9px;
    padding: 1px 4px;
    border-radius: 3px;
    background: color-mix(in oklch, var(--foreground) 6%, transparent);
    color: var(--muted-foreground);
  }
  .tc-controls { display: flex; align-items: center; gap: 5px; flex-wrap: wrap; }
  .tc-cta {
    display: inline-flex; align-items: center;
    font-weight: 500; font-size: 9.5px; line-height: 1;
    padding: 4px 8px;
    border-radius: var(--radius-sm, 4px);
    background: var(--primary);
    color: var(--primary-foreground);
  }
  .tc-ghost {
    display: inline-flex; align-items: center;
    font-weight: 500; font-size: 9.5px; line-height: 1;
    padding: 4px 8px;
    border-radius: var(--radius-sm, 4px);
    background: transparent;
    color: var(--muted-foreground);
    border: 1px solid var(--border);
  }
  .tc-chip {
    display: inline-flex; align-items: center;
    font-weight: 600; font-size: 8.5px; line-height: 1;
    padding: 3px 6px;
    border-radius: 999px;
  }
  .tc-chip-accent {
    background: color-mix(in oklch, var(--accent) 18%, transparent);
    color: var(--accent);
    box-shadow: inset 0 0 0 1px color-mix(in oklch, var(--accent) 30%, transparent);
  }
  .tc-chip-muted {
    background: color-mix(in oklch, var(--foreground) 8%, transparent);
    color: var(--muted-foreground);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.placement=n.placement,t.labelStyle=n.labelStyle,t.persist=n.persist;let a=null;if(Mn&&typeof Mn.setup==="function"){if(e.scope=t,a=Mn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(v)=>X({anchor:v,items:()=>t.groups(),key:(y,w)=>{let k=Object.create(t);return k.g=y,k.__i=w,((A)=>y.label)(k)},render:(y,w)=>{let k=Object.create(t);return k.g=y,k.__i=w,k.__v=P(0),((A)=>{let x=(G)=>X({anchor:G,items:()=>A.g.themes,key:(q,Q)=>{let Z=Object.create(A);return Z.t=q,Z.__i=Q,((R)=>q.id)(Z)},render:(q,Q)=>{let Z=Object.create(A);return Z.t=q,Z.__i=Q,Z.__v=P(0),((R)=>{let at=document.createDocumentFragment(),I=[],_=document.createElement("button");_.setAttribute("class","card"),_.setAttribute("role","option"),_.setAttribute("type","button"),I.push(E(_,"data-theme-card",()=>(R.__v(),R.t.id))),I.push(E(_,"aria-label",()=>(R.__v(),R.t.label))),I.push(E(_,"aria-selected",()=>(R.__v(),R.activeTheme()===R.t.id?"true":"false"))),I.push(E(_,"tabindex",()=>(R.__v(),R.activeTheme()===R.t.id?"0":"-1"))),I.push(V(_,"click",function(){let ht=()=>R.selectTheme(R.t.id);return typeof ht==="function"?ht.bind(void 0):()=>{}}(),{modifiers:[]}));let mt=document.createElement("span");mt.setAttribute("class","tc-bg"),I.push(E(mt,"data-theme-preview",()=>(R.__v(),R.t.id))),mt.setAttribute("aria-hidden","true");let sr=document.createElement("span");sr.setAttribute("class","tc-overlay"),mt.appendChild(sr);let Lt=document.createElement("span");Lt.setAttribute("class","tc-card");let Ut=document.createElement("span");Ut.setAttribute("class","tc-card-head");let ke=document.createElement("span");ke.setAttribute("class","tc-eyebrow");let uo=document.createTextNode("THEME");ke.appendChild(uo),Ut.appendChild(ke);let Ee=document.createElement("span");Ee.setAttribute("class","tc-heading");let ur=document.createTextNode("");Ee.appendChild(ur),I.push(F(ur,()=>(R.__v(),R.t.label))),Ut.appendChild(Ee),Lt.appendChild(Ut);let Xt=document.createElement("span");Xt.setAttribute("class","tc-typo");let Ce=document.createElement("span");Ce.setAttribute("class","tc-body");let lo=document.createTextNode("The quick brown fox");Ce.appendChild(lo),Xt.appendChild(Ce);let ze=document.createElement("span");ze.setAttribute("class","tc-mono");let lr=document.createTextNode("");ze.appendChild(lr),I.push(F(lr,()=>(R.__v(),"{ build: ok }"))),Xt.appendChild(ze),Lt.appendChild(Xt);let bt=document.createElement("span");bt.setAttribute("class","tc-controls");let je=document.createElement("span");je.setAttribute("class","tc-cta");let co=document.createTextNode("Action");je.appendChild(co),bt.appendChild(je);let Le=document.createElement("span");Le.setAttribute("class","tc-ghost");let po=document.createTextNode("Cancel");Le.appendChild(po),bt.appendChild(Le);let De=document.createElement("span");De.setAttribute("class","tc-chip tc-chip-accent");let fo=document.createTextNode("New");De.appendChild(fo),bt.appendChild(De);let Oe=document.createElement("span");Oe.setAttribute("class","tc-chip tc-chip-muted");let mo=document.createTextNode("Idle");Oe.appendChild(mo),bt.appendChild(Oe),Lt.appendChild(bt),mt.appendChild(Lt),_.appendChild(mt);let Rt=document.createElement("span");Rt.setAttribute("class","row");let Te=document.createElement("span");Te.setAttribute("class","name");let dr=document.createTextNode("");Te.appendChild(dr),I.push(F(dr,()=>(R.__v(),R.t.label))),Rt.appendChild(Te);let Me=document.createElement("span");return Me.setAttribute("class","radio"),Me.setAttribute("aria-hidden","true"),Rt.appendChild(Me),_.appendChild(Rt),at.appendChild(_),{nodes:Array.from(at.childNodes),cleanup:()=>{for(let ht of I)try{ht()}catch(cr){}},update:(ht,cr)=>{R.t=ht,R.__i=cr,R.__v.set((bo)=>bo+1)}}})(Z)}}),C=document.createDocumentFragment(),z=[],M=document.createElement("section");M.setAttribute("class","group"),z.push(E(M,"aria-label",()=>(A.__v(),A.g.label)));let N=document.createElement("div");N.setAttribute("class","group-h");let S=document.createTextNode("");N.appendChild(S),z.push(F(S,()=>(A.__v(),A.g.label))),M.appendChild(N);let B=document.createElement("div");B.setAttribute("class","group-rows");let U=document.createComment("for");return B.appendChild(U),z.push(x(U)),M.appendChild(B),C.appendChild(M),{nodes:Array.from(C.childNodes),cleanup:()=>{for(let G of z)try{G()}catch(q){}},update:(G,q)=>{A.g=G,A.__i=q,A.__v.set((Q)=>Q+1)}}})(k)}}),c=r.shadowRoot??r;L(c);let f=document.createElement("button");f.setAttribute("class","trigger"),f.setAttribute("part","trigger"),f.setAttribute("type","button"),T(t,"triggerBtn",f),f.setAttribute("aria-label","Choose theme"),f.setAttribute("aria-haspopup","dialog"),o.push(E(f,"aria-expanded",()=>t.panelOpen()?"true":"false")),o.push(V(f,"click",function(){let v=()=>t.togglePanel();return typeof v==="function"?v.bind(void 0):()=>{}}(),{modifiers:[]}));let p=document.createElement("span");p.setAttribute("class","dot"),p.setAttribute("part","dot"),o.push(D(()=>{let v=t.dotColor();if(v==null)p.style.removeProperty("background");else p.style.setProperty("background",String(v))})),p.setAttribute("aria-hidden","true"),f.appendChild(p);let d=document.createElement("span");d.setAttribute("class","label"),d.setAttribute("part","label");let l=document.createTextNode("");d.appendChild(l),o.push(F(l,()=>t.triggerLabel())),f.appendChild(d);let i=document.createElement("svg");i.setAttribute("class","caret"),i.setAttribute("viewbox","0 0 10 6"),i.setAttribute("aria-hidden","true");let u=document.createElement("path");u.setAttribute("d","M1 1l4 4 4-4"),u.setAttribute("fill","none"),u.setAttribute("stroke","currentColor"),u.setAttribute("stroke-width","1.5"),u.setAttribute("stroke-linecap","round"),u.setAttribute("stroke-linejoin","round"),i.appendChild(u),f.appendChild(i),c.appendChild(f);let s=document.createElement("div");s.setAttribute("class","panel"),s.setAttribute("part","panel"),s.setAttribute("popover","auto"),T(t,"panelEl",s);let m=document.createElement("div");m.setAttribute("class","grid"),m.setAttribute("part","grid"),m.setAttribute("role","listbox"),m.setAttribute("aria-label","Theme");let h=document.createComment("for");if(m.appendChild(h),o.push(b(h)),s.appendChild(m),c.appendChild(s),a!=null){let v=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(v){let y=new Proxy(t,{get(w,k){return w[k]},set(w,k,A){let x=w[k];if(x!=null&&typeof x.set==="function"){if(typeof A==="function")x.set(()=>A);else x.set(A);return!0}return!1}});if(v.mount){let w=v.mount(y,r);if(typeof w==="function")o.push(w)}if(v.unmount)o.push(()=>v.unmount(y))}}return t}});var Pn=O({mount(e,n){let r=e,o=D(()=>{let b=r.toggleBtn(),c=r.collapsed();if(b)b.setAttribute("aria-label",c?"Show rail":"Hide rail")}),t=()=>{let b=!r.collapsed();r.collapsed.set(b),n.dispatchEvent(new CustomEvent("sui-rail:toggle",{detail:{collapsed:b},bubbles:!0,composed:!0}))},a=D(()=>{let b=r.toggleBtn();if(!b)return;return b.addEventListener("click",t),()=>b.removeEventListener("click",t)});return()=>{o(),a()}}});var yf=j("sui-rail",{props:{collapsed:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    width: var(--ship-rail-w, 200px);
    background: var(--sidebar, var(--muted));
    border-right: 1px solid var(--border);
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
    --rail-label-display: flex;
    --rail-item-justify: flex-start;
  }
  :host([collapsed]) {
    width: 48px;
    --rail-label-display: none;
    --rail-item-justify: center;
  }
  :host([collapsed]) .rail-footer { padding: 4px; }

  .rail-nav {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    overflow-x: hidden;
    padding: var(--space-2, 8px) 0;
    scrollbar-width: thin;
    scrollbar-color: transparent transparent;
  }
  .rail-nav:hover {
    scrollbar-color: color-mix(in oklch, var(--foreground) 20%, transparent) transparent;
  }

  .rail-footer {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: var(--space-2, 8px);
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }
  :host([collapsed]) .rail-footer {
    gap: 4px;
  }

  .project-badge {
    appearance: none;
    background: var(--primary);
    color: var(--primary-foreground, white);
    border: none;
    border-radius: 6px;
    width: 28px;
    height: 28px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: background-color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .project-badge:hover {
    background: color-mix(in oklch, var(--primary) 80%, black);
  }
  .project-badge:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }

  .footer-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 4px;
    width: 100%;
  }
  :host([collapsed]) .footer-actions {
    flex-direction: column;
    width: auto;
  }

  .footer-btn {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    padding: 4px;
    color: var(--muted-foreground);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    transition:
      background-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .footer-btn:hover {
    background: color-mix(in oklch, var(--foreground) 8%, transparent);
    border-color: var(--border);
    color: var(--foreground);
  }
  .footer-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }

  .toggle-btn {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-sm, 4px);
    padding: 4px;
    color: var(--muted-foreground);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    transition:
      background-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .toggle-btn:hover {
    background: color-mix(in oklch, var(--foreground) 8%, transparent);
    border-color: var(--border);
    color: var(--foreground);
  }
  .toggle-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }
  .toggle-btn::before { content: '◀'; font-size: 10px; line-height: 1; }
  :host([collapsed]) .toggle-btn::before { content: '▶'; }

  .project-popover-container {
    position: relative;
  }

  .project-popover {
    position: absolute;
    bottom: 100%;
    left: 8px;
    margin-bottom: 4px;
    min-width: 200px;
    background: var(--popover, var(--card, white));
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 8px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 100;
    overflow: hidden;
  }
  .project-popover[hidden] {
    display: none;
  }
  .project-popover-title {
    padding: 8px 12px;
    font-size: 11px;
    font-weight: 600;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .project-popover-row {
    padding: 8px 12px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 2px;
    transition: background-color var(--duration-fast, 100ms) var(--ease-out, ease-out);
  }
  .project-popover-row:hover {
    background: color-mix(in oklch, var(--foreground) 6%, transparent);
  }
  .project-popover-row.active {
    background: color-mix(in oklch, var(--primary) 12%, transparent);
    color: var(--primary);
  }
  .project-popover-row-name {
    font-size: 13px;
    font-weight: 500;
  }
  .project-popover-row-path {
    font-size: 11px;
    color: var(--muted-foreground);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .project-popover-sep {
    height: 1px;
    background: var(--border);
    margin: 4px 0;
  }
  .project-popover-row.add {
    color: var(--primary);
    font-weight: 500;
  }
  .project-popover-empty {
    padding: 12px;
    text-align: center;
    font-size: 12px;
    color: var(--muted-foreground);
  }

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.collapsed=n.collapsed;let a=null;if(Pn&&typeof Pn.setup==="function"){if(e.scope=t,a=Pn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("nav");c.setAttribute("class","rail-nav"),c.setAttribute("part","nav");let f=document.createElement("slot");c.appendChild(f),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","rail-footer"),p.setAttribute("part","footer");let d=document.createElement("button");d.setAttribute("type","button"),d.setAttribute("class","project-badge"),d.setAttribute("part","project-badge"),d.setAttribute("data-rail","project-badge"),d.setAttribute("aria-label","Switch project"),p.appendChild(d);let l=document.createElement("div");l.setAttribute("class","project-popover-container");let i=document.createElement("div");i.setAttribute("class","project-popover"),i.setAttribute("part","project-popover"),i.setAttribute("data-rail","project-popover"),i.setAttribute("hidden","");let u=document.createElement("div");u.setAttribute("class","project-popover-title");let s=document.createTextNode("Projects");u.appendChild(s),i.appendChild(u);let m=document.createElement("div");m.setAttribute("data-rail","project-list"),i.appendChild(m);let h=document.createElement("div");h.setAttribute("class","project-popover-sep"),i.appendChild(h);let v=document.createElement("div");v.setAttribute("class","project-popover-row add"),v.setAttribute("data-rail","add-project");let y=document.createElement("span"),w=document.createTextNode("+ Add project");y.appendChild(w),v.appendChild(y),i.appendChild(v),l.appendChild(i),p.appendChild(l);let k=document.createElement("div");k.setAttribute("class","footer-actions");let A=document.createElement("button");A.setAttribute("type","button"),A.setAttribute("class","footer-btn"),A.setAttribute("part","launcher-btn"),A.setAttribute("data-rail","launcher"),A.setAttribute("aria-label","Open launcher");let x=document.createElement("sui-icon");x.setAttribute("name","folder"),x.setAttribute("size","sm"),A.appendChild(x),k.appendChild(A);let C=document.createElement("button");C.setAttribute("type","button"),C.setAttribute("class","footer-btn"),C.setAttribute("part","create-btn"),C.setAttribute("data-rail","create"),C.setAttribute("aria-label","Create new");let z=document.createElement("sui-icon");z.setAttribute("name","plus"),z.setAttribute("size","sm"),C.appendChild(z),k.appendChild(C);let M=document.createElement("button");if(M.setAttribute("type","button"),M.setAttribute("class","toggle-btn"),M.setAttribute("part","toggle"),M.setAttribute("data-rail","toggle"),M.setAttribute("aria-label","Collapse rail"),k.appendChild(M),p.appendChild(k),b.appendChild(p),a!=null){let N=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(N){let S=new Proxy(t,{get(B,U){return B[U]},set(B,U,G){let q=B[U];if(q!=null&&typeof q.set==="function"){if(typeof G==="function")q.set(()=>G);else q.set(G);return!0}return!1}});if(N.mount){let B=N.mount(S,r);if(typeof B==="function")o.push(B)}if(N.unmount)o.push(()=>N.unmount(S))}}return t}});var Af=j("sui-rail-item",{props:{href:g.string({default:""}),label:g.string({default:""}),icon:g.string({default:""}),active:g.bool({default:!1,reflect:!0}),badge:g.string({default:""}),disabled:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    padding: 0 var(--space-2, 8px);
  }

  .item {
    display: flex;
    align-items: center;
    justify-content: var(--rail-item-justify, flex-start);
    gap: var(--space-2, 8px);
    width: 100%;
    padding: 6px var(--space-2, 8px);
    border-radius: var(--radius-sm, 4px);
    border: none;
    background: transparent;
    color: var(--muted-foreground);
    font-family: var(--font-body);
    font-size: var(--text-sm, 13px);
    font-weight: var(--font-weight-medium, 500);
    text-decoration: none;
    cursor: pointer;
    transition:
      background-color var(--duration-fast, 100ms) var(--ease-out, ease-out),
      color var(--duration-fast, 100ms) var(--ease-out, ease-out);
    white-space: nowrap;
    overflow: hidden;
    position: relative;
    box-sizing: border-box;
  }
  .item:hover {
    background: color-mix(in oklch, var(--foreground) 6%, transparent);
    color: var(--foreground);
  }
  :host([active]) .item {
    background: color-mix(in oklch, var(--primary) 12%, transparent);
    color: var(--primary);
  }
  :host([active]) .item:hover {
    background: color-mix(in oklch, var(--primary) 16%, transparent);
  }
  :host([disabled]) .item {
    opacity: 0.45;
    cursor: not-allowed;
    pointer-events: none;
  }
  .item:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }

  .icon-wrap {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 18px;
    height: 18px;
  }

  .label {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: var(--rail-label-display, flex);
    align-items: center;
  }

  .badge {
    display: none;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 999px;
    background: var(--primary);
    color: var(--primary-foreground, white);
    font-size: 10px;
    font-weight: 600;
    line-height: 1;
    flex-shrink: 0;
  }
  .badge:not(:empty) { display: var(--rail-label-display, inline-flex); }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.href=n.href,t.label=n.label,t.icon=n.icon,t.active=n.active,t.badge=n.badge,t.disabled=n.disabled;let a=r.shadowRoot??r;L(a);let b=document.createElement("a");b.setAttribute("class","item"),b.setAttribute("part","item"),T(t,"itemEl",b),o.push(E(b,"href",()=>t.href()||"#")),o.push(E(b,"aria-current",()=>t.active()?"page":null)),o.push(E(b,"aria-disabled",()=>t.disabled()?"true":null)),o.push(E(b,"title",()=>t.label()));let c=document.createElement("span");c.setAttribute("class","icon-wrap"),c.setAttribute("part","icon-wrap");let f=document.createElement("sui-icon");o.push(E(f,"name",()=>t.icon())),f.setAttribute("size","sm"),c.appendChild(f),b.appendChild(c);let p=document.createElement("span");p.setAttribute("class","label"),p.setAttribute("part","label");let d=document.createTextNode("");p.appendChild(d),o.push(F(d,()=>t.label())),b.appendChild(p);let l=document.createElement("span");l.setAttribute("class","badge"),l.setAttribute("part","badge"),T(t,"badgeEl",l);let i=document.createTextNode("");return l.appendChild(i),o.push(F(i,()=>t.badge())),b.appendChild(l),a.appendChild(b),t}});var Cf=j("sui-rail-section",{props:{label:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: block;
    margin-top: var(--space-2, 8px);
  }
  :host(:first-child) { margin-top: 0; }

  .heading {
    display: var(--rail-label-display, flex);
    padding: var(--space-1, 4px) var(--space-4, 16px);
    font-family: var(--font-body);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted-foreground);
    opacity: 0.65;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.label=n.label;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");b.setAttribute("class","heading"),b.setAttribute("part","heading");let c=document.createTextNode("");b.appendChild(c),o.push(F(c,()=>t.label())),a.appendChild(b);let f=document.createElement("slot");return a.appendChild(f),t}});var Lf=j("sui-status-bar",{props:{branch:g.string({default:""}),theme:g.string({default:""}),online:g.bool({default:!1,reflect:!0}),agents:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: flex;
    align-items: center;
    gap: var(--space-3, 12px);
    height: 24px;
    padding: 0 var(--space-3, 12px);
    border-top: 1px solid var(--border);
    background: var(--sidebar, var(--muted));
    font-family: var(--font-body);
    font-size: 11px;
    color: var(--muted-foreground);
    user-select: none;
    -webkit-user-select: none;
    overflow: hidden;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full, 9999px);
    background: var(--destructive);
    flex-shrink: 0;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  :host([online]) .dot {
    background: var(--success);
  }

  .text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 0;
  }
  .text:empty { display: none; }

  .slot-wrap {
    display: inline-flex;
    gap: 8px;
    align-items: center;
  }

  .spacer { flex: 1; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.branch=n.branch,t.theme=n.theme,t.online=n.online,t.agents=n.agents;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");b.setAttribute("class","dot"),b.setAttribute("part","dot"),b.setAttribute("aria-hidden","true"),a.appendChild(b);let c=document.createElement("span");c.setAttribute("class","text"),c.setAttribute("part","branch");let f=document.createTextNode("");c.appendChild(f),o.push(F(f,()=>t.branch())),a.appendChild(c);let p=document.createElement("span");p.setAttribute("class","text"),p.setAttribute("part","agents");let d=document.createTextNode("");p.appendChild(d),o.push(F(d,()=>t.agents())),a.appendChild(p);let l=document.createElement("span");l.setAttribute("class","slot-wrap");let i=document.createElement("slot");i.setAttribute("name","start"),l.appendChild(i),a.appendChild(l);let u=document.createElement("span");u.setAttribute("class","spacer"),a.appendChild(u);let s=document.createElement("span");s.setAttribute("class","slot-wrap");let m=document.createElement("slot");m.setAttribute("name","end"),s.appendChild(m),a.appendChild(s);let h=document.createElement("span");h.setAttribute("class","text"),h.setAttribute("part","theme");let v=document.createTextNode("");return h.appendChild(v),o.push(F(v,()=>t.theme())),a.appendChild(h),t}});var fa=["a[href]","area[href]","button:not([disabled])",'input:not([disabled]):not([type="hidden"])',"select:not([disabled])","textarea:not([disabled])","iframe","object","embed","summary",'[contenteditable]:not([contenteditable="false"])','[tabindex]:not([tabindex^="-"])'].join(",");function zt(e){return Array.from(e.querySelectorAll(fa)).filter((n)=>{if(n.hasAttribute("disabled"))return!1;if(n.getAttribute("aria-hidden")==="true")return!1;return!0})}class Bt{saved=null;save(){this.saved=document.activeElement??null}restore(){let e=this.saved;if(this.saved=null,e&&typeof e.focus==="function")e.focus()}}var Ft=0,rr=null,or=null;function Nn(){if(typeof document>"u")return;if(Ft+=1,Ft>1)return;let e=document.body;rr=e.style.overflow,or=e.style.paddingRight;let n=typeof window<"u"?window.innerWidth-document.documentElement.clientWidth:0;if(n>0){let r=parseFloat(getComputedStyle(e).paddingRight||"0")||0;e.style.paddingRight=`${r+n}px`}e.style.overflow="hidden"}function jt(){if(typeof document>"u")return;if(Ft===0)return;if(Ft-=1,Ft>0)return;let e=document.body;e.style.overflow=rr??"",e.style.paddingRight=or??"",rr=null,or=null}var ma=0,Hn=O({mount(e,n){let r=e,o=++ma,t=`sui-dialog-header-${o}`,a=`sui-dialog-body-${o}`,b=r.dialogEl(),c=r.closeBtn(),f=r.sentinelStart(),p=r.sentinelEnd();if(!b)return;let d=D(()=>{let N=r.headerEl();if(N)N.id=t}),l=D(()=>{let N=r.bodyEl();if(N)N.id=a});b.setAttribute("aria-labelledby",t),b.setAttribute("aria-describedby",a);let i=!1,u=new Bt;function s(){b.setAttribute("role",r.roleOverride()||"dialog"),b.setAttribute("aria-modal","true")}s();function m(){let N=zt(b).filter((B)=>!B.classList.contains("sentinel"));return[...zt(n),...N]}function h(){let N=n.querySelector("[autofocus]")??b.querySelector("[autofocus]");if(N){N.focus();return}let S=m(),B=S.find((U)=>U!==c)??S[0];if(B)B.focus();else b.focus()}function v(){u.save();try{b.showModal()}catch{}if(!i)Nn(),i=!0;queueMicrotask(()=>h()),n.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function y(){if(b.open)b.close()}let w=()=>{if(r.open())r.open.set(!1);if(i)jt(),i=!1;u.restore(),n.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},k=(N)=>{if(!r.dismissible())return;if(N.target===b)r.open.set(!1)},A=()=>{n.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),r.open.set(!1)},x=()=>{let N=m();if(N.length)N[N.length-1].focus()},C=()=>{let N=m();if(N.length)N[0].focus()};b.addEventListener("close",w),b.addEventListener("click",k),c?.addEventListener("click",A),f?.addEventListener("focus",x),p?.addEventListener("focus",C);let z=D(()=>{let N=r.open();if(n.setAttribute("data-state",N?"open":"closed"),N&&!b.open)v();else if(!N&&b.open)y()}),M=D(()=>{s()});return n.show=()=>r.open.set(!0),n.showModal=()=>r.open.set(!0),n.close=()=>r.open.set(!1),()=>{if(i)jt();b.removeEventListener("close",w),b.removeEventListener("click",k),c?.removeEventListener("click",A),f?.removeEventListener("focus",x),p?.removeEventListener("focus",C),z(),M(),d(),l()}}});var Sf=j("sui-dialog",{props:{open:g.bool({default:!1,reflect:!0}),size:g.union(["sm","md","lg","full"],{default:"md",reflect:!0}),backdrop:g.union(["default","blur","solid"],{default:"default",reflect:!0}),dismissible:g.bool({default:!0}),"no-close-button":g.bool({default:!1,reflect:!0}),"no-header":g.bool({default:!1,reflect:!0}),"no-footer":g.bool({default:!1,reflect:!0}),"role-override":g.union(["dialog","alertdialog"],{default:"dialog",reflect:!0})},shadow:!0,styles:`
  :host { display: contents; }

  :host(:not([size])), :host([size='md']) { --dlg-max-w: 500px; --dlg-px: 24px; --dlg-hpy: 18px; --dlg-bpy: 16px; --dlg-fpy: 16px; }
  :host([size='sm']) { --dlg-max-w: 400px; --dlg-px: 20px; --dlg-hpy: 16px; --dlg-bpy: 14px; --dlg-fpy: 14px; }
  :host([size='lg']) { --dlg-max-w: 700px; --dlg-px: 28px; --dlg-hpy: 20px; --dlg-bpy: 20px; --dlg-fpy: 18px; }
  :host([size='full']) { --dlg-max-w: 90vw; --dlg-px: 28px; --dlg-hpy: 20px; --dlg-bpy: 20px; --dlg-fpy: 18px; }

  dialog {
    position: fixed;
    inset: 0;
    margin: auto;
    padding: 0;
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-lg, 8px);
    background: var(--card, #fff);
    color: var(--card-foreground, #111);
    width: calc(100vw - 32px);
    max-width: var(--dlg-max-w, 500px);
    max-height: min(calc(100vh - 32px), 85vh);
    box-shadow: var(--shadow-xl, 0 20px 60px rgba(0,0,0,0.3));
    font-family: var(--font-body, system-ui);
    overflow: visible;
    display: flex;
    flex-direction: column;
  }
  :host([size='full']) dialog { max-height: 90vh; }

  /* Open/close polish is driven by the \`transition:fade\` directive on
     the content wrapper (Web Animations API) and by the native
     ::backdrop fade below — no CSS transition on \`dialog\` itself. */
  .dialog-content {
    display: contents;
  }

  dialog::backdrop {
    background: hsl(0 0% 0% / 0.5);
    opacity: 0;
    transition: opacity var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  dialog[open]::backdrop { opacity: 1; }
  :host([backdrop='blur']) dialog::backdrop { backdrop-filter: blur(6px); -webkit-backdrop-filter: blur(6px); }
  :host([backdrop='solid']) dialog::backdrop { background: var(--background, #fff); }

  .sentinel { position: fixed; width: 1px; height: 1px; pointer-events: none; opacity: 0; }

  .header {
    padding: var(--dlg-hpy, 18px) var(--dlg-px, 24px);
    padding-right: calc(var(--dlg-px, 24px) + 36px);
    flex: 0 0 auto;
    font-size: var(--text-lg, 16px);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--foreground, #111);
  }
  :host([no-header]) .header { display: none; }

  .body {
    padding: var(--dlg-bpy, 16px) var(--dlg-px, 24px);
    flex: 1 1 auto;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    font-size: var(--text-base, 14px);
    line-height: var(--leading-normal, 1.5);
    scrollbar-width: thin;
  }

  .footer {
    padding: var(--dlg-fpy, 16px) var(--dlg-px, 24px);
    border-top: 1px solid var(--border, #e5e5e5);
    display: flex;
    gap: var(--space-2, 8px);
    justify-content: flex-end;
    align-items: center;
    flex: 0 0 auto;
  }
  :host([no-footer]) .footer { display: none; }

  .close {
    position: absolute;
    top: 12px;
    inset-inline-end: 12px;
    width: 28px; height: 28px;
    display: inline-flex; align-items: center; justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md, 6px);
    color: var(--muted-foreground, #888);
    cursor: pointer; padding: 0; font: inherit; z-index: 1;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out),
      color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  .close:hover { background: var(--muted, #f5f5f5); color: var(--foreground, #111); }
  .close svg { width: 14px; height: 14px; display: block; }
  :host([no-close-button]) .close { display: none; }

  @media (prefers-reduced-motion: reduce) {
    dialog::backdrop { transition: none; }
  }
  @media (forced-colors: active) {
    dialog { border: 1px solid CanvasText; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.size=n.size,t.backdrop=n.backdrop,t.dismissible=n.dismissible,t.noCloseButton=n.noCloseButton,t.noHeader=n.noHeader,t.noFooter=n.noFooter,t.roleOverride=n.roleOverride;let a=null;if(Hn&&typeof Hn.setup==="function"){if(e.scope=t,a=Hn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(v)=>H({anchor:v,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let y=document.createDocumentFragment(),w=[],k=document.createElement("div");k.setAttribute("class","dialog-content"),dt(k,Pt,{duration:150},"both");let A=document.createElement("div");A.setAttribute("class","header"),A.setAttribute("part","header"),T(t,"headerEl",A);let x=document.createElement("slot");x.setAttribute("name","header"),A.appendChild(x),k.appendChild(A);let C=document.createElement("div");C.setAttribute("class","body"),C.setAttribute("part","body"),T(t,"bodyEl",C);let z=document.createElement("slot");C.appendChild(z),k.appendChild(C);let M=document.createElement("div");M.setAttribute("class","footer"),M.setAttribute("part","footer");let N=document.createElement("slot");return N.setAttribute("name","footer"),M.appendChild(N),k.appendChild(M),y.appendChild(k),{nodes:Array.from(y.childNodes),cleanup:()=>{for(let S of w)try{S()}catch(B){}}}}]}),c=(v)=>H({anchor:v,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let y=document.createDocumentFragment(),w=[],k=document.createElement("sui-portal");k.setAttribute("target","body");let A=document.createElement("div");return A.setAttribute("class","sui-dialog-presence"),A.setAttribute("aria-hidden","true"),A.setAttribute("data-sui-dialog-open",""),k.appendChild(A),y.appendChild(k),{nodes:Array.from(y.childNodes),cleanup:()=>{for(let x of w)try{x()}catch(C){}}}}]}),f=r.shadowRoot??r;L(f);let p=document.createElement("dialog");p.setAttribute("part","dialog"),T(t,"dialogEl",p);let d=document.createElement("span");d.setAttribute("tabindex","0"),d.setAttribute("class","sentinel"),d.setAttribute("aria-hidden","true"),T(t,"sentinelStart",d),p.appendChild(d);let l=document.createElement("button");l.setAttribute("class","close"),l.setAttribute("part","close"),l.setAttribute("type","button"),l.setAttribute("aria-label","Close dialog"),T(t,"closeBtn",l);let i=document.createElement("svg");i.setAttribute("viewbox","0 0 16 16"),i.setAttribute("fill","none"),i.setAttribute("stroke","currentColor"),i.setAttribute("stroke-width","1.75"),i.setAttribute("stroke-linecap","round"),i.setAttribute("stroke-linejoin","round"),i.setAttribute("aria-hidden","true"),i.setAttribute("focusable","false");let u=document.createElement("path");u.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),i.appendChild(u),l.appendChild(i),p.appendChild(l);let s=document.createComment("if");p.appendChild(s),o.push(b(s));let m=document.createElement("span");m.setAttribute("tabindex","0"),m.setAttribute("class","sentinel"),m.setAttribute("aria-hidden","true"),T(t,"sentinelEnd",m),p.appendChild(m),f.appendChild(p);let h=document.createComment("if");if(f.appendChild(h),o.push(c(h)),a!=null){let v=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(v){let y=new Proxy(t,{get(w,k){return w[k]},set(w,k,A){let x=w[k];if(x!=null&&typeof x.set==="function"){if(typeof A==="function")x.set(()=>A);else x.set(A);return!0}return!1}});if(v.mount){let w=v.mount(y,r);if(typeof w==="function")o.push(w)}if(v.unmount)o.push(()=>v.unmount(y))}}return t}});var ba=0,Sn=O({mount(e,n){let r=e,o=++ba,t=`sui-drawer-header-${o}`,a=`sui-drawer-body-${o}`,b=r.dialogEl(),c=r.closeBtn(),f=r.sentinelStart(),p=r.sentinelEnd(),d=r.headerEl(),l=r.bodyEl();if(!b)return;if(d)d.id=t;if(l)l.id=a;b.setAttribute("aria-labelledby",t),b.setAttribute("aria-describedby",a);let i=!1,u=new Bt;function s(){let z=zt(b).filter((N)=>!N.classList.contains("sentinel"));return[...zt(n),...z]}function m(){let z=n.querySelector("[autofocus]")??b.querySelector("[autofocus]");if(z){z.focus();return}let M=s(),N=M.find((S)=>S!==c)??M[0];if(N)N.focus();else b.focus()}function h(){u.save();try{b.showModal()}catch{}if(!i)Nn(),i=!0;queueMicrotask(()=>m()),n.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function v(){if(b.open)b.close()}let y=()=>{if(r.open())r.open.set(!1);if(i)jt(),i=!1;u.restore(),n.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},w=(z)=>{if(!r.dismissible())return;if(z.target===b)r.open.set(!1)},k=()=>{n.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),r.open.set(!1)},A=()=>{let z=s();if(z.length)z[z.length-1].focus()},x=()=>{let z=s();if(z.length)z[0].focus()};b.addEventListener("close",y),b.addEventListener("click",w),c?.addEventListener("click",k),f?.addEventListener("focus",A),p?.addEventListener("focus",x);let C=D(()=>{let z=r.open();if(n.setAttribute("data-state",z?"open":"closed"),z&&!b.open)h();else if(!z&&b.open)v()});return n.show=()=>r.open.set(!0),n.close=()=>r.open.set(!1),()=>{if(i)jt();b.removeEventListener("close",y),b.removeEventListener("click",w),c?.removeEventListener("click",k),f?.removeEventListener("focus",A),p?.removeEventListener("focus",x),C()}}});var qf=j("sui-drawer",{props:{open:g.bool({default:!1,reflect:!0}),side:g.union(["right","left"],{default:"right",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),dismissible:g.bool({default:!0})},shadow:!0,styles:`
  :host { display: contents; }

  :host(:not([size])), :host([size='md']) { --dr-w: 420px; --dr-px: 24px; --dr-hpy: 16px; --dr-bpy: 16px; --dr-fpy: 14px; }
  :host([size='sm']) { --dr-w: 320px; --dr-px: 20px; --dr-hpy: 14px; --dr-bpy: 14px; --dr-fpy: 12px; }
  :host([size='lg']) { --dr-w: 560px; --dr-px: 28px; --dr-hpy: 20px; --dr-bpy: 20px; --dr-fpy: 16px; }

  dialog {
    position: fixed;
    top: 0; bottom: 0;
    right: 0; left: auto;
    margin: 0; padding: 0;
    border: none;
    border-left: 1px solid var(--border, #e5e5e5);
    background: var(--card, #fff);
    color: var(--card-foreground, #111);
    width: min(var(--dr-w, 420px), calc(100vw - 32px));
    max-width: 100vw;
    height: 100%;
    max-height: 100%;
    box-shadow: var(--shadow-xl, -4px 0 30px rgba(0,0,0,0.15));
    font-family: var(--font-body, system-ui);
    display: flex;
    flex-direction: column;
    opacity: 0;
    transform: translateX(24px);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  :host([side='left']) dialog {
    right: auto; left: 0;
    border-left: none;
    border-right: 1px solid var(--border, #e5e5e5);
    box-shadow: var(--shadow-xl, 4px 0 30px rgba(0,0,0,0.15));
    transform: translateX(-24px);
  }
  dialog[open] { opacity: 1; transform: translateX(0); }

  dialog::backdrop {
    background: hsl(0 0% 0% / 0.5);
    opacity: 0;
    transition: opacity var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  dialog[open]::backdrop { opacity: 1; }

  .sentinel { position: fixed; width: 1px; height: 1px; pointer-events: none; opacity: 0; }

  .header {
    display: flex; align-items: flex-start; gap: var(--space-2, 8px);
    padding: var(--dr-hpy, 16px) var(--dr-px, 24px);
    padding-right: calc(var(--dr-px, 24px) + 36px);
    border-bottom: 1px solid var(--border, #e5e5e5);
    flex: 0 0 auto; position: relative;
  }
  .header-content { flex: 1 1 0; min-width: 0; }
  .title {
    font-size: var(--text-base, 14px);
    font-weight: var(--font-weight-semibold, 600);
    line-height: var(--leading-tight, 1.25);
    color: var(--foreground, #111);
  }
  .description {
    font-size: var(--text-sm, 12px);
    color: var(--muted-foreground, #888);
    margin-top: 2px;
    line-height: var(--leading-normal, 1.5);
  }

  .body {
    flex: 1 1 auto; min-height: 0;
    overflow-y: auto; overscroll-behavior: contain;
    padding: var(--dr-bpy, 16px) var(--dr-px, 24px);
    font-size: var(--text-base, 14px);
    line-height: var(--leading-normal, 1.5);
    scrollbar-width: thin;
  }

  .footer {
    flex: 0 0 auto;
    padding: var(--dr-fpy, 14px) var(--dr-px, 24px);
    border-top: 1px solid var(--border, #e5e5e5);
    display: flex; gap: var(--space-2, 8px);
    justify-content: flex-end; align-items: center;
  }

  .close {
    position: absolute;
    top: 12px; inset-inline-end: 12px;
    width: 28px; height: 28px;
    display: inline-flex; align-items: center; justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md, 6px);
    color: var(--muted-foreground, #888);
    cursor: pointer; padding: 0; font: inherit; z-index: 1;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out),
      color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  .close:hover { background: var(--muted, #f5f5f5); color: var(--foreground, #111); }
  .close svg { width: 14px; height: 14px; display: block; }

  @media (prefers-reduced-motion: reduce) {
    dialog { transition: none; transform: none !important; opacity: 1; }
    dialog::backdrop { transition: none; }
  }
  @media (forced-colors: active) {
    dialog { border: 1px solid CanvasText; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.side=n.side,t.size=n.size,t.dismissible=n.dismissible;let a=null;if(Sn&&typeof Sn.setup==="function"){if(e.scope=t,a=Sn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("dialog");c.setAttribute("part","dialog"),c.setAttribute("role","dialog"),c.setAttribute("aria-modal","true"),T(t,"dialogEl",c);let f=document.createElement("span");f.setAttribute("tabindex","0"),f.setAttribute("class","sentinel"),f.setAttribute("aria-hidden","true"),T(t,"sentinelStart",f),c.appendChild(f);let p=document.createElement("div");p.setAttribute("class","header"),p.setAttribute("part","header"),T(t,"headerEl",p);let d=document.createElement("div");d.setAttribute("class","header-content");let l=document.createElement("div");l.setAttribute("class","title");let i=document.createElement("slot");i.setAttribute("name","title"),l.appendChild(i),d.appendChild(l);let u=document.createElement("div");u.setAttribute("class","description");let s=document.createElement("slot");s.setAttribute("name","description"),u.appendChild(s),d.appendChild(u),p.appendChild(d);let m=document.createElement("button");m.setAttribute("class","close"),m.setAttribute("part","close"),m.setAttribute("type","button"),m.setAttribute("aria-label","Close drawer"),T(t,"closeBtn",m);let h=document.createElement("svg");h.setAttribute("viewbox","0 0 16 16"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","1.75"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true"),h.setAttribute("focusable","false");let v=document.createElement("path");v.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),h.appendChild(v),m.appendChild(h),p.appendChild(m),c.appendChild(p);let y=document.createElement("div");y.setAttribute("class","body"),y.setAttribute("part","body"),T(t,"bodyEl",y);let w=document.createElement("slot");y.appendChild(w),c.appendChild(y);let k=document.createElement("div");k.setAttribute("class","footer"),k.setAttribute("part","footer");let A=document.createElement("slot");A.setAttribute("name","footer"),k.appendChild(A),c.appendChild(k);let x=document.createElement("span");if(x.setAttribute("tabindex","0"),x.setAttribute("class","sentinel"),x.setAttribute("aria-hidden","true"),T(t,"sentinelEnd",x),c.appendChild(x),b.appendChild(c),a!=null){let C=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(t,{get(M,N){return M[N]},set(M,N,S){let B=M[N];if(B!=null&&typeof B.set==="function"){if(typeof S==="function")B.set(()=>S);else B.set(S);return!0}return!1}});if(C.mount){let M=C.mount(z,r);if(typeof M==="function")o.push(M)}if(C.unmount)o.push(()=>C.unmount(z))}}return t}});var ha=0,Fn=O({mount(e,n){let r=e,o=`sui-dropdown-menu-panel-${++ha}`,t=r.triggerWrap(),a=r.triggerSlot(),b=r.panelEl();if(!b)return;b.id=o;let c="",f=null;function p(x){if(f)clearTimeout(f);c+=x.toLowerCase(),f=setTimeout(()=>{c=""},350);let C=d(),z=C.indexOf(document.activeElement);for(let M=0;M<C.length;M++){let N=(z+1+M)%C.length;if((C[N].textContent??"").trim().toLowerCase().startsWith(c))return N}return-1}function d(){return Array.from(n.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((x)=>!x.hasAttribute("disabled"))}function l(x){let C=d();if(!C.length)return;let z=(x%C.length+C.length)%C.length;for(let M of C)M.setAttribute("tabindex","-1");C[z].setAttribute("tabindex","0"),C[z].focus()}function i(){return d().indexOf(document.activeElement)}function u(){d().forEach((x,C)=>{if(!x.hasAttribute("role"))x.setAttribute("role","menuitem");if(!x.hasAttribute("tabindex"))x.setAttribute("tabindex",C===0?"0":"-1")})}function s(){let x=a?.assignedElements({flatten:!0})??[];for(let C of x)C.setAttribute("aria-haspopup","menu"),C.setAttribute("aria-controls",o),C.setAttribute("aria-expanded",r.open()?"true":"false")}function m(){let x=a?.assignedElements({flatten:!0})?.[0];if(x&&b){let C=x.getBoundingClientRect();b.style.top=`${C.bottom+4}px`,b.style.left=`${C.left}px`}b?.showPopover?.()}function h(){b?.hidePopover?.()}let v=()=>{if(r.open())h();else m()},y=(x)=>{let z=x.newState==="open";if(z!==r.open())r.open.set(z);if(n.setAttribute("data-state",z?"open":"closed"),s(),z)queueMicrotask(()=>{u(),l(0)})},w=(x)=>{let C=x.target.closest("[data-value]");if(!C)return;n.emit("sui-select",{value:C.dataset.value??""}),h()},k=(x)=>{if(!r.open())return;let C=d();if(!C.length)return;let z=i();if(x.key==="ArrowDown")x.preventDefault(),l(z+1);else if(x.key==="ArrowUp")x.preventDefault(),l(z-1);else if(x.key==="Home")x.preventDefault(),l(0);else if(x.key==="End")x.preventDefault(),l(C.length-1);else if(x.key==="Escape")x.preventDefault(),h();else if(x.key==="Tab")h();else if((x.key==="Enter"||x.key===" ")&&z>=0)x.preventDefault(),C[z].click();else if(x.key.length===1&&!x.ctrlKey&&!x.metaKey){let M=p(x.key);if(M>=0)x.preventDefault(),l(M)}};if(t)W(t,"click",v);if(W(b,"toggle",y),W(b,"click",w),W(n,"keydown",k),a)W(a,"slotchange",s);W(b,"slotchange",u);let A=D(()=>{let x=r.open();n.setAttribute("data-state",x?"open":"closed"),s()});return()=>{if(f)clearTimeout(f);A()}}});var Vf=j("sui-dropdown-menu",{props:{open:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { position: relative; display: inline-flex; }

  .panel {
    inset: auto;
    margin: 0;
    min-width: 160px;
    padding: 4px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius, var(--radius-md, 6px));
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    display: flex; flex-direction: column;
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
    opacity: 0;
    transform: translateY(-4px) scale(0.97);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  .panel::backdrop { background: transparent; }

  ::slotted(button), ::slotted([role='menuitem']) {
    text-align: left;
    background: transparent; border: 0;
    padding: 6px 10px;
    border-radius: var(--radius-sm, 4px);
    color: inherit; font: inherit;
    cursor: pointer; width: 100%;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  ::slotted(button:hover), ::slotted([role='menuitem']:hover),
  ::slotted(button:focus-visible), ::slotted([role='menuitem']:focus-visible) {
    background: var(--accent, var(--muted, #f5f5f5));
    outline: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .panel { transition: none; transform: none !important; }
  }
  @media (forced-colors: active) {
    .panel { border: 1px solid CanvasText; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open;let a=null;if(Fn&&typeof Fn.setup==="function"){if(e.scope=t,a=Fn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("span");c.setAttribute("class","trigger-wrap"),T(t,"triggerWrap",c);let f=document.createElement("slot");f.setAttribute("name","trigger"),T(t,"triggerSlot",f),c.appendChild(f),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","panel"),p.setAttribute("part","panel"),p.setAttribute("popover","auto"),p.setAttribute("role","menu"),T(t,"panelEl",p);let d=document.createElement("slot");if(T(t,"itemSlot",d),p.appendChild(d),b.appendChild(p),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(u,s){return u[s]},set(u,s,m){let h=u[s];if(h!=null&&typeof h.set==="function"){if(typeof m==="function")h.set(()=>m);else h.set(m);return!0}return!1}});if(l.mount){let u=l.mount(i,r);if(typeof u==="function")o.push(u)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var va=0,Bn=O({mount(e,n){let r=e,o=`sui-context-menu-panel-${++va}`,t=r.triggerSlot(),a=r.panelEl();if(!a)return;a.id=o;let b="",c=null;function f(A){if(c)clearTimeout(c);b+=A.toLowerCase(),c=setTimeout(()=>{b=""},350);let x=p(),C=x.indexOf(document.activeElement);for(let z=0;z<x.length;z++){let M=(C+1+z)%x.length;if((x[M].textContent??"").trim().toLowerCase().startsWith(b))return M}return-1}function p(){return Array.from(n.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((A)=>!A.hasAttribute("disabled"))}function d(A){let x=p();if(!x.length)return;let C=(A%x.length+x.length)%x.length;for(let z of x)z.setAttribute("tabindex","-1");x[C].setAttribute("tabindex","0"),x[C].focus()}function l(){return p().indexOf(document.activeElement)}function i(){p().forEach((A,x)=>{if(!A.hasAttribute("role"))A.setAttribute("role","menuitem");if(!A.hasAttribute("tabindex"))A.setAttribute("tabindex",x===0?"0":"-1")})}function u(A,x){if(a)a.style.top=`${x}px`,a.style.left=`${A}px`;a?.showPopover?.()}function s(){a?.hidePopover?.()}n.expose({show:u,close:s});let m=(A)=>{A.preventDefault(),u(A.clientX,A.clientY)},h=(A)=>{let C=A.newState==="open";if(C!==r.open())r.open.set(C);if(n.setAttribute("data-state",C?"open":"closed"),C)n.emit("sui-open"),queueMicrotask(()=>{i(),d(0)});else n.emit("sui-close")},v=(A)=>{let x=A.target.closest("[data-value]");if(!x)return;n.emit("sui-select",{value:x.dataset.value??""}),s()},y=(A)=>{if(!r.open())return;let x=p();if(!x.length)return;let C=l();if(A.key==="ArrowDown")A.preventDefault(),d(C+1);else if(A.key==="ArrowUp")A.preventDefault(),d(C-1);else if(A.key==="Home")A.preventDefault(),d(0);else if(A.key==="End")A.preventDefault(),d(x.length-1);else if(A.key==="Escape")A.preventDefault(),s();else if(A.key==="Tab")s();else if((A.key==="Enter"||A.key===" ")&&C>=0)A.preventDefault(),x[C].click();else if(A.key.length===1&&!A.ctrlKey&&!A.metaKey){let z=f(A.key);if(z>=0)A.preventDefault(),d(z)}},w=()=>{let A=t?.assignedElements({flatten:!0})??[];for(let x of A)x.addEventListener("contextmenu",m)};if(t)W(t,"slotchange",w);w(),W(a,"toggle",h),W(a,"click",v),W(n,"keydown",y);let k=D(()=>{n.setAttribute("data-state",r.open()?"open":"closed")});return()=>{let A=t?.assignedElements({flatten:!0})??[];for(let x of A)x.removeEventListener("contextmenu",m);if(c)clearTimeout(c);k()}}});var nm=j("sui-context-menu",{props:{open:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: contents; }

  .panel {
    inset: auto;
    margin: 0;
    min-width: 160px;
    padding: 4px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius, var(--radius-md, 6px));
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    display: flex; flex-direction: column;
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
    opacity: 0;
    transform: scale(0.96);
    transition:
      opacity var(--duration-fast, 120ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: scale(1);
  }
  .panel::backdrop { background: transparent; }

  ::slotted(button), ::slotted([role='menuitem']) {
    text-align: left;
    background: transparent; border: 0;
    padding: 6px 10px;
    border-radius: var(--radius-sm, 4px);
    color: inherit; font: inherit;
    cursor: pointer; width: 100%;
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, ease-out);
  }
  ::slotted(button:hover), ::slotted([role='menuitem']:hover),
  ::slotted(button:focus-visible), ::slotted([role='menuitem']:focus-visible) {
    background: var(--accent, var(--muted, #f5f5f5));
    outline: none;
  }

  @media (prefers-reduced-motion: reduce) {
    .panel { transition: none; transform: none !important; }
  }
  @media (forced-colors: active) {
    .panel { border: 1px solid CanvasText; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open;let a=null;if(Bn&&typeof Bn.setup==="function"){if(e.scope=t,a=Bn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");c.setAttribute("name","trigger"),T(t,"triggerSlot",c),b.appendChild(c);let f=document.createElement("div");f.setAttribute("class","panel"),f.setAttribute("part","panel"),f.setAttribute("popover","auto"),f.setAttribute("role","menu"),f.setAttribute("aria-label","Context menu"),T(t,"panelEl",f);let p=document.createElement("slot");if(T(t,"itemSlot",p),f.appendChild(p),b.appendChild(f),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var ga=0,Gn=8;function ya(e,n,r){let o=e.getBoundingClientRect();switch(n.style.top="",n.style.left="",n.style.right="",n.style.bottom="",r){case"top":n.style.bottom=`${window.innerHeight-o.top+Gn}px`,n.style.left=`${o.left+o.width/2}px`,n.style.transform="translateX(-50%)";break;case"left":n.style.right=`${window.innerWidth-o.left+Gn}px`,n.style.top=`${o.top}px`;break;case"right":n.style.left=`${o.right+Gn}px`,n.style.top=`${o.top}px`;break;case"bottom":default:n.style.top=`${o.bottom+Gn}px`,n.style.left=`${o.left+o.width/2}px`,n.style.transform="translateX(-50%)"}}var Un=O({mount(e,n){let r=e,o=`sui-hover-card-${++ga}`,t=r.triggerSlot(),a=r.panelEl();if(!a)return;a.id=o;let b=null,c=null;function f(){if(b)clearTimeout(b),b=null;if(c)clearTimeout(c),c=null}function p(){let y=t?.assignedElements({flatten:!0})??[];for(let w of y)w.setAttribute("aria-describedby",o)}function d(){f();let y=t?.assignedElements({flatten:!0})?.[0];if(y&&a)ya(y,a,r.placement()||"bottom");a?.showPopover?.()}function l(){f(),a?.hidePopover?.()}n.show=d,n.hide=l;let i=(y)=>{let k=y.newState==="open";if(k!==r.open())r.open.set(k);n.setAttribute("data-state",k?"open":"closed"),n.dispatchEvent(new CustomEvent(k?"sui-open":"sui-close",{bubbles:!0,composed:!0}))},u=()=>{f(),b=setTimeout(()=>d(),r.showDelay()??300)},s=()=>{f(),c=setTimeout(()=>l(),r.hideDelay()??100)},m=()=>{f(),d()},h=()=>{f(),c=setTimeout(()=>l(),r.hideDelay()??100)};a.addEventListener("toggle",i),t?.addEventListener("slotchange",p),p(),n.addEventListener("pointerenter",u),n.addEventListener("pointerleave",s),n.addEventListener("focusin",m),n.addEventListener("focusout",h);let v=D(()=>{n.setAttribute("data-state",r.open()?"open":"closed")});return()=>{f(),a.removeEventListener("toggle",i),t?.removeEventListener("slotchange",p),n.removeEventListener("pointerenter",u),n.removeEventListener("pointerleave",s),n.removeEventListener("focusin",m),n.removeEventListener("focusout",h),v()}}});var sm=j("sui-hover-card",{props:{open:g.bool({default:!1,reflect:!0}),placement:g.union(["top","bottom","left","right"],{default:"bottom",reflect:!0}),"show-delay":g.number({default:300}),"hide-delay":g.number({default:100})},shadow:!0,styles:`
  :host { display: inline-flex; position: relative; }

  .panel {
    inset: auto;
    margin: 0;
    min-width: 240px;
    max-width: 340px;
    padding: 16px;
    background: var(--popover, var(--card, #fff));
    color: var(--popover-foreground, var(--card-foreground, #111));
    border: 1px solid var(--border, #e5e5e5);
    border-radius: var(--radius-lg, 8px);
    box-shadow: var(--shadow-lg, 0 8px 24px rgba(0,0,0,0.12));
    font-family: var(--font-body, system-ui);
    font-size: var(--text-base, 14px);
    line-height: var(--leading-normal, 1.5);
    opacity: 0;
    transform: translateY(-4px);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  .panel:popover-open {
    opacity: 1;
    transform: translateY(0);
  }
  .panel::backdrop { background: transparent; }

  @media (prefers-reduced-motion: reduce) {
    .panel { transition: none; transform: none !important; }
  }
  @media (forced-colors: active) {
    .panel { border: 1px solid CanvasText; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.placement=n.placement,t.showDelay=n.showDelay,t.hideDelay=n.hideDelay;let a=null;if(Un&&typeof Un.setup==="function"){if(e.scope=t,a=Un.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");c.setAttribute("name","trigger"),T(t,"triggerSlot",c),b.appendChild(c);let f=document.createElement("div");f.setAttribute("class","panel"),f.setAttribute("part","panel"),f.setAttribute("popover","auto"),T(t,"panelEl",f);let p=document.createElement("slot");if(f.appendChild(p),b.appendChild(f),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var Xn=O({mount(e,n){let r=e,o=()=>Array.from(n.querySelectorAll(":scope > sui-accordion-item")),t=()=>o().filter((d)=>!d.hasAttribute("disabled")),a=()=>{let d=r.value();return new Set(d?d.split(",").filter(Boolean):[])},b=()=>{let d=a();for(let l of o()){let i=l.getAttribute("value")||"";if(d.has(i))l.setAttribute("open","");else l.removeAttribute("open")}},c=D(()=>{r.value(),b()}),f=(d)=>{let l=d;l.stopPropagation();let i=l.detail?.value,u=a();if(r.type()==="single")r.value.set(u.has(i)?"":i);else{if(u.has(i))u.delete(i);else u.add(i);r.value.set(Array.from(u).join(","))}n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:r.value()},bubbles:!0,composed:!0}))},p=(d)=>{let l=d;l.stopPropagation();let i=t();if(!i.length)return;let u=l.target?.closest("sui-accordion-item"),s=u?i.indexOf(u):-1,m;switch(l.detail?.direction){case"next":m=i[(s+1)%i.length];break;case"prev":m=i[(s-1+i.length)%i.length];break;case"first":m=i[0];break;case"last":m=i[i.length-1];break}if(m)m.shadowRoot?.querySelector("button")?.focus()};return n.addEventListener("sui-accordion-item-toggle",f),n.addEventListener("sui-accordion-nav",p),()=>{c(),n.removeEventListener("sui-accordion-item-toggle",f),n.removeEventListener("sui-accordion-nav",p)}}});var fm=j("sui-accordion",{props:{type:g.string({default:"single",reflect:!0}),value:g.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host { display: block; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.type=n.type,t.value=n.value;let a=null;if(Xn&&typeof Xn.setup==="function"){if(e.scope=t,a=Xn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});var xa=0,Rn=O({mount(e,n){let r=e,o=++xa,t=`sui-acc-trigger-${o}`,a=`sui-acc-content-${o}`,b=D(()=>{let l=r.triggerBtn(),i=r.contentWrap();if(l)l.id=t,l.setAttribute("aria-controls",a);if(i)i.id=a,i.setAttribute("aria-labelledby",t)}),c=D(()=>{let l=r.triggerBtn();if(l)l.setAttribute("aria-expanded",r.open()?"true":"false")}),f=()=>{if(r.disabled())return;n.dispatchEvent(new CustomEvent("sui-accordion-item-toggle",{detail:{value:r.value()},bubbles:!0,composed:!0}))},p=(l)=>{if(l.key==="ArrowDown"||l.key==="ArrowUp")l.preventDefault(),n.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:l.key==="ArrowDown"?"next":"prev"},bubbles:!0,composed:!0}));else if(l.key==="Home"||l.key==="End")l.preventDefault(),n.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:l.key==="Home"?"first":"last"},bubbles:!0,composed:!0}))},d=D(()=>{let l=r.triggerBtn();if(!l)return;return l.addEventListener("click",f),l.addEventListener("keydown",p),()=>{l.removeEventListener("click",f),l.removeEventListener("keydown",p)}});return()=>{b(),c(),d()}}});var ym=j("sui-accordion-item",{props:{value:g.string({default:"",reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),open:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    color: var(--foreground);
  }

  .trigger {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 10px 0;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--border);
    text-align: left;
    font: inherit;
    font-size: var(--text-base);
    font-weight: var(--font-weight-medium);
    color: var(--foreground);
    cursor: pointer;
    gap: 8px;
  }
  .trigger:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .trigger:focus-visible {
    outline: var(--ring-width) solid var(--ring-color);
    outline-offset: var(--ring-offset);
  }
  @media (forced-colors: active) {
    .trigger:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

  .trigger-label { flex: 1; }

  .chevron {
    flex-shrink: 0;
    width: 16px;
    height: 16px;
    color: var(--muted-foreground);
    transition: transform var(--duration-medium) var(--ease-out);
  }
  :host([open]) .chevron { transform: rotate(180deg); }
  @media (prefers-reduced-motion: reduce) { .chevron { transition: none; } }

  .content-wrap {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows var(--duration-medium) var(--ease-out);
  }
  :host([open]) .content-wrap { grid-template-rows: 1fr; }
  @media (prefers-reduced-motion: reduce) { .content-wrap { transition: none; } }

  .content-inner { overflow: hidden; }
  .content-body { padding: 12px 0; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.disabled=n.disabled,t.open=n.open;let a=null;if(Rn&&typeof Rn.setup==="function"){if(e.scope=t,a=Rn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("button");c.setAttribute("class","trigger"),c.setAttribute("part","trigger"),T(t,"triggerBtn",c),o.push(E(c,"disabled",()=>t.disabled()));let f=document.createElement("span");f.setAttribute("class","trigger-label");let p=document.createElement("slot");p.setAttribute("name","trigger"),f.appendChild(p),c.appendChild(f);let d=document.createElement("svg");d.setAttribute("class","chevron"),d.setAttribute("part","chevron"),d.setAttribute("viewbox","0 0 16 16"),d.setAttribute("fill","none"),d.setAttribute("stroke","currentColor"),d.setAttribute("stroke-width","1.75"),d.setAttribute("stroke-linecap","round"),d.setAttribute("stroke-linejoin","round"),d.setAttribute("aria-hidden","true");let l=document.createElement("path");l.setAttribute("d","M4 6l4 4 4-4"),d.appendChild(l),c.appendChild(d),b.appendChild(c);let i=document.createElement("div");i.setAttribute("class","content-wrap"),i.setAttribute("part","content"),T(t,"contentWrap",i),i.setAttribute("role","region");let u=document.createElement("div");u.setAttribute("class","content-inner");let s=document.createElement("div");s.setAttribute("class","content-body");let m=document.createElement("slot");if(s.appendChild(m),u.appendChild(s),i.appendChild(u),b.appendChild(i),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(y,w){return y[w]},set(y,w,k){let A=y[w];if(A!=null&&typeof A.set==="function"){if(typeof k==="function")A.set(()=>k);else A.set(k);return!0}return!1}});if(h.mount){let y=h.mount(v,r);if(typeof y==="function")o.push(y)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var Qr=`
sui-list-tile[variant='row'] + sui-list-tile[variant='row'] {
  border-top: 1px solid var(--border-subtle);
}
sui-list-tile[variant='row'][selected] + sui-list-tile[variant='row'],
sui-list-tile[variant='row'] + sui-list-tile[variant='row'][selected] {
  border-top: none;
}
`,Vr=new WeakSet;function wa(e){if(Vr.has(e))return;Vr.add(e);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let r=new CSSStyleSheet;r.replaceSync(Qr),e.adoptedStyleSheets=[...e.adoptedStyleSheets,r];return}}catch{}let n=e.createElement("style");n.setAttribute("data-sui-list-tile-dividers",""),n.textContent=Qr,e.head.appendChild(n)}var qn=O({mount(e,n){let r=e;wa(n.ownerDocument);let o=()=>{if(r.navigable())return"link";if(r.interactive())return"button";return"listitem"},t=D(()=>{r.navigable(),r.interactive(),n.setAttribute("role",o())}),a=D(()=>{n.setAttribute("aria-selected",r.selected()?"true":"false")}),b=D(()=>{n.setAttribute("aria-disabled",r.disabled()?"true":"false")}),c=D(()=>{let i=r.tileEl();if(!i)return;let u=r.interactive(),s=r.disabled();if(u&&!s)i.setAttribute("tabindex","0");else i.removeAttribute("tabindex")}),f=()=>{if(r.disabled()||!r.interactive())return!1;let i=new CustomEvent("sui-select",{detail:{},bubbles:!0,composed:!0,cancelable:!0});return n.dispatchEvent(i),i.defaultPrevented},p=(i)=>{if(!r.interactive())return;if(f()&&r.navigable())i.preventDefault()},d=(i)=>{if(!r.interactive()||r.disabled())return;if(r.navigable()&&i.key==="Enter")return;if(i.key==="Enter"||i.key===" ")i.preventDefault(),f()},l=D(()=>{let i=r.tileEl();if(!i)return;return i.addEventListener("click",p),i.addEventListener("keydown",d),()=>{i.removeEventListener("click",p),i.removeEventListener("keydown",d)}});return()=>{t(),a(),b(),c(),l()}}});var Cm=j("sui-list-tile",{props:{variant:g.string({default:"row",reflect:!0}),interactive:g.bool({default:!1,reflect:!0}),navigable:g.string({default:"",reflect:!0}),selected:g.bool({default:!1,reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),density:g.string({default:"normal",reflect:!0}),tone:g.string({default:"default",reflect:!0}),target:g.string({default:"",reflect:!0}),rel:g.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    --pad-y: 10px;
    --pad-x: 10px;
  }
  :host([density='compact']) { --pad-y: 6px;  --pad-x: 8px; }
  :host([density='relaxed']) { --pad-y: 16px; --pad-x: 14px; }

  .tile {
    width: 100%;
    text-align: left;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    padding: var(--pad-y) var(--pad-x);
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: inherit;
    font: inherit;
    text-decoration: none;
    box-sizing: border-box;
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }
  a.tile { display: flex; }

  :host([variant='card']) { height: 100%; box-sizing: border-box; }
  :host([variant='card']) .tile {
    min-height: 100%;
    box-sizing: border-box;
  }
  :host([variant='card']) .row { flex: 1; align-items: stretch; }
  :host([variant='card']) ::slotted([slot='leading']) { align-self: flex-start; }
  :host([variant='card']) .primary {
    display: flex;
    flex-direction: column;
    align-self: stretch;
  }

  :host([variant='row']) .tile:hover,
  :host(:not([variant])) .tile:hover {
    background: var(--muted);
    border-color: var(--border);
  }

  :host([variant='card']) .tile {
    background: var(--card);
    border-color: var(--border-subtle);
    box-shadow: var(--shadow-card-rest);
    backdrop-filter: var(--card-backdrop-filter);
    -webkit-backdrop-filter: var(--card-backdrop-filter);
    position: relative;
  }
  :host([variant='card']) .tile::after {
    content: '';
    position: absolute;
    inset: 0;
    pointer-events: none;
    background: var(--card-texture);
    mix-blend-mode: var(--card-texture-blend);
    opacity: var(--card-texture-opacity);
    border-radius: inherit;
    z-index: 0;
  }
  :host([variant='card']) .tile > * { position: relative; z-index: 1; }
  @media (prefers-reduced-motion: reduce) {
    :host([variant='card']) .tile::after { background: none; }
  }
  :host([variant='card']) .tile:hover {
    border-color: var(--primary);
    box-shadow: var(--shadow-hover);
    transform: translateY(-1px);
  }
  @media (prefers-reduced-motion: reduce) {
    :host([variant='card']) .tile:hover { transform: none; }
  }
  :host([variant='card'][interactive]) .tile:active {
    transform: translateY(0);
    box-shadow: var(--shadow-card-rest);
    transition-duration: 60ms;
  }

  .tile:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset) var(--background),
      0 0 0 calc(var(--ring-offset) + var(--ring-width))
        color-mix(in oklch, var(--ring-color) var(--ring-opacity), transparent);
  }
  @media (forced-colors: active) {
    .tile:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

  :host([selected]) .tile {
    background: var(--muted);
    border-color: var(--primary);
  }

  :host([disabled]) .tile {
    opacity: 0.6;
    cursor: not-allowed;
    pointer-events: none;
  }

  :host([interactive]) .tile { cursor: pointer; }

  :host([tone]:not([tone='default'])) .tile {
    border-left: 3px solid var(--tone-fg);
    padding-left: calc(var(--pad-x) - 2px);
  }

  /* tone-fg mapping */
  :host { --tone-fg: var(--foreground); }
  :host([tone='primary'])     { --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone-fg: var(--info); }
  :host([tone='success'])     { --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }

  .row { display: flex; align-items: center; gap: 8px; }
  .primary {
    flex: 1;
    min-width: 0;
    font-weight: var(--font-weight-medium);
    font-size: var(--text-base);
  }
  .secondary {
    display: flex;
    gap: 6px;
    align-items: center;
    font-size: 10px;
    color: var(--muted-foreground);
  }
  .secondary:empty { display: none; }

  ::slotted([slot='leading']),
  ::slotted([slot='trailing']) {
    display: inline-flex;
    align-items: center;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.variant=n.variant,t.interactive=n.interactive,t.navigable=n.navigable,t.selected=n.selected,t.disabled=n.disabled,t.density=n.density,t.tone=n.tone,t.target=n.target,t.rel=n.rel;let a=null;if(qn&&typeof qn.setup==="function"){if(e.scope=t,a=qn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(p)=>H({anchor:p,test:()=>{if(t.navigable())return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("a");i.setAttribute("class","tile"),i.setAttribute("part","button"),T(t,"tileEl",i),l.push(E(i,"href",()=>t.navigable())),l.push(E(i,"target",()=>t.target()||null)),l.push(E(i,"rel",()=>t.rel()||null));let u=document.createElement("div");u.setAttribute("class","row"),u.setAttribute("part","row");let s=document.createElement("slot");s.setAttribute("name","leading"),u.appendChild(s);let m=document.createElement("span");m.setAttribute("class","primary"),m.setAttribute("part","primary");let h=document.createElement("slot");m.appendChild(h),u.appendChild(m);let v=document.createElement("span");v.setAttribute("part","trailing");let y=document.createElement("slot");y.setAttribute("name","trailing"),v.appendChild(y),u.appendChild(v),i.appendChild(u);let w=document.createElement("div");w.setAttribute("class","secondary"),w.setAttribute("part","secondary");let k=document.createElement("slot");return k.setAttribute("name","meta"),w.appendChild(k),i.appendChild(w),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let A of l)try{A()}catch(x){}}}},()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("div");i.setAttribute("class","tile"),i.setAttribute("part","button"),T(t,"tileEl",i);let u=document.createElement("div");u.setAttribute("class","row"),u.setAttribute("part","row");let s=document.createElement("slot");s.setAttribute("name","leading"),u.appendChild(s);let m=document.createElement("span");m.setAttribute("class","primary"),m.setAttribute("part","primary");let h=document.createElement("slot");m.appendChild(h),u.appendChild(m);let v=document.createElement("span");v.setAttribute("part","trailing");let y=document.createElement("slot");y.setAttribute("name","trailing"),v.appendChild(y),u.appendChild(v),i.appendChild(u);let w=document.createElement("div");w.setAttribute("class","secondary"),w.setAttribute("part","secondary");let k=document.createElement("slot");return k.setAttribute("name","meta"),w.appendChild(k),i.appendChild(w),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let A of l)try{A()}catch(x){}}}}]}),c=r.shadowRoot??r;L(c);let f=document.createComment("if");if(c.appendChild(f),o.push(b(f)),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var Jn=O({scope:(e,n)=>{let r=P(null),o=P(!1);return{caughtError:r,hasFallbackSlot:o,reset:()=>{if(r.set(null),n instanceof HTMLElement)n.removeAttribute("data-error-message")}}},mount(e,n){let{caughtError:r,hasFallbackSlot:o,reset:t}=e;if(!r||!o||!t)throw Error("[sui-error-boundary] scope signals missing — controller wiring is broken.");let a=n.querySelectorAll(':scope > template[slot="fallback"]');for(let b of Array.from(a)){let c=document.createElement("div");c.setAttribute("slot","fallback"),c.appendChild(b.content.cloneNode(!0)),b.replaceWith(c)}return o.set(!!n.querySelector(':scope > [slot="fallback"]')),Ne(n,{catch(b,c){if(r()!==null)return;let f=b instanceof Error?b:Error(String(b));n.setAttribute("data-error-message",f.message),r.set(f),n.dispatchEvent(new CustomEvent("sui-error-boundary:caught",{bubbles:!0,composed:!0,detail:{error:f,source:c}}))},reset:t}),n.reset=t,()=>{He(n),delete n.reset}}});var Tm=j("sui-error-boundary",{props:{"fallback-text":g.string({default:""})},shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
  }

  .fallback {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-2, 8px);
    padding: var(--space-3, 12px) var(--space-4, 16px);
    border: 1px solid color-mix(in oklch, var(--destructive) 30%, var(--border));
    border-radius: var(--radius);
    background: color-mix(in oklch, var(--destructive) 6%, var(--background));
    color: var(--foreground);
    font-size: var(--text-sm, 13px);
  }

  .message {
    color: var(--muted-foreground);
  }

  .retry {
    appearance: none;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 4px 10px;
    font: inherit;
    font-size: var(--text-xs, 12px);
    color: var(--foreground);
    cursor: pointer;
  }
  .retry:hover {
    background: var(--muted);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.fallbackText=n.fallbackText;let a=null;if(Jn&&typeof Jn.setup==="function"){if(e.scope=t,a=Jn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>H({anchor:d,test:()=>{if(t.hasFallbackSlot())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("slot");return u.setAttribute("name","fallback"),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(m){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("div");u.setAttribute("class","fallback"),u.setAttribute("part","fallback"),u.setAttribute("role","alert");let s=document.createElement("div");s.setAttribute("class","message"),s.setAttribute("part","message");let m=document.createTextNode("");s.appendChild(m),i.push(F(m,()=>t.fallbackText()||"Something went wrong")),u.appendChild(s);let h=document.createElement("button");h.setAttribute("class","retry"),h.setAttribute("part","retry"),h.setAttribute("type","button"),i.push(V(h,"click",function(){let y=t.reset;return typeof y==="function"?y.bind(void 0):()=>{}}(),{modifiers:[]}));let v=document.createTextNode("Retry");return h.appendChild(v),u.appendChild(h),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let y of i)try{y()}catch(w){}}}}]}),c=(d)=>H({anchor:d,test:()=>{if(t.caughtError()===null)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("slot");return l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(m){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createComment("if");return l.appendChild(u),i.push(b(u)),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(m){}}}}]}),f=r.shadowRoot??r;L(f);let p=document.createComment("if");if(f.appendChild(p),o.push(c(p)),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var Yn=O({mount(e,n){let r=e,o=[],t=null,a=()=>{let i=r.target()||"body",u=document.querySelector(i);if(!u)return console.warn(`[sui-portal] target "${i}" not found in document`),null;return u},b=(i)=>{let u=r.portalSlot(),s=u?u.assignedNodes():Array.from(n.childNodes);for(let m of s)if(i.appendChild(m),!o.includes(m))o.push(m)},c=(i)=>{for(let u of o)i.appendChild(u)},f=()=>{for(let i of o)if(i.parentNode)i.parentNode.removeChild(i);o.length=0},p=D(()=>{let i=a();if(!i){t=null;return}if(t===null)b(i);else if(t!==i)c(i);t=i}),d=r.portalSlot(),l=()=>{if(!t||!d)return;for(let i of d.assignedNodes())if(!o.includes(i))t.appendChild(i),o.push(i)};if(d)d.addEventListener("slotchange",l);return()=>{if(p(),d)d.removeEventListener("slotchange",l);f()}}});var Fm=j("sui-portal",{props:{target:g.string({default:"body"})},shadow:!0,styles:`
  :host { display: none; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.target=n.target;let a=null;if(Yn&&typeof Yn.setup==="function"){if(e.scope=t,a=Yn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(T(t,"portalSlot",c),b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});var Wn=O({mount(e,n){let r=e,o=!1,t=(f)=>{f.textContent=o?"Hide details":"Show details",f.setAttribute("aria-expanded",o?"true":"false")},a=D(()=>{let f=r.detailsSlot();if(!f)return;let p=()=>{if(f.assignedNodes({flatten:!0}).some((l)=>l.nodeType===Node.ELEMENT_NODE||l.nodeType===Node.TEXT_NODE&&(l.textContent??"").trim().length>0))n.setAttribute("data-has-details","");else{n.removeAttribute("data-has-details"),n.removeAttribute("data-details-open"),o=!1;let l=r.detailsToggleBtn();if(l)t(l)}};return f.addEventListener("slotchange",p),p(),()=>f.removeEventListener("slotchange",p)}),b=D(()=>{let f=r.detailsToggleBtn();if(!f)return;t(f);let p=()=>{if(o=!o,o)n.setAttribute("data-details-open","");else n.removeAttribute("data-details-open");t(f)};return f.addEventListener("click",p),()=>f.removeEventListener("click",p)}),c=D(()=>{let f=r.actionSlot();if(!f)return;let p=()=>{let d=f.assignedElements().find((l)=>l.getAttribute("slot")==="retry");if(d)requestAnimationFrame(()=>d.focus?.())};return f.addEventListener("slotchange",p),()=>f.removeEventListener("slotchange",p)});return()=>{a(),b(),c()}}});var qm=j("sui-error-state",{props:{title:g.string({default:""}),body:g.string({default:""}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:g.union(["default","inline"],{default:"default",reflect:!0}),icon:g.string({default:"alert-circle"})},shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    --tone-fg: var(--destructive);
    --es-pad-y: 64px;
    --es-pad-x: 16px;
    --es-gap: var(--space-3, 12px);
    --es-icon: 22px;
    --es-tile: 48px;
    --es-title-fs: var(--text-md, 16px);
    --es-desc-fs: var(--text-sm, 14px);
  }

  :host([size='sm']) {
    --es-pad-y: var(--space-7, 28px);
    --es-pad-x: var(--space-4, 16px);
    --es-gap: var(--space-2, 8px);
    --es-icon: 16px; --es-tile: 36px;
    --es-title-fs: var(--text-base, 14px);
  }
  :host([size='lg']) {
    --es-pad-y: 96px;
    --es-pad-x: var(--space-6, 24px);
    --es-gap: var(--space-4, 16px);
    --es-icon: 28px; --es-tile: 64px;
    --es-title-fs: var(--text-lg, 18px);
    --es-desc-fs: var(--text-base, 15px);
  }

  .box {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--es-gap);
    padding: var(--es-pad-y) var(--es-pad-x);
    text-align: center;
    background: var(--card);
    border: 1px dashed var(--border);
    border-radius: var(--radius-lg);
  }
  :host([variant='inline']) .box {
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    text-align: left;
    background: color-mix(in oklch, var(--destructive) 4%, var(--card));
    border: 1px solid color-mix(in oklch, var(--destructive) 24%, var(--border));
    border-radius: var(--radius-md);
    padding: var(--space-3, 12px) var(--space-4, 16px);
  }

  .icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: var(--es-tile);
    height: var(--es-tile);
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--destructive) 18%, var(--muted));
    color: var(--destructive);
    line-height: 1;
  }
  :host([variant='inline']) .icon {
    width: var(--es-icon);
    height: var(--es-icon);
    background: transparent;
  }

  .copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: center;
    min-width: 0;
  }
  :host([variant='inline']) .copy { align-items: flex-start; }

  .title {
    font-size: var(--es-title-fs);
    font-weight: var(--font-weight-semibold, 600);
    color: var(--foreground);
    line-height: var(--leading-tight, 1.25);
  }

  .desc {
    font-size: var(--es-desc-fs);
    max-width: 60ch;
    color: var(--muted-foreground);
  }

  /* details-toggle: hidden until controller detects slot content */
  .details-toggle {
    display: none;
    appearance: none;
    background: transparent;
    border: 0;
    font: inherit;
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    cursor: pointer;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    margin-top: var(--space-1);
  }
  :host([data-has-details]) .details-toggle { display: inline-block; }
  .details-toggle:hover { color: var(--foreground); background: var(--muted); }

  /* details panel: always in DOM for slotchange detection, shown when open */
  .details {
    display: none;
    margin-top: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: color-mix(in oklch, var(--destructive) 6%, var(--muted));
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    white-space: pre-wrap;
    word-break: break-word;
    max-width: 60ch;
    text-align: left;
  }
  :host([data-has-details][data-details-open]) .details { display: block; }

  .actions {
    display: flex;
    gap: var(--space-2, 8px);
    justify-content: center;
    margin-top: 4px;
    flex-wrap: wrap;
  }
  :host([variant='inline']) .actions {
    margin-top: 0;
    margin-left: auto;
    justify-content: flex-end;
  }

  @media (forced-colors: active) {
    .box { border-color: CanvasText; }
    .icon { background: Canvas; color: CanvasText; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.title=n.title,t.body=n.body,t.size=n.size,t.variant=n.variant,t.icon=n.icon;let a=null;if(Wn&&typeof Wn.setup==="function"){if(e.scope=t,a=Wn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","box"),c.setAttribute("part","box"),c.setAttribute("role","alert");let f=document.createElement("div");f.setAttribute("class","icon"),f.setAttribute("part","icon"),f.setAttribute("aria-hidden","true");let p=document.createElement("sui-icon");o.push(E(p,"name",()=>t.icon())),p.setAttribute("size","md"),f.appendChild(p),c.appendChild(f);let d=document.createElement("div");d.setAttribute("class","copy");let l=document.createElement("div");l.setAttribute("class","title"),l.setAttribute("part","title");let i=document.createElement("slot");i.setAttribute("name","title");let u=document.createTextNode("");i.appendChild(u),o.push(F(u,()=>t.title()||"Something went wrong")),l.appendChild(i),d.appendChild(l);let s=document.createElement("div");s.setAttribute("class","desc"),s.setAttribute("part","desc");let m=document.createElement("slot"),h=document.createTextNode("");m.appendChild(h),o.push(F(h,()=>t.body())),s.appendChild(m),d.appendChild(s);let v=document.createElement("button");v.setAttribute("class","details-toggle"),T(t,"detailsToggleBtn",v),v.setAttribute("type","button"),d.appendChild(v);let y=document.createElement("div");y.setAttribute("class","details"),T(t,"detailsPanel",y);let w=document.createElement("slot");w.setAttribute("name","details"),T(t,"detailsSlot",w),y.appendChild(w),d.appendChild(y),c.appendChild(d);let k=document.createElement("div");k.setAttribute("class","actions"),k.setAttribute("part","actions");let A=document.createElement("slot");if(A.setAttribute("name","action"),T(t,"actionSlot",A),k.appendChild(A),c.appendChild(k),b.appendChild(c),a!=null){let x=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(x){let C=new Proxy(t,{get(z,M){return z[M]},set(z,M,N){let S=z[M];if(S!=null&&typeof S.set==="function"){if(typeof N==="function")S.set(()=>N);else S.set(N);return!0}return!1}});if(x.mount){let z=x.mount(C,r);if(typeof z==="function")o.push(z)}if(x.unmount)o.push(()=>x.unmount(C))}}return t}});var Zn=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","progressbar");let o=D(()=>{let t=Math.max(1,Math.floor(r.total()||0)),a=Math.max(1,Math.min(t,Math.floor(r.current()||1))),b=r.label();if(n.setAttribute("aria-valuemin","1"),n.setAttribute("aria-valuemax",String(t)),n.setAttribute("aria-valuenow",String(a)),b)n.setAttribute("aria-label",b);else n.setAttribute("aria-label",`Step ${a} of ${t}`);let c=r.dotsEl();if(!c)return;c.textContent="";for(let f=1;f<=t;f++){let p=document.createElement("span");p.className="dot",p.setAttribute("part","dot"),p.setAttribute("data-state",f===a?"active":f<a?"done":"pending"),p.setAttribute("aria-hidden","true"),c.appendChild(p)}});return()=>{o()}}});var Vm=j("sui-stepper",{props:{total:g.number({default:4,reflect:!0}),current:g.number({default:1,reflect:!0}),tone:g.string({default:"",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),label:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2, 8px);
    --dot-w: 7px;
    --dot-h: 7px;
    --dot-active-w: 22px;
    --tone-fg: var(--primary);
  }

  :host([size='sm']) { --dot-w: 6px; --dot-h: 6px; --dot-active-w: 18px; }
  :host([size='lg']) { --dot-w: 9px; --dot-h: 9px; --dot-active-w: 28px; }

  /* Tone overrides */
  :host([tone='primary'])     { --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone-fg: var(--info); }
  :host([tone='success'])     { --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }

  .dot {
    width: var(--dot-w);
    height: var(--dot-h);
    border-radius: var(--radius-full);
    background: color-mix(in oklch, var(--muted-foreground) 35%, transparent);
    transition: all var(--duration-medium) var(--ease-out);
    flex-shrink: 0;
  }

  .dot[data-state='active'] {
    width: var(--dot-active-w);
    background: var(--primary);
  }
  :host([tone]) .dot[data-state='active'] {
    background: var(--tone-fg);
  }

  .dot[data-state='done'] {
    background: color-mix(in oklch, var(--primary) 55%, transparent);
  }
  :host([tone]) .dot[data-state='done'] {
    background: color-mix(in oklch, var(--tone-fg) 55%, transparent);
  }

  @media (prefers-reduced-motion: reduce) {
    .dot { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.total=n.total,t.current=n.current,t.tone=n.tone,t.size=n.size,t.label=n.label;let a=null;if(Zn&&typeof Zn.setup==="function"){if(e.scope=t,a=Zn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");if(c.setAttribute("class","dots"),T(t,"dotsEl",c),b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});var Aa=new Set(["done","failed","pending","running"]);function ka(e){return e.split(",").map((n)=>n.trim().toLowerCase()).filter((n)=>n.length>0).map((n)=>Aa.has(n)?n:"pending")}var Qn=O({scope:()=>({segments:P([]),ariaLabel:P(null)}),mount(e,n){let r=e,o=r.segments,t=r.ariaLabel,a=D(()=>{let b=ka(r.values()||"");o.set(b.map((c,f)=>({id:`s${f}`,state:c}))),t.set(b.length===0?null:`${b.length} step outcome${b.length===1?"":"s"}`)});return()=>a()}});var nb=j("sui-step-segments",{props:{values:g.string({default:"",reflect:!0}),size:g.union(["sm","md"],{default:"sm",reflect:!0})},shadow:!0,styles:`
  :host { display: inline-flex; vertical-align: middle; }

  .row { display: inline-flex; gap: 3px; align-items: center; }
  .row:empty { display: none; }

  .seg {
    display: inline-block;
    width: 18px;
    height: 6px;
    border-radius: 2px;
    background: color-mix(in oklch, var(--muted-foreground) 28%, transparent);
  }

  :host([size='md']) .seg { width: 24px; height: 8px; border-radius: 3px; }

  .seg[data-state='done']    { background: var(--success); }
  .seg[data-state='failed']  { background: var(--destructive); }
  .seg[data-state='running'] {
    background: var(--warning);
    animation: sui-step-segments-pulse var(--duration-slow, 1200ms) ease-in-out infinite;
  }

  @keyframes sui-step-segments-pulse {
    0%, 100% { opacity: 1; }
    50%      { opacity: 0.55; }
  }

  @media (prefers-reduced-motion: reduce) {
    .seg[data-state='running'] { animation: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.values=n.values,t.size=n.size;let a=null;if(Qn&&typeof Qn.setup==="function"){if(e.scope=t,a=Qn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>X({anchor:d,items:()=>t.segments(),key:(l,i)=>{let u=Object.create(t);return u.seg=l,u.__i=i,((s)=>l.id)(u)},render:(l,i)=>{let u=Object.create(t);return u.seg=l,u.__i=i,u.__v=P(0),((s)=>{let m=document.createDocumentFragment(),h=[],v=document.createElement("span");return v.setAttribute("class","seg"),v.setAttribute("part","seg"),h.push(E(v,"data-state",()=>(s.__v(),s.seg.state))),m.appendChild(v),{nodes:Array.from(m.childNodes),cleanup:()=>{for(let y of h)try{y()}catch(w){}},update:(y,w)=>{s.seg=y,s.__i=w,s.__v.set((k)=>k+1)}}})(u)}}),c=r.shadowRoot??r;L(c);let f=document.createElement("span");f.setAttribute("class","row"),f.setAttribute("part","row"),o.push(E(f,"role",()=>t.segments().length>0?"img":null)),o.push(E(f,"aria-label",()=>t.ariaLabel()));let p=document.createComment("for");if(f.appendChild(p),o.push(b(p)),c.appendChild(f),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var Vn=O({mount(e,n){if(!n.hasAttribute("role"))n.setAttribute("role","list")}});var sb=j("sui-timeline",{shadow:!0,styles:`
  :host { display: block; position: relative; }
  .list { display: flex; flex-direction: column; gap: 4px; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(Vn&&typeof Vn.setup==="function"){if(e.scope=t,a=Vn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","list"),c.setAttribute("part","list");let f=document.createElement("slot");if(c.appendChild(f),b.appendChild(c),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var Kn=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","listitem");let o=D(()=>{if(r.interactive()){if(n.setAttribute("role","button"),!n.hasAttribute("tabindex"))n.setAttribute("tabindex","0")}else n.setAttribute("role","listitem"),n.removeAttribute("tabindex")}),t=()=>{n.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},a=()=>{if(r.interactive())t()},b=(c)=>{if(!r.interactive())return;if(c.key==="Enter"||c.key===" ")c.preventDefault(),t()};return n.addEventListener("click",a),n.addEventListener("keydown",b),()=>{o(),n.removeEventListener("click",a),n.removeEventListener("keydown",b)}}});var fb=j("sui-timeline-item",{props:{tone:g.string({default:"default",reflect:!0}),at:g.string({default:"",reflect:!0}),interactive:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: grid;
    grid-template-columns: 20px 1fr;
    column-gap: 10px;
    padding: 6px 0;
    position: relative;
    --tone-fg: var(--muted-foreground);
  }

  /* Tone color mapping */
  :host([tone='default'])     { --tone-fg: var(--foreground); }
  :host([tone='primary'])     { --tone-fg: var(--primary); }
  :host([tone='info'])        { --tone-fg: var(--info); }
  :host([tone='success'])     { --tone-fg: var(--success); }
  :host([tone='warning'])     { --tone-fg: var(--warning); }
  :host([tone='destructive']) { --tone-fg: var(--destructive); }
  :host([tone='accent'])      { --tone-fg: var(--accent-foreground); }
  :host([tone='muted'])       { --tone-fg: var(--muted-foreground); }

  .gutter {
    position: relative;
    display: flex;
    justify-content: center;
  }
  .gutter::before {
    content: '';
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--border);
  }
  :host(:first-of-type) .gutter::before { top: 10px; }
  :host(:last-of-type) .gutter::before { bottom: calc(100% - 10px); }

  .dot {
    position: relative;
    width: 10px;
    height: 10px;
    border-radius: var(--radius-full);
    background: var(--tone-fg);
    border: 2px solid var(--background);
    margin-top: 4px;
    box-shadow: var(--shadow-xs);
    flex-shrink: 0;
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

  .at {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    margin-left: auto;
  }

  .content {
    font-size: var(--text-sm);
    color: var(--muted-foreground);
    line-height: var(--leading-normal);
    margin-top: 2px;
  }

  :host([interactive]) {
    cursor: pointer;
    border-radius: var(--radius-sm);
    outline: none;
  }
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.at=n.at,t.interactive=n.interactive;let a=null;if(Kn&&typeof Kn.setup==="function"){if(e.scope=t,a=Kn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(h)=>H({anchor:h,test:()=>{if(t.at())return 0;return-1},branches:[()=>{let v=document.createDocumentFragment(),y=[],w=document.createElement("span");w.setAttribute("class","at"),w.setAttribute("part","at");let k=document.createTextNode("");return w.appendChild(k),y.push(F(k,()=>t.at())),v.appendChild(w),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let A of y)try{A()}catch(x){}}}}]}),c=r.shadowRoot??r;L(c);let f=document.createElement("div");f.setAttribute("class","gutter");let p=document.createElement("span");p.setAttribute("class","dot"),p.setAttribute("part","dot"),f.appendChild(p),c.appendChild(f);let d=document.createElement("div");d.setAttribute("class","body"),d.setAttribute("part","body");let l=document.createElement("div");l.setAttribute("class","head");let i=document.createElement("slot");i.setAttribute("name","title"),l.appendChild(i);let u=document.createComment("if");l.appendChild(u),o.push(b(u)),d.appendChild(l);let s=document.createElement("div");s.setAttribute("class","content");let m=document.createElement("slot");if(s.appendChild(m),d.appendChild(s),c.appendChild(d),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(y,w){return y[w]},set(y,w,k){let A=y[w];if(A!=null&&typeof A.set==="function"){if(typeof k==="function")A.set(()=>k);else A.set(k);return!0}return!1}});if(h.mount){let y=h.mount(v,r);if(typeof y==="function")o.push(y)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var Ea=["Waking daemon","Warming routes","Fetching state","Hydrating shell"];function Kr(e){let n=e.trim();if(!n)return Ea;return n.split("|").map((r)=>r.trim()).filter(Boolean)}var In=O({mount(e,n){let r=e,o=r.msgEl();if(!o)return;let t=0;(()=>{let c=Kr(r.messages());o.textContent=c[t]??"",o.style.opacity="1"})();let b=setInterval(()=>{o.style.opacity="0",requestAnimationFrame(()=>{let c=Kr(r.messages());t=(t+1)%c.length,o.textContent=c[t]??"",o.style.opacity="1"})},1400);return()=>clearInterval(b)}});var yb=j("sui-loading",{props:{messages:g.string({default:""}),compact:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3, 12px);
    min-height: 240px;
    width: 100%;
    color: var(--muted-foreground);
  }
  :host([compact]) {
    min-height: 80px;
    gap: 6px;
  }

  /* Stage: fixed footprint so bob/flame animations don't shift the message. */
  .stage {
    position: relative;
    width: 56px;
    height: 64px;
    display: grid;
    place-items: center;
  }
  :host([compact]) .stage {
    width: 28px;
    height: 32px;
  }

  .rocket {
    position: relative;
    z-index: 2;
    width: 32px;
    height: 32px;
    color: var(--primary);
    filter: drop-shadow(0 4px 6px color-mix(in oklch, var(--primary) 35%, transparent));
    animation: sui-loading-rocket 2.2s ease-in-out infinite;
    transform-origin: 50% 100%;
  }
  :host([compact]) .rocket {
    width: 18px;
    height: 18px;
    filter: none;
  }
  .rocket svg {
    width: 100%;
    height: 100%;
  }

  .flame {
    position: absolute;
    left: 50%;
    bottom: 6px;
    transform: translateX(-50%);
    width: 14px;
    height: 18px;
    z-index: 1;
    background: radial-gradient(
      ellipse at 50% 30%,
      color-mix(in oklch, var(--primary) 95%, white) 0%,
      color-mix(in oklch, var(--primary) 70%, transparent) 55%,
      transparent 80%
    );
    border-radius: 50% 50% 35% 35% / 60% 60% 35% 35%;
    filter: blur(0.5px);
    transform-origin: 50% 0%;
    animation:
      sui-loading-flame 0.18s ease-in-out infinite alternate,
      sui-loading-flame-pulse 2.2s ease-in-out infinite;
  }
  :host([compact]) .flame {
    width: 8px;
    height: 10px;
    bottom: 4px;
  }

  .particle {
    position: absolute;
    left: 50%;
    top: 75%;
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: color-mix(in oklch, var(--primary) 85%, transparent);
    opacity: 0;
    z-index: 0;
  }
  .p1 { animation: sui-loading-particle 1.4s ease-out 0s    infinite; }
  .p2 { animation: sui-loading-particle 1.7s ease-out 0.4s  infinite; width: 2px; height: 2px; transform: translate(-50%, 0); }
  .p3 { animation: sui-loading-particle 1.5s ease-out 0.8s  infinite; width: 4px; height: 4px; }
  :host([compact]) .particle { display: none; }

  .message {
    font-size: var(--text-sm);
    color: var(--muted-foreground);
    transition: opacity var(--duration-medium, 220ms) var(--ease-out, ease);
    letter-spacing: 0.01em;
  }
  :host([compact]) .message { font-size: var(--text-xs); }

  @keyframes sui-loading-rocket {
    0%   { transform: translateY(0)    rotate(0deg);    }
    40%  { transform: translateY(-6px) rotate(-1.5deg); }
    60%  { transform: translateY(-7px) rotate(1.5deg);  }
    100% { transform: translateY(0)    rotate(0deg);    }
  }
  @keyframes sui-loading-flame {
    0%   { transform: translateX(-50%) scaleY(1)    scaleX(1);    opacity: 0.9; }
    100% { transform: translateX(-50%) scaleY(1.15) scaleX(0.92); opacity: 1;   }
  }
  @keyframes sui-loading-flame-pulse {
    0%, 100% { height: 16px; }
    50%      { height: 22px; }
  }
  @keyframes sui-loading-particle {
    0%   { transform: translate(-50%, 0)    scale(1);   opacity: 0.7; }
    80%  { transform: translate(-50%, 22px) scale(0.5); opacity: 0;   }
    100% { transform: translate(-50%, 26px) scale(0.3); opacity: 0;   }
  }

  @media (prefers-reduced-motion: reduce) {
    .rocket, .flame, .particle { animation: none; }
    .flame { opacity: 0.7; }
    .particle { display: none; }
    .message { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.messages=n.messages,t.compact=n.compact;let a=null;if(In&&typeof In.setup==="function"){if(e.scope=t,a=In.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","stage"),c.setAttribute("part","stage"),c.setAttribute("aria-hidden","true");let f=document.createElement("div");f.setAttribute("class","rocket");let p=document.createElement("svg");p.setAttribute("viewbox","0 0 32 40"),p.setAttribute("fill","none"),p.setAttribute("xmlns","http://www.w3.org/2000/svg");let d=document.createElement("path");d.setAttribute("d","M16 2 C20 6 22 11 22 16 H10 C10 11 12 6 16 2 Z"),d.setAttribute("fill","currentColor"),p.appendChild(d);let l=document.createElement("rect");l.setAttribute("x","10"),l.setAttribute("y","16"),l.setAttribute("width","12"),l.setAttribute("height","14"),l.setAttribute("rx","2"),l.setAttribute("fill","currentColor"),l.setAttribute("opacity","0.9"),p.appendChild(l);let i=document.createElement("circle");i.setAttribute("cx","16"),i.setAttribute("cy","20"),i.setAttribute("r","2.5"),i.setAttribute("fill","var(--background, #fff)"),i.setAttribute("opacity","0.85"),p.appendChild(i);let u=document.createElement("path");u.setAttribute("d","M10 24 L4 32 L10 30 Z"),u.setAttribute("fill","currentColor"),u.setAttribute("opacity","0.75"),p.appendChild(u);let s=document.createElement("path");s.setAttribute("d","M22 24 L28 32 L22 30 Z"),s.setAttribute("fill","currentColor"),s.setAttribute("opacity","0.75"),p.appendChild(s);let m=document.createElement("rect");m.setAttribute("x","13"),m.setAttribute("y","29"),m.setAttribute("width","6"),m.setAttribute("height","2"),m.setAttribute("rx","0.5"),m.setAttribute("fill","var(--background, #000)"),m.setAttribute("opacity","0.5"),p.appendChild(m),f.appendChild(p),c.appendChild(f);let h=document.createElement("div");h.setAttribute("class","flame"),c.appendChild(h);let v=document.createElement("div");v.setAttribute("class","particle p1"),c.appendChild(v);let y=document.createElement("div");y.setAttribute("class","particle p2"),c.appendChild(y);let w=document.createElement("div");w.setAttribute("class","particle p3"),c.appendChild(w),b.appendChild(c);let k=document.createElement("div");if(k.setAttribute("class","message"),k.setAttribute("part","message"),T(t,"msgEl",k),b.appendChild(k),a!=null){let A=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(A){let x=new Proxy(t,{get(C,z){return C[z]},set(C,z,M){let N=C[z];if(N!=null&&typeof N.set==="function"){if(typeof M==="function")N.set(()=>M);else N.set(M);return!0}return!1}});if(A.mount){let C=A.mount(x,r);if(typeof C==="function")o.push(C)}if(A.unmount)o.push(()=>A.unmount(x))}}return t}});function Ir(e){let n=document.getElementById("ship-view-root");if(!n)return null;let r=e.target;if(!r||!n.contains(r))return null;if(r.closest("sui-edit-overlay"))return null;return r}function $r(e){let n=e.tagName.toLowerCase(),r=e.id||null,o=n;if(r)o=`#${r}`;else if(e.classList.length>0)o=`${n}.${Array.from(e.classList).slice(0,2).join(".")}`;return{tag:n,selector:o,id:r}}var $n=O({mount(e,n){let r=e,o=r.boxEl(),t=r.labelEl(),a=null;function b(s){if(!o||!t)return;if(!s){o.classList.remove("visible"),t.classList.remove("visible");return}o.style.cssText=`top:${s.top}px;left:${s.left}px;width:${s.width}px;height:${s.height}px`,o.classList.add("visible"),t.style.cssText=`top:${s.top}px;left:${s.left}px`,t.textContent=`<${s.tag}${s.id?`#${s.id}`:""}>`,t.classList.add("visible")}let c=(s)=>{if(a)return;a=requestAnimationFrame(()=>{a=null;let m=Ir(s);if(!m){b(null);return}let h=m.getBoundingClientRect(),{tag:v,id:y}=$r(m);b({top:h.top,left:h.left,width:h.width,height:h.height,tag:v,id:y})})},f=()=>{b(null)},p=(s)=>{let m=Ir(s);if(!m)return;s.preventDefault(),s.stopPropagation();let{tag:h,selector:v,id:y}=$r(m);(n.closest("sui-shell")??n.closest("ship-shell"))?.dispatchEvent(new CustomEvent("sui:chat:annotate",{detail:{tag:h,selector:v,id:y,preview:(m.textContent??"").trim().slice(0,60)},bubbles:!0})),window.ship?.chat?.open?.()};function d(){let s=document.getElementById("ship-view-root");if(!s)return;s.addEventListener("pointermove",c,!0),s.addEventListener("pointerleave",f,!0),s.addEventListener("click",p,!0)}function l(){let s=document.getElementById("ship-view-root");if(!s)return;s.removeEventListener("pointermove",c,!0),s.removeEventListener("pointerleave",f,!0),s.removeEventListener("click",p,!0)}function i(){let s=document.body.classList.contains("edit-mode");if(n.classList.toggle("active",s),s)d();else if(l(),b(null),a)cancelAnimationFrame(a),a=null}i();let u=new MutationObserver(()=>i());return u.observe(document.body,{attributes:!0,attributeFilter:["class"]}),()=>{if(u.disconnect(),l(),a)cancelAnimationFrame(a),a=null}}});var Cb=j("sui-edit-overlay",{shadow:!0,styles:`
  :host {
    position: fixed;
    inset: 0;
    pointer-events: none;
    z-index: 9500;
    display: none;
  }
  :host(.active) {
    display: block;
  }

  .box {
    position: absolute;
    border: 2px solid var(--primary);
    background: color-mix(in oklch, var(--primary) 8%, transparent);
    border-radius: 2px;
    box-shadow: 0 0 0 1px color-mix(in oklch, var(--primary) 40%, transparent);
    transition: all 60ms linear;
    display: none;
  }
  .box.visible { display: block; }

  .label {
    position: absolute;
    padding: 2px 6px;
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--primary-foreground, #fff);
    background: var(--primary);
    border-radius: 2px;
    white-space: nowrap;
    transform: translateY(-100%);
    display: none;
  }
  .label.visible { display: block; }

  .hint {
    position: fixed;
    bottom: 40px;
    left: 50%;
    transform: translateX(-50%);
    padding: 6px 12px;
    background: var(--card, var(--background));
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
  }
  .hint kbd {
    font-family: var(--font-mono);
    background: var(--muted);
    padding: 1px 4px;
    border-radius: 3px;
    border: 1px solid var(--border);
    margin: 0 2px;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if($n&&typeof $n.setup==="function"){if(e.scope=t,a=$n.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","box"),c.setAttribute("part","box"),T(t,"boxEl",c),b.appendChild(c);let f=document.createElement("div");f.setAttribute("class","label"),f.setAttribute("part","label"),T(t,"labelEl",f),b.appendChild(f);let p=document.createElement("div");p.setAttribute("class","hint"),p.setAttribute("part","hint");let d=document.createTextNode(`
  Edit mode — hover an element, click to annotate.
  `);p.appendChild(d);let l=document.createElement("kbd"),i=document.createTextNode("⌘⇧E");l.appendChild(i),p.appendChild(l);let u=document.createTextNode(` to exit.
`);if(p.appendChild(u),b.appendChild(p),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let m=new Proxy(t,{get(h,v){return h[v]},set(h,v,y){let w=h[v];if(w!=null&&typeof w.set==="function"){if(typeof y==="function")w.set(()=>y);else w.set(y);return!0}return!1}});if(s.mount){let h=s.mount(m,r);if(typeof h==="function")o.push(h)}if(s.unmount)o.push(()=>s.unmount(m))}}return t}});function Ca(e=Date.now()){let n=e,r=[,,,,,,,,,,];for(let a=9;a>=0;a--)r[a]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[n%32],n=Math.floor(n/32);let o=Array(16),t=typeof globalThis<"u"?globalThis.crypto:void 0;if(t?.getRandomValues){let a=new Uint8Array(16);t.getRandomValues(a);for(let b=0;b<16;b++)o[b]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[a[b]%32]}else for(let a=0;a<16;a++)o[a]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[Math.floor(Math.random()*32)];return r.join("")+o.join("")}function _r(e){let n={};if(e.viewId)n.view_id=e.viewId;if(e.selector)n.selector=e.selector;if(e.rect)n.rect=e.rect;return{id:Ca(),kind:e.kind,target:n,note:e.note,created_at:new Date().toISOString(),author_kind:"human"}}var za="/api/feedback",_n=O({scope:(e,n)=>{let{viewId:r,mode:o,endpoint:t,staging:a}=e,b=P([]),c=P(null),f=P(null),p=P(""),d={},l=J(()=>{let A=f();if(!A)return!1;return Math.abs(A.curX-A.startX)>4||Math.abs(A.curY-A.startY)>4}),i=J(()=>{let A=f();if(!A)return"";let x=Math.min(A.startX,A.curX),C=Math.min(A.startY,A.curY),z=Math.abs(A.curX-A.startX),M=Math.abs(A.curY-A.startY);return`left:${x}px;top:${C}px;width:${z}px;height:${M}px`}),u=J(()=>{let A=c();if(!A)return"";let x=Math.min(A.x+12,1e4),C=Math.min(A.y+12,1e4);return`left:${x}px;top:${C}px`}),s=J(()=>b().map((A,x)=>({id:A.id,kind:A.kind,index:x,rect:A.kind==="region"&&A.target.rect?A.target.rect.join(","):"",pos:d[A.id]??{x:0,y:0}})));async function m(A){let x=t()||za;try{let C=await fetch(x,{method:"POST",credentials:"include",headers:{"Content-Type":"application/json"},body:JSON.stringify(A)});if(!C.ok){n.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:`http_${C.status}`,id:A.id},bubbles:!0,composed:!0}));return}b.set(b().filter((z)=>z.id!==A.id)),n.dispatchEvent(new CustomEvent("sui-annotation-sent",{detail:{id:A.id,status:C.status},bubbles:!0,composed:!0}))}catch(C){n.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:"network_error",message:String(C),id:A.id},bubbles:!0,composed:!0}))}}function h(A){let x=c();if(!x)return;let C=A.trim();if(!C){c.set(null),p.set("");return}let z=_r({viewId:r()||void 0,kind:x.kind,selector:x.kind==="point"?x.selector:void 0,rect:x.kind==="region"?x.rect:void 0,note:C});if(d[z.id]={x:x.x,y:x.y},b.set([...b(),z]),n.dispatchEvent(new CustomEvent("sui-annotation-staged",{detail:z,bubbles:!0,composed:!0})),c.set(null),p.set(""),!a())m(z)}let v=()=>{h(p())},y=()=>{c.set(null),p.set("")},w=(A)=>{if(A.key==="Escape")y();else if(A.key==="Enter"&&(A.metaKey||A.ctrlKey)){let x=A.currentTarget;h(x.value)}},k=Y(null);return{staged:b,pending:c,drag:f,noteText:p,dragVisible:l,dragRectStyle:i,popoverStyle:u,markers:s,overlayEl:k,confirmNote:h,sendNote:v,cancelPending:y,onNoteKeydown:w,sendOne:m}},mount(e,n){let r=e,o=null,t=null,a=(d)=>{let l=r.overlayEl();if(!l)return{x:0,y:0};let i=l.getBoundingClientRect();return{x:d.clientX-i.left,y:d.clientY-i.top}},b=(d)=>{if(r.mode()==="off")return;let i=d.target;if(i.closest("[data-annotation-popover]")||i.closest("sui-annotation-marker"))return;if(r.pending()){r.pending.set(null),r.noteText.set("");return}d.preventDefault();let u=a(d);o={startX:u.x,startY:u.y,curX:u.x,curY:u.y},r.drag.set(o),d.target.setPointerCapture?.(d.pointerId)},c=(d)=>{if(!o)return;let l=a(d);if(o={...o,curX:l.x,curY:l.y},t!==null)return;t=requestAnimationFrame(()=>{if(t=null,o)r.drag.set({...o})})},f=(d)=>{if(!o)return;let l=a(d),i=Math.abs(l.x-o.startX),u=Math.abs(l.y-o.startY),s=r.mode(),m=null;if(s==="point"||s==="region"&&i<8&&u<8)m={kind:"point",x:l.x,y:l.y,selector:"body"};else if(s==="region"){let h=Math.min(o.startX,l.x),v=Math.min(o.startY,l.y),y=[Math.round(h),Math.round(v),Math.round(i),Math.round(u)];m={kind:"region",x:h+i/2,y:v+u/2,rect:y}}if(o=null,r.drag.set(null),r.noteText.set(""),m)r.pending.set(m)},p=r.overlayEl();if(p)p.addEventListener("pointerdown",b),p.addEventListener("pointermove",c),p.addEventListener("pointerup",f);return Object.defineProperty(n,"staged",{configurable:!0,get:()=>r.staged()}),n.send=async()=>{for(let d of[...r.staged()])await r.sendOne(d)},n.clear=()=>{r.staged.set([])},()=>{if(t!==null)cancelAnimationFrame(t),t=null;if(p)p.removeEventListener("pointerdown",b),p.removeEventListener("pointermove",c),p.removeEventListener("pointerup",f)}}});var Pb=j("sui-annotation-layer",{props:{"view-id":g.string({default:""}),mode:g.union(["off","point","region"],{default:"off",reflect:!0}),endpoint:g.string({default:"/api/feedback"}),staging:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    position: relative;
    display: block;
    font-family: var(--font-body);
  }

  .surface {
    position: relative;
    width: 100%;
    height: 100%;
  }

  .overlay {
    position: absolute;
    inset: 0;
    z-index: 10;
    pointer-events: none;
  }
  :host([mode='point']) .overlay,
  :host([mode='region']) .overlay {
    pointer-events: auto;
    cursor: crosshair;
  }

  /* Drag selection rectangle */
  .drag-rect {
    position: absolute;
    border: 2px dashed var(--primary);
    background: color-mix(in oklch, var(--primary) 8%, transparent);
    pointer-events: none;
    border-radius: var(--radius-sm);
  }

  /* Inline note popover */
  .popover {
    position: absolute;
    z-index: 20;
    min-width: 220px;
    padding: 8px;
    background: var(--popover, var(--background));
    color: var(--popover-foreground, var(--foreground));
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    display: flex;
    flex-direction: column;
    gap: 6px;
    pointer-events: auto;
  }
  .popover textarea {
    font: inherit;
    width: 100%;
    min-height: 60px;
    resize: vertical;
    padding: 4px 6px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--background);
    color: var(--foreground);
    box-sizing: border-box;
  }
  .popover-actions { display: flex; justify-content: flex-end; gap: 6px; }
  .popover button {
    font: inherit;
    font-size: var(--text-sm);
    padding: 4px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--background);
    color: var(--foreground);
    cursor: pointer;
  }
  .popover button[data-primary] {
    background: var(--primary);
    color: var(--primary-foreground);
    border-color: var(--primary);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.viewId=n.viewId,t.mode=n.mode,t.endpoint=n.endpoint,t.staging=n.staging;let a=null;if(_n&&typeof _n.setup==="function"){if(e.scope=t,a=_n.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(h)=>H({anchor:h,test:()=>{if(t.dragVisible())return 0;return-1},branches:[()=>{let v=document.createDocumentFragment(),y=[],w=document.createElement("div");return w.setAttribute("class","drag-rect"),y.push(E(w,"style",()=>t.dragRectStyle())),v.appendChild(w),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let k of y)try{k()}catch(A){}}}}]}),c=(h)=>X({anchor:h,items:()=>t.markers(),key:(v,y)=>{let w=Object.create(t);return w.m=v,w.__i=y,((k)=>v.id)(w)},render:(v,y)=>{let w=Object.create(t);return w.m=v,w.__i=y,w.__v=P(0),((k)=>{let A=document.createDocumentFragment(),x=[],C=document.createElement("sui-annotation-marker");return x.push($(C,"kind",()=>(k.__v(),k.m.kind))),x.push($(C,"index",()=>(k.__v(),k.m.index))),x.push($(C,"rect",()=>(k.__v(),k.m.rect))),x.push($(C,"x",()=>(k.__v(),k.m.pos.x))),x.push($(C,"y",()=>(k.__v(),k.m.pos.y))),A.appendChild(C),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let z of x)try{z()}catch(M){}},update:(z,M)=>{k.m=z,k.__i=M,k.__v.set((N)=>N+1)}}})(w)}}),f=(h)=>H({anchor:h,test:()=>{if(t.pending())return 0;return-1},branches:[()=>{let v=document.createDocumentFragment(),y=[],w=document.createElement("div");w.setAttribute("class","popover"),w.setAttribute("data-annotation-popover",""),y.push(E(w,"style",()=>t.popoverStyle())),y.push(V(w,"pointerdown",function(){let N=(S)=>S.stopPropagation();return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]}));let k=document.createElement("textarea");k.setAttribute("part","note-input"),k.setAttribute("placeholder","Add a note…"),y.push(pt(k,t.noteText)),y.push(V(k,"keydown",function(){let N=t.onNoteKeydown;return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]})),w.appendChild(k);let A=document.createElement("div");A.setAttribute("class","popover-actions");let x=document.createElement("button");x.setAttribute("type","button"),y.push(V(x,"click",function(){let N=t.cancelPending;return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]}));let C=document.createTextNode("Cancel");x.appendChild(C),A.appendChild(x);let z=document.createElement("button");z.setAttribute("type","button"),z.setAttribute("data-primary",""),y.push(V(z,"click",function(){let N=t.sendNote;return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]}));let M=document.createTextNode("Send");return z.appendChild(M),A.appendChild(z),w.appendChild(A),v.appendChild(w),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let N of y)try{N()}catch(S){}}}}]}),p=r.shadowRoot??r;L(p);let d=document.createElement("div");d.setAttribute("class","surface"),d.setAttribute("part","surface");let l=document.createElement("slot");d.appendChild(l);let i=document.createElement("div");i.setAttribute("class","overlay"),i.setAttribute("part","overlay"),T(t,"overlayEl",i);let u=document.createComment("if");i.appendChild(u),o.push(b(u));let s=document.createComment("for");i.appendChild(s),o.push(c(s));let m=document.createComment("if");if(i.appendChild(m),o.push(f(m)),d.appendChild(i),p.appendChild(d),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(y,w){return y[w]},set(y,w,k){let A=y[w];if(A!=null&&typeof A.set==="function"){if(typeof k==="function")A.set(()=>k);else A.set(k);return!0}return!1}});if(h.mount){let y=h.mount(v,r);if(typeof y==="function")o.push(y)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var te=O({mount(e,n){let r=e;return D(()=>{let t=r.pinEl(),a=r.boxEl(),b=r.x(),c=r.y(),f=r.index(),p=r.rect();if(t)t.style.left=`${b}px`,t.style.top=`${c}px`,t.textContent=String(f+1);if(a&&p){let d=p.split(",").map(Number);if(d.length===4&&d.every((l)=>Number.isFinite(l))){let[l,i,u,s]=d;a.style.cssText=`left:${l}px;top:${i}px;width:${u}px;height:${s}px`}}})}});var Gb=j("sui-annotation-marker",{props:{kind:g.union(["point","region"],{default:"point",reflect:!0}),x:g.number({default:0}),y:g.number({default:0}),rect:g.string({default:""}),active:g.bool({default:!1,reflect:!0}),index:g.number({default:0})},shadow:!0,styles:`
  :host {
    position: absolute;
    pointer-events: auto;
    font-family: var(--font-body);
    font-size: var(--text-xs);
  }

  [part='pin'] {
    position: absolute;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--primary);
    color: var(--primary-foreground);
    display: none;
    align-items: center;
    justify-content: center;
    font-weight: var(--font-weight-medium);
    box-shadow: var(--shadow-sm);
    cursor: pointer;
    transform: translate(-50%, -50%);
    border: 2px solid var(--background);
    transition: filter var(--duration-fast) var(--ease-out);
  }
  :host([kind='point']) [part='pin'] { display: inline-flex; }
  :host([active]) [part='pin'] {
    filter: brightness(1.2);
    outline: 2px solid var(--ring-color);
    outline-offset: 2px;
  }

  [part='box'] {
    position: absolute;
    border: 2px dashed var(--primary);
    background: var(--tone-primary-bg);
    border-radius: var(--radius-sm);
    cursor: pointer;
    box-sizing: border-box;
    display: none;
  }
  :host([kind='region']) [part='box'] { display: block; }
  :host([active]) [part='box'] {
    background: color-mix(in oklch, var(--primary) 14%, transparent);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.kind=n.kind,t.x=n.x,t.y=n.y,t.rect=n.rect,t.active=n.active,t.index=n.index;let a=null;if(te&&typeof te.setup==="function"){if(e.scope=t,a=te.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("part","pin"),c.setAttribute("role","button"),c.setAttribute("tabindex","0"),T(t,"pinEl",c),b.appendChild(c);let f=document.createElement("div");if(f.setAttribute("part","box"),f.setAttribute("role","button"),f.setAttribute("tabindex","0"),T(t,"boxEl",f),b.appendChild(f),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var ne=O({scope:(e,n)=>{return{staged:P([]),emitRemove:(t)=>{n.dispatchEvent(new CustomEvent("sui-annotation-remove",{detail:{id:t},bubbles:!0,composed:!0}))}}},mount(e,n){let o=e.staged,t=n.staged;if(t!=null)o.set(Array.isArray(t)?t:[]);Object.defineProperty(n,"staged",{configurable:!0,get:()=>o(),set:(a)=>o.set(Array.isArray(a)?a:[])})}});var Yb=j("sui-annotation-list",{shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
  }

  ol {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--background);
  }
  .kind {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--muted-foreground);
    min-width: 48px;
  }
  .note {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty {
    color: var(--muted-foreground);
    padding: 4px 8px;
  }
  button {
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--muted-foreground);
    cursor: pointer;
    font-size: var(--text-xs);
    padding: 2px 6px;
  }
  button:hover {
    color: var(--destructive);
    border-color: var(--destructive);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(ne&&typeof ne.setup==="function"){if(e.scope=t,a=ne.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>X({anchor:d,items:()=>t.staged(),key:(l,i)=>{let u=Object.create(t);return u.a=l,u.__i=i,((s)=>l.id)(u)},render:(l,i)=>{let u=Object.create(t);return u.a=l,u.__i=i,u.__v=P(0),((s)=>{let m=document.createDocumentFragment(),h=[],v=document.createElement("li");v.setAttribute("part","item"),h.push(E(v,"data-id",()=>(s.__v(),s.a.id)));let y=document.createElement("span");y.setAttribute("class","kind");let w=document.createTextNode("");y.appendChild(w),h.push(F(w,()=>(s.__v(),s.a.kind))),v.appendChild(y);let k=document.createElement("span");k.setAttribute("class","note"),h.push(E(k,"title",()=>(s.__v(),s.a.note??"")));let A=document.createTextNode("");k.appendChild(A),h.push(F(A,()=>(s.__v(),s.a.note??"(no note)"))),v.appendChild(k);let x=document.createElement("button");x.setAttribute("type","button"),x.setAttribute("aria-label","Remove annotation"),h.push(V(x,"click",function(){let z=()=>s.emitRemove(s.a.id);return typeof z==="function"?z.bind(void 0):()=>{}}(),{modifiers:[]}));let C=document.createTextNode("Remove");return x.appendChild(C),v.appendChild(x),m.appendChild(v),{nodes:Array.from(m.childNodes),cleanup:()=>{for(let z of h)try{z()}catch(M){}},update:(z,M)=>{s.a=z,s.__i=M,s.__v.set((N)=>N+1)}}})(u)}}),c=(d)=>H({anchor:d,test:()=>{if(t.staged().length===0)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("div");u.setAttribute("class","empty"),u.setAttribute("part","empty");let s=document.createTextNode("No staged annotations.");return u.appendChild(s),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let m of i)try{m()}catch(h){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("ol");u.setAttribute("part","list");let s=document.createComment("for");return u.appendChild(s),i.push(b(s)),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let m of i)try{m()}catch(h){}}}}]}),f=r.shadowRoot??r;L(f);let p=document.createComment("if");if(f.appendChild(p),o.push(c(p)),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var ee=O({mount(e,n){let r=e,o=D(()=>{let t=r.inputEl();if(!t)return;let a=()=>{if(r.disabled())return;n.dispatchEvent(new CustomEvent("sui-radio-select",{detail:{value:r.value()},bubbles:!0,composed:!0}))},b=(c)=>{if(!["ArrowDown","ArrowRight","ArrowUp","ArrowLeft"].includes(c.key))return;c.preventDefault();let p=c.key==="ArrowDown"||c.key==="ArrowRight";n.dispatchEvent(new CustomEvent("sui-radio-nav",{detail:{direction:p?"next":"prev"},bubbles:!0,composed:!0}))};return t.addEventListener("click",a),t.addEventListener("keydown",b),()=>{t.removeEventListener("click",a),t.removeEventListener("keydown",b)}});return n.focusInput=()=>{n.shadowRoot?.querySelector("input")?.focus()},()=>o()}});var Ib=j("sui-radio",{props:{value:g.string({default:"",reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),checked:g.bool({default:!1,reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    vertical-align: middle;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);
    -webkit-tap-highlight-color: transparent;
    --tone: var(--primary);
  }

  :host(:not([tone])),
  :host([tone='default'])     { --tone: var(--primary); }
  :host([tone='primary'])     { --tone: var(--primary); }
  :host([tone='info'])        { --tone: var(--info); }
  :host([tone='success'])     { --tone: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) { --rd-size: 16px; --rd-dot: 8px; }
  :host([size='sm']) { --rd-size: 14px; --rd-dot: 6px; }
  :host([size='lg']) { --rd-size: 20px; --rd-dot: 10px; }

  label {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2, 8px);
    cursor: pointer;
    user-select: none;
    -webkit-user-select: none;
  }
  :host([disabled]) label { cursor: not-allowed; }

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
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }

  :host(:not([disabled]):not([checked])) label:hover .circle {
    border-color: color-mix(in oklch, var(--border) 220%, transparent);
  }
  :host([checked]) .circle { border-color: var(--tone); }

  .dot {
    width: var(--rd-dot);
    height: var(--rd-dot);
    border-radius: 50%;
    background: var(--tone);
    opacity: 0;
    transform: scale(0);
    transition:
      opacity var(--duration-fast, 150ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 150ms) var(--ease-out, ease-out);
    pointer-events: none;
  }
  :host([checked]) .dot { opacity: 1; transform: scale(1); }

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

  :host(:not([disabled])) label:has(input:focus-visible) .circle {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }

  @media (forced-colors: active) {
    .circle { border-color: CanvasText; }
    :host([checked]) .dot { background: Highlight; }
    :host(:not([disabled])) label:has(input:focus-visible) .circle {
      outline: 2px solid CanvasText;
      outline-offset: 2px;
    }
  }

  :host([disabled]) { color: var(--muted-foreground); }
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
    line-height: var(--leading-tight, 1.25);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.disabled=n.disabled,t.checked=n.checked,t.tone=n.tone,t.size=n.size;let a=null;if(ee&&typeof ee.setup==="function"){if(e.scope=t,a=ee.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("label");c.setAttribute("part","label");let f=document.createElement("span");f.setAttribute("class","circle"),f.setAttribute("part","circle");let p=document.createElement("input");p.setAttribute("part","input"),p.setAttribute("type","radio"),o.push(ft(p,t.checked)),o.push(E(p,"disabled",()=>t.disabled())),o.push(E(p,"aria-checked",()=>t.checked()?"true":"false")),o.push(E(p,"tabindex",()=>t.checked()?"0":"-1")),T(t,"inputEl",p),f.appendChild(p);let d=document.createElement("span");d.setAttribute("class","dot"),d.setAttribute("part","dot"),d.setAttribute("aria-hidden","true"),f.appendChild(d),c.appendChild(f);let l=document.createElement("slot");if(c.appendChild(l),b.appendChild(c),a!=null){let i=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(i){let u=new Proxy(t,{get(s,m){return s[m]},set(s,m,h){let v=s[m];if(v!=null&&typeof v.set==="function"){if(typeof h==="function")v.set(()=>h);else v.set(h);return!0}return!1}});if(i.mount){let s=i.mount(u,r);if(typeof s==="function")o.push(s)}if(i.unmount)o.push(()=>i.unmount(u))}}return t}});var re=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","radiogroup");let o=document.createElement("input");o.type="hidden",n.appendChild(o);let t=()=>Array.from(n.querySelectorAll(":scope > sui-radio")),a=()=>t().filter((l)=>!l.hasAttribute("disabled")&&!r.disabled()),b=()=>{let l=r.value(),i=r.disabled(),u=r.tone(),s=r.size();for(let m of t()){if(m.toggleAttribute("checked",m.getAttribute("value")===l),i)m.setAttribute("disabled","");if(u&&!m.hasAttribute("tone"))m.setAttribute("tone",u);if(s&&!m.hasAttribute("size"))m.setAttribute("size",s)}o.name=r.name(),o.value=l},c=D(()=>{r.value(),r.disabled(),r.tone(),r.size(),b()}),f=D(()=>{let l=r.label();if(l)n.setAttribute("aria-label",l);else n.removeAttribute("aria-label")}),p=(l)=>{let i=l;i.stopPropagation();let u=i.detail.value;if(u===r.value())return;r.value.set(u),b(),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:u},bubbles:!0,composed:!0}))},d=(l)=>{let i=l;i.stopPropagation();let u=a();if(!u.length)return;let s=i.composedPath().find((y)=>y.tagName?.toLowerCase()==="sui-radio"),m=s?u.indexOf(s):-1,h=i.detail.direction==="next"?u[(m+1)%u.length]:u[(m-1+u.length)%u.length],v=h.getAttribute("value")??"";r.value.set(v),b(),h.focusInput?.(),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:v},bubbles:!0,composed:!0}))};return n.addEventListener("sui-radio-select",p),n.addEventListener("sui-radio-nav",d),()=>{c(),f(),n.removeEventListener("sui-radio-select",p),n.removeEventListener("sui-radio-nav",d),o.remove()}}});var rh=j("sui-radio-group",{props:{value:g.string({default:"",reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),name:g.string({default:""}),label:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    gap: var(--space-2, 8px);
    font-family: var(--font-body);
    color: var(--foreground);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.disabled=n.disabled,t.tone=n.tone,t.size=n.size,t.name=n.name,t.label=n.label;let a=null;if(re&&typeof re.setup==="function"){if(e.scope=t,a=re.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});var oe=O({mount(e,n){let r=e,o=document.createElement("input");o.type="hidden",n.appendChild(o);let t=(u)=>{let s=r.step(),m=r.min(),h=r.max(),v=Math.round((u-m)/s)*s+m;return Math.max(m,Math.min(h,v))},a=(u)=>{let s=t(u);if(s===r.value())return;r.value.set(s),o.name=r.name(),o.value=String(s),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:s},bubbles:!0,composed:!0}))},b=(u)=>{let s=r.trackEl();if(!s)return r.value();let m=s.getBoundingClientRect(),h=Math.max(0,Math.min(1,(u-m.left)/m.width));return r.min()+h*(r.max()-r.min())},c=D(()=>{let u=r.value(),s=r.min(),h=r.max()-s||1,v=(u-s)/h*100,y=r.fillEl(),w=r.thumbEl();if(y)y.style.width=`${v}%`;if(w)w.style.left=`${v}%`;o.name=r.name(),o.value=String(u)}),f=!1,p=(u)=>{if(r.disabled())return;u.preventDefault(),f=!0,a(b(u.clientX)),r.thumbEl()?.focus();let m=(v)=>{if(f)a(b(v.clientX))},h=()=>{f=!1,document.removeEventListener("pointermove",m),document.removeEventListener("pointerup",h)};document.addEventListener("pointermove",m),document.addEventListener("pointerup",h)},d=(u)=>{if(r.disabled())return;let s=r.step(),m=!0;switch(u.key){case"ArrowRight":case"ArrowUp":a(r.value()+s);break;case"ArrowLeft":case"ArrowDown":a(r.value()-s);break;case"Home":a(r.min());break;case"End":a(r.max());break;case"PageUp":a(r.value()+s*10);break;case"PageDown":a(r.value()-s*10);break;default:m=!1}if(m)u.preventDefault()},l=D(()=>{let u=r.containerEl();if(!u)return;return u.addEventListener("pointerdown",p),()=>u.removeEventListener("pointerdown",p)}),i=D(()=>{let u=r.thumbEl();if(!u)return;return u.addEventListener("keydown",d),()=>u.removeEventListener("keydown",d)});return()=>{c(),l(),i(),o.remove()}}});var lh=j("sui-slider",{props:{value:g.number({default:0,reflect:!0}),min:g.number({default:0}),max:g.number({default:100}),step:g.number({default:1}),disabled:g.bool({default:!1,reflect:!0}),tone:g.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),name:g.string({default:""}),label:g.string({default:"slider"})},shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    -webkit-tap-highlight-color: transparent;
    --tone: var(--primary);
  }

  :host(:not([tone])),
  :host([tone='default'])     { --tone: var(--primary); }
  :host([tone='primary'])     { --tone: var(--primary); }
  :host([tone='info'])        { --tone: var(--info); }
  :host([tone='success'])     { --tone: var(--success); }
  :host([tone='warning'])     { --tone: var(--warning); }
  :host([tone='destructive']) { --tone: var(--destructive); }
  :host([tone='accent'])      { --tone: var(--accent); }
  :host([tone='muted'])       { --tone: var(--muted-foreground); }

  :host(:not([size])),
  :host([size='md']) { --sl-h: 6px; --sl-thumb: 16px; }
  :host([size='sm']) { --sl-h: 4px; --sl-thumb: 12px; }
  :host([size='lg']) { --sl-h: 8px; --sl-thumb: 20px; }

  .container {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
    height: var(--sl-thumb);
    cursor: pointer;
    touch-action: none;
  }
  :host([disabled]) .container { cursor: not-allowed; opacity: 0.5; }

  .track {
    position: relative;
    width: 100%;
    height: var(--sl-h);
    background: var(--muted);
    border-radius: var(--radius-full, 9999px);
    overflow: hidden;
  }

  .fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: var(--tone);
    border-radius: inherit;
  }

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
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out),
      transform var(--duration-fast, 150ms) var(--ease-out, ease-out);
    box-sizing: border-box;
    outline: none;
  }
  .thumb:hover { box-shadow: var(--shadow-md); transform: translate(-50%, -50%) scale(1.05); }
  .thumb:active { transform: translate(-50%, -50%) scale(0.95); }

  :host(:not([disabled])) .thumb:focus-visible {
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--tone) var(--ring-opacity, 50%), transparent);
  }

  @media (forced-colors: active) {
    .thumb { border-color: CanvasText; }
    .fill { background: Highlight; }
    .thumb:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.min=n.min,t.max=n.max,t.step=n.step,t.disabled=n.disabled,t.tone=n.tone,t.size=n.size,t.name=n.name,t.label=n.label;let a=null;if(oe&&typeof oe.setup==="function"){if(e.scope=t,a=oe.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","container"),T(t,"containerEl",c);let f=document.createElement("div");f.setAttribute("class","track"),f.setAttribute("part","track"),T(t,"trackEl",f);let p=document.createElement("div");p.setAttribute("class","fill"),p.setAttribute("part","fill"),T(t,"fillEl",p),f.appendChild(p),c.appendChild(f);let d=document.createElement("div");if(d.setAttribute("class","thumb"),d.setAttribute("part","thumb"),d.setAttribute("role","slider"),o.push(E(d,"tabindex",()=>t.disabled()?"-1":"0")),o.push(E(d,"aria-valuemin",()=>t.min())),o.push(E(d,"aria-valuemax",()=>t.max())),o.push(E(d,"aria-valuenow",()=>t.value())),o.push(E(d,"aria-label",()=>t.label()||"slider")),o.push(E(d,"aria-disabled",()=>t.disabled()?"true":"false")),T(t,"thumbEl",d),c.appendChild(d),b.appendChild(c),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(u,s){return u[s]},set(u,s,m){let h=u[s];if(h!=null&&typeof h.set==="function"){if(typeof m==="function")h.set(()=>m);else h.set(m);return!0}return!1}});if(l.mount){let u=l.mount(i,r);if(typeof u==="function")o.push(u)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var ph=j("sui-segmented-option",{props:{value:g.string({default:"",reflect:!0}),icon:g.string({default:"",reflect:!0}),disabled:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.icon=n.icon,t.disabled=n.disabled;let a=r.shadowRoot??r;L(a);let b=document.createTextNode(`

`);a.appendChild(b);let c=document.createTextNode(`
`);a.appendChild(c);let f=document.createTextNode(`
`);a.appendChild(f);let p=document.createTextNode(`

`);a.appendChild(p);let d=document.createTextNode(`
`);return a.appendChild(d),t}});var ja=(e)=>typeof CSS<"u"&&CSS.escape?CSS.escape(e):e.replace(/[^\w-]/g,"\\$&"),ae=O({scope:()=>{let e=P([]),n=Y(null);return{options:e,containerEl:n}},mount(e,n){let r=e,o=r.options;if(!n.hasAttribute("role"))n.setAttribute("role","radiogroup");let t=document.createElement("input");t.type="hidden",n.appendChild(t);let a=()=>{return Array.from(n.querySelectorAll(":scope > sui-segmented-option")).map((u)=>({value:u.getAttribute("value")??"",label:u.textContent?.trim()??"",icon:u.getAttribute("icon")||null,disabled:u.hasAttribute("disabled")}))},b=(i)=>{if(r.disabled())return;let s=o().find((m)=>m.value===i);if(!s||s.disabled)return;if(r.value()===i)return;r.value.set(i),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:i},bubbles:!0,composed:!0}))},c=(i)=>{let u=i.target?.closest(".btn");if(!u)return;let s=u.getAttribute("data-value");if(s!==null)b(s)},f=(i)=>{let s=o().filter((k)=>!k.disabled&&!r.disabled());if(!s.length)return;let m=s.map((k)=>k.value),h=r.value(),v=Math.max(0,m.indexOf(h)),y=v;if(i.key==="ArrowRight"||i.key==="ArrowDown")i.preventDefault(),y=(v+1)%m.length;else if(i.key==="ArrowLeft"||i.key==="ArrowUp")i.preventDefault(),y=(v-1+m.length)%m.length;else if(i.key==="Enter"||i.key===" "){i.preventDefault();let A=n.shadowRoot.activeElement?.getAttribute("data-value");if(A)b(A);return}else return;let w=m[y]??"";b(w),queueMicrotask(()=>{n.shadowRoot?.querySelector(`.btn[data-value="${ja(w)}"]`)?.focus()})};o.set(a());let p=new MutationObserver(()=>o.set(a()));p.observe(n,{childList:!0,subtree:!1});let d=D(()=>{t.name=r.name(),t.value=r.value()}),l=D(()=>{let i=r.containerEl();if(!i)return;return i.addEventListener("click",c),i.addEventListener("keydown",f),()=>{i.removeEventListener("click",c),i.removeEventListener("keydown",f)}});return()=>{p.disconnect(),d(),l(),t.remove()}}});var gh=j("sui-segmented",{props:{value:g.string({default:"",reflect:!0}),size:g.union(["sm","md"],{default:"md",reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),name:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline-flex;
  }

  .container {
    display: inline-flex;
    padding: 2px;
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    gap: 1px;
  }

  .btn {
    appearance: none;
    border: 0;
    background: transparent;
    border-radius: var(--radius-sm, 4px);
    font-family: var(--font-body);
    color: var(--muted-foreground);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: var(--space-1, 4px);
    transition:
      background var(--duration-fast, 150ms) var(--ease-out, ease-out),
      color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      box-shadow var(--duration-fast, 150ms) var(--ease-out, ease-out);
    white-space: nowrap;
    line-height: 1;
  }

  :host(:not([size])) .btn,
  :host([size='md']) .btn {
    padding: 5px var(--space-3, 12px);
    font-size: var(--text-sm, 13px);
  }
  :host([size='sm']) .btn {
    padding: 4px var(--space-2, 8px);
    font-size: var(--text-xs, 11px);
  }

  .btn[aria-checked='true'] {
    background: var(--card);
    color: var(--foreground);
    box-shadow: var(--shadow-xs);
  }

  .btn:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 2px var(--background),
      0 0 0 4px color-mix(in oklch, var(--primary) 60%, transparent);
  }

  .btn[disabled],
  :host([disabled]) .btn {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
  }

  .btn-icon {
    display: inline-flex;
    align-items: center;
    width: 14px;
    height: 14px;
  }
  :host(:not([size='sm'])) .btn-icon { width: 16px; height: 16px; }

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.size=n.size,t.disabled=n.disabled,t.name=n.name;let a=null;if(ae&&typeof ae.setup==="function"){if(e.scope=t,a=ae.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>X({anchor:d,items:()=>t.options(),key:(l,i)=>{let u=Object.create(t);return u.opt=l,u.__i=i,((s)=>l.value)(u)},render:(l,i)=>{let u=Object.create(t);return u.opt=l,u.__i=i,u.__v=P(0),((s)=>{let m=(A)=>H({anchor:A,test:()=>{if(s.opt.icon)return 0;return-1},branches:[()=>{let x=document.createDocumentFragment(),C=[],z=document.createElement("span");z.setAttribute("class","btn-icon"),z.setAttribute("aria-hidden","true");let M=document.createElement("sui-icon");return C.push($(M,"name",()=>(s.__v(),s.opt.icon))),M.setAttribute("size","sm"),z.appendChild(M),x.appendChild(z),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let N of C)try{N()}catch(S){}}}}]}),h=document.createDocumentFragment(),v=[],y=document.createElement("button");y.setAttribute("class","btn"),y.setAttribute("role","radio"),y.setAttribute("type","button"),v.push(E(y,"data-value",()=>(s.__v(),s.opt.value))),v.push(E(y,"aria-checked",()=>(s.__v(),s.opt.value===s.value()?"true":"false"))),v.push(E(y,"aria-disabled",()=>(s.__v(),s.disabled()||s.opt.disabled?"true":"false"))),v.push($(y,"disabled",()=>(s.__v(),s.disabled()||s.opt.disabled))),v.push(E(y,"tabindex",()=>(s.__v(),s.opt.value===s.value()?"0":"-1")));let w=document.createComment("if");y.appendChild(w),v.push(m(w));let k=document.createTextNode("");return y.appendChild(k),v.push(F(k,()=>(s.__v(),s.opt.label))),h.appendChild(y),{nodes:Array.from(h.childNodes),cleanup:()=>{for(let A of v)try{A()}catch(x){}},update:(A,x)=>{s.opt=A,s.__i=x,s.__v.set((C)=>C+1)}}})(u)}}),c=r.shadowRoot??r;L(c);let f=document.createElement("div");f.setAttribute("class","container"),f.setAttribute("part","container"),T(t,"containerEl",f);let p=document.createComment("for");if(f.appendChild(p),o.push(b(p)),c.appendChild(f),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var ie=O({mount(e,n){let r=e,o=D(()=>{let t=r.buttonEl();if(!t)return;let a=()=>{if(r.disabled())return;r.pressed.set(!r.pressed()),n.dispatchEvent(new CustomEvent("sui-toggle",{detail:{pressed:r.pressed(),value:r.value()},bubbles:!0,composed:!0}))};return t.addEventListener("click",a),()=>t.removeEventListener("click",a)});return n.focusButton=()=>{n.shadowRoot?.querySelector("button")?.focus()},()=>o()}});var Eh=j("sui-toggle",{props:{pressed:g.bool({default:!1,reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),value:g.string({default:"",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:g.union(["outline","ghost"],{default:"outline",reflect:!0})},shadow:!0,styles:`
  :host { display: inline-flex; }

  :host(:not([size])), :host([size='md']) {
    --tgl-h: 36px; --tgl-px: 12px; --tgl-fs: var(--text-base, 14px);
  }
  :host([size='sm']) {
    --tgl-h: 28px; --tgl-px: 8px; --tgl-fs: var(--text-sm, 13px);
  }
  :host([size='lg']) {
    --tgl-h: 42px; --tgl-px: 16px; --tgl-fs: var(--text-base, 14px);
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
    font-weight: var(--font-weight-medium, 500);
    line-height: 1;
    color: var(--muted-foreground);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: var(--radius, 6px);
    cursor: pointer;
    transition:
      background-color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      color var(--duration-fast, 150ms) var(--ease-out, ease-out),
      border-color var(--duration-fast, 150ms) var(--ease-out, ease-out);
  }
  :host([variant='ghost']) button { border-color: transparent; }
  button:hover { background: var(--muted); color: var(--foreground); }
  :host([pressed]) button {
    background: var(--muted);
    color: var(--foreground);
    border-color: var(--border);
  }
  :host([disabled]) button { cursor: not-allowed; opacity: 0.5; pointer-events: none; }

  button:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--primary) 50%, transparent);
  }

  @media (forced-colors: active) {
    button:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
    :host([pressed]) button { outline: 2px solid CanvasText; }
  }

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.pressed=n.pressed,t.disabled=n.disabled,t.value=n.value,t.size=n.size,t.variant=n.variant;let a=null;if(ie&&typeof ie.setup==="function"){if(e.scope=t,a=ie.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("button");c.setAttribute("part","button"),c.setAttribute("type","button"),o.push(E(c,"aria-pressed",()=>t.pressed()?"true":"false")),o.push(E(c,"disabled",()=>t.disabled())),T(t,"buttonEl",c);let f=document.createElement("slot");if(c.appendChild(f),b.appendChild(c),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var se=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","group");let o=document.createElement("input");o.type="hidden",n.appendChild(o);let t=()=>{let s=r.defaultSlotEl();if(!s)return Array.from(n.querySelectorAll(":scope > sui-toggle"));return s.assignedElements().filter((m)=>m.tagName.toLowerCase()==="sui-toggle")},a=()=>t().filter((s)=>!s.hasAttribute("disabled")),b=()=>new Set(r.value().split(",").filter(Boolean)),c=(s)=>{let m=Array.from(s).join(",");r.value.set(m),f(),o.name=r.name(),o.value=m,n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:m},bubbles:!0,composed:!0}))},f=()=>{let s=b(),m=r.size(),h=r.variant();for(let v of t()){let y=v.getAttribute("value")??"";if(v.toggleAttribute("pressed",s.has(y)),m)v.setAttribute("size",m);if(h)v.setAttribute("variant",h)}},p=(s)=>{let m=s;m.stopPropagation();let h=m.composedPath().find((w)=>w.tagName?.toLowerCase()==="sui-toggle");if(!h)return;let v=h.getAttribute("value")??"",y=b();if(r.type()==="single")if(y.has(v))c(new Set);else c(new Set([v]));else{if(y.has(v))y.delete(v);else y.add(v);c(y)}},d=(s)=>{if(s.key!=="ArrowRight"&&s.key!=="ArrowLeft")return;let m=a();if(!m.length)return;let h=s.composedPath().find((w)=>m.includes(w)),v=h?m.indexOf(h):-1,y=s.key==="ArrowRight"?(v+1)%m.length:(v-1+m.length)%m.length;m[y]?.focusButton?.(),s.preventDefault()},l=D(()=>{r.value(),r.size(),r.variant(),f()}),i=D(()=>{let s=r.defaultSlotEl();if(!s)return;let m=()=>f();return s.addEventListener("slotchange",m),()=>s.removeEventListener("slotchange",m)}),u=D(()=>{o.name=r.name(),o.value=r.value()});return n.addEventListener("sui-toggle",p),n.addEventListener("keydown",d),()=>{l(),i(),u(),n.removeEventListener("sui-toggle",p),n.removeEventListener("keydown",d),o.remove()}}});var Oh=j("sui-toggle-group",{props:{type:g.union(["single","multiple"],{default:"single",reflect:!0}),value:g.string({default:"",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:g.union(["outline","ghost"],{default:"outline",reflect:!0}),name:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    gap: 0;
  }

  ::slotted(sui-toggle:not(:first-child)) { margin-left: -1px; }
  ::slotted(sui-toggle:first-child) { border-radius: var(--radius, 6px) 0 0 var(--radius, 6px); }
  ::slotted(sui-toggle:last-child) { border-radius: 0 var(--radius, 6px) var(--radius, 6px) 0; }
  ::slotted(sui-toggle:not(:first-child):not(:last-child)) { border-radius: 0; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.type=n.type,t.value=n.value,t.size=n.size,t.variant=n.variant,t.name=n.name;let a=null;if(se&&typeof se.setup==="function"){if(e.scope=t,a=se.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(T(t,"defaultSlotEl",c),b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});var Ph=j("sui-code",{props:{lang:g.string({default:"",reflect:!0}),block:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline;
    font-family: var(--font-mono);
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.lang=n.lang,t.block=n.block;let a=(p)=>H({anchor:p,test:()=>{if(t.lang())return 0;return-1},branches:[()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("header");i.setAttribute("part","label");let u=document.createTextNode("");return i.appendChild(u),l.push(F(u,()=>t.lang())),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let s of l)try{s()}catch(m){}}}}]}),b=(p)=>H({anchor:p,test:()=>{if(t.block())return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("div");i.setAttribute("class","block"),i.setAttribute("part","block"),i.setAttribute("role","region"),l.push(E(i,"aria-label",()=>t.lang()?t.lang()+" code block":"code block"));let u=document.createComment("if");i.appendChild(u),l.push(a(u));let s=document.createElement("pre");s.setAttribute("part","pre");let m=document.createElement("slot");return s.appendChild(m),i.appendChild(s),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let h of l)try{h()}catch(v){}}}},()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("code");i.setAttribute("class","inline"),i.setAttribute("part","inline");let u=document.createElement("slot");return i.appendChild(u),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let s of l)try{s()}catch(m){}}}}]}),c=r.shadowRoot??r;L(c);let f=document.createComment("if");return c.appendChild(f),o.push(b(f)),t}});var ue=O({mount(e,n){let r=e,o,t=async()=>{try{await navigator.clipboard.writeText(r.value())}catch{let b=document.createElement("textarea");b.value=r.value(),document.body.appendChild(b),b.select();try{document.execCommand("copy")}catch{}b.remove()}if(r.copied.set(!0),n.dispatchEvent(new CustomEvent("sui-copy",{detail:{value:r.value()},bubbles:!0,composed:!0})),o)clearTimeout(o);o=setTimeout(()=>{r.copied.set(!1)},1200)},a=D(()=>{let b=r.buttonEl();if(!b)return;return b.addEventListener("click",t),()=>b.removeEventListener("click",t)});return()=>{if(a(),o)clearTimeout(o)}}});var Gh=j("sui-copy-button",{props:{value:g.string({default:"",reflect:!0}),label:g.string({default:"Copy",reflect:!0}),copied:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-flex;
  }

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
  button:hover {
    color: var(--foreground);
    background: var(--muted);
  }
  button:focus-visible {
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--ring-color, var(--primary)) var(--ring-opacity, 50%), transparent);
  }
  @media (forced-colors: active) {
    button:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
  :host([copied]) button {
    color: var(--success);
    border-color: var(--success);
  }

  /* Show default label when slot is empty (JSDOM always reports empty, so tests see this) */
  .default-label { display: inline; }
  :host(:not([data-empty])) .default-label { display: none; }

  sui-icon {
    flex: 0 0 auto;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.label=n.label,t.copied=n.copied;let a=null;if(ue&&typeof ue.setup==="function"){if(e.scope=t,a=ue.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>H({anchor:u,test:()=>{if(t.copied())return 0;return 1},branches:[()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("sui-icon");return h.setAttribute("name","check"),h.setAttribute("size","sm"),h.setAttribute("aria-hidden","true"),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let v of m)try{v()}catch(y){}}}},()=>{let s=document.createDocumentFragment(),m=[],h=document.createElement("sui-icon");return h.setAttribute("name","copy"),h.setAttribute("size","sm"),h.setAttribute("aria-hidden","true"),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let v of m)try{v()}catch(y){}}}}]}),c=r.shadowRoot??r;L(c);let f=document.createElement("button");f.setAttribute("part","button"),f.setAttribute("type","button"),o.push(E(f,"aria-label",()=>t.label())),T(t,"buttonEl",f);let p=document.createComment("if");f.appendChild(p),o.push(b(p));let d=document.createElement("slot");o.push(tt(d,r,"data-empty")),f.appendChild(d);let l=document.createElement("span");l.setAttribute("class","default-label");let i=document.createTextNode("");if(l.appendChild(i),o.push(F(i,()=>t.copied()?"Copied":t.label())),f.appendChild(l),c.appendChild(f),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(m,h){return m[h]},set(m,h,v){let y=m[h];if(y!=null&&typeof y.set==="function"){if(typeof v==="function")y.set(()=>v);else y.set(v);return!0}return!1}});if(u.mount){let m=u.mount(s,r);if(typeof m==="function")o.push(m)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});function ar(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function to(e,n,r,o,t){let a=n!==null?`<span class="key">"${ar(n)}"</span>: `:"";if(e===null)return`${a}<span class="null">null</span>`;if(typeof e==="string")return`${a}<span class="s">"${ar(e)}"</span>`;if(typeof e==="number")return`${a}<span class="n">${e}</span>`;if(typeof e==="boolean")return`${a}<span class="b">${String(e)}</span>`;let b=Array.isArray(e),c=b?e.map((u,s)=>[String(s),u]):Object.entries(e),f=o||r<t,p=b?"[":"{",d=b?"]":"}",l=c.length;if(l===0)return`${a}${p}${d}`;let i=c.map(([u,s])=>`<div class="node">${to(s,b?null:u,r+1,o,t)}</div>`).join("");return`<details${f?" open":""}>
<summary><span class="caret">▶</span>${a}${p}<span class="meta">${l} ${b?"items":"keys"}</span></summary>
<div class="branch">${i}</div>
${d}</details>`}var le=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","tree");if(!n.hasAttribute("aria-label"))n.setAttribute("aria-label","JSON tree");let o=r.contentEl();if(!o)return()=>{};return nt(o,()=>{let t=r.data(),a=r.expanded(),b=r.depth();if(!t)return"";let c;try{c=JSON.parse(t)}catch(f){return`<span class="error" part="error" role="alert">Invalid JSON: ${ar(f.message)}</span>`}return`<div part="root">${to(c,null,0,a,b)}</div>`})}});var Yh=j("sui-json-view",{props:{data:g.string({default:"",reflect:!0}),expanded:g.bool({default:!1,reflect:!0}),depth:g.number({default:1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: var(--leading-normal);
    color: var(--foreground);
  }

  .node { display: block; }
  .branch {
    margin-left: 12px;
    border-left: 1px dashed var(--border);
    padding-left: 10px;
  }
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.expanded=n.expanded,t.depth=n.depth;let a=null;if(le&&typeof le.setup==="function"){if(e.scope=t,a=le.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");if(c.setAttribute("class","json-root"),T(t,"contentEl",c),b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});function no(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function La(e,n){let r=e.split(`
`),o=n.split(`
`),t=r.length,a=o.length,b=Array.from({length:t+1},()=>Array(a+1).fill(0));for(let d=t-1;d>=0;d--)for(let l=a-1;l>=0;l--)b[d][l]=r[d]===o[l]?b[d+1][l+1]+1:Math.max(b[d+1][l],b[d][l+1]);let c=[],f=0,p=0;while(f<t&&p<a)if(r[f]===o[p])c.push({kind:"ctx",text:r[f]}),f++,p++;else if(b[f+1][p]>=b[f][p+1])c.push({kind:"del",text:r[f]}),f++;else c.push({kind:"add",text:o[p]}),p++;while(f<t)c.push({kind:"del",text:r[f++]});while(p<a)c.push({kind:"add",text:o[p++]});return c}function Da(e){return e.split(`
`).map((n)=>{if(n.startsWith("@@"))return{kind:"hunk",text:n};if(n.startsWith("+"))return{kind:"add",text:n.slice(1)};if(n.startsWith("-"))return{kind:"del",text:n.slice(1)};return{kind:"ctx",text:n.startsWith(" ")?n.slice(1):n}})}function Oa(e){return e.map((n)=>{let r=n.kind==="add"?"+":n.kind==="del"?"-":n.kind==="hunk"?"@":" ",o=n.kind==="add"?"added":n.kind==="del"?"removed":n.kind==="hunk"?"hunk":"context";return`<div class="line ${n.kind}" part="line" role="listitem" aria-label="${o}: ${no(n.text)}"><span class="sign" aria-hidden="true">${r}</span><span>${no(n.text)}</span></div>`}).join("")}var de=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","region");if(!n.hasAttribute("aria-label"))n.setAttribute("aria-label","diff");let o=r.contentEl();if(!o)return()=>{};return nt(o,()=>{let t=r.value(),a=r.from(),b=r.to(),c;if(t)c=Da(t);else if(a||b)c=La(a,b);else c=[];return Oa(c)})}});var Ih=j("sui-diff",{props:{from:g.string({default:"",reflect:!0}),to:g.string({default:"",reflect:!0}),value:g.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-mono);
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
  .line.add { background: var(--tone-success-bg); color: var(--foreground); }
  .line.del { background: var(--tone-destructive-bg); color: var(--foreground); }
  .line.hunk { background: var(--tone-info-bg); color: var(--info); }
  .sign { color: var(--muted-foreground); user-select: none; }
  .line.add .sign { color: var(--success); }
  .line.del .sign { color: var(--destructive); }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.from=n.from,t.to=n.to,t.value=n.value;let a=null;if(de&&typeof de.setup==="function"){if(e.scope=t,a=de.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");if(c.setAttribute("class","diff-root"),T(t,"contentEl",c),b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});function ir(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function Gt(e){let n=ir(e);return n=n.replace(/`([^`]+)`/g,(r,o)=>`<code>${o}</code>`),n=n.replace(/\*\*([^*]+)\*\*/g,"<strong>$1</strong>"),n=n.replace(/(^|[^*])\*([^*\n]+)\*/g,"$1<em>$2</em>"),n=n.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g,(r,o,t)=>`<a href="${t}" rel="noopener noreferrer">${o}</a>`),n}function Ta(e){let n=e.replace(/\r\n/g,`
`).split(`
`),r=[],o=0;while(o<n.length){let t=n[o];if(/^```/.test(t)){let c=t.slice(3).trim(),f=[];o++;while(o<n.length&&!/^```/.test(n[o]))f.push(n[o++]);o++,r.push(`<pre data-lang="${ir(c)}"><code>${ir(f.join(`
`))}</code></pre>`);continue}let a=/^(#{1,6})\s+(.*)$/.exec(t);if(a){r.push(`<h${a[1].length}>${Gt(a[2])}</h${a[1].length}>`),o++;continue}if(/^\s*[-*]\s+/.test(t)){let c=[];while(o<n.length&&/^\s*[-*]\s+/.test(n[o]))c.push(`<li>${Gt(n[o].replace(/^\s*[-*]\s+/,""))}</li>`),o++;r.push(`<ul>${c.join("")}</ul>`);continue}if(/^\s*\d+\.\s+/.test(t)){let c=[];while(o<n.length&&/^\s*\d+\.\s+/.test(n[o]))c.push(`<li>${Gt(n[o].replace(/^\s*\d+\.\s+/,""))}</li>`),o++;r.push(`<ol>${c.join("")}</ol>`);continue}if(/^>\s?/.test(t)){let c=[];while(o<n.length&&/^>\s?/.test(n[o]))c.push(n[o].replace(/^>\s?/,"")),o++;r.push(`<blockquote>${Gt(c.join(" "))}</blockquote>`);continue}if(t.trim()===""){o++;continue}let b=[];while(o<n.length&&n[o].trim()!==""&&!/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(n[o]))b.push(n[o]),o++;r.push(`<p>${Gt(b.join(" "))}</p>`)}return r.join(`
`)}var ce=O({mount(e){let n=e,r=n.contentEl();if(!r)return()=>{};return nt(r,()=>Ta(n.value()??""))}});var rv=j("sui-markdown",{props:{value:g.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-base);
    line-height: var(--leading-normal);
    color: var(--foreground);
  }

  h1, h2, h3, h4, h5, h6 {
    font-family: var(--heading-font, var(--font-body));
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
    font-family: var(--font-mono);
    font-size: 0.92em;
    padding: 1px 5px;
    border-radius: var(--radius-sm);
    background: var(--muted);
    border: 1px solid var(--border);
  }
  pre {
    font-family: var(--font-mono);
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value;let a=null;if(ce&&typeof ce.setup==="function"){if(e.scope=t,a=ce.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");if(c.setAttribute("class","md-root"),T(t,"contentEl",c),b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});var eo={actions:"#view-topbar-actions","status-start":"#view-status-start","status-end":"#view-status-end"},pe={title:"#breadcrumbs-title",branch:"#status-branch"},fe=O({mount(e,n){let r=e,o={};function t(){for(let f of Object.keys(eo)){let p=document.querySelector(eo[f]);if(!p)continue;let d=Array.from(n.querySelectorAll(`[slot="${f}"]`));if(d.length===0)continue;for(let l of d)p.appendChild(l);o[f]=d}}function a(){let f=document.querySelector(pe.title);if(f)f.textContent="";if(r.branch()){let p=document.querySelector(pe.branch);if(p)p.textContent=""}if(r.ahead()>0||r.behind()>0){let p=document.querySelector("#status-ab");if(p)p.textContent=""}if(r.tone()!=="default"||r.busy()){let p=document.querySelector("#status-dot");if(p)delete p.dataset.tone,p.removeAttribute("data-busy")}}function b(){for(let f of Object.keys(o)){for(let p of o[f]??[])p.remove();delete o[f]}a()}let c=D(()=>{let f=r.title(),p=r.branch(),d=r.ahead(),l=r.behind(),i=r.tone(),u=r.busy();if(f){let s=document.querySelector(pe.title);if(s)s.textContent=f}if(p){let s=document.querySelector(pe.branch);if(s)s.textContent=p}if(d>0||l>0){let s=document.querySelector("#status-ab");if(s){let m=[];if(d>0)m.push(`↑${d}`);if(l>0)m.push(`↓${l}`);s.textContent=m.join(" ")}}if(i!=="default"||u){let s=document.querySelector("#status-dot");if(s)if(s.dataset.tone=i,u)s.setAttribute("data-busy","");else s.removeAttribute("data-busy")}});return queueMicrotask(()=>t()),()=>{c(),b()}}});var lv=j("sui-view-chrome",{props:{title:g.string({default:"",reflect:!0}),branch:g.string({default:"",reflect:!0}),ahead:g.number({default:0,reflect:!0}),behind:g.number({default:0,reflect:!0}),tone:g.union(["default","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),busy:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.title=n.title,t.branch=n.branch,t.ahead=n.ahead,t.behind=n.behind,t.tone=n.tone,t.busy=n.busy;let a=null;if(fe&&typeof fe.setup==="function"){if(e.scope=t,a=fe.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createTextNode(`

`);b.appendChild(c);let f=document.createTextNode(`
`);b.appendChild(f);let p=document.createTextNode(`
`);b.appendChild(p);let d=document.createTextNode(`
`);b.appendChild(d);let l=document.createTextNode(`
`);b.appendChild(l);let i=document.createTextNode(`
`);b.appendChild(i);let u=document.createTextNode(`

`);b.appendChild(u);let s=document.createTextNode(`
`);if(b.appendChild(s),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let h=new Proxy(t,{get(v,y){return v[y]},set(v,y,w){let k=v[y];if(k!=null&&typeof k.set==="function"){if(typeof w==="function")k.set(()=>w);else k.set(w);return!0}return!1}});if(m.mount){let v=m.mount(h,r);if(typeof v==="function")o.push(v)}if(m.unmount)o.push(()=>m.unmount(h))}}return t}});var me=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","navigation");let o=()=>Array.from(n.querySelectorAll(":scope > sui-navigation-item")),t=()=>o().filter((d)=>!d.hasAttribute("disabled")),a=()=>{let d=r.value();for(let l of o())l.toggleAttribute("active",l.getAttribute("value")===d)},b=(d)=>{let l=d;l.stopPropagation();let i=l.detail.value;r.value.set(i),a(),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:i},bubbles:!0,composed:!0}))},c=(d)=>{if(d.key!=="ArrowRight"&&d.key!=="ArrowLeft"&&d.key!=="Home"&&d.key!=="End")return;let l=t();if(!l.length)return;let i=d.composedPath().find((m)=>l.includes(m)),u=i?l.indexOf(i):-1,s;if(d.key==="ArrowRight")s=l[(u+1)%l.length];else if(d.key==="ArrowLeft")s=l[(u-1+l.length)%l.length];else if(d.key==="Home")s=l[0];else if(d.key==="End")s=l[l.length-1];if(s)d.preventDefault(),s.focusLink?.()},f=D(()=>{r.value(),a()}),p=D(()=>{let d=r.defaultSlotEl();if(!d)return;let l=()=>a();return d.addEventListener("slotchange",l),()=>d.removeEventListener("slotchange",l)});return n.addEventListener("sui-nav-select",b),n.addEventListener("keydown",c),()=>{f(),p(),n.removeEventListener("sui-nav-select",b),n.removeEventListener("keydown",c)}}});var bv=j("sui-navigation-menu",{props:{value:g.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    align-items: stretch;
    gap: 0;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-body);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value;let a=null;if(me&&typeof me.setup==="function"){if(e.scope=t,a=me.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(T(t,"defaultSlotEl",c),b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(f.mount){let d=f.mount(p,r);if(typeof d==="function")o.push(d)}if(f.unmount)o.push(()=>f.unmount(p))}}return t}});var be=O({mount(e,n){let r=e,o=(t)=>{if(r.disabled())return;if(!r.href())t.preventDefault();n.dispatchEvent(new CustomEvent("sui-nav-select",{detail:{value:r.value()},bubbles:!0,composed:!0}))};return n.focusLink=()=>{n.shadowRoot?.querySelector(".link")?.focus()},n.addEventListener("click",o),()=>n.removeEventListener("click",o)}});var wv=j("sui-navigation-item",{props:{value:g.string({default:"",reflect:!0}),active:g.bool({default:!1,reflect:!0}),disabled:g.bool({default:!1,reflect:!0}),href:g.string({default:""})},shadow:!0,styles:`
  :host { display: inline-flex; position: relative; }

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
    outline: none;
    box-shadow:
      0 0 0 var(--ring-offset, 2px) var(--background),
      0 0 0 calc(var(--ring-offset, 2px) + var(--ring-width, 2px))
        color-mix(in oklch, var(--primary) 50%, transparent);
  }

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
    .link:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
    :host([active]) .indicator { background: CanvasText; }
  }

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.active=n.active,t.disabled=n.disabled,t.href=n.href;let a=null;if(be&&typeof be.setup==="function"){if(e.scope=t,a=be.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>H({anchor:d,test:()=>{if(t.href())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("a");u.setAttribute("class","link"),u.setAttribute("part","link"),i.push(E(u,"href",()=>t.href())),i.push(E(u,"aria-current",()=>t.active()?"page":"false"));let s=document.createElement("slot");return u.appendChild(s),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let m of i)try{m()}catch(h){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("button");u.setAttribute("class","link"),u.setAttribute("part","link"),u.setAttribute("type","button"),i.push(E(u,"aria-current",()=>t.active()?"true":"false")),i.push(E(u,"disabled",()=>t.disabled()));let s=document.createElement("slot");return u.appendChild(s),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let m of i)try{m()}catch(h){}}}}]}),c=r.shadowRoot??r;L(c);let f=document.createComment("if");c.appendChild(f),o.push(b(f));let p=document.createElement("div");if(p.setAttribute("class","indicator"),p.setAttribute("part","indicator"),p.setAttribute("aria-hidden","true"),c.appendChild(p),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var he=O({mount(e,n){let r=e,o=null,t=()=>{let c=r.viewportEl();if(!c)return;let f=r.orientation(),p=f==="vertical"||f==="both",d=f==="horizontal"||f==="both",l=r.vThumb(),i=r.hThumb();if(p&&l){let u=c.clientHeight/c.scrollHeight;if(u<1){let s=Math.max(24,u*c.clientHeight),h=c.scrollTop/(c.scrollHeight-c.clientHeight)*(c.clientHeight-s);l.style.height=`${s}px`,l.style.top=`${h}px`,l.style.display=""}else l.style.display="none"}if(d&&i){let u=c.clientWidth/c.scrollWidth;if(u<1){let s=Math.max(24,u*c.clientWidth),h=c.scrollLeft/(c.scrollWidth-c.clientWidth)*(c.clientWidth-s);i.style.width=`${s}px`,i.style.left=`${h}px`,i.style.display=""}else i.style.display="none"}},a=()=>{t();let c=r.vScrollbar(),f=r.hScrollbar();if(c?.classList.add("active"),f?.classList.add("active"),o)clearTimeout(o);o=setTimeout(()=>{c?.classList.remove("active"),f?.classList.remove("active")},1200)},b=r.viewportEl();if(b)b.addEventListener("scroll",a,{passive:!0});return t(),()=>{if(o)clearTimeout(o);if(b)b.removeEventListener("scroll",a)}}});var jv=j("sui-scroll-area",{props:{orientation:g.union(["vertical","horizontal","both"],{default:"vertical",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    position: relative;
    overflow: hidden;
  }

  :host(:not([size])), :host([size='md']) { --sb-size: 8px; }
  :host([size='sm']) { --sb-size: 6px; }
  :host([size='lg']) { --sb-size: 10px; }

  .viewport {
    width: 100%;
    height: 100%;
    overflow: hidden;
    scrollbar-width: none;
    -ms-overflow-style: none;
  }
  .viewport::-webkit-scrollbar { display: none; }

  :host(:not([orientation])) .viewport,
  :host([orientation='vertical']) .viewport { overflow-y: auto; }
  :host([orientation='horizontal']) .viewport { overflow-x: auto; }
  :host([orientation='both']) .viewport { overflow: auto; }

  /* Overlay scrollbar tracks — absolute-positioned, opacity-fade on hover/scroll */
  .scrollbar {
    position: absolute;
    opacity: 0;
    transition: opacity var(--duration-medium) var(--ease-out);
    z-index: 1;
  }
  :host(:hover) .scrollbar,
  :host(:focus-within) .scrollbar,
  .scrollbar.active { opacity: 1; }

  .scrollbar-v {
    top: 0; right: 2px; bottom: 0;
    width: var(--sb-size);
  }
  .scrollbar-h {
    left: 0; right: 0; bottom: 2px;
    height: var(--sb-size);
  }

  /* Orientation-driven track visibility */
  :host(:not([orientation])) .scrollbar-h,
  :host([orientation='vertical']) .scrollbar-h { display: none; }
  :host([orientation='horizontal']) .scrollbar-v { display: none; }

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

  @media (prefers-reduced-motion: reduce) {
    .scrollbar { transition: none; }
    .thumb { transition: none; }
  }

  @media (forced-colors: active) {
    .thumb { background: CanvasText; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.orientation=n.orientation,t.size=n.size;let a=null;if(he&&typeof he.setup==="function"){if(e.scope=t,a=he.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","viewport"),c.setAttribute("part","viewport"),c.setAttribute("tabindex","0"),T(t,"viewportEl",c);let f=document.createElement("slot");c.appendChild(f),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","scrollbar scrollbar-v"),p.setAttribute("part","scrollbar-v"),p.setAttribute("aria-hidden","true"),T(t,"vScrollbar",p);let d=document.createElement("div");d.setAttribute("class","thumb"),T(t,"vThumb",d),p.appendChild(d),b.appendChild(p);let l=document.createElement("div");l.setAttribute("class","scrollbar scrollbar-h"),l.setAttribute("part","scrollbar-h"),l.setAttribute("aria-hidden","true"),T(t,"hScrollbar",l);let i=document.createElement("div");if(i.setAttribute("class","thumb"),T(t,"hThumb",i),l.appendChild(i),b.appendChild(l),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(m,h){return m[h]},set(m,h,v){let y=m[h];if(y!=null&&typeof y.set==="function"){if(typeof v==="function")y.set(()=>v);else y.set(v);return!0}return!1}});if(u.mount){let m=u.mount(s,r);if(typeof m==="function")o.push(m)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var ve=O({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","toolbar");if(!n.hasAttribute("aria-orientation"))n.setAttribute("aria-orientation","horizontal");let o=()=>{return(r.defaultSlot()?.assignedElements({flatten:!0})??[]).filter((c)=>!c.hasAttribute("disabled"))},t=(a)=>{if(a.key!=="ArrowRight"&&a.key!=="ArrowLeft")return;let b=o();if(!b.length)return;let c=a.composedPath().find((d)=>b.includes(d))||null,f=c?b.indexOf(c):-1,p=a.key==="ArrowRight"?(f+1)%b.length:(f-1+b.length)%b.length;b[p]?.focus(),a.preventDefault()};return n.addEventListener("keydown",t),()=>n.removeEventListener("keydown",t)}});var Pv=j("sui-toolbar",{props:{justify:g.union(["start","center","end","between"],{default:"start",reflect:!0}),align:g.union(["center","start","end","baseline"],{default:"center",reflect:!0}),gap:g.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:g.union(["default","bordered"],{default:"default",reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    align-items: center;
    gap: 8px;
  }
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.justify=n.justify,t.align=n.align,t.gap=n.gap,t.variant=n.variant;let a=null;if(ve&&typeof ve.setup==="function"){if(e.scope=t,a=ve.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");c.setAttribute("name","spacer"),b.appendChild(c);let f=document.createElement("slot");if(T(t,"defaultSlot",f),b.appendChild(f),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var ro={dashboard:"layout-dashboard",pipelines:"workflow",agents:"bot",skills:"sparkles",git:"git-branch",collections:"database",canvas:"frame",settings:"settings",chat:"messages-square",events:"radio",specs:"file-text",theme:"palette",setup:"rocket",overview:"compass",mesh:"network"},oo={"theme-studio":"theme"},Ma=new Set([...Object.keys(ro),...Object.keys(oo)]),ge=O({mount(e,n){let r=e,o=D(()=>{let t=r.name(),a=r.label(),b=oo[t]??t,f=Ma.has(t)?ro[b]:"dot",p=r.glyphEl();if(p)p.setAttribute("name",f);if(a)n.setAttribute("role","img"),n.setAttribute("aria-label",a),n.removeAttribute("aria-hidden");else n.setAttribute("aria-hidden","true"),n.removeAttribute("role"),n.removeAttribute("aria-label")});return()=>o()}});var Gv=j("sui-app-icon",{props:{name:g.string({default:"",reflect:!0}),variant:g.union(["glyph","tile","solid"],{default:"glyph",reflect:!0}),size:g.union(["sm","md","lg"],{default:"md",reflect:!0}),radius:g.string({default:"",reflect:!0}),label:g.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    vertical-align: middle;

    --app-accent: var(--foreground);
    --tile-size: 20px;
    --glyph-size: 14px;
    --tile-radius: var(--radius-md);

    width: var(--tile-size);
    height: var(--tile-size);
    color: var(--app-accent);
  }

  :host([name='dashboard'])   { --app-accent: var(--app-accent-dashboard); }
  :host([name='pipelines'])   { --app-accent: var(--app-accent-pipelines); }
  :host([name='agents'])      { --app-accent: var(--app-accent-agents); }
  :host([name='skills'])      { --app-accent: var(--app-accent-skills); }
  :host([name='git'])         { --app-accent: var(--app-accent-git); }
  :host([name='collections']) { --app-accent: var(--app-accent-collections); }
  :host([name='canvas'])      { --app-accent: var(--app-accent-canvas); }
  :host([name='settings'])    { --app-accent: var(--app-accent-settings); }
  :host([name='chat'])        { --app-accent: var(--app-accent-chat); }
  :host([name='events'])      { --app-accent: var(--app-accent-events); }
  :host([name='specs'])       { --app-accent: var(--app-accent-specs); }
  :host([name='theme'])       { --app-accent: var(--app-accent-theme); }
  :host([name='setup'])       { --app-accent: var(--app-accent-setup); }
  :host([name='overview'])    { --app-accent: var(--app-accent-overview); }
  :host([name='mesh'])        { --app-accent: var(--app-accent-mesh); }
  :host([name='theme-studio']) { --app-accent: var(--app-accent-theme); }

  :host([size='sm']) { --tile-size: 16px; --glyph-size: 12px; --tile-radius: var(--radius-sm); }
  :host([size='md']) { --tile-size: 20px; --glyph-size: 14px; --tile-radius: var(--radius-md); }
  :host([size='lg']) { --tile-size: 28px; --glyph-size: 18px; --tile-radius: var(--radius-lg); }

  :host([radius='sm'])   { --tile-radius: var(--radius-sm); }
  :host([radius='md'])   { --tile-radius: var(--radius-md); }
  :host([radius='lg'])   { --tile-radius: var(--radius-lg); }
  :host([radius='full']) { --tile-radius: var(--radius-full); }

  .frame {
    width: 100%;
    height: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--tile-radius);
    transition: background-color var(--duration-fast) var(--ease-out),
                color var(--duration-fast) var(--ease-out),
                box-shadow var(--duration-fast) var(--ease-out);
  }

  :host(:not([variant])) .frame,
  :host([variant='glyph']) .frame {
    background: transparent;
    color: var(--app-accent);
  }

  :host([variant='tile']) .frame {
    background: color-mix(in oklab, var(--app-accent) 16%, transparent);
    color: var(--app-accent);
  }

  :host([variant='solid']) .frame {
    background: var(--app-accent);
    color: var(--background);
    box-shadow: var(--shadow-xs);
  }

  sui-icon {
    width: var(--glyph-size);
    height: var(--glyph-size);
  }
  ::slotted(*) {
    width: var(--glyph-size);
    height: var(--glyph-size);
    display: block;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.name=n.name,t.variant=n.variant,t.size=n.size,t.radius=n.radius,t.label=n.label;let a=null;if(ge&&typeof ge.setup==="function"){if(e.scope=t,a=ge.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("span");c.setAttribute("class","frame"),c.setAttribute("part","frame");let f=document.createElement("slot"),p=document.createElement("sui-icon");if(T(t,"glyphEl",p),p.setAttribute("part","glyph"),p.setAttribute("aria-hidden","true"),f.appendChild(p),c.appendChild(f),b.appendChild(c),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var ye=O({scope:()=>{return{nodes:P([])}},mount(e,n){let o=e.nodes,t=n.nodes;if(t!=null)o.set(t);if(Object.defineProperty(n,"nodes",{configurable:!0,get:()=>o(),set:(a)=>o.set(a)}),!n.hasAttribute("role"))n.setAttribute("role","tree")}});var Yv=j("sui-tree-view",{shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
  }
  ul { list-style: none; margin: 0; padding: 0; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(ye&&typeof ye.setup==="function"){if(e.scope=t,a=ye.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>X({anchor:d,items:()=>t.nodes(),key:(l,i)=>{let u=Object.create(t);return u.n=l,u.__i=i,((s)=>l.id)(u)},render:(l,i)=>{let u=Object.create(t);return u.n=l,u.__i=i,u.__v=P(0),((s)=>{let m=document.createDocumentFragment(),h=[],v=document.createElement("sui-tree-item");return h.push($(v,"node",()=>(s.__v(),s.n))),m.appendChild(v),{nodes:Array.from(m.childNodes),cleanup:()=>{for(let y of h)try{y()}catch(w){}},update:(y,w)=>{s.n=y,s.__i=w,s.__v.set((k)=>k+1)}}})(u)}}),c=(d)=>H({anchor:d,test:()=>{if(t.nodes().length>0)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("ul");u.setAttribute("role","tree");let s=document.createComment("for");return u.appendChild(s),i.push(b(s)),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let m of i)try{m()}catch(h){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("slot");return l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(m){}}}}]}),f=r.shadowRoot??r;L(f);let p=document.createComment("if");if(f.appendChild(p),o.push(c(p)),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let m=i[u];if(m!=null&&typeof m.set==="function"){if(typeof s==="function")m.set(()=>s);else m.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var xe=O({scope:({label:e})=>{let n=P(null),r=J(()=>{let a=n();return a?a.label:e()}),o=J(()=>{return n()?.children??[]}),t=Y(null);return{node:n,nodeLabel:r,nodeChildren:o,rowEl:t}},mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","treeitem");let o=r.node,t=n.node;if(t!=null)o.set(t);Object.defineProperty(n,"node",{configurable:!0,get:()=>o(),set:(i)=>o.set(i)});let a=D(()=>{let i=o();if(i!==null)if(!i.children||i.children.length===0)n.setAttribute("leaf","");else n.removeAttribute("leaf")}),b=D(()=>{let i=r.rowEl();if(!i)return;if(!r.leaf())i.setAttribute("aria-expanded",String(r.expanded()));else i.removeAttribute("aria-expanded");i.setAttribute("aria-selected",String(r.selected()))}),c=(i)=>{if(r.leaf())return;i.stopPropagation();let u=!r.expanded();r.expanded.set(u),n.dispatchEvent(new CustomEvent("sui-toggle",{detail:{expanded:u},bubbles:!0,composed:!0}))},f=()=>{n.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},p=(i)=>{f(),c(i)},d=(i)=>{if(i.key==="Enter"||i.key===" ")i.preventDefault(),f();else if(i.key==="ArrowRight"&&!r.leaf()&&!r.expanded())i.preventDefault(),c(i);else if(i.key==="ArrowLeft"&&r.expanded())i.preventDefault(),c(i)},l=D(()=>{let i=r.rowEl();if(!i)return;return i.addEventListener("click",p),i.addEventListener("keydown",d),()=>{i.removeEventListener("click",p),i.removeEventListener("keydown",d)}});return()=>{a(),b(),l()}}});var Iv=j("sui-tree-item",{props:{label:g.string({default:"",reflect:!0}),expanded:g.bool({default:!1,reflect:!0}),selected:g.bool({default:!1,reflect:!0}),leaf:g.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
    background: var(--tone-primary-bg);
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
    width: 12px;
    height: 12px;
    color: var(--muted-foreground);
    transition: transform var(--duration-fast) var(--ease-out);
  }
  :host([expanded]) .caret { transform: rotate(90deg); }
  :host([leaf]) .caret { visibility: hidden; }

  .label {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .trailing { margin-left: auto; color: var(--muted-foreground); font-size: var(--text-xs); }

  .children {
    margin-left: 16px;
    border-left: 1px dashed var(--border);
    padding-left: 6px;
  }
  :host(:not([expanded])) .children { display: none; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.label=n.label,t.expanded=n.expanded,t.selected=n.selected,t.leaf=n.leaf;let a=null;if(xe&&typeof xe.setup==="function"){if(e.scope=t,a=xe.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(k)=>X({anchor:k,items:()=>t.nodeChildren(),key:(A,x)=>{let C=Object.create(t);return C.c=A,C.__i=x,((z)=>A.id)(C)},render:(A,x)=>{let C=Object.create(t);return C.c=A,C.__i=x,C.__v=P(0),((z)=>{let M=document.createDocumentFragment(),N=[],S=document.createElement("sui-tree-item");return N.push($(S,"node",()=>(z.__v(),z.c))),M.appendChild(S),{nodes:Array.from(M.childNodes),cleanup:()=>{for(let B of N)try{B()}catch(U){}},update:(B,U)=>{z.c=B,z.__i=U,z.__v.set((G)=>G+1)}}})(C)}}),c=(k)=>H({anchor:k,test:()=>{if(t.nodeChildren().length>0)return 0;return 1},branches:[()=>{let A=document.createDocumentFragment(),x=[],C=document.createComment("for");return A.appendChild(C),x.push(b(C)),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let z of x)try{z()}catch(M){}}}},()=>{let A=document.createDocumentFragment(),x=[],C=document.createElement("slot");return A.appendChild(C),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let z of x)try{z()}catch(M){}}}}]}),f=r.shadowRoot??r;L(f);let p=document.createElement("div");p.setAttribute("class","row"),p.setAttribute("part","row"),p.setAttribute("tabindex","0"),T(t,"rowEl",p);let d=document.createElement("span");d.setAttribute("class","caret"),d.setAttribute("part","caret"),d.setAttribute("aria-hidden","true");let l=document.createElement("svg");l.setAttribute("viewbox","0 0 12 12"),l.setAttribute("fill","none"),l.setAttribute("stroke","currentColor"),l.setAttribute("stroke-width","1.5");let i=document.createElement("polyline");i.setAttribute("points","4 3 8 6 4 9"),l.appendChild(i),d.appendChild(l),p.appendChild(d);let u=document.createElement("slot");u.setAttribute("name","icon"),p.appendChild(u);let s=document.createElement("span");s.setAttribute("class","label"),s.setAttribute("part","label");let m=document.createTextNode("");s.appendChild(m),o.push(F(m,()=>t.nodeLabel())),p.appendChild(s);let h=document.createElement("span");h.setAttribute("class","trailing");let v=document.createElement("slot");v.setAttribute("name","trailing"),h.appendChild(v),p.appendChild(h),f.appendChild(p);let y=document.createElement("div");y.setAttribute("class","children"),y.setAttribute("part","children"),y.setAttribute("role","group");let w=document.createComment("if");if(y.appendChild(w),o.push(c(w)),f.appendChild(y),a!=null){let k=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(k){let A=new Proxy(t,{get(x,C){return x[C]},set(x,C,z){let M=x[C];if(M!=null&&typeof M.set==="function"){if(typeof z==="function")M.set(()=>z);else M.set(z);return!0}return!1}});if(k.mount){let x=k.mount(A,r);if(typeof x==="function")o.push(x)}if(k.unmount)o.push(()=>k.unmount(A))}}return t}});var ao=new WeakSet;function Pa(e){if(ao.has(e))return;ao.add(e);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let r=new CSSStyleSheet;r.replaceSync(io),e.adoptedStyleSheets=[...e.adoptedStyleSheets,r];return}}catch{}let n=e.createElement("style");n.setAttribute("data-sui-table",""),n.textContent=io,e.head.appendChild(n)}var we=O({scope:()=>{let e=P([]),n=P([]),r=P(null),o=P("asc"),t=P([]),a=J(()=>{let b=r(),c=o(),f=e();if(!b)return f;return[...f].sort((p,d)=>{let l=String(p[b]??""),i=String(d[b]??""),u=l<i?-1:l>i?1:0;return c==="asc"?u:-u})});return{rows:e,columns:n,sortKey:r,sortDir:o,selected:t,sortedRows:a}},on:{sortBy(e,n){if(e.sortKey===n)e.sortDir=e.sortDir==="asc"?"desc":"asc";else e.sortKey=n,e.sortDir="asc"},toggleSelect(e,n){let r=e.selected;e.selected=r.includes(n)?r.filter((o)=>o!==n):[...r,n]},selectAll(e){e.selected=e.rows.map((n)=>n.id)},clearSelection(e){e.selected=[]}},mount(e,n){let r=e,o=r.rows,t=r.columns,a=n.rows;if(a!=null)o.set(a);let b=n.columns;if(b!=null)t.set(b);Object.defineProperty(n,"rows",{configurable:!0,get:()=>o(),set:(m)=>o.set(m)}),Object.defineProperty(n,"columns",{configurable:!0,get:()=>t(),set:(m)=>t.set(m)}),Pa(n.ownerDocument);let c=()=>{n.querySelectorAll("th[data-sort-key]").forEach((m)=>{let h=m.getAttribute("aria-sort")||"none",v=m.querySelector(".sui-sort-indicator");if(!v)return;if(h==="ascending")v.style.transform="rotate(180deg)",v.style.opacity="1";else if(h==="descending")v.style.transform="rotate(0deg)",v.style.opacity="1";else v.style.transform="rotate(0deg)",v.style.opacity="0.4"})},f=()=>{n.querySelectorAll("th[data-sort-key]").forEach((m)=>{if(!m.hasAttribute("aria-sort"))m.setAttribute("aria-sort","none");if(!m.hasAttribute("tabindex"))m.setAttribute("tabindex","0");if(!m.hasAttribute("role"))m.setAttribute("role","columnheader");if(!m.querySelector(".sui-sort-indicator")){let h=document.createElement("span");h.className="sui-sort-indicator",h.setAttribute("aria-hidden","true"),h.style.cssText="margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;";let v="http://www.w3.org/2000/svg",y=document.createElementNS(v,"svg");y.setAttribute("width","10"),y.setAttribute("height","10"),y.setAttribute("viewBox","0 0 10 10"),y.setAttribute("fill","none"),y.setAttribute("stroke","currentColor"),y.setAttribute("stroke-width","1.5"),y.setAttribute("stroke-linecap","round"),y.setAttribute("stroke-linejoin","round"),y.setAttribute("aria-hidden","true");let w=document.createElementNS(v,"path");w.setAttribute("d","M2 4l3-2 3 2M2 6l3 2 3-2"),y.appendChild(w),h.appendChild(y),m.appendChild(h)}}),c()},p=()=>{let m=n.querySelector(":scope > table tbody > tr")||n.querySelector(":scope > table > tr:not(:has(th))")||n.querySelector(":scope > tbody > tr")||n.querySelector(':scope > tr:not([slot="head"])'),h=n.querySelector('[slot="empty"]')!=null;if(!m&&h)n.setAttribute("data-empty","");else n.removeAttribute("data-empty")},d=(m)=>{let h=m.getAttribute("data-sort-key")||"",v=m.getAttribute("aria-sort")||"none",y=v==="none"?"ascending":v==="ascending"?"descending":"none";n.querySelectorAll("th[data-sort-key]").forEach((w)=>{if(w!==m)w.setAttribute("aria-sort","none")}),m.setAttribute("aria-sort",y),c(),n.emit("sui-sort",{key:h,direction:y})},l=(m)=>{let h=m.target.closest("th[data-sort-key]");if(!h||!n.contains(h))return;d(h)},i=(m)=>{if(m.key!=="Enter"&&m.key!==" ")return;let h=m.target.closest("th[data-sort-key]");if(!h||!n.contains(h))return;m.preventDefault(),d(h)},u=(m)=>{let h=m.target.closest("td");if(!h||!n.contains(h))return;if(h.classList.contains("sui-table-gutter-cell"))return;let v=h.closest("tr");if(!v)return;let y=v.getAttribute("data-row-id");if(!y)return;n.emit("sui-row-click",{id:y,index:v.rowIndex})};W(n,"click",l),W(n,"click",u),W(n,"keydown",i),f(),p();let s=new MutationObserver(()=>{f(),p()});return s.observe(n,{childList:!0,subtree:!0}),()=>{s.disconnect()}}}),io=`
  sui-table {
    --sui-table-border-soft: color-mix(in oklch, var(--border) 80%, transparent);
  }

  sui-table[variant='native'] > table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    table-layout: auto;
    font-variant-numeric: tabular-nums;
  }

  sui-table th,
  sui-table td {
    text-align: left;
    padding: var(--row-py, 10px) var(--row-px, 12px);
    vertical-align: middle;
    border-bottom: 1px solid var(--sui-table-border-soft);
    font-size: var(--text-base, 13px);
    color: var(--foreground);
    box-sizing: border-box;
  }
  sui-table th {
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
  sui-table thead {
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  sui-table > tr:last-of-type > *,
  sui-table tbody tr:last-child > * {
    border-bottom: 0;
  }

  sui-table th[align='right'], sui-table td[align='right'] { text-align: right; }
  sui-table th[align='center'], sui-table td[align='center'] { text-align: center; }

  sui-table .mono {
    font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
    font-size: var(--text-sm, 12px);
    letter-spacing: -0.01em;
  }
  sui-table .muted { color: var(--muted-foreground); }
  sui-table .null { color: var(--muted-foreground); font-style: italic; }

  sui-table > tr:not([slot='head']),
  sui-table tbody tr {
    transition: background-color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  sui-table:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  sui-table:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table[striped] > tr:not([slot='head']):nth-of-type(even) > *,
  sui-table[striped] tbody tr:nth-child(even) > * {
    background: color-mix(in oklch, var(--muted) 25%, transparent);
  }
  sui-table[striped]:not([hoverable='false']) > tr:not([slot='head']):hover > *,
  sui-table[striped]:not([hoverable='false']) tbody tr:hover > * {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table tr[data-selected] > *,
  sui-table tr[aria-selected='true'] > * {
    background: var(--tone-primary-bg);
  }
  sui-table tr[data-selected] > *:first-child,
  sui-table tr[aria-selected='true'] > *:first-child {
    box-shadow: inset 3px 0 0 0 var(--primary);
  }

  sui-table[sticky-header] thead th,
  sui-table[sticky-header] tr[slot='head'] > th {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky, 20);
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
    box-shadow: 0 1px 0 0 var(--border);
  }

  sui-table th[data-sort-key],
  sui-table th.sortable {
    cursor: pointer;
    -webkit-tap-highlight-color: transparent;
    transition: color var(--duration-fast, 120ms) var(--ease-out, cubic-bezier(0.16,1,0.3,1));
  }
  sui-table th[data-sort-key]:hover,
  sui-table th.sortable:hover {
    color: var(--foreground);
  }
  sui-table th[aria-sort='ascending'],
  sui-table th[aria-sort='descending'] {
    color: var(--foreground);
  }
  sui-table th[data-sort-key]:focus-visible {
    outline: none;
    box-shadow: inset 0 0 0 var(--ring-width, 3px)
      color-mix(in oklch, var(--ring-color, var(--primary)) var(--ring-opacity, 25%), transparent);
  }

  sui-table[data-empty] > table {
    display: none;
  }

  sui-table[sticky-first-col] th:first-child,
  sui-table[sticky-first-col] td:first-child {
    position: sticky;
    left: 0;
    z-index: 1;
    background: var(--card);
    border-right: 1px solid var(--border);
  }
  sui-table[sticky-first-col] thead th:first-child {
    z-index: 3;
    background: color-mix(in oklch, var(--muted) 55%, var(--card));
  }
  sui-table[sticky-first-col] tbody tr:hover td:first-child {
    background: color-mix(in oklch, var(--muted) var(--row-hover-mix, 45%), transparent);
  }

  sui-table[gutter] th.sui-table-gutter-cell,
  sui-table[gutter] td.sui-table-gutter-cell {
    width: 40px;
    min-width: 40px;
    max-width: 40px;
    padding: 0;
    text-align: center;
    color: var(--muted-foreground);
    background: color-mix(in oklch, var(--muted-foreground) 5%, var(--background));
    border-right: 1px solid var(--border);
    position: sticky;
    left: 0;
    z-index: 2;
  }
  sui-table[gutter] thead th.sui-table-gutter-cell { z-index: 4; }

  sui-table tr[data-row-id] > td:not(.sui-table-gutter-cell) {
    cursor: pointer;
  }

  @media (forced-colors: active) {
    sui-table th, sui-table td { border-bottom-color: CanvasText; }
  }
`;var rg=j("sui-table",{props:{variant:g.union(["slot","native","data"],{default:"slot",reflect:!0}),density:g.union(["compact","default","comfortable"],{default:"default",reflect:!0}),dense:g.bool({default:!1,reflect:!0}),hoverable:g.bool({default:!0,reflect:!0}),striped:g.bool({default:!1,reflect:!0}),"sticky-header":g.bool({default:!1,reflect:!0}),"sticky-first-col":g.bool({default:!1,reflect:!0}),gutter:g.bool({default:!1,reflect:!0}),label:g.string({default:""})},shadow:!0,styles:`
  :host {
    display: block;
    width: 100%;
    font-family: var(--font-body);
    font-size: var(--text-base);
    color: var(--foreground);

    --row-py: 10px;
    --row-px: var(--space-3);
    --th-py: 10px;
    --th-fs: var(--text-xs);
    --table-radius: var(--radius-lg);
    --row-hover-mix: 45%;
  }

  :host([density='compact']),
  :host([dense]) {
    --row-py: 6px;
    --row-px: var(--space-2);
    --th-py: 6px;
  }
  :host([density='comfortable']) {
    --row-py: 14px;
    --row-px: var(--space-4);
    --th-py: 14px;
  }

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

  :host([sticky-header]) { display: block; }
  :host([sticky-header]) .wrap {
    overflow-y: auto;
    max-height: inherit;
  }

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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.variant=n.variant,t.density=n.density,t.dense=n.dense,t.hoverable=n.hoverable,t.striped=n.striped,t.stickyHeader=n.stickyHeader,t.stickyFirstCol=n.stickyFirstCol,t.gutter=n.gutter,t.label=n.label;let a=null;if(we&&typeof we.setup==="function"){if(e.scope=t,a=we.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(l)=>X({anchor:l,items:()=>t.columns(),key:(i,u)=>{let s=Object.create(t);return s.c=i,s.__i=u,((m)=>i.field)(s)},render:(i,u)=>{let s=Object.create(t);return s.c=i,s.__i=u,s.__v=P(0),((m)=>{let h=document.createDocumentFragment(),v=[],y=document.createElement("th");v.push(V(y,"click",function(){let k=()=>m.sortBy(m.c.field);return typeof k==="function"?k.bind(void 0):()=>{}}(),{modifiers:[]})),v.push(gt(y,"sorted",()=>(m.__v(),m.sortKey()===m.c.field)));let w=document.createTextNode("");return y.appendChild(w),v.push(F(w,()=>(m.__v(),m.c.label))),h.appendChild(y),{nodes:Array.from(h.childNodes),cleanup:()=>{for(let k of v)try{k()}catch(A){}},update:(k,A)=>{m.c=k,m.__i=A,m.__v.set((x)=>x+1)}}})(s)}}),c=(l)=>X({anchor:l,items:()=>t.sortedRows(),key:(i,u)=>{let s=Object.create(t);return s.r=i,s.__i=u,((m)=>i.id)(s)},render:(i,u)=>{let s=Object.create(t);return s.r=i,s.__i=u,s.__v=P(0),((m)=>{let h=(A)=>X({anchor:A,items:()=>m.columns(),key:(x,C)=>{let z=Object.create(m);return z.c=x,z.__i=C,((M)=>x.field)(z)},render:(x,C)=>{let z=Object.create(m);return z.c=x,z.__i=C,z.__v=P(0),((M)=>{let N=document.createDocumentFragment(),S=[],B=document.createElement("td"),U=document.createTextNode("");return B.appendChild(U),S.push(F(U,()=>(M.__v(),M.r[M.c.field]))),N.appendChild(B),{nodes:Array.from(N.childNodes),cleanup:()=>{for(let G of S)try{G()}catch(q){}},update:(G,q)=>{M.c=G,M.__i=q,M.__v.set((Q)=>Q+1)}}})(z)}}),v=document.createDocumentFragment(),y=[],w=document.createElement("tr");y.push(gt(w,"selected",()=>(m.__v(),m.selected().includes(m.r.id)))),y.push(V(w,"click",function(){let A=()=>m.toggleSelect(m.r.id);return typeof A==="function"?A.bind(void 0):()=>{}}(),{modifiers:[]}));let k=document.createComment("for");return w.appendChild(k),y.push(h(k)),v.appendChild(w),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let A of y)try{A()}catch(x){}},update:(A,x)=>{m.r=A,m.__i=x,m.__v.set((C)=>C+1)}}})(s)}}),f=(l)=>H({anchor:l,test:()=>{if(t.variant()==="data")return 0;if(t.variant()!=="native")return 1;return 2},branches:[()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("div");s.setAttribute("class","wrap"),s.setAttribute("part","wrap");let m=document.createElement("table");m.setAttribute("part","table"),u.push(E(m,"aria-label",()=>t.label()||null));let h=document.createElement("thead");h.setAttribute("part","thead");let v=document.createElement("tr"),y=document.createComment("for");v.appendChild(y),u.push(b(y)),h.appendChild(v),m.appendChild(h);let w=document.createElement("tbody");w.setAttribute("part","tbody");let k=document.createComment("for");return w.appendChild(k),u.push(c(k)),m.appendChild(w),s.appendChild(m),i.appendChild(s),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let A of u)try{A()}catch(x){}}}},()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("div");s.setAttribute("class","wrap"),s.setAttribute("part","wrap");let m=document.createElement("table");m.setAttribute("part","table"),u.push(E(m,"aria-label",()=>t.label()||null));let h=document.createElement("thead");h.setAttribute("part","thead");let v=document.createElement("slot");v.setAttribute("name","head"),h.appendChild(v),m.appendChild(h);let y=document.createElement("tbody");y.setAttribute("part","tbody");let w=document.createElement("slot");y.appendChild(w),m.appendChild(y),s.appendChild(m);let k=document.createElement("div");k.setAttribute("class","empty"),k.setAttribute("part","empty"),k.setAttribute("role","status");let A=document.createElement("slot");return A.setAttribute("name","empty"),k.appendChild(A),s.appendChild(k),i.appendChild(s),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let x of u)try{x()}catch(C){}}}},()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("div");s.setAttribute("class","wrap"),s.setAttribute("part","wrap");let m=document.createElement("slot");s.appendChild(m);let h=document.createElement("div");h.setAttribute("class","empty"),h.setAttribute("part","empty"),h.setAttribute("role","status");let v=document.createElement("slot");return v.setAttribute("name","empty"),h.appendChild(v),s.appendChild(h),i.appendChild(s),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let y of u)try{y()}catch(w){}}}}]}),p=r.shadowRoot??r;L(p);let d=document.createComment("if");if(p.appendChild(d),o.push(f(d)),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(u,s){return u[s]},set(u,s,m){let h=u[s];if(h!=null&&typeof h.set==="function"){if(typeof m==="function")h.set(()=>m);else h.set(m);return!0}return!1}});if(l.mount){let u=l.mount(i,r);if(typeof u==="function")o.push(u)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});function et(e){return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#39;")}function so(e,n){if(Array.isArray(e))return e;if(typeof e==="string"&&e.trim())try{let r=JSON.parse(e);return Array.isArray(r)?r:n}catch{return n}return n}function Na(e,n,r,o){let t=new Map,a=e.some((i)=>typeof i.x!=="number"||typeof i.y!=="number");if(o==="positioned"&&!a){for(let i of e)t.set(i.id,{x:i.x,y:i.y});return t}let b=n*0.22,c=r*0.5,f=e.filter((i)=>i.kind==="agent"),p=e.filter((i)=>i.kind!=="agent");f.forEach((i,u)=>{let s=Math.max(1,f.length),m=c+(u-(s-1)/2)*80;t.set(i.id,{x:b,y:m})});let d=new Map;for(let i of p){let u=d.get(i.kind)??[];u.push(i),d.set(i.kind,u)}let l=Array.from(d.keys());return l.forEach((i,u)=>{let s=d.get(i),m=n*(0.5+(u+1)/(l.length+1)*0.45);s.forEach((h,v)=>{let y=r*(v+1)/(s.length+1);t.set(h.id,{x:m,y})})}),t}function Ha(e,n){let r=Math.max(40,Math.abs(n.x-e.x)*0.4);return`M ${e.x} ${e.y} C ${e.x+r} ${e.y}, ${n.x-r} ${n.y}, ${n.x} ${n.y}`}function Sa(e,n){let r=e.querySelectorAll('template[slot="node"]');for(let o of Array.from(r))if(o.dataset.kind===n)return o;return null}function Fa(e,n){let r=Sa(e,n.kind);if(!r){let o=n.sub?`<div class="node-sub">${et(n.sub)}</div>`:"";return`<div class="node-h"><span class="tag">${et(n.kind)}</span></div><div class="node-title">${et(n.title??n.id)}</div>${o}`}return r.innerHTML.replace(/\$\{title\}/g,et(n.title??"")).replace(/\$\{sub\}/g,et(n.sub??"")).replace(/\$\{id\}/g,et(n.id))}function Ba(e,n,r,o,t,a,b){let c=so(n,[]),f=so(r,[]),p=Na(c,o,t,a),d=f.map((i)=>{let u=p.get(i.from),s=p.get(i.to);if(!u||!s)return"";let m=et(Ha(u,s)),h=i.dashed?" data-dashed":"";return`<path d="${m}" data-kind="${et(i.kind??"default")}"${h}></path>`}).join(""),l=c.map((i)=>{let u=p.get(i.id);if(!u)return"";let s=u.x/o*100,m=u.y/t*100,h=b&&b===i.id?" data-selected":"",v=i.w?`--node-w: ${i.w}px;`:"",y=et(`${i.kind}: ${i.title??i.id}`),w=Fa(e,i);return`<div class="node" part="node" data-kind="${et(i.kind)}" data-id="${et(i.id)}"${h} tabindex="0" role="button" aria-label="${y}" style="left: ${s}%; top: ${m}%;${v}">${w}</div>`}).join("");return`<svg class="edges" part="edges" viewBox="0 0 ${o} ${t}" preserveAspectRatio="none" aria-hidden="true">${d}</svg>`+l}var Ae=O({mount(e,n){let r=e,o=r.canvasEl();if(!o)return;let t=o.ownerDocument.createElement("div");o.appendChild(t);let a=nt(t,()=>{let d=r.nodes(),l=r.edges(),i=r.width()||1000,u=r.height()||600,s=r.layout()||"positioned",m=r.selectedId();return Ba(n,d,l,i,u,s,m)}),b="",c=(d)=>{let l=d.target.closest(".node[data-id]");if(!l)return;let i=l.getAttribute("data-id")??"",u=l.getAttribute("data-kind")??"";n.setAttribute("selected-id",i),n.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:i,kind:u},bubbles:!0,composed:!0}))},f=(d)=>{if(d.key!=="Enter"&&d.key!==" ")return;let l=d.target.closest(".node[data-id]");if(!l)return;d.preventDefault();let i=l.getAttribute("data-id")??"",u=l.getAttribute("data-kind")??"";n.setAttribute("selected-id",i),n.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:i,kind:u},bubbles:!0,composed:!0}))},p=(d)=>{let l=d.target.closest(".node[data-id]");if(!l){b="";return}let i=l.getAttribute("data-id")??"";if(i===b)return;b=i;let u=l.getAttribute("data-kind")??"";n.dispatchEvent(new CustomEvent("sui-node-hover",{detail:{id:i,kind:u},bubbles:!0,composed:!0}))};return o.addEventListener("click",c),o.addEventListener("keydown",f),o.addEventListener("mouseover",p),()=>{a(),o.removeEventListener("click",c),o.removeEventListener("keydown",f),o.removeEventListener("mouseover",p)}}});var lg=j("sui-graph-canvas",{props:{nodes:g.string({default:""}),edges:g.string({default:""}),width:g.number({default:1000,reflect:!0}),height:g.number({default:600,reflect:!0}),layout:g.union(["positioned","radial"],{default:"positioned",reflect:!0}),"selected-id":g.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    position: relative;
    width: 100%;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    overflow: hidden;
    /* Component knobs — consumers may override on the host. */
    --graph-aspect: 5 / 3;
    --node-w: 180px;
    --edge-default: var(--border);
    --edge-agent: var(--tone-primary-fg, var(--foreground));
    --edge-skill: var(--tone-success-fg);
    --edge-mcp: var(--tone-info-fg);
    --edge-rule: var(--tone-warning-fg);
    --edge-perm: var(--tone-destructive-fg);
    --edge-provider: var(--tone-accent-fg);
  }

  .canvas {
    position: relative;
    width: 100%;
    aspect-ratio: var(--graph-aspect, 5 / 3);
  }

  svg.edges {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
  }
  svg.edges path {
    fill: none;
    stroke: var(--edge-default);
    stroke-width: 1.5;
    opacity: 0.85;
  }
  svg.edges path[data-kind='agent']    { stroke: var(--edge-agent); }
  svg.edges path[data-kind='skill']    { stroke: var(--edge-skill); }
  svg.edges path[data-kind='mcp']      { stroke: var(--edge-mcp); }
  svg.edges path[data-kind='rule']     { stroke: var(--edge-rule); }
  svg.edges path[data-kind='perm']     { stroke: var(--edge-perm); }
  svg.edges path[data-kind='provider'] { stroke: var(--edge-provider); }
  svg.edges path[data-dashed] { stroke-dasharray: 4 3; }

  .node {
    position: absolute;
    transform: translate(-50%, -50%);
    width: var(--node-w, 180px);
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: var(--space-2) var(--space-3);
    box-shadow: var(--shadow-xs);
    cursor: pointer;
    color: var(--foreground);
    font-family: var(--font-body);
    font-size: var(--text-sm);
    transition: box-shadow var(--duration-fast) var(--ease-out),
                border-color var(--duration-fast) var(--ease-out);
  }
  .node:hover { box-shadow: var(--shadow-sm); }
  .node:focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: 2px;
  }
  .node[data-selected] {
    border-color: var(--ring);
    box-shadow: var(--shadow-sm);
  }
  .node[data-kind='agent']    { border-left: 3px solid var(--edge-agent); }
  .node[data-kind='skill']    { border-left: 3px solid var(--edge-skill); }
  .node[data-kind='mcp']      { border-left: 3px solid var(--edge-mcp); }
  .node[data-kind='rule']     { border-left: 3px solid var(--edge-rule); }
  .node[data-kind='perm']     { border-left: 3px solid var(--edge-perm); }
  .node[data-kind='provider'] { border-left: 3px solid var(--edge-provider); }

  .node-h {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-1);
  }
  .tag {
    font-size: var(--text-xs);
    font-weight: var(--font-weight-semibold);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--muted-foreground);
  }
  .node-title {
    font-weight: var(--font-weight-semibold);
    color: var(--foreground);
  }
  .node-sub {
    font-size: var(--text-xs);
    color: var(--muted-foreground);
  }

  ::slotted(template) { display: none; }

  @media (prefers-reduced-motion: reduce) {
    .node { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.nodes=n.nodes,t.edges=n.edges,t.width=n.width,t.height=n.height,t.layout=n.layout,t.selectedId=n.selectedId;let a=null;if(Ae&&typeof Ae.setup==="function"){if(e.scope=t,a=Ae.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","canvas"),c.setAttribute("part","canvas"),T(t,"canvasEl",c),b.appendChild(c);let f=document.createElement("slot");if(f.setAttribute("name","node"),f.setAttribute("hidden",""),b.appendChild(f),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});
