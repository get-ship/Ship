/*
 * grainient.ts — the reactbits "Grainient" background, ported to raw WebGL2
 * (no React, no ogl, no new dependency → bun.lock untouched). The fragment
 * shader + default uniforms are verbatim from the original; the only change
 * is the three colours come from the live Ship theme tokens, so it recolours
 * with the theme. Mount on a <canvas>; start()/stop() drive the rAF loop.
 */

const VERT = `#version 300 es
in vec2 position;
void main() { gl_Position = vec4(position, 0.0, 1.0); }`;

const FRAG = `#version 300 es
precision highp float;
uniform vec2 iResolution;
uniform float iTime;
uniform float uTimeSpeed;
uniform float uColorBalance;
uniform float uWarpStrength;
uniform float uWarpFrequency;
uniform float uWarpSpeed;
uniform float uWarpAmplitude;
uniform float uBlendAngle;
uniform float uBlendSoftness;
uniform float uRotationAmount;
uniform float uNoiseScale;
uniform float uGrainAmount;
uniform float uGrainScale;
uniform float uGrainAnimated;
uniform float uContrast;
uniform float uGamma;
uniform float uSaturation;
uniform vec2 uCenterOffset;
uniform float uZoom;
uniform vec3 uColor1;
uniform vec3 uColor2;
uniform vec3 uColor3;
out vec4 fragColor;
#define S(a,b,t) smoothstep(a,b,t)
mat2 Rot(float a){float s=sin(a),c=cos(a);return mat2(c,-s,s,c);}
vec2 hash(vec2 p){p=vec2(dot(p,vec2(2127.1,81.17)),dot(p,vec2(1269.5,283.37)));return fract(sin(p)*43758.5453);}
float noise(vec2 p){vec2 i=floor(p),f=fract(p),u=f*f*(3.0-2.0*f);float n=mix(mix(dot(-1.0+2.0*hash(i+vec2(0.0,0.0)),f-vec2(0.0,0.0)),dot(-1.0+2.0*hash(i+vec2(1.0,0.0)),f-vec2(1.0,0.0)),u.x),mix(dot(-1.0+2.0*hash(i+vec2(0.0,1.0)),f-vec2(0.0,1.0)),dot(-1.0+2.0*hash(i+vec2(1.0,1.0)),f-vec2(1.0,1.0)),u.x),u.y);return 0.5+0.5*n;}
void mainImage(out vec4 o, vec2 C){
  float t=iTime*uTimeSpeed;
  vec2 uv=C/iResolution.xy;
  float ratio=iResolution.x/iResolution.y;
  vec2 tuv=uv-0.5+uCenterOffset;
  tuv/=max(uZoom,0.001);
  float degree=noise(vec2(t*0.1,tuv.x*tuv.y)*uNoiseScale);
  tuv.y*=1.0/ratio;
  tuv*=Rot(radians((degree-0.5)*uRotationAmount+180.0));
  tuv.y*=ratio;
  float frequency=uWarpFrequency;
  float ws=max(uWarpStrength,0.001);
  float amplitude=uWarpAmplitude/ws;
  float warpTime=t*uWarpSpeed;
  tuv.x+=sin(tuv.y*frequency+warpTime)/amplitude;
  tuv.y+=sin(tuv.x*(frequency*1.5)+warpTime)/(amplitude*0.5);
  vec3 colLav=uColor1;
  vec3 colOrg=uColor2;
  vec3 colDark=uColor3;
  float b=uColorBalance;
  float s=max(uBlendSoftness,0.0);
  mat2 blendRot=Rot(radians(uBlendAngle));
  float blendX=(tuv*blendRot).x;
  float edge0=-0.3-b-s;
  float edge1=0.2-b+s;
  float v0=0.5-b+s;
  float v1=-0.3-b-s;
  vec3 layer1=mix(colDark,colOrg,S(edge0,edge1,blendX));
  vec3 layer2=mix(colOrg,colLav,S(edge0,edge1,blendX));
  vec3 col=mix(layer1,layer2,S(v0,v1,tuv.y));
  vec2 grainUv=uv*max(uGrainScale,0.001);
  if(uGrainAnimated>0.5){grainUv+=vec2(iTime*0.05);}
  float grain=fract(sin(dot(grainUv,vec2(12.9898,78.233)))*43758.5453);
  col+=(grain-0.5)*uGrainAmount;
  col=(col-0.5)*uContrast+0.5;
  float luma=dot(col,vec3(0.2126,0.7152,0.0722));
  col=mix(vec3(luma),col,uSaturation);
  col=pow(max(col,0.0),vec3(1.0/max(uGamma,0.001)));
  col=clamp(col,0.0,1.0);
  o=vec4(col,1.0);
}
void main(){ vec4 o=vec4(0.0); mainImage(o,gl_FragCoord.xy); fragColor=o; }`;

// Static uniform defaults — verbatim from the original component.
const DEFAULTS: Record<string, number> = {
  uTimeSpeed: 0.25, uColorBalance: 0.0, uWarpStrength: 1.0, uWarpFrequency: 5.0,
  uWarpSpeed: 2.0, uWarpAmplitude: 50.0, uBlendAngle: 0.0, uBlendSoftness: 0.05,
  uRotationAmount: 500.0, uNoiseScale: 2.0, uGrainAmount: 0.1, uGrainScale: 2.0,
  uGrainAnimated: 0.0, uContrast: 1.5, uGamma: 1.0, uSaturation: 1.0, uZoom: 0.9,
};

function compile(gl: WebGL2RenderingContext, type: number, src: string): WebGLShader | null {
  const sh = gl.createShader(type)!;
  gl.shaderSource(sh, src);
  gl.compileShader(sh);
  if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) { console.warn('[grainient]', gl.getShaderInfoLog(sh)); return null; }
  return sh;
}

