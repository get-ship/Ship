var S0=Symbol.for("sui:error-boundary");function OJ(J){let X=J;while(X){if(X instanceof Element){let Q=X[S0];if(Q)return Q}let Z=X.parentNode;if(Z){X=Z;continue}let z=X;if(z.host){X=z.host;continue}X=null}return null}function TJ(J,X,Z){try{return Z()}catch(z){let Q=OJ(J);if(Q){Q.catch(z,X);return}console.error("[sui] uncaught throw (no error boundary):",z);return}}var PJ=null,KJ=null,y0=0,b0=new Set,wJ=[];function sJ(J,X){wJ.push(J);try{return X()}finally{wJ.pop()}}function $0(){return wJ.length>0?wJ[wJ.length-1]:null}function q0(J){if(PJ===null)return;if(J.has(PJ))return;let X=PJ;if(J.add(X),KJ!==null)KJ.push(()=>J.delete(X))}function S(J){let X=J,Z=new Set,z=()=>{return q0(Z),X};return z.set=(Q)=>{let j=typeof Q==="function"?Q(X):Q;if(Object.is(j,X))return;if(X=j,y0>0)for(let q of Z)b0.add(q);else for(let q of[...Z])q()},z.__subs=Z,z.__peek=()=>X,z}function QJ(J){let X,Z=!0,z=new Set,Q=[],j=()=>{if(Z)return;Z=!0;for(let B of[...z])B()},q=()=>{for(let R of Q)R();Q=[];let B=PJ,D=KJ;PJ=j,KJ=Q;try{X=J()}finally{PJ=B,KJ=D}Z=!1},K=()=>{if(q0(z),Z)q();return X};return K.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},K.__subs=z,K.__peek=()=>{if(Z)q();return X},K}function l(J){let X=void 0,Z=!0,z=[],Q=$0(),j=()=>{if(!Z)return;if(X){try{X()}catch(B){console.error("[sui] effect cleanup threw:",B)}X=void 0}for(let B of z)B();z=[];let q=PJ,K=KJ;PJ=j,KJ=z;try{X=J()}catch(B){let D=Q?OJ(Q):null;if(D)D.catch(B,"effect");else console.error("[sui] effect threw:",B)}finally{PJ=q,KJ=K}};return j(),()=>{Z=!1;for(let q of z)q();if(z=[],X){try{X()}catch(q){console.error("[sui] effect cleanup threw:",q)}X=void 0}}}var gJ=null;function kJ(J,X){let Z=gJ;gJ=J;try{return X()}finally{gJ=Z}}function xJ(J){let X=gJ;if(!X)return;X.cleanup(J)}function SJ(){let J=[],X=!1;return{cleanup(z){if(X){try{z()}catch(Q){console.error("[sui/scope] late cleanup threw:",Q)}return}J.push(z)},dispose(){if(X)return;X=!0;for(let z=J.length-1;z>=0;z--)try{J[z]()}catch(Q){console.error("[sui/scope] cleanup threw:",Q)}J.length=0},get disposed(){return X}}}var V0=Symbol("sui.shared");function t(J){return J[V0]=!0,J}function M0(J){return!!J?.[V0]}function _J(J,X,Z){J[`__sui_ctx_${X}`]=Z}function yJ(J,X){let Z=J;while(Z){let z=Z[`__sui_ctx_${X}`];if(z!==void 0)return z;let Q=Z.parentElement;if(Q){Z=Q;continue}let j=Z.getRootNode();Z=j instanceof ShadowRoot?j.host:null}return null}function tJ(J){let X={};for(let Z in J)if(Object.prototype.hasOwnProperty.call(J,Z)&&M0(J[Z]))X[Z]=J[Z];return X}function eJ(J=null){return S(null)}function J0(J,X,Z){let z=J[X];if(typeof z==="function"&&typeof z.set==="function")return z.set(Z),z;let Q=S(Z);return J[X]=Q,Q}function g(J,X){let Z;return l(()=>{let z=X(),Q=z==null?"":String(z);if(Q===Z)return;if(Z===void 0&&J.textContent===Q){Z=Q;return}J.textContent=Q,Z=Q})}function JJ(J,X,Z){let z,Q=!1;return l(()=>{let j=Z();if(Q&&j===z)return;let q=j==null||j===!1||j===""?null:j===!0?"":String(j);if(Q||J.getAttribute(X)!==q)if(q===null)J.removeAttribute(X);else J.setAttribute(X,q);z=j,Q=!0})}function EJ(J,X,Z){let z;return l(()=>{let Q=!!Z();if(Q===z)return;if(z===void 0&&Q===J.classList.contains(X)){z=Q;return}if(Q)J.classList.add(X);else J.classList.remove(X);z=Q})}function HJ(J,X,Z){let z=l(()=>{let j=X();if(J.value!==j)J.value=j}),Q=()=>{let j=J.value;if(X.set(j),Z)Z(j)};return J.addEventListener("input",Q),()=>{z(),J.removeEventListener("input",Q)}}function XJ(J,X,Z,z={}){let Q=z.modifiers??[],j,q=!0,K=(D)=>{for(let O of Q)switch(O.kind){case"preventDefault":D.preventDefault();break;case"stopPropagation":D.stopPropagation();break;case"stopImmediatePropagation":D.stopImmediatePropagation();break;case"self":if(D.target!==J)return;break;case"trusted":if(!D.isTrusted)return;break;case"key":{if(!D0(D,O.arg??""))return;break}}let R=Q.find((O)=>O.kind==="debounce"),b=Q.find((O)=>O.kind==="throttle");if(R){let O=Number(R.arg)||0;if(j!==void 0)clearTimeout(j);j=setTimeout(()=>{j=void 0,Z(D)},O);return}if(b){let O=Number(b.arg)||0;if(!q)return;q=!1,Z(D),j=setTimeout(()=>{q=!0},O);return}Z(D)},B={};if(z.capture)B.capture=!0;if(z.passive)B.passive=!0;if(z.once)B.once=!0;return J.addEventListener(X,K,B),()=>{if(J.removeEventListener(X,K,B),j!==void 0)clearTimeout(j)}}function D0(J,X){let Z=X.split("+").map((R)=>R.toLowerCase()),z=Z.pop()??"",Q=Z.includes("meta")||Z.includes("cmd"),j=Z.includes("ctrl")||Z.includes("control"),q=Z.includes("shift"),K=Z.includes("alt")||Z.includes("option"),B=Z.includes("mod"),D=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(Q&&!J.metaKey)return!1;if(j&&!J.ctrlKey)return!1;if(q&&!J.shiftKey)return!1;if(K&&!J.altKey)return!1;if(B&&!(D?J.metaKey:J.ctrlKey))return!1;return J.key.toLowerCase()===z}function H0(J,X,Z){J.dispatchEvent(new CustomEvent(X,{detail:Z,bubbles:!0,composed:!0}))}function P0(J,X){let Z=J,z=Object.keys(X);for(let Q of z)Z[Q]=X[Q];xJ(()=>{for(let Q of z)if(Z[Q]===X[Q])try{delete Z[Q]}catch{Z[Q]=void 0}})}function uJ(J){let X=J;X.emit=function(Z,z){H0(J,Z,z)},X.expose=function(Z){P0(J,Z)}}var f0=Symbol("sui:transition");function hJ(J){return J[f0]}function v0(){if(typeof window>"u"||typeof window.matchMedia!=="function")return!1;try{return window.matchMedia("(prefers-reduced-motion: reduce)").matches}catch{return!1}}function U0(J){if(!v0())return J;return{...J,duration:0,delay:0}}function K0(J){let X=hJ(J);if(!X?.inT?.in)return;X.current?.cancel();let Z=X.inT.in(J,U0(X.inOpts));return X.current=Z,Z}function B0(J){let X=hJ(J);if(!X?.outT?.out)return;X.current?.cancel();let Z=X.outT.out(J,U0(X.outOpts));return X.current=Z,Z}function X0(J){if(J.nodeType!==1)return;let X=J;if(X.hasAttribute("data-sui-transition")){let Z=hJ(X);if(Z?.current?.cancel(),Z)Z.current=void 0}for(let Z of X.querySelectorAll("[data-sui-transition]")){let z=hJ(Z);if(z?.current?.cancel(),z)z.current=void 0}}function O0(J,X){for(let Z of J){if(Z.nodeType!==1)continue;let z=Z;if(z.hasAttribute("data-sui-transition"))X(z);for(let Q of z.querySelectorAll("[data-sui-transition]"))X(Q)}}function RJ(J){O0(J,(X)=>{K0(X)})}function bJ(J){let X=[];if(O0(J,(Z)=>{let z=B0(Z);if(z)X.push(z.finished.catch(()=>{}))}),X.length===0)return null;return Promise.all(X).then(()=>{return})}function m0(J,X,Z,z,Q){if(Z.fill(0,0,X),X===0)return;let j=0;for(let K=0;K<X;K++){let B=J[2*K+1],D=0,R=j;while(D<R){let b=D+R>>>1;if(J[2*Q[b]+1]<B)D=b+1;else R=b}if(z[K]=D>0?Q[D-1]:-1,Q[D]=K,D===j)j++}let q=Q[j-1];while(q>=0)Z[q]=1,q=z[q]}function MJ(J){let{anchor:X,items:Z,key:z,render:Q}=J,j=[],q=!0,K=new Set,B=new Map,D=new Map,R=new Map,b=new Set,O=[],x=[],n=new Int32Array(0),d=new Uint8Array(0),s=new Int32Array(0),zJ=new Int32Array(0),c=(u)=>{for(let a of u.nodes)try{a.parentNode?.removeChild(a)}catch{}u.cleanup?.()},WJ=l(()=>{if(!q)return;let u=Z(),a=u.length;if(n.length<a*2)n=new Int32Array(Math.max(n.length*2,a*2,16));if(d.length<a){let H=Math.max(d.length*2,a,8);d=new Uint8Array(H),s=new Int32Array(H),zJ=new Int32Array(H)}for(let H=0;H<a;H++){let i=u[H],r=z(i,H);if(B.has(r))console.warn("[sui] {#for}: duplicate key",r,"at index",H,"(prior at",B.get(r),"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");B.set(r,H)}for(let H of j)D.set(H.key,H);for(let H of j){if(B.has(H.key))continue;let i=bJ(H.nodes);if(i===null){c(H);continue}K.add(H),i.then(()=>{if(!K.delete(H))return;c(H)})}x.length=0,O.length=0;for(let H=0;H<a;H++){let i=u[H],r=z(i,H),LJ=D.get(r),FJ;if(LJ)LJ.item=i,TJ(X,"render",()=>LJ.update?.(i,H)),FJ=LJ;else{let e=TJ(X,"render",()=>Q(i,H));if(!e)continue;FJ={key:r,item:i,nodes:e.nodes,...e.cleanup&&{cleanup:e.cleanup},...e.update&&{update:e.update}},O.push(...e.nodes)}x.push(FJ)}for(let H=0;H<j.length;H++)R.set(j[H].key,H);let _=0;for(let H=0;H<x.length;H++){let i=R.get(x[H].key);if(i!==void 0)n[2*_]=H,n[2*_+1]=i,_++}m0(n,_,d,s,zJ);for(let H=0;H<_;H++)if(d[H])b.add(n[2*H]);let ZJ=X.parentNode,I=X;for(let H=0;H<x.length;H++){if(!ZJ)break;let i=x[H];if(b.has(H))I=i.nodes[i.nodes.length-1]??I;else for(let r of i.nodes)ZJ.insertBefore(r,I.nextSibling),I=r}let f=j;if(j=x,x=f,O.length>0)RJ(O);B.clear(),D.clear(),R.clear(),b.clear(),O.length=0});return()=>{q=!1,WJ();for(let u of j)c(u);for(let u of K)c(u);K.clear(),j=[],x.length=0}}function jJ(J){let{anchor:X}=J,{test:Z,branches:z}=g0(J)?{test:J.test,branches:J.branches}:u0(J),Q=null,j=-2,q=null,K=(D)=>{for(let R of D)try{R.parentNode?.removeChild(R)}catch{}},B=l(()=>{let D=Z();if(Q&&D===j)return;if(q&&D===q.idx&&D>=0&&D<z.length){let O=q;q=null;for(let x of O.nodes)X0(x);Q={nodes:O.nodes,...O.cleanup&&{cleanup:O.cleanup}},j=D,RJ(Q.nodes);return}if(Q){let O={idx:j,nodes:Q.nodes,...Q.cleanup&&{cleanup:Q.cleanup}},x=bJ(O.nodes);if(Q=null,x===null)K(O.nodes),O.cleanup?.();else q=O,x.then(()=>{if(q!==O)return;K(O.nodes),O.cleanup?.(),q=null})}if(j=D,D<0||D>=z.length)return;let R=TJ(X,"render",()=>z[D]());if(!R)return;Q=R;let b=X.parentNode;if(!b)return;for(let O of Q.nodes)b.insertBefore(O,X);RJ(Q.nodes)});return()=>{if(B(),Q)K(Q.nodes),Q.cleanup?.(),Q=null;if(q)K(q.nodes),q.cleanup?.(),q=null}}function g0(J){return typeof J.test==="function"}function u0(J){let{cond:X,then_:Z,else_:z}=J;return{test:()=>X()?0:z?1:-1,branches:z?[Z,z]:[Z]}}var Z0=(J)=>J??"",Q0=(J)=>{if(J===null||J==="")return NaN;return Number(J)},z0=(J)=>{if(J===null)return!1;if(J.toLowerCase()==="false")return!1;return!0};function lJ(J){return(X)=>{if(X===null||X==="")return J;try{return JSON.parse(X)}catch{return J}}}function W0(J,X){let Z=new Set(J);return(z)=>z!==null&&Z.has(z)?z:X}var dJ=(J)=>{switch(J.type){case"string":return Z0;case"bool":return z0;case"number":return Q0;case"union":{let X=J.values??[],Z=J.default??X[0]??"";return W0(X,Z)}case"object":return lJ(J.default);case"array":return lJ(J.default??[])}};function I0(J,X,Z,z){if(!X)return;let Q={},j,q=Z.formValue;if(q!==void 0){let{sig:B,propDef:D}=N0(Z,z,q,"formValue"),R=D.default,b=dJ(D);j=l(()=>{let O=B();X.setFormValue(h0(O))}),Q.reset=()=>{B.set(R)},Q.restore=(O)=>{B.set(b(O))}}let K=Z.formDisabled;if(K!==void 0){let{sig:B,propDef:D}=N0(Z,z,K,"formDisabled");if(D.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${K}" must be a bool-typed prop`);Q.disabled=(R)=>{B.set(!!R)}}if(J.__shipSetFormHooks=Q,j)J.__shipSetFormValueCleanup=j;return j}function C0(J){let X=J;delete X.__shipSetFormHooks,delete X.__shipSetFormValueCleanup}function h0(J){if(J===null||J===void 0)return"";if(typeof J==="boolean")return J?"on":"";return String(J)}function N0(J,X,Z,z){let Q=J.props[Z];if(!Q)throw Error(`[sui-runtime] ${z}: "${Z}" — no matching prop on this component`);let j=X[AJ(Z)];if(!j)throw Error(`[sui-runtime] ${z}: "${Z}" — prop signal missing`);return{sig:j,propDef:Q}}var $J={string(J={}){return{type:"string",...J}},bool(J={}){return{type:"bool",...J}},number(J={}){return{type:"number",...J}},union(J,X={}){return{type:"union",values:J,...X}},object(J={}){return{type:"object",...J}},array(J={}){return{type:"array",...J}}},l0=(J,X)=>{if(X===void 0||X===null)return null;switch(J.type){case"string":return X===""?null:String(X);case"number":return Number.isNaN(X)?null:String(X);case"bool":return X?"":null;case"union":return String(X);case"object":case"array":try{return JSON.stringify(X)}catch{return null}}},d0=(J,X)=>{let Z=`ship-set-style-${J}`;if(document.getElementById(Z))return;let z=document.createElement("style");z.id=Z,z.textContent=`[data-component="${J}"] { ${X} }`,document.head.appendChild(z)},p0=(J,X,Z,z)=>{let Q=J.getAttribute(X),j=Q!==null?z(Q):Z.default;return S(j)},i0=(J)=>{let X=[],Z={};for(let[j,q]of Object.entries(J.props??{}))X.push(j),Z[j]={camel:AJ(j),decode:dJ(q)};let z=J.formAssociated===!0||J.formValue!==void 0||J.formDisabled!==void 0,Q={observedAttrs:X,props:Z};if(z)Q.formAssociated=!0;return Q};function BJ(J,X){let Z=i0(X),z=Object.entries(X.props??{});Y0(J,Z,(Q,j,q)=>{let K=null;if(X.shadow){if(K=Q.shadowRoot??Q.attachShadow({mode:"open"}),X.styles){let D=document.createElement("style");D.textContent=X.styles,K.appendChild(D)}}else if(X.styles)d0(J,X.styles);let B=[];for(let[D,R]of z){let b=AJ(D),O=Z.props[D].decode,x=p0(Q,D,R,O);if(j[b]=x,R.reflect){let n=!0,d=l(()=>{let s=x();if(n){n=!1;return}let zJ=l0(R,s);if(zJ===null)Q.removeAttribute(D);else Q.setAttribute(D,zJ)});B.push(d)}}if(X.formValue!==void 0||X.formDisabled!==void 0){let D=I0(Q,q,X,j);if(typeof D==="function")B.push(D)}if(X.setup){let D={props:j,host:Q,cleanups:B};if(q)D.internals=q;X.setup(D)}return()=>{C0(Q);for(let D of B.splice(0))try{D()}catch(R){console.error(`[ship-set:${J}] cleanup threw:`,R)}}})}var T0=new Set;function Y0(J,X,Z){if(customElements.get(J)){if(!T0.has(J))T0.add(J),console.warn(`[sui-runtime] customElements.define("${J}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(X.formAssociated){class Q extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return X.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=J,uJ(this),this.scope=SJ();try{this.cleanup=kJ(this.scope,()=>sJ(this,()=>Z(this,this.propSignals,this.internals)))}catch(j){let q=OJ(this.parentNode);if(q)q.catch(j,"mount");else console.error(`[ship-set:${J}] setup threw:`,j)}}disconnectedCallback(){try{this.cleanup?.()}catch(j){console.error(`[ship-set:${J}] cleanup threw:`,j)}this.cleanup=void 0;try{this.scope?.dispose()}catch(j){console.error(`[ship-set:${J}] scope dispose threw:`,j)}this.scope=void 0}attributeChangedCallback(j,q,K){let B=X.props[j];if(!B)return;let D=this.propSignals[B.camel];if(D)D.set(B.decode(K))}formResetCallback(){let j=this.__shipSetFormHooks;try{j?.reset?.()}catch(q){console.error(`[ship-set:${J}] formResetCallback threw:`,q)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(j,q){let K=this.__shipSetFormHooks;try{K?.restore?.(j)}catch(B){console.error(`[ship-set:${J}] formStateRestoreCallback threw:`,B)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:j,mode:q}}))}formDisabledCallback(j){let q=this.__shipSetFormHooks;try{q?.disabled?.(j)}catch(K){console.error(`[ship-set:${J}] formDisabledCallback threw:`,K)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:j}}))}}customElements.define(J,Q);return}class z extends HTMLElement{static get observedAttributes(){return X.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=J,uJ(this),this.scope=SJ();try{this.cleanup=kJ(this.scope,()=>sJ(this,()=>Z(this,this.propSignals)))}catch(Q){let j=OJ(this.parentNode);if(j)j.catch(Q,"mount");else console.error(`[ship-set:${J}] setup threw:`,Q)}}disconnectedCallback(){try{this.cleanup?.()}catch(Q){console.error(`[ship-set:${J}] cleanup threw:`,Q)}this.cleanup=void 0;try{this.scope?.dispose()}catch(Q){console.error(`[ship-set:${J}] scope dispose threw:`,Q)}this.scope=void 0}attributeChangedCallback(Q,j,q){let K=X.props[Q];if(!K)return;let B=this.propSignals[K.camel];if(B)B.set(K.decode(q))}}customElements.define(J,z)}function AJ(J){return J.replace(/-([a-z])/g,(X,Z)=>Z.toUpperCase())}function j0(J,X,Z){let z=a0(J.state,{ship:X.ship,...Z}),Q={};for(let[O,x]of Object.entries(z))Q[O]=S(x);let j={ship:X.ship,route:X.route??{},navigate:X.navigate??((O)=>{}),...Z,...Q},q=r0(Q,j),K={...j},B={};for(let[O,x]of Object.entries(J.derived??{}))B[O]=QJ(()=>x(K));Object.assign(K,B);let D={};for(let[O,x]of Object.entries(J.on??{}))D[O]=(...n)=>x(q,...n);let R={},b=typeof J.use==="function"?J.use(K):J.use??{};for(let[O,x]of Object.entries(b))R[O]=j0(x,X,K);return{...Q,...B,...D,...R}}function a0(J,X){if(J===void 0)return{};if(typeof J==="function")return J(X);return J}function r0(J,X){return new Proxy({},{get(Z,z){if(z in J)return J[z]();return X[z]},set(Z,z,Q){if(z in J)return J[z].set(Q),!0;return console.warn(`[ship-set] hook handler tried to write s.${z} = … but ${z} is not a hook state key. The assignment was ignored.`),!0},has(Z,z){return z in J||z in X},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(Z,z){if(z in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[z]()};return}})}var E0="__shipSetLifecycle";function R0(J,X){let Z=o0(J,X),z={ship:X.ship,route:X.route??{},navigate:X.navigate??((O)=>{}),...X.scope??{}},Q=Z.scope?Z.scope(z,X.host):{};Object.assign(z,Q);let j=s0(z,X.stateKeys,Q),q=_0(j,z),K={...z},B=t0(Z,K);Object.assign(K,B);let D=JX(Z,K,X);for(let[O,x]of Object.entries(D))K[O]=x;let R=e0(Z,q),b={...Q,...B,...R,...D};if(Object.keys(Q).length>0&&X.host){let O=tJ(Q);if(Object.keys(O).length>0){let x=X.host,n=x.tagName.toLowerCase();_J(x,n,O)}}if(Z.mount||Z.unmount){let O={state:q};if(Z.mount)O.mount=Z.mount;if(Z.unmount)O.unmount=Z.unmount;Object.defineProperty(b,E0,{value:O,enumerable:!0,configurable:!0,writable:!1})}return b}function o0(J,X){if(typeof J==="function")return J({ship:X.ship});return J}function s0(J,X,Z){let z={},Q=X!==void 0,j=Z?Object.keys(Z):[];if(Q||j.length>0){if(X)for(let q of X){let K=J[q];if(G0(K))z[q]=K}for(let q of j){if(q in z)continue;let K=J[q];if(G0(K))z[q]=K}return z}for(let[q,K]of Object.entries(J))if(G0(K))z[q]=K;return z}function G0(J){return typeof J==="function"&&"set"in J&&typeof J.set==="function"}function _0(J,X){return new Proxy({},{get(Z,z){if(z in J)return J[z]();return X[z]},set(Z,z,Q){if(z in J){if(typeof Q==="function")J[z].set(()=>Q);else J[z].set(Q);return!0}return console.warn(`[ship-set] handler tried to write s.${z} = … but ${z} is not declared as <state name="${z}"> in the template. The assignment was ignored.`),!0},has(Z,z){return z in J||z in X},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(Z,z){if(z in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[z]()};return}})}function t0(J,X){let Z=J.derived??{},z={};for(let[Q,j]of Object.entries(Z))z[Q]=QJ(()=>j(X));return z}function e0(J,X){let Z=J.on??{},z={};for(let[Q,j]of Object.entries(Z))z[Q]=(...q)=>j(X,...q);return z}function JX(J,X,Z){if(J.use===void 0)return{};let z=typeof J.use==="function"?J.use(X):J.use,Q={};for(let[j,q]of Object.entries(z))Q[j]=j0(q,Z,X);return Q}function NJ(J){return{setup:(X)=>R0(J,X),__def:J}}if(typeof customElements<"u"){for(let J of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(J))customElements.define(J,class extends HTMLElement{})}var zX=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,WX=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function A0(J){if(typeof CSSStyleSheet>"u")return null;try{let X=new CSSStyleSheet;return X.replaceSync(J),X}catch{return null}}var w0=A0(zX),k0=A0(WX);function IJ(J){if(typeof ShadowRoot>"u")return;if(!(J instanceof ShadowRoot))return;let X=[k0,w0].filter((Z)=>Z!=null);if(X.length===0)return;try{J.adoptedStyleSheets=[...X,...J.adoptedStyleSheets??[]]}catch{}}var pJ=[{id:"view",label:"View",icon:"file",shapes:["list","form","detail","dashboard","tabs","blank"],shapeLabel:"Starting shape"},{id:"skill",label:"Skill",icon:"sparkles",shapes:["task","reference","workflow"],shapeLabel:"Skill kind"},{id:"agent",label:"Agent",icon:"users",shapes:["specialist","generalist","reviewer"],shapeLabel:"Agent kind"},{id:"pipeline",label:"Pipeline",icon:"play",shapes:["sequential","fanout","review-gated"],shapeLabel:"Topology"},{id:"trigger",label:"Trigger",icon:"zap",shapes:["webhook","schedule","file-change"],shapeLabel:"Kind"},{id:"theme",label:"Theme",icon:"palette",shapes:["light","dark","brand-accent","high-contrast"],shapeLabel:"Starting shape"}],iJ=[{label:"Dashboard",path:"/dashboard",icon:"sparkles",keywords:"home overview"},{label:"Pipelines",path:"/pipelines",icon:"play",keywords:"workflows runs jobs"},{label:"Agents",path:"/agents",icon:"users",keywords:"configure profiles"},{label:"Skills",path:"/skills",icon:"zap",keywords:"browse edit files"},{label:"Git",path:"/git",icon:"git",keywords:"status diff commit branch"},{label:"Collections",path:"/collections",icon:"folder",keywords:"data items query"},{label:"Canvas",path:"/canvas",icon:"image",keywords:"docs visual feedback"},{label:"Events",path:"/events",icon:"sparkles",keywords:"stream log kernel"},{label:"Settings",path:"/settings",icon:"settings",keywords:"config theme provider"},{label:"Chat",path:"/chat",icon:"message-square-plus",keywords:"agent conversation"},{label:"Specs",path:"/specs",icon:"file",keywords:"features design"},{label:"Theme Studio",path:"/theme-studio",icon:"palette",keywords:"design tokens"},{label:"Chats (Mission Control)",path:"/chats",icon:"messages-square",keywords:"sessions active"}],nJ=[{key:"new",label:"/new",hint:"Scaffold a view, skill, agent, …"},{key:"chats",label:"/chats",hint:"Open Mission Control"},{key:"settings",label:"/settings",hint:"Jump to settings"},{key:"lint",label:"/lint",hint:"Run lint on the current view"},{key:"end",label:"/end",hint:"End the current session"},{key:"close",label:"/close",hint:"Close the command bar"}],YX=/^[a-z][a-z0-9-]{0,48}$/;function x0(J){return YX.test(J)}function fJ(J,X){if(J.startsWith("/"))return"command";if(J.startsWith(">"))return"find";if(J.startsWith("+"))return"create";if(J.trim().length===0)return X;if(/(^|\s)@[^\s@]*$/.test(J))return"mention";return"ai"}async function DJ(J){let X=window;if(X.ship?.navigate){X.ship.navigate(J);return}history.pushState({},"",J),document.dispatchEvent(new CustomEvent("sui:navigate",{detail:{to:J}}))}var cJ=NJ({scope:(J)=>{let X=J,Z=t(X.open),z=t(X.mode),Q=t(S("")),j=t(S(null)),q=t(S([])),K=t(S([])),B=t(S([])),D=t(S([])),R=t(S(0)),b=t(S(!1)),O=t(S("")),x=t(S("")),n=t(S(!1)),d=t(S(null)),s=t(S([])),zJ=t(S("main")),c=t(S("view")),WJ=t(S("")),u=t(S("")),a=t(S("list")),_=t(S(!1)),ZJ=t(S(null)),I=t(QJ(()=>z()==="ai"&&j()!==null)),f=eJ(null);return{open:Z,mode:z,value:Q,session:j,agents:q,providers:K,allSessions:B,views:D,selectedIdx:R,sessionPickerOpen:b,spawnAgent:O,spawnProvider:x,spawning:n,spawnError:d,thread:s,workspaceId:zJ,createSurface:c,createName:WJ,createPurpose:u,createShape:a,createSubmitting:_,createError:ZJ,isChat:I,dialogRef:f}},derived:{triggerLabel:(J)=>{let X=J.session();return X?`Ask ${X.agent_id??"Ship"} or jump to…`:"Ask Ship or jump to…"},dotBg:(J)=>{let X=J.session();if(X?.status==="thinking")return"var(--warning,#d97706)";if(X)return"var(--success,#16a34a)";return"var(--muted-foreground)"}},on:{openFind:(J)=>{J.mode="find",J.value="",J.selectedIdx=0,J.open=!0}},mount:(J,X)=>{let Z=J,z=Z.open,Q=Z.mode,j=Z.value,q=Z.session,K=Z.agents,B=Z.providers,D=Z.allSessions,R=Z.views,b=Z.selectedIdx,O=Z.spawnAgent,x=Z.spawnProvider,n=Z.spawning,d=Z.spawnError,s=Z.thread,zJ=Z.workspaceId,c=Z.createSurface,WJ=Z.createName,u=Z.createPurpose,a=Z.createShape,_=Z.createSubmitting,ZJ=Z.createError,I=Z.dialogRef?.(),f=null,H=0,i=l(()=>{let F=z();if(X.setAttribute("data-state",F?"open":"closed"),F){if(typeof I?.show==="function")I.show();queueMicrotask(()=>{X.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelector(".cb-search")?.focus()})}else{if(typeof I?.close==="function")I.close();X.dispatchEvent(new CustomEvent("sui-command:cancel",{detail:{mode:Q()},bubbles:!0,composed:!0}))}});X.show=(F,U)=>{if(F)Q.set(F);if(typeof U==="string")j.set(U);z.set(!0)},X.hide=()=>z.set(!1),X.toggle=(F)=>{if(z())z.set(!1);else{if(F)Q.set(F);z.set(!0)}},X.setMode=(F)=>Q.set(F);function r(F){LJ();let U=new AbortController;f=U;let C=new EventSource(`/api/v1/sessions/${F}/events`,{withCredentials:!0}),E=null;C.addEventListener("provider",(YJ)=>{let o;try{o=JSON.parse(YJ.data)}catch{return}switch(o.kind){case"message":{let qJ=o.text??"";if(!qJ)return;if(E)s.set(s().map((h)=>h.id===E?{...h,text:h.text+qJ}:h));else{let h=String(Date.now()+Math.random());E=h,s.set([...s(),{id:h,sender:"agent",text:qJ}])}break}case"tool_use":s.set([...s(),{id:String(Date.now()+Math.random()),sender:"system",text:`→ tool: ${o.tool??"?"}`}]);break;case"turn_complete":E=null;break}}),C.onerror=()=>{if(!U.signal.aborted)C.close()},U.signal.addEventListener("abort",()=>C.close())}function LJ(){f?.abort(),f=null}let FJ=l(()=>{let F=q();if(F?.id)r(F.id);else LJ()});async function e(){try{let F=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(F.ok){let U=await F.json();D.set(U.sessions);let C=U.sessions.find((E)=>E.status!=="ended"&&E.agent_id);if(C&&C.id!==q()?.id)q.set(C),s.set([]);else if(!C)q.set(null)}}catch{}if(K().length===0)GJ();if(B().length===0)UJ();if(R().length===0)m()}async function GJ(){try{let F=await fetch("/api/v1/agents",{credentials:"same-origin"});if(!F.ok)return;let C=((await F.json()).items??[]).map((E)=>({id:E.id}));if(K.set(C),!O()&&C.length>0)O.set(C[0].id)}catch{}}async function UJ(){try{let F=await fetch("/api/v1/providers/installed",{credentials:"same-origin"});if(!F.ok)return;let U=await F.json();if(B.set(U.providers),!x()&&U.providers.length>0)x.set(U.providers[0])}catch{}}async function m(){try{let F=await fetch("/api/v1/views",{credentials:"same-origin"});if(!F.ok)return;let U=await F.json(),C=U.data?.views??U.views??[];R.set(C.map((E)=>E.name).sort())}catch{}}function p(){return X.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelectorAll(".cb-item")}function v(F){let U=p();if(!U?.length)return;b.set(Math.max(0,Math.min(U.length-1,b()+F)))}function w(){let F=Q(),U=j(),E=p()?.[b()];if(E){let YJ=E.dataset.path;if(YJ)DJ(YJ),j.set(""),z.set(!1);let o=E.dataset.cmd;if(o)j.set(""),y("/"+o);let qJ=E.dataset.mentionKind,h=E.dataset.mentionId;if(qJ&&h){let VJ=qJ==="view"?`@${h}`:`@${qJ}:${h}`;j.set(U.replace(/@[^@\s]*$/,`${VJ} `)),Q.set(fJ(j(),F))}X.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:F,item:E.dataset},bubbles:!0,composed:!0}));return}if(F==="command")y(U);else if(F==="find")N(U)}function y(F){let U=F.replace(/^\/+\s*/,"").toLowerCase(),C=nJ.find((E)=>E.key===U||E.key.startsWith(U));if(!C)return;switch(C.key){case"new":Q.set("create"),j.set("");break;case"chats":DJ("/chats"),z.set(!1);break;case"settings":DJ("/settings"),z.set(!1);break;case"end":if(q())G(q().id);z.set(!1);break;case"close":z.set(!1);break;case"lint":L();break}}function N(F){let U=F.replace(/^>+\s*/,"").toLowerCase().trim(),C=iJ.find((E)=>E.label.toLowerCase().includes(U)||E.keywords.toLowerCase().includes(U)||E.path.toLowerCase().includes(U));if(C)DJ(C.path),j.set(""),z.set(!1)}async function G(F){try{await fetch(`/api/v1/sessions/${F}`,{method:"DELETE",credentials:"same-origin"})}catch{}if(q()?.id===F)q.set(null);await e()}async function L(){let U=window.ship?.current?.()?.path?.replace(/^\/+/,"").split("/")[0]??"";if(!U)return;try{let C=await fetch("/api/v1/lint/view",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:U})});if(C.ok){let YJ=(await C.json()).data?.findings?.length??0;console.log(YJ===0?`${U}: no lint findings`:`${U}: ${YJ} lint findings`)}}catch{}z.set(!1)}let W=(F)=>{if(F.key==="Escape"&&z()){z.set(!1);return}if(F.key===" "&&!z()){let U=F.target;if(U&&(U.tagName==="INPUT"||U.tagName==="TEXTAREA"||U.isContentEditable))return;let C=performance.now();if(C-H<350)H=0,F.preventDefault(),Q.set("ai"),j.set(""),z.set(!0);else H=C}},Y=(F)=>{if(F.key==="ArrowDown")F.preventDefault(),v(1);else if(F.key==="ArrowUp")F.preventDefault(),v(-1);else if(F.key==="Enter")F.preventDefault(),w()};X.addEventListener("keydown",Y),document.addEventListener("keydown",W);let P=()=>{if(z())z.set(!1)};I?.addEventListener("sui-close",P);let M=()=>void e(),V=(F)=>{let U=F;if(U.detail?.mode)Q.set(U.detail.mode);if(typeof U.detail?.prefill==="string")j.set(U.detail.prefill);z.set(!0)},T=()=>{Q.set("create"),z.set(!0)};window.addEventListener("ship:session:changed",M);let k=document.querySelector("ship-shell")??document.querySelector("sui-shell");k?.addEventListener("ship:bar:open",V),k?.addEventListener("ship:create:open",T);let A=X["__sui_ctx_sui-command-bar"];if(A)A.endSession=G,A.loadCurrentSession=e;return e(),()=>{if(document.removeEventListener("keydown",W),X.removeEventListener("keydown",Y),window.removeEventListener("ship:session:changed",M),k?.removeEventListener("ship:bar:open",V),k?.removeEventListener("ship:create:open",T),I)I.removeEventListener("sui-close",P);FJ(),i()}}});var Lz=BJ("sui-command-bar",{props:{open:$J.bool({default:!1,reflect:!0}),mode:$J.string({default:"ai",reflect:!0})},shadow:!0,styles:`
  :host { display: contents; }

  /* ── Trigger pill ─────────────────────────────────────────────── */
  .cb-trigger {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    background: var(--background);
    color: var(--muted-foreground);
    font: inherit;
    font-size: var(--text-xs, 12px);
    cursor: pointer;
    min-width: 200px;
    max-width: 320px;
    text-align: left;
    transition: border-color var(--duration-fast, 120ms) var(--ease-out, ease);
  }
  .cb-trigger:hover {
    border-color: color-mix(in oklch, var(--foreground) 30%, var(--border));
  }
  .cb-label { flex: 1; }
  .cb-trigger kbd {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 5px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--muted);
    color: var(--muted-foreground);
  }

  /* ── Status dot ───────────────────────────────────────────────── */
  .cb-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--muted-foreground);
    flex-shrink: 0;
    display: inline-block;
  }

  /* ── Dialog host override ─────────────────────────────────────── */
  .cb-dialog::part(body) {
    padding: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  /* ── Wrap (inside dialog body slot) ──────────────────────────── */
  .cb-wrap {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null);Q.open=X.open,Q.mode=X.mode;let j=null;if(cJ&&typeof cJ.setup==="function"){if(J.scope=Q,j=cJ.setup(J),j&&typeof j==="object")Object.assign(Q,j)}let q=(c)=>jJ({anchor:c,test:()=>{if(Q.isChat())return 0;return 1},branches:[()=>{let WJ=document.createDocumentFragment(),u=[],a=document.createElement("sui-command-chat");return WJ.appendChild(a),{nodes:Array.from(WJ.childNodes),cleanup:()=>{for(let _ of u)try{_()}catch(ZJ){}}}},()=>{let WJ=document.createDocumentFragment(),u=[],a=document.createElement("sui-command-palette");return WJ.appendChild(a),{nodes:Array.from(WJ.childNodes),cleanup:()=>{for(let _ of u)try{_()}catch(ZJ){}}}}]}),K=Z.shadowRoot??Z;IJ(K);let B=document.createElement("button");B.setAttribute("class","cb-trigger"),B.setAttribute("type","button"),B.setAttribute("aria-label","Open command bar"),z.push(XJ(B,"click",function(){let c=Q.openFind;return typeof c==="function"?c.bind(void 0):()=>{}}(),{modifiers:[]}));let D=document.createElement("sui-icon");D.setAttribute("name","search"),D.setAttribute("size","sm"),B.appendChild(D);let R=document.createElement("span");R.setAttribute("class","cb-label");let b=document.createTextNode("");R.appendChild(b),z.push(g(b,()=>Q.triggerLabel())),B.appendChild(R);let O=document.createElement("span");O.setAttribute("class","cb-dot"),z.push(l(()=>{let c=Q.dotBg();if(c==null)O.style.removeProperty("background");else O.style.setProperty("background",String(c))})),B.appendChild(O);let x=document.createElement("kbd"),n=document.createTextNode("⌘K");x.appendChild(n),B.appendChild(x),K.appendChild(B);let d=document.createElement("sui-dialog");d.setAttribute("class","cb-dialog"),d.setAttribute("no-header",""),d.setAttribute("no-footer",""),d.setAttribute("no-close-button",""),d.setAttribute("backdrop","blur"),d.setAttribute("size","full"),d.setAttribute("style","--dlg-max-w: min(840px, 94vw)"),J0(Q,"dialogRef",d);let s=document.createElement("div");s.setAttribute("class","cb-wrap");let zJ=document.createComment("if");if(s.appendChild(zJ),z.push(q(zJ)),d.appendChild(s),K.appendChild(d),j!=null){let c=Object.getOwnPropertyDescriptor(j,"__shipSetLifecycle")?.value;if(c){let WJ=new Proxy(Q,{get(u,a){return u[a]},set(u,a,_){let ZJ=u[a];if(ZJ!=null&&typeof ZJ.set==="function"){if(typeof _==="function")ZJ.set(()=>_);else ZJ.set(_);return!0}return!1}});if(c.mount){let u=c.mount(WJ,Z);if(typeof u==="function")z.push(u)}if(c.unmount)z.push(()=>c.unmount(WJ))}}return Q}});var aJ=NJ({scope:(J,X)=>{let Z=yJ(X,"sui-command-bar"),z=QJ(()=>{if(Z.session()?.status==="thinking")return"var(--warning,#d97706)";return"var(--success,#16a34a)"}),Q=QJ(()=>Z.allSessions().filter((D)=>D.status!=="ended"&&D.agent_id));function j(D){if(fetch(`/api/v1/sessions/${D}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}),Z.session()?.id===D)Z.session.set(null);Z.sessionPickerOpen.set(!1)}function q(D){let R=Z.allSessions().find((b)=>b.id===D);if(R)Z.session.set(R),Z.thread.set([]);Z.sessionPickerOpen.set(!1)}function K(){Z.sessionPickerOpen.set(!1),Z.mode.set("ai"),Z.session.set(null)}function B(){Z.sessionPickerOpen.set(!1),DJ("/chats"),Z.open.set(!1)}return{bar:Z,headerDotBg:z,activeSessions:Q,endSession:j,selectSession:q,startNewSession:K,viewChats:B}},on:{toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},sendMessage:async(J)=>{let X=J.bar.value().trim();if(!X||!J.bar.session())return;let Z=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:X}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${Z}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:X})})}catch{}},onComposerKeydown:async(J,X)=>{if(X.key==="Enter"&&!X.shiftKey){X.preventDefault();let Z=J.bar.value().trim();if(!Z||!J.bar.session())return;let z=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:Z}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${z}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:Z})})}catch{}}}}});var Pz=BJ("sui-command-chat",{shadow:!0,styles:`
  :host { display: flex; flex-direction: column; flex: 1; min-height: 0; overflow: hidden; }

  /* ── Header ───────────────────────────────────────────────────── */
  .cb-chat-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
    flex: 0 0 auto;
  }
  .cb-dot {
    width: 8px; height: 8px;
    border-radius: 999px;
    flex-shrink: 0;
  }
  .cb-chat-header .title { font-size: 13px; font-weight: 600; }
  .cb-chat-header .sub { font-size: 11px; color: var(--muted-foreground); }
  .cb-chip-wrap { position: relative; flex-shrink: 0; }
  .cb-session-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    font-size: 11px;
    color: var(--muted-foreground);
    cursor: pointer;
    user-select: none;
  }
  .cb-session-chip:hover { background: var(--muted); }
  .cb-esc {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 5px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--muted);
    color: var(--muted-foreground);
  }

  /* ── Session picker ───────────────────────────────────────────── */
  .cb-session-popover {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    min-width: 280px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-lg);
    z-index: 10;
    padding: 4px 0;
  }
  .cb-session-row {
    display: grid;
    grid-template-columns: 14px 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 7px 12px;
    cursor: pointer;
    font-size: 12px;
  }
  .cb-session-row:hover { background: var(--muted); }
  .cb-session-row.current { background: color-mix(in oklch, var(--primary) 8%, transparent); }
  .cb-session-row .meta { color: var(--muted-foreground); font-size: 11px; }
  .cb-session-end {
    background: transparent; border: none; cursor: pointer;
    color: var(--muted-foreground); font-size: 14px; padding: 0 2px;
  }
  .cb-session-end:hover { color: var(--destructive, red); }
  .cb-session-sep { height: 1px; background: var(--border); margin: 4px 0; }
  .cb-session-action {
    display: flex; align-items: center; gap: 8px;
    padding: 7px 12px; cursor: pointer; font-size: 12px;
    color: var(--muted-foreground);
  }
  .cb-session-action:hover { background: var(--muted); color: var(--foreground); }

  /* ── Messages ─────────────────────────────────────────────────── */
  .cb-chat-messages {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-width: thin;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .cb-chat-empty { color: var(--muted-foreground); font-size: 13px; text-align: center; margin-top: 24px; }
  .cb-chat-turn { display: flex; flex-direction: column; }
  .cb-chat-turn[data-sender='user'] { align-items: flex-end; }
  .cb-chat-turn[data-sender='agent'] { align-items: flex-start; }
  .cb-chat-turn[data-sender='system'] { align-items: center; }
  .cb-chat-bubble {
    max-width: 80%;
    padding: 7px 10px;
    border-radius: var(--radius-md, 6px);
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .cb-chat-turn[data-sender='user'] .cb-chat-bubble {
    background: var(--primary);
    color: var(--primary-foreground, #fff);
  }
  .cb-chat-turn[data-sender='agent'] .cb-chat-bubble {
    background: var(--muted);
    color: var(--foreground);
  }
  .cb-chat-turn[data-sender='system'] .cb-chat-bubble {
    background: transparent;
    color: var(--muted-foreground);
    font-size: 11px;
    font-family: var(--font-mono);
  }

  /* ── Composer ─────────────────────────────────────────────────── */
  .cb-chat-composer {
    display: flex;
    gap: 8px;
    align-items: flex-end;
    padding: 10px 14px;
    border-top: 1px solid var(--border);
    flex: 0 0 auto;
  }
  .cb-chat-input {
    flex: 1;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    background: var(--background);
    color: var(--foreground);
    font: inherit;
    font-size: 13px;
    padding: 6px 8px;
    resize: none;
    line-height: 1.4;
  }
  .cb-chat-input:focus { outline: none; border-color: var(--primary); }
  .cb-chat-send {
    height: 32px;
    padding: 0 14px;
    border: none;
    border-radius: var(--radius-sm, 4px);
    background: var(--primary);
    color: var(--primary-foreground, #fff);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
    flex-shrink: 0;
    align-self: flex-end;
  }
  .cb-chat-send:disabled { opacity: 0.5; cursor: not-allowed; }
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null),j=null;if(aJ&&typeof aJ.setup==="function"){if(J.scope=Q,j=aJ.setup(J),j&&typeof j==="object")Object.assign(Q,j)}let q=(m)=>MJ({anchor:m,items:()=>Q.activeSessions(),key:(p,v)=>{let w=Object.create(Q);return w.s=p,w.__i=v,((y)=>p.id)(w)},render:(p,v)=>{let w=Object.create(Q);return w.s=p,w.__i=v,w.__v=S(0),((y)=>{let N=document.createDocumentFragment(),G=[],L=document.createElement("div");L.setAttribute("class","cb-session-row"),G.push(EJ(L,"current",()=>(y.__v(),y.bar.session()?.id===y.s.id))),L.setAttribute("role","option"),G.push(XJ(L,"click",function(){let C=()=>y.selectSession(y.s.id);return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let W=document.createElement("span");W.setAttribute("class","cb-dot"),G.push(l(()=>{y.__v();let C=y.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(C==null)W.style.removeProperty("background");else W.style.setProperty("background",String(C))})),L.appendChild(W);let Y=document.createElement("span"),P=document.createElement("strong"),M=document.createTextNode("");P.appendChild(M),G.push(g(M,()=>(y.__v(),y.s.agent_id??""))),Y.appendChild(P);let V=document.createElement("span");V.setAttribute("class","meta");let T=document.createTextNode(" · ");V.appendChild(T);let k=document.createTextNode("");V.appendChild(k),G.push(g(k,()=>(y.__v(),y.s.provider??"")));let $=document.createTextNode(" · ");V.appendChild($);let A=document.createTextNode("");V.appendChild(A),G.push(g(A,()=>(y.__v(),y.s.workspace_id))),Y.appendChild(V),L.appendChild(Y);let F=document.createElement("button");F.setAttribute("class","cb-session-end"),F.setAttribute("type","button"),F.setAttribute("aria-label","End session"),G.push(XJ(F,"click",function(){let C=(E)=>{E.stopPropagation(),y.endSession(y.s.id)};return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let U=document.createTextNode("×");return F.appendChild(U),L.appendChild(F),N.appendChild(L),{nodes:Array.from(N.childNodes),cleanup:()=>{for(let C of G)try{C()}catch(E){}},update:(C,E)=>{y.s=C,y.__i=E,y.__v.set((YJ)=>YJ+1)}}})(w)}}),K=(m)=>jJ({anchor:m,test:()=>{if(Q.activeSessions().length===0)return 0;return-1},branches:[()=>{let p=document.createDocumentFragment(),v=[],w=document.createElement("div");w.setAttribute("class","cb-session-row");let y=document.createElement("span");w.appendChild(y);let N=document.createElement("span");N.setAttribute("class","meta");let G=document.createTextNode("No active sessions");N.appendChild(G),w.appendChild(N);let L=document.createElement("span");return w.appendChild(L),p.appendChild(w),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let W of v)try{W()}catch(Y){}}}}]}),B=(m)=>jJ({anchor:m,test:()=>{if(Q.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let p=document.createDocumentFragment(),v=[],w=document.createElement("div");w.setAttribute("class","cb-session-popover"),w.setAttribute("role","listbox"),w.setAttribute("aria-label","Sessions");let y=document.createComment("for");w.appendChild(y),v.push(q(y));let N=document.createComment("if");w.appendChild(N),v.push(K(N));let G=document.createElement("div");G.setAttribute("class","cb-session-sep"),G.setAttribute("role","separator"),w.appendChild(G);let L=document.createElement("div");L.setAttribute("class","cb-session-action"),v.push(XJ(L,"click",function(){let $=Q.startNewSession;return typeof $==="function"?$.bind(void 0):()=>{}}(),{modifiers:[]}));let W=document.createElement("sui-icon");W.setAttribute("name","plus"),W.setAttribute("size","sm"),W.setAttribute("aria-hidden","true"),L.appendChild(W);let Y=document.createElement("span"),P=document.createTextNode("Start new session");Y.appendChild(P),L.appendChild(Y),w.appendChild(L);let M=document.createElement("div");M.setAttribute("class","cb-session-action"),v.push(XJ(M,"click",function(){let $=Q.viewChats;return typeof $==="function"?$.bind(void 0):()=>{}}(),{modifiers:[]}));let V=document.createElement("sui-icon");V.setAttribute("name","columns"),V.setAttribute("size","sm"),V.setAttribute("aria-hidden","true"),M.appendChild(V);let T=document.createElement("span"),k=document.createTextNode("View all chats");return T.appendChild(k),M.appendChild(T),w.appendChild(M),p.appendChild(w),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let $ of v)try{$()}catch(A){}}}}]}),D=(m)=>MJ({anchor:m,items:()=>Q.bar.thread(),key:(p,v)=>{let w=Object.create(Q);return w.msg=p,w.__i=v,((y)=>p.id)(w)},render:(p,v)=>{let w=Object.create(Q);return w.msg=p,w.__i=v,w.__v=S(0),((y)=>{let N=document.createDocumentFragment(),G=[],L=document.createElement("div");L.setAttribute("class","cb-chat-turn"),G.push(JJ(L,"data-sender",()=>(y.__v(),y.msg.sender)));let W=document.createElement("div");W.setAttribute("class","cb-chat-bubble");let Y=document.createTextNode("");return W.appendChild(Y),G.push(g(Y,()=>(y.__v(),y.msg.text))),L.appendChild(W),N.appendChild(L),{nodes:Array.from(N.childNodes),cleanup:()=>{for(let P of G)try{P()}catch(M){}},update:(P,M)=>{y.msg=P,y.__i=M,y.__v.set((V)=>V+1)}}})(w)}}),R=(m)=>jJ({anchor:m,test:()=>{if(Q.bar.thread().length===0)return 0;return 1},branches:[()=>{let p=document.createDocumentFragment(),v=[],w=document.createElement("div");w.setAttribute("class","cb-chat-empty");let y=document.createTextNode("Type below and press Enter. The agent's replies land here.");return w.appendChild(y),p.appendChild(w),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let N of v)try{N()}catch(G){}}}},()=>{let p=document.createDocumentFragment(),v=[],w=document.createComment("for");return p.appendChild(w),v.push(D(w)),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let y of v)try{y()}catch(N){}}}}]}),b=Z.shadowRoot??Z;IJ(b);let O=document.createElement("div");O.setAttribute("class","cb-chat-header");let x=document.createElement("span");x.setAttribute("class","cb-dot"),z.push(l(()=>{let m=Q.headerDotBg();if(m==null)x.style.removeProperty("background");else x.style.setProperty("background",String(m))})),x.setAttribute("aria-hidden","true"),O.appendChild(x);let n=document.createElement("div");n.setAttribute("style","display:grid;line-height:1.25");let d=document.createElement("span");d.setAttribute("class","title");let s=document.createTextNode("");d.appendChild(s),z.push(g(s,()=>Q.bar.session()?.agent_id??"")),n.appendChild(d);let zJ=document.createElement("span");zJ.setAttribute("class","sub");let c=document.createTextNode("");zJ.appendChild(c),z.push(g(c,()=>Q.bar.session()?.provider??"—"));let WJ=document.createTextNode(" · ");zJ.appendChild(WJ);let u=document.createTextNode("");zJ.appendChild(u),z.push(g(u,()=>Q.bar.session()?.workspace_id??"")),n.appendChild(zJ),O.appendChild(n);let a=document.createElement("span");a.setAttribute("style","flex:1"),O.appendChild(a);let _=document.createElement("div");_.setAttribute("class","cb-chip-wrap");let ZJ=document.createElement("span");ZJ.setAttribute("class","cb-session-chip"),ZJ.setAttribute("role","button"),ZJ.setAttribute("aria-haspopup","listbox"),z.push(XJ(ZJ,"click",function(){let m=Q.toggleSessionPicker;return typeof m==="function"?m.bind(void 0):()=>{}}(),{modifiers:[]}));let I=document.createTextNode("Switch");ZJ.appendChild(I),_.appendChild(ZJ);let f=document.createComment("if");_.appendChild(f),z.push(B(f)),O.appendChild(_);let H=document.createElement("kbd");H.setAttribute("class","cb-esc"),H.setAttribute("aria-label","Escape to close");let i=document.createTextNode("esc");H.appendChild(i),O.appendChild(H),b.appendChild(O);let r=document.createElement("div");r.setAttribute("class","cb-chat-messages");let LJ=document.createComment("if");r.appendChild(LJ),z.push(R(LJ)),b.appendChild(r);let FJ=document.createElement("div");FJ.setAttribute("class","cb-chat-composer");let e=document.createElement("textarea");e.setAttribute("class","cb-chat-input"),e.setAttribute("rows","2"),e.setAttribute("placeholder","Message the agent…"),z.push(HJ(e,Q.bar.value)),z.push(XJ(e,"keydown",function(){let m=Q.onComposerKeydown;return typeof m==="function"?m.bind(void 0):()=>{}}(),{modifiers:[]})),FJ.appendChild(e);let GJ=document.createElement("button");GJ.setAttribute("class","cb-chat-send"),GJ.setAttribute("type","button"),z.push(JJ(GJ,"disabled",()=>!Q.bar.value().trim())),z.push(XJ(GJ,"click",function(){let m=Q.sendMessage;return typeof m==="function"?m.bind(void 0):()=>{}}(),{modifiers:[]}));let UJ=document.createTextNode("Send");if(GJ.appendChild(UJ),FJ.appendChild(GJ),b.appendChild(FJ),j!=null){let m=Object.getOwnPropertyDescriptor(j,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(Q,{get(v,w){return v[w]},set(v,w,y){let N=v[w];if(N!=null&&typeof N.set==="function"){if(typeof y==="function")N.set(()=>y);else N.set(y);return!0}return!1}});if(m.mount){let v=m.mount(p,Z);if(typeof v==="function")z.push(v)}if(m.unmount)z.push(()=>m.unmount(p))}}return Q}});var rJ=NJ({scope:(J,X)=>{let Z=yJ(X,"sui-command-bar"),z=QJ(()=>{return{command:"/",find:">",mention:"@",create:"+",ai:"ai"}[Z.mode()]??">"}),Q=QJ(()=>{return{command:"Run a Ship action…",find:"Jump to a view…",mention:"Reference an agent, view, file…",create:"Create a view, skill, agent…",ai:"Ask Ship or type / for actions"}[Z.mode()]??"Search…"}),j=QJ(()=>Z.session()?.agent_id??"No session"),q=QJ(()=>{let I=Z.session();if(I?.status==="thinking")return"var(--warning,#d97706)";if(I)return"var(--success,#16a34a)";return"var(--muted-foreground)"}),K=QJ(()=>Z.allSessions().filter((I)=>I.status!=="ended"&&I.agent_id));function B(I){if(I==="view")return"app-window";if(I==="agent")return"bot";return"message-square-plus"}let D=QJ(()=>{let I=Z.value().replace(/^>+\s*/,"").toLowerCase().trim();return iJ.filter((f)=>!I||f.label.toLowerCase().includes(I)||f.keywords.toLowerCase().includes(I)||f.path.toLowerCase().includes(I)).map((f,H)=>({...f,idx:H}))}),R=QJ(()=>{let I=Z.value().replace(/^\/+\s*/,"").toLowerCase().trim();return nJ.filter((f)=>!I||f.key.includes(I)||f.label.toLowerCase().includes(I)||f.hint.toLowerCase().includes(I)).map((f,H)=>({...f,idx:H}))}),b=QJ(()=>{let I=Z.value().replace(/^@+/,"").toLowerCase().trim();return[...Z.views().map((H)=>({kind:"view",id:H,label:H})),...Z.agents().map((H)=>({kind:"agent",id:H.id,label:H.id})),...Z.allSessions().filter((H)=>H.status!=="ended"&&H.agent_id).map((H)=>({kind:"session",id:H.id,label:`${H.agent_id} (session)`}))].filter((H)=>!I||H.id.toLowerCase().includes(I)).slice(0,40).map((H,i)=>({...H,idx:i,icon:B(H.kind)}))}),O=QJ(()=>pJ),x=QJ(()=>pJ.find((I)=>I.id===Z.createSurface())),n=QJ(()=>x0(Z.createName()));function d(I){fetch(`/api/v1/sessions/${I}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}).then(()=>{if(Z.session()?.id===I)Z.session.set(null);ZJ()}),Z.sessionPickerOpen.set(!1)}function s(I){let f=Z.allSessions().find((H)=>H.id===I);if(f)Z.session.set(f),Z.thread.set([]);Z.sessionPickerOpen.set(!1)}function zJ(){Z.sessionPickerOpen.set(!1),Z.mode.set("ai"),Z.session.set(null)}function c(){Z.sessionPickerOpen.set(!1),DJ("/chats"),Z.open.set(!1)}let WJ=X;function u(I){WJ?.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:"find",item:{path:I}},bubbles:!0,composed:!0})),DJ(I),Z.value.set(""),Z.open.set(!1)}function a(I){switch(Z.value.set(""),I){case"new":Z.mode.set("create");break;case"chats":DJ("/chats"),Z.open.set(!1);break;case"settings":DJ("/settings"),Z.open.set(!1);break;case"end":if(Z.session())d(Z.session().id);Z.open.set(!1);break;case"close":Z.open.set(!1);break}}function _(I){let f=I.kind==="view"?`@${I.id}`:`@${I.kind}:${I.id}`,H=Z.value();Z.value.set(H.replace(/@[^@\s]*$/,`${f} `)),Z.mode.set(fJ(Z.value(),Z.mode()))}async function ZJ(){try{let I=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(I.ok){let f=await I.json();Z.allSessions.set(f.sessions);let H=f.sessions.find((i)=>i.status!=="ended"&&i.agent_id);if(H&&H.id!==Z.session()?.id)Z.session.set(H),Z.thread.set([]);else if(!H)Z.session.set(null)}}catch{}}return{bar:Z,modeGlyph:z,searchPlaceholder:Q,chipLabel:j,chipDotBg:q,activeSessions:K,findItems:D,cmdItems:R,mentionItems:b,createSurfaces:O,currentSurfaceDef:x,createNameValid:n,endSession:d,selectSession:s,startNewSession:zJ,viewChats:c,activateNavItem:u,activateCmd:a,activateMention:_}},on:{onInput:(J,X)=>{let Z=X.target.value;J.bar.value.set(Z),J.bar.mode.set(fJ(Z,J.bar.mode())),J.bar.selectedIdx.set(0)},toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},selectSurface:(J,X)=>{J.bar.createSurface.set(X);let Z=pJ.find((z)=>z.id===X);if(Z?.shapes[0])J.bar.createShape.set(Z.shapes[0])},spawnSession:async(J)=>{let X=J.bar.spawnAgent(),Z=J.bar.spawnProvider();if(!X||!Z)return;J.bar.spawning.set(!0),J.bar.spawnError.set(null);try{let z=await fetch("/api/v1/sessions",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({agent_id:X,provider:Z})});if(!z.ok){let Q=await z.json().catch(()=>({}));J.bar.spawnError.set(Q.error??`HTTP ${z.status}`)}else J.bar.mode.set("ai")}catch(z){J.bar.spawnError.set(z instanceof Error?z.message:"Unknown error")}finally{J.bar.spawning.set(!1)}},submitCreate:async(J)=>{if(!J.createNameValid()||J.bar.createSubmitting())return;J.bar.createSubmitting.set(!0),J.bar.createError.set(null);try{let X=await fetch("/api/v1/scaffold",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({surface:J.bar.createSurface(),name:J.bar.createName(),shape:J.bar.createShape(),purpose:J.bar.createPurpose()})});if(!X.ok){let Z=await X.json().catch(()=>({}));J.bar.createError.set(Z.error??`HTTP ${X.status}`)}else J.bar.createName.set(""),J.bar.createPurpose.set(""),J.bar.open.set(!1)}catch(X){J.bar.createError.set(X instanceof Error?X.message:"Unknown error")}finally{J.bar.createSubmitting.set(!1)}}}});var Cz=BJ("sui-command-palette",{shadow:!0,styles:`
  :host { display: flex; flex-direction: column; flex: 1; min-height: 0; overflow: hidden; }

  /* ── Input row ────────────────────────────────────────────────── */
  .cb-input-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
    flex: 0 0 auto;
  }
  .cb-mode-glyph {
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 600;
    color: var(--muted-foreground);
    flex-shrink: 0;
    min-width: 16px;
    text-align: center;
  }
  .cb-mode-glyph.ai {
    font-size: 10px;
    padding: 1px 4px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--muted);
  }
  .cb-search {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    font: inherit;
    font-size: 14px;
    color: var(--foreground);
    caret-color: var(--foreground);
  }
  .cb-search::placeholder { color: var(--muted-foreground); }
  .cb-esc {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 5px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--muted);
    color: var(--muted-foreground);
    flex-shrink: 0;
  }

  /* ── Session chip ─────────────────────────────────────────────── */
  .cb-chip-wrap { position: relative; flex-shrink: 0; }
  .cb-session-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    font-size: 11px;
    color: var(--muted-foreground);
    cursor: pointer;
    user-select: none;
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cb-session-chip:hover { background: var(--muted); }
  .cb-dot {
    width: 6px; height: 6px;
    border-radius: 999px;
    flex-shrink: 0;
    display: inline-block;
  }

  /* ── Session popover ──────────────────────────────────────────── */
  .cb-session-popover {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    min-width: 280px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-lg);
    z-index: 10;
    padding: 4px 0;
    overflow: hidden;
  }
  .cb-session-row {
    display: grid;
    grid-template-columns: 14px 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 7px 12px;
    cursor: pointer;
    font-size: 12px;
  }
  .cb-session-row:hover { background: var(--muted); }
  .cb-session-row.current { background: color-mix(in oklch, var(--primary) 8%, transparent); }
  .cb-session-row .meta { color: var(--muted-foreground); font-size: 11px; }
  .cb-session-end {
    background: transparent; border: none; cursor: pointer;
    color: var(--muted-foreground); font-size: 14px; padding: 0 2px;
  }
  .cb-session-end:hover { color: var(--destructive, red); }
  .cb-session-sep { height: 1px; background: var(--border); margin: 4px 0; }
  .cb-session-action {
    display: flex; align-items: center; gap: 8px;
    padding: 7px 12px; cursor: pointer; font-size: 12px;
    color: var(--muted-foreground);
  }
  .cb-session-action:hover { background: var(--muted); color: var(--foreground); }

  /* ── Body / list ──────────────────────────────────────────────── */
  .cb-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-width: thin;
    padding: 6px 0;
  }
  .cb-mode-hint {
    padding: 4px 14px 2px;
    font-size: 11px;
    font-weight: 600;
    color: var(--muted-foreground);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .cb-list { padding: 2px 6px; }
  .cb-item {
    display: grid;
    grid-template-columns: 16px 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 7px 8px;
    border-radius: var(--radius-sm, 4px);
    cursor: pointer;
    font-size: 13px;
  }
  .cb-item:hover,
  .cb-item[aria-selected='true'] { background: var(--muted); }
  .cb-item .hint {
    color: var(--muted-foreground);
    font-size: 11px;
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cb-empty {
    padding: 24px 16px;
    text-align: center;
    color: var(--muted-foreground);
    font-size: 13px;
  }

  /* ── Create form ──────────────────────────────────────────────── */
  .cb-create-surfaces {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 8px 14px;
  }
  .cb-surface-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: transparent;
    font: inherit;
    font-size: 12px;
    color: var(--muted-foreground);
    cursor: pointer;
  }
  .cb-surface-btn[aria-pressed='true'] {
    background: var(--primary);
    color: var(--primary-foreground, #fff);
    border-color: var(--primary);
  }
  .cb-surface-btn:hover:not([aria-pressed='true']) { background: var(--muted); }
  .cb-create-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    padding: 0 14px 10px;
  }
  .cb-create-grid label { display: flex; flex-direction: column; gap: 4px; font-size: 12px; color: var(--muted-foreground); }
  .cb-create-grid input,
  .cb-create-grid select {
    height: 30px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    background: var(--background);
    color: var(--foreground);
    font: inherit;
    font-size: 13px;
    padding: 0 8px;
  }
  .span-2 { grid-column: span 2; }
  .cb-create-error { padding: 0 14px 6px; font-size: 12px; color: var(--destructive, red); }
  .cb-create-actions { padding: 0 14px 14px; display: flex; justify-content: flex-end; }
  .cb-create-submit {
    height: 32px;
    padding: 0 16px;
    border: none;
    border-radius: var(--radius-sm, 4px);
    background: var(--primary);
    color: var(--primary-foreground, #fff);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .cb-create-submit:disabled { opacity: 0.5; cursor: not-allowed; }

  /* ── AI spawn card ────────────────────────────────────────────── */
  .cb-spawn-card {
    margin: 20px 16px;
    padding: 16px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    background: var(--muted);
  }
  .cb-spawn-title { font-weight: 600; font-size: 13px; margin-bottom: 12px; }
  .cb-spawn-row {
    display: grid;
    grid-template-columns: 64px 1fr;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }
  .cb-spawn-row label { font-size: 12px; color: var(--muted-foreground); }
  .cb-spawn-row select {
    height: 30px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    background: var(--background);
    color: var(--foreground);
    font: inherit;
    font-size: 13px;
    padding: 0 6px;
  }
  .cb-spawn-error { font-size: 12px; color: var(--destructive, red); margin-bottom: 8px; }
  .cb-spawn-start {
    margin-top: 4px;
    height: 32px;
    width: 100%;
    border: none;
    border-radius: var(--radius-sm, 4px);
    background: var(--primary);
    color: var(--primary-foreground, #fff);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .cb-spawn-start:disabled { opacity: 0.5; cursor: not-allowed; }
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null),j=null;if(rJ&&typeof rJ.setup==="function"){if(J.scope=Q,j=rJ.setup(J),j&&typeof j==="object")Object.assign(Q,j)}let q=(N)=>MJ({anchor:N,items:()=>Q.activeSessions(),key:(G,L)=>{let W=Object.create(Q);return W.s=G,W.__i=L,((Y)=>G.id)(W)},render:(G,L)=>{let W=Object.create(Q);return W.s=G,W.__i=L,W.__v=S(0),((Y)=>{let P=document.createDocumentFragment(),M=[],V=document.createElement("div");V.setAttribute("class","cb-session-row"),M.push(EJ(V,"current",()=>(Y.__v(),Y.bar.session()?.id===Y.s.id))),V.setAttribute("role","option"),M.push(XJ(V,"click",function(){let h=()=>Y.selectSession(Y.s.id);return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("span");T.setAttribute("class","cb-dot"),M.push(l(()=>{Y.__v();let h=Y.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(h==null)T.style.removeProperty("background");else T.style.setProperty("background",String(h))})),V.appendChild(T);let k=document.createElement("span"),$=document.createElement("strong"),A=document.createTextNode("");$.appendChild(A),M.push(g(A,()=>(Y.__v(),Y.s.agent_id??""))),k.appendChild($);let F=document.createElement("span");F.setAttribute("class","meta");let U=document.createTextNode(" · ");F.appendChild(U);let C=document.createTextNode("");F.appendChild(C),M.push(g(C,()=>(Y.__v(),Y.s.provider??"")));let E=document.createTextNode(" · ");F.appendChild(E);let YJ=document.createTextNode("");F.appendChild(YJ),M.push(g(YJ,()=>(Y.__v(),Y.s.workspace_id))),k.appendChild(F),V.appendChild(k);let o=document.createElement("button");o.setAttribute("class","cb-session-end"),o.setAttribute("type","button"),o.setAttribute("aria-label","End session"),M.push(XJ(o,"click",function(){let h=(VJ)=>{VJ.stopPropagation(),Y.endSession(Y.s.id)};return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let qJ=document.createTextNode("×");return o.appendChild(qJ),V.appendChild(o),P.appendChild(V),{nodes:Array.from(P.childNodes),cleanup:()=>{for(let h of M)try{h()}catch(VJ){}},update:(h,VJ)=>{Y.s=h,Y.__i=VJ,Y.__v.set((vJ)=>vJ+1)}}})(W)}}),K=(N)=>jJ({anchor:N,test:()=>{if(Q.activeSessions().length===0)return 0;return-1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-session-row");let Y=document.createElement("span");W.appendChild(Y);let P=document.createElement("span");P.setAttribute("class","meta");let M=document.createTextNode("No active sessions");P.appendChild(M),W.appendChild(P);let V=document.createElement("span");return W.appendChild(V),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let T of L)try{T()}catch(k){}}}}]}),B=(N)=>jJ({anchor:N,test:()=>{if(Q.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-session-popover"),W.setAttribute("role","listbox"),W.setAttribute("aria-label","Sessions");let Y=document.createComment("for");W.appendChild(Y),L.push(q(Y));let P=document.createComment("if");W.appendChild(P),L.push(K(P));let M=document.createElement("div");M.setAttribute("class","cb-session-sep"),M.setAttribute("role","separator"),W.appendChild(M);let V=document.createElement("div");V.setAttribute("class","cb-session-action"),L.push(XJ(V,"click",function(){let E=Q.startNewSession;return typeof E==="function"?E.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");T.setAttribute("name","plus"),T.setAttribute("size","sm"),T.setAttribute("aria-hidden","true"),V.appendChild(T);let k=document.createElement("span"),$=document.createTextNode("Start new session");k.appendChild($),V.appendChild(k),W.appendChild(V);let A=document.createElement("div");A.setAttribute("class","cb-session-action"),L.push(XJ(A,"click",function(){let E=Q.viewChats;return typeof E==="function"?E.bind(void 0):()=>{}}(),{modifiers:[]}));let F=document.createElement("sui-icon");F.setAttribute("name","columns"),F.setAttribute("size","sm"),F.setAttribute("aria-hidden","true"),A.appendChild(F);let U=document.createElement("span"),C=document.createTextNode("View all chats");return U.appendChild(C),A.appendChild(U),W.appendChild(A),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let E of L)try{E()}catch(YJ){}}}}]}),D=(N)=>MJ({anchor:N,items:()=>Q.findItems(),key:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,((Y)=>G.path)(W)},render:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,W.__v=S(0),((Y)=>{let P=document.createDocumentFragment(),M=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),M.push(JJ(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false"))),M.push(JJ(V,"data-path",()=>(Y.__v(),Y.item.path))),M.push(XJ(V,"click",function(){let U=()=>Y.activateNavItem(Y.item.path);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");M.push(JJ(T,"name",()=>(Y.__v(),Y.item.icon))),T.setAttribute("size","sm"),T.setAttribute("aria-hidden","true"),V.appendChild(T);let k=document.createElement("span"),$=document.createTextNode("");k.appendChild($),M.push(g($,()=>(Y.__v(),Y.item.label))),V.appendChild(k);let A=document.createElement("span");A.setAttribute("class","hint");let F=document.createTextNode("");return A.appendChild(F),M.push(g(F,()=>(Y.__v(),Y.item.path))),V.appendChild(A),P.appendChild(V),{nodes:Array.from(P.childNodes),cleanup:()=>{for(let U of M)try{U()}catch(C){}},update:(U,C)=>{Y.item=U,Y.__i=C,Y.__v.set((E)=>E+1)}}})(W)}}),R=(N)=>jJ({anchor:N,test:()=>{if(Q.findItems().length>0)return 0;return 1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Jump to");W.appendChild(Y),G.appendChild(W);let P=document.createElement("div");P.setAttribute("class","cb-list"),P.setAttribute("role","listbox");let M=document.createComment("for");return P.appendChild(M),L.push(D(M)),G.appendChild(P),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(T){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No matches.");return W.appendChild(Y),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}}}}]}),b=(N)=>MJ({anchor:N,items:()=>Q.cmdItems(),key:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,((Y)=>G.key)(W)},render:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,W.__v=S(0),((Y)=>{let P=document.createDocumentFragment(),M=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),M.push(JJ(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false"))),M.push(JJ(V,"data-cmd",()=>(Y.__v(),Y.item.key))),M.push(XJ(V,"click",function(){let U=()=>Y.activateCmd(Y.item.key);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("span");T.setAttribute("style","font-family:var(--font-mono)");let k=document.createTextNode("");T.appendChild(k),M.push(g(k,()=>(Y.__v(),Y.item.label))),V.appendChild(T);let $=document.createElement("span");V.appendChild($);let A=document.createElement("span");A.setAttribute("class","hint");let F=document.createTextNode("");return A.appendChild(F),M.push(g(F,()=>(Y.__v(),Y.item.hint))),V.appendChild(A),P.appendChild(V),{nodes:Array.from(P.childNodes),cleanup:()=>{for(let U of M)try{U()}catch(C){}},update:(U,C)=>{Y.item=U,Y.__i=C,Y.__v.set((E)=>E+1)}}})(W)}}),O=(N)=>jJ({anchor:N,test:()=>{if(Q.cmdItems().length>0)return 0;return 1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Actions");W.appendChild(Y),G.appendChild(W);let P=document.createElement("div");P.setAttribute("class","cb-list"),P.setAttribute("role","listbox");let M=document.createComment("for");return P.appendChild(M),L.push(b(M)),G.appendChild(P),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(T){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No commands match.");return W.appendChild(Y),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}}}}]}),x=(N)=>MJ({anchor:N,items:()=>Q.mentionItems(),key:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,((Y)=>G.id+G.kind)(W)},render:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,W.__v=S(0),((Y)=>{let P=document.createDocumentFragment(),M=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),M.push(JJ(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false"))),M.push(JJ(V,"data-mention-kind",()=>(Y.__v(),Y.item.kind))),M.push(JJ(V,"data-mention-id",()=>(Y.__v(),Y.item.id))),M.push(XJ(V,"click",function(){let U=()=>Y.activateMention(Y.item);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");M.push(JJ(T,"name",()=>(Y.__v(),Y.item.icon))),T.setAttribute("size","sm"),T.setAttribute("aria-hidden","true"),V.appendChild(T);let k=document.createElement("span"),$=document.createTextNode("");k.appendChild($),M.push(g($,()=>(Y.__v(),Y.item.label))),V.appendChild(k);let A=document.createElement("span");A.setAttribute("class","hint");let F=document.createTextNode("");return A.appendChild(F),M.push(g(F,()=>(Y.__v(),Y.item.kind))),V.appendChild(A),P.appendChild(V),{nodes:Array.from(P.childNodes),cleanup:()=>{for(let U of M)try{U()}catch(C){}},update:(U,C)=>{Y.item=U,Y.__i=C,Y.__v.set((E)=>E+1)}}})(W)}}),n=(N)=>jJ({anchor:N,test:()=>{if(Q.mentionItems().length>0)return 0;return 1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Reference");W.appendChild(Y),G.appendChild(W);let P=document.createElement("div");P.setAttribute("class","cb-list"),P.setAttribute("role","listbox");let M=document.createComment("for");return P.appendChild(M),L.push(x(M)),G.appendChild(P),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(T){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No matching views or agents.");return W.appendChild(Y),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}}}}]}),d=(N)=>MJ({anchor:N,items:()=>Q.createSurfaces(),key:(G,L)=>{let W=Object.create(Q);return W.surf=G,W.__i=L,((Y)=>G.id)(W)},render:(G,L)=>{let W=Object.create(Q);return W.surf=G,W.__i=L,W.__v=S(0),((Y)=>{let P=document.createDocumentFragment(),M=[],V=document.createElement("button");V.setAttribute("class","cb-surface-btn"),V.setAttribute("type","button"),M.push(JJ(V,"aria-pressed",()=>(Y.__v(),Y.bar.createSurface()===Y.surf.id?"true":"false"))),M.push(XJ(V,"click",function(){let A=()=>Y.selectSurface(Y.surf.id);return typeof A==="function"?A.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");M.push(JJ(T,"name",()=>(Y.__v(),Y.surf.icon))),T.setAttribute("aria-hidden","true"),V.appendChild(T);let k=document.createElement("span"),$=document.createTextNode("");return k.appendChild($),M.push(g($,()=>(Y.__v(),Y.surf.label))),V.appendChild(k),P.appendChild(V),{nodes:Array.from(P.childNodes),cleanup:()=>{for(let A of M)try{A()}catch(F){}},update:(A,F)=>{Y.surf=A,Y.__i=F,Y.__v.set((U)=>U+1)}}})(W)}}),s=(N)=>MJ({anchor:N,items:()=>Q.currentSurfaceDef().shapes,key:(G,L)=>{let W=Object.create(Q);return W.sh=G,W.__i=L,((Y)=>G)(W)},render:(G,L)=>{let W=Object.create(Q);return W.sh=G,W.__i=L,W.__v=S(0),((Y)=>{let P=document.createDocumentFragment(),M=[],V=document.createElement("option");M.push(JJ(V,"value",()=>(Y.__v(),Y.sh)));let T=document.createTextNode("");return V.appendChild(T),M.push(g(T,()=>(Y.__v(),Y.sh))),P.appendChild(V),{nodes:Array.from(P.childNodes),cleanup:()=>{for(let k of M)try{k()}catch($){}},update:(k,$)=>{Y.sh=k,Y.__i=$,Y.__v.set((A)=>A+1)}}})(W)}}),zJ=(N)=>jJ({anchor:N,test:()=>{if(Q.bar.createError())return 0;return-1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-create-error");let Y=document.createTextNode("");return W.appendChild(Y),L.push(g(Y,()=>Q.bar.createError())),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}}}}]}),c=(N)=>MJ({anchor:N,items:()=>Q.bar.agents(),key:(G,L)=>{let W=Object.create(Q);return W.a=G,W.__i=L,((Y)=>G.id)(W)},render:(G,L)=>{let W=Object.create(Q);return W.a=G,W.__i=L,W.__v=S(0),((Y)=>{let P=document.createDocumentFragment(),M=[],V=document.createElement("option");M.push(JJ(V,"value",()=>(Y.__v(),Y.a.id)));let T=document.createTextNode("");return V.appendChild(T),M.push(g(T,()=>(Y.__v(),Y.a.name??Y.a.label??Y.a.id))),P.appendChild(V),{nodes:Array.from(P.childNodes),cleanup:()=>{for(let k of M)try{k()}catch($){}},update:(k,$)=>{Y.a=k,Y.__i=$,Y.__v.set((A)=>A+1)}}})(W)}}),WJ=(N)=>jJ({anchor:N,test:()=>{if(Q.bar.agents().length===0)return 0;return 1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("option");W.setAttribute("value","");let Y=document.createTextNode("No agents found");return W.appendChild(Y),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createComment("for");return G.appendChild(W),L.push(c(W)),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(P){}}}}]}),u=(N)=>MJ({anchor:N,items:()=>Q.bar.providers(),key:(G,L)=>{let W=Object.create(Q);return W.p=G,W.__i=L,((Y)=>G)(W)},render:(G,L)=>{let W=Object.create(Q);return W.p=G,W.__i=L,W.__v=S(0),((Y)=>{let P=document.createDocumentFragment(),M=[],V=document.createElement("option");M.push(JJ(V,"value",()=>(Y.__v(),Y.p)));let T=document.createTextNode("");return V.appendChild(T),M.push(g(T,()=>(Y.__v(),Y.p))),P.appendChild(V),{nodes:Array.from(P.childNodes),cleanup:()=>{for(let k of M)try{k()}catch($){}},update:(k,$)=>{Y.p=k,Y.__i=$,Y.__v.set((A)=>A+1)}}})(W)}}),a=(N)=>jJ({anchor:N,test:()=>{if(Q.bar.providers().length===0)return 0;return 1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("option");W.setAttribute("value","");let Y=document.createTextNode("No providers installed");return W.appendChild(Y),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createComment("for");return G.appendChild(W),L.push(u(W)),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(P){}}}}]}),_=(N)=>jJ({anchor:N,test:()=>{if(Q.bar.spawnError())return 0;return-1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-spawn-error");let Y=document.createTextNode("");return W.appendChild(Y),L.push(g(Y,()=>Q.bar.spawnError())),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let P of L)try{P()}catch(M){}}}}]}),ZJ=(N)=>jJ({anchor:N,test:()=>{let G=Q.bar.mode();if(G==="find")return 0;if(G==="command")return 1;if(G==="mention")return 2;if(G==="create")return 3;return 4},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createComment("if");return G.appendChild(W),L.push(R(W)),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(P){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createComment("if");return G.appendChild(W),L.push(O(W)),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(P){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createComment("if");return G.appendChild(W),L.push(n(W)),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(P){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Create");W.appendChild(Y),G.appendChild(W);let P=document.createElement("div");P.setAttribute("class","cb-create-surfaces"),P.setAttribute("role","group"),P.setAttribute("aria-label","Surface type");let M=document.createComment("for");P.appendChild(M),L.push(d(M)),G.appendChild(P);let V=document.createElement("div");V.setAttribute("class","cb-create-grid");let T=document.createElement("label"),k=document.createElement("span"),$=document.createTextNode("Name (kebab-case)");k.appendChild($),T.appendChild(k);let A=document.createElement("input");A.setAttribute("type","text"),A.setAttribute("class","cb-create-name"),L.push(JJ(A,"placeholder",()=>"my-"+Q.bar.createSurface())),L.push(HJ(A,Q.bar.createName)),A.setAttribute("autocomplete","off"),T.appendChild(A),V.appendChild(T);let F=document.createElement("label"),U=document.createElement("span"),C=document.createTextNode("");U.appendChild(C),L.push(g(C,()=>Q.currentSurfaceDef().shapeLabel)),F.appendChild(U);let E=document.createElement("select");E.setAttribute("class","cb-create-shape"),L.push(HJ(E,Q.bar.createShape));let YJ=document.createComment("for");E.appendChild(YJ),L.push(s(YJ)),F.appendChild(E),V.appendChild(F);let o=document.createElement("label");o.setAttribute("class","span-2");let qJ=document.createElement("span"),h=document.createTextNode("Purpose (optional)");qJ.appendChild(h),o.appendChild(qJ);let VJ=document.createElement("input");VJ.setAttribute("type","text"),VJ.setAttribute("class","cb-create-purpose"),VJ.setAttribute("placeholder","One-line description"),L.push(HJ(VJ,Q.bar.createPurpose)),VJ.setAttribute("autocomplete","off"),o.appendChild(VJ),V.appendChild(o),G.appendChild(V);let vJ=document.createComment("if");G.appendChild(vJ),L.push(zJ(vJ));let oJ=document.createElement("div");oJ.setAttribute("class","cb-create-actions");let CJ=document.createElement("button");CJ.setAttribute("class","cb-create-submit"),CJ.setAttribute("type","button"),L.push(JJ(CJ,"disabled",()=>Q.bar.createSubmitting()||!Q.createNameValid())),L.push(XJ(CJ,"click",function(){let mJ=Q.submitCreate;return typeof mJ==="function"?mJ.bind(void 0):()=>{}}(),{modifiers:[]}));let L0=document.createTextNode("");return CJ.appendChild(L0),L.push(g(L0,()=>Q.bar.createSubmitting()?"Creating…":"Create "+Q.currentSurfaceDef().label)),oJ.appendChild(CJ),G.appendChild(oJ),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let mJ of L)try{mJ()}catch(jX){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-spawn-card");let Y=document.createElement("div");Y.setAttribute("class","cb-spawn-title");let P=document.createTextNode("Start a session");Y.appendChild(P),W.appendChild(Y);let M=document.createElement("div");M.setAttribute("class","cb-spawn-row");let V=document.createElement("label"),T=document.createTextNode("Agent");V.appendChild(T),M.appendChild(V);let k=document.createElement("select");k.setAttribute("class","cb-spawn-agent"),L.push(HJ(k,Q.bar.spawnAgent));let $=document.createComment("if");k.appendChild($),L.push(WJ($)),M.appendChild(k),W.appendChild(M);let A=document.createElement("div");A.setAttribute("class","cb-spawn-row");let F=document.createElement("label"),U=document.createTextNode("Provider");F.appendChild(U),A.appendChild(F);let C=document.createElement("select");C.setAttribute("class","cb-spawn-provider"),L.push(HJ(C,Q.bar.spawnProvider));let E=document.createComment("if");C.appendChild(E),L.push(a(E)),A.appendChild(C),W.appendChild(A);let YJ=document.createComment("if");W.appendChild(YJ),L.push(_(YJ));let o=document.createElement("button");o.setAttribute("class","cb-spawn-start"),o.setAttribute("type","button"),L.push(JJ(o,"disabled",()=>Q.bar.spawning()||!Q.bar.spawnAgent()||!Q.bar.spawnProvider())),L.push(XJ(o,"click",function(){let h=Q.spawnSession;return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let qJ=document.createTextNode("");return o.appendChild(qJ),L.push(g(qJ,()=>Q.bar.spawning()?"Starting…":"Start session")),W.appendChild(o),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let h of L)try{h()}catch(VJ){}}}}]}),I=Z.shadowRoot??Z;IJ(I);let f=document.createElement("div");f.setAttribute("class","cb-input-row");let H=document.createElement("span");H.setAttribute("class","cb-mode-glyph"),z.push(EJ(H,"ai",()=>Q.bar.mode()==="ai"));let i=document.createTextNode("");H.appendChild(i),z.push(g(i,()=>Q.modeGlyph())),f.appendChild(H);let r=document.createElement("input");r.setAttribute("class","cb-search"),r.setAttribute("type","text"),r.setAttribute("autocomplete","off"),r.setAttribute("spellcheck","false"),z.push(JJ(r,"placeholder",()=>Q.searchPlaceholder())),z.push(HJ(r,Q.bar.value)),r.setAttribute("aria-label","Command bar search"),r.setAttribute("aria-autocomplete","list"),z.push(XJ(r,"input",function(){let N=Q.onInput;return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]})),f.appendChild(r);let LJ=document.createElement("kbd");LJ.setAttribute("class","cb-esc"),LJ.setAttribute("aria-label","Escape to close");let FJ=document.createTextNode("esc");LJ.appendChild(FJ),f.appendChild(LJ);let e=document.createElement("div");e.setAttribute("class","cb-chip-wrap");let GJ=document.createElement("span");GJ.setAttribute("class","cb-session-chip"),GJ.setAttribute("role","button"),GJ.setAttribute("aria-haspopup","listbox"),z.push(XJ(GJ,"click",function(){let N=Q.toggleSessionPicker;return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]}));let UJ=document.createElement("span");UJ.setAttribute("class","cb-dot"),z.push(l(()=>{let N=Q.chipDotBg();if(N==null)UJ.style.removeProperty("background");else UJ.style.setProperty("background",String(N))})),GJ.appendChild(UJ);let m=document.createElement("span"),p=document.createTextNode("");m.appendChild(p),z.push(g(p,()=>Q.chipLabel())),GJ.appendChild(m),e.appendChild(GJ);let v=document.createComment("if");e.appendChild(v),z.push(B(v)),f.appendChild(e),I.appendChild(f);let w=document.createElement("div");w.setAttribute("class","cb-body");let y=document.createComment("match");if(w.appendChild(y),z.push(ZJ(y)),I.appendChild(w),j!=null){let N=Object.getOwnPropertyDescriptor(j,"__shipSetLifecycle")?.value;if(N){let G=new Proxy(Q,{get(L,W){return L[W]},set(L,W,Y){let P=L[W];if(P!=null&&typeof P.set==="function"){if(typeof Y==="function")P.set(()=>Y);else P.set(Y);return!0}return!1}});if(N.mount){let L=N.mount(G,Z);if(typeof L==="function")z.push(L)}if(N.unmount)z.push(()=>N.unmount(G))}}return Q}});
