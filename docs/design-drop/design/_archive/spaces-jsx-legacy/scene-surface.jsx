// scene-surface.jsx — the component surface for spaces.
// (1) Manifest: declarative model of a space
// (2) Customization vectors: inventory of the knobs Ship exposes
// (3) Studio: the editor — live preview + control panel

const SURF_W = 1200;
const SURF_H = 720;

// ──────────────────────────────────────────────────────────────
// JSON syntax-highlight helper — token-painted, no deps
// ──────────────────────────────────────────────────────────────
function JSONC({ src }) {
  const lines = src.split('\n');
  return (
    <pre style={{
      margin: 0,
      fontFamily: SHIP_MONO,
      fontSize: 12,
      lineHeight: 1.65,
      color: 'var(--foreground)',
      whiteSpace: 'pre',
      counterReset: 'ln',
    }}>
      {lines.map((line, i) => (
        <div key={i} style={{ display: 'grid', gridTemplateColumns: '36px 1fr', alignItems: 'baseline' }}>
          <span style={{ color: 'var(--muted-foreground)', opacity: 0.5, userSelect: 'none', textAlign: 'right', paddingRight: 12 }}>{i + 1}</span>
          <code dangerouslySetInnerHTML={{ __html: paintJSON(line) }} />
        </div>
      ))}
    </pre>
  );
}
function paintJSON(line) {
  // Comments first (// only here)
  if (/^\s*\/\//.test(line)) {
    return `<span style="color:var(--syntax-comment, oklch(0.55 0.05 100));font-style:italic">${escHtml(line)}</span>`;
  }
  let out = escHtml(line);
  // keys
  out = out.replace(/(&quot;[^&]+?&quot;)(\s*:)/g, (_, k, c) =>
    `<span style="color:var(--syntax-keyword, oklch(0.5 0.18 270))">${k}</span>${c}`);
  // strings
  out = out.replace(/:\s*(&quot;[^&]*?&quot;)/g, (_, s) =>
    `: <span style="color:var(--syntax-string, oklch(0.55 0.15 145))">${s}</span>`);
  // numbers
  out = out.replace(/:\s*(-?\d+\.?\d*)/g, (_, n) =>
    `: <span style="color:var(--syntax-number, oklch(0.55 0.18 35))">${n}</span>`);
  // booleans / null
  out = out.replace(/(\b(?:true|false|null)\b)/g, (_, w) =>
    `<span style="color:var(--syntax-keyword, oklch(0.55 0.18 35))">${w}</span>`);
  // trailing comment
  out = out.replace(/(\s)(\/\/[^<]*$)/, (_, sp, c) =>
    `${sp}<span style="color:var(--syntax-comment, oklch(0.55 0.05 100));font-style:italic">${c}</span>`);
  return out;
}
function escHtml(s) { return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;'); }

// ──────────────────────────────────────────────────────────────
// Artboard 1: space.jsonc manifest + what it produces
// ──────────────────────────────────────────────────────────────
const SAMPLE_MANIFEST = `// ~/Spaces/paperback/space.jsonc
{
  "id": "paperback",
  "name": "Paperback",
  "version": "0.4.1",

  // Identity — registered with the host OS launcher
  "icon": {
    "glyph": "P",
    "accent": "oklch(0.55 0.14 28)",
    "shape": "squircle"
  },

  // Theme + chrome preferences (override Ship defaults)
  "appearance": {
    "theme": "paper",
    "nav": "top",
    "dock": "off",
    "density": "comfortable"
  },

  // What this space opens to
  "home": "views/writing",

  // OS integration — Ship handles per-platform install
  "os": {
    "register": ["dock", "spotlight", "cli"],
    "windowDefaults": { "width": 1280, "height": 800 }
  }
}`;

function ManifestArt() {
  return (
    <WinFrame title="Studio" subtitle="paperback · manifest" style={{ width: SURF_W, height: SURF_H }}>
      <div style={{ flex: 1, display: 'grid', gridTemplateColumns: '1fr 1fr', minHeight: 0 }}>
        {/* Left: source */}
        <div style={{ padding: '20px 24px', borderRight: '1px solid var(--border)', display: 'flex', flexDirection: 'column', minHeight: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 14 }}>
            <MicroLabel>Source</MicroLabel>
            <span style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)' }}>space.jsonc</span>
            <span style={{ flex: 1 }} />
            <span style={{
              padding: '2px 8px', borderRadius: 4, fontSize: 10, fontFamily: SHIP_MONO,
              background: 'color-mix(in oklch, var(--info, oklch(0.65 0.15 240)) 12%, transparent)',
              color: 'var(--info, oklch(0.65 0.15 240))',
            }}>watch · live</span>
          </div>
          <div style={{
            flex: 1, minHeight: 0, overflow: 'auto',
            background: 'color-mix(in oklch, var(--card) 60%, var(--background))',
            border: '1px solid var(--border)',
            borderRadius: 10,
            padding: '14px 16px',
          }}>
            <JSONC src={SAMPLE_MANIFEST} />
          </div>
          <div style={{
            marginTop: 14, padding: '10px 12px', borderRadius: 8,
            border: '1px solid var(--border)',
            background: 'var(--card)',
            fontFamily: SHIP_MONO, fontSize: 11.5,
            color: 'var(--muted-foreground)',
            display: 'flex', alignItems: 'center', gap: 10,
          }}>
            <span style={{ color: 'var(--success, oklch(0.55 0.13 145))' }}>✓</span>
            valid · registers on save · 4 OS integrations
          </div>
        </div>

        {/* Right: produced */}
        <div style={{ padding: '20px 24px', display: 'flex', flexDirection: 'column', gap: 14, minHeight: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <MicroLabel>What this declares</MicroLabel>
            <span style={{ flex: 1 }} />
            <span style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)' }}>
              → applied
            </span>
          </div>

          {/* OS launcher row */}
          <RegisteredRow
            title="Host OS launcher"
            sub="dock · spotlight · CLI alias"
            preview={
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '6px 10px', background: 'color-mix(in oklch, var(--foreground) 6%, transparent)', borderRadius: 8 }}>
                <SpaceMark size={28} accent="oklch(0.55 0.14 28)" char="P" />
                <span style={{ fontSize: 12.5, fontFamily: SHIP_FONT }}>Paperback</span>
                <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)', marginLeft: 'auto' }}>installed</span>
              </div>
            }
          />

          {/* Window chrome row */}
          <RegisteredRow
            title="Window chrome"
            sub="paper theme · top nav · no dock"
            preview={
              <div style={{
                border: '1px solid var(--border)', borderRadius: 6, overflow: 'hidden',
                background: '#f4ece0',
              }}>
                <div style={{ height: 12, background: '#fbf6ec', borderBottom: '1px solid #d6c8b0', display: 'flex', alignItems: 'center', gap: 3, paddingLeft: 5 }}>
                  <span style={{ width: 5, height: 5, borderRadius: '50%', background: '#ff5f57' }} />
                  <span style={{ width: 5, height: 5, borderRadius: '50%', background: '#febc2e' }} />
                  <span style={{ width: 5, height: 5, borderRadius: '50%', background: '#28c840' }} />
                </div>
                <div style={{ height: 10, background: 'rgba(0,0,0,0.05)', display: 'flex', gap: 8, padding: '2px 8px', alignItems: 'center' }}>
                  <span style={{ width: 14, height: 2, background: '#a14b30', borderRadius: 1 }} />
                  <span style={{ width: 12, height: 2, background: '#7a6a55', borderRadius: 1 }} />
                  <span style={{ width: 18, height: 2, background: '#7a6a55', borderRadius: 1 }} />
                </div>
                <div style={{ height: 30, padding: 6 }}>
                  <span style={{ display: 'block', width: '80%', height: 3, background: 'rgba(0,0,0,0.1)', borderRadius: 1, marginBottom: 3 }} />
                  <span style={{ display: 'block', width: '60%', height: 3, background: 'rgba(0,0,0,0.1)', borderRadius: 1, marginBottom: 3 }} />
                  <span style={{ display: 'block', width: '70%', height: 3, background: 'rgba(0,0,0,0.1)', borderRadius: 1 }} />
                </div>
              </div>
            }
          />

          {/* Entry view row */}
          <RegisteredRow
            title="Default view on launch"
            sub="views/writing"
            preview={
              <div style={{
                padding: '6px 10px', borderRadius: 6,
                fontFamily: SHIP_MONO, fontSize: 11,
                background: 'color-mix(in oklch, var(--foreground) 5%, transparent)',
                color: 'var(--foreground)',
                display: 'flex', alignItems: 'center', gap: 6,
              }}>
                <span style={{ color: 'var(--muted-foreground)' }}>→</span>
                /writing
                <span style={{ marginLeft: 'auto', color: 'var(--muted-foreground)', fontSize: 10 }}>(resolves to /paperback/writing)</span>
              </div>
            }
          />

          <div style={{ flex: 1, minHeight: 0 }} />

          {/* What Ship handles for you */}
          <div style={{
            padding: '12px 14px', borderRadius: 10,
            background: 'color-mix(in oklch, var(--card) 50%, var(--background))',
            border: '1px solid var(--border)',
          }}>
            <MicroLabel style={{ marginBottom: 8 }}>Ship handles for you</MicroLabel>
            <ul style={{ margin: 0, padding: 0, listStyle: 'none', fontSize: 12, lineHeight: 1.65, color: 'var(--muted-foreground)', display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '2px 16px', fontFamily: SHIP_MONO }}>
              <li>·  .app bundle (macOS)</li>
              <li>·  .desktop entry (linux)</li>
              <li>·  Start menu (windows)</li>
              <li>·  Window restore + state</li>
              <li>·  PATH alias <span style={{ color: 'var(--foreground)' }}>ship open ...</span></li>
              <li>·  Uninstall cleanup</li>
            </ul>
          </div>
        </div>
      </div>
    </WinFrame>
  );
}

