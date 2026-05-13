var o0=Symbol.for("sui:error-boundary");function e0(i,u){i[o0]=u}function s0(i){delete i[o0]}function gn(i){let u=i;while(u){if(u instanceof Element){let n=u[o0];if(n)return n}let d=u.parentNode;if(d){u=d;continue}let m=u;if(m.host){u=m.host;continue}u=null}return null}function Xn(i,u,d){try{return d()}catch(m){let n=gn(i);if(n){n.catch(m,u);return}console.error("[sui] uncaught throw (no error boundary):",m);return}}var Mn=null,Hn=null,Qd=0,Vd=new Set,Kn=[];function p0(i,u){Kn.push(i);try{return u()}finally{Kn.pop()}}function Yd(){return Kn.length>0?Kn[Kn.length-1]:null}function Tu(i){if(Mn===null)return;if(i.has(Mn))return;let u=Mn;if(i.add(u),Hn!==null)Hn.push(()=>i.delete(u))}function J(i){let u=i,d=new Set,m=()=>{return Tu(d),u};return m.set=(n)=>{let b=typeof n==="function"?n(u):n;if(Object.is(b,u))return;if(u=b,Qd>0)for(let w of d)Vd.add(w);else for(let w of[...d])w()},m.__subs=d,m.__peek=()=>u,m}function s(i){let u,d=!0,m=new Set,n=[],b=()=>{if(d)return;if(d=!0,m.size===0)return;let M=u;if(w(),Object.is(M,u))return;for(let y of[...m])y()},w=()=>{for(let L of n)L();n=[];let M=Mn,y=Hn;Mn=b,Hn=n;try{u=i()}finally{Mn=M,Hn=y}d=!1},E=()=>{if(Tu(m),d)w();return u};return E.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},E.__subs=m,E.__peek=()=>{if(d)w();return u},E}function U(i){let u=void 0,d=!0,m=[],n=Yd(),b=()=>{if(!d)return;if(u){try{u()}catch(M){console.error("[sui] effect cleanup threw:",M)}u=void 0}for(let M of m)M();m=[];let w=Mn,E=Hn;Mn=b,Hn=m;try{u=i()}catch(M){let y=n?gn(n):null;if(y)y.catch(M,"effect");else console.error("[sui] effect threw:",M)}finally{Mn=w,Hn=E}};return b(),()=>{d=!1;for(let w of m)w();if(m=[],u){try{u()}catch(w){console.error("[sui] effect cleanup threw:",w)}u=void 0}}}var pn=null;function Bn(i,u){let d=pn;pn=i;try{return u()}finally{pn=d}}function yn(i){let u=pn;if(!u)return;u.cleanup(i)}function In(){let i=[],u=!1;return{cleanup(m){if(u){try{m()}catch(n){console.error("[sui/scope] late cleanup threw:",n)}return}i.push(m)},dispose(){if(u)return;u=!0;for(let m=i.length-1;m>=0;m--)try{i[m]()}catch(n){console.error("[sui/scope] cleanup threw:",n)}i.length=0},get disposed(){return u}}}var qd=Symbol("sui.shared");function Ju(i){return!!i?.[qd]}function c0(i,u,d){i[`__sui_ctx_${u}`]=d}function _0(i){let u={};for(let d in i)if(Object.prototype.hasOwnProperty.call(i,d)&&Ju(i[d]))u[d]=i[d];return u}function e(i=null){return J(null)}function A(i,u,d){let m=i[u];if(typeof m==="function"&&typeof m.set==="function")return m.set(d),m;let n=J(d);return i[u]=n,n}function dn(i,u,d){let m=()=>{if(i.assignedNodes({flatten:!0}).some(Ud))u.removeAttribute(d);else u.setAttribute(d,"")};return i.addEventListener("slotchange",m),m(),()=>{i.removeEventListener("slotchange",m)}}function Ud(i){if(i.nodeType===Node.TEXT_NODE)return(i.textContent??"").trim().length>0;if(i.nodeType===Node.ELEMENT_NODE){let u=i;return(u.textContent??"").trim().length>0||u.childElementCount>0}return!1}function Rn(i,u,d={}){if(typeof u==="string"){let m=i.tagName.toLowerCase(),n=i.namespaceURI==="http://www.w3.org/2000/svg"?i.ownerDocument.createElementNS("http://www.w3.org/2000/svg",m):i.ownerDocument.createElement(m);n.innerHTML=u,Gu(i,n,d);return}Xu(i,u,d)}function Xu(i,u,d){if(d.skip?.(i))return;if(i.tagName!==u.tagName){i.replaceWith(u.cloneNode(!0));return}Nd(i,u),Gu(i,u,d)}function Nd(i,u){let d=Array.from(i.attributes);for(let n of d)if(!u.hasAttribute(n.name))i.removeAttribute(n.name);let m=Array.from(u.attributes);for(let n of m)if(i.getAttribute(n.name)!==n.value)i.setAttribute(n.name,n.value);if(i instanceof HTMLInputElement&&u instanceof HTMLInputElement){if(u.hasAttribute("value")&&i.value!==u.value)i.value=u.value;if(i.checked!==u.checked)i.checked=u.checked}}function Gu(i,u,d){let m=new Map;for(let E of Array.from(i.children))if(E.id)m.set(E.id,E);let n=new Set;for(let E of Array.from(u.children))if(E.id)n.add(E.id);let b=Array.from(u.childNodes),w=i.firstChild;for(let E of b){while(w&&w.nodeType===Node.ELEMENT_NODE&&w.id&&n.has(w.id)&&!(E.nodeType===Node.ELEMENT_NODE&&E.id===w.id))w=w.nextSibling;let M=null;if(E.nodeType===Node.ELEMENT_NODE){let y=E.id;if(y&&m.has(y))M=m.get(y),m.delete(y)}if(!M&&w&&Ad(w,E))M=w;if(M){if(M===w)w=w.nextSibling;else i.insertBefore(M,w);Fd(M,E,i,d)}else i.insertBefore(E.cloneNode(!0),w)}while(w){let E=w.nextSibling;i.removeChild(w),w=E}}function Ad(i,u){if(i.nodeType!==u.nodeType)return!1;if(i.nodeType===Node.ELEMENT_NODE)return i.tagName===u.tagName;return!0}function Fd(i,u,d,m){if(i.nodeType!==u.nodeType){d.replaceChild(u.cloneNode(!0),i);return}if(i.nodeType===Node.TEXT_NODE||i.nodeType===Node.COMMENT_NODE){if(i.nodeValue!==u.nodeValue)i.nodeValue=u.nodeValue;return}if(i.nodeType===Node.ELEMENT_NODE)Xu(i,u,m)}function a(i,u){let d;return U(()=>{let m=u(),n=m==null?"":String(m);if(n===d)return;if(d===void 0&&i.textContent===n){d=n;return}i.textContent=n,d=n})}function W(i,u,d){let m,n=!1;return U(()=>{let b=d();if(n&&b===m)return;let w=b==null||b===!1||b===""?null:b===!0?"":String(b);if(n||i.getAttribute(u)!==w)if(w===null)i.removeAttribute(u);else i.setAttribute(u,w);m=b,n=!0})}function Cn(i,u,d){let m;return U(()=>{let n=!!d();if(n===m)return;if(m===void 0&&n===i.classList.contains(u)){m=n;return}if(n)i.classList.add(u);else i.classList.remove(u);m=n})}function Gn(i,u,d){return U(()=>{i[u]=d()})}function Pn(i,u,d){let m=U(()=>{let b=u();if(i.value!==b)i.value=b}),n=()=>{let b=i.value;if(u.set(b),d)d(b)};return i.addEventListener("input",n),()=>{m(),i.removeEventListener("input",n)}}function jn(i,u){let d=U(()=>{let n=u();if(i.checked!==n)i.checked=n}),m=()=>u.set(i.checked);return i.addEventListener("change",m),()=>{d(),i.removeEventListener("change",m)}}function mn(i,u){let m=i.namespaceURI==="http://www.w3.org/2000/svg",n=i.tagName.toLowerCase();return U(()=>{let b=u(),w=m?i.ownerDocument.createElementNS("http://www.w3.org/2000/svg",n):i.ownerDocument.createElement(n);w.innerHTML=b,Rn(i,w)})}function nn(i,u,d,m={}){let n=m.modifiers??[],b,w=!0,E=(y)=>{for(let f of n)switch(f.kind){case"preventDefault":y.preventDefault();break;case"stopPropagation":y.stopPropagation();break;case"stopImmediatePropagation":y.stopImmediatePropagation();break;case"self":if(y.target!==i)return;break;case"trusted":if(!y.isTrusted)return;break;case"key":{if(!Wu(y,f.arg??""))return;break}}let L=n.find((f)=>f.kind==="debounce"),l=n.find((f)=>f.kind==="throttle");if(L){let f=Number(L.arg)||0;if(b!==void 0)clearTimeout(b);b=setTimeout(()=>{b=void 0,d(y)},f);return}if(l){let f=Number(l.arg)||0;if(!w)return;w=!1,d(y),b=setTimeout(()=>{w=!0},f);return}d(y)},M={};if(m.capture)M.capture=!0;if(m.passive)M.passive=!0;if(m.once)M.once=!0;return i.addEventListener(u,E,M),()=>{if(i.removeEventListener(u,E,M),b!==void 0)clearTimeout(b)}}function Wu(i,u){let d=u.split("+").map((L)=>L.toLowerCase()),m=d.pop()??"",n=d.includes("meta")||d.includes("cmd"),b=d.includes("ctrl")||d.includes("control"),w=d.includes("shift"),E=d.includes("alt")||d.includes("option"),M=d.includes("mod"),y=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(n&&!i.metaKey)return!1;if(b&&!i.ctrlKey)return!1;if(w&&!i.shiftKey)return!1;if(E&&!i.altKey)return!1;if(M&&!(y?i.metaKey:i.ctrlKey))return!1;return i.key.toLowerCase()===m}function Qu(i,u,d){i.dispatchEvent(new CustomEvent(u,{detail:d,bubbles:!0,composed:!0}))}function Vu(i,u){let d=i,m=Object.keys(u);for(let n of m)d[n]=u[n];yn(()=>{for(let n of m)if(d[n]===u[n])try{delete d[n]}catch{d[n]=void 0}})}function cn(i){let u=i;u.emit=function(d,m){Qu(i,d,m)},u.expose=function(d){Vu(i,d)}}function c(i,u,d,m){i.addEventListener(u,d,m),yn(()=>i.removeEventListener(u,d,m))}function nu(i,u){let d=U(()=>{let m=i();if(!m)return;let n=()=>{u(m.assignedElements())};return m.addEventListener("slotchange",n),n(),()=>m.removeEventListener("slotchange",n)});yn(d)}var iu=Symbol("sui:transition");function _n(i){return i[iu]}function Od(i){let u=i,d=u[iu];if(!d)d={inOpts:{},outOpts:{}},u[iu]=d;return d}function Ln(i,u,d={},m="both"){let n=Od(i);if(m==="in"||m==="both")n.inT=u,n.inOpts=d;if(m==="out"||m==="both")n.outT=u,n.outOpts=d;if(!i.hasAttribute("data-sui-transition"))i.setAttribute("data-sui-transition","")}function Kd(){if(typeof window>"u"||typeof window.matchMedia!=="function")return!1;try{return window.matchMedia("(prefers-reduced-motion: reduce)").matches}catch{return!1}}function Uu(i){if(!Kd())return i;return{...i,duration:0,delay:0}}function Nu(i){let u=_n(i);if(!u?.inT?.in)return;u.current?.cancel();let d=u.inT.in(i,Uu(u.inOpts));return u.current=d,d}function Au(i){let u=_n(i);if(!u?.outT?.out)return;u.current?.cancel();let d=u.outT.out(i,Uu(u.outOpts));return u.current=d,d}function uu(i){if(i.nodeType!==1)return;let u=i;if(u.hasAttribute("data-sui-transition")){let d=_n(u);if(d?.current?.cancel(),d)d.current=void 0}for(let d of u.querySelectorAll("[data-sui-transition]")){let m=_n(d);if(m?.current?.cancel(),m)m.current=void 0}}function Fu(i,u){for(let d of i){if(d.nodeType!==1)continue;let m=d;if(m.hasAttribute("data-sui-transition"))u(m);for(let n of m.querySelectorAll("[data-sui-transition]"))u(n)}}function Vn(i){Fu(i,(u)=>{Nu(u)})}function kn(i){let u=[];if(Fu(i,(d)=>{let m=Au(d);if(m)u.push(m.finished.catch(()=>{}))}),u.length===0)return null;return Promise.all(u).then(()=>{return})}var Bd=200,Id="cubic-bezier(0.4, 0, 0.2, 1)",Rd=0;function Wn(i){return{duration:typeof i.duration==="number"?i.duration:Bd,easing:typeof i.easing==="string"?i.easing:Id,delay:typeof i.delay==="number"?i.delay:Rd,fill:"both"}}function Qn(i,u,d){let m=i.animate;if(typeof m!=="function")return;return m.call(i,u,d)}var Dn={in:(i,u)=>Qn(i,[{opacity:0},{opacity:1}],Wn(u)),out:(i,u)=>Qn(i,[{opacity:1},{opacity:0}],Wn(u))};function Yu(i,u){let d=i==="y"?"translateY(-100%)":"translateX(-100%)",m="translate(0, 0)";return u==="in"?[{opacity:0,transform:d},{opacity:1,transform:"translate(0, 0)"}]:[{opacity:1,transform:"translate(0, 0)"},{opacity:0,transform:d}]}var du={in:(i,u)=>{let d=u.axis==="x"?"x":"y";return Qn(i,Yu(d,"in"),Wn(u))},out:(i,u)=>{let d=u.axis==="x"?"x":"y";return Qn(i,Yu(d,"out"),Wn(u))}};function qu(i,u){let d={opacity:0,transform:`scale(${i})`},m={opacity:1,transform:"scale(1)"};return u==="in"?[d,m]:[m,d]}var mu={in:(i,u)=>{let d=typeof u.from==="number"?u.from:0;return Qn(i,qu(d,"in"),Wn(u))},out:(i,u)=>{let d=typeof u.from==="number"?u.from:0;return Qn(i,qu(d,"out"),Wn(u))}};function kd(i,u,d,m,n){if(d.fill(0,0,u),u===0)return;let b=0;for(let E=0;E<u;E++){let M=i[2*E+1],y=0,L=b;while(y<L){let l=y+L>>>1;if(i[2*n[l]+1]<M)y=l+1;else L=l}if(m[E]=y>0?n[y-1]:-1,n[y]=E,y===b)b++}let w=n[b-1];while(w>=0)d[w]=1,w=m[w]}function h(i){let{anchor:u,items:d,key:m,render:n}=i,b=[],w=!0,E=new Set,M=new Map,y=new Map,L=new Map,l=new Set,f=[],t=[],r=new Int32Array(0),H=new Uint8Array(0),C=new Int32Array(0),z=new Int32Array(0),g=(T)=>{for(let D of T.nodes)try{D.parentNode?.removeChild(D)}catch{}T.cleanup?.()},j=U(()=>{if(!w)return;let T=d(),D=T.length;if(r.length<D*2)r=new Int32Array(Math.max(r.length*2,D*2,16));if(H.length<D){let Q=Math.max(H.length*2,D,8);H=new Uint8Array(Q),C=new Int32Array(Q),z=new Int32Array(Q)}for(let Q=0;Q<D;Q++){let q=T[Q],F=m(q,Q);if(M.has(F))console.warn("[sui] {#for}: duplicate key",F,"at index",Q,"(prior at",M.get(F),"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");M.set(F,Q)}for(let Q of b)y.set(Q.key,Q);for(let Q of b){if(M.has(Q.key))continue;let q=kn(Q.nodes);if(q===null){g(Q);continue}E.add(Q),q.then(()=>{if(!E.delete(Q))return;g(Q)})}t.length=0,f.length=0;for(let Q=0;Q<D;Q++){let q=T[Q],F=m(q,Q),k=y.get(F),K;if(k)k.item=q,Xn(u,"render",()=>k.update?.(q,Q)),K=k;else{let B=Xn(u,"render",()=>n(q,Q));if(!B)continue;K={key:F,item:q,nodes:B.nodes,...B.cleanup&&{cleanup:B.cleanup},...B.update&&{update:B.update}},f.push(...B.nodes)}t.push(K)}for(let Q=0;Q<b.length;Q++)L.set(b[Q].key,Q);let Z=0;for(let Q=0;Q<t.length;Q++){let q=L.get(t[Q].key);if(q!==void 0)r[2*Z]=Q,r[2*Z+1]=q,Z++}kd(r,Z,H,C,z);for(let Q=0;Q<Z;Q++)if(H[Q])l.add(r[2*Q]);let X=u.parentNode;if(X&&t.length>0){let Q=u,q=null;for(let F=0;F<t.length;F++){let k=t[F];if(l.has(F)){if(q)X.insertBefore(q,Q.nextSibling),q=null;Q=k.nodes[k.nodes.length-1]??Q}else{if(!q)q=document.createDocumentFragment();let K=k.nodes;for(let B=0;B<K.length;B++)q.appendChild(K[B])}}if(q){X.insertBefore(q,Q.nextSibling);let F=t[t.length-1].nodes;Q=F[F.length-1]??Q}}let G=b;if(b=t,t=G,f.length>0)Vn(f);M.clear(),y.clear(),L.clear(),l.clear(),f.length=0});return()=>{w=!1,j();for(let T of b)g(T);for(let T of E)g(T);E.clear(),b=[],t.length=0}}function I(i){let{anchor:u}=i,{test:d,branches:m}=ad(i)?{test:i.test,branches:i.branches}:Sd(i),n=null,b=-2,w=null,E=(y)=>{for(let L of y)try{L.parentNode?.removeChild(L)}catch{}},M=U(()=>{let y=d();if(n&&y===b)return;if(w&&y===w.idx&&y>=0&&y<m.length){let f=w;w=null;for(let t of f.nodes)uu(t);n={nodes:f.nodes,...f.cleanup&&{cleanup:f.cleanup}},b=y,Vn(n.nodes);return}if(n){let f={idx:b,nodes:n.nodes,...n.cleanup&&{cleanup:n.cleanup}},t=kn(f.nodes);if(n=null,t===null)E(f.nodes),f.cleanup?.();else w=f,t.then(()=>{if(w!==f)return;E(f.nodes),f.cleanup?.(),w=null})}if(b=y,y<0||y>=m.length)return;let L=Xn(u,"render",()=>m[y]());if(!L)return;n=L;let l=u.parentNode;if(!l)return;for(let f of n.nodes)l.insertBefore(f,u);Vn(n.nodes)});return()=>{if(M(),n)E(n.nodes),n.cleanup?.(),n=null;if(w)E(w.nodes),w.cleanup?.(),w=null}}function ad(i){return typeof i.test==="function"}function Sd(i){let{cond:u,then_:d,else_:m}=i;return{test:()=>u()?0:m?1:-1,branches:m?[d,m]:[d]}}var bu=(i)=>i??"",fu=(i)=>{if(i===null||i==="")return NaN;return Number(i)},tu=(i)=>{if(i===null)return!1;if(i.toLowerCase()==="false")return!1;return!0};function ni(i){return(u)=>{if(u===null||u==="")return i;try{return JSON.parse(u)}catch{return i}}}function ru(i,u){let d=new Set(i);return(m)=>m!==null&&d.has(m)?m:u}var ii=(i)=>{switch(i.type){case"string":return bu;case"bool":return tu;case"number":return fu;case"union":{let u=i.values??[],d=i.default??u[0]??"";return ru(u,d)}case"object":return ni(i.default);case"array":return ni(i.default??[])}};function Ku(i,u,d,m){if(!u)return;let n={},b,w=d.formValue;if(w!==void 0){let{sig:M,propDef:y}=Ou(d,m,w,"formValue"),L=y.default,l=ii(y);b=U(()=>{let f=M();u.setFormValue(vd(f))}),n.reset=()=>{M.set(L)},n.restore=(f)=>{M.set(l(f))}}let E=d.formDisabled;if(E!==void 0){let{sig:M,propDef:y}=Ou(d,m,E,"formDisabled");if(y.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${E}" must be a bool-typed prop`);n.disabled=(L)=>{M.set(!!L)}}if(i.__shipSetFormHooks=n,b)i.__shipSetFormValueCleanup=b;return b}function Bu(i){let u=i;delete u.__shipSetFormHooks,delete u.__shipSetFormValueCleanup}function vd(i){if(i===null||i===void 0)return"";if(typeof i==="boolean")return i?"on":"";return String(i)}function Ou(i,u,d,m){let n=i.props[d];if(!n)throw Error(`[sui-runtime] ${m}: "${d}" — no matching prop on this component`);let b=u[Yn(d)];if(!b)throw Error(`[sui-runtime] ${m}: "${d}" — prop signal missing`);return{sig:b,propDef:n}}var P={string(i={}){return{type:"string",...i}},bool(i={}){return{type:"bool",...i}},number(i={}){return{type:"number",...i}},union(i,u={}){return{type:"union",values:i,...u}},object(i={}){return{type:"object",...i}},array(i={}){return{type:"array",...i}}},xd=(i,u)=>{if(u===void 0||u===null)return null;switch(i.type){case"string":return u===""?null:String(u);case"number":return Number.isNaN(u)?null:String(u);case"bool":return u?"":null;case"union":return String(u);case"object":case"array":try{return JSON.stringify(u)}catch{return null}}},hd=(i,u)=>{let d=`ship-set-style-${i}`;if(document.getElementById(d))return;let m=document.createElement("style");m.id=d,m.textContent=`[data-component="${i}"] { ${u} }`,document.head.appendChild(m)},$d=(i,u,d,m)=>{let n=i.getAttribute(u),b=n!==null?m(n):d.default;return J(b)},od=(i)=>{let u=[],d={};for(let[b,w]of Object.entries(i.props??{}))u.push(b),d[b]={camel:Yn(b),decode:ii(w)};let m=i.formAssociated===!0||i.formValue!==void 0||i.formDisabled!==void 0,n={observedAttrs:u,props:d};if(m)n.formAssociated=!0;return n};function V(i,u){let d=od(u),m=Object.entries(u.props??{});Lu(i,d,(n,b,w)=>{let E=null;if(u.shadow){if(E=n.shadowRoot??n.attachShadow({mode:"open"}),u.styles){let y=document.createElement("style");y.textContent=u.styles,E.appendChild(y)}}else if(u.styles)hd(i,u.styles);let M=[];for(let[y,L]of m){let l=Yn(y),f=d.props[y].decode,t=$d(n,y,L,f);if(b[l]=t,L.reflect){let r=!0,H=U(()=>{let C=t();if(r){r=!1;return}let z=xd(L,C);if(z===null)n.removeAttribute(y);else n.setAttribute(y,z)});M.push(H)}}if(u.formValue!==void 0||u.formDisabled!==void 0){let y=Ku(n,w,u,b);if(typeof y==="function")M.push(y)}if(u.setup){let y={props:b,host:n,cleanups:M};if(w)y.internals=w;u.setup(y)}return()=>{Bu(n);for(let y of M.splice(0))try{y()}catch(L){console.error(`[ship-set:${i}] cleanup threw:`,L)}}})}var Iu=new Set;function Lu(i,u,d){if(customElements.get(i)){if(!Iu.has(i))Iu.add(i),console.warn(`[sui-runtime] customElements.define("${i}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(u.formAssociated){class n extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return u.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=i,cn(this),this.scope=In();try{this.cleanup=Bn(this.scope,()=>p0(this,()=>d(this,this.propSignals,this.internals)))}catch(b){let w=gn(this.parentNode);if(w)w.catch(b,"mount");else console.error(`[ship-set:${i}] setup threw:`,b)}}disconnectedCallback(){try{this.cleanup?.()}catch(b){console.error(`[ship-set:${i}] cleanup threw:`,b)}this.cleanup=void 0;try{this.scope?.dispose()}catch(b){console.error(`[ship-set:${i}] scope dispose threw:`,b)}this.scope=void 0}attributeChangedCallback(b,w,E){let M=u.props[b];if(!M)return;let y=this.propSignals[M.camel];if(y)y.set(M.decode(E))}formResetCallback(){let b=this.__shipSetFormHooks;try{b?.reset?.()}catch(w){console.error(`[ship-set:${i}] formResetCallback threw:`,w)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(b,w){let E=this.__shipSetFormHooks;try{E?.restore?.(b)}catch(M){console.error(`[ship-set:${i}] formStateRestoreCallback threw:`,M)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:b,mode:w}}))}formDisabledCallback(b){let w=this.__shipSetFormHooks;try{w?.disabled?.(b)}catch(E){console.error(`[ship-set:${i}] formDisabledCallback threw:`,E)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:b}}))}}customElements.define(i,n);return}class m extends HTMLElement{static get observedAttributes(){return u.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=i,cn(this),this.scope=In();try{this.cleanup=Bn(this.scope,()=>p0(this,()=>d(this,this.propSignals)))}catch(n){let b=gn(this.parentNode);if(b)b.catch(n,"mount");else console.error(`[ship-set:${i}] setup threw:`,n)}}disconnectedCallback(){try{this.cleanup?.()}catch(n){console.error(`[ship-set:${i}] cleanup threw:`,n)}this.cleanup=void 0;try{this.scope?.dispose()}catch(n){console.error(`[ship-set:${i}] scope dispose threw:`,n)}this.scope=void 0}attributeChangedCallback(n,b,w){let E=u.props[n];if(!E)return;let M=this.propSignals[E.camel];if(M)M.set(E.decode(w))}}customElements.define(i,m)}function Yn(i){return i.replace(/-([a-z])/g,(u,d)=>d.toUpperCase())}function lu(i,u,d){let m=pd(i.state,{ship:u.ship,...d}),n={};for(let[f,t]of Object.entries(m))n[f]=J(t);let b={ship:u.ship,route:u.route??{},navigate:u.navigate??((f)=>{}),...d,...n},w=cd(n,b),E={...b},M={};for(let[f,t]of Object.entries(i.derived??{}))M[f]=s(()=>t(E));Object.assign(E,M);let y={};for(let[f,t]of Object.entries(i.on??{}))y[f]=(...r)=>t(w,...r);let L={},l=typeof i.use==="function"?i.use(E):i.use??{};for(let[f,t]of Object.entries(l))L[f]=lu(t,u,E);return{...n,...M,...y,...L}}function pd(i,u){if(i===void 0)return{};if(typeof i==="function")return i(u);return i}function cd(i,u){return new Proxy({},{get(d,m){if(m in i)return i[m]();return u[m]},set(d,m,n){if(m in i)return i[m].set(n),!0;return console.warn(`[ship-set] hook handler tried to write s.${m} = … but ${m} is not a hook state key. The assignment was ignored.`),!0},has(d,m){return m in i||m in u},ownKeys(){return Object.keys(i)},getOwnPropertyDescriptor(d,m){if(m in i)return{configurable:!0,enumerable:!0,writable:!0,value:i[m]()};return}})}var Ru="__shipSetLifecycle";function ku(i,u){let d=_d(i,u),m={ship:u.ship,route:u.route??{},navigate:u.navigate??((f)=>{}),...u.scope??{}},n=d.scope?d.scope(m,u.host):{};Object.assign(m,n);let b=nm(m,u.stateKeys,n),w=im(b,m),E={...m},M=um(d,E);Object.assign(E,M);let y=mm(d,E,u);for(let[f,t]of Object.entries(y))E[f]=t;let L=dm(d,w),l={...n,...M,...L,...y};if(Object.keys(n).length>0&&u.host){let f=_0(n);if(Object.keys(f).length>0){let t=u.host,r=t.tagName.toLowerCase();c0(t,r,f)}}if(d.mount||d.unmount){let f={state:w};if(d.mount)f.mount=d.mount;if(d.unmount)f.unmount=d.unmount;Object.defineProperty(l,Ru,{value:f,enumerable:!0,configurable:!0,writable:!1})}return l}function _d(i,u){if(typeof i==="function")return i({ship:u.ship});return i}function nm(i,u,d){let m={},n=u!==void 0,b=d?Object.keys(d):[];if(n||b.length>0){if(u)for(let w of u){let E=i[w];if(Eu(E))m[w]=E}for(let w of b){if(w in m)continue;let E=i[w];if(Eu(E))m[w]=E}return m}for(let[w,E]of Object.entries(i))if(Eu(E))m[w]=E;return m}function Eu(i){return typeof i==="function"&&"set"in i&&typeof i.set==="function"}function im(i,u){return new Proxy({},{get(d,m){if(m in i)return i[m]();return u[m]},set(d,m,n){if(m in i){if(typeof n==="function")i[m].set(()=>n);else i[m].set(n);return!0}return console.warn(`[ship-set] handler tried to write s.${m} = … but ${m} is not declared as <state name="${m}"> in the template. The assignment was ignored.`),!0},has(d,m){return m in i||m in u},ownKeys(){return Object.keys(i)},getOwnPropertyDescriptor(d,m){if(m in i)return{configurable:!0,enumerable:!0,writable:!0,value:i[m]()};return}})}function um(i,u){let d=i.derived??{},m={};for(let[n,b]of Object.entries(d))m[n]=s(()=>b(u));return m}function dm(i,u){let d=i.on??{},m={};for(let[n,b]of Object.entries(d))m[n]=(...w)=>b(u,...w);return m}function mm(i,u,d){if(i.use===void 0)return{};let m=typeof i.use==="function"?i.use(u):i.use,n={};for(let[b,w]of Object.entries(m))n[b]=lu(w,d,u);return n}function N(i){return{setup:(u)=>ku(i,u),__def:i}}if(typeof customElements<"u"){for(let i of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(i))customElements.define(i,class extends HTMLElement{})}var rm=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,Lm=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function au(i){if(typeof CSSStyleSheet>"u")return null;try{let u=new CSSStyleSheet;return u.replaceSync(i),u}catch{return null}}var Su=au(rm),vu=au(Lm);function Y(i){if(typeof ShadowRoot>"u")return;if(!(i instanceof ShadowRoot))return;let u=[vu,Su].filter((d)=>d!=null);if(u.length===0)return;try{i.adoptedStyleSheets=[...u,...i.adoptedStyleSheets??[]]}catch{}}var at=V("sui-status-dot",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"success",reflect:!0}),pulse:P.bool({default:!1,reflect:!0}),size:P.union(["sm","md"],{default:"md",reflect:!0}),label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.pulse=u.pulse,n.size=u.size,n.label=u.label;let b=d.shadowRoot??d;Y(b);let w=document.createElement("span");return w.setAttribute("class","dot"),w.setAttribute("part","dot"),m.push(W(w,"role",()=>n.label()?"img":null)),m.push(W(w,"aria-label",()=>n.label()||null)),m.push(W(w,"aria-hidden",()=>n.label()?null:"true")),b.appendChild(w),n}});var xt=V("sui-spinner",{props:{size:P.union(["xs","sm","md","lg"],{default:"md",reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),label:P.string({default:"Loading"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.size=u.size,n.tone=u.tone,n.label=u.label;let b=d.shadowRoot??d;Y(b);let w=document.createElement("span");return w.setAttribute("class","ring"),w.setAttribute("part","ring"),m.push(W(w,"role",()=>n.label()?"status":null)),m.push(W(w,"aria-label",()=>n.label()||null)),m.push(W(w,"aria-hidden",()=>n.label()?null:"true")),b.appendChild(w),n}});var ot=V("sui-badge",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["solid","soft","outline"],{default:"soft",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),pill:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.variant=u.variant,n.size=u.size,n.pill=u.pill;let b=d.shadowRoot??d;Y(b);let w=document.createElement("span");w.setAttribute("class","badge"),w.setAttribute("part","badge");let E=document.createElement("span");E.setAttribute("class","leading"),E.setAttribute("part","leading");let M=document.createElement("slot");M.setAttribute("name","leading"),E.appendChild(M),w.appendChild(E);let y=document.createElement("slot");return w.appendChild(y),b.appendChild(w),n}});var ui=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","button");if(!u.hasAttribute("tabindex")&&!d.disabled())u.setAttribute("tabindex","0");let m=U(()=>{let w=d.selected(),E=d.disabled();if(u.setAttribute("aria-pressed",w?"true":"false"),E){if(u.setAttribute("aria-disabled","true"),u.getAttribute("tabindex")!=="-1")u.setAttribute("tabindex","-1")}else if(u.removeAttribute("aria-disabled"),u.getAttribute("tabindex")==="-1")u.setAttribute("tabindex","0")}),n=(w)=>{if(d.disabled())return;if(w.key==="Enter"||w.key===" ")w.preventDefault(),u.click();else if(d.removable()&&(w.key==="Backspace"||w.key==="Delete"))w.preventDefault(),u.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};u.addEventListener("keydown",n);let b=U(()=>{let w=d.removeBtn?.();if(!w)return;let E=(M)=>{if(M.stopPropagation(),d.disabled())return;u.dispatchEvent(new CustomEvent("ship-remove",{detail:{},bubbles:!0,composed:!0}))};return w.addEventListener("click",E),()=>w.removeEventListener("click",E)});return()=>{m(),u.removeEventListener("keydown",n),b()}}});var n1=V("sui-chip",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0}),variant:P.union(["solid","soft","outline","ghost"],{default:"soft",reflect:!0}),size:P.union(["sm","md","lg"],{default:"sm",reflect:!0}),selected:P.bool({default:!1,reflect:!0}),removable:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.variant=u.variant,n.size=u.size,n.selected=u.selected,n.removable=u.removable,n.disabled=u.disabled,n.removeBtn=J(null);let b=null;if(ui&&typeof ui.setup==="function"){if(i.scope=n,b=ui.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(t)=>I({anchor:t,test:()=>{if(n.removable())return 0;return-1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("button");C.setAttribute("class","remove"),C.setAttribute("part","remove"),C.setAttribute("type","button"),C.setAttribute("aria-label","Remove"),C.setAttribute("tabindex","-1"),A(n,"removeBtn",C);let z=document.createElement("svg");z.setAttribute("viewbox","0 0 10 10"),z.setAttribute("aria-hidden","true");let g=document.createElement("path");return g.setAttribute("d","M2 2 L8 8 M8 2 L2 8"),z.appendChild(g),C.appendChild(z),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let j of H)try{j()}catch(T){}}}}]}),E=d.shadowRoot??d;Y(E);let M=document.createElement("span");M.setAttribute("class","chip"),M.setAttribute("part","chip");let y=document.createElement("span");y.setAttribute("class","leading"),y.setAttribute("part","leading");let L=document.createElement("slot");L.setAttribute("name","leading"),y.appendChild(L),M.appendChild(y);let l=document.createElement("slot");M.appendChild(l);let f=document.createComment("if");if(M.appendChild(f),m.push(w(f)),E.appendChild(M),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let r=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let g=H[C];if(g!=null&&typeof g.set==="function"){if(typeof z==="function")g.set(()=>z);else g.set(z);return!0}return!1}});if(t.mount){let H=t.mount(r,d);if(typeof H==="function")m.push(H)}if(t.unmount)m.push(()=>t.unmount(r))}}return n}});var di=N({mount(i,u){let d=i,m=(b)=>{if(b.key==="Enter"||b.key===" ")b.preventDefault(),u.click()},n=U(()=>{if(d.interactive()){if(!u.hasAttribute("role"))u.setAttribute("role","button");if(!u.hasAttribute("tabindex"))u.setAttribute("tabindex","0");return u.addEventListener("keydown",m),()=>u.removeEventListener("keydown",m)}else{if(u.getAttribute("role")==="button")u.removeAttribute("role");if(u.getAttribute("tabindex")==="0")u.removeAttribute("tabindex")}});return()=>{n()}}});var f1=V("sui-card",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["default","outline"],{default:"default",reflect:!0}),padding:P.union(["none","sm","md","lg"],{default:"md",reflect:!0}),interactive:P.bool({default:!1,reflect:!0}),"accent-bar":P.bool({default:!1,reflect:!0}),"status-rail":P.bool({default:!1,reflect:!0}),fill:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.variant=u.variant,n.padding=u.padding,n.interactive=u.interactive,n.accentBar=u.accentBar,n.statusRail=u.statusRail,n.fill=u.fill;let b=null;if(di&&typeof di.setup==="function"){if(i.scope=n,b=di.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","header"),E.setAttribute("part","header");let M=document.createElement("slot");M.setAttribute("name","header"),m.push(dn(M,d,"data-no-header")),E.appendChild(M),w.appendChild(E);let y=document.createElement("div");y.setAttribute("class","content"),y.setAttribute("part","content");let L=document.createElement("slot");y.appendChild(L),w.appendChild(y);let l=document.createElement("div");l.setAttribute("class","footer"),l.setAttribute("part","footer");let f=document.createElement("slot");if(f.setAttribute("name","footer"),m.push(dn(f,d,"data-no-footer")),l.appendChild(f),w.appendChild(l),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let r=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let g=H[C];if(g!=null&&typeof g.set==="function"){if(typeof z==="function")g.set(()=>z);else g.set(z);return!0}return!1}});if(t.mount){let H=t.mount(r,d);if(typeof H==="function")m.push(H)}if(t.unmount)m.push(()=>t.unmount(r))}}return n}});var L1=V("sui-link",{props:{href:P.string({default:""}),external:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["default","underline","subtle"],{default:"default",reflect:!0}),target:P.string({default:""}),rel:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.href=u.href,n.external=u.external,n.tone=u.tone,n.variant=u.variant,n.target=u.target,n.rel=u.rel;let b=(L)=>I({anchor:L,test:()=>{if(n.external())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("span");t.setAttribute("class","sr-only");let r=document.createTextNode("(opens in new tab)");t.appendChild(r),l.appendChild(t);let H=document.createElement("span");H.setAttribute("class","ext"),H.setAttribute("aria-hidden","true");let C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round");let z=document.createElement("path");return z.setAttribute("d","M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6M15 3h6v6M10 14L21 3"),C.appendChild(z),H.appendChild(C),l.appendChild(H),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let g of f)try{g()}catch(j){}}}}]}),w=d.shadowRoot??d;Y(w);let E=document.createElement("a");E.setAttribute("part","link"),m.push(W(E,"href",()=>n.href())),m.push(W(E,"target",()=>n.external()?"_blank":n.target())),m.push(W(E,"rel",()=>n.external()?"noopener noreferrer":n.rel()));let M=document.createElement("slot");E.appendChild(M);let y=document.createComment("if");return E.appendChild(y),m.push(b(y)),w.appendChild(E),n}});var M1=V("sui-divider",{props:{orientation:P.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:P.union(["solid","dashed"],{default:"solid",reflect:!0}),tone:P.union(["default","muted","primary","destructive"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.orientation=u.orientation,n.variant=u.variant,n.tone=u.tone;let b=d.shadowRoot??d;Y(b);let w=document.createElement("hr");return w.setAttribute("class","line"),w.setAttribute("part","line"),w.setAttribute("role","separator"),m.push(W(w,"aria-orientation",()=>n.orientation())),b.appendChild(w),n}});var H1=V("sui-skeleton",{props:{shape:P.union(["text","line","block","circle"],{default:"text",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.shape=u.shape,n.size=u.size;let b=d.shadowRoot??d;Y(b);let w=document.createElement("span");return w.setAttribute("class","sk"),w.setAttribute("part","sk"),w.setAttribute("role","status"),w.setAttribute("aria-label","Loading"),w.setAttribute("aria-live","polite"),w.setAttribute("aria-busy","true"),b.appendChild(w),n}});var mi=N({mount(i,u){let d=i,m=(b)=>{if(b.key==="Enter"||b.key===" ")b.preventDefault(),u.click()},n=U(()=>{if(d.interactive()){if(!u.hasAttribute("role"))u.setAttribute("role","button");if(!u.hasAttribute("tabindex"))u.setAttribute("tabindex","0");return u.addEventListener("keydown",m),()=>u.removeEventListener("keydown",m)}else{if(u.getAttribute("role")==="button")u.removeAttribute("role");if(u.getAttribute("tabindex")==="0")u.removeAttribute("tabindex")}});return()=>{n(),u.removeEventListener("keydown",m)}}});var D1=V("sui-stat",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent"],{default:"default",reflect:!0}),interactive:P.bool({default:!1,reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.interactive=u.interactive,n.size=u.size;let b=null;if(mi&&typeof mi.setup==="function"){if(i.scope=n,b=mi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("span");E.setAttribute("class","label"),E.setAttribute("part","label");let M=document.createElement("slot");M.setAttribute("name","label"),E.appendChild(M),w.appendChild(E);let y=document.createElement("span");y.setAttribute("class","value-row"),y.setAttribute("part","value-row");let L=document.createElement("span");L.setAttribute("class","value"),L.setAttribute("part","value");let l=document.createElement("slot");L.appendChild(l),y.appendChild(L);let f=document.createElement("span");f.setAttribute("class","trend"),f.setAttribute("part","trend");let t=document.createElement("slot");t.setAttribute("name","trend"),f.appendChild(t),y.appendChild(f),w.appendChild(y);let r=document.createElement("span");r.setAttribute("class","hint"),r.setAttribute("part","hint");let H=document.createElement("slot");if(H.setAttribute("name","hint"),r.appendChild(H),w.appendChild(r),b!=null){let C=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(n,{get(g,j){return g[j]},set(g,j,T){let D=g[j];if(D!=null&&typeof D.set==="function"){if(typeof T==="function")D.set(()=>T);else D.set(T);return!0}return!1}});if(C.mount){let g=C.mount(z,d);if(typeof g==="function")m.push(g)}if(C.unmount)m.push(()=>C.unmount(z))}}return n}});var J1=V("sui-mark",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"warning",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone;let b=d.shadowRoot??d;Y(b);let w=document.createElement("mark");w.setAttribute("part","mark");let E=document.createElement("slot");return w.appendChild(E),b.appendChild(w),n}});var bi=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","group");let m=U(()=>{let n=d.keys();if(n)u.setAttribute("aria-label",`Keyboard shortcut: ${n}`);else u.removeAttribute("aria-label")});return()=>m()}});var Y1=V("sui-kbd",{props:{keys:P.string({default:""}),size:P.union(["sm","md"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.keys=u.keys,n.size=u.size;let b=null;if(bi&&typeof bi.setup==="function"){if(i.scope=n,b=bi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("kbd");E.setAttribute("part","kbd");let M=document.createElement("slot");if(E.appendChild(M),w.appendChild(E),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let L=new Proxy(n,{get(l,f){return l[f]},set(l,f,t){let r=l[f];if(r!=null&&typeof r.set==="function"){if(typeof t==="function")r.set(()=>t);else r.set(t);return!0}return!1}});if(y.mount){let l=y.mount(L,d);if(typeof l==="function")m.push(l)}if(y.unmount)m.push(()=>y.unmount(L))}}return n}});var N1=V("sui-avatar",{props:{src:P.string({default:""}),name:P.string({default:""}),initials:P.string({default:""}),"icon-slug":P.string({default:""}),size:P.union(["xs","sm","md","lg","xl"],{default:"md",reflect:!0}),shape:P.union(["round","square"],{default:"round",reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.src=u.src,n.name=u.name,n.initials=u.initials,n.iconSlug=u.iconSlug,n.size=u.size,n.shape=u.shape,n.tone=u.tone;let b=(y)=>I({anchor:y,test:()=>{if(n.iconSlug()||n.src())return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),l=[],f=document.createElement("img");return f.setAttribute("part","img"),l.push(W(f,"src",()=>n.iconSlug()?"https://cdn.simpleicons.org/"+n.iconSlug():n.src())),l.push(W(f,"alt",()=>n.name())),L.appendChild(f),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let t of l)try{t()}catch(r){}}}},()=>{let L=document.createDocumentFragment(),l=[],f=document.createElement("span");f.setAttribute("part","initials");let t=document.createTextNode("");return f.appendChild(t),l.push(a(t,()=>n.initials()?n.initials().slice(0,2).toUpperCase():n.name().trim().slice(0,1).toUpperCase()||"?")),L.appendChild(f),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let r of l)try{r()}catch(H){}}}}]}),w=d.shadowRoot??d;Y(w);let E=document.createElement("span");E.setAttribute("class","root"),E.setAttribute("part","root"),E.setAttribute("role","img"),m.push(W(E,"aria-label",()=>n.name()||n.initials()||"avatar"));let M=document.createComment("if");return E.appendChild(M),m.push(b(M)),w.appendChild(E),n}});var O1=V("sui-progress",{props:{value:P.number({default:0}),max:P.number({default:100}),tone:P.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),indeterminate:P.bool({default:!1,reflect:!0}),label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.max=u.max,n.tone=u.tone,n.size=u.size,n.indeterminate=u.indeterminate,n.label=u.label;let b=d.shadowRoot??d;Y(b);let w=document.createElement("div");w.setAttribute("class","track"),w.setAttribute("part","track"),w.setAttribute("role","progressbar"),m.push(W(w,"aria-valuenow",()=>n.indeterminate()?null:n.value())),w.setAttribute("aria-valuemin","0"),m.push(W(w,"aria-valuemax",()=>n.max())),m.push(W(w,"aria-busy",()=>n.indeterminate()?"true":"false")),m.push(W(w,"aria-label",()=>n.label()));let E=document.createElement("div");return E.setAttribute("class","fill"),E.setAttribute("part","fill"),m.push(W(E,"style",()=>n.indeterminate()?"":"width: clamp(0%, "+n.value()/(n.max()||1)*100+"%, 100%)")),w.appendChild(E),b.appendChild(w),n}});var I1=V("sui-progress-ring",{props:{value:P.number({default:0}),max:P.number({default:100}),size:P.number({default:48}),"track-width":P.number({default:4}),tone:P.union(["default","primary","info","success","warning","destructive","accent"],{default:"primary",reflect:!0}),indeterminate:P.bool({default:!1,reflect:!0}),label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.max=u.max,n.size=u.size,n.trackWidth=u.trackWidth,n.tone=u.tone,n.indeterminate=u.indeterminate,n.label=u.label;let b=d.shadowRoot??d;Y(b);let w=document.createElement("div");w.setAttribute("class","ring"),m.push(W(w,"style",()=>"width:"+n.size()+"px;height:"+n.size()+"px"));let E=document.createElement("svg");E.setAttribute("class","ring-svg"),m.push(W(E,"viewbox",()=>"0 0 "+n.size()+" "+n.size())),E.setAttribute("role","progressbar"),E.setAttribute("aria-valuemin","0"),m.push(W(E,"aria-valuemax",()=>n.max())),m.push(W(E,"aria-valuenow",()=>n.indeterminate()?null:n.value()<0?0:n.value()>n.max()?n.max():n.value())),m.push(W(E,"aria-label",()=>n.label()||null)),E.setAttribute("xmlns","http://www.w3.org/2000/svg");let M=document.createElement("circle");M.setAttribute("class","track"),M.setAttribute("part","track"),m.push(W(M,"cx",()=>n.size()/2)),m.push(W(M,"cy",()=>n.size()/2)),m.push(W(M,"r",()=>(n.size()-n.trackWidth())/2)),M.setAttribute("fill","none"),m.push(W(M,"stroke-width",()=>n.trackWidth())),E.appendChild(M);let y=document.createElement("circle");y.setAttribute("class","bar"),y.setAttribute("part","ring"),m.push(W(y,"cx",()=>n.size()/2)),m.push(W(y,"cy",()=>n.size()/2)),m.push(W(y,"r",()=>(n.size()-n.trackWidth())/2)),y.setAttribute("fill","none"),m.push(W(y,"stroke-width",()=>n.trackWidth())),y.setAttribute("stroke-linecap","round"),m.push(W(y,"stroke-dasharray",()=>3.141592653589793*(n.size()-n.trackWidth()))),m.push(W(y,"stroke-dashoffset",()=>n.indeterminate()?3.141592653589793*(n.size()-n.trackWidth())*0.25:3.141592653589793*(n.size()-n.trackWidth())*(1-(n.value()<0?0:n.value()>(n.max()||1)?1:n.value()/(n.max()||1))))),E.appendChild(y),w.appendChild(E);let L=document.createElement("div");L.setAttribute("class","label"),L.setAttribute("aria-hidden","true");let l=document.createElement("slot");return L.appendChild(l),w.appendChild(L),b.appendChild(w),n}});var Mu={default:"var(--foreground)",primary:"var(--primary)",info:"var(--info)",success:"var(--success)",warning:"var(--warning)",destructive:"var(--destructive)",accent:"var(--accent)",muted:"var(--muted-foreground)"};function un(i){if(!i)return Mu.default;return Mu[i]??Mu.default}function qn(i){if(!i)return[];return i.split(",").map((u)=>Number.parseFloat(u.trim())).filter((u)=>Number.isFinite(u))}function an(i){if(!i)return[];return i.split(",").map((u)=>u.trim())}function ln(i,u){if(!i)return u;try{return JSON.parse(i)}catch{return u}}function Un(i,u,d,m=2){if(i.length===0)return{d:"",points:[]};let n=Math.min(...i),w=Math.max(...i)-n||1,E=d-m*2,M=i.length===1?0:u/(i.length-1),y=i.map((l,f)=>{let t=i.length===1?u/2:f*M,r=m+E-(l-n)/w*E;return[t,r]});return{d:y.map(([l,f],t)=>`${t===0?"M":"L"}${l.toFixed(2)},${f.toFixed(2)}`).join(" "),points:y}}function xu(i,u,d,m){return[i+d*Math.cos(m),u+d*Math.sin(m)]}function Sn(i,u,d,m,n){let[b,w]=xu(i,u,d,m),[E,M]=xu(i,u,d,n),y=n-m>Math.PI?1:0,L=n>m?1:0;return`M${b.toFixed(2)},${w.toFixed(2)} A${d},${d} 0 ${y} ${L} ${E.toFixed(2)},${M.toFixed(2)}`}var fi=N({scope:()=>{let i=J([]),u=J([]);return{seriesPaths:i,gridLines:u}},mount(i,u){let d=U(()=>{let m=ln(i.series(),[]),n=i.width(),b=i.height(),w=m.flatMap((f)=>f.data),E=w.length?Math.min(...w):0,y=(w.length?Math.max(...w):1)-E||1,L=b-4;i.gridLines.set(i.grid()?[0.25,0.5,0.75].map((f,t)=>({id:`g${t}`,y:2+L*f})):[]);let l=[];for(let f of m){if(f.data.length<2)continue;let t=f.data.map((T)=>(T-E)/y*100),{d:r,points:H}=Un(t,n,b,2);if(!r)continue;let C=un(f.tone),z=H[0],g=H[H.length-1],j=`${r} L${g[0].toFixed(2)},${b} L${z[0].toFixed(2)},${b} Z`;l.push({name:f.name,areaD:j,strokeD:r,color:C,pts:H.map(([T,D],Z)=>({id:`${f.name}-${Z}`,cx:T,cy:D}))})}i.seriesPaths.set(l)});return()=>d()}});var $1=V("sui-area-chart",{props:{series:P.string({default:"",reflect:!0}),labels:P.string({default:""}),width:P.number({default:300}),height:P.number({default:200}),points:P.bool({default:!1,reflect:!0}),grid:P.bool({default:!1,reflect:!0}),label:P.string({default:"area chart"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.series=u.series,n.labels=u.labels,n.width=u.width,n.height=u.height,n.points=u.points,n.grid=u.grid,n.label=u.label;let b=null;if(fi&&typeof fi.setup==="function"){if(i.scope=n,b=fi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(H)=>h({anchor:H,items:()=>n.gridLines(),key:(C,z)=>{let g=Object.create(n);return g.g=C,g.__i=z,((j)=>C.id)(g)},render:(C,z)=>{let g=Object.create(n);return g.g=C,g.__i=z,g.__v=J(0),((j)=>{let T=document.createDocumentFragment(),D=[],Z=document.createElement("line");Z.setAttribute("class","grid-line"),Z.setAttribute("x1","0"),D.push(W(Z,"x2",()=>(j.__v(),j.width())));{let X=j.g.y;if(X==null||X===!1||X==="")Z.removeAttribute("y1");else Z.setAttribute("y1",X===!0?"":String(X))}{let X=j.g.y;if(X==null||X===!1||X==="")Z.removeAttribute("y2");else Z.setAttribute("y2",X===!0?"":String(X))}return T.appendChild(Z),{nodes:Array.from(T.childNodes),cleanup:()=>{for(let X of D)try{X()}catch(G){}},update:(X,G)=>{j.g=X,j.__i=G;{let Q=j.g.y;if(Q==null||Q===!1||Q==="")Z.removeAttribute("y1");else Z.setAttribute("y1",Q===!0?"":String(Q))}{let Q=j.g.y;if(Q==null||Q===!1||Q==="")Z.removeAttribute("y2");else Z.setAttribute("y2",Q===!0?"":String(Q))}j.__v.set((Q)=>Q+1)}}})(g)}}),E=(H)=>h({anchor:H,items:()=>n.seriesPaths(),key:(C,z)=>{let g=Object.create(n);return g.s=C,g.__i=z,((j)=>C.name)(g)},render:(C,z)=>{let g=Object.create(n);return g.s=C,g.__i=z,g.__v=J(0),((j)=>{let T=(F)=>h({anchor:F,items:()=>j.s.pts,key:(k,K)=>{let B=Object.create(j);return B.p=k,B.__i=K,((S)=>k.id)(B)},render:(k,K)=>{let B=Object.create(j);return B.p=k,B.__i=K,B.__v=J(0),((S)=>{let R=document.createDocumentFragment(),v=[],O=document.createElement("circle");{let x=S.p.cx;if(x==null||x===!1||x==="")O.removeAttribute("cx");else O.setAttribute("cx",x===!0?"":String(x))}{let x=S.p.cy;if(x==null||x===!1||x==="")O.removeAttribute("cy");else O.setAttribute("cy",x===!0?"":String(x))}O.setAttribute("r","2.5");{let x=S.s.color;if(x==null||x===!1||x==="")O.removeAttribute("fill");else O.setAttribute("fill",x===!0?"":String(x))}return R.appendChild(O),{nodes:Array.from(R.childNodes),cleanup:()=>{for(let x of v)try{x()}catch($){}},update:(x,$)=>{S.p=x,S.__i=$;{let o=S.p.cx;if(o==null||o===!1||o==="")O.removeAttribute("cx");else O.setAttribute("cx",o===!0?"":String(o))}{let o=S.p.cy;if(o==null||o===!1||o==="")O.removeAttribute("cy");else O.setAttribute("cy",o===!0?"":String(o))}{let o=S.s.color;if(o==null||o===!1||o==="")O.removeAttribute("fill");else O.setAttribute("fill",o===!0?"":String(o))}S.__v.set((o)=>o+1)}}})(B)}}),D=(F)=>I({anchor:F,test:()=>{if(j.points())return 0;return-1},branches:[()=>{let k=document.createDocumentFragment(),K=[],B=document.createComment("for");return k.appendChild(B),K.push(T(B)),{nodes:Array.from(k.childNodes),cleanup:()=>{for(let S of K)try{S()}catch(R){}}}}]}),Z=document.createDocumentFragment(),X=[],G=document.createElement("path");{let F=j.s.areaD;if(F==null||F===!1||F==="")G.removeAttribute("d");else G.setAttribute("d",F===!0?"":String(F))}{let F=j.s.color;if(F==null||F===!1||F==="")G.removeAttribute("fill");else G.setAttribute("fill",F===!0?"":String(F))}G.setAttribute("fill-opacity","0.3"),G.setAttribute("stroke","none"),Z.appendChild(G);let Q=document.createElement("path");{let F=j.s.strokeD;if(F==null||F===!1||F==="")Q.removeAttribute("d");else Q.setAttribute("d",F===!0?"":String(F))}Q.setAttribute("fill","none");{let F=j.s.color;if(F==null||F===!1||F==="")Q.removeAttribute("stroke");else Q.setAttribute("stroke",F===!0?"":String(F))}Q.setAttribute("stroke-width","1.75"),Q.setAttribute("stroke-linecap","round"),Q.setAttribute("stroke-linejoin","round"),Z.appendChild(Q);let q=document.createComment("if");return Z.appendChild(q),X.push(D(q)),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let F of X)try{F()}catch(k){}},update:(F,k)=>{j.s=F,j.__i=k;{let K=j.s.areaD;if(K==null||K===!1||K==="")G.removeAttribute("d");else G.setAttribute("d",K===!0?"":String(K))}{let K=j.s.color;if(K==null||K===!1||K==="")G.removeAttribute("fill");else G.setAttribute("fill",K===!0?"":String(K))}{let K=j.s.strokeD;if(K==null||K===!1||K==="")Q.removeAttribute("d");else Q.setAttribute("d",K===!0?"":String(K))}{let K=j.s.color;if(K==null||K===!1||K==="")Q.removeAttribute("stroke");else Q.setAttribute("stroke",K===!0?"":String(K))}j.__v.set((K)=>K+1)}}})(g)}}),M=d.shadowRoot??d;Y(M);let y=document.createElement("div");y.setAttribute("class","wrap");let L=document.createElement("svg");m.push(W(L,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),L.setAttribute("preserveaspectratio","none"),L.setAttribute("role","img"),m.push(W(L,"aria-label",()=>n.label()));let l=document.createComment("for");L.appendChild(l),m.push(w(l));let f=document.createComment("for");L.appendChild(f),m.push(E(f)),y.appendChild(L);let t=document.createElement("div");t.setAttribute("class","legend");let r=document.createElement("slot");if(t.appendChild(r),y.appendChild(t),M.appendChild(y),b!=null){let H=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(H){let C=new Proxy(n,{get(z,g){return z[g]},set(z,g,j){let T=z[g];if(T!=null&&typeof T.set==="function"){if(typeof j==="function")T.set(()=>j);else T.set(j);return!0}return!1}});if(H.mount){let z=H.mount(C,d);if(typeof z==="function")m.push(z)}if(H.unmount)m.push(()=>H.unmount(C))}}return n}});var ti=N({scope:()=>{let i=J([]),u=J(!0);return{bars:i,isEmpty:u}},mount(i,u){let d=U(()=>{let m=qn(i.data());if(m.length===0){i.bars.set([]),i.isEmpty.set(!0);return}let n=an(i.labels()),b=i.width(),w=i.height(),E=i.max(),M=E>0?E:Math.max(...m,0),y=M<=0?1:M,L=un(i.tone()),l=4,f=3,t=i.orientation()==="horizontal",r=m.length;i.bars.set(m.map((H,C)=>{let z=Math.max(0,H)/y,g,j,T,D;if(t)D=(w-l*(r-1))/r,j=C*(D+l),T=z*b,g=0;else T=(b-l*(r-1))/r,g=C*(T+l),D=z*w,j=w-D;return{id:`b${C}`,x:g,y:j,w:Math.max(0,T),h:Math.max(0,D),rx:f,ry:f,color:L,title:`${n[C]??String(H)}: ${H}`}})),i.isEmpty.set(!1)});return()=>d()}});var nr=V("sui-bar-chart",{props:{data:P.string({default:"",reflect:!0}),labels:P.string({default:""}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),orientation:P.union(["vertical","horizontal"],{default:"vertical",reflect:!0}),width:P.number({default:300}),height:P.number({default:200}),max:P.number({default:0}),label:P.string({default:"bar chart"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.data=u.data,n.labels=u.labels,n.tone=u.tone,n.orientation=u.orientation,n.width=u.width,n.height=u.height,n.max=u.max,n.label=u.label;let b=null;if(ti&&typeof ti.setup==="function"){if(i.scope=n,b=ti.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>h({anchor:L,items:()=>n.bars(),key:(l,f)=>{let t=Object.create(n);return t.b=l,t.__i=f,((r)=>l.id)(t)},render:(l,f)=>{let t=Object.create(n);return t.b=l,t.__i=f,t.__v=J(0),((r)=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("rect");{let T=r.b.x;if(T==null||T===!1||T==="")z.removeAttribute("x");else z.setAttribute("x",T===!0?"":String(T))}{let T=r.b.y;if(T==null||T===!1||T==="")z.removeAttribute("y");else z.setAttribute("y",T===!0?"":String(T))}{let T=r.b.w;if(T==null||T===!1||T==="")z.removeAttribute("width");else z.setAttribute("width",T===!0?"":String(T))}{let T=r.b.h;if(T==null||T===!1||T==="")z.removeAttribute("height");else z.setAttribute("height",T===!0?"":String(T))}{let T=r.b.rx;if(T==null||T===!1||T==="")z.removeAttribute("rx");else z.setAttribute("rx",T===!0?"":String(T))}{let T=r.b.ry;if(T==null||T===!1||T==="")z.removeAttribute("ry");else z.setAttribute("ry",T===!0?"":String(T))}{let T=r.b.color;if(T==null||T===!1||T==="")z.removeAttribute("fill");else z.setAttribute("fill",T===!0?"":String(T))}let g=document.createElement("title"),j=document.createTextNode("");return g.appendChild(j),j.textContent=r.b.title,z.appendChild(g),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let T of C)try{T()}catch(D){}},update:(T,D)=>{r.b=T,r.__i=D;{let Z=r.b.x;if(Z==null||Z===!1||Z==="")z.removeAttribute("x");else z.setAttribute("x",Z===!0?"":String(Z))}{let Z=r.b.y;if(Z==null||Z===!1||Z==="")z.removeAttribute("y");else z.setAttribute("y",Z===!0?"":String(Z))}{let Z=r.b.w;if(Z==null||Z===!1||Z==="")z.removeAttribute("width");else z.setAttribute("width",Z===!0?"":String(Z))}{let Z=r.b.h;if(Z==null||Z===!1||Z==="")z.removeAttribute("height");else z.setAttribute("height",Z===!0?"":String(Z))}{let Z=r.b.rx;if(Z==null||Z===!1||Z==="")z.removeAttribute("rx");else z.setAttribute("rx",Z===!0?"":String(Z))}{let Z=r.b.ry;if(Z==null||Z===!1||Z==="")z.removeAttribute("ry");else z.setAttribute("ry",Z===!0?"":String(Z))}{let Z=r.b.color;if(Z==null||Z===!1||Z==="")z.removeAttribute("fill");else z.setAttribute("fill",Z===!0?"":String(Z))}j.textContent=r.b.title,r.__v.set((Z)=>Z+1)}}})(t)}}),E=(L)=>I({anchor:L,test:()=>{if(n.isEmpty())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("svg");return f.push(W(t,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),t.setAttribute("aria-hidden","true"),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let r of f)try{r()}catch(H){}}}},()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("svg");f.push(W(t,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),t.setAttribute("preserveaspectratio","none"),t.setAttribute("role","img"),f.push(W(t,"aria-label",()=>n.label()));let r=document.createComment("for");return t.appendChild(r),f.push(w(r)),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let H of f)try{H()}catch(C){}}}}]}),M=d.shadowRoot??d;Y(M);let y=document.createComment("if");if(M.appendChild(y),m.push(E(y)),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var ri=N({scope:()=>{let i=J([]),u=J([]);return{seriesPaths:i,gridLines:u}},mount(i,u){let d=U(()=>{let m=ln(i.series(),[]),n=i.width(),b=i.height(),w=m.flatMap((f)=>f.data),E=w.length?Math.min(...w):0,y=(w.length?Math.max(...w):1)-E||1,L=b-4;i.gridLines.set(i.grid()?[0.25,0.5,0.75].map((f,t)=>({id:`g${t}`,y:2+L*f})):[]);let l=[];for(let f of m){if(f.data.length<2)continue;let t=f.data.map((z)=>(z-E)/y*100),{d:r,points:H}=Un(t,n,b,2);if(!r)continue;let C=un(f.tone);l.push({name:f.name,strokeD:r,color:C,pts:H.map(([z,g],j)=>({id:`${f.name}-${j}`,cx:z,cy:g}))})}i.seriesPaths.set(l)});return()=>d()}});var tr=V("sui-line-chart",{props:{series:P.string({default:"",reflect:!0}),labels:P.string({default:""}),width:P.number({default:300}),height:P.number({default:200}),points:P.bool({default:!1,reflect:!0}),grid:P.bool({default:!1,reflect:!0}),label:P.string({default:"line chart"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.series=u.series,n.labels=u.labels,n.width=u.width,n.height=u.height,n.points=u.points,n.grid=u.grid,n.label=u.label;let b=null;if(ri&&typeof ri.setup==="function"){if(i.scope=n,b=ri.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(H)=>h({anchor:H,items:()=>n.gridLines(),key:(C,z)=>{let g=Object.create(n);return g.g=C,g.__i=z,((j)=>C.id)(g)},render:(C,z)=>{let g=Object.create(n);return g.g=C,g.__i=z,g.__v=J(0),((j)=>{let T=document.createDocumentFragment(),D=[],Z=document.createElement("line");Z.setAttribute("class","grid-line"),Z.setAttribute("x1","0"),D.push(W(Z,"x2",()=>(j.__v(),j.width())));{let X=j.g.y;if(X==null||X===!1||X==="")Z.removeAttribute("y1");else Z.setAttribute("y1",X===!0?"":String(X))}{let X=j.g.y;if(X==null||X===!1||X==="")Z.removeAttribute("y2");else Z.setAttribute("y2",X===!0?"":String(X))}return T.appendChild(Z),{nodes:Array.from(T.childNodes),cleanup:()=>{for(let X of D)try{X()}catch(G){}},update:(X,G)=>{j.g=X,j.__i=G;{let Q=j.g.y;if(Q==null||Q===!1||Q==="")Z.removeAttribute("y1");else Z.setAttribute("y1",Q===!0?"":String(Q))}{let Q=j.g.y;if(Q==null||Q===!1||Q==="")Z.removeAttribute("y2");else Z.setAttribute("y2",Q===!0?"":String(Q))}j.__v.set((Q)=>Q+1)}}})(g)}}),E=(H)=>h({anchor:H,items:()=>n.seriesPaths(),key:(C,z)=>{let g=Object.create(n);return g.s=C,g.__i=z,((j)=>C.name)(g)},render:(C,z)=>{let g=Object.create(n);return g.s=C,g.__i=z,g.__v=J(0),((j)=>{let T=(q)=>h({anchor:q,items:()=>j.s.pts,key:(F,k)=>{let K=Object.create(j);return K.p=F,K.__i=k,((B)=>F.id)(K)},render:(F,k)=>{let K=Object.create(j);return K.p=F,K.__i=k,K.__v=J(0),((B)=>{let S=document.createDocumentFragment(),R=[],v=document.createElement("circle");{let O=B.p.cx;if(O==null||O===!1||O==="")v.removeAttribute("cx");else v.setAttribute("cx",O===!0?"":String(O))}{let O=B.p.cy;if(O==null||O===!1||O==="")v.removeAttribute("cy");else v.setAttribute("cy",O===!0?"":String(O))}v.setAttribute("r","2.5");{let O=B.s.color;if(O==null||O===!1||O==="")v.removeAttribute("fill");else v.setAttribute("fill",O===!0?"":String(O))}return S.appendChild(v),{nodes:Array.from(S.childNodes),cleanup:()=>{for(let O of R)try{O()}catch(x){}},update:(O,x)=>{B.p=O,B.__i=x;{let $=B.p.cx;if($==null||$===!1||$==="")v.removeAttribute("cx");else v.setAttribute("cx",$===!0?"":String($))}{let $=B.p.cy;if($==null||$===!1||$==="")v.removeAttribute("cy");else v.setAttribute("cy",$===!0?"":String($))}{let $=B.s.color;if($==null||$===!1||$==="")v.removeAttribute("fill");else v.setAttribute("fill",$===!0?"":String($))}B.__v.set(($)=>$+1)}}})(K)}}),D=(q)=>I({anchor:q,test:()=>{if(j.points())return 0;return-1},branches:[()=>{let F=document.createDocumentFragment(),k=[],K=document.createComment("for");return F.appendChild(K),k.push(T(K)),{nodes:Array.from(F.childNodes),cleanup:()=>{for(let B of k)try{B()}catch(S){}}}}]}),Z=document.createDocumentFragment(),X=[],G=document.createElement("path");{let q=j.s.strokeD;if(q==null||q===!1||q==="")G.removeAttribute("d");else G.setAttribute("d",q===!0?"":String(q))}G.setAttribute("fill","none");{let q=j.s.color;if(q==null||q===!1||q==="")G.removeAttribute("stroke");else G.setAttribute("stroke",q===!0?"":String(q))}G.setAttribute("stroke-width","1.75"),G.setAttribute("stroke-linecap","round"),G.setAttribute("stroke-linejoin","round"),Z.appendChild(G);let Q=document.createComment("if");return Z.appendChild(Q),X.push(D(Q)),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let q of X)try{q()}catch(F){}},update:(q,F)=>{j.s=q,j.__i=F;{let k=j.s.strokeD;if(k==null||k===!1||k==="")G.removeAttribute("d");else G.setAttribute("d",k===!0?"":String(k))}{let k=j.s.color;if(k==null||k===!1||k==="")G.removeAttribute("stroke");else G.setAttribute("stroke",k===!0?"":String(k))}j.__v.set((k)=>k+1)}}})(g)}}),M=d.shadowRoot??d;Y(M);let y=document.createElement("div");y.setAttribute("class","wrap");let L=document.createElement("svg");m.push(W(L,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),L.setAttribute("preserveaspectratio","none"),L.setAttribute("role","img"),m.push(W(L,"aria-label",()=>n.label()));let l=document.createComment("for");L.appendChild(l),m.push(w(l));let f=document.createComment("for");L.appendChild(f),m.push(E(f)),y.appendChild(L);let t=document.createElement("div");t.setAttribute("class","legend");let r=document.createElement("slot");if(t.appendChild(r),y.appendChild(t),M.appendChild(y),b!=null){let H=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(H){let C=new Proxy(n,{get(z,g){return z[g]},set(z,g,j){let T=z[g];if(T!=null&&typeof T.set==="function"){if(typeof j==="function")T.set(()=>j);else T.set(j);return!0}return!1}});if(H.mount){let z=H.mount(C,d);if(typeof z==="function")m.push(z)}if(H.unmount)m.push(()=>H.unmount(C))}}return n}});var Li=N({scope:()=>{let i=J(!1),u=J(""),d=J(""),m=J("var(--primary)");return{hasData:i,strokePath:u,areaPath:d,lineColor:m}},mount(i,u){let d=U(()=>{let m=qn(i.data()),n=i.width(),b=i.height(),w=un(i.tone());if(i.lineColor.set(w),m.length<2){i.hasData.set(!1),i.strokePath.set(""),i.areaPath.set("");return}let{d:E,points:M}=Un(m,n,b,2);if(i.strokePath.set(E),i.fill()){let y=M[0],L=M[M.length-1];i.areaPath.set(`${E} L${L[0].toFixed(2)},${b} L${y[0].toFixed(2)},${b} Z`)}else i.areaPath.set("");i.hasData.set(!0)});return()=>d()}});var wr=V("sui-sparkline",{props:{data:P.string({default:"",reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),width:P.number({default:80}),height:P.number({default:20}),fill:P.bool({default:!1,reflect:!0}),label:P.string({default:"sparkline"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.data=u.data,n.tone=u.tone,n.width=u.width,n.height=u.height,n.fill=u.fill,n.label=u.label;let b=null;if(Li&&typeof Li.setup==="function"){if(i.scope=n,b=Li.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>I({anchor:L,test:()=>{if(n.fill())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("path");return f.push(W(t,"d",()=>n.areaPath())),f.push(W(t,"fill",()=>n.lineColor())),t.setAttribute("fill-opacity","0.18"),t.setAttribute("stroke","none"),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let r of f)try{r()}catch(H){}}}}]}),E=(L)=>I({anchor:L,test:()=>{if(n.hasData())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("svg");f.push(W(t,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),f.push(W(t,"width",()=>n.width())),f.push(W(t,"height",()=>n.height())),t.setAttribute("role","img"),f.push(W(t,"aria-label",()=>n.label()));let r=document.createComment("if");t.appendChild(r),f.push(w(r));let H=document.createElement("path");f.push(W(H,"d",()=>n.strokePath())),H.setAttribute("fill","none"),f.push(W(H,"stroke",()=>n.lineColor())),H.setAttribute("stroke-width","1.5"),H.setAttribute("stroke-linecap","round"),H.setAttribute("stroke-linejoin","round");let C=document.createTextNode(`
    `);return H.appendChild(C),t.appendChild(H),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let z of f)try{z()}catch(g){}}}},()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("svg");f.push(W(t,"viewbox",()=>"0 0 "+n.width()+" "+n.height())),f.push(W(t,"width",()=>n.width())),f.push(W(t,"height",()=>n.height())),t.setAttribute("aria-hidden","true");let r=document.createTextNode(`
  `);return t.appendChild(r),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let H of f)try{H()}catch(C){}}}}]}),M=d.shadowRoot??d;Y(M);let y=document.createComment("if");if(M.appendChild(y),m.push(E(y)),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var li=N({scope:()=>({segments:J([])}),mount(i,u){let d=U(()=>{let m=ln(i.data(),[]),n=m.reduce((f,t)=>f+Math.max(0,t.value),0);if(n<=0){i.segments.set([]);return}let b=i.size(),w=i.thickness(),E=(b-w)/2,M=b/2,y=b/2,L=2*Math.PI,l=-Math.PI/2;i.segments.set(m.filter((f)=>f.value>0).map((f,t)=>{let r=f.value/n*L,H=r>=L-0.001?L-0.001:r,C=Sn(M,y,E,l,l+H);return l+=r,{id:`s${t}`,d:C,color:un(f.tone),title:`${f.label}: ${f.value}`}}))});return()=>d()}});var Dr=V("sui-donut",{props:{data:P.string({default:"",reflect:!0}),size:P.number({default:120}),thickness:P.number({default:16}),label:P.string({default:"donut chart"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.data=u.data,n.size=u.size,n.thickness=u.thickness,n.label=u.label;let b=null;if(li&&typeof li.setup==="function"){if(i.scope=n,b=li.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(r)=>h({anchor:r,items:()=>n.segments(),key:(H,C)=>{let z=Object.create(n);return z.seg=H,z.__i=C,((g)=>H.id)(z)},render:(H,C)=>{let z=Object.create(n);return z.seg=H,z.__i=C,z.__v=J(0),((g)=>{let j=document.createDocumentFragment(),T=[],D=document.createElement("path");D.setAttribute("part","segment");{let G=g.seg.d;if(G==null||G===!1||G==="")D.removeAttribute("d");else D.setAttribute("d",G===!0?"":String(G))}D.setAttribute("fill","none");{let G=g.seg.color;if(G==null||G===!1||G==="")D.removeAttribute("stroke");else D.setAttribute("stroke",G===!0?"":String(G))}T.push(W(D,"stroke-width",()=>(g.__v(),g.thickness()))),D.setAttribute("stroke-linecap","butt");let Z=document.createElement("title"),X=document.createTextNode("");return Z.appendChild(X),X.textContent=g.seg.title,D.appendChild(Z),j.appendChild(D),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let G of T)try{G()}catch(Q){}},update:(G,Q)=>{g.seg=G,g.__i=Q;{let q=g.seg.d;if(q==null||q===!1||q==="")D.removeAttribute("d");else D.setAttribute("d",q===!0?"":String(q))}{let q=g.seg.color;if(q==null||q===!1||q==="")D.removeAttribute("stroke");else D.setAttribute("stroke",q===!0?"":String(q))}X.textContent=g.seg.title,g.__v.set((q)=>q+1)}}})(z)}}),E=d.shadowRoot??d;Y(E);let M=document.createElement("svg");m.push(W(M,"viewbox",()=>"0 0 "+n.size()+" "+n.size())),M.setAttribute("role","img"),m.push(W(M,"aria-label",()=>n.label())),m.push(W(M,"style",()=>"width:"+n.size()+"px;height:"+n.size()+"px"));let y=document.createElement("circle");y.setAttribute("class","track"),y.setAttribute("part","track"),m.push(W(y,"cx",()=>n.size()/2)),m.push(W(y,"cy",()=>n.size()/2)),m.push(W(y,"r",()=>(n.size()-n.thickness())/2)),y.setAttribute("fill","none"),m.push(W(y,"stroke-width",()=>n.thickness()));let L=document.createTextNode(`
  `);y.appendChild(L),M.appendChild(y);let l=document.createComment("for");M.appendChild(l),m.push(w(l)),E.appendChild(M);let f=document.createElement("div");f.setAttribute("class","center");let t=document.createElement("slot");if(f.appendChild(t),E.appendChild(f),b!=null){let r=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(r){let H=new Proxy(n,{get(C,z){return C[z]},set(C,z,g){let j=C[z];if(j!=null&&typeof j.set==="function"){if(typeof g==="function")j.set(()=>g);else j.set(g);return!0}return!1}});if(r.mount){let C=r.mount(H,d);if(typeof C==="function")m.push(C)}if(r.unmount)m.push(()=>r.unmount(H))}}return n}});var $u=Math.PI/180,Ei=135*$u,hu=405*$u,Mi=N({scope:()=>({trackD:J(""),valueD:J(""),arcColor:J("var(--primary)")}),mount(i,u){let d=U(()=>{let m=i.size(),n=i.thickness(),b=(m-n)/2-2,w=m/2,E=m/2;i.trackD.set(Sn(w,E,b,Ei,hu));let M=i.value(),y=i.min(),L=i.max(),l=Math.max(0,Math.min(1,(M-y)/(L-y||1))),f=Ei+(hu-Ei)*l;i.valueD.set(l>0?Sn(w,E,b,Ei,f):"");let t=i.tone(),r=i.lowThreshold(),H=i.highThreshold(),C=t;if(!C)if(r>0&&M<r)C="destructive";else if(H>0&&M<H)C="warning";else if(r>0||H>0)C="success";else C="primary";i.arcColor.set(un(C))});return()=>d()}});var Qr=V("sui-gauge",{props:{value:P.number({default:0,reflect:!0}),min:P.number({default:0}),max:P.number({default:100}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"",reflect:!0}),"low-threshold":P.number({default:0}),"high-threshold":P.number({default:0}),size:P.number({default:140}),thickness:P.number({default:12}),label:P.string({default:"gauge"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.min=u.min,n.max=u.max,n.tone=u.tone,n.lowThreshold=u.lowThreshold,n.highThreshold=u.highThreshold,n.size=u.size,n.thickness=u.thickness,n.label=u.label;let b=null;if(Mi&&typeof Mi.setup==="function"){if(i.scope=n,b=Mi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(r)=>I({anchor:r,test:()=>{if(n.valueD())return 0;return-1},branches:[()=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("path");z.setAttribute("part","fill"),C.push(W(z,"d",()=>n.valueD())),z.setAttribute("fill","none"),C.push(W(z,"stroke",()=>n.arcColor())),C.push(W(z,"stroke-width",()=>n.thickness())),z.setAttribute("stroke-linecap","round");let g=document.createTextNode(`
    `);return z.appendChild(g),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let j of C)try{j()}catch(T){}}}}]}),E=d.shadowRoot??d;Y(E);let M=document.createElement("svg");m.push(W(M,"viewbox",()=>"0 0 "+n.size()+" "+n.size())),M.setAttribute("role","img"),m.push(W(M,"aria-label",()=>n.label())),m.push(W(M,"style",()=>"width:"+n.size()+"px;height:"+n.size()+"px"));let y=document.createElement("path");y.setAttribute("class","track"),y.setAttribute("part","track"),m.push(W(y,"d",()=>n.trackD())),y.setAttribute("fill","none"),m.push(W(y,"stroke-width",()=>n.thickness())),y.setAttribute("stroke-linecap","round");let L=document.createTextNode(`
  `);y.appendChild(L),M.appendChild(y);let l=document.createComment("if");M.appendChild(l),m.push(w(l)),E.appendChild(M);let f=document.createElement("div");f.setAttribute("class","gauge-label");let t=document.createElement("slot");if(f.appendChild(t),E.appendChild(f),b!=null){let r=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(r){let H=new Proxy(n,{get(C,z){return C[z]},set(C,z,g){let j=C[z];if(j!=null&&typeof j.set==="function"){if(typeof g==="function")j.set(()=>g);else j.set(g);return!0}return!1}});if(r.mount){let C=r.mount(H,d);if(typeof C==="function")m.push(C)}if(r.unmount)m.push(()=>r.unmount(H))}}return n}});var yi=N({scope:()=>({bars:J([])}),mount(i,u){let d=U(()=>{let m=ln(i.segments(),[]),n=m.reduce((M,y)=>M+Math.max(0,y.count),0),b=i.total(),w=b>0?b:n,E=w<=0?1:w;i.bars.set(m.filter((M)=>M.count>0).map((M,y)=>{let L=Math.max(0,M.count)/E*100;return{id:`b${y}`,style:`width:${L.toFixed(3)}%;background:${un(M.tone)};`,title:`${M.label??M.tone}: ${M.count}`}}))});return()=>d()}});var Fr=V("sui-meter",{props:{segments:P.string({default:"",reflect:!0}),total:P.number({default:0}),label:P.string({default:"meter"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.segments=u.segments,n.total=u.total,n.label=u.label;let b=null;if(yi&&typeof yi.setup==="function"){if(i.scope=n,b=yi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(t)=>h({anchor:t,items:()=>n.bars(),key:(r,H)=>{let C=Object.create(n);return C.b=r,C.__i=H,((z)=>r.id)(C)},render:(r,H)=>{let C=Object.create(n);return C.b=r,C.__i=H,C.__v=J(0),((z)=>{let g=document.createDocumentFragment(),j=[],T=document.createElement("div");T.setAttribute("class","seg"),T.setAttribute("part","segment");{let D=z.b.style;if(D==null||D===!1||D==="")T.removeAttribute("style");else T.setAttribute("style",D===!0?"":String(D))}{let D=z.b.title;if(D==null||D===!1||D==="")T.removeAttribute("title");else T.setAttribute("title",D===!0?"":String(D))}return g.appendChild(T),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let D of j)try{D()}catch(Z){}},update:(D,Z)=>{z.b=D,z.__i=Z;{let X=z.b.style;if(X==null||X===!1||X==="")T.removeAttribute("style");else T.setAttribute("style",X===!0?"":String(X))}{let X=z.b.title;if(X==null||X===!1||X==="")T.removeAttribute("title");else T.setAttribute("title",X===!0?"":String(X))}z.__v.set((X)=>X+1)}}})(C)}}),E=d.shadowRoot??d;Y(E);let M=document.createElement("div");M.setAttribute("class","wrap");let y=document.createElement("div");y.setAttribute("class","bar"),y.setAttribute("part","bar"),y.setAttribute("role","img"),m.push(W(y,"aria-label",()=>n.label()));let L=document.createComment("for");y.appendChild(L),m.push(w(L)),M.appendChild(y);let l=document.createElement("div");l.setAttribute("class","legend");let f=document.createElement("slot");if(l.appendChild(f),M.appendChild(l),E.appendChild(M),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let r=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let g=H[C];if(g!=null&&typeof g.set==="function"){if(typeof z==="function")g.set(()=>z);else g.set(z);return!0}return!1}});if(t.mount){let H=t.mount(r,d);if(typeof H==="function")m.push(H)}if(t.unmount)m.push(()=>t.unmount(r))}}return n}});var wi=N({scope:()=>({cells:J([]),svgW:J(0),svgH:J(0)}),mount(i,u){let d=U(()=>{let m=an(i.rows()),n=an(i.cols()),b=i.cellSize(),w=i.gap(),E=i.data(),M=ln(E,[]),y=Array.isArray(M)&&M.length>0&&Array.isArray(M[0]),L;if(y)L=M;else{let T=qn(E),D=n.length||1,Z=m.length||(T.length>0?Math.ceil(T.length/D):0);L=Array.from({length:Z},(X,G)=>T.slice(G*D,(G+1)*D))}let l=L.length,f=l>0?Math.max(...L.map((T)=>T.length)):0,t=L.flatMap((T)=>T.filter((D)=>typeof D==="number")),r=t.length?Math.min(...t):0,C=(t.length?Math.max(...t):1)-r,z=un(i.tone()),g=b+w;i.svgW.set(f>0?f*b+(f-1)*w:0),i.svgH.set(l>0?l*b+(l-1)*w:0);let j=[];for(let T=0;T<l;T++){let D=L[T]??[];for(let Z=0;Z<f;Z++){let X=D[Z]??0,G=C===0?r>0?1:0:(X-r)/C,Q=(G*100).toFixed(1),q=G>0?`fill:color-mix(in oklch,${z} ${Q}%,transparent)`:"";j.push({id:`c${T}-${Z}`,x:Z*g,y:T*g,w:b,h:b,fill:q})}}i.cells.set(j)});return()=>d()}});var ar=V("sui-heatmap",{props:{data:P.string({default:"",reflect:!0}),rows:P.string({default:""}),cols:P.string({default:""}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"primary",reflect:!0}),"cell-size":P.number({default:20}),gap:P.number({default:3}),label:P.string({default:"heatmap"})},shadow:!0,styles:`
  :host {
    display: inline-block;
    font-family: var(--font-body);
  }
  svg { display: block; overflow: visible; }
  .cell { fill: var(--muted); }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.data=u.data,n.rows=u.rows,n.cols=u.cols,n.tone=u.tone,n.cellSize=u.cellSize,n.gap=u.gap,n.label=u.label;let b=null;if(wi&&typeof wi.setup==="function"){if(i.scope=n,b=wi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>h({anchor:L,items:()=>n.cells(),key:(l,f)=>{let t=Object.create(n);return t.cell=l,t.__i=f,((r)=>l.id)(t)},render:(l,f)=>{let t=Object.create(n);return t.cell=l,t.__i=f,t.__v=J(0),((r)=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("rect");z.setAttribute("class","cell"),z.setAttribute("part","cell");{let j=r.cell.x;if(j==null||j===!1||j==="")z.removeAttribute("x");else z.setAttribute("x",j===!0?"":String(j))}{let j=r.cell.y;if(j==null||j===!1||j==="")z.removeAttribute("y");else z.setAttribute("y",j===!0?"":String(j))}{let j=r.cell.w;if(j==null||j===!1||j==="")z.removeAttribute("width");else z.setAttribute("width",j===!0?"":String(j))}{let j=r.cell.h;if(j==null||j===!1||j==="")z.removeAttribute("height");else z.setAttribute("height",j===!0?"":String(j))}z.setAttribute("rx","2");{let j=r.cell.fill;if(j==null||j===!1||j==="")z.removeAttribute("style");else z.setAttribute("style",j===!0?"":String(j))}let g=document.createTextNode(`
    `);return z.appendChild(g),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let j of C)try{j()}catch(T){}},update:(j,T)=>{r.cell=j,r.__i=T;{let D=r.cell.x;if(D==null||D===!1||D==="")z.removeAttribute("x");else z.setAttribute("x",D===!0?"":String(D))}{let D=r.cell.y;if(D==null||D===!1||D==="")z.removeAttribute("y");else z.setAttribute("y",D===!0?"":String(D))}{let D=r.cell.w;if(D==null||D===!1||D==="")z.removeAttribute("width");else z.setAttribute("width",D===!0?"":String(D))}{let D=r.cell.h;if(D==null||D===!1||D==="")z.removeAttribute("height");else z.setAttribute("height",D===!0?"":String(D))}{let D=r.cell.fill;if(D==null||D===!1||D==="")z.removeAttribute("style");else z.setAttribute("style",D===!0?"":String(D))}r.__v.set((D)=>D+1)}}})(t)}}),E=d.shadowRoot??d;Y(E);let M=document.createElement("svg");m.push(W(M,"viewbox",()=>"0 0 "+n.svgW()+" "+n.svgH())),M.setAttribute("role","img"),m.push(W(M,"aria-label",()=>n.label()));let y=document.createComment("for");if(M.appendChild(y),m.push(w(y)),E.appendChild(M),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var xr=V("sui-button",{props:{tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["solid","soft","outline","ghost","default"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),type:P.union(["button","submit","reset"],{default:"button",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),loading:P.bool({default:!1,reflect:!0}),label:P.string({default:""}),pressed:P.string({default:"",reflect:!0}),expanded:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
  `,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.variant=u.variant,n.size=u.size,n.type=u.type,n.disabled=u.disabled,n.loading=u.loading,n.label=u.label,n.pressed=u.pressed,n.expanded=u.expanded;let b=(t)=>I({anchor:t,test:()=>{if(n.loading())return 0;return-1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("span");C.setAttribute("class","spinner-wrap"),C.setAttribute("aria-hidden","true");let z=document.createElement("span");return z.setAttribute("class","spinner"),C.appendChild(z),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let g of H)try{g()}catch(j){}}}}]}),w=d.shadowRoot??d;Y(w);let E=document.createElement("button");E.setAttribute("part","button"),m.push(W(E,"type",()=>n.type())),m.push(W(E,"disabled",()=>n.disabled()||n.loading())),m.push(W(E,"aria-busy",()=>n.loading()?"true":"false")),m.push(W(E,"aria-disabled",()=>n.disabled()||n.loading()?"true":"false")),m.push(W(E,"aria-label",()=>n.label())),m.push(W(E,"aria-pressed",()=>n.pressed())),m.push(W(E,"aria-expanded",()=>n.expanded()));let M=document.createElement("span");M.setAttribute("class","content");let y=document.createElement("slot");y.setAttribute("name","leading"),M.appendChild(y);let L=document.createElement("slot");m.push(dn(L,d,"data-icon-only")),M.appendChild(L);let l=document.createElement("slot");l.setAttribute("name","trailing"),M.appendChild(l),E.appendChild(M);let f=document.createComment("if");return E.appendChild(f),m.push(b(f)),w.appendChild(E),n}});var Hi=N({mount(i,u){let d=i,m=J(0),n=()=>{let f=d.tabSlot();if(!f)return Array.from(u.querySelectorAll(":scope > sui-tab"));return f.assignedElements().filter((t)=>t.tagName.toLowerCase()==="sui-tab")},b=(f)=>{if(d.active()===f)return;d.active.set(f),u.dispatchEvent(new CustomEvent("sui-tab-change",{detail:{name:f},bubbles:!0,composed:!0}))},w=U(()=>{m();let f=d.active(),t=n(),r=f;if(!r&&t.length>0){if(r=t[0].getAttribute("name")??"",r)d.active.set(r)}for(let H of t){let C=H.getAttribute("name")??"";if(C===r)H.setAttribute("active","");else H.removeAttribute("active");H.id=`sui-tab-${C}`,H.setAttribute("aria-controls",`sui-tabpanel-${C}`)}for(let H of Array.from(u.querySelectorAll(':scope > sui-tab-panel, :scope > [role="tabpanel"]')))if((H.getAttribute("name")??"")===r)H.removeAttribute("hidden"),H.setAttribute("active","");else H.setAttribute("hidden",""),H.removeAttribute("active")}),E=(f)=>{let t=d.orientation()==="vertical",r=t?"ArrowDown":"ArrowRight",H=t?"ArrowUp":"ArrowLeft",C=[r,H,"Home","End"],z=["Enter"," "];if(!C.includes(f.key)&&!z.includes(f.key))return;f.preventDefault();let g=n().filter((G)=>!G.hasAttribute("disabled"));if(!g.length)return;let j=g.map((G)=>G.getAttribute("name")??""),T=d.active(),D=Math.max(0,j.indexOf(T)),Z=D;if(f.key===r)Z=(D+1)%j.length;else if(f.key===H)Z=(D-1+j.length)%j.length;else if(f.key==="Home")Z=0;else if(f.key==="End")Z=j.length-1;let X=j[Z];b(X),queueMicrotask(()=>{let G=g[Z],Q=G.shadowRoot?.querySelector("button");Q?Q.focus():G.focus()})},M=(f)=>{let t=f.target.closest("sui-tab");if(!t||t.hasAttribute("disabled"))return;let r=t.getAttribute("name")??"";if(r)b(r)},y=()=>m.set(m()+1),L=d.strip(),l=d.tabSlot();if(L)L.addEventListener("keydown",E),L.addEventListener("click",M);if(l)l.addEventListener("slotchange",y);return()=>{if(w(),L)L.removeEventListener("keydown",E),L.removeEventListener("click",M);if(l)l.removeEventListener("slotchange",y)}}});var pr=V("sui-tabs",{props:{active:P.string({default:"",reflect:!0}),variant:P.union(["underline","pill","enclosed"],{default:"underline",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),sticky:P.bool({default:!1,reflect:!0}),orientation:P.union(["horizontal","vertical"],{default:"horizontal",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.active=u.active,n.variant=u.variant,n.size=u.size,n.tone=u.tone,n.sticky=u.sticky,n.orientation=u.orientation,n.strip=J(null),n.tabSlot=J(null);let b=null;if(Hi&&typeof Hi.setup==="function"){if(i.scope=n,b=Hi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","strip"),E.setAttribute("part","strip"),E.setAttribute("role","tablist"),m.push(W(E,"aria-orientation",()=>n.orientation())),A(n,"strip",E);let M=document.createElement("slot");if(A(n,"tabSlot",M),E.appendChild(M),w.appendChild(E),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let L=new Proxy(n,{get(l,f){return l[f]},set(l,f,t){let r=l[f];if(r!=null&&typeof r.set==="function"){if(typeof t==="function")r.set(()=>t);else r.set(t);return!0}return!1}});if(y.mount){let l=y.mount(L,d);if(typeof l==="function")m.push(l)}if(y.unmount)m.push(()=>y.unmount(L))}}return n}});var Ci=N({mount(i,u){let d=i,m=U(()=>{let n=d.btn();if(!n)return;n.tabIndex=d.active()?0:-1});return()=>{m()}}});var dL=V("sui-tab",{props:{name:P.string({default:"",reflect:!0}),label:P.string({default:"",reflect:!0}),count:P.string({default:"",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),active:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.label=u.label,n.count=u.count,n.disabled=u.disabled,n.active=u.active,n.btn=J(null);let b=null;if(Ci&&typeof Ci.setup==="function"){if(i.scope=n,b=Ci.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(t)=>I({anchor:t,test:()=>{if(n.count()!=="")return 0;return-1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("span");C.setAttribute("class","count"),C.setAttribute("aria-hidden","true");let z=document.createTextNode("");return C.appendChild(z),H.push(a(z,()=>n.count())),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let g of H)try{g()}catch(j){}}}}]}),E=d.shadowRoot??d;Y(E);let M=document.createElement("button");M.setAttribute("part","button"),M.setAttribute("type","button"),M.setAttribute("role","tab"),m.push(W(M,"data-name",()=>n.name())),m.push(W(M,"aria-selected",()=>n.active()?"true":"false")),m.push(W(M,"aria-disabled",()=>n.disabled()?"true":"false")),m.push(W(M,"disabled",()=>n.disabled())),A(n,"btn",M);let y=document.createElement("span");y.setAttribute("class","leading"),y.setAttribute("part","leading");let L=document.createElement("slot");L.setAttribute("name","leading"),y.appendChild(L),M.appendChild(y);let l=document.createElement("slot");M.appendChild(l);let f=document.createComment("if");if(M.appendChild(f),m.push(w(f)),E.appendChild(M),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let r=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let g=H[C];if(g!=null&&typeof g.set==="function"){if(typeof z==="function")g.set(()=>z);else g.set(z);return!0}return!1}});if(t.mount){let H=t.mount(r,d);if(typeof H==="function")m.push(H)}if(t.unmount)m.push(()=>t.unmount(r))}}return n}});var fL=V("sui-empty-state",{props:{variant:P.union(["default","inline","placeholder","connecting"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"muted",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.variant=u.variant,n.size=u.size,n.tone=u.tone;let b=d.shadowRoot??d;Y(b);let w=document.createElement("div");w.setAttribute("class","box"),w.setAttribute("part","box"),m.push(W(w,"role",()=>n.variant()==="connecting"?"status":"note")),m.push(W(w,"aria-live",()=>n.variant()==="connecting"?"polite":"off"));let E=document.createElement("div");E.setAttribute("class","icon"),E.setAttribute("part","icon"),E.setAttribute("aria-hidden","true");let M=document.createElement("slot");M.setAttribute("name","icon"),m.push(dn(M,d,"data-no-icon")),E.appendChild(M),w.appendChild(E);let y=document.createElement("div");y.setAttribute("class","copy");let L=document.createElement("div");L.setAttribute("class","title"),L.setAttribute("part","title");let l=document.createElement("slot");l.setAttribute("name","title"),L.appendChild(l),y.appendChild(L);let f=document.createElement("div");f.setAttribute("class","desc"),f.setAttribute("part","desc");let t=document.createElement("slot");f.appendChild(t),y.appendChild(f),w.appendChild(y);let r=document.createElement("div");r.setAttribute("class","actions"),r.setAttribute("part","actions");let H=document.createElement("slot");return H.setAttribute("name","action"),r.appendChild(H),w.appendChild(r),b.appendChild(w),n}});var lm=0,zi=N({mount(i,u){let d=i,m=d.inputEl();if(!m)return;u.expose({focus:()=>m.focus(),blur:()=>m.blur(),clear:()=>{m.value="",d.value.set("")},reportValidity:()=>m.reportValidity()}),c(m,"input",()=>u.emit("sui-input",{value:m.value})),c(m,"change",()=>u.emit("sui-input-change",{value:m.value}));let n=`sui-input-err-${++lm}`;yn(W(m,"aria-describedby",()=>{let b=[];if(d.invalid()&&d.errorMessage()){let E=d.errorEl,M=typeof E==="function"?E():null;if(M)M.id=n,b.push(n)}let w=d.describedBy();if(w)b.push(w);return b.length>0?b.join(" "):null}))}});var ML=V("sui-input",{props:{type:P.union(["text","search","email","url","tel","password","number"],{default:"text",reflect:!0}),value:P.string({default:""}),placeholder:P.string({default:""}),name:P.string({default:""}),autocomplete:P.string({default:""}),required:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),readonly:P.bool({default:!1,reflect:!0}),invalid:P.bool({default:!1,reflect:!0}),loading:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["default","ghost"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),label:P.string({default:""}),"described-by":P.string({default:""}),"error-message":P.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.type=u.type,n.value=u.value,n.placeholder=u.placeholder,n.name=u.name,n.autocomplete=u.autocomplete,n.required=u.required,n.disabled=u.disabled,n.readonly=u.readonly,n.invalid=u.invalid,n.loading=u.loading,n.tone=u.tone,n.variant=u.variant,n.size=u.size,n.label=u.label,n.describedBy=u.describedBy,n.errorMessage=u.errorMessage,n.errorEl=J(null),n.inputEl=J(null);let b=null;if(zi&&typeof zi.setup==="function"){if(i.scope=n,b=zi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(r)=>I({anchor:r,test:()=>{if(n.loading())return 0;return 1},branches:[()=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("span");return z.setAttribute("class","spinner"),z.setAttribute("aria-hidden","true"),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let g of C)try{g()}catch(j){}}}},()=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("slot");return z.setAttribute("name","trailing"),C.push(dn(z,d,"data-no-trailing")),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let g of C)try{g()}catch(j){}}}}]}),E=(r)=>I({anchor:r,test:()=>{if(n.invalid()&&n.errorMessage())return 0;return-1},branches:[()=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("span");z.setAttribute("class","error"),z.setAttribute("role","alert"),A(n,"errorEl",z);let g=document.createTextNode("");return z.appendChild(g),C.push(a(g,()=>n.errorMessage())),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let j of C)try{j()}catch(T){}}}}]}),M=d.shadowRoot??d;Y(M);let y=document.createElement("div");y.setAttribute("class","wrap"),y.setAttribute("part","wrap");let L=document.createElement("slot");L.setAttribute("name","leading"),m.push(dn(L,d,"data-no-leading")),y.appendChild(L);let l=document.createElement("input");l.setAttribute("part","input"),m.push(W(l,"type",()=>n.type())),m.push(W(l,"placeholder",()=>n.placeholder())),m.push(W(l,"name",()=>n.name())),m.push(W(l,"autocomplete",()=>n.autocomplete()||"off")),m.push(W(l,"disabled",()=>n.disabled())),m.push(W(l,"readonly",()=>n.readonly())),m.push(W(l,"required",()=>n.required())),m.push(W(l,"aria-label",()=>n.label())),m.push(W(l,"aria-invalid",()=>n.invalid()?"true":"false")),m.push(W(l,"aria-required",()=>n.required()?"true":"false")),m.push(W(l,"aria-busy",()=>n.loading()?"true":"false")),m.push(Pn(l,n.value)),A(n,"inputEl",l),y.appendChild(l);let f=document.createComment("if");y.appendChild(f),m.push(w(f)),M.appendChild(y);let t=document.createComment("if");if(M.appendChild(t),m.push(E(t)),b!=null){let r=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(r){let H=new Proxy(n,{get(C,z){return C[z]},set(C,z,g){let j=C[z];if(j!=null&&typeof j.set==="function"){if(typeof g==="function")j.set(()=>g);else j.set(g);return!0}return!1}});if(r.mount){let C=r.mount(H,d);if(typeof C==="function")m.push(C)}if(r.unmount)m.push(()=>r.unmount(H))}}return n}});var Em={git:"git-branch",refresh:"refresh-cw",stop:"stop-circle",warning:"triangle-alert"};var wn=null,vn=null;function gi(){return wn}function ou(){if(wn)return Promise.resolve(wn);if(vn)return vn;return vn=fetch("/_sui/lucide-icons.json").then((i)=>{if(!i.ok)throw Error(`lucide registry fetch ${i.status}`);return i.json()}).then((i)=>{return wn=i,i}).catch((i)=>{throw vn=null,i}),vn}function eu(i){if(!i||!wn)return;if(wn[i])return i;let u=Em[i];if(u&&wn[u])return u;return}function su(i){return wn?wn[i]:void 0}var pu="http://www.w3.org/2000/svg",wu=J(gi()?1:0),yu=!1;function Mm(){if(yu||gi())return;yu=!0,ou().then(()=>wu.set(wu()+1)).catch(()=>{yu=!1})}function ym(i){let u=i.createElementNS(pu,"svg");return u.setAttribute("part","svg"),u.setAttribute("class","sui-icon-svg"),u.setAttribute("viewBox","0 0 24 24"),u.setAttribute("fill","none"),u.setAttribute("stroke","currentColor"),u.setAttribute("stroke-width","2"),u.setAttribute("stroke-linecap","round"),u.setAttribute("stroke-linejoin","round"),u}function wm(i,u){let d=new DOMParser().parseFromString(`<svg xmlns="${pu}">${u}</svg>`,"image/svg+xml").documentElement,m=[];while(d.firstChild)m.push(i.ownerDocument.adoptNode(d.firstChild));i.replaceChildren(...m)}var Pi=N({mount(i,u){let d=i,m=u.shadowRoot,n=null,b=null,w=U(()=>{let M=d.label();if(M)u.setAttribute("role","img"),u.setAttribute("aria-label",M),u.removeAttribute("aria-hidden");else u.setAttribute("aria-hidden","true"),u.removeAttribute("role"),u.removeAttribute("aria-label")}),E=U(()=>{wu();let M=d.name(),y=eu(M)??null,L=d.label();if(!y){if(n)n.remove(),n=null;if(b=null,M&&!gi())Mm();return}if(!n)n=ym(u.ownerDocument),m.insertBefore(n,m.firstChild);if(y!==b)wm(n,su(y)),b=y;if(L)n.removeAttribute("aria-hidden");else n.setAttribute("aria-hidden","true")});return()=>{w(),E()}}});var jL=V("sui-icon",{props:{name:P.string({default:"",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),tone:P.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),label:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.size=u.size,n.tone=u.tone,n.label=u.label;let b=null;if(Pi&&typeof Pi.setup==="function"){if(i.scope=n,b=Pi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");if(w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var ji=N({mount(i,u){let d=i,m=()=>Array.from(u.querySelectorAll(":scope > option, :scope > optgroup > option")),n=()=>{let t=d.value(),r=m();if(r.length===0)return t;let H=r.find((z)=>z.value===t)??r.find((z)=>z.hasAttribute("selected"))??r[0];return(H?.textContent??"").trim()||H?.value||t},b=()=>{let t=d.selectEl();if(!t){let H=m(),C=H.find((z)=>z.value===d.value())??H.find((z)=>z.hasAttribute("selected"))??H[0];if(!C||C.value==="")u.setAttribute("data-placeholder","");else u.removeAttribute("data-placeholder");return}let r=t.selectedOptions?.[0]??t.options[t.selectedIndex];if(!r||r.value==="")u.setAttribute("data-placeholder","");else u.removeAttribute("data-placeholder")},w=()=>{let t=d.selectEl();if(!t)return;let r=u.querySelectorAll(":scope > option, :scope > optgroup");t.replaceChildren(...Array.from(r,(H)=>H.cloneNode(!0))),t.value=d.value()},E=()=>{let t=d.valueLabel();if(t)t.textContent=n()},M=U(()=>{let t=d.value(),r=d.selectEl();if(r)r.value=t;E(),b()}),y=(t)=>{let r=t.target.value;d.value.set(r),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:r},bubbles:!0,composed:!0}))},L=d.selectEl();if(L)L.addEventListener("change",y);let l=d.optionsSlot(),f=()=>{w(),E(),b()};if(l)l.addEventListener("slotchange",f);return w(),E(),b(),()=>{if(L)L.removeEventListener("change",y);if(l)l.removeEventListener("slotchange",f);M()}}});var GL=V("sui-select",{props:{value:P.string({default:""}),name:P.string({default:""}),required:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),invalid:P.bool({default:!1,reflect:!0}),tone:P.union(["default","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),label:P.string({default:""}),"described-by":P.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.name=u.name,n.required=u.required,n.disabled=u.disabled,n.invalid=u.invalid,n.tone=u.tone,n.size=u.size,n.label=u.label,n.describedBy=u.describedBy,n.optionsSlot=J(null),n.selectEl=J(null),n.valueLabel=J(null);let b=null;if(ji&&typeof ji.setup==="function"){if(i.scope=n,b=ji.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","field"),E.setAttribute("part","field");let M=document.createElement("span");M.setAttribute("class","value"),M.setAttribute("aria-hidden","true"),A(n,"valueLabel",M),E.appendChild(M);let y=document.createElement("svg");y.setAttribute("class","chevron"),y.setAttribute("part","chevron"),y.setAttribute("viewbox","0 0 16 16"),y.setAttribute("aria-hidden","true"),y.setAttribute("focusable","false"),y.setAttribute("fill","none"),y.setAttribute("stroke","currentColor"),y.setAttribute("stroke-width","1.75"),y.setAttribute("stroke-linecap","round"),y.setAttribute("stroke-linejoin","round");let L=document.createElement("path");L.setAttribute("d","M4 6l4 4 4-4"),y.appendChild(L),E.appendChild(y);let l=document.createElement("select");l.setAttribute("part","select"),m.push(W(l,"name",()=>n.name())),m.push(W(l,"disabled",()=>n.disabled())),m.push(W(l,"required",()=>n.required())),m.push(W(l,"aria-invalid",()=>n.invalid()?"true":"false")),m.push(W(l,"aria-label",()=>n.label())),m.push(W(l,"aria-describedby",()=>n.describedBy())),A(n,"selectEl",l),E.appendChild(l),w.appendChild(E);let f=document.createElement("span");f.setAttribute("hidden","");let t=document.createElement("slot");if(A(n,"optionsSlot",t),f.appendChild(t),w.appendChild(f),b!=null){let r=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(r){let H=new Proxy(n,{get(C,z){return C[z]},set(C,z,g){let j=C[z];if(j!=null&&typeof j.set==="function"){if(typeof g==="function")j.set(()=>g);else j.set(g);return!0}return!1}});if(r.mount){let C=r.mount(H,d);if(typeof C==="function")m.push(C)}if(r.unmount)m.push(()=>r.unmount(H))}}return n}});var VL=V("sui-separator",{props:{orientation:P.union(["horizontal","vertical"],{default:"horizontal",reflect:!0}),variant:P.union(["solid","dashed"],{default:"solid",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.orientation=u.orientation,n.variant=u.variant;let b=d.shadowRoot??d;Y(b);let w=document.createElement("hr");return w.setAttribute("class","sep"),w.setAttribute("part","sep"),w.setAttribute("role","separator"),m.push(W(w,"aria-orientation",()=>n.orientation())),b.appendChild(w),n}});var Di=N({mount(i,u){let d=i,m=U(()=>{let n=d.labelEl();if(!n)return;n.htmlFor=d.for?.()??""});return()=>{m()}}});var FL=V("sui-label",{props:{for:P.string({default:""}),size:P.union(["sm","md"],{default:"sm",reflect:!0}),required:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.for=u.for,n.size=u.size,n.required=u.required,n.labelEl=J(null);let b=null;if(Di&&typeof Di.setup==="function"){if(i.scope=n,b=Di.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>I({anchor:l,test:()=>{if(n.required())return 0;return-1},branches:[()=>{let f=document.createDocumentFragment(),t=[],r=document.createElement("span");r.setAttribute("class","req"),r.setAttribute("aria-hidden","true");let H=document.createTextNode("*");return r.appendChild(H),f.appendChild(r),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let C of t)try{C()}catch(z){}}}}]}),E=d.shadowRoot??d;Y(E);let M=document.createElement("label");M.setAttribute("part","label"),A(n,"labelEl",M);let y=document.createElement("slot");M.appendChild(y);let L=document.createComment("if");if(M.appendChild(L),m.push(w(L)),E.appendChild(M),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let f=new Proxy(n,{get(t,r){return t[r]},set(t,r,H){let C=t[r];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(l.mount){let t=l.mount(f,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(f))}}return n}});var Zi=N({mount(i,u){let d=i,m=U(()=>{let n=d.closeBtn();if(!n)return;let b=()=>{if(!d.dismissible())return;u.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),u.remove()};return n.addEventListener("click",b),()=>n.removeEventListener("click",b)});return()=>{m()}}});var kL=V("sui-alert",{props:{tone:P.union(["info","success","warning","destructive","muted"],{default:"info",reflect:!0}),dismissible:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.dismissible=u.dismissible,n.closeBtn=J(null);let b=null;if(Zi&&typeof Zi.setup==="function"){if(i.scope=n,b=Zi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","box"),E.setAttribute("part","box"),m.push(W(E,"role",()=>n.tone()==="destructive"||n.tone()==="warning"?"alert":"status")),m.push(W(E,"aria-live",()=>n.tone()==="destructive"||n.tone()==="warning"?"assertive":"polite")),E.setAttribute("aria-atomic","true");let M=document.createElement("span"),y=document.createElement("slot");y.setAttribute("name","icon"),M.appendChild(y),E.appendChild(M);let L=document.createElement("div");L.setAttribute("class","content");let l=document.createElement("div");l.setAttribute("class","title");let f=document.createElement("slot");f.setAttribute("name","title"),l.appendChild(f),L.appendChild(l);let t=document.createElement("div");t.setAttribute("class","body");let r=document.createElement("slot");t.appendChild(r),L.appendChild(t);let H=document.createElement("div");H.setAttribute("class","actions");let C=document.createElement("slot");C.setAttribute("name","actions"),H.appendChild(C),L.appendChild(H),E.appendChild(L);let z=document.createElement("button");z.setAttribute("class","close"),z.setAttribute("part","close"),z.setAttribute("aria-label","Dismiss"),A(n,"closeBtn",z);let g=document.createTextNode("×");if(z.appendChild(g),E.appendChild(z),w.appendChild(E),b!=null){let j=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(j){let T=new Proxy(n,{get(D,Z){return D[Z]},set(D,Z,X){let G=D[Z];if(G!=null&&typeof G.set==="function"){if(typeof X==="function")G.set(()=>X);else G.set(X);return!0}return!1}});if(j.mount){let D=j.mount(T,d);if(typeof D==="function")m.push(D)}if(j.unmount)m.push(()=>j.unmount(T))}}return n}});var Hm=0,Ti=N({mount(i,u){let d=i,m=`sui-tooltip-${++Hm}`,n=U(()=>{let M=d.bubbleEl();if(M)M.id=m}),b=()=>{let M=d.defaultSlot();if(!M)return;let y=M.assignedElements({flatten:!0})[0];if(!y)return;let L=y.getAttribute("aria-describedby")??"";if(!L.split(/\s+/).includes(m))y.setAttribute("aria-describedby",L?`${L} ${m}`:m)},w=U(()=>{let M=d.defaultSlot();if(!M)return;return M.addEventListener("slotchange",b),b(),()=>M.removeEventListener("slotchange",b)}),E=(M)=>{if(M.key==="Escape")u.blur()};return u.addEventListener("keydown",E),()=>{n(),w(),u.removeEventListener("keydown",E)}}});var $L=V("sui-tooltip",{props:{placement:P.union(["top","bottom","left","right"],{default:"top",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.placement=u.placement,n.bubbleEl=J(null),n.defaultSlot=J(null);let b=null;if(Ti&&typeof Ti.setup==="function"){if(i.scope=n,b=Ti.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");A(n,"defaultSlot",E),w.appendChild(E);let M=document.createElement("span");M.setAttribute("class","bubble"),M.setAttribute("role","tooltip"),M.setAttribute("part","bubble"),A(n,"bubbleEl",M);let y=document.createElement("slot");if(y.setAttribute("name","content"),M.appendChild(y),w.appendChild(M),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var Cm=0,Ji=N({mount(i,u){let d=i,m=`sui-collapsible-${++Cm}`,n=U(()=>{let l=d.contentWrap();if(l)l.id=m}),b=()=>{let l=d.triggerSlot();if(!l)return;let f=d.open();for(let t of l.assignedElements()){let r=t;if(r.setAttribute("aria-expanded",String(f)),r.setAttribute("aria-controls",m),!r.hasAttribute("tabindex")&&!r.matches("button, a, input, select, textarea"))r.setAttribute("tabindex","0")}},w=U(()=>{let l=d.triggerSlot();if(!l)return;return l.addEventListener("slotchange",b),b(),()=>l.removeEventListener("slotchange",b)}),E=U(()=>{d.open(),d.triggerSlot(),b()}),M=()=>{if(d.disabled())return;let l=!d.open();d.open.set(l),u.dispatchEvent(new CustomEvent("sui-toggle",{detail:{open:l},bubbles:!0,composed:!0}))},y=(l)=>{if(l.key==="Enter"||l.key===" ")l.preventDefault(),M()},L=U(()=>{let l=d.triggerWrap();if(!l)return;return l.addEventListener("click",M),l.addEventListener("keydown",y),()=>{l.removeEventListener("click",M),l.removeEventListener("keydown",y)}});return()=>{n(),w(),E(),L()}}});var _L=V("sui-collapsible",{props:{open:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.disabled=u.disabled,n.contentWrap=J(null),n.triggerSlot=J(null),n.triggerWrap=J(null);let b=null;if(Ji&&typeof Ji.setup==="function"){if(i.scope=n,b=Ji.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","trigger-wrap"),A(n,"triggerWrap",E);let M=document.createElement("slot");M.setAttribute("name","trigger"),A(n,"triggerSlot",M),E.appendChild(M),w.appendChild(E);let y=document.createElement("div");y.setAttribute("class","content-wrap"),y.setAttribute("part","content"),A(n,"contentWrap",y);let L=document.createElement("div");L.setAttribute("class","content-inner");let l=document.createElement("slot");if(L.appendChild(l),y.appendChild(L),w.appendChild(y),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let t=new Proxy(n,{get(r,H){return r[H]},set(r,H,C){let z=r[H];if(z!=null&&typeof z.set==="function"){if(typeof C==="function")z.set(()=>C);else z.set(C);return!0}return!1}});if(f.mount){let r=f.mount(t,d);if(typeof r==="function")m.push(r)}if(f.unmount)m.push(()=>f.unmount(t))}}return n}});var Xi=N({mount(i,u){let d=i,m=U(()=>{let b=d.inputEl();if(!b)return;b.indeterminate=d.indeterminate()}),n=U(()=>{let b=d.inputEl();if(!b)return;let w=()=>{d.indeterminate.set(!1),u.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:b.checked,value:d.value()},bubbles:!0,composed:!0}))};return b.addEventListener("change",w),()=>b.removeEventListener("change",w)});return()=>{m(),n()}}});var bl=V("sui-checkbox",{props:{checked:P.bool({default:!1,reflect:!0}),indeterminate:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),required:P.bool({default:!1,reflect:!0}),invalid:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),name:P.string({default:""}),value:P.string({default:"on"}),label:P.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.checked=u.checked,n.indeterminate=u.indeterminate,n.disabled=u.disabled,n.required=u.required,n.invalid=u.invalid,n.tone=u.tone,n.size=u.size,n.name=u.name,n.value=u.value,n.label=u.label,n.inputEl=J(null);let b=null;if(Xi&&typeof Xi.setup==="function"){if(i.scope=n,b=Xi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("label");E.setAttribute("part","label");let M=document.createElement("span");M.setAttribute("class","box"),M.setAttribute("part","box");let y=document.createElement("input");y.setAttribute("part","input"),y.setAttribute("type","checkbox"),m.push(jn(y,n.checked)),m.push(W(y,"disabled",()=>n.disabled())),m.push(W(y,"required",()=>n.required())),m.push(W(y,"name",()=>n.name())),m.push(W(y,"value",()=>n.value())),m.push(W(y,"aria-checked",()=>n.indeterminate()?"mixed":n.checked()?"true":"false")),m.push(W(y,"aria-invalid",()=>n.invalid()?"true":"false")),m.push(W(y,"aria-label",()=>n.label()||n.name()||"checkbox")),A(n,"inputEl",y),M.appendChild(y);let L=document.createElement("span");L.setAttribute("class","glyph check"),L.setAttribute("aria-hidden","true"),M.appendChild(L);let l=document.createElement("span");l.setAttribute("class","glyph mixed"),l.setAttribute("aria-hidden","true"),M.appendChild(l),E.appendChild(M);let f=document.createElement("slot");if(E.appendChild(f),w.appendChild(E),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let r=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let g=H[C];if(g!=null&&typeof g.set==="function"){if(typeof z==="function")g.set(()=>z);else g.set(z);return!0}return!1}});if(t.mount){let H=t.mount(r,d);if(typeof H==="function")m.push(H)}if(t.unmount)m.push(()=>t.unmount(r))}}return n}});var Gi=N({mount(i,u){let d=i,m=U(()=>{let n=d.inputEl();if(!n)return;let b=()=>{u.dispatchEvent(new CustomEvent("sui-change",{detail:{checked:n.checked},bubbles:!0,composed:!0}))};return n.addEventListener("change",b),()=>n.removeEventListener("change",b)});return()=>m()}});var El=V("sui-switch",{props:{checked:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),name:P.string({default:""}),value:P.string({default:"on"}),label:P.string({default:""})},shadow:!0,formAssociated:!0,formValue:"checked",formDisabled:"disabled",styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.checked=u.checked,n.disabled=u.disabled,n.tone=u.tone,n.size=u.size,n.name=u.name,n.value=u.value,n.label=u.label,n.inputEl=J(null);let b=null;if(Gi&&typeof Gi.setup==="function"){if(i.scope=n,b=Gi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("label"),M=document.createElement("span");M.setAttribute("class","track"),M.setAttribute("part","track");let y=document.createElement("input");y.setAttribute("part","input"),y.setAttribute("type","checkbox"),y.setAttribute("role","switch"),m.push(jn(y,n.checked)),m.push(W(y,"disabled",()=>n.disabled())),m.push(W(y,"name",()=>n.name())),m.push(W(y,"value",()=>n.value())),m.push(W(y,"aria-checked",()=>n.checked()?"true":"false")),m.push(W(y,"aria-label",()=>n.label()||n.name()||"toggle")),A(n,"inputEl",y),M.appendChild(y);let L=document.createElement("span");L.setAttribute("class","thumb"),L.setAttribute("part","thumb"),M.appendChild(L),E.appendChild(M);let l=document.createElement("slot");if(E.appendChild(l),w.appendChild(E),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let t=new Proxy(n,{get(r,H){return r[H]},set(r,H,C){let z=r[H];if(z!=null&&typeof z.set==="function"){if(typeof C==="function")z.set(()=>C);else z.set(C);return!0}return!1}});if(f.mount){let r=f.mount(t,d);if(typeof r==="function")m.push(r)}if(f.unmount)m.push(()=>f.unmount(t))}}return n}});var Wi=N({mount(i,u){let d=i,m=d.value(),n=U(()=>{let b=d.textareaEl();if(!b)return;let w=()=>{u.dispatchEvent(new CustomEvent("sui-input",{detail:{value:b.value},bubbles:!0,composed:!0}))},E=()=>{let M=d.value();if(M!==m)m=M,u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:M},bubbles:!0,composed:!0}))};return b.addEventListener("input",w),b.addEventListener("blur",E),()=>{b.removeEventListener("input",w),b.removeEventListener("blur",E)}});return()=>n()}});var zl=V("sui-textarea",{props:{value:P.string({default:""}),placeholder:P.string({default:""}),name:P.string({default:""}),rows:P.number({default:4}),required:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),readonly:P.bool({default:!1,reflect:!0}),invalid:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),variant:P.union(["default","code"],{default:"default",reflect:!0}),size:P.union(["sm","md"],{default:"md",reflect:!0}),resize:P.union(["none","vertical","horizontal","both"],{default:"vertical"}),label:P.string({default:""}),"described-by":P.string({default:""})},shadow:!0,formAssociated:!0,formValue:"value",formDisabled:"disabled",styles:`
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

`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.placeholder=u.placeholder,n.name=u.name,n.rows=u.rows,n.required=u.required,n.disabled=u.disabled,n.readonly=u.readonly,n.invalid=u.invalid,n.tone=u.tone,n.variant=u.variant,n.size=u.size,n.resize=u.resize,n.label=u.label,n.describedBy=u.describedBy,n.textareaEl=J(null);let b=null;if(Wi&&typeof Wi.setup==="function"){if(i.scope=n,b=Wi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("textarea");if(E.setAttribute("part","textarea"),m.push(W(E,"placeholder",()=>n.placeholder())),m.push(W(E,"name",()=>n.name())),m.push(W(E,"rows",()=>n.rows())),m.push(W(E,"disabled",()=>n.disabled())),m.push(W(E,"required",()=>n.required())),m.push(W(E,"readonly",()=>n.readonly())),m.push(W(E,"aria-invalid",()=>n.invalid()?"true":"false")),m.push(W(E,"aria-required",()=>n.required()?"true":"false")),m.push(W(E,"aria-label",()=>n.label())),m.push(W(E,"aria-describedby",()=>n.describedBy())),m.push(U(()=>{let M=n.resize();if(M==null)E.style.removeProperty("resize");else E.style.setProperty("resize",String(M))})),m.push(Pn(E,n.value)),A(n,"textareaEl",E),w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var cu=0,Qi=N({mount(i,u){let d=i,m=`sui-field-hint-${++cu}`,n=`sui-field-error-${++cu}`,b=U(()=>{let y=d.hintEl(),L=d.errorEl();if(y)y.id=m;if(L)L.id=n}),w=()=>{let L=d.controlSlot()?.assignedElements({flatten:!0})[0];if(!L)return;let l=d.invalid(),f=l?n:m,r=[...(L.getAttribute("aria-describedby")||"").split(/\s+/).filter((H)=>H&&H!==m&&H!==n),f].join(" ").trim();if(r)L.setAttribute("aria-describedby",r);if("invalid"in L)L.invalid=l;else L.setAttribute("aria-invalid",l?"true":"false")},E=U(()=>{d.invalid(),w()}),M=U(()=>{let y=d.controlSlot();if(!y)return;return y.addEventListener("slotchange",w),()=>y.removeEventListener("slotchange",w)});return()=>{b(),E(),M()}}});var Tl=V("sui-field",{props:{invalid:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.invalid=u.invalid,n.controlSlot=J(null),n.errorEl=J(null),n.hintEl=J(null);let b=null;if(Qi&&typeof Qi.setup==="function"){if(i.scope=n,b=Qi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");E.setAttribute("name","label"),w.appendChild(E);let M=document.createElement("slot");A(n,"controlSlot",M),w.appendChild(M);let y=document.createElement("div");y.setAttribute("class","hint"),A(n,"hintEl",y);let L=document.createElement("slot");L.setAttribute("name","hint"),y.appendChild(L),w.appendChild(y);let l=document.createElement("div");l.setAttribute("class","error"),l.setAttribute("role","alert"),l.setAttribute("aria-live","polite"),A(n,"errorEl",l);let f=document.createElement("slot");if(f.setAttribute("name","error"),l.appendChild(f),w.appendChild(l),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let r=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let g=H[C];if(g!=null&&typeof g.set==="function"){if(typeof z==="function")g.set(()=>z);else g.set(z);return!0}return!1}});if(t.mount){let H=t.mount(r,d);if(typeof H==="function")m.push(H)}if(t.unmount)m.push(()=>t.unmount(r))}}return n}});var Vi=N({scope:(i,u)=>{let d=i,m=e(null),n=s(()=>d.tone()==="destructive"?"destructive":"default");function b(M){u.dispatchEvent(new CustomEvent(M,{bubbles:!0,composed:!0})),d.open.set(!1)}function w(){b("sui-confirm")}function E(){b("sui-cancel")}return{dialogRef:m,confirmTone:n,onConfirm:w,onCancel:E}},mount(i,u){let d=i,m=d.open,n=d.dialogRef,b=U(()=>{let E=m(),M=n();if(!M)return;if(E&&typeof M.show==="function")M.show();else if(!E&&typeof M.close==="function")M.close()}),w=()=>{if(m())m.set(!1)};return u.addEventListener("sui-close",w),()=>{b(),u.removeEventListener("sui-close",w)}}});var Vl=V("sui-alert-dialog",{props:{open:P.bool({default:!1,reflect:!0}),tone:P.union(["default","destructive"],{default:"default",reflect:!0}),"confirm-label":P.string({default:"Confirm"}),"cancel-label":P.string({default:"Cancel"})},shadow:!0,styles:`
  :host { display: contents; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.tone=u.tone,n.confirmLabel=u.confirmLabel,n.cancelLabel=u.cancelLabel,n.dialogRef=J(null);let b=null;if(Vi&&typeof Vi.setup==="function"){if(i.scope=n,b=Vi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("sui-dialog");E.setAttribute("role-override","alertdialog"),E.setAttribute("size","sm"),E.setAttribute("no-close-button",""),A(n,"dialogRef",E);let M=document.createElement("span");M.setAttribute("slot","header");let y=document.createElement("slot");y.setAttribute("name","title"),M.appendChild(y),E.appendChild(M);let L=document.createElement("slot");E.appendChild(L);let l=document.createElement("span");l.setAttribute("slot","footer");let f=document.createElement("sui-button");f.setAttribute("variant","outline"),m.push(nn(f,"click",function(){let C=n.onCancel;return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let t=document.createTextNode("");f.appendChild(t),m.push(a(t,()=>n.cancelLabel())),l.appendChild(f);let r=document.createElement("sui-button");r.setAttribute("variant","solid"),m.push(W(r,"tone",()=>n.confirmTone())),m.push(nn(r,"click",function(){let C=n.onConfirm;return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let H=document.createTextNode("");if(r.appendChild(H),m.push(a(H,()=>n.confirmLabel())),l.appendChild(r),E.appendChild(l),w.appendChild(E),b!=null){let C=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(n,{get(g,j){return g[j]},set(g,j,T){let D=g[j];if(D!=null&&typeof D.set==="function"){if(typeof T==="function")D.set(()=>T);else D.set(T);return!0}return!1}});if(C.mount){let g=C.mount(z,d);if(typeof g==="function")m.push(g)}if(C.unmount)m.push(()=>C.unmount(z))}}return n}});function _u(i){if(i<1024)return`${i} B`;if(i<1048576)return`${(i/1024).toFixed(1)} KB`;return`${(i/1048576).toFixed(1)} MB`}var Yi=N({scope:()=>{let i=J([]),u=J(!1),d=J(""),m=e(null),n=e(null),b=e(null),w=s(()=>i().map((E,M)=>({file:E,idx:M,key:`${M}:${E.name}:${E.size}`,name:E.name,sizeStr:_u(E.size)})));return{files:i,over:u,errorMsg:d,displayFiles:w,dropzoneEl:m,inputEl:n,fileListEl:b}},mount(i,u){let d=(C)=>{let z=i.accept().split(",").map((T)=>T.trim().toLowerCase()).filter(Boolean);if(z.length===0)return!0;let g="."+(C.name.split(".").pop()||"").toLowerCase(),j=C.type.toLowerCase();return z.some((T)=>{if(T.startsWith("."))return g===T;if(T.endsWith("/*"))return j.startsWith(T.slice(0,-1));return j===T})},m=(C)=>{u.dispatchEvent(new CustomEvent("sui-change",{detail:{files:[...C]},bubbles:!0,composed:!0}))},n=(C)=>{i.errorMsg.set("");let z=[];for(let T of C){if(T.size>i.maxSize()){i.errorMsg.set(`"${T.name}" exceeds max size (${_u(i.maxSize())}).`);continue}if(i.accept()&&!d(T)){i.errorMsg.set(`"${T.name}" is not an accepted file type.`);continue}z.push(T)}let g=i.files(),j=i.multiple()?[...g,...z]:z.slice(0,1);i.files.set(j),m(j)},b=(C)=>{let z=i.files();if(C<0||C>=z.length)return;let g=z.filter((j,T)=>T!==C);i.files.set(g),i.errorMsg.set(""),m(g)},w=()=>{if(i.disabled())return;i.inputEl()?.click()},E=()=>w(),M=(C)=>{if(i.disabled())return;if(C.key==="Enter"||C.key===" ")C.preventDefault(),w()},y=(C)=>{if(i.disabled())return;C.preventDefault(),i.over.set(!0)},L=()=>i.over.set(!1),l=(C)=>{if(i.disabled())return;C.preventDefault(),i.over.set(!1);let z=C.dataTransfer?.files;if(z&&z.length)n(Array.from(z))},f=(C)=>{let z=C.target;if(z.files&&z.files.length)n(Array.from(z.files));z.value=""},t=i.dropzoneEl(),r=i.inputEl();t?.addEventListener("click",E),t?.addEventListener("keydown",M),t?.addEventListener("dragover",y),t?.addEventListener("dragleave",L),t?.addEventListener("drop",l),r?.addEventListener("change",f);let H=(C)=>{let g=C.target?.closest?.("button.remove");if(!g||!u.shadowRoot?.contains(g))return;let j=Number(g.dataset.idx);if(Number.isInteger(j))b(j)};return u.shadowRoot?.addEventListener("click",H),()=>{t?.removeEventListener("click",E),t?.removeEventListener("keydown",M),t?.removeEventListener("dragover",y),t?.removeEventListener("dragleave",L),t?.removeEventListener("drop",l),r?.removeEventListener("change",f),u.shadowRoot?.removeEventListener("click",H)}}});var Fl=V("sui-file-upload",{props:{accept:P.string({default:""}),multiple:P.bool({default:!1}),"max-size":P.number({default:10485760}),disabled:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.accept=u.accept,n.multiple=u.multiple,n.maxSize=u.maxSize,n.disabled=u.disabled,n.tone=u.tone,n.dropzoneEl=J(null),n.fileListEl=J(null),n.inputEl=J(null);let b=null;if(Yi&&typeof Yi.setup==="function"){if(i.scope=n,b=Yi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(g)=>I({anchor:g,test:()=>{if(n.errorMsg())return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),T=[],D=document.createElement("div");D.setAttribute("class","error"),D.setAttribute("role","alert"),D.setAttribute("part","error");let Z=document.createTextNode("");return D.appendChild(Z),T.push(a(Z,()=>n.errorMsg())),j.appendChild(D),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let X of T)try{X()}catch(G){}}}}]}),E=(g)=>h({anchor:g,items:()=>n.displayFiles(),key:(j,T)=>{let D=Object.create(n);return D.f=j,D.__i=T,((Z)=>j.key)(D)},render:(j,T)=>{let D=Object.create(n);return D.f=j,D.__i=T,D.__v=J(0),((Z)=>{let X=document.createDocumentFragment(),G=[],Q=document.createElement("div");Q.setAttribute("class","file-item"),Q.setAttribute("part","file-item");let q=document.createElement("span");q.setAttribute("class","file-name");let F=document.createTextNode("");q.appendChild(F),F.textContent=Z.f.name,Q.appendChild(q);let k=document.createElement("span");k.setAttribute("class","file-size");let K=document.createTextNode("");k.appendChild(K),K.textContent=Z.f.sizeStr,Q.appendChild(k);let B=document.createElement("button");B.setAttribute("class","remove"),B.setAttribute("part","remove"),B.setAttribute("type","button");{let R=Z.f.idx;if(R==null||R===!1||R==="")B.removeAttribute("data-idx");else B.setAttribute("data-idx",R===!0?"":String(R))}{let R="Remove "+Z.f.name;if(R==null||R===!1||R==="")B.removeAttribute("aria-label");else B.setAttribute("aria-label",R===!0?"":String(R))}let S=document.createTextNode("×");return B.appendChild(S),Q.appendChild(B),X.appendChild(Q),{nodes:Array.from(X.childNodes),cleanup:()=>{for(let R of G)try{R()}catch(v){}},update:(R,v)=>{Z.f=R,Z.__i=v,F.textContent=Z.f.name,K.textContent=Z.f.sizeStr;{let O=Z.f.idx;if(O==null||O===!1||O==="")B.removeAttribute("data-idx");else B.setAttribute("data-idx",O===!0?"":String(O))}{let O="Remove "+Z.f.name;if(O==null||O===!1||O==="")B.removeAttribute("aria-label");else B.setAttribute("aria-label",O===!0?"":String(O))}Z.__v.set((O)=>O+1)}}})(D)}}),M=(g)=>I({anchor:g,test:()=>{if(n.displayFiles().length>0)return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),T=[],D=document.createElement("div");D.setAttribute("class","file-list"),D.setAttribute("part","file-list"),D.setAttribute("aria-live","polite"),A(n,"fileListEl",D);let Z=document.createComment("for");return D.appendChild(Z),T.push(E(Z)),j.appendChild(D),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let X of T)try{X()}catch(G){}}}}]}),y=d.shadowRoot??d;Y(y);let L=document.createElement("div");L.setAttribute("class","dropzone"),m.push(Cn(L,"over",()=>n.over())),L.setAttribute("part","dropzone"),L.setAttribute("role","button"),m.push(W(L,"tabindex",()=>n.disabled()?"-1":"0")),m.push(W(L,"aria-label",()=>n.accept()?"Upload files ("+n.accept()+")":"Upload files")),m.push(W(L,"aria-disabled",()=>n.disabled()?"true":"false")),A(n,"dropzoneEl",L);let l=document.createElement("span");l.setAttribute("class","icon"),l.setAttribute("aria-hidden","true");let f=document.createTextNode("\uD83D\uDCC4");l.appendChild(f),L.appendChild(l);let t=document.createElement("span"),r=document.createTextNode("Drop files here or click to browse");t.appendChild(r),L.appendChild(t),y.appendChild(L);let H=document.createElement("input");H.setAttribute("type","file"),H.setAttribute("part","input"),m.push(W(H,"accept",()=>n.accept())),m.push(W(H,"multiple",()=>n.multiple())),H.setAttribute("aria-hidden","true"),H.setAttribute("tabindex","-1"),A(n,"inputEl",H),y.appendChild(H);let C=document.createComment("if");y.appendChild(C),m.push(w(C));let z=document.createComment("if");if(y.appendChild(z),m.push(M(z)),b!=null){let g=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(g){let j=new Proxy(n,{get(T,D){return T[D]},set(T,D,Z){let X=T[D];if(X!=null&&typeof X.set==="function"){if(typeof Z==="function")X.set(()=>Z);else X.set(Z);return!0}return!1}});if(g.mount){let T=g.mount(j,d);if(typeof T==="function")m.push(T)}if(g.unmount)m.push(()=>g.unmount(j))}}return n}});var zm=new Set(["info","success","warning","destructive","accent"]),qi=N({scope:()=>{let i=J(!1),u=J(!0),d=e(null),m=e(null),n=e(null),b=e(null);return{hasSlottedIcon:i,visible:u,iconSlotEl:d,titleSlotEl:m,actionsSlotEl:n,closeBtn:b}},mount(i,u){let d=i,m=null,n=0,b=0,w=!1;if(!u.hasAttribute("tabindex"))u.tabIndex=0;function E(){if(m)clearTimeout(m),m=null}function M(){E();let Z=d.duration();if(Z>0)n=Z,b=Date.now(),m=setTimeout(()=>y("timeout"),Z),u.style.setProperty("--toast-duration",`${Z}ms`)}function y(Z="manual"){if(w)return;w=!0,E(),u.dispatchEvent(new CustomEvent("sui-dismiss",{detail:{reason:Z},bubbles:!0,composed:!0})),u.remove()}async function L(Z="manual"){if(w)return;if(typeof window<"u"&&window.matchMedia?.("(prefers-reduced-motion: reduce)").matches){y(Z);return}E(),u.setAttribute("closing",""),d.visible.set(!1),await new Promise((Q)=>setTimeout(Q,200)),y(Z)}let l=()=>{if(!m)return;let Z=Date.now()-b;n=Math.max(0,n-Z),E(),u.toggleAttribute("data-paused",!0)},f=()=>{if(w||d.duration()<=0)return;if(n<=0)return;u.toggleAttribute("data-paused",!1),b=Date.now(),m=setTimeout(()=>y("timeout"),n)},t=(Z)=>{if(Z.key==="Escape"&&d.dismissible())Z.stopPropagation(),y("manual")};u.addEventListener("mouseenter",l),u.addEventListener("mouseleave",f),u.addEventListener("focusin",l),u.addEventListener("focusout",f),u.addEventListener("keydown",t),M();let r=d.hasSlottedIcon,H=(Z)=>Z.assignedNodes({flatten:!0}).some((X)=>{if(X.nodeType===Node.ELEMENT_NODE)return!0;return(X.textContent?.trim().length??0)>0}),C=d.iconSlotEl();if(C){let Z=()=>r.set(H(C));C.addEventListener("slotchange",Z),r.set(H(C))}let z=d.titleSlotEl(),g=d.actionsSlotEl();if(z){let Z=()=>u.toggleAttribute("data-no-title",!H(z));z.addEventListener("slotchange",Z),u.toggleAttribute("data-no-title",!H(z))}if(g){let Z=()=>u.toggleAttribute("data-no-actions",!H(g));g.addEventListener("slotchange",Z),u.toggleAttribute("data-no-actions",!H(g))}let j=U(()=>{let Z=d.tone(),X=d.hasSlottedIcon(),G=zm.has(Z);u.toggleAttribute("data-no-icon",!G&&!X)}),T=U(()=>{let Z=d.closeBtn();if(!Z)return;let X=()=>y("manual");return Z.addEventListener("click",X),()=>Z.removeEventListener("click",X)}),D=u;return D.dismiss=y,D.close=L,()=>{w=!0,E(),u.removeEventListener("mouseenter",l),u.removeEventListener("mouseleave",f),u.removeEventListener("focusin",l),u.removeEventListener("focusout",f),u.removeEventListener("keydown",t),j(),T()}}});var kl=V("sui-toast",{props:{tone:P.union(["default","info","success","warning","destructive","accent","muted","primary"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),duration:P.number({default:5000}),dismissible:P.bool({default:!0,reflect:!0}),"show-progress":P.bool({default:!1,reflect:!0}),closing:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.size=u.size,n.duration=u.duration,n.dismissible=u.dismissible,n.showProgress=u.showProgress,n.closing=u.closing,n.actionsSlotEl=J(null),n.closeBtn=J(null),n.iconSlotEl=J(null),n.titleSlotEl=J(null);let b=null;if(qi&&typeof qi.setup==="function"){if(i.scope=n,b=qi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(t)=>I({anchor:t,test:()=>{let r=n.tone();if(r==="info")return 0;if(r==="success")return 1;if(r==="warning")return 2;if(r==="destructive")return 3;if(r==="accent")return 4;return-1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true");let z=document.createElement("circle");z.setAttribute("cx","12"),z.setAttribute("cy","12"),z.setAttribute("r","9"),C.appendChild(z);let g=document.createElement("path");g.setAttribute("d","M12 8h.01"),C.appendChild(g);let j=document.createElement("path");return j.setAttribute("d","M11 12h1v4h1"),C.appendChild(j),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let T of H)try{T()}catch(D){}}}},()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true");let z=document.createElement("circle");z.setAttribute("cx","12"),z.setAttribute("cy","12"),z.setAttribute("r","9"),C.appendChild(z);let g=document.createElement("path");return g.setAttribute("d","m8.5 12.5 2.5 2.5 4.5-5"),C.appendChild(g),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let j of H)try{j()}catch(T){}}}},()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true");let z=document.createElement("path");z.setAttribute("d","M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"),C.appendChild(z);let g=document.createElement("line");g.setAttribute("x1","12"),g.setAttribute("x2","12"),g.setAttribute("y1","9"),g.setAttribute("y2","13"),C.appendChild(g);let j=document.createElement("line");return j.setAttribute("x1","12"),j.setAttribute("x2","12.01"),j.setAttribute("y1","17"),j.setAttribute("y2","17"),C.appendChild(j),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let T of H)try{T()}catch(D){}}}},()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true");let z=document.createElement("circle");z.setAttribute("cx","12"),z.setAttribute("cy","12"),z.setAttribute("r","9"),C.appendChild(z);let g=document.createElement("line");g.setAttribute("x1","15"),g.setAttribute("x2","9"),g.setAttribute("y1","9"),g.setAttribute("y2","15"),C.appendChild(g);let j=document.createElement("line");return j.setAttribute("x1","9"),j.setAttribute("x2","15"),j.setAttribute("y1","9"),j.setAttribute("y2","15"),C.appendChild(j),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let T of H)try{T()}catch(D){}}}},()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("svg");C.setAttribute("viewbox","0 0 24 24"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","2"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true");let z=document.createElement("path");return z.setAttribute("d","M12 2 3 14h7l-1 8 10-12h-7l1-8Z"),C.appendChild(z),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let g of H)try{g()}catch(j){}}}}]}),E=(t)=>I({anchor:t,test:()=>{if(!n.hasSlottedIcon())return 0;return-1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createComment("match");return r.appendChild(C),H.push(w(C)),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let z of H)try{z()}catch(g){}}}}]}),M=(t)=>I({anchor:t,test:()=>{if(n.dismissible())return 0;return-1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("button");C.setAttribute("class","close"),C.setAttribute("part","close"),C.setAttribute("type","button"),C.setAttribute("aria-label","Dismiss notification"),A(n,"closeBtn",C);let z=document.createElement("svg");z.setAttribute("viewbox","0 0 24 24"),z.setAttribute("fill","none"),z.setAttribute("stroke","currentColor"),z.setAttribute("stroke-width","2"),z.setAttribute("stroke-linecap","round"),z.setAttribute("stroke-linejoin","round"),z.setAttribute("aria-hidden","true");let g=document.createElement("line");g.setAttribute("x1","18"),g.setAttribute("y1","6"),g.setAttribute("x2","6"),g.setAttribute("y2","18"),z.appendChild(g);let j=document.createElement("line");return j.setAttribute("x1","6"),j.setAttribute("y1","6"),j.setAttribute("x2","18"),j.setAttribute("y2","18"),z.appendChild(j),C.appendChild(z),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let T of H)try{T()}catch(D){}}}}]}),y=(t)=>I({anchor:t,test:()=>{if(n.showProgress()&&n.duration()>0)return 0;return-1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("span");return C.setAttribute("class","progress"),C.setAttribute("aria-hidden","true"),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let z of H)try{z()}catch(g){}}}}]}),L=(t)=>I({anchor:t,test:()=>{if(n.visible())return 0;return-1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("div");C.setAttribute("class","toast"),Ln(C,du,{axis:"y",duration:200},"in"),Ln(C,Dn,{duration:180},"out"),H.push(W(C,"role",()=>n.tone()==="destructive"||n.tone()==="warning"?"alert":"status")),H.push(W(C,"aria-live",()=>n.tone()==="destructive"||n.tone()==="warning"?"assertive":"polite")),C.setAttribute("aria-atomic","true"),C.setAttribute("part","toast");let z=document.createElement("span");z.setAttribute("class","icon");let g=document.createElement("slot");g.setAttribute("name","icon"),A(n,"iconSlotEl",g),z.appendChild(g);let j=document.createComment("if");z.appendChild(j),H.push(E(j)),C.appendChild(z);let T=document.createElement("div");T.setAttribute("class","content");let D=document.createElement("div");D.setAttribute("class","title");let Z=document.createElement("slot");Z.setAttribute("name","title"),A(n,"titleSlotEl",Z),D.appendChild(Z),T.appendChild(D);let X=document.createElement("div");X.setAttribute("class","body");let G=document.createElement("slot");X.appendChild(G),T.appendChild(X);let Q=document.createElement("div");Q.setAttribute("class","actions");let q=document.createElement("slot");q.setAttribute("name","actions"),A(n,"actionsSlotEl",q),Q.appendChild(q),T.appendChild(Q),C.appendChild(T);let F=document.createComment("if");C.appendChild(F),H.push(M(F));let k=document.createComment("if");return C.appendChild(k),H.push(y(k)),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let K of H)try{K()}catch(B){}}}}]}),l=d.shadowRoot??d;Y(l);let f=document.createComment("if");if(l.appendChild(f),m.push(L(f)),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let r=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let g=H[C];if(g!=null&&typeof g.set==="function"){if(typeof z==="function")g.set(()=>z);else g.set(z);return!0}return!1}});if(t.mount){let H=t.mount(r,d);if(typeof H==="function")m.push(H)}if(t.unmount)m.push(()=>t.unmount(r))}}return n}});var gm={"top-left":"top","top-center":"top","top-right":"top","bottom-left":"bottom","bottom-center":"bottom","bottom-right":"bottom"},En="16px",Nn="position:fixed;display:flex;gap:var(--space-2,8px);pointer-events:none;z-index:var(--z-toast,9999);max-width:min(440px,calc(100vw - 32px));padding:0;margin:0;box-sizing:border-box;";function Pm(i){switch(i){case"top-right":return`${Nn}top:${En};right:${En};flex-direction:column;align-items:flex-end;`;case"top-left":return`${Nn}top:${En};left:${En};flex-direction:column;align-items:flex-start;`;case"top-center":return`${Nn}top:${En};left:50%;transform:translateX(-50%);flex-direction:column;align-items:center;`;case"bottom-left":return`${Nn}bottom:${En};left:${En};flex-direction:column-reverse;align-items:flex-start;`;case"bottom-center":return`${Nn}bottom:${En};left:50%;transform:translateX(-50%);flex-direction:column-reverse;align-items:center;`;case"bottom-right":default:return`${Nn}bottom:${En};right:${En};flex-direction:column-reverse;align-items:flex-end;`}}var Ui=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","region");if(!u.hasAttribute("aria-label"))u.setAttribute("aria-label","Notifications");if(!u.hasAttribute("aria-live"))u.setAttribute("aria-live","polite");let m=J(0),n=d.defaultSlotEl(),b=d.stackEl(),w=(r)=>{if(!b)return;if(r.parentElement!==b)b.appendChild(r)},E=()=>{if(!b)return[];return Array.from(b.children).filter((r)=>r.tagName.toLowerCase()==="sui-toast")},M=U(()=>{if(!b)return;let r=d.position();b.style.cssText=Pm(r),b.setAttribute("data-position",r)}),y=U(()=>{m();let r=d.position(),H=d.maxVisible(),C=gm[r]??"bottom",z=E();for(let g of z)g.setAttribute("data-slide-from",C),g.style.pointerEvents="auto";if(H>0&&z.length>H){let g=z.length-H;for(let j=0;j<g;j++){let T=z[j];if(typeof T.close==="function")T.close("programmatic");else T.dismiss?.("programmatic")}}}),L=()=>{if(n){for(let r of n.assignedElements())if(r.tagName.toLowerCase()==="sui-toast")w(r)}m.set(m()+1)};if(n)n.addEventListener("slotchange",L),L();let l=new MutationObserver((r)=>{let H=!1;for(let C of r){for(let z of C.addedNodes)if(z.nodeType===Node.ELEMENT_NODE){let g=z;if(g.tagName.toLowerCase()==="sui-toast")w(g),H=!0}if(C.removedNodes.length)H=!0}if(H)m.set(m()+1)});l.observe(u,{childList:!0});for(let r of Array.from(u.children))if(r.tagName.toLowerCase()==="sui-toast")w(r);m.set(m()+1);let f=b?new MutationObserver(()=>m.set(m()+1)):null;if(b&&f)f.observe(b,{childList:!0});function t(r={}){let H=u.ownerDocument??document,C=H.createElement("sui-toast");if(r.tone)C.setAttribute("tone",r.tone);if(r.size)C.setAttribute("size",r.size);if(typeof r.duration==="number")C.setAttribute("duration",String(r.duration));if(r.dismissible===!1)C.removeAttribute("dismissible");else C.setAttribute("dismissible","");if(r.showProgress)C.setAttribute("show-progress","");if(r.title){let z=H.createElement("span");z.setAttribute("slot","title"),z.textContent=r.title,C.appendChild(z)}if(r.description)C.appendChild(H.createTextNode(r.description));if(r.actionHtml){let z=H.createElement("span");z.setAttribute("slot","actions");let g=r.actionHtml;mn(z,()=>g),C.appendChild(z)}if(b)b.appendChild(C);else u.appendChild(C);return C}return u.show=t,()=>{if(y(),M(),l.disconnect(),f?.disconnect(),n)n.removeEventListener("slotchange",L)}}});var $l=V("sui-toast-container",{props:{position:P.union(["top-left","top-center","top-right","bottom-left","bottom-center","bottom-right"],{default:"bottom-right",reflect:!0}),"max-visible":P.number({default:5,reflect:!0})},shadow:!0,styles:`
  /* Host collapses — the visible stack lives in body via <sui-portal>
     (see template). The host stays minimal so it can sit anywhere in
     the document without taking layout space. */
  :host { display: contents; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.position=u.position,n.maxVisible=u.maxVisible,n.defaultSlotEl=J(null),n.stackEl=J(null);let b=null;if(Ui&&typeof Ui.setup==="function"){if(i.scope=n,b=Ui.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("sui-portal");E.setAttribute("target","body");let M=document.createElement("div");M.setAttribute("class","sui-toast-stack"),M.setAttribute("data-sui-toast-stack",""),A(n,"stackEl",M),E.appendChild(M),w.appendChild(E);let y=document.createElement("slot");if(A(n,"defaultSlotEl",y),w.appendChild(y),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var Ni=N({scope:()=>{let i=J([]),u=J("");return{items:i,lastSlot:u}},mount(i,u){let d=()=>{let b=Array.from(u.querySelectorAll(":scope > sui-breadcrumb-item")).map((w,E)=>{let M=`i${E}`;return w.setAttribute("slot",M),{slot:M}});i.lastSlot.set(b.length>0?b[b.length-1].slot:""),i.items.set(b)};d();let m=new MutationObserver(d);return m.observe(u,{childList:!0}),()=>m.disconnect()}});var _l=V("sui-breadcrumb",{props:{separator:P.string({default:"/",reflect:!0})},shadow:!0,styles:`
  :host { display: flex; align-items: center; }
  nav { display: inline-flex; align-items: center; flex-wrap: wrap; gap: 2px; }
  .sep {
    display: inline-flex; align-items: center; justify-content: center;
    width: 12px; height: 12px;
    color: var(--muted-foreground);
    opacity: 0.7;
  }
  .sep svg { width: 12px; height: 12px; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.separator=u.separator;let b=null;if(Ni&&typeof Ni.setup==="function"){if(i.scope=n,b=Ni.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>h({anchor:L,items:()=>n.items(),key:(l,f)=>{let t=Object.create(n);return t.item=l,t.__i=f,((r)=>l.slot)(t)},render:(l,f)=>{let t=Object.create(n);return t.item=l,t.__i=f,t.__v=J(0),((r)=>{let H=(D)=>I({anchor:D,test:()=>{if(r.separator()==="/")return 0;return 1},branches:[()=>{let Z=document.createDocumentFragment(),X=[],G=document.createElement("svg");G.setAttribute("viewbox","0 0 24 24"),G.setAttribute("fill","none"),G.setAttribute("stroke","currentColor"),G.setAttribute("stroke-width","2"),G.setAttribute("stroke-linecap","round"),G.setAttribute("stroke-linejoin","round");let Q=document.createElement("path");return Q.setAttribute("d","M9 6l6 6-6 6"),G.appendChild(Q),Z.appendChild(G),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let q of X)try{q()}catch(F){}}}},()=>{let Z=document.createDocumentFragment(),X=[],G=document.createTextNode("");return Z.appendChild(G),X.push(a(G,()=>(r.__v(),r.separator()))),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let Q of X)try{Q()}catch(q){}}}}]}),C=(D)=>I({anchor:D,test:()=>{if(r.item.slot!==r.lastSlot())return 0;return-1},branches:[()=>{let Z=document.createDocumentFragment(),X=[],G=document.createElement("span");G.setAttribute("class","sep"),G.setAttribute("aria-hidden","true");let Q=document.createComment("if");return G.appendChild(Q),X.push(H(Q)),Z.appendChild(G),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let q of X)try{q()}catch(F){}}}}]}),z=document.createDocumentFragment(),g=[],j=document.createElement("slot");{let D=r.item.slot;if(D==null||D===!1||D==="")j.removeAttribute("name");else j.setAttribute("name",D===!0?"":String(D))}z.appendChild(j);let T=document.createComment("if");return z.appendChild(T),g.push(C(T)),{nodes:Array.from(z.childNodes),cleanup:()=>{for(let D of g)try{D()}catch(Z){}},update:(D,Z)=>{r.item=D,r.__i=Z;{let X=r.item.slot;if(X==null||X===!1||X==="")j.removeAttribute("name");else j.setAttribute("name",X===!0?"":String(X))}r.__v.set((X)=>X+1)}}})(t)}}),E=d.shadowRoot??d;Y(E);let M=document.createElement("nav");M.setAttribute("aria-label","Breadcrumb"),M.setAttribute("part","nav");let y=document.createComment("for");if(M.appendChild(y),m.push(w(y)),E.appendChild(M),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var u5=V("sui-breadcrumb-item",{props:{href:P.string({default:"",reflect:!0}),active:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.href=u.href,n.active=u.active;let b=(M)=>I({anchor:M,test:()=>{if(n.href()&&!n.active())return 0;return 1},branches:[()=>{let y=document.createDocumentFragment(),L=[],l=document.createElement("a");L.push(W(l,"href",()=>n.href())),l.setAttribute("part","link");let f=document.createElement("slot");return l.appendChild(f),y.appendChild(l),{nodes:Array.from(y.childNodes),cleanup:()=>{for(let t of L)try{t()}catch(r){}}}},()=>{let y=document.createDocumentFragment(),L=[],l=document.createElement("span");l.setAttribute("class","label"),l.setAttribute("part","label"),L.push(W(l,"aria-current",()=>n.active()?"page":"false"));let f=document.createElement("slot");return l.appendChild(f),y.appendChild(l),{nodes:Array.from(y.childNodes),cleanup:()=>{for(let t of L)try{t()}catch(r){}}}}]}),w=d.shadowRoot??d;Y(w);let E=document.createComment("if");return w.appendChild(E),m.push(b(E)),n}});var Ai=N({scope:()=>({totalPages:J(1),info:J(""),prevBtn:e(null),nextBtn:e(null)}),mount(i,u){let d=i,m=U(()=>{let M=d.page(),y=d.total(),L=d.perPage(),l=y>0?Math.ceil(y/L):1;if(d.totalPages.set(l),y>0){let f=(M-1)*L+1,t=Math.min(M*L,y);d.info.set(`Page ${M} of ${l} (${f}–${t} of ${y})`)}else d.info.set(`Page ${M} of ${l}`)}),n=d.prevBtn(),b=d.nextBtn(),w=()=>{let M=d.page();if(M>1){let y=M-1;d.page.set(y),u.dispatchEvent(new CustomEvent("sui-page",{detail:{page:y},bubbles:!0,composed:!0}))}},E=()=>{let M=d.page(),y=d.totalPages();if(M<y){let L=M+1;d.page.set(L),u.dispatchEvent(new CustomEvent("sui-page",{detail:{page:L},bubbles:!0,composed:!0}))}};return n?.addEventListener("click",w),b?.addEventListener("click",E),()=>{m(),n?.removeEventListener("click",w),b?.removeEventListener("click",E)}}});var r5=V("sui-pagination",{props:{page:P.number({default:1,reflect:!0}),total:P.number({default:0,reflect:!0}),"per-page":P.number({default:25,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.page=u.page,n.total=u.total,n.perPage=u.perPage,n.nextBtn=J(null),n.prevBtn=J(null);let b=null;if(Ai&&typeof Ai.setup==="function"){if(i.scope=n,b=Ai.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("nav");E.setAttribute("class","bar"),E.setAttribute("aria-label","Pagination");let M=document.createElement("button");A(n,"prevBtn",M),M.setAttribute("class","pg-btn"),M.setAttribute("part","prev"),m.push(W(M,"disabled",()=>n.page()<=1)),M.setAttribute("aria-label","Previous page"),m.push(W(M,"aria-disabled",()=>n.page()<=1?"true":"false"));let y=document.createTextNode("Prev");M.appendChild(y),E.appendChild(M);let L=document.createElement("span");L.setAttribute("class","info"),L.setAttribute("part","info");let l=document.createTextNode("");L.appendChild(l),m.push(a(l,()=>n.info())),E.appendChild(L);let f=document.createElement("span");f.setAttribute("class","spacer"),E.appendChild(f);let t=document.createElement("button");A(n,"nextBtn",t),t.setAttribute("class","pg-btn"),t.setAttribute("part","next"),m.push(W(t,"disabled",()=>n.page()>=n.totalPages())),t.setAttribute("aria-label","Next page"),m.push(W(t,"aria-disabled",()=>n.page()>=n.totalPages()?"true":"false"));let r=document.createTextNode("Next");if(t.appendChild(r),E.appendChild(t),w.appendChild(E),b!=null){let H=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(H){let C=new Proxy(n,{get(z,g){return z[g]},set(z,g,j){let T=z[g];if(T!=null&&typeof T.set==="function"){if(typeof j==="function")T.set(()=>j);else T.set(j);return!0}return!1}});if(H.mount){let z=H.mount(C,d);if(typeof z==="function")m.push(z)}if(H.unmount)m.push(()=>H.unmount(C))}}return n}});function nd(i,u=Date.now()){let d=Date.parse(i);if(Number.isNaN(d))return i;let m=u-d,n=Math.abs(m),b=m>=0?"ago":"from now";if(n<60000)return"just now";if(n<3600000)return`${Math.floor(n/60000)}m ${b}`;if(n<86400000)return`${Math.floor(n/3600000)}h ${b}`;let w=Math.floor(n/86400000);if(w<30)return`${w}d ${b}`;if(w<365)return`${Math.floor(w/30)}mo ${b}`;return`${Math.floor(w/365)}y ${b}`}function id(i){let u=Date.parse(i);return Number.isNaN(u)?i:new Date(u).toISOString()}var jm=60000,Fi=N({scope:({value:i})=>{let u=J(0),d=s(()=>{let n=i();return n?id(n):""}),m=s(()=>{u();let n=i();return n?nd(n):""});return{tick:u,absStr:d,relStr:m}},mount(i,u){let d=U(()=>{if(i.value())u.title=i.absStr()}),m=setInterval(()=>i.tick.set(i.tick()+1),jm);return()=>{d(),clearInterval(m)}}});var C5=V("sui-timestamp",{props:{value:P.string({default:"",reflect:!0}),format:P.union(["relative","absolute","both"],{default:"relative",reflect:!0})},shadow:!0,styles:`
  :host {
    display: inline;
    font-family: var(--font-body);
    font-size: inherit;
    color: inherit;
  }
  .abs { color: var(--muted-foreground); margin-left: 4px; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.format=u.format;let b=null;if(Fi&&typeof Fi.setup==="function"){if(i.scope=n,b=Fi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>I({anchor:l,test:()=>{if(n.format()==="absolute")return 0;return 1},branches:[()=>{let f=document.createDocumentFragment(),t=[],r=document.createTextNode("");return f.appendChild(r),t.push(a(r,()=>n.absStr())),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let H of t)try{H()}catch(C){}}}},()=>{let f=document.createDocumentFragment(),t=[],r=document.createTextNode("");return f.appendChild(r),t.push(a(r,()=>n.relStr())),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let H of t)try{H()}catch(C){}}}}]}),E=(l)=>I({anchor:l,test:()=>{if(n.format()==="both")return 0;return-1},branches:[()=>{let f=document.createDocumentFragment(),t=[],r=document.createElement("span");r.setAttribute("class","abs"),r.setAttribute("part","abs");let H=document.createTextNode("(");r.appendChild(H);let C=document.createTextNode("");r.appendChild(C),t.push(a(C,()=>n.absStr()));let z=document.createTextNode(")");return r.appendChild(z),f.appendChild(r),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let g of t)try{g()}catch(j){}}}}]}),M=(l)=>I({anchor:l,test:()=>{if(n.value())return 0;return-1},branches:[()=>{let f=document.createDocumentFragment(),t=[],r=document.createElement("time");t.push(W(r,"datetime",()=>n.value())),r.setAttribute("part","time");let H=document.createComment("if");r.appendChild(H),t.push(w(H)),f.appendChild(r);let C=document.createComment("if");return f.appendChild(C),t.push(E(C)),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let z of t)try{z()}catch(g){}}}}]}),y=d.shadowRoot??d;Y(y);let L=document.createComment("if");if(y.appendChild(L),m.push(M(L)),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let f=new Proxy(n,{get(t,r){return t[r]},set(t,r,H){let C=t[r];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(l.mount){let t=l.mount(f,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(f))}}return n}});var P5=V("sui-metachip",{props:{label:P.string({default:"",reflect:!0}),value:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.label=u.label,n.value=u.value;let b=d.shadowRoot??d;Y(b);let w=document.createElement("span");w.setAttribute("class","k"),w.setAttribute("part","label");let E=document.createElement("slot");E.setAttribute("name","label");let M=document.createTextNode("");E.appendChild(M),m.push(a(M,()=>n.label())),w.appendChild(E),b.appendChild(w);let y=document.createElement("span");y.setAttribute("class","v"),y.setAttribute("part","value");let L=document.createElement("slot"),l=document.createTextNode("");return L.appendChild(l),m.push(a(l,()=>n.value())),y.appendChild(L),b.appendChild(y),n}});var Dm=0,ud=typeof CSS<"u"&&typeof CSS.supports==="function"&&CSS.supports("anchor-name","--x"),Oi=N({mount(i,u){let d=i,m=++Dm,n=`--sui-popover-${m}`,b=`sui-popover-panel-${m}`;u.style.setProperty("--sui-popover-anchor",n);let w=d.triggerSlot(),E=d.panelEl();if(!E)return;E.id=b,nu(d.triggerSlot,(f)=>{for(let t of f)t.setAttribute("popovertarget",b),t.setAttribute("aria-haspopup","dialog"),t.setAttribute("aria-controls",b),t.setAttribute("aria-expanded",d.open()?"true":"false")});function M(f){let r=f.newState==="open",H=u.hasAttribute("open");if(r&&!H)u.setAttribute("open","");else if(!r&&H)u.removeAttribute("open");let z=w?.assignedElements({flatten:!0})??[];for(let g of z)g.setAttribute("aria-expanded",r?"true":"false");if(u.emit("sui-toggle",{open:r}),r&&!ud)L()}c(E,"toggle",M),u.expose({show:()=>E.showPopover?.(),hide:()=>E.hidePopover?.(),toggle:()=>typeof E.togglePopover==="function"?void E.togglePopover():E.matches?.(":popover-open")?E.hidePopover?.():E.showPopover?.()});let y=U(()=>{let f=d.open(),t=E.matches?.(":popover-open")??!1;if(f&&!t)E.showPopover?.();else if(!f&&t)E.hidePopover?.()});function L(){let r=(w?.assignedElements({flatten:!0})??[])[0];if(!r||!E)return;let H=r.getBoundingClientRect(),C=d.placement();E.style.top="",E.style.left="",E.style.right="",E.style.bottom="";let z=6;switch(C){case"top":E.style.bottom=`${window.innerHeight-H.top+z}px`,E.style.left=`${H.left}px`;break;case"top-end":E.style.bottom=`${window.innerHeight-H.top+z}px`,E.style.right=`${window.innerWidth-H.right}px`;break;case"bottom-end":E.style.top=`${H.bottom+z}px`,E.style.right=`${window.innerWidth-H.right}px`;break;case"left":E.style.right=`${window.innerWidth-H.left+z}px`,E.style.top=`${H.top}px`;break;case"left-end":E.style.right=`${window.innerWidth-H.left+z}px`,E.style.bottom=`${window.innerHeight-H.bottom}px`;break;case"right":E.style.left=`${H.right+z}px`,E.style.top=`${H.top}px`;break;case"right-end":E.style.left=`${H.right+z}px`,E.style.bottom=`${window.innerHeight-H.bottom}px`;break;case"bottom":default:E.style.top=`${H.bottom+z}px`,E.style.left=`${H.left}px`}}function l(){if(!d.open())return;L()}if(!ud)c(window,"resize",l),c(window,"scroll",l,{capture:!0,passive:!0});return()=>{y()}}});var X5=V("sui-popover",{props:{open:P.bool({default:!1,reflect:!0}),placement:P.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.placement=u.placement,n.panelEl=J(null),n.triggerSlot=J(null);let b=null;if(Oi&&typeof Oi.setup==="function"){if(i.scope=n,b=Oi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(t)=>I({anchor:t,test:()=>{if(n.open())return 0;return-1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("div");C.setAttribute("class","surface"),Ln(C,mu,{duration:120},"both");let z=document.createElement("slot");return C.appendChild(z),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let g of H)try{g()}catch(j){}}}}]}),E=(t)=>I({anchor:t,test:()=>{if(n.open())return 0;return-1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("sui-portal");C.setAttribute("target","body");let z=document.createElement("div");return z.setAttribute("class","sui-popover-presence"),z.setAttribute("aria-hidden","true"),z.setAttribute("data-sui-popover-open",""),C.appendChild(z),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let g of H)try{g()}catch(j){}}}}]}),M=d.shadowRoot??d;Y(M);let y=document.createElement("slot");y.setAttribute("name","trigger"),A(n,"triggerSlot",y),M.appendChild(y);let L=document.createElement("div");L.setAttribute("class","panel"),L.setAttribute("part","panel"),L.setAttribute("popover","auto"),A(n,"panelEl",L),L.setAttribute("role","dialog"),L.setAttribute("aria-modal","false");let l=document.createComment("if");L.appendChild(l),m.push(w(l)),M.appendChild(L);let f=document.createComment("if");if(M.appendChild(f),m.push(E(f)),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let r=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let g=H[C];if(g!=null&&typeof g.set==="function"){if(typeof z==="function")g.set(()=>z);else g.set(z);return!0}return!1}});if(t.mount){let H=t.mount(r,d);if(typeof H==="function")m.push(H)}if(t.unmount)m.push(()=>t.unmount(r))}}return n}});var Hu=[{id:"ship-light",label:"SHIP Light",dark:!1,swatches:["oklch(0.99 0.001 106.423)","oklch(1 0 0)","oklch(0.67 0.16 58)","oklch(0.61 0.23 303)"]},{id:"ship-dark",label:"SHIP Dark",dark:!0,swatches:["oklch(0.18 0.01 250)","oklch(0.18 0.01 50)","oklch(0.77 0.16 70)","oklch(0.3 0.05 300)"]},{id:"paper",label:"Paper",dark:!1,swatches:["oklch(0.97 0.014 80)","oklch(0.99 0.010 80)","oklch(0.55 0.14 35)","oklch(0.48 0.09 55)"]},{id:"halo",label:"Halo (Soft)",dark:!1,swatches:["oklch(0.92 0.008 230)","oklch(0.92 0.008 230)","oklch(0.62 0.14 50)","oklch(0.55 0.05 230)"]},{id:"console",label:"Console",dark:!0,swatches:["oklch(0.13 0.008 220)","oklch(0.16 0.010 220)","oklch(0.78 0.20 145)","oklch(0.78 0.16 195)"]},{id:"contrast",label:"High Contrast",dark:!0,swatches:["oklch(0.09 0 0)","oklch(0.15 0 0)","oklch(0.87 0.18 85)","oklch(0.99 0 0)"]},{id:"glass",label:"Glass",dark:!1,swatches:["oklch(0.95 0.04 230)","oklch(0.99 0.005 240)","oklch(0.62 0.16 50)","oklch(0.62 0.14 280)"]}],dd=Object.fromEntries(Hu.map((i)=>[i.id,i]));var Ki=(i)=>typeof i==="string"&&(i in dd),Zm=(i)=>dd[i]?.dark??!1;function Cu(i,u=document.documentElement){let d=Ki(i)?i:"ship-light",m=Zm(d);if(u.dataset.theme=d,u.classList.toggle("dark",m),typeof document<"u"&&typeof CustomEvent<"u")document.dispatchEvent(new CustomEvent("ship:theme:change",{detail:{id:d,dark:m}}));return d}function zu(){if(typeof localStorage>"u")return"ship-light";try{let i=localStorage.getItem("ship-site-theme");return Ki(i)?i:"ship-light"}catch{return"ship-light"}}function md(i){if(typeof localStorage>"u")return;try{localStorage.setItem("ship-site-theme",i)}catch{}try{fetch("/api/settings",{method:"PUT",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({key:"shell.theme",value:i})}).catch(()=>{})}catch{}}var Tm=[{label:"Default",ids:["ship-light","ship-dark"]},{label:"Texture",ids:["paper","halo"]},{label:"Effects",ids:["glass"]},{label:"Special",ids:["console","contrast"]}],Bi=new Map(Hu.map((i)=>[i.id,i]));function Jm(i){let u=i.shadowRoot;if(!u)return;if(u.querySelector("link[data-sui-theme-preview-styles]"))return;let d=document.createElement("link");d.rel="stylesheet",d.href="/api/assets/view-tokens.css",d.setAttribute("data-sui-theme-preview-styles",""),u.appendChild(d)}var Ii=N({scope:(i)=>{let u=i,d=J(zu()),m=J(!1),n=J(Tm.map((f)=>({label:f.label,themes:f.ids.map((t)=>Bi.get(t)).filter((t)=>!!t)}))),b=s(()=>Bi.get(d())?.swatches[2]??"var(--primary, #3b82f6)"),w=s(()=>Bi.get(d())?.label??"Theme"),E={host:null,panel:null};function M(){let f=E.panel;if(!f)return;if(m())f.hidePopover?.();else f.showPopover?.()}function y(f){let t=Cu(f);if(u.persist())md(t);d.set(t);let r=E.host;if(r)r.dispatchEvent(new CustomEvent("sui-theme-change",{detail:{id:t,dark:Bi.get(t)?.dark??!1},bubbles:!0,composed:!0}));E.panel?.hidePopover?.()}let L=e(null),l=e(null);return{activeTheme:d,panelOpen:m,groups:n,dotColor:b,triggerLabel:w,togglePanel:M,selectTheme:y,triggerBtn:L,panelEl:l,_internals:E}},mount(i,u){let d=i;d._internals.host=u,Jm(u),Cu(zu());let m=U(()=>{let y=d.panelEl();if(d._internals.panel=y,!y)return;let L=(l)=>{let t=l.newState==="open";if(d.panelOpen()!==t)d.panelOpen.set(t);if(t)n()};return y.addEventListener("toggle",L),()=>y.removeEventListener("toggle",L)});function n(){let y=d.panelEl(),L=d.triggerBtn();if(!y||!L)return;let l=L.getBoundingClientRect();y.style.top=`${l.bottom+4}px`,y.style.left=`${Math.max(8,l.right-y.offsetWidth)}px`}function b(y){let L=y.detail?.id;if(!Ki(L))return;if(d.activeTheme()!==L)d.activeTheme.set(L)}document.addEventListener("ship:theme:change",b);function w(y){let L=d.panelEl();if(!L)return;let l=Array.from(L.querySelectorAll(".card"));if(!l.length)return;let f=L.getRootNode(),t=l.findIndex((r)=>r===f.activeElement);switch(y.key){case"ArrowDown":y.preventDefault(),l[(t+1+l.length)%l.length]?.focus();break;case"ArrowUp":y.preventDefault(),l[(t-1+l.length)%l.length]?.focus();break;case"Home":y.preventDefault(),l[0]?.focus();break;case"End":y.preventDefault(),l[l.length-1]?.focus();break;case"Escape":y.preventDefault(),L.hidePopover?.();break}}let E=U(()=>{let y=d.panelEl();if(!y)return;return y.addEventListener("keydown",w),()=>y.removeEventListener("keydown",w)}),M=U(()=>{if(!d.panelOpen())return;requestAnimationFrame(()=>{let y=d.panelEl();if(!y)return;let L=Array.from(y.querySelectorAll(".card"));(L.find((f)=>f.getAttribute("aria-selected")==="true")??L[0])?.focus()})});return()=>{document.removeEventListener("ship:theme:change",b),m(),E(),M()}}});var N5=V("sui-theme-picker",{props:{placement:P.union(["top","top-end","bottom","bottom-end","left","left-end","right","right-end"],{default:"bottom-end",reflect:!0}),"label-style":P.union(["text","compact"],{default:"text",reflect:!0}),persist:P.bool({default:!0,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.placement=u.placement,n.labelStyle=u.labelStyle,n.persist=u.persist,n.panelEl=J(null),n.triggerBtn=J(null);let b=null;if(Ii&&typeof Ii.setup==="function"){if(i.scope=n,b=Ii.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(z)=>h({anchor:z,items:()=>n.groups(),key:(g,j)=>{let T=Object.create(n);return T.g=g,T.__i=j,((D)=>g.label)(T)},render:(g,j)=>{let T=Object.create(n);return T.g=g,T.__i=j,T.__v=J(0),((D)=>{let Z=(B)=>h({anchor:B,items:()=>D.g.themes,key:(S,R)=>{let v=Object.create(D);return v.t=S,v.__i=R,((O)=>S.id)(v)},render:(S,R)=>{let v=Object.create(D);return v.t=S,v.__i=R,v.__v=J(0),((O)=>{let x=document.createDocumentFragment(),$=[],o=document.createElement("button");o.setAttribute("class","card"),o.setAttribute("role","option"),o.setAttribute("type","button");{let p=O.t.id;if(p==null||p===!1||p==="")o.removeAttribute("data-theme-card");else o.setAttribute("data-theme-card",p===!0?"":String(p))}{let p=O.t.label;if(p==null||p===!1||p==="")o.removeAttribute("aria-label");else o.setAttribute("aria-label",p===!0?"":String(p))}$.push(W(o,"aria-selected",()=>(O.__v(),O.activeTheme()===O.t.id?"true":"false"))),$.push(W(o,"tabindex",()=>(O.__v(),O.activeTheme()===O.t.id?"0":"-1"))),$.push(nn(o,"click",function(){let p=()=>O.selectTheme(O.t.id);return typeof p==="function"?p.bind(void 0):()=>{}}(),{modifiers:[]}));let bn=document.createElement("span");bn.setAttribute("class","tc-bg");{let p=O.t.id;if(p==null||p===!1||p==="")bn.removeAttribute("data-theme-preview");else bn.setAttribute("data-theme-preview",p===!0?"":String(p))}bn.setAttribute("aria-hidden","true");let Tn=document.createElement("span");Tn.setAttribute("class","tc-overlay"),bn.appendChild(Tn);let fn=document.createElement("span");fn.setAttribute("class","tc-card");let rn=document.createElement("span");rn.setAttribute("class","tc-card-head");let zn=document.createElement("span");zn.setAttribute("class","tc-eyebrow");let K0=document.createTextNode("THEME");zn.appendChild(K0),rn.appendChild(zn);let On=document.createElement("span");On.setAttribute("class","tc-heading");let on=document.createTextNode("");On.appendChild(on),on.textContent=O.t.label,rn.appendChild(On),fn.appendChild(rn);let en=document.createElement("span");en.setAttribute("class","tc-typo");let B0=document.createElement("span");B0.setAttribute("class","tc-body");let Td=document.createTextNode("The quick brown fox");B0.appendChild(Td),en.appendChild(B0);let I0=document.createElement("span");I0.setAttribute("class","tc-mono");let R0=document.createTextNode("");I0.appendChild(R0),R0.textContent="{ build: ok }",en.appendChild(I0),fn.appendChild(en);let Jn=document.createElement("span");Jn.setAttribute("class","tc-controls");let k0=document.createElement("span");k0.setAttribute("class","tc-cta");let Jd=document.createTextNode("Action");k0.appendChild(Jd),Jn.appendChild(k0);let a0=document.createElement("span");a0.setAttribute("class","tc-ghost");let Xd=document.createTextNode("Cancel");a0.appendChild(Xd),Jn.appendChild(a0);let S0=document.createElement("span");S0.setAttribute("class","tc-chip tc-chip-accent");let Gd=document.createTextNode("New");S0.appendChild(Gd),Jn.appendChild(S0);let v0=document.createElement("span");v0.setAttribute("class","tc-chip tc-chip-muted");let Wd=document.createTextNode("Idle");v0.appendChild(Wd),Jn.appendChild(v0),fn.appendChild(Jn),bn.appendChild(fn),o.appendChild(bn);let sn=document.createElement("span");sn.setAttribute("class","row");let x0=document.createElement("span");x0.setAttribute("class","name");let h0=document.createTextNode("");x0.appendChild(h0),h0.textContent=O.t.label,sn.appendChild(x0);let $0=document.createElement("span");return $0.setAttribute("class","radio"),$0.setAttribute("aria-hidden","true"),sn.appendChild($0),o.appendChild(sn),x.appendChild(o),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let p of $)try{p()}catch(Zu){}},update:(p,Zu)=>{O.t=p,O.__i=Zu;{let _=O.t.id;if(_==null||_===!1||_==="")o.removeAttribute("data-theme-card");else o.setAttribute("data-theme-card",_===!0?"":String(_))}{let _=O.t.label;if(_==null||_===!1||_==="")o.removeAttribute("aria-label");else o.setAttribute("aria-label",_===!0?"":String(_))}{let _=O.t.id;if(_==null||_===!1||_==="")bn.removeAttribute("data-theme-preview");else bn.setAttribute("data-theme-preview",_===!0?"":String(_))}on.textContent=O.t.label,R0.textContent="{ build: ok }",h0.textContent=O.t.label,O.__v.set((_)=>_+1)}}})(v)}}),X=document.createDocumentFragment(),G=[],Q=document.createElement("section");Q.setAttribute("class","group");{let B=D.g.label;if(B==null||B===!1||B==="")Q.removeAttribute("aria-label");else Q.setAttribute("aria-label",B===!0?"":String(B))}let q=document.createElement("div");q.setAttribute("class","group-h");let F=document.createTextNode("");q.appendChild(F),F.textContent=D.g.label,Q.appendChild(q);let k=document.createElement("div");k.setAttribute("class","group-rows");let K=document.createComment("for");return k.appendChild(K),G.push(Z(K)),Q.appendChild(k),X.appendChild(Q),{nodes:Array.from(X.childNodes),cleanup:()=>{for(let B of G)try{B()}catch(S){}},update:(B,S)=>{D.g=B,D.__i=S;{let R=D.g.label;if(R==null||R===!1||R==="")Q.removeAttribute("aria-label");else Q.setAttribute("aria-label",R===!0?"":String(R))}F.textContent=D.g.label,D.__v.set((R)=>R+1)}}})(T)}}),E=d.shadowRoot??d;Y(E);let M=document.createElement("button");M.setAttribute("class","trigger"),M.setAttribute("part","trigger"),M.setAttribute("type","button"),A(n,"triggerBtn",M),M.setAttribute("aria-label","Choose theme"),M.setAttribute("aria-haspopup","dialog"),m.push(W(M,"aria-expanded",()=>n.panelOpen()?"true":"false")),m.push(nn(M,"click",function(){let z=()=>n.togglePanel();return typeof z==="function"?z.bind(void 0):()=>{}}(),{modifiers:[]}));let y=document.createElement("span");y.setAttribute("class","dot"),y.setAttribute("part","dot"),m.push(U(()=>{let z=n.dotColor();if(z==null)y.style.removeProperty("background");else y.style.setProperty("background",String(z))})),y.setAttribute("aria-hidden","true"),M.appendChild(y);let L=document.createElement("span");L.setAttribute("class","label"),L.setAttribute("part","label");let l=document.createTextNode("");L.appendChild(l),m.push(a(l,()=>n.triggerLabel())),M.appendChild(L);let f=document.createElement("svg");f.setAttribute("class","caret"),f.setAttribute("viewbox","0 0 10 6"),f.setAttribute("aria-hidden","true");let t=document.createElement("path");t.setAttribute("d","M1 1l4 4 4-4"),t.setAttribute("fill","none"),t.setAttribute("stroke","currentColor"),t.setAttribute("stroke-width","1.5"),t.setAttribute("stroke-linecap","round"),t.setAttribute("stroke-linejoin","round"),f.appendChild(t),M.appendChild(f),E.appendChild(M);let r=document.createElement("div");r.setAttribute("class","panel"),r.setAttribute("part","panel"),r.setAttribute("popover","auto"),A(n,"panelEl",r);let H=document.createElement("div");H.setAttribute("class","grid"),H.setAttribute("part","grid"),H.setAttribute("role","listbox"),H.setAttribute("aria-label","Theme");let C=document.createComment("for");if(H.appendChild(C),m.push(w(C)),r.appendChild(H),E.appendChild(r),b!=null){let z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(z){let g=new Proxy(n,{get(j,T){return j[T]},set(j,T,D){let Z=j[T];if(Z!=null&&typeof Z.set==="function"){if(typeof D==="function")Z.set(()=>D);else Z.set(D);return!0}return!1}});if(z.mount){let j=z.mount(g,d);if(typeof j==="function")m.push(j)}if(z.unmount)m.push(()=>z.unmount(g))}}return n}});var Xm=`
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
`,bd=!1;function Gm(){if(bd)return;bd=!0;let i=document.createElement("style");i.setAttribute("data-sui-rail",""),i.textContent=Xm,document.head.appendChild(i)}function Ri(i,u,d){i.dispatchEvent(new CustomEvent(u,{detail:d,bubbles:!0,composed:!0}))}function fd(i){Gm();let{host:u,open:d,projects:m,activeProjectId:n,popoverEl:b,badgeEl:w}=i,E=()=>d.set(!1),M=async()=>{try{let g=await fetch("/api/v1/projects",{credentials:"same-origin"});if(!g.ok)return;let j=await g.json();m.set(j.projects??[]),n.set(j.active??null)}catch{}},y=async(g)=>{try{await fetch("/api/v1/projects/switch",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({id:g})}),n.set(g),E(),Ri(u,"ship:project:change",{id:g})}catch{}M()},L=async()=>{let g=window.prompt("Path to project directory:");if(!g)return;try{let j=await fetch("/api/v1/projects",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({path:g})});if(!j.ok){let T=await j.text().catch(()=>"");Ri(u,"sui:notify",{tone:"warning",title:`Add failed: ${T||j.status}`});return}await M(),Ri(u,"sui:notify",{tone:"success",title:"Project added."})}catch(j){Ri(u,"sui:notify",{tone:"destructive",title:String(j.message??j)})}},l=()=>{let g=b(),j=w();if(!g||!j)return;let T=j.getBoundingClientRect(),D=g.offsetWidth||220,Z=8,X=Math.max(8,Math.min(T.left,window.innerWidth-D-8)),G=Math.max(8,window.innerHeight-T.top+Z);g.style.left=`${X}px`,g.style.bottom=`${G}px`},f=(g)=>{let j=g.target;if(!j)return;if(j.closest("[data-rail-add]")){L();return}let T=j.closest("[data-project-id]");if(T?.dataset.projectId)y(T.dataset.projectId)},t=()=>d.set(!d()),r=(g)=>{if(!d())return;let j=typeof g.composedPath==="function"?g.composedPath():[],T=b(),D=w();for(let Z of j)if(Z===T||Z===D)return;E()},H=()=>{if(d())l()},C=U(()=>{let g=w();if(!g)return;return g.addEventListener("click",t),()=>g.removeEventListener("click",t)}),z=U(()=>{let g=b();if(!g)return;return requestAnimationFrame(l),g.addEventListener("click",f),()=>g.removeEventListener("click",f)});return document.addEventListener("click",r,!0),window.addEventListener("resize",H),window.addEventListener("scroll",H,{capture:!0,passive:!0}),{loadProjects:M,dispose(){C(),z(),document.removeEventListener("click",r,!0),window.removeEventListener("resize",H),window.removeEventListener("scroll",H,!0)}}}var Wm=15000;function td(i){return i.split("/").filter(Boolean)[0]??"dashboard"}function rd(i,u,d){i.dispatchEvent(new CustomEvent(u,{detail:d,bubbles:!0,composed:!0}))}var ki=N({scope:()=>{let i=J(!1),u=J([]),d=J(null),m=J(typeof location<"u"?td(location.pathname):""),n=J(0);return{projectOpen:i,projects:u,activeProjectId:d,activeView:m,sessionCount:n,projectInitial:()=>{let w=u(),E=w.find((M)=>M.id===d())??w[0];if(!E)return"";return(E.label??E.name).charAt(0).toUpperCase()},popoverEl:e(),projectBadgeEl:e(),launcherBtnEl:e(),createBtnEl:e(),missionEl:e(),navSlot:e()}},mount(i,u){let d=fd({host:u,open:i.projectOpen,projects:i.projects,activeProjectId:i.activeProjectId,popoverEl:i.popoverEl,badgeEl:i.projectBadgeEl});d.loadProjects();let m=()=>i.projectOpen.set(!1),n=(H)=>{rd(u,"ship:launcher:open",{anchor:H.currentTarget}),m()},b=()=>{rd(u,"ship:create:open"),m()},w=U(()=>{let H=i.launcherBtnEl();if(!H)return;return H.addEventListener("click",n),()=>H.removeEventListener("click",n)}),E=U(()=>{let H=i.createBtnEl();if(!H)return;return H.addEventListener("click",b),()=>H.removeEventListener("click",b)}),M=async()=>{try{let H=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(!H.ok)return;let z=(await H.json()).sessions.filter((g)=>g.status!=="ended").length;i.sessionCount.set(z)}catch{}};M();let y=window.setInterval(()=>void M(),Wm),L=()=>void M();window.addEventListener("ship:session:changed",L);let l=U(()=>{let H=i.missionEl();if(!H)return;let C=i.sessionCount();H.setAttribute("badge",C>0?String(C):"")}),f=()=>{i.activeView.set(td(location.pathname))},t=()=>f();document.addEventListener("sui:navigate",t),window.addEventListener("popstate",t);let r=U(()=>{let H=i.activeView(),C=u.querySelectorAll("sui-rail-item[data-view]");for(let g of C)if(g.dataset.view===H)g.setAttribute("active","");else g.removeAttribute("active");let z=i.missionEl();if(z)if(H==="mesh")z.setAttribute("active","");else z.removeAttribute("active")});return()=>{w(),E(),l(),r(),d.dispose(),document.removeEventListener("sui:navigate",t),window.removeEventListener("popstate",t),window.removeEventListener("ship:session:changed",L),window.clearInterval(y)}}});var a5=V("sui-rail",{props:{collapsed:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.collapsed=u.collapsed,n.createBtnEl=J(null),n.launcherBtnEl=J(null),n.navSlot=J(null),n.popoverEl=J(null),n.projectBadgeEl=J(null);let b=null;if(ki&&typeof ki.setup==="function"){if(i.scope=n,b=ki.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(Z)=>I({anchor:Z,test:()=>{if(n.projects().length===0)return 0;return-1},branches:[()=>{let X=document.createDocumentFragment(),G=[],Q=document.createElement("div");Q.setAttribute("class","sui-rail-project-popover-empty");let q=document.createTextNode("No projects tracked.");return Q.appendChild(q),X.appendChild(Q),{nodes:Array.from(X.childNodes),cleanup:()=>{for(let F of G)try{F()}catch(k){}}}}]}),E=(Z)=>h({anchor:Z,items:()=>n.projects(),key:(X,G)=>{let Q=Object.create(n);return Q.p=X,Q.__i=G,((q)=>X.id)(Q)},render:(X,G)=>{let Q=Object.create(n);return Q.p=X,Q.__i=G,Q.__v=J(0),((q)=>{let F=document.createDocumentFragment(),k=[],K=document.createElement("div");K.setAttribute("class","sui-rail-project-popover-row"),k.push(Cn(K,"active",()=>(q.__v(),q.p.id===q.activeProjectId())));{let O=q.p.id;if(O==null||O===!1||O==="")K.removeAttribute("data-project-id");else K.setAttribute("data-project-id",O===!0?"":String(O))}let B=document.createElement("span");B.setAttribute("class","sui-rail-project-popover-row-name");let S=document.createTextNode("");B.appendChild(S),S.textContent=q.p.label||q.p.name,K.appendChild(B);let R=document.createElement("span");R.setAttribute("class","sui-rail-project-popover-row-path");let v=document.createTextNode("");return R.appendChild(v),v.textContent=q.p.path,K.appendChild(R),F.appendChild(K),{nodes:Array.from(F.childNodes),cleanup:()=>{for(let O of k)try{O()}catch(x){}},update:(O,x)=>{q.p=O,q.__i=x;{let $=q.p.id;if($==null||$===!1||$==="")K.removeAttribute("data-project-id");else K.setAttribute("data-project-id",$===!0?"":String($))}S.textContent=q.p.label||q.p.name,v.textContent=q.p.path,q.__v.set(($)=>$+1)}}})(Q)}}),M=(Z)=>I({anchor:Z,test:()=>{if(n.projectOpen())return 0;return-1},branches:[()=>{let X=document.createDocumentFragment(),G=[],Q=document.createElement("div");Q.setAttribute("class","sui-rail-project-popover"),Q.setAttribute("part","project-popover"),Q.setAttribute("role","dialog"),Q.setAttribute("aria-label","Switch project"),A(n,"popoverEl",Q),Ln(Q,Dn,{duration:150},"both");let q=document.createElement("div");q.setAttribute("class","sui-rail-project-popover-title");let F=document.createTextNode("Projects");q.appendChild(F),Q.appendChild(q);let k=document.createComment("if");Q.appendChild(k),G.push(w(k));let K=document.createComment("for");Q.appendChild(K),G.push(E(K));let B=document.createElement("div");B.setAttribute("class","sui-rail-project-popover-sep"),Q.appendChild(B);let S=document.createElement("div");S.setAttribute("class","sui-rail-project-popover-row add"),S.setAttribute("data-rail-add","");let R=document.createElement("span"),v=document.createTextNode("+ Add project");return R.appendChild(v),S.appendChild(R),Q.appendChild(S),X.appendChild(Q),{nodes:Array.from(X.childNodes),cleanup:()=>{for(let O of G)try{O()}catch(x){}}}}]}),y=d.shadowRoot??d;Y(y);let L=document.createElement("nav");L.setAttribute("class","rail-nav"),L.setAttribute("part","nav");let l=document.createElement("slot");A(n,"navSlot",l),L.appendChild(l),y.appendChild(L);let f=document.createElement("div");f.setAttribute("class","rail-footer"),f.setAttribute("part","footer");let t=document.createElement("button");t.setAttribute("type","button"),t.setAttribute("class","project-badge"),t.setAttribute("part","project-badge"),t.setAttribute("aria-label","Switch project"),t.setAttribute("aria-haspopup","dialog"),t.setAttribute("title","Switch project"),A(n,"projectBadgeEl",t);let r=document.createTextNode("");t.appendChild(r),m.push(a(r,()=>n.projectInitial())),f.appendChild(t);let H=document.createElement("div");H.setAttribute("class","footer-actions");let C=document.createElement("button");C.setAttribute("type","button"),C.setAttribute("class","footer-btn"),C.setAttribute("part","launcher-btn"),C.setAttribute("title","Open launcher"),C.setAttribute("aria-label","Open launcher"),A(n,"launcherBtnEl",C);let z=document.createElement("sui-icon");z.setAttribute("name","folder"),z.setAttribute("size","sm"),C.appendChild(z),H.appendChild(C);let g=document.createElement("button");g.setAttribute("type","button"),g.setAttribute("class","footer-btn"),g.setAttribute("part","create-btn"),g.setAttribute("title","Create new"),g.setAttribute("aria-label","Create new"),A(n,"createBtnEl",g);let j=document.createElement("sui-icon");j.setAttribute("name","plus"),j.setAttribute("size","sm"),g.appendChild(j),H.appendChild(g),f.appendChild(H),y.appendChild(f);let T=document.createElement("sui-portal");T.setAttribute("target","body");let D=document.createComment("if");if(T.appendChild(D),m.push(M(D)),y.appendChild(T),b!=null){let Z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(Z){let X=new Proxy(n,{get(G,Q){return G[Q]},set(G,Q,q){let F=G[Q];if(F!=null&&typeof F.set==="function"){if(typeof q==="function")F.set(()=>q);else F.set(q);return!0}return!1}});if(Z.mount){let G=Z.mount(X,d);if(typeof G==="function")m.push(G)}if(Z.unmount)m.push(()=>Z.unmount(X))}}return n}});var x5=V("sui-rail-item",{props:{href:P.string({default:""}),label:P.string({default:""}),icon:P.string({default:""}),active:P.bool({default:!1,reflect:!0}),badge:P.string({default:""}),disabled:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.href=u.href,n.label=u.label,n.icon=u.icon,n.active=u.active,n.badge=u.badge,n.disabled=u.disabled,n.badgeEl=J(null),n.itemEl=J(null);let b=d.shadowRoot??d;Y(b);let w=document.createElement("a");w.setAttribute("class","item"),w.setAttribute("part","item"),A(n,"itemEl",w),m.push(W(w,"href",()=>n.href()||"#")),m.push(W(w,"aria-current",()=>n.active()?"page":null)),m.push(W(w,"aria-disabled",()=>n.disabled()?"true":null)),m.push(W(w,"title",()=>n.label()));let E=document.createElement("span");E.setAttribute("class","icon-wrap"),E.setAttribute("part","icon-wrap");let M=document.createElement("sui-icon");m.push(W(M,"name",()=>n.icon())),M.setAttribute("size","sm"),E.appendChild(M),w.appendChild(E);let y=document.createElement("span");y.setAttribute("class","label"),y.setAttribute("part","label");let L=document.createTextNode("");y.appendChild(L),m.push(a(L,()=>n.label())),w.appendChild(y);let l=document.createElement("span");l.setAttribute("class","badge"),l.setAttribute("part","badge"),A(n,"badgeEl",l);let f=document.createTextNode("");return l.appendChild(f),m.push(a(f,()=>n.badge())),w.appendChild(l),b.appendChild(w),n}});var o5=V("sui-rail-section",{props:{label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.label=u.label;let b=d.shadowRoot??d;Y(b);let w=document.createElement("span");w.setAttribute("class","heading"),w.setAttribute("part","heading");let E=document.createTextNode("");w.appendChild(E),m.push(a(E,()=>n.label())),b.appendChild(w);let M=document.createElement("slot");return b.appendChild(M),n}});var p5=V("sui-status-bar",{props:{branch:P.string({default:""}),theme:P.string({default:""}),online:P.bool({default:!1,reflect:!0}),agents:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.branch=u.branch,n.theme=u.theme,n.online=u.online,n.agents=u.agents;let b=d.shadowRoot??d;Y(b);let w=document.createElement("span");w.setAttribute("class","dot"),w.setAttribute("part","dot"),w.setAttribute("aria-hidden","true"),b.appendChild(w);let E=document.createElement("span");E.setAttribute("class","text"),E.setAttribute("part","branch");let M=document.createTextNode("");E.appendChild(M),m.push(a(M,()=>n.branch())),b.appendChild(E);let y=document.createElement("span");y.setAttribute("class","text"),y.setAttribute("part","agents");let L=document.createTextNode("");y.appendChild(L),m.push(a(L,()=>n.agents())),b.appendChild(y);let l=document.createElement("span");l.setAttribute("class","slot-wrap");let f=document.createElement("slot");f.setAttribute("name","start"),l.appendChild(f),b.appendChild(l);let t=document.createElement("span");t.setAttribute("class","spacer"),b.appendChild(t);let r=document.createElement("span");r.setAttribute("class","slot-wrap");let H=document.createElement("slot");H.setAttribute("name","end"),r.appendChild(H),b.appendChild(r);let C=document.createElement("span");C.setAttribute("class","text"),C.setAttribute("part","theme");let z=document.createTextNode("");return C.appendChild(z),m.push(a(z,()=>n.theme())),b.appendChild(C),n}});var Qm=["a[href]","area[href]","button:not([disabled])",'input:not([disabled]):not([type="hidden"])',"select:not([disabled])","textarea:not([disabled])","iframe","object","embed","summary",'[contenteditable]:not([contenteditable="false"])','[tabindex]:not([tabindex^="-"])'].join(",");function An(i){return Array.from(i.querySelectorAll(Qm)).filter((u)=>{if(u.hasAttribute("disabled"))return!1;if(u.getAttribute("aria-hidden")==="true")return!1;return!0})}class hn{saved=null;save(){this.saved=document.activeElement??null}restore(){let i=this.saved;if(this.saved=null,i&&typeof i.focus==="function")i.focus()}}var xn=0,gu=null,Pu=null;function ai(){if(typeof document>"u")return;if(xn+=1,xn>1)return;let i=document.body;gu=i.style.overflow,Pu=i.style.paddingRight;let u=typeof window<"u"?window.innerWidth-document.documentElement.clientWidth:0;if(u>0){let d=parseFloat(getComputedStyle(i).paddingRight||"0")||0;i.style.paddingRight=`${d+u}px`}i.style.overflow="hidden"}function Fn(){if(typeof document>"u")return;if(xn===0)return;if(xn-=1,xn>0)return;let i=document.body;i.style.overflow=gu??"",i.style.paddingRight=Pu??"",gu=null,Pu=null}var Vm=0,Si=N({mount(i,u){let d=i,m=++Vm,n=`sui-dialog-header-${m}`,b=`sui-dialog-body-${m}`,w=d.dialogEl(),E=d.closeBtn(),M=d.sentinelStart(),y=d.sentinelEnd();if(!w)return;let L=U(()=>{let q=d.headerEl();if(q)q.id=n}),l=U(()=>{let q=d.bodyEl();if(q)q.id=b});w.setAttribute("aria-labelledby",n),w.setAttribute("aria-describedby",b);let f=!1,t=new hn;function r(){w.setAttribute("role",d.roleOverride()||"dialog"),w.setAttribute("aria-modal","true")}r();function H(){let q=An(w).filter((k)=>!k.classList.contains("sentinel"));return[...An(u),...q]}function C(){let q=u.querySelector("[autofocus]")??w.querySelector("[autofocus]");if(q){q.focus();return}let F=H(),k=F.find((K)=>K!==E)??F[0];if(k)k.focus();else w.focus()}function z(){t.save();try{w.showModal()}catch{}if(!f)ai(),f=!0;queueMicrotask(()=>C()),u.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function g(){if(w.open)w.close()}let j=()=>{if(d.open())d.open.set(!1);if(f)Fn(),f=!1;t.restore(),u.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},T=(q)=>{if(!d.dismissible())return;if(q.target===w)d.open.set(!1)},D=()=>{u.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),d.open.set(!1)},Z=()=>{let q=H();if(q.length)q[q.length-1].focus()},X=()=>{let q=H();if(q.length)q[0].focus()};w.addEventListener("close",j),w.addEventListener("click",T),E?.addEventListener("click",D),M?.addEventListener("focus",Z),y?.addEventListener("focus",X);let G=U(()=>{let q=d.open();if(u.setAttribute("data-state",q?"open":"closed"),q&&!w.open)z();else if(!q&&w.open)g()}),Q=U(()=>{r()});return u.show=()=>d.open.set(!0),u.showModal=()=>d.open.set(!0),u.close=()=>d.open.set(!1),()=>{if(f)Fn();w.removeEventListener("close",j),w.removeEventListener("click",T),E?.removeEventListener("click",D),M?.removeEventListener("focus",Z),y?.removeEventListener("focus",X),G(),Q(),L(),l()}}});var b8=V("sui-dialog",{props:{open:P.bool({default:!1,reflect:!0}),size:P.union(["sm","md","lg","full"],{default:"md",reflect:!0}),backdrop:P.union(["default","blur","solid"],{default:"default",reflect:!0}),dismissible:P.bool({default:!0}),"no-close-button":P.bool({default:!1,reflect:!0}),"no-header":P.bool({default:!1,reflect:!0}),"no-footer":P.bool({default:!1,reflect:!0}),"role-override":P.union(["dialog","alertdialog"],{default:"dialog",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.size=u.size,n.backdrop=u.backdrop,n.dismissible=u.dismissible,n.noCloseButton=u.noCloseButton,n.noHeader=u.noHeader,n.noFooter=u.noFooter,n.roleOverride=u.roleOverride,n.bodyEl=J(null),n.closeBtn=J(null),n.dialogEl=J(null),n.headerEl=J(null),n.sentinelEnd=J(null),n.sentinelStart=J(null);let b=null;if(Si&&typeof Si.setup==="function"){if(i.scope=n,b=Si.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(z)=>I({anchor:z,test:()=>{if(n.open())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),j=[],T=document.createElement("div");T.setAttribute("class","dialog-content"),Ln(T,Dn,{duration:150},"both");let D=document.createElement("div");D.setAttribute("class","header"),D.setAttribute("part","header"),A(n,"headerEl",D);let Z=document.createElement("slot");Z.setAttribute("name","header"),D.appendChild(Z),T.appendChild(D);let X=document.createElement("div");X.setAttribute("class","body"),X.setAttribute("part","body"),A(n,"bodyEl",X);let G=document.createElement("slot");X.appendChild(G),T.appendChild(X);let Q=document.createElement("div");Q.setAttribute("class","footer"),Q.setAttribute("part","footer");let q=document.createElement("slot");return q.setAttribute("name","footer"),Q.appendChild(q),T.appendChild(Q),g.appendChild(T),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let F of j)try{F()}catch(k){}}}}]}),E=(z)=>I({anchor:z,test:()=>{if(n.open())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),j=[],T=document.createElement("sui-portal");T.setAttribute("target","body");let D=document.createElement("div");return D.setAttribute("class","sui-dialog-presence"),D.setAttribute("aria-hidden","true"),D.setAttribute("data-sui-dialog-open",""),T.appendChild(D),g.appendChild(T),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let Z of j)try{Z()}catch(X){}}}}]}),M=d.shadowRoot??d;Y(M);let y=document.createElement("dialog");y.setAttribute("part","dialog"),A(n,"dialogEl",y);let L=document.createElement("span");L.setAttribute("tabindex","0"),L.setAttribute("class","sentinel"),L.setAttribute("aria-hidden","true"),A(n,"sentinelStart",L),y.appendChild(L);let l=document.createElement("button");l.setAttribute("class","close"),l.setAttribute("part","close"),l.setAttribute("type","button"),l.setAttribute("aria-label","Close dialog"),A(n,"closeBtn",l);let f=document.createElement("svg");f.setAttribute("viewbox","0 0 16 16"),f.setAttribute("fill","none"),f.setAttribute("stroke","currentColor"),f.setAttribute("stroke-width","1.75"),f.setAttribute("stroke-linecap","round"),f.setAttribute("stroke-linejoin","round"),f.setAttribute("aria-hidden","true"),f.setAttribute("focusable","false");let t=document.createElement("path");t.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),f.appendChild(t),l.appendChild(f),y.appendChild(l);let r=document.createComment("if");y.appendChild(r),m.push(w(r));let H=document.createElement("span");H.setAttribute("tabindex","0"),H.setAttribute("class","sentinel"),H.setAttribute("aria-hidden","true"),A(n,"sentinelEnd",H),y.appendChild(H),M.appendChild(y);let C=document.createComment("if");if(M.appendChild(C),m.push(E(C)),b!=null){let z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(z){let g=new Proxy(n,{get(j,T){return j[T]},set(j,T,D){let Z=j[T];if(Z!=null&&typeof Z.set==="function"){if(typeof D==="function")Z.set(()=>D);else Z.set(D);return!0}return!1}});if(z.mount){let j=z.mount(g,d);if(typeof j==="function")m.push(j)}if(z.unmount)m.push(()=>z.unmount(g))}}return n}});var Ym=0,vi=N({mount(i,u){let d=i,m=++Ym,n=`sui-drawer-header-${m}`,b=`sui-drawer-body-${m}`,w=d.dialogEl(),E=d.closeBtn(),M=d.sentinelStart(),y=d.sentinelEnd(),L=d.headerEl(),l=d.bodyEl();if(!w)return;if(L)L.id=n;if(l)l.id=b;w.setAttribute("aria-labelledby",n),w.setAttribute("aria-describedby",b);let f=!1,t=new hn;function r(){let G=An(w).filter((q)=>!q.classList.contains("sentinel"));return[...An(u),...G]}function H(){let G=u.querySelector("[autofocus]")??w.querySelector("[autofocus]");if(G){G.focus();return}let Q=r(),q=Q.find((F)=>F!==E)??Q[0];if(q)q.focus();else w.focus()}function C(){t.save();try{w.showModal()}catch{}if(!f)ai(),f=!0;queueMicrotask(()=>H()),u.dispatchEvent(new CustomEvent("sui-open",{bubbles:!0,composed:!0}))}function z(){if(w.open)w.close()}let g=()=>{if(d.open())d.open.set(!1);if(f)Fn(),f=!1;t.restore(),u.dispatchEvent(new CustomEvent("sui-close",{bubbles:!0,composed:!0}))},j=(G)=>{if(!d.dismissible())return;if(G.target===w)d.open.set(!1)},T=()=>{u.dispatchEvent(new CustomEvent("sui-dismiss",{bubbles:!0,composed:!0})),d.open.set(!1)},D=()=>{let G=r();if(G.length)G[G.length-1].focus()},Z=()=>{let G=r();if(G.length)G[0].focus()};w.addEventListener("close",g),w.addEventListener("click",j),E?.addEventListener("click",T),M?.addEventListener("focus",D),y?.addEventListener("focus",Z);let X=U(()=>{let G=d.open();if(u.setAttribute("data-state",G?"open":"closed"),G&&!w.open)C();else if(!G&&w.open)z()});return u.show=()=>d.open.set(!0),u.close=()=>d.open.set(!1),()=>{if(f)Fn();w.removeEventListener("close",g),w.removeEventListener("click",j),E?.removeEventListener("click",T),M?.removeEventListener("focus",D),y?.removeEventListener("focus",Z),X()}}});var M8=V("sui-drawer",{props:{open:P.bool({default:!1,reflect:!0}),side:P.union(["right","left"],{default:"right",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),dismissible:P.bool({default:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.side=u.side,n.size=u.size,n.dismissible=u.dismissible,n.bodyEl=J(null),n.closeBtn=J(null),n.dialogEl=J(null),n.headerEl=J(null),n.sentinelEnd=J(null),n.sentinelStart=J(null);let b=null;if(vi&&typeof vi.setup==="function"){if(i.scope=n,b=vi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("dialog");E.setAttribute("part","dialog"),E.setAttribute("role","dialog"),E.setAttribute("aria-modal","true"),A(n,"dialogEl",E);let M=document.createElement("span");M.setAttribute("tabindex","0"),M.setAttribute("class","sentinel"),M.setAttribute("aria-hidden","true"),A(n,"sentinelStart",M),E.appendChild(M);let y=document.createElement("div");y.setAttribute("class","header"),y.setAttribute("part","header"),A(n,"headerEl",y);let L=document.createElement("div");L.setAttribute("class","header-content");let l=document.createElement("div");l.setAttribute("class","title");let f=document.createElement("slot");f.setAttribute("name","title"),l.appendChild(f),L.appendChild(l);let t=document.createElement("div");t.setAttribute("class","description");let r=document.createElement("slot");r.setAttribute("name","description"),t.appendChild(r),L.appendChild(t),y.appendChild(L);let H=document.createElement("button");H.setAttribute("class","close"),H.setAttribute("part","close"),H.setAttribute("type","button"),H.setAttribute("aria-label","Close drawer"),A(n,"closeBtn",H);let C=document.createElement("svg");C.setAttribute("viewbox","0 0 16 16"),C.setAttribute("fill","none"),C.setAttribute("stroke","currentColor"),C.setAttribute("stroke-width","1.75"),C.setAttribute("stroke-linecap","round"),C.setAttribute("stroke-linejoin","round"),C.setAttribute("aria-hidden","true"),C.setAttribute("focusable","false");let z=document.createElement("path");z.setAttribute("d","M3.5 3.5l9 9M12.5 3.5l-9 9"),C.appendChild(z),H.appendChild(C),y.appendChild(H),E.appendChild(y);let g=document.createElement("div");g.setAttribute("class","body"),g.setAttribute("part","body"),A(n,"bodyEl",g);let j=document.createElement("slot");g.appendChild(j),E.appendChild(g);let T=document.createElement("div");T.setAttribute("class","footer"),T.setAttribute("part","footer");let D=document.createElement("slot");D.setAttribute("name","footer"),T.appendChild(D),E.appendChild(T);let Z=document.createElement("span");if(Z.setAttribute("tabindex","0"),Z.setAttribute("class","sentinel"),Z.setAttribute("aria-hidden","true"),A(n,"sentinelEnd",Z),E.appendChild(Z),w.appendChild(E),b!=null){let X=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(X){let G=new Proxy(n,{get(Q,q){return Q[q]},set(Q,q,F){let k=Q[q];if(k!=null&&typeof k.set==="function"){if(typeof F==="function")k.set(()=>F);else k.set(F);return!0}return!1}});if(X.mount){let Q=X.mount(G,d);if(typeof Q==="function")m.push(Q)}if(X.unmount)m.push(()=>X.unmount(G))}}return n}});var qm=0,xi=N({mount(i,u){let d=i,m=`sui-dropdown-menu-panel-${++qm}`,n=d.triggerWrap(),b=d.triggerSlot(),w=d.panelEl();if(!w)return;w.id=m;let E="",M=null;function y(Z){if(M)clearTimeout(M);E+=Z.toLowerCase(),M=setTimeout(()=>{E=""},350);let X=L(),G=X.indexOf(document.activeElement);for(let Q=0;Q<X.length;Q++){let q=(G+1+Q)%X.length;if((X[q].textContent??"").trim().toLowerCase().startsWith(E))return q}return-1}function L(){return Array.from(u.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((Z)=>!Z.hasAttribute("disabled"))}function l(Z){let X=L();if(!X.length)return;let G=(Z%X.length+X.length)%X.length;for(let Q of X)Q.setAttribute("tabindex","-1");X[G].setAttribute("tabindex","0"),X[G].focus()}function f(){return L().indexOf(document.activeElement)}function t(){L().forEach((Z,X)=>{if(!Z.hasAttribute("role"))Z.setAttribute("role","menuitem");if(!Z.hasAttribute("tabindex"))Z.setAttribute("tabindex",X===0?"0":"-1")})}function r(){let Z=b?.assignedElements({flatten:!0})??[];for(let X of Z)X.setAttribute("aria-haspopup","menu"),X.setAttribute("aria-controls",m),X.setAttribute("aria-expanded",d.open()?"true":"false")}function H(){let Z=b?.assignedElements({flatten:!0})?.[0];if(Z&&w){let X=Z.getBoundingClientRect();w.style.top=`${X.bottom+4}px`,w.style.left=`${X.left}px`}w?.showPopover?.()}function C(){w?.hidePopover?.()}let z=()=>{if(d.open())C();else H()},g=(Z)=>{let G=Z.newState==="open";if(G!==d.open())d.open.set(G);if(u.setAttribute("data-state",G?"open":"closed"),r(),G)queueMicrotask(()=>{t(),l(0)})},j=(Z)=>{let X=Z.target.closest("[data-value]");if(!X)return;u.emit("sui-select",{value:X.dataset.value??""}),C()},T=(Z)=>{if(!d.open())return;let X=L();if(!X.length)return;let G=f();if(Z.key==="ArrowDown")Z.preventDefault(),l(G+1);else if(Z.key==="ArrowUp")Z.preventDefault(),l(G-1);else if(Z.key==="Home")Z.preventDefault(),l(0);else if(Z.key==="End")Z.preventDefault(),l(X.length-1);else if(Z.key==="Escape")Z.preventDefault(),C();else if(Z.key==="Tab")C();else if((Z.key==="Enter"||Z.key===" ")&&G>=0)Z.preventDefault(),X[G].click();else if(Z.key.length===1&&!Z.ctrlKey&&!Z.metaKey){let Q=y(Z.key);if(Q>=0)Z.preventDefault(),l(Q)}};if(n)c(n,"click",z);if(c(w,"toggle",g),c(w,"click",j),c(u,"keydown",T),b)c(b,"slotchange",r);c(w,"slotchange",t);let D=U(()=>{let Z=d.open();u.setAttribute("data-state",Z?"open":"closed"),r()});return()=>{if(M)clearTimeout(M);D()}}});var g8=V("sui-dropdown-menu",{props:{open:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.itemSlot=J(null),n.panelEl=J(null),n.triggerSlot=J(null),n.triggerWrap=J(null);let b=null;if(xi&&typeof xi.setup==="function"){if(i.scope=n,b=xi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("span");E.setAttribute("class","trigger-wrap"),A(n,"triggerWrap",E);let M=document.createElement("slot");M.setAttribute("name","trigger"),A(n,"triggerSlot",M),E.appendChild(M),w.appendChild(E);let y=document.createElement("div");y.setAttribute("class","panel"),y.setAttribute("part","panel"),y.setAttribute("popover","auto"),y.setAttribute("role","menu"),A(n,"panelEl",y);let L=document.createElement("slot");if(A(n,"itemSlot",L),y.appendChild(L),w.appendChild(y),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let f=new Proxy(n,{get(t,r){return t[r]},set(t,r,H){let C=t[r];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(l.mount){let t=l.mount(f,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(f))}}return n}});var Um=0,hi=N({mount(i,u){let d=i,m=`sui-context-menu-panel-${++Um}`,n=d.triggerSlot(),b=d.panelEl();if(!b)return;b.id=m;let w="",E=null;function M(D){if(E)clearTimeout(E);w+=D.toLowerCase(),E=setTimeout(()=>{w=""},350);let Z=y(),X=Z.indexOf(document.activeElement);for(let G=0;G<Z.length;G++){let Q=(X+1+G)%Z.length;if((Z[Q].textContent??"").trim().toLowerCase().startsWith(w))return Q}return-1}function y(){return Array.from(u.querySelectorAll('[role="menuitem"], button[data-value], [data-value]')).filter((D)=>!D.hasAttribute("disabled"))}function L(D){let Z=y();if(!Z.length)return;let X=(D%Z.length+Z.length)%Z.length;for(let G of Z)G.setAttribute("tabindex","-1");Z[X].setAttribute("tabindex","0"),Z[X].focus()}function l(){return y().indexOf(document.activeElement)}function f(){y().forEach((D,Z)=>{if(!D.hasAttribute("role"))D.setAttribute("role","menuitem");if(!D.hasAttribute("tabindex"))D.setAttribute("tabindex",Z===0?"0":"-1")})}function t(D,Z){if(b)b.style.top=`${Z}px`,b.style.left=`${D}px`;b?.showPopover?.()}function r(){b?.hidePopover?.()}u.expose({show:t,close:r});let H=(D)=>{D.preventDefault(),t(D.clientX,D.clientY)},C=(D)=>{let X=D.newState==="open";if(X!==d.open())d.open.set(X);if(u.setAttribute("data-state",X?"open":"closed"),X)u.emit("sui-open"),queueMicrotask(()=>{f(),L(0)});else u.emit("sui-close")},z=(D)=>{let Z=D.target.closest("[data-value]");if(!Z)return;u.emit("sui-select",{value:Z.dataset.value??""}),r()},g=(D)=>{if(!d.open())return;let Z=y();if(!Z.length)return;let X=l();if(D.key==="ArrowDown")D.preventDefault(),L(X+1);else if(D.key==="ArrowUp")D.preventDefault(),L(X-1);else if(D.key==="Home")D.preventDefault(),L(0);else if(D.key==="End")D.preventDefault(),L(Z.length-1);else if(D.key==="Escape")D.preventDefault(),r();else if(D.key==="Tab")r();else if((D.key==="Enter"||D.key===" ")&&X>=0)D.preventDefault(),Z[X].click();else if(D.key.length===1&&!D.ctrlKey&&!D.metaKey){let G=M(D.key);if(G>=0)D.preventDefault(),L(G)}},j=()=>{let D=n?.assignedElements({flatten:!0})??[];for(let Z of D)Z.addEventListener("contextmenu",H)};if(n)c(n,"slotchange",j);j(),c(b,"toggle",C),c(b,"click",z),c(u,"keydown",g);let T=U(()=>{u.setAttribute("data-state",d.open()?"open":"closed")});return()=>{let D=n?.assignedElements({flatten:!0})??[];for(let Z of D)Z.removeEventListener("contextmenu",H);if(E)clearTimeout(E);T()}}});var J8=V("sui-context-menu",{props:{open:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.itemSlot=J(null),n.panelEl=J(null),n.triggerSlot=J(null);let b=null;if(hi&&typeof hi.setup==="function"){if(i.scope=n,b=hi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");E.setAttribute("name","trigger"),A(n,"triggerSlot",E),w.appendChild(E);let M=document.createElement("div");M.setAttribute("class","panel"),M.setAttribute("part","panel"),M.setAttribute("popover","auto"),M.setAttribute("role","menu"),M.setAttribute("aria-label","Context menu"),A(n,"panelEl",M);let y=document.createElement("slot");if(A(n,"itemSlot",y),M.appendChild(y),w.appendChild(M),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var Nm=0,$i=8;function Am(i,u,d){let m=i.getBoundingClientRect();switch(u.style.top="",u.style.left="",u.style.right="",u.style.bottom="",d){case"top":u.style.bottom=`${window.innerHeight-m.top+$i}px`,u.style.left=`${m.left+m.width/2}px`,u.style.transform="translateX(-50%)";break;case"left":u.style.right=`${window.innerWidth-m.left+$i}px`,u.style.top=`${m.top}px`;break;case"right":u.style.left=`${m.right+$i}px`,u.style.top=`${m.top}px`;break;case"bottom":default:u.style.top=`${m.bottom+$i}px`,u.style.left=`${m.left+m.width/2}px`,u.style.transform="translateX(-50%)"}}var oi=N({mount(i,u){let d=i,m=`sui-hover-card-${++Nm}`,n=d.triggerSlot(),b=d.panelEl();if(!b)return;b.id=m;let w=null,E=null;function M(){if(w)clearTimeout(w),w=null;if(E)clearTimeout(E),E=null}function y(){let g=n?.assignedElements({flatten:!0})??[];for(let j of g)j.setAttribute("aria-describedby",m)}function L(){M();let g=n?.assignedElements({flatten:!0})?.[0];if(g&&b)Am(g,b,d.placement()||"bottom");b?.showPopover?.()}function l(){M(),b?.hidePopover?.()}u.show=L,u.hide=l;let f=(g)=>{let T=g.newState==="open";if(T!==d.open())d.open.set(T);u.setAttribute("data-state",T?"open":"closed"),u.dispatchEvent(new CustomEvent(T?"sui-open":"sui-close",{bubbles:!0,composed:!0}))},t=()=>{M(),w=setTimeout(()=>L(),d.showDelay()??300)},r=()=>{M(),E=setTimeout(()=>l(),d.hideDelay()??100)},H=()=>{M(),L()},C=()=>{M(),E=setTimeout(()=>l(),d.hideDelay()??100)};b.addEventListener("toggle",f),n?.addEventListener("slotchange",y),y(),u.addEventListener("pointerenter",t),u.addEventListener("pointerleave",r),u.addEventListener("focusin",H),u.addEventListener("focusout",C);let z=U(()=>{u.setAttribute("data-state",d.open()?"open":"closed")});return()=>{M(),b.removeEventListener("toggle",f),n?.removeEventListener("slotchange",y),u.removeEventListener("pointerenter",t),u.removeEventListener("pointerleave",r),u.removeEventListener("focusin",H),u.removeEventListener("focusout",C),z()}}});var Y8=V("sui-hover-card",{props:{open:P.bool({default:!1,reflect:!0}),placement:P.union(["top","bottom","left","right"],{default:"bottom",reflect:!0}),"show-delay":P.number({default:300}),"hide-delay":P.number({default:100})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.open=u.open,n.placement=u.placement,n.showDelay=u.showDelay,n.hideDelay=u.hideDelay,n.panelEl=J(null),n.triggerSlot=J(null);let b=null;if(oi&&typeof oi.setup==="function"){if(i.scope=n,b=oi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");E.setAttribute("name","trigger"),A(n,"triggerSlot",E),w.appendChild(E);let M=document.createElement("div");M.setAttribute("class","panel"),M.setAttribute("part","panel"),M.setAttribute("popover","auto"),A(n,"panelEl",M);let y=document.createElement("slot");if(M.appendChild(y),w.appendChild(M),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var ei=N({mount(i,u){let d=i,m=()=>Array.from(u.querySelectorAll(":scope > sui-accordion-item")),n=()=>m().filter((L)=>!L.hasAttribute("disabled")),b=()=>{let L=d.value();return new Set(L?L.split(",").filter(Boolean):[])},w=()=>{let L=b();for(let l of m()){let f=l.getAttribute("value")||"";if(L.has(f))l.setAttribute("open","");else l.removeAttribute("open")}},E=U(()=>{d.value(),w()}),M=(L)=>{let l=L;l.stopPropagation();let f=l.detail?.value,t=b();if(d.type()==="single")d.value.set(t.has(f)?"":f);else{if(t.has(f))t.delete(f);else t.add(f);d.value.set(Array.from(t).join(","))}u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:d.value()},bubbles:!0,composed:!0}))},y=(L)=>{let l=L;l.stopPropagation();let f=n();if(!f.length)return;let t=l.target?.closest("sui-accordion-item"),r=t?f.indexOf(t):-1,H;switch(l.detail?.direction){case"next":H=f[(r+1)%f.length];break;case"prev":H=f[(r-1+f.length)%f.length];break;case"first":H=f[0];break;case"last":H=f[f.length-1];break}if(H)H.shadowRoot?.querySelector("button")?.focus()};return u.addEventListener("sui-accordion-item-toggle",M),u.addEventListener("sui-accordion-nav",y),()=>{E(),u.removeEventListener("sui-accordion-item-toggle",M),u.removeEventListener("sui-accordion-nav",y)}}});var O8=V("sui-accordion",{props:{type:P.string({default:"single",reflect:!0}),value:P.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host { display: block; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.type=u.type,n.value=u.value;let b=null;if(ei&&typeof ei.setup==="function"){if(i.scope=n,b=ei.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");if(w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var Fm=0,si=N({mount(i,u){let d=i,m=++Fm,n=`sui-acc-trigger-${m}`,b=`sui-acc-content-${m}`,w=U(()=>{let l=d.triggerBtn(),f=d.contentWrap();if(l)l.id=n,l.setAttribute("aria-controls",b);if(f)f.id=b,f.setAttribute("aria-labelledby",n)}),E=U(()=>{let l=d.triggerBtn();if(l)l.setAttribute("aria-expanded",d.open()?"true":"false")}),M=()=>{if(d.disabled())return;u.dispatchEvent(new CustomEvent("sui-accordion-item-toggle",{detail:{value:d.value()},bubbles:!0,composed:!0}))},y=(l)=>{if(l.key==="ArrowDown"||l.key==="ArrowUp")l.preventDefault(),u.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:l.key==="ArrowDown"?"next":"prev"},bubbles:!0,composed:!0}));else if(l.key==="Home"||l.key==="End")l.preventDefault(),u.dispatchEvent(new CustomEvent("sui-accordion-nav",{detail:{direction:l.key==="Home"?"first":"last"},bubbles:!0,composed:!0}))},L=U(()=>{let l=d.triggerBtn();if(!l)return;return l.addEventListener("click",M),l.addEventListener("keydown",y),()=>{l.removeEventListener("click",M),l.removeEventListener("keydown",y)}});return()=>{w(),E(),L()}}});var a8=V("sui-accordion-item",{props:{value:P.string({default:"",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),open:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.disabled=u.disabled,n.open=u.open,n.contentWrap=J(null),n.triggerBtn=J(null);let b=null;if(si&&typeof si.setup==="function"){if(i.scope=n,b=si.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("button");E.setAttribute("class","trigger"),E.setAttribute("part","trigger"),A(n,"triggerBtn",E),m.push(W(E,"disabled",()=>n.disabled()));let M=document.createElement("span");M.setAttribute("class","trigger-label");let y=document.createElement("slot");y.setAttribute("name","trigger"),M.appendChild(y),E.appendChild(M);let L=document.createElement("svg");L.setAttribute("class","chevron"),L.setAttribute("part","chevron"),L.setAttribute("viewbox","0 0 16 16"),L.setAttribute("fill","none"),L.setAttribute("stroke","currentColor"),L.setAttribute("stroke-width","1.75"),L.setAttribute("stroke-linecap","round"),L.setAttribute("stroke-linejoin","round"),L.setAttribute("aria-hidden","true");let l=document.createElement("path");l.setAttribute("d","M4 6l4 4 4-4"),L.appendChild(l),E.appendChild(L),w.appendChild(E);let f=document.createElement("div");f.setAttribute("class","content-wrap"),f.setAttribute("part","content"),A(n,"contentWrap",f),f.setAttribute("role","region");let t=document.createElement("div");t.setAttribute("class","content-inner");let r=document.createElement("div");r.setAttribute("class","content-body");let H=document.createElement("slot");if(r.appendChild(H),t.appendChild(r),f.appendChild(t),w.appendChild(f),b!=null){let C=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(n,{get(g,j){return g[j]},set(g,j,T){let D=g[j];if(D!=null&&typeof D.set==="function"){if(typeof T==="function")D.set(()=>T);else D.set(T);return!0}return!1}});if(C.mount){let g=C.mount(z,d);if(typeof g==="function")m.push(g)}if(C.unmount)m.push(()=>C.unmount(z))}}return n}});var Ld=`
sui-list-tile[variant='row'] + sui-list-tile[variant='row'] {
  border-top: 1px solid var(--border-subtle);
}
sui-list-tile[variant='row'][selected] + sui-list-tile[variant='row'],
sui-list-tile[variant='row'] + sui-list-tile[variant='row'][selected] {
  border-top: none;
}
`,ld=new WeakSet;function Om(i){if(ld.has(i))return;ld.add(i);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let d=new CSSStyleSheet;d.replaceSync(Ld),i.adoptedStyleSheets=[...i.adoptedStyleSheets,d];return}}catch{}let u=i.createElement("style");u.setAttribute("data-sui-list-tile-dividers",""),u.textContent=Ld,i.head.appendChild(u)}var pi=N({mount(i,u){let d=i;Om(u.ownerDocument);let m=()=>{if(d.navigable())return"link";if(d.interactive())return"button";return"listitem"},n=U(()=>{d.navigable(),d.interactive(),u.setAttribute("role",m())}),b=U(()=>{u.setAttribute("aria-selected",d.selected()?"true":"false")}),w=U(()=>{u.setAttribute("aria-disabled",d.disabled()?"true":"false")}),E=U(()=>{let f=d.tileEl();if(!f)return;let t=d.interactive(),r=d.disabled();if(t&&!r)f.setAttribute("tabindex","0");else f.removeAttribute("tabindex")}),M=()=>{if(d.disabled()||!d.interactive())return!1;let f=new CustomEvent("sui-select",{detail:{},bubbles:!0,composed:!0,cancelable:!0});return u.dispatchEvent(f),f.defaultPrevented},y=(f)=>{if(!d.interactive())return;if(M()&&d.navigable())f.preventDefault()},L=(f)=>{if(!d.interactive()||d.disabled())return;if(d.navigable()&&f.key==="Enter")return;if(f.key==="Enter"||f.key===" ")f.preventDefault(),M()},l=U(()=>{let f=d.tileEl();if(!f)return;return f.addEventListener("click",y),f.addEventListener("keydown",L),()=>{f.removeEventListener("click",y),f.removeEventListener("keydown",L)}});return()=>{n(),b(),w(),E(),l()}}});var o8=V("sui-list-tile",{props:{variant:P.string({default:"row",reflect:!0}),interactive:P.bool({default:!1,reflect:!0}),navigable:P.string({default:"",reflect:!0}),selected:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),density:P.string({default:"normal",reflect:!0}),tone:P.string({default:"default",reflect:!0}),target:P.string({default:"",reflect:!0}),rel:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.variant=u.variant,n.interactive=u.interactive,n.navigable=u.navigable,n.selected=u.selected,n.disabled=u.disabled,n.density=u.density,n.tone=u.tone,n.target=u.target,n.rel=u.rel,n.tileEl=J(null);let b=null;if(pi&&typeof pi.setup==="function"){if(i.scope=n,b=pi.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(y)=>I({anchor:y,test:()=>{if(n.navigable())return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),l=[],f=document.createElement("a");f.setAttribute("class","tile"),f.setAttribute("part","button"),A(n,"tileEl",f),l.push(W(f,"href",()=>n.navigable())),l.push(W(f,"target",()=>n.target()||null)),l.push(W(f,"rel",()=>n.rel()||null));let t=document.createElement("div");t.setAttribute("class","row"),t.setAttribute("part","row");let r=document.createElement("slot");r.setAttribute("name","leading"),t.appendChild(r);let H=document.createElement("span");H.setAttribute("class","primary"),H.setAttribute("part","primary");let C=document.createElement("slot");H.appendChild(C),t.appendChild(H);let z=document.createElement("span");z.setAttribute("part","trailing");let g=document.createElement("slot");g.setAttribute("name","trailing"),z.appendChild(g),t.appendChild(z),f.appendChild(t);let j=document.createElement("div");j.setAttribute("class","secondary"),j.setAttribute("part","secondary");let T=document.createElement("slot");return T.setAttribute("name","meta"),j.appendChild(T),f.appendChild(j),L.appendChild(f),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let D of l)try{D()}catch(Z){}}}},()=>{let L=document.createDocumentFragment(),l=[],f=document.createElement("div");f.setAttribute("class","tile"),f.setAttribute("part","button"),A(n,"tileEl",f);let t=document.createElement("div");t.setAttribute("class","row"),t.setAttribute("part","row");let r=document.createElement("slot");r.setAttribute("name","leading"),t.appendChild(r);let H=document.createElement("span");H.setAttribute("class","primary"),H.setAttribute("part","primary");let C=document.createElement("slot");H.appendChild(C),t.appendChild(H);let z=document.createElement("span");z.setAttribute("part","trailing");let g=document.createElement("slot");g.setAttribute("name","trailing"),z.appendChild(g),t.appendChild(z),f.appendChild(t);let j=document.createElement("div");j.setAttribute("class","secondary"),j.setAttribute("part","secondary");let T=document.createElement("slot");return T.setAttribute("name","meta"),j.appendChild(T),f.appendChild(j),L.appendChild(f),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let D of l)try{D()}catch(Z){}}}}]}),E=d.shadowRoot??d;Y(E);let M=document.createComment("if");if(E.appendChild(M),m.push(w(M)),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let L=new Proxy(n,{get(l,f){return l[f]},set(l,f,t){let r=l[f];if(r!=null&&typeof r.set==="function"){if(typeof t==="function")r.set(()=>t);else r.set(t);return!0}return!1}});if(y.mount){let l=y.mount(L,d);if(typeof l==="function")m.push(l)}if(y.unmount)m.push(()=>y.unmount(L))}}return n}});var ci=N({scope:(i,u)=>{let d=J(null),m=J(!1);return{caughtError:d,hasFallbackSlot:m,reset:()=>{if(d.set(null),u instanceof HTMLElement)u.removeAttribute("data-error-message")}}},mount(i,u){let d=u.querySelectorAll(':scope > template[slot="fallback"]');for(let m of Array.from(d)){let n=document.createElement("div");n.setAttribute("slot","fallback"),n.appendChild(m.content.cloneNode(!0)),m.replaceWith(n)}return i.hasFallbackSlot.set(!!u.querySelector(':scope > [slot="fallback"]')),e0(u,{catch(m,n){if(i.caughtError()!==null)return;let b=m instanceof Error?m:Error(String(m));u.setAttribute("data-error-message",b.message),i.caughtError.set(b),u.dispatchEvent(new CustomEvent("sui-error-boundary:caught",{bubbles:!0,composed:!0,detail:{error:b,source:n}}))},reset:i.reset}),u.reset=i.reset,()=>{s0(u),delete u.reset}}});var nE=V("sui-error-boundary",{props:{"fallback-text":P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.fallbackText=u.fallbackText;let b=null;if(ci&&typeof ci.setup==="function"){if(i.scope=n,b=ci.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>I({anchor:L,test:()=>{if(n.hasFallbackSlot())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("slot");return t.setAttribute("name","fallback"),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let r of f)try{r()}catch(H){}}}},()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("div");t.setAttribute("class","fallback"),t.setAttribute("part","fallback"),t.setAttribute("role","alert");let r=document.createElement("div");r.setAttribute("class","message"),r.setAttribute("part","message");let H=document.createTextNode("");r.appendChild(H),f.push(a(H,()=>n.fallbackText()||"Something went wrong")),t.appendChild(r);let C=document.createElement("button");C.setAttribute("class","retry"),C.setAttribute("part","retry"),C.setAttribute("type","button"),f.push(nn(C,"click",function(){let g=n.reset;return typeof g==="function"?g.bind(void 0):()=>{}}(),{modifiers:[]}));let z=document.createTextNode("Retry");return C.appendChild(z),t.appendChild(C),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let g of f)try{g()}catch(j){}}}}]}),E=(L)=>I({anchor:L,test:()=>{if(n.caughtError()===null)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("slot");return l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let r of f)try{r()}catch(H){}}}},()=>{let l=document.createDocumentFragment(),f=[],t=document.createComment("if");return l.appendChild(t),f.push(w(t)),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let r of f)try{r()}catch(H){}}}}]}),M=d.shadowRoot??d;Y(M);let y=document.createComment("if");if(M.appendChild(y),m.push(E(y)),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var _i=N({mount(i,u){let d=i,m=[],n=null,b=()=>{let f=d.target()||"body",t=document.querySelector(f);if(!t)return console.warn(`[sui-portal] target "${f}" not found in document`),null;return t},w=(f)=>{let t=d.portalSlot(),r=t?t.assignedNodes():Array.from(u.childNodes);for(let H of r)if(f.appendChild(H),!m.includes(H))m.push(H)},E=(f)=>{for(let t of m)f.appendChild(t)},M=()=>{for(let f of m)if(f.parentNode)f.parentNode.removeChild(f);m.length=0},y=U(()=>{let f=b();if(!f){n=null;return}if(n===null)w(f);else if(n!==f)E(f);n=f}),L=d.portalSlot(),l=()=>{if(!n||!L)return;for(let f of L.assignedNodes())if(!m.includes(f))n.appendChild(f),m.push(f)};if(L)L.addEventListener("slotchange",l);return()=>{if(y(),L)L.removeEventListener("slotchange",l);M()}}});var fE=V("sui-portal",{props:{target:P.string({default:"body"})},shadow:!0,styles:`
  :host { display: none; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.target=u.target,n.portalSlot=J(null);let b=null;if(_i&&typeof _i.setup==="function"){if(i.scope=n,b=_i.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");if(A(n,"portalSlot",E),w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var n0=N({mount(i,u){let d=i,m=!1,n=(M)=>{M.textContent=m?"Hide details":"Show details",M.setAttribute("aria-expanded",m?"true":"false")},b=U(()=>{let M=d.detailsSlot();if(!M)return;let y=()=>{if(M.assignedNodes({flatten:!0}).some((l)=>l.nodeType===Node.ELEMENT_NODE||l.nodeType===Node.TEXT_NODE&&(l.textContent??"").trim().length>0))u.setAttribute("data-has-details","");else{u.removeAttribute("data-has-details"),u.removeAttribute("data-details-open"),m=!1;let l=d.detailsToggleBtn();if(l)n(l)}};return M.addEventListener("slotchange",y),y(),()=>M.removeEventListener("slotchange",y)}),w=U(()=>{let M=d.detailsToggleBtn();if(!M)return;n(M);let y=()=>{if(m=!m,m)u.setAttribute("data-details-open","");else u.removeAttribute("data-details-open");n(M)};return M.addEventListener("click",y),()=>M.removeEventListener("click",y)}),E=U(()=>{let M=d.actionSlot();if(!M)return;let y=()=>{let L=M.assignedElements().find((l)=>l.getAttribute("slot")==="retry");if(L)requestAnimationFrame(()=>L.focus?.())};return M.addEventListener("slotchange",y),()=>M.removeEventListener("slotchange",y)});return()=>{b(),w(),E()}}});var ME=V("sui-error-state",{props:{title:P.string({default:""}),body:P.string({default:""}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:P.union(["default","inline"],{default:"default",reflect:!0}),icon:P.string({default:"alert-circle"})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.title=u.title,n.body=u.body,n.size=u.size,n.variant=u.variant,n.icon=u.icon,n.actionSlot=J(null),n.detailsPanel=J(null),n.detailsSlot=J(null),n.detailsToggleBtn=J(null);let b=null;if(n0&&typeof n0.setup==="function"){if(i.scope=n,b=n0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","box"),E.setAttribute("part","box"),E.setAttribute("role","alert");let M=document.createElement("div");M.setAttribute("class","icon"),M.setAttribute("part","icon"),M.setAttribute("aria-hidden","true");let y=document.createElement("sui-icon");m.push(W(y,"name",()=>n.icon())),y.setAttribute("size","md"),M.appendChild(y),E.appendChild(M);let L=document.createElement("div");L.setAttribute("class","copy");let l=document.createElement("div");l.setAttribute("class","title"),l.setAttribute("part","title");let f=document.createElement("slot");f.setAttribute("name","title");let t=document.createTextNode("");f.appendChild(t),m.push(a(t,()=>n.title()||"Something went wrong")),l.appendChild(f),L.appendChild(l);let r=document.createElement("div");r.setAttribute("class","desc"),r.setAttribute("part","desc");let H=document.createElement("slot"),C=document.createTextNode("");H.appendChild(C),m.push(a(C,()=>n.body())),r.appendChild(H),L.appendChild(r);let z=document.createElement("button");z.setAttribute("class","details-toggle"),A(n,"detailsToggleBtn",z),z.setAttribute("type","button"),L.appendChild(z);let g=document.createElement("div");g.setAttribute("class","details"),A(n,"detailsPanel",g);let j=document.createElement("slot");j.setAttribute("name","details"),A(n,"detailsSlot",j),g.appendChild(j),L.appendChild(g),E.appendChild(L);let T=document.createElement("div");T.setAttribute("class","actions"),T.setAttribute("part","actions");let D=document.createElement("slot");if(D.setAttribute("name","action"),A(n,"actionSlot",D),T.appendChild(D),E.appendChild(T),w.appendChild(E),b!=null){let Z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(Z){let X=new Proxy(n,{get(G,Q){return G[Q]},set(G,Q,q){let F=G[Q];if(F!=null&&typeof F.set==="function"){if(typeof q==="function")F.set(()=>q);else F.set(q);return!0}return!1}});if(Z.mount){let G=Z.mount(X,d);if(typeof G==="function")m.push(G)}if(Z.unmount)m.push(()=>Z.unmount(X))}}return n}});var i0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","progressbar");let m=U(()=>{let n=Math.max(1,Math.floor(d.total()||0)),b=Math.max(1,Math.min(n,Math.floor(d.current()||1))),w=d.label();if(u.setAttribute("aria-valuemin","1"),u.setAttribute("aria-valuemax",String(n)),u.setAttribute("aria-valuenow",String(b)),w)u.setAttribute("aria-label",w);else u.setAttribute("aria-label",`Step ${b} of ${n}`);let E=d.dotsEl();if(!E)return;let M=[];for(let y=1;y<=n;y++){let L=document.createElement("span");L.className="dot",L.setAttribute("part","dot"),L.setAttribute("data-state",y===b?"active":y<b?"done":"pending"),L.setAttribute("aria-hidden","true"),M.push(L)}E.replaceChildren(...M)});return()=>{m()}}});var gE=V("sui-stepper",{props:{total:P.number({default:4,reflect:!0}),current:P.number({default:1,reflect:!0}),tone:P.string({default:"",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.total=u.total,n.current=u.current,n.tone=u.tone,n.size=u.size,n.label=u.label,n.dotsEl=J(null);let b=null;if(i0&&typeof i0.setup==="function"){if(i.scope=n,b=i0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");if(E.setAttribute("class","dots"),A(n,"dotsEl",E),w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var Km=new Set(["done","failed","pending","running"]);function Bm(i){return i.split(",").map((u)=>u.trim().toLowerCase()).filter((u)=>u.length>0).map((u)=>Km.has(u)?u:"pending")}var u0=N({scope:()=>({segments:J([]),ariaLabel:J(null)}),mount(i,u){let d=U(()=>{let m=Bm(i.values()||"");i.segments.set(m.map((n,b)=>({id:`s${b}`,state:n}))),i.ariaLabel.set(m.length===0?null:`${m.length} step outcome${m.length===1?"":"s"}`)});return()=>d()}});var JE=V("sui-step-segments",{props:{values:P.string({default:"",reflect:!0}),size:P.union(["sm","md"],{default:"sm",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.values=u.values,n.size=u.size;let b=null;if(u0&&typeof u0.setup==="function"){if(i.scope=n,b=u0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>h({anchor:L,items:()=>n.segments(),key:(l,f)=>{let t=Object.create(n);return t.seg=l,t.__i=f,((r)=>l.id)(t)},render:(l,f)=>{let t=Object.create(n);return t.seg=l,t.__i=f,t.__v=J(0),((r)=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("span");z.setAttribute("class","seg"),z.setAttribute("part","seg");{let g=r.seg.state;if(g==null||g===!1||g==="")z.removeAttribute("data-state");else z.setAttribute("data-state",g===!0?"":String(g))}return H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let g of C)try{g()}catch(j){}},update:(g,j)=>{r.seg=g,r.__i=j;{let T=r.seg.state;if(T==null||T===!1||T==="")z.removeAttribute("data-state");else z.setAttribute("data-state",T===!0?"":String(T))}r.__v.set((T)=>T+1)}}})(t)}}),E=d.shadowRoot??d;Y(E);let M=document.createElement("span");M.setAttribute("class","row"),M.setAttribute("part","row"),m.push(W(M,"role",()=>n.segments().length>0?"img":null)),m.push(W(M,"aria-label",()=>n.ariaLabel()));let y=document.createComment("for");if(M.appendChild(y),m.push(w(y)),E.appendChild(M),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var d0=N({mount(i,u){if(!u.hasAttribute("role"))u.setAttribute("role","list")}});var YE=V("sui-timeline",{shadow:!0,styles:`
  :host { display: block; position: relative; }
  .list { display: flex; flex-direction: column; gap: 4px; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null),b=null;if(d0&&typeof d0.setup==="function"){if(i.scope=n,b=d0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","list"),E.setAttribute("part","list");let M=document.createElement("slot");if(E.appendChild(M),w.appendChild(E),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let L=new Proxy(n,{get(l,f){return l[f]},set(l,f,t){let r=l[f];if(r!=null&&typeof r.set==="function"){if(typeof t==="function")r.set(()=>t);else r.set(t);return!0}return!1}});if(y.mount){let l=y.mount(L,d);if(typeof l==="function")m.push(l)}if(y.unmount)m.push(()=>y.unmount(L))}}return n}});var m0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","listitem");let m=U(()=>{if(d.interactive()){if(u.setAttribute("role","button"),!u.hasAttribute("tabindex"))u.setAttribute("tabindex","0")}else u.setAttribute("role","listitem"),u.removeAttribute("tabindex")}),n=()=>{u.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},b=()=>{if(d.interactive())n()},w=(E)=>{if(!d.interactive())return;if(E.key==="Enter"||E.key===" ")E.preventDefault(),n()};return u.addEventListener("click",b),u.addEventListener("keydown",w),()=>{m(),u.removeEventListener("click",b),u.removeEventListener("keydown",w)}}});var OE=V("sui-timeline-item",{props:{tone:P.string({default:"default",reflect:!0}),at:P.string({default:"",reflect:!0}),interactive:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.tone=u.tone,n.at=u.at,n.interactive=u.interactive;let b=null;if(m0&&typeof m0.setup==="function"){if(i.scope=n,b=m0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(C)=>I({anchor:C,test:()=>{if(n.at())return 0;return-1},branches:[()=>{let z=document.createDocumentFragment(),g=[],j=document.createElement("span");j.setAttribute("class","at"),j.setAttribute("part","at");let T=document.createTextNode("");return j.appendChild(T),g.push(a(T,()=>n.at())),z.appendChild(j),{nodes:Array.from(z.childNodes),cleanup:()=>{for(let D of g)try{D()}catch(Z){}}}}]}),E=d.shadowRoot??d;Y(E);let M=document.createElement("div");M.setAttribute("class","gutter");let y=document.createElement("span");y.setAttribute("class","dot"),y.setAttribute("part","dot"),M.appendChild(y),E.appendChild(M);let L=document.createElement("div");L.setAttribute("class","body"),L.setAttribute("part","body");let l=document.createElement("div");l.setAttribute("class","head");let f=document.createElement("slot");f.setAttribute("name","title"),l.appendChild(f);let t=document.createComment("if");l.appendChild(t),m.push(w(t)),L.appendChild(l);let r=document.createElement("div");r.setAttribute("class","content");let H=document.createElement("slot");if(r.appendChild(H),L.appendChild(r),E.appendChild(L),b!=null){let C=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(n,{get(g,j){return g[j]},set(g,j,T){let D=g[j];if(D!=null&&typeof D.set==="function"){if(typeof T==="function")D.set(()=>T);else D.set(T);return!0}return!1}});if(C.mount){let g=C.mount(z,d);if(typeof g==="function")m.push(g)}if(C.unmount)m.push(()=>C.unmount(z))}}return n}});var Im=["Waking daemon","Warming routes","Fetching state","Hydrating shell"];function Ed(i){let u=i.trim();if(!u)return Im;return u.split("|").map((d)=>d.trim()).filter(Boolean)}var b0=N({mount(i,u){let d=i,m=d.msgEl();if(!m)return;let n=0;(()=>{let E=Ed(d.messages());m.textContent=E[n]??"",m.style.opacity="1"})();let w=setInterval(()=>{m.style.opacity="0",requestAnimationFrame(()=>{let E=Ed(d.messages());n=(n+1)%E.length,m.textContent=E[n]??"",m.style.opacity="1"})},1400);return()=>clearInterval(w)}});var aE=V("sui-loading",{props:{messages:P.string({default:""}),compact:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.messages=u.messages,n.compact=u.compact,n.msgEl=J(null);let b=null;if(b0&&typeof b0.setup==="function"){if(i.scope=n,b=b0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","stage"),E.setAttribute("part","stage"),E.setAttribute("aria-hidden","true");let M=document.createElement("div");M.setAttribute("class","rocket");let y=document.createElement("svg");y.setAttribute("viewbox","0 0 32 40"),y.setAttribute("fill","none"),y.setAttribute("xmlns","http://www.w3.org/2000/svg");let L=document.createElement("path");L.setAttribute("d","M16 2 C20 6 22 11 22 16 H10 C10 11 12 6 16 2 Z"),L.setAttribute("fill","currentColor"),y.appendChild(L);let l=document.createElement("rect");l.setAttribute("x","10"),l.setAttribute("y","16"),l.setAttribute("width","12"),l.setAttribute("height","14"),l.setAttribute("rx","2"),l.setAttribute("fill","currentColor"),l.setAttribute("opacity","0.9"),y.appendChild(l);let f=document.createElement("circle");f.setAttribute("cx","16"),f.setAttribute("cy","20"),f.setAttribute("r","2.5"),f.setAttribute("fill","var(--background, #fff)"),f.setAttribute("opacity","0.85"),y.appendChild(f);let t=document.createElement("path");t.setAttribute("d","M10 24 L4 32 L10 30 Z"),t.setAttribute("fill","currentColor"),t.setAttribute("opacity","0.75"),y.appendChild(t);let r=document.createElement("path");r.setAttribute("d","M22 24 L28 32 L22 30 Z"),r.setAttribute("fill","currentColor"),r.setAttribute("opacity","0.75"),y.appendChild(r);let H=document.createElement("rect");H.setAttribute("x","13"),H.setAttribute("y","29"),H.setAttribute("width","6"),H.setAttribute("height","2"),H.setAttribute("rx","0.5"),H.setAttribute("fill","var(--background, #000)"),H.setAttribute("opacity","0.5"),y.appendChild(H),M.appendChild(y),E.appendChild(M);let C=document.createElement("div");C.setAttribute("class","flame"),E.appendChild(C);let z=document.createElement("div");z.setAttribute("class","particle p1"),E.appendChild(z);let g=document.createElement("div");g.setAttribute("class","particle p2"),E.appendChild(g);let j=document.createElement("div");j.setAttribute("class","particle p3"),E.appendChild(j),w.appendChild(E);let T=document.createElement("div");if(T.setAttribute("class","message"),T.setAttribute("part","message"),A(n,"msgEl",T),w.appendChild(T),b!=null){let D=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(D){let Z=new Proxy(n,{get(X,G){return X[G]},set(X,G,Q){let q=X[G];if(q!=null&&typeof q.set==="function"){if(typeof Q==="function")q.set(()=>Q);else q.set(Q);return!0}return!1}});if(D.mount){let X=D.mount(Z,d);if(typeof X==="function")m.push(X)}if(D.unmount)m.push(()=>D.unmount(Z))}}return n}});function Md(i){let u=document.getElementById("ship-view-root");if(!u)return null;let d=i.target;if(!d||!u.contains(d))return null;if(d.closest("sui-edit-overlay"))return null;return d}function yd(i){let u=i.tagName.toLowerCase(),d=i.id||null,m=u;if(d)m=`#${d}`;else if(i.classList.length>0)m=`${u}.${Array.from(i.classList).slice(0,2).join(".")}`;return{tag:u,selector:m,id:d}}var f0=N({mount(i,u){let d=i,m=d.boxEl(),n=d.labelEl(),b=null;function w(r){if(!m||!n)return;if(!r){m.classList.remove("visible"),n.classList.remove("visible");return}m.style.cssText=`top:${r.top}px;left:${r.left}px;width:${r.width}px;height:${r.height}px`,m.classList.add("visible"),n.style.cssText=`top:${r.top}px;left:${r.left}px`,n.textContent=`<${r.tag}${r.id?`#${r.id}`:""}>`,n.classList.add("visible")}let E=(r)=>{if(b)return;b=requestAnimationFrame(()=>{b=null;let H=Md(r);if(!H){w(null);return}let C=H.getBoundingClientRect(),{tag:z,id:g}=yd(H);w({top:C.top,left:C.left,width:C.width,height:C.height,tag:z,id:g})})},M=()=>{w(null)},y=(r)=>{let H=Md(r);if(!H)return;r.preventDefault(),r.stopPropagation();let{tag:C,selector:z,id:g}=yd(H);(u.closest("sui-shell")??u.closest("ship-shell"))?.dispatchEvent(new CustomEvent("sui:chat:annotate",{detail:{tag:C,selector:z,id:g,preview:(H.textContent??"").trim().slice(0,60)},bubbles:!0})),window.ship?.chat?.open?.()};function L(){let r=document.getElementById("ship-view-root");if(!r)return;r.addEventListener("pointermove",E,!0),r.addEventListener("pointerleave",M,!0),r.addEventListener("click",y,!0)}function l(){let r=document.getElementById("ship-view-root");if(!r)return;r.removeEventListener("pointermove",E,!0),r.removeEventListener("pointerleave",M,!0),r.removeEventListener("click",y,!0)}function f(){let r=document.body.classList.contains("edit-mode");if(u.classList.toggle("active",r),r)L();else if(l(),w(null),b)cancelAnimationFrame(b),b=null}f();let t=new MutationObserver(()=>f());return t.observe(document.body,{attributes:!0,attributeFilter:["class"]}),()=>{if(t.disconnect(),l(),b)cancelAnimationFrame(b),b=null}}});var oE=V("sui-edit-overlay",{shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.boxEl=J(null),n.labelEl=J(null);let b=null;if(f0&&typeof f0.setup==="function"){if(i.scope=n,b=f0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","box"),E.setAttribute("part","box"),A(n,"boxEl",E),w.appendChild(E);let M=document.createElement("div");M.setAttribute("class","label"),M.setAttribute("part","label"),A(n,"labelEl",M),w.appendChild(M);let y=document.createElement("div");y.setAttribute("class","hint"),y.setAttribute("part","hint");let L=document.createTextNode(`
  Edit mode — hover an element, click to annotate.
  `);y.appendChild(L);let l=document.createElement("kbd"),f=document.createTextNode("⌘⇧E");l.appendChild(f),y.appendChild(l);let t=document.createTextNode(` to exit.
`);if(y.appendChild(t),w.appendChild(y),b!=null){let r=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(r){let H=new Proxy(n,{get(C,z){return C[z]},set(C,z,g){let j=C[z];if(j!=null&&typeof j.set==="function"){if(typeof g==="function")j.set(()=>g);else j.set(g);return!0}return!1}});if(r.mount){let C=r.mount(H,d);if(typeof C==="function")m.push(C)}if(r.unmount)m.push(()=>r.unmount(H))}}return n}});function Rm(i=Date.now()){let u=i,d=[,,,,,,,,,,];for(let b=9;b>=0;b--)d[b]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[u%32],u=Math.floor(u/32);let m=Array(16),n=typeof globalThis<"u"?globalThis.crypto:void 0;if(n?.getRandomValues){let b=new Uint8Array(16);n.getRandomValues(b);for(let w=0;w<16;w++)m[w]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[b[w]%32]}else for(let b=0;b<16;b++)m[b]="0123456789ABCDEFGHJKMNPQRSTVWXYZ"[Math.floor(Math.random()*32)];return d.join("")+m.join("")}function wd(i){let u={};if(i.viewId)u.view_id=i.viewId;if(i.selector)u.selector=i.selector;if(i.rect)u.rect=i.rect;return{id:Rm(),kind:i.kind,target:u,note:i.note,created_at:new Date().toISOString(),author_kind:"human"}}var km="/api/feedback",t0=N({scope:(i,u)=>{let{viewId:d,mode:m,endpoint:n,staging:b}=i,w=J([]),E=J(null),M=J(null),y=J(""),L={},l=s(()=>{let D=M();if(!D)return!1;return Math.abs(D.curX-D.startX)>4||Math.abs(D.curY-D.startY)>4}),f=s(()=>{let D=M();if(!D)return"";let Z=Math.min(D.startX,D.curX),X=Math.min(D.startY,D.curY),G=Math.abs(D.curX-D.startX),Q=Math.abs(D.curY-D.startY);return`left:${Z}px;top:${X}px;width:${G}px;height:${Q}px`}),t=s(()=>{let D=E();if(!D)return"";let Z=Math.min(D.x+12,1e4),X=Math.min(D.y+12,1e4);return`left:${Z}px;top:${X}px`}),r=s(()=>w().map((D,Z)=>({id:D.id,kind:D.kind,index:Z,rect:D.kind==="region"&&D.target.rect?D.target.rect.join(","):"",pos:L[D.id]??{x:0,y:0}})));async function H(D){let Z=n()||km;try{let X=await fetch(Z,{method:"POST",credentials:"include",headers:{"Content-Type":"application/json"},body:JSON.stringify(D)});if(!X.ok){u.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:`http_${X.status}`,id:D.id},bubbles:!0,composed:!0}));return}w.set(w().filter((G)=>G.id!==D.id)),u.dispatchEvent(new CustomEvent("sui-annotation-sent",{detail:{id:D.id,status:X.status},bubbles:!0,composed:!0}))}catch(X){u.dispatchEvent(new CustomEvent("sui-annotation-error",{detail:{code:"network_error",message:String(X),id:D.id},bubbles:!0,composed:!0}))}}function C(D){let Z=E();if(!Z)return;let X=D.trim();if(!X){E.set(null),y.set("");return}let G=wd({viewId:d()||void 0,kind:Z.kind,selector:Z.kind==="point"?Z.selector:void 0,rect:Z.kind==="region"?Z.rect:void 0,note:X});if(L[G.id]={x:Z.x,y:Z.y},w.set([...w(),G]),u.dispatchEvent(new CustomEvent("sui-annotation-staged",{detail:G,bubbles:!0,composed:!0})),E.set(null),y.set(""),!b())H(G)}let z=()=>{C(y())},g=()=>{E.set(null),y.set("")},j=(D)=>{if(D.key==="Escape")g();else if(D.key==="Enter"&&(D.metaKey||D.ctrlKey)){let Z=D.currentTarget;C(Z.value)}},T=e(null);return{staged:w,pending:E,drag:M,noteText:y,dragVisible:l,dragRectStyle:f,popoverStyle:t,markers:r,overlayEl:T,confirmNote:C,sendNote:z,cancelPending:g,onNoteKeydown:j,sendOne:H}},mount(i,u){let d=i,m=null,n=null,b=(L)=>{let l=d.overlayEl();if(!l)return{x:0,y:0};let f=l.getBoundingClientRect();return{x:L.clientX-f.left,y:L.clientY-f.top}},w=(L)=>{if(d.mode()==="off")return;let f=L.target;if(f.closest("[data-annotation-popover]")||f.closest("sui-annotation-marker"))return;if(d.pending()){d.pending.set(null),d.noteText.set("");return}L.preventDefault();let t=b(L);m={startX:t.x,startY:t.y,curX:t.x,curY:t.y},d.drag.set(m),L.target.setPointerCapture?.(L.pointerId)},E=(L)=>{if(!m)return;let l=b(L);if(m={...m,curX:l.x,curY:l.y},n!==null)return;n=requestAnimationFrame(()=>{if(n=null,m)d.drag.set({...m})})},M=(L)=>{if(!m)return;let l=b(L),f=Math.abs(l.x-m.startX),t=Math.abs(l.y-m.startY),r=d.mode(),H=null;if(r==="point"||r==="region"&&f<8&&t<8)H={kind:"point",x:l.x,y:l.y,selector:"body"};else if(r==="region"){let C=Math.min(m.startX,l.x),z=Math.min(m.startY,l.y),g=[Math.round(C),Math.round(z),Math.round(f),Math.round(t)];H={kind:"region",x:C+f/2,y:z+t/2,rect:g}}if(m=null,d.drag.set(null),d.noteText.set(""),H)d.pending.set(H)},y=d.overlayEl();if(y)y.addEventListener("pointerdown",w),y.addEventListener("pointermove",E),y.addEventListener("pointerup",M);return Object.defineProperty(u,"staged",{configurable:!0,get:()=>d.staged()}),u.send=async()=>{for(let L of[...d.staged()])await d.sendOne(L)},u.clear=()=>{d.staged.set([])},()=>{if(n!==null)cancelAnimationFrame(n),n=null;if(y)y.removeEventListener("pointerdown",w),y.removeEventListener("pointermove",E),y.removeEventListener("pointerup",M)}}});var u9=V("sui-annotation-layer",{props:{"view-id":P.string({default:""}),mode:P.union(["off","point","region"],{default:"off",reflect:!0}),endpoint:P.string({default:"/api/feedback"}),staging:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.viewId=u.viewId,n.mode=u.mode,n.endpoint=u.endpoint,n.staging=u.staging,n.overlayEl=J(null);let b=null;if(t0&&typeof t0.setup==="function"){if(i.scope=n,b=t0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(C)=>I({anchor:C,test:()=>{if(n.dragVisible())return 0;return-1},branches:[()=>{let z=document.createDocumentFragment(),g=[],j=document.createElement("div");return j.setAttribute("class","drag-rect"),g.push(W(j,"style",()=>n.dragRectStyle())),z.appendChild(j),{nodes:Array.from(z.childNodes),cleanup:()=>{for(let T of g)try{T()}catch(D){}}}}]}),E=(C)=>h({anchor:C,items:()=>n.markers(),key:(z,g)=>{let j=Object.create(n);return j.m=z,j.__i=g,((T)=>z.id)(j)},render:(z,g)=>{let j=Object.create(n);return j.m=z,j.__i=g,j.__v=J(0),((T)=>{let D=document.createDocumentFragment(),Z=[],X=document.createElement("sui-annotation-marker");return X.kind=T.m.kind,X.index=T.m.index,X.rect=T.m.rect,X.x=T.m.pos.x,X.y=T.m.pos.y,D.appendChild(X),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let G of Z)try{G()}catch(Q){}},update:(G,Q)=>{T.m=G,T.__i=Q,X.kind=T.m.kind,X.index=T.m.index,X.rect=T.m.rect,X.x=T.m.pos.x,X.y=T.m.pos.y,T.__v.set((q)=>q+1)}}})(j)}}),M=(C)=>I({anchor:C,test:()=>{if(n.pending())return 0;return-1},branches:[()=>{let z=document.createDocumentFragment(),g=[],j=document.createElement("div");j.setAttribute("class","popover"),j.setAttribute("data-annotation-popover",""),g.push(W(j,"style",()=>n.popoverStyle())),g.push(nn(j,"pointerdown",function(){let q=(F)=>F.stopPropagation();return typeof q==="function"?q.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("textarea");T.setAttribute("part","note-input"),T.setAttribute("placeholder","Add a note…"),g.push(Pn(T,n.noteText)),g.push(nn(T,"keydown",function(){let q=n.onNoteKeydown;return typeof q==="function"?q.bind(void 0):()=>{}}(),{modifiers:[]})),j.appendChild(T);let D=document.createElement("div");D.setAttribute("class","popover-actions");let Z=document.createElement("button");Z.setAttribute("type","button"),g.push(nn(Z,"click",function(){let q=n.cancelPending;return typeof q==="function"?q.bind(void 0):()=>{}}(),{modifiers:[]}));let X=document.createTextNode("Cancel");Z.appendChild(X),D.appendChild(Z);let G=document.createElement("button");G.setAttribute("type","button"),G.setAttribute("data-primary",""),g.push(nn(G,"click",function(){let q=n.sendNote;return typeof q==="function"?q.bind(void 0):()=>{}}(),{modifiers:[]}));let Q=document.createTextNode("Send");return G.appendChild(Q),D.appendChild(G),j.appendChild(D),z.appendChild(j),{nodes:Array.from(z.childNodes),cleanup:()=>{for(let q of g)try{q()}catch(F){}}}}]}),y=d.shadowRoot??d;Y(y);let L=document.createElement("div");L.setAttribute("class","surface"),L.setAttribute("part","surface");let l=document.createElement("slot");L.appendChild(l);let f=document.createElement("div");f.setAttribute("class","overlay"),f.setAttribute("part","overlay"),A(n,"overlayEl",f);let t=document.createComment("if");f.appendChild(t),m.push(w(t));let r=document.createComment("for");f.appendChild(r),m.push(E(r));let H=document.createComment("if");if(f.appendChild(H),m.push(M(H)),L.appendChild(f),y.appendChild(L),b!=null){let C=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(C){let z=new Proxy(n,{get(g,j){return g[j]},set(g,j,T){let D=g[j];if(D!=null&&typeof D.set==="function"){if(typeof T==="function")D.set(()=>T);else D.set(T);return!0}return!1}});if(C.mount){let g=C.mount(z,d);if(typeof g==="function")m.push(g)}if(C.unmount)m.push(()=>C.unmount(z))}}return n}});var r0=N({mount(i,u){let d=i;return U(()=>{let n=d.pinEl(),b=d.boxEl(),w=d.x(),E=d.y(),M=d.index(),y=d.rect();if(n)n.style.left=`${w}px`,n.style.top=`${E}px`,n.textContent=String(M+1);if(b&&y){let L=y.split(",").map(Number);if(L.length===4&&L.every((l)=>Number.isFinite(l))){let[l,f,t,r]=L;b.style.cssText=`left:${l}px;top:${f}px;width:${t}px;height:${r}px`}}})}});var r9=V("sui-annotation-marker",{props:{kind:P.union(["point","region"],{default:"point",reflect:!0}),x:P.number({default:0}),y:P.number({default:0}),rect:P.string({default:""}),active:P.bool({default:!1,reflect:!0}),index:P.number({default:0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.kind=u.kind,n.x=u.x,n.y=u.y,n.rect=u.rect,n.active=u.active,n.index=u.index,n.boxEl=J(null),n.pinEl=J(null);let b=null;if(r0&&typeof r0.setup==="function"){if(i.scope=n,b=r0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("part","pin"),E.setAttribute("role","button"),E.setAttribute("tabindex","0"),A(n,"pinEl",E),w.appendChild(E);let M=document.createElement("div");if(M.setAttribute("part","box"),M.setAttribute("role","button"),M.setAttribute("tabindex","0"),A(n,"boxEl",M),w.appendChild(M),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let L=new Proxy(n,{get(l,f){return l[f]},set(l,f,t){let r=l[f];if(r!=null&&typeof r.set==="function"){if(typeof t==="function")r.set(()=>t);else r.set(t);return!0}return!1}});if(y.mount){let l=y.mount(L,d);if(typeof l==="function")m.push(l)}if(y.unmount)m.push(()=>y.unmount(L))}}return n}});var L0=N({scope:(i,u)=>{return{staged:J([]),emitRemove:(n)=>{u.dispatchEvent(new CustomEvent("sui-annotation-remove",{detail:{id:n},bubbles:!0,composed:!0}))}}},mount(i,u){let d=u.staged;if(d!=null)i.staged.set(Array.isArray(d)?d:[]);Object.defineProperty(u,"staged",{configurable:!0,get:()=>i.staged(),set:(m)=>i.staged.set(Array.isArray(m)?m:[])})}});var w9=V("sui-annotation-list",{shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null),b=null;if(L0&&typeof L0.setup==="function"){if(i.scope=n,b=L0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>h({anchor:L,items:()=>n.staged(),key:(l,f)=>{let t=Object.create(n);return t.a=l,t.__i=f,((r)=>l.id)(t)},render:(l,f)=>{let t=Object.create(n);return t.a=l,t.__i=f,t.__v=J(0),((r)=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("li");z.setAttribute("part","item");{let G=r.a.id;if(G==null||G===!1||G==="")z.removeAttribute("data-id");else z.setAttribute("data-id",G===!0?"":String(G))}let g=document.createElement("span");g.setAttribute("class","kind");let j=document.createTextNode("");g.appendChild(j),j.textContent=r.a.kind,z.appendChild(g);let T=document.createElement("span");T.setAttribute("class","note"),C.push(W(T,"title",()=>(r.__v(),r.a.note??"")));let D=document.createTextNode("");T.appendChild(D),C.push(a(D,()=>(r.__v(),r.a.note??"(no note)"))),z.appendChild(T);let Z=document.createElement("button");Z.setAttribute("type","button"),Z.setAttribute("aria-label","Remove annotation"),C.push(nn(Z,"click",function(){let G=()=>r.emitRemove(r.a.id);return typeof G==="function"?G.bind(void 0):()=>{}}(),{modifiers:[]}));let X=document.createTextNode("Remove");return Z.appendChild(X),z.appendChild(Z),H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let G of C)try{G()}catch(Q){}},update:(G,Q)=>{r.a=G,r.__i=Q;{let q=r.a.id;if(q==null||q===!1||q==="")z.removeAttribute("data-id");else z.setAttribute("data-id",q===!0?"":String(q))}j.textContent=r.a.kind,r.__v.set((q)=>q+1)}}})(t)}}),E=(L)=>I({anchor:L,test:()=>{if(n.staged().length===0)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("div");t.setAttribute("class","empty"),t.setAttribute("part","empty");let r=document.createTextNode("No staged annotations.");return t.appendChild(r),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let H of f)try{H()}catch(C){}}}},()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("ol");t.setAttribute("part","list");let r=document.createComment("for");return t.appendChild(r),f.push(w(r)),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let H of f)try{H()}catch(C){}}}}]}),M=d.shadowRoot??d;Y(M);let y=document.createComment("if");if(M.appendChild(y),m.push(E(y)),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var l0=N({mount(i,u){let d=i,m=U(()=>{let n=d.inputEl();if(!n)return;let b=()=>{if(d.disabled())return;u.dispatchEvent(new CustomEvent("sui-radio-select",{detail:{value:d.value()},bubbles:!0,composed:!0}))},w=(E)=>{if(!["ArrowDown","ArrowRight","ArrowUp","ArrowLeft"].includes(E.key))return;E.preventDefault();let y=E.key==="ArrowDown"||E.key==="ArrowRight";u.dispatchEvent(new CustomEvent("sui-radio-nav",{detail:{direction:y?"next":"prev"},bubbles:!0,composed:!0}))};return n.addEventListener("click",b),n.addEventListener("keydown",w),()=>{n.removeEventListener("click",b),n.removeEventListener("keydown",w)}});return u.focusInput=()=>{u.shadowRoot?.querySelector("input")?.focus()},()=>m()}});var j9=V("sui-radio",{props:{value:P.string({default:"",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),checked:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.disabled=u.disabled,n.checked=u.checked,n.tone=u.tone,n.size=u.size,n.inputEl=J(null);let b=null;if(l0&&typeof l0.setup==="function"){if(i.scope=n,b=l0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("label");E.setAttribute("part","label");let M=document.createElement("span");M.setAttribute("class","circle"),M.setAttribute("part","circle");let y=document.createElement("input");y.setAttribute("part","input"),y.setAttribute("type","radio"),m.push(jn(y,n.checked)),m.push(W(y,"disabled",()=>n.disabled())),m.push(W(y,"aria-checked",()=>n.checked()?"true":"false")),m.push(W(y,"tabindex",()=>n.checked()?"0":"-1")),A(n,"inputEl",y),M.appendChild(y);let L=document.createElement("span");L.setAttribute("class","dot"),L.setAttribute("part","dot"),L.setAttribute("aria-hidden","true"),M.appendChild(L),E.appendChild(M);let l=document.createElement("slot");if(E.appendChild(l),w.appendChild(E),b!=null){let f=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(f){let t=new Proxy(n,{get(r,H){return r[H]},set(r,H,C){let z=r[H];if(z!=null&&typeof z.set==="function"){if(typeof C==="function")z.set(()=>C);else z.set(C);return!0}return!1}});if(f.mount){let r=f.mount(t,d);if(typeof r==="function")m.push(r)}if(f.unmount)m.push(()=>f.unmount(t))}}return n}});var E0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","radiogroup");let m=document.createElement("input");m.type="hidden",u.appendChild(m);let n=()=>Array.from(u.querySelectorAll(":scope > sui-radio")),b=()=>n().filter((l)=>!l.hasAttribute("disabled")&&!d.disabled()),w=()=>{let l=d.value(),f=d.disabled(),t=d.tone(),r=d.size();for(let H of n()){if(H.toggleAttribute("checked",H.getAttribute("value")===l),f)H.setAttribute("disabled","");if(t&&!H.hasAttribute("tone"))H.setAttribute("tone",t);if(r&&!H.hasAttribute("size"))H.setAttribute("size",r)}m.name=d.name(),m.value=l},E=U(()=>{d.value(),d.disabled(),d.tone(),d.size(),w()}),M=U(()=>{let l=d.label();if(l)u.setAttribute("aria-label",l);else u.removeAttribute("aria-label")}),y=(l)=>{let f=l;f.stopPropagation();let t=f.detail.value;if(t===d.value())return;d.value.set(t),w(),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:t},bubbles:!0,composed:!0}))},L=(l)=>{let f=l;f.stopPropagation();let t=b();if(!t.length)return;let r=f.composedPath().find((g)=>g.tagName?.toLowerCase()==="sui-radio"),H=r?t.indexOf(r):-1,C=f.detail.direction==="next"?t[(H+1)%t.length]:t[(H-1+t.length)%t.length],z=C.getAttribute("value")??"";d.value.set(z),w(),C.focusInput?.(),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:z},bubbles:!0,composed:!0}))};return u.addEventListener("sui-radio-select",y),u.addEventListener("sui-radio-nav",L),()=>{E(),M(),u.removeEventListener("sui-radio-select",y),u.removeEventListener("sui-radio-nav",L),m.remove()}}});var G9=V("sui-radio-group",{props:{value:P.string({default:"",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),name:P.string({default:""}),label:P.string({default:""})},shadow:!0,styles:`
  :host {
    display: flex;
    flex-direction: column;
    gap: var(--space-2, 8px);
    font-family: var(--font-body);
    color: var(--foreground);
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.disabled=u.disabled,n.tone=u.tone,n.size=u.size,n.name=u.name,n.label=u.label;let b=null;if(E0&&typeof E0.setup==="function"){if(i.scope=n,b=E0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");if(w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var M0=N({mount(i,u){let d=i,m=document.createElement("input");m.type="hidden",u.appendChild(m);let n=(t)=>{let r=d.step(),H=d.min(),C=d.max(),z=Math.round((t-H)/r)*r+H;return Math.max(H,Math.min(C,z))},b=(t)=>{let r=n(t);if(r===d.value())return;d.value.set(r),m.name=d.name(),m.value=String(r),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:r},bubbles:!0,composed:!0}))},w=(t)=>{let r=d.trackEl();if(!r)return d.value();let H=r.getBoundingClientRect(),C=Math.max(0,Math.min(1,(t-H.left)/H.width));return d.min()+C*(d.max()-d.min())},E=U(()=>{let t=d.value(),r=d.min(),C=d.max()-r||1,z=(t-r)/C*100,g=d.fillEl(),j=d.thumbEl();if(g)g.style.width=`${z}%`;if(j)j.style.left=`${z}%`;m.name=d.name(),m.value=String(t)}),M=!1,y=(t)=>{if(d.disabled())return;t.preventDefault(),M=!0,b(w(t.clientX)),d.thumbEl()?.focus();let H=(z)=>{if(M)b(w(z.clientX))},C=()=>{M=!1,document.removeEventListener("pointermove",H),document.removeEventListener("pointerup",C)};document.addEventListener("pointermove",H),document.addEventListener("pointerup",C)},L=(t)=>{if(d.disabled())return;let r=d.step(),H=!0;switch(t.key){case"ArrowRight":case"ArrowUp":b(d.value()+r);break;case"ArrowLeft":case"ArrowDown":b(d.value()-r);break;case"Home":b(d.min());break;case"End":b(d.max());break;case"PageUp":b(d.value()+r*10);break;case"PageDown":b(d.value()-r*10);break;default:H=!1}if(H)t.preventDefault()},l=U(()=>{let t=d.containerEl();if(!t)return;return t.addEventListener("pointerdown",y),()=>t.removeEventListener("pointerdown",y)}),f=U(()=>{let t=d.thumbEl();if(!t)return;return t.addEventListener("keydown",L),()=>t.removeEventListener("keydown",L)});return()=>{E(),l(),f(),m.remove()}}});var U9=V("sui-slider",{props:{value:P.number({default:0,reflect:!0}),min:P.number({default:0}),max:P.number({default:100}),step:P.number({default:1}),disabled:P.bool({default:!1,reflect:!0}),tone:P.union(["default","primary","info","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),name:P.string({default:""}),label:P.string({default:"slider"})},shadow:!0,styles:`
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

`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.min=u.min,n.max=u.max,n.step=u.step,n.disabled=u.disabled,n.tone=u.tone,n.size=u.size,n.name=u.name,n.label=u.label,n.containerEl=J(null),n.fillEl=J(null),n.thumbEl=J(null),n.trackEl=J(null);let b=null;if(M0&&typeof M0.setup==="function"){if(i.scope=n,b=M0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","container"),A(n,"containerEl",E);let M=document.createElement("div");M.setAttribute("class","track"),M.setAttribute("part","track"),A(n,"trackEl",M);let y=document.createElement("div");y.setAttribute("class","fill"),y.setAttribute("part","fill"),A(n,"fillEl",y),M.appendChild(y),E.appendChild(M);let L=document.createElement("div");if(L.setAttribute("class","thumb"),L.setAttribute("part","thumb"),L.setAttribute("role","slider"),m.push(W(L,"tabindex",()=>n.disabled()?"-1":"0")),m.push(W(L,"aria-valuemin",()=>n.min())),m.push(W(L,"aria-valuemax",()=>n.max())),m.push(W(L,"aria-valuenow",()=>n.value())),m.push(W(L,"aria-label",()=>n.label()||"slider")),m.push(W(L,"aria-disabled",()=>n.disabled()?"true":"false")),A(n,"thumbEl",L),E.appendChild(L),w.appendChild(E),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let f=new Proxy(n,{get(t,r){return t[r]},set(t,r,H){let C=t[r];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(l.mount){let t=l.mount(f,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(f))}}return n}});var F9=V("sui-segmented-option",{props:{value:P.string({default:"",reflect:!0}),icon:P.string({default:"",reflect:!0}),disabled:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.icon=u.icon,n.disabled=u.disabled;let b=d.shadowRoot??d;Y(b);let w=document.createTextNode(`

`);b.appendChild(w);let E=document.createTextNode(`
`);b.appendChild(E);let M=document.createTextNode(`
`);b.appendChild(M);let y=document.createTextNode(`

`);b.appendChild(y);let L=document.createTextNode(`
`);return b.appendChild(L),n}});var am=(i)=>typeof CSS<"u"&&CSS.escape?CSS.escape(i):i.replace(/[^\w-]/g,"\\$&"),y0=N({scope:()=>{let i=J([]),u=e(null);return{options:i,containerEl:u}},mount(i,u){if(!u.hasAttribute("role"))u.setAttribute("role","radiogroup");let d=document.createElement("input");d.type="hidden",u.appendChild(d);let m=()=>{return Array.from(u.querySelectorAll(":scope > sui-segmented-option")).map((l)=>({value:l.getAttribute("value")??"",label:l.textContent?.trim()??"",icon:l.getAttribute("icon")||null,disabled:l.hasAttribute("disabled")}))},n=(L)=>{if(i.disabled())return;let f=i.options().find((t)=>t.value===L);if(!f||f.disabled)return;if(i.value()===L)return;i.value.set(L),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:L},bubbles:!0,composed:!0}))},b=(L)=>{let l=L.target?.closest(".btn");if(!l)return;let f=l.getAttribute("data-value");if(f!==null)n(f)},w=(L)=>{let f=i.options().filter((g)=>!g.disabled&&!i.disabled());if(!f.length)return;let t=f.map((g)=>g.value),r=i.value(),H=Math.max(0,t.indexOf(r)),C=H;if(L.key==="ArrowRight"||L.key==="ArrowDown")L.preventDefault(),C=(H+1)%t.length;else if(L.key==="ArrowLeft"||L.key==="ArrowUp")L.preventDefault(),C=(H-1+t.length)%t.length;else if(L.key==="Enter"||L.key===" "){L.preventDefault();let j=u.shadowRoot.activeElement?.getAttribute("data-value");if(j)n(j);return}else return;let z=t[C]??"";n(z),queueMicrotask(()=>{u.shadowRoot?.querySelector(`.btn[data-value="${am(z)}"]`)?.focus()})};i.options.set(m());let E=new MutationObserver(()=>i.options.set(m()));E.observe(u,{childList:!0,subtree:!1});let M=U(()=>{d.name=i.name(),d.value=i.value()}),y=U(()=>{let L=i.containerEl();if(!L)return;return L.addEventListener("click",b),L.addEventListener("keydown",w),()=>{L.removeEventListener("click",b),L.removeEventListener("keydown",w)}});return()=>{E.disconnect(),M(),y(),d.remove()}}});var k9=V("sui-segmented",{props:{value:P.string({default:"",reflect:!0}),size:P.union(["sm","md"],{default:"md",reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),name:P.string({default:""})},shadow:!0,styles:`
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

`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.size=u.size,n.disabled=u.disabled,n.name=u.name,n.containerEl=J(null);let b=null;if(y0&&typeof y0.setup==="function"){if(i.scope=n,b=y0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>h({anchor:L,items:()=>n.options(),key:(l,f)=>{let t=Object.create(n);return t.opt=l,t.__i=f,((r)=>l.value)(t)},render:(l,f)=>{let t=Object.create(n);return t.opt=l,t.__i=f,t.__v=J(0),((r)=>{let H=(D)=>I({anchor:D,test:()=>{if(r.opt.icon)return 0;return-1},branches:[()=>{let Z=document.createDocumentFragment(),X=[],G=document.createElement("span");G.setAttribute("class","btn-icon"),G.setAttribute("aria-hidden","true");let Q=document.createElement("sui-icon");return Q.name=r.opt.icon,Q.setAttribute("size","sm"),G.appendChild(Q),Z.appendChild(G),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let q of X)try{q()}catch(F){}}}}]}),C=document.createDocumentFragment(),z=[],g=document.createElement("button");g.setAttribute("class","btn"),g.setAttribute("role","radio"),g.setAttribute("type","button");{let D=r.opt.value;if(D==null||D===!1||D==="")g.removeAttribute("data-value");else g.setAttribute("data-value",D===!0?"":String(D))}z.push(W(g,"aria-checked",()=>(r.__v(),r.opt.value===r.value()?"true":"false"))),z.push(W(g,"aria-disabled",()=>(r.__v(),r.disabled()||r.opt.disabled?"true":"false"))),z.push(Gn(g,"disabled",()=>(r.__v(),r.disabled()||r.opt.disabled))),z.push(W(g,"tabindex",()=>(r.__v(),r.opt.value===r.value()?"0":"-1")));let j=document.createComment("if");g.appendChild(j),z.push(H(j));let T=document.createTextNode("");return g.appendChild(T),T.textContent=r.opt.label,C.appendChild(g),{nodes:Array.from(C.childNodes),cleanup:()=>{for(let D of z)try{D()}catch(Z){}},update:(D,Z)=>{r.opt=D,r.__i=Z;{let X=r.opt.value;if(X==null||X===!1||X==="")g.removeAttribute("data-value");else g.setAttribute("data-value",X===!0?"":String(X))}__e11.name=r.opt.icon,T.textContent=r.opt.label,r.__v.set((X)=>X+1)}}})(t)}}),E=d.shadowRoot??d;Y(E);let M=document.createElement("div");M.setAttribute("class","container"),M.setAttribute("part","container"),A(n,"containerEl",M);let y=document.createComment("for");if(M.appendChild(y),m.push(w(y)),E.appendChild(M),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var w0=N({mount(i,u){let d=i,m=U(()=>{let n=d.buttonEl();if(!n)return;let b=()=>{if(d.disabled())return;d.pressed.set(!d.pressed()),u.dispatchEvent(new CustomEvent("sui-toggle",{detail:{pressed:d.pressed(),value:d.value()},bubbles:!0,composed:!0}))};return n.addEventListener("click",b),()=>n.removeEventListener("click",b)});return u.focusButton=()=>{u.shadowRoot?.querySelector("button")?.focus()},()=>m()}});var $9=V("sui-toggle",{props:{pressed:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),value:P.string({default:"",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:P.union(["outline","ghost"],{default:"outline",reflect:!0})},shadow:!0,styles:`
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

`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.pressed=u.pressed,n.disabled=u.disabled,n.value=u.value,n.size=u.size,n.variant=u.variant,n.buttonEl=J(null);let b=null;if(w0&&typeof w0.setup==="function"){if(i.scope=n,b=w0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("button");E.setAttribute("part","button"),E.setAttribute("type","button"),m.push(W(E,"aria-pressed",()=>n.pressed()?"true":"false")),m.push(W(E,"disabled",()=>n.disabled())),A(n,"buttonEl",E);let M=document.createElement("slot");if(E.appendChild(M),w.appendChild(E),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let L=new Proxy(n,{get(l,f){return l[f]},set(l,f,t){let r=l[f];if(r!=null&&typeof r.set==="function"){if(typeof t==="function")r.set(()=>t);else r.set(t);return!0}return!1}});if(y.mount){let l=y.mount(L,d);if(typeof l==="function")m.push(l)}if(y.unmount)m.push(()=>y.unmount(L))}}return n}});var H0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","group");let m=document.createElement("input");m.type="hidden",u.appendChild(m);let n=()=>{let r=d.defaultSlotEl();if(!r)return Array.from(u.querySelectorAll(":scope > sui-toggle"));return r.assignedElements().filter((H)=>H.tagName.toLowerCase()==="sui-toggle")},b=()=>n().filter((r)=>!r.hasAttribute("disabled")),w=()=>new Set(d.value().split(",").filter(Boolean)),E=(r)=>{let H=Array.from(r).join(",");d.value.set(H),M(),m.name=d.name(),m.value=H,u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:H},bubbles:!0,composed:!0}))},M=()=>{let r=w(),H=d.size(),C=d.variant();for(let z of n()){let g=z.getAttribute("value")??"";if(z.toggleAttribute("pressed",r.has(g)),H)z.setAttribute("size",H);if(C)z.setAttribute("variant",C)}},y=(r)=>{let H=r;H.stopPropagation();let C=H.composedPath().find((j)=>j.tagName?.toLowerCase()==="sui-toggle");if(!C)return;let z=C.getAttribute("value")??"",g=w();if(d.type()==="single")if(g.has(z))E(new Set);else E(new Set([z]));else{if(g.has(z))g.delete(z);else g.add(z);E(g)}},L=(r)=>{if(r.key!=="ArrowRight"&&r.key!=="ArrowLeft")return;let H=b();if(!H.length)return;let C=r.composedPath().find((j)=>H.includes(j)),z=C?H.indexOf(C):-1,g=r.key==="ArrowRight"?(z+1)%H.length:(z-1+H.length)%H.length;H[g]?.focusButton?.(),r.preventDefault()},l=U(()=>{d.value(),d.size(),d.variant(),M()}),f=U(()=>{let r=d.defaultSlotEl();if(!r)return;let H=()=>M();return r.addEventListener("slotchange",H),()=>r.removeEventListener("slotchange",H)}),t=U(()=>{m.name=d.name(),m.value=d.value()});return u.addEventListener("sui-toggle",y),u.addEventListener("keydown",L),()=>{l(),f(),t(),u.removeEventListener("sui-toggle",y),u.removeEventListener("keydown",L),m.remove()}}});var _9=V("sui-toggle-group",{props:{type:P.union(["single","multiple"],{default:"single",reflect:!0}),value:P.string({default:"",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:P.union(["outline","ghost"],{default:"outline",reflect:!0}),name:P.string({default:""})},shadow:!0,styles:`
  :host {
    display: inline-flex;
    gap: 0;
  }

  ::slotted(sui-toggle:not(:first-child)) { margin-left: -1px; }
  ::slotted(sui-toggle:first-child) { border-radius: var(--radius, 6px) 0 0 var(--radius, 6px); }
  ::slotted(sui-toggle:last-child) { border-radius: 0 var(--radius, 6px) var(--radius, 6px) 0; }
  ::slotted(sui-toggle:not(:first-child):not(:last-child)) { border-radius: 0; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.type=u.type,n.value=u.value,n.size=u.size,n.variant=u.variant,n.name=u.name,n.defaultSlotEl=J(null);let b=null;if(H0&&typeof H0.setup==="function"){if(i.scope=n,b=H0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");if(A(n,"defaultSlotEl",E),w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var u7=V("sui-code",{props:{lang:P.string({default:"",reflect:!0}),block:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.lang=u.lang,n.block=u.block;let b=(y)=>I({anchor:y,test:()=>{if(n.lang())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),l=[],f=document.createElement("header");f.setAttribute("part","label");let t=document.createTextNode("");return f.appendChild(t),l.push(a(t,()=>n.lang())),L.appendChild(f),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let r of l)try{r()}catch(H){}}}}]}),w=(y)=>I({anchor:y,test:()=>{if(n.block())return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),l=[],f=document.createElement("div");f.setAttribute("class","block"),f.setAttribute("part","block"),f.setAttribute("role","region"),l.push(W(f,"aria-label",()=>n.lang()?n.lang()+" code block":"code block"));let t=document.createComment("if");f.appendChild(t),l.push(b(t));let r=document.createElement("pre");r.setAttribute("part","pre");let H=document.createElement("slot");return r.appendChild(H),f.appendChild(r),L.appendChild(f),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let C of l)try{C()}catch(z){}}}},()=>{let L=document.createDocumentFragment(),l=[],f=document.createElement("code");f.setAttribute("class","inline"),f.setAttribute("part","inline");let t=document.createElement("slot");return f.appendChild(t),L.appendChild(f),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let r of l)try{r()}catch(H){}}}}]}),E=d.shadowRoot??d;Y(E);let M=document.createComment("if");return E.appendChild(M),m.push(w(M)),n}});var C0=N({mount(i,u){let d=i,m,n=async()=>{try{await navigator.clipboard.writeText(d.value())}catch{let w=document.createElement("textarea");w.value=d.value(),document.body.appendChild(w),w.select();try{document.execCommand("copy")}catch{}w.remove()}if(d.copied.set(!0),u.dispatchEvent(new CustomEvent("sui-copy",{detail:{value:d.value()},bubbles:!0,composed:!0})),m)clearTimeout(m);m=setTimeout(()=>{d.copied.set(!1)},1200)},b=U(()=>{let w=d.buttonEl();if(!w)return;return w.addEventListener("click",n),()=>w.removeEventListener("click",n)});return()=>{if(b(),m)clearTimeout(m)}}});var r7=V("sui-copy-button",{props:{value:P.string({default:"",reflect:!0}),label:P.string({default:"Copy",reflect:!0}),copied:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.label=u.label,n.copied=u.copied,n.buttonEl=J(null);let b=null;if(C0&&typeof C0.setup==="function"){if(i.scope=n,b=C0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(t)=>I({anchor:t,test:()=>{if(n.copied())return 0;return 1},branches:[()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("sui-icon");return C.setAttribute("name","check"),C.setAttribute("size","sm"),C.setAttribute("aria-hidden","true"),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let z of H)try{z()}catch(g){}}}},()=>{let r=document.createDocumentFragment(),H=[],C=document.createElement("sui-icon");return C.setAttribute("name","copy"),C.setAttribute("size","sm"),C.setAttribute("aria-hidden","true"),r.appendChild(C),{nodes:Array.from(r.childNodes),cleanup:()=>{for(let z of H)try{z()}catch(g){}}}}]}),E=d.shadowRoot??d;Y(E);let M=document.createElement("button");M.setAttribute("part","button"),M.setAttribute("type","button"),m.push(W(M,"aria-label",()=>n.label())),A(n,"buttonEl",M);let y=document.createComment("if");M.appendChild(y),m.push(w(y));let L=document.createElement("slot");m.push(dn(L,d,"data-empty")),M.appendChild(L);let l=document.createElement("span");l.setAttribute("class","default-label");let f=document.createTextNode("");if(l.appendChild(f),m.push(a(f,()=>n.copied()?"Copied":n.label())),M.appendChild(l),E.appendChild(M),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let r=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let g=H[C];if(g!=null&&typeof g.set==="function"){if(typeof z==="function")g.set(()=>z);else g.set(z);return!0}return!1}});if(t.mount){let H=t.mount(r,d);if(typeof H==="function")m.push(H)}if(t.unmount)m.push(()=>t.unmount(r))}}return n}});function ju(i){return i.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function Hd(i,u,d,m,n){let b=u!==null?`<span class="key">"${ju(u)}"</span>: `:"";if(i===null)return`${b}<span class="null">null</span>`;if(typeof i==="string")return`${b}<span class="s">"${ju(i)}"</span>`;if(typeof i==="number")return`${b}<span class="n">${i}</span>`;if(typeof i==="boolean")return`${b}<span class="b">${String(i)}</span>`;let w=Array.isArray(i),E=w?i.map((t,r)=>[String(r),t]):Object.entries(i),M=m||d<n,y=w?"[":"{",L=w?"]":"}",l=E.length;if(l===0)return`${b}${y}${L}`;let f=E.map(([t,r])=>`<div class="node">${Hd(r,w?null:t,d+1,m,n)}</div>`).join("");return`<details${M?" open":""}>
<summary><span class="caret">▶</span>${b}${y}<span class="meta">${l} ${w?"items":"keys"}</span></summary>
<div class="branch">${f}</div>
${L}</details>`}var z0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","tree");if(!u.hasAttribute("aria-label"))u.setAttribute("aria-label","JSON tree");let m=d.contentEl();if(!m)return()=>{};return mn(m,()=>{let n=d.data(),b=d.expanded(),w=d.depth();if(!n)return"";let E;try{E=JSON.parse(n)}catch(M){return`<span class="error" part="error" role="alert">Invalid JSON: ${ju(M.message)}</span>`}return`<div part="root">${Hd(E,null,0,b,w)}</div>`})}});var w7=V("sui-json-view",{props:{data:P.string({default:"",reflect:!0}),expanded:P.bool({default:!1,reflect:!0}),depth:P.number({default:1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.data=u.data,n.expanded=u.expanded,n.depth=u.depth,n.contentEl=J(null);let b=null;if(z0&&typeof z0.setup==="function"){if(i.scope=n,b=z0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");if(E.setAttribute("class","json-root"),A(n,"contentEl",E),w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});function Cd(i){return i.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function Sm(i,u){let d=i.split(`
`),m=u.split(`
`),n=d.length,b=m.length,w=Array.from({length:n+1},()=>Array(b+1).fill(0));for(let L=n-1;L>=0;L--)for(let l=b-1;l>=0;l--)w[L][l]=d[L]===m[l]?w[L+1][l+1]+1:Math.max(w[L+1][l],w[L][l+1]);let E=[],M=0,y=0;while(M<n&&y<b)if(d[M]===m[y])E.push({kind:"ctx",text:d[M]}),M++,y++;else if(w[M+1][y]>=w[M][y+1])E.push({kind:"del",text:d[M]}),M++;else E.push({kind:"add",text:m[y]}),y++;while(M<n)E.push({kind:"del",text:d[M++]});while(y<b)E.push({kind:"add",text:m[y++]});return E}function vm(i){return i.split(`
`).map((u)=>{if(u.startsWith("@@"))return{kind:"hunk",text:u};if(u.startsWith("+"))return{kind:"add",text:u.slice(1)};if(u.startsWith("-"))return{kind:"del",text:u.slice(1)};return{kind:"ctx",text:u.startsWith(" ")?u.slice(1):u}})}function xm(i){return i.map((u)=>{let d=u.kind==="add"?"+":u.kind==="del"?"-":u.kind==="hunk"?"@":" ",m=u.kind==="add"?"added":u.kind==="del"?"removed":u.kind==="hunk"?"hunk":"context";return`<div class="line ${u.kind}" part="line" role="listitem" aria-label="${m}: ${Cd(u.text)}"><span class="sign" aria-hidden="true">${d}</span><span>${Cd(u.text)}</span></div>`}).join("")}var g0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","region");if(!u.hasAttribute("aria-label"))u.setAttribute("aria-label","diff");let m=d.contentEl();if(!m)return()=>{};return mn(m,()=>{let n=d.value(),b=d.from(),w=d.to(),E;if(n)E=vm(n);else if(b||w)E=Sm(b,w);else E=[];return xm(E)})}});var j7=V("sui-diff",{props:{from:P.string({default:"",reflect:!0}),to:P.string({default:"",reflect:!0}),value:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.from=u.from,n.to=u.to,n.value=u.value,n.contentEl=J(null);let b=null;if(g0&&typeof g0.setup==="function"){if(i.scope=n,b=g0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");if(E.setAttribute("class","diff-root"),A(n,"contentEl",E),w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});function Du(i){return i.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;")}function $n(i){let u=Du(i);return u=u.replace(/`([^`]+)`/g,(d,m)=>`<code>${m}</code>`),u=u.replace(/\*\*([^*]+)\*\*/g,"<strong>$1</strong>"),u=u.replace(/(^|[^*])\*([^*\n]+)\*/g,"$1<em>$2</em>"),u=u.replace(/\[([^\]]+)\]\(([^)\s]+)\)/g,(d,m,n)=>`<a href="${n}" rel="noopener noreferrer">${m}</a>`),u}function hm(i){let u=i.replace(/\r\n/g,`
`).split(`
`),d=[],m=0;while(m<u.length){let n=u[m];if(/^```/.test(n)){let E=n.slice(3).trim(),M=[];m++;while(m<u.length&&!/^```/.test(u[m]))M.push(u[m++]);m++,d.push(`<pre data-lang="${Du(E)}"><code>${Du(M.join(`
`))}</code></pre>`);continue}let b=/^(#{1,6})\s+(.*)$/.exec(n);if(b){d.push(`<h${b[1].length}>${$n(b[2])}</h${b[1].length}>`),m++;continue}if(/^\s*[-*]\s+/.test(n)){let E=[];while(m<u.length&&/^\s*[-*]\s+/.test(u[m]))E.push(`<li>${$n(u[m].replace(/^\s*[-*]\s+/,""))}</li>`),m++;d.push(`<ul>${E.join("")}</ul>`);continue}if(/^\s*\d+\.\s+/.test(n)){let E=[];while(m<u.length&&/^\s*\d+\.\s+/.test(u[m]))E.push(`<li>${$n(u[m].replace(/^\s*\d+\.\s+/,""))}</li>`),m++;d.push(`<ol>${E.join("")}</ol>`);continue}if(/^>\s?/.test(n)){let E=[];while(m<u.length&&/^>\s?/.test(u[m]))E.push(u[m].replace(/^>\s?/,"")),m++;d.push(`<blockquote>${$n(E.join(" "))}</blockquote>`);continue}if(n.trim()===""){m++;continue}let w=[];while(m<u.length&&u[m].trim()!==""&&!/^(```|#|>|\s*[-*]\s+|\s*\d+\.\s+)/.test(u[m]))w.push(u[m]),m++;d.push(`<p>${$n(w.join(" "))}</p>`)}return d.join(`
`)}var P0=N({mount(i){let u=i,d=u.contentEl();if(!d)return()=>{};return mn(d,()=>hm(u.value()??""))}});var G7=V("sui-markdown",{props:{value:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.contentEl=J(null);let b=null;if(P0&&typeof P0.setup==="function"){if(i.scope=n,b=P0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");if(E.setAttribute("class","md-root"),A(n,"contentEl",E),w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var zd={actions:"#view-topbar-actions","status-start":"#view-status-start","status-end":"#view-status-end"},j0={title:"#breadcrumbs-title",branch:"#status-branch"},D0=N({mount(i,u){let d=i,m={};function n(){for(let M of Object.keys(zd)){let y=document.querySelector(zd[M]);if(!y)continue;let L=Array.from(u.querySelectorAll(`[slot="${M}"]`));if(L.length===0)continue;for(let l of L)y.appendChild(l);m[M]=L}}function b(){let M=document.querySelector(j0.title);if(M)M.textContent="";if(d.branch()){let y=document.querySelector(j0.branch);if(y)y.textContent=""}if(d.ahead()>0||d.behind()>0){let y=document.querySelector("#status-ab");if(y)y.textContent=""}if(d.tone()!=="default"||d.busy()){let y=document.querySelector("#status-dot");if(y)delete y.dataset.tone,y.removeAttribute("data-busy")}}function w(){for(let M of Object.keys(m)){for(let y of m[M]??[])y.remove();delete m[M]}b()}let E=U(()=>{let M=d.title(),y=d.branch(),L=d.ahead(),l=d.behind(),f=d.tone(),t=d.busy();if(M){let r=document.querySelector(j0.title);if(r)r.textContent=M}if(y){let r=document.querySelector(j0.branch);if(r)r.textContent=y}if(L>0||l>0){let r=document.querySelector("#status-ab");if(r){let H=[];if(L>0)H.push(`↑${L}`);if(l>0)H.push(`↓${l}`);r.textContent=H.join(" ")}}if(f!=="default"||t){let r=document.querySelector("#status-dot");if(r)if(r.dataset.tone=f,t)r.setAttribute("data-busy","");else r.removeAttribute("data-busy")}});return queueMicrotask(()=>n()),()=>{E(),w()}}});var U7=V("sui-view-chrome",{props:{title:P.string({default:"",reflect:!0}),branch:P.string({default:"",reflect:!0}),ahead:P.number({default:0,reflect:!0}),behind:P.number({default:0,reflect:!0}),tone:P.union(["default","success","warning","destructive","accent","muted"],{default:"default",reflect:!0}),busy:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
  :host { display: none; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.title=u.title,n.branch=u.branch,n.ahead=u.ahead,n.behind=u.behind,n.tone=u.tone,n.busy=u.busy;let b=null;if(D0&&typeof D0.setup==="function"){if(i.scope=n,b=D0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createTextNode(`

`);w.appendChild(E);let M=document.createTextNode(`
`);w.appendChild(M);let y=document.createTextNode(`
`);w.appendChild(y);let L=document.createTextNode(`
`);w.appendChild(L);let l=document.createTextNode(`
`);w.appendChild(l);let f=document.createTextNode(`
`);w.appendChild(f);let t=document.createTextNode(`

`);w.appendChild(t);let r=document.createTextNode(`
`);if(w.appendChild(r),b!=null){let H=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(H){let C=new Proxy(n,{get(z,g){return z[g]},set(z,g,j){let T=z[g];if(T!=null&&typeof T.set==="function"){if(typeof j==="function")T.set(()=>j);else T.set(j);return!0}return!1}});if(H.mount){let z=H.mount(C,d);if(typeof z==="function")m.push(z)}if(H.unmount)m.push(()=>H.unmount(C))}}return n}});var Z0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","navigation");let m=()=>Array.from(u.querySelectorAll(":scope > sui-navigation-item")),n=()=>m().filter((L)=>!L.hasAttribute("disabled")),b=()=>{let L=d.value();for(let l of m())l.toggleAttribute("active",l.getAttribute("value")===L)},w=(L)=>{let l=L;l.stopPropagation();let f=l.detail.value;d.value.set(f),b(),u.dispatchEvent(new CustomEvent("sui-change",{detail:{value:f},bubbles:!0,composed:!0}))},E=(L)=>{if(L.key!=="ArrowRight"&&L.key!=="ArrowLeft"&&L.key!=="Home"&&L.key!=="End")return;let l=n();if(!l.length)return;let f=L.composedPath().find((H)=>l.includes(H)),t=f?l.indexOf(f):-1,r;if(L.key==="ArrowRight")r=l[(t+1)%l.length];else if(L.key==="ArrowLeft")r=l[(t-1+l.length)%l.length];else if(L.key==="Home")r=l[0];else if(L.key==="End")r=l[l.length-1];if(r)L.preventDefault(),r.focusLink?.()},M=U(()=>{d.value(),b()}),y=U(()=>{let L=d.defaultSlotEl();if(!L)return;let l=()=>b();return L.addEventListener("slotchange",l),()=>L.removeEventListener("slotchange",l)});return u.addEventListener("sui-nav-select",w),u.addEventListener("keydown",E),()=>{M(),y(),u.removeEventListener("sui-nav-select",w),u.removeEventListener("keydown",E)}}});var B7=V("sui-navigation-menu",{props:{value:P.string({default:"",reflect:!0})},shadow:!0,styles:`
  :host {
    display: flex;
    align-items: stretch;
    gap: 0;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-body);
  }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.defaultSlotEl=J(null);let b=null;if(Z0&&typeof Z0.setup==="function"){if(i.scope=n,b=Z0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");if(A(n,"defaultSlotEl",E),w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var T0=N({mount(i,u){let d=i,m=(n)=>{if(d.disabled())return;if(!d.href())n.preventDefault();u.dispatchEvent(new CustomEvent("sui-nav-select",{detail:{value:d.value()},bubbles:!0,composed:!0}))};return u.focusLink=()=>{u.shadowRoot?.querySelector(".link")?.focus()},u.addEventListener("click",m),()=>u.removeEventListener("click",m)}});var v7=V("sui-navigation-item",{props:{value:P.string({default:"",reflect:!0}),active:P.bool({default:!1,reflect:!0}),disabled:P.bool({default:!1,reflect:!0}),href:P.string({default:""})},shadow:!0,styles:`
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

`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.value=u.value,n.active=u.active,n.disabled=u.disabled,n.href=u.href;let b=null;if(T0&&typeof T0.setup==="function"){if(i.scope=n,b=T0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>I({anchor:L,test:()=>{if(n.href())return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("a");t.setAttribute("class","link"),t.setAttribute("part","link"),f.push(W(t,"href",()=>n.href())),f.push(W(t,"aria-current",()=>n.active()?"page":"false"));let r=document.createElement("slot");return t.appendChild(r),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let H of f)try{H()}catch(C){}}}},()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("button");t.setAttribute("class","link"),t.setAttribute("part","link"),t.setAttribute("type","button"),f.push(W(t,"aria-current",()=>n.active()?"true":"false")),f.push(W(t,"disabled",()=>n.disabled()));let r=document.createElement("slot");return t.appendChild(r),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let H of f)try{H()}catch(C){}}}}]}),E=d.shadowRoot??d;Y(E);let M=document.createComment("if");E.appendChild(M),m.push(w(M));let y=document.createElement("div");if(y.setAttribute("class","indicator"),y.setAttribute("part","indicator"),y.setAttribute("aria-hidden","true"),E.appendChild(y),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var J0=N({mount(i,u){let d=i,m=null,n=()=>{let E=d.viewportEl();if(!E)return;let M=d.orientation(),y=M==="vertical"||M==="both",L=M==="horizontal"||M==="both",l=d.vThumb(),f=d.hThumb();if(y&&l){let t=E.clientHeight/E.scrollHeight;if(t<1){let r=Math.max(24,t*E.clientHeight),C=E.scrollTop/(E.scrollHeight-E.clientHeight)*(E.clientHeight-r);l.style.height=`${r}px`,l.style.top=`${C}px`,l.style.display=""}else l.style.display="none"}if(L&&f){let t=E.clientWidth/E.scrollWidth;if(t<1){let r=Math.max(24,t*E.clientWidth),C=E.scrollLeft/(E.scrollWidth-E.clientWidth)*(E.clientWidth-r);f.style.width=`${r}px`,f.style.left=`${C}px`,f.style.display=""}else f.style.display="none"}},b=()=>{n();let E=d.vScrollbar(),M=d.hScrollbar();if(E?.classList.add("active"),M?.classList.add("active"),m)clearTimeout(m);m=setTimeout(()=>{E?.classList.remove("active"),M?.classList.remove("active")},1200)},w=d.viewportEl();if(w)w.addEventListener("scroll",b,{passive:!0});return n(),()=>{if(m)clearTimeout(m);if(w)w.removeEventListener("scroll",b)}}});var s7=V("sui-scroll-area",{props:{orientation:P.union(["vertical","horizontal","both"],{default:"vertical",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.orientation=u.orientation,n.size=u.size,n.hScrollbar=J(null),n.hThumb=J(null),n.vScrollbar=J(null),n.vThumb=J(null),n.viewportEl=J(null);let b=null;if(J0&&typeof J0.setup==="function"){if(i.scope=n,b=J0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","viewport"),E.setAttribute("part","viewport"),E.setAttribute("tabindex","0"),A(n,"viewportEl",E);let M=document.createElement("slot");E.appendChild(M),w.appendChild(E);let y=document.createElement("div");y.setAttribute("class","scrollbar scrollbar-v"),y.setAttribute("part","scrollbar-v"),y.setAttribute("aria-hidden","true"),A(n,"vScrollbar",y);let L=document.createElement("div");L.setAttribute("class","thumb"),A(n,"vThumb",L),y.appendChild(L),w.appendChild(y);let l=document.createElement("div");l.setAttribute("class","scrollbar scrollbar-h"),l.setAttribute("part","scrollbar-h"),l.setAttribute("aria-hidden","true"),A(n,"hScrollbar",l);let f=document.createElement("div");if(f.setAttribute("class","thumb"),A(n,"hThumb",f),l.appendChild(f),w.appendChild(l),b!=null){let t=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(t){let r=new Proxy(n,{get(H,C){return H[C]},set(H,C,z){let g=H[C];if(g!=null&&typeof g.set==="function"){if(typeof z==="function")g.set(()=>z);else g.set(z);return!0}return!1}});if(t.mount){let H=t.mount(r,d);if(typeof H==="function")m.push(H)}if(t.unmount)m.push(()=>t.unmount(r))}}return n}});var X0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","toolbar");if(!u.hasAttribute("aria-orientation"))u.setAttribute("aria-orientation","horizontal");let m=()=>{return(d.defaultSlot()?.assignedElements({flatten:!0})??[]).filter((E)=>!E.hasAttribute("disabled"))},n=(b)=>{if(b.key!=="ArrowRight"&&b.key!=="ArrowLeft")return;let w=m();if(!w.length)return;let E=b.composedPath().find((L)=>w.includes(L))||null,M=E?w.indexOf(E):-1,y=b.key==="ArrowRight"?(M+1)%w.length:(M-1+w.length)%w.length;w[y]?.focus(),b.preventDefault()};return u.addEventListener("keydown",n),()=>u.removeEventListener("keydown",n)}});var u6=V("sui-toolbar",{props:{justify:P.union(["start","center","end","between"],{default:"start",reflect:!0}),align:P.union(["center","start","end","baseline"],{default:"center",reflect:!0}),gap:P.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:P.union(["default","bordered"],{default:"default",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.justify=u.justify,n.align=u.align,n.gap=u.gap,n.variant=u.variant,n.defaultSlot=J(null);let b=null;if(X0&&typeof X0.setup==="function"){if(i.scope=n,b=X0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");E.setAttribute("name","spacer"),w.appendChild(E);let M=document.createElement("slot");if(A(n,"defaultSlot",M),w.appendChild(M),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let L=new Proxy(n,{get(l,f){return l[f]},set(l,f,t){let r=l[f];if(r!=null&&typeof r.set==="function"){if(typeof t==="function")r.set(()=>t);else r.set(t);return!0}return!1}});if(y.mount){let l=y.mount(L,d);if(typeof l==="function")m.push(l)}if(y.unmount)m.push(()=>y.unmount(L))}}return n}});var gd={dashboard:"layout-dashboard",pipelines:"workflow",agents:"bot",skills:"sparkles",git:"git-branch",collections:"database",canvas:"frame",settings:"settings",chat:"messages-square",events:"radio",specs:"file-text",theme:"palette",setup:"rocket",overview:"compass",mesh:"network"},Pd={"theme-studio":"theme"},$m=new Set([...Object.keys(gd),...Object.keys(Pd)]),G0=N({mount(i,u){let d=i,m=U(()=>{let n=d.name(),b=d.label(),w=Pd[n]??n,M=$m.has(n)?gd[w]:"dot",y=d.glyphEl();if(y)y.setAttribute("name",M);if(b)u.setAttribute("role","img"),u.setAttribute("aria-label",b),u.removeAttribute("aria-hidden");else u.setAttribute("aria-hidden","true"),u.removeAttribute("role"),u.removeAttribute("aria-label")});return()=>m()}});var r6=V("sui-app-icon",{props:{name:P.string({default:"",reflect:!0}),variant:P.union(["glyph","tile","solid"],{default:"glyph",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),radius:P.string({default:"",reflect:!0}),label:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.variant=u.variant,n.size=u.size,n.radius=u.radius,n.label=u.label,n.glyphEl=J(null);let b=null;if(G0&&typeof G0.setup==="function"){if(i.scope=n,b=G0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("span");E.setAttribute("class","frame"),E.setAttribute("part","frame");let M=document.createElement("slot"),y=document.createElement("sui-icon");if(A(n,"glyphEl",y),y.setAttribute("part","glyph"),y.setAttribute("aria-hidden","true"),M.appendChild(y),E.appendChild(M),w.appendChild(E),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var W0=N({scope:()=>{return{nodes:J([])}},mount(i,u){let d=u.nodes;if(d!=null)i.nodes.set(d);if(Object.defineProperty(u,"nodes",{configurable:!0,get:()=>i.nodes(),set:(m)=>i.nodes.set(m)}),!u.hasAttribute("role"))u.setAttribute("role","tree")}});var w6=V("sui-tree-view",{shadow:!0,styles:`
  :host {
    display: block;
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--foreground);
  }
  ul { list-style: none; margin: 0; padding: 0; }
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null),b=null;if(W0&&typeof W0.setup==="function"){if(i.scope=n,b=W0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(L)=>h({anchor:L,items:()=>n.nodes(),key:(l,f)=>{let t=Object.create(n);return t.n=l,t.__i=f,((r)=>l.id)(t)},render:(l,f)=>{let t=Object.create(n);return t.n=l,t.__i=f,t.__v=J(0),((r)=>{let H=document.createDocumentFragment(),C=[],z=document.createElement("sui-tree-item");return z.node=r.n,H.appendChild(z),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let g of C)try{g()}catch(j){}},update:(g,j)=>{r.n=g,r.__i=j,z.node=r.n,r.__v.set((T)=>T+1)}}})(t)}}),E=(L)=>I({anchor:L,test:()=>{if(n.nodes().length>0)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("ul");t.setAttribute("role","tree");let r=document.createComment("for");return t.appendChild(r),f.push(w(r)),l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let H of f)try{H()}catch(C){}}}},()=>{let l=document.createDocumentFragment(),f=[],t=document.createElement("slot");return l.appendChild(t),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let r of f)try{r()}catch(H){}}}}]}),M=d.shadowRoot??d;Y(M);let y=document.createComment("if");if(M.appendChild(y),m.push(E(y)),b!=null){let L=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(L){let l=new Proxy(n,{get(f,t){return f[t]},set(f,t,r){let H=f[t];if(H!=null&&typeof H.set==="function"){if(typeof r==="function")H.set(()=>r);else H.set(r);return!0}return!1}});if(L.mount){let f=L.mount(l,d);if(typeof f==="function")m.push(f)}if(L.unmount)m.push(()=>L.unmount(l))}}return n}});var Q0=N({scope:({label:i})=>{let u=J(null),d=s(()=>{let b=u();return b?b.label:i()}),m=s(()=>{return u()?.children??[]}),n=e(null);return{node:u,nodeLabel:d,nodeChildren:m,rowEl:n}},mount(i,u){if(!u.hasAttribute("role"))u.setAttribute("role","treeitem");let d=u.node;if(d!=null)i.node.set(d);Object.defineProperty(u,"node",{configurable:!0,get:()=>i.node(),set:(L)=>i.node.set(L)});let m=U(()=>{let L=i.node();if(L!==null)if(!L.children||L.children.length===0)u.setAttribute("leaf","");else u.removeAttribute("leaf")}),n=U(()=>{let L=i.rowEl();if(!L)return;if(!i.leaf())L.setAttribute("aria-expanded",String(i.expanded()));else L.removeAttribute("aria-expanded");L.setAttribute("aria-selected",String(i.selected()))}),b=(L)=>{if(i.leaf())return;L.stopPropagation();let l=!i.expanded();i.expanded.set(l),u.dispatchEvent(new CustomEvent("sui-toggle",{detail:{expanded:l},bubbles:!0,composed:!0}))},w=()=>{u.dispatchEvent(new CustomEvent("sui-select",{bubbles:!0,composed:!0,detail:{}}))},E=(L)=>{w(),b(L)},M=(L)=>{if(L.key==="Enter"||L.key===" ")L.preventDefault(),w();else if(L.key==="ArrowRight"&&!i.leaf()&&!i.expanded())L.preventDefault(),b(L);else if(L.key==="ArrowLeft"&&i.expanded())L.preventDefault(),b(L)},y=U(()=>{let L=i.rowEl();if(!L)return;return L.addEventListener("click",E),L.addEventListener("keydown",M),()=>{L.removeEventListener("click",E),L.removeEventListener("keydown",M)}});return()=>{m(),n(),y()}}});var j6=V("sui-tree-item",{props:{label:P.string({default:"",reflect:!0}),expanded:P.bool({default:!1,reflect:!0}),selected:P.bool({default:!1,reflect:!0}),leaf:P.bool({default:!1,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.label=u.label,n.expanded=u.expanded,n.selected=u.selected,n.leaf=u.leaf,n.rowEl=J(null);let b=null;if(Q0&&typeof Q0.setup==="function"){if(i.scope=n,b=Q0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(T)=>h({anchor:T,items:()=>n.nodeChildren(),key:(D,Z)=>{let X=Object.create(n);return X.c=D,X.__i=Z,((G)=>D.id)(X)},render:(D,Z)=>{let X=Object.create(n);return X.c=D,X.__i=Z,X.__v=J(0),((G)=>{let Q=document.createDocumentFragment(),q=[],F=document.createElement("sui-tree-item");return F.node=G.c,Q.appendChild(F),{nodes:Array.from(Q.childNodes),cleanup:()=>{for(let k of q)try{k()}catch(K){}},update:(k,K)=>{G.c=k,G.__i=K,F.node=G.c,G.__v.set((B)=>B+1)}}})(X)}}),E=(T)=>I({anchor:T,test:()=>{if(n.nodeChildren().length>0)return 0;return 1},branches:[()=>{let D=document.createDocumentFragment(),Z=[],X=document.createComment("for");return D.appendChild(X),Z.push(w(X)),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let G of Z)try{G()}catch(Q){}}}},()=>{let D=document.createDocumentFragment(),Z=[],X=document.createElement("slot");return D.appendChild(X),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let G of Z)try{G()}catch(Q){}}}}]}),M=d.shadowRoot??d;Y(M);let y=document.createElement("div");y.setAttribute("class","row"),y.setAttribute("part","row"),y.setAttribute("tabindex","0"),A(n,"rowEl",y);let L=document.createElement("span");L.setAttribute("class","caret"),L.setAttribute("part","caret"),L.setAttribute("aria-hidden","true");let l=document.createElement("svg");l.setAttribute("viewbox","0 0 12 12"),l.setAttribute("fill","none"),l.setAttribute("stroke","currentColor"),l.setAttribute("stroke-width","1.5");let f=document.createElement("polyline");f.setAttribute("points","4 3 8 6 4 9"),l.appendChild(f),L.appendChild(l),y.appendChild(L);let t=document.createElement("slot");t.setAttribute("name","icon"),y.appendChild(t);let r=document.createElement("span");r.setAttribute("class","label"),r.setAttribute("part","label");let H=document.createTextNode("");r.appendChild(H),m.push(a(H,()=>n.nodeLabel())),y.appendChild(r);let C=document.createElement("span");C.setAttribute("class","trailing");let z=document.createElement("slot");z.setAttribute("name","trailing"),C.appendChild(z),y.appendChild(C),M.appendChild(y);let g=document.createElement("div");g.setAttribute("class","children"),g.setAttribute("part","children"),g.setAttribute("role","group");let j=document.createComment("if");if(g.appendChild(j),m.push(E(j)),M.appendChild(g),b!=null){let T=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(T){let D=new Proxy(n,{get(Z,X){return Z[X]},set(Z,X,G){let Q=Z[X];if(Q!=null&&typeof Q.set==="function"){if(typeof G==="function")Q.set(()=>G);else Q.set(G);return!0}return!1}});if(T.mount){let Z=T.mount(D,d);if(typeof Z==="function")m.push(Z)}if(T.unmount)m.push(()=>T.unmount(D))}}return n}});var jd=new WeakSet;function om(i){if(jd.has(i))return;jd.add(i);try{if(typeof CSSStyleSheet<"u"&&"replaceSync"in CSSStyleSheet.prototype){let d=new CSSStyleSheet;d.replaceSync(Dd),i.adoptedStyleSheets=[...i.adoptedStyleSheets,d];return}}catch{}let u=i.createElement("style");u.setAttribute("data-sui-table",""),u.textContent=Dd,i.head.appendChild(u)}var V0=N({scope:()=>{let i=J([]),u=J([]),d=J(null),m=J("asc"),n=J([]),b=s(()=>{let w=d(),E=m(),M=i();if(!w)return M;return[...M].sort((y,L)=>{let l=String(y[w]??""),f=String(L[w]??""),t=l<f?-1:l>f?1:0;return E==="asc"?t:-t})});return{rows:i,columns:u,sortKey:d,sortDir:m,selected:n,sortedRows:b}},on:{sortBy(i,u){if(i.sortKey===u)i.sortDir=i.sortDir==="asc"?"desc":"asc";else i.sortKey=u,i.sortDir="asc"},toggleSelect(i,u){let d=i.selected;i.selected=d.includes(u)?d.filter((m)=>m!==u):[...d,u]},selectAll(i){i.selected=i.rows.map((u)=>u.id)},clearSelection(i){i.selected=[]}},mount(i,u){let d=u.rows;if(d!=null)i.rows.set(d);let m=u.columns;if(m!=null)i.columns.set(m);Object.defineProperty(u,"rows",{configurable:!0,get:()=>i.rows(),set:(f)=>i.rows.set(f)}),Object.defineProperty(u,"columns",{configurable:!0,get:()=>i.columns(),set:(f)=>i.columns.set(f)}),om(u.ownerDocument);let n=()=>{u.querySelectorAll("th[data-sort-key]").forEach((f)=>{let t=f.getAttribute("aria-sort")||"none",r=f.querySelector(".sui-sort-indicator");if(!r)return;if(t==="ascending")r.style.transform="rotate(180deg)",r.style.opacity="1";else if(t==="descending")r.style.transform="rotate(0deg)",r.style.opacity="1";else r.style.transform="rotate(0deg)",r.style.opacity="0.4"})},b=()=>{u.querySelectorAll("th[data-sort-key]").forEach((f)=>{if(!f.hasAttribute("aria-sort"))f.setAttribute("aria-sort","none");if(!f.hasAttribute("tabindex"))f.setAttribute("tabindex","0");if(!f.hasAttribute("role"))f.setAttribute("role","columnheader");if(!f.querySelector(".sui-sort-indicator")){let t=document.createElement("span");t.className="sui-sort-indicator",t.setAttribute("aria-hidden","true"),t.style.cssText="margin-left:4px;display:inline-block;opacity:0.5;transition:transform 120ms var(--ease-out),opacity 120ms var(--ease-out);vertical-align:middle;";let r="http://www.w3.org/2000/svg",H=document.createElementNS(r,"svg");H.setAttribute("width","10"),H.setAttribute("height","10"),H.setAttribute("viewBox","0 0 10 10"),H.setAttribute("fill","none"),H.setAttribute("stroke","currentColor"),H.setAttribute("stroke-width","1.5"),H.setAttribute("stroke-linecap","round"),H.setAttribute("stroke-linejoin","round"),H.setAttribute("aria-hidden","true");let C=document.createElementNS(r,"path");C.setAttribute("d","M2 4l3-2 3 2M2 6l3 2 3-2"),H.appendChild(C),t.appendChild(H),f.appendChild(t)}}),n()},w=()=>{let f=u.querySelector(":scope > table tbody > tr")||u.querySelector(":scope > table > tr:not(:has(th))")||u.querySelector(":scope > tbody > tr")||u.querySelector(':scope > tr:not([slot="head"])'),t=u.querySelector('[slot="empty"]')!=null;if(!f&&t)u.setAttribute("data-empty","");else u.removeAttribute("data-empty")},E=(f)=>{let t=f.getAttribute("data-sort-key")||"",r=f.getAttribute("aria-sort")||"none",H=r==="none"?"ascending":r==="ascending"?"descending":"none";u.querySelectorAll("th[data-sort-key]").forEach((C)=>{if(C!==f)C.setAttribute("aria-sort","none")}),f.setAttribute("aria-sort",H),n(),u.emit("sui-sort",{key:t,direction:H})},M=(f)=>{let t=f.target.closest("th[data-sort-key]");if(!t||!u.contains(t))return;E(t)},y=(f)=>{if(f.key!=="Enter"&&f.key!==" ")return;let t=f.target.closest("th[data-sort-key]");if(!t||!u.contains(t))return;f.preventDefault(),E(t)},L=(f)=>{let t=f.target.closest("td");if(!t||!u.contains(t))return;if(t.classList.contains("sui-table-gutter-cell"))return;let r=t.closest("tr");if(!r)return;let H=r.getAttribute("data-row-id");if(!H)return;u.emit("sui-row-click",{id:H,index:r.rowIndex})};c(u,"click",M),c(u,"click",L),c(u,"keydown",y),b(),w();let l=new MutationObserver(()=>{b(),w()});return l.observe(u,{childList:!0,subtree:!0}),()=>{l.disconnect()}}}),Dd=`
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
`;var G6=V("sui-table",{props:{variant:P.union(["slot","native","data"],{default:"slot",reflect:!0}),density:P.union(["compact","default","comfortable"],{default:"default",reflect:!0}),dense:P.bool({default:!1,reflect:!0}),hoverable:P.bool({default:!0,reflect:!0}),striped:P.bool({default:!1,reflect:!0}),"sticky-header":P.bool({default:!1,reflect:!0}),"sticky-first-col":P.bool({default:!1,reflect:!0}),gutter:P.bool({default:!1,reflect:!0}),label:P.string({default:""})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.variant=u.variant,n.density=u.density,n.dense=u.dense,n.hoverable=u.hoverable,n.striped=u.striped,n.stickyHeader=u.stickyHeader,n.stickyFirstCol=u.stickyFirstCol,n.gutter=u.gutter,n.label=u.label;let b=null;if(V0&&typeof V0.setup==="function"){if(i.scope=n,b=V0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(l)=>h({anchor:l,items:()=>n.columns(),key:(f,t)=>{let r=Object.create(n);return r.c=f,r.__i=t,((H)=>f.field)(r)},render:(f,t)=>{let r=Object.create(n);return r.c=f,r.__i=t,r.__v=J(0),((H)=>{let C=document.createDocumentFragment(),z=[],g=document.createElement("th");z.push(nn(g,"click",function(){let T=()=>H.sortBy(H.c.field);return typeof T==="function"?T.bind(void 0):()=>{}}(),{modifiers:[]})),z.push(Cn(g,"sorted",()=>(H.__v(),H.sortKey()===H.c.field)));let j=document.createTextNode("");return g.appendChild(j),j.textContent=H.c.label,C.appendChild(g),{nodes:Array.from(C.childNodes),cleanup:()=>{for(let T of z)try{T()}catch(D){}},update:(T,D)=>{H.c=T,H.__i=D,j.textContent=H.c.label,H.__v.set((Z)=>Z+1)}}})(r)}}),E=(l)=>h({anchor:l,items:()=>n.sortedRows(),key:(f,t)=>{let r=Object.create(n);return r.r=f,r.__i=t,((H)=>f.id)(r)},render:(f,t)=>{let r=Object.create(n);return r.r=f,r.__i=t,r.__v=J(0),((H)=>{let C=(D)=>h({anchor:D,items:()=>H.columns(),key:(Z,X)=>{let G=Object.create(H);return G.c=Z,G.__i=X,((Q)=>Z.field)(G)},render:(Z,X)=>{let G=Object.create(H);return G.c=Z,G.__i=X,G.__v=J(0),((Q)=>{let q=document.createDocumentFragment(),F=[],k=document.createElement("td"),K=document.createTextNode("");return k.appendChild(K),K.textContent=Q.r[Q.c.field],q.appendChild(k),{nodes:Array.from(q.childNodes),cleanup:()=>{for(let B of F)try{B()}catch(S){}},update:(B,S)=>{Q.c=B,Q.__i=S,K.textContent=Q.r[Q.c.field],Q.__v.set((R)=>R+1)}}})(G)}}),z=document.createDocumentFragment(),g=[],j=document.createElement("tr");g.push(Cn(j,"selected",()=>(H.__v(),H.selected().includes(H.r.id)))),g.push(nn(j,"click",function(){let D=()=>H.toggleSelect(H.r.id);return typeof D==="function"?D.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createComment("for");return j.appendChild(T),g.push(C(T)),z.appendChild(j),{nodes:Array.from(z.childNodes),cleanup:()=>{for(let D of g)try{D()}catch(Z){}},update:(D,Z)=>{H.r=D,H.__i=Z,H.__v.set((X)=>X+1)}}})(r)}}),M=(l)=>I({anchor:l,test:()=>{if(n.variant()==="data")return 0;if(n.variant()!=="native")return 1;return 2},branches:[()=>{let f=document.createDocumentFragment(),t=[],r=document.createElement("div");r.setAttribute("class","wrap"),r.setAttribute("part","wrap");let H=document.createElement("table");H.setAttribute("part","table"),t.push(W(H,"aria-label",()=>n.label()||null));let C=document.createElement("thead");C.setAttribute("part","thead");let z=document.createElement("tr"),g=document.createComment("for");z.appendChild(g),t.push(w(g)),C.appendChild(z),H.appendChild(C);let j=document.createElement("tbody");j.setAttribute("part","tbody");let T=document.createComment("for");return j.appendChild(T),t.push(E(T)),H.appendChild(j),r.appendChild(H),f.appendChild(r),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let D of t)try{D()}catch(Z){}}}},()=>{let f=document.createDocumentFragment(),t=[],r=document.createElement("div");r.setAttribute("class","wrap"),r.setAttribute("part","wrap");let H=document.createElement("table");H.setAttribute("part","table"),t.push(W(H,"aria-label",()=>n.label()||null));let C=document.createElement("thead");C.setAttribute("part","thead");let z=document.createElement("slot");z.setAttribute("name","head"),C.appendChild(z),H.appendChild(C);let g=document.createElement("tbody");g.setAttribute("part","tbody");let j=document.createElement("slot");g.appendChild(j),H.appendChild(g),r.appendChild(H);let T=document.createElement("div");T.setAttribute("class","empty"),T.setAttribute("part","empty"),T.setAttribute("role","status");let D=document.createElement("slot");return D.setAttribute("name","empty"),T.appendChild(D),r.appendChild(T),f.appendChild(r),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let Z of t)try{Z()}catch(X){}}}},()=>{let f=document.createDocumentFragment(),t=[],r=document.createElement("div");r.setAttribute("class","wrap"),r.setAttribute("part","wrap");let H=document.createElement("slot");r.appendChild(H);let C=document.createElement("div");C.setAttribute("class","empty"),C.setAttribute("part","empty"),C.setAttribute("role","status");let z=document.createElement("slot");return z.setAttribute("name","empty"),C.appendChild(z),r.appendChild(C),f.appendChild(r),{nodes:Array.from(f.childNodes),cleanup:()=>{for(let g of t)try{g()}catch(j){}}}}]}),y=d.shadowRoot??d;Y(y);let L=document.createComment("if");if(y.appendChild(L),m.push(M(L)),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let f=new Proxy(n,{get(t,r){return t[r]},set(t,r,H){let C=t[r];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(l.mount){let t=l.mount(f,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(f))}}return n}});function tn(i){return String(i).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#39;")}function Zd(i,u){if(Array.isArray(i))return i;if(typeof i==="string"&&i.trim())try{let d=JSON.parse(i);return Array.isArray(d)?d:u}catch{return u}return u}function em(i,u,d,m){let n=new Map,b=i.some((f)=>typeof f.x!=="number"||typeof f.y!=="number");if(m==="positioned"&&!b){for(let f of i)n.set(f.id,{x:f.x,y:f.y});return n}let w=u*0.22,E=d*0.5,M=i.filter((f)=>f.kind==="agent"),y=i.filter((f)=>f.kind!=="agent");M.forEach((f,t)=>{let r=Math.max(1,M.length),H=E+(t-(r-1)/2)*80;n.set(f.id,{x:w,y:H})});let L=new Map;for(let f of y){let t=L.get(f.kind)??[];t.push(f),L.set(f.kind,t)}let l=Array.from(L.keys());return l.forEach((f,t)=>{let r=L.get(f),H=u*(0.5+(t+1)/(l.length+1)*0.45);r.forEach((C,z)=>{let g=d*(z+1)/(r.length+1);n.set(C.id,{x:H,y:g})})}),n}function sm(i,u){let d=Math.max(40,Math.abs(u.x-i.x)*0.4);return`M ${i.x} ${i.y} C ${i.x+d} ${i.y}, ${u.x-d} ${u.y}, ${u.x} ${u.y}`}function pm(i,u){let d=i.querySelectorAll('template[slot="node"]');for(let m of Array.from(d))if(m.dataset.kind===u)return m;return null}function cm(i,u){let d=pm(i,u.kind);if(!d){let m=u.sub?`<div class="node-sub">${tn(u.sub)}</div>`:"";return`<div class="node-h"><span class="tag">${tn(u.kind)}</span></div><div class="node-title">${tn(u.title??u.id)}</div>${m}`}return d.innerHTML.replace(/\$\{title\}/g,tn(u.title??"")).replace(/\$\{sub\}/g,tn(u.sub??"")).replace(/\$\{id\}/g,tn(u.id))}function _m(i,u,d,m,n,b,w){let E=Zd(u,[]),M=Zd(d,[]),y=em(E,m,n,b),L=M.map((f)=>{let t=y.get(f.from),r=y.get(f.to);if(!t||!r)return"";let H=tn(sm(t,r)),C=f.dashed?" data-dashed":"";return`<path d="${H}" data-kind="${tn(f.kind??"default")}"${C}></path>`}).join(""),l=E.map((f)=>{let t=y.get(f.id);if(!t)return"";let r=t.x/m*100,H=t.y/n*100,C=w&&w===f.id?" data-selected":"",z=f.w?`--node-w: ${f.w}px;`:"",g=tn(`${f.kind}: ${f.title??f.id}`),j=cm(i,f);return`<div class="node" part="node" data-kind="${tn(f.kind)}" data-id="${tn(f.id)}"${C} tabindex="0" role="button" aria-label="${g}" style="left: ${r}%; top: ${H}%;${z}">${j}</div>`}).join("");return`<svg class="edges" part="edges" viewBox="0 0 ${m} ${n}" preserveAspectRatio="none" aria-hidden="true">${L}</svg>`+l}var Y0=N({mount(i,u){let d=i,m=d.canvasEl();if(!m)return;let n=m.ownerDocument.createElement("div");m.appendChild(n);let b=mn(n,()=>{let L=d.nodes(),l=d.edges(),f=d.width()||1000,t=d.height()||600,r=d.layout()||"positioned",H=d.selectedId();return _m(u,L,l,f,t,r,H)}),w="",E=(L)=>{let l=L.target.closest(".node[data-id]");if(!l)return;let f=l.getAttribute("data-id")??"",t=l.getAttribute("data-kind")??"";u.setAttribute("selected-id",f),u.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:f,kind:t},bubbles:!0,composed:!0}))},M=(L)=>{if(L.key!=="Enter"&&L.key!==" ")return;let l=L.target.closest(".node[data-id]");if(!l)return;L.preventDefault();let f=l.getAttribute("data-id")??"",t=l.getAttribute("data-kind")??"";u.setAttribute("selected-id",f),u.dispatchEvent(new CustomEvent("sui-node-select",{detail:{id:f,kind:t},bubbles:!0,composed:!0}))},y=(L)=>{let l=L.target.closest(".node[data-id]");if(!l){w="";return}let f=l.getAttribute("data-id")??"";if(f===w)return;w=f;let t=l.getAttribute("data-kind")??"";u.dispatchEvent(new CustomEvent("sui-node-hover",{detail:{id:f,kind:t},bubbles:!0,composed:!0}))};return m.addEventListener("click",E),m.addEventListener("keydown",M),m.addEventListener("mouseover",y),()=>{b(),m.removeEventListener("click",E),m.removeEventListener("keydown",M),m.removeEventListener("mouseover",y)}}});var U6=V("sui-graph-canvas",{props:{nodes:P.string({default:""}),edges:P.string({default:""}),width:P.number({default:1000,reflect:!0}),height:P.number({default:600,reflect:!0}),layout:P.union(["positioned","radial"],{default:"positioned",reflect:!0}),"selected-id":P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.nodes=u.nodes,n.edges=u.edges,n.width=u.width,n.height=u.height,n.layout=u.layout,n.selectedId=u.selectedId,n.canvasEl=J(null);let b=null;if(Y0&&typeof Y0.setup==="function"){if(i.scope=n,b=Y0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("div");E.setAttribute("class","canvas"),E.setAttribute("part","canvas"),A(n,"canvasEl",E),w.appendChild(E);let M=document.createElement("slot");if(M.setAttribute("name","node"),M.setAttribute("hidden",""),w.appendChild(M),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let L=new Proxy(n,{get(l,f){return l[f]},set(l,f,t){let r=l[f];if(r!=null&&typeof r.set==="function"){if(typeof t==="function")r.set(()=>t);else r.set(t);return!0}return!1}});if(y.mount){let l=y.mount(L,d);if(typeof l==="function")m.push(l)}if(y.unmount)m.push(()=>y.unmount(L))}}return n}});var q0=N({mount(i,u){if(!u.hasAttribute("role"))u.setAttribute("role","list");if(!u.hasAttribute("aria-label"))u.setAttribute("aria-label","pipeline steps")}});var B6=V("sui-pipeline-dag",{shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null),b=null;if(q0&&typeof q0.setup==="function"){if(i.scope=n,b=q0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("slot");if(w.appendChild(E),b!=null){let M=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(M){let y=new Proxy(n,{get(L,l){return L[l]},set(L,l,f){let t=L[l];if(t!=null&&typeof t.set==="function"){if(typeof f==="function")t.set(()=>f);else t.set(f);return!0}return!1}});if(M.mount){let L=M.mount(y,d);if(typeof L==="function")m.push(L)}if(M.unmount)m.push(()=>M.unmount(y))}}return n}});var U0=N({mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","listitem");let m=U(()=>{let n=d.name(),b=d.stepType(),w=d.agent(),E=d.state(),M=`${b} step ${n}${w?` (${w})`:""}: ${E}`;u.setAttribute("aria-label",M)});return()=>{m()}}});var v6=V("sui-pipeline-step",{props:{name:P.string({default:"",reflect:!0}),"step-type":P.union(["agent","gate","merge","run","human"],{default:"agent",reflect:!0}),state:P.union(["pending","running","completed","failed"],{default:"pending",reflect:!0}),agent:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.stepType=u.stepType,n.state=u.state,n.agent=u.agent;let b=null;if(U0&&typeof U0.setup==="function"){if(i.scope=n,b=U0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(z)=>I({anchor:z,test:()=>{if(n.state()==="running")return 0;return 1},branches:[()=>{let g=document.createDocumentFragment(),j=[],T=document.createElement("sui-spinner");return T.setAttribute("size","sm"),T.setAttribute("tone","info"),T.setAttribute("label","Running"),g.appendChild(T),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let D of j)try{D()}catch(Z){}}}},()=>{let g=document.createDocumentFragment(),j=[],T=document.createElement("span");T.setAttribute("class","icon"),T.setAttribute("aria-hidden","true");let D=document.createTextNode("");return T.appendChild(D),j.push(a(D,()=>n.state()==="completed"?"✓":n.state()==="failed"?"✗":"○")),g.appendChild(T),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let Z of j)try{Z()}catch(X){}}}}]}),E=(z)=>I({anchor:z,test:()=>{if(n.agent())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),j=[],T=document.createTextNode(" · ");g.appendChild(T);let D=document.createTextNode("");return g.appendChild(D),j.push(a(D,()=>n.agent())),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let Z of j)try{Z()}catch(X){}}}}]}),M=d.shadowRoot??d;Y(M);let y=document.createComment("if");M.appendChild(y),m.push(w(y));let L=document.createElement("span");L.setAttribute("class","name");let l=document.createTextNode("");L.appendChild(l),m.push(a(l,()=>n.name())),M.appendChild(L);let f=document.createElement("span");f.setAttribute("class","type");let t=document.createTextNode("");f.appendChild(t),m.push(a(t,()=>n.stepType()));let r=document.createComment("if");f.appendChild(r),m.push(E(r)),M.appendChild(f);let H=document.createElement("span");H.setAttribute("class","status");let C=document.createTextNode("");if(H.appendChild(C),m.push(a(C,()=>n.state())),M.appendChild(H),b!=null){let z=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(z){let g=new Proxy(n,{get(j,T){return j[T]},set(j,T,D){let Z=j[T];if(Z!=null&&typeof Z.set==="function"){if(typeof D==="function")Z.set(()=>D);else Z.set(D);return!0}return!1}});if(z.mount){let j=z.mount(g,d);if(typeof j==="function")m.push(j)}if(z.unmount)m.push(()=>z.unmount(g))}}return n}});var nb={dispatched:"info",running:"info",gatepending:"warning",completed:"success",failed:"destructive",merged:"accent",pending:"muted"},N0=N({scope:(i)=>{let u=i;return{tone:s(()=>nb[u.state()]??"muted")}},mount(i,u){let d=i;if(!u.hasAttribute("role"))u.setAttribute("role","article");let m=U(()=>{let n=d.slug(),b=d.agent(),w=d.state();u.setAttribute("aria-label",`Pipeline run ${n||""}${b?` on ${b}`:""}: ${w}`)});return()=>{m()}}});var s6=V("sui-pipeline-run",{props:{state:P.union(["dispatched","gatepending","running","completed","failed","merged","pending"],{default:"pending",reflect:!0}),slug:P.string({default:"",reflect:!0}),agent:P.string({default:"",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.state=u.state,n.slug=u.slug,n.agent=u.agent;let b=null;if(N0&&typeof N0.setup==="function"){if(i.scope=n,b=N0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(D)=>I({anchor:D,test:()=>{if(n.agent())return 0;return-1},branches:[()=>{let Z=document.createDocumentFragment(),X=[],G=document.createElement("span");G.setAttribute("class","agent");let Q=document.createElement("code"),q=document.createTextNode("");return Q.appendChild(q),X.push(a(q,()=>n.agent())),G.appendChild(Q),Z.appendChild(G),{nodes:Array.from(Z.childNodes),cleanup:()=>{for(let F of X)try{F()}catch(k){}}}}]}),E=d.shadowRoot??d;Y(E);let M=document.createElement("sui-card");M.setAttribute("status-rail",""),m.push(W(M,"tone",()=>n.tone()));let y=document.createElement("div");y.setAttribute("slot","header");let L=document.createElement("header"),l=document.createElement("div");l.setAttribute("class","left");let f=document.createElement("span");f.setAttribute("class","slug");let t=document.createTextNode("");f.appendChild(t),m.push(a(t,()=>n.slug())),l.appendChild(f);let r=document.createComment("if");l.appendChild(r),m.push(w(r));let H=document.createElement("slot");H.setAttribute("name","actions"),l.appendChild(H),L.appendChild(l);let C=document.createElement("sui-badge");m.push(W(C,"tone",()=>n.tone()));let z=document.createTextNode("");C.appendChild(z),m.push(a(z,()=>n.state())),L.appendChild(C),y.appendChild(L),M.appendChild(y);let g=document.createElement("slot");M.appendChild(g);let j=document.createElement("div");j.setAttribute("slot","footer");let T=document.createElement("slot");if(T.setAttribute("name","footer"),j.appendChild(T),M.appendChild(j),E.appendChild(M),b!=null){let D=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(D){let Z=new Proxy(n,{get(X,G){return X[G]},set(X,G,Q){let q=X[G];if(q!=null&&typeof q.set==="function"){if(typeof Q==="function")q.set(()=>Q);else q.set(Q);return!0}return!1}});if(D.mount){let X=D.mount(Z,d);if(typeof X==="function")m.push(X)}if(D.unmount)m.push(()=>D.unmount(Z))}}return n}});var ib={run:"muted",agent:"info",gate:"warning",merge:"accent",human:"warning"},A0=N({scope:(i)=>{let u=i,d=s(()=>ib[u.stepType()]??"muted"),m=s(()=>(u.after()||"").split(",").map((b)=>b.trim()).filter(Boolean)),n=s(()=>(u.command()||"").trim());return{kindTone:d,afterList:m,commandTrim:n}}});var uM=V("sui-pipeline-step-def",{props:{name:P.string({default:"",reflect:!0}),"step-type":P.union(["agent","gate","run","merge","human"],{default:"agent",reflect:!0}),agent:P.string({default:"",reflect:!0}),provider:P.string({default:"",reflect:!0}),model:P.string({default:"",reflect:!0}),after:P.string({default:"",reflect:!0}),command:P.string({default:""}),index:P.number({default:0,reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.name=u.name,n.stepType=u.stepType,n.agent=u.agent,n.provider=u.provider,n.model=u.model,n.after=u.after,n.command=u.command,n.index=u.index;let b=null;if(A0&&typeof A0.setup==="function"){if(i.scope=n,b=A0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=(K)=>h({anchor:K,items:()=>n.afterList(),key:(B,S)=>{let R=Object.create(n);return R.a=B,R.idx=S,((v)=>B)(R)},render:(B,S)=>{let R=Object.create(n);return R.a=B,R.idx=S,R.__v=J(0),((v)=>{let O=(fn)=>I({anchor:fn,test:()=>{if(v.idx>0)return 0;return-1},branches:[()=>{let rn=document.createDocumentFragment(),zn=[],K0=document.createTextNode(", ");return rn.appendChild(K0),{nodes:Array.from(rn.childNodes),cleanup:()=>{for(let On of zn)try{On()}catch(on){}}}}]}),x=document.createDocumentFragment(),$=[],o=document.createComment("if");x.appendChild(o),$.push(O(o));let bn=document.createElement("code"),Tn=document.createTextNode("");return bn.appendChild(Tn),Tn.textContent=v.a,x.appendChild(bn),{nodes:Array.from(x.childNodes),cleanup:()=>{for(let fn of $)try{fn()}catch(rn){}},update:(fn,rn)=>{v.a=fn,v.idx=rn,Tn.textContent=v.a,v.__v.set((zn)=>zn+1)}}})(R)}}),E=(K)=>I({anchor:K,test:()=>{if(n.afterList().length>0)return 0;return-1},branches:[()=>{let B=document.createDocumentFragment(),S=[],R=document.createElement("span");R.setAttribute("class","after");let v=document.createTextNode("after ");R.appendChild(v);let O=document.createComment("for");return R.appendChild(O),S.push(w(O)),B.appendChild(R),{nodes:Array.from(B.childNodes),cleanup:()=>{for(let x of S)try{x()}catch($){}}}}]}),M=(K)=>I({anchor:K,test:()=>{if(n.agent())return 0;return-1},branches:[()=>{let B=document.createDocumentFragment(),S=[],R=document.createElement("sui-chip");R.setAttribute("size","sm");let v=document.createTextNode("");return R.appendChild(v),S.push(a(v,()=>n.agent())),B.appendChild(R),{nodes:Array.from(B.childNodes),cleanup:()=>{for(let O of S)try{O()}catch(x){}}}}]}),y=(K)=>I({anchor:K,test:()=>{if(n.provider())return 0;return-1},branches:[()=>{let B=document.createDocumentFragment(),S=[],R=document.createElement("sui-chip");R.setAttribute("size","sm");let v=document.createTextNode("");return R.appendChild(v),S.push(a(v,()=>n.provider())),B.appendChild(R),{nodes:Array.from(B.childNodes),cleanup:()=>{for(let O of S)try{O()}catch(x){}}}}]}),L=(K)=>I({anchor:K,test:()=>{if(n.model())return 0;return-1},branches:[()=>{let B=document.createDocumentFragment(),S=[],R=document.createElement("sui-chip");R.setAttribute("size","sm");let v=document.createTextNode("");return R.appendChild(v),S.push(a(v,()=>n.model())),B.appendChild(R),{nodes:Array.from(B.childNodes),cleanup:()=>{for(let O of S)try{O()}catch(x){}}}}]}),l=(K)=>I({anchor:K,test:()=>{if(n.agent()||n.provider()||n.model())return 0;return-1},branches:[()=>{let B=document.createDocumentFragment(),S=[],R=document.createElement("div");R.setAttribute("class","chips");let v=document.createComment("if");R.appendChild(v),S.push(M(v));let O=document.createComment("if");R.appendChild(O),S.push(y(O));let x=document.createComment("if");return R.appendChild(x),S.push(L(x)),B.appendChild(R),{nodes:Array.from(B.childNodes),cleanup:()=>{for(let $ of S)try{$()}catch(o){}}}}]}),f=(K)=>I({anchor:K,test:()=>{if(n.commandTrim())return 0;return-1},branches:[()=>{let B=document.createDocumentFragment(),S=[],R=document.createElement("pre");R.setAttribute("part","command");let v=document.createTextNode("");return R.appendChild(v),S.push(a(v,()=>n.commandTrim())),B.appendChild(R),{nodes:Array.from(B.childNodes),cleanup:()=>{for(let O of S)try{O()}catch(x){}}}}]}),t=d.shadowRoot??d;Y(t);let r=document.createElement("div");r.setAttribute("part","row");let H=document.createElement("span");H.setAttribute("class","index");let C=document.createTextNode("");H.appendChild(C),m.push(a(C,()=>n.index()||"")),r.appendChild(H);let z=document.createElement("div");z.setAttribute("class","body");let g=document.createElement("div");g.setAttribute("part","header");let j=document.createElement("span");j.setAttribute("class","name");let T=document.createTextNode("");j.appendChild(T),m.push(a(T,()=>n.name())),g.appendChild(j);let D=document.createElement("sui-chip");D.setAttribute("size","sm"),m.push(W(D,"tone",()=>n.kindTone()));let Z=document.createTextNode("");D.appendChild(Z),m.push(a(Z,()=>n.stepType())),g.appendChild(D);let X=document.createComment("if");g.appendChild(X),m.push(E(X));let G=document.createElement("span");G.setAttribute("class","trailing");let Q=document.createElement("slot");Q.setAttribute("name","trailing"),G.appendChild(Q),g.appendChild(G),z.appendChild(g);let q=document.createComment("if");z.appendChild(q),m.push(l(q));let F=document.createComment("if");z.appendChild(F),m.push(f(F));let k=document.createElement("slot");if(k.setAttribute("name","gate"),z.appendChild(k),r.appendChild(z),t.appendChild(r),b!=null){let K=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(K){let B=new Proxy(n,{get(S,R){return S[R]},set(S,R,v){let O=S[R];if(O!=null&&typeof O.set==="function"){if(typeof v==="function")O.set(()=>v);else O.set(v);return!0}return!1}});if(K.mount){let S=K.mount(B,d);if(typeof S==="function")m.push(S)}if(K.unmount)m.push(()=>K.unmount(B))}}return n}});var F0=N({scope:()=>{return{preEl:e(null)}},mount(i,u){let d=i,m=null,n="",b=!0,w=()=>{let t=d.preEl();if(!t)return;if(n.length===0)t.textContent="(no output yet)",t.setAttribute("data-empty","");else t.textContent=n,t.removeAttribute("data-empty");if(d.autoscroll()&&b)t.scrollTop=t.scrollHeight},E=(t)=>{n+=t;let r=d.maxBytes();if(n.length>r){let H=n.length-r,C=n.indexOf(`
`,H);n=n.slice(C>=0?C+1:H)}w()},M=()=>{if(m)m.close(),m=null,u.dispatchEvent(new CustomEvent("sui-stream-close",{bubbles:!0,composed:!0}))},y=()=>{let t=d.url();if(!t||m)return;try{m=new EventSource(t,{withCredentials:!0})}catch{u.dispatchEvent(new CustomEvent("sui-stream-error",{bubbles:!0,composed:!0}));return}m.addEventListener(d.event(),(r)=>{let H=r.data??"";E(H),u.dispatchEvent(new CustomEvent("sui-log-tick",{detail:{bytes:H.length},bubbles:!0,composed:!0}))}),m.onopen=()=>{u.dispatchEvent(new CustomEvent("sui-stream-open",{bubbles:!0,composed:!0}))},m.onerror=()=>{u.dispatchEvent(new CustomEvent("sui-stream-error",{bubbles:!0,composed:!0}))}},L=(t)=>{let r=t.currentTarget;b=r.scrollHeight-r.clientHeight<=r.scrollTop+4},l=U(()=>{let t=d.preEl();if(!t)return;return t.addEventListener("scroll",L),w(),()=>t.removeEventListener("scroll",L)}),f=U(()=>{if(d.url(),d.event(),d.active()){if(m)M();y()}else M()});return u.clear=()=>{n="",w()},()=>{l(),f(),M()}}});var rM=V("sui-log-stream",{props:{url:P.string({default:"",reflect:!0}),event:P.string({default:"log",reflect:!0}),autoscroll:P.bool({default:!1,reflect:!0}),active:P.bool({default:!1,reflect:!0}),"max-bytes":P.number({default:1e6})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.url=u.url,n.event=u.event,n.autoscroll=u.autoscroll,n.active=u.active,n.maxBytes=u.maxBytes,n.preEl=J(null);let b=null;if(F0&&typeof F0.setup==="function"){if(i.scope=n,b=F0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("pre");E.setAttribute("part","scroll"),E.setAttribute("data-empty",""),A(n,"preEl",E);let M=document.createTextNode("(no output yet)");if(E.appendChild(M),w.appendChild(E),b!=null){let y=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(y){let L=new Proxy(n,{get(l,f){return l[f]},set(l,f,t){let r=l[f];if(r!=null&&typeof r.set==="function"){if(typeof t==="function")r.set(()=>t);else r.set(t);return!0}return!1}});if(y.mount){let l=y.mount(L,d);if(typeof l==="function")m.push(l)}if(y.unmount)m.push(()=>y.unmount(L))}}return n}});var Zn={claude:{label:"Claude",hue:"oklch(0.70 0.16 35)",svg:'<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="#D97757"><title>Claude</title><path d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/></svg>'},codex:{label:"Codex",hue:"oklch(0.22 0.02 220)",svg:'<svg width="721" height="721" viewBox="0 0 721 721" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1637_2934)"><g clip-path="url(#clip1_1637_2934)"><path d="M304.246 294.611V249.028C304.246 245.189 305.687 242.309 309.044 240.392L400.692 187.612C413.167 180.415 428.042 177.058 443.394 177.058C500.971 177.058 537.44 221.682 537.44 269.182C537.44 272.54 537.44 276.379 536.959 280.218L441.954 224.558C436.197 221.201 430.437 221.201 424.68 224.558L304.246 294.611ZM518.245 472.145V363.224C518.245 356.505 515.364 351.707 509.608 348.349L389.174 278.296L428.519 255.743C431.877 253.826 434.757 253.826 438.115 255.743L529.762 308.523C556.154 323.879 573.905 356.505 573.905 388.171C573.905 424.636 552.315 458.225 518.245 472.141V472.145ZM275.937 376.182L236.592 353.152C233.235 351.235 231.794 348.354 231.794 344.515V238.956C231.794 187.617 271.139 148.749 324.4 148.749C344.555 148.749 363.264 155.468 379.102 167.463L284.578 222.164C278.822 225.521 275.942 230.319 275.942 237.039V376.186L275.937 376.182ZM360.626 425.122L304.246 393.455V326.283L360.626 294.616L417.002 326.283V393.455L360.626 425.122ZM396.852 570.989C376.698 570.989 357.989 564.27 342.151 552.276L436.674 497.574C442.431 494.217 445.311 489.419 445.311 482.699V343.552L485.138 366.582C488.495 368.499 489.936 371.379 489.936 375.219V480.778C489.936 532.117 450.109 570.985 396.852 570.985V570.989ZM283.134 463.99L191.486 411.211C165.094 395.854 147.343 363.229 147.343 331.562C147.343 294.616 169.415 261.509 203.48 247.593V356.991C203.48 363.71 206.361 368.508 212.117 371.866L332.074 441.437L292.729 463.99C289.372 465.907 286.491 465.907 283.134 463.99ZM277.859 542.68C223.639 542.68 183.813 501.895 183.813 451.514C183.813 447.675 184.294 443.836 184.771 439.997L279.295 494.698C285.051 498.056 290.812 498.056 296.568 494.698L417.002 425.127V470.71C417.002 474.549 415.562 477.429 412.204 479.346L320.557 532.126C308.081 539.323 293.206 542.68 277.854 542.68H277.859ZM396.852 599.776C454.911 599.776 503.37 558.513 514.41 503.812C568.149 489.896 602.696 439.515 602.696 388.176C602.696 354.587 588.303 321.962 562.392 298.45C564.791 288.373 566.231 278.296 566.231 268.224C566.231 199.611 510.571 148.267 446.274 148.267C433.322 148.267 420.846 150.184 408.37 154.505C386.775 133.392 357.026 119.958 324.4 119.958C266.342 119.958 217.883 161.22 206.843 215.921C153.104 229.837 118.557 280.218 118.557 331.557C118.557 365.146 132.95 397.771 158.861 421.283C156.462 431.36 155.022 441.437 155.022 451.51C155.022 520.123 210.682 571.466 274.978 571.466C287.931 571.466 300.407 569.549 312.883 565.228C334.473 586.341 364.222 599.776 396.852 599.776Z" fill="black"/></g></g><defs><clipPath id="clip0_1637_2934"><rect width="720" height="720" fill="white" transform="translate(0.606934 0.0999756)"/></clipPath><clipPath id="clip1_1637_2934"><rect width="484.139" height="479.818" fill="white" transform="translate(118.557 119.958)"/></clipPath></defs></svg>',svgDark:'<svg width="721" height="721" viewBox="0 0 721 721" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1637_2935)"><g clip-path="url(#clip1_1637_2935)"><path d="M304.246 295.411V249.828C304.246 245.989 305.687 243.109 309.044 241.191L400.692 188.412C413.167 181.215 428.042 177.858 443.394 177.858C500.971 177.858 537.44 222.482 537.44 269.982C537.44 273.34 537.44 277.179 536.959 281.018L441.954 225.358C436.197 222 430.437 222 424.68 225.358L304.246 295.411ZM518.245 472.945V364.024C518.245 357.304 515.364 352.507 509.608 349.149L389.174 279.096L428.519 256.543C431.877 254.626 434.757 254.626 438.115 256.543L529.762 309.323C556.154 324.679 573.905 357.304 573.905 388.971C573.905 425.436 552.315 459.024 518.245 472.941V472.945ZM275.937 376.982L236.592 353.952C233.235 352.034 231.794 349.154 231.794 345.315V239.756C231.794 188.416 271.139 149.548 324.4 149.548C344.555 149.548 363.264 156.268 379.102 168.262L284.578 222.964C278.822 226.321 275.942 231.119 275.942 237.838V376.986L275.937 376.982ZM360.626 425.922L304.246 394.255V327.083L360.626 295.416L417.002 327.083V394.255L360.626 425.922ZM396.852 571.789C376.698 571.789 357.989 565.07 342.151 553.075L436.674 498.374C442.431 495.017 445.311 490.219 445.311 483.499V344.352L485.138 367.382C488.495 369.299 489.936 372.179 489.936 376.018V481.577C489.936 532.917 450.109 571.785 396.852 571.785V571.789ZM283.134 464.79L191.486 412.01C165.094 396.654 147.343 364.029 147.343 332.362C147.343 295.416 169.415 262.309 203.48 248.393V357.791C203.48 364.51 206.361 369.308 212.117 372.665L332.074 442.237L292.729 464.79C289.372 466.707 286.491 466.707 283.134 464.79ZM277.859 543.48C223.639 543.48 183.813 502.695 183.813 452.314C183.813 448.475 184.294 444.636 184.771 440.797L279.295 495.498C285.051 498.856 290.812 498.856 296.568 495.498L417.002 425.927V471.509C417.002 475.349 415.562 478.229 412.204 480.146L320.557 532.926C308.081 540.122 293.206 543.48 277.854 543.48H277.859ZM396.852 600.576C454.911 600.576 503.37 559.313 514.41 504.612C568.149 490.696 602.696 440.315 602.696 388.976C602.696 355.387 588.303 322.762 562.392 299.25C564.791 289.173 566.231 279.096 566.231 269.024C566.231 200.411 510.571 149.067 446.274 149.067C433.322 149.067 420.846 150.984 408.37 155.305C386.775 134.192 357.026 120.758 324.4 120.758C266.342 120.758 217.883 162.02 206.843 216.721C153.104 230.637 118.557 281.018 118.557 332.357C118.557 365.946 132.95 398.571 158.861 422.083C156.462 432.16 155.022 442.237 155.022 452.309C155.022 520.922 210.682 572.266 274.978 572.266C287.931 572.266 300.407 570.349 312.883 566.028C334.473 587.141 364.222 600.576 396.852 600.576Z" fill="white"/></g></g><defs><clipPath id="clip0_1637_2935"><rect width="720" height="720" fill="white" transform="translate(0.606934 0.899902)"/></clipPath><clipPath id="clip1_1637_2935"><rect width="484.139" height="479.818" fill="white" transform="translate(118.557 120.758)"/></clipPath></defs></svg>'},cursor:{label:"Cursor",hue:"oklch(0.45 0.02 250)",svg:'<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor"><title>Cursor</title><path d="M11.925.0078 24 6.9702v10.0562L11.925 24 0 17.0288V6.9702ZM2.64 8.529 12 13.924v9.6448l9.36-5.3962V7.529l-9.36 5.4ZM10.677 11.11 2.64 6.4702l9.36-5.4038 8.01 4.6212Z"/></svg>'},gemini:{label:"Gemini",hue:"oklch(0.62 0.14 285)",svg:'<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="#8E75B2"><title>Google Gemini</title><path d="M11.04 19.32Q12 21.51 12 24q0-2.49.93-4.68.96-2.19 2.58-3.81t3.81-2.55Q21.51 12 24 12q-2.49 0-4.68-.93a12.3 12.3 0 0 1-3.81-2.58 12.3 12.3 0 0 1-2.58-3.81Q12 2.49 12 0q0 2.49-.96 4.68-.93 2.19-2.55 3.81a12.3 12.3 0 0 1-3.81 2.58Q2.49 12 0 12q2.49 0 4.68.96 2.19.93 3.81 2.55t2.55 3.81"/></svg>'},opencode:{label:"OpenCode",hue:"oklch(0.60 0.18 35)",svg:'<svg width="240" height="300" viewBox="0 0 240 300" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1401_86274)"><mask id="mask0_1401_86274" style="mask-type:luminance" maskUnits="userSpaceOnUse" x="0" y="0" width="240" height="300"><path d="M240 0H0V300H240V0Z" fill="white"/></mask><g mask="url(#mask0_1401_86274)"><path d="M180 240H60V120H180V240Z" fill="#CFCECD"/><path d="M180 60H60V240H180V60ZM240 300H0V0H240V300Z" fill="#211E1E"/></g></g><defs><clipPath id="clip0_1401_86274"><rect width="240" height="300" fill="white"/></clipPath></defs></svg>',svgDark:'<svg width="240" height="300" viewBox="0 0 240 300" fill="none" xmlns="http://www.w3.org/2000/svg"><g clip-path="url(#clip0_1401_86283)"><mask id="mask0_1401_86283" style="mask-type:luminance" maskUnits="userSpaceOnUse" x="0" y="0" width="240" height="300"><path d="M240 0H0V300H240V0Z" fill="white"/></mask><g mask="url(#mask0_1401_86283)"><path d="M180 240H60V120H180V240Z" fill="#4B4646"/><path d="M180 60H60V240H180V60ZM240 300H0V0H240V300Z" fill="#F1ECEC"/></g></g><defs><clipPath id="clip0_1401_86283"><rect width="240" height="300" fill="white"/></clipPath></defs></svg>'}};var O0=N({scope:(i)=>{let u=i,d=e(null),m=s(()=>(Zn[u.provider()]??Zn.claude).label);return{tileEl:d,tileLabel:m}},mount(i,u){let d=i,m=U(()=>{let b=Zn[d.provider()]??Zn.claude;u.style.setProperty("--blob-hue",b.hue)}),n=U(()=>{let b=d.tileEl();if(!b)return;return mn(b,()=>{return(Zn[d.provider()]??Zn.claude).svg})});return()=>{m(),n()}}});var CM=V("sui-provider-blob",{props:{provider:P.union(["claude","codex","cursor","gemini","opencode"],{default:"claude",reflect:!0}),size:P.union(["sm","md","lg"],{default:"md",reflect:!0}),variant:P.union(["tile","glyph"],{default:"tile",reflect:!0})},shadow:!0,styles:`
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
`,setup(i){let{props:u,host:d}=i;if(!i.cleanups)i.cleanups=[];let m=i.cleanups,n=Object.create(null);n.provider=u.provider,n.size=u.size,n.variant=u.variant,n.tileEl=J(null);let b=null;if(O0&&typeof O0.setup==="function"){if(i.scope=n,b=O0.setup(i),b&&typeof b==="object")Object.assign(n,b)}let w=d.shadowRoot??d;Y(w);let E=document.createElement("span");E.setAttribute("class","tile"),E.setAttribute("role","img"),m.push(W(E,"aria-label",()=>n.tileLabel()));let M=document.createElement("span");M.setAttribute("class","tile-inner"),A(n,"tileEl",M),E.appendChild(M),w.appendChild(E);let y=document.createElement("span");y.setAttribute("class","label");let L=document.createElement("slot");if(y.appendChild(L),w.appendChild(y),b!=null){let l=Object.getOwnPropertyDescriptor(b,"__shipSetLifecycle")?.value;if(l){let f=new Proxy(n,{get(t,r){return t[r]},set(t,r,H){let C=t[r];if(C!=null&&typeof C.set==="function"){if(typeof H==="function")C.set(()=>H);else C.set(H);return!0}return!1}});if(l.mount){let t=l.mount(f,d);if(typeof t==="function")m.push(t)}if(l.unmount)m.push(()=>l.unmount(f))}}return n}});
