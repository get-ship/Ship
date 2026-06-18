// shared-ui.jsx — small primitives reused across all scenes
// Window chrome, app/space icons, section labels, dotted/grid backdrop wrap.

const SHIP_FONT = 'var(--font-body, ui-sans-serif, system-ui, sans-serif)';
const SHIP_MONO = 'var(--font-mono, ui-monospace, "JetBrains Mono", monospace)';

// ──────────────────────────────────────────────────────────────
// MacOS window chrome — tighter than the starter, used everywhere so
// the user reads each artboard as "an OS window".
// ──────────────────────────────────────────────────────────────
function WinFrame({ title, subtitle, accent = 'var(--primary)', icon, children, style = {}, dark = false }) {
  return (
    <div style={{
      borderRadius: 12,
      overflow: 'hidden',
      border: '1px solid var(--border)',
      background: 'var(--background)',
      color: 'var(--foreground)',
      boxShadow: '0 24px 60px -24px rgba(0,0,0,0.32), 0 2px 6px -2px rgba(0,0,0,0.12)',
      display: 'flex', flexDirection: 'column',
      fontFamily: SHIP_FONT,
      ...style,
    }}>
      {/* title bar */}
      <div style={{
        height: 38, flexShrink: 0,
        display: 'grid',
        gridTemplateColumns: '1fr auto 1fr',
        alignItems: 'center',
        padding: '0 12px',
        background: dark ? 'color-mix(in oklch, var(--background) 88%, #000)'
                         : 'color-mix(in oklch, var(--background) 70%, var(--card))',
        borderBottom: '1px solid var(--border)',
        backdropFilter: 'blur(12px) saturate(1.2)',
        WebkitBackdropFilter: 'blur(12px) saturate(1.2)',
      }}>
        <div style={{ display: 'flex', gap: 7, alignItems: 'center' }}>
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#ff5f57' }} />
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#febc2e' }} />
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#28c840' }} />
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, justifySelf: 'center' }}>
          {icon ? <SpaceMark size={16} accent={accent} char={icon} /> : null}
          <span style={{ fontSize: 12, fontWeight: 600, letterSpacing: '-0.005em' }}>{title}</span>
          {subtitle ? (
            <span style={{
              fontSize: 11, color: 'var(--muted-foreground)', fontFamily: SHIP_MONO,
              marginLeft: 4,
            }}>{subtitle}</span>
          ) : null}
        </div>
        <div />
      </div>
      <div style={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column' }}>
        {children}
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────
// Space mark — the icon for a space. Tiny rounded tile with a single
// glyph or letter, painted in the space's accent. Used in: hub, dock,
// window titlebar, manifest preview, OS launcher mockup.
// ──────────────────────────────────────────────────────────────
function SpaceMark({ size = 44, accent = 'var(--primary)', char = 'S', shape = 'squircle', dim = false }) {
  const radius = shape === 'circle' ? '50%' : shape === 'squircle' ? size * 0.32 : size * 0.22;
  return (
    <div style={{
      width: size, height: size,
      borderRadius: radius,
      background: dim ? 'color-mix(in oklch, var(--card) 92%, transparent)'
                     : `linear-gradient(155deg, color-mix(in oklch, ${accent} 92%, white) 0%, ${accent} 60%, color-mix(in oklch, ${accent} 80%, black) 100%)`,
      color: dim ? 'var(--muted-foreground)' : 'white',
      display: 'grid', placeItems: 'center',
      fontFamily: SHIP_MONO,
      fontSize: size * 0.42,
      fontWeight: 700,
      letterSpacing: '-0.02em',
      flexShrink: 0,
      position: 'relative',
      overflow: 'hidden',
      boxShadow: dim ? 'inset 0 0 0 1px var(--border)'
                    : `inset 0 1px 0 color-mix(in oklch, white 26%, transparent),
                       inset 0 -1px 0 color-mix(in oklch, black 14%, transparent),
                       0 ${size * 0.05}px ${size * 0.2}px color-mix(in oklch, ${accent} 35%, transparent)`,
    }}>
      <span style={{ position: 'relative', zIndex: 1 }}>{char}</span>
      {!dim && (
        <span style={{
          position: 'absolute', inset: 0,
          background: 'linear-gradient(160deg, color-mix(in oklch, white 22%, transparent) 0%, transparent 55%)',
          pointerEvents: 'none',
        }} />
      )}
    </div>
  );
}

// Sectiony label (CAPS, mono, muted)
function MicroLabel({ children, color, style = {} }) {
  return (
    <div style={{
      fontFamily: SHIP_MONO,
      fontSize: 10,
      fontWeight: 600,
      letterSpacing: '0.14em',
      textTransform: 'uppercase',
      color: color || 'var(--muted-foreground)',
      ...style,
    }}>{children}</div>
  );
}

// Wee dot — status indicator. tone: 'live' | 'idle' | 'ok' | 'err'
function Dot({ tone = 'idle', size = 8 }) {
  const colors = {
    live: 'var(--info, oklch(0.65 0.15 240))',
    idle: 'var(--muted-foreground)',
    ok: 'var(--success, oklch(0.55 0.13 145))',
    err: 'var(--destructive, oklch(0.55 0.2 25))',
    accent: 'var(--primary)',
  };
  return (
    <span style={{
      width: size, height: size, borderRadius: '50%',
      background: colors[tone] || colors.idle,
      display: 'inline-block',
      flexShrink: 0,
      ...(tone === 'live' ? { animation: 'sb-pulse 1.6s ease-in-out infinite' } : null),
    }} />
  );
}

// Dotted-grid backdrop region — used inside windows that should feel like
// a "canvas" the user can pan around in.
function DotBackdrop({ children, style = {} }) {
  return (
    <div style={{
      position: 'relative',
      flex: 1,
      minHeight: 0,
      backgroundImage: 'radial-gradient(circle at 1px 1px, color-mix(in oklch, var(--foreground) 14%, transparent) 1px, transparent 0)',
      backgroundSize: '18px 18px',
      ...style,
    }}>
      {children}
    </div>
  );
}

// Tiny kbd chip
function Kbd({ children }) {
  return (
    <span style={{
      fontFamily: SHIP_MONO, fontSize: 10,
      padding: '2px 5px',
      border: '1px solid var(--border)',
      borderRadius: 4,
      background: 'var(--card)',
      color: 'var(--muted-foreground)',
    }}>{children}</span>
  );
}

// Keyframes once — pulse anim used by Dot tone="live"
if (typeof document !== 'undefined' && !document.getElementById('shared-ui-kf')) {
  const s = document.createElement('style');
  s.id = 'shared-ui-kf';
  s.textContent = '@keyframes sb-pulse { 0%,100% { opacity:1 } 50% { opacity:.35 } }';
  document.head.appendChild(s);
}

Object.assign(window, { WinFrame, SpaceMark, MicroLabel, Dot, DotBackdrop, Kbd, SHIP_FONT, SHIP_MONO });