function RegisteredRow({ title, sub, preview }) {
  return (
    <div style={{
      padding: '12px 14px', borderRadius: 10,
      border: '1px solid var(--border)',
      background: 'var(--card)',
      display: 'grid', gridTemplateColumns: '1fr 220px', gap: 14, alignItems: 'center',
    }}>
      <div>
        <div style={{ fontSize: 13, fontWeight: 600, letterSpacing: '-0.005em' }}>{title}</div>
        <div style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)', marginTop: 2 }}>{sub}</div>
      </div>
      <div>{preview}</div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 2: Studio · Space editor (live preview + control panel)
// ──────────────────────────────────────────────────────────────
function StudioEditor() {
  return (
    <WinFrame title="Studio" subtitle="customize · paperback" style={{ width: SURF_W, height: SURF_H }}>
      {/* Top crumb */}
      <div style={{
        height: 42, flexShrink: 0,
        display: 'flex', alignItems: 'center', gap: 10,
        padding: '0 18px',
        borderBottom: '1px solid var(--border)',
        background: 'color-mix(in oklch, var(--background) 90%, var(--card))',
        fontSize: 12.5,
      }}>
        <span style={{ color: 'var(--muted-foreground)' }}>Studio</span>
        <span style={{ color: 'var(--muted-foreground)' }}>›</span>
        <span style={{ color: 'var(--muted-foreground)' }}>Spaces</span>
        <span style={{ color: 'var(--muted-foreground)' }}>›</span>
        <SpaceMark size={18} accent="oklch(0.55 0.14 28)" char="P" />
        <span style={{ fontWeight: 600 }}>Paperback</span>
        <span style={{ flex: 1 }} />
        <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6, fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>
          <Dot tone="live" size={6} /> live · changes applied to window
        </span>
        <div style={{ width: 1, height: 16, background: 'var(--border)', margin: '0 4px' }} />
        <button style={{ padding: '4px 10px', borderRadius: 6, border: '1px solid var(--border)', background: 'var(--background)', fontSize: 11, fontFamily: SHIP_MONO, cursor: 'pointer' }}>Revert</button>
        <button style={{ padding: '4px 10px', borderRadius: 6, border: '1px solid var(--primary)', background: 'var(--primary)', color: 'var(--primary-foreground)', fontSize: 11, fontWeight: 600, cursor: 'pointer' }}>Save manifest</button>
      </div>

      {/* Body: editor sections (left) + live preview (right) */}
      <div style={{ flex: 1, display: 'grid', gridTemplateColumns: '340px 1fr', minHeight: 0 }}>
        {/* Controls */}
        <div style={{
          borderRight: '1px solid var(--border)',
          overflow: 'auto',
          background: 'color-mix(in oklch, var(--background) 96%, var(--card))',
        }}>
          <Section title="Identity" />
          <SectionRow label="Name">
            <Input value="Paperback" />
          </SectionRow>
          <SectionRow label="Glyph">
            <div style={{ display: 'flex', gap: 5, flexWrap: 'wrap' }}>
              {['P', '◆', '✦', '▲', 'M', '⌘'].map((c, i) => (
                <div key={c} style={{
                  width: 30, height: 30, borderRadius: 6,
                  background: i === 0 ? 'color-mix(in oklch, oklch(0.55 0.14 28) 14%, var(--card))' : 'var(--card)',
                  border: i === 0 ? '1.5px solid oklch(0.55 0.14 28)' : '1px solid var(--border)',
                  display: 'grid', placeItems: 'center',
                  fontFamily: SHIP_MONO, fontWeight: 700, fontSize: 13,
                  color: i === 0 ? 'oklch(0.55 0.14 28)' : 'var(--foreground)',
                  cursor: 'pointer',
                }}>{c}</div>
              ))}
            </div>
          </SectionRow>
          <SectionRow label="Accent">
            <div style={{ display: 'flex', gap: 7 }}>
              {[
                'oklch(0.55 0.14 28)',
                'oklch(0.7 0.16 140)',
                'oklch(0.6 0.16 240)',
                'oklch(0.62 0.18 305)',
                'oklch(0.65 0.18 80)',
                'oklch(0.55 0.05 240)',
              ].map((s, i) => (
                <div key={i} style={{
                  width: 22, height: 22, borderRadius: '50%',
                  background: s,
                  boxShadow: i === 0 ? '0 0 0 2px var(--background), 0 0 0 4px ' + s : 'inset 0 0 0 1px rgba(0,0,0,0.1)',
                  cursor: 'pointer',
                }} />
              ))}
            </div>
          </SectionRow>

          <Section title="Appearance" hint="space owns its theme" />
          <SectionRow label="Theme">
            <Select value="paper" options={['ship-light', 'ship-dark', 'paper', 'halo', 'console', 'contrast', 'glass']} />
          </SectionRow>
          <SectionRow label="Density">
            <Seg value="comfortable" options={['compact', 'comfortable', 'spacious']} />
          </SectionRow>

          <Section title="Chrome" hint="layout regions only · A in your RFC" />
          <SectionRow label="Navigation">
            <Seg value="top" options={['rail', 'top', 'none']} />
          </SectionRow>
          <SectionRow label="Dock">
            <Seg value="off" options={['left', 'center', 'right', 'off']} />
          </SectionRow>
          <SectionRow label="Status bar">
            <Toggle value={true} />
          </SectionRow>

          <Section title="OS integration" hint="registered on save" />
          <SectionRow label="Dock entry">
            <Toggle value={true} />
          </SectionRow>
          <SectionRow label="Spotlight">
            <Toggle value={true} />
          </SectionRow>
          <SectionRow label="CLI alias">
            <Input value="ship open paperback" mono />
          </SectionRow>
          <SectionRow label="Default size">
            <div style={{ display: 'flex', gap: 6 }}>
              <Input value="1280" mono small />
              <span style={{ alignSelf: 'center', color: 'var(--muted-foreground)' }}>×</span>
              <Input value="800" mono small />
            </div>
          </SectionRow>

          <Section title="Advanced" hint="future · vector C" />
          <SectionRow label="Custom chrome">
            <Toggle value={false} disabled />
          </SectionRow>
          <div style={{ padding: '8px 16px 20px', fontSize: 11, color: 'var(--muted-foreground)', fontFamily: SHIP_MONO, lineHeight: 1.5 }}>
            Replace the entire chrome scaffold with your own <code>{'<sui-view-chrome>'}</code> module. Coming soon.
          </div>
        </div>

        {/* Live preview */}
        <div style={{
          padding: 28,
          background: 'repeating-linear-gradient(45deg, color-mix(in oklch, var(--background) 96%, var(--foreground)) 0, color-mix(in oklch, var(--background) 96%, var(--foreground)) 2px, var(--background) 2px, var(--background) 14px)',
          display: 'flex', flexDirection: 'column', gap: 14, minHeight: 0, overflow: 'hidden',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10, color: 'var(--muted-foreground)' }}>
            <MicroLabel>Live preview</MicroLabel>
            <span style={{ flex: 1 }} />
            <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5 }}>1280×800 · scaled to fit</span>
          </div>

          {/* Pretend window preview — paper theme, top nav, no dock */}
          <div style={{
            flex: 1, minHeight: 0,
            background: '#f4ece0',
            border: '1px solid #c0a78a',
            borderRadius: 8,
            overflow: 'hidden',
            display: 'flex', flexDirection: 'column',
            boxShadow: '0 20px 60px -20px rgba(0,0,0,0.25)',
          }}>
            {/* fake title bar */}
            <div style={{ height: 30, background: '#fbf6ec', borderBottom: '1px solid #d6c8b0', display: 'grid', gridTemplateColumns: '1fr auto 1fr', alignItems: 'center', padding: '0 10px', flexShrink: 0 }}>
              <div style={{ display: 'flex', gap: 6 }}>
                <span style={{ width: 11, height: 11, borderRadius: '50%', background: '#ff5f57' }} />
                <span style={{ width: 11, height: 11, borderRadius: '50%', background: '#febc2e' }} />
                <span style={{ width: 11, height: 11, borderRadius: '50%', background: '#28c840' }} />
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 7, justifySelf: 'center', color: '#2d1f12', fontFamily: SHIP_FONT }}>
                <SpaceMark size={14} accent="oklch(0.55 0.14 28)" char="P" />
                <span style={{ fontSize: 11.5, fontWeight: 600 }}>Paperback</span>
              </div>
              <div />
            </div>

            {/* top nav */}
            <div style={{
              height: 32, flexShrink: 0,
              background: '#fbf6ec', borderBottom: '1px solid #d6c8b0',
              display: 'flex', alignItems: 'center', gap: 14,
              padding: '0 16px', fontFamily: SHIP_FONT, fontSize: 11.5,
              color: '#7a6a55',
            }}>
              <span style={{ color: '#a14b30', fontWeight: 600, position: 'relative' }}>
                Posts
                <span style={{ position: 'absolute', bottom: -8, left: 0, right: 0, height: 2, background: '#a14b30' }} />
              </span>
              <span>Drafts</span>
              <span>Schedule</span>
              <span>Analytics</span>
              <span style={{ flex: 1 }} />
              <span style={{ padding: '3px 8px', background: '#a14b30', color: '#fbf6ec', borderRadius: 4, fontWeight: 600, fontSize: 10.5 }}>Publish</span>
            </div>

            {/* content stub */}
            <div style={{
              flex: 1, padding: '20px 36px',
              fontFamily: 'Georgia, serif',
              color: '#2d1f12',
              display: 'flex', flexDirection: 'column', gap: 8,
              overflow: 'hidden',
            }}>
              <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: '#7a6a55', letterSpacing: '0.12em', textTransform: 'uppercase' }}>
                Draft · 4m ago
              </div>
              <div style={{ fontSize: 22, fontWeight: 600, letterSpacing: '-0.01em', lineHeight: 1.15 }}>
                On the slow death of the workspace tab
              </div>
              <div style={{ fontSize: 13, lineHeight: 1.6, color: 'color-mix(in oklch, #2d1f12 75%, transparent)' }}>
                For most of the last decade, the workspace has been a tab — one of many — inside a
                browser-shaped vessel called The App.
              </div>
              <div style={{ display: 'flex', gap: 5, marginTop: 6 }}>
                {['workspaces', 'os', 'focus'].map(t => (
                  <span key={t} style={{ padding: '1px 7px', borderRadius: 999, border: '1px solid #d6c8b0', background: '#fbf6ec', fontSize: 10, color: '#7a6a55', fontFamily: SHIP_MONO }}>{t}</span>
                ))}
              </div>
            </div>
          </div>

          {/* diff stack — small chip showing what changed */}
          <div style={{
            display: 'flex', gap: 8, alignItems: 'center',
            padding: '8px 12px', borderRadius: 8,
            background: 'color-mix(in oklch, var(--card) 70%, var(--background))',
            border: '1px solid var(--border)',
            fontFamily: SHIP_MONO, fontSize: 11,
            color: 'var(--muted-foreground)',
            flexShrink: 0,
          }}>
            <span style={{ color: 'var(--primary)', fontWeight: 600 }}>3 unsaved</span>
            <span>theme: <s style={{ opacity: 0.5 }}>ship-light</s> → <b style={{ color: 'var(--foreground)' }}>paper</b></span>
            <span>·</span>
            <span>nav: <s style={{ opacity: 0.5 }}>rail</s> → <b style={{ color: 'var(--foreground)' }}>top</b></span>
            <span>·</span>
            <span>dock: <s style={{ opacity: 0.5 }}>center</s> → <b style={{ color: 'var(--foreground)' }}>off</b></span>
          </div>
        </div>
      </div>
    </WinFrame>
  );
}

