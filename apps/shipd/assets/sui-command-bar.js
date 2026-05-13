var k0=Symbol.for("sui:error-boundary");function OJ(J){let X=J;while(X){if(X instanceof Element){let Q=X[k0];if(Q)return Q}let Z=X.parentNode;if(Z){X=Z;continue}let z=X;if(z.host){X=z.host;continue}X=null}return null}function TJ(J,X,Z){try{return Z()}catch(z){let Q=OJ(J);if(Q){Q.catch(z,X);return}console.error("[sui] uncaught throw (no error boundary):",z);return}}var HJ=null,BJ=null,S0=0,y0=new Set,wJ=[];function sJ(J,X){wJ.push(J);try{return X()}finally{wJ.pop()}}function b0(){return wJ.length>0?wJ[wJ.length-1]:null}function M0(J){if(HJ===null)return;if(J.has(HJ))return;let X=HJ;if(J.add(X),BJ!==null)BJ.push(()=>J.delete(X))}function k(J){let X=J,Z=new Set,z=()=>{return M0(Z),X};return z.set=(Q)=>{let j=typeof Q==="function"?Q(X):Q;if(Object.is(j,X))return;if(X=j,S0>0)for(let M of Z)y0.add(M);else for(let M of[...Z])M()},z.__subs=Z,z.__peek=()=>X,z}function JJ(J){let X,Z=!0,z=new Set,Q=[],j=()=>{if(Z)return;Z=!0;for(let P of[...z])P()},M=()=>{for(let T of Q)T();Q=[];let P=HJ,H=BJ;HJ=j,BJ=Q;try{X=J()}finally{HJ=P,BJ=H}Z=!1},B=()=>{if(M0(z),Z)M();return X};return B.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},B.__subs=z,B.__peek=()=>{if(Z)M();return X},B}function d(J){let X=void 0,Z=!0,z=[],Q=b0(),j=()=>{if(!Z)return;if(X){try{X()}catch(P){console.error("[sui] effect cleanup threw:",P)}X=void 0}for(let P of z)P();z=[];let M=HJ,B=BJ;HJ=j,BJ=z;try{X=J()}catch(P){let H=Q?OJ(Q):null;if(H)H.catch(P,"effect");else console.error("[sui] effect threw:",P)}finally{HJ=M,BJ=B}};return j(),()=>{Z=!1;for(let M of z)M();if(z=[],X){try{X()}catch(M){console.error("[sui] effect cleanup threw:",M)}X=void 0}}}var gJ=null;function xJ(J,X){let Z=gJ;gJ=J;try{return X()}finally{gJ=Z}}function kJ(J){let X=gJ;if(!X)return;X.cleanup(J)}function SJ(){let J=[],X=!1;return{cleanup(z){if(X){try{z()}catch(Q){console.error("[sui/scope] late cleanup threw:",Q)}return}J.push(z)},dispose(){if(X)return;X=!0;for(let z=J.length-1;z>=0;z--)try{J[z]()}catch(Q){console.error("[sui/scope] cleanup threw:",Q)}J.length=0},get disposed(){return X}}}var V0=Symbol("sui.shared");function r(J){return J[V0]=!0,J}function q0(J){return!!J?.[V0]}function tJ(J,X,Z){J[`__sui_ctx_${X}`]=Z}function yJ(J,X){let Z=J;while(Z){let z=Z[`__sui_ctx_${X}`];if(z!==void 0)return z;let Q=Z.parentElement;if(Q){Z=Q;continue}let j=Z.getRootNode();Z=j instanceof ShadowRoot?j.host:null}return null}function _J(J){let X={};for(let Z in J)if(Object.prototype.hasOwnProperty.call(J,Z)&&q0(J[Z]))X[Z]=J[Z];return X}function eJ(J=null){return k(null)}function J0(J,X,Z){let z=J[X];if(typeof z==="function"&&typeof z.set==="function")return z.set(Z),z;let Q=k(Z);return J[X]=Q,Q}function h(J,X){return d(()=>{let Z=X();J.textContent=Z==null?"":String(Z)})}function s(J,X,Z){return d(()=>{let z=Z();if(z==null||z===!1||z==="")J.removeAttribute(X);else if(z===!0)J.setAttribute(X,"");else J.setAttribute(X,String(z))})}function EJ(J,X,Z){return d(()=>{if(Z())J.classList.add(X);else J.classList.remove(X)})}function DJ(J,X,Z){let z=d(()=>{let j=X();if(J.value!==j)J.value=j}),Q=()=>{let j=J.value;if(X.set(j),Z)Z(j)};return J.addEventListener("input",Q),()=>{z(),J.removeEventListener("input",Q)}}function t(J,X,Z,z={}){let Q=z.modifiers??[],j,M=!0,B=(H)=>{for(let K of Q)switch(K.kind){case"preventDefault":H.preventDefault();break;case"stopPropagation":H.stopPropagation();break;case"stopImmediatePropagation":H.stopImmediatePropagation();break;case"self":if(H.target!==J)return;break;case"trusted":if(!H.isTrusted)return;break;case"key":{if(!F0(H,K.arg??""))return;break}}let T=Q.find((K)=>K.kind==="debounce"),$=Q.find((K)=>K.kind==="throttle");if(T){let K=Number(T.arg)||0;if(j!==void 0)clearTimeout(j);j=setTimeout(()=>{j=void 0,Z(H)},K);return}if($){let K=Number($.arg)||0;if(!M)return;M=!1,Z(H),j=setTimeout(()=>{M=!0},K);return}Z(H)},P={};if(z.capture)P.capture=!0;if(z.passive)P.passive=!0;if(z.once)P.once=!0;return J.addEventListener(X,B,P),()=>{if(J.removeEventListener(X,B,P),j!==void 0)clearTimeout(j)}}function F0(J,X){let Z=X.split("+").map((T)=>T.toLowerCase()),z=Z.pop()??"",Q=Z.includes("meta")||Z.includes("cmd"),j=Z.includes("ctrl")||Z.includes("control"),M=Z.includes("shift"),B=Z.includes("alt")||Z.includes("option"),P=Z.includes("mod"),H=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(Q&&!J.metaKey)return!1;if(j&&!J.ctrlKey)return!1;if(M&&!J.shiftKey)return!1;if(B&&!J.altKey)return!1;if(P&&!(H?J.metaKey:J.ctrlKey))return!1;return J.key.toLowerCase()===z}function D0(J,X,Z){J.dispatchEvent(new CustomEvent(X,{detail:Z,bubbles:!0,composed:!0}))}function H0(J,X){let Z=J,z=Object.keys(X);for(let Q of z)Z[Q]=X[Q];kJ(()=>{for(let Q of z)if(Z[Q]===X[Q])try{delete Z[Q]}catch{Z[Q]=void 0}})}function uJ(J){let X=J;X.emit=function(Z,z){D0(J,Z,z)},X.expose=function(Z){H0(J,Z)}}var $0=Symbol("sui:transition");function hJ(J){return J[$0]}function f0(){if(typeof window>"u"||typeof window.matchMedia!=="function")return!1;try{return window.matchMedia("(prefers-reduced-motion: reduce)").matches}catch{return!1}}function P0(J){if(!f0())return J;return{...J,duration:0,delay:0}}function U0(J){let X=hJ(J);if(!X?.inT?.in)return;X.current?.cancel();let Z=X.inT.in(J,P0(X.inOpts));return X.current=Z,Z}function B0(J){let X=hJ(J);if(!X?.outT?.out)return;X.current?.cancel();let Z=X.outT.out(J,P0(X.outOpts));return X.current=Z,Z}function X0(J){if(J.nodeType!==1)return;let X=J;if(X.hasAttribute("data-sui-transition")){let Z=hJ(X);if(Z?.current?.cancel(),Z)Z.current=void 0}for(let Z of X.querySelectorAll("[data-sui-transition]")){let z=hJ(Z);if(z?.current?.cancel(),z)z.current=void 0}}function K0(J,X){for(let Z of J){if(Z.nodeType!==1)continue;let z=Z;if(z.hasAttribute("data-sui-transition"))X(z);for(let Q of z.querySelectorAll("[data-sui-transition]"))X(Q)}}function RJ(J){K0(J,(X)=>{U0(X)})}function bJ(J){let X=[];if(K0(J,(Z)=>{let z=B0(Z);if(z)X.push(z.finished.catch(()=>{}))}),X.length===0)return null;return Promise.all(X).then(()=>{return})}function v0(J){let X=J.length,Z=Array(X).fill(!1);if(X===0)return Z;let z=Array(X).fill(-1),Q=[];for(let M=0;M<X;M++){let B=J[M],P=0,H=Q.length;while(P<H){let T=P+H>>>1;if(J[Q[T]]<B)P=T+1;else H=T}if(P>0)z[M]=Q[P-1];Q[P]=M}let j=Q[Q.length-1];while(j>=0)Z[j]=!0,j=z[j];return Z}function VJ(J){let{anchor:X,items:Z,key:z,render:Q}=J,j=[],M=!0,B=new Set,P=(T)=>{for(let $ of T.nodes)try{$.parentNode?.removeChild($)}catch{}T.cleanup?.()},H=d(()=>{if(!M)return;let T=Z(),$=new Map;for(let C=0;C<T.length;C++){let m=T[C],I=z(m,C);if($.has(I))console.warn("[sui] {#for}: duplicate key",I,"at index",C,"(prior at",$.get(I).index,"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");$.set(I,{item:m,index:C})}let K=new Map;for(let C of j)K.set(C.key,C);for(let C of j){if($.has(C.key))continue;let m=bJ(C.nodes);if(m===null){P(C);continue}B.add(C),m.then(()=>{if(!B.delete(C))return;P(C)})}let b=[],o=[];for(let C=0;C<T.length;C++){let m=T[C],I=z(m,C),f=K.get(I),S;if(f)f.item=m,TJ(X,"render",()=>f.update?.(m,C)),S=f;else{let QJ=TJ(X,"render",()=>Q(m,C));if(!QJ)continue;S={key:I,item:m,nodes:QJ.nodes,...QJ.cleanup&&{cleanup:QJ.cleanup},...QJ.update&&{update:QJ.update}},o.push(...QJ.nodes)}b.push(S)}let i=new Map;for(let C=0;C<j.length;C++)i.set(j[C].key,C);let n=[],XJ=[];for(let C=0;C<b.length;C++){let m=i.get(b[C].key);if(m!==void 0)n.push(C),XJ.push(m)}let a=v0(XJ),ZJ=new Set;for(let C=0;C<n.length;C++)if(a[C])ZJ.add(n[C]);let _=X.parentNode,e=X;for(let C=0;C<b.length;C++){if(!_)break;let m=b[C];if(ZJ.has(C))e=m.nodes[m.nodes.length-1]??e;else for(let I of m.nodes)_.insertBefore(I,e.nextSibling),e=I}if(j=b,o.length>0)RJ(o)});return()=>{M=!1,H();for(let T of j)P(T);for(let T of B)P(T);B.clear(),j=[]}}function WJ(J){let{anchor:X}=J,{test:Z,branches:z}=m0(J)?{test:J.test,branches:J.branches}:g0(J),Q=null,j=-2,M=null,B=(H)=>{for(let T of H)try{T.parentNode?.removeChild(T)}catch{}},P=d(()=>{let H=Z();if(Q&&H===j)return;if(M&&H===M.idx&&H>=0&&H<z.length){let K=M;M=null;for(let b of K.nodes)X0(b);Q={nodes:K.nodes,...K.cleanup&&{cleanup:K.cleanup}},j=H,RJ(Q.nodes);return}if(Q){let K={idx:j,nodes:Q.nodes,...Q.cleanup&&{cleanup:Q.cleanup}},b=bJ(K.nodes);if(Q=null,b===null)B(K.nodes),K.cleanup?.();else M=K,b.then(()=>{if(M!==K)return;B(K.nodes),K.cleanup?.(),M=null})}if(j=H,H<0||H>=z.length)return;let T=TJ(X,"render",()=>z[H]());if(!T)return;Q=T;let $=X.parentNode;if(!$)return;for(let K of Q.nodes)$.insertBefore(K,X);RJ(Q.nodes)});return()=>{if(P(),Q)B(Q.nodes),Q.cleanup?.(),Q=null;if(M)B(M.nodes),M.cleanup?.(),M=null}}function m0(J){return typeof J.test==="function"}function g0(J){let{cond:X,then_:Z,else_:z}=J;return{test:()=>X()?0:z?1:-1,branches:z?[Z,z]:[Z]}}var Z0=(J)=>J??"",Q0=(J)=>{if(J===null||J==="")return NaN;return Number(J)},z0=(J)=>{if(J===null)return!1;if(J.toLowerCase()==="false")return!1;return!0};function lJ(J){return(X)=>{if(X===null||X==="")return J;try{return JSON.parse(X)}catch{return J}}}function W0(J,X){let Z=new Set(J);return(z)=>z!==null&&Z.has(z)?z:X}var dJ=(J)=>{switch(J.type){case"string":return Z0;case"bool":return z0;case"number":return Q0;case"union":{let X=J.values??[],Z=J.default??X[0]??"";return W0(X,Z)}case"object":return lJ(J.default);case"array":return lJ(J.default??[])}};function I0(J,X,Z,z){if(!X)return;let Q={},j,M=Z.formValue;if(M!==void 0){let{sig:P,propDef:H}=O0(Z,z,M,"formValue"),T=H.default,$=dJ(H);j=d(()=>{let K=P();X.setFormValue(u0(K))}),Q.reset=()=>{P.set(T)},Q.restore=(K)=>{P.set($(K))}}let B=Z.formDisabled;if(B!==void 0){let{sig:P,propDef:H}=O0(Z,z,B,"formDisabled");if(H.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${B}" must be a bool-typed prop`);Q.disabled=(T)=>{P.set(!!T)}}if(J.__shipSetFormHooks=Q,j)J.__shipSetFormValueCleanup=j;return j}function N0(J){let X=J;delete X.__shipSetFormHooks,delete X.__shipSetFormValueCleanup}function u0(J){if(J===null||J===void 0)return"";if(typeof J==="boolean")return J?"on":"";return String(J)}function O0(J,X,Z,z){let Q=J.props[Z];if(!Q)throw Error(`[sui-runtime] ${z}: "${Z}" — no matching prop on this component`);let j=X[AJ(Z)];if(!j)throw Error(`[sui-runtime] ${z}: "${Z}" — prop signal missing`);return{sig:j,propDef:Q}}var $J={string(J={}){return{type:"string",...J}},bool(J={}){return{type:"bool",...J}},number(J={}){return{type:"number",...J}},union(J,X={}){return{type:"union",values:J,...X}},object(J={}){return{type:"object",...J}},array(J={}){return{type:"array",...J}}},h0=(J,X)=>{if(X===void 0||X===null)return null;switch(J.type){case"string":return X===""?null:String(X);case"number":return Number.isNaN(X)?null:String(X);case"bool":return X?"":null;case"union":return String(X);case"object":case"array":try{return JSON.stringify(X)}catch{return null}}},l0=(J,X)=>{let Z=`ship-set-style-${J}`;if(document.getElementById(Z))return;let z=document.createElement("style");z.id=Z,z.textContent=`[data-component="${J}"] { ${X} }`,document.head.appendChild(z)},d0=(J,X,Z,z)=>{let Q=J.getAttribute(X),j=Q!==null?z(Q):Z.default;return k(j)},p0=(J)=>{let X=[],Z={};for(let[j,M]of Object.entries(J.props??{}))X.push(j),Z[j]={camel:AJ(j),decode:dJ(M)};let z=J.formAssociated===!0||J.formValue!==void 0||J.formDisabled!==void 0,Q={observedAttrs:X,props:Z};if(z)Q.formAssociated=!0;return Q};function KJ(J,X){let Z=p0(X),z=Object.entries(X.props??{});Y0(J,Z,(Q,j,M)=>{let B=null;if(X.shadow){if(B=Q.shadowRoot??Q.attachShadow({mode:"open"}),X.styles){let H=document.createElement("style");H.textContent=X.styles,B.appendChild(H)}}else if(X.styles)l0(J,X.styles);let P=[];for(let[H,T]of z){let $=AJ(H),K=Z.props[H].decode,b=d0(Q,H,T,K);if(j[$]=b,T.reflect){let o=!0,i=d(()=>{let n=b();if(o){o=!1;return}let XJ=h0(T,n);if(XJ===null)Q.removeAttribute(H);else Q.setAttribute(H,XJ)});P.push(i)}}if(X.formValue!==void 0||X.formDisabled!==void 0){let H=I0(Q,M,X,j);if(typeof H==="function")P.push(H)}if(X.setup){let H={props:j,host:Q,cleanups:P};if(M)H.internals=M;X.setup(H)}return()=>{N0(Q);for(let H of P.splice(0))try{H()}catch(T){console.error(`[ship-set:${J}] cleanup threw:`,T)}}})}var C0=new Set;function Y0(J,X,Z){if(customElements.get(J)){if(!C0.has(J))C0.add(J),console.warn(`[sui-runtime] customElements.define("${J}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(X.formAssociated){class Q extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return X.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=J,uJ(this),this.scope=SJ();try{this.cleanup=xJ(this.scope,()=>sJ(this,()=>Z(this,this.propSignals,this.internals)))}catch(j){let M=OJ(this.parentNode);if(M)M.catch(j,"mount");else console.error(`[ship-set:${J}] setup threw:`,j)}}disconnectedCallback(){try{this.cleanup?.()}catch(j){console.error(`[ship-set:${J}] cleanup threw:`,j)}this.cleanup=void 0;try{this.scope?.dispose()}catch(j){console.error(`[ship-set:${J}] scope dispose threw:`,j)}this.scope=void 0}attributeChangedCallback(j,M,B){let P=X.props[j];if(!P)return;let H=this.propSignals[P.camel];if(H)H.set(P.decode(B))}formResetCallback(){let j=this.__shipSetFormHooks;try{j?.reset?.()}catch(M){console.error(`[ship-set:${J}] formResetCallback threw:`,M)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(j,M){let B=this.__shipSetFormHooks;try{B?.restore?.(j)}catch(P){console.error(`[ship-set:${J}] formStateRestoreCallback threw:`,P)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:j,mode:M}}))}formDisabledCallback(j){let M=this.__shipSetFormHooks;try{M?.disabled?.(j)}catch(B){console.error(`[ship-set:${J}] formDisabledCallback threw:`,B)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:j}}))}}customElements.define(J,Q);return}class z extends HTMLElement{static get observedAttributes(){return X.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=J,uJ(this),this.scope=SJ();try{this.cleanup=xJ(this.scope,()=>sJ(this,()=>Z(this,this.propSignals)))}catch(Q){let j=OJ(this.parentNode);if(j)j.catch(Q,"mount");else console.error(`[ship-set:${J}] setup threw:`,Q)}}disconnectedCallback(){try{this.cleanup?.()}catch(Q){console.error(`[ship-set:${J}] cleanup threw:`,Q)}this.cleanup=void 0;try{this.scope?.dispose()}catch(Q){console.error(`[ship-set:${J}] scope dispose threw:`,Q)}this.scope=void 0}attributeChangedCallback(Q,j,M){let B=X.props[Q];if(!B)return;let P=this.propSignals[B.camel];if(P)P.set(B.decode(M))}}customElements.define(J,z)}function AJ(J){return J.replace(/-([a-z])/g,(X,Z)=>Z.toUpperCase())}function j0(J,X,Z){let z=c0(J.state,{ship:X.ship,...Z}),Q={};for(let[K,b]of Object.entries(z))Q[K]=k(b);let j={ship:X.ship,route:X.route??{},navigate:X.navigate??((K)=>{}),...Z,...Q},M=a0(Q,j),B={...j},P={};for(let[K,b]of Object.entries(J.derived??{}))P[K]=JJ(()=>b(B));Object.assign(B,P);let H={};for(let[K,b]of Object.entries(J.on??{}))H[K]=(...o)=>b(M,...o);let T={},$=typeof J.use==="function"?J.use(B):J.use??{};for(let[K,b]of Object.entries($))T[K]=j0(b,X,B);return{...Q,...P,...H,...T}}function c0(J,X){if(J===void 0)return{};if(typeof J==="function")return J(X);return J}function a0(J,X){return new Proxy({},{get(Z,z){if(z in J)return J[z]();return X[z]},set(Z,z,Q){if(z in J)return J[z].set(Q),!0;return console.warn(`[ship-set] hook handler tried to write s.${z} = … but ${z} is not a hook state key. The assignment was ignored.`),!0},has(Z,z){return z in J||z in X},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(Z,z){if(z in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[z]()};return}})}var T0="__shipSetLifecycle";function E0(J,X){let Z=r0(J,X),z={ship:X.ship,route:X.route??{},navigate:X.navigate??((K)=>{}),...X.scope??{}},Q=Z.scope?Z.scope(z,X.host):{};Object.assign(z,Q);let j=o0(z,X.stateKeys,Q),M=s0(j,z),B={...z},P=t0(Z,B);Object.assign(B,P);let H=e0(Z,B,X);for(let[K,b]of Object.entries(H))B[K]=b;let T=_0(Z,M),$={...Q,...P,...T,...H};if(Object.keys(Q).length>0&&X.host){let K=_J(Q);if(Object.keys(K).length>0){let b=X.host,o=b.tagName.toLowerCase();tJ(b,o,K)}}if(Z.mount||Z.unmount){let K={state:M};if(Z.mount)K.mount=Z.mount;if(Z.unmount)K.unmount=Z.unmount;Object.defineProperty($,T0,{value:K,enumerable:!0,configurable:!0,writable:!1})}return $}function r0(J,X){if(typeof J==="function")return J({ship:X.ship});return J}function o0(J,X,Z){let z={},Q=X!==void 0,j=Z?Object.keys(Z):[];if(Q||j.length>0){if(X)for(let M of X){let B=J[M];if(G0(B))z[M]=B}for(let M of j){if(M in z)continue;let B=J[M];if(G0(B))z[M]=B}return z}for(let[M,B]of Object.entries(J))if(G0(B))z[M]=B;return z}function G0(J){return typeof J==="function"&&"set"in J&&typeof J.set==="function"}function s0(J,X){return new Proxy({},{get(Z,z){if(z in J)return J[z]();return X[z]},set(Z,z,Q){if(z in J){if(typeof Q==="function")J[z].set(()=>Q);else J[z].set(Q);return!0}return console.warn(`[ship-set] handler tried to write s.${z} = … but ${z} is not declared as <state name="${z}"> in the template. The assignment was ignored.`),!0},has(Z,z){return z in J||z in X},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(Z,z){if(z in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[z]()};return}})}function t0(J,X){let Z=J.derived??{},z={};for(let[Q,j]of Object.entries(Z))z[Q]=JJ(()=>j(X));return z}function _0(J,X){let Z=J.on??{},z={};for(let[Q,j]of Object.entries(Z))z[Q]=(...M)=>j(X,...M);return z}function e0(J,X,Z){if(J.use===void 0)return{};let z=typeof J.use==="function"?J.use(X):J.use,Q={};for(let[j,M]of Object.entries(z))Q[j]=j0(M,Z,X);return Q}function IJ(J){return{setup:(X)=>E0(J,X),__def:J}}if(typeof customElements<"u"){for(let J of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(J))customElements.define(J,class extends HTMLElement{})}var QX=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,zX=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function R0(J){if(typeof CSSStyleSheet>"u")return null;try{let X=new CSSStyleSheet;return X.replaceSync(J),X}catch{return null}}var A0=R0(QX),w0=R0(zX);function NJ(J){if(typeof ShadowRoot>"u")return;if(!(J instanceof ShadowRoot))return;let X=[w0,A0].filter((Z)=>Z!=null);if(X.length===0)return;try{J.adoptedStyleSheets=[...X,...J.adoptedStyleSheets??[]]}catch{}}var pJ=[{id:"view",label:"View",icon:"file",shapes:["list","form","detail","dashboard","tabs","blank"],shapeLabel:"Starting shape"},{id:"skill",label:"Skill",icon:"sparkles",shapes:["task","reference","workflow"],shapeLabel:"Skill kind"},{id:"agent",label:"Agent",icon:"users",shapes:["specialist","generalist","reviewer"],shapeLabel:"Agent kind"},{id:"pipeline",label:"Pipeline",icon:"play",shapes:["sequential","fanout","review-gated"],shapeLabel:"Topology"},{id:"trigger",label:"Trigger",icon:"zap",shapes:["webhook","schedule","file-change"],shapeLabel:"Kind"},{id:"theme",label:"Theme",icon:"palette",shapes:["light","dark","brand-accent","high-contrast"],shapeLabel:"Starting shape"}],iJ=[{label:"Dashboard",path:"/dashboard",icon:"sparkles",keywords:"home overview"},{label:"Pipelines",path:"/pipelines",icon:"play",keywords:"workflows runs jobs"},{label:"Agents",path:"/agents",icon:"users",keywords:"configure profiles"},{label:"Skills",path:"/skills",icon:"zap",keywords:"browse edit files"},{label:"Git",path:"/git",icon:"git",keywords:"status diff commit branch"},{label:"Collections",path:"/collections",icon:"folder",keywords:"data items query"},{label:"Canvas",path:"/canvas",icon:"image",keywords:"docs visual feedback"},{label:"Events",path:"/events",icon:"sparkles",keywords:"stream log kernel"},{label:"Settings",path:"/settings",icon:"settings",keywords:"config theme provider"},{label:"Chat",path:"/chat",icon:"message-square-plus",keywords:"agent conversation"},{label:"Specs",path:"/specs",icon:"file",keywords:"features design"},{label:"Theme Studio",path:"/theme-studio",icon:"palette",keywords:"design tokens"},{label:"Chats (Mission Control)",path:"/chats",icon:"messages-square",keywords:"sessions active"}],nJ=[{key:"new",label:"/new",hint:"Scaffold a view, skill, agent, …"},{key:"chats",label:"/chats",hint:"Open Mission Control"},{key:"settings",label:"/settings",hint:"Jump to settings"},{key:"lint",label:"/lint",hint:"Run lint on the current view"},{key:"end",label:"/end",hint:"End the current session"},{key:"close",label:"/close",hint:"Close the command bar"}],WX=/^[a-z][a-z0-9-]{0,48}$/;function x0(J){return WX.test(J)}function fJ(J,X){if(J.startsWith("/"))return"command";if(J.startsWith(">"))return"find";if(J.startsWith("+"))return"create";if(J.trim().length===0)return X;if(/(^|\s)@[^\s@]*$/.test(J))return"mention";return"ai"}async function qJ(J){let X=window;if(X.ship?.navigate){X.ship.navigate(J);return}history.pushState({},"",J),document.dispatchEvent(new CustomEvent("sui:navigate",{detail:{to:J}}))}var cJ=IJ({scope:(J)=>{let X=J,Z=r(X.open),z=r(X.mode),Q=r(k("")),j=r(k(null)),M=r(k([])),B=r(k([])),P=r(k([])),H=r(k([])),T=r(k(0)),$=r(k(!1)),K=r(k("")),b=r(k("")),o=r(k(!1)),i=r(k(null)),n=r(k([])),XJ=r(k("main")),a=r(k("view")),ZJ=r(k("")),_=r(k("")),e=r(k("list")),C=r(k(!1)),m=r(k(null)),I=r(JJ(()=>z()==="ai"&&j()!==null)),f=eJ(null);return{open:Z,mode:z,value:Q,session:j,agents:M,providers:B,allSessions:P,views:H,selectedIdx:T,sessionPickerOpen:$,spawnAgent:K,spawnProvider:b,spawning:o,spawnError:i,thread:n,workspaceId:XJ,createSurface:a,createName:ZJ,createPurpose:_,createShape:e,createSubmitting:C,createError:m,isChat:I,dialogRef:f}},derived:{triggerLabel:(J)=>{let X=J.session();return X?`Ask ${X.agent_id??"Ship"} or jump to…`:"Ask Ship or jump to…"},dotBg:(J)=>{let X=J.session();if(X?.status==="thinking")return"var(--warning,#d97706)";if(X)return"var(--success,#16a34a)";return"var(--muted-foreground)"}},on:{openFind:(J)=>{J.mode="find",J.value="",J.selectedIdx=0,J.open=!0}},mount:(J,X)=>{let Z=J,z=Z.open,Q=Z.mode,j=Z.value,M=Z.session,B=Z.agents,P=Z.providers,H=Z.allSessions,T=Z.views,$=Z.selectedIdx,K=Z.spawnAgent,b=Z.spawnProvider,o=Z.spawning,i=Z.spawnError,n=Z.thread,XJ=Z.workspaceId,a=Z.createSurface,ZJ=Z.createName,_=Z.createPurpose,e=Z.createShape,C=Z.createSubmitting,m=Z.createError,I=Z.dialogRef?.(),f=null,S=0,QJ=d(()=>{let F=z();if(X.setAttribute("data-state",F?"open":"closed"),F){if(typeof I?.show==="function")I.show();queueMicrotask(()=>{X.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelector(".cb-search")?.focus()})}else{if(typeof I?.close==="function")I.close();X.dispatchEvent(new CustomEvent("sui-command:cancel",{detail:{mode:Q()},bubbles:!0,composed:!0}))}});X.show=(F,U)=>{if(F)Q.set(F);if(typeof U==="string")j.set(U);z.set(!0)},X.hide=()=>z.set(!1),X.toggle=(F)=>{if(z())z.set(!1);else{if(F)Q.set(F);z.set(!0)}},X.setMode=(F)=>Q.set(F);function YJ(F){FJ();let U=new AbortController;f=U;let N=new EventSource(`/api/v1/sessions/${F}/events`,{withCredentials:!0}),R=null;N.addEventListener("provider",(zJ)=>{let c;try{c=JSON.parse(zJ.data)}catch{return}switch(c.kind){case"message":{let LJ=c.text??"";if(!LJ)return;if(R)n.set(n().map((l)=>l.id===R?{...l,text:l.text+LJ}:l));else{let l=String(Date.now()+Math.random());R=l,n.set([...n(),{id:l,sender:"agent",text:LJ}])}break}case"tool_use":n.set([...n(),{id:String(Date.now()+Math.random()),sender:"system",text:`→ tool: ${c.tool??"?"}`}]);break;case"turn_complete":R=null;break}}),N.onerror=()=>{if(!U.signal.aborted)N.close()},U.signal.addEventListener("abort",()=>N.close())}function FJ(){f?.abort(),f=null}let PJ=d(()=>{let F=M();if(F?.id)YJ(F.id);else FJ()});async function jJ(){try{let F=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(F.ok){let U=await F.json();H.set(U.sessions);let N=U.sessions.find((R)=>R.status!=="ended"&&R.agent_id);if(N&&N.id!==M()?.id)M.set(N),n.set([]);else if(!N)M.set(null)}}catch{}if(B().length===0)GJ();if(P().length===0)UJ();if(T().length===0)u()}async function GJ(){try{let F=await fetch("/api/v1/agents",{credentials:"same-origin"});if(!F.ok)return;let N=((await F.json()).items??[]).map((R)=>({id:R.id}));if(B.set(N),!K()&&N.length>0)K.set(N[0].id)}catch{}}async function UJ(){try{let F=await fetch("/api/v1/providers/installed",{credentials:"same-origin"});if(!F.ok)return;let U=await F.json();if(P.set(U.providers),!b()&&U.providers.length>0)b.set(U.providers[0])}catch{}}async function u(){try{let F=await fetch("/api/v1/views",{credentials:"same-origin"});if(!F.ok)return;let U=await F.json(),N=U.data?.views??U.views??[];T.set(N.map((R)=>R.name).sort())}catch{}}function p(){return X.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelectorAll(".cb-item")}function g(F){let U=p();if(!U?.length)return;$.set(Math.max(0,Math.min(U.length-1,$()+F)))}function w(){let F=Q(),U=j(),R=p()?.[$()];if(R){let zJ=R.dataset.path;if(zJ)qJ(zJ),j.set(""),z.set(!1);let c=R.dataset.cmd;if(c)j.set(""),y("/"+c);let LJ=R.dataset.mentionKind,l=R.dataset.mentionId;if(LJ&&l){let MJ=LJ==="view"?`@${l}`:`@${LJ}:${l}`;j.set(U.replace(/@[^@\s]*$/,`${MJ} `)),Q.set(fJ(j(),F))}X.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:F,item:R.dataset},bubbles:!0,composed:!0}));return}if(F==="command")y(U);else if(F==="find")O(U)}function y(F){let U=F.replace(/^\/+\s*/,"").toLowerCase(),N=nJ.find((R)=>R.key===U||R.key.startsWith(U));if(!N)return;switch(N.key){case"new":Q.set("create"),j.set("");break;case"chats":qJ("/chats"),z.set(!1);break;case"settings":qJ("/settings"),z.set(!1);break;case"end":if(M())G(M().id);z.set(!1);break;case"close":z.set(!1);break;case"lint":L();break}}function O(F){let U=F.replace(/^>+\s*/,"").toLowerCase().trim(),N=iJ.find((R)=>R.label.toLowerCase().includes(U)||R.keywords.toLowerCase().includes(U)||R.path.toLowerCase().includes(U));if(N)qJ(N.path),j.set(""),z.set(!1)}async function G(F){try{await fetch(`/api/v1/sessions/${F}`,{method:"DELETE",credentials:"same-origin"})}catch{}if(M()?.id===F)M.set(null);await jJ()}async function L(){let U=window.ship?.current?.()?.path?.replace(/^\/+/,"").split("/")[0]??"";if(!U)return;try{let N=await fetch("/api/v1/lint/view",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:U})});if(N.ok){let zJ=(await N.json()).data?.findings?.length??0;console.log(zJ===0?`${U}: no lint findings`:`${U}: ${zJ} lint findings`)}}catch{}z.set(!1)}let W=(F)=>{if(F.key==="Escape"&&z()){z.set(!1);return}if(F.key===" "&&!z()){let U=F.target;if(U&&(U.tagName==="INPUT"||U.tagName==="TEXTAREA"||U.isContentEditable))return;let N=performance.now();if(N-S<350)S=0,F.preventDefault(),Q.set("ai"),j.set(""),z.set(!0);else S=N}},Y=(F)=>{if(F.key==="ArrowDown")F.preventDefault(),g(1);else if(F.key==="ArrowUp")F.preventDefault(),g(-1);else if(F.key==="Enter")F.preventDefault(),w()};X.addEventListener("keydown",Y),document.addEventListener("keydown",W);let D=()=>{if(z())z.set(!1)};I?.addEventListener("sui-close",D);let q=()=>void jJ(),V=(F)=>{let U=F;if(U.detail?.mode)Q.set(U.detail.mode);if(typeof U.detail?.prefill==="string")j.set(U.detail.prefill);z.set(!0)},E=()=>{Q.set("create"),z.set(!0)};window.addEventListener("ship:session:changed",q);let x=document.querySelector("ship-shell")??document.querySelector("sui-shell");x?.addEventListener("ship:bar:open",V),x?.addEventListener("ship:create:open",E);let A=X["__sui_ctx_sui-command-bar"];if(A)A.endSession=G,A.loadCurrentSession=jJ;return jJ(),()=>{if(document.removeEventListener("keydown",W),X.removeEventListener("keydown",Y),window.removeEventListener("ship:session:changed",q),x?.removeEventListener("ship:bar:open",V),x?.removeEventListener("ship:create:open",E),I)I.removeEventListener("sui-close",D);PJ(),QJ()}}});var jz=KJ("sui-command-bar",{props:{open:$J.bool({default:!1,reflect:!0}),mode:$J.string({default:"ai",reflect:!0})},shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null);Q.open=X.open,Q.mode=X.mode;let j=null;if(cJ&&typeof cJ.setup==="function"){if(J.scope=Q,j=cJ.setup(J),j&&typeof j==="object")Object.assign(Q,j)}let M=(a)=>WJ({anchor:a,test:()=>{if(Q.isChat())return 0;return 1},branches:[()=>{let ZJ=document.createDocumentFragment(),_=[],e=document.createElement("sui-command-chat");return ZJ.appendChild(e),{nodes:Array.from(ZJ.childNodes),cleanup:()=>{for(let C of _)try{C()}catch(m){}}}},()=>{let ZJ=document.createDocumentFragment(),_=[],e=document.createElement("sui-command-palette");return ZJ.appendChild(e),{nodes:Array.from(ZJ.childNodes),cleanup:()=>{for(let C of _)try{C()}catch(m){}}}}]}),B=Z.shadowRoot??Z;NJ(B);let P=document.createElement("button");P.setAttribute("class","cb-trigger"),P.setAttribute("type","button"),P.setAttribute("aria-label","Open command bar"),z.push(t(P,"click",function(){let a=Q.openFind;return typeof a==="function"?a.bind(void 0):()=>{}}(),{modifiers:[]}));let H=document.createElement("sui-icon");H.setAttribute("name","search"),H.setAttribute("size","sm"),P.appendChild(H);let T=document.createElement("span");T.setAttribute("class","cb-label");let $=document.createTextNode("");T.appendChild($),z.push(h($,()=>Q.triggerLabel())),P.appendChild(T);let K=document.createElement("span");K.setAttribute("class","cb-dot"),z.push(d(()=>{let a=Q.dotBg();if(a==null)K.style.removeProperty("background");else K.style.setProperty("background",String(a))})),P.appendChild(K);let b=document.createElement("kbd"),o=document.createTextNode("⌘K");b.appendChild(o),P.appendChild(b),B.appendChild(P);let i=document.createElement("sui-dialog");i.setAttribute("class","cb-dialog"),i.setAttribute("no-header",""),i.setAttribute("no-footer",""),i.setAttribute("no-close-button",""),i.setAttribute("backdrop","blur"),i.setAttribute("size","full"),i.setAttribute("style","--dlg-max-w: min(840px, 94vw)"),J0(Q,"dialogRef",i);let n=document.createElement("div");n.setAttribute("class","cb-wrap");let XJ=document.createComment("if");if(n.appendChild(XJ),z.push(M(XJ)),i.appendChild(n),B.appendChild(i),j!=null){let a=Object.getOwnPropertyDescriptor(j,"__shipSetLifecycle")?.value;if(a){let ZJ=new Proxy(Q,{get(_,e){return _[e]},set(_,e,C){let m=_[e];if(m!=null&&typeof m.set==="function"){if(typeof C==="function")m.set(()=>C);else m.set(C);return!0}return!1}});if(a.mount){let _=a.mount(ZJ,Z);if(typeof _==="function")z.push(_)}if(a.unmount)z.push(()=>a.unmount(ZJ))}}return Q}});var aJ=IJ({scope:(J,X)=>{let Z=yJ(X,"sui-command-bar"),z=JJ(()=>{if(Z.session()?.status==="thinking")return"var(--warning,#d97706)";return"var(--success,#16a34a)"}),Q=JJ(()=>Z.allSessions().filter((H)=>H.status!=="ended"&&H.agent_id));function j(H){if(fetch(`/api/v1/sessions/${H}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}),Z.session()?.id===H)Z.session.set(null);Z.sessionPickerOpen.set(!1)}function M(H){let T=Z.allSessions().find(($)=>$.id===H);if(T)Z.session.set(T),Z.thread.set([]);Z.sessionPickerOpen.set(!1)}function B(){Z.sessionPickerOpen.set(!1),Z.mode.set("ai"),Z.session.set(null)}function P(){Z.sessionPickerOpen.set(!1),qJ("/chats"),Z.open.set(!1)}return{bar:Z,headerDotBg:z,activeSessions:Q,endSession:j,selectSession:M,startNewSession:B,viewChats:P}},on:{toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},sendMessage:async(J)=>{let X=J.bar.value().trim();if(!X||!J.bar.session())return;let Z=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:X}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${Z}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:X})})}catch{}},onComposerKeydown:async(J,X)=>{if(X.key==="Enter"&&!X.shiftKey){X.preventDefault();let Z=J.bar.value().trim();if(!Z||!J.bar.session())return;let z=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:Z}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${z}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:Z})})}catch{}}}}});var Dz=KJ("sui-command-chat",{shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null),j=null;if(aJ&&typeof aJ.setup==="function"){if(J.scope=Q,j=aJ.setup(J),j&&typeof j==="object")Object.assign(Q,j)}let M=(u)=>VJ({anchor:u,items:()=>Q.activeSessions(),key:(p,g)=>{let w=Object.create(Q);return w.s=p,w.__i=g,((y)=>p.id)(w)},render:(p,g)=>{let w=Object.create(Q);return w.s=p,w.__i=g,w.__v=k(0),((y)=>{let O=document.createDocumentFragment(),G=[],L=document.createElement("div");L.setAttribute("class","cb-session-row"),G.push(EJ(L,"current",()=>(y.__v(),y.bar.session()?.id===y.s.id))),L.setAttribute("role","option"),G.push(t(L,"click",function(){let N=()=>y.selectSession(y.s.id);return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]}));let W=document.createElement("span");W.setAttribute("class","cb-dot"),G.push(d(()=>{y.__v();let N=y.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(N==null)W.style.removeProperty("background");else W.style.setProperty("background",String(N))})),L.appendChild(W);let Y=document.createElement("span"),D=document.createElement("strong"),q=document.createTextNode("");D.appendChild(q),G.push(h(q,()=>(y.__v(),y.s.agent_id??""))),Y.appendChild(D);let V=document.createElement("span");V.setAttribute("class","meta");let E=document.createTextNode(" · ");V.appendChild(E);let x=document.createTextNode("");V.appendChild(x),G.push(h(x,()=>(y.__v(),y.s.provider??"")));let v=document.createTextNode(" · ");V.appendChild(v);let A=document.createTextNode("");V.appendChild(A),G.push(h(A,()=>(y.__v(),y.s.workspace_id))),Y.appendChild(V),L.appendChild(Y);let F=document.createElement("button");F.setAttribute("class","cb-session-end"),F.setAttribute("type","button"),F.setAttribute("aria-label","End session"),G.push(t(F,"click",function(){let N=(R)=>{R.stopPropagation(),y.endSession(y.s.id)};return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]}));let U=document.createTextNode("×");return F.appendChild(U),L.appendChild(F),O.appendChild(L),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let N of G)try{N()}catch(R){}},update:(N,R)=>{y.s=N,y.__i=R,y.__v.set((zJ)=>zJ+1)}}})(w)}}),B=(u)=>WJ({anchor:u,test:()=>{if(Q.activeSessions().length===0)return 0;return-1},branches:[()=>{let p=document.createDocumentFragment(),g=[],w=document.createElement("div");w.setAttribute("class","cb-session-row");let y=document.createElement("span");w.appendChild(y);let O=document.createElement("span");O.setAttribute("class","meta");let G=document.createTextNode("No active sessions");O.appendChild(G),w.appendChild(O);let L=document.createElement("span");return w.appendChild(L),p.appendChild(w),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let W of g)try{W()}catch(Y){}}}}]}),P=(u)=>WJ({anchor:u,test:()=>{if(Q.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let p=document.createDocumentFragment(),g=[],w=document.createElement("div");w.setAttribute("class","cb-session-popover"),w.setAttribute("role","listbox"),w.setAttribute("aria-label","Sessions");let y=document.createComment("for");w.appendChild(y),g.push(M(y));let O=document.createComment("if");w.appendChild(O),g.push(B(O));let G=document.createElement("div");G.setAttribute("class","cb-session-sep"),G.setAttribute("role","separator"),w.appendChild(G);let L=document.createElement("div");L.setAttribute("class","cb-session-action"),g.push(t(L,"click",function(){let v=Q.startNewSession;return typeof v==="function"?v.bind(void 0):()=>{}}(),{modifiers:[]}));let W=document.createElement("sui-icon");W.setAttribute("name","plus"),W.setAttribute("size","sm"),W.setAttribute("aria-hidden","true"),L.appendChild(W);let Y=document.createElement("span"),D=document.createTextNode("Start new session");Y.appendChild(D),L.appendChild(Y),w.appendChild(L);let q=document.createElement("div");q.setAttribute("class","cb-session-action"),g.push(t(q,"click",function(){let v=Q.viewChats;return typeof v==="function"?v.bind(void 0):()=>{}}(),{modifiers:[]}));let V=document.createElement("sui-icon");V.setAttribute("name","columns"),V.setAttribute("size","sm"),V.setAttribute("aria-hidden","true"),q.appendChild(V);let E=document.createElement("span"),x=document.createTextNode("View all chats");return E.appendChild(x),q.appendChild(E),w.appendChild(q),p.appendChild(w),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let v of g)try{v()}catch(A){}}}}]}),H=(u)=>VJ({anchor:u,items:()=>Q.bar.thread(),key:(p,g)=>{let w=Object.create(Q);return w.msg=p,w.__i=g,((y)=>p.id)(w)},render:(p,g)=>{let w=Object.create(Q);return w.msg=p,w.__i=g,w.__v=k(0),((y)=>{let O=document.createDocumentFragment(),G=[],L=document.createElement("div");L.setAttribute("class","cb-chat-turn"),G.push(s(L,"data-sender",()=>(y.__v(),y.msg.sender)));let W=document.createElement("div");W.setAttribute("class","cb-chat-bubble");let Y=document.createTextNode("");return W.appendChild(Y),G.push(h(Y,()=>(y.__v(),y.msg.text))),L.appendChild(W),O.appendChild(L),{nodes:Array.from(O.childNodes),cleanup:()=>{for(let D of G)try{D()}catch(q){}},update:(D,q)=>{y.msg=D,y.__i=q,y.__v.set((V)=>V+1)}}})(w)}}),T=(u)=>WJ({anchor:u,test:()=>{if(Q.bar.thread().length===0)return 0;return 1},branches:[()=>{let p=document.createDocumentFragment(),g=[],w=document.createElement("div");w.setAttribute("class","cb-chat-empty");let y=document.createTextNode("Type below and press Enter. The agent's replies land here.");return w.appendChild(y),p.appendChild(w),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let O of g)try{O()}catch(G){}}}},()=>{let p=document.createDocumentFragment(),g=[],w=document.createComment("for");return p.appendChild(w),g.push(H(w)),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let y of g)try{y()}catch(O){}}}}]}),$=Z.shadowRoot??Z;NJ($);let K=document.createElement("div");K.setAttribute("class","cb-chat-header");let b=document.createElement("span");b.setAttribute("class","cb-dot"),z.push(d(()=>{let u=Q.headerDotBg();if(u==null)b.style.removeProperty("background");else b.style.setProperty("background",String(u))})),b.setAttribute("aria-hidden","true"),K.appendChild(b);let o=document.createElement("div");o.setAttribute("style","display:grid;line-height:1.25");let i=document.createElement("span");i.setAttribute("class","title");let n=document.createTextNode("");i.appendChild(n),z.push(h(n,()=>Q.bar.session()?.agent_id??"")),o.appendChild(i);let XJ=document.createElement("span");XJ.setAttribute("class","sub");let a=document.createTextNode("");XJ.appendChild(a),z.push(h(a,()=>Q.bar.session()?.provider??"—"));let ZJ=document.createTextNode(" · ");XJ.appendChild(ZJ);let _=document.createTextNode("");XJ.appendChild(_),z.push(h(_,()=>Q.bar.session()?.workspace_id??"")),o.appendChild(XJ),K.appendChild(o);let e=document.createElement("span");e.setAttribute("style","flex:1"),K.appendChild(e);let C=document.createElement("div");C.setAttribute("class","cb-chip-wrap");let m=document.createElement("span");m.setAttribute("class","cb-session-chip"),m.setAttribute("role","button"),m.setAttribute("aria-haspopup","listbox"),z.push(t(m,"click",function(){let u=Q.toggleSessionPicker;return typeof u==="function"?u.bind(void 0):()=>{}}(),{modifiers:[]}));let I=document.createTextNode("Switch");m.appendChild(I),C.appendChild(m);let f=document.createComment("if");C.appendChild(f),z.push(P(f)),K.appendChild(C);let S=document.createElement("kbd");S.setAttribute("class","cb-esc"),S.setAttribute("aria-label","Escape to close");let QJ=document.createTextNode("esc");S.appendChild(QJ),K.appendChild(S),$.appendChild(K);let YJ=document.createElement("div");YJ.setAttribute("class","cb-chat-messages");let FJ=document.createComment("if");YJ.appendChild(FJ),z.push(T(FJ)),$.appendChild(YJ);let PJ=document.createElement("div");PJ.setAttribute("class","cb-chat-composer");let jJ=document.createElement("textarea");jJ.setAttribute("class","cb-chat-input"),jJ.setAttribute("rows","2"),jJ.setAttribute("placeholder","Message the agent…"),z.push(DJ(jJ,Q.bar.value)),z.push(t(jJ,"keydown",function(){let u=Q.onComposerKeydown;return typeof u==="function"?u.bind(void 0):()=>{}}(),{modifiers:[]})),PJ.appendChild(jJ);let GJ=document.createElement("button");GJ.setAttribute("class","cb-chat-send"),GJ.setAttribute("type","button"),z.push(s(GJ,"disabled",()=>!Q.bar.value().trim())),z.push(t(GJ,"click",function(){let u=Q.sendMessage;return typeof u==="function"?u.bind(void 0):()=>{}}(),{modifiers:[]}));let UJ=document.createTextNode("Send");if(GJ.appendChild(UJ),PJ.appendChild(GJ),$.appendChild(PJ),j!=null){let u=Object.getOwnPropertyDescriptor(j,"__shipSetLifecycle")?.value;if(u){let p=new Proxy(Q,{get(g,w){return g[w]},set(g,w,y){let O=g[w];if(O!=null&&typeof O.set==="function"){if(typeof y==="function")O.set(()=>y);else O.set(y);return!0}return!1}});if(u.mount){let g=u.mount(p,Z);if(typeof g==="function")z.push(g)}if(u.unmount)z.push(()=>u.unmount(p))}}return Q}});var rJ=IJ({scope:(J,X)=>{let Z=yJ(X,"sui-command-bar"),z=JJ(()=>{return{command:"/",find:">",mention:"@",create:"+",ai:"ai"}[Z.mode()]??">"}),Q=JJ(()=>{return{command:"Run a Ship action…",find:"Jump to a view…",mention:"Reference an agent, view, file…",create:"Create a view, skill, agent…",ai:"Ask Ship or type / for actions"}[Z.mode()]??"Search…"}),j=JJ(()=>Z.session()?.agent_id??"No session"),M=JJ(()=>{let I=Z.session();if(I?.status==="thinking")return"var(--warning,#d97706)";if(I)return"var(--success,#16a34a)";return"var(--muted-foreground)"}),B=JJ(()=>Z.allSessions().filter((I)=>I.status!=="ended"&&I.agent_id));function P(I){if(I==="view")return"app-window";if(I==="agent")return"bot";return"message-square-plus"}let H=JJ(()=>{let I=Z.value().replace(/^>+\s*/,"").toLowerCase().trim();return iJ.filter((f)=>!I||f.label.toLowerCase().includes(I)||f.keywords.toLowerCase().includes(I)||f.path.toLowerCase().includes(I)).map((f,S)=>({...f,idx:S}))}),T=JJ(()=>{let I=Z.value().replace(/^\/+\s*/,"").toLowerCase().trim();return nJ.filter((f)=>!I||f.key.includes(I)||f.label.toLowerCase().includes(I)||f.hint.toLowerCase().includes(I)).map((f,S)=>({...f,idx:S}))}),$=JJ(()=>{let I=Z.value().replace(/^@+/,"").toLowerCase().trim();return[...Z.views().map((S)=>({kind:"view",id:S,label:S})),...Z.agents().map((S)=>({kind:"agent",id:S.id,label:S.id})),...Z.allSessions().filter((S)=>S.status!=="ended"&&S.agent_id).map((S)=>({kind:"session",id:S.id,label:`${S.agent_id} (session)`}))].filter((S)=>!I||S.id.toLowerCase().includes(I)).slice(0,40).map((S,QJ)=>({...S,idx:QJ,icon:P(S.kind)}))}),K=JJ(()=>pJ),b=JJ(()=>pJ.find((I)=>I.id===Z.createSurface())),o=JJ(()=>x0(Z.createName()));function i(I){fetch(`/api/v1/sessions/${I}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}).then(()=>{if(Z.session()?.id===I)Z.session.set(null);m()}),Z.sessionPickerOpen.set(!1)}function n(I){let f=Z.allSessions().find((S)=>S.id===I);if(f)Z.session.set(f),Z.thread.set([]);Z.sessionPickerOpen.set(!1)}function XJ(){Z.sessionPickerOpen.set(!1),Z.mode.set("ai"),Z.session.set(null)}function a(){Z.sessionPickerOpen.set(!1),qJ("/chats"),Z.open.set(!1)}let ZJ=X;function _(I){qJ(I),Z.value.set(""),Z.open.set(!1),ZJ?.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:"find",item:{path:I}},bubbles:!0,composed:!0}))}function e(I){switch(Z.value.set(""),I){case"new":Z.mode.set("create");break;case"chats":qJ("/chats"),Z.open.set(!1);break;case"settings":qJ("/settings"),Z.open.set(!1);break;case"end":if(Z.session())i(Z.session().id);Z.open.set(!1);break;case"close":Z.open.set(!1);break}}function C(I){let f=I.kind==="view"?`@${I.id}`:`@${I.kind}:${I.id}`,S=Z.value();Z.value.set(S.replace(/@[^@\s]*$/,`${f} `)),Z.mode.set(fJ(Z.value(),Z.mode()))}async function m(){try{let I=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(I.ok){let f=await I.json();Z.allSessions.set(f.sessions);let S=f.sessions.find((QJ)=>QJ.status!=="ended"&&QJ.agent_id);if(S&&S.id!==Z.session()?.id)Z.session.set(S),Z.thread.set([]);else if(!S)Z.session.set(null)}}catch{}}return{bar:Z,modeGlyph:z,searchPlaceholder:Q,chipLabel:j,chipDotBg:M,activeSessions:B,findItems:H,cmdItems:T,mentionItems:$,createSurfaces:K,currentSurfaceDef:b,createNameValid:o,endSession:i,selectSession:n,startNewSession:XJ,viewChats:a,activateNavItem:_,activateCmd:e,activateMention:C}},on:{onInput:(J,X)=>{let Z=X.target.value;J.bar.value.set(Z),J.bar.mode.set(fJ(Z,J.bar.mode())),J.bar.selectedIdx.set(0)},toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},selectSurface:(J,X)=>{J.bar.createSurface.set(X);let Z=pJ.find((z)=>z.id===X);if(Z?.shapes[0])J.bar.createShape.set(Z.shapes[0])},spawnSession:async(J)=>{let X=J.bar.spawnAgent(),Z=J.bar.spawnProvider();if(!X||!Z)return;J.bar.spawning.set(!0),J.bar.spawnError.set(null);try{let z=await fetch("/api/v1/sessions",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({agent_id:X,provider:Z})});if(!z.ok){let Q=await z.json().catch(()=>({}));J.bar.spawnError.set(Q.error??`HTTP ${z.status}`)}else J.bar.mode.set("ai")}catch(z){J.bar.spawnError.set(z instanceof Error?z.message:"Unknown error")}finally{J.bar.spawning.set(!1)}},submitCreate:async(J)=>{if(!J.createNameValid()||J.bar.createSubmitting())return;J.bar.createSubmitting.set(!0),J.bar.createError.set(null);try{let X=await fetch("/api/v1/scaffold",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({surface:J.bar.createSurface(),name:J.bar.createName(),shape:J.bar.createShape(),purpose:J.bar.createPurpose()})});if(!X.ok){let Z=await X.json().catch(()=>({}));J.bar.createError.set(Z.error??`HTTP ${X.status}`)}else J.bar.createName.set(""),J.bar.createPurpose.set(""),J.bar.open.set(!1)}catch(X){J.bar.createError.set(X instanceof Error?X.message:"Unknown error")}finally{J.bar.createSubmitting.set(!1)}}}});var Iz=KJ("sui-command-palette",{shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null),j=null;if(rJ&&typeof rJ.setup==="function"){if(J.scope=Q,j=rJ.setup(J),j&&typeof j==="object")Object.assign(Q,j)}let M=(O)=>VJ({anchor:O,items:()=>Q.activeSessions(),key:(G,L)=>{let W=Object.create(Q);return W.s=G,W.__i=L,((Y)=>G.id)(W)},render:(G,L)=>{let W=Object.create(Q);return W.s=G,W.__i=L,W.__v=k(0),((Y)=>{let D=document.createDocumentFragment(),q=[],V=document.createElement("div");V.setAttribute("class","cb-session-row"),q.push(EJ(V,"current",()=>(Y.__v(),Y.bar.session()?.id===Y.s.id))),V.setAttribute("role","option"),q.push(t(V,"click",function(){let l=()=>Y.selectSession(Y.s.id);return typeof l==="function"?l.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("span");E.setAttribute("class","cb-dot"),q.push(d(()=>{Y.__v();let l=Y.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(l==null)E.style.removeProperty("background");else E.style.setProperty("background",String(l))})),V.appendChild(E);let x=document.createElement("span"),v=document.createElement("strong"),A=document.createTextNode("");v.appendChild(A),q.push(h(A,()=>(Y.__v(),Y.s.agent_id??""))),x.appendChild(v);let F=document.createElement("span");F.setAttribute("class","meta");let U=document.createTextNode(" · ");F.appendChild(U);let N=document.createTextNode("");F.appendChild(N),q.push(h(N,()=>(Y.__v(),Y.s.provider??"")));let R=document.createTextNode(" · ");F.appendChild(R);let zJ=document.createTextNode("");F.appendChild(zJ),q.push(h(zJ,()=>(Y.__v(),Y.s.workspace_id))),x.appendChild(F),V.appendChild(x);let c=document.createElement("button");c.setAttribute("class","cb-session-end"),c.setAttribute("type","button"),c.setAttribute("aria-label","End session"),q.push(t(c,"click",function(){let l=(MJ)=>{MJ.stopPropagation(),Y.endSession(Y.s.id)};return typeof l==="function"?l.bind(void 0):()=>{}}(),{modifiers:[]}));let LJ=document.createTextNode("×");return c.appendChild(LJ),V.appendChild(c),D.appendChild(V),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let l of q)try{l()}catch(MJ){}},update:(l,MJ)=>{Y.s=l,Y.__i=MJ,Y.__v.set((vJ)=>vJ+1)}}})(W)}}),B=(O)=>WJ({anchor:O,test:()=>{if(Q.activeSessions().length===0)return 0;return-1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-session-row");let Y=document.createElement("span");W.appendChild(Y);let D=document.createElement("span");D.setAttribute("class","meta");let q=document.createTextNode("No active sessions");D.appendChild(q),W.appendChild(D);let V=document.createElement("span");return W.appendChild(V),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let E of L)try{E()}catch(x){}}}}]}),P=(O)=>WJ({anchor:O,test:()=>{if(Q.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-session-popover"),W.setAttribute("role","listbox"),W.setAttribute("aria-label","Sessions");let Y=document.createComment("for");W.appendChild(Y),L.push(M(Y));let D=document.createComment("if");W.appendChild(D),L.push(B(D));let q=document.createElement("div");q.setAttribute("class","cb-session-sep"),q.setAttribute("role","separator"),W.appendChild(q);let V=document.createElement("div");V.setAttribute("class","cb-session-action"),L.push(t(V,"click",function(){let R=Q.startNewSession;return typeof R==="function"?R.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("sui-icon");E.setAttribute("name","plus"),E.setAttribute("size","sm"),E.setAttribute("aria-hidden","true"),V.appendChild(E);let x=document.createElement("span"),v=document.createTextNode("Start new session");x.appendChild(v),V.appendChild(x),W.appendChild(V);let A=document.createElement("div");A.setAttribute("class","cb-session-action"),L.push(t(A,"click",function(){let R=Q.viewChats;return typeof R==="function"?R.bind(void 0):()=>{}}(),{modifiers:[]}));let F=document.createElement("sui-icon");F.setAttribute("name","columns"),F.setAttribute("size","sm"),F.setAttribute("aria-hidden","true"),A.appendChild(F);let U=document.createElement("span"),N=document.createTextNode("View all chats");return U.appendChild(N),A.appendChild(U),W.appendChild(A),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let R of L)try{R()}catch(zJ){}}}}]}),H=(O)=>VJ({anchor:O,items:()=>Q.findItems(),key:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,((Y)=>G.path)(W)},render:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,W.__v=k(0),((Y)=>{let D=document.createDocumentFragment(),q=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),q.push(s(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false"))),q.push(s(V,"data-path",()=>(Y.__v(),Y.item.path))),q.push(t(V,"click",function(){let U=()=>Y.activateNavItem(Y.item.path);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("sui-icon");q.push(s(E,"name",()=>(Y.__v(),Y.item.icon))),E.setAttribute("size","sm"),E.setAttribute("aria-hidden","true"),V.appendChild(E);let x=document.createElement("span"),v=document.createTextNode("");x.appendChild(v),q.push(h(v,()=>(Y.__v(),Y.item.label))),V.appendChild(x);let A=document.createElement("span");A.setAttribute("class","hint");let F=document.createTextNode("");return A.appendChild(F),q.push(h(F,()=>(Y.__v(),Y.item.path))),V.appendChild(A),D.appendChild(V),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let U of q)try{U()}catch(N){}},update:(U,N)=>{Y.item=U,Y.__i=N,Y.__v.set((R)=>R+1)}}})(W)}}),T=(O)=>WJ({anchor:O,test:()=>{if(Q.findItems().length>0)return 0;return 1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Jump to");W.appendChild(Y),G.appendChild(W);let D=document.createElement("div");D.setAttribute("class","cb-list"),D.setAttribute("role","listbox");let q=document.createComment("for");return D.appendChild(q),L.push(H(q)),G.appendChild(D),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(E){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No matches.");return W.appendChild(Y),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let D of L)try{D()}catch(q){}}}}]}),$=(O)=>VJ({anchor:O,items:()=>Q.cmdItems(),key:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,((Y)=>G.key)(W)},render:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,W.__v=k(0),((Y)=>{let D=document.createDocumentFragment(),q=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),q.push(s(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false"))),q.push(s(V,"data-cmd",()=>(Y.__v(),Y.item.key))),q.push(t(V,"click",function(){let U=()=>Y.activateCmd(Y.item.key);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("span");E.setAttribute("style","font-family:var(--font-mono)");let x=document.createTextNode("");E.appendChild(x),q.push(h(x,()=>(Y.__v(),Y.item.label))),V.appendChild(E);let v=document.createElement("span");V.appendChild(v);let A=document.createElement("span");A.setAttribute("class","hint");let F=document.createTextNode("");return A.appendChild(F),q.push(h(F,()=>(Y.__v(),Y.item.hint))),V.appendChild(A),D.appendChild(V),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let U of q)try{U()}catch(N){}},update:(U,N)=>{Y.item=U,Y.__i=N,Y.__v.set((R)=>R+1)}}})(W)}}),K=(O)=>WJ({anchor:O,test:()=>{if(Q.cmdItems().length>0)return 0;return 1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Actions");W.appendChild(Y),G.appendChild(W);let D=document.createElement("div");D.setAttribute("class","cb-list"),D.setAttribute("role","listbox");let q=document.createComment("for");return D.appendChild(q),L.push($(q)),G.appendChild(D),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(E){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No commands match.");return W.appendChild(Y),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let D of L)try{D()}catch(q){}}}}]}),b=(O)=>VJ({anchor:O,items:()=>Q.mentionItems(),key:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,((Y)=>G.id+G.kind)(W)},render:(G,L)=>{let W=Object.create(Q);return W.item=G,W.__i=L,W.__v=k(0),((Y)=>{let D=document.createDocumentFragment(),q=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),q.push(s(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false"))),q.push(s(V,"data-mention-kind",()=>(Y.__v(),Y.item.kind))),q.push(s(V,"data-mention-id",()=>(Y.__v(),Y.item.id))),q.push(t(V,"click",function(){let U=()=>Y.activateMention(Y.item);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("sui-icon");q.push(s(E,"name",()=>(Y.__v(),Y.item.icon))),E.setAttribute("size","sm"),E.setAttribute("aria-hidden","true"),V.appendChild(E);let x=document.createElement("span"),v=document.createTextNode("");x.appendChild(v),q.push(h(v,()=>(Y.__v(),Y.item.label))),V.appendChild(x);let A=document.createElement("span");A.setAttribute("class","hint");let F=document.createTextNode("");return A.appendChild(F),q.push(h(F,()=>(Y.__v(),Y.item.kind))),V.appendChild(A),D.appendChild(V),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let U of q)try{U()}catch(N){}},update:(U,N)=>{Y.item=U,Y.__i=N,Y.__v.set((R)=>R+1)}}})(W)}}),o=(O)=>WJ({anchor:O,test:()=>{if(Q.mentionItems().length>0)return 0;return 1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Reference");W.appendChild(Y),G.appendChild(W);let D=document.createElement("div");D.setAttribute("class","cb-list"),D.setAttribute("role","listbox");let q=document.createComment("for");return D.appendChild(q),L.push(b(q)),G.appendChild(D),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(E){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No matching views or agents.");return W.appendChild(Y),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let D of L)try{D()}catch(q){}}}}]}),i=(O)=>VJ({anchor:O,items:()=>Q.createSurfaces(),key:(G,L)=>{let W=Object.create(Q);return W.surf=G,W.__i=L,((Y)=>G.id)(W)},render:(G,L)=>{let W=Object.create(Q);return W.surf=G,W.__i=L,W.__v=k(0),((Y)=>{let D=document.createDocumentFragment(),q=[],V=document.createElement("button");V.setAttribute("class","cb-surface-btn"),V.setAttribute("type","button"),q.push(s(V,"aria-pressed",()=>(Y.__v(),Y.bar.createSurface()===Y.surf.id?"true":"false"))),q.push(t(V,"click",function(){let A=()=>Y.selectSurface(Y.surf.id);return typeof A==="function"?A.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("sui-icon");q.push(s(E,"name",()=>(Y.__v(),Y.surf.icon))),E.setAttribute("aria-hidden","true"),V.appendChild(E);let x=document.createElement("span"),v=document.createTextNode("");return x.appendChild(v),q.push(h(v,()=>(Y.__v(),Y.surf.label))),V.appendChild(x),D.appendChild(V),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let A of q)try{A()}catch(F){}},update:(A,F)=>{Y.surf=A,Y.__i=F,Y.__v.set((U)=>U+1)}}})(W)}}),n=(O)=>VJ({anchor:O,items:()=>Q.currentSurfaceDef().shapes,key:(G,L)=>{let W=Object.create(Q);return W.sh=G,W.__i=L,((Y)=>G)(W)},render:(G,L)=>{let W=Object.create(Q);return W.sh=G,W.__i=L,W.__v=k(0),((Y)=>{let D=document.createDocumentFragment(),q=[],V=document.createElement("option");q.push(s(V,"value",()=>(Y.__v(),Y.sh)));let E=document.createTextNode("");return V.appendChild(E),q.push(h(E,()=>(Y.__v(),Y.sh))),D.appendChild(V),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let x of q)try{x()}catch(v){}},update:(x,v)=>{Y.sh=x,Y.__i=v,Y.__v.set((A)=>A+1)}}})(W)}}),XJ=(O)=>WJ({anchor:O,test:()=>{if(Q.bar.createError())return 0;return-1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-create-error");let Y=document.createTextNode("");return W.appendChild(Y),L.push(h(Y,()=>Q.bar.createError())),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let D of L)try{D()}catch(q){}}}}]}),a=(O)=>VJ({anchor:O,items:()=>Q.bar.agents(),key:(G,L)=>{let W=Object.create(Q);return W.a=G,W.__i=L,((Y)=>G.id)(W)},render:(G,L)=>{let W=Object.create(Q);return W.a=G,W.__i=L,W.__v=k(0),((Y)=>{let D=document.createDocumentFragment(),q=[],V=document.createElement("option");q.push(s(V,"value",()=>(Y.__v(),Y.a.id)));let E=document.createTextNode("");return V.appendChild(E),q.push(h(E,()=>(Y.__v(),Y.a.name??Y.a.label??Y.a.id))),D.appendChild(V),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let x of q)try{x()}catch(v){}},update:(x,v)=>{Y.a=x,Y.__i=v,Y.__v.set((A)=>A+1)}}})(W)}}),ZJ=(O)=>WJ({anchor:O,test:()=>{if(Q.bar.agents().length===0)return 0;return 1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("option");W.setAttribute("value","");let Y=document.createTextNode("No agents found");return W.appendChild(Y),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let D of L)try{D()}catch(q){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createComment("for");return G.appendChild(W),L.push(a(W)),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(D){}}}}]}),_=(O)=>VJ({anchor:O,items:()=>Q.bar.providers(),key:(G,L)=>{let W=Object.create(Q);return W.p=G,W.__i=L,((Y)=>G)(W)},render:(G,L)=>{let W=Object.create(Q);return W.p=G,W.__i=L,W.__v=k(0),((Y)=>{let D=document.createDocumentFragment(),q=[],V=document.createElement("option");q.push(s(V,"value",()=>(Y.__v(),Y.p)));let E=document.createTextNode("");return V.appendChild(E),q.push(h(E,()=>(Y.__v(),Y.p))),D.appendChild(V),{nodes:Array.from(D.childNodes),cleanup:()=>{for(let x of q)try{x()}catch(v){}},update:(x,v)=>{Y.p=x,Y.__i=v,Y.__v.set((A)=>A+1)}}})(W)}}),e=(O)=>WJ({anchor:O,test:()=>{if(Q.bar.providers().length===0)return 0;return 1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("option");W.setAttribute("value","");let Y=document.createTextNode("No providers installed");return W.appendChild(Y),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let D of L)try{D()}catch(q){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createComment("for");return G.appendChild(W),L.push(_(W)),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(D){}}}}]}),C=(O)=>WJ({anchor:O,test:()=>{if(Q.bar.spawnError())return 0;return-1},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-spawn-error");let Y=document.createTextNode("");return W.appendChild(Y),L.push(h(Y,()=>Q.bar.spawnError())),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let D of L)try{D()}catch(q){}}}}]}),m=(O)=>WJ({anchor:O,test:()=>{let G=Q.bar.mode();if(G==="find")return 0;if(G==="command")return 1;if(G==="mention")return 2;if(G==="create")return 3;return 4},branches:[()=>{let G=document.createDocumentFragment(),L=[],W=document.createComment("if");return G.appendChild(W),L.push(T(W)),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(D){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createComment("if");return G.appendChild(W),L.push(K(W)),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(D){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createComment("if");return G.appendChild(W),L.push(o(W)),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(D){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Create");W.appendChild(Y),G.appendChild(W);let D=document.createElement("div");D.setAttribute("class","cb-create-surfaces"),D.setAttribute("role","group"),D.setAttribute("aria-label","Surface type");let q=document.createComment("for");D.appendChild(q),L.push(i(q)),G.appendChild(D);let V=document.createElement("div");V.setAttribute("class","cb-create-grid");let E=document.createElement("label"),x=document.createElement("span"),v=document.createTextNode("Name (kebab-case)");x.appendChild(v),E.appendChild(x);let A=document.createElement("input");A.setAttribute("type","text"),A.setAttribute("class","cb-create-name"),L.push(s(A,"placeholder",()=>"my-"+Q.bar.createSurface())),L.push(DJ(A,Q.bar.createName)),A.setAttribute("autocomplete","off"),E.appendChild(A),V.appendChild(E);let F=document.createElement("label"),U=document.createElement("span"),N=document.createTextNode("");U.appendChild(N),L.push(h(N,()=>Q.currentSurfaceDef().shapeLabel)),F.appendChild(U);let R=document.createElement("select");R.setAttribute("class","cb-create-shape"),L.push(DJ(R,Q.bar.createShape));let zJ=document.createComment("for");R.appendChild(zJ),L.push(n(zJ)),F.appendChild(R),V.appendChild(F);let c=document.createElement("label");c.setAttribute("class","span-2");let LJ=document.createElement("span"),l=document.createTextNode("Purpose (optional)");LJ.appendChild(l),c.appendChild(LJ);let MJ=document.createElement("input");MJ.setAttribute("type","text"),MJ.setAttribute("class","cb-create-purpose"),MJ.setAttribute("placeholder","One-line description"),L.push(DJ(MJ,Q.bar.createPurpose)),MJ.setAttribute("autocomplete","off"),c.appendChild(MJ),V.appendChild(c),G.appendChild(V);let vJ=document.createComment("if");G.appendChild(vJ),L.push(XJ(vJ));let oJ=document.createElement("div");oJ.setAttribute("class","cb-create-actions");let CJ=document.createElement("button");CJ.setAttribute("class","cb-create-submit"),CJ.setAttribute("type","button"),L.push(s(CJ,"disabled",()=>Q.bar.createSubmitting()||!Q.createNameValid())),L.push(t(CJ,"click",function(){let mJ=Q.submitCreate;return typeof mJ==="function"?mJ.bind(void 0):()=>{}}(),{modifiers:[]}));let L0=document.createTextNode("");return CJ.appendChild(L0),L.push(h(L0,()=>Q.bar.createSubmitting()?"Creating…":"Create "+Q.currentSurfaceDef().label)),oJ.appendChild(CJ),G.appendChild(oJ),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let mJ of L)try{mJ()}catch(YX){}}}},()=>{let G=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-spawn-card");let Y=document.createElement("div");Y.setAttribute("class","cb-spawn-title");let D=document.createTextNode("Start a session");Y.appendChild(D),W.appendChild(Y);let q=document.createElement("div");q.setAttribute("class","cb-spawn-row");let V=document.createElement("label"),E=document.createTextNode("Agent");V.appendChild(E),q.appendChild(V);let x=document.createElement("select");x.setAttribute("class","cb-spawn-agent"),L.push(DJ(x,Q.bar.spawnAgent));let v=document.createComment("if");x.appendChild(v),L.push(ZJ(v)),q.appendChild(x),W.appendChild(q);let A=document.createElement("div");A.setAttribute("class","cb-spawn-row");let F=document.createElement("label"),U=document.createTextNode("Provider");F.appendChild(U),A.appendChild(F);let N=document.createElement("select");N.setAttribute("class","cb-spawn-provider"),L.push(DJ(N,Q.bar.spawnProvider));let R=document.createComment("if");N.appendChild(R),L.push(e(R)),A.appendChild(N),W.appendChild(A);let zJ=document.createComment("if");W.appendChild(zJ),L.push(C(zJ));let c=document.createElement("button");c.setAttribute("class","cb-spawn-start"),c.setAttribute("type","button"),L.push(s(c,"disabled",()=>Q.bar.spawning()||!Q.bar.spawnAgent()||!Q.bar.spawnProvider())),L.push(t(c,"click",function(){let l=Q.spawnSession;return typeof l==="function"?l.bind(void 0):()=>{}}(),{modifiers:[]}));let LJ=document.createTextNode("");return c.appendChild(LJ),L.push(h(LJ,()=>Q.bar.spawning()?"Starting…":"Start session")),W.appendChild(c),G.appendChild(W),{nodes:Array.from(G.childNodes),cleanup:()=>{for(let l of L)try{l()}catch(MJ){}}}}]}),I=Z.shadowRoot??Z;NJ(I);let f=document.createElement("div");f.setAttribute("class","cb-input-row");let S=document.createElement("span");S.setAttribute("class","cb-mode-glyph"),z.push(EJ(S,"ai",()=>Q.bar.mode()==="ai"));let QJ=document.createTextNode("");S.appendChild(QJ),z.push(h(QJ,()=>Q.modeGlyph())),f.appendChild(S);let YJ=document.createElement("input");YJ.setAttribute("class","cb-search"),YJ.setAttribute("type","text"),YJ.setAttribute("autocomplete","off"),YJ.setAttribute("spellcheck","false"),z.push(s(YJ,"placeholder",()=>Q.searchPlaceholder())),z.push(DJ(YJ,Q.bar.value)),YJ.setAttribute("aria-label","Command bar search"),YJ.setAttribute("aria-autocomplete","list"),z.push(t(YJ,"input",function(){let O=Q.onInput;return typeof O==="function"?O.bind(void 0):()=>{}}(),{modifiers:[]})),f.appendChild(YJ);let FJ=document.createElement("kbd");FJ.setAttribute("class","cb-esc"),FJ.setAttribute("aria-label","Escape to close");let PJ=document.createTextNode("esc");FJ.appendChild(PJ),f.appendChild(FJ);let jJ=document.createElement("div");jJ.setAttribute("class","cb-chip-wrap");let GJ=document.createElement("span");GJ.setAttribute("class","cb-session-chip"),GJ.setAttribute("role","button"),GJ.setAttribute("aria-haspopup","listbox"),z.push(t(GJ,"click",function(){let O=Q.toggleSessionPicker;return typeof O==="function"?O.bind(void 0):()=>{}}(),{modifiers:[]}));let UJ=document.createElement("span");UJ.setAttribute("class","cb-dot"),z.push(d(()=>{let O=Q.chipDotBg();if(O==null)UJ.style.removeProperty("background");else UJ.style.setProperty("background",String(O))})),GJ.appendChild(UJ);let u=document.createElement("span"),p=document.createTextNode("");u.appendChild(p),z.push(h(p,()=>Q.chipLabel())),GJ.appendChild(u),jJ.appendChild(GJ);let g=document.createComment("if");jJ.appendChild(g),z.push(P(g)),f.appendChild(jJ),I.appendChild(f);let w=document.createElement("div");w.setAttribute("class","cb-body");let y=document.createComment("match");if(w.appendChild(y),z.push(m(y)),I.appendChild(w),j!=null){let O=Object.getOwnPropertyDescriptor(j,"__shipSetLifecycle")?.value;if(O){let G=new Proxy(Q,{get(L,W){return L[W]},set(L,W,Y){let D=L[W];if(D!=null&&typeof D.set==="function"){if(typeof Y==="function")D.set(()=>Y);else D.set(Y);return!0}return!1}});if(O.mount){let L=O.mount(G,Z);if(typeof L==="function")z.push(L)}if(O.unmount)z.push(()=>O.unmount(G))}}return Q}});
