// scene-windows.jsx — two running space WINDOWS, side-by-side, with
// distinct themes/chromes. Conveys "each space is its own OS app".

const SPACE_W = 760;
const SPACE_H = 540;

// ──────────────────────────────────────────────────────────────
// SPACE A: Ship · dev project (ship-dark, widget desktop)
// ──────────────────────────────────────────────────────────────
function SpaceShip() {
  const dark = '#16140f';
  const fg = '#e8e1cf';
  const muted = '#7c7565';
  const card = '#1f1c16';
  const border = '#332d22';
  const accent = '#e0a16b';
  const accent2 = '#b08fd9';
  return (
    <div style={{
      borderRadius: 12,
      overflow: 'hidden',
      border: '1px solid rgba(0,0,0,0.35)',
      background: dark,
      color: fg,
      boxShadow: '0 32px 80px -28px rgba(0,0,0,0.55), 0 4px 12px -4px rgba(0,0,0,0.2)',
      display: 'flex', flexDirection: 'column',
      width: SPACE_W, height: SPACE_H,
      fontFamily: SHIP_FONT,
    }}>
      {/* OS title bar */}
      <div style={{
        height: 36, flexShrink: 0,
        display: 'grid', gridTemplateColumns: '1fr auto 1fr',
        alignItems: 'center',
        padding: '0 12px',
        background: 'rgba(0,0,0,0.35)',
        borderBottom: '1px solid rgba(255,255,255,0.05)',
      }}>
        <div style={{ display: 'flex', gap: 7 }}>
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#ff5f57' }} />
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#febc2e' }} />
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#28c840' }} />
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, justifySelf: 'center' }}>
          <SpaceMark size={16} accent={accent} char="S" />
          <span style={{ fontSize: 12, fontWeight: 600 }}>Ship</span>
          <span style={{ fontSize: 11, color: muted, fontFamily: SHIP_MONO }}>~/code/ship</span>
        </div>
        <div style={{ justifySelf: 'end', fontFamily: SHIP_MONO, fontSize: 11, color: muted, display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ display: 'inline-flex', alignItems: 'center', gap: 5 }}>
            <span style={{ width: 6, height: 6, borderRadius: '50%', background: 'oklch(0.65 0.15 145)' }} />
            3 agents
          </span>
        </div>
      </div>

      {/* Body: thin rail + grid */}
      <div style={{ flex: 1, display: 'flex', minHeight: 0 }}>
        {/* Rail */}
        <div style={{
          width: 44, flexShrink: 0,
          borderRight: '1px solid ' + border,
          background: 'rgba(0,0,0,0.18)',
          display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '10px 0',
          gap: 4,
        }}>
          {['bot','sparkles','workflow','library','scroll'].map((g, i) => (
            <div key={g} style={{
              width: 28, height: 28, borderRadius: 6,
              background: i === 0 ? 'color-mix(in oklch, ' + accent + ' 18%, transparent)' : 'transparent',
              color: i === 0 ? accent : muted,
              display: 'grid', placeItems: 'center',
            }}>
              <RailGlyph kind={g} />
            </div>
          ))}
          <div style={{ height: 1, width: 24, background: border, margin: '6px 0' }} />
          {['frame','grid','palette'].map(g => (
            <div key={g} style={{ width: 28, height: 28, borderRadius: 6, color: muted, display: 'grid', placeItems: 'center' }}>
              <RailGlyph kind={g} />
            </div>
          ))}
          <div style={{ flex: 1 }} />
          <div style={{ width: 28, height: 28, borderRadius: 6, color: muted, display: 'grid', placeItems: 'center' }}>
            <RailGlyph kind="gear" />
          </div>
        </div>

        {/* Content: widget grid */}
        <main style={{
          flex: 1, padding: '14px 16px',
          backgroundImage: `radial-gradient(circle at 1px 1px, ${border} 1px, transparent 0)`,
          backgroundSize: '18px 18px',
          display: 'grid',
          gridTemplateColumns: 'repeat(4, 1fr)',
          gridTemplateRows: 'auto 1fr 1fr',
          gap: 10,
          overflow: 'hidden',
        }}>
          {/* Greet, spans 3 */}
          <div style={{
            gridColumn: '1 / span 3',
            background: card, border: '1px solid ' + border, borderRadius: 10,
            padding: '14px 16px', position: 'relative', overflow: 'hidden',
          }}>
            <span style={{ position: 'absolute', top: 0, left: 0, width: 60, height: 2, background: accent2 }} />
            <div style={{ fontSize: 17, fontWeight: 600, letterSpacing: '-0.015em', textWrap: 'balance' }}>
              Good evening. <i style={{ color: accent, fontStyle: 'normal' }}>Three agents</i> are doing work.
            </div>
            <div style={{ color: muted, fontSize: 12, marginTop: 6, fontFamily: SHIP_MONO }}>
              14 skills · 247 events · 1 awaiting review
            </div>
          </div>

          {/* Quick */}
          <div style={{ background: card, border: '1px solid ' + border, borderRadius: 10, padding: 12, position: 'relative', overflow: 'hidden' }}>
            <span style={{ position: 'absolute', top: 0, left: 0, width: 48, height: 2, background: accent }} />
            <MicroLabel color={muted} style={{ marginBottom: 8 }}>Quick</MicroLabel>
            {['New agent', 'Run pipe', 'Find skill'].map((q, i) => (
              <div key={q} style={{ fontSize: 11.5, padding: '4px 0', color: i === 0 ? fg : muted, fontFamily: SHIP_MONO }}>
                + {q}
              </div>
            ))}
          </div>

          {/* Active runs, spans 2 */}
          <div style={{
            gridColumn: '1 / span 2',
            background: card, border: '1px solid ' + border, borderRadius: 10,
            padding: 12, overflow: 'hidden', position: 'relative',
          }}>
            <span style={{ position: 'absolute', top: 0, left: 0, width: 48, height: 2, background: accent }} />
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
              <MicroLabel color={muted}>Active runs</MicroLabel>
              <span style={{ fontFamily: SHIP_MONO, fontSize: 10, color: muted, marginLeft: 'auto' }}>3 of 14</span>
            </div>
            {[
              ['Rust Lane','cargo check','2m','live'],
              ['Brand Voice','draft tweet','14s','live'],
              ['Research','summarise pdf','47s','live'],
              ['Doc Lane','generate skill','4m','ok'],
            ].map(([n,o,t,tone]) => (
              <div key={n} style={{ display: 'grid', gridTemplateColumns: 'auto 1fr auto', gap: 8, alignItems: 'center', padding: '3px 0', fontSize: 11.5 }}>
                <Dot tone={tone} size={6} />
                <span><b style={{ fontWeight: 600 }}>{n}</b> <span style={{ color: muted, fontFamily: SHIP_MONO, fontSize: 10.5 }}>{o}</span></span>
                <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: muted }}>{t}</span>
              </div>
            ))}
          </div>

          {/* Pipeline DAG */}
          <div style={{ background: card, border: '1px solid ' + border, borderRadius: 10, padding: 12, position: 'relative', overflow: 'hidden' }}>
            <span style={{ position: 'absolute', top: 0, left: 0, width: 48, height: 2, background: accent2 }} />
            <MicroLabel color={muted} style={{ marginBottom: 10 }}>overnight</MicroLabel>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 4, position: 'relative' }}>
              <span style={{ position: 'absolute', top: '50%', left: '5%', right: '5%', height: 1, background: border, zIndex: 0 }} />
              {['fetch','parse','embed','idx'].map((n, i) => (
                <div key={n} style={{
                  aspectRatio: '1',
                  border: '1px solid ' + (i === 2 ? accent2 : i < 2 ? 'oklch(0.55 0.13 145)' : border),
                  background: i === 2 ? 'color-mix(in oklch, ' + accent2 + ' 22%, transparent)' :
                              i < 2 ? 'color-mix(in oklch, oklch(0.55 0.13 145) 18%, transparent)' : dark,
                  borderRadius: 4,
                  display: 'grid', placeItems: 'center',
                  fontFamily: SHIP_MONO, fontSize: 9, fontWeight: 600,
                  color: i === 2 ? accent2 : i < 2 ? 'oklch(0.6 0.13 145)' : muted,
                  zIndex: 1,
                }}>{n}</div>
              ))}
            </div>
          </div>

          {/* Events sparkline */}
          <div style={{ background: card, border: '1px solid ' + border, borderRadius: 10, padding: 12, position: 'relative', overflow: 'hidden' }}>
            <span style={{ position: 'absolute', top: 0, left: 0, width: 48, height: 2, background: accent }} />
            <MicroLabel color={muted}>Events / hr</MicroLabel>
            <div style={{ display: 'flex', alignItems: 'baseline', gap: 6, marginTop: 4 }}>
              <div style={{ fontSize: 22, fontWeight: 700, letterSpacing: '-0.02em', fontVariantNumeric: 'tabular-nums' }}>247</div>
              <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'oklch(0.65 0.13 145)' }}>↑38%</div>
            </div>
            <div style={{ display: 'flex', alignItems: 'flex-end', gap: 2, height: 28, marginTop: 6 }}>
              {[3,7,5,9,6,12,8,11,14,10,16,13,18,15,19,22,17].map((n, i) => (
                <div key={i} style={{ flex: 1, height: (n / 22 * 100) + '%', background: 'color-mix(in oklch, ' + accent + ' 70%, transparent)', borderRadius: 1 }} />
              ))}
            </div>
          </div>

          {/* Calendar */}
          <div style={{ background: card, border: '1px solid ' + border, borderRadius: 10, padding: 12, position: 'relative', overflow: 'hidden' }}>
            <span style={{ position: 'absolute', top: 0, left: 0, width: 48, height: 2, background: 'oklch(0.6 0.2 25)' }} />
            <div style={{ fontFamily: SHIP_MONO, fontSize: 9, letterSpacing: '0.14em', color: 'oklch(0.6 0.2 25)', fontWeight: 600 }}>MAY</div>
            <div style={{ fontSize: 32, fontWeight: 700, letterSpacing: '-0.025em', lineHeight: 1, marginTop: 4, fontVariantNumeric: 'tabular-nums' }}>26</div>
            <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: muted, marginTop: 2 }}>Tuesday</div>
          </div>
        </main>
      </div>

      {/* Bottom dock */}
      <div style={{
        height: 50, flexShrink: 0,
        display: 'flex', justifyContent: 'center', alignItems: 'center',
        padding: '0 12px',
        borderTop: '1px solid ' + border,
      }}>
        <div style={{
          display: 'flex', gap: 4, padding: '4px 8px',
          borderRadius: 14, background: 'rgba(255,255,255,0.04)',
          border: '1px solid ' + border,
          backdropFilter: 'blur(10px)',
        }}>
          {[
            ['bot', accent, true],
            ['sparkles', accent2, false],
            ['workflow', accent, false],
            ['library', accent2, false],
            ['scroll', accent, false],
          ].map(([g, c, active], i) => (
            <div key={i} style={{
              width: 30, height: 30, borderRadius: 7,
              background: 'color-mix(in oklch, ' + c + ' 12%, transparent)',
              color: c, display: 'grid', placeItems: 'center',
              position: 'relative',
            }}>
              <RailGlyph kind={g} />
              {active ? <span style={{ position: 'absolute', bottom: -5, width: 3, height: 3, borderRadius: '50%', background: fg }} /> : null}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────
// SPACE B: Paperback · publishing app (paper theme, top nav, editor view)
// ──────────────────────────────────────────────────────────────
function SpacePaper() {
  const bg = '#f4ece0';
  const fg = '#2d1f12';
  const muted = '#7a6a55';
  const card = '#fbf6ec';
  const border = '#d6c8b0';
  const accent = '#a14b30';
  return (
    <div style={{
      borderRadius: 12,
      overflow: 'hidden',
      border: '1px solid rgba(0,0,0,0.18)',
      background: bg,
      color: fg,
      boxShadow: '0 32px 80px -28px rgba(0,0,0,0.35), 0 4px 12px -4px rgba(0,0,0,0.1)',
      display: 'flex', flexDirection: 'column',
      width: SPACE_W, height: SPACE_H,
      fontFamily: 'Georgia, "Iowan Old Style", serif',
    }}>
      {/* OS title bar */}
      <div style={{
        height: 36, flexShrink: 0,
        display: 'grid', gridTemplateColumns: '1fr auto 1fr',
        alignItems: 'center',
        padding: '0 12px',
        background: 'rgba(255,255,255,0.5)',
        borderBottom: '1px solid ' + border,
      }}>
        <div style={{ display: 'flex', gap: 7 }}>
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#ff5f57' }} />
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#febc2e' }} />
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#28c840' }} />
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, justifySelf: 'center' }}>
          <SpaceMark size={16} accent={accent} char="P" />
          <span style={{ fontSize: 12, fontWeight: 600, fontFamily: SHIP_FONT }}>Paperback</span>
          <span style={{ fontSize: 11, color: muted, fontFamily: SHIP_MONO }}>~/Writing/paperback</span>
        </div>
        <div style={{ justifySelf: 'end', fontFamily: SHIP_MONO, fontSize: 11, color: muted }}>
          <span style={{ display: 'inline-flex', alignItems: 'center', gap: 5 }}>
            <Dot tone="ok" size={6} /> draft saved
          </span>
        </div>
      </div>

      {/* Top nav (no rail) */}
      <div style={{
        height: 40, flexShrink: 0,
        display: 'flex', alignItems: 'center', gap: 4,
        padding: '0 20px',
        background: card,
        borderBottom: '1px solid ' + border,
        fontFamily: SHIP_FONT, fontSize: 12.5,
      }}>
        <div style={{
          display: 'flex', alignItems: 'center', gap: 14,
          color: fg,
        }}>
          {[
            ['Posts', false],
            ['Drafts', true],
            ['Schedule', false],
            ['Series', false],
            ['Analytics', false],
          ].map(([l, active]) => (
            <div key={l} style={{
              position: 'relative',
              padding: '12px 0',
              color: active ? accent : muted,
              fontWeight: active ? 600 : 500,
              cursor: 'pointer',
            }}>
              {l}
              {active ? <span style={{
                position: 'absolute', bottom: -1, left: 0, right: 0, height: 2,
                background: accent, borderRadius: 1,
              }} /> : null}
            </div>
          ))}
        </div>
        <div style={{ flex: 1 }} />
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <button style={{
            padding: '5px 10px', borderRadius: 6,
            border: '1px solid ' + border, background: bg, color: fg,
            fontFamily: SHIP_FONT, fontSize: 11.5, cursor: 'pointer',
          }}>Preview</button>
          <button style={{
            padding: '5px 12px', borderRadius: 6,
            border: '1px solid ' + accent, background: accent, color: card,
            fontFamily: SHIP_FONT, fontSize: 11.5, fontWeight: 600, cursor: 'pointer',
          }}>Publish →</button>
        </div>
      </div>

      {/* Body: editor + metadata column */}
      <div style={{ flex: 1, display: 'flex', minHeight: 0 }}>
        {/* Editor */}
        <main style={{
          flex: 1, padding: '20px 32px',
          overflow: 'hidden',
          display: 'flex', flexDirection: 'column', gap: 12,
        }}>
          <div style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: muted, letterSpacing: '0.12em', textTransform: 'uppercase' }}>
            Draft · last edit 4m ago
          </div>
          <div style={{
            fontFamily: 'Georgia, serif', fontSize: 26, fontWeight: 600, letterSpacing: '-0.01em',
            lineHeight: 1.15,
          }}>
            On the slow death of the workspace tab
          </div>
          <div style={{ fontFamily: 'Georgia, serif', fontSize: 14, lineHeight: 1.6, color: 'color-mix(in oklch, ' + fg + ' 80%, transparent)' }}>
            <p style={{ margin: 0, marginBottom: 10 }}>
              For most of the last decade, the workspace has been a tab — one of many — inside a
              browser-shaped vessel called The App. The tab pretended to be a window, the window
              pretended to be a desktop, the desktop pretended not to exist.
            </p>
            <p style={{ margin: 0, marginBottom: 10 }}>
              That pretending is finally collapsing. <span style={{ background: 'color-mix(in oklch, ' + accent + ' 15%, transparent)', padding: '0 3px' }}>What the OS</span>
              already does well — focus, switching, identity — wins over what The App
              tried to recreate.
            </p>
            <p style={{ margin: 0, color: muted, fontSize: 12.5 }}>
              <span style={{ display: 'inline-block', width: 1.5, height: 14, background: accent, marginRight: 2, verticalAlign: 'text-bottom', animation: 'sb-pulse 1.2s linear infinite' }} />
            </p>
          </div>
        </main>

        {/* Right metadata column */}
        <aside style={{
          width: 200, flexShrink: 0,
          padding: '20px 18px',
          borderLeft: '1px solid ' + border,
          background: 'color-mix(in oklch, ' + card + ' 50%, ' + bg + ')',
          display: 'flex', flexDirection: 'column', gap: 14,
          fontFamily: SHIP_FONT, fontSize: 12,
        }}>
          {[
            ['slug', 'death-of-the-workspace-tab'],
            ['status', 'draft'],
            ['scheduled', '—'],
            ['series', 'workbench'],
          ].map(([k,v]) => (
            <div key={k}>
              <div style={{ fontFamily: SHIP_MONO, fontSize: 10, letterSpacing: '0.12em', textTransform: 'uppercase', color: muted, marginBottom: 3 }}>{k}</div>
              <div style={{ color: fg, wordBreak: 'break-all' }}>{v}</div>
            </div>
          ))}
          <div>
            <div style={{ fontFamily: SHIP_MONO, fontSize: 10, letterSpacing: '0.12em', textTransform: 'uppercase', color: muted, marginBottom: 6 }}>tags</div>
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
              {['workspaces','os','focus','rant'].map(t => (
                <span key={t} style={{
                  padding: '2px 7px', borderRadius: 999,
                  border: '1px solid ' + border, background: card,
                  fontSize: 10.5, color: muted, fontFamily: SHIP_MONO,
                }}>{t}</span>
              ))}
            </div>
          </div>
          <div style={{ flex: 1 }} />
          <div style={{
            padding: 8, borderRadius: 6,
            background: 'color-mix(in oklch, ' + accent + ' 8%, ' + card + ')',
            border: '1px solid color-mix(in oklch, ' + accent + ' 25%, ' + border + ')',
            fontSize: 11, color: muted,
            display: 'flex', gap: 6, alignItems: 'flex-start',
          }}>
            <Dot tone="live" size={6} />
            <span><b style={{ color: fg, fontWeight: 600 }}>Editor</b> agent reading along · 1m</span>
          </div>
        </aside>
      </div>
    </div>
  );
}