function Section({ title, hint }) {
  return (
    <div style={{
      padding: '14px 16px 6px',
      borderTop: '1px solid color-mix(in oklch, var(--border) 60%, transparent)',
      display: 'flex', alignItems: 'baseline', gap: 8,
    }}>
      <MicroLabel>{title}</MicroLabel>
      {hint ? <span style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)', opacity: 0.7 }}>· {hint}</span> : null}
    </div>
  );
}
function SectionRow({ label, children }) {
  return (
    <div style={{ padding: '6px 16px 10px', display: 'grid', gridTemplateColumns: '90px 1fr', gap: 12, alignItems: 'center' }}>
      <div style={{ fontSize: 12, color: 'var(--muted-foreground)' }}>{label}</div>
      <div>{children}</div>
    </div>
  );
}
function Input({ value, mono = false, small = false }) {
  return (
    <div style={{
      padding: small ? '4px 8px' : '5px 10px',
      border: '1px solid var(--border)',
      borderRadius: 6,
      background: 'var(--background)',
      fontSize: small ? 11.5 : 12,
      fontFamily: mono ? SHIP_MONO : SHIP_FONT,
      color: 'var(--foreground)',
      width: small ? 72 : '100%',
    }}>{value}</div>
  );
}
function Select({ value, options }) {
  return (
    <div style={{
      padding: '5px 10px',
      border: '1px solid var(--border)',
      borderRadius: 6,
      background: 'var(--background)',
      fontSize: 12, fontFamily: SHIP_FONT,
      color: 'var(--foreground)',
      display: 'flex', alignItems: 'center', gap: 6,
    }}>
      <span style={{ flex: 1 }}>{value}</span>
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M6 9l6 6 6-6" /></svg>
    </div>
  );
}
function Seg({ value, options }) {
  return (
    <div style={{
      display: 'grid',
      gridTemplateColumns: `repeat(${options.length}, 1fr)`,
      border: '1px solid var(--border)',
      borderRadius: 6, overflow: 'hidden',
      background: 'var(--background)',
    }}>
      {options.map((o, i) => (
        <div key={o} style={{
          padding: '5px 0',
          textAlign: 'center',
          fontSize: 11.5,
          fontFamily: SHIP_MONO,
          background: o === value ? 'var(--foreground)' : 'transparent',
          color: o === value ? 'var(--background)' : 'var(--foreground)',
          borderRight: i < options.length - 1 ? '1px solid var(--border)' : 'none',
          cursor: 'pointer',
        }}>{o}</div>
      ))}
    </div>
  );
}
function Toggle({ value, disabled = false }) {
  return (
    <div style={{
      width: 32, height: 18, borderRadius: 9,
      background: value ? 'var(--primary)' : 'color-mix(in oklch, var(--foreground) 14%, transparent)',
      position: 'relative',
      opacity: disabled ? 0.4 : 1,
      cursor: disabled ? 'not-allowed' : 'pointer',
    }}>
      <span style={{
        position: 'absolute',
        top: 2, left: value ? 16 : 2,
        width: 14, height: 14, borderRadius: '50%', background: 'white',
        boxShadow: '0 1px 2px rgba(0,0,0,0.2)',
      }} />
    </div>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 3: Customization vectors — inventory diagram
// ──────────────────────────────────────────────────────────────
function VectorsArt() {
  const w = 920, h = 560;
  return (
    <WinFrame title="Studio" subtitle="customization vectors · reference" style={{ width: w, height: h }}>
      <div style={{ flex: 1, padding: '24px 28px', display: 'flex', flexDirection: 'column', gap: 14, minHeight: 0, overflow: 'auto' }}>
        <div>
          <div style={{ fontSize: 18, fontWeight: 600, letterSpacing: '-0.01em' }}>What a space can change</div>
          <div style={{ color: 'var(--muted-foreground)', fontSize: 13, marginTop: 4 }}>
            Three vectors, gated by tier. Today only <b style={{ color: 'var(--foreground)' }}>A</b> ships — <i>B</i> next, <i>C</i> later.
          </div>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 14, flex: 1, minHeight: 0 }}>
          <VectorCard
            tier="A" available
            title="Theme tokens"
            sub="ships now"
            color="oklch(0.55 0.13 145)"
            items={[
              ['theme', 'one of 7 built-in or marketplace'],
              ['accent', 'override --primary'],
              ['density', 'compact · comfortable · spacious'],
              ['icon', 'glyph + accent + shape'],
            ]}
          />
          <VectorCard
            tier="B" available={false}
            title="Layout regions"
            sub="next milestone"
            color="oklch(0.65 0.15 240)"
            items={[
              ['nav', 'rail · top · none'],
              ['dock', 'left · center · right · off'],
              ['status bar', 'on · off'],
              ['command bar', 'k · double-tap mod · off'],
            ]}
          />
          <VectorCard
            tier="C" available={false}
            title="Custom chrome"
            sub="future"
            color="oklch(0.6 0.18 280)"
            items={[
              ['<sui-view-chrome>', 'full takeover of the window body'],
              ['scaffold', 'window · IPC · runtime are still Ship'],
              ['examples', 'terminal · canvas · doc · custom OS'],
            ]}
          />
        </div>

        <div style={{
          padding: '12px 16px', borderRadius: 10,
          background: 'color-mix(in oklch, var(--card) 70%, var(--background))',
          border: '1px solid var(--border)',
          fontSize: 12, color: 'var(--muted-foreground)',
          lineHeight: 1.6,
          display: 'flex', alignItems: 'flex-start', gap: 10,
        }}>
          <span style={{ color: 'var(--primary)', fontFamily: SHIP_MONO, fontSize: 11, fontWeight: 600 }}>RFC</span>
          <span>
            Vector A (theme + identity) is the floor. B (layout regions) earns its way in once we've watched
            real spaces in the wild for a quarter. C (custom chrome) is a question of whether a space can
            stop looking like a Ship space — open until we've seen what people try to do.
          </span>
        </div>
      </div>
    </WinFrame>
  );
}