function tokenRGB(probe: HTMLElement, scratch: CanvasRenderingContext2D, varName: string, fallback: [number, number, number]): [number, number, number] {
  try {
    probe.style.color = `var(${varName})`;
    const resolved = getComputedStyle(probe).color;
    scratch.clearRect(0, 0, 1, 1);
    scratch.fillStyle = '#000';
    scratch.fillStyle = resolved;
    scratch.fillRect(0, 0, 1, 1);
    const d = scratch.getImageData(0, 0, 1, 1).data;
    return [d[0] / 255, d[1] / 255, d[2] / 255];
  } catch { return fallback; }
}

export interface Grainient { start(): void; stop(): void; recolor(): void; destroy(): void; }

export function mountGrainient(canvas: HTMLCanvasElement): Grainient | null {
  const gl = canvas.getContext('webgl2', { antialias: false, alpha: true });
  if (!gl) return null;
  const vs = compile(gl, gl.VERTEX_SHADER, VERT);
  const fs = compile(gl, gl.FRAGMENT_SHADER, FRAG);
  if (!vs || !fs) return null;
  const prog = gl.createProgram()!;
  gl.attachShader(prog, vs);
  gl.attachShader(prog, fs);
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) { console.warn('[grainient]', gl.getProgramInfoLog(prog)); return null; }
  gl.useProgram(prog);

  // Fullscreen triangle (same as ogl's Triangle geometry).
  const buf = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, buf);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
  const pos = gl.getAttribLocation(prog, 'position');
  gl.enableVertexAttribArray(pos);
  gl.vertexAttribPointer(pos, 2, gl.FLOAT, false, 0, 0);

  const U = (n: string) => gl.getUniformLocation(prog, n);
  for (const [k, v] of Object.entries(DEFAULTS)) gl.uniform1f(U(k), v);
  gl.uniform2f(U('uCenterOffset'), 0, 0);
  const uTime = U('iTime'), uRes = U('iResolution');

  const probe = document.createElement('span');
  probe.style.cssText = 'position:absolute;width:0;height:0;visibility:hidden';
  canvas.after(probe);
  const scratch = document.createElement('canvas').getContext('2d', { willReadFrequently: true })!;

  function recolor() {
    gl!.useProgram(prog);
    gl!.uniform3fv(U('uColor1'), tokenRGB(probe, scratch, '--accent', [1.0, 0.62, 0.99]));
    gl!.uniform3fv(U('uColor2'), tokenRGB(probe, scratch, '--primary', [0.32, 0.15, 1.0]));
    gl!.uniform3fv(U('uColor3'), tokenRGB(probe, scratch, '--app-accent-agents', [0.71, 0.59, 0.81]));
  }

  const dpr = () => Math.min(window.devicePixelRatio || 1, 2);
  function resize() {
    const w = Math.max(1, Math.floor(canvas.clientWidth * dpr()));
    const h = Math.max(1, Math.floor(canvas.clientHeight * dpr()));
    if (canvas.width !== w || canvas.height !== h) { canvas.width = w; canvas.height = h; }
    gl!.viewport(0, 0, canvas.width, canvas.height);
    gl!.uniform2f(uRes, canvas.width, canvas.height);
  }

  // Low-frequency rAF: paint at ~6fps (one frame every 160ms). The
  // original ran at 30fps and read canvas.clientWidth (layout reflow)
  // every frame — that was the "not responding" cliff. At 6fps the
  // motion reads as a slow ambient drift, the resize() only runs once
  // per actual viewport change (cached), and the GL draw is the only
  // cost per tick. Compared to the static version we lose ~50ms of
  // main-thread time per second; small price for visible life.
  const FRAME_MS = 160;
  const t0 = performance.now();
  let raf = 0;
  let lastDraw = 0;
  let cachedW = 0, cachedH = 0;
  function ensureSize(): void {
    const w = Math.max(1, Math.floor(canvas.clientWidth * dpr()));
    const h = Math.max(1, Math.floor(canvas.clientHeight * dpr()));
    if (w !== cachedW || h !== cachedH) {
      canvas.width = w; canvas.height = h;
      gl!.viewport(0, 0, w, h);
      gl!.uniform2f(uRes, w, h);
      cachedW = w; cachedH = h;
    }
  }
  function drawNow(): void {
    ensureSize();
    gl!.uniform1f(uTime, (performance.now() - t0) / 1000);
    gl!.drawArrays(gl!.TRIANGLES, 0, 3);
    lastDraw = performance.now();
  }
  function loop(now: number): void {
    if (now - lastDraw >= FRAME_MS) drawNow();
    raf = requestAnimationFrame(loop);
  }

  resize();
  recolor();
  drawNow();
  const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;

  function start(): void {
    if (raf || reduced) return;
    if (document.hidden) return;
    raf = requestAnimationFrame(loop);
  }
  function stop(): void {
    if (raf) { cancelAnimationFrame(raf); raf = 0; }
  }

  function onResize(): void {
    // Force the cached size to miss next frame so ensureSize re-uploads
    // the resolution uniform.
    cachedW = 0; cachedH = 0;
    if (!raf) drawNow();
  }
  window.addEventListener('resize', onResize, { passive: true });

  function onVis(): void {
    if (document.hidden) stop();
    else start();
  }
  document.addEventListener('visibilitychange', onVis);

  return {
    start,
    stop,
    recolor() { recolor(); drawNow(); },
    destroy() {
      stop();
      window.removeEventListener('resize', onResize);
      document.removeEventListener('visibilitychange', onVis);
      probe.remove();
    },
  };
}
