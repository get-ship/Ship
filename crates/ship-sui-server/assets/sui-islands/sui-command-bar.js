var $0=Symbol.for("sui:error-boundary");function IJ(J){let X=J;while(X){if(X instanceof Element){let z=X[$0];if(z)return z}let Z=X.parentNode;if(Z){X=Z;continue}let Q=X;if(Q.host){X=Q.host;continue}X=null}return null}function RJ(J,X,Z){try{return Z()}catch(Q){let z=IJ(J);if(z){z.catch(Q,X);return}console.error("[sui] uncaught throw (no error boundary):",Q);return}}var UJ=null,BJ=null,f0=0,g0=new Set,SJ=[];function eJ(J,X){SJ.push(J);try{return X()}finally{SJ.pop()}}function m0(){return SJ.length>0?SJ[SJ.length-1]:null}function D0(J){if(UJ===null)return;if(J.has(UJ))return;let X=UJ;if(J.add(X),BJ!==null)BJ.push(()=>J.delete(X))}function x(J){let X=J,Z=new Set,Q=()=>{return D0(Z),X};return Q.set=(z)=>{let G=typeof z==="function"?z(X):z;if(Object.is(G,X))return;if(X=G,f0>0)for(let F of Z)g0.add(F);else for(let F of[...Z])F()},Q.__subs=Z,Q.__peek=()=>X,Q}function ZJ(J){let X,Z=!0,Q=new Set,z=[],G=()=>{if(Z)return;if(Z=!0,Q.size===0)return;let I=X;if(F(),Object.is(I,X))return;for(let H of[...Q])H()},F=()=>{for(let w of z)w();z=[];let I=UJ,H=BJ;UJ=G,BJ=z;try{X=J()}finally{UJ=I,BJ=H}Z=!1},O=()=>{if(D0(Q),Z)F();return X};return O.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},O.__subs=Q,O.__peek=()=>{if(Z)F();return X},O}function h(J){let X=void 0,Z=!0,Q=[],z=m0(),G=()=>{if(!Z)return;if(X){try{X()}catch(I){console.error("[sui] effect cleanup threw:",I)}X=void 0}for(let I of Q)I();Q=[];let F=UJ,O=BJ;UJ=G,BJ=Q;try{X=J()}catch(I){let H=z?IJ(z):null;if(H)H.catch(I,"effect");else console.error("[sui] effect threw:",I)}finally{UJ=F,BJ=O}};return G(),()=>{Z=!1;for(let F of Q)F();if(Q=[],X){try{X()}catch(F){console.error("[sui] effect cleanup threw:",F)}X=void 0}}}var lJ=null;function bJ(J,X){let Z=lJ;lJ=J;try{return X()}finally{lJ=Z}}function yJ(J){let X=lJ;if(!X)return;X.cleanup(J)}function $J(){let J=[],X=!1;return{cleanup(Q){if(X){try{Q()}catch(z){console.error("[sui/scope] late cleanup threw:",z)}return}J.push(Q)},dispose(){if(X)return;X=!0;for(let Q=J.length-1;Q>=0;Q--)try{J[Q]()}catch(z){console.error("[sui/scope] cleanup threw:",z)}J.length=0},get disposed(){return X}}}var H0=Symbol("sui.shared");function _(J){return J[H0]=!0,J}function K0(J){return!!J?.[H0]}function J0(J,X,Z){J[`__sui_ctx_${X}`]=Z}function fJ(J,X){let Z=J;while(Z){let Q=Z[`__sui_ctx_${X}`];if(Q!==void 0)return Q;let z=Z.parentElement;if(z){Z=z;continue}let G=Z.getRootNode();Z=G instanceof ShadowRoot?G.host:null}return null}function X0(J){let X={};for(let Z in J)if(Object.prototype.hasOwnProperty.call(J,Z)&&K0(J[Z]))X[Z]=J[Z];return X}function Z0(J=null){return x(null)}function z0(J,X,Z){let Q=J[X];if(typeof Q==="function"&&typeof Q.set==="function")return Q.set(Z),Q;let z=x(Z);return J[X]=z,z}function VJ(J,X){let Z;return h(()=>{let Q=X(),z=Q==null?"":String(Q);if(z===Z)return;if(Z===void 0&&J.textContent===z){Z=z;return}J.textContent=z,Z=z})}function FJ(J,X,Z){let Q,z=!1;return h(()=>{let G=Z();if(z&&G===Q)return;let F=G==null||G===!1||G===""?null:G===!0?"":String(G);if(z||J.getAttribute(X)!==F)if(F===null)J.removeAttribute(X);else J.setAttribute(X,F);Q=G,z=!0})}function AJ(J,X,Z){let Q;return h(()=>{let z=!!Z();if(z===Q)return;if(Q===void 0&&z===J.classList.contains(X)){Q=z;return}if(z)J.classList.add(X);else J.classList.remove(X);Q=z})}function PJ(J,X,Z){let Q=h(()=>{let G=X();if(J.value!==G)J.value=G}),z=()=>{let G=J.value;if(X.set(G),Z)Z(G)};return J.addEventListener("input",z),()=>{Q(),J.removeEventListener("input",z)}}function e(J,X,Z,Q={}){let z=Q.modifiers??[],G,F=!0,O=(H)=>{for(let C of z)switch(C.kind){case"preventDefault":H.preventDefault();break;case"stopPropagation":H.stopPropagation();break;case"stopImmediatePropagation":H.stopImmediatePropagation();break;case"self":if(H.target!==J)return;break;case"trusted":if(!H.isTrusted)return;break;case"key":{if(!P0(H,C.arg??""))return;break}}let w=z.find((C)=>C.kind==="debounce"),$=z.find((C)=>C.kind==="throttle");if(w){let C=Number(w.arg)||0;if(G!==void 0)clearTimeout(G);G=setTimeout(()=>{G=void 0,Z(H)},C);return}if($){let C=Number($.arg)||0;if(!F)return;F=!1,Z(H),G=setTimeout(()=>{F=!0},C);return}Z(H)},I={};if(Q.capture)I.capture=!0;if(Q.passive)I.passive=!0;if(Q.once)I.once=!0;return J.addEventListener(X,O,I),()=>{if(J.removeEventListener(X,O,I),G!==void 0)clearTimeout(G)}}function P0(J,X){let Z=X.split("+").map((w)=>w.toLowerCase()),Q=Z.pop()??"",z=Z.includes("meta")||Z.includes("cmd"),G=Z.includes("ctrl")||Z.includes("control"),F=Z.includes("shift"),O=Z.includes("alt")||Z.includes("option"),I=Z.includes("mod"),H=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(z&&!J.metaKey)return!1;if(G&&!J.ctrlKey)return!1;if(F&&!J.shiftKey)return!1;if(O&&!J.altKey)return!1;if(I&&!(H?J.metaKey:J.ctrlKey))return!1;return J.key.toLowerCase()===Q}function U0(J,X,Z){J.dispatchEvent(new CustomEvent(X,{detail:Z,bubbles:!0,composed:!0}))}function B0(J,X){let Z=J,Q=Object.keys(X);for(let z of Q)Z[z]=X[z];yJ(()=>{for(let z of Q)if(Z[z]===X[z])try{delete Z[z]}catch{Z[z]=void 0}})}function pJ(J){let X=J;X.emit=function(Z,Q){U0(J,Z,Q)},X.expose=function(Z){B0(J,Z)}}var v0=Symbol("sui:transition");function iJ(J){return J[v0]}function u0(){if(typeof window>"u"||typeof window.matchMedia!=="function")return!1;try{return window.matchMedia("(prefers-reduced-motion: reduce)").matches}catch{return!1}}function O0(J){if(!u0())return J;return{...J,duration:0,delay:0}}function N0(J){let X=iJ(J);if(!X?.inT?.in)return;X.current?.cancel();let Z=X.inT.in(J,O0(X.inOpts));return X.current=Z,Z}function I0(J){let X=iJ(J);if(!X?.outT?.out)return;X.current?.cancel();let Z=X.outT.out(J,O0(X.outOpts));return X.current=Z,Z}function Y0(J){if(J.nodeType!==1)return;let X=J;if(X.hasAttribute("data-sui-transition")){let Z=iJ(X);if(Z?.current?.cancel(),Z)Z.current=void 0}for(let Z of X.querySelectorAll("[data-sui-transition]")){let Q=iJ(Z);if(Q?.current?.cancel(),Q)Q.current=void 0}}function C0(J,X){for(let Z of J){if(Z.nodeType!==1)continue;let Q=Z;if(Q.hasAttribute("data-sui-transition"))X(Q);for(let z of Q.querySelectorAll("[data-sui-transition]"))X(z)}}function wJ(J){C0(J,(X)=>{N0(X)})}function gJ(J){let X=[];if(C0(J,(Z)=>{let Q=I0(Z);if(Q)X.push(Q.finished.catch(()=>{}))}),X.length===0)return null;return Promise.all(X).then(()=>{return})}function h0(J,X,Z,Q,z){if(Z.fill(0,0,X),X===0)return;let G=0;for(let O=0;O<X;O++){let I=J[2*O+1],H=0,w=G;while(H<w){let $=H+w>>>1;if(J[2*z[$]+1]<I)H=$+1;else w=$}if(Q[O]=H>0?z[H-1]:-1,z[H]=O,H===G)G++}let F=z[G-1];while(F>=0)Z[F]=1,F=Q[F]}function DJ(J){let{anchor:X,items:Z,key:Q,render:z}=J,G=[],F=!0,O=new Set,I=new Map,H=new Map,w=new Map,$=new Set,C=[],k=[],n=new Int32Array(0),l=new Uint8Array(0),a=new Int32Array(0),WJ=new Int32Array(0),c=(d)=>{for(let p of d.nodes)try{p.parentNode?.removeChild(p)}catch{}d.cleanup?.()},GJ=null,zJ=h(()=>{if(!F)return;let d=Z();if(d===GJ)return;GJ=d;let p=d.length;if(n.length<p*2)n=new Int32Array(Math.max(n.length*2,p*2,16));if(l.length<p){let P=Math.max(l.length*2,p,8);l=new Uint8Array(P),a=new Int32Array(P),WJ=new Int32Array(P)}for(let P=0;P<p;P++){let S=d[P],r=Q(S,P);if(I.has(r))console.warn("[sui] {#for}: duplicate key",r,"at index",P,"(prior at",I.get(r),"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");I.set(r,P)}for(let P of G)H.set(P.key,P);for(let P of G){if(I.has(P.key))continue;let S=gJ(P.nodes);if(S===null){c(P);continue}O.add(P),S.then(()=>{if(!O.delete(P))return;c(P)})}k.length=0,C.length=0;let JJ=!1;for(let P=0;P<p;P++){let S=d[P],r=Q(S,P),o=H.get(r),s;if(o){if(JJ=!0,o.item!==S||o.__i!==P)o.item=S,o.__i=P,RJ(X,"render",()=>o.update?.(S,P));s=o}else{let jJ=RJ(X,"render",()=>z(S,P));if(!jJ)continue;s={key:r,item:S,__i:P,nodes:jJ.nodes,...jJ.cleanup&&{cleanup:jJ.cleanup},...jJ.update&&{update:jJ.update}},C.push(...jJ.nodes)}k.push(s)}let QJ=0;if(JJ){for(let P=0;P<G.length;P++)w.set(G[P].key,P);for(let P=0;P<k.length;P++){let S=w.get(k[P].key);if(S!==void 0)n[2*QJ]=P,n[2*QJ+1]=S,QJ++}h0(n,QJ,l,a,WJ);for(let P=0;P<QJ;P++)if(l[P])$.add(n[2*P])}let A=X.parentNode;if(A&&QJ===0&&k.length>0){let P=document.createDocumentFragment();for(let S=0;S<k.length;S++){let r=k[S].nodes;for(let o=0;o<r.length;o++)P.appendChild(r[o])}A.insertBefore(P,X.nextSibling)}else{let P=X;for(let S=0;S<k.length;S++){if(!A)break;let r=k[S];if($.has(S))P=r.nodes[r.nodes.length-1]??P;else for(let o of r.nodes)A.insertBefore(o,P.nextSibling),P=o}}let f=G;if(G=k,k=f,C.length>0)wJ(C);I.clear(),H.clear(),w.clear(),$.clear(),C.length=0});return()=>{F=!1,zJ();for(let d of G)c(d);for(let d of O)c(d);O.clear(),G=[],k.length=0,GJ=null}}function MJ(J){let{anchor:X}=J,{test:Z,branches:Q}=d0(J)?{test:J.test,branches:J.branches}:l0(J),z=null,G=-2,F=null,O=(H)=>{for(let w of H)try{w.parentNode?.removeChild(w)}catch{}},I=h(()=>{let H=Z();if(z&&H===G)return;if(F&&H===F.idx&&H>=0&&H<Q.length){let C=F;F=null;for(let k of C.nodes)Y0(k);z={nodes:C.nodes,...C.cleanup&&{cleanup:C.cleanup}},G=H,wJ(z.nodes);return}if(z){let C={idx:G,nodes:z.nodes,...z.cleanup&&{cleanup:z.cleanup}},k=gJ(C.nodes);if(z=null,k===null)O(C.nodes),C.cleanup?.();else F=C,k.then(()=>{if(F!==C)return;O(C.nodes),C.cleanup?.(),F=null})}if(G=H,H<0||H>=Q.length)return;let w=RJ(X,"render",()=>Q[H]());if(!w)return;z=w;let $=X.parentNode;if(!$)return;for(let C of z.nodes)$.insertBefore(C,X);wJ(z.nodes)});return()=>{if(I(),z)O(z.nodes),z.cleanup?.(),z=null;if(F)O(F.nodes),F.cleanup?.(),F=null}}function d0(J){return typeof J.test==="function"}function l0(J){let{cond:X,then_:Z,else_:Q}=J;return{test:()=>X()?0:Q?1:-1,branches:Q?[Z,Q]:[Z]}}var W0=(J)=>J??"",G0=(J)=>{if(J===null||J==="")return NaN;return Number(J)},j0=(J)=>{if(J===null)return!1;if(J.toLowerCase()==="false")return!1;return!0};function nJ(J){return(X)=>{if(X===null||X==="")return J;try{return JSON.parse(X)}catch{return J}}}function L0(J,X){let Z=new Set(J);return(Q)=>Q!==null&&Z.has(Q)?Q:X}var cJ=(J)=>{switch(J.type){case"string":return W0;case"bool":return j0;case"number":return G0;case"union":{let X=J.values??[],Z=J.default??X[0]??"";return L0(X,Z)}case"object":return nJ(J.default);case"array":return nJ(J.default??[])}};function E0(J,X,Z,Q){if(!X)return;let z={},G,F=Z.formValue;if(F!==void 0){let{sig:I,propDef:H}=T0(Z,Q,F,"formValue"),w=H.default,$=cJ(H);G=h(()=>{let C=I();X.setFormValue(p0(C))}),z.reset=()=>{I.set(w)},z.restore=(C)=>{I.set($(C))}}let O=Z.formDisabled;if(O!==void 0){let{sig:I,propDef:H}=T0(Z,Q,O,"formDisabled");if(H.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${O}" must be a bool-typed prop`);z.disabled=(w)=>{I.set(!!w)}}if(J.__shipSetFormHooks=z,G)J.__shipSetFormValueCleanup=G;return G}function R0(J){let X=J;delete X.__shipSetFormHooks,delete X.__shipSetFormValueCleanup}function p0(J){if(J===null||J===void 0)return"";if(typeof J==="boolean")return J?"on":"";return String(J)}function T0(J,X,Z,Q){let z=J.props[Z];if(!z)throw Error(`[sui-runtime] ${Q}: "${Z}" — no matching prop on this component`);let G=X[xJ(Z)];if(!G)throw Error(`[sui-runtime] ${Q}: "${Z}" — prop signal missing`);return{sig:G,propDef:z}}var mJ={string(J={}){return{type:"string",...J}},bool(J={}){return{type:"bool",...J}},number(J={}){return{type:"number",...J}},union(J,X={}){return{type:"union",values:J,...X}},object(J={}){return{type:"object",...J}},array(J={}){return{type:"array",...J}}},i0=(J,X)=>{if(X===void 0||X===null)return null;switch(J.type){case"string":return X===""?null:String(X);case"number":return Number.isNaN(X)?null:String(X);case"bool":return X?"":null;case"union":return String(X);case"object":case"array":try{return JSON.stringify(X)}catch{return null}}},n0=(J,X)=>{let Z=`ship-set-style-${J}`;if(document.getElementById(Z))return;let Q=document.createElement("style");Q.id=Z,Q.textContent=`[data-component="${J}"] { ${X} }`,document.head.appendChild(Q)},c0=(J,X,Z,Q)=>{let z=J.getAttribute(X),G=z!==null?Q(z):Z.default;return x(G)},r0=(J)=>{let X=[],Z={};for(let[G,F]of Object.entries(J.props??{}))X.push(G),Z[G]={camel:xJ(G),decode:cJ(F)};let Q=J.formAssociated===!0||J.formValue!==void 0||J.formDisabled!==void 0,z={observedAttrs:X,props:Z};if(Q)z.formAssociated=!0;return z};function OJ(J,X){let Z=r0(X),Q=Object.entries(X.props??{});V0(J,Z,(z,G,F)=>{let O=null;if(X.shadow){if(O=z.shadowRoot??z.attachShadow({mode:"open"}),X.styles){let H=document.createElement("style");H.textContent=X.styles,O.appendChild(H)}}else if(X.styles)n0(J,X.styles);let I=[];for(let[H,w]of Q){let $=xJ(H),C=Z.props[H].decode,k=c0(z,H,w,C);if(G[$]=k,w.reflect){let n=!0,l=h(()=>{let a=k();if(n){n=!1;return}let WJ=i0(w,a);if(WJ===null)z.removeAttribute(H);else z.setAttribute(H,WJ)});I.push(l)}}if(X.formValue!==void 0||X.formDisabled!==void 0){let H=E0(z,F,X,G);if(typeof H==="function")I.push(H)}if(X.setup){let H={props:G,host:z,cleanups:I};if(F)H.internals=F;X.setup(H)}return()=>{R0(z);for(let H of I.splice(0))try{H()}catch(w){console.error(`[ship-set:${J}] cleanup threw:`,w)}}})}var A0=new Set;function V0(J,X,Z){if(customElements.get(J)){if(!A0.has(J))A0.add(J),console.warn(`[sui-runtime] customElements.define("${J}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(X.formAssociated){class z extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return X.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=J,pJ(this),this.scope=$J();try{this.cleanup=bJ(this.scope,()=>eJ(this,()=>Z(this,this.propSignals,this.internals)))}catch(G){let F=IJ(this.parentNode);if(F)F.catch(G,"mount");else console.error(`[ship-set:${J}] setup threw:`,G)}}disconnectedCallback(){try{this.cleanup?.()}catch(G){console.error(`[ship-set:${J}] cleanup threw:`,G)}this.cleanup=void 0;try{this.scope?.dispose()}catch(G){console.error(`[ship-set:${J}] scope dispose threw:`,G)}this.scope=void 0}attributeChangedCallback(G,F,O){let I=X.props[G];if(!I)return;let H=this.propSignals[I.camel];if(H)H.set(I.decode(O))}formResetCallback(){let G=this.__shipSetFormHooks;try{G?.reset?.()}catch(F){console.error(`[ship-set:${J}] formResetCallback threw:`,F)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(G,F){let O=this.__shipSetFormHooks;try{O?.restore?.(G)}catch(I){console.error(`[ship-set:${J}] formStateRestoreCallback threw:`,I)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:G,mode:F}}))}formDisabledCallback(G){let F=this.__shipSetFormHooks;try{F?.disabled?.(G)}catch(O){console.error(`[ship-set:${J}] formDisabledCallback threw:`,O)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:G}}))}}customElements.define(J,z);return}class Q extends HTMLElement{static get observedAttributes(){return X.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=J,pJ(this),this.scope=$J();try{this.cleanup=bJ(this.scope,()=>eJ(this,()=>Z(this,this.propSignals)))}catch(z){let G=IJ(this.parentNode);if(G)G.catch(z,"mount");else console.error(`[ship-set:${J}] setup threw:`,z)}}disconnectedCallback(){try{this.cleanup?.()}catch(z){console.error(`[ship-set:${J}] cleanup threw:`,z)}this.cleanup=void 0;try{this.scope?.dispose()}catch(z){console.error(`[ship-set:${J}] scope dispose threw:`,z)}this.scope=void 0}attributeChangedCallback(z,G,F){let O=X.props[z];if(!O)return;let I=this.propSignals[O.camel];if(I)I.set(O.decode(F))}}customElements.define(J,Q)}function xJ(J){return J.replace(/-([a-z])/g,(X,Z)=>Z.toUpperCase())}function M0(J,X,Z){let Q=s0(J.state,{ship:X.ship,...Z}),z={};for(let[C,k]of Object.entries(Q))z[C]=x(k);let G={ship:X.ship,route:X.route??{},navigate:X.navigate??((C)=>{}),...Z,...z},F=t0(z,G),O={...G},I={};for(let[C,k]of Object.entries(J.derived??{}))I[C]=ZJ(()=>k(O));Object.assign(O,I);let H={};for(let[C,k]of Object.entries(J.on??{}))H[C]=(...n)=>k(F,...n);let w={},$=typeof J.use==="function"?J.use(O):J.use??{};for(let[C,k]of Object.entries($))w[C]=M0(k,X,O);return{...z,...I,...H,...w}}function s0(J,X){if(J===void 0)return{};if(typeof J==="function")return J(X);return J}function t0(J,X){return new Proxy({},{get(Z,Q){if(Q in J)return J[Q]();return X[Q]},set(Z,Q,z){if(Q in J)return J[Q].set(z),!0;return console.warn(`[ship-set] hook handler tried to write s.${Q} = … but ${Q} is not a hook state key. The assignment was ignored.`),!0},has(Z,Q){return Q in J||Q in X},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(Z,Q){if(Q in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[Q]()};return}})}var w0="__shipSetLifecycle";function x0(J,X){let Z=_0(J,X),Q={ship:X.ship,route:X.route??{},navigate:X.navigate??((C)=>{}),...X.scope??{}},z=Z.scope?Z.scope(Q,X.host):{};Object.assign(Q,z);let G=e0(Q,X.stateKeys,z),F=JX(G,Q),O={...Q},I=XX(Z,O);Object.assign(O,I);let H=zX(Z,O,X);for(let[C,k]of Object.entries(H))O[C]=k;let w=ZX(Z,F),$={...z,...I,...w,...H};if(Object.keys(z).length>0&&X.host){let C=X0(z);if(Object.keys(C).length>0){let k=X.host,n=k.tagName.toLowerCase();J0(k,n,C)}}if(Z.mount||Z.unmount){let C={state:F};if(Z.mount)C.mount=Z.mount;if(Z.unmount)C.unmount=Z.unmount;Object.defineProperty($,w0,{value:C,enumerable:!0,configurable:!0,writable:!1})}return $}function _0(J,X){if(typeof J==="function")return J({ship:X.ship});return J}function e0(J,X,Z){let Q={},z=X!==void 0,G=Z?Object.keys(Z):[];if(z||G.length>0){if(X)for(let F of X){let O=J[F];if(q0(O))Q[F]=O}for(let F of G){if(F in Q)continue;let O=J[F];if(q0(O))Q[F]=O}return Q}for(let[F,O]of Object.entries(J))if(q0(O))Q[F]=O;return Q}function q0(J){return typeof J==="function"&&"set"in J&&typeof J.set==="function"}function JX(J,X){return new Proxy({},{get(Z,Q){if(Q in J)return J[Q]();return X[Q]},set(Z,Q,z){if(Q in J){if(typeof z==="function")J[Q].set(()=>z);else J[Q].set(z);return!0}return console.warn(`[ship-set] handler tried to write s.${Q} = … but ${Q} is not declared as <state name="${Q}"> in the template. The assignment was ignored.`),!0},has(Z,Q){return Q in J||Q in X},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(Z,Q){if(Q in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[Q]()};return}})}function XX(J,X){let Z=J.derived??{},Q={};for(let[z,G]of Object.entries(Z))Q[z]=ZJ(()=>G(X));return Q}function ZX(J,X){let Z=J.on??{},Q={};for(let[z,G]of Object.entries(Z))Q[z]=(...F)=>G(X,...F);return Q}function zX(J,X,Z){if(J.use===void 0)return{};let Q=typeof J.use==="function"?J.use(X):J.use,z={};for(let[G,F]of Object.entries(Q))z[G]=M0(F,Z,X);return z}function CJ(J){return{setup:(X)=>x0(J,X),__def:J}}if(typeof customElements<"u"){for(let J of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(J))customElements.define(J,class extends HTMLElement{})}var GX=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,jX=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function k0(J){if(typeof CSSStyleSheet>"u")return null;try{let X=new CSSStyleSheet;return X.replaceSync(J),X}catch{return null}}var S0=k0(GX),b0=k0(jX);function TJ(J){if(typeof ShadowRoot>"u")return;if(!(J instanceof ShadowRoot))return;let X=[b0,S0].filter((Z)=>Z!=null);if(X.length===0)return;try{J.adoptedStyleSheets=[...X,...J.adoptedStyleSheets??[]]}catch{}}var rJ=[{id:"view",label:"View",icon:"file",shapes:["list","form","detail","dashboard","tabs","blank"],shapeLabel:"Starting shape"},{id:"skill",label:"Skill",icon:"sparkles",shapes:["task","reference","workflow"],shapeLabel:"Skill kind"},{id:"agent",label:"Agent",icon:"users",shapes:["specialist","generalist","reviewer"],shapeLabel:"Agent kind"},{id:"pipeline",label:"Pipeline",icon:"play",shapes:["sequential","fanout","review-gated"],shapeLabel:"Topology"},{id:"trigger",label:"Trigger",icon:"zap",shapes:["webhook","schedule","file-change"],shapeLabel:"Kind"},{id:"theme",label:"Theme",icon:"palette",shapes:["light","dark","brand-accent","high-contrast"],shapeLabel:"Starting shape"}],vJ=[{label:"Dashboard",path:"/dashboard",icon:"sparkles",keywords:"home overview"},{label:"Pipelines",path:"/pipelines",icon:"play",keywords:"workflows runs jobs"},{label:"Agents",path:"/agents",icon:"users",keywords:"configure profiles"},{label:"Skills",path:"/skills",icon:"zap",keywords:"browse edit files"},{label:"Git",path:"/git",icon:"git",keywords:"status diff commit branch"},{label:"Collections",path:"/collections",icon:"folder",keywords:"data items query"},{label:"Canvas",path:"/canvas",icon:"image",keywords:"docs visual feedback"},{label:"Events",path:"/events",icon:"sparkles",keywords:"stream log kernel"},{label:"Settings",path:"/settings",icon:"settings",keywords:"config theme provider"},{label:"Chat",path:"/chat",icon:"message-square-plus",keywords:"agent conversation"},{label:"Specs",path:"/specs",icon:"file",keywords:"features design"},{label:"Theme Studio",path:"/theme-studio",icon:"palette",keywords:"design tokens"},{label:"Chats (Mission Control)",path:"/chats",icon:"messages-square",keywords:"sessions active"}],aJ=[{key:"new",label:"/new",hint:"Scaffold a view, skill, agent, …"},{key:"chats",label:"/chats",hint:"Open Mission Control"},{key:"settings",label:"/settings",hint:"Jump to settings"},{key:"lint",label:"/lint",hint:"Run lint on the current view"},{key:"end",label:"/end",hint:"End the current session"},{key:"close",label:"/close",hint:"Close the command bar"}],LX=/^[a-z][a-z0-9-]{0,48}$/;function y0(J){return LX.test(J)}function uJ(J,X){if(J.startsWith("/"))return"command";if(J.startsWith(">"))return"find";if(J.startsWith("+"))return"create";if(J.trim().length===0)return X;if(/(^|\s)@[^\s@]*$/.test(J))return"mention";return"ai"}async function KJ(J){let X=window;if(X.ship?.navigate){X.ship.navigate(J);return}history.pushState({},"",J),document.dispatchEvent(new CustomEvent("sui:navigate",{detail:{to:J}}))}var oJ=CJ({scope:(J)=>{let X=J,Z=_(X.open),Q=_(X.mode),z=_(x("")),G=_(x(null)),F=_(x([])),O=_(x([])),I=_(x([])),H=_(x([])),w=_(x(0)),$=_(x(!1)),C=_(x("")),k=_(x("")),n=_(x(!1)),l=_(x(null)),a=_(x([])),WJ=_(x("main")),c=_(x("view")),GJ=_(x("")),zJ=_(x("")),d=_(x("list")),p=_(x(!1)),JJ=_(x(null)),QJ=_(ZJ(()=>Q()==="ai"&&G()!==null)),A=Z0(null);return{open:Z,mode:Q,value:z,session:G,agents:F,providers:O,allSessions:I,views:H,selectedIdx:w,sessionPickerOpen:$,spawnAgent:C,spawnProvider:k,spawning:n,spawnError:l,thread:a,workspaceId:WJ,createSurface:c,createName:GJ,createPurpose:zJ,createShape:d,createSubmitting:p,createError:JJ,isChat:QJ,dialogRef:A}},derived:{triggerLabel:(J)=>{let X=J.session();return X?`Ask ${X.agent_id??"Ship"} or jump to…`:"Ask Ship or jump to…"},dotBg:(J)=>{let X=J.session();if(X?.status==="thinking")return"var(--warning,#d97706)";if(X)return"var(--success,#16a34a)";return"var(--muted-foreground)"}},on:{openFind:(J)=>{J.mode="find",J.value="",J.selectedIdx=0,J.open=!0}},mount:(J,X)=>{let Z=J,Q=Z.open,z=Z.mode,G=Z.value,F=Z.session,O=Z.agents,I=Z.providers,H=Z.allSessions,w=Z.views,$=Z.selectedIdx,C=Z.spawnAgent,k=Z.spawnProvider,n=Z.spawning,l=Z.spawnError,a=Z.thread,WJ=Z.workspaceId,c=Z.createSurface,GJ=Z.createName,zJ=Z.createPurpose,d=Z.createShape,p=Z.createSubmitting,JJ=Z.createError,QJ=Z.dialogRef?.(),A=null,f=0,P=h(()=>{let M=Q();if(X.setAttribute("data-state",M?"open":"closed"),M){if(typeof QJ?.show==="function")QJ.show();queueMicrotask(()=>{X.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelector(".cb-search")?.focus()})}else{if(typeof QJ?.close==="function")QJ.close();X.dispatchEvent(new CustomEvent("sui-command:cancel",{detail:{mode:z()},bubbles:!0,composed:!0}))}});X.show=(M,U)=>{if(M)z.set(M);if(typeof U==="string")G.set(U);Q.set(!0)},X.hide=()=>Q.set(!1),X.toggle=(M)=>{if(Q())Q.set(!1);else{if(M)z.set(M);Q.set(!0)}},X.setMode=(M)=>z.set(M);function S(M){r();let U=new AbortController;A=U;let q=new EventSource(`/api/v1/sessions/${M}/events`,{withCredentials:!0}),R=null;q.addEventListener("provider",(D)=>{let u;try{u=JSON.parse(D.data)}catch{return}switch(u.kind){case"message":{let B=u.text??"";if(!B)return;if(R)a.set(a().map((YJ)=>YJ.id===R?{...YJ,text:YJ.text+B}:YJ));else{let YJ=String(Date.now()+Math.random());R=YJ,a.set([...a(),{id:YJ,sender:"agent",text:B}])}break}case"tool_use":a.set([...a(),{id:String(Date.now()+Math.random()),sender:"system",text:`→ tool: ${u.tool??"?"}`}]);break;case"turn_complete":R=null;break}}),q.onerror=()=>{if(!U.signal.aborted)q.close()},U.signal.addEventListener("abort",()=>q.close())}function r(){A?.abort(),A=null}let o=h(()=>{let M=F();if(M?.id)S(M.id);else r()});async function s(){try{let M=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(M.ok){let U=await M.json();H.set(U.sessions);let q=U.sessions.find((R)=>R.status!=="ended"&&R.agent_id);if(q&&q.id!==F()?.id)F.set(q),a.set([]);else if(!q)F.set(null)}}catch{}if(O().length===0)jJ();if(I().length===0)kJ();if(w().length===0)v()}async function jJ(){try{let M=await fetch("/api/v1/agents",{credentials:"same-origin"});if(!M.ok)return;let q=((await M.json()).items??[]).map((R)=>({id:R.id}));if(O.set(q),!C()&&q.length>0)C.set(q[0].id)}catch{}}async function kJ(){try{let M=await fetch("/api/v1/providers/installed",{credentials:"same-origin"});if(!M.ok)return;let U=await M.json();if(I.set(U.providers),!k()&&U.providers.length>0)k.set(U.providers[0])}catch{}}async function v(){try{let M=await fetch("/api/v1/views",{credentials:"same-origin"});if(!M.ok)return;let U=await M.json(),q=U.data?.views??U.views??[];w.set(q.map((R)=>R.name).sort())}catch{}}function g(){return X.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelectorAll(".cb-item")}function m(M){let U=g();if(!U?.length)return;$.set(Math.max(0,Math.min(U.length-1,$()+M)))}function b(){let M=z(),U=G(),R=g()?.[$()];if(R){let D=R.dataset.path;if(D)KJ(D),G.set(""),Q.set(!1);let u=R.dataset.cmd;if(u)G.set(""),y("/"+u);let B=R.dataset.mentionKind,YJ=R.dataset.mentionId;if(B&&YJ){let LJ=B==="view"?`@${YJ}`:`@${B}:${YJ}`;G.set(U.replace(/@[^@\s]*$/,`${LJ} `)),z.set(uJ(G(),M))}X.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:M,item:R.dataset},bubbles:!0,composed:!0}));return}if(M==="command")y(U);else if(M==="find")t(U)}function y(M){let U=M.replace(/^\/+\s*/,"").toLowerCase(),q=aJ.find((R)=>R.key===U||R.key.startsWith(U));if(!q)return;switch(q.key){case"new":z.set("create"),G.set("");break;case"chats":KJ("/chats"),Q.set(!1);break;case"settings":KJ("/settings"),Q.set(!1);break;case"end":if(F())XJ(F().id);Q.set(!1);break;case"close":Q.set(!1);break;case"lint":i();break}}function t(M){let U=M.replace(/^>+\s*/,"").toLowerCase().trim(),q=vJ.find((R)=>R.label.toLowerCase().includes(U)||R.keywords.toLowerCase().includes(U)||R.path.toLowerCase().includes(U));if(q)KJ(q.path),G.set(""),Q.set(!1)}async function XJ(M){try{await fetch(`/api/v1/sessions/${M}`,{method:"DELETE",credentials:"same-origin"})}catch{}if(F()?.id===M)F.set(null);await s()}async function i(){let U=window.ship?.current?.()?.path?.replace(/^\/+/,"").split("/")[0]??"";if(!U)return;try{let q=await fetch("/api/v1/lint/view",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:U})});if(q.ok){let D=(await q.json()).data?.findings?.length??0;console.log(D===0?`${U}: no lint findings`:`${U}: ${D} lint findings`)}}catch{}Q.set(!1)}let E=(M)=>{if(M.key==="Escape"&&Q()){Q.set(!1);return}if(M.key===" "&&!Q()){let U=M.target;if(U&&(U.tagName==="INPUT"||U.tagName==="TEXTAREA"||U.isContentEditable))return;let q=performance.now();if(q-f<350)f=0,M.preventDefault(),z.set("ai"),G.set(""),Q.set(!0);else f=q}},j=(M)=>{if(M.key==="ArrowDown")M.preventDefault(),m(1);else if(M.key==="ArrowUp")M.preventDefault(),m(-1);else if(M.key==="Enter")M.preventDefault(),b()};X.addEventListener("keydown",j),document.addEventListener("keydown",E);let L=()=>{if(Q())Q.set(!1)};QJ?.addEventListener("sui-close",L);let W=()=>void s(),Y=(M)=>{let U=M;if(U.detail?.mode)z.set(U.detail.mode);if(typeof U.detail?.prefill==="string")G.set(U.detail.prefill);Q.set(!0)},K=()=>{z.set("create"),Q.set(!0)};window.addEventListener("ship:session:changed",W);let N=document.querySelector("ship-shell")??document.querySelector("sui-shell");N?.addEventListener("ship:bar:open",Y),N?.addEventListener("ship:create:open",K);let T=X["__sui_ctx_sui-command-bar"];if(T)T.endSession=XJ,T.loadCurrentSession=s;return s(),()=>{if(document.removeEventListener("keydown",E),X.removeEventListener("keydown",j),window.removeEventListener("ship:session:changed",W),N?.removeEventListener("ship:bar:open",Y),N?.removeEventListener("ship:create:open",K),QJ)QJ.removeEventListener("sui-close",L);o(),P()}}});var BQ=OJ("sui-command-bar",{props:{open:mJ.bool({default:!1,reflect:!0}),mode:mJ.string({default:"ai",reflect:!0})},shadow:!0,styles:`
  /* Host participates in layout so the trigger pill carries its own
     box (probes asserting commandBar.offsetWidth need a non-zero
     measurement). Inline-flex keeps the pill sized to its content
     while the explicit min-width matches the trigger's design width. */
  :host {
    display: inline-flex;
    align-items: center;
    min-width: 200px;
    max-width: 320px;
  }

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
    width: 100%;
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let Q=J.cleanups,z=Object.create(null);z.open=X.open,z.mode=X.mode,z.dialogRef=x(null);let G=null;if(oJ&&typeof oJ.setup==="function"){if(J.scope=z,G=oJ.setup(J),G&&typeof G==="object")Object.assign(z,G)}let F=(c)=>MJ({anchor:c,test:()=>{if(z.isChat())return 0;return 1},branches:[()=>{let GJ=document.createDocumentFragment(),zJ=[],d=document.createElement("sui-command-chat");return GJ.appendChild(d),{nodes:Array.from(GJ.childNodes),cleanup:()=>{for(let p of zJ)try{p()}catch(JJ){}}}},()=>{let GJ=document.createDocumentFragment(),zJ=[],d=document.createElement("sui-command-palette");return GJ.appendChild(d),{nodes:Array.from(GJ.childNodes),cleanup:()=>{for(let p of zJ)try{p()}catch(JJ){}}}}]}),O=Z.shadowRoot??Z;TJ(O);let I=document.createElement("button");I.setAttribute("class","cb-trigger"),I.setAttribute("type","button"),I.setAttribute("aria-label","Open command bar"),Q.push(e(I,"click",function(){let c=z.openFind;return typeof c==="function"?c.bind(void 0):()=>{}}(),{modifiers:[]}));let H=document.createElement("sui-icon");H.setAttribute("name","search"),H.setAttribute("size","sm"),I.appendChild(H);let w=document.createElement("span");w.setAttribute("class","cb-label");let $=document.createTextNode("");w.appendChild($),Q.push(VJ($,()=>z.triggerLabel())),I.appendChild(w);let C=document.createElement("span");C.setAttribute("class","cb-dot"),Q.push(h(()=>{let c=z.dotBg();if(c==null)C.style.removeProperty("background");else C.style.setProperty("background",String(c))})),I.appendChild(C);let k=document.createElement("kbd"),n=document.createTextNode("⌘K");k.appendChild(n),I.appendChild(k),O.appendChild(I);let l=document.createElement("sui-dialog");l.setAttribute("class","cb-dialog"),l.setAttribute("no-header",""),l.setAttribute("no-footer",""),l.setAttribute("no-close-button",""),l.setAttribute("backdrop","blur"),l.setAttribute("size","full"),l.setAttribute("style","--dlg-max-w: min(840px, 94vw)"),z0(z,"dialogRef",l);let a=document.createElement("div");a.setAttribute("class","cb-wrap");let WJ=document.createComment("if");if(a.appendChild(WJ),Q.push(F(WJ)),l.appendChild(a),O.appendChild(l),G!=null){let c=Object.getOwnPropertyDescriptor(G,"__shipSetLifecycle")?.value;if(c){let GJ=new Proxy(z,{get(zJ,d){return zJ[d]},set(zJ,d,p){let JJ=zJ[d];if(JJ!=null&&typeof JJ.set==="function"){if(typeof p==="function")JJ.set(()=>p);else JJ.set(p);return!0}return!1}});if(c.mount){let zJ=c.mount(GJ,Z);if(typeof zJ==="function")Q.push(zJ)}if(c.unmount)Q.push(()=>c.unmount(GJ))}}return z}});var sJ=CJ({scope:(J,X)=>{let Z=fJ(X,"sui-command-bar"),Q=ZJ(()=>{if(Z.session()?.status==="thinking")return"var(--warning,#d97706)";return"var(--success,#16a34a)"}),z=ZJ(()=>Z.allSessions().filter((H)=>H.status!=="ended"&&H.agent_id));function G(H){if(fetch(`/api/v1/sessions/${H}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}),Z.session()?.id===H)Z.session.set(null);Z.sessionPickerOpen.set(!1)}function F(H){let w=Z.allSessions().find(($)=>$.id===H);if(w)Z.session.set(w),Z.thread.set([]);Z.sessionPickerOpen.set(!1)}function O(){Z.sessionPickerOpen.set(!1),Z.mode.set("ai"),Z.session.set(null)}function I(){Z.sessionPickerOpen.set(!1),KJ("/chats"),Z.open.set(!1)}return{bar:Z,headerDotBg:Q,activeSessions:z,endSession:G,selectSession:F,startNewSession:O,viewChats:I}},on:{toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},sendMessage:async(J)=>{let X=J.bar.value().trim();if(!X||!J.bar.session())return;let Z=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:X}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${Z}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:X})})}catch{}},onComposerKeydown:async(J,X)=>{if(X.key==="Enter"&&!X.shiftKey){X.preventDefault();let Z=J.bar.value().trim();if(!Z||!J.bar.session())return;let Q=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:Z}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${Q}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:Z})})}catch{}}}}});var RQ=OJ("sui-command-chat",{shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let Q=J.cleanups,z=Object.create(null),G=null;if(sJ&&typeof sJ.setup==="function"){if(J.scope=z,G=sJ.setup(J),G&&typeof G==="object")Object.assign(z,G)}let F=(v)=>DJ({anchor:v,items:()=>z.activeSessions(),key:(g,m)=>{let b=Object.create(z);return b.s=g,b.__i=m,((y)=>g.id)(b)},render:(g,m)=>{let b=Object.create(z);return b.s=g,b.__i=m,b.__v=x(0),((y)=>{let t=document.createDocumentFragment(),XJ=[],i=document.createElement("div");i.setAttribute("class","cb-session-row"),XJ.push(AJ(i,"current",()=>(y.__v(),y.bar.session()?.id===y.s.id))),i.setAttribute("role","option"),XJ.push(e(i,"click",function(){let q=()=>y.selectSession(y.s.id);return typeof q==="function"?q.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("span");E.setAttribute("class","cb-dot"),XJ.push(h(()=>{y.__v();let q=y.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(q==null)E.style.removeProperty("background");else E.style.setProperty("background",String(q))})),i.appendChild(E);let j=document.createElement("span"),L=document.createElement("strong"),W=document.createTextNode("");L.appendChild(W),XJ.push(VJ(W,()=>(y.__v(),y.s.agent_id??""))),j.appendChild(L);let Y=document.createElement("span");Y.setAttribute("class","meta");let K=document.createTextNode(" · ");Y.appendChild(K);let N=document.createTextNode("");Y.appendChild(N),XJ.push(VJ(N,()=>(y.__v(),y.s.provider??"")));let V=document.createTextNode(" · ");Y.appendChild(V);let T=document.createTextNode("");Y.appendChild(T),T.textContent=y.s.workspace_id,j.appendChild(Y),i.appendChild(j);let M=document.createElement("button");M.setAttribute("class","cb-session-end"),M.setAttribute("type","button"),M.setAttribute("aria-label","End session"),XJ.push(e(M,"click",function(){let q=(R)=>{R.stopPropagation(),y.endSession(y.s.id)};return typeof q==="function"?q.bind(void 0):()=>{}}(),{modifiers:[]}));let U=document.createTextNode("×");return M.appendChild(U),i.appendChild(M),t.appendChild(i),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let q of XJ)try{q()}catch(R){}},update:(q,R)=>{y.s=q,y.__i=R,T.textContent=y.s.workspace_id,y.__v.set((D)=>D+1)}}})(b)}}),O=(v)=>MJ({anchor:v,test:()=>{if(z.activeSessions().length===0)return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),m=[],b=document.createElement("div");b.setAttribute("class","cb-session-row");let y=document.createElement("span");b.appendChild(y);let t=document.createElement("span");t.setAttribute("class","meta");let XJ=document.createTextNode("No active sessions");t.appendChild(XJ),b.appendChild(t);let i=document.createElement("span");return b.appendChild(i),g.appendChild(b),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let E of m)try{E()}catch(j){}}}}]}),I=(v)=>MJ({anchor:v,test:()=>{if(z.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let g=document.createDocumentFragment(),m=[],b=document.createElement("div");b.setAttribute("class","cb-session-popover"),b.setAttribute("role","listbox"),b.setAttribute("aria-label","Sessions");let y=document.createComment("for");b.appendChild(y),m.push(F(y));let t=document.createComment("if");b.appendChild(t),m.push(O(t));let XJ=document.createElement("div");XJ.setAttribute("class","cb-session-sep"),XJ.setAttribute("role","separator"),b.appendChild(XJ);let i=document.createElement("div");i.setAttribute("class","cb-session-action"),m.push(e(i,"click",function(){let V=z.startNewSession;return typeof V==="function"?V.bind(void 0):()=>{}}(),{modifiers:[]}));let E=document.createElement("sui-icon");E.setAttribute("name","plus"),E.setAttribute("size","sm"),E.setAttribute("aria-hidden","true"),i.appendChild(E);let j=document.createElement("span"),L=document.createTextNode("Start new session");j.appendChild(L),i.appendChild(j),b.appendChild(i);let W=document.createElement("div");W.setAttribute("class","cb-session-action"),m.push(e(W,"click",function(){let V=z.viewChats;return typeof V==="function"?V.bind(void 0):()=>{}}(),{modifiers:[]}));let Y=document.createElement("sui-icon");Y.setAttribute("name","columns"),Y.setAttribute("size","sm"),Y.setAttribute("aria-hidden","true"),W.appendChild(Y);let K=document.createElement("span"),N=document.createTextNode("View all chats");return K.appendChild(N),W.appendChild(K),b.appendChild(W),g.appendChild(b),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let V of m)try{V()}catch(T){}}}}]}),H=(v)=>DJ({anchor:v,items:()=>z.bar.thread(),key:(g,m)=>{let b=Object.create(z);return b.msg=g,b.__i=m,((y)=>g.id)(b)},render:(g,m)=>{let b=Object.create(z);return b.msg=g,b.__i=m,b.__v=x(0),((y)=>{let t=document.createDocumentFragment(),XJ=[],i=document.createElement("div");i.setAttribute("class","cb-chat-turn");{let L=y.msg.sender;if(L==null||L===!1||L==="")i.removeAttribute("data-sender");else i.setAttribute("data-sender",L===!0?"":String(L))}let E=document.createElement("div");E.setAttribute("class","cb-chat-bubble");let j=document.createTextNode("");return E.appendChild(j),j.textContent=y.msg.text,i.appendChild(E),t.appendChild(i),{nodes:Array.from(t.childNodes),cleanup:()=>{for(let L of XJ)try{L()}catch(W){}},update:(L,W)=>{y.msg=L,y.__i=W;{let Y=y.msg.sender;if(Y==null||Y===!1||Y==="")i.removeAttribute("data-sender");else i.setAttribute("data-sender",Y===!0?"":String(Y))}j.textContent=y.msg.text,y.__v.set((Y)=>Y+1)}}})(b)}}),w=(v)=>MJ({anchor:v,test:()=>{if(z.bar.thread().length===0)return 0;return 1},branches:[()=>{let g=document.createDocumentFragment(),m=[],b=document.createElement("div");b.setAttribute("class","cb-chat-empty");let y=document.createTextNode("Type below and press Enter. The agent's replies land here.");return b.appendChild(y),g.appendChild(b),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let t of m)try{t()}catch(XJ){}}}},()=>{let g=document.createDocumentFragment(),m=[],b=document.createComment("for");return g.appendChild(b),m.push(H(b)),{nodes:Array.from(g.childNodes),cleanup:()=>{for(let y of m)try{y()}catch(t){}}}}]}),$=Z.shadowRoot??Z;TJ($);let C=document.createElement("div");C.setAttribute("class","cb-chat-header");let k=document.createElement("span");k.setAttribute("class","cb-dot"),Q.push(h(()=>{let v=z.headerDotBg();if(v==null)k.style.removeProperty("background");else k.style.setProperty("background",String(v))})),k.setAttribute("aria-hidden","true"),C.appendChild(k);let n=document.createElement("div");n.setAttribute("style","display:grid;line-height:1.25");let l=document.createElement("span");l.setAttribute("class","title");let a=document.createTextNode("");l.appendChild(a),Q.push(VJ(a,()=>z.bar.session()?.agent_id??"")),n.appendChild(l);let WJ=document.createElement("span");WJ.setAttribute("class","sub");let c=document.createTextNode("");WJ.appendChild(c),Q.push(VJ(c,()=>z.bar.session()?.provider??"—"));let GJ=document.createTextNode(" · ");WJ.appendChild(GJ);let zJ=document.createTextNode("");WJ.appendChild(zJ),Q.push(VJ(zJ,()=>z.bar.session()?.workspace_id??"")),n.appendChild(WJ),C.appendChild(n);let d=document.createElement("span");d.setAttribute("style","flex:1"),C.appendChild(d);let p=document.createElement("div");p.setAttribute("class","cb-chip-wrap");let JJ=document.createElement("span");JJ.setAttribute("class","cb-session-chip"),JJ.setAttribute("role","button"),JJ.setAttribute("aria-haspopup","listbox"),Q.push(e(JJ,"click",function(){let v=z.toggleSessionPicker;return typeof v==="function"?v.bind(void 0):()=>{}}(),{modifiers:[]}));let QJ=document.createTextNode("Switch");JJ.appendChild(QJ),p.appendChild(JJ);let A=document.createComment("if");p.appendChild(A),Q.push(I(A)),C.appendChild(p);let f=document.createElement("kbd");f.setAttribute("class","cb-esc"),f.setAttribute("aria-label","Escape to close");let P=document.createTextNode("esc");f.appendChild(P),C.appendChild(f),$.appendChild(C);let S=document.createElement("div");S.setAttribute("class","cb-chat-messages");let r=document.createComment("if");S.appendChild(r),Q.push(w(r)),$.appendChild(S);let o=document.createElement("div");o.setAttribute("class","cb-chat-composer");let s=document.createElement("textarea");s.setAttribute("class","cb-chat-input"),s.setAttribute("rows","2"),s.setAttribute("placeholder","Message the agent…"),Q.push(PJ(s,z.bar.value)),Q.push(e(s,"keydown",function(){let v=z.onComposerKeydown;return typeof v==="function"?v.bind(void 0):()=>{}}(),{modifiers:[]})),o.appendChild(s);let jJ=document.createElement("button");jJ.setAttribute("class","cb-chat-send"),jJ.setAttribute("type","button"),Q.push(FJ(jJ,"disabled",()=>!z.bar.value().trim())),Q.push(e(jJ,"click",function(){let v=z.sendMessage;return typeof v==="function"?v.bind(void 0):()=>{}}(),{modifiers:[]}));let kJ=document.createTextNode("Send");if(jJ.appendChild(kJ),o.appendChild(jJ),$.appendChild(o),G!=null){let v=Object.getOwnPropertyDescriptor(G,"__shipSetLifecycle")?.value;if(v){let g=new Proxy(z,{get(m,b){return m[b]},set(m,b,y){let t=m[b];if(t!=null&&typeof t.set==="function"){if(typeof y==="function")t.set(()=>y);else t.set(y);return!0}return!1}});if(v.mount){let m=v.mount(g,Z);if(typeof m==="function")Q.push(m)}if(v.unmount)Q.push(()=>v.unmount(g))}}return z}});var tJ=CJ({scope:(J,X)=>{let Z=fJ(X,"sui-command-bar"),Q=ZJ(()=>{return{command:"/",find:">",mention:"@",create:"+",ai:"ai"}[Z.mode()]??">"}),z=ZJ(()=>{return{command:"Run a Ship action…",find:"Jump to a view…",mention:"Reference an agent, view, file…",create:"Create a view, skill, agent…",ai:"Ask Ship or type / for actions"}[Z.mode()]??"Search…"}),G=ZJ(()=>Z.session()?.agent_id??"No session"),F=ZJ(()=>{let A=Z.session();if(A?.status==="thinking")return"var(--warning,#d97706)";if(A)return"var(--success,#16a34a)";return"var(--muted-foreground)"}),O=ZJ(()=>Z.allSessions().filter((A)=>A.status!=="ended"&&A.agent_id));function I(A){if(A==="view")return"app-window";if(A==="agent")return"bot";return"message-square-plus"}let H=ZJ(()=>{let A=Z.value().replace(/^>+\s*/,"").toLowerCase().trim();return vJ.filter((f)=>!A||f.label.toLowerCase().includes(A)||f.keywords.toLowerCase().includes(A)||f.path.toLowerCase().includes(A)).map((f,P)=>({...f,idx:P}))}),w=ZJ(()=>{let A=Z.value().replace(/^>+\s*/,"").toLowerCase().trim(),f=H().length,P=new Set(vJ.map((S)=>S.path));return Z.views().filter((S)=>!P.has("/"+S)).filter((S)=>!A||S.toLowerCase().includes(A)).map((S,r)=>({name:S,label:S.replace(/-/g," ").replace(/\b\w/g,(o)=>o.toUpperCase()),path:"/"+S,icon:"app-window",idx:f+r}))}),$=ZJ(()=>{let A=Z.value().replace(/^\/+\s*/,"").toLowerCase().trim();return aJ.filter((f)=>!A||f.key.includes(A)||f.label.toLowerCase().includes(A)||f.hint.toLowerCase().includes(A)).map((f,P)=>({...f,idx:P}))}),C=ZJ(()=>{let A=Z.value().replace(/^@+/,"").toLowerCase().trim();return[...Z.views().map((P)=>({kind:"view",id:P,label:P})),...Z.agents().map((P)=>({kind:"agent",id:P.id,label:P.id})),...Z.allSessions().filter((P)=>P.status!=="ended"&&P.agent_id).map((P)=>({kind:"session",id:P.id,label:`${P.agent_id} (session)`}))].filter((P)=>!A||P.id.toLowerCase().includes(A)).slice(0,40).map((P,S)=>({...P,idx:S,icon:I(P.kind)}))}),k=ZJ(()=>rJ),n=ZJ(()=>rJ.find((A)=>A.id===Z.createSurface())),l=ZJ(()=>y0(Z.createName()));function a(A){fetch(`/api/v1/sessions/${A}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}).then(()=>{if(Z.session()?.id===A)Z.session.set(null);QJ()}),Z.sessionPickerOpen.set(!1)}function WJ(A){let f=Z.allSessions().find((P)=>P.id===A);if(f)Z.session.set(f),Z.thread.set([]);Z.sessionPickerOpen.set(!1)}function c(){Z.sessionPickerOpen.set(!1),Z.mode.set("ai"),Z.session.set(null)}function GJ(){Z.sessionPickerOpen.set(!1),KJ("/chats"),Z.open.set(!1)}let zJ=X;function d(A){zJ?.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:"find",item:{path:A}},bubbles:!0,composed:!0})),KJ(A),Z.value.set(""),Z.open.set(!1)}function p(A){switch(Z.value.set(""),A){case"new":Z.mode.set("create");break;case"chats":KJ("/chats"),Z.open.set(!1);break;case"settings":KJ("/settings"),Z.open.set(!1);break;case"end":if(Z.session())a(Z.session().id);Z.open.set(!1);break;case"close":Z.open.set(!1);break}}function JJ(A){let f=A.kind==="view"?`@${A.id}`:`@${A.kind}:${A.id}`,P=Z.value();Z.value.set(P.replace(/@[^@\s]*$/,`${f} `)),Z.mode.set(uJ(Z.value(),Z.mode()))}async function QJ(){try{let A=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(A.ok){let f=await A.json();Z.allSessions.set(f.sessions);let P=f.sessions.find((S)=>S.status!=="ended"&&S.agent_id);if(P&&P.id!==Z.session()?.id)Z.session.set(P),Z.thread.set([]);else if(!P)Z.session.set(null)}}catch{}}return{bar:Z,modeGlyph:Q,searchPlaceholder:z,chipLabel:G,chipDotBg:F,activeSessions:O,findItems:H,viewItems:w,cmdItems:$,mentionItems:C,createSurfaces:k,currentSurfaceDef:n,createNameValid:l,endSession:a,selectSession:WJ,startNewSession:c,viewChats:GJ,activateNavItem:d,activateCmd:p,activateMention:JJ}},on:{onInput:(J,X)=>{let Z=X.target.value;J.bar.value.set(Z),J.bar.mode.set(uJ(Z,J.bar.mode())),J.bar.selectedIdx.set(0)},toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},selectSurface:(J,X)=>{J.bar.createSurface.set(X);let Z=rJ.find((Q)=>Q.id===X);if(Z?.shapes[0])J.bar.createShape.set(Z.shapes[0])},spawnSession:async(J)=>{let X=J.bar.spawnAgent(),Z=J.bar.spawnProvider();if(!X||!Z)return;J.bar.spawning.set(!0),J.bar.spawnError.set(null);try{let Q=await fetch("/api/v1/sessions",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({agent_id:X,provider:Z})});if(!Q.ok){let z=await Q.json().catch(()=>({}));J.bar.spawnError.set(z.error??`HTTP ${Q.status}`)}else J.bar.mode.set("ai")}catch(Q){J.bar.spawnError.set(Q instanceof Error?Q.message:"Unknown error")}finally{J.bar.spawning.set(!1)}},submitCreate:async(J)=>{if(!J.createNameValid()||J.bar.createSubmitting())return;J.bar.createSubmitting.set(!0),J.bar.createError.set(null);try{let X=await fetch("/api/v1/scaffold",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({surface:J.bar.createSurface(),name:J.bar.createName(),shape:J.bar.createShape(),purpose:J.bar.createPurpose()})});if(!X.ok){let Z=await X.json().catch(()=>({}));J.bar.createError.set(Z.error??`HTTP ${X.status}`)}else J.bar.createName.set(""),J.bar.createPurpose.set(""),J.bar.open.set(!1)}catch(X){J.bar.createError.set(X instanceof Error?X.message:"Unknown error")}finally{J.bar.createSubmitting.set(!1)}}}});var yQ=OJ("sui-command-palette",{shadow:!0,styles:`
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
`,setup(J){let{props:X,host:Z}=J;if(!J.cleanups)J.cleanups=[];let Q=J.cleanups,z=Object.create(null),G=null;if(tJ&&typeof tJ.setup==="function"){if(J.scope=z,G=tJ.setup(J),G&&typeof G==="object")Object.assign(z,G)}let F=(E)=>DJ({anchor:E,items:()=>z.activeSessions(),key:(j,L)=>{let W=Object.create(z);return W.s=j,W.__i=L,((Y)=>j.id)(W)},render:(j,L)=>{let W=Object.create(z);return W.s=j,W.__i=L,W.__v=x(0),((Y)=>{let K=document.createDocumentFragment(),N=[],V=document.createElement("div");V.setAttribute("class","cb-session-row"),N.push(AJ(V,"current",()=>(Y.__v(),Y.bar.session()?.id===Y.s.id))),V.setAttribute("role","option"),N.push(e(V,"click",function(){let qJ=()=>Y.selectSession(Y.s.id);return typeof qJ==="function"?qJ.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("span");T.setAttribute("class","cb-dot"),N.push(h(()=>{Y.__v();let qJ=Y.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(qJ==null)T.style.removeProperty("background");else T.style.setProperty("background",String(qJ))})),V.appendChild(T);let M=document.createElement("span"),U=document.createElement("strong"),q=document.createTextNode("");U.appendChild(q),N.push(VJ(q,()=>(Y.__v(),Y.s.agent_id??""))),M.appendChild(U);let R=document.createElement("span");R.setAttribute("class","meta");let D=document.createTextNode(" · ");R.appendChild(D);let u=document.createTextNode("");R.appendChild(u),N.push(VJ(u,()=>(Y.__v(),Y.s.provider??"")));let B=document.createTextNode(" · ");R.appendChild(B);let YJ=document.createTextNode("");R.appendChild(YJ),YJ.textContent=Y.s.workspace_id,M.appendChild(R),V.appendChild(M);let LJ=document.createElement("button");LJ.setAttribute("class","cb-session-end"),LJ.setAttribute("type","button"),LJ.setAttribute("aria-label","End session"),N.push(e(LJ,"click",function(){let qJ=(HJ)=>{HJ.stopPropagation(),Y.endSession(Y.s.id)};return typeof qJ==="function"?qJ.bind(void 0):()=>{}}(),{modifiers:[]}));let NJ=document.createTextNode("×");return LJ.appendChild(NJ),V.appendChild(LJ),K.appendChild(V),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let qJ of N)try{qJ()}catch(HJ){}},update:(qJ,HJ)=>{Y.s=qJ,Y.__i=HJ,YJ.textContent=Y.s.workspace_id,Y.__v.set((hJ)=>hJ+1)}}})(W)}}),O=(E)=>MJ({anchor:E,test:()=>{if(z.activeSessions().length===0)return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-session-row");let Y=document.createElement("span");W.appendChild(Y);let K=document.createElement("span");K.setAttribute("class","meta");let N=document.createTextNode("No active sessions");K.appendChild(N),W.appendChild(K);let V=document.createElement("span");return W.appendChild(V),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let T of L)try{T()}catch(M){}}}}]}),I=(E)=>MJ({anchor:E,test:()=>{if(z.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-session-popover"),W.setAttribute("role","listbox"),W.setAttribute("aria-label","Sessions");let Y=document.createComment("for");W.appendChild(Y),L.push(F(Y));let K=document.createComment("if");W.appendChild(K),L.push(O(K));let N=document.createElement("div");N.setAttribute("class","cb-session-sep"),N.setAttribute("role","separator"),W.appendChild(N);let V=document.createElement("div");V.setAttribute("class","cb-session-action"),L.push(e(V,"click",function(){let B=z.startNewSession;return typeof B==="function"?B.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");T.setAttribute("name","plus"),T.setAttribute("size","sm"),T.setAttribute("aria-hidden","true"),V.appendChild(T);let M=document.createElement("span"),U=document.createTextNode("Start new session");M.appendChild(U),V.appendChild(M),W.appendChild(V);let q=document.createElement("div");q.setAttribute("class","cb-session-action"),L.push(e(q,"click",function(){let B=z.viewChats;return typeof B==="function"?B.bind(void 0):()=>{}}(),{modifiers:[]}));let R=document.createElement("sui-icon");R.setAttribute("name","columns"),R.setAttribute("size","sm"),R.setAttribute("aria-hidden","true"),q.appendChild(R);let D=document.createElement("span"),u=document.createTextNode("View all chats");return D.appendChild(u),q.appendChild(D),W.appendChild(q),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let B of L)try{B()}catch(YJ){}}}}]}),H=(E)=>DJ({anchor:E,items:()=>z.findItems(),key:(j,L)=>{let W=Object.create(z);return W.item=j,W.__i=L,((Y)=>j.path)(W)},render:(j,L)=>{let W=Object.create(z);return W.item=j,W.__i=L,W.__v=x(0),((Y)=>{let K=document.createDocumentFragment(),N=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),N.push(FJ(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false")));{let D=Y.item.path;if(D==null||D===!1||D==="")V.removeAttribute("data-path");else V.setAttribute("data-path",D===!0?"":String(D))}N.push(e(V,"click",function(){let D=()=>Y.activateNavItem(Y.item.path);return typeof D==="function"?D.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");{let D=Y.item.icon;if(D==null||D===!1||D==="")T.removeAttribute("name");else T.setAttribute("name",D===!0?"":String(D))}T.setAttribute("size","sm"),T.setAttribute("aria-hidden","true"),V.appendChild(T);let M=document.createElement("span"),U=document.createTextNode("");M.appendChild(U),U.textContent=Y.item.label,V.appendChild(M);let q=document.createElement("span");q.setAttribute("class","hint");let R=document.createTextNode("");return q.appendChild(R),R.textContent=Y.item.path,V.appendChild(q),K.appendChild(V),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let D of N)try{D()}catch(u){}},update:(D,u)=>{Y.item=D,Y.__i=u;{let B=Y.item.path;if(B==null||B===!1||B==="")V.removeAttribute("data-path");else V.setAttribute("data-path",B===!0?"":String(B))}{let B=Y.item.icon;if(B==null||B===!1||B==="")T.removeAttribute("name");else T.setAttribute("name",B===!0?"":String(B))}U.textContent=Y.item.label,R.textContent=Y.item.path,Y.__v.set((B)=>B+1)}}})(W)}}),w=(E)=>MJ({anchor:E,test:()=>{if(z.findItems().length>0)return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Jump to");W.appendChild(Y),j.appendChild(W);let K=document.createElement("div");K.setAttribute("class","cb-list"),K.setAttribute("role","listbox");let N=document.createComment("for");return K.appendChild(N),L.push(H(N)),j.appendChild(K),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(T){}}}}]}),$=(E)=>DJ({anchor:E,items:()=>z.viewItems(),key:(j,L)=>{let W=Object.create(z);return W.item=j,W.__i=L,((Y)=>j.path)(W)},render:(j,L)=>{let W=Object.create(z);return W.item=j,W.__i=L,W.__v=x(0),((Y)=>{let K=document.createDocumentFragment(),N=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),N.push(FJ(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false")));{let D=Y.item.path;if(D==null||D===!1||D==="")V.removeAttribute("data-path");else V.setAttribute("data-path",D===!0?"":String(D))}N.push(e(V,"click",function(){let D=()=>Y.activateNavItem(Y.item.path);return typeof D==="function"?D.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");{let D=Y.item.icon;if(D==null||D===!1||D==="")T.removeAttribute("name");else T.setAttribute("name",D===!0?"":String(D))}T.setAttribute("size","sm"),T.setAttribute("aria-hidden","true"),V.appendChild(T);let M=document.createElement("span"),U=document.createTextNode("");M.appendChild(U),U.textContent=Y.item.label,V.appendChild(M);let q=document.createElement("span");q.setAttribute("class","hint");let R=document.createTextNode("");return q.appendChild(R),R.textContent=Y.item.path,V.appendChild(q),K.appendChild(V),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let D of N)try{D()}catch(u){}},update:(D,u)=>{Y.item=D,Y.__i=u;{let B=Y.item.path;if(B==null||B===!1||B==="")V.removeAttribute("data-path");else V.setAttribute("data-path",B===!0?"":String(B))}{let B=Y.item.icon;if(B==null||B===!1||B==="")T.removeAttribute("name");else T.setAttribute("name",B===!0?"":String(B))}U.textContent=Y.item.label,R.textContent=Y.item.path,Y.__v.set((B)=>B+1)}}})(W)}}),C=(E)=>MJ({anchor:E,test:()=>{if(z.viewItems().length>0)return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Views");W.appendChild(Y),j.appendChild(W);let K=document.createElement("div");K.setAttribute("class","cb-list"),K.setAttribute("role","listbox");let N=document.createComment("for");return K.appendChild(N),L.push($(N)),j.appendChild(K),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(T){}}}}]}),k=(E)=>MJ({anchor:E,test:()=>{if(z.findItems().length===0&&z.viewItems().length===0)return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No matches.");return W.appendChild(Y),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let K of L)try{K()}catch(N){}}}}]}),n=(E)=>DJ({anchor:E,items:()=>z.cmdItems(),key:(j,L)=>{let W=Object.create(z);return W.item=j,W.__i=L,((Y)=>j.key)(W)},render:(j,L)=>{let W=Object.create(z);return W.item=j,W.__i=L,W.__v=x(0),((Y)=>{let K=document.createDocumentFragment(),N=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),N.push(FJ(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false")));{let D=Y.item.key;if(D==null||D===!1||D==="")V.removeAttribute("data-cmd");else V.setAttribute("data-cmd",D===!0?"":String(D))}N.push(e(V,"click",function(){let D=()=>Y.activateCmd(Y.item.key);return typeof D==="function"?D.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("span");T.setAttribute("style","font-family:var(--font-mono)");let M=document.createTextNode("");T.appendChild(M),M.textContent=Y.item.label,V.appendChild(T);let U=document.createElement("span");V.appendChild(U);let q=document.createElement("span");q.setAttribute("class","hint");let R=document.createTextNode("");return q.appendChild(R),R.textContent=Y.item.hint,V.appendChild(q),K.appendChild(V),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let D of N)try{D()}catch(u){}},update:(D,u)=>{Y.item=D,Y.__i=u;{let B=Y.item.key;if(B==null||B===!1||B==="")V.removeAttribute("data-cmd");else V.setAttribute("data-cmd",B===!0?"":String(B))}M.textContent=Y.item.label,R.textContent=Y.item.hint,Y.__v.set((B)=>B+1)}}})(W)}}),l=(E)=>MJ({anchor:E,test:()=>{if(z.cmdItems().length>0)return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Actions");W.appendChild(Y),j.appendChild(W);let K=document.createElement("div");K.setAttribute("class","cb-list"),K.setAttribute("role","listbox");let N=document.createComment("for");return K.appendChild(N),L.push(n(N)),j.appendChild(K),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(T){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No commands match.");return W.appendChild(Y),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let K of L)try{K()}catch(N){}}}}]}),a=(E)=>DJ({anchor:E,items:()=>z.mentionItems(),key:(j,L)=>{let W=Object.create(z);return W.item=j,W.__i=L,((Y)=>j.id+j.kind)(W)},render:(j,L)=>{let W=Object.create(z);return W.item=j,W.__i=L,W.__v=x(0),((Y)=>{let K=document.createDocumentFragment(),N=[],V=document.createElement("div");V.setAttribute("class","cb-item"),V.setAttribute("role","option"),N.push(FJ(V,"aria-selected",()=>(Y.__v(),Y.bar.selectedIdx()===Y.item.idx?"true":"false")));{let D=Y.item.kind;if(D==null||D===!1||D==="")V.removeAttribute("data-mention-kind");else V.setAttribute("data-mention-kind",D===!0?"":String(D))}{let D=Y.item.id;if(D==null||D===!1||D==="")V.removeAttribute("data-mention-id");else V.setAttribute("data-mention-id",D===!0?"":String(D))}N.push(e(V,"click",function(){let D=()=>Y.activateMention(Y.item);return typeof D==="function"?D.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");{let D=Y.item.icon;if(D==null||D===!1||D==="")T.removeAttribute("name");else T.setAttribute("name",D===!0?"":String(D))}T.setAttribute("size","sm"),T.setAttribute("aria-hidden","true"),V.appendChild(T);let M=document.createElement("span"),U=document.createTextNode("");M.appendChild(U),U.textContent=Y.item.label,V.appendChild(M);let q=document.createElement("span");q.setAttribute("class","hint");let R=document.createTextNode("");return q.appendChild(R),R.textContent=Y.item.kind,V.appendChild(q),K.appendChild(V),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let D of N)try{D()}catch(u){}},update:(D,u)=>{Y.item=D,Y.__i=u;{let B=Y.item.kind;if(B==null||B===!1||B==="")V.removeAttribute("data-mention-kind");else V.setAttribute("data-mention-kind",B===!0?"":String(B))}{let B=Y.item.id;if(B==null||B===!1||B==="")V.removeAttribute("data-mention-id");else V.setAttribute("data-mention-id",B===!0?"":String(B))}{let B=Y.item.icon;if(B==null||B===!1||B==="")T.removeAttribute("name");else T.setAttribute("name",B===!0?"":String(B))}U.textContent=Y.item.label,R.textContent=Y.item.kind,Y.__v.set((B)=>B+1)}}})(W)}}),WJ=(E)=>MJ({anchor:E,test:()=>{if(z.mentionItems().length>0)return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Reference");W.appendChild(Y),j.appendChild(W);let K=document.createElement("div");K.setAttribute("class","cb-list"),K.setAttribute("role","listbox");let N=document.createComment("for");return K.appendChild(N),L.push(a(N)),j.appendChild(K),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let V of L)try{V()}catch(T){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-empty");let Y=document.createTextNode("No matching views or agents.");return W.appendChild(Y),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let K of L)try{K()}catch(N){}}}}]}),c=(E)=>DJ({anchor:E,items:()=>z.createSurfaces(),key:(j,L)=>{let W=Object.create(z);return W.surf=j,W.__i=L,((Y)=>j.id)(W)},render:(j,L)=>{let W=Object.create(z);return W.surf=j,W.__i=L,W.__v=x(0),((Y)=>{let K=document.createDocumentFragment(),N=[],V=document.createElement("button");V.setAttribute("class","cb-surface-btn"),V.setAttribute("type","button"),N.push(FJ(V,"aria-pressed",()=>(Y.__v(),Y.bar.createSurface()===Y.surf.id?"true":"false"))),N.push(e(V,"click",function(){let q=()=>Y.selectSurface(Y.surf.id);return typeof q==="function"?q.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");{let q=Y.surf.icon;if(q==null||q===!1||q==="")T.removeAttribute("name");else T.setAttribute("name",q===!0?"":String(q))}T.setAttribute("aria-hidden","true"),V.appendChild(T);let M=document.createElement("span"),U=document.createTextNode("");return M.appendChild(U),U.textContent=Y.surf.label,V.appendChild(M),K.appendChild(V),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let q of N)try{q()}catch(R){}},update:(q,R)=>{Y.surf=q,Y.__i=R;{let D=Y.surf.icon;if(D==null||D===!1||D==="")T.removeAttribute("name");else T.setAttribute("name",D===!0?"":String(D))}U.textContent=Y.surf.label,Y.__v.set((D)=>D+1)}}})(W)}}),GJ=(E)=>DJ({anchor:E,items:()=>z.currentSurfaceDef().shapes,key:(j,L)=>{let W=Object.create(z);return W.sh=j,W.__i=L,((Y)=>j)(W)},render:(j,L)=>{let W=Object.create(z);return W.sh=j,W.__i=L,W.__v=x(0),((Y)=>{let K=document.createDocumentFragment(),N=[],V=document.createElement("option");{let M=Y.sh;if(M==null||M===!1||M==="")V.removeAttribute("value");else V.setAttribute("value",M===!0?"":String(M))}let T=document.createTextNode("");return V.appendChild(T),T.textContent=Y.sh,K.appendChild(V),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let M of N)try{M()}catch(U){}},update:(M,U)=>{Y.sh=M,Y.__i=U;{let q=Y.sh;if(q==null||q===!1||q==="")V.removeAttribute("value");else V.setAttribute("value",q===!0?"":String(q))}T.textContent=Y.sh,Y.__v.set((q)=>q+1)}}})(W)}}),zJ=(E)=>MJ({anchor:E,test:()=>{if(z.bar.createError())return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-create-error");let Y=document.createTextNode("");return W.appendChild(Y),L.push(VJ(Y,()=>z.bar.createError())),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let K of L)try{K()}catch(N){}}}}]}),d=(E)=>DJ({anchor:E,items:()=>z.bar.agents(),key:(j,L)=>{let W=Object.create(z);return W.a=j,W.__i=L,((Y)=>j.id)(W)},render:(j,L)=>{let W=Object.create(z);return W.a=j,W.__i=L,W.__v=x(0),((Y)=>{let K=document.createDocumentFragment(),N=[],V=document.createElement("option");{let M=Y.a.id;if(M==null||M===!1||M==="")V.removeAttribute("value");else V.setAttribute("value",M===!0?"":String(M))}let T=document.createTextNode("");return V.appendChild(T),N.push(VJ(T,()=>(Y.__v(),Y.a.name??Y.a.label??Y.a.id))),K.appendChild(V),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let M of N)try{M()}catch(U){}},update:(M,U)=>{Y.a=M,Y.__i=U;{let q=Y.a.id;if(q==null||q===!1||q==="")V.removeAttribute("value");else V.setAttribute("value",q===!0?"":String(q))}Y.__v.set((q)=>q+1)}}})(W)}}),p=(E)=>MJ({anchor:E,test:()=>{if(z.bar.agents().length===0)return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("option");W.setAttribute("value","");let Y=document.createTextNode("No agents found");return W.appendChild(Y),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let K of L)try{K()}catch(N){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createComment("for");return j.appendChild(W),L.push(d(W)),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(K){}}}}]}),JJ=(E)=>DJ({anchor:E,items:()=>z.bar.providers(),key:(j,L)=>{let W=Object.create(z);return W.p=j,W.__i=L,((Y)=>j)(W)},render:(j,L)=>{let W=Object.create(z);return W.p=j,W.__i=L,W.__v=x(0),((Y)=>{let K=document.createDocumentFragment(),N=[],V=document.createElement("option");{let M=Y.p;if(M==null||M===!1||M==="")V.removeAttribute("value");else V.setAttribute("value",M===!0?"":String(M))}let T=document.createTextNode("");return V.appendChild(T),T.textContent=Y.p,K.appendChild(V),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let M of N)try{M()}catch(U){}},update:(M,U)=>{Y.p=M,Y.__i=U;{let q=Y.p;if(q==null||q===!1||q==="")V.removeAttribute("value");else V.setAttribute("value",q===!0?"":String(q))}T.textContent=Y.p,Y.__v.set((q)=>q+1)}}})(W)}}),QJ=(E)=>MJ({anchor:E,test:()=>{if(z.bar.providers().length===0)return 0;return 1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("option");W.setAttribute("value","");let Y=document.createTextNode("No providers installed");return W.appendChild(Y),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let K of L)try{K()}catch(N){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createComment("for");return j.appendChild(W),L.push(JJ(W)),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(K){}}}}]}),A=(E)=>MJ({anchor:E,test:()=>{if(z.bar.spawnError())return 0;return-1},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-spawn-error");let Y=document.createTextNode("");return W.appendChild(Y),L.push(VJ(Y,()=>z.bar.spawnError())),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let K of L)try{K()}catch(N){}}}}]}),f=(E)=>MJ({anchor:E,test:()=>{let j=z.bar.mode();if(j==="find")return 0;if(j==="command")return 1;if(j==="mention")return 2;if(j==="create")return 3;return 4},branches:[()=>{let j=document.createDocumentFragment(),L=[],W=document.createComment("if");j.appendChild(W),L.push(w(W));let Y=document.createComment("if");j.appendChild(Y),L.push(C(Y));let K=document.createComment("if");return j.appendChild(K),L.push(k(K)),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let N of L)try{N()}catch(V){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createComment("if");return j.appendChild(W),L.push(l(W)),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(K){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createComment("if");return j.appendChild(W),L.push(WJ(W)),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let Y of L)try{Y()}catch(K){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-mode-hint");let Y=document.createTextNode("Create");W.appendChild(Y),j.appendChild(W);let K=document.createElement("div");K.setAttribute("class","cb-create-surfaces"),K.setAttribute("role","group"),K.setAttribute("aria-label","Surface type");let N=document.createComment("for");K.appendChild(N),L.push(c(N)),j.appendChild(K);let V=document.createElement("div");V.setAttribute("class","cb-create-grid");let T=document.createElement("label"),M=document.createElement("span"),U=document.createTextNode("Name (kebab-case)");M.appendChild(U),T.appendChild(M);let q=document.createElement("input");q.setAttribute("type","text"),q.setAttribute("class","cb-create-name"),L.push(FJ(q,"placeholder",()=>"my-"+z.bar.createSurface())),L.push(PJ(q,z.bar.createName)),q.setAttribute("autocomplete","off"),T.appendChild(q),V.appendChild(T);let R=document.createElement("label"),D=document.createElement("span"),u=document.createTextNode("");D.appendChild(u),L.push(VJ(u,()=>z.currentSurfaceDef().shapeLabel)),R.appendChild(D);let B=document.createElement("select");B.setAttribute("class","cb-create-shape"),L.push(PJ(B,z.bar.createShape));let YJ=document.createComment("for");B.appendChild(YJ),L.push(GJ(YJ)),R.appendChild(B),V.appendChild(R);let LJ=document.createElement("label");LJ.setAttribute("class","span-2");let NJ=document.createElement("span"),qJ=document.createTextNode("Purpose (optional)");NJ.appendChild(qJ),LJ.appendChild(NJ);let HJ=document.createElement("input");HJ.setAttribute("type","text"),HJ.setAttribute("class","cb-create-purpose"),HJ.setAttribute("placeholder","One-line description"),L.push(PJ(HJ,z.bar.createPurpose)),HJ.setAttribute("autocomplete","off"),LJ.appendChild(HJ),V.appendChild(LJ),j.appendChild(V);let hJ=document.createComment("if");j.appendChild(hJ),L.push(zJ(hJ));let _J=document.createElement("div");_J.setAttribute("class","cb-create-actions");let EJ=document.createElement("button");EJ.setAttribute("class","cb-create-submit"),EJ.setAttribute("type","button"),L.push(FJ(EJ,"disabled",()=>z.bar.createSubmitting()||!z.createNameValid())),L.push(e(EJ,"click",function(){let dJ=z.submitCreate;return typeof dJ==="function"?dJ.bind(void 0):()=>{}}(),{modifiers:[]}));let F0=document.createTextNode("");return EJ.appendChild(F0),L.push(VJ(F0,()=>z.bar.createSubmitting()?"Creating…":"Create "+z.currentSurfaceDef().label)),_J.appendChild(EJ),j.appendChild(_J),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let dJ of L)try{dJ()}catch(VX){}}}},()=>{let j=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-spawn-card");let Y=document.createElement("div");Y.setAttribute("class","cb-spawn-title");let K=document.createTextNode("Start a session");Y.appendChild(K),W.appendChild(Y);let N=document.createElement("div");N.setAttribute("class","cb-spawn-row");let V=document.createElement("label"),T=document.createTextNode("Agent");V.appendChild(T),N.appendChild(V);let M=document.createElement("select");M.setAttribute("class","cb-spawn-agent"),L.push(PJ(M,z.bar.spawnAgent));let U=document.createComment("if");M.appendChild(U),L.push(p(U)),N.appendChild(M),W.appendChild(N);let q=document.createElement("div");q.setAttribute("class","cb-spawn-row");let R=document.createElement("label"),D=document.createTextNode("Provider");R.appendChild(D),q.appendChild(R);let u=document.createElement("select");u.setAttribute("class","cb-spawn-provider"),L.push(PJ(u,z.bar.spawnProvider));let B=document.createComment("if");u.appendChild(B),L.push(QJ(B)),q.appendChild(u),W.appendChild(q);let YJ=document.createComment("if");W.appendChild(YJ),L.push(A(YJ));let LJ=document.createElement("button");LJ.setAttribute("class","cb-spawn-start"),LJ.setAttribute("type","button"),L.push(FJ(LJ,"disabled",()=>z.bar.spawning()||!z.bar.spawnAgent()||!z.bar.spawnProvider())),L.push(e(LJ,"click",function(){let qJ=z.spawnSession;return typeof qJ==="function"?qJ.bind(void 0):()=>{}}(),{modifiers:[]}));let NJ=document.createTextNode("");return LJ.appendChild(NJ),L.push(VJ(NJ,()=>z.bar.spawning()?"Starting…":"Start session")),W.appendChild(LJ),j.appendChild(W),{nodes:Array.from(j.childNodes),cleanup:()=>{for(let qJ of L)try{qJ()}catch(HJ){}}}}]}),P=Z.shadowRoot??Z;TJ(P);let S=document.createElement("div");S.setAttribute("class","cb-input-row");let r=document.createElement("span");r.setAttribute("class","cb-mode-glyph"),Q.push(AJ(r,"ai",()=>z.bar.mode()==="ai"));let o=document.createTextNode("");r.appendChild(o),Q.push(VJ(o,()=>z.modeGlyph())),S.appendChild(r);let s=document.createElement("input");s.setAttribute("class","cb-search"),s.setAttribute("type","text"),s.setAttribute("autocomplete","off"),s.setAttribute("spellcheck","false"),Q.push(FJ(s,"placeholder",()=>z.searchPlaceholder())),Q.push(PJ(s,z.bar.value)),s.setAttribute("aria-label","Command bar search"),s.setAttribute("aria-autocomplete","list"),Q.push(e(s,"input",function(){let E=z.onInput;return typeof E==="function"?E.bind(void 0):()=>{}}(),{modifiers:[]})),S.appendChild(s);let jJ=document.createElement("kbd");jJ.setAttribute("class","cb-esc"),jJ.setAttribute("aria-label","Escape to close");let kJ=document.createTextNode("esc");jJ.appendChild(kJ),S.appendChild(jJ);let v=document.createElement("div");v.setAttribute("class","cb-chip-wrap");let g=document.createElement("span");g.setAttribute("class","cb-session-chip"),g.setAttribute("role","button"),g.setAttribute("aria-haspopup","listbox"),Q.push(e(g,"click",function(){let E=z.toggleSessionPicker;return typeof E==="function"?E.bind(void 0):()=>{}}(),{modifiers:[]}));let m=document.createElement("span");m.setAttribute("class","cb-dot"),Q.push(h(()=>{let E=z.chipDotBg();if(E==null)m.style.removeProperty("background");else m.style.setProperty("background",String(E))})),g.appendChild(m);let b=document.createElement("span"),y=document.createTextNode("");b.appendChild(y),Q.push(VJ(y,()=>z.chipLabel())),g.appendChild(b),v.appendChild(g);let t=document.createComment("if");v.appendChild(t),Q.push(I(t)),S.appendChild(v),P.appendChild(S);let XJ=document.createElement("div");XJ.setAttribute("class","cb-body");let i=document.createComment("match");if(XJ.appendChild(i),Q.push(f(i)),P.appendChild(XJ),G!=null){let E=Object.getOwnPropertyDescriptor(G,"__shipSetLifecycle")?.value;if(E){let j=new Proxy(z,{get(L,W){return L[W]},set(L,W,Y){let K=L[W];if(K!=null&&typeof K.set==="function"){if(typeof Y==="function")K.set(()=>Y);else K.set(Y);return!0}return!1}});if(E.mount){let L=E.mount(j,Z);if(typeof L==="function")Q.push(L)}if(E.unmount)Q.push(()=>E.unmount(j))}}return z}});