function VectorCard({ tier, title, sub, color, items, available }) {
  return (
    <div style={{
      border: '1px solid ' + (available ? 'color-mix(in oklch, ' + color + ' 35%, var(--border))' : 'var(--border)'),
      borderRadius: 10, padding: 14,
      background: 'var(--card)',
      display: 'flex', flexDirection: 'column', gap: 10,
      opacity: available ? 1 : 0.72,
      position: 'relative',
      overflow: 'hidden',
    }}>
      <span style={{
        position: 'absolute', top: 0, left: 0, width: 36, height: 3, background: color,
      }} />
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <div style={{
          width: 26, height: 26, borderRadius: 6,
          background: 'color-mix(in oklch, ' + color + ' 14%, transparent)',
          color: color,
          display: 'grid', placeItems: 'center',
          fontFamily: SHIP_MONO, fontWeight: 700, fontSize: 14,
        }}>{tier}</div>
        <div>
          <div style={{ fontSize: 13.5, fontWeight: 600 }}>{title}</div>
          <div style={{ fontSize: 10.5, fontFamily: SHIP_MONO, color: 'var(--muted-foreground)', letterSpacing: '0.06em', textTransform: 'uppercase' }}>{sub}</div>
        </div>
        {available ? (
          <span style={{ marginLeft: 'auto', fontFamily: SHIP_MONO, fontSize: 10, color, padding: '2px 7px', borderRadius: 999, background: 'color-mix(in oklch, ' + color + ' 14%, transparent)' }}>ready</span>
        ) : (
          <span style={{ marginLeft: 'auto', fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)', padding: '2px 7px', borderRadius: 999, border: '1px solid var(--border)' }}>later</span>
        )}
      </div>
      <ul style={{ margin: 0, padding: 0, listStyle: 'none', display: 'flex', flexDirection: 'column', gap: 6 }}>
        {items.map(([k, v]) => (
          <li key={k} style={{ display: 'grid', gridTemplateColumns: '94px 1fr', gap: 10, fontSize: 11.5, alignItems: 'baseline' }}>
            <span style={{ fontFamily: SHIP_MONO, color: color, fontWeight: 600 }}>{k}</span>
            <span style={{ color: 'var(--muted-foreground)' }}>{v}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}

Object.assign(window, { ManifestArt, StudioEditor, VectorsArt, JSONC, paintJSON });
