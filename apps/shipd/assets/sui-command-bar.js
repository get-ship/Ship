var S0=Symbol.for("sui:error-boundary");function OJ(J){let X=J;while(X){if(X instanceof Element){let Q=X[S0];if(Q)return Q}let Z=X.parentNode;if(Z){X=Z;continue}let z=X;if(z.host){X=z.host;continue}X=null}return null}function TJ(J,X,Z){try{return Z()}catch(z){let Q=OJ(J);if(Q){Q.catch(z,X);return}console.error("[sui] uncaught throw (no error boundary):",z);return}}var PJ=null,KJ=null,y0=0,b0=new Set,wJ=[];function sJ(J,X){wJ.push(J);try{return X()}finally{wJ.pop()}}function $0(){return wJ.length>0?wJ[wJ.length-1]:null}function q0(J){if(PJ===null)return;if(J.has(PJ))return;let X=PJ;if(J.add(X),KJ!==null)KJ.push(()=>J.delete(X))}function y(J){let X=J,Z=new Set,z=()=>{return q0(Z),X};return z.set=(Q)=>{let G=typeof Q==="function"?Q(X):Q;if(Object.is(G,X))return;if(X=G,y0>0)for(let q of Z)b0.add(q);else for(let q of[...Z])q()},z.__subs=Z,z.__peek=()=>X,z}function ZJ(J){let X,Z=!0,z=new Set,Q=[],G=()=>{if(Z)return;Z=!0;for(let B of[...z])B()},q=()=>{for(let A of Q)A();Q=[];let B=PJ,D=KJ;PJ=G,KJ=Q;try{X=J()}finally{PJ=B,KJ=D}Z=!1},K=()=>{if(q0(z),Z)q();return X};return K.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},K.__subs=z,K.__peek=()=>{if(Z)q();return X},K}function l(J){let X=void 0,Z=!0,z=[],Q=$0(),G=()=>{if(!Z)return;if(X){try{X()}catch(B){console.error("[sui] effect cleanup threw:",B)}X=void 0}for(let B of z)B();z=[];let q=PJ,K=KJ;PJ=G,KJ=z;try{X=J()}catch(B){let D=Q?OJ(Q):null;if(D)D.catch(B,"effect");else console.error("[sui] effect threw:",B)}finally{PJ=q,KJ=K}};return G(),()=>{Z=!1;for(let q of z)q();if(z=[],X){try{X()}catch(q){console.error("[sui] effect cleanup threw:",q)}X=void 0}}}var gJ=null;function kJ(J,X){let Z=gJ;gJ=J;try{return X()}finally{gJ=Z}}function xJ(J){let X=gJ;if(!X)return;X.cleanup(J)}function SJ(){let J=[],X=!1;return{cleanup(z){if(X){try{z()}catch(Q){console.error("[sui/scope] late cleanup threw:",Q)}return}J.push(z)},dispose(){if(X)return;X=!0;for(let z=J.length-1;z>=0;z--)try{J[z]()}catch(Q){console.error("[sui/scope] cleanup threw:",Q)}J.length=0},get disposed(){return X}}}var V0=Symbol("sui.shared");function e(J){return J[V0]=!0,J}function M0(J){return!!J?.[V0]}function _J(J,X,Z){J[`__sui_ctx_${X}`]=Z}function yJ(J,X){let Z=J;while(Z){let z=Z[`__sui_ctx_${X}`];if(z!==void 0)return z;let Q=Z.parentElement;if(Q){Z=Q;continue}let G=Z.getRootNode();Z=G instanceof ShadowRoot?G.host:null}return null}function tJ(J){let X={};for(let Z in J)if(Object.prototype.hasOwnProperty.call(J,Z)&&M0(J[Z]))X[Z]=J[Z];return X}function eJ(J=null){return y(null)}function J0(J,X,Z){let z=J[X];if(typeof z==="function"&&typeof z.set==="function")return z.set(Z),z;let Q=y(Z);return J[X]=Q,Q}function g(J,X){let Z;return l(()=>{let z=X(),Q=z==null?"":String(z);if(Q===Z)return;if(Z===void 0&&J.textContent===Q){Z=Q;return}J.textContent=Q,Z=Q})}function JJ(J,X,Z){let z,Q=!1;return l(()=>{let G=Z();if(Q&&G===z)return;let q=G==null||G===!1||G===""?null:G===!0?"":String(G);if(Q||J.getAttribute(X)!==q)if(q===null)J.removeAttribute(X);else J.setAttribute(X,q);z=G,Q=!0})}function EJ(J,X,Z){let z;return l(()=>{let Q=!!Z();if(Q===z)return;if(z===void 0&&Q===J.classList.contains(X)){z=Q;return}if(Q)J.classList.add(X);else J.classList.remove(X);z=Q})}function HJ(J,X,Z){let z=l(()=>{let G=X();if(J.value!==G)J.value=G}),Q=()=>{let G=J.value;if(X.set(G),Z)Z(G)};return J.addEventListener("input",Q),()=>{z(),J.removeEventListener("input",Q)}}function XJ(J,X,Z,z={}){let Q=z.modifiers??[],G,q=!0,K=(D)=>{for(let O of Q)switch(O.kind){case"preventDefault":D.preventDefault();break;case"stopPropagation":D.stopPropagation();break;case"stopImmediatePropagation":D.stopImmediatePropagation();break;case"self":if(D.target!==J)return;break;case"trusted":if(!D.isTrusted)return;break;case"key":{if(!D0(D,O.arg??""))return;break}}let A=Q.find((O)=>O.kind==="debounce"),$=Q.find((O)=>O.kind==="throttle");if(A){let O=Number(A.arg)||0;if(G!==void 0)clearTimeout(G);G=setTimeout(()=>{G=void 0,Z(D)},O);return}if($){let O=Number($.arg)||0;if(!q)return;q=!1,Z(D),G=setTimeout(()=>{q=!0},O);return}Z(D)},B={};if(z.capture)B.capture=!0;if(z.passive)B.passive=!0;if(z.once)B.once=!0;return J.addEventListener(X,K,B),()=>{if(J.removeEventListener(X,K,B),G!==void 0)clearTimeout(G)}}function D0(J,X){let Z=X.split("+").map((A)=>A.toLowerCase()),z=Z.pop()??"",Q=Z.includes("meta")||Z.includes("cmd"),G=Z.includes("ctrl")||Z.includes("control"),q=Z.includes("shift"),K=Z.includes("alt")||Z.includes("option"),B=Z.includes("mod"),D=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(Q&&!J.metaKey)return!1;if(G&&!J.ctrlKey)return!1;if(q&&!J.shiftKey)return!1;if(K&&!J.altKey)return!1;if(B&&!(D?J.metaKey:J.ctrlKey))return!1;return J.key.toLowerCase()===z}function H0(J,X,Z){J.dispatchEvent(new CustomEvent(X,{detail:Z,bubbles:!0,composed:!0}))}function P0(J,X){let Z=J,z=Object.keys(X);for(let Q of z)Z[Q]=X[Q];xJ(()=>{for(let Q of z)if(Z[Q]===X[Q])try{delete Z[Q]}catch{Z[Q]=void 0}})}function uJ(J){let X=J;X.emit=function(Z,z){H0(J,Z,z)},X.expose=function(Z){P0(J,Z)}}var f0=Symbol("sui:transition");function hJ(J){return J[f0]}function v0(){if(typeof window>"u"||typeof window.matchMedia!=="function")return!1;try{return window.matchMedia("(prefers-reduced-motion: reduce)").matches}catch{return!1}}function U0(J){if(!v0())return J;return{...J,duration:0,delay:0}}function K0(J){let X=hJ(J);if(!X?.inT?.in)return;X.current?.cancel();let Z=X.inT.in(J,U0(X.inOpts));return X.current=Z,Z}function B0(J){let X=hJ(J);if(!X?.outT?.out)return;X.current?.cancel();let Z=X.outT.out(J,U0(X.outOpts));return X.current=Z,Z}function X0(J){if(J.nodeType!==1)return;let X=J;if(X.hasAttribute("data-sui-transition")){let Z=hJ(X);if(Z?.current?.cancel(),Z)Z.current=void 0}for(let Z of X.querySelectorAll("[data-sui-transition]")){let z=hJ(Z);if(z?.current?.cancel(),z)z.current=void 0}}function O0(J,X){for(let Z of J){if(Z.nodeType!==1)continue;let z=Z;if(z.hasAttribute("data-sui-transition"))X(z);for(let Q of z.querySelectorAll("[data-sui-transition]"))X(Q)}}function RJ(J){O0(J,(X)=>{K0(X)})}function bJ(J){let X=[];if(O0(J,(Z)=>{let z=B0(Z);if(z)X.push(z.finished.catch(()=>{}))}),X.length===0)return null;return Promise.all(X).then(()=>{return})}function m0(J,X,Z,z,Q){if(Z.fill(0,0,X),X===0)return;let G=0;for(let K=0;K<X;K++){let B=J[2*K+1],D=0,A=G;while(D<A){let $=D+A>>>1;if(J[2*Q[$]+1]<B)D=$+1;else A=$}if(z[K]=D>0?Q[D-1]:-1,Q[D]=K,D===G)G++}let q=Q[G-1];while(q>=0)Z[q]=1,q=z[q]}function FJ(J){let{anchor:X,items:Z,key:z,render:Q}=J,G=[],q=!0,K=new Set,B=new Map,D=new Map,A=new Map,$=new Set,O=[],k=[],i=new Int32Array(0),d=new Uint8Array(0),s=new Int32Array(0),QJ=new Int32Array(0),n=(u)=>{for(let a of u.nodes)try{a.parentNode?.removeChild(a)}catch{}u.cleanup?.()},zJ=l(()=>{if(!q)return;let u=Z(),a=u.length;if(i.length<a*2)i=new Int32Array(Math.max(i.length*2,a*2,16));if(d.length<a){let P=Math.max(d.length*2,a,8);d=new Uint8Array(P),s=new Int32Array(P),QJ=new Int32Array(P)}for(let P=0;P<a;P++){let I=u[P],r=z(I,P);if(B.has(r))console.warn("[sui] {#for}: duplicate key",r,"at index",P,"(prior at",B.get(r),"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");B.set(r,P)}for(let P of G)D.set(P.key,P);for(let P of G){if(B.has(P.key))continue;let I=bJ(P.nodes);if(I===null){n(P);continue}K.add(P),I.then(()=>{if(!K.delete(P))return;n(P)})}k.length=0,O.length=0;for(let P=0;P<a;P++){let I=u[P],r=z(I,P),c=D.get(r),VJ;if(c)c.item=I,TJ(X,"render",()=>c.update?.(I,P)),VJ=c;else{let GJ=TJ(X,"render",()=>Q(I,P));if(!GJ)continue;VJ={key:r,item:I,nodes:GJ.nodes,...GJ.cleanup&&{cleanup:GJ.cleanup},...GJ.update&&{update:GJ.update}},O.push(...GJ.nodes)}k.push(VJ)}for(let P=0;P<G.length;P++)A.set(G[P].key,P);let _=0;for(let P=0;P<k.length;P++){let I=A.get(k[P].key);if(I!==void 0)i[2*_]=P,i[2*_+1]=I,_++}m0(i,_,d,s,QJ);for(let P=0;P<_;P++)if(d[P])$.add(i[2*P]);let t=X.parentNode;if(t&&G.length===0&&k.length>0){let P=document.createDocumentFragment();for(let I=0;I<k.length;I++){let r=k[I].nodes;for(let c=0;c<r.length;c++)P.appendChild(r[c])}t.insertBefore(P,X.nextSibling)}else{let P=X;for(let I=0;I<k.length;I++){if(!t)break;let r=k[I];if($.has(I))P=r.nodes[r.nodes.length-1]??P;else for(let c of r.nodes)t.insertBefore(c,P.nextSibling),P=c}}let T=G;if(G=k,k=T,O.length>0)RJ(O);B.clear(),D.clear(),A.clear(),$.clear(),O.length=0});return()=>{q=!1,zJ();for(let u of G)n(u);for(let u of K)n(u);K.clear(),G=[],k.length=0}}function YJ(J){let{anchor:X}=J,{test:Z,branches:z}=g0(J)?{test:J.test,branches:J.branches}:u0(J),Q=null,G=-2,q=null,K=(D)=>{for(let A of D)try{A.parentNode?.removeChild(A)}catch{}},B=l(()=>{let D=Z();if(Q&&D===G)return;if(q&&D===q.idx&&D>=0&&D<z.length){let O=q;q=null;for(let k of O.nodes)X0(k);Q={nodes:O.nodes,...O.cleanup&&{cleanup:O.cleanup}},G=D,RJ(Q.nodes);return}if(Q){let O={idx:G,nodes:Q.nodes,...Q.cleanup&&{cleanup:Q.cleanup}},k=bJ(O.nodes);if(Q=null,k===null)K(O.nodes),O.cleanup?.();else q=O,k.then(()=>{if(q!==O)return;K(O.nodes),O.cleanup?.(),q=null})}if(G=D,D<0||D>=z.length)return;let A=TJ(X,"render",()=>z[D]());if(!A)return;Q=A;let $=X.parentNode;if(!$)return;for(let O of Q.nodes)$.insertBefore(O,X);RJ(Q.nodes)});return()=>{if(B(),Q)K(Q.nodes),Q.cleanup?.(),Q=null;if(q)K(q.nodes),q.cleanup?.(),q=null}}function g0(J){return typeof J.test==="function"}function u0(J){let{cond:X,then_:Z,else_:z}=J;return{test:()=>X()?0:z?1:-1,branches:z?[Z,z]:[Z]}}var Z0=(J)=>J??"",Q0=(J)=>{if(J===null||J==="")return NaN;return Number(J)},z0=(J)=>{if(J===null)return!1;if(J.toLowerCase()==="false")return!1;return!0};function lJ(J){return(X)=>{if(X===null||X==="")return J;try{return JSON.parse(X)}catch{return J}}}function W0(J,X){let Z=new Set(J);return(z)=>z!==null&&Z.has(z)?z:X}var dJ=(J)=>{switch(J.type){case"string":return Z0;case"bool":return z0;case"number":return Q0;case"union":{let X=J.values??[],Z=J.default??X[0]??"";return W0(X,Z)}case"object":return lJ(J.default);case"array":return lJ(J.default??[])}};function I0(J,X,Z,z){if(!X)return;let Q={},G,q=Z.formValue;if(q!==void 0){let{sig:B,propDef:D}=N0(Z,z,q,"formValue"),A=D.default,$=dJ(D);G=l(()=>{let O=B();X.setFormValue(h0(O))}),Q.reset=()=>{B.set(A)},Q.restore=(O)=>{B.set($(O))}}let K=Z.formDisabled;if(K!==void 0){let{sig:B,propDef:D}=N0(Z,z,K,"formDisabled");if(D.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${K}" must be a bool-typed prop`);Q.disabled=(A)=>{B.set(!!A)}}if(J.__shipSetFormHooks=Q,G)J.__shipSetFormValueCleanup=G;return G}function C0(J){let X=J;delete X.__shipSetFormHooks,delete X.__shipSetFormValueCleanup}function h0(J){if(J===null||J===void 0)return"";if(typeof J==="boolean")return J?"on":"";return String(J)}function N0(J,X,Z,z){let Q=J.props[Z];if(!Q)throw Error(`[sui-runtime] ${z}: "${Z}" — no matching prop on this component`);let G=X[AJ(Z)];if(!G)throw Error(`[sui-runtime] ${z}: "${Z}" — prop signal missing`);return{sig:G,propDef:Q}}var $J={string(J={}){return{type:"string",...J}},bool(J={}){return{type:"bool",...J}},number(J={}){return{type:"number",...J}},union(J,X={}){return{type:"union",values:J,...X}},object(J={}){return{type:"object",...J}},array(J={}){return{type:"array",...J}}},l0=(J,X)=>{if(X===void 0||X===null)return null;switch(J.type){case"string":return X===""?null:String(X);case"number":return Number.isNaN(X)?null:String(X);case"bool":return X?"":null;case"union":return String(X);case"object":case"array":try{return JSON.stringify(X)}catch{return null}}},d0=(J,X)=>{let Z=`ship-set-style-${J}`;if(document.getElementById(Z))return;let z=document.createElement("style");z.id=Z,z.textContent=`[data-component="${J}"] { ${X} }`,document.head.appendChild(z)},p0=(J,X,Z,z)=>{let Q=J.getAttribute(X),G=Q!==null?z(Q):Z.default;return y(G)},i0=(J)=>{let X=[],Z={};for(let[G,q]of Object.entries(J.props??{}))X.push(G),Z[G]={camel:AJ(G),decode:dJ(q)};let z=J.formAssociated===!0||J.formValue!==void 0||J.formDisabled!==void 0,Q={observedAttrs:X,props:Z};if(z)Q.formAssociated=!0;return Q};function BJ(J,X){let Z=i0(X),z=Object.entries(X.props??{});Y0(J,Z,(Q,G,q)=>{let K=null;if(X.shadow){if(K=Q.shadowRoot??Q.attachShadow({mode:"open"}),X.styles){let D=document.createElement("style");D.textContent=X.styles,K.appendChild(D)}}else if(X.styles)d0(J,X.styles);let B=[];for(let[D,A]of z){let $=AJ(D),O=Z.props[D].decode,k=p0(Q,D,A,O);if(G[$]=k,A.reflect){let i=!0,d=l(()=>{let s=k();if(i){i=!1;return}let QJ=l0(A,s);if(QJ===null)Q.removeAttribute(D);else Q.setAttribute(D,QJ)});B.push(d)}}if(X.formValue!==void 0||X.formDisabled!==void 0){let D=I0(Q,q,X,G);if(typeof D==="function")B.push(D)}if(X.setup){let D={props:G,host:Q,cleanups:B};if(q)D.internals=q;X.setup(D)}return()=>{C0(Q);for(let D of B.splice(0))try{D()}catch(A){console.error(`[ship-set:${J}] cleanup threw:`,A)}}})}var T0=new Set;function Y0(J,X,Z){if(customElements.get(J)){if(!T0.has(J))T0.add(J),console.warn(`[sui-runtime] customElements.define("${J}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(X.formAssociated){class Q extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return X.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=J,uJ(this),this.scope=SJ();try{this.cleanup=kJ(this.scope,()=>sJ(this,()=>Z(this,this.propSignals,this.internals)))}catch(G){let q=OJ(this.parentNode);if(q)q.catch(G,"mount");else console.error(`[ship-set:${J}] setup threw:`,G)}}disconnectedCallback(){try{this.cleanup?.()}catch(G){console.error(`[ship-set:${J}] cleanup threw:`,G)}this.cleanup=void 0;try{this.scope?.dispose()}catch(G){console.error(`[ship-set:${J}] scope dispose threw:`,G)}this.scope=void 0}attributeChangedCallback(G,q,K){let B=X.props[G];if(!B)return;let D=this.propSignals[B.camel];if(D)D.set(B.decode(K))}formResetCallback(){let G=this.__shipSetFormHooks;try{G?.reset?.()}catch(q){console.error(`[ship-set:${J}] formResetCallback threw:`,q)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(G,q){let K=this.__shipSetFormHooks;try{K?.restore?.(G)}catch(B){console.error(`[ship-set:${J}] formStateRestoreCallback threw:`,B)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:G,mode:q}}))}formDisabledCallback(G){let q=this.__shipSetFormHooks;try{q?.disabled?.(G)}catch(K){console.error(`[ship-set:${J}] formDisabledCallback threw:`,K)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:G}}))}}customElements.define(J,Q);return}class z extends HTMLElement{static get observedAttributes(){return X.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=J,uJ(this),this.scope=SJ();try{this.cleanup=kJ(this.scope,()=>sJ(this,()=>Z(this,this.propSignals)))}catch(Q){let G=OJ(this.parentNode);if(G)G.catch(Q,"mount");else console.error(`[ship-set:${J}] setup threw:`,Q)}}disconnectedCallback(){try{this.cleanup?.()}catch(Q){console.error(`[ship-set:${J}] cleanup threw:`,Q)}this.cleanup=void 0;try{this.scope?.dispose()}catch(Q){console.error(`[ship-set:${J}] scope dispose threw:`,Q)}this.scope=void 0}attributeChangedCallback(Q,G,q){let K=X.props[Q];if(!K)return;let B=this.propSignals[K.camel];if(B)B.set(K.decode(q))}}customElements.define(J,z)}function AJ(J){return J.replace(/-([a-z])/g,(X,Z)=>Z.toUpperCase())}function G0(J,X,Z){let z=a0(J.state,{ship:X.ship,...Z}),Q={};for(let[O,k]of Object.entries(z))Q[O]=y(k);let G={ship:X.ship,route:X.route??{},navigate:X.navigate??((O)=>{}),...Z,...Q},q=r0(Q,G),K={...G},B={};for(let[O,k]of Object.entries(J.derived??{}))B[O]=ZJ(()=>k(K));Object.assign(K,B);let D={};for(let[O,k]of Object.entries(J.on??{}))D[O]=(...i)=>k(q,...i);let A={},$=typeof J.use==="function"?J.use(K):J.use??{};for(let[O,k]of Object.entries($))A[O]=G0(k,X,K);return{...Q,...B,...D,...A}}function a0(J,X){if(J===void 0)return{};if(typeof J==="function")return J(X);return J}function r0(J,X){return new Proxy({},{get(Z,z){if(z in J)return J[z]();return X[z]},set(Z,z,Q){if(z in J)return J[z].set(Q),!0;return console.warn(`[ship-set] hook handler tried to write s.${z} = … but ${z} is not a hook state key. The assignment was ignored.`),!0},has(Z,z){return z in J||z in X},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(Z,z){if(z in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[z]()};return}})}var E0="__shipSetLifecycle";function R0(J,X){let Z=o0(J,X),z={ship:X.ship,route:X.route??{},navigate:X.navigate??((O)=>{}),...X.scope??{}},Q=Z.scope?Z.scope(z,X.host):{};Object.assign(z,Q);let G=s0(z,X.stateKeys,Q),q=_0(G,z),K={...z},B=t0(Z,K);Object.assign(K,B);let D=JX(Z,K,X);for(let[O,k]of Object.entries(D))K[O]=k;let A=e0(Z,q),$={...Q,...B,...A,...D};if(Object.keys(Q).length>0&&X.host){let O=tJ(Q);if(Object.keys(O).length>0){let k=X.host,i=k.tagName.toLowerCase();_J(k,i,O)}}if(Z.mount||Z.unmount){let O={state:q};if(Z.mount)O.mount=Z.mount;if(Z.unmount)O.unmount=Z.unmount;Object.defineProperty($,E0,{value:O,enumerable:!0,configurable:!0,writable:!1})}return $}function o0(J,X){if(typeof J==="function")return J({ship:X.ship});return J}function s0(J,X,Z){let z={},Q=X!==void 0,G=Z?Object.keys(Z):[];if(Q||G.length>0){if(X)for(let q of X){let K=J[q];if(j0(K))z[q]=K}for(let q of G){if(q in z)continue;let K=J[q];if(j0(K))z[q]=K}return z}for(let[q,K]of Object.entries(J))if(j0(K))z[q]=K;return z}function j0(J){return typeof J==="function"&&"set"in J&&typeof J.set==="function"}function _0(J,X){return new Proxy({},{get(Z,z){if(z in J)return J[z]();return X[z]},set(Z,z,Q){if(z in J){if(typeof Q==="function")J[z].set(()=>Q);else J[z].set(Q);return!0}return console.warn(`[ship-set] handler tried to write s.${z} = … but ${z} is not declared as <state name="${z}"> in the template. The assignment was ignored.`),!0},has(Z,z){return z in J||z in X},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(Z,z){if(z in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[z]()};return}})}function t0(J,X){let Z=J.derived??{},z={};for(let[Q,G]of Object.entries(Z))z[Q]=ZJ(()=>G(X));return z}function e0(J,X){let Z=J.on??{},z={};for(let[Q,G]of Object.entries(Z))z[Q]=(...q)=>G(X,...q);return z}function JX(J,X,Z){if(J.use===void 0)return{};let z=typeof J.use==="function"?J.use(X):J.use,Q={};for(let[G,q]of Object.entries(z))Q[G]=G0(q,Z,X);return Q}function NJ(J){return{setup:(X)=>R0(J,X),__def:J}}if(typeof customElements<"u"){for(let J of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(J))customElements.define(J,class extends HTMLElement{})}var zX=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,WX=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function A0(J){if(typeof CSSStyleSheet>"u")return null;try{let X=new CSSStyleSheet;return X.replaceSync(J),X}catch{return null}}var w0=A0(zX),k0=A0(WX);function IJ(J){if(typeof ShadowRoot>"u")return;if(!(J instanceof ShadowRoot))return;let X=[k0,w0].filter((Z)=>Z!=null);if(X.length===0)return;try{J.adoptedStyleSheets=[...X,...J.adoptedStyleSheets??[]]}catch{}}var pJ=[{id:"view",label:"View",icon:"file",shapes:["list","form","detail","dashboard","tabs","blank"],shapeLabel:"Starting shape"},{id:"skill",label:"Skill",icon:"sparkles",shapes:["task","reference","workflow"],shapeLabel:"Skill kind"},{id:"agent",label:"Agent",icon:"users",shapes:["specialist","generalist","reviewer"],shapeLabel:"Agent kind"},{id:"pipeline",label:"Pipeline",icon:"play",shapes:["sequential","fanout","review-gated"],shapeLabel:"Topology"},{id:"trigger",label:"Trigger",icon:"zap",shapes:["webhook","schedule","file-change"],shapeLabel:"Kind"},{id:"theme",label:"Theme",icon:"palette",shapes:["light","dark","brand-accent","high-contrast"],shapeLabel:"Starting shape"}],iJ=[{label:"Dashboard",path:"/dashboard",icon:"sparkles",keywords:"home overview"},{label:"Pipelines",path:"/pipelines",icon:"play",keywords:"workflows runs jobs"},{label:"Agents",path:"/agents",icon:"users",keywords:"configure profiles"},{label:"Skills",path:"/skills",icon:"zap",keywords:"browse edit files"},{label:"Git",path:"/git",icon:"git",keywords:"status diff commit branch"},{label:"Collections",path:"/collections",icon:"folder",keywords:"data items query"},{label:"Canvas",path:"/canvas",icon:"image",keywords:"docs visual feedback"},{label:"Events",path:"/events",icon:"sparkles",keywords:"stream log kernel"},{label:"Settings",path:"/settings",icon:"settings",keywords:"config theme provider"},{label:"Chat",path:"/chat",icon:"message-square-plus",keywords:"agent conversation"},{label:"Specs",path:"/specs",icon:"file",keywords:"features design"},{label:"Theme Studio",path:"/theme-studio",icon:"palette",keywords:"design tokens"},{label:"Chats (Mission Control)",path:"/chats",icon:"messages-square",keywords:"sessions active"}],nJ=[{key:"new",label:"/new",hint:"Scaffold a view, skill, agent, …"},{key:"chats",label:"/chats",hint:"Open Mission Control"},{key:"settings",label:"/settings",hint:"Jump to settings"},{key:"lint",label:"/lint",hint:"Run lint on the current view"},{key:"end",label:"/end",hint:"End the current session"},{key:"close",label:"/close",hint:"Close the command bar"}],YX=/^[a-z][a-z0-9-]{0,48}$/;function x0(J){return YX.test(J)}function fJ(J,X){if(J.startsWith("/"))return"command";if(J.startsWith(">"))return"find";if(J.startsWith("+"))return"create";if(J.trim().length===0)return X;if(/(^|\s)@[^\s@]*$/.test(J))return"mention";return"ai"}async function DJ(J){let X=window;if(X.ship?.navigate){X.ship.navigate(J);return}history.pushState({},"",J),document.dispatchEvent(new CustomEvent("sui:navigate",{detail:{to:J}}))}var cJ=NJ({scope:(J)=>{let X=J,Z=e(X.open),z=e(X.mode),Q=e(y("")),G=e(y(null)),q=e(y([])),K=e(y([])),B=e(y([])),D=e(y([])),A=e(y(0)),$=e(y(!1)),O=e(y("")),k=e(y("")),i=e(y(!1)),d=e(y(null)),s=e(y([])),QJ=e(y("main")),n=e(y("view")),zJ=e(y("")),u=e(y("")),a=e(y("list")),_=e(y(!1)),t=e(y(null)),T=e(ZJ(()=>z()==="ai"&&G()!==null)),P=eJ(null);return{open:Z,mode:z,value:Q,session:G,agents:q,providers:K,allSessions:B,views:D,selectedIdx:A,sessionPickerOpen:$,spawnAgent:O,spawnProvider:k,spawning:i,spawnError:d,thread:s,workspaceId:QJ,createSurface:n,createName:zJ,createPurpose:u,createShape:a,createSubmitting:_,createError:t,isChat:T,dialogRef:P}},derived:{triggerLabel:(J)=>{let X=J.session();return X?`Ask ${X.agent_id??"Ship"} or jump to…`:"Ask Ship or jump to…"},dotBg:(J)=>{let X=J.session();if(X?.status==="thinking")return"var(--warning,#d97706)";if(X)return"var(--success,#16a34a)";return"var(--muted-foreground)"}},on:{openFind:(J)=>{J.mode="find",J.value="",J.selectedIdx=0,J.open=!0}},mount:(J,X)=>{let Z=J,z=Z.open,Q=Z.mode,G=Z.value,q=Z.session,K=Z.agents,B=Z.providers,D=Z.allSessions,A=Z.views,$=Z.selectedIdx,O=Z.spawnAgent,k=Z.spawnProvider,i=Z.spawning,d=Z.spawnError,s=Z.thread,QJ=Z.workspaceId,n=Z.createSurface,zJ=Z.createName,u=Z.createPurpose,a=Z.createShape,_=Z.createSubmitting,t=Z.createError,T=Z.dialogRef?.(),P=null,I=0,r=l(()=>{let F=z();if(X.setAttribute("data-state",F?"open":"closed"),F){if(typeof T?.show==="function")T.show();queueMicrotask(()=>{X.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelector(".cb-search")?.focus()})}else{if(typeof T?.close==="function")T.close();X.dispatchEvent(new CustomEvent("sui-command:cancel",{detail:{mode:Q()},bubbles:!0,composed:!0}))}});X.show=(F,U)=>{if(F)Q.set(F);if(typeof U==="string")G.set(U);z.set(!0)},X.hide=()=>z.set(!1),X.toggle=(F)=>{if(z())z.set(!1);else{if(F)Q.set(F);z.set(!0)}},X.setMode=(F)=>Q.set(F);function c(F){VJ();let U=new AbortController;P=U;let C=new EventSource(`/api/v1/sessions/${F}/events`,{withCredentials:!0}),R=null;C.addEventListener("provider",(WJ)=>{let o;try{o=JSON.parse(WJ.data)}catch{return}switch(o.kind){case"message":{let qJ=o.text??"";if(!qJ)return;if(R)s.set(s().map((h)=>h.id===R?{...h,text:h.text+qJ}:h));else{let h=String(Date.now()+Math.random());R=h,s.set([...s(),{id:h,sender:"agent",text:qJ}])}break}case"tool_use":s.set([...s(),{id:String(Date.now()+Math.random()),sender:"system",text:`→ tool: ${o.tool??"?"}`}]);break;case"turn_complete":R=null;break}}),C.onerror=()=>{if(!U.signal.aborted)C.close()},U.signal.addEventListener("abort",()=>C.close())}function VJ(){P?.abort(),P=null}let GJ=l(()=>{let F=q();if(F?.id)c(F.id);else VJ()});async function jJ(){try{let F=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(F.ok){let U=await F.json();D.set(U.sessions);let C=U.sessions.find((R)=>R.status!=="ended"&&R.agent_id);if(C&&C.id!==q()?.id)q.set(C),s.set([]);else if(!C)q.set(null)}}catch{}if(K().length===0)LJ();if(B().length===0)UJ();if(A().length===0)m()}async function LJ(){try{let F=await fetch("/api/v1/agents",{credentials:"same-origin"});if(!F.ok)return;let C=((await F.json()).items??[]).map((R)=>({id:R.id}));if(K.set(C),!O()&&C.length>0)O.set(C[0].id)}catch{}}async function UJ(){try{let F=await fetch("/api/v1/providers/installed",{credentials:"same-origin"});if(!F.ok)return;let U=await F.json();if(B.set(U.providers),!k()&&U.providers.length>0)k.set(U.providers[0])}catch{}}async function m(){try{let F=await fetch("/api/v1/views",{credentials:"same-origin"});if(!F.ok)return;let U=await F.json(),C=U.data?.views??U.views??[];A.set(C.map((R)=>R.name).sort())}catch{}}function p(){return X.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelectorAll(".cb-item")}function v(F){let U=p();if(!U?.length)return;$.set(Math.max(0,Math.min(U.length-1,$()+F)))}function x(){let F=Q(),U=G(),R=p()?.[$()];if(R){let WJ=R.dataset.path;if(WJ)DJ(WJ),G.set(""),z.set(!1);let o=R.dataset.cmd;if(o)G.set(""),b("/"+o);let qJ=R.dataset.mentionKind,h=R.dataset.mentionId;if(qJ&&h){let MJ=qJ==="view"?`@${h}`:`@${qJ}:${h}`;G.set(U.replace(/@[^@\s]*$/,`${MJ} `)),Q.set(fJ(G(),F))}X.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:F,item:R.dataset},bubbles:!0,composed:!0}));return}if(F==="command")b(U);else if(F==="find")N(U)}function b(F){let U=F.replace(/^\/+\s*/,"").toLowerCase(),C=nJ.find((R)=>R.key===U||R.key.startsWith(U));if(!C)return;switch(C.key){case"new":Q.set("create"),G.set("");break;case"chats":DJ("/chats"),z.set(!1);break;case"settings":DJ("/settings"),z.set(!1);break;case"end":if(q())j(q().id);z.set(!1);break;case"close":z.set(!1);break;case"lint":L();break}}function N(F){let U=F.replace(/^>+\s*/,"").toLowerCase().trim(),C=iJ.find((R)=>R.label.toLowerCase().includes(U)||R.keywords.toLowerCase().includes(U)||R.path.toLowerCase().includes(U));if(C)DJ(C.path),G.set(""),z.set(!1)}async function j(F){try{await fetch(`/api/v1/sessions/${F}`,{method:"DELETE",credentials:"same-origin"})}catch{}if(q()?.id===F)q.set(null);await jJ()}async function L(){let U=window.ship?.current?.()?.path?.replace(/^\/+/,"").split("/")[0]??"";if(!U)return;try{let C=await fetch("/api/v1/lint/view",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:U})});if(C.ok){let WJ=(await C.json()).data?.findings?.length??0;console.log(WJ===0?`${U}: no lint findings`:`${U}: ${WJ} lint findings`)}}catch{}z.set(!1)}let W=(F)=>{if(F.key==="Escape"&&z()){z.set(!1);return}if(F.key===" "&&!z()){let U=F.target;if(U&&(U.tagName==="INPUT"||U.tagName==="TEXTAREA"||U.isContentEditable))return;let C=performance.now();if(C-I<350)I=0,F.preventDefault(),Q.set("ai"),G.set(""),z.set(!0);else I=C}},Y=(F)=>{if(F.key==="ArrowDown")F.preventDefault(),v(1);else if(F.key==="ArrowUp")F.preventDefault(),v(-1);else if(F.key==="Enter")F.preventDefault(),x()};X.addEventListener("keydown",Y),document.addEventListener("keydown",W);let H=()=>{if(z())z.set(!1)};T?.addEventListener("sui-close",H);let M=()=>void jJ(),V=(F)=>{let U=F;if(U.detail?.mode)Q.set(U.detail.mode);if(typeof U.detail?.prefill==="string")G.set(U.detail.prefill);z.set(!0)},E=()=>{Q.set("create"),z.set(!0)};window.addEventListener("ship:session:changed",M);let S=document.querySelector("ship-shell")??document.querySelector("sui-shell");S?.addEventListener("ship:bar:open",V),S?.addEventListener("ship:create:open",E);let w=X["__sui_ctx_sui-command-bar"];if(w)w.endSession=j,w.loadCurrentSession=jJ;return jJ(),()=>{if(document.removeEventListener("keydown",W),X.removeEventListener("keydown",Y),window.removeEventListener("ship:session:changed",M),S?.removeEventListener("ship:bar:open",V),S?.removeEventListener("ship:create:open",E),T)T.removeEventListener("sui-close",H);GJ(),r()}}});var Lz=BJ("sui-command-bar",{props:{open:$J.bool({default:!1,reflect:!0}),mode:$J.string({default:"ai",reflect:!0})},shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null);Q.open=X.open,Q.mode=X.mode;let G=null;if(cJ&&typeof cJ.setup==="function"){if(J.scope=Q,G=cJ.setup(J),G&&typeof G==="object")Object.assign(Q,G)}let q=(n)=>YJ({anchor:n,test:()=>{if(Q.isChat())return 0;return 1},branches:[()=>{let zJ=document.createDocumentFragment(),u=[],a=document.createElement("sui-command-chat");return zJ.appendChild(a),{nodes:Array.from(zJ.childNodes),cleanup:()=>{for(let _ of u)try{_()}catch(t){}}}},()=>{let zJ=document.createDocumentFragment(),u=[],a=document.createElement("sui-command-palette");return zJ.appendChild(a),{nodes:Array.from(zJ.childNodes),cleanup:()=>{for(let _ of u)try{_()}catch(t){}}}}]}),K=Z.shadowRoot??Z;IJ(K);let B=document.createElement("button");B.setAttribute("class","cb-trigger"),B.setAttribute("type","button"),B.setAttribute("aria-label","Open command bar"),z.push(XJ(B,"click",function(){let n=Q.openFind;return typeof n==="function"?n.bind(void 0):()=>{}}(),{modifiers:[]}));let D=document.createElement("sui-icon");D.setAttribute("name","search"),D.setAttribute("size","sm"),B.appendChild(D);let A=document.createElement("span");A.setAttribute("class","cb-label");let $=document.createTextNode("");A.appendChild($),z.push(g($,()=>Q.triggerLabel())),B.appendChild(A);let O=document.createElement("span");O.setAttribute("class","cb-dot"),z.push(l(()=>{let n=Q.dotBg();if(n==null)O.style.removeProperty("background");else O.style.setProperty("background",String(n))})),B.appendChild(O);let k=document.createElement("kbd"),i=document.createTextNode("⌘K");k.appendChild(i),B.appendChild(k),K.appendChild(B);let d=document.createElement("sui-dialog");d.setAttribute("class","cb-dialog"),d.setAttribute("no-header",""),d.setAttribute("no-footer",""),d.setAttribute("no-close-button",""),d.setAttribute("backdrop","blur"),d.setAttribute("size","full"),d.setAttribute("style","--dlg-max-w: min(840px, 94vw)"),J0(Q,"dialogRef",d);let s=document.createElement("div");s.setAttribute("class","cb-wrap");let QJ=document.createComment("if");if(s.appendChild(QJ),z.push(q(QJ)),d.appendChild(s),K.appendChild(d),G!=null){let n=Object.getOwnPropertyDescriptor(G,"__shipSetLifecycle")?.value;if(n){let zJ=new Proxy(Q,{get(u,a){return u[a]},set(u,a,_){let t=u[a];if(t!=null&&typeof t.set==="function"){if(typeof _==="function")t.set(()=>_);else t.set(_);return!0}return!1}});if(n.mount){let u=n.mount(zJ,Z);if(typeof u==="function")z.push(u)}if(n.unmount)z.push(()=>n.unmount(zJ))}}return Q}});var aJ=NJ({scope:(J,X)=>{let Z=yJ(X,"sui-command-bar"),z=ZJ(()=>{if(Z.session()?.status==="thinking")return"var(--warning,#d97706)";return"var(--success,#16a34a)"}),Q=ZJ(()=>Z.allSessions().filter((D)=>D.status!=="ended"&&D.agent_id));function G(D){if(fetch(`/api/v1/sessions/${D}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}),Z.session()?.id===D)Z.session.set(null);Z.sessionPickerOpen.set(!1)}function q(D){let A=Z.allSessions().find(($)=>$.id===D);if(A)Z.session.set(A),Z.thread.set([]);Z.sessionPickerOpen.set(!1)}function K(){Z.sessionPickerOpen.set(!1),Z.mode.set("ai"),Z.session.set(null)}function B(){Z.sessionPickerOpen.set(!1),DJ("/chats"),Z.open.set(!1)}return{bar:Z,headerDotBg:z,activeSessions:Q,endSession:G,selectSession:q,startNewSession:K,viewChats:B}},on:{toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},sendMessage:async(J)=>{let X=J.bar.value().trim();if(!X||!J.bar.session())return;let Z=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:X}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${Z}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:X})})}catch{}},onComposerKeydown:async(J,X)=>{if(X.key==="Enter"&&!X.shiftKey){X.preventDefault();let Z=J.bar.value().trim();if(!Z||!J.bar.session())return;let z=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:Z}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${z}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:Z})})}catch{}}}}});var Pz=BJ("sui-command-chat",{shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null),G=null;if(aJ&&typeof aJ.setup==="function"){if(J.scope=Q,G=aJ.setup(J),G&&typeof G==="object")Object.assign(Q,G)}let q=(m)=>FJ({anchor:m,items:()=>Q.activeSessions(),key:(p,v)=>{let x=Object.create(Q);return x.s=p,x.__i=v,((b)=>p.id)(x)},render:(p,v)=>{let x=Object.create(Q);return x.s=p,x.__i=v,x.__v=y(0),((b)=>{let N=document.createDocumentFragment(),j=[],L=document.createElement("div");L.setAttribute("class","cb-session-row"),j.push(EJ(L,"current",()=>(b.__v(),b.bar.session()?.id===b.s.id))),L.setAttribute("role","option"),j.push(XJ(L,"click",function(){let C=()=>b.selectSession(b.s.id);return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let W=document.createElement("span");W.setAttribute("class","cb-dot"),j.push(l(()=>{b.__v();let C=b.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(C==null)W.style.removeProperty("background");else W.style.setProperty("background",String(C))})),L.appendChild(W);let Y=document.createElement("span"),H=document.createElement("strong"),M=document.createTextNode("");H.appendChild(M),j.push(g(M,()=>(b.__v(),b.s.agent_id??""))),Y.appendChild(H);let V=document.createElement("span");V.setAttribute("class","meta");let E=document.createTextNode(" · ");V.appendChild(E);let S=document.createTextNode("");V.appendChild(S),j.push(g(S,()=>(b.__v(),b.s.provider??"")));let f=document.createTextNode(" · ");V.appendChild(f);let w=document.createTextNode("");V.appendChild(w),j.push(g(w,()=>(b.__v(),b.s.workspace_id))),Y.appendChild(V),L.appendChild(Y);let F=document.createElement("button");F.setAttribute("class","cb-session-end"),F.setAttribute("type","button"),F.setAttribute("aria-label","End session"),j.push(XJ(F,"click",function(){let C=(R)=>{R.stopPropagation(),b.endSession(b.s.id)};return typeof C==="function"?C.bind(void 0):()=>{}}(),{modifiers:[]}));let U=document.createTextNode("×");return F.appendChild(U),L.appendChild(F),N.appendChild(L),{nodes:Array.from(N.childNodes),cleanup:()=>{for(let C of j)try{C()}catch(R){}},update:(C,R)=>{b.s=C,b.__i=R,b.__v.set((WJ)=>WJ+1)}}})(x)}}),K=(m)=>YJ({anchor:m,test:()=>{if(Q.activeSessions().length===0)return 0;return-1},branches:[()=>{let p=document.createDocumentFragment(),v=[],x=document.createElement("div");x.setAttribute("class","cb-session-row");let b=document.createElement("span");x.appendChild(b);let N=document.createElement("span");N.setAttribute("class","meta");let j=document.createTextNode("No active sessions");N.appendChild(j),x.appendChild(N);let L=document.createElement("span");return x.appendChild(L),p.appendChild(x),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let W of v)try{W()}catch(Y){}}}}]}),B=(m)=>YJ({anchor:m,test:()=>{if(Q.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let p=document.createDocumentFragment(),v=[],x=document.createElement("div");x.setAttribute("class","cb-session-popover"),x.setAttribute("role","listbox"),x.setAttribute("aria-label","Sessions");let b=document.createComment("for");x.appendChild(b),v.push(q(b));let N=document.createComment("if");x.appendChild(N),v.push(K(N));let j=document.createElement("div");j.setAttribute("class","cb-session-sep"),j.setAttribute("role","separator"),x.appendChild(j);let L=document.createElement("div");L.setAttribute("class","cb-session-action"),v.push(XJ(L,"click",function(){let f=Q.startNewSession;return typeof f==="function"?f.bind(void 0):()=>{}}(),{modifiers:[]}));let W=document.createElement("sui-icon");W.setAttribute("name","plus"),W.setAttribute("size","sm"),W.setAttribute("aria-hidden","true"),L.appendChild(W);let Y=document.createElement("span"),H=document.createTextNode("Start new session");Y.appendChild(H),L.appendChild(Y),x.appendChild(L);let M=document.createElement("div");M.setAttribute("class","cb-session-action"),v.push(XJ(M,"click",function(){let f=Q.viewChats;return typeof f==="function"?f.bind(void 0):()=>{}}(),{modifiers:[]}));let V=document.createElement("sui-icon");V.setAttribute("name","columns"),V.setAttribute("size","sm"),V.setAttribute("aria-hidden","true"),M.appendChild(V);let E=document.createElement("span"),S=document.createTextNode("View all chats");return E.appendChild(S),M.appendChild(E),x.appendChild(M),p.appendChild(x),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let f of v)try{f()}catch(w){}}}}]}),D=(m)=>FJ({anchor:m,items:()=>Q.bar.thread(),key:(p,v)=>{let x=Object.create(Q);return x.msg=p,x.__i=v,((b)=>p.id)(x)},render:(p,v)=>{let x=Object.create(Q);return x.msg=p,x.__i=v,x.__v=y(0),((b)=>{let N=document.createDocumentFragment(),j=[],L=document.createElement("div");L.setAttribute("class","cb-chat-turn"),j.push(JJ(L,"data-sender",()=>(b.__v(),b.msg.sender)));let W=document.createElement("div");W.setAttribute("class","cb-chat-bubble");let Y=document.createTextNode("");return W.appendChild(Y),j.push(g(Y,()=>(b.__v(),b.msg.text))),L.appendChild(W),N.appendChild(L),{nodes:Array.from(N.childNodes),cleanup:()=>{for(let H of j)try{H()}catch(M){}},update:(H,M)=>{b.msg=H,b.__i=M,b.__v.set((V)=>V+1)}}})(x)}}),A=(m)=>YJ({anchor:m,test:()=>{if(Q.bar.thread().length===0)return 0;return 1},branches:[()=>{let p=document.createDocumentFragment(),v=[],x=document.createElement("div");x.setAttribute("class","cb-chat-empty");let b=document.createTextNode("Type below and press Enter. The agent's replies land here.");return x.appendChild(b),p.appendChild(x),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let N of v)try{N()}catch(j){}}}},()=>{let p=document.createDocumentFragment(),v=[],x=document.createComment("for");return p.appendChild(x),v.push(D(x)),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let b of v)try{b()}catch(N){}}}}]}),$=Z.shadowRoot??Z;IJ($);let O=document.createElement("div");O.setAttribute("class","cb-chat-header");let k=document.createElement("span");k.setAttribute("class","cb-dot"),z.push(l(()=>{let m=Q.headerDotBg();if(m==null)k.style.removeProperty("background");else k.style.setProperty("background",String(m))})),k.setAttribute("aria-hidden","true"),O.appendChild(k);let i=document.createElement("div");i.setAttribute("style","display:grid;line-height:1.25");let d=document.createElement("span");d.setAttribute("class","title");let s=document.createTextNode("");d.appendChild(s),z.push(g(s,()=>Q.bar.session()?.agent_id??"")),i.appendChild(d);let QJ=document.createElement("span");QJ.setAttribute("class","sub");let n=document.createTextNode("");QJ.appendChild(n),z.push(g(n,()=>Q.bar.session()?.provider??"—"));let zJ=document.createTextNode(" · ");QJ.appendChild(zJ);let u=document.createTextNode("");QJ.appendChild(u),z.push(g(u,()=>Q.bar.session()?.workspace_id??"")),i.appendChild(QJ),O.appendChild(i);let a=document.createElement("span");a.setAttribute("style","flex:1"),O.appendChild(a);let _=document.createElement("div");_.setAttribute("class","cb-chip-wrap");let t=document.createElement("span");t.setAttribute("class","cb-session-chip"),t.setAttribute("role","button"),t.setAttribute("aria-haspopup","listbox"),z.push(XJ(t,"click",function(){let m=Q.toggleSessionPicker;return typeof m==="function"?m.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createTextNode("Switch");t.appendChild(T),_.appendChild(t);let P=document.createComment("if");_.appendChild(P),z.push(B(P)),O.appendChild(_);let I=document.createElement("kbd");I.setAttribute("class","cb-esc"),I.setAttribute("aria-label","Escape to close");let r=document.createTextNode("esc");I.appendChild(r),O.appendChild(I),$.appendChild(O);let c=document.createElement("div");c.setAttribute("class","cb-chat-messages");let VJ=document.createComment("if");c.appendChild(VJ),z.push(A(VJ)),$.appendChild(c);let GJ=document.createElement("div");GJ.setAttribute("class","cb-chat-composer");let jJ=document.createElement("textarea");jJ.setAttribute("class","cb-chat-input"),jJ.setAttribute("rows","2"),jJ.setAttribute("placeholder","Message the agent…"),z.push(HJ(jJ,Q.bar.value)),z.push(XJ(jJ,"keydown",function(){let m=Q.onComposerKeydown;return typeof m==="function"?m.bind(void 0):()=>{}}(),{modifiers:[]})),GJ.appendChild(jJ);let LJ=document.createElement("button");LJ.setAttribute("class","cb-chat-send"),LJ.setAttribute("type","button"),z.push(JJ(LJ,"disabled",()=>!Q.bar.value().trim())),z.push(XJ(LJ,"click",function(){let m=Q.sendMessage;return typeof m==="function"?m.bind(void 0):()=>{}}(),{modifiers:[]}));let UJ=document.createTextNode("Send");if(LJ.appendChild(UJ),GJ.appendChild(LJ),$.appendChild(GJ),G!=null){let m=Object.getOwnPropertyDescriptor(G,"__shipSetLifecycle")?.value;if(m){let p=new Proxy(Q,{get(v,x){return v[x]},set(v,x,b){let N=v[x];if(N!=null&&typeof N.set==="function"){if(typeof b==="function")N.set(()=>b);else N.set(b);return!0}return!1}});if(m.mount){let v=m.mount(p,Z);if(typeof v==="function")z.push(v)}if(m.unmount)z.push(()=>m.unmount(p))}}return Q}});var rJ=NJ({scope:(J,X)=>{let Z=yJ(X,"sui-command-bar"),z=ZJ(()=>{return{command:"/",find:">",mention:"@",create:"+",ai:"ai"}[Z.mode()]??">"}),Q=ZJ(()=>{return{command:"Run a Ship action…",find:"Jump to a view…",mention:"Reference an agent, view, file…",create:"Create a view, skill, agent…",ai:"Ask Ship or type / for actions"}[Z.mode()]??"Search…"}),G=ZJ(()=>Z.session()?.agent_id??"No session"),q=ZJ(()=>{let T=Z.session();if(T?.status==="thinking")return"var(--warning,#d97706)";if(T)return"var(--success,#16a34a)";return"var(--muted-foreground)"}),K=ZJ(()=>Z.allSessions().filter((T)=>T.status!=="ended"&&T.agent_id));function B(T){if(T==="view")return"app-window";if(T==="agent")return"bot";return"message-square-plus"}let D=ZJ(()=>{let T=Z.value().replace(/^>+\s*/,"").toLowerCase().trim();return iJ.filter((P)=>!T||P.label.toLowerCase().includes(T)||P.keywords.toLowerCase().includes(T)||P.path.toLowerCase().includes(T)).map((P,I)=>({...P,idx:I}))}),A=ZJ(()=>{let T=Z.value().replace(/^\/+\s*/,"").toLowerCase().trim();return nJ.filter((P)=>!T||P.key.includes(T)||P.label.toLowerCase().includes(T)||P.hint.toLowerCase().includes(T)).map((P,I)=>({...P,idx:I}))}),$=ZJ(()=>{let T=Z.value().replace(/^@+/,"").toLowerCase().trim();return[...Z.views().map((I)=>({kind:"view",id:I,label:I})),...Z.agents().map((I)=>({kind:"agent",id:I.id,label:I.id})),...Z.allSessions().filter((I)=>I.status!=="ended"&&I.agent_id).map((I)=>({kind:"session",id:I.id,label:`${I.agent_id} (session)`}))].filter((I)=>!T||I.id.toLowerCase().includes(T)).slice(0,40).map((I,r)=>({...I,idx:r,icon:B(I.kind)}))}),O=ZJ(()=>pJ),k=ZJ(()=>pJ.find((T)=>T.id===Z.createSurface())),i=ZJ(()=>x0(Z.createName()));function d(T){fetch(`/api/v1/sessions/${T}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}).then(()=>{if(Z.session()?.id===T)Z.session.set(null);t()}),Z.sessionPickerOpen.set(!1)}function s(T){let P=Z.allSessions().find((I)=>I.id===T);if(P)Z.session.set(P),Z.thread.set([]);Z.sessionPickerOpen.set(!1)}function QJ(){Z.sessionPickerOpen.set(!1),Z.mode.set("ai"),Z.session.set(null)}function n(){Z.sessionPickerOpen.set(!1),DJ("/chats"),Z.open.set(!1)}let zJ=X;function u(T){zJ?.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:"find",item:{path:T}},bubbles:!0,composed:!0})),DJ(T),Z.value.set(""),Z.open.set(!1)}function a(T){switch(Z.value.set(""),T){case"new":Z.mode.set("create");break;case"chats":DJ("/chats"),Z.open.set(!1);break;case"settings":DJ("/settings"),Z.open.set(!1);break;case"end":if(Z.session())d(Z.session().id);Z.open.set(!1);break;case"close":Z.open.set(!1);break}}function _(T){let P=T.kind==="view"?`@${T.id}`:`@${T.kind}:${T.id}`,I=Z.value();Z.value.set(I.replace(/@[^@\s]*$/,`${P} `)),Z.mode.set(fJ(Z.value(),Z.mode()))}async function t(){try{let T=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(T.ok){let P=await T.json();Z.allSessions.set(P.sessions);let I=P.sessions.find((r)=>r.status!=="ended"&&r.agent_id);if(I&&I.id!==Z.session()?.id)Z.session.set(I),Z.thread.set([]);else if(!I)Z.session.set(null)}}catch{}}return{bar:Z,modeGlyph:z,searchPlaceholder:Q,chipLabel:G,chipDotBg:q,activeSessions:K,findItems:D,cmdItems:A,mentionItems:$,createSurfaces:O,currentSurfaceDef:k,createNameValid:i,endSession:d,selectSession:s,startNewSession:QJ,viewChats:n,activateNavItem:u,activateCmd:a,activateMention:_}},on:{onInput:(J,X)=>{let Z=X.target.value;J.bar.value.set(Z),J.bar.mode.set(fJ(Z,J.bar.mode())),J.bar.selectedIdx.set(0)},toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},selectSurface:(J,X)=>{J.bar.createSurface.set(X);let Z=pJ.find((z)=>z.id===X);if(Z?.shapes[0])J.bar.createShape.set(Z.shapes[0])},spawnSession:async(J)=>{let X=J.bar.spawnAgent(),Z=J.bar.spawnProvider();if(!X||!Z)return;J.bar.spawning.set(!0),J.bar.spawnError.set(null);try{let z=await fetch("/api/v1/sessions",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({agent_id:X,provider:Z})});if(!z.ok){let Q=await z.json().catch(()=>({}));J.bar.spawnError.set(Q.error??`HTTP ${z.status}`)}else J.bar.mode.set("ai")}catch(z){J.bar.spawnError.set(z instanceof Error?z.message:"Unknown error")}finally{J.bar.spawning.set(!1)}},submitCreate:async(J)=>{if(!J.createNameValid()||J.bar.createSubmitting())return;J.bar.createSubmitting.set(!0),J.bar.createError.set(null);try{let X=await fetch("/api/v1/scaffold",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({surface:J.bar.createSurface(),name:J.bar.createName(),shape:J.bar.createShape(),purpose:J.bar.createPurpose()})});if(!X.ok){let Z=await X.json().catch(()=>({}));J.bar.createError.set(Z.error??`HTTP ${X.status}`)}else J.bar.createName.set(""),J.bar.createPurpose.set(""),J.bar.open.set(!1)}catch(X){J.bar.createError.set(X instanceof Error?X.message:"Unknown error")}finally{J.bar.createSubmitting.set(!1)}}}});var Cz=BJ("sui-command-palette",{shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null),G=null;if(rJ&&typeof rJ.setup==="function"){if(J.scope=Q,G=rJ.setup(J),G&&typeof G==="object")Object.assign(Q,G)}let q=(N)=>FJ({anchor:N,items:()=>Q.activeSessions(),key:(j,L)=>{let W=Object.create(Q);return W.s=j,W.__i=L,((Y)=>j.id)(W)},render:(j,L)=>{let W=Object.create(Q);return W.s=j,W.__i=L,W.__v=y(0),((Y)=>{let H=document.createDocumentFragment(),M=[],V=document.createElement("div");V.setAttribute("class","cb-session-row"),M.push(EJ(V,"current",()=>(Y.__v(),Y.bar.session()?.id===Y.s.id))),V.setAttribute("role","option"),M.push(XJ(V,"click",function(){let h=()=>Y.selectSession(Y.s.id);return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("span");E.setAttribute("class","cb-dot"),M.push(l(()=>{Y.__v();let h=Y.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(h==null)E.style.removeProperty("background");else E.style.setProperty("background",String(h))})),V.appendChild(E);let S=document.createElement("span"),f=document.createElement("strong"),w=document.createTextNode("");f.appendChild(w),M.push(g(w,()=>(Y.__v(),Y.s.agent_id??""))),S.appendChild(f);let F=document.createElement("span");F.setAttribute("class","meta");let U=document.createTextNode(" · ");F.appendChild(U);let C=document.createTextNode("");F.appendChild(C),M.push(g(C,()=>(Y.__v(),Y.s.provider??"")));let R=document.createTextNode(" · ");F.appendChild(R);let WJ=document.createTextNode("");F.appendChild(WJ),M.push(g(WJ,()=>(Y.__v(),Y.s.workspace_id))),S.appendChild(F),V.appendChild(S);let o=document.createElement("button");o.setAttribute("class","cb-session-end"),o.setAttribute("type","button"),o.setAttribute("aria-label","End session"),M.push(XJ(o,"click",function(){let h=(MJ)=>{MJ.stopPropagation(),Y.endSession(Y.s.id)};return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let qJ=document.createTextNode("×");return o.appendChild(qJ),V.appendChild(o),H.appendChild(V),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let h of M)try{h()}catch(MJ){}},update:(h,MJ)=>{Y.s=h,Y.__i=MJ,Y.__v.set((vJ)=>vJ+1)}}})(W)}}),K=(N)=>YJ({anchor:N,test:()=>{if(Q.activeSessions().length===0)return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-session-row");let Y=document.createElement("span");W.appendChild(Y);let H=document.createElement("span");H.setAttribute("class","meta");let M=document.createTextNode("No active sessions");H.appendChild(M),W.appendChild(H);let V=document.createElement("span");return W.appendChild(V),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let E of L)try{E()}catch(S){}}}}]}),B=(N)=>YJ({anchor:N,test:()=>{if(Q.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-session-popover"),W.setAttribute("role","listbox"),W.setAttribute("aria-label","Sessions");let Y=document.createComment("for");W.appendChild(Y),L.push(q(Y));let H=document.createComment("if");W.appendChild(H),L.push(K(H));let M=document.createElement("div");M.setAttribute("class","cb-session-sep"),M.setAttribute("role","separator"),W.appendChild(M);let V=document.createElement("div");V.setAttribute("class","cb-session-action"),L.push(XJ(V,"click",function(){let R=Q.startNewSession;return typeof R==="function"?R.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("sui-icon");E.setAttribute("name","plus"),E.setAttribute("size","sm"),E.setAttribute("aria-hidden","true"),V.appendChild(E);let S=document.createElement("span"),f=document.createTextNode("Start new session");S.appendChild(f),V.appendChild(S),W.appendChild(V);let w=document.createElement("div");w.setAttribute("class","cb-session-action"),L.push(XJ(w,"click",function(){let R=Q.viewChats;return typeof R==="function"?R.bind(void 0):()=>{}}(),{modifiers:[]}));let F=document.createElement("sui-icon");F.setAttribute("name","columns"),F.setAttribute("size","sm"),F.setAttribute("aria-hidden","true"),w.appendChild(F);let U=document.createElement("span"),C=document.createTextNode("View all chats");return U.appendChild(C),w.appendChild(U),W.appendChild(w),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let R of L)try{R()}catch(WJ){}}}}]}),D=(N)=>FJ({anchor:N,items:()=>Q.findItems(),key:(j,L)=>{let W=Object.create(Q);return W.item=j,W.__i=L,((Y)=>j.path)(W)},render:(j,L)=>{let W=Object.create(Q);return W.item=j,W.__i=L,W.__v=y(0),((Y)=>{let H=document.createDocumentFragment(),M=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),M.push(JJ(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false"))),M.push(JJ(V,"data-path",()=>(Y.__v(),Y.item.path))),M.push(XJ(V,"click",function(){let U=()=>Y.activateNavItem(Y.item.path);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("sui-icon");M.push(JJ(E,"name",()=>(Y.__v(),Y.item.icon))),E.setAttribute("size","sm"),E.setAttribute("aria-hidden","true"),V.appendChild(E);let S=document.createElement("span"),f=document.createTextNode("");S.appendChild(f),M.push(g(f,()=>(Y.__v(),Y.item.label))),V.appendChild(S);let w=document.createElement("span");w.setAttribute("class","hint");let F=document.createTextNode("");return w.appendChild(F),M.push(g(F,()=>(Y.__v(),Y.item.path))),V.appendChild(w),H.appendChild(V),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let U of M)try{U()}catch(C){}},update:(U,C)=>{Y.item=U,Y.__i=C,Y.__v.set((R)=>R+1)}}})(W)}}),A=(N)=>YJ({anchor:N,test:()=>{if(Q.findItems().length>0)return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Jump to");W.appendChild(Y),j.appendChild(W);let H=document.createElement("div");H.setAttribute("class","cb-list"),H.setAttribute("role","listbox");let M=document.createComment("for");return H.appendChild(M),L.push(D(M)),j.appendChild(H),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(E){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No matches.");return W.appendChild(Y),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let H of L)try{H()}catch(M){}}}}]}),$=(N)=>FJ({anchor:N,items:()=>Q.cmdItems(),key:(j,L)=>{let W=Object.create(Q);return W.item=j,W.__i=L,((Y)=>j.key)(W)},render:(j,L)=>{let W=Object.create(Q);return W.item=j,W.__i=L,W.__v=y(0),((Y)=>{let H=document.createDocumentFragment(),M=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),M.push(JJ(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false"))),M.push(JJ(V,"data-cmd",()=>(Y.__v(),Y.item.key))),M.push(XJ(V,"click",function(){let U=()=>Y.activateCmd(Y.item.key);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("span");E.setAttribute("style","font-family:var(--font-mono)");let S=document.createTextNode("");E.appendChild(S),M.push(g(S,()=>(Y.__v(),Y.item.label))),V.appendChild(E);let f=document.createElement("span");V.appendChild(f);let w=document.createElement("span");w.setAttribute("class","hint");let F=document.createTextNode("");return w.appendChild(F),M.push(g(F,()=>(Y.__v(),Y.item.hint))),V.appendChild(w),H.appendChild(V),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let U of M)try{U()}catch(C){}},update:(U,C)=>{Y.item=U,Y.__i=C,Y.__v.set((R)=>R+1)}}})(W)}}),O=(N)=>YJ({anchor:N,test:()=>{if(Q.cmdItems().length>0)return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Actions");W.appendChild(Y),j.appendChild(W);let H=document.createElement("div");H.setAttribute("class","cb-list"),H.setAttribute("role","listbox");let M=document.createComment("for");return H.appendChild(M),L.push($(M)),j.appendChild(H),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(E){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No commands match.");return W.appendChild(Y),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let H of L)try{H()}catch(M){}}}}]}),k=(N)=>FJ({anchor:N,items:()=>Q.mentionItems(),key:(j,L)=>{let W=Object.create(Q);return W.item=j,W.__i=L,((Y)=>j.id+j.kind)(W)},render:(j,L)=>{let W=Object.create(Q);return W.item=j,W.__i=L,W.__v=y(0),((Y)=>{let H=document.createDocumentFragment(),M=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),M.push(JJ(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false"))),M.push(JJ(V,"data-mention-kind",()=>(Y.__v(),Y.item.kind))),M.push(JJ(V,"data-mention-id",()=>(Y.__v(),Y.item.id))),M.push(XJ(V,"click",function(){let U=()=>Y.activateMention(Y.item);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("sui-icon");M.push(JJ(E,"name",()=>(Y.__v(),Y.item.icon))),E.setAttribute("size","sm"),E.setAttribute("aria-hidden","true"),V.appendChild(E);let S=document.createElement("span"),f=document.createTextNode("");S.appendChild(f),M.push(g(f,()=>(Y.__v(),Y.item.label))),V.appendChild(S);let w=document.createElement("span");w.setAttribute("class","hint");let F=document.createTextNode("");return w.appendChild(F),M.push(g(F,()=>(Y.__v(),Y.item.kind))),V.appendChild(w),H.appendChild(V),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let U of M)try{U()}catch(C){}},update:(U,C)=>{Y.item=U,Y.__i=C,Y.__v.set((R)=>R+1)}}})(W)}}),i=(N)=>YJ({anchor:N,test:()=>{if(Q.mentionItems().length>0)return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Reference");W.appendChild(Y),j.appendChild(W);let H=document.createElement("div");H.setAttribute("class","cb-list"),H.setAttribute("role","listbox");let M=document.createComment("for");return H.appendChild(M),L.push(k(M)),j.appendChild(H),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(E){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No matching views or agents.");return W.appendChild(Y),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let H of L)try{H()}catch(M){}}}}]}),d=(N)=>FJ({anchor:N,items:()=>Q.createSurfaces(),key:(j,L)=>{let W=Object.create(Q);return W.surf=j,W.__i=L,((Y)=>j.id)(W)},render:(j,L)=>{let W=Object.create(Q);return W.surf=j,W.__i=L,W.__v=y(0),((Y)=>{let H=document.createDocumentFragment(),M=[],V=document.createElement("button");V.setAttribute("class","cb-surface-btn"),V.setAttribute("type","button"),M.push(JJ(V,"aria-pressed",()=>(Y.__v(),Y.bar.createSurface()===Y.surf.id?"true":"false"))),M.push(XJ(V,"click",function(){let w=()=>Y.selectSurface(Y.surf.id);return typeof w==="function"?w.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("sui-icon");M.push(JJ(E,"name",()=>(Y.__v(),Y.surf.icon))),E.setAttribute("aria-hidden","true"),V.appendChild(E);let S=document.createElement("span"),f=document.createTextNode("");return S.appendChild(f),M.push(g(f,()=>(Y.__v(),Y.surf.label))),V.appendChild(S),H.appendChild(V),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let w of M)try{w()}catch(F){}},update:(w,F)=>{Y.surf=w,Y.__i=F,Y.__v.set((U)=>U+1)}}})(W)}}),s=(N)=>FJ({anchor:N,items:()=>Q.currentSurfaceDef().shapes,key:(j,L)=>{let W=Object.create(Q);return W.sh=j,W.__i=L,((Y)=>j)(W)},render:(j,L)=>{let W=Object.create(Q);return W.sh=j,W.__i=L,W.__v=y(0),((Y)=>{let H=document.createDocumentFragment(),M=[],V=document.createElement("option");M.push(JJ(V,"value",()=>(Y.__v(),Y.sh)));let E=document.createTextNode("");return V.appendChild(E),M.push(g(E,()=>(Y.__v(),Y.sh))),H.appendChild(V),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let S of M)try{S()}catch(f){}},update:(S,f)=>{Y.sh=S,Y.__i=f,Y.__v.set((w)=>w+1)}}})(W)}}),QJ=(N)=>YJ({anchor:N,test:()=>{if(Q.bar.createError())return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-create-error");let Y=document.createTextNode("");return W.appendChild(Y),L.push(g(Y,()=>Q.bar.createError())),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let H of L)try{H()}catch(M){}}}}]}),n=(N)=>FJ({anchor:N,items:()=>Q.bar.agents(),key:(j,L)=>{let W=Object.create(Q);return W.a=j,W.__i=L,((Y)=>j.id)(W)},render:(j,L)=>{let W=Object.create(Q);return W.a=j,W.__i=L,W.__v=y(0),((Y)=>{let H=document.createDocumentFragment(),M=[],V=document.createElement("option");M.push(JJ(V,"value",()=>(Y.__v(),Y.a.id)));let E=document.createTextNode("");return V.appendChild(E),M.push(g(E,()=>(Y.__v(),Y.a.name??Y.a.label??Y.a.id))),H.appendChild(V),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let S of M)try{S()}catch(f){}},update:(S,f)=>{Y.a=S,Y.__i=f,Y.__v.set((w)=>w+1)}}})(W)}}),zJ=(N)=>YJ({anchor:N,test:()=>{if(Q.bar.agents().length===0)return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("option");W.setAttribute("value","");let Y=document.createTextNode("No agents found");return W.appendChild(Y),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let H of L)try{H()}catch(M){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createComment("for");return j.appendChild(W),L.push(n(W)),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(H){}}}}]}),u=(N)=>FJ({anchor:N,items:()=>Q.bar.providers(),key:(j,L)=>{let W=Object.create(Q);return W.p=j,W.__i=L,((Y)=>j)(W)},render:(j,L)=>{let W=Object.create(Q);return W.p=j,W.__i=L,W.__v=y(0),((Y)=>{let H=document.createDocumentFragment(),M=[],V=document.createElement("option");M.push(JJ(V,"value",()=>(Y.__v(),Y.p)));let E=document.createTextNode("");return V.appendChild(E),M.push(g(E,()=>(Y.__v(),Y.p))),H.appendChild(V),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let S of M)try{S()}catch(f){}},update:(S,f)=>{Y.p=S,Y.__i=f,Y.__v.set((w)=>w+1)}}})(W)}}),a=(N)=>YJ({anchor:N,test:()=>{if(Q.bar.providers().length===0)return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("option");W.setAttribute("value","");let Y=document.createTextNode("No providers installed");return W.appendChild(Y),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let H of L)try{H()}catch(M){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createComment("for");return j.appendChild(W),L.push(u(W)),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(H){}}}}]}),_=(N)=>YJ({anchor:N,test:()=>{if(Q.bar.spawnError())return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-spawn-error");let Y=document.createTextNode("");return W.appendChild(Y),L.push(g(Y,()=>Q.bar.spawnError())),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let H of L)try{H()}catch(M){}}}}]}),t=(N)=>YJ({anchor:N,test:()=>{let j=Q.bar.mode();if(j==="find")return 0;if(j==="command")return 1;if(j==="mention")return 2;if(j==="create")return 3;return 4},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createComment("if");return j.appendChild(W),L.push(A(W)),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(H){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createComment("if");return j.appendChild(W),L.push(O(W)),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(H){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createComment("if");return j.appendChild(W),L.push(i(W)),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(H){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Create");W.appendChild(Y),j.appendChild(W);let H=document.createElement("div");H.setAttribute("class","cb-create-surfaces"),H.setAttribute("role","group"),H.setAttribute("aria-label","Surface type");let M=document.createComment("for");H.appendChild(M),L.push(d(M)),j.appendChild(H);let V=document.createElement("div");V.setAttribute("class","cb-create-grid");let E=document.createElement("label"),S=document.createElement("span"),f=document.createTextNode("Name (kebab-case)");S.appendChild(f),E.appendChild(S);let w=document.createElement("input");w.setAttribute("type","text"),w.setAttribute("class","cb-create-name"),L.push(JJ(w,"placeholder",()=>"my-"+Q.bar.createSurface())),L.push(HJ(w,Q.bar.createName)),w.setAttribute("autocomplete","off"),E.appendChild(w),V.appendChild(E);let F=document.createElement("label"),U=document.createElement("span"),C=document.createTextNode("");U.appendChild(C),L.push(g(C,()=>Q.currentSurfaceDef().shapeLabel)),F.appendChild(U);let R=document.createElement("select");R.setAttribute("class","cb-create-shape"),L.push(HJ(R,Q.bar.createShape));let WJ=document.createComment("for");R.appendChild(WJ),L.push(s(WJ)),F.appendChild(R),V.appendChild(F);let o=document.createElement("label");o.setAttribute("class","span-2");let qJ=document.createElement("span"),h=document.createTextNode("Purpose (optional)");qJ.appendChild(h),o.appendChild(qJ);let MJ=document.createElement("input");MJ.setAttribute("type","text"),MJ.setAttribute("class","cb-create-purpose"),MJ.setAttribute("placeholder","One-line description"),L.push(HJ(MJ,Q.bar.createPurpose)),MJ.setAttribute("autocomplete","off"),o.appendChild(MJ),V.appendChild(o),j.appendChild(V);let vJ=document.createComment("if");j.appendChild(vJ),L.push(QJ(vJ));let oJ=document.createElement("div");oJ.setAttribute("class","cb-create-actions");let CJ=document.createElement("button");CJ.setAttribute("class","cb-create-submit"),CJ.setAttribute("type","button"),L.push(JJ(CJ,"disabled",()=>Q.bar.createSubmitting()||!Q.createNameValid())),L.push(XJ(CJ,"click",function(){let mJ=Q.submitCreate;return typeof mJ==="function"?mJ.bind(void 0):()=>{}}(),{modifiers:[]}));let L0=document.createTextNode("");return CJ.appendChild(L0),L.push(g(L0,()=>Q.bar.createSubmitting()?"Creating…":"Create "+Q.currentSurfaceDef().label)),oJ.appendChild(CJ),j.appendChild(oJ),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let mJ of L)try{mJ()}catch(GX){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-spawn-card");let Y=document.createElement("div");Y.setAttribute("class","cb-spawn-title");let H=document.createTextNode("Start a session");Y.appendChild(H),W.appendChild(Y);let M=document.createElement("div");M.setAttribute("class","cb-spawn-row");let V=document.createElement("label"),E=document.createTextNode("Agent");V.appendChild(E),M.appendChild(V);let S=document.createElement("select");S.setAttribute("class","cb-spawn-agent"),L.push(HJ(S,Q.bar.spawnAgent));let f=document.createComment("if");S.appendChild(f),L.push(zJ(f)),M.appendChild(S),W.appendChild(M);let w=document.createElement("div");w.setAttribute("class","cb-spawn-row");let F=document.createElement("label"),U=document.createTextNode("Provider");F.appendChild(U),w.appendChild(F);let C=document.createElement("select");C.setAttribute("class","cb-spawn-provider"),L.push(HJ(C,Q.bar.spawnProvider));let R=document.createComment("if");C.appendChild(R),L.push(a(R)),w.appendChild(C),W.appendChild(w);let WJ=document.createComment("if");W.appendChild(WJ),L.push(_(WJ));let o=document.createElement("button");o.setAttribute("class","cb-spawn-start"),o.setAttribute("type","button"),L.push(JJ(o,"disabled",()=>Q.bar.spawning()||!Q.bar.spawnAgent()||!Q.bar.spawnProvider())),L.push(XJ(o,"click",function(){let h=Q.spawnSession;return typeof h==="function"?h.bind(void 0):()=>{}}(),{modifiers:[]}));let qJ=document.createTextNode("");return o.appendChild(qJ),L.push(g(qJ,()=>Q.bar.spawning()?"Starting…":"Start session")),W.appendChild(o),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let h of L)try{h()}catch(MJ){}}}}]}),T=Z.shadowRoot??Z;IJ(T);let P=document.createElement("div");P.setAttribute("class","cb-input-row");let I=document.createElement("span");I.setAttribute("class","cb-mode-glyph"),z.push(EJ(I,"ai",()=>Q.bar.mode()==="ai"));let r=document.createTextNode("");I.appendChild(r),z.push(g(r,()=>Q.modeGlyph())),P.appendChild(I);let c=document.createElement("input");c.setAttribute("class","cb-search"),c.setAttribute("type","text"),c.setAttribute("autocomplete","off"),c.setAttribute("spellcheck","false"),z.push(JJ(c,"placeholder",()=>Q.searchPlaceholder())),z.push(HJ(c,Q.bar.value)),c.setAttribute("aria-label","Command bar search"),c.setAttribute("aria-autocomplete","list"),z.push(XJ(c,"input",function(){let N=Q.onInput;return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]})),P.appendChild(c);let VJ=document.createElement("kbd");VJ.setAttribute("class","cb-esc"),VJ.setAttribute("aria-label","Escape to close");let GJ=document.createTextNode("esc");VJ.appendChild(GJ),P.appendChild(VJ);let jJ=document.createElement("div");jJ.setAttribute("class","cb-chip-wrap");let LJ=document.createElement("span");LJ.setAttribute("class","cb-session-chip"),LJ.setAttribute("role","button"),LJ.setAttribute("aria-haspopup","listbox"),z.push(XJ(LJ,"click",function(){let N=Q.toggleSessionPicker;return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]}));let UJ=document.createElement("span");UJ.setAttribute("class","cb-dot"),z.push(l(()=>{let N=Q.chipDotBg();if(N==null)UJ.style.removeProperty("background");else UJ.style.setProperty("background",String(N))})),LJ.appendChild(UJ);let m=document.createElement("span"),p=document.createTextNode("");m.appendChild(p),z.push(g(p,()=>Q.chipLabel())),LJ.appendChild(m),jJ.appendChild(LJ);let v=document.createComment("if");jJ.appendChild(v),z.push(B(v)),P.appendChild(jJ),T.appendChild(P);let x=document.createElement("div");x.setAttribute("class","cb-body");let b=document.createComment("match");if(x.appendChild(b),z.push(t(b)),T.appendChild(x),G!=null){let N=Object.getOwnPropertyDescriptor(G,"__shipSetLifecycle")?.value;if(N){let j=new Proxy(Q,{get(L,W){return L[W]},set(L,W,Y){let H=L[W];if(H!=null&&typeof H.set==="function"){if(typeof Y==="function")H.set(()=>Y);else H.set(Y);return!0}return!1}});if(N.mount){let L=N.mount(j,Z);if(typeof L==="function")z.push(L)}if(N.unmount)z.push(()=>N.unmount(j))}}return Q}});