// Rail glyphs shared
function RailGlyph({ kind }) {
  const p = { width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 1.8, strokeLinecap: 'round', strokeLinejoin: 'round' };
  switch (kind) {
    case 'bot': return <svg {...p}><rect x="3" y="9" width="18" height="11" rx="2" /><path d="M12 5v4M9 5h6M8 14h.01M16 14h.01" /></svg>;
    case 'sparkles': return <svg {...p}><path d="M12 3l1.6 4.4L18 9l-4.4 1.6L12 15l-1.6-4.4L6 9l4.4-1.6L12 3zM5 17l.8 2.2L8 20l-2.2.8L5 23l-.8-2.2L2 20l2.2-.8L5 17z" /></svg>;
    case 'workflow': return <svg {...p}><rect x="3" y="3" width="6" height="6" /><rect x="15" y="15" width="6" height="6" /><path d="M9 6h6a3 3 0 0 1 3 3v6" /></svg>;
    case 'library': return <svg {...p}><path d="M4 4v16M20 4v16M4 8h16M4 16h16M9 4v16M14 4v16" /></svg>;
    case 'scroll': return <svg {...p}><path d="M8 21h11a2 2 0 0 0 0-4h-1V5a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2zM8 7h6M8 11h6M8 15h6" /></svg>;
    case 'frame': return <svg {...p}><rect x="3" y="3" width="18" height="18" rx="2" /><path d="M3 9h18M9 3v18" /></svg>;
    case 'grid': return <svg {...p}><rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /></svg>;
    case 'palette': return <svg {...p}><path d="M12 3a9 9 0 1 0 0 18c1.5 0 2.5-1 2.5-2.5S13.5 16 13.5 14.5 14.5 12 16 12h2.5a3 3 0 0 0 3-3 9 9 0 0 0-9.5-6z" /></svg>;
    case 'gear': return <svg {...p}><circle cx="12" cy="12" r="3" /><path d="M19 12a7 7 0 0 0-.1-1.3l2-1.5-2-3.5-2.4.9a7 7 0 0 0-2.3-1.3L13.5 3h-3l-.7 2.3a7 7 0 0 0-2.3 1.3l-2.4-.9-2 3.5 2 1.5A7 7 0 0 0 5 12c0 .5 0 .9.1 1.3l-2 1.5 2 3.5 2.4-.9a7 7 0 0 0 2.3 1.3l.7 2.3h3l.7-2.3a7 7 0 0 0 2.3-1.3l2.4.9 2-3.5-2-1.5c.1-.4.1-.8.1-1.3z" /></svg>;
    default: return null;
  }
}

Object.assign(window, { SpaceShip, SpacePaper, RailGlyph, SPACE_W, SPACE_H });
