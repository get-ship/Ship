var at=null,lt=null,Ur=0,Yr=new Set;function qe(e){if(at===null)return;if(e.has(at))return;let n=at;if(e.add(n),lt!==null)lt.push(()=>e.delete(n))}function N(e){let n=e,r=new Set,o=()=>{return qe(r),n};return o.set=(t)=>{let a=typeof t==="function"?t(n):t;if(Object.is(a,n))return;if(n=a,Ur>0)for(let m of r)Yr.add(m);else for(let m of[...r])m()},o.__subs=r,o.__peek=()=>n,o}function q(e){let n,r=!0,o=new Set,t=[],a=()=>{if(r)return;r=!0;for(let f of[...o])f()},m=()=>{for(let p of t)p();t=[];let f=at,c=lt;at=a,lt=t;try{n=e()}finally{at=f,lt=c}r=!1},d=()=>{if(qe(o),r)m();return n};return d.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},d.__subs=o,d.__peek=()=>{if(r)m();return n},d}function T(e){let n=void 0,r=!0,o=[],t=()=>{if(!r)return;if(n){try{n()}catch(d){console.error("[sui] effect cleanup threw:",d)}n=void 0}for(let d of o)d();o=[];let a=at,m=lt;at=t,lt=o;try{n=e()}catch(d){console.error("[sui] effect threw:",d)}finally{at=a,lt=m}};return t(),()=>{r=!1;for(let a of o)a();if(o=[],n){try{n()}catch(a){console.error("[sui] effect cleanup threw:",a)}n=void 0}}}var St=null;function wt(e,n){let r=St;St=e;try{return n()}finally{St=r}}function it(e){let n=St;if(!n)return;n.cleanup(e)}function At(){let e=[],n=!1;return{cleanup(o){if(n){try{o()}catch(t){console.error("[sui/scope] late cleanup threw:",t)}return}e.push(o)},dispose(){if(n)return;n=!0;for(let o=e.length-1;o>=0;o--)try{e[o]()}catch(t){console.error("[sui/scope] cleanup threw:",t)}e.length=0},get disposed(){return n}}}var Xr=Symbol("sui.shared");function Je(e){return!!e?.[Xr]}function ye(e,n,r){e[`__sui_ctx_${n}`]=r}function xe(e){let n={};for(let r in e)if(Object.prototype.hasOwnProperty.call(e,r)&&Je(e[r]))n[r]=e[r];return n}function W(e=null){return N(null)}function D(e,n,r){let o=e[n];if(typeof o==="function"&&typeof o.set==="function")return o.set(r),o;let t=N(r);return e[n]=t,t}function tt(e,n,r){let o=()=>{if(e.assignedNodes({flatten:!0}).some(qr))n.removeAttribute(r);else n.setAttribute(r,"")};return e.addEventListener("slotchange",o),o(),()=>{e.removeEventListener("slotchange",o)}}function qr(e){if(e.nodeType===Node.TEXT_NODE)return(e.textContent??"").trim().length>0;if(e.nodeType===Node.ELEMENT_NODE){let n=e;return(n.textContent??"").trim().length>0||n.childElementCount>0}return!1}function We(e,n,r={}){Ze(e,n,r)}function Ze(e,n,r){if(r.skip?.(e))return;if(e.tagName!==n.tagName){e.replaceWith(n.cloneNode(!0));return}Jr(e,n),Wr(e,n,r)}function Jr(e,n){let r=Array.from(e.attributes);for(let t of r)if(!n.hasAttribute(t.name))e.removeAttribute(t.name);let o=Array.from(n.attributes);for(let t of o)if(e.getAttribute(t.name)!==t.value)e.setAttribute(t.name,t.value);if(e instanceof HTMLInputElement&&n instanceof HTMLInputElement){if(n.hasAttribute("value")&&e.value!==n.value)e.value=n.value;if(e.checked!==n.checked)e.checked=n.checked}}function Wr(e,n,r){let o=Array.from(e.childNodes),t=Array.from(n.childNodes),a=Math.max(o.length,t.length);for(let m=0;m<a;m++){let d=o[m],f=t[m];if(!f&&d){e.removeChild(d);continue}if(!d&&f){e.appendChild(f.cloneNode(!0));continue}if(!d||!f)continue;Zr(d,f,e,r)}}function Zr(e,n,r,o){if(e.nodeType!==n.nodeType){r.replaceChild(n.cloneNode(!0),e);return}if(e.nodeType===Node.TEXT_NODE||e.nodeType===Node.COMMENT_NODE){if(e.nodeValue!==n.nodeValue)e.nodeValue=n.nodeValue;return}if(e.nodeType===Node.ELEMENT_NODE)Ze(e,n,o)}function R(e,n){return T(()=>{let r=n();e.textContent=r==null?"":String(r)})}function k(e,n,r){return T(()=>{let o=r();if(o==null||o===!1||o==="")e.removeAttribute(n);else if(o===!0)e.setAttribute(n,"");else e.setAttribute(n,String(o))})}function mt(e,n,r){return T(()=>{if(r())e.classList.add(n);else e.classList.remove(n)})}function $(e,n,r){return T(()=>{e[n]=r()})}function ut(e,n,r){let o=T(()=>{let a=n();if(e.value!==a)e.value=a}),t=()=>{let a=e.value;if(n.set(a),r)r(a)};return e.addEventListener("input",t),()=>{o(),e.removeEventListener("input",t)}}function dt(e,n){let r=T(()=>{let t=n();if(e.checked!==t)e.checked=t}),o=()=>n.set(e.checked);return e.addEventListener("change",o),()=>{r(),e.removeEventListener("change",o)}}function nt(e,n){let o=e.namespaceURI==="http://www.w3.org/2000/svg",t=e.tagName.toLowerCase();return T(()=>{let a=n(),m=o?e.ownerDocument.createElementNS("http://www.w3.org/2000/svg",t):e.ownerDocument.createElement(t);m.innerHTML=a,We(e,m)})}function I(e,n,r,o={}){let t=o.modifiers??[],a,m=!0,d=(c)=>{for(let i of t)switch(i.kind){case"preventDefault":c.preventDefault();break;case"stopPropagation":c.stopPropagation();break;case"stopImmediatePropagation":c.stopImmediatePropagation();break;case"self":if(c.target!==e)return;break;case"trusted":if(!c.isTrusted)return;break;case"key":{if(!Qe(c,i.arg??""))return;break}}let p=t.find((i)=>i.kind==="debounce"),l=t.find((i)=>i.kind==="throttle");if(p){let i=Number(p.arg)||0;if(a!==void 0)clearTimeout(a);a=setTimeout(()=>{a=void 0,r(c)},i);return}if(l){let i=Number(l.arg)||0;if(!m)return;m=!1,r(c),a=setTimeout(()=>{m=!0},i);return}r(c)},f={};if(o.capture)f.capture=!0;if(o.passive)f.passive=!0;if(o.once)f.once=!0;return e.addEventListener(n,d,f),()=>{if(e.removeEventListener(n,d,f),a!==void 0)clearTimeout(a)}}function Qe(e,n){let r=n.split("+").map((p)=>p.toLowerCase()),o=r.pop()??"",t=r.includes("meta")||r.includes("cmd"),a=r.includes("ctrl")||r.includes("control"),m=r.includes("shift"),d=r.includes("alt")||r.includes("option"),f=r.includes("mod"),c=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(t&&!e.metaKey)return!1;if(a&&!e.ctrlKey)return!1;if(m&&!e.shiftKey)return!1;if(d&&!e.altKey)return!1;if(f&&!(c?e.metaKey:e.ctrlKey))return!1;return e.key.toLowerCase()===o}function Ie(e,n,r){e.dispatchEvent(new CustomEvent(n,{detail:r,bubbles:!0,composed:!0}))}function Ve(e,n){let r=e,o=Object.keys(n);for(let t of o)r[t]=n[t];it(()=>{for(let t of o)if(r[t]===n[t])try{delete r[t]}catch{r[t]=void 0}})}function Mt(e){let n=e;n.emit=function(r,o){Ie(e,r,o)},n.expose=function(r){Ve(e,r)}}function Z(e,n,r,o){e.addEventListener(n,r,o),it(()=>e.removeEventListener(n,r,o))}function we(e,n){let r=T(()=>{let o=e();if(!o)return;let t=()=>{n(o.assignedElements())};return o.addEventListener("slotchange",t),t(),()=>o.removeEventListener("slotchange",t)});it(r)}function Qr(e){let n=e.length,r=Array(n).fill(!1);if(n===0)return r;let o=Array(n).fill(-1),t=[];for(let m=0;m<n;m++){let d=e[m],f=0,c=t.length;while(f<c){let p=f+c>>>1;if(e[t[p]]<d)f=p+1;else c=p}if(f>0)o[m]=t[f-1];t[f]=m}let a=t[t.length-1];while(a>=0)r[a]=!0,a=o[a];return r}function Y(e){let{anchor:n,items:r,key:o,render:t}=e,a=[],m=!0,d=T(()=>{if(!m)return;let f=r(),c=new Map;for(let x=0;x<f.length;x++){let E=f[x],w=o(E,x);if(c.has(w))console.warn("[sui] {#for}: duplicate key",w,"at index",x,"(prior at",c.get(w).index,"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");c.set(w,{item:E,index:x})}let p=new Map;for(let x of a)p.set(x.key,x);for(let x of a)if(!c.has(x.key)){for(let E of x.nodes)try{E.parentNode?.removeChild(E)}catch{}x.cleanup?.()}let l=[];for(let x=0;x<f.length;x++){let E=f[x],w=o(E,x),A=p.get(w),L;if(A)A.item=E,A.update?.(E,x),L=A;else{let C=t(E,x);L={key:w,item:E,nodes:C.nodes,...C.cleanup&&{cleanup:C.cleanup},...C.update&&{update:C.update}}}l.push(L)}let i=new Map;for(let x=0;x<a.length;x++)i.set(a[x].key,x);let s=[],u=[];for(let x=0;x<l.length;x++){let E=i.get(l[x].key);if(E!==void 0)s.push(x),u.push(E)}let b=Qr(u),h=new Set;for(let x=0;x<s.length;x++)if(b[x])h.add(s[x]);let g=n.parentNode,y=n;for(let x=0;x<l.length;x++){if(!g)break;let E=l[x];if(h.has(x))y=E.nodes[E.nodes.length-1]??y;else for(let w of E.nodes)g.insertBefore(w,y.nextSibling),y=w}a=l});return()=>{m=!1,d();for(let f of a){for(let c of f.nodes)try{c.parentNode?.removeChild(c)}catch{}f.cleanup?.()}a=[]}}function G(e){let{anchor:n}=e,{test:r,branches:o}=Ir(e)?{test:e.test,branches:e.branches}:Vr(e),t=null,a=-2,m=()=>{if(!t)return;for(let f of t.nodes)try{f.parentNode?.removeChild(f)}catch{}t.cleanup?.(),t=null},d=T(()=>{let f=r();if(f===a)return;if(m(),a=f,f<0||f>=o.length){t=null;return}t=o[f]();let c=n.parentNode;if(!c)return;for(let p of t.nodes)c.insertBefore(p,n)});return()=>{d(),m()}}function Ir(e){return typeof e.test==="function"}function Vr(e){let{cond:n,then_:r,else_:o}=e;return{test:()=>n()?0:o?1:-1,branches:o?[r,o]:[r]}}var Ae=(e)=>e??"",Ee=(e)=>{if(e===null||e==="")return NaN;return Number(e)},ke=(e)=>{if(e===null)return!1;if(e.toLowerCase()==="false")return!1;return!0};function Pt(e){return(n)=>{if(n===null||n==="")return e;try{return JSON.parse(n)}catch{return e}}}function Ce(e,n){let r=new Set(e);return(o)=>o!==null&&r.has(o)?o:n}var Nt=(e)=>{switch(e.type){case"string":return Ae;case"bool":return ke;case"number":return Ee;case"union":{let n=e.values??[],r=e.default??n[0]??"";return Ce(n,r)}case"object":return Pt(e.default);case"array":return Pt(e.default??[])}};function $e(e,n,r,o){if(!n)return;let t={},a,m=r.formValue;if(m!==void 0){let{sig:f,propDef:c}=Ke(r,o,m,"formValue"),p=c.default,l=Nt(c);a=T(()=>{let i=f();n.setFormValue(Kr(i))}),t.reset=()=>{f.set(p)},t.restore=(i)=>{f.set(l(i))}}let d=r.formDisabled;if(d!==void 0){let{sig:f,propDef:c}=Ke(r,o,d,"formDisabled");if(c.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${d}" must be a bool-typed prop`);t.disabled=(p)=>{f.set(!!p)}}if(e.__shipSetFormHooks=t,a)e.__shipSetFormValueCleanup=a;return a}function _e(e){let n=e;delete n.__shipSetFormHooks,delete n.__shipSetFormValueCleanup}function Kr(e){if(e===null||e===void 0)return"";if(typeof e==="boolean")return e?"on":"";return String(e)}function Ke(e,n,r,o){let t=e.props[r];if(!t)throw Error(`[sui-runtime] ${o}: "${r}" — no matching prop on this component`);let a=n[bt(r)];if(!a)throw Error(`[sui-runtime] ${o}: "${r}" — prop signal missing`);return{sig:a,propDef:t}}var v={string(e={}){return{type:"string",...e}},bool(e={}){return{type:"bool",...e}},number(e={}){return{type:"number",...e}},union(e,n={}){return{type:"union",values:e,...n}},object(e={}){return{type:"object",...e}},array(e={}){return{type:"array",...e}}},$r=(e,n)=>{if(n===void 0||n===null)return null;switch(e.type){case"string":return n===""?null:String(n);case"number":return Number.isNaN(n)?null:String(n);case"bool":return n?"":null;case"union":return String(n);case"object":case"array":try{return JSON.stringify(n)}catch{return null}}},_r=(e,n)=>{let r=`ship-set-style-${e}`;if(document.getElementById(r))return;let o=document.createElement("style");o.id=r,o.textContent=`[data-component="${e}"] { ${n} }`,document.head.appendChild(o)},to=(e,n,r,o)=>{let t=e.getAttribute(n),a=t!==null?o(t):r.default;return N(a)},no=(e)=>{let n=[],r={};for(let[a,m]of Object.entries(e.props??{}))n.push(a),r[a]={camel:bt(a),decode:Nt(m)};let o=e.formAssociated===!0||e.formValue!==void 0||e.formDisabled!==void 0,t={observedAttrs:n,props:r};if(o)t.formAssociated=!0;return t};function z(e,n){let r=no(n),o=Object.entries(n.props??{});Le(e,r,(t,a,m)=>{let d=null;if(n.shadow){if(d=t.shadowRoot??t.attachShadow({mode:"open"}),n.styles){let c=document.createElement("style");c.textContent=n.styles,d.appendChild(c)}}else if(n.styles)_r(e,n.styles);let f=[];for(let[c,p]of o){let l=bt(c),i=r.props[c].decode,s=to(t,c,p,i);if(a[l]=s,p.reflect){let u=!0,b=T(()=>{let h=s();if(u){u=!1;return}let g=$r(p,h);if(g===null)t.removeAttribute(c);else t.setAttribute(c,g)});f.push(b)}}if(n.formValue!==void 0||n.formDisabled!==void 0){let c=$e(t,m,n,a);if(typeof c==="function")f.push(c)}if(n.setup){let c={props:a,host:t,cleanups:f};if(m)c.internals=m;n.setup(c)}return()=>{_e(t);for(let c of f.splice(0))try{c()}catch(p){console.error(`[ship-set:${e}] cleanup threw:`,p)}}})}var tr=new Set;function Le(e,n,r){if(customElements.get(e)){if(!tr.has(e))tr.add(e),console.warn(`[sui-runtime] customElements.define("${e}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(n.formAssociated){class t extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return n.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=e,Mt(this),this.scope=At(),this.cleanup=wt(this.scope,()=>r(this,this.propSignals,this.internals))}disconnectedCallback(){try{this.cleanup?.()}catch(a){console.error(`[ship-set:${e}] cleanup threw:`,a)}this.cleanup=void 0;try{this.scope?.dispose()}catch(a){console.error(`[ship-set:${e}] scope dispose threw:`,a)}this.scope=void 0}attributeChangedCallback(a,m,d){let f=n.props[a];if(!f)return;let c=this.propSignals[f.camel];if(c)c.set(f.decode(d))}formResetCallback(){let a=this.__shipSetFormHooks;try{a?.reset?.()}catch(m){console.error(`[ship-set:${e}] formResetCallback threw:`,m)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(a,m){let d=this.__shipSetFormHooks;try{d?.restore?.(a)}catch(f){console.error(`[ship-set:${e}] formStateRestoreCallback threw:`,f)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:a,mode:m}}))}formDisabledCallback(a){let m=this.__shipSetFormHooks;try{m?.disabled?.(a)}catch(d){console.error(`[ship-set:${e}] formDisabledCallback threw:`,d)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:a}}))}}customElements.define(e,t);return}class o extends HTMLElement{static get observedAttributes(){return n.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=e,Mt(this),this.scope=At(),this.cleanup=wt(this.scope,()=>r(this,this.propSignals))}disconnectedCallback(){try{this.cleanup?.()}catch(t){console.error(`[ship-set:${e}] cleanup threw:`,t)}this.cleanup=void 0;try{this.scope?.dispose()}catch(t){console.error(`[ship-set:${e}] scope dispose threw:`,t)}this.scope=void 0}attributeChangedCallback(t,a,m){let d=n.props[t];if(!d)return;let f=this.propSignals[d.camel];if(f)f.set(d.decode(m))}}customElements.define(e,o)}function bt(e){return e.replace(/-([a-z])/g,(n,r)=>r.toUpperCase())}function ze(e,n,r){let o=oo(e.state,{ship:n.ship,...r}),t={};for(let[i,s]of Object.entries(o))t[i]=N(s);let a={ship:n.ship,route:n.route??{},navigate:n.navigate??((i)=>{}),...r,...t},m=ao(t,a),d={...a},f={};for(let[i,s]of Object.entries(e.derived??{}))f[i]=q(()=>s(d));Object.assign(d,f);let c={};for(let[i,s]of Object.entries(e.on??{}))c[i]=(...u)=>s(m,...u);let p={},l=typeof e.use==="function"?e.use(d):e.use??{};for(let[i,s]of Object.entries(l))p[i]=ze(s,n,d);return{...t,...f,...c,...p}}function oo(e,n){if(e===void 0)return{};if(typeof e==="function")return e(n);return e}function ao(e,n){return new Proxy({},{get(r,o){if(o in e)return e[o]();return n[o]},set(r,o,t){if(o in e)return e[o].set(t),!0;return console.warn(`[ship-set] hook handler tried to write s.${o} = … but ${o} is not a hook state key. The assignment was ignored.`),!0},has(r,o){return o in e||o in n},ownKeys(){return Object.keys(e)},getOwnPropertyDescriptor(r,o){if(o in e)return{configurable:!0,enumerable:!0,writable:!0,value:e[o]()};return}})}var nr="__shipSetLifecycle";function er(e,n){let r=io(e,n),o={ship:n.ship,route:n.route??{},navigate:n.navigate??((i)=>{}),...n.scope??{}},t=r.scope?r.scope(o,n.host):{};Object.assign(o,t);let a=so(o,n.stateKeys,t),m=lo(a,o),d={...o},f=uo(r,d);Object.assign(d,f);let c=po(r,d,n);for(let[i,s]of Object.entries(c))d[i]=s;let p=co(r,m),l={...t,...f,...p,...c};if(Object.keys(t).length>0&&n.host){let i=xe(t);if(Object.keys(i).length>0){let s=n.host,u=s.tagName.toLowerCase();ye(s,u,i)}}if(r.mount||r.unmount){let i={state:m};if(r.mount)i.mount=r.mount;if(r.unmount)i.unmount=r.unmount;Object.defineProperty(l,nr,{value:i,enumerable:!0,configurable:!0,writable:!1})}return l}function io(e,n){if(typeof e==="function")return e({ship:n.ship});return e}function so(e,n,r){let o={},t=n!==void 0,a=r?Object.keys(r):[];if(t||a.length>0){if(n)for(let m of n){let d=e[m];if(je(d))o[m]=d}for(let m of a){if(m in o)continue;let d=e[m];if(je(d))o[m]=d}return o}for(let[m,d]of Object.entries(e))if(je(d))o[m]=d;return o}function je(e){return typeof e==="function"&&"set"in e&&typeof e.set==="function"}function lo(e,n){return new Proxy({},{get(r,o){if(o in e)return e[o]();return n[o]},set(r,o,t){if(o in e){if(typeof t==="function")e[o].set(()=>t);else e[o].set(t);return!0}return console.warn(`[ship-set] handler tried to write s.${o} = … but ${o} is not declared as <state name="${o}"> in the template. The assignment was ignored.`),!0},has(r,o){return o in e||o in n},ownKeys(){return Object.keys(e)},getOwnPropertyDescriptor(r,o){if(o in e)return{configurable:!0,enumerable:!0,writable:!0,value:e[o]()};return}})}function uo(e,n){let r=e.derived??{},o={};for(let[t,a]of Object.entries(r))o[t]=q(()=>a(n));return o}function co(e,n){let r=e.on??{},o={};for(let[t,a]of Object.entries(r))o[t]=(...m)=>a(n,...m);return o}function po(e,n,r){if(e.use===void 0)return{};let o=typeof e.use==="function"?e.use(n):e.use,t={};for(let[a,m]of Object.entries(o))t[a]=ze(m,r,n);return t}function S(e){return{setup:(n)=>er(e,n),__def:e}}if(typeof customElements<"u"){for(let e of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(e))customElements.define(e,class extends HTMLElement{})}var ho=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,vo=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function rr(e){if(typeof CSSStyleSheet>"u")return null;try{let n=new CSSStyleSheet;return n.replaceSync(e),n}catch{return null}}var or=rr(ho),ar=rr(vo);function j(e){if(typeof ShadowRoot>"u")return;if(!(e instanceof ShadowRoot))return;let n=[ar,or].filter((r)=>r!=null);if(n.length===0)return;try{e.adoptedStyleSheets=[...n,...e.adoptedStyleSheets??[]]}catch{}}var Ts=z("sui-status-dot",{props:{tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"success",reflect:!0}),pulse:v.bool({default:!1,reflect:!0}),size:v.union(["sm","md"],{default:"md",reflect:!0}),label:v.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.pulse=n.pulse,t.size=n.size,t.label=n.label;let a=r.shadowRoot??r;j(a);let m=document.createElement("span");return m.setAttribute("class","dot"),m.setAttribute("part","dot"),o.push(k(m,"role",()=>t.label()?"img":null)),o.push(k(m,"aria-label",()=>t.label()||null)),o.push(k(m,"aria-hidden",()=>t.label()?null:"true")),a.appendChild(m),t}});var Ss=z("sui-spinner",{props:{size:v.union(["xs","sm","md","lg"],{default:"md",reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),label:v.string({default:"Loading"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.size=n.size,t.tone=n.tone,t.label=n.label;let a=r.shadowRoot??r;j(a);let m=document.createElement("span");return m.setAttribute("class","ring"),m.setAttribute("part","ring"),o.push(k(m,"role",()=>t.label()?"status":null)),o.push(k(m,"aria-label",()=>t.label()||null)),o.push(k(m,"aria-hidden",()=>t.label()?null:"true")),a.appendChild(m),t}});var Ns=z("sui-badge",{props:{tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:v.union(["solid","soft","outline"],{default:"soft",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),pill:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.pill=n.pill;let a=r.shadowRoot??r;j(a);let m=document.createElement("span");m.setAttribute("class","badge"),m.setAttribute("part","badge");let d=document.createElement("span");d.setAttribute("class","leading"),d.setAttribute("part","leading");let f=document.createElement("slot");f.setAttribute("name","leading"),d.appendChild(f),m.appendChild(d);let c=document.createElement("slot");return m.appendChild(c),a.appendChild(m),t}});var Ht=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","button");if(!n.hasAttribute("tabindex")&&!r.disabled())n.setAttribute("tabindex","0");let o=T(()=>{let m=r.selected(),d=r.disabled();if(n.setAttribute("aria-pressed",m?"true":"false"),d){if(n.setAttribute("aria-disabled","true"),n.getAttribute("tabindex")!=="-1")n.setAttribute("tabindex","-1")}else if(n.removeAttribute("aria-disabled"),n.getAttribute("tabindex")==="-1")n.setAttribute("tabindex","0")}),t=(m)=>{if(r.disabled())return;if(m.key==="Enter"||m.key===" ")m.preventDefault(),n.click();else if(r.removable()&&(m.key==="Backspace"||m.key==="Delete"))m.preventDefault(),n.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};n.addEventListener("keydown",t);let a=T(()=>{let m=r.removeBtn?.();if(!m)return;let d=(f)=>{if(f.stopPropagation(),r.disabled())return;n.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};return m.addEventListener("click",d),()=>m.removeEventListener("click",d)});return()=>{o(),n.removeEventListener("keydown",t),a()}}});var Us=z("sui-chip",{props:{tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0}),variant:v.union(["solid","soft","outline","ghost"],{default:"soft",reflect:!0}),size:v.union(["sm","md","lg"],{default:"sm",reflect:!0}),selected:v.bool({default:!1,reflect:!0}),removable:v.bool({default:!1,reflect:!0}),disabled:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.selected=n.selected,t.removable=n.removable,t.disabled=n.disabled;let a=null;if(Ht&&typeof Ht.setup==="function"){if(e.scope=t,a=Ht.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(s)=>G({anchor:s,test:()=>{if(t.removable())return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("button");h.setAttribute("class","remove"),h.setAttribute("part","remove"),h.setAttribute("type","button"),h.setAttribute("aria-label","Remove"),h.setAttribute("tabindex","-1"),D(t,"removeBtn",h);let g=document.createElement("svg");g.setAttribute("viewbox","0 0 10 10"),g.setAttribute("aria-hidden","true");let y=document.createElement("path");return y.setAttribute("d","M2 2 L8 8 M8 2 L2 8"),g.appendChild(y),h.appendChild(g),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let x of b)try{x()}catch(E){}}}}]}),d=r.shadowRoot??r;j(d);let f=document.createElement("span");f.setAttribute("class","chip"),f.setAttribute("part","chip");let c=document.createElement("span");c.setAttribute("class","leading"),c.setAttribute("part","leading");let p=document.createElement("slot");p.setAttribute("name","leading"),c.appendChild(p),f.appendChild(c);let l=document.createElement("slot");f.appendChild(l);let i=document.createComment("if");if(f.appendChild(i),o.push(m(i)),d.appendChild(f),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,g){let y=b[h];if(y!=null&&typeof y.set==="function"){if(typeof g==="function")y.set(()=>g);else y.set(g);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var Ft=S({mount(e,n){let r=e,o=(a)=>{if(a.key==="Enter"||a.key===" ")a.preventDefault(),n.click()},t=T(()=>{if(r.interactive()){if(!n.hasAttribute("role"))n.setAttribute("role","button");if(!n.hasAttribute("tabindex"))n.setAttribute("tabindex","0");return n.addEventListener("keydown",o),()=>n.removeEventListener("keydown",o)}else{if(n.getAttribute("role")==="button")n.removeAttribute("role");if(n.getAttribute("tabindex")==="0")n.removeAttribute("tabindex")}});return()=>{t()}}});var Zs=z("sui-card",{props:{tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:v.union(["default","outline"],{default:"default",reflect:!0}),padding:v.union(["none","sm","md","lg"],{default:"md",reflect:!0}),interactive:v.bool({default:!1,reflect:!0}),"accent-bar":v.bool({default:!1,reflect:!0}),"status-rail":v.bool({default:!1,reflect:!0}),fill:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.padding=n.padding,t.interactive=n.interactive,t.accentBar=n.accentBar,t.statusRail=n.statusRail,t.fill=n.fill;let a=null;if(Ft&&typeof Ft.setup==="function"){if(e.scope=t,a=Ft.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","header"),d.setAttribute("part","header");let f=document.createElement("slot");f.setAttribute("name","header"),o.push(tt(f,r,"data-no-header")),d.appendChild(f),m.appendChild(d);let c=document.createElement("div");c.setAttribute("class","content"),c.setAttribute("part","content");let p=document.createElement("slot");c.appendChild(p),m.appendChild(c);let l=document.createElement("div");l.setAttribute("class","footer"),l.setAttribute("part","footer");let i=document.createElement("slot");if(i.setAttribute("name","footer"),o.push(tt(i,r,"data-no-footer")),l.appendChild(i),m.appendChild(l),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,g){let y=b[h];if(y!=null&&typeof y.set==="function"){if(typeof g==="function")y.set(()=>g);else y.set(g);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var Vs=z("sui-link",{props:{href:v.string({default:""}),external:v.bool({default:!1,reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:v.union(["default","underline","subtle"],{default:"default",reflect:!0}),target:v.string({default:""}),rel:v.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.href=n.href,t.external=n.external,t.tone=n.tone,t.variant=n.variant,t.target=n.target,t.rel=n.rel;let a=(p)=>G({anchor:p,test:()=>{if(t.external())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("span");s.setAttribute("class","sr-only");let u=document.createTextNode("(opens in new tab)");s.appendChild(u),l.appendChild(s);let b=document.createElement("span");b.setAttribute("class","ext"),b.setAttribute("aria-hidden","true");let h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round");let g=document.createElement("path");return g.setAttribute("d","M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3"),h.appendChild(g),b.appendChild(h),l.appendChild(b),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let y of i)try{y()}catch(x){}}}}]}),m=r.shadowRoot??r;j(m);let d=document.createElement("a");d.setAttribute("part","link"),o.push(k(d,"href",()=>t.href())),o.push(k(d,"target",()=>t.external()?"_blank":t.target())),o.push(k(d,"rel",()=>t.external()?"noopener noreferrer":t.rel()));let f=document.createElement("slot");d.appendChild(f);let c=document.createComment("if");return d.appendChild(c),o.push(a(c)),m.appendChild(d),t}});var _s=z("sui-divider",{props:{orientation:v.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:v.union(["solid","dashed"],{default:"solid",reflect:!0}),tone:v.union(["default","muted","primary","destructive"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.orientation=n.orientation,t.variant=n.variant,t.tone=n.tone;let a=r.shadowRoot??r;j(a);let m=document.createElement("hr");return m.setAttribute("class","line"),m.setAttribute("part","line"),m.setAttribute("role","separator"),o.push(k(m,"aria-orientation",()=>t.orientation())),a.appendChild(m),t}});var el=z("sui-skeleton",{props:{shape:v.union(["text","line","block","circle"],{default:"text",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.shape=n.shape,t.size=n.size;let a=r.shadowRoot??r;j(a);let m=document.createElement("span");return m.setAttribute("class","sk"),m.setAttribute("part","sk"),m.setAttribute("role","status"),m.setAttribute("aria-label","Loading"),m.setAttribute("aria-live","polite"),m.setAttribute("aria-busy","true"),a.appendChild(m),t}});var Bt=S({mount(e,n){let r=e,o=(a)=>{if(a.key==="Enter"||a.key===" ")a.preventDefault(),n.click()},t=T(()=>{if(r.interactive()){if(!n.hasAttribute("role"))n.setAttribute("role","button");if(!n.hasAttribute("tabindex"))n.setAttribute("tabindex","0");return n.addEventListener("keydown",o),()=>n.removeEventListener("keydown",o)}else{if(n.getAttribute("role")==="button")n.removeAttribute("role");if(n.getAttribute("tabindex")==="0")n.removeAttribute("tabindex")}});return()=>{t(),n.removeEventListener("keydown",o)}}});var ll=z("sui-stat",{props:{tone:v.union(["default","primary","info","success","warning","destructive","accent"],{default:"default",reflect:!0}),interactive:v.bool({default:!1,reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.interactive=n.interactive,t.size=n.size;let a=null;if(Bt&&typeof Bt.setup==="function"){if(e.scope=t,a=Bt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("span");d.setAttribute("class","label"),d.setAttribute("part","label");let f=document.createElement("slot");f.setAttribute("name","label"),d.appendChild(f),m.appendChild(d);let c=document.createElement("span");c.setAttribute("class","value-row"),c.setAttribute("part","value-row");let p=document.createElement("span");p.setAttribute("class","value"),p.setAttribute("part","value");let l=document.createElement("slot");p.appendChild(l),c.appendChild(p);let i=document.createElement("span");i.setAttribute("class","trend"),i.setAttribute("part","trend");let s=document.createElement("slot");s.setAttribute("name","trend"),i.appendChild(s),c.appendChild(i),m.appendChild(c);let u=document.createElement("span");u.setAttribute("class","hint"),u.setAttribute("part","hint");let b=document.createElement("slot");if(b.setAttribute("name","hint"),u.appendChild(b),m.appendChild(u),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let g=new Proxy(t,{get(y,x){return y[x]},set(y,x,E){let w=y[x];if(w!=null&&typeof w.set==="function"){if(typeof E==="function")w.set(()=>E);else w.set(E);return!0}return!1}});if(h.mount){let y=h.mount(g,r);if(typeof y==="function")o.push(y)}if(h.unmount)o.push(()=>h.unmount(g))}}return t}});var cl=z("sui-mark",{props:{tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"warning",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone;let a=r.shadowRoot??r;j(a);let m=document.createElement("mark");m.setAttribute("part","mark");let d=document.createElement("slot");return m.appendChild(d),a.appendChild(m),t}});var Gt=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","group");let o=T(()=>{let t=r.keys();if(t)n.setAttribute("aria-label",`Keyboard shortcut: ${t}`);else n.removeAttribute("aria-label")});return()=>o()}});var vl=z("sui-kbd",{props:{keys:v.string({default:""}),size:v.union(["sm","md"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.keys=n.keys,t.size=n.size;let a=null;if(Gt&&typeof Gt.setup==="function"){if(e.scope=t,a=Gt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("kbd");d.setAttribute("part","kbd");let f=document.createElement("slot");if(d.appendChild(f),m.appendChild(d),a!=null){let c=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(c){let p=new Proxy(t,{get(l,i){return l[i]},set(l,i,s){let u=l[i];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(c.mount){let l=c.mount(p,r);if(typeof l==="function")o.push(l)}if(c.unmount)o.push(()=>c.unmount(p))}}return t}});var xl=z("sui-avatar",{props:{src:v.string({default:""}),name:v.string({default:""}),initials:v.string({default:""}),"icon-slug":v.string({default:""}),size:v.union(["xs","sm","md","lg","xl"],{default:"md",reflect:!0}),shape:v.union(["round","square"],{default:"round",reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.src=n.src,t.name=n.name,t.initials=n.initials,t.iconSlug=n.iconSlug,t.size=n.size,t.shape=n.shape,t.tone=n.tone;let a=(c)=>G({anchor:c,test:()=>{if(t.iconSlug()||t.src())return 0;return 1},branches:[()=>{let p=document.createDocumentFragment(),l=[],i=document.createElement("img");return i.setAttribute("part","img"),l.push(k(i,"src",()=>t.iconSlug()?"https://cdn.simpleicons.org/"+t.iconSlug():t.src())),l.push(k(i,"alt",()=>t.name())),p.appendChild(i),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let s of l)try{s()}catch(u){}}}},()=>{let p=document.createDocumentFragment(),l=[],i=document.createElement("span");i.setAttribute("part","initials");let s=document.createTextNode("");return i.appendChild(s),l.push(R(s,()=>t.initials()?t.initials().slice(0,2).toUpperCase():t.name().trim().slice(0,1).toUpperCase()||"?")),p.appendChild(i),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let u of l)try{u()}catch(b){}}}}]}),m=r.shadowRoot??r;j(m);let d=document.createElement("span");d.setAttribute("class","root"),d.setAttribute("part","root"),d.setAttribute("role","img"),o.push(k(d,"aria-label",()=>t.name()||t.initials()||"avatar"));let f=document.createComment("if");return d.appendChild(f),o.push(a(f)),m.appendChild(d),t}});var El=z("sui-progress",{props:{value:v.number({default:0}),max:v.number({default:100}),tone:v.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),indeterminate:v.bool({default:!1,reflect:!0}),label:v.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.max=n.max,t.tone=n.tone,t.size=n.size,t.indeterminate=n.indeterminate,t.label=n.label;let a=r.shadowRoot??r;j(a);let m=document.createElement("div");m.setAttribute("class","track"),m.setAttribute("part","track"),m.setAttribute("role","progressbar"),o.push(k(m,"aria-valuenow",()=>t.indeterminate()?null:t.value())),m.setAttribute("aria-valuemin","0"),o.push(k(m,"aria-valuemax",()=>t.max())),o.push(k(m,"aria-busy",()=>t.indeterminate()?"true":"false")),o.push(k(m,"aria-label",()=>t.label()));let d=document.createElement("div");return d.setAttribute("class","fill"),d.setAttribute("part","fill"),o.push(k(d,"style",()=>t.indeterminate()?"":"width: clamp(0%, "+t.value()/(t.max()||1)*100+"%, 100%)")),m.appendChild(d),a.appendChild(m),t}});var Ll=z("sui-progress-ring",{props:{value:v.number({default:0}),max:v.number({default:100}),size:v.number({default:48}),"track-width":v.number({default:4}),tone:v.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),indeterminate:v.bool({default:!1,reflect:!0}),label:v.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.max=n.max,t.size=n.size,t.trackWidth=n.trackWidth,t.tone=n.tone,t.indeterminate=n.indeterminate,t.label=n.label;let a=r.shadowRoot??r;j(a);let m=document.createElement("div");m.setAttribute("class","ring"),o.push(k(m,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let d=document.createElement("svg");d.setAttribute("class","ring-svg"),o.push(k(d,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),d.setAttribute("role","progressbar"),d.setAttribute("aria-valuemin","0"),o.push(k(d,"aria-valuemax",()=>t.max())),o.push(k(d,"aria-valuenow",()=>t.indeterminate()?null:t.value()<0?0:t.value()>t.max()?t.max():t.value())),o.push(k(d,"aria-label",()=>t.label()||null)),d.setAttribute("xmlns","http://www.w3.org/2000/svg");let f=document.createElement("circle");f.setAttribute("class","track"),f.setAttribute("part","track"),o.push(k(f,"cx",()=>t.size()/2)),o.push(k(f,"cy",()=>t.size()/2)),o.push(k(f,"r",()=>(t.size()-t.trackWidth())/2)),f.setAttribute("fill","none"),o.push(k(f,"stroke-width",()=>t.trackWidth())),d.appendChild(f);let c=document.createElement("circle");c.setAttribute("class","bar"),c.setAttribute("part","ring"),o.push(k(c,"cx",()=>t.size()/2)),o.push(k(c,"cy",()=>t.size()/2)),o.push(k(c,"r",()=>(t.size()-t.trackWidth())/2)),c.setAttribute("fill","none"),o.push(k(c,"stroke-width",()=>t.trackWidth())),c.setAttribute("stroke-linecap","round"),o.push(k(c,"stroke-dasharray",()=>3.141592653589793*(t.size()-t.trackWidth()))),o.push(k(c,"stroke-dashoffset",()=>t.indeterminate()?3.141592653589793*(t.size()-t.trackWidth())*0.25:3.141592653589793*(t.size()-t.trackWidth())*(1-(t.value()<0?0:t.value()>(t.max()||1)?1:t.value()/(t.max()||1))))),d.appendChild(c),m.appendChild(d);let p=document.createElement("div");p.setAttribute("class","label"),p.setAttribute("aria-hidden","true");let l=document.createElement("slot");return p.appendChild(l),m.appendChild(p),a.appendChild(m),t}});var Te={default:"var(--foreground)",primary:"var(--primary)",info:"var(--info)",success:"var(--success)",warning:"var(--warning)",destructive:"var(--destructive)",accent:"var(--accent)",muted:"var(--muted-foreground)"};function V(e){if(!e)return Te.default;return Te[e]??Te.default}function ht(e){if(!e)return[];return e.split(",").map((n)=>Number.parseFloat(n.trim())).filter((n)=>Number.isFinite(n))}function Et(e){if(!e)return[];return e.split(",").map((n)=>n.trim())}function rt(e,n){if(!e)return n;try{return JSON.parse(e)}catch{return n}}function vt(e,n,r,o=2){if(e.length===0)return{d:"",points:[]};let t=Math.min(...e),m=Math.max(...e)-t||1,d=r-o*2,f=e.length===1?0:n/(e.length-1),c=e.map((l,i)=>{let s=e.length===1?n/2:i*f,u=o+d-(l-t)/m*d;return[s,u]});return{d:c.map(([l,i],s)=>`${s===0?"M":"L"}${l.toFixed(2)},${i.toFixed(2)}`).join(" "),points:c}}function ir(e,n,r,o){return[e+r*Math.cos(o),n+r*Math.sin(o)]}function kt(e,n,r,o,t){let[a,m]=ir(e,n,r,o),[d,f]=ir(e,n,r,t),c=t-o>Math.PI?1:0,p=t>o?1:0;return`M${a.toFixed(2)},${m.toFixed(2)} A${r},${r} 0 ${c} ${p} ${d.toFixed(2)},${f.toFixed(2)}`}var Rt=S({scope:()=>{let e=N([]),n=N([]);return{seriesPaths:e,gridLines:n}},mount(e,n){let r=e,o=r.seriesPaths,t=r.gridLines,a=T(()=>{let m=rt(r.series(),[]),d=r.width(),f=r.height(),c=m.flatMap((b)=>b.data),p=c.length?Math.min(...c):0,i=(c.length?Math.max(...c):1)-p||1,s=f-4;t.set(r.grid()?[0.25,0.5,0.75].map((b,h)=>({id:`g${h}`,y:2+s*b})):[]);let u=[];for(let b of m){if(b.data.length<2)continue;let h=b.data.map((L)=>(L-p)/i*100),{d:g,points:y}=vt(h,d,f,2);if(!g)continue;let x=V(b.tone),E=y[0],w=y[y.length-1],A=`${g} L${w[0].toFixed(2)},${f} L${E[0].toFixed(2)},${f} Z`;u.push({name:b.name,areaD:A,strokeD:g,color:x,pts:y.map(([L,C],O)=>({id:`${b.name}-${O}`,cx:L,cy:C}))})}o.set(u)});return()=>a()}});var Pl=z("sui-area-chart",{props:{series:v.string({default:"",reflect:!0}),labels:v.string({default:""}),width:v.number({default:300}),height:v.number({default:200}),points:v.bool({default:!1,reflect:!0}),grid:v.bool({default:!1,reflect:!0}),label:v.string({default:"area chart"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.series=n.series,t.labels=n.labels,t.width=n.width,t.height=n.height,t.points=n.points,t.grid=n.grid,t.label=n.label;let a=null;if(Rt&&typeof Rt.setup==="function"){if(e.scope=t,a=Rt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(b)=>Y({anchor:b,items:()=>t.gridLines(),key:(h,g)=>{let y=Object.create(t);return y.g=h,y.__i=g,((x)=>h.id)(y)},render:(h,g)=>{let y=Object.create(t);return y.g=h,y.__i=g,y.__v=N(0),((x)=>{let E=document.createDocumentFragment(),w=[],A=document.createElement("line");return A.setAttribute("class","grid-line"),A.setAttribute("x1","0"),w.push(k(A,"x2",()=>(x.__v(),x.width()))),w.push(k(A,"y1",()=>(x.__v(),x.g.y))),w.push(k(A,"y2",()=>(x.__v(),x.g.y))),E.appendChild(A),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let L of w)try{L()}catch(C){}},update:(L,C)=>{x.g=L,x.__i=C,x.__v.set((O)=>O+1)}}})(y)}}),d=(b)=>Y({anchor:b,items:()=>t.seriesPaths(),key:(h,g)=>{let y=Object.create(t);return y.s=h,y.__i=g,((x)=>h.name)(y)},render:(h,g)=>{let y=Object.create(t);return y.s=h,y.__i=g,y.__v=N(0),((x)=>{let E=(M)=>Y({anchor:M,items:()=>x.s.pts,key:(H,B)=>{let F=Object.create(x);return F.p=H,F.__i=B,((U)=>H.id)(F)},render:(H,B)=>{let F=Object.create(x);return F.p=H,F.__i=B,F.__v=N(0),((U)=>{let J=document.createDocumentFragment(),Q=[],X=document.createElement("circle");return Q.push(k(X,"cx",()=>(U.__v(),U.p.cx))),Q.push(k(X,"cy",()=>(U.__v(),U.p.cy))),X.setAttribute("r","2.5"),Q.push(k(X,"fill",()=>(U.__v(),U.s.color))),J.appendChild(X),{nodes:Array.from(J.childNodes),cleanup:()=>{for(let ot of Q)try{ot()}catch(K){}},update:(ot,K)=>{U.p=ot,U.__i=K,U.__v.set((_)=>_+1)}}})(F)}}),w=(M)=>G({anchor:M,test:()=>{if(x.points())return 0;return-1},branches:[()=>{let H=document.createDocumentFragment(),B=[],F=document.createComment("for");return H.appendChild(F),B.push(E(F)),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let U of B)try{U()}catch(J){}}}}]}),A=document.createDocumentFragment(),L=[],C=document.createElement("path");L.push(k(C,"d",()=>(x.__v(),x.s.areaD))),L.push(k(C,"fill",()=>(x.__v(),x.s.color))),C.setAttribute("fill-opacity","0.3"),C.setAttribute("stroke","none"),A.appendChild(C);let O=document.createElement("path");L.push(k(O,"d",()=>(x.__v(),x.s.strokeD))),O.setAttribute("fill","none"),L.push(k(O,"stroke",()=>(x.__v(),x.s.color))),O.setAttribute("stroke-width","1.75"),O.setAttribute("stroke-linecap","round"),O.setAttribute("stroke-linejoin","round"),A.appendChild(O);let P=document.createComment("if");return A.appendChild(P),L.push(w(P)),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let M of L)try{M()}catch(H){}},update:(M,H)=>{x.s=M,x.__i=H,x.__v.set((B)=>B+1)}}})(y)}}),f=r.shadowRoot??r;j(f);let c=document.createElement("div");c.setAttribute("class","wrap");let p=document.createElement("svg");o.push(k(p,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),p.setAttribute("preserveaspectratio","none"),p.setAttribute("role","img"),o.push(k(p,"aria-label",()=>t.label()));let l=document.createComment("for");p.appendChild(l),o.push(m(l));let i=document.createComment("for");p.appendChild(i),o.push(d(i)),c.appendChild(p);let s=document.createElement("div");s.setAttribute("class","legend");let u=document.createElement("slot");if(s.appendChild(u),c.appendChild(s),f.appendChild(c),a!=null){let b=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(b){let h=new Proxy(t,{get(g,y){return g[y]},set(g,y,x){let E=g[y];if(E!=null&&typeof E.set==="function"){if(typeof x==="function")E.set(()=>x);else E.set(x);return!0}return!1}});if(b.mount){let g=b.mount(h,r);if(typeof g==="function")o.push(g)}if(b.unmount)o.push(()=>b.unmount(h))}}return t}});var Ut=S({scope:()=>{let e=N([]),n=N(!0);return{bars:e,isEmpty:n}},mount(e,n){let r=e,o=r.bars,t=r.isEmpty,a=T(()=>{let m=ht(r.data());if(m.length===0){o.set([]),t.set(!0);return}let d=Et(r.labels()),f=r.width(),c=r.height(),p=r.max(),l=p>0?p:Math.max(...m,0),i=l<=0?1:l,s=V(r.tone()),u=4,b=3,h=r.orientation()==="horizontal",g=m.length;o.set(m.map((y,x)=>{let E=Math.max(0,y)/i,w,A,L,C;if(h)C=(c-u*(g-1))/g,A=x*(C+u),L=E*f,w=0;else L=(f-u*(g-1))/g,w=x*(L+u),C=E*c,A=c-C;return{id:`b${x}`,x:w,y:A,w:Math.max(0,L),h:Math.max(0,C),rx:b,ry:b,color:s,title:`${d[x]??String(y)}: ${y}`}})),t.set(!1)});return()=>a()}});var Ul=z("sui-bar-chart",{props:{data:v.string({default:"",reflect:!0}),labels:v.string({default:""}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),orientation:v.union(["vertical","horizontal"],{default:"vertical",reflect:!0}),width:v.number({default:300}),height:v.number({default:200}),max:v.number({default:0}),label:v.string({default:"bar chart"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.labels=n.labels,t.tone=n.tone,t.orientation=n.orientation,t.width=n.width,t.height=n.height,t.max=n.max,t.label=n.label;let a=null;if(Ut&&typeof Ut.setup==="function"){if(e.scope=t,a=Ut.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(p)=>Y({anchor:p,items:()=>t.bars(),key:(l,i)=>{let s=Object.create(t);return s.b=l,s.__i=i,((u)=>l.id)(s)},render:(l,i)=>{let s=Object.create(t);return s.b=l,s.__i=i,s.__v=N(0),((u)=>{let b=document.createDocumentFragment(),h=[],g=document.createElement("rect");h.push(k(g,"x",()=>(u.__v(),u.b.x))),h.push(k(g,"y",()=>(u.__v(),u.b.y))),h.push(k(g,"width",()=>(u.__v(),u.b.w))),h.push(k(g,"height",()=>(u.__v(),u.b.h))),h.push(k(g,"rx",()=>(u.__v(),u.b.rx))),h.push(k(g,"ry",()=>(u.__v(),u.b.ry))),h.push(k(g,"fill",()=>(u.__v(),u.b.color)));let y=document.createElement("title"),x=document.createTextNode("");return y.appendChild(x),h.push(R(x,()=>(u.__v(),u.b.title))),g.appendChild(y),b.appendChild(g),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let E of h)try{E()}catch(w){}},update:(E,w)=>{u.b=E,u.__i=w,u.__v.set((A)=>A+1)}}})(s)}}),d=(p)=>G({anchor:p,test:()=>{if(t.isEmpty())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("svg");return i.push(k(s,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),s.setAttribute("aria-hidden","true"),l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let u of i)try{u()}catch(b){}}}},()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("svg");i.push(k(s,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),s.setAttribute("preserveaspectratio","none"),s.setAttribute("role","img"),i.push(k(s,"aria-label",()=>t.label()));let u=document.createComment("for");return s.appendChild(u),i.push(m(u)),l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let b of i)try{b()}catch(h){}}}}]}),f=r.shadowRoot??r;j(f);let c=document.createComment("if");if(f.appendChild(c),o.push(d(c)),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var Yt=S({scope:()=>{let e=N([]),n=N([]);return{seriesPaths:e,gridLines:n}},mount(e,n){let r=e,o=r.seriesPaths,t=r.gridLines,a=T(()=>{let m=rt(r.series(),[]),d=r.width(),f=r.height(),c=m.flatMap((b)=>b.data),p=c.length?Math.min(...c):0,i=(c.length?Math.max(...c):1)-p||1,s=f-4;t.set(r.grid()?[0.25,0.5,0.75].map((b,h)=>({id:`g${h}`,y:2+s*b})):[]);let u=[];for(let b of m){if(b.data.length<2)continue;let h=b.data.map((E)=>(E-p)/i*100),{d:g,points:y}=vt(h,d,f,2);if(!g)continue;let x=V(b.tone);u.push({name:b.name,strokeD:g,color:x,pts:y.map(([E,w],A)=>({id:`${b.name}-${A}`,cx:E,cy:w}))})}o.set(u)});return()=>a()}});var Ql=z("sui-line-chart",{props:{series:v.string({default:"",reflect:!0}),labels:v.string({default:""}),width:v.number({default:300}),height:v.number({default:200}),points:v.bool({default:!1,reflect:!0}),grid:v.bool({default:!1,reflect:!0}),label:v.string({default:"line chart"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.series=n.series,t.labels=n.labels,t.width=n.width,t.height=n.height,t.points=n.points,t.grid=n.grid,t.label=n.label;let a=null;if(Yt&&typeof Yt.setup==="function"){if(e.scope=t,a=Yt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(b)=>Y({anchor:b,items:()=>t.gridLines(),key:(h,g)=>{let y=Object.create(t);return y.g=h,y.__i=g,((x)=>h.id)(y)},render:(h,g)=>{let y=Object.create(t);return y.g=h,y.__i=g,y.__v=N(0),((x)=>{let E=document.createDocumentFragment(),w=[],A=document.createElement("line");return A.setAttribute("class","grid-line"),A.setAttribute("x1","0"),w.push(k(A,"x2",()=>(x.__v(),x.width()))),w.push(k(A,"y1",()=>(x.__v(),x.g.y))),w.push(k(A,"y2",()=>(x.__v(),x.g.y))),E.appendChild(A),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let L of w)try{L()}catch(C){}},update:(L,C)=>{x.g=L,x.__i=C,x.__v.set((O)=>O+1)}}})(y)}}),d=(b)=>Y({anchor:b,items:()=>t.seriesPaths(),key:(h,g)=>{let y=Object.create(t);return y.s=h,y.__i=g,((x)=>h.name)(y)},render:(h,g)=>{let y=Object.create(t);return y.s=h,y.__i=g,y.__v=N(0),((x)=>{let E=(P)=>Y({anchor:P,items:()=>x.s.pts,key:(M,H)=>{let B=Object.create(x);return B.p=M,B.__i=H,((F)=>M.id)(B)},render:(M,H)=>{let B=Object.create(x);return B.p=M,B.__i=H,B.__v=N(0),((F)=>{let U=document.createDocumentFragment(),J=[],Q=document.createElement("circle");return J.push(k(Q,"cx",()=>(F.__v(),F.p.cx))),J.push(k(Q,"cy",()=>(F.__v(),F.p.cy))),Q.setAttribute("r","2.5"),J.push(k(Q,"fill",()=>(F.__v(),F.s.color))),U.appendChild(Q),{nodes:Array.from(U.childNodes),cleanup:()=>{for(let X of J)try{X()}catch(ot){}},update:(X,ot)=>{F.p=X,F.__i=ot,F.__v.set((K)=>K+1)}}})(B)}}),w=(P)=>G({anchor:P,test:()=>{if(x.points())return 0;return-1},branches:[()=>{let M=document.createDocumentFragment(),H=[],B=document.createComment("for");return M.appendChild(B),H.push(E(B)),{nodes:Array.from(M.childNodes),cleanup:()=>{for(let F of H)try{F()}catch(U){}}}}]}),A=document.createDocumentFragment(),L=[],C=document.createElement("path");L.push(k(C,"d",()=>(x.__v(),x.s.strokeD))),C.setAttribute("fill","none"),L.push(k(C,"stroke",()=>(x.__v(),x.s.color))),C.setAttribute("stroke-width","1.75"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),A.appendChild(C);let O=document.createComment("if");return A.appendChild(O),L.push(w(O)),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}},update:(P,M)=>{x.s=P,x.__i=M,x.__v.set((H)=>H+1)}}})(y)}}),f=r.shadowRoot??r;j(f);let c=document.createElement("div");c.setAttribute("class","wrap");let p=document.createElement("svg");o.push(k(p,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),p.setAttribute("preserveaspectratio","none"),p.setAttribute("role","img"),o.push(k(p,"aria-label",()=>t.label()));let l=document.createComment("for");p.appendChild(l),o.push(m(l));let i=document.createComment("for");p.appendChild(i),o.push(d(i)),c.appendChild(p);let s=document.createElement("div");s.setAttribute("class","legend");let u=document.createElement("slot");if(s.appendChild(u),c.appendChild(s),f.appendChild(c),a!=null){let b=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(b){let h=new Proxy(t,{get(g,y){return g[y]},set(g,y,x){let E=g[y];if(E!=null&&typeof E.set==="function"){if(typeof x==="function")E.set(()=>x);else E.set(x);return!0}return!1}});if(b.mount){let g=b.mount(h,r);if(typeof g==="function")o.push(g)}if(b.unmount)o.push(()=>b.unmount(h))}}return t}});var Xt=S({scope:()=>{let e=N(!1),n=N(""),r=N(""),o=N("var(--primary)");return{hasData:e,strokePath:n,areaPath:r,lineColor:o}},mount(e,n){let r=e,o=r.hasData,t=r.strokePath,a=r.areaPath,m=r.lineColor,d=T(()=>{let f=ht(r.data()),c=r.width(),p=r.height(),l=V(r.tone());if(m.set(l),f.length<2){o.set(!1),t.set(""),a.set("");return}let{d:i,points:s}=vt(f,c,p,2);if(t.set(i),r.fill()){let u=s[0],b=s[s.length-1];a.set(`${i} L${b[0].toFixed(2)},${p} L${u[0].toFixed(2)},${p} Z`)}else a.set("");o.set(!0)});return()=>d()}});var nu=z("sui-sparkline",{props:{data:v.string({default:"",reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),width:v.number({default:80}),height:v.number({default:20}),fill:v.bool({default:!1,reflect:!0}),label:v.string({default:"sparkline"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.tone=n.tone,t.width=n.width,t.height=n.height,t.fill=n.fill,t.label=n.label;let a=null;if(Xt&&typeof Xt.setup==="function"){if(e.scope=t,a=Xt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(p)=>G({anchor:p,test:()=>{if(t.fill())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("path");return i.push(k(s,"d",()=>t.areaPath())),i.push(k(s,"fill",()=>t.lineColor())),s.setAttribute("fill-opacity","0.18"),s.setAttribute("stroke","none"),l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let u of i)try{u()}catch(b){}}}}]}),d=(p)=>G({anchor:p,test:()=>{if(t.hasData())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("svg");i.push(k(s,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),i.push(k(s,"width",()=>t.width())),i.push(k(s,"height",()=>t.height())),s.setAttribute("role","img"),i.push(k(s,"aria-label",()=>t.label()));let u=document.createComment("if");s.appendChild(u),i.push(m(u));let b=document.createElement("path");i.push(k(b,"d",()=>t.strokePath())),b.setAttribute("fill","none"),i.push(k(b,"stroke",()=>t.lineColor())),b.setAttribute("stroke-width","1.5"),b.setAttribute("stroke-linecap","round"),b.setAttribute("stroke-linejoin","round");let h=document.createTextNode(`
    `);return b.appendChild(h),s.appendChild(b),l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let g of i)try{g()}catch(y){}}}},()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("svg");i.push(k(s,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),i.push(k(s,"width",()=>t.width())),i.push(k(s,"height",()=>t.height())),s.setAttribute("aria-hidden","true");let u=document.createTextNode(`
  `);return s.appendChild(u),l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let b of i)try{b()}catch(h){}}}}]}),f=r.shadowRoot??r;j(f);let c=document.createComment("if");if(f.appendChild(c),o.push(d(c)),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var qt=S({scope:()=>({segments:N([])}),mount(e,n){let r=e,o=r.segments,t=T(()=>{let a=rt(r.data(),[]),m=a.reduce((u,b)=>u+Math.max(0,b.value),0);if(m<=0){o.set([]);return}let d=r.size(),f=r.thickness(),c=(d-f)/2,p=d/2,l=d/2,i=2*Math.PI,s=-Math.PI/2;o.set(a.filter((u)=>u.value>0).map((u,b)=>{let h=u.value/m*i,g=h>=i-0.001?i-0.001:h,y=kt(p,l,c,s,s+g);return s+=h,{id:`s${b}`,d:y,color:V(u.tone),title:`${u.label}: ${u.value}`}}))});return()=>t()}});var lu=z("sui-donut",{props:{data:v.string({default:"",reflect:!0}),size:v.number({default:120}),thickness:v.number({default:16}),label:v.string({default:"donut chart"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.size=n.size,t.thickness=n.thickness,t.label=n.label;let a=null;if(qt&&typeof qt.setup==="function"){if(e.scope=t,a=qt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(u)=>Y({anchor:u,items:()=>t.segments(),key:(b,h)=>{let g=Object.create(t);return g.seg=b,g.__i=h,((y)=>b.id)(g)},render:(b,h)=>{let g=Object.create(t);return g.seg=b,g.__i=h,g.__v=N(0),((y)=>{let x=document.createDocumentFragment(),E=[],w=document.createElement("path");w.setAttribute("part","segment"),E.push(k(w,"d",()=>(y.__v(),y.seg.d))),w.setAttribute("fill","none"),E.push(k(w,"stroke",()=>(y.__v(),y.seg.color))),E.push(k(w,"stroke-width",()=>(y.__v(),y.thickness()))),w.setAttribute("stroke-linecap","butt");let A=document.createElement("title"),L=document.createTextNode("");return A.appendChild(L),E.push(R(L,()=>(y.__v(),y.seg.title))),w.appendChild(A),x.appendChild(w),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let C of E)try{C()}catch(O){}},update:(C,O)=>{y.seg=C,y.__i=O,y.__v.set((P)=>P+1)}}})(g)}}),d=r.shadowRoot??r;j(d);let f=document.createElement("svg");o.push(k(f,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),f.setAttribute("role","img"),o.push(k(f,"aria-label",()=>t.label())),o.push(k(f,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let c=document.createElement("circle");c.setAttribute("class","track"),c.setAttribute("part","track"),o.push(k(c,"cx",()=>t.size()/2)),o.push(k(c,"cy",()=>t.size()/2)),o.push(k(c,"r",()=>(t.size()-t.thickness())/2)),c.setAttribute("fill","none"),o.push(k(c,"stroke-width",()=>t.thickness()));let p=document.createTextNode(`
  `);c.appendChild(p),f.appendChild(c);let l=document.createComment("for");f.appendChild(l),o.push(m(l)),d.appendChild(f);let i=document.createElement("div");i.setAttribute("class","center");let s=document.createElement("slot");if(i.appendChild(s),d.appendChild(i),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let b=new Proxy(t,{get(h,g){return h[g]},set(h,g,y){let x=h[g];if(x!=null&&typeof x.set==="function"){if(typeof y==="function")x.set(()=>y);else x.set(y);return!0}return!1}});if(u.mount){let h=u.mount(b,r);if(typeof h==="function")o.push(h)}if(u.unmount)o.push(()=>u.unmount(b))}}return t}});var lr=Math.PI/180,Jt=135*lr,sr=405*lr,Wt=S({scope:()=>({trackD:N(""),valueD:N(""),arcColor:N("var(--primary)")}),mount(e,n){let r=e,o=r.trackD,t=r.valueD,a=r.arcColor,m=T(()=>{let d=r.size(),f=r.thickness(),c=(d-f)/2-2,p=d/2,l=d/2;o.set(kt(p,l,c,Jt,sr));let i=r.value(),s=r.min(),u=r.max(),b=Math.max(0,Math.min(1,(i-s)/(u-s||1))),h=Jt+(sr-Jt)*b;t.set(b>0?kt(p,l,c,Jt,h):"");let g=r.tone(),y=r.lowThreshold(),x=r.highThreshold(),E=g;if(!E)if(y>0&&i<y)E="destructive";else if(x>0&&i<x)E="warning";else if(y>0||x>0)E="success";else E="primary";a.set(V(E))});return()=>m()}});var bu=z("sui-gauge",{props:{value:v.number({default:0,reflect:!0}),min:v.number({default:0}),max:v.number({default:100}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"",reflect:!0}),"low-threshold":v.number({default:0}),"high-threshold":v.number({default:0}),size:v.number({default:140}),thickness:v.number({default:12}),label:v.string({default:"gauge"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.min=n.min,t.max=n.max,t.tone=n.tone,t.lowThreshold=n.lowThreshold,t.highThreshold=n.highThreshold,t.size=n.size,t.thickness=n.thickness,t.label=n.label;let a=null;if(Wt&&typeof Wt.setup==="function"){if(e.scope=t,a=Wt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(u)=>G({anchor:u,test:()=>{if(t.valueD())return 0;return-1},branches:[()=>{let b=document.createDocumentFragment(),h=[],g=document.createElement("path");g.setAttribute("part","fill"),h.push(k(g,"d",()=>t.valueD())),g.setAttribute("fill","none"),h.push(k(g,"stroke",()=>t.arcColor())),h.push(k(g,"stroke-width",()=>t.thickness())),g.setAttribute("stroke-linecap","round");let y=document.createTextNode(`
    `);return g.appendChild(y),b.appendChild(g),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let x of h)try{x()}catch(E){}}}}]}),d=r.shadowRoot??r;j(d);let f=document.createElement("svg");o.push(k(f,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),f.setAttribute("role","img"),o.push(k(f,"aria-label",()=>t.label())),o.push(k(f,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let c=document.createElement("path");c.setAttribute("class","track"),c.setAttribute("part","track"),o.push(k(c,"d",()=>t.trackD())),c.setAttribute("fill","none"),o.push(k(c,"stroke-width",()=>t.thickness())),c.setAttribute("stroke-linecap","round");let p=document.createTextNode(`
  `);c.appendChild(p),f.appendChild(c);let l=document.createComment("if");f.appendChild(l),o.push(m(l)),d.appendChild(f);let i=document.createElement("div");i.setAttribute("class","gauge-label");let s=document.createElement("slot");if(i.appendChild(s),d.appendChild(i),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let b=new Proxy(t,{get(h,g){return h[g]},set(h,g,y){let x=h[g];if(x!=null&&typeof x.set==="function"){if(typeof y==="function")x.set(()=>y);else x.set(y);return!0}return!1}});if(u.mount){let h=u.mount(b,r);if(typeof h==="function")o.push(h)}if(u.unmount)o.push(()=>u.unmount(b))}}return t}});var Zt=S({scope:()=>({bars:N([])}),mount(e,n){let r=e,o=r.bars,t=T(()=>{let a=rt(r.segments(),[]),m=a.reduce((p,l)=>p+Math.max(0,l.count),0),d=r.total(),f=d>0?d:m,c=f<=0?1:f;o.set(a.filter((p)=>p.count>0).map((p,l)=>{let i=Math.max(0,p.count)/c*100;return{id:`b${l}`,style:`width:${i.toFixed(3)}%;background:${V(p.tone)};`,title:`${p.label??p.tone}: ${p.count}`}}))});return()=>t()}});var Au=z("sui-meter",{props:{segments:v.string({default:"",reflect:!0}),total:v.number({default:0}),label:v.string({default:"meter"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.segments=n.segments,t.total=n.total,t.label=n.label;let a=null;if(Zt&&typeof Zt.setup==="function"){if(e.scope=t,a=Zt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(s)=>Y({anchor:s,items:()=>t.bars(),key:(u,b)=>{let h=Object.create(t);return h.b=u,h.__i=b,((g)=>u.id)(h)},render:(u,b)=>{let h=Object.create(t);return h.b=u,h.__i=b,h.__v=N(0),((g)=>{let y=document.createDocumentFragment(),x=[],E=document.createElement("div");return E.setAttribute("class","seg"),E.setAttribute("part","segment"),x.push(k(E,"style",()=>(g.__v(),g.b.style))),x.push(k(E,"title",()=>(g.__v(),g.b.title))),y.appendChild(E),{nodes:Array.from(y.childNodes),cleanup:()=>{for(let w of x)try{w()}catch(A){}},update:(w,A)=>{g.b=w,g.__i=A,g.__v.set((L)=>L+1)}}})(h)}}),d=r.shadowRoot??r;j(d);let f=document.createElement("div");f.setAttribute("class","wrap");let c=document.createElement("div");c.setAttribute("class","bar"),c.setAttribute("part","bar"),c.setAttribute("role","img"),o.push(k(c,"aria-label",()=>t.label()));let p=document.createComment("for");c.appendChild(p),o.push(m(p)),f.appendChild(c);let l=document.createElement("div");l.setAttribute("class","legend");let i=document.createElement("slot");if(l.appendChild(i),f.appendChild(l),d.appendChild(f),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,g){let y=b[h];if(y!=null&&typeof y.set==="function"){if(typeof g==="function")y.set(()=>g);else y.set(g);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var Qt=S({scope:()=>({cells:N([]),svgW:N(0),svgH:N(0)}),mount(e,n){let r=e,o=r.cells,t=r.svgW,a=r.svgH,m=T(()=>{let d=Et(r.rows()),f=Et(r.cols()),c=r.cellSize(),p=r.gap(),l=r.data(),i=rt(l,[]),s=Array.isArray(i)&&i.length>0&&Array.isArray(i[0]),u;if(s)u=i;else{let C=ht(l),O=f.length||1,P=d.length||(C.length>0?Math.ceil(C.length/O):0);u=Array.from({length:P},(M,H)=>C.slice(H*O,(H+1)*O))}let b=u.length,h=b>0?Math.max(...u.map((C)=>C.length)):0,g=u.flatMap((C)=>C.filter((O)=>typeof O==="number")),y=g.length?Math.min(...g):0,E=(g.length?Math.max(...g):1)-y,w=V(r.tone()),A=c+p;t.set(h>0?h*c+(h-1)*p:0),a.set(b>0?b*c+(b-1)*p:0);let L=[];for(let C=0;C<b;C++){let O=u[C]??[];for(let P=0;P<h;P++){let M=O[P]??0,H=E===0?y>0?1:0:(M-y)/E,B=(H*100).toFixed(1),F=H>0?`fill:color-mix(in oklch,${w} ${B}%,transparent)`:"";L.push({id:`c${C}-${P}:${F}`,x:P*A,y:C*A,w:c,h:c,fill:F})}}o.set(L)});return()=>m()}});var Tu=z("sui-heatmap",{props:{data:v.string({default:"",reflect:!0}),rows:v.string({default:""}),cols:v.string({default:""}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),"cell-size":v.number({default:20}),gap:v.number({default:3}),label:v.string({default:"heatmap"})},shadow:!0,styles:`
  :host {
    display: inline-block;
    font-family: var(--font-body);
  }
  svg { display: block; overflow: visible; }
  .cell { fill: var(--muted); }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.rows=n.rows,t.cols=n.cols,t.tone=n.tone,t.cellSize=n.cellSize,t.gap=n.gap,t.label=n.label;let a=null;if(Qt&&typeof Qt.setup==="function"){if(e.scope=t,a=Qt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(p)=>Y({anchor:p,items:()=>t.cells(),key:(l,i)=>{let s=Object.create(t);return s.cell=l,s.__i=i,((u)=>l.id)(s)},render:(l,i)=>{let s=Object.create(t);return s.cell=l,s.__i=i,s.__v=N(0),((u)=>{let b=document.createDocumentFragment(),h=[],g=document.createElement("rect");g.setAttribute("class","cell"),g.setAttribute("part","cell"),h.push(k(g,"x",()=>(u.__v(),u.cell.x))),h.push(k(g,"y",()=>(u.__v(),u.cell.y))),h.push(k(g,"width",()=>(u.__v(),u.cell.w))),h.push(k(g,"height",()=>(u.__v(),u.cell.h))),g.setAttribute("rx","2"),h.push(k(g,"style",()=>(u.__v(),u.cell.fill)));let y=document.createTextNode(`
    `);return g.appendChild(y),b.appendChild(g),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let x of h)try{x()}catch(E){}},update:(x,E)=>{u.cell=x,u.__i=E,u.__v.set((w)=>w+1)}}})(s)}}),d=r.shadowRoot??r;j(d);let f=document.createElement("svg");o.push(k(f,"viewbox",()=>"0 0 "+t.svgW()+" "+t.svgH())),f.setAttribute("role","img"),o.push(k(f,"aria-label",()=>t.label()));let c=document.createComment("for");if(f.appendChild(c),o.push(m(c)),d.appendChild(f),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var Su=z("sui-button",{props:{tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:v.union(["solid","soft","outline","ghost","default"],{default:"default",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),type:v.union(["button","submit","reset"],{default:"button",reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),loading:v.bool({default:!1,reflect:!0}),label:v.string({default:""}),pressed:v.string({default:"",reflect:!0}),expanded:v.string({default:"",reflect:!0})},shadow:!0,styles:`
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
  `,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.type=n.type,t.disabled=n.disabled,t.loading=n.loading,t.label=n.label,t.pressed=n.pressed,t.expanded=n.expanded;let a=(s)=>G({anchor:s,test:()=>{if(t.loading())return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("span");h.setAttribute("class","spinner-wrap"),h.setAttribute("aria-hidden","true");let g=document.createElement("span");return g.setAttribute("class","spinner"),h.appendChild(g),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let y of b)try{y()}catch(x){}}}}]}),m=r.shadowRoot??r;j(m);let d=document.createElement("button");d.setAttribute("part","button"),o.push(k(d,"type",()=>t.type())),o.push(k(d,"disabled",()=>t.disabled()||t.loading())),o.push(k(d,"aria-busy",()=>t.loading()?"true":"false")),o.push(k(d,"aria-disabled",()=>t.disabled()||t.loading()?"true":"false")),o.push(k(d,"aria-label",()=>t.label())),o.push(k(d,"aria-pressed",()=>t.pressed())),o.push(k(d,"aria-expanded",()=>t.expanded()));let f=document.createElement("span");f.setAttribute("class","content");let c=document.createElement("slot");c.setAttribute("name","leading"),f.appendChild(c);let p=document.createElement("slot");o.push(tt(p,r,"data-icon-only")),f.appendChild(p);let l=document.createElement("slot");l.setAttribute("name","trailing"),f.appendChild(l),d.appendChild(f);let i=document.createComment("if");return d.appendChild(i),o.push(a(i)),m.appendChild(d),t}});var It=S({mount(e,n){let r=e,o=N(0),t=()=>{let i=r.tabSlot();if(!i)return Array.from(n.querySelectorAll(":scope > sui-tab"));return i.assignedElements().filter((s)=>s.tagName.toLowerCase()==="sui-tab")},a=(i)=>{if(r.active()===i)return;r.active.set(i),n.dispatchEvent(new CustomEvent("sui-tab-change",{detail:{name:i},bubbles:!0,composed:!0}))},m=T(()=>{o();let i=r.active(),s=t(),u=i;if(!u&&s.length>0){if(u=s[0].getAttribute("name")??"",u)r.active.set(u)}for(let b of s){let h=b.getAttribute("name")??"";if(h===u)b.setAttribute("active","");else b.removeAttribute("active");b.id=`sui-tab-${h}`,b.setAttribute("aria-controls",`sui-tabpanel-${h}`)}for(let b of Array.from(n.querySelectorAll(':scope > sui-tab-panel, :scope > [role="tabpanel"]')))if((b.getAttribute("name")??"")===u)b.removeAttribute("hidden"),b.setAttribute("active","");else b.setAttribute("hidden",""),b.removeAttribute("active")}),d=(i)=>{let s=r.orientation()==="vertical",u=s?"ArrowDown":"ArrowRight",b=s?"ArrowUp":"ArrowLeft",h=[u,b,"Home","End"],g=["Enter"," "];if(!h.includes(i.key)&&!g.includes(i.key))return;i.preventDefault();let y=t().filter((C)=>!C.hasAttribute("disabled"));if(!y.length)return;let x=y.map((C)=>C.getAttribute("name")??""),E=r.active(),w=Math.max(0,x.indexOf(E)),A=w;if(i.key===u)A=(w+1)%x.length;else if(i.key===b)A=(w-1+x.length)%x.length;else if(i.key==="Home")A=0;else if(i.key==="End")A=x.length-1;let L=x[A];a(L),queueMicrotask(()=>{let C=y[A],O=C.shadowRoot?.querySelector("button");O?O.focus():C.focus()})},f=(i)=>{let s=i.target.closest("sui-tab");if(!s||s.hasAttribute("disabled"))return;let u=s.getAttribute("name")??"";if(u)a(u)},c=()=>o.set(o()+1),p=r.strip(),l=r.tabSlot();if(p)p.addEventListener("keydown",d),p.addEventListener("click",f);if(l)l.addEventListener("slotchange",c);return()=>{if(m(),p)p.removeEventListener("keydown",d),p.removeEventListener("click",f);if(l)l.removeEventListener("slotchange",c)}}});var Bu=z("sui-tabs",{props:{active:v.string({default:"",reflect:!0}),variant:v.union(["underline","pill","enclosed"],{default:"underline",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),sticky:v.bool({default:!1,reflect:!0}),orientation:v.union(["horizontal","vertical"],{default:"horizontal",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.active=n.active,t.variant=n.variant,t.size=n.size,t.tone=n.tone,t.sticky=n.sticky,t.orientation=n.orientation;let a=null;if(It&&typeof It.setup==="function"){if(e.scope=t,a=It.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","strip"),d.setAttribute("part","strip"),d.setAttribute("role","tablist"),o.push(k(d,"aria-orientation",()=>t.orientation())),D(t,"strip",d);let f=document.createElement("slot");if(D(t,"tabSlot",f),d.appendChild(f),m.appendChild(d),a!=null){let c=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(c){let p=new Proxy(t,{get(l,i){return l[i]},set(l,i,s){let u=l[i];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(c.mount){let l=c.mount(p,r);if(typeof l==="function")o.push(l)}if(c.unmount)o.push(()=>c.unmount(p))}}return t}});var Vt=S({mount(e,n){let r=e,o=T(()=>{let t=r.btn();if(!t)return;t.tabIndex=r.active()?0:-1});return()=>{o()}}});var qu=z("sui-tab",{props:{name:v.string({default:"",reflect:!0}),label:v.string({default:"",reflect:!0}),count:v.string({default:"",reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),active:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.name=n.name,t.label=n.label,t.count=n.count,t.disabled=n.disabled,t.active=n.active;let a=null;if(Vt&&typeof Vt.setup==="function"){if(e.scope=t,a=Vt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(s)=>G({anchor:s,test:()=>{if(t.count()!=="")return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("span");h.setAttribute("class","count"),h.setAttribute("aria-hidden","true");let g=document.createTextNode("");return h.appendChild(g),b.push(R(g,()=>t.count())),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let y of b)try{y()}catch(x){}}}}]}),d=r.shadowRoot??r;j(d);let f=document.createElement("button");f.setAttribute("part","button"),f.setAttribute("type","button"),f.setAttribute("role","tab"),o.push(k(f,"data-name",()=>t.name())),o.push(k(f,"aria-selected",()=>t.active()?"true":"false")),o.push(k(f,"aria-disabled",()=>t.disabled()?"true":"false")),o.push(k(f,"disabled",()=>t.disabled())),D(t,"btn",f);let c=document.createElement("span");c.setAttribute("class","leading"),c.setAttribute("part","leading");let p=document.createElement("slot");p.setAttribute("name","leading"),c.appendChild(p),f.appendChild(c);let l=document.createElement("slot");f.appendChild(l);let i=document.createComment("if");if(f.appendChild(i),o.push(m(i)),d.appendChild(f),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,g){let y=b[h];if(y!=null&&typeof y.set==="function"){if(typeof g==="function")y.set(()=>g);else y.set(g);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var Zu=z("sui-empty-state",{props:{variant:v.union(["default","inline","placeholder","connecting"],{default:"default",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.variant=n.variant,t.size=n.size,t.tone=n.tone;let a=r.shadowRoot??r;j(a);let m=document.createElement("div");m.setAttribute("class","box"),m.setAttribute("part","box"),o.push(k(m,"role",()=>t.variant()==="connecting"?"status":"note")),o.push(k(m,"aria-live",()=>t.variant()==="connecting"?"polite":"off"));let d=document.createElement("div");d.setAttribute("class","icon"),d.setAttribute("part","icon"),d.setAttribute("aria-hidden","true");let f=document.createElement("slot");f.setAttribute("name","icon"),o.push(tt(f,r,"data-no-icon")),d.appendChild(f),m.appendChild(d);let c=document.createElement("div");c.setAttribute("class","copy");let p=document.createElement("div");p.setAttribute("class","title"),p.setAttribute("part","title");let l=document.createElement("slot");l.setAttribute("name","title"),p.appendChild(l),c.appendChild(p);let i=document.createElement("div");i.setAttribute("class","desc"),i.setAttribute("part","desc");let s=document.createElement("slot");i.appendChild(s),c.appendChild(i),m.appendChild(c);let u=document.createElement("div");u.setAttribute("class","actions"),u.setAttribute("part","actions");let b=document.createElement("slot");return b.setAttribute("name","action"),u.appendChild(b),m.appendChild(u),a.appendChild(m),t}});var go=0,Kt=S({mount(e,n){let r=e,o=r.inputEl();if(!o)return;n.expose({focus:()=>o.focus(),blur:()=>o.blur(),clear:()=>{o.value="",r.value.set("")},reportValidity:()=>o.reportValidity()}),Z(o,"input",()=>n.emit("sui-input",{value:o.value})),Z(o,"change",()=>n.emit("sui-input-change",{value:o.value}));let t=`sui-input-err-${++go}`;it(k(o,"aria-describedby",()=>{let a=[];if(r.invalid()&&r.errorMessage()){let d=r.errorEl,f=typeof d==="function"?d():null;if(f)f.id=t,a.push(t)}let m=r.describedBy();if(m)a.push(m);return a.length>0?a.join(" "):null}))}});var _u=z("sui-input",{props:{type:v.union(["text","search","email","url","tel","password","number"],{default:"text",reflect:!0}),value:v.string({default:""}),placeholder:v.string({default:""}),name:v.string({default:""}),autocomplete:v.string({default:""}),required:v.bool({default:!1,reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),readonly:v.bool({default:!1,reflect:!0}),invalid:v.bool({default:!1,reflect:!0}),loading:v.bool({default:!1,reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:v.union(["default","ghost"],{default:"default",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),label:v.string({default:""}),"described-by":v.string({default:""}),"error-message":v.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.type=n.type,t.value=n.value,t.placeholder=n.placeholder,t.name=n.name,t.autocomplete=n.autocomplete,t.required=n.required,t.disabled=n.disabled,t.readonly=n.readonly,t.invalid=n.invalid,t.loading=n.loading,t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.label=n.label,t.describedBy=n.describedBy,t.errorMessage=n.errorMessage;let a=null;if(Kt&&typeof Kt.setup==="function"){if(e.scope=t,a=Kt.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(u)=>G({anchor:u,test:()=>{if(t.loading())return 0;return 1},branches:[()=>{let b=document.createDocumentFragment(),h=[],g=document.createElement("span");return g.setAttribute("class","spinner"),g.setAttribute("aria-hidden","true"),b.appendChild(g),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let y of h)try{y()}catch(x){}}}},()=>{let b=document.createDocumentFragment(),h=[],g=document.createElement("slot");return g.setAttribute("name","trailing"),h.push(tt(g,r,"data-no-trailing")),b.appendChild(g),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let y of h)try{y()}catch(x){}}}}]}),d=(u)=>G({anchor:u,test:()=>{if(t.invalid()&&t.errorMessage())return 0;return-1},branches:[()=>{let b=document.createDocumentFragment(),h=[],g=document.createElement("span");g.setAttribute("class","error"),g.setAttribute("role","alert"),D(t,"errorEl",g);let y=document.createTextNode("");return g.appendChild(y),h.push(R(y,()=>t.errorMessage())),b.appendChild(g),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let x of h)try{x()}catch(E){}}}}]}),f=r.shadowRoot??r;j(f);let c=document.createElement("div");c.setAttribute("class","wrap"),c.setAttribute("part","wrap");let p=document.createElement("slot");p.setAttribute("name","leading"),o.push(tt(p,r,"data-no-leading")),c.appendChild(p);let l=document.createElement("input");l.setAttribute("part","input"),o.push(k(l,"type",()=>t.type())),o.push(k(l,"placeholder",()=>t.placeholder())),o.push(k(l,"name",()=>t.name())),o.push(k(l,"autocomplete",()=>t.autocomplete()||"off")),o.push(k(l,"disabled",()=>t.disabled())),o.push(k(l,"readonly",()=>t.readonly())),o.push(k(l,"required",()=>t.required())),o.push(k(l,"aria-label",()=>t.label())),o.push(k(l,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(k(l,"aria-required",()=>t.required()?"true":"false")),o.push(k(l,"aria-busy",()=>t.loading()?"true":"false")),o.push(ut(l,t.value)),D(t,"inputEl",l),c.appendChild(l);let i=document.createComment("if");c.appendChild(i),o.push(m(i)),f.appendChild(c);let s=document.createComment("if");if(f.appendChild(s),o.push(d(s)),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let b=new Proxy(t,{get(h,g){return h[g]},set(h,g,y){let x=h[g];if(x!=null&&typeof x.set==="function"){if(typeof y==="function")x.set(()=>y);else x.set(y);return!0}return!1}});if(u.mount){let h=u.mount(b,r);if(typeof h==="function")o.push(h)}if(u.unmount)o.push(()=>u.unmount(b))}}return t}});var yo={git:"git-branch",refresh:"refresh-cw",stop:"stop-circle",warning:"triangle-alert"};var st=null,Ct=null;function $t(){return st}function ur(){if(st)return Promise.resolve(st);if(Ct)return Ct;return Ct=fetch("/_sui/lucide-icons.json").then((e)=>{if(!e.ok)throw Error(`lucide registry fetch ${e.status}`);return e.json()}).then((e)=>{return st=e,e}).catch((e)=>{throw Ct=null,e}),Ct}function dr(e){if(!e||!st)return;if(st[e])return e;let n=yo[e];if(n&&st[n])return n;return}function cr(e){return st?st[e]:void 0}var pr="http://www.w3.org/2000/svg",De=N($t()?1:0),Oe=!1;function xo(){if(Oe||$t())return;Oe=!0,ur().then(()=>De.set(De()+1)).catch(()=>{Oe=!1})}function wo(e){let n=e.createElementNS(pr,"svg");return n.setAttribute("part","svg"),n.setAttribute("class","sui-icon-svg"),n.setAttribute("viewBox","0 0 24 24"),n.setAttribute("fill","none"),n.setAttribute("stroke","currentColor"),n.setAttribute("stroke-width","2"),n.setAttribute("stroke-linecap","round"),n.setAttribute("stroke-linejoin","round"),n}function Ao(e,n){while(e.firstChild)e.removeChild(e.firstChild);let r=new DOMParser().parseFromString(`<svg xmlns="${pr}">${n}</svg>`,"image/svg+xml").documentElement;while(r.firstChild)e.appendChild(e.ownerDocument.adoptNode(r.firstChild))}var _t=S({mount(e,n){let r=e,o=n.shadowRoot,t=null,a=null,m=T(()=>{let f=r.label();if(f)n.setAttribute("role","img"),n.setAttribute("aria-label",f),n.removeAttribute("aria-hidden");else n.setAttribute("aria-hidden","true"),n.removeAttribute("role"),n.removeAttribute("aria-label")}),d=T(()=>{De();let f=r.name(),c=dr(f)??null,p=r.label();if(!c){if(t)t.remove(),t=null;if(a=null,f&&!$t())xo();return}if(!t)t=wo(n.ownerDocument),o.insertBefore(t,o.firstChild);if(c!==a)Ao(t,cr(c)),a=c;if(p)t.removeAttribute("aria-hidden");else t.setAttribute("aria-hidden","true")});return()=>{m(),d()}}});var sd=z("sui-icon",{props:{name:v.string({default:"",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:v.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),label:v.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.name=n.name,t.size=n.size,t.tone=n.tone,t.label=n.label;let a=null;if(_t&&typeof _t.setup==="function"){if(e.scope=t,a=_t.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");if(m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});var tn=S({mount(e,n){let r=e,o=()=>Array.from(n.querySelectorAll(":scope > option, :scope > optgroup > option")),t=()=>{let s=r.value(),u=o();if(u.length===0)return s;let b=u.find((g)=>g.value===s)??u.find((g)=>g.hasAttribute("selected"))??u[0];return(b?.textContent??"").trim()||b?.value||s},a=()=>{let s=r.selectEl();if(!s){let b=o(),h=b.find((g)=>g.value===r.value())??b.find((g)=>g.hasAttribute("selected"))??b[0];if(!h||h.value==="")n.setAttribute("data-placeholder","");else n.removeAttribute("data-placeholder");return}let u=s.selectedOptions?.[0]??s.options[s.selectedIndex];if(!u||u.value==="")n.setAttribute("data-placeholder","");else n.removeAttribute("data-placeholder")},m=()=>{let s=r.selectEl();if(!s)return;while(s.firstChild)s.removeChild(s.firstChild);let u=document.createDocumentFragment(),b=n.querySelectorAll(":scope > option, :scope > optgroup");for(let h of Array.from(b))u.appendChild(h.cloneNode(!0));s.appendChild(u),s.value=r.value()},d=()=>{let s=r.valueLabel();if(s)s.textContent=t()},f=T(()=>{let s=r.value(),u=r.selectEl();if(u)u.value=s;d(),a()}),c=(s)=>{let u=s.target.value;r.value.set(u),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:u},bubbles:!0,composed:!0}))},p=r.selectEl();if(p)p.addEventListener("change",c);let l=r.optionsSlot(),i=()=>{m(),d(),a()};if(l)l.addEventListener("slotchange",i);return m(),d(),a(),()=>{if(p)p.removeEventListener("change",c);if(l)l.removeEventListener("slotchange",i);f()}}});var fd=z("sui-select",{props:{value:v.string({default:""}),name:v.string({default:""}),required:v.bool({default:!1,reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),invalid:v.bool({default:!1,reflect:!0}),tone:v.union(["default","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),label:v.string({default:""}),"described-by":v.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.name=n.name,t.required=n.required,t.disabled=n.disabled,t.invalid=n.invalid,t.tone=n.tone,t.size=n.size,t.label=n.label,t.describedBy=n.describedBy;let a=null;if(tn&&typeof tn.setup==="function"){if(e.scope=t,a=tn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","field"),d.setAttribute("part","field");let f=document.createElement("span");f.setAttribute("class","value"),f.setAttribute("aria-hidden","true"),D(t,"valueLabel",f),d.appendChild(f);let c=document.createElement("svg");c.setAttribute("class","chevron"),c.setAttribute("part","chevron"),c.setAttribute("viewbox","0 0 16 16"),c.setAttribute("aria-hidden","true"),c.setAttribute("focusable","false"),c.setAttribute("fill","none"),c.setAttribute("stroke","currentColor"),c.setAttribute("stroke-width","1.75"),c.setAttribute("stroke-linecap","round"),c.setAttribute("stroke-linejoin","round");let p=document.createElement("path");p.setAttribute("d","M4 6l4 4 4-4"),c.appendChild(p),d.appendChild(c);let l=document.createElement("select");l.setAttribute("part","select"),o.push(k(l,"name",()=>t.name())),o.push(k(l,"disabled",()=>t.disabled())),o.push(k(l,"required",()=>t.required())),o.push(k(l,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(k(l,"aria-label",()=>t.label())),o.push(k(l,"aria-describedby",()=>t.describedBy())),D(t,"selectEl",l),d.appendChild(l),m.appendChild(d);let i=document.createElement("span");i.setAttribute("hidden","");let s=document.createElement("slot");if(D(t,"optionsSlot",s),i.appendChild(s),m.appendChild(i),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let b=new Proxy(t,{get(h,g){return h[g]},set(h,g,y){let x=h[g];if(x!=null&&typeof x.set==="function"){if(typeof y==="function")x.set(()=>y);else x.set(y);return!0}return!1}});if(u.mount){let h=u.mount(b,r);if(typeof h==="function")o.push(h)}if(u.unmount)o.push(()=>u.unmount(b))}}return t}});var hd=z("sui-separator",{props:{orientation:v.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:v.union(["solid","dashed"],{default:"solid",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.orientation=n.orientation,t.variant=n.variant;let a=r.shadowRoot??r;j(a);let m=document.createElement("hr");return m.setAttribute("class","sep"),m.setAttribute("part","sep"),m.setAttribute("role","separator"),o.push(k(m,"aria-orientation",()=>t.orientation())),a.appendChild(m),t}});var nn=S({mount(e,n){let r=e,o=T(()=>{let t=r.labelEl();if(!t)return;t.htmlFor=r.for?.()??""});return()=>{o()}}});var Ad=z("sui-label",{props:{for:v.string({default:""}),size:v.union(["sm","md"],{default:"sm",reflect:!0}),required:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.for=n.for,t.size=n.size,t.required=n.required;let a=null;if(nn&&typeof nn.setup==="function"){if(e.scope=t,a=nn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(l)=>G({anchor:l,test:()=>{if(t.required())return 0;return-1},branches:[()=>{let i=document.createDocumentFragment(),s=[],u=document.createElement("span");u.setAttribute("class","req"),u.setAttribute("aria-hidden","true");let b=document.createTextNode("*");return u.appendChild(b),i.appendChild(u),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let h of s)try{h()}catch(g){}}}}]}),d=r.shadowRoot??r;j(d);let f=document.createElement("label");f.setAttribute("part","label"),D(t,"labelEl",f);let c=document.createElement("slot");f.appendChild(c);let p=document.createComment("if");if(f.appendChild(p),o.push(m(p)),d.appendChild(f),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(l.mount){let s=l.mount(i,r);if(typeof s==="function")o.push(s)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var en=S({mount(e,n){let r=e,o=T(()=>{let t=r.closeBtn();if(!t)return;let a=()=>{if(!r.dismissible())return;n.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),n.remove()};return t.addEventListener("click",a),()=>t.removeEventListener("click",a)});return()=>{o()}}});var jd=z("sui-alert",{props:{tone:v.union(["info","success","warning","destructive","muted"],{default:"info",reflect:!0}),dismissible:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.dismissible=n.dismissible;let a=null;if(en&&typeof en.setup==="function"){if(e.scope=t,a=en.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","box"),d.setAttribute("part","box"),o.push(k(d,"role",()=>t.tone()==="destructive"||t.tone()==="warning"?"alert":"status")),o.push(k(d,"aria-live",()=>t.tone()==="destructive"||t.tone()==="warning"?"assertive":"polite")),d.setAttribute("aria-atomic","true");let f=document.createElement("span"),c=document.createElement("slot");c.setAttribute("name","icon"),f.appendChild(c),d.appendChild(f);let p=document.createElement("div");p.setAttribute("class","content");let l=document.createElement("div");l.setAttribute("class","title");let i=document.createElement("slot");i.setAttribute("name","title"),l.appendChild(i),p.appendChild(l);let s=document.createElement("div");s.setAttribute("class","body");let u=document.createElement("slot");s.appendChild(u),p.appendChild(s);let b=document.createElement("div");b.setAttribute("class","actions");let h=document.createElement("slot");h.setAttribute("name","actions"),b.appendChild(h),p.appendChild(b),d.appendChild(p);let g=document.createElement("button");g.setAttribute("class","close"),g.setAttribute("part","close"),g.setAttribute("aria-label","Dismiss"),D(t,"closeBtn",g);let y=document.createTextNode("×");if(g.appendChild(y),d.appendChild(g),m.appendChild(d),a!=null){let x=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(x){let E=new Proxy(t,{get(w,A){return w[A]},set(w,A,L){let C=w[A];if(C!=null&&typeof C.set==="function"){if(typeof L==="function")C.set(()=>L);else C.set(L);return!0}return!1}});if(x.mount){let w=x.mount(E,r);if(typeof w==="function")o.push(w)}if(x.unmount)o.push(()=>x.unmount(E))}}return t}});var Eo=0,rn=S({mount(e,n){let r=e,o=`sui-tooltip-${++Eo}`,t=T(()=>{let f=r.bubbleEl();if(f)f.id=o}),a=()=>{let f=r.defaultSlot();if(!f)return;let c=f.assignedElements({flatten:!0})[0];if(!c)return;let p=c.getAttribute("aria-describedby")??"";if(!p.split(/\s+/).includes(o))c.setAttribute("aria-describedby",p?`${p} ${o}`:o)},m=T(()=>{let f=r.defaultSlot();if(!f)return;return f.addEventListener("slotchange",a),a(),()=>f.removeEventListener("slotchange",a)}),d=(f)=>{if(f.key==="Escape")n.blur()};return n.addEventListener("keydown",d),()=>{t(),m(),n.removeEventListener("keydown",d)}}});var Pd=z("sui-tooltip",{props:{placement:v.union(["top","bottom","left","right"],{default:"top",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.placement=n.placement;let a=null;if(rn&&typeof rn.setup==="function"){if(e.scope=t,a=rn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");D(t,"defaultSlot",d),m.appendChild(d);let f=document.createElement("span");f.setAttribute("class","bubble"),f.setAttribute("role","tooltip"),f.setAttribute("part","bubble"),D(t,"bubbleEl",f);let c=document.createElement("slot");if(c.setAttribute("name","content"),f.appendChild(c),m.appendChild(f),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var ko=0,on=S({mount(e,n){let r=e,o=`sui-collapsible-${++ko}`,t=T(()=>{let l=r.contentWrap();if(l)l.id=o}),a=()=>{let l=r.triggerSlot();if(!l)return;let i=r.open();for(let s of l.assignedElements()){let u=s;if(u.setAttribute("aria-expanded",String(i)),u.setAttribute("aria-controls",o),!u.hasAttribute("tabindex")&&!u.matches("button, a, input, select, textarea"))u.setAttribute("tabindex","0")}},m=T(()=>{let l=r.triggerSlot();if(!l)return;return l.addEventListener("slotchange",a),a(),()=>l.removeEventListener("slotchange",a)}),d=T(()=>{r.open(),r.triggerSlot(),a()}),f=()=>{if(r.disabled())return;let l=!r.open();r.open.set(l),n.dispatchEvent(new CustomEvent("sui-toggle",{detail:{open:l},bubbles:!0,composed:!0}))},c=(l)=>{if(l.key==="Enter"||l.key===" ")l.preventDefault(),f()},p=T(()=>{let l=r.triggerWrap();if(!l)return;return l.addEventListener("click",f),l.addEventListener("keydown",c),()=>{l.removeEventListener("click",f),l.removeEventListener("keydown",c)}});return()=>{t(),m(),d(),p()}}});var Rd=z("sui-collapsible",{props:{open:v.bool({default:!1,reflect:!0}),disabled:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.disabled=n.disabled;let a=null;if(on&&typeof on.setup==="function"){if(e.scope=t,a=on.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","trigger-wrap"),D(t,"triggerWrap",d);let f=document.createElement("slot");f.setAttribute("name","trigger"),D(t,"triggerSlot",f),d.appendChild(f),m.appendChild(d);let c=document.createElement("div");c.setAttribute("class","content-wrap"),c.setAttribute("part","content"),D(t,"contentWrap",c);let p=document.createElement("div");p.setAttribute("class","content-inner");let l=document.createElement("slot");if(p.appendChild(l),c.appendChild(p),m.appendChild(c),a!=null){let i=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(i){let s=new Proxy(t,{get(u,b){return u[b]},set(u,b,h){let g=u[b];if(g!=null&&typeof g.set==="function"){if(typeof h==="function")g.set(()=>h);else g.set(h);return!0}return!1}});if(i.mount){let u=i.mount(s,r);if(typeof u==="function")o.push(u)}if(i.unmount)o.push(()=>i.unmount(s))}}return t}});var an=S({mount(e,n){let r=e,o=T(()=>{let a=r.inputEl();if(!a)return;a.indeterminate=r.indeterminate()}),t=T(()=>{let a=r.inputEl();if(!a)return;let m=()=>{r.indeterminate.set(!1),n.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:a.checked,value:r.value()},bubbles:!0,composed:!0}))};return a.addEventListener("change",m),()=>a.removeEventListener("change",m)});return()=>{o(),t()}}});var Wd=z("sui-checkbox",{props:{checked:v.bool({default:!1,reflect:!0}),indeterminate:v.bool({default:!1,reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),required:v.bool({default:!1,reflect:!0}),invalid:v.bool({default:!1,reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),name:v.string({default:""}),value:v.string({default:"on"}),label:v.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.checked=n.checked,t.indeterminate=n.indeterminate,t.disabled=n.disabled,t.required=n.required,t.invalid=n.invalid,t.tone=n.tone,t.size=n.size,t.name=n.name,t.value=n.value,t.label=n.label;let a=null;if(an&&typeof an.setup==="function"){if(e.scope=t,a=an.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("label");d.setAttribute("part","label");let f=document.createElement("span");f.setAttribute("class","box"),f.setAttribute("part","box");let c=document.createElement("input");c.setAttribute("part","input"),c.setAttribute("type","checkbox"),o.push(dt(c,t.checked)),o.push(k(c,"disabled",()=>t.disabled())),o.push(k(c,"required",()=>t.required())),o.push(k(c,"name",()=>t.name())),o.push(k(c,"value",()=>t.value())),o.push(k(c,"aria-checked",()=>t.indeterminate()?"mixed":t.checked()?"true":"false")),o.push(k(c,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(k(c,"aria-label",()=>t.label()||t.name()||"checkbox")),D(t,"inputEl",c),f.appendChild(c);let p=document.createElement("span");p.setAttribute("class","glyph check"),p.setAttribute("aria-hidden","true"),f.appendChild(p);let l=document.createElement("span");l.setAttribute("class","glyph mixed"),l.setAttribute("aria-hidden","true"),f.appendChild(l),d.appendChild(f);let i=document.createElement("slot");if(d.appendChild(i),m.appendChild(d),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,g){let y=b[h];if(y!=null&&typeof y.set==="function"){if(typeof g==="function")y.set(()=>g);else y.set(g);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var sn=S({mount(e,n){let r=e,o=T(()=>{let t=r.inputEl();if(!t)return;let a=()=>{n.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:t.checked},bubbles:!0,composed:!0}))};return t.addEventListener("change",a),()=>t.removeEventListener("change",a)});return()=>o()}});var $d=z("sui-switch",{props:{checked:v.bool({default:!1,reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),name:v.string({default:""}),value:v.string({default:"on"}),label:v.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.checked=n.checked,t.disabled=n.disabled,t.tone=n.tone,t.size=n.size,t.name=n.name,t.value=n.value,t.label=n.label;let a=null;if(sn&&typeof sn.setup==="function"){if(e.scope=t,a=sn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("label"),f=document.createElement("span");f.setAttribute("class","track"),f.setAttribute("part","track");let c=document.createElement("input");c.setAttribute("part","input"),c.setAttribute("type","checkbox"),c.setAttribute("role","switch"),o.push(dt(c,t.checked)),o.push(k(c,"disabled",()=>t.disabled())),o.push(k(c,"name",()=>t.name())),o.push(k(c,"value",()=>t.value())),o.push(k(c,"aria-checked",()=>t.checked()?"true":"false")),o.push(k(c,"aria-label",()=>t.label()||t.name()||"toggle")),D(t,"inputEl",c),f.appendChild(c);let p=document.createElement("span");p.setAttribute("class","thumb"),p.setAttribute("part","thumb"),f.appendChild(p),d.appendChild(f);let l=document.createElement("slot");if(d.appendChild(l),m.appendChild(d),a!=null){let i=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(i){let s=new Proxy(t,{get(u,b){return u[b]},set(u,b,h){let g=u[b];if(g!=null&&typeof g.set==="function"){if(typeof h==="function")g.set(()=>h);else g.set(h);return!0}return!1}});if(i.mount){let u=i.mount(s,r);if(typeof u==="function")o.push(u)}if(i.unmount)o.push(()=>i.unmount(s))}}return t}});var ln=S({mount(e,n){let r=e,o=r.value(),t=T(()=>{let a=r.textareaEl();if(!a)return;let m=()=>{n.dispatchEvent(new CustomEvent("sui-input",{detail:{value:a.value},bubbles:!0,composed:!0}))},d=()=>{let f=r.value();if(f!==o)o=f,n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:f},bubbles:!0,composed:!0}))};return a.addEventListener("input",m),a.addEventListener("blur",d),()=>{a.removeEventListener("input",m),a.removeEventListener("blur",d)}});return()=>t()}});var oc=z("sui-textarea",{props:{value:v.string({default:""}),placeholder:v.string({default:""}),name:v.string({default:""}),rows:v.number({default:4}),required:v.bool({default:!1,reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),readonly:v.bool({default:!1,reflect:!0}),invalid:v.bool({default:!1,reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:v.union(["default","code"],{default:"default",reflect:!0}),size:v.union(["sm","md"],{default:"md",reflect:!0}),resize:v.union(["none","vertical","horizontal","both"],{default:"vertical"}),label:v.string({default:""}),"described-by":v.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.placeholder=n.placeholder,t.name=n.name,t.rows=n.rows,t.required=n.required,t.disabled=n.disabled,t.readonly=n.readonly,t.invalid=n.invalid,t.tone=n.tone,t.variant=n.variant,t.size=n.size,t.resize=n.resize,t.label=n.label,t.describedBy=n.describedBy;let a=null;if(ln&&typeof ln.setup==="function"){if(e.scope=t,a=ln.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("textarea");if(d.setAttribute("part","textarea"),o.push(k(d,"placeholder",()=>t.placeholder())),o.push(k(d,"name",()=>t.name())),o.push(k(d,"rows",()=>t.rows())),o.push(k(d,"disabled",()=>t.disabled())),o.push(k(d,"required",()=>t.required())),o.push(k(d,"readonly",()=>t.readonly())),o.push(k(d,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(k(d,"aria-required",()=>t.required()?"true":"false")),o.push(k(d,"aria-label",()=>t.label())),o.push(k(d,"aria-describedby",()=>t.describedBy())),o.push(T(()=>{let f=t.resize();if(f==null)d.style.removeProperty("resize");else d.style.setProperty("resize",String(f))})),o.push(ut(d,t.value)),D(t,"textareaEl",d),m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});var fr=0,un=S({mount(e,n){let r=e,o=`sui-field-hint-${++fr}`,t=`sui-field-error-${++fr}`,a=T(()=>{let c=r.hintEl(),p=r.errorEl();if(c)c.id=o;if(p)p.id=t}),m=()=>{let p=r.controlSlot()?.assignedElements({flatten:!0})[0];if(!p)return;let l=r.invalid(),i=l?t:o,u=[...(p.getAttribute("aria-describedby")||"").split(/\s+/).filter((b)=>b&&b!==o&&b!==t),i].join(" ").trim();if(u)p.setAttribute("aria-describedby",u);if("invalid"in p)p.invalid=l;else p.setAttribute("aria-invalid",l?"true":"false")},d=T(()=>{r.invalid(),m()}),f=T(()=>{let c=r.controlSlot();if(!c)return;return c.addEventListener("slotchange",m),()=>c.removeEventListener("slotchange",m)});return()=>{a(),d(),f()}}});var dc=z("sui-field",{props:{invalid:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.invalid=n.invalid;let a=null;if(un&&typeof un.setup==="function"){if(e.scope=t,a=un.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");d.setAttribute("name","label"),m.appendChild(d);let f=document.createElement("slot");D(t,"controlSlot",f),m.appendChild(f);let c=document.createElement("div");c.setAttribute("class","hint"),D(t,"hintEl",c);let p=document.createElement("slot");p.setAttribute("name","hint"),c.appendChild(p),m.appendChild(c);let l=document.createElement("div");l.setAttribute("class","error"),l.setAttribute("role","alert"),l.setAttribute("aria-live","polite"),D(t,"errorEl",l);let i=document.createElement("slot");if(i.setAttribute("name","error"),l.appendChild(i),m.appendChild(l),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,g){let y=b[h];if(y!=null&&typeof y.set==="function"){if(typeof g==="function")y.set(()=>g);else y.set(g);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var dn=S({scope:(e,n)=>{let r=e,o=W(null),t=q(()=>r.tone()==="destructive"?"destructive":"default");function a(f){n.dispatchEvent(new CustomEvent(f,{bubbles:!0,composed:!0})),r.open.set(!1)}function m(){a("sui-confirm")}function d(){a("sui-cancel")}return{dialogRef:o,confirmTone:t,onConfirm:m,onCancel:d}},mount(e,n){let r=e,o=r.open,t=r.dialogRef,a=T(()=>{let d=o(),f=t();if(!f)return;if(d&&typeof f.show==="function")f.show();else if(!d&&typeof f.close==="function")f.close()}),m=()=>{if(o())o.set(!1)};return n.addEventListener("sui-close",m),()=>{a(),n.removeEventListener("sui-close",m)}}});var hc=z("sui-alert-dialog",{props:{open:v.bool({default:!1,reflect:!0}),tone:v.union(["default","destructive"],{default:"default",reflect:!0}),"confirm-label":v.string({default:"Confirm"}),"cancel-label":v.string({default:"Cancel"})},shadow:!0,styles:`
  :host { display: contents; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.tone=n.tone,t.confirmLabel=n.confirmLabel,t.cancelLabel=n.cancelLabel;let a=null;if(dn&&typeof dn.setup==="function"){if(e.scope=t,a=dn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("sui-dialog");d.setAttribute("role-override","alertdialog"),d.setAttribute("size","sm"),d.setAttribute("no-close-button",""),D(t,"dialogRef",d);let f=document.createElement("span");f.setAttribute("slot","header");let c=document.createElement("slot");c.setAttribute("name","title"),f.appendChild(c),d.appendChild(f);let p=document.createElement("slot");d.appendChild(p);let l=document.createElement("span");l.setAttribute("slot","footer");let i=document.createElement("sui-button");i.setAttribute("variant","outline"),o.push(I(i,"click",function(){let h=t.onCancel;return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let s=document.createTextNode("");i.appendChild(s),o.push(R(s,()=>t.cancelLabel())),l.appendChild(i);let u=document.createElement("sui-button");u.setAttribute("variant","solid"),o.push(k(u,"tone",()=>t.confirmTone())),o.push(I(u,"click",function(){let h=t.onConfirm;return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let b=document.createTextNode("");if(u.appendChild(b),o.push(R(b,()=>t.confirmLabel())),l.appendChild(u),d.appendChild(l),m.appendChild(d),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let g=new Proxy(t,{get(y,x){return y[x]},set(y,x,E){let w=y[x];if(w!=null&&typeof w.set==="function"){if(typeof E==="function")w.set(()=>E);else w.set(E);return!0}return!1}});if(h.mount){let y=h.mount(g,r);if(typeof y==="function")o.push(y)}if(h.unmount)o.push(()=>h.unmount(g))}}return t}});function mr(e){if(e<1024)return`${e} B`;if(e<1048576)return`${(e/1024).toFixed(1)} KB`;return`${(e/1048576).toFixed(1)} MB`}var cn=S({scope:()=>{let e=N([]),n=N(!1),r=N(""),o=W(null),t=W(null),a=W(null),m=q(()=>e().map((d,f)=>({file:d,idx:f,key:`${f}:${d.name}:${d.size}`,name:d.name,sizeStr:mr(d.size)})));return{files:e,over:n,errorMsg:r,displayFiles:m,dropzoneEl:o,inputEl:t,fileListEl:a}},mount(e,n){let r=e,o=r.files,t=r.over,a=r.errorMsg,m=(E)=>{let w=r.accept().split(",").map((C)=>C.trim().toLowerCase()).filter(Boolean);if(w.length===0)return!0;let A="."+(E.name.split(".").pop()||"").toLowerCase(),L=E.type.toLowerCase();return w.some((C)=>{if(C.startsWith("."))return A===C;if(C.endsWith("/*"))return L.startsWith(C.slice(0,-1));return L===C})},d=(E)=>{n.dispatchEvent(new CustomEvent("sui-change",{detail:{files:[...E]},bubbles:!0,composed:!0}))},f=(E)=>{a.set("");let w=[];for(let C of E){if(C.size>r.maxSize()){a.set(`"${C.name}" exceeds max size (${mr(r.maxSize())}).`);continue}if(r.accept()&&!m(C)){a.set(`"${C.name}" is not an accepted file type.`);continue}w.push(C)}let A=o(),L=r.multiple()?[...A,...w]:w.slice(0,1);o.set(L),d(L)},c=(E)=>{let w=o();if(E<0||E>=w.length)return;let A=w.filter((L,C)=>C!==E);o.set(A),a.set(""),d(A)},p=()=>{if(r.disabled())return;r.inputEl()?.click()},l=()=>p(),i=(E)=>{if(r.disabled())return;if(E.key==="Enter"||E.key===" ")E.preventDefault(),p()},s=(E)=>{if(r.disabled())return;E.preventDefault(),t.set(!0)},u=()=>t.set(!1),b=(E)=>{if(r.disabled())return;E.preventDefault(),t.set(!1);let w=E.dataTransfer?.files;if(w&&w.length)f(Array.from(w))},h=(E)=>{let w=E.target;if(w.files&&w.files.length)f(Array.from(w.files));w.value=""},g=r.dropzoneEl(),y=r.inputEl();g?.addEventListener("click",l),g?.addEventListener("keydown",i),g?.addEventListener("dragover",s),g?.addEventListener("dragleave",u),g?.addEventListener("drop",b),y?.addEventListener("change",h);let x=(E)=>{let A=E.target?.closest?.("button.remove");if(!A||!n.shadowRoot?.contains(A))return;let L=Number(A.dataset.idx);if(Number.isInteger(L))c(L)};return n.shadowRoot?.addEventListener("click",x),()=>{g?.removeEventListener("click",l),g?.removeEventListener("keydown",i),g?.removeEventListener("dragover",s),g?.removeEventListener("dragleave",u),g?.removeEventListener("drop",b),y?.removeEventListener("change",h),n.shadowRoot?.removeEventListener("click",x)}}});var Ac=z("sui-file-upload",{props:{accept:v.string({default:""}),multiple:v.bool({default:!1}),"max-size":v.number({default:10485760}),disabled:v.bool({default:!1,reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.accept=n.accept,t.multiple=n.multiple,t.maxSize=n.maxSize,t.disabled=n.disabled,t.tone=n.tone;let a=null;if(cn&&typeof cn.setup==="function"){if(e.scope=t,a=cn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(y)=>G({anchor:y,test:()=>{if(t.errorMsg())return 0;return-1},branches:[()=>{let x=document.createDocumentFragment(),E=[],w=document.createElement("div");w.setAttribute("class","error"),w.setAttribute("role","alert"),w.setAttribute("part","error");let A=document.createTextNode("");return w.appendChild(A),E.push(R(A,()=>t.errorMsg())),x.appendChild(w),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let L of E)try{L()}catch(C){}}}}]}),d=(y)=>Y({anchor:y,items:()=>t.displayFiles(),key:(x,E)=>{let w=Object.create(t);return w.f=x,w.__i=E,((A)=>x.key)(w)},render:(x,E)=>{let w=Object.create(t);return w.f=x,w.__i=E,w.__v=N(0),((A)=>{let L=document.createDocumentFragment(),C=[],O=document.createElement("div");O.setAttribute("class","file-item"),O.setAttribute("part","file-item");let P=document.createElement("span");P.setAttribute("class","file-name");let M=document.createTextNode("");P.appendChild(M),C.push(R(M,()=>(A.__v(),A.f.name))),O.appendChild(P);let H=document.createElement("span");H.setAttribute("class","file-size");let B=document.createTextNode("");H.appendChild(B),C.push(R(B,()=>(A.__v(),A.f.sizeStr))),O.appendChild(H);let F=document.createElement("button");F.setAttribute("class","remove"),F.setAttribute("part","remove"),F.setAttribute("type","button"),C.push(k(F,"data-idx",()=>(A.__v(),A.f.idx))),C.push(k(F,"aria-label",()=>(A.__v(),"Remove "+A.f.name)));let U=document.createTextNode("×");return F.appendChild(U),O.appendChild(F),L.appendChild(O),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let J of C)try{J()}catch(Q){}},update:(J,Q)=>{A.f=J,A.__i=Q,A.__v.set((X)=>X+1)}}})(w)}}),f=(y)=>G({anchor:y,test:()=>{if(t.displayFiles().length>0)return 0;return-1},branches:[()=>{let x=document.createDocumentFragment(),E=[],w=document.createElement("div");w.setAttribute("class","file-list"),w.setAttribute("part","file-list"),w.setAttribute("aria-live","polite"),D(t,"fileListEl",w);let A=document.createComment("for");return w.appendChild(A),E.push(d(A)),x.appendChild(w),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let L of E)try{L()}catch(C){}}}}]}),c=r.shadowRoot??r;j(c);let p=document.createElement("div");p.setAttribute("class","dropzone"),o.push(mt(p,"over",()=>t.over())),p.setAttribute("part","dropzone"),p.setAttribute("role","button"),o.push(k(p,"tabindex",()=>t.disabled()?"-1":"0")),o.push(k(p,"aria-label",()=>t.accept()?"Upload files ("+t.accept()+")":"Upload files")),o.push(k(p,"aria-disabled",()=>t.disabled()?"true":"false")),D(t,"dropzoneEl",p);let l=document.createElement("span");l.setAttribute("class","icon"),l.setAttribute("aria-hidden","true");let i=document.createTextNode("\uD83D\uDCC4");l.appendChild(i),p.appendChild(l);let s=document.createElement("span"),u=document.createTextNode("Drop files here or click to browse");s.appendChild(u),p.appendChild(s),c.appendChild(p);let b=document.createElement("input");b.setAttribute("type","file"),b.setAttribute("part","input"),o.push(k(b,"accept",()=>t.accept())),o.push(k(b,"multiple",()=>t.multiple())),b.setAttribute("aria-hidden","true"),b.setAttribute("tabindex","-1"),D(t,"inputEl",b),c.appendChild(b);let h=document.createComment("if");c.appendChild(h),o.push(m(h));let g=document.createComment("if");if(c.appendChild(g),o.push(f(g)),a!=null){let y=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(y){let x=new Proxy(t,{get(E,w){return E[w]},set(E,w,A){let L=E[w];if(L!=null&&typeof L.set==="function"){if(typeof A==="function")L.set(()=>A);else L.set(A);return!0}return!1}});if(y.mount){let E=y.mount(x,r);if(typeof E==="function")o.push(E)}if(y.unmount)o.push(()=>y.unmount(x))}}return t}});var Co=new Set(["info","success","warning","destructive","accent"]),pn=S({scope:()=>{let e=N(!1),n=W(null),r=W(null),o=W(null),t=W(null);return{hasSlottedIcon:e,iconSlotEl:n,titleSlotEl:r,actionsSlotEl:o,closeBtn:t}},mount(e,n){let r=e,o=null,t=0,a=0,m=!1;if(!n.hasAttribute("tabindex"))n.tabIndex=0;function d(){if(o)clearTimeout(o),o=null}function f(){d();let A=r.duration();if(A>0)t=A,a=Date.now(),o=setTimeout(()=>c("timeout"),A),n.style.setProperty("--toast-duration",`${A}ms`)}function c(A="manual"){if(m)return;m=!0,d(),n.dispatchEvent(new CustomEvent("sui-dismiss",{detail:{reason:A},bubbles:!0,composed:!0})),n.remove()}async function p(A="manual"){if(m)return;if(typeof window<"u"&&window.matchMedia?.("(prefers-reduced-motion: reduce)").matches){c(A);return}d(),n.setAttribute("closing",""),await new Promise((C)=>{let O=()=>{n.removeEventListener("animationend",O),C()};n.addEventListener("animationend",O),setTimeout(()=>C(),260)}),c(A)}let l=()=>{if(!o)return;let A=Date.now()-a;t=Math.max(0,t-A),d(),n.toggleAttribute("data-paused",!0)},i=()=>{if(m||r.duration()<=0)return;if(t<=0)return;n.toggleAttribute("data-paused",!1),a=Date.now(),o=setTimeout(()=>c("timeout"),t)},s=(A)=>{if(A.key==="Escape"&&r.dismissible())A.stopPropagation(),c("manual")};n.addEventListener("mouseenter",l),n.addEventListener("mouseleave",i),n.addEventListener("focusin",l),n.addEventListener("focusout",i),n.addEventListener("keydown",s),f();let u=r.hasSlottedIcon,b=(A)=>A.assignedNodes({flatten:!0}).some((L)=>{if(L.nodeType===Node.ELEMENT_NODE)return!0;return(L.textContent?.trim().length??0)>0}),h=r.iconSlotEl();if(h){let A=()=>u.set(b(h));h.addEventListener("slotchange",A),u.set(b(h))}let g=r.titleSlotEl(),y=r.actionsSlotEl();if(g){let A=()=>n.toggleAttribute("data-no-title",!b(g));g.addEventListener("slotchange",A),n.toggleAttribute("data-no-title",!b(g))}if(y){let A=()=>n.toggleAttribute("data-no-actions",!b(y));y.addEventListener("slotchange",A),n.toggleAttribute("data-no-actions",!b(y))}let x=T(()=>{let A=r.tone(),L=r.hasSlottedIcon(),C=Co.has(A);n.toggleAttribute("data-no-icon",!C&&!L)}),E=T(()=>{let A=r.closeBtn();if(!A)return;let L=()=>c("manual");return A.addEventListener("click",L),()=>A.removeEventListener("click",L)}),w=n;return w.dismiss=c,w.close=p,()=>{m=!0,d(),n.removeEventListener("mouseenter",l),n.removeEventListener("mouseleave",i),n.removeEventListener("focusin",l),n.removeEventListener("focusout",i),n.removeEventListener("keydown",s),x(),E()}}});var jc=z("sui-toast",{props:{tone:v.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),duration:v.number({default:5000}),dismissible:v.bool({default:!0,reflect:!0}),"show-progress":v.bool({default:!1,reflect:!0}),closing:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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

  @keyframes sui-toast-in {
    from {
      opacity: 0;
      transform: translate3d(var(--toast-slide-x), var(--toast-slide-y), 0) scale(0.98);
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
    animation: sui-toast-in var(--duration-medium, 200ms) var(--ease-out, ease-out) both;
  }

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

  :host([closing]) {
    animation: sui-toast-out var(--duration-medium, 200ms) var(--ease-in, ease-in) forwards;
  }
  @keyframes sui-toast-out {
    to {
      opacity: 0;
      transform: translate3d(var(--toast-slide-x), var(--toast-slide-y), 0) scale(0.96);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .toast { animation: none; }
    :host([closing]) { animation: none; }
    .progress { animation: none; }
  }

  @media (forced-colors: active) {
    .toast { border: 1px solid CanvasText; }
    .close:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.size=n.size,t.duration=n.duration,t.dismissible=n.dismissible,t.showProgress=n.showProgress,t.closing=n.closing;let a=null;if(pn&&typeof pn.setup==="function"){if(e.scope=t,a=pn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(C)=>G({anchor:C,test:()=>{let O=t.tone();if(O==="info")return 0;if(O==="success")return 1;if(O==="warning")return 2;if(O==="destructive")return 3;if(O==="accent")return 4;return-1},branches:[()=>{let O=document.createDocumentFragment(),P=[],M=document.createElement("svg");M.setAttribute("viewbox","0 0 24 24"),M.setAttribute("fill","none"),M.setAttribute("stroke","currentColor"),M.setAttribute("stroke-width","2"),M.setAttribute("stroke-linecap","round"),M.setAttribute("stroke-linejoin","round"),M.setAttribute("aria-hidden","true");let H=document.createElement("circle");H.setAttribute("cx","12"),H.setAttribute("cy","12"),H.setAttribute("r","9"),M.appendChild(H);let B=document.createElement("path");B.setAttribute("d","M12 8h.01"),M.appendChild(B);let F=document.createElement("path");return F.setAttribute("d","M11 12h1v4h1"),M.appendChild(F),O.appendChild(M),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let U of P)try{U()}catch(J){}}}},()=>{let O=document.createDocumentFragment(),P=[],M=document.createElement("svg");M.setAttribute("viewbox","0 0 24 24"),M.setAttribute("fill","none"),M.setAttribute("stroke","currentColor"),M.setAttribute("stroke-width","2"),M.setAttribute("stroke-linecap","round"),M.setAttribute("stroke-linejoin","round"),M.setAttribute("aria-hidden","true");let H=document.createElement("circle");H.setAttribute("cx","12"),H.setAttribute("cy","12"),H.setAttribute("r","9"),M.appendChild(H);let B=document.createElement("path");return B.setAttribute("d","m8.5 12.5 2.5 2.5 4.5-5"),M.appendChild(B),O.appendChild(M),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let F of P)try{F()}catch(U){}}}},()=>{let O=document.createDocumentFragment(),P=[],M=document.createElement("svg");M.setAttribute("viewbox","0 0 24 24"),M.setAttribute("fill","none"),M.setAttribute("stroke","currentColor"),M.setAttribute("stroke-width","2"),M.setAttribute("stroke-linecap","round"),M.setAttribute("stroke-linejoin","round"),M.setAttribute("aria-hidden","true");let H=document.createElement("path");H.setAttribute("d","M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"),M.appendChild(H);let B=document.createElement("line");B.setAttribute("x1","12"),B.setAttribute("x2","12"),B.setAttribute("y1","9"),B.setAttribute("y2","13"),M.appendChild(B);let F=document.createElement("line");return F.setAttribute("x1","12"),F.setAttribute("x2","12.01"),F.setAttribute("y1","17"),F.setAttribute("y2","17"),M.appendChild(F),O.appendChild(M),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let U of P)try{U()}catch(J){}}}},()=>{let O=document.createDocumentFragment(),P=[],M=document.createElement("svg");M.setAttribute("viewbox","0 0 24 24"),M.setAttribute("fill","none"),M.setAttribute("stroke","currentColor"),M.setAttribute("stroke-width","2"),M.setAttribute("stroke-linecap","round"),M.setAttribute("stroke-linejoin","round"),M.setAttribute("aria-hidden","true");let H=document.createElement("circle");H.setAttribute("cx","12"),H.setAttribute("cy","12"),H.setAttribute("r","9"),M.appendChild(H);let B=document.createElement("line");B.setAttribute("x1","15"),B.setAttribute("x2","9"),B.setAttribute("y1","9"),B.setAttribute("y2","15"),M.appendChild(B);let F=document.createElement("line");return F.setAttribute("x1","9"),F.setAttribute("x2","15"),F.setAttribute("y1","9"),F.setAttribute("y2","15"),M.appendChild(F),O.appendChild(M),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let U of P)try{U()}catch(J){}}}},()=>{let O=document.createDocumentFragment(),P=[],M=document.createElement("svg");M.setAttribute("viewbox","0 0 24 24"),M.setAttribute("fill","none"),M.setAttribute("stroke","currentColor"),M.setAttribute("stroke-width","2"),M.setAttribute("stroke-linecap","round"),M.setAttribute("stroke-linejoin","round"),M.setAttribute("aria-hidden","true");let H=document.createElement("path");return H.setAttribute("d","M12 2 3 14h7l-1 8 10-12h-7l1-8Z"),M.appendChild(H),O.appendChild(M),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let B of P)try{B()}catch(F){}}}}]}),d=(C)=>G({anchor:C,test:()=>{if(!t.hasSlottedIcon())return 0;return-1},branches:[()=>{let O=document.createDocumentFragment(),P=[],M=document.createComment("match");return O.appendChild(M),P.push(m(M)),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let H of P)try{H()}catch(B){}}}}]}),f=(C)=>G({anchor:C,test:()=>{if(t.dismissible())return 0;return-1},branches:[()=>{let O=document.createDocumentFragment(),P=[],M=document.createElement("button");M.setAttribute("class","close"),M.setAttribute("part","close"),M.setAttribute("type","button"),M.setAttribute("aria-label","Dismiss notification"),D(t,"closeBtn",M);let H=document.createElement("svg");H.setAttribute("viewbox","0 0 24 24"),H.setAttribute("fill","none"),H.setAttribute("stroke","currentColor"),H.setAttribute("stroke-width","2"),H.setAttribute("stroke-linecap","round"),H.setAttribute("stroke-linejoin","round"),H.setAttribute("aria-hidden","true");let B=document.createElement("line");B.setAttribute("x1","18"),B.setAttribute("y1","6"),B.setAttribute("x2","6"),B.setAttribute("y2","18"),H.appendChild(B);let F=document.createElement("line");return F.setAttribute("x1","6"),F.setAttribute("y1","6"),F.setAttribute("x2","18"),F.setAttribute("y2","18"),H.appendChild(F),M.appendChild(H),O.appendChild(M),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let U of P)try{U()}catch(J){}}}}]}),c=(C)=>G({anchor:C,test:()=>{if(t.showProgress()&&t.duration()>0)return 0;return-1},branches:[()=>{let O=document.createDocumentFragment(),P=[],M=document.createElement("span");return M.setAttribute("class","progress"),M.setAttribute("aria-hidden","true"),O.appendChild(M),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let H of P)try{H()}catch(B){}}}}]}),p=r.shadowRoot??r;j(p);let l=document.createElement("div");l.setAttribute("class","toast"),o.push(k(l,"role",()=>t.tone()==="destructive"||t.tone()==="warning"?"alert":"status")),o.push(k(l,"aria-live",()=>t.tone()==="destructive"||t.tone()==="warning"?"assertive":"polite")),l.setAttribute("aria-atomic","true"),l.setAttribute("part","toast");let i=document.createElement("span");i.setAttribute("class","icon");let s=document.createElement("slot");s.setAttribute("name","icon"),D(t,"iconSlotEl",s),i.appendChild(s);let u=document.createComment("if");i.appendChild(u),o.push(d(u)),l.appendChild(i);let b=document.createElement("div");b.setAttribute("class","content");let h=document.createElement("div");h.setAttribute("class","title");let g=document.createElement("slot");g.setAttribute("name","title"),D(t,"titleSlotEl",g),h.appendChild(g),b.appendChild(h);let y=document.createElement("div");y.setAttribute("class","body");let x=document.createElement("slot");y.appendChild(x),b.appendChild(y);let E=document.createElement("div");E.setAttribute("class","actions");let w=document.createElement("slot");w.setAttribute("name","actions"),D(t,"actionsSlotEl",w),E.appendChild(w),b.appendChild(E),l.appendChild(b);let A=document.createComment("if");l.appendChild(A),o.push(f(A));let L=document.createComment("if");if(l.appendChild(L),o.push(c(L)),p.appendChild(l),a!=null){let C=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(C){let O=new Proxy(t,{get(P,M){return P[M]},set(P,M,H){let B=P[M];if(B!=null&&typeof B.set==="function"){if(typeof H==="function")B.set(()=>H);else B.set(H);return!0}return!1}});if(C.mount){let P=C.mount(O,r);if(typeof P==="function")o.push(P)}if(C.unmount)o.push(()=>C.unmount(O))}}return t}});var Lo={"top-left":"top","top-center":"top","top-right":"top","bottom-left":"bottom","bottom-center":"bottom","bottom-right":"bottom"},fn=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","region");if(!n.hasAttribute("aria-label"))n.setAttribute("aria-label","Notifications");if(!n.hasAttribute("aria-live"))n.setAttribute("aria-live","polite");let o=N(0),t=r.defaultSlotEl(),a=()=>{if(t)return t.assignedElements().filter((c)=>c.tagName.toLowerCase()==="sui-toast");return Array.from(n.querySelectorAll(":scope > sui-toast"))},m=T(()=>{o();let c=r.position(),p=r.maxVisible(),l=Lo[c]??"bottom",i=a();for(let s of i)s.setAttribute("data-slide-from",l);if(p>0&&i.length>p){let s=i.length-p;for(let u=0;u<s;u++){let b=i[u];if(typeof b.close==="function")b.close("programmatic");else b.dismiss?.("programmatic")}}}),d=()=>o.set(o()+1);if(t)t.addEventListener("slotchange",d);function f(c={}){let p=n.ownerDocument??document,l=p.createElement("sui-toast");if(c.tone)l.setAttribute("tone",c.tone);if(c.size)l.setAttribute("size",c.size);if(typeof c.duration==="number")l.setAttribute("duration",String(c.duration));if(c.dismissible===!1)l.removeAttribute("dismissible");else l.setAttribute("dismissible","");if(c.showProgress)l.setAttribute("show-progress","");if(c.title){let i=p.createElement("span");i.setAttribute("slot","title"),i.textContent=c.title,l.appendChild(i)}if(c.description)l.appendChild(p.createTextNode(c.description));if(c.actionHtml){let i=p.createElement("span");i.setAttribute("slot","actions");let s=c.actionHtml;nt(i,()=>s),l.appendChild(i)}return n.appendChild(l),l}return n.show=f,()=>{if(m(),t)t.removeEventListener("slotchange",d)}}});var Pc=z("sui-toast-container",{props:{position:v.union(["top-left","top-center","top-right","bottom-left","bottom-center","bottom-right"],{default:"bottom-right",reflect:!0}),"max-visible":v.number({default:5,reflect:!0})},shadow:!0,styles:`
  :host {
    position: fixed;
    z-index: var(--z-toast, 9999);
    display: flex;
    flex-direction: column;
    gap: var(--space-2, 8px);
    pointer-events: none;
    max-width: min(440px, calc(100vw - 32px));
    --toast-edge: 16px;
    padding: 0;
    margin: 0;
    box-sizing: border-box;
  }

  :host(:not([position])),
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
  :host([position='top-right']) {
    top: var(--toast-edge);
    right: var(--toast-edge);
    bottom: auto;
    left: auto;
    flex-direction: column;
    align-items: flex-end;
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

  ::slotted(sui-toast) {
    pointer-events: auto;
    width: auto;
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.position=n.position,t.maxVisible=n.maxVisible;let a=null;if(fn&&typeof fn.setup==="function"){if(e.scope=t,a=fn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");if(D(t,"defaultSlotEl",d),m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});var mn=S({scope:()=>{let e=N([]),n=N("");return{items:e,lastSlot:n}},mount(e,n){let r=e,o=r.items,t=r.lastSlot,a=()=>{let f=Array.from(n.querySelectorAll(":scope > sui-breadcrumb-item")).map((c,p)=>{let l=`i${p}`;return c.setAttribute("slot",l),{slot:l}});t.set(f.length>0?f[f.length-1].slot:""),o.set(f)};a();let m=new MutationObserver(a);return m.observe(n,{childList:!0}),()=>m.disconnect()}});var Rc=z("sui-breadcrumb",{props:{separator:v.string({default:"/",reflect:!0})},shadow:!0,styles:`
  :host { display: flex; align-items: center; }
  nav { display: inline-flex; align-items: center; flex-wrap: wrap; gap: 2px; }
  .sep {
    display: inline-flex; align-items: center; justify-content: center;
    width: 12px; height: 12px;
    color: var(--muted-foreground);
    opacity: 0.7;
  }
  .sep svg { width: 12px; height: 12px; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.separator=n.separator;let a=null;if(mn&&typeof mn.setup==="function"){if(e.scope=t,a=mn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(p)=>Y({anchor:p,items:()=>t.items(),key:(l,i)=>{let s=Object.create(t);return s.item=l,s.__i=i,((u)=>l.slot)(s)},render:(l,i)=>{let s=Object.create(t);return s.item=l,s.__i=i,s.__v=N(0),((u)=>{let b=(w)=>G({anchor:w,test:()=>{if(u.separator()==="/")return 0;return 1},branches:[()=>{let A=document.createDocumentFragment(),L=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round");let O=document.createElement("path");return O.setAttribute("d","M9 6l6 6-6 6"),C.appendChild(O),A.appendChild(C),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}}}},()=>{let A=document.createDocumentFragment(),L=[],C=document.createTextNode("");return A.appendChild(C),L.push(R(C,()=>(u.__v(),u.separator()))),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let O of L)try{O()}catch(P){}}}}]}),h=(w)=>G({anchor:w,test:()=>{if(u.item.slot!==u.lastSlot())return 0;return-1},branches:[()=>{let A=document.createDocumentFragment(),L=[],C=document.createElement("span");C.setAttribute("class","sep"),C.setAttribute("aria-hidden","true");let O=document.createComment("if");return C.appendChild(O),L.push(b(O)),A.appendChild(C),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}}}}]}),g=document.createDocumentFragment(),y=[],x=document.createElement("slot");y.push(k(x,"name",()=>(u.__v(),u.item.slot))),g.appendChild(x);let E=document.createComment("if");return g.appendChild(E),y.push(h(E)),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let w of y)try{w()}catch(A){}},update:(w,A)=>{u.item=w,u.__i=A,u.__v.set((L)=>L+1)}}})(s)}}),d=r.shadowRoot??r;j(d);let f=document.createElement("nav");f.setAttribute("aria-label","Breadcrumb"),f.setAttribute("part","nav");let c=document.createComment("for");if(f.appendChild(c),o.push(m(c)),d.appendChild(f),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var Xc=z("sui-breadcrumb-item",{props:{href:v.string({default:"",reflect:!0}),active:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.href=n.href,t.active=n.active;let a=(f)=>G({anchor:f,test:()=>{if(t.href()&&!t.active())return 0;return 1},branches:[()=>{let c=document.createDocumentFragment(),p=[],l=document.createElement("a");p.push(k(l,"href",()=>t.href())),l.setAttribute("part","link");let i=document.createElement("slot");return l.appendChild(i),c.appendChild(l),{nodes:Array.from(c.childNodes),cleanup:()=>{for(let s of p)try{s()}catch(u){}}}},()=>{let c=document.createDocumentFragment(),p=[],l=document.createElement("span");l.setAttribute("class","label"),l.setAttribute("part","label"),p.push(k(l,"aria-current",()=>t.active()?"page":"false"));let i=document.createElement("slot");return l.appendChild(i),c.appendChild(l),{nodes:Array.from(c.childNodes),cleanup:()=>{for(let s of p)try{s()}catch(u){}}}}]}),m=r.shadowRoot??r;j(m);let d=document.createComment("if");return m.appendChild(d),o.push(a(d)),t}});var bn=S({scope:()=>({totalPages:N(1),info:N(""),prevBtn:W(null),nextBtn:W(null)}),mount(e,n){let r=e,o=T(()=>{let f=r.page(),c=r.total(),p=r.perPage(),l=c>0?Math.ceil(c/p):1;if(r.totalPages.set(l),c>0){let i=(f-1)*p+1,s=Math.min(f*p,c);r.info.set(`Page ${f} of ${l} (${i}–${s} of ${c})`)}else r.info.set(`Page ${f} of ${l}`)}),t=r.prevBtn(),a=r.nextBtn(),m=()=>{let f=r.page();if(f>1){let c=f-1;r.page.set(c),n.dispatchEvent(new CustomEvent("sui-page",{detail:{page:c},bubbles:!0,composed:!0}))}},d=()=>{let f=r.page(),c=r.totalPages();if(f<c){let p=f+1;r.page.set(p),n.dispatchEvent(new CustomEvent("sui-page",{detail:{page:p},bubbles:!0,composed:!0}))}};return t?.addEventListener("click",m),a?.addEventListener("click",d),()=>{o(),t?.removeEventListener("click",m),a?.removeEventListener("click",d)}}});var Ic=z("sui-pagination",{props:{page:v.number({default:1,reflect:!0}),total:v.number({default:0,reflect:!0}),"per-page":v.number({default:25,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.page=n.page,t.total=n.total,t.perPage=n.perPage;let a=null;if(bn&&typeof bn.setup==="function"){if(e.scope=t,a=bn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("nav");d.setAttribute("class","bar"),d.setAttribute("aria-label","Pagination");let f=document.createElement("button");D(t,"prevBtn",f),f.setAttribute("class","pg-btn"),f.setAttribute("part","prev"),o.push(k(f,"disabled",()=>t.page()<=1)),f.setAttribute("aria-label","Previous page"),o.push(k(f,"aria-disabled",()=>t.page()<=1?"true":"false"));let c=document.createTextNode("Prev");f.appendChild(c),d.appendChild(f);let p=document.createElement("span");p.setAttribute("class","info"),p.setAttribute("part","info");let l=document.createTextNode("");p.appendChild(l),o.push(R(l,()=>t.info())),d.appendChild(p);let i=document.createElement("span");i.setAttribute("class","spacer"),d.appendChild(i);let s=document.createElement("button");D(t,"nextBtn",s),s.setAttribute("class","pg-btn"),s.setAttribute("part","next"),o.push(k(s,"disabled",()=>t.page()>=t.totalPages())),s.setAttribute("aria-label","Next page"),o.push(k(s,"aria-disabled",()=>t.page()>=t.totalPages()?"true":"false"));let u=document.createTextNode("Next");if(s.appendChild(u),d.appendChild(s),m.appendChild(d),a!=null){let b=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(b){let h=new Proxy(t,{get(g,y){return g[y]},set(g,y,x){let E=g[y];if(E!=null&&typeof E.set==="function"){if(typeof x==="function")E.set(()=>x);else E.set(x);return!0}return!1}});if(b.mount){let g=b.mount(h,r);if(typeof g==="function")o.push(g)}if(b.unmount)o.push(()=>b.unmount(h))}}return t}});function br(e,n=Date.now()){let r=Date.parse(e);if(Number.isNaN(r))return e;let o=n-r,t=Math.abs(o),a=o>=0?"ago":"from now";if(t<60000)return"just now";if(t<3600000)return`${Math.floor(t/60000)}m ${a}`;if(t<86400000)return`${Math.floor(t/3600000)}h ${a}`;let m=Math.floor(t/86400000);if(m<30)return`${m}d ${a}`;if(m<365)return`${Math.floor(m/30)}mo ${a}`;return`${Math.floor(m/365)}y ${a}`}function hr(e){let n=Date.parse(e);return Number.isNaN(n)?e:new Date(n).toISOString()}var zo=60000,hn=S({scope:({value:e})=>{let n=N(0),r=q(()=>{let t=e();return t?hr(t):""}),o=q(()=>{n();let t=e();return t?br(t):""});return{tick:n,absStr:r,relStr:o}},mount(e,n){let r=e,o=r.tick,t=T(()=>{if(r.value())n.title=r.absStr()}),a=setInterval(()=>o.set(o()+1),zo);return()=>{t(),clearInterval(a)}}});var rp=z("sui-timestamp",{props:{value:v.string({default:"",reflect:!0}),format:v.union(["relative","absolute","both"],{default:"relative",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline;
    font-family: var(--font-body);
    font-size: inherit;
    color: inherit;
  }
  .abs { color: var(--muted-foreground); margin-left: 4px; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.format=n.format;let a=null;if(hn&&typeof hn.setup==="function"){if(e.scope=t,a=hn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(l)=>G({anchor:l,test:()=>{if(t.format()==="absolute")return 0;return 1},branches:[()=>{let i=document.createDocumentFragment(),s=[],u=document.createTextNode("");return i.appendChild(u),s.push(R(u,()=>t.absStr())),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let b of s)try{b()}catch(h){}}}},()=>{let i=document.createDocumentFragment(),s=[],u=document.createTextNode("");return i.appendChild(u),s.push(R(u,()=>t.relStr())),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let b of s)try{b()}catch(h){}}}}]}),d=(l)=>G({anchor:l,test:()=>{if(t.format()==="both")return 0;return-1},branches:[()=>{let i=document.createDocumentFragment(),s=[],u=document.createElement("span");u.setAttribute("class","abs"),u.setAttribute("part","abs");let b=document.createTextNode("(");u.appendChild(b);let h=document.createTextNode("");u.appendChild(h),s.push(R(h,()=>t.absStr()));let g=document.createTextNode(")");return u.appendChild(g),i.appendChild(u),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let y of s)try{y()}catch(x){}}}}]}),f=(l)=>G({anchor:l,test:()=>{if(t.value())return 0;return-1},branches:[()=>{let i=document.createDocumentFragment(),s=[],u=document.createElement("time");s.push(k(u,"datetime",()=>t.value())),u.setAttribute("part","time");let b=document.createComment("if");u.appendChild(b),s.push(m(b)),i.appendChild(u);let h=document.createComment("if");return i.appendChild(h),s.push(d(h)),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let g of s)try{g()}catch(y){}}}}]}),c=r.shadowRoot??r;j(c);let p=document.createComment("if");if(c.appendChild(p),o.push(f(p)),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(l.mount){let s=l.mount(i,r);if(typeof s==="function")o.push(s)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var ip=z("sui-metachip",{props:{label:v.string({default:"",reflect:!0}),value:v.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.label=n.label,t.value=n.value;let a=r.shadowRoot??r;j(a);let m=document.createElement("span");m.setAttribute("class","k"),m.setAttribute("part","label");let d=document.createElement("slot");d.setAttribute("name","label");let f=document.createTextNode("");d.appendChild(f),o.push(R(f,()=>t.label())),m.appendChild(d),a.appendChild(m);let c=document.createElement("span");c.setAttribute("class","v"),c.setAttribute("part","value");let p=document.createElement("slot"),l=document.createTextNode("");return p.appendChild(l),o.push(R(l,()=>t.value())),c.appendChild(p),a.appendChild(c),t}});var jo=0,vr=typeof CSS<"u"&&typeof CSS.supports==="function"&&CSS.supports("anchor-name","--x"),vn=S({mount(e,n){let r=e,o=++jo,t=`--sui-popover-${o}`,a=`sui-popover-panel-${o}`;n.style.setProperty("--sui-popover-anchor",t);let m=r.triggerSlot(),d=r.panelEl();if(!d)return;d.id=a,we(r.triggerSlot,(i)=>{for(let s of i)s.setAttribute("popovertarget",a),s.setAttribute("aria-haspopup","dialog"),s.setAttribute("aria-controls",a),s.setAttribute("aria-expanded",r.open()?"true":"false")});function f(i){let u=i.newState==="open",b=n.hasAttribute("open");if(u&&!b)n.setAttribute("open","");else if(!u&&b)n.removeAttribute("open");let g=m?.assignedElements({flatten:!0})??[];for(let y of g)y.setAttribute("aria-expanded",u?"true":"false");if(n.emit("sui-toggle",{open:u}),u&&!vr)p()}Z(d,"toggle",f),n.expose({show:()=>d.showPopover?.(),hide:()=>d.hidePopover?.(),toggle:()=>typeof d.togglePopover==="function"?void d.togglePopover():d.matches?.(":popover-open")?d.hidePopover?.():d.showPopover?.()});let c=T(()=>{let i=r.open(),s=d.matches?.(":popover-open")??!1;if(i&&!s)d.showPopover?.();else if(!i&&s)d.hidePopover?.()});function p(){let u=(m?.assignedElements({flatten:!0})??[])[0];if(!u||!d)return;let b=u.getBoundingClientRect(),h=r.placement();d.style.top="",d.style.left="",d.style.right="",d.style.bottom="";let g=6;switch(h){case"top":d.style.bottom=`${window.innerHeight-b.top+g}px`,d.style.left=`${b.left}px`;break;case"top-end":d.style.bottom=`${window.innerHeight-b.top+g}px`,d.style.right=`${window.innerWidth-b.right}px`;break;case"bottom-end":d.style.top=`${b.bottom+g}px`,d.style.right=`${window.innerWidth-b.right}px`;break;case"left":d.style.right=`${window.innerWidth-b.left+g}px`,d.style.top=`${b.top}px`;break;case"left-end":d.style.right=`${window.innerWidth-b.left+g}px`,d.style.bottom=`${window.innerHeight-b.bottom}px`;break;case"right":d.style.left=`${b.right+g}px`,d.style.top=`${b.top}px`;break;case"right-end":d.style.left=`${b.right+g}px`,d.style.bottom=`${window.innerHeight-b.bottom}px`;break;case"bottom":default:d.style.top=`${b.bottom+g}px`,d.style.left=`${b.left}px`}}function l(){if(!r.open())return;p()}if(!vr)Z(window,"resize",l),Z(window,"scroll",l,{capture:!0,passive:!0});return()=>{c()}}});var pp=z("sui-popover",{props:{open:v.bool({default:!1,reflect:!0}),placement:v.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom",reflect:!0})},shadow:!0,styles:`
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
    opacity: 0;
    transform: translateY(-4px);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  .panel:popover-open > .surface {
    opacity: 1;
    transform: translateY(0);
  }

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
    .panel > .surface { transition: none; transform: none; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.placement=n.placement;let a=null;if(vn&&typeof vn.setup==="function"){if(e.scope=t,a=vn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");d.setAttribute("name","trigger"),D(t,"triggerSlot",d),m.appendChild(d);let f=document.createElement("div");f.setAttribute("class","panel"),f.setAttribute("part","panel"),f.setAttribute("popover","auto"),D(t,"panelEl",f),f.setAttribute("role","dialog"),f.setAttribute("aria-modal","false");let c=document.createElement("div");c.setAttribute("class","surface");let p=document.createElement("slot");if(c.appendChild(p),f.appendChild(c),m.appendChild(f),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(l.mount){let s=l.mount(i,r);if(typeof s==="function")o.push(s)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var Se=[{id:"ship-light",label:"SHIP Light",dark:!1,swatches:["oklch(0.99 0.001 106.423)","oklch(1 0 0)","oklch(0.67 0.16 58)","oklch(0.61 0.23 303)"]},{id:"ship-dark",label:"SHIP Dark",dark:!0,swatches:["oklch(0.18 0.01 250)","oklch(0.18 0.01 50)","oklch(0.77 0.16 70)","oklch(0.3 0.05 300)"]},{id:"paper",label:"Paper",dark:!1,swatches:["oklch(0.97 0.014 80)","oklch(0.99 0.010 80)","oklch(0.55 0.14 35)","oklch(0.48 0.09 55)"]},{id:"halo",label:"Halo (Soft)",dark:!1,swatches:["oklch(0.92 0.008 230)","oklch(0.92 0.008 230)","oklch(0.62 0.14 50)","oklch(0.55 0.05 230)"]},{id:"console",label:"Console",dark:!0,swatches:["oklch(0.13 0.008 220)","oklch(0.16 0.010 220)","oklch(0.78 0.20 145)","oklch(0.78 0.16 195)"]},{id:"contrast",label:"High Contrast",dark:!0,swatches:["oklch(0.09 0 0)","oklch(0.15 0 0)","oklch(0.87 0.18 85)","oklch(0.99 0 0)"]},{id:"glass",label:"Glass",dark:!1,swatches:["oklch(0.95 0.04 230)","oklch(0.99 0.005 240)","oklch(0.62 0.16 50)","oklch(0.62 0.14 280)"]}],gr=Object.fromEntries(Se.map((e)=>[e.id,e]));var gn=(e)=>typeof e==="string"&&(e in gr),To=(e)=>gr[e]?.dark??!1;function Me(e,n=document.documentElement){let r=gn(e)?e:"ship-light",o=To(r);if(n.dataset.theme=r,n.classList.toggle("dark",o),typeof document<"u"&&typeof CustomEvent<"u")document.dispatchEvent(new CustomEvent("ship:theme:change",{detail:{id:r,dark:o}}));return r}function Pe(){if(typeof localStorage>"u")return"ship-light";try{let e=localStorage.getItem("ship-site-theme");return gn(e)?e:"ship-light"}catch{return"ship-light"}}function yr(e){if(typeof localStorage>"u")return;try{localStorage.setItem("ship-site-theme",e)}catch{}try{fetch("/api/settings",{method:"PUT",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:"shell.theme",value:e})}).catch(()=>{})}catch{}}var Oo=[{label:"Default",ids:["ship-light","ship-dark"]},{label:"Texture",ids:["paper","halo"]},{label:"Effects",ids:["glass"]},{label:"Special",ids:["console","contrast"]}],yn=new Map(Se.map((e)=>[e.id,e]));function Do(e){let n=e.shadowRoot;if(!n)return;if(n.querySelector("link[data-sui-theme-preview-styles]"))return;let r=document.createElement("link");r.rel="stylesheet",r.href="/api/assets/view-tokens.css",r.setAttribute("data-sui-theme-preview-styles",""),n.appendChild(r)}var xn=S({scope:(e)=>{let n=e,r=N(Pe()),o=N(!1),t=N(Oo.map((i)=>({label:i.label,themes:i.ids.map((s)=>yn.get(s)).filter((s)=>!!s)}))),a=q(()=>yn.get(r())?.swatches[2]??"var(--primary, #3b82f6)"),m=q(()=>yn.get(r())?.label??"Theme"),d={host:null,panel:null};function f(){let i=d.panel;if(!i)return;if(o())i.hidePopover?.();else i.showPopover?.()}function c(i){let s=Me(i);if(n.persist())yr(s);r.set(s);let u=d.host;if(u)u.dispatchEvent(new CustomEvent("sui-theme-change",{detail:{id:s,dark:yn.get(s)?.dark??!1},bubbles:!0,composed:!0}));d.panel?.hidePopover?.()}let p=W(null),l=W(null);return{activeTheme:r,panelOpen:o,groups:t,dotColor:a,triggerLabel:m,togglePanel:f,selectTheme:c,triggerBtn:p,panelEl:l,_internals:d}},mount(e,n){let r=e;r._internals.host=n,Do(n),Me(Pe());let o=T(()=>{let c=r.panelEl();if(r._internals.panel=c,!c)return;let p=(l)=>{let s=l.newState==="open";if(r.panelOpen()!==s)r.panelOpen.set(s);if(s)t()};return c.addEventListener("toggle",p),()=>c.removeEventListener("toggle",p)});function t(){let c=r.panelEl(),p=r.triggerBtn();if(!c||!p)return;let l=p.getBoundingClientRect();c.style.top=`${l.bottom+4}px`,c.style.left=`${Math.max(8,l.right-c.offsetWidth)}px`}function a(c){let p=c.detail?.id;if(!gn(p))return;if(r.activeTheme()!==p)r.activeTheme.set(p)}document.addEventListener("ship:theme:change",a);function m(c){let p=r.panelEl();if(!p)return;let l=Array.from(p.querySelectorAll(".card"));if(!l.length)return;let i=p.getRootNode(),s=l.findIndex((u)=>u===i.activeElement);switch(c.key){case"ArrowDown":c.preventDefault(),l[(s+1+l.length)%l.length]?.focus();break;case"ArrowUp":c.preventDefault(),l[(s-1+l.length)%l.length]?.focus();break;case"Home":c.preventDefault(),l[0]?.focus();break;case"End":c.preventDefault(),l[l.length-1]?.focus();break;case"Escape":c.preventDefault(),p.hidePopover?.();break}}let d=T(()=>{let c=r.panelEl();if(!c)return;return c.addEventListener("keydown",m),()=>c.removeEventListener("keydown",m)}),f=T(()=>{if(!r.panelOpen())return;requestAnimationFrame(()=>{let c=r.panelEl();if(!c)return;let p=Array.from(c.querySelectorAll(".card"));(p.find((i)=>i.getAttribute("aria-selected")==="true")??p[0])?.focus()})});return()=>{document.removeEventListener("ship:theme:change",a),o(),d(),f()}}});var xp=z("sui-theme-picker",{props:{placement:v.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom-end",reflect:!0}),"label-style":v.union(["text","compact"],{default:"text",reflect:!0}),persist:v.bool({default:!0,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.placement=n.placement,t.labelStyle=n.labelStyle,t.persist=n.persist;let a=null;if(xn&&typeof xn.setup==="function"){if(e.scope=t,a=xn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(g)=>Y({anchor:g,items:()=>t.groups(),key:(y,x)=>{let E=Object.create(t);return E.g=y,E.__i=x,((w)=>y.label)(E)},render:(y,x)=>{let E=Object.create(t);return E.g=y,E.__i=x,E.__v=N(0),((w)=>{let A=(F)=>Y({anchor:F,items:()=>w.g.themes,key:(U,J)=>{let Q=Object.create(w);return Q.t=U,Q.__i=J,((X)=>U.id)(Q)},render:(U,J)=>{let Q=Object.create(w);return Q.t=U,Q.__i=J,Q.__v=N(0),((X)=>{let ot=document.createDocumentFragment(),K=[],_=document.createElement("button");_.setAttribute("class","card"),_.setAttribute("role","option"),_.setAttribute("type","button"),K.push(k(_,"data-theme-card",()=>(X.__v(),X.t.id))),K.push(k(_,"aria-label",()=>(X.__v(),X.t.label))),K.push(k(_,"aria-selected",()=>(X.__v(),X.activeTheme()===X.t.id?"true":"false"))),K.push(k(_,"tabindex",()=>(X.__v(),X.activeTheme()===X.t.id?"0":"-1"))),K.push(I(_,"click",function(){let ft=()=>X.selectTheme(X.t.id);return typeof ft==="function"?ft.bind(void 0):()=>{}}(),{modifiers:[]}));let ct=document.createElement("span");ct.setAttribute("class","tc-bg"),K.push(k(ct,"data-theme-preview",()=>(X.__v(),X.t.id))),ct.setAttribute("aria-hidden","true");let Ge=document.createElement("span");Ge.setAttribute("class","tc-overlay"),ct.appendChild(Ge);let xt=document.createElement("span");xt.setAttribute("class","tc-card");let Tt=document.createElement("span");Tt.setAttribute("class","tc-card-head");let ue=document.createElement("span");ue.setAttribute("class","tc-eyebrow");let Pr=document.createTextNode("THEME");ue.appendChild(Pr),Tt.appendChild(ue);let de=document.createElement("span");de.setAttribute("class","tc-heading");let Re=document.createTextNode("");de.appendChild(Re),K.push(R(Re,()=>(X.__v(),X.t.label))),Tt.appendChild(de),xt.appendChild(Tt);let Ot=document.createElement("span");Ot.setAttribute("class","tc-typo");let ce=document.createElement("span");ce.setAttribute("class","tc-body");let Nr=document.createTextNode("The quick brown fox");ce.appendChild(Nr),Ot.appendChild(ce);let pe=document.createElement("span");pe.setAttribute("class","tc-mono");let Ue=document.createTextNode("");pe.appendChild(Ue),K.push(R(Ue,()=>(X.__v(),"{ build: ok }"))),Ot.appendChild(pe),xt.appendChild(Ot);let pt=document.createElement("span");pt.setAttribute("class","tc-controls");let fe=document.createElement("span");fe.setAttribute("class","tc-cta");let Hr=document.createTextNode("Action");fe.appendChild(Hr),pt.appendChild(fe);let me=document.createElement("span");me.setAttribute("class","tc-ghost");let Fr=document.createTextNode("Cancel");me.appendChild(Fr),pt.appendChild(me);let be=document.createElement("span");be.setAttribute("class","tc-chip tc-chip-accent");let Br=document.createTextNode("New");be.appendChild(Br),pt.appendChild(be);let he=document.createElement("span");he.setAttribute("class","tc-chip tc-chip-muted");let Gr=document.createTextNode("Idle");he.appendChild(Gr),pt.appendChild(he),xt.appendChild(pt),ct.appendChild(xt),_.appendChild(ct);let Dt=document.createElement("span");Dt.setAttribute("class","row");let ve=document.createElement("span");ve.setAttribute("class","name");let Ye=document.createTextNode("");ve.appendChild(Ye),K.push(R(Ye,()=>(X.__v(),X.t.label))),Dt.appendChild(ve);let ge=document.createElement("span");return ge.setAttribute("class","radio"),ge.setAttribute("aria-hidden","true"),Dt.appendChild(ge),_.appendChild(Dt),ot.appendChild(_),{nodes:Array.from(ot.childNodes),cleanup:()=>{for(let ft of K)try{ft()}catch(Xe){}},update:(ft,Xe)=>{X.t=ft,X.__i=Xe,X.__v.set((Rr)=>Rr+1)}}})(Q)}}),L=document.createDocumentFragment(),C=[],O=document.createElement("section");O.setAttribute("class","group"),C.push(k(O,"aria-label",()=>(w.__v(),w.g.label)));let P=document.createElement("div");P.setAttribute("class","group-h");let M=document.createTextNode("");P.appendChild(M),C.push(R(M,()=>(w.__v(),w.g.label))),O.appendChild(P);let H=document.createElement("div");H.setAttribute("class","group-rows");let B=document.createComment("for");return H.appendChild(B),C.push(A(B)),O.appendChild(H),L.appendChild(O),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let F of C)try{F()}catch(U){}},update:(F,U)=>{w.g=F,w.__i=U,w.__v.set((J)=>J+1)}}})(E)}}),d=r.shadowRoot??r;j(d);let f=document.createElement("button");f.setAttribute("class","trigger"),f.setAttribute("part","trigger"),f.setAttribute("type","button"),D(t,"triggerBtn",f),f.setAttribute("aria-label","Choose theme"),f.setAttribute("aria-haspopup","dialog"),o.push(k(f,"aria-expanded",()=>t.panelOpen()?"true":"false")),o.push(I(f,"click",function(){let g=()=>t.togglePanel();return typeof g==="function"?g.bind(void 0):()=>{}}(),{modifiers:[]}));let c=document.createElement("span");c.setAttribute("class","dot"),c.setAttribute("part","dot"),o.push(T(()=>{let g=t.dotColor();if(g==null)c.style.removeProperty("background");else c.style.setProperty("background",String(g))})),c.setAttribute("aria-hidden","true"),f.appendChild(c);let p=document.createElement("span");p.setAttribute("class","label"),p.setAttribute("part","label");let l=document.createTextNode("");p.appendChild(l),o.push(R(l,()=>t.triggerLabel())),f.appendChild(p);let i=document.createElement("svg");i.setAttribute("class","caret"),i.setAttribute("viewbox","0 0 10 6"),i.setAttribute("aria-hidden","true");let s=document.createElement("path");s.setAttribute("d","M1 1l4 4 4-4"),s.setAttribute("fill","none"),s.setAttribute("stroke","currentColor"),s.setAttribute("stroke-width","1.5"),s.setAttribute("stroke-linecap","round"),s.setAttribute("stroke-linejoin","round"),i.appendChild(s),f.appendChild(i),d.appendChild(f);let u=document.createElement("div");u.setAttribute("class","panel"),u.setAttribute("part","panel"),u.setAttribute("popover","auto"),D(t,"panelEl",u);let b=document.createElement("div");b.setAttribute("class","grid"),b.setAttribute("part","grid"),b.setAttribute("role","listbox"),b.setAttribute("aria-label","Theme");let h=document.createComment("for");if(b.appendChild(h),o.push(m(h)),u.appendChild(b),d.appendChild(u),a!=null){let g=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(g){let y=new Proxy(t,{get(x,E){return x[E]},set(x,E,w){let A=x[E];if(A!=null&&typeof A.set==="function"){if(typeof w==="function")A.set(()=>w);else A.set(w);return!0}return!1}});if(g.mount){let x=g.mount(y,r);if(typeof x==="function")o.push(x)}if(g.unmount)o.push(()=>g.unmount(y))}}return t}});var wn=S({mount(e,n){let r=e,o=T(()=>{let m=r.toggleBtn(),d=r.collapsed();if(m)m.setAttribute("aria-label",d?"Show rail":"Hide rail")}),t=()=>{let m=!r.collapsed();r.collapsed.set(m),n.dispatchEvent(new CustomEvent("sui-rail:toggle",{detail:{collapsed:m},bubbles:!0,composed:!0}))},a=T(()=>{let m=r.toggleBtn();if(!m)return;return m.addEventListener("click",t),()=>m.removeEventListener("click",t)});return()=>{o(),a()}}});var Lp=z("sui-rail",{props:{collapsed:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.collapsed=n.collapsed;let a=null;if(wn&&typeof wn.setup==="function"){if(e.scope=t,a=wn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("nav");d.setAttribute("class","rail-nav"),d.setAttribute("part","nav");let f=document.createElement("slot");d.appendChild(f),m.appendChild(d);let c=document.createElement("div");c.setAttribute("class","rail-footer"),c.setAttribute("part","footer");let p=document.createElement("button");p.setAttribute("type","button"),p.setAttribute("class","project-badge"),p.setAttribute("part","project-badge"),p.setAttribute("data-rail","project-badge"),p.setAttribute("aria-label","Switch project"),c.appendChild(p);let l=document.createElement("div");l.setAttribute("class","project-popover-container");let i=document.createElement("div");i.setAttribute("class","project-popover"),i.setAttribute("part","project-popover"),i.setAttribute("data-rail","project-popover"),i.setAttribute("hidden","");let s=document.createElement("div");s.setAttribute("class","project-popover-title");let u=document.createTextNode("Projects");s.appendChild(u),i.appendChild(s);let b=document.createElement("div");b.setAttribute("data-rail","project-list"),i.appendChild(b);let h=document.createElement("div");h.setAttribute("class","project-popover-sep"),i.appendChild(h);let g=document.createElement("div");g.setAttribute("class","project-popover-row add"),g.setAttribute("data-rail","add-project");let y=document.createElement("span"),x=document.createTextNode("+ Add project");y.appendChild(x),g.appendChild(y),i.appendChild(g),l.appendChild(i),c.appendChild(l);let E=document.createElement("div");E.setAttribute("class","footer-actions");let w=document.createElement("button");w.setAttribute("type","button"),w.setAttribute("class","footer-btn"),w.setAttribute("part","launcher-btn"),w.setAttribute("data-rail","launcher"),w.setAttribute("aria-label","Open launcher");let A=document.createElement("sui-icon");A.setAttribute("name","folder"),A.setAttribute("size","sm"),w.appendChild(A),E.appendChild(w);let L=document.createElement("button");L.setAttribute("type","button"),L.setAttribute("class","footer-btn"),L.setAttribute("part","create-btn"),L.setAttribute("data-rail","create"),L.setAttribute("aria-label","Create new");let C=document.createElement("sui-icon");C.setAttribute("name","plus"),C.setAttribute("size","sm"),L.appendChild(C),E.appendChild(L);let O=document.createElement("button");if(O.setAttribute("type","button"),O.setAttribute("class","toggle-btn"),O.setAttribute("part","toggle"),O.setAttribute("data-rail","toggle"),O.setAttribute("aria-label","Collapse rail"),E.appendChild(O),c.appendChild(E),m.appendChild(c),a!=null){let P=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(P){let M=new Proxy(t,{get(H,B){return H[B]},set(H,B,F){let U=H[B];if(U!=null&&typeof U.set==="function"){if(typeof F==="function")U.set(()=>F);else U.set(F);return!0}return!1}});if(P.mount){let H=P.mount(M,r);if(typeof H==="function")o.push(H)}if(P.unmount)o.push(()=>P.unmount(M))}}return t}});var Tp=z("sui-rail-item",{props:{href:v.string({default:""}),label:v.string({default:""}),icon:v.string({default:""}),active:v.bool({default:!1,reflect:!0}),badge:v.string({default:""}),disabled:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.href=n.href,t.label=n.label,t.icon=n.icon,t.active=n.active,t.badge=n.badge,t.disabled=n.disabled;let a=r.shadowRoot??r;j(a);let m=document.createElement("a");m.setAttribute("class","item"),m.setAttribute("part","item"),D(t,"itemEl",m),o.push(k(m,"href",()=>t.href()||"#")),o.push(k(m,"aria-current",()=>t.active()?"page":null)),o.push(k(m,"aria-disabled",()=>t.disabled()?"true":null)),o.push(k(m,"title",()=>t.label()));let d=document.createElement("span");d.setAttribute("class","icon-wrap"),d.setAttribute("part","icon-wrap");let f=document.createElement("sui-icon");o.push(k(f,"name",()=>t.icon())),f.setAttribute("size","sm"),d.appendChild(f),m.appendChild(d);let c=document.createElement("span");c.setAttribute("class","label"),c.setAttribute("part","label");let p=document.createTextNode("");c.appendChild(p),o.push(R(p,()=>t.label())),m.appendChild(c);let l=document.createElement("span");l.setAttribute("class","badge"),l.setAttribute("part","badge"),D(t,"badgeEl",l);let i=document.createTextNode("");return l.appendChild(i),o.push(R(i,()=>t.badge())),m.appendChild(l),a.appendChild(m),t}});var Sp=z("sui-rail-section",{props:{label:v.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.label=n.label;let a=r.shadowRoot??r;j(a);let m=document.createElement("span");m.setAttribute("class","heading"),m.setAttribute("part","heading");let d=document.createTextNode("");m.appendChild(d),o.push(R(d,()=>t.label())),a.appendChild(m);let f=document.createElement("slot");return a.appendChild(f),t}});var Np=z("sui-status-bar",{props:{branch:v.string({default:""}),theme:v.string({default:""}),online:v.bool({default:!1,reflect:!0}),agents:v.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.branch=n.branch,t.theme=n.theme,t.online=n.online,t.agents=n.agents;let a=r.shadowRoot??r;j(a);let m=document.createElement("span");m.setAttribute("class","dot"),m.setAttribute("part","dot"),m.setAttribute("aria-hidden","true"),a.appendChild(m);let d=document.createElement("span");d.setAttribute("class","text"),d.setAttribute("part","branch");let f=document.createTextNode("");d.appendChild(f),o.push(R(f,()=>t.branch())),a.appendChild(d);let c=document.createElement("span");c.setAttribute("class","text"),c.setAttribute("part","agents");let p=document.createTextNode("");c.appendChild(p),o.push(R(p,()=>t.agents())),a.appendChild(c);let l=document.createElement("span");l.setAttribute("class","slot-wrap");let i=document.createElement("slot");i.setAttribute("name","start"),l.appendChild(i),a.appendChild(l);let s=document.createElement("span");s.setAttribute("class","spacer"),a.appendChild(s);let u=document.createElement("span");u.setAttribute("class","slot-wrap");let b=document.createElement("slot");b.setAttribute("name","end"),u.appendChild(b),a.appendChild(u);let h=document.createElement("span");h.setAttribute("class","text"),h.setAttribute("part","theme");let g=document.createTextNode("");return h.appendChild(g),o.push(R(g,()=>t.theme())),a.appendChild(h),t}});var So=["a[href]","area[href]","button:not([disabled])",'input:not([disabled]):not([type="hidden"])',"select:not([disabled])","textarea:not([disabled])","iframe","object","embed","summary",'[contenteditable]:not([contenteditable="false"])','[tabindex]:not([tabindex^="-"])'].join(",");function gt(e){return Array.from(e.querySelectorAll(So)).filter((n)=>{if(n.hasAttribute("disabled"))return!1;if(n.getAttribute("aria-hidden")==="true")return!1;return!0})}class zt{saved=null;save(){this.saved=document.activeElement??null}restore(){let e=this.saved;if(this.saved=null,e&&typeof e.focus==="function")e.focus()}}var Lt=0,Ne=null,He=null;function An(){if(typeof document>"u")return;if(Lt+=1,Lt>1)return;let e=document.body;Ne=e.style.overflow,He=e.style.paddingRight;let n=typeof window<"u"?window.innerWidth-document.documentElement.clientWidth:0;if(n>0){let r=parseFloat(getComputedStyle(e).paddingRight||"0")||0;e.style.paddingRight=`${r+n}px`}e.style.overflow="hidden"}function yt(){if(typeof document>"u")return;if(Lt===0)return;if(Lt-=1,Lt>0)return;let e=document.body;e.style.overflow=Ne??"",e.style.paddingRight=He??"",Ne=null,He=null}var Mo=0,En=S({mount(e,n){let r=e,o=++Mo,t=`sui-dialog-header-${o}`,a=`sui-dialog-body-${o}`,m=r.dialogEl(),d=r.closeBtn(),f=r.sentinelStart(),c=r.sentinelEnd(),p=r.headerEl(),l=r.bodyEl();if(!m)return;if(p)p.id=t;if(l)l.id=a;m.setAttribute("aria-labelledby",t),m.setAttribute("aria-describedby",a);let i=!1,s=new zt;function u(){m.setAttribute("role",r.roleOverride()||"dialog"),m.setAttribute("aria-modal","true")}u();function b(){let P=gt(m).filter((H)=>!H.classList.contains("sentinel"));return[...gt(n),...P]}function h(){let P=n.querySelector("[autofocus]")??m.querySelector("[autofocus]");if(P){P.focus();return}let M=b(),H=M.find((B)=>B!==d)??M[0];if(H)H.focus();else m.focus()}function g(){s.save();try{m.showModal()}catch{}if(!i)An(),i=!0;queueMicrotask(()=>h()),n.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function y(){if(m.open)m.close()}let x=()=>{if(r.open())r.open.set(!1);if(i)yt(),i=!1;s.restore(),n.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},E=(P)=>{if(!r.dismissible())return;if(P.target===m)r.open.set(!1)},w=()=>{n.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),r.open.set(!1)},A=()=>{let P=b();if(P.length)P[P.length-1].focus()},L=()=>{let P=b();if(P.length)P[0].focus()};m.addEventListener("close",x),m.addEventListener("click",E),d?.addEventListener("click",w),f?.addEventListener("focus",A),c?.addEventListener("focus",L);let C=T(()=>{let P=r.open();if(n.setAttribute("data-state",P?"open":"closed"),P&&!m.open)g();else if(!P&&m.open)y()}),O=T(()=>{u()});return n.show=()=>r.open.set(!0),n.showModal=()=>r.open.set(!0),n.close=()=>r.open.set(!1),()=>{if(i)yt();m.removeEventListener("close",x),m.removeEventListener("click",E),d?.removeEventListener("click",w),f?.removeEventListener("focus",A),c?.removeEventListener("focus",L),C(),O()}}});var Xp=z("sui-dialog",{props:{open:v.bool({default:!1,reflect:!0}),size:v.union(["sm","md","lg","full"],{default:"md",reflect:!0}),backdrop:v.union(["default","blur","solid"],{default:"default",reflect:!0}),dismissible:v.bool({default:!0}),"no-close-button":v.bool({default:!1,reflect:!0}),"no-header":v.bool({default:!1,reflect:!0}),"no-footer":v.bool({default:!1,reflect:!0}),"role-override":v.union(["dialog","alertdialog"],{default:"dialog",reflect:!0})},shadow:!0,styles:`
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
    opacity: 0;
    /* Closed state: don't intercept clicks. The dialog stays in the
       layout tree so the open-state CSS transition has something to
       animate from, but \`position: fixed; inset: 0\` would otherwise
       cover the entire viewport with an invisible (opacity: 0) but
       click-eating overlay — clicks would land on the dialog instead
       of the underlying app surface. \`pointer-events: none\` lifts that
       restriction whenever the dialog isn't open. */
    pointer-events: none;
    transform: translateY(8px) scale(0.96);
    transition:
      opacity var(--duration-medium, 200ms) var(--ease-out, ease-out),
      transform var(--duration-medium, 200ms) var(--ease-out, ease-out);
  }
  :host([size='full']) dialog { max-height: 90vh; }
  dialog[open] {
    opacity: 1;
    transform: translateY(0) scale(1);
    pointer-events: auto;
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
    dialog { transition: none; transform: none !important; opacity: 1; }
    dialog::backdrop { transition: none; }
  }
  @media (forced-colors: active) {
    dialog { border: 1px solid CanvasText; }
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.size=n.size,t.backdrop=n.backdrop,t.dismissible=n.dismissible,t.noCloseButton=n.noCloseButton,t.noHeader=n.noHeader,t.noFooter=n.noFooter,t.roleOverride=n.roleOverride;let a=null;if(En&&typeof En.setup==="function"){if(e.scope=t,a=En.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("dialog");d.setAttribute("part","dialog"),D(t,"dialogEl",d);let f=document.createElement("span");f.setAttribute("tabindex","0"),f.setAttribute("class","sentinel"),f.setAttribute("aria-hidden","true"),D(t,"sentinelStart",f),d.appendChild(f);let c=document.createElement("button");c.setAttribute("class","close"),c.setAttribute("part","close"),c.setAttribute("type","button"),c.setAttribute("aria-label","Close dialog"),D(t,"closeBtn",c);let p=document.createElement("svg");p.setAttribute("viewbox","0 0 16 16"),p.setAttribute("fill","none"),p.setAttribute("stroke","currentColor"),p.setAttribute("stroke-width","1.75"),p.setAttribute("stroke-linecap","round"),p.setAttribute("stroke-linejoin","round"),p.setAttribute("aria-hidden","true"),p.setAttribute("focusable","false");let l=document.createElement("path");l.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),p.appendChild(l),c.appendChild(p),d.appendChild(c);let i=document.createElement("div");i.setAttribute("class","header"),i.setAttribute("part","header"),D(t,"headerEl",i);let s=document.createElement("slot");s.setAttribute("name","header"),i.appendChild(s),d.appendChild(i);let u=document.createElement("div");u.setAttribute("class","body"),u.setAttribute("part","body"),D(t,"bodyEl",u);let b=document.createElement("slot");u.appendChild(b),d.appendChild(u);let h=document.createElement("div");h.setAttribute("class","footer"),h.setAttribute("part","footer");let g=document.createElement("slot");g.setAttribute("name","footer"),h.appendChild(g),d.appendChild(h);let y=document.createElement("span");if(y.setAttribute("tabindex","0"),y.setAttribute("class","sentinel"),y.setAttribute("aria-hidden","true"),D(t,"sentinelEnd",y),d.appendChild(y),m.appendChild(d),a!=null){let x=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(x){let E=new Proxy(t,{get(w,A){return w[A]},set(w,A,L){let C=w[A];if(C!=null&&typeof C.set==="function"){if(typeof L==="function")C.set(()=>L);else C.set(L);return!0}return!1}});if(x.mount){let w=x.mount(E,r);if(typeof w==="function")o.push(w)}if(x.unmount)o.push(()=>x.unmount(E))}}return t}});var Po=0,kn=S({mount(e,n){let r=e,o=++Po,t=`sui-drawer-header-${o}`,a=`sui-drawer-body-${o}`,m=r.dialogEl(),d=r.closeBtn(),f=r.sentinelStart(),c=r.sentinelEnd(),p=r.headerEl(),l=r.bodyEl();if(!m)return;if(p)p.id=t;if(l)l.id=a;m.setAttribute("aria-labelledby",t),m.setAttribute("aria-describedby",a);let i=!1,s=new zt;function u(){let C=gt(m).filter((P)=>!P.classList.contains("sentinel"));return[...gt(n),...C]}function b(){let C=n.querySelector("[autofocus]")??m.querySelector("[autofocus]");if(C){C.focus();return}let O=u(),P=O.find((M)=>M!==d)??O[0];if(P)P.focus();else m.focus()}function h(){s.save();try{m.showModal()}catch{}if(!i)An(),i=!0;queueMicrotask(()=>b()),n.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function g(){if(m.open)m.close()}let y=()=>{if(r.open())r.open.set(!1);if(i)yt(),i=!1;s.restore(),n.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},x=(C)=>{if(!r.dismissible())return;if(C.target===m)r.open.set(!1)},E=()=>{n.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),r.open.set(!1)},w=()=>{let C=u();if(C.length)C[C.length-1].focus()},A=()=>{let C=u();if(C.length)C[0].focus()};m.addEventListener("close",y),m.addEventListener("click",x),d?.addEventListener("click",E),f?.addEventListener("focus",w),c?.addEventListener("focus",A);let L=T(()=>{let C=r.open();if(n.setAttribute("data-state",C?"open":"closed"),C&&!m.open)h();else if(!C&&m.open)g()});return n.show=()=>r.open.set(!0),n.close=()=>r.open.set(!1),()=>{if(i)yt();m.removeEventListener("close",y),m.removeEventListener("click",x),d?.removeEventListener("click",E),f?.removeEventListener("focus",w),c?.removeEventListener("focus",A),L()}}});var Vp=z("sui-drawer",{props:{open:v.bool({default:!1,reflect:!0}),side:v.union(["right","left"],{default:"right",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),dismissible:v.bool({default:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.side=n.side,t.size=n.size,t.dismissible=n.dismissible;let a=null;if(kn&&typeof kn.setup==="function"){if(e.scope=t,a=kn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("dialog");d.setAttribute("part","dialog"),d.setAttribute("role","dialog"),d.setAttribute("aria-modal","true"),D(t,"dialogEl",d);let f=document.createElement("span");f.setAttribute("tabindex","0"),f.setAttribute("class","sentinel"),f.setAttribute("aria-hidden","true"),D(t,"sentinelStart",f),d.appendChild(f);let c=document.createElement("div");c.setAttribute("class","header"),c.setAttribute("part","header"),D(t,"headerEl",c);let p=document.createElement("div");p.setAttribute("class","header-content");let l=document.createElement("div");l.setAttribute("class","title");let i=document.createElement("slot");i.setAttribute("name","title"),l.appendChild(i),p.appendChild(l);let s=document.createElement("div");s.setAttribute("class","description");let u=document.createElement("slot");u.setAttribute("name","description"),s.appendChild(u),p.appendChild(s),c.appendChild(p);let b=document.createElement("button");b.setAttribute("class","close"),b.setAttribute("part","close"),b.setAttribute("type","button"),b.setAttribute("aria-label","Close drawer"),D(t,"closeBtn",b);let h=document.createElement("svg");h.setAttribute("viewbox","0 0 16 16"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","1.75"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true"),h.setAttribute("focusable","false");let g=document.createElement("path");g.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),h.appendChild(g),b.appendChild(h),c.appendChild(b),d.appendChild(c);let y=document.createElement("div");y.setAttribute("class","body"),y.setAttribute("part","body"),D(t,"bodyEl",y);let x=document.createElement("slot");y.appendChild(x),d.appendChild(y);let E=document.createElement("div");E.setAttribute("class","footer"),E.setAttribute("part","footer");let w=document.createElement("slot");w.setAttribute("name","footer"),E.appendChild(w),d.appendChild(E);let A=document.createElement("span");if(A.setAttribute("tabindex","0"),A.setAttribute("class","sentinel"),A.setAttribute("aria-hidden","true"),D(t,"sentinelEnd",A),d.appendChild(A),m.appendChild(d),a!=null){let L=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(L){let C=new Proxy(t,{get(O,P){return O[P]},set(O,P,M){let H=O[P];if(H!=null&&typeof H.set==="function"){if(typeof M==="function")H.set(()=>M);else H.set(M);return!0}return!1}});if(L.mount){let O=L.mount(C,r);if(typeof O==="function")o.push(O)}if(L.unmount)o.push(()=>L.unmount(C))}}return t}});var No=0,Cn=S({mount(e,n){let r=e,o=`sui-dropdown-menu-panel-${++No}`,t=r.triggerWrap(),a=r.triggerSlot(),m=r.panelEl();if(!m)return;m.id=o;let d="",f=null;function c(A){if(f)clearTimeout(f);d+=A.toLowerCase(),f=setTimeout(()=>{d=""},350);let L=p(),C=L.indexOf(document.activeElement);for(let O=0;O<L.length;O++){let P=(C+1+O)%L.length;if((L[P].textContent??"").trim().toLowerCase().startsWith(d))return P}return-1}function p(){return Array.from(n.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((A)=>!A.hasAttribute("disabled"))}function l(A){let L=p();if(!L.length)return;let C=(A%L.length+L.length)%L.length;for(let O of L)O.setAttribute("tabindex","-1");L[C].setAttribute("tabindex","0"),L[C].focus()}function i(){return p().indexOf(document.activeElement)}function s(){p().forEach((A,L)=>{if(!A.hasAttribute("role"))A.setAttribute("role","menuitem");if(!A.hasAttribute("tabindex"))A.setAttribute("tabindex",L===0?"0":"-1")})}function u(){let A=a?.assignedElements({flatten:!0})??[];for(let L of A)L.setAttribute("aria-haspopup","menu"),L.setAttribute("aria-controls",o),L.setAttribute("aria-expanded",r.open()?"true":"false")}function b(){let A=a?.assignedElements({flatten:!0})?.[0];if(A&&m){let L=A.getBoundingClientRect();m.style.top=`${L.bottom+4}px`,m.style.left=`${L.left}px`}m?.showPopover?.()}function h(){m?.hidePopover?.()}let g=()=>{if(r.open())h();else b()},y=(A)=>{let C=A.newState==="open";if(C!==r.open())r.open.set(C);if(n.setAttribute("data-state",C?"open":"closed"),u(),C)queueMicrotask(()=>{s(),l(0)})},x=(A)=>{let L=A.target.closest("[data-value]");if(!L)return;n.emit("sui-select",{value:L.dataset.value??""}),h()},E=(A)=>{if(!r.open())return;let L=p();if(!L.length)return;let C=i();if(A.key==="ArrowDown")A.preventDefault(),l(C+1);else if(A.key==="ArrowUp")A.preventDefault(),l(C-1);else if(A.key==="Home")A.preventDefault(),l(0);else if(A.key==="End")A.preventDefault(),l(L.length-1);else if(A.key==="Escape")A.preventDefault(),h();else if(A.key==="Tab")h();else if((A.key==="Enter"||A.key===" ")&&C>=0)A.preventDefault(),L[C].click();else if(A.key.length===1&&!A.ctrlKey&&!A.metaKey){let O=c(A.key);if(O>=0)A.preventDefault(),l(O)}};if(t)Z(t,"click",g);if(Z(m,"toggle",y),Z(m,"click",x),Z(n,"keydown",E),a)Z(a,"slotchange",u);Z(m,"slotchange",s);let w=T(()=>{let A=r.open();n.setAttribute("data-state",A?"open":"closed"),u()});return()=>{if(f)clearTimeout(f);w()}}});var ef=z("sui-dropdown-menu",{props:{open:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open;let a=null;if(Cn&&typeof Cn.setup==="function"){if(e.scope=t,a=Cn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("span");d.setAttribute("class","trigger-wrap"),D(t,"triggerWrap",d);let f=document.createElement("slot");f.setAttribute("name","trigger"),D(t,"triggerSlot",f),d.appendChild(f),m.appendChild(d);let c=document.createElement("div");c.setAttribute("class","panel"),c.setAttribute("part","panel"),c.setAttribute("popover","auto"),c.setAttribute("role","menu"),D(t,"panelEl",c);let p=document.createElement("slot");if(D(t,"itemSlot",p),c.appendChild(p),m.appendChild(c),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(l.mount){let s=l.mount(i,r);if(typeof s==="function")o.push(s)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var Ho=0,Ln=S({mount(e,n){let r=e,o=`sui-context-menu-panel-${++Ho}`,t=r.triggerSlot(),a=r.panelEl();if(!a)return;a.id=o;let m="",d=null;function f(w){if(d)clearTimeout(d);m+=w.toLowerCase(),d=setTimeout(()=>{m=""},350);let A=c(),L=A.indexOf(document.activeElement);for(let C=0;C<A.length;C++){let O=(L+1+C)%A.length;if((A[O].textContent??"").trim().toLowerCase().startsWith(m))return O}return-1}function c(){return Array.from(n.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((w)=>!w.hasAttribute("disabled"))}function p(w){let A=c();if(!A.length)return;let L=(w%A.length+A.length)%A.length;for(let C of A)C.setAttribute("tabindex","-1");A[L].setAttribute("tabindex","0"),A[L].focus()}function l(){return c().indexOf(document.activeElement)}function i(){c().forEach((w,A)=>{if(!w.hasAttribute("role"))w.setAttribute("role","menuitem");if(!w.hasAttribute("tabindex"))w.setAttribute("tabindex",A===0?"0":"-1")})}function s(w,A){if(a)a.style.top=`${A}px`,a.style.left=`${w}px`;a?.showPopover?.()}function u(){a?.hidePopover?.()}n.expose({show:s,close:u});let b=(w)=>{w.preventDefault(),s(w.clientX,w.clientY)},h=(w)=>{let L=w.newState==="open";if(L!==r.open())r.open.set(L);if(n.setAttribute("data-state",L?"open":"closed"),L)n.emit("sui-open"),queueMicrotask(()=>{i(),p(0)});else n.emit("sui-close")},g=(w)=>{let A=w.target.closest("[data-value]");if(!A)return;n.emit("sui-select",{value:A.dataset.value??""}),u()},y=(w)=>{if(!r.open())return;let A=c();if(!A.length)return;let L=l();if(w.key==="ArrowDown")w.preventDefault(),p(L+1);else if(w.key==="ArrowUp")w.preventDefault(),p(L-1);else if(w.key==="Home")w.preventDefault(),p(0);else if(w.key==="End")w.preventDefault(),p(A.length-1);else if(w.key==="Escape")w.preventDefault(),u();else if(w.key==="Tab")u();else if((w.key==="Enter"||w.key===" ")&&L>=0)w.preventDefault(),A[L].click();else if(w.key.length===1&&!w.ctrlKey&&!w.metaKey){let C=f(w.key);if(C>=0)w.preventDefault(),p(C)}},x=()=>{let w=t?.assignedElements({flatten:!0})??[];for(let A of w)A.addEventListener("contextmenu",b)};if(t)Z(t,"slotchange",x);x(),Z(a,"toggle",h),Z(a,"click",g),Z(n,"keydown",y);let E=T(()=>{n.setAttribute("data-state",r.open()?"open":"closed")});return()=>{let w=t?.assignedElements({flatten:!0})??[];for(let A of w)A.removeEventListener("contextmenu",b);if(d)clearTimeout(d);E()}}});var uf=z("sui-context-menu",{props:{open:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open;let a=null;if(Ln&&typeof Ln.setup==="function"){if(e.scope=t,a=Ln.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");d.setAttribute("name","trigger"),D(t,"triggerSlot",d),m.appendChild(d);let f=document.createElement("div");f.setAttribute("class","panel"),f.setAttribute("part","panel"),f.setAttribute("popover","auto"),f.setAttribute("role","menu"),f.setAttribute("aria-label","Context menu"),D(t,"panelEl",f);let c=document.createElement("slot");if(D(t,"itemSlot",c),f.appendChild(c),m.appendChild(f),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var Fo=0,zn=8;function Bo(e,n,r){let o=e.getBoundingClientRect();switch(n.style.top="",n.style.left="",n.style.right="",n.style.bottom="",r){case"top":n.style.bottom=`${window.innerHeight-o.top+zn}px`,n.style.left=`${o.left+o.width/2}px`,n.style.transform="translateX(-50%)";break;case"left":n.style.right=`${window.innerWidth-o.left+zn}px`,n.style.top=`${o.top}px`;break;case"right":n.style.left=`${o.right+zn}px`,n.style.top=`${o.top}px`;break;case"bottom":default:n.style.top=`${o.bottom+zn}px`,n.style.left=`${o.left+o.width/2}px`,n.style.transform="translateX(-50%)"}}var jn=S({mount(e,n){let r=e,o=`sui-hover-card-${++Fo}`,t=r.triggerSlot(),a=r.panelEl();if(!a)return;a.id=o;let m=null,d=null;function f(){if(m)clearTimeout(m),m=null;if(d)clearTimeout(d),d=null}function c(){let y=t?.assignedElements({flatten:!0})??[];for(let x of y)x.setAttribute("aria-describedby",o)}function p(){f();let y=t?.assignedElements({flatten:!0})?.[0];if(y&&a)Bo(y,a,r.placement()||"bottom");a?.showPopover?.()}function l(){f(),a?.hidePopover?.()}n.show=p,n.hide=l;let i=(y)=>{let E=y.newState==="open";if(E!==r.open())r.open.set(E);n.setAttribute("data-state",E?"open":"closed"),n.dispatchEvent(new CustomEvent(E?"sui-open":"sui-close",{bubbles:!0,composed:!0}))},s=()=>{f(),m=setTimeout(()=>p(),r.showDelay()??300)},u=()=>{f(),d=setTimeout(()=>l(),r.hideDelay()??100)},b=()=>{f(),p()},h=()=>{f(),d=setTimeout(()=>l(),r.hideDelay()??100)};a.addEventListener("toggle",i),t?.addEventListener("slotchange",c),c(),n.addEventListener("pointerenter",s),n.addEventListener("pointerleave",u),n.addEventListener("focusin",b),n.addEventListener("focusout",h);let g=T(()=>{n.setAttribute("data-state",r.open()?"open":"closed")});return()=>{f(),a.removeEventListener("toggle",i),t?.removeEventListener("slotchange",c),n.removeEventListener("pointerenter",s),n.removeEventListener("pointerleave",u),n.removeEventListener("focusin",b),n.removeEventListener("focusout",h),g()}}});var bf=z("sui-hover-card",{props:{open:v.bool({default:!1,reflect:!0}),placement:v.union(["top","bottom","left","right"],{default:"bottom",reflect:!0}),"show-delay":v.number({default:300}),"hide-delay":v.number({default:100})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.open=n.open,t.placement=n.placement,t.showDelay=n.showDelay,t.hideDelay=n.hideDelay;let a=null;if(jn&&typeof jn.setup==="function"){if(e.scope=t,a=jn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");d.setAttribute("name","trigger"),D(t,"triggerSlot",d),m.appendChild(d);let f=document.createElement("div");f.setAttribute("class","panel"),f.setAttribute("part","panel"),f.setAttribute("popover","auto"),D(t,"panelEl",f);let c=document.createElement("slot");if(f.appendChild(c),m.appendChild(f),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var Tn=S({mount(e,n){let r=e,o=()=>Array.from(n.querySelectorAll(":scope > sui-accordion-item")),t=()=>o().filter((p)=>!p.hasAttribute("disabled")),a=()=>{let p=r.value();return new Set(p?p.split(",").filter(Boolean):[])},m=()=>{let p=a();for(let l of o()){let i=l.getAttribute("value")||"";if(p.has(i))l.setAttribute("open","");else l.removeAttribute("open")}},d=T(()=>{r.value(),m()}),f=(p)=>{let l=p;l.stopPropagation();let i=l.detail?.value,s=a();if(r.type()==="single")r.value.set(s.has(i)?"":i);else{if(s.has(i))s.delete(i);else s.add(i);r.value.set(Array.from(s).join(","))}n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:r.value()},bubbles:!0,composed:!0}))},c=(p)=>{let l=p;l.stopPropagation();let i=t();if(!i.length)return;let s=l.target?.closest("sui-accordion-item"),u=s?i.indexOf(s):-1,b;switch(l.detail?.direction){case"next":b=i[(u+1)%i.length];break;case"prev":b=i[(u-1+i.length)%i.length];break;case"first":b=i[0];break;case"last":b=i[i.length-1];break}if(b)b.shadowRoot?.querySelector("button")?.focus()};return n.addEventListener("sui-accordion-item-toggle",f),n.addEventListener("sui-accordion-nav",c),()=>{d(),n.removeEventListener("sui-accordion-item-toggle",f),n.removeEventListener("sui-accordion-nav",c)}}});var wf=z("sui-accordion",{props:{type:v.string({default:"single",reflect:!0}),value:v.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host { display: block; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.type=n.type,t.value=n.value;let a=null;if(Tn&&typeof Tn.setup==="function"){if(e.scope=t,a=Tn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");if(m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});var Go=0,On=S({mount(e,n){let r=e,o=++Go,t=`sui-acc-trigger-${o}`,a=`sui-acc-content-${o}`,m=T(()=>{let l=r.triggerBtn(),i=r.contentWrap();if(l)l.id=t,l.setAttribute("aria-controls",a);if(i)i.id=a,i.setAttribute("aria-labelledby",t)}),d=T(()=>{let l=r.triggerBtn();if(l)l.setAttribute("aria-expanded",r.open()?"true":"false")}),f=()=>{if(r.disabled())return;n.dispatchEvent(new CustomEvent("sui-accordion-item-toggle",{detail:{value:r.value()},bubbles:!0,composed:!0}))},c=(l)=>{if(l.key==="ArrowDown"||l.key==="ArrowUp")l.preventDefault(),n.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:l.key==="ArrowDown"?"next":"prev"},bubbles:!0,composed:!0}));else if(l.key==="Home"||l.key==="End")l.preventDefault(),n.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:l.key==="Home"?"first":"last"},bubbles:!0,composed:!0}))},p=T(()=>{let l=r.triggerBtn();if(!l)return;return l.addEventListener("click",f),l.addEventListener("keydown",c),()=>{l.removeEventListener("click",f),l.removeEventListener("keydown",c)}});return()=>{m(),d(),p()}}});var zf=z("sui-accordion-item",{props:{value:v.string({default:"",reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),open:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.disabled=n.disabled,t.open=n.open;let a=null;if(On&&typeof On.setup==="function"){if(e.scope=t,a=On.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("button");d.setAttribute("class","trigger"),d.setAttribute("part","trigger"),D(t,"triggerBtn",d),o.push(k(d,"disabled",()=>t.disabled()));let f=document.createElement("span");f.setAttribute("class","trigger-label");let c=document.createElement("slot");c.setAttribute("name","trigger"),f.appendChild(c),d.appendChild(f);let p=document.createElement("svg");p.setAttribute("class","chevron"),p.setAttribute("part","chevron"),p.setAttribute("viewbox","0 0 16 16"),p.setAttribute("fill","none"),p.setAttribute("stroke","currentColor"),p.setAttribute("stroke-width","1.75"),p.setAttribute("stroke-linecap","round"),p.setAttribute("stroke-linejoin","round"),p.setAttribute("aria-hidden","true");let l=document.createElement("path");l.setAttribute("d","M4 6l4 4 4-4"),p.appendChild(l),d.appendChild(p),m.appendChild(d);let i=document.createElement("div");i.setAttribute("class","content-wrap"),i.setAttribute("part","content"),D(t,"contentWrap",i),i.setAttribute("role","region");let s=document.createElement("div");s.setAttribute("class","content-inner");let u=document.createElement("div");u.setAttribute("class","content-body");let b=document.createElement("slot");if(u.appendChild(b),s.appendChild(u),i.appendChild(s),m.appendChild(i),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let g=new Proxy(t,{get(y,x){return y[x]},set(y,x,E){let w=y[x];if(w!=null&&typeof w.set==="function"){if(typeof E==="function")w.set(()=>E);else w.set(E);return!0}return!1}});if(h.mount){let y=h.mount(g,r);if(typeof y==="function")o.push(y)}if(h.unmount)o.push(()=>h.unmount(g))}}return t}});var xr=`
sui-list-tile[variant='row'] + sui-list-tile[variant='row'] {
  border-top: 1px solid var(--border-subtle);
}
sui-list-tile[variant='row'][selected] + sui-list-tile[variant='row'],
sui-list-tile[variant='row'] + sui-list-tile[variant='row'][selected] {
  border-top: none;
}
`,wr=new WeakSet;function Ro(e){if(wr.has(e))return;wr.add(e);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let r=new CSSStyleSheet;r.replaceSync(xr),e.adoptedStyleSheets=[...e.adoptedStyleSheets,r];return}}catch{}let n=e.createElement("style");n.setAttribute("data-sui-list-tile-dividers",""),n.textContent=xr,e.head.appendChild(n)}var Dn=S({mount(e,n){let r=e;Ro(n.ownerDocument);let o=()=>{if(r.navigable())return"link";if(r.interactive())return"button";return"listitem"},t=T(()=>{r.navigable(),r.interactive(),n.setAttribute("role",o())}),a=T(()=>{n.setAttribute("aria-selected",r.selected()?"true":"false")}),m=T(()=>{n.setAttribute("aria-disabled",r.disabled()?"true":"false")}),d=T(()=>{let i=r.tileEl();if(!i)return;let s=r.interactive(),u=r.disabled();if(s&&!u)i.setAttribute("tabindex","0");else i.removeAttribute("tabindex")}),f=()=>{if(r.disabled()||!r.interactive())return!1;let i=new CustomEvent("sui-select",{detail:{},bubbles:!0,composed:!0,cancelable:!0});return n.dispatchEvent(i),i.defaultPrevented},c=(i)=>{if(!r.interactive())return;if(f()&&r.navigable())i.preventDefault()},p=(i)=>{if(!r.interactive()||r.disabled())return;if(r.navigable()&&i.key==="Enter")return;if(i.key==="Enter"||i.key===" ")i.preventDefault(),f()},l=T(()=>{let i=r.tileEl();if(!i)return;return i.addEventListener("click",c),i.addEventListener("keydown",p),()=>{i.removeEventListener("click",c),i.removeEventListener("keydown",p)}});return()=>{t(),a(),m(),d(),l()}}});var Mf=z("sui-list-tile",{props:{variant:v.string({default:"row",reflect:!0}),interactive:v.bool({default:!1,reflect:!0}),navigable:v.string({default:"",reflect:!0}),selected:v.bool({default:!1,reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),density:v.string({default:"normal",reflect:!0}),tone:v.string({default:"default",reflect:!0}),target:v.string({default:"",reflect:!0}),rel:v.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.variant=n.variant,t.interactive=n.interactive,t.navigable=n.navigable,t.selected=n.selected,t.disabled=n.disabled,t.density=n.density,t.tone=n.tone,t.target=n.target,t.rel=n.rel;let a=null;if(Dn&&typeof Dn.setup==="function"){if(e.scope=t,a=Dn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(c)=>G({anchor:c,test:()=>{if(t.navigable())return 0;return 1},branches:[()=>{let p=document.createDocumentFragment(),l=[],i=document.createElement("a");i.setAttribute("class","tile"),i.setAttribute("part","button"),D(t,"tileEl",i),l.push(k(i,"href",()=>t.navigable())),l.push(k(i,"target",()=>t.target()||null)),l.push(k(i,"rel",()=>t.rel()||null));let s=document.createElement("div");s.setAttribute("class","row"),s.setAttribute("part","row");let u=document.createElement("slot");u.setAttribute("name","leading"),s.appendChild(u);let b=document.createElement("span");b.setAttribute("class","primary"),b.setAttribute("part","primary");let h=document.createElement("slot");b.appendChild(h),s.appendChild(b);let g=document.createElement("span");g.setAttribute("part","trailing");let y=document.createElement("slot");y.setAttribute("name","trailing"),g.appendChild(y),s.appendChild(g),i.appendChild(s);let x=document.createElement("div");x.setAttribute("class","secondary"),x.setAttribute("part","secondary");let E=document.createElement("slot");return E.setAttribute("name","meta"),x.appendChild(E),i.appendChild(x),p.appendChild(i),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let w of l)try{w()}catch(A){}}}},()=>{let p=document.createDocumentFragment(),l=[],i=document.createElement("div");i.setAttribute("class","tile"),i.setAttribute("part","button"),D(t,"tileEl",i);let s=document.createElement("div");s.setAttribute("class","row"),s.setAttribute("part","row");let u=document.createElement("slot");u.setAttribute("name","leading"),s.appendChild(u);let b=document.createElement("span");b.setAttribute("class","primary"),b.setAttribute("part","primary");let h=document.createElement("slot");b.appendChild(h),s.appendChild(b);let g=document.createElement("span");g.setAttribute("part","trailing");let y=document.createElement("slot");y.setAttribute("name","trailing"),g.appendChild(y),s.appendChild(g),i.appendChild(s);let x=document.createElement("div");x.setAttribute("class","secondary"),x.setAttribute("part","secondary");let E=document.createElement("slot");return E.setAttribute("name","meta"),x.appendChild(E),i.appendChild(x),p.appendChild(i),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let w of l)try{w()}catch(A){}}}}]}),d=r.shadowRoot??r;j(d);let f=document.createComment("if");if(d.appendChild(f),o.push(m(f)),a!=null){let c=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(c){let p=new Proxy(t,{get(l,i){return l[i]},set(l,i,s){let u=l[i];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(c.mount){let l=c.mount(p,r);if(typeof l==="function")o.push(l)}if(c.unmount)o.push(()=>c.unmount(p))}}return t}});var Sn=S({mount(e,n){let r=e,o=!1,t=(f)=>{f.textContent=o?"Hide details":"Show details",f.setAttribute("aria-expanded",o?"true":"false")},a=T(()=>{let f=r.detailsSlot();if(!f)return;let c=()=>{if(f.assignedNodes({flatten:!0}).some((l)=>l.nodeType===Node.ELEMENT_NODE||l.nodeType===Node.TEXT_NODE&&(l.textContent??"").trim().length>0))n.setAttribute("data-has-details","");else{n.removeAttribute("data-has-details"),n.removeAttribute("data-details-open"),o=!1;let l=r.detailsToggleBtn();if(l)t(l)}};return f.addEventListener("slotchange",c),c(),()=>f.removeEventListener("slotchange",c)}),m=T(()=>{let f=r.detailsToggleBtn();if(!f)return;t(f);let c=()=>{if(o=!o,o)n.setAttribute("data-details-open","");else n.removeAttribute("data-details-open");t(f)};return f.addEventListener("click",c),()=>f.removeEventListener("click",c)}),d=T(()=>{let f=r.actionSlot();if(!f)return;let c=()=>{let p=f.assignedElements().find((l)=>l.getAttribute("slot")==="retry");if(p)requestAnimationFrame(()=>p.focus?.())};return f.addEventListener("slotchange",c),()=>f.removeEventListener("slotchange",c)});return()=>{a(),m(),d()}}});var Gf=z("sui-error-state",{props:{title:v.string({default:""}),body:v.string({default:""}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:v.union(["default","inline"],{default:"default",reflect:!0}),icon:v.string({default:"alert-circle"})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.title=n.title,t.body=n.body,t.size=n.size,t.variant=n.variant,t.icon=n.icon;let a=null;if(Sn&&typeof Sn.setup==="function"){if(e.scope=t,a=Sn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","box"),d.setAttribute("part","box"),d.setAttribute("role","alert");let f=document.createElement("div");f.setAttribute("class","icon"),f.setAttribute("part","icon"),f.setAttribute("aria-hidden","true");let c=document.createElement("sui-icon");o.push(k(c,"name",()=>t.icon())),c.setAttribute("size","md"),f.appendChild(c),d.appendChild(f);let p=document.createElement("div");p.setAttribute("class","copy");let l=document.createElement("div");l.setAttribute("class","title"),l.setAttribute("part","title");let i=document.createElement("slot");i.setAttribute("name","title");let s=document.createTextNode("");i.appendChild(s),o.push(R(s,()=>t.title()||"Something went wrong")),l.appendChild(i),p.appendChild(l);let u=document.createElement("div");u.setAttribute("class","desc"),u.setAttribute("part","desc");let b=document.createElement("slot"),h=document.createTextNode("");b.appendChild(h),o.push(R(h,()=>t.body())),u.appendChild(b),p.appendChild(u);let g=document.createElement("button");g.setAttribute("class","details-toggle"),D(t,"detailsToggleBtn",g),g.setAttribute("type","button"),p.appendChild(g);let y=document.createElement("div");y.setAttribute("class","details"),D(t,"detailsPanel",y);let x=document.createElement("slot");x.setAttribute("name","details"),D(t,"detailsSlot",x),y.appendChild(x),p.appendChild(y),d.appendChild(p);let E=document.createElement("div");E.setAttribute("class","actions"),E.setAttribute("part","actions");let w=document.createElement("slot");if(w.setAttribute("name","action"),D(t,"actionSlot",w),E.appendChild(w),d.appendChild(E),m.appendChild(d),a!=null){let A=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(A){let L=new Proxy(t,{get(C,O){return C[O]},set(C,O,P){let M=C[O];if(M!=null&&typeof M.set==="function"){if(typeof P==="function")M.set(()=>P);else M.set(P);return!0}return!1}});if(A.mount){let C=A.mount(L,r);if(typeof C==="function")o.push(C)}if(A.unmount)o.push(()=>A.unmount(L))}}return t}});var Mn=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","progressbar");let o=T(()=>{let t=Math.max(1,Math.floor(r.total()||0)),a=Math.max(1,Math.min(t,Math.floor(r.current()||1))),m=r.label();if(n.setAttribute("aria-valuemin","1"),n.setAttribute("aria-valuemax",String(t)),n.setAttribute("aria-valuenow",String(a)),m)n.setAttribute("aria-label",m);else n.setAttribute("aria-label",`Step ${a} of ${t}`);let d=r.dotsEl();if(!d)return;d.textContent="";for(let f=1;f<=t;f++){let c=document.createElement("span");c.className="dot",c.setAttribute("part","dot"),c.setAttribute("data-state",f===a?"active":f<a?"done":"pending"),c.setAttribute("aria-hidden","true"),d.appendChild(c)}});return()=>{o()}}});var Jf=z("sui-stepper",{props:{total:v.number({default:4,reflect:!0}),current:v.number({default:1,reflect:!0}),tone:v.string({default:"",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),label:v.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.total=n.total,t.current=n.current,t.tone=n.tone,t.size=n.size,t.label=n.label;let a=null;if(Mn&&typeof Mn.setup==="function"){if(e.scope=t,a=Mn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");if(d.setAttribute("class","dots"),D(t,"dotsEl",d),m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});var Uo=new Set(["done","failed","pending","running"]);function Yo(e){return e.split(",").map((n)=>n.trim().toLowerCase()).filter((n)=>n.length>0).map((n)=>Uo.has(n)?n:"pending")}var Pn=S({mount(e,n){let r=e,o=T(()=>{let t=Yo(r.values()||""),a=r.rowEl();if(!a)return;if(a.textContent="",t.length===0){a.removeAttribute("role"),a.removeAttribute("aria-label");return}a.setAttribute("role","img"),a.setAttribute("aria-label",`${t.length} step outcome${t.length===1?"":"s"}`);for(let m of t){let d=document.createElement("span");d.className="seg",d.setAttribute("part","seg"),d.setAttribute("data-state",m),a.appendChild(d)}});return()=>o()}});var Kf=z("sui-step-segments",{props:{values:v.string({default:"",reflect:!0}),size:v.union(["sm","md"],{default:"sm",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.values=n.values,t.size=n.size;let a=null;if(Pn&&typeof Pn.setup==="function"){if(e.scope=t,a=Pn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("span");if(d.setAttribute("class","row"),d.setAttribute("part","row"),D(t,"rowEl",d),m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});var Nn=S({mount(e,n){if(!n.hasAttribute("role"))n.setAttribute("role","list")}});var rm=z("sui-timeline",{shadow:!0,styles:`
  :host { display: block; position: relative; }
  .list { display: flex; flex-direction: column; gap: 4px; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(Nn&&typeof Nn.setup==="function"){if(e.scope=t,a=Nn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","list"),d.setAttribute("part","list");let f=document.createElement("slot");if(d.appendChild(f),m.appendChild(d),a!=null){let c=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(c){let p=new Proxy(t,{get(l,i){return l[i]},set(l,i,s){let u=l[i];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(c.mount){let l=c.mount(p,r);if(typeof l==="function")o.push(l)}if(c.unmount)o.push(()=>c.unmount(p))}}return t}});var Hn=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","listitem");let o=T(()=>{if(r.interactive()){if(n.setAttribute("role","button"),!n.hasAttribute("tabindex"))n.setAttribute("tabindex","0")}else n.setAttribute("role","listitem"),n.removeAttribute("tabindex")}),t=()=>{n.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},a=()=>{if(r.interactive())t()},m=(d)=>{if(!r.interactive())return;if(d.key==="Enter"||d.key===" ")d.preventDefault(),t()};return n.addEventListener("click",a),n.addEventListener("keydown",m),()=>{o(),n.removeEventListener("click",a),n.removeEventListener("keydown",m)}}});var um=z("sui-timeline-item",{props:{tone:v.string({default:"default",reflect:!0}),at:v.string({default:"",reflect:!0}),interactive:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.tone=n.tone,t.at=n.at,t.interactive=n.interactive;let a=null;if(Hn&&typeof Hn.setup==="function"){if(e.scope=t,a=Hn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(h)=>G({anchor:h,test:()=>{if(t.at())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),y=[],x=document.createElement("span");x.setAttribute("class","at"),x.setAttribute("part","at");let E=document.createTextNode("");return x.appendChild(E),y.push(R(E,()=>t.at())),g.appendChild(x),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let w of y)try{w()}catch(A){}}}}]}),d=r.shadowRoot??r;j(d);let f=document.createElement("div");f.setAttribute("class","gutter");let c=document.createElement("span");c.setAttribute("class","dot"),c.setAttribute("part","dot"),f.appendChild(c),d.appendChild(f);let p=document.createElement("div");p.setAttribute("class","body"),p.setAttribute("part","body");let l=document.createElement("div");l.setAttribute("class","head");let i=document.createElement("slot");i.setAttribute("name","title"),l.appendChild(i);let s=document.createComment("if");l.appendChild(s),o.push(m(s)),p.appendChild(l);let u=document.createElement("div");u.setAttribute("class","content");let b=document.createElement("slot");if(u.appendChild(b),p.appendChild(u),d.appendChild(p),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let g=new Proxy(t,{get(y,x){return y[x]},set(y,x,E){let w=y[x];if(w!=null&&typeof w.set==="function"){if(typeof E==="function")w.set(()=>E);else w.set(E);return!0}return!1}});if(h.mount){let y=h.mount(g,r);if(typeof y==="function")o.push(y)}if(h.unmount)o.push(()=>h.unmount(g))}}return t}});var Xo=["Waking daemon","Warming routes","Fetching state","Hydrating shell"];function Ar(e){let n=e.trim();if(!n)return Xo;return n.split("|").map((r)=>r.trim()).filter(Boolean)}var Fn=S({mount(e,n){let r=e,o=r.msgEl();if(!o)return;let t=0;(()=>{let d=Ar(r.messages());o.textContent=d[t]??"",o.style.opacity="1"})();let m=setInterval(()=>{o.style.opacity="0",requestAnimationFrame(()=>{let d=Ar(r.messages());t=(t+1)%d.length,o.textContent=d[t]??"",o.style.opacity="1"})},1400);return()=>clearInterval(m)}});var bm=z("sui-loading",{props:{messages:v.string({default:""}),compact:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.messages=n.messages,t.compact=n.compact;let a=null;if(Fn&&typeof Fn.setup==="function"){if(e.scope=t,a=Fn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","stage"),d.setAttribute("part","stage"),d.setAttribute("aria-hidden","true");let f=document.createElement("div");f.setAttribute("class","rocket");let c=document.createElement("svg");c.setAttribute("viewbox","0 0 32 40"),c.setAttribute("fill","none"),c.setAttribute("xmlns","http://www.w3.org/2000/svg");let p=document.createElement("path");p.setAttribute("d","M16 2 C20 6 22 11 22 16 H10 C10 11 12 6 16 2 Z"),p.setAttribute("fill","currentColor"),c.appendChild(p);let l=document.createElement("rect");l.setAttribute("x","10"),l.setAttribute("y","16"),l.setAttribute("width","12"),l.setAttribute("height","14"),l.setAttribute("rx","2"),l.setAttribute("fill","currentColor"),l.setAttribute("opacity","0.9"),c.appendChild(l);let i=document.createElement("circle");i.setAttribute("cx","16"),i.setAttribute("cy","20"),i.setAttribute("r","2.5"),i.setAttribute("fill","var(--background, #fff)"),i.setAttribute("opacity","0.85"),c.appendChild(i);let s=document.createElement("path");s.setAttribute("d","M10 24 L4 32 L10 30 Z"),s.setAttribute("fill","currentColor"),s.setAttribute("opacity","0.75"),c.appendChild(s);let u=document.createElement("path");u.setAttribute("d","M22 24 L28 32 L22 30 Z"),u.setAttribute("fill","currentColor"),u.setAttribute("opacity","0.75"),c.appendChild(u);let b=document.createElement("rect");b.setAttribute("x","13"),b.setAttribute("y","29"),b.setAttribute("width","6"),b.setAttribute("height","2"),b.setAttribute("rx","0.5"),b.setAttribute("fill","var(--background, #000)"),b.setAttribute("opacity","0.5"),c.appendChild(b),f.appendChild(c),d.appendChild(f);let h=document.createElement("div");h.setAttribute("class","flame"),d.appendChild(h);let g=document.createElement("div");g.setAttribute("class","particle p1"),d.appendChild(g);let y=document.createElement("div");y.setAttribute("class","particle p2"),d.appendChild(y);let x=document.createElement("div");x.setAttribute("class","particle p3"),d.appendChild(x),m.appendChild(d);let E=document.createElement("div");if(E.setAttribute("class","message"),E.setAttribute("part","message"),D(t,"msgEl",E),m.appendChild(E),a!=null){let w=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(w){let A=new Proxy(t,{get(L,C){return L[C]},set(L,C,O){let P=L[C];if(P!=null&&typeof P.set==="function"){if(typeof O==="function")P.set(()=>O);else P.set(O);return!0}return!1}});if(w.mount){let L=w.mount(A,r);if(typeof L==="function")o.push(L)}if(w.unmount)o.push(()=>w.unmount(A))}}return t}});function Er(e){let n=document.getElementById("ship-view-root");if(!n)return null;let r=e.target;if(!r||!n.contains(r))return null;if(r.closest("sui-edit-overlay"))return null;return r}function kr(e){let n=e.tagName.toLowerCase(),r=e.id||null,o=n;if(r)o=`#${r}`;else if(e.classList.length>0)o=`${n}.${Array.from(e.classList).slice(0,2).join(".")}`;return{tag:n,selector:o,id:r}}var Bn=S({mount(e,n){let r=e,o=r.boxEl(),t=r.labelEl(),a=null;function m(u){if(!o||!t)return;if(!u){o.classList.remove("visible"),t.classList.remove("visible");return}o.style.cssText=`top:${u.top}px;left:${u.left}px;width:${u.width}px;height:${u.height}px`,o.classList.add("visible"),t.style.cssText=`top:${u.top}px;left:${u.left}px`,t.textContent=`<${u.tag}${u.id?`#${u.id}`:""}>`,t.classList.add("visible")}let d=(u)=>{if(a)return;a=requestAnimationFrame(()=>{a=null;let b=Er(u);if(!b){m(null);return}let h=b.getBoundingClientRect(),{tag:g,id:y}=kr(b);m({top:h.top,left:h.left,width:h.width,height:h.height,tag:g,id:y})})},f=()=>{m(null)},c=(u)=>{let b=Er(u);if(!b)return;u.preventDefault(),u.stopPropagation();let{tag:h,selector:g,id:y}=kr(b);(n.closest("sui-shell")??n.closest("ship-shell"))?.dispatchEvent(new CustomEvent("sui:chat:annotate",{detail:{tag:h,selector:g,id:y,preview:(b.textContent??"").trim().slice(0,60)},bubbles:!0})),window.ship?.chat?.open?.()};function p(){let u=document.getElementById("ship-view-root");if(!u)return;u.addEventListener("pointermove",d,!0),u.addEventListener("pointerleave",f,!0),u.addEventListener("click",c,!0)}function l(){let u=document.getElementById("ship-view-root");if(!u)return;u.removeEventListener("pointermove",d,!0),u.removeEventListener("pointerleave",f,!0),u.removeEventListener("click",c,!0)}function i(){let u=document.body.classList.contains("edit-mode");if(n.classList.toggle("active",u),u)p();else if(l(),m(null),a)cancelAnimationFrame(a),a=null}i();let s=new MutationObserver(()=>i());return s.observe(document.body,{attributes:!0,attributeFilter:["class"]}),()=>{if(s.disconnect(),l(),a)cancelAnimationFrame(a),a=null}}});var wm=z("sui-edit-overlay",{shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(Bn&&typeof Bn.setup==="function"){if(e.scope=t,a=Bn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","box"),d.setAttribute("part","box"),D(t,"boxEl",d),m.appendChild(d);let f=document.createElement("div");f.setAttribute("class","label"),f.setAttribute("part","label"),D(t,"labelEl",f),m.appendChild(f);let c=document.createElement("div");c.setAttribute("class","hint"),c.setAttribute("part","hint");let p=document.createTextNode(`
  Edit mode — hover an element, click to annotate.
  `);c.appendChild(p);let l=document.createElement("kbd"),i=document.createTextNode("⌘⇧E");l.appendChild(i),c.appendChild(l);let s=document.createTextNode(` to exit.
`);if(c.appendChild(s),m.appendChild(c),a!=null){let u=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(u){let b=new Proxy(t,{get(h,g){return h[g]},set(h,g,y){let x=h[g];if(x!=null&&typeof x.set==="function"){if(typeof y==="function")x.set(()=>y);else x.set(y);return!0}return!1}});if(u.mount){let h=u.mount(b,r);if(typeof h==="function")o.push(h)}if(u.unmount)o.push(()=>u.unmount(b))}}return t}});function qo(e=Date.now()){let n=e,r=[,,,,,,,,,,];for(let a=9;a>=0;a--)r[a]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[n%32],n=Math.floor(n/32);let o=Array(16),t=typeof globalThis<"u"?globalThis.crypto:void 0;if(t?.getRandomValues){let a=new Uint8Array(16);t.getRandomValues(a);for(let m=0;m<16;m++)o[m]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[a[m]%32]}else for(let a=0;a<16;a++)o[a]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[Math.floor(Math.random()*32)];return r.join("")+o.join("")}function Cr(e){let n={};if(e.viewId)n.view_id=e.viewId;if(e.selector)n.selector=e.selector;if(e.rect)n.rect=e.rect;return{id:qo(),kind:e.kind,target:n,note:e.note,created_at:new Date().toISOString(),author_kind:"human"}}var Jo="/api/feedback",Gn=S({scope:(e,n)=>{let{viewId:r,mode:o,endpoint:t,staging:a}=e,m=N([]),d=N(null),f=N(null),c=N(""),p={},l=q(()=>{let w=f();if(!w)return!1;return Math.abs(w.curX-w.startX)>4||Math.abs(w.curY-w.startY)>4}),i=q(()=>{let w=f();if(!w)return"";let A=Math.min(w.startX,w.curX),L=Math.min(w.startY,w.curY),C=Math.abs(w.curX-w.startX),O=Math.abs(w.curY-w.startY);return`left:${A}px;top:${L}px;width:${C}px;height:${O}px`}),s=q(()=>{let w=d();if(!w)return"";let A=Math.min(w.x+12,1e4),L=Math.min(w.y+12,1e4);return`left:${A}px;top:${L}px`}),u=q(()=>m().map((w,A)=>({id:w.id,kind:w.kind,index:A,rect:w.kind==="region"&&w.target.rect?w.target.rect.join(","):"",pos:p[w.id]??{x:0,y:0}})));async function b(w){let A=t()||Jo;try{let L=await fetch(A,{method:"POST",credentials:"include",headers:{"Content-Type":"application/json"},body:JSON.stringify(w)});if(!L.ok){n.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:`http_${L.status}`,id:w.id},bubbles:!0,composed:!0}));return}m.set(m().filter((C)=>C.id!==w.id)),n.dispatchEvent(new CustomEvent("sui-annotation-sent",{detail:{id:w.id,status:L.status},bubbles:!0,composed:!0}))}catch(L){n.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:"network_error",message:String(L),id:w.id},bubbles:!0,composed:!0}))}}function h(w){let A=d();if(!A)return;let L=w.trim();if(!L){d.set(null),c.set("");return}let C=Cr({viewId:r()||void 0,kind:A.kind,selector:A.kind==="point"?A.selector:void 0,rect:A.kind==="region"?A.rect:void 0,note:L});if(p[C.id]={x:A.x,y:A.y},m.set([...m(),C]),n.dispatchEvent(new CustomEvent("sui-annotation-staged",{detail:C,bubbles:!0,composed:!0})),d.set(null),c.set(""),!a())b(C)}let g=()=>{h(c())},y=()=>{d.set(null),c.set("")},x=(w)=>{if(w.key==="Escape")y();else if(w.key==="Enter"&&(w.metaKey||w.ctrlKey)){let A=w.currentTarget;h(A.value)}},E=W(null);return{staged:m,pending:d,drag:f,noteText:c,dragVisible:l,dragRectStyle:i,popoverStyle:s,markers:u,overlayEl:E,confirmNote:h,sendNote:g,cancelPending:y,onNoteKeydown:x,sendOne:b}},mount(e,n){let r=e,o=null,t=null,a=(p)=>{let l=r.overlayEl();if(!l)return{x:0,y:0};let i=l.getBoundingClientRect();return{x:p.clientX-i.left,y:p.clientY-i.top}},m=(p)=>{if(r.mode()==="off")return;let i=p.target;if(i.closest("[data-annotation-popover]")||i.closest("sui-annotation-marker"))return;if(r.pending()){r.pending.set(null),r.noteText.set("");return}p.preventDefault();let s=a(p);o={startX:s.x,startY:s.y,curX:s.x,curY:s.y},r.drag.set(o),p.target.setPointerCapture?.(p.pointerId)},d=(p)=>{if(!o)return;let l=a(p);if(o={...o,curX:l.x,curY:l.y},t!==null)return;t=requestAnimationFrame(()=>{if(t=null,o)r.drag.set({...o})})},f=(p)=>{if(!o)return;let l=a(p),i=Math.abs(l.x-o.startX),s=Math.abs(l.y-o.startY),u=r.mode(),b=null;if(u==="point"||u==="region"&&i<8&&s<8)b={kind:"point",x:l.x,y:l.y,selector:"body"};else if(u==="region"){let h=Math.min(o.startX,l.x),g=Math.min(o.startY,l.y),y=[Math.round(h),Math.round(g),Math.round(i),Math.round(s)];b={kind:"region",x:h+i/2,y:g+s/2,rect:y}}if(o=null,r.drag.set(null),r.noteText.set(""),b)r.pending.set(b)},c=r.overlayEl();if(c)c.addEventListener("pointerdown",m),c.addEventListener("pointermove",d),c.addEventListener("pointerup",f);return Object.defineProperty(n,"staged",{configurable:!0,get:()=>r.staged()}),n.send=async()=>{for(let p of[...r.staged()])await r.sendOne(p)},n.clear=()=>{r.staged.set([])},()=>{if(t!==null)cancelAnimationFrame(t),t=null;if(c)c.removeEventListener("pointerdown",m),c.removeEventListener("pointermove",d),c.removeEventListener("pointerup",f)}}});var Tm=z("sui-annotation-layer",{props:{"view-id":v.string({default:""}),mode:v.union(["off","point","region"],{default:"off",reflect:!0}),endpoint:v.string({default:"/api/feedback"}),staging:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.viewId=n.viewId,t.mode=n.mode,t.endpoint=n.endpoint,t.staging=n.staging;let a=null;if(Gn&&typeof Gn.setup==="function"){if(e.scope=t,a=Gn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(h)=>G({anchor:h,test:()=>{if(t.dragVisible())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),y=[],x=document.createElement("div");return x.setAttribute("class","drag-rect"),y.push(k(x,"style",()=>t.dragRectStyle())),g.appendChild(x),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let E of y)try{E()}catch(w){}}}}]}),d=(h)=>Y({anchor:h,items:()=>t.markers(),key:(g,y)=>{let x=Object.create(t);return x.m=g,x.__i=y,((E)=>g.id)(x)},render:(g,y)=>{let x=Object.create(t);return x.m=g,x.__i=y,x.__v=N(0),((E)=>{let w=document.createDocumentFragment(),A=[],L=document.createElement("sui-annotation-marker");return A.push($(L,"kind",()=>(E.__v(),E.m.kind))),A.push($(L,"index",()=>(E.__v(),E.m.index))),A.push($(L,"rect",()=>(E.__v(),E.m.rect))),A.push($(L,"x",()=>(E.__v(),E.m.pos.x))),A.push($(L,"y",()=>(E.__v(),E.m.pos.y))),w.appendChild(L),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of A)try{C()}catch(O){}},update:(C,O)=>{E.m=C,E.__i=O,E.__v.set((P)=>P+1)}}})(x)}}),f=(h)=>G({anchor:h,test:()=>{if(t.pending())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),y=[],x=document.createElement("div");x.setAttribute("class","popover"),x.setAttribute("data-annotation-popover",""),y.push(k(x,"style",()=>t.popoverStyle())),y.push(I(x,"pointerdown",function(){let P=(M)=>M.stopPropagation();return typeof P==="function"?P.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("textarea");E.setAttribute("part","note-input"),E.setAttribute("placeholder","Add a note…"),y.push(ut(E,t.noteText)),y.push(I(E,"keydown",function(){let P=t.onNoteKeydown;return typeof P==="function"?P.bind(void 0):()=>{}}(),{modifiers:[]})),x.appendChild(E);let w=document.createElement("div");w.setAttribute("class","popover-actions");let A=document.createElement("button");A.setAttribute("type","button"),y.push(I(A,"click",function(){let P=t.cancelPending;return typeof P==="function"?P.bind(void 0):()=>{}}(),{modifiers:[]}));let L=document.createTextNode("Cancel");A.appendChild(L),w.appendChild(A);let C=document.createElement("button");C.setAttribute("type","button"),C.setAttribute("data-primary",""),y.push(I(C,"click",function(){let P=t.sendNote;return typeof P==="function"?P.bind(void 0):()=>{}}(),{modifiers:[]}));let O=document.createTextNode("Send");return C.appendChild(O),w.appendChild(C),x.appendChild(w),g.appendChild(x),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let P of y)try{P()}catch(M){}}}}]}),c=r.shadowRoot??r;j(c);let p=document.createElement("div");p.setAttribute("class","surface"),p.setAttribute("part","surface");let l=document.createElement("slot");p.appendChild(l);let i=document.createElement("div");i.setAttribute("class","overlay"),i.setAttribute("part","overlay"),D(t,"overlayEl",i);let s=document.createComment("if");i.appendChild(s),o.push(m(s));let u=document.createComment("for");i.appendChild(u),o.push(d(u));let b=document.createComment("if");if(i.appendChild(b),o.push(f(b)),p.appendChild(i),c.appendChild(p),a!=null){let h=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(h){let g=new Proxy(t,{get(y,x){return y[x]},set(y,x,E){let w=y[x];if(w!=null&&typeof w.set==="function"){if(typeof E==="function")w.set(()=>E);else w.set(E);return!0}return!1}});if(h.mount){let y=h.mount(g,r);if(typeof y==="function")o.push(y)}if(h.unmount)o.push(()=>h.unmount(g))}}return t}});var Rn=S({mount(e,n){let r=e;return T(()=>{let t=r.pinEl(),a=r.boxEl(),m=r.x(),d=r.y(),f=r.index(),c=r.rect();if(t)t.style.left=`${m}px`,t.style.top=`${d}px`,t.textContent=String(f+1);if(a&&c){let p=c.split(",").map(Number);if(p.length===4&&p.every((l)=>Number.isFinite(l))){let[l,i,s,u]=p;a.style.cssText=`left:${l}px;top:${i}px;width:${s}px;height:${u}px`}}})}});var Nm=z("sui-annotation-marker",{props:{kind:v.union(["point","region"],{default:"point",reflect:!0}),x:v.number({default:0}),y:v.number({default:0}),rect:v.string({default:""}),active:v.bool({default:!1,reflect:!0}),index:v.number({default:0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.kind=n.kind,t.x=n.x,t.y=n.y,t.rect=n.rect,t.active=n.active,t.index=n.index;let a=null;if(Rn&&typeof Rn.setup==="function"){if(e.scope=t,a=Rn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("part","pin"),d.setAttribute("role","button"),d.setAttribute("tabindex","0"),D(t,"pinEl",d),m.appendChild(d);let f=document.createElement("div");if(f.setAttribute("part","box"),f.setAttribute("role","button"),f.setAttribute("tabindex","0"),D(t,"boxEl",f),m.appendChild(f),a!=null){let c=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(c){let p=new Proxy(t,{get(l,i){return l[i]},set(l,i,s){let u=l[i];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(c.mount){let l=c.mount(p,r);if(typeof l==="function")o.push(l)}if(c.unmount)o.push(()=>c.unmount(p))}}return t}});var Un=S({scope:(e,n)=>{return{staged:N([]),emitRemove:(t)=>{n.dispatchEvent(new CustomEvent("sui-annotation-remove",{detail:{id:t},bubbles:!0,composed:!0}))}}},mount(e,n){let o=e.staged,t=n.staged;if(t!=null)o.set(Array.isArray(t)?t:[]);Object.defineProperty(n,"staged",{configurable:!0,get:()=>o(),set:(a)=>o.set(Array.isArray(a)?a:[])})}});var Um=z("sui-annotation-list",{shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(Un&&typeof Un.setup==="function"){if(e.scope=t,a=Un.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(p)=>Y({anchor:p,items:()=>t.staged(),key:(l,i)=>{let s=Object.create(t);return s.a=l,s.__i=i,((u)=>l.id)(s)},render:(l,i)=>{let s=Object.create(t);return s.a=l,s.__i=i,s.__v=N(0),((u)=>{let b=document.createDocumentFragment(),h=[],g=document.createElement("li");g.setAttribute("part","item"),h.push(k(g,"data-id",()=>(u.__v(),u.a.id)));let y=document.createElement("span");y.setAttribute("class","kind");let x=document.createTextNode("");y.appendChild(x),h.push(R(x,()=>(u.__v(),u.a.kind))),g.appendChild(y);let E=document.createElement("span");E.setAttribute("class","note"),h.push(k(E,"title",()=>(u.__v(),u.a.note??"")));let w=document.createTextNode("");E.appendChild(w),h.push(R(w,()=>(u.__v(),u.a.note??"(no note)"))),g.appendChild(E);let A=document.createElement("button");A.setAttribute("type","button"),A.setAttribute("aria-label","Remove annotation"),h.push(I(A,"click",function(){let C=()=>u.emitRemove(u.a.id);return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let L=document.createTextNode("Remove");return A.appendChild(L),g.appendChild(A),b.appendChild(g),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let C of h)try{C()}catch(O){}},update:(C,O)=>{u.a=C,u.__i=O,u.__v.set((P)=>P+1)}}})(s)}}),d=(p)=>G({anchor:p,test:()=>{if(t.staged().length===0)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("div");s.setAttribute("class","empty"),s.setAttribute("part","empty");let u=document.createTextNode("No staged annotations.");return s.appendChild(u),l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let b of i)try{b()}catch(h){}}}},()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("ol");s.setAttribute("part","list");let u=document.createComment("for");return s.appendChild(u),i.push(m(u)),l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let b of i)try{b()}catch(h){}}}}]}),f=r.shadowRoot??r;j(f);let c=document.createComment("if");if(f.appendChild(c),o.push(d(c)),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var Yn=S({mount(e,n){let r=e,o=T(()=>{let t=r.inputEl();if(!t)return;let a=()=>{if(r.disabled())return;n.dispatchEvent(new CustomEvent("sui-radio-select",{detail:{value:r.value()},bubbles:!0,composed:!0}))},m=(d)=>{if(!["ArrowDown","ArrowRight","ArrowUp","ArrowLeft"].includes(d.key))return;d.preventDefault();let c=d.key==="ArrowDown"||d.key==="ArrowRight";n.dispatchEvent(new CustomEvent("sui-radio-nav",{detail:{direction:c?"next":"prev"},bubbles:!0,composed:!0}))};return t.addEventListener("click",a),t.addEventListener("keydown",m),()=>{t.removeEventListener("click",a),t.removeEventListener("keydown",m)}});return n.focusInput=()=>{n.shadowRoot?.querySelector("input")?.focus()},()=>o()}});var Zm=z("sui-radio",{props:{value:v.string({default:"",reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),checked:v.bool({default:!1,reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.disabled=n.disabled,t.checked=n.checked,t.tone=n.tone,t.size=n.size;let a=null;if(Yn&&typeof Yn.setup==="function"){if(e.scope=t,a=Yn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("label");d.setAttribute("part","label");let f=document.createElement("span");f.setAttribute("class","circle"),f.setAttribute("part","circle");let c=document.createElement("input");c.setAttribute("part","input"),c.setAttribute("type","radio"),o.push(dt(c,t.checked)),o.push(k(c,"disabled",()=>t.disabled())),o.push(k(c,"aria-checked",()=>t.checked()?"true":"false")),o.push(k(c,"tabindex",()=>t.checked()?"0":"-1")),D(t,"inputEl",c),f.appendChild(c);let p=document.createElement("span");p.setAttribute("class","dot"),p.setAttribute("part","dot"),p.setAttribute("aria-hidden","true"),f.appendChild(p),d.appendChild(f);let l=document.createElement("slot");if(d.appendChild(l),m.appendChild(d),a!=null){let i=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(i){let s=new Proxy(t,{get(u,b){return u[b]},set(u,b,h){let g=u[b];if(g!=null&&typeof g.set==="function"){if(typeof h==="function")g.set(()=>h);else g.set(h);return!0}return!1}});if(i.mount){let u=i.mount(s,r);if(typeof u==="function")o.push(u)}if(i.unmount)o.push(()=>i.unmount(s))}}return t}});var Xn=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","radiogroup");let o=document.createElement("input");o.type="hidden",n.appendChild(o);let t=()=>Array.from(n.querySelectorAll(":scope > sui-radio")),a=()=>t().filter((l)=>!l.hasAttribute("disabled")&&!r.disabled()),m=()=>{let l=r.value(),i=r.disabled(),s=r.tone(),u=r.size();for(let b of t()){if(b.toggleAttribute("checked",b.getAttribute("value")===l),i)b.setAttribute("disabled","");if(s&&!b.hasAttribute("tone"))b.setAttribute("tone",s);if(u&&!b.hasAttribute("size"))b.setAttribute("size",u)}o.name=r.name(),o.value=l},d=T(()=>{r.value(),r.disabled(),r.tone(),r.size(),m()}),f=T(()=>{let l=r.label();if(l)n.setAttribute("aria-label",l);else n.removeAttribute("aria-label")}),c=(l)=>{let i=l;i.stopPropagation();let s=i.detail.value;if(s===r.value())return;r.value.set(s),m(),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:s},bubbles:!0,composed:!0}))},p=(l)=>{let i=l;i.stopPropagation();let s=a();if(!s.length)return;let u=i.composedPath().find((y)=>y.tagName?.toLowerCase()==="sui-radio"),b=u?s.indexOf(u):-1,h=i.detail.direction==="next"?s[(b+1)%s.length]:s[(b-1+s.length)%s.length],g=h.getAttribute("value")??"";r.value.set(g),m(),h.focusInput?.(),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:g},bubbles:!0,composed:!0}))};return n.addEventListener("sui-radio-select",c),n.addEventListener("sui-radio-nav",p),()=>{d(),f(),n.removeEventListener("sui-radio-select",c),n.removeEventListener("sui-radio-nav",p),o.remove()}}});var _m=z("sui-radio-group",{props:{value:v.string({default:"",reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),name:v.string({default:""}),label:v.string({default:""})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    gap: var(--space-2, 8px);
    font-family: var(--font-body);
    color: var(--foreground);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.disabled=n.disabled,t.tone=n.tone,t.size=n.size,t.name=n.name,t.label=n.label;let a=null;if(Xn&&typeof Xn.setup==="function"){if(e.scope=t,a=Xn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");if(m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});var qn=S({mount(e,n){let r=e,o=document.createElement("input");o.type="hidden",n.appendChild(o);let t=(s)=>{let u=r.step(),b=r.min(),h=r.max(),g=Math.round((s-b)/u)*u+b;return Math.max(b,Math.min(h,g))},a=(s)=>{let u=t(s);if(u===r.value())return;r.value.set(u),o.name=r.name(),o.value=String(u),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:u},bubbles:!0,composed:!0}))},m=(s)=>{let u=r.trackEl();if(!u)return r.value();let b=u.getBoundingClientRect(),h=Math.max(0,Math.min(1,(s-b.left)/b.width));return r.min()+h*(r.max()-r.min())},d=T(()=>{let s=r.value(),u=r.min(),h=r.max()-u||1,g=(s-u)/h*100,y=r.fillEl(),x=r.thumbEl();if(y)y.style.width=`${g}%`;if(x)x.style.left=`${g}%`;o.name=r.name(),o.value=String(s)}),f=!1,c=(s)=>{if(r.disabled())return;s.preventDefault(),f=!0,a(m(s.clientX)),r.thumbEl()?.focus();let b=(g)=>{if(f)a(m(g.clientX))},h=()=>{f=!1,document.removeEventListener("pointermove",b),document.removeEventListener("pointerup",h)};document.addEventListener("pointermove",b),document.addEventListener("pointerup",h)},p=(s)=>{if(r.disabled())return;let u=r.step(),b=!0;switch(s.key){case"ArrowRight":case"ArrowUp":a(r.value()+u);break;case"ArrowLeft":case"ArrowDown":a(r.value()-u);break;case"Home":a(r.min());break;case"End":a(r.max());break;case"PageUp":a(r.value()+u*10);break;case"PageDown":a(r.value()-u*10);break;default:b=!1}if(b)s.preventDefault()},l=T(()=>{let s=r.containerEl();if(!s)return;return s.addEventListener("pointerdown",c),()=>s.removeEventListener("pointerdown",c)}),i=T(()=>{let s=r.thumbEl();if(!s)return;return s.addEventListener("keydown",p),()=>s.removeEventListener("keydown",p)});return()=>{d(),l(),i(),o.remove()}}});var ab=z("sui-slider",{props:{value:v.number({default:0,reflect:!0}),min:v.number({default:0}),max:v.number({default:100}),step:v.number({default:1}),disabled:v.bool({default:!1,reflect:!0}),tone:v.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),name:v.string({default:""}),label:v.string({default:"slider"})},shadow:!0,styles:`
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

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.min=n.min,t.max=n.max,t.step=n.step,t.disabled=n.disabled,t.tone=n.tone,t.size=n.size,t.name=n.name,t.label=n.label;let a=null;if(qn&&typeof qn.setup==="function"){if(e.scope=t,a=qn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","container"),D(t,"containerEl",d);let f=document.createElement("div");f.setAttribute("class","track"),f.setAttribute("part","track"),D(t,"trackEl",f);let c=document.createElement("div");c.setAttribute("class","fill"),c.setAttribute("part","fill"),D(t,"fillEl",c),f.appendChild(c),d.appendChild(f);let p=document.createElement("div");if(p.setAttribute("class","thumb"),p.setAttribute("part","thumb"),p.setAttribute("role","slider"),o.push(k(p,"tabindex",()=>t.disabled()?"-1":"0")),o.push(k(p,"aria-valuemin",()=>t.min())),o.push(k(p,"aria-valuemax",()=>t.max())),o.push(k(p,"aria-valuenow",()=>t.value())),o.push(k(p,"aria-label",()=>t.label()||"slider")),o.push(k(p,"aria-disabled",()=>t.disabled()?"true":"false")),D(t,"thumbEl",p),d.appendChild(p),m.appendChild(d),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(l.mount){let s=l.mount(i,r);if(typeof s==="function")o.push(s)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});var lb=z("sui-segmented-option",{props:{value:v.string({default:"",reflect:!0}),icon:v.string({default:"",reflect:!0}),disabled:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.icon=n.icon,t.disabled=n.disabled;let a=r.shadowRoot??r;j(a);let m=document.createTextNode(`

`);a.appendChild(m);let d=document.createTextNode(`
`);a.appendChild(d);let f=document.createTextNode(`
`);a.appendChild(f);let c=document.createTextNode(`

`);a.appendChild(c);let p=document.createTextNode(`
`);return a.appendChild(p),t}});var Wo=(e)=>typeof CSS<"u"&&CSS.escape?CSS.escape(e):e.replace(/[^\w-]/g,"\\$&"),Jn=S({scope:()=>{let e=N([]),n=W(null);return{options:e,containerEl:n}},mount(e,n){let r=e,o=r.options;if(!n.hasAttribute("role"))n.setAttribute("role","radiogroup");let t=document.createElement("input");t.type="hidden",n.appendChild(t);let a=()=>{return Array.from(n.querySelectorAll(":scope > sui-segmented-option")).map((s)=>({value:s.getAttribute("value")??"",label:s.textContent?.trim()??"",icon:s.getAttribute("icon")||null,disabled:s.hasAttribute("disabled")}))},m=(i)=>{if(r.disabled())return;let u=o().find((b)=>b.value===i);if(!u||u.disabled)return;if(r.value()===i)return;r.value.set(i),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:i},bubbles:!0,composed:!0}))},d=(i)=>{let s=i.target?.closest(".btn");if(!s)return;let u=s.getAttribute("data-value");if(u!==null)m(u)},f=(i)=>{let u=o().filter((E)=>!E.disabled&&!r.disabled());if(!u.length)return;let b=u.map((E)=>E.value),h=r.value(),g=Math.max(0,b.indexOf(h)),y=g;if(i.key==="ArrowRight"||i.key==="ArrowDown")i.preventDefault(),y=(g+1)%b.length;else if(i.key==="ArrowLeft"||i.key==="ArrowUp")i.preventDefault(),y=(g-1+b.length)%b.length;else if(i.key==="Enter"||i.key===" "){i.preventDefault();let w=n.shadowRoot.activeElement?.getAttribute("data-value");if(w)m(w);return}else return;let x=b[y]??"";m(x),queueMicrotask(()=>{n.shadowRoot?.querySelector(`.btn[data-value="${Wo(x)}"]`)?.focus()})};o.set(a());let c=new MutationObserver(()=>o.set(a()));c.observe(n,{childList:!0,subtree:!1});let p=T(()=>{t.name=r.name(),t.value=r.value()}),l=T(()=>{let i=r.containerEl();if(!i)return;return i.addEventListener("click",d),i.addEventListener("keydown",f),()=>{i.removeEventListener("click",d),i.removeEventListener("keydown",f)}});return()=>{c.disconnect(),p(),l(),t.remove()}}});var mb=z("sui-segmented",{props:{value:v.string({default:"",reflect:!0}),size:v.union(["sm","md"],{default:"md",reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),name:v.string({default:""})},shadow:!0,styles:`
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

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.size=n.size,t.disabled=n.disabled,t.name=n.name;let a=null;if(Jn&&typeof Jn.setup==="function"){if(e.scope=t,a=Jn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(p)=>Y({anchor:p,items:()=>t.options(),key:(l,i)=>{let s=Object.create(t);return s.opt=l,s.__i=i,((u)=>l.value)(s)},render:(l,i)=>{let s=Object.create(t);return s.opt=l,s.__i=i,s.__v=N(0),((u)=>{let b=(w)=>G({anchor:w,test:()=>{if(u.opt.icon)return 0;return-1},branches:[()=>{let A=document.createDocumentFragment(),L=[],C=document.createElement("span");C.setAttribute("class","btn-icon"),C.setAttribute("aria-hidden","true");let O=document.createElement("sui-icon");return L.push($(O,"name",()=>(u.__v(),u.opt.icon))),O.setAttribute("size","sm"),C.appendChild(O),A.appendChild(C),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}}}}]}),h=document.createDocumentFragment(),g=[],y=document.createElement("button");y.setAttribute("class","btn"),y.setAttribute("role","radio"),y.setAttribute("type","button"),g.push(k(y,"data-value",()=>(u.__v(),u.opt.value))),g.push(k(y,"aria-checked",()=>(u.__v(),u.opt.value===u.value()?"true":"false"))),g.push(k(y,"aria-disabled",()=>(u.__v(),u.disabled()||u.opt.disabled?"true":"false"))),g.push($(y,"disabled",()=>(u.__v(),u.disabled()||u.opt.disabled))),g.push(k(y,"tabindex",()=>(u.__v(),u.opt.value===u.value()?"0":"-1")));let x=document.createComment("if");y.appendChild(x),g.push(b(x));let E=document.createTextNode("");return y.appendChild(E),g.push(R(E,()=>(u.__v(),u.opt.label))),h.appendChild(y),{nodes:Array.from(h.childNodes),cleanup:()=>{for(let w of g)try{w()}catch(A){}},update:(w,A)=>{u.opt=w,u.__i=A,u.__v.set((L)=>L+1)}}})(s)}}),d=r.shadowRoot??r;j(d);let f=document.createElement("div");f.setAttribute("class","container"),f.setAttribute("part","container"),D(t,"containerEl",f);let c=document.createComment("for");if(f.appendChild(c),o.push(m(c)),d.appendChild(f),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var Wn=S({mount(e,n){let r=e,o=T(()=>{let t=r.buttonEl();if(!t)return;let a=()=>{if(r.disabled())return;r.pressed.set(!r.pressed()),n.dispatchEvent(new CustomEvent("sui-toggle",{detail:{pressed:r.pressed(),value:r.value()},bubbles:!0,composed:!0}))};return t.addEventListener("click",a),()=>t.removeEventListener("click",a)});return n.focusButton=()=>{n.shadowRoot?.querySelector("button")?.focus()},()=>o()}});var xb=z("sui-toggle",{props:{pressed:v.bool({default:!1,reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),value:v.string({default:"",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:v.union(["outline","ghost"],{default:"outline",reflect:!0})},shadow:!0,styles:`
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

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.pressed=n.pressed,t.disabled=n.disabled,t.value=n.value,t.size=n.size,t.variant=n.variant;let a=null;if(Wn&&typeof Wn.setup==="function"){if(e.scope=t,a=Wn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("button");d.setAttribute("part","button"),d.setAttribute("type","button"),o.push(k(d,"aria-pressed",()=>t.pressed()?"true":"false")),o.push(k(d,"disabled",()=>t.disabled())),D(t,"buttonEl",d);let f=document.createElement("slot");if(d.appendChild(f),m.appendChild(d),a!=null){let c=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(c){let p=new Proxy(t,{get(l,i){return l[i]},set(l,i,s){let u=l[i];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(c.mount){let l=c.mount(p,r);if(typeof l==="function")o.push(l)}if(c.unmount)o.push(()=>c.unmount(p))}}return t}});var Zn=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","group");let o=document.createElement("input");o.type="hidden",n.appendChild(o);let t=()=>{let u=r.defaultSlotEl();if(!u)return Array.from(n.querySelectorAll(":scope > sui-toggle"));return u.assignedElements().filter((b)=>b.tagName.toLowerCase()==="sui-toggle")},a=()=>t().filter((u)=>!u.hasAttribute("disabled")),m=()=>new Set(r.value().split(",").filter(Boolean)),d=(u)=>{let b=Array.from(u).join(",");r.value.set(b),f(),o.name=r.name(),o.value=b,n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:b},bubbles:!0,composed:!0}))},f=()=>{let u=m(),b=r.size(),h=r.variant();for(let g of t()){let y=g.getAttribute("value")??"";if(g.toggleAttribute("pressed",u.has(y)),b)g.setAttribute("size",b);if(h)g.setAttribute("variant",h)}},c=(u)=>{let b=u;b.stopPropagation();let h=b.composedPath().find((x)=>x.tagName?.toLowerCase()==="sui-toggle");if(!h)return;let g=h.getAttribute("value")??"",y=m();if(r.type()==="single")if(y.has(g))d(new Set);else d(new Set([g]));else{if(y.has(g))y.delete(g);else y.add(g);d(y)}},p=(u)=>{if(u.key!=="ArrowRight"&&u.key!=="ArrowLeft")return;let b=a();if(!b.length)return;let h=u.composedPath().find((x)=>b.includes(x)),g=h?b.indexOf(h):-1,y=u.key==="ArrowRight"?(g+1)%b.length:(g-1+b.length)%b.length;b[y]?.focusButton?.(),u.preventDefault()},l=T(()=>{r.value(),r.size(),r.variant(),f()}),i=T(()=>{let u=r.defaultSlotEl();if(!u)return;let b=()=>f();return u.addEventListener("slotchange",b),()=>u.removeEventListener("slotchange",b)}),s=T(()=>{o.name=r.name(),o.value=r.value()});return n.addEventListener("sui-toggle",c),n.addEventListener("keydown",p),()=>{l(),i(),s(),n.removeEventListener("sui-toggle",c),n.removeEventListener("keydown",p),o.remove()}}});var Lb=z("sui-toggle-group",{props:{type:v.union(["single","multiple"],{default:"single",reflect:!0}),value:v.string({default:"",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:v.union(["outline","ghost"],{default:"outline",reflect:!0}),name:v.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    gap: 0;
  }

  ::slotted(sui-toggle:not(:first-child)) { margin-left: -1px; }
  ::slotted(sui-toggle:first-child) { border-radius: var(--radius, 6px) 0 0 var(--radius, 6px); }
  ::slotted(sui-toggle:last-child) { border-radius: 0 var(--radius, 6px) var(--radius, 6px) 0; }
  ::slotted(sui-toggle:not(:first-child):not(:last-child)) { border-radius: 0; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.type=n.type,t.value=n.value,t.size=n.size,t.variant=n.variant,t.name=n.name;let a=null;if(Zn&&typeof Zn.setup==="function"){if(e.scope=t,a=Zn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");if(D(t,"defaultSlotEl",d),m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});var Tb=z("sui-code",{props:{lang:v.string({default:"",reflect:!0}),block:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.lang=n.lang,t.block=n.block;let a=(c)=>G({anchor:c,test:()=>{if(t.lang())return 0;return-1},branches:[()=>{let p=document.createDocumentFragment(),l=[],i=document.createElement("header");i.setAttribute("part","label");let s=document.createTextNode("");return i.appendChild(s),l.push(R(s,()=>t.lang())),p.appendChild(i),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let u of l)try{u()}catch(b){}}}}]}),m=(c)=>G({anchor:c,test:()=>{if(t.block())return 0;return 1},branches:[()=>{let p=document.createDocumentFragment(),l=[],i=document.createElement("div");i.setAttribute("class","block"),i.setAttribute("part","block"),i.setAttribute("role","region"),l.push(k(i,"aria-label",()=>t.lang()?t.lang()+" code block":"code block"));let s=document.createComment("if");i.appendChild(s),l.push(a(s));let u=document.createElement("pre");u.setAttribute("part","pre");let b=document.createElement("slot");return u.appendChild(b),i.appendChild(u),p.appendChild(i),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let h of l)try{h()}catch(g){}}}},()=>{let p=document.createDocumentFragment(),l=[],i=document.createElement("code");i.setAttribute("class","inline"),i.setAttribute("part","inline");let s=document.createElement("slot");return i.appendChild(s),p.appendChild(i),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let u of l)try{u()}catch(b){}}}}]}),d=r.shadowRoot??r;j(d);let f=document.createComment("if");return d.appendChild(f),o.push(m(f)),t}});var Qn=S({mount(e,n){let r=e,o,t=async()=>{try{await navigator.clipboard.writeText(r.value())}catch{let m=document.createElement("textarea");m.value=r.value(),document.body.appendChild(m),m.select();try{document.execCommand("copy")}catch{}m.remove()}if(r.copied.set(!0),n.dispatchEvent(new CustomEvent("sui-copy",{detail:{value:r.value()},bubbles:!0,composed:!0})),o)clearTimeout(o);o=setTimeout(()=>{r.copied.set(!1)},1200)},a=T(()=>{let m=r.buttonEl();if(!m)return;return m.addEventListener("click",t),()=>m.removeEventListener("click",t)});return()=>{if(a(),o)clearTimeout(o)}}});var Nb=z("sui-copy-button",{props:{value:v.string({default:"",reflect:!0}),label:v.string({default:"Copy",reflect:!0}),copied:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.label=n.label,t.copied=n.copied;let a=null;if(Qn&&typeof Qn.setup==="function"){if(e.scope=t,a=Qn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(s)=>G({anchor:s,test:()=>{if(t.copied())return 0;return 1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("sui-icon");return h.setAttribute("name","check"),h.setAttribute("size","sm"),h.setAttribute("aria-hidden","true"),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let g of b)try{g()}catch(y){}}}},()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("sui-icon");return h.setAttribute("name","copy"),h.setAttribute("size","sm"),h.setAttribute("aria-hidden","true"),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let g of b)try{g()}catch(y){}}}}]}),d=r.shadowRoot??r;j(d);let f=document.createElement("button");f.setAttribute("part","button"),f.setAttribute("type","button"),o.push(k(f,"aria-label",()=>t.label())),D(t,"buttonEl",f);let c=document.createComment("if");f.appendChild(c),o.push(m(c));let p=document.createElement("slot");o.push(tt(p,r,"data-empty")),f.appendChild(p);let l=document.createElement("span");l.setAttribute("class","default-label");let i=document.createTextNode("");if(l.appendChild(i),o.push(R(i,()=>t.copied()?"Copied":t.label())),f.appendChild(l),d.appendChild(f),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,g){let y=b[h];if(y!=null&&typeof y.set==="function"){if(typeof g==="function")y.set(()=>g);else y.set(g);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});function Fe(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function Lr(e,n,r,o,t){let a=n!==null?`<span class="key">"${Fe(n)}"</span>: `:"";if(e===null)return`${a}<span class="null">null</span>`;if(typeof e==="string")return`${a}<span class="s">"${Fe(e)}"</span>`;if(typeof e==="number")return`${a}<span class="n">${e}</span>`;if(typeof e==="boolean")return`${a}<span class="b">${String(e)}</span>`;let m=Array.isArray(e),d=m?e.map((s,u)=>[String(u),s]):Object.entries(e),f=o||r<t,c=m?"[":"{",p=m?"]":"}",l=d.length;if(l===0)return`${a}${c}${p}`;let i=d.map(([s,u])=>`<div class="node">${Lr(u,m?null:s,r+1,o,t)}</div>`).join("");return`<details${f?" open":""}>
<summary><span class="caret">▶</span>${a}${c}<span class="meta">${l} ${m?"items":"keys"}</span></summary>
<div class="branch">${i}</div>
${p}</details>`}var In=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","tree");if(!n.hasAttribute("aria-label"))n.setAttribute("aria-label","JSON tree");let o=r.contentEl();if(!o)return()=>{};return nt(o,()=>{let t=r.data(),a=r.expanded(),m=r.depth();if(!t)return"";let d;try{d=JSON.parse(t)}catch(f){return`<span class="error" part="error" role="alert">Invalid JSON: ${Fe(f.message)}</span>`}return`<div part="root">${Lr(d,null,0,a,m)}</div>`})}});var Ub=z("sui-json-view",{props:{data:v.string({default:"",reflect:!0}),expanded:v.bool({default:!1,reflect:!0}),depth:v.number({default:1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.data=n.data,t.expanded=n.expanded,t.depth=n.depth;let a=null;if(In&&typeof In.setup==="function"){if(e.scope=t,a=In.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");if(d.setAttribute("class","json-root"),D(t,"contentEl",d),m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});function zr(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function Zo(e,n){let r=e.split(`
`),o=n.split(`
`),t=r.length,a=o.length,m=Array.from({length:t+1},()=>Array(a+1).fill(0));for(let p=t-1;p>=0;p--)for(let l=a-1;l>=0;l--)m[p][l]=r[p]===o[l]?m[p+1][l+1]+1:Math.max(m[p+1][l],m[p][l+1]);let d=[],f=0,c=0;while(f<t&&c<a)if(r[f]===o[c])d.push({kind:"ctx",text:r[f]}),f++,c++;else if(m[f+1][c]>=m[f][c+1])d.push({kind:"del",text:r[f]}),f++;else d.push({kind:"add",text:o[c]}),c++;while(f<t)d.push({kind:"del",text:r[f++]});while(c<a)d.push({kind:"add",text:o[c++]});return d}function Qo(e){return e.split(`
`).map((n)=>{if(n.startsWith("@@"))return{kind:"hunk",text:n};if(n.startsWith("+"))return{kind:"add",text:n.slice(1)};if(n.startsWith("-"))return{kind:"del",text:n.slice(1)};return{kind:"ctx",text:n.startsWith(" ")?n.slice(1):n}})}function Io(e){return e.map((n)=>{let r=n.kind==="add"?"+":n.kind==="del"?"-":n.kind==="hunk"?"@":" ",o=n.kind==="add"?"added":n.kind==="del"?"removed":n.kind==="hunk"?"hunk":"context";return`<div class="line ${n.kind}" part="line" role="listitem" aria-label="${o}: ${zr(n.text)}"><span class="sign" aria-hidden="true">${r}</span><span>${zr(n.text)}</span></div>`}).join("")}var Vn=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","region");if(!n.hasAttribute("aria-label"))n.setAttribute("aria-label","diff");let o=r.contentEl();if(!o)return()=>{};return nt(o,()=>{let t=r.value(),a=r.from(),m=r.to(),d;if(t)d=Qo(t);else if(a||m)d=Zo(a,m);else d=[];return Io(d)})}});var Zb=z("sui-diff",{props:{from:v.string({default:"",reflect:!0}),to:v.string({default:"",reflect:!0}),value:v.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.from=n.from,t.to=n.to,t.value=n.value;let a=null;if(Vn&&typeof Vn.setup==="function"){if(e.scope=t,a=Vn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");if(d.setAttribute("class","diff-root"),D(t,"contentEl",d),m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});function Be(e){return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function jt(e){let n=Be(e);return n=n.replace(/`([^`]+)`/g,(r,o)=>`<code>${o}</code>`),n=n.replace(/\*\*([^*]+)\*\*/g,"<strong>$1</strong>"),n=n.replace(/(^|[^*])\*([^*\n]+)\*/g,"$1<em>$2</em>"),n=n.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g,(r,o,t)=>`<a href="${t}" rel="noopener noreferrer">${o}</a>`),n}function Vo(e){let n=e.replace(/\r\n/g,`
`).split(`
`),r=[],o=0;while(o<n.length){let t=n[o];if(/^```/.test(t)){let d=t.slice(3).trim(),f=[];o++;while(o<n.length&&!/^```/.test(n[o]))f.push(n[o++]);o++,r.push(`<pre data-lang="${Be(d)}"><code>${Be(f.join(`
`))}</code></pre>`);continue}let a=/^(#{1,6})\s+(.*)$/.exec(t);if(a){r.push(`<h${a[1].length}>${jt(a[2])}</h${a[1].length}>`),o++;continue}if(/^\s*[-*]\s+/.test(t)){let d=[];while(o<n.length&&/^\s*[-*]\s+/.test(n[o]))d.push(`<li>${jt(n[o].replace(/^\s*[-*]\s+/,""))}</li>`),o++;r.push(`<ul>${d.join("")}</ul>`);continue}if(/^\s*\d+\.\s+/.test(t)){let d=[];while(o<n.length&&/^\s*\d+\.\s+/.test(n[o]))d.push(`<li>${jt(n[o].replace(/^\s*\d+\.\s+/,""))}</li>`),o++;r.push(`<ol>${d.join("")}</ol>`);continue}if(/^>\s?/.test(t)){let d=[];while(o<n.length&&/^>\s?/.test(n[o]))d.push(n[o].replace(/^>\s?/,"")),o++;r.push(`<blockquote>${jt(d.join(" "))}</blockquote>`);continue}if(t.trim()===""){o++;continue}let m=[];while(o<n.length&&n[o].trim()!==""&&!/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(n[o]))m.push(n[o]),o++;r.push(`<p>${jt(m.join(" "))}</p>`)}return r.join(`
`)}var Kn=S({mount(e){let n=e,r=n.contentEl();if(!r)return()=>{};return nt(r,()=>Vo(n.value()??""))}});var _b=z("sui-markdown",{props:{value:v.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value;let a=null;if(Kn&&typeof Kn.setup==="function"){if(e.scope=t,a=Kn.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");if(d.setAttribute("class","md-root"),D(t,"contentEl",d),m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});var jr={actions:"#view-topbar-actions","status-start":"#view-status-start","status-end":"#view-status-end"},$n={title:"#breadcrumbs-title",branch:"#status-branch"},_n=S({mount(e,n){let r=e,o={};function t(){for(let f of Object.keys(jr)){let c=document.querySelector(jr[f]);if(!c)continue;let p=Array.from(n.querySelectorAll(`[slot="${f}"]`));if(p.length===0)continue;for(let l of p)c.appendChild(l);o[f]=p}}function a(){let f=document.querySelector($n.title);if(f)f.textContent="";if(r.branch()){let c=document.querySelector($n.branch);if(c)c.textContent=""}if(r.ahead()>0||r.behind()>0){let c=document.querySelector("#status-ab");if(c)c.textContent=""}if(r.tone()!=="default"||r.busy()){let c=document.querySelector("#status-dot");if(c)delete c.dataset.tone,c.removeAttribute("data-busy")}}function m(){for(let f of Object.keys(o)){for(let c of o[f]??[])c.remove();delete o[f]}a()}let d=T(()=>{let f=r.title(),c=r.branch(),p=r.ahead(),l=r.behind(),i=r.tone(),s=r.busy();if(f){let u=document.querySelector($n.title);if(u)u.textContent=f}if(c){let u=document.querySelector($n.branch);if(u)u.textContent=c}if(p>0||l>0){let u=document.querySelector("#status-ab");if(u){let b=[];if(p>0)b.push(`↑${p}`);if(l>0)b.push(`↓${l}`);u.textContent=b.join(" ")}}if(i!=="default"||s){let u=document.querySelector("#status-dot");if(u)if(u.dataset.tone=i,s)u.setAttribute("data-busy","");else u.removeAttribute("data-busy")}});return queueMicrotask(()=>t()),()=>{d(),m()}}});var ah=z("sui-view-chrome",{props:{title:v.string({default:"",reflect:!0}),branch:v.string({default:"",reflect:!0}),ahead:v.number({default:0,reflect:!0}),behind:v.number({default:0,reflect:!0}),tone:v.union(["default","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),busy:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.title=n.title,t.branch=n.branch,t.ahead=n.ahead,t.behind=n.behind,t.tone=n.tone,t.busy=n.busy;let a=null;if(_n&&typeof _n.setup==="function"){if(e.scope=t,a=_n.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createTextNode(`

`);m.appendChild(d);let f=document.createTextNode(`
`);m.appendChild(f);let c=document.createTextNode(`
`);m.appendChild(c);let p=document.createTextNode(`
`);m.appendChild(p);let l=document.createTextNode(`
`);m.appendChild(l);let i=document.createTextNode(`
`);m.appendChild(i);let s=document.createTextNode(`

`);m.appendChild(s);let u=document.createTextNode(`
`);if(m.appendChild(u),a!=null){let b=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(b){let h=new Proxy(t,{get(g,y){return g[y]},set(g,y,x){let E=g[y];if(E!=null&&typeof E.set==="function"){if(typeof x==="function")E.set(()=>x);else E.set(x);return!0}return!1}});if(b.mount){let g=b.mount(h,r);if(typeof g==="function")o.push(g)}if(b.unmount)o.push(()=>b.unmount(h))}}return t}});var te=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","navigation");let o=()=>Array.from(n.querySelectorAll(":scope > sui-navigation-item")),t=()=>o().filter((p)=>!p.hasAttribute("disabled")),a=()=>{let p=r.value();for(let l of o())l.toggleAttribute("active",l.getAttribute("value")===p)},m=(p)=>{let l=p;l.stopPropagation();let i=l.detail.value;r.value.set(i),a(),n.dispatchEvent(new CustomEvent("sui-change",{detail:{value:i},bubbles:!0,composed:!0}))},d=(p)=>{if(p.key!=="ArrowRight"&&p.key!=="ArrowLeft"&&p.key!=="Home"&&p.key!=="End")return;let l=t();if(!l.length)return;let i=p.composedPath().find((b)=>l.includes(b)),s=i?l.indexOf(i):-1,u;if(p.key==="ArrowRight")u=l[(s+1)%l.length];else if(p.key==="ArrowLeft")u=l[(s-1+l.length)%l.length];else if(p.key==="Home")u=l[0];else if(p.key==="End")u=l[l.length-1];if(u)p.preventDefault(),u.focusLink?.()},f=T(()=>{r.value(),a()}),c=T(()=>{let p=r.defaultSlotEl();if(!p)return;let l=()=>a();return p.addEventListener("slotchange",l),()=>p.removeEventListener("slotchange",l)});return n.addEventListener("sui-nav-select",m),n.addEventListener("keydown",d),()=>{f(),c(),n.removeEventListener("sui-nav-select",m),n.removeEventListener("keydown",d)}}});var ch=z("sui-navigation-menu",{props:{value:v.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    align-items: stretch;
    gap: 0;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-body);
  }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value;let a=null;if(te&&typeof te.setup==="function"){if(e.scope=t,a=te.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");if(D(t,"defaultSlotEl",d),m.appendChild(d),a!=null){let f=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(f){let c=new Proxy(t,{get(p,l){return p[l]},set(p,l,i){let s=p[l];if(s!=null&&typeof s.set==="function"){if(typeof i==="function")s.set(()=>i);else s.set(i);return!0}return!1}});if(f.mount){let p=f.mount(c,r);if(typeof p==="function")o.push(p)}if(f.unmount)o.push(()=>f.unmount(c))}}return t}});var ne=S({mount(e,n){let r=e,o=(t)=>{if(r.disabled())return;if(!r.href())t.preventDefault();n.dispatchEvent(new CustomEvent("sui-nav-select",{detail:{value:r.value()},bubbles:!0,composed:!0}))};return n.focusLink=()=>{n.shadowRoot?.querySelector(".link")?.focus()},n.addEventListener("click",o),()=>n.removeEventListener("click",o)}});var vh=z("sui-navigation-item",{props:{value:v.string({default:"",reflect:!0}),active:v.bool({default:!1,reflect:!0}),disabled:v.bool({default:!1,reflect:!0}),href:v.string({default:""})},shadow:!0,styles:`
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

`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.value=n.value,t.active=n.active,t.disabled=n.disabled,t.href=n.href;let a=null;if(ne&&typeof ne.setup==="function"){if(e.scope=t,a=ne.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(p)=>G({anchor:p,test:()=>{if(t.href())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("a");s.setAttribute("class","link"),s.setAttribute("part","link"),i.push(k(s,"href",()=>t.href())),i.push(k(s,"aria-current",()=>t.active()?"page":"false"));let u=document.createElement("slot");return s.appendChild(u),l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let b of i)try{b()}catch(h){}}}},()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("button");s.setAttribute("class","link"),s.setAttribute("part","link"),s.setAttribute("type","button"),i.push(k(s,"aria-current",()=>t.active()?"true":"false")),i.push(k(s,"disabled",()=>t.disabled()));let u=document.createElement("slot");return s.appendChild(u),l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let b of i)try{b()}catch(h){}}}}]}),d=r.shadowRoot??r;j(d);let f=document.createComment("if");d.appendChild(f),o.push(m(f));let c=document.createElement("div");if(c.setAttribute("class","indicator"),c.setAttribute("part","indicator"),c.setAttribute("aria-hidden","true"),d.appendChild(c),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var ee=S({mount(e,n){let r=e,o=null,t=()=>{let d=r.viewportEl();if(!d)return;let f=r.orientation(),c=f==="vertical"||f==="both",p=f==="horizontal"||f==="both",l=r.vThumb(),i=r.hThumb();if(c&&l){let s=d.clientHeight/d.scrollHeight;if(s<1){let u=Math.max(24,s*d.clientHeight),h=d.scrollTop/(d.scrollHeight-d.clientHeight)*(d.clientHeight-u);l.style.height=`${u}px`,l.style.top=`${h}px`,l.style.display=""}else l.style.display="none"}if(p&&i){let s=d.clientWidth/d.scrollWidth;if(s<1){let u=Math.max(24,s*d.clientWidth),h=d.scrollLeft/(d.scrollWidth-d.clientWidth)*(d.clientWidth-u);i.style.width=`${u}px`,i.style.left=`${h}px`,i.style.display=""}else i.style.display="none"}},a=()=>{t();let d=r.vScrollbar(),f=r.hScrollbar();if(d?.classList.add("active"),f?.classList.add("active"),o)clearTimeout(o);o=setTimeout(()=>{d?.classList.remove("active"),f?.classList.remove("active")},1200)},m=r.viewportEl();if(m)m.addEventListener("scroll",a,{passive:!0});return t(),()=>{if(o)clearTimeout(o);if(m)m.removeEventListener("scroll",a)}}});var Eh=z("sui-scroll-area",{props:{orientation:v.union(["vertical","horizontal","both"],{default:"vertical",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.orientation=n.orientation,t.size=n.size;let a=null;if(ee&&typeof ee.setup==="function"){if(e.scope=t,a=ee.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","viewport"),d.setAttribute("part","viewport"),d.setAttribute("tabindex","0"),D(t,"viewportEl",d);let f=document.createElement("slot");d.appendChild(f),m.appendChild(d);let c=document.createElement("div");c.setAttribute("class","scrollbar scrollbar-v"),c.setAttribute("part","scrollbar-v"),c.setAttribute("aria-hidden","true"),D(t,"vScrollbar",c);let p=document.createElement("div");p.setAttribute("class","thumb"),D(t,"vThumb",p),c.appendChild(p),m.appendChild(c);let l=document.createElement("div");l.setAttribute("class","scrollbar scrollbar-h"),l.setAttribute("part","scrollbar-h"),l.setAttribute("aria-hidden","true"),D(t,"hScrollbar",l);let i=document.createElement("div");if(i.setAttribute("class","thumb"),D(t,"hThumb",i),l.appendChild(i),m.appendChild(l),a!=null){let s=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,g){let y=b[h];if(y!=null&&typeof y.set==="function"){if(typeof g==="function")y.set(()=>g);else y.set(g);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var re=S({mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","toolbar");if(!n.hasAttribute("aria-orientation"))n.setAttribute("aria-orientation","horizontal");let o=()=>{return(r.defaultSlot()?.assignedElements({flatten:!0})??[]).filter((d)=>!d.hasAttribute("disabled"))},t=(a)=>{if(a.key!=="ArrowRight"&&a.key!=="ArrowLeft")return;let m=o();if(!m.length)return;let d=a.composedPath().find((p)=>m.includes(p))||null,f=d?m.indexOf(d):-1,c=a.key==="ArrowRight"?(f+1)%m.length:(f-1+m.length)%m.length;m[c]?.focus(),a.preventDefault()};return n.addEventListener("keydown",t),()=>n.removeEventListener("keydown",t)}});var Th=z("sui-toolbar",{props:{justify:v.union(["start","center","end","between"],{default:"start",reflect:!0}),align:v.union(["center","start","end","baseline"],{default:"center",reflect:!0}),gap:v.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:v.union(["default","bordered"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.justify=n.justify,t.align=n.align,t.gap=n.gap,t.variant=n.variant;let a=null;if(re&&typeof re.setup==="function"){if(e.scope=t,a=re.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("slot");d.setAttribute("name","spacer"),m.appendChild(d);let f=document.createElement("slot");if(D(t,"defaultSlot",f),m.appendChild(f),a!=null){let c=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(c){let p=new Proxy(t,{get(l,i){return l[i]},set(l,i,s){let u=l[i];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(c.mount){let l=c.mount(p,r);if(typeof l==="function")o.push(l)}if(c.unmount)o.push(()=>c.unmount(p))}}return t}});var Tr={dashboard:"layout-dashboard",pipelines:"workflow",agents:"bot",skills:"sparkles",git:"git-branch",collections:"database",canvas:"frame",settings:"settings",chat:"messages-square",events:"radio",specs:"file-text",theme:"palette",setup:"rocket",overview:"compass",mesh:"network"},Or={"theme-studio":"theme"},Ko=new Set([...Object.keys(Tr),...Object.keys(Or)]),oe=S({mount(e,n){let r=e,o=T(()=>{let t=r.name(),a=r.label(),m=Or[t]??t,f=Ko.has(t)?Tr[m]:"dot",c=r.glyphEl();if(c)c.setAttribute("name",f);if(a)n.setAttribute("role","img"),n.setAttribute("aria-label",a),n.removeAttribute("aria-hidden");else n.setAttribute("aria-hidden","true"),n.removeAttribute("role"),n.removeAttribute("aria-label")});return()=>o()}});var Nh=z("sui-app-icon",{props:{name:v.string({default:"",reflect:!0}),variant:v.union(["glyph","tile","solid"],{default:"glyph",reflect:!0}),size:v.union(["sm","md","lg"],{default:"md",reflect:!0}),radius:v.string({default:"",reflect:!0}),label:v.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.name=n.name,t.variant=n.variant,t.size=n.size,t.radius=n.radius,t.label=n.label;let a=null;if(oe&&typeof oe.setup==="function"){if(e.scope=t,a=oe.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("span");d.setAttribute("class","frame"),d.setAttribute("part","frame");let f=document.createElement("slot"),c=document.createElement("sui-icon");if(D(t,"glyphEl",c),c.setAttribute("part","glyph"),c.setAttribute("aria-hidden","true"),f.appendChild(c),d.appendChild(f),m.appendChild(d),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var ae=S({scope:()=>{return{nodes:N([])}},mount(e,n){let o=e.nodes,t=n.nodes;if(t!=null)o.set(t);if(Object.defineProperty(n,"nodes",{configurable:!0,get:()=>o(),set:(a)=>o.set(a)}),!n.hasAttribute("role"))n.setAttribute("role","tree")}});var Uh=z("sui-tree-view",{shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
  }
  ul { list-style: none; margin: 0; padding: 0; }
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null),a=null;if(ae&&typeof ae.setup==="function"){if(e.scope=t,a=ae.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(p)=>Y({anchor:p,items:()=>t.nodes(),key:(l,i)=>{let s=Object.create(t);return s.n=l,s.__i=i,((u)=>l.id)(s)},render:(l,i)=>{let s=Object.create(t);return s.n=l,s.__i=i,s.__v=N(0),((u)=>{let b=document.createDocumentFragment(),h=[],g=document.createElement("sui-tree-item");return h.push($(g,"node",()=>(u.__v(),u.n))),b.appendChild(g),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let y of h)try{y()}catch(x){}},update:(y,x)=>{u.n=y,u.__i=x,u.__v.set((E)=>E+1)}}})(s)}}),d=(p)=>G({anchor:p,test:()=>{if(t.nodes().length>0)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("ul");s.setAttribute("role","tree");let u=document.createComment("for");return s.appendChild(u),i.push(m(u)),l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let b of i)try{b()}catch(h){}}}},()=>{let l=document.createDocumentFragment(),i=[],s=document.createElement("slot");return l.appendChild(s),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let u of i)try{u()}catch(b){}}}}]}),f=r.shadowRoot??r;j(f);let c=document.createComment("if");if(f.appendChild(c),o.push(d(c)),a!=null){let p=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(i,s){return i[s]},set(i,s,u){let b=i[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(p.mount){let i=p.mount(l,r);if(typeof i==="function")o.push(i)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var ie=S({scope:({label:e})=>{let n=N(null),r=q(()=>{let a=n();return a?a.label:e()}),o=q(()=>{return n()?.children??[]}),t=W(null);return{node:n,nodeLabel:r,nodeChildren:o,rowEl:t}},mount(e,n){let r=e;if(!n.hasAttribute("role"))n.setAttribute("role","treeitem");let o=r.node,t=n.node;if(t!=null)o.set(t);Object.defineProperty(n,"node",{configurable:!0,get:()=>o(),set:(i)=>o.set(i)});let a=T(()=>{let i=o();if(i!==null)if(!i.children||i.children.length===0)n.setAttribute("leaf","");else n.removeAttribute("leaf")}),m=T(()=>{let i=r.rowEl();if(!i)return;if(!r.leaf())i.setAttribute("aria-expanded",String(r.expanded()));else i.removeAttribute("aria-expanded");i.setAttribute("aria-selected",String(r.selected()))}),d=(i)=>{if(r.leaf())return;i.stopPropagation();let s=!r.expanded();r.expanded.set(s),n.dispatchEvent(new CustomEvent("sui-toggle",{detail:{expanded:s},bubbles:!0,composed:!0}))},f=()=>{n.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},c=(i)=>{f(),d(i)},p=(i)=>{if(i.key==="Enter"||i.key===" ")i.preventDefault(),f();else if(i.key==="ArrowRight"&&!r.leaf()&&!r.expanded())i.preventDefault(),d(i);else if(i.key==="ArrowLeft"&&r.expanded())i.preventDefault(),d(i)},l=T(()=>{let i=r.rowEl();if(!i)return;return i.addEventListener("click",c),i.addEventListener("keydown",p),()=>{i.removeEventListener("click",c),i.removeEventListener("keydown",p)}});return()=>{a(),m(),l()}}});var Zh=z("sui-tree-item",{props:{label:v.string({default:"",reflect:!0}),expanded:v.bool({default:!1,reflect:!0}),selected:v.bool({default:!1,reflect:!0}),leaf:v.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.label=n.label,t.expanded=n.expanded,t.selected=n.selected,t.leaf=n.leaf;let a=null;if(ie&&typeof ie.setup==="function"){if(e.scope=t,a=ie.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(E)=>Y({anchor:E,items:()=>t.nodeChildren(),key:(w,A)=>{let L=Object.create(t);return L.c=w,L.__i=A,((C)=>w.id)(L)},render:(w,A)=>{let L=Object.create(t);return L.c=w,L.__i=A,L.__v=N(0),((C)=>{let O=document.createDocumentFragment(),P=[],M=document.createElement("sui-tree-item");return P.push($(M,"node",()=>(C.__v(),C.c))),O.appendChild(M),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let H of P)try{H()}catch(B){}},update:(H,B)=>{C.c=H,C.__i=B,C.__v.set((F)=>F+1)}}})(L)}}),d=(E)=>G({anchor:E,test:()=>{if(t.nodeChildren().length>0)return 0;return 1},branches:[()=>{let w=document.createDocumentFragment(),A=[],L=document.createComment("for");return w.appendChild(L),A.push(m(L)),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of A)try{C()}catch(O){}}}},()=>{let w=document.createDocumentFragment(),A=[],L=document.createElement("slot");return w.appendChild(L),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of A)try{C()}catch(O){}}}}]}),f=r.shadowRoot??r;j(f);let c=document.createElement("div");c.setAttribute("class","row"),c.setAttribute("part","row"),c.setAttribute("tabindex","0"),D(t,"rowEl",c);let p=document.createElement("span");p.setAttribute("class","caret"),p.setAttribute("part","caret"),p.setAttribute("aria-hidden","true");let l=document.createElement("svg");l.setAttribute("viewbox","0 0 12 12"),l.setAttribute("fill","none"),l.setAttribute("stroke","currentColor"),l.setAttribute("stroke-width","1.5");let i=document.createElement("polyline");i.setAttribute("points","4 3 8 6 4 9"),l.appendChild(i),p.appendChild(l),c.appendChild(p);let s=document.createElement("slot");s.setAttribute("name","icon"),c.appendChild(s);let u=document.createElement("span");u.setAttribute("class","label"),u.setAttribute("part","label");let b=document.createTextNode("");u.appendChild(b),o.push(R(b,()=>t.nodeLabel())),c.appendChild(u);let h=document.createElement("span");h.setAttribute("class","trailing");let g=document.createElement("slot");g.setAttribute("name","trailing"),h.appendChild(g),c.appendChild(h),f.appendChild(c);let y=document.createElement("div");y.setAttribute("class","children"),y.setAttribute("part","children"),y.setAttribute("role","group");let x=document.createComment("if");if(y.appendChild(x),o.push(d(x)),f.appendChild(y),a!=null){let E=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(E){let w=new Proxy(t,{get(A,L){return A[L]},set(A,L,C){let O=A[L];if(O!=null&&typeof O.set==="function"){if(typeof C==="function")O.set(()=>C);else O.set(C);return!0}return!1}});if(E.mount){let A=E.mount(w,r);if(typeof A==="function")o.push(A)}if(E.unmount)o.push(()=>E.unmount(w))}}return t}});var Dr=new WeakSet;function $o(e){if(Dr.has(e))return;Dr.add(e);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let r=new CSSStyleSheet;r.replaceSync(Sr),e.adoptedStyleSheets=[...e.adoptedStyleSheets,r];return}}catch{}let n=e.createElement("style");n.setAttribute("data-sui-table",""),n.textContent=Sr,e.head.appendChild(n)}var se=S({scope:()=>{let e=N([]),n=N([]),r=N(null),o=N("asc"),t=N([]),a=q(()=>{let m=r(),d=o(),f=e();if(!m)return f;return[...f].sort((c,p)=>{let l=String(c[m]??""),i=String(p[m]??""),s=l<i?-1:l>i?1:0;return d==="asc"?s:-s})});return{rows:e,columns:n,sortKey:r,sortDir:o,selected:t,sortedRows:a}},on:{sortBy(e,n){if(e.sortKey===n)e.sortDir=e.sortDir==="asc"?"desc":"asc";else e.sortKey=n,e.sortDir="asc"},toggleSelect(e,n){let r=e.selected;e.selected=r.includes(n)?r.filter((o)=>o!==n):[...r,n]},selectAll(e){e.selected=e.rows.map((n)=>n.id)},clearSelection(e){e.selected=[]}},mount(e,n){let r=e,o=r.rows,t=r.columns,a=n.rows;if(a!=null)o.set(a);let m=n.columns;if(m!=null)t.set(m);Object.defineProperty(n,"rows",{configurable:!0,get:()=>o(),set:(b)=>o.set(b)}),Object.defineProperty(n,"columns",{configurable:!0,get:()=>t(),set:(b)=>t.set(b)}),$o(n.ownerDocument);let d=()=>{n.querySelectorAll("th[data-sort-key]").forEach((b)=>{let h=b.getAttribute("aria-sort")||"none",g=b.querySelector(".sui-sort-indicator");if(!g)return;if(h==="ascending")g.style.transform="rotate(180deg)",g.style.opacity="1";else if(h==="descending")g.style.transform="rotate(0deg)",g.style.opacity="1";else g.style.transform="rotate(0deg)",g.style.opacity="0.4"})},f=()=>{n.querySelectorAll("th[data-sort-key]").forEach((b)=>{if(!b.hasAttribute("aria-sort"))b.setAttribute("aria-sort","none");if(!b.hasAttribute("tabindex"))b.setAttribute("tabindex","0");if(!b.hasAttribute("role"))b.setAttribute("role","columnheader");if(!b.querySelector(".sui-sort-indicator")){let h=document.createElement("span");h.className="sui-sort-indicator",h.setAttribute("aria-hidden","true"),h.style.cssText="margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;";let g="http://www.w3.org/2000/svg",y=document.createElementNS(g,"svg");y.setAttribute("width","10"),y.setAttribute("height","10"),y.setAttribute("viewBox","0 0 10 10"),y.setAttribute("fill","none"),y.setAttribute("stroke","currentColor"),y.setAttribute("stroke-width","1.5"),y.setAttribute("stroke-linecap","round"),y.setAttribute("stroke-linejoin","round"),y.setAttribute("aria-hidden","true");let x=document.createElementNS(g,"path");x.setAttribute("d","M2 4l3-2 3 2M2 6l3 2 3-2"),y.appendChild(x),h.appendChild(y),b.appendChild(h)}}),d()},c=()=>{let b=n.querySelector(":scope > table tbody > tr")||n.querySelector(":scope > table > tr:not(:has(th))")||n.querySelector(":scope > tbody > tr")||n.querySelector(':scope > tr:not([slot="head"])'),h=n.querySelector('[slot="empty"]')!=null;if(!b&&h)n.setAttribute("data-empty","");else n.removeAttribute("data-empty")},p=(b)=>{let h=b.getAttribute("data-sort-key")||"",g=b.getAttribute("aria-sort")||"none",y=g==="none"?"ascending":g==="ascending"?"descending":"none";n.querySelectorAll("th[data-sort-key]").forEach((x)=>{if(x!==b)x.setAttribute("aria-sort","none")}),b.setAttribute("aria-sort",y),d(),n.emit("sui-sort",{key:h,direction:y})},l=(b)=>{let h=b.target.closest("th[data-sort-key]");if(!h||!n.contains(h))return;p(h)},i=(b)=>{if(b.key!=="Enter"&&b.key!==" ")return;let h=b.target.closest("th[data-sort-key]");if(!h||!n.contains(h))return;b.preventDefault(),p(h)},s=(b)=>{let h=b.target.closest("td");if(!h||!n.contains(h))return;if(h.classList.contains("sui-table-gutter-cell"))return;let g=h.closest("tr");if(!g)return;let y=g.getAttribute("data-row-id");if(!y)return;n.emit("sui-row-click",{id:y,index:g.rowIndex})};Z(n,"click",l),Z(n,"click",s),Z(n,"keydown",i),f(),c();let u=new MutationObserver(()=>{f(),c()});return u.observe(n,{childList:!0,subtree:!0}),()=>{u.disconnect()}}}),Sr=`
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
`;var _h=z("sui-table",{props:{variant:v.union(["slot","native","data"],{default:"slot",reflect:!0}),density:v.union(["compact","default","comfortable"],{default:"default",reflect:!0}),dense:v.bool({default:!1,reflect:!0}),hoverable:v.bool({default:!0,reflect:!0}),striped:v.bool({default:!1,reflect:!0}),"sticky-header":v.bool({default:!1,reflect:!0}),"sticky-first-col":v.bool({default:!1,reflect:!0}),gutter:v.bool({default:!1,reflect:!0}),label:v.string({default:""})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.variant=n.variant,t.density=n.density,t.dense=n.dense,t.hoverable=n.hoverable,t.striped=n.striped,t.stickyHeader=n.stickyHeader,t.stickyFirstCol=n.stickyFirstCol,t.gutter=n.gutter,t.label=n.label;let a=null;if(se&&typeof se.setup==="function"){if(e.scope=t,a=se.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=(l)=>Y({anchor:l,items:()=>t.columns(),key:(i,s)=>{let u=Object.create(t);return u.c=i,u.__i=s,((b)=>i.field)(u)},render:(i,s)=>{let u=Object.create(t);return u.c=i,u.__i=s,u.__v=N(0),((b)=>{let h=document.createDocumentFragment(),g=[],y=document.createElement("th");g.push(I(y,"click",function(){let E=()=>b.sortBy(b.c.field);return typeof E==="function"?E.bind(void 0):()=>{}}(),{modifiers:[]})),g.push(mt(y,"sorted",()=>(b.__v(),b.sortKey()===b.c.field)));let x=document.createTextNode("");return y.appendChild(x),g.push(R(x,()=>(b.__v(),b.c.label))),h.appendChild(y),{nodes:Array.from(h.childNodes),cleanup:()=>{for(let E of g)try{E()}catch(w){}},update:(E,w)=>{b.c=E,b.__i=w,b.__v.set((A)=>A+1)}}})(u)}}),d=(l)=>Y({anchor:l,items:()=>t.sortedRows(),key:(i,s)=>{let u=Object.create(t);return u.r=i,u.__i=s,((b)=>i.id)(u)},render:(i,s)=>{let u=Object.create(t);return u.r=i,u.__i=s,u.__v=N(0),((b)=>{let h=(w)=>Y({anchor:w,items:()=>b.columns(),key:(A,L)=>{let C=Object.create(b);return C.c=A,C.__i=L,((O)=>A.field)(C)},render:(A,L)=>{let C=Object.create(b);return C.c=A,C.__i=L,C.__v=N(0),((O)=>{let P=document.createDocumentFragment(),M=[],H=document.createElement("td"),B=document.createTextNode("");return H.appendChild(B),M.push(R(B,()=>(O.__v(),O.r[O.c.field]))),P.appendChild(H),{nodes:Array.from(P.childNodes),cleanup:()=>{for(let F of M)try{F()}catch(U){}},update:(F,U)=>{O.c=F,O.__i=U,O.__v.set((J)=>J+1)}}})(C)}}),g=document.createDocumentFragment(),y=[],x=document.createElement("tr");y.push(mt(x,"selected",()=>(b.__v(),b.selected().includes(b.r.id)))),y.push(I(x,"click",function(){let w=()=>b.toggleSelect(b.r.id);return typeof w==="function"?w.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createComment("for");return x.appendChild(E),y.push(h(E)),g.appendChild(x),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let w of y)try{w()}catch(A){}},update:(w,A)=>{b.r=w,b.__i=A,b.__v.set((L)=>L+1)}}})(u)}}),f=(l)=>G({anchor:l,test:()=>{if(t.variant()==="data")return 0;if(t.variant()!=="native")return 1;return 2},branches:[()=>{let i=document.createDocumentFragment(),s=[],u=document.createElement("div");u.setAttribute("class","wrap"),u.setAttribute("part","wrap");let b=document.createElement("table");b.setAttribute("part","table"),s.push(k(b,"aria-label",()=>t.label()||null));let h=document.createElement("thead");h.setAttribute("part","thead");let g=document.createElement("tr"),y=document.createComment("for");g.appendChild(y),s.push(m(y)),h.appendChild(g),b.appendChild(h);let x=document.createElement("tbody");x.setAttribute("part","tbody");let E=document.createComment("for");return x.appendChild(E),s.push(d(E)),b.appendChild(x),u.appendChild(b),i.appendChild(u),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let w of s)try{w()}catch(A){}}}},()=>{let i=document.createDocumentFragment(),s=[],u=document.createElement("div");u.setAttribute("class","wrap"),u.setAttribute("part","wrap");let b=document.createElement("table");b.setAttribute("part","table"),s.push(k(b,"aria-label",()=>t.label()||null));let h=document.createElement("thead");h.setAttribute("part","thead");let g=document.createElement("slot");g.setAttribute("name","head"),h.appendChild(g),b.appendChild(h);let y=document.createElement("tbody");y.setAttribute("part","tbody");let x=document.createElement("slot");y.appendChild(x),b.appendChild(y),u.appendChild(b);let E=document.createElement("div");E.setAttribute("class","empty"),E.setAttribute("part","empty"),E.setAttribute("role","status");let w=document.createElement("slot");return w.setAttribute("name","empty"),E.appendChild(w),u.appendChild(E),i.appendChild(u),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let A of s)try{A()}catch(L){}}}},()=>{let i=document.createDocumentFragment(),s=[],u=document.createElement("div");u.setAttribute("class","wrap"),u.setAttribute("part","wrap");let b=document.createElement("slot");u.appendChild(b);let h=document.createElement("div");h.setAttribute("class","empty"),h.setAttribute("part","empty"),h.setAttribute("role","status");let g=document.createElement("slot");return g.setAttribute("name","empty"),h.appendChild(g),u.appendChild(h),i.appendChild(u),{nodes:Array.from(i.childNodes),cleanup:()=>{for(let y of s)try{y()}catch(x){}}}}]}),c=r.shadowRoot??r;j(c);let p=document.createComment("if");if(c.appendChild(p),o.push(f(p)),a!=null){let l=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(l){let i=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(l.mount){let s=l.mount(i,r);if(typeof s==="function")o.push(s)}if(l.unmount)o.push(()=>l.unmount(i))}}return t}});function et(e){return String(e).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#39;")}function Mr(e,n){if(Array.isArray(e))return e;if(typeof e==="string"&&e.trim())try{let r=JSON.parse(e);return Array.isArray(r)?r:n}catch{return n}return n}function _o(e,n,r,o){let t=new Map,a=e.some((i)=>typeof i.x!=="number"||typeof i.y!=="number");if(o==="positioned"&&!a){for(let i of e)t.set(i.id,{x:i.x,y:i.y});return t}let m=n*0.22,d=r*0.5,f=e.filter((i)=>i.kind==="agent"),c=e.filter((i)=>i.kind!=="agent");f.forEach((i,s)=>{let u=Math.max(1,f.length),b=d+(s-(u-1)/2)*80;t.set(i.id,{x:m,y:b})});let p=new Map;for(let i of c){let s=p.get(i.kind)??[];s.push(i),p.set(i.kind,s)}let l=Array.from(p.keys());return l.forEach((i,s)=>{let u=p.get(i),b=n*(0.5+(s+1)/(l.length+1)*0.45);u.forEach((h,g)=>{let y=r*(g+1)/(u.length+1);t.set(h.id,{x:b,y})})}),t}function ta(e,n){let r=Math.max(40,Math.abs(n.x-e.x)*0.4);return`M ${e.x} ${e.y} C ${e.x+r} ${e.y}, ${n.x-r} ${n.y}, ${n.x} ${n.y}`}function na(e,n){let r=e.querySelectorAll('template[slot="node"]');for(let o of Array.from(r))if(o.dataset.kind===n)return o;return null}function ea(e,n){let r=na(e,n.kind);if(!r){let o=n.sub?`<div class="node-sub">${et(n.sub)}</div>`:"";return`<div class="node-h"><span class="tag">${et(n.kind)}</span></div><div class="node-title">${et(n.title??n.id)}</div>${o}`}return r.innerHTML.replace(/\$\{title\}/g,et(n.title??"")).replace(/\$\{sub\}/g,et(n.sub??"")).replace(/\$\{id\}/g,et(n.id))}function ra(e,n,r,o,t,a,m){let d=Mr(n,[]),f=Mr(r,[]),c=_o(d,o,t,a),p=f.map((i)=>{let s=c.get(i.from),u=c.get(i.to);if(!s||!u)return"";let b=et(ta(s,u)),h=i.dashed?" data-dashed":"";return`<path d="${b}" data-kind="${et(i.kind??"default")}"${h}></path>`}).join(""),l=d.map((i)=>{let s=c.get(i.id);if(!s)return"";let u=s.x/o*100,b=s.y/t*100,h=m&&m===i.id?" data-selected":"",g=i.w?`--node-w: ${i.w}px;`:"",y=et(`${i.kind}: ${i.title??i.id}`),x=ea(e,i);return`<div class="node" part="node" data-kind="${et(i.kind)}" data-id="${et(i.id)}"${h} tabindex="0" role="button" aria-label="${y}" style="left: ${u}%; top: ${b}%;${g}">${x}</div>`}).join("");return`<svg class="edges" part="edges" viewBox="0 0 ${o} ${t}" preserveAspectRatio="none" aria-hidden="true">${p}</svg>`+l}var le=S({mount(e,n){let r=e,o=r.canvasEl();if(!o)return;let t=o.ownerDocument.createElement("div");o.appendChild(t);let a=nt(t,()=>{let p=r.nodes(),l=r.edges(),i=r.width()||1000,s=r.height()||600,u=r.layout()||"positioned",b=r.selectedId();return ra(n,p,l,i,s,u,b)}),m="",d=(p)=>{let l=p.target.closest(".node[data-id]");if(!l)return;let i=l.getAttribute("data-id")??"",s=l.getAttribute("data-kind")??"";n.setAttribute("selected-id",i),n.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:i,kind:s},bubbles:!0,composed:!0}))},f=(p)=>{if(p.key!=="Enter"&&p.key!==" ")return;let l=p.target.closest(".node[data-id]");if(!l)return;p.preventDefault();let i=l.getAttribute("data-id")??"",s=l.getAttribute("data-kind")??"";n.setAttribute("selected-id",i),n.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:i,kind:s},bubbles:!0,composed:!0}))},c=(p)=>{let l=p.target.closest(".node[data-id]");if(!l){m="";return}let i=l.getAttribute("data-id")??"";if(i===m)return;m=i;let s=l.getAttribute("data-kind")??"";n.dispatchEvent(new CustomEvent("sui-node-hover",{detail:{id:i,kind:s},bubbles:!0,composed:!0}))};return o.addEventListener("click",d),o.addEventListener("keydown",f),o.addEventListener("mouseover",c),()=>{a(),o.removeEventListener("click",d),o.removeEventListener("keydown",f),o.removeEventListener("mouseover",c)}}});var av=z("sui-graph-canvas",{props:{nodes:v.string({default:""}),edges:v.string({default:""}),width:v.number({default:1000,reflect:!0}),height:v.number({default:600,reflect:!0}),layout:v.union(["positioned","radial"],{default:"positioned",reflect:!0}),"selected-id":v.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(e){let{props:n,host:r}=e;if(!e.cleanups)e.cleanups=[];let o=e.cleanups,t=Object.create(null);t.nodes=n.nodes,t.edges=n.edges,t.width=n.width,t.height=n.height,t.layout=n.layout,t.selectedId=n.selectedId;let a=null;if(le&&typeof le.setup==="function"){if(e.scope=t,a=le.setup(e),a&&typeof a==="object")Object.assign(t,a)}let m=r.shadowRoot??r;j(m);let d=document.createElement("div");d.setAttribute("class","canvas"),d.setAttribute("part","canvas"),D(t,"canvasEl",d),m.appendChild(d);let f=document.createElement("slot");if(f.setAttribute("name","node"),f.setAttribute("hidden",""),m.appendChild(f),a!=null){let c=Object.getOwnPropertyDescriptor(a,"__shipSetLifecycle")?.value;if(c){let p=new Proxy(t,{get(l,i){return l[i]},set(l,i,s){let u=l[i];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(c.mount){let l=c.mount(p,r);if(typeof l==="function")o.push(l)}if(c.unmount)o.push(()=>c.unmount(p))}}return t}});
