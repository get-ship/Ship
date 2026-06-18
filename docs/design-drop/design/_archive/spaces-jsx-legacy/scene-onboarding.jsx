// scene-onboarding.jsx — first-run + create-space flow.
// Four small artboards composing the wizard sequence.

const WIZ_W = 640;
const WIZ_H = 560;

// Step shell — title bar + content area + footer with progress + actions
function WizCard({ step, total, title, footerPrimary, footerSecondary, children, accent }) {
  return (
    <WinFrame title="Ship" subtitle={step ? `setup · ${step} of ${total}` : 'welcome'} style={{ width: WIZ_W, height: WIZ_H }}>
      <div style={{
        flex: 1, padding: '28px 32px 0', display: 'flex', flexDirection: 'column', gap: 18,
        background: 'var(--background)',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          {step ? <MicroLabel>Step {step}</MicroLabel> : null}
          {step ? <span style={{ flex: 1, height: 2, borderRadius: 1, background: 'var(--border)', position: 'relative' }}>
            <span style={{ position: 'absolute', left: 0, top: 0, bottom: 0, width: `${(step / total) * 100}%`, background: accent || 'var(--primary)', borderRadius: 1 }} />
          </span> : null}
        </div>
        <div>
          <h2 style={{
            margin: 0, fontSize: 22, fontWeight: 600, letterSpacing: '-0.015em',
            textWrap: 'balance',
          }}>{title}</h2>
        </div>
        <div style={{ flex: 1, minHeight: 0, overflow: 'hidden' }}>{children}</div>
      </div>
      <footer style={{
        padding: '14px 32px 18px',
        display: 'flex', alignItems: 'center', gap: 8,
        borderTop: '1px solid var(--border)',
      }}>
        {footerSecondary}
        <div style={{ flex: 1 }} />
        {footerPrimary}
      </footer>
    </WinFrame>
  );
}

function PrimaryBtn({ children }) {
  return (
    <button style={{
      padding: '7px 16px', borderRadius: 8,
      border: '1px solid var(--primary)',
      background: 'var(--primary)', color: 'var(--primary-foreground)',
      fontSize: 12.5, fontWeight: 600, fontFamily: SHIP_FONT,
      cursor: 'pointer',
    }}>{children}</button>
  );
}
function GhostBtn({ children }) {
  return (
    <button style={{
      padding: '7px 14px', borderRadius: 8,
      border: '1px solid var(--border)',
      background: 'transparent', color: 'var(--foreground)',
      fontSize: 12.5, fontWeight: 500, fontFamily: SHIP_FONT,
      cursor: 'pointer',
    }}>{children}</button>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 1: First-run welcome with disclaimer
// ──────────────────────────────────────────────────────────────
function OnbWelcome() {
  return (
    <WinFrame title="Ship" subtitle="welcome" style={{ width: WIZ_W, height: WIZ_H }}>
      <div style={{
        flex: 1, padding: '36px 36px 0',
        display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: 20,
      }}>
        <div style={{
          display: 'flex', alignItems: 'center', gap: 14, padding: '4px 0',
        }}>
          <SpaceMark size={44} accent="var(--primary)" char="S" />
          <div>
            <div style={{ fontSize: 26, fontWeight: 600, letterSpacing: '-0.02em', lineHeight: 1.1 }}>
              Welcome to Ship.
            </div>
            <div style={{ color: 'var(--muted-foreground)', fontFamily: SHIP_MONO, fontSize: 11.5, marginTop: 4, letterSpacing: '0.04em' }}>
              v1.4.2 · macOS · hub installed
            </div>
          </div>
        </div>

        <div style={{ fontSize: 14.5, lineHeight: 1.55, color: 'var(--foreground)', maxWidth: 480, textWrap: 'pretty' }}>
          Ship lives in your applications dock. Inside Ship, every <b>space</b> you create
          becomes its own app — its own window, its own theme, its own icon in your OS launcher.
        </div>

        <ul style={{
          margin: 0, padding: 0, listStyle: 'none',
          display: 'flex', flexDirection: 'column', gap: 10,
          fontSize: 13.5, color: 'var(--foreground)', maxWidth: 480,
        }}>
          {[
            ['Ship hub', 'this app — manage spaces, account, marketplace, system settings, activity.'],
            ['Spaces', 'each one runs in its own window. Switch between them like any other app.'],
            ['Studio', 'inside the hub — customize a space’s theme, dock, chrome.'],
          ].map(([k, v]) => (
            <li key={k} style={{ display: 'grid', gridTemplateColumns: '110px 1fr', gap: 14, alignItems: 'baseline' }}>
              <span style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--primary)', fontWeight: 600, letterSpacing: '0.04em' }}>{k}</span>
              <span style={{ color: 'var(--muted-foreground)' }}>{v}</span>
            </li>
          ))}
        </ul>

        <div style={{
          marginTop: 'auto', marginBottom: 0,
          padding: '10px 12px',
          borderRadius: 8,
          border: '1px solid color-mix(in oklch, var(--info, oklch(0.65 0.15 240)) 35%, var(--border))',
          background: 'color-mix(in oklch, var(--info, oklch(0.65 0.15 240)) 6%, var(--card))',
          fontSize: 12, color: 'var(--muted-foreground)', fontFamily: SHIP_MONO,
          display: 'flex', alignItems: 'flex-start', gap: 8, maxWidth: 480,
        }}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ marginTop: 1, color: 'var(--info, oklch(0.65 0.15 240))' }}>
            <circle cx="12" cy="12" r="9" /><path d="M12 8h.01M11 12h1v5h1" strokeLinecap="round" />
          </svg>
          <span>
            Heads-up: Ship will register each space as an application on your system
            (Applications · Start Menu · Activities). You can uninstall them from the hub.
          </span>
        </div>
      </div>
      <footer style={{
        padding: '14px 36px 18px', display: 'flex', alignItems: 'center', gap: 8,
        borderTop: '1px solid var(--border)',
      }}>
        <GhostBtn>Skip — explore the hub</GhostBtn>
        <div style={{ flex: 1 }} />
        <PrimaryBtn>Create your first space →</PrimaryBtn>
      </footer>
    </WinFrame>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 2: Step 1 — name + icon
// ──────────────────────────────────────────────────────────────
const ICON_CHARS = ['S', 'P', 'M', 'L', 'V', '⌘', '◆', '▲', '✦', '◐'];
const ACCENT_SWATCHES = [
  'oklch(0.62 0.16 35)',
  'oklch(0.55 0.14 28)',
  'oklch(0.7 0.16 140)',
  'oklch(0.6 0.16 240)',
  'oklch(0.62 0.18 305)',
  'oklch(0.6 0.18 280)',
  'oklch(0.55 0.05 240)',
  'oklch(0.65 0.18 80)',
];

function OnbStep1() {
  const accent = ACCENT_SWATCHES[2]; // chosen: green
  const char = '◆';
  return (
    <WizCard
      step={1} total={3} accent={accent}
      title="Name your space."
      footerSecondary={<GhostBtn>← Back</GhostBtn>}
      footerPrimary={<PrimaryBtn>Continue →</PrimaryBtn>}
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>

        {/* Live preview */}
        <div style={{
          display: 'flex', alignItems: 'center', gap: 14,
          padding: '14px 16px',
          background: 'color-mix(in oklch, var(--card) 75%, transparent)',
          border: '1px solid var(--border)',
          borderRadius: 10,
        }}>
          <SpaceMark size={46} accent={accent} char={char} />
          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{ fontSize: 15, fontWeight: 600 }}>Brand Voice</div>
            <div style={{ color: 'var(--muted-foreground)', fontFamily: SHIP_MONO, fontSize: 11 }}>
              ~/Spaces/brand-voice
            </div>
          </div>
          <MicroLabel>preview</MicroLabel>
        </div>

        {/* Name input */}
        <div>
          <label style={{ display: 'block', marginBottom: 6 }}>
            <MicroLabel>Name</MicroLabel>
          </label>
          <div style={{
            padding: '8px 12px',
            border: '1.5px solid var(--primary)',
            borderRadius: 8,
            background: 'var(--background)',
            fontSize: 14, fontFamily: SHIP_FONT,
            display: 'flex', alignItems: 'center', gap: 8,
            color: 'var(--foreground)',
          }}>
            Brand Voice
            <span style={{ display: 'inline-block', width: 1.5, height: 16, background: 'var(--primary)', animation: 'sb-pulse 1.2s linear infinite' }} />
          </div>
        </div>

        {/* Icon picker */}
        <div>
          <label style={{ display: 'block', marginBottom: 8 }}>
            <MicroLabel>Glyph &amp; accent</MicroLabel>
          </label>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(10, 1fr)', gap: 6, marginBottom: 10 }}>
            {ICON_CHARS.map((c, i) => (
              <div key={c} style={{
                aspectRatio: '1', borderRadius: 6,
                background: i === 6 ? 'color-mix(in oklch, ' + accent + ' 12%, transparent)' : 'var(--card)',
                border: i === 6 ? '1.5px solid ' + accent : '1px solid var(--border)',
                display: 'grid', placeItems: 'center',
                fontFamily: SHIP_MONO, fontWeight: 700, fontSize: 14,
                color: i === 6 ? accent : 'var(--foreground)',
                cursor: 'pointer',
              }}>{c}</div>
            ))}
          </div>
          <div style={{ display: 'flex', gap: 8 }}>
            {ACCENT_SWATCHES.map((s, i) => (
              <div key={i} style={{
                width: 28, height: 28, borderRadius: '50%',
                background: s,
                boxShadow: i === 2 ? '0 0 0 2px var(--background), 0 0 0 4px ' + s : 'inset 0 0 0 1px color-mix(in oklch, black 10%, transparent)',
                cursor: 'pointer',
              }} />
            ))}
          </div>
        </div>
      </div>
    </WizCard>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 3: Step 2 — theme + layout preferences
// ──────────────────────────────────────────────────────────────
const THEME_OPTIONS = [
  { id: 'ship-light', label: 'Light', bg: '#faf7f1', fg: '#2a261c', accent: '#c96442' },
  { id: 'ship-dark', label: 'Dark', bg: '#1c1b18', fg: '#e8e3d4', accent: '#e0a16b' },
  { id: 'paper', label: 'Paper', bg: '#f4ece0', fg: '#2d1f12', accent: '#a14b30' },
  { id: 'halo', label: 'Halo', bg: '#e9f0f7', fg: '#1d2632', accent: '#5a8fbf' },
  { id: 'console', label: 'Console', bg: '#0b0d0a', fg: '#aef39e', accent: '#7be36b' },
  { id: 'contrast', label: 'Contrast', bg: '#0d0d0e', fg: '#f3e8c4', accent: '#e0a248' },
];

function OnbStep2() {
  const accent = ACCENT_SWATCHES[2];
  const selectedTheme = 'halo';
  const selectedDock = 'centered';
  const selectedNav = 'rail';
  return (
    <WizCard
      step={2} total={3} accent={accent}
      title="Pick its look."
      footerSecondary={<GhostBtn>← Back</GhostBtn>}
      footerPrimary={<PrimaryBtn>Continue →</PrimaryBtn>}
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
        {/* Theme grid */}
        <div>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
            <MicroLabel>Theme</MicroLabel>
            <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>
              7 built-in · marketplace has more
            </span>
          </div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(6, 1fr)', gap: 8 }}>
            {THEME_OPTIONS.map(t => (
              <div key={t.id} style={{
                border: t.id === selectedTheme ? '1.5px solid var(--primary)' : '1px solid var(--border)',
                borderRadius: 8, overflow: 'hidden',
                cursor: 'pointer',
                boxShadow: t.id === selectedTheme ? '0 0 0 3px color-mix(in oklch, var(--primary) 15%, transparent)' : 'none',
              }}>
                <div style={{ height: 44, background: t.bg, position: 'relative' }}>
                  <span style={{ position: 'absolute', top: 6, left: 6, width: 18, height: 4, borderRadius: 2, background: t.accent }} />
                  <span style={{ position: 'absolute', top: 14, left: 6, width: 28, height: 3, borderRadius: 1.5, background: t.fg, opacity: 0.7 }} />
                  <span style={{ position: 'absolute', top: 21, left: 6, width: 22, height: 3, borderRadius: 1.5, background: t.fg, opacity: 0.4 }} />
                </div>
                <div style={{ padding: '6px 8px', fontSize: 11, fontFamily: SHIP_MONO, color: 'var(--foreground)' }}>
                  {t.label}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Layout prefs */}
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 14 }}>
          <div>
            <MicroLabel style={{ marginBottom: 8 }}>Navigation</MicroLabel>
            <div style={{ display: 'flex', gap: 6 }}>
              {['rail', 'top', 'none'].map(v => (
                <div key={v} style={{
                  flex: 1,
                  padding: '12px 8px',
                  border: v === selectedNav ? '1.5px solid var(--primary)' : '1px solid var(--border)',
                  borderRadius: 8,
                  background: v === selectedNav ? 'color-mix(in oklch, var(--primary) 8%, var(--card))' : 'var(--card)',
                  cursor: 'pointer',
                  display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 8,
                  fontFamily: SHIP_MONO, fontSize: 11,
                }}>
                  <NavMini kind={v} />
                  <span>{v}</span>
                </div>
              ))}
            </div>
          </div>
          <div>
            <MicroLabel style={{ marginBottom: 8 }}>Dock</MicroLabel>
            <div style={{ display: 'flex', gap: 6 }}>
              {['left', 'centered', 'right', 'off'].map(v => (
                <div key={v} style={{
                  flex: 1,
                  padding: '12px 6px',
                  border: v === selectedDock ? '1.5px solid var(--primary)' : '1px solid var(--border)',
                  borderRadius: 8,
                  background: v === selectedDock ? 'color-mix(in oklch, var(--primary) 8%, var(--card))' : 'var(--card)',
                  cursor: 'pointer',
                  display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 8,
                  fontFamily: SHIP_MONO, fontSize: 10.5,
                }}>
                  <DockMini kind={v} />
                  <span>{v}</span>
                </div>
              ))}
            </div>
          </div>
        </div>

        <div style={{ fontSize: 11.5, fontFamily: SHIP_MONO, color: 'var(--muted-foreground)', display: 'flex', gap: 8, alignItems: 'center' }}>
          <span style={{ opacity: 0.6 }}>tip</span>
          <span>You can change all of this later, in Studio.</span>
        </div>
      </div>
    </WizCard>
  );
}

function NavMini({ kind }) {
  return (
    <svg width="56" height="36" viewBox="0 0 56 36">
      <rect x="0.5" y="0.5" width="55" height="35" rx="3" fill="none" stroke="currentColor" strokeOpacity="0.25" />
      {kind === 'rail' && <rect x="2" y="2" width="9" height="32" rx="1.5" fill="currentColor" opacity="0.25" />}
      {kind === 'top' && <rect x="2" y="2" width="52" height="7" rx="1.5" fill="currentColor" opacity="0.25" />}
      <g opacity="0.3">
        <rect x={kind === 'rail' ? 15 : 4} y={kind === 'top' ? 13 : 6} width="38" height="2" rx="1" fill="currentColor" />
        <rect x={kind === 'rail' ? 15 : 4} y={kind === 'top' ? 18 : 11} width="22" height="2" rx="1" fill="currentColor" />
      </g>
    </svg>
  );
}
function DockMini({ kind }) {
  const dotPos = {
    left: [4, 10, 16],
    centered: [22, 28, 34],
    right: [40, 46, 52],
    off: [],
  }[kind];
  return (
    <svg width="56" height="36" viewBox="0 0 56 36">
      <rect x="0.5" y="0.5" width="55" height="35" rx="3" fill="none" stroke="currentColor" strokeOpacity="0.25" />
      <rect x="4" y="4" width="48" height="22" rx="2" fill="currentColor" opacity="0.08" />
      {dotPos.map(x => (
        <rect key={x} x={x} y="29" width="4" height="4" rx="1" fill="currentColor" opacity="0.5" />
      ))}
    </svg>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 4: Step 3 — registered, here's where to find it
// ──────────────────────────────────────────────────────────────
function OnbStep3() {
  const accent = ACCENT_SWATCHES[2];
  return (
    <WizCard
      step={3} total={3} accent={accent}
      title="Brand Voice is installed."
      footerSecondary={<GhostBtn>Done</GhostBtn>}
      footerPrimary={<PrimaryBtn>Launch Brand Voice →</PrimaryBtn>}
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
        {/* Hero */}
        <div style={{
          display: 'flex', alignItems: 'center', gap: 16,
          padding: '16px 18px',
          background: `linear-gradient(135deg, color-mix(in oklch, ${accent} 12%, var(--card)) 0%, var(--card) 100%)`,
          border: '1px solid color-mix(in oklch, ' + accent + ' 30%, var(--border))',
          borderRadius: 12,
        }}>
          <SpaceMark size={54} accent={accent} char="◆" />
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: 17, fontWeight: 600, letterSpacing: '-0.01em' }}>Brand Voice</div>
            <div style={{ color: 'var(--muted-foreground)', fontFamily: SHIP_MONO, fontSize: 11.5, marginTop: 2 }}>
              halo · rail · ~/Spaces/brand-voice
            </div>
          </div>
          <Dot tone="ok" />
        </div>

        {/* Registration receipts */}
        <div>
          <MicroLabel style={{ marginBottom: 10 }}>Where you'll find it</MicroLabel>
          <ul style={{ margin: 0, padding: 0, listStyle: 'none', display: 'flex', flexDirection: 'column', gap: 6 }}>
            {[
              ['Applications', 'macOS dock — drag to keep it there'],
              ['Spotlight', '⌘ Space → "Brand Voice"'],
              ['Ship hub', 'Spaces · marked as installed'],
              ['Shell PATH', 'ship open brand-voice'],
            ].map(([k, v]) => (
              <li key={k} style={{
                display: 'grid', gridTemplateColumns: '24px 130px 1fr', gap: 10, alignItems: 'center',
                padding: '8px 10px', borderRadius: 8,
                background: 'color-mix(in oklch, var(--card) 70%, transparent)',
                border: '1px solid color-mix(in oklch, var(--border) 70%, transparent)',
                fontSize: 12.5,
              }}>
                <span style={{ color: 'var(--success, oklch(0.55 0.13 145))', display: 'grid', placeItems: 'center' }}>
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M4 12l5 5L20 6" /></svg>
                </span>
                <span style={{ fontWeight: 600 }}>{k}</span>
                <span style={{ color: 'var(--muted-foreground)', fontFamily: SHIP_MONO, fontSize: 11 }}>{v}</span>
              </li>
            ))}
          </ul>
        </div>

        <div style={{
          padding: '10px 12px',
          borderRadius: 8,
          background: 'color-mix(in oklch, var(--foreground) 4%, var(--card))',
          fontSize: 11.5, fontFamily: SHIP_MONO, color: 'var(--muted-foreground)',
          display: 'flex', alignItems: 'flex-start', gap: 8,
          lineHeight: 1.5,
        }}>
          <span style={{ opacity: 0.6, marginTop: 1 }}>note</span>
          <span>Uninstall any space from <b style={{ color: 'var(--foreground)' }}>Hub → Spaces → ⋯</b>. Ship removes the host-OS entries it created.</span>
        </div>
      </div>
    </WizCard>
  );
}

Object.assign(window, { OnbWelcome, OnbStep1, OnbStep2, OnbStep3, WIZ_W, WIZ_H });
