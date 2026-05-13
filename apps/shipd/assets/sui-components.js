var Se=Symbol.for("sui:error-boundary");function He(n,e){n[Se]=e}function Be(n){delete n[Se]}function ft(n){let e=n;while(e){if(e instanceof Element){let t=e[Se];if(t)return t}let r=e.parentNode;if(r){e=r;continue}let o=e;if(o.host){e=o.host;continue}e=null}return null}function yt(n,e,r){try{return r()}catch(o){let t=ft(n);if(t){t.catch(o,e);return}console.error("[sui] uncaught throw (no error boundary):",o);return}}var st=null,ct=null,Ao=0,Eo=new Set,Ot=[];function Fe(n,e){Ot.push(n);try{return e()}finally{Ot.pop()}}function ko(){return Ot.length>0?Ot[Ot.length-1]:null}function br(n){if(st===null)return;if(n.has(st))return;let e=st;if(n.add(e),ct!==null)ct.push(()=>n.delete(e))}function M(n){let e=n,r=new Set,o=()=>{return br(r),e};return o.set=(t)=>{let i=typeof t==="function"?t(e):t;if(Object.is(i,e))return;if(e=i,Ao>0)for(let f of r)Eo.add(f);else for(let f of[...r])f()},o.__subs=r,o.__peek=()=>e,o}function Z(n){let e,r=!0,o=new Set,t=[],i=()=>{if(r)return;r=!0;for(let m of[...o])m()},f=()=>{for(let l of t)l();t=[];let m=st,p=ct;st=i,ct=t;try{e=n()}finally{st=m,ct=p}r=!1},c=()=>{if(br(o),r)f();return e};return c.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},c.__subs=o,c.__peek=()=>{if(r)f();return e},c}function D(n){let e=void 0,r=!0,o=[],t=ko(),i=()=>{if(!r)return;if(e){try{e()}catch(m){console.error("[sui] effect cleanup threw:",m)}e=void 0}for(let m of o)m();o=[];let f=st,c=ct;st=i,ct=o;try{e=n()}catch(m){let p=t?ft(t):null;if(p)p.catch(m,"effect");else console.error("[sui] effect threw:",m)}finally{st=f,ct=c}};return i(),()=>{r=!1;for(let f of o)f();if(o=[],e){try{e()}catch(f){console.error("[sui] effect cleanup threw:",f)}e=void 0}}}var Rt=null;function Tt(n,e){let r=Rt;Rt=n;try{return e()}finally{Rt=r}}function ut(n){let e=Rt;if(!e)return;e.cleanup(n)}function Pt(){let n=[],e=!1;return{cleanup(o){if(e){try{o()}catch(t){console.error("[sui/scope] late cleanup threw:",t)}return}n.push(o)},dispose(){if(e)return;e=!0;for(let o=n.length-1;o>=0;o--)try{n[o]()}catch(t){console.error("[sui/scope] cleanup threw:",t)}n.length=0},get disposed(){return e}}}var Co=Symbol("sui.shared");function hr(n){return!!n?.[Co]}function Ge(n,e,r){n[`__sui_ctx_${e}`]=r}function Ue(n){let e={};for(let r in n)if(Object.prototype.hasOwnProperty.call(n,r)&&hr(n[r]))e[r]=n[r];return e}function J(n=null){return M(null)}function T(n,e,r){let o=n[e];if(typeof o==="function"&&typeof o.set==="function")return o.set(r),o;let t=M(r);return n[e]=t,t}function nt(n,e,r){let o=()=>{if(n.assignedNodes({flatten:!0}).some(jo))e.removeAttribute(r);else e.setAttribute(r,"")};return n.addEventListener("slotchange",o),o(),()=>{n.removeEventListener("slotchange",o)}}function jo(n){if(n.nodeType===Node.TEXT_NODE)return(n.textContent??"").trim().length>0;if(n.nodeType===Node.ELEMENT_NODE){let e=n;return(e.textContent??"").trim().length>0||e.childElementCount>0}return!1}function Jt(n,e,r={}){if(typeof e==="string"){let o=n.tagName.toLowerCase(),t=n.namespaceURI==="http://www.w3.org/2000/svg"?n.ownerDocument.createElementNS("http://www.w3.org/2000/svg",o):n.ownerDocument.createElement(o);t.innerHTML=e,gr(n,t,r);return}vr(n,e,r)}function vr(n,e,r){if(r.skip?.(n))return;if(n.tagName!==e.tagName){n.replaceWith(e.cloneNode(!0));return}zo(n,e),gr(n,e,r)}function zo(n,e){let r=Array.from(n.attributes);for(let t of r)if(!e.hasAttribute(t.name))n.removeAttribute(t.name);let o=Array.from(e.attributes);for(let t of o)if(n.getAttribute(t.name)!==t.value)n.setAttribute(t.name,t.value);if(n instanceof HTMLInputElement&&e instanceof HTMLInputElement){if(e.hasAttribute("value")&&n.value!==e.value)n.value=e.value;if(n.checked!==e.checked)n.checked=e.checked}}function gr(n,e,r){let o=new Map;for(let c of Array.from(n.children))if(c.id)o.set(c.id,c);let t=new Set;for(let c of Array.from(e.children))if(c.id)t.add(c.id);let i=Array.from(e.childNodes),f=n.firstChild;for(let c of i){while(f&&f.nodeType===Node.ELEMENT_NODE&&f.id&&t.has(f.id)&&!(c.nodeType===Node.ELEMENT_NODE&&c.id===f.id))f=f.nextSibling;let m=null;if(c.nodeType===Node.ELEMENT_NODE){let p=c.id;if(p&&o.has(p))m=o.get(p),o.delete(p)}if(!m&&f&&Lo(f,c))m=f;if(m){if(m===f)f=f.nextSibling;else n.insertBefore(m,f);Do(m,c,n,r)}else n.insertBefore(c.cloneNode(!0),f)}while(f){let c=f.nextSibling;n.removeChild(f),f=c}}function Lo(n,e){if(n.nodeType!==e.nodeType)return!1;if(n.nodeType===Node.ELEMENT_NODE)return n.tagName===e.tagName;return!0}function Do(n,e,r,o){if(n.nodeType!==e.nodeType){r.replaceChild(e.cloneNode(!0),n);return}if(n.nodeType===Node.TEXT_NODE||n.nodeType===Node.COMMENT_NODE){if(n.nodeValue!==e.nodeValue)n.nodeValue=e.nodeValue;return}if(n.nodeType===Node.ELEMENT_NODE)vr(n,e,o)}function F(n,e){let r;return D(()=>{let o=e(),t=o==null?"":String(o);if(t===r)return;if(r===void 0&&n.textContent===t){r=t;return}n.textContent=t,r=t})}function k(n,e,r){let o,t=!1;return D(()=>{let i=r();if(t&&i===o)return;let f=i==null||i===!1||i===""?null:i===!0?"":String(i);if(t||n.getAttribute(e)!==f)if(f===null)n.removeAttribute(e);else n.setAttribute(e,f);o=i,t=!0})}function pt(n,e,r){let o;return D(()=>{let t=!!r();if(t===o)return;if(o===void 0&&t===n.classList.contains(e)){o=t;return}if(t)n.classList.add(e);else n.classList.remove(e);o=t})}function tt(n,e,r){return D(()=>{n[e]=r()})}function mt(n,e,r){let o=D(()=>{let i=e();if(n.value!==i)n.value=i}),t=()=>{let i=n.value;if(e.set(i),r)r(i)};return n.addEventListener("input",t),()=>{o(),n.removeEventListener("input",t)}}function bt(n,e){let r=D(()=>{let t=e();if(n.checked!==t)n.checked=t}),o=()=>e.set(n.checked);return n.addEventListener("change",o),()=>{r(),n.removeEventListener("change",o)}}function et(n,e){let o=n.namespaceURI==="http://www.w3.org/2000/svg",t=n.tagName.toLowerCase();return D(()=>{let i=e(),f=o?n.ownerDocument.createElementNS("http://www.w3.org/2000/svg",t):n.ownerDocument.createElement(t);f.innerHTML=i,Jt(n,f)})}function K(n,e,r,o={}){let t=o.modifiers??[],i,f=!0,c=(p)=>{for(let a of t)switch(a.kind){case"preventDefault":p.preventDefault();break;case"stopPropagation":p.stopPropagation();break;case"stopImmediatePropagation":p.stopImmediatePropagation();break;case"self":if(p.target!==n)return;break;case"trusted":if(!p.isTrusted)return;break;case"key":{if(!yr(p,a.arg??""))return;break}}let l=t.find((a)=>a.kind==="debounce"),d=t.find((a)=>a.kind==="throttle");if(l){let a=Number(l.arg)||0;if(i!==void 0)clearTimeout(i);i=setTimeout(()=>{i=void 0,r(p)},a);return}if(d){let a=Number(d.arg)||0;if(!f)return;f=!1,r(p),i=setTimeout(()=>{f=!0},a);return}r(p)},m={};if(o.capture)m.capture=!0;if(o.passive)m.passive=!0;if(o.once)m.once=!0;return n.addEventListener(e,c,m),()=>{if(n.removeEventListener(e,c,m),i!==void 0)clearTimeout(i)}}function yr(n,e){let r=e.split("+").map((l)=>l.toLowerCase()),o=r.pop()??"",t=r.includes("meta")||r.includes("cmd"),i=r.includes("ctrl")||r.includes("control"),f=r.includes("shift"),c=r.includes("alt")||r.includes("option"),m=r.includes("mod"),p=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(t&&!n.metaKey)return!1;if(i&&!n.ctrlKey)return!1;if(f&&!n.shiftKey)return!1;if(c&&!n.altKey)return!1;if(m&&!(p?n.metaKey:n.ctrlKey))return!1;return n.key.toLowerCase()===o}function xr(n,e,r){n.dispatchEvent(new CustomEvent(e,{detail:r,bubbles:!0,composed:!0}))}function wr(n,e){let r=n,o=Object.keys(e);for(let t of o)r[t]=e[t];ut(()=>{for(let t of o)if(r[t]===e[t])try{delete r[t]}catch{r[t]=void 0}})}function Wt(n){let e=n;e.emit=function(r,o){xr(n,r,o)},e.expose=function(r){wr(n,r)}}function Q(n,e,r,o){n.addEventListener(e,r,o),ut(()=>n.removeEventListener(e,r,o))}function Xe(n,e){let r=D(()=>{let o=n();if(!o)return;let t=()=>{e(o.assignedElements())};return o.addEventListener("slotchange",t),t(),()=>o.removeEventListener("slotchange",t)});ut(r)}var qe=Symbol("sui:transition");function Yt(n){return n[qe]}function Oo(n){let e=n,r=e[qe];if(!r)r={inOpts:{},outOpts:{}},e[qe]=r;return r}function ot(n,e,r={},o="both"){let t=Oo(n);if(o==="in"||o==="both")t.inT=e,t.inOpts=r;if(o==="out"||o==="both")t.outT=e,t.outOpts=r;if(!n.hasAttribute("data-sui-transition"))n.setAttribute("data-sui-transition","")}function To(){if(typeof window>"u"||typeof window.matchMedia!=="function")return!1;try{return window.matchMedia("(prefers-reduced-motion: reduce)").matches}catch{return!1}}function kr(n){if(!To())return n;return{...n,duration:0,delay:0}}function Cr(n){let e=Yt(n);if(!e?.inT?.in)return;e.current?.cancel();let r=e.inT.in(n,kr(e.inOpts));return e.current=r,r}function jr(n){let e=Yt(n);if(!e?.outT?.out)return;e.current?.cancel();let r=e.outT.out(n,kr(e.outOpts));return e.current=r,r}function Re(n){if(n.nodeType!==1)return;let e=n;if(e.hasAttribute("data-sui-transition")){let r=Yt(e);if(r?.current?.cancel(),r)r.current=void 0}for(let r of e.querySelectorAll("[data-sui-transition]")){let o=Yt(r);if(o?.current?.cancel(),o)o.current=void 0}}function zr(n,e){for(let r of n){if(r.nodeType!==1)continue;let o=r;if(o.hasAttribute("data-sui-transition"))e(o);for(let t of o.querySelectorAll("[data-sui-transition]"))e(t)}}function At(n){zr(n,(e)=>{Cr(e)})}function Mt(n){let e=[];if(zr(n,(r)=>{let o=jr(r);if(o)e.push(o.finished.catch(()=>{}))}),e.length===0)return null;return Promise.all(e).then(()=>{return})}var Po=200,Mo="cubic-bezier(0.4, 0, 0.2, 1)",No=0;function xt(n){return{duration:typeof n.duration==="number"?n.duration:Po,easing:typeof n.easing==="string"?n.easing:Mo,delay:typeof n.delay==="number"?n.delay:No,fill:"both"}}function wt(n,e,r){let o=n.animate;if(typeof o!=="function")return;return o.call(n,e,r)}var ht={in:(n,e)=>wt(n,[{opacity:0},{opacity:1}],xt(e)),out:(n,e)=>wt(n,[{opacity:1},{opacity:0}],xt(e))};function Ar(n,e){let r=n==="y"?"translateY(-100%)":"translateX(-100%)",o="translate(0, 0)";return e==="in"?[{opacity:0,transform:r},{opacity:1,transform:"translate(0, 0)"}]:[{opacity:1,transform:"translate(0, 0)"},{opacity:0,transform:r}]}var Je={in:(n,e)=>{let r=e.axis==="x"?"x":"y";return wt(n,Ar(r,"in"),xt(e))},out:(n,e)=>{let r=e.axis==="x"?"x":"y";return wt(n,Ar(r,"out"),xt(e))}};function Er(n,e){let r={opacity:0,transform:`scale(${n})`},o={opacity:1,transform:"scale(1)"};return e==="in"?[r,o]:[o,r]}var We={in:(n,e)=>{let r=typeof e.from==="number"?e.from:0;return wt(n,Er(r,"in"),xt(e))},out:(n,e)=>{let r=typeof e.from==="number"?e.from:0;return wt(n,Er(r,"out"),xt(e))}};function So(n,e,r,o,t){if(r.fill(0,0,e),e===0)return;let i=0;for(let c=0;c<e;c++){let m=n[2*c+1],p=0,l=i;while(p<l){let d=p+l>>>1;if(n[2*t[d]+1]<m)p=d+1;else l=d}if(o[c]=p>0?t[p-1]:-1,t[p]=c,p===i)i++}let f=t[i-1];while(f>=0)r[f]=1,f=o[f]}function R(n){let{anchor:e,items:r,key:o,render:t}=n,i=[],f=!0,c=new Set,m=new Map,p=new Map,l=new Map,d=new Set,a=[],s=[],u=new Int32Array(0),b=new Uint8Array(0),h=new Int32Array(0),v=new Int32Array(0),g=(A)=>{for(let w of A.nodes)try{w.parentNode?.removeChild(w)}catch{}A.cleanup?.()},x=D(()=>{if(!f)return;let A=r(),w=A.length;if(u.length<w*2)u=new Int32Array(Math.max(u.length*2,w*2,16));if(b.length<w){let O=Math.max(b.length*2,w,8);b=new Uint8Array(O),h=new Int32Array(O),v=new Int32Array(O)}for(let O=0;O<w;O++){let S=A[O],H=o(S,O);if(m.has(H))console.warn("[sui] {#for}: duplicate key",H,"at index",O,"(prior at",m.get(H),"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");m.set(H,O)}for(let O of i)p.set(O.key,O);for(let O of i){if(m.has(O.key))continue;let S=Mt(O.nodes);if(S===null){g(O);continue}c.add(O),S.then(()=>{if(!c.delete(O))return;g(O)})}s.length=0,a.length=0;for(let O=0;O<w;O++){let S=A[O],H=o(S,O),U=p.get(H),G;if(U)U.item=S,yt(e,"render",()=>U.update?.(S,O)),G=U;else{let X=yt(e,"render",()=>t(S,O));if(!X)continue;G={key:H,item:S,nodes:X.nodes,...X.cleanup&&{cleanup:X.cleanup},...X.update&&{update:X.update}},a.push(...X.nodes)}s.push(G)}for(let O=0;O<i.length;O++)l.set(i[O].key,O);let E=0;for(let O=0;O<s.length;O++){let S=l.get(s[O].key);if(S!==void 0)u[2*E]=O,u[2*E+1]=S,E++}So(u,E,b,h,v);for(let O=0;O<E;O++)if(b[O])d.add(u[2*O]);let j=e.parentNode,C=e;for(let O=0;O<s.length;O++){if(!j)break;let S=s[O];if(d.has(O))C=S.nodes[S.nodes.length-1]??C;else for(let H of S.nodes)j.insertBefore(H,C.nextSibling),C=H}let N=i;if(i=s,s=N,a.length>0)At(a);m.clear(),p.clear(),l.clear(),d.clear(),a.length=0});return()=>{f=!1,x();for(let A of i)g(A);for(let A of c)g(A);c.clear(),i=[],s.length=0}}function B(n){let{anchor:e}=n,{test:r,branches:o}=Ho(n)?{test:n.test,branches:n.branches}:Bo(n),t=null,i=-2,f=null,c=(p)=>{for(let l of p)try{l.parentNode?.removeChild(l)}catch{}},m=D(()=>{let p=r();if(t&&p===i)return;if(f&&p===f.idx&&p>=0&&p<o.length){let a=f;f=null;for(let s of a.nodes)Re(s);t={nodes:a.nodes,...a.cleanup&&{cleanup:a.cleanup}},i=p,At(t.nodes);return}if(t){let a={idx:i,nodes:t.nodes,...t.cleanup&&{cleanup:t.cleanup}},s=Mt(a.nodes);if(t=null,s===null)c(a.nodes),a.cleanup?.();else f=a,s.then(()=>{if(f!==a)return;c(a.nodes),a.cleanup?.(),f=null})}if(i=p,p<0||p>=o.length)return;let l=yt(e,"render",()=>o[p]());if(!l)return;t=l;let d=e.parentNode;if(!d)return;for(let a of t.nodes)d.insertBefore(a,e);At(t.nodes)});return()=>{if(m(),t)c(t.nodes),t.cleanup?.(),t=null;if(f)c(f.nodes),f.cleanup?.(),f=null}}function Ho(n){return typeof n.test==="function"}function Bo(n){let{cond:e,then_:r,else_:o}=n;return{test:()=>e()?0:o?1:-1,branches:o?[r,o]:[r]}}var Ye=(n)=>n??"",Ze=(n)=>{if(n===null||n==="")return NaN;return Number(n)},Qe=(n)=>{if(n===null)return!1;if(n.toLowerCase()==="false")return!1;return!0};function Zt(n){return(e)=>{if(e===null||e==="")return n;try{return JSON.parse(e)}catch{return n}}}function Ve(n,e){let r=new Set(n);return(o)=>o!==null&&r.has(o)?o:e}var Qt=(n)=>{switch(n.type){case"string":return Ye;case"bool":return Qe;case"number":return Ze;case"union":{let e=n.values??[],r=n.default??e[0]??"";return Ve(e,r)}case"object":return Zt(n.default);case"array":return Zt(n.default??[])}};function Dr(n,e,r,o){if(!e)return;let t={},i,f=r.formValue;if(f!==void 0){let{sig:m,propDef:p}=Lr(r,o,f,"formValue"),l=p.default,d=Qt(p);i=D(()=>{let a=m();e.setFormValue(Fo(a))}),t.reset=()=>{m.set(l)},t.restore=(a)=>{m.set(d(a))}}let c=r.formDisabled;if(c!==void 0){let{sig:m,propDef:p}=Lr(r,o,c,"formDisabled");if(p.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${c}" must be a bool-typed prop`);t.disabled=(l)=>{m.set(!!l)}}if(n.__shipSetFormHooks=t,i)n.__shipSetFormValueCleanup=i;return i}function Or(n){let e=n;delete e.__shipSetFormHooks,delete e.__shipSetFormValueCleanup}function Fo(n){if(n===null||n===void 0)return"";if(typeof n==="boolean")return n?"on":"";return String(n)}function Lr(n,e,r,o){let t=n.props[r];if(!t)throw Error(`[sui-runtime] ${o}: "${r}" — no matching prop on this component`);let i=e[Et(r)];if(!i)throw Error(`[sui-runtime] ${o}: "${r}" — prop signal missing`);return{sig:i,propDef:t}}var y={string(n={}){return{type:"string",...n}},bool(n={}){return{type:"bool",...n}},number(n={}){return{type:"number",...n}},union(n,e={}){return{type:"union",values:n,...e}},object(n={}){return{type:"object",...n}},array(n={}){return{type:"array",...n}}},Go=(n,e)=>{if(e===void 0||e===null)return null;switch(n.type){case"string":return e===""?null:String(e);case"number":return Number.isNaN(e)?null:String(e);case"bool":return e?"":null;case"union":return String(e);case"object":case"array":try{return JSON.stringify(e)}catch{return null}}},Uo=(n,e)=>{let r=`ship-set-style-${n}`;if(document.getElementById(r))return;let o=document.createElement("style");o.id=r,o.textContent=`[data-component="${n}"] { ${e} }`,document.head.appendChild(o)},Xo=(n,e,r,o)=>{let t=n.getAttribute(e),i=t!==null?o(t):r.default;return M(i)},qo=(n)=>{let e=[],r={};for(let[i,f]of Object.entries(n.props??{}))e.push(i),r[i]={camel:Et(i),decode:Qt(f)};let o=n.formAssociated===!0||n.formValue!==void 0||n.formDisabled!==void 0,t={observedAttrs:e,props:r};if(o)t.formAssociated=!0;return t};function z(n,e){let r=qo(e),o=Object.entries(e.props??{});Ke(n,r,(t,i,f)=>{let c=null;if(e.shadow){if(c=t.shadowRoot??t.attachShadow({mode:"open"}),e.styles){let p=document.createElement("style");p.textContent=e.styles,c.appendChild(p)}}else if(e.styles)Uo(n,e.styles);let m=[];for(let[p,l]of o){let d=Et(p),a=r.props[p].decode,s=Xo(t,p,l,a);if(i[d]=s,l.reflect){let u=!0,b=D(()=>{let h=s();if(u){u=!1;return}let v=Go(l,h);if(v===null)t.removeAttribute(p);else t.setAttribute(p,v)});m.push(b)}}if(e.formValue!==void 0||e.formDisabled!==void 0){let p=Dr(t,f,e,i);if(typeof p==="function")m.push(p)}if(e.setup){let p={props:i,host:t,cleanups:m};if(f)p.internals=f;e.setup(p)}return()=>{Or(t);for(let p of m.splice(0))try{p()}catch(l){console.error(`[ship-set:${n}] cleanup threw:`,l)}}})}var Tr=new Set;function Ke(n,e,r){if(customElements.get(n)){if(!Tr.has(n))Tr.add(n),console.warn(`[sui-runtime] customElements.define("${n}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(e.formAssociated){class t extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return e.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=n,Wt(this),this.scope=Pt();try{this.cleanup=Tt(this.scope,()=>Fe(this,()=>r(this,this.propSignals,this.internals)))}catch(i){let f=ft(this.parentNode);if(f)f.catch(i,"mount");else console.error(`[ship-set:${n}] setup threw:`,i)}}disconnectedCallback(){try{this.cleanup?.()}catch(i){console.error(`[ship-set:${n}] cleanup threw:`,i)}this.cleanup=void 0;try{this.scope?.dispose()}catch(i){console.error(`[ship-set:${n}] scope dispose threw:`,i)}this.scope=void 0}attributeChangedCallback(i,f,c){let m=e.props[i];if(!m)return;let p=this.propSignals[m.camel];if(p)p.set(m.decode(c))}formResetCallback(){let i=this.__shipSetFormHooks;try{i?.reset?.()}catch(f){console.error(`[ship-set:${n}] formResetCallback threw:`,f)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(i,f){let c=this.__shipSetFormHooks;try{c?.restore?.(i)}catch(m){console.error(`[ship-set:${n}] formStateRestoreCallback threw:`,m)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:i,mode:f}}))}formDisabledCallback(i){let f=this.__shipSetFormHooks;try{f?.disabled?.(i)}catch(c){console.error(`[ship-set:${n}] formDisabledCallback threw:`,c)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:i}}))}}customElements.define(n,t);return}class o extends HTMLElement{static get observedAttributes(){return e.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=n,Wt(this),this.scope=Pt();try{this.cleanup=Tt(this.scope,()=>Fe(this,()=>r(this,this.propSignals)))}catch(t){let i=ft(this.parentNode);if(i)i.catch(t,"mount");else console.error(`[ship-set:${n}] setup threw:`,t)}}disconnectedCallback(){try{this.cleanup?.()}catch(t){console.error(`[ship-set:${n}] cleanup threw:`,t)}this.cleanup=void 0;try{this.scope?.dispose()}catch(t){console.error(`[ship-set:${n}] scope dispose threw:`,t)}this.scope=void 0}attributeChangedCallback(t,i,f){let c=e.props[t];if(!c)return;let m=this.propSignals[c.camel];if(m)m.set(c.decode(f))}}customElements.define(n,o)}function Et(n){return n.replace(/-([a-z])/g,(e,r)=>r.toUpperCase())}function Ie(n,e,r){let o=Wo(n.state,{ship:e.ship,...r}),t={};for(let[a,s]of Object.entries(o))t[a]=M(s);let i={ship:e.ship,route:e.route??{},navigate:e.navigate??((a)=>{}),...r,...t},f=Yo(t,i),c={...i},m={};for(let[a,s]of Object.entries(n.derived??{}))m[a]=Z(()=>s(c));Object.assign(c,m);let p={};for(let[a,s]of Object.entries(n.on??{}))p[a]=(...u)=>s(f,...u);let l={},d=typeof n.use==="function"?n.use(c):n.use??{};for(let[a,s]of Object.entries(d))l[a]=Ie(s,e,c);return{...t,...m,...p,...l}}function Wo(n,e){if(n===void 0)return{};if(typeof n==="function")return n(e);return n}function Yo(n,e){return new Proxy({},{get(r,o){if(o in n)return n[o]();return e[o]},set(r,o,t){if(o in n)return n[o].set(t),!0;return console.warn(`[ship-set] hook handler tried to write s.${o} = … but ${o} is not a hook state key. The assignment was ignored.`),!0},has(r,o){return o in n||o in e},ownKeys(){return Object.keys(n)},getOwnPropertyDescriptor(r,o){if(o in n)return{configurable:!0,enumerable:!0,writable:!0,value:n[o]()};return}})}var Pr="__shipSetLifecycle";function Mr(n,e){let r=Zo(n,e),o={ship:e.ship,route:e.route??{},navigate:e.navigate??((a)=>{}),...e.scope??{}},t=r.scope?r.scope(o,e.host):{};Object.assign(o,t);let i=Qo(o,e.stateKeys,t),f=Vo(i,o),c={...o},m=Ko(r,c);Object.assign(c,m);let p=$o(r,c,e);for(let[a,s]of Object.entries(p))c[a]=s;let l=Io(r,f),d={...t,...m,...l,...p};if(Object.keys(t).length>0&&e.host){let a=Ue(t);if(Object.keys(a).length>0){let s=e.host,u=s.tagName.toLowerCase();Ge(s,u,a)}}if(r.mount||r.unmount){let a={state:f};if(r.mount)a.mount=r.mount;if(r.unmount)a.unmount=r.unmount;Object.defineProperty(d,Pr,{value:a,enumerable:!0,configurable:!0,writable:!1})}return d}function Zo(n,e){if(typeof n==="function")return n({ship:e.ship});return n}function Qo(n,e,r){let o={},t=e!==void 0,i=r?Object.keys(r):[];if(t||i.length>0){if(e)for(let f of e){let c=n[f];if($e(c))o[f]=c}for(let f of i){if(f in o)continue;let c=n[f];if($e(c))o[f]=c}return o}for(let[f,c]of Object.entries(n))if($e(c))o[f]=c;return o}function $e(n){return typeof n==="function"&&"set"in n&&typeof n.set==="function"}function Vo(n,e){return new Proxy({},{get(r,o){if(o in n)return n[o]();return e[o]},set(r,o,t){if(o in n){if(typeof t==="function")n[o].set(()=>t);else n[o].set(t);return!0}return console.warn(`[ship-set] handler tried to write s.${o} = … but ${o} is not declared as <state name="${o}"> in the template. The assignment was ignored.`),!0},has(r,o){return o in n||o in e},ownKeys(){return Object.keys(n)},getOwnPropertyDescriptor(r,o){if(o in n)return{configurable:!0,enumerable:!0,writable:!0,value:n[o]()};return}})}function Ko(n,e){let r=n.derived??{},o={};for(let[t,i]of Object.entries(r))o[t]=Z(()=>i(e));return o}function Io(n,e){let r=n.on??{},o={};for(let[t,i]of Object.entries(r))o[t]=(...f)=>i(e,...f);return o}function $o(n,e,r){if(n.use===void 0)return{};let o=typeof n.use==="function"?n.use(e):n.use,t={};for(let[i,f]of Object.entries(o))t[i]=Ie(f,r,e);return t}function P(n){return{setup:(e)=>Mr(n,e),__def:n}}if(typeof customElements<"u"){for(let n of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(n))customElements.define(n,class extends HTMLElement{})}var ei=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,ri=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function Nr(n){if(typeof CSSStyleSheet>"u")return null;try{let e=new CSSStyleSheet;return e.replaceSync(n),e}catch{return null}}var Sr=Nr(ei),Hr=Nr(ri);function L(n){if(typeof ShadowRoot>"u")return;if(!(n instanceof ShadowRoot))return;let e=[Hr,Sr].filter((r)=>r!=null);if(e.length===0)return;try{n.adoptedStyleSheets=[...e,...n.adoptedStyleSheets??[]]}catch{}}var Pu=z("sui-status-dot",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"success",reflect:!0}),pulse:y.bool({default:!1,reflect:!0}),size:y.union(["sm","md"],{default:"md",reflect:!0}),label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.tone=e.tone,t.pulse=e.pulse,t.size=e.size,t.label=e.label;let i=r.shadowRoot??r;L(i);let f=document.createElement("span");return f.setAttribute("class","dot"),f.setAttribute("part","dot"),o.push(k(f,"role",()=>t.label()?"img":null)),o.push(k(f,"aria-label",()=>t.label()||null)),o.push(k(f,"aria-hidden",()=>t.label()?null:"true")),i.appendChild(f),t}});var Su=z("sui-spinner",{props:{size:y.union(["xs","sm","md","lg"],{default:"md",reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),label:y.string({default:"Loading"})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.size=e.size,t.tone=e.tone,t.label=e.label;let i=r.shadowRoot??r;L(i);let f=document.createElement("span");return f.setAttribute("class","ring"),f.setAttribute("part","ring"),o.push(k(f,"role",()=>t.label()?"status":null)),o.push(k(f,"aria-label",()=>t.label()||null)),o.push(k(f,"aria-hidden",()=>t.label()?null:"true")),i.appendChild(f),t}});var Fu=z("sui-badge",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["solid","soft","outline"],{default:"soft",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),pill:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.tone=e.tone,t.variant=e.variant,t.size=e.size,t.pill=e.pill;let i=r.shadowRoot??r;L(i);let f=document.createElement("span");f.setAttribute("class","badge"),f.setAttribute("part","badge");let c=document.createElement("span");c.setAttribute("class","leading"),c.setAttribute("part","leading");let m=document.createElement("slot");m.setAttribute("name","leading"),c.appendChild(m),f.appendChild(c);let p=document.createElement("slot");return f.appendChild(p),i.appendChild(f),t}});var Vt=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","button");if(!e.hasAttribute("tabindex")&&!r.disabled())e.setAttribute("tabindex","0");let o=D(()=>{let f=r.selected(),c=r.disabled();if(e.setAttribute("aria-pressed",f?"true":"false"),c){if(e.setAttribute("aria-disabled","true"),e.getAttribute("tabindex")!=="-1")e.setAttribute("tabindex","-1")}else if(e.removeAttribute("aria-disabled"),e.getAttribute("tabindex")==="-1")e.setAttribute("tabindex","0")}),t=(f)=>{if(r.disabled())return;if(f.key==="Enter"||f.key===" ")f.preventDefault(),e.click();else if(r.removable()&&(f.key==="Backspace"||f.key==="Delete"))f.preventDefault(),e.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};e.addEventListener("keydown",t);let i=D(()=>{let f=r.removeBtn?.();if(!f)return;let c=(m)=>{if(m.stopPropagation(),r.disabled())return;e.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};return f.addEventListener("click",c),()=>f.removeEventListener("click",c)});return()=>{o(),e.removeEventListener("keydown",t),i()}}});var Ju=z("sui-chip",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0}),variant:y.union(["solid","soft","outline","ghost"],{default:"soft",reflect:!0}),size:y.union(["sm","md","lg"],{default:"sm",reflect:!0}),selected:y.bool({default:!1,reflect:!0}),removable:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.tone=e.tone,t.variant=e.variant,t.size=e.size,t.selected=e.selected,t.removable=e.removable,t.disabled=e.disabled;let i=null;if(Vt&&typeof Vt.setup==="function"){if(n.scope=t,i=Vt.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(s)=>B({anchor:s,test:()=>{if(t.removable())return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("button");h.setAttribute("class","remove"),h.setAttribute("part","remove"),h.setAttribute("type","button"),h.setAttribute("aria-label","Remove"),h.setAttribute("tabindex","-1"),T(t,"removeBtn",h);let v=document.createElement("svg");v.setAttribute("viewbox","0 0 10 10"),v.setAttribute("aria-hidden","true");let g=document.createElement("path");return g.setAttribute("d","M2 2 L8 8 M8 2 L2 8"),v.appendChild(g),h.appendChild(v),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let x of b)try{x()}catch(A){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("span");m.setAttribute("class","chip"),m.setAttribute("part","chip");let p=document.createElement("span");p.setAttribute("class","leading"),p.setAttribute("part","leading");let l=document.createElement("slot");l.setAttribute("name","leading"),p.appendChild(l),m.appendChild(p);let d=document.createElement("slot");m.appendChild(d);let a=document.createComment("if");if(m.appendChild(a),o.push(f(a)),c.appendChild(m),i!=null){let s=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,v){let g=b[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var Kt=P({mount(n,e){let r=n,o=(i)=>{if(i.key==="Enter"||i.key===" ")i.preventDefault(),e.click()},t=D(()=>{if(r.interactive()){if(!e.hasAttribute("role"))e.setAttribute("role","button");if(!e.hasAttribute("tabindex"))e.setAttribute("tabindex","0");return e.addEventListener("keydown",o),()=>e.removeEventListener("keydown",o)}else{if(e.getAttribute("role")==="button")e.removeAttribute("role");if(e.getAttribute("tabindex")==="0")e.removeAttribute("tabindex")}});return()=>{t()}}});var Ku=z("sui-card",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["default","outline"],{default:"default",reflect:!0}),padding:y.union(["none","sm","md","lg"],{default:"md",reflect:!0}),interactive:y.bool({default:!1,reflect:!0}),"accent-bar":y.bool({default:!1,reflect:!0}),"status-rail":y.bool({default:!1,reflect:!0}),fill:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.tone=e.tone,t.variant=e.variant,t.padding=e.padding,t.interactive=e.interactive,t.accentBar=e.accentBar,t.statusRail=e.statusRail,t.fill=e.fill;let i=null;if(Kt&&typeof Kt.setup==="function"){if(n.scope=t,i=Kt.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","header"),c.setAttribute("part","header");let m=document.createElement("slot");m.setAttribute("name","header"),o.push(nt(m,r,"data-no-header")),c.appendChild(m),f.appendChild(c);let p=document.createElement("div");p.setAttribute("class","content"),p.setAttribute("part","content");let l=document.createElement("slot");p.appendChild(l),f.appendChild(p);let d=document.createElement("div");d.setAttribute("class","footer"),d.setAttribute("part","footer");let a=document.createElement("slot");if(a.setAttribute("name","footer"),o.push(nt(a,r,"data-no-footer")),d.appendChild(a),f.appendChild(d),i!=null){let s=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,v){let g=b[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var _u=z("sui-link",{props:{href:y.string({default:""}),external:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["default","underline","subtle"],{default:"default",reflect:!0}),target:y.string({default:""}),rel:y.string({default:""})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.href=e.href,t.external=e.external,t.tone=e.tone,t.variant=e.variant,t.target=e.target,t.rel=e.rel;let i=(l)=>B({anchor:l,test:()=>{if(t.external())return 0;return-1},branches:[()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("span");s.setAttribute("class","sr-only");let u=document.createTextNode("(opens in new tab)");s.appendChild(u),d.appendChild(s);let b=document.createElement("span");b.setAttribute("class","ext"),b.setAttribute("aria-hidden","true");let h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round");let v=document.createElement("path");return v.setAttribute("d","M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3"),h.appendChild(v),b.appendChild(h),d.appendChild(b),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let g of a)try{g()}catch(x){}}}}]}),f=r.shadowRoot??r;L(f);let c=document.createElement("a");c.setAttribute("part","link"),o.push(k(c,"href",()=>t.href())),o.push(k(c,"target",()=>t.external()?"_blank":t.target())),o.push(k(c,"rel",()=>t.external()?"noopener noreferrer":t.rel()));let m=document.createElement("slot");c.appendChild(m);let p=document.createComment("if");return c.appendChild(p),o.push(i(p)),f.appendChild(c),t}});var el=z("sui-divider",{props:{orientation:y.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:y.union(["solid","dashed"],{default:"solid",reflect:!0}),tone:y.union(["default","muted","primary","destructive"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.orientation=e.orientation,t.variant=e.variant,t.tone=e.tone;let i=r.shadowRoot??r;L(i);let f=document.createElement("hr");return f.setAttribute("class","line"),f.setAttribute("part","line"),f.setAttribute("role","separator"),o.push(k(f,"aria-orientation",()=>t.orientation())),i.appendChild(f),t}});var il=z("sui-skeleton",{props:{shape:y.union(["text","line","block","circle"],{default:"text",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.shape=e.shape,t.size=e.size;let i=r.shadowRoot??r;L(i);let f=document.createElement("span");return f.setAttribute("class","sk"),f.setAttribute("part","sk"),f.setAttribute("role","status"),f.setAttribute("aria-label","Loading"),f.setAttribute("aria-live","polite"),f.setAttribute("aria-busy","true"),i.appendChild(f),t}});var It=P({mount(n,e){let r=n,o=(i)=>{if(i.key==="Enter"||i.key===" ")i.preventDefault(),e.click()},t=D(()=>{if(r.interactive()){if(!e.hasAttribute("role"))e.setAttribute("role","button");if(!e.hasAttribute("tabindex"))e.setAttribute("tabindex","0");return e.addEventListener("keydown",o),()=>e.removeEventListener("keydown",o)}else{if(e.getAttribute("role")==="button")e.removeAttribute("role");if(e.getAttribute("tabindex")==="0")e.removeAttribute("tabindex")}});return()=>{t(),e.removeEventListener("keydown",o)}}});var cl=z("sui-stat",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent"],{default:"default",reflect:!0}),interactive:y.bool({default:!1,reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.tone=e.tone,t.interactive=e.interactive,t.size=e.size;let i=null;if(It&&typeof It.setup==="function"){if(n.scope=t,i=It.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("span");c.setAttribute("class","label"),c.setAttribute("part","label");let m=document.createElement("slot");m.setAttribute("name","label"),c.appendChild(m),f.appendChild(c);let p=document.createElement("span");p.setAttribute("class","value-row"),p.setAttribute("part","value-row");let l=document.createElement("span");l.setAttribute("class","value"),l.setAttribute("part","value");let d=document.createElement("slot");l.appendChild(d),p.appendChild(l);let a=document.createElement("span");a.setAttribute("class","trend"),a.setAttribute("part","trend");let s=document.createElement("slot");s.setAttribute("name","trend"),a.appendChild(s),p.appendChild(a),f.appendChild(p);let u=document.createElement("span");u.setAttribute("class","hint"),u.setAttribute("part","hint");let b=document.createElement("slot");if(b.setAttribute("name","hint"),u.appendChild(b),f.appendChild(u),i!=null){let h=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(g,x){return g[x]},set(g,x,A){let w=g[x];if(w!=null&&typeof w.set==="function"){if(typeof A==="function")w.set(()=>A);else w.set(A);return!0}return!1}});if(h.mount){let g=h.mount(v,r);if(typeof g==="function")o.push(g)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var ml=z("sui-mark",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"warning",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.tone=e.tone;let i=r.shadowRoot??r;L(i);let f=document.createElement("mark");f.setAttribute("part","mark");let c=document.createElement("slot");return f.appendChild(c),i.appendChild(f),t}});var $t=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","group");let o=D(()=>{let t=r.keys();if(t)e.setAttribute("aria-label",`Keyboard shortcut: ${t}`);else e.removeAttribute("aria-label")});return()=>o()}});var xl=z("sui-kbd",{props:{keys:y.string({default:""}),size:y.union(["sm","md"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.keys=e.keys,t.size=e.size;let i=null;if($t&&typeof $t.setup==="function"){if(n.scope=t,i=$t.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("kbd");c.setAttribute("part","kbd");let m=document.createElement("slot");if(c.appendChild(m),f.appendChild(c),i!=null){let p=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(d,a){return d[a]},set(d,a,s){let u=d[a];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(p.mount){let d=p.mount(l,r);if(typeof d==="function")o.push(d)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var El=z("sui-avatar",{props:{src:y.string({default:""}),name:y.string({default:""}),initials:y.string({default:""}),"icon-slug":y.string({default:""}),size:y.union(["xs","sm","md","lg","xl"],{default:"md",reflect:!0}),shape:y.union(["round","square"],{default:"round",reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.src=e.src,t.name=e.name,t.initials=e.initials,t.iconSlug=e.iconSlug,t.size=e.size,t.shape=e.shape,t.tone=e.tone;let i=(p)=>B({anchor:p,test:()=>{if(t.iconSlug()||t.src())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),d=[],a=document.createElement("img");return a.setAttribute("part","img"),d.push(k(a,"src",()=>t.iconSlug()?"https://cdn.simpleicons.org/"+t.iconSlug():t.src())),d.push(k(a,"alt",()=>t.name())),l.appendChild(a),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let s of d)try{s()}catch(u){}}}},()=>{let l=document.createDocumentFragment(),d=[],a=document.createElement("span");a.setAttribute("part","initials");let s=document.createTextNode("");return a.appendChild(s),d.push(F(s,()=>t.initials()?t.initials().slice(0,2).toUpperCase():t.name().trim().slice(0,1).toUpperCase()||"?")),l.appendChild(a),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let u of d)try{u()}catch(b){}}}}]}),f=r.shadowRoot??r;L(f);let c=document.createElement("span");c.setAttribute("class","root"),c.setAttribute("part","root"),c.setAttribute("role","img"),o.push(k(c,"aria-label",()=>t.name()||t.initials()||"avatar"));let m=document.createComment("if");return c.appendChild(m),o.push(i(m)),f.appendChild(c),t}});var jl=z("sui-progress",{props:{value:y.number({default:0}),max:y.number({default:100}),tone:y.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),indeterminate:y.bool({default:!1,reflect:!0}),label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.max=e.max,t.tone=e.tone,t.size=e.size,t.indeterminate=e.indeterminate,t.label=e.label;let i=r.shadowRoot??r;L(i);let f=document.createElement("div");f.setAttribute("class","track"),f.setAttribute("part","track"),f.setAttribute("role","progressbar"),o.push(k(f,"aria-valuenow",()=>t.indeterminate()?null:t.value())),f.setAttribute("aria-valuemin","0"),o.push(k(f,"aria-valuemax",()=>t.max())),o.push(k(f,"aria-busy",()=>t.indeterminate()?"true":"false")),o.push(k(f,"aria-label",()=>t.label()));let c=document.createElement("div");return c.setAttribute("class","fill"),c.setAttribute("part","fill"),o.push(k(c,"style",()=>t.indeterminate()?"":"width: clamp(0%, "+t.value()/(t.max()||1)*100+"%, 100%)")),f.appendChild(c),i.appendChild(f),t}});var Dl=z("sui-progress-ring",{props:{value:y.number({default:0}),max:y.number({default:100}),size:y.number({default:48}),"track-width":y.number({default:4}),tone:y.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),indeterminate:y.bool({default:!1,reflect:!0}),label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.max=e.max,t.size=e.size,t.trackWidth=e.trackWidth,t.tone=e.tone,t.indeterminate=e.indeterminate,t.label=e.label;let i=r.shadowRoot??r;L(i);let f=document.createElement("div");f.setAttribute("class","ring"),o.push(k(f,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let c=document.createElement("svg");c.setAttribute("class","ring-svg"),o.push(k(c,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),c.setAttribute("role","progressbar"),c.setAttribute("aria-valuemin","0"),o.push(k(c,"aria-valuemax",()=>t.max())),o.push(k(c,"aria-valuenow",()=>t.indeterminate()?null:t.value()<0?0:t.value()>t.max()?t.max():t.value())),o.push(k(c,"aria-label",()=>t.label()||null)),c.setAttribute("xmlns","http://www.w3.org/2000/svg");let m=document.createElement("circle");m.setAttribute("class","track"),m.setAttribute("part","track"),o.push(k(m,"cx",()=>t.size()/2)),o.push(k(m,"cy",()=>t.size()/2)),o.push(k(m,"r",()=>(t.size()-t.trackWidth())/2)),m.setAttribute("fill","none"),o.push(k(m,"stroke-width",()=>t.trackWidth())),c.appendChild(m);let p=document.createElement("circle");p.setAttribute("class","bar"),p.setAttribute("part","ring"),o.push(k(p,"cx",()=>t.size()/2)),o.push(k(p,"cy",()=>t.size()/2)),o.push(k(p,"r",()=>(t.size()-t.trackWidth())/2)),p.setAttribute("fill","none"),o.push(k(p,"stroke-width",()=>t.trackWidth())),p.setAttribute("stroke-linecap","round"),o.push(k(p,"stroke-dasharray",()=>3.141592653589793*(t.size()-t.trackWidth()))),o.push(k(p,"stroke-dashoffset",()=>t.indeterminate()?3.141592653589793*(t.size()-t.trackWidth())*0.25:3.141592653589793*(t.size()-t.trackWidth())*(1-(t.value()<0?0:t.value()>(t.max()||1)?1:t.value()/(t.max()||1))))),c.appendChild(p),f.appendChild(c);let l=document.createElement("div");l.setAttribute("class","label"),l.setAttribute("aria-hidden","true");let d=document.createElement("slot");return l.appendChild(d),f.appendChild(l),i.appendChild(f),t}});var _e={default:"var(--foreground)",primary:"var(--primary)",info:"var(--info)",success:"var(--success)",warning:"var(--warning)",destructive:"var(--destructive)",accent:"var(--accent)",muted:"var(--muted-foreground)"};function $(n){if(!n)return _e.default;return _e[n]??_e.default}function kt(n){if(!n)return[];return n.split(",").map((e)=>Number.parseFloat(e.trim())).filter((e)=>Number.isFinite(e))}function Nt(n){if(!n)return[];return n.split(",").map((e)=>e.trim())}function it(n,e){if(!n)return e;try{return JSON.parse(n)}catch{return e}}function Ct(n,e,r,o=2){if(n.length===0)return{d:"",points:[]};let t=Math.min(...n),f=Math.max(...n)-t||1,c=r-o*2,m=n.length===1?0:e/(n.length-1),p=n.map((d,a)=>{let s=n.length===1?e/2:a*m,u=o+c-(d-t)/f*c;return[s,u]});return{d:p.map(([d,a],s)=>`${s===0?"M":"L"}${d.toFixed(2)},${a.toFixed(2)}`).join(" "),points:p}}function Br(n,e,r,o){return[n+r*Math.cos(o),e+r*Math.sin(o)]}function St(n,e,r,o,t){let[i,f]=Br(n,e,r,o),[c,m]=Br(n,e,r,t),p=t-o>Math.PI?1:0,l=t>o?1:0;return`M${i.toFixed(2)},${f.toFixed(2)} A${r},${r} 0 ${p} ${l} ${c.toFixed(2)},${m.toFixed(2)}`}var _t=P({scope:()=>{let n=M([]),e=M([]);return{seriesPaths:n,gridLines:e}},mount(n,e){let r=D(()=>{let o=it(n.series(),[]),t=n.width(),i=n.height(),f=o.flatMap((a)=>a.data),c=f.length?Math.min(...f):0,p=(f.length?Math.max(...f):1)-c||1,l=i-4;n.gridLines.set(n.grid()?[0.25,0.5,0.75].map((a,s)=>({id:`g${s}`,y:2+l*a})):[]);let d=[];for(let a of o){if(a.data.length<2)continue;let s=a.data.map((A)=>(A-c)/p*100),{d:u,points:b}=Ct(s,t,i,2);if(!u)continue;let h=$(a.tone),v=b[0],g=b[b.length-1],x=`${u} L${g[0].toFixed(2)},${i} L${v[0].toFixed(2)},${i} Z`;d.push({name:a.name,areaD:x,strokeD:u,color:h,pts:b.map(([A,w],E)=>({id:`${a.name}-${E}`,cx:A,cy:w}))})}n.seriesPaths.set(d)});return()=>r()}});var Bl=z("sui-area-chart",{props:{series:y.string({default:"",reflect:!0}),labels:y.string({default:""}),width:y.number({default:300}),height:y.number({default:200}),points:y.bool({default:!1,reflect:!0}),grid:y.bool({default:!1,reflect:!0}),label:y.string({default:"area chart"})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.series=e.series,t.labels=e.labels,t.width=e.width,t.height=e.height,t.points=e.points,t.grid=e.grid,t.label=e.label;let i=null;if(_t&&typeof _t.setup==="function"){if(n.scope=t,i=_t.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(b)=>R({anchor:b,items:()=>t.gridLines(),key:(h,v)=>{let g=Object.create(t);return g.g=h,g.__i=v,((x)=>h.id)(g)},render:(h,v)=>{let g=Object.create(t);return g.g=h,g.__i=v,g.__v=M(0),((x)=>{let A=document.createDocumentFragment(),w=[],E=document.createElement("line");return E.setAttribute("class","grid-line"),E.setAttribute("x1","0"),w.push(k(E,"x2",()=>(x.__v(),x.width()))),w.push(k(E,"y1",()=>(x.__v(),x.g.y))),w.push(k(E,"y2",()=>(x.__v(),x.g.y))),A.appendChild(E),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let j of w)try{j()}catch(C){}},update:(j,C)=>{x.g=j,x.__i=C,x.__v.set((N)=>N+1)}}})(g)}}),c=(b)=>R({anchor:b,items:()=>t.seriesPaths(),key:(h,v)=>{let g=Object.create(t);return g.s=h,g.__i=v,((x)=>h.name)(g)},render:(h,v)=>{let g=Object.create(t);return g.s=h,g.__i=v,g.__v=M(0),((x)=>{let A=(S)=>R({anchor:S,items:()=>x.s.pts,key:(H,U)=>{let G=Object.create(x);return G.p=H,G.__i=U,((X)=>H.id)(G)},render:(H,U)=>{let G=Object.create(x);return G.p=H,G.__i=U,G.__v=M(0),((X)=>{let Y=document.createDocumentFragment(),W=[],q=document.createElement("circle");return W.push(k(q,"cx",()=>(X.__v(),X.p.cx))),W.push(k(q,"cy",()=>(X.__v(),X.p.cy))),q.setAttribute("r","2.5"),W.push(k(q,"fill",()=>(X.__v(),X.s.color))),Y.appendChild(q),{nodes:Array.from(Y.childNodes),cleanup:()=>{for(let _ of W)try{_()}catch(V){}},update:(_,V)=>{X.p=_,X.__i=V,X.__v.set((I)=>I+1)}}})(G)}}),w=(S)=>B({anchor:S,test:()=>{if(x.points())return 0;return-1},branches:[()=>{let H=document.createDocumentFragment(),U=[],G=document.createComment("for");return H.appendChild(G),U.push(A(G)),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let X of U)try{X()}catch(Y){}}}}]}),E=document.createDocumentFragment(),j=[],C=document.createElement("path");j.push(k(C,"d",()=>(x.__v(),x.s.areaD))),j.push(k(C,"fill",()=>(x.__v(),x.s.color))),C.setAttribute("fill-opacity","0.3"),C.setAttribute("stroke","none"),E.appendChild(C);let N=document.createElement("path");j.push(k(N,"d",()=>(x.__v(),x.s.strokeD))),N.setAttribute("fill","none"),j.push(k(N,"stroke",()=>(x.__v(),x.s.color))),N.setAttribute("stroke-width","1.75"),N.setAttribute("stroke-linecap","round"),N.setAttribute("stroke-linejoin","round"),E.appendChild(N);let O=document.createComment("if");return E.appendChild(O),j.push(w(O)),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let S of j)try{S()}catch(H){}},update:(S,H)=>{x.s=S,x.__i=H,x.__v.set((U)=>U+1)}}})(g)}}),m=r.shadowRoot??r;L(m);let p=document.createElement("div");p.setAttribute("class","wrap");let l=document.createElement("svg");o.push(k(l,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),l.setAttribute("preserveaspectratio","none"),l.setAttribute("role","img"),o.push(k(l,"aria-label",()=>t.label()));let d=document.createComment("for");l.appendChild(d),o.push(f(d));let a=document.createComment("for");l.appendChild(a),o.push(c(a)),p.appendChild(l);let s=document.createElement("div");s.setAttribute("class","legend");let u=document.createElement("slot");if(s.appendChild(u),p.appendChild(s),m.appendChild(p),i!=null){let b=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(b){let h=new Proxy(t,{get(v,g){return v[g]},set(v,g,x){let A=v[g];if(A!=null&&typeof A.set==="function"){if(typeof x==="function")A.set(()=>x);else A.set(x);return!0}return!1}});if(b.mount){let v=b.mount(h,r);if(typeof v==="function")o.push(v)}if(b.unmount)o.push(()=>b.unmount(h))}}return t}});var tn=P({scope:()=>{let n=M([]),e=M(!0);return{bars:n,isEmpty:e}},mount(n,e){let r=D(()=>{let o=kt(n.data());if(o.length===0){n.bars.set([]),n.isEmpty.set(!0);return}let t=Nt(n.labels()),i=n.width(),f=n.height(),c=n.max(),m=c>0?c:Math.max(...o,0),p=m<=0?1:m,l=$(n.tone()),d=4,a=3,s=n.orientation()==="horizontal",u=o.length;n.bars.set(o.map((b,h)=>{let v=Math.max(0,b)/p,g,x,A,w;if(s)w=(f-d*(u-1))/u,x=h*(w+d),A=v*i,g=0;else A=(i-d*(u-1))/u,g=h*(A+d),w=v*f,x=f-w;return{id:`b${h}`,x:g,y:x,w:Math.max(0,A),h:Math.max(0,w),rx:a,ry:a,color:l,title:`${t[h]??String(b)}: ${b}`}})),n.isEmpty.set(!1)});return()=>r()}});var Jl=z("sui-bar-chart",{props:{data:y.string({default:"",reflect:!0}),labels:y.string({default:""}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),orientation:y.union(["vertical","horizontal"],{default:"vertical",reflect:!0}),width:y.number({default:300}),height:y.number({default:200}),max:y.number({default:0}),label:y.string({default:"bar chart"})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.data=e.data,t.labels=e.labels,t.tone=e.tone,t.orientation=e.orientation,t.width=e.width,t.height=e.height,t.max=e.max,t.label=e.label;let i=null;if(tn&&typeof tn.setup==="function"){if(n.scope=t,i=tn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(l)=>R({anchor:l,items:()=>t.bars(),key:(d,a)=>{let s=Object.create(t);return s.b=d,s.__i=a,((u)=>d.id)(s)},render:(d,a)=>{let s=Object.create(t);return s.b=d,s.__i=a,s.__v=M(0),((u)=>{let b=document.createDocumentFragment(),h=[],v=document.createElement("rect");h.push(k(v,"x",()=>(u.__v(),u.b.x))),h.push(k(v,"y",()=>(u.__v(),u.b.y))),h.push(k(v,"width",()=>(u.__v(),u.b.w))),h.push(k(v,"height",()=>(u.__v(),u.b.h))),h.push(k(v,"rx",()=>(u.__v(),u.b.rx))),h.push(k(v,"ry",()=>(u.__v(),u.b.ry))),h.push(k(v,"fill",()=>(u.__v(),u.b.color)));let g=document.createElement("title"),x=document.createTextNode("");return g.appendChild(x),h.push(F(x,()=>(u.__v(),u.b.title))),v.appendChild(g),b.appendChild(v),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let A of h)try{A()}catch(w){}},update:(A,w)=>{u.b=A,u.__i=w,u.__v.set((E)=>E+1)}}})(s)}}),c=(l)=>B({anchor:l,test:()=>{if(t.isEmpty())return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("svg");return a.push(k(s,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),s.setAttribute("aria-hidden","true"),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let u of a)try{u()}catch(b){}}}},()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("svg");a.push(k(s,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),s.setAttribute("preserveaspectratio","none"),s.setAttribute("role","img"),a.push(k(s,"aria-label",()=>t.label()));let u=document.createComment("for");return s.appendChild(u),a.push(f(u)),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let b of a)try{b()}catch(h){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createComment("if");if(m.appendChild(p),o.push(c(p)),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var nn=P({scope:()=>{let n=M([]),e=M([]);return{seriesPaths:n,gridLines:e}},mount(n,e){let r=D(()=>{let o=it(n.series(),[]),t=n.width(),i=n.height(),f=o.flatMap((a)=>a.data),c=f.length?Math.min(...f):0,p=(f.length?Math.max(...f):1)-c||1,l=i-4;n.gridLines.set(n.grid()?[0.25,0.5,0.75].map((a,s)=>({id:`g${s}`,y:2+l*a})):[]);let d=[];for(let a of o){if(a.data.length<2)continue;let s=a.data.map((v)=>(v-c)/p*100),{d:u,points:b}=Ct(s,t,i,2);if(!u)continue;let h=$(a.tone);d.push({name:a.name,strokeD:u,color:h,pts:b.map(([v,g],x)=>({id:`${a.name}-${x}`,cx:v,cy:g}))})}n.seriesPaths.set(d)});return()=>r()}});var Il=z("sui-line-chart",{props:{series:y.string({default:"",reflect:!0}),labels:y.string({default:""}),width:y.number({default:300}),height:y.number({default:200}),points:y.bool({default:!1,reflect:!0}),grid:y.bool({default:!1,reflect:!0}),label:y.string({default:"line chart"})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.series=e.series,t.labels=e.labels,t.width=e.width,t.height=e.height,t.points=e.points,t.grid=e.grid,t.label=e.label;let i=null;if(nn&&typeof nn.setup==="function"){if(n.scope=t,i=nn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(b)=>R({anchor:b,items:()=>t.gridLines(),key:(h,v)=>{let g=Object.create(t);return g.g=h,g.__i=v,((x)=>h.id)(g)},render:(h,v)=>{let g=Object.create(t);return g.g=h,g.__i=v,g.__v=M(0),((x)=>{let A=document.createDocumentFragment(),w=[],E=document.createElement("line");return E.setAttribute("class","grid-line"),E.setAttribute("x1","0"),w.push(k(E,"x2",()=>(x.__v(),x.width()))),w.push(k(E,"y1",()=>(x.__v(),x.g.y))),w.push(k(E,"y2",()=>(x.__v(),x.g.y))),A.appendChild(E),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let j of w)try{j()}catch(C){}},update:(j,C)=>{x.g=j,x.__i=C,x.__v.set((N)=>N+1)}}})(g)}}),c=(b)=>R({anchor:b,items:()=>t.seriesPaths(),key:(h,v)=>{let g=Object.create(t);return g.s=h,g.__i=v,((x)=>h.name)(g)},render:(h,v)=>{let g=Object.create(t);return g.s=h,g.__i=v,g.__v=M(0),((x)=>{let A=(O)=>R({anchor:O,items:()=>x.s.pts,key:(S,H)=>{let U=Object.create(x);return U.p=S,U.__i=H,((G)=>S.id)(U)},render:(S,H)=>{let U=Object.create(x);return U.p=S,U.__i=H,U.__v=M(0),((G)=>{let X=document.createDocumentFragment(),Y=[],W=document.createElement("circle");return Y.push(k(W,"cx",()=>(G.__v(),G.p.cx))),Y.push(k(W,"cy",()=>(G.__v(),G.p.cy))),W.setAttribute("r","2.5"),Y.push(k(W,"fill",()=>(G.__v(),G.s.color))),X.appendChild(W),{nodes:Array.from(X.childNodes),cleanup:()=>{for(let q of Y)try{q()}catch(_){}},update:(q,_)=>{G.p=q,G.__i=_,G.__v.set((V)=>V+1)}}})(U)}}),w=(O)=>B({anchor:O,test:()=>{if(x.points())return 0;return-1},branches:[()=>{let S=document.createDocumentFragment(),H=[],U=document.createComment("for");return S.appendChild(U),H.push(A(U)),{nodes:Array.from(S.childNodes),cleanup:()=>{for(let G of H)try{G()}catch(X){}}}}]}),E=document.createDocumentFragment(),j=[],C=document.createElement("path");j.push(k(C,"d",()=>(x.__v(),x.s.strokeD))),C.setAttribute("fill","none"),j.push(k(C,"stroke",()=>(x.__v(),x.s.color))),C.setAttribute("stroke-width","1.75"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),E.appendChild(C);let N=document.createComment("if");return E.appendChild(N),j.push(w(N)),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let O of j)try{O()}catch(S){}},update:(O,S)=>{x.s=O,x.__i=S,x.__v.set((H)=>H+1)}}})(g)}}),m=r.shadowRoot??r;L(m);let p=document.createElement("div");p.setAttribute("class","wrap");let l=document.createElement("svg");o.push(k(l,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),l.setAttribute("preserveaspectratio","none"),l.setAttribute("role","img"),o.push(k(l,"aria-label",()=>t.label()));let d=document.createComment("for");l.appendChild(d),o.push(f(d));let a=document.createComment("for");l.appendChild(a),o.push(c(a)),p.appendChild(l);let s=document.createElement("div");s.setAttribute("class","legend");let u=document.createElement("slot");if(s.appendChild(u),p.appendChild(s),m.appendChild(p),i!=null){let b=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(b){let h=new Proxy(t,{get(v,g){return v[g]},set(v,g,x){let A=v[g];if(A!=null&&typeof A.set==="function"){if(typeof x==="function")A.set(()=>x);else A.set(x);return!0}return!1}});if(b.mount){let v=b.mount(h,r);if(typeof v==="function")o.push(v)}if(b.unmount)o.push(()=>b.unmount(h))}}return t}});var en=P({scope:()=>{let n=M(!1),e=M(""),r=M(""),o=M("var(--primary)");return{hasData:n,strokePath:e,areaPath:r,lineColor:o}},mount(n,e){let r=D(()=>{let o=kt(n.data()),t=n.width(),i=n.height(),f=$(n.tone());if(n.lineColor.set(f),o.length<2){n.hasData.set(!1),n.strokePath.set(""),n.areaPath.set("");return}let{d:c,points:m}=Ct(o,t,i,2);if(n.strokePath.set(c),n.fill()){let p=m[0],l=m[m.length-1];n.areaPath.set(`${c} L${l[0].toFixed(2)},${i} L${p[0].toFixed(2)},${i} Z`)}else n.areaPath.set("");n.hasData.set(!0)});return()=>r()}});var od=z("sui-sparkline",{props:{data:y.string({default:"",reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),width:y.number({default:80}),height:y.number({default:20}),fill:y.bool({default:!1,reflect:!0}),label:y.string({default:"sparkline"})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.data=e.data,t.tone=e.tone,t.width=e.width,t.height=e.height,t.fill=e.fill,t.label=e.label;let i=null;if(en&&typeof en.setup==="function"){if(n.scope=t,i=en.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(l)=>B({anchor:l,test:()=>{if(t.fill())return 0;return-1},branches:[()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("path");return a.push(k(s,"d",()=>t.areaPath())),a.push(k(s,"fill",()=>t.lineColor())),s.setAttribute("fill-opacity","0.18"),s.setAttribute("stroke","none"),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let u of a)try{u()}catch(b){}}}}]}),c=(l)=>B({anchor:l,test:()=>{if(t.hasData())return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("svg");a.push(k(s,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),a.push(k(s,"width",()=>t.width())),a.push(k(s,"height",()=>t.height())),s.setAttribute("role","img"),a.push(k(s,"aria-label",()=>t.label()));let u=document.createComment("if");s.appendChild(u),a.push(f(u));let b=document.createElement("path");a.push(k(b,"d",()=>t.strokePath())),b.setAttribute("fill","none"),a.push(k(b,"stroke",()=>t.lineColor())),b.setAttribute("stroke-width","1.5"),b.setAttribute("stroke-linecap","round"),b.setAttribute("stroke-linejoin","round");let h=document.createTextNode(`
    `);return b.appendChild(h),s.appendChild(b),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let v of a)try{v()}catch(g){}}}},()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("svg");a.push(k(s,"viewbox",()=>"0 0 "+t.width()+" "+t.height())),a.push(k(s,"width",()=>t.width())),a.push(k(s,"height",()=>t.height())),s.setAttribute("aria-hidden","true");let u=document.createTextNode(`
  `);return s.appendChild(u),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let b of a)try{b()}catch(h){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createComment("if");if(m.appendChild(p),o.push(c(p)),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var rn=P({scope:()=>({segments:M([])}),mount(n,e){let r=D(()=>{let o=it(n.data(),[]),t=o.reduce((a,s)=>a+Math.max(0,s.value),0);if(t<=0){n.segments.set([]);return}let i=n.size(),f=n.thickness(),c=(i-f)/2,m=i/2,p=i/2,l=2*Math.PI,d=-Math.PI/2;n.segments.set(o.filter((a)=>a.value>0).map((a,s)=>{let u=a.value/t*l,b=u>=l-0.001?l-0.001:u,h=St(m,p,c,d,d+b);return d+=u,{id:`s${s}`,d:h,color:$(a.tone),title:`${a.label}: ${a.value}`}}))});return()=>r()}});var cd=z("sui-donut",{props:{data:y.string({default:"",reflect:!0}),size:y.number({default:120}),thickness:y.number({default:16}),label:y.string({default:"donut chart"})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.data=e.data,t.size=e.size,t.thickness=e.thickness,t.label=e.label;let i=null;if(rn&&typeof rn.setup==="function"){if(n.scope=t,i=rn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(u)=>R({anchor:u,items:()=>t.segments(),key:(b,h)=>{let v=Object.create(t);return v.seg=b,v.__i=h,((g)=>b.id)(v)},render:(b,h)=>{let v=Object.create(t);return v.seg=b,v.__i=h,v.__v=M(0),((g)=>{let x=document.createDocumentFragment(),A=[],w=document.createElement("path");w.setAttribute("part","segment"),A.push(k(w,"d",()=>(g.__v(),g.seg.d))),w.setAttribute("fill","none"),A.push(k(w,"stroke",()=>(g.__v(),g.seg.color))),A.push(k(w,"stroke-width",()=>(g.__v(),g.thickness()))),w.setAttribute("stroke-linecap","butt");let E=document.createElement("title"),j=document.createTextNode("");return E.appendChild(j),A.push(F(j,()=>(g.__v(),g.seg.title))),w.appendChild(E),x.appendChild(w),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let C of A)try{C()}catch(N){}},update:(C,N)=>{g.seg=C,g.__i=N,g.__v.set((O)=>O+1)}}})(v)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("svg");o.push(k(m,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),m.setAttribute("role","img"),o.push(k(m,"aria-label",()=>t.label())),o.push(k(m,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let p=document.createElement("circle");p.setAttribute("class","track"),p.setAttribute("part","track"),o.push(k(p,"cx",()=>t.size()/2)),o.push(k(p,"cy",()=>t.size()/2)),o.push(k(p,"r",()=>(t.size()-t.thickness())/2)),p.setAttribute("fill","none"),o.push(k(p,"stroke-width",()=>t.thickness()));let l=document.createTextNode(`
  `);p.appendChild(l),m.appendChild(p);let d=document.createComment("for");m.appendChild(d),o.push(f(d)),c.appendChild(m);let a=document.createElement("div");a.setAttribute("class","center");let s=document.createElement("slot");if(a.appendChild(s),c.appendChild(a),i!=null){let u=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(u){let b=new Proxy(t,{get(h,v){return h[v]},set(h,v,g){let x=h[v];if(x!=null&&typeof x.set==="function"){if(typeof g==="function")x.set(()=>g);else x.set(g);return!0}return!1}});if(u.mount){let h=u.mount(b,r);if(typeof h==="function")o.push(h)}if(u.unmount)o.push(()=>u.unmount(b))}}return t}});var Gr=Math.PI/180,on=135*Gr,Fr=405*Gr,an=P({scope:()=>({trackD:M(""),valueD:M(""),arcColor:M("var(--primary)")}),mount(n,e){let r=D(()=>{let o=n.size(),t=n.thickness(),i=(o-t)/2-2,f=o/2,c=o/2;n.trackD.set(St(f,c,i,on,Fr));let m=n.value(),p=n.min(),l=n.max(),d=Math.max(0,Math.min(1,(m-p)/(l-p||1))),a=on+(Fr-on)*d;n.valueD.set(d>0?St(f,c,i,on,a):"");let s=n.tone(),u=n.lowThreshold(),b=n.highThreshold(),h=s;if(!h)if(u>0&&m<u)h="destructive";else if(b>0&&m<b)h="warning";else if(u>0||b>0)h="success";else h="primary";n.arcColor.set($(h))});return()=>r()}});var gd=z("sui-gauge",{props:{value:y.number({default:0,reflect:!0}),min:y.number({default:0}),max:y.number({default:100}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"",reflect:!0}),"low-threshold":y.number({default:0}),"high-threshold":y.number({default:0}),size:y.number({default:140}),thickness:y.number({default:12}),label:y.string({default:"gauge"})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.min=e.min,t.max=e.max,t.tone=e.tone,t.lowThreshold=e.lowThreshold,t.highThreshold=e.highThreshold,t.size=e.size,t.thickness=e.thickness,t.label=e.label;let i=null;if(an&&typeof an.setup==="function"){if(n.scope=t,i=an.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(u)=>B({anchor:u,test:()=>{if(t.valueD())return 0;return-1},branches:[()=>{let b=document.createDocumentFragment(),h=[],v=document.createElement("path");v.setAttribute("part","fill"),h.push(k(v,"d",()=>t.valueD())),v.setAttribute("fill","none"),h.push(k(v,"stroke",()=>t.arcColor())),h.push(k(v,"stroke-width",()=>t.thickness())),v.setAttribute("stroke-linecap","round");let g=document.createTextNode(`
    `);return v.appendChild(g),b.appendChild(v),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let x of h)try{x()}catch(A){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("svg");o.push(k(m,"viewbox",()=>"0 0 "+t.size()+" "+t.size())),m.setAttribute("role","img"),o.push(k(m,"aria-label",()=>t.label())),o.push(k(m,"style",()=>"width:"+t.size()+"px;height:"+t.size()+"px"));let p=document.createElement("path");p.setAttribute("class","track"),p.setAttribute("part","track"),o.push(k(p,"d",()=>t.trackD())),p.setAttribute("fill","none"),o.push(k(p,"stroke-width",()=>t.thickness())),p.setAttribute("stroke-linecap","round");let l=document.createTextNode(`
  `);p.appendChild(l),m.appendChild(p);let d=document.createComment("if");m.appendChild(d),o.push(f(d)),c.appendChild(m);let a=document.createElement("div");a.setAttribute("class","gauge-label");let s=document.createElement("slot");if(a.appendChild(s),c.appendChild(a),i!=null){let u=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(u){let b=new Proxy(t,{get(h,v){return h[v]},set(h,v,g){let x=h[v];if(x!=null&&typeof x.set==="function"){if(typeof g==="function")x.set(()=>g);else x.set(g);return!0}return!1}});if(u.mount){let h=u.mount(b,r);if(typeof h==="function")o.push(h)}if(u.unmount)o.push(()=>u.unmount(b))}}return t}});var sn=P({scope:()=>({bars:M([])}),mount(n,e){let r=D(()=>{let o=it(n.segments(),[]),t=o.reduce((m,p)=>m+Math.max(0,p.count),0),i=n.total(),f=i>0?i:t,c=f<=0?1:f;n.bars.set(o.filter((m)=>m.count>0).map((m,p)=>{let l=Math.max(0,m.count)/c*100;return{id:`b${p}`,style:`width:${l.toFixed(3)}%;background:${$(m.tone)};`,title:`${m.label??m.tone}: ${m.count}`}}))});return()=>r()}});var Cd=z("sui-meter",{props:{segments:y.string({default:"",reflect:!0}),total:y.number({default:0}),label:y.string({default:"meter"})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.segments=e.segments,t.total=e.total,t.label=e.label;let i=null;if(sn&&typeof sn.setup==="function"){if(n.scope=t,i=sn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(s)=>R({anchor:s,items:()=>t.bars(),key:(u,b)=>{let h=Object.create(t);return h.b=u,h.__i=b,((v)=>u.id)(h)},render:(u,b)=>{let h=Object.create(t);return h.b=u,h.__i=b,h.__v=M(0),((v)=>{let g=document.createDocumentFragment(),x=[],A=document.createElement("div");return A.setAttribute("class","seg"),A.setAttribute("part","segment"),x.push(k(A,"style",()=>(v.__v(),v.b.style))),x.push(k(A,"title",()=>(v.__v(),v.b.title))),g.appendChild(A),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let w of x)try{w()}catch(E){}},update:(w,E)=>{v.b=w,v.__i=E,v.__v.set((j)=>j+1)}}})(h)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("div");m.setAttribute("class","wrap");let p=document.createElement("div");p.setAttribute("class","bar"),p.setAttribute("part","bar"),p.setAttribute("role","img"),o.push(k(p,"aria-label",()=>t.label()));let l=document.createComment("for");p.appendChild(l),o.push(f(l)),m.appendChild(p);let d=document.createElement("div");d.setAttribute("class","legend");let a=document.createElement("slot");if(d.appendChild(a),m.appendChild(d),c.appendChild(m),i!=null){let s=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,v){let g=b[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var un=P({scope:()=>({cells:M([]),svgW:M(0),svgH:M(0)}),mount(n,e){let r=D(()=>{let o=Nt(n.rows()),t=Nt(n.cols()),i=n.cellSize(),f=n.gap(),c=n.data(),m=it(c,[]),p=Array.isArray(m)&&m.length>0&&Array.isArray(m[0]),l;if(p)l=m;else{let A=kt(c),w=t.length||1,E=o.length||(A.length>0?Math.ceil(A.length/w):0);l=Array.from({length:E},(j,C)=>A.slice(C*w,(C+1)*w))}let d=l.length,a=d>0?Math.max(...l.map((A)=>A.length)):0,s=l.flatMap((A)=>A.filter((w)=>typeof w==="number")),u=s.length?Math.min(...s):0,h=(s.length?Math.max(...s):1)-u,v=$(n.tone()),g=i+f;n.svgW.set(a>0?a*i+(a-1)*f:0),n.svgH.set(d>0?d*i+(d-1)*f:0);let x=[];for(let A=0;A<d;A++){let w=l[A]??[];for(let E=0;E<a;E++){let j=w[E]??0,C=h===0?u>0?1:0:(j-u)/h,N=(C*100).toFixed(1),O=C>0?`fill:color-mix(in oklch,${v} ${N}%,transparent)`:"";x.push({id:`c${A}-${E}`,x:E*g,y:A*g,w:i,h:i,fill:O})}}n.cells.set(x)});return()=>r()}});var Pd=z("sui-heatmap",{props:{data:y.string({default:"",reflect:!0}),rows:y.string({default:""}),cols:y.string({default:""}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),"cell-size":y.number({default:20}),gap:y.number({default:3}),label:y.string({default:"heatmap"})},shadow:!0,styles:`
  :host {
    display: inline-block;
    font-family: var(--font-body);
  }
  svg { display: block; overflow: visible; }
  .cell { fill: var(--muted); }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.data=e.data,t.rows=e.rows,t.cols=e.cols,t.tone=e.tone,t.cellSize=e.cellSize,t.gap=e.gap,t.label=e.label;let i=null;if(un&&typeof un.setup==="function"){if(n.scope=t,i=un.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(l)=>R({anchor:l,items:()=>t.cells(),key:(d,a)=>{let s=Object.create(t);return s.cell=d,s.__i=a,((u)=>d.id)(s)},render:(d,a)=>{let s=Object.create(t);return s.cell=d,s.__i=a,s.__v=M(0),((u)=>{let b=document.createDocumentFragment(),h=[],v=document.createElement("rect");v.setAttribute("class","cell"),v.setAttribute("part","cell"),h.push(k(v,"x",()=>(u.__v(),u.cell.x))),h.push(k(v,"y",()=>(u.__v(),u.cell.y))),h.push(k(v,"width",()=>(u.__v(),u.cell.w))),h.push(k(v,"height",()=>(u.__v(),u.cell.h))),v.setAttribute("rx","2"),h.push(k(v,"style",()=>(u.__v(),u.cell.fill)));let g=document.createTextNode(`
    `);return v.appendChild(g),b.appendChild(v),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let x of h)try{x()}catch(A){}},update:(x,A)=>{u.cell=x,u.__i=A,u.__v.set((w)=>w+1)}}})(s)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("svg");o.push(k(m,"viewbox",()=>"0 0 "+t.svgW()+" "+t.svgH())),m.setAttribute("role","img"),o.push(k(m,"aria-label",()=>t.label()));let p=document.createComment("for");if(m.appendChild(p),o.push(f(p)),c.appendChild(m),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var Sd=z("sui-button",{props:{tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["solid","soft","outline","ghost","default"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),type:y.union(["button","submit","reset"],{default:"button",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),loading:y.bool({default:!1,reflect:!0}),label:y.string({default:""}),pressed:y.string({default:"",reflect:!0}),expanded:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
  `,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.tone=e.tone,t.variant=e.variant,t.size=e.size,t.type=e.type,t.disabled=e.disabled,t.loading=e.loading,t.label=e.label,t.pressed=e.pressed,t.expanded=e.expanded;let i=(s)=>B({anchor:s,test:()=>{if(t.loading())return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("span");h.setAttribute("class","spinner-wrap"),h.setAttribute("aria-hidden","true");let v=document.createElement("span");return v.setAttribute("class","spinner"),h.appendChild(v),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let g of b)try{g()}catch(x){}}}}]}),f=r.shadowRoot??r;L(f);let c=document.createElement("button");c.setAttribute("part","button"),o.push(k(c,"type",()=>t.type())),o.push(k(c,"disabled",()=>t.disabled()||t.loading())),o.push(k(c,"aria-busy",()=>t.loading()?"true":"false")),o.push(k(c,"aria-disabled",()=>t.disabled()||t.loading()?"true":"false")),o.push(k(c,"aria-label",()=>t.label())),o.push(k(c,"aria-pressed",()=>t.pressed())),o.push(k(c,"aria-expanded",()=>t.expanded()));let m=document.createElement("span");m.setAttribute("class","content");let p=document.createElement("slot");p.setAttribute("name","leading"),m.appendChild(p);let l=document.createElement("slot");o.push(nt(l,r,"data-icon-only")),m.appendChild(l);let d=document.createElement("slot");d.setAttribute("name","trailing"),m.appendChild(d),c.appendChild(m);let a=document.createComment("if");return c.appendChild(a),o.push(i(a)),f.appendChild(c),t}});var ln=P({mount(n,e){let r=n,o=M(0),t=()=>{let a=r.tabSlot();if(!a)return Array.from(e.querySelectorAll(":scope > sui-tab"));return a.assignedElements().filter((s)=>s.tagName.toLowerCase()==="sui-tab")},i=(a)=>{if(r.active()===a)return;r.active.set(a),e.dispatchEvent(new CustomEvent("sui-tab-change",{detail:{name:a},bubbles:!0,composed:!0}))},f=D(()=>{o();let a=r.active(),s=t(),u=a;if(!u&&s.length>0){if(u=s[0].getAttribute("name")??"",u)r.active.set(u)}for(let b of s){let h=b.getAttribute("name")??"";if(h===u)b.setAttribute("active","");else b.removeAttribute("active");b.id=`sui-tab-${h}`,b.setAttribute("aria-controls",`sui-tabpanel-${h}`)}for(let b of Array.from(e.querySelectorAll(':scope > sui-tab-panel, :scope > [role="tabpanel"]')))if((b.getAttribute("name")??"")===u)b.removeAttribute("hidden"),b.setAttribute("active","");else b.setAttribute("hidden",""),b.removeAttribute("active")}),c=(a)=>{let s=r.orientation()==="vertical",u=s?"ArrowDown":"ArrowRight",b=s?"ArrowUp":"ArrowLeft",h=[u,b,"Home","End"],v=["Enter"," "];if(!h.includes(a.key)&&!v.includes(a.key))return;a.preventDefault();let g=t().filter((C)=>!C.hasAttribute("disabled"));if(!g.length)return;let x=g.map((C)=>C.getAttribute("name")??""),A=r.active(),w=Math.max(0,x.indexOf(A)),E=w;if(a.key===u)E=(w+1)%x.length;else if(a.key===b)E=(w-1+x.length)%x.length;else if(a.key==="Home")E=0;else if(a.key==="End")E=x.length-1;let j=x[E];i(j),queueMicrotask(()=>{let C=g[E],N=C.shadowRoot?.querySelector("button");N?N.focus():C.focus()})},m=(a)=>{let s=a.target.closest("sui-tab");if(!s||s.hasAttribute("disabled"))return;let u=s.getAttribute("name")??"";if(u)i(u)},p=()=>o.set(o()+1),l=r.strip(),d=r.tabSlot();if(l)l.addEventListener("keydown",c),l.addEventListener("click",m);if(d)d.addEventListener("slotchange",p);return()=>{if(f(),l)l.removeEventListener("keydown",c),l.removeEventListener("click",m);if(d)d.removeEventListener("slotchange",p)}}});var Xd=z("sui-tabs",{props:{active:y.string({default:"",reflect:!0}),variant:y.union(["underline","pill","enclosed"],{default:"underline",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),sticky:y.bool({default:!1,reflect:!0}),orientation:y.union(["horizontal","vertical"],{default:"horizontal",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.active=e.active,t.variant=e.variant,t.size=e.size,t.tone=e.tone,t.sticky=e.sticky,t.orientation=e.orientation;let i=null;if(ln&&typeof ln.setup==="function"){if(n.scope=t,i=ln.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","strip"),c.setAttribute("part","strip"),c.setAttribute("role","tablist"),o.push(k(c,"aria-orientation",()=>t.orientation())),T(t,"strip",c);let m=document.createElement("slot");if(T(t,"tabSlot",m),c.appendChild(m),f.appendChild(c),i!=null){let p=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(d,a){return d[a]},set(d,a,s){let u=d[a];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(p.mount){let d=p.mount(l,r);if(typeof d==="function")o.push(d)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var dn=P({mount(n,e){let r=n,o=D(()=>{let t=r.btn();if(!t)return;t.tabIndex=r.active()?0:-1});return()=>{o()}}});var Zd=z("sui-tab",{props:{name:y.string({default:"",reflect:!0}),label:y.string({default:"",reflect:!0}),count:y.string({default:"",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),active:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.name=e.name,t.label=e.label,t.count=e.count,t.disabled=e.disabled,t.active=e.active;let i=null;if(dn&&typeof dn.setup==="function"){if(n.scope=t,i=dn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(s)=>B({anchor:s,test:()=>{if(t.count()!=="")return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("span");h.setAttribute("class","count"),h.setAttribute("aria-hidden","true");let v=document.createTextNode("");return h.appendChild(v),b.push(F(v,()=>t.count())),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let g of b)try{g()}catch(x){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("button");m.setAttribute("part","button"),m.setAttribute("type","button"),m.setAttribute("role","tab"),o.push(k(m,"data-name",()=>t.name())),o.push(k(m,"aria-selected",()=>t.active()?"true":"false")),o.push(k(m,"aria-disabled",()=>t.disabled()?"true":"false")),o.push(k(m,"disabled",()=>t.disabled())),T(t,"btn",m);let p=document.createElement("span");p.setAttribute("class","leading"),p.setAttribute("part","leading");let l=document.createElement("slot");l.setAttribute("name","leading"),p.appendChild(l),m.appendChild(p);let d=document.createElement("slot");m.appendChild(d);let a=document.createComment("if");if(m.appendChild(a),o.push(f(a)),c.appendChild(m),i!=null){let s=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,v){let g=b[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var Kd=z("sui-empty-state",{props:{variant:y.union(["default","inline","placeholder","connecting"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.variant=e.variant,t.size=e.size,t.tone=e.tone;let i=r.shadowRoot??r;L(i);let f=document.createElement("div");f.setAttribute("class","box"),f.setAttribute("part","box"),o.push(k(f,"role",()=>t.variant()==="connecting"?"status":"note")),o.push(k(f,"aria-live",()=>t.variant()==="connecting"?"polite":"off"));let c=document.createElement("div");c.setAttribute("class","icon"),c.setAttribute("part","icon"),c.setAttribute("aria-hidden","true");let m=document.createElement("slot");m.setAttribute("name","icon"),o.push(nt(m,r,"data-no-icon")),c.appendChild(m),f.appendChild(c);let p=document.createElement("div");p.setAttribute("class","copy");let l=document.createElement("div");l.setAttribute("class","title"),l.setAttribute("part","title");let d=document.createElement("slot");d.setAttribute("name","title"),l.appendChild(d),p.appendChild(l);let a=document.createElement("div");a.setAttribute("class","desc"),a.setAttribute("part","desc");let s=document.createElement("slot");a.appendChild(s),p.appendChild(a),f.appendChild(p);let u=document.createElement("div");u.setAttribute("class","actions"),u.setAttribute("part","actions");let b=document.createElement("slot");return b.setAttribute("name","action"),u.appendChild(b),f.appendChild(u),i.appendChild(f),t}});var oi=0,cn=P({mount(n,e){let r=n,o=r.inputEl();if(!o)return;e.expose({focus:()=>o.focus(),blur:()=>o.blur(),clear:()=>{o.value="",r.value.set("")},reportValidity:()=>o.reportValidity()}),Q(o,"input",()=>e.emit("sui-input",{value:o.value})),Q(o,"change",()=>e.emit("sui-input-change",{value:o.value}));let t=`sui-input-err-${++oi}`;ut(k(o,"aria-describedby",()=>{let i=[];if(r.invalid()&&r.errorMessage()){let c=r.errorEl,m=typeof c==="function"?c():null;if(m)m.id=t,i.push(t)}let f=r.describedBy();if(f)i.push(f);return i.length>0?i.join(" "):null}))}});var ec=z("sui-input",{props:{type:y.union(["text","search","email","url","tel","password","number"],{default:"text",reflect:!0}),value:y.string({default:""}),placeholder:y.string({default:""}),name:y.string({default:""}),autocomplete:y.string({default:""}),required:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),readonly:y.bool({default:!1,reflect:!0}),invalid:y.bool({default:!1,reflect:!0}),loading:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["default","ghost"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),label:y.string({default:""}),"described-by":y.string({default:""}),"error-message":y.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.type=e.type,t.value=e.value,t.placeholder=e.placeholder,t.name=e.name,t.autocomplete=e.autocomplete,t.required=e.required,t.disabled=e.disabled,t.readonly=e.readonly,t.invalid=e.invalid,t.loading=e.loading,t.tone=e.tone,t.variant=e.variant,t.size=e.size,t.label=e.label,t.describedBy=e.describedBy,t.errorMessage=e.errorMessage;let i=null;if(cn&&typeof cn.setup==="function"){if(n.scope=t,i=cn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(u)=>B({anchor:u,test:()=>{if(t.loading())return 0;return 1},branches:[()=>{let b=document.createDocumentFragment(),h=[],v=document.createElement("span");return v.setAttribute("class","spinner"),v.setAttribute("aria-hidden","true"),b.appendChild(v),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let g of h)try{g()}catch(x){}}}},()=>{let b=document.createDocumentFragment(),h=[],v=document.createElement("slot");return v.setAttribute("name","trailing"),h.push(nt(v,r,"data-no-trailing")),b.appendChild(v),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let g of h)try{g()}catch(x){}}}}]}),c=(u)=>B({anchor:u,test:()=>{if(t.invalid()&&t.errorMessage())return 0;return-1},branches:[()=>{let b=document.createDocumentFragment(),h=[],v=document.createElement("span");v.setAttribute("class","error"),v.setAttribute("role","alert"),T(t,"errorEl",v);let g=document.createTextNode("");return v.appendChild(g),h.push(F(g,()=>t.errorMessage())),b.appendChild(v),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let x of h)try{x()}catch(A){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createElement("div");p.setAttribute("class","wrap"),p.setAttribute("part","wrap");let l=document.createElement("slot");l.setAttribute("name","leading"),o.push(nt(l,r,"data-no-leading")),p.appendChild(l);let d=document.createElement("input");d.setAttribute("part","input"),o.push(k(d,"type",()=>t.type())),o.push(k(d,"placeholder",()=>t.placeholder())),o.push(k(d,"name",()=>t.name())),o.push(k(d,"autocomplete",()=>t.autocomplete()||"off")),o.push(k(d,"disabled",()=>t.disabled())),o.push(k(d,"readonly",()=>t.readonly())),o.push(k(d,"required",()=>t.required())),o.push(k(d,"aria-label",()=>t.label())),o.push(k(d,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(k(d,"aria-required",()=>t.required()?"true":"false")),o.push(k(d,"aria-busy",()=>t.loading()?"true":"false")),o.push(mt(d,t.value)),T(t,"inputEl",d),p.appendChild(d);let a=document.createComment("if");p.appendChild(a),o.push(f(a)),m.appendChild(p);let s=document.createComment("if");if(m.appendChild(s),o.push(c(s)),i!=null){let u=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(u){let b=new Proxy(t,{get(h,v){return h[v]},set(h,v,g){let x=h[v];if(x!=null&&typeof x.set==="function"){if(typeof g==="function")x.set(()=>g);else x.set(g);return!0}return!1}});if(u.mount){let h=u.mount(b,r);if(typeof h==="function")o.push(h)}if(u.unmount)o.push(()=>u.unmount(b))}}return t}});var ii={git:"git-branch",refresh:"refresh-cw",stop:"stop-circle",warning:"triangle-alert"};var lt=null,Ht=null;function pn(){return lt}function Ur(){if(lt)return Promise.resolve(lt);if(Ht)return Ht;return Ht=fetch("/_sui/lucide-icons.json").then((n)=>{if(!n.ok)throw Error(`lucide registry fetch ${n.status}`);return n.json()}).then((n)=>{return lt=n,n}).catch((n)=>{throw Ht=null,n}),Ht}function Xr(n){if(!n||!lt)return;if(lt[n])return n;let e=ii[n];if(e&&lt[e])return e;return}function qr(n){return lt?lt[n]:void 0}var Rr="http://www.w3.org/2000/svg",nr=M(pn()?1:0),tr=!1;function ai(){if(tr||pn())return;tr=!0,Ur().then(()=>nr.set(nr()+1)).catch(()=>{tr=!1})}function si(n){let e=n.createElementNS(Rr,"svg");return e.setAttribute("part","svg"),e.setAttribute("class","sui-icon-svg"),e.setAttribute("viewBox","0 0 24 24"),e.setAttribute("fill","none"),e.setAttribute("stroke","currentColor"),e.setAttribute("stroke-width","2"),e.setAttribute("stroke-linecap","round"),e.setAttribute("stroke-linejoin","round"),e}function ui(n,e){while(n.firstChild)n.removeChild(n.firstChild);let r=new DOMParser().parseFromString(`<svg xmlns="${Rr}">${e}</svg>`,"image/svg+xml").documentElement;while(r.firstChild)n.appendChild(n.ownerDocument.adoptNode(r.firstChild))}var fn=P({mount(n,e){let r=n,o=e.shadowRoot,t=null,i=null,f=D(()=>{let m=r.label();if(m)e.setAttribute("role","img"),e.setAttribute("aria-label",m),e.removeAttribute("aria-hidden");else e.setAttribute("aria-hidden","true"),e.removeAttribute("role"),e.removeAttribute("aria-label")}),c=D(()=>{nr();let m=r.name(),p=Xr(m)??null,l=r.label();if(!p){if(t)t.remove(),t=null;if(i=null,m&&!pn())ai();return}if(!t)t=si(e.ownerDocument),o.insertBefore(t,o.firstChild);if(p!==i)ui(t,qr(p)),i=p;if(l)t.removeAttribute("aria-hidden");else t.setAttribute("aria-hidden","true")});return()=>{f(),c()}}});var dc=z("sui-icon",{props:{name:y.string({default:"",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:y.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),label:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.name=e.name,t.size=e.size,t.tone=e.tone,t.label=e.label;let i=null;if(fn&&typeof fn.setup==="function"){if(n.scope=t,i=fn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");if(f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var mn=P({mount(n,e){let r=n,o=()=>Array.from(e.querySelectorAll(":scope > option, :scope > optgroup > option")),t=()=>{let s=r.value(),u=o();if(u.length===0)return s;let b=u.find((v)=>v.value===s)??u.find((v)=>v.hasAttribute("selected"))??u[0];return(b?.textContent??"").trim()||b?.value||s},i=()=>{let s=r.selectEl();if(!s){let b=o(),h=b.find((v)=>v.value===r.value())??b.find((v)=>v.hasAttribute("selected"))??b[0];if(!h||h.value==="")e.setAttribute("data-placeholder","");else e.removeAttribute("data-placeholder");return}let u=s.selectedOptions?.[0]??s.options[s.selectedIndex];if(!u||u.value==="")e.setAttribute("data-placeholder","");else e.removeAttribute("data-placeholder")},f=()=>{let s=r.selectEl();if(!s)return;while(s.firstChild)s.removeChild(s.firstChild);let u=document.createDocumentFragment(),b=e.querySelectorAll(":scope > option, :scope > optgroup");for(let h of Array.from(b))u.appendChild(h.cloneNode(!0));s.appendChild(u),s.value=r.value()},c=()=>{let s=r.valueLabel();if(s)s.textContent=t()},m=D(()=>{let s=r.value(),u=r.selectEl();if(u)u.value=s;c(),i()}),p=(s)=>{let u=s.target.value;r.value.set(u),e.dispatchEvent(new CustomEvent("sui-change",{detail:{value:u},bubbles:!0,composed:!0}))},l=r.selectEl();if(l)l.addEventListener("change",p);let d=r.optionsSlot(),a=()=>{f(),c(),i()};if(d)d.addEventListener("slotchange",a);return f(),c(),i(),()=>{if(l)l.removeEventListener("change",p);if(d)d.removeEventListener("slotchange",a);m()}}});var hc=z("sui-select",{props:{value:y.string({default:""}),name:y.string({default:""}),required:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),invalid:y.bool({default:!1,reflect:!0}),tone:y.union(["default","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),label:y.string({default:""}),"described-by":y.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.name=e.name,t.required=e.required,t.disabled=e.disabled,t.invalid=e.invalid,t.tone=e.tone,t.size=e.size,t.label=e.label,t.describedBy=e.describedBy;let i=null;if(mn&&typeof mn.setup==="function"){if(n.scope=t,i=mn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","field"),c.setAttribute("part","field");let m=document.createElement("span");m.setAttribute("class","value"),m.setAttribute("aria-hidden","true"),T(t,"valueLabel",m),c.appendChild(m);let p=document.createElement("svg");p.setAttribute("class","chevron"),p.setAttribute("part","chevron"),p.setAttribute("viewbox","0 0 16 16"),p.setAttribute("aria-hidden","true"),p.setAttribute("focusable","false"),p.setAttribute("fill","none"),p.setAttribute("stroke","currentColor"),p.setAttribute("stroke-width","1.75"),p.setAttribute("stroke-linecap","round"),p.setAttribute("stroke-linejoin","round");let l=document.createElement("path");l.setAttribute("d","M4 6l4 4 4-4"),p.appendChild(l),c.appendChild(p);let d=document.createElement("select");d.setAttribute("part","select"),o.push(k(d,"name",()=>t.name())),o.push(k(d,"disabled",()=>t.disabled())),o.push(k(d,"required",()=>t.required())),o.push(k(d,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(k(d,"aria-label",()=>t.label())),o.push(k(d,"aria-describedby",()=>t.describedBy())),T(t,"selectEl",d),c.appendChild(d),f.appendChild(c);let a=document.createElement("span");a.setAttribute("hidden","");let s=document.createElement("slot");if(T(t,"optionsSlot",s),a.appendChild(s),f.appendChild(a),i!=null){let u=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(u){let b=new Proxy(t,{get(h,v){return h[v]},set(h,v,g){let x=h[v];if(x!=null&&typeof x.set==="function"){if(typeof g==="function")x.set(()=>g);else x.set(g);return!0}return!1}});if(u.mount){let h=u.mount(b,r);if(typeof h==="function")o.push(h)}if(u.unmount)o.push(()=>u.unmount(b))}}return t}});var yc=z("sui-separator",{props:{orientation:y.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:y.union(["solid","dashed"],{default:"solid",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.orientation=e.orientation,t.variant=e.variant;let i=r.shadowRoot??r;L(i);let f=document.createElement("hr");return f.setAttribute("class","sep"),f.setAttribute("part","sep"),f.setAttribute("role","separator"),o.push(k(f,"aria-orientation",()=>t.orientation())),i.appendChild(f),t}});var bn=P({mount(n,e){let r=n,o=D(()=>{let t=r.labelEl();if(!t)return;t.htmlFor=r.for?.()??""});return()=>{o()}}});var Cc=z("sui-label",{props:{for:y.string({default:""}),size:y.union(["sm","md"],{default:"sm",reflect:!0}),required:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.for=e.for,t.size=e.size,t.required=e.required;let i=null;if(bn&&typeof bn.setup==="function"){if(n.scope=t,i=bn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(d)=>B({anchor:d,test:()=>{if(t.required())return 0;return-1},branches:[()=>{let a=document.createDocumentFragment(),s=[],u=document.createElement("span");u.setAttribute("class","req"),u.setAttribute("aria-hidden","true");let b=document.createTextNode("*");return u.appendChild(b),a.appendChild(u),{nodes:Array.from(a.childNodes),cleanup:()=>{for(let h of s)try{h()}catch(v){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("label");m.setAttribute("part","label"),T(t,"labelEl",m);let p=document.createElement("slot");m.appendChild(p);let l=document.createComment("if");if(m.appendChild(l),o.push(f(l)),c.appendChild(m),i!=null){let d=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(d){let a=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(d.mount){let s=d.mount(a,r);if(typeof s==="function")o.push(s)}if(d.unmount)o.push(()=>d.unmount(a))}}return t}});var hn=P({mount(n,e){let r=n,o=D(()=>{let t=r.closeBtn();if(!t)return;let i=()=>{if(!r.dismissible())return;e.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),e.remove()};return t.addEventListener("click",i),()=>t.removeEventListener("click",i)});return()=>{o()}}});var Tc=z("sui-alert",{props:{tone:y.union(["info","success","warning","destructive","muted"],{default:"info",reflect:!0}),dismissible:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.tone=e.tone,t.dismissible=e.dismissible;let i=null;if(hn&&typeof hn.setup==="function"){if(n.scope=t,i=hn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","box"),c.setAttribute("part","box"),o.push(k(c,"role",()=>t.tone()==="destructive"||t.tone()==="warning"?"alert":"status")),o.push(k(c,"aria-live",()=>t.tone()==="destructive"||t.tone()==="warning"?"assertive":"polite")),c.setAttribute("aria-atomic","true");let m=document.createElement("span"),p=document.createElement("slot");p.setAttribute("name","icon"),m.appendChild(p),c.appendChild(m);let l=document.createElement("div");l.setAttribute("class","content");let d=document.createElement("div");d.setAttribute("class","title");let a=document.createElement("slot");a.setAttribute("name","title"),d.appendChild(a),l.appendChild(d);let s=document.createElement("div");s.setAttribute("class","body");let u=document.createElement("slot");s.appendChild(u),l.appendChild(s);let b=document.createElement("div");b.setAttribute("class","actions");let h=document.createElement("slot");h.setAttribute("name","actions"),b.appendChild(h),l.appendChild(b),c.appendChild(l);let v=document.createElement("button");v.setAttribute("class","close"),v.setAttribute("part","close"),v.setAttribute("aria-label","Dismiss"),T(t,"closeBtn",v);let g=document.createTextNode("×");if(v.appendChild(g),c.appendChild(v),f.appendChild(c),i!=null){let x=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(x){let A=new Proxy(t,{get(w,E){return w[E]},set(w,E,j){let C=w[E];if(C!=null&&typeof C.set==="function"){if(typeof j==="function")C.set(()=>j);else C.set(j);return!0}return!1}});if(x.mount){let w=x.mount(A,r);if(typeof w==="function")o.push(w)}if(x.unmount)o.push(()=>x.unmount(A))}}return t}});var li=0,vn=P({mount(n,e){let r=n,o=`sui-tooltip-${++li}`,t=D(()=>{let m=r.bubbleEl();if(m)m.id=o}),i=()=>{let m=r.defaultSlot();if(!m)return;let p=m.assignedElements({flatten:!0})[0];if(!p)return;let l=p.getAttribute("aria-describedby")??"";if(!l.split(/\s+/).includes(o))p.setAttribute("aria-describedby",l?`${l} ${o}`:o)},f=D(()=>{let m=r.defaultSlot();if(!m)return;return m.addEventListener("slotchange",i),i(),()=>m.removeEventListener("slotchange",i)}),c=(m)=>{if(m.key==="Escape")e.blur()};return e.addEventListener("keydown",c),()=>{t(),f(),e.removeEventListener("keydown",c)}}});var Bc=z("sui-tooltip",{props:{placement:y.union(["top","bottom","left","right"],{default:"top",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.placement=e.placement;let i=null;if(vn&&typeof vn.setup==="function"){if(n.scope=t,i=vn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");T(t,"defaultSlot",c),f.appendChild(c);let m=document.createElement("span");m.setAttribute("class","bubble"),m.setAttribute("role","tooltip"),m.setAttribute("part","bubble"),T(t,"bubbleEl",m);let p=document.createElement("slot");if(p.setAttribute("name","content"),m.appendChild(p),f.appendChild(m),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var di=0,gn=P({mount(n,e){let r=n,o=`sui-collapsible-${++di}`,t=D(()=>{let d=r.contentWrap();if(d)d.id=o}),i=()=>{let d=r.triggerSlot();if(!d)return;let a=r.open();for(let s of d.assignedElements()){let u=s;if(u.setAttribute("aria-expanded",String(a)),u.setAttribute("aria-controls",o),!u.hasAttribute("tabindex")&&!u.matches("button, a, input, select, textarea"))u.setAttribute("tabindex","0")}},f=D(()=>{let d=r.triggerSlot();if(!d)return;return d.addEventListener("slotchange",i),i(),()=>d.removeEventListener("slotchange",i)}),c=D(()=>{r.open(),r.triggerSlot(),i()}),m=()=>{if(r.disabled())return;let d=!r.open();r.open.set(d),e.dispatchEvent(new CustomEvent("sui-toggle",{detail:{open:d},bubbles:!0,composed:!0}))},p=(d)=>{if(d.key==="Enter"||d.key===" ")d.preventDefault(),m()},l=D(()=>{let d=r.triggerWrap();if(!d)return;return d.addEventListener("click",m),d.addEventListener("keydown",p),()=>{d.removeEventListener("click",m),d.removeEventListener("keydown",p)}});return()=>{t(),f(),c(),l()}}});var Rc=z("sui-collapsible",{props:{open:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.open=e.open,t.disabled=e.disabled;let i=null;if(gn&&typeof gn.setup==="function"){if(n.scope=t,i=gn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","trigger-wrap"),T(t,"triggerWrap",c);let m=document.createElement("slot");m.setAttribute("name","trigger"),T(t,"triggerSlot",m),c.appendChild(m),f.appendChild(c);let p=document.createElement("div");p.setAttribute("class","content-wrap"),p.setAttribute("part","content"),T(t,"contentWrap",p);let l=document.createElement("div");l.setAttribute("class","content-inner");let d=document.createElement("slot");if(l.appendChild(d),p.appendChild(l),f.appendChild(p),i!=null){let a=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(a){let s=new Proxy(t,{get(u,b){return u[b]},set(u,b,h){let v=u[b];if(v!=null&&typeof v.set==="function"){if(typeof h==="function")v.set(()=>h);else v.set(h);return!0}return!1}});if(a.mount){let u=a.mount(s,r);if(typeof u==="function")o.push(u)}if(a.unmount)o.push(()=>a.unmount(s))}}return t}});var yn=P({mount(n,e){let r=n,o=D(()=>{let i=r.inputEl();if(!i)return;i.indeterminate=r.indeterminate()}),t=D(()=>{let i=r.inputEl();if(!i)return;let f=()=>{r.indeterminate.set(!1),e.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:i.checked,value:r.value()},bubbles:!0,composed:!0}))};return i.addEventListener("change",f),()=>i.removeEventListener("change",f)});return()=>{o(),t()}}});var Vc=z("sui-checkbox",{props:{checked:y.bool({default:!1,reflect:!0}),indeterminate:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),required:y.bool({default:!1,reflect:!0}),invalid:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),name:y.string({default:""}),value:y.string({default:"on"}),label:y.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.checked=e.checked,t.indeterminate=e.indeterminate,t.disabled=e.disabled,t.required=e.required,t.invalid=e.invalid,t.tone=e.tone,t.size=e.size,t.name=e.name,t.value=e.value,t.label=e.label;let i=null;if(yn&&typeof yn.setup==="function"){if(n.scope=t,i=yn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("label");c.setAttribute("part","label");let m=document.createElement("span");m.setAttribute("class","box"),m.setAttribute("part","box");let p=document.createElement("input");p.setAttribute("part","input"),p.setAttribute("type","checkbox"),o.push(bt(p,t.checked)),o.push(k(p,"disabled",()=>t.disabled())),o.push(k(p,"required",()=>t.required())),o.push(k(p,"name",()=>t.name())),o.push(k(p,"value",()=>t.value())),o.push(k(p,"aria-checked",()=>t.indeterminate()?"mixed":t.checked()?"true":"false")),o.push(k(p,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(k(p,"aria-label",()=>t.label()||t.name()||"checkbox")),T(t,"inputEl",p),m.appendChild(p);let l=document.createElement("span");l.setAttribute("class","glyph check"),l.setAttribute("aria-hidden","true"),m.appendChild(l);let d=document.createElement("span");d.setAttribute("class","glyph mixed"),d.setAttribute("aria-hidden","true"),m.appendChild(d),c.appendChild(m);let a=document.createElement("slot");if(c.appendChild(a),f.appendChild(c),i!=null){let s=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,v){let g=b[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var xn=P({mount(n,e){let r=n,o=D(()=>{let t=r.inputEl();if(!t)return;let i=()=>{e.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:t.checked},bubbles:!0,composed:!0}))};return t.addEventListener("change",i),()=>t.removeEventListener("change",i)});return()=>o()}});var np=z("sui-switch",{props:{checked:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),name:y.string({default:""}),value:y.string({default:"on"}),label:y.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.checked=e.checked,t.disabled=e.disabled,t.tone=e.tone,t.size=e.size,t.name=e.name,t.value=e.value,t.label=e.label;let i=null;if(xn&&typeof xn.setup==="function"){if(n.scope=t,i=xn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("label"),m=document.createElement("span");m.setAttribute("class","track"),m.setAttribute("part","track");let p=document.createElement("input");p.setAttribute("part","input"),p.setAttribute("type","checkbox"),p.setAttribute("role","switch"),o.push(bt(p,t.checked)),o.push(k(p,"disabled",()=>t.disabled())),o.push(k(p,"name",()=>t.name())),o.push(k(p,"value",()=>t.value())),o.push(k(p,"aria-checked",()=>t.checked()?"true":"false")),o.push(k(p,"aria-label",()=>t.label()||t.name()||"toggle")),T(t,"inputEl",p),m.appendChild(p);let l=document.createElement("span");l.setAttribute("class","thumb"),l.setAttribute("part","thumb"),m.appendChild(l),c.appendChild(m);let d=document.createElement("slot");if(c.appendChild(d),f.appendChild(c),i!=null){let a=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(a){let s=new Proxy(t,{get(u,b){return u[b]},set(u,b,h){let v=u[b];if(v!=null&&typeof v.set==="function"){if(typeof h==="function")v.set(()=>h);else v.set(h);return!0}return!1}});if(a.mount){let u=a.mount(s,r);if(typeof u==="function")o.push(u)}if(a.unmount)o.push(()=>a.unmount(s))}}return t}});var wn=P({mount(n,e){let r=n,o=r.value(),t=D(()=>{let i=r.textareaEl();if(!i)return;let f=()=>{e.dispatchEvent(new CustomEvent("sui-input",{detail:{value:i.value},bubbles:!0,composed:!0}))},c=()=>{let m=r.value();if(m!==o)o=m,e.dispatchEvent(new CustomEvent("sui-change",{detail:{value:m},bubbles:!0,composed:!0}))};return i.addEventListener("input",f),i.addEventListener("blur",c),()=>{i.removeEventListener("input",f),i.removeEventListener("blur",c)}});return()=>t()}});var sp=z("sui-textarea",{props:{value:y.string({default:""}),placeholder:y.string({default:""}),name:y.string({default:""}),rows:y.number({default:4}),required:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),readonly:y.bool({default:!1,reflect:!0}),invalid:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:y.union(["default","code"],{default:"default",reflect:!0}),size:y.union(["sm","md"],{default:"md",reflect:!0}),resize:y.union(["none","vertical","horizontal","both"],{default:"vertical"}),label:y.string({default:""}),"described-by":y.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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

`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.placeholder=e.placeholder,t.name=e.name,t.rows=e.rows,t.required=e.required,t.disabled=e.disabled,t.readonly=e.readonly,t.invalid=e.invalid,t.tone=e.tone,t.variant=e.variant,t.size=e.size,t.resize=e.resize,t.label=e.label,t.describedBy=e.describedBy;let i=null;if(wn&&typeof wn.setup==="function"){if(n.scope=t,i=wn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("textarea");if(c.setAttribute("part","textarea"),o.push(k(c,"placeholder",()=>t.placeholder())),o.push(k(c,"name",()=>t.name())),o.push(k(c,"rows",()=>t.rows())),o.push(k(c,"disabled",()=>t.disabled())),o.push(k(c,"required",()=>t.required())),o.push(k(c,"readonly",()=>t.readonly())),o.push(k(c,"aria-invalid",()=>t.invalid()?"true":"false")),o.push(k(c,"aria-required",()=>t.required()?"true":"false")),o.push(k(c,"aria-label",()=>t.label())),o.push(k(c,"aria-describedby",()=>t.describedBy())),o.push(D(()=>{let m=t.resize();if(m==null)c.style.removeProperty("resize");else c.style.setProperty("resize",String(m))})),o.push(mt(c,t.value)),T(t,"textareaEl",c),f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var Jr=0,An=P({mount(n,e){let r=n,o=`sui-field-hint-${++Jr}`,t=`sui-field-error-${++Jr}`,i=D(()=>{let p=r.hintEl(),l=r.errorEl();if(p)p.id=o;if(l)l.id=t}),f=()=>{let l=r.controlSlot()?.assignedElements({flatten:!0})[0];if(!l)return;let d=r.invalid(),a=d?t:o,u=[...(l.getAttribute("aria-describedby")||"").split(/\s+/).filter((b)=>b&&b!==o&&b!==t),a].join(" ").trim();if(u)l.setAttribute("aria-describedby",u);if("invalid"in l)l.invalid=d;else l.setAttribute("aria-invalid",d?"true":"false")},c=D(()=>{r.invalid(),f()}),m=D(()=>{let p=r.controlSlot();if(!p)return;return p.addEventListener("slotchange",f),()=>p.removeEventListener("slotchange",f)});return()=>{i(),c(),m()}}});var fp=z("sui-field",{props:{invalid:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.invalid=e.invalid;let i=null;if(An&&typeof An.setup==="function"){if(n.scope=t,i=An.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");c.setAttribute("name","label"),f.appendChild(c);let m=document.createElement("slot");T(t,"controlSlot",m),f.appendChild(m);let p=document.createElement("div");p.setAttribute("class","hint"),T(t,"hintEl",p);let l=document.createElement("slot");l.setAttribute("name","hint"),p.appendChild(l),f.appendChild(p);let d=document.createElement("div");d.setAttribute("class","error"),d.setAttribute("role","alert"),d.setAttribute("aria-live","polite"),T(t,"errorEl",d);let a=document.createElement("slot");if(a.setAttribute("name","error"),d.appendChild(a),f.appendChild(d),i!=null){let s=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,v){let g=b[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var En=P({scope:(n,e)=>{let r=n,o=J(null),t=Z(()=>r.tone()==="destructive"?"destructive":"default");function i(m){e.dispatchEvent(new CustomEvent(m,{bubbles:!0,composed:!0})),r.open.set(!1)}function f(){i("sui-confirm")}function c(){i("sui-cancel")}return{dialogRef:o,confirmTone:t,onConfirm:f,onCancel:c}},mount(n,e){let r=n,o=r.open,t=r.dialogRef,i=D(()=>{let c=o(),m=t();if(!m)return;if(c&&typeof m.show==="function")m.show();else if(!c&&typeof m.close==="function")m.close()}),f=()=>{if(o())o.set(!1)};return e.addEventListener("sui-close",f),()=>{i(),e.removeEventListener("sui-close",f)}}});var yp=z("sui-alert-dialog",{props:{open:y.bool({default:!1,reflect:!0}),tone:y.union(["default","destructive"],{default:"default",reflect:!0}),"confirm-label":y.string({default:"Confirm"}),"cancel-label":y.string({default:"Cancel"})},shadow:!0,styles:`
  :host { display: contents; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.open=e.open,t.tone=e.tone,t.confirmLabel=e.confirmLabel,t.cancelLabel=e.cancelLabel;let i=null;if(En&&typeof En.setup==="function"){if(n.scope=t,i=En.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("sui-dialog");c.setAttribute("role-override","alertdialog"),c.setAttribute("size","sm"),c.setAttribute("no-close-button",""),T(t,"dialogRef",c);let m=document.createElement("span");m.setAttribute("slot","header");let p=document.createElement("slot");p.setAttribute("name","title"),m.appendChild(p),c.appendChild(m);let l=document.createElement("slot");c.appendChild(l);let d=document.createElement("span");d.setAttribute("slot","footer");let a=document.createElement("sui-button");a.setAttribute("variant","outline"),o.push(K(a,"click",function(){let h=t.onCancel;return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let s=document.createTextNode("");a.appendChild(s),o.push(F(s,()=>t.cancelLabel())),d.appendChild(a);let u=document.createElement("sui-button");u.setAttribute("variant","solid"),o.push(k(u,"tone",()=>t.confirmTone())),o.push(K(u,"click",function(){let h=t.onConfirm;return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let b=document.createTextNode("");if(u.appendChild(b),o.push(F(b,()=>t.confirmLabel())),d.appendChild(u),c.appendChild(d),f.appendChild(c),i!=null){let h=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(g,x){return g[x]},set(g,x,A){let w=g[x];if(w!=null&&typeof w.set==="function"){if(typeof A==="function")w.set(()=>A);else w.set(A);return!0}return!1}});if(h.mount){let g=h.mount(v,r);if(typeof g==="function")o.push(g)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});function Wr(n){if(n<1024)return`${n} B`;if(n<1048576)return`${(n/1024).toFixed(1)} KB`;return`${(n/1048576).toFixed(1)} MB`}var kn=P({scope:()=>{let n=M([]),e=M(!1),r=M(""),o=J(null),t=J(null),i=J(null),f=Z(()=>n().map((c,m)=>({file:c,idx:m,key:`${m}:${c.name}:${c.size}`,name:c.name,sizeStr:Wr(c.size)})));return{files:n,over:e,errorMsg:r,displayFiles:f,dropzoneEl:o,inputEl:t,fileListEl:i}},mount(n,e){let r=(h)=>{let v=n.accept().split(",").map((A)=>A.trim().toLowerCase()).filter(Boolean);if(v.length===0)return!0;let g="."+(h.name.split(".").pop()||"").toLowerCase(),x=h.type.toLowerCase();return v.some((A)=>{if(A.startsWith("."))return g===A;if(A.endsWith("/*"))return x.startsWith(A.slice(0,-1));return x===A})},o=(h)=>{e.dispatchEvent(new CustomEvent("sui-change",{detail:{files:[...h]},bubbles:!0,composed:!0}))},t=(h)=>{n.errorMsg.set("");let v=[];for(let A of h){if(A.size>n.maxSize()){n.errorMsg.set(`"${A.name}" exceeds max size (${Wr(n.maxSize())}).`);continue}if(n.accept()&&!r(A)){n.errorMsg.set(`"${A.name}" is not an accepted file type.`);continue}v.push(A)}let g=n.files(),x=n.multiple()?[...g,...v]:v.slice(0,1);n.files.set(x),o(x)},i=(h)=>{let v=n.files();if(h<0||h>=v.length)return;let g=v.filter((x,A)=>A!==h);n.files.set(g),n.errorMsg.set(""),o(g)},f=()=>{if(n.disabled())return;n.inputEl()?.click()},c=()=>f(),m=(h)=>{if(n.disabled())return;if(h.key==="Enter"||h.key===" ")h.preventDefault(),f()},p=(h)=>{if(n.disabled())return;h.preventDefault(),n.over.set(!0)},l=()=>n.over.set(!1),d=(h)=>{if(n.disabled())return;h.preventDefault(),n.over.set(!1);let v=h.dataTransfer?.files;if(v&&v.length)t(Array.from(v))},a=(h)=>{let v=h.target;if(v.files&&v.files.length)t(Array.from(v.files));v.value=""},s=n.dropzoneEl(),u=n.inputEl();s?.addEventListener("click",c),s?.addEventListener("keydown",m),s?.addEventListener("dragover",p),s?.addEventListener("dragleave",l),s?.addEventListener("drop",d),u?.addEventListener("change",a);let b=(h)=>{let g=h.target?.closest?.("button.remove");if(!g||!e.shadowRoot?.contains(g))return;let x=Number(g.dataset.idx);if(Number.isInteger(x))i(x)};return e.shadowRoot?.addEventListener("click",b),()=>{s?.removeEventListener("click",c),s?.removeEventListener("keydown",m),s?.removeEventListener("dragover",p),s?.removeEventListener("dragleave",l),s?.removeEventListener("drop",d),u?.removeEventListener("change",a),e.shadowRoot?.removeEventListener("click",b)}}});var Cp=z("sui-file-upload",{props:{accept:y.string({default:""}),multiple:y.bool({default:!1}),"max-size":y.number({default:10485760}),disabled:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.accept=e.accept,t.multiple=e.multiple,t.maxSize=e.maxSize,t.disabled=e.disabled,t.tone=e.tone;let i=null;if(kn&&typeof kn.setup==="function"){if(n.scope=t,i=kn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(g)=>B({anchor:g,test:()=>{if(t.errorMsg())return 0;return-1},branches:[()=>{let x=document.createDocumentFragment(),A=[],w=document.createElement("div");w.setAttribute("class","error"),w.setAttribute("role","alert"),w.setAttribute("part","error");let E=document.createTextNode("");return w.appendChild(E),A.push(F(E,()=>t.errorMsg())),x.appendChild(w),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let j of A)try{j()}catch(C){}}}}]}),c=(g)=>R({anchor:g,items:()=>t.displayFiles(),key:(x,A)=>{let w=Object.create(t);return w.f=x,w.__i=A,((E)=>x.key)(w)},render:(x,A)=>{let w=Object.create(t);return w.f=x,w.__i=A,w.__v=M(0),((E)=>{let j=document.createDocumentFragment(),C=[],N=document.createElement("div");N.setAttribute("class","file-item"),N.setAttribute("part","file-item");let O=document.createElement("span");O.setAttribute("class","file-name");let S=document.createTextNode("");O.appendChild(S),C.push(F(S,()=>(E.__v(),E.f.name))),N.appendChild(O);let H=document.createElement("span");H.setAttribute("class","file-size");let U=document.createTextNode("");H.appendChild(U),C.push(F(U,()=>(E.__v(),E.f.sizeStr))),N.appendChild(H);let G=document.createElement("button");G.setAttribute("class","remove"),G.setAttribute("part","remove"),G.setAttribute("type","button"),C.push(k(G,"data-idx",()=>(E.__v(),E.f.idx))),C.push(k(G,"aria-label",()=>(E.__v(),"Remove "+E.f.name)));let X=document.createTextNode("×");return G.appendChild(X),N.appendChild(G),j.appendChild(N),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let Y of C)try{Y()}catch(W){}},update:(Y,W)=>{E.f=Y,E.__i=W,E.__v.set((q)=>q+1)}}})(w)}}),m=(g)=>B({anchor:g,test:()=>{if(t.displayFiles().length>0)return 0;return-1},branches:[()=>{let x=document.createDocumentFragment(),A=[],w=document.createElement("div");w.setAttribute("class","file-list"),w.setAttribute("part","file-list"),w.setAttribute("aria-live","polite"),T(t,"fileListEl",w);let E=document.createComment("for");return w.appendChild(E),A.push(c(E)),x.appendChild(w),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let j of A)try{j()}catch(C){}}}}]}),p=r.shadowRoot??r;L(p);let l=document.createElement("div");l.setAttribute("class","dropzone"),o.push(pt(l,"over",()=>t.over())),l.setAttribute("part","dropzone"),l.setAttribute("role","button"),o.push(k(l,"tabindex",()=>t.disabled()?"-1":"0")),o.push(k(l,"aria-label",()=>t.accept()?"Upload files ("+t.accept()+")":"Upload files")),o.push(k(l,"aria-disabled",()=>t.disabled()?"true":"false")),T(t,"dropzoneEl",l);let d=document.createElement("span");d.setAttribute("class","icon"),d.setAttribute("aria-hidden","true");let a=document.createTextNode("\uD83D\uDCC4");d.appendChild(a),l.appendChild(d);let s=document.createElement("span"),u=document.createTextNode("Drop files here or click to browse");s.appendChild(u),l.appendChild(s),p.appendChild(l);let b=document.createElement("input");b.setAttribute("type","file"),b.setAttribute("part","input"),o.push(k(b,"accept",()=>t.accept())),o.push(k(b,"multiple",()=>t.multiple())),b.setAttribute("aria-hidden","true"),b.setAttribute("tabindex","-1"),T(t,"inputEl",b),p.appendChild(b);let h=document.createComment("if");p.appendChild(h),o.push(f(h));let v=document.createComment("if");if(p.appendChild(v),o.push(m(v)),i!=null){let g=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(g){let x=new Proxy(t,{get(A,w){return A[w]},set(A,w,E){let j=A[w];if(j!=null&&typeof j.set==="function"){if(typeof E==="function")j.set(()=>E);else j.set(E);return!0}return!1}});if(g.mount){let A=g.mount(x,r);if(typeof A==="function")o.push(A)}if(g.unmount)o.push(()=>g.unmount(x))}}return t}});var ci=new Set(["info","success","warning","destructive","accent"]),Cn=P({scope:()=>{let n=M(!1),e=M(!0),r=J(null),o=J(null),t=J(null),i=J(null);return{hasSlottedIcon:n,visible:e,iconSlotEl:r,titleSlotEl:o,actionsSlotEl:t,closeBtn:i}},mount(n,e){let r=n,o=null,t=0,i=0,f=!1;if(!e.hasAttribute("tabindex"))e.tabIndex=0;function c(){if(o)clearTimeout(o),o=null}function m(){c();let E=r.duration();if(E>0)t=E,i=Date.now(),o=setTimeout(()=>p("timeout"),E),e.style.setProperty("--toast-duration",`${E}ms`)}function p(E="manual"){if(f)return;f=!0,c(),e.dispatchEvent(new CustomEvent("sui-dismiss",{detail:{reason:E},bubbles:!0,composed:!0})),e.remove()}async function l(E="manual"){if(f)return;if(typeof window<"u"&&window.matchMedia?.("(prefers-reduced-motion: reduce)").matches){p(E);return}c(),e.setAttribute("closing",""),r.visible.set(!1),await new Promise((N)=>setTimeout(N,200)),p(E)}let d=()=>{if(!o)return;let E=Date.now()-i;t=Math.max(0,t-E),c(),e.toggleAttribute("data-paused",!0)},a=()=>{if(f||r.duration()<=0)return;if(t<=0)return;e.toggleAttribute("data-paused",!1),i=Date.now(),o=setTimeout(()=>p("timeout"),t)},s=(E)=>{if(E.key==="Escape"&&r.dismissible())E.stopPropagation(),p("manual")};e.addEventListener("mouseenter",d),e.addEventListener("mouseleave",a),e.addEventListener("focusin",d),e.addEventListener("focusout",a),e.addEventListener("keydown",s),m();let u=r.hasSlottedIcon,b=(E)=>E.assignedNodes({flatten:!0}).some((j)=>{if(j.nodeType===Node.ELEMENT_NODE)return!0;return(j.textContent?.trim().length??0)>0}),h=r.iconSlotEl();if(h){let E=()=>u.set(b(h));h.addEventListener("slotchange",E),u.set(b(h))}let v=r.titleSlotEl(),g=r.actionsSlotEl();if(v){let E=()=>e.toggleAttribute("data-no-title",!b(v));v.addEventListener("slotchange",E),e.toggleAttribute("data-no-title",!b(v))}if(g){let E=()=>e.toggleAttribute("data-no-actions",!b(g));g.addEventListener("slotchange",E),e.toggleAttribute("data-no-actions",!b(g))}let x=D(()=>{let E=r.tone(),j=r.hasSlottedIcon(),C=ci.has(E);e.toggleAttribute("data-no-icon",!C&&!j)}),A=D(()=>{let E=r.closeBtn();if(!E)return;let j=()=>p("manual");return E.addEventListener("click",j),()=>E.removeEventListener("click",j)}),w=e;return w.dismiss=p,w.close=l,()=>{f=!0,c(),e.removeEventListener("mouseenter",d),e.removeEventListener("mouseleave",a),e.removeEventListener("focusin",d),e.removeEventListener("focusout",a),e.removeEventListener("keydown",s),x(),A()}}});var Tp=z("sui-toast",{props:{tone:y.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),duration:y.number({default:5000}),dismissible:y.bool({default:!0,reflect:!0}),"show-progress":y.bool({default:!1,reflect:!0}),closing:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.tone=e.tone,t.size=e.size,t.duration=e.duration,t.dismissible=e.dismissible,t.showProgress=e.showProgress,t.closing=e.closing;let i=null;if(Cn&&typeof Cn.setup==="function"){if(n.scope=t,i=Cn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(s)=>B({anchor:s,test:()=>{let u=t.tone();if(u==="info")return 0;if(u==="success")return 1;if(u==="warning")return 2;if(u==="destructive")return 3;if(u==="accent")return 4;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("circle");v.setAttribute("cx","12"),v.setAttribute("cy","12"),v.setAttribute("r","9"),h.appendChild(v);let g=document.createElement("path");g.setAttribute("d","M12 8h.01"),h.appendChild(g);let x=document.createElement("path");return x.setAttribute("d","M11 12h1v4h1"),h.appendChild(x),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let A of b)try{A()}catch(w){}}}},()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("circle");v.setAttribute("cx","12"),v.setAttribute("cy","12"),v.setAttribute("r","9"),h.appendChild(v);let g=document.createElement("path");return g.setAttribute("d","m8.5 12.5 2.5 2.5 4.5-5"),h.appendChild(g),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let x of b)try{x()}catch(A){}}}},()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("path");v.setAttribute("d","M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"),h.appendChild(v);let g=document.createElement("line");g.setAttribute("x1","12"),g.setAttribute("x2","12"),g.setAttribute("y1","9"),g.setAttribute("y2","13"),h.appendChild(g);let x=document.createElement("line");return x.setAttribute("x1","12"),x.setAttribute("x2","12.01"),x.setAttribute("y1","17"),x.setAttribute("y2","17"),h.appendChild(x),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let A of b)try{A()}catch(w){}}}},()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("circle");v.setAttribute("cx","12"),v.setAttribute("cy","12"),v.setAttribute("r","9"),h.appendChild(v);let g=document.createElement("line");g.setAttribute("x1","15"),g.setAttribute("x2","9"),g.setAttribute("y1","9"),g.setAttribute("y2","15"),h.appendChild(g);let x=document.createElement("line");return x.setAttribute("x1","9"),x.setAttribute("x2","15"),x.setAttribute("y1","9"),x.setAttribute("y2","15"),h.appendChild(x),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let A of b)try{A()}catch(w){}}}},()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("svg");h.setAttribute("viewbox","0 0 24 24"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","2"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true");let v=document.createElement("path");return v.setAttribute("d","M12 2 3 14h7l-1 8 10-12h-7l1-8Z"),h.appendChild(v),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let g of b)try{g()}catch(x){}}}}]}),c=(s)=>B({anchor:s,test:()=>{if(!t.hasSlottedIcon())return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createComment("match");return u.appendChild(h),b.push(f(h)),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let v of b)try{v()}catch(g){}}}}]}),m=(s)=>B({anchor:s,test:()=>{if(t.dismissible())return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("button");h.setAttribute("class","close"),h.setAttribute("part","close"),h.setAttribute("type","button"),h.setAttribute("aria-label","Dismiss notification"),T(t,"closeBtn",h);let v=document.createElement("svg");v.setAttribute("viewbox","0 0 24 24"),v.setAttribute("fill","none"),v.setAttribute("stroke","currentColor"),v.setAttribute("stroke-width","2"),v.setAttribute("stroke-linecap","round"),v.setAttribute("stroke-linejoin","round"),v.setAttribute("aria-hidden","true");let g=document.createElement("line");g.setAttribute("x1","18"),g.setAttribute("y1","6"),g.setAttribute("x2","6"),g.setAttribute("y2","18"),v.appendChild(g);let x=document.createElement("line");return x.setAttribute("x1","6"),x.setAttribute("y1","6"),x.setAttribute("x2","18"),x.setAttribute("y2","18"),v.appendChild(x),h.appendChild(v),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let A of b)try{A()}catch(w){}}}}]}),p=(s)=>B({anchor:s,test:()=>{if(t.showProgress()&&t.duration()>0)return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("span");return h.setAttribute("class","progress"),h.setAttribute("aria-hidden","true"),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let v of b)try{v()}catch(g){}}}}]}),l=(s)=>B({anchor:s,test:()=>{if(t.visible())return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("div");h.setAttribute("class","toast"),ot(h,Je,{axis:"y",duration:200},"in"),ot(h,ht,{duration:180},"out"),b.push(k(h,"role",()=>t.tone()==="destructive"||t.tone()==="warning"?"alert":"status")),b.push(k(h,"aria-live",()=>t.tone()==="destructive"||t.tone()==="warning"?"assertive":"polite")),h.setAttribute("aria-atomic","true"),h.setAttribute("part","toast");let v=document.createElement("span");v.setAttribute("class","icon");let g=document.createElement("slot");g.setAttribute("name","icon"),T(t,"iconSlotEl",g),v.appendChild(g);let x=document.createComment("if");v.appendChild(x),b.push(c(x)),h.appendChild(v);let A=document.createElement("div");A.setAttribute("class","content");let w=document.createElement("div");w.setAttribute("class","title");let E=document.createElement("slot");E.setAttribute("name","title"),T(t,"titleSlotEl",E),w.appendChild(E),A.appendChild(w);let j=document.createElement("div");j.setAttribute("class","body");let C=document.createElement("slot");j.appendChild(C),A.appendChild(j);let N=document.createElement("div");N.setAttribute("class","actions");let O=document.createElement("slot");O.setAttribute("name","actions"),T(t,"actionsSlotEl",O),N.appendChild(O),A.appendChild(N),h.appendChild(A);let S=document.createComment("if");h.appendChild(S),b.push(m(S));let H=document.createComment("if");return h.appendChild(H),b.push(p(H)),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let U of b)try{U()}catch(G){}}}}]}),d=r.shadowRoot??r;L(d);let a=document.createComment("if");if(d.appendChild(a),o.push(l(a)),i!=null){let s=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,v){let g=b[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var pi={"top-left":"top","top-center":"top","top-right":"top","bottom-left":"bottom","bottom-center":"bottom","bottom-right":"bottom"},at="16px",jt="position:fixed;display:flex;gap:var(--space-2,8px);pointer-events:none;z-index:var(--z-toast,9999);max-width:min(440px,calc(100vw - 32px));padding:0;margin:0;box-sizing:border-box;";function fi(n){switch(n){case"top-right":return`${jt}top:${at};right:${at};flex-direction:column;align-items:flex-end;`;case"top-left":return`${jt}top:${at};left:${at};flex-direction:column;align-items:flex-start;`;case"top-center":return`${jt}top:${at};left:50%;transform:translateX(-50%);flex-direction:column;align-items:center;`;case"bottom-left":return`${jt}bottom:${at};left:${at};flex-direction:column-reverse;align-items:flex-start;`;case"bottom-center":return`${jt}bottom:${at};left:50%;transform:translateX(-50%);flex-direction:column-reverse;align-items:center;`;case"bottom-right":default:return`${jt}bottom:${at};right:${at};flex-direction:column-reverse;align-items:flex-end;`}}var jn=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","region");if(!e.hasAttribute("aria-label"))e.setAttribute("aria-label","Notifications");if(!e.hasAttribute("aria-live"))e.setAttribute("aria-live","polite");let o=M(0),t=r.defaultSlotEl(),i=r.stackEl(),f=(u)=>{if(!i)return;if(u.parentElement!==i)i.appendChild(u)},c=()=>{if(!i)return[];return Array.from(i.children).filter((u)=>u.tagName.toLowerCase()==="sui-toast")},m=D(()=>{if(!i)return;let u=r.position();i.style.cssText=fi(u),i.setAttribute("data-position",u)}),p=D(()=>{o();let u=r.position(),b=r.maxVisible(),h=pi[u]??"bottom",v=c();for(let g of v)g.setAttribute("data-slide-from",h),g.style.pointerEvents="auto";if(b>0&&v.length>b){let g=v.length-b;for(let x=0;x<g;x++){let A=v[x];if(typeof A.close==="function")A.close("programmatic");else A.dismiss?.("programmatic")}}}),l=()=>{if(t){for(let u of t.assignedElements())if(u.tagName.toLowerCase()==="sui-toast")f(u)}o.set(o()+1)};if(t)t.addEventListener("slotchange",l),l();let d=new MutationObserver((u)=>{let b=!1;for(let h of u){for(let v of h.addedNodes)if(v.nodeType===Node.ELEMENT_NODE){let g=v;if(g.tagName.toLowerCase()==="sui-toast")f(g),b=!0}if(h.removedNodes.length)b=!0}if(b)o.set(o()+1)});d.observe(e,{childList:!0});for(let u of Array.from(e.children))if(u.tagName.toLowerCase()==="sui-toast")f(u);o.set(o()+1);let a=i?new MutationObserver(()=>o.set(o()+1)):null;if(i&&a)a.observe(i,{childList:!0});function s(u={}){let b=e.ownerDocument??document,h=b.createElement("sui-toast");if(u.tone)h.setAttribute("tone",u.tone);if(u.size)h.setAttribute("size",u.size);if(typeof u.duration==="number")h.setAttribute("duration",String(u.duration));if(u.dismissible===!1)h.removeAttribute("dismissible");else h.setAttribute("dismissible","");if(u.showProgress)h.setAttribute("show-progress","");if(u.title){let v=b.createElement("span");v.setAttribute("slot","title"),v.textContent=u.title,h.appendChild(v)}if(u.description)h.appendChild(b.createTextNode(u.description));if(u.actionHtml){let v=b.createElement("span");v.setAttribute("slot","actions");let g=u.actionHtml;et(v,()=>g),h.appendChild(v)}if(i)i.appendChild(h);else e.appendChild(h);return h}return e.show=s,()=>{if(p(),m(),d.disconnect(),a?.disconnect(),t)t.removeEventListener("slotchange",l)}}});var Bp=z("sui-toast-container",{props:{position:y.union(["top-left","top-center","top-right","bottom-left","bottom-center","bottom-right"],{default:"bottom-right",reflect:!0}),"max-visible":y.number({default:5,reflect:!0})},shadow:!0,styles:`
  /* Host collapses — the visible stack lives in body via <sui-portal>
     (see template). The host stays minimal so it can sit anywhere in
     the document without taking layout space. */
  :host { display: contents; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.position=e.position,t.maxVisible=e.maxVisible;let i=null;if(jn&&typeof jn.setup==="function"){if(n.scope=t,i=jn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("sui-portal");c.setAttribute("target","body");let m=document.createElement("div");m.setAttribute("class","sui-toast-stack"),m.setAttribute("data-sui-toast-stack",""),T(t,"stackEl",m),c.appendChild(m),f.appendChild(c);let p=document.createElement("slot");if(T(t,"defaultSlotEl",p),f.appendChild(p),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var zn=P({scope:()=>{let n=M([]),e=M("");return{items:n,lastSlot:e}},mount(n,e){let r=()=>{let i=Array.from(e.querySelectorAll(":scope > sui-breadcrumb-item")).map((f,c)=>{let m=`i${c}`;return f.setAttribute("slot",m),{slot:m}});n.lastSlot.set(i.length>0?i[i.length-1].slot:""),n.items.set(i)};r();let o=new MutationObserver(r);return o.observe(e,{childList:!0}),()=>o.disconnect()}});var Rp=z("sui-breadcrumb",{props:{separator:y.string({default:"/",reflect:!0})},shadow:!0,styles:`
  :host { display: flex; align-items: center; }
  nav { display: inline-flex; align-items: center; flex-wrap: wrap; gap: 2px; }
  .sep {
    display: inline-flex; align-items: center; justify-content: center;
    width: 12px; height: 12px;
    color: var(--muted-foreground);
    opacity: 0.7;
  }
  .sep svg { width: 12px; height: 12px; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.separator=e.separator;let i=null;if(zn&&typeof zn.setup==="function"){if(n.scope=t,i=zn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(l)=>R({anchor:l,items:()=>t.items(),key:(d,a)=>{let s=Object.create(t);return s.item=d,s.__i=a,((u)=>d.slot)(s)},render:(d,a)=>{let s=Object.create(t);return s.item=d,s.__i=a,s.__v=M(0),((u)=>{let b=(w)=>B({anchor:w,test:()=>{if(u.separator()==="/")return 0;return 1},branches:[()=>{let E=document.createDocumentFragment(),j=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round");let N=document.createElement("path");return N.setAttribute("d","M9 6l6 6-6 6"),C.appendChild(N),E.appendChild(C),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let O of j)try{O()}catch(S){}}}},()=>{let E=document.createDocumentFragment(),j=[],C=document.createTextNode("");return E.appendChild(C),j.push(F(C,()=>(u.__v(),u.separator()))),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let N of j)try{N()}catch(O){}}}}]}),h=(w)=>B({anchor:w,test:()=>{if(u.item.slot!==u.lastSlot())return 0;return-1},branches:[()=>{let E=document.createDocumentFragment(),j=[],C=document.createElement("span");C.setAttribute("class","sep"),C.setAttribute("aria-hidden","true");let N=document.createComment("if");return C.appendChild(N),j.push(b(N)),E.appendChild(C),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let O of j)try{O()}catch(S){}}}}]}),v=document.createDocumentFragment(),g=[],x=document.createElement("slot");g.push(k(x,"name",()=>(u.__v(),u.item.slot))),v.appendChild(x);let A=document.createComment("if");return v.appendChild(A),g.push(h(A)),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let w of g)try{w()}catch(E){}},update:(w,E)=>{u.item=w,u.__i=E,u.__v.set((j)=>j+1)}}})(s)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("nav");m.setAttribute("aria-label","Breadcrumb"),m.setAttribute("part","nav");let p=document.createComment("for");if(m.appendChild(p),o.push(f(p)),c.appendChild(m),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var Yp=z("sui-breadcrumb-item",{props:{href:y.string({default:"",reflect:!0}),active:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.href=e.href,t.active=e.active;let i=(m)=>B({anchor:m,test:()=>{if(t.href()&&!t.active())return 0;return 1},branches:[()=>{let p=document.createDocumentFragment(),l=[],d=document.createElement("a");l.push(k(d,"href",()=>t.href())),d.setAttribute("part","link");let a=document.createElement("slot");return d.appendChild(a),p.appendChild(d),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let s of l)try{s()}catch(u){}}}},()=>{let p=document.createDocumentFragment(),l=[],d=document.createElement("span");d.setAttribute("class","label"),d.setAttribute("part","label"),l.push(k(d,"aria-current",()=>t.active()?"page":"false"));let a=document.createElement("slot");return d.appendChild(a),p.appendChild(d),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let s of l)try{s()}catch(u){}}}}]}),f=r.shadowRoot??r;L(f);let c=document.createComment("if");return f.appendChild(c),o.push(i(c)),t}});var Ln=P({scope:()=>({totalPages:M(1),info:M(""),prevBtn:J(null),nextBtn:J(null)}),mount(n,e){let r=n,o=D(()=>{let m=r.page(),p=r.total(),l=r.perPage(),d=p>0?Math.ceil(p/l):1;if(r.totalPages.set(d),p>0){let a=(m-1)*l+1,s=Math.min(m*l,p);r.info.set(`Page ${m} of ${d} (${a}–${s} of ${p})`)}else r.info.set(`Page ${m} of ${d}`)}),t=r.prevBtn(),i=r.nextBtn(),f=()=>{let m=r.page();if(m>1){let p=m-1;r.page.set(p),e.dispatchEvent(new CustomEvent("sui-page",{detail:{page:p},bubbles:!0,composed:!0}))}},c=()=>{let m=r.page(),p=r.totalPages();if(m<p){let l=m+1;r.page.set(l),e.dispatchEvent(new CustomEvent("sui-page",{detail:{page:l},bubbles:!0,composed:!0}))}};return t?.addEventListener("click",f),i?.addEventListener("click",c),()=>{o(),t?.removeEventListener("click",f),i?.removeEventListener("click",c)}}});var $p=z("sui-pagination",{props:{page:y.number({default:1,reflect:!0}),total:y.number({default:0,reflect:!0}),"per-page":y.number({default:25,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.page=e.page,t.total=e.total,t.perPage=e.perPage;let i=null;if(Ln&&typeof Ln.setup==="function"){if(n.scope=t,i=Ln.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("nav");c.setAttribute("class","bar"),c.setAttribute("aria-label","Pagination");let m=document.createElement("button");T(t,"prevBtn",m),m.setAttribute("class","pg-btn"),m.setAttribute("part","prev"),o.push(k(m,"disabled",()=>t.page()<=1)),m.setAttribute("aria-label","Previous page"),o.push(k(m,"aria-disabled",()=>t.page()<=1?"true":"false"));let p=document.createTextNode("Prev");m.appendChild(p),c.appendChild(m);let l=document.createElement("span");l.setAttribute("class","info"),l.setAttribute("part","info");let d=document.createTextNode("");l.appendChild(d),o.push(F(d,()=>t.info())),c.appendChild(l);let a=document.createElement("span");a.setAttribute("class","spacer"),c.appendChild(a);let s=document.createElement("button");T(t,"nextBtn",s),s.setAttribute("class","pg-btn"),s.setAttribute("part","next"),o.push(k(s,"disabled",()=>t.page()>=t.totalPages())),s.setAttribute("aria-label","Next page"),o.push(k(s,"aria-disabled",()=>t.page()>=t.totalPages()?"true":"false"));let u=document.createTextNode("Next");if(s.appendChild(u),c.appendChild(s),f.appendChild(c),i!=null){let b=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(b){let h=new Proxy(t,{get(v,g){return v[g]},set(v,g,x){let A=v[g];if(A!=null&&typeof A.set==="function"){if(typeof x==="function")A.set(()=>x);else A.set(x);return!0}return!1}});if(b.mount){let v=b.mount(h,r);if(typeof v==="function")o.push(v)}if(b.unmount)o.push(()=>b.unmount(h))}}return t}});function Yr(n,e=Date.now()){let r=Date.parse(n);if(Number.isNaN(r))return n;let o=e-r,t=Math.abs(o),i=o>=0?"ago":"from now";if(t<60000)return"just now";if(t<3600000)return`${Math.floor(t/60000)}m ${i}`;if(t<86400000)return`${Math.floor(t/3600000)}h ${i}`;let f=Math.floor(t/86400000);if(f<30)return`${f}d ${i}`;if(f<365)return`${Math.floor(f/30)}mo ${i}`;return`${Math.floor(f/365)}y ${i}`}function Zr(n){let e=Date.parse(n);return Number.isNaN(e)?n:new Date(e).toISOString()}var mi=60000,Dn=P({scope:({value:n})=>{let e=M(0),r=Z(()=>{let t=n();return t?Zr(t):""}),o=Z(()=>{e();let t=n();return t?Yr(t):""});return{tick:e,absStr:r,relStr:o}},mount(n,e){let r=D(()=>{if(n.value())e.title=n.absStr()}),o=setInterval(()=>n.tick.set(n.tick()+1),mi);return()=>{r(),clearInterval(o)}}});var sf=z("sui-timestamp",{props:{value:y.string({default:"",reflect:!0}),format:y.union(["relative","absolute","both"],{default:"relative",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline;
    font-family: var(--font-body);
    font-size: inherit;
    color: inherit;
  }
  .abs { color: var(--muted-foreground); margin-left: 4px; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.format=e.format;let i=null;if(Dn&&typeof Dn.setup==="function"){if(n.scope=t,i=Dn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(d)=>B({anchor:d,test:()=>{if(t.format()==="absolute")return 0;return 1},branches:[()=>{let a=document.createDocumentFragment(),s=[],u=document.createTextNode("");return a.appendChild(u),s.push(F(u,()=>t.absStr())),{nodes:Array.from(a.childNodes),cleanup:()=>{for(let b of s)try{b()}catch(h){}}}},()=>{let a=document.createDocumentFragment(),s=[],u=document.createTextNode("");return a.appendChild(u),s.push(F(u,()=>t.relStr())),{nodes:Array.from(a.childNodes),cleanup:()=>{for(let b of s)try{b()}catch(h){}}}}]}),c=(d)=>B({anchor:d,test:()=>{if(t.format()==="both")return 0;return-1},branches:[()=>{let a=document.createDocumentFragment(),s=[],u=document.createElement("span");u.setAttribute("class","abs"),u.setAttribute("part","abs");let b=document.createTextNode("(");u.appendChild(b);let h=document.createTextNode("");u.appendChild(h),s.push(F(h,()=>t.absStr()));let v=document.createTextNode(")");return u.appendChild(v),a.appendChild(u),{nodes:Array.from(a.childNodes),cleanup:()=>{for(let g of s)try{g()}catch(x){}}}}]}),m=(d)=>B({anchor:d,test:()=>{if(t.value())return 0;return-1},branches:[()=>{let a=document.createDocumentFragment(),s=[],u=document.createElement("time");s.push(k(u,"datetime",()=>t.value())),u.setAttribute("part","time");let b=document.createComment("if");u.appendChild(b),s.push(f(b)),a.appendChild(u);let h=document.createComment("if");return a.appendChild(h),s.push(c(h)),{nodes:Array.from(a.childNodes),cleanup:()=>{for(let v of s)try{v()}catch(g){}}}}]}),p=r.shadowRoot??r;L(p);let l=document.createComment("if");if(p.appendChild(l),o.push(m(l)),i!=null){let d=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(d){let a=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(d.mount){let s=d.mount(a,r);if(typeof s==="function")o.push(s)}if(d.unmount)o.push(()=>d.unmount(a))}}return t}});var df=z("sui-metachip",{props:{label:y.string({default:"",reflect:!0}),value:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.label=e.label,t.value=e.value;let i=r.shadowRoot??r;L(i);let f=document.createElement("span");f.setAttribute("class","k"),f.setAttribute("part","label");let c=document.createElement("slot");c.setAttribute("name","label");let m=document.createTextNode("");c.appendChild(m),o.push(F(m,()=>t.label())),f.appendChild(c),i.appendChild(f);let p=document.createElement("span");p.setAttribute("class","v"),p.setAttribute("part","value");let l=document.createElement("slot"),d=document.createTextNode("");return l.appendChild(d),o.push(F(d,()=>t.value())),p.appendChild(l),i.appendChild(p),t}});var bi=0,Qr=typeof CSS<"u"&&typeof CSS.supports==="function"&&CSS.supports("anchor-name","--x"),On=P({mount(n,e){let r=n,o=++bi,t=`--sui-popover-${o}`,i=`sui-popover-panel-${o}`;e.style.setProperty("--sui-popover-anchor",t);let f=r.triggerSlot(),c=r.panelEl();if(!c)return;c.id=i,Xe(r.triggerSlot,(a)=>{for(let s of a)s.setAttribute("popovertarget",i),s.setAttribute("aria-haspopup","dialog"),s.setAttribute("aria-controls",i),s.setAttribute("aria-expanded",r.open()?"true":"false")});function m(a){let u=a.newState==="open",b=e.hasAttribute("open");if(u&&!b)e.setAttribute("open","");else if(!u&&b)e.removeAttribute("open");let v=f?.assignedElements({flatten:!0})??[];for(let g of v)g.setAttribute("aria-expanded",u?"true":"false");if(e.emit("sui-toggle",{open:u}),u&&!Qr)l()}Q(c,"toggle",m),e.expose({show:()=>c.showPopover?.(),hide:()=>c.hidePopover?.(),toggle:()=>typeof c.togglePopover==="function"?void c.togglePopover():c.matches?.(":popover-open")?c.hidePopover?.():c.showPopover?.()});let p=D(()=>{let a=r.open(),s=c.matches?.(":popover-open")??!1;if(a&&!s)c.showPopover?.();else if(!a&&s)c.hidePopover?.()});function l(){let u=(f?.assignedElements({flatten:!0})??[])[0];if(!u||!c)return;let b=u.getBoundingClientRect(),h=r.placement();c.style.top="",c.style.left="",c.style.right="",c.style.bottom="";let v=6;switch(h){case"top":c.style.bottom=`${window.innerHeight-b.top+v}px`,c.style.left=`${b.left}px`;break;case"top-end":c.style.bottom=`${window.innerHeight-b.top+v}px`,c.style.right=`${window.innerWidth-b.right}px`;break;case"bottom-end":c.style.top=`${b.bottom+v}px`,c.style.right=`${window.innerWidth-b.right}px`;break;case"left":c.style.right=`${window.innerWidth-b.left+v}px`,c.style.top=`${b.top}px`;break;case"left-end":c.style.right=`${window.innerWidth-b.left+v}px`,c.style.bottom=`${window.innerHeight-b.bottom}px`;break;case"right":c.style.left=`${b.right+v}px`,c.style.top=`${b.top}px`;break;case"right-end":c.style.left=`${b.right+v}px`,c.style.bottom=`${window.innerHeight-b.bottom}px`;break;case"bottom":default:c.style.top=`${b.bottom+v}px`,c.style.left=`${b.left}px`}}function d(){if(!r.open())return;l()}if(!Qr)Q(window,"resize",d),Q(window,"scroll",d,{capture:!0,passive:!0});return()=>{p()}}});var hf=z("sui-popover",{props:{open:y.bool({default:!1,reflect:!0}),placement:y.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.open=e.open,t.placement=e.placement;let i=null;if(On&&typeof On.setup==="function"){if(n.scope=t,i=On.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(s)=>B({anchor:s,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("div");h.setAttribute("class","surface"),ot(h,We,{duration:120},"both");let v=document.createElement("slot");return h.appendChild(v),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let g of b)try{g()}catch(x){}}}}]}),c=(s)=>B({anchor:s,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("sui-portal");h.setAttribute("target","body");let v=document.createElement("div");return v.setAttribute("class","sui-popover-presence"),v.setAttribute("aria-hidden","true"),v.setAttribute("data-sui-popover-open",""),h.appendChild(v),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let g of b)try{g()}catch(x){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createElement("slot");p.setAttribute("name","trigger"),T(t,"triggerSlot",p),m.appendChild(p);let l=document.createElement("div");l.setAttribute("class","panel"),l.setAttribute("part","panel"),l.setAttribute("popover","auto"),T(t,"panelEl",l),l.setAttribute("role","dialog"),l.setAttribute("aria-modal","false");let d=document.createComment("if");l.appendChild(d),o.push(f(d)),m.appendChild(l);let a=document.createComment("if");if(m.appendChild(a),o.push(c(a)),i!=null){let s=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,v){let g=b[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var er=[{id:"ship-light",label:"SHIP Light",dark:!1,swatches:["oklch(0.99 0.001 106.423)","oklch(1 0 0)","oklch(0.67 0.16 58)","oklch(0.61 0.23 303)"]},{id:"ship-dark",label:"SHIP Dark",dark:!0,swatches:["oklch(0.18 0.01 250)","oklch(0.18 0.01 50)","oklch(0.77 0.16 70)","oklch(0.3 0.05 300)"]},{id:"paper",label:"Paper",dark:!1,swatches:["oklch(0.97 0.014 80)","oklch(0.99 0.010 80)","oklch(0.55 0.14 35)","oklch(0.48 0.09 55)"]},{id:"halo",label:"Halo (Soft)",dark:!1,swatches:["oklch(0.92 0.008 230)","oklch(0.92 0.008 230)","oklch(0.62 0.14 50)","oklch(0.55 0.05 230)"]},{id:"console",label:"Console",dark:!0,swatches:["oklch(0.13 0.008 220)","oklch(0.16 0.010 220)","oklch(0.78 0.20 145)","oklch(0.78 0.16 195)"]},{id:"contrast",label:"High Contrast",dark:!0,swatches:["oklch(0.09 0 0)","oklch(0.15 0 0)","oklch(0.87 0.18 85)","oklch(0.99 0 0)"]},{id:"glass",label:"Glass",dark:!1,swatches:["oklch(0.95 0.04 230)","oklch(0.99 0.005 240)","oklch(0.62 0.16 50)","oklch(0.62 0.14 280)"]}],Vr=Object.fromEntries(er.map((n)=>[n.id,n]));var Tn=(n)=>typeof n==="string"&&(n in Vr),hi=(n)=>Vr[n]?.dark??!1;function rr(n,e=document.documentElement){let r=Tn(n)?n:"ship-light",o=hi(r);if(e.dataset.theme=r,e.classList.toggle("dark",o),typeof document<"u"&&typeof CustomEvent<"u")document.dispatchEvent(new CustomEvent("ship:theme:change",{detail:{id:r,dark:o}}));return r}function or(){if(typeof localStorage>"u")return"ship-light";try{let n=localStorage.getItem("ship-site-theme");return Tn(n)?n:"ship-light"}catch{return"ship-light"}}function Kr(n){if(typeof localStorage>"u")return;try{localStorage.setItem("ship-site-theme",n)}catch{}try{fetch("/api/settings",{method:"PUT",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:"shell.theme",value:n})}).catch(()=>{})}catch{}}var vi=[{label:"Default",ids:["ship-light","ship-dark"]},{label:"Texture",ids:["paper","halo"]},{label:"Effects",ids:["glass"]},{label:"Special",ids:["console","contrast"]}],Pn=new Map(er.map((n)=>[n.id,n]));function gi(n){let e=n.shadowRoot;if(!e)return;if(e.querySelector("link[data-sui-theme-preview-styles]"))return;let r=document.createElement("link");r.rel="stylesheet",r.href="/api/assets/view-tokens.css",r.setAttribute("data-sui-theme-preview-styles",""),e.appendChild(r)}var Mn=P({scope:(n)=>{let e=n,r=M(or()),o=M(!1),t=M(vi.map((a)=>({label:a.label,themes:a.ids.map((s)=>Pn.get(s)).filter((s)=>!!s)}))),i=Z(()=>Pn.get(r())?.swatches[2]??"var(--primary, #3b82f6)"),f=Z(()=>Pn.get(r())?.label??"Theme"),c={host:null,panel:null};function m(){let a=c.panel;if(!a)return;if(o())a.hidePopover?.();else a.showPopover?.()}function p(a){let s=rr(a);if(e.persist())Kr(s);r.set(s);let u=c.host;if(u)u.dispatchEvent(new CustomEvent("sui-theme-change",{detail:{id:s,dark:Pn.get(s)?.dark??!1},bubbles:!0,composed:!0}));c.panel?.hidePopover?.()}let l=J(null),d=J(null);return{activeTheme:r,panelOpen:o,groups:t,dotColor:i,triggerLabel:f,togglePanel:m,selectTheme:p,triggerBtn:l,panelEl:d,_internals:c}},mount(n,e){let r=n;r._internals.host=e,gi(e),rr(or());let o=D(()=>{let p=r.panelEl();if(r._internals.panel=p,!p)return;let l=(d)=>{let s=d.newState==="open";if(r.panelOpen()!==s)r.panelOpen.set(s);if(s)t()};return p.addEventListener("toggle",l),()=>p.removeEventListener("toggle",l)});function t(){let p=r.panelEl(),l=r.triggerBtn();if(!p||!l)return;let d=l.getBoundingClientRect();p.style.top=`${d.bottom+4}px`,p.style.left=`${Math.max(8,d.right-p.offsetWidth)}px`}function i(p){let l=p.detail?.id;if(!Tn(l))return;if(r.activeTheme()!==l)r.activeTheme.set(l)}document.addEventListener("ship:theme:change",i);function f(p){let l=r.panelEl();if(!l)return;let d=Array.from(l.querySelectorAll(".card"));if(!d.length)return;let a=l.getRootNode(),s=d.findIndex((u)=>u===a.activeElement);switch(p.key){case"ArrowDown":p.preventDefault(),d[(s+1+d.length)%d.length]?.focus();break;case"ArrowUp":p.preventDefault(),d[(s-1+d.length)%d.length]?.focus();break;case"Home":p.preventDefault(),d[0]?.focus();break;case"End":p.preventDefault(),d[d.length-1]?.focus();break;case"Escape":p.preventDefault(),l.hidePopover?.();break}}let c=D(()=>{let p=r.panelEl();if(!p)return;return p.addEventListener("keydown",f),()=>p.removeEventListener("keydown",f)}),m=D(()=>{if(!r.panelOpen())return;requestAnimationFrame(()=>{let p=r.panelEl();if(!p)return;let l=Array.from(p.querySelectorAll(".card"));(l.find((a)=>a.getAttribute("aria-selected")==="true")??l[0])?.focus()})});return()=>{document.removeEventListener("ship:theme:change",i),o(),c(),m()}}});var kf=z("sui-theme-picker",{props:{placement:y.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom-end",reflect:!0}),"label-style":y.union(["text","compact"],{default:"text",reflect:!0}),persist:y.bool({default:!0,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.placement=e.placement,t.labelStyle=e.labelStyle,t.persist=e.persist;let i=null;if(Mn&&typeof Mn.setup==="function"){if(n.scope=t,i=Mn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(v)=>R({anchor:v,items:()=>t.groups(),key:(g,x)=>{let A=Object.create(t);return A.g=g,A.__i=x,((w)=>g.label)(A)},render:(g,x)=>{let A=Object.create(t);return A.g=g,A.__i=x,A.__v=M(0),((w)=>{let E=(G)=>R({anchor:G,items:()=>w.g.themes,key:(X,Y)=>{let W=Object.create(w);return W.t=X,W.__i=Y,((q)=>X.id)(W)},render:(X,Y)=>{let W=Object.create(w);return W.t=X,W.__i=Y,W.__v=M(0),((q)=>{let _=document.createDocumentFragment(),V=[],I=document.createElement("button");I.setAttribute("class","card"),I.setAttribute("role","option"),I.setAttribute("type","button"),V.push(k(I,"data-theme-card",()=>(q.__v(),q.t.id))),V.push(k(I,"aria-label",()=>(q.__v(),q.t.label))),V.push(k(I,"aria-selected",()=>(q.__v(),q.activeTheme()===q.t.id?"true":"false"))),V.push(k(I,"tabindex",()=>(q.__v(),q.activeTheme()===q.t.id?"0":"-1"))),V.push(K(I,"click",function(){let gt=()=>q.selectTheme(q.t.id);return typeof gt==="function"?gt.bind(void 0):()=>{}}(),{modifiers:[]}));let dt=document.createElement("span");dt.setAttribute("class","tc-bg"),V.push(k(dt,"data-theme-preview",()=>(q.__v(),q.t.id))),dt.setAttribute("aria-hidden","true");let dr=document.createElement("span");dr.setAttribute("class","tc-overlay"),dt.appendChild(dr);let Dt=document.createElement("span");Dt.setAttribute("class","tc-card");let Ut=document.createElement("span");Ut.setAttribute("class","tc-card-head");let Ce=document.createElement("span");Ce.setAttribute("class","tc-eyebrow");let bo=document.createTextNode("THEME");Ce.appendChild(bo),Ut.appendChild(Ce);let je=document.createElement("span");je.setAttribute("class","tc-heading");let cr=document.createTextNode("");je.appendChild(cr),V.push(F(cr,()=>(q.__v(),q.t.label))),Ut.appendChild(je),Dt.appendChild(Ut);let Xt=document.createElement("span");Xt.setAttribute("class","tc-typo");let ze=document.createElement("span");ze.setAttribute("class","tc-body");let ho=document.createTextNode("The quick brown fox");ze.appendChild(ho),Xt.appendChild(ze);let Le=document.createElement("span");Le.setAttribute("class","tc-mono");let pr=document.createTextNode("");Le.appendChild(pr),V.push(F(pr,()=>(q.__v(),"{ build: ok }"))),Xt.appendChild(Le),Dt.appendChild(Xt);let vt=document.createElement("span");vt.setAttribute("class","tc-controls");let De=document.createElement("span");De.setAttribute("class","tc-cta");let vo=document.createTextNode("Action");De.appendChild(vo),vt.appendChild(De);let Oe=document.createElement("span");Oe.setAttribute("class","tc-ghost");let go=document.createTextNode("Cancel");Oe.appendChild(go),vt.appendChild(Oe);let Te=document.createElement("span");Te.setAttribute("class","tc-chip tc-chip-accent");let yo=document.createTextNode("New");Te.appendChild(yo),vt.appendChild(Te);let Pe=document.createElement("span");Pe.setAttribute("class","tc-chip tc-chip-muted");let xo=document.createTextNode("Idle");Pe.appendChild(xo),vt.appendChild(Pe),Dt.appendChild(vt),dt.appendChild(Dt),I.appendChild(dt);let qt=document.createElement("span");qt.setAttribute("class","row");let Me=document.createElement("span");Me.setAttribute("class","name");let fr=document.createTextNode("");Me.appendChild(fr),V.push(F(fr,()=>(q.__v(),q.t.label))),qt.appendChild(Me);let Ne=document.createElement("span");return Ne.setAttribute("class","radio"),Ne.setAttribute("aria-hidden","true"),qt.appendChild(Ne),I.appendChild(qt),_.appendChild(I),{nodes:Array.from(_.childNodes),cleanup:()=>{for(let gt of V)try{gt()}catch(mr){}},update:(gt,mr)=>{q.t=gt,q.__i=mr,q.__v.set((wo)=>wo+1)}}})(W)}}),j=document.createDocumentFragment(),C=[],N=document.createElement("section");N.setAttribute("class","group"),C.push(k(N,"aria-label",()=>(w.__v(),w.g.label)));let O=document.createElement("div");O.setAttribute("class","group-h");let S=document.createTextNode("");O.appendChild(S),C.push(F(S,()=>(w.__v(),w.g.label))),N.appendChild(O);let H=document.createElement("div");H.setAttribute("class","group-rows");let U=document.createComment("for");return H.appendChild(U),C.push(E(U)),N.appendChild(H),j.appendChild(N),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let G of C)try{G()}catch(X){}},update:(G,X)=>{w.g=G,w.__i=X,w.__v.set((Y)=>Y+1)}}})(A)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("button");m.setAttribute("class","trigger"),m.setAttribute("part","trigger"),m.setAttribute("type","button"),T(t,"triggerBtn",m),m.setAttribute("aria-label","Choose theme"),m.setAttribute("aria-haspopup","dialog"),o.push(k(m,"aria-expanded",()=>t.panelOpen()?"true":"false")),o.push(K(m,"click",function(){let v=()=>t.togglePanel();return typeof v==="function"?v.bind(void 0):()=>{}}(),{modifiers:[]}));let p=document.createElement("span");p.setAttribute("class","dot"),p.setAttribute("part","dot"),o.push(D(()=>{let v=t.dotColor();if(v==null)p.style.removeProperty("background");else p.style.setProperty("background",String(v))})),p.setAttribute("aria-hidden","true"),m.appendChild(p);let l=document.createElement("span");l.setAttribute("class","label"),l.setAttribute("part","label");let d=document.createTextNode("");l.appendChild(d),o.push(F(d,()=>t.triggerLabel())),m.appendChild(l);let a=document.createElement("svg");a.setAttribute("class","caret"),a.setAttribute("viewbox","0 0 10 6"),a.setAttribute("aria-hidden","true");let s=document.createElement("path");s.setAttribute("d","M1 1l4 4 4-4"),s.setAttribute("fill","none"),s.setAttribute("stroke","currentColor"),s.setAttribute("stroke-width","1.5"),s.setAttribute("stroke-linecap","round"),s.setAttribute("stroke-linejoin","round"),a.appendChild(s),m.appendChild(a),c.appendChild(m);let u=document.createElement("div");u.setAttribute("class","panel"),u.setAttribute("part","panel"),u.setAttribute("popover","auto"),T(t,"panelEl",u);let b=document.createElement("div");b.setAttribute("class","grid"),b.setAttribute("part","grid"),b.setAttribute("role","listbox"),b.setAttribute("aria-label","Theme");let h=document.createComment("for");if(b.appendChild(h),o.push(f(h)),u.appendChild(b),c.appendChild(u),i!=null){let v=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(v){let g=new Proxy(t,{get(x,A){return x[A]},set(x,A,w){let E=x[A];if(E!=null&&typeof E.set==="function"){if(typeof w==="function")E.set(()=>w);else E.set(w);return!0}return!1}});if(v.mount){let x=v.mount(g,r);if(typeof x==="function")o.push(x)}if(v.unmount)o.push(()=>v.unmount(g))}}return t}});var yi=`
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
`,Ir=!1;function xi(){if(Ir)return;Ir=!0;let n=document.createElement("style");n.setAttribute("data-sui-rail",""),n.textContent=yi,document.head.appendChild(n)}function Nn(n,e,r){n.dispatchEvent(new CustomEvent(e,{detail:r,bubbles:!0,composed:!0}))}function $r(n){xi();let{host:e,open:r,projects:o,activeProjectId:t,popoverEl:i,badgeEl:f}=n,c=()=>r.set(!1),m=async()=>{try{let g=await fetch("/api/v1/projects",{credentials:"same-origin"});if(!g.ok)return;let x=await g.json();o.set(x.projects??[]),t.set(x.active??null)}catch{}},p=async(g)=>{try{await fetch("/api/v1/projects/switch",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({id:g})}),t.set(g),c(),Nn(e,"ship:project:change",{id:g})}catch{}m()},l=async()=>{let g=window.prompt("Path to project directory:");if(!g)return;try{let x=await fetch("/api/v1/projects",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({path:g})});if(!x.ok){let A=await x.text().catch(()=>"");Nn(e,"sui:notify",{tone:"warning",title:`Add failed: ${A||x.status}`});return}await m(),Nn(e,"sui:notify",{tone:"success",title:"Project added."})}catch(x){Nn(e,"sui:notify",{tone:"destructive",title:String(x.message??x)})}},d=()=>{let g=i(),x=f();if(!g||!x)return;let A=x.getBoundingClientRect(),w=g.offsetWidth||220,E=8,j=Math.max(8,Math.min(A.left,window.innerWidth-w-8)),C=Math.max(8,window.innerHeight-A.top+E);g.style.left=`${j}px`,g.style.bottom=`${C}px`},a=(g)=>{let x=g.target;if(!x)return;if(x.closest("[data-rail-add]")){l();return}let A=x.closest("[data-project-id]");if(A?.dataset.projectId)p(A.dataset.projectId)},s=()=>r.set(!r()),u=(g)=>{if(!r())return;let x=typeof g.composedPath==="function"?g.composedPath():[],A=i(),w=f();for(let E of x)if(E===A||E===w)return;c()},b=()=>{if(r())d()},h=D(()=>{let g=f();if(!g)return;return g.addEventListener("click",s),()=>g.removeEventListener("click",s)}),v=D(()=>{let g=i();if(!g)return;return requestAnimationFrame(d),g.addEventListener("click",a),()=>g.removeEventListener("click",a)});return document.addEventListener("click",u,!0),window.addEventListener("resize",b),window.addEventListener("scroll",b,{capture:!0,passive:!0}),{loadProjects:m,dispose(){h(),v(),document.removeEventListener("click",u,!0),window.removeEventListener("resize",b),window.removeEventListener("scroll",b,!0)}}}var wi=15000;function _r(n){return n.split("/").filter(Boolean)[0]??"dashboard"}function ir(n,e,r){n.dispatchEvent(new CustomEvent(e,{detail:r,bubbles:!0,composed:!0}))}var Sn=P({scope:()=>{let n=M(!1),e=M([]),r=M(null),o=M(typeof location<"u"?_r(location.pathname):""),t=M(0);return{projectOpen:n,projects:e,activeProjectId:r,activeView:o,sessionCount:t,projectInitial:()=>{let f=e(),c=f.find((m)=>m.id===r())??f[0];if(!c)return"";return(c.label??c.name).charAt(0).toUpperCase()},popoverEl:J(),projectBadgeEl:J(),launcherBtnEl:J(),createBtnEl:J(),toggleBtn:J(),missionEl:J(),navSlot:J()}},mount(n,e){let r=D(()=>{let g=n.toggleBtn(),x=n.collapsed();if(g)g.setAttribute("aria-label",x?"Show rail":"Hide rail")}),o=()=>{let g=!n.collapsed();n.collapsed.set(g),ir(e,"sui-rail:toggle",{collapsed:g})},t=D(()=>{let g=n.toggleBtn();if(!g)return;return g.addEventListener("click",o),()=>g.removeEventListener("click",o)}),i=$r({host:e,open:n.projectOpen,projects:n.projects,activeProjectId:n.activeProjectId,popoverEl:n.popoverEl,badgeEl:n.projectBadgeEl});i.loadProjects();let f=()=>n.projectOpen.set(!1),c=(g)=>{ir(e,"ship:launcher:open",{anchor:g.currentTarget}),f()},m=()=>{ir(e,"ship:create:open"),f()},p=D(()=>{let g=n.launcherBtnEl();if(!g)return;return g.addEventListener("click",c),()=>g.removeEventListener("click",c)}),l=D(()=>{let g=n.createBtnEl();if(!g)return;return g.addEventListener("click",m),()=>g.removeEventListener("click",m)}),d=async()=>{try{let g=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(!g.ok)return;let A=(await g.json()).sessions.filter((w)=>w.status!=="ended").length;n.sessionCount.set(A)}catch{}};d();let a=window.setInterval(()=>void d(),wi),s=()=>void d();window.addEventListener("ship:session:changed",s);let u=D(()=>{let g=n.missionEl();if(!g)return;let x=n.sessionCount();g.setAttribute("badge",x>0?String(x):"")}),b=()=>{n.activeView.set(_r(location.pathname))},h=()=>b();document.addEventListener("sui:navigate",h),window.addEventListener("popstate",h);let v=D(()=>{let g=n.activeView(),x=e.querySelectorAll("sui-rail-item[data-view]");for(let w of x)if(w.dataset.view===g)w.setAttribute("active","");else w.removeAttribute("active");let A=n.missionEl();if(A)if(g==="mesh")A.setAttribute("active","");else A.removeAttribute("active")});return()=>{r(),t(),p(),l(),u(),v(),i.dispose(),document.removeEventListener("sui:navigate",h),window.removeEventListener("popstate",h),window.removeEventListener("ship:session:changed",s),window.clearInterval(a)}}});var Mf=z("sui-rail",{props:{collapsed:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.collapsed=e.collapsed;let i=null;if(Sn&&typeof Sn.setup==="function"){if(n.scope=t,i=Sn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(C)=>B({anchor:C,test:()=>{if(t.projects().length===0)return 0;return-1},branches:[()=>{let N=document.createDocumentFragment(),O=[],S=document.createElement("div");S.setAttribute("class","sui-rail-project-popover-empty");let H=document.createTextNode("No projects tracked.");return S.appendChild(H),N.appendChild(S),{nodes:Array.from(N.childNodes),cleanup:()=>{for(let U of O)try{U()}catch(G){}}}}]}),c=(C)=>R({anchor:C,items:()=>t.projects(),key:(N,O)=>{let S=Object.create(t);return S.p=N,S.__i=O,((H)=>N.id)(S)},render:(N,O)=>{let S=Object.create(t);return S.p=N,S.__i=O,S.__v=M(0),((H)=>{let U=document.createDocumentFragment(),G=[],X=document.createElement("div");X.setAttribute("class","sui-rail-project-popover-row"),G.push(pt(X,"active",()=>(H.__v(),H.p.id===H.activeProjectId()))),G.push(k(X,"data-project-id",()=>(H.__v(),H.p.id)));let Y=document.createElement("span");Y.setAttribute("class","sui-rail-project-popover-row-name");let W=document.createTextNode("");Y.appendChild(W),G.push(F(W,()=>(H.__v(),H.p.label||H.p.name))),X.appendChild(Y);let q=document.createElement("span");q.setAttribute("class","sui-rail-project-popover-row-path");let _=document.createTextNode("");return q.appendChild(_),G.push(F(_,()=>(H.__v(),H.p.path))),X.appendChild(q),U.appendChild(X),{nodes:Array.from(U.childNodes),cleanup:()=>{for(let V of G)try{V()}catch(I){}},update:(V,I)=>{H.p=V,H.__i=I,H.__v.set((dt)=>dt+1)}}})(S)}}),m=(C)=>B({anchor:C,test:()=>{if(t.projectOpen())return 0;return-1},branches:[()=>{let N=document.createDocumentFragment(),O=[],S=document.createElement("div");S.setAttribute("class","sui-rail-project-popover"),S.setAttribute("part","project-popover"),S.setAttribute("role","dialog"),S.setAttribute("aria-label","Switch project"),T(t,"popoverEl",S),ot(S,ht,{duration:150},"both");let H=document.createElement("div");H.setAttribute("class","sui-rail-project-popover-title");let U=document.createTextNode("Projects");H.appendChild(U),S.appendChild(H);let G=document.createComment("if");S.appendChild(G),O.push(f(G));let X=document.createComment("for");S.appendChild(X),O.push(c(X));let Y=document.createElement("div");Y.setAttribute("class","sui-rail-project-popover-sep"),S.appendChild(Y);let W=document.createElement("div");W.setAttribute("class","sui-rail-project-popover-row add"),W.setAttribute("data-rail-add","");let q=document.createElement("span"),_=document.createTextNode("+ Add project");return q.appendChild(_),W.appendChild(q),S.appendChild(W),N.appendChild(S),{nodes:Array.from(N.childNodes),cleanup:()=>{for(let V of O)try{V()}catch(I){}}}}]}),p=r.shadowRoot??r;L(p);let l=document.createElement("nav");l.setAttribute("class","rail-nav"),l.setAttribute("part","nav");let d=document.createElement("slot");T(t,"navSlot",d),l.appendChild(d);let a=document.createElement("sui-rail-item");T(t,"missionEl",a),a.setAttribute("part","mission-control"),a.setAttribute("data-view","mesh"),a.setAttribute("icon","message-square-plus"),a.setAttribute("label","Mission Control"),a.setAttribute("href","/mesh"),l.appendChild(a),p.appendChild(l);let s=document.createElement("div");s.setAttribute("class","rail-footer"),s.setAttribute("part","footer");let u=document.createElement("button");u.setAttribute("type","button"),u.setAttribute("class","project-badge"),u.setAttribute("part","project-badge"),u.setAttribute("aria-label","Switch project"),u.setAttribute("aria-haspopup","dialog"),T(t,"projectBadgeEl",u);let b=document.createTextNode("");u.appendChild(b),o.push(F(b,()=>t.projectInitial())),s.appendChild(u);let h=document.createElement("div");h.setAttribute("class","footer-actions");let v=document.createElement("button");v.setAttribute("type","button"),v.setAttribute("class","footer-btn"),v.setAttribute("part","launcher-btn"),v.setAttribute("aria-label","Open launcher"),T(t,"launcherBtnEl",v);let g=document.createElement("sui-icon");g.setAttribute("name","folder"),g.setAttribute("size","sm"),v.appendChild(g),h.appendChild(v);let x=document.createElement("button");x.setAttribute("type","button"),x.setAttribute("class","footer-btn"),x.setAttribute("part","create-btn"),x.setAttribute("aria-label","Create new"),T(t,"createBtnEl",x);let A=document.createElement("sui-icon");A.setAttribute("name","plus"),A.setAttribute("size","sm"),x.appendChild(A),h.appendChild(x);let w=document.createElement("button");w.setAttribute("type","button"),w.setAttribute("class","toggle-btn"),w.setAttribute("part","toggle"),w.setAttribute("aria-label","Collapse rail"),T(t,"toggleBtn",w),h.appendChild(w),s.appendChild(h),p.appendChild(s);let E=document.createElement("sui-portal");E.setAttribute("target","body");let j=document.createComment("if");if(E.appendChild(j),o.push(m(j)),p.appendChild(E),i!=null){let C=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(C){let N=new Proxy(t,{get(O,S){return O[S]},set(O,S,H){let U=O[S];if(U!=null&&typeof U.set==="function"){if(typeof H==="function")U.set(()=>H);else U.set(H);return!0}return!1}});if(C.mount){let O=C.mount(N,r);if(typeof O==="function")o.push(O)}if(C.unmount)o.push(()=>C.unmount(N))}}return t}});var Hf=z("sui-rail-item",{props:{href:y.string({default:""}),label:y.string({default:""}),icon:y.string({default:""}),active:y.bool({default:!1,reflect:!0}),badge:y.string({default:""}),disabled:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.href=e.href,t.label=e.label,t.icon=e.icon,t.active=e.active,t.badge=e.badge,t.disabled=e.disabled;let i=r.shadowRoot??r;L(i);let f=document.createElement("a");f.setAttribute("class","item"),f.setAttribute("part","item"),T(t,"itemEl",f),o.push(k(f,"href",()=>t.href()||"#")),o.push(k(f,"aria-current",()=>t.active()?"page":null)),o.push(k(f,"aria-disabled",()=>t.disabled()?"true":null)),o.push(k(f,"title",()=>t.label()));let c=document.createElement("span");c.setAttribute("class","icon-wrap"),c.setAttribute("part","icon-wrap");let m=document.createElement("sui-icon");o.push(k(m,"name",()=>t.icon())),m.setAttribute("size","sm"),c.appendChild(m),f.appendChild(c);let p=document.createElement("span");p.setAttribute("class","label"),p.setAttribute("part","label");let l=document.createTextNode("");p.appendChild(l),o.push(F(l,()=>t.label())),f.appendChild(p);let d=document.createElement("span");d.setAttribute("class","badge"),d.setAttribute("part","badge"),T(t,"badgeEl",d);let a=document.createTextNode("");return d.appendChild(a),o.push(F(a,()=>t.badge())),f.appendChild(d),i.appendChild(f),t}});var Gf=z("sui-rail-section",{props:{label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.label=e.label;let i=r.shadowRoot??r;L(i);let f=document.createElement("span");f.setAttribute("class","heading"),f.setAttribute("part","heading");let c=document.createTextNode("");f.appendChild(c),o.push(F(c,()=>t.label())),i.appendChild(f);let m=document.createElement("slot");return i.appendChild(m),t}});var qf=z("sui-status-bar",{props:{branch:y.string({default:""}),theme:y.string({default:""}),online:y.bool({default:!1,reflect:!0}),agents:y.string({default:""})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.branch=e.branch,t.theme=e.theme,t.online=e.online,t.agents=e.agents;let i=r.shadowRoot??r;L(i);let f=document.createElement("span");f.setAttribute("class","dot"),f.setAttribute("part","dot"),f.setAttribute("aria-hidden","true"),i.appendChild(f);let c=document.createElement("span");c.setAttribute("class","text"),c.setAttribute("part","branch");let m=document.createTextNode("");c.appendChild(m),o.push(F(m,()=>t.branch())),i.appendChild(c);let p=document.createElement("span");p.setAttribute("class","text"),p.setAttribute("part","agents");let l=document.createTextNode("");p.appendChild(l),o.push(F(l,()=>t.agents())),i.appendChild(p);let d=document.createElement("span");d.setAttribute("class","slot-wrap");let a=document.createElement("slot");a.setAttribute("name","start"),d.appendChild(a),i.appendChild(d);let s=document.createElement("span");s.setAttribute("class","spacer"),i.appendChild(s);let u=document.createElement("span");u.setAttribute("class","slot-wrap");let b=document.createElement("slot");b.setAttribute("name","end"),u.appendChild(b),i.appendChild(u);let h=document.createElement("span");h.setAttribute("class","text"),h.setAttribute("part","theme");let v=document.createTextNode("");return h.appendChild(v),o.push(F(v,()=>t.theme())),i.appendChild(h),t}});var Ai=["a[href]","area[href]","button:not([disabled])",'input:not([disabled]):not([type="hidden"])',"select:not([disabled])","textarea:not([disabled])","iframe","object","embed","summary",'[contenteditable]:not([contenteditable="false"])','[tabindex]:not([tabindex^="-"])'].join(",");function zt(n){return Array.from(n.querySelectorAll(Ai)).filter((e)=>{if(e.hasAttribute("disabled"))return!1;if(e.getAttribute("aria-hidden")==="true")return!1;return!0})}class Ft{saved=null;save(){this.saved=document.activeElement??null}restore(){let n=this.saved;if(this.saved=null,n&&typeof n.focus==="function")n.focus()}}var Bt=0,ar=null,sr=null;function Hn(){if(typeof document>"u")return;if(Bt+=1,Bt>1)return;let n=document.body;ar=n.style.overflow,sr=n.style.paddingRight;let e=typeof window<"u"?window.innerWidth-document.documentElement.clientWidth:0;if(e>0){let r=parseFloat(getComputedStyle(n).paddingRight||"0")||0;n.style.paddingRight=`${r+e}px`}n.style.overflow="hidden"}function Lt(){if(typeof document>"u")return;if(Bt===0)return;if(Bt-=1,Bt>0)return;let n=document.body;n.style.overflow=ar??"",n.style.paddingRight=sr??"",ar=null,sr=null}var Ei=0,Bn=P({mount(n,e){let r=n,o=++Ei,t=`sui-dialog-header-${o}`,i=`sui-dialog-body-${o}`,f=r.dialogEl(),c=r.closeBtn(),m=r.sentinelStart(),p=r.sentinelEnd();if(!f)return;let l=D(()=>{let O=r.headerEl();if(O)O.id=t}),d=D(()=>{let O=r.bodyEl();if(O)O.id=i});f.setAttribute("aria-labelledby",t),f.setAttribute("aria-describedby",i);let a=!1,s=new Ft;function u(){f.setAttribute("role",r.roleOverride()||"dialog"),f.setAttribute("aria-modal","true")}u();function b(){let O=zt(f).filter((H)=>!H.classList.contains("sentinel"));return[...zt(e),...O]}function h(){let O=e.querySelector("[autofocus]")??f.querySelector("[autofocus]");if(O){O.focus();return}let S=b(),H=S.find((U)=>U!==c)??S[0];if(H)H.focus();else f.focus()}function v(){s.save();try{f.showModal()}catch{}if(!a)Hn(),a=!0;queueMicrotask(()=>h()),e.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function g(){if(f.open)f.close()}let x=()=>{if(r.open())r.open.set(!1);if(a)Lt(),a=!1;s.restore(),e.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},A=(O)=>{if(!r.dismissible())return;if(O.target===f)r.open.set(!1)},w=()=>{e.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),r.open.set(!1)},E=()=>{let O=b();if(O.length)O[O.length-1].focus()},j=()=>{let O=b();if(O.length)O[0].focus()};f.addEventListener("close",x),f.addEventListener("click",A),c?.addEventListener("click",w),m?.addEventListener("focus",E),p?.addEventListener("focus",j);let C=D(()=>{let O=r.open();if(e.setAttribute("data-state",O?"open":"closed"),O&&!f.open)v();else if(!O&&f.open)g()}),N=D(()=>{u()});return e.show=()=>r.open.set(!0),e.showModal=()=>r.open.set(!0),e.close=()=>r.open.set(!1),()=>{if(a)Lt();f.removeEventListener("close",x),f.removeEventListener("click",A),c?.removeEventListener("click",w),m?.removeEventListener("focus",E),p?.removeEventListener("focus",j),C(),N(),l(),d()}}});var Kf=z("sui-dialog",{props:{open:y.bool({default:!1,reflect:!0}),size:y.union(["sm","md","lg","full"],{default:"md",reflect:!0}),backdrop:y.union(["default","blur","solid"],{default:"default",reflect:!0}),dismissible:y.bool({default:!0}),"no-close-button":y.bool({default:!1,reflect:!0}),"no-header":y.bool({default:!1,reflect:!0}),"no-footer":y.bool({default:!1,reflect:!0}),"role-override":y.union(["dialog","alertdialog"],{default:"dialog",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.open=e.open,t.size=e.size,t.backdrop=e.backdrop,t.dismissible=e.dismissible,t.noCloseButton=e.noCloseButton,t.noHeader=e.noHeader,t.noFooter=e.noFooter,t.roleOverride=e.roleOverride;let i=null;if(Bn&&typeof Bn.setup==="function"){if(n.scope=t,i=Bn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(v)=>B({anchor:v,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),x=[],A=document.createElement("div");A.setAttribute("class","dialog-content"),ot(A,ht,{duration:150},"both");let w=document.createElement("div");w.setAttribute("class","header"),w.setAttribute("part","header"),T(t,"headerEl",w);let E=document.createElement("slot");E.setAttribute("name","header"),w.appendChild(E),A.appendChild(w);let j=document.createElement("div");j.setAttribute("class","body"),j.setAttribute("part","body"),T(t,"bodyEl",j);let C=document.createElement("slot");j.appendChild(C),A.appendChild(j);let N=document.createElement("div");N.setAttribute("class","footer"),N.setAttribute("part","footer");let O=document.createElement("slot");return O.setAttribute("name","footer"),N.appendChild(O),A.appendChild(N),g.appendChild(A),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let S of x)try{S()}catch(H){}}}}]}),c=(v)=>B({anchor:v,test:()=>{if(t.open())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),x=[],A=document.createElement("sui-portal");A.setAttribute("target","body");let w=document.createElement("div");return w.setAttribute("class","sui-dialog-presence"),w.setAttribute("aria-hidden","true"),w.setAttribute("data-sui-dialog-open",""),A.appendChild(w),g.appendChild(A),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let E of x)try{E()}catch(j){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createElement("dialog");p.setAttribute("part","dialog"),T(t,"dialogEl",p);let l=document.createElement("span");l.setAttribute("tabindex","0"),l.setAttribute("class","sentinel"),l.setAttribute("aria-hidden","true"),T(t,"sentinelStart",l),p.appendChild(l);let d=document.createElement("button");d.setAttribute("class","close"),d.setAttribute("part","close"),d.setAttribute("type","button"),d.setAttribute("aria-label","Close dialog"),T(t,"closeBtn",d);let a=document.createElement("svg");a.setAttribute("viewbox","0 0 16 16"),a.setAttribute("fill","none"),a.setAttribute("stroke","currentColor"),a.setAttribute("stroke-width","1.75"),a.setAttribute("stroke-linecap","round"),a.setAttribute("stroke-linejoin","round"),a.setAttribute("aria-hidden","true"),a.setAttribute("focusable","false");let s=document.createElement("path");s.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),a.appendChild(s),d.appendChild(a),p.appendChild(d);let u=document.createComment("if");p.appendChild(u),o.push(f(u));let b=document.createElement("span");b.setAttribute("tabindex","0"),b.setAttribute("class","sentinel"),b.setAttribute("aria-hidden","true"),T(t,"sentinelEnd",b),p.appendChild(b),m.appendChild(p);let h=document.createComment("if");if(m.appendChild(h),o.push(c(h)),i!=null){let v=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(v){let g=new Proxy(t,{get(x,A){return x[A]},set(x,A,w){let E=x[A];if(E!=null&&typeof E.set==="function"){if(typeof w==="function")E.set(()=>w);else E.set(w);return!0}return!1}});if(v.mount){let x=v.mount(g,r);if(typeof x==="function")o.push(x)}if(v.unmount)o.push(()=>v.unmount(g))}}return t}});var ki=0,Fn=P({mount(n,e){let r=n,o=++ki,t=`sui-drawer-header-${o}`,i=`sui-drawer-body-${o}`,f=r.dialogEl(),c=r.closeBtn(),m=r.sentinelStart(),p=r.sentinelEnd(),l=r.headerEl(),d=r.bodyEl();if(!f)return;if(l)l.id=t;if(d)d.id=i;f.setAttribute("aria-labelledby",t),f.setAttribute("aria-describedby",i);let a=!1,s=new Ft;function u(){let C=zt(f).filter((O)=>!O.classList.contains("sentinel"));return[...zt(e),...C]}function b(){let C=e.querySelector("[autofocus]")??f.querySelector("[autofocus]");if(C){C.focus();return}let N=u(),O=N.find((S)=>S!==c)??N[0];if(O)O.focus();else f.focus()}function h(){s.save();try{f.showModal()}catch{}if(!a)Hn(),a=!0;queueMicrotask(()=>b()),e.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function v(){if(f.open)f.close()}let g=()=>{if(r.open())r.open.set(!1);if(a)Lt(),a=!1;s.restore(),e.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},x=(C)=>{if(!r.dismissible())return;if(C.target===f)r.open.set(!1)},A=()=>{e.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),r.open.set(!1)},w=()=>{let C=u();if(C.length)C[C.length-1].focus()},E=()=>{let C=u();if(C.length)C[0].focus()};f.addEventListener("close",g),f.addEventListener("click",x),c?.addEventListener("click",A),m?.addEventListener("focus",w),p?.addEventListener("focus",E);let j=D(()=>{let C=r.open();if(e.setAttribute("data-state",C?"open":"closed"),C&&!f.open)h();else if(!C&&f.open)v()});return e.show=()=>r.open.set(!0),e.close=()=>r.open.set(!1),()=>{if(a)Lt();f.removeEventListener("close",g),f.removeEventListener("click",x),c?.removeEventListener("click",A),m?.removeEventListener("focus",w),p?.removeEventListener("focus",E),j()}}});var rm=z("sui-drawer",{props:{open:y.bool({default:!1,reflect:!0}),side:y.union(["right","left"],{default:"right",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),dismissible:y.bool({default:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.open=e.open,t.side=e.side,t.size=e.size,t.dismissible=e.dismissible;let i=null;if(Fn&&typeof Fn.setup==="function"){if(n.scope=t,i=Fn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("dialog");c.setAttribute("part","dialog"),c.setAttribute("role","dialog"),c.setAttribute("aria-modal","true"),T(t,"dialogEl",c);let m=document.createElement("span");m.setAttribute("tabindex","0"),m.setAttribute("class","sentinel"),m.setAttribute("aria-hidden","true"),T(t,"sentinelStart",m),c.appendChild(m);let p=document.createElement("div");p.setAttribute("class","header"),p.setAttribute("part","header"),T(t,"headerEl",p);let l=document.createElement("div");l.setAttribute("class","header-content");let d=document.createElement("div");d.setAttribute("class","title");let a=document.createElement("slot");a.setAttribute("name","title"),d.appendChild(a),l.appendChild(d);let s=document.createElement("div");s.setAttribute("class","description");let u=document.createElement("slot");u.setAttribute("name","description"),s.appendChild(u),l.appendChild(s),p.appendChild(l);let b=document.createElement("button");b.setAttribute("class","close"),b.setAttribute("part","close"),b.setAttribute("type","button"),b.setAttribute("aria-label","Close drawer"),T(t,"closeBtn",b);let h=document.createElement("svg");h.setAttribute("viewbox","0 0 16 16"),h.setAttribute("fill","none"),h.setAttribute("stroke","currentColor"),h.setAttribute("stroke-width","1.75"),h.setAttribute("stroke-linecap","round"),h.setAttribute("stroke-linejoin","round"),h.setAttribute("aria-hidden","true"),h.setAttribute("focusable","false");let v=document.createElement("path");v.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),h.appendChild(v),b.appendChild(h),p.appendChild(b),c.appendChild(p);let g=document.createElement("div");g.setAttribute("class","body"),g.setAttribute("part","body"),T(t,"bodyEl",g);let x=document.createElement("slot");g.appendChild(x),c.appendChild(g);let A=document.createElement("div");A.setAttribute("class","footer"),A.setAttribute("part","footer");let w=document.createElement("slot");w.setAttribute("name","footer"),A.appendChild(w),c.appendChild(A);let E=document.createElement("span");if(E.setAttribute("tabindex","0"),E.setAttribute("class","sentinel"),E.setAttribute("aria-hidden","true"),T(t,"sentinelEnd",E),c.appendChild(E),f.appendChild(c),i!=null){let j=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(j){let C=new Proxy(t,{get(N,O){return N[O]},set(N,O,S){let H=N[O];if(H!=null&&typeof H.set==="function"){if(typeof S==="function")H.set(()=>S);else H.set(S);return!0}return!1}});if(j.mount){let N=j.mount(C,r);if(typeof N==="function")o.push(N)}if(j.unmount)o.push(()=>j.unmount(C))}}return t}});var Ci=0,Gn=P({mount(n,e){let r=n,o=`sui-dropdown-menu-panel-${++Ci}`,t=r.triggerWrap(),i=r.triggerSlot(),f=r.panelEl();if(!f)return;f.id=o;let c="",m=null;function p(E){if(m)clearTimeout(m);c+=E.toLowerCase(),m=setTimeout(()=>{c=""},350);let j=l(),C=j.indexOf(document.activeElement);for(let N=0;N<j.length;N++){let O=(C+1+N)%j.length;if((j[O].textContent??"").trim().toLowerCase().startsWith(c))return O}return-1}function l(){return Array.from(e.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((E)=>!E.hasAttribute("disabled"))}function d(E){let j=l();if(!j.length)return;let C=(E%j.length+j.length)%j.length;for(let N of j)N.setAttribute("tabindex","-1");j[C].setAttribute("tabindex","0"),j[C].focus()}function a(){return l().indexOf(document.activeElement)}function s(){l().forEach((E,j)=>{if(!E.hasAttribute("role"))E.setAttribute("role","menuitem");if(!E.hasAttribute("tabindex"))E.setAttribute("tabindex",j===0?"0":"-1")})}function u(){let E=i?.assignedElements({flatten:!0})??[];for(let j of E)j.setAttribute("aria-haspopup","menu"),j.setAttribute("aria-controls",o),j.setAttribute("aria-expanded",r.open()?"true":"false")}function b(){let E=i?.assignedElements({flatten:!0})?.[0];if(E&&f){let j=E.getBoundingClientRect();f.style.top=`${j.bottom+4}px`,f.style.left=`${j.left}px`}f?.showPopover?.()}function h(){f?.hidePopover?.()}let v=()=>{if(r.open())h();else b()},g=(E)=>{let C=E.newState==="open";if(C!==r.open())r.open.set(C);if(e.setAttribute("data-state",C?"open":"closed"),u(),C)queueMicrotask(()=>{s(),d(0)})},x=(E)=>{let j=E.target.closest("[data-value]");if(!j)return;e.emit("sui-select",{value:j.dataset.value??""}),h()},A=(E)=>{if(!r.open())return;let j=l();if(!j.length)return;let C=a();if(E.key==="ArrowDown")E.preventDefault(),d(C+1);else if(E.key==="ArrowUp")E.preventDefault(),d(C-1);else if(E.key==="Home")E.preventDefault(),d(0);else if(E.key==="End")E.preventDefault(),d(j.length-1);else if(E.key==="Escape")E.preventDefault(),h();else if(E.key==="Tab")h();else if((E.key==="Enter"||E.key===" ")&&C>=0)E.preventDefault(),j[C].click();else if(E.key.length===1&&!E.ctrlKey&&!E.metaKey){let N=p(E.key);if(N>=0)E.preventDefault(),d(N)}};if(t)Q(t,"click",v);if(Q(f,"toggle",g),Q(f,"click",x),Q(e,"keydown",A),i)Q(i,"slotchange",u);Q(f,"slotchange",s);let w=D(()=>{let E=r.open();e.setAttribute("data-state",E?"open":"closed"),u()});return()=>{if(m)clearTimeout(m);w()}}});var lm=z("sui-dropdown-menu",{props:{open:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.open=e.open;let i=null;if(Gn&&typeof Gn.setup==="function"){if(n.scope=t,i=Gn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("span");c.setAttribute("class","trigger-wrap"),T(t,"triggerWrap",c);let m=document.createElement("slot");m.setAttribute("name","trigger"),T(t,"triggerSlot",m),c.appendChild(m),f.appendChild(c);let p=document.createElement("div");p.setAttribute("class","panel"),p.setAttribute("part","panel"),p.setAttribute("popover","auto"),p.setAttribute("role","menu"),T(t,"panelEl",p);let l=document.createElement("slot");if(T(t,"itemSlot",l),p.appendChild(l),f.appendChild(p),i!=null){let d=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(d){let a=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(d.mount){let s=d.mount(a,r);if(typeof s==="function")o.push(s)}if(d.unmount)o.push(()=>d.unmount(a))}}return t}});var ji=0,Un=P({mount(n,e){let r=n,o=`sui-context-menu-panel-${++ji}`,t=r.triggerSlot(),i=r.panelEl();if(!i)return;i.id=o;let f="",c=null;function m(w){if(c)clearTimeout(c);f+=w.toLowerCase(),c=setTimeout(()=>{f=""},350);let E=p(),j=E.indexOf(document.activeElement);for(let C=0;C<E.length;C++){let N=(j+1+C)%E.length;if((E[N].textContent??"").trim().toLowerCase().startsWith(f))return N}return-1}function p(){return Array.from(e.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((w)=>!w.hasAttribute("disabled"))}function l(w){let E=p();if(!E.length)return;let j=(w%E.length+E.length)%E.length;for(let C of E)C.setAttribute("tabindex","-1");E[j].setAttribute("tabindex","0"),E[j].focus()}function d(){return p().indexOf(document.activeElement)}function a(){p().forEach((w,E)=>{if(!w.hasAttribute("role"))w.setAttribute("role","menuitem");if(!w.hasAttribute("tabindex"))w.setAttribute("tabindex",E===0?"0":"-1")})}function s(w,E){if(i)i.style.top=`${E}px`,i.style.left=`${w}px`;i?.showPopover?.()}function u(){i?.hidePopover?.()}e.expose({show:s,close:u});let b=(w)=>{w.preventDefault(),s(w.clientX,w.clientY)},h=(w)=>{let j=w.newState==="open";if(j!==r.open())r.open.set(j);if(e.setAttribute("data-state",j?"open":"closed"),j)e.emit("sui-open"),queueMicrotask(()=>{a(),l(0)});else e.emit("sui-close")},v=(w)=>{let E=w.target.closest("[data-value]");if(!E)return;e.emit("sui-select",{value:E.dataset.value??""}),u()},g=(w)=>{if(!r.open())return;let E=p();if(!E.length)return;let j=d();if(w.key==="ArrowDown")w.preventDefault(),l(j+1);else if(w.key==="ArrowUp")w.preventDefault(),l(j-1);else if(w.key==="Home")w.preventDefault(),l(0);else if(w.key==="End")w.preventDefault(),l(E.length-1);else if(w.key==="Escape")w.preventDefault(),u();else if(w.key==="Tab")u();else if((w.key==="Enter"||w.key===" ")&&j>=0)w.preventDefault(),E[j].click();else if(w.key.length===1&&!w.ctrlKey&&!w.metaKey){let C=m(w.key);if(C>=0)w.preventDefault(),l(C)}},x=()=>{let w=t?.assignedElements({flatten:!0})??[];for(let E of w)E.addEventListener("contextmenu",b)};if(t)Q(t,"slotchange",x);x(),Q(i,"toggle",h),Q(i,"click",v),Q(e,"keydown",g);let A=D(()=>{e.setAttribute("data-state",r.open()?"open":"closed")});return()=>{let w=t?.assignedElements({flatten:!0})??[];for(let E of w)E.removeEventListener("contextmenu",b);if(c)clearTimeout(c);A()}}});var bm=z("sui-context-menu",{props:{open:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.open=e.open;let i=null;if(Un&&typeof Un.setup==="function"){if(n.scope=t,i=Un.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");c.setAttribute("name","trigger"),T(t,"triggerSlot",c),f.appendChild(c);let m=document.createElement("div");m.setAttribute("class","panel"),m.setAttribute("part","panel"),m.setAttribute("popover","auto"),m.setAttribute("role","menu"),m.setAttribute("aria-label","Context menu"),T(t,"panelEl",m);let p=document.createElement("slot");if(T(t,"itemSlot",p),m.appendChild(p),f.appendChild(m),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var zi=0,Xn=8;function Li(n,e,r){let o=n.getBoundingClientRect();switch(e.style.top="",e.style.left="",e.style.right="",e.style.bottom="",r){case"top":e.style.bottom=`${window.innerHeight-o.top+Xn}px`,e.style.left=`${o.left+o.width/2}px`,e.style.transform="translateX(-50%)";break;case"left":e.style.right=`${window.innerWidth-o.left+Xn}px`,e.style.top=`${o.top}px`;break;case"right":e.style.left=`${o.right+Xn}px`,e.style.top=`${o.top}px`;break;case"bottom":default:e.style.top=`${o.bottom+Xn}px`,e.style.left=`${o.left+o.width/2}px`,e.style.transform="translateX(-50%)"}}var qn=P({mount(n,e){let r=n,o=`sui-hover-card-${++zi}`,t=r.triggerSlot(),i=r.panelEl();if(!i)return;i.id=o;let f=null,c=null;function m(){if(f)clearTimeout(f),f=null;if(c)clearTimeout(c),c=null}function p(){let g=t?.assignedElements({flatten:!0})??[];for(let x of g)x.setAttribute("aria-describedby",o)}function l(){m();let g=t?.assignedElements({flatten:!0})?.[0];if(g&&i)Li(g,i,r.placement()||"bottom");i?.showPopover?.()}function d(){m(),i?.hidePopover?.()}e.show=l,e.hide=d;let a=(g)=>{let A=g.newState==="open";if(A!==r.open())r.open.set(A);e.setAttribute("data-state",A?"open":"closed"),e.dispatchEvent(new CustomEvent(A?"sui-open":"sui-close",{bubbles:!0,composed:!0}))},s=()=>{m(),f=setTimeout(()=>l(),r.showDelay()??300)},u=()=>{m(),c=setTimeout(()=>d(),r.hideDelay()??100)},b=()=>{m(),l()},h=()=>{m(),c=setTimeout(()=>d(),r.hideDelay()??100)};i.addEventListener("toggle",a),t?.addEventListener("slotchange",p),p(),e.addEventListener("pointerenter",s),e.addEventListener("pointerleave",u),e.addEventListener("focusin",b),e.addEventListener("focusout",h);let v=D(()=>{e.setAttribute("data-state",r.open()?"open":"closed")});return()=>{m(),i.removeEventListener("toggle",a),t?.removeEventListener("slotchange",p),e.removeEventListener("pointerenter",s),e.removeEventListener("pointerleave",u),e.removeEventListener("focusin",b),e.removeEventListener("focusout",h),v()}}});var wm=z("sui-hover-card",{props:{open:y.bool({default:!1,reflect:!0}),placement:y.union(["top","bottom","left","right"],{default:"bottom",reflect:!0}),"show-delay":y.number({default:300}),"hide-delay":y.number({default:100})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.open=e.open,t.placement=e.placement,t.showDelay=e.showDelay,t.hideDelay=e.hideDelay;let i=null;if(qn&&typeof qn.setup==="function"){if(n.scope=t,i=qn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");c.setAttribute("name","trigger"),T(t,"triggerSlot",c),f.appendChild(c);let m=document.createElement("div");m.setAttribute("class","panel"),m.setAttribute("part","panel"),m.setAttribute("popover","auto"),T(t,"panelEl",m);let p=document.createElement("slot");if(m.appendChild(p),f.appendChild(m),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var Rn=P({mount(n,e){let r=n,o=()=>Array.from(e.querySelectorAll(":scope > sui-accordion-item")),t=()=>o().filter((l)=>!l.hasAttribute("disabled")),i=()=>{let l=r.value();return new Set(l?l.split(",").filter(Boolean):[])},f=()=>{let l=i();for(let d of o()){let a=d.getAttribute("value")||"";if(l.has(a))d.setAttribute("open","");else d.removeAttribute("open")}},c=D(()=>{r.value(),f()}),m=(l)=>{let d=l;d.stopPropagation();let a=d.detail?.value,s=i();if(r.type()==="single")r.value.set(s.has(a)?"":a);else{if(s.has(a))s.delete(a);else s.add(a);r.value.set(Array.from(s).join(","))}e.dispatchEvent(new CustomEvent("sui-change",{detail:{value:r.value()},bubbles:!0,composed:!0}))},p=(l)=>{let d=l;d.stopPropagation();let a=t();if(!a.length)return;let s=d.target?.closest("sui-accordion-item"),u=s?a.indexOf(s):-1,b;switch(d.detail?.direction){case"next":b=a[(u+1)%a.length];break;case"prev":b=a[(u-1+a.length)%a.length];break;case"first":b=a[0];break;case"last":b=a[a.length-1];break}if(b)b.shadowRoot?.querySelector("button")?.focus()};return e.addEventListener("sui-accordion-item-toggle",m),e.addEventListener("sui-accordion-nav",p),()=>{c(),e.removeEventListener("sui-accordion-item-toggle",m),e.removeEventListener("sui-accordion-nav",p)}}});var zm=z("sui-accordion",{props:{type:y.string({default:"single",reflect:!0}),value:y.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host { display: block; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.type=e.type,t.value=e.value;let i=null;if(Rn&&typeof Rn.setup==="function"){if(n.scope=t,i=Rn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");if(f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var Di=0,Jn=P({mount(n,e){let r=n,o=++Di,t=`sui-acc-trigger-${o}`,i=`sui-acc-content-${o}`,f=D(()=>{let d=r.triggerBtn(),a=r.contentWrap();if(d)d.id=t,d.setAttribute("aria-controls",i);if(a)a.id=i,a.setAttribute("aria-labelledby",t)}),c=D(()=>{let d=r.triggerBtn();if(d)d.setAttribute("aria-expanded",r.open()?"true":"false")}),m=()=>{if(r.disabled())return;e.dispatchEvent(new CustomEvent("sui-accordion-item-toggle",{detail:{value:r.value()},bubbles:!0,composed:!0}))},p=(d)=>{if(d.key==="ArrowDown"||d.key==="ArrowUp")d.preventDefault(),e.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:d.key==="ArrowDown"?"next":"prev"},bubbles:!0,composed:!0}));else if(d.key==="Home"||d.key==="End")d.preventDefault(),e.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:d.key==="Home"?"first":"last"},bubbles:!0,composed:!0}))},l=D(()=>{let d=r.triggerBtn();if(!d)return;return d.addEventListener("click",m),d.addEventListener("keydown",p),()=>{d.removeEventListener("click",m),d.removeEventListener("keydown",p)}});return()=>{f(),c(),l()}}});var Mm=z("sui-accordion-item",{props:{value:y.string({default:"",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),open:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.disabled=e.disabled,t.open=e.open;let i=null;if(Jn&&typeof Jn.setup==="function"){if(n.scope=t,i=Jn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("button");c.setAttribute("class","trigger"),c.setAttribute("part","trigger"),T(t,"triggerBtn",c),o.push(k(c,"disabled",()=>t.disabled()));let m=document.createElement("span");m.setAttribute("class","trigger-label");let p=document.createElement("slot");p.setAttribute("name","trigger"),m.appendChild(p),c.appendChild(m);let l=document.createElement("svg");l.setAttribute("class","chevron"),l.setAttribute("part","chevron"),l.setAttribute("viewbox","0 0 16 16"),l.setAttribute("fill","none"),l.setAttribute("stroke","currentColor"),l.setAttribute("stroke-width","1.75"),l.setAttribute("stroke-linecap","round"),l.setAttribute("stroke-linejoin","round"),l.setAttribute("aria-hidden","true");let d=document.createElement("path");d.setAttribute("d","M4 6l4 4 4-4"),l.appendChild(d),c.appendChild(l),f.appendChild(c);let a=document.createElement("div");a.setAttribute("class","content-wrap"),a.setAttribute("part","content"),T(t,"contentWrap",a),a.setAttribute("role","region");let s=document.createElement("div");s.setAttribute("class","content-inner");let u=document.createElement("div");u.setAttribute("class","content-body");let b=document.createElement("slot");if(u.appendChild(b),s.appendChild(u),a.appendChild(s),f.appendChild(a),i!=null){let h=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(g,x){return g[x]},set(g,x,A){let w=g[x];if(w!=null&&typeof w.set==="function"){if(typeof A==="function")w.set(()=>A);else w.set(A);return!0}return!1}});if(h.mount){let g=h.mount(v,r);if(typeof g==="function")o.push(g)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var to=`
sui-list-tile[variant='row'] + sui-list-tile[variant='row'] {
  border-top: 1px solid var(--border-subtle);
}
sui-list-tile[variant='row'][selected] + sui-list-tile[variant='row'],
sui-list-tile[variant='row'] + sui-list-tile[variant='row'][selected] {
  border-top: none;
}
`,no=new WeakSet;function Oi(n){if(no.has(n))return;no.add(n);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let r=new CSSStyleSheet;r.replaceSync(to),n.adoptedStyleSheets=[...n.adoptedStyleSheets,r];return}}catch{}let e=n.createElement("style");e.setAttribute("data-sui-list-tile-dividers",""),e.textContent=to,n.head.appendChild(e)}var Wn=P({mount(n,e){let r=n;Oi(e.ownerDocument);let o=()=>{if(r.navigable())return"link";if(r.interactive())return"button";return"listitem"},t=D(()=>{r.navigable(),r.interactive(),e.setAttribute("role",o())}),i=D(()=>{e.setAttribute("aria-selected",r.selected()?"true":"false")}),f=D(()=>{e.setAttribute("aria-disabled",r.disabled()?"true":"false")}),c=D(()=>{let a=r.tileEl();if(!a)return;let s=r.interactive(),u=r.disabled();if(s&&!u)a.setAttribute("tabindex","0");else a.removeAttribute("tabindex")}),m=()=>{if(r.disabled()||!r.interactive())return!1;let a=new CustomEvent("sui-select",{detail:{},bubbles:!0,composed:!0,cancelable:!0});return e.dispatchEvent(a),a.defaultPrevented},p=(a)=>{if(!r.interactive())return;if(m()&&r.navigable())a.preventDefault()},l=(a)=>{if(!r.interactive()||r.disabled())return;if(r.navigable()&&a.key==="Enter")return;if(a.key==="Enter"||a.key===" ")a.preventDefault(),m()},d=D(()=>{let a=r.tileEl();if(!a)return;return a.addEventListener("click",p),a.addEventListener("keydown",l),()=>{a.removeEventListener("click",p),a.removeEventListener("keydown",l)}});return()=>{t(),i(),f(),c(),d()}}});var Gm=z("sui-list-tile",{props:{variant:y.string({default:"row",reflect:!0}),interactive:y.bool({default:!1,reflect:!0}),navigable:y.string({default:"",reflect:!0}),selected:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),density:y.string({default:"normal",reflect:!0}),tone:y.string({default:"default",reflect:!0}),target:y.string({default:"",reflect:!0}),rel:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.variant=e.variant,t.interactive=e.interactive,t.navigable=e.navigable,t.selected=e.selected,t.disabled=e.disabled,t.density=e.density,t.tone=e.tone,t.target=e.target,t.rel=e.rel;let i=null;if(Wn&&typeof Wn.setup==="function"){if(n.scope=t,i=Wn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(p)=>B({anchor:p,test:()=>{if(t.navigable())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),d=[],a=document.createElement("a");a.setAttribute("class","tile"),a.setAttribute("part","button"),T(t,"tileEl",a),d.push(k(a,"href",()=>t.navigable())),d.push(k(a,"target",()=>t.target()||null)),d.push(k(a,"rel",()=>t.rel()||null));let s=document.createElement("div");s.setAttribute("class","row"),s.setAttribute("part","row");let u=document.createElement("slot");u.setAttribute("name","leading"),s.appendChild(u);let b=document.createElement("span");b.setAttribute("class","primary"),b.setAttribute("part","primary");let h=document.createElement("slot");b.appendChild(h),s.appendChild(b);let v=document.createElement("span");v.setAttribute("part","trailing");let g=document.createElement("slot");g.setAttribute("name","trailing"),v.appendChild(g),s.appendChild(v),a.appendChild(s);let x=document.createElement("div");x.setAttribute("class","secondary"),x.setAttribute("part","secondary");let A=document.createElement("slot");return A.setAttribute("name","meta"),x.appendChild(A),a.appendChild(x),l.appendChild(a),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let w of d)try{w()}catch(E){}}}},()=>{let l=document.createDocumentFragment(),d=[],a=document.createElement("div");a.setAttribute("class","tile"),a.setAttribute("part","button"),T(t,"tileEl",a);let s=document.createElement("div");s.setAttribute("class","row"),s.setAttribute("part","row");let u=document.createElement("slot");u.setAttribute("name","leading"),s.appendChild(u);let b=document.createElement("span");b.setAttribute("class","primary"),b.setAttribute("part","primary");let h=document.createElement("slot");b.appendChild(h),s.appendChild(b);let v=document.createElement("span");v.setAttribute("part","trailing");let g=document.createElement("slot");g.setAttribute("name","trailing"),v.appendChild(g),s.appendChild(v),a.appendChild(s);let x=document.createElement("div");x.setAttribute("class","secondary"),x.setAttribute("part","secondary");let A=document.createElement("slot");return A.setAttribute("name","meta"),x.appendChild(A),a.appendChild(x),l.appendChild(a),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let w of d)try{w()}catch(E){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createComment("if");if(c.appendChild(m),o.push(f(m)),i!=null){let p=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(d,a){return d[a]},set(d,a,s){let u=d[a];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(p.mount){let d=p.mount(l,r);if(typeof d==="function")o.push(d)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var Yn=P({scope:(n,e)=>{let r=M(null),o=M(!1);return{caughtError:r,hasFallbackSlot:o,reset:()=>{if(r.set(null),e instanceof HTMLElement)e.removeAttribute("data-error-message")}}},mount(n,e){let r=e.querySelectorAll(':scope > template[slot="fallback"]');for(let o of Array.from(r)){let t=document.createElement("div");t.setAttribute("slot","fallback"),t.appendChild(o.content.cloneNode(!0)),o.replaceWith(t)}return n.hasFallbackSlot.set(!!e.querySelector(':scope > [slot="fallback"]')),He(e,{catch(o,t){if(n.caughtError()!==null)return;let i=o instanceof Error?o:Error(String(o));e.setAttribute("data-error-message",i.message),n.caughtError.set(i),e.dispatchEvent(new CustomEvent("sui-error-boundary:caught",{bubbles:!0,composed:!0,detail:{error:i,source:t}}))},reset:n.reset}),e.reset=n.reset,()=>{Be(e),delete e.reset}}});var Wm=z("sui-error-boundary",{props:{"fallback-text":y.string({default:""})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.fallbackText=e.fallbackText;let i=null;if(Yn&&typeof Yn.setup==="function"){if(n.scope=t,i=Yn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(l)=>B({anchor:l,test:()=>{if(t.hasFallbackSlot())return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("slot");return s.setAttribute("name","fallback"),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let u of a)try{u()}catch(b){}}}},()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("div");s.setAttribute("class","fallback"),s.setAttribute("part","fallback"),s.setAttribute("role","alert");let u=document.createElement("div");u.setAttribute("class","message"),u.setAttribute("part","message");let b=document.createTextNode("");u.appendChild(b),a.push(F(b,()=>t.fallbackText()||"Something went wrong")),s.appendChild(u);let h=document.createElement("button");h.setAttribute("class","retry"),h.setAttribute("part","retry"),h.setAttribute("type","button"),a.push(K(h,"click",function(){let g=t.reset;return typeof g==="function"?g.bind(void 0):()=>{}}(),{modifiers:[]}));let v=document.createTextNode("Retry");return h.appendChild(v),s.appendChild(h),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let g of a)try{g()}catch(x){}}}}]}),c=(l)=>B({anchor:l,test:()=>{if(t.caughtError()===null)return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("slot");return d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let u of a)try{u()}catch(b){}}}},()=>{let d=document.createDocumentFragment(),a=[],s=document.createComment("if");return d.appendChild(s),a.push(f(s)),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let u of a)try{u()}catch(b){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createComment("if");if(m.appendChild(p),o.push(c(p)),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var Zn=P({mount(n,e){let r=n,o=[],t=null,i=()=>{let a=r.target()||"body",s=document.querySelector(a);if(!s)return console.warn(`[sui-portal] target "${a}" not found in document`),null;return s},f=(a)=>{let s=r.portalSlot(),u=s?s.assignedNodes():Array.from(e.childNodes);for(let b of u)if(a.appendChild(b),!o.includes(b))o.push(b)},c=(a)=>{for(let s of o)a.appendChild(s)},m=()=>{for(let a of o)if(a.parentNode)a.parentNode.removeChild(a);o.length=0},p=D(()=>{let a=i();if(!a){t=null;return}if(t===null)f(a);else if(t!==a)c(a);t=a}),l=r.portalSlot(),d=()=>{if(!t||!l)return;for(let a of l.assignedNodes())if(!o.includes(a))t.appendChild(a),o.push(a)};if(l)l.addEventListener("slotchange",d);return()=>{if(p(),l)l.removeEventListener("slotchange",d);m()}}});var Im=z("sui-portal",{props:{target:y.string({default:"body"})},shadow:!0,styles:`
  :host { display: none; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.target=e.target;let i=null;if(Zn&&typeof Zn.setup==="function"){if(n.scope=t,i=Zn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");if(T(t,"portalSlot",c),f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var Qn=P({mount(n,e){let r=n,o=!1,t=(m)=>{m.textContent=o?"Hide details":"Show details",m.setAttribute("aria-expanded",o?"true":"false")},i=D(()=>{let m=r.detailsSlot();if(!m)return;let p=()=>{if(m.assignedNodes({flatten:!0}).some((d)=>d.nodeType===Node.ELEMENT_NODE||d.nodeType===Node.TEXT_NODE&&(d.textContent??"").trim().length>0))e.setAttribute("data-has-details","");else{e.removeAttribute("data-has-details"),e.removeAttribute("data-details-open"),o=!1;let d=r.detailsToggleBtn();if(d)t(d)}};return m.addEventListener("slotchange",p),p(),()=>m.removeEventListener("slotchange",p)}),f=D(()=>{let m=r.detailsToggleBtn();if(!m)return;t(m);let p=()=>{if(o=!o,o)e.setAttribute("data-details-open","");else e.removeAttribute("data-details-open");t(m)};return m.addEventListener("click",p),()=>m.removeEventListener("click",p)}),c=D(()=>{let m=r.actionSlot();if(!m)return;let p=()=>{let l=m.assignedElements().find((d)=>d.getAttribute("slot")==="retry");if(l)requestAnimationFrame(()=>l.focus?.())};return m.addEventListener("slotchange",p),()=>m.removeEventListener("slotchange",p)});return()=>{i(),f(),c()}}});var rb=z("sui-error-state",{props:{title:y.string({default:""}),body:y.string({default:""}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:y.union(["default","inline"],{default:"default",reflect:!0}),icon:y.string({default:"alert-circle"})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.title=e.title,t.body=e.body,t.size=e.size,t.variant=e.variant,t.icon=e.icon;let i=null;if(Qn&&typeof Qn.setup==="function"){if(n.scope=t,i=Qn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","box"),c.setAttribute("part","box"),c.setAttribute("role","alert");let m=document.createElement("div");m.setAttribute("class","icon"),m.setAttribute("part","icon"),m.setAttribute("aria-hidden","true");let p=document.createElement("sui-icon");o.push(k(p,"name",()=>t.icon())),p.setAttribute("size","md"),m.appendChild(p),c.appendChild(m);let l=document.createElement("div");l.setAttribute("class","copy");let d=document.createElement("div");d.setAttribute("class","title"),d.setAttribute("part","title");let a=document.createElement("slot");a.setAttribute("name","title");let s=document.createTextNode("");a.appendChild(s),o.push(F(s,()=>t.title()||"Something went wrong")),d.appendChild(a),l.appendChild(d);let u=document.createElement("div");u.setAttribute("class","desc"),u.setAttribute("part","desc");let b=document.createElement("slot"),h=document.createTextNode("");b.appendChild(h),o.push(F(h,()=>t.body())),u.appendChild(b),l.appendChild(u);let v=document.createElement("button");v.setAttribute("class","details-toggle"),T(t,"detailsToggleBtn",v),v.setAttribute("type","button"),l.appendChild(v);let g=document.createElement("div");g.setAttribute("class","details"),T(t,"detailsPanel",g);let x=document.createElement("slot");x.setAttribute("name","details"),T(t,"detailsSlot",x),g.appendChild(x),l.appendChild(g),c.appendChild(l);let A=document.createElement("div");A.setAttribute("class","actions"),A.setAttribute("part","actions");let w=document.createElement("slot");if(w.setAttribute("name","action"),T(t,"actionSlot",w),A.appendChild(w),c.appendChild(A),f.appendChild(c),i!=null){let E=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(E){let j=new Proxy(t,{get(C,N){return C[N]},set(C,N,O){let S=C[N];if(S!=null&&typeof S.set==="function"){if(typeof O==="function")S.set(()=>O);else S.set(O);return!0}return!1}});if(E.mount){let C=E.mount(j,r);if(typeof C==="function")o.push(C)}if(E.unmount)o.push(()=>E.unmount(j))}}return t}});var Vn=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","progressbar");let o=D(()=>{let t=Math.max(1,Math.floor(r.total()||0)),i=Math.max(1,Math.min(t,Math.floor(r.current()||1))),f=r.label();if(e.setAttribute("aria-valuemin","1"),e.setAttribute("aria-valuemax",String(t)),e.setAttribute("aria-valuenow",String(i)),f)e.setAttribute("aria-label",f);else e.setAttribute("aria-label",`Step ${i} of ${t}`);let c=r.dotsEl();if(!c)return;c.textContent="";for(let m=1;m<=t;m++){let p=document.createElement("span");p.className="dot",p.setAttribute("part","dot"),p.setAttribute("data-state",m===i?"active":m<i?"done":"pending"),p.setAttribute("aria-hidden","true"),c.appendChild(p)}});return()=>{o()}}});var lb=z("sui-stepper",{props:{total:y.number({default:4,reflect:!0}),current:y.number({default:1,reflect:!0}),tone:y.string({default:"",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.total=e.total,t.current=e.current,t.tone=e.tone,t.size=e.size,t.label=e.label;let i=null;if(Vn&&typeof Vn.setup==="function"){if(n.scope=t,i=Vn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");if(c.setAttribute("class","dots"),T(t,"dotsEl",c),f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var Ti=new Set(["done","failed","pending","running"]);function Pi(n){return n.split(",").map((e)=>e.trim().toLowerCase()).filter((e)=>e.length>0).map((e)=>Ti.has(e)?e:"pending")}var Kn=P({scope:()=>({segments:M([]),ariaLabel:M(null)}),mount(n,e){let r=D(()=>{let o=Pi(n.values()||"");n.segments.set(o.map((t,i)=>({id:`s${i}`,state:t}))),n.ariaLabel.set(o.length===0?null:`${o.length} step outcome${o.length===1?"":"s"}`)});return()=>r()}});var bb=z("sui-step-segments",{props:{values:y.string({default:"",reflect:!0}),size:y.union(["sm","md"],{default:"sm",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.values=e.values,t.size=e.size;let i=null;if(Kn&&typeof Kn.setup==="function"){if(n.scope=t,i=Kn.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(l)=>R({anchor:l,items:()=>t.segments(),key:(d,a)=>{let s=Object.create(t);return s.seg=d,s.__i=a,((u)=>d.id)(s)},render:(d,a)=>{let s=Object.create(t);return s.seg=d,s.__i=a,s.__v=M(0),((u)=>{let b=document.createDocumentFragment(),h=[],v=document.createElement("span");return v.setAttribute("class","seg"),v.setAttribute("part","seg"),h.push(k(v,"data-state",()=>(u.__v(),u.seg.state))),b.appendChild(v),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let g of h)try{g()}catch(x){}},update:(g,x)=>{u.seg=g,u.__i=x,u.__v.set((A)=>A+1)}}})(s)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("span");m.setAttribute("class","row"),m.setAttribute("part","row"),o.push(k(m,"role",()=>t.segments().length>0?"img":null)),o.push(k(m,"aria-label",()=>t.ariaLabel()));let p=document.createComment("for");if(m.appendChild(p),o.push(f(p)),c.appendChild(m),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var In=P({mount(n,e){if(!e.hasAttribute("role"))e.setAttribute("role","list")}});var wb=z("sui-timeline",{shadow:!0,styles:`
  :host { display: block; position: relative; }
  .list { display: flex; flex-direction: column; gap: 4px; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null),i=null;if(In&&typeof In.setup==="function"){if(n.scope=t,i=In.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","list"),c.setAttribute("part","list");let m=document.createElement("slot");if(c.appendChild(m),f.appendChild(c),i!=null){let p=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(d,a){return d[a]},set(d,a,s){let u=d[a];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(p.mount){let d=p.mount(l,r);if(typeof d==="function")o.push(d)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var $n=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","listitem");let o=D(()=>{if(r.interactive()){if(e.setAttribute("role","button"),!e.hasAttribute("tabindex"))e.setAttribute("tabindex","0")}else e.setAttribute("role","listitem"),e.removeAttribute("tabindex")}),t=()=>{e.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},i=()=>{if(r.interactive())t()},f=(c)=>{if(!r.interactive())return;if(c.key==="Enter"||c.key===" ")c.preventDefault(),t()};return e.addEventListener("click",i),e.addEventListener("keydown",f),()=>{o(),e.removeEventListener("click",i),e.removeEventListener("keydown",f)}}});var zb=z("sui-timeline-item",{props:{tone:y.string({default:"default",reflect:!0}),at:y.string({default:"",reflect:!0}),interactive:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.tone=e.tone,t.at=e.at,t.interactive=e.interactive;let i=null;if($n&&typeof $n.setup==="function"){if(n.scope=t,i=$n.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(h)=>B({anchor:h,test:()=>{if(t.at())return 0;return-1},branches:[()=>{let v=document.createDocumentFragment(),g=[],x=document.createElement("span");x.setAttribute("class","at"),x.setAttribute("part","at");let A=document.createTextNode("");return x.appendChild(A),g.push(F(A,()=>t.at())),v.appendChild(x),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let w of g)try{w()}catch(E){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("div");m.setAttribute("class","gutter");let p=document.createElement("span");p.setAttribute("class","dot"),p.setAttribute("part","dot"),m.appendChild(p),c.appendChild(m);let l=document.createElement("div");l.setAttribute("class","body"),l.setAttribute("part","body");let d=document.createElement("div");d.setAttribute("class","head");let a=document.createElement("slot");a.setAttribute("name","title"),d.appendChild(a);let s=document.createComment("if");d.appendChild(s),o.push(f(s)),l.appendChild(d);let u=document.createElement("div");u.setAttribute("class","content");let b=document.createElement("slot");if(u.appendChild(b),l.appendChild(u),c.appendChild(l),i!=null){let h=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(g,x){return g[x]},set(g,x,A){let w=g[x];if(w!=null&&typeof w.set==="function"){if(typeof A==="function")w.set(()=>A);else w.set(A);return!0}return!1}});if(h.mount){let g=h.mount(v,r);if(typeof g==="function")o.push(g)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var Mi=["Waking daemon","Warming routes","Fetching state","Hydrating shell"];function eo(n){let e=n.trim();if(!e)return Mi;return e.split("|").map((r)=>r.trim()).filter(Boolean)}var _n=P({mount(n,e){let r=n,o=r.msgEl();if(!o)return;let t=0;(()=>{let c=eo(r.messages());o.textContent=c[t]??"",o.style.opacity="1"})();let f=setInterval(()=>{o.style.opacity="0",requestAnimationFrame(()=>{let c=eo(r.messages());t=(t+1)%c.length,o.textContent=c[t]??"",o.style.opacity="1"})},1400);return()=>clearInterval(f)}});var Mb=z("sui-loading",{props:{messages:y.string({default:""}),compact:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.messages=e.messages,t.compact=e.compact;let i=null;if(_n&&typeof _n.setup==="function"){if(n.scope=t,i=_n.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","stage"),c.setAttribute("part","stage"),c.setAttribute("aria-hidden","true");let m=document.createElement("div");m.setAttribute("class","rocket");let p=document.createElement("svg");p.setAttribute("viewbox","0 0 32 40"),p.setAttribute("fill","none"),p.setAttribute("xmlns","http://www.w3.org/2000/svg");let l=document.createElement("path");l.setAttribute("d","M16 2 C20 6 22 11 22 16 H10 C10 11 12 6 16 2 Z"),l.setAttribute("fill","currentColor"),p.appendChild(l);let d=document.createElement("rect");d.setAttribute("x","10"),d.setAttribute("y","16"),d.setAttribute("width","12"),d.setAttribute("height","14"),d.setAttribute("rx","2"),d.setAttribute("fill","currentColor"),d.setAttribute("opacity","0.9"),p.appendChild(d);let a=document.createElement("circle");a.setAttribute("cx","16"),a.setAttribute("cy","20"),a.setAttribute("r","2.5"),a.setAttribute("fill","var(--background, #fff)"),a.setAttribute("opacity","0.85"),p.appendChild(a);let s=document.createElement("path");s.setAttribute("d","M10 24 L4 32 L10 30 Z"),s.setAttribute("fill","currentColor"),s.setAttribute("opacity","0.75"),p.appendChild(s);let u=document.createElement("path");u.setAttribute("d","M22 24 L28 32 L22 30 Z"),u.setAttribute("fill","currentColor"),u.setAttribute("opacity","0.75"),p.appendChild(u);let b=document.createElement("rect");b.setAttribute("x","13"),b.setAttribute("y","29"),b.setAttribute("width","6"),b.setAttribute("height","2"),b.setAttribute("rx","0.5"),b.setAttribute("fill","var(--background, #000)"),b.setAttribute("opacity","0.5"),p.appendChild(b),m.appendChild(p),c.appendChild(m);let h=document.createElement("div");h.setAttribute("class","flame"),c.appendChild(h);let v=document.createElement("div");v.setAttribute("class","particle p1"),c.appendChild(v);let g=document.createElement("div");g.setAttribute("class","particle p2"),c.appendChild(g);let x=document.createElement("div");x.setAttribute("class","particle p3"),c.appendChild(x),f.appendChild(c);let A=document.createElement("div");if(A.setAttribute("class","message"),A.setAttribute("part","message"),T(t,"msgEl",A),f.appendChild(A),i!=null){let w=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(w){let E=new Proxy(t,{get(j,C){return j[C]},set(j,C,N){let O=j[C];if(O!=null&&typeof O.set==="function"){if(typeof N==="function")O.set(()=>N);else O.set(N);return!0}return!1}});if(w.mount){let j=w.mount(E,r);if(typeof j==="function")o.push(j)}if(w.unmount)o.push(()=>w.unmount(E))}}return t}});function ro(n){let e=document.getElementById("ship-view-root");if(!e)return null;let r=n.target;if(!r||!e.contains(r))return null;if(r.closest("sui-edit-overlay"))return null;return r}function oo(n){let e=n.tagName.toLowerCase(),r=n.id||null,o=e;if(r)o=`#${r}`;else if(n.classList.length>0)o=`${e}.${Array.from(n.classList).slice(0,2).join(".")}`;return{tag:e,selector:o,id:r}}var te=P({mount(n,e){let r=n,o=r.boxEl(),t=r.labelEl(),i=null;function f(u){if(!o||!t)return;if(!u){o.classList.remove("visible"),t.classList.remove("visible");return}o.style.cssText=`top:${u.top}px;left:${u.left}px;width:${u.width}px;height:${u.height}px`,o.classList.add("visible"),t.style.cssText=`top:${u.top}px;left:${u.left}px`,t.textContent=`<${u.tag}${u.id?`#${u.id}`:""}>`,t.classList.add("visible")}let c=(u)=>{if(i)return;i=requestAnimationFrame(()=>{i=null;let b=ro(u);if(!b){f(null);return}let h=b.getBoundingClientRect(),{tag:v,id:g}=oo(b);f({top:h.top,left:h.left,width:h.width,height:h.height,tag:v,id:g})})},m=()=>{f(null)},p=(u)=>{let b=ro(u);if(!b)return;u.preventDefault(),u.stopPropagation();let{tag:h,selector:v,id:g}=oo(b);(e.closest("sui-shell")??e.closest("ship-shell"))?.dispatchEvent(new CustomEvent("sui:chat:annotate",{detail:{tag:h,selector:v,id:g,preview:(b.textContent??"").trim().slice(0,60)},bubbles:!0})),window.ship?.chat?.open?.()};function l(){let u=document.getElementById("ship-view-root");if(!u)return;u.addEventListener("pointermove",c,!0),u.addEventListener("pointerleave",m,!0),u.addEventListener("click",p,!0)}function d(){let u=document.getElementById("ship-view-root");if(!u)return;u.removeEventListener("pointermove",c,!0),u.removeEventListener("pointerleave",m,!0),u.removeEventListener("click",p,!0)}function a(){let u=document.body.classList.contains("edit-mode");if(e.classList.toggle("active",u),u)l();else if(d(),f(null),i)cancelAnimationFrame(i),i=null}a();let s=new MutationObserver(()=>a());return s.observe(document.body,{attributes:!0,attributeFilter:["class"]}),()=>{if(s.disconnect(),d(),i)cancelAnimationFrame(i),i=null}}});var Gb=z("sui-edit-overlay",{shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null),i=null;if(te&&typeof te.setup==="function"){if(n.scope=t,i=te.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","box"),c.setAttribute("part","box"),T(t,"boxEl",c),f.appendChild(c);let m=document.createElement("div");m.setAttribute("class","label"),m.setAttribute("part","label"),T(t,"labelEl",m),f.appendChild(m);let p=document.createElement("div");p.setAttribute("class","hint"),p.setAttribute("part","hint");let l=document.createTextNode(`
  Edit mode — hover an element, click to annotate.
  `);p.appendChild(l);let d=document.createElement("kbd"),a=document.createTextNode("⌘⇧E");d.appendChild(a),p.appendChild(d);let s=document.createTextNode(` to exit.
`);if(p.appendChild(s),f.appendChild(p),i!=null){let u=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(u){let b=new Proxy(t,{get(h,v){return h[v]},set(h,v,g){let x=h[v];if(x!=null&&typeof x.set==="function"){if(typeof g==="function")x.set(()=>g);else x.set(g);return!0}return!1}});if(u.mount){let h=u.mount(b,r);if(typeof h==="function")o.push(h)}if(u.unmount)o.push(()=>u.unmount(b))}}return t}});function Ni(n=Date.now()){let e=n,r=[,,,,,,,,,,];for(let i=9;i>=0;i--)r[i]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[e%32],e=Math.floor(e/32);let o=Array(16),t=typeof globalThis<"u"?globalThis.crypto:void 0;if(t?.getRandomValues){let i=new Uint8Array(16);t.getRandomValues(i);for(let f=0;f<16;f++)o[f]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[i[f]%32]}else for(let i=0;i<16;i++)o[i]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[Math.floor(Math.random()*32)];return r.join("")+o.join("")}function io(n){let e={};if(n.viewId)e.view_id=n.viewId;if(n.selector)e.selector=n.selector;if(n.rect)e.rect=n.rect;return{id:Ni(),kind:n.kind,target:e,note:n.note,created_at:new Date().toISOString(),author_kind:"human"}}var Si="/api/feedback",ne=P({scope:(n,e)=>{let{viewId:r,mode:o,endpoint:t,staging:i}=n,f=M([]),c=M(null),m=M(null),p=M(""),l={},d=Z(()=>{let w=m();if(!w)return!1;return Math.abs(w.curX-w.startX)>4||Math.abs(w.curY-w.startY)>4}),a=Z(()=>{let w=m();if(!w)return"";let E=Math.min(w.startX,w.curX),j=Math.min(w.startY,w.curY),C=Math.abs(w.curX-w.startX),N=Math.abs(w.curY-w.startY);return`left:${E}px;top:${j}px;width:${C}px;height:${N}px`}),s=Z(()=>{let w=c();if(!w)return"";let E=Math.min(w.x+12,1e4),j=Math.min(w.y+12,1e4);return`left:${E}px;top:${j}px`}),u=Z(()=>f().map((w,E)=>({id:w.id,kind:w.kind,index:E,rect:w.kind==="region"&&w.target.rect?w.target.rect.join(","):"",pos:l[w.id]??{x:0,y:0}})));async function b(w){let E=t()||Si;try{let j=await fetch(E,{method:"POST",credentials:"include",headers:{"Content-Type":"application/json"},body:JSON.stringify(w)});if(!j.ok){e.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:`http_${j.status}`,id:w.id},bubbles:!0,composed:!0}));return}f.set(f().filter((C)=>C.id!==w.id)),e.dispatchEvent(new CustomEvent("sui-annotation-sent",{detail:{id:w.id,status:j.status},bubbles:!0,composed:!0}))}catch(j){e.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:"network_error",message:String(j),id:w.id},bubbles:!0,composed:!0}))}}function h(w){let E=c();if(!E)return;let j=w.trim();if(!j){c.set(null),p.set("");return}let C=io({viewId:r()||void 0,kind:E.kind,selector:E.kind==="point"?E.selector:void 0,rect:E.kind==="region"?E.rect:void 0,note:j});if(l[C.id]={x:E.x,y:E.y},f.set([...f(),C]),e.dispatchEvent(new CustomEvent("sui-annotation-staged",{detail:C,bubbles:!0,composed:!0})),c.set(null),p.set(""),!i())b(C)}let v=()=>{h(p())},g=()=>{c.set(null),p.set("")},x=(w)=>{if(w.key==="Escape")g();else if(w.key==="Enter"&&(w.metaKey||w.ctrlKey)){let E=w.currentTarget;h(E.value)}},A=J(null);return{staged:f,pending:c,drag:m,noteText:p,dragVisible:d,dragRectStyle:a,popoverStyle:s,markers:u,overlayEl:A,confirmNote:h,sendNote:v,cancelPending:g,onNoteKeydown:x,sendOne:b}},mount(n,e){let r=n,o=null,t=null,i=(l)=>{let d=r.overlayEl();if(!d)return{x:0,y:0};let a=d.getBoundingClientRect();return{x:l.clientX-a.left,y:l.clientY-a.top}},f=(l)=>{if(r.mode()==="off")return;let a=l.target;if(a.closest("[data-annotation-popover]")||a.closest("sui-annotation-marker"))return;if(r.pending()){r.pending.set(null),r.noteText.set("");return}l.preventDefault();let s=i(l);o={startX:s.x,startY:s.y,curX:s.x,curY:s.y},r.drag.set(o),l.target.setPointerCapture?.(l.pointerId)},c=(l)=>{if(!o)return;let d=i(l);if(o={...o,curX:d.x,curY:d.y},t!==null)return;t=requestAnimationFrame(()=>{if(t=null,o)r.drag.set({...o})})},m=(l)=>{if(!o)return;let d=i(l),a=Math.abs(d.x-o.startX),s=Math.abs(d.y-o.startY),u=r.mode(),b=null;if(u==="point"||u==="region"&&a<8&&s<8)b={kind:"point",x:d.x,y:d.y,selector:"body"};else if(u==="region"){let h=Math.min(o.startX,d.x),v=Math.min(o.startY,d.y),g=[Math.round(h),Math.round(v),Math.round(a),Math.round(s)];b={kind:"region",x:h+a/2,y:v+s/2,rect:g}}if(o=null,r.drag.set(null),r.noteText.set(""),b)r.pending.set(b)},p=r.overlayEl();if(p)p.addEventListener("pointerdown",f),p.addEventListener("pointermove",c),p.addEventListener("pointerup",m);return Object.defineProperty(e,"staged",{configurable:!0,get:()=>r.staged()}),e.send=async()=>{for(let l of[...r.staged()])await r.sendOne(l)},e.clear=()=>{r.staged.set([])},()=>{if(t!==null)cancelAnimationFrame(t),t=null;if(p)p.removeEventListener("pointerdown",f),p.removeEventListener("pointermove",c),p.removeEventListener("pointerup",m)}}});var Zb=z("sui-annotation-layer",{props:{"view-id":y.string({default:""}),mode:y.union(["off","point","region"],{default:"off",reflect:!0}),endpoint:y.string({default:"/api/feedback"}),staging:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.viewId=e.viewId,t.mode=e.mode,t.endpoint=e.endpoint,t.staging=e.staging;let i=null;if(ne&&typeof ne.setup==="function"){if(n.scope=t,i=ne.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(h)=>B({anchor:h,test:()=>{if(t.dragVisible())return 0;return-1},branches:[()=>{let v=document.createDocumentFragment(),g=[],x=document.createElement("div");return x.setAttribute("class","drag-rect"),g.push(k(x,"style",()=>t.dragRectStyle())),v.appendChild(x),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let A of g)try{A()}catch(w){}}}}]}),c=(h)=>R({anchor:h,items:()=>t.markers(),key:(v,g)=>{let x=Object.create(t);return x.m=v,x.__i=g,((A)=>v.id)(x)},render:(v,g)=>{let x=Object.create(t);return x.m=v,x.__i=g,x.__v=M(0),((A)=>{let w=document.createDocumentFragment(),E=[],j=document.createElement("sui-annotation-marker");return E.push(tt(j,"kind",()=>(A.__v(),A.m.kind))),E.push(tt(j,"index",()=>(A.__v(),A.m.index))),E.push(tt(j,"rect",()=>(A.__v(),A.m.rect))),E.push(tt(j,"x",()=>(A.__v(),A.m.pos.x))),E.push(tt(j,"y",()=>(A.__v(),A.m.pos.y))),w.appendChild(j),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of E)try{C()}catch(N){}},update:(C,N)=>{A.m=C,A.__i=N,A.__v.set((O)=>O+1)}}})(x)}}),m=(h)=>B({anchor:h,test:()=>{if(t.pending())return 0;return-1},branches:[()=>{let v=document.createDocumentFragment(),g=[],x=document.createElement("div");x.setAttribute("class","popover"),x.setAttribute("data-annotation-popover",""),g.push(k(x,"style",()=>t.popoverStyle())),g.push(K(x,"pointerdown",function(){let O=(S)=>S.stopPropagation();return typeof O==="function"?O.bind(void 0):()=>{}}(),{modifiers:[]}));let A=document.createElement("textarea");A.setAttribute("part","note-input"),A.setAttribute("placeholder","Add a note…"),g.push(mt(A,t.noteText)),g.push(K(A,"keydown",function(){let O=t.onNoteKeydown;return typeof O==="function"?O.bind(void 0):()=>{}}(),{modifiers:[]})),x.appendChild(A);let w=document.createElement("div");w.setAttribute("class","popover-actions");let E=document.createElement("button");E.setAttribute("type","button"),g.push(K(E,"click",function(){let O=t.cancelPending;return typeof O==="function"?O.bind(void 0):()=>{}}(),{modifiers:[]}));let j=document.createTextNode("Cancel");E.appendChild(j),w.appendChild(E);let C=document.createElement("button");C.setAttribute("type","button"),C.setAttribute("data-primary",""),g.push(K(C,"click",function(){let O=t.sendNote;return typeof O==="function"?O.bind(void 0):()=>{}}(),{modifiers:[]}));let N=document.createTextNode("Send");return C.appendChild(N),w.appendChild(C),x.appendChild(w),v.appendChild(x),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let O of g)try{O()}catch(S){}}}}]}),p=r.shadowRoot??r;L(p);let l=document.createElement("div");l.setAttribute("class","surface"),l.setAttribute("part","surface");let d=document.createElement("slot");l.appendChild(d);let a=document.createElement("div");a.setAttribute("class","overlay"),a.setAttribute("part","overlay"),T(t,"overlayEl",a);let s=document.createComment("if");a.appendChild(s),o.push(f(s));let u=document.createComment("for");a.appendChild(u),o.push(c(u));let b=document.createComment("if");if(a.appendChild(b),o.push(m(b)),l.appendChild(a),p.appendChild(l),i!=null){let h=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(h){let v=new Proxy(t,{get(g,x){return g[x]},set(g,x,A){let w=g[x];if(w!=null&&typeof w.set==="function"){if(typeof A==="function")w.set(()=>A);else w.set(A);return!0}return!1}});if(h.mount){let g=h.mount(v,r);if(typeof g==="function")o.push(g)}if(h.unmount)o.push(()=>h.unmount(v))}}return t}});var ee=P({mount(n,e){let r=n;return D(()=>{let t=r.pinEl(),i=r.boxEl(),f=r.x(),c=r.y(),m=r.index(),p=r.rect();if(t)t.style.left=`${f}px`,t.style.top=`${c}px`,t.textContent=String(m+1);if(i&&p){let l=p.split(",").map(Number);if(l.length===4&&l.every((d)=>Number.isFinite(d))){let[d,a,s,u]=l;i.style.cssText=`left:${d}px;top:${a}px;width:${s}px;height:${u}px`}}})}});var _b=z("sui-annotation-marker",{props:{kind:y.union(["point","region"],{default:"point",reflect:!0}),x:y.number({default:0}),y:y.number({default:0}),rect:y.string({default:""}),active:y.bool({default:!1,reflect:!0}),index:y.number({default:0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.kind=e.kind,t.x=e.x,t.y=e.y,t.rect=e.rect,t.active=e.active,t.index=e.index;let i=null;if(ee&&typeof ee.setup==="function"){if(n.scope=t,i=ee.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("part","pin"),c.setAttribute("role","button"),c.setAttribute("tabindex","0"),T(t,"pinEl",c),f.appendChild(c);let m=document.createElement("div");if(m.setAttribute("part","box"),m.setAttribute("role","button"),m.setAttribute("tabindex","0"),T(t,"boxEl",m),f.appendChild(m),i!=null){let p=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(d,a){return d[a]},set(d,a,s){let u=d[a];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(p.mount){let d=p.mount(l,r);if(typeof d==="function")o.push(d)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var re=P({scope:(n,e)=>{return{staged:M([]),emitRemove:(t)=>{e.dispatchEvent(new CustomEvent("sui-annotation-remove",{detail:{id:t},bubbles:!0,composed:!0}))}}},mount(n,e){let r=e.staged;if(r!=null)n.staged.set(Array.isArray(r)?r:[]);Object.defineProperty(e,"staged",{configurable:!0,get:()=>n.staged(),set:(o)=>n.staged.set(Array.isArray(o)?o:[])})}});var ih=z("sui-annotation-list",{shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null),i=null;if(re&&typeof re.setup==="function"){if(n.scope=t,i=re.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(l)=>R({anchor:l,items:()=>t.staged(),key:(d,a)=>{let s=Object.create(t);return s.a=d,s.__i=a,((u)=>d.id)(s)},render:(d,a)=>{let s=Object.create(t);return s.a=d,s.__i=a,s.__v=M(0),((u)=>{let b=document.createDocumentFragment(),h=[],v=document.createElement("li");v.setAttribute("part","item"),h.push(k(v,"data-id",()=>(u.__v(),u.a.id)));let g=document.createElement("span");g.setAttribute("class","kind");let x=document.createTextNode("");g.appendChild(x),h.push(F(x,()=>(u.__v(),u.a.kind))),v.appendChild(g);let A=document.createElement("span");A.setAttribute("class","note"),h.push(k(A,"title",()=>(u.__v(),u.a.note??"")));let w=document.createTextNode("");A.appendChild(w),h.push(F(w,()=>(u.__v(),u.a.note??"(no note)"))),v.appendChild(A);let E=document.createElement("button");E.setAttribute("type","button"),E.setAttribute("aria-label","Remove annotation"),h.push(K(E,"click",function(){let C=()=>u.emitRemove(u.a.id);return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let j=document.createTextNode("Remove");return E.appendChild(j),v.appendChild(E),b.appendChild(v),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let C of h)try{C()}catch(N){}},update:(C,N)=>{u.a=C,u.__i=N,u.__v.set((O)=>O+1)}}})(s)}}),c=(l)=>B({anchor:l,test:()=>{if(t.staged().length===0)return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("div");s.setAttribute("class","empty"),s.setAttribute("part","empty");let u=document.createTextNode("No staged annotations.");return s.appendChild(u),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let b of a)try{b()}catch(h){}}}},()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("ol");s.setAttribute("part","list");let u=document.createComment("for");return s.appendChild(u),a.push(f(u)),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let b of a)try{b()}catch(h){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createComment("if");if(m.appendChild(p),o.push(c(p)),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var oe=P({mount(n,e){let r=n,o=D(()=>{let t=r.inputEl();if(!t)return;let i=()=>{if(r.disabled())return;e.dispatchEvent(new CustomEvent("sui-radio-select",{detail:{value:r.value()},bubbles:!0,composed:!0}))},f=(c)=>{if(!["ArrowDown","ArrowRight","ArrowUp","ArrowLeft"].includes(c.key))return;c.preventDefault();let p=c.key==="ArrowDown"||c.key==="ArrowRight";e.dispatchEvent(new CustomEvent("sui-radio-nav",{detail:{direction:p?"next":"prev"},bubbles:!0,composed:!0}))};return t.addEventListener("click",i),t.addEventListener("keydown",f),()=>{t.removeEventListener("click",i),t.removeEventListener("keydown",f)}});return e.focusInput=()=>{e.shadowRoot?.querySelector("input")?.focus()},()=>o()}});var ch=z("sui-radio",{props:{value:y.string({default:"",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),checked:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.disabled=e.disabled,t.checked=e.checked,t.tone=e.tone,t.size=e.size;let i=null;if(oe&&typeof oe.setup==="function"){if(n.scope=t,i=oe.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("label");c.setAttribute("part","label");let m=document.createElement("span");m.setAttribute("class","circle"),m.setAttribute("part","circle");let p=document.createElement("input");p.setAttribute("part","input"),p.setAttribute("type","radio"),o.push(bt(p,t.checked)),o.push(k(p,"disabled",()=>t.disabled())),o.push(k(p,"aria-checked",()=>t.checked()?"true":"false")),o.push(k(p,"tabindex",()=>t.checked()?"0":"-1")),T(t,"inputEl",p),m.appendChild(p);let l=document.createElement("span");l.setAttribute("class","dot"),l.setAttribute("part","dot"),l.setAttribute("aria-hidden","true"),m.appendChild(l),c.appendChild(m);let d=document.createElement("slot");if(c.appendChild(d),f.appendChild(c),i!=null){let a=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(a){let s=new Proxy(t,{get(u,b){return u[b]},set(u,b,h){let v=u[b];if(v!=null&&typeof v.set==="function"){if(typeof h==="function")v.set(()=>h);else v.set(h);return!0}return!1}});if(a.mount){let u=a.mount(s,r);if(typeof u==="function")o.push(u)}if(a.unmount)o.push(()=>a.unmount(s))}}return t}});var ie=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","radiogroup");let o=document.createElement("input");o.type="hidden",e.appendChild(o);let t=()=>Array.from(e.querySelectorAll(":scope > sui-radio")),i=()=>t().filter((d)=>!d.hasAttribute("disabled")&&!r.disabled()),f=()=>{let d=r.value(),a=r.disabled(),s=r.tone(),u=r.size();for(let b of t()){if(b.toggleAttribute("checked",b.getAttribute("value")===d),a)b.setAttribute("disabled","");if(s&&!b.hasAttribute("tone"))b.setAttribute("tone",s);if(u&&!b.hasAttribute("size"))b.setAttribute("size",u)}o.name=r.name(),o.value=d},c=D(()=>{r.value(),r.disabled(),r.tone(),r.size(),f()}),m=D(()=>{let d=r.label();if(d)e.setAttribute("aria-label",d);else e.removeAttribute("aria-label")}),p=(d)=>{let a=d;a.stopPropagation();let s=a.detail.value;if(s===r.value())return;r.value.set(s),f(),e.dispatchEvent(new CustomEvent("sui-change",{detail:{value:s},bubbles:!0,composed:!0}))},l=(d)=>{let a=d;a.stopPropagation();let s=i();if(!s.length)return;let u=a.composedPath().find((g)=>g.tagName?.toLowerCase()==="sui-radio"),b=u?s.indexOf(u):-1,h=a.detail.direction==="next"?s[(b+1)%s.length]:s[(b-1+s.length)%s.length],v=h.getAttribute("value")??"";r.value.set(v),f(),h.focusInput?.(),e.dispatchEvent(new CustomEvent("sui-change",{detail:{value:v},bubbles:!0,composed:!0}))};return e.addEventListener("sui-radio-select",p),e.addEventListener("sui-radio-nav",l),()=>{c(),m(),e.removeEventListener("sui-radio-select",p),e.removeEventListener("sui-radio-nav",l),o.remove()}}});var vh=z("sui-radio-group",{props:{value:y.string({default:"",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),name:y.string({default:""}),label:y.string({default:""})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    gap: var(--space-2, 8px);
    font-family: var(--font-body);
    color: var(--foreground);
  }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.disabled=e.disabled,t.tone=e.tone,t.size=e.size,t.name=e.name,t.label=e.label;let i=null;if(ie&&typeof ie.setup==="function"){if(n.scope=t,i=ie.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");if(f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var ae=P({mount(n,e){let r=n,o=document.createElement("input");o.type="hidden",e.appendChild(o);let t=(s)=>{let u=r.step(),b=r.min(),h=r.max(),v=Math.round((s-b)/u)*u+b;return Math.max(b,Math.min(h,v))},i=(s)=>{let u=t(s);if(u===r.value())return;r.value.set(u),o.name=r.name(),o.value=String(u),e.dispatchEvent(new CustomEvent("sui-change",{detail:{value:u},bubbles:!0,composed:!0}))},f=(s)=>{let u=r.trackEl();if(!u)return r.value();let b=u.getBoundingClientRect(),h=Math.max(0,Math.min(1,(s-b.left)/b.width));return r.min()+h*(r.max()-r.min())},c=D(()=>{let s=r.value(),u=r.min(),h=r.max()-u||1,v=(s-u)/h*100,g=r.fillEl(),x=r.thumbEl();if(g)g.style.width=`${v}%`;if(x)x.style.left=`${v}%`;o.name=r.name(),o.value=String(s)}),m=!1,p=(s)=>{if(r.disabled())return;s.preventDefault(),m=!0,i(f(s.clientX)),r.thumbEl()?.focus();let b=(v)=>{if(m)i(f(v.clientX))},h=()=>{m=!1,document.removeEventListener("pointermove",b),document.removeEventListener("pointerup",h)};document.addEventListener("pointermove",b),document.addEventListener("pointerup",h)},l=(s)=>{if(r.disabled())return;let u=r.step(),b=!0;switch(s.key){case"ArrowRight":case"ArrowUp":i(r.value()+u);break;case"ArrowLeft":case"ArrowDown":i(r.value()-u);break;case"Home":i(r.min());break;case"End":i(r.max());break;case"PageUp":i(r.value()+u*10);break;case"PageDown":i(r.value()-u*10);break;default:b=!1}if(b)s.preventDefault()},d=D(()=>{let s=r.containerEl();if(!s)return;return s.addEventListener("pointerdown",p),()=>s.removeEventListener("pointerdown",p)}),a=D(()=>{let s=r.thumbEl();if(!s)return;return s.addEventListener("keydown",l),()=>s.removeEventListener("keydown",l)});return()=>{c(),d(),a(),o.remove()}}});var Eh=z("sui-slider",{props:{value:y.number({default:0,reflect:!0}),min:y.number({default:0}),max:y.number({default:100}),step:y.number({default:1}),disabled:y.bool({default:!1,reflect:!0}),tone:y.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),name:y.string({default:""}),label:y.string({default:"slider"})},shadow:!0,styles:`
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

`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.min=e.min,t.max=e.max,t.step=e.step,t.disabled=e.disabled,t.tone=e.tone,t.size=e.size,t.name=e.name,t.label=e.label;let i=null;if(ae&&typeof ae.setup==="function"){if(n.scope=t,i=ae.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","container"),T(t,"containerEl",c);let m=document.createElement("div");m.setAttribute("class","track"),m.setAttribute("part","track"),T(t,"trackEl",m);let p=document.createElement("div");p.setAttribute("class","fill"),p.setAttribute("part","fill"),T(t,"fillEl",p),m.appendChild(p),c.appendChild(m);let l=document.createElement("div");if(l.setAttribute("class","thumb"),l.setAttribute("part","thumb"),l.setAttribute("role","slider"),o.push(k(l,"tabindex",()=>t.disabled()?"-1":"0")),o.push(k(l,"aria-valuemin",()=>t.min())),o.push(k(l,"aria-valuemax",()=>t.max())),o.push(k(l,"aria-valuenow",()=>t.value())),o.push(k(l,"aria-label",()=>t.label()||"slider")),o.push(k(l,"aria-disabled",()=>t.disabled()?"true":"false")),T(t,"thumbEl",l),c.appendChild(l),f.appendChild(c),i!=null){let d=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(d){let a=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(d.mount){let s=d.mount(a,r);if(typeof s==="function")o.push(s)}if(d.unmount)o.push(()=>d.unmount(a))}}return t}});var jh=z("sui-segmented-option",{props:{value:y.string({default:"",reflect:!0}),icon:y.string({default:"",reflect:!0}),disabled:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.icon=e.icon,t.disabled=e.disabled;let i=r.shadowRoot??r;L(i);let f=document.createTextNode(`

`);i.appendChild(f);let c=document.createTextNode(`
`);i.appendChild(c);let m=document.createTextNode(`
`);i.appendChild(m);let p=document.createTextNode(`

`);i.appendChild(p);let l=document.createTextNode(`
`);return i.appendChild(l),t}});var Hi=(n)=>typeof CSS<"u"&&CSS.escape?CSS.escape(n):n.replace(/[^\w-]/g,"\\$&"),se=P({scope:()=>{let n=M([]),e=J(null);return{options:n,containerEl:e}},mount(n,e){if(!e.hasAttribute("role"))e.setAttribute("role","radiogroup");let r=document.createElement("input");r.type="hidden",e.appendChild(r);let o=()=>{return Array.from(e.querySelectorAll(":scope > sui-segmented-option")).map((d)=>({value:d.getAttribute("value")??"",label:d.textContent?.trim()??"",icon:d.getAttribute("icon")||null,disabled:d.hasAttribute("disabled")}))},t=(l)=>{if(n.disabled())return;let a=n.options().find((s)=>s.value===l);if(!a||a.disabled)return;if(n.value()===l)return;n.value.set(l),e.dispatchEvent(new CustomEvent("sui-change",{detail:{value:l},bubbles:!0,composed:!0}))},i=(l)=>{let d=l.target?.closest(".btn");if(!d)return;let a=d.getAttribute("data-value");if(a!==null)t(a)},f=(l)=>{let a=n.options().filter((g)=>!g.disabled&&!n.disabled());if(!a.length)return;let s=a.map((g)=>g.value),u=n.value(),b=Math.max(0,s.indexOf(u)),h=b;if(l.key==="ArrowRight"||l.key==="ArrowDown")l.preventDefault(),h=(b+1)%s.length;else if(l.key==="ArrowLeft"||l.key==="ArrowUp")l.preventDefault(),h=(b-1+s.length)%s.length;else if(l.key==="Enter"||l.key===" "){l.preventDefault();let x=e.shadowRoot.activeElement?.getAttribute("data-value");if(x)t(x);return}else return;let v=s[h]??"";t(v),queueMicrotask(()=>{e.shadowRoot?.querySelector(`.btn[data-value="${Hi(v)}"]`)?.focus()})};n.options.set(o());let c=new MutationObserver(()=>n.options.set(o()));c.observe(e,{childList:!0,subtree:!1});let m=D(()=>{r.name=n.name(),r.value=n.value()}),p=D(()=>{let l=n.containerEl();if(!l)return;return l.addEventListener("click",i),l.addEventListener("keydown",f),()=>{l.removeEventListener("click",i),l.removeEventListener("keydown",f)}});return()=>{c.disconnect(),m(),p(),r.remove()}}});var Ph=z("sui-segmented",{props:{value:y.string({default:"",reflect:!0}),size:y.union(["sm","md"],{default:"md",reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),name:y.string({default:""})},shadow:!0,styles:`
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

`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.size=e.size,t.disabled=e.disabled,t.name=e.name;let i=null;if(se&&typeof se.setup==="function"){if(n.scope=t,i=se.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(l)=>R({anchor:l,items:()=>t.options(),key:(d,a)=>{let s=Object.create(t);return s.opt=d,s.__i=a,((u)=>d.value)(s)},render:(d,a)=>{let s=Object.create(t);return s.opt=d,s.__i=a,s.__v=M(0),((u)=>{let b=(w)=>B({anchor:w,test:()=>{if(u.opt.icon)return 0;return-1},branches:[()=>{let E=document.createDocumentFragment(),j=[],C=document.createElement("span");C.setAttribute("class","btn-icon"),C.setAttribute("aria-hidden","true");let N=document.createElement("sui-icon");return j.push(tt(N,"name",()=>(u.__v(),u.opt.icon))),N.setAttribute("size","sm"),C.appendChild(N),E.appendChild(C),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let O of j)try{O()}catch(S){}}}}]}),h=document.createDocumentFragment(),v=[],g=document.createElement("button");g.setAttribute("class","btn"),g.setAttribute("role","radio"),g.setAttribute("type","button"),v.push(k(g,"data-value",()=>(u.__v(),u.opt.value))),v.push(k(g,"aria-checked",()=>(u.__v(),u.opt.value===u.value()?"true":"false"))),v.push(k(g,"aria-disabled",()=>(u.__v(),u.disabled()||u.opt.disabled?"true":"false"))),v.push(tt(g,"disabled",()=>(u.__v(),u.disabled()||u.opt.disabled))),v.push(k(g,"tabindex",()=>(u.__v(),u.opt.value===u.value()?"0":"-1")));let x=document.createComment("if");g.appendChild(x),v.push(b(x));let A=document.createTextNode("");return g.appendChild(A),v.push(F(A,()=>(u.__v(),u.opt.label))),h.appendChild(g),{nodes:Array.from(h.childNodes),cleanup:()=>{for(let w of v)try{w()}catch(E){}},update:(w,E)=>{u.opt=w,u.__i=E,u.__v.set((j)=>j+1)}}})(s)}}),c=r.shadowRoot??r;L(c);let m=document.createElement("div");m.setAttribute("class","container"),m.setAttribute("part","container"),T(t,"containerEl",m);let p=document.createComment("for");if(m.appendChild(p),o.push(f(p)),c.appendChild(m),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var ue=P({mount(n,e){let r=n,o=D(()=>{let t=r.buttonEl();if(!t)return;let i=()=>{if(r.disabled())return;r.pressed.set(!r.pressed()),e.dispatchEvent(new CustomEvent("sui-toggle",{detail:{pressed:r.pressed(),value:r.value()},bubbles:!0,composed:!0}))};return t.addEventListener("click",i),()=>t.removeEventListener("click",i)});return e.focusButton=()=>{e.shadowRoot?.querySelector("button")?.focus()},()=>o()}});var Fh=z("sui-toggle",{props:{pressed:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),value:y.string({default:"",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:y.union(["outline","ghost"],{default:"outline",reflect:!0})},shadow:!0,styles:`
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

`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.pressed=e.pressed,t.disabled=e.disabled,t.value=e.value,t.size=e.size,t.variant=e.variant;let i=null;if(ue&&typeof ue.setup==="function"){if(n.scope=t,i=ue.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("button");c.setAttribute("part","button"),c.setAttribute("type","button"),o.push(k(c,"aria-pressed",()=>t.pressed()?"true":"false")),o.push(k(c,"disabled",()=>t.disabled())),T(t,"buttonEl",c);let m=document.createElement("slot");if(c.appendChild(m),f.appendChild(c),i!=null){let p=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(d,a){return d[a]},set(d,a,s){let u=d[a];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(p.mount){let d=p.mount(l,r);if(typeof d==="function")o.push(d)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var le=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","group");let o=document.createElement("input");o.type="hidden",e.appendChild(o);let t=()=>{let u=r.defaultSlotEl();if(!u)return Array.from(e.querySelectorAll(":scope > sui-toggle"));return u.assignedElements().filter((b)=>b.tagName.toLowerCase()==="sui-toggle")},i=()=>t().filter((u)=>!u.hasAttribute("disabled")),f=()=>new Set(r.value().split(",").filter(Boolean)),c=(u)=>{let b=Array.from(u).join(",");r.value.set(b),m(),o.name=r.name(),o.value=b,e.dispatchEvent(new CustomEvent("sui-change",{detail:{value:b},bubbles:!0,composed:!0}))},m=()=>{let u=f(),b=r.size(),h=r.variant();for(let v of t()){let g=v.getAttribute("value")??"";if(v.toggleAttribute("pressed",u.has(g)),b)v.setAttribute("size",b);if(h)v.setAttribute("variant",h)}},p=(u)=>{let b=u;b.stopPropagation();let h=b.composedPath().find((x)=>x.tagName?.toLowerCase()==="sui-toggle");if(!h)return;let v=h.getAttribute("value")??"",g=f();if(r.type()==="single")if(g.has(v))c(new Set);else c(new Set([v]));else{if(g.has(v))g.delete(v);else g.add(v);c(g)}},l=(u)=>{if(u.key!=="ArrowRight"&&u.key!=="ArrowLeft")return;let b=i();if(!b.length)return;let h=u.composedPath().find((x)=>b.includes(x)),v=h?b.indexOf(h):-1,g=u.key==="ArrowRight"?(v+1)%b.length:(v-1+b.length)%b.length;b[g]?.focusButton?.(),u.preventDefault()},d=D(()=>{r.value(),r.size(),r.variant(),m()}),a=D(()=>{let u=r.defaultSlotEl();if(!u)return;let b=()=>m();return u.addEventListener("slotchange",b),()=>u.removeEventListener("slotchange",b)}),s=D(()=>{o.name=r.name(),o.value=r.value()});return e.addEventListener("sui-toggle",p),e.addEventListener("keydown",l),()=>{d(),a(),s(),e.removeEventListener("sui-toggle",p),e.removeEventListener("keydown",l),o.remove()}}});var Jh=z("sui-toggle-group",{props:{type:y.union(["single","multiple"],{default:"single",reflect:!0}),value:y.string({default:"",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:y.union(["outline","ghost"],{default:"outline",reflect:!0}),name:y.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    gap: 0;
  }

  ::slotted(sui-toggle:not(:first-child)) { margin-left: -1px; }
  ::slotted(sui-toggle:first-child) { border-radius: var(--radius, 6px) 0 0 var(--radius, 6px); }
  ::slotted(sui-toggle:last-child) { border-radius: 0 var(--radius, 6px) var(--radius, 6px) 0; }
  ::slotted(sui-toggle:not(:first-child):not(:last-child)) { border-radius: 0; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.type=e.type,t.value=e.value,t.size=e.size,t.variant=e.variant,t.name=e.name;let i=null;if(le&&typeof le.setup==="function"){if(n.scope=t,i=le.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");if(T(t,"defaultSlotEl",c),f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var Zh=z("sui-code",{props:{lang:y.string({default:"",reflect:!0}),block:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.lang=e.lang,t.block=e.block;let i=(p)=>B({anchor:p,test:()=>{if(t.lang())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),d=[],a=document.createElement("header");a.setAttribute("part","label");let s=document.createTextNode("");return a.appendChild(s),d.push(F(s,()=>t.lang())),l.appendChild(a),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let u of d)try{u()}catch(b){}}}}]}),f=(p)=>B({anchor:p,test:()=>{if(t.block())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),d=[],a=document.createElement("div");a.setAttribute("class","block"),a.setAttribute("part","block"),a.setAttribute("role","region"),d.push(k(a,"aria-label",()=>t.lang()?t.lang()+" code block":"code block"));let s=document.createComment("if");a.appendChild(s),d.push(i(s));let u=document.createElement("pre");u.setAttribute("part","pre");let b=document.createElement("slot");return u.appendChild(b),a.appendChild(u),l.appendChild(a),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let h of d)try{h()}catch(v){}}}},()=>{let l=document.createDocumentFragment(),d=[],a=document.createElement("code");a.setAttribute("class","inline"),a.setAttribute("part","inline");let s=document.createElement("slot");return a.appendChild(s),l.appendChild(a),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let u of d)try{u()}catch(b){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createComment("if");return c.appendChild(m),o.push(f(m)),t}});var de=P({mount(n,e){let r=n,o,t=async()=>{try{await navigator.clipboard.writeText(r.value())}catch{let f=document.createElement("textarea");f.value=r.value(),document.body.appendChild(f),f.select();try{document.execCommand("copy")}catch{}f.remove()}if(r.copied.set(!0),e.dispatchEvent(new CustomEvent("sui-copy",{detail:{value:r.value()},bubbles:!0,composed:!0})),o)clearTimeout(o);o=setTimeout(()=>{r.copied.set(!1)},1200)},i=D(()=>{let f=r.buttonEl();if(!f)return;return f.addEventListener("click",t),()=>f.removeEventListener("click",t)});return()=>{if(i(),o)clearTimeout(o)}}});var _h=z("sui-copy-button",{props:{value:y.string({default:"",reflect:!0}),label:y.string({default:"Copy",reflect:!0}),copied:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.label=e.label,t.copied=e.copied;let i=null;if(de&&typeof de.setup==="function"){if(n.scope=t,i=de.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(s)=>B({anchor:s,test:()=>{if(t.copied())return 0;return 1},branches:[()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("sui-icon");return h.setAttribute("name","check"),h.setAttribute("size","sm"),h.setAttribute("aria-hidden","true"),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let v of b)try{v()}catch(g){}}}},()=>{let u=document.createDocumentFragment(),b=[],h=document.createElement("sui-icon");return h.setAttribute("name","copy"),h.setAttribute("size","sm"),h.setAttribute("aria-hidden","true"),u.appendChild(h),{nodes:Array.from(u.childNodes),cleanup:()=>{for(let v of b)try{v()}catch(g){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createElement("button");m.setAttribute("part","button"),m.setAttribute("type","button"),o.push(k(m,"aria-label",()=>t.label())),T(t,"buttonEl",m);let p=document.createComment("if");m.appendChild(p),o.push(f(p));let l=document.createElement("slot");o.push(nt(l,r,"data-empty")),m.appendChild(l);let d=document.createElement("span");d.setAttribute("class","default-label");let a=document.createTextNode("");if(d.appendChild(a),o.push(F(a,()=>t.copied()?"Copied":t.label())),m.appendChild(d),c.appendChild(m),i!=null){let s=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,v){let g=b[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});function ur(n){return n.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function ao(n,e,r,o,t){let i=e!==null?`<span class="key">"${ur(e)}"</span>: `:"";if(n===null)return`${i}<span class="null">null</span>`;if(typeof n==="string")return`${i}<span class="s">"${ur(n)}"</span>`;if(typeof n==="number")return`${i}<span class="n">${n}</span>`;if(typeof n==="boolean")return`${i}<span class="b">${String(n)}</span>`;let f=Array.isArray(n),c=f?n.map((s,u)=>[String(u),s]):Object.entries(n),m=o||r<t,p=f?"[":"{",l=f?"]":"}",d=c.length;if(d===0)return`${i}${p}${l}`;let a=c.map(([s,u])=>`<div class="node">${ao(u,f?null:s,r+1,o,t)}</div>`).join("");return`<details${m?" open":""}>
<summary><span class="caret">▶</span>${i}${p}<span class="meta">${d} ${f?"items":"keys"}</span></summary>
<div class="branch">${a}</div>
${l}</details>`}var ce=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","tree");if(!e.hasAttribute("aria-label"))e.setAttribute("aria-label","JSON tree");let o=r.contentEl();if(!o)return()=>{};return et(o,()=>{let t=r.data(),i=r.expanded(),f=r.depth();if(!t)return"";let c;try{c=JSON.parse(t)}catch(m){return`<span class="error" part="error" role="alert">Invalid JSON: ${ur(m.message)}</span>`}return`<div part="root">${ao(c,null,0,i,f)}</div>`})}});var iv=z("sui-json-view",{props:{data:y.string({default:"",reflect:!0}),expanded:y.bool({default:!1,reflect:!0}),depth:y.number({default:1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.data=e.data,t.expanded=e.expanded,t.depth=e.depth;let i=null;if(ce&&typeof ce.setup==="function"){if(n.scope=t,i=ce.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");if(c.setAttribute("class","json-root"),T(t,"contentEl",c),f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});function so(n){return n.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function Bi(n,e){let r=n.split(`
`),o=e.split(`
`),t=r.length,i=o.length,f=Array.from({length:t+1},()=>Array(i+1).fill(0));for(let l=t-1;l>=0;l--)for(let d=i-1;d>=0;d--)f[l][d]=r[l]===o[d]?f[l+1][d+1]+1:Math.max(f[l+1][d],f[l][d+1]);let c=[],m=0,p=0;while(m<t&&p<i)if(r[m]===o[p])c.push({kind:"ctx",text:r[m]}),m++,p++;else if(f[m+1][p]>=f[m][p+1])c.push({kind:"del",text:r[m]}),m++;else c.push({kind:"add",text:o[p]}),p++;while(m<t)c.push({kind:"del",text:r[m++]});while(p<i)c.push({kind:"add",text:o[p++]});return c}function Fi(n){return n.split(`
`).map((e)=>{if(e.startsWith("@@"))return{kind:"hunk",text:e};if(e.startsWith("+"))return{kind:"add",text:e.slice(1)};if(e.startsWith("-"))return{kind:"del",text:e.slice(1)};return{kind:"ctx",text:e.startsWith(" ")?e.slice(1):e}})}function Gi(n){return n.map((e)=>{let r=e.kind==="add"?"+":e.kind==="del"?"-":e.kind==="hunk"?"@":" ",o=e.kind==="add"?"added":e.kind==="del"?"removed":e.kind==="hunk"?"hunk":"context";return`<div class="line ${e.kind}" part="line" role="listitem" aria-label="${o}: ${so(e.text)}"><span class="sign" aria-hidden="true">${r}</span><span>${so(e.text)}</span></div>`}).join("")}var pe=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","region");if(!e.hasAttribute("aria-label"))e.setAttribute("aria-label","diff");let o=r.contentEl();if(!o)return()=>{};return et(o,()=>{let t=r.value(),i=r.from(),f=r.to(),c;if(t)c=Fi(t);else if(i||f)c=Bi(i,f);else c=[];return Gi(c)})}});var cv=z("sui-diff",{props:{from:y.string({default:"",reflect:!0}),to:y.string({default:"",reflect:!0}),value:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.from=e.from,t.to=e.to,t.value=e.value;let i=null;if(pe&&typeof pe.setup==="function"){if(n.scope=t,i=pe.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");if(c.setAttribute("class","diff-root"),T(t,"contentEl",c),f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});function lr(n){return n.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function Gt(n){let e=lr(n);return e=e.replace(/`([^`]+)`/g,(r,o)=>`<code>${o}</code>`),e=e.replace(/\*\*([^*]+)\*\*/g,"<strong>$1</strong>"),e=e.replace(/(^|[^*])\*([^*\n]+)\*/g,"$1<em>$2</em>"),e=e.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g,(r,o,t)=>`<a href="${t}" rel="noopener noreferrer">${o}</a>`),e}function Ui(n){let e=n.replace(/\r\n/g,`
`).split(`
`),r=[],o=0;while(o<e.length){let t=e[o];if(/^```/.test(t)){let c=t.slice(3).trim(),m=[];o++;while(o<e.length&&!/^```/.test(e[o]))m.push(e[o++]);o++,r.push(`<pre data-lang="${lr(c)}"><code>${lr(m.join(`
`))}</code></pre>`);continue}let i=/^(#{1,6})\s+(.*)$/.exec(t);if(i){r.push(`<h${i[1].length}>${Gt(i[2])}</h${i[1].length}>`),o++;continue}if(/^\s*[-*]\s+/.test(t)){let c=[];while(o<e.length&&/^\s*[-*]\s+/.test(e[o]))c.push(`<li>${Gt(e[o].replace(/^\s*[-*]\s+/,""))}</li>`),o++;r.push(`<ul>${c.join("")}</ul>`);continue}if(/^\s*\d+\.\s+/.test(t)){let c=[];while(o<e.length&&/^\s*\d+\.\s+/.test(e[o]))c.push(`<li>${Gt(e[o].replace(/^\s*\d+\.\s+/,""))}</li>`),o++;r.push(`<ol>${c.join("")}</ol>`);continue}if(/^>\s?/.test(t)){let c=[];while(o<e.length&&/^>\s?/.test(e[o]))c.push(e[o].replace(/^>\s?/,"")),o++;r.push(`<blockquote>${Gt(c.join(" "))}</blockquote>`);continue}if(t.trim()===""){o++;continue}let f=[];while(o<e.length&&e[o].trim()!==""&&!/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(e[o]))f.push(e[o]),o++;r.push(`<p>${Gt(f.join(" "))}</p>`)}return r.join(`
`)}var fe=P({mount(n){let e=n,r=e.contentEl();if(!r)return()=>{};return et(r,()=>Ui(e.value()??""))}});var vv=z("sui-markdown",{props:{value:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value;let i=null;if(fe&&typeof fe.setup==="function"){if(n.scope=t,i=fe.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");if(c.setAttribute("class","md-root"),T(t,"contentEl",c),f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var uo={actions:"#view-topbar-actions","status-start":"#view-status-start","status-end":"#view-status-end"},me={title:"#breadcrumbs-title",branch:"#status-branch"},be=P({mount(n,e){let r=n,o={};function t(){for(let m of Object.keys(uo)){let p=document.querySelector(uo[m]);if(!p)continue;let l=Array.from(e.querySelectorAll(`[slot="${m}"]`));if(l.length===0)continue;for(let d of l)p.appendChild(d);o[m]=l}}function i(){let m=document.querySelector(me.title);if(m)m.textContent="";if(r.branch()){let p=document.querySelector(me.branch);if(p)p.textContent=""}if(r.ahead()>0||r.behind()>0){let p=document.querySelector("#status-ab");if(p)p.textContent=""}if(r.tone()!=="default"||r.busy()){let p=document.querySelector("#status-dot");if(p)delete p.dataset.tone,p.removeAttribute("data-busy")}}function f(){for(let m of Object.keys(o)){for(let p of o[m]??[])p.remove();delete o[m]}i()}let c=D(()=>{let m=r.title(),p=r.branch(),l=r.ahead(),d=r.behind(),a=r.tone(),s=r.busy();if(m){let u=document.querySelector(me.title);if(u)u.textContent=m}if(p){let u=document.querySelector(me.branch);if(u)u.textContent=p}if(l>0||d>0){let u=document.querySelector("#status-ab");if(u){let b=[];if(l>0)b.push(`↑${l}`);if(d>0)b.push(`↓${d}`);u.textContent=b.join(" ")}}if(a!=="default"||s){let u=document.querySelector("#status-dot");if(u)if(u.dataset.tone=a,s)u.setAttribute("data-busy","");else u.removeAttribute("data-busy")}});return queueMicrotask(()=>t()),()=>{c(),f()}}});var Ev=z("sui-view-chrome",{props:{title:y.string({default:"",reflect:!0}),branch:y.string({default:"",reflect:!0}),ahead:y.number({default:0,reflect:!0}),behind:y.number({default:0,reflect:!0}),tone:y.union(["default","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),busy:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.title=e.title,t.branch=e.branch,t.ahead=e.ahead,t.behind=e.behind,t.tone=e.tone,t.busy=e.busy;let i=null;if(be&&typeof be.setup==="function"){if(n.scope=t,i=be.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createTextNode(`

`);f.appendChild(c);let m=document.createTextNode(`
`);f.appendChild(m);let p=document.createTextNode(`
`);f.appendChild(p);let l=document.createTextNode(`
`);f.appendChild(l);let d=document.createTextNode(`
`);f.appendChild(d);let a=document.createTextNode(`
`);f.appendChild(a);let s=document.createTextNode(`

`);f.appendChild(s);let u=document.createTextNode(`
`);if(f.appendChild(u),i!=null){let b=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(b){let h=new Proxy(t,{get(v,g){return v[g]},set(v,g,x){let A=v[g];if(A!=null&&typeof A.set==="function"){if(typeof x==="function")A.set(()=>x);else A.set(x);return!0}return!1}});if(b.mount){let v=b.mount(h,r);if(typeof v==="function")o.push(v)}if(b.unmount)o.push(()=>b.unmount(h))}}return t}});var he=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","navigation");let o=()=>Array.from(e.querySelectorAll(":scope > sui-navigation-item")),t=()=>o().filter((l)=>!l.hasAttribute("disabled")),i=()=>{let l=r.value();for(let d of o())d.toggleAttribute("active",d.getAttribute("value")===l)},f=(l)=>{let d=l;d.stopPropagation();let a=d.detail.value;r.value.set(a),i(),e.dispatchEvent(new CustomEvent("sui-change",{detail:{value:a},bubbles:!0,composed:!0}))},c=(l)=>{if(l.key!=="ArrowRight"&&l.key!=="ArrowLeft"&&l.key!=="Home"&&l.key!=="End")return;let d=t();if(!d.length)return;let a=l.composedPath().find((b)=>d.includes(b)),s=a?d.indexOf(a):-1,u;if(l.key==="ArrowRight")u=d[(s+1)%d.length];else if(l.key==="ArrowLeft")u=d[(s-1+d.length)%d.length];else if(l.key==="Home")u=d[0];else if(l.key==="End")u=d[d.length-1];if(u)l.preventDefault(),u.focusLink?.()},m=D(()=>{r.value(),i()}),p=D(()=>{let l=r.defaultSlotEl();if(!l)return;let d=()=>i();return l.addEventListener("slotchange",d),()=>l.removeEventListener("slotchange",d)});return e.addEventListener("sui-nav-select",f),e.addEventListener("keydown",c),()=>{m(),p(),e.removeEventListener("sui-nav-select",f),e.removeEventListener("keydown",c)}}});var Dv=z("sui-navigation-menu",{props:{value:y.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    align-items: stretch;
    gap: 0;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-body);
  }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value;let i=null;if(he&&typeof he.setup==="function"){if(n.scope=t,i=he.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");if(T(t,"defaultSlotEl",c),f.appendChild(c),i!=null){let m=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(t,{get(l,d){return l[d]},set(l,d,a){let s=l[d];if(s!=null&&typeof s.set==="function"){if(typeof a==="function")s.set(()=>a);else s.set(a);return!0}return!1}});if(m.mount){let l=m.mount(p,r);if(typeof l==="function")o.push(l)}if(m.unmount)o.push(()=>m.unmount(p))}}return t}});var ve=P({mount(n,e){let r=n,o=(t)=>{if(r.disabled())return;if(!r.href())t.preventDefault();e.dispatchEvent(new CustomEvent("sui-nav-select",{detail:{value:r.value()},bubbles:!0,composed:!0}))};return e.focusLink=()=>{e.shadowRoot?.querySelector(".link")?.focus()},e.addEventListener("click",o),()=>e.removeEventListener("click",o)}});var Sv=z("sui-navigation-item",{props:{value:y.string({default:"",reflect:!0}),active:y.bool({default:!1,reflect:!0}),disabled:y.bool({default:!1,reflect:!0}),href:y.string({default:""})},shadow:!0,styles:`
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

`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.value=e.value,t.active=e.active,t.disabled=e.disabled,t.href=e.href;let i=null;if(ve&&typeof ve.setup==="function"){if(n.scope=t,i=ve.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(l)=>B({anchor:l,test:()=>{if(t.href())return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("a");s.setAttribute("class","link"),s.setAttribute("part","link"),a.push(k(s,"href",()=>t.href())),a.push(k(s,"aria-current",()=>t.active()?"page":"false"));let u=document.createElement("slot");return s.appendChild(u),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let b of a)try{b()}catch(h){}}}},()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("button");s.setAttribute("class","link"),s.setAttribute("part","link"),s.setAttribute("type","button"),a.push(k(s,"aria-current",()=>t.active()?"true":"false")),a.push(k(s,"disabled",()=>t.disabled()));let u=document.createElement("slot");return s.appendChild(u),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let b of a)try{b()}catch(h){}}}}]}),c=r.shadowRoot??r;L(c);let m=document.createComment("if");c.appendChild(m),o.push(f(m));let p=document.createElement("div");if(p.setAttribute("class","indicator"),p.setAttribute("part","indicator"),p.setAttribute("aria-hidden","true"),c.appendChild(p),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var ge=P({mount(n,e){let r=n,o=null,t=()=>{let c=r.viewportEl();if(!c)return;let m=r.orientation(),p=m==="vertical"||m==="both",l=m==="horizontal"||m==="both",d=r.vThumb(),a=r.hThumb();if(p&&d){let s=c.clientHeight/c.scrollHeight;if(s<1){let u=Math.max(24,s*c.clientHeight),h=c.scrollTop/(c.scrollHeight-c.clientHeight)*(c.clientHeight-u);d.style.height=`${u}px`,d.style.top=`${h}px`,d.style.display=""}else d.style.display="none"}if(l&&a){let s=c.clientWidth/c.scrollWidth;if(s<1){let u=Math.max(24,s*c.clientWidth),h=c.scrollLeft/(c.scrollWidth-c.clientWidth)*(c.clientWidth-u);a.style.width=`${u}px`,a.style.left=`${h}px`,a.style.display=""}else a.style.display="none"}},i=()=>{t();let c=r.vScrollbar(),m=r.hScrollbar();if(c?.classList.add("active"),m?.classList.add("active"),o)clearTimeout(o);o=setTimeout(()=>{c?.classList.remove("active"),m?.classList.remove("active")},1200)},f=r.viewportEl();if(f)f.addEventListener("scroll",i,{passive:!0});return t(),()=>{if(o)clearTimeout(o);if(f)f.removeEventListener("scroll",i)}}});var Xv=z("sui-scroll-area",{props:{orientation:y.union(["vertical","horizontal","both"],{default:"vertical",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.orientation=e.orientation,t.size=e.size;let i=null;if(ge&&typeof ge.setup==="function"){if(n.scope=t,i=ge.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","viewport"),c.setAttribute("part","viewport"),c.setAttribute("tabindex","0"),T(t,"viewportEl",c);let m=document.createElement("slot");c.appendChild(m),f.appendChild(c);let p=document.createElement("div");p.setAttribute("class","scrollbar scrollbar-v"),p.setAttribute("part","scrollbar-v"),p.setAttribute("aria-hidden","true"),T(t,"vScrollbar",p);let l=document.createElement("div");l.setAttribute("class","thumb"),T(t,"vThumb",l),p.appendChild(l),f.appendChild(p);let d=document.createElement("div");d.setAttribute("class","scrollbar scrollbar-h"),d.setAttribute("part","scrollbar-h"),d.setAttribute("aria-hidden","true"),T(t,"hScrollbar",d);let a=document.createElement("div");if(a.setAttribute("class","thumb"),T(t,"hThumb",a),d.appendChild(a),f.appendChild(d),i!=null){let s=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(s){let u=new Proxy(t,{get(b,h){return b[h]},set(b,h,v){let g=b[h];if(g!=null&&typeof g.set==="function"){if(typeof v==="function")g.set(()=>v);else g.set(v);return!0}return!1}});if(s.mount){let b=s.mount(u,r);if(typeof b==="function")o.push(b)}if(s.unmount)o.push(()=>s.unmount(u))}}return t}});var ye=P({mount(n,e){let r=n;if(!e.hasAttribute("role"))e.setAttribute("role","toolbar");if(!e.hasAttribute("aria-orientation"))e.setAttribute("aria-orientation","horizontal");let o=()=>{return(r.defaultSlot()?.assignedElements({flatten:!0})??[]).filter((c)=>!c.hasAttribute("disabled"))},t=(i)=>{if(i.key!=="ArrowRight"&&i.key!=="ArrowLeft")return;let f=o();if(!f.length)return;let c=i.composedPath().find((l)=>f.includes(l))||null,m=c?f.indexOf(c):-1,p=i.key==="ArrowRight"?(m+1)%f.length:(m-1+f.length)%f.length;f[p]?.focus(),i.preventDefault()};return e.addEventListener("keydown",t),()=>e.removeEventListener("keydown",t)}});var Zv=z("sui-toolbar",{props:{justify:y.union(["start","center","end","between"],{default:"start",reflect:!0}),align:y.union(["center","start","end","baseline"],{default:"center",reflect:!0}),gap:y.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:y.union(["default","bordered"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.justify=e.justify,t.align=e.align,t.gap=e.gap,t.variant=e.variant;let i=null;if(ye&&typeof ye.setup==="function"){if(n.scope=t,i=ye.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("slot");c.setAttribute("name","spacer"),f.appendChild(c);let m=document.createElement("slot");if(T(t,"defaultSlot",m),f.appendChild(m),i!=null){let p=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(d,a){return d[a]},set(d,a,s){let u=d[a];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(p.mount){let d=p.mount(l,r);if(typeof d==="function")o.push(d)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});var lo={dashboard:"layout-dashboard",pipelines:"workflow",agents:"bot",skills:"sparkles",git:"git-branch",collections:"database",canvas:"frame",settings:"settings",chat:"messages-square",events:"radio",specs:"file-text",theme:"palette",setup:"rocket",overview:"compass",mesh:"network"},co={"theme-studio":"theme"},Xi=new Set([...Object.keys(lo),...Object.keys(co)]),xe=P({mount(n,e){let r=n,o=D(()=>{let t=r.name(),i=r.label(),f=co[t]??t,m=Xi.has(t)?lo[f]:"dot",p=r.glyphEl();if(p)p.setAttribute("name",m);if(i)e.setAttribute("role","img"),e.setAttribute("aria-label",i),e.removeAttribute("aria-hidden");else e.setAttribute("aria-hidden","true"),e.removeAttribute("role"),e.removeAttribute("aria-label")});return()=>o()}});var _v=z("sui-app-icon",{props:{name:y.string({default:"",reflect:!0}),variant:y.union(["glyph","tile","solid"],{default:"glyph",reflect:!0}),size:y.union(["sm","md","lg"],{default:"md",reflect:!0}),radius:y.string({default:"",reflect:!0}),label:y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.name=e.name,t.variant=e.variant,t.size=e.size,t.radius=e.radius,t.label=e.label;let i=null;if(xe&&typeof xe.setup==="function"){if(n.scope=t,i=xe.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("span");c.setAttribute("class","frame"),c.setAttribute("part","frame");let m=document.createElement("slot"),p=document.createElement("sui-icon");if(T(t,"glyphEl",p),p.setAttribute("part","glyph"),p.setAttribute("aria-hidden","true"),m.appendChild(p),c.appendChild(m),f.appendChild(c),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var we=P({scope:()=>{return{nodes:M([])}},mount(n,e){let r=e.nodes;if(r!=null)n.nodes.set(r);if(Object.defineProperty(e,"nodes",{configurable:!0,get:()=>n.nodes(),set:(o)=>n.nodes.set(o)}),!e.hasAttribute("role"))e.setAttribute("role","tree")}});var ig=z("sui-tree-view",{shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
  }
  ul { list-style: none; margin: 0; padding: 0; }
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null),i=null;if(we&&typeof we.setup==="function"){if(n.scope=t,i=we.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(l)=>R({anchor:l,items:()=>t.nodes(),key:(d,a)=>{let s=Object.create(t);return s.n=d,s.__i=a,((u)=>d.id)(s)},render:(d,a)=>{let s=Object.create(t);return s.n=d,s.__i=a,s.__v=M(0),((u)=>{let b=document.createDocumentFragment(),h=[],v=document.createElement("sui-tree-item");return h.push(tt(v,"node",()=>(u.__v(),u.n))),b.appendChild(v),{nodes:Array.from(b.childNodes),cleanup:()=>{for(let g of h)try{g()}catch(x){}},update:(g,x)=>{u.n=g,u.__i=x,u.__v.set((A)=>A+1)}}})(s)}}),c=(l)=>B({anchor:l,test:()=>{if(t.nodes().length>0)return 0;return 1},branches:[()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("ul");s.setAttribute("role","tree");let u=document.createComment("for");return s.appendChild(u),a.push(f(u)),d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let b of a)try{b()}catch(h){}}}},()=>{let d=document.createDocumentFragment(),a=[],s=document.createElement("slot");return d.appendChild(s),{nodes:Array.from(d.childNodes),cleanup:()=>{for(let u of a)try{u()}catch(b){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createComment("if");if(m.appendChild(p),o.push(c(p)),i!=null){let l=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(l){let d=new Proxy(t,{get(a,s){return a[s]},set(a,s,u){let b=a[s];if(b!=null&&typeof b.set==="function"){if(typeof u==="function")b.set(()=>u);else b.set(u);return!0}return!1}});if(l.mount){let a=l.mount(d,r);if(typeof a==="function")o.push(a)}if(l.unmount)o.push(()=>l.unmount(d))}}return t}});var Ae=P({scope:({label:n})=>{let e=M(null),r=Z(()=>{let i=e();return i?i.label:n()}),o=Z(()=>{return e()?.children??[]}),t=J(null);return{node:e,nodeLabel:r,nodeChildren:o,rowEl:t}},mount(n,e){if(!e.hasAttribute("role"))e.setAttribute("role","treeitem");let r=e.node;if(r!=null)n.node.set(r);Object.defineProperty(e,"node",{configurable:!0,get:()=>n.node(),set:(l)=>n.node.set(l)});let o=D(()=>{let l=n.node();if(l!==null)if(!l.children||l.children.length===0)e.setAttribute("leaf","");else e.removeAttribute("leaf")}),t=D(()=>{let l=n.rowEl();if(!l)return;if(!n.leaf())l.setAttribute("aria-expanded",String(n.expanded()));else l.removeAttribute("aria-expanded");l.setAttribute("aria-selected",String(n.selected()))}),i=(l)=>{if(n.leaf())return;l.stopPropagation();let d=!n.expanded();n.expanded.set(d),e.dispatchEvent(new CustomEvent("sui-toggle",{detail:{expanded:d},bubbles:!0,composed:!0}))},f=()=>{e.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},c=(l)=>{f(),i(l)},m=(l)=>{if(l.key==="Enter"||l.key===" ")l.preventDefault(),f();else if(l.key==="ArrowRight"&&!n.leaf()&&!n.expanded())l.preventDefault(),i(l);else if(l.key==="ArrowLeft"&&n.expanded())l.preventDefault(),i(l)},p=D(()=>{let l=n.rowEl();if(!l)return;return l.addEventListener("click",c),l.addEventListener("keydown",m),()=>{l.removeEventListener("click",c),l.removeEventListener("keydown",m)}});return()=>{o(),t(),p()}}});var cg=z("sui-tree-item",{props:{label:y.string({default:"",reflect:!0}),expanded:y.bool({default:!1,reflect:!0}),selected:y.bool({default:!1,reflect:!0}),leaf:y.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.label=e.label,t.expanded=e.expanded,t.selected=e.selected,t.leaf=e.leaf;let i=null;if(Ae&&typeof Ae.setup==="function"){if(n.scope=t,i=Ae.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(A)=>R({anchor:A,items:()=>t.nodeChildren(),key:(w,E)=>{let j=Object.create(t);return j.c=w,j.__i=E,((C)=>w.id)(j)},render:(w,E)=>{let j=Object.create(t);return j.c=w,j.__i=E,j.__v=M(0),((C)=>{let N=document.createDocumentFragment(),O=[],S=document.createElement("sui-tree-item");return O.push(tt(S,"node",()=>(C.__v(),C.c))),N.appendChild(S),{nodes:Array.from(N.childNodes),cleanup:()=>{for(let H of O)try{H()}catch(U){}},update:(H,U)=>{C.c=H,C.__i=U,C.__v.set((G)=>G+1)}}})(j)}}),c=(A)=>B({anchor:A,test:()=>{if(t.nodeChildren().length>0)return 0;return 1},branches:[()=>{let w=document.createDocumentFragment(),E=[],j=document.createComment("for");return w.appendChild(j),E.push(f(j)),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of E)try{C()}catch(N){}}}},()=>{let w=document.createDocumentFragment(),E=[],j=document.createElement("slot");return w.appendChild(j),{nodes:Array.from(w.childNodes),cleanup:()=>{for(let C of E)try{C()}catch(N){}}}}]}),m=r.shadowRoot??r;L(m);let p=document.createElement("div");p.setAttribute("class","row"),p.setAttribute("part","row"),p.setAttribute("tabindex","0"),T(t,"rowEl",p);let l=document.createElement("span");l.setAttribute("class","caret"),l.setAttribute("part","caret"),l.setAttribute("aria-hidden","true");let d=document.createElement("svg");d.setAttribute("viewbox","0 0 12 12"),d.setAttribute("fill","none"),d.setAttribute("stroke","currentColor"),d.setAttribute("stroke-width","1.5");let a=document.createElement("polyline");a.setAttribute("points","4 3 8 6 4 9"),d.appendChild(a),l.appendChild(d),p.appendChild(l);let s=document.createElement("slot");s.setAttribute("name","icon"),p.appendChild(s);let u=document.createElement("span");u.setAttribute("class","label"),u.setAttribute("part","label");let b=document.createTextNode("");u.appendChild(b),o.push(F(b,()=>t.nodeLabel())),p.appendChild(u);let h=document.createElement("span");h.setAttribute("class","trailing");let v=document.createElement("slot");v.setAttribute("name","trailing"),h.appendChild(v),p.appendChild(h),m.appendChild(p);let g=document.createElement("div");g.setAttribute("class","children"),g.setAttribute("part","children"),g.setAttribute("role","group");let x=document.createComment("if");if(g.appendChild(x),o.push(c(x)),m.appendChild(g),i!=null){let A=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(A){let w=new Proxy(t,{get(E,j){return E[j]},set(E,j,C){let N=E[j];if(N!=null&&typeof N.set==="function"){if(typeof C==="function")N.set(()=>C);else N.set(C);return!0}return!1}});if(A.mount){let E=A.mount(w,r);if(typeof E==="function")o.push(E)}if(A.unmount)o.push(()=>A.unmount(w))}}return t}});var po=new WeakSet;function qi(n){if(po.has(n))return;po.add(n);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let r=new CSSStyleSheet;r.replaceSync(fo),n.adoptedStyleSheets=[...n.adoptedStyleSheets,r];return}}catch{}let e=n.createElement("style");e.setAttribute("data-sui-table",""),e.textContent=fo,n.head.appendChild(e)}var Ee=P({scope:()=>{let n=M([]),e=M([]),r=M(null),o=M("asc"),t=M([]),i=Z(()=>{let f=r(),c=o(),m=n();if(!f)return m;return[...m].sort((p,l)=>{let d=String(p[f]??""),a=String(l[f]??""),s=d<a?-1:d>a?1:0;return c==="asc"?s:-s})});return{rows:n,columns:e,sortKey:r,sortDir:o,selected:t,sortedRows:i}},on:{sortBy(n,e){if(n.sortKey===e)n.sortDir=n.sortDir==="asc"?"desc":"asc";else n.sortKey=e,n.sortDir="asc"},toggleSelect(n,e){let r=n.selected;n.selected=r.includes(e)?r.filter((o)=>o!==e):[...r,e]},selectAll(n){n.selected=n.rows.map((e)=>e.id)},clearSelection(n){n.selected=[]}},mount(n,e){let r=e.rows;if(r!=null)n.rows.set(r);let o=e.columns;if(o!=null)n.columns.set(o);Object.defineProperty(e,"rows",{configurable:!0,get:()=>n.rows(),set:(a)=>n.rows.set(a)}),Object.defineProperty(e,"columns",{configurable:!0,get:()=>n.columns(),set:(a)=>n.columns.set(a)}),qi(e.ownerDocument);let t=()=>{e.querySelectorAll("th[data-sort-key]").forEach((a)=>{let s=a.getAttribute("aria-sort")||"none",u=a.querySelector(".sui-sort-indicator");if(!u)return;if(s==="ascending")u.style.transform="rotate(180deg)",u.style.opacity="1";else if(s==="descending")u.style.transform="rotate(0deg)",u.style.opacity="1";else u.style.transform="rotate(0deg)",u.style.opacity="0.4"})},i=()=>{e.querySelectorAll("th[data-sort-key]").forEach((a)=>{if(!a.hasAttribute("aria-sort"))a.setAttribute("aria-sort","none");if(!a.hasAttribute("tabindex"))a.setAttribute("tabindex","0");if(!a.hasAttribute("role"))a.setAttribute("role","columnheader");if(!a.querySelector(".sui-sort-indicator")){let s=document.createElement("span");s.className="sui-sort-indicator",s.setAttribute("aria-hidden","true"),s.style.cssText="margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;";let u="http://www.w3.org/2000/svg",b=document.createElementNS(u,"svg");b.setAttribute("width","10"),b.setAttribute("height","10"),b.setAttribute("viewBox","0 0 10 10"),b.setAttribute("fill","none"),b.setAttribute("stroke","currentColor"),b.setAttribute("stroke-width","1.5"),b.setAttribute("stroke-linecap","round"),b.setAttribute("stroke-linejoin","round"),b.setAttribute("aria-hidden","true");let h=document.createElementNS(u,"path");h.setAttribute("d","M2 4l3-2 3 2M2 6l3 2 3-2"),b.appendChild(h),s.appendChild(b),a.appendChild(s)}}),t()},f=()=>{let a=e.querySelector(":scope > table tbody > tr")||e.querySelector(":scope > table > tr:not(:has(th))")||e.querySelector(":scope > tbody > tr")||e.querySelector(':scope > tr:not([slot="head"])'),s=e.querySelector('[slot="empty"]')!=null;if(!a&&s)e.setAttribute("data-empty","");else e.removeAttribute("data-empty")},c=(a)=>{let s=a.getAttribute("data-sort-key")||"",u=a.getAttribute("aria-sort")||"none",b=u==="none"?"ascending":u==="ascending"?"descending":"none";e.querySelectorAll("th[data-sort-key]").forEach((h)=>{if(h!==a)h.setAttribute("aria-sort","none")}),a.setAttribute("aria-sort",b),t(),e.emit("sui-sort",{key:s,direction:b})},m=(a)=>{let s=a.target.closest("th[data-sort-key]");if(!s||!e.contains(s))return;c(s)},p=(a)=>{if(a.key!=="Enter"&&a.key!==" ")return;let s=a.target.closest("th[data-sort-key]");if(!s||!e.contains(s))return;a.preventDefault(),c(s)},l=(a)=>{let s=a.target.closest("td");if(!s||!e.contains(s))return;if(s.classList.contains("sui-table-gutter-cell"))return;let u=s.closest("tr");if(!u)return;let b=u.getAttribute("data-row-id");if(!b)return;e.emit("sui-row-click",{id:b,index:u.rowIndex})};Q(e,"click",m),Q(e,"click",l),Q(e,"keydown",p),i(),f();let d=new MutationObserver(()=>{i(),f()});return d.observe(e,{childList:!0,subtree:!0}),()=>{d.disconnect()}}}),fo=`
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
`;var vg=z("sui-table",{props:{variant:y.union(["slot","native","data"],{default:"slot",reflect:!0}),density:y.union(["compact","default","comfortable"],{default:"default",reflect:!0}),dense:y.bool({default:!1,reflect:!0}),hoverable:y.bool({default:!0,reflect:!0}),striped:y.bool({default:!1,reflect:!0}),"sticky-header":y.bool({default:!1,reflect:!0}),"sticky-first-col":y.bool({default:!1,reflect:!0}),gutter:y.bool({default:!1,reflect:!0}),label:y.string({default:""})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.variant=e.variant,t.density=e.density,t.dense=e.dense,t.hoverable=e.hoverable,t.striped=e.striped,t.stickyHeader=e.stickyHeader,t.stickyFirstCol=e.stickyFirstCol,t.gutter=e.gutter,t.label=e.label;let i=null;if(Ee&&typeof Ee.setup==="function"){if(n.scope=t,i=Ee.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=(d)=>R({anchor:d,items:()=>t.columns(),key:(a,s)=>{let u=Object.create(t);return u.c=a,u.__i=s,((b)=>a.field)(u)},render:(a,s)=>{let u=Object.create(t);return u.c=a,u.__i=s,u.__v=M(0),((b)=>{let h=document.createDocumentFragment(),v=[],g=document.createElement("th");v.push(K(g,"click",function(){let A=()=>b.sortBy(b.c.field);return typeof A==="function"?A.bind(void 0):()=>{}}(),{modifiers:[]})),v.push(pt(g,"sorted",()=>(b.__v(),b.sortKey()===b.c.field)));let x=document.createTextNode("");return g.appendChild(x),v.push(F(x,()=>(b.__v(),b.c.label))),h.appendChild(g),{nodes:Array.from(h.childNodes),cleanup:()=>{for(let A of v)try{A()}catch(w){}},update:(A,w)=>{b.c=A,b.__i=w,b.__v.set((E)=>E+1)}}})(u)}}),c=(d)=>R({anchor:d,items:()=>t.sortedRows(),key:(a,s)=>{let u=Object.create(t);return u.r=a,u.__i=s,((b)=>a.id)(u)},render:(a,s)=>{let u=Object.create(t);return u.r=a,u.__i=s,u.__v=M(0),((b)=>{let h=(w)=>R({anchor:w,items:()=>b.columns(),key:(E,j)=>{let C=Object.create(b);return C.c=E,C.__i=j,((N)=>E.field)(C)},render:(E,j)=>{let C=Object.create(b);return C.c=E,C.__i=j,C.__v=M(0),((N)=>{let O=document.createDocumentFragment(),S=[],H=document.createElement("td"),U=document.createTextNode("");return H.appendChild(U),S.push(F(U,()=>(N.__v(),N.r[N.c.field]))),O.appendChild(H),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let G of S)try{G()}catch(X){}},update:(G,X)=>{N.c=G,N.__i=X,N.__v.set((Y)=>Y+1)}}})(C)}}),v=document.createDocumentFragment(),g=[],x=document.createElement("tr");g.push(pt(x,"selected",()=>(b.__v(),b.selected().includes(b.r.id)))),g.push(K(x,"click",function(){let w=()=>b.toggleSelect(b.r.id);return typeof w==="function"?w.bind(void 0):()=>{}}(),{modifiers:[]}));let A=document.createComment("for");return x.appendChild(A),g.push(h(A)),v.appendChild(x),{nodes:Array.from(v.childNodes),cleanup:()=>{for(let w of g)try{w()}catch(E){}},update:(w,E)=>{b.r=w,b.__i=E,b.__v.set((j)=>j+1)}}})(u)}}),m=(d)=>B({anchor:d,test:()=>{if(t.variant()==="data")return 0;if(t.variant()!=="native")return 1;return 2},branches:[()=>{let a=document.createDocumentFragment(),s=[],u=document.createElement("div");u.setAttribute("class","wrap"),u.setAttribute("part","wrap");let b=document.createElement("table");b.setAttribute("part","table"),s.push(k(b,"aria-label",()=>t.label()||null));let h=document.createElement("thead");h.setAttribute("part","thead");let v=document.createElement("tr"),g=document.createComment("for");v.appendChild(g),s.push(f(g)),h.appendChild(v),b.appendChild(h);let x=document.createElement("tbody");x.setAttribute("part","tbody");let A=document.createComment("for");return x.appendChild(A),s.push(c(A)),b.appendChild(x),u.appendChild(b),a.appendChild(u),{nodes:Array.from(a.childNodes),cleanup:()=>{for(let w of s)try{w()}catch(E){}}}},()=>{let a=document.createDocumentFragment(),s=[],u=document.createElement("div");u.setAttribute("class","wrap"),u.setAttribute("part","wrap");let b=document.createElement("table");b.setAttribute("part","table"),s.push(k(b,"aria-label",()=>t.label()||null));let h=document.createElement("thead");h.setAttribute("part","thead");let v=document.createElement("slot");v.setAttribute("name","head"),h.appendChild(v),b.appendChild(h);let g=document.createElement("tbody");g.setAttribute("part","tbody");let x=document.createElement("slot");g.appendChild(x),b.appendChild(g),u.appendChild(b);let A=document.createElement("div");A.setAttribute("class","empty"),A.setAttribute("part","empty"),A.setAttribute("role","status");let w=document.createElement("slot");return w.setAttribute("name","empty"),A.appendChild(w),u.appendChild(A),a.appendChild(u),{nodes:Array.from(a.childNodes),cleanup:()=>{for(let E of s)try{E()}catch(j){}}}},()=>{let a=document.createDocumentFragment(),s=[],u=document.createElement("div");u.setAttribute("class","wrap"),u.setAttribute("part","wrap");let b=document.createElement("slot");u.appendChild(b);let h=document.createElement("div");h.setAttribute("class","empty"),h.setAttribute("part","empty"),h.setAttribute("role","status");let v=document.createElement("slot");return v.setAttribute("name","empty"),h.appendChild(v),u.appendChild(h),a.appendChild(u),{nodes:Array.from(a.childNodes),cleanup:()=>{for(let g of s)try{g()}catch(x){}}}}]}),p=r.shadowRoot??r;L(p);let l=document.createComment("if");if(p.appendChild(l),o.push(m(l)),i!=null){let d=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(d){let a=new Proxy(t,{get(s,u){return s[u]},set(s,u,b){let h=s[u];if(h!=null&&typeof h.set==="function"){if(typeof b==="function")h.set(()=>b);else h.set(b);return!0}return!1}});if(d.mount){let s=d.mount(a,r);if(typeof s==="function")o.push(s)}if(d.unmount)o.push(()=>d.unmount(a))}}return t}});function rt(n){return String(n).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#39;")}function mo(n,e){if(Array.isArray(n))return n;if(typeof n==="string"&&n.trim())try{let r=JSON.parse(n);return Array.isArray(r)?r:e}catch{return e}return e}function Ri(n,e,r,o){let t=new Map,i=n.some((a)=>typeof a.x!=="number"||typeof a.y!=="number");if(o==="positioned"&&!i){for(let a of n)t.set(a.id,{x:a.x,y:a.y});return t}let f=e*0.22,c=r*0.5,m=n.filter((a)=>a.kind==="agent"),p=n.filter((a)=>a.kind!=="agent");m.forEach((a,s)=>{let u=Math.max(1,m.length),b=c+(s-(u-1)/2)*80;t.set(a.id,{x:f,y:b})});let l=new Map;for(let a of p){let s=l.get(a.kind)??[];s.push(a),l.set(a.kind,s)}let d=Array.from(l.keys());return d.forEach((a,s)=>{let u=l.get(a),b=e*(0.5+(s+1)/(d.length+1)*0.45);u.forEach((h,v)=>{let g=r*(v+1)/(u.length+1);t.set(h.id,{x:b,y:g})})}),t}function Ji(n,e){let r=Math.max(40,Math.abs(e.x-n.x)*0.4);return`M ${n.x} ${n.y} C ${n.x+r} ${n.y}, ${e.x-r} ${e.y}, ${e.x} ${e.y}`}function Wi(n,e){let r=n.querySelectorAll('template[slot="node"]');for(let o of Array.from(r))if(o.dataset.kind===e)return o;return null}function Yi(n,e){let r=Wi(n,e.kind);if(!r){let o=e.sub?`<div class="node-sub">${rt(e.sub)}</div>`:"";return`<div class="node-h"><span class="tag">${rt(e.kind)}</span></div><div class="node-title">${rt(e.title??e.id)}</div>${o}`}return r.innerHTML.replace(/\$\{title\}/g,rt(e.title??"")).replace(/\$\{sub\}/g,rt(e.sub??"")).replace(/\$\{id\}/g,rt(e.id))}function Zi(n,e,r,o,t,i,f){let c=mo(e,[]),m=mo(r,[]),p=Ri(c,o,t,i),l=m.map((a)=>{let s=p.get(a.from),u=p.get(a.to);if(!s||!u)return"";let b=rt(Ji(s,u)),h=a.dashed?" data-dashed":"";return`<path d="${b}" data-kind="${rt(a.kind??"default")}"${h}></path>`}).join(""),d=c.map((a)=>{let s=p.get(a.id);if(!s)return"";let u=s.x/o*100,b=s.y/t*100,h=f&&f===a.id?" data-selected":"",v=a.w?`--node-w: ${a.w}px;`:"",g=rt(`${a.kind}: ${a.title??a.id}`),x=Yi(n,a);return`<div class="node" part="node" data-kind="${rt(a.kind)}" data-id="${rt(a.id)}"${h} tabindex="0" role="button" aria-label="${g}" style="left: ${u}%; top: ${b}%;${v}">${x}</div>`}).join("");return`<svg class="edges" part="edges" viewBox="0 0 ${o} ${t}" preserveAspectRatio="none" aria-hidden="true">${l}</svg>`+d}var ke=P({mount(n,e){let r=n,o=r.canvasEl();if(!o)return;let t=o.ownerDocument.createElement("div");o.appendChild(t);let i=et(t,()=>{let l=r.nodes(),d=r.edges(),a=r.width()||1000,s=r.height()||600,u=r.layout()||"positioned",b=r.selectedId();return Zi(e,l,d,a,s,u,b)}),f="",c=(l)=>{let d=l.target.closest(".node[data-id]");if(!d)return;let a=d.getAttribute("data-id")??"",s=d.getAttribute("data-kind")??"";e.setAttribute("selected-id",a),e.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:a,kind:s},bubbles:!0,composed:!0}))},m=(l)=>{if(l.key!=="Enter"&&l.key!==" ")return;let d=l.target.closest(".node[data-id]");if(!d)return;l.preventDefault();let a=d.getAttribute("data-id")??"",s=d.getAttribute("data-kind")??"";e.setAttribute("selected-id",a),e.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:a,kind:s},bubbles:!0,composed:!0}))},p=(l)=>{let d=l.target.closest(".node[data-id]");if(!d){f="";return}let a=d.getAttribute("data-id")??"";if(a===f)return;f=a;let s=d.getAttribute("data-kind")??"";e.dispatchEvent(new CustomEvent("sui-node-hover",{detail:{id:a,kind:s},bubbles:!0,composed:!0}))};return o.addEventListener("click",c),o.addEventListener("keydown",m),o.addEventListener("mouseover",p),()=>{i(),o.removeEventListener("click",c),o.removeEventListener("keydown",m),o.removeEventListener("mouseover",p)}}});var Eg=z("sui-graph-canvas",{props:{nodes:y.string({default:""}),edges:y.string({default:""}),width:y.number({default:1000,reflect:!0}),height:y.number({default:600,reflect:!0}),layout:y.union(["positioned","radial"],{default:"positioned",reflect:!0}),"selected-id":y.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(n){let{props:e,host:r}=n;if(!n.cleanups)n.cleanups=[];let o=n.cleanups,t=Object.create(null);t.nodes=e.nodes,t.edges=e.edges,t.width=e.width,t.height=e.height,t.layout=e.layout,t.selectedId=e.selectedId;let i=null;if(ke&&typeof ke.setup==="function"){if(n.scope=t,i=ke.setup(n),i&&typeof i==="object")Object.assign(t,i)}let f=r.shadowRoot??r;L(f);let c=document.createElement("div");c.setAttribute("class","canvas"),c.setAttribute("part","canvas"),T(t,"canvasEl",c),f.appendChild(c);let m=document.createElement("slot");if(m.setAttribute("name","node"),m.setAttribute("hidden",""),f.appendChild(m),i!=null){let p=Object.getOwnPropertyDescriptor(i,"__shipSetLifecycle")?.value;if(p){let l=new Proxy(t,{get(d,a){return d[a]},set(d,a,s){let u=d[a];if(u!=null&&typeof u.set==="function"){if(typeof s==="function")u.set(()=>s);else u.set(s);return!0}return!1}});if(p.mount){let d=p.mount(l,r);if(typeof d==="function")o.push(d)}if(p.unmount)o.push(()=>p.unmount(l))}}return t}});
