var iu=Symbol.for("sui:error-boundary");function uu(i,u){i[iu]=u}function du(i){delete i[iu]}function Pn(i){let u=i;while(u){if(u instanceof Element){let n=u[iu];if(n)return n}let d=u.parentNode;if(d){u=d;continue}let m=u;if(m.host){u=m.host;continue}u=null}return null}function Gn(i,u,d){try{return d()}catch(m){let n=Pn(i);if(n){n.catch(m,u);return}console.error("[sui] uncaught throw (no error boundary):",m);return}}var yn=null,Cn=null,Bd=0,Id=new Set,Kn=[];function mu(i,u){Kn.push(i);try{return u()}finally{Kn.pop()}}function Rd(){return Kn.length>0?Kn[Kn.length-1]:null}function qu(i){if(yn===null)return;if(i.has(yn))return;let u=yn;if(i.add(u),Cn!==null)Cn.push(()=>i.delete(u))}function G(i){let u=i,d=new Set,m=()=>{return qu(d),u};return m.set=(n)=>{let b=typeof n==="function"?n(u):n;if(Object.is(b,u))return;if(u=b,Bd>0)for(let w of d)Id.add(w);else for(let w of[...d])w()},m.__subs=d,m.__peek=()=>u,m}function h(i){let u,d=!0,m=new Set,n=[],b=()=>{if(d)return;if(d=!0,m.size===0)return;let M=u;if(w(),Object.is(M,u))return;for(let y of[...m])y()},w=()=>{for(let l of n)l();n=[];let M=yn,y=Cn;yn=b,Cn=n;try{u=i()}finally{yn=M,Cn=y}d=!1},r=()=>{if(qu(m),d)w();return u};return r.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},r.__subs=m,r.__peek=()=>{if(d)w();return u},r}function U(i){let u=void 0,d=!0,m=[],n=Rd(),b=()=>{if(!d)return;if(u){try{u()}catch(M){console.error("[sui] effect cleanup threw:",M)}u=void 0}for(let M of m)M();m=[];let w=yn,r=Cn;yn=b,Cn=m;try{u=i()}catch(M){let y=n?Pn(n):null;if(y)y.catch(M,"effect");else console.error("[sui] effect threw:",M)}finally{yn=w,Cn=r}};return b(),()=>{d=!1;for(let w of m)w();if(m=[],u){try{u()}catch(w){console.error("[sui] effect cleanup threw:",w)}u=void 0}}}var pn=null;function Bn(i,u){let d=pn;pn=i;try{return u()}finally{pn=d}}function wn(i){let u=pn;if(!u)return;u.cleanup(i)}function In(){let i=[],u=!1;return{cleanup(m){if(u){try{m()}catch(n){console.error("[sui/scope] late cleanup threw:",n)}return}i.push(m)},dispose(){if(u)return;u=!0;for(let m=i.length-1;m>=0;m--)try{i[m]()}catch(n){console.error("[sui/scope] cleanup threw:",n)}i.length=0},get disposed(){return u}}}var kd=Symbol("sui.shared");function Yu(i){return!!i?.[kd]}function bu(i,u,d){i[`__sui_ctx_${u}`]=d}function fu(i){let u={};for(let d in i)if(Object.prototype.hasOwnProperty.call(i,d)&&Yu(i[d]))u[d]=i[d];return u}function e(i=null){return G(null)}function F(i,u,d){let m=i[u];if(typeof m==="function"&&typeof m.set==="function")return m.set(d),m;let n=G(d);return i[u]=n,n}function bn(i,u,d){let m=()=>{if(i.assignedNodes({flatten:!0}).some(Sd))u.removeAttribute(d);else u.setAttribute(d,"")};return i.addEventListener("slotchange",m),m(),()=>{i.removeEventListener("slotchange",m)}}function Sd(i){if(i.nodeType===Node.TEXT_NODE)return(i.textContent??"").trim().length>0;if(i.nodeType===Node.ELEMENT_NODE){let u=i;return(u.textContent??"").trim().length>0||u.childElementCount>0}return!1}function Rn(i,u,d={}){if(typeof u==="string"){let m=i.tagName.toLowerCase(),n=i.namespaceURI==="http://www.w3.org/2000/svg"?i.ownerDocument.createElementNS("http://www.w3.org/2000/svg",m):i.ownerDocument.createElement(m);n.innerHTML=u,Nu(i,n,d);return}Uu(i,u,d)}function Uu(i,u,d){if(d.skip?.(i))return;if(i.tagName!==u.tagName){i.replaceWith(u.cloneNode(!0));return}ad(i,u),Nu(i,u,d)}function ad(i,u){let d=Array.from(i.attributes);for(let n of d)if(!u.hasAttribute(n.name))i.removeAttribute(n.name);let m=Array.from(u.attributes);for(let n of m)if(i.getAttribute(n.name)!==n.value)i.setAttribute(n.name,n.value);if(i instanceof HTMLInputElement&&u instanceof HTMLInputElement){if(u.hasAttribute("value")&&i.value!==u.value)i.value=u.value;if(i.checked!==u.checked)i.checked=u.checked}}function Nu(i,u,d){let m=new Map;for(let r of Array.from(i.children))if(r.id)m.set(r.id,r);let n=new Set;for(let r of Array.from(u.children))if(r.id)n.add(r.id);let b=Array.from(u.childNodes),w=i.firstChild;for(let r of b){while(w&&w.nodeType===Node.ELEMENT_NODE&&w.id&&n.has(w.id)&&!(r.nodeType===Node.ELEMENT_NODE&&r.id===w.id))w=w.nextSibling;let M=null;if(r.nodeType===Node.ELEMENT_NODE){let y=r.id;if(y&&m.has(y))M=m.get(y),m.delete(y)}if(!M&&w&&vd(w,r))M=w;if(M){if(M===w)w=w.nextSibling;else i.insertBefore(M,w);xd(M,r,i,d)}else i.insertBefore(r.cloneNode(!0),w)}while(w){let r=w.nextSibling;i.removeChild(w),w=r}}function vd(i,u){if(i.nodeType!==u.nodeType)return!1;if(i.nodeType===Node.ELEMENT_NODE)return i.tagName===u.tagName;return!0}function xd(i,u,d,m){if(i.nodeType!==u.nodeType){d.replaceChild(u.cloneNode(!0),i);return}if(i.nodeType===Node.TEXT_NODE||i.nodeType===Node.COMMENT_NODE){if(i.nodeValue!==u.nodeValue)i.nodeValue=u.nodeValue;return}if(i.nodeType===Node.ELEMENT_NODE)Uu(i,u,m)}function k(i,u){let d;return U(()=>{let m=u(),n=m==null?"":String(m);if(n===d)return;if(d===void 0&&i.textContent===n){d=n;return}i.textContent=n,d=n})}function T(i,u,d){let m,n=!1;return U(()=>{let b=d();if(n&&b===m)return;let w=b==null||b===!1||b===""?null:b===!0?"":String(b);if(n||i.getAttribute(u)!==w)if(w===null)i.removeAttribute(u);else i.setAttribute(u,w);m=b,n=!0})}function zn(i,u,d){let m;return U(()=>{let n=!!d();if(n===m)return;if(m===void 0&&n===i.classList.contains(u)){m=n;return}if(n)i.classList.add(u);else i.classList.remove(u);m=n})}function Wn(i,u,d){return U(()=>{i[u]=d()})}function gn(i,u,d){let m=U(()=>{let b=u();if(i.value!==b)i.value=b}),n=()=>{let b=i.value;if(u.set(b),d)d(b)};return i.addEventListener("input",n),()=>{m(),i.removeEventListener("input",n)}}function Dn(i,u){let d=U(()=>{let n=u();if(i.checked!==n)i.checked=n}),m=()=>u.set(i.checked);return i.addEventListener("change",m),()=>{d(),i.removeEventListener("change",m)}}function fn(i,u){let m=i.namespaceURI==="http://www.w3.org/2000/svg",n=i.tagName.toLowerCase();return U(()=>{let b=u(),w=m?i.ownerDocument.createElementNS("http://www.w3.org/2000/svg",n):i.ownerDocument.createElement(n);w.innerHTML=b,Rn(i,w)})}function p(i,u,d,m={}){let n=m.modifiers??[],b,w=!0,r=(y)=>{for(let t of n)switch(t.kind){case"preventDefault":y.preventDefault();break;case"stopPropagation":y.stopPropagation();break;case"stopImmediatePropagation":y.stopImmediatePropagation();break;case"self":if(y.target!==i)return;break;case"trusted":if(!y.isTrusted)return;break;case"key":{if(!Fu(y,t.arg??""))return;break}}let l=n.find((t)=>t.kind==="debounce"),E=n.find((t)=>t.kind==="throttle");if(l){let t=Number(l.arg)||0;if(b!==void 0)clearTimeout(b);b=setTimeout(()=>{b=void 0,d(y)},t);return}if(E){let t=Number(E.arg)||0;if(!w)return;w=!1,d(y),b=setTimeout(()=>{w=!0},t);return}d(y)},M={};if(m.capture)M.capture=!0;if(m.passive)M.passive=!0;if(m.once)M.once=!0;return i.addEventListener(u,r,M),()=>{if(i.removeEventListener(u,r,M),b!==void 0)clearTimeout(b)}}function Fu(i,u){let d=u.split("+").map((l)=>l.toLowerCase()),m=d.pop()??"",n=d.includes("meta")||d.includes("cmd"),b=d.includes("ctrl")||d.includes("control"),w=d.includes("shift"),r=d.includes("alt")||d.includes("option"),M=d.includes("mod"),y=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(n&&!i.metaKey)return!1;if(b&&!i.ctrlKey)return!1;if(w&&!i.shiftKey)return!1;if(r&&!i.altKey)return!1;if(M&&!(y?i.metaKey:i.ctrlKey))return!1;return i.key.toLowerCase()===m}function Ou(i,u,d){i.dispatchEvent(new CustomEvent(u,{detail:d,bubbles:!0,composed:!0}))}function Au(i,u){let d=i,m=Object.keys(u);for(let n of m)d[n]=u[n];wn(()=>{for(let n of m)if(d[n]===u[n])try{delete d[n]}catch{d[n]=void 0}})}function cn(i){let u=i;u.emit=function(d,m){Ou(i,d,m)},u.expose=function(d){Au(i,d)}}function _(i,u,d,m){i.addEventListener(u,d,m),wn(()=>i.removeEventListener(u,d,m))}function tu(i,u){let d=U(()=>{let m=i();if(!m)return;let n=()=>{u(m.assignedElements())};return m.addEventListener("slotchange",n),n(),()=>m.removeEventListener("slotchange",n)});wn(d)}var Lu=Symbol("sui:transition");function _n(i){return i[Lu]}function hd(i){let u=i,d=u[Lu];if(!d)d={inOpts:{},outOpts:{}},u[Lu]=d;return d}function En(i,u,d={},m="both"){let n=hd(i);if(m==="in"||m==="both")n.inT=u,n.inOpts=d;if(m==="out"||m==="both")n.outT=u,n.outOpts=d;if(!i.hasAttribute("data-sui-transition"))i.setAttribute("data-sui-transition","")}function $d(){if(typeof window>"u"||typeof window.matchMedia!=="function")return!1;try{return window.matchMedia("(prefers-reduced-motion: reduce)").matches}catch{return!1}}function Iu(i){if(!$d())return i;return{...i,duration:0,delay:0}}function Ru(i){let u=_n(i);if(!u?.inT?.in)return;u.current?.cancel();let d=u.inT.in(i,Iu(u.inOpts));return u.current=d,d}function ku(i){let u=_n(i);if(!u?.outT?.out)return;u.current?.cancel();let d=u.outT.out(i,Iu(u.outOpts));return u.current=d,d}function lu(i){if(i.nodeType!==1)return;let u=i;if(u.hasAttribute("data-sui-transition")){let d=_n(u);if(d?.current?.cancel(),d)d.current=void 0}for(let d of u.querySelectorAll("[data-sui-transition]")){let m=_n(d);if(m?.current?.cancel(),m)m.current=void 0}}function Su(i,u){for(let d of i){if(d.nodeType!==1)continue;let m=d;if(m.hasAttribute("data-sui-transition"))u(m);for(let n of m.querySelectorAll("[data-sui-transition]"))u(n)}}function Vn(i){Su(i,(u)=>{Ru(u)})}function kn(i){let u=[];if(Su(i,(d)=>{let m=ku(d);if(m)u.push(m.finished.catch(()=>{}))}),u.length===0)return null;return Promise.all(u).then(()=>{return})}var od=200,sd="cubic-bezier(0.4, 0, 0.2, 1)",ed=0;function Tn(i){return{duration:typeof i.duration==="number"?i.duration:od,easing:typeof i.easing==="string"?i.easing:sd,delay:typeof i.delay==="number"?i.delay:ed,fill:"both"}}function Qn(i,u,d){let m=i.animate;if(typeof m!=="function")return;return m.call(i,u,d)}var Zn={in:(i,u)=>Qn(i,[{opacity:0},{opacity:1}],Tn(u)),out:(i,u)=>Qn(i,[{opacity:1},{opacity:0}],Tn(u))};function Ku(i,u){let d=i==="y"?"translateY(-100%)":"translateX(-100%)",m="translate(0, 0)";return u==="in"?[{opacity:0,transform:d},{opacity:1,transform:"translate(0, 0)"}]:[{opacity:1,transform:"translate(0, 0)"},{opacity:0,transform:d}]}var Eu={in:(i,u)=>{let d=u.axis==="x"?"x":"y";return Qn(i,Ku(d,"in"),Tn(u))},out:(i,u)=>{let d=u.axis==="x"?"x":"y";return Qn(i,Ku(d,"out"),Tn(u))}};function Bu(i,u){let d={opacity:0,transform:`scale(${i})`},m={opacity:1,transform:"scale(1)"};return u==="in"?[d,m]:[m,d]}var ru={in:(i,u)=>{let d=typeof u.from==="number"?u.from:0;return Qn(i,Bu(d,"in"),Tn(u))},out:(i,u)=>{let d=typeof u.from==="number"?u.from:0;return Qn(i,Bu(d,"out"),Tn(u))}};function pd(i,u,d,m,n){if(d.fill(0,0,u),u===0)return;let b=0;for(let r=0;r<u;r++){let M=i[2*r+1],y=0,l=b;while(y<l){let E=y+l>>>1;if(i[2*n[E]+1]<M)y=E+1;else l=E}if(m[r]=y>0?n[y-1]:-1,n[y]=r,y===b)b++}let w=n[b-1];while(w>=0)d[w]=1,w=m[w]}function o(i){let{anchor:u,items:d,key:m,render:n}=i,b=[],w=!0,r=new Set,M=new Map,y=new Map,l=new Map,E=new Set,t=[],f=[],L=new Int32Array(0),H=new Uint8Array(0),C=new Int32Array(0),z=new Int32Array(0),j=(Z)=>{for(let D of Z.nodes)try{D.parentNode?.removeChild(D)}catch{}Z.cleanup?.()},g=U(()=>{if(!w)return;let Z=d(),D=Z.length;if(L.length<D*2)L=new Int32Array(Math.max(L.length*2,D*2,16));if(H.length<D){let Q=Math.max(H.length*2,D,8);H=new Uint8Array(Q),C=new Int32Array(Q),z=new Int32Array(Q)}for(let Q=0;Q<D;Q++){let Y=Z[Q],O=m(Y,Q);if(M.has(O))console.warn("[sui] {#for}: duplicate key",O,"at index",Q,"(prior at",M.get(O),"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");M.set(O,Q)}for(let Q of b)y.set(Q.key,Q);for(let Q of b){if(M.has(Q.key))continue;let Y=kn(Q.nodes);if(Y===null){j(Q);continue}r.add(Q),Y.then(()=>{if(!r.delete(Q))return;j(Q)})}f.length=0,t.length=0;for(let Q=0;Q<D;Q++){let Y=Z[Q],O=m(Y,Q),K=y.get(O),B;if(K)K.item=Y,Gn(u,"render",()=>K.update?.(Y,Q)),B=K;else{let A=Gn(u,"render",()=>n(Y,Q));if(!A)continue;B={key:O,item:Y,nodes:A.nodes,...A.cleanup&&{cleanup:A.cleanup},...A.update&&{update:A.update}},t.push(...A.nodes)}f.push(B)}for(let Q=0;Q<b.length;Q++)l.set(b[Q].key,Q);let J=0;for(let Q=0;Q<f.length;Q++){let Y=l.get(f[Q].key);if(Y!==void 0)L[2*J]=Q,L[2*J+1]=Y,J++}pd(L,J,H,C,z);for(let Q=0;Q<J;Q++)if(H[Q])E.add(L[2*Q]);let X=u.parentNode;if(X&&b.length===0&&f.length>0){let Q=document.createDocumentFragment();for(let Y=0;Y<f.length;Y++){let O=f[Y].nodes;for(let K=0;K<O.length;K++)Q.appendChild(O[K])}X.insertBefore(Q,u.nextSibling)}else{let Q=u;for(let Y=0;Y<f.length;Y++){if(!X)break;let O=f[Y];if(E.has(Y))Q=O.nodes[O.nodes.length-1]??Q;else for(let K of O.nodes)X.insertBefore(K,Q.nextSibling),Q=K}}let W=b;if(b=f,f=W,t.length>0)Vn(t);M.clear(),y.clear(),l.clear(),E.clear(),t.length=0});return()=>{w=!1,g();for(let Z of b)j(Z);for(let Z of r)j(Z);r.clear(),b=[],f.length=0}}function I(i){let{anchor:u}=i,{test:d,branches:m}=cd(i)?{test:i.test,branches:i.branches}:_d(i),n=null,b=-2,w=null,r=(y)=>{for(let l of y)try{l.parentNode?.removeChild(l)}catch{}},M=U(()=>{let y=d();if(n&&y===b)return;if(w&&y===w.idx&&y>=0&&y<m.length){let t=w;w=null;for(let f of t.nodes)lu(f);n={nodes:t.nodes,...t.cleanup&&{cleanup:t.cleanup}},b=y,Vn(n.nodes);return}if(n){let t={idx:b,nodes:n.nodes,...n.cleanup&&{cleanup:n.cleanup}},f=kn(t.nodes);if(n=null,f===null)r(t.nodes),t.cleanup?.();else w=t,f.then(()=>{if(w!==t)return;r(t.nodes),t.cleanup?.(),w=null})}if(b=y,y<0||y>=m.length)return;let l=Gn(u,"render",()=>m[y]());if(!l)return;n=l;let E=u.parentNode;if(!E)return;for(let t of n.nodes)E.insertBefore(t,u);Vn(n.nodes)});return()=>{if(M(),n)r(n.nodes),n.cleanup?.(),n=null;if(w)r(w.nodes),w.cleanup?.(),w=null}}function cd(i){return typeof i.test==="function"}function _d(i){let{cond:u,then_:d,else_:m}=i;return{test:()=>u()?0:m?1:-1,branches:m?[d,m]:[d]}}var Mu=(i)=>i??"",yu=(i)=>{if(i===null||i==="")return NaN;return Number(i)},wu=(i)=>{if(i===null)return!1;if(i.toLowerCase()==="false")return!1;return!0};function ni(i){return(u)=>{if(u===null||u==="")return i;try{return JSON.parse(u)}catch{return i}}}function Hu(i,u){let d=new Set(i);return(m)=>m!==null&&d.has(m)?m:u}var ii=(i)=>{switch(i.type){case"string":return Mu;case"bool":return wu;case"number":return yu;case"union":{let u=i.values??[],d=i.default??u[0]??"";return Hu(u,d)}case"object":return ni(i.default);case"array":return ni(i.default??[])}};function vu(i,u,d,m){if(!u)return;let n={},b,w=d.formValue;if(w!==void 0){let{sig:M,propDef:y}=au(d,m,w,"formValue"),l=y.default,E=ii(y);b=U(()=>{let t=M();u.setFormValue(nm(t))}),n.reset=()=>{M.set(l)},n.restore=(t)=>{M.set(E(t))}}let r=d.formDisabled;if(r!==void 0){let{sig:M,propDef:y}=au(d,m,r,"formDisabled");if(y.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${r}" must be a bool-typed prop`);n.disabled=(l)=>{M.set(!!l)}}if(i.__shipSetFormHooks=n,b)i.__shipSetFormValueCleanup=b;return b}function xu(i){let u=i;delete u.__shipSetFormHooks,delete u.__shipSetFormValueCleanup}function nm(i){if(i===null||i===void 0)return"";if(typeof i==="boolean")return i?"on":"";return String(i)}function au(i,u,d,m){let n=i.props[d];if(!n)throw Error(`[sui-runtime] ${m}: "${d}" — no matching prop on this component`);let b=u[qn(d)];if(!b)throw Error(`[sui-runtime] ${m}: "${d}" — prop signal missing`);return{sig:b,propDef:n}}var P={string(i={}){return{type:"string",...i}},bool(i={}){return{type:"bool",...i}},number(i={}){return{type:"number",...i}},union(i,u={}){return{type:"union",values:i,...u}},object(i={}){return{type:"object",...i}},array(i={}){return{type:"array",...i}}},im=(i,u)=>{if(u===void 0||u===null)return null;switch(i.type){case"string":return u===""?null:String(u);case"number":return Number.isNaN(u)?null:String(u);case"bool":return u?"":null;case"union":return String(u);case"object":case"array":try{return JSON.stringify(u)}catch{return null}}},um=(i,u)=>{let d=`ship-set-style-${i}`;if(document.getElementById(d))return;let m=document.createElement("style");m.id=d,m.textContent=`[data-component="${i}"] { ${u} }`,document.head.appendChild(m)},dm=(i,u,d,m)=>{let n=i.getAttribute(u),b=n!==null?m(n):d.default;return G(b)},mm=(i)=>{let u=[],d={};for(let[b,w]of Object.entries(i.props??{}))u.push(b),d[b]={camel:qn(b),decode:ii(w)};let m=i.formAssociated===!0||i.formValue!==void 0||i.formDisabled!==void 0,n={observedAttrs:u,props:d};if(m)n.formAssociated=!0;return n};function V(i,u){let d=mm(u),m=Object.entries(u.props??{});Cu(i,d,(n,b,w)=>{let r=null;if(u.shadow){if(r=n.shadowRoot??n.attachShadow({mode:"open"}),u.styles){let y=document.createElement("style");y.textContent=u.styles,r.appendChild(y)}}else if(u.styles)um(i,u.styles);let M=[];for(let[y,l]of m){let E=qn(y),t=d.props[y].decode,f=dm(n,y,l,t);if(b[E]=f,l.reflect){let L=!0,H=U(()=>{let C=f();if(L){L=!1;return}let z=im(l,C);if(z===null)n.removeAttribute(y);else n.setAttribute(y,z)});M.push(H)}}if(u.formValue!==void 0||u.formDisabled!==void 0){let y=vu(n,w,u,b);if(typeof y==="function")M.push(y)}if(u.setup){let y={props:b,host:n,cleanups:M};if(w)y.internals=w;u.setup(y)}return()=>{xu(n);for(let y of M.splice(0))try{y()}catch(l){console.error(`[ship-set:${i}] cleanup threw:`,l)}}})}var hu=new Set;function Cu(i,u,d){if(customElements.get(i)){if(!hu.has(i))hu.add(i),console.warn(`[sui-runtime] customElements.define("${i}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(u.formAssociated){class n extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return u.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=i,cn(this),this.scope=In();try{this.cleanup=Bn(this.scope,()=>mu(this,()=>d(this,this.propSignals,this.internals)))}catch(b){let w=Pn(this.parentNode);if(w)w.catch(b,"mount");else console.error(`[ship-set:${i}] setup threw:`,b)}}disconnectedCallback(){try{this.cleanup?.()}catch(b){console.error(`[ship-set:${i}] cleanup threw:`,b)}this.cleanup=void 0;try{this.scope?.dispose()}catch(b){console.error(`[ship-set:${i}] scope dispose threw:`,b)}this.scope=void 0}attributeChangedCallback(b,w,r){let M=u.props[b];if(!M)return;let y=this.propSignals[M.camel];if(y)y.set(M.decode(r))}formResetCallback(){let b=this.__shipSetFormHooks;try{b?.reset?.()}catch(w){console.error(`[ship-set:${i}] formResetCallback threw:`,w)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(b,w){let r=this.__shipSetFormHooks;try{r?.restore?.(b)}catch(M){console.error(`[ship-set:${i}] formStateRestoreCallback threw:`,M)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:b,mode:w}}))}formDisabledCallback(b){let w=this.__shipSetFormHooks;try{w?.disabled?.(b)}catch(r){console.error(`[ship-set:${i}] formDisabledCallback threw:`,r)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:b}}))}}customElements.define(i,n);return}class m extends HTMLElement{static get observedAttributes(){return u.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=i,cn(this),this.scope=In();try{this.cleanup=Bn(this.scope,()=>mu(this,()=>d(this,this.propSignals)))}catch(n){let b=Pn(this.parentNode);if(b)b.catch(n,"mount");else console.error(`[ship-set:${i}] setup threw:`,n)}}disconnectedCallback(){try{this.cleanup?.()}catch(n){console.error(`[ship-set:${i}] cleanup threw:`,n)}this.cleanup=void 0;try{this.scope?.dispose()}catch(n){console.error(`[ship-set:${i}] scope dispose threw:`,n)}this.scope=void 0}attributeChangedCallback(n,b,w){let r=u.props[n];if(!r)return;let M=this.propSignals[r.camel];if(M)M.set(r.decode(w))}}customElements.define(i,m)}function qn(i){return i.replace(/-([a-z])/g,(u,d)=>d.toUpperCase())}function zu(i,u,d){let m=tm(i.state,{ship:u.ship,...d}),n={};for(let[t,f]of Object.entries(m))n[t]=G(f);let b={ship:u.ship,route:u.route??{},navigate:u.navigate??((t)=>{}),...d,...n},w=Lm(n,b),r={...b},M={};for(let[t,f]of Object.entries(i.derived??{}))M[t]=h(()=>f(r));Object.assign(r,M);let y={};for(let[t,f]of Object.entries(i.on??{}))y[t]=(...L)=>f(w,...L);let l={},E=typeof i.use==="function"?i.use(r):i.use??{};for(let[t,f]of Object.entries(E))l[t]=zu(f,u,r);return{...n,...M,...y,...l}}function tm(i,u){if(i===void 0)return{};if(typeof i==="function")return i(u);return i}function Lm(i,u){return new Proxy({},{get(d,m){if(m in i)return i[m]();return u[m]},set(d,m,n){if(m in i)return i[m].set(n),!0;return console.warn(`[ship-set] hook handler tried to write s.${m} = … but ${m} is not a hook state key. The assignment was ignored.`),!0},has(d,m){return m in i||m in u},ownKeys(){return Object.keys(i)},getOwnPropertyDescriptor(d,m){if(m in i)return{configurable:!0,enumerable:!0,writable:!0,value:i[m]()};return}})}var $u="__shipSetLifecycle";function ou(i,u){let d=lm(i,u),m={ship:u.ship,route:u.route??{},navigate:u.navigate??((t)=>{}),...u.scope??{}},n=d.scope?d.scope(m,u.host):{};Object.assign(m,n);let b=Em(m,u.stateKeys,n),w=rm(b,m),r={...m},M=Mm(d,r);Object.assign(r,M);let y=wm(d,r,u);for(let[t,f]of Object.entries(y))r[t]=f;let l=ym(d,w),E={...n,...M,...l,...y};if(Object.keys(n).length>0&&u.host){let t=fu(n);if(Object.keys(t).length>0){let f=u.host,L=f.tagName.toLowerCase();bu(f,L,t)}}if(d.mount||d.unmount){let t={state:w};if(d.mount)t.mount=d.mount;if(d.unmount)t.unmount=d.unmount;Object.defineProperty(E,$u,{value:t,enumerable:!0,configurable:!0,writable:!1})}return E}function lm(i,u){if(typeof i==="function")return i({ship:u.ship});return i}function Em(i,u,d){let m={},n=u!==void 0,b=d?Object.keys(d):[];if(n||b.length>0){if(u)for(let w of u){let r=i[w];if(ju(r))m[w]=r}for(let w of b){if(w in m)continue;let r=i[w];if(ju(r))m[w]=r}return m}for(let[w,r]of Object.entries(i))if(ju(r))m[w]=r;return m}function ju(i){return typeof i==="function"&&"set"in i&&typeof i.set==="function"}function rm(i,u){return new Proxy({},{get(d,m){if(m in i)return i[m]();return u[m]},set(d,m,n){if(m in i){if(typeof n==="function")i[m].set(()=>n);else i[m].set(n);return!0}return console.warn(`[ship-set] handler tried to write s.${m} = … but ${m} is not declared as <state name="${m}"> in the template. The assignment was ignored.`),!0},has(d,m){return m in i||m in u},ownKeys(){return Object.keys(i)},getOwnPropertyDescriptor(d,m){if(m in i)return{configurable:!0,enumerable:!0,writable:!0,value:i[m]()};return}})}function Mm(i,u){let d=i.derived??{},m={};for(let[n,b]of Object.entries(d))m[n]=h(()=>b(u));return m}function ym(i,u){let d=i.on??{},m={};for(let[n,b]of Object.entries(d))m[n]=(...w)=>b(u,...w);return m}function wm(i,u,d){if(i.use===void 0)return{};let m=typeof i.use==="function"?i.use(u):i.use,n={};for(let[b,w]of Object.entries(m))n[b]=zu(w,d,u);return n}function N(i){return{setup:(u)=>ou(i,u),__def:i}}if(typeof customElements<"u"){for(let i of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(i))customElements.define(i,class extends HTMLElement{})}var jm=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,Pm=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function su(i){if(typeof CSSStyleSheet>"u")return null;try{let u=new CSSStyleSheet;return u.replaceSync(i),u}catch{return null}}var eu=su(jm),pu=su(Pm);function q(i){if(typeof ShadowRoot>"u")return;if(!(i instanceof ShadowRoot))return;let u=[pu,eu].filter((d)=>d!=null);if(u.length===0)return;try{i.adoptedStyleSheets=[...u,...i.adoptedStyleSheets??[]]}catch{}}var _1=V("sui-status-dot",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"success",reflect:!0}),pulse:P.bool({default:!1,reflect:!0}),size:P.union(["sm","md"],{default:"md",reflect:!0}),label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.pulse=u.pulse,n.size=u.size,n.label=u.label;let b=d.shadowRoot??d;q(b);let w=document.createElement("span");return w.setAttribute("class","dot"),w.setAttribute("part","dot"),m.push(T(w,"role",()=>n.label()?"img":null)),m.push(T(w,"aria-label",()=>n.label()||null)),m.push(T(w,"aria-hidden",()=>n.label()?null:"true")),b.appendChild(w),n}});var ut=V("sui-spinner",{props:{size:P.union(["xs","sm","md","lg"],{default:"md",reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),label:P.string({default:"Loading"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.size=u.size,n.tone=u.tone,n.label=u.label;let b=d.shadowRoot??d;q(b);let w=document.createElement("span");return w.setAttribute("class","ring"),w.setAttribute("part","ring"),m.push(T(w,"role",()=>n.label()?"status":null)),m.push(T(w,"aria-label",()=>n.label()||null)),m.push(T(w,"aria-hidden",()=>n.label()?null:"true")),b.appendChild(w),n}});var bt=V("sui-badge",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["solid","soft","outline"],{default:"soft",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),pill:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.variant=u.variant,n.size=u.size,n.pill=u.pill;let b=d.shadowRoot??d;q(b);let w=document.createElement("span");w.setAttribute("class","badge"),w.setAttribute("part","badge");let r=document.createElement("span");r.setAttribute("class","leading"),r.setAttribute("part","leading");let M=document.createElement("slot");M.setAttribute("name","leading"),r.appendChild(M),w.appendChild(r);let y=document.createElement("slot");return w.appendChild(y),b.appendChild(w),n}});var ui=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","button");if(!u.hasAttribute("tabindex")&&!d.disabled())u.setAttribute("tabindex","0");let m=U(()=>{let w=d.selected(),r=d.disabled();if(u.setAttribute("aria-pressed",w?"true":"false"),r){if(u.setAttribute("aria-disabled","true"),u.getAttribute("tabindex")!=="-1")u.setAttribute("tabindex","-1")}else if(u.removeAttribute("aria-disabled"),u.getAttribute("tabindex")==="-1")u.setAttribute("tabindex","0")}),n=(w)=>{if(d.disabled())return;if(w.key==="Enter"||w.key===" ")w.preventDefault(),u.click();else if(d.removable()&&(w.key==="Backspace"||w.key==="Delete"))w.preventDefault(),u.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};u.addEventListener("keydown",n);let b=U(()=>{let w=d.removeBtn?.();if(!w)return;let r=(M)=>{if(M.stopPropagation(),d.disabled())return;u.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};return w.addEventListener("click",r),()=>w.removeEventListener("click",r)});return()=>{m(),u.removeEventListener("keydown",n),b()}}});var rt=V("sui-chip",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0}),variant:P.union(["solid","soft","outline","ghost"],{default:"soft",reflect:!0}),size:P.union(["sm","md","lg"],{default:"sm",reflect:!0}),selected:P.bool({default:!1,reflect:!0}),removable:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.variant=u.variant,n.size=u.size,n.selected=u.selected,n.removable=u.removable,n.disabled=u.disabled,n.removeBtn=G(null);let b=null;if(ui&&typeof ui.setup==="function"){if(i.scope=n,b=ui.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(f)=>I({anchor:f,test:()=>{if(n.removable())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("button");C.setAttribute("class","remove"),C.setAttribute("part","remove"),C.setAttribute("type","button"),C.setAttribute("aria-label","Remove"),C.setAttribute("tabindex","-1"),F(n,"removeBtn",C);let z=document.createElement("svg");z.setAttribute("viewbox","0 0 10 10"),z.setAttribute("aria-hidden","true");let j=document.createElement("path");return j.setAttribute("d","M2 2 L8 8 M8 2 L2 8"),z.appendChild(j),C.appendChild(z),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let g of H)try{g()}catch(Z){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createElement("span");M.setAttribute("class","chip"),M.setAttribute("part","chip");let y=document.createElement("span");y.setAttribute("class","leading"),y.setAttribute("part","leading");let l=document.createElement("slot");l.setAttribute("name","leading"),y.appendChild(l),M.appendChild(y);let E=document.createElement("slot");M.appendChild(E);let t=document.createComment("if");if(M.appendChild(t),m.push(w(t)),r.appendChild(M),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let L=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let j=H[C];if(j!=null&&typeof j.set==="function"){if(typeof z==="function")j.set(()=>z);else j.set(z);return!0}return!1}});if(f.mount){let H=f.mount(L,d);if(typeof H==="function")m.push(H)}if(f.unmount)m.push(()=>f.unmount(L))}}return n}});var di=N({mount(i,u){let d=i,m=(b)=>{if(b.key==="Enter"||b.key===" ")b.preventDefault(),u.click()},n=U(()=>{if(d.interactive()){if(!u.hasAttribute("role"))u.setAttribute("role","button");if(!u.hasAttribute("tabindex"))u.setAttribute("tabindex","0");return u.addEventListener("keydown",m),()=>u.removeEventListener("keydown",m)}else{if(u.getAttribute("role")==="button")u.removeAttribute("role");if(u.getAttribute("tabindex")==="0")u.removeAttribute("tabindex")}});return()=>{n()}}});var zt=V("sui-card",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["default","outline"],{default:"default",reflect:!0}),padding:P.union(["none","sm","md","lg"],{default:"md",reflect:!0}),interactive:P.bool({default:!1,reflect:!0}),"accent-bar":P.bool({default:!1,reflect:!0}),"status-rail":P.bool({default:!1,reflect:!0}),fill:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.variant=u.variant,n.padding=u.padding,n.interactive=u.interactive,n.accentBar=u.accentBar,n.statusRail=u.statusRail,n.fill=u.fill;let b=null;if(di&&typeof di.setup==="function"){if(i.scope=n,b=di.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","header"),r.setAttribute("part","header");let M=document.createElement("slot");M.setAttribute("name","header"),m.push(bn(M,d,"data-no-header")),r.appendChild(M),w.appendChild(r);let y=document.createElement("div");y.setAttribute("class","content"),y.setAttribute("part","content");let l=document.createElement("slot");y.appendChild(l),w.appendChild(y);let E=document.createElement("div");E.setAttribute("class","footer"),E.setAttribute("part","footer");let t=document.createElement("slot");if(t.setAttribute("name","footer"),m.push(bn(t,d,"data-no-footer")),E.appendChild(t),w.appendChild(E),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let L=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let j=H[C];if(j!=null&&typeof j.set==="function"){if(typeof z==="function")j.set(()=>z);else j.set(z);return!0}return!1}});if(f.mount){let H=f.mount(L,d);if(typeof H==="function")m.push(H)}if(f.unmount)m.push(()=>f.unmount(L))}}return n}});var gt=V("sui-link",{props:{href:P.string({default:""}),external:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["default","underline","subtle"],{default:"default",reflect:!0}),target:P.string({default:""}),rel:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.href=u.href,n.external=u.external,n.tone=u.tone,n.variant=u.variant,n.target=u.target,n.rel=u.rel;let b=(l)=>I({anchor:l,test:()=>{if(n.external())return 0;return-1},branches:[()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("span");f.setAttribute("class","sr-only");let L=document.createTextNode("(opens in new tab)");f.appendChild(L),E.appendChild(f);let H=document.createElement("span");H.setAttribute("class","ext"),H.setAttribute("aria-hidden","true");let C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round");let z=document.createElement("path");return z.setAttribute("d","M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3"),C.appendChild(z),H.appendChild(C),E.appendChild(H),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let j of t)try{j()}catch(g){}}}}]}),w=d.shadowRoot??d;q(w);let r=document.createElement("a");r.setAttribute("part","link"),m.push(T(r,"href",()=>n.href())),m.push(T(r,"target",()=>n.external()?"_blank":n.target())),m.push(T(r,"rel",()=>n.external()?"noopener noreferrer":n.rel()));let M=document.createElement("slot");r.appendChild(M);let y=document.createComment("if");return r.appendChild(y),m.push(b(y)),w.appendChild(r),n}});var Jt=V("sui-divider",{props:{orientation:P.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:P.union(["solid","dashed"],{default:"solid",reflect:!0}),tone:P.union(["default","muted","primary","destructive"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.orientation=u.orientation,n.variant=u.variant,n.tone=u.tone;let b=d.shadowRoot??d;q(b);let w=document.createElement("hr");return w.setAttribute("class","line"),w.setAttribute("part","line"),w.setAttribute("role","separator"),m.push(T(w,"aria-orientation",()=>n.orientation())),b.appendChild(w),n}});var Wt=V("sui-skeleton",{props:{shape:P.union(["text","line","block","circle"],{default:"text",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.shape=u.shape,n.size=u.size;let b=d.shadowRoot??d;q(b);let w=document.createElement("span");return w.setAttribute("class","sk"),w.setAttribute("part","sk"),w.setAttribute("role","status"),w.setAttribute("aria-label","Loading"),w.setAttribute("aria-live","polite"),w.setAttribute("aria-busy","true"),b.appendChild(w),n}});var mi=N({mount(i,u){let d=i,m=(b)=>{if(b.key==="Enter"||b.key===" ")b.preventDefault(),u.click()},n=U(()=>{if(d.interactive()){if(!u.hasAttribute("role"))u.setAttribute("role","button");if(!u.hasAttribute("tabindex"))u.setAttribute("tabindex","0");return u.addEventListener("keydown",m),()=>u.removeEventListener("keydown",m)}else{if(u.getAttribute("role")==="button")u.removeAttribute("role");if(u.getAttribute("tabindex")==="0")u.removeAttribute("tabindex")}});return()=>{n(),u.removeEventListener("keydown",m)}}});var Ut=V("sui-stat",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent"],{default:"default",reflect:!0}),interactive:P.bool({default:!1,reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.interactive=u.interactive,n.size=u.size;let b=null;if(mi&&typeof mi.setup==="function"){if(i.scope=n,b=mi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("span");r.setAttribute("class","label"),r.setAttribute("part","label");let M=document.createElement("slot");M.setAttribute("name","label"),r.appendChild(M),w.appendChild(r);let y=document.createElement("span");y.setAttribute("class","value-row"),y.setAttribute("part","value-row");let l=document.createElement("span");l.setAttribute("class","value"),l.setAttribute("part","value");let E=document.createElement("slot");l.appendChild(E),y.appendChild(l);let t=document.createElement("span");t.setAttribute("class","trend"),t.setAttribute("part","trend");let f=document.createElement("slot");f.setAttribute("name","trend"),t.appendChild(f),y.appendChild(t),w.appendChild(y);let L=document.createElement("span");L.setAttribute("class","hint"),L.setAttribute("part","hint");let H=document.createElement("slot");if(H.setAttribute("name","hint"),L.appendChild(H),w.appendChild(L),b!=null){let C=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(n,{get(j,g){return j[g]},set(j,g,Z){let D=j[g];if(D!=null&&typeof D.set==="function"){if(typeof Z==="function")D.set(()=>Z);else D.set(Z);return!0}return!1}});if(C.mount){let j=C.mount(z,d);if(typeof j==="function")m.push(j)}if(C.unmount)m.push(()=>C.unmount(z))}}return n}});var Ot=V("sui-mark",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"warning",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone;let b=d.shadowRoot??d;q(b);let w=document.createElement("mark");w.setAttribute("part","mark");let r=document.createElement("slot");return w.appendChild(r),b.appendChild(w),n}});var bi=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","group");let m=U(()=>{let n=d.keys();if(n)u.setAttribute("aria-label",`Keyboard shortcut: ${n}`);else u.removeAttribute("aria-label")});return()=>m()}});var kt=V("sui-kbd",{props:{keys:P.string({default:""}),size:P.union(["sm","md"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.keys=u.keys,n.size=u.size;let b=null;if(bi&&typeof bi.setup==="function"){if(i.scope=n,b=bi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("kbd");r.setAttribute("part","kbd");let M=document.createElement("slot");if(r.appendChild(M),w.appendChild(r),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let l=new Proxy(n,{get(E,t){return E[t]},set(E,t,f){let L=E[t];if(L!=null&&typeof L.set==="function"){if(typeof f==="function")L.set(()=>f);else L.set(f);return!0}return!1}});if(y.mount){let E=y.mount(l,d);if(typeof E==="function")m.push(E)}if(y.unmount)m.push(()=>y.unmount(l))}}return n}});var vt=V("sui-avatar",{props:{src:P.string({default:""}),name:P.string({default:""}),initials:P.string({default:""}),"icon-slug":P.string({default:""}),size:P.union(["xs","sm","md","lg","xl"],{default:"md",reflect:!0}),shape:P.union(["round","square"],{default:"round",reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.src=u.src,n.name=u.name,n.initials=u.initials,n.iconSlug=u.iconSlug,n.size=u.size,n.shape=u.shape,n.tone=u.tone;let b=(y)=>I({anchor:y,test:()=>{if(n.iconSlug()||n.src())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),E=[],t=document.createElement("img");return t.setAttribute("part","img"),E.push(T(t,"src",()=>n.iconSlug()?"https://cdn.simpleicons.org/"+n.iconSlug():n.src())),E.push(T(t,"alt",()=>n.name())),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let f of E)try{f()}catch(L){}}}},()=>{let l=document.createDocumentFragment(),E=[],t=document.createElement("span");t.setAttribute("part","initials");let f=document.createTextNode("");return t.appendChild(f),E.push(k(f,()=>n.initials()?n.initials().slice(0,2).toUpperCase():n.name().trim().slice(0,1).toUpperCase()||"?")),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let L of E)try{L()}catch(H){}}}}]}),w=d.shadowRoot??d;q(w);let r=document.createElement("span");r.setAttribute("class","root"),r.setAttribute("part","root"),r.setAttribute("role","img"),m.push(T(r,"aria-label",()=>n.name()||n.initials()||"avatar"));let M=document.createComment("if");return r.appendChild(M),m.push(b(M)),w.appendChild(r),n}});var $t=V("sui-progress",{props:{value:P.number({default:0}),max:P.number({default:100}),tone:P.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),indeterminate:P.bool({default:!1,reflect:!0}),label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.max=u.max,n.tone=u.tone,n.size=u.size,n.indeterminate=u.indeterminate,n.label=u.label;let b=d.shadowRoot??d;q(b);let w=document.createElement("div");w.setAttribute("class","track"),w.setAttribute("part","track"),w.setAttribute("role","progressbar"),m.push(T(w,"aria-valuenow",()=>n.indeterminate()?null:n.value())),w.setAttribute("aria-valuemin","0"),m.push(T(w,"aria-valuemax",()=>n.max())),m.push(T(w,"aria-busy",()=>n.indeterminate()?"true":"false")),m.push(T(w,"aria-label",()=>n.label()));let r=document.createElement("div");return r.setAttribute("class","fill"),r.setAttribute("part","fill"),m.push(T(r,"style",()=>n.indeterminate()?"":"width: clamp(0%, "+n.value()/(n.max()||1)*100+"%, 100%)")),w.appendChild(r),b.appendChild(w),n}});var et=V("sui-progress-ring",{props:{value:P.number({default:0}),max:P.number({default:100}),size:P.number({default:48}),"track-width":P.number({default:4}),tone:P.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),indeterminate:P.bool({default:!1,reflect:!0}),label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.max=u.max,n.size=u.size,n.trackWidth=u.trackWidth,n.tone=u.tone,n.indeterminate=u.indeterminate,n.label=u.label;let b=d.shadowRoot??d;q(b);let w=document.createElement("div");w.setAttribute("class","ring"),m.push(T(w,"style",()=>"width:"+n.size()+"px;height:"+n.size()+"px"));let r=document.createElement("svg");r.setAttribute("class","ring-svg"),m.push(T(r,"viewbox",()=>"0 0 "+n.size()+" "+n.size())),r.setAttribute("role","progressbar"),r.setAttribute("aria-valuemin","0"),m.push(T(r,"aria-valuemax",()=>n.max())),m.push(T(r,"aria-valuenow",()=>n.indeterminate()?null:n.value()<0?0:n.value()>n.max()?n.max():n.value())),m.push(T(r,"aria-label",()=>n.label()||null)),r.setAttribute("xmlns","http://www.w3.org/2000/svg");let M=document.createElement("circle");M.setAttribute("class","track"),M.setAttribute("part","track"),m.push(T(M,"cx",()=>n.size()/2)),m.push(T(M,"cy",()=>n.size()/2)),m.push(T(M,"r",()=>(n.size()-n.trackWidth())/2)),M.setAttribute("fill","none"),m.push(T(M,"stroke-width",()=>n.trackWidth())),r.appendChild(M);let y=document.createElement("circle");y.setAttribute("class","bar"),y.setAttribute("part","ring"),m.push(T(y,"cx",()=>n.size()/2)),m.push(T(y,"cy",()=>n.size()/2)),m.push(T(y,"r",()=>(n.size()-n.trackWidth())/2)),y.setAttribute("fill","none"),m.push(T(y,"stroke-width",()=>n.trackWidth())),y.setAttribute("stroke-linecap","round"),m.push(T(y,"stroke-dasharray",()=>3.141592653589793*(n.size()-n.trackWidth()))),m.push(T(y,"stroke-dashoffset",()=>n.indeterminate()?3.141592653589793*(n.size()-n.trackWidth())*0.25:3.141592653589793*(n.size()-n.trackWidth())*(1-(n.value()<0?0:n.value()>(n.max()||1)?1:n.value()/(n.max()||1))))),r.appendChild(y),w.appendChild(r);let l=document.createElement("div");l.setAttribute("class","label"),l.setAttribute("aria-hidden","true");let E=document.createElement("slot");return l.appendChild(E),w.appendChild(l),b.appendChild(w),n}});var Pu={default:"var(--foreground)",primary:"var(--primary)",info:"var(--info)",success:"var(--success)",warning:"var(--warning)",destructive:"var(--destructive)",accent:"var(--accent)",muted:"var(--muted-foreground)"};function mn(i){if(!i)return Pu.default;return Pu[i]??Pu.default}function Yn(i){if(!i)return[];return i.split(",").map((u)=>Number.parseFloat(u.trim())).filter((u)=>Number.isFinite(u))}function Sn(i){if(!i)return[];return i.split(",").map((u)=>u.trim())}function rn(i,u){if(!i)return u;try{return JSON.parse(i)}catch{return u}}function Un(i,u,d,m=2){if(i.length===0)return{d:"",points:[]};let n=Math.min(...i),w=Math.max(...i)-n||1,r=d-m*2,M=i.length===1?0:u/(i.length-1),y=i.map((E,t)=>{let f=i.length===1?u/2:t*M,L=m+r-(E-n)/w*r;return[f,L]});return{d:y.map(([E,t],f)=>`${f===0?"M":"L"}${E.toFixed(2)},${t.toFixed(2)}`).join(" "),points:y}}function cu(i,u,d,m){return[i+d*Math.cos(m),u+d*Math.sin(m)]}function an(i,u,d,m,n){let[b,w]=cu(i,u,d,m),[r,M]=cu(i,u,d,n),y=n-m>Math.PI?1:0,l=n>m?1:0;return`M${b.toFixed(2)},${w.toFixed(2)} A${d},${d} 0 ${y} ${l} ${r.toFixed(2)},${M.toFixed(2)}`}var fi=N({scope:()=>{let i=G([]),u=G([]);return{seriesPaths:i,gridLines:u}},mount(i,u){let d=U(()=>{let m=rn(i.series(),[]),n=i.width(),b=i.height(),w=m.flatMap((t)=>t.data),r=w.length?Math.min(...w):0,y=(w.length?Math.max(...w):1)-r||1,l=b-4;i.gridLines.set(i.grid()?[0.25,0.5,0.75].map((t,f)=>({id:`g${f}`,y:2+l*t})):[]);let E=[];for(let t of m){if(t.data.length<2)continue;let f=t.data.map((Z)=>(Z-r)/y*100),{d:L,points:H}=Un(f,n,b,2);if(!L)continue;let C=mn(t.tone),z=H[0],j=H[H.length-1],g=`${L} L${j[0].toFixed(2)},${b} L${z[0].toFixed(2)},${b} Z`;E.push({name:t.name,areaD:g,strokeD:L,color:C,pts:H.map(([Z,D],J)=>({id:`${t.name}-${J}`,cx:Z,cy:D}))})}i.seriesPaths.set(E)});return()=>d()}});var mL=V("sui-area-chart",{props:{series:P.string({default:"",reflect:!0}),labels:P.string({default:""}),width:P.number({default:300}),height:P.number({default:200}),points:P.bool({default:!1,reflect:!0}),grid:P.bool({default:!1,reflect:!0}),label:P.string({default:"area chart"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.series=u.series,n.labels=u.labels,n.width=u.width,n.height=u.height,n.points=u.points,n.grid=u.grid,n.label=u.label;let b=null;if(fi&&typeof fi.setup==="function"){if(i.scope=n,b=fi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(H)=>o({anchor:H,items:()=>n.gridLines(),key:(C,z)=>{let j=Object.create(n);return j.g=C,j.__i=z,((g)=>C.id)(j)},render:(C,z)=>{let j=Object.create(n);return j.g=C,j.__i=z,j.__v=G(0),((g)=>{let Z=document.createDocumentFragment(),D=[],J=document.createElement("line");J.setAttribute("class","grid-line"),J.setAttribute("x1","0"),D.push(T(J,"x2",()=>(g.__v(),g.width())));{let X=g.g.y;if(X==null||X===!1||X==="")J.removeAttribute("y1");else J.setAttribute("y1",X===!0?"":String(X))}{let X=g.g.y;if(X==null||X===!1||X==="")J.removeAttribute("y2");else J.setAttribute("y2",X===!0?"":String(X))}return Z.appendChild(J),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let X of D)try{X()}catch(W){}},update:(X,W)=>{g.g=X,g.__i=W;{let Q=g.g.y;if(Q==null||Q===!1||Q==="")J.removeAttribute("y1");else J.setAttribute("y1",Q===!0?"":String(Q))}{let Q=g.g.y;if(Q==null||Q===!1||Q==="")J.removeAttribute("y2");else J.setAttribute("y2",Q===!0?"":String(Q))}g.__v.set((Q)=>Q+1)}}})(j)}}),r=(H)=>o({anchor:H,items:()=>n.seriesPaths(),key:(C,z)=>{let j=Object.create(n);return j.s=C,j.__i=z,((g)=>C.name)(j)},render:(C,z)=>{let j=Object.create(n);return j.s=C,j.__i=z,j.__v=G(0),((g)=>{let Z=(O)=>o({anchor:O,items:()=>g.s.pts,key:(K,B)=>{let A=Object.create(g);return A.p=K,A.__i=B,((a)=>K.id)(A)},render:(K,B)=>{let A=Object.create(g);return A.p=K,A.__i=B,A.__v=G(0),((a)=>{let S=document.createDocumentFragment(),v=[],R=document.createElement("circle");{let x=a.p.cx;if(x==null||x===!1||x==="")R.removeAttribute("cx");else R.setAttribute("cx",x===!0?"":String(x))}{let x=a.p.cy;if(x==null||x===!1||x==="")R.removeAttribute("cy");else R.setAttribute("cy",x===!0?"":String(x))}R.setAttribute("r","2.5");{let x=a.s.color;if(x==null||x===!1||x==="")R.removeAttribute("fill");else R.setAttribute("fill",x===!0?"":String(x))}return S.appendChild(R),{nodes:Array.from(S.childNodes),cleanup:()=>{for(let x of v)try{x()}catch(s){}},update:(x,s)=>{a.p=x,a.__i=s;{let $=a.p.cx;if($==null||$===!1||$==="")R.removeAttribute("cx");else R.setAttribute("cx",$===!0?"":String($))}{let $=a.p.cy;if($==null||$===!1||$==="")R.removeAttribute("cy");else R.setAttribute("cy",$===!0?"":String($))}{let $=a.s.color;if($==null||$===!1||$==="")R.removeAttribute("fill");else R.setAttribute("fill",$===!0?"":String($))}a.__v.set(($)=>$+1)}}})(A)}}),D=(O)=>I({anchor:O,test:()=>{if(g.points())return 0;return-1},branches:[()=>{let K=document.createDocumentFragment(),B=[],A=document.createComment("for");return K.appendChild(A),B.push(Z(A)),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let a of B)try{a()}catch(S){}}}}]}),J=document.createDocumentFragment(),X=[],W=document.createElement("path");{let O=g.s.areaD;if(O==null||O===!1||O==="")W.removeAttribute("d");else W.setAttribute("d",O===!0?"":String(O))}{let O=g.s.color;if(O==null||O===!1||O==="")W.removeAttribute("fill");else W.setAttribute("fill",O===!0?"":String(O))}W.setAttribute("fill-opacity","0.3"),W.setAttribute("stroke","none"),J.appendChild(W);let Q=document.createElement("path");{let O=g.s.strokeD;if(O==null||O===!1||O==="")Q.removeAttribute("d");else Q.setAttribute("d",O===!0?"":String(O))}Q.setAttribute("fill","none");{let O=g.s.color;if(O==null||O===!1||O==="")Q.removeAttribute("stroke");else Q.setAttribute("stroke",O===!0?"":String(O))}Q.setAttribute("stroke-width","1.75"),Q.setAttribute("stroke-linecap","round"),Q.setAttribute("stroke-linejoin","round"),J.appendChild(Q);let Y=document.createComment("if");return J.appendChild(Y),X.push(D(Y)),{nodes:Array.from(J.childNodes),cleanup:()=>{for(let O of X)try{O()}catch(K){}},update:(O,K)=>{g.s=O,g.__i=K;{let B=g.s.areaD;if(B==null||B===!1||B==="")W.removeAttribute("d");else W.setAttribute("d",B===!0?"":String(B))}{let B=g.s.color;if(B==null||B===!1||B==="")W.removeAttribute("fill");else W.setAttribute("fill",B===!0?"":String(B))}{let B=g.s.strokeD;if(B==null||B===!1||B==="")Q.removeAttribute("d");else Q.setAttribute("d",B===!0?"":String(B))}{let B=g.s.color;if(B==null||B===!1||B==="")Q.removeAttribute("stroke");else Q.setAttribute("stroke",B===!0?"":String(B))}g.__v.set((B)=>B+1)}}})(j)}}),M=d.shadowRoot??d;q(M);let y=document.createElement("div");y.setAttribute("class","wrap");let l=document.createElement("svg");m.push(T(l,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),l.setAttribute("preserveaspectratio","none"),l.setAttribute("role","img"),m.push(T(l,"aria-label",()=>n.label()));let E=document.createComment("for");l.appendChild(E),m.push(w(E));let t=document.createComment("for");l.appendChild(t),m.push(r(t)),y.appendChild(l);let f=document.createElement("div");f.setAttribute("class","legend");let L=document.createElement("slot");if(f.appendChild(L),y.appendChild(f),M.appendChild(y),b!=null){let H=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(H){let C=new Proxy(n,{get(z,j){return z[j]},set(z,j,g){let Z=z[j];if(Z!=null&&typeof Z.set==="function"){if(typeof g==="function")Z.set(()=>g);else Z.set(g);return!0}return!1}});if(H.mount){let z=H.mount(C,d);if(typeof z==="function")m.push(z)}if(H.unmount)m.push(()=>H.unmount(C))}}return n}});var ti=N({scope:()=>{let i=G([]),u=G(!0);return{bars:i,isEmpty:u}},mount(i,u){let d=U(()=>{let m=Yn(i.data());if(m.length===0){i.bars.set([]),i.isEmpty.set(!0);return}let n=Sn(i.labels()),b=i.width(),w=i.height(),r=i.max(),M=r>0?r:Math.max(...m,0),y=M<=0?1:M,l=mn(i.tone()),E=4,t=3,f=i.orientation()==="horizontal",L=m.length;i.bars.set(m.map((H,C)=>{let z=Math.max(0,H)/y,j,g,Z,D;if(f)D=(w-E*(L-1))/L,g=C*(D+E),Z=z*b,j=0;else Z=(b-E*(L-1))/L,j=C*(Z+E),D=z*w,g=w-D;return{id:`b${C}`,x:j,y:g,w:Math.max(0,Z),h:Math.max(0,D),rx:t,ry:t,color:l,title:`${n[C]??String(H)}: ${H}`}})),i.isEmpty.set(!1)});return()=>d()}});var rL=V("sui-bar-chart",{props:{data:P.string({default:"",reflect:!0}),labels:P.string({default:""}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),orientation:P.union(["vertical","horizontal"],{default:"vertical",reflect:!0}),width:P.number({default:300}),height:P.number({default:200}),max:P.number({default:0}),label:P.string({default:"bar chart"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.data=u.data,n.labels=u.labels,n.tone=u.tone,n.orientation=u.orientation,n.width=u.width,n.height=u.height,n.max=u.max,n.label=u.label;let b=null;if(ti&&typeof ti.setup==="function"){if(i.scope=n,b=ti.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>o({anchor:l,items:()=>n.bars(),key:(E,t)=>{let f=Object.create(n);return f.b=E,f.__i=t,((L)=>E.id)(f)},render:(E,t)=>{let f=Object.create(n);return f.b=E,f.__i=t,f.__v=G(0),((L)=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("rect");{let Z=L.b.x;if(Z==null||Z===!1||Z==="")z.removeAttribute("x");else z.setAttribute("x",Z===!0?"":String(Z))}{let Z=L.b.y;if(Z==null||Z===!1||Z==="")z.removeAttribute("y");else z.setAttribute("y",Z===!0?"":String(Z))}{let Z=L.b.w;if(Z==null||Z===!1||Z==="")z.removeAttribute("width");else z.setAttribute("width",Z===!0?"":String(Z))}{let Z=L.b.h;if(Z==null||Z===!1||Z==="")z.removeAttribute("height");else z.setAttribute("height",Z===!0?"":String(Z))}{let Z=L.b.rx;if(Z==null||Z===!1||Z==="")z.removeAttribute("rx");else z.setAttribute("rx",Z===!0?"":String(Z))}{let Z=L.b.ry;if(Z==null||Z===!1||Z==="")z.removeAttribute("ry");else z.setAttribute("ry",Z===!0?"":String(Z))}{let Z=L.b.color;if(Z==null||Z===!1||Z==="")z.removeAttribute("fill");else z.setAttribute("fill",Z===!0?"":String(Z))}let j=document.createElement("title"),g=document.createTextNode("");return j.appendChild(g),g.textContent=L.b.title,z.appendChild(j),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let Z of C)try{Z()}catch(D){}},update:(Z,D)=>{L.b=Z,L.__i=D;{let J=L.b.x;if(J==null||J===!1||J==="")z.removeAttribute("x");else z.setAttribute("x",J===!0?"":String(J))}{let J=L.b.y;if(J==null||J===!1||J==="")z.removeAttribute("y");else z.setAttribute("y",J===!0?"":String(J))}{let J=L.b.w;if(J==null||J===!1||J==="")z.removeAttribute("width");else z.setAttribute("width",J===!0?"":String(J))}{let J=L.b.h;if(J==null||J===!1||J==="")z.removeAttribute("height");else z.setAttribute("height",J===!0?"":String(J))}{let J=L.b.rx;if(J==null||J===!1||J==="")z.removeAttribute("rx");else z.setAttribute("rx",J===!0?"":String(J))}{let J=L.b.ry;if(J==null||J===!1||J==="")z.removeAttribute("ry");else z.setAttribute("ry",J===!0?"":String(J))}{let J=L.b.color;if(J==null||J===!1||J==="")z.removeAttribute("fill");else z.setAttribute("fill",J===!0?"":String(J))}g.textContent=L.b.title,L.__v.set((J)=>J+1)}}})(f)}}),r=(l)=>I({anchor:l,test:()=>{if(n.isEmpty())return 0;return 1},branches:[()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("svg");return t.push(T(f,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),f.setAttribute("aria-hidden","true"),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let L of t)try{L()}catch(H){}}}},()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("svg");t.push(T(f,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),f.setAttribute("preserveaspectratio","none"),f.setAttribute("role","img"),t.push(T(f,"aria-label",()=>n.label()));let L=document.createComment("for");return f.appendChild(L),t.push(w(L)),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let H of t)try{H()}catch(C){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createComment("if");if(M.appendChild(y),m.push(r(y)),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var Li=N({scope:()=>{let i=G([]),u=G([]);return{seriesPaths:i,gridLines:u}},mount(i,u){let d=U(()=>{let m=rn(i.series(),[]),n=i.width(),b=i.height(),w=m.flatMap((t)=>t.data),r=w.length?Math.min(...w):0,y=(w.length?Math.max(...w):1)-r||1,l=b-4;i.gridLines.set(i.grid()?[0.25,0.5,0.75].map((t,f)=>({id:`g${f}`,y:2+l*t})):[]);let E=[];for(let t of m){if(t.data.length<2)continue;let f=t.data.map((z)=>(z-r)/y*100),{d:L,points:H}=Un(f,n,b,2);if(!L)continue;let C=mn(t.tone);E.push({name:t.name,strokeD:L,color:C,pts:H.map(([z,j],g)=>({id:`${t.name}-${g}`,cx:z,cy:j}))})}i.seriesPaths.set(E)});return()=>d()}});var jL=V("sui-line-chart",{props:{series:P.string({default:"",reflect:!0}),labels:P.string({default:""}),width:P.number({default:300}),height:P.number({default:200}),points:P.bool({default:!1,reflect:!0}),grid:P.bool({default:!1,reflect:!0}),label:P.string({default:"line chart"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.series=u.series,n.labels=u.labels,n.width=u.width,n.height=u.height,n.points=u.points,n.grid=u.grid,n.label=u.label;let b=null;if(Li&&typeof Li.setup==="function"){if(i.scope=n,b=Li.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(H)=>o({anchor:H,items:()=>n.gridLines(),key:(C,z)=>{let j=Object.create(n);return j.g=C,j.__i=z,((g)=>C.id)(j)},render:(C,z)=>{let j=Object.create(n);return j.g=C,j.__i=z,j.__v=G(0),((g)=>{let Z=document.createDocumentFragment(),D=[],J=document.createElement("line");J.setAttribute("class","grid-line"),J.setAttribute("x1","0"),D.push(T(J,"x2",()=>(g.__v(),g.width())));{let X=g.g.y;if(X==null||X===!1||X==="")J.removeAttribute("y1");else J.setAttribute("y1",X===!0?"":String(X))}{let X=g.g.y;if(X==null||X===!1||X==="")J.removeAttribute("y2");else J.setAttribute("y2",X===!0?"":String(X))}return Z.appendChild(J),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let X of D)try{X()}catch(W){}},update:(X,W)=>{g.g=X,g.__i=W;{let Q=g.g.y;if(Q==null||Q===!1||Q==="")J.removeAttribute("y1");else J.setAttribute("y1",Q===!0?"":String(Q))}{let Q=g.g.y;if(Q==null||Q===!1||Q==="")J.removeAttribute("y2");else J.setAttribute("y2",Q===!0?"":String(Q))}g.__v.set((Q)=>Q+1)}}})(j)}}),r=(H)=>o({anchor:H,items:()=>n.seriesPaths(),key:(C,z)=>{let j=Object.create(n);return j.s=C,j.__i=z,((g)=>C.name)(j)},render:(C,z)=>{let j=Object.create(n);return j.s=C,j.__i=z,j.__v=G(0),((g)=>{let Z=(Y)=>o({anchor:Y,items:()=>g.s.pts,key:(O,K)=>{let B=Object.create(g);return B.p=O,B.__i=K,((A)=>O.id)(B)},render:(O,K)=>{let B=Object.create(g);return B.p=O,B.__i=K,B.__v=G(0),((A)=>{let a=document.createDocumentFragment(),S=[],v=document.createElement("circle");{let R=A.p.cx;if(R==null||R===!1||R==="")v.removeAttribute("cx");else v.setAttribute("cx",R===!0?"":String(R))}{let R=A.p.cy;if(R==null||R===!1||R==="")v.removeAttribute("cy");else v.setAttribute("cy",R===!0?"":String(R))}v.setAttribute("r","2.5");{let R=A.s.color;if(R==null||R===!1||R==="")v.removeAttribute("fill");else v.setAttribute("fill",R===!0?"":String(R))}return a.appendChild(v),{nodes:Array.from(a.childNodes),cleanup:()=>{for(let R of S)try{R()}catch(x){}},update:(R,x)=>{A.p=R,A.__i=x;{let s=A.p.cx;if(s==null||s===!1||s==="")v.removeAttribute("cx");else v.setAttribute("cx",s===!0?"":String(s))}{let s=A.p.cy;if(s==null||s===!1||s==="")v.removeAttribute("cy");else v.setAttribute("cy",s===!0?"":String(s))}{let s=A.s.color;if(s==null||s===!1||s==="")v.removeAttribute("fill");else v.setAttribute("fill",s===!0?"":String(s))}A.__v.set((s)=>s+1)}}})(B)}}),D=(Y)=>I({anchor:Y,test:()=>{if(g.points())return 0;return-1},branches:[()=>{let O=document.createDocumentFragment(),K=[],B=document.createComment("for");return O.appendChild(B),K.push(Z(B)),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let A of K)try{A()}catch(a){}}}}]}),J=document.createDocumentFragment(),X=[],W=document.createElement("path");{let Y=g.s.strokeD;if(Y==null||Y===!1||Y==="")W.removeAttribute("d");else W.setAttribute("d",Y===!0?"":String(Y))}W.setAttribute("fill","none");{let Y=g.s.color;if(Y==null||Y===!1||Y==="")W.removeAttribute("stroke");else W.setAttribute("stroke",Y===!0?"":String(Y))}W.setAttribute("stroke-width","1.75"),W.setAttribute("stroke-linecap","round"),W.setAttribute("stroke-linejoin","round"),J.appendChild(W);let Q=document.createComment("if");return J.appendChild(Q),X.push(D(Q)),{nodes:Array.from(J.childNodes),cleanup:()=>{for(let Y of X)try{Y()}catch(O){}},update:(Y,O)=>{g.s=Y,g.__i=O;{let K=g.s.strokeD;if(K==null||K===!1||K==="")W.removeAttribute("d");else W.setAttribute("d",K===!0?"":String(K))}{let K=g.s.color;if(K==null||K===!1||K==="")W.removeAttribute("stroke");else W.setAttribute("stroke",K===!0?"":String(K))}g.__v.set((K)=>K+1)}}})(j)}}),M=d.shadowRoot??d;q(M);let y=document.createElement("div");y.setAttribute("class","wrap");let l=document.createElement("svg");m.push(T(l,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),l.setAttribute("preserveaspectratio","none"),l.setAttribute("role","img"),m.push(T(l,"aria-label",()=>n.label()));let E=document.createComment("for");l.appendChild(E),m.push(w(E));let t=document.createComment("for");l.appendChild(t),m.push(r(t)),y.appendChild(l);let f=document.createElement("div");f.setAttribute("class","legend");let L=document.createElement("slot");if(f.appendChild(L),y.appendChild(f),M.appendChild(y),b!=null){let H=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(H){let C=new Proxy(n,{get(z,j){return z[j]},set(z,j,g){let Z=z[j];if(Z!=null&&typeof Z.set==="function"){if(typeof g==="function")Z.set(()=>g);else Z.set(g);return!0}return!1}});if(H.mount){let z=H.mount(C,d);if(typeof z==="function")m.push(z)}if(H.unmount)m.push(()=>H.unmount(C))}}return n}});var li=N({scope:()=>{let i=G(!1),u=G(""),d=G(""),m=G("var(--primary)");return{hasData:i,strokePath:u,areaPath:d,lineColor:m}},mount(i,u){let d=U(()=>{let m=Yn(i.data()),n=i.width(),b=i.height(),w=mn(i.tone());if(i.lineColor.set(w),m.length<2){i.hasData.set(!1),i.strokePath.set(""),i.areaPath.set("");return}let{d:r,points:M}=Un(m,n,b,2);if(i.strokePath.set(r),i.fill()){let y=M[0],l=M[M.length-1];i.areaPath.set(`${r} L${l[0].toFixed(2)},${b} L${y[0].toFixed(2)},${b} Z`)}else i.areaPath.set("");i.hasData.set(!0)});return()=>d()}});var GL=V("sui-sparkline",{props:{data:P.string({default:"",reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),width:P.number({default:80}),height:P.number({default:20}),fill:P.bool({default:!1,reflect:!0}),label:P.string({default:"sparkline"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.data=u.data,n.tone=u.tone,n.width=u.width,n.height=u.height,n.fill=u.fill,n.label=u.label;let b=null;if(li&&typeof li.setup==="function"){if(i.scope=n,b=li.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>I({anchor:l,test:()=>{if(n.fill())return 0;return-1},branches:[()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("path");return t.push(T(f,"d",()=>n.areaPath())),t.push(T(f,"fill",()=>n.lineColor())),f.setAttribute("fill-opacity","0.18"),f.setAttribute("stroke","none"),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let L of t)try{L()}catch(H){}}}}]}),r=(l)=>I({anchor:l,test:()=>{if(n.hasData())return 0;return 1},branches:[()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("svg");t.push(T(f,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),t.push(T(f,"width",()=>n.width())),t.push(T(f,"height",()=>n.height())),f.setAttribute("role","img"),t.push(T(f,"aria-label",()=>n.label()));let L=document.createComment("if");f.appendChild(L),t.push(w(L));let H=document.createElement("path");t.push(T(H,"d",()=>n.strokePath())),H.setAttribute("fill","none"),t.push(T(H,"stroke",()=>n.lineColor())),H.setAttribute("stroke-width","1.5"),H.setAttribute("stroke-linecap","round"),H.setAttribute("stroke-linejoin","round");let C=document.createTextNode(`
    `);return H.appendChild(C),f.appendChild(H),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let z of t)try{z()}catch(j){}}}},()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("svg");t.push(T(f,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),t.push(T(f,"width",()=>n.width())),t.push(T(f,"height",()=>n.height())),f.setAttribute("aria-hidden","true");let L=document.createTextNode(`
  `);return f.appendChild(L),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let H of t)try{H()}catch(C){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createComment("if");if(M.appendChild(y),m.push(r(y)),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var Ei=N({scope:()=>({segments:G([])}),mount(i,u){let d=U(()=>{let m=rn(i.data(),[]),n=m.reduce((t,f)=>t+Math.max(0,f.value),0);if(n<=0){i.segments.set([]);return}let b=i.size(),w=i.thickness(),r=(b-w)/2,M=b/2,y=b/2,l=2*Math.PI,E=-Math.PI/2;i.segments.set(m.filter((t)=>t.value>0).map((t,f)=>{let L=t.value/n*l,H=L>=l-0.001?l-0.001:L,C=an(M,y,r,E,E+H);return E+=L,{id:`s${f}`,d:C,color:mn(t.tone),title:`${t.label}: ${t.value}`}}))});return()=>d()}});var UL=V("sui-donut",{props:{data:P.string({default:"",reflect:!0}),size:P.number({default:120}),thickness:P.number({default:16}),label:P.string({default:"donut chart"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.data=u.data,n.size=u.size,n.thickness=u.thickness,n.label=u.label;let b=null;if(Ei&&typeof Ei.setup==="function"){if(i.scope=n,b=Ei.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>o({anchor:L,items:()=>n.segments(),key:(H,C)=>{let z=Object.create(n);return z.seg=H,z.__i=C,((j)=>H.id)(z)},render:(H,C)=>{let z=Object.create(n);return z.seg=H,z.__i=C,z.__v=G(0),((j)=>{let g=document.createDocumentFragment(),Z=[],D=document.createElement("path");D.setAttribute("part","segment");{let W=j.seg.d;if(W==null||W===!1||W==="")D.removeAttribute("d");else D.setAttribute("d",W===!0?"":String(W))}D.setAttribute("fill","none");{let W=j.seg.color;if(W==null||W===!1||W==="")D.removeAttribute("stroke");else D.setAttribute("stroke",W===!0?"":String(W))}Z.push(T(D,"stroke-width",()=>(j.__v(),j.thickness()))),D.setAttribute("stroke-linecap","butt");let J=document.createElement("title"),X=document.createTextNode("");return J.appendChild(X),X.textContent=j.seg.title,D.appendChild(J),g.appendChild(D),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let W of Z)try{W()}catch(Q){}},update:(W,Q)=>{j.seg=W,j.__i=Q;{let Y=j.seg.d;if(Y==null||Y===!1||Y==="")D.removeAttribute("d");else D.setAttribute("d",Y===!0?"":String(Y))}{let Y=j.seg.color;if(Y==null||Y===!1||Y==="")D.removeAttribute("stroke");else D.setAttribute("stroke",Y===!0?"":String(Y))}X.textContent=j.seg.title,j.__v.set((Y)=>Y+1)}}})(z)}}),r=d.shadowRoot??d;q(r);let M=document.createElement("svg");m.push(T(M,"viewbox",()=>"0 0 "+n.size()+" "+n.size())),M.setAttribute("role","img"),m.push(T(M,"aria-label",()=>n.label())),m.push(T(M,"style",()=>"width:"+n.size()+"px;height:"+n.size()+"px"));let y=document.createElement("circle");y.setAttribute("class","track"),y.setAttribute("part","track"),m.push(T(y,"cx",()=>n.size()/2)),m.push(T(y,"cy",()=>n.size()/2)),m.push(T(y,"r",()=>(n.size()-n.thickness())/2)),y.setAttribute("fill","none"),m.push(T(y,"stroke-width",()=>n.thickness()));let l=document.createTextNode(`
  `);y.appendChild(l),M.appendChild(y);let E=document.createComment("for");M.appendChild(E),m.push(w(E)),r.appendChild(M);let t=document.createElement("div");t.setAttribute("class","center");let f=document.createElement("slot");if(t.appendChild(f),r.appendChild(t),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let H=new Proxy(n,{get(C,z){return C[z]},set(C,z,j){let g=C[z];if(g!=null&&typeof g.set==="function"){if(typeof j==="function")g.set(()=>j);else g.set(j);return!0}return!1}});if(L.mount){let C=L.mount(H,d);if(typeof C==="function")m.push(C)}if(L.unmount)m.push(()=>L.unmount(H))}}return n}});var nd=Math.PI/180,ri=135*nd,_u=405*nd,Mi=N({scope:()=>({trackD:G(""),valueD:G(""),arcColor:G("var(--primary)")}),mount(i,u){let d=U(()=>{let m=i.size(),n=i.thickness(),b=(m-n)/2-2,w=m/2,r=m/2;i.trackD.set(an(w,r,b,ri,_u));let M=i.value(),y=i.min(),l=i.max(),E=Math.max(0,Math.min(1,(M-y)/(l-y||1))),t=ri+(_u-ri)*E;i.valueD.set(E>0?an(w,r,b,ri,t):"");let f=i.tone(),L=i.lowThreshold(),H=i.highThreshold(),C=f;if(!C)if(L>0&&M<L)C="destructive";else if(H>0&&M<H)C="warning";else if(L>0||H>0)C="success";else C="primary";i.arcColor.set(mn(C))});return()=>d()}});var IL=V("sui-gauge",{props:{value:P.number({default:0,reflect:!0}),min:P.number({default:0}),max:P.number({default:100}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"",reflect:!0}),"low-threshold":P.number({default:0}),"high-threshold":P.number({default:0}),size:P.number({default:140}),thickness:P.number({default:12}),label:P.string({default:"gauge"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.min=u.min,n.max=u.max,n.tone=u.tone,n.lowThreshold=u.lowThreshold,n.highThreshold=u.highThreshold,n.size=u.size,n.thickness=u.thickness,n.label=u.label;let b=null;if(Mi&&typeof Mi.setup==="function"){if(i.scope=n,b=Mi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>I({anchor:L,test:()=>{if(n.valueD())return 0;return-1},branches:[()=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("path");z.setAttribute("part","fill"),C.push(T(z,"d",()=>n.valueD())),z.setAttribute("fill","none"),C.push(T(z,"stroke",()=>n.arcColor())),C.push(T(z,"stroke-width",()=>n.thickness())),z.setAttribute("stroke-linecap","round");let j=document.createTextNode(`
    `);return z.appendChild(j),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let g of C)try{g()}catch(Z){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createElement("svg");m.push(T(M,"viewbox",()=>"0 0 "+n.size()+" "+n.size())),M.setAttribute("role","img"),m.push(T(M,"aria-label",()=>n.label())),m.push(T(M,"style",()=>"width:"+n.size()+"px;height:"+n.size()+"px"));let y=document.createElement("path");y.setAttribute("class","track"),y.setAttribute("part","track"),m.push(T(y,"d",()=>n.trackD())),y.setAttribute("fill","none"),m.push(T(y,"stroke-width",()=>n.thickness())),y.setAttribute("stroke-linecap","round");let l=document.createTextNode(`
  `);y.appendChild(l),M.appendChild(y);let E=document.createComment("if");M.appendChild(E),m.push(w(E)),r.appendChild(M);let t=document.createElement("div");t.setAttribute("class","gauge-label");let f=document.createElement("slot");if(t.appendChild(f),r.appendChild(t),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let H=new Proxy(n,{get(C,z){return C[z]},set(C,z,j){let g=C[z];if(g!=null&&typeof g.set==="function"){if(typeof j==="function")g.set(()=>j);else g.set(j);return!0}return!1}});if(L.mount){let C=L.mount(H,d);if(typeof C==="function")m.push(C)}if(L.unmount)m.push(()=>L.unmount(H))}}return n}});var yi=N({scope:()=>({bars:G([])}),mount(i,u){let d=U(()=>{let m=rn(i.segments(),[]),n=m.reduce((M,y)=>M+Math.max(0,y.count),0),b=i.total(),w=b>0?b:n,r=w<=0?1:w;i.bars.set(m.filter((M)=>M.count>0).map((M,y)=>{let l=Math.max(0,M.count)/r*100;return{id:`b${y}`,style:`width:${l.toFixed(3)}%;background:${mn(M.tone)};`,title:`${M.label??M.tone}: ${M.count}`}}))});return()=>d()}});var hL=V("sui-meter",{props:{segments:P.string({default:"",reflect:!0}),total:P.number({default:0}),label:P.string({default:"meter"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.segments=u.segments,n.total=u.total,n.label=u.label;let b=null;if(yi&&typeof yi.setup==="function"){if(i.scope=n,b=yi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(f)=>o({anchor:f,items:()=>n.bars(),key:(L,H)=>{let C=Object.create(n);return C.b=L,C.__i=H,((z)=>L.id)(C)},render:(L,H)=>{let C=Object.create(n);return C.b=L,C.__i=H,C.__v=G(0),((z)=>{let j=document.createDocumentFragment(),g=[],Z=document.createElement("div");Z.setAttribute("class","seg"),Z.setAttribute("part","segment");{let D=z.b.style;if(D==null||D===!1||D==="")Z.removeAttribute("style");else Z.setAttribute("style",D===!0?"":String(D))}{let D=z.b.title;if(D==null||D===!1||D==="")Z.removeAttribute("title");else Z.setAttribute("title",D===!0?"":String(D))}return j.appendChild(Z),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let D of g)try{D()}catch(J){}},update:(D,J)=>{z.b=D,z.__i=J;{let X=z.b.style;if(X==null||X===!1||X==="")Z.removeAttribute("style");else Z.setAttribute("style",X===!0?"":String(X))}{let X=z.b.title;if(X==null||X===!1||X==="")Z.removeAttribute("title");else Z.setAttribute("title",X===!0?"":String(X))}z.__v.set((X)=>X+1)}}})(C)}}),r=d.shadowRoot??d;q(r);let M=document.createElement("div");M.setAttribute("class","wrap");let y=document.createElement("div");y.setAttribute("class","bar"),y.setAttribute("part","bar"),y.setAttribute("role","img"),m.push(T(y,"aria-label",()=>n.label()));let l=document.createComment("for");y.appendChild(l),m.push(w(l)),M.appendChild(y);let E=document.createElement("div");E.setAttribute("class","legend");let t=document.createElement("slot");if(E.appendChild(t),M.appendChild(E),r.appendChild(M),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let L=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let j=H[C];if(j!=null&&typeof j.set==="function"){if(typeof z==="function")j.set(()=>z);else j.set(z);return!0}return!1}});if(f.mount){let H=f.mount(L,d);if(typeof H==="function")m.push(H)}if(f.unmount)m.push(()=>f.unmount(L))}}return n}});var wi=N({scope:()=>({cells:G([]),svgW:G(0),svgH:G(0)}),mount(i,u){let d=U(()=>{let m=Sn(i.rows()),n=Sn(i.cols()),b=i.cellSize(),w=i.gap(),r=i.data(),M=rn(r,[]),y=Array.isArray(M)&&M.length>0&&Array.isArray(M[0]),l;if(y)l=M;else{let Z=Yn(r),D=n.length||1,J=m.length||(Z.length>0?Math.ceil(Z.length/D):0);l=Array.from({length:J},(X,W)=>Z.slice(W*D,(W+1)*D))}let E=l.length,t=E>0?Math.max(...l.map((Z)=>Z.length)):0,f=l.flatMap((Z)=>Z.filter((D)=>typeof D==="number")),L=f.length?Math.min(...f):0,C=(f.length?Math.max(...f):1)-L,z=mn(i.tone()),j=b+w;i.svgW.set(t>0?t*b+(t-1)*w:0),i.svgH.set(E>0?E*b+(E-1)*w:0);let g=[];for(let Z=0;Z<E;Z++){let D=l[Z]??[];for(let J=0;J<t;J++){let X=D[J]??0,W=C===0?L>0?1:0:(X-L)/C,Q=(W*100).toFixed(1),Y=W>0?`fill:color-mix(in oklch,${z} ${Q}%,transparent)`:"";g.push({id:`c${Z}-${J}`,x:J*j,y:Z*j,w:b,h:b,fill:Y})}}i.cells.set(g)});return()=>d()}});var _L=V("sui-heatmap",{props:{data:P.string({default:"",reflect:!0}),rows:P.string({default:""}),cols:P.string({default:""}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),"cell-size":P.number({default:20}),gap:P.number({default:3}),label:P.string({default:"heatmap"})},shadow:!0,styles:`
  :host {
    display: inline-block;
    font-family: var(--font-body);
  }
  svg { display: block; overflow: visible; }
  .cell { fill: var(--muted); }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.data=u.data,n.rows=u.rows,n.cols=u.cols,n.tone=u.tone,n.cellSize=u.cellSize,n.gap=u.gap,n.label=u.label;let b=null;if(wi&&typeof wi.setup==="function"){if(i.scope=n,b=wi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>o({anchor:l,items:()=>n.cells(),key:(E,t)=>{let f=Object.create(n);return f.cell=E,f.__i=t,((L)=>E.id)(f)},render:(E,t)=>{let f=Object.create(n);return f.cell=E,f.__i=t,f.__v=G(0),((L)=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("rect");z.setAttribute("class","cell"),z.setAttribute("part","cell");{let g=L.cell.x;if(g==null||g===!1||g==="")z.removeAttribute("x");else z.setAttribute("x",g===!0?"":String(g))}{let g=L.cell.y;if(g==null||g===!1||g==="")z.removeAttribute("y");else z.setAttribute("y",g===!0?"":String(g))}{let g=L.cell.w;if(g==null||g===!1||g==="")z.removeAttribute("width");else z.setAttribute("width",g===!0?"":String(g))}{let g=L.cell.h;if(g==null||g===!1||g==="")z.removeAttribute("height");else z.setAttribute("height",g===!0?"":String(g))}z.setAttribute("rx","2");{let g=L.cell.fill;if(g==null||g===!1||g==="")z.removeAttribute("style");else z.setAttribute("style",g===!0?"":String(g))}let j=document.createTextNode(`
    `);return z.appendChild(j),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let g of C)try{g()}catch(Z){}},update:(g,Z)=>{L.cell=g,L.__i=Z;{let D=L.cell.x;if(D==null||D===!1||D==="")z.removeAttribute("x");else z.setAttribute("x",D===!0?"":String(D))}{let D=L.cell.y;if(D==null||D===!1||D==="")z.removeAttribute("y");else z.setAttribute("y",D===!0?"":String(D))}{let D=L.cell.w;if(D==null||D===!1||D==="")z.removeAttribute("width");else z.setAttribute("width",D===!0?"":String(D))}{let D=L.cell.h;if(D==null||D===!1||D==="")z.removeAttribute("height");else z.setAttribute("height",D===!0?"":String(D))}{let D=L.cell.fill;if(D==null||D===!1||D==="")z.removeAttribute("style");else z.setAttribute("style",D===!0?"":String(D))}L.__v.set((D)=>D+1)}}})(f)}}),r=d.shadowRoot??d;q(r);let M=document.createElement("svg");m.push(T(M,"viewbox",()=>"0 0 "+n.svgW()+" "+n.svgH())),M.setAttribute("role","img"),m.push(T(M,"aria-label",()=>n.label()));let y=document.createComment("for");if(M.appendChild(y),m.push(w(y)),r.appendChild(M),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var ul=V("sui-button",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["solid","soft","outline","ghost","default"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),type:P.union(["button","submit","reset"],{default:"button",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),loading:P.bool({default:!1,reflect:!0}),label:P.string({default:""}),pressed:P.string({default:"",reflect:!0}),expanded:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
  `,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.variant=u.variant,n.size=u.size,n.type=u.type,n.disabled=u.disabled,n.loading=u.loading,n.label=u.label,n.pressed=u.pressed,n.expanded=u.expanded;let b=(f)=>I({anchor:f,test:()=>{if(n.loading())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("span");C.setAttribute("class","spinner-wrap"),C.setAttribute("aria-hidden","true");let z=document.createElement("span");return z.setAttribute("class","spinner"),C.appendChild(z),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let j of H)try{j()}catch(g){}}}}]}),w=d.shadowRoot??d;q(w);let r=document.createElement("button");r.setAttribute("part","button"),m.push(T(r,"type",()=>n.type())),m.push(T(r,"disabled",()=>n.disabled()||n.loading())),m.push(T(r,"aria-busy",()=>n.loading()?"true":"false")),m.push(T(r,"aria-disabled",()=>n.disabled()||n.loading()?"true":"false")),m.push(T(r,"aria-label",()=>n.label())),m.push(T(r,"aria-pressed",()=>n.pressed())),m.push(T(r,"aria-expanded",()=>n.expanded()));let M=document.createElement("span");M.setAttribute("class","content");let y=document.createElement("slot");y.setAttribute("name","leading"),M.appendChild(y);let l=document.createElement("slot");m.push(bn(l,d,"data-icon-only")),M.appendChild(l);let E=document.createElement("slot");E.setAttribute("name","trailing"),M.appendChild(E),r.appendChild(M);let t=document.createComment("if");return r.appendChild(t),m.push(b(t)),w.appendChild(r),n}});var Hi=N({mount(i,u){let d=i,m=G(0),n=()=>{let t=d.tabSlot();if(!t)return Array.from(u.querySelectorAll(":scope > sui-tab"));return t.assignedElements().filter((f)=>f.tagName.toLowerCase()==="sui-tab")},b=(t)=>{if(d.active()===t)return;d.active.set(t),u.dispatchEvent(new CustomEvent("sui-tab-change",{detail:{name:t},bubbles:!0,composed:!0}))},w=U(()=>{m();let t=d.active(),f=n(),L=t;if(!L&&f.length>0){if(L=f[0].getAttribute("name")??"",L)d.active.set(L)}for(let H of f){let C=H.getAttribute("name")??"";if(C===L)H.setAttribute("active","");else H.removeAttribute("active");H.id=`sui-tab-${C}`,H.setAttribute("aria-controls",`sui-tabpanel-${C}`)}for(let H of Array.from(u.querySelectorAll(':scope > sui-tab-panel, :scope > [role="tabpanel"]')))if((H.getAttribute("name")??"")===L)H.removeAttribute("hidden"),H.setAttribute("active","");else H.setAttribute("hidden",""),H.removeAttribute("active")}),r=(t)=>{let f=d.orientation()==="vertical",L=f?"ArrowDown":"ArrowRight",H=f?"ArrowUp":"ArrowLeft",C=[L,H,"Home","End"],z=["Enter"," "];if(!C.includes(t.key)&&!z.includes(t.key))return;t.preventDefault();let j=n().filter((W)=>!W.hasAttribute("disabled"));if(!j.length)return;let g=j.map((W)=>W.getAttribute("name")??""),Z=d.active(),D=Math.max(0,g.indexOf(Z)),J=D;if(t.key===L)J=(D+1)%g.length;else if(t.key===H)J=(D-1+g.length)%g.length;else if(t.key==="Home")J=0;else if(t.key==="End")J=g.length-1;let X=g[J];b(X),queueMicrotask(()=>{let W=j[J],Q=W.shadowRoot?.querySelector("button");Q?Q.focus():W.focus()})},M=(t)=>{let f=t.target.closest("sui-tab");if(!f||f.hasAttribute("disabled"))return;let L=f.getAttribute("name")??"";if(L)b(L)},y=()=>m.set(m()+1),l=d.strip(),E=d.tabSlot();if(l)l.addEventListener("keydown",r),l.addEventListener("click",M);if(E)E.addEventListener("slotchange",y);return()=>{if(w(),l)l.removeEventListener("keydown",r),l.removeEventListener("click",M);if(E)E.removeEventListener("slotchange",y)}}});var Ll=V("sui-tabs",{props:{active:P.string({default:"",reflect:!0}),variant:P.union(["underline","pill","enclosed"],{default:"underline",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),sticky:P.bool({default:!1,reflect:!0}),orientation:P.union(["horizontal","vertical"],{default:"horizontal",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.active=u.active,n.variant=u.variant,n.size=u.size,n.tone=u.tone,n.sticky=u.sticky,n.orientation=u.orientation,n.strip=G(null),n.tabSlot=G(null);let b=null;if(Hi&&typeof Hi.setup==="function"){if(i.scope=n,b=Hi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","strip"),r.setAttribute("part","strip"),r.setAttribute("role","tablist"),m.push(T(r,"aria-orientation",()=>n.orientation())),F(n,"strip",r);let M=document.createElement("slot");if(F(n,"tabSlot",M),r.appendChild(M),w.appendChild(r),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let l=new Proxy(n,{get(E,t){return E[t]},set(E,t,f){let L=E[t];if(L!=null&&typeof L.set==="function"){if(typeof f==="function")L.set(()=>f);else L.set(f);return!0}return!1}});if(y.mount){let E=y.mount(l,d);if(typeof E==="function")m.push(E)}if(y.unmount)m.push(()=>y.unmount(l))}}return n}});var Ci=N({mount(i,u){let d=i,m=U(()=>{let n=d.btn();if(!n)return;n.tabIndex=d.active()?0:-1});return()=>{m()}}});var wl=V("sui-tab",{props:{name:P.string({default:"",reflect:!0}),label:P.string({default:"",reflect:!0}),count:P.string({default:"",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),active:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.label=u.label,n.count=u.count,n.disabled=u.disabled,n.active=u.active,n.btn=G(null);let b=null;if(Ci&&typeof Ci.setup==="function"){if(i.scope=n,b=Ci.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(f)=>I({anchor:f,test:()=>{if(n.count()!=="")return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("span");C.setAttribute("class","count"),C.setAttribute("aria-hidden","true");let z=document.createTextNode("");return C.appendChild(z),H.push(k(z,()=>n.count())),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let j of H)try{j()}catch(g){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createElement("button");M.setAttribute("part","button"),M.setAttribute("type","button"),M.setAttribute("role","tab"),m.push(T(M,"data-name",()=>n.name())),m.push(T(M,"aria-selected",()=>n.active()?"true":"false")),m.push(T(M,"aria-disabled",()=>n.disabled()?"true":"false")),m.push(T(M,"disabled",()=>n.disabled())),F(n,"btn",M);let y=document.createElement("span");y.setAttribute("class","leading"),y.setAttribute("part","leading");let l=document.createElement("slot");l.setAttribute("name","leading"),y.appendChild(l),M.appendChild(y);let E=document.createElement("slot");M.appendChild(E);let t=document.createComment("if");if(M.appendChild(t),m.push(w(t)),r.appendChild(M),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let L=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let j=H[C];if(j!=null&&typeof j.set==="function"){if(typeof z==="function")j.set(()=>z);else j.set(z);return!0}return!1}});if(f.mount){let H=f.mount(L,d);if(typeof H==="function")m.push(H)}if(f.unmount)m.push(()=>f.unmount(L))}}return n}});var zl=V("sui-empty-state",{props:{variant:P.union(["default","inline","placeholder","connecting"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.variant=u.variant,n.size=u.size,n.tone=u.tone;let b=d.shadowRoot??d;q(b);let w=document.createElement("div");w.setAttribute("class","box"),w.setAttribute("part","box"),m.push(T(w,"role",()=>n.variant()==="connecting"?"status":"note")),m.push(T(w,"aria-live",()=>n.variant()==="connecting"?"polite":"off"));let r=document.createElement("div");r.setAttribute("class","icon"),r.setAttribute("part","icon"),r.setAttribute("aria-hidden","true");let M=document.createElement("slot");M.setAttribute("name","icon"),m.push(bn(M,d,"data-no-icon")),r.appendChild(M),w.appendChild(r);let y=document.createElement("div");y.setAttribute("class","copy");let l=document.createElement("div");l.setAttribute("class","title"),l.setAttribute("part","title");let E=document.createElement("slot");E.setAttribute("name","title"),l.appendChild(E),y.appendChild(l);let t=document.createElement("div");t.setAttribute("class","desc"),t.setAttribute("part","desc");let f=document.createElement("slot");t.appendChild(f),y.appendChild(t),w.appendChild(y);let L=document.createElement("div");L.setAttribute("class","actions"),L.setAttribute("part","actions");let H=document.createElement("slot");return H.setAttribute("name","action"),L.appendChild(H),w.appendChild(L),b.appendChild(w),n}});var gm=0,zi=N({mount(i,u){let d=i,m=d.inputEl();if(!m)return;u.expose({focus:()=>m.focus(),blur:()=>m.blur(),clear:()=>{m.value="",d.value.set("")},reportValidity:()=>m.reportValidity()}),_(m,"input",()=>u.emit("sui-input",{value:m.value})),_(m,"change",()=>u.emit("sui-input-change",{value:m.value}));let n=`sui-input-err-${++gm}`;wn(T(m,"aria-describedby",()=>{let b=[];if(d.invalid()&&d.errorMessage()){let r=d.errorEl,M=typeof r==="function"?r():null;if(M)M.id=n,b.push(n)}let w=d.describedBy();if(w)b.push(w);return b.length>0?b.join(" "):null}))}});var Jl=V("sui-input",{props:{type:P.union(["text","search","email","url","tel","password","number"],{default:"text",reflect:!0}),value:P.string({default:""}),placeholder:P.string({default:""}),name:P.string({default:""}),autocomplete:P.string({default:""}),required:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),readonly:P.bool({default:!1,reflect:!0}),invalid:P.bool({default:!1,reflect:!0}),loading:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["default","ghost"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),label:P.string({default:""}),"described-by":P.string({default:""}),"error-message":P.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.type=u.type,n.value=u.value,n.placeholder=u.placeholder,n.name=u.name,n.autocomplete=u.autocomplete,n.required=u.required,n.disabled=u.disabled,n.readonly=u.readonly,n.invalid=u.invalid,n.loading=u.loading,n.tone=u.tone,n.variant=u.variant,n.size=u.size,n.label=u.label,n.describedBy=u.describedBy,n.errorMessage=u.errorMessage,n.errorEl=G(null),n.inputEl=G(null);let b=null;if(zi&&typeof zi.setup==="function"){if(i.scope=n,b=zi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>I({anchor:L,test:()=>{if(n.loading())return 0;return 1},branches:[()=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("span");return z.setAttribute("class","spinner"),z.setAttribute("aria-hidden","true"),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let j of C)try{j()}catch(g){}}}},()=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("slot");return z.setAttribute("name","trailing"),C.push(bn(z,d,"data-no-trailing")),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let j of C)try{j()}catch(g){}}}}]}),r=(L)=>I({anchor:L,test:()=>{if(n.invalid()&&n.errorMessage())return 0;return-1},branches:[()=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("span");z.setAttribute("class","error"),z.setAttribute("role","alert"),F(n,"errorEl",z);let j=document.createTextNode("");return z.appendChild(j),C.push(k(j,()=>n.errorMessage())),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let g of C)try{g()}catch(Z){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createElement("div");y.setAttribute("class","wrap"),y.setAttribute("part","wrap");let l=document.createElement("slot");l.setAttribute("name","leading"),m.push(bn(l,d,"data-no-leading")),y.appendChild(l);let E=document.createElement("input");E.setAttribute("part","input"),m.push(T(E,"type",()=>n.type())),m.push(T(E,"placeholder",()=>n.placeholder())),m.push(T(E,"name",()=>n.name())),m.push(T(E,"autocomplete",()=>n.autocomplete()||"off")),m.push(T(E,"disabled",()=>n.disabled())),m.push(T(E,"readonly",()=>n.readonly())),m.push(T(E,"required",()=>n.required())),m.push(T(E,"aria-label",()=>n.label())),m.push(T(E,"aria-invalid",()=>n.invalid()?"true":"false")),m.push(T(E,"aria-required",()=>n.required()?"true":"false")),m.push(T(E,"aria-busy",()=>n.loading()?"true":"false")),m.push(gn(E,n.value)),F(n,"inputEl",E),y.appendChild(E);let t=document.createComment("if");y.appendChild(t),m.push(w(t)),M.appendChild(y);let f=document.createComment("if");if(M.appendChild(f),m.push(r(f)),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let H=new Proxy(n,{get(C,z){return C[z]},set(C,z,j){let g=C[z];if(g!=null&&typeof g.set==="function"){if(typeof j==="function")g.set(()=>j);else g.set(j);return!0}return!1}});if(L.mount){let C=L.mount(H,d);if(typeof C==="function")m.push(C)}if(L.unmount)m.push(()=>L.unmount(H))}}return n}});var Dm={git:"git-branch",refresh:"refresh-cw",stop:"stop-circle",warning:"triangle-alert"};var Hn=null,vn=null;function ji(){return Hn}function id(){if(Hn)return Promise.resolve(Hn);if(vn)return vn;return vn=fetch("/_sui/lucide-icons.json").then((i)=>{if(!i.ok)throw Error(`lucide registry fetch ${i.status}`);return i.json()}).then((i)=>{return Hn=i,i}).catch((i)=>{throw vn=null,i}),vn}function ud(i){if(!i||!Hn)return;if(Hn[i])return i;let u=Dm[i];if(u&&Hn[u])return u;return}function dd(i){return Hn?Hn[i]:void 0}var md="http://www.w3.org/2000/svg",Du=G(ji()?1:0),gu=!1;function Zm(){if(gu||ji())return;gu=!0,id().then(()=>Du.set(Du()+1)).catch(()=>{gu=!1})}function Jm(i){let u=i.createElementNS(md,"svg");return u.setAttribute("part","svg"),u.setAttribute("class","sui-icon-svg"),u.setAttribute("viewBox","0 0 24 24"),u.setAttribute("fill","none"),u.setAttribute("stroke","currentColor"),u.setAttribute("stroke-width","2"),u.setAttribute("stroke-linecap","round"),u.setAttribute("stroke-linejoin","round"),u}function Xm(i,u){let d=new DOMParser().parseFromString(`<svg xmlns="${md}">${u}</svg>`,"image/svg+xml").documentElement,m=[];while(d.firstChild)m.push(i.ownerDocument.adoptNode(d.firstChild));i.replaceChildren(...m)}var Pi=N({mount(i,u){let d=i,m=u.shadowRoot,n=null,b=null,w=U(()=>{let M=d.label();if(M)u.setAttribute("role","img"),u.setAttribute("aria-label",M),u.removeAttribute("aria-hidden");else u.setAttribute("aria-hidden","true"),u.removeAttribute("role"),u.removeAttribute("aria-label")}),r=U(()=>{Du();let M=d.name(),y=ud(M)??null,l=d.label();if(!y){if(n)n.remove(),n=null;if(b=null,M&&!ji())Zm();return}if(!n)n=Jm(u.ownerDocument),m.insertBefore(n,m.firstChild);if(y!==b)Xm(n,dd(y)),b=y;if(l)n.removeAttribute("aria-hidden");else n.setAttribute("aria-hidden","true")});return()=>{w(),r()}}});var Yl=V("sui-icon",{props:{name:P.string({default:"",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:P.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),label:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.size=u.size,n.tone=u.tone,n.label=u.label;let b=null;if(Pi&&typeof Pi.setup==="function"){if(i.scope=n,b=Pi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");if(w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var gi=N({mount(i,u){let d=i,m=()=>Array.from(u.querySelectorAll(":scope > option, :scope > optgroup > option")),n=()=>{let f=d.value(),L=m();if(L.length===0)return f;let H=L.find((z)=>z.value===f)??L.find((z)=>z.hasAttribute("selected"))??L[0];return(H?.textContent??"").trim()||H?.value||f},b=()=>{let f=d.selectEl();if(!f){let H=m(),C=H.find((z)=>z.value===d.value())??H.find((z)=>z.hasAttribute("selected"))??H[0];if(!C||C.value==="")u.setAttribute("data-placeholder","");else u.removeAttribute("data-placeholder");return}let L=f.selectedOptions?.[0]??f.options[f.selectedIndex];if(!L||L.value==="")u.setAttribute("data-placeholder","");else u.removeAttribute("data-placeholder")},w=()=>{let f=d.selectEl();if(!f)return;let L=u.querySelectorAll(":scope > option, :scope > optgroup");f.replaceChildren(...Array.from(L,(H)=>H.cloneNode(!0))),f.value=d.value()},r=()=>{let f=d.valueLabel();if(f)f.textContent=n()},M=U(()=>{let f=d.value(),L=d.selectEl();if(L)L.value=f;r(),b()}),y=(f)=>{let L=f.target.value;d.value.set(L),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:L},bubbles:!0,composed:!0}))},l=d.selectEl();if(l)l.addEventListener("change",y);let E=d.optionsSlot(),t=()=>{w(),r(),b()};if(E)E.addEventListener("slotchange",t);return w(),r(),b(),()=>{if(l)l.removeEventListener("change",y);if(E)E.removeEventListener("slotchange",t);M()}}});var Kl=V("sui-select",{props:{value:P.string({default:""}),name:P.string({default:""}),required:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),invalid:P.bool({default:!1,reflect:!0}),tone:P.union(["default","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),label:P.string({default:""}),"described-by":P.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.name=u.name,n.required=u.required,n.disabled=u.disabled,n.invalid=u.invalid,n.tone=u.tone,n.size=u.size,n.label=u.label,n.describedBy=u.describedBy,n.optionsSlot=G(null),n.selectEl=G(null),n.valueLabel=G(null);let b=null;if(gi&&typeof gi.setup==="function"){if(i.scope=n,b=gi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","field"),r.setAttribute("part","field");let M=document.createElement("span");M.setAttribute("class","value"),M.setAttribute("aria-hidden","true"),F(n,"valueLabel",M),r.appendChild(M);let y=document.createElement("svg");y.setAttribute("class","chevron"),y.setAttribute("part","chevron"),y.setAttribute("viewbox","0 0 16 16"),y.setAttribute("aria-hidden","true"),y.setAttribute("focusable","false"),y.setAttribute("fill","none"),y.setAttribute("stroke","currentColor"),y.setAttribute("stroke-width","1.75"),y.setAttribute("stroke-linecap","round"),y.setAttribute("stroke-linejoin","round");let l=document.createElement("path");l.setAttribute("d","M4 6l4 4 4-4"),y.appendChild(l),r.appendChild(y);let E=document.createElement("select");E.setAttribute("part","select"),m.push(T(E,"name",()=>n.name())),m.push(T(E,"disabled",()=>n.disabled())),m.push(T(E,"required",()=>n.required())),m.push(T(E,"aria-invalid",()=>n.invalid()?"true":"false")),m.push(T(E,"aria-label",()=>n.label())),m.push(T(E,"aria-describedby",()=>n.describedBy())),F(n,"selectEl",E),r.appendChild(E),w.appendChild(r);let t=document.createElement("span");t.setAttribute("hidden","");let f=document.createElement("slot");if(F(n,"optionsSlot",f),t.appendChild(f),w.appendChild(t),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let H=new Proxy(n,{get(C,z){return C[z]},set(C,z,j){let g=C[z];if(g!=null&&typeof g.set==="function"){if(typeof j==="function")g.set(()=>j);else g.set(j);return!0}return!1}});if(L.mount){let C=L.mount(H,d);if(typeof C==="function")m.push(C)}if(L.unmount)m.push(()=>L.unmount(H))}}return n}});var Rl=V("sui-separator",{props:{orientation:P.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:P.union(["solid","dashed"],{default:"solid",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.orientation=u.orientation,n.variant=u.variant;let b=d.shadowRoot??d;q(b);let w=document.createElement("hr");return w.setAttribute("class","sep"),w.setAttribute("part","sep"),w.setAttribute("role","separator"),m.push(T(w,"aria-orientation",()=>n.orientation())),b.appendChild(w),n}});var Di=N({mount(i,u){let d=i,m=U(()=>{let n=d.labelEl();if(!n)return;n.htmlFor=d.for?.()??""});return()=>{m()}}});var hl=V("sui-label",{props:{for:P.string({default:""}),size:P.union(["sm","md"],{default:"sm",reflect:!0}),required:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.for=u.for,n.size=u.size,n.required=u.required,n.labelEl=G(null);let b=null;if(Di&&typeof Di.setup==="function"){if(i.scope=n,b=Di.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(E)=>I({anchor:E,test:()=>{if(n.required())return 0;return-1},branches:[()=>{let t=document.createDocumentFragment(),f=[],L=document.createElement("span");L.setAttribute("class","req"),L.setAttribute("aria-hidden","true");let H=document.createTextNode("*");return L.appendChild(H),t.appendChild(L),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let C of f)try{C()}catch(z){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createElement("label");M.setAttribute("part","label"),F(n,"labelEl",M);let y=document.createElement("slot");M.appendChild(y);let l=document.createComment("if");if(M.appendChild(l),m.push(w(l)),r.appendChild(M),b!=null){let E=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(E){let t=new Proxy(n,{get(f,L){return f[L]},set(f,L,H){let C=f[L];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(E.mount){let f=E.mount(t,d);if(typeof f==="function")m.push(f)}if(E.unmount)m.push(()=>E.unmount(t))}}return n}});var Zi=N({mount(i,u){let d=i,m=U(()=>{let n=d.closeBtn();if(!n)return;let b=()=>{if(!d.dismissible())return;u.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),u.remove()};return n.addEventListener("click",b),()=>n.removeEventListener("click",b)});return()=>{m()}}});var cl=V("sui-alert",{props:{tone:P.union(["info","success","warning","destructive","muted"],{default:"info",reflect:!0}),dismissible:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.dismissible=u.dismissible,n.closeBtn=G(null);let b=null;if(Zi&&typeof Zi.setup==="function"){if(i.scope=n,b=Zi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","box"),r.setAttribute("part","box"),m.push(T(r,"role",()=>n.tone()==="destructive"||n.tone()==="warning"?"alert":"status")),m.push(T(r,"aria-live",()=>n.tone()==="destructive"||n.tone()==="warning"?"assertive":"polite")),r.setAttribute("aria-atomic","true");let M=document.createElement("span"),y=document.createElement("slot");y.setAttribute("name","icon"),M.appendChild(y),r.appendChild(M);let l=document.createElement("div");l.setAttribute("class","content");let E=document.createElement("div");E.setAttribute("class","title");let t=document.createElement("slot");t.setAttribute("name","title"),E.appendChild(t),l.appendChild(E);let f=document.createElement("div");f.setAttribute("class","body");let L=document.createElement("slot");f.appendChild(L),l.appendChild(f);let H=document.createElement("div");H.setAttribute("class","actions");let C=document.createElement("slot");C.setAttribute("name","actions"),H.appendChild(C),l.appendChild(H),r.appendChild(l);let z=document.createElement("button");z.setAttribute("class","close"),z.setAttribute("part","close"),z.setAttribute("aria-label","Dismiss"),F(n,"closeBtn",z);let j=document.createTextNode("×");if(z.appendChild(j),r.appendChild(z),w.appendChild(r),b!=null){let g=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(g){let Z=new Proxy(n,{get(D,J){return D[J]},set(D,J,X){let W=D[J];if(W!=null&&typeof W.set==="function"){if(typeof X==="function")W.set(()=>X);else W.set(X);return!0}return!1}});if(g.mount){let D=g.mount(Z,d);if(typeof D==="function")m.push(D)}if(g.unmount)m.push(()=>g.unmount(Z))}}return n}});var Gm=0,Ji=N({mount(i,u){let d=i,m=`sui-tooltip-${++Gm}`,n=U(()=>{let M=d.bubbleEl();if(M)M.id=m}),b=()=>{let M=d.defaultSlot();if(!M)return;let y=M.assignedElements({flatten:!0})[0];if(!y)return;let l=y.getAttribute("aria-describedby")??"";if(!l.split(/\s+/).includes(m))y.setAttribute("aria-describedby",l?`${l} ${m}`:m)},w=U(()=>{let M=d.defaultSlot();if(!M)return;return M.addEventListener("slotchange",b),b(),()=>M.removeEventListener("slotchange",b)}),r=(M)=>{if(M.key==="Escape")u.blur()};return u.addEventListener("keydown",r),()=>{n(),w(),u.removeEventListener("keydown",r)}}});var mE=V("sui-tooltip",{props:{placement:P.union(["top","bottom","left","right"],{default:"top",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.placement=u.placement,n.bubbleEl=G(null),n.defaultSlot=G(null);let b=null;if(Ji&&typeof Ji.setup==="function"){if(i.scope=n,b=Ji.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");F(n,"defaultSlot",r),w.appendChild(r);let M=document.createElement("span");M.setAttribute("class","bubble"),M.setAttribute("role","tooltip"),M.setAttribute("part","bubble"),F(n,"bubbleEl",M);let y=document.createElement("slot");if(y.setAttribute("name","content"),M.appendChild(y),w.appendChild(M),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var Wm=0,Xi=N({mount(i,u){let d=i,m=`sui-collapsible-${++Wm}`,n=U(()=>{let E=d.contentWrap();if(E)E.id=m}),b=()=>{let E=d.triggerSlot();if(!E)return;let t=d.open();for(let f of E.assignedElements()){let L=f;if(L.setAttribute("aria-expanded",String(t)),L.setAttribute("aria-controls",m),!L.hasAttribute("tabindex")&&!L.matches("button, a, input, select, textarea"))L.setAttribute("tabindex","0")}},w=U(()=>{let E=d.triggerSlot();if(!E)return;return E.addEventListener("slotchange",b),b(),()=>E.removeEventListener("slotchange",b)}),r=U(()=>{d.open(),d.triggerSlot(),b()}),M=()=>{if(d.disabled())return;let E=!d.open();d.open.set(E),u.dispatchEvent(new CustomEvent("sui-toggle",{detail:{open:E},bubbles:!0,composed:!0}))},y=(E)=>{if(E.key==="Enter"||E.key===" ")E.preventDefault(),M()},l=U(()=>{let E=d.triggerWrap();if(!E)return;return E.addEventListener("click",M),E.addEventListener("keydown",y),()=>{E.removeEventListener("click",M),E.removeEventListener("keydown",y)}});return()=>{n(),w(),r(),l()}}});var EE=V("sui-collapsible",{props:{open:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.disabled=u.disabled,n.contentWrap=G(null),n.triggerSlot=G(null),n.triggerWrap=G(null);let b=null;if(Xi&&typeof Xi.setup==="function"){if(i.scope=n,b=Xi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","trigger-wrap"),F(n,"triggerWrap",r);let M=document.createElement("slot");M.setAttribute("name","trigger"),F(n,"triggerSlot",M),r.appendChild(M),w.appendChild(r);let y=document.createElement("div");y.setAttribute("class","content-wrap"),y.setAttribute("part","content"),F(n,"contentWrap",y);let l=document.createElement("div");l.setAttribute("class","content-inner");let E=document.createElement("slot");if(l.appendChild(E),y.appendChild(l),w.appendChild(y),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let f=new Proxy(n,{get(L,H){return L[H]},set(L,H,C){let z=L[H];if(z!=null&&typeof z.set==="function"){if(typeof C==="function")z.set(()=>C);else z.set(C);return!0}return!1}});if(t.mount){let L=t.mount(f,d);if(typeof L==="function")m.push(L)}if(t.unmount)m.push(()=>t.unmount(f))}}return n}});var Gi=N({mount(i,u){let d=i,m=U(()=>{let b=d.inputEl();if(!b)return;b.indeterminate=d.indeterminate()}),n=U(()=>{let b=d.inputEl();if(!b)return;let w=()=>{d.indeterminate.set(!1),u.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:b.checked,value:d.value()},bubbles:!0,composed:!0}))};return b.addEventListener("change",w),()=>b.removeEventListener("change",w)});return()=>{m(),n()}}});var CE=V("sui-checkbox",{props:{checked:P.bool({default:!1,reflect:!0}),indeterminate:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),required:P.bool({default:!1,reflect:!0}),invalid:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),name:P.string({default:""}),value:P.string({default:"on"}),label:P.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.checked=u.checked,n.indeterminate=u.indeterminate,n.disabled=u.disabled,n.required=u.required,n.invalid=u.invalid,n.tone=u.tone,n.size=u.size,n.name=u.name,n.value=u.value,n.label=u.label,n.inputEl=G(null);let b=null;if(Gi&&typeof Gi.setup==="function"){if(i.scope=n,b=Gi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("label");r.setAttribute("part","label");let M=document.createElement("span");M.setAttribute("class","box"),M.setAttribute("part","box");let y=document.createElement("input");y.setAttribute("part","input"),y.setAttribute("type","checkbox"),m.push(Dn(y,n.checked)),m.push(T(y,"disabled",()=>n.disabled())),m.push(T(y,"required",()=>n.required())),m.push(T(y,"name",()=>n.name())),m.push(T(y,"value",()=>n.value())),m.push(T(y,"aria-checked",()=>n.indeterminate()?"mixed":n.checked()?"true":"false")),m.push(T(y,"aria-invalid",()=>n.invalid()?"true":"false")),m.push(T(y,"aria-label",()=>n.label()||n.name()||"checkbox")),F(n,"inputEl",y),M.appendChild(y);let l=document.createElement("span");l.setAttribute("class","glyph check"),l.setAttribute("aria-hidden","true"),M.appendChild(l);let E=document.createElement("span");E.setAttribute("class","glyph mixed"),E.setAttribute("aria-hidden","true"),M.appendChild(E),r.appendChild(M);let t=document.createElement("slot");if(r.appendChild(t),w.appendChild(r),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let L=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let j=H[C];if(j!=null&&typeof j.set==="function"){if(typeof z==="function")j.set(()=>z);else j.set(z);return!0}return!1}});if(f.mount){let H=f.mount(L,d);if(typeof H==="function")m.push(H)}if(f.unmount)m.push(()=>f.unmount(L))}}return n}});var Wi=N({mount(i,u){let d=i,m=U(()=>{let n=d.inputEl();if(!n)return;let b=()=>{u.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:n.checked},bubbles:!0,composed:!0}))};return n.addEventListener("change",b),()=>n.removeEventListener("change",b)});return()=>m()}});var ZE=V("sui-switch",{props:{checked:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),name:P.string({default:""}),value:P.string({default:"on"}),label:P.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.checked=u.checked,n.disabled=u.disabled,n.tone=u.tone,n.size=u.size,n.name=u.name,n.value=u.value,n.label=u.label,n.inputEl=G(null);let b=null;if(Wi&&typeof Wi.setup==="function"){if(i.scope=n,b=Wi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("label"),M=document.createElement("span");M.setAttribute("class","track"),M.setAttribute("part","track");let y=document.createElement("input");y.setAttribute("part","input"),y.setAttribute("type","checkbox"),y.setAttribute("role","switch"),m.push(Dn(y,n.checked)),m.push(T(y,"disabled",()=>n.disabled())),m.push(T(y,"name",()=>n.name())),m.push(T(y,"value",()=>n.value())),m.push(T(y,"aria-checked",()=>n.checked()?"true":"false")),m.push(T(y,"aria-label",()=>n.label()||n.name()||"toggle")),F(n,"inputEl",y),M.appendChild(y);let l=document.createElement("span");l.setAttribute("class","thumb"),l.setAttribute("part","thumb"),M.appendChild(l),r.appendChild(M);let E=document.createElement("slot");if(r.appendChild(E),w.appendChild(r),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let f=new Proxy(n,{get(L,H){return L[H]},set(L,H,C){let z=L[H];if(z!=null&&typeof z.set==="function"){if(typeof C==="function")z.set(()=>C);else z.set(C);return!0}return!1}});if(t.mount){let L=t.mount(f,d);if(typeof L==="function")m.push(L)}if(t.unmount)m.push(()=>t.unmount(f))}}return n}});var Ti=N({mount(i,u){let d=i,m=d.value(),n=U(()=>{let b=d.textareaEl();if(!b)return;let w=()=>{u.dispatchEvent(new CustomEvent("sui-input",{detail:{value:b.value},bubbles:!0,composed:!0}))},r=()=>{let M=d.value();if(M!==m)m=M,u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:M},bubbles:!0,composed:!0}))};return b.addEventListener("input",w),b.addEventListener("blur",r),()=>{b.removeEventListener("input",w),b.removeEventListener("blur",r)}});return()=>n()}});var QE=V("sui-textarea",{props:{value:P.string({default:""}),placeholder:P.string({default:""}),name:P.string({default:""}),rows:P.number({default:4}),required:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),readonly:P.bool({default:!1,reflect:!0}),invalid:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["default","code"],{default:"default",reflect:!0}),size:P.union(["sm","md"],{default:"md",reflect:!0}),resize:P.union(["none","vertical","horizontal","both"],{default:"vertical"}),label:P.string({default:""}),"described-by":P.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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

`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.placeholder=u.placeholder,n.name=u.name,n.rows=u.rows,n.required=u.required,n.disabled=u.disabled,n.readonly=u.readonly,n.invalid=u.invalid,n.tone=u.tone,n.variant=u.variant,n.size=u.size,n.resize=u.resize,n.label=u.label,n.describedBy=u.describedBy,n.textareaEl=G(null);let b=null;if(Ti&&typeof Ti.setup==="function"){if(i.scope=n,b=Ti.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("textarea");if(r.setAttribute("part","textarea"),m.push(T(r,"placeholder",()=>n.placeholder())),m.push(T(r,"name",()=>n.name())),m.push(T(r,"rows",()=>n.rows())),m.push(T(r,"disabled",()=>n.disabled())),m.push(T(r,"required",()=>n.required())),m.push(T(r,"readonly",()=>n.readonly())),m.push(T(r,"aria-invalid",()=>n.invalid()?"true":"false")),m.push(T(r,"aria-required",()=>n.required()?"true":"false")),m.push(T(r,"aria-label",()=>n.label())),m.push(T(r,"aria-describedby",()=>n.describedBy())),m.push(U(()=>{let M=n.resize();if(M==null)r.style.removeProperty("resize");else r.style.setProperty("resize",String(M))})),m.push(gn(r,n.value)),F(n,"textareaEl",r),w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var bd=0,Qi=N({mount(i,u){let d=i,m=`sui-field-hint-${++bd}`,n=`sui-field-error-${++bd}`,b=U(()=>{let y=d.hintEl(),l=d.errorEl();if(y)y.id=m;if(l)l.id=n}),w=()=>{let l=d.controlSlot()?.assignedElements({flatten:!0})[0];if(!l)return;let E=d.invalid(),t=E?n:m,L=[...(l.getAttribute("aria-describedby")||"").split(/\s+/).filter((H)=>H&&H!==m&&H!==n),t].join(" ").trim();if(L)l.setAttribute("aria-describedby",L);if("invalid"in l)l.invalid=E;else l.setAttribute("aria-invalid",E?"true":"false")},r=U(()=>{d.invalid(),w()}),M=U(()=>{let y=d.controlSlot();if(!y)return;return y.addEventListener("slotchange",w),()=>y.removeEventListener("slotchange",w)});return()=>{b(),r(),M()}}});var FE=V("sui-field",{props:{invalid:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.invalid=u.invalid,n.controlSlot=G(null),n.errorEl=G(null),n.hintEl=G(null);let b=null;if(Qi&&typeof Qi.setup==="function"){if(i.scope=n,b=Qi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");r.setAttribute("name","label"),w.appendChild(r);let M=document.createElement("slot");F(n,"controlSlot",M),w.appendChild(M);let y=document.createElement("div");y.setAttribute("class","hint"),F(n,"hintEl",y);let l=document.createElement("slot");l.setAttribute("name","hint"),y.appendChild(l),w.appendChild(y);let E=document.createElement("div");E.setAttribute("class","error"),E.setAttribute("role","alert"),E.setAttribute("aria-live","polite"),F(n,"errorEl",E);let t=document.createElement("slot");if(t.setAttribute("name","error"),E.appendChild(t),w.appendChild(E),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let L=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let j=H[C];if(j!=null&&typeof j.set==="function"){if(typeof z==="function")j.set(()=>z);else j.set(z);return!0}return!1}});if(f.mount){let H=f.mount(L,d);if(typeof H==="function")m.push(H)}if(f.unmount)m.push(()=>f.unmount(L))}}return n}});var Vi=N({scope:(i,u)=>{let d=i,m=e(null),n=h(()=>d.tone()==="destructive"?"destructive":"default");function b(M){u.dispatchEvent(new CustomEvent(M,{bubbles:!0,composed:!0})),d.open.set(!1)}function w(){b("sui-confirm")}function r(){b("sui-cancel")}return{dialogRef:m,confirmTone:n,onConfirm:w,onCancel:r}},mount(i,u){let d=i,m=d.open,n=d.dialogRef,b=U(()=>{let r=m(),M=n();if(!M)return;if(r&&typeof M.show==="function")M.show();else if(!r&&typeof M.close==="function")M.close()}),w=()=>{if(m())m.set(!1)};return u.addEventListener("sui-close",w),()=>{b(),u.removeEventListener("sui-close",w)}}});var RE=V("sui-alert-dialog",{props:{open:P.bool({default:!1,reflect:!0}),tone:P.union(["default","destructive"],{default:"default",reflect:!0}),"confirm-label":P.string({default:"Confirm"}),"cancel-label":P.string({default:"Cancel"})},shadow:!0,styles:`
  :host { display: contents; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.tone=u.tone,n.confirmLabel=u.confirmLabel,n.cancelLabel=u.cancelLabel,n.dialogRef=G(null);let b=null;if(Vi&&typeof Vi.setup==="function"){if(i.scope=n,b=Vi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("sui-dialog");r.setAttribute("role-override","alertdialog"),r.setAttribute("size","sm"),r.setAttribute("no-close-button",""),F(n,"dialogRef",r);let M=document.createElement("span");M.setAttribute("slot","header");let y=document.createElement("slot");y.setAttribute("name","title"),M.appendChild(y),r.appendChild(M);let l=document.createElement("slot");r.appendChild(l);let E=document.createElement("span");E.setAttribute("slot","footer");let t=document.createElement("sui-button");t.setAttribute("variant","outline"),m.push(p(t,"click",function(){let C=n.onCancel;return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let f=document.createTextNode("");t.appendChild(f),m.push(k(f,()=>n.cancelLabel())),E.appendChild(t);let L=document.createElement("sui-button");L.setAttribute("variant","solid"),m.push(T(L,"tone",()=>n.confirmTone())),m.push(p(L,"click",function(){let C=n.onConfirm;return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let H=document.createTextNode("");if(L.appendChild(H),m.push(k(H,()=>n.confirmLabel())),E.appendChild(L),r.appendChild(E),w.appendChild(r),b!=null){let C=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(n,{get(j,g){return j[g]},set(j,g,Z){let D=j[g];if(D!=null&&typeof D.set==="function"){if(typeof Z==="function")D.set(()=>Z);else D.set(Z);return!0}return!1}});if(C.mount){let j=C.mount(z,d);if(typeof j==="function")m.push(j)}if(C.unmount)m.push(()=>C.unmount(z))}}return n}});function fd(i){if(i<1024)return`${i} B`;if(i<1048576)return`${(i/1024).toFixed(1)} KB`;return`${(i/1048576).toFixed(1)} MB`}var qi=N({scope:()=>{let i=G([]),u=G(!1),d=G(""),m=e(null),n=e(null),b=e(null),w=h(()=>i().map((r,M)=>({file:r,idx:M,key:`${M}:${r.name}:${r.size}`,name:r.name,sizeStr:fd(r.size)})));return{files:i,over:u,errorMsg:d,displayFiles:w,dropzoneEl:m,inputEl:n,fileListEl:b}},mount(i,u){let d=(C)=>{let z=i.accept().split(",").map((Z)=>Z.trim().toLowerCase()).filter(Boolean);if(z.length===0)return!0;let j="."+(C.name.split(".").pop()||"").toLowerCase(),g=C.type.toLowerCase();return z.some((Z)=>{if(Z.startsWith("."))return j===Z;if(Z.endsWith("/*"))return g.startsWith(Z.slice(0,-1));return g===Z})},m=(C)=>{u.dispatchEvent(new CustomEvent("sui-change",{detail:{files:[...C]},bubbles:!0,composed:!0}))},n=(C)=>{i.errorMsg.set("");let z=[];for(let Z of C){if(Z.size>i.maxSize()){i.errorMsg.set(`"${Z.name}" exceeds max size (${fd(i.maxSize())}).`);continue}if(i.accept()&&!d(Z)){i.errorMsg.set(`"${Z.name}" is not an accepted file type.`);continue}z.push(Z)}let j=i.files(),g=i.multiple()?[...j,...z]:z.slice(0,1);i.files.set(g),m(g)},b=(C)=>{let z=i.files();if(C<0||C>=z.length)return;let j=z.filter((g,Z)=>Z!==C);i.files.set(j),i.errorMsg.set(""),m(j)},w=()=>{if(i.disabled())return;i.inputEl()?.click()},r=()=>w(),M=(C)=>{if(i.disabled())return;if(C.key==="Enter"||C.key===" ")C.preventDefault(),w()},y=(C)=>{if(i.disabled())return;C.preventDefault(),i.over.set(!0)},l=()=>i.over.set(!1),E=(C)=>{if(i.disabled())return;C.preventDefault(),i.over.set(!1);let z=C.dataTransfer?.files;if(z&&z.length)n(Array.from(z))},t=(C)=>{let z=C.target;if(z.files&&z.files.length)n(Array.from(z.files));z.value=""},f=i.dropzoneEl(),L=i.inputEl();f?.addEventListener("click",r),f?.addEventListener("keydown",M),f?.addEventListener("dragover",y),f?.addEventListener("dragleave",l),f?.addEventListener("drop",E),L?.addEventListener("change",t);let H=(C)=>{let j=C.target?.closest?.("button.remove");if(!j||!u.shadowRoot?.contains(j))return;let g=Number(j.dataset.idx);if(Number.isInteger(g))b(g)};return u.shadowRoot?.addEventListener("click",H),()=>{f?.removeEventListener("click",r),f?.removeEventListener("keydown",M),f?.removeEventListener("dragover",y),f?.removeEventListener("dragleave",l),f?.removeEventListener("drop",E),L?.removeEventListener("change",t),u.shadowRoot?.removeEventListener("click",H)}}});var hE=V("sui-file-upload",{props:{accept:P.string({default:""}),multiple:P.bool({default:!1}),"max-size":P.number({default:10485760}),disabled:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.accept=u.accept,n.multiple=u.multiple,n.maxSize=u.maxSize,n.disabled=u.disabled,n.tone=u.tone,n.dropzoneEl=G(null),n.fileListEl=G(null),n.inputEl=G(null);let b=null;if(qi&&typeof qi.setup==="function"){if(i.scope=n,b=qi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(j)=>I({anchor:j,test:()=>{if(n.errorMsg())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),Z=[],D=document.createElement("div");D.setAttribute("class","error"),D.setAttribute("role","alert"),D.setAttribute("part","error");let J=document.createTextNode("");return D.appendChild(J),Z.push(k(J,()=>n.errorMsg())),g.appendChild(D),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let X of Z)try{X()}catch(W){}}}}]}),r=(j)=>o({anchor:j,items:()=>n.displayFiles(),key:(g,Z)=>{let D=Object.create(n);return D.f=g,D.__i=Z,((J)=>g.key)(D)},render:(g,Z)=>{let D=Object.create(n);return D.f=g,D.__i=Z,D.__v=G(0),((J)=>{let X=document.createDocumentFragment(),W=[],Q=document.createElement("div");Q.setAttribute("class","file-item"),Q.setAttribute("part","file-item");let Y=document.createElement("span");Y.setAttribute("class","file-name");let O=document.createTextNode("");Y.appendChild(O),O.textContent=J.f.name,Q.appendChild(Y);let K=document.createElement("span");K.setAttribute("class","file-size");let B=document.createTextNode("");K.appendChild(B),B.textContent=J.f.sizeStr,Q.appendChild(K);let A=document.createElement("button");A.setAttribute("class","remove"),A.setAttribute("part","remove"),A.setAttribute("type","button");{let S=J.f.idx;if(S==null||S===!1||S==="")A.removeAttribute("data-idx");else A.setAttribute("data-idx",S===!0?"":String(S))}{let S="Remove "+J.f.name;if(S==null||S===!1||S==="")A.removeAttribute("aria-label");else A.setAttribute("aria-label",S===!0?"":String(S))}let a=document.createTextNode("×");return A.appendChild(a),Q.appendChild(A),X.appendChild(Q),{nodes:Array.from(X.childNodes),cleanup:()=>{for(let S of W)try{S()}catch(v){}},update:(S,v)=>{J.f=S,J.__i=v,O.textContent=J.f.name,B.textContent=J.f.sizeStr;{let R=J.f.idx;if(R==null||R===!1||R==="")A.removeAttribute("data-idx");else A.setAttribute("data-idx",R===!0?"":String(R))}{let R="Remove "+J.f.name;if(R==null||R===!1||R==="")A.removeAttribute("aria-label");else A.setAttribute("aria-label",R===!0?"":String(R))}J.__v.set((R)=>R+1)}}})(D)}}),M=(j)=>I({anchor:j,test:()=>{if(n.displayFiles().length>0)return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),Z=[],D=document.createElement("div");D.setAttribute("class","file-list"),D.setAttribute("part","file-list"),D.setAttribute("aria-live","polite"),F(n,"fileListEl",D);let J=document.createComment("for");return D.appendChild(J),Z.push(r(J)),g.appendChild(D),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let X of Z)try{X()}catch(W){}}}}]}),y=d.shadowRoot??d;q(y);let l=document.createElement("div");l.setAttribute("class","dropzone"),m.push(zn(l,"over",()=>n.over())),l.setAttribute("part","dropzone"),l.setAttribute("role","button"),m.push(T(l,"tabindex",()=>n.disabled()?"-1":"0")),m.push(T(l,"aria-label",()=>n.accept()?"Upload files ("+n.accept()+")":"Upload files")),m.push(T(l,"aria-disabled",()=>n.disabled()?"true":"false")),F(n,"dropzoneEl",l);let E=document.createElement("span");E.setAttribute("class","icon"),E.setAttribute("aria-hidden","true");let t=document.createTextNode("\uD83D\uDCC4");E.appendChild(t),l.appendChild(E);let f=document.createElement("span"),L=document.createTextNode("Drop files here or click to browse");f.appendChild(L),l.appendChild(f),y.appendChild(l);let H=document.createElement("input");H.setAttribute("type","file"),H.setAttribute("part","input"),m.push(T(H,"accept",()=>n.accept())),m.push(T(H,"multiple",()=>n.multiple())),H.setAttribute("aria-hidden","true"),H.setAttribute("tabindex","-1"),F(n,"inputEl",H),y.appendChild(H);let C=document.createComment("if");y.appendChild(C),m.push(w(C));let z=document.createComment("if");if(y.appendChild(z),m.push(M(z)),b!=null){let j=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(j){let g=new Proxy(n,{get(Z,D){return Z[D]},set(Z,D,J){let X=Z[D];if(X!=null&&typeof X.set==="function"){if(typeof J==="function")X.set(()=>J);else X.set(J);return!0}return!1}});if(j.mount){let Z=j.mount(g,d);if(typeof Z==="function")m.push(Z)}if(j.unmount)m.push(()=>j.unmount(g))}}return n}});var Tm=new Set(["info","success","warning","destructive","accent"]),Yi=N({scope:()=>{let i=G(!1),u=G(!0),d=e(null),m=e(null),n=e(null),b=e(null);return{hasSlottedIcon:i,visible:u,iconSlotEl:d,titleSlotEl:m,actionsSlotEl:n,closeBtn:b}},mount(i,u){let d=i,m=null,n=0,b=0,w=!1;if(!u.hasAttribute("tabindex"))u.tabIndex=0;function r(){if(m)clearTimeout(m),m=null}function M(){r();let J=d.duration();if(J>0)n=J,b=Date.now(),m=setTimeout(()=>y("timeout"),J),u.style.setProperty("--toast-duration",`${J}ms`)}function y(J="manual"){if(w)return;w=!0,r(),u.dispatchEvent(new CustomEvent("sui-dismiss",{detail:{reason:J},bubbles:!0,composed:!0})),u.remove()}async function l(J="manual"){if(w)return;if(typeof window<"u"&&window.matchMedia?.("(prefers-reduced-motion: reduce)").matches){y(J);return}r(),u.setAttribute("closing",""),d.visible.set(!1),await new Promise((Q)=>setTimeout(Q,200)),y(J)}let E=()=>{if(!m)return;let J=Date.now()-b;n=Math.max(0,n-J),r(),u.toggleAttribute("data-paused",!0)},t=()=>{if(w||d.duration()<=0)return;if(n<=0)return;u.toggleAttribute("data-paused",!1),b=Date.now(),m=setTimeout(()=>y("timeout"),n)},f=(J)=>{if(J.key==="Escape"&&d.dismissible())J.stopPropagation(),y("manual")};u.addEventListener("mouseenter",E),u.addEventListener("mouseleave",t),u.addEventListener("focusin",E),u.addEventListener("focusout",t),u.addEventListener("keydown",f),M();let L=d.hasSlottedIcon,H=(J)=>J.assignedNodes({flatten:!0}).some((X)=>{if(X.nodeType===Node.ELEMENT_NODE)return!0;return(X.textContent?.trim().length??0)>0}),C=d.iconSlotEl();if(C){let J=()=>L.set(H(C));C.addEventListener("slotchange",J),L.set(H(C))}let z=d.titleSlotEl(),j=d.actionsSlotEl();if(z){let J=()=>u.toggleAttribute("data-no-title",!H(z));z.addEventListener("slotchange",J),u.toggleAttribute("data-no-title",!H(z))}if(j){let J=()=>u.toggleAttribute("data-no-actions",!H(j));j.addEventListener("slotchange",J),u.toggleAttribute("data-no-actions",!H(j))}let g=U(()=>{let J=d.tone(),X=d.hasSlottedIcon(),W=Tm.has(J);u.toggleAttribute("data-no-icon",!W&&!X)}),Z=U(()=>{let J=d.closeBtn();if(!J)return;let X=()=>y("manual");return J.addEventListener("click",X),()=>J.removeEventListener("click",X)}),D=u;return D.dismiss=y,D.close=l,()=>{w=!0,r(),u.removeEventListener("mouseenter",E),u.removeEventListener("mouseleave",t),u.removeEventListener("focusin",E),u.removeEventListener("focusout",t),u.removeEventListener("keydown",f),g(),Z()}}});var cE=V("sui-toast",{props:{tone:P.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),duration:P.number({default:5000}),dismissible:P.bool({default:!0,reflect:!0}),"show-progress":P.bool({default:!1,reflect:!0}),closing:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.size=u.size,n.duration=u.duration,n.dismissible=u.dismissible,n.showProgress=u.showProgress,n.closing=u.closing,n.actionsSlotEl=G(null),n.closeBtn=G(null),n.iconSlotEl=G(null),n.titleSlotEl=G(null);let b=null;if(Yi&&typeof Yi.setup==="function"){if(i.scope=n,b=Yi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(f)=>I({anchor:f,test:()=>{let L=n.tone();if(L==="info")return 0;if(L==="success")return 1;if(L==="warning")return 2;if(L==="destructive")return 3;if(L==="accent")return 4;return-1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true");let z=document.createElement("circle");z.setAttribute("cx","12"),z.setAttribute("cy","12"),z.setAttribute("r","9"),C.appendChild(z);let j=document.createElement("path");j.setAttribute("d","M12 8h.01"),C.appendChild(j);let g=document.createElement("path");return g.setAttribute("d","M11 12h1v4h1"),C.appendChild(g),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let Z of H)try{Z()}catch(D){}}}},()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true");let z=document.createElement("circle");z.setAttribute("cx","12"),z.setAttribute("cy","12"),z.setAttribute("r","9"),C.appendChild(z);let j=document.createElement("path");return j.setAttribute("d","m8.5 12.5 2.5 2.5 4.5-5"),C.appendChild(j),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let g of H)try{g()}catch(Z){}}}},()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true");let z=document.createElement("path");z.setAttribute("d","M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"),C.appendChild(z);let j=document.createElement("line");j.setAttribute("x1","12"),j.setAttribute("x2","12"),j.setAttribute("y1","9"),j.setAttribute("y2","13"),C.appendChild(j);let g=document.createElement("line");return g.setAttribute("x1","12"),g.setAttribute("x2","12.01"),g.setAttribute("y1","17"),g.setAttribute("y2","17"),C.appendChild(g),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let Z of H)try{Z()}catch(D){}}}},()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true");let z=document.createElement("circle");z.setAttribute("cx","12"),z.setAttribute("cy","12"),z.setAttribute("r","9"),C.appendChild(z);let j=document.createElement("line");j.setAttribute("x1","15"),j.setAttribute("x2","9"),j.setAttribute("y1","9"),j.setAttribute("y2","15"),C.appendChild(j);let g=document.createElement("line");return g.setAttribute("x1","9"),g.setAttribute("x2","15"),g.setAttribute("y1","9"),g.setAttribute("y2","15"),C.appendChild(g),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let Z of H)try{Z()}catch(D){}}}},()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true");let z=document.createElement("path");return z.setAttribute("d","M12 2 3 14h7l-1 8 10-12h-7l1-8Z"),C.appendChild(z),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let j of H)try{j()}catch(g){}}}}]}),r=(f)=>I({anchor:f,test:()=>{if(!n.hasSlottedIcon())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createComment("match");return L.appendChild(C),H.push(w(C)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let z of H)try{z()}catch(j){}}}}]}),M=(f)=>I({anchor:f,test:()=>{if(n.dismissible())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("button");C.setAttribute("class","close"),C.setAttribute("part","close"),C.setAttribute("type","button"),C.setAttribute("aria-label","Dismiss notification"),F(n,"closeBtn",C);let z=document.createElement("svg");z.setAttribute("viewbox","0 0 24 24"),z.setAttribute("fill","none"),z.setAttribute("stroke","currentColor"),z.setAttribute("stroke-width","2"),z.setAttribute("stroke-linecap","round"),z.setAttribute("stroke-linejoin","round"),z.setAttribute("aria-hidden","true");let j=document.createElement("line");j.setAttribute("x1","18"),j.setAttribute("y1","6"),j.setAttribute("x2","6"),j.setAttribute("y2","18"),z.appendChild(j);let g=document.createElement("line");return g.setAttribute("x1","6"),g.setAttribute("y1","6"),g.setAttribute("x2","18"),g.setAttribute("y2","18"),z.appendChild(g),C.appendChild(z),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let Z of H)try{Z()}catch(D){}}}}]}),y=(f)=>I({anchor:f,test:()=>{if(n.showProgress()&&n.duration()>0)return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("span");return C.setAttribute("class","progress"),C.setAttribute("aria-hidden","true"),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let z of H)try{z()}catch(j){}}}}]}),l=(f)=>I({anchor:f,test:()=>{if(n.visible())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("div");C.setAttribute("class","toast"),En(C,Eu,{axis:"y",duration:200},"in"),En(C,Zn,{duration:180},"out"),H.push(T(C,"role",()=>n.tone()==="destructive"||n.tone()==="warning"?"alert":"status")),H.push(T(C,"aria-live",()=>n.tone()==="destructive"||n.tone()==="warning"?"assertive":"polite")),C.setAttribute("aria-atomic","true"),C.setAttribute("part","toast");let z=document.createElement("span");z.setAttribute("class","icon");let j=document.createElement("slot");j.setAttribute("name","icon"),F(n,"iconSlotEl",j),z.appendChild(j);let g=document.createComment("if");z.appendChild(g),H.push(r(g)),C.appendChild(z);let Z=document.createElement("div");Z.setAttribute("class","content");let D=document.createElement("div");D.setAttribute("class","title");let J=document.createElement("slot");J.setAttribute("name","title"),F(n,"titleSlotEl",J),D.appendChild(J),Z.appendChild(D);let X=document.createElement("div");X.setAttribute("class","body");let W=document.createElement("slot");X.appendChild(W),Z.appendChild(X);let Q=document.createElement("div");Q.setAttribute("class","actions");let Y=document.createElement("slot");Y.setAttribute("name","actions"),F(n,"actionsSlotEl",Y),Q.appendChild(Y),Z.appendChild(Q),C.appendChild(Z);let O=document.createComment("if");C.appendChild(O),H.push(M(O));let K=document.createComment("if");return C.appendChild(K),H.push(y(K)),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let B of H)try{B()}catch(A){}}}}]}),E=d.shadowRoot??d;q(E);let t=document.createComment("if");if(E.appendChild(t),m.push(l(t)),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let L=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let j=H[C];if(j!=null&&typeof j.set==="function"){if(typeof z==="function")j.set(()=>z);else j.set(z);return!0}return!1}});if(f.mount){let H=f.mount(L,d);if(typeof H==="function")m.push(H)}if(f.unmount)m.push(()=>f.unmount(L))}}return n}});var Qm={"top-left":"top","top-center":"top","top-right":"top","bottom-left":"bottom","bottom-center":"bottom","bottom-right":"bottom"},Mn="16px",Nn="position:fixed;display:flex;gap:var(--space-2,8px);pointer-events:none;z-index:var(--z-toast,9999);max-width:min(440px,calc(100vw - 32px));padding:0;margin:0;box-sizing:border-box;";function Vm(i){switch(i){case"top-right":return`${Nn}top:${Mn};right:${Mn};flex-direction:column;align-items:flex-end;`;case"top-left":return`${Nn}top:${Mn};left:${Mn};flex-direction:column;align-items:flex-start;`;case"top-center":return`${Nn}top:${Mn};left:50%;transform:translateX(-50%);flex-direction:column;align-items:center;`;case"bottom-left":return`${Nn}bottom:${Mn};left:${Mn};flex-direction:column-reverse;align-items:flex-start;`;case"bottom-center":return`${Nn}bottom:${Mn};left:50%;transform:translateX(-50%);flex-direction:column-reverse;align-items:center;`;case"bottom-right":default:return`${Nn}bottom:${Mn};right:${Mn};flex-direction:column-reverse;align-items:flex-end;`}}var Ui=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","region");if(!u.hasAttribute("aria-label"))u.setAttribute("aria-label","Notifications");if(!u.hasAttribute("aria-live"))u.setAttribute("aria-live","polite");let m=G(0),n=d.defaultSlotEl(),b=d.stackEl(),w=(L)=>{if(!b)return;if(L.parentElement!==b)b.appendChild(L)},r=()=>{if(!b)return[];return Array.from(b.children).filter((L)=>L.tagName.toLowerCase()==="sui-toast")},M=U(()=>{if(!b)return;let L=d.position();b.style.cssText=Vm(L),b.setAttribute("data-position",L)}),y=U(()=>{m();let L=d.position(),H=d.maxVisible(),C=Qm[L]??"bottom",z=r();for(let j of z)j.setAttribute("data-slide-from",C),j.style.pointerEvents="auto";if(H>0&&z.length>H){let j=z.length-H;for(let g=0;g<j;g++){let Z=z[g];if(typeof Z.close==="function")Z.close("programmatic");else Z.dismiss?.("programmatic")}}}),l=()=>{if(n){for(let L of n.assignedElements())if(L.tagName.toLowerCase()==="sui-toast")w(L)}m.set(m()+1)};if(n)n.addEventListener("slotchange",l),l();let E=new MutationObserver((L)=>{let H=!1;for(let C of L){for(let z of C.addedNodes)if(z.nodeType===Node.ELEMENT_NODE){let j=z;if(j.tagName.toLowerCase()==="sui-toast")w(j),H=!0}if(C.removedNodes.length)H=!0}if(H)m.set(m()+1)});E.observe(u,{childList:!0});for(let L of Array.from(u.children))if(L.tagName.toLowerCase()==="sui-toast")w(L);m.set(m()+1);let t=b?new MutationObserver(()=>m.set(m()+1)):null;if(b&&t)t.observe(b,{childList:!0});function f(L={}){let H=u.ownerDocument??document,C=H.createElement("sui-toast");if(L.tone)C.setAttribute("tone",L.tone);if(L.size)C.setAttribute("size",L.size);if(typeof L.duration==="number")C.setAttribute("duration",String(L.duration));if(L.dismissible===!1)C.removeAttribute("dismissible");else C.setAttribute("dismissible","");if(L.showProgress)C.setAttribute("show-progress","");if(L.title){let z=H.createElement("span");z.setAttribute("slot","title"),z.textContent=L.title,C.appendChild(z)}if(L.description)C.appendChild(H.createTextNode(L.description));if(L.actionHtml){let z=H.createElement("span");z.setAttribute("slot","actions");let j=L.actionHtml;fn(z,()=>j),C.appendChild(z)}if(b)b.appendChild(C);else u.appendChild(C);return C}return u.show=f,()=>{if(y(),M(),E.disconnect(),t?.disconnect(),n)n.removeEventListener("slotchange",l)}}});var mr=V("sui-toast-container",{props:{position:P.union(["top-left","top-center","top-right","bottom-left","bottom-center","bottom-right"],{default:"bottom-right",reflect:!0}),"max-visible":P.number({default:5,reflect:!0})},shadow:!0,styles:`
  /* Host collapses — the visible stack lives in body via <sui-portal>
     (see template). The host stays minimal so it can sit anywhere in
     the document without taking layout space. */
  :host { display: contents; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.position=u.position,n.maxVisible=u.maxVisible,n.defaultSlotEl=G(null),n.stackEl=G(null);let b=null;if(Ui&&typeof Ui.setup==="function"){if(i.scope=n,b=Ui.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("sui-portal");r.setAttribute("target","body");let M=document.createElement("div");M.setAttribute("class","sui-toast-stack"),M.setAttribute("data-sui-toast-stack",""),F(n,"stackEl",M),r.appendChild(M),w.appendChild(r);let y=document.createElement("slot");if(F(n,"defaultSlotEl",y),w.appendChild(y),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var Ni=N({scope:()=>{let i=G([]),u=G("");return{items:i,lastSlot:u}},mount(i,u){let d=()=>{let b=Array.from(u.querySelectorAll(":scope > sui-breadcrumb-item")).map((w,r)=>{let M=`i${r}`;return w.setAttribute("slot",M),{slot:M}});i.lastSlot.set(b.length>0?b[b.length-1].slot:""),i.items.set(b)};d();let m=new MutationObserver(d);return m.observe(u,{childList:!0}),()=>m.disconnect()}});var Er=V("sui-breadcrumb",{props:{separator:P.string({default:"/",reflect:!0})},shadow:!0,styles:`
  :host { display: flex; align-items: center; }
  nav { display: inline-flex; align-items: center; flex-wrap: wrap; gap: 2px; }
  .sep {
    display: inline-flex; align-items: center; justify-content: center;
    width: 12px; height: 12px;
    color: var(--muted-foreground);
    opacity: 0.7;
  }
  .sep svg { width: 12px; height: 12px; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.separator=u.separator;let b=null;if(Ni&&typeof Ni.setup==="function"){if(i.scope=n,b=Ni.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>o({anchor:l,items:()=>n.items(),key:(E,t)=>{let f=Object.create(n);return f.item=E,f.__i=t,((L)=>E.slot)(f)},render:(E,t)=>{let f=Object.create(n);return f.item=E,f.__i=t,f.__v=G(0),((L)=>{let H=(D)=>I({anchor:D,test:()=>{if(L.separator()==="/")return 0;return 1},branches:[()=>{let J=document.createDocumentFragment(),X=[],W=document.createElement("svg");W.setAttribute("viewbox","0 0 24 24"),W.setAttribute("fill","none"),W.setAttribute("stroke","currentColor"),W.setAttribute("stroke-width","2"),W.setAttribute("stroke-linecap","round"),W.setAttribute("stroke-linejoin","round");let Q=document.createElement("path");return Q.setAttribute("d","M9 6l6 6-6 6"),W.appendChild(Q),J.appendChild(W),{nodes:Array.from(J.childNodes),cleanup:()=>{for(let Y of X)try{Y()}catch(O){}}}},()=>{let J=document.createDocumentFragment(),X=[],W=document.createTextNode("");return J.appendChild(W),X.push(k(W,()=>(L.__v(),L.separator()))),{nodes:Array.from(J.childNodes),cleanup:()=>{for(let Q of X)try{Q()}catch(Y){}}}}]}),C=(D)=>I({anchor:D,test:()=>{if(L.item.slot!==L.lastSlot())return 0;return-1},branches:[()=>{let J=document.createDocumentFragment(),X=[],W=document.createElement("span");W.setAttribute("class","sep"),W.setAttribute("aria-hidden","true");let Q=document.createComment("if");return W.appendChild(Q),X.push(H(Q)),J.appendChild(W),{nodes:Array.from(J.childNodes),cleanup:()=>{for(let Y of X)try{Y()}catch(O){}}}}]}),z=document.createDocumentFragment(),j=[],g=document.createElement("slot");{let D=L.item.slot;if(D==null||D===!1||D==="")g.removeAttribute("name");else g.setAttribute("name",D===!0?"":String(D))}z.appendChild(g);let Z=document.createComment("if");return z.appendChild(Z),j.push(C(Z)),{nodes:Array.from(z.childNodes),cleanup:()=>{for(let D of j)try{D()}catch(J){}},update:(D,J)=>{L.item=D,L.__i=J;{let X=L.item.slot;if(X==null||X===!1||X==="")g.removeAttribute("name");else g.setAttribute("name",X===!0?"":String(X))}L.__v.set((X)=>X+1)}}})(f)}}),r=d.shadowRoot??d;q(r);let M=document.createElement("nav");M.setAttribute("aria-label","Breadcrumb"),M.setAttribute("part","nav");let y=document.createComment("for");if(M.appendChild(y),m.push(w(y)),r.appendChild(M),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var yr=V("sui-breadcrumb-item",{props:{href:P.string({default:"",reflect:!0}),active:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.href=u.href,n.active=u.active;let b=(M)=>I({anchor:M,test:()=>{if(n.href()&&!n.active())return 0;return 1},branches:[()=>{let y=document.createDocumentFragment(),l=[],E=document.createElement("a");l.push(T(E,"href",()=>n.href())),E.setAttribute("part","link");let t=document.createElement("slot");return E.appendChild(t),y.appendChild(E),{nodes:Array.from(y.childNodes),cleanup:()=>{for(let f of l)try{f()}catch(L){}}}},()=>{let y=document.createDocumentFragment(),l=[],E=document.createElement("span");E.setAttribute("class","label"),E.setAttribute("part","label"),l.push(T(E,"aria-current",()=>n.active()?"page":"false"));let t=document.createElement("slot");return E.appendChild(t),y.appendChild(E),{nodes:Array.from(y.childNodes),cleanup:()=>{for(let f of l)try{f()}catch(L){}}}}]}),w=d.shadowRoot??d;q(w);let r=document.createComment("if");return w.appendChild(r),m.push(b(r)),n}});var Fi=N({scope:()=>({totalPages:G(1),info:G(""),prevBtn:e(null),nextBtn:e(null)}),mount(i,u){let d=i,m=U(()=>{let M=d.page(),y=d.total(),l=d.perPage(),E=y>0?Math.ceil(y/l):1;if(d.totalPages.set(E),y>0){let t=(M-1)*l+1,f=Math.min(M*l,y);d.info.set(`Page ${M} of ${E} (${t}–${f} of ${y})`)}else d.info.set(`Page ${M} of ${E}`)}),n=d.prevBtn(),b=d.nextBtn(),w=()=>{let M=d.page();if(M>1){let y=M-1;d.page.set(y),u.dispatchEvent(new CustomEvent("sui-page",{detail:{page:y},bubbles:!0,composed:!0}))}},r=()=>{let M=d.page(),y=d.totalPages();if(M<y){let l=M+1;d.page.set(l),u.dispatchEvent(new CustomEvent("sui-page",{detail:{page:l},bubbles:!0,composed:!0}))}};return n?.addEventListener("click",w),b?.addEventListener("click",r),()=>{m(),n?.removeEventListener("click",w),b?.removeEventListener("click",r)}}});var Pr=V("sui-pagination",{props:{page:P.number({default:1,reflect:!0}),total:P.number({default:0,reflect:!0}),"per-page":P.number({default:25,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.page=u.page,n.total=u.total,n.perPage=u.perPage,n.nextBtn=G(null),n.prevBtn=G(null);let b=null;if(Fi&&typeof Fi.setup==="function"){if(i.scope=n,b=Fi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("nav");r.setAttribute("class","bar"),r.setAttribute("aria-label","Pagination");let M=document.createElement("button");F(n,"prevBtn",M),M.setAttribute("class","pg-btn"),M.setAttribute("part","prev"),m.push(T(M,"disabled",()=>n.page()<=1)),M.setAttribute("aria-label","Previous page"),m.push(T(M,"aria-disabled",()=>n.page()<=1?"true":"false"));let y=document.createTextNode("Prev");M.appendChild(y),r.appendChild(M);let l=document.createElement("span");l.setAttribute("class","info"),l.setAttribute("part","info");let E=document.createTextNode("");l.appendChild(E),m.push(k(E,()=>n.info())),r.appendChild(l);let t=document.createElement("span");t.setAttribute("class","spacer"),r.appendChild(t);let f=document.createElement("button");F(n,"nextBtn",f),f.setAttribute("class","pg-btn"),f.setAttribute("part","next"),m.push(T(f,"disabled",()=>n.page()>=n.totalPages())),f.setAttribute("aria-label","Next page"),m.push(T(f,"aria-disabled",()=>n.page()>=n.totalPages()?"true":"false"));let L=document.createTextNode("Next");if(f.appendChild(L),r.appendChild(f),w.appendChild(r),b!=null){let H=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(H){let C=new Proxy(n,{get(z,j){return z[j]},set(z,j,g){let Z=z[j];if(Z!=null&&typeof Z.set==="function"){if(typeof g==="function")Z.set(()=>g);else Z.set(g);return!0}return!1}});if(H.mount){let z=H.mount(C,d);if(typeof z==="function")m.push(z)}if(H.unmount)m.push(()=>H.unmount(C))}}return n}});function td(i,u=Date.now()){let d=Date.parse(i);if(Number.isNaN(d))return i;let m=u-d,n=Math.abs(m),b=m>=0?"ago":"from now";if(n<60000)return"just now";if(n<3600000)return`${Math.floor(n/60000)}m ${b}`;if(n<86400000)return`${Math.floor(n/3600000)}h ${b}`;let w=Math.floor(n/86400000);if(w<30)return`${w}d ${b}`;if(w<365)return`${Math.floor(w/30)}mo ${b}`;return`${Math.floor(w/365)}y ${b}`}function Ld(i){let u=Date.parse(i);return Number.isNaN(u)?i:new Date(u).toISOString()}var qm=60000,Oi=N({scope:({value:i})=>{let u=G(0),d=h(()=>{let n=i();return n?Ld(n):""}),m=h(()=>{u();let n=i();return n?td(n):""});return{tick:u,absStr:d,relStr:m}},mount(i,u){let d=U(()=>{if(i.value())u.title=i.absStr()}),m=setInterval(()=>i.tick.set(i.tick()+1),qm);return()=>{d(),clearInterval(m)}}});var Tr=V("sui-timestamp",{props:{value:P.string({default:"",reflect:!0}),format:P.union(["relative","absolute","both"],{default:"relative",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline;
    font-family: var(--font-body);
    font-size: inherit;
    color: inherit;
  }
  .abs { color: var(--muted-foreground); margin-left: 4px; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.format=u.format;let b=null;if(Oi&&typeof Oi.setup==="function"){if(i.scope=n,b=Oi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(E)=>I({anchor:E,test:()=>{if(n.format()==="absolute")return 0;return 1},branches:[()=>{let t=document.createDocumentFragment(),f=[],L=document.createTextNode("");return t.appendChild(L),f.push(k(L,()=>n.absStr())),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let H of f)try{H()}catch(C){}}}},()=>{let t=document.createDocumentFragment(),f=[],L=document.createTextNode("");return t.appendChild(L),f.push(k(L,()=>n.relStr())),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let H of f)try{H()}catch(C){}}}}]}),r=(E)=>I({anchor:E,test:()=>{if(n.format()==="both")return 0;return-1},branches:[()=>{let t=document.createDocumentFragment(),f=[],L=document.createElement("span");L.setAttribute("class","abs"),L.setAttribute("part","abs");let H=document.createTextNode("(");L.appendChild(H);let C=document.createTextNode("");L.appendChild(C),f.push(k(C,()=>n.absStr()));let z=document.createTextNode(")");return L.appendChild(z),t.appendChild(L),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let j of f)try{j()}catch(g){}}}}]}),M=(E)=>I({anchor:E,test:()=>{if(n.value())return 0;return-1},branches:[()=>{let t=document.createDocumentFragment(),f=[],L=document.createElement("time");f.push(T(L,"datetime",()=>n.value())),L.setAttribute("part","time");let H=document.createComment("if");L.appendChild(H),f.push(w(H)),t.appendChild(L);let C=document.createComment("if");return t.appendChild(C),f.push(r(C)),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let z of f)try{z()}catch(j){}}}}]}),y=d.shadowRoot??d;q(y);let l=document.createComment("if");if(y.appendChild(l),m.push(M(l)),b!=null){let E=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(E){let t=new Proxy(n,{get(f,L){return f[L]},set(f,L,H){let C=f[L];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(E.mount){let f=E.mount(t,d);if(typeof f==="function")m.push(f)}if(E.unmount)m.push(()=>E.unmount(t))}}return n}});var qr=V("sui-metachip",{props:{label:P.string({default:"",reflect:!0}),value:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.label=u.label,n.value=u.value;let b=d.shadowRoot??d;q(b);let w=document.createElement("span");w.setAttribute("class","k"),w.setAttribute("part","label");let r=document.createElement("slot");r.setAttribute("name","label");let M=document.createTextNode("");r.appendChild(M),m.push(k(M,()=>n.label())),w.appendChild(r),b.appendChild(w);let y=document.createElement("span");y.setAttribute("class","v"),y.setAttribute("part","value");let l=document.createElement("slot"),E=document.createTextNode("");return l.appendChild(E),m.push(k(E,()=>n.value())),y.appendChild(l),b.appendChild(y),n}});var Ym=0,ld=typeof CSS<"u"&&typeof CSS.supports==="function"&&CSS.supports("anchor-name","--x"),Ai=N({mount(i,u){let d=i,m=++Ym,n=`--sui-popover-${m}`,b=`sui-popover-panel-${m}`;u.style.setProperty("--sui-popover-anchor",n);let w=d.triggerSlot(),r=d.panelEl();if(!r)return;r.id=b,tu(d.triggerSlot,(t)=>{for(let f of t)f.setAttribute("popovertarget",b),f.setAttribute("aria-haspopup","dialog"),f.setAttribute("aria-controls",b),f.setAttribute("aria-expanded",d.open()?"true":"false")});function M(t){let L=t.newState==="open",H=u.hasAttribute("open");if(L&&!H)u.setAttribute("open","");else if(!L&&H)u.removeAttribute("open");let z=w?.assignedElements({flatten:!0})??[];for(let j of z)j.setAttribute("aria-expanded",L?"true":"false");if(u.emit("sui-toggle",{open:L}),L&&!ld)l()}_(r,"toggle",M),u.expose({show:()=>r.showPopover?.(),hide:()=>r.hidePopover?.(),toggle:()=>typeof r.togglePopover==="function"?void r.togglePopover():r.matches?.(":popover-open")?r.hidePopover?.():r.showPopover?.()});let y=U(()=>{let t=d.open(),f=r.matches?.(":popover-open")??!1;if(t&&!f)r.showPopover?.();else if(!t&&f)r.hidePopover?.()});function l(){let L=(w?.assignedElements({flatten:!0})??[])[0];if(!L||!r)return;let H=L.getBoundingClientRect(),C=d.placement();r.style.top="",r.style.left="",r.style.right="",r.style.bottom="";let z=6;switch(C){case"top":r.style.bottom=`${window.innerHeight-H.top+z}px`,r.style.left=`${H.left}px`;break;case"top-end":r.style.bottom=`${window.innerHeight-H.top+z}px`,r.style.right=`${window.innerWidth-H.right}px`;break;case"bottom-end":r.style.top=`${H.bottom+z}px`,r.style.right=`${window.innerWidth-H.right}px`;break;case"left":r.style.right=`${window.innerWidth-H.left+z}px`,r.style.top=`${H.top}px`;break;case"left-end":r.style.right=`${window.innerWidth-H.left+z}px`,r.style.bottom=`${window.innerHeight-H.bottom}px`;break;case"right":r.style.left=`${H.right+z}px`,r.style.top=`${H.top}px`;break;case"right-end":r.style.left=`${H.right+z}px`,r.style.bottom=`${window.innerHeight-H.bottom}px`;break;case"bottom":default:r.style.top=`${H.bottom+z}px`,r.style.left=`${H.left}px`}}function E(){if(!d.open())return;l()}if(!ld)_(window,"resize",E),_(window,"scroll",E,{capture:!0,passive:!0});return()=>{y()}}});var Ar=V("sui-popover",{props:{open:P.bool({default:!1,reflect:!0}),placement:P.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.placement=u.placement,n.panelEl=G(null),n.triggerSlot=G(null);let b=null;if(Ai&&typeof Ai.setup==="function"){if(i.scope=n,b=Ai.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(f)=>I({anchor:f,test:()=>{if(n.open())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("div");C.setAttribute("class","surface"),En(C,ru,{duration:120},"both");let z=document.createElement("slot");return C.appendChild(z),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let j of H)try{j()}catch(g){}}}}]}),r=(f)=>I({anchor:f,test:()=>{if(n.open())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("sui-portal");C.setAttribute("target","body");let z=document.createElement("div");return z.setAttribute("class","sui-popover-presence"),z.setAttribute("aria-hidden","true"),z.setAttribute("data-sui-popover-open",""),C.appendChild(z),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let j of H)try{j()}catch(g){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createElement("slot");y.setAttribute("name","trigger"),F(n,"triggerSlot",y),M.appendChild(y);let l=document.createElement("div");l.setAttribute("class","panel"),l.setAttribute("part","panel"),l.setAttribute("popover","auto"),F(n,"panelEl",l),l.setAttribute("role","dialog"),l.setAttribute("aria-modal","false");let E=document.createComment("if");l.appendChild(E),m.push(w(E)),M.appendChild(l);let t=document.createComment("if");if(M.appendChild(t),m.push(r(t)),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let L=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let j=H[C];if(j!=null&&typeof j.set==="function"){if(typeof z==="function")j.set(()=>z);else j.set(z);return!0}return!1}});if(f.mount){let H=f.mount(L,d);if(typeof H==="function")m.push(H)}if(f.unmount)m.push(()=>f.unmount(L))}}return n}});var Zu=[{id:"ship-light",label:"SHIP Light",dark:!1,swatches:["oklch(0.99 0.001 106.423)","oklch(1 0 0)","oklch(0.67 0.16 58)","oklch(0.61 0.23 303)"]},{id:"ship-dark",label:"SHIP Dark",dark:!0,swatches:["oklch(0.18 0.01 250)","oklch(0.18 0.01 50)","oklch(0.77 0.16 70)","oklch(0.3 0.05 300)"]},{id:"paper",label:"Paper",dark:!1,swatches:["oklch(0.97 0.014 80)","oklch(0.99 0.010 80)","oklch(0.55 0.14 35)","oklch(0.48 0.09 55)"]},{id:"halo",label:"Halo (Soft)",dark:!1,swatches:["oklch(0.92 0.008 230)","oklch(0.92 0.008 230)","oklch(0.62 0.14 50)","oklch(0.55 0.05 230)"]},{id:"console",label:"Console",dark:!0,swatches:["oklch(0.13 0.008 220)","oklch(0.16 0.010 220)","oklch(0.78 0.20 145)","oklch(0.78 0.16 195)"]},{id:"contrast",label:"High Contrast",dark:!0,swatches:["oklch(0.09 0 0)","oklch(0.15 0 0)","oklch(0.87 0.18 85)","oklch(0.99 0 0)"]},{id:"glass",label:"Glass",dark:!1,swatches:["oklch(0.95 0.04 230)","oklch(0.99 0.005 240)","oklch(0.62 0.16 50)","oklch(0.62 0.14 280)"]}],Ed=Object.fromEntries(Zu.map((i)=>[i.id,i]));var Ki=(i)=>typeof i==="string"&&(i in Ed),Um=(i)=>Ed[i]?.dark??!1;function Ju(i,u=document.documentElement){let d=Ki(i)?i:"ship-light",m=Um(d);if(u.dataset.theme=d,u.classList.toggle("dark",m),typeof document<"u"&&typeof CustomEvent<"u")document.dispatchEvent(new CustomEvent("ship:theme:change",{detail:{id:d,dark:m}}));return d}function Xu(){if(typeof localStorage>"u")return"ship-light";try{let i=localStorage.getItem("ship-site-theme");return Ki(i)?i:"ship-light"}catch{return"ship-light"}}function rd(i){if(typeof localStorage>"u")return;try{localStorage.setItem("ship-site-theme",i)}catch{}try{fetch("/api/settings",{method:"PUT",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:"shell.theme",value:i})}).catch(()=>{})}catch{}}var Nm=[{label:"Default",ids:["ship-light","ship-dark"]},{label:"Texture",ids:["paper","halo"]},{label:"Effects",ids:["glass"]},{label:"Special",ids:["console","contrast"]}],Bi=new Map(Zu.map((i)=>[i.id,i]));function Fm(i){let u=i.shadowRoot;if(!u)return;if(u.querySelector("link[data-sui-theme-preview-styles]"))return;let d=document.createElement("link");d.rel="stylesheet",d.href="/api/assets/view-tokens.css",d.setAttribute("data-sui-theme-preview-styles",""),u.appendChild(d)}var Ii=N({scope:(i)=>{let u=i,d=G(Xu()),m=G(!1),n=G(Nm.map((t)=>({label:t.label,themes:t.ids.map((f)=>Bi.get(f)).filter((f)=>!!f)}))),b=h(()=>Bi.get(d())?.swatches[2]??"var(--primary, #3b82f6)"),w=h(()=>Bi.get(d())?.label??"Theme"),r={host:null,panel:null};function M(){let t=r.panel;if(!t)return;if(m())t.hidePopover?.();else t.showPopover?.()}function y(t){let f=Ju(t);if(u.persist())rd(f);d.set(f);let L=r.host;if(L)L.dispatchEvent(new CustomEvent("sui-theme-change",{detail:{id:f,dark:Bi.get(f)?.dark??!1},bubbles:!0,composed:!0}));r.panel?.hidePopover?.()}let l=e(null),E=e(null);return{activeTheme:d,panelOpen:m,groups:n,dotColor:b,triggerLabel:w,togglePanel:M,selectTheme:y,triggerBtn:l,panelEl:E,_internals:r}},mount(i,u){let d=i;d._internals.host=u,Fm(u),Ju(Xu());let m=U(()=>{let y=d.panelEl();if(d._internals.panel=y,!y)return;let l=(E)=>{let f=E.newState==="open";if(d.panelOpen()!==f)d.panelOpen.set(f);if(f)n()};return y.addEventListener("toggle",l),()=>y.removeEventListener("toggle",l)});function n(){let y=d.panelEl(),l=d.triggerBtn();if(!y||!l)return;let E=l.getBoundingClientRect();y.style.top=`${E.bottom+4}px`,y.style.left=`${Math.max(8,E.right-y.offsetWidth)}px`}function b(y){let l=y.detail?.id;if(!Ki(l))return;if(d.activeTheme()!==l)d.activeTheme.set(l)}document.addEventListener("ship:theme:change",b);function w(y){let l=d.panelEl();if(!l)return;let E=Array.from(l.querySelectorAll(".card"));if(!E.length)return;let t=l.getRootNode(),f=E.findIndex((L)=>L===t.activeElement);switch(y.key){case"ArrowDown":y.preventDefault(),E[(f+1+E.length)%E.length]?.focus();break;case"ArrowUp":y.preventDefault(),E[(f-1+E.length)%E.length]?.focus();break;case"Home":y.preventDefault(),E[0]?.focus();break;case"End":y.preventDefault(),E[E.length-1]?.focus();break;case"Escape":y.preventDefault(),l.hidePopover?.();break}}let r=U(()=>{let y=d.panelEl();if(!y)return;return y.addEventListener("keydown",w),()=>y.removeEventListener("keydown",w)}),M=U(()=>{if(!d.panelOpen())return;requestAnimationFrame(()=>{let y=d.panelEl();if(!y)return;let l=Array.from(y.querySelectorAll(".card"));(l.find((t)=>t.getAttribute("aria-selected")==="true")??l[0])?.focus()})});return()=>{document.removeEventListener("ship:theme:change",b),m(),r(),M()}}});var vr=V("sui-theme-picker",{props:{placement:P.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom-end",reflect:!0}),"label-style":P.union(["text","compact"],{default:"text",reflect:!0}),persist:P.bool({default:!0,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.placement=u.placement,n.labelStyle=u.labelStyle,n.persist=u.persist,n.panelEl=G(null),n.triggerBtn=G(null);let b=null;if(Ii&&typeof Ii.setup==="function"){if(i.scope=n,b=Ii.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(z)=>o({anchor:z,items:()=>n.groups(),key:(j,g)=>{let Z=Object.create(n);return Z.g=j,Z.__i=g,((D)=>j.label)(Z)},render:(j,g)=>{let Z=Object.create(n);return Z.g=j,Z.__i=g,Z.__v=G(0),((D)=>{let J=(A)=>o({anchor:A,items:()=>D.g.themes,key:(a,S)=>{let v=Object.create(D);return v.t=a,v.__i=S,((R)=>a.id)(v)},render:(a,S)=>{let v=Object.create(D);return v.t=a,v.__i=S,v.__v=G(0),((R)=>{let x=document.createDocumentFragment(),s=[],$=document.createElement("button");$.setAttribute("class","card"),$.setAttribute("role","option"),$.setAttribute("type","button");{let c=R.t.id;if(c==null||c===!1||c==="")$.removeAttribute("data-theme-card");else $.setAttribute("data-theme-card",c===!0?"":String(c))}{let c=R.t.label;if(c==null||c===!1||c==="")$.removeAttribute("aria-label");else $.setAttribute("aria-label",c===!0?"":String(c))}s.push(T($,"aria-selected",()=>(R.__v(),R.activeTheme()===R.t.id?"true":"false"))),s.push(T($,"tabindex",()=>(R.__v(),R.activeTheme()===R.t.id?"0":"-1"))),s.push(p($,"click",function(){let c=()=>R.selectTheme(R.t.id);return typeof c==="function"?c.bind(void 0):()=>{}}(),{modifiers:[]}));let nn=document.createElement("span");nn.setAttribute("class","tc-bg");{let c=R.t.id;if(c==null||c===!1||c==="")nn.removeAttribute("data-theme-preview");else nn.setAttribute("data-theme-preview",c===!0?"":String(c))}nn.setAttribute("aria-hidden","true");let tn=document.createElement("span");tn.setAttribute("class","tc-overlay"),nn.appendChild(tn);let dn=document.createElement("span");dn.setAttribute("class","tc-card");let Ln=document.createElement("span");Ln.setAttribute("class","tc-card-head");let jn=document.createElement("span");jn.setAttribute("class","tc-eyebrow");let v0=document.createTextNode("THEME");jn.appendChild(v0),Ln.appendChild(jn);let An=document.createElement("span");An.setAttribute("class","tc-heading");let on=document.createTextNode("");An.appendChild(on),on.textContent=R.t.label,Ln.appendChild(An),dn.appendChild(Ln);let sn=document.createElement("span");sn.setAttribute("class","tc-typo");let x0=document.createElement("span");x0.setAttribute("class","tc-body");let Nd=document.createTextNode("The quick brown fox");x0.appendChild(Nd),sn.appendChild(x0);let h0=document.createElement("span");h0.setAttribute("class","tc-mono");let $0=document.createTextNode("");h0.appendChild($0),$0.textContent="{ build: ok }",sn.appendChild(h0),dn.appendChild(sn);let Xn=document.createElement("span");Xn.setAttribute("class","tc-controls");let o0=document.createElement("span");o0.setAttribute("class","tc-cta");let Fd=document.createTextNode("Action");o0.appendChild(Fd),Xn.appendChild(o0);let s0=document.createElement("span");s0.setAttribute("class","tc-ghost");let Od=document.createTextNode("Cancel");s0.appendChild(Od),Xn.appendChild(s0);let e0=document.createElement("span");e0.setAttribute("class","tc-chip tc-chip-accent");let Ad=document.createTextNode("New");e0.appendChild(Ad),Xn.appendChild(e0);let p0=document.createElement("span");p0.setAttribute("class","tc-chip tc-chip-muted");let Kd=document.createTextNode("Idle");p0.appendChild(Kd),Xn.appendChild(p0),dn.appendChild(Xn),nn.appendChild(dn),$.appendChild(nn);let en=document.createElement("span");en.setAttribute("class","row");let c0=document.createElement("span");c0.setAttribute("class","name");let _0=document.createTextNode("");c0.appendChild(_0),_0.textContent=R.t.label,en.appendChild(c0);let nu=document.createElement("span");return nu.setAttribute("class","radio"),nu.setAttribute("aria-hidden","true"),en.appendChild(nu),$.appendChild(en),x.appendChild($),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let c of s)try{c()}catch(Vu){}},update:(c,Vu)=>{R.t=c,R.__i=Vu;{let un=R.t.id;if(un==null||un===!1||un==="")$.removeAttribute("data-theme-card");else $.setAttribute("data-theme-card",un===!0?"":String(un))}{let un=R.t.label;if(un==null||un===!1||un==="")$.removeAttribute("aria-label");else $.setAttribute("aria-label",un===!0?"":String(un))}{let un=R.t.id;if(un==null||un===!1||un==="")nn.removeAttribute("data-theme-preview");else nn.setAttribute("data-theme-preview",un===!0?"":String(un))}on.textContent=R.t.label,$0.textContent="{ build: ok }",_0.textContent=R.t.label,R.__v.set((un)=>un+1)}}})(v)}}),X=document.createDocumentFragment(),W=[],Q=document.createElement("section");Q.setAttribute("class","group");{let A=D.g.label;if(A==null||A===!1||A==="")Q.removeAttribute("aria-label");else Q.setAttribute("aria-label",A===!0?"":String(A))}let Y=document.createElement("div");Y.setAttribute("class","group-h");let O=document.createTextNode("");Y.appendChild(O),O.textContent=D.g.label,Q.appendChild(Y);let K=document.createElement("div");K.setAttribute("class","group-rows");let B=document.createComment("for");return K.appendChild(B),W.push(J(B)),Q.appendChild(K),X.appendChild(Q),{nodes:Array.from(X.childNodes),cleanup:()=>{for(let A of W)try{A()}catch(a){}},update:(A,a)=>{D.g=A,D.__i=a;{let S=D.g.label;if(S==null||S===!1||S==="")Q.removeAttribute("aria-label");else Q.setAttribute("aria-label",S===!0?"":String(S))}O.textContent=D.g.label,D.__v.set((S)=>S+1)}}})(Z)}}),r=d.shadowRoot??d;q(r);let M=document.createElement("button");M.setAttribute("class","trigger"),M.setAttribute("part","trigger"),M.setAttribute("type","button"),F(n,"triggerBtn",M),M.setAttribute("aria-label","Choose theme"),M.setAttribute("aria-haspopup","dialog"),m.push(T(M,"aria-expanded",()=>n.panelOpen()?"true":"false")),m.push(p(M,"click",function(){let z=()=>n.togglePanel();return typeof z==="function"?z.bind(void 0):()=>{}}(),{modifiers:[]}));let y=document.createElement("span");y.setAttribute("class","dot"),y.setAttribute("part","dot"),m.push(U(()=>{let z=n.dotColor();if(z==null)y.style.removeProperty("background");else y.style.setProperty("background",String(z))})),y.setAttribute("aria-hidden","true"),M.appendChild(y);let l=document.createElement("span");l.setAttribute("class","label"),l.setAttribute("part","label");let E=document.createTextNode("");l.appendChild(E),m.push(k(E,()=>n.triggerLabel())),M.appendChild(l);let t=document.createElement("svg");t.setAttribute("class","caret"),t.setAttribute("viewbox","0 0 10 6"),t.setAttribute("aria-hidden","true");let f=document.createElement("path");f.setAttribute("d","M1 1l4 4 4-4"),f.setAttribute("fill","none"),f.setAttribute("stroke","currentColor"),f.setAttribute("stroke-width","1.5"),f.setAttribute("stroke-linecap","round"),f.setAttribute("stroke-linejoin","round"),t.appendChild(f),M.appendChild(t),r.appendChild(M);let L=document.createElement("div");L.setAttribute("class","panel"),L.setAttribute("part","panel"),L.setAttribute("popover","auto"),F(n,"panelEl",L);let H=document.createElement("div");H.setAttribute("class","grid"),H.setAttribute("part","grid"),H.setAttribute("role","listbox"),H.setAttribute("aria-label","Theme");let C=document.createComment("for");if(H.appendChild(C),m.push(w(C)),L.appendChild(H),r.appendChild(L),b!=null){let z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(z){let j=new Proxy(n,{get(g,Z){return g[Z]},set(g,Z,D){let J=g[Z];if(J!=null&&typeof J.set==="function"){if(typeof D==="function")J.set(()=>D);else J.set(D);return!0}return!1}});if(z.mount){let g=z.mount(j,d);if(typeof g==="function")m.push(g)}if(z.unmount)m.push(()=>z.unmount(j))}}return n}});var Om=`
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
`,Md=!1;function Am(){if(Md)return;Md=!0;let i=document.createElement("style");i.setAttribute("data-sui-rail",""),i.textContent=Om,document.head.appendChild(i)}function Ri(i,u,d){i.dispatchEvent(new CustomEvent(u,{detail:d,bubbles:!0,composed:!0}))}function yd(i){Am();let{host:u,open:d,projects:m,activeProjectId:n,popoverEl:b,badgeEl:w}=i,r=()=>d.set(!1),M=async()=>{try{let j=await fetch("/api/v1/projects",{credentials:"same-origin"});if(!j.ok)return;let g=await j.json();m.set(g.projects??[]),n.set(g.active??null)}catch{}},y=async(j)=>{try{await fetch("/api/v1/projects/switch",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({id:j})}),n.set(j),r(),Ri(u,"ship:project:change",{id:j})}catch{}M()},l=async()=>{let j=window.prompt("Path to project directory:");if(!j)return;try{let g=await fetch("/api/v1/projects",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({path:j})});if(!g.ok){let Z=await g.text().catch(()=>"");Ri(u,"sui:notify",{tone:"warning",title:`Add failed: ${Z||g.status}`});return}await M(),Ri(u,"sui:notify",{tone:"success",title:"Project added."})}catch(g){Ri(u,"sui:notify",{tone:"destructive",title:String(g.message??g)})}},E=()=>{let j=b(),g=w();if(!j||!g)return;let Z=g.getBoundingClientRect(),D=j.offsetWidth||220,J=8,X=Math.max(8,Math.min(Z.left,window.innerWidth-D-8)),W=Math.max(8,window.innerHeight-Z.top+J);j.style.left=`${X}px`,j.style.bottom=`${W}px`},t=(j)=>{let g=j.target;if(!g)return;if(g.closest("[data-rail-add]")){l();return}let Z=g.closest("[data-project-id]");if(Z?.dataset.projectId)y(Z.dataset.projectId)},f=()=>d.set(!d()),L=(j)=>{if(!d())return;let g=typeof j.composedPath==="function"?j.composedPath():[],Z=b(),D=w();for(let J of g)if(J===Z||J===D)return;r()},H=()=>{if(d())E()},C=U(()=>{let j=w();if(!j)return;return j.addEventListener("click",f),()=>j.removeEventListener("click",f)}),z=U(()=>{let j=b();if(!j)return;return requestAnimationFrame(E),j.addEventListener("click",t),()=>j.removeEventListener("click",t)});return document.addEventListener("click",L,!0),window.addEventListener("resize",H),window.addEventListener("scroll",H,{capture:!0,passive:!0}),{loadProjects:M,dispose(){C(),z(),document.removeEventListener("click",L,!0),window.removeEventListener("resize",H),window.removeEventListener("scroll",H,!0)}}}var Km=15000;function wd(i){return i.split("/").filter(Boolean)[0]??"dashboard"}function Hd(i,u,d){i.dispatchEvent(new CustomEvent(u,{detail:d,bubbles:!0,composed:!0}))}var ki=N({scope:()=>{let i=G(!1),u=G([]),d=G(null),m=G(typeof location<"u"?wd(location.pathname):""),n=G(0);return{projectOpen:i,projects:u,activeProjectId:d,activeView:m,sessionCount:n,projectInitial:()=>{let w=u(),r=w.find((M)=>M.id===d())??w[0];if(!r)return"";return(r.label??r.name).charAt(0).toUpperCase()},popoverEl:e(),projectBadgeEl:e(),launcherBtnEl:e(),createBtnEl:e(),missionEl:e(),navSlot:e()}},mount(i,u){let d=yd({host:u,open:i.projectOpen,projects:i.projects,activeProjectId:i.activeProjectId,popoverEl:i.popoverEl,badgeEl:i.projectBadgeEl});d.loadProjects();let m=()=>i.projectOpen.set(!1),n=(H)=>{Hd(u,"ship:launcher:open",{anchor:H.currentTarget}),m()},b=()=>{Hd(u,"ship:create:open"),m()},w=U(()=>{let H=i.launcherBtnEl();if(!H)return;return H.addEventListener("click",n),()=>H.removeEventListener("click",n)}),r=U(()=>{let H=i.createBtnEl();if(!H)return;return H.addEventListener("click",b),()=>H.removeEventListener("click",b)}),M=async()=>{try{let H=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(!H.ok)return;let z=(await H.json()).sessions.filter((j)=>j.status!=="ended").length;i.sessionCount.set(z)}catch{}};M();let y=window.setInterval(()=>void M(),Km),l=()=>void M();window.addEventListener("ship:session:changed",l);let E=U(()=>{let H=i.missionEl();if(!H)return;let C=i.sessionCount();H.setAttribute("badge",C>0?String(C):"")}),t=()=>{i.activeView.set(wd(location.pathname))},f=()=>t();document.addEventListener("sui:navigate",f),window.addEventListener("popstate",f);let L=U(()=>{let H=i.activeView(),C=u.querySelectorAll("sui-rail-item[data-view]");for(let j of C)if(j.dataset.view===H)j.setAttribute("active","");else j.removeAttribute("active");let z=i.missionEl();if(z)if(H==="mesh")z.setAttribute("active","");else z.removeAttribute("active")});return()=>{w(),r(),E(),L(),d.dispose(),document.removeEventListener("sui:navigate",f),window.removeEventListener("popstate",f),window.removeEventListener("ship:session:changed",l),window.clearInterval(y)}}});var _r=V("sui-rail",{props:{collapsed:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    width: var(--ship-rail-w-collapsed, 48px);
    background: var(--sidebar, var(--muted));
    border-right: 1px solid var(--border);
    overflow: hidden;
    user-select: none;
    -webkit-user-select: none;
    --rail-label-display: none;
    --rail-item-justify: center;
  }
  :host([collapsed]),
  .rail-footer { padding: 4px; }

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
    flex-direction: column;
    align-items: center;
    gap: 4px;
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

  @media (prefers-reduced-motion: reduce) {
    :host { transition: none; }
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.collapsed=u.collapsed,n.createBtnEl=G(null),n.launcherBtnEl=G(null),n.missionEl=G(null),n.navSlot=G(null),n.popoverEl=G(null),n.projectBadgeEl=G(null);let b=null;if(ki&&typeof ki.setup==="function"){if(i.scope=n,b=ki.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(X)=>I({anchor:X,test:()=>{if(n.projects().length===0)return 0;return-1},branches:[()=>{let W=document.createDocumentFragment(),Q=[],Y=document.createElement("div");Y.setAttribute("class","sui-rail-project-popover-empty");let O=document.createTextNode("No projects tracked.");return Y.appendChild(O),W.appendChild(Y),{nodes:Array.from(W.childNodes),cleanup:()=>{for(let K of Q)try{K()}catch(B){}}}}]}),r=(X)=>o({anchor:X,items:()=>n.projects(),key:(W,Q)=>{let Y=Object.create(n);return Y.p=W,Y.__i=Q,((O)=>W.id)(Y)},render:(W,Q)=>{let Y=Object.create(n);return Y.p=W,Y.__i=Q,Y.__v=G(0),((O)=>{let K=document.createDocumentFragment(),B=[],A=document.createElement("div");A.setAttribute("class","sui-rail-project-popover-row"),B.push(zn(A,"active",()=>(O.__v(),O.p.id===O.activeProjectId())));{let x=O.p.id;if(x==null||x===!1||x==="")A.removeAttribute("data-project-id");else A.setAttribute("data-project-id",x===!0?"":String(x))}let a=document.createElement("span");a.setAttribute("class","sui-rail-project-popover-row-name");let S=document.createTextNode("");a.appendChild(S),S.textContent=O.p.label||O.p.name,A.appendChild(a);let v=document.createElement("span");v.setAttribute("class","sui-rail-project-popover-row-path");let R=document.createTextNode("");return v.appendChild(R),R.textContent=O.p.path,A.appendChild(v),K.appendChild(A),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let x of B)try{x()}catch(s){}},update:(x,s)=>{O.p=x,O.__i=s;{let $=O.p.id;if($==null||$===!1||$==="")A.removeAttribute("data-project-id");else A.setAttribute("data-project-id",$===!0?"":String($))}S.textContent=O.p.label||O.p.name,R.textContent=O.p.path,O.__v.set(($)=>$+1)}}})(Y)}}),M=(X)=>I({anchor:X,test:()=>{if(n.projectOpen())return 0;return-1},branches:[()=>{let W=document.createDocumentFragment(),Q=[],Y=document.createElement("div");Y.setAttribute("class","sui-rail-project-popover"),Y.setAttribute("part","project-popover"),Y.setAttribute("role","dialog"),Y.setAttribute("aria-label","Switch project"),F(n,"popoverEl",Y),En(Y,Zn,{duration:150},"both");let O=document.createElement("div");O.setAttribute("class","sui-rail-project-popover-title");let K=document.createTextNode("Projects");O.appendChild(K),Y.appendChild(O);let B=document.createComment("if");Y.appendChild(B),Q.push(w(B));let A=document.createComment("for");Y.appendChild(A),Q.push(r(A));let a=document.createElement("div");a.setAttribute("class","sui-rail-project-popover-sep"),Y.appendChild(a);let S=document.createElement("div");S.setAttribute("class","sui-rail-project-popover-row add"),S.setAttribute("data-rail-add","");let v=document.createElement("span"),R=document.createTextNode("+ Add project");return v.appendChild(R),S.appendChild(v),Y.appendChild(S),W.appendChild(Y),{nodes:Array.from(W.childNodes),cleanup:()=>{for(let x of Q)try{x()}catch(s){}}}}]}),y=d.shadowRoot??d;q(y);let l=document.createElement("nav");l.setAttribute("class","rail-nav"),l.setAttribute("part","nav");let E=document.createElement("slot");F(n,"navSlot",E),l.appendChild(E);let t=document.createElement("sui-rail-item");F(n,"missionEl",t),t.setAttribute("part","mission-control"),t.setAttribute("data-view","mesh"),t.setAttribute("icon","message-square-plus"),t.setAttribute("label","Mission Control"),t.setAttribute("href","/mesh"),l.appendChild(t),y.appendChild(l);let f=document.createElement("div");f.setAttribute("class","rail-footer"),f.setAttribute("part","footer");let L=document.createElement("button");L.setAttribute("type","button"),L.setAttribute("class","project-badge"),L.setAttribute("part","project-badge"),L.setAttribute("aria-label","Switch project"),L.setAttribute("aria-haspopup","dialog"),L.setAttribute("title","Switch project"),F(n,"projectBadgeEl",L);let H=document.createTextNode("");L.appendChild(H),m.push(k(H,()=>n.projectInitial())),f.appendChild(L);let C=document.createElement("div");C.setAttribute("class","footer-actions");let z=document.createElement("button");z.setAttribute("type","button"),z.setAttribute("class","footer-btn"),z.setAttribute("part","launcher-btn"),z.setAttribute("title","Open launcher"),z.setAttribute("aria-label","Open launcher"),F(n,"launcherBtnEl",z);let j=document.createElement("sui-icon");j.setAttribute("name","folder"),j.setAttribute("size","sm"),z.appendChild(j),C.appendChild(z);let g=document.createElement("button");g.setAttribute("type","button"),g.setAttribute("class","footer-btn"),g.setAttribute("part","create-btn"),g.setAttribute("title","Create new"),g.setAttribute("aria-label","Create new"),F(n,"createBtnEl",g);let Z=document.createElement("sui-icon");Z.setAttribute("name","plus"),Z.setAttribute("size","sm"),g.appendChild(Z),C.appendChild(g),f.appendChild(C),y.appendChild(f);let D=document.createElement("sui-portal");D.setAttribute("target","body");let J=document.createComment("if");if(D.appendChild(J),m.push(M(J)),y.appendChild(D),b!=null){let X=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(X){let W=new Proxy(n,{get(Q,Y){return Q[Y]},set(Q,Y,O){let K=Q[Y];if(K!=null&&typeof K.set==="function"){if(typeof O==="function")K.set(()=>O);else K.set(O);return!0}return!1}});if(X.mount){let Q=X.mount(W,d);if(typeof Q==="function")m.push(Q)}if(X.unmount)m.push(()=>X.unmount(W))}}return n}});var u5=V("sui-rail-item",{props:{href:P.string({default:""}),label:P.string({default:""}),icon:P.string({default:""}),active:P.bool({default:!1,reflect:!0}),badge:P.string({default:""}),disabled:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.href=u.href,n.label=u.label,n.icon=u.icon,n.active=u.active,n.badge=u.badge,n.disabled=u.disabled,n.badgeEl=G(null),n.itemEl=G(null);let b=d.shadowRoot??d;q(b);let w=document.createElement("a");w.setAttribute("class","item"),w.setAttribute("part","item"),F(n,"itemEl",w),m.push(T(w,"href",()=>n.href()||"#")),m.push(T(w,"aria-current",()=>n.active()?"page":null)),m.push(T(w,"aria-disabled",()=>n.disabled()?"true":null)),m.push(T(w,"title",()=>n.label()));let r=document.createElement("span");r.setAttribute("class","icon-wrap"),r.setAttribute("part","icon-wrap");let M=document.createElement("sui-icon");m.push(T(M,"name",()=>n.icon())),M.setAttribute("size","sm"),r.appendChild(M),w.appendChild(r);let y=document.createElement("span");y.setAttribute("class","label"),y.setAttribute("part","label");let l=document.createTextNode("");y.appendChild(l),m.push(k(l,()=>n.label())),w.appendChild(y);let E=document.createElement("span");E.setAttribute("class","badge"),E.setAttribute("part","badge"),F(n,"badgeEl",E);let t=document.createTextNode("");return E.appendChild(t),m.push(k(t,()=>n.badge())),w.appendChild(E),b.appendChild(w),n}});var b5=V("sui-rail-section",{props:{label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.label=u.label;let b=d.shadowRoot??d;q(b);let w=document.createElement("span");w.setAttribute("class","heading"),w.setAttribute("part","heading");let r=document.createTextNode("");w.appendChild(r),m.push(k(r,()=>n.label())),b.appendChild(w);let M=document.createElement("slot");return b.appendChild(M),n}});var L5=V("sui-status-bar",{props:{branch:P.string({default:""}),theme:P.string({default:""}),online:P.bool({default:!1,reflect:!0}),agents:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.branch=u.branch,n.theme=u.theme,n.online=u.online,n.agents=u.agents;let b=d.shadowRoot??d;q(b);let w=document.createElement("span");w.setAttribute("class","dot"),w.setAttribute("part","dot"),w.setAttribute("aria-hidden","true"),b.appendChild(w);let r=document.createElement("span");r.setAttribute("class","text"),r.setAttribute("part","branch");let M=document.createTextNode("");r.appendChild(M),m.push(k(M,()=>n.branch())),b.appendChild(r);let y=document.createElement("span");y.setAttribute("class","text"),y.setAttribute("part","agents");let l=document.createTextNode("");y.appendChild(l),m.push(k(l,()=>n.agents())),b.appendChild(y);let E=document.createElement("span");E.setAttribute("class","slot-wrap");let t=document.createElement("slot");t.setAttribute("name","start"),E.appendChild(t),b.appendChild(E);let f=document.createElement("span");f.setAttribute("class","spacer"),b.appendChild(f);let L=document.createElement("span");L.setAttribute("class","slot-wrap");let H=document.createElement("slot");H.setAttribute("name","end"),L.appendChild(H),b.appendChild(L);let C=document.createElement("span");C.setAttribute("class","text"),C.setAttribute("part","theme");let z=document.createTextNode("");return C.appendChild(z),m.push(k(z,()=>n.theme())),b.appendChild(C),n}});var Bm=["a[href]","area[href]","button:not([disabled])",'input:not([disabled]):not([type="hidden"])',"select:not([disabled])","textarea:not([disabled])","iframe","object","embed","summary",'[contenteditable]:not([contenteditable="false"])','[tabindex]:not([tabindex^="-"])'].join(",");function Fn(i){return Array.from(i.querySelectorAll(Bm)).filter((u)=>{if(u.hasAttribute("disabled"))return!1;if(u.getAttribute("aria-hidden")==="true")return!1;return!0})}class hn{saved=null;save(){this.saved=document.activeElement??null}restore(){let i=this.saved;if(this.saved=null,i&&typeof i.focus==="function")i.focus()}}var xn=0,Gu=null,Wu=null;function Si(){if(typeof document>"u")return;if(xn+=1,xn>1)return;let i=document.body;Gu=i.style.overflow,Wu=i.style.paddingRight;let u=typeof window<"u"?window.innerWidth-document.documentElement.clientWidth:0;if(u>0){let d=parseFloat(getComputedStyle(i).paddingRight||"0")||0;i.style.paddingRight=`${d+u}px`}i.style.overflow="hidden"}function On(){if(typeof document>"u")return;if(xn===0)return;if(xn-=1,xn>0)return;let i=document.body;i.style.overflow=Gu??"",i.style.paddingRight=Wu??"",Gu=null,Wu=null}var Im=0,ai=N({mount(i,u){let d=i,m=++Im,n=`sui-dialog-header-${m}`,b=`sui-dialog-body-${m}`,w=d.dialogEl(),r=d.closeBtn(),M=d.sentinelStart(),y=d.sentinelEnd();if(!w)return;let l=U(()=>{let Y=d.headerEl();if(Y)Y.id=n}),E=U(()=>{let Y=d.bodyEl();if(Y)Y.id=b});w.setAttribute("aria-labelledby",n),w.setAttribute("aria-describedby",b);let t=!1,f=new hn;function L(){w.setAttribute("role",d.roleOverride()||"dialog"),w.setAttribute("aria-modal","true")}L();function H(){let Y=Fn(w).filter((K)=>!K.classList.contains("sentinel"));return[...Fn(u),...Y]}function C(){let Y=u.querySelector("[autofocus]")??w.querySelector("[autofocus]");if(Y){Y.focus();return}let O=H(),K=O.find((B)=>B!==r)??O[0];if(K)K.focus();else w.focus()}function z(){f.save();try{w.showModal()}catch{}if(!t)Si(),t=!0;queueMicrotask(()=>C()),u.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function j(){if(w.open)w.close()}let g=()=>{if(d.open())d.open.set(!1);if(t)On(),t=!1;f.restore(),u.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},Z=(Y)=>{if(!d.dismissible())return;if(Y.target===w)d.open.set(!1)},D=()=>{u.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),d.open.set(!1)},J=()=>{let Y=H();if(Y.length)Y[Y.length-1].focus()},X=()=>{let Y=H();if(Y.length)Y[0].focus()};w.addEventListener("close",g),w.addEventListener("click",Z),r?.addEventListener("click",D),M?.addEventListener("focus",J),y?.addEventListener("focus",X);let W=U(()=>{let Y=d.open();if(u.setAttribute("data-state",Y?"open":"closed"),Y&&!w.open)z();else if(!Y&&w.open)j()}),Q=U(()=>{L()});return u.show=()=>d.open.set(!0),u.showModal=()=>d.open.set(!0),u.close=()=>d.open.set(!1),()=>{if(t)On();w.removeEventListener("close",g),w.removeEventListener("click",Z),r?.removeEventListener("click",D),M?.removeEventListener("focus",J),y?.removeEventListener("focus",X),W(),Q(),l(),E()}}});var C5=V("sui-dialog",{props:{open:P.bool({default:!1,reflect:!0}),size:P.union(["sm","md","lg","full"],{default:"md",reflect:!0}),backdrop:P.union(["default","blur","solid"],{default:"default",reflect:!0}),dismissible:P.bool({default:!0}),"no-close-button":P.bool({default:!1,reflect:!0}),"no-header":P.bool({default:!1,reflect:!0}),"no-footer":P.bool({default:!1,reflect:!0}),"role-override":P.union(["dialog","alertdialog"],{default:"dialog",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.size=u.size,n.backdrop=u.backdrop,n.dismissible=u.dismissible,n.noCloseButton=u.noCloseButton,n.noHeader=u.noHeader,n.noFooter=u.noFooter,n.roleOverride=u.roleOverride,n.bodyEl=G(null),n.closeBtn=G(null),n.dialogEl=G(null),n.headerEl=G(null),n.sentinelEnd=G(null),n.sentinelStart=G(null);let b=null;if(ai&&typeof ai.setup==="function"){if(i.scope=n,b=ai.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(z)=>I({anchor:z,test:()=>{if(n.open())return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),g=[],Z=document.createElement("div");Z.setAttribute("class","dialog-content"),En(Z,Zn,{duration:150},"both");let D=document.createElement("div");D.setAttribute("class","header"),D.setAttribute("part","header"),F(n,"headerEl",D);let J=document.createElement("slot");J.setAttribute("name","header"),D.appendChild(J),Z.appendChild(D);let X=document.createElement("div");X.setAttribute("class","body"),X.setAttribute("part","body"),F(n,"bodyEl",X);let W=document.createElement("slot");X.appendChild(W),Z.appendChild(X);let Q=document.createElement("div");Q.setAttribute("class","footer"),Q.setAttribute("part","footer");let Y=document.createElement("slot");return Y.setAttribute("name","footer"),Q.appendChild(Y),Z.appendChild(Q),j.appendChild(Z),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let O of g)try{O()}catch(K){}}}}]}),r=(z)=>I({anchor:z,test:()=>{if(n.open())return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),g=[],Z=document.createElement("sui-portal");Z.setAttribute("target","body");let D=document.createElement("div");return D.setAttribute("class","sui-dialog-presence"),D.setAttribute("aria-hidden","true"),D.setAttribute("data-sui-dialog-open",""),Z.appendChild(D),j.appendChild(Z),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let J of g)try{J()}catch(X){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createElement("dialog");y.setAttribute("part","dialog"),F(n,"dialogEl",y);let l=document.createElement("span");l.setAttribute("tabindex","0"),l.setAttribute("class","sentinel"),l.setAttribute("aria-hidden","true"),F(n,"sentinelStart",l),y.appendChild(l);let E=document.createElement("button");E.setAttribute("class","close"),E.setAttribute("part","close"),E.setAttribute("type","button"),E.setAttribute("aria-label","Close dialog"),F(n,"closeBtn",E);let t=document.createElement("svg");t.setAttribute("viewbox","0 0 16 16"),t.setAttribute("fill","none"),t.setAttribute("stroke","currentColor"),t.setAttribute("stroke-width","1.75"),t.setAttribute("stroke-linecap","round"),t.setAttribute("stroke-linejoin","round"),t.setAttribute("aria-hidden","true"),t.setAttribute("focusable","false");let f=document.createElement("path");f.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),t.appendChild(f),E.appendChild(t),y.appendChild(E);let L=document.createComment("if");y.appendChild(L),m.push(w(L));let H=document.createElement("span");H.setAttribute("tabindex","0"),H.setAttribute("class","sentinel"),H.setAttribute("aria-hidden","true"),F(n,"sentinelEnd",H),y.appendChild(H),M.appendChild(y);let C=document.createComment("if");if(M.appendChild(C),m.push(r(C)),b!=null){let z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(z){let j=new Proxy(n,{get(g,Z){return g[Z]},set(g,Z,D){let J=g[Z];if(J!=null&&typeof J.set==="function"){if(typeof D==="function")J.set(()=>D);else J.set(D);return!0}return!1}});if(z.mount){let g=z.mount(j,d);if(typeof g==="function")m.push(g)}if(z.unmount)m.push(()=>z.unmount(j))}}return n}});var Rm=0,vi=N({mount(i,u){let d=i,m=++Rm,n=`sui-drawer-header-${m}`,b=`sui-drawer-body-${m}`,w=d.dialogEl(),r=d.closeBtn(),M=d.sentinelStart(),y=d.sentinelEnd(),l=d.headerEl(),E=d.bodyEl();if(!w)return;if(l)l.id=n;if(E)E.id=b;w.setAttribute("aria-labelledby",n),w.setAttribute("aria-describedby",b);let t=!1,f=new hn;function L(){let W=Fn(w).filter((Y)=>!Y.classList.contains("sentinel"));return[...Fn(u),...W]}function H(){let W=u.querySelector("[autofocus]")??w.querySelector("[autofocus]");if(W){W.focus();return}let Q=L(),Y=Q.find((O)=>O!==r)??Q[0];if(Y)Y.focus();else w.focus()}function C(){f.save();try{w.showModal()}catch{}if(!t)Si(),t=!0;queueMicrotask(()=>H()),u.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function z(){if(w.open)w.close()}let j=()=>{if(d.open())d.open.set(!1);if(t)On(),t=!1;f.restore(),u.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},g=(W)=>{if(!d.dismissible())return;if(W.target===w)d.open.set(!1)},Z=()=>{u.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),d.open.set(!1)},D=()=>{let W=L();if(W.length)W[W.length-1].focus()},J=()=>{let W=L();if(W.length)W[0].focus()};w.addEventListener("close",j),w.addEventListener("click",g),r?.addEventListener("click",Z),M?.addEventListener("focus",D),y?.addEventListener("focus",J);let X=U(()=>{let W=d.open();if(u.setAttribute("data-state",W?"open":"closed"),W&&!w.open)C();else if(!W&&w.open)z()});return u.show=()=>d.open.set(!0),u.close=()=>d.open.set(!1),()=>{if(t)On();w.removeEventListener("close",j),w.removeEventListener("click",g),r?.removeEventListener("click",Z),M?.removeEventListener("focus",D),y?.removeEventListener("focus",J),X()}}});var J5=V("sui-drawer",{props:{open:P.bool({default:!1,reflect:!0}),side:P.union(["right","left"],{default:"right",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),dismissible:P.bool({default:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.side=u.side,n.size=u.size,n.dismissible=u.dismissible,n.bodyEl=G(null),n.closeBtn=G(null),n.dialogEl=G(null),n.headerEl=G(null),n.sentinelEnd=G(null),n.sentinelStart=G(null);let b=null;if(vi&&typeof vi.setup==="function"){if(i.scope=n,b=vi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("dialog");r.setAttribute("part","dialog"),r.setAttribute("role","dialog"),r.setAttribute("aria-modal","true"),F(n,"dialogEl",r);let M=document.createElement("span");M.setAttribute("tabindex","0"),M.setAttribute("class","sentinel"),M.setAttribute("aria-hidden","true"),F(n,"sentinelStart",M),r.appendChild(M);let y=document.createElement("div");y.setAttribute("class","header"),y.setAttribute("part","header"),F(n,"headerEl",y);let l=document.createElement("div");l.setAttribute("class","header-content");let E=document.createElement("div");E.setAttribute("class","title");let t=document.createElement("slot");t.setAttribute("name","title"),E.appendChild(t),l.appendChild(E);let f=document.createElement("div");f.setAttribute("class","description");let L=document.createElement("slot");L.setAttribute("name","description"),f.appendChild(L),l.appendChild(f),y.appendChild(l);let H=document.createElement("button");H.setAttribute("class","close"),H.setAttribute("part","close"),H.setAttribute("type","button"),H.setAttribute("aria-label","Close drawer"),F(n,"closeBtn",H);let C=document.createElement("svg");C.setAttribute("viewbox","0 0 16 16"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","1.75"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true"),C.setAttribute("focusable","false");let z=document.createElement("path");z.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),C.appendChild(z),H.appendChild(C),y.appendChild(H),r.appendChild(y);let j=document.createElement("div");j.setAttribute("class","body"),j.setAttribute("part","body"),F(n,"bodyEl",j);let g=document.createElement("slot");j.appendChild(g),r.appendChild(j);let Z=document.createElement("div");Z.setAttribute("class","footer"),Z.setAttribute("part","footer");let D=document.createElement("slot");D.setAttribute("name","footer"),Z.appendChild(D),r.appendChild(Z);let J=document.createElement("span");if(J.setAttribute("tabindex","0"),J.setAttribute("class","sentinel"),J.setAttribute("aria-hidden","true"),F(n,"sentinelEnd",J),r.appendChild(J),w.appendChild(r),b!=null){let X=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(X){let W=new Proxy(n,{get(Q,Y){return Q[Y]},set(Q,Y,O){let K=Q[Y];if(K!=null&&typeof K.set==="function"){if(typeof O==="function")K.set(()=>O);else K.set(O);return!0}return!1}});if(X.mount){let Q=X.mount(W,d);if(typeof Q==="function")m.push(Q)}if(X.unmount)m.push(()=>X.unmount(W))}}return n}});var km=0,xi=N({mount(i,u){let d=i,m=`sui-dropdown-menu-panel-${++km}`,n=d.triggerWrap(),b=d.triggerSlot(),w=d.panelEl();if(!w)return;w.id=m;let r="",M=null;function y(J){if(M)clearTimeout(M);r+=J.toLowerCase(),M=setTimeout(()=>{r=""},350);let X=l(),W=X.indexOf(document.activeElement);for(let Q=0;Q<X.length;Q++){let Y=(W+1+Q)%X.length;if((X[Y].textContent??"").trim().toLowerCase().startsWith(r))return Y}return-1}function l(){return Array.from(u.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((J)=>!J.hasAttribute("disabled"))}function E(J){let X=l();if(!X.length)return;let W=(J%X.length+X.length)%X.length;for(let Q of X)Q.setAttribute("tabindex","-1");X[W].setAttribute("tabindex","0"),X[W].focus()}function t(){return l().indexOf(document.activeElement)}function f(){l().forEach((J,X)=>{if(!J.hasAttribute("role"))J.setAttribute("role","menuitem");if(!J.hasAttribute("tabindex"))J.setAttribute("tabindex",X===0?"0":"-1")})}function L(){let J=b?.assignedElements({flatten:!0})??[];for(let X of J)X.setAttribute("aria-haspopup","menu"),X.setAttribute("aria-controls",m),X.setAttribute("aria-expanded",d.open()?"true":"false")}function H(){let J=b?.assignedElements({flatten:!0})?.[0];if(J&&w){let X=J.getBoundingClientRect();w.style.top=`${X.bottom+4}px`,w.style.left=`${X.left}px`}w?.showPopover?.()}function C(){w?.hidePopover?.()}let z=()=>{if(d.open())C();else H()},j=(J)=>{let W=J.newState==="open";if(W!==d.open())d.open.set(W);if(u.setAttribute("data-state",W?"open":"closed"),L(),W)queueMicrotask(()=>{f(),E(0)})},g=(J)=>{let X=J.target.closest("[data-value]");if(!X)return;u.emit("sui-select",{value:X.dataset.value??""}),C()},Z=(J)=>{if(!d.open())return;let X=l();if(!X.length)return;let W=t();if(J.key==="ArrowDown")J.preventDefault(),E(W+1);else if(J.key==="ArrowUp")J.preventDefault(),E(W-1);else if(J.key==="Home")J.preventDefault(),E(0);else if(J.key==="End")J.preventDefault(),E(X.length-1);else if(J.key==="Escape")J.preventDefault(),C();else if(J.key==="Tab")C();else if((J.key==="Enter"||J.key===" ")&&W>=0)J.preventDefault(),X[W].click();else if(J.key.length===1&&!J.ctrlKey&&!J.metaKey){let Q=y(J.key);if(Q>=0)J.preventDefault(),E(Q)}};if(n)_(n,"click",z);if(_(w,"toggle",j),_(w,"click",g),_(u,"keydown",Z),b)_(b,"slotchange",L);_(w,"slotchange",f);let D=U(()=>{let J=d.open();u.setAttribute("data-state",J?"open":"closed"),L()});return()=>{if(M)clearTimeout(M);D()}}});var V5=V("sui-dropdown-menu",{props:{open:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.itemSlot=G(null),n.panelEl=G(null),n.triggerSlot=G(null),n.triggerWrap=G(null);let b=null;if(xi&&typeof xi.setup==="function"){if(i.scope=n,b=xi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("span");r.setAttribute("class","trigger-wrap"),F(n,"triggerWrap",r);let M=document.createElement("slot");M.setAttribute("name","trigger"),F(n,"triggerSlot",M),r.appendChild(M),w.appendChild(r);let y=document.createElement("div");y.setAttribute("class","panel"),y.setAttribute("part","panel"),y.setAttribute("popover","auto"),y.setAttribute("role","menu"),F(n,"panelEl",y);let l=document.createElement("slot");if(F(n,"itemSlot",l),y.appendChild(l),w.appendChild(y),b!=null){let E=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(E){let t=new Proxy(n,{get(f,L){return f[L]},set(f,L,H){let C=f[L];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(E.mount){let f=E.mount(t,d);if(typeof f==="function")m.push(f)}if(E.unmount)m.push(()=>E.unmount(t))}}return n}});var Sm=0,hi=N({mount(i,u){let d=i,m=`sui-context-menu-panel-${++Sm}`,n=d.triggerSlot(),b=d.panelEl();if(!b)return;b.id=m;let w="",r=null;function M(D){if(r)clearTimeout(r);w+=D.toLowerCase(),r=setTimeout(()=>{w=""},350);let J=y(),X=J.indexOf(document.activeElement);for(let W=0;W<J.length;W++){let Q=(X+1+W)%J.length;if((J[Q].textContent??"").trim().toLowerCase().startsWith(w))return Q}return-1}function y(){return Array.from(u.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((D)=>!D.hasAttribute("disabled"))}function l(D){let J=y();if(!J.length)return;let X=(D%J.length+J.length)%J.length;for(let W of J)W.setAttribute("tabindex","-1");J[X].setAttribute("tabindex","0"),J[X].focus()}function E(){return y().indexOf(document.activeElement)}function t(){y().forEach((D,J)=>{if(!D.hasAttribute("role"))D.setAttribute("role","menuitem");if(!D.hasAttribute("tabindex"))D.setAttribute("tabindex",J===0?"0":"-1")})}function f(D,J){if(b)b.style.top=`${J}px`,b.style.left=`${D}px`;b?.showPopover?.()}function L(){b?.hidePopover?.()}u.expose({show:f,close:L});let H=(D)=>{D.preventDefault(),f(D.clientX,D.clientY)},C=(D)=>{let X=D.newState==="open";if(X!==d.open())d.open.set(X);if(u.setAttribute("data-state",X?"open":"closed"),X)u.emit("sui-open"),queueMicrotask(()=>{t(),l(0)});else u.emit("sui-close")},z=(D)=>{let J=D.target.closest("[data-value]");if(!J)return;u.emit("sui-select",{value:J.dataset.value??""}),L()},j=(D)=>{if(!d.open())return;let J=y();if(!J.length)return;let X=E();if(D.key==="ArrowDown")D.preventDefault(),l(X+1);else if(D.key==="ArrowUp")D.preventDefault(),l(X-1);else if(D.key==="Home")D.preventDefault(),l(0);else if(D.key==="End")D.preventDefault(),l(J.length-1);else if(D.key==="Escape")D.preventDefault(),L();else if(D.key==="Tab")L();else if((D.key==="Enter"||D.key===" ")&&X>=0)D.preventDefault(),J[X].click();else if(D.key.length===1&&!D.ctrlKey&&!D.metaKey){let W=M(D.key);if(W>=0)D.preventDefault(),l(W)}},g=()=>{let D=n?.assignedElements({flatten:!0})??[];for(let J of D)J.addEventListener("contextmenu",H)};if(n)_(n,"slotchange",g);g(),_(b,"toggle",C),_(b,"click",z),_(u,"keydown",j);let Z=U(()=>{u.setAttribute("data-state",d.open()?"open":"closed")});return()=>{let D=n?.assignedElements({flatten:!0})??[];for(let J of D)J.removeEventListener("contextmenu",H);if(r)clearTimeout(r);Z()}}});var O5=V("sui-context-menu",{props:{open:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.itemSlot=G(null),n.panelEl=G(null),n.triggerSlot=G(null);let b=null;if(hi&&typeof hi.setup==="function"){if(i.scope=n,b=hi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");r.setAttribute("name","trigger"),F(n,"triggerSlot",r),w.appendChild(r);let M=document.createElement("div");M.setAttribute("class","panel"),M.setAttribute("part","panel"),M.setAttribute("popover","auto"),M.setAttribute("role","menu"),M.setAttribute("aria-label","Context menu"),F(n,"panelEl",M);let y=document.createElement("slot");if(F(n,"itemSlot",y),M.appendChild(y),w.appendChild(M),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var am=0,$i=8;function vm(i,u,d){let m=i.getBoundingClientRect();switch(u.style.top="",u.style.left="",u.style.right="",u.style.bottom="",d){case"top":u.style.bottom=`${window.innerHeight-m.top+$i}px`,u.style.left=`${m.left+m.width/2}px`,u.style.transform="translateX(-50%)";break;case"left":u.style.right=`${window.innerWidth-m.left+$i}px`,u.style.top=`${m.top}px`;break;case"right":u.style.left=`${m.right+$i}px`,u.style.top=`${m.top}px`;break;case"bottom":default:u.style.top=`${m.bottom+$i}px`,u.style.left=`${m.left+m.width/2}px`,u.style.transform="translateX(-50%)"}}var oi=N({mount(i,u){let d=i,m=`sui-hover-card-${++am}`,n=d.triggerSlot(),b=d.panelEl();if(!b)return;b.id=m;let w=null,r=null;function M(){if(w)clearTimeout(w),w=null;if(r)clearTimeout(r),r=null}function y(){let j=n?.assignedElements({flatten:!0})??[];for(let g of j)g.setAttribute("aria-describedby",m)}function l(){M();let j=n?.assignedElements({flatten:!0})?.[0];if(j&&b)vm(j,b,d.placement()||"bottom");b?.showPopover?.()}function E(){M(),b?.hidePopover?.()}u.show=l,u.hide=E;let t=(j)=>{let Z=j.newState==="open";if(Z!==d.open())d.open.set(Z);u.setAttribute("data-state",Z?"open":"closed"),u.dispatchEvent(new CustomEvent(Z?"sui-open":"sui-close",{bubbles:!0,composed:!0}))},f=()=>{M(),w=setTimeout(()=>l(),d.showDelay()??300)},L=()=>{M(),r=setTimeout(()=>E(),d.hideDelay()??100)},H=()=>{M(),l()},C=()=>{M(),r=setTimeout(()=>E(),d.hideDelay()??100)};b.addEventListener("toggle",t),n?.addEventListener("slotchange",y),y(),u.addEventListener("pointerenter",f),u.addEventListener("pointerleave",L),u.addEventListener("focusin",H),u.addEventListener("focusout",C);let z=U(()=>{u.setAttribute("data-state",d.open()?"open":"closed")});return()=>{M(),b.removeEventListener("toggle",t),n?.removeEventListener("slotchange",y),u.removeEventListener("pointerenter",f),u.removeEventListener("pointerleave",L),u.removeEventListener("focusin",H),u.removeEventListener("focusout",C),z()}}});var k5=V("sui-hover-card",{props:{open:P.bool({default:!1,reflect:!0}),placement:P.union(["top","bottom","left","right"],{default:"bottom",reflect:!0}),"show-delay":P.number({default:300}),"hide-delay":P.number({default:100})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.placement=u.placement,n.showDelay=u.showDelay,n.hideDelay=u.hideDelay,n.panelEl=G(null),n.triggerSlot=G(null);let b=null;if(oi&&typeof oi.setup==="function"){if(i.scope=n,b=oi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");r.setAttribute("name","trigger"),F(n,"triggerSlot",r),w.appendChild(r);let M=document.createElement("div");M.setAttribute("class","panel"),M.setAttribute("part","panel"),M.setAttribute("popover","auto"),F(n,"panelEl",M);let y=document.createElement("slot");if(M.appendChild(y),w.appendChild(M),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var si=N({mount(i,u){let d=i,m=()=>Array.from(u.querySelectorAll(":scope > sui-accordion-item")),n=()=>m().filter((l)=>!l.hasAttribute("disabled")),b=()=>{let l=d.value();return new Set(l?l.split(",").filter(Boolean):[])},w=()=>{let l=b();for(let E of m()){let t=E.getAttribute("value")||"";if(l.has(t))E.setAttribute("open","");else E.removeAttribute("open")}},r=U(()=>{d.value(),w()}),M=(l)=>{let E=l;E.stopPropagation();let t=E.detail?.value,f=b();if(d.type()==="single")d.value.set(f.has(t)?"":t);else{if(f.has(t))f.delete(t);else f.add(t);d.value.set(Array.from(f).join(","))}u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:d.value()},bubbles:!0,composed:!0}))},y=(l)=>{let E=l;E.stopPropagation();let t=n();if(!t.length)return;let f=E.target?.closest("sui-accordion-item"),L=f?t.indexOf(f):-1,H;switch(E.detail?.direction){case"next":H=t[(L+1)%t.length];break;case"prev":H=t[(L-1+t.length)%t.length];break;case"first":H=t[0];break;case"last":H=t[t.length-1];break}if(H)H.shadowRoot?.querySelector("button")?.focus()};return u.addEventListener("sui-accordion-item-toggle",M),u.addEventListener("sui-accordion-nav",y),()=>{r(),u.removeEventListener("sui-accordion-item-toggle",M),u.removeEventListener("sui-accordion-nav",y)}}});var $5=V("sui-accordion",{props:{type:P.string({default:"single",reflect:!0}),value:P.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host { display: block; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.type=u.type,n.value=u.value;let b=null;if(si&&typeof si.setup==="function"){if(i.scope=n,b=si.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");if(w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var xm=0,ei=N({mount(i,u){let d=i,m=++xm,n=`sui-acc-trigger-${m}`,b=`sui-acc-content-${m}`,w=U(()=>{let E=d.triggerBtn(),t=d.contentWrap();if(E)E.id=n,E.setAttribute("aria-controls",b);if(t)t.id=b,t.setAttribute("aria-labelledby",n)}),r=U(()=>{let E=d.triggerBtn();if(E)E.setAttribute("aria-expanded",d.open()?"true":"false")}),M=()=>{if(d.disabled())return;u.dispatchEvent(new CustomEvent("sui-accordion-item-toggle",{detail:{value:d.value()},bubbles:!0,composed:!0}))},y=(E)=>{if(E.key==="ArrowDown"||E.key==="ArrowUp")E.preventDefault(),u.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:E.key==="ArrowDown"?"next":"prev"},bubbles:!0,composed:!0}));else if(E.key==="Home"||E.key==="End")E.preventDefault(),u.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:E.key==="Home"?"first":"last"},bubbles:!0,composed:!0}))},l=U(()=>{let E=d.triggerBtn();if(!E)return;return E.addEventListener("click",M),E.addEventListener("keydown",y),()=>{E.removeEventListener("click",M),E.removeEventListener("keydown",y)}});return()=>{w(),r(),l()}}});var _5=V("sui-accordion-item",{props:{value:P.string({default:"",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),open:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.disabled=u.disabled,n.open=u.open,n.contentWrap=G(null),n.triggerBtn=G(null);let b=null;if(ei&&typeof ei.setup==="function"){if(i.scope=n,b=ei.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("button");r.setAttribute("class","trigger"),r.setAttribute("part","trigger"),F(n,"triggerBtn",r),m.push(T(r,"disabled",()=>n.disabled()));let M=document.createElement("span");M.setAttribute("class","trigger-label");let y=document.createElement("slot");y.setAttribute("name","trigger"),M.appendChild(y),r.appendChild(M);let l=document.createElement("svg");l.setAttribute("class","chevron"),l.setAttribute("part","chevron"),l.setAttribute("viewbox","0 0 16 16"),l.setAttribute("fill","none"),l.setAttribute("stroke","currentColor"),l.setAttribute("stroke-width","1.75"),l.setAttribute("stroke-linecap","round"),l.setAttribute("stroke-linejoin","round"),l.setAttribute("aria-hidden","true");let E=document.createElement("path");E.setAttribute("d","M4 6l4 4 4-4"),l.appendChild(E),r.appendChild(l),w.appendChild(r);let t=document.createElement("div");t.setAttribute("class","content-wrap"),t.setAttribute("part","content"),F(n,"contentWrap",t),t.setAttribute("role","region");let f=document.createElement("div");f.setAttribute("class","content-inner");let L=document.createElement("div");L.setAttribute("class","content-body");let H=document.createElement("slot");if(L.appendChild(H),f.appendChild(L),t.appendChild(f),w.appendChild(t),b!=null){let C=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(n,{get(j,g){return j[g]},set(j,g,Z){let D=j[g];if(D!=null&&typeof D.set==="function"){if(typeof Z==="function")D.set(()=>Z);else D.set(Z);return!0}return!1}});if(C.mount){let j=C.mount(z,d);if(typeof j==="function")m.push(j)}if(C.unmount)m.push(()=>C.unmount(z))}}return n}});var Cd=`
sui-list-tile[variant='row'] + sui-list-tile[variant='row'] {
  border-top: 1px solid var(--border-subtle);
}
sui-list-tile[variant='row'][selected] + sui-list-tile[variant='row'],
sui-list-tile[variant='row'] + sui-list-tile[variant='row'][selected] {
  border-top: none;
}
`,zd=new WeakSet;function hm(i){if(zd.has(i))return;zd.add(i);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let d=new CSSStyleSheet;d.replaceSync(Cd),i.adoptedStyleSheets=[...i.adoptedStyleSheets,d];return}}catch{}let u=i.createElement("style");u.setAttribute("data-sui-list-tile-dividers",""),u.textContent=Cd,i.head.appendChild(u)}var pi=N({mount(i,u){let d=i;hm(u.ownerDocument);let m=()=>{if(d.navigable())return"link";if(d.interactive())return"button";return"listitem"},n=U(()=>{d.navigable(),d.interactive(),u.setAttribute("role",m())}),b=U(()=>{u.setAttribute("aria-selected",d.selected()?"true":"false")}),w=U(()=>{u.setAttribute("aria-disabled",d.disabled()?"true":"false")}),r=U(()=>{let t=d.tileEl();if(!t)return;let f=d.interactive(),L=d.disabled();if(f&&!L)t.setAttribute("tabindex","0");else t.removeAttribute("tabindex")}),M=()=>{if(d.disabled()||!d.interactive())return!1;let t=new CustomEvent("sui-select",{detail:{},bubbles:!0,composed:!0,cancelable:!0});return u.dispatchEvent(t),t.defaultPrevented},y=(t)=>{if(!d.interactive())return;if(M()&&d.navigable())t.preventDefault()},l=(t)=>{if(!d.interactive()||d.disabled())return;if(d.navigable()&&t.key==="Enter")return;if(t.key==="Enter"||t.key===" ")t.preventDefault(),M()},E=U(()=>{let t=d.tileEl();if(!t)return;return t.addEventListener("click",y),t.addEventListener("keydown",l),()=>{t.removeEventListener("click",y),t.removeEventListener("keydown",l)}});return()=>{n(),b(),w(),r(),E()}}});var b8=V("sui-list-tile",{props:{variant:P.string({default:"row",reflect:!0}),interactive:P.bool({default:!1,reflect:!0}),navigable:P.string({default:"",reflect:!0}),selected:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),density:P.string({default:"normal",reflect:!0}),tone:P.string({default:"default",reflect:!0}),target:P.string({default:"",reflect:!0}),rel:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.variant=u.variant,n.interactive=u.interactive,n.navigable=u.navigable,n.selected=u.selected,n.disabled=u.disabled,n.density=u.density,n.tone=u.tone,n.target=u.target,n.rel=u.rel,n.tileEl=G(null);let b=null;if(pi&&typeof pi.setup==="function"){if(i.scope=n,b=pi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(y)=>I({anchor:y,test:()=>{if(n.navigable())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),E=[],t=document.createElement("a");t.setAttribute("class","tile"),t.setAttribute("part","button"),F(n,"tileEl",t),E.push(T(t,"href",()=>n.navigable())),E.push(T(t,"target",()=>n.target()||null)),E.push(T(t,"rel",()=>n.rel()||null));let f=document.createElement("div");f.setAttribute("class","row"),f.setAttribute("part","row");let L=document.createElement("slot");L.setAttribute("name","leading"),f.appendChild(L);let H=document.createElement("span");H.setAttribute("class","primary"),H.setAttribute("part","primary");let C=document.createElement("slot");H.appendChild(C),f.appendChild(H);let z=document.createElement("span");z.setAttribute("part","trailing");let j=document.createElement("slot");j.setAttribute("name","trailing"),z.appendChild(j),f.appendChild(z),t.appendChild(f);let g=document.createElement("div");g.setAttribute("class","secondary"),g.setAttribute("part","secondary");let Z=document.createElement("slot");return Z.setAttribute("name","meta"),g.appendChild(Z),t.appendChild(g),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let D of E)try{D()}catch(J){}}}},()=>{let l=document.createDocumentFragment(),E=[],t=document.createElement("div");t.setAttribute("class","tile"),t.setAttribute("part","button"),F(n,"tileEl",t);let f=document.createElement("div");f.setAttribute("class","row"),f.setAttribute("part","row");let L=document.createElement("slot");L.setAttribute("name","leading"),f.appendChild(L);let H=document.createElement("span");H.setAttribute("class","primary"),H.setAttribute("part","primary");let C=document.createElement("slot");H.appendChild(C),f.appendChild(H);let z=document.createElement("span");z.setAttribute("part","trailing");let j=document.createElement("slot");j.setAttribute("name","trailing"),z.appendChild(j),f.appendChild(z),t.appendChild(f);let g=document.createElement("div");g.setAttribute("class","secondary"),g.setAttribute("part","secondary");let Z=document.createElement("slot");return Z.setAttribute("name","meta"),g.appendChild(Z),t.appendChild(g),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let D of E)try{D()}catch(J){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createComment("if");if(r.appendChild(M),m.push(w(M)),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let l=new Proxy(n,{get(E,t){return E[t]},set(E,t,f){let L=E[t];if(L!=null&&typeof L.set==="function"){if(typeof f==="function")L.set(()=>f);else L.set(f);return!0}return!1}});if(y.mount){let E=y.mount(l,d);if(typeof E==="function")m.push(E)}if(y.unmount)m.push(()=>y.unmount(l))}}return n}});var ci=N({scope:(i,u)=>{let d=G(null),m=G(!1);return{caughtError:d,hasFallbackSlot:m,reset:()=>{if(d.set(null),u instanceof HTMLElement)u.removeAttribute("data-error-message")}}},mount(i,u){let d=u.querySelectorAll(':scope > template[slot="fallback"]');for(let m of Array.from(d)){let n=document.createElement("div");n.setAttribute("slot","fallback"),n.appendChild(m.content.cloneNode(!0)),m.replaceWith(n)}return i.hasFallbackSlot.set(!!u.querySelector(':scope > [slot="fallback"]')),uu(u,{catch(m,n){if(i.caughtError()!==null)return;let b=m instanceof Error?m:Error(String(m));u.setAttribute("data-error-message",b.message),i.caughtError.set(b),u.dispatchEvent(new CustomEvent("sui-error-boundary:caught",{bubbles:!0,composed:!0,detail:{error:b,source:n}}))},reset:i.reset}),u.reset=i.reset,()=>{du(u),delete u.reset}}});var r8=V("sui-error-boundary",{props:{"fallback-text":P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.fallbackText=u.fallbackText;let b=null;if(ci&&typeof ci.setup==="function"){if(i.scope=n,b=ci.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>I({anchor:l,test:()=>{if(n.hasFallbackSlot())return 0;return 1},branches:[()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("slot");return f.setAttribute("name","fallback"),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let L of t)try{L()}catch(H){}}}},()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("div");f.setAttribute("class","fallback"),f.setAttribute("part","fallback"),f.setAttribute("role","alert");let L=document.createElement("div");L.setAttribute("class","message"),L.setAttribute("part","message");let H=document.createTextNode("");L.appendChild(H),t.push(k(H,()=>n.fallbackText()||"Something went wrong")),f.appendChild(L);let C=document.createElement("button");C.setAttribute("class","retry"),C.setAttribute("part","retry"),C.setAttribute("type","button"),t.push(p(C,"click",function(){let j=n.reset;return typeof j==="function"?j.bind(void 0):()=>{}}(),{modifiers:[]}));let z=document.createTextNode("Retry");return C.appendChild(z),f.appendChild(C),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let j of t)try{j()}catch(g){}}}}]}),r=(l)=>I({anchor:l,test:()=>{if(n.caughtError()===null)return 0;return 1},branches:[()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("slot");return E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let L of t)try{L()}catch(H){}}}},()=>{let E=document.createDocumentFragment(),t=[],f=document.createComment("if");return E.appendChild(f),t.push(w(f)),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let L of t)try{L()}catch(H){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createComment("if");if(M.appendChild(y),m.push(r(y)),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var _i=N({mount(i,u){let d=i,m=[],n=null,b=()=>{let t=d.target()||"body",f=document.querySelector(t);if(!f)return console.warn(`[sui-portal] target "${t}" not found in document`),null;return f},w=(t)=>{let f=d.portalSlot(),L=f?f.assignedNodes():Array.from(u.childNodes);for(let H of L)if(t.appendChild(H),!m.includes(H))m.push(H)},r=(t)=>{for(let f of m)t.appendChild(f)},M=()=>{for(let t of m)if(t.parentNode)t.parentNode.removeChild(t);m.length=0},y=U(()=>{let t=b();if(!t){n=null;return}if(n===null)w(t);else if(n!==t)r(t);n=t}),l=d.portalSlot(),E=()=>{if(!n||!l)return;for(let t of l.assignedNodes())if(!m.includes(t))n.appendChild(t),m.push(t)};if(l)l.addEventListener("slotchange",E);return()=>{if(y(),l)l.removeEventListener("slotchange",E);M()}}});var z8=V("sui-portal",{props:{target:P.string({default:"body"})},shadow:!0,styles:`
  :host { display: none; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.target=u.target,n.portalSlot=G(null);let b=null;if(_i&&typeof _i.setup==="function"){if(i.scope=n,b=_i.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");if(F(n,"portalSlot",r),w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var n0=N({mount(i,u){let d=i,m=!1,n=(M)=>{M.textContent=m?"Hide details":"Show details",M.setAttribute("aria-expanded",m?"true":"false")},b=U(()=>{let M=d.detailsSlot();if(!M)return;let y=()=>{if(M.assignedNodes({flatten:!0}).some((E)=>E.nodeType===Node.ELEMENT_NODE||E.nodeType===Node.TEXT_NODE&&(E.textContent??"").trim().length>0))u.setAttribute("data-has-details","");else{u.removeAttribute("data-has-details"),u.removeAttribute("data-details-open"),m=!1;let E=d.detailsToggleBtn();if(E)n(E)}};return M.addEventListener("slotchange",y),y(),()=>M.removeEventListener("slotchange",y)}),w=U(()=>{let M=d.detailsToggleBtn();if(!M)return;n(M);let y=()=>{if(m=!m,m)u.setAttribute("data-details-open","");else u.removeAttribute("data-details-open");n(M)};return M.addEventListener("click",y),()=>M.removeEventListener("click",y)}),r=U(()=>{let M=d.actionSlot();if(!M)return;let y=()=>{let l=M.assignedElements().find((E)=>E.getAttribute("slot")==="retry");if(l)requestAnimationFrame(()=>l.focus?.())};return M.addEventListener("slotchange",y),()=>M.removeEventListener("slotchange",y)});return()=>{b(),w(),r()}}});var J8=V("sui-error-state",{props:{title:P.string({default:""}),body:P.string({default:""}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:P.union(["default","inline"],{default:"default",reflect:!0}),icon:P.string({default:"alert-circle"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.title=u.title,n.body=u.body,n.size=u.size,n.variant=u.variant,n.icon=u.icon,n.actionSlot=G(null),n.detailsPanel=G(null),n.detailsSlot=G(null),n.detailsToggleBtn=G(null);let b=null;if(n0&&typeof n0.setup==="function"){if(i.scope=n,b=n0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","box"),r.setAttribute("part","box"),r.setAttribute("role","alert");let M=document.createElement("div");M.setAttribute("class","icon"),M.setAttribute("part","icon"),M.setAttribute("aria-hidden","true");let y=document.createElement("sui-icon");m.push(T(y,"name",()=>n.icon())),y.setAttribute("size","md"),M.appendChild(y),r.appendChild(M);let l=document.createElement("div");l.setAttribute("class","copy");let E=document.createElement("div");E.setAttribute("class","title"),E.setAttribute("part","title");let t=document.createElement("slot");t.setAttribute("name","title");let f=document.createTextNode("");t.appendChild(f),m.push(k(f,()=>n.title()||"Something went wrong")),E.appendChild(t),l.appendChild(E);let L=document.createElement("div");L.setAttribute("class","desc"),L.setAttribute("part","desc");let H=document.createElement("slot"),C=document.createTextNode("");H.appendChild(C),m.push(k(C,()=>n.body())),L.appendChild(H),l.appendChild(L);let z=document.createElement("button");z.setAttribute("class","details-toggle"),F(n,"detailsToggleBtn",z),z.setAttribute("type","button"),l.appendChild(z);let j=document.createElement("div");j.setAttribute("class","details"),F(n,"detailsPanel",j);let g=document.createElement("slot");g.setAttribute("name","details"),F(n,"detailsSlot",g),j.appendChild(g),l.appendChild(j),r.appendChild(l);let Z=document.createElement("div");Z.setAttribute("class","actions"),Z.setAttribute("part","actions");let D=document.createElement("slot");if(D.setAttribute("name","action"),F(n,"actionSlot",D),Z.appendChild(D),r.appendChild(Z),w.appendChild(r),b!=null){let J=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(J){let X=new Proxy(n,{get(W,Q){return W[Q]},set(W,Q,Y){let O=W[Q];if(O!=null&&typeof O.set==="function"){if(typeof Y==="function")O.set(()=>Y);else O.set(Y);return!0}return!1}});if(J.mount){let W=J.mount(X,d);if(typeof W==="function")m.push(W)}if(J.unmount)m.push(()=>J.unmount(X))}}return n}});var i0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","progressbar");let m=U(()=>{let n=Math.max(1,Math.floor(d.total()||0)),b=Math.max(1,Math.min(n,Math.floor(d.current()||1))),w=d.label();if(u.setAttribute("aria-valuemin","1"),u.setAttribute("aria-valuemax",String(n)),u.setAttribute("aria-valuenow",String(b)),w)u.setAttribute("aria-label",w);else u.setAttribute("aria-label",`Step ${b} of ${n}`);let r=d.dotsEl();if(!r)return;let M=[];for(let y=1;y<=n;y++){let l=document.createElement("span");l.className="dot",l.setAttribute("part","dot"),l.setAttribute("data-state",y===b?"active":y<b?"done":"pending"),l.setAttribute("aria-hidden","true"),M.push(l)}r.replaceChildren(...M)});return()=>{m()}}});var V8=V("sui-stepper",{props:{total:P.number({default:4,reflect:!0}),current:P.number({default:1,reflect:!0}),tone:P.string({default:"",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.total=u.total,n.current=u.current,n.tone=u.tone,n.size=u.size,n.label=u.label,n.dotsEl=G(null);let b=null;if(i0&&typeof i0.setup==="function"){if(i.scope=n,b=i0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");if(r.setAttribute("class","dots"),F(n,"dotsEl",r),w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var $m=new Set(["done","failed","pending","running"]);function om(i){return i.split(",").map((u)=>u.trim().toLowerCase()).filter((u)=>u.length>0).map((u)=>$m.has(u)?u:"pending")}var u0=N({scope:()=>({segments:G([]),ariaLabel:G(null)}),mount(i,u){let d=U(()=>{let m=om(i.values()||"");i.segments.set(m.map((n,b)=>({id:`s${b}`,state:n}))),i.ariaLabel.set(m.length===0?null:`${m.length} step outcome${m.length===1?"":"s"}`)});return()=>d()}});var O8=V("sui-step-segments",{props:{values:P.string({default:"",reflect:!0}),size:P.union(["sm","md"],{default:"sm",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.values=u.values,n.size=u.size;let b=null;if(u0&&typeof u0.setup==="function"){if(i.scope=n,b=u0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>o({anchor:l,items:()=>n.segments(),key:(E,t)=>{let f=Object.create(n);return f.seg=E,f.__i=t,((L)=>E.id)(f)},render:(E,t)=>{let f=Object.create(n);return f.seg=E,f.__i=t,f.__v=G(0),((L)=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("span");z.setAttribute("class","seg"),z.setAttribute("part","seg");{let j=L.seg.state;if(j==null||j===!1||j==="")z.removeAttribute("data-state");else z.setAttribute("data-state",j===!0?"":String(j))}return H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let j of C)try{j()}catch(g){}},update:(j,g)=>{L.seg=j,L.__i=g;{let Z=L.seg.state;if(Z==null||Z===!1||Z==="")z.removeAttribute("data-state");else z.setAttribute("data-state",Z===!0?"":String(Z))}L.__v.set((Z)=>Z+1)}}})(f)}}),r=d.shadowRoot??d;q(r);let M=document.createElement("span");M.setAttribute("class","row"),M.setAttribute("part","row"),m.push(T(M,"role",()=>n.segments().length>0?"img":null)),m.push(T(M,"aria-label",()=>n.ariaLabel()));let y=document.createComment("for");if(M.appendChild(y),m.push(w(y)),r.appendChild(M),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var d0=N({mount(i,u){if(!u.hasAttribute("role"))u.setAttribute("role","list")}});var k8=V("sui-timeline",{shadow:!0,styles:`
  :host { display: block; position: relative; }
  .list { display: flex; flex-direction: column; gap: 4px; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null),b=null;if(d0&&typeof d0.setup==="function"){if(i.scope=n,b=d0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","list"),r.setAttribute("part","list");let M=document.createElement("slot");if(r.appendChild(M),w.appendChild(r),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let l=new Proxy(n,{get(E,t){return E[t]},set(E,t,f){let L=E[t];if(L!=null&&typeof L.set==="function"){if(typeof f==="function")L.set(()=>f);else L.set(f);return!0}return!1}});if(y.mount){let E=y.mount(l,d);if(typeof E==="function")m.push(E)}if(y.unmount)m.push(()=>y.unmount(l))}}return n}});var m0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","listitem");let m=U(()=>{if(d.interactive()){if(u.setAttribute("role","button"),!u.hasAttribute("tabindex"))u.setAttribute("tabindex","0")}else u.setAttribute("role","listitem"),u.removeAttribute("tabindex")}),n=()=>{u.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},b=()=>{if(d.interactive())n()},w=(r)=>{if(!d.interactive())return;if(r.key==="Enter"||r.key===" ")r.preventDefault(),n()};return u.addEventListener("click",b),u.addEventListener("keydown",w),()=>{m(),u.removeEventListener("click",b),u.removeEventListener("keydown",w)}}});var $8=V("sui-timeline-item",{props:{tone:P.string({default:"default",reflect:!0}),at:P.string({default:"",reflect:!0}),interactive:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.at=u.at,n.interactive=u.interactive;let b=null;if(m0&&typeof m0.setup==="function"){if(i.scope=n,b=m0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(C)=>I({anchor:C,test:()=>{if(n.at())return 0;return-1},branches:[()=>{let z=document.createDocumentFragment(),j=[],g=document.createElement("span");g.setAttribute("class","at"),g.setAttribute("part","at");let Z=document.createTextNode("");return g.appendChild(Z),j.push(k(Z,()=>n.at())),z.appendChild(g),{nodes:Array.from(z.childNodes),cleanup:()=>{for(let D of j)try{D()}catch(J){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createElement("div");M.setAttribute("class","gutter");let y=document.createElement("span");y.setAttribute("class","dot"),y.setAttribute("part","dot"),M.appendChild(y),r.appendChild(M);let l=document.createElement("div");l.setAttribute("class","body"),l.setAttribute("part","body");let E=document.createElement("div");E.setAttribute("class","head");let t=document.createElement("slot");t.setAttribute("name","title"),E.appendChild(t);let f=document.createComment("if");E.appendChild(f),m.push(w(f)),l.appendChild(E);let L=document.createElement("div");L.setAttribute("class","content");let H=document.createElement("slot");if(L.appendChild(H),l.appendChild(L),r.appendChild(l),b!=null){let C=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(n,{get(j,g){return j[g]},set(j,g,Z){let D=j[g];if(D!=null&&typeof D.set==="function"){if(typeof Z==="function")D.set(()=>Z);else D.set(Z);return!0}return!1}});if(C.mount){let j=C.mount(z,d);if(typeof j==="function")m.push(j)}if(C.unmount)m.push(()=>C.unmount(z))}}return n}});var sm=["Waking daemon","Warming routes","Fetching state","Hydrating shell"];function jd(i){let u=i.trim();if(!u)return sm;return u.split("|").map((d)=>d.trim()).filter(Boolean)}var b0=N({mount(i,u){let d=i,m=d.msgEl();if(!m)return;let n=0;(()=>{let r=jd(d.messages());m.textContent=r[n]??"",m.style.opacity="1"})();let w=setInterval(()=>{m.style.opacity="0",requestAnimationFrame(()=>{let r=jd(d.messages());n=(n+1)%r.length,m.textContent=r[n]??"",m.style.opacity="1"})},1400);return()=>clearInterval(w)}});var _8=V("sui-loading",{props:{messages:P.string({default:""}),compact:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.messages=u.messages,n.compact=u.compact,n.msgEl=G(null);let b=null;if(b0&&typeof b0.setup==="function"){if(i.scope=n,b=b0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","stage"),r.setAttribute("part","stage"),r.setAttribute("aria-hidden","true");let M=document.createElement("div");M.setAttribute("class","rocket");let y=document.createElement("svg");y.setAttribute("viewbox","0 0 32 40"),y.setAttribute("fill","none"),y.setAttribute("xmlns","http://www.w3.org/2000/svg");let l=document.createElement("path");l.setAttribute("d","M16 2 C20 6 22 11 22 16 H10 C10 11 12 6 16 2 Z"),l.setAttribute("fill","currentColor"),y.appendChild(l);let E=document.createElement("rect");E.setAttribute("x","10"),E.setAttribute("y","16"),E.setAttribute("width","12"),E.setAttribute("height","14"),E.setAttribute("rx","2"),E.setAttribute("fill","currentColor"),E.setAttribute("opacity","0.9"),y.appendChild(E);let t=document.createElement("circle");t.setAttribute("cx","16"),t.setAttribute("cy","20"),t.setAttribute("r","2.5"),t.setAttribute("fill","var(--background, #fff)"),t.setAttribute("opacity","0.85"),y.appendChild(t);let f=document.createElement("path");f.setAttribute("d","M10 24 L4 32 L10 30 Z"),f.setAttribute("fill","currentColor"),f.setAttribute("opacity","0.75"),y.appendChild(f);let L=document.createElement("path");L.setAttribute("d","M22 24 L28 32 L22 30 Z"),L.setAttribute("fill","currentColor"),L.setAttribute("opacity","0.75"),y.appendChild(L);let H=document.createElement("rect");H.setAttribute("x","13"),H.setAttribute("y","29"),H.setAttribute("width","6"),H.setAttribute("height","2"),H.setAttribute("rx","0.5"),H.setAttribute("fill","var(--background, #000)"),H.setAttribute("opacity","0.5"),y.appendChild(H),M.appendChild(y),r.appendChild(M);let C=document.createElement("div");C.setAttribute("class","flame"),r.appendChild(C);let z=document.createElement("div");z.setAttribute("class","particle p1"),r.appendChild(z);let j=document.createElement("div");j.setAttribute("class","particle p2"),r.appendChild(j);let g=document.createElement("div");g.setAttribute("class","particle p3"),r.appendChild(g),w.appendChild(r);let Z=document.createElement("div");if(Z.setAttribute("class","message"),Z.setAttribute("part","message"),F(n,"msgEl",Z),w.appendChild(Z),b!=null){let D=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(D){let J=new Proxy(n,{get(X,W){return X[W]},set(X,W,Q){let Y=X[W];if(Y!=null&&typeof Y.set==="function"){if(typeof Q==="function")Y.set(()=>Q);else Y.set(Q);return!0}return!1}});if(D.mount){let X=D.mount(J,d);if(typeof X==="function")m.push(X)}if(D.unmount)m.push(()=>D.unmount(J))}}return n}});function Pd(i){let u=document.getElementById("ship-view-root");if(!u)return null;let d=i.target;if(!d||!u.contains(d))return null;if(d.closest("sui-edit-overlay"))return null;return d}function gd(i){let u=i.tagName.toLowerCase(),d=i.id||null,m=u;if(d)m=`#${d}`;else if(i.classList.length>0)m=`${u}.${Array.from(i.classList).slice(0,2).join(".")}`;return{tag:u,selector:m,id:d}}var f0=N({mount(i,u){let d=i,m=d.boxEl(),n=d.labelEl(),b=null;function w(L){if(!m||!n)return;if(!L){m.classList.remove("visible"),n.classList.remove("visible");return}m.style.cssText=`top:${L.top}px;left:${L.left}px;width:${L.width}px;height:${L.height}px`,m.classList.add("visible"),n.style.cssText=`top:${L.top}px;left:${L.left}px`,n.textContent=`<${L.tag}${L.id?`#${L.id}`:""}>`,n.classList.add("visible")}let r=(L)=>{if(b)return;b=requestAnimationFrame(()=>{b=null;let H=Pd(L);if(!H){w(null);return}let C=H.getBoundingClientRect(),{tag:z,id:j}=gd(H);w({top:C.top,left:C.left,width:C.width,height:C.height,tag:z,id:j})})},M=()=>{w(null)},y=(L)=>{let H=Pd(L);if(!H)return;L.preventDefault(),L.stopPropagation();let{tag:C,selector:z,id:j}=gd(H);(u.closest("sui-shell")??u.closest("ship-shell"))?.dispatchEvent(new CustomEvent("sui:chat:annotate",{detail:{tag:C,selector:z,id:j,preview:(H.textContent??"").trim().slice(0,60)},bubbles:!0})),window.ship?.chat?.open?.()};function l(){let L=document.getElementById("ship-view-root");if(!L)return;L.addEventListener("pointermove",r,!0),L.addEventListener("pointerleave",M,!0),L.addEventListener("click",y,!0)}function E(){let L=document.getElementById("ship-view-root");if(!L)return;L.removeEventListener("pointermove",r,!0),L.removeEventListener("pointerleave",M,!0),L.removeEventListener("click",y,!0)}function t(){let L=document.body.classList.contains("edit-mode");if(u.classList.toggle("active",L),L)l();else if(E(),w(null),b)cancelAnimationFrame(b),b=null}t();let f=new MutationObserver(()=>t());return f.observe(document.body,{attributes:!0,attributeFilter:["class"]}),()=>{if(f.disconnect(),E(),b)cancelAnimationFrame(b),b=null}}});var bM=V("sui-edit-overlay",{shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.boxEl=G(null),n.labelEl=G(null);let b=null;if(f0&&typeof f0.setup==="function"){if(i.scope=n,b=f0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","box"),r.setAttribute("part","box"),F(n,"boxEl",r),w.appendChild(r);let M=document.createElement("div");M.setAttribute("class","label"),M.setAttribute("part","label"),F(n,"labelEl",M),w.appendChild(M);let y=document.createElement("div");y.setAttribute("class","hint"),y.setAttribute("part","hint");let l=document.createTextNode(`
  Edit mode — hover an element, click to annotate.
  `);y.appendChild(l);let E=document.createElement("kbd"),t=document.createTextNode("⌘⇧E");E.appendChild(t),y.appendChild(E);let f=document.createTextNode(` to exit.
`);if(y.appendChild(f),w.appendChild(y),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let H=new Proxy(n,{get(C,z){return C[z]},set(C,z,j){let g=C[z];if(g!=null&&typeof g.set==="function"){if(typeof j==="function")g.set(()=>j);else g.set(j);return!0}return!1}});if(L.mount){let C=L.mount(H,d);if(typeof C==="function")m.push(C)}if(L.unmount)m.push(()=>L.unmount(H))}}return n}});function em(i=Date.now()){let u=i,d=[,,,,,,,,,,];for(let b=9;b>=0;b--)d[b]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[u%32],u=Math.floor(u/32);let m=Array(16),n=typeof globalThis<"u"?globalThis.crypto:void 0;if(n?.getRandomValues){let b=new Uint8Array(16);n.getRandomValues(b);for(let w=0;w<16;w++)m[w]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[b[w]%32]}else for(let b=0;b<16;b++)m[b]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[Math.floor(Math.random()*32)];return d.join("")+m.join("")}function Dd(i){let u={};if(i.viewId)u.view_id=i.viewId;if(i.selector)u.selector=i.selector;if(i.rect)u.rect=i.rect;return{id:em(),kind:i.kind,target:u,note:i.note,created_at:new Date().toISOString(),author_kind:"human"}}var pm="/api/feedback",t0=N({scope:(i,u)=>{let{viewId:d,mode:m,endpoint:n,staging:b}=i,w=G([]),r=G(null),M=G(null),y=G(""),l={},E=h(()=>{let D=M();if(!D)return!1;return Math.abs(D.curX-D.startX)>4||Math.abs(D.curY-D.startY)>4}),t=h(()=>{let D=M();if(!D)return"";let J=Math.min(D.startX,D.curX),X=Math.min(D.startY,D.curY),W=Math.abs(D.curX-D.startX),Q=Math.abs(D.curY-D.startY);return`left:${J}px;top:${X}px;width:${W}px;height:${Q}px`}),f=h(()=>{let D=r();if(!D)return"";let J=Math.min(D.x+12,1e4),X=Math.min(D.y+12,1e4);return`left:${J}px;top:${X}px`}),L=h(()=>w().map((D,J)=>({id:D.id,kind:D.kind,index:J,rect:D.kind==="region"&&D.target.rect?D.target.rect.join(","):"",pos:l[D.id]??{x:0,y:0}})));async function H(D){let J=n()||pm;try{let X=await fetch(J,{method:"POST",credentials:"include",headers:{"Content-Type":"application/json"},body:JSON.stringify(D)});if(!X.ok){u.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:`http_${X.status}`,id:D.id},bubbles:!0,composed:!0}));return}w.set(w().filter((W)=>W.id!==D.id)),u.dispatchEvent(new CustomEvent("sui-annotation-sent",{detail:{id:D.id,status:X.status},bubbles:!0,composed:!0}))}catch(X){u.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:"network_error",message:String(X),id:D.id},bubbles:!0,composed:!0}))}}function C(D){let J=r();if(!J)return;let X=D.trim();if(!X){r.set(null),y.set("");return}let W=Dd({viewId:d()||void 0,kind:J.kind,selector:J.kind==="point"?J.selector:void 0,rect:J.kind==="region"?J.rect:void 0,note:X});if(l[W.id]={x:J.x,y:J.y},w.set([...w(),W]),u.dispatchEvent(new CustomEvent("sui-annotation-staged",{detail:W,bubbles:!0,composed:!0})),r.set(null),y.set(""),!b())H(W)}let z=()=>{C(y())},j=()=>{r.set(null),y.set("")},g=(D)=>{if(D.key==="Escape")j();else if(D.key==="Enter"&&(D.metaKey||D.ctrlKey)){let J=D.currentTarget;C(J.value)}},Z=e(null);return{staged:w,pending:r,drag:M,noteText:y,dragVisible:E,dragRectStyle:t,popoverStyle:f,markers:L,overlayEl:Z,confirmNote:C,sendNote:z,cancelPending:j,onNoteKeydown:g,sendOne:H}},mount(i,u){let d=i,m=null,n=null,b=(l)=>{let E=d.overlayEl();if(!E)return{x:0,y:0};let t=E.getBoundingClientRect();return{x:l.clientX-t.left,y:l.clientY-t.top}},w=(l)=>{if(d.mode()==="off")return;let t=l.target;if(t.closest("[data-annotation-popover]")||t.closest("sui-annotation-marker"))return;if(d.pending()){d.pending.set(null),d.noteText.set("");return}l.preventDefault();let f=b(l);m={startX:f.x,startY:f.y,curX:f.x,curY:f.y},d.drag.set(m),l.target.setPointerCapture?.(l.pointerId)},r=(l)=>{if(!m)return;let E=b(l);if(m={...m,curX:E.x,curY:E.y},n!==null)return;n=requestAnimationFrame(()=>{if(n=null,m)d.drag.set({...m})})},M=(l)=>{if(!m)return;let E=b(l),t=Math.abs(E.x-m.startX),f=Math.abs(E.y-m.startY),L=d.mode(),H=null;if(L==="point"||L==="region"&&t<8&&f<8)H={kind:"point",x:E.x,y:E.y,selector:"body"};else if(L==="region"){let C=Math.min(m.startX,E.x),z=Math.min(m.startY,E.y),j=[Math.round(C),Math.round(z),Math.round(t),Math.round(f)];H={kind:"region",x:C+t/2,y:z+f/2,rect:j}}if(m=null,d.drag.set(null),d.noteText.set(""),H)d.pending.set(H)},y=d.overlayEl();if(y)y.addEventListener("pointerdown",w),y.addEventListener("pointermove",r),y.addEventListener("pointerup",M);return Object.defineProperty(u,"staged",{configurable:!0,get:()=>d.staged()}),u.send=async()=>{for(let l of[...d.staged()])await d.sendOne(l)},u.clear=()=>{d.staged.set([])},()=>{if(n!==null)cancelAnimationFrame(n),n=null;if(y)y.removeEventListener("pointerdown",w),y.removeEventListener("pointermove",r),y.removeEventListener("pointerup",M)}}});var yM=V("sui-annotation-layer",{props:{"view-id":P.string({default:""}),mode:P.union(["off","point","region"],{default:"off",reflect:!0}),endpoint:P.string({default:"/api/feedback"}),staging:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.viewId=u.viewId,n.mode=u.mode,n.endpoint=u.endpoint,n.staging=u.staging,n.overlayEl=G(null);let b=null;if(t0&&typeof t0.setup==="function"){if(i.scope=n,b=t0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(C)=>I({anchor:C,test:()=>{if(n.dragVisible())return 0;return-1},branches:[()=>{let z=document.createDocumentFragment(),j=[],g=document.createElement("div");return g.setAttribute("class","drag-rect"),j.push(T(g,"style",()=>n.dragRectStyle())),z.appendChild(g),{nodes:Array.from(z.childNodes),cleanup:()=>{for(let Z of j)try{Z()}catch(D){}}}}]}),r=(C)=>o({anchor:C,items:()=>n.markers(),key:(z,j)=>{let g=Object.create(n);return g.m=z,g.__i=j,((Z)=>z.id)(g)},render:(z,j)=>{let g=Object.create(n);return g.m=z,g.__i=j,g.__v=G(0),((Z)=>{let D=document.createDocumentFragment(),J=[],X=document.createElement("sui-annotation-marker");return X.kind=Z.m.kind,X.index=Z.m.index,X.rect=Z.m.rect,X.x=Z.m.pos.x,X.y=Z.m.pos.y,D.appendChild(X),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let W of J)try{W()}catch(Q){}},update:(W,Q)=>{Z.m=W,Z.__i=Q,X.kind=Z.m.kind,X.index=Z.m.index,X.rect=Z.m.rect,X.x=Z.m.pos.x,X.y=Z.m.pos.y,Z.__v.set((Y)=>Y+1)}}})(g)}}),M=(C)=>I({anchor:C,test:()=>{if(n.pending())return 0;return-1},branches:[()=>{let z=document.createDocumentFragment(),j=[],g=document.createElement("div");g.setAttribute("class","popover"),g.setAttribute("data-annotation-popover",""),j.push(T(g,"style",()=>n.popoverStyle())),j.push(p(g,"pointerdown",function(){let Y=(O)=>O.stopPropagation();return typeof Y==="function"?Y.bind(void 0):()=>{}}(),{modifiers:[]}));let Z=document.createElement("textarea");Z.setAttribute("part","note-input"),Z.setAttribute("placeholder","Add a note…"),j.push(gn(Z,n.noteText)),j.push(p(Z,"keydown",function(){let Y=n.onNoteKeydown;return typeof Y==="function"?Y.bind(void 0):()=>{}}(),{modifiers:[]})),g.appendChild(Z);let D=document.createElement("div");D.setAttribute("class","popover-actions");let J=document.createElement("button");J.setAttribute("type","button"),j.push(p(J,"click",function(){let Y=n.cancelPending;return typeof Y==="function"?Y.bind(void 0):()=>{}}(),{modifiers:[]}));let X=document.createTextNode("Cancel");J.appendChild(X),D.appendChild(J);let W=document.createElement("button");W.setAttribute("type","button"),W.setAttribute("data-primary",""),j.push(p(W,"click",function(){let Y=n.sendNote;return typeof Y==="function"?Y.bind(void 0):()=>{}}(),{modifiers:[]}));let Q=document.createTextNode("Send");return W.appendChild(Q),D.appendChild(W),g.appendChild(D),z.appendChild(g),{nodes:Array.from(z.childNodes),cleanup:()=>{for(let Y of j)try{Y()}catch(O){}}}}]}),y=d.shadowRoot??d;q(y);let l=document.createElement("div");l.setAttribute("class","surface"),l.setAttribute("part","surface");let E=document.createElement("slot");l.appendChild(E);let t=document.createElement("div");t.setAttribute("class","overlay"),t.setAttribute("part","overlay"),F(n,"overlayEl",t);let f=document.createComment("if");t.appendChild(f),m.push(w(f));let L=document.createComment("for");t.appendChild(L),m.push(r(L));let H=document.createComment("if");if(t.appendChild(H),m.push(M(H)),l.appendChild(t),y.appendChild(l),b!=null){let C=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(n,{get(j,g){return j[g]},set(j,g,Z){let D=j[g];if(D!=null&&typeof D.set==="function"){if(typeof Z==="function")D.set(()=>Z);else D.set(Z);return!0}return!1}});if(C.mount){let j=C.mount(z,d);if(typeof j==="function")m.push(j)}if(C.unmount)m.push(()=>C.unmount(z))}}return n}});var L0=N({mount(i,u){let d=i;return U(()=>{let n=d.pinEl(),b=d.boxEl(),w=d.x(),r=d.y(),M=d.index(),y=d.rect();if(n)n.style.left=`${w}px`,n.style.top=`${r}px`,n.textContent=String(M+1);if(b&&y){let l=y.split(",").map(Number);if(l.length===4&&l.every((E)=>Number.isFinite(E))){let[E,t,f,L]=l;b.style.cssText=`left:${E}px;top:${t}px;width:${f}px;height:${L}px`}}})}});var PM=V("sui-annotation-marker",{props:{kind:P.union(["point","region"],{default:"point",reflect:!0}),x:P.number({default:0}),y:P.number({default:0}),rect:P.string({default:""}),active:P.bool({default:!1,reflect:!0}),index:P.number({default:0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.kind=u.kind,n.x=u.x,n.y=u.y,n.rect=u.rect,n.active=u.active,n.index=u.index,n.boxEl=G(null),n.pinEl=G(null);let b=null;if(L0&&typeof L0.setup==="function"){if(i.scope=n,b=L0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("part","pin"),r.setAttribute("role","button"),r.setAttribute("tabindex","0"),F(n,"pinEl",r),w.appendChild(r);let M=document.createElement("div");if(M.setAttribute("part","box"),M.setAttribute("role","button"),M.setAttribute("tabindex","0"),F(n,"boxEl",M),w.appendChild(M),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let l=new Proxy(n,{get(E,t){return E[t]},set(E,t,f){let L=E[t];if(L!=null&&typeof L.set==="function"){if(typeof f==="function")L.set(()=>f);else L.set(f);return!0}return!1}});if(y.mount){let E=y.mount(l,d);if(typeof E==="function")m.push(E)}if(y.unmount)m.push(()=>y.unmount(l))}}return n}});var l0=N({scope:(i,u)=>{return{staged:G([]),emitRemove:(n)=>{u.dispatchEvent(new CustomEvent("sui-annotation-remove",{detail:{id:n},bubbles:!0,composed:!0}))}}},mount(i,u){let d=u.staged;if(d!=null)i.staged.set(Array.isArray(d)?d:[]);Object.defineProperty(u,"staged",{configurable:!0,get:()=>i.staged(),set:(m)=>i.staged.set(Array.isArray(m)?m:[])})}});var GM=V("sui-annotation-list",{shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null),b=null;if(l0&&typeof l0.setup==="function"){if(i.scope=n,b=l0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>o({anchor:l,items:()=>n.staged(),key:(E,t)=>{let f=Object.create(n);return f.a=E,f.__i=t,((L)=>E.id)(f)},render:(E,t)=>{let f=Object.create(n);return f.a=E,f.__i=t,f.__v=G(0),((L)=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("li");z.setAttribute("part","item");{let W=L.a.id;if(W==null||W===!1||W==="")z.removeAttribute("data-id");else z.setAttribute("data-id",W===!0?"":String(W))}let j=document.createElement("span");j.setAttribute("class","kind");let g=document.createTextNode("");j.appendChild(g),g.textContent=L.a.kind,z.appendChild(j);let Z=document.createElement("span");Z.setAttribute("class","note"),C.push(T(Z,"title",()=>(L.__v(),L.a.note??"")));let D=document.createTextNode("");Z.appendChild(D),C.push(k(D,()=>(L.__v(),L.a.note??"(no note)"))),z.appendChild(Z);let J=document.createElement("button");J.setAttribute("type","button"),J.setAttribute("aria-label","Remove annotation"),C.push(p(J,"click",function(){let W=()=>L.emitRemove(L.a.id);return typeof W==="function"?W.bind(void 0):()=>{}}(),{modifiers:[]}));let X=document.createTextNode("Remove");return J.appendChild(X),z.appendChild(J),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let W of C)try{W()}catch(Q){}},update:(W,Q)=>{L.a=W,L.__i=Q;{let Y=L.a.id;if(Y==null||Y===!1||Y==="")z.removeAttribute("data-id");else z.setAttribute("data-id",Y===!0?"":String(Y))}g.textContent=L.a.kind,L.__v.set((Y)=>Y+1)}}})(f)}}),r=(l)=>I({anchor:l,test:()=>{if(n.staged().length===0)return 0;return 1},branches:[()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("div");f.setAttribute("class","empty"),f.setAttribute("part","empty");let L=document.createTextNode("No staged annotations.");return f.appendChild(L),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let H of t)try{H()}catch(C){}}}},()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("ol");f.setAttribute("part","list");let L=document.createComment("for");return f.appendChild(L),t.push(w(L)),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let H of t)try{H()}catch(C){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createComment("if");if(M.appendChild(y),m.push(r(y)),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var E0=N({mount(i,u){let d=i,m=U(()=>{let n=d.inputEl();if(!n)return;let b=()=>{if(d.disabled())return;u.dispatchEvent(new CustomEvent("sui-radio-select",{detail:{value:d.value()},bubbles:!0,composed:!0}))},w=(r)=>{if(!["ArrowDown","ArrowRight","ArrowUp","ArrowLeft"].includes(r.key))return;r.preventDefault();let y=r.key==="ArrowDown"||r.key==="ArrowRight";u.dispatchEvent(new CustomEvent("sui-radio-nav",{detail:{direction:y?"next":"prev"},bubbles:!0,composed:!0}))};return n.addEventListener("click",b),n.addEventListener("keydown",w),()=>{n.removeEventListener("click",b),n.removeEventListener("keydown",w)}});return u.focusInput=()=>{u.shadowRoot?.querySelector("input")?.focus()},()=>m()}});var YM=V("sui-radio",{props:{value:P.string({default:"",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),checked:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.disabled=u.disabled,n.checked=u.checked,n.tone=u.tone,n.size=u.size,n.inputEl=G(null);let b=null;if(E0&&typeof E0.setup==="function"){if(i.scope=n,b=E0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("label");r.setAttribute("part","label");let M=document.createElement("span");M.setAttribute("class","circle"),M.setAttribute("part","circle");let y=document.createElement("input");y.setAttribute("part","input"),y.setAttribute("type","radio"),m.push(Dn(y,n.checked)),m.push(T(y,"disabled",()=>n.disabled())),m.push(T(y,"aria-checked",()=>n.checked()?"true":"false")),m.push(T(y,"tabindex",()=>n.checked()?"0":"-1")),F(n,"inputEl",y),M.appendChild(y);let l=document.createElement("span");l.setAttribute("class","dot"),l.setAttribute("part","dot"),l.setAttribute("aria-hidden","true"),M.appendChild(l),r.appendChild(M);let E=document.createElement("slot");if(r.appendChild(E),w.appendChild(r),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let f=new Proxy(n,{get(L,H){return L[H]},set(L,H,C){let z=L[H];if(z!=null&&typeof z.set==="function"){if(typeof C==="function")z.set(()=>C);else z.set(C);return!0}return!1}});if(t.mount){let L=t.mount(f,d);if(typeof L==="function")m.push(L)}if(t.unmount)m.push(()=>t.unmount(f))}}return n}});var r0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","radiogroup");let m=document.createElement("input");m.type="hidden",u.appendChild(m);let n=()=>Array.from(u.querySelectorAll(":scope > sui-radio")),b=()=>n().filter((E)=>!E.hasAttribute("disabled")&&!d.disabled()),w=()=>{let E=d.value(),t=d.disabled(),f=d.tone(),L=d.size();for(let H of n()){if(H.toggleAttribute("checked",H.getAttribute("value")===E),t)H.setAttribute("disabled","");if(f&&!H.hasAttribute("tone"))H.setAttribute("tone",f);if(L&&!H.hasAttribute("size"))H.setAttribute("size",L)}m.name=d.name(),m.value=E},r=U(()=>{d.value(),d.disabled(),d.tone(),d.size(),w()}),M=U(()=>{let E=d.label();if(E)u.setAttribute("aria-label",E);else u.removeAttribute("aria-label")}),y=(E)=>{let t=E;t.stopPropagation();let f=t.detail.value;if(f===d.value())return;d.value.set(f),w(),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:f},bubbles:!0,composed:!0}))},l=(E)=>{let t=E;t.stopPropagation();let f=b();if(!f.length)return;let L=t.composedPath().find((j)=>j.tagName?.toLowerCase()==="sui-radio"),H=L?f.indexOf(L):-1,C=t.detail.direction==="next"?f[(H+1)%f.length]:f[(H-1+f.length)%f.length],z=C.getAttribute("value")??"";d.value.set(z),w(),C.focusInput?.(),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:z},bubbles:!0,composed:!0}))};return u.addEventListener("sui-radio-select",y),u.addEventListener("sui-radio-nav",l),()=>{r(),M(),u.removeEventListener("sui-radio-select",y),u.removeEventListener("sui-radio-nav",l),m.remove()}}});var KM=V("sui-radio-group",{props:{value:P.string({default:"",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),name:P.string({default:""}),label:P.string({default:""})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    gap: var(--space-2, 8px);
    font-family: var(--font-body);
    color: var(--foreground);
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.disabled=u.disabled,n.tone=u.tone,n.size=u.size,n.name=u.name,n.label=u.label;let b=null;if(r0&&typeof r0.setup==="function"){if(i.scope=n,b=r0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");if(w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var M0=N({mount(i,u){let d=i,m=document.createElement("input");m.type="hidden",u.appendChild(m);let n=(f)=>{let L=d.step(),H=d.min(),C=d.max(),z=Math.round((f-H)/L)*L+H;return Math.max(H,Math.min(C,z))},b=(f)=>{let L=n(f);if(L===d.value())return;d.value.set(L),m.name=d.name(),m.value=String(L),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:L},bubbles:!0,composed:!0}))},w=(f)=>{let L=d.trackEl();if(!L)return d.value();let H=L.getBoundingClientRect(),C=Math.max(0,Math.min(1,(f-H.left)/H.width));return d.min()+C*(d.max()-d.min())},r=U(()=>{let f=d.value(),L=d.min(),C=d.max()-L||1,z=(f-L)/C*100,j=d.fillEl(),g=d.thumbEl();if(j)j.style.width=`${z}%`;if(g)g.style.left=`${z}%`;m.name=d.name(),m.value=String(f)}),M=!1,y=(f)=>{if(d.disabled())return;f.preventDefault(),M=!0,b(w(f.clientX)),d.thumbEl()?.focus();let H=(z)=>{if(M)b(w(z.clientX))},C=()=>{M=!1,document.removeEventListener("pointermove",H),document.removeEventListener("pointerup",C)};document.addEventListener("pointermove",H),document.addEventListener("pointerup",C)},l=(f)=>{if(d.disabled())return;let L=d.step(),H=!0;switch(f.key){case"ArrowRight":case"ArrowUp":b(d.value()+L);break;case"ArrowLeft":case"ArrowDown":b(d.value()-L);break;case"Home":b(d.min());break;case"End":b(d.max());break;case"PageUp":b(d.value()+L*10);break;case"PageDown":b(d.value()-L*10);break;default:H=!1}if(H)f.preventDefault()},E=U(()=>{let f=d.containerEl();if(!f)return;return f.addEventListener("pointerdown",y),()=>f.removeEventListener("pointerdown",y)}),t=U(()=>{let f=d.thumbEl();if(!f)return;return f.addEventListener("keydown",l),()=>f.removeEventListener("keydown",l)});return()=>{r(),E(),t(),m.remove()}}});var aM=V("sui-slider",{props:{value:P.number({default:0,reflect:!0}),min:P.number({default:0}),max:P.number({default:100}),step:P.number({default:1}),disabled:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),name:P.string({default:""}),label:P.string({default:"slider"})},shadow:!0,styles:`
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

`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.min=u.min,n.max=u.max,n.step=u.step,n.disabled=u.disabled,n.tone=u.tone,n.size=u.size,n.name=u.name,n.label=u.label,n.containerEl=G(null),n.fillEl=G(null),n.thumbEl=G(null),n.trackEl=G(null);let b=null;if(M0&&typeof M0.setup==="function"){if(i.scope=n,b=M0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","container"),F(n,"containerEl",r);let M=document.createElement("div");M.setAttribute("class","track"),M.setAttribute("part","track"),F(n,"trackEl",M);let y=document.createElement("div");y.setAttribute("class","fill"),y.setAttribute("part","fill"),F(n,"fillEl",y),M.appendChild(y),r.appendChild(M);let l=document.createElement("div");if(l.setAttribute("class","thumb"),l.setAttribute("part","thumb"),l.setAttribute("role","slider"),m.push(T(l,"tabindex",()=>n.disabled()?"-1":"0")),m.push(T(l,"aria-valuemin",()=>n.min())),m.push(T(l,"aria-valuemax",()=>n.max())),m.push(T(l,"aria-valuenow",()=>n.value())),m.push(T(l,"aria-label",()=>n.label()||"slider")),m.push(T(l,"aria-disabled",()=>n.disabled()?"true":"false")),F(n,"thumbEl",l),r.appendChild(l),w.appendChild(r),b!=null){let E=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(E){let t=new Proxy(n,{get(f,L){return f[L]},set(f,L,H){let C=f[L];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(E.mount){let f=E.mount(t,d);if(typeof f==="function")m.push(f)}if(E.unmount)m.push(()=>E.unmount(t))}}return n}});var hM=V("sui-segmented-option",{props:{value:P.string({default:"",reflect:!0}),icon:P.string({default:"",reflect:!0}),disabled:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.icon=u.icon,n.disabled=u.disabled;let b=d.shadowRoot??d;q(b);let w=document.createTextNode(`

`);b.appendChild(w);let r=document.createTextNode(`
`);b.appendChild(r);let M=document.createTextNode(`
`);b.appendChild(M);let y=document.createTextNode(`

`);b.appendChild(y);let l=document.createTextNode(`
`);return b.appendChild(l),n}});var cm=(i)=>typeof CSS<"u"&&CSS.escape?CSS.escape(i):i.replace(/[^\w-]/g,"\\$&"),y0=N({scope:()=>{let i=G([]),u=e(null);return{options:i,containerEl:u}},mount(i,u){if(!u.hasAttribute("role"))u.setAttribute("role","radiogroup");let d=document.createElement("input");d.type="hidden",u.appendChild(d);let m=()=>{return Array.from(u.querySelectorAll(":scope > sui-segmented-option")).map((E)=>({value:E.getAttribute("value")??"",label:E.textContent?.trim()??"",icon:E.getAttribute("icon")||null,disabled:E.hasAttribute("disabled")}))},n=(l)=>{if(i.disabled())return;let t=i.options().find((f)=>f.value===l);if(!t||t.disabled)return;if(i.value()===l)return;i.value.set(l),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:l},bubbles:!0,composed:!0}))},b=(l)=>{let E=l.target?.closest(".btn");if(!E)return;let t=E.getAttribute("data-value");if(t!==null)n(t)},w=(l)=>{let t=i.options().filter((j)=>!j.disabled&&!i.disabled());if(!t.length)return;let f=t.map((j)=>j.value),L=i.value(),H=Math.max(0,f.indexOf(L)),C=H;if(l.key==="ArrowRight"||l.key==="ArrowDown")l.preventDefault(),C=(H+1)%f.length;else if(l.key==="ArrowLeft"||l.key==="ArrowUp")l.preventDefault(),C=(H-1+f.length)%f.length;else if(l.key==="Enter"||l.key===" "){l.preventDefault();let g=u.shadowRoot.activeElement?.getAttribute("data-value");if(g)n(g);return}else return;let z=f[C]??"";n(z),queueMicrotask(()=>{u.shadowRoot?.querySelector(`.btn[data-value="${cm(z)}"]`)?.focus()})};i.options.set(m());let r=new MutationObserver(()=>i.options.set(m()));r.observe(u,{childList:!0,subtree:!1});let M=U(()=>{d.name=i.name(),d.value=i.value()}),y=U(()=>{let l=i.containerEl();if(!l)return;return l.addEventListener("click",b),l.addEventListener("keydown",w),()=>{l.removeEventListener("click",b),l.removeEventListener("keydown",w)}});return()=>{r.disconnect(),M(),y(),d.remove()}}});var cM=V("sui-segmented",{props:{value:P.string({default:"",reflect:!0}),size:P.union(["sm","md"],{default:"md",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),name:P.string({default:""})},shadow:!0,styles:`
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

`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.size=u.size,n.disabled=u.disabled,n.name=u.name,n.containerEl=G(null);let b=null;if(y0&&typeof y0.setup==="function"){if(i.scope=n,b=y0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>o({anchor:l,items:()=>n.options(),key:(E,t)=>{let f=Object.create(n);return f.opt=E,f.__i=t,((L)=>E.value)(f)},render:(E,t)=>{let f=Object.create(n);return f.opt=E,f.__i=t,f.__v=G(0),((L)=>{let H=(D)=>I({anchor:D,test:()=>{if(L.opt.icon)return 0;return-1},branches:[()=>{let J=document.createDocumentFragment(),X=[],W=document.createElement("span");W.setAttribute("class","btn-icon"),W.setAttribute("aria-hidden","true");let Q=document.createElement("sui-icon");return Q.name=L.opt.icon,Q.setAttribute("size","sm"),W.appendChild(Q),J.appendChild(W),{nodes:Array.from(J.childNodes),cleanup:()=>{for(let Y of X)try{Y()}catch(O){}}}}]}),C=document.createDocumentFragment(),z=[],j=document.createElement("button");j.setAttribute("class","btn"),j.setAttribute("role","radio"),j.setAttribute("type","button");{let D=L.opt.value;if(D==null||D===!1||D==="")j.removeAttribute("data-value");else j.setAttribute("data-value",D===!0?"":String(D))}z.push(T(j,"aria-checked",()=>(L.__v(),L.opt.value===L.value()?"true":"false"))),z.push(T(j,"aria-disabled",()=>(L.__v(),L.disabled()||L.opt.disabled?"true":"false"))),z.push(Wn(j,"disabled",()=>(L.__v(),L.disabled()||L.opt.disabled))),z.push(T(j,"tabindex",()=>(L.__v(),L.opt.value===L.value()?"0":"-1")));let g=document.createComment("if");j.appendChild(g),z.push(H(g));let Z=document.createTextNode("");return j.appendChild(Z),Z.textContent=L.opt.label,C.appendChild(j),{nodes:Array.from(C.childNodes),cleanup:()=>{for(let D of z)try{D()}catch(J){}},update:(D,J)=>{L.opt=D,L.__i=J;{let X=L.opt.value;if(X==null||X===!1||X==="")j.removeAttribute("data-value");else j.setAttribute("data-value",X===!0?"":String(X))}__e11.name=L.opt.icon,Z.textContent=L.opt.label,L.__v.set((X)=>X+1)}}})(f)}}),r=d.shadowRoot??d;q(r);let M=document.createElement("div");M.setAttribute("class","container"),M.setAttribute("part","container"),F(n,"containerEl",M);let y=document.createComment("for");if(M.appendChild(y),m.push(w(y)),r.appendChild(M),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var w0=N({mount(i,u){let d=i,m=U(()=>{let n=d.buttonEl();if(!n)return;let b=()=>{if(d.disabled())return;d.pressed.set(!d.pressed()),u.dispatchEvent(new CustomEvent("sui-toggle",{detail:{pressed:d.pressed(),value:d.value()},bubbles:!0,composed:!0}))};return n.addEventListener("click",b),()=>n.removeEventListener("click",b)});return u.focusButton=()=>{u.shadowRoot?.querySelector("button")?.focus()},()=>m()}});var my=V("sui-toggle",{props:{pressed:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),value:P.string({default:"",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:P.union(["outline","ghost"],{default:"outline",reflect:!0})},shadow:!0,styles:`
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

`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.pressed=u.pressed,n.disabled=u.disabled,n.value=u.value,n.size=u.size,n.variant=u.variant,n.buttonEl=G(null);let b=null;if(w0&&typeof w0.setup==="function"){if(i.scope=n,b=w0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("button");r.setAttribute("part","button"),r.setAttribute("type","button"),m.push(T(r,"aria-pressed",()=>n.pressed()?"true":"false")),m.push(T(r,"disabled",()=>n.disabled())),F(n,"buttonEl",r);let M=document.createElement("slot");if(r.appendChild(M),w.appendChild(r),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let l=new Proxy(n,{get(E,t){return E[t]},set(E,t,f){let L=E[t];if(L!=null&&typeof L.set==="function"){if(typeof f==="function")L.set(()=>f);else L.set(f);return!0}return!1}});if(y.mount){let E=y.mount(l,d);if(typeof E==="function")m.push(E)}if(y.unmount)m.push(()=>y.unmount(l))}}return n}});var H0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","group");let m=document.createElement("input");m.type="hidden",u.appendChild(m);let n=()=>{let L=d.defaultSlotEl();if(!L)return Array.from(u.querySelectorAll(":scope > sui-toggle"));return L.assignedElements().filter((H)=>H.tagName.toLowerCase()==="sui-toggle")},b=()=>n().filter((L)=>!L.hasAttribute("disabled")),w=()=>new Set(d.value().split(",").filter(Boolean)),r=(L)=>{let H=Array.from(L).join(",");d.value.set(H),M(),m.name=d.name(),m.value=H,u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:H},bubbles:!0,composed:!0}))},M=()=>{let L=w(),H=d.size(),C=d.variant();for(let z of n()){let j=z.getAttribute("value")??"";if(z.toggleAttribute("pressed",L.has(j)),H)z.setAttribute("size",H);if(C)z.setAttribute("variant",C)}},y=(L)=>{let H=L;H.stopPropagation();let C=H.composedPath().find((g)=>g.tagName?.toLowerCase()==="sui-toggle");if(!C)return;let z=C.getAttribute("value")??"",j=w();if(d.type()==="single")if(j.has(z))r(new Set);else r(new Set([z]));else{if(j.has(z))j.delete(z);else j.add(z);r(j)}},l=(L)=>{if(L.key!=="ArrowRight"&&L.key!=="ArrowLeft")return;let H=b();if(!H.length)return;let C=L.composedPath().find((g)=>H.includes(g)),z=C?H.indexOf(C):-1,j=L.key==="ArrowRight"?(z+1)%H.length:(z-1+H.length)%H.length;H[j]?.focusButton?.(),L.preventDefault()},E=U(()=>{d.value(),d.size(),d.variant(),M()}),t=U(()=>{let L=d.defaultSlotEl();if(!L)return;let H=()=>M();return L.addEventListener("slotchange",H),()=>L.removeEventListener("slotchange",H)}),f=U(()=>{m.name=d.name(),m.value=d.value()});return u.addEventListener("sui-toggle",y),u.addEventListener("keydown",l),()=>{E(),t(),f(),u.removeEventListener("sui-toggle",y),u.removeEventListener("keydown",l),m.remove()}}});var Ey=V("sui-toggle-group",{props:{type:P.union(["single","multiple"],{default:"single",reflect:!0}),value:P.string({default:"",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:P.union(["outline","ghost"],{default:"outline",reflect:!0}),name:P.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    gap: 0;
  }

  ::slotted(sui-toggle:not(:first-child)) { margin-left: -1px; }
  ::slotted(sui-toggle:first-child) { border-radius: var(--radius, 6px) 0 0 var(--radius, 6px); }
  ::slotted(sui-toggle:last-child) { border-radius: 0 var(--radius, 6px) var(--radius, 6px) 0; }
  ::slotted(sui-toggle:not(:first-child):not(:last-child)) { border-radius: 0; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.type=u.type,n.value=u.value,n.size=u.size,n.variant=u.variant,n.name=u.name,n.defaultSlotEl=G(null);let b=null;if(H0&&typeof H0.setup==="function"){if(i.scope=n,b=H0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");if(F(n,"defaultSlotEl",r),w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var yy=V("sui-code",{props:{lang:P.string({default:"",reflect:!0}),block:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.lang=u.lang,n.block=u.block;let b=(y)=>I({anchor:y,test:()=>{if(n.lang())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),E=[],t=document.createElement("header");t.setAttribute("part","label");let f=document.createTextNode("");return t.appendChild(f),E.push(k(f,()=>n.lang())),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let L of E)try{L()}catch(H){}}}}]}),w=(y)=>I({anchor:y,test:()=>{if(n.block())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),E=[],t=document.createElement("div");t.setAttribute("class","block"),t.setAttribute("part","block"),t.setAttribute("role","region"),E.push(T(t,"aria-label",()=>n.lang()?n.lang()+" code block":"code block"));let f=document.createComment("if");t.appendChild(f),E.push(b(f));let L=document.createElement("pre");L.setAttribute("part","pre");let H=document.createElement("slot");return L.appendChild(H),t.appendChild(L),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let C of E)try{C()}catch(z){}}}},()=>{let l=document.createDocumentFragment(),E=[],t=document.createElement("code");t.setAttribute("class","inline"),t.setAttribute("part","inline");let f=document.createElement("slot");return t.appendChild(f),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let L of E)try{L()}catch(H){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createComment("if");return r.appendChild(M),m.push(w(M)),n}});var C0=N({mount(i,u){let d=i,m,n=async()=>{try{await navigator.clipboard.writeText(d.value())}catch{let w=document.createElement("textarea");w.value=d.value(),document.body.appendChild(w),w.select();try{document.execCommand("copy")}catch{}w.remove()}if(d.copied.set(!0),u.dispatchEvent(new CustomEvent("sui-copy",{detail:{value:d.value()},bubbles:!0,composed:!0})),m)clearTimeout(m);m=setTimeout(()=>{d.copied.set(!1)},1200)},b=U(()=>{let w=d.buttonEl();if(!w)return;return w.addEventListener("click",n),()=>w.removeEventListener("click",n)});return()=>{if(b(),m)clearTimeout(m)}}});var Py=V("sui-copy-button",{props:{value:P.string({default:"",reflect:!0}),label:P.string({default:"Copy",reflect:!0}),copied:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.label=u.label,n.copied=u.copied,n.buttonEl=G(null);let b=null;if(C0&&typeof C0.setup==="function"){if(i.scope=n,b=C0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(f)=>I({anchor:f,test:()=>{if(n.copied())return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("sui-icon");return C.setAttribute("name","check"),C.setAttribute("size","sm"),C.setAttribute("aria-hidden","true"),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let z of H)try{z()}catch(j){}}}},()=>{let L=document.createDocumentFragment(),H=[],C=document.createElement("sui-icon");return C.setAttribute("name","copy"),C.setAttribute("size","sm"),C.setAttribute("aria-hidden","true"),L.appendChild(C),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let z of H)try{z()}catch(j){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createElement("button");M.setAttribute("part","button"),M.setAttribute("type","button"),m.push(T(M,"aria-label",()=>n.label())),F(n,"buttonEl",M);let y=document.createComment("if");M.appendChild(y),m.push(w(y));let l=document.createElement("slot");m.push(bn(l,d,"data-empty")),M.appendChild(l);let E=document.createElement("span");E.setAttribute("class","default-label");let t=document.createTextNode("");if(E.appendChild(t),m.push(k(t,()=>n.copied()?"Copied":n.label())),M.appendChild(E),r.appendChild(M),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let L=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let j=H[C];if(j!=null&&typeof j.set==="function"){if(typeof z==="function")j.set(()=>z);else j.set(z);return!0}return!1}});if(f.mount){let H=f.mount(L,d);if(typeof H==="function")m.push(H)}if(f.unmount)m.push(()=>f.unmount(L))}}return n}});function Tu(i){return i.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function Zd(i,u,d,m,n){let b=u!==null?`<span class="key">"${Tu(u)}"</span>: `:"";if(i===null)return`${b}<span class="null">null</span>`;if(typeof i==="string")return`${b}<span class="s">"${Tu(i)}"</span>`;if(typeof i==="number")return`${b}<span class="n">${i}</span>`;if(typeof i==="boolean")return`${b}<span class="b">${String(i)}</span>`;let w=Array.isArray(i),r=w?i.map((f,L)=>[String(L),f]):Object.entries(i),M=m||d<n,y=w?"[":"{",l=w?"]":"}",E=r.length;if(E===0)return`${b}${y}${l}`;let t=r.map(([f,L])=>`<div class="node">${Zd(L,w?null:f,d+1,m,n)}</div>`).join("");return`<details${M?" open":""}>
<summary><span class="caret">▶</span>${b}${y}<span class="meta">${E} ${w?"items":"keys"}</span></summary>
<div class="branch">${t}</div>
${l}</details>`}var z0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","tree");if(!u.hasAttribute("aria-label"))u.setAttribute("aria-label","JSON tree");let m=d.contentEl();if(!m)return()=>{};return fn(m,()=>{let n=d.data(),b=d.expanded(),w=d.depth();if(!n)return"";let r;try{r=JSON.parse(n)}catch(M){return`<span class="error" part="error" role="alert">Invalid JSON: ${Tu(M.message)}</span>`}return`<div part="root">${Zd(r,null,0,b,w)}</div>`})}});var Gy=V("sui-json-view",{props:{data:P.string({default:"",reflect:!0}),expanded:P.bool({default:!1,reflect:!0}),depth:P.number({default:1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.data=u.data,n.expanded=u.expanded,n.depth=u.depth,n.contentEl=G(null);let b=null;if(z0&&typeof z0.setup==="function"){if(i.scope=n,b=z0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");if(r.setAttribute("class","json-root"),F(n,"contentEl",r),w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});function Jd(i){return i.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function _m(i,u){let d=i.split(`
`),m=u.split(`
`),n=d.length,b=m.length,w=Array.from({length:n+1},()=>Array(b+1).fill(0));for(let l=n-1;l>=0;l--)for(let E=b-1;E>=0;E--)w[l][E]=d[l]===m[E]?w[l+1][E+1]+1:Math.max(w[l+1][E],w[l][E+1]);let r=[],M=0,y=0;while(M<n&&y<b)if(d[M]===m[y])r.push({kind:"ctx",text:d[M]}),M++,y++;else if(w[M+1][y]>=w[M][y+1])r.push({kind:"del",text:d[M]}),M++;else r.push({kind:"add",text:m[y]}),y++;while(M<n)r.push({kind:"del",text:d[M++]});while(y<b)r.push({kind:"add",text:m[y++]});return r}function nb(i){return i.split(`
`).map((u)=>{if(u.startsWith("@@"))return{kind:"hunk",text:u};if(u.startsWith("+"))return{kind:"add",text:u.slice(1)};if(u.startsWith("-"))return{kind:"del",text:u.slice(1)};return{kind:"ctx",text:u.startsWith(" ")?u.slice(1):u}})}function ib(i){return i.map((u)=>{let d=u.kind==="add"?"+":u.kind==="del"?"-":u.kind==="hunk"?"@":" ",m=u.kind==="add"?"added":u.kind==="del"?"removed":u.kind==="hunk"?"hunk":"context";return`<div class="line ${u.kind}" part="line" role="listitem" aria-label="${m}: ${Jd(u.text)}"><span class="sign" aria-hidden="true">${d}</span><span>${Jd(u.text)}</span></div>`}).join("")}var j0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","region");if(!u.hasAttribute("aria-label"))u.setAttribute("aria-label","diff");let m=d.contentEl();if(!m)return()=>{};return fn(m,()=>{let n=d.value(),b=d.from(),w=d.to(),r;if(n)r=nb(n);else if(b||w)r=_m(b,w);else r=[];return ib(r)})}});var Yy=V("sui-diff",{props:{from:P.string({default:"",reflect:!0}),to:P.string({default:"",reflect:!0}),value:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.from=u.from,n.to=u.to,n.value=u.value,n.contentEl=G(null);let b=null;if(j0&&typeof j0.setup==="function"){if(i.scope=n,b=j0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");if(r.setAttribute("class","diff-root"),F(n,"contentEl",r),w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});function Qu(i){return i.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function $n(i){let u=Qu(i);return u=u.replace(/`([^`]+)`/g,(d,m)=>`<code>${m}</code>`),u=u.replace(/\*\*([^*]+)\*\*/g,"<strong>$1</strong>"),u=u.replace(/(^|[^*])\*([^*\n]+)\*/g,"$1<em>$2</em>"),u=u.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g,(d,m,n)=>`<a href="${n}" rel="noopener noreferrer">${m}</a>`),u}function ub(i){let u=i.replace(/\r\n/g,`
`).split(`
`),d=[],m=0;while(m<u.length){let n=u[m];if(/^```/.test(n)){let r=n.slice(3).trim(),M=[];m++;while(m<u.length&&!/^```/.test(u[m]))M.push(u[m++]);m++,d.push(`<pre data-lang="${Qu(r)}"><code>${Qu(M.join(`
`))}</code></pre>`);continue}let b=/^(#{1,6})\s+(.*)$/.exec(n);if(b){d.push(`<h${b[1].length}>${$n(b[2])}</h${b[1].length}>`),m++;continue}if(/^\s*[-*]\s+/.test(n)){let r=[];while(m<u.length&&/^\s*[-*]\s+/.test(u[m]))r.push(`<li>${$n(u[m].replace(/^\s*[-*]\s+/,""))}</li>`),m++;d.push(`<ul>${r.join("")}</ul>`);continue}if(/^\s*\d+\.\s+/.test(n)){let r=[];while(m<u.length&&/^\s*\d+\.\s+/.test(u[m]))r.push(`<li>${$n(u[m].replace(/^\s*\d+\.\s+/,""))}</li>`),m++;d.push(`<ol>${r.join("")}</ol>`);continue}if(/^>\s?/.test(n)){let r=[];while(m<u.length&&/^>\s?/.test(u[m]))r.push(u[m].replace(/^>\s?/,"")),m++;d.push(`<blockquote>${$n(r.join(" "))}</blockquote>`);continue}if(n.trim()===""){m++;continue}let w=[];while(m<u.length&&u[m].trim()!==""&&!/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(u[m]))w.push(u[m]),m++;d.push(`<p>${$n(w.join(" "))}</p>`)}return d.join(`
`)}var P0=N({mount(i){let u=i,d=u.contentEl();if(!d)return()=>{};return fn(d,()=>ub(u.value()??""))}});var Ky=V("sui-markdown",{props:{value:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.contentEl=G(null);let b=null;if(P0&&typeof P0.setup==="function"){if(i.scope=n,b=P0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");if(r.setAttribute("class","md-root"),F(n,"contentEl",r),w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var Xd={actions:"#view-topbar-actions","status-start":"#view-status-start","status-end":"#view-status-end"},g0={title:"#breadcrumbs-title",branch:"#status-branch"},D0=N({mount(i,u){let d=i,m={};function n(){for(let M of Object.keys(Xd)){let y=document.querySelector(Xd[M]);if(!y)continue;let l=Array.from(u.querySelectorAll(`[slot="${M}"]`));if(l.length===0)continue;for(let E of l)y.appendChild(E);m[M]=l}}function b(){let M=document.querySelector(g0.title);if(M)M.textContent="";if(d.branch()){let y=document.querySelector(g0.branch);if(y)y.textContent=""}if(d.ahead()>0||d.behind()>0){let y=document.querySelector("#status-ab");if(y)y.textContent=""}if(d.tone()!=="default"||d.busy()){let y=document.querySelector("#status-dot");if(y)delete y.dataset.tone,y.removeAttribute("data-busy")}}function w(){for(let M of Object.keys(m)){for(let y of m[M]??[])y.remove();delete m[M]}b()}let r=U(()=>{let M=d.title(),y=d.branch(),l=d.ahead(),E=d.behind(),t=d.tone(),f=d.busy();if(M){let L=document.querySelector(g0.title);if(L)L.textContent=M}if(y){let L=document.querySelector(g0.branch);if(L)L.textContent=y}if(l>0||E>0){let L=document.querySelector("#status-ab");if(L){let H=[];if(l>0)H.push(`↑${l}`);if(E>0)H.push(`↓${E}`);L.textContent=H.join(" ")}}if(t!=="default"||f){let L=document.querySelector("#status-dot");if(L)if(L.dataset.tone=t,f)L.setAttribute("data-busy","");else L.removeAttribute("data-busy")}});return queueMicrotask(()=>n()),()=>{r(),w()}}});var ay=V("sui-view-chrome",{props:{title:P.string({default:"",reflect:!0}),branch:P.string({default:"",reflect:!0}),ahead:P.number({default:0,reflect:!0}),behind:P.number({default:0,reflect:!0}),tone:P.union(["default","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),busy:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.title=u.title,n.branch=u.branch,n.ahead=u.ahead,n.behind=u.behind,n.tone=u.tone,n.busy=u.busy;let b=null;if(D0&&typeof D0.setup==="function"){if(i.scope=n,b=D0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createTextNode(`

`);w.appendChild(r);let M=document.createTextNode(`
`);w.appendChild(M);let y=document.createTextNode(`
`);w.appendChild(y);let l=document.createTextNode(`
`);w.appendChild(l);let E=document.createTextNode(`
`);w.appendChild(E);let t=document.createTextNode(`
`);w.appendChild(t);let f=document.createTextNode(`

`);w.appendChild(f);let L=document.createTextNode(`
`);if(w.appendChild(L),b!=null){let H=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(H){let C=new Proxy(n,{get(z,j){return z[j]},set(z,j,g){let Z=z[j];if(Z!=null&&typeof Z.set==="function"){if(typeof g==="function")Z.set(()=>g);else Z.set(g);return!0}return!1}});if(H.mount){let z=H.mount(C,d);if(typeof z==="function")m.push(z)}if(H.unmount)m.push(()=>H.unmount(C))}}return n}});var Z0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","navigation");let m=()=>Array.from(u.querySelectorAll(":scope > sui-navigation-item")),n=()=>m().filter((l)=>!l.hasAttribute("disabled")),b=()=>{let l=d.value();for(let E of m())E.toggleAttribute("active",E.getAttribute("value")===l)},w=(l)=>{let E=l;E.stopPropagation();let t=E.detail.value;d.value.set(t),b(),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:t},bubbles:!0,composed:!0}))},r=(l)=>{if(l.key!=="ArrowRight"&&l.key!=="ArrowLeft"&&l.key!=="Home"&&l.key!=="End")return;let E=n();if(!E.length)return;let t=l.composedPath().find((H)=>E.includes(H)),f=t?E.indexOf(t):-1,L;if(l.key==="ArrowRight")L=E[(f+1)%E.length];else if(l.key==="ArrowLeft")L=E[(f-1+E.length)%E.length];else if(l.key==="Home")L=E[0];else if(l.key==="End")L=E[E.length-1];if(L)l.preventDefault(),L.focusLink?.()},M=U(()=>{d.value(),b()}),y=U(()=>{let l=d.defaultSlotEl();if(!l)return;let E=()=>b();return l.addEventListener("slotchange",E),()=>l.removeEventListener("slotchange",E)});return u.addEventListener("sui-nav-select",w),u.addEventListener("keydown",r),()=>{M(),y(),u.removeEventListener("sui-nav-select",w),u.removeEventListener("keydown",r)}}});var sy=V("sui-navigation-menu",{props:{value:P.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    align-items: stretch;
    gap: 0;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-body);
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.defaultSlotEl=G(null);let b=null;if(Z0&&typeof Z0.setup==="function"){if(i.scope=n,b=Z0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");if(F(n,"defaultSlotEl",r),w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var J0=N({mount(i,u){let d=i,m=(n)=>{if(d.disabled())return;if(!d.href())n.preventDefault();u.dispatchEvent(new CustomEvent("sui-nav-select",{detail:{value:d.value()},bubbles:!0,composed:!0}))};return u.focusLink=()=>{u.shadowRoot?.querySelector(".link")?.focus()},u.addEventListener("click",m),()=>u.removeEventListener("click",m)}});var i7=V("sui-navigation-item",{props:{value:P.string({default:"",reflect:!0}),active:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),href:P.string({default:""})},shadow:!0,styles:`
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

`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.active=u.active,n.disabled=u.disabled,n.href=u.href;let b=null;if(J0&&typeof J0.setup==="function"){if(i.scope=n,b=J0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>I({anchor:l,test:()=>{if(n.href())return 0;return 1},branches:[()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("a");f.setAttribute("class","link"),f.setAttribute("part","link"),t.push(T(f,"href",()=>n.href())),t.push(T(f,"aria-current",()=>n.active()?"page":"false"));let L=document.createElement("slot");return f.appendChild(L),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let H of t)try{H()}catch(C){}}}},()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("button");f.setAttribute("class","link"),f.setAttribute("part","link"),f.setAttribute("type","button"),t.push(T(f,"aria-current",()=>n.active()?"true":"false")),t.push(T(f,"disabled",()=>n.disabled()));let L=document.createElement("slot");return f.appendChild(L),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let H of t)try{H()}catch(C){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createComment("if");r.appendChild(M),m.push(w(M));let y=document.createElement("div");if(y.setAttribute("class","indicator"),y.setAttribute("part","indicator"),y.setAttribute("aria-hidden","true"),r.appendChild(y),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var X0=N({mount(i,u){let d=i,m=null,n=()=>{let r=d.viewportEl();if(!r)return;let M=d.orientation(),y=M==="vertical"||M==="both",l=M==="horizontal"||M==="both",E=d.vThumb(),t=d.hThumb();if(y&&E){let f=r.clientHeight/r.scrollHeight;if(f<1){let L=Math.max(24,f*r.clientHeight),C=r.scrollTop/(r.scrollHeight-r.clientHeight)*(r.clientHeight-L);E.style.height=`${L}px`,E.style.top=`${C}px`,E.style.display=""}else E.style.display="none"}if(l&&t){let f=r.clientWidth/r.scrollWidth;if(f<1){let L=Math.max(24,f*r.clientWidth),C=r.scrollLeft/(r.scrollWidth-r.clientWidth)*(r.clientWidth-L);t.style.width=`${L}px`,t.style.left=`${C}px`,t.style.display=""}else t.style.display="none"}},b=()=>{n();let r=d.vScrollbar(),M=d.hScrollbar();if(r?.classList.add("active"),M?.classList.add("active"),m)clearTimeout(m);m=setTimeout(()=>{r?.classList.remove("active"),M?.classList.remove("active")},1200)},w=d.viewportEl();if(w)w.addEventListener("scroll",b,{passive:!0});return n(),()=>{if(m)clearTimeout(m);if(w)w.removeEventListener("scroll",b)}}});var t7=V("sui-scroll-area",{props:{orientation:P.union(["vertical","horizontal","both"],{default:"vertical",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.orientation=u.orientation,n.size=u.size,n.hScrollbar=G(null),n.hThumb=G(null),n.vScrollbar=G(null),n.vThumb=G(null),n.viewportEl=G(null);let b=null;if(X0&&typeof X0.setup==="function"){if(i.scope=n,b=X0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","viewport"),r.setAttribute("part","viewport"),r.setAttribute("tabindex","0"),F(n,"viewportEl",r);let M=document.createElement("slot");r.appendChild(M),w.appendChild(r);let y=document.createElement("div");y.setAttribute("class","scrollbar scrollbar-v"),y.setAttribute("part","scrollbar-v"),y.setAttribute("aria-hidden","true"),F(n,"vScrollbar",y);let l=document.createElement("div");l.setAttribute("class","thumb"),F(n,"vThumb",l),y.appendChild(l),w.appendChild(y);let E=document.createElement("div");E.setAttribute("class","scrollbar scrollbar-h"),E.setAttribute("part","scrollbar-h"),E.setAttribute("aria-hidden","true"),F(n,"hScrollbar",E);let t=document.createElement("div");if(t.setAttribute("class","thumb"),F(n,"hThumb",t),E.appendChild(t),w.appendChild(E),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let L=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let j=H[C];if(j!=null&&typeof j.set==="function"){if(typeof z==="function")j.set(()=>z);else j.set(z);return!0}return!1}});if(f.mount){let H=f.mount(L,d);if(typeof H==="function")m.push(H)}if(f.unmount)m.push(()=>f.unmount(L))}}return n}});var G0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","toolbar");if(!u.hasAttribute("aria-orientation"))u.setAttribute("aria-orientation","horizontal");let m=()=>{return(d.defaultSlot()?.assignedElements({flatten:!0})??[]).filter((r)=>!r.hasAttribute("disabled"))},n=(b)=>{if(b.key!=="ArrowRight"&&b.key!=="ArrowLeft")return;let w=m();if(!w.length)return;let r=b.composedPath().find((l)=>w.includes(l))||null,M=r?w.indexOf(r):-1,y=b.key==="ArrowRight"?(M+1)%w.length:(M-1+w.length)%w.length;w[y]?.focus(),b.preventDefault()};return u.addEventListener("keydown",n),()=>u.removeEventListener("keydown",n)}});var y7=V("sui-toolbar",{props:{justify:P.union(["start","center","end","between"],{default:"start",reflect:!0}),align:P.union(["center","start","end","baseline"],{default:"center",reflect:!0}),gap:P.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:P.union(["default","bordered"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.justify=u.justify,n.align=u.align,n.gap=u.gap,n.variant=u.variant,n.defaultSlot=G(null);let b=null;if(G0&&typeof G0.setup==="function"){if(i.scope=n,b=G0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");r.setAttribute("name","spacer"),w.appendChild(r);let M=document.createElement("slot");if(F(n,"defaultSlot",M),w.appendChild(M),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let l=new Proxy(n,{get(E,t){return E[t]},set(E,t,f){let L=E[t];if(L!=null&&typeof L.set==="function"){if(typeof f==="function")L.set(()=>f);else L.set(f);return!0}return!1}});if(y.mount){let E=y.mount(l,d);if(typeof E==="function")m.push(E)}if(y.unmount)m.push(()=>y.unmount(l))}}return n}});var Gd={dashboard:"layout-dashboard",pipelines:"workflow",agents:"bot",skills:"sparkles",git:"git-branch",collections:"database",canvas:"frame",settings:"settings",chat:"messages-square",events:"radio",specs:"file-text",theme:"palette",setup:"rocket",overview:"compass",mesh:"network"},Wd={"theme-studio":"theme"},db=new Set([...Object.keys(Gd),...Object.keys(Wd)]),W0=N({mount(i,u){let d=i,m=U(()=>{let n=d.name(),b=d.label(),w=Wd[n]??n,M=db.has(n)?Gd[w]:"dot",y=d.glyphEl();if(y)y.setAttribute("name",M);if(b)u.setAttribute("role","img"),u.setAttribute("aria-label",b),u.removeAttribute("aria-hidden");else u.setAttribute("aria-hidden","true"),u.removeAttribute("role"),u.removeAttribute("aria-label")});return()=>m()}});var P7=V("sui-app-icon",{props:{name:P.string({default:"",reflect:!0}),variant:P.union(["glyph","tile","solid"],{default:"glyph",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),radius:P.string({default:"",reflect:!0}),label:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.variant=u.variant,n.size=u.size,n.radius=u.radius,n.label=u.label,n.glyphEl=G(null);let b=null;if(W0&&typeof W0.setup==="function"){if(i.scope=n,b=W0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("span");r.setAttribute("class","frame"),r.setAttribute("part","frame");let M=document.createElement("slot"),y=document.createElement("sui-icon");if(F(n,"glyphEl",y),y.setAttribute("part","glyph"),y.setAttribute("aria-hidden","true"),M.appendChild(y),r.appendChild(M),w.appendChild(r),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var T0=N({scope:()=>{return{nodes:G([])}},mount(i,u){let d=u.nodes;if(d!=null)i.nodes.set(d);if(Object.defineProperty(u,"nodes",{configurable:!0,get:()=>i.nodes(),set:(m)=>i.nodes.set(m)}),!u.hasAttribute("role"))u.setAttribute("role","tree")}});var G7=V("sui-tree-view",{shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
  }
  ul { list-style: none; margin: 0; padding: 0; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null),b=null;if(T0&&typeof T0.setup==="function"){if(i.scope=n,b=T0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>o({anchor:l,items:()=>n.nodes(),key:(E,t)=>{let f=Object.create(n);return f.n=E,f.__i=t,((L)=>E.id)(f)},render:(E,t)=>{let f=Object.create(n);return f.n=E,f.__i=t,f.__v=G(0),((L)=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("sui-tree-item");return z.node=L.n,H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let j of C)try{j()}catch(g){}},update:(j,g)=>{L.n=j,L.__i=g,z.node=L.n,L.__v.set((Z)=>Z+1)}}})(f)}}),r=(l)=>I({anchor:l,test:()=>{if(n.nodes().length>0)return 0;return 1},branches:[()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("ul");f.setAttribute("role","tree");let L=document.createComment("for");return f.appendChild(L),t.push(w(L)),E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let H of t)try{H()}catch(C){}}}},()=>{let E=document.createDocumentFragment(),t=[],f=document.createElement("slot");return E.appendChild(f),{nodes:Array.from(E.childNodes),cleanup:()=>{for(let L of t)try{L()}catch(H){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createComment("if");if(M.appendChild(y),m.push(r(y)),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let E=new Proxy(n,{get(t,f){return t[f]},set(t,f,L){let H=t[f];if(H!=null&&typeof H.set==="function"){if(typeof L==="function")H.set(()=>L);else H.set(L);return!0}return!1}});if(l.mount){let t=l.mount(E,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(E))}}return n}});var Q0=N({scope:({label:i})=>{let u=G(null),d=h(()=>{let b=u();return b?b.label:i()}),m=h(()=>{return u()?.children??[]}),n=e(null);return{node:u,nodeLabel:d,nodeChildren:m,rowEl:n}},mount(i,u){if(!u.hasAttribute("role"))u.setAttribute("role","treeitem");let d=u.node;if(d!=null)i.node.set(d);Object.defineProperty(u,"node",{configurable:!0,get:()=>i.node(),set:(l)=>i.node.set(l)});let m=U(()=>{let l=i.node();if(l!==null)if(!l.children||l.children.length===0)u.setAttribute("leaf","");else u.removeAttribute("leaf")}),n=U(()=>{let l=i.rowEl();if(!l)return;if(!i.leaf())l.setAttribute("aria-expanded",String(i.expanded()));else l.removeAttribute("aria-expanded");l.setAttribute("aria-selected",String(i.selected()))}),b=(l)=>{if(i.leaf())return;l.stopPropagation();let E=!i.expanded();i.expanded.set(E),u.dispatchEvent(new CustomEvent("sui-toggle",{detail:{expanded:E},bubbles:!0,composed:!0}))},w=()=>{u.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},r=(l)=>{w(),b(l)},M=(l)=>{if(l.key==="Enter"||l.key===" ")l.preventDefault(),w();else if(l.key==="ArrowRight"&&!i.leaf()&&!i.expanded())l.preventDefault(),b(l);else if(l.key==="ArrowLeft"&&i.expanded())l.preventDefault(),b(l)},y=U(()=>{let l=i.rowEl();if(!l)return;return l.addEventListener("click",r),l.addEventListener("keydown",M),()=>{l.removeEventListener("click",r),l.removeEventListener("keydown",M)}});return()=>{m(),n(),y()}}});var Y7=V("sui-tree-item",{props:{label:P.string({default:"",reflect:!0}),expanded:P.bool({default:!1,reflect:!0}),selected:P.bool({default:!1,reflect:!0}),leaf:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.label=u.label,n.expanded=u.expanded,n.selected=u.selected,n.leaf=u.leaf,n.rowEl=G(null);let b=null;if(Q0&&typeof Q0.setup==="function"){if(i.scope=n,b=Q0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(Z)=>o({anchor:Z,items:()=>n.nodeChildren(),key:(D,J)=>{let X=Object.create(n);return X.c=D,X.__i=J,((W)=>D.id)(X)},render:(D,J)=>{let X=Object.create(n);return X.c=D,X.__i=J,X.__v=G(0),((W)=>{let Q=document.createDocumentFragment(),Y=[],O=document.createElement("sui-tree-item");return O.node=W.c,Q.appendChild(O),{nodes:Array.from(Q.childNodes),cleanup:()=>{for(let K of Y)try{K()}catch(B){}},update:(K,B)=>{W.c=K,W.__i=B,O.node=W.c,W.__v.set((A)=>A+1)}}})(X)}}),r=(Z)=>I({anchor:Z,test:()=>{if(n.nodeChildren().length>0)return 0;return 1},branches:[()=>{let D=document.createDocumentFragment(),J=[],X=document.createComment("for");return D.appendChild(X),J.push(w(X)),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let W of J)try{W()}catch(Q){}}}},()=>{let D=document.createDocumentFragment(),J=[],X=document.createElement("slot");return D.appendChild(X),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let W of J)try{W()}catch(Q){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createElement("div");y.setAttribute("class","row"),y.setAttribute("part","row"),y.setAttribute("tabindex","0"),F(n,"rowEl",y);let l=document.createElement("span");l.setAttribute("class","caret"),l.setAttribute("part","caret"),l.setAttribute("aria-hidden","true");let E=document.createElement("svg");E.setAttribute("viewbox","0 0 12 12"),E.setAttribute("fill","none"),E.setAttribute("stroke","currentColor"),E.setAttribute("stroke-width","1.5");let t=document.createElement("polyline");t.setAttribute("points","4 3 8 6 4 9"),E.appendChild(t),l.appendChild(E),y.appendChild(l);let f=document.createElement("slot");f.setAttribute("name","icon"),y.appendChild(f);let L=document.createElement("span");L.setAttribute("class","label"),L.setAttribute("part","label");let H=document.createTextNode("");L.appendChild(H),m.push(k(H,()=>n.nodeLabel())),y.appendChild(L);let C=document.createElement("span");C.setAttribute("class","trailing");let z=document.createElement("slot");z.setAttribute("name","trailing"),C.appendChild(z),y.appendChild(C),M.appendChild(y);let j=document.createElement("div");j.setAttribute("class","children"),j.setAttribute("part","children"),j.setAttribute("role","group");let g=document.createComment("if");if(j.appendChild(g),m.push(r(g)),M.appendChild(j),b!=null){let Z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(Z){let D=new Proxy(n,{get(J,X){return J[X]},set(J,X,W){let Q=J[X];if(Q!=null&&typeof Q.set==="function"){if(typeof W==="function")Q.set(()=>W);else Q.set(W);return!0}return!1}});if(Z.mount){let J=Z.mount(D,d);if(typeof J==="function")m.push(J)}if(Z.unmount)m.push(()=>Z.unmount(D))}}return n}});var Td=new WeakSet;function mb(i){if(Td.has(i))return;Td.add(i);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let d=new CSSStyleSheet;d.replaceSync(Qd),i.adoptedStyleSheets=[...i.adoptedStyleSheets,d];return}}catch{}let u=i.createElement("style");u.setAttribute("data-sui-table",""),u.textContent=Qd,i.head.appendChild(u)}var V0=N({scope:()=>{let i=G([]),u=G([]),d=G(null),m=G("asc"),n=G([]),b=h(()=>{let w=d(),r=m(),M=i();if(!w)return M;return[...M].sort((y,l)=>{let E=String(y[w]??""),t=String(l[w]??""),f=E<t?-1:E>t?1:0;return r==="asc"?f:-f})});return{rows:i,columns:u,sortKey:d,sortDir:m,selected:n,sortedRows:b}},on:{sortBy(i,u){if(i.sortKey===u)i.sortDir=i.sortDir==="asc"?"desc":"asc";else i.sortKey=u,i.sortDir="asc"},toggleSelect(i,u){let d=i.selected;i.selected=d.includes(u)?d.filter((m)=>m!==u):[...d,u]},selectAll(i){i.selected=i.rows.map((u)=>u.id)},clearSelection(i){i.selected=[]}},mount(i,u){let d=u.rows;if(d!=null)i.rows.set(d);let m=u.columns;if(m!=null)i.columns.set(m);Object.defineProperty(u,"rows",{configurable:!0,get:()=>i.rows(),set:(t)=>i.rows.set(t)}),Object.defineProperty(u,"columns",{configurable:!0,get:()=>i.columns(),set:(t)=>i.columns.set(t)}),mb(u.ownerDocument);let n=()=>{u.querySelectorAll("th[data-sort-key]").forEach((t)=>{let f=t.getAttribute("aria-sort")||"none",L=t.querySelector(".sui-sort-indicator");if(!L)return;if(f==="ascending")L.style.transform="rotate(180deg)",L.style.opacity="1";else if(f==="descending")L.style.transform="rotate(0deg)",L.style.opacity="1";else L.style.transform="rotate(0deg)",L.style.opacity="0.4"})},b=()=>{u.querySelectorAll("th[data-sort-key]").forEach((t)=>{if(!t.hasAttribute("aria-sort"))t.setAttribute("aria-sort","none");if(!t.hasAttribute("tabindex"))t.setAttribute("tabindex","0");if(!t.hasAttribute("role"))t.setAttribute("role","columnheader");if(!t.querySelector(".sui-sort-indicator")){let f=document.createElement("span");f.className="sui-sort-indicator",f.setAttribute("aria-hidden","true"),f.style.cssText="margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;";let L="http://www.w3.org/2000/svg",H=document.createElementNS(L,"svg");H.setAttribute("width","10"),H.setAttribute("height","10"),H.setAttribute("viewBox","0 0 10 10"),H.setAttribute("fill","none"),H.setAttribute("stroke","currentColor"),H.setAttribute("stroke-width","1.5"),H.setAttribute("stroke-linecap","round"),H.setAttribute("stroke-linejoin","round"),H.setAttribute("aria-hidden","true");let C=document.createElementNS(L,"path");C.setAttribute("d","M2 4l3-2 3 2M2 6l3 2 3-2"),H.appendChild(C),f.appendChild(H),t.appendChild(f)}}),n()},w=()=>{let t=u.querySelector(":scope > table tbody > tr")||u.querySelector(":scope > table > tr:not(:has(th))")||u.querySelector(":scope > tbody > tr")||u.querySelector(':scope > tr:not([slot="head"])'),f=u.querySelector('[slot="empty"]')!=null;if(!t&&f)u.setAttribute("data-empty","");else u.removeAttribute("data-empty")},r=(t)=>{let f=t.getAttribute("data-sort-key")||"",L=t.getAttribute("aria-sort")||"none",H=L==="none"?"ascending":L==="ascending"?"descending":"none";u.querySelectorAll("th[data-sort-key]").forEach((C)=>{if(C!==t)C.setAttribute("aria-sort","none")}),t.setAttribute("aria-sort",H),n(),u.emit("sui-sort",{key:f,direction:H})},M=(t)=>{let f=t.target.closest("th[data-sort-key]");if(!f||!u.contains(f))return;r(f)},y=(t)=>{if(t.key!=="Enter"&&t.key!==" ")return;let f=t.target.closest("th[data-sort-key]");if(!f||!u.contains(f))return;t.preventDefault(),r(f)},l=(t)=>{let f=t.target.closest("td");if(!f||!u.contains(f))return;if(f.classList.contains("sui-table-gutter-cell"))return;let L=f.closest("tr");if(!L)return;let H=L.getAttribute("data-row-id");if(!H)return;u.emit("sui-row-click",{id:H,index:L.rowIndex})};_(u,"click",M),_(u,"click",l),_(u,"keydown",y),b(),w();let E=new MutationObserver(()=>{b(),w()});return E.observe(u,{childList:!0,subtree:!0}),()=>{E.disconnect()}}}),Qd=`
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
`;var K7=V("sui-table",{props:{variant:P.union(["slot","native","data"],{default:"slot",reflect:!0}),density:P.union(["compact","default","comfortable"],{default:"default",reflect:!0}),dense:P.bool({default:!1,reflect:!0}),hoverable:P.bool({default:!0,reflect:!0}),striped:P.bool({default:!1,reflect:!0}),"sticky-header":P.bool({default:!1,reflect:!0}),"sticky-first-col":P.bool({default:!1,reflect:!0}),gutter:P.bool({default:!1,reflect:!0}),label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.variant=u.variant,n.density=u.density,n.dense=u.dense,n.hoverable=u.hoverable,n.striped=u.striped,n.stickyHeader=u.stickyHeader,n.stickyFirstCol=u.stickyFirstCol,n.gutter=u.gutter,n.label=u.label;let b=null;if(V0&&typeof V0.setup==="function"){if(i.scope=n,b=V0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(E)=>o({anchor:E,items:()=>n.columns(),key:(t,f)=>{let L=Object.create(n);return L.c=t,L.__i=f,((H)=>t.field)(L)},render:(t,f)=>{let L=Object.create(n);return L.c=t,L.__i=f,L.__v=G(0),((H)=>{let C=document.createDocumentFragment(),z=[],j=document.createElement("th");z.push(p(j,"click",function(){let Z=()=>H.sortBy(H.c.field);return typeof Z==="function"?Z.bind(void 0):()=>{}}(),{modifiers:[]})),z.push(zn(j,"sorted",()=>(H.__v(),H.sortKey()===H.c.field)));let g=document.createTextNode("");return j.appendChild(g),g.textContent=H.c.label,C.appendChild(j),{nodes:Array.from(C.childNodes),cleanup:()=>{for(let Z of z)try{Z()}catch(D){}},update:(Z,D)=>{H.c=Z,H.__i=D,g.textContent=H.c.label,H.__v.set((J)=>J+1)}}})(L)}}),r=(E)=>o({anchor:E,items:()=>n.sortedRows(),key:(t,f)=>{let L=Object.create(n);return L.r=t,L.__i=f,((H)=>t.id)(L)},render:(t,f)=>{let L=Object.create(n);return L.r=t,L.__i=f,L.__v=G(0),((H)=>{let C=(D)=>o({anchor:D,items:()=>H.columns(),key:(J,X)=>{let W=Object.create(H);return W.c=J,W.__i=X,((Q)=>J.field)(W)},render:(J,X)=>{let W=Object.create(H);return W.c=J,W.__i=X,W.__v=G(0),((Q)=>{let Y=document.createDocumentFragment(),O=[],K=document.createElement("td"),B=document.createTextNode("");return K.appendChild(B),B.textContent=Q.r[Q.c.field],Y.appendChild(K),{nodes:Array.from(Y.childNodes),cleanup:()=>{for(let A of O)try{A()}catch(a){}},update:(A,a)=>{Q.c=A,Q.__i=a,B.textContent=Q.r[Q.c.field],Q.__v.set((S)=>S+1)}}})(W)}}),z=document.createDocumentFragment(),j=[],g=document.createElement("tr");j.push(zn(g,"selected",()=>(H.__v(),H.selected().includes(H.r.id)))),j.push(p(g,"click",function(){let D=()=>H.toggleSelect(H.r.id);return typeof D==="function"?D.bind(void 0):()=>{}}(),{modifiers:[]}));let Z=document.createComment("for");return g.appendChild(Z),j.push(C(Z)),z.appendChild(g),{nodes:Array.from(z.childNodes),cleanup:()=>{for(let D of j)try{D()}catch(J){}},update:(D,J)=>{H.r=D,H.__i=J,H.__v.set((X)=>X+1)}}})(L)}}),M=(E)=>I({anchor:E,test:()=>{if(n.variant()==="data")return 0;if(n.variant()!=="native")return 1;return 2},branches:[()=>{let t=document.createDocumentFragment(),f=[],L=document.createElement("div");L.setAttribute("class","wrap"),L.setAttribute("part","wrap");let H=document.createElement("table");H.setAttribute("part","table"),f.push(T(H,"aria-label",()=>n.label()||null));let C=document.createElement("thead");C.setAttribute("part","thead");let z=document.createElement("tr"),j=document.createComment("for");z.appendChild(j),f.push(w(j)),C.appendChild(z),H.appendChild(C);let g=document.createElement("tbody");g.setAttribute("part","tbody");let Z=document.createComment("for");return g.appendChild(Z),f.push(r(Z)),H.appendChild(g),L.appendChild(H),t.appendChild(L),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let D of f)try{D()}catch(J){}}}},()=>{let t=document.createDocumentFragment(),f=[],L=document.createElement("div");L.setAttribute("class","wrap"),L.setAttribute("part","wrap");let H=document.createElement("table");H.setAttribute("part","table"),f.push(T(H,"aria-label",()=>n.label()||null));let C=document.createElement("thead");C.setAttribute("part","thead");let z=document.createElement("slot");z.setAttribute("name","head"),C.appendChild(z),H.appendChild(C);let j=document.createElement("tbody");j.setAttribute("part","tbody");let g=document.createElement("slot");j.appendChild(g),H.appendChild(j),L.appendChild(H);let Z=document.createElement("div");Z.setAttribute("class","empty"),Z.setAttribute("part","empty"),Z.setAttribute("role","status");let D=document.createElement("slot");return D.setAttribute("name","empty"),Z.appendChild(D),L.appendChild(Z),t.appendChild(L),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let J of f)try{J()}catch(X){}}}},()=>{let t=document.createDocumentFragment(),f=[],L=document.createElement("div");L.setAttribute("class","wrap"),L.setAttribute("part","wrap");let H=document.createElement("slot");L.appendChild(H);let C=document.createElement("div");C.setAttribute("class","empty"),C.setAttribute("part","empty"),C.setAttribute("role","status");let z=document.createElement("slot");return z.setAttribute("name","empty"),C.appendChild(z),L.appendChild(C),t.appendChild(L),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let j of f)try{j()}catch(g){}}}}]}),y=d.shadowRoot??d;q(y);let l=document.createComment("if");if(y.appendChild(l),m.push(M(l)),b!=null){let E=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(E){let t=new Proxy(n,{get(f,L){return f[L]},set(f,L,H){let C=f[L];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(E.mount){let f=E.mount(t,d);if(typeof f==="function")m.push(f)}if(E.unmount)m.push(()=>E.unmount(t))}}return n}});function ln(i){return String(i).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#39;")}function Vd(i,u){if(Array.isArray(i))return i;if(typeof i==="string"&&i.trim())try{let d=JSON.parse(i);return Array.isArray(d)?d:u}catch{return u}return u}function bb(i,u,d,m){let n=new Map,b=i.some((t)=>typeof t.x!=="number"||typeof t.y!=="number");if(m==="positioned"&&!b){for(let t of i)n.set(t.id,{x:t.x,y:t.y});return n}let w=u*0.22,r=d*0.5,M=i.filter((t)=>t.kind==="agent"),y=i.filter((t)=>t.kind!=="agent");M.forEach((t,f)=>{let L=Math.max(1,M.length),H=r+(f-(L-1)/2)*80;n.set(t.id,{x:w,y:H})});let l=new Map;for(let t of y){let f=l.get(t.kind)??[];f.push(t),l.set(t.kind,f)}let E=Array.from(l.keys());return E.forEach((t,f)=>{let L=l.get(t),H=u*(0.5+(f+1)/(E.length+1)*0.45);L.forEach((C,z)=>{let j=d*(z+1)/(L.length+1);n.set(C.id,{x:H,y:j})})}),n}function fb(i,u){let d=Math.max(40,Math.abs(u.x-i.x)*0.4);return`M ${i.x} ${i.y} C ${i.x+d} ${i.y}, ${u.x-d} ${u.y}, ${u.x} ${u.y}`}function tb(i,u){let d=i.querySelectorAll('template[slot="node"]');for(let m of Array.from(d))if(m.dataset.kind===u)return m;return null}function Lb(i,u){let d=tb(i,u.kind);if(!d){let m=u.sub?`<div class="node-sub">${ln(u.sub)}</div>`:"";return`<div class="node-h"><span class="tag">${ln(u.kind)}</span></div><div class="node-title">${ln(u.title??u.id)}</div>${m}`}return d.innerHTML.replace(/\$\{title\}/g,ln(u.title??"")).replace(/\$\{sub\}/g,ln(u.sub??"")).replace(/\$\{id\}/g,ln(u.id))}function lb(i,u,d,m,n,b,w){let r=Vd(u,[]),M=Vd(d,[]),y=bb(r,m,n,b),l=M.map((t)=>{let f=y.get(t.from),L=y.get(t.to);if(!f||!L)return"";let H=ln(fb(f,L)),C=t.dashed?" data-dashed":"";return`<path d="${H}" data-kind="${ln(t.kind??"default")}"${C}></path>`}).join(""),E=r.map((t)=>{let f=y.get(t.id);if(!f)return"";let L=f.x/m*100,H=f.y/n*100,C=w&&w===t.id?" data-selected":"",z=t.w?`--node-w: ${t.w}px;`:"",j=ln(`${t.kind}: ${t.title??t.id}`),g=Lb(i,t);return`<div class="node" part="node" data-kind="${ln(t.kind)}" data-id="${ln(t.id)}"${C} tabindex="0" role="button" aria-label="${j}" style="left: ${L}%; top: ${H}%;${z}">${g}</div>`}).join("");return`<svg class="edges" part="edges" viewBox="0 0 ${m} ${n}" preserveAspectRatio="none" aria-hidden="true">${l}</svg>`+E}var q0=N({mount(i,u){let d=i,m=d.canvasEl();if(!m)return;let n=m.ownerDocument.createElement("div");m.appendChild(n);let b=fn(n,()=>{let l=d.nodes(),E=d.edges(),t=d.width()||1000,f=d.height()||600,L=d.layout()||"positioned",H=d.selectedId();return lb(u,l,E,t,f,L,H)}),w="",r=(l)=>{let E=l.target.closest(".node[data-id]");if(!E)return;let t=E.getAttribute("data-id")??"",f=E.getAttribute("data-kind")??"";u.setAttribute("selected-id",t),u.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:t,kind:f},bubbles:!0,composed:!0}))},M=(l)=>{if(l.key!=="Enter"&&l.key!==" ")return;let E=l.target.closest(".node[data-id]");if(!E)return;l.preventDefault();let t=E.getAttribute("data-id")??"",f=E.getAttribute("data-kind")??"";u.setAttribute("selected-id",t),u.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:t,kind:f},bubbles:!0,composed:!0}))},y=(l)=>{let E=l.target.closest(".node[data-id]");if(!E){w="";return}let t=E.getAttribute("data-id")??"";if(t===w)return;w=t;let f=E.getAttribute("data-kind")??"";u.dispatchEvent(new CustomEvent("sui-node-hover",{detail:{id:t,kind:f},bubbles:!0,composed:!0}))};return m.addEventListener("click",r),m.addEventListener("keydown",M),m.addEventListener("mouseover",y),()=>{b(),m.removeEventListener("click",r),m.removeEventListener("keydown",M),m.removeEventListener("mouseover",y)}}});var a7=V("sui-graph-canvas",{props:{nodes:P.string({default:""}),edges:P.string({default:""}),width:P.number({default:1000,reflect:!0}),height:P.number({default:600,reflect:!0}),layout:P.union(["positioned","radial"],{default:"positioned",reflect:!0}),"selected-id":P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.nodes=u.nodes,n.edges=u.edges,n.width=u.width,n.height=u.height,n.layout=u.layout,n.selectedId=u.selectedId,n.canvasEl=G(null);let b=null;if(q0&&typeof q0.setup==="function"){if(i.scope=n,b=q0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("div");r.setAttribute("class","canvas"),r.setAttribute("part","canvas"),F(n,"canvasEl",r),w.appendChild(r);let M=document.createElement("slot");if(M.setAttribute("name","node"),M.setAttribute("hidden",""),w.appendChild(M),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let l=new Proxy(n,{get(E,t){return E[t]},set(E,t,f){let L=E[t];if(L!=null&&typeof L.set==="function"){if(typeof f==="function")L.set(()=>f);else L.set(f);return!0}return!1}});if(y.mount){let E=y.mount(l,d);if(typeof E==="function")m.push(E)}if(y.unmount)m.push(()=>y.unmount(l))}}return n}});var Y0=N({mount(i,u){if(!u.hasAttribute("role"))u.setAttribute("role","list");if(!u.hasAttribute("aria-label"))u.setAttribute("aria-label","pipeline steps")}});var s7=V("sui-pipeline-dag",{shadow:!0,styles:`
  :host {
    display: flex;
    align-items: stretch;
    gap: 0;
    overflow-x: auto;
    padding: 4px 0 2px;
  }
  ::slotted(.arrow) {
    display: flex;
    align-items: center;
    padding: 0 6px;
    color: var(--border);
    font-size: 14px;
    flex-shrink: 0;
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null),b=null;if(Y0&&typeof Y0.setup==="function"){if(i.scope=n,b=Y0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("slot");if(w.appendChild(r),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(l,E){return l[E]},set(l,E,t){let f=l[E];if(f!=null&&typeof f.set==="function"){if(typeof t==="function")f.set(()=>t);else f.set(t);return!0}return!1}});if(M.mount){let l=M.mount(y,d);if(typeof l==="function")m.push(l)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var U0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","listitem");let m=U(()=>{let n=d.name(),b=d.stepType(),w=d.agent(),r=d.state(),M=`${b} step ${n}${w?` (${w})`:""}: ${r}`;u.setAttribute("aria-label",M)});return()=>{m()}}});var i9=V("sui-pipeline-step",{props:{name:P.string({default:"",reflect:!0}),"step-type":P.union(["agent","gate","merge","run","human"],{default:"agent",reflect:!0}),state:P.union(["pending","running","completed","failed"],{default:"pending",reflect:!0}),agent:P.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    gap: 4px;
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
    top: 10px;
    right: 12px;
    font-size: 10px;
    color: var(--muted-foreground);
    line-height: 1;
  }
  :host([state='completed']) {
    border-color: var(--tone-success-border);
    background: var(--tone-success-bg);
  }
  :host([state='completed']) .icon { color: var(--success); }
  :host([state='running']) {
    border-color: var(--tone-info-border);
    background: var(--tone-info-bg);
    box-shadow: 0 0 0 3px var(--tone-info-ring);
  }
  :host([state='running']) .icon { color: var(--info); }
  :host([state='failed']) {
    border-color: var(--tone-destructive-border);
    background: var(--tone-destructive-bg);
  }
  :host([state='failed']) .icon { color: var(--destructive); }

  .name {
    font-family: var(--font-mono);
    font-size: 11.5px;
    font-weight: 600;
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
  sui-spinner {
    position: absolute;
    top: 10px;
    right: 12px;
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.stepType=u.stepType,n.state=u.state,n.agent=u.agent;let b=null;if(U0&&typeof U0.setup==="function"){if(i.scope=n,b=U0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(z)=>I({anchor:z,test:()=>{if(n.state()==="running")return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),g=[],Z=document.createElement("sui-spinner");return Z.setAttribute("size","sm"),Z.setAttribute("tone","info"),Z.setAttribute("label","Running"),j.appendChild(Z),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let D of g)try{D()}catch(J){}}}},()=>{let j=document.createDocumentFragment(),g=[],Z=document.createElement("span");Z.setAttribute("class","icon"),Z.setAttribute("aria-hidden","true");let D=document.createTextNode("");return Z.appendChild(D),g.push(k(D,()=>n.state()==="completed"?"✓":n.state()==="failed"?"✗":"○")),j.appendChild(Z),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let J of g)try{J()}catch(X){}}}}]}),r=(z)=>I({anchor:z,test:()=>{if(n.agent())return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),g=[],Z=document.createTextNode(" · ");j.appendChild(Z);let D=document.createTextNode("");return j.appendChild(D),g.push(k(D,()=>n.agent())),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let J of g)try{J()}catch(X){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createComment("if");M.appendChild(y),m.push(w(y));let l=document.createElement("span");l.setAttribute("class","name");let E=document.createTextNode("");l.appendChild(E),m.push(k(E,()=>n.name())),M.appendChild(l);let t=document.createElement("span");t.setAttribute("class","type");let f=document.createTextNode("");t.appendChild(f),m.push(k(f,()=>n.stepType()));let L=document.createComment("if");t.appendChild(L),m.push(r(L)),M.appendChild(t);let H=document.createElement("span");H.setAttribute("class","status");let C=document.createTextNode("");if(H.appendChild(C),m.push(k(C,()=>n.state())),M.appendChild(H),b!=null){let z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(z){let j=new Proxy(n,{get(g,Z){return g[Z]},set(g,Z,D){let J=g[Z];if(J!=null&&typeof J.set==="function"){if(typeof D==="function")J.set(()=>D);else J.set(D);return!0}return!1}});if(z.mount){let g=z.mount(j,d);if(typeof g==="function")m.push(g)}if(z.unmount)m.push(()=>z.unmount(j))}}return n}});var Eb={dispatched:"info",running:"info",gatepending:"warning",completed:"success",failed:"destructive",merged:"accent",pending:"muted"},N0=N({scope:(i)=>{let u=i;return{tone:h(()=>Eb[u.state()]??"muted")}},mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","article");let m=U(()=>{let n=d.slug(),b=d.agent(),w=d.state();u.setAttribute("aria-label",`Pipeline run ${n||""}${b?` on ${b}`:""}: ${w}`)});return()=>{m()}}});var t9=V("sui-pipeline-run",{props:{state:P.union(["dispatched","gatepending","running","completed","failed","merged","pending"],{default:"pending",reflect:!0}),slug:P.string({default:"",reflect:!0}),agent:P.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host { display: block; }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    gap: 12px;
    flex-wrap: wrap;
  }
  .left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }
  .slug {
    font-family: var(--font-mono);
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
    font-family: var(--font-mono);
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.state=u.state,n.slug=u.slug,n.agent=u.agent;let b=null;if(N0&&typeof N0.setup==="function"){if(i.scope=n,b=N0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(D)=>I({anchor:D,test:()=>{if(n.agent())return 0;return-1},branches:[()=>{let J=document.createDocumentFragment(),X=[],W=document.createElement("span");W.setAttribute("class","agent");let Q=document.createElement("code"),Y=document.createTextNode("");return Q.appendChild(Y),X.push(k(Y,()=>n.agent())),W.appendChild(Q),J.appendChild(W),{nodes:Array.from(J.childNodes),cleanup:()=>{for(let O of X)try{O()}catch(K){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createElement("sui-card");M.setAttribute("status-rail",""),m.push(T(M,"tone",()=>n.tone()));let y=document.createElement("div");y.setAttribute("slot","header");let l=document.createElement("header"),E=document.createElement("div");E.setAttribute("class","left");let t=document.createElement("span");t.setAttribute("class","slug");let f=document.createTextNode("");t.appendChild(f),m.push(k(f,()=>n.slug())),E.appendChild(t);let L=document.createComment("if");E.appendChild(L),m.push(w(L));let H=document.createElement("slot");H.setAttribute("name","actions"),E.appendChild(H),l.appendChild(E);let C=document.createElement("sui-badge");m.push(T(C,"tone",()=>n.tone()));let z=document.createTextNode("");C.appendChild(z),m.push(k(z,()=>n.state())),l.appendChild(C),y.appendChild(l),M.appendChild(y);let j=document.createElement("slot");M.appendChild(j);let g=document.createElement("div");g.setAttribute("slot","footer");let Z=document.createElement("slot");if(Z.setAttribute("name","footer"),g.appendChild(Z),M.appendChild(g),r.appendChild(M),b!=null){let D=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(D){let J=new Proxy(n,{get(X,W){return X[W]},set(X,W,Q){let Y=X[W];if(Y!=null&&typeof Y.set==="function"){if(typeof Q==="function")Y.set(()=>Q);else Y.set(Q);return!0}return!1}});if(D.mount){let X=D.mount(J,d);if(typeof X==="function")m.push(X)}if(D.unmount)m.push(()=>D.unmount(J))}}return n}});var rb={run:"muted",agent:"info",gate:"warning",merge:"accent",human:"warning"},F0=N({scope:(i)=>{let u=i,d=h(()=>rb[u.stepType()]??"muted"),m=h(()=>(u.after()||"").split(",").map((b)=>b.trim()).filter(Boolean)),n=h(()=>(u.command()||"").trim());return{kindTone:d,afterList:m,commandTrim:n}}});var y9=V("sui-pipeline-step-def",{props:{name:P.string({default:"",reflect:!0}),"step-type":P.union(["agent","gate","run","merge","human"],{default:"agent",reflect:!0}),agent:P.string({default:"",reflect:!0}),provider:P.string({default:"",reflect:!0}),model:P.string({default:"",reflect:!0}),after:P.string({default:"",reflect:!0}),command:P.string({default:""}),index:P.number({default:0,reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    padding: 16px 0;
    border-top: 1px solid var(--border-subtle);
  }
  :host(:first-of-type), :host([first]) { border-top: none; padding-top: 0; }
  :host(:last-of-type) { padding-bottom: 0; }

  [part='row'] {
    display: grid;
    grid-template-columns: 2ch 1fr;
    gap: 16px;
  }
  .index {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    padding-top: 3px;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }
  [part='header'] {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .name {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: var(--font-weight-semibold);
    color: var(--foreground);
  }
  .after {
    font-size: var(--text-xs);
    color: var(--muted-foreground);
  }
  .after code {
    font-family: var(--font-mono);
    background: color-mix(in oklch, var(--muted) 40%, transparent);
    padding: 1px 5px;
    border-radius: 4px;
    font-size: var(--text-xs);
  }
  .chips { display: flex; gap: 6px; flex-wrap: wrap; }
  [part='command'] {
    margin: 0;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    background: color-mix(in oklch, var(--muted) 35%, transparent);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    line-height: 1.55;
    color: var(--foreground);
    white-space: pre-wrap;
    word-break: break-word;
    overflow-x: auto;
  }
  .trailing { margin-left: auto; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.stepType=u.stepType,n.agent=u.agent,n.provider=u.provider,n.model=u.model,n.after=u.after,n.command=u.command,n.index=u.index;let b=null;if(F0&&typeof F0.setup==="function"){if(i.scope=n,b=F0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(B)=>o({anchor:B,items:()=>n.afterList(),key:(A,a)=>{let S=Object.create(n);return S.a=A,S.idx=a,((v)=>A)(S)},render:(A,a)=>{let S=Object.create(n);return S.a=A,S.idx=a,S.__v=G(0),((v)=>{let R=(dn)=>I({anchor:dn,test:()=>{if(v.idx>0)return 0;return-1},branches:[()=>{let Ln=document.createDocumentFragment(),jn=[],v0=document.createTextNode(", ");return Ln.appendChild(v0),{nodes:Array.from(Ln.childNodes),cleanup:()=>{for(let An of jn)try{An()}catch(on){}}}}]}),x=document.createDocumentFragment(),s=[],$=document.createComment("if");x.appendChild($),s.push(R($));let nn=document.createElement("code"),tn=document.createTextNode("");return nn.appendChild(tn),tn.textContent=v.a,x.appendChild(nn),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let dn of s)try{dn()}catch(Ln){}},update:(dn,Ln)=>{v.a=dn,v.idx=Ln,tn.textContent=v.a,v.__v.set((jn)=>jn+1)}}})(S)}}),r=(B)=>I({anchor:B,test:()=>{if(n.afterList().length>0)return 0;return-1},branches:[()=>{let A=document.createDocumentFragment(),a=[],S=document.createElement("span");S.setAttribute("class","after");let v=document.createTextNode("after ");S.appendChild(v);let R=document.createComment("for");return S.appendChild(R),a.push(w(R)),A.appendChild(S),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let x of a)try{x()}catch(s){}}}}]}),M=(B)=>I({anchor:B,test:()=>{if(n.agent())return 0;return-1},branches:[()=>{let A=document.createDocumentFragment(),a=[],S=document.createElement("sui-chip");S.setAttribute("size","sm");let v=document.createTextNode("");return S.appendChild(v),a.push(k(v,()=>n.agent())),A.appendChild(S),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let R of a)try{R()}catch(x){}}}}]}),y=(B)=>I({anchor:B,test:()=>{if(n.provider())return 0;return-1},branches:[()=>{let A=document.createDocumentFragment(),a=[],S=document.createElement("sui-chip");S.setAttribute("size","sm");let v=document.createTextNode("");return S.appendChild(v),a.push(k(v,()=>n.provider())),A.appendChild(S),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let R of a)try{R()}catch(x){}}}}]}),l=(B)=>I({anchor:B,test:()=>{if(n.model())return 0;return-1},branches:[()=>{let A=document.createDocumentFragment(),a=[],S=document.createElement("sui-chip");S.setAttribute("size","sm");let v=document.createTextNode("");return S.appendChild(v),a.push(k(v,()=>n.model())),A.appendChild(S),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let R of a)try{R()}catch(x){}}}}]}),E=(B)=>I({anchor:B,test:()=>{if(n.agent()||n.provider()||n.model())return 0;return-1},branches:[()=>{let A=document.createDocumentFragment(),a=[],S=document.createElement("div");S.setAttribute("class","chips");let v=document.createComment("if");S.appendChild(v),a.push(M(v));let R=document.createComment("if");S.appendChild(R),a.push(y(R));let x=document.createComment("if");return S.appendChild(x),a.push(l(x)),A.appendChild(S),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let s of a)try{s()}catch($){}}}}]}),t=(B)=>I({anchor:B,test:()=>{if(n.commandTrim())return 0;return-1},branches:[()=>{let A=document.createDocumentFragment(),a=[],S=document.createElement("pre");S.setAttribute("part","command");let v=document.createTextNode("");return S.appendChild(v),a.push(k(v,()=>n.commandTrim())),A.appendChild(S),{nodes:Array.from(A.childNodes),cleanup:()=>{for(let R of a)try{R()}catch(x){}}}}]}),f=d.shadowRoot??d;q(f);let L=document.createElement("div");L.setAttribute("part","row");let H=document.createElement("span");H.setAttribute("class","index");let C=document.createTextNode("");H.appendChild(C),m.push(k(C,()=>n.index()||"")),L.appendChild(H);let z=document.createElement("div");z.setAttribute("class","body");let j=document.createElement("div");j.setAttribute("part","header");let g=document.createElement("span");g.setAttribute("class","name");let Z=document.createTextNode("");g.appendChild(Z),m.push(k(Z,()=>n.name())),j.appendChild(g);let D=document.createElement("sui-chip");D.setAttribute("size","sm"),m.push(T(D,"tone",()=>n.kindTone()));let J=document.createTextNode("");D.appendChild(J),m.push(k(J,()=>n.stepType())),j.appendChild(D);let X=document.createComment("if");j.appendChild(X),m.push(r(X));let W=document.createElement("span");W.setAttribute("class","trailing");let Q=document.createElement("slot");Q.setAttribute("name","trailing"),W.appendChild(Q),j.appendChild(W),z.appendChild(j);let Y=document.createComment("if");z.appendChild(Y),m.push(E(Y));let O=document.createComment("if");z.appendChild(O),m.push(t(O));let K=document.createElement("slot");if(K.setAttribute("name","gate"),z.appendChild(K),L.appendChild(z),f.appendChild(L),b!=null){let B=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(B){let A=new Proxy(n,{get(a,S){return a[S]},set(a,S,v){let R=a[S];if(R!=null&&typeof R.set==="function"){if(typeof v==="function")R.set(()=>v);else R.set(v);return!0}return!1}});if(B.mount){let a=B.mount(A,d);if(typeof a==="function")m.push(a)}if(B.unmount)m.push(()=>B.unmount(A))}}return n}});var O0=N({scope:(i)=>{let u=i;return{stepsLabel:h(()=>{let m=u.steps();return`${m} step${m===1?"":"s"}`})}},mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","article");let m=U(()=>{let n=d.name(),b=d.steps(),w=d.description(),r=b===1?"":"s",M=w?`: ${w}`:"";u.setAttribute("aria-label",`Pipeline ${n}, ${b} step${r}${M}`)});return()=>{m()}}});var P9=V("sui-pipeline-definition",{props:{name:P.string({default:"",reflect:!0}),steps:P.number({default:0,reflect:!0}),description:P.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host { display: block; }
  header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 12px;
  }
  .name {
    font-family: var(--heading-font, var(--font-body));
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
    margin: 6px 0 0;
  }
  .no-desc { opacity: 0.5; }
  sui-card { display: block; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.steps=u.steps,n.description=u.description;let b=null;if(O0&&typeof O0.setup==="function"){if(i.scope=n,b=O0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(z)=>I({anchor:z,test:()=>{if(n.description())return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),g=[],Z=document.createElement("p");Z.setAttribute("class","desc");let D=document.createTextNode("");return Z.appendChild(D),g.push(k(D,()=>n.description())),j.appendChild(Z),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let J of g)try{J()}catch(X){}}}},()=>{let j=document.createDocumentFragment(),g=[],Z=document.createElement("p");Z.setAttribute("class","desc no-desc");let D=document.createTextNode("No description");return Z.appendChild(D),j.appendChild(Z),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let J of g)try{J()}catch(X){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createElement("sui-card");M.setAttribute("interactive",""),M.setAttribute("accent-bar",""),M.setAttribute("padding","md");let y=document.createElement("div");y.setAttribute("slot","header");let l=document.createElement("header"),E=document.createElement("span");E.setAttribute("class","name");let t=document.createTextNode("");E.appendChild(t),m.push(k(t,()=>n.name())),l.appendChild(E);let f=document.createElement("span");f.setAttribute("class","steps");let L=document.createTextNode("");f.appendChild(L),m.push(k(L,()=>n.stepsLabel())),l.appendChild(f);let H=document.createElement("slot");H.setAttribute("name","actions"),l.appendChild(H),y.appendChild(l),M.appendChild(y);let C=document.createComment("if");if(M.appendChild(C),m.push(w(C)),r.appendChild(M),b!=null){let z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(z){let j=new Proxy(n,{get(g,Z){return g[Z]},set(g,Z,D){let J=g[Z];if(J!=null&&typeof J.set==="function"){if(typeof D==="function")J.set(()=>D);else J.set(D);return!0}return!1}});if(z.mount){let g=z.mount(j,d);if(typeof g==="function")m.push(g)}if(z.unmount)m.push(()=>z.unmount(j))}}return n}});var A0=N({scope:()=>{return{preEl:e(null)}},mount(i,u){let d=i,m=null,n="",b=!0,w=()=>{let f=d.preEl();if(!f)return;if(n.length===0)f.textContent="(no output yet)",f.setAttribute("data-empty","");else f.textContent=n,f.removeAttribute("data-empty");if(d.autoscroll()&&b)f.scrollTop=f.scrollHeight},r=(f)=>{n+=f;let L=d.maxBytes();if(n.length>L){let H=n.length-L,C=n.indexOf(`
`,H);n=n.slice(C>=0?C+1:H)}w()},M=()=>{if(m)m.close(),m=null,u.dispatchEvent(new CustomEvent("sui-stream-close",{bubbles:!0,composed:!0}))},y=()=>{let f=d.url();if(!f||m)return;try{m=new EventSource(f,{withCredentials:!0})}catch{u.dispatchEvent(new CustomEvent("sui-stream-error",{bubbles:!0,composed:!0}));return}m.addEventListener(d.event(),(L)=>{let H=L.data??"";r(H),u.dispatchEvent(new CustomEvent("sui-log-tick",{detail:{bytes:H.length},bubbles:!0,composed:!0}))}),m.onopen=()=>{u.dispatchEvent(new CustomEvent("sui-stream-open",{bubbles:!0,composed:!0}))},m.onerror=()=>{u.dispatchEvent(new CustomEvent("sui-stream-error",{bubbles:!0,composed:!0}))}},l=(f)=>{let L=f.currentTarget;b=L.scrollHeight-L.clientHeight<=L.scrollTop+4},E=U(()=>{let f=d.preEl();if(!f)return;return f.addEventListener("scroll",l),w(),()=>f.removeEventListener("scroll",l)}),t=U(()=>{if(d.url(),d.event(),d.active()){if(m)M();y()}else M()});return u.clear=()=>{n="",w()},()=>{E(),t(),M()}}});var G9=V("sui-log-stream",{props:{url:P.string({default:"",reflect:!0}),event:P.string({default:"log",reflect:!0}),autoscroll:P.bool({default:!1,reflect:!0}),active:P.bool({default:!1,reflect:!0}),"max-bytes":P.number({default:1e6})},shadow:!0,styles:`
  :host {
    display: block;
    min-height: 8rem;
    max-height: 24rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    background: color-mix(in oklch, var(--background) 95%, black 5%);
    color: var(--foreground);
    overflow: hidden;
  }
  [part='scroll'] {
    height: 100%;
    max-height: inherit;
    margin: 0;
    padding: var(--space-3) var(--space-4);
    font-family: var(--font-mono);
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.url=u.url,n.event=u.event,n.autoscroll=u.autoscroll,n.active=u.active,n.maxBytes=u.maxBytes,n.preEl=G(null);let b=null;if(A0&&typeof A0.setup==="function"){if(i.scope=n,b=A0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("pre");r.setAttribute("part","scroll"),r.setAttribute("data-empty",""),F(n,"preEl",r);let M=document.createTextNode("(no output yet)");if(r.appendChild(M),w.appendChild(r),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let l=new Proxy(n,{get(E,t){return E[t]},set(E,t,f){let L=E[t];if(L!=null&&typeof L.set==="function"){if(typeof f==="function")L.set(()=>f);else L.set(f);return!0}return!1}});if(y.mount){let E=y.mount(l,d);if(typeof E==="function")m.push(E)}if(y.unmount)m.push(()=>y.unmount(l))}}return n}});var Jn={claude:{label:"Claude",hue:"oklch(0.70 0.16 35)",svg:'<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="#D97757"><title>Claude</title><path d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></svg>'},codex:{label:"Codex",hue:"oklch(0.22 0.02 220)",svg:'<svg width="721" height="721" viewBox="0 0 721 721" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1637_2934)"><g clip-path="url(#clip1_1637_2934)"><path d="M304.246 294.611V249.028C304.246 245.189 305.687 242.309 309.044 240.392L400.692 187.612C413.167 180.415 428.042 177.058 443.394 177.058C500.971 177.058 537.44 221.682 537.44 269.182C537.44 272.54 537.44 276.379 536.959 280.218L441.954 224.558C436.197 221.201 430.437 221.201 424.68 224.558L304.246 294.611ZM518.245 472.145V363.224C518.245 356.505 515.364 351.707 509.608 348.349L389.174 278.296L428.519 255.743C431.877 253.826 434.757 253.826 438.115 255.743L529.762 308.523C556.154 323.879 573.905 356.505 573.905 388.171C573.905 424.636 552.315 458.225 518.245 472.141V472.145ZM275.937 376.182L236.592 353.152C233.235 351.235 231.794 348.354 231.794 344.515V238.956C231.794 187.617 271.139 148.749 324.4 148.749C344.555 148.749 363.264 155.468 379.102 167.463L284.578 222.164C278.822 225.521 275.942 230.319 275.942 237.039V376.186L275.937 376.182ZM360.626 425.122L304.246 393.455V326.283L360.626 294.616L417.002 326.283V393.455L360.626 425.122ZM396.852 570.989C376.698 570.989 357.989 564.27 342.151 552.276L436.674 497.574C442.431 494.217 445.311 489.419 445.311 482.699V343.552L485.138 366.582C488.495 368.499 489.936 371.379 489.936 375.219V480.778C489.936 532.117 450.109 570.985 396.852 570.985V570.989ZM283.134 463.99L191.486 411.211C165.094 395.854 147.343 363.229 147.343 331.562C147.343 294.616 169.415 261.509 203.48 247.593V356.991C203.48 363.71 206.361 368.508 212.117 371.866L332.074 441.437L292.729 463.99C289.372 465.907 286.491 465.907 283.134 463.99ZM277.859 542.68C223.639 542.68 183.813 501.895 183.813 451.514C183.813 447.675 184.294 443.836 184.771 439.997L279.295 494.698C285.051 498.056 290.812 498.056 296.568 494.698L417.002 425.127V470.71C417.002 474.549 415.562 477.429 412.204 479.346L320.557 532.126C308.081 539.323 293.206 542.68 277.854 542.68H277.859ZM396.852 599.776C454.911 599.776 503.37 558.513 514.41 503.812C568.149 489.896 602.696 439.515 602.696 388.176C602.696 354.587 588.303 321.962 562.392 298.45C564.791 288.373 566.231 278.296 566.231 268.224C566.231 199.611 510.571 148.267 446.274 148.267C433.322 148.267 420.846 150.184 408.37 154.505C386.775 133.392 357.026 119.958 324.4 119.958C266.342 119.958 217.883 161.22 206.843 215.921C153.104 229.837 118.557 280.218 118.557 331.557C118.557 365.146 132.95 397.771 158.861 421.283C156.462 431.36 155.022 441.437 155.022 451.51C155.022 520.123 210.682 571.466 274.978 571.466C287.931 571.466 300.407 569.549 312.883 565.228C334.473 586.341 364.222 599.776 396.852 599.776Z" fill="black"/></g></g><defs><clipPath id="clip0_1637_2934"><rect width="720" height="720" fill="white" transform="translate(0.606934 0.0999756)"/></clipPath><clipPath id="clip1_1637_2934"><rect width="484.139" height="479.818" fill="white" transform="translate(118.557 119.958)"/></clipPath></defs></svg>',svgDark:'<svg width="721" height="721" viewBox="0 0 721 721" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1637_2935)"><g clip-path="url(#clip1_1637_2935)"><path d="M304.246 295.411V249.828C304.246 245.989 305.687 243.109 309.044 241.191L400.692 188.412C413.167 181.215 428.042 177.858 443.394 177.858C500.971 177.858 537.44 222.482 537.44 269.982C537.44 273.34 537.44 277.179 536.959 281.018L441.954 225.358C436.197 222 430.437 222 424.68 225.358L304.246 295.411ZM518.245 472.945V364.024C518.245 357.304 515.364 352.507 509.608 349.149L389.174 279.096L428.519 256.543C431.877 254.626 434.757 254.626 438.115 256.543L529.762 309.323C556.154 324.679 573.905 357.304 573.905 388.971C573.905 425.436 552.315 459.024 518.245 472.941V472.945ZM275.937 376.982L236.592 353.952C233.235 352.034 231.794 349.154 231.794 345.315V239.756C231.794 188.416 271.139 149.548 324.4 149.548C344.555 149.548 363.264 156.268 379.102 168.262L284.578 222.964C278.822 226.321 275.942 231.119 275.942 237.838V376.986L275.937 376.982ZM360.626 425.922L304.246 394.255V327.083L360.626 295.416L417.002 327.083V394.255L360.626 425.922ZM396.852 571.789C376.698 571.789 357.989 565.07 342.151 553.075L436.674 498.374C442.431 495.017 445.311 490.219 445.311 483.499V344.352L485.138 367.382C488.495 369.299 489.936 372.179 489.936 376.018V481.577C489.936 532.917 450.109 571.785 396.852 571.785V571.789ZM283.134 464.79L191.486 412.01C165.094 396.654 147.343 364.029 147.343 332.362C147.343 295.416 169.415 262.309 203.48 248.393V357.791C203.48 364.51 206.361 369.308 212.117 372.665L332.074 442.237L292.729 464.79C289.372 466.707 286.491 466.707 283.134 464.79ZM277.859 543.48C223.639 543.48 183.813 502.695 183.813 452.314C183.813 448.475 184.294 444.636 184.771 440.797L279.295 495.498C285.051 498.856 290.812 498.856 296.568 495.498L417.002 425.927V471.509C417.002 475.349 415.562 478.229 412.204 480.146L320.557 532.926C308.081 540.122 293.206 543.48 277.854 543.48H277.859ZM396.852 600.576C454.911 600.576 503.37 559.313 514.41 504.612C568.149 490.696 602.696 440.315 602.696 388.976C602.696 355.387 588.303 322.762 562.392 299.25C564.791 289.173 566.231 279.096 566.231 269.024C566.231 200.411 510.571 149.067 446.274 149.067C433.322 149.067 420.846 150.984 408.37 155.305C386.775 134.192 357.026 120.758 324.4 120.758C266.342 120.758 217.883 162.02 206.843 216.721C153.104 230.637 118.557 281.018 118.557 332.357C118.557 365.946 132.95 398.571 158.861 422.083C156.462 432.16 155.022 442.237 155.022 452.309C155.022 520.922 210.682 572.266 274.978 572.266C287.931 572.266 300.407 570.349 312.883 566.028C334.473 587.141 364.222 600.576 396.852 600.576Z" fill="white"/></g></g><defs><clipPath id="clip0_1637_2935"><rect width="720" height="720" fill="white" transform="translate(0.606934 0.899902)"/></clipPath><clipPath id="clip1_1637_2935"><rect width="484.139" height="479.818" fill="white" transform="translate(118.557 120.758)"/></clipPath></defs></svg>'},cursor:{label:"Cursor",hue:"oklch(0.45 0.02 250)",svg:'<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><title>Cursor</title><path d="M11.925.0078 24 6.9702v10.0562L11.925 24 0 17.0288V6.9702ZM2.64 8.529 12 13.924v9.6448l9.36-5.3962V7.529l-9.36 5.4ZM10.677 11.11 2.64 6.4702l9.36-5.4038 8.01 4.6212Z"/></svg>'},gemini:{label:"Gemini",hue:"oklch(0.62 0.14 285)",svg:'<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="#8E75B2"><title>Google Gemini</title><path d="M11.04 19.32Q12 21.51 12 24q0-2.49.93-4.68.96-2.19 2.58-3.81t3.81-2.55Q21.51 12 24 12q-2.49 0-4.68-.93a12.3 12.3 0 0 1-3.81-2.58 12.3 12.3 0 0 1-2.58-3.81Q12 2.49 12 0q0 2.49-.96 4.68-.93 2.19-2.55 3.81a12.3 12.3 0 0 1-3.81 2.58Q2.49 12 0 12q2.49 0 4.68.96 2.19.93 3.81 2.55t2.55 3.81"/></svg>'},opencode:{label:"OpenCode",hue:"oklch(0.60 0.18 35)",svg:'<svg width="240" height="300" viewBox="0 0 240 300" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1401_86274)"><mask id="mask0_1401_86274" style="mask-type:luminance" maskUnits="userSpaceOnUse" x="0" y="0" width="240" height="300"><path d="M240 0H0V300H240V0Z" fill="white"/></mask><g mask="url(#mask0_1401_86274)"><path d="M180 240H60V120H180V240Z" fill="#CFCECD"/><path d="M180 60H60V240H180V60ZM240 300H0V0H240V300Z" fill="#211E1E"/></g></g><defs><clipPath id="clip0_1401_86274"><rect width="240" height="300" fill="white"/></clipPath></defs></svg>',svgDark:'<svg width="240" height="300" viewBox="0 0 240 300" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1401_86283)"><mask id="mask0_1401_86283" style="mask-type:luminance" maskUnits="userSpaceOnUse" x="0" y="0" width="240" height="300"><path d="M240 0H0V300H240V0Z" fill="white"/></mask><g mask="url(#mask0_1401_86283)"><path d="M180 240H60V120H180V240Z" fill="#4B4646"/><path d="M180 60H60V240H180V60ZM240 300H0V0H240V300Z" fill="#F1ECEC"/></g></g><defs><clipPath id="clip0_1401_86283"><rect width="240" height="300" fill="white"/></clipPath></defs></svg>'}};var K0=N({scope:(i)=>{let u=i,d=e(null),m=h(()=>(Jn[u.provider()]??Jn.claude).label);return{tileEl:d,tileLabel:m}},mount(i,u){let d=i,m=U(()=>{let b=Jn[d.provider()]??Jn.claude;u.style.setProperty("--blob-hue",b.hue)}),n=U(()=>{let b=d.tileEl();if(!b)return;return fn(b,()=>{return(Jn[d.provider()]??Jn.claude).svg})});return()=>{m(),n()}}});var N9=V("sui-provider-blob",{props:{provider:P.union(["claude","codex","cursor","gemini","opencode"],{default:"claude",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:P.union(["tile","glyph"],{default:"tile",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    vertical-align: middle;
    font-family: var(--font-body);
    font-size: var(--text-xs);
    color: var(--muted-foreground);
    --blob-size: 20px;
    --blob-hue: var(--muted);
  }
  :host([size='sm']) { --blob-size: 14px; font-size: 11px; }
  :host([size='md']) { --blob-size: 20px; }
  :host([size='lg']) { --blob-size: 28px; font-size: var(--text-sm); }
  .tile {
    width: var(--blob-size);
    height: var(--blob-size);
    flex-shrink: 0;
    border-radius: calc(var(--blob-size) * 0.22);
    display: grid;
    place-items: center;
    background: color-mix(in oklch, var(--blob-hue) 14%, var(--surface-1));
    box-shadow: inset 0 0 0 1px color-mix(in oklch, var(--blob-hue) 20%, transparent);
    overflow: hidden;
  }
  :host([variant='glyph']) .tile {
    background: transparent;
    box-shadow: none;
  }
  .tile-inner {
    display: contents;
  }
  .tile svg {
    width: 72%;
    height: 72%;
    display: block;
  }
  :host([variant='glyph']) .tile svg {
    width: 100%;
    height: 100%;
  }
  .label {
    white-space: nowrap;
    text-transform: capitalize;
  }
  .label:empty { display: none; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.provider=u.provider,n.size=u.size,n.variant=u.variant,n.tileEl=G(null);let b=null;if(K0&&typeof K0.setup==="function"){if(i.scope=n,b=K0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("span");r.setAttribute("class","tile"),r.setAttribute("role","img"),m.push(T(r,"aria-label",()=>n.tileLabel()));let M=document.createElement("span");M.setAttribute("class","tile-inner"),F(n,"tileEl",M),r.appendChild(M),w.appendChild(r);let y=document.createElement("span");y.setAttribute("class","label");let l=document.createElement("slot");if(y.appendChild(l),w.appendChild(y),b!=null){let E=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(E){let t=new Proxy(n,{get(f,L){return f[L]},set(f,L,H){let C=f[L];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(E.mount){let f=E.mount(t,d);if(typeof f==="function")m.push(f)}if(E.unmount)m.push(()=>E.unmount(t))}}return n}});var Yd="ship-theme";function Mb(i){if(i==="light"||i==="dark")return i;if(typeof window>"u")return"light";try{let d=localStorage.getItem(Yd);if(d==="light"||d==="dark")return d}catch{}return window.matchMedia?.("(prefers-color-scheme: dark)")?.matches?"dark":"light"}function qd(i){if(typeof document>"u")return;document.documentElement.classList.toggle("dark",i==="dark")}var B0=N({scope:(i,u)=>{let d=i,m=h(()=>d.theme()==="dark");function n(){let b=d.theme()==="dark"?"light":"dark";d.theme.set(b);try{localStorage.setItem(Yd,b)}catch{}qd(b),u.dispatchEvent(new CustomEvent("sui-theme-change",{detail:{theme:b},bubbles:!0,composed:!0}))}return{isDark:m,toggle:n}},mount(i){let u=i,d=Mb(u.theme());if(d!==u.theme())u.theme.set(d);qd(d)}});var I9=V("sui-theme-toggle",{props:{theme:P.union(["light","dark","auto"],{default:"auto",reflect:!0})},shadow:!0,styles:`
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
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  button:hover { background: var(--muted); }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.theme=u.theme;let b=null;if(B0&&typeof B0.setup==="function"){if(i.scope=n,b=B0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("button");r.setAttribute("type","button"),r.setAttribute("aria-label","Toggle theme"),m.push(T(r,"aria-pressed",()=>n.isDark()?"true":"false")),r.setAttribute("title","Toggle theme"),m.push(p(r,"click",function(){let E=n.toggle;return typeof E==="function"?E.bind(void 0):()=>{}}(),{modifiers:[]}));let M=document.createTextNode("");r.appendChild(M),m.push(k(M,()=>n.isDark()?"\uD83C\uDF19":"☀"));let y=document.createElement("slot"),l=document.createTextNode("");if(y.appendChild(l),m.push(k(l,()=>n.isDark()?"Dark":"Light")),r.appendChild(y),w.appendChild(r),b!=null){let E=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(E){let t=new Proxy(n,{get(f,L){return f[L]},set(f,L,H){let C=f[L];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(E.mount){let f=E.mount(t,d);if(typeof f==="function")m.push(f)}if(E.unmount)m.push(()=>E.unmount(t))}}return n}});var Ud=/^([a-z]+(?:\([^)]+\))?!?):\s*(.+)$/,I0=N({scope:(i,u)=>{let d=i,m=h(()=>{let M=d.subject().match(Ud);return M?`${M[1]}:`:""}),n=h(()=>{let M=d.subject().match(Ud);return M?M[2]:d.subject()}),b=h(()=>(d.author()||"?").charAt(0).toUpperCase()),w=h(()=>d.active()?6:4.5);function r(){u.dispatchEvent(new CustomEvent("sui-select",{detail:{sha:d.sha()},bubbles:!0,composed:!0}))}return{scopePrefix:m,subjectRest:n,initial:b,dotR:w,emitSelect:r}},mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","button");if(!u.hasAttribute("tabindex"))u.setAttribute("tabindex","0");let m=(n)=>{if(n.key==="Enter"||n.key===" ")n.preventDefault(),u.dispatchEvent(new CustomEvent("sui-select",{detail:{sha:d.sha()},bubbles:!0,composed:!0}))};return u.addEventListener("keydown",m),()=>{u.removeEventListener("keydown",m)}}});var x9=V("sui-history-row",{props:{sha:P.string({default:"",reflect:!0}),subject:P.string({default:"",reflect:!0}),author:P.string({default:"",reflect:!0}),timestamp:P.string({default:"",reflect:!0}),additions:P.number({default:0,reflect:!0}),deletions:P.number({default:0,reflect:!0}),active:P.bool({default:!1,reflect:!0}),human:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    align-items: center;
    gap: 0;
    cursor: pointer;
    border-bottom: 1px solid var(--border-subtle);
    transition: background var(--duration-fast) ease;
    font-family: var(--font-body);
  }
  :host(:hover) { background: color-mix(in oklch, var(--muted) 50%, transparent); }
  :host([active]) { background: color-mix(in oklch, var(--primary) 8%, transparent); }

  .lane {
    width: 44px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    align-self: stretch;
  }
  .lane svg { display: block; }

  .avatar {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    font-size: 9px;
    font-weight: 600;
    color: var(--primary-foreground);
    background: color-mix(in oklch, var(--primary) 70%, var(--muted));
    text-transform: uppercase;
  }
  :host([human]) .avatar {
    background: conic-gradient(from 40deg, var(--status-blue, var(--info)), var(--accent));
  }

  .msg {
    flex: 1;
    min-width: 0;
    padding: 8px 6px 8px 10px;
  }
  .title {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--foreground);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    line-height: 1.35;
  }
  :host([active]) .title { color: var(--primary); }
  .scope { color: var(--primary); opacity: 0.85; }
  .meta {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-top: 2px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--muted-foreground);
  }

  .stats {
    display: flex;
    gap: 6px;
    padding-right: 14px;
    flex-shrink: 0;
    font-family: var(--font-mono);
    font-size: 10.5px;
  }
  .add { color: var(--success); }
  .del { color: var(--destructive); }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.sha=u.sha,n.subject=u.subject,n.author=u.author,n.timestamp=u.timestamp,n.additions=u.additions,n.deletions=u.deletions,n.active=u.active,n.human=u.human;let b=null;if(I0&&typeof I0.setup==="function"){if(i.scope=n,b=I0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(K)=>I({anchor:K,test:()=>{if(n.scopePrefix())return 0;return-1},branches:[()=>{let B=document.createDocumentFragment(),A=[],a=document.createElement("span");a.setAttribute("class","scope");let S=document.createTextNode("");return a.appendChild(S),A.push(k(S,()=>n.scopePrefix())),B.appendChild(a),{nodes:Array.from(B.childNodes),cleanup:()=>{for(let v of A)try{v()}catch(R){}}}}]}),r=(K)=>I({anchor:K,test:()=>{if(n.additions()>0)return 0;return-1},branches:[()=>{let B=document.createDocumentFragment(),A=[],a=document.createElement("span");a.setAttribute("class","add");let S=document.createTextNode("+");a.appendChild(S);let v=document.createTextNode("");return a.appendChild(v),A.push(k(v,()=>n.additions())),B.appendChild(a),{nodes:Array.from(B.childNodes),cleanup:()=>{for(let R of A)try{R()}catch(x){}}}}]}),M=(K)=>I({anchor:K,test:()=>{if(n.deletions()>0)return 0;return-1},branches:[()=>{let B=document.createDocumentFragment(),A=[],a=document.createElement("span");a.setAttribute("class","del");let S=document.createTextNode("−");a.appendChild(S);let v=document.createTextNode("");return a.appendChild(v),A.push(k(v,()=>n.deletions())),B.appendChild(a),{nodes:Array.from(B.childNodes),cleanup:()=>{for(let R of A)try{R()}catch(x){}}}}]}),y=d.shadowRoot??d;q(y);let l=document.createElement("div");l.setAttribute("class","lane"),m.push(p(l,"click",function(){let K=n.emitSelect;return typeof K==="function"?K.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("svg");E.setAttribute("width","44"),E.setAttribute("height","46"),E.setAttribute("aria-hidden","true");let t=document.createElement("line");t.setAttribute("x1","22"),t.setAttribute("y1","0"),t.setAttribute("x2","22"),t.setAttribute("y2","46"),t.setAttribute("stroke","var(--primary)"),t.setAttribute("stroke-width","1.5"),t.setAttribute("opacity","0.45"),E.appendChild(t);let f=document.createElement("circle");f.setAttribute("cx","22"),f.setAttribute("cy","23"),m.push(T(f,"r",()=>n.dotR())),m.push(T(f,"fill",()=>n.active()?"var(--primary)":"var(--background)")),f.setAttribute("stroke","var(--primary)"),f.setAttribute("stroke-width","2"),E.appendChild(f),l.appendChild(E),y.appendChild(l);let L=document.createElement("div");L.setAttribute("class","avatar");let H=document.createTextNode("");L.appendChild(H),m.push(k(H,()=>n.initial())),y.appendChild(L);let C=document.createElement("div");C.setAttribute("class","msg"),m.push(p(C,"click",function(){let K=n.emitSelect;return typeof K==="function"?K.bind(void 0):()=>{}}(),{modifiers:[]}));let z=document.createElement("div");z.setAttribute("class","title");let j=document.createComment("if");z.appendChild(j),m.push(w(j));let g=document.createTextNode("");z.appendChild(g),m.push(k(g,()=>n.subjectRest())),C.appendChild(z);let Z=document.createElement("div");Z.setAttribute("class","meta");let D=document.createElement("span"),J=document.createTextNode("");D.appendChild(J),m.push(k(J,()=>n.sha())),Z.appendChild(D);let X=document.createElement("span"),W=document.createTextNode("");X.appendChild(W),m.push(k(W,()=>n.timestamp())),Z.appendChild(X),C.appendChild(Z),y.appendChild(C);let Q=document.createElement("div");Q.setAttribute("class","stats");let Y=document.createComment("if");Q.appendChild(Y),m.push(r(Y));let O=document.createComment("if");if(Q.appendChild(O),m.push(M(O)),y.appendChild(Q),b!=null){let K=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(K){let B=new Proxy(n,{get(A,a){return A[a]},set(A,a,S){let v=A[a];if(v!=null&&typeof v.set==="function"){if(typeof S==="function")v.set(()=>S);else v.set(S);return!0}return!1}});if(K.mount){let A=K.mount(B,d);if(typeof A==="function")m.push(A)}if(K.unmount)m.push(()=>K.unmount(B))}}return n}});var R0=N({scope:(i,u)=>{let d=i,m=e(null),n=e(null),b=e(null),w=G(!1),r=h(()=>d.mode()==="sign-up"?"Create your account":"Welcome back"),M=h(()=>{if(d.loading())return d.mode()==="sign-up"?"Creating account…":"Signing in…";return d.mode()==="sign-up"?"Create account":"Sign in"}),y=h(()=>d.mode()==="sign-up"?"sign-in":"sign-up"),l=h(()=>y()==="sign-up"?"Create an account":"Sign in instead"),E=h(()=>d.mode()==="sign-up"?"Already have an account?":"New here?"),t=h(()=>d.mode()==="sign-up"?"new-password":"current-password");function f(j){if(j===d.mode())return;d.mode.set(j),u.dispatchEvent(new CustomEvent("sui-mode-change",{detail:{mode:j},bubbles:!0,composed:!0}))}function L(j){let g=j.detail;if(!g||typeof g.name!=="string")return;let Z=g.name==="sign-up"?"sign-up":"sign-in";f(Z)}function H(j){j.preventDefault();let g=j.currentTarget??n();if(!g)return;let Z=g.elements.namedItem("email")?.value??"",D=g.elements.namedItem("password")?.value??"";u.dispatchEvent(new CustomEvent("sui-auth-submit",{detail:{mode:d.mode(),email:Z,password:D},bubbles:!0,composed:!0}))}function C(){f(y())}function z(){if(d.open())d.open.set(!1)}return{dialogRef:m,formRef:n,providersSlot:b,hasProviders:w,defaultTitle:r,submitLabel:M,otherMode:y,switchLabel:l,switchPrompt:E,passwordAutocomplete:t,onTabChange:L,onSubmit:H,onSwitchClick:C,onDialogClose:z}},mount(i,u){let d=i,m=()=>{let r=d.providersSlot();if(!r){d.hasProviders.set(!1);return}let M=r.assignedNodes({flatten:!0}).some((y)=>{if(y.nodeType===Node.TEXT_NODE)return(y.textContent||"").trim().length>0;return y.nodeType===Node.ELEMENT_NODE});d.hasProviders.set(M)},n=U(()=>{let r=d.providersSlot();if(!r)return;return r.addEventListener("slotchange",m),m(),()=>r.removeEventListener("slotchange",m)}),b=U(()=>{let r=d.open(),M=d.dialogRef();if(!M)return;if(r&&typeof M.showModal==="function"){if(!M.hasAttribute("open"))M.showModal()}else if(!r&&typeof M.close==="function"){if(M.hasAttribute("open"))M.close()}}),w=u;return w.show=()=>d.open.set(!0),w.showModal=()=>d.open.set(!0),w.close=()=>d.open.set(!1),()=>{n(),b()}}});var p9=V("sui-auth-dialog",{props:{open:P.bool({default:!1,reflect:!0}),mode:P.union(["sign-in","sign-up"],{default:"sign-in",reflect:!0}),loading:P.bool({default:!1,reflect:!0}),backdrop:P.union(["default","blur","solid"],{default:"blur",reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
  :host([open]) { display: contents; }

  .body {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 4px 0 0;
    font-family: var(--font-body);
    color: var(--foreground);
  }

  .providers {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  ::slotted(*) { width: 100%; }

  .or {
    display: flex;
    align-items: center;
    gap: 12px;
    color: var(--muted-foreground);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .or::before, .or::after {
    content: '';
    flex: 1;
    height: 1px;
    background: color-mix(in oklch, var(--border) 55%, transparent);
  }

  form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 12px;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 500;
  }
  input {
    height: 38px;
    padding: 0 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--card);
    color: var(--foreground);
    font: inherit;
    font-size: 14px;
    transition: border-color 120ms ease, box-shadow 120ms ease;
  }
  input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--tone-accent-ring, var(--ring));
  }

  .submit-row {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: 4px;
  }
  .submit-row sui-button { width: 100%; }

  .terms {
    color: var(--muted-foreground);
    font-size: 11.5px;
    text-align: center;
    line-height: 1.5;
    margin: 0;
  }

  .switch {
    color: var(--muted-foreground);
    font-size: 12.5px;
    text-align: center;
    margin: 0;
  }
  .switch button {
    appearance: none;
    background: transparent;
    border: 0;
    padding: 0;
    color: var(--accent);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.mode=u.mode,n.loading=u.loading,n.backdrop=u.backdrop,n.dialogRef=G(null),n.formRef=G(null),n.providersSlot=G(null);let b=null;if(R0&&typeof R0.setup==="function"){if(i.scope=n,b=R0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(x)=>I({anchor:x,test:()=>{if(n.hasProviders())return 0;return-1},branches:[()=>{let s=document.createDocumentFragment(),$=[],nn=document.createElement("div");nn.setAttribute("class","or");let tn=document.createTextNode("or");return nn.appendChild(tn),s.appendChild(nn),{nodes:Array.from(s.childNodes),cleanup:()=>{for(let dn of $)try{dn()}catch(Ln){}}}}]}),r=d.shadowRoot??d;q(r);let M=document.createElement("sui-dialog");M.setAttribute("part","dialog"),M.setAttribute("size","sm"),m.push(T(M,"backdrop",()=>n.backdrop())),F(n,"dialogRef",M),m.push(p(M,"sui-close",function(){let x=n.onDialogClose;return typeof x==="function"?x.bind(void 0):()=>{}}(),{modifiers:[]}));let y=document.createElement("div");y.setAttribute("slot","header");let l=document.createElement("slot");l.setAttribute("name","header");let E=document.createTextNode("");l.appendChild(E),m.push(k(E,()=>n.defaultTitle())),y.appendChild(l),M.appendChild(y);let t=document.createElement("div");t.setAttribute("class","body");let f=document.createElement("sui-tabs");f.setAttribute("variant","pill"),m.push(T(f,"active",()=>n.mode())),m.push(p(f,"sui-tab-change",function(){let x=n.onTabChange;return typeof x==="function"?x.bind(void 0):()=>{}}(),{modifiers:[]}));let L=document.createElement("sui-tab");L.setAttribute("name","sign-in"),L.setAttribute("label","Sign in"),f.appendChild(L);let H=document.createElement("sui-tab");H.setAttribute("name","sign-up"),H.setAttribute("label","Sign up"),f.appendChild(H),t.appendChild(f);let C=document.createElement("div");C.setAttribute("class","providers");let z=document.createElement("slot");z.setAttribute("name","providers"),F(n,"providersSlot",z),C.appendChild(z),t.appendChild(C);let j=document.createComment("if");t.appendChild(j),m.push(w(j));let g=document.createElement("form");g.setAttribute("part","form"),m.push(p(g,"submit",function(){let x=n.onSubmit;return typeof x==="function"?x.bind(void 0):()=>{}}(),{modifiers:[]})),g.setAttribute("novalidate",""),F(n,"formRef",g);let Z=document.createElement("label"),D=document.createTextNode(`
        Email
        `);Z.appendChild(D);let J=document.createElement("input");J.setAttribute("name","email"),J.setAttribute("type","email"),J.setAttribute("autocomplete","email"),J.setAttribute("inputmode","email"),J.setAttribute("required",""),J.setAttribute("placeholder","you@company.com"),Z.appendChild(J),g.appendChild(Z);let X=document.createElement("label"),W=document.createTextNode(`
        Password
        `);X.appendChild(W);let Q=document.createElement("input");Q.setAttribute("name","password"),Q.setAttribute("type","password"),m.push(T(Q,"autocomplete",()=>n.passwordAutocomplete())),Q.setAttribute("required",""),Q.setAttribute("minlength","8"),X.appendChild(Q),g.appendChild(X);let Y=document.createElement("div");Y.setAttribute("class","submit-row");let O=document.createElement("sui-button");O.setAttribute("part","submit"),O.setAttribute("tone","accent"),O.setAttribute("variant","solid"),O.setAttribute("size","md"),O.setAttribute("type","submit"),m.push(T(O,"loading",()=>n.loading())),m.push(T(O,"disabled",()=>n.loading()));let K=document.createTextNode("");O.appendChild(K),m.push(k(K,()=>n.submitLabel())),Y.appendChild(O);let B=document.createElement("p");B.setAttribute("class","terms");let A=document.createElement("slot");A.setAttribute("name","terms"),B.appendChild(A),Y.appendChild(B),g.appendChild(Y),t.appendChild(g);let a=document.createElement("p");a.setAttribute("class","switch");let S=document.createTextNode("");a.appendChild(S),m.push(k(S,()=>n.switchPrompt()));let v=document.createElement("button");v.setAttribute("type","button"),m.push(p(v,"click",function(){let x=n.onSwitchClick;return typeof x==="function"?x.bind(void 0):()=>{}}(),{modifiers:[]}));let R=document.createTextNode("");if(v.appendChild(R),m.push(k(R,()=>n.switchLabel())),a.appendChild(v),t.appendChild(a),M.appendChild(t),r.appendChild(M),b!=null){let x=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(x){let s=new Proxy(n,{get($,nn){return $[nn]},set($,nn,tn){let dn=$[nn];if(dn!=null&&typeof dn.set==="function"){if(typeof tn==="function")dn.set(()=>tn);else dn.set(tn);return!0}return!1}});if(x.mount){let $=x.mount(s,d);if(typeof $==="function")m.push($)}if(x.unmount)m.push(()=>x.unmount(s))}}return n}});var k0=N({scope:(i,u)=>{let d=i,m=G(!1),n=h(()=>d.empty()||d.files()===0),b=h(()=>{let l=d.files();return`${l} file${l===1?"":"s"} changed`}),w=h(()=>m()?"chevrons-up-down":"chevrons-down-up"),r=h(()=>m()?"Expand all":"Collapse all"),M=h(()=>m()?"Expand all files":"Collapse all files");function y(){let l=!m();m.set(l),u.querySelectorAll("sui-git-diff-file").forEach((t)=>{if(l)t.setAttribute("collapsed","");else t.removeAttribute("collapsed")})}return{allCollapsed:m,showEmpty:n,filesLabel:b,toggleIcon:w,toggleLabel:r,toggleTitle:M,toggleAll:y}},mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","region");let m=U(()=>{let n=d.files(),b=d.additions(),w=d.deletions();u.setAttribute("aria-label",`Diff: ${n} file${n===1?"":"s"} changed, ${b} additions, ${w} deletions`)});return()=>{m()}}});var dw=V("sui-git-diff",{props:{files:P.number({default:0,reflect:!0}),additions:P.number({default:0,reflect:!0}),deletions:P.number({default:0,reflect:!0}),empty:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    min-height: 0;
    font-family: var(--font-body);
    background: var(--background);
    color: var(--foreground);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }
  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 16px;
    border-bottom: 1px solid var(--border-subtle);
    background: color-mix(in oklch, var(--card) 50%, transparent);
    flex-shrink: 0;
  }
  .count { font-size: 12px; color: var(--muted-foreground); }
  .adds { font-size: 11px; color: var(--success); }
  .dels { font-size: 11px; color: var(--destructive); }
  .spacer { flex: 1; }
  .toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: var(--radius-md);
    background: transparent;
    border: 0;
    font-family: inherit;
    font-size: 11px;
    color: var(--muted-foreground);
    cursor: pointer;
    transition: background var(--duration-fast) ease, color var(--duration-fast) ease;
  }
  .toggle:hover { background: var(--muted); color: var(--foreground); }
  .body { flex: 1; overflow: auto; }
  :host([empty]) .toolbar { display: none; }
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 48px 16px;
    color: var(--muted-foreground);
    font-size: 13px;
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.files=u.files,n.additions=u.additions,n.deletions=u.deletions,n.empty=u.empty;let b=null;if(k0&&typeof k0.setup==="function"){if(i.scope=n,b=k0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(E)=>I({anchor:E,test:()=>{if(n.additions()>0)return 0;return-1},branches:[()=>{let t=document.createDocumentFragment(),f=[],L=document.createElement("span");L.setAttribute("class","adds");let H=document.createTextNode("+");L.appendChild(H);let C=document.createTextNode("");return L.appendChild(C),f.push(k(C,()=>n.additions())),t.appendChild(L),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let z of f)try{z()}catch(j){}}}}]}),r=(E)=>I({anchor:E,test:()=>{if(n.deletions()>0)return 0;return-1},branches:[()=>{let t=document.createDocumentFragment(),f=[],L=document.createElement("span");L.setAttribute("class","dels");let H=document.createTextNode("-");L.appendChild(H);let C=document.createTextNode("");return L.appendChild(C),f.push(k(C,()=>n.deletions())),t.appendChild(L),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let z of f)try{z()}catch(j){}}}}]}),M=(E)=>I({anchor:E,test:()=>{if(n.showEmpty())return 0;return 1},branches:[()=>{let t=document.createDocumentFragment(),f=[],L=document.createElement("div");L.setAttribute("class","empty");let H=document.createTextNode("No changes to display");return L.appendChild(H),t.appendChild(L),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let C of f)try{C()}catch(z){}}}},()=>{let t=document.createDocumentFragment(),f=[],L=document.createElement("div");L.setAttribute("class","toolbar");let H=document.createElement("span");H.setAttribute("class","count");let C=document.createTextNode("");H.appendChild(C),f.push(k(C,()=>n.filesLabel())),L.appendChild(H);let z=document.createComment("if");L.appendChild(z),f.push(w(z));let j=document.createComment("if");L.appendChild(j),f.push(r(j));let g=document.createElement("span");g.setAttribute("class","spacer"),L.appendChild(g);let Z=document.createElement("button");Z.setAttribute("class","toggle"),Z.setAttribute("type","button"),f.push(p(Z,"click",function(){let Q=n.toggleAll;return typeof Q==="function"?Q.bind(void 0):()=>{}}(),{modifiers:[]})),f.push(T(Z,"title",()=>n.toggleTitle()));let D=document.createElement("sui-icon");D.setAttribute("size","sm"),f.push(T(D,"name",()=>n.toggleIcon())),Z.appendChild(D);let J=document.createTextNode("");Z.appendChild(J),f.push(k(J,()=>n.toggleLabel())),L.appendChild(Z),t.appendChild(L);let X=document.createElement("div");X.setAttribute("class","body");let W=document.createElement("slot");return X.appendChild(W),t.appendChild(X),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let Q of f)try{Q()}catch(Y){}}}}]}),y=d.shadowRoot??d;q(y);let l=document.createComment("if");if(y.appendChild(l),m.push(M(l)),b!=null){let E=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(E){let t=new Proxy(n,{get(f,L){return f[L]},set(f,L,H){let C=f[L];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(E.mount){let f=E.mount(t,d);if(typeof f==="function")m.push(f)}if(E.unmount)m.push(()=>E.unmount(t))}}return n}});var S0=N({scope:(i)=>{let u=i,d=h(()=>u.collapsed()?"chevron-right":"chevron-down"),m=h(()=>u.collapsed()?"false":"true");function n(){u.collapsed.set(!u.collapsed())}return{chevIcon:d,expandedLabel:m,toggle:n}},mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","group");let m=U(()=>{let n=d.path(),b=d.additions(),w=d.deletions();u.setAttribute("aria-label",`${n}: +${b} -${w}`)});return()=>{m()}}});var lw=V("sui-git-diff-file",{props:{path:P.string({default:"",reflect:!0}),additions:P.number({default:0,reflect:!0}),deletions:P.number({default:0,reflect:!0}),collapsed:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    border-bottom: 1px solid color-mix(in oklch, var(--border) 35%, transparent);
  }
  .header {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--border-subtle);
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
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--foreground);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .stats { display: inline-flex; align-items: center; gap: 8px; flex-shrink: 0; }
  .adds, .dels {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    font-size: 10px;
    font-family: var(--font-mono);
  }
  .adds { color: var(--success); }
  .dels { color: var(--destructive); }
  .body { display: block; }
  :host([collapsed]) .body { display: none; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.path=u.path,n.additions=u.additions,n.deletions=u.deletions,n.collapsed=u.collapsed;let b=null;if(S0&&typeof S0.setup==="function"){if(i.scope=n,b=S0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(g)=>I({anchor:g,test:()=>{if(n.additions()>0)return 0;return-1},branches:[()=>{let Z=document.createDocumentFragment(),D=[],J=document.createElement("span");J.setAttribute("class","adds");let X=document.createElement("sui-icon");X.setAttribute("size","sm"),X.setAttribute("name","plus"),J.appendChild(X);let W=document.createTextNode("");return J.appendChild(W),D.push(k(W,()=>n.additions())),Z.appendChild(J),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let Q of D)try{Q()}catch(Y){}}}}]}),r=(g)=>I({anchor:g,test:()=>{if(n.deletions()>0)return 0;return-1},branches:[()=>{let Z=document.createDocumentFragment(),D=[],J=document.createElement("span");J.setAttribute("class","dels");let X=document.createElement("sui-icon");X.setAttribute("size","sm"),X.setAttribute("name","minus"),J.appendChild(X);let W=document.createTextNode("");return J.appendChild(W),D.push(k(W,()=>n.deletions())),Z.appendChild(J),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let Q of D)try{Q()}catch(Y){}}}}]}),M=d.shadowRoot??d;q(M);let y=document.createElement("button");y.setAttribute("class","header"),y.setAttribute("type","button"),m.push(p(y,"click",function(){let g=n.toggle;return typeof g==="function"?g.bind(void 0):()=>{}}(),{modifiers:[]})),m.push(T(y,"aria-expanded",()=>n.expandedLabel()));let l=document.createElement("sui-icon");l.setAttribute("class","chev"),l.setAttribute("size","sm"),m.push(T(l,"name",()=>n.chevIcon())),y.appendChild(l);let E=document.createElement("sui-icon");E.setAttribute("class","fileicon"),E.setAttribute("size","sm"),E.setAttribute("name","file-code"),y.appendChild(E);let t=document.createElement("span");t.setAttribute("class","path");let f=document.createTextNode("");t.appendChild(f),m.push(k(f,()=>n.path())),y.appendChild(t);let L=document.createElement("span");L.setAttribute("class","stats");let H=document.createComment("if");L.appendChild(H),m.push(w(H));let C=document.createComment("if");L.appendChild(C),m.push(r(C)),y.appendChild(L),M.appendChild(y);let z=document.createElement("div");z.setAttribute("class","body");let j=document.createElement("slot");if(z.appendChild(j),M.appendChild(z),b!=null){let g=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(g){let Z=new Proxy(n,{get(D,J){return D[J]},set(D,J,X){let W=D[J];if(W!=null&&typeof W.set==="function"){if(typeof X==="function")W.set(()=>X);else W.set(X);return!0}return!1}});if(g.mount){let D=g.mount(Z,d);if(typeof D==="function")m.push(D)}if(g.unmount)m.push(()=>g.unmount(Z))}}return n}});var Mw=V("sui-git-diff-hunk",{props:{header:P.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: block;
    padding: 2px 12px;
    background: var(--tone-info-bg);
    border-top: 1px solid var(--tone-info-bg);
    border-bottom: 1px solid var(--tone-info-bg);
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--info);
    user-select: none;
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.header=u.header;let b=d.shadowRoot??d;q(b);let w=document.createTextNode(`

`);b.appendChild(w);let r=document.createTextNode(`

`);b.appendChild(r);let M=document.createTextNode("");return b.appendChild(M),m.push(k(M,()=>n.header())),n}});var a0=N({scope:(i,u)=>{let d=i,m=d.new,n=d.old,b=d.type,w=h(()=>n?.()??""),r=h(()=>m?.()??""),M=h(()=>{let l=b?.();return l==="add"?"+":l==="del"?"-":" "});function y(l){l.stopPropagation(),u.dispatchEvent(new CustomEvent("sui-comment",{detail:{old:n?.()??"",new:m?.()??""},bubbles:!0,composed:!0}))}return{oldCol:w,newCol:r,sigChar:M,emitComment:y}},mount(i,u){let d=i,m=d.new,n=d.old,b=d.type,w=U(()=>{let r=b?.()??"context",M=m?.()??"",y=n?.()??"",l=r==="add"?"added":r==="del"?"removed":"context";u.setAttribute("aria-label",`line ${M||y} ${l}`)});return()=>{w()}}});var jw=V("sui-git-diff-line",{props:{type:P.union(["add","del","context"],{default:"context",reflect:!0}),old:P.string({default:"",reflect:!0}),new:P.string({default:"",reflect:!0}),"comment-count":P.number({default:0,reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.45;
    position: relative;
  }
  :host([type='add']) { background: var(--tone-success-bg); }
  :host([type='del']) { background: var(--tone-destructive-bg); }
  :host([comment-count]) { border-left: 2px solid var(--accent); }
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
    padding-right: 28px;
    white-space: pre-wrap;
    word-break: break-all;
    color: var(--muted-foreground);
  }
  :host([type='add']) .sig,
  :host([type='add']) .content { color: var(--success); }
  :host([type='del']) .sig,
  :host([type='del']) .content { color: var(--destructive); }

  .comment-btn {
    position: absolute;
    right: 4px;
    top: 50%;
    transform: translateY(-50%);
    width: 18px;
    height: 18px;
    border-radius: var(--radius-sm);
    background: var(--accent);
    color: var(--accent-foreground);
    border: 0;
    display: grid;
    place-items: center;
    cursor: pointer;
    opacity: 0;
    transition: opacity var(--duration-fast) ease;
    font-size: 13px;
    line-height: 1;
    font-family: var(--font-body);
  }
  :host(:hover) .comment-btn { opacity: 1; }
  :host([comment-count]) .comment-btn { opacity: 0.7; }
  :host([comment-count]:hover) .comment-btn { opacity: 1; }
  .comment-btn:focus-visible {
    outline: 2px solid var(--ring-color, var(--ring));
    outline-offset: 1px;
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.type=u.type,n.old=u.old,n.new=u.new,n.commentCount=u.commentCount;let b=null;if(a0&&typeof a0.setup==="function"){if(i.scope=n,b=a0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;q(w);let r=document.createElement("span");r.setAttribute("class","num");let M=document.createTextNode("");r.appendChild(M),m.push(k(M,()=>n.oldCol())),w.appendChild(r);let y=document.createElement("span");y.setAttribute("class","num");let l=document.createTextNode("");y.appendChild(l),m.push(k(l,()=>n.newCol())),w.appendChild(y);let E=document.createElement("span");E.setAttribute("class","sig");let t=document.createTextNode("");E.appendChild(t),m.push(k(t,()=>n.sigChar())),w.appendChild(E);let f=document.createElement("span");f.setAttribute("class","content");let L=document.createElement("slot");f.appendChild(L),w.appendChild(f);let H=document.createElement("button");H.setAttribute("class","comment-btn"),H.setAttribute("type","button"),H.setAttribute("aria-label","Comment on this line"),H.setAttribute("title","Comment on this line"),m.push(p(H,"click",function(){let z=n.emitComment;return typeof z==="function"?z.bind(void 0):()=>{}}(),{modifiers:[]}));let C=document.createTextNode("+");if(H.appendChild(C),w.appendChild(H),b!=null){let z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(z){let j=new Proxy(n,{get(g,Z){return g[Z]},set(g,Z,D){let J=g[Z];if(J!=null&&typeof J.set==="function"){if(typeof D==="function")J.set(()=>D);else J.set(D);return!0}return!1}});if(z.mount){let g=z.mount(j,d);if(typeof g==="function")m.push(g)}if(z.unmount)m.push(()=>z.unmount(j))}}return n}});
