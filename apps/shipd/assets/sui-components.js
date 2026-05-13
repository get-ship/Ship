var Ne=Symbol.for("sui:error-boundary");function He(e,n){e[Ne]=n}function Se(e){delete e[Ne]}function ft(e){let n=e;while(n){if(n instanceof Element){let t=n[Ne];if(t)return t}let r=n.parentNode;if(r){n=r;continue}let o=n;if(o.host){n=o.host;continue}n=null}return null}function yt(e,n,r){try{return r()}catch(o){let t=ft(e);if(t){t.catch(o,n);return}console.error("[sui] uncaught throw (no error boundary):",o);return}}var st=null,ct=null,wo=0,Ao=new Set,Ot=[];function Fe(e,n){Ot.push(e);try{return n()}finally{Ot.pop()}}function ko(){return Ot.length>0?Ot[Ot.length-1]:null}function mr(e){if(st===null)return;if(e.has(st))return;let n=st;if(e.add(n),ct!==null)ct.push(()=>e.delete(n))}function N(e){let n=e,r=new Set,o=()=>{return mr(r),n};return o.set=(t)=>{let a=typeof t==="function"?t(n):t;if(Object.is(a,n))return;if(n=a,wo>0)for(let b of r)Ao.add(b);else for(let b of[...r])b()},o.__subs=r,o.__peek=()=>n,o}function Z(e){let n,r=!0,o=new Set,t=[],a=()=>{if(r)return;r=!0;for(let m of[...o])m()},b=()=>{for(let d of t)d();t=[];let m=st,p=ct;st=a,ct=t;try{n=e()}finally{st=m,ct=p}r=!1},c=()=>{if(mr(o),r)b();return n};return c.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},c.__subs=o,c.__peek=()=>{if(r)b();return n},c}function D(e){let n=void 0,r=!0,o=[],t=ko(),a=()=>{if(!r)return;if(n){try{n()}catch(m){console.error("[sui] effect cleanup threw:",m)}n=void 0}for(let m of o)m();o=[];let b=st,c=ct;st=a,ct=o;try{n=e()}catch(m){let p=t?ft(t):null;if(p)p.catch(m,"effect");else console.error("[sui] effect threw:",m)}finally{st=b,ct=c}};return a(),()=>{r=!1;for(let b of o)b();if(o=[],n){try{n()}catch(b){console.error("[sui] effect cleanup threw:",b)}n=void 0}}}var Rt=null;function Tt(e,n){let r=Rt;Rt=e;try{return n()}finally{Rt=r}}function ut(e){let n=Rt;if(!n)return;n.cleanup(e)}function Pt(){let e=[],n=!1;return{cleanup(o){if(n){try{o()}catch(t){console.error("[sui/scope] late cleanup threw:",t)}return}e.push(o)},dispose(){if(n)return;n=!0;for(let o=e.length-1;o>=0;o--)try{e[o]()}catch(t){console.error("[sui/scope] cleanup threw:",t)}e.length=0},get disposed(){return n}}}var Eo=Symbol("sui.shared");function br(e){return!!e?.[Eo]}function Be(e,n,r){e[`__sui_ctx_${n}`]=r}function Ge(e){let n={};for(let r in e)if(Object.prototype.hasOwnProperty.call(e,r)&&br(e[r]))n[r]=e[r];return n}function J(e=null){return N(null)}function T(e,n,r){let o=e[n];if(typeof o==="function"&&typeof o.set==="function")return o.set(r),o;let t=N(r);return e[n]=t,t}function nt(e,n,r){let o=()=>{if(e.assignedNodes({flatten:!0}).some(Co))n.removeAttribute(r);else n.setAttribute(r,"")};return e.addEventListener("slotchange",o),o(),()=>{e.removeEventListener("slotchange",o)}}function Co(e){if(e.nodeType===Node.TEXT_NODE)return(e.textContent??"").trim().length>0;if(e.nodeType===Node.ELEMENT_NODE){let n=e;return(n.textContent??"").trim().length>0||n.childElementCount>0}return!1}function hr(e,n,r={}){vr(e,n,r)}function vr(e,n,r){if(r.skip?.(e))return;if(e.tagName!==n.tagName){e.replaceWith(n.cloneNode(!0));return}jo(e,n),zo(e,n,r)}function jo(e,n){let r=Array.from(e.attributes);for(let t of r)if(!n.hasAttribute(t.name))e.removeAttribute(t.name);let o=Array.from(n.attributes);for(let t of o)if(e.getAttribute(t.name)!==t.value)e.setAttribute(t.name,t.value);if(e instanceof HTMLInputElement&&n instanceof HTMLInputElement){if(n.hasAttribute("value")&&e.value!==n.value)e.value=n.value;if(e.checked!==n.checked)e.checked=n.checked}}function zo(e,n,r){let o=Array.from(e.childNodes),t=Array.from(n.childNodes),a=Math.max(o.length,t.length);for(let b=0;b<a;b++){let c=o[b],m=t[b];if(!m&&c){e.removeChild(c);continue}if(!c&&m){e.appendChild(m.cloneNode(!0));continue}if(!c||!m)continue;Lo(c,m,e,r)}}function Lo(e,n,r,o){if(e.nodeType!==n.nodeType){r.replaceChild(n.cloneNode(!0),e);return}if(e.nodeType===Node.TEXT_NODE||e.nodeType===Node.COMMENT_NODE){if(e.nodeValue!==n.nodeValue)e.nodeValue=n.nodeValue;return}if(e.nodeType===Node.ELEMENT_NODE)vr(e,n,o)}function B(e,n){let r;return D(()=>{let o=n(),t=o==null?"":String(o);if(t===r)return;if(r===void 0&&e.textContent===t){r=t;return}e.textContent=t,r=t})}function E(e,n,r){let o,t=!1;return D(()=>{let a=r();if(t&&a===o)return;let b=a==null||a===!1||a===""?null:a===!0?"":String(a);if(t||e.getAttribute(n)!==b)if(b===null)e.removeAttribute(n);else e.setAttribute(n,b);o=a,t=!0})}function pt(e,n,r){let o;return D(()=>{let t=!!r();if(t===o)return;if(o===void 0&&t===e.classList.contains(n)){o=t;return}if(t)e.classList.add(n);else e.classList.remove(n);o=t})}function tt(e,n,r){return D(()=>{e[n]=r()})}function mt(e,n,r){let o=D(()=>{let a=n();if(e.value!==a)e.value=a}),t=()=>{let a=e.value;if(n.set(a),r)r(a)};return e.addEventListener("input",t),()=>{o(),e.removeEventListener("input",t)}}function bt(e,n){let r=D(()=>{let t=n();if(e.checked!==t)e.checked=t}),o=()=>n.set(e.checked);return e.addEventListener("change",o),()=>{r(),e.removeEventListener("change",o)}}function et(e,n){let o=e.namespaceURI==="http://www.w3.org/2000/svg",t=e.tagName.toLowerCase();return D(()=>{let a=n(),b=o?e.ownerDocument.createElementNS("http://www.w3.org/2000/svg",t):e.ownerDocument.createElement(t);b.innerHTML=a,hr(e,b)})}function K(e,n,r,o={}){let t=o.modifiers??[],a,b=!0,c=(p)=>{for(let i of t)switch(i.kind){case"preventDefault":p.preventDefault();break;case"stopPropagation":p.stopPropagation();break;case"stopImmediatePropagation":p.stopImmediatePropagation();break;case"self":if(p.target!==e)return;break;case"trusted":if(!p.isTrusted)return;break;case"key":{if(!gr(p,i.arg??""))return;break}}let d=t.find((i)=>i.kind==="debounce"),l=t.find((i)=>i.kind==="throttle");if(d){let i=Number(d.arg)||0;if(a!==void 0)clearTimeout(a);a=setTimeout(()=>{a=void 0,r(p)},i);return}if(l){let i=Number(l.arg)||0;if(!b)return;b=!1,r(p),a=setTimeout(()=>{b=!0},i);return}r(p)},m={};if(o.capture)m.capture=!0;if(o.passive)m.passive=!0;if(o.once)m.once=!0;return e.addEventListener(n,c,m),()=>{if(e.removeEventListener(n,c,m),a!==void 0)clearTimeout(a)}}function gr(e,n){let r=n.split("+").map((d)=>d.toLowerCase()),o=r.pop()??"",t=r.includes("meta")||r.includes("cmd"),a=r.includes("ctrl")||r.includes("control"),b=r.includes("shift"),c=r.includes("alt")||r.includes("option"),m=r.includes("mod"),p=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(t&&!e.metaKey)return!1;if(a&&!e.ctrlKey)return!1;if(b&&!e.shiftKey)return!1;if(c&&!e.altKey)return!1;if(m&&!(p?e.metaKey:e.ctrlKey))return!1;return e.key.toLowerCase()===o}function yr(e,n,r){e.dispatchEvent(new CustomEvent(n,{detail:r,bubbles:!0,composed:!0}))}function xr(e,n){let r=e,o=Object.keys(n);for(let t of o)r[t]=n[t];ut(()=>{for(let t of o)if(r[t]===n[t])try{delete r[t]}catch{r[t]=void 0}})}function Jt(e){let n=e;n.emit=function(r,o){yr(e,r,o)},n.expose=function(r){xr(e,r)}}function Q(e,n,r,o){e.addEventListener(n,r,o),ut(()=>e.removeEventListener(n,r,o))}function Ue(e,n){let r=D(()=>{let o=e();if(!o)return;let t=()=>{n(o.assignedElements())};return o.addEventListener("slotchange",t),t(),()=>o.removeEventListener("slotchange",t)});ut(r)}var Xe=Symbol("sui:transition");function Yt(e){return e[Xe]}function Do(e){let n=e,r=n[Xe];if(!r)r={inOpts:{},outOpts:{}},n[Xe]=r;return r}function ot(e,n,r={},o="both"){let t=Do(e);if(o==="in"||o==="both")t.inT=n,t.inOpts=r;if(o==="out"||o==="both")t.outT=n,t.outOpts=r;if(!e.hasAttribute("data-sui-transition"))e.setAttribute("data-sui-transition","")}function Oo(){if(typeof window>"u"||typeof window.matchMedia!=="function")return!1;try{return window.matchMedia("(prefers-reduced-motion: reduce)").matches}catch{return!1}}function kr(e){if(!Oo())return e;return{...e,duration:0,delay:0}}function Er(e){let n=Yt(e);if(!n?.inT?.in)return;n.current?.cancel();let r=n.inT.in(e,kr(n.inOpts));return n.current=r,r}function Cr(e){let n=Yt(e);if(!n?.outT?.out)return;n.current?.cancel();let r=n.outT.out(e,kr(n.outOpts));return n.current=r,r}function qe(e){if(e.nodeType!==1)return;let n=e;if(n.hasAttribute("data-sui-transition")){let r=Yt(n);if(r?.current?.cancel(),r)r.current=void 0}for(let r of n.querySelectorAll("[data-sui-transition]")){let o=Yt(r);if(o?.current?.cancel(),o)o.current=void 0}}function jr(e,n){for(let r of e){if(r.nodeType!==1)continue;let o=r;if(o.hasAttribute("data-sui-transition"))n(o);for(let t of o.querySelectorAll("[data-sui-transition]"))n(t)}}function At(e){jr(e,(n)=>{Er(n)})}function Mt(e){let n=[];if(jr(e,(r)=>{let o=Cr(r);if(o)n.push(o.finished.catch(()=>{}))}),n.length===0)return null;return Promise.all(n).then(()=>{return})}var To=200,Po="cubic-bezier(0.4, 0, 0.2, 1)",Mo=0;function xt(e){return{duration:typeof e.duration==="number"?e.duration:To,easing:typeof e.easing==="string"?e.easing:Po,delay:typeof e.delay==="number"?e.delay:Mo,fill:"both"}}function wt(e,n,r){let o=e.animate;if(typeof o!=="function")return;return o.call(e,n,r)}var ht={in:(e,n)=>wt(e,[{opacity:0},{opacity:1}],xt(n)),out:(e,n)=>wt(e,[{opacity:1},{opacity:0}],xt(n))};function wr(e,n){let r=e==="y"?"translateY(-100%)":"translateX(-100%)",o="translate(0, 0)";return n==="in"?[{opacity:0,transform:r},{opacity:1,transform:"translate(0, 0)"}]:[{opacity:1,transform:"translate(0, 0)"},{opacity:0,transform:r}]}var Re={in:(e,n)=>{let r=n.axis==="x"?"x":"y";return wt(e,wr(r,"in"),xt(n))},out:(e,n)=>{let r=n.axis==="x"?"x":"y";return wt(e,wr(r,"out"),xt(n))}};function Ar(e,n){let r={opacity:0,transform:`scale(${e})`},o={opacity:1,transform:"scale(1)"};return n==="in"?[r,o]:[o,r]}var Je={in:(e,n)=>{let r=typeof n.from==="number"?n.from:0;return wt(e,Ar(r,"in"),xt(n))},out:(e,n)=>{let r=typeof n.from==="number"?n.from:0;return wt(e,Ar(r,"out"),xt(n))}};function No(e,n,r,o,t){if(r.fill(0,0,n),n===0)return;let a=0;for(let c=0;c<n;c++){let m=e[2*c+1],p=0,d=a;while(p<d){let l=p+d>>>1;if(e[2*t[l]+1]<m)p=l+1;else d=l}if(o[c]=p>0?t[p-1]:-1,t[p]=c,p===a)a++}let b=t[a-1];while(b>=0)r[b]=1,b=o[b]}function R(e){let{anchor:n,items:r,key:o,render:t}=e,a=[],b=!0,c=new Set,m=new Map,p=new Map,d=new Map,l=new Set,i=[],u=[],s=new Int32Array(0),f=new Uint8Array(0),h=new Int32Array(0),v=new Int32Array(0),g=(k)=>{for(let w of k.nodes)try{w.parentNode?.removeChild(w)}catch{}k.cleanup?.()},x=D(()=>{if(!b)return;let k=r(),w=k.length;if(s.length<w*2)s=new Int32Array(Math.max(s.length*2,w*2,16));if(f.length<w){let O=Math.max(f.length*2,w,8);f=new Uint8Array(O),h=new Int32Array(O),v=new Int32Array(O)}for(let O=0;O<w;O++){let H=k[O],S=o(H,O);if(m.has(S))console.warn("[sui] {#for}: duplicate key",S,"at index",O,"(prior at",m.get(S),"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");m.set(S,O)}for(let O of a)p.set(O.key,O);for(let O of a){if(m.has(O.key))continue;let H=Mt(O.nodes);if(H===null){g(O);continue}c.add(O),H.then(()=>{if(!c.delete(O))return;g(O)})}u.length=0,i.length=0;for(let O=0;O<w;O++){let H=k[O],S=o(H,O),U=p.get(S),G;if(U)U.item=H,yt(n,"render",()=>U.update?.(H,O)),G=U;else{let X=yt(n,"render",()=>t(H,O));if(!X)continue;G={key:S,item:H,nodes:X.nodes,...X.cleanup&&{cleanup:X.cleanup},...X.update&&{update:X.update}},i.push(...X.nodes)}u.push(G)}for(let O=0;O<a.length;O++)d.set(a[O].key,O);let A=0;for(let O=0;O<u.length;O++){let H=d.get(u[O].key);if(H!==void 0)s[2*A]=O,s[2*A+1]=H,A++}No(s,A,f,h,v);for(let O=0;O<A;O++)if(f[O])l.add(s[2*O]);let j=n.parentNode,C=n;for(let O=0;O<u.length;O++){if(!j)break;let H=u[O];if(l.has(O))C=H.nodes[H.nodes.length-1]??C;else for(let S of H.nodes)j.insertBefore(S,C.nextSibling),C=S}let M=a;if(a=u,u=M,i.length>0)At(i);m.clear(),p.clear(),d.clear(),l.clear(),i.length=0});return()=>{b=!1,x();for(let k of a)g(k);for(let k of c)g(k);c.clear(),a=[],u.length=0}}function F(e){let{anchor:n}=e,{test:r,branches:o}=Ho(e)?{test:e.test,branches:e.branches}:So(e),t=null,a=-2,b=null,c=(p)=>{for(let d of p)try{d.parentNode?.removeChild(d)}catch{}},m=D(()=>{let p=r();if(t&&p===a)return;if(b&&p===b.idx&&p>=0&&p<o.length){let i=b;b=null;for(let u of i.nodes)qe(u);t={nodes:i.nodes,...i.cleanup&&{cleanup:i.cleanup}},a=p,At(t.nodes);return}if(t){let i={idx:a,nodes:t.nodes,...t.cleanup&&{cleanup:t.cleanup}},u=Mt(i.nodes);if(t=null,u===null)c(i.nodes),i.cleanup?.();else b=i,u.then(()=>{if(b!==i)return;c(i.nodes),i.cleanup?.(),b=null})}if(a=p,p<0||p>=o.length)return;let d=yt(n,"render",()=>o[p]());if(!d)return;t=d;let l=n.parentNode;if(!l)return;for(let i of t.nodes)l.insertBefore(i,n);At(t.nodes)});return()=>{if(m(),t)c(t.nodes),t.cleanup?.(),t=null;if(b)c(b.nodes),b.cleanup?.(),b=null}}function Ho(e){return typeof e.test==="function"}function So(e){let{cond:n,then_:r,else_:o}=e;return{test:()=>n()?0:o?1:-1,branches:o?[r,o]:[r]}}var Ye=(e)=>e??"",We=(e)=>{if(e===null||e==="")return NaN;return Number(e)},Ze=(e)=>{if(e===null)return!1;if(e.toLowerCase()==="false")return!1;return!0};function Wt(e){return(n)=>{if(n===null||n==="")return e;try{return JSON.parse(n)}catch{return e}}}function Qe(e,n){let r=new Set(e);return(o)=>o!==null&&r.has(o)?o:n}var Zt=(e)=>{switch(e.type){case"string":return Ye;case"bool":return Ze;case"number":return We;case"union":{let n=e.values??[],r=e.default??n[0]??"";return Qe(n,r)}case"object":return Wt(e.default);case"array":return Wt(e.default??[])}};function Lr(e,n,r,o){if(!n)return;let t={},a,b=r.formValue;if(b!==void 0){let{sig:m,propDef:p}=zr(r,o,b,"formValue"),d=p.default,l=Zt(p);a=D(()=>{let i=m();n.setFormValue(Fo(i))}),t.reset=()=>{m.set(d)},t.restore=(i)=>{m.set(l(i))}}let c=r.formDisabled;if(c!==void 0){let{sig:m,propDef:p}=zr(r,o,c,"formDisabled");if(p.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${c}" must be a bool-typed prop`);t.disabled=(d)=>{m.set(!!d)}}if(e.__shipSetFormHooks=t,a)e.__shipSetFormValueCleanup=a;return a}function Dr(e){let n=e;delete n.__shipSetFormHooks,delete n.__shipSetFormValueCleanup}function Fo(e){if(e===null||e===void 0)return"";if(typeof e==="boolean")return e?"on":"";return String(e)}function zr(e,n,r,o){let t=e.props[r];if(!t)throw Error(`[sui-runtime] ${o}: "${r}" — no matching prop on this component`);let a=n[kt(r)];if(!a)throw Error(`[sui-runtime] ${o}: "${r}" — prop signal missing`);return{sig:a,propDef:t}}var y={string(e={}){return{type:"string",...e}},bool(e={}){return{type:"bool",...e}},number(e={}){return{type:"number",...e}},union(e,n={}){return{type:"union",values:e,...n}},object(e={}){return{type:"object",...e}},array(e={}){return{type:"array",...e}}},Bo=(e,n)=>{if(n===void 0||n===null)return null;switch(e.type){case"string":return n===""?null:String(n);case"number":return Number.isNaN(n)?null:String(n);case"bool":return n?"":null;case"union":return String(n);case"object":case"array":try{return JSON.stringify(n)}catch{return null}}},Go=(e,n)=>{let r=`ship-set-style-${e}`;if(document.getElementById(r))return;let o=document.createElement("style");o.id=r,o.textContent=`[data-component="${e}"] { ${n} }`,document.head.appendChild(o)},Uo=(e,n,r,o)=>{let t=e.getAttribute(n),a=t!==null?o(t):r.default;return N(a)},Xo=(e)=>{let n=[],r={};for(let[a,b]of Object.entries(e.props??{}))n.push(a),r[a]={camel:kt(a),decode:Zt(b)};let o=e.formAssociated===!0||e.formValue!==void 0||e.formDisabled!==void 0,t={observedAttrs:n,props:r};if(o)t.formAssociated=!0;return t};function z(e,n){let r=Xo(n),o=Object.entries(n.props??{});Ve(e,r,(t,a,b)=>{let c=null;if(n.shadow){if(c=t.shadowRoot??t.attachShadow({mode:"open"}),n.styles){let p=document.createElement("style");p.textContent=n.styles,c.appendChild(p)}}else if(n.styles)Go(e,n.styles);let m=[];for(let[p,d]of o){let l=kt(p),i=r.props[p].decode,u=Uo(t,p,d,i);if(a[l]=u,d.reflect){let s=!0,f=D(()=>{let h=u();if(s){s=!1;return}let v=Bo(d,h);if(v===null)t.removeAttribute(p);else t.setAttribute(p,v)});m.push(f)}}if(n.formValue!==void 0||n.formDisabled!==void 0){let p=Lr(t,b,n,a);if(typeof p==="function")m.push(p)}if(n.setup){let p={props:a,host:t,cleanups:m};if(b)p.internals=b;n.setup(p)}return()=>{Dr(t);for(let p of m.splice(0))try{p()}catch(d){console.error(`[ship-set:${e}] cleanup threw:`,d)}}})}var Or=new Set;function Ve(e,n,r){if(customElements.get(e)){if(!Or.has(e))Or.add(e),console.warn(`[sui-runtime] customElements.define("${e}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(n.formAssociated){class t extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return n.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=e,Jt(this),this.scope=Pt();try{this.cleanup=Tt(this.scope,()=>Fe(this,()=>r(this,this.propSignals,this.internals)))}catch(a){let b=ft(this.parentNode);if(b)b.catch(a,"mount");else console.error(`[ship-set:${e}] setup threw:`,a)}}disconnectedCallback(){try{this.cleanup?.()}catch(a){console.error(`[ship-set:${e}] cleanup threw:`,a)}this.cleanup=void 0;try{this.scope?.dispose()}catch(a){console.error(`[ship-set:${e}] scope dispose threw:`,a)}this.scope=void 0}attributeChangedCallback(a,b,c){let m=n.props[a];if(!m)return;let p=this.propSignals[m.camel];if(p)p.set(m.decode(c))}formResetCallback(){let a=this.__shipSetFormHooks;try{a?.reset?.()}catch(b){console.error(`[ship-set:${e}] formResetCallback threw:`,b)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(a,b){let c=this.__shipSetFormHooks;try{c?.restore?.(a)}catch(m){console.error(`[ship-set:${e}] formStateRestoreCallback threw:`,m)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:a,mode:b}}))}formDisabledCallback(a){let b=this.__shipSetFormHooks;try{b?.disabled?.(a)}catch(c){console.error(`[ship-set:${e}] formDisabledCallback threw:`,c)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:a}}))}}customElements.define(e,t);return}class o extends HTMLElement{static get observedAttributes(){return n.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=e,Jt(this),this.scope=Pt();try{this.cleanup=Tt(this.scope,()=>Fe(this,()=>r(this,this.propSignals)))}catch(t){let a=ft(this.parentNode);if(a)a.catch(t,"mount");else console.error(`[ship-set:${e}] setup threw:`,t)}}disconnectedCallback(){try{this.cleanup?.()}catch(t){console.error(`[ship-set:${e}] cleanup threw:`,t)}this.cleanup=void 0;try{this.scope?.dispose()}catch(t){console.error(`[ship-set:${e}] scope dispose threw:`,t)}this.scope=void 0}attributeChangedCallback(t,a,b){let c=n.props[t];if(!c)return;let m=this.propSignals[c.camel];if(m)m.set(c.decode(b))}}customElements.define(e,o)}function kt(e){return e.replace(/-([a-z])/g,(n,r)=>r.toUpperCase())}function Ke(e,n,r){let o=Jo(e.state,{ship:n.ship,...r}),t={};for(let[i,u]of Object.entries(o))t[i]=N(u);let a={ship:n.ship,route:n.route??{},navigate:n.navigate??((i)=>{}),...r,...t},b=Yo(t,a),c={...a},m={};for(let[i,u]of Object.entries(e.derived??{}))m[i]=Z(()=>u(c));Object.assign(c,m);let p={};for(let[i,u]of Object.entries(e.on??{}))p[i]=(...s)=>u(b,...s);let d={},l=typeof e.use==="function"?e.use(c):e.use??{};for(let[i,u]of Object.entries(l))d[i]=Ke(u,n,c);return{...t,...m,...p,...d}}function Jo(e,n){if(e===void 0)return{};if(typeof e==="function")return e(n);return e}function Yo(e,n){return new Proxy({},{get(r,o){if(o in e)return e[o]();return n[o]},set(r,o,t){if(o in e)return e[o].set(t),!0;return console.warn(`[ship-set] hook handler tried to write s.${o} = … but ${o} is not a hook state key. The assignment was ignored.`),!0},has(r,o){return o in e||o in n},ownKeys(){return Object.keys(e)},getOwnPropertyDescriptor(r,o){if(o in e)return{configurable:!0,enumerable:!0,writable:!0,value:e[o]()};return}})}var Tr="__shipSetLifecycle";function Pr(e,n){let r=Wo(e,n),o={ship:n.ship,route:n.route??{},navigate:n.navigate??((i)=>{}),...n.scope??{}},t=r.scope?r.scope(o,n.host):{};Object.assign(o,t);let a=Zo(o,n.stateKeys,t),b=Qo(a,o),c={...o},m=Vo(r,c);Object.assign(c,m);let p=Io(r,c,n);for(let[i,u]of Object.entries(p))c[i]=u;let d=Ko(r,b),l={...t,...m,...d,...p};if(Object.keys(t).length>0&&n.host){let i=Ge(t);if(Object.keys(i).length>0){let u=n.host,s=u.tagName.toLowerCase();Be(u,s,i)}}if(r.mount||r.unmount){let i={state:b};if(r.mount)i.mount=r.mount;if(r.unmount)i.unmount=r.unmount;Object.defineProperty(l,Tr,{value:i,enumerable:!0,configurable:!0,writable:!1})}return l}function Wo(e,n){if(typeof e==="function")return e({ship:n.ship});return e}function Zo(e,n,r){let o={},t=n!==void 0,a=r?Object.keys(r):[];if(t||a.length>0){if(n)for(let b of n){let c=e[b];if(Ie(c))o[b]=c}for(let b of a){if(b in o)continue;let c=e[b];if(Ie(c))o[b]=c}return o}for(let[b,c]of Object.entries(e))if(Ie(c))o[b]=c;return o}function Ie(e){return typeof e==="function"&&"set"in e&&typeof e.set==="function"}function Qo(e,n){return new Proxy({},{get(r,o){if(o in e)return e[o]();return n[o]},set(r,o,t){if(o in e){if(typeof t==="function")e[o].set(()=>t);else e[o].set(t);return!0}return console.warn(`[ship-set] handler tried to write s.${o} = … but ${o} is not declared as <state name="${o}"> in the template. The assignment was ignored.`),!0},has(r,o){return o in e||o in n},ownKeys(){return Object.keys(e)},getOwnPropertyDescriptor(r,o){if(o in e)return{configurable:!0,enumerable:!0,writable:!0,value:e[o]()};return}})}function Vo(e,n){let r=e.derived??{},o={};for(let[t,a]of Object.entries(r))o[t]=Z(()=>a(n));return o}function Ko(e,n){let r=e.on??{},o={};for(let[t,a]of Object.entries(r))o[t]=(...b)=>a(n,...b);return o}function Io(e,n,r){if(e.use===void 0)return{};let o=typeof e.use==="function"?e.use(n):e.use,t={};for(let[a,b]of Object.entries(o))t[a]=Ke(b,r,n);return t}function P(e){return{setup:(n)=>Pr(e,n),__def:e}}if(typeof customElements<"u"){for(let e of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(e))customElements.define(e,class extends HTMLElement{})}var na=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,ea=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function Mr(e){if(typeof CSSStyleSheet>"u")return null;try{let n=new CSSStyleSheet;return n.replaceSync(e),n}catch{return null}}var Nr=Mr(na),Hr=Mr(ea);function L(e){if(typeof ShadowRoot>"u")return;if(!(e instanceof ShadowRoot))return;let n=[Hr,Nr].filter((r)=>r!=null);if(n.length===0)return;try{e.adoptedStyleSheets=[...n,...e.adoptedStyleSheets??[]]}catch{}}var Du=z("sui-status-dot",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"success",reflect:!0}),pulse:y.bool({default:!1,reflect:!0}),size:y.union(["sm","md"],{default:"md",reflect:!0}),label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.pulse=n.pulse,t.size=n.size,t.label=n.label;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");return b.setAttribute("class","dot"),b.setAttribute("part","dot"),o.push(E(b,"role",()=>t.label()?"img":null)),o.push(E(b,"aria-label",()=>t.label()||null)),o.push(E(b,"aria-hidden",()=>t.label()?null:"true")),a.appendChild(b),t}});var Pu=z("sui-spinner",{props:{size:y.union(["xs","sm","md","lg"],{default:"md",reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),label:y.string({default:"Loading"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.size=n.size,t.tone=n.tone,t.label=n.label;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");return b.setAttribute("class","ring"),b.setAttribute("part","ring"),o.push(E(b,"role",()=>t.label()?"status":null)),o.push(E(b,"aria-label",()=>t.label()||null)),o.push(E(b,"aria-hidden",()=>t.label()?null:"true")),a.appendChild(b),t}});var Hu=z("sui-badge",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["solid","soft","outline"],{default:"soft",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),pill:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.pill=n.pill;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");b.setAttribute("class","badge"),b.setAttribute("part","badge");let c=document.createElement("span");c.setAttribute("class","leading"),c.setAttribute("part","leading");let m=document.createElement("slot");m.setAttribute("name","leading"),c.appendChild(m),b.appendChild(c);let p=document.createElement("slot");return b.appendChild(p),a.appendChild(b),t}});var Qt=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","button");if(!n.hasAttribute("tabindex")&&!r.disabled())n.setAttribute("tabindex","0");let o=D(()=>{let b=r.selected(),c=r.disabled();if(n.setAttribute("aria-pressed",b?"true":"false"),c){if(n.setAttribute("aria-disabled","true"),n.getAttribute("tabindex")!=="-1")n.setAttribute("tabindex","-1")}else if(n.removeAttribute("aria-disabled"),n.getAttribute("tabindex")==="-1")n.setAttribute("tabindex","0")}),t=(b)=>{if(r.disabled())return;if(b.key==="Enter"||b.key===" ")b.preventDefault(),n.click();else if(r.removable()&&(b.key==="Backspace"||b.key==="Delete"))b.preventDefault(),n.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};n.addEventListener("keydown",t);let a=D(()=>{let b=r.removeBtn?.();if(!b)return;let c=(m)=>{if(m.stopPropagation(),r.disabled())return;n.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};return b.addEventListener("click",c),()=>b.removeEventListener("click",c)});return()=>{o(),n.removeEventListener("keydown",t),a()}}});var Xu=z("sui-chip",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0}),variant:y.union(["solid","soft","outline","ghost"],{default:"soft",reflect:!0}),size:y.union(["sm","md","lg"],{default:"sm",reflect:!0}),selected:y.bool({default:!1,reflect:!0}),removable:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.selected=n.selected,t.removable=n.removable,t.disabled=n.disabled;let a=null;if(Qt&&typeof Qt.setup==="function"){if(e.scope=t,a=Qt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>F({anchor:u,test:()=>{if(t.removable())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("button");h.setAttribute("class","remove"),h.setAttribute("part","remove"),h.setAttribute("type","button"),h.setAttribute("aria-label","Remove"),h.setAttribute("tabindex","-1"),T(t,"removeBtn",h);let v=document.createElement("svg");v.setAttribute("viewbox","0 0 10 10"),v.setAttribute("aria-hidden","true");let g=document.createElement("path");return g.setAttribute("d","M2 2 L8 8 M8 2 L2 8"),v.appendChild(g),h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let x of f)try{x()}catch(k){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("span");m.setAttribute("class","chip"),m.setAttribute("part","chip");let p=document.createElement("span");p.setAttribute("class","leading"),p.setAttribute("part","leading");let d=document.createElement("slot");d.setAttribute("name","leading"),p.appendChild(d),m.appendChild(p);let l=document.createElement("slot");m.appendChild(l);let i=document.createComment("if");if(m.appendChild(i),o.push(b(i)),c.appendChild(m),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(f,h){return f[h]},set(f,h,v){let g=f[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(u.mount){let f=u.mount(s,r);if(typeof f==="function")o.push(f)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var Vt=P({mount(e,n){let r=e,o=(a)=>{if(a.key==="Enter"||a.key===" ")a.preventDefault(),n.click()},t=D(()=>{if(r.interactive()){if(!n.hasAttribute("role"))n.setAttribute("role","button");if(!n.hasAttribute("tabindex"))n.setAttribute("tabindex","0");return n.addEventListener("keydown",o),()=>n.removeEventListener("keydown",o)}else{if(n.getAttribute("role")==="button")n.removeAttribute("role");if(n.getAttribute("tabindex")==="0")n.removeAttribute("tabindex")}});return()=>{t()}}});var Zu=z("sui-card",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["default","outline"],{default:"default",reflect:!0}),padding:y.union(["none","sm","md","lg"],{default:"md",reflect:!0}),interactive:y.bool({default:!1,reflect:!0}),"accent-bar":y.bool({default:!1,reflect:!0}),"status-rail":y.bool({default:!1,reflect:!0}),fill:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.padding=n.padding,t.interactive=n.interactive,t.accentBar=n.accentBar,t.statusRail=n.statusRail,t.fill=n.fill;let a=null;if(Vt&&typeof Vt.setup==="function"){if(e.scope=t,a=Vt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","header"),c.setAttribute("part","header");let m=document.createElement("slot");m.setAttribute("name","header"),o.push(nt(m,r,"data-no-header")),c.appendChild(m),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","content"),p.setAttribute("part","content");let d=document.createElement("slot");p.appendChild(d),b.appendChild(p);let l=document.createElement("div");l.setAttribute("class","footer"),l.setAttribute("part","footer");let i=document.createElement("slot");if(i.setAttribute("name","footer"),o.push(nt(i,r,"data-no-footer")),l.appendChild(i),b.appendChild(l),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(f,h){return f[h]},set(f,h,v){let g=f[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(u.mount){let f=u.mount(s,r);if(typeof f==="function")o.push(f)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var Ku=z("sui-link",{props:{href:y.string({default:""}),external:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["default","underline","subtle"],{default:"default",reflect:!0}),target:y.string({default:""}),rel:y.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.href=n.href,t.external=n.external,t.tone=n.tone,t.variant=n.variant,t.target=n.target,t.rel=n.rel;let a=(d)=>F({anchor:d,test:()=>{if(t.external())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("span");u.setAttribute("class","sr-only");let s=document.createTextNode("(opens in new tab)");u.appendChild(s),l.appendChild(u);let f=document.createElement("span");f.setAttribute("class","ext"),f.setAttribute("aria-hidden","true");let h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round");let v=document.createElement("path");return v.setAttribute("d","M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3"),h.appendChild(v),f.appendChild(h),l.appendChild(f),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let g of i)try{g()}catch(x){}}}}]}),b=r.shadowRoot??r;L(b);let c=document.createElement("a");c.setAttribute("part","link"),o.push(E(c,"href",()=>t.href())),o.push(E(c,"target",()=>t.external()?"_blank":t.target())),o.push(E(c,"rel",()=>t.external()?"noopener noreferrer":t.rel()));let m=document.createElement("slot");c.appendChild(m);let p=document.createComment("if");return c.appendChild(p),o.push(a(p)),b.appendChild(c),t}});var _u=z("sui-divider",{props:{orientation:y.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:y.union(["solid","dashed"],{default:"solid",reflect:!0}),tone:y.union(["default","muted","primary","destructive"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.orientation=n.orientation,t.variant=n.variant,t.tone=n.tone;let a=r.shadowRoot??r;L(a);let b=document.createElement("hr");return b.setAttribute("class","line"),b.setAttribute("part","line"),b.setAttribute("role","separator"),o.push(E(b,"aria-orientation",()=>t.orientation())),a.appendChild(b),t}});var el=z("sui-skeleton",{props:{shape:y.union(["text","line","block","circle"],{default:"text",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.shape=n.shape,t.size=n.size;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");return b.setAttribute("class","sk"),b.setAttribute("part","sk"),b.setAttribute("role","status"),b.setAttribute("aria-label","Loading"),b.setAttribute("aria-live","polite"),b.setAttribute("aria-busy","true"),a.appendChild(b),t}});var Kt=P({mount(e,n){let r=e,o=(a)=>{if(a.key==="Enter"||a.key===" ")a.preventDefault(),n.click()},t=D(()=>{if(r.interactive()){if(!n.hasAttribute("role"))n.setAttribute("role","button");if(!n.hasAttribute("tabindex"))n.setAttribute("tabindex","0");return n.addEventListener("keydown",o),()=>n.removeEventListener("keydown",o)}else{if(n.getAttribute("role")==="button")n.removeAttribute("role");if(n.getAttribute("tabindex")==="0")n.removeAttribute("tabindex")}});return()=>{t(),n.removeEventListener("keydown",o)}}});var ul=z("sui-stat",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent"],{default:"default",reflect:!0}),interactive:y.bool({default:!1,reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.interactive=n.interactive,t.size=n.size;let a=null;if(Kt&&typeof Kt.setup==="function"){if(e.scope=t,a=Kt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("span");c.setAttribute("class","label"),c.setAttribute("part","label");let m=document.createElement("slot");m.setAttribute("name","label"),c.appendChild(m),b.appendChild(c);let p=document.createElement("span");p.setAttribute("class","value-row"),p.setAttribute("part","value-row");let d=document.createElement("span");d.setAttribute("class","value"),d.setAttribute("part","value");let l=document.createElement("slot");d.appendChild(l),p.appendChild(d);let i=document.createElement("span");i.setAttribute("class","trend"),i.setAttribute("part","trend");let u=document.createElement("slot");u.setAttribute("name","trend"),i.appendChild(u),p.appendChild(i),b.appendChild(p);let s=document.createElement("span");s.setAttribute("class","hint"),s.setAttribute("part","hint");let f=document.createElement("slot");if(f.setAttribute("name","hint"),s.appendChild(f),b.appendChild(s),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(g,x){return g[x]},set(g,x,k){let w=g[x];if(w!=null&&typeof w.set==="function"){if(typeof k==="function")w.set(()=>k);else w.set(k);return!0}return!1}});if(h.mount){let g=h.mount(v,r);if(typeof g==="function")o.push(g)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var cl=z("sui-mark",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"warning",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone;let a=r.shadowRoot??r;L(a);let b=document.createElement("mark");b.setAttribute("part","mark");let c=document.createElement("slot");return b.appendChild(c),a.appendChild(b),t}});var It=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","group");let o=D(()=>{let t=r.keys();if(t)n.setAttribute("aria-label",`Keyboard shortcut: ${t}`);else n.removeAttribute("aria-label")});return()=>o()}});var vl=z("sui-kbd",{props:{keys:y.string({default:""}),size:y.union(["sm","md"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.keys=n.keys,t.size=n.size;let a=null;if(It&&typeof It.setup==="function"){if(e.scope=t,a=It.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("kbd");c.setAttribute("part","kbd");let m=document.createElement("slot");if(c.appendChild(m),b.appendChild(c),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var xl=z("sui-avatar",{props:{src:y.string({default:""}),name:y.string({default:""}),initials:y.string({default:""}),"icon-slug":y.string({default:""}),size:y.union(["xs","sm","md","lg","xl"],{default:"md",reflect:!0}),shape:y.union(["round","square"],{default:"round",reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.src=n.src,t.name=n.name,t.initials=n.initials,t.iconSlug=n.iconSlug,t.size=n.size,t.shape=n.shape,t.tone=n.tone;let a=(p)=>F({anchor:p,test:()=>{if(t.iconSlug()||t.src())return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("img");return i.setAttribute("part","img"),l.push(E(i,"src",()=>t.iconSlug()?"https://cdn.simpleicons.org/"+t.iconSlug():t.src())),l.push(E(i,"alt",()=>t.name())),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let u of l)try{u()}catch(s){}}}},()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("span");i.setAttribute("part","initials");let u=document.createTextNode("");return i.appendChild(u),l.push(B(u,()=>t.initials()?t.initials().slice(0,2).toUpperCase():t.name().trim().slice(0,1).toUpperCase()||"?")),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let s of l)try{s()}catch(f){}}}}]}),b=r.shadowRoot??r;L(b);let c=document.createElement("span");c.setAttribute("class","root"),c.setAttribute("part","root"),c.setAttribute("role","img"),o.push(E(c,"aria-label",()=>t.name()||t.initials()||"avatar"));let m=document.createComment("if");return c.appendChild(m),o.push(a(m)),b.appendChild(c),t}});var kl=z("sui-progress",{props:{value:y.number({default:0}),max:y.number({default:100}),tone:y.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),indeterminate:y.bool({default:!1,reflect:!0}),label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.max=n.max,t.tone=n.tone,t.size=n.size,t.indeterminate=n.indeterminate,t.label=n.label;let a=r.shadowRoot??r;L(a);let b=document.createElement("div");b.setAttribute("class","track"),b.setAttribute("part","track"),b.setAttribute("role","progressbar"),o.push(E(b,"aria-valuenow",()=>t.indeterminate()?null:t.value())),b.setAttribute("aria-valuemin","0"),o.push(E(b,"aria-valuemax",()=>t.max())),o.push(E(b,"aria-busy",()=>t.indeterminate()?"true":"false")),o.push(E(b,"aria-label",()=>t.label()));let c=document.createElement("div");return c.setAttribute("class","fill"),c.setAttribute("part","fill"),o.push(E(c,"style",()=>t.indeterminate()?"":"width: clamp(0%, "+t.value()/(t.max()||1)*100+"%, 100%)")),b.appendChild(c),a.appendChild(b),t}});var jl=z("sui-progress-ring",{props:{value:y.number({default:0}),max:y.number({default:100}),size:y.number({default:48}),"track-width":y.number({default:4}),tone:y.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),indeterminate:y.bool({default:!1,reflect:!0}),label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.max=n.max,t.size=n.size,t.trackWidth=n.trackWidth,t.tone=n.tone,t.indeterminate=n.indeterminate,t.label=n.label;let a=r.shadowRoot??r;L(a);let b=document.createElement("div");b.setAttribute("class","ring"),o.push(E(b,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let c=document.createElement("svg");c.setAttribute("class","ring-svg"),o.push(E(c,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),c.setAttribute("role","progressbar"),c.setAttribute("aria-valuemin","0"),o.push(E(c,"aria-valuemax",()=>t.max())),o.push(E(c,"aria-valuenow",()=>t.indeterminate()?null:t.value()<0?0:t.value()>t.max()?t.max():t.value())),o.push(E(c,"aria-label",()=>t.label()||null)),c.setAttribute("xmlns","http://www.w3.org/2000/svg");let m=document.createElement("circle");m.setAttribute("class","track"),m.setAttribute("part","track"),o.push(E(m,"cx",()=>t.size()/2)),o.push(E(m,"cy",()=>t.size()/2)),o.push(E(m,"r",()=>(t.size()-t.trackWidth())/2)),m.setAttribute("fill","none"),o.push(E(m,"stroke-width",()=>t.trackWidth())),c.appendChild(m);let p=document.createElement("circle");p.setAttribute("class","bar"),p.setAttribute("part","ring"),o.push(E(p,"cx",()=>t.size()/2)),o.push(E(p,"cy",()=>t.size()/2)),o.push(E(p,"r",()=>(t.size()-t.trackWidth())/2)),p.setAttribute("fill","none"),o.push(E(p,"stroke-width",()=>t.trackWidth())),p.setAttribute("stroke-linecap","round"),o.push(E(p,"stroke-dasharray",()=>3.141592653589793*(t.size()-t.trackWidth()))),o.push(E(p,"stroke-dashoffset",()=>t.indeterminate()?3.141592653589793*(t.size()-t.trackWidth())*0.25:3.141592653589793*(t.size()-t.trackWidth())*(1-(t.value()<0?0:t.value()>(t.max()||1)?1:t.value()/(t.max()||1))))),c.appendChild(p),b.appendChild(c);let d=document.createElement("div");d.setAttribute("class","label"),d.setAttribute("aria-hidden","true");let l=document.createElement("slot");return d.appendChild(l),b.appendChild(d),a.appendChild(b),t}});var $e={default:"var(--foreground)",primary:"var(--primary)",info:"var(--info)",success:"var(--success)",warning:"var(--warning)",destructive:"var(--destructive)",accent:"var(--accent)",muted:"var(--muted-foreground)"};function $(e){if(!e)return $e.default;return $e[e]??$e.default}function Et(e){if(!e)return[];return e.split(",").map((n)=>Number.parseFloat(n.trim())).filter((n)=>Number.isFinite(n))}function Nt(e){if(!e)return[];return e.split(",").map((n)=>n.trim())}function at(e,n){if(!e)return n;try{return JSON.parse(e)}catch{return n}}function Ct(e,n,r,o=2){if(e.length===0)return{d:"",points:[]};let t=Math.min(...e),b=Math.max(...e)-t||1,c=r-o*2,m=e.length===1?0:n/(e.length-1),p=e.map((l,i)=>{let u=e.length===1?n/2:i*m,s=o+c-(l-t)/b*c;return[u,s]});return{d:p.map(([l,i],u)=>`${u===0?"M":"L"}${l.toFixed(2)},${i.toFixed(2)}`).join(" "),points:p}}function Sr(e,n,r,o){return[e+r*Math.cos(o),n+r*Math.sin(o)]}function Ht(e,n,r,o,t){let[a,b]=Sr(e,n,r,o),[c,m]=Sr(e,n,r,t),p=t-o>Math.PI?1:0,d=t>o?1:0;return`M${a.toFixed(2)},${b.toFixed(2)} A${r},${r} 0 ${p} ${d} ${c.toFixed(2)},${m.toFixed(2)}`}var $t=P({scope:()=>{let e=N([]),n=N([]);return{seriesPaths:e,gridLines:n}},mount(e,n){let r=e,o=r.seriesPaths,t=r.gridLines,a=D(()=>{let b=at(r.series(),[]),c=r.width(),m=r.height(),p=b.flatMap((f)=>f.data),d=p.length?Math.min(...p):0,i=(p.length?Math.max(...p):1)-d||1,u=m-4;t.set(r.grid()?[0.25,0.5,0.75].map((f,h)=>({id:`g${h}`,y:2+u*f})):[]);let s=[];for(let f of b){if(f.data.length<2)continue;let h=f.data.map((j)=>(j-d)/i*100),{d:v,points:g}=Ct(h,c,m,2);if(!v)continue;let x=$(f.tone),k=g[0],w=g[g.length-1],A=`${v} L${w[0].toFixed(2)},${m} L${k[0].toFixed(2)},${m} Z`;s.push({name:f.name,areaD:A,strokeD:v,color:x,pts:g.map(([j,C],M)=>({id:`${f.name}-${M}`,cx:j,cy:C}))})}o.set(s)});return()=>a()}});var Nl=z("sui-area-chart",{props:{series:y.string({default:"",reflect:!0}),labels:y.string({default:""}),width:y.number({default:300}),height:y.number({default:200}),points:y.bool({default:!1,reflect:!0}),grid:y.bool({default:!1,reflect:!0}),label:y.string({default:"area chart"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.series=n.series,t.labels=n.labels,t.width=n.width,t.height=n.height,t.points=n.points,t.grid=n.grid,t.label=n.label;let a=null;if($t&&typeof $t.setup==="function"){if(e.scope=t,a=$t.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(f)=>R({anchor:f,items:()=>t.gridLines(),key:(h,v)=>{let g=Object.create(t);return g.g=h,g.__i=v,((x)=>h.id)(g)},render:(h,v)=>{let g=Object.create(t);return g.g=h,g.__i=v,g.__v=N(0),((x)=>{let k=document.createDocumentFragment(),w=[],A=document.createElement("line");return A.setAttribute("class","grid-line"),A.setAttribute("x1","0"),w.push(E(A,"x2",()=>(x.__v(),x.width()))),w.push(E(A,"y1",()=>(x.__v(),x.g.y))),w.push(E(A,"y2",()=>(x.__v(),x.g.y))),k.appendChild(A),{nodes:Array.from(k.childNodes),cleanup:()=>{for(let j of w)try{j()}catch(C){}},update:(j,C)=>{x.g=j,x.__i=C,x.__v.set((M)=>M+1)}}})(g)}}),c=(f)=>R({anchor:f,items:()=>t.seriesPaths(),key:(h,v)=>{let g=Object.create(t);return g.s=h,g.__i=v,((x)=>h.name)(g)},render:(h,v)=>{let g=Object.create(t);return g.s=h,g.__i=v,g.__v=N(0),((x)=>{let k=(H)=>R({anchor:H,items:()=>x.s.pts,key:(S,U)=>{let G=Object.create(x);return G.p=S,G.__i=U,((X)=>S.id)(G)},render:(S,U)=>{let G=Object.create(x);return G.p=S,G.__i=U,G.__v=N(0),((X)=>{let W=document.createDocumentFragment(),Y=[],q=document.createElement("circle");return Y.push(E(q,"cx",()=>(X.__v(),X.p.cx))),Y.push(E(q,"cy",()=>(X.__v(),X.p.cy))),q.setAttribute("r","2.5"),Y.push(E(q,"fill",()=>(X.__v(),X.s.color))),W.appendChild(q),{nodes:Array.from(W.childNodes),cleanup:()=>{for(let _ of Y)try{_()}catch(V){}},update:(_,V)=>{X.p=_,X.__i=V,X.__v.set((I)=>I+1)}}})(G)}}),w=(H)=>F({anchor:H,test:()=>{if(x.points())return 0;return-1},branches:[()=>{let S=document.createDocumentFragment(),U=[],G=document.createComment("for");return S.appendChild(G),U.push(k(G)),{nodes:Array.from(S.childNodes),cleanup:()=>{for(let X of U)try{X()}catch(W){}}}}]}),A=document.createDocumentFragment(),j=[],C=document.createElement("path");j.push(E(C,"d",()=>(x.__v(),x.s.areaD))),j.push(E(C,"fill",()=>(x.__v(),x.s.color))),C.setAttribute("fill-opacity","0.3"),C.setAttribute("stroke","none"),A.appendChild(C);let M=document.createElement("path");j.push(E(M,"d",()=>(x.__v(),x.s.strokeD))),M.setAttribute("fill","none"),j.push(E(M,"stroke",()=>(x.__v(),x.s.color))),M.setAttribute("stroke-width","1.75"),M.setAttribute("stroke-linecap","round"),M.setAttribute("stroke-linejoin","round"),A.appendChild(M);let O=document.createComment("if");return A.appendChild(O),j.push(w(O)),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let H of j)try{H()}catch(S){}},update:(H,S)=>{x.s=H,x.__i=S,x.__v.set((U)=>U+1)}}})(g)}}),m=r.shadowRoot??r;L(m);let p=document.createElement("div");p.setAttribute("class","wrap");let d=document.createElement("svg");o.push(E(d,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),d.setAttribute("preserveaspectratio","none"),d.setAttribute("role","img"),o.push(E(d,"aria-label",()=>t.label()));let l=document.createComment("for");d.appendChild(l),o.push(b(l));let i=document.createComment("for");d.appendChild(i),o.push(c(i)),p.appendChild(d);let u=document.createElement("div");u.setAttribute("class","legend");let s=document.createElement("slot");if(u.appendChild(s),p.appendChild(u),m.appendChild(p),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let h=new Proxy(t,{get(v,g){return v[g]},set(v,g,x){let k=v[g];if(k!=null&&typeof k.set==="function"){if(typeof x==="function")k.set(()=>x);else k.set(x);return!0}return!1}});if(f.mount){let v=f.mount(h,r);if(typeof v==="function")o.push(v)}if(f.unmount)o.push(()=>f.unmount(h))}}return t}});var _t=P({scope:()=>{let e=N([]),n=N(!0);return{bars:e,isEmpty:n}},mount(e,n){let r=e,o=r.bars,t=r.isEmpty,a=D(()=>{let b=Et(r.data());if(b.length===0){o.set([]),t.set(!0);return}let c=Nt(r.labels()),m=r.width(),p=r.height(),d=r.max(),l=d>0?d:Math.max(...b,0),i=l<=0?1:l,u=$(r.tone()),s=4,f=3,h=r.orientation()==="horizontal",v=b.length;o.set(b.map((g,x)=>{let k=Math.max(0,g)/i,w,A,j,C;if(h)C=(p-s*(v-1))/v,A=x*(C+s),j=k*m,w=0;else j=(m-s*(v-1))/v,w=x*(j+s),C=k*p,A=p-C;return{id:`b${x}`,x:w,y:A,w:Math.max(0,j),h:Math.max(0,C),rx:f,ry:f,color:u,title:`${c[x]??String(g)}: ${g}`}})),t.set(!1)});return()=>a()}});var Xl=z("sui-bar-chart",{props:{data:y.string({default:"",reflect:!0}),labels:y.string({default:""}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),orientation:y.union(["vertical","horizontal"],{default:"vertical",reflect:!0}),width:y.number({default:300}),height:y.number({default:200}),max:y.number({default:0}),label:y.string({default:"bar chart"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.labels=n.labels,t.tone=n.tone,t.orientation=n.orientation,t.width=n.width,t.height=n.height,t.max=n.max,t.label=n.label;let a=null;if(_t&&typeof _t.setup==="function"){if(e.scope=t,a=_t.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>R({anchor:d,items:()=>t.bars(),key:(l,i)=>{let u=Object.create(t);return u.b=l,u.__i=i,((s)=>l.id)(u)},render:(l,i)=>{let u=Object.create(t);return u.b=l,u.__i=i,u.__v=N(0),((s)=>{let f=document.createDocumentFragment(),h=[],v=document.createElement("rect");h.push(E(v,"x",()=>(s.__v(),s.b.x))),h.push(E(v,"y",()=>(s.__v(),s.b.y))),h.push(E(v,"width",()=>(s.__v(),s.b.w))),h.push(E(v,"height",()=>(s.__v(),s.b.h))),h.push(E(v,"rx",()=>(s.__v(),s.b.rx))),h.push(E(v,"ry",()=>(s.__v(),s.b.ry))),h.push(E(v,"fill",()=>(s.__v(),s.b.color)));let g=document.createElement("title"),x=document.createTextNode("");return g.appendChild(x),h.push(B(x,()=>(s.__v(),s.b.title))),v.appendChild(g),f.appendChild(v),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let k of h)try{k()}catch(w){}},update:(k,w)=>{s.b=k,s.__i=w,s.__v.set((A)=>A+1)}}})(u)}}),c=(d)=>F({anchor:d,test:()=>{if(t.isEmpty())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("svg");return i.push(E(u,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),u.setAttribute("aria-hidden","true"),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(f){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("svg");i.push(E(u,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),u.setAttribute("preserveaspectratio","none"),u.setAttribute("role","img"),i.push(E(u,"aria-label",()=>t.label()));let s=document.createComment("for");return u.appendChild(s),i.push(b(s)),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let f of i)try{f()}catch(h){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createComment("if");if(m.appendChild(p),o.push(c(p)),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var tn=P({scope:()=>{let e=N([]),n=N([]);return{seriesPaths:e,gridLines:n}},mount(e,n){let r=e,o=r.seriesPaths,t=r.gridLines,a=D(()=>{let b=at(r.series(),[]),c=r.width(),m=r.height(),p=b.flatMap((f)=>f.data),d=p.length?Math.min(...p):0,i=(p.length?Math.max(...p):1)-d||1,u=m-4;t.set(r.grid()?[0.25,0.5,0.75].map((f,h)=>({id:`g${h}`,y:2+u*f})):[]);let s=[];for(let f of b){if(f.data.length<2)continue;let h=f.data.map((k)=>(k-d)/i*100),{d:v,points:g}=Ct(h,c,m,2);if(!v)continue;let x=$(f.tone);s.push({name:f.name,strokeD:v,color:x,pts:g.map(([k,w],A)=>({id:`${f.name}-${A}`,cx:k,cy:w}))})}o.set(s)});return()=>a()}});var Ql=z("sui-line-chart",{props:{series:y.string({default:"",reflect:!0}),labels:y.string({default:""}),width:y.number({default:300}),height:y.number({default:200}),points:y.bool({default:!1,reflect:!0}),grid:y.bool({default:!1,reflect:!0}),label:y.string({default:"line chart"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.series=n.series,t.labels=n.labels,t.width=n.width,t.height=n.height,t.points=n.points,t.grid=n.grid,t.label=n.label;let a=null;if(tn&&typeof tn.setup==="function"){if(e.scope=t,a=tn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(f)=>R({anchor:f,items:()=>t.gridLines(),key:(h,v)=>{let g=Object.create(t);return g.g=h,g.__i=v,((x)=>h.id)(g)},render:(h,v)=>{let g=Object.create(t);return g.g=h,g.__i=v,g.__v=N(0),((x)=>{let k=document.createDocumentFragment(),w=[],A=document.createElement("line");return A.setAttribute("class","grid-line"),A.setAttribute("x1","0"),w.push(E(A,"x2",()=>(x.__v(),x.width()))),w.push(E(A,"y1",()=>(x.__v(),x.g.y))),w.push(E(A,"y2",()=>(x.__v(),x.g.y))),k.appendChild(A),{nodes:Array.from(k.childNodes),cleanup:()=>{for(let j of w)try{j()}catch(C){}},update:(j,C)=>{x.g=j,x.__i=C,x.__v.set((M)=>M+1)}}})(g)}}),c=(f)=>R({anchor:f,items:()=>t.seriesPaths(),key:(h,v)=>{let g=Object.create(t);return g.s=h,g.__i=v,((x)=>h.name)(g)},render:(h,v)=>{let g=Object.create(t);return g.s=h,g.__i=v,g.__v=N(0),((x)=>{let k=(O)=>R({anchor:O,items:()=>x.s.pts,key:(H,S)=>{let U=Object.create(x);return U.p=H,U.__i=S,((G)=>H.id)(U)},render:(H,S)=>{let U=Object.create(x);return U.p=H,U.__i=S,U.__v=N(0),((G)=>{let X=document.createDocumentFragment(),W=[],Y=document.createElement("circle");return W.push(E(Y,"cx",()=>(G.__v(),G.p.cx))),W.push(E(Y,"cy",()=>(G.__v(),G.p.cy))),Y.setAttribute("r","2.5"),W.push(E(Y,"fill",()=>(G.__v(),G.s.color))),X.appendChild(Y),{nodes:Array.from(X.childNodes),cleanup:()=>{for(let q of W)try{q()}catch(_){}},update:(q,_)=>{G.p=q,G.__i=_,G.__v.set((V)=>V+1)}}})(U)}}),w=(O)=>F({anchor:O,test:()=>{if(x.points())return 0;return-1},branches:[()=>{let H=document.createDocumentFragment(),S=[],U=document.createComment("for");return H.appendChild(U),S.push(k(U)),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let G of S)try{G()}catch(X){}}}}]}),A=document.createDocumentFragment(),j=[],C=document.createElement("path");j.push(E(C,"d",()=>(x.__v(),x.s.strokeD))),C.setAttribute("fill","none"),j.push(E(C,"stroke",()=>(x.__v(),x.s.color))),C.setAttribute("stroke-width","1.75"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),A.appendChild(C);let M=document.createComment("if");return A.appendChild(M),j.push(w(M)),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let O of j)try{O()}catch(H){}},update:(O,H)=>{x.s=O,x.__i=H,x.__v.set((S)=>S+1)}}})(g)}}),m=r.shadowRoot??r;L(m);let p=document.createElement("div");p.setAttribute("class","wrap");let d=document.createElement("svg");o.push(E(d,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),d.setAttribute("preserveaspectratio","none"),d.setAttribute("role","img"),o.push(E(d,"aria-label",()=>t.label()));let l=document.createComment("for");d.appendChild(l),o.push(b(l));let i=document.createComment("for");d.appendChild(i),o.push(c(i)),p.appendChild(d);let u=document.createElement("div");u.setAttribute("class","legend");let s=document.createElement("slot");if(u.appendChild(s),p.appendChild(u),m.appendChild(p),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let h=new Proxy(t,{get(v,g){return v[g]},set(v,g,x){let k=v[g];if(k!=null&&typeof k.set==="function"){if(typeof x==="function")k.set(()=>x);else k.set(x);return!0}return!1}});if(f.mount){let v=f.mount(h,r);if(typeof v==="function")o.push(v)}if(f.unmount)o.push(()=>f.unmount(h))}}return t}});var nn=P({scope:()=>{let e=N(!1),n=N(""),r=N(""),o=N("var(--primary)");return{hasData:e,strokePath:n,areaPath:r,lineColor:o}},mount(e,n){let r=e,o=r.hasData,t=r.strokePath,a=r.areaPath,b=r.lineColor,c=D(()=>{let m=Et(r.data()),p=r.width(),d=r.height(),l=$(r.tone());if(b.set(l),m.length<2){o.set(!1),t.set(""),a.set("");return}let{d:i,points:u}=Ct(m,p,d,2);if(t.set(i),r.fill()){let s=u[0],f=u[u.length-1];a.set(`${i} L${f[0].toFixed(2)},${d} L${s[0].toFixed(2)},${d} Z`)}else a.set("");o.set(!0)});return()=>c()}});var nd=z("sui-sparkline",{props:{data:y.string({default:"",reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),width:y.number({default:80}),height:y.number({default:20}),fill:y.bool({default:!1,reflect:!0}),label:y.string({default:"sparkline"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.tone=n.tone,t.width=n.width,t.height=n.height,t.fill=n.fill,t.label=n.label;let a=null;if(nn&&typeof nn.setup==="function"){if(e.scope=t,a=nn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>F({anchor:d,test:()=>{if(t.fill())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("path");return i.push(E(u,"d",()=>t.areaPath())),i.push(E(u,"fill",()=>t.lineColor())),u.setAttribute("fill-opacity","0.18"),u.setAttribute("stroke","none"),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(f){}}}}]}),c=(d)=>F({anchor:d,test:()=>{if(t.hasData())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("svg");i.push(E(u,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),i.push(E(u,"width",()=>t.width())),i.push(E(u,"height",()=>t.height())),u.setAttribute("role","img"),i.push(E(u,"aria-label",()=>t.label()));let s=document.createComment("if");u.appendChild(s),i.push(b(s));let f=document.createElement("path");i.push(E(f,"d",()=>t.strokePath())),f.setAttribute("fill","none"),i.push(E(f,"stroke",()=>t.lineColor())),f.setAttribute("stroke-width","1.5"),f.setAttribute("stroke-linecap","round"),f.setAttribute("stroke-linejoin","round");let h=document.createTextNode(`
    `);return f.appendChild(h),u.appendChild(f),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let v of i)try{v()}catch(g){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("svg");i.push(E(u,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),i.push(E(u,"width",()=>t.width())),i.push(E(u,"height",()=>t.height())),u.setAttribute("aria-hidden","true");let s=document.createTextNode(`
  `);return u.appendChild(s),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let f of i)try{f()}catch(h){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createComment("if");if(m.appendChild(p),o.push(c(p)),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var en=P({scope:()=>({segments:N([])}),mount(e,n){let r=e,o=r.segments,t=D(()=>{let a=at(r.data(),[]),b=a.reduce((s,f)=>s+Math.max(0,f.value),0);if(b<=0){o.set([]);return}let c=r.size(),m=r.thickness(),p=(c-m)/2,d=c/2,l=c/2,i=2*Math.PI,u=-Math.PI/2;o.set(a.filter((s)=>s.value>0).map((s,f)=>{let h=s.value/b*i,v=h>=i-0.001?i-0.001:h,g=Ht(d,l,p,u,u+v);return u+=h,{id:`s${f}`,d:g,color:$(s.tone),title:`${s.label}: ${s.value}`}}))});return()=>t()}});var ud=z("sui-donut",{props:{data:y.string({default:"",reflect:!0}),size:y.number({default:120}),thickness:y.number({default:16}),label:y.string({default:"donut chart"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.size=n.size,t.thickness=n.thickness,t.label=n.label;let a=null;if(en&&typeof en.setup==="function"){if(e.scope=t,a=en.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(s)=>R({anchor:s,items:()=>t.segments(),key:(f,h)=>{let v=Object.create(t);return v.seg=f,v.__i=h,((g)=>f.id)(v)},render:(f,h)=>{let v=Object.create(t);return v.seg=f,v.__i=h,v.__v=N(0),((g)=>{let x=document.createDocumentFragment(),k=[],w=document.createElement("path");w.setAttribute("part","segment"),k.push(E(w,"d",()=>(g.__v(),g.seg.d))),w.setAttribute("fill","none"),k.push(E(w,"stroke",()=>(g.__v(),g.seg.color))),k.push(E(w,"stroke-width",()=>(g.__v(),g.thickness()))),w.setAttribute("stroke-linecap","butt");let A=document.createElement("title"),j=document.createTextNode("");return A.appendChild(j),k.push(B(j,()=>(g.__v(),g.seg.title))),w.appendChild(A),x.appendChild(w),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let C of k)try{C()}catch(M){}},update:(C,M)=>{g.seg=C,g.__i=M,g.__v.set((O)=>O+1)}}})(v)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("svg");o.push(E(m,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),m.setAttribute("role","img"),o.push(E(m,"aria-label",()=>t.label())),o.push(E(m,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let p=document.createElement("circle");p.setAttribute("class","track"),p.setAttribute("part","track"),o.push(E(p,"cx",()=>t.size()/2)),o.push(E(p,"cy",()=>t.size()/2)),o.push(E(p,"r",()=>(t.size()-t.thickness())/2)),p.setAttribute("fill","none"),o.push(E(p,"stroke-width",()=>t.thickness()));let d=document.createTextNode(`
  `);p.appendChild(d),m.appendChild(p);let l=document.createComment("for");m.appendChild(l),o.push(b(l)),c.appendChild(m);let i=document.createElement("div");i.setAttribute("class","center");let u=document.createElement("slot");if(i.appendChild(u),c.appendChild(i),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let f=new Proxy(t,{get(h,v){return h[v]},set(h,v,g){let x=h[v];if(x!=null&&typeof x.set==="function"){if(typeof g==="function")x.set(()=>g);else x.set(g);return!0}return!1}});if(s.mount){let h=s.mount(f,r);if(typeof h==="function")o.push(h)}if(s.unmount)o.push(()=>s.unmount(f))}}return t}});var Br=Math.PI/180,rn=135*Br,Fr=405*Br,on=P({scope:()=>({trackD:N(""),valueD:N(""),arcColor:N("var(--primary)")}),mount(e,n){let r=e,o=r.trackD,t=r.valueD,a=r.arcColor,b=D(()=>{let c=r.size(),m=r.thickness(),p=(c-m)/2-2,d=c/2,l=c/2;o.set(Ht(d,l,p,rn,Fr));let i=r.value(),u=r.min(),s=r.max(),f=Math.max(0,Math.min(1,(i-u)/(s-u||1))),h=rn+(Fr-rn)*f;t.set(f>0?Ht(d,l,p,rn,h):"");let v=r.tone(),g=r.lowThreshold(),x=r.highThreshold(),k=v;if(!k)if(g>0&&i<g)k="destructive";else if(x>0&&i<x)k="warning";else if(g>0||x>0)k="success";else k="primary";a.set($(k))});return()=>b()}});var bd=z("sui-gauge",{props:{value:y.number({default:0,reflect:!0}),min:y.number({default:0}),max:y.number({default:100}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"",reflect:!0}),"low-threshold":y.number({default:0}),"high-threshold":y.number({default:0}),size:y.number({default:140}),thickness:y.number({default:12}),label:y.string({default:"gauge"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.min=n.min,t.max=n.max,t.tone=n.tone,t.lowThreshold=n.lowThreshold,t.highThreshold=n.highThreshold,t.size=n.size,t.thickness=n.thickness,t.label=n.label;let a=null;if(on&&typeof on.setup==="function"){if(e.scope=t,a=on.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(s)=>F({anchor:s,test:()=>{if(t.valueD())return 0;return-1},branches:[()=>{let f=document.createDocumentFragment(),h=[],v=document.createElement("path");v.setAttribute("part","fill"),h.push(E(v,"d",()=>t.valueD())),v.setAttribute("fill","none"),h.push(E(v,"stroke",()=>t.arcColor())),h.push(E(v,"stroke-width",()=>t.thickness())),v.setAttribute("stroke-linecap","round");let g=document.createTextNode(`
    `);return v.appendChild(g),f.appendChild(v),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let x of h)try{x()}catch(k){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("svg");o.push(E(m,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),m.setAttribute("role","img"),o.push(E(m,"aria-label",()=>t.label())),o.push(E(m,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let p=document.createElement("path");p.setAttribute("class","track"),p.setAttribute("part","track"),o.push(E(p,"d",()=>t.trackD())),p.setAttribute("fill","none"),o.push(E(p,"stroke-width",()=>t.thickness())),p.setAttribute("stroke-linecap","round");let d=document.createTextNode(`
  `);p.appendChild(d),m.appendChild(p);let l=document.createComment("if");m.appendChild(l),o.push(b(l)),c.appendChild(m);let i=document.createElement("div");i.setAttribute("class","gauge-label");let u=document.createElement("slot");if(i.appendChild(u),c.appendChild(i),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let f=new Proxy(t,{get(h,v){return h[v]},set(h,v,g){let x=h[v];if(x!=null&&typeof x.set==="function"){if(typeof g==="function")x.set(()=>g);else x.set(g);return!0}return!1}});if(s.mount){let h=s.mount(f,r);if(typeof h==="function")o.push(h)}if(s.unmount)o.push(()=>s.unmount(f))}}return t}});var an=P({scope:()=>({bars:N([])}),mount(e,n){let r=e,o=r.bars,t=D(()=>{let a=at(r.segments(),[]),b=a.reduce((d,l)=>d+Math.max(0,l.count),0),c=r.total(),m=c>0?c:b,p=m<=0?1:m;o.set(a.filter((d)=>d.count>0).map((d,l)=>{let i=Math.max(0,d.count)/p*100;return{id:`b${l}`,style:`width:${i.toFixed(3)}%;background:${$(d.tone)};`,title:`${d.label??d.tone}: ${d.count}`}}))});return()=>t()}});var Ad=z("sui-meter",{props:{segments:y.string({default:"",reflect:!0}),total:y.number({default:0}),label:y.string({default:"meter"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.segments=n.segments,t.total=n.total,t.label=n.label;let a=null;if(an&&typeof an.setup==="function"){if(e.scope=t,a=an.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>R({anchor:u,items:()=>t.bars(),key:(s,f)=>{let h=Object.create(t);return h.b=s,h.__i=f,((v)=>s.id)(h)},render:(s,f)=>{let h=Object.create(t);return h.b=s,h.__i=f,h.__v=N(0),((v)=>{let g=document.createDocumentFragment(),x=[],k=document.createElement("div");return k.setAttribute("class","seg"),k.setAttribute("part","segment"),x.push(E(k,"style",()=>(v.__v(),v.b.style))),x.push(E(k,"title",()=>(v.__v(),v.b.title))),g.appendChild(k),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let w of x)try{w()}catch(A){}},update:(w,A)=>{v.b=w,v.__i=A,v.__v.set((j)=>j+1)}}})(h)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("div");m.setAttribute("class","wrap");let p=document.createElement("div");p.setAttribute("class","bar"),p.setAttribute("part","bar"),p.setAttribute("role","img"),o.push(E(p,"aria-label",()=>t.label()));let d=document.createComment("for");p.appendChild(d),o.push(b(d)),m.appendChild(p);let l=document.createElement("div");l.setAttribute("class","legend");let i=document.createElement("slot");if(l.appendChild(i),m.appendChild(l),c.appendChild(m),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(f,h){return f[h]},set(f,h,v){let g=f[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(u.mount){let f=u.mount(s,r);if(typeof f==="function")o.push(f)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var sn=P({scope:()=>({cells:N([]),svgW:N(0),svgH:N(0)}),mount(e,n){let r=e,o=r.cells,t=r.svgW,a=r.svgH,b=D(()=>{let c=Nt(r.rows()),m=Nt(r.cols()),p=r.cellSize(),d=r.gap(),l=r.data(),i=at(l,[]),u=Array.isArray(i)&&i.length>0&&Array.isArray(i[0]),s;if(u)s=i;else{let C=Et(l),M=m.length||1,O=c.length||(C.length>0?Math.ceil(C.length/M):0);s=Array.from({length:O},(H,S)=>C.slice(S*M,(S+1)*M))}let f=s.length,h=f>0?Math.max(...s.map((C)=>C.length)):0,v=s.flatMap((C)=>C.filter((M)=>typeof M==="number")),g=v.length?Math.min(...v):0,k=(v.length?Math.max(...v):1)-g,w=$(r.tone()),A=p+d;t.set(h>0?h*p+(h-1)*d:0),a.set(f>0?f*p+(f-1)*d:0);let j=[];for(let C=0;C<f;C++){let M=s[C]??[];for(let O=0;O<h;O++){let H=M[O]??0,S=k===0?g>0?1:0:(H-g)/k,U=(S*100).toFixed(1),G=S>0?`fill:color-mix(in oklch,${w} ${U}%,transparent)`:"";j.push({id:`c${C}-${O}`,x:O*A,y:C*A,w:p,h:p,fill:G})}}o.set(j)});return()=>b()}});var Dd=z("sui-heatmap",{props:{data:y.string({default:"",reflect:!0}),rows:y.string({default:""}),cols:y.string({default:""}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),"cell-size":y.number({default:20}),gap:y.number({default:3}),label:y.string({default:"heatmap"})},shadow:!0,styles:`
  :host {
    display: inline-block;
    font-family: var(--font-body);
  }
  svg { display: block; overflow: visible; }
  .cell { fill: var(--muted); }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.rows=n.rows,t.cols=n.cols,t.tone=n.tone,t.cellSize=n.cellSize,t.gap=n.gap,t.label=n.label;let a=null;if(sn&&typeof sn.setup==="function"){if(e.scope=t,a=sn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>R({anchor:d,items:()=>t.cells(),key:(l,i)=>{let u=Object.create(t);return u.cell=l,u.__i=i,((s)=>l.id)(u)},render:(l,i)=>{let u=Object.create(t);return u.cell=l,u.__i=i,u.__v=N(0),((s)=>{let f=document.createDocumentFragment(),h=[],v=document.createElement("rect");v.setAttribute("class","cell"),v.setAttribute("part","cell"),h.push(E(v,"x",()=>(s.__v(),s.cell.x))),h.push(E(v,"y",()=>(s.__v(),s.cell.y))),h.push(E(v,"width",()=>(s.__v(),s.cell.w))),h.push(E(v,"height",()=>(s.__v(),s.cell.h))),v.setAttribute("rx","2"),h.push(E(v,"style",()=>(s.__v(),s.cell.fill)));let g=document.createTextNode(`
    `);return v.appendChild(g),f.appendChild(v),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let x of h)try{x()}catch(k){}},update:(x,k)=>{s.cell=x,s.__i=k,s.__v.set((w)=>w+1)}}})(u)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("svg");o.push(E(m,"viewbox",()=>"0 0 "+t.svgW()+" "+t.svgH())),m.setAttribute("role","img"),o.push(E(m,"aria-label",()=>t.label()));let p=document.createComment("for");if(m.appendChild(p),o.push(b(p)),c.appendChild(m),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var Pd=z("sui-button",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["solid","soft","outline","ghost","default"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),type:y.union(["button","submit","reset"],{default:"button",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),loading:y.bool({default:!1,reflect:!0}),label:y.string({default:""}),pressed:y.string({default:"",reflect:!0}),expanded:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
  `,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.type=n.type,t.disabled=n.disabled,t.loading=n.loading,t.label=n.label,t.pressed=n.pressed,t.expanded=n.expanded;let a=(u)=>F({anchor:u,test:()=>{if(t.loading())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("span");h.setAttribute("class","spinner-wrap"),h.setAttribute("aria-hidden","true");let v=document.createElement("span");return v.setAttribute("class","spinner"),h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let g of f)try{g()}catch(x){}}}}]}),b=r.shadowRoot??r;L(b);let c=document.createElement("button");c.setAttribute("part","button"),o.push(E(c,"type",()=>t.type())),o.push(E(c,"disabled",()=>t.disabled()||t.loading())),o.push(E(c,"aria-busy",()=>t.loading()?"true":"false")),o.push(E(c,"aria-disabled",()=>t.disabled()||t.loading()?"true":"false")),o.push(E(c,"aria-label",()=>t.label())),o.push(E(c,"aria-pressed",()=>t.pressed())),o.push(E(c,"aria-expanded",()=>t.expanded()));let m=document.createElement("span");m.setAttribute("class","content");let p=document.createElement("slot");p.setAttribute("name","leading"),m.appendChild(p);let d=document.createElement("slot");o.push(nt(d,r,"data-icon-only")),m.appendChild(d);let l=document.createElement("slot");l.setAttribute("name","trailing"),m.appendChild(l),c.appendChild(m);let i=document.createComment("if");return c.appendChild(i),o.push(a(i)),b.appendChild(c),t}});var un=P({mount(e,n){let r=e,o=N(0),t=()=>{let i=r.tabSlot();if(!i)return Array.from(n.querySelectorAll(":scope > sui-tab"));return i.assignedElements().filter((u)=>u.tagName.toLowerCase()==="sui-tab")},a=(i)=>{if(r.active()===i)return;r.active.set(i),n.dispatchEvent(new CustomEvent("sui-tab-change",{detail:{name:i},bubbles:!0,composed:!0}))},b=D(()=>{o();let i=r.active(),u=t(),s=i;if(!s&&u.length>0){if(s=u[0].getAttribute("name")??"",s)r.active.set(s)}for(let f of u){let h=f.getAttribute("name")??"";if(h===s)f.setAttribute("active","");else f.removeAttribute("active");f.id=`sui-tab-${h}`,f.setAttribute("aria-controls",`sui-tabpanel-${h}`)}for(let f of Array.from(n.querySelectorAll(':scope > sui-tab-panel, :scope > [role="tabpanel"]')))if((f.getAttribute("name")??"")===s)f.removeAttribute("hidden"),f.setAttribute("active","");else f.setAttribute("hidden",""),f.removeAttribute("active")}),c=(i)=>{let u=r.orientation()==="vertical",s=u?"ArrowDown":"ArrowRight",f=u?"ArrowUp":"ArrowLeft",h=[s,f,"Home","End"],v=["Enter"," "];if(!h.includes(i.key)&&!v.includes(i.key))return;i.preventDefault();let g=t().filter((C)=>!C.hasAttribute("disabled"));if(!g.length)return;let x=g.map((C)=>C.getAttribute("name")??""),k=r.active(),w=Math.max(0,x.indexOf(k)),A=w;if(i.key===s)A=(w+1)%x.length;else if(i.key===f)A=(w-1+x.length)%x.length;else if(i.key==="Home")A=0;else if(i.key==="End")A=x.length-1;let j=x[A];a(j),queueMicrotask(()=>{let C=g[A],M=C.shadowRoot?.querySelector("button");M?M.focus():C.focus()})},m=(i)=>{let u=i.target.closest("sui-tab");if(!u||u.hasAttribute("disabled"))return;let s=u.getAttribute("name")??"";if(s)a(s)},p=()=>o.set(o()+1),d=r.strip(),l=r.tabSlot();if(d)d.addEventListener("keydown",c),d.addEventListener("click",m);if(l)l.addEventListener("slotchange",p);return()=>{if(b(),d)d.removeEventListener("keydown",c),d.removeEventListener("click",m);if(l)l.removeEventListener("slotchange",p)}}});var Bd=z("sui-tabs",{props:{active:y.string({default:"",reflect:!0}),variant:y.union(["underline","pill","enclosed"],{default:"underline",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),sticky:y.bool({default:!1,reflect:!0}),orientation:y.union(["horizontal","vertical"],{default:"horizontal",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.active=n.active,t.variant=n.variant,t.size=n.size,t.tone=n.tone,t.sticky=n.sticky,t.orientation=n.orientation;let a=null;if(un&&typeof un.setup==="function"){if(e.scope=t,a=un.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","strip"),c.setAttribute("part","strip"),c.setAttribute("role","tablist"),o.push(E(c,"aria-orientation",()=>t.orientation())),T(t,"strip",c);let m=document.createElement("slot");if(T(t,"tabSlot",m),c.appendChild(m),b.appendChild(c),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var ln=P({mount(e,n){let r=e,o=D(()=>{let t=r.btn();if(!t)return;t.tabIndex=r.active()?0:-1});return()=>{o()}}});var Jd=z("sui-tab",{props:{name:y.string({default:"",reflect:!0}),label:y.string({default:"",reflect:!0}),count:y.string({default:"",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),active:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.name=n.name,t.label=n.label,t.count=n.count,t.disabled=n.disabled,t.active=n.active;let a=null;if(ln&&typeof ln.setup==="function"){if(e.scope=t,a=ln.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>F({anchor:u,test:()=>{if(t.count()!=="")return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("span");h.setAttribute("class","count"),h.setAttribute("aria-hidden","true");let v=document.createTextNode("");return h.appendChild(v),f.push(B(v,()=>t.count())),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let g of f)try{g()}catch(x){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("button");m.setAttribute("part","button"),m.setAttribute("type","button"),m.setAttribute("role","tab"),o.push(E(m,"data-name",()=>t.name())),o.push(E(m,"aria-selected",()=>t.active()?"true":"false")),o.push(E(m,"aria-disabled",()=>t.disabled()?"true":"false")),o.push(E(m,"disabled",()=>t.disabled())),T(t,"btn",m);let p=document.createElement("span");p.setAttribute("class","leading"),p.setAttribute("part","leading");let d=document.createElement("slot");d.setAttribute("name","leading"),p.appendChild(d),m.appendChild(p);let l=document.createElement("slot");m.appendChild(l);let i=document.createComment("if");if(m.appendChild(i),o.push(b(i)),c.appendChild(m),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(f,h){return f[h]},set(f,h,v){let g=f[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(u.mount){let f=u.mount(s,r);if(typeof f==="function")o.push(f)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var Zd=z("sui-empty-state",{props:{variant:y.union(["default","inline","placeholder","connecting"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.variant=n.variant,t.size=n.size,t.tone=n.tone;let a=r.shadowRoot??r;L(a);let b=document.createElement("div");b.setAttribute("class","box"),b.setAttribute("part","box"),o.push(E(b,"role",()=>t.variant()==="connecting"?"status":"note")),o.push(E(b,"aria-live",()=>t.variant()==="connecting"?"polite":"off"));let c=document.createElement("div");c.setAttribute("class","icon"),c.setAttribute("part","icon"),c.setAttribute("aria-hidden","true");let m=document.createElement("slot");m.setAttribute("name","icon"),o.push(nt(m,r,"data-no-icon")),c.appendChild(m),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","copy");let d=document.createElement("div");d.setAttribute("class","title"),d.setAttribute("part","title");let l=document.createElement("slot");l.setAttribute("name","title"),d.appendChild(l),p.appendChild(d);let i=document.createElement("div");i.setAttribute("class","desc"),i.setAttribute("part","desc");let u=document.createElement("slot");i.appendChild(u),p.appendChild(i),b.appendChild(p);let s=document.createElement("div");s.setAttribute("class","actions"),s.setAttribute("part","actions");let f=document.createElement("slot");return f.setAttribute("name","action"),s.appendChild(f),b.appendChild(s),a.appendChild(b),t}});var ra=0,dn=P({mount(e,n){let r=e,o=r.inputEl();if(!o)return;n.expose({focus:()=>o.focus(),blur:()=>o.blur(),clear:()=>{o.value="",r.value.set("")},reportValidity:()=>o.reportValidity()}),Q(o,"input",()=>n.emit("sui-input",{value:o.value})),Q(o,"change",()=>n.emit("sui-input-change",{value:o.value}));let t=`sui-input-err-${++ra}`;ut(E(o,"aria-describedby",()=>{let a=[];if(r.invalid()&&r.errorMessage()){let c=r.errorEl,m=typeof c==="function"?c():null;if(m)m.id=t,a.push(t)}let b=r.describedBy();if(b)a.push(b);return a.length>0?a.join(" "):null}))}});var _d=z("sui-input",{props:{type:y.union(["text","search","email","url","tel","password","number"],{default:"text",reflect:!0}),value:y.string({default:""}),placeholder:y.string({default:""}),name:y.string({default:""}),autocomplete:y.string({default:""}),required:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),readonly:y.bool({default:!1,reflect:!0}),invalid:y.bool({default:!1,reflect:!0}),loading:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["default","ghost"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),label:y.string({default:""}),"described-by":y.string({default:""}),"error-message":y.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.type=n.type,t.value=n.value,t.placeholder=n.placeholder,t.name=n.name,t.autocomplete=n.autocomplete,t.required=n.required,t.disabled=n.disabled,t.readonly=n.readonly,t.invalid=n.invalid,t.loading=n.loading,t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.label=n.label,t.describedBy=n.describedBy,t.errorMessage=n.errorMessage;let a=null;if(dn&&typeof dn.setup==="function"){if(e.scope=t,a=dn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(s)=>F({anchor:s,test:()=>{if(t.loading())return 0;return 1},branches:[()=>{let f=document.createDocumentFragment(),h=[],v=document.createElement("span");return v.setAttribute("class","spinner"),v.setAttribute("aria-hidden","true"),f.appendChild(v),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let g of h)try{g()}catch(x){}}}},()=>{let f=document.createDocumentFragment(),h=[],v=document.createElement("slot");return v.setAttribute("name","trailing"),h.push(nt(v,r,"data-no-trailing")),f.appendChild(v),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let g of h)try{g()}catch(x){}}}}]}),c=(s)=>F({anchor:s,test:()=>{if(t.invalid()&&t.errorMessage())return 0;return-1},branches:[()=>{let f=document.createDocumentFragment(),h=[],v=document.createElement("span");v.setAttribute("class","error"),v.setAttribute("role","alert"),T(t,"errorEl",v);let g=document.createTextNode("");return v.appendChild(g),h.push(B(g,()=>t.errorMessage())),f.appendChild(v),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let x of h)try{x()}catch(k){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createElement("div");p.setAttribute("class","wrap"),p.setAttribute("part","wrap");let d=document.createElement("slot");d.setAttribute("name","leading"),o.push(nt(d,r,"data-no-leading")),p.appendChild(d);let l=document.createElement("input");l.setAttribute("part","input"),o.push(E(l,"type",()=>t.type())),o.push(E(l,"placeholder",()=>t.placeholder())),o.push(E(l,"name",()=>t.name())),o.push(E(l,"autocomplete",()=>t.autocomplete()||"off")),o.push(E(l,"disabled",()=>t.disabled())),o.push(E(l,"readonly",()=>t.readonly())),o.push(E(l,"required",()=>t.required())),o.push(E(l,"aria-label",()=>t.label())),o.push(E(l,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(E(l,"aria-required",()=>t.required()?"true":"false")),o.push(E(l,"aria-busy",()=>t.loading()?"true":"false")),o.push(mt(l,t.value)),T(t,"inputEl",l),p.appendChild(l);let i=document.createComment("if");p.appendChild(i),o.push(b(i)),m.appendChild(p);let u=document.createComment("if");if(m.appendChild(u),o.push(c(u)),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let f=new Proxy(t,{get(h,v){return h[v]},set(h,v,g){let x=h[v];if(x!=null&&typeof x.set==="function"){if(typeof g==="function")x.set(()=>g);else x.set(g);return!0}return!1}});if(s.mount){let h=s.mount(f,r);if(typeof h==="function")o.push(h)}if(s.unmount)o.push(()=>s.unmount(f))}}return t}});var oa={git:"git-branch",refresh:"refresh-cw",stop:"stop-circle",warning:"triangle-alert"};var lt=null,St=null;function cn(){return lt}function Gr(){if(lt)return Promise.resolve(lt);if(St)return St;return St=fetch("/_sui/lucide-icons.json").then((e)=>{if(!e.ok)throw Error(`lucide registry fetch ${e.status}`);return e.json()}).then((e)=>{return lt=e,e}).catch((e)=>{throw St=null,e}),St}function Ur(e){if(!e||!lt)return;if(lt[e])return e;let n=oa[e];if(n&&lt[n])return n;return}function Xr(e){return lt?lt[e]:void 0}var qr="http://www.w3.org/2000/svg",tr=N(cn()?1:0),_e=!1;function aa(){if(_e||cn())return;_e=!0,Gr().then(()=>tr.set(tr()+1)).catch(()=>{_e=!1})}function ia(e){let n=e.createElementNS(qr,"svg");return n.setAttribute("part","svg"),n.setAttribute("class","sui-icon-svg"),n.setAttribute("viewBox","0 0 24 24"),n.setAttribute("fill","none"),n.setAttribute("stroke","currentColor"),n.setAttribute("stroke-width","2"),n.setAttribute("stroke-linecap","round"),n.setAttribute("stroke-linejoin","round"),n}function sa(e,n){while(e.firstChild)e.removeChild(e.firstChild);let r=new DOMParser().parseFromString(`<svg xmlns="${qr}">${n}</svg>`,"image/svg+xml").documentElement;while(r.firstChild)e.appendChild(e.ownerDocument.adoptNode(r.firstChild))}var pn=P({mount(e,n){let r=e,o=n.shadowRoot,t=null,a=null,b=D(()=>{let m=r.label();if(m)n.setAttribute("role","img"),n.setAttribute("aria-label",m),n.removeAttribute("aria-hidden");else n.setAttribute("aria-hidden","true"),n.removeAttribute("role"),n.removeAttribute("aria-label")}),c=D(()=>{tr();let m=r.name(),p=Ur(m)??null,d=r.label();if(!p){if(t)t.remove(),t=null;if(a=null,m&&!cn())aa();return}if(!t)t=ia(n.ownerDocument),o.insertBefore(t,o.firstChild);if(p!==a)sa(t,Xr(p)),a=p;if(d)t.removeAttribute("aria-hidden");else t.setAttribute("aria-hidden","true")});return()=>{b(),c()}}});var sc=z("sui-icon",{props:{name:y.string({default:"",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:y.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),label:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.name=n.name,t.size=n.size,t.tone=n.tone,t.label=n.label;let a=null;if(pn&&typeof pn.setup==="function"){if(e.scope=t,a=pn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var fn=P({mount(e,n){let r=e,o=()=>Array.from(n.querySelectorAll(":scope > option, :scope > optgroup > option")),t=()=>{let u=r.value(),s=o();if(s.length===0)return u;let f=s.find((v)=>v.value===u)??s.find((v)=>v.hasAttribute("selected"))??s[0];return(f?.textContent??"").trim()||f?.value||u},a=()=>{let u=r.selectEl();if(!u){let f=o(),h=f.find((v)=>v.value===r.value())??f.find((v)=>v.hasAttribute("selected"))??f[0];if(!h||h.value==="")n.setAttribute("data-placeholder","");else n.removeAttribute("data-placeholder");return}let s=u.selectedOptions?.[0]??u.options[u.selectedIndex];if(!s||s.value==="")n.setAttribute("data-placeholder","");else n.removeAttribute("data-placeholder")},b=()=>{let u=r.selectEl();if(!u)return;while(u.firstChild)u.removeChild(u.firstChild);let s=document.createDocumentFragment(),f=n.querySelectorAll(":scope > option, :scope > optgroup");for(let h of Array.from(f))s.appendChild(h.cloneNode(!0));u.appendChild(s),u.value=r.value()},c=()=>{let u=r.valueLabel();if(u)u.textContent=t()},m=D(()=>{let u=r.value(),s=r.selectEl();if(s)s.value=u;c(),a()}),p=(u)=>{let s=u.target.value;r.value.set(s),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:s},bubbles:!0,composed:!0}))},d=r.selectEl();if(d)d.addEventListener("change",p);let l=r.optionsSlot(),i=()=>{b(),c(),a()};if(l)l.addEventListener("slotchange",i);return b(),c(),a(),()=>{if(d)d.removeEventListener("change",p);if(l)l.removeEventListener("slotchange",i);m()}}});var fc=z("sui-select",{props:{value:y.string({default:""}),name:y.string({default:""}),required:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),invalid:y.bool({default:!1,reflect:!0}),tone:y.union(["default","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),label:y.string({default:""}),"described-by":y.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.name=n.name,t.required=n.required,t.disabled=n.disabled,t.invalid=n.invalid,t.tone=n.tone,t.size=n.size,t.label=n.label,t.describedBy=n.describedBy;let a=null;if(fn&&typeof fn.setup==="function"){if(e.scope=t,a=fn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","field"),c.setAttribute("part","field");let m=document.createElement("span");m.setAttribute("class","value"),m.setAttribute("aria-hidden","true"),T(t,"valueLabel",m),c.appendChild(m);let p=document.createElement("svg");p.setAttribute("class","chevron"),p.setAttribute("part","chevron"),p.setAttribute("viewbox","0 0 16 16"),p.setAttribute("aria-hidden","true"),p.setAttribute("focusable","false"),p.setAttribute("fill","none"),p.setAttribute("stroke","currentColor"),p.setAttribute("stroke-width","1.75"),p.setAttribute("stroke-linecap","round"),p.setAttribute("stroke-linejoin","round");let d=document.createElement("path");d.setAttribute("d","M4 6l4 4 4-4"),p.appendChild(d),c.appendChild(p);let l=document.createElement("select");l.setAttribute("part","select"),o.push(E(l,"name",()=>t.name())),o.push(E(l,"disabled",()=>t.disabled())),o.push(E(l,"required",()=>t.required())),o.push(E(l,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(E(l,"aria-label",()=>t.label())),o.push(E(l,"aria-describedby",()=>t.describedBy())),T(t,"selectEl",l),c.appendChild(l),b.appendChild(c);let i=document.createElement("span");i.setAttribute("hidden","");let u=document.createElement("slot");if(T(t,"optionsSlot",u),i.appendChild(u),b.appendChild(i),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let f=new Proxy(t,{get(h,v){return h[v]},set(h,v,g){let x=h[v];if(x!=null&&typeof x.set==="function"){if(typeof g==="function")x.set(()=>g);else x.set(g);return!0}return!1}});if(s.mount){let h=s.mount(f,r);if(typeof h==="function")o.push(h)}if(s.unmount)o.push(()=>s.unmount(f))}}return t}});var hc=z("sui-separator",{props:{orientation:y.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:y.union(["solid","dashed"],{default:"solid",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.orientation=n.orientation,t.variant=n.variant;let a=r.shadowRoot??r;L(a);let b=document.createElement("hr");return b.setAttribute("class","sep"),b.setAttribute("part","sep"),b.setAttribute("role","separator"),o.push(E(b,"aria-orientation",()=>t.orientation())),a.appendChild(b),t}});var mn=P({mount(e,n){let r=e,o=D(()=>{let t=r.labelEl();if(!t)return;t.htmlFor=r.for?.()??""});return()=>{o()}}});var Ac=z("sui-label",{props:{for:y.string({default:""}),size:y.union(["sm","md"],{default:"sm",reflect:!0}),required:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.for=n.for,t.size=n.size,t.required=n.required;let a=null;if(mn&&typeof mn.setup==="function"){if(e.scope=t,a=mn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(l)=>F({anchor:l,test:()=>{if(t.required())return 0;return-1},branches:[()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("span");s.setAttribute("class","req"),s.setAttribute("aria-hidden","true");let f=document.createTextNode("*");return s.appendChild(f),i.appendChild(s),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let h of u)try{h()}catch(v){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("label");m.setAttribute("part","label"),T(t,"labelEl",m);let p=document.createElement("slot");m.appendChild(p);let d=document.createComment("if");if(m.appendChild(d),o.push(b(d)),c.appendChild(m),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(u,s){return u[s]},set(u,s,f){let h=u[s];if(h!=null&&typeof h.set==="function"){if(typeof f==="function")h.set(()=>f);else h.set(f);return!0}return!1}});if(l.mount){let u=l.mount(i,r);if(typeof u==="function")o.push(u)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var bn=P({mount(e,n){let r=e,o=D(()=>{let t=r.closeBtn();if(!t)return;let a=()=>{if(!r.dismissible())return;n.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),n.remove()};return t.addEventListener("click",a),()=>t.removeEventListener("click",a)});return()=>{o()}}});var Lc=z("sui-alert",{props:{tone:y.union(["info","success","warning","destructive","muted"],{default:"info",reflect:!0}),dismissible:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.dismissible=n.dismissible;let a=null;if(bn&&typeof bn.setup==="function"){if(e.scope=t,a=bn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","box"),c.setAttribute("part","box"),o.push(E(c,"role",()=>t.tone()==="destructive"||t.tone()==="warning"?"alert":"status")),o.push(E(c,"aria-live",()=>t.tone()==="destructive"||t.tone()==="warning"?"assertive":"polite")),c.setAttribute("aria-atomic","true");let m=document.createElement("span"),p=document.createElement("slot");p.setAttribute("name","icon"),m.appendChild(p),c.appendChild(m);let d=document.createElement("div");d.setAttribute("class","content");let l=document.createElement("div");l.setAttribute("class","title");let i=document.createElement("slot");i.setAttribute("name","title"),l.appendChild(i),d.appendChild(l);let u=document.createElement("div");u.setAttribute("class","body");let s=document.createElement("slot");u.appendChild(s),d.appendChild(u);let f=document.createElement("div");f.setAttribute("class","actions");let h=document.createElement("slot");h.setAttribute("name","actions"),f.appendChild(h),d.appendChild(f),c.appendChild(d);let v=document.createElement("button");v.setAttribute("class","close"),v.setAttribute("part","close"),v.setAttribute("aria-label","Dismiss"),T(t,"closeBtn",v);let g=document.createTextNode("×");if(v.appendChild(g),c.appendChild(v),b.appendChild(c),a!=null){let x=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(x){let k=new Proxy(t,{get(w,A){return w[A]},set(w,A,j){let C=w[A];if(C!=null&&typeof C.set==="function"){if(typeof j==="function")C.set(()=>j);else C.set(j);return!0}return!1}});if(x.mount){let w=x.mount(k,r);if(typeof w==="function")o.push(w)}if(x.unmount)o.push(()=>x.unmount(k))}}return t}});var ua=0,hn=P({mount(e,n){let r=e,o=`sui-tooltip-${++ua}`,t=D(()=>{let m=r.bubbleEl();if(m)m.id=o}),a=()=>{let m=r.defaultSlot();if(!m)return;let p=m.assignedElements({flatten:!0})[0];if(!p)return;let d=p.getAttribute("aria-describedby")??"";if(!d.split(/\s+/).includes(o))p.setAttribute("aria-describedby",d?`${d} ${o}`:o)},b=D(()=>{let m=r.defaultSlot();if(!m)return;return m.addEventListener("slotchange",a),a(),()=>m.removeEventListener("slotchange",a)}),c=(m)=>{if(m.key==="Escape")n.blur()};return n.addEventListener("keydown",c),()=>{t(),b(),n.removeEventListener("keydown",c)}}});var Nc=z("sui-tooltip",{props:{placement:y.union(["top","bottom","left","right"],{default:"top",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.placement=n.placement;let a=null;if(hn&&typeof hn.setup==="function"){if(e.scope=t,a=hn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");T(t,"defaultSlot",c),b.appendChild(c);let m=document.createElement("span");m.setAttribute("class","bubble"),m.setAttribute("role","tooltip"),m.setAttribute("part","bubble"),T(t,"bubbleEl",m);let p=document.createElement("slot");if(p.setAttribute("name","content"),m.appendChild(p),b.appendChild(m),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var la=0,vn=P({mount(e,n){let r=e,o=`sui-collapsible-${++la}`,t=D(()=>{let l=r.contentWrap();if(l)l.id=o}),a=()=>{let l=r.triggerSlot();if(!l)return;let i=r.open();for(let u of l.assignedElements()){let s=u;if(s.setAttribute("aria-expanded",String(i)),s.setAttribute("aria-controls",o),!s.hasAttribute("tabindex")&&!s.matches("button, a, input, select, textarea"))s.setAttribute("tabindex","0")}},b=D(()=>{let l=r.triggerSlot();if(!l)return;return l.addEventListener("slotchange",a),a(),()=>l.removeEventListener("slotchange",a)}),c=D(()=>{r.open(),r.triggerSlot(),a()}),m=()=>{if(r.disabled())return;let l=!r.open();r.open.set(l),n.dispatchEvent(new CustomEvent("sui-toggle",{detail:{open:l},bubbles:!0,composed:!0}))},p=(l)=>{if(l.key==="Enter"||l.key===" ")l.preventDefault(),m()},d=D(()=>{let l=r.triggerWrap();if(!l)return;return l.addEventListener("click",m),l.addEventListener("keydown",p),()=>{l.removeEventListener("click",m),l.removeEventListener("keydown",p)}});return()=>{t(),b(),c(),d()}}});var Uc=z("sui-collapsible",{props:{open:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.disabled=n.disabled;let a=null;if(vn&&typeof vn.setup==="function"){if(e.scope=t,a=vn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","trigger-wrap"),T(t,"triggerWrap",c);let m=document.createElement("slot");m.setAttribute("name","trigger"),T(t,"triggerSlot",m),c.appendChild(m),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","content-wrap"),p.setAttribute("part","content"),T(t,"contentWrap",p);let d=document.createElement("div");d.setAttribute("class","content-inner");let l=document.createElement("slot");if(d.appendChild(l),p.appendChild(d),b.appendChild(p),a!=null){let i=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(i){let u=new Proxy(t,{get(s,f){return s[f]},set(s,f,h){let v=s[f];if(v!=null&&typeof v.set==="function"){if(typeof h==="function")v.set(()=>h);else v.set(h);return!0}return!1}});if(i.mount){let s=i.mount(u,r);if(typeof s==="function")o.push(s)}if(i.unmount)o.push(()=>i.unmount(u))}}return t}});var gn=P({mount(e,n){let r=e,o=D(()=>{let a=r.inputEl();if(!a)return;a.indeterminate=r.indeterminate()}),t=D(()=>{let a=r.inputEl();if(!a)return;let b=()=>{r.indeterminate.set(!1),n.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:a.checked,value:r.value()},bubbles:!0,composed:!0}))};return a.addEventListener("change",b),()=>a.removeEventListener("change",b)});return()=>{o(),t()}}});var Wc=z("sui-checkbox",{props:{checked:y.bool({default:!1,reflect:!0}),indeterminate:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),required:y.bool({default:!1,reflect:!0}),invalid:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),name:y.string({default:""}),value:y.string({default:"on"}),label:y.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.checked=n.checked,t.indeterminate=n.indeterminate,t.disabled=n.disabled,t.required=n.required,t.invalid=n.invalid,t.tone=n.tone,t.size=n.size,t.name=n.name,t.value=n.value,t.label=n.label;let a=null;if(gn&&typeof gn.setup==="function"){if(e.scope=t,a=gn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("label");c.setAttribute("part","label");let m=document.createElement("span");m.setAttribute("class","box"),m.setAttribute("part","box");let p=document.createElement("input");p.setAttribute("part","input"),p.setAttribute("type","checkbox"),o.push(bt(p,t.checked)),o.push(E(p,"disabled",()=>t.disabled())),o.push(E(p,"required",()=>t.required())),o.push(E(p,"name",()=>t.name())),o.push(E(p,"value",()=>t.value())),o.push(E(p,"aria-checked",()=>t.indeterminate()?"mixed":t.checked()?"true":"false")),o.push(E(p,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(E(p,"aria-label",()=>t.label()||t.name()||"checkbox")),T(t,"inputEl",p),m.appendChild(p);let d=document.createElement("span");d.setAttribute("class","glyph check"),d.setAttribute("aria-hidden","true"),m.appendChild(d);let l=document.createElement("span");l.setAttribute("class","glyph mixed"),l.setAttribute("aria-hidden","true"),m.appendChild(l),c.appendChild(m);let i=document.createElement("slot");if(c.appendChild(i),b.appendChild(c),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(f,h){return f[h]},set(f,h,v){let g=f[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(u.mount){let f=u.mount(s,r);if(typeof f==="function")o.push(f)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var yn=P({mount(e,n){let r=e,o=D(()=>{let t=r.inputEl();if(!t)return;let a=()=>{n.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:t.checked},bubbles:!0,composed:!0}))};return t.addEventListener("change",a),()=>t.removeEventListener("change",a)});return()=>o()}});var $c=z("sui-switch",{props:{checked:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),name:y.string({default:""}),value:y.string({default:"on"}),label:y.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.checked=n.checked,t.disabled=n.disabled,t.tone=n.tone,t.size=n.size,t.name=n.name,t.value=n.value,t.label=n.label;let a=null;if(yn&&typeof yn.setup==="function"){if(e.scope=t,a=yn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("label"),m=document.createElement("span");m.setAttribute("class","track"),m.setAttribute("part","track");let p=document.createElement("input");p.setAttribute("part","input"),p.setAttribute("type","checkbox"),p.setAttribute("role","switch"),o.push(bt(p,t.checked)),o.push(E(p,"disabled",()=>t.disabled())),o.push(E(p,"name",()=>t.name())),o.push(E(p,"value",()=>t.value())),o.push(E(p,"aria-checked",()=>t.checked()?"true":"false")),o.push(E(p,"aria-label",()=>t.label()||t.name()||"toggle")),T(t,"inputEl",p),m.appendChild(p);let d=document.createElement("span");d.setAttribute("class","thumb"),d.setAttribute("part","thumb"),m.appendChild(d),c.appendChild(m);let l=document.createElement("slot");if(c.appendChild(l),b.appendChild(c),a!=null){let i=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(i){let u=new Proxy(t,{get(s,f){return s[f]},set(s,f,h){let v=s[f];if(v!=null&&typeof v.set==="function"){if(typeof h==="function")v.set(()=>h);else v.set(h);return!0}return!1}});if(i.mount){let s=i.mount(u,r);if(typeof s==="function")o.push(s)}if(i.unmount)o.push(()=>i.unmount(u))}}return t}});var xn=P({mount(e,n){let r=e,o=r.value(),t=D(()=>{let a=r.textareaEl();if(!a)return;let b=()=>{n.dispatchEvent(new CustomEvent("sui-input",{detail:{value:a.value},bubbles:!0,composed:!0}))},c=()=>{let m=r.value();if(m!==o)o=m,n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:m},bubbles:!0,composed:!0}))};return a.addEventListener("input",b),a.addEventListener("blur",c),()=>{a.removeEventListener("input",b),a.removeEventListener("blur",c)}});return()=>t()}});var op=z("sui-textarea",{props:{value:y.string({default:""}),placeholder:y.string({default:""}),name:y.string({default:""}),rows:y.number({default:4}),required:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),readonly:y.bool({default:!1,reflect:!0}),invalid:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["default","code"],{default:"default",reflect:!0}),size:y.union(["sm","md"],{default:"md",reflect:!0}),resize:y.union(["none","vertical","horizontal","both"],{default:"vertical"}),label:y.string({default:""}),"described-by":y.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.placeholder=n.placeholder,t.name=n.name,t.rows=n.rows,t.required=n.required,t.disabled=n.disabled,t.readonly=n.readonly,t.invalid=n.invalid,t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.resize=n.resize,t.label=n.label,t.describedBy=n.describedBy;let a=null;if(xn&&typeof xn.setup==="function"){if(e.scope=t,a=xn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("textarea");if(c.setAttribute("part","textarea"),o.push(E(c,"placeholder",()=>t.placeholder())),o.push(E(c,"name",()=>t.name())),o.push(E(c,"rows",()=>t.rows())),o.push(E(c,"disabled",()=>t.disabled())),o.push(E(c,"required",()=>t.required())),o.push(E(c,"readonly",()=>t.readonly())),o.push(E(c,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(E(c,"aria-required",()=>t.required()?"true":"false")),o.push(E(c,"aria-label",()=>t.label())),o.push(E(c,"aria-describedby",()=>t.describedBy())),o.push(D(()=>{let m=t.resize();if(m==null)c.style.removeProperty("resize");else c.style.setProperty("resize",String(m))})),o.push(mt(c,t.value)),T(t,"textareaEl",c),b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var Rr=0,wn=P({mount(e,n){let r=e,o=`sui-field-hint-${++Rr}`,t=`sui-field-error-${++Rr}`,a=D(()=>{let p=r.hintEl(),d=r.errorEl();if(p)p.id=o;if(d)d.id=t}),b=()=>{let d=r.controlSlot()?.assignedElements({flatten:!0})[0];if(!d)return;let l=r.invalid(),i=l?t:o,s=[...(d.getAttribute("aria-describedby")||"").split(/\s+/).filter((f)=>f&&f!==o&&f!==t),i].join(" ").trim();if(s)d.setAttribute("aria-describedby",s);if("invalid"in d)d.invalid=l;else d.setAttribute("aria-invalid",l?"true":"false")},c=D(()=>{r.invalid(),b()}),m=D(()=>{let p=r.controlSlot();if(!p)return;return p.addEventListener("slotchange",b),()=>p.removeEventListener("slotchange",b)});return()=>{a(),c(),m()}}});var dp=z("sui-field",{props:{invalid:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.invalid=n.invalid;let a=null;if(wn&&typeof wn.setup==="function"){if(e.scope=t,a=wn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");c.setAttribute("name","label"),b.appendChild(c);let m=document.createElement("slot");T(t,"controlSlot",m),b.appendChild(m);let p=document.createElement("div");p.setAttribute("class","hint"),T(t,"hintEl",p);let d=document.createElement("slot");d.setAttribute("name","hint"),p.appendChild(d),b.appendChild(p);let l=document.createElement("div");l.setAttribute("class","error"),l.setAttribute("role","alert"),l.setAttribute("aria-live","polite"),T(t,"errorEl",l);let i=document.createElement("slot");if(i.setAttribute("name","error"),l.appendChild(i),b.appendChild(l),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(f,h){return f[h]},set(f,h,v){let g=f[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(u.mount){let f=u.mount(s,r);if(typeof f==="function")o.push(f)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var An=P({scope:(e,n)=>{let r=e,o=J(null),t=Z(()=>r.tone()==="destructive"?"destructive":"default");function a(m){n.dispatchEvent(new CustomEvent(m,{bubbles:!0,composed:!0})),r.open.set(!1)}function b(){a("sui-confirm")}function c(){a("sui-cancel")}return{dialogRef:o,confirmTone:t,onConfirm:b,onCancel:c}},mount(e,n){let r=e,o=r.open,t=r.dialogRef,a=D(()=>{let c=o(),m=t();if(!m)return;if(c&&typeof m.show==="function")m.show();else if(!c&&typeof m.close==="function")m.close()}),b=()=>{if(o())o.set(!1)};return n.addEventListener("sui-close",b),()=>{a(),n.removeEventListener("sui-close",b)}}});var hp=z("sui-alert-dialog",{props:{open:y.bool({default:!1,reflect:!0}),tone:y.union(["default","destructive"],{default:"default",reflect:!0}),"confirm-label":y.string({default:"Confirm"}),"cancel-label":y.string({default:"Cancel"})},shadow:!0,styles:`
  :host { display: contents; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.tone=n.tone,t.confirmLabel=n.confirmLabel,t.cancelLabel=n.cancelLabel;let a=null;if(An&&typeof An.setup==="function"){if(e.scope=t,a=An.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("sui-dialog");c.setAttribute("role-override","alertdialog"),c.setAttribute("size","sm"),c.setAttribute("no-close-button",""),T(t,"dialogRef",c);let m=document.createElement("span");m.setAttribute("slot","header");let p=document.createElement("slot");p.setAttribute("name","title"),m.appendChild(p),c.appendChild(m);let d=document.createElement("slot");c.appendChild(d);let l=document.createElement("span");l.setAttribute("slot","footer");let i=document.createElement("sui-button");i.setAttribute("variant","outline"),o.push(K(i,"click",function(){let h=t.onCancel;return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let u=document.createTextNode("");i.appendChild(u),o.push(B(u,()=>t.cancelLabel())),l.appendChild(i);let s=document.createElement("sui-button");s.setAttribute("variant","solid"),o.push(E(s,"tone",()=>t.confirmTone())),o.push(K(s,"click",function(){let h=t.onConfirm;return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let f=document.createTextNode("");if(s.appendChild(f),o.push(B(f,()=>t.confirmLabel())),l.appendChild(s),c.appendChild(l),b.appendChild(c),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(g,x){return g[x]},set(g,x,k){let w=g[x];if(w!=null&&typeof w.set==="function"){if(typeof k==="function")w.set(()=>k);else w.set(k);return!0}return!1}});if(h.mount){let g=h.mount(v,r);if(typeof g==="function")o.push(g)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});function Jr(e){if(e<1024)return`${e} B`;if(e<1048576)return`${(e/1024).toFixed(1)} KB`;return`${(e/1048576).toFixed(1)} MB`}var kn=P({scope:()=>{let e=N([]),n=N(!1),r=N(""),o=J(null),t=J(null),a=J(null),b=Z(()=>e().map((c,m)=>({file:c,idx:m,key:`${m}:${c.name}:${c.size}`,name:c.name,sizeStr:Jr(c.size)})));return{files:e,over:n,errorMsg:r,displayFiles:b,dropzoneEl:o,inputEl:t,fileListEl:a}},mount(e,n){let r=e,o=r.files,t=r.over,a=r.errorMsg,b=(k)=>{let w=r.accept().split(",").map((C)=>C.trim().toLowerCase()).filter(Boolean);if(w.length===0)return!0;let A="."+(k.name.split(".").pop()||"").toLowerCase(),j=k.type.toLowerCase();return w.some((C)=>{if(C.startsWith("."))return A===C;if(C.endsWith("/*"))return j.startsWith(C.slice(0,-1));return j===C})},c=(k)=>{n.dispatchEvent(new CustomEvent("sui-change",{detail:{files:[...k]},bubbles:!0,composed:!0}))},m=(k)=>{a.set("");let w=[];for(let C of k){if(C.size>r.maxSize()){a.set(`"${C.name}" exceeds max size (${Jr(r.maxSize())}).`);continue}if(r.accept()&&!b(C)){a.set(`"${C.name}" is not an accepted file type.`);continue}w.push(C)}let A=o(),j=r.multiple()?[...A,...w]:w.slice(0,1);o.set(j),c(j)},p=(k)=>{let w=o();if(k<0||k>=w.length)return;let A=w.filter((j,C)=>C!==k);o.set(A),a.set(""),c(A)},d=()=>{if(r.disabled())return;r.inputEl()?.click()},l=()=>d(),i=(k)=>{if(r.disabled())return;if(k.key==="Enter"||k.key===" ")k.preventDefault(),d()},u=(k)=>{if(r.disabled())return;k.preventDefault(),t.set(!0)},s=()=>t.set(!1),f=(k)=>{if(r.disabled())return;k.preventDefault(),t.set(!1);let w=k.dataTransfer?.files;if(w&&w.length)m(Array.from(w))},h=(k)=>{let w=k.target;if(w.files&&w.files.length)m(Array.from(w.files));w.value=""},v=r.dropzoneEl(),g=r.inputEl();v?.addEventListener("click",l),v?.addEventListener("keydown",i),v?.addEventListener("dragover",u),v?.addEventListener("dragleave",s),v?.addEventListener("drop",f),g?.addEventListener("change",h);let x=(k)=>{let A=k.target?.closest?.("button.remove");if(!A||!n.shadowRoot?.contains(A))return;let j=Number(A.dataset.idx);if(Number.isInteger(j))p(j)};return n.shadowRoot?.addEventListener("click",x),()=>{v?.removeEventListener("click",l),v?.removeEventListener("keydown",i),v?.removeEventListener("dragover",u),v?.removeEventListener("dragleave",s),v?.removeEventListener("drop",f),g?.removeEventListener("change",h),n.shadowRoot?.removeEventListener("click",x)}}});var Ap=z("sui-file-upload",{props:{accept:y.string({default:""}),multiple:y.bool({default:!1}),"max-size":y.number({default:10485760}),disabled:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.accept=n.accept,t.multiple=n.multiple,t.maxSize=n.maxSize,t.disabled=n.disabled,t.tone=n.tone;let a=null;if(kn&&typeof kn.setup==="function"){if(e.scope=t,a=kn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(g)=>F({anchor:g,test:()=>{if(t.errorMsg())return 0;return-1},branches:[()=>{let x=document.createDocumentFragment(),k=[],w=document.createElement("div");w.setAttribute("class","error"),w.setAttribute("role","alert"),w.setAttribute("part","error");let A=document.createTextNode("");return w.appendChild(A),k.push(B(A,()=>t.errorMsg())),x.appendChild(w),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let j of k)try{j()}catch(C){}}}}]}),c=(g)=>R({anchor:g,items:()=>t.displayFiles(),key:(x,k)=>{let w=Object.create(t);return w.f=x,w.__i=k,((A)=>x.key)(w)},render:(x,k)=>{let w=Object.create(t);return w.f=x,w.__i=k,w.__v=N(0),((A)=>{let j=document.createDocumentFragment(),C=[],M=document.createElement("div");M.setAttribute("class","file-item"),M.setAttribute("part","file-item");let O=document.createElement("span");O.setAttribute("class","file-name");let H=document.createTextNode("");O.appendChild(H),C.push(B(H,()=>(A.__v(),A.f.name))),M.appendChild(O);let S=document.createElement("span");S.setAttribute("class","file-size");let U=document.createTextNode("");S.appendChild(U),C.push(B(U,()=>(A.__v(),A.f.sizeStr))),M.appendChild(S);let G=document.createElement("button");G.setAttribute("class","remove"),G.setAttribute("part","remove"),G.setAttribute("type","button"),C.push(E(G,"data-idx",()=>(A.__v(),A.f.idx))),C.push(E(G,"aria-label",()=>(A.__v(),"Remove "+A.f.name)));let X=document.createTextNode("×");return G.appendChild(X),M.appendChild(G),j.appendChild(M),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let W of C)try{W()}catch(Y){}},update:(W,Y)=>{A.f=W,A.__i=Y,A.__v.set((q)=>q+1)}}})(w)}}),m=(g)=>F({anchor:g,test:()=>{if(t.displayFiles().length>0)return 0;return-1},branches:[()=>{let x=document.createDocumentFragment(),k=[],w=document.createElement("div");w.setAttribute("class","file-list"),w.setAttribute("part","file-list"),w.setAttribute("aria-live","polite"),T(t,"fileListEl",w);let A=document.createComment("for");return w.appendChild(A),k.push(c(A)),x.appendChild(w),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let j of k)try{j()}catch(C){}}}}]}),p=r.shadowRoot??r;L(p);let d=document.createElement("div");d.setAttribute("class","dropzone"),o.push(pt(d,"over",()=>t.over())),d.setAttribute("part","dropzone"),d.setAttribute("role","button"),o.push(E(d,"tabindex",()=>t.disabled()?"-1":"0")),o.push(E(d,"aria-label",()=>t.accept()?"Upload files ("+t.accept()+")":"Upload files")),o.push(E(d,"aria-disabled",()=>t.disabled()?"true":"false")),T(t,"dropzoneEl",d);let l=document.createElement("span");l.setAttribute("class","icon"),l.setAttribute("aria-hidden","true");let i=document.createTextNode("\uD83D\uDCC4");l.appendChild(i),d.appendChild(l);let u=document.createElement("span"),s=document.createTextNode("Drop files here or click to browse");u.appendChild(s),d.appendChild(u),p.appendChild(d);let f=document.createElement("input");f.setAttribute("type","file"),f.setAttribute("part","input"),o.push(E(f,"accept",()=>t.accept())),o.push(E(f,"multiple",()=>t.multiple())),f.setAttribute("aria-hidden","true"),f.setAttribute("tabindex","-1"),T(t,"inputEl",f),p.appendChild(f);let h=document.createComment("if");p.appendChild(h),o.push(b(h));let v=document.createComment("if");if(p.appendChild(v),o.push(m(v)),a!=null){let g=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(g){let x=new Proxy(t,{get(k,w){return k[w]},set(k,w,A){let j=k[w];if(j!=null&&typeof j.set==="function"){if(typeof A==="function")j.set(()=>A);else j.set(A);return!0}return!1}});if(g.mount){let k=g.mount(x,r);if(typeof k==="function")o.push(k)}if(g.unmount)o.push(()=>g.unmount(x))}}return t}});var da=new Set(["info","success","warning","destructive","accent"]),En=P({scope:()=>{let e=N(!1),n=N(!0),r=J(null),o=J(null),t=J(null),a=J(null);return{hasSlottedIcon:e,visible:n,iconSlotEl:r,titleSlotEl:o,actionsSlotEl:t,closeBtn:a}},mount(e,n){let r=e,o=null,t=0,a=0,b=!1;if(!n.hasAttribute("tabindex"))n.tabIndex=0;function c(){if(o)clearTimeout(o),o=null}function m(){c();let A=r.duration();if(A>0)t=A,a=Date.now(),o=setTimeout(()=>p("timeout"),A),n.style.setProperty("--toast-duration",`${A}ms`)}function p(A="manual"){if(b)return;b=!0,c(),n.dispatchEvent(new CustomEvent("sui-dismiss",{detail:{reason:A},bubbles:!0,composed:!0})),n.remove()}async function d(A="manual"){if(b)return;if(typeof window<"u"&&window.matchMedia?.("(prefers-reduced-motion: reduce)").matches){p(A);return}c(),n.setAttribute("closing",""),r.visible.set(!1),await new Promise((M)=>setTimeout(M,200)),p(A)}let l=()=>{if(!o)return;let A=Date.now()-a;t=Math.max(0,t-A),c(),n.toggleAttribute("data-paused",!0)},i=()=>{if(b||r.duration()<=0)return;if(t<=0)return;n.toggleAttribute("data-paused",!1),a=Date.now(),o=setTimeout(()=>p("timeout"),t)},u=(A)=>{if(A.key==="Escape"&&r.dismissible())A.stopPropagation(),p("manual")};n.addEventListener("mouseenter",l),n.addEventListener("mouseleave",i),n.addEventListener("focusin",l),n.addEventListener("focusout",i),n.addEventListener("keydown",u),m();let s=r.hasSlottedIcon,f=(A)=>A.assignedNodes({flatten:!0}).some((j)=>{if(j.nodeType===Node.ELEMENT_NODE)return!0;return(j.textContent?.trim().length??0)>0}),h=r.iconSlotEl();if(h){let A=()=>s.set(f(h));h.addEventListener("slotchange",A),s.set(f(h))}let v=r.titleSlotEl(),g=r.actionsSlotEl();if(v){let A=()=>n.toggleAttribute("data-no-title",!f(v));v.addEventListener("slotchange",A),n.toggleAttribute("data-no-title",!f(v))}if(g){let A=()=>n.toggleAttribute("data-no-actions",!f(g));g.addEventListener("slotchange",A),n.toggleAttribute("data-no-actions",!f(g))}let x=D(()=>{let A=r.tone(),j=r.hasSlottedIcon(),C=da.has(A);n.toggleAttribute("data-no-icon",!C&&!j)}),k=D(()=>{let A=r.closeBtn();if(!A)return;let j=()=>p("manual");return A.addEventListener("click",j),()=>A.removeEventListener("click",j)}),w=n;return w.dismiss=p,w.close=d,()=>{b=!0,c(),n.removeEventListener("mouseenter",l),n.removeEventListener("mouseleave",i),n.removeEventListener("focusin",l),n.removeEventListener("focusout",i),n.removeEventListener("keydown",u),x(),k()}}});var Lp=z("sui-toast",{props:{tone:y.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),duration:y.number({default:5000}),dismissible:y.bool({default:!0,reflect:!0}),"show-progress":y.bool({default:!1,reflect:!0}),closing:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.size=n.size,t.duration=n.duration,t.dismissible=n.dismissible,t.showProgress=n.showProgress,t.closing=n.closing;let a=null;if(En&&typeof En.setup==="function"){if(e.scope=t,a=En.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>F({anchor:u,test:()=>{let s=t.tone();if(s==="info")return 0;if(s==="success")return 1;if(s==="warning")return 2;if(s==="destructive")return 3;if(s==="accent")return 4;return-1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("circle");v.setAttribute("cx","12"),v.setAttribute("cy","12"),v.setAttribute("r","9"),h.appendChild(v);let g=document.createElement("path");g.setAttribute("d","M12 8h.01"),h.appendChild(g);let x=document.createElement("path");return x.setAttribute("d","M11 12h1v4h1"),h.appendChild(x),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let k of f)try{k()}catch(w){}}}},()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("circle");v.setAttribute("cx","12"),v.setAttribute("cy","12"),v.setAttribute("r","9"),h.appendChild(v);let g=document.createElement("path");return g.setAttribute("d","m8.5 12.5 2.5 2.5 4.5-5"),h.appendChild(g),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let x of f)try{x()}catch(k){}}}},()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("path");v.setAttribute("d","M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"),h.appendChild(v);let g=document.createElement("line");g.setAttribute("x1","12"),g.setAttribute("x2","12"),g.setAttribute("y1","9"),g.setAttribute("y2","13"),h.appendChild(g);let x=document.createElement("line");return x.setAttribute("x1","12"),x.setAttribute("x2","12.01"),x.setAttribute("y1","17"),x.setAttribute("y2","17"),h.appendChild(x),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let k of f)try{k()}catch(w){}}}},()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("circle");v.setAttribute("cx","12"),v.setAttribute("cy","12"),v.setAttribute("r","9"),h.appendChild(v);let g=document.createElement("line");g.setAttribute("x1","15"),g.setAttribute("x2","9"),g.setAttribute("y1","9"),g.setAttribute("y2","15"),h.appendChild(g);let x=document.createElement("line");return x.setAttribute("x1","9"),x.setAttribute("x2","15"),x.setAttribute("y1","9"),x.setAttribute("y2","15"),h.appendChild(x),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let k of f)try{k()}catch(w){}}}},()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("path");return v.setAttribute("d","M12 2 3 14h7l-1 8 10-12h-7l1-8Z"),h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let g of f)try{g()}catch(x){}}}}]}),c=(u)=>F({anchor:u,test:()=>{if(!t.hasSlottedIcon())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createComment("match");return s.appendChild(h),f.push(b(h)),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let v of f)try{v()}catch(g){}}}}]}),m=(u)=>F({anchor:u,test:()=>{if(t.dismissible())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("button");h.setAttribute("class","close"),h.setAttribute("part","close"),h.setAttribute("type","button"),h.setAttribute("aria-label","Dismiss notification"),T(t,"closeBtn",h);let v=document.createElement("svg");v.setAttribute("viewbox","0 0 24 24"),v.setAttribute("fill","none"),v.setAttribute("stroke","currentColor"),v.setAttribute("stroke-width","2"),v.setAttribute("stroke-linecap","round"),v.setAttribute("stroke-linejoin","round"),v.setAttribute("aria-hidden","true");let g=document.createElement("line");g.setAttribute("x1","18"),g.setAttribute("y1","6"),g.setAttribute("x2","6"),g.setAttribute("y2","18"),v.appendChild(g);let x=document.createElement("line");return x.setAttribute("x1","6"),x.setAttribute("y1","6"),x.setAttribute("x2","18"),x.setAttribute("y2","18"),v.appendChild(x),h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let k of f)try{k()}catch(w){}}}}]}),p=(u)=>F({anchor:u,test:()=>{if(t.showProgress()&&t.duration()>0)return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("span");return h.setAttribute("class","progress"),h.setAttribute("aria-hidden","true"),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let v of f)try{v()}catch(g){}}}}]}),d=(u)=>F({anchor:u,test:()=>{if(t.visible())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("div");h.setAttribute("class","toast"),ot(h,Re,{axis:"y",duration:200},"in"),ot(h,ht,{duration:180},"out"),f.push(E(h,"role",()=>t.tone()==="destructive"||t.tone()==="warning"?"alert":"status")),f.push(E(h,"aria-live",()=>t.tone()==="destructive"||t.tone()==="warning"?"assertive":"polite")),h.setAttribute("aria-atomic","true"),h.setAttribute("part","toast");let v=document.createElement("span");v.setAttribute("class","icon");let g=document.createElement("slot");g.setAttribute("name","icon"),T(t,"iconSlotEl",g),v.appendChild(g);let x=document.createComment("if");v.appendChild(x),f.push(c(x)),h.appendChild(v);let k=document.createElement("div");k.setAttribute("class","content");let w=document.createElement("div");w.setAttribute("class","title");let A=document.createElement("slot");A.setAttribute("name","title"),T(t,"titleSlotEl",A),w.appendChild(A),k.appendChild(w);let j=document.createElement("div");j.setAttribute("class","body");let C=document.createElement("slot");j.appendChild(C),k.appendChild(j);let M=document.createElement("div");M.setAttribute("class","actions");let O=document.createElement("slot");O.setAttribute("name","actions"),T(t,"actionsSlotEl",O),M.appendChild(O),k.appendChild(M),h.appendChild(k);let H=document.createComment("if");h.appendChild(H),f.push(m(H));let S=document.createComment("if");return h.appendChild(S),f.push(p(S)),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let U of f)try{U()}catch(G){}}}}]}),l=r.shadowRoot??r;L(l);let i=document.createComment("if");if(l.appendChild(i),o.push(d(i)),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(f,h){return f[h]},set(f,h,v){let g=f[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(u.mount){let f=u.mount(s,r);if(typeof f==="function")o.push(f)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var ca={"top-left":"top","top-center":"top","top-right":"top","bottom-left":"bottom","bottom-center":"bottom","bottom-right":"bottom"},it="16px",jt="position:fixed;display:flex;gap:var(--space-2,8px);pointer-events:none;z-index:var(--z-toast,9999);max-width:min(440px,calc(100vw - 32px));padding:0;margin:0;box-sizing:border-box;";function pa(e){switch(e){case"top-right":return`${jt}top:${it};right:${it};flex-direction:column;align-items:flex-end;`;case"top-left":return`${jt}top:${it};left:${it};flex-direction:column;align-items:flex-start;`;case"top-center":return`${jt}top:${it};left:50%;transform:translateX(-50%);flex-direction:column;align-items:center;`;case"bottom-left":return`${jt}bottom:${it};left:${it};flex-direction:column-reverse;align-items:flex-start;`;case"bottom-center":return`${jt}bottom:${it};left:50%;transform:translateX(-50%);flex-direction:column-reverse;align-items:center;`;case"bottom-right":default:return`${jt}bottom:${it};right:${it};flex-direction:column-reverse;align-items:flex-end;`}}var Cn=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","region");if(!n.hasAttribute("aria-label"))n.setAttribute("aria-label","Notifications");if(!n.hasAttribute("aria-live"))n.setAttribute("aria-live","polite");let o=N(0),t=r.defaultSlotEl(),a=r.stackEl(),b=(s)=>{if(!a)return;if(s.parentElement!==a)a.appendChild(s)},c=()=>{if(!a)return[];return Array.from(a.children).filter((s)=>s.tagName.toLowerCase()==="sui-toast")},m=D(()=>{if(!a)return;let s=r.position();a.style.cssText=pa(s),a.setAttribute("data-position",s)}),p=D(()=>{o();let s=r.position(),f=r.maxVisible(),h=ca[s]??"bottom",v=c();for(let g of v)g.setAttribute("data-slide-from",h),g.style.pointerEvents="auto";if(f>0&&v.length>f){let g=v.length-f;for(let x=0;x<g;x++){let k=v[x];if(typeof k.close==="function")k.close("programmatic");else k.dismiss?.("programmatic")}}}),d=()=>{if(t){for(let s of t.assignedElements())if(s.tagName.toLowerCase()==="sui-toast")b(s)}o.set(o()+1)};if(t)t.addEventListener("slotchange",d),d();let l=new MutationObserver((s)=>{let f=!1;for(let h of s){for(let v of h.addedNodes)if(v.nodeType===Node.ELEMENT_NODE){let g=v;if(g.tagName.toLowerCase()==="sui-toast")b(g),f=!0}if(h.removedNodes.length)f=!0}if(f)o.set(o()+1)});l.observe(n,{childList:!0});for(let s of Array.from(n.children))if(s.tagName.toLowerCase()==="sui-toast")b(s);o.set(o()+1);let i=a?new MutationObserver(()=>o.set(o()+1)):null;if(a&&i)i.observe(a,{childList:!0});function u(s={}){let f=n.ownerDocument??document,h=f.createElement("sui-toast");if(s.tone)h.setAttribute("tone",s.tone);if(s.size)h.setAttribute("size",s.size);if(typeof s.duration==="number")h.setAttribute("duration",String(s.duration));if(s.dismissible===!1)h.removeAttribute("dismissible");else h.setAttribute("dismissible","");if(s.showProgress)h.setAttribute("show-progress","");if(s.title){let v=f.createElement("span");v.setAttribute("slot","title"),v.textContent=s.title,h.appendChild(v)}if(s.description)h.appendChild(f.createTextNode(s.description));if(s.actionHtml){let v=f.createElement("span");v.setAttribute("slot","actions");let g=s.actionHtml;et(v,()=>g),h.appendChild(v)}if(a)a.appendChild(h);else n.appendChild(h);return h}return n.show=u,()=>{if(p(),m(),l.disconnect(),i?.disconnect(),t)t.removeEventListener("slotchange",d)}}});var Np=z("sui-toast-container",{props:{position:y.union(["top-left","top-center","top-right","bottom-left","bottom-center","bottom-right"],{default:"bottom-right",reflect:!0}),"max-visible":y.number({default:5,reflect:!0})},shadow:!0,styles:`
  /* Host collapses — the visible stack lives in body via <sui-portal>
     (see template). The host stays minimal so it can sit anywhere in
     the document without taking layout space. */
  :host { display: contents; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.position=n.position,t.maxVisible=n.maxVisible;let a=null;if(Cn&&typeof Cn.setup==="function"){if(e.scope=t,a=Cn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("sui-portal");c.setAttribute("target","body");let m=document.createElement("div");m.setAttribute("class","sui-toast-stack"),m.setAttribute("data-sui-toast-stack",""),T(t,"stackEl",m),c.appendChild(m),b.appendChild(c);let p=document.createElement("slot");if(T(t,"defaultSlotEl",p),b.appendChild(p),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var jn=P({scope:()=>{let e=N([]),n=N("");return{items:e,lastSlot:n}},mount(e,n){let r=e,o=r.items,t=r.lastSlot,a=()=>{let m=Array.from(n.querySelectorAll(":scope > sui-breadcrumb-item")).map((p,d)=>{let l=`i${d}`;return p.setAttribute("slot",l),{slot:l}});t.set(m.length>0?m[m.length-1].slot:""),o.set(m)};a();let b=new MutationObserver(a);return b.observe(n,{childList:!0}),()=>b.disconnect()}});var Up=z("sui-breadcrumb",{props:{separator:y.string({default:"/",reflect:!0})},shadow:!0,styles:`
  :host { display: flex; align-items: center; }
  nav { display: inline-flex; align-items: center; flex-wrap: wrap; gap: 2px; }
  .sep {
    display: inline-flex; align-items: center; justify-content: center;
    width: 12px; height: 12px;
    color: var(--muted-foreground);
    opacity: 0.7;
  }
  .sep svg { width: 12px; height: 12px; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.separator=n.separator;let a=null;if(jn&&typeof jn.setup==="function"){if(e.scope=t,a=jn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>R({anchor:d,items:()=>t.items(),key:(l,i)=>{let u=Object.create(t);return u.item=l,u.__i=i,((s)=>l.slot)(u)},render:(l,i)=>{let u=Object.create(t);return u.item=l,u.__i=i,u.__v=N(0),((s)=>{let f=(w)=>F({anchor:w,test:()=>{if(s.separator()==="/")return 0;return 1},branches:[()=>{let A=document.createDocumentFragment(),j=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round");let M=document.createElement("path");return M.setAttribute("d","M9 6l6 6-6 6"),C.appendChild(M),A.appendChild(C),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let O of j)try{O()}catch(H){}}}},()=>{let A=document.createDocumentFragment(),j=[],C=document.createTextNode("");return A.appendChild(C),j.push(B(C,()=>(s.__v(),s.separator()))),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let M of j)try{M()}catch(O){}}}}]}),h=(w)=>F({anchor:w,test:()=>{if(s.item.slot!==s.lastSlot())return 0;return-1},branches:[()=>{let A=document.createDocumentFragment(),j=[],C=document.createElement("span");C.setAttribute("class","sep"),C.setAttribute("aria-hidden","true");let M=document.createComment("if");return C.appendChild(M),j.push(f(M)),A.appendChild(C),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let O of j)try{O()}catch(H){}}}}]}),v=document.createDocumentFragment(),g=[],x=document.createElement("slot");g.push(E(x,"name",()=>(s.__v(),s.item.slot))),v.appendChild(x);let k=document.createComment("if");return v.appendChild(k),g.push(h(k)),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let w of g)try{w()}catch(A){}},update:(w,A)=>{s.item=w,s.__i=A,s.__v.set((j)=>j+1)}}})(u)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("nav");m.setAttribute("aria-label","Breadcrumb"),m.setAttribute("part","nav");let p=document.createComment("for");if(m.appendChild(p),o.push(b(p)),c.appendChild(m),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var Rp=z("sui-breadcrumb-item",{props:{href:y.string({default:"",reflect:!0}),active:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.href=n.href,t.active=n.active;let a=(m)=>F({anchor:m,test:()=>{if(t.href()&&!t.active())return 0;return 1},branches:[()=>{let p=document.createDocumentFragment(),d=[],l=document.createElement("a");d.push(E(l,"href",()=>t.href())),l.setAttribute("part","link");let i=document.createElement("slot");return l.appendChild(i),p.appendChild(l),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let u of d)try{u()}catch(s){}}}},()=>{let p=document.createDocumentFragment(),d=[],l=document.createElement("span");l.setAttribute("class","label"),l.setAttribute("part","label"),d.push(E(l,"aria-current",()=>t.active()?"page":"false"));let i=document.createElement("slot");return l.appendChild(i),p.appendChild(l),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let u of d)try{u()}catch(s){}}}}]}),b=r.shadowRoot??r;L(b);let c=document.createComment("if");return b.appendChild(c),o.push(a(c)),t}});var zn=P({scope:()=>({totalPages:N(1),info:N(""),prevBtn:J(null),nextBtn:J(null)}),mount(e,n){let r=e,o=D(()=>{let m=r.page(),p=r.total(),d=r.perPage(),l=p>0?Math.ceil(p/d):1;if(r.totalPages.set(l),p>0){let i=(m-1)*d+1,u=Math.min(m*d,p);r.info.set(`Page ${m} of ${l} (${i}–${u} of ${p})`)}else r.info.set(`Page ${m} of ${l}`)}),t=r.prevBtn(),a=r.nextBtn(),b=()=>{let m=r.page();if(m>1){let p=m-1;r.page.set(p),n.dispatchEvent(new CustomEvent("sui-page",{detail:{page:p},bubbles:!0,composed:!0}))}},c=()=>{let m=r.page(),p=r.totalPages();if(m<p){let d=m+1;r.page.set(d),n.dispatchEvent(new CustomEvent("sui-page",{detail:{page:d},bubbles:!0,composed:!0}))}};return t?.addEventListener("click",b),a?.addEventListener("click",c),()=>{o(),t?.removeEventListener("click",b),a?.removeEventListener("click",c)}}});var Vp=z("sui-pagination",{props:{page:y.number({default:1,reflect:!0}),total:y.number({default:0,reflect:!0}),"per-page":y.number({default:25,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.page=n.page,t.total=n.total,t.perPage=n.perPage;let a=null;if(zn&&typeof zn.setup==="function"){if(e.scope=t,a=zn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("nav");c.setAttribute("class","bar"),c.setAttribute("aria-label","Pagination");let m=document.createElement("button");T(t,"prevBtn",m),m.setAttribute("class","pg-btn"),m.setAttribute("part","prev"),o.push(E(m,"disabled",()=>t.page()<=1)),m.setAttribute("aria-label","Previous page"),o.push(E(m,"aria-disabled",()=>t.page()<=1?"true":"false"));let p=document.createTextNode("Prev");m.appendChild(p),c.appendChild(m);let d=document.createElement("span");d.setAttribute("class","info"),d.setAttribute("part","info");let l=document.createTextNode("");d.appendChild(l),o.push(B(l,()=>t.info())),c.appendChild(d);let i=document.createElement("span");i.setAttribute("class","spacer"),c.appendChild(i);let u=document.createElement("button");T(t,"nextBtn",u),u.setAttribute("class","pg-btn"),u.setAttribute("part","next"),o.push(E(u,"disabled",()=>t.page()>=t.totalPages())),u.setAttribute("aria-label","Next page"),o.push(E(u,"aria-disabled",()=>t.page()>=t.totalPages()?"true":"false"));let s=document.createTextNode("Next");if(u.appendChild(s),c.appendChild(u),b.appendChild(c),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let h=new Proxy(t,{get(v,g){return v[g]},set(v,g,x){let k=v[g];if(k!=null&&typeof k.set==="function"){if(typeof x==="function")k.set(()=>x);else k.set(x);return!0}return!1}});if(f.mount){let v=f.mount(h,r);if(typeof v==="function")o.push(v)}if(f.unmount)o.push(()=>f.unmount(h))}}return t}});function Yr(e,n=Date.now()){let r=Date.parse(e);if(Number.isNaN(r))return e;let o=n-r,t=Math.abs(o),a=o>=0?"ago":"from now";if(t<60000)return"just now";if(t<3600000)return`${Math.floor(t/60000)}m ${a}`;if(t<86400000)return`${Math.floor(t/3600000)}h ${a}`;let b=Math.floor(t/86400000);if(b<30)return`${b}d ${a}`;if(b<365)return`${Math.floor(b/30)}mo ${a}`;return`${Math.floor(b/365)}y ${a}`}function Wr(e){let n=Date.parse(e);return Number.isNaN(n)?e:new Date(n).toISOString()}var fa=60000,Ln=P({scope:({value:e})=>{let n=N(0),r=Z(()=>{let t=e();return t?Wr(t):""}),o=Z(()=>{n();let t=e();return t?Yr(t):""});return{tick:n,absStr:r,relStr:o}},mount(e,n){let r=e,o=r.tick,t=D(()=>{if(r.value())n.title=r.absStr()}),a=setInterval(()=>o.set(o()+1),fa);return()=>{t(),clearInterval(a)}}});var rf=z("sui-timestamp",{props:{value:y.string({default:"",reflect:!0}),format:y.union(["relative","absolute","both"],{default:"relative",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline;
    font-family: var(--font-body);
    font-size: inherit;
    color: inherit;
  }
  .abs { color: var(--muted-foreground); margin-left: 4px; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.format=n.format;let a=null;if(Ln&&typeof Ln.setup==="function"){if(e.scope=t,a=Ln.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(l)=>F({anchor:l,test:()=>{if(t.format()==="absolute")return 0;return 1},branches:[()=>{let i=document.createDocumentFragment(),u=[],s=document.createTextNode("");return i.appendChild(s),u.push(B(s,()=>t.absStr())),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let f of u)try{f()}catch(h){}}}},()=>{let i=document.createDocumentFragment(),u=[],s=document.createTextNode("");return i.appendChild(s),u.push(B(s,()=>t.relStr())),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let f of u)try{f()}catch(h){}}}}]}),c=(l)=>F({anchor:l,test:()=>{if(t.format()==="both")return 0;return-1},branches:[()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("span");s.setAttribute("class","abs"),s.setAttribute("part","abs");let f=document.createTextNode("(");s.appendChild(f);let h=document.createTextNode("");s.appendChild(h),u.push(B(h,()=>t.absStr()));let v=document.createTextNode(")");return s.appendChild(v),i.appendChild(s),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let g of u)try{g()}catch(x){}}}}]}),m=(l)=>F({anchor:l,test:()=>{if(t.value())return 0;return-1},branches:[()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("time");u.push(E(s,"datetime",()=>t.value())),s.setAttribute("part","time");let f=document.createComment("if");s.appendChild(f),u.push(b(f)),i.appendChild(s);let h=document.createComment("if");return i.appendChild(h),u.push(c(h)),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let v of u)try{v()}catch(g){}}}}]}),p=r.shadowRoot??r;L(p);let d=document.createComment("if");if(p.appendChild(d),o.push(m(d)),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(u,s){return u[s]},set(u,s,f){let h=u[s];if(h!=null&&typeof h.set==="function"){if(typeof f==="function")h.set(()=>f);else h.set(f);return!0}return!1}});if(l.mount){let u=l.mount(i,r);if(typeof u==="function")o.push(u)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var sf=z("sui-metachip",{props:{label:y.string({default:"",reflect:!0}),value:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.label=n.label,t.value=n.value;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");b.setAttribute("class","k"),b.setAttribute("part","label");let c=document.createElement("slot");c.setAttribute("name","label");let m=document.createTextNode("");c.appendChild(m),o.push(B(m,()=>t.label())),b.appendChild(c),a.appendChild(b);let p=document.createElement("span");p.setAttribute("class","v"),p.setAttribute("part","value");let d=document.createElement("slot"),l=document.createTextNode("");return d.appendChild(l),o.push(B(l,()=>t.value())),p.appendChild(d),a.appendChild(p),t}});var ma=0,Zr=typeof CSS<"u"&&typeof CSS.supports==="function"&&CSS.supports("anchor-name","--x"),Dn=P({mount(e,n){let r=e,o=++ma,t=`--sui-popover-${o}`,a=`sui-popover-panel-${o}`;n.style.setProperty("--sui-popover-anchor",t);let b=r.triggerSlot(),c=r.panelEl();if(!c)return;c.id=a,Ue(r.triggerSlot,(i)=>{for(let u of i)u.setAttribute("popovertarget",a),u.setAttribute("aria-haspopup","dialog"),u.setAttribute("aria-controls",a),u.setAttribute("aria-expanded",r.open()?"true":"false")});function m(i){let s=i.newState==="open",f=n.hasAttribute("open");if(s&&!f)n.setAttribute("open","");else if(!s&&f)n.removeAttribute("open");let v=b?.assignedElements({flatten:!0})??[];for(let g of v)g.setAttribute("aria-expanded",s?"true":"false");if(n.emit("sui-toggle",{open:s}),s&&!Zr)d()}Q(c,"toggle",m),n.expose({show:()=>c.showPopover?.(),hide:()=>c.hidePopover?.(),toggle:()=>typeof c.togglePopover==="function"?void c.togglePopover():c.matches?.(":popover-open")?c.hidePopover?.():c.showPopover?.()});let p=D(()=>{let i=r.open(),u=c.matches?.(":popover-open")??!1;if(i&&!u)c.showPopover?.();else if(!i&&u)c.hidePopover?.()});function d(){let s=(b?.assignedElements({flatten:!0})??[])[0];if(!s||!c)return;let f=s.getBoundingClientRect(),h=r.placement();c.style.top="",c.style.left="",c.style.right="",c.style.bottom="";let v=6;switch(h){case"top":c.style.bottom=`${window.innerHeight-f.top+v}px`,c.style.left=`${f.left}px`;break;case"top-end":c.style.bottom=`${window.innerHeight-f.top+v}px`,c.style.right=`${window.innerWidth-f.right}px`;break;case"bottom-end":c.style.top=`${f.bottom+v}px`,c.style.right=`${window.innerWidth-f.right}px`;break;case"left":c.style.right=`${window.innerWidth-f.left+v}px`,c.style.top=`${f.top}px`;break;case"left-end":c.style.right=`${window.innerWidth-f.left+v}px`,c.style.bottom=`${window.innerHeight-f.bottom}px`;break;case"right":c.style.left=`${f.right+v}px`,c.style.top=`${f.top}px`;break;case"right-end":c.style.left=`${f.right+v}px`,c.style.bottom=`${window.innerHeight-f.bottom}px`;break;case"bottom":default:c.style.top=`${f.bottom+v}px`,c.style.left=`${f.left}px`}}function l(){if(!r.open())return;d()}if(!Zr)Q(window,"resize",l),Q(window,"scroll",l,{capture:!0,passive:!0});return()=>{p()}}});var ff=z("sui-popover",{props:{open:y.bool({default:!1,reflect:!0}),placement:y.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.placement=n.placement;let a=null;if(Dn&&typeof Dn.setup==="function"){if(e.scope=t,a=Dn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>F({anchor:u,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("div");h.setAttribute("class","surface"),ot(h,Je,{duration:120},"both");let v=document.createElement("slot");return h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let g of f)try{g()}catch(x){}}}}]}),c=(u)=>F({anchor:u,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("sui-portal");h.setAttribute("target","body");let v=document.createElement("div");return v.setAttribute("class","sui-popover-presence"),v.setAttribute("aria-hidden","true"),v.setAttribute("data-sui-popover-open",""),h.appendChild(v),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let g of f)try{g()}catch(x){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createElement("slot");p.setAttribute("name","trigger"),T(t,"triggerSlot",p),m.appendChild(p);let d=document.createElement("div");d.setAttribute("class","panel"),d.setAttribute("part","panel"),d.setAttribute("popover","auto"),T(t,"panelEl",d),d.setAttribute("role","dialog"),d.setAttribute("aria-modal","false");let l=document.createComment("if");d.appendChild(l),o.push(b(l)),m.appendChild(d);let i=document.createComment("if");if(m.appendChild(i),o.push(c(i)),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(f,h){return f[h]},set(f,h,v){let g=f[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(u.mount){let f=u.mount(s,r);if(typeof f==="function")o.push(f)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var nr=[{id:"ship-light",label:"SHIP Light",dark:!1,swatches:["oklch(0.99 0.001 106.423)","oklch(1 0 0)","oklch(0.67 0.16 58)","oklch(0.61 0.23 303)"]},{id:"ship-dark",label:"SHIP Dark",dark:!0,swatches:["oklch(0.18 0.01 250)","oklch(0.18 0.01 50)","oklch(0.77 0.16 70)","oklch(0.3 0.05 300)"]},{id:"paper",label:"Paper",dark:!1,swatches:["oklch(0.97 0.014 80)","oklch(0.99 0.010 80)","oklch(0.55 0.14 35)","oklch(0.48 0.09 55)"]},{id:"halo",label:"Halo (Soft)",dark:!1,swatches:["oklch(0.92 0.008 230)","oklch(0.92 0.008 230)","oklch(0.62 0.14 50)","oklch(0.55 0.05 230)"]},{id:"console",label:"Console",dark:!0,swatches:["oklch(0.13 0.008 220)","oklch(0.16 0.010 220)","oklch(0.78 0.20 145)","oklch(0.78 0.16 195)"]},{id:"contrast",label:"High Contrast",dark:!0,swatches:["oklch(0.09 0 0)","oklch(0.15 0 0)","oklch(0.87 0.18 85)","oklch(0.99 0 0)"]},{id:"glass",label:"Glass",dark:!1,swatches:["oklch(0.95 0.04 230)","oklch(0.99 0.005 240)","oklch(0.62 0.16 50)","oklch(0.62 0.14 280)"]}],Qr=Object.fromEntries(nr.map((e)=>[e.id,e]));var On=(e)=>typeof e==="string"&&(e in Qr),ba=(e)=>Qr[e]?.dark??!1;function er(e,n=document.documentElement){let r=On(e)?e:"ship-light",o=ba(r);if(n.dataset.theme=r,n.classList.toggle("dark",o),typeof document<"u"&&typeof CustomEvent<"u")document.dispatchEvent(new CustomEvent("ship:theme:change",{detail:{id:r,dark:o}}));return r}function rr(){if(typeof localStorage>"u")return"ship-light";try{let e=localStorage.getItem("ship-site-theme");return On(e)?e:"ship-light"}catch{return"ship-light"}}function Vr(e){if(typeof localStorage>"u")return;try{localStorage.setItem("ship-site-theme",e)}catch{}try{fetch("/api/settings",{method:"PUT",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:"shell.theme",value:e})}).catch(()=>{})}catch{}}var ha=[{label:"Default",ids:["ship-light","ship-dark"]},{label:"Texture",ids:["paper","halo"]},{label:"Effects",ids:["glass"]},{label:"Special",ids:["console","contrast"]}],Tn=new Map(nr.map((e)=>[e.id,e]));function va(e){let n=e.shadowRoot;if(!n)return;if(n.querySelector("link[data-sui-theme-preview-styles]"))return;let r=document.createElement("link");r.rel="stylesheet",r.href="/api/assets/view-tokens.css",r.setAttribute("data-sui-theme-preview-styles",""),n.appendChild(r)}var Pn=P({scope:(e)=>{let n=e,r=N(rr()),o=N(!1),t=N(ha.map((i)=>({label:i.label,themes:i.ids.map((u)=>Tn.get(u)).filter((u)=>!!u)}))),a=Z(()=>Tn.get(r())?.swatches[2]??"var(--primary, #3b82f6)"),b=Z(()=>Tn.get(r())?.label??"Theme"),c={host:null,panel:null};function m(){let i=c.panel;if(!i)return;if(o())i.hidePopover?.();else i.showPopover?.()}function p(i){let u=er(i);if(n.persist())Vr(u);r.set(u);let s=c.host;if(s)s.dispatchEvent(new CustomEvent("sui-theme-change",{detail:{id:u,dark:Tn.get(u)?.dark??!1},bubbles:!0,composed:!0}));c.panel?.hidePopover?.()}let d=J(null),l=J(null);return{activeTheme:r,panelOpen:o,groups:t,dotColor:a,triggerLabel:b,togglePanel:m,selectTheme:p,triggerBtn:d,panelEl:l,_internals:c}},mount(e,n){let r=e;r._internals.host=n,va(n),er(rr());let o=D(()=>{let p=r.panelEl();if(r._internals.panel=p,!p)return;let d=(l)=>{let u=l.newState==="open";if(r.panelOpen()!==u)r.panelOpen.set(u);if(u)t()};return p.addEventListener("toggle",d),()=>p.removeEventListener("toggle",d)});function t(){let p=r.panelEl(),d=r.triggerBtn();if(!p||!d)return;let l=d.getBoundingClientRect();p.style.top=`${l.bottom+4}px`,p.style.left=`${Math.max(8,l.right-p.offsetWidth)}px`}function a(p){let d=p.detail?.id;if(!On(d))return;if(r.activeTheme()!==d)r.activeTheme.set(d)}document.addEventListener("ship:theme:change",a);function b(p){let d=r.panelEl();if(!d)return;let l=Array.from(d.querySelectorAll(".card"));if(!l.length)return;let i=d.getRootNode(),u=l.findIndex((s)=>s===i.activeElement);switch(p.key){case"ArrowDown":p.preventDefault(),l[(u+1+l.length)%l.length]?.focus();break;case"ArrowUp":p.preventDefault(),l[(u-1+l.length)%l.length]?.focus();break;case"Home":p.preventDefault(),l[0]?.focus();break;case"End":p.preventDefault(),l[l.length-1]?.focus();break;case"Escape":p.preventDefault(),d.hidePopover?.();break}}let c=D(()=>{let p=r.panelEl();if(!p)return;return p.addEventListener("keydown",b),()=>p.removeEventListener("keydown",b)}),m=D(()=>{if(!r.panelOpen())return;requestAnimationFrame(()=>{let p=r.panelEl();if(!p)return;let d=Array.from(p.querySelectorAll(".card"));(d.find((i)=>i.getAttribute("aria-selected")==="true")??d[0])?.focus()})});return()=>{document.removeEventListener("ship:theme:change",a),o(),c(),m()}}});var wf=z("sui-theme-picker",{props:{placement:y.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom-end",reflect:!0}),"label-style":y.union(["text","compact"],{default:"text",reflect:!0}),persist:y.bool({default:!0,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.placement=n.placement,t.labelStyle=n.labelStyle,t.persist=n.persist;let a=null;if(Pn&&typeof Pn.setup==="function"){if(e.scope=t,a=Pn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(v)=>R({anchor:v,items:()=>t.groups(),key:(g,x)=>{let k=Object.create(t);return k.g=g,k.__i=x,((w)=>g.label)(k)},render:(g,x)=>{let k=Object.create(t);return k.g=g,k.__i=x,k.__v=N(0),((w)=>{let A=(G)=>R({anchor:G,items:()=>w.g.themes,key:(X,W)=>{let Y=Object.create(w);return Y.t=X,Y.__i=W,((q)=>X.id)(Y)},render:(X,W)=>{let Y=Object.create(w);return Y.t=X,Y.__i=W,Y.__v=N(0),((q)=>{let _=document.createDocumentFragment(),V=[],I=document.createElement("button");I.setAttribute("class","card"),I.setAttribute("role","option"),I.setAttribute("type","button"),V.push(E(I,"data-theme-card",()=>(q.__v(),q.t.id))),V.push(E(I,"aria-label",()=>(q.__v(),q.t.label))),V.push(E(I,"aria-selected",()=>(q.__v(),q.activeTheme()===q.t.id?"true":"false"))),V.push(E(I,"tabindex",()=>(q.__v(),q.activeTheme()===q.t.id?"0":"-1"))),V.push(K(I,"click",function(){let gt=()=>q.selectTheme(q.t.id);return typeof gt==="function"?gt.bind(void 0):()=>{}}(),{modifiers:[]}));let dt=document.createElement("span");dt.setAttribute("class","tc-bg"),V.push(E(dt,"data-theme-preview",()=>(q.__v(),q.t.id))),dt.setAttribute("aria-hidden","true");let lr=document.createElement("span");lr.setAttribute("class","tc-overlay"),dt.appendChild(lr);let Dt=document.createElement("span");Dt.setAttribute("class","tc-card");let Ut=document.createElement("span");Ut.setAttribute("class","tc-card-head");let Ee=document.createElement("span");Ee.setAttribute("class","tc-eyebrow");let mo=document.createTextNode("THEME");Ee.appendChild(mo),Ut.appendChild(Ee);let Ce=document.createElement("span");Ce.setAttribute("class","tc-heading");let dr=document.createTextNode("");Ce.appendChild(dr),V.push(B(dr,()=>(q.__v(),q.t.label))),Ut.appendChild(Ce),Dt.appendChild(Ut);let Xt=document.createElement("span");Xt.setAttribute("class","tc-typo");let je=document.createElement("span");je.setAttribute("class","tc-body");let bo=document.createTextNode("The quick brown fox");je.appendChild(bo),Xt.appendChild(je);let ze=document.createElement("span");ze.setAttribute("class","tc-mono");let cr=document.createTextNode("");ze.appendChild(cr),V.push(B(cr,()=>(q.__v(),"{ build: ok }"))),Xt.appendChild(ze),Dt.appendChild(Xt);let vt=document.createElement("span");vt.setAttribute("class","tc-controls");let Le=document.createElement("span");Le.setAttribute("class","tc-cta");let ho=document.createTextNode("Action");Le.appendChild(ho),vt.appendChild(Le);let De=document.createElement("span");De.setAttribute("class","tc-ghost");let vo=document.createTextNode("Cancel");De.appendChild(vo),vt.appendChild(De);let Oe=document.createElement("span");Oe.setAttribute("class","tc-chip tc-chip-accent");let go=document.createTextNode("New");Oe.appendChild(go),vt.appendChild(Oe);let Te=document.createElement("span");Te.setAttribute("class","tc-chip tc-chip-muted");let yo=document.createTextNode("Idle");Te.appendChild(yo),vt.appendChild(Te),Dt.appendChild(vt),dt.appendChild(Dt),I.appendChild(dt);let qt=document.createElement("span");qt.setAttribute("class","row");let Pe=document.createElement("span");Pe.setAttribute("class","name");let pr=document.createTextNode("");Pe.appendChild(pr),V.push(B(pr,()=>(q.__v(),q.t.label))),qt.appendChild(Pe);let Me=document.createElement("span");return Me.setAttribute("class","radio"),Me.setAttribute("aria-hidden","true"),qt.appendChild(Me),I.appendChild(qt),_.appendChild(I),{nodes:Array.from(_.childNodes),cleanup:()=>{for(let gt of V)try{gt()}catch(fr){}},update:(gt,fr)=>{q.t=gt,q.__i=fr,q.__v.set((xo)=>xo+1)}}})(Y)}}),j=document.createDocumentFragment(),C=[],M=document.createElement("section");M.setAttribute("class","group"),C.push(E(M,"aria-label",()=>(w.__v(),w.g.label)));let O=document.createElement("div");O.setAttribute("class","group-h");let H=document.createTextNode("");O.appendChild(H),C.push(B(H,()=>(w.__v(),w.g.label))),M.appendChild(O);let S=document.createElement("div");S.setAttribute("class","group-rows");let U=document.createComment("for");return S.appendChild(U),C.push(A(U)),M.appendChild(S),j.appendChild(M),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let G of C)try{G()}catch(X){}},update:(G,X)=>{w.g=G,w.__i=X,w.__v.set((W)=>W+1)}}})(k)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("button");m.setAttribute("class","trigger"),m.setAttribute("part","trigger"),m.setAttribute("type","button"),T(t,"triggerBtn",m),m.setAttribute("aria-label","Choose theme"),m.setAttribute("aria-haspopup","dialog"),o.push(E(m,"aria-expanded",()=>t.panelOpen()?"true":"false")),o.push(K(m,"click",function(){let v=()=>t.togglePanel();return typeof v==="function"?v.bind(void 0):()=>{}}(),{modifiers:[]}));let p=document.createElement("span");p.setAttribute("class","dot"),p.setAttribute("part","dot"),o.push(D(()=>{let v=t.dotColor();if(v==null)p.style.removeProperty("background");else p.style.setProperty("background",String(v))})),p.setAttribute("aria-hidden","true"),m.appendChild(p);let d=document.createElement("span");d.setAttribute("class","label"),d.setAttribute("part","label");let l=document.createTextNode("");d.appendChild(l),o.push(B(l,()=>t.triggerLabel())),m.appendChild(d);let i=document.createElement("svg");i.setAttribute("class","caret"),i.setAttribute("viewbox","0 0 10 6"),i.setAttribute("aria-hidden","true");let u=document.createElement("path");u.setAttribute("d","M1 1l4 4 4-4"),u.setAttribute("fill","none"),u.setAttribute("stroke","currentColor"),u.setAttribute("stroke-width","1.5"),u.setAttribute("stroke-linecap","round"),u.setAttribute("stroke-linejoin","round"),i.appendChild(u),m.appendChild(i),c.appendChild(m);let s=document.createElement("div");s.setAttribute("class","panel"),s.setAttribute("part","panel"),s.setAttribute("popover","auto"),T(t,"panelEl",s);let f=document.createElement("div");f.setAttribute("class","grid"),f.setAttribute("part","grid"),f.setAttribute("role","listbox"),f.setAttribute("aria-label","Theme");let h=document.createComment("for");if(f.appendChild(h),o.push(b(h)),s.appendChild(f),c.appendChild(s),a!=null){let v=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(v){let g=new Proxy(t,{get(x,k){return x[k]},set(x,k,w){let A=x[k];if(A!=null&&typeof A.set==="function"){if(typeof w==="function")A.set(()=>w);else A.set(w);return!0}return!1}});if(v.mount){let x=v.mount(g,r);if(typeof x==="function")o.push(x)}if(v.unmount)o.push(()=>v.unmount(g))}}return t}});var ga=`
.sui-rail-project-popover {
  position: fixed;
  z-index: var(--z-popover, 1000);
  min-width: 220px;
  background: var(--popover, var(--card, #fff));
  color: var(--popover-foreground, var(--card-foreground, #111));
  border: 1px solid var(--border, #e5e5e5);
  border-radius: var(--radius-md, 8px);
  box-shadow: var(--shadow-md, 0 4px 12px rgba(0,0,0,0.15));
  overflow: hidden;
  font-family: var(--font-body, system-ui);
  font-size: var(--text-base, 14px);
}
.sui-rail-project-popover-title {
  padding: 8px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--muted-foreground);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.sui-rail-project-popover-row {
  padding: 8px 12px;
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 2px;
  transition: background-color var(--duration-fast, 100ms) var(--ease-out, ease-out);
}
.sui-rail-project-popover-row:hover {
  background: color-mix(in oklch, var(--foreground) 6%, transparent);
}
.sui-rail-project-popover-row.active {
  background: color-mix(in oklch, var(--primary) 12%, transparent);
  color: var(--primary);
}
.sui-rail-project-popover-row-name {
  font-size: 13px;
  font-weight: 500;
}
.sui-rail-project-popover-row-path {
  font-size: 11px;
  color: var(--muted-foreground);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.sui-rail-project-popover-sep {
  height: 1px;
  background: var(--border);
  margin: 4px 0;
}
.sui-rail-project-popover-row.add {
  color: var(--primary);
  font-weight: 500;
}
.sui-rail-project-popover-empty {
  padding: 12px;
  text-align: center;
  font-size: 12px;
  color: var(--muted-foreground);
}
`,Kr=!1;function ya(){if(Kr)return;Kr=!0;let e=document.createElement("style");e.setAttribute("data-sui-rail",""),e.textContent=ga,document.head.appendChild(e)}function Mn(e,n,r){e.dispatchEvent(new CustomEvent(n,{detail:r,bubbles:!0,composed:!0}))}function Ir(e){ya();let{host:n,open:r,projects:o,activeProjectId:t,popoverEl:a,badgeEl:b}=e,c=()=>r.set(!1),m=async()=>{try{let g=await fetch("/api/v1/projects",{credentials:"same-origin"});if(!g.ok)return;let x=await g.json();o.set(x.projects??[]),t.set(x.active??null)}catch{}},p=async(g)=>{try{await fetch("/api/v1/projects/switch",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({id:g})}),t.set(g),c(),Mn(n,"ship:project:change",{id:g})}catch{}m()},d=async()=>{let g=window.prompt("Path to project directory:");if(!g)return;try{let x=await fetch("/api/v1/projects",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({path:g})});if(!x.ok){let k=await x.text().catch(()=>"");Mn(n,"sui:notify",{tone:"warning",title:`Add failed: ${k||x.status}`});return}await m(),Mn(n,"sui:notify",{tone:"success",title:"Project added."})}catch(x){Mn(n,"sui:notify",{tone:"destructive",title:String(x.message??x)})}},l=()=>{let g=a(),x=b();if(!g||!x)return;let k=x.getBoundingClientRect(),w=g.offsetWidth||220,A=8,j=Math.max(8,Math.min(k.left,window.innerWidth-w-8)),C=Math.max(8,window.innerHeight-k.top+A);g.style.left=`${j}px`,g.style.bottom=`${C}px`},i=(g)=>{let x=g.target;if(!x)return;if(x.closest("[data-rail-add]")){d();return}let k=x.closest("[data-project-id]");if(k?.dataset.projectId)p(k.dataset.projectId)},u=()=>r.set(!r()),s=(g)=>{if(!r())return;let x=typeof g.composedPath==="function"?g.composedPath():[],k=a(),w=b();for(let A of x)if(A===k||A===w)return;c()},f=()=>{if(r())l()},h=D(()=>{let g=b();if(!g)return;return g.addEventListener("click",u),()=>g.removeEventListener("click",u)}),v=D(()=>{let g=a();if(!g)return;return requestAnimationFrame(l),g.addEventListener("click",i),()=>g.removeEventListener("click",i)});return document.addEventListener("click",s,!0),window.addEventListener("resize",f),window.addEventListener("scroll",f,{capture:!0,passive:!0}),{loadProjects:m,dispose(){h(),v(),document.removeEventListener("click",s,!0),window.removeEventListener("resize",f),window.removeEventListener("scroll",f,!0)}}}var xa=15000;function $r(e){return e.split("/").filter(Boolean)[0]??"dashboard"}function or(e,n,r){e.dispatchEvent(new CustomEvent(n,{detail:r,bubbles:!0,composed:!0}))}var Nn=P({scope:()=>{let e=N(!1),n=N([]),r=N(null),o=N(typeof location<"u"?$r(location.pathname):""),t=N(0);return{projectOpen:e,projects:n,activeProjectId:r,activeView:o,sessionCount:t,projectInitial:()=>{let b=n(),c=b.find((m)=>m.id===r())??b[0];if(!c)return"";return(c.label??c.name).charAt(0).toUpperCase()},popoverEl:J(),projectBadgeEl:J(),launcherBtnEl:J(),createBtnEl:J(),toggleBtn:J(),missionEl:J(),navSlot:J()}},mount(e,n){let r=e,o=D(()=>{let x=r.toggleBtn(),k=r.collapsed();if(x)x.setAttribute("aria-label",k?"Show rail":"Hide rail")}),t=()=>{let x=!r.collapsed();r.collapsed.set(x),or(n,"sui-rail:toggle",{collapsed:x})},a=D(()=>{let x=r.toggleBtn();if(!x)return;return x.addEventListener("click",t),()=>x.removeEventListener("click",t)}),b=Ir({host:n,open:r.projectOpen,projects:r.projects,activeProjectId:r.activeProjectId,popoverEl:r.popoverEl,badgeEl:r.projectBadgeEl});b.loadProjects();let c=()=>r.projectOpen.set(!1),m=(x)=>{or(n,"ship:launcher:open",{anchor:x.currentTarget}),c()},p=()=>{or(n,"ship:create:open"),c()},d=D(()=>{let x=r.launcherBtnEl();if(!x)return;return x.addEventListener("click",m),()=>x.removeEventListener("click",m)}),l=D(()=>{let x=r.createBtnEl();if(!x)return;return x.addEventListener("click",p),()=>x.removeEventListener("click",p)}),i=async()=>{try{let x=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(!x.ok)return;let w=(await x.json()).sessions.filter((A)=>A.status!=="ended").length;r.sessionCount.set(w)}catch{}};i();let u=window.setInterval(()=>void i(),xa),s=()=>void i();window.addEventListener("ship:session:changed",s);let f=D(()=>{let x=r.missionEl();if(!x)return;let k=r.sessionCount();x.setAttribute("badge",k>0?String(k):"")}),h=()=>{r.activeView.set($r(location.pathname))},v=()=>h();document.addEventListener("sui:navigate",v),window.addEventListener("popstate",v);let g=D(()=>{let x=r.activeView(),k=n.querySelectorAll("sui-rail-item[data-view]");for(let A of k)if(A.dataset.view===x)A.setAttribute("active","");else A.removeAttribute("active");let w=r.missionEl();if(w)if(x==="mesh")w.setAttribute("active","");else w.removeAttribute("active")});return()=>{o(),a(),d(),l(),f(),g(),b.dispose(),document.removeEventListener("sui:navigate",v),window.removeEventListener("popstate",v),window.removeEventListener("ship:session:changed",s),window.clearInterval(u)}}});var Of=z("sui-rail",{props:{collapsed:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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

  .footer-btn,
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
  .footer-btn:hover,
  .toggle-btn:hover {
    background: color-mix(in oklch, var(--foreground) 8%, transparent);
    border-color: var(--border);
    color: var(--foreground);
  }
  .footer-btn:focus-visible,
  .toggle-btn:focus-visible {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in oklch, var(--primary, #3b82f6) 50%, transparent);
  }
  .toggle-btn::before { content: '◀'; font-size: 10px; line-height: 1; }
  :host([collapsed]) .toggle-btn::before { content: '▶'; }

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.collapsed=n.collapsed;let a=null;if(Nn&&typeof Nn.setup==="function"){if(e.scope=t,a=Nn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(C)=>F({anchor:C,test:()=>{if(t.projects().length===0)return 0;return-1},branches:[()=>{let M=document.createDocumentFragment(),O=[],H=document.createElement("div");H.setAttribute("class","sui-rail-project-popover-empty");let S=document.createTextNode("No projects tracked.");return H.appendChild(S),M.appendChild(H),{nodes:Array.from(M.childNodes),cleanup:()=>{for(let U of O)try{U()}catch(G){}}}}]}),c=(C)=>R({anchor:C,items:()=>t.projects(),key:(M,O)=>{let H=Object.create(t);return H.p=M,H.__i=O,((S)=>M.id)(H)},render:(M,O)=>{let H=Object.create(t);return H.p=M,H.__i=O,H.__v=N(0),((S)=>{let U=document.createDocumentFragment(),G=[],X=document.createElement("div");X.setAttribute("class","sui-rail-project-popover-row"),G.push(pt(X,"active",()=>(S.__v(),S.p.id===S.activeProjectId()))),G.push(E(X,"data-project-id",()=>(S.__v(),S.p.id)));let W=document.createElement("span");W.setAttribute("class","sui-rail-project-popover-row-name");let Y=document.createTextNode("");W.appendChild(Y),G.push(B(Y,()=>(S.__v(),S.p.label||S.p.name))),X.appendChild(W);let q=document.createElement("span");q.setAttribute("class","sui-rail-project-popover-row-path");let _=document.createTextNode("");return q.appendChild(_),G.push(B(_,()=>(S.__v(),S.p.path))),X.appendChild(q),U.appendChild(X),{nodes:Array.from(U.childNodes),cleanup:()=>{for(let V of G)try{V()}catch(I){}},update:(V,I)=>{S.p=V,S.__i=I,S.__v.set((dt)=>dt+1)}}})(H)}}),m=(C)=>F({anchor:C,test:()=>{if(t.projectOpen())return 0;return-1},branches:[()=>{let M=document.createDocumentFragment(),O=[],H=document.createElement("div");H.setAttribute("class","sui-rail-project-popover"),H.setAttribute("part","project-popover"),H.setAttribute("role","dialog"),H.setAttribute("aria-label","Switch project"),T(t,"popoverEl",H),ot(H,ht,{duration:150},"both");let S=document.createElement("div");S.setAttribute("class","sui-rail-project-popover-title");let U=document.createTextNode("Projects");S.appendChild(U),H.appendChild(S);let G=document.createComment("if");H.appendChild(G),O.push(b(G));let X=document.createComment("for");H.appendChild(X),O.push(c(X));let W=document.createElement("div");W.setAttribute("class","sui-rail-project-popover-sep"),H.appendChild(W);let Y=document.createElement("div");Y.setAttribute("class","sui-rail-project-popover-row add"),Y.setAttribute("data-rail-add","");let q=document.createElement("span"),_=document.createTextNode("+ Add project");return q.appendChild(_),Y.appendChild(q),H.appendChild(Y),M.appendChild(H),{nodes:Array.from(M.childNodes),cleanup:()=>{for(let V of O)try{V()}catch(I){}}}}]}),p=r.shadowRoot??r;L(p);let d=document.createElement("nav");d.setAttribute("class","rail-nav"),d.setAttribute("part","nav");let l=document.createElement("slot");T(t,"navSlot",l),d.appendChild(l);let i=document.createElement("sui-rail-item");T(t,"missionEl",i),i.setAttribute("part","mission-control"),i.setAttribute("data-view","mesh"),i.setAttribute("icon","message-square-plus"),i.setAttribute("label","Mission Control"),i.setAttribute("href","/mesh"),d.appendChild(i),p.appendChild(d);let u=document.createElement("div");u.setAttribute("class","rail-footer"),u.setAttribute("part","footer");let s=document.createElement("button");s.setAttribute("type","button"),s.setAttribute("class","project-badge"),s.setAttribute("part","project-badge"),s.setAttribute("aria-label","Switch project"),s.setAttribute("aria-haspopup","dialog"),T(t,"projectBadgeEl",s);let f=document.createTextNode("");s.appendChild(f),o.push(B(f,()=>t.projectInitial())),u.appendChild(s);let h=document.createElement("div");h.setAttribute("class","footer-actions");let v=document.createElement("button");v.setAttribute("type","button"),v.setAttribute("class","footer-btn"),v.setAttribute("part","launcher-btn"),v.setAttribute("aria-label","Open launcher"),T(t,"launcherBtnEl",v);let g=document.createElement("sui-icon");g.setAttribute("name","folder"),g.setAttribute("size","sm"),v.appendChild(g),h.appendChild(v);let x=document.createElement("button");x.setAttribute("type","button"),x.setAttribute("class","footer-btn"),x.setAttribute("part","create-btn"),x.setAttribute("aria-label","Create new"),T(t,"createBtnEl",x);let k=document.createElement("sui-icon");k.setAttribute("name","plus"),k.setAttribute("size","sm"),x.appendChild(k),h.appendChild(x);let w=document.createElement("button");w.setAttribute("type","button"),w.setAttribute("class","toggle-btn"),w.setAttribute("part","toggle"),w.setAttribute("aria-label","Collapse rail"),T(t,"toggleBtn",w),h.appendChild(w),u.appendChild(h),p.appendChild(u);let A=document.createElement("sui-portal");A.setAttribute("target","body");let j=document.createComment("if");if(A.appendChild(j),o.push(m(j)),p.appendChild(A),a!=null){let C=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(C){let M=new Proxy(t,{get(O,H){return O[H]},set(O,H,S){let U=O[H];if(U!=null&&typeof U.set==="function"){if(typeof S==="function")U.set(()=>S);else U.set(S);return!0}return!1}});if(C.mount){let O=C.mount(M,r);if(typeof O==="function")o.push(O)}if(C.unmount)o.push(()=>C.unmount(M))}}return t}});var Mf=z("sui-rail-item",{props:{href:y.string({default:""}),label:y.string({default:""}),icon:y.string({default:""}),active:y.bool({default:!1,reflect:!0}),badge:y.string({default:""}),disabled:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.href=n.href,t.label=n.label,t.icon=n.icon,t.active=n.active,t.badge=n.badge,t.disabled=n.disabled;let a=r.shadowRoot??r;L(a);let b=document.createElement("a");b.setAttribute("class","item"),b.setAttribute("part","item"),T(t,"itemEl",b),o.push(E(b,"href",()=>t.href()||"#")),o.push(E(b,"aria-current",()=>t.active()?"page":null)),o.push(E(b,"aria-disabled",()=>t.disabled()?"true":null)),o.push(E(b,"title",()=>t.label()));let c=document.createElement("span");c.setAttribute("class","icon-wrap"),c.setAttribute("part","icon-wrap");let m=document.createElement("sui-icon");o.push(E(m,"name",()=>t.icon())),m.setAttribute("size","sm"),c.appendChild(m),b.appendChild(c);let p=document.createElement("span");p.setAttribute("class","label"),p.setAttribute("part","label");let d=document.createTextNode("");p.appendChild(d),o.push(B(d,()=>t.label())),b.appendChild(p);let l=document.createElement("span");l.setAttribute("class","badge"),l.setAttribute("part","badge"),T(t,"badgeEl",l);let i=document.createTextNode("");return l.appendChild(i),o.push(B(i,()=>t.badge())),b.appendChild(l),a.appendChild(b),t}});var Sf=z("sui-rail-section",{props:{label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.label=n.label;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");b.setAttribute("class","heading"),b.setAttribute("part","heading");let c=document.createTextNode("");b.appendChild(c),o.push(B(c,()=>t.label())),a.appendChild(b);let m=document.createElement("slot");return a.appendChild(m),t}});var Gf=z("sui-status-bar",{props:{branch:y.string({default:""}),theme:y.string({default:""}),online:y.bool({default:!1,reflect:!0}),agents:y.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.branch=n.branch,t.theme=n.theme,t.online=n.online,t.agents=n.agents;let a=r.shadowRoot??r;L(a);let b=document.createElement("span");b.setAttribute("class","dot"),b.setAttribute("part","dot"),b.setAttribute("aria-hidden","true"),a.appendChild(b);let c=document.createElement("span");c.setAttribute("class","text"),c.setAttribute("part","branch");let m=document.createTextNode("");c.appendChild(m),o.push(B(m,()=>t.branch())),a.appendChild(c);let p=document.createElement("span");p.setAttribute("class","text"),p.setAttribute("part","agents");let d=document.createTextNode("");p.appendChild(d),o.push(B(d,()=>t.agents())),a.appendChild(p);let l=document.createElement("span");l.setAttribute("class","slot-wrap");let i=document.createElement("slot");i.setAttribute("name","start"),l.appendChild(i),a.appendChild(l);let u=document.createElement("span");u.setAttribute("class","spacer"),a.appendChild(u);let s=document.createElement("span");s.setAttribute("class","slot-wrap");let f=document.createElement("slot");f.setAttribute("name","end"),s.appendChild(f),a.appendChild(s);let h=document.createElement("span");h.setAttribute("class","text"),h.setAttribute("part","theme");let v=document.createTextNode("");return h.appendChild(v),o.push(B(v,()=>t.theme())),a.appendChild(h),t}});var wa=["a[href]","area[href]","button:not([disabled])",'input:not([disabled]):not([type="hidden"])',"select:not([disabled])","textarea:not([disabled])","iframe","object","embed","summary",'[contenteditable]:not([contenteditable="false"])','[tabindex]:not([tabindex^="-"])'].join(",");function zt(e){return Array.from(e.querySelectorAll(wa)).filter((n)=>{if(n.hasAttribute("disabled"))return!1;if(n.getAttribute("aria-hidden")==="true")return!1;return!0})}class Bt{saved=null;save(){this.saved=document.activeElement??null}restore(){let e=this.saved;if(this.saved=null,e&&typeof e.focus==="function")e.focus()}}var Ft=0,ar=null,ir=null;function Hn(){if(typeof document>"u")return;if(Ft+=1,Ft>1)return;let e=document.body;ar=e.style.overflow,ir=e.style.paddingRight;let n=typeof window<"u"?window.innerWidth-document.documentElement.clientWidth:0;if(n>0){let r=parseFloat(getComputedStyle(e).paddingRight||"0")||0;e.style.paddingRight=`${r+n}px`}e.style.overflow="hidden"}function Lt(){if(typeof document>"u")return;if(Ft===0)return;if(Ft-=1,Ft>0)return;let e=document.body;e.style.overflow=ar??"",e.style.paddingRight=ir??"",ar=null,ir=null}var Aa=0,Sn=P({mount(e,n){let r=e,o=++Aa,t=`sui-dialog-header-${o}`,a=`sui-dialog-body-${o}`,b=r.dialogEl(),c=r.closeBtn(),m=r.sentinelStart(),p=r.sentinelEnd();if(!b)return;let d=D(()=>{let O=r.headerEl();if(O)O.id=t}),l=D(()=>{let O=r.bodyEl();if(O)O.id=a});b.setAttribute("aria-labelledby",t),b.setAttribute("aria-describedby",a);let i=!1,u=new Bt;function s(){b.setAttribute("role",r.roleOverride()||"dialog"),b.setAttribute("aria-modal","true")}s();function f(){let O=zt(b).filter((S)=>!S.classList.contains("sentinel"));return[...zt(n),...O]}function h(){let O=n.querySelector("[autofocus]")??b.querySelector("[autofocus]");if(O){O.focus();return}let H=f(),S=H.find((U)=>U!==c)??H[0];if(S)S.focus();else b.focus()}function v(){u.save();try{b.showModal()}catch{}if(!i)Hn(),i=!0;queueMicrotask(()=>h()),n.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function g(){if(b.open)b.close()}let x=()=>{if(r.open())r.open.set(!1);if(i)Lt(),i=!1;u.restore(),n.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},k=(O)=>{if(!r.dismissible())return;if(O.target===b)r.open.set(!1)},w=()=>{n.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),r.open.set(!1)},A=()=>{let O=f();if(O.length)O[O.length-1].focus()},j=()=>{let O=f();if(O.length)O[0].focus()};b.addEventListener("close",x),b.addEventListener("click",k),c?.addEventListener("click",w),m?.addEventListener("focus",A),p?.addEventListener("focus",j);let C=D(()=>{let O=r.open();if(n.setAttribute("data-state",O?"open":"closed"),O&&!b.open)v();else if(!O&&b.open)g()}),M=D(()=>{s()});return n.show=()=>r.open.set(!0),n.showModal=()=>r.open.set(!0),n.close=()=>r.open.set(!1),()=>{if(i)Lt();b.removeEventListener("close",x),b.removeEventListener("click",k),c?.removeEventListener("click",w),m?.removeEventListener("focus",A),p?.removeEventListener("focus",j),C(),M(),d(),l()}}});var Zf=z("sui-dialog",{props:{open:y.bool({default:!1,reflect:!0}),size:y.union(["sm","md","lg","full"],{default:"md",reflect:!0}),backdrop:y.union(["default","blur","solid"],{default:"default",reflect:!0}),dismissible:y.bool({default:!0}),"no-close-button":y.bool({default:!1,reflect:!0}),"no-header":y.bool({default:!1,reflect:!0}),"no-footer":y.bool({default:!1,reflect:!0}),"role-override":y.union(["dialog","alertdialog"],{default:"dialog",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.size=n.size,t.backdrop=n.backdrop,t.dismissible=n.dismissible,t.noCloseButton=n.noCloseButton,t.noHeader=n.noHeader,t.noFooter=n.noFooter,t.roleOverride=n.roleOverride;let a=null;if(Sn&&typeof Sn.setup==="function"){if(e.scope=t,a=Sn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(v)=>F({anchor:v,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),x=[],k=document.createElement("div");k.setAttribute("class","dialog-content"),ot(k,ht,{duration:150},"both");let w=document.createElement("div");w.setAttribute("class","header"),w.setAttribute("part","header"),T(t,"headerEl",w);let A=document.createElement("slot");A.setAttribute("name","header"),w.appendChild(A),k.appendChild(w);let j=document.createElement("div");j.setAttribute("class","body"),j.setAttribute("part","body"),T(t,"bodyEl",j);let C=document.createElement("slot");j.appendChild(C),k.appendChild(j);let M=document.createElement("div");M.setAttribute("class","footer"),M.setAttribute("part","footer");let O=document.createElement("slot");return O.setAttribute("name","footer"),M.appendChild(O),k.appendChild(M),g.appendChild(k),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let H of x)try{H()}catch(S){}}}}]}),c=(v)=>F({anchor:v,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),x=[],k=document.createElement("sui-portal");k.setAttribute("target","body");let w=document.createElement("div");return w.setAttribute("class","sui-dialog-presence"),w.setAttribute("aria-hidden","true"),w.setAttribute("data-sui-dialog-open",""),k.appendChild(w),g.appendChild(k),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let A of x)try{A()}catch(j){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createElement("dialog");p.setAttribute("part","dialog"),T(t,"dialogEl",p);let d=document.createElement("span");d.setAttribute("tabindex","0"),d.setAttribute("class","sentinel"),d.setAttribute("aria-hidden","true"),T(t,"sentinelStart",d),p.appendChild(d);let l=document.createElement("button");l.setAttribute("class","close"),l.setAttribute("part","close"),l.setAttribute("type","button"),l.setAttribute("aria-label","Close dialog"),T(t,"closeBtn",l);let i=document.createElement("svg");i.setAttribute("viewbox","0 0 16 16"),i.setAttribute("fill","none"),i.setAttribute("stroke","currentColor"),i.setAttribute("stroke-width","1.75"),i.setAttribute("stroke-linecap","round"),i.setAttribute("stroke-linejoin","round"),i.setAttribute("aria-hidden","true"),i.setAttribute("focusable","false");let u=document.createElement("path");u.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),i.appendChild(u),l.appendChild(i),p.appendChild(l);let s=document.createComment("if");p.appendChild(s),o.push(b(s));let f=document.createElement("span");f.setAttribute("tabindex","0"),f.setAttribute("class","sentinel"),f.setAttribute("aria-hidden","true"),T(t,"sentinelEnd",f),p.appendChild(f),m.appendChild(p);let h=document.createComment("if");if(m.appendChild(h),o.push(c(h)),a!=null){let v=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(v){let g=new Proxy(t,{get(x,k){return x[k]},set(x,k,w){let A=x[k];if(A!=null&&typeof A.set==="function"){if(typeof w==="function")A.set(()=>w);else A.set(w);return!0}return!1}});if(v.mount){let x=v.mount(g,r);if(typeof x==="function")o.push(x)}if(v.unmount)o.push(()=>v.unmount(g))}}return t}});var ka=0,Fn=P({mount(e,n){let r=e,o=++ka,t=`sui-drawer-header-${o}`,a=`sui-drawer-body-${o}`,b=r.dialogEl(),c=r.closeBtn(),m=r.sentinelStart(),p=r.sentinelEnd(),d=r.headerEl(),l=r.bodyEl();if(!b)return;if(d)d.id=t;if(l)l.id=a;b.setAttribute("aria-labelledby",t),b.setAttribute("aria-describedby",a);let i=!1,u=new Bt;function s(){let C=zt(b).filter((O)=>!O.classList.contains("sentinel"));return[...zt(n),...C]}function f(){let C=n.querySelector("[autofocus]")??b.querySelector("[autofocus]");if(C){C.focus();return}let M=s(),O=M.find((H)=>H!==c)??M[0];if(O)O.focus();else b.focus()}function h(){u.save();try{b.showModal()}catch{}if(!i)Hn(),i=!0;queueMicrotask(()=>f()),n.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function v(){if(b.open)b.close()}let g=()=>{if(r.open())r.open.set(!1);if(i)Lt(),i=!1;u.restore(),n.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},x=(C)=>{if(!r.dismissible())return;if(C.target===b)r.open.set(!1)},k=()=>{n.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),r.open.set(!1)},w=()=>{let C=s();if(C.length)C[C.length-1].focus()},A=()=>{let C=s();if(C.length)C[0].focus()};b.addEventListener("close",g),b.addEventListener("click",x),c?.addEventListener("click",k),m?.addEventListener("focus",w),p?.addEventListener("focus",A);let j=D(()=>{let C=r.open();if(n.setAttribute("data-state",C?"open":"closed"),C&&!b.open)h();else if(!C&&b.open)v()});return n.show=()=>r.open.set(!0),n.close=()=>r.open.set(!1),()=>{if(i)Lt();b.removeEventListener("close",g),b.removeEventListener("click",x),c?.removeEventListener("click",k),m?.removeEventListener("focus",w),p?.removeEventListener("focus",A),j()}}});var tm=z("sui-drawer",{props:{open:y.bool({default:!1,reflect:!0}),side:y.union(["right","left"],{default:"right",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),dismissible:y.bool({default:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.side=n.side,t.size=n.size,t.dismissible=n.dismissible;let a=null;if(Fn&&typeof Fn.setup==="function"){if(e.scope=t,a=Fn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("dialog");c.setAttribute("part","dialog"),c.setAttribute("role","dialog"),c.setAttribute("aria-modal","true"),T(t,"dialogEl",c);let m=document.createElement("span");m.setAttribute("tabindex","0"),m.setAttribute("class","sentinel"),m.setAttribute("aria-hidden","true"),T(t,"sentinelStart",m),c.appendChild(m);let p=document.createElement("div");p.setAttribute("class","header"),p.setAttribute("part","header"),T(t,"headerEl",p);let d=document.createElement("div");d.setAttribute("class","header-content");let l=document.createElement("div");l.setAttribute("class","title");let i=document.createElement("slot");i.setAttribute("name","title"),l.appendChild(i),d.appendChild(l);let u=document.createElement("div");u.setAttribute("class","description");let s=document.createElement("slot");s.setAttribute("name","description"),u.appendChild(s),d.appendChild(u),p.appendChild(d);let f=document.createElement("button");f.setAttribute("class","close"),f.setAttribute("part","close"),f.setAttribute("type","button"),f.setAttribute("aria-label","Close drawer"),T(t,"closeBtn",f);let h=document.createElement("svg");h.setAttribute("viewbox","0 0 16 16"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","1.75"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true"),h.setAttribute("focusable","false");let v=document.createElement("path");v.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),h.appendChild(v),f.appendChild(h),p.appendChild(f),c.appendChild(p);let g=document.createElement("div");g.setAttribute("class","body"),g.setAttribute("part","body"),T(t,"bodyEl",g);let x=document.createElement("slot");g.appendChild(x),c.appendChild(g);let k=document.createElement("div");k.setAttribute("class","footer"),k.setAttribute("part","footer");let w=document.createElement("slot");w.setAttribute("name","footer"),k.appendChild(w),c.appendChild(k);let A=document.createElement("span");if(A.setAttribute("tabindex","0"),A.setAttribute("class","sentinel"),A.setAttribute("aria-hidden","true"),T(t,"sentinelEnd",A),c.appendChild(A),b.appendChild(c),a!=null){let j=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(j){let C=new Proxy(t,{get(M,O){return M[O]},set(M,O,H){let S=M[O];if(S!=null&&typeof S.set==="function"){if(typeof H==="function")S.set(()=>H);else S.set(H);return!0}return!1}});if(j.mount){let M=j.mount(C,r);if(typeof M==="function")o.push(M)}if(j.unmount)o.push(()=>j.unmount(C))}}return t}});var Ea=0,Bn=P({mount(e,n){let r=e,o=`sui-dropdown-menu-panel-${++Ea}`,t=r.triggerWrap(),a=r.triggerSlot(),b=r.panelEl();if(!b)return;b.id=o;let c="",m=null;function p(A){if(m)clearTimeout(m);c+=A.toLowerCase(),m=setTimeout(()=>{c=""},350);let j=d(),C=j.indexOf(document.activeElement);for(let M=0;M<j.length;M++){let O=(C+1+M)%j.length;if((j[O].textContent??"").trim().toLowerCase().startsWith(c))return O}return-1}function d(){return Array.from(n.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((A)=>!A.hasAttribute("disabled"))}function l(A){let j=d();if(!j.length)return;let C=(A%j.length+j.length)%j.length;for(let M of j)M.setAttribute("tabindex","-1");j[C].setAttribute("tabindex","0"),j[C].focus()}function i(){return d().indexOf(document.activeElement)}function u(){d().forEach((A,j)=>{if(!A.hasAttribute("role"))A.setAttribute("role","menuitem");if(!A.hasAttribute("tabindex"))A.setAttribute("tabindex",j===0?"0":"-1")})}function s(){let A=a?.assignedElements({flatten:!0})??[];for(let j of A)j.setAttribute("aria-haspopup","menu"),j.setAttribute("aria-controls",o),j.setAttribute("aria-expanded",r.open()?"true":"false")}function f(){let A=a?.assignedElements({flatten:!0})?.[0];if(A&&b){let j=A.getBoundingClientRect();b.style.top=`${j.bottom+4}px`,b.style.left=`${j.left}px`}b?.showPopover?.()}function h(){b?.hidePopover?.()}let v=()=>{if(r.open())h();else f()},g=(A)=>{let C=A.newState==="open";if(C!==r.open())r.open.set(C);if(n.setAttribute("data-state",C?"open":"closed"),s(),C)queueMicrotask(()=>{u(),l(0)})},x=(A)=>{let j=A.target.closest("[data-value]");if(!j)return;n.emit("sui-select",{value:j.dataset.value??""}),h()},k=(A)=>{if(!r.open())return;let j=d();if(!j.length)return;let C=i();if(A.key==="ArrowDown")A.preventDefault(),l(C+1);else if(A.key==="ArrowUp")A.preventDefault(),l(C-1);else if(A.key==="Home")A.preventDefault(),l(0);else if(A.key==="End")A.preventDefault(),l(j.length-1);else if(A.key==="Escape")A.preventDefault(),h();else if(A.key==="Tab")h();else if((A.key==="Enter"||A.key===" ")&&C>=0)A.preventDefault(),j[C].click();else if(A.key.length===1&&!A.ctrlKey&&!A.metaKey){let M=p(A.key);if(M>=0)A.preventDefault(),l(M)}};if(t)Q(t,"click",v);if(Q(b,"toggle",g),Q(b,"click",x),Q(n,"keydown",k),a)Q(a,"slotchange",s);Q(b,"slotchange",u);let w=D(()=>{let A=r.open();n.setAttribute("data-state",A?"open":"closed"),s()});return()=>{if(m)clearTimeout(m);w()}}});var im=z("sui-dropdown-menu",{props:{open:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open;let a=null;if(Bn&&typeof Bn.setup==="function"){if(e.scope=t,a=Bn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("span");c.setAttribute("class","trigger-wrap"),T(t,"triggerWrap",c);let m=document.createElement("slot");m.setAttribute("name","trigger"),T(t,"triggerSlot",m),c.appendChild(m),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","panel"),p.setAttribute("part","panel"),p.setAttribute("popover","auto"),p.setAttribute("role","menu"),T(t,"panelEl",p);let d=document.createElement("slot");if(T(t,"itemSlot",d),p.appendChild(d),b.appendChild(p),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(u,s){return u[s]},set(u,s,f){let h=u[s];if(h!=null&&typeof h.set==="function"){if(typeof f==="function")h.set(()=>f);else h.set(f);return!0}return!1}});if(l.mount){let u=l.mount(i,r);if(typeof u==="function")o.push(u)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var Ca=0,Gn=P({mount(e,n){let r=e,o=`sui-context-menu-panel-${++Ca}`,t=r.triggerSlot(),a=r.panelEl();if(!a)return;a.id=o;let b="",c=null;function m(w){if(c)clearTimeout(c);b+=w.toLowerCase(),c=setTimeout(()=>{b=""},350);let A=p(),j=A.indexOf(document.activeElement);for(let C=0;C<A.length;C++){let M=(j+1+C)%A.length;if((A[M].textContent??"").trim().toLowerCase().startsWith(b))return M}return-1}function p(){return Array.from(n.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((w)=>!w.hasAttribute("disabled"))}function d(w){let A=p();if(!A.length)return;let j=(w%A.length+A.length)%A.length;for(let C of A)C.setAttribute("tabindex","-1");A[j].setAttribute("tabindex","0"),A[j].focus()}function l(){return p().indexOf(document.activeElement)}function i(){p().forEach((w,A)=>{if(!w.hasAttribute("role"))w.setAttribute("role","menuitem");if(!w.hasAttribute("tabindex"))w.setAttribute("tabindex",A===0?"0":"-1")})}function u(w,A){if(a)a.style.top=`${A}px`,a.style.left=`${w}px`;a?.showPopover?.()}function s(){a?.hidePopover?.()}n.expose({show:u,close:s});let f=(w)=>{w.preventDefault(),u(w.clientX,w.clientY)},h=(w)=>{let j=w.newState==="open";if(j!==r.open())r.open.set(j);if(n.setAttribute("data-state",j?"open":"closed"),j)n.emit("sui-open"),queueMicrotask(()=>{i(),d(0)});else n.emit("sui-close")},v=(w)=>{let A=w.target.closest("[data-value]");if(!A)return;n.emit("sui-select",{value:A.dataset.value??""}),s()},g=(w)=>{if(!r.open())return;let A=p();if(!A.length)return;let j=l();if(w.key==="ArrowDown")w.preventDefault(),d(j+1);else if(w.key==="ArrowUp")w.preventDefault(),d(j-1);else if(w.key==="Home")w.preventDefault(),d(0);else if(w.key==="End")w.preventDefault(),d(A.length-1);else if(w.key==="Escape")w.preventDefault(),s();else if(w.key==="Tab")s();else if((w.key==="Enter"||w.key===" ")&&j>=0)w.preventDefault(),A[j].click();else if(w.key.length===1&&!w.ctrlKey&&!w.metaKey){let C=m(w.key);if(C>=0)w.preventDefault(),d(C)}},x=()=>{let w=t?.assignedElements({flatten:!0})??[];for(let A of w)A.addEventListener("contextmenu",f)};if(t)Q(t,"slotchange",x);x(),Q(a,"toggle",h),Q(a,"click",v),Q(n,"keydown",g);let k=D(()=>{n.setAttribute("data-state",r.open()?"open":"closed")});return()=>{let w=t?.assignedElements({flatten:!0})??[];for(let A of w)A.removeEventListener("contextmenu",f);if(c)clearTimeout(c);k()}}});var pm=z("sui-context-menu",{props:{open:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open;let a=null;if(Gn&&typeof Gn.setup==="function"){if(e.scope=t,a=Gn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");c.setAttribute("name","trigger"),T(t,"triggerSlot",c),b.appendChild(c);let m=document.createElement("div");m.setAttribute("class","panel"),m.setAttribute("part","panel"),m.setAttribute("popover","auto"),m.setAttribute("role","menu"),m.setAttribute("aria-label","Context menu"),T(t,"panelEl",m);let p=document.createElement("slot");if(T(t,"itemSlot",p),m.appendChild(p),b.appendChild(m),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var ja=0,Un=8;function za(e,n,r){let o=e.getBoundingClientRect();switch(n.style.top="",n.style.left="",n.style.right="",n.style.bottom="",r){case"top":n.style.bottom=`${window.innerHeight-o.top+Un}px`,n.style.left=`${o.left+o.width/2}px`,n.style.transform="translateX(-50%)";break;case"left":n.style.right=`${window.innerWidth-o.left+Un}px`,n.style.top=`${o.top}px`;break;case"right":n.style.left=`${o.right+Un}px`,n.style.top=`${o.top}px`;break;case"bottom":default:n.style.top=`${o.bottom+Un}px`,n.style.left=`${o.left+o.width/2}px`,n.style.transform="translateX(-50%)"}}var Xn=P({mount(e,n){let r=e,o=`sui-hover-card-${++ja}`,t=r.triggerSlot(),a=r.panelEl();if(!a)return;a.id=o;let b=null,c=null;function m(){if(b)clearTimeout(b),b=null;if(c)clearTimeout(c),c=null}function p(){let g=t?.assignedElements({flatten:!0})??[];for(let x of g)x.setAttribute("aria-describedby",o)}function d(){m();let g=t?.assignedElements({flatten:!0})?.[0];if(g&&a)za(g,a,r.placement()||"bottom");a?.showPopover?.()}function l(){m(),a?.hidePopover?.()}n.show=d,n.hide=l;let i=(g)=>{let k=g.newState==="open";if(k!==r.open())r.open.set(k);n.setAttribute("data-state",k?"open":"closed"),n.dispatchEvent(new CustomEvent(k?"sui-open":"sui-close",{bubbles:!0,composed:!0}))},u=()=>{m(),b=setTimeout(()=>d(),r.showDelay()??300)},s=()=>{m(),c=setTimeout(()=>l(),r.hideDelay()??100)},f=()=>{m(),d()},h=()=>{m(),c=setTimeout(()=>l(),r.hideDelay()??100)};a.addEventListener("toggle",i),t?.addEventListener("slotchange",p),p(),n.addEventListener("pointerenter",u),n.addEventListener("pointerleave",s),n.addEventListener("focusin",f),n.addEventListener("focusout",h);let v=D(()=>{n.setAttribute("data-state",r.open()?"open":"closed")});return()=>{m(),a.removeEventListener("toggle",i),t?.removeEventListener("slotchange",p),n.removeEventListener("pointerenter",u),n.removeEventListener("pointerleave",s),n.removeEventListener("focusin",f),n.removeEventListener("focusout",h),v()}}});var gm=z("sui-hover-card",{props:{open:y.bool({default:!1,reflect:!0}),placement:y.union(["top","bottom","left","right"],{default:"bottom",reflect:!0}),"show-delay":y.number({default:300}),"hide-delay":y.number({default:100})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.placement=n.placement,t.showDelay=n.showDelay,t.hideDelay=n.hideDelay;let a=null;if(Xn&&typeof Xn.setup==="function"){if(e.scope=t,a=Xn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");c.setAttribute("name","trigger"),T(t,"triggerSlot",c),b.appendChild(c);let m=document.createElement("div");m.setAttribute("class","panel"),m.setAttribute("part","panel"),m.setAttribute("popover","auto"),T(t,"panelEl",m);let p=document.createElement("slot");if(m.appendChild(p),b.appendChild(m),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var qn=P({mount(e,n){let r=e,o=()=>Array.from(n.querySelectorAll(":scope > sui-accordion-item")),t=()=>o().filter((d)=>!d.hasAttribute("disabled")),a=()=>{let d=r.value();return new Set(d?d.split(",").filter(Boolean):[])},b=()=>{let d=a();for(let l of o()){let i=l.getAttribute("value")||"";if(d.has(i))l.setAttribute("open","");else l.removeAttribute("open")}},c=D(()=>{r.value(),b()}),m=(d)=>{let l=d;l.stopPropagation();let i=l.detail?.value,u=a();if(r.type()==="single")r.value.set(u.has(i)?"":i);else{if(u.has(i))u.delete(i);else u.add(i);r.value.set(Array.from(u).join(","))}n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:r.value()},bubbles:!0,composed:!0}))},p=(d)=>{let l=d;l.stopPropagation();let i=t();if(!i.length)return;let u=l.target?.closest("sui-accordion-item"),s=u?i.indexOf(u):-1,f;switch(l.detail?.direction){case"next":f=i[(s+1)%i.length];break;case"prev":f=i[(s-1+i.length)%i.length];break;case"first":f=i[0];break;case"last":f=i[i.length-1];break}if(f)f.shadowRoot?.querySelector("button")?.focus()};return n.addEventListener("sui-accordion-item-toggle",m),n.addEventListener("sui-accordion-nav",p),()=>{c(),n.removeEventListener("sui-accordion-item-toggle",m),n.removeEventListener("sui-accordion-nav",p)}}});var Em=z("sui-accordion",{props:{type:y.string({default:"single",reflect:!0}),value:y.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host { display: block; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.type=n.type,t.value=n.value;let a=null;if(qn&&typeof qn.setup==="function"){if(e.scope=t,a=qn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var La=0,Rn=P({mount(e,n){let r=e,o=++La,t=`sui-acc-trigger-${o}`,a=`sui-acc-content-${o}`,b=D(()=>{let l=r.triggerBtn(),i=r.contentWrap();if(l)l.id=t,l.setAttribute("aria-controls",a);if(i)i.id=a,i.setAttribute("aria-labelledby",t)}),c=D(()=>{let l=r.triggerBtn();if(l)l.setAttribute("aria-expanded",r.open()?"true":"false")}),m=()=>{if(r.disabled())return;n.dispatchEvent(new CustomEvent("sui-accordion-item-toggle",{detail:{value:r.value()},bubbles:!0,composed:!0}))},p=(l)=>{if(l.key==="ArrowDown"||l.key==="ArrowUp")l.preventDefault(),n.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:l.key==="ArrowDown"?"next":"prev"},bubbles:!0,composed:!0}));else if(l.key==="Home"||l.key==="End")l.preventDefault(),n.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:l.key==="Home"?"first":"last"},bubbles:!0,composed:!0}))},d=D(()=>{let l=r.triggerBtn();if(!l)return;return l.addEventListener("click",m),l.addEventListener("keydown",p),()=>{l.removeEventListener("click",m),l.removeEventListener("keydown",p)}});return()=>{b(),c(),d()}}});var Om=z("sui-accordion-item",{props:{value:y.string({default:"",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),open:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.disabled=n.disabled,t.open=n.open;let a=null;if(Rn&&typeof Rn.setup==="function"){if(e.scope=t,a=Rn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("button");c.setAttribute("class","trigger"),c.setAttribute("part","trigger"),T(t,"triggerBtn",c),o.push(E(c,"disabled",()=>t.disabled()));let m=document.createElement("span");m.setAttribute("class","trigger-label");let p=document.createElement("slot");p.setAttribute("name","trigger"),m.appendChild(p),c.appendChild(m);let d=document.createElement("svg");d.setAttribute("class","chevron"),d.setAttribute("part","chevron"),d.setAttribute("viewbox","0 0 16 16"),d.setAttribute("fill","none"),d.setAttribute("stroke","currentColor"),d.setAttribute("stroke-width","1.75"),d.setAttribute("stroke-linecap","round"),d.setAttribute("stroke-linejoin","round"),d.setAttribute("aria-hidden","true");let l=document.createElement("path");l.setAttribute("d","M4 6l4 4 4-4"),d.appendChild(l),c.appendChild(d),b.appendChild(c);let i=document.createElement("div");i.setAttribute("class","content-wrap"),i.setAttribute("part","content"),T(t,"contentWrap",i),i.setAttribute("role","region");let u=document.createElement("div");u.setAttribute("class","content-inner");let s=document.createElement("div");s.setAttribute("class","content-body");let f=document.createElement("slot");if(s.appendChild(f),u.appendChild(s),i.appendChild(u),b.appendChild(i),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(g,x){return g[x]},set(g,x,k){let w=g[x];if(w!=null&&typeof w.set==="function"){if(typeof k==="function")w.set(()=>k);else w.set(k);return!0}return!1}});if(h.mount){let g=h.mount(v,r);if(typeof g==="function")o.push(g)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var _r=`
sui-list-tile[variant='row'] + sui-list-tile[variant='row'] {
  border-top: 1px solid var(--border-subtle);
}
sui-list-tile[variant='row'][selected] + sui-list-tile[variant='row'],
sui-list-tile[variant='row'] + sui-list-tile[variant='row'][selected] {
  border-top: none;
}
`,to=new WeakSet;function Da(e){if(to.has(e))return;to.add(e);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let r=new CSSStyleSheet;r.replaceSync(_r),e.adoptedStyleSheets=[...e.adoptedStyleSheets,r];return}}catch{}let n=e.createElement("style");n.setAttribute("data-sui-list-tile-dividers",""),n.textContent=_r,e.head.appendChild(n)}var Jn=P({mount(e,n){let r=e;Da(n.ownerDocument);let o=()=>{if(r.navigable())return"link";if(r.interactive())return"button";return"listitem"},t=D(()=>{r.navigable(),r.interactive(),n.setAttribute("role",o())}),a=D(()=>{n.setAttribute("aria-selected",r.selected()?"true":"false")}),b=D(()=>{n.setAttribute("aria-disabled",r.disabled()?"true":"false")}),c=D(()=>{let i=r.tileEl();if(!i)return;let u=r.interactive(),s=r.disabled();if(u&&!s)i.setAttribute("tabindex","0");else i.removeAttribute("tabindex")}),m=()=>{if(r.disabled()||!r.interactive())return!1;let i=new CustomEvent("sui-select",{detail:{},bubbles:!0,composed:!0,cancelable:!0});return n.dispatchEvent(i),i.defaultPrevented},p=(i)=>{if(!r.interactive())return;if(m()&&r.navigable())i.preventDefault()},d=(i)=>{if(!r.interactive()||r.disabled())return;if(r.navigable()&&i.key==="Enter")return;if(i.key==="Enter"||i.key===" ")i.preventDefault(),m()},l=D(()=>{let i=r.tileEl();if(!i)return;return i.addEventListener("click",p),i.addEventListener("keydown",d),()=>{i.removeEventListener("click",p),i.removeEventListener("keydown",d)}});return()=>{t(),a(),b(),c(),l()}}});var Sm=z("sui-list-tile",{props:{variant:y.string({default:"row",reflect:!0}),interactive:y.bool({default:!1,reflect:!0}),navigable:y.string({default:"",reflect:!0}),selected:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),density:y.string({default:"normal",reflect:!0}),tone:y.string({default:"default",reflect:!0}),target:y.string({default:"",reflect:!0}),rel:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.variant=n.variant,t.interactive=n.interactive,t.navigable=n.navigable,t.selected=n.selected,t.disabled=n.disabled,t.density=n.density,t.tone=n.tone,t.target=n.target,t.rel=n.rel;let a=null;if(Jn&&typeof Jn.setup==="function"){if(e.scope=t,a=Jn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(p)=>F({anchor:p,test:()=>{if(t.navigable())return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("a");i.setAttribute("class","tile"),i.setAttribute("part","button"),T(t,"tileEl",i),l.push(E(i,"href",()=>t.navigable())),l.push(E(i,"target",()=>t.target()||null)),l.push(E(i,"rel",()=>t.rel()||null));let u=document.createElement("div");u.setAttribute("class","row"),u.setAttribute("part","row");let s=document.createElement("slot");s.setAttribute("name","leading"),u.appendChild(s);let f=document.createElement("span");f.setAttribute("class","primary"),f.setAttribute("part","primary");let h=document.createElement("slot");f.appendChild(h),u.appendChild(f);let v=document.createElement("span");v.setAttribute("part","trailing");let g=document.createElement("slot");g.setAttribute("name","trailing"),v.appendChild(g),u.appendChild(v),i.appendChild(u);let x=document.createElement("div");x.setAttribute("class","secondary"),x.setAttribute("part","secondary");let k=document.createElement("slot");return k.setAttribute("name","meta"),x.appendChild(k),i.appendChild(x),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let w of l)try{w()}catch(A){}}}},()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("div");i.setAttribute("class","tile"),i.setAttribute("part","button"),T(t,"tileEl",i);let u=document.createElement("div");u.setAttribute("class","row"),u.setAttribute("part","row");let s=document.createElement("slot");s.setAttribute("name","leading"),u.appendChild(s);let f=document.createElement("span");f.setAttribute("class","primary"),f.setAttribute("part","primary");let h=document.createElement("slot");f.appendChild(h),u.appendChild(f);let v=document.createElement("span");v.setAttribute("part","trailing");let g=document.createElement("slot");g.setAttribute("name","trailing"),v.appendChild(g),u.appendChild(v),i.appendChild(u);let x=document.createElement("div");x.setAttribute("class","secondary"),x.setAttribute("part","secondary");let k=document.createElement("slot");return k.setAttribute("name","meta"),x.appendChild(k),i.appendChild(x),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let w of l)try{w()}catch(A){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createComment("if");if(c.appendChild(m),o.push(b(m)),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var Yn=P({scope:(e,n)=>{let r=N(null),o=N(!1);return{caughtError:r,hasFallbackSlot:o,reset:()=>{if(r.set(null),n instanceof HTMLElement)n.removeAttribute("data-error-message")}}},mount(e,n){let{caughtError:r,hasFallbackSlot:o,reset:t}=e;if(!r||!o||!t)throw Error("[sui-error-boundary] scope signals missing — controller wiring is broken.");let a=n.querySelectorAll(':scope > template[slot="fallback"]');for(let b of Array.from(a)){let c=document.createElement("div");c.setAttribute("slot","fallback"),c.appendChild(b.content.cloneNode(!0)),b.replaceWith(c)}return o.set(!!n.querySelector(':scope > [slot="fallback"]')),He(n,{catch(b,c){if(r()!==null)return;let m=b instanceof Error?b:Error(String(b));n.setAttribute("data-error-message",m.message),r.set(m),n.dispatchEvent(new CustomEvent("sui-error-boundary:caught",{bubbles:!0,composed:!0,detail:{error:m,source:c}}))},reset:t}),n.reset=t,()=>{Se(n),delete n.reset}}});var qm=z("sui-error-boundary",{props:{"fallback-text":y.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.fallbackText=n.fallbackText;let a=null;if(Yn&&typeof Yn.setup==="function"){if(e.scope=t,a=Yn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>F({anchor:d,test:()=>{if(t.hasFallbackSlot())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("slot");return u.setAttribute("name","fallback"),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(f){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("div");u.setAttribute("class","fallback"),u.setAttribute("part","fallback"),u.setAttribute("role","alert");let s=document.createElement("div");s.setAttribute("class","message"),s.setAttribute("part","message");let f=document.createTextNode("");s.appendChild(f),i.push(B(f,()=>t.fallbackText()||"Something went wrong")),u.appendChild(s);let h=document.createElement("button");h.setAttribute("class","retry"),h.setAttribute("part","retry"),h.setAttribute("type","button"),i.push(K(h,"click",function(){let g=t.reset;return typeof g==="function"?g.bind(void 0):()=>{}}(),{modifiers:[]}));let v=document.createTextNode("Retry");return h.appendChild(v),u.appendChild(h),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let g of i)try{g()}catch(x){}}}}]}),c=(d)=>F({anchor:d,test:()=>{if(t.caughtError()===null)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("slot");return l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(f){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createComment("if");return l.appendChild(u),i.push(b(u)),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(f){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createComment("if");if(m.appendChild(p),o.push(c(p)),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var Wn=P({mount(e,n){let r=e,o=[],t=null,a=()=>{let i=r.target()||"body",u=document.querySelector(i);if(!u)return console.warn(`[sui-portal] target "${i}" not found in document`),null;return u},b=(i)=>{let u=r.portalSlot(),s=u?u.assignedNodes():Array.from(n.childNodes);for(let f of s)if(i.appendChild(f),!o.includes(f))o.push(f)},c=(i)=>{for(let u of o)i.appendChild(u)},m=()=>{for(let i of o)if(i.parentNode)i.parentNode.removeChild(i);o.length=0},p=D(()=>{let i=a();if(!i){t=null;return}if(t===null)b(i);else if(t!==i)c(i);t=i}),d=r.portalSlot(),l=()=>{if(!t||!d)return;for(let i of d.assignedNodes())if(!o.includes(i))t.appendChild(i),o.push(i)};if(d)d.addEventListener("slotchange",l);return()=>{if(p(),d)d.removeEventListener("slotchange",l);m()}}});var Qm=z("sui-portal",{props:{target:y.string({default:"body"})},shadow:!0,styles:`
  :host { display: none; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.target=n.target;let a=null;if(Wn&&typeof Wn.setup==="function"){if(e.scope=t,a=Wn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(T(t,"portalSlot",c),b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var Zn=P({mount(e,n){let r=e,o=!1,t=(m)=>{m.textContent=o?"Hide details":"Show details",m.setAttribute("aria-expanded",o?"true":"false")},a=D(()=>{let m=r.detailsSlot();if(!m)return;let p=()=>{if(m.assignedNodes({flatten:!0}).some((l)=>l.nodeType===Node.ELEMENT_NODE||l.nodeType===Node.TEXT_NODE&&(l.textContent??"").trim().length>0))n.setAttribute("data-has-details","");else{n.removeAttribute("data-has-details"),n.removeAttribute("data-details-open"),o=!1;let l=r.detailsToggleBtn();if(l)t(l)}};return m.addEventListener("slotchange",p),p(),()=>m.removeEventListener("slotchange",p)}),b=D(()=>{let m=r.detailsToggleBtn();if(!m)return;t(m);let p=()=>{if(o=!o,o)n.setAttribute("data-details-open","");else n.removeAttribute("data-details-open");t(m)};return m.addEventListener("click",p),()=>m.removeEventListener("click",p)}),c=D(()=>{let m=r.actionSlot();if(!m)return;let p=()=>{let d=m.assignedElements().find((l)=>l.getAttribute("slot")==="retry");if(d)requestAnimationFrame(()=>d.focus?.())};return m.addEventListener("slotchange",p),()=>m.removeEventListener("slotchange",p)});return()=>{a(),b(),c()}}});var tb=z("sui-error-state",{props:{title:y.string({default:""}),body:y.string({default:""}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:y.union(["default","inline"],{default:"default",reflect:!0}),icon:y.string({default:"alert-circle"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.title=n.title,t.body=n.body,t.size=n.size,t.variant=n.variant,t.icon=n.icon;let a=null;if(Zn&&typeof Zn.setup==="function"){if(e.scope=t,a=Zn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","box"),c.setAttribute("part","box"),c.setAttribute("role","alert");let m=document.createElement("div");m.setAttribute("class","icon"),m.setAttribute("part","icon"),m.setAttribute("aria-hidden","true");let p=document.createElement("sui-icon");o.push(E(p,"name",()=>t.icon())),p.setAttribute("size","md"),m.appendChild(p),c.appendChild(m);let d=document.createElement("div");d.setAttribute("class","copy");let l=document.createElement("div");l.setAttribute("class","title"),l.setAttribute("part","title");let i=document.createElement("slot");i.setAttribute("name","title");let u=document.createTextNode("");i.appendChild(u),o.push(B(u,()=>t.title()||"Something went wrong")),l.appendChild(i),d.appendChild(l);let s=document.createElement("div");s.setAttribute("class","desc"),s.setAttribute("part","desc");let f=document.createElement("slot"),h=document.createTextNode("");f.appendChild(h),o.push(B(h,()=>t.body())),s.appendChild(f),d.appendChild(s);let v=document.createElement("button");v.setAttribute("class","details-toggle"),T(t,"detailsToggleBtn",v),v.setAttribute("type","button"),d.appendChild(v);let g=document.createElement("div");g.setAttribute("class","details"),T(t,"detailsPanel",g);let x=document.createElement("slot");x.setAttribute("name","details"),T(t,"detailsSlot",x),g.appendChild(x),d.appendChild(g),c.appendChild(d);let k=document.createElement("div");k.setAttribute("class","actions"),k.setAttribute("part","actions");let w=document.createElement("slot");if(w.setAttribute("name","action"),T(t,"actionSlot",w),k.appendChild(w),c.appendChild(k),b.appendChild(c),a!=null){let A=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(A){let j=new Proxy(t,{get(C,M){return C[M]},set(C,M,O){let H=C[M];if(H!=null&&typeof H.set==="function"){if(typeof O==="function")H.set(()=>O);else H.set(O);return!0}return!1}});if(A.mount){let C=A.mount(j,r);if(typeof C==="function")o.push(C)}if(A.unmount)o.push(()=>A.unmount(j))}}return t}});var Qn=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","progressbar");let o=D(()=>{let t=Math.max(1,Math.floor(r.total()||0)),a=Math.max(1,Math.min(t,Math.floor(r.current()||1))),b=r.label();if(n.setAttribute("aria-valuemin","1"),n.setAttribute("aria-valuemax",String(t)),n.setAttribute("aria-valuenow",String(a)),b)n.setAttribute("aria-label",b);else n.setAttribute("aria-label",`Step ${a} of ${t}`);let c=r.dotsEl();if(!c)return;c.textContent="";for(let m=1;m<=t;m++){let p=document.createElement("span");p.className="dot",p.setAttribute("part","dot"),p.setAttribute("data-state",m===a?"active":m<a?"done":"pending"),p.setAttribute("aria-hidden","true"),c.appendChild(p)}});return()=>{o()}}});var ib=z("sui-stepper",{props:{total:y.number({default:4,reflect:!0}),current:y.number({default:1,reflect:!0}),tone:y.string({default:"",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.total=n.total,t.current=n.current,t.tone=n.tone,t.size=n.size,t.label=n.label;let a=null;if(Qn&&typeof Qn.setup==="function"){if(e.scope=t,a=Qn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");if(c.setAttribute("class","dots"),T(t,"dotsEl",c),b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var Oa=new Set(["done","failed","pending","running"]);function Ta(e){return e.split(",").map((n)=>n.trim().toLowerCase()).filter((n)=>n.length>0).map((n)=>Oa.has(n)?n:"pending")}var Vn=P({scope:()=>({segments:N([]),ariaLabel:N(null)}),mount(e,n){let r=e,o=r.segments,t=r.ariaLabel,a=D(()=>{let b=Ta(r.values()||"");o.set(b.map((c,m)=>({id:`s${m}`,state:c}))),t.set(b.length===0?null:`${b.length} step outcome${b.length===1?"":"s"}`)});return()=>a()}});var pb=z("sui-step-segments",{props:{values:y.string({default:"",reflect:!0}),size:y.union(["sm","md"],{default:"sm",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.values=n.values,t.size=n.size;let a=null;if(Vn&&typeof Vn.setup==="function"){if(e.scope=t,a=Vn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>R({anchor:d,items:()=>t.segments(),key:(l,i)=>{let u=Object.create(t);return u.seg=l,u.__i=i,((s)=>l.id)(u)},render:(l,i)=>{let u=Object.create(t);return u.seg=l,u.__i=i,u.__v=N(0),((s)=>{let f=document.createDocumentFragment(),h=[],v=document.createElement("span");return v.setAttribute("class","seg"),v.setAttribute("part","seg"),h.push(E(v,"data-state",()=>(s.__v(),s.seg.state))),f.appendChild(v),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let g of h)try{g()}catch(x){}},update:(g,x)=>{s.seg=g,s.__i=x,s.__v.set((k)=>k+1)}}})(u)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("span");m.setAttribute("class","row"),m.setAttribute("part","row"),o.push(E(m,"role",()=>t.segments().length>0?"img":null)),o.push(E(m,"aria-label",()=>t.ariaLabel()));let p=document.createComment("for");if(m.appendChild(p),o.push(b(p)),c.appendChild(m),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var Kn=P({mount(e,n){if(!n.hasAttribute("role"))n.setAttribute("role","list")}});var gb=z("sui-timeline",{shadow:!0,styles:`
  :host { display: block; position: relative; }
  .list { display: flex; flex-direction: column; gap: 4px; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(Kn&&typeof Kn.setup==="function"){if(e.scope=t,a=Kn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","list"),c.setAttribute("part","list");let m=document.createElement("slot");if(c.appendChild(m),b.appendChild(c),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var In=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","listitem");let o=D(()=>{if(r.interactive()){if(n.setAttribute("role","button"),!n.hasAttribute("tabindex"))n.setAttribute("tabindex","0")}else n.setAttribute("role","listitem"),n.removeAttribute("tabindex")}),t=()=>{n.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},a=()=>{if(r.interactive())t()},b=(c)=>{if(!r.interactive())return;if(c.key==="Enter"||c.key===" ")c.preventDefault(),t()};return n.addEventListener("click",a),n.addEventListener("keydown",b),()=>{o(),n.removeEventListener("click",a),n.removeEventListener("keydown",b)}}});var Eb=z("sui-timeline-item",{props:{tone:y.string({default:"default",reflect:!0}),at:y.string({default:"",reflect:!0}),interactive:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.at=n.at,t.interactive=n.interactive;let a=null;if(In&&typeof In.setup==="function"){if(e.scope=t,a=In.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(h)=>F({anchor:h,test:()=>{if(t.at())return 0;return-1},branches:[()=>{let v=document.createDocumentFragment(),g=[],x=document.createElement("span");x.setAttribute("class","at"),x.setAttribute("part","at");let k=document.createTextNode("");return x.appendChild(k),g.push(B(k,()=>t.at())),v.appendChild(x),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let w of g)try{w()}catch(A){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("div");m.setAttribute("class","gutter");let p=document.createElement("span");p.setAttribute("class","dot"),p.setAttribute("part","dot"),m.appendChild(p),c.appendChild(m);let d=document.createElement("div");d.setAttribute("class","body"),d.setAttribute("part","body");let l=document.createElement("div");l.setAttribute("class","head");let i=document.createElement("slot");i.setAttribute("name","title"),l.appendChild(i);let u=document.createComment("if");l.appendChild(u),o.push(b(u)),d.appendChild(l);let s=document.createElement("div");s.setAttribute("class","content");let f=document.createElement("slot");if(s.appendChild(f),d.appendChild(s),c.appendChild(d),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(g,x){return g[x]},set(g,x,k){let w=g[x];if(w!=null&&typeof w.set==="function"){if(typeof k==="function")w.set(()=>k);else w.set(k);return!0}return!1}});if(h.mount){let g=h.mount(v,r);if(typeof g==="function")o.push(g)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var Pa=["Waking daemon","Warming routes","Fetching state","Hydrating shell"];function no(e){let n=e.trim();if(!n)return Pa;return n.split("|").map((r)=>r.trim()).filter(Boolean)}var $n=P({mount(e,n){let r=e,o=r.msgEl();if(!o)return;let t=0;(()=>{let c=no(r.messages());o.textContent=c[t]??"",o.style.opacity="1"})();let b=setInterval(()=>{o.style.opacity="0",requestAnimationFrame(()=>{let c=no(r.messages());t=(t+1)%c.length,o.textContent=c[t]??"",o.style.opacity="1"})},1400);return()=>clearInterval(b)}});var Ob=z("sui-loading",{props:{messages:y.string({default:""}),compact:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.messages=n.messages,t.compact=n.compact;let a=null;if($n&&typeof $n.setup==="function"){if(e.scope=t,a=$n.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","stage"),c.setAttribute("part","stage"),c.setAttribute("aria-hidden","true");let m=document.createElement("div");m.setAttribute("class","rocket");let p=document.createElement("svg");p.setAttribute("viewbox","0 0 32 40"),p.setAttribute("fill","none"),p.setAttribute("xmlns","http://www.w3.org/2000/svg");let d=document.createElement("path");d.setAttribute("d","M16 2 C20 6 22 11 22 16 H10 C10 11 12 6 16 2 Z"),d.setAttribute("fill","currentColor"),p.appendChild(d);let l=document.createElement("rect");l.setAttribute("x","10"),l.setAttribute("y","16"),l.setAttribute("width","12"),l.setAttribute("height","14"),l.setAttribute("rx","2"),l.setAttribute("fill","currentColor"),l.setAttribute("opacity","0.9"),p.appendChild(l);let i=document.createElement("circle");i.setAttribute("cx","16"),i.setAttribute("cy","20"),i.setAttribute("r","2.5"),i.setAttribute("fill","var(--background, #fff)"),i.setAttribute("opacity","0.85"),p.appendChild(i);let u=document.createElement("path");u.setAttribute("d","M10 24 L4 32 L10 30 Z"),u.setAttribute("fill","currentColor"),u.setAttribute("opacity","0.75"),p.appendChild(u);let s=document.createElement("path");s.setAttribute("d","M22 24 L28 32 L22 30 Z"),s.setAttribute("fill","currentColor"),s.setAttribute("opacity","0.75"),p.appendChild(s);let f=document.createElement("rect");f.setAttribute("x","13"),f.setAttribute("y","29"),f.setAttribute("width","6"),f.setAttribute("height","2"),f.setAttribute("rx","0.5"),f.setAttribute("fill","var(--background, #000)"),f.setAttribute("opacity","0.5"),p.appendChild(f),m.appendChild(p),c.appendChild(m);let h=document.createElement("div");h.setAttribute("class","flame"),c.appendChild(h);let v=document.createElement("div");v.setAttribute("class","particle p1"),c.appendChild(v);let g=document.createElement("div");g.setAttribute("class","particle p2"),c.appendChild(g);let x=document.createElement("div");x.setAttribute("class","particle p3"),c.appendChild(x),b.appendChild(c);let k=document.createElement("div");if(k.setAttribute("class","message"),k.setAttribute("part","message"),T(t,"msgEl",k),b.appendChild(k),a!=null){let w=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(w){let A=new Proxy(t,{get(j,C){return j[C]},set(j,C,M){let O=j[C];if(O!=null&&typeof O.set==="function"){if(typeof M==="function")O.set(()=>M);else O.set(M);return!0}return!1}});if(w.mount){let j=w.mount(A,r);if(typeof j==="function")o.push(j)}if(w.unmount)o.push(()=>w.unmount(A))}}return t}});function eo(e){let n=document.getElementById("ship-view-root");if(!n)return null;let r=e.target;if(!r||!n.contains(r))return null;if(r.closest("sui-edit-overlay"))return null;return r}function ro(e){let n=e.tagName.toLowerCase(),r=e.id||null,o=n;if(r)o=`#${r}`;else if(e.classList.length>0)o=`${n}.${Array.from(e.classList).slice(0,2).join(".")}`;return{tag:n,selector:o,id:r}}var _n=P({mount(e,n){let r=e,o=r.boxEl(),t=r.labelEl(),a=null;function b(s){if(!o||!t)return;if(!s){o.classList.remove("visible"),t.classList.remove("visible");return}o.style.cssText=`top:${s.top}px;left:${s.left}px;width:${s.width}px;height:${s.height}px`,o.classList.add("visible"),t.style.cssText=`top:${s.top}px;left:${s.left}px`,t.textContent=`<${s.tag}${s.id?`#${s.id}`:""}>`,t.classList.add("visible")}let c=(s)=>{if(a)return;a=requestAnimationFrame(()=>{a=null;let f=eo(s);if(!f){b(null);return}let h=f.getBoundingClientRect(),{tag:v,id:g}=ro(f);b({top:h.top,left:h.left,width:h.width,height:h.height,tag:v,id:g})})},m=()=>{b(null)},p=(s)=>{let f=eo(s);if(!f)return;s.preventDefault(),s.stopPropagation();let{tag:h,selector:v,id:g}=ro(f);(n.closest("sui-shell")??n.closest("ship-shell"))?.dispatchEvent(new CustomEvent("sui:chat:annotate",{detail:{tag:h,selector:v,id:g,preview:(f.textContent??"").trim().slice(0,60)},bubbles:!0})),window.ship?.chat?.open?.()};function d(){let s=document.getElementById("ship-view-root");if(!s)return;s.addEventListener("pointermove",c,!0),s.addEventListener("pointerleave",m,!0),s.addEventListener("click",p,!0)}function l(){let s=document.getElementById("ship-view-root");if(!s)return;s.removeEventListener("pointermove",c,!0),s.removeEventListener("pointerleave",m,!0),s.removeEventListener("click",p,!0)}function i(){let s=document.body.classList.contains("edit-mode");if(n.classList.toggle("active",s),s)d();else if(l(),b(null),a)cancelAnimationFrame(a),a=null}i();let u=new MutationObserver(()=>i());return u.observe(document.body,{attributes:!0,attributeFilter:["class"]}),()=>{if(u.disconnect(),l(),a)cancelAnimationFrame(a),a=null}}});var Sb=z("sui-edit-overlay",{shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(_n&&typeof _n.setup==="function"){if(e.scope=t,a=_n.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","box"),c.setAttribute("part","box"),T(t,"boxEl",c),b.appendChild(c);let m=document.createElement("div");m.setAttribute("class","label"),m.setAttribute("part","label"),T(t,"labelEl",m),b.appendChild(m);let p=document.createElement("div");p.setAttribute("class","hint"),p.setAttribute("part","hint");let d=document.createTextNode(`
  Edit mode — hover an element, click to annotate.
  `);p.appendChild(d);let l=document.createElement("kbd"),i=document.createTextNode("⌘⇧E");l.appendChild(i),p.appendChild(l);let u=document.createTextNode(` to exit.
`);if(p.appendChild(u),b.appendChild(p),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let f=new Proxy(t,{get(h,v){return h[v]},set(h,v,g){let x=h[v];if(x!=null&&typeof x.set==="function"){if(typeof g==="function")x.set(()=>g);else x.set(g);return!0}return!1}});if(s.mount){let h=s.mount(f,r);if(typeof h==="function")o.push(h)}if(s.unmount)o.push(()=>s.unmount(f))}}return t}});function Ma(e=Date.now()){let n=e,r=[,,,,,,,,,,];for(let a=9;a>=0;a--)r[a]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[n%32],n=Math.floor(n/32);let o=Array(16),t=typeof globalThis<"u"?globalThis.crypto:void 0;if(t?.getRandomValues){let a=new Uint8Array(16);t.getRandomValues(a);for(let b=0;b<16;b++)o[b]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[a[b]%32]}else for(let a=0;a<16;a++)o[a]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[Math.floor(Math.random()*32)];return r.join("")+o.join("")}function oo(e){let n={};if(e.viewId)n.view_id=e.viewId;if(e.selector)n.selector=e.selector;if(e.rect)n.rect=e.rect;return{id:Ma(),kind:e.kind,target:n,note:e.note,created_at:new Date().toISOString(),author_kind:"human"}}var Na="/api/feedback",te=P({scope:(e,n)=>{let{viewId:r,mode:o,endpoint:t,staging:a}=e,b=N([]),c=N(null),m=N(null),p=N(""),d={},l=Z(()=>{let w=m();if(!w)return!1;return Math.abs(w.curX-w.startX)>4||Math.abs(w.curY-w.startY)>4}),i=Z(()=>{let w=m();if(!w)return"";let A=Math.min(w.startX,w.curX),j=Math.min(w.startY,w.curY),C=Math.abs(w.curX-w.startX),M=Math.abs(w.curY-w.startY);return`left:${A}px;top:${j}px;width:${C}px;height:${M}px`}),u=Z(()=>{let w=c();if(!w)return"";let A=Math.min(w.x+12,1e4),j=Math.min(w.y+12,1e4);return`left:${A}px;top:${j}px`}),s=Z(()=>b().map((w,A)=>({id:w.id,kind:w.kind,index:A,rect:w.kind==="region"&&w.target.rect?w.target.rect.join(","):"",pos:d[w.id]??{x:0,y:0}})));async function f(w){let A=t()||Na;try{let j=await fetch(A,{method:"POST",credentials:"include",headers:{"Content-Type":"application/json"},body:JSON.stringify(w)});if(!j.ok){n.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:`http_${j.status}`,id:w.id},bubbles:!0,composed:!0}));return}b.set(b().filter((C)=>C.id!==w.id)),n.dispatchEvent(new CustomEvent("sui-annotation-sent",{detail:{id:w.id,status:j.status},bubbles:!0,composed:!0}))}catch(j){n.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:"network_error",message:String(j),id:w.id},bubbles:!0,composed:!0}))}}function h(w){let A=c();if(!A)return;let j=w.trim();if(!j){c.set(null),p.set("");return}let C=oo({viewId:r()||void 0,kind:A.kind,selector:A.kind==="point"?A.selector:void 0,rect:A.kind==="region"?A.rect:void 0,note:j});if(d[C.id]={x:A.x,y:A.y},b.set([...b(),C]),n.dispatchEvent(new CustomEvent("sui-annotation-staged",{detail:C,bubbles:!0,composed:!0})),c.set(null),p.set(""),!a())f(C)}let v=()=>{h(p())},g=()=>{c.set(null),p.set("")},x=(w)=>{if(w.key==="Escape")g();else if(w.key==="Enter"&&(w.metaKey||w.ctrlKey)){let A=w.currentTarget;h(A.value)}},k=J(null);return{staged:b,pending:c,drag:m,noteText:p,dragVisible:l,dragRectStyle:i,popoverStyle:u,markers:s,overlayEl:k,confirmNote:h,sendNote:v,cancelPending:g,onNoteKeydown:x,sendOne:f}},mount(e,n){let r=e,o=null,t=null,a=(d)=>{let l=r.overlayEl();if(!l)return{x:0,y:0};let i=l.getBoundingClientRect();return{x:d.clientX-i.left,y:d.clientY-i.top}},b=(d)=>{if(r.mode()==="off")return;let i=d.target;if(i.closest("[data-annotation-popover]")||i.closest("sui-annotation-marker"))return;if(r.pending()){r.pending.set(null),r.noteText.set("");return}d.preventDefault();let u=a(d);o={startX:u.x,startY:u.y,curX:u.x,curY:u.y},r.drag.set(o),d.target.setPointerCapture?.(d.pointerId)},c=(d)=>{if(!o)return;let l=a(d);if(o={...o,curX:l.x,curY:l.y},t!==null)return;t=requestAnimationFrame(()=>{if(t=null,o)r.drag.set({...o})})},m=(d)=>{if(!o)return;let l=a(d),i=Math.abs(l.x-o.startX),u=Math.abs(l.y-o.startY),s=r.mode(),f=null;if(s==="point"||s==="region"&&i<8&&u<8)f={kind:"point",x:l.x,y:l.y,selector:"body"};else if(s==="region"){let h=Math.min(o.startX,l.x),v=Math.min(o.startY,l.y),g=[Math.round(h),Math.round(v),Math.round(i),Math.round(u)];f={kind:"region",x:h+i/2,y:v+u/2,rect:g}}if(o=null,r.drag.set(null),r.noteText.set(""),f)r.pending.set(f)},p=r.overlayEl();if(p)p.addEventListener("pointerdown",b),p.addEventListener("pointermove",c),p.addEventListener("pointerup",m);return Object.defineProperty(n,"staged",{configurable:!0,get:()=>r.staged()}),n.send=async()=>{for(let d of[...r.staged()])await r.sendOne(d)},n.clear=()=>{r.staged.set([])},()=>{if(t!==null)cancelAnimationFrame(t),t=null;if(p)p.removeEventListener("pointerdown",b),p.removeEventListener("pointermove",c),p.removeEventListener("pointerup",m)}}});var Jb=z("sui-annotation-layer",{props:{"view-id":y.string({default:""}),mode:y.union(["off","point","region"],{default:"off",reflect:!0}),endpoint:y.string({default:"/api/feedback"}),staging:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.viewId=n.viewId,t.mode=n.mode,t.endpoint=n.endpoint,t.staging=n.staging;let a=null;if(te&&typeof te.setup==="function"){if(e.scope=t,a=te.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(h)=>F({anchor:h,test:()=>{if(t.dragVisible())return 0;return-1},branches:[()=>{let v=document.createDocumentFragment(),g=[],x=document.createElement("div");return x.setAttribute("class","drag-rect"),g.push(E(x,"style",()=>t.dragRectStyle())),v.appendChild(x),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let k of g)try{k()}catch(w){}}}}]}),c=(h)=>R({anchor:h,items:()=>t.markers(),key:(v,g)=>{let x=Object.create(t);return x.m=v,x.__i=g,((k)=>v.id)(x)},render:(v,g)=>{let x=Object.create(t);return x.m=v,x.__i=g,x.__v=N(0),((k)=>{let w=document.createDocumentFragment(),A=[],j=document.createElement("sui-annotation-marker");return A.push(tt(j,"kind",()=>(k.__v(),k.m.kind))),A.push(tt(j,"index",()=>(k.__v(),k.m.index))),A.push(tt(j,"rect",()=>(k.__v(),k.m.rect))),A.push(tt(j,"x",()=>(k.__v(),k.m.pos.x))),A.push(tt(j,"y",()=>(k.__v(),k.m.pos.y))),w.appendChild(j),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of A)try{C()}catch(M){}},update:(C,M)=>{k.m=C,k.__i=M,k.__v.set((O)=>O+1)}}})(x)}}),m=(h)=>F({anchor:h,test:()=>{if(t.pending())return 0;return-1},branches:[()=>{let v=document.createDocumentFragment(),g=[],x=document.createElement("div");x.setAttribute("class","popover"),x.setAttribute("data-annotation-popover",""),g.push(E(x,"style",()=>t.popoverStyle())),g.push(K(x,"pointerdown",function(){let O=(H)=>H.stopPropagation();return typeof O==="function"?O.bind(void 0):()=>{}}(),{modifiers:[]}));let k=document.createElement("textarea");k.setAttribute("part","note-input"),k.setAttribute("placeholder","Add a note…"),g.push(mt(k,t.noteText)),g.push(K(k,"keydown",function(){let O=t.onNoteKeydown;return typeof O==="function"?O.bind(void 0):()=>{}}(),{modifiers:[]})),x.appendChild(k);let w=document.createElement("div");w.setAttribute("class","popover-actions");let A=document.createElement("button");A.setAttribute("type","button"),g.push(K(A,"click",function(){let O=t.cancelPending;return typeof O==="function"?O.bind(void 0):()=>{}}(),{modifiers:[]}));let j=document.createTextNode("Cancel");A.appendChild(j),w.appendChild(A);let C=document.createElement("button");C.setAttribute("type","button"),C.setAttribute("data-primary",""),g.push(K(C,"click",function(){let O=t.sendNote;return typeof O==="function"?O.bind(void 0):()=>{}}(),{modifiers:[]}));let M=document.createTextNode("Send");return C.appendChild(M),w.appendChild(C),x.appendChild(w),v.appendChild(x),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let O of g)try{O()}catch(H){}}}}]}),p=r.shadowRoot??r;L(p);let d=document.createElement("div");d.setAttribute("class","surface"),d.setAttribute("part","surface");let l=document.createElement("slot");d.appendChild(l);let i=document.createElement("div");i.setAttribute("class","overlay"),i.setAttribute("part","overlay"),T(t,"overlayEl",i);let u=document.createComment("if");i.appendChild(u),o.push(b(u));let s=document.createComment("for");i.appendChild(s),o.push(c(s));let f=document.createComment("if");if(i.appendChild(f),o.push(m(f)),d.appendChild(i),p.appendChild(d),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(g,x){return g[x]},set(g,x,k){let w=g[x];if(w!=null&&typeof w.set==="function"){if(typeof k==="function")w.set(()=>k);else w.set(k);return!0}return!1}});if(h.mount){let g=h.mount(v,r);if(typeof g==="function")o.push(g)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var ne=P({mount(e,n){let r=e;return D(()=>{let t=r.pinEl(),a=r.boxEl(),b=r.x(),c=r.y(),m=r.index(),p=r.rect();if(t)t.style.left=`${b}px`,t.style.top=`${c}px`,t.textContent=String(m+1);if(a&&p){let d=p.split(",").map(Number);if(d.length===4&&d.every((l)=>Number.isFinite(l))){let[l,i,u,s]=d;a.style.cssText=`left:${l}px;top:${i}px;width:${u}px;height:${s}px`}}})}});var Kb=z("sui-annotation-marker",{props:{kind:y.union(["point","region"],{default:"point",reflect:!0}),x:y.number({default:0}),y:y.number({default:0}),rect:y.string({default:""}),active:y.bool({default:!1,reflect:!0}),index:y.number({default:0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.kind=n.kind,t.x=n.x,t.y=n.y,t.rect=n.rect,t.active=n.active,t.index=n.index;let a=null;if(ne&&typeof ne.setup==="function"){if(e.scope=t,a=ne.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("part","pin"),c.setAttribute("role","button"),c.setAttribute("tabindex","0"),T(t,"pinEl",c),b.appendChild(c);let m=document.createElement("div");if(m.setAttribute("part","box"),m.setAttribute("role","button"),m.setAttribute("tabindex","0"),T(t,"boxEl",m),b.appendChild(m),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var ee=P({scope:(e,n)=>{return{staged:N([]),emitRemove:(t)=>{n.dispatchEvent(new CustomEvent("sui-annotation-remove",{detail:{id:t},bubbles:!0,composed:!0}))}}},mount(e,n){let o=e.staged,t=n.staged;if(t!=null)o.set(Array.isArray(t)?t:[]);Object.defineProperty(n,"staged",{configurable:!0,get:()=>o(),set:(a)=>o.set(Array.isArray(a)?a:[])})}});var eh=z("sui-annotation-list",{shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(ee&&typeof ee.setup==="function"){if(e.scope=t,a=ee.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>R({anchor:d,items:()=>t.staged(),key:(l,i)=>{let u=Object.create(t);return u.a=l,u.__i=i,((s)=>l.id)(u)},render:(l,i)=>{let u=Object.create(t);return u.a=l,u.__i=i,u.__v=N(0),((s)=>{let f=document.createDocumentFragment(),h=[],v=document.createElement("li");v.setAttribute("part","item"),h.push(E(v,"data-id",()=>(s.__v(),s.a.id)));let g=document.createElement("span");g.setAttribute("class","kind");let x=document.createTextNode("");g.appendChild(x),h.push(B(x,()=>(s.__v(),s.a.kind))),v.appendChild(g);let k=document.createElement("span");k.setAttribute("class","note"),h.push(E(k,"title",()=>(s.__v(),s.a.note??"")));let w=document.createTextNode("");k.appendChild(w),h.push(B(w,()=>(s.__v(),s.a.note??"(no note)"))),v.appendChild(k);let A=document.createElement("button");A.setAttribute("type","button"),A.setAttribute("aria-label","Remove annotation"),h.push(K(A,"click",function(){let C=()=>s.emitRemove(s.a.id);return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let j=document.createTextNode("Remove");return A.appendChild(j),v.appendChild(A),f.appendChild(v),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let C of h)try{C()}catch(M){}},update:(C,M)=>{s.a=C,s.__i=M,s.__v.set((O)=>O+1)}}})(u)}}),c=(d)=>F({anchor:d,test:()=>{if(t.staged().length===0)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("div");u.setAttribute("class","empty"),u.setAttribute("part","empty");let s=document.createTextNode("No staged annotations.");return u.appendChild(s),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let f of i)try{f()}catch(h){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("ol");u.setAttribute("part","list");let s=document.createComment("for");return u.appendChild(s),i.push(b(s)),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let f of i)try{f()}catch(h){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createComment("if");if(m.appendChild(p),o.push(c(p)),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var re=P({mount(e,n){let r=e,o=D(()=>{let t=r.inputEl();if(!t)return;let a=()=>{if(r.disabled())return;n.dispatchEvent(new CustomEvent("sui-radio-select",{detail:{value:r.value()},bubbles:!0,composed:!0}))},b=(c)=>{if(!["ArrowDown","ArrowRight","ArrowUp","ArrowLeft"].includes(c.key))return;c.preventDefault();let p=c.key==="ArrowDown"||c.key==="ArrowRight";n.dispatchEvent(new CustomEvent("sui-radio-nav",{detail:{direction:p?"next":"prev"},bubbles:!0,composed:!0}))};return t.addEventListener("click",a),t.addEventListener("keydown",b),()=>{t.removeEventListener("click",a),t.removeEventListener("keydown",b)}});return n.focusInput=()=>{n.shadowRoot?.querySelector("input")?.focus()},()=>o()}});var uh=z("sui-radio",{props:{value:y.string({default:"",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),checked:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.disabled=n.disabled,t.checked=n.checked,t.tone=n.tone,t.size=n.size;let a=null;if(re&&typeof re.setup==="function"){if(e.scope=t,a=re.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("label");c.setAttribute("part","label");let m=document.createElement("span");m.setAttribute("class","circle"),m.setAttribute("part","circle");let p=document.createElement("input");p.setAttribute("part","input"),p.setAttribute("type","radio"),o.push(bt(p,t.checked)),o.push(E(p,"disabled",()=>t.disabled())),o.push(E(p,"aria-checked",()=>t.checked()?"true":"false")),o.push(E(p,"tabindex",()=>t.checked()?"0":"-1")),T(t,"inputEl",p),m.appendChild(p);let d=document.createElement("span");d.setAttribute("class","dot"),d.setAttribute("part","dot"),d.setAttribute("aria-hidden","true"),m.appendChild(d),c.appendChild(m);let l=document.createElement("slot");if(c.appendChild(l),b.appendChild(c),a!=null){let i=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(i){let u=new Proxy(t,{get(s,f){return s[f]},set(s,f,h){let v=s[f];if(v!=null&&typeof v.set==="function"){if(typeof h==="function")v.set(()=>h);else v.set(h);return!0}return!1}});if(i.mount){let s=i.mount(u,r);if(typeof s==="function")o.push(s)}if(i.unmount)o.push(()=>i.unmount(u))}}return t}});var oe=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","radiogroup");let o=document.createElement("input");o.type="hidden",n.appendChild(o);let t=()=>Array.from(n.querySelectorAll(":scope > sui-radio")),a=()=>t().filter((l)=>!l.hasAttribute("disabled")&&!r.disabled()),b=()=>{let l=r.value(),i=r.disabled(),u=r.tone(),s=r.size();for(let f of t()){if(f.toggleAttribute("checked",f.getAttribute("value")===l),i)f.setAttribute("disabled","");if(u&&!f.hasAttribute("tone"))f.setAttribute("tone",u);if(s&&!f.hasAttribute("size"))f.setAttribute("size",s)}o.name=r.name(),o.value=l},c=D(()=>{r.value(),r.disabled(),r.tone(),r.size(),b()}),m=D(()=>{let l=r.label();if(l)n.setAttribute("aria-label",l);else n.removeAttribute("aria-label")}),p=(l)=>{let i=l;i.stopPropagation();let u=i.detail.value;if(u===r.value())return;r.value.set(u),b(),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:u},bubbles:!0,composed:!0}))},d=(l)=>{let i=l;i.stopPropagation();let u=a();if(!u.length)return;let s=i.composedPath().find((g)=>g.tagName?.toLowerCase()==="sui-radio"),f=s?u.indexOf(s):-1,h=i.detail.direction==="next"?u[(f+1)%u.length]:u[(f-1+u.length)%u.length],v=h.getAttribute("value")??"";r.value.set(v),b(),h.focusInput?.(),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:v},bubbles:!0,composed:!0}))};return n.addEventListener("sui-radio-select",p),n.addEventListener("sui-radio-nav",d),()=>{c(),m(),n.removeEventListener("sui-radio-select",p),n.removeEventListener("sui-radio-nav",d),o.remove()}}});var mh=z("sui-radio-group",{props:{value:y.string({default:"",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),name:y.string({default:""}),label:y.string({default:""})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    gap: var(--space-2, 8px);
    font-family: var(--font-body);
    color: var(--foreground);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.disabled=n.disabled,t.tone=n.tone,t.size=n.size,t.name=n.name,t.label=n.label;let a=null;if(oe&&typeof oe.setup==="function"){if(e.scope=t,a=oe.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var ae=P({mount(e,n){let r=e,o=document.createElement("input");o.type="hidden",n.appendChild(o);let t=(u)=>{let s=r.step(),f=r.min(),h=r.max(),v=Math.round((u-f)/s)*s+f;return Math.max(f,Math.min(h,v))},a=(u)=>{let s=t(u);if(s===r.value())return;r.value.set(s),o.name=r.name(),o.value=String(s),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:s},bubbles:!0,composed:!0}))},b=(u)=>{let s=r.trackEl();if(!s)return r.value();let f=s.getBoundingClientRect(),h=Math.max(0,Math.min(1,(u-f.left)/f.width));return r.min()+h*(r.max()-r.min())},c=D(()=>{let u=r.value(),s=r.min(),h=r.max()-s||1,v=(u-s)/h*100,g=r.fillEl(),x=r.thumbEl();if(g)g.style.width=`${v}%`;if(x)x.style.left=`${v}%`;o.name=r.name(),o.value=String(u)}),m=!1,p=(u)=>{if(r.disabled())return;u.preventDefault(),m=!0,a(b(u.clientX)),r.thumbEl()?.focus();let f=(v)=>{if(m)a(b(v.clientX))},h=()=>{m=!1,document.removeEventListener("pointermove",f),document.removeEventListener("pointerup",h)};document.addEventListener("pointermove",f),document.addEventListener("pointerup",h)},d=(u)=>{if(r.disabled())return;let s=r.step(),f=!0;switch(u.key){case"ArrowRight":case"ArrowUp":a(r.value()+s);break;case"ArrowLeft":case"ArrowDown":a(r.value()-s);break;case"Home":a(r.min());break;case"End":a(r.max());break;case"PageUp":a(r.value()+s*10);break;case"PageDown":a(r.value()-s*10);break;default:f=!1}if(f)u.preventDefault()},l=D(()=>{let u=r.containerEl();if(!u)return;return u.addEventListener("pointerdown",p),()=>u.removeEventListener("pointerdown",p)}),i=D(()=>{let u=r.thumbEl();if(!u)return;return u.addEventListener("keydown",d),()=>u.removeEventListener("keydown",d)});return()=>{c(),l(),i(),o.remove()}}});var xh=z("sui-slider",{props:{value:y.number({default:0,reflect:!0}),min:y.number({default:0}),max:y.number({default:100}),step:y.number({default:1}),disabled:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),name:y.string({default:""}),label:y.string({default:"slider"})},shadow:!0,styles:`
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

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.min=n.min,t.max=n.max,t.step=n.step,t.disabled=n.disabled,t.tone=n.tone,t.size=n.size,t.name=n.name,t.label=n.label;let a=null;if(ae&&typeof ae.setup==="function"){if(e.scope=t,a=ae.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","container"),T(t,"containerEl",c);let m=document.createElement("div");m.setAttribute("class","track"),m.setAttribute("part","track"),T(t,"trackEl",m);let p=document.createElement("div");p.setAttribute("class","fill"),p.setAttribute("part","fill"),T(t,"fillEl",p),m.appendChild(p),c.appendChild(m);let d=document.createElement("div");if(d.setAttribute("class","thumb"),d.setAttribute("part","thumb"),d.setAttribute("role","slider"),o.push(E(d,"tabindex",()=>t.disabled()?"-1":"0")),o.push(E(d,"aria-valuemin",()=>t.min())),o.push(E(d,"aria-valuemax",()=>t.max())),o.push(E(d,"aria-valuenow",()=>t.value())),o.push(E(d,"aria-label",()=>t.label()||"slider")),o.push(E(d,"aria-disabled",()=>t.disabled()?"true":"false")),T(t,"thumbEl",d),c.appendChild(d),b.appendChild(c),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(u,s){return u[s]},set(u,s,f){let h=u[s];if(h!=null&&typeof h.set==="function"){if(typeof f==="function")h.set(()=>f);else h.set(f);return!0}return!1}});if(l.mount){let u=l.mount(i,r);if(typeof u==="function")o.push(u)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var kh=z("sui-segmented-option",{props:{value:y.string({default:"",reflect:!0}),icon:y.string({default:"",reflect:!0}),disabled:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.icon=n.icon,t.disabled=n.disabled;let a=r.shadowRoot??r;L(a);let b=document.createTextNode(`

`);a.appendChild(b);let c=document.createTextNode(`
`);a.appendChild(c);let m=document.createTextNode(`
`);a.appendChild(m);let p=document.createTextNode(`

`);a.appendChild(p);let d=document.createTextNode(`
`);return a.appendChild(d),t}});var Ha=(e)=>typeof CSS<"u"&&CSS.escape?CSS.escape(e):e.replace(/[^\w-]/g,"\\$&"),ie=P({scope:()=>{let e=N([]),n=J(null);return{options:e,containerEl:n}},mount(e,n){let r=e,o=r.options;if(!n.hasAttribute("role"))n.setAttribute("role","radiogroup");let t=document.createElement("input");t.type="hidden",n.appendChild(t);let a=()=>{return Array.from(n.querySelectorAll(":scope > sui-segmented-option")).map((u)=>({value:u.getAttribute("value")??"",label:u.textContent?.trim()??"",icon:u.getAttribute("icon")||null,disabled:u.hasAttribute("disabled")}))},b=(i)=>{if(r.disabled())return;let s=o().find((f)=>f.value===i);if(!s||s.disabled)return;if(r.value()===i)return;r.value.set(i),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:i},bubbles:!0,composed:!0}))},c=(i)=>{let u=i.target?.closest(".btn");if(!u)return;let s=u.getAttribute("data-value");if(s!==null)b(s)},m=(i)=>{let s=o().filter((k)=>!k.disabled&&!r.disabled());if(!s.length)return;let f=s.map((k)=>k.value),h=r.value(),v=Math.max(0,f.indexOf(h)),g=v;if(i.key==="ArrowRight"||i.key==="ArrowDown")i.preventDefault(),g=(v+1)%f.length;else if(i.key==="ArrowLeft"||i.key==="ArrowUp")i.preventDefault(),g=(v-1+f.length)%f.length;else if(i.key==="Enter"||i.key===" "){i.preventDefault();let w=n.shadowRoot.activeElement?.getAttribute("data-value");if(w)b(w);return}else return;let x=f[g]??"";b(x),queueMicrotask(()=>{n.shadowRoot?.querySelector(`.btn[data-value="${Ha(x)}"]`)?.focus()})};o.set(a());let p=new MutationObserver(()=>o.set(a()));p.observe(n,{childList:!0,subtree:!1});let d=D(()=>{t.name=r.name(),t.value=r.value()}),l=D(()=>{let i=r.containerEl();if(!i)return;return i.addEventListener("click",c),i.addEventListener("keydown",m),()=>{i.removeEventListener("click",c),i.removeEventListener("keydown",m)}});return()=>{p.disconnect(),d(),l(),t.remove()}}});var Dh=z("sui-segmented",{props:{value:y.string({default:"",reflect:!0}),size:y.union(["sm","md"],{default:"md",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),name:y.string({default:""})},shadow:!0,styles:`
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

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.size=n.size,t.disabled=n.disabled,t.name=n.name;let a=null;if(ie&&typeof ie.setup==="function"){if(e.scope=t,a=ie.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>R({anchor:d,items:()=>t.options(),key:(l,i)=>{let u=Object.create(t);return u.opt=l,u.__i=i,((s)=>l.value)(u)},render:(l,i)=>{let u=Object.create(t);return u.opt=l,u.__i=i,u.__v=N(0),((s)=>{let f=(w)=>F({anchor:w,test:()=>{if(s.opt.icon)return 0;return-1},branches:[()=>{let A=document.createDocumentFragment(),j=[],C=document.createElement("span");C.setAttribute("class","btn-icon"),C.setAttribute("aria-hidden","true");let M=document.createElement("sui-icon");return j.push(tt(M,"name",()=>(s.__v(),s.opt.icon))),M.setAttribute("size","sm"),C.appendChild(M),A.appendChild(C),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let O of j)try{O()}catch(H){}}}}]}),h=document.createDocumentFragment(),v=[],g=document.createElement("button");g.setAttribute("class","btn"),g.setAttribute("role","radio"),g.setAttribute("type","button"),v.push(E(g,"data-value",()=>(s.__v(),s.opt.value))),v.push(E(g,"aria-checked",()=>(s.__v(),s.opt.value===s.value()?"true":"false"))),v.push(E(g,"aria-disabled",()=>(s.__v(),s.disabled()||s.opt.disabled?"true":"false"))),v.push(tt(g,"disabled",()=>(s.__v(),s.disabled()||s.opt.disabled))),v.push(E(g,"tabindex",()=>(s.__v(),s.opt.value===s.value()?"0":"-1")));let x=document.createComment("if");g.appendChild(x),v.push(f(x));let k=document.createTextNode("");return g.appendChild(k),v.push(B(k,()=>(s.__v(),s.opt.label))),h.appendChild(g),{nodes:Array.from(h.childNodes),cleanup:()=>{for(let w of v)try{w()}catch(A){}},update:(w,A)=>{s.opt=w,s.__i=A,s.__v.set((j)=>j+1)}}})(u)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("div");m.setAttribute("class","container"),m.setAttribute("part","container"),T(t,"containerEl",m);let p=document.createComment("for");if(m.appendChild(p),o.push(b(p)),c.appendChild(m),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var se=P({mount(e,n){let r=e,o=D(()=>{let t=r.buttonEl();if(!t)return;let a=()=>{if(r.disabled())return;r.pressed.set(!r.pressed()),n.dispatchEvent(new CustomEvent("sui-toggle",{detail:{pressed:r.pressed(),value:r.value()},bubbles:!0,composed:!0}))};return t.addEventListener("click",a),()=>t.removeEventListener("click",a)});return n.focusButton=()=>{n.shadowRoot?.querySelector("button")?.focus()},()=>o()}});var Hh=z("sui-toggle",{props:{pressed:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),value:y.string({default:"",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:y.union(["outline","ghost"],{default:"outline",reflect:!0})},shadow:!0,styles:`
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

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.pressed=n.pressed,t.disabled=n.disabled,t.value=n.value,t.size=n.size,t.variant=n.variant;let a=null;if(se&&typeof se.setup==="function"){if(e.scope=t,a=se.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("button");c.setAttribute("part","button"),c.setAttribute("type","button"),o.push(E(c,"aria-pressed",()=>t.pressed()?"true":"false")),o.push(E(c,"disabled",()=>t.disabled())),T(t,"buttonEl",c);let m=document.createElement("slot");if(c.appendChild(m),b.appendChild(c),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var ue=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","group");let o=document.createElement("input");o.type="hidden",n.appendChild(o);let t=()=>{let s=r.defaultSlotEl();if(!s)return Array.from(n.querySelectorAll(":scope > sui-toggle"));return s.assignedElements().filter((f)=>f.tagName.toLowerCase()==="sui-toggle")},a=()=>t().filter((s)=>!s.hasAttribute("disabled")),b=()=>new Set(r.value().split(",").filter(Boolean)),c=(s)=>{let f=Array.from(s).join(",");r.value.set(f),m(),o.name=r.name(),o.value=f,n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:f},bubbles:!0,composed:!0}))},m=()=>{let s=b(),f=r.size(),h=r.variant();for(let v of t()){let g=v.getAttribute("value")??"";if(v.toggleAttribute("pressed",s.has(g)),f)v.setAttribute("size",f);if(h)v.setAttribute("variant",h)}},p=(s)=>{let f=s;f.stopPropagation();let h=f.composedPath().find((x)=>x.tagName?.toLowerCase()==="sui-toggle");if(!h)return;let v=h.getAttribute("value")??"",g=b();if(r.type()==="single")if(g.has(v))c(new Set);else c(new Set([v]));else{if(g.has(v))g.delete(v);else g.add(v);c(g)}},d=(s)=>{if(s.key!=="ArrowRight"&&s.key!=="ArrowLeft")return;let f=a();if(!f.length)return;let h=s.composedPath().find((x)=>f.includes(x)),v=h?f.indexOf(h):-1,g=s.key==="ArrowRight"?(v+1)%f.length:(v-1+f.length)%f.length;f[g]?.focusButton?.(),s.preventDefault()},l=D(()=>{r.value(),r.size(),r.variant(),m()}),i=D(()=>{let s=r.defaultSlotEl();if(!s)return;let f=()=>m();return s.addEventListener("slotchange",f),()=>s.removeEventListener("slotchange",f)}),u=D(()=>{o.name=r.name(),o.value=r.value()});return n.addEventListener("sui-toggle",p),n.addEventListener("keydown",d),()=>{l(),i(),u(),n.removeEventListener("sui-toggle",p),n.removeEventListener("keydown",d),o.remove()}}});var Xh=z("sui-toggle-group",{props:{type:y.union(["single","multiple"],{default:"single",reflect:!0}),value:y.string({default:"",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:y.union(["outline","ghost"],{default:"outline",reflect:!0}),name:y.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    gap: 0;
  }

  ::slotted(sui-toggle:not(:first-child)) { margin-left: -1px; }
  ::slotted(sui-toggle:first-child) { border-radius: var(--radius, 6px) 0 0 var(--radius, 6px); }
  ::slotted(sui-toggle:last-child) { border-radius: 0 var(--radius, 6px) var(--radius, 6px) 0; }
  ::slotted(sui-toggle:not(:first-child):not(:last-child)) { border-radius: 0; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.type=n.type,t.value=n.value,t.size=n.size,t.variant=n.variant,t.name=n.name;let a=null;if(ue&&typeof ue.setup==="function"){if(e.scope=t,a=ue.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(T(t,"defaultSlotEl",c),b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var Jh=z("sui-code",{props:{lang:y.string({default:"",reflect:!0}),block:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.lang=n.lang,t.block=n.block;let a=(p)=>F({anchor:p,test:()=>{if(t.lang())return 0;return-1},branches:[()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("header");i.setAttribute("part","label");let u=document.createTextNode("");return i.appendChild(u),l.push(B(u,()=>t.lang())),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let s of l)try{s()}catch(f){}}}}]}),b=(p)=>F({anchor:p,test:()=>{if(t.block())return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("div");i.setAttribute("class","block"),i.setAttribute("part","block"),i.setAttribute("role","region"),l.push(E(i,"aria-label",()=>t.lang()?t.lang()+" code block":"code block"));let u=document.createComment("if");i.appendChild(u),l.push(a(u));let s=document.createElement("pre");s.setAttribute("part","pre");let f=document.createElement("slot");return s.appendChild(f),i.appendChild(s),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let h of l)try{h()}catch(v){}}}},()=>{let d=document.createDocumentFragment(),l=[],i=document.createElement("code");i.setAttribute("class","inline"),i.setAttribute("part","inline");let u=document.createElement("slot");return i.appendChild(u),d.appendChild(i),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let s of l)try{s()}catch(f){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createComment("if");return c.appendChild(m),o.push(b(m)),t}});var le=P({mount(e,n){let r=e,o,t=async()=>{try{await navigator.clipboard.writeText(r.value())}catch{let b=document.createElement("textarea");b.value=r.value(),document.body.appendChild(b),b.select();try{document.execCommand("copy")}catch{}b.remove()}if(r.copied.set(!0),n.dispatchEvent(new CustomEvent("sui-copy",{detail:{value:r.value()},bubbles:!0,composed:!0})),o)clearTimeout(o);o=setTimeout(()=>{r.copied.set(!1)},1200)},a=D(()=>{let b=r.buttonEl();if(!b)return;return b.addEventListener("click",t),()=>b.removeEventListener("click",t)});return()=>{if(a(),o)clearTimeout(o)}}});var Kh=z("sui-copy-button",{props:{value:y.string({default:"",reflect:!0}),label:y.string({default:"Copy",reflect:!0}),copied:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.label=n.label,t.copied=n.copied;let a=null;if(le&&typeof le.setup==="function"){if(e.scope=t,a=le.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(u)=>F({anchor:u,test:()=>{if(t.copied())return 0;return 1},branches:[()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("sui-icon");return h.setAttribute("name","check"),h.setAttribute("size","sm"),h.setAttribute("aria-hidden","true"),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let v of f)try{v()}catch(g){}}}},()=>{let s=document.createDocumentFragment(),f=[],h=document.createElement("sui-icon");return h.setAttribute("name","copy"),h.setAttribute("size","sm"),h.setAttribute("aria-hidden","true"),s.appendChild(h),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let v of f)try{v()}catch(g){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("button");m.setAttribute("part","button"),m.setAttribute("type","button"),o.push(E(m,"aria-label",()=>t.label())),T(t,"buttonEl",m);let p=document.createComment("if");m.appendChild(p),o.push(b(p));let d=document.createElement("slot");o.push(nt(d,r,"data-empty")),m.appendChild(d);let l=document.createElement("span");l.setAttribute("class","default-label");let i=document.createTextNode("");if(l.appendChild(i),o.push(B(i,()=>t.copied()?"Copied":t.label())),m.appendChild(l),c.appendChild(m),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(f,h){return f[h]},set(f,h,v){let g=f[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(u.mount){let f=u.mount(s,r);if(typeof f==="function")o.push(f)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});function sr(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function ao(e,n,r,o,t){let a=n!==null?`<span class="key">"${sr(n)}"</span>: `:"";if(e===null)return`${a}<span class="null">null</span>`;if(typeof e==="string")return`${a}<span class="s">"${sr(e)}"</span>`;if(typeof e==="number")return`${a}<span class="n">${e}</span>`;if(typeof e==="boolean")return`${a}<span class="b">${String(e)}</span>`;let b=Array.isArray(e),c=b?e.map((u,s)=>[String(s),u]):Object.entries(e),m=o||r<t,p=b?"[":"{",d=b?"]":"}",l=c.length;if(l===0)return`${a}${p}${d}`;let i=c.map(([u,s])=>`<div class="node">${ao(s,b?null:u,r+1,o,t)}</div>`).join("");return`<details${m?" open":""}>
<summary><span class="caret">▶</span>${a}${p}<span class="meta">${l} ${b?"items":"keys"}</span></summary>
<div class="branch">${i}</div>
${d}</details>`}var de=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","tree");if(!n.hasAttribute("aria-label"))n.setAttribute("aria-label","JSON tree");let o=r.contentEl();if(!o)return()=>{};return et(o,()=>{let t=r.data(),a=r.expanded(),b=r.depth();if(!t)return"";let c;try{c=JSON.parse(t)}catch(m){return`<span class="error" part="error" role="alert">Invalid JSON: ${sr(m.message)}</span>`}return`<div part="root">${ao(c,null,0,a,b)}</div>`})}});var ev=z("sui-json-view",{props:{data:y.string({default:"",reflect:!0}),expanded:y.bool({default:!1,reflect:!0}),depth:y.number({default:1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.expanded=n.expanded,t.depth=n.depth;let a=null;if(de&&typeof de.setup==="function"){if(e.scope=t,a=de.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");if(c.setAttribute("class","json-root"),T(t,"contentEl",c),b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});function io(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function Sa(e,n){let r=e.split(`
`),o=n.split(`
`),t=r.length,a=o.length,b=Array.from({length:t+1},()=>Array(a+1).fill(0));for(let d=t-1;d>=0;d--)for(let l=a-1;l>=0;l--)b[d][l]=r[d]===o[l]?b[d+1][l+1]+1:Math.max(b[d+1][l],b[d][l+1]);let c=[],m=0,p=0;while(m<t&&p<a)if(r[m]===o[p])c.push({kind:"ctx",text:r[m]}),m++,p++;else if(b[m+1][p]>=b[m][p+1])c.push({kind:"del",text:r[m]}),m++;else c.push({kind:"add",text:o[p]}),p++;while(m<t)c.push({kind:"del",text:r[m++]});while(p<a)c.push({kind:"add",text:o[p++]});return c}function Fa(e){return e.split(`
`).map((n)=>{if(n.startsWith("@@"))return{kind:"hunk",text:n};if(n.startsWith("+"))return{kind:"add",text:n.slice(1)};if(n.startsWith("-"))return{kind:"del",text:n.slice(1)};return{kind:"ctx",text:n.startsWith(" ")?n.slice(1):n}})}function Ba(e){return e.map((n)=>{let r=n.kind==="add"?"+":n.kind==="del"?"-":n.kind==="hunk"?"@":" ",o=n.kind==="add"?"added":n.kind==="del"?"removed":n.kind==="hunk"?"hunk":"context";return`<div class="line ${n.kind}" part="line" role="listitem" aria-label="${o}: ${io(n.text)}"><span class="sign" aria-hidden="true">${r}</span><span>${io(n.text)}</span></div>`}).join("")}var ce=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","region");if(!n.hasAttribute("aria-label"))n.setAttribute("aria-label","diff");let o=r.contentEl();if(!o)return()=>{};return et(o,()=>{let t=r.value(),a=r.from(),b=r.to(),c;if(t)c=Fa(t);else if(a||b)c=Sa(a,b);else c=[];return Ba(c)})}});var uv=z("sui-diff",{props:{from:y.string({default:"",reflect:!0}),to:y.string({default:"",reflect:!0}),value:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.from=n.from,t.to=n.to,t.value=n.value;let a=null;if(ce&&typeof ce.setup==="function"){if(e.scope=t,a=ce.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");if(c.setAttribute("class","diff-root"),T(t,"contentEl",c),b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});function ur(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function Gt(e){let n=ur(e);return n=n.replace(/`([^`]+)`/g,(r,o)=>`<code>${o}</code>`),n=n.replace(/\*\*([^*]+)\*\*/g,"<strong>$1</strong>"),n=n.replace(/(^|[^*])\*([^*\n]+)\*/g,"$1<em>$2</em>"),n=n.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g,(r,o,t)=>`<a href="${t}" rel="noopener noreferrer">${o}</a>`),n}function Ga(e){let n=e.replace(/\r\n/g,`
`).split(`
`),r=[],o=0;while(o<n.length){let t=n[o];if(/^```/.test(t)){let c=t.slice(3).trim(),m=[];o++;while(o<n.length&&!/^```/.test(n[o]))m.push(n[o++]);o++,r.push(`<pre data-lang="${ur(c)}"><code>${ur(m.join(`
`))}</code></pre>`);continue}let a=/^(#{1,6})\s+(.*)$/.exec(t);if(a){r.push(`<h${a[1].length}>${Gt(a[2])}</h${a[1].length}>`),o++;continue}if(/^\s*[-*]\s+/.test(t)){let c=[];while(o<n.length&&/^\s*[-*]\s+/.test(n[o]))c.push(`<li>${Gt(n[o].replace(/^\s*[-*]\s+/,""))}</li>`),o++;r.push(`<ul>${c.join("")}</ul>`);continue}if(/^\s*\d+\.\s+/.test(t)){let c=[];while(o<n.length&&/^\s*\d+\.\s+/.test(n[o]))c.push(`<li>${Gt(n[o].replace(/^\s*\d+\.\s+/,""))}</li>`),o++;r.push(`<ol>${c.join("")}</ol>`);continue}if(/^>\s?/.test(t)){let c=[];while(o<n.length&&/^>\s?/.test(n[o]))c.push(n[o].replace(/^>\s?/,"")),o++;r.push(`<blockquote>${Gt(c.join(" "))}</blockquote>`);continue}if(t.trim()===""){o++;continue}let b=[];while(o<n.length&&n[o].trim()!==""&&!/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(n[o]))b.push(n[o]),o++;r.push(`<p>${Gt(b.join(" "))}</p>`)}return r.join(`
`)}var pe=P({mount(e){let n=e,r=n.contentEl();if(!r)return()=>{};return et(r,()=>Ga(n.value()??""))}});var mv=z("sui-markdown",{props:{value:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value;let a=null;if(pe&&typeof pe.setup==="function"){if(e.scope=t,a=pe.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");if(c.setAttribute("class","md-root"),T(t,"contentEl",c),b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var so={actions:"#view-topbar-actions","status-start":"#view-status-start","status-end":"#view-status-end"},fe={title:"#breadcrumbs-title",branch:"#status-branch"},me=P({mount(e,n){let r=e,o={};function t(){for(let m of Object.keys(so)){let p=document.querySelector(so[m]);if(!p)continue;let d=Array.from(n.querySelectorAll(`[slot="${m}"]`));if(d.length===0)continue;for(let l of d)p.appendChild(l);o[m]=d}}function a(){let m=document.querySelector(fe.title);if(m)m.textContent="";if(r.branch()){let p=document.querySelector(fe.branch);if(p)p.textContent=""}if(r.ahead()>0||r.behind()>0){let p=document.querySelector("#status-ab");if(p)p.textContent=""}if(r.tone()!=="default"||r.busy()){let p=document.querySelector("#status-dot");if(p)delete p.dataset.tone,p.removeAttribute("data-busy")}}function b(){for(let m of Object.keys(o)){for(let p of o[m]??[])p.remove();delete o[m]}a()}let c=D(()=>{let m=r.title(),p=r.branch(),d=r.ahead(),l=r.behind(),i=r.tone(),u=r.busy();if(m){let s=document.querySelector(fe.title);if(s)s.textContent=m}if(p){let s=document.querySelector(fe.branch);if(s)s.textContent=p}if(d>0||l>0){let s=document.querySelector("#status-ab");if(s){let f=[];if(d>0)f.push(`↑${d}`);if(l>0)f.push(`↓${l}`);s.textContent=f.join(" ")}}if(i!=="default"||u){let s=document.querySelector("#status-dot");if(s)if(s.dataset.tone=i,u)s.setAttribute("data-busy","");else s.removeAttribute("data-busy")}});return queueMicrotask(()=>t()),()=>{c(),b()}}});var xv=z("sui-view-chrome",{props:{title:y.string({default:"",reflect:!0}),branch:y.string({default:"",reflect:!0}),ahead:y.number({default:0,reflect:!0}),behind:y.number({default:0,reflect:!0}),tone:y.union(["default","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),busy:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.title=n.title,t.branch=n.branch,t.ahead=n.ahead,t.behind=n.behind,t.tone=n.tone,t.busy=n.busy;let a=null;if(me&&typeof me.setup==="function"){if(e.scope=t,a=me.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createTextNode(`

`);b.appendChild(c);let m=document.createTextNode(`
`);b.appendChild(m);let p=document.createTextNode(`
`);b.appendChild(p);let d=document.createTextNode(`
`);b.appendChild(d);let l=document.createTextNode(`
`);b.appendChild(l);let i=document.createTextNode(`
`);b.appendChild(i);let u=document.createTextNode(`

`);b.appendChild(u);let s=document.createTextNode(`
`);if(b.appendChild(s),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let h=new Proxy(t,{get(v,g){return v[g]},set(v,g,x){let k=v[g];if(k!=null&&typeof k.set==="function"){if(typeof x==="function")k.set(()=>x);else k.set(x);return!0}return!1}});if(f.mount){let v=f.mount(h,r);if(typeof v==="function")o.push(v)}if(f.unmount)o.push(()=>f.unmount(h))}}return t}});var be=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","navigation");let o=()=>Array.from(n.querySelectorAll(":scope > sui-navigation-item")),t=()=>o().filter((d)=>!d.hasAttribute("disabled")),a=()=>{let d=r.value();for(let l of o())l.toggleAttribute("active",l.getAttribute("value")===d)},b=(d)=>{let l=d;l.stopPropagation();let i=l.detail.value;r.value.set(i),a(),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:i},bubbles:!0,composed:!0}))},c=(d)=>{if(d.key!=="ArrowRight"&&d.key!=="ArrowLeft"&&d.key!=="Home"&&d.key!=="End")return;let l=t();if(!l.length)return;let i=d.composedPath().find((f)=>l.includes(f)),u=i?l.indexOf(i):-1,s;if(d.key==="ArrowRight")s=l[(u+1)%l.length];else if(d.key==="ArrowLeft")s=l[(u-1+l.length)%l.length];else if(d.key==="Home")s=l[0];else if(d.key==="End")s=l[l.length-1];if(s)d.preventDefault(),s.focusLink?.()},m=D(()=>{r.value(),a()}),p=D(()=>{let d=r.defaultSlotEl();if(!d)return;let l=()=>a();return d.addEventListener("slotchange",l),()=>d.removeEventListener("slotchange",l)});return n.addEventListener("sui-nav-select",b),n.addEventListener("keydown",c),()=>{m(),p(),n.removeEventListener("sui-nav-select",b),n.removeEventListener("keydown",c)}}});var jv=z("sui-navigation-menu",{props:{value:y.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    align-items: stretch;
    gap: 0;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-body);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value;let a=null;if(be&&typeof be.setup==="function"){if(e.scope=t,a=be.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");if(T(t,"defaultSlotEl",c),b.appendChild(c),a!=null){let m=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(d,l){return d[l]},set(d,l,i){let u=d[l];if(u!=null&&typeof u.set==="function"){if(typeof i==="function")u.set(()=>i);else u.set(i);return!0}return!1}});if(m.mount){let d=m.mount(p,r);if(typeof d==="function")o.push(d)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var he=P({mount(e,n){let r=e,o=(t)=>{if(r.disabled())return;if(!r.href())t.preventDefault();n.dispatchEvent(new CustomEvent("sui-nav-select",{detail:{value:r.value()},bubbles:!0,composed:!0}))};return n.focusLink=()=>{n.shadowRoot?.querySelector(".link")?.focus()},n.addEventListener("click",o),()=>n.removeEventListener("click",o)}});var Pv=z("sui-navigation-item",{props:{value:y.string({default:"",reflect:!0}),active:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),href:y.string({default:""})},shadow:!0,styles:`
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

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.active=n.active,t.disabled=n.disabled,t.href=n.href;let a=null;if(he&&typeof he.setup==="function"){if(e.scope=t,a=he.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>F({anchor:d,test:()=>{if(t.href())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("a");u.setAttribute("class","link"),u.setAttribute("part","link"),i.push(E(u,"href",()=>t.href())),i.push(E(u,"aria-current",()=>t.active()?"page":"false"));let s=document.createElement("slot");return u.appendChild(s),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let f of i)try{f()}catch(h){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("button");u.setAttribute("class","link"),u.setAttribute("part","link"),u.setAttribute("type","button"),i.push(E(u,"aria-current",()=>t.active()?"true":"false")),i.push(E(u,"disabled",()=>t.disabled()));let s=document.createElement("slot");return u.appendChild(s),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let f of i)try{f()}catch(h){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createComment("if");c.appendChild(m),o.push(b(m));let p=document.createElement("div");if(p.setAttribute("class","indicator"),p.setAttribute("part","indicator"),p.setAttribute("aria-hidden","true"),c.appendChild(p),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var ve=P({mount(e,n){let r=e,o=null,t=()=>{let c=r.viewportEl();if(!c)return;let m=r.orientation(),p=m==="vertical"||m==="both",d=m==="horizontal"||m==="both",l=r.vThumb(),i=r.hThumb();if(p&&l){let u=c.clientHeight/c.scrollHeight;if(u<1){let s=Math.max(24,u*c.clientHeight),h=c.scrollTop/(c.scrollHeight-c.clientHeight)*(c.clientHeight-s);l.style.height=`${s}px`,l.style.top=`${h}px`,l.style.display=""}else l.style.display="none"}if(d&&i){let u=c.clientWidth/c.scrollWidth;if(u<1){let s=Math.max(24,u*c.clientWidth),h=c.scrollLeft/(c.scrollWidth-c.clientWidth)*(c.clientWidth-s);i.style.width=`${s}px`,i.style.left=`${h}px`,i.style.display=""}else i.style.display="none"}},a=()=>{t();let c=r.vScrollbar(),m=r.hScrollbar();if(c?.classList.add("active"),m?.classList.add("active"),o)clearTimeout(o);o=setTimeout(()=>{c?.classList.remove("active"),m?.classList.remove("active")},1200)},b=r.viewportEl();if(b)b.addEventListener("scroll",a,{passive:!0});return t(),()=>{if(o)clearTimeout(o);if(b)b.removeEventListener("scroll",a)}}});var Bv=z("sui-scroll-area",{props:{orientation:y.union(["vertical","horizontal","both"],{default:"vertical",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.orientation=n.orientation,t.size=n.size;let a=null;if(ve&&typeof ve.setup==="function"){if(e.scope=t,a=ve.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","viewport"),c.setAttribute("part","viewport"),c.setAttribute("tabindex","0"),T(t,"viewportEl",c);let m=document.createElement("slot");c.appendChild(m),b.appendChild(c);let p=document.createElement("div");p.setAttribute("class","scrollbar scrollbar-v"),p.setAttribute("part","scrollbar-v"),p.setAttribute("aria-hidden","true"),T(t,"vScrollbar",p);let d=document.createElement("div");d.setAttribute("class","thumb"),T(t,"vThumb",d),p.appendChild(d),b.appendChild(p);let l=document.createElement("div");l.setAttribute("class","scrollbar scrollbar-h"),l.setAttribute("part","scrollbar-h"),l.setAttribute("aria-hidden","true"),T(t,"hScrollbar",l);let i=document.createElement("div");if(i.setAttribute("class","thumb"),T(t,"hThumb",i),l.appendChild(i),b.appendChild(l),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let s=new Proxy(t,{get(f,h){return f[h]},set(f,h,v){let g=f[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(u.mount){let f=u.mount(s,r);if(typeof f==="function")o.push(f)}if(u.unmount)o.push(()=>u.unmount(s))}}return t}});var ge=P({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","toolbar");if(!n.hasAttribute("aria-orientation"))n.setAttribute("aria-orientation","horizontal");let o=()=>{return(r.defaultSlot()?.assignedElements({flatten:!0})??[]).filter((c)=>!c.hasAttribute("disabled"))},t=(a)=>{if(a.key!=="ArrowRight"&&a.key!=="ArrowLeft")return;let b=o();if(!b.length)return;let c=a.composedPath().find((d)=>b.includes(d))||null,m=c?b.indexOf(c):-1,p=a.key==="ArrowRight"?(m+1)%b.length:(m-1+b.length)%b.length;b[p]?.focus(),a.preventDefault()};return n.addEventListener("keydown",t),()=>n.removeEventListener("keydown",t)}});var Jv=z("sui-toolbar",{props:{justify:y.union(["start","center","end","between"],{default:"start",reflect:!0}),align:y.union(["center","start","end","baseline"],{default:"center",reflect:!0}),gap:y.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:y.union(["default","bordered"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.justify=n.justify,t.align=n.align,t.gap=n.gap,t.variant=n.variant;let a=null;if(ge&&typeof ge.setup==="function"){if(e.scope=t,a=ge.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("slot");c.setAttribute("name","spacer"),b.appendChild(c);let m=document.createElement("slot");if(T(t,"defaultSlot",m),b.appendChild(m),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});var uo={dashboard:"layout-dashboard",pipelines:"workflow",agents:"bot",skills:"sparkles",git:"git-branch",collections:"database",canvas:"frame",settings:"settings",chat:"messages-square",events:"radio",specs:"file-text",theme:"palette",setup:"rocket",overview:"compass",mesh:"network"},lo={"theme-studio":"theme"},Ua=new Set([...Object.keys(uo),...Object.keys(lo)]),ye=P({mount(e,n){let r=e,o=D(()=>{let t=r.name(),a=r.label(),b=lo[t]??t,m=Ua.has(t)?uo[b]:"dot",p=r.glyphEl();if(p)p.setAttribute("name",m);if(a)n.setAttribute("role","img"),n.setAttribute("aria-label",a),n.removeAttribute("aria-hidden");else n.setAttribute("aria-hidden","true"),n.removeAttribute("role"),n.removeAttribute("aria-label")});return()=>o()}});var Kv=z("sui-app-icon",{props:{name:y.string({default:"",reflect:!0}),variant:y.union(["glyph","tile","solid"],{default:"glyph",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),radius:y.string({default:"",reflect:!0}),label:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.name=n.name,t.variant=n.variant,t.size=n.size,t.radius=n.radius,t.label=n.label;let a=null;if(ye&&typeof ye.setup==="function"){if(e.scope=t,a=ye.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("span");c.setAttribute("class","frame"),c.setAttribute("part","frame");let m=document.createElement("slot"),p=document.createElement("sui-icon");if(T(t,"glyphEl",p),p.setAttribute("part","glyph"),p.setAttribute("aria-hidden","true"),m.appendChild(p),c.appendChild(m),b.appendChild(c),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var xe=P({scope:()=>{return{nodes:N([])}},mount(e,n){let o=e.nodes,t=n.nodes;if(t!=null)o.set(t);if(Object.defineProperty(n,"nodes",{configurable:!0,get:()=>o(),set:(a)=>o.set(a)}),!n.hasAttribute("role"))n.setAttribute("role","tree")}});var eg=z("sui-tree-view",{shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
  }
  ul { list-style: none; margin: 0; padding: 0; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(xe&&typeof xe.setup==="function"){if(e.scope=t,a=xe.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(d)=>R({anchor:d,items:()=>t.nodes(),key:(l,i)=>{let u=Object.create(t);return u.n=l,u.__i=i,((s)=>l.id)(u)},render:(l,i)=>{let u=Object.create(t);return u.n=l,u.__i=i,u.__v=N(0),((s)=>{let f=document.createDocumentFragment(),h=[],v=document.createElement("sui-tree-item");return h.push(tt(v,"node",()=>(s.__v(),s.n))),f.appendChild(v),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let g of h)try{g()}catch(x){}},update:(g,x)=>{s.n=g,s.__i=x,s.__v.set((k)=>k+1)}}})(u)}}),c=(d)=>F({anchor:d,test:()=>{if(t.nodes().length>0)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("ul");u.setAttribute("role","tree");let s=document.createComment("for");return u.appendChild(s),i.push(b(s)),l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let f of i)try{f()}catch(h){}}}},()=>{let l=document.createDocumentFragment(),i=[],u=document.createElement("slot");return l.appendChild(u),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of i)try{s()}catch(f){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createComment("if");if(m.appendChild(p),o.push(c(p)),a!=null){let d=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(d){let l=new Proxy(t,{get(i,u){return i[u]},set(i,u,s){let f=i[u];if(f!=null&&typeof f.set==="function"){if(typeof s==="function")f.set(()=>s);else f.set(s);return!0}return!1}});if(d.mount){let i=d.mount(l,r);if(typeof i==="function")o.push(i)}if(d.unmount)o.push(()=>d.unmount(l))}}return t}});var we=P({scope:({label:e})=>{let n=N(null),r=Z(()=>{let a=n();return a?a.label:e()}),o=Z(()=>{return n()?.children??[]}),t=J(null);return{node:n,nodeLabel:r,nodeChildren:o,rowEl:t}},mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","treeitem");let o=r.node,t=n.node;if(t!=null)o.set(t);Object.defineProperty(n,"node",{configurable:!0,get:()=>o(),set:(i)=>o.set(i)});let a=D(()=>{let i=o();if(i!==null)if(!i.children||i.children.length===0)n.setAttribute("leaf","");else n.removeAttribute("leaf")}),b=D(()=>{let i=r.rowEl();if(!i)return;if(!r.leaf())i.setAttribute("aria-expanded",String(r.expanded()));else i.removeAttribute("aria-expanded");i.setAttribute("aria-selected",String(r.selected()))}),c=(i)=>{if(r.leaf())return;i.stopPropagation();let u=!r.expanded();r.expanded.set(u),n.dispatchEvent(new CustomEvent("sui-toggle",{detail:{expanded:u},bubbles:!0,composed:!0}))},m=()=>{n.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},p=(i)=>{m(),c(i)},d=(i)=>{if(i.key==="Enter"||i.key===" ")i.preventDefault(),m();else if(i.key==="ArrowRight"&&!r.leaf()&&!r.expanded())i.preventDefault(),c(i);else if(i.key==="ArrowLeft"&&r.expanded())i.preventDefault(),c(i)},l=D(()=>{let i=r.rowEl();if(!i)return;return i.addEventListener("click",p),i.addEventListener("keydown",d),()=>{i.removeEventListener("click",p),i.removeEventListener("keydown",d)}});return()=>{a(),b(),l()}}});var ug=z("sui-tree-item",{props:{label:y.string({default:"",reflect:!0}),expanded:y.bool({default:!1,reflect:!0}),selected:y.bool({default:!1,reflect:!0}),leaf:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.label=n.label,t.expanded=n.expanded,t.selected=n.selected,t.leaf=n.leaf;let a=null;if(we&&typeof we.setup==="function"){if(e.scope=t,a=we.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(k)=>R({anchor:k,items:()=>t.nodeChildren(),key:(w,A)=>{let j=Object.create(t);return j.c=w,j.__i=A,((C)=>w.id)(j)},render:(w,A)=>{let j=Object.create(t);return j.c=w,j.__i=A,j.__v=N(0),((C)=>{let M=document.createDocumentFragment(),O=[],H=document.createElement("sui-tree-item");return O.push(tt(H,"node",()=>(C.__v(),C.c))),M.appendChild(H),{nodes:Array.from(M.childNodes),cleanup:()=>{for(let S of O)try{S()}catch(U){}},update:(S,U)=>{C.c=S,C.__i=U,C.__v.set((G)=>G+1)}}})(j)}}),c=(k)=>F({anchor:k,test:()=>{if(t.nodeChildren().length>0)return 0;return 1},branches:[()=>{let w=document.createDocumentFragment(),A=[],j=document.createComment("for");return w.appendChild(j),A.push(b(j)),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of A)try{C()}catch(M){}}}},()=>{let w=document.createDocumentFragment(),A=[],j=document.createElement("slot");return w.appendChild(j),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of A)try{C()}catch(M){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createElement("div");p.setAttribute("class","row"),p.setAttribute("part","row"),p.setAttribute("tabindex","0"),T(t,"rowEl",p);let d=document.createElement("span");d.setAttribute("class","caret"),d.setAttribute("part","caret"),d.setAttribute("aria-hidden","true");let l=document.createElement("svg");l.setAttribute("viewbox","0 0 12 12"),l.setAttribute("fill","none"),l.setAttribute("stroke","currentColor"),l.setAttribute("stroke-width","1.5");let i=document.createElement("polyline");i.setAttribute("points","4 3 8 6 4 9"),l.appendChild(i),d.appendChild(l),p.appendChild(d);let u=document.createElement("slot");u.setAttribute("name","icon"),p.appendChild(u);let s=document.createElement("span");s.setAttribute("class","label"),s.setAttribute("part","label");let f=document.createTextNode("");s.appendChild(f),o.push(B(f,()=>t.nodeLabel())),p.appendChild(s);let h=document.createElement("span");h.setAttribute("class","trailing");let v=document.createElement("slot");v.setAttribute("name","trailing"),h.appendChild(v),p.appendChild(h),m.appendChild(p);let g=document.createElement("div");g.setAttribute("class","children"),g.setAttribute("part","children"),g.setAttribute("role","group");let x=document.createComment("if");if(g.appendChild(x),o.push(c(x)),m.appendChild(g),a!=null){let k=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(k){let w=new Proxy(t,{get(A,j){return A[j]},set(A,j,C){let M=A[j];if(M!=null&&typeof M.set==="function"){if(typeof C==="function")M.set(()=>C);else M.set(C);return!0}return!1}});if(k.mount){let A=k.mount(w,r);if(typeof A==="function")o.push(A)}if(k.unmount)o.push(()=>k.unmount(w))}}return t}});var co=new WeakSet;function Xa(e){if(co.has(e))return;co.add(e);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let r=new CSSStyleSheet;r.replaceSync(po),e.adoptedStyleSheets=[...e.adoptedStyleSheets,r];return}}catch{}let n=e.createElement("style");n.setAttribute("data-sui-table",""),n.textContent=po,e.head.appendChild(n)}var Ae=P({scope:()=>{let e=N([]),n=N([]),r=N(null),o=N("asc"),t=N([]),a=Z(()=>{let b=r(),c=o(),m=e();if(!b)return m;return[...m].sort((p,d)=>{let l=String(p[b]??""),i=String(d[b]??""),u=l<i?-1:l>i?1:0;return c==="asc"?u:-u})});return{rows:e,columns:n,sortKey:r,sortDir:o,selected:t,sortedRows:a}},on:{sortBy(e,n){if(e.sortKey===n)e.sortDir=e.sortDir==="asc"?"desc":"asc";else e.sortKey=n,e.sortDir="asc"},toggleSelect(e,n){let r=e.selected;e.selected=r.includes(n)?r.filter((o)=>o!==n):[...r,n]},selectAll(e){e.selected=e.rows.map((n)=>n.id)},clearSelection(e){e.selected=[]}},mount(e,n){let r=e,o=r.rows,t=r.columns,a=n.rows;if(a!=null)o.set(a);let b=n.columns;if(b!=null)t.set(b);Object.defineProperty(n,"rows",{configurable:!0,get:()=>o(),set:(f)=>o.set(f)}),Object.defineProperty(n,"columns",{configurable:!0,get:()=>t(),set:(f)=>t.set(f)}),Xa(n.ownerDocument);let c=()=>{n.querySelectorAll("th[data-sort-key]").forEach((f)=>{let h=f.getAttribute("aria-sort")||"none",v=f.querySelector(".sui-sort-indicator");if(!v)return;if(h==="ascending")v.style.transform="rotate(180deg)",v.style.opacity="1";else if(h==="descending")v.style.transform="rotate(0deg)",v.style.opacity="1";else v.style.transform="rotate(0deg)",v.style.opacity="0.4"})},m=()=>{n.querySelectorAll("th[data-sort-key]").forEach((f)=>{if(!f.hasAttribute("aria-sort"))f.setAttribute("aria-sort","none");if(!f.hasAttribute("tabindex"))f.setAttribute("tabindex","0");if(!f.hasAttribute("role"))f.setAttribute("role","columnheader");if(!f.querySelector(".sui-sort-indicator")){let h=document.createElement("span");h.className="sui-sort-indicator",h.setAttribute("aria-hidden","true"),h.style.cssText="margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;";let v="http://www.w3.org/2000/svg",g=document.createElementNS(v,"svg");g.setAttribute("width","10"),g.setAttribute("height","10"),g.setAttribute("viewBox","0 0 10 10"),g.setAttribute("fill","none"),g.setAttribute("stroke","currentColor"),g.setAttribute("stroke-width","1.5"),g.setAttribute("stroke-linecap","round"),g.setAttribute("stroke-linejoin","round"),g.setAttribute("aria-hidden","true");let x=document.createElementNS(v,"path");x.setAttribute("d","M2 4l3-2 3 2M2 6l3 2 3-2"),g.appendChild(x),h.appendChild(g),f.appendChild(h)}}),c()},p=()=>{let f=n.querySelector(":scope > table tbody > tr")||n.querySelector(":scope > table > tr:not(:has(th))")||n.querySelector(":scope > tbody > tr")||n.querySelector(':scope > tr:not([slot="head"])'),h=n.querySelector('[slot="empty"]')!=null;if(!f&&h)n.setAttribute("data-empty","");else n.removeAttribute("data-empty")},d=(f)=>{let h=f.getAttribute("data-sort-key")||"",v=f.getAttribute("aria-sort")||"none",g=v==="none"?"ascending":v==="ascending"?"descending":"none";n.querySelectorAll("th[data-sort-key]").forEach((x)=>{if(x!==f)x.setAttribute("aria-sort","none")}),f.setAttribute("aria-sort",g),c(),n.emit("sui-sort",{key:h,direction:g})},l=(f)=>{let h=f.target.closest("th[data-sort-key]");if(!h||!n.contains(h))return;d(h)},i=(f)=>{if(f.key!=="Enter"&&f.key!==" ")return;let h=f.target.closest("th[data-sort-key]");if(!h||!n.contains(h))return;f.preventDefault(),d(h)},u=(f)=>{let h=f.target.closest("td");if(!h||!n.contains(h))return;if(h.classList.contains("sui-table-gutter-cell"))return;let v=h.closest("tr");if(!v)return;let g=v.getAttribute("data-row-id");if(!g)return;n.emit("sui-row-click",{id:g,index:v.rowIndex})};Q(n,"click",l),Q(n,"click",u),Q(n,"keydown",i),m(),p();let s=new MutationObserver(()=>{m(),p()});return s.observe(n,{childList:!0,subtree:!0}),()=>{s.disconnect()}}}),po=`
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
`;var mg=z("sui-table",{props:{variant:y.union(["slot","native","data"],{default:"slot",reflect:!0}),density:y.union(["compact","default","comfortable"],{default:"default",reflect:!0}),dense:y.bool({default:!1,reflect:!0}),hoverable:y.bool({default:!0,reflect:!0}),striped:y.bool({default:!1,reflect:!0}),"sticky-header":y.bool({default:!1,reflect:!0}),"sticky-first-col":y.bool({default:!1,reflect:!0}),gutter:y.bool({default:!1,reflect:!0}),label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.variant=n.variant,t.density=n.density,t.dense=n.dense,t.hoverable=n.hoverable,t.striped=n.striped,t.stickyHeader=n.stickyHeader,t.stickyFirstCol=n.stickyFirstCol,t.gutter=n.gutter,t.label=n.label;let a=null;if(Ae&&typeof Ae.setup==="function"){if(e.scope=t,a=Ae.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=(l)=>R({anchor:l,items:()=>t.columns(),key:(i,u)=>{let s=Object.create(t);return s.c=i,s.__i=u,((f)=>i.field)(s)},render:(i,u)=>{let s=Object.create(t);return s.c=i,s.__i=u,s.__v=N(0),((f)=>{let h=document.createDocumentFragment(),v=[],g=document.createElement("th");v.push(K(g,"click",function(){let k=()=>f.sortBy(f.c.field);return typeof k==="function"?k.bind(void 0):()=>{}}(),{modifiers:[]})),v.push(pt(g,"sorted",()=>(f.__v(),f.sortKey()===f.c.field)));let x=document.createTextNode("");return g.appendChild(x),v.push(B(x,()=>(f.__v(),f.c.label))),h.appendChild(g),{nodes:Array.from(h.childNodes),cleanup:()=>{for(let k of v)try{k()}catch(w){}},update:(k,w)=>{f.c=k,f.__i=w,f.__v.set((A)=>A+1)}}})(s)}}),c=(l)=>R({anchor:l,items:()=>t.sortedRows(),key:(i,u)=>{let s=Object.create(t);return s.r=i,s.__i=u,((f)=>i.id)(s)},render:(i,u)=>{let s=Object.create(t);return s.r=i,s.__i=u,s.__v=N(0),((f)=>{let h=(w)=>R({anchor:w,items:()=>f.columns(),key:(A,j)=>{let C=Object.create(f);return C.c=A,C.__i=j,((M)=>A.field)(C)},render:(A,j)=>{let C=Object.create(f);return C.c=A,C.__i=j,C.__v=N(0),((M)=>{let O=document.createDocumentFragment(),H=[],S=document.createElement("td"),U=document.createTextNode("");return S.appendChild(U),H.push(B(U,()=>(M.__v(),M.r[M.c.field]))),O.appendChild(S),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let G of H)try{G()}catch(X){}},update:(G,X)=>{M.c=G,M.__i=X,M.__v.set((W)=>W+1)}}})(C)}}),v=document.createDocumentFragment(),g=[],x=document.createElement("tr");g.push(pt(x,"selected",()=>(f.__v(),f.selected().includes(f.r.id)))),g.push(K(x,"click",function(){let w=()=>f.toggleSelect(f.r.id);return typeof w==="function"?w.bind(void 0):()=>{}}(),{modifiers:[]}));let k=document.createComment("for");return x.appendChild(k),g.push(h(k)),v.appendChild(x),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let w of g)try{w()}catch(A){}},update:(w,A)=>{f.r=w,f.__i=A,f.__v.set((j)=>j+1)}}})(s)}}),m=(l)=>F({anchor:l,test:()=>{if(t.variant()==="data")return 0;if(t.variant()!=="native")return 1;return 2},branches:[()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("div");s.setAttribute("class","wrap"),s.setAttribute("part","wrap");let f=document.createElement("table");f.setAttribute("part","table"),u.push(E(f,"aria-label",()=>t.label()||null));let h=document.createElement("thead");h.setAttribute("part","thead");let v=document.createElement("tr"),g=document.createComment("for");v.appendChild(g),u.push(b(g)),h.appendChild(v),f.appendChild(h);let x=document.createElement("tbody");x.setAttribute("part","tbody");let k=document.createComment("for");return x.appendChild(k),u.push(c(k)),f.appendChild(x),s.appendChild(f),i.appendChild(s),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let w of u)try{w()}catch(A){}}}},()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("div");s.setAttribute("class","wrap"),s.setAttribute("part","wrap");let f=document.createElement("table");f.setAttribute("part","table"),u.push(E(f,"aria-label",()=>t.label()||null));let h=document.createElement("thead");h.setAttribute("part","thead");let v=document.createElement("slot");v.setAttribute("name","head"),h.appendChild(v),f.appendChild(h);let g=document.createElement("tbody");g.setAttribute("part","tbody");let x=document.createElement("slot");g.appendChild(x),f.appendChild(g),s.appendChild(f);let k=document.createElement("div");k.setAttribute("class","empty"),k.setAttribute("part","empty"),k.setAttribute("role","status");let w=document.createElement("slot");return w.setAttribute("name","empty"),k.appendChild(w),s.appendChild(k),i.appendChild(s),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let A of u)try{A()}catch(j){}}}},()=>{let i=document.createDocumentFragment(),u=[],s=document.createElement("div");s.setAttribute("class","wrap"),s.setAttribute("part","wrap");let f=document.createElement("slot");s.appendChild(f);let h=document.createElement("div");h.setAttribute("class","empty"),h.setAttribute("part","empty"),h.setAttribute("role","status");let v=document.createElement("slot");return v.setAttribute("name","empty"),h.appendChild(v),s.appendChild(h),i.appendChild(s),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let g of u)try{g()}catch(x){}}}}]}),p=r.shadowRoot??r;L(p);let d=document.createComment("if");if(p.appendChild(d),o.push(m(d)),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(u,s){return u[s]},set(u,s,f){let h=u[s];if(h!=null&&typeof h.set==="function"){if(typeof f==="function")h.set(()=>f);else h.set(f);return!0}return!1}});if(l.mount){let u=l.mount(i,r);if(typeof u==="function")o.push(u)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});function rt(e){return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#39;")}function fo(e,n){if(Array.isArray(e))return e;if(typeof e==="string"&&e.trim())try{let r=JSON.parse(e);return Array.isArray(r)?r:n}catch{return n}return n}function qa(e,n,r,o){let t=new Map,a=e.some((i)=>typeof i.x!=="number"||typeof i.y!=="number");if(o==="positioned"&&!a){for(let i of e)t.set(i.id,{x:i.x,y:i.y});return t}let b=n*0.22,c=r*0.5,m=e.filter((i)=>i.kind==="agent"),p=e.filter((i)=>i.kind!=="agent");m.forEach((i,u)=>{let s=Math.max(1,m.length),f=c+(u-(s-1)/2)*80;t.set(i.id,{x:b,y:f})});let d=new Map;for(let i of p){let u=d.get(i.kind)??[];u.push(i),d.set(i.kind,u)}let l=Array.from(d.keys());return l.forEach((i,u)=>{let s=d.get(i),f=n*(0.5+(u+1)/(l.length+1)*0.45);s.forEach((h,v)=>{let g=r*(v+1)/(s.length+1);t.set(h.id,{x:f,y:g})})}),t}function Ra(e,n){let r=Math.max(40,Math.abs(n.x-e.x)*0.4);return`M ${e.x} ${e.y} C ${e.x+r} ${e.y}, ${n.x-r} ${n.y}, ${n.x} ${n.y}`}function Ja(e,n){let r=e.querySelectorAll('template[slot="node"]');for(let o of Array.from(r))if(o.dataset.kind===n)return o;return null}function Ya(e,n){let r=Ja(e,n.kind);if(!r){let o=n.sub?`<div class="node-sub">${rt(n.sub)}</div>`:"";return`<div class="node-h"><span class="tag">${rt(n.kind)}</span></div><div class="node-title">${rt(n.title??n.id)}</div>${o}`}return r.innerHTML.replace(/\$\{title\}/g,rt(n.title??"")).replace(/\$\{sub\}/g,rt(n.sub??"")).replace(/\$\{id\}/g,rt(n.id))}function Wa(e,n,r,o,t,a,b){let c=fo(n,[]),m=fo(r,[]),p=qa(c,o,t,a),d=m.map((i)=>{let u=p.get(i.from),s=p.get(i.to);if(!u||!s)return"";let f=rt(Ra(u,s)),h=i.dashed?" data-dashed":"";return`<path d="${f}" data-kind="${rt(i.kind??"default")}"${h}></path>`}).join(""),l=c.map((i)=>{let u=p.get(i.id);if(!u)return"";let s=u.x/o*100,f=u.y/t*100,h=b&&b===i.id?" data-selected":"",v=i.w?`--node-w: ${i.w}px;`:"",g=rt(`${i.kind}: ${i.title??i.id}`),x=Ya(e,i);return`<div class="node" part="node" data-kind="${rt(i.kind)}" data-id="${rt(i.id)}"${h} tabindex="0" role="button" aria-label="${g}" style="left: ${s}%; top: ${f}%;${v}">${x}</div>`}).join("");return`<svg class="edges" part="edges" viewBox="0 0 ${o} ${t}" preserveAspectRatio="none" aria-hidden="true">${d}</svg>`+l}var ke=P({mount(e,n){let r=e,o=r.canvasEl();if(!o)return;let t=o.ownerDocument.createElement("div");o.appendChild(t);let a=et(t,()=>{let d=r.nodes(),l=r.edges(),i=r.width()||1000,u=r.height()||600,s=r.layout()||"positioned",f=r.selectedId();return Wa(n,d,l,i,u,s,f)}),b="",c=(d)=>{let l=d.target.closest(".node[data-id]");if(!l)return;let i=l.getAttribute("data-id")??"",u=l.getAttribute("data-kind")??"";n.setAttribute("selected-id",i),n.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:i,kind:u},bubbles:!0,composed:!0}))},m=(d)=>{if(d.key!=="Enter"&&d.key!==" ")return;let l=d.target.closest(".node[data-id]");if(!l)return;d.preventDefault();let i=l.getAttribute("data-id")??"",u=l.getAttribute("data-kind")??"";n.setAttribute("selected-id",i),n.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:i,kind:u},bubbles:!0,composed:!0}))},p=(d)=>{let l=d.target.closest(".node[data-id]");if(!l){b="";return}let i=l.getAttribute("data-id")??"";if(i===b)return;b=i;let u=l.getAttribute("data-kind")??"";n.dispatchEvent(new CustomEvent("sui-node-hover",{detail:{id:i,kind:u},bubbles:!0,composed:!0}))};return o.addEventListener("click",c),o.addEventListener("keydown",m),o.addEventListener("mouseover",p),()=>{a(),o.removeEventListener("click",c),o.removeEventListener("keydown",m),o.removeEventListener("mouseover",p)}}});var xg=z("sui-graph-canvas",{props:{nodes:y.string({default:""}),edges:y.string({default:""}),width:y.number({default:1000,reflect:!0}),height:y.number({default:600,reflect:!0}),layout:y.union(["positioned","radial"],{default:"positioned",reflect:!0}),"selected-id":y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.nodes=n.nodes,t.edges=n.edges,t.width=n.width,t.height=n.height,t.layout=n.layout,t.selectedId=n.selectedId;let a=null;if(ke&&typeof ke.setup==="function"){if(e.scope=t,a=ke.setup(e),a&&typeof a==="object")Object.assign(t,a)}let b=r.shadowRoot??r;L(b);let c=document.createElement("div");c.setAttribute("class","canvas"),c.setAttribute("part","canvas"),T(t,"canvasEl",c),b.appendChild(c);let m=document.createElement("slot");if(m.setAttribute("name","node"),m.setAttribute("hidden",""),b.appendChild(m),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let d=new Proxy(t,{get(l,i){return l[i]},set(l,i,u){let s=l[i];if(s!=null&&typeof s.set==="function"){if(typeof u==="function")s.set(()=>u);else s.set(u);return!0}return!1}});if(p.mount){let l=p.mount(d,r);if(typeof l==="function")o.push(l)}if(p.unmount)o.push(()=>p.unmount(d))}}return t}});
