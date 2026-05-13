var S0=Symbol.for("sui:error-boundary");function OJ(J){let X=J;while(X){if(X instanceof Element){let Q=X[S0];if(Q)return Q}let Z=X.parentNode;if(Z){X=Z;continue}let z=X;if(z.host){X=z.host;continue}X=null}return null}function TJ(J,X,Z){try{return Z()}catch(z){let Q=OJ(J);if(Q){Q.catch(z,X);return}console.error("[sui] uncaught throw (no error boundary):",z);return}}var PJ=null,KJ=null,y0=0,b0=new Set,wJ=[];function sJ(J,X){wJ.push(J);try{return X()}finally{wJ.pop()}}function $0(){return wJ.length>0?wJ[wJ.length-1]:null}function V0(J){if(PJ===null)return;if(J.has(PJ))return;let X=PJ;if(J.add(X),KJ!==null)KJ.push(()=>J.delete(X))}function y(J){let X=J,Z=new Set,z=()=>{return V0(Z),X};return z.set=(Q)=>{let G=typeof Q==="function"?Q(X):Q;if(Object.is(G,X))return;if(X=G,y0>0)for(let M of Z)b0.add(M);else for(let M of[...Z])M()},z.__subs=Z,z.__peek=()=>X,z}function JJ(J){let X,Z=!0,z=new Set,Q=[],G=()=>{if(Z)return;if(Z=!0,z.size===0)return;let B=X;if(M(),Object.is(B,X))return;for(let D of[...z])D()},M=()=>{for(let x of Q)x();Q=[];let B=PJ,D=KJ;PJ=G,KJ=Q;try{X=J()}finally{PJ=B,KJ=D}Z=!1},K=()=>{if(V0(z),Z)M();return X};return K.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},K.__subs=z,K.__peek=()=>{if(Z)M();return X},K}function h(J){let X=void 0,Z=!0,z=[],Q=$0(),G=()=>{if(!Z)return;if(X){try{X()}catch(B){console.error("[sui] effect cleanup threw:",B)}X=void 0}for(let B of z)B();z=[];let M=PJ,K=KJ;PJ=G,KJ=z;try{X=J()}catch(B){let D=Q?OJ(Q):null;if(D)D.catch(B,"effect");else console.error("[sui] effect threw:",B)}finally{PJ=M,KJ=K}};return G(),()=>{Z=!1;for(let M of z)M();if(z=[],X){try{X()}catch(M){console.error("[sui] effect cleanup threw:",M)}X=void 0}}}var vJ=null;function xJ(J,X){let Z=vJ;vJ=J;try{return X()}finally{vJ=Z}}function kJ(J){let X=vJ;if(!X)return;X.cleanup(J)}function SJ(){let J=[],X=!1;return{cleanup(z){if(X){try{z()}catch(Q){console.error("[sui/scope] late cleanup threw:",Q)}return}J.push(z)},dispose(){if(X)return;X=!0;for(let z=J.length-1;z>=0;z--)try{J[z]()}catch(Q){console.error("[sui/scope] cleanup threw:",Q)}J.length=0},get disposed(){return X}}}var M0=Symbol("sui.shared");function t(J){return J[M0]=!0,J}function F0(J){return!!J?.[M0]}function tJ(J,X,Z){J[`__sui_ctx_${X}`]=Z}function yJ(J,X){let Z=J;while(Z){let z=Z[`__sui_ctx_${X}`];if(z!==void 0)return z;let Q=Z.parentElement;if(Q){Z=Q;continue}let G=Z.getRootNode();Z=G instanceof ShadowRoot?G.host:null}return null}function _J(J){let X={};for(let Z in J)if(Object.prototype.hasOwnProperty.call(J,Z)&&F0(J[Z]))X[Z]=J[Z];return X}function eJ(J=null){return y(null)}function J0(J,X,Z){let z=J[X];if(typeof z==="function"&&typeof z.set==="function")return z.set(Z),z;let Q=y(Z);return J[X]=Q,Q}function zJ(J,X){let Z;return h(()=>{let z=X(),Q=z==null?"":String(z);if(Q===Z)return;if(Z===void 0&&J.textContent===Q){Z=Q;return}J.textContent=Q,Z=Q})}function MJ(J,X,Z){let z,Q=!1;return h(()=>{let G=Z();if(Q&&G===z)return;let M=G==null||G===!1||G===""?null:G===!0?"":String(G);if(Q||J.getAttribute(X)!==M)if(M===null)J.removeAttribute(X);else J.setAttribute(X,M);z=G,Q=!0})}function EJ(J,X,Z){let z;return h(()=>{let Q=!!Z();if(Q===z)return;if(z===void 0&&Q===J.classList.contains(X)){z=Q;return}if(Q)J.classList.add(X);else J.classList.remove(X);z=Q})}function HJ(J,X,Z){let z=h(()=>{let G=X();if(J.value!==G)J.value=G}),Q=()=>{let G=J.value;if(X.set(G),Z)Z(G)};return J.addEventListener("input",Q),()=>{z(),J.removeEventListener("input",Q)}}function e(J,X,Z,z={}){let Q=z.modifiers??[],G,M=!0,K=(D)=>{for(let O of Q)switch(O.kind){case"preventDefault":D.preventDefault();break;case"stopPropagation":D.stopPropagation();break;case"stopImmediatePropagation":D.stopImmediatePropagation();break;case"self":if(D.target!==J)return;break;case"trusted":if(!D.isTrusted)return;break;case"key":{if(!D0(D,O.arg??""))return;break}}let x=Q.find((O)=>O.kind==="debounce"),f=Q.find((O)=>O.kind==="throttle");if(x){let O=Number(x.arg)||0;if(G!==void 0)clearTimeout(G);G=setTimeout(()=>{G=void 0,Z(D)},O);return}if(f){let O=Number(f.arg)||0;if(!M)return;M=!1,Z(D),G=setTimeout(()=>{M=!0},O);return}Z(D)},B={};if(z.capture)B.capture=!0;if(z.passive)B.passive=!0;if(z.once)B.once=!0;return J.addEventListener(X,K,B),()=>{if(J.removeEventListener(X,K,B),G!==void 0)clearTimeout(G)}}function D0(J,X){let Z=X.split("+").map((x)=>x.toLowerCase()),z=Z.pop()??"",Q=Z.includes("meta")||Z.includes("cmd"),G=Z.includes("ctrl")||Z.includes("control"),M=Z.includes("shift"),K=Z.includes("alt")||Z.includes("option"),B=Z.includes("mod"),D=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(Q&&!J.metaKey)return!1;if(G&&!J.ctrlKey)return!1;if(M&&!J.shiftKey)return!1;if(K&&!J.altKey)return!1;if(B&&!(D?J.metaKey:J.ctrlKey))return!1;return J.key.toLowerCase()===z}function H0(J,X,Z){J.dispatchEvent(new CustomEvent(X,{detail:Z,bubbles:!0,composed:!0}))}function P0(J,X){let Z=J,z=Object.keys(X);for(let Q of z)Z[Q]=X[Q];kJ(()=>{for(let Q of z)if(Z[Q]===X[Q])try{delete Z[Q]}catch{Z[Q]=void 0}})}function uJ(J){let X=J;X.emit=function(Z,z){H0(J,Z,z)},X.expose=function(Z){P0(J,Z)}}var f0=Symbol("sui:transition");function hJ(J){return J[f0]}function m0(){if(typeof window>"u"||typeof window.matchMedia!=="function")return!1;try{return window.matchMedia("(prefers-reduced-motion: reduce)").matches}catch{return!1}}function U0(J){if(!m0())return J;return{...J,duration:0,delay:0}}function K0(J){let X=hJ(J);if(!X?.inT?.in)return;X.current?.cancel();let Z=X.inT.in(J,U0(X.inOpts));return X.current=Z,Z}function B0(J){let X=hJ(J);if(!X?.outT?.out)return;X.current?.cancel();let Z=X.outT.out(J,U0(X.outOpts));return X.current=Z,Z}function Z0(J){if(J.nodeType!==1)return;let X=J;if(X.hasAttribute("data-sui-transition")){let Z=hJ(X);if(Z?.current?.cancel(),Z)Z.current=void 0}for(let Z of X.querySelectorAll("[data-sui-transition]")){let z=hJ(Z);if(z?.current?.cancel(),z)z.current=void 0}}function O0(J,X){for(let Z of J){if(Z.nodeType!==1)continue;let z=Z;if(z.hasAttribute("data-sui-transition"))X(z);for(let Q of z.querySelectorAll("[data-sui-transition]"))X(Q)}}function RJ(J){O0(J,(X)=>{K0(X)})}function bJ(J){let X=[];if(O0(J,(Z)=>{let z=B0(Z);if(z)X.push(z.finished.catch(()=>{}))}),X.length===0)return null;return Promise.all(X).then(()=>{return})}function g0(J,X,Z,z,Q){if(Z.fill(0,0,X),X===0)return;let G=0;for(let K=0;K<X;K++){let B=J[2*K+1],D=0,x=G;while(D<x){let f=D+x>>>1;if(J[2*Q[f]+1]<B)D=f+1;else x=f}if(z[K]=D>0?Q[D-1]:-1,Q[D]=K,D===G)G++}let M=Q[G-1];while(M>=0)Z[M]=1,M=z[M]}function FJ(J){let{anchor:X,items:Z,key:z,render:Q}=J,G=[],M=!0,K=new Set,B=new Map,D=new Map,x=new Map,f=new Set,O=[],k=[],p=new Int32Array(0),d=new Uint8Array(0),a=new Int32Array(0),XJ=new Int32Array(0),i=(v)=>{for(let n of v.nodes)try{n.parentNode?.removeChild(n)}catch{}v.cleanup?.()},WJ=h(()=>{if(!M)return;let v=Z(),n=v.length;if(p.length<n*2)p=new Int32Array(Math.max(p.length*2,n*2,16));if(d.length<n){let U=Math.max(d.length*2,n,8);d=new Uint8Array(U),a=new Int32Array(U),XJ=new Int32Array(U)}for(let U=0;U<n;U++){let E=v[U],c=z(E,U);if(B.has(c))console.warn("[sui] {#for}: duplicate key",c,"at index",U,"(prior at",B.get(c),"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");B.set(c,U)}for(let U of G)D.set(U.key,U);for(let U of G){if(B.has(U.key))continue;let E=bJ(U.nodes);if(E===null){i(U);continue}K.add(U),E.then(()=>{if(!K.delete(U))return;i(U)})}k.length=0,O.length=0;for(let U=0;U<n;U++){let E=v[U],c=z(E,U),s=D.get(c),GJ;if(s)s.item=E,TJ(X,"render",()=>s.update?.(E,U)),GJ=s;else{let ZJ=TJ(X,"render",()=>Q(E,U));if(!ZJ)continue;GJ={key:c,item:E,nodes:ZJ.nodes,...ZJ.cleanup&&{cleanup:ZJ.cleanup},...ZJ.update&&{update:ZJ.update}},O.push(...ZJ.nodes)}k.push(GJ)}for(let U=0;U<G.length;U++)x.set(G[U].key,U);let o=0;for(let U=0;U<k.length;U++){let E=x.get(k[U].key);if(E!==void 0)p[2*o]=U,p[2*o+1]=E,o++}g0(p,o,d,a,XJ);for(let U=0;U<o;U++)if(d[U])f.add(p[2*U]);let _=X.parentNode;if(_&&k.length>0){let U=X,E=null;for(let c=0;c<k.length;c++){let s=k[c];if(f.has(c)){if(E)_.insertBefore(E,U.nextSibling),E=null;U=s.nodes[s.nodes.length-1]??U}else{if(!E)E=document.createDocumentFragment();let GJ=s.nodes;for(let ZJ=0;ZJ<GJ.length;ZJ++)E.appendChild(GJ[ZJ])}}if(E){_.insertBefore(E,U.nextSibling);let c=k[k.length-1].nodes;U=c[c.length-1]??U}}let w=G;if(G=k,k=w,O.length>0)RJ(O);B.clear(),D.clear(),x.clear(),f.clear(),O.length=0});return()=>{M=!1,WJ();for(let v of G)i(v);for(let v of K)i(v);K.clear(),G=[],k.length=0}}function YJ(J){let{anchor:X}=J,{test:Z,branches:z}=v0(J)?{test:J.test,branches:J.branches}:u0(J),Q=null,G=-2,M=null,K=(D)=>{for(let x of D)try{x.parentNode?.removeChild(x)}catch{}},B=h(()=>{let D=Z();if(Q&&D===G)return;if(M&&D===M.idx&&D>=0&&D<z.length){let O=M;M=null;for(let k of O.nodes)Z0(k);Q={nodes:O.nodes,...O.cleanup&&{cleanup:O.cleanup}},G=D,RJ(Q.nodes);return}if(Q){let O={idx:G,nodes:Q.nodes,...Q.cleanup&&{cleanup:Q.cleanup}},k=bJ(O.nodes);if(Q=null,k===null)K(O.nodes),O.cleanup?.();else M=O,k.then(()=>{if(M!==O)return;K(O.nodes),O.cleanup?.(),M=null})}if(G=D,D<0||D>=z.length)return;let x=TJ(X,"render",()=>z[D]());if(!x)return;Q=x;let f=X.parentNode;if(!f)return;for(let O of Q.nodes)f.insertBefore(O,X);RJ(Q.nodes)});return()=>{if(B(),Q)K(Q.nodes),Q.cleanup?.(),Q=null;if(M)K(M.nodes),M.cleanup?.(),M=null}}function v0(J){return typeof J.test==="function"}function u0(J){let{cond:X,then_:Z,else_:z}=J;return{test:()=>X()?0:z?1:-1,branches:z?[Z,z]:[Z]}}var Q0=(J)=>J??"",z0=(J)=>{if(J===null||J==="")return NaN;return Number(J)},W0=(J)=>{if(J===null)return!1;if(J.toLowerCase()==="false")return!1;return!0};function dJ(J){return(X)=>{if(X===null||X==="")return J;try{return JSON.parse(X)}catch{return J}}}function Y0(J,X){let Z=new Set(J);return(z)=>z!==null&&Z.has(z)?z:X}var lJ=(J)=>{switch(J.type){case"string":return Q0;case"bool":return W0;case"number":return z0;case"union":{let X=J.values??[],Z=J.default??X[0]??"";return Y0(X,Z)}case"object":return dJ(J.default);case"array":return dJ(J.default??[])}};function I0(J,X,Z,z){if(!X)return;let Q={},G,M=Z.formValue;if(M!==void 0){let{sig:B,propDef:D}=N0(Z,z,M,"formValue"),x=D.default,f=lJ(D);G=h(()=>{let O=B();X.setFormValue(h0(O))}),Q.reset=()=>{B.set(x)},Q.restore=(O)=>{B.set(f(O))}}let K=Z.formDisabled;if(K!==void 0){let{sig:B,propDef:D}=N0(Z,z,K,"formDisabled");if(D.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${K}" must be a bool-typed prop`);Q.disabled=(x)=>{B.set(!!x)}}if(J.__shipSetFormHooks=Q,G)J.__shipSetFormValueCleanup=G;return G}function C0(J){let X=J;delete X.__shipSetFormHooks,delete X.__shipSetFormValueCleanup}function h0(J){if(J===null||J===void 0)return"";if(typeof J==="boolean")return J?"on":"";return String(J)}function N0(J,X,Z,z){let Q=J.props[Z];if(!Q)throw Error(`[sui-runtime] ${z}: "${Z}" — no matching prop on this component`);let G=X[AJ(Z)];if(!G)throw Error(`[sui-runtime] ${z}: "${Z}" — prop signal missing`);return{sig:G,propDef:Q}}var $J={string(J={}){return{type:"string",...J}},bool(J={}){return{type:"bool",...J}},number(J={}){return{type:"number",...J}},union(J,X={}){return{type:"union",values:J,...X}},object(J={}){return{type:"object",...J}},array(J={}){return{type:"array",...J}}},d0=(J,X)=>{if(X===void 0||X===null)return null;switch(J.type){case"string":return X===""?null:String(X);case"number":return Number.isNaN(X)?null:String(X);case"bool":return X?"":null;case"union":return String(X);case"object":case"array":try{return JSON.stringify(X)}catch{return null}}},l0=(J,X)=>{let Z=`ship-set-style-${J}`;if(document.getElementById(Z))return;let z=document.createElement("style");z.id=Z,z.textContent=`[data-component="${J}"] { ${X} }`,document.head.appendChild(z)},p0=(J,X,Z,z)=>{let Q=J.getAttribute(X),G=Q!==null?z(Q):Z.default;return y(G)},i0=(J)=>{let X=[],Z={};for(let[G,M]of Object.entries(J.props??{}))X.push(G),Z[G]={camel:AJ(G),decode:lJ(M)};let z=J.formAssociated===!0||J.formValue!==void 0||J.formDisabled!==void 0,Q={observedAttrs:X,props:Z};if(z)Q.formAssociated=!0;return Q};function BJ(J,X){let Z=i0(X),z=Object.entries(X.props??{});G0(J,Z,(Q,G,M)=>{let K=null;if(X.shadow){if(K=Q.shadowRoot??Q.attachShadow({mode:"open"}),X.styles){let D=document.createElement("style");D.textContent=X.styles,K.appendChild(D)}}else if(X.styles)l0(J,X.styles);let B=[];for(let[D,x]of z){let f=AJ(D),O=Z.props[D].decode,k=p0(Q,D,x,O);if(G[f]=k,x.reflect){let p=!0,d=h(()=>{let a=k();if(p){p=!1;return}let XJ=d0(x,a);if(XJ===null)Q.removeAttribute(D);else Q.setAttribute(D,XJ)});B.push(d)}}if(X.formValue!==void 0||X.formDisabled!==void 0){let D=I0(Q,M,X,G);if(typeof D==="function")B.push(D)}if(X.setup){let D={props:G,host:Q,cleanups:B};if(M)D.internals=M;X.setup(D)}return()=>{C0(Q);for(let D of B.splice(0))try{D()}catch(x){console.error(`[ship-set:${J}] cleanup threw:`,x)}}})}var T0=new Set;function G0(J,X,Z){if(customElements.get(J)){if(!T0.has(J))T0.add(J),console.warn(`[sui-runtime] customElements.define("${J}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(X.formAssociated){class Q extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return X.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=J,uJ(this),this.scope=SJ();try{this.cleanup=xJ(this.scope,()=>sJ(this,()=>Z(this,this.propSignals,this.internals)))}catch(G){let M=OJ(this.parentNode);if(M)M.catch(G,"mount");else console.error(`[ship-set:${J}] setup threw:`,G)}}disconnectedCallback(){try{this.cleanup?.()}catch(G){console.error(`[ship-set:${J}] cleanup threw:`,G)}this.cleanup=void 0;try{this.scope?.dispose()}catch(G){console.error(`[ship-set:${J}] scope dispose threw:`,G)}this.scope=void 0}attributeChangedCallback(G,M,K){let B=X.props[G];if(!B)return;let D=this.propSignals[B.camel];if(D)D.set(B.decode(K))}formResetCallback(){let G=this.__shipSetFormHooks;try{G?.reset?.()}catch(M){console.error(`[ship-set:${J}] formResetCallback threw:`,M)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(G,M){let K=this.__shipSetFormHooks;try{K?.restore?.(G)}catch(B){console.error(`[ship-set:${J}] formStateRestoreCallback threw:`,B)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:G,mode:M}}))}formDisabledCallback(G){let M=this.__shipSetFormHooks;try{M?.disabled?.(G)}catch(K){console.error(`[ship-set:${J}] formDisabledCallback threw:`,K)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:G}}))}}customElements.define(J,Q);return}class z extends HTMLElement{static get observedAttributes(){return X.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=J,uJ(this),this.scope=SJ();try{this.cleanup=xJ(this.scope,()=>sJ(this,()=>Z(this,this.propSignals)))}catch(Q){let G=OJ(this.parentNode);if(G)G.catch(Q,"mount");else console.error(`[ship-set:${J}] setup threw:`,Q)}}disconnectedCallback(){try{this.cleanup?.()}catch(Q){console.error(`[ship-set:${J}] cleanup threw:`,Q)}this.cleanup=void 0;try{this.scope?.dispose()}catch(Q){console.error(`[ship-set:${J}] scope dispose threw:`,Q)}this.scope=void 0}attributeChangedCallback(Q,G,M){let K=X.props[Q];if(!K)return;let B=this.propSignals[K.camel];if(B)B.set(K.decode(M))}}customElements.define(J,z)}function AJ(J){return J.replace(/-([a-z])/g,(X,Z)=>Z.toUpperCase())}function j0(J,X,Z){let z=r0(J.state,{ship:X.ship,...Z}),Q={};for(let[O,k]of Object.entries(z))Q[O]=y(k);let G={ship:X.ship,route:X.route??{},navigate:X.navigate??((O)=>{}),...Z,...Q},M=a0(Q,G),K={...G},B={};for(let[O,k]of Object.entries(J.derived??{}))B[O]=JJ(()=>k(K));Object.assign(K,B);let D={};for(let[O,k]of Object.entries(J.on??{}))D[O]=(...p)=>k(M,...p);let x={},f=typeof J.use==="function"?J.use(K):J.use??{};for(let[O,k]of Object.entries(f))x[O]=j0(k,X,K);return{...Q,...B,...D,...x}}function r0(J,X){if(J===void 0)return{};if(typeof J==="function")return J(X);return J}function a0(J,X){return new Proxy({},{get(Z,z){if(z in J)return J[z]();return X[z]},set(Z,z,Q){if(z in J)return J[z].set(Q),!0;return console.warn(`[ship-set] hook handler tried to write s.${z} = … but ${z} is not a hook state key. The assignment was ignored.`),!0},has(Z,z){return z in J||z in X},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(Z,z){if(z in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[z]()};return}})}var E0="__shipSetLifecycle";function R0(J,X){let Z=o0(J,X),z={ship:X.ship,route:X.route??{},navigate:X.navigate??((O)=>{}),...X.scope??{}},Q=Z.scope?Z.scope(z,X.host):{};Object.assign(z,Q);let G=s0(z,X.stateKeys,Q),M=t0(G,z),K={...z},B=_0(Z,K);Object.assign(K,B);let D=JX(Z,K,X);for(let[O,k]of Object.entries(D))K[O]=k;let x=e0(Z,M),f={...Q,...B,...x,...D};if(Object.keys(Q).length>0&&X.host){let O=_J(Q);if(Object.keys(O).length>0){let k=X.host,p=k.tagName.toLowerCase();tJ(k,p,O)}}if(Z.mount||Z.unmount){let O={state:M};if(Z.mount)O.mount=Z.mount;if(Z.unmount)O.unmount=Z.unmount;Object.defineProperty(f,E0,{value:O,enumerable:!0,configurable:!0,writable:!1})}return f}function o0(J,X){if(typeof J==="function")return J({ship:X.ship});return J}function s0(J,X,Z){let z={},Q=X!==void 0,G=Z?Object.keys(Z):[];if(Q||G.length>0){if(X)for(let M of X){let K=J[M];if(L0(K))z[M]=K}for(let M of G){if(M in z)continue;let K=J[M];if(L0(K))z[M]=K}return z}for(let[M,K]of Object.entries(J))if(L0(K))z[M]=K;return z}function L0(J){return typeof J==="function"&&"set"in J&&typeof J.set==="function"}function t0(J,X){return new Proxy({},{get(Z,z){if(z in J)return J[z]();return X[z]},set(Z,z,Q){if(z in J){if(typeof Q==="function")J[z].set(()=>Q);else J[z].set(Q);return!0}return console.warn(`[ship-set] handler tried to write s.${z} = … but ${z} is not declared as <state name="${z}"> in the template. The assignment was ignored.`),!0},has(Z,z){return z in J||z in X},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(Z,z){if(z in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[z]()};return}})}function _0(J,X){let Z=J.derived??{},z={};for(let[Q,G]of Object.entries(Z))z[Q]=JJ(()=>G(X));return z}function e0(J,X){let Z=J.on??{},z={};for(let[Q,G]of Object.entries(Z))z[Q]=(...M)=>G(X,...M);return z}function JX(J,X,Z){if(J.use===void 0)return{};let z=typeof J.use==="function"?J.use(X):J.use,Q={};for(let[G,M]of Object.entries(z))Q[G]=j0(M,Z,X);return Q}function NJ(J){return{setup:(X)=>R0(J,X),__def:J}}if(typeof customElements<"u"){for(let J of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(J))customElements.define(J,class extends HTMLElement{})}var zX=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,WX=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function A0(J){if(typeof CSSStyleSheet>"u")return null;try{let X=new CSSStyleSheet;return X.replaceSync(J),X}catch{return null}}var w0=A0(zX),x0=A0(WX);function IJ(J){if(typeof ShadowRoot>"u")return;if(!(J instanceof ShadowRoot))return;let X=[x0,w0].filter((Z)=>Z!=null);if(X.length===0)return;try{J.adoptedStyleSheets=[...X,...J.adoptedStyleSheets??[]]}catch{}}var pJ=[{id:"view",label:"View",icon:"file",shapes:["list","form","detail","dashboard","tabs","blank"],shapeLabel:"Starting shape"},{id:"skill",label:"Skill",icon:"sparkles",shapes:["task","reference","workflow"],shapeLabel:"Skill kind"},{id:"agent",label:"Agent",icon:"users",shapes:["specialist","generalist","reviewer"],shapeLabel:"Agent kind"},{id:"pipeline",label:"Pipeline",icon:"play",shapes:["sequential","fanout","review-gated"],shapeLabel:"Topology"},{id:"trigger",label:"Trigger",icon:"zap",shapes:["webhook","schedule","file-change"],shapeLabel:"Kind"},{id:"theme",label:"Theme",icon:"palette",shapes:["light","dark","brand-accent","high-contrast"],shapeLabel:"Starting shape"}],iJ=[{label:"Dashboard",path:"/dashboard",icon:"sparkles",keywords:"home overview"},{label:"Pipelines",path:"/pipelines",icon:"play",keywords:"workflows runs jobs"},{label:"Agents",path:"/agents",icon:"users",keywords:"configure profiles"},{label:"Skills",path:"/skills",icon:"zap",keywords:"browse edit files"},{label:"Git",path:"/git",icon:"git",keywords:"status diff commit branch"},{label:"Collections",path:"/collections",icon:"folder",keywords:"data items query"},{label:"Canvas",path:"/canvas",icon:"image",keywords:"docs visual feedback"},{label:"Events",path:"/events",icon:"sparkles",keywords:"stream log kernel"},{label:"Settings",path:"/settings",icon:"settings",keywords:"config theme provider"},{label:"Chat",path:"/chat",icon:"message-square-plus",keywords:"agent conversation"},{label:"Specs",path:"/specs",icon:"file",keywords:"features design"},{label:"Theme Studio",path:"/theme-studio",icon:"palette",keywords:"design tokens"},{label:"Chats (Mission Control)",path:"/chats",icon:"messages-square",keywords:"sessions active"}],nJ=[{key:"new",label:"/new",hint:"Scaffold a view, skill, agent, …"},{key:"chats",label:"/chats",hint:"Open Mission Control"},{key:"settings",label:"/settings",hint:"Jump to settings"},{key:"lint",label:"/lint",hint:"Run lint on the current view"},{key:"end",label:"/end",hint:"End the current session"},{key:"close",label:"/close",hint:"Close the command bar"}],YX=/^[a-z][a-z0-9-]{0,48}$/;function k0(J){return YX.test(J)}function fJ(J,X){if(J.startsWith("/"))return"command";if(J.startsWith(">"))return"find";if(J.startsWith("+"))return"create";if(J.trim().length===0)return X;if(/(^|\s)@[^\s@]*$/.test(J))return"mention";return"ai"}async function DJ(J){let X=window;if(X.ship?.navigate){X.ship.navigate(J);return}history.pushState({},"",J),document.dispatchEvent(new CustomEvent("sui:navigate",{detail:{to:J}}))}var cJ=NJ({scope:(J)=>{let X=J,Z=t(X.open),z=t(X.mode),Q=t(y("")),G=t(y(null)),M=t(y([])),K=t(y([])),B=t(y([])),D=t(y([])),x=t(y(0)),f=t(y(!1)),O=t(y("")),k=t(y("")),p=t(y(!1)),d=t(y(null)),a=t(y([])),XJ=t(y("main")),i=t(y("view")),WJ=t(y("")),v=t(y("")),n=t(y("list")),o=t(y(!1)),_=t(y(null)),w=t(JJ(()=>z()==="ai"&&G()!==null)),U=eJ(null);return{open:Z,mode:z,value:Q,session:G,agents:M,providers:K,allSessions:B,views:D,selectedIdx:x,sessionPickerOpen:f,spawnAgent:O,spawnProvider:k,spawning:p,spawnError:d,thread:a,workspaceId:XJ,createSurface:i,createName:WJ,createPurpose:v,createShape:n,createSubmitting:o,createError:_,isChat:w,dialogRef:U}},derived:{triggerLabel:(J)=>{let X=J.session();return X?`Ask ${X.agent_id??"Ship"} or jump to…`:"Ask Ship or jump to…"},dotBg:(J)=>{let X=J.session();if(X?.status==="thinking")return"var(--warning,#d97706)";if(X)return"var(--success,#16a34a)";return"var(--muted-foreground)"}},on:{openFind:(J)=>{J.mode="find",J.value="",J.selectedIdx=0,J.open=!0}},mount:(J,X)=>{let Z=J,z=Z.open,Q=Z.mode,G=Z.value,M=Z.session,K=Z.agents,B=Z.providers,D=Z.allSessions,x=Z.views,f=Z.selectedIdx,O=Z.spawnAgent,k=Z.spawnProvider,p=Z.spawning,d=Z.spawnError,a=Z.thread,XJ=Z.workspaceId,i=Z.createSurface,WJ=Z.createName,v=Z.createPurpose,n=Z.createShape,o=Z.createSubmitting,_=Z.createError,w=Z.dialogRef?.(),U=null,E=0,c=h(()=>{let F=z();if(X.setAttribute("data-state",F?"open":"closed"),F){if(typeof w?.show==="function")w.show();queueMicrotask(()=>{X.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelector(".cb-search")?.focus()})}else{if(typeof w?.close==="function")w.close();X.dispatchEvent(new CustomEvent("sui-command:cancel",{detail:{mode:Q()},bubbles:!0,composed:!0}))}});X.show=(F,V)=>{if(F)Q.set(F);if(typeof V==="string")G.set(V);z.set(!0)},X.hide=()=>z.set(!1),X.toggle=(F)=>{if(z())z.set(!1);else{if(F)Q.set(F);z.set(!0)}},X.setMode=(F)=>Q.set(F);function s(F){GJ();let V=new AbortController;U=V;let A=new EventSource(`/api/v1/sessions/${F}/events`,{withCredentials:!0}),P=null;A.addEventListener("provider",(QJ)=>{let r;try{r=JSON.parse(QJ.data)}catch{return}switch(r.kind){case"message":{let qJ=r.text??"";if(!qJ)return;if(P)a.set(a().map((u)=>u.id===P?{...u,text:u.text+qJ}:u));else{let u=String(Date.now()+Math.random());P=u,a.set([...a(),{id:u,sender:"agent",text:qJ}])}break}case"tool_use":a.set([...a(),{id:String(Date.now()+Math.random()),sender:"system",text:`→ tool: ${r.tool??"?"}`}]);break;case"turn_complete":P=null;break}}),A.onerror=()=>{if(!V.signal.aborted)A.close()},V.signal.addEventListener("abort",()=>A.close())}function GJ(){U?.abort(),U=null}let ZJ=h(()=>{let F=M();if(F?.id)s(F.id);else GJ()});async function jJ(){try{let F=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(F.ok){let V=await F.json();D.set(V.sessions);let A=V.sessions.find((P)=>P.status!=="ended"&&P.agent_id);if(A&&A.id!==M()?.id)M.set(A),a.set([]);else if(!A)M.set(null)}}catch{}if(K().length===0)LJ();if(B().length===0)UJ();if(x().length===0)g()}async function LJ(){try{let F=await fetch("/api/v1/agents",{credentials:"same-origin"});if(!F.ok)return;let A=((await F.json()).items??[]).map((P)=>({id:P.id}));if(K.set(A),!O()&&A.length>0)O.set(A[0].id)}catch{}}async function UJ(){try{let F=await fetch("/api/v1/providers/installed",{credentials:"same-origin"});if(!F.ok)return;let V=await F.json();if(B.set(V.providers),!k()&&V.providers.length>0)k.set(V.providers[0])}catch{}}async function g(){try{let F=await fetch("/api/v1/views",{credentials:"same-origin"});if(!F.ok)return;let V=await F.json(),A=V.data?.views??V.views??[];x.set(A.map((P)=>P.name).sort())}catch{}}function l(){return X.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelectorAll(".cb-item")}function m(F){let V=l();if(!V?.length)return;f.set(Math.max(0,Math.min(V.length-1,f()+F)))}function S(){let F=Q(),V=G(),P=l()?.[f()];if(P){let QJ=P.dataset.path;if(QJ)DJ(QJ),G.set(""),z.set(!1);let r=P.dataset.cmd;if(r)G.set(""),b("/"+r);let qJ=P.dataset.mentionKind,u=P.dataset.mentionId;if(qJ&&u){let VJ=qJ==="view"?`@${u}`:`@${qJ}:${u}`;G.set(V.replace(/@[^@\s]*$/,`${VJ} `)),Q.set(fJ(G(),F))}X.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:F,item:P.dataset},bubbles:!0,composed:!0}));return}if(F==="command")b(V);else if(F==="find")C(V)}function b(F){let V=F.replace(/^\/+\s*/,"").toLowerCase(),A=nJ.find((P)=>P.key===V||P.key.startsWith(V));if(!A)return;switch(A.key){case"new":Q.set("create"),G.set("");break;case"chats":DJ("/chats"),z.set(!1);break;case"settings":DJ("/settings"),z.set(!1);break;case"end":if(M())L(M().id);z.set(!1);break;case"close":z.set(!1);break;case"lint":j();break}}function C(F){let V=F.replace(/^>+\s*/,"").toLowerCase().trim(),A=iJ.find((P)=>P.label.toLowerCase().includes(V)||P.keywords.toLowerCase().includes(V)||P.path.toLowerCase().includes(V));if(A)DJ(A.path),G.set(""),z.set(!1)}async function L(F){try{await fetch(`/api/v1/sessions/${F}`,{method:"DELETE",credentials:"same-origin"})}catch{}if(M()?.id===F)M.set(null);await jJ()}async function j(){let V=window.ship?.current?.()?.path?.replace(/^\/+/,"").split("/")[0]??"";if(!V)return;try{let A=await fetch("/api/v1/lint/view",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:V})});if(A.ok){let QJ=(await A.json()).data?.findings?.length??0;console.log(QJ===0?`${V}: no lint findings`:`${V}: ${QJ} lint findings`)}}catch{}z.set(!1)}let Y=(F)=>{if(F.key==="Escape"&&z()){z.set(!1);return}if(F.key===" "&&!z()){let V=F.target;if(V&&(V.tagName==="INPUT"||V.tagName==="TEXTAREA"||V.isContentEditable))return;let A=performance.now();if(A-E<350)E=0,F.preventDefault(),Q.set("ai"),G.set(""),z.set(!0);else E=A}},W=(F)=>{if(F.key==="ArrowDown")F.preventDefault(),m(1);else if(F.key==="ArrowUp")F.preventDefault(),m(-1);else if(F.key==="Enter")F.preventDefault(),S()};X.addEventListener("keydown",W),document.addEventListener("keydown",Y);let H=()=>{if(z())z.set(!1)};w?.addEventListener("sui-close",H);let I=()=>void jJ(),q=(F)=>{let V=F;if(V.detail?.mode)Q.set(V.detail.mode);if(typeof V.detail?.prefill==="string")G.set(V.detail.prefill);z.set(!0)},R=()=>{Q.set("create"),z.set(!0)};window.addEventListener("ship:session:changed",I);let T=document.querySelector("ship-shell")??document.querySelector("sui-shell");T?.addEventListener("ship:bar:open",q),T?.addEventListener("ship:create:open",R);let N=X["__sui_ctx_sui-command-bar"];if(N)N.endSession=L,N.loadCurrentSession=jJ;return jJ(),()=>{if(document.removeEventListener("keydown",Y),X.removeEventListener("keydown",W),window.removeEventListener("ship:session:changed",I),T?.removeEventListener("ship:bar:open",q),T?.removeEventListener("ship:create:open",R),w)w.removeEventListener("sui-close",H);ZJ(),c()}}});var qz=BJ("sui-command-bar",{props:{open:$J.bool({default:!1,reflect:!0}),mode:$J.string({default:"ai",reflect:!0})},shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null);Q.open=X.open,Q.mode=X.mode,Q.dialogRef=y(null);let G=null;if(cJ&&typeof cJ.setup==="function"){if(J.scope=Q,G=cJ.setup(J),G&&typeof G==="object")Object.assign(Q,G)}let M=(i)=>YJ({anchor:i,test:()=>{if(Q.isChat())return 0;return 1},branches:[()=>{let WJ=document.createDocumentFragment(),v=[],n=document.createElement("sui-command-chat");return WJ.appendChild(n),{nodes:Array.from(WJ.childNodes),cleanup:()=>{for(let o of v)try{o()}catch(_){}}}},()=>{let WJ=document.createDocumentFragment(),v=[],n=document.createElement("sui-command-palette");return WJ.appendChild(n),{nodes:Array.from(WJ.childNodes),cleanup:()=>{for(let o of v)try{o()}catch(_){}}}}]}),K=Z.shadowRoot??Z;IJ(K);let B=document.createElement("button");B.setAttribute("class","cb-trigger"),B.setAttribute("type","button"),B.setAttribute("aria-label","Open command bar"),z.push(e(B,"click",function(){let i=Q.openFind;return typeof i==="function"?i.bind(void 0):()=>{}}(),{modifiers:[]}));let D=document.createElement("sui-icon");D.setAttribute("name","search"),D.setAttribute("size","sm"),B.appendChild(D);let x=document.createElement("span");x.setAttribute("class","cb-label");let f=document.createTextNode("");x.appendChild(f),z.push(zJ(f,()=>Q.triggerLabel())),B.appendChild(x);let O=document.createElement("span");O.setAttribute("class","cb-dot"),z.push(h(()=>{let i=Q.dotBg();if(i==null)O.style.removeProperty("background");else O.style.setProperty("background",String(i))})),B.appendChild(O);let k=document.createElement("kbd"),p=document.createTextNode("⌘K");k.appendChild(p),B.appendChild(k),K.appendChild(B);let d=document.createElement("sui-dialog");d.setAttribute("class","cb-dialog"),d.setAttribute("no-header",""),d.setAttribute("no-footer",""),d.setAttribute("no-close-button",""),d.setAttribute("backdrop","blur"),d.setAttribute("size","full"),d.setAttribute("style","--dlg-max-w: min(840px, 94vw)"),J0(Q,"dialogRef",d);let a=document.createElement("div");a.setAttribute("class","cb-wrap");let XJ=document.createComment("if");if(a.appendChild(XJ),z.push(M(XJ)),d.appendChild(a),K.appendChild(d),G!=null){let i=Object.getOwnPropertyDescriptor(G,"__shipSetLifecycle")?.value;if(i){let WJ=new Proxy(Q,{get(v,n){return v[n]},set(v,n,o){let _=v[n];if(_!=null&&typeof _.set==="function"){if(typeof o==="function")_.set(()=>o);else _.set(o);return!0}return!1}});if(i.mount){let v=i.mount(WJ,Z);if(typeof v==="function")z.push(v)}if(i.unmount)z.push(()=>i.unmount(WJ))}}return Q}});var rJ=NJ({scope:(J,X)=>{let Z=yJ(X,"sui-command-bar"),z=JJ(()=>{if(Z.session()?.status==="thinking")return"var(--warning,#d97706)";return"var(--success,#16a34a)"}),Q=JJ(()=>Z.allSessions().filter((D)=>D.status!=="ended"&&D.agent_id));function G(D){if(fetch(`/api/v1/sessions/${D}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}),Z.session()?.id===D)Z.session.set(null);Z.sessionPickerOpen.set(!1)}function M(D){let x=Z.allSessions().find((f)=>f.id===D);if(x)Z.session.set(x),Z.thread.set([]);Z.sessionPickerOpen.set(!1)}function K(){Z.sessionPickerOpen.set(!1),Z.mode.set("ai"),Z.session.set(null)}function B(){Z.sessionPickerOpen.set(!1),DJ("/chats"),Z.open.set(!1)}return{bar:Z,headerDotBg:z,activeSessions:Q,endSession:G,selectSession:M,startNewSession:K,viewChats:B}},on:{toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},sendMessage:async(J)=>{let X=J.bar.value().trim();if(!X||!J.bar.session())return;let Z=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:X}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${Z}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:X})})}catch{}},onComposerKeydown:async(J,X)=>{if(X.key==="Enter"&&!X.shiftKey){X.preventDefault();let Z=J.bar.value().trim();if(!Z||!J.bar.session())return;let z=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:Z}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${z}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:Z})})}catch{}}}}});var Uz=BJ("sui-command-chat",{shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null),G=null;if(rJ&&typeof rJ.setup==="function"){if(J.scope=Q,G=rJ.setup(J),G&&typeof G==="object")Object.assign(Q,G)}let M=(g)=>FJ({anchor:g,items:()=>Q.activeSessions(),key:(l,m)=>{let S=Object.create(Q);return S.s=l,S.__i=m,((b)=>l.id)(S)},render:(l,m)=>{let S=Object.create(Q);return S.s=l,S.__i=m,S.__v=y(0),((b)=>{let C=document.createDocumentFragment(),L=[],j=document.createElement("div");j.setAttribute("class","cb-session-row"),L.push(EJ(j,"current",()=>(b.__v(),b.bar.session()?.id===b.s.id))),j.setAttribute("role","option"),L.push(e(j,"click",function(){let A=()=>b.selectSession(b.s.id);return typeof A==="function"?A.bind(void 0):()=>{}}(),{modifiers:[]}));let Y=document.createElement("span");Y.setAttribute("class","cb-dot"),L.push(h(()=>{b.__v();let A=b.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(A==null)Y.style.removeProperty("background");else Y.style.setProperty("background",String(A))})),j.appendChild(Y);let W=document.createElement("span"),H=document.createElement("strong"),I=document.createTextNode("");H.appendChild(I),L.push(zJ(I,()=>(b.__v(),b.s.agent_id??""))),W.appendChild(H);let q=document.createElement("span");q.setAttribute("class","meta");let R=document.createTextNode(" · ");q.appendChild(R);let T=document.createTextNode("");q.appendChild(T),L.push(zJ(T,()=>(b.__v(),b.s.provider??"")));let $=document.createTextNode(" · ");q.appendChild($);let N=document.createTextNode("");q.appendChild(N),N.textContent=b.s.workspace_id,W.appendChild(q),j.appendChild(W);let F=document.createElement("button");F.setAttribute("class","cb-session-end"),F.setAttribute("type","button"),F.setAttribute("aria-label","End session"),L.push(e(F,"click",function(){let A=(P)=>{P.stopPropagation(),b.endSession(b.s.id)};return typeof A==="function"?A.bind(void 0):()=>{}}(),{modifiers:[]}));let V=document.createTextNode("×");return F.appendChild(V),j.appendChild(F),C.appendChild(j),{nodes:Array.from(C.childNodes),cleanup:()=>{for(let A of L)try{A()}catch(P){}},update:(A,P)=>{b.s=A,b.__i=P,N.textContent=b.s.workspace_id,b.__v.set((QJ)=>QJ+1)}}})(S)}}),K=(g)=>YJ({anchor:g,test:()=>{if(Q.activeSessions().length===0)return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),m=[],S=document.createElement("div");S.setAttribute("class","cb-session-row");let b=document.createElement("span");S.appendChild(b);let C=document.createElement("span");C.setAttribute("class","meta");let L=document.createTextNode("No active sessions");C.appendChild(L),S.appendChild(C);let j=document.createElement("span");return S.appendChild(j),l.appendChild(S),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let Y of m)try{Y()}catch(W){}}}}]}),B=(g)=>YJ({anchor:g,test:()=>{if(Q.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let l=document.createDocumentFragment(),m=[],S=document.createElement("div");S.setAttribute("class","cb-session-popover"),S.setAttribute("role","listbox"),S.setAttribute("aria-label","Sessions");let b=document.createComment("for");S.appendChild(b),m.push(M(b));let C=document.createComment("if");S.appendChild(C),m.push(K(C));let L=document.createElement("div");L.setAttribute("class","cb-session-sep"),L.setAttribute("role","separator"),S.appendChild(L);let j=document.createElement("div");j.setAttribute("class","cb-session-action"),m.push(e(j,"click",function(){let $=Q.startNewSession;return typeof $==="function"?$.bind(void 0):()=>{}}(),{modifiers:[]}));let Y=document.createElement("sui-icon");Y.setAttribute("name","plus"),Y.setAttribute("size","sm"),Y.setAttribute("aria-hidden","true"),j.appendChild(Y);let W=document.createElement("span"),H=document.createTextNode("Start new session");W.appendChild(H),j.appendChild(W),S.appendChild(j);let I=document.createElement("div");I.setAttribute("class","cb-session-action"),m.push(e(I,"click",function(){let $=Q.viewChats;return typeof $==="function"?$.bind(void 0):()=>{}}(),{modifiers:[]}));let q=document.createElement("sui-icon");q.setAttribute("name","columns"),q.setAttribute("size","sm"),q.setAttribute("aria-hidden","true"),I.appendChild(q);let R=document.createElement("span"),T=document.createTextNode("View all chats");return R.appendChild(T),I.appendChild(R),S.appendChild(I),l.appendChild(S),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let $ of m)try{$()}catch(N){}}}}]}),D=(g)=>FJ({anchor:g,items:()=>Q.bar.thread(),key:(l,m)=>{let S=Object.create(Q);return S.msg=l,S.__i=m,((b)=>l.id)(S)},render:(l,m)=>{let S=Object.create(Q);return S.msg=l,S.__i=m,S.__v=y(0),((b)=>{let C=document.createDocumentFragment(),L=[],j=document.createElement("div");j.setAttribute("class","cb-chat-turn");{let H=b.msg.sender;if(H==null||H===!1||H==="")j.removeAttribute("data-sender");else j.setAttribute("data-sender",H===!0?"":String(H))}let Y=document.createElement("div");Y.setAttribute("class","cb-chat-bubble");let W=document.createTextNode("");return Y.appendChild(W),W.textContent=b.msg.text,j.appendChild(Y),C.appendChild(j),{nodes:Array.from(C.childNodes),cleanup:()=>{for(let H of L)try{H()}catch(I){}},update:(H,I)=>{b.msg=H,b.__i=I;{let q=b.msg.sender;if(q==null||q===!1||q==="")j.removeAttribute("data-sender");else j.setAttribute("data-sender",q===!0?"":String(q))}W.textContent=b.msg.text,b.__v.set((q)=>q+1)}}})(S)}}),x=(g)=>YJ({anchor:g,test:()=>{if(Q.bar.thread().length===0)return 0;return 1},branches:[()=>{let l=document.createDocumentFragment(),m=[],S=document.createElement("div");S.setAttribute("class","cb-chat-empty");let b=document.createTextNode("Type below and press Enter. The agent's replies land here.");return S.appendChild(b),l.appendChild(S),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let C of m)try{C()}catch(L){}}}},()=>{let l=document.createDocumentFragment(),m=[],S=document.createComment("for");return l.appendChild(S),m.push(D(S)),{nodes:Array.from(l.childNodes),cleanup:()=>{for(let b of m)try{b()}catch(C){}}}}]}),f=Z.shadowRoot??Z;IJ(f);let O=document.createElement("div");O.setAttribute("class","cb-chat-header");let k=document.createElement("span");k.setAttribute("class","cb-dot"),z.push(h(()=>{let g=Q.headerDotBg();if(g==null)k.style.removeProperty("background");else k.style.setProperty("background",String(g))})),k.setAttribute("aria-hidden","true"),O.appendChild(k);let p=document.createElement("div");p.setAttribute("style","display:grid;line-height:1.25");let d=document.createElement("span");d.setAttribute("class","title");let a=document.createTextNode("");d.appendChild(a),z.push(zJ(a,()=>Q.bar.session()?.agent_id??"")),p.appendChild(d);let XJ=document.createElement("span");XJ.setAttribute("class","sub");let i=document.createTextNode("");XJ.appendChild(i),z.push(zJ(i,()=>Q.bar.session()?.provider??"—"));let WJ=document.createTextNode(" · ");XJ.appendChild(WJ);let v=document.createTextNode("");XJ.appendChild(v),z.push(zJ(v,()=>Q.bar.session()?.workspace_id??"")),p.appendChild(XJ),O.appendChild(p);let n=document.createElement("span");n.setAttribute("style","flex:1"),O.appendChild(n);let o=document.createElement("div");o.setAttribute("class","cb-chip-wrap");let _=document.createElement("span");_.setAttribute("class","cb-session-chip"),_.setAttribute("role","button"),_.setAttribute("aria-haspopup","listbox"),z.push(e(_,"click",function(){let g=Q.toggleSessionPicker;return typeof g==="function"?g.bind(void 0):()=>{}}(),{modifiers:[]}));let w=document.createTextNode("Switch");_.appendChild(w),o.appendChild(_);let U=document.createComment("if");o.appendChild(U),z.push(B(U)),O.appendChild(o);let E=document.createElement("kbd");E.setAttribute("class","cb-esc"),E.setAttribute("aria-label","Escape to close");let c=document.createTextNode("esc");E.appendChild(c),O.appendChild(E),f.appendChild(O);let s=document.createElement("div");s.setAttribute("class","cb-chat-messages");let GJ=document.createComment("if");s.appendChild(GJ),z.push(x(GJ)),f.appendChild(s);let ZJ=document.createElement("div");ZJ.setAttribute("class","cb-chat-composer");let jJ=document.createElement("textarea");jJ.setAttribute("class","cb-chat-input"),jJ.setAttribute("rows","2"),jJ.setAttribute("placeholder","Message the agent…"),z.push(HJ(jJ,Q.bar.value)),z.push(e(jJ,"keydown",function(){let g=Q.onComposerKeydown;return typeof g==="function"?g.bind(void 0):()=>{}}(),{modifiers:[]})),ZJ.appendChild(jJ);let LJ=document.createElement("button");LJ.setAttribute("class","cb-chat-send"),LJ.setAttribute("type","button"),z.push(MJ(LJ,"disabled",()=>!Q.bar.value().trim())),z.push(e(LJ,"click",function(){let g=Q.sendMessage;return typeof g==="function"?g.bind(void 0):()=>{}}(),{modifiers:[]}));let UJ=document.createTextNode("Send");if(LJ.appendChild(UJ),ZJ.appendChild(LJ),f.appendChild(ZJ),G!=null){let g=Object.getOwnPropertyDescriptor(G,"__shipSetLifecycle")?.value;if(g){let l=new Proxy(Q,{get(m,S){return m[S]},set(m,S,b){let C=m[S];if(C!=null&&typeof C.set==="function"){if(typeof b==="function")C.set(()=>b);else C.set(b);return!0}return!1}});if(g.mount){let m=g.mount(l,Z);if(typeof m==="function")z.push(m)}if(g.unmount)z.push(()=>g.unmount(l))}}return Q}});var aJ=NJ({scope:(J,X)=>{let Z=yJ(X,"sui-command-bar"),z=JJ(()=>{return{command:"/",find:">",mention:"@",create:"+",ai:"ai"}[Z.mode()]??">"}),Q=JJ(()=>{return{command:"Run a Ship action…",find:"Jump to a view…",mention:"Reference an agent, view, file…",create:"Create a view, skill, agent…",ai:"Ask Ship or type / for actions"}[Z.mode()]??"Search…"}),G=JJ(()=>Z.session()?.agent_id??"No session"),M=JJ(()=>{let w=Z.session();if(w?.status==="thinking")return"var(--warning,#d97706)";if(w)return"var(--success,#16a34a)";return"var(--muted-foreground)"}),K=JJ(()=>Z.allSessions().filter((w)=>w.status!=="ended"&&w.agent_id));function B(w){if(w==="view")return"app-window";if(w==="agent")return"bot";return"message-square-plus"}let D=JJ(()=>{let w=Z.value().replace(/^>+\s*/,"").toLowerCase().trim();return iJ.filter((U)=>!w||U.label.toLowerCase().includes(w)||U.keywords.toLowerCase().includes(w)||U.path.toLowerCase().includes(w)).map((U,E)=>({...U,idx:E}))}),x=JJ(()=>{let w=Z.value().replace(/^\/+\s*/,"").toLowerCase().trim();return nJ.filter((U)=>!w||U.key.includes(w)||U.label.toLowerCase().includes(w)||U.hint.toLowerCase().includes(w)).map((U,E)=>({...U,idx:E}))}),f=JJ(()=>{let w=Z.value().replace(/^@+/,"").toLowerCase().trim();return[...Z.views().map((E)=>({kind:"view",id:E,label:E})),...Z.agents().map((E)=>({kind:"agent",id:E.id,label:E.id})),...Z.allSessions().filter((E)=>E.status!=="ended"&&E.agent_id).map((E)=>({kind:"session",id:E.id,label:`${E.agent_id} (session)`}))].filter((E)=>!w||E.id.toLowerCase().includes(w)).slice(0,40).map((E,c)=>({...E,idx:c,icon:B(E.kind)}))}),O=JJ(()=>pJ),k=JJ(()=>pJ.find((w)=>w.id===Z.createSurface())),p=JJ(()=>k0(Z.createName()));function d(w){fetch(`/api/v1/sessions/${w}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}).then(()=>{if(Z.session()?.id===w)Z.session.set(null);_()}),Z.sessionPickerOpen.set(!1)}function a(w){let U=Z.allSessions().find((E)=>E.id===w);if(U)Z.session.set(U),Z.thread.set([]);Z.sessionPickerOpen.set(!1)}function XJ(){Z.sessionPickerOpen.set(!1),Z.mode.set("ai"),Z.session.set(null)}function i(){Z.sessionPickerOpen.set(!1),DJ("/chats"),Z.open.set(!1)}let WJ=X;function v(w){WJ?.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:"find",item:{path:w}},bubbles:!0,composed:!0})),DJ(w),Z.value.set(""),Z.open.set(!1)}function n(w){switch(Z.value.set(""),w){case"new":Z.mode.set("create");break;case"chats":DJ("/chats"),Z.open.set(!1);break;case"settings":DJ("/settings"),Z.open.set(!1);break;case"end":if(Z.session())d(Z.session().id);Z.open.set(!1);break;case"close":Z.open.set(!1);break}}function o(w){let U=w.kind==="view"?`@${w.id}`:`@${w.kind}:${w.id}`,E=Z.value();Z.value.set(E.replace(/@[^@\s]*$/,`${U} `)),Z.mode.set(fJ(Z.value(),Z.mode()))}async function _(){try{let w=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(w.ok){let U=await w.json();Z.allSessions.set(U.sessions);let E=U.sessions.find((c)=>c.status!=="ended"&&c.agent_id);if(E&&E.id!==Z.session()?.id)Z.session.set(E),Z.thread.set([]);else if(!E)Z.session.set(null)}}catch{}}return{bar:Z,modeGlyph:z,searchPlaceholder:Q,chipLabel:G,chipDotBg:M,activeSessions:K,findItems:D,cmdItems:x,mentionItems:f,createSurfaces:O,currentSurfaceDef:k,createNameValid:p,endSession:d,selectSession:a,startNewSession:XJ,viewChats:i,activateNavItem:v,activateCmd:n,activateMention:o}},on:{onInput:(J,X)=>{let Z=X.target.value;J.bar.value.set(Z),J.bar.mode.set(fJ(Z,J.bar.mode())),J.bar.selectedIdx.set(0)},toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},selectSurface:(J,X)=>{J.bar.createSurface.set(X);let Z=pJ.find((z)=>z.id===X);if(Z?.shapes[0])J.bar.createShape.set(Z.shapes[0])},spawnSession:async(J)=>{let X=J.bar.spawnAgent(),Z=J.bar.spawnProvider();if(!X||!Z)return;J.bar.spawning.set(!0),J.bar.spawnError.set(null);try{let z=await fetch("/api/v1/sessions",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({agent_id:X,provider:Z})});if(!z.ok){let Q=await z.json().catch(()=>({}));J.bar.spawnError.set(Q.error??`HTTP ${z.status}`)}else J.bar.mode.set("ai")}catch(z){J.bar.spawnError.set(z instanceof Error?z.message:"Unknown error")}finally{J.bar.spawning.set(!1)}},submitCreate:async(J)=>{if(!J.createNameValid()||J.bar.createSubmitting())return;J.bar.createSubmitting.set(!0),J.bar.createError.set(null);try{let X=await fetch("/api/v1/scaffold",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({surface:J.bar.createSurface(),name:J.bar.createName(),shape:J.bar.createShape(),purpose:J.bar.createPurpose()})});if(!X.ok){let Z=await X.json().catch(()=>({}));J.bar.createError.set(Z.error??`HTTP ${X.status}`)}else J.bar.createName.set(""),J.bar.createPurpose.set(""),J.bar.open.set(!1)}catch(X){J.bar.createError.set(X instanceof Error?X.message:"Unknown error")}finally{J.bar.createSubmitting.set(!1)}}}});var Tz=BJ("sui-command-palette",{shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null),G=null;if(aJ&&typeof aJ.setup==="function"){if(J.scope=Q,G=aJ.setup(J),G&&typeof G==="object")Object.assign(Q,G)}let M=(C)=>FJ({anchor:C,items:()=>Q.activeSessions(),key:(L,j)=>{let Y=Object.create(Q);return Y.s=L,Y.__i=j,((W)=>L.id)(Y)},render:(L,j)=>{let Y=Object.create(Q);return Y.s=L,Y.__i=j,Y.__v=y(0),((W)=>{let H=document.createDocumentFragment(),I=[],q=document.createElement("div");q.setAttribute("class","cb-session-row"),I.push(EJ(q,"current",()=>(W.__v(),W.bar.session()?.id===W.s.id))),q.setAttribute("role","option"),I.push(e(q,"click",function(){let u=()=>W.selectSession(W.s.id);return typeof u==="function"?u.bind(void 0):()=>{}}(),{modifiers:[]}));let R=document.createElement("span");R.setAttribute("class","cb-dot"),I.push(h(()=>{W.__v();let u=W.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(u==null)R.style.removeProperty("background");else R.style.setProperty("background",String(u))})),q.appendChild(R);let T=document.createElement("span"),$=document.createElement("strong"),N=document.createTextNode("");$.appendChild(N),I.push(zJ(N,()=>(W.__v(),W.s.agent_id??""))),T.appendChild($);let F=document.createElement("span");F.setAttribute("class","meta");let V=document.createTextNode(" · ");F.appendChild(V);let A=document.createTextNode("");F.appendChild(A),I.push(zJ(A,()=>(W.__v(),W.s.provider??"")));let P=document.createTextNode(" · ");F.appendChild(P);let QJ=document.createTextNode("");F.appendChild(QJ),QJ.textContent=W.s.workspace_id,T.appendChild(F),q.appendChild(T);let r=document.createElement("button");r.setAttribute("class","cb-session-end"),r.setAttribute("type","button"),r.setAttribute("aria-label","End session"),I.push(e(r,"click",function(){let u=(VJ)=>{VJ.stopPropagation(),W.endSession(W.s.id)};return typeof u==="function"?u.bind(void 0):()=>{}}(),{modifiers:[]}));let qJ=document.createTextNode("×");return r.appendChild(qJ),q.appendChild(r),H.appendChild(q),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let u of I)try{u()}catch(VJ){}},update:(u,VJ)=>{W.s=u,W.__i=VJ,QJ.textContent=W.s.workspace_id,W.__v.set((mJ)=>mJ+1)}}})(Y)}}),K=(C)=>YJ({anchor:C,test:()=>{if(Q.activeSessions().length===0)return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-session-row");let W=document.createElement("span");Y.appendChild(W);let H=document.createElement("span");H.setAttribute("class","meta");let I=document.createTextNode("No active sessions");H.appendChild(I),Y.appendChild(H);let q=document.createElement("span");return Y.appendChild(q),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let R of j)try{R()}catch(T){}}}}]}),B=(C)=>YJ({anchor:C,test:()=>{if(Q.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-session-popover"),Y.setAttribute("role","listbox"),Y.setAttribute("aria-label","Sessions");let W=document.createComment("for");Y.appendChild(W),j.push(M(W));let H=document.createComment("if");Y.appendChild(H),j.push(K(H));let I=document.createElement("div");I.setAttribute("class","cb-session-sep"),I.setAttribute("role","separator"),Y.appendChild(I);let q=document.createElement("div");q.setAttribute("class","cb-session-action"),j.push(e(q,"click",function(){let P=Q.startNewSession;return typeof P==="function"?P.bind(void 0):()=>{}}(),{modifiers:[]}));let R=document.createElement("sui-icon");R.setAttribute("name","plus"),R.setAttribute("size","sm"),R.setAttribute("aria-hidden","true"),q.appendChild(R);let T=document.createElement("span"),$=document.createTextNode("Start new session");T.appendChild($),q.appendChild(T),Y.appendChild(q);let N=document.createElement("div");N.setAttribute("class","cb-session-action"),j.push(e(N,"click",function(){let P=Q.viewChats;return typeof P==="function"?P.bind(void 0):()=>{}}(),{modifiers:[]}));let F=document.createElement("sui-icon");F.setAttribute("name","columns"),F.setAttribute("size","sm"),F.setAttribute("aria-hidden","true"),N.appendChild(F);let V=document.createElement("span"),A=document.createTextNode("View all chats");return V.appendChild(A),N.appendChild(V),Y.appendChild(N),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let P of j)try{P()}catch(QJ){}}}}]}),D=(C)=>FJ({anchor:C,items:()=>Q.findItems(),key:(L,j)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=j,((W)=>L.path)(Y)},render:(L,j)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=j,Y.__v=y(0),((W)=>{let H=document.createDocumentFragment(),I=[],q=document.createElement("div");q.setAttribute("class","cb-item"),q.setAttribute("role","option"),I.push(MJ(q,"aria-selected",()=>(W.__v(),W.bar.selectedIdx()===W.item.idx?"true":"false")));{let V=W.item.path;if(V==null||V===!1||V==="")q.removeAttribute("data-path");else q.setAttribute("data-path",V===!0?"":String(V))}I.push(e(q,"click",function(){let V=()=>W.activateNavItem(W.item.path);return typeof V==="function"?V.bind(void 0):()=>{}}(),{modifiers:[]}));let R=document.createElement("sui-icon");{let V=W.item.icon;if(V==null||V===!1||V==="")R.removeAttribute("name");else R.setAttribute("name",V===!0?"":String(V))}R.setAttribute("size","sm"),R.setAttribute("aria-hidden","true"),q.appendChild(R);let T=document.createElement("span"),$=document.createTextNode("");T.appendChild($),$.textContent=W.item.label,q.appendChild(T);let N=document.createElement("span");N.setAttribute("class","hint");let F=document.createTextNode("");return N.appendChild(F),F.textContent=W.item.path,q.appendChild(N),H.appendChild(q),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let V of I)try{V()}catch(A){}},update:(V,A)=>{W.item=V,W.__i=A;{let P=W.item.path;if(P==null||P===!1||P==="")q.removeAttribute("data-path");else q.setAttribute("data-path",P===!0?"":String(P))}{let P=W.item.icon;if(P==null||P===!1||P==="")R.removeAttribute("name");else R.setAttribute("name",P===!0?"":String(P))}$.textContent=W.item.label,F.textContent=W.item.path,W.__v.set((P)=>P+1)}}})(Y)}}),x=(C)=>YJ({anchor:C,test:()=>{if(Q.findItems().length>0)return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-mode-hint");let W=document.createTextNode("Jump to");Y.appendChild(W),L.appendChild(Y);let H=document.createElement("div");H.setAttribute("class","cb-list"),H.setAttribute("role","listbox");let I=document.createComment("for");return H.appendChild(I),j.push(D(I)),L.appendChild(H),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let q of j)try{q()}catch(R){}}}},()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-empty");let W=document.createTextNode("No matches.");return Y.appendChild(W),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of j)try{H()}catch(I){}}}}]}),f=(C)=>FJ({anchor:C,items:()=>Q.cmdItems(),key:(L,j)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=j,((W)=>L.key)(Y)},render:(L,j)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=j,Y.__v=y(0),((W)=>{let H=document.createDocumentFragment(),I=[],q=document.createElement("div");q.setAttribute("class","cb-item"),q.setAttribute("role","option"),I.push(MJ(q,"aria-selected",()=>(W.__v(),W.bar.selectedIdx()===W.item.idx?"true":"false")));{let V=W.item.key;if(V==null||V===!1||V==="")q.removeAttribute("data-cmd");else q.setAttribute("data-cmd",V===!0?"":String(V))}I.push(e(q,"click",function(){let V=()=>W.activateCmd(W.item.key);return typeof V==="function"?V.bind(void 0):()=>{}}(),{modifiers:[]}));let R=document.createElement("span");R.setAttribute("style","font-family:var(--font-mono)");let T=document.createTextNode("");R.appendChild(T),T.textContent=W.item.label,q.appendChild(R);let $=document.createElement("span");q.appendChild($);let N=document.createElement("span");N.setAttribute("class","hint");let F=document.createTextNode("");return N.appendChild(F),F.textContent=W.item.hint,q.appendChild(N),H.appendChild(q),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let V of I)try{V()}catch(A){}},update:(V,A)=>{W.item=V,W.__i=A;{let P=W.item.key;if(P==null||P===!1||P==="")q.removeAttribute("data-cmd");else q.setAttribute("data-cmd",P===!0?"":String(P))}T.textContent=W.item.label,F.textContent=W.item.hint,W.__v.set((P)=>P+1)}}})(Y)}}),O=(C)=>YJ({anchor:C,test:()=>{if(Q.cmdItems().length>0)return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-mode-hint");let W=document.createTextNode("Actions");Y.appendChild(W),L.appendChild(Y);let H=document.createElement("div");H.setAttribute("class","cb-list"),H.setAttribute("role","listbox");let I=document.createComment("for");return H.appendChild(I),j.push(f(I)),L.appendChild(H),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let q of j)try{q()}catch(R){}}}},()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-empty");let W=document.createTextNode("No commands match.");return Y.appendChild(W),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of j)try{H()}catch(I){}}}}]}),k=(C)=>FJ({anchor:C,items:()=>Q.mentionItems(),key:(L,j)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=j,((W)=>L.id+L.kind)(Y)},render:(L,j)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=j,Y.__v=y(0),((W)=>{let H=document.createDocumentFragment(),I=[],q=document.createElement("div");q.setAttribute("class","cb-item"),q.setAttribute("role","option"),I.push(MJ(q,"aria-selected",()=>(W.__v(),W.bar.selectedIdx()===W.item.idx?"true":"false")));{let V=W.item.kind;if(V==null||V===!1||V==="")q.removeAttribute("data-mention-kind");else q.setAttribute("data-mention-kind",V===!0?"":String(V))}{let V=W.item.id;if(V==null||V===!1||V==="")q.removeAttribute("data-mention-id");else q.setAttribute("data-mention-id",V===!0?"":String(V))}I.push(e(q,"click",function(){let V=()=>W.activateMention(W.item);return typeof V==="function"?V.bind(void 0):()=>{}}(),{modifiers:[]}));let R=document.createElement("sui-icon");{let V=W.item.icon;if(V==null||V===!1||V==="")R.removeAttribute("name");else R.setAttribute("name",V===!0?"":String(V))}R.setAttribute("size","sm"),R.setAttribute("aria-hidden","true"),q.appendChild(R);let T=document.createElement("span"),$=document.createTextNode("");T.appendChild($),$.textContent=W.item.label,q.appendChild(T);let N=document.createElement("span");N.setAttribute("class","hint");let F=document.createTextNode("");return N.appendChild(F),F.textContent=W.item.kind,q.appendChild(N),H.appendChild(q),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let V of I)try{V()}catch(A){}},update:(V,A)=>{W.item=V,W.__i=A;{let P=W.item.kind;if(P==null||P===!1||P==="")q.removeAttribute("data-mention-kind");else q.setAttribute("data-mention-kind",P===!0?"":String(P))}{let P=W.item.id;if(P==null||P===!1||P==="")q.removeAttribute("data-mention-id");else q.setAttribute("data-mention-id",P===!0?"":String(P))}{let P=W.item.icon;if(P==null||P===!1||P==="")R.removeAttribute("name");else R.setAttribute("name",P===!0?"":String(P))}$.textContent=W.item.label,F.textContent=W.item.kind,W.__v.set((P)=>P+1)}}})(Y)}}),p=(C)=>YJ({anchor:C,test:()=>{if(Q.mentionItems().length>0)return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-mode-hint");let W=document.createTextNode("Reference");Y.appendChild(W),L.appendChild(Y);let H=document.createElement("div");H.setAttribute("class","cb-list"),H.setAttribute("role","listbox");let I=document.createComment("for");return H.appendChild(I),j.push(k(I)),L.appendChild(H),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let q of j)try{q()}catch(R){}}}},()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-empty");let W=document.createTextNode("No matching views or agents.");return Y.appendChild(W),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of j)try{H()}catch(I){}}}}]}),d=(C)=>FJ({anchor:C,items:()=>Q.createSurfaces(),key:(L,j)=>{let Y=Object.create(Q);return Y.surf=L,Y.__i=j,((W)=>L.id)(Y)},render:(L,j)=>{let Y=Object.create(Q);return Y.surf=L,Y.__i=j,Y.__v=y(0),((W)=>{let H=document.createDocumentFragment(),I=[],q=document.createElement("button");q.setAttribute("class","cb-surface-btn"),q.setAttribute("type","button"),I.push(MJ(q,"aria-pressed",()=>(W.__v(),W.bar.createSurface()===W.surf.id?"true":"false"))),I.push(e(q,"click",function(){let N=()=>W.selectSurface(W.surf.id);return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]}));let R=document.createElement("sui-icon");{let N=W.surf.icon;if(N==null||N===!1||N==="")R.removeAttribute("name");else R.setAttribute("name",N===!0?"":String(N))}R.setAttribute("aria-hidden","true"),q.appendChild(R);let T=document.createElement("span"),$=document.createTextNode("");return T.appendChild($),$.textContent=W.surf.label,q.appendChild(T),H.appendChild(q),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let N of I)try{N()}catch(F){}},update:(N,F)=>{W.surf=N,W.__i=F;{let V=W.surf.icon;if(V==null||V===!1||V==="")R.removeAttribute("name");else R.setAttribute("name",V===!0?"":String(V))}$.textContent=W.surf.label,W.__v.set((V)=>V+1)}}})(Y)}}),a=(C)=>FJ({anchor:C,items:()=>Q.currentSurfaceDef().shapes,key:(L,j)=>{let Y=Object.create(Q);return Y.sh=L,Y.__i=j,((W)=>L)(Y)},render:(L,j)=>{let Y=Object.create(Q);return Y.sh=L,Y.__i=j,Y.__v=y(0),((W)=>{let H=document.createDocumentFragment(),I=[],q=document.createElement("option");{let T=W.sh;if(T==null||T===!1||T==="")q.removeAttribute("value");else q.setAttribute("value",T===!0?"":String(T))}let R=document.createTextNode("");return q.appendChild(R),R.textContent=W.sh,H.appendChild(q),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let T of I)try{T()}catch($){}},update:(T,$)=>{W.sh=T,W.__i=$;{let N=W.sh;if(N==null||N===!1||N==="")q.removeAttribute("value");else q.setAttribute("value",N===!0?"":String(N))}R.textContent=W.sh,W.__v.set((N)=>N+1)}}})(Y)}}),XJ=(C)=>YJ({anchor:C,test:()=>{if(Q.bar.createError())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-create-error");let W=document.createTextNode("");return Y.appendChild(W),j.push(zJ(W,()=>Q.bar.createError())),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of j)try{H()}catch(I){}}}}]}),i=(C)=>FJ({anchor:C,items:()=>Q.bar.agents(),key:(L,j)=>{let Y=Object.create(Q);return Y.a=L,Y.__i=j,((W)=>L.id)(Y)},render:(L,j)=>{let Y=Object.create(Q);return Y.a=L,Y.__i=j,Y.__v=y(0),((W)=>{let H=document.createDocumentFragment(),I=[],q=document.createElement("option");{let T=W.a.id;if(T==null||T===!1||T==="")q.removeAttribute("value");else q.setAttribute("value",T===!0?"":String(T))}let R=document.createTextNode("");return q.appendChild(R),I.push(zJ(R,()=>(W.__v(),W.a.name??W.a.label??W.a.id))),H.appendChild(q),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let T of I)try{T()}catch($){}},update:(T,$)=>{W.a=T,W.__i=$;{let N=W.a.id;if(N==null||N===!1||N==="")q.removeAttribute("value");else q.setAttribute("value",N===!0?"":String(N))}W.__v.set((N)=>N+1)}}})(Y)}}),WJ=(C)=>YJ({anchor:C,test:()=>{if(Q.bar.agents().length===0)return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("option");Y.setAttribute("value","");let W=document.createTextNode("No agents found");return Y.appendChild(W),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of j)try{H()}catch(I){}}}},()=>{let L=document.createDocumentFragment(),j=[],Y=document.createComment("for");return L.appendChild(Y),j.push(i(Y)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let W of j)try{W()}catch(H){}}}}]}),v=(C)=>FJ({anchor:C,items:()=>Q.bar.providers(),key:(L,j)=>{let Y=Object.create(Q);return Y.p=L,Y.__i=j,((W)=>L)(Y)},render:(L,j)=>{let Y=Object.create(Q);return Y.p=L,Y.__i=j,Y.__v=y(0),((W)=>{let H=document.createDocumentFragment(),I=[],q=document.createElement("option");{let T=W.p;if(T==null||T===!1||T==="")q.removeAttribute("value");else q.setAttribute("value",T===!0?"":String(T))}let R=document.createTextNode("");return q.appendChild(R),R.textContent=W.p,H.appendChild(q),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let T of I)try{T()}catch($){}},update:(T,$)=>{W.p=T,W.__i=$;{let N=W.p;if(N==null||N===!1||N==="")q.removeAttribute("value");else q.setAttribute("value",N===!0?"":String(N))}R.textContent=W.p,W.__v.set((N)=>N+1)}}})(Y)}}),n=(C)=>YJ({anchor:C,test:()=>{if(Q.bar.providers().length===0)return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("option");Y.setAttribute("value","");let W=document.createTextNode("No providers installed");return Y.appendChild(W),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of j)try{H()}catch(I){}}}},()=>{let L=document.createDocumentFragment(),j=[],Y=document.createComment("for");return L.appendChild(Y),j.push(v(Y)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let W of j)try{W()}catch(H){}}}}]}),o=(C)=>YJ({anchor:C,test:()=>{if(Q.bar.spawnError())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-spawn-error");let W=document.createTextNode("");return Y.appendChild(W),j.push(zJ(W,()=>Q.bar.spawnError())),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of j)try{H()}catch(I){}}}}]}),_=(C)=>YJ({anchor:C,test:()=>{let L=Q.bar.mode();if(L==="find")return 0;if(L==="command")return 1;if(L==="mention")return 2;if(L==="create")return 3;return 4},branches:[()=>{let L=document.createDocumentFragment(),j=[],Y=document.createComment("if");return L.appendChild(Y),j.push(x(Y)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let W of j)try{W()}catch(H){}}}},()=>{let L=document.createDocumentFragment(),j=[],Y=document.createComment("if");return L.appendChild(Y),j.push(O(Y)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let W of j)try{W()}catch(H){}}}},()=>{let L=document.createDocumentFragment(),j=[],Y=document.createComment("if");return L.appendChild(Y),j.push(p(Y)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let W of j)try{W()}catch(H){}}}},()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-mode-hint");let W=document.createTextNode("Create");Y.appendChild(W),L.appendChild(Y);let H=document.createElement("div");H.setAttribute("class","cb-create-surfaces"),H.setAttribute("role","group"),H.setAttribute("aria-label","Surface type");let I=document.createComment("for");H.appendChild(I),j.push(d(I)),L.appendChild(H);let q=document.createElement("div");q.setAttribute("class","cb-create-grid");let R=document.createElement("label"),T=document.createElement("span"),$=document.createTextNode("Name (kebab-case)");T.appendChild($),R.appendChild(T);let N=document.createElement("input");N.setAttribute("type","text"),N.setAttribute("class","cb-create-name"),j.push(MJ(N,"placeholder",()=>"my-"+Q.bar.createSurface())),j.push(HJ(N,Q.bar.createName)),N.setAttribute("autocomplete","off"),R.appendChild(N),q.appendChild(R);let F=document.createElement("label"),V=document.createElement("span"),A=document.createTextNode("");V.appendChild(A),j.push(zJ(A,()=>Q.currentSurfaceDef().shapeLabel)),F.appendChild(V);let P=document.createElement("select");P.setAttribute("class","cb-create-shape"),j.push(HJ(P,Q.bar.createShape));let QJ=document.createComment("for");P.appendChild(QJ),j.push(a(QJ)),F.appendChild(P),q.appendChild(F);let r=document.createElement("label");r.setAttribute("class","span-2");let qJ=document.createElement("span"),u=document.createTextNode("Purpose (optional)");qJ.appendChild(u),r.appendChild(qJ);let VJ=document.createElement("input");VJ.setAttribute("type","text"),VJ.setAttribute("class","cb-create-purpose"),VJ.setAttribute("placeholder","One-line description"),j.push(HJ(VJ,Q.bar.createPurpose)),VJ.setAttribute("autocomplete","off"),r.appendChild(VJ),q.appendChild(r),L.appendChild(q);let mJ=document.createComment("if");L.appendChild(mJ),j.push(XJ(mJ));let oJ=document.createElement("div");oJ.setAttribute("class","cb-create-actions");let CJ=document.createElement("button");CJ.setAttribute("class","cb-create-submit"),CJ.setAttribute("type","button"),j.push(MJ(CJ,"disabled",()=>Q.bar.createSubmitting()||!Q.createNameValid())),j.push(e(CJ,"click",function(){let gJ=Q.submitCreate;return typeof gJ==="function"?gJ.bind(void 0):()=>{}}(),{modifiers:[]}));let q0=document.createTextNode("");return CJ.appendChild(q0),j.push(zJ(q0,()=>Q.bar.createSubmitting()?"Creating…":"Create "+Q.currentSurfaceDef().label)),oJ.appendChild(CJ),L.appendChild(oJ),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let gJ of j)try{gJ()}catch(GX){}}}},()=>{let L=document.createDocumentFragment(),j=[],Y=document.createElement("div");Y.setAttribute("class","cb-spawn-card");let W=document.createElement("div");W.setAttribute("class","cb-spawn-title");let H=document.createTextNode("Start a session");W.appendChild(H),Y.appendChild(W);let I=document.createElement("div");I.setAttribute("class","cb-spawn-row");let q=document.createElement("label"),R=document.createTextNode("Agent");q.appendChild(R),I.appendChild(q);let T=document.createElement("select");T.setAttribute("class","cb-spawn-agent"),j.push(HJ(T,Q.bar.spawnAgent));let $=document.createComment("if");T.appendChild($),j.push(WJ($)),I.appendChild(T),Y.appendChild(I);let N=document.createElement("div");N.setAttribute("class","cb-spawn-row");let F=document.createElement("label"),V=document.createTextNode("Provider");F.appendChild(V),N.appendChild(F);let A=document.createElement("select");A.setAttribute("class","cb-spawn-provider"),j.push(HJ(A,Q.bar.spawnProvider));let P=document.createComment("if");A.appendChild(P),j.push(n(P)),N.appendChild(A),Y.appendChild(N);let QJ=document.createComment("if");Y.appendChild(QJ),j.push(o(QJ));let r=document.createElement("button");r.setAttribute("class","cb-spawn-start"),r.setAttribute("type","button"),j.push(MJ(r,"disabled",()=>Q.bar.spawning()||!Q.bar.spawnAgent()||!Q.bar.spawnProvider())),j.push(e(r,"click",function(){let u=Q.spawnSession;return typeof u==="function"?u.bind(void 0):()=>{}}(),{modifiers:[]}));let qJ=document.createTextNode("");return r.appendChild(qJ),j.push(zJ(qJ,()=>Q.bar.spawning()?"Starting…":"Start session")),Y.appendChild(r),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let u of j)try{u()}catch(VJ){}}}}]}),w=Z.shadowRoot??Z;IJ(w);let U=document.createElement("div");U.setAttribute("class","cb-input-row");let E=document.createElement("span");E.setAttribute("class","cb-mode-glyph"),z.push(EJ(E,"ai",()=>Q.bar.mode()==="ai"));let c=document.createTextNode("");E.appendChild(c),z.push(zJ(c,()=>Q.modeGlyph())),U.appendChild(E);let s=document.createElement("input");s.setAttribute("class","cb-search"),s.setAttribute("type","text"),s.setAttribute("autocomplete","off"),s.setAttribute("spellcheck","false"),z.push(MJ(s,"placeholder",()=>Q.searchPlaceholder())),z.push(HJ(s,Q.bar.value)),s.setAttribute("aria-label","Command bar search"),s.setAttribute("aria-autocomplete","list"),z.push(e(s,"input",function(){let C=Q.onInput;return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]})),U.appendChild(s);let GJ=document.createElement("kbd");GJ.setAttribute("class","cb-esc"),GJ.setAttribute("aria-label","Escape to close");let ZJ=document.createTextNode("esc");GJ.appendChild(ZJ),U.appendChild(GJ);let jJ=document.createElement("div");jJ.setAttribute("class","cb-chip-wrap");let LJ=document.createElement("span");LJ.setAttribute("class","cb-session-chip"),LJ.setAttribute("role","button"),LJ.setAttribute("aria-haspopup","listbox"),z.push(e(LJ,"click",function(){let C=Q.toggleSessionPicker;return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let UJ=document.createElement("span");UJ.setAttribute("class","cb-dot"),z.push(h(()=>{let C=Q.chipDotBg();if(C==null)UJ.style.removeProperty("background");else UJ.style.setProperty("background",String(C))})),LJ.appendChild(UJ);let g=document.createElement("span"),l=document.createTextNode("");g.appendChild(l),z.push(zJ(l,()=>Q.chipLabel())),LJ.appendChild(g),jJ.appendChild(LJ);let m=document.createComment("if");jJ.appendChild(m),z.push(B(m)),U.appendChild(jJ),w.appendChild(U);let S=document.createElement("div");S.setAttribute("class","cb-body");let b=document.createComment("match");if(S.appendChild(b),z.push(_(b)),w.appendChild(S),G!=null){let C=Object.getOwnPropertyDescriptor(G,"__shipSetLifecycle")?.value;if(C){let L=new Proxy(Q,{get(j,Y){return j[Y]},set(j,Y,W){let H=j[Y];if(H!=null&&typeof H.set==="function"){if(typeof W==="function")H.set(()=>W);else H.set(W);return!0}return!1}});if(C.mount){let j=C.mount(L,Z);if(typeof j==="function")z.push(j)}if(C.unmount)z.push(()=>C.unmount(L))}}return Q}});
