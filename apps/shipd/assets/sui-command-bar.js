var DJ=null,BJ=null,B0=0,K0=new Set;function Z0(J){if(DJ===null)return;if(J.has(DJ))return;let Z=DJ;if(J.add(Z),BJ!==null)BJ.push(()=>J.delete(Z))}function k(J){let Z=J,X=new Set,z=()=>{return Z0(X),Z};return z.set=(Q)=>{let j=typeof Q==="function"?Q(Z):Q;if(Object.is(j,Z))return;if(Z=j,B0>0)for(let D of X)K0.add(D);else for(let D of[...X])D()},z.__subs=X,z.__peek=()=>Z,z}function XJ(J){let Z,X=!0,z=new Set,Q=[],j=()=>{if(X)return;X=!0;for(let q of[...z])q()},D=()=>{for(let x of Q)x();Q=[];let q=DJ,P=BJ;DJ=j,BJ=Q;try{Z=J()}finally{DJ=q,BJ=P}X=!1},B=()=>{if(Z0(z),X)D();return Z};return B.set=()=>{throw Error("cannot set on a derived; mutate its dependencies instead")},B.__subs=z,B.__peek=()=>{if(X)D();return Z},B}function l(J){let Z=void 0,X=!0,z=[],Q=()=>{if(!X)return;if(Z){try{Z()}catch(B){console.error("[sui] effect cleanup threw:",B)}Z=void 0}for(let B of z)B();z=[];let j=DJ,D=BJ;DJ=Q,BJ=z;try{Z=J()}catch(B){console.error("[sui] effect threw:",B)}finally{DJ=j,BJ=D}};return Q(),()=>{X=!1;for(let j of z)j();if(z=[],Z){try{Z()}catch(j){console.error("[sui] effect cleanup threw:",j)}Z=void 0}}}var bJ=null;function EJ(J,Z){let X=bJ;bJ=J;try{return Z()}finally{bJ=X}}function RJ(J){let Z=bJ;if(!Z)return;Z.cleanup(J)}function AJ(){let J=[],Z=!1;return{cleanup(z){if(Z){try{z()}catch(Q){console.error("[sui/scope] late cleanup threw:",Q)}return}J.push(z)},dispose(){if(Z)return;Z=!0;for(let z=J.length-1;z>=0;z--)try{J[z]()}catch(Q){console.error("[sui/scope] cleanup threw:",Q)}J.length=0},get disposed(){return Z}}}var Q0=Symbol("sui.shared");function o(J){return J[Q0]=!0,J}function z0(J){return!!J?.[Q0]}function iJ(J,Z,X){J[`__sui_ctx_${Z}`]=X}function wJ(J,Z){let X=J;while(X){let z=X[`__sui_ctx_${Z}`];if(z!==void 0)return z;let Q=X.parentElement;if(Q){X=Q;continue}let j=X.getRootNode();X=j instanceof ShadowRoot?j.host:null}return null}function nJ(J){let Z={};for(let X in J)if(Object.prototype.hasOwnProperty.call(J,X)&&z0(J[X]))Z[X]=J[X];return Z}function cJ(J=null){return k(null)}function rJ(J,Z,X){let z=J[Z];if(typeof z==="function"&&typeof z.set==="function")return z.set(X),z;let Q=k(X);return J[Z]=Q,Q}function h(J,Z){return l(()=>{let X=Z();J.textContent=X==null?"":String(X)})}function _(J,Z,X){return l(()=>{let z=X();if(z==null||z===!1||z==="")J.removeAttribute(Z);else if(z===!0)J.setAttribute(Z,"");else J.setAttribute(Z,String(z))})}function CJ(J,Z,X){return l(()=>{if(X())J.classList.add(Z);else J.classList.remove(Z)})}function qJ(J,Z,X){let z=l(()=>{let j=Z();if(J.value!==j)J.value=j}),Q=()=>{let j=J.value;if(Z.set(j),X)X(j)};return J.addEventListener("input",Q),()=>{z(),J.removeEventListener("input",Q)}}function e(J,Z,X,z={}){let Q=z.modifiers??[],j,D=!0,B=(P)=>{for(let I of Q)switch(I.kind){case"preventDefault":P.preventDefault();break;case"stopPropagation":P.stopPropagation();break;case"stopImmediatePropagation":P.stopImmediatePropagation();break;case"self":if(P.target!==J)return;break;case"trusted":if(!P.isTrusted)return;break;case"key":{if(!Y0(P,I.arg??""))return;break}}let x=Q.find((I)=>I.kind==="debounce"),f=Q.find((I)=>I.kind==="throttle");if(x){let I=Number(x.arg)||0;if(j!==void 0)clearTimeout(j);j=setTimeout(()=>{j=void 0,X(P)},I);return}if(f){let I=Number(f.arg)||0;if(!D)return;D=!1,X(P),j=setTimeout(()=>{D=!0},I);return}X(P)},q={};if(z.capture)q.capture=!0;if(z.passive)q.passive=!0;if(z.once)q.once=!0;return J.addEventListener(Z,B,q),()=>{if(J.removeEventListener(Z,B,q),j!==void 0)clearTimeout(j)}}function Y0(J,Z){let X=Z.split("+").map((x)=>x.toLowerCase()),z=X.pop()??"",Q=X.includes("meta")||X.includes("cmd"),j=X.includes("ctrl")||X.includes("control"),D=X.includes("shift"),B=X.includes("alt")||X.includes("option"),q=X.includes("mod"),P=typeof navigator<"u"&&/Mac|iPhone|iPad|iPod/i.test(navigator.platform||navigator.userAgent||"");if(Q&&!J.metaKey)return!1;if(j&&!J.ctrlKey)return!1;if(D&&!J.shiftKey)return!1;if(B&&!J.altKey)return!1;if(q&&!(P?J.metaKey:J.ctrlKey))return!1;return J.key.toLowerCase()===z}function G0(J,Z,X){J.dispatchEvent(new CustomEvent(Z,{detail:X,bubbles:!0,composed:!0}))}function L0(J,Z){let X=J,z=Object.keys(Z);for(let Q of z)X[Q]=Z[Q];RJ(()=>{for(let Q of z)if(X[Q]===Z[Q])try{delete X[Q]}catch{X[Q]=void 0}})}function $J(J){let Z=J;Z.emit=function(X,z){G0(J,X,z)},Z.expose=function(X){L0(J,X)}}function O0(J){let Z=J.length,X=Array(Z).fill(!1);if(Z===0)return X;let z=Array(Z).fill(-1),Q=[];for(let D=0;D<Z;D++){let B=J[D],q=0,P=Q.length;while(q<P){let x=q+P>>>1;if(J[Q[x]]<B)q=x+1;else P=x}if(q>0)z[D]=Q[q-1];Q[q]=D}let j=Q[Q.length-1];while(j>=0)X[j]=!0,j=z[j];return X}function MJ(J){let{anchor:Z,items:X,key:z,render:Q}=J,j=[],D=!0,B=l(()=>{if(!D)return;let q=X(),P=new Map;for(let C=0;C<q.length;C++){let $=q[C],n=z($,C);if(P.has(n))console.warn("[sui] {#for}: duplicate key",n,"at index",C,"(prior at",P.get(n).index,"); the earlier item will be dropped from the rendered output.","Fix the key function so each item maps to a unique value.");P.set(n,{item:$,index:C})}let x=new Map;for(let C of j)x.set(C.key,C);for(let C of j)if(!P.has(C.key)){for(let $ of C.nodes)try{$.parentNode?.removeChild($)}catch{}C.cleanup?.()}let f=[];for(let C=0;C<q.length;C++){let $=q[C],n=z($,C),s=x.get(n),t;if(s)s.item=$,s.update?.($,C),t=s;else{let O=Q($,C);t={key:n,item:$,nodes:O.nodes,...O.cleanup&&{cleanup:O.cleanup},...O.update&&{update:O.update}}}f.push(t)}let I=new Map;for(let C=0;C<j.length;C++)I.set(j[C].key,C);let v=[],JJ=[];for(let C=0;C<f.length;C++){let $=I.get(f[C].key);if($!==void 0)v.push(C),JJ.push($)}let c=O0(JJ),a=new Set;for(let C=0;C<v.length;C++)if(c[C])a.add(v[C]);let ZJ=Z.parentNode,i=Z;for(let C=0;C<f.length;C++){if(!ZJ)break;let $=f[C];if(a.has(C))i=$.nodes[$.nodes.length-1]??i;else for(let n of $.nodes)ZJ.insertBefore(n,i.nextSibling),i=n}j=f});return()=>{D=!1,B();for(let q of j){for(let P of q.nodes)try{P.parentNode?.removeChild(P)}catch{}q.cleanup?.()}j=[]}}function zJ(J){let{anchor:Z}=J,{test:X,branches:z}=N0(J)?{test:J.test,branches:J.branches}:I0(J),Q=null,j=-2,D=()=>{if(!Q)return;for(let q of Q.nodes)try{q.parentNode?.removeChild(q)}catch{}Q.cleanup?.(),Q=null},B=l(()=>{let q=X();if(q===j)return;if(D(),j=q,q<0||q>=z.length){Q=null;return}Q=z[q]();let P=Z.parentNode;if(!P)return;for(let x of Q.nodes)P.insertBefore(x,Z)});return()=>{B(),D()}}function N0(J){return typeof J.test==="function"}function I0(J){let{cond:Z,then_:X,else_:z}=J;return{test:()=>Z()?0:z?1:-1,branches:z?[X,z]:[X]}}var aJ=(J)=>J??"",oJ=(J)=>{if(J===null||J==="")return NaN;return Number(J)},sJ=(J)=>{if(J===null)return!1;if(J.toLowerCase()==="false")return!1;return!0};function fJ(J){return(Z)=>{if(Z===null||Z==="")return J;try{return JSON.parse(Z)}catch{return J}}}function tJ(J,Z){let X=new Set(J);return(z)=>z!==null&&X.has(z)?z:Z}var vJ=(J)=>{switch(J.type){case"string":return aJ;case"bool":return sJ;case"number":return oJ;case"union":{let Z=J.values??[],X=J.default??Z[0]??"";return tJ(Z,X)}case"object":return fJ(J.default);case"array":return fJ(J.default??[])}};function j0(J,Z,X,z){if(!Z)return;let Q={},j,D=X.formValue;if(D!==void 0){let{sig:q,propDef:P}=W0(X,z,D,"formValue"),x=P.default,f=vJ(P);j=l(()=>{let I=q();Z.setFormValue(C0(I))}),Q.reset=()=>{q.set(x)},Q.restore=(I)=>{q.set(f(I))}}let B=X.formDisabled;if(B!==void 0){let{sig:q,propDef:P}=W0(X,z,B,"formDisabled");if(P.type!=="bool")throw Error(`[sui-runtime] formDisabled: "${B}" must be a bool-typed prop`);Q.disabled=(x)=>{q.set(!!x)}}if(J.__shipSetFormHooks=Q,j)J.__shipSetFormValueCleanup=j;return j}function M0(J){let Z=J;delete Z.__shipSetFormHooks,delete Z.__shipSetFormValueCleanup}function C0(J){if(J===null||J===void 0)return"";if(typeof J==="boolean")return J?"on":"";return String(J)}function W0(J,Z,X,z){let Q=J.props[X];if(!Q)throw Error(`[sui-runtime] ${z}: "${X}" — no matching prop on this component`);let j=Z[TJ(X)];if(!j)throw Error(`[sui-runtime] ${z}: "${X}" — prop signal missing`);return{sig:j,propDef:Q}}var xJ={string(J={}){return{type:"string",...J}},bool(J={}){return{type:"bool",...J}},number(J={}){return{type:"number",...J}},union(J,Z={}){return{type:"union",values:J,...Z}},object(J={}){return{type:"object",...J}},array(J={}){return{type:"array",...J}}},T0=(J,Z)=>{if(Z===void 0||Z===null)return null;switch(J.type){case"string":return Z===""?null:String(Z);case"number":return Number.isNaN(Z)?null:String(Z);case"bool":return Z?"":null;case"union":return String(Z);case"object":case"array":try{return JSON.stringify(Z)}catch{return null}}},E0=(J,Z)=>{let X=`ship-set-style-${J}`;if(document.getElementById(X))return;let z=document.createElement("style");z.id=X,z.textContent=`[data-component="${J}"] { ${Z} }`,document.head.appendChild(z)},R0=(J,Z,X,z)=>{let Q=J.getAttribute(Z),j=Q!==null?z(Q):X.default;return k(j)},A0=(J)=>{let Z=[],X={};for(let[j,D]of Object.entries(J.props??{}))Z.push(j),X[j]={camel:TJ(j),decode:vJ(D)};let z=J.formAssociated===!0||J.formValue!==void 0||J.formDisabled!==void 0,Q={observedAttrs:Z,props:X};if(z)Q.formAssociated=!0;return Q};function KJ(J,Z){let X=A0(Z),z=Object.entries(Z.props??{});_J(J,X,(Q,j,D)=>{let B=null;if(Z.shadow){if(B=Q.shadowRoot??Q.attachShadow({mode:"open"}),Z.styles){let P=document.createElement("style");P.textContent=Z.styles,B.appendChild(P)}}else if(Z.styles)E0(J,Z.styles);let q=[];for(let[P,x]of z){let f=TJ(P),I=X.props[P].decode,v=R0(Q,P,x,I);if(j[f]=v,x.reflect){let JJ=!0,c=l(()=>{let a=v();if(JJ){JJ=!1;return}let ZJ=T0(x,a);if(ZJ===null)Q.removeAttribute(P);else Q.setAttribute(P,ZJ)});q.push(c)}}if(Z.formValue!==void 0||Z.formDisabled!==void 0){let P=j0(Q,D,Z,j);if(typeof P==="function")q.push(P)}if(Z.setup){let P={props:j,host:Q,cleanups:q};if(D)P.internals=D;Z.setup(P)}return()=>{M0(Q);for(let P of q.splice(0))try{P()}catch(x){console.error(`[ship-set:${J}] cleanup threw:`,x)}}})}var F0=new Set;function _J(J,Z,X){if(customElements.get(J)){if(!F0.has(J))F0.add(J),console.warn(`[sui-runtime] customElements.define("${J}", …) called twice. `+"The first registration wins; this call is a no-op. Likely cause: two modules independently registering the same tag.");return}if(Z.formAssociated){class Q extends HTMLElement{static formAssociated=!0;static get observedAttributes(){return Z.observedAttrs}cleanup;scope;propSignals={};internals;constructor(){super();this.internals=this.attachInternals()}connectedCallback(){this.dataset.component=J,$J(this),this.scope=AJ(),this.cleanup=EJ(this.scope,()=>X(this,this.propSignals,this.internals))}disconnectedCallback(){try{this.cleanup?.()}catch(j){console.error(`[ship-set:${J}] cleanup threw:`,j)}this.cleanup=void 0;try{this.scope?.dispose()}catch(j){console.error(`[ship-set:${J}] scope dispose threw:`,j)}this.scope=void 0}attributeChangedCallback(j,D,B){let q=Z.props[j];if(!q)return;let P=this.propSignals[q.camel];if(P)P.set(q.decode(B))}formResetCallback(){let j=this.__shipSetFormHooks;try{j?.reset?.()}catch(D){console.error(`[ship-set:${J}] formResetCallback threw:`,D)}this.dispatchEvent(new CustomEvent("ship-set:form-reset"))}formStateRestoreCallback(j,D){let B=this.__shipSetFormHooks;try{B?.restore?.(j)}catch(q){console.error(`[ship-set:${J}] formStateRestoreCallback threw:`,q)}this.dispatchEvent(new CustomEvent("ship-set:form-restore",{detail:{state:j,mode:D}}))}formDisabledCallback(j){let D=this.__shipSetFormHooks;try{D?.disabled?.(j)}catch(B){console.error(`[ship-set:${J}] formDisabledCallback threw:`,B)}this.dispatchEvent(new CustomEvent("ship-set:form-disabled",{detail:{disabled:j}}))}}customElements.define(J,Q);return}class z extends HTMLElement{static get observedAttributes(){return Z.observedAttrs}cleanup;scope;propSignals={};connectedCallback(){this.dataset.component=J,$J(this),this.scope=AJ(),this.cleanup=EJ(this.scope,()=>X(this,this.propSignals))}disconnectedCallback(){try{this.cleanup?.()}catch(Q){console.error(`[ship-set:${J}] cleanup threw:`,Q)}this.cleanup=void 0;try{this.scope?.dispose()}catch(Q){console.error(`[ship-set:${J}] scope dispose threw:`,Q)}this.scope=void 0}attributeChangedCallback(Q,j,D){let B=Z.props[Q];if(!B)return;let q=this.propSignals[B.camel];if(q)q.set(B.decode(D))}}customElements.define(J,z)}function TJ(J){return J.replace(/-([a-z])/g,(Z,X)=>X.toUpperCase())}function eJ(J,Z,X){let z=k0(J.state,{ship:Z.ship,...X}),Q={};for(let[I,v]of Object.entries(z))Q[I]=k(v);let j={ship:Z.ship,route:Z.route??{},navigate:Z.navigate??((I)=>{}),...X,...Q},D=S0(Q,j),B={...j},q={};for(let[I,v]of Object.entries(J.derived??{}))q[I]=XJ(()=>v(B));Object.assign(B,q);let P={};for(let[I,v]of Object.entries(J.on??{}))P[I]=(...JJ)=>v(D,...JJ);let x={},f=typeof J.use==="function"?J.use(B):J.use??{};for(let[I,v]of Object.entries(f))x[I]=eJ(v,Z,B);return{...Q,...q,...P,...x}}function k0(J,Z){if(J===void 0)return{};if(typeof J==="function")return J(Z);return J}function S0(J,Z){return new Proxy({},{get(X,z){if(z in J)return J[z]();return Z[z]},set(X,z,Q){if(z in J)return J[z].set(Q),!0;return console.warn(`[ship-set] hook handler tried to write s.${z} = … but ${z} is not a hook state key. The assignment was ignored.`),!0},has(X,z){return z in J||z in Z},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(X,z){if(z in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[z]()};return}})}var V0="__shipSetLifecycle";function H0(J,Z){let X=y0(J,Z),z={ship:Z.ship,route:Z.route??{},navigate:Z.navigate??((I)=>{}),...Z.scope??{}},Q=X.scope?X.scope(z,Z.host):{};Object.assign(z,Q);let j=b0(z,Z.stateKeys,Q),D=$0(j,z),B={...z},q=f0(X,B);Object.assign(B,q);let P=m0(X,B,Z);for(let[I,v]of Object.entries(P))B[I]=v;let x=v0(X,D),f={...Q,...q,...x,...P};if(Object.keys(Q).length>0&&Z.host){let I=nJ(Q);if(Object.keys(I).length>0){let v=Z.host,JJ=v.tagName.toLowerCase();iJ(v,JJ,I)}}if(X.mount||X.unmount){let I={state:D};if(X.mount)I.mount=X.mount;if(X.unmount)I.unmount=X.unmount;Object.defineProperty(f,V0,{value:I,enumerable:!0,configurable:!0,writable:!1})}return f}function y0(J,Z){if(typeof J==="function")return J({ship:Z.ship});return J}function b0(J,Z,X){let z={},Q=Z!==void 0,j=X?Object.keys(X):[];if(Q||j.length>0){if(Z)for(let D of Z){let B=J[D];if(J0(B))z[D]=B}for(let D of j){if(D in z)continue;let B=J[D];if(J0(B))z[D]=B}return z}for(let[D,B]of Object.entries(J))if(J0(B))z[D]=B;return z}function J0(J){return typeof J==="function"&&"set"in J&&typeof J.set==="function"}function $0(J,Z){return new Proxy({},{get(X,z){if(z in J)return J[z]();return Z[z]},set(X,z,Q){if(z in J){if(typeof Q==="function")J[z].set(()=>Q);else J[z].set(Q);return!0}return console.warn(`[ship-set] handler tried to write s.${z} = … but ${z} is not declared as <state name="${z}"> in the template. The assignment was ignored.`),!0},has(X,z){return z in J||z in Z},ownKeys(){return Object.keys(J)},getOwnPropertyDescriptor(X,z){if(z in J)return{configurable:!0,enumerable:!0,writable:!0,value:J[z]()};return}})}function f0(J,Z){let X=J.derived??{},z={};for(let[Q,j]of Object.entries(X))z[Q]=XJ(()=>j(Z));return z}function v0(J,Z){let X=J.on??{},z={};for(let[Q,j]of Object.entries(X))z[Q]=(...D)=>j(Z,...D);return z}function m0(J,Z,X){if(J.use===void 0)return{};let z=typeof J.use==="function"?J.use(Z):J.use,Q={};for(let[j,D]of Object.entries(z))Q[j]=eJ(D,X,Z);return Q}function OJ(J){return{setup:(Z)=>H0(J,Z),__def:J}}if(typeof customElements<"u"){for(let J of["sui-if","sui-for","sui-match","sui-while-loading","sui-while-error"])if(!customElements.get(J))customElements.define(J,class extends HTMLElement{})}var d0=`:host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }
:host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }
:host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }
:host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }
:host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }
:host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }
:host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }
:host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }
`,l0=`:host([hidden]) { display: none !important; }
@media (prefers-reduced-motion: reduce) { * { transition: none !important; animation: none !important; } }
`;function q0(J){if(typeof CSSStyleSheet>"u")return null;try{let Z=new CSSStyleSheet;return Z.replaceSync(J),Z}catch{return null}}var D0=q0(d0),P0=q0(l0);function NJ(J){if(typeof ShadowRoot>"u")return;if(!(J instanceof ShadowRoot))return;let Z=[P0,D0].filter((X)=>X!=null);if(Z.length===0)return;try{J.adoptedStyleSheets=[...Z,...J.adoptedStyleSheets??[]]}catch{}}var mJ=[{id:"view",label:"View",icon:"file",shapes:["list","form","detail","dashboard","tabs","blank"],shapeLabel:"Starting shape"},{id:"skill",label:"Skill",icon:"sparkles",shapes:["task","reference","workflow"],shapeLabel:"Skill kind"},{id:"agent",label:"Agent",icon:"users",shapes:["specialist","generalist","reviewer"],shapeLabel:"Agent kind"},{id:"pipeline",label:"Pipeline",icon:"play",shapes:["sequential","fanout","review-gated"],shapeLabel:"Topology"},{id:"trigger",label:"Trigger",icon:"zap",shapes:["webhook","schedule","file-change"],shapeLabel:"Kind"},{id:"theme",label:"Theme",icon:"palette",shapes:["light","dark","brand-accent","high-contrast"],shapeLabel:"Starting shape"}],gJ=[{label:"Dashboard",path:"/dashboard",icon:"sparkles",keywords:"home overview"},{label:"Pipelines",path:"/pipelines",icon:"play",keywords:"workflows runs jobs"},{label:"Agents",path:"/agents",icon:"users",keywords:"configure profiles"},{label:"Skills",path:"/skills",icon:"zap",keywords:"browse edit files"},{label:"Git",path:"/git",icon:"git",keywords:"status diff commit branch"},{label:"Collections",path:"/collections",icon:"folder",keywords:"data items query"},{label:"Canvas",path:"/canvas",icon:"image",keywords:"docs visual feedback"},{label:"Events",path:"/events",icon:"sparkles",keywords:"stream log kernel"},{label:"Settings",path:"/settings",icon:"settings",keywords:"config theme provider"},{label:"Chat",path:"/chat",icon:"message-square-plus",keywords:"agent conversation"},{label:"Specs",path:"/specs",icon:"file",keywords:"features design"},{label:"Theme Studio",path:"/theme-studio",icon:"palette",keywords:"design tokens"},{label:"Chats (Mission Control)",path:"/chats",icon:"messages-square",keywords:"sessions active"}],uJ=[{key:"new",label:"/new",hint:"Scaffold a view, skill, agent, …"},{key:"chats",label:"/chats",hint:"Open Mission Control"},{key:"settings",label:"/settings",hint:"Jump to settings"},{key:"lint",label:"/lint",hint:"Run lint on the current view"},{key:"end",label:"/end",hint:"End the current session"},{key:"close",label:"/close",hint:"Close the command bar"}],p0=/^[a-z][a-z0-9-]{0,48}$/;function U0(J){return p0.test(J)}function kJ(J,Z){if(J.startsWith("/"))return"command";if(J.startsWith(">"))return"find";if(J.startsWith("+"))return"create";if(J.trim().length===0)return Z;if(/(^|\s)@[^\s@]*$/.test(J))return"mention";return"ai"}async function FJ(J){let Z=window;if(Z.ship?.navigate){Z.ship.navigate(J);return}history.pushState({},"",J),document.dispatchEvent(new CustomEvent("sui:navigate",{detail:{to:J}}))}var hJ=OJ({scope:(J)=>{let Z=J,X=o(Z.open),z=o(Z.mode),Q=o(k("")),j=o(k(null)),D=o(k([])),B=o(k([])),q=o(k([])),P=o(k([])),x=o(k(0)),f=o(k(!1)),I=o(k("")),v=o(k("")),JJ=o(k(!1)),c=o(k(null)),a=o(k([])),ZJ=o(k("main")),i=o(k("view")),C=o(k("")),$=o(k("")),n=o(k("list")),s=o(k(!1)),t=o(k(null)),O=o(XJ(()=>z()==="ai"&&j()!==null)),m=cJ(null);return{open:X,mode:z,value:Q,session:j,agents:D,providers:B,allSessions:q,views:P,selectedIdx:x,sessionPickerOpen:f,spawnAgent:I,spawnProvider:v,spawning:JJ,spawnError:c,thread:a,workspaceId:ZJ,createSurface:i,createName:C,createPurpose:$,createShape:n,createSubmitting:s,createError:t,isChat:O,dialogRef:m}},derived:{triggerLabel:(J)=>{let Z=J.session();return Z?`Ask ${Z.agent_id??"Ship"} or jump to…`:"Ask Ship or jump to…"},dotBg:(J)=>{let Z=J.session();if(Z?.status==="thinking")return"var(--warning,#d97706)";if(Z)return"var(--success,#16a34a)";return"var(--muted-foreground)"}},on:{openFind:(J)=>{J.mode="find",J.value="",J.selectedIdx=0,J.open=!0}},mount:(J,Z)=>{let X=J,z=X.open,Q=X.mode,j=X.value,D=X.session,B=X.agents,q=X.providers,P=X.allSessions,x=X.views,f=X.selectedIdx,I=X.spawnAgent,v=X.spawnProvider,JJ=X.spawning,c=X.spawnError,a=X.thread,ZJ=X.workspaceId,i=X.createSurface,C=X.createName,$=X.createPurpose,n=X.createShape,s=X.createSubmitting,t=X.createError,O=X.dialogRef?.(),m=null,y=0,VJ=l(()=>{let V=z();if(Z.setAttribute("data-state",V?"open":"closed"),V){if(typeof O?.show==="function")O.show();queueMicrotask(()=>{Z.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelector(".cb-search")?.focus()})}else{if(typeof O?.close==="function")O.close();Z.dispatchEvent(new CustomEvent("sui-command:cancel",{detail:{mode:Q()},bubbles:!0,composed:!0}))}});Z.show=(V,U)=>{if(V)Q.set(V);if(typeof U==="string")j.set(U);z.set(!0)},Z.hide=()=>z.set(!1),Z.toggle=(V)=>{if(z())z.set(!1);else{if(V)Q.set(V);z.set(!0)}},Z.setMode=(V)=>Q.set(V);function YJ(V){HJ();let U=new AbortController;m=U;let N=new EventSource(`/api/v1/sessions/${V}/events`,{withCredentials:!0}),E=null;N.addEventListener("provider",(QJ)=>{let r;try{r=JSON.parse(QJ.data)}catch{return}switch(r.kind){case"message":{let WJ=r.text??"";if(!WJ)return;if(E)a.set(a().map((d)=>d.id===E?{...d,text:d.text+WJ}:d));else{let d=String(Date.now()+Math.random());E=d,a.set([...a(),{id:d,sender:"agent",text:WJ}])}break}case"tool_use":a.set([...a(),{id:String(Date.now()+Math.random()),sender:"system",text:`→ tool: ${r.tool??"?"}`}]);break;case"turn_complete":E=null;break}}),N.onerror=()=>{if(!U.signal.aborted)N.close()},U.signal.addEventListener("abort",()=>N.close())}function HJ(){m?.abort(),m=null}let PJ=l(()=>{let V=D();if(V?.id)YJ(V.id);else HJ()});async function GJ(){try{let V=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(V.ok){let U=await V.json();P.set(U.sessions);let N=U.sessions.find((E)=>E.status!=="ended"&&E.agent_id);if(N&&N.id!==D()?.id)D.set(N),a.set([]);else if(!N)D.set(null)}}catch{}if(B().length===0)LJ();if(q().length===0)UJ();if(x().length===0)u()}async function LJ(){try{let V=await fetch("/api/v1/agents",{credentials:"same-origin"});if(!V.ok)return;let N=((await V.json()).items??[]).map((E)=>({id:E.id}));if(B.set(N),!I()&&N.length>0)I.set(N[0].id)}catch{}}async function UJ(){try{let V=await fetch("/api/v1/providers/installed",{credentials:"same-origin"});if(!V.ok)return;let U=await V.json();if(q.set(U.providers),!v()&&U.providers.length>0)v.set(U.providers[0])}catch{}}async function u(){try{let V=await fetch("/api/v1/views",{credentials:"same-origin"});if(!V.ok)return;let U=await V.json(),N=U.data?.views??U.views??[];x.set(N.map((E)=>E.name).sort())}catch{}}function p(){return Z.shadowRoot?.querySelector("sui-command-palette")?.shadowRoot?.querySelectorAll(".cb-item")}function g(V){let U=p();if(!U?.length)return;f.set(Math.max(0,Math.min(U.length-1,f()+V)))}function A(){let V=Q(),U=j(),E=p()?.[f()];if(E){let QJ=E.dataset.path;if(QJ)FJ(QJ),j.set(""),z.set(!1);let r=E.dataset.cmd;if(r)j.set(""),S("/"+r);let WJ=E.dataset.mentionKind,d=E.dataset.mentionId;if(WJ&&d){let jJ=WJ==="view"?`@${d}`:`@${WJ}:${d}`;j.set(U.replace(/@[^@\s]*$/,`${jJ} `)),Q.set(kJ(j(),V))}Z.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:V,item:E.dataset},bubbles:!0,composed:!0}));return}if(V==="command")S(U);else if(V==="find")K(U)}function S(V){let U=V.replace(/^\/+\s*/,"").toLowerCase(),N=uJ.find((E)=>E.key===U||E.key.startsWith(U));if(!N)return;switch(N.key){case"new":Q.set("create"),j.set("");break;case"chats":FJ("/chats"),z.set(!1);break;case"settings":FJ("/settings"),z.set(!1);break;case"end":if(D())L(D().id);z.set(!1);break;case"close":z.set(!1);break;case"lint":W();break}}function K(V){let U=V.replace(/^>+\s*/,"").toLowerCase().trim(),N=gJ.find((E)=>E.label.toLowerCase().includes(U)||E.keywords.toLowerCase().includes(U)||E.path.toLowerCase().includes(U));if(N)FJ(N.path),j.set(""),z.set(!1)}async function L(V){try{await fetch(`/api/v1/sessions/${V}`,{method:"DELETE",credentials:"same-origin"})}catch{}if(D()?.id===V)D.set(null);await GJ()}async function W(){let U=window.ship?.current?.()?.path?.replace(/^\/+/,"").split("/")[0]??"";if(!U)return;try{let N=await fetch("/api/v1/lint/view",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:U})});if(N.ok){let QJ=(await N.json()).data?.findings?.length??0;console.log(QJ===0?`${U}: no lint findings`:`${U}: ${QJ} lint findings`)}}catch{}z.set(!1)}let Y=(V)=>{if(V.key==="Escape"&&z()){z.set(!1);return}if(V.key===" "&&!z()){let U=V.target;if(U&&(U.tagName==="INPUT"||U.tagName==="TEXTAREA"||U.isContentEditable))return;let N=performance.now();if(N-y<350)y=0,V.preventDefault(),Q.set("ai"),j.set(""),z.set(!0);else y=N}},G=(V)=>{if(V.key==="ArrowDown")V.preventDefault(),g(1);else if(V.key==="ArrowUp")V.preventDefault(),g(-1);else if(V.key==="Enter")V.preventDefault(),A()};Z.addEventListener("keydown",G),document.addEventListener("keydown",Y);let H=()=>{if(z())z.set(!1)};O?.addEventListener("sui-close",H);let F=()=>void GJ(),M=(V)=>{let U=V;if(U.detail?.mode)Q.set(U.detail.mode);if(typeof U.detail?.prefill==="string")j.set(U.detail.prefill);z.set(!0)},T=()=>{Q.set("create"),z.set(!0)};window.addEventListener("ship:session:changed",F);let w=document.querySelector("ship-shell")??document.querySelector("sui-shell");w?.addEventListener("ship:bar:open",M),w?.addEventListener("ship:create:open",T);let R=Z["__sui_ctx_sui-command-bar"];if(R)R.endSession=L,R.loadCurrentSession=GJ;return GJ(),()=>{if(document.removeEventListener("keydown",Y),Z.removeEventListener("keydown",G),window.removeEventListener("ship:session:changed",F),w?.removeEventListener("ship:bar:open",M),w?.removeEventListener("ship:create:open",T),O)O.removeEventListener("sui-close",H);PJ(),VJ()}}});var wQ=KJ("sui-command-bar",{props:{open:xJ.bool({default:!1,reflect:!0}),mode:xJ.string({default:"ai",reflect:!0})},shadow:!0,styles:`
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
`,setup(J){let{props:Z,host:X}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null);Q.open=Z.open,Q.mode=Z.mode;let j=null;if(hJ&&typeof hJ.setup==="function"){if(J.scope=Q,j=hJ.setup(J),j&&typeof j==="object")Object.assign(Q,j)}let D=(i)=>zJ({anchor:i,test:()=>{if(Q.isChat())return 0;return 1},branches:[()=>{let C=document.createDocumentFragment(),$=[],n=document.createElement("sui-command-chat");return C.appendChild(n),{nodes:Array.from(C.childNodes),cleanup:()=>{for(let s of $)try{s()}catch(t){}}}},()=>{let C=document.createDocumentFragment(),$=[],n=document.createElement("sui-command-palette");return C.appendChild(n),{nodes:Array.from(C.childNodes),cleanup:()=>{for(let s of $)try{s()}catch(t){}}}}]}),B=X.shadowRoot??X;NJ(B);let q=document.createElement("button");q.setAttribute("class","cb-trigger"),q.setAttribute("type","button"),q.setAttribute("aria-label","Open command bar"),z.push(e(q,"click",function(){let i=Q.openFind;return typeof i==="function"?i.bind(void 0):()=>{}}(),{modifiers:[]}));let P=document.createElement("sui-icon");P.setAttribute("name","search"),P.setAttribute("size","sm"),q.appendChild(P);let x=document.createElement("span");x.setAttribute("class","cb-label");let f=document.createTextNode("");x.appendChild(f),z.push(h(f,()=>Q.triggerLabel())),q.appendChild(x);let I=document.createElement("span");I.setAttribute("class","cb-dot"),z.push(l(()=>{let i=Q.dotBg();if(i==null)I.style.removeProperty("background");else I.style.setProperty("background",String(i))})),q.appendChild(I);let v=document.createElement("kbd"),JJ=document.createTextNode("⌘K");v.appendChild(JJ),q.appendChild(v),B.appendChild(q);let c=document.createElement("sui-dialog");c.setAttribute("class","cb-dialog"),c.setAttribute("no-header",""),c.setAttribute("no-footer",""),c.setAttribute("no-close-button",""),c.setAttribute("backdrop","blur"),c.setAttribute("size","full"),c.setAttribute("style","--dlg-max-w: min(840px, 94vw)"),rJ(Q,"dialogRef",c);let a=document.createElement("div");a.setAttribute("class","cb-wrap");let ZJ=document.createComment("if");if(a.appendChild(ZJ),z.push(D(ZJ)),c.appendChild(a),B.appendChild(c),j!=null){let i=Object.getOwnPropertyDescriptor(j,"__shipSetLifecycle")?.value;if(i){let C=new Proxy(Q,{get($,n){return $[n]},set($,n,s){let t=$[n];if(t!=null&&typeof t.set==="function"){if(typeof s==="function")t.set(()=>s);else t.set(s);return!0}return!1}});if(i.mount){let $=i.mount(C,X);if(typeof $==="function")z.push($)}if(i.unmount)z.push(()=>i.unmount(C))}}return Q}});var dJ=OJ({scope:(J,Z)=>{let X=wJ(Z,"sui-command-bar"),z=XJ(()=>{if(X.session()?.status==="thinking")return"var(--warning,#d97706)";return"var(--success,#16a34a)"}),Q=XJ(()=>X.allSessions().filter((P)=>P.status!=="ended"&&P.agent_id));function j(P){if(fetch(`/api/v1/sessions/${P}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}),X.session()?.id===P)X.session.set(null);X.sessionPickerOpen.set(!1)}function D(P){let x=X.allSessions().find((f)=>f.id===P);if(x)X.session.set(x),X.thread.set([]);X.sessionPickerOpen.set(!1)}function B(){X.sessionPickerOpen.set(!1),X.mode.set("ai"),X.session.set(null)}function q(){X.sessionPickerOpen.set(!1),FJ("/chats"),X.open.set(!1)}return{bar:X,headerDotBg:z,activeSessions:Q,endSession:j,selectSession:D,startNewSession:B,viewChats:q}},on:{toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},sendMessage:async(J)=>{let Z=J.bar.value().trim();if(!Z||!J.bar.session())return;let X=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:Z}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${X}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:Z})})}catch{}},onComposerKeydown:async(J,Z)=>{if(Z.key==="Enter"&&!Z.shiftKey){Z.preventDefault();let X=J.bar.value().trim();if(!X||!J.bar.session())return;let z=J.bar.session().id;J.bar.thread.set([...J.bar.thread(),{id:String(Date.now()+Math.random()),sender:"user",text:X}]),J.bar.value.set("");try{await fetch(`/api/v1/sessions/${z}/message`,{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({text:X})})}catch{}}}}});var fQ=KJ("sui-command-chat",{shadow:!0,styles:`
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
`,setup(J){let{props:Z,host:X}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null),j=null;if(dJ&&typeof dJ.setup==="function"){if(J.scope=Q,j=dJ.setup(J),j&&typeof j==="object")Object.assign(Q,j)}let D=(u)=>MJ({anchor:u,items:()=>Q.activeSessions(),key:(p,g)=>{let A=Object.create(Q);return A.s=p,A.__i=g,((S)=>p.id)(A)},render:(p,g)=>{let A=Object.create(Q);return A.s=p,A.__i=g,A.__v=k(0),((S)=>{let K=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-session-row"),L.push(CJ(W,"current",()=>(S.__v(),S.bar.session()?.id===S.s.id))),W.setAttribute("role","option"),L.push(e(W,"click",function(){let N=()=>S.selectSession(S.s.id);return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]}));let Y=document.createElement("span");Y.setAttribute("class","cb-dot"),L.push(l(()=>{S.__v();let N=S.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(N==null)Y.style.removeProperty("background");else Y.style.setProperty("background",String(N))})),W.appendChild(Y);let G=document.createElement("span"),H=document.createElement("strong"),F=document.createTextNode("");H.appendChild(F),L.push(h(F,()=>(S.__v(),S.s.agent_id??""))),G.appendChild(H);let M=document.createElement("span");M.setAttribute("class","meta");let T=document.createTextNode(" · ");M.appendChild(T);let w=document.createTextNode("");M.appendChild(w),L.push(h(w,()=>(S.__v(),S.s.provider??"")));let b=document.createTextNode(" · ");M.appendChild(b);let R=document.createTextNode("");M.appendChild(R),L.push(h(R,()=>(S.__v(),S.s.workspace_id))),G.appendChild(M),W.appendChild(G);let V=document.createElement("button");V.setAttribute("class","cb-session-end"),V.setAttribute("type","button"),V.setAttribute("aria-label","End session"),L.push(e(V,"click",function(){let N=(E)=>{E.stopPropagation(),S.endSession(S.s.id)};return typeof N==="function"?N.bind(void 0):()=>{}}(),{modifiers:[]}));let U=document.createTextNode("×");return V.appendChild(U),W.appendChild(V),K.appendChild(W),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let N of L)try{N()}catch(E){}},update:(N,E)=>{S.s=N,S.__i=E,S.__v.set((QJ)=>QJ+1)}}})(A)}}),B=(u)=>zJ({anchor:u,test:()=>{if(Q.activeSessions().length===0)return 0;return-1},branches:[()=>{let p=document.createDocumentFragment(),g=[],A=document.createElement("div");A.setAttribute("class","cb-session-row");let S=document.createElement("span");A.appendChild(S);let K=document.createElement("span");K.setAttribute("class","meta");let L=document.createTextNode("No active sessions");K.appendChild(L),A.appendChild(K);let W=document.createElement("span");return A.appendChild(W),p.appendChild(A),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let Y of g)try{Y()}catch(G){}}}}]}),q=(u)=>zJ({anchor:u,test:()=>{if(Q.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let p=document.createDocumentFragment(),g=[],A=document.createElement("div");A.setAttribute("class","cb-session-popover"),A.setAttribute("role","listbox"),A.setAttribute("aria-label","Sessions");let S=document.createComment("for");A.appendChild(S),g.push(D(S));let K=document.createComment("if");A.appendChild(K),g.push(B(K));let L=document.createElement("div");L.setAttribute("class","cb-session-sep"),L.setAttribute("role","separator"),A.appendChild(L);let W=document.createElement("div");W.setAttribute("class","cb-session-action"),g.push(e(W,"click",function(){let b=Q.startNewSession;return typeof b==="function"?b.bind(void 0):()=>{}}(),{modifiers:[]}));let Y=document.createElement("sui-icon");Y.setAttribute("name","plus"),Y.setAttribute("size","sm"),Y.setAttribute("aria-hidden","true"),W.appendChild(Y);let G=document.createElement("span"),H=document.createTextNode("Start new session");G.appendChild(H),W.appendChild(G),A.appendChild(W);let F=document.createElement("div");F.setAttribute("class","cb-session-action"),g.push(e(F,"click",function(){let b=Q.viewChats;return typeof b==="function"?b.bind(void 0):()=>{}}(),{modifiers:[]}));let M=document.createElement("sui-icon");M.setAttribute("name","columns"),M.setAttribute("size","sm"),M.setAttribute("aria-hidden","true"),F.appendChild(M);let T=document.createElement("span"),w=document.createTextNode("View all chats");return T.appendChild(w),F.appendChild(T),A.appendChild(F),p.appendChild(A),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let b of g)try{b()}catch(R){}}}}]}),P=(u)=>MJ({anchor:u,items:()=>Q.bar.thread(),key:(p,g)=>{let A=Object.create(Q);return A.msg=p,A.__i=g,((S)=>p.id)(A)},render:(p,g)=>{let A=Object.create(Q);return A.msg=p,A.__i=g,A.__v=k(0),((S)=>{let K=document.createDocumentFragment(),L=[],W=document.createElement("div");W.setAttribute("class","cb-chat-turn"),L.push(_(W,"data-sender",()=>(S.__v(),S.msg.sender)));let Y=document.createElement("div");Y.setAttribute("class","cb-chat-bubble");let G=document.createTextNode("");return Y.appendChild(G),L.push(h(G,()=>(S.__v(),S.msg.text))),W.appendChild(Y),K.appendChild(W),{nodes:Array.from(K.childNodes),cleanup:()=>{for(let H of L)try{H()}catch(F){}},update:(H,F)=>{S.msg=H,S.__i=F,S.__v.set((M)=>M+1)}}})(A)}}),x=(u)=>zJ({anchor:u,test:()=>{if(Q.bar.thread().length===0)return 0;return 1},branches:[()=>{let p=document.createDocumentFragment(),g=[],A=document.createElement("div");A.setAttribute("class","cb-chat-empty");let S=document.createTextNode("Type below and press Enter. The agent's replies land here.");return A.appendChild(S),p.appendChild(A),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let K of g)try{K()}catch(L){}}}},()=>{let p=document.createDocumentFragment(),g=[],A=document.createComment("for");return p.appendChild(A),g.push(P(A)),{nodes:Array.from(p.childNodes),cleanup:()=>{for(let S of g)try{S()}catch(K){}}}}]}),f=X.shadowRoot??X;NJ(f);let I=document.createElement("div");I.setAttribute("class","cb-chat-header");let v=document.createElement("span");v.setAttribute("class","cb-dot"),z.push(l(()=>{let u=Q.headerDotBg();if(u==null)v.style.removeProperty("background");else v.style.setProperty("background",String(u))})),v.setAttribute("aria-hidden","true"),I.appendChild(v);let JJ=document.createElement("div");JJ.setAttribute("style","display:grid;line-height:1.25");let c=document.createElement("span");c.setAttribute("class","title");let a=document.createTextNode("");c.appendChild(a),z.push(h(a,()=>Q.bar.session()?.agent_id??"")),JJ.appendChild(c);let ZJ=document.createElement("span");ZJ.setAttribute("class","sub");let i=document.createTextNode("");ZJ.appendChild(i),z.push(h(i,()=>Q.bar.session()?.provider??"—"));let C=document.createTextNode(" · ");ZJ.appendChild(C);let $=document.createTextNode("");ZJ.appendChild($),z.push(h($,()=>Q.bar.session()?.workspace_id??"")),JJ.appendChild(ZJ),I.appendChild(JJ);let n=document.createElement("span");n.setAttribute("style","flex:1"),I.appendChild(n);let s=document.createElement("div");s.setAttribute("class","cb-chip-wrap");let t=document.createElement("span");t.setAttribute("class","cb-session-chip"),t.setAttribute("role","button"),t.setAttribute("aria-haspopup","listbox"),z.push(e(t,"click",function(){let u=Q.toggleSessionPicker;return typeof u==="function"?u.bind(void 0):()=>{}}(),{modifiers:[]}));let O=document.createTextNode("Switch");t.appendChild(O),s.appendChild(t);let m=document.createComment("if");s.appendChild(m),z.push(q(m)),I.appendChild(s);let y=document.createElement("kbd");y.setAttribute("class","cb-esc"),y.setAttribute("aria-label","Escape to close");let VJ=document.createTextNode("esc");y.appendChild(VJ),I.appendChild(y),f.appendChild(I);let YJ=document.createElement("div");YJ.setAttribute("class","cb-chat-messages");let HJ=document.createComment("if");YJ.appendChild(HJ),z.push(x(HJ)),f.appendChild(YJ);let PJ=document.createElement("div");PJ.setAttribute("class","cb-chat-composer");let GJ=document.createElement("textarea");GJ.setAttribute("class","cb-chat-input"),GJ.setAttribute("rows","2"),GJ.setAttribute("placeholder","Message the agent…"),z.push(qJ(GJ,Q.bar.value)),z.push(e(GJ,"keydown",function(){let u=Q.onComposerKeydown;return typeof u==="function"?u.bind(void 0):()=>{}}(),{modifiers:[]})),PJ.appendChild(GJ);let LJ=document.createElement("button");LJ.setAttribute("class","cb-chat-send"),LJ.setAttribute("type","button"),z.push(_(LJ,"disabled",()=>!Q.bar.value().trim())),z.push(e(LJ,"click",function(){let u=Q.sendMessage;return typeof u==="function"?u.bind(void 0):()=>{}}(),{modifiers:[]}));let UJ=document.createTextNode("Send");if(LJ.appendChild(UJ),PJ.appendChild(LJ),f.appendChild(PJ),j!=null){let u=Object.getOwnPropertyDescriptor(j,"__shipSetLifecycle")?.value;if(u){let p=new Proxy(Q,{get(g,A){return g[A]},set(g,A,S){let K=g[A];if(K!=null&&typeof K.set==="function"){if(typeof S==="function")K.set(()=>S);else K.set(S);return!0}return!1}});if(u.mount){let g=u.mount(p,X);if(typeof g==="function")z.push(g)}if(u.unmount)z.push(()=>u.unmount(p))}}return Q}});var lJ=OJ({scope:(J,Z)=>{let X=wJ(Z,"sui-command-bar"),z=XJ(()=>{return{command:"/",find:">",mention:"@",create:"+",ai:"ai"}[X.mode()]??">"}),Q=XJ(()=>{return{command:"Run a Ship action…",find:"Jump to a view…",mention:"Reference an agent, view, file…",create:"Create a view, skill, agent…",ai:"Ask Ship or type / for actions"}[X.mode()]??"Search…"}),j=XJ(()=>X.session()?.agent_id??"No session"),D=XJ(()=>{let O=X.session();if(O?.status==="thinking")return"var(--warning,#d97706)";if(O)return"var(--success,#16a34a)";return"var(--muted-foreground)"}),B=XJ(()=>X.allSessions().filter((O)=>O.status!=="ended"&&O.agent_id));function q(O){if(O==="view")return"app-window";if(O==="agent")return"bot";return"message-square-plus"}let P=XJ(()=>{let O=X.value().replace(/^>+\s*/,"").toLowerCase().trim();return gJ.filter((m)=>!O||m.label.toLowerCase().includes(O)||m.keywords.toLowerCase().includes(O)||m.path.toLowerCase().includes(O)).map((m,y)=>({...m,idx:y}))}),x=XJ(()=>{let O=X.value().replace(/^\/+\s*/,"").toLowerCase().trim();return uJ.filter((m)=>!O||m.key.includes(O)||m.label.toLowerCase().includes(O)||m.hint.toLowerCase().includes(O)).map((m,y)=>({...m,idx:y}))}),f=XJ(()=>{let O=X.value().replace(/^@+/,"").toLowerCase().trim();return[...X.views().map((y)=>({kind:"view",id:y,label:y})),...X.agents().map((y)=>({kind:"agent",id:y.id,label:y.id})),...X.allSessions().filter((y)=>y.status!=="ended"&&y.agent_id).map((y)=>({kind:"session",id:y.id,label:`${y.agent_id} (session)`}))].filter((y)=>!O||y.id.toLowerCase().includes(O)).slice(0,40).map((y,VJ)=>({...y,idx:VJ,icon:q(y.kind)}))}),I=XJ(()=>mJ),v=XJ(()=>mJ.find((O)=>O.id===X.createSurface())),JJ=XJ(()=>U0(X.createName()));function c(O){fetch(`/api/v1/sessions/${O}`,{method:"DELETE",credentials:"same-origin"}).catch(()=>{}).then(()=>{if(X.session()?.id===O)X.session.set(null);t()}),X.sessionPickerOpen.set(!1)}function a(O){let m=X.allSessions().find((y)=>y.id===O);if(m)X.session.set(m),X.thread.set([]);X.sessionPickerOpen.set(!1)}function ZJ(){X.sessionPickerOpen.set(!1),X.mode.set("ai"),X.session.set(null)}function i(){X.sessionPickerOpen.set(!1),FJ("/chats"),X.open.set(!1)}let C=Z;function $(O){FJ(O),X.value.set(""),X.open.set(!1),C?.dispatchEvent(new CustomEvent("sui-command:select",{detail:{mode:"find",item:{path:O}},bubbles:!0,composed:!0}))}function n(O){switch(X.value.set(""),O){case"new":X.mode.set("create");break;case"chats":FJ("/chats"),X.open.set(!1);break;case"settings":FJ("/settings"),X.open.set(!1);break;case"end":if(X.session())c(X.session().id);X.open.set(!1);break;case"close":X.open.set(!1);break}}function s(O){let m=O.kind==="view"?`@${O.id}`:`@${O.kind}:${O.id}`,y=X.value();X.value.set(y.replace(/@[^@\s]*$/,`${m} `)),X.mode.set(kJ(X.value(),X.mode()))}async function t(){try{let O=await fetch("/api/v1/pty-sessions",{credentials:"same-origin"});if(O.ok){let m=await O.json();X.allSessions.set(m.sessions);let y=m.sessions.find((VJ)=>VJ.status!=="ended"&&VJ.agent_id);if(y&&y.id!==X.session()?.id)X.session.set(y),X.thread.set([]);else if(!y)X.session.set(null)}}catch{}}return{bar:X,modeGlyph:z,searchPlaceholder:Q,chipLabel:j,chipDotBg:D,activeSessions:B,findItems:P,cmdItems:x,mentionItems:f,createSurfaces:I,currentSurfaceDef:v,createNameValid:JJ,endSession:c,selectSession:a,startNewSession:ZJ,viewChats:i,activateNavItem:$,activateCmd:n,activateMention:s}},on:{onInput:(J,Z)=>{let X=Z.target.value;J.bar.value.set(X),J.bar.mode.set(kJ(X,J.bar.mode())),J.bar.selectedIdx.set(0)},toggleSessionPicker:(J)=>{J.bar.sessionPickerOpen.set(!J.bar.sessionPickerOpen())},selectSurface:(J,Z)=>{J.bar.createSurface.set(Z);let X=mJ.find((z)=>z.id===Z);if(X?.shapes[0])J.bar.createShape.set(X.shapes[0])},spawnSession:async(J)=>{let Z=J.bar.spawnAgent(),X=J.bar.spawnProvider();if(!Z||!X)return;J.bar.spawning.set(!0),J.bar.spawnError.set(null);try{let z=await fetch("/api/v1/sessions",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({agent_id:Z,provider:X})});if(!z.ok){let Q=await z.json().catch(()=>({}));J.bar.spawnError.set(Q.error??`HTTP ${z.status}`)}else J.bar.mode.set("ai")}catch(z){J.bar.spawnError.set(z instanceof Error?z.message:"Unknown error")}finally{J.bar.spawning.set(!1)}},submitCreate:async(J)=>{if(!J.createNameValid()||J.bar.createSubmitting())return;J.bar.createSubmitting.set(!0),J.bar.createError.set(null);try{let Z=await fetch("/api/v1/scaffold",{method:"POST",credentials:"same-origin",headers:{"Content-Type":"application/json"},body:JSON.stringify({surface:J.bar.createSurface(),name:J.bar.createName(),shape:J.bar.createShape(),purpose:J.bar.createPurpose()})});if(!Z.ok){let X=await Z.json().catch(()=>({}));J.bar.createError.set(X.error??`HTTP ${Z.status}`)}else J.bar.createName.set(""),J.bar.createPurpose.set(""),J.bar.open.set(!1)}catch(Z){J.bar.createError.set(Z instanceof Error?Z.message:"Unknown error")}finally{J.bar.createSubmitting.set(!1)}}}});var lQ=KJ("sui-command-palette",{shadow:!0,styles:`
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
`,setup(J){let{props:Z,host:X}=J;if(!J.cleanups)J.cleanups=[];let z=J.cleanups,Q=Object.create(null),j=null;if(lJ&&typeof lJ.setup==="function"){if(J.scope=Q,j=lJ.setup(J),j&&typeof j==="object")Object.assign(Q,j)}let D=(K)=>MJ({anchor:K,items:()=>Q.activeSessions(),key:(L,W)=>{let Y=Object.create(Q);return Y.s=L,Y.__i=W,((G)=>L.id)(Y)},render:(L,W)=>{let Y=Object.create(Q);return Y.s=L,Y.__i=W,Y.__v=k(0),((G)=>{let H=document.createDocumentFragment(),F=[],M=document.createElement("div");M.setAttribute("class","cb-session-row"),F.push(CJ(M,"current",()=>(G.__v(),G.bar.session()?.id===G.s.id))),M.setAttribute("role","option"),F.push(e(M,"click",function(){let d=()=>G.selectSession(G.s.id);return typeof d==="function"?d.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("span");T.setAttribute("class","cb-dot"),F.push(l(()=>{G.__v();let d=G.s.status==="thinking"?"var(--warning,#d97706)":"var(--success,#16a34a)";if(d==null)T.style.removeProperty("background");else T.style.setProperty("background",String(d))})),M.appendChild(T);let w=document.createElement("span"),b=document.createElement("strong"),R=document.createTextNode("");b.appendChild(R),F.push(h(R,()=>(G.__v(),G.s.agent_id??""))),w.appendChild(b);let V=document.createElement("span");V.setAttribute("class","meta");let U=document.createTextNode(" · ");V.appendChild(U);let N=document.createTextNode("");V.appendChild(N),F.push(h(N,()=>(G.__v(),G.s.provider??"")));let E=document.createTextNode(" · ");V.appendChild(E);let QJ=document.createTextNode("");V.appendChild(QJ),F.push(h(QJ,()=>(G.__v(),G.s.workspace_id))),w.appendChild(V),M.appendChild(w);let r=document.createElement("button");r.setAttribute("class","cb-session-end"),r.setAttribute("type","button"),r.setAttribute("aria-label","End session"),F.push(e(r,"click",function(){let d=(jJ)=>{jJ.stopPropagation(),G.endSession(G.s.id)};return typeof d==="function"?d.bind(void 0):()=>{}}(),{modifiers:[]}));let WJ=document.createTextNode("×");return r.appendChild(WJ),M.appendChild(r),H.appendChild(M),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let d of F)try{d()}catch(jJ){}},update:(d,jJ)=>{G.s=d,G.__i=jJ,G.__v.set((SJ)=>SJ+1)}}})(Y)}}),B=(K)=>zJ({anchor:K,test:()=>{if(Q.activeSessions().length===0)return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-session-row");let G=document.createElement("span");Y.appendChild(G);let H=document.createElement("span");H.setAttribute("class","meta");let F=document.createTextNode("No active sessions");H.appendChild(F),Y.appendChild(H);let M=document.createElement("span");return Y.appendChild(M),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let T of W)try{T()}catch(w){}}}}]}),q=(K)=>zJ({anchor:K,test:()=>{if(Q.bar.sessionPickerOpen())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-session-popover"),Y.setAttribute("role","listbox"),Y.setAttribute("aria-label","Sessions");let G=document.createComment("for");Y.appendChild(G),W.push(D(G));let H=document.createComment("if");Y.appendChild(H),W.push(B(H));let F=document.createElement("div");F.setAttribute("class","cb-session-sep"),F.setAttribute("role","separator"),Y.appendChild(F);let M=document.createElement("div");M.setAttribute("class","cb-session-action"),W.push(e(M,"click",function(){let E=Q.startNewSession;return typeof E==="function"?E.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");T.setAttribute("name","plus"),T.setAttribute("size","sm"),T.setAttribute("aria-hidden","true"),M.appendChild(T);let w=document.createElement("span"),b=document.createTextNode("Start new session");w.appendChild(b),M.appendChild(w),Y.appendChild(M);let R=document.createElement("div");R.setAttribute("class","cb-session-action"),W.push(e(R,"click",function(){let E=Q.viewChats;return typeof E==="function"?E.bind(void 0):()=>{}}(),{modifiers:[]}));let V=document.createElement("sui-icon");V.setAttribute("name","columns"),V.setAttribute("size","sm"),V.setAttribute("aria-hidden","true"),R.appendChild(V);let U=document.createElement("span"),N=document.createTextNode("View all chats");return U.appendChild(N),R.appendChild(U),Y.appendChild(R),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let E of W)try{E()}catch(QJ){}}}}]}),P=(K)=>MJ({anchor:K,items:()=>Q.findItems(),key:(L,W)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=W,((G)=>L.path)(Y)},render:(L,W)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=W,Y.__v=k(0),((G)=>{let H=document.createDocumentFragment(),F=[],M=document.createElement("div");M.setAttribute("class","cb-item"),M.setAttribute("role","option"),F.push(_(M,"aria-selected",()=>(G.__v(),G.bar.selectedIdx()===G.item.idx?"true":"false"))),F.push(_(M,"data-path",()=>(G.__v(),G.item.path))),F.push(e(M,"click",function(){let U=()=>G.activateNavItem(G.item.path);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");F.push(_(T,"name",()=>(G.__v(),G.item.icon))),T.setAttribute("size","sm"),T.setAttribute("aria-hidden","true"),M.appendChild(T);let w=document.createElement("span"),b=document.createTextNode("");w.appendChild(b),F.push(h(b,()=>(G.__v(),G.item.label))),M.appendChild(w);let R=document.createElement("span");R.setAttribute("class","hint");let V=document.createTextNode("");return R.appendChild(V),F.push(h(V,()=>(G.__v(),G.item.path))),M.appendChild(R),H.appendChild(M),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let U of F)try{U()}catch(N){}},update:(U,N)=>{G.item=U,G.__i=N,G.__v.set((E)=>E+1)}}})(Y)}}),x=(K)=>zJ({anchor:K,test:()=>{if(Q.findItems().length>0)return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-mode-hint");let G=document.createTextNode("Jump to");Y.appendChild(G),L.appendChild(Y);let H=document.createElement("div");H.setAttribute("class","cb-list"),H.setAttribute("role","listbox");let F=document.createComment("for");return H.appendChild(F),W.push(P(F)),L.appendChild(H),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let M of W)try{M()}catch(T){}}}},()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-empty");let G=document.createTextNode("No matches.");return Y.appendChild(G),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of W)try{H()}catch(F){}}}}]}),f=(K)=>MJ({anchor:K,items:()=>Q.cmdItems(),key:(L,W)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=W,((G)=>L.key)(Y)},render:(L,W)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=W,Y.__v=k(0),((G)=>{let H=document.createDocumentFragment(),F=[],M=document.createElement("div");M.setAttribute("class","cb-item"),M.setAttribute("role","option"),F.push(_(M,"aria-selected",()=>(G.__v(),G.bar.selectedIdx()===G.item.idx?"true":"false"))),F.push(_(M,"data-cmd",()=>(G.__v(),G.item.key))),F.push(e(M,"click",function(){let U=()=>G.activateCmd(G.item.key);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("span");T.setAttribute("style","font-family:var(--font-mono)");let w=document.createTextNode("");T.appendChild(w),F.push(h(w,()=>(G.__v(),G.item.label))),M.appendChild(T);let b=document.createElement("span");M.appendChild(b);let R=document.createElement("span");R.setAttribute("class","hint");let V=document.createTextNode("");return R.appendChild(V),F.push(h(V,()=>(G.__v(),G.item.hint))),M.appendChild(R),H.appendChild(M),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let U of F)try{U()}catch(N){}},update:(U,N)=>{G.item=U,G.__i=N,G.__v.set((E)=>E+1)}}})(Y)}}),I=(K)=>zJ({anchor:K,test:()=>{if(Q.cmdItems().length>0)return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-mode-hint");let G=document.createTextNode("Actions");Y.appendChild(G),L.appendChild(Y);let H=document.createElement("div");H.setAttribute("class","cb-list"),H.setAttribute("role","listbox");let F=document.createComment("for");return H.appendChild(F),W.push(f(F)),L.appendChild(H),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let M of W)try{M()}catch(T){}}}},()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-empty");let G=document.createTextNode("No commands match.");return Y.appendChild(G),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of W)try{H()}catch(F){}}}}]}),v=(K)=>MJ({anchor:K,items:()=>Q.mentionItems(),key:(L,W)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=W,((G)=>L.id+L.kind)(Y)},render:(L,W)=>{let Y=Object.create(Q);return Y.item=L,Y.__i=W,Y.__v=k(0),((G)=>{let H=document.createDocumentFragment(),F=[],M=document.createElement("div");M.setAttribute("class","cb-item"),M.setAttribute("role","option"),F.push(_(M,"aria-selected",()=>(G.__v(),G.bar.selectedIdx()===G.item.idx?"true":"false"))),F.push(_(M,"data-mention-kind",()=>(G.__v(),G.item.kind))),F.push(_(M,"data-mention-id",()=>(G.__v(),G.item.id))),F.push(e(M,"click",function(){let U=()=>G.activateMention(G.item);return typeof U==="function"?U.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");F.push(_(T,"name",()=>(G.__v(),G.item.icon))),T.setAttribute("size","sm"),T.setAttribute("aria-hidden","true"),M.appendChild(T);let w=document.createElement("span"),b=document.createTextNode("");w.appendChild(b),F.push(h(b,()=>(G.__v(),G.item.label))),M.appendChild(w);let R=document.createElement("span");R.setAttribute("class","hint");let V=document.createTextNode("");return R.appendChild(V),F.push(h(V,()=>(G.__v(),G.item.kind))),M.appendChild(R),H.appendChild(M),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let U of F)try{U()}catch(N){}},update:(U,N)=>{G.item=U,G.__i=N,G.__v.set((E)=>E+1)}}})(Y)}}),JJ=(K)=>zJ({anchor:K,test:()=>{if(Q.mentionItems().length>0)return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-mode-hint");let G=document.createTextNode("Reference");Y.appendChild(G),L.appendChild(Y);let H=document.createElement("div");H.setAttribute("class","cb-list"),H.setAttribute("role","listbox");let F=document.createComment("for");return H.appendChild(F),W.push(v(F)),L.appendChild(H),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let M of W)try{M()}catch(T){}}}},()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-empty");let G=document.createTextNode("No matching views or agents.");return Y.appendChild(G),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of W)try{H()}catch(F){}}}}]}),c=(K)=>MJ({anchor:K,items:()=>Q.createSurfaces(),key:(L,W)=>{let Y=Object.create(Q);return Y.surf=L,Y.__i=W,((G)=>L.id)(Y)},render:(L,W)=>{let Y=Object.create(Q);return Y.surf=L,Y.__i=W,Y.__v=k(0),((G)=>{let H=document.createDocumentFragment(),F=[],M=document.createElement("button");M.setAttribute("class","cb-surface-btn"),M.setAttribute("type","button"),F.push(_(M,"aria-pressed",()=>(G.__v(),G.bar.createSurface()===G.surf.id?"true":"false"))),F.push(e(M,"click",function(){let R=()=>G.selectSurface(G.surf.id);return typeof R==="function"?R.bind(void 0):()=>{}}(),{modifiers:[]}));let T=document.createElement("sui-icon");F.push(_(T,"name",()=>(G.__v(),G.surf.icon))),T.setAttribute("aria-hidden","true"),M.appendChild(T);let w=document.createElement("span"),b=document.createTextNode("");return w.appendChild(b),F.push(h(b,()=>(G.__v(),G.surf.label))),M.appendChild(w),H.appendChild(M),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let R of F)try{R()}catch(V){}},update:(R,V)=>{G.surf=R,G.__i=V,G.__v.set((U)=>U+1)}}})(Y)}}),a=(K)=>MJ({anchor:K,items:()=>Q.currentSurfaceDef().shapes,key:(L,W)=>{let Y=Object.create(Q);return Y.sh=L,Y.__i=W,((G)=>L)(Y)},render:(L,W)=>{let Y=Object.create(Q);return Y.sh=L,Y.__i=W,Y.__v=k(0),((G)=>{let H=document.createDocumentFragment(),F=[],M=document.createElement("option");F.push(_(M,"value",()=>(G.__v(),G.sh)));let T=document.createTextNode("");return M.appendChild(T),F.push(h(T,()=>(G.__v(),G.sh))),H.appendChild(M),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let w of F)try{w()}catch(b){}},update:(w,b)=>{G.sh=w,G.__i=b,G.__v.set((R)=>R+1)}}})(Y)}}),ZJ=(K)=>zJ({anchor:K,test:()=>{if(Q.bar.createError())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-create-error");let G=document.createTextNode("");return Y.appendChild(G),W.push(h(G,()=>Q.bar.createError())),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of W)try{H()}catch(F){}}}}]}),i=(K)=>MJ({anchor:K,items:()=>Q.bar.agents(),key:(L,W)=>{let Y=Object.create(Q);return Y.a=L,Y.__i=W,((G)=>L.id)(Y)},render:(L,W)=>{let Y=Object.create(Q);return Y.a=L,Y.__i=W,Y.__v=k(0),((G)=>{let H=document.createDocumentFragment(),F=[],M=document.createElement("option");F.push(_(M,"value",()=>(G.__v(),G.a.id)));let T=document.createTextNode("");return M.appendChild(T),F.push(h(T,()=>(G.__v(),G.a.name??G.a.label??G.a.id))),H.appendChild(M),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let w of F)try{w()}catch(b){}},update:(w,b)=>{G.a=w,G.__i=b,G.__v.set((R)=>R+1)}}})(Y)}}),C=(K)=>zJ({anchor:K,test:()=>{if(Q.bar.agents().length===0)return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("option");Y.setAttribute("value","");let G=document.createTextNode("No agents found");return Y.appendChild(G),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of W)try{H()}catch(F){}}}},()=>{let L=document.createDocumentFragment(),W=[],Y=document.createComment("for");return L.appendChild(Y),W.push(i(Y)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let G of W)try{G()}catch(H){}}}}]}),$=(K)=>MJ({anchor:K,items:()=>Q.bar.providers(),key:(L,W)=>{let Y=Object.create(Q);return Y.p=L,Y.__i=W,((G)=>L)(Y)},render:(L,W)=>{let Y=Object.create(Q);return Y.p=L,Y.__i=W,Y.__v=k(0),((G)=>{let H=document.createDocumentFragment(),F=[],M=document.createElement("option");F.push(_(M,"value",()=>(G.__v(),G.p)));let T=document.createTextNode("");return M.appendChild(T),F.push(h(T,()=>(G.__v(),G.p))),H.appendChild(M),{nodes:Array.from(H.childNodes),cleanup:()=>{for(let w of F)try{w()}catch(b){}},update:(w,b)=>{G.p=w,G.__i=b,G.__v.set((R)=>R+1)}}})(Y)}}),n=(K)=>zJ({anchor:K,test:()=>{if(Q.bar.providers().length===0)return 0;return 1},branches:[()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("option");Y.setAttribute("value","");let G=document.createTextNode("No providers installed");return Y.appendChild(G),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of W)try{H()}catch(F){}}}},()=>{let L=document.createDocumentFragment(),W=[],Y=document.createComment("for");return L.appendChild(Y),W.push($(Y)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let G of W)try{G()}catch(H){}}}}]}),s=(K)=>zJ({anchor:K,test:()=>{if(Q.bar.spawnError())return 0;return-1},branches:[()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-spawn-error");let G=document.createTextNode("");return Y.appendChild(G),W.push(h(G,()=>Q.bar.spawnError())),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let H of W)try{H()}catch(F){}}}}]}),t=(K)=>zJ({anchor:K,test:()=>{let L=Q.bar.mode();if(L==="find")return 0;if(L==="command")return 1;if(L==="mention")return 2;if(L==="create")return 3;return 4},branches:[()=>{let L=document.createDocumentFragment(),W=[],Y=document.createComment("if");return L.appendChild(Y),W.push(x(Y)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let G of W)try{G()}catch(H){}}}},()=>{let L=document.createDocumentFragment(),W=[],Y=document.createComment("if");return L.appendChild(Y),W.push(I(Y)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let G of W)try{G()}catch(H){}}}},()=>{let L=document.createDocumentFragment(),W=[],Y=document.createComment("if");return L.appendChild(Y),W.push(JJ(Y)),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let G of W)try{G()}catch(H){}}}},()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-mode-hint");let G=document.createTextNode("Create");Y.appendChild(G),L.appendChild(Y);let H=document.createElement("div");H.setAttribute("class","cb-create-surfaces"),H.setAttribute("role","group"),H.setAttribute("aria-label","Surface type");let F=document.createComment("for");H.appendChild(F),W.push(c(F)),L.appendChild(H);let M=document.createElement("div");M.setAttribute("class","cb-create-grid");let T=document.createElement("label"),w=document.createElement("span"),b=document.createTextNode("Name (kebab-case)");w.appendChild(b),T.appendChild(w);let R=document.createElement("input");R.setAttribute("type","text"),R.setAttribute("class","cb-create-name"),W.push(_(R,"placeholder",()=>"my-"+Q.bar.createSurface())),W.push(qJ(R,Q.bar.createName)),R.setAttribute("autocomplete","off"),T.appendChild(R),M.appendChild(T);let V=document.createElement("label"),U=document.createElement("span"),N=document.createTextNode("");U.appendChild(N),W.push(h(N,()=>Q.currentSurfaceDef().shapeLabel)),V.appendChild(U);let E=document.createElement("select");E.setAttribute("class","cb-create-shape"),W.push(qJ(E,Q.bar.createShape));let QJ=document.createComment("for");E.appendChild(QJ),W.push(a(QJ)),V.appendChild(E),M.appendChild(V);let r=document.createElement("label");r.setAttribute("class","span-2");let WJ=document.createElement("span"),d=document.createTextNode("Purpose (optional)");WJ.appendChild(d),r.appendChild(WJ);let jJ=document.createElement("input");jJ.setAttribute("type","text"),jJ.setAttribute("class","cb-create-purpose"),jJ.setAttribute("placeholder","One-line description"),W.push(qJ(jJ,Q.bar.createPurpose)),jJ.setAttribute("autocomplete","off"),r.appendChild(jJ),M.appendChild(r),L.appendChild(M);let SJ=document.createComment("if");L.appendChild(SJ),W.push(ZJ(SJ));let pJ=document.createElement("div");pJ.setAttribute("class","cb-create-actions");let IJ=document.createElement("button");IJ.setAttribute("class","cb-create-submit"),IJ.setAttribute("type","button"),W.push(_(IJ,"disabled",()=>Q.bar.createSubmitting()||!Q.createNameValid())),W.push(e(IJ,"click",function(){let yJ=Q.submitCreate;return typeof yJ==="function"?yJ.bind(void 0):()=>{}}(),{modifiers:[]}));let X0=document.createTextNode("");return IJ.appendChild(X0),W.push(h(X0,()=>Q.bar.createSubmitting()?"Creating…":"Create "+Q.currentSurfaceDef().label)),pJ.appendChild(IJ),L.appendChild(pJ),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let yJ of W)try{yJ()}catch(i0){}}}},()=>{let L=document.createDocumentFragment(),W=[],Y=document.createElement("div");Y.setAttribute("class","cb-spawn-card");let G=document.createElement("div");G.setAttribute("class","cb-spawn-title");let H=document.createTextNode("Start a session");G.appendChild(H),Y.appendChild(G);let F=document.createElement("div");F.setAttribute("class","cb-spawn-row");let M=document.createElement("label"),T=document.createTextNode("Agent");M.appendChild(T),F.appendChild(M);let w=document.createElement("select");w.setAttribute("class","cb-spawn-agent"),W.push(qJ(w,Q.bar.spawnAgent));let b=document.createComment("if");w.appendChild(b),W.push(C(b)),F.appendChild(w),Y.appendChild(F);let R=document.createElement("div");R.setAttribute("class","cb-spawn-row");let V=document.createElement("label"),U=document.createTextNode("Provider");V.appendChild(U),R.appendChild(V);let N=document.createElement("select");N.setAttribute("class","cb-spawn-provider"),W.push(qJ(N,Q.bar.spawnProvider));let E=document.createComment("if");N.appendChild(E),W.push(n(E)),R.appendChild(N),Y.appendChild(R);let QJ=document.createComment("if");Y.appendChild(QJ),W.push(s(QJ));let r=document.createElement("button");r.setAttribute("class","cb-spawn-start"),r.setAttribute("type","button"),W.push(_(r,"disabled",()=>Q.bar.spawning()||!Q.bar.spawnAgent()||!Q.bar.spawnProvider())),W.push(e(r,"click",function(){let d=Q.spawnSession;return typeof d==="function"?d.bind(void 0):()=>{}}(),{modifiers:[]}));let WJ=document.createTextNode("");return r.appendChild(WJ),W.push(h(WJ,()=>Q.bar.spawning()?"Starting…":"Start session")),Y.appendChild(r),L.appendChild(Y),{nodes:Array.from(L.childNodes),cleanup:()=>{for(let d of W)try{d()}catch(jJ){}}}}]}),O=X.shadowRoot??X;NJ(O);let m=document.createElement("div");m.setAttribute("class","cb-input-row");let y=document.createElement("span");y.setAttribute("class","cb-mode-glyph"),z.push(CJ(y,"ai",()=>Q.bar.mode()==="ai"));let VJ=document.createTextNode("");y.appendChild(VJ),z.push(h(VJ,()=>Q.modeGlyph())),m.appendChild(y);let YJ=document.createElement("input");YJ.setAttribute("class","cb-search"),YJ.setAttribute("type","text"),YJ.setAttribute("autocomplete","off"),YJ.setAttribute("spellcheck","false"),z.push(_(YJ,"placeholder",()=>Q.searchPlaceholder())),z.push(qJ(YJ,Q.bar.value)),YJ.setAttribute("aria-label","Command bar search"),YJ.setAttribute("aria-autocomplete","list"),z.push(e(YJ,"input",function(){let K=Q.onInput;return typeof K==="function"?K.bind(void 0):()=>{}}(),{modifiers:[]})),m.appendChild(YJ);let HJ=document.createElement("kbd");HJ.setAttribute("class","cb-esc"),HJ.setAttribute("aria-label","Escape to close");let PJ=document.createTextNode("esc");HJ.appendChild(PJ),m.appendChild(HJ);let GJ=document.createElement("div");GJ.setAttribute("class","cb-chip-wrap");let LJ=document.createElement("span");LJ.setAttribute("class","cb-session-chip"),LJ.setAttribute("role","button"),LJ.setAttribute("aria-haspopup","listbox"),z.push(e(LJ,"click",function(){let K=Q.toggleSessionPicker;return typeof K==="function"?K.bind(void 0):()=>{}}(),{modifiers:[]}));let UJ=document.createElement("span");UJ.setAttribute("class","cb-dot"),z.push(l(()=>{let K=Q.chipDotBg();if(K==null)UJ.style.removeProperty("background");else UJ.style.setProperty("background",String(K))})),LJ.appendChild(UJ);let u=document.createElement("span"),p=document.createTextNode("");u.appendChild(p),z.push(h(p,()=>Q.chipLabel())),LJ.appendChild(u),GJ.appendChild(LJ);let g=document.createComment("if");GJ.appendChild(g),z.push(q(g)),m.appendChild(GJ),O.appendChild(m);let A=document.createElement("div");A.setAttribute("class","cb-body");let S=document.createComment("match");if(A.appendChild(S),z.push(t(S)),O.appendChild(A),j!=null){let K=Object.getOwnPropertyDescriptor(j,"__shipSetLifecycle")?.value;if(K){let L=new Proxy(Q,{get(W,Y){return W[Y]},set(W,Y,G){let H=W[Y];if(H!=null&&typeof H.set==="function"){if(typeof G==="function")H.set(()=>G);else H.set(G);return!0}return!1}});if(K.mount){let W=K.mount(L,X);if(typeof W==="function")z.push(W)}if(K.unmount)z.push(()=>K.unmount(L))}}return Q}});
