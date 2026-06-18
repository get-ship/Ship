// scene-paired.jsx — The two-window editing model.
// Hub holds the IDE (source, structure, controls). The Space is the live
// runtime — what you ship to your future self. Edits in the hub stream
// into the running space.

const PAIRED_W = 1900;
const PAIRED_H = 760;
const PAIR_HUB_W = 1040;
const PAIR_HUB_H = 720;
const PAIR_SPACE_W = 720;
const PAIR_SPACE_H = 540;

// ──────────────────────────────────────────────────────────────
// Layout primitive: two macOS windows + sync ribbon between them
// ──────────────────────────────────────────────────────────────
function TwoWindow({ children, syncLabel, syncDetail, accent = 'var(--primary)' }) {
  return (
    <div style={{
      width: PAIRED_W, height: PAIRED_H,
      position: 'relative',
      display: 'flex', alignItems: 'center',
      padding: '20px 28px',
      gap: 0,
    }}>
      {children}
      {/* Sync indicator floats above the gap */}
      <SyncPill label={syncLabel} detail={syncDetail} accent={accent} />
    </div>
  );
}

function SyncPill({ label, detail, accent }) {
  return (
    <div style={{
      position: 'absolute',
      left: 28 + PAIR_HUB_W - 28,           // overlap into the gap
      top: '50%',
      transform: 'translate(0, -50%)',
      zIndex: 5,
      width: 92,
      display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 6,
      pointerEvents: 'none',
    }}>
      {/* dashed wire */}
      <div style={{
        position: 'absolute', top: '50%', left: -8, right: -8, height: 0,
        borderTop: '1.5px dashed ' + accent,
        opacity: 0.6,
      }} />
      {/* directional arrow */}
      <div style={{
        position: 'absolute', top: '50%', right: -16, transform: 'translateY(-50%)',
        width: 0, height: 0,
        borderTop: '6px solid transparent',
        borderBottom: '6px solid transparent',
        borderLeft: '8px solid ' + accent,
        opacity: 0.8,
      }} />
      {/* pill */}
      <div style={{
        background: 'var(--background)',
        border: '1.5px solid ' + accent,
        borderRadius: 999,
        padding: '6px 12px',
        fontFamily: SHIP_MONO, fontSize: 11, fontWeight: 600,
        color: accent,
        display: 'flex', alignItems: 'center', gap: 6,
        boxShadow: '0 4px 14px -2px color-mix(in oklch, ' + accent + ' 25%, transparent)',
        position: 'relative', zIndex: 1,
      }}>
        <Dot tone="live" size={6} />
        {label}
      </div>
      {detail ? (
        <div style={{
          fontFamily: SHIP_MONO, fontSize: 10,
          color: 'var(--muted-foreground)',
          background: 'color-mix(in oklch, var(--background) 90%, transparent)',
          padding: '1px 6px', borderRadius: 4,
          position: 'relative', zIndex: 1,
          whiteSpace: 'nowrap',
        }}>{detail}</div>
      ) : null}
    </div>
  );
}

// Small generic file tree column reused across IDEs
function FileTree({ items, activeId }) {
  return (
    <div style={{
      width: 200, flexShrink: 0,
      borderRight: '1px solid var(--border)',
      background: 'color-mix(in oklch, var(--background) 96%, var(--card))',
      padding: '12px 0',
      display: 'flex', flexDirection: 'column',
      fontFamily: SHIP_MONO, fontSize: 11.5,
      overflow: 'hidden',
    }}>
      {items.map((it, i) => {
        if (it.divider) return <div key={i} style={{ height: 1, background: 'var(--border)', margin: '8px 12px' }} />;
        if (it.section) return (
          <div key={i} style={{
            padding: '8px 14px 4px',
            fontFamily: SHIP_MONO, fontSize: 9.5,
            color: 'var(--muted-foreground)',
            letterSpacing: '0.14em', textTransform: 'uppercase', fontWeight: 600,
          }}>{it.section}</div>
        );
        const active = it.id === activeId;
        return (
          <div key={i} style={{
            display: 'flex', alignItems: 'center', gap: 8,
            padding: '4px 14px 4px ' + (8 + (it.depth || 0) * 12) + 'px',
            background: active ? 'color-mix(in oklch, var(--primary) 10%, transparent)' : 'transparent',
            color: active ? 'var(--foreground)' : 'var(--muted-foreground)',
            fontWeight: active ? 600 : 400,
            cursor: 'pointer',
            position: 'relative',
          }}>
            {active && (
              <span style={{ position: 'absolute', left: 0, top: 4, bottom: 4, width: 2, background: 'var(--primary)', borderRadius: 1 }} />
            )}
            <FileGlyph kind={it.kind} />
            <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{it.name}</span>
            {it.badge ? (
              <span style={{
                marginLeft: 'auto',
                fontFamily: SHIP_MONO, fontSize: 9,
                color: it.badgeTone === 'mod' ? 'oklch(0.55 0.18 60)' : 'var(--muted-foreground)',
              }}>{it.badge}</span>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}

function FileGlyph({ kind }) {
  const p = { width: 12, height: 12, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 1.8, strokeLinecap: 'round', strokeLinejoin: 'round', opacity: 0.7 };
  switch (kind) {
    case 'folder': return <svg {...p}><path d="M3 6a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6z" /></svg>;
    case 'doc': return <svg {...p}><path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9l-6-6z" /><path d="M14 3v6h6" /></svg>;
    case 'md': return <svg {...p}><rect x="3" y="5" width="18" height="14" rx="1.5" /><path d="M7 14V9l3 3 3-3v5M17 9v5M15 12l2 2 2-2" /></svg>;
    case 'json': return <svg {...p}><path d="M8 4c-2 0-3 1-3 3v3c0 1-.5 2-2 2 1.5 0 2 1 2 2v3c0 2 1 3 3 3M16 4c2 0 3 1 3 3v3c0 1 .5 2 2 2-1.5 0-2 1-2 2v3c0 2-1 3-3 3" /></svg>;
    case 'code': return <svg {...p}><path d="M9 8l-4 4 4 4M15 8l4 4-4 4" /></svg>;
    case 'pipe': return <svg {...p}><circle cx="5" cy="6" r="2" /><circle cx="19" cy="6" r="2" /><circle cx="5" cy="18" r="2" /><circle cx="19" cy="18" r="2" /><path d="M7 6h10M5 8v8M19 8v8" /></svg>;
    case 'sui': return <svg {...p}><path d="M4 4h16v16H4z M9 9h6v6h-6z" /></svg>;
    default: return <svg {...p}><circle cx="12" cy="12" r="8" /></svg>;
  }
}

function PairedHubChrome({ title, breadcrumb, children, statusLeft, statusRight }) {
  return (
    <div style={{
      width: PAIR_HUB_W, height: PAIR_HUB_H,
      borderRadius: 12,
      overflow: 'hidden',
      border: '1px solid var(--border)',
      background: 'var(--background)',
      color: 'var(--foreground)',
      boxShadow: '0 32px 80px -28px rgba(0,0,0,0.32), 0 4px 12px -4px rgba(0,0,0,0.12)',
      display: 'flex', flexDirection: 'column',
      fontFamily: SHIP_FONT,
      flexShrink: 0,
    }}>
      {/* OS title bar */}
      <div style={{
        height: 36, flexShrink: 0,
        display: 'grid', gridTemplateColumns: '1fr auto 1fr',
        alignItems: 'center',
        padding: '0 12px',
        background: 'color-mix(in oklch, var(--background) 70%, var(--card))',
        borderBottom: '1px solid var(--border)',
      }}>
        <div style={{ display: 'flex', gap: 7 }}>
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#ff5f57' }} />
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#febc2e' }} />
          <span style={{ width: 12, height: 12, borderRadius: '50%', background: '#28c840' }} />
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, justifySelf: 'center' }}>
          <SpaceMark size={16} accent="var(--primary)" char="S" />
          <span style={{ fontSize: 12, fontWeight: 600 }}>Ship</span>
          <span style={{ fontSize: 11, color: 'var(--muted-foreground)', fontFamily: SHIP_MONO }}>{title}</span>
        </div>
        <div style={{ justifySelf: 'end', fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)', display: 'flex', alignItems: 'center', gap: 8 }}>
          {statusRight}
        </div>
      </div>

      {/* breadcrumb */}
      {breadcrumb ? (
        <div style={{
          height: 32, flexShrink: 0,
          padding: '0 14px',
          borderBottom: '1px solid var(--border)',
          background: 'color-mix(in oklch, var(--background) 92%, var(--card))',
          display: 'flex', alignItems: 'center', gap: 8,
          fontSize: 11.5,
        }}>
          {breadcrumb}
          <span style={{ flex: 1 }} />
          {statusLeft}
        </div>
      ) : null}

      {children}
    </div>
  );
}

function PairedSpaceChrome({ title, sub, icon, accent, dark, fg, bg, card, border, muted, children, fontFamily, status, themeName }) {
  return (
    <div style={{
      width: PAIR_SPACE_W, height: PAIR_SPACE_H,
      borderRadius: 12,
      overflow: 'hidden',
      border: '1px solid ' + (dark ? 'rgba(0,0,0,0.35)' : border),
      background: bg,
      color: fg,
      boxShadow: '0 32px 80px -28px rgba(0,0,0,0.45), 0 4px 12px -4px rgba(0,0,0,0.18)',
      display: 'flex', flexDirection: 'column',
      flexShrink: 0,
      fontFamily: fontFamily || SHIP_FONT,
      position: 'relative',
    }}>
      <div style={{
        height: 32, flexShrink: 0,
        display: 'grid', gridTemplateColumns: '1fr auto 1fr',
        alignItems: 'center',
        padding: '0 10px',
        background: dark ? 'rgba(0,0,0,0.35)' : 'color-mix(in oklch, ' + bg + ' 85%, white)',
        borderBottom: '1px solid ' + (dark ? 'rgba(255,255,255,0.05)' : border),
      }}>
        <div style={{ display: 'flex', gap: 6 }}>
          <span style={{ width: 11, height: 11, borderRadius: '50%', background: '#ff5f57' }} />
          <span style={{ width: 11, height: 11, borderRadius: '50%', background: '#febc2e' }} />
          <span style={{ width: 11, height: 11, borderRadius: '50%', background: '#28c840' }} />
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 7, justifySelf: 'center', fontFamily: SHIP_FONT }}>
          <SpaceMark size={14} accent={accent} char={icon} />
          <span style={{ fontSize: 11.5, fontWeight: 600 }}>{title}</span>
          {sub ? <span style={{ fontSize: 10.5, color: muted, fontFamily: SHIP_MONO }}>{sub}</span> : null}
        </div>
        <div style={{ justifySelf: 'end', fontFamily: SHIP_MONO, fontSize: 10.5, color: muted, display: 'flex', alignItems: 'center', gap: 6 }}>
          {status}
        </div>
      </div>
      {children}
      {themeName ? (
        <div style={{
          position: 'absolute', bottom: 8, right: 10,
          fontFamily: SHIP_MONO, fontSize: 9.5,
          color: muted, opacity: 0.85,
          background: 'color-mix(in oklch, ' + bg + ' 85%, transparent)',
          padding: '2px 6px', borderRadius: 4,
          backdropFilter: 'blur(4px)',
          letterSpacing: '0.06em', textTransform: 'uppercase',
        }}>{themeName}</div>
      ) : null}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 0: Hero / explainer — small diagram
// ──────────────────────────────────────────────────────────────
const HERO_W = 1120;
const HERO_H = 540;
function PairedHero() {
  return (
    <div style={{
      width: HERO_W, height: HERO_H,
      borderRadius: 12,
      background: 'linear-gradient(165deg, color-mix(in oklch, var(--primary) 6%, var(--background)) 0%, var(--background) 60%)',
      border: '1px solid var(--border)',
      padding: 36,
      display: 'flex', flexDirection: 'column', gap: 28,
      fontFamily: SHIP_FONT,
      overflow: 'hidden',
    }}>
      <div>
        <MicroLabel>Interaction model · the dual-window pattern</MicroLabel>
        <div style={{ fontSize: 28, fontWeight: 600, letterSpacing: '-0.02em', marginTop: 8, textWrap: 'balance', maxWidth: 720 }}>
          The hub edits. The space lives.
        </div>
        <div style={{ marginTop: 6, color: 'var(--muted-foreground)', fontSize: 14, lineHeight: 1.55, maxWidth: 640 }}>
          Anything you build inside Ship — a space's chrome, a pipeline, a skill — has two faces.
          You edit the source in the <b style={{ color: 'var(--foreground)' }}>hub</b>; you run the result in a <b style={{ color: 'var(--foreground)' }}>space</b> window beside it.
        </div>
      </div>

      {/* diagram */}
      <div style={{ flex: 1, display: 'grid', gridTemplateColumns: '1.2fr 80px 1fr', gap: 12, alignItems: 'center' }}>
        {/* hub mock */}
        <div style={{
          height: 260,
          background: 'var(--card)', borderRadius: 10,
          border: '1px solid var(--border)',
          boxShadow: '0 12px 30px -10px rgba(0,0,0,0.2)',
          padding: 14,
          display: 'flex', flexDirection: 'column', gap: 8,
          position: 'relative',
          overflow: 'hidden',
        }}>
          <div style={{ display: 'flex', gap: 5 }}>
            <span style={{ width: 8, height: 8, borderRadius: '50%', background: '#ff5f57' }} />
            <span style={{ width: 8, height: 8, borderRadius: '50%', background: '#febc2e' }} />
            <span style={{ width: 8, height: 8, borderRadius: '50%', background: '#28c840' }} />
            <span style={{ marginLeft: 12, fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>Ship · studio</span>
          </div>
          <div style={{
            flex: 1, display: 'grid', gridTemplateColumns: '70px 1fr',
            gap: 8, marginTop: 4,
          }}>
            <div style={{ background: 'color-mix(in oklch, var(--foreground) 4%, transparent)', borderRadius: 6, padding: '6px 6px', display: 'flex', flexDirection: 'column', gap: 4 }}>
              {[1, 2, 3, 4, 5].map(i => <span key={i} style={{ height: 5, background: i === 2 ? 'color-mix(in oklch, var(--primary) 30%, transparent)' : 'color-mix(in oklch, var(--foreground) 8%, transparent)', borderRadius: 1 }} />)}
            </div>
            <div style={{ background: 'color-mix(in oklch, var(--foreground) 3%, transparent)', borderRadius: 6, padding: 8, fontFamily: SHIP_MONO, fontSize: 10, lineHeight: 1.5, color: 'var(--muted-foreground)' }}>
              <div><span style={{ color: 'var(--syntax-keyword, oklch(0.5 0.18 270))' }}>"theme"</span>: <span style={{ color: 'var(--syntax-string, oklch(0.55 0.15 145))' }}>"paper"</span>,</div>
              <div><span style={{ color: 'var(--syntax-keyword, oklch(0.5 0.18 270))' }}>"nav"</span>: <span style={{ color: 'var(--syntax-string, oklch(0.55 0.15 145))' }}>"top"</span>,</div>
              <div><span style={{ color: 'var(--syntax-keyword, oklch(0.5 0.18 270))' }}>"dock"</span>: <span style={{ color: 'var(--syntax-string, oklch(0.55 0.15 145))' }}>"off"</span></div>
            </div>
          </div>
          <div style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)', display: 'flex', gap: 8, alignItems: 'center' }}>
            <span style={{ width: 6, height: 6, borderRadius: '50%', background: 'var(--primary)' }} />
            hub · IDE / source
          </div>
        </div>

        {/* arrow */}
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: 6, fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--primary)' }}>
          <span style={{ fontWeight: 600 }}>edits</span>
          <svg width="60" height="20" viewBox="0 0 60 20" fill="none" stroke="var(--primary)" strokeWidth="1.5">
            <path d="M0 10 L52 10" strokeDasharray="3 3" />
            <path d="M48 5 L56 10 L48 15" fill="var(--primary)" />
          </svg>
          <span style={{ color: 'var(--muted-foreground)' }}>live · &lt;16ms</span>
        </div>

        {/* space mock */}
        <div style={{
          height: 220,
          background: '#f4ece0', borderRadius: 10,
          border: '1px solid #d6c8b0',
          boxShadow: '0 12px 30px -10px rgba(0,0,0,0.25)',
          padding: 12, display: 'flex', flexDirection: 'column', gap: 6,
        }}>
          <div style={{ display: 'flex', gap: 5 }}>
            <span style={{ width: 8, height: 8, borderRadius: '50%', background: '#ff5f57' }} />
            <span style={{ width: 8, height: 8, borderRadius: '50%', background: '#febc2e' }} />
            <span style={{ width: 8, height: 8, borderRadius: '50%', background: '#28c840' }} />
            <span style={{ marginLeft: 12, fontFamily: SHIP_MONO, fontSize: 10.5, color: '#7a6a55' }}>Paperback</span>
          </div>
          <div style={{ display: 'flex', gap: 12, padding: '4px 0', borderBottom: '1px solid #d6c8b0', fontSize: 10, color: '#7a6a55', fontFamily: SHIP_FONT }}>
            <span style={{ color: '#a14b30', fontWeight: 600 }}>Posts</span>
            <span>Drafts</span>
            <span>Schedule</span>
          </div>
          <div style={{ flex: 1, padding: 4, fontFamily: 'Georgia, serif', color: '#2d1f12' }}>
            <div style={{ fontSize: 13, fontWeight: 600 }}>The slow death of the tab</div>
            <div style={{ fontSize: 10, color: '#7a6a55', lineHeight: 1.4, marginTop: 3 }}>For most of the last decade, the workspace has been…</div>
          </div>
          <div style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: '#7a6a55', display: 'flex', gap: 6, alignItems: 'center' }}>
            <span style={{ width: 6, height: 6, borderRadius: '50%', background: '#a14b30' }} />
            space · live runtime
          </div>
        </div>
      </div>

      {/* tagline strip */}
      <div style={{
        display: 'flex', alignItems: 'center', gap: 14,
        padding: '12px 16px', borderRadius: 10,
        background: 'var(--card)', border: '1px solid var(--border)',
        fontSize: 12, color: 'var(--muted-foreground)',
      }}>
        <span style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--primary)', fontWeight: 600, letterSpacing: '0.06em', textTransform: 'uppercase' }}>Applies to</span>
        <span><b style={{ color: 'var(--foreground)' }}>Space chrome</b> · theme + layout</span>
        <span style={{ opacity: 0.4 }}>·</span>
        <span><b style={{ color: 'var(--foreground)' }}>Pipelines</b> · the DAG you're tuning</span>
        <span style={{ opacity: 0.4 }}>·</span>
        <span><b style={{ color: 'var(--foreground)' }}>Agents &amp; skills</b> · prompts, tools, examples</span>
        <span style={{ flex: 1 }} />
        <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5 }}>Cmd-shift-E · open IDE for current space</span>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 1: SPACES — Hub IDE (chrome editor) + Paperback live
// ──────────────────────────────────────────────────────────────
function PairedSpaces() {
  return (
    <TwoWindow
      syncLabel="live · paperback"
      syncDetail="4 edits · 38ms"
      accent="oklch(0.55 0.14 28)"
    >
      {/* HUB — chrome IDE */}
      <PairedHubChrome
        title="studio · paperback / chrome"
        breadcrumb={
          <>
            <span style={{ color: 'var(--muted-foreground)' }}>Studio</span>
            <span style={{ color: 'var(--muted-foreground)' }}>›</span>
            <SpaceMark size={14} accent="oklch(0.55 0.14 28)" char="P" />
            <span style={{ fontWeight: 600 }}>Paperback</span>
            <span style={{ color: 'var(--muted-foreground)' }}>›</span>
            <span style={{ fontFamily: SHIP_MONO, color: 'var(--muted-foreground)' }}>space.jsonc</span>
          </>
        }
        statusLeft={<span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'oklch(0.55 0.18 60)' }}>● 3 unsaved</span>}
        statusRight={<><Dot tone="live" size={6} /><span>linked → window 2</span></>}
      >
        <div style={{ flex: 1, display: 'grid', gridTemplateColumns: '180px 1fr 240px', minHeight: 0 }}>
          <FileTree
            activeId="space"
            items={[
              { section: '~/Writing/paperback' },
              { kind: 'folder', name: 'views/', depth: 0 },
              { kind: 'folder', name: 'skills/', depth: 0 },
              { kind: 'folder', name: 'pipelines/', depth: 0 },
              { kind: 'json', name: 'space.jsonc', depth: 0, id: 'space', badge: '●', badgeTone: 'mod' },
              { kind: 'md', name: 'README.md', depth: 0 },
              { divider: true },
              { section: 'Inherited' },
              { kind: 'sui', name: 'view-chrome', depth: 0 },
              { kind: 'doc', name: 'paper theme', depth: 0 },
            ]}
          />

          {/* editor + controls */}
          <div style={{ display: 'flex', flexDirection: 'column', minHeight: 0, overflow: 'hidden' }}>
            {/* tab strip */}
            <div style={{
              height: 30, flexShrink: 0,
              display: 'flex', alignItems: 'stretch',
              background: 'color-mix(in oklch, var(--background) 95%, var(--card))',
              borderBottom: '1px solid var(--border)',
              fontFamily: SHIP_MONO, fontSize: 11,
            }}>
              <div style={{
                padding: '0 14px', display: 'flex', alignItems: 'center', gap: 6,
                background: 'var(--background)',
                borderRight: '1px solid var(--border)',
                fontWeight: 600,
              }}>
                <FileGlyph kind="json" />
                space.jsonc
                <span style={{ color: 'oklch(0.55 0.18 60)' }}>●</span>
              </div>
              <div style={{
                padding: '0 14px', display: 'flex', alignItems: 'center', gap: 6,
                color: 'var(--muted-foreground)',
                borderRight: '1px solid var(--border)',
              }}>
                <FileGlyph kind="md" /> README.md
              </div>
              <span style={{ flex: 1 }} />
            </div>

            {/* code body */}
            <div style={{
              flex: 1, minHeight: 0, overflow: 'auto',
              padding: '12px 0',
              background: 'var(--background)',
            }}>
              <CodeBlock lines={[
                ['{', ''],
                ['  "id": "paperback",', ''],
                ['  "name": "Paperback",', ''],
                ['  "version": "0.4.1",', ''],
                ['', ''],
                ['  "icon": {', ''],
                ['    "glyph": "P", "accent": "oklch(0.55 0.14 28)"', 'mod'],
                ['  },', ''],
                ['', ''],
                ['  "appearance": {', ''],
                ['    "theme": "paper",', 'mod'],
                ['    "nav": "top",', 'mod'],
                ['    "dock": "off",', 'mod'],
                ['    "density": "comfortable"', ''],
                ['  },', ''],
                ['', ''],
                ['  "home": "views/writing",', ''],
                ['  "os": { "register": ["dock", "spotlight", "cli"] }', ''],
                ['}', ''],
              ]} />
            </div>

            {/* footer status */}
            <div style={{
              height: 26, flexShrink: 0,
              borderTop: '1px solid var(--border)',
              background: 'color-mix(in oklch, var(--background) 92%, var(--card))',
              display: 'flex', alignItems: 'center', gap: 10, padding: '0 14px',
              fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)',
            }}>
              <span>jsonc · 18 lines</span>
              <span>·</span>
              <span style={{ color: 'oklch(0.55 0.13 145)' }}>✓ valid</span>
              <span style={{ flex: 1 }} />
              <span>Ln 11, Col 22</span>
            </div>
          </div>

          {/* right inspector — visual controls */}
          <div style={{
            borderLeft: '1px solid var(--border)',
            background: 'color-mix(in oklch, var(--background) 97%, var(--card))',
            overflow: 'auto',
            display: 'flex', flexDirection: 'column',
          }}>
            <InspectorTitle>Appearance</InspectorTitle>
            <InspectorRow label="Theme">
              <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
                {[['ship-light', '#f0eee9'], ['paper', '#f4ece0', true], ['ship-dark', '#16140f'], ['halo', '#e6edf5']].map(([n, c, sel]) => (
                  <div key={n} style={{
                    padding: '3px 8px', borderRadius: 4,
                    border: sel ? '1.5px solid oklch(0.55 0.14 28)' : '1px solid var(--border)',
                    background: sel ? 'color-mix(in oklch, oklch(0.55 0.14 28) 10%, var(--background))' : 'var(--background)',
                    fontFamily: SHIP_MONO, fontSize: 10,
                    display: 'flex', alignItems: 'center', gap: 4,
                  }}>
                    <span style={{ width: 8, height: 8, borderRadius: 2, background: c, border: '1px solid rgba(0,0,0,0.1)' }} />
                    {n}
                  </div>
                ))}
              </div>
            </InspectorRow>
            <InspectorRow label="Nav">
              <MiniSeg value="top" options={['rail', 'top', 'none']} />
            </InspectorRow>
            <InspectorRow label="Dock">
              <MiniSeg value="off" options={['left', 'center', 'right', 'off']} />
            </InspectorRow>
            <InspectorRow label="Density">
              <MiniSeg value="cozy" options={['tight', 'cozy', 'loose']} />
            </InspectorRow>

            <InspectorTitle>Icon</InspectorTitle>
            <InspectorRow label="Glyph">
              <div style={{ display: 'flex', gap: 3, flexWrap: 'wrap' }}>
                {['P', '◆', '✦', '▲', 'M'].map((c, i) => (
                  <div key={c} style={{
                    width: 22, height: 22, borderRadius: 4,
                    background: i === 0 ? 'color-mix(in oklch, oklch(0.55 0.14 28) 14%, var(--background))' : 'var(--background)',
                    border: i === 0 ? '1.5px solid oklch(0.55 0.14 28)' : '1px solid var(--border)',
                    display: 'grid', placeItems: 'center',
                    fontFamily: SHIP_MONO, fontSize: 11, fontWeight: 700,
                    color: i === 0 ? 'oklch(0.55 0.14 28)' : 'var(--foreground)',
                  }}>{c}</div>
                ))}
              </div>
            </InspectorRow>
            <InspectorRow label="Accent">
              <div style={{ display: 'flex', gap: 5 }}>
                {['oklch(0.55 0.14 28)', 'oklch(0.7 0.16 140)', 'oklch(0.6 0.16 240)', 'oklch(0.62 0.18 305)'].map((s, i) => (
                  <div key={i} style={{
                    width: 18, height: 18, borderRadius: '50%',
                    background: s,
                    boxShadow: i === 0 ? '0 0 0 1.5px var(--background), 0 0 0 3px ' + s : 'inset 0 0 0 1px rgba(0,0,0,0.1)',
                  }} />
                ))}
              </div>
            </InspectorRow>

            <InspectorTitle>OS integration</InspectorTitle>
            <InspectorRow label="Dock"><MiniToggle on /></InspectorRow>
            <InspectorRow label="Spotlight"><MiniToggle on /></InspectorRow>
            <InspectorRow label="CLI alias"><MiniToggle on /></InspectorRow>
          </div>
        </div>
      </PairedHubChrome>

      {/* SPACE — Paperback live */}
      <PairedSpaceChrome
        title="Paperback" sub="~/Writing/paperback"
        icon="P" accent="oklch(0.55 0.14 28)"
        dark={false}
        bg="#f4ece0" fg="#2d1f12" card="#fbf6ec" border="#d6c8b0" muted="#7a6a55"
        fontFamily="Georgia, serif"
        themeName="paper · top · no dock"
        status={<><Dot tone="ok" size={6} /><span>draft saved</span></>}
      >
        {/* top nav */}
        <div style={{
          height: 32, flexShrink: 0,
          display: 'flex', alignItems: 'center', gap: 14,
          padding: '0 18px',
          background: '#fbf6ec', borderBottom: '1px solid #d6c8b0',
          fontFamily: SHIP_FONT, fontSize: 11.5, color: '#7a6a55',
        }}>
          {[['Posts', false], ['Drafts', true], ['Schedule', false], ['Analytics', false]].map(([l, a]) => (
            <span key={l} style={{
              padding: '8px 0',
              color: a ? '#a14b30' : '#7a6a55',
              fontWeight: a ? 600 : 500,
              borderBottom: a ? '2px solid #a14b30' : '2px solid transparent',
            }}>{l}</span>
          ))}
          <span style={{ flex: 1 }} />
          <span style={{
            padding: '3px 9px', borderRadius: 5,
            background: '#a14b30', color: '#fbf6ec',
            fontWeight: 600, fontSize: 11,
          }}>Publish →</span>
        </div>

        {/* editor */}
        <div style={{
          flex: 1, padding: '18px 28px',
          fontFamily: 'Georgia, serif',
          display: 'flex', flexDirection: 'column', gap: 8,
          minHeight: 0, overflow: 'hidden', position: 'relative',
        }}>
          <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: '#7a6a55', letterSpacing: '0.12em', textTransform: 'uppercase' }}>
            Draft · 4m ago · live editing on
          </div>
          <div style={{ fontSize: 22, fontWeight: 600, letterSpacing: '-0.01em', lineHeight: 1.15 }}>
            On the slow death of the workspace tab
          </div>
          <div style={{ fontSize: 13, lineHeight: 1.6, color: 'color-mix(in oklch, #2d1f12 80%, transparent)' }}>
            For most of the last decade, the workspace has been a tab — one of many — inside a
            browser-shaped vessel called The App. <span style={{ background: 'color-mix(in oklch, #a14b30 15%, transparent)', padding: '0 3px' }}>What the OS</span> already does well wins
            over what The App tries to recreate.
          </div>
          <div style={{ display: 'flex', gap: 5, marginTop: 4 }}>
            {['workspaces', 'os', 'focus'].map(t => (
              <span key={t} style={{ padding: '1px 7px', borderRadius: 999, border: '1px solid #d6c8b0', background: '#fbf6ec', fontSize: 10, color: '#7a6a55', fontFamily: SHIP_MONO }}>{t}</span>
            ))}
          </div>

          {/* live-edit ripple — a subtle pulse on the chrome to show it just re-rendered */}
          <div style={{
            position: 'absolute', top: -2, left: -2, right: -2, bottom: -2,
            border: '2px solid oklch(0.55 0.14 28)',
            borderRadius: 8,
            opacity: 0,
            animation: 'sb-pulse 2.4s ease-in-out infinite',
            pointerEvents: 'none',
          }} />
        </div>
      </PairedSpaceChrome>
    </TwoWindow>
  );
}

// ──────────────────────────────────────────────────────────────
// Code rendering helpers
// ──────────────────────────────────────────────────────────────
function CodeBlock({ lines }) {
  return (
    <pre style={{
      margin: 0, fontFamily: SHIP_MONO, fontSize: 12, lineHeight: 1.7,
      color: 'var(--foreground)', whiteSpace: 'pre',
    }}>
      {lines.map(([txt, modKind], i) => (
        <div key={i} style={{
          display: 'grid', gridTemplateColumns: '38px 1fr',
          alignItems: 'baseline',
          background: modKind === 'mod' ? 'color-mix(in oklch, oklch(0.7 0.16 80) 8%, transparent)' :
                      modKind === 'add' ? 'color-mix(in oklch, oklch(0.6 0.13 145) 8%, transparent)' :
                      'transparent',
          position: 'relative',
        }}>
          {modKind ? (
            <span style={{
              position: 'absolute', left: 0, top: 0, bottom: 0, width: 2,
              background: modKind === 'mod' ? 'oklch(0.65 0.18 60)' : 'oklch(0.6 0.13 145)',
            }} />
          ) : null}
          <span style={{ color: 'var(--muted-foreground)', opacity: 0.5, userSelect: 'none', textAlign: 'right', paddingRight: 12 }}>{i + 1}</span>
          <code dangerouslySetInnerHTML={{ __html: paintJSON ? paintJSON(txt) : txt }} />
        </div>
      ))}
    </pre>
  );
}

// Simple inspector primitives
function InspectorTitle({ children }) {
  return (
    <div style={{
      padding: '12px 14px 4px',
      fontFamily: SHIP_MONO, fontSize: 9.5,
      letterSpacing: '0.14em', textTransform: 'uppercase', fontWeight: 600,
      color: 'var(--muted-foreground)',
      borderTop: '1px solid color-mix(in oklch, var(--border) 50%, transparent)',
    }}>{children}</div>
  );
}
function InspectorRow({ label, children }) {
  return (
    <div style={{ padding: '4px 14px 8px', display: 'grid', gridTemplateColumns: '60px 1fr', gap: 8, alignItems: 'center' }}>
      <span style={{ fontSize: 11, color: 'var(--muted-foreground)' }}>{label}</span>
      <div>{children}</div>
    </div>
  );
}
function MiniSeg({ value, options }) {
  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(' + options.length + ', 1fr)', border: '1px solid var(--border)', borderRadius: 4, overflow: 'hidden', background: 'var(--background)' }}>
      {options.map((o, i) => (
        <div key={o} style={{
          padding: '3px 0', textAlign: 'center',
          fontSize: 10, fontFamily: SHIP_MONO,
          background: o === value ? 'var(--foreground)' : 'transparent',
          color: o === value ? 'var(--background)' : 'var(--muted-foreground)',
          borderRight: i < options.length - 1 ? '1px solid var(--border)' : 'none',
        }}>{o}</div>
      ))}
    </div>
  );
}
function MiniToggle({ on }) {
  return (
    <div style={{
      width: 26, height: 14, borderRadius: 7,
      background: on ? 'var(--primary)' : 'color-mix(in oklch, var(--foreground) 14%, transparent)',
      position: 'relative',
    }}>
      <span style={{ position: 'absolute', top: 2, left: on ? 14 : 2, width: 10, height: 10, borderRadius: '50%', background: 'white', boxShadow: '0 1px 2px rgba(0,0,0,0.2)' }} />
    </div>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 2: PIPELINES — Hub IDE (DAG + code) + Ship space (live runs)
// ──────────────────────────────────────────────────────────────
function PairedPipelines() {
  const accent = 'oklch(0.62 0.16 35)';
  return (
    <TwoWindow
      syncLabel="live · overnight"
      syncDetail="run #482 · embed 64%"
      accent={accent}
    >
      <PairedHubChrome
        title="studio · ship / pipelines"
        breadcrumb={
          <>
            <span style={{ color: 'var(--muted-foreground)' }}>Studio</span>
            <span style={{ color: 'var(--muted-foreground)' }}>›</span>
            <SpaceMark size={14} accent={accent} char="S" />
            <span style={{ fontWeight: 600 }}>Ship</span>
            <span style={{ color: 'var(--muted-foreground)' }}>›</span>
            <span style={{ fontFamily: SHIP_MONO, color: 'var(--muted-foreground)' }}>pipelines / overnight.pipe</span>
          </>
        }
        statusLeft={<span style={{ fontFamily: SHIP_MONO, fontSize: 10.5 }}>edited 4s ago</span>}
        statusRight={<><Dot tone="live" size={6} /><span>linked → run #482</span></>}
      >
        <div style={{ flex: 1, display: 'grid', gridTemplateColumns: '180px 1fr', minHeight: 0 }}>
          <FileTree
            activeId="pipe"
            items={[
              { section: '~/code/ship · pipelines' },
              { kind: 'pipe', name: 'overnight.pipe', depth: 0, id: 'pipe', badge: '●', badgeTone: 'mod' },
              { kind: 'pipe', name: 'index-docs.pipe', depth: 0 },
              { kind: 'pipe', name: 'cron-mastodon.pipe', depth: 0 },
              { divider: true },
              { section: 'Steps' },
              { kind: 'code', name: 'fetch.ts', depth: 0 },
              { kind: 'code', name: 'parse.ts', depth: 0 },
              { kind: 'code', name: 'embed.ts', depth: 0 },
              { kind: 'code', name: 'idx.ts', depth: 0 },
            ]}
          />

          {/* main: DAG editor up top, code below */}
          <div style={{ display: 'flex', flexDirection: 'column', minHeight: 0, overflow: 'hidden' }}>
            {/* toolbar */}
            <div style={{
              height: 36, flexShrink: 0,
              borderBottom: '1px solid var(--border)',
              background: 'color-mix(in oklch, var(--background) 95%, var(--card))',
              display: 'flex', alignItems: 'center', gap: 8, padding: '0 14px',
              fontFamily: SHIP_MONO, fontSize: 11,
            }}>
              <span style={{ fontWeight: 600, color: 'var(--foreground)' }}>overnight.pipe</span>
              <span style={{ color: 'oklch(0.55 0.18 60)' }}>● 1 unsaved</span>
              <span style={{ flex: 1 }} />
              <span style={{ display: 'inline-flex', alignItems: 'center', gap: 5, color: 'var(--muted-foreground)' }}>
                <Dot tone="live" size={6} /> run #482 · 4m 12s elapsed
              </span>
              <span style={{ padding: '3px 9px', borderRadius: 4, border: '1px solid var(--border)', background: 'var(--background)', cursor: 'pointer' }}>Re-run</span>
              <span style={{ padding: '3px 9px', borderRadius: 4, border: '1px solid var(--primary)', background: 'var(--primary)', color: 'var(--primary-foreground)', cursor: 'pointer', fontWeight: 600 }}>Save</span>
            </div>

            {/* DAG canvas */}
            <div style={{
              flexShrink: 0,
              padding: '20px 24px',
              backgroundImage: 'radial-gradient(circle at 1px 1px, color-mix(in oklch, var(--foreground) 12%, transparent) 1px, transparent 0)',
              backgroundSize: '14px 14px',
              borderBottom: '1px solid var(--border)',
              height: 280,
              position: 'relative',
            }}>
              {/* edges */}
              <svg viewBox="0 0 720 240" preserveAspectRatio="none" style={{ position: 'absolute', inset: '20px 24px', width: 'calc(100% - 48px)', height: 'calc(100% - 40px)', pointerEvents: 'none' }}>
                <defs>
                  <marker id="ah" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto">
                    <path d="M0,0 L10,5 L0,10 z" fill="var(--muted-foreground)" opacity="0.6" />
                  </marker>
                </defs>
                <path d="M120 60 Q180 60 220 80 T320 100" fill="none" stroke="oklch(0.55 0.13 145)" strokeWidth="1.5" markerEnd="url(#ah)" />
                <path d="M380 100 L460 100" fill="none" stroke="oklch(0.65 0.18 60)" strokeWidth="2" markerEnd="url(#ah)" />
                <path d="M520 100 L600 100" fill="none" stroke="var(--muted-foreground)" strokeWidth="1.5" strokeDasharray="4 3" markerEnd="url(#ah)" opacity="0.5" />
                <path d="M120 180 Q180 180 220 160 T320 140" fill="none" stroke="oklch(0.55 0.13 145)" strokeWidth="1.5" markerEnd="url(#ah)" />
              </svg>

              {/* nodes */}
              <PipeNode x={40} y={32} label="watch" sub="*/30s" status="ok" kind="trigger" />
              <PipeNode x={40} y={150} label="manual" sub="trigger" status="idle" kind="trigger" />
              <PipeNode x={260} y={70} label="fetch" sub="rss + sitemap" status="ok" tag="2s · 38 items" />
              <PipeNode x={420} y={70} label="parse" sub="extract markdown" status="ok" tag="1s · 28 docs" />
              <PipeNode x={580} y={70} label="embed" sub="text-3-small" status="run" tag="64% · 18/28" tone="warn" />
              <PipeNode x={740} y={70} label="idx" sub="upsert qdrant" status="pending" tag="—" muted />
              <PipeNode x={580} y={170} label="notify" sub="discord webhook" status="branch" tag="on:err" muted />
            </div>

            {/* selected node — code editor */}
            <div style={{ flex: 1, minHeight: 0, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
              <div style={{
                padding: '6px 14px',
                borderBottom: '1px solid var(--border)',
                background: 'color-mix(in oklch, var(--background) 96%, var(--card))',
                fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)',
                display: 'flex', alignItems: 'center', gap: 8,
              }}>
                <span style={{ color: 'var(--foreground)', fontWeight: 600 }}>embed</span>
                <span>·</span>
                <span>embed.ts</span>
                <span style={{ flex: 1 }} />
                <span style={{ color: 'oklch(0.55 0.18 60)' }}>● modified · was max=32</span>
              </div>
              <div style={{ flex: 1, minHeight: 0, overflow: 'auto', background: 'var(--background)', padding: '10px 0' }}>
                <CodeBlock lines={[
                  ["export default step({", ''],
                  ["  name: 'embed',", ''],
                  ["  in:  { docs: t.array(Doc) },", ''],
                  ["  out: { vectors: t.array(Vector) },", ''],
                  ["  parallel: 8,", 'mod'],
                  ["  async run({ docs, openai }) {", ''],
                  ["    return Promise.all(", ''],
                  ["      docs.map(d =>", ''],
                  ["        openai.embed('text-3-small', d.body)", ''],
                  ["      )", ''],
                  ["    );", ''],
                  ["  }", ''],
                  ["});", ''],
                ]} />
              </div>
            </div>
          </div>
        </div>
      </PairedHubChrome>

      {/* SPACE — Ship dark, pipelines view */}
      <PairedSpaceChrome
        title="Ship" sub="~/code/ship"
        icon="S" accent={accent}
        dark={true}
        bg="#16140f" fg="#e8e1cf" card="#1f1c16" border="#332d22" muted="#7c7565"
        themeName="ship-dark · pipelines"
        status={<><Dot tone="live" size={6} /><span>3 agents</span></>}
      >
        <div style={{ flex: 1, display: 'flex', minHeight: 0, fontFamily: SHIP_FONT }}>
          {/* rail */}
          <div style={{
            width: 38, flexShrink: 0,
            borderRight: '1px solid #332d22',
            background: 'rgba(0,0,0,0.2)',
            display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '8px 0',
            gap: 3,
          }}>
            {['bot', 'sparkles', 'workflow', 'library'].map((g, i) => (
              <div key={g} style={{
                width: 24, height: 24, borderRadius: 5,
                background: i === 2 ? 'color-mix(in oklch, ' + accent + ' 22%, transparent)' : 'transparent',
                color: i === 2 ? accent : '#7c7565',
                display: 'grid', placeItems: 'center',
              }}>
                <RailGlyph kind={g} />
              </div>
            ))}
          </div>

          {/* pipelines view */}
          <div style={{ flex: 1, padding: '14px 18px', display: 'flex', flexDirection: 'column', gap: 14, minHeight: 0, overflow: 'hidden' }}>
            {/* header */}
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <div>
                <div style={{ fontFamily: SHIP_MONO, fontSize: 9.5, color: '#7c7565', letterSpacing: '0.14em', textTransform: 'uppercase' }}>pipeline · overnight</div>
                <div style={{ fontSize: 15, fontWeight: 600, marginTop: 2, color: '#e8e1cf' }}>Run #482</div>
              </div>
              <span style={{ flex: 1 }} />
              <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: '#7c7565' }}>started 4m 12s ago · auto via watch</span>
            </div>

            {/* live DAG */}
            <div style={{
              background: '#1f1c16', border: '1px solid #332d22', borderRadius: 8,
              padding: 14, position: 'relative', height: 130, flexShrink: 0,
            }}>
              <div style={{ position: 'absolute', top: 0, left: 0, width: 56, height: 2, background: accent }} />
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 8, position: 'relative', height: '100%' }}>
                <span style={{ position: 'absolute', top: '50%', left: '5%', right: '5%', height: 1, background: '#332d22', zIndex: 0 }} />
                {[
                  ['fetch', 'ok', '2s'],
                  ['parse', 'ok', '1s'],
                  ['embed', 'run', '64%'],
                  ['idx', 'pending', '—'],
                ].map(([n, s, t], i) => {
                  const c = s === 'ok' ? 'oklch(0.6 0.13 145)' : s === 'run' ? 'oklch(0.65 0.18 60)' : '#7c7565';
                  const bg = s === 'ok' ? 'color-mix(in oklch, oklch(0.6 0.13 145) 18%, transparent)'
                          : s === 'run' ? 'color-mix(in oklch, oklch(0.65 0.18 60) 22%, transparent)'
                          : 'rgba(0,0,0,0.2)';
                  return (
                    <div key={n} style={{
                      border: '1px solid ' + (s === 'pending' ? '#332d22' : c),
                      background: bg,
                      borderRadius: 6, padding: 8, zIndex: 1,
                      display: 'flex', flexDirection: 'column', justifyContent: 'space-between',
                    }}>
                      <div style={{ fontFamily: SHIP_MONO, fontSize: 11, fontWeight: 600, color: c }}>{n}</div>
                      <div style={{ fontFamily: SHIP_MONO, fontSize: 9.5, color: '#7c7565' }}>{t}</div>
                    </div>
                  );
                })}
              </div>
            </div>

            {/* logs stream */}
            <div style={{
              flex: 1, minHeight: 0,
              background: '#0e0d0a', border: '1px solid #332d22', borderRadius: 8,
              padding: '10px 12px',
              fontFamily: SHIP_MONO, fontSize: 10.5,
              overflow: 'hidden',
              display: 'flex', flexDirection: 'column', gap: 3,
              color: '#a59c89',
            }}>
              {[
                ['12:04:18', 'embed', 'doc 17/28 · workspace-tab.md · 1842 tokens', 'live'],
                ['12:04:17', 'embed', 'doc 16/28 · death-of-tabs.md · 1502 tokens', 'ok'],
                ['12:04:15', 'embed', 'doc 15/28 · ship-runtime.md · 2204 tokens', 'ok'],
                ['12:04:13', 'embed', 'parallelism bumped 4 → 8', 'note'],
                ['12:04:12', 'parse',  '28 docs from 38 fetched', 'ok'],
                ['12:04:10', 'fetch',  'rss · 32 new · sitemap · 6 new', 'ok'],
              ].map(([t, step, msg, tone], i) => (
                <div key={i} style={{ display: 'grid', gridTemplateColumns: '60px 56px 1fr', gap: 8 }}>
                  <span style={{ color: '#665e4f' }}>{t}</span>
                  <span style={{ color: tone === 'note' ? 'oklch(0.65 0.18 60)' : tone === 'live' ? '#e0a16b' : 'oklch(0.6 0.13 145)' }}>{step}</span>
                  <span style={{ color: tone === 'live' ? '#e8e1cf' : '#a59c89' }}>{msg}</span>
                </div>
              ))}
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, color: '#e0a16b' }}>
                <span style={{ width: 6, height: 12, background: '#e0a16b', animation: 'sb-pulse 1s linear infinite' }} />
              </div>
            </div>
          </div>
        </div>
      </PairedSpaceChrome>
    </TwoWindow>
  );
}

function PipeNode({ x, y, label, sub, status, tag, kind, tone, muted }) {
  const colors = {
    ok: { c: 'oklch(0.55 0.13 145)', bg: 'color-mix(in oklch, oklch(0.6 0.13 145) 8%, var(--card))' },
    run: { c: 'oklch(0.55 0.18 60)', bg: 'color-mix(in oklch, oklch(0.65 0.18 60) 10%, var(--card))' },
    pending: { c: 'var(--muted-foreground)', bg: 'var(--card)' },
    idle: { c: 'var(--muted-foreground)', bg: 'var(--card)' },
    branch: { c: 'oklch(0.55 0.2 25)', bg: 'var(--card)' },
  }[status] || { c: 'var(--border)', bg: 'var(--card)' };
  const isTrig = kind === 'trigger';
  return (
    <div style={{
      position: 'absolute',
      left: x, top: y,
      width: isTrig ? 110 : 140,
      padding: '7px 10px',
      borderRadius: isTrig ? 999 : 8,
      border: '1.5px solid ' + (muted ? 'var(--border)' : colors.c),
      background: muted ? 'var(--card)' : colors.bg,
      boxShadow: status === 'run' ? '0 0 0 4px color-mix(in oklch, oklch(0.65 0.18 60) 10%, transparent)' : 'none',
      display: 'flex', flexDirection: 'column', gap: 2,
      fontFamily: SHIP_FONT,
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
        <Dot tone={status === 'ok' ? 'ok' : status === 'run' ? 'live' : status === 'branch' ? 'err' : 'idle'} size={6} />
        <span style={{ fontWeight: 600, fontSize: 11.5, color: muted ? 'var(--muted-foreground)' : 'var(--foreground)' }}>{label}</span>
      </div>
      {sub && <div style={{ fontFamily: SHIP_MONO, fontSize: 9.5, color: 'var(--muted-foreground)' }}>{sub}</div>}
      {tag && !isTrig && <div style={{ fontFamily: SHIP_MONO, fontSize: 9.5, color: colors.c, fontWeight: 600 }}>{tag}</div>}
    </div>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 3: AGENTS/SKILLS — Hub IDE (skill editor) + Ship space (agent at work)
// ──────────────────────────────────────────────────────────────
function PairedSkills() {
  const accent = 'oklch(0.6 0.18 280)'; // skills tend toward purple
  return (
    <TwoWindow
      syncLabel="live · doc lane"
      syncDetail="hot-reloaded · turn 7"
      accent={accent}
    >
      <PairedHubChrome
        title="studio · ship / skills"
        breadcrumb={
          <>
            <span style={{ color: 'var(--muted-foreground)' }}>Studio</span>
            <span style={{ color: 'var(--muted-foreground)' }}>›</span>
            <SpaceMark size={14} accent="oklch(0.62 0.16 35)" char="S" />
            <span style={{ fontWeight: 600 }}>Ship</span>
            <span style={{ color: 'var(--muted-foreground)' }}>›</span>
            <span style={{ fontFamily: SHIP_MONO, color: 'var(--muted-foreground)' }}>skills / refactor-modules.md</span>
          </>
        }
        statusLeft={<span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'oklch(0.55 0.18 60)' }}>● 2 unsaved · auto-reload on</span>}
        statusRight={<><Dot tone="live" size={6} /><span>Doc Lane · turn 7</span></>}
      >
        <div style={{ flex: 1, display: 'grid', gridTemplateColumns: '180px 1fr 240px', minHeight: 0 }}>
          <FileTree
            activeId="skill"
            items={[
              { section: 'Doc Lane · agent' },
              { kind: 'doc', name: 'agent.jsonc', depth: 0 },
              { kind: 'folder', name: 'skills/', depth: 0 },
              { kind: 'md', name: 'refactor-modules.md', depth: 1, id: 'skill', badge: '●', badgeTone: 'mod' },
              { kind: 'md', name: 'write-docs.md', depth: 1 },
              { kind: 'md', name: 'lint-prose.md', depth: 1 },
              { kind: 'folder', name: 'tools/', depth: 0 },
              { kind: 'code', name: 'shell.ts', depth: 1 },
              { kind: 'code', name: 'edit.ts', depth: 1 },
              { divider: true },
              { section: 'Conversations' },
              { kind: 'doc', name: 'turn 7 · live', depth: 0 },
              { kind: 'doc', name: 'turn 6 · history', depth: 0 },
            ]}
          />

          {/* markdown editor */}
          <div style={{ display: 'flex', flexDirection: 'column', minHeight: 0, overflow: 'hidden' }}>
            <div style={{
              height: 30, flexShrink: 0,
              display: 'flex', alignItems: 'stretch',
              background: 'color-mix(in oklch, var(--background) 95%, var(--card))',
              borderBottom: '1px solid var(--border)',
              fontFamily: SHIP_MONO, fontSize: 11,
            }}>
              <div style={{ padding: '0 14px', display: 'flex', alignItems: 'center', gap: 6, background: 'var(--background)', borderRight: '1px solid var(--border)', fontWeight: 600 }}>
                <FileGlyph kind="md" /> refactor-modules.md
                <span style={{ color: 'oklch(0.55 0.18 60)' }}>●</span>
              </div>
              <span style={{ flex: 1 }} />
              <div style={{ padding: '0 14px', display: 'flex', alignItems: 'center', gap: 8, color: 'var(--muted-foreground)' }}>
                <span style={{ display: 'inline-flex', alignItems: 'center', gap: 4, padding: '2px 7px', borderRadius: 999, border: '1px solid var(--border)' }}>
                  <Dot tone="live" size={5} /> hot reload
                </span>
              </div>
            </div>

            <div style={{ flex: 1, minHeight: 0, overflow: 'auto', padding: '14px 18px', background: 'var(--background)' }}>
              <MarkdownPreview />
            </div>
          </div>

          {/* skill metadata */}
          <div style={{
            borderLeft: '1px solid var(--border)',
            background: 'color-mix(in oklch, var(--background) 97%, var(--card))',
            overflow: 'auto',
            display: 'flex', flexDirection: 'column',
          }}>
            <InspectorTitle>Skill</InspectorTitle>
            <InspectorRow label="Name"><InspectorVal>refactor-modules</InspectorVal></InspectorRow>
            <InspectorRow label="Trigger"><InspectorVal>when:asked</InspectorVal></InspectorRow>
            <InspectorRow label="Cost"><InspectorVal>~6k tok</InspectorVal></InspectorRow>

            <InspectorTitle>Tools allowed</InspectorTitle>
            <ToolChip name="edit" scope="fs:write" on />
            <ToolChip name="shell" scope="zsh:cargo,rustc" on />
            <ToolChip name="search" scope="ripgrep" on />
            <ToolChip name="net" scope="—" on={false} />

            <InspectorTitle>Used by agents</InspectorTitle>
            <AgentChip name="Doc Lane" sub="ship · active" tone="live" />
            <AgentChip name="Rust Lane" sub="ship · idle" tone="idle" />

            <InspectorTitle>Stats</InspectorTitle>
            <InspectorRow label="Invocations"><InspectorVal>418</InspectorVal></InspectorRow>
            <InspectorRow label="Avg turns"><InspectorVal>5.4</InspectorVal></InspectorRow>
            <InspectorRow label="Success"><InspectorVal>92%</InspectorVal></InspectorRow>
          </div>
        </div>
      </PairedHubChrome>

      {/* SPACE — Ship dark, agent surface */}
      <PairedSpaceChrome
        title="Ship" sub="~/code/ship"
        icon="S" accent="oklch(0.62 0.16 35)"
        dark={true}
        bg="#16140f" fg="#e8e1cf" card="#1f1c16" border="#332d22" muted="#7c7565"
        themeName="ship-dark · agent"
        status={<><Dot tone="live" size={6} /><span>doc lane · thinking</span></>}
      >
        <div style={{ flex: 1, display: 'flex', minHeight: 0, fontFamily: SHIP_FONT }}>
          {/* agents rail */}
          <div style={{
            width: 38, flexShrink: 0,
            borderRight: '1px solid #332d22',
            background: 'rgba(0,0,0,0.2)',
            display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '8px 0',
            gap: 3,
          }}>
            {[['bot', accent], ['sparkles', '#e0a16b'], ['workflow', '#7c7565'], ['library', '#7c7565']].map(([g, c], i) => (
              <div key={i} style={{
                width: 24, height: 24, borderRadius: 5,
                background: i === 0 ? 'color-mix(in oklch, ' + c + ' 22%, transparent)' : 'transparent',
                color: i === 0 ? c : '#7c7565',
                display: 'grid', placeItems: 'center',
                position: 'relative',
              }}>
                <RailGlyph kind={g} />
                {i === 0 && <span style={{ position: 'absolute', right: -2, top: -2, width: 7, height: 7, borderRadius: '50%', background: 'oklch(0.65 0.18 60)', border: '1.5px solid #16140f' }} />}
              </div>
            ))}
          </div>

          {/* chat */}
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0, overflow: 'hidden' }}>
            <div style={{
              padding: '10px 16px', borderBottom: '1px solid #332d22',
              display: 'flex', alignItems: 'center', gap: 10,
              background: 'rgba(0,0,0,0.15)',
            }}>
              <div style={{
                width: 28, height: 28, borderRadius: 7,
                background: 'color-mix(in oklch, ' + accent + ' 24%, transparent)',
                color: accent,
                display: 'grid', placeItems: 'center',
              }}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8"><rect x="3" y="9" width="18" height="11" rx="2" /><path d="M12 5v4M9 5h6M8 14h.01M16 14h.01" /></svg>
              </div>
              <div>
                <div style={{ fontSize: 12.5, fontWeight: 600, color: '#e8e1cf' }}>Doc Lane</div>
                <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: '#7c7565' }}>using <span style={{ color: accent }}>refactor-modules</span> · turn 7 of n</div>
              </div>
              <span style={{ flex: 1 }} />
              <span style={{ fontFamily: SHIP_MONO, fontSize: 10, color: '#7c7565', padding: '2px 7px', borderRadius: 999, border: '1px solid #332d22' }}>2 tools used</span>
            </div>

            <div style={{ flex: 1, minHeight: 0, overflow: 'auto', padding: '12px 16px', display: 'flex', flexDirection: 'column', gap: 12 }}>
              <ChatMsg role="user">Split <code style={{ fontFamily: SHIP_MONO, color: '#e0a16b' }}>sui-runtime.ts</code> into runtime/registry/dom modules.</ChatMsg>
              <ChatMsg role="agent" accent={accent}>
                Reading the file. It's 2.1k lines — three concerns are tangled (loop, registry, DOM mutation). I'll cut along those seams.
              </ChatMsg>
              <ChatToolCall name="shell" cmd="rg 'register|render|mount' src/runtime.ts -c" result="48 matches across 14 sections" />
              <ChatToolCall name="edit" cmd="3 files · +412 −387" status="run" />
              <ChatMsg role="agent" accent={accent}>
                <span style={{ background: 'color-mix(in oklch, ' + accent + ' 18%, transparent)', padding: '0 4px', borderRadius: 3 }}>Hot reload caught your edit</span> — I'm now skipping the registry split since you removed that
                bullet. Continuing with runtime + DOM only.
                <span style={{ display: 'inline-block', width: 1.5, height: 11, background: accent, marginLeft: 4, verticalAlign: 'baseline', animation: 'sb-pulse 1s linear infinite' }} />
              </ChatMsg>
            </div>

            <div style={{ padding: '10px 14px', borderTop: '1px solid #332d22', background: 'rgba(0,0,0,0.15)' }}>
              <div style={{
                padding: '8px 12px', borderRadius: 8,
                background: '#1f1c16', border: '1px solid #332d22',
                color: '#7c7565', fontSize: 12,
                display: 'flex', alignItems: 'center', gap: 8,
              }}>
                <span style={{ flex: 1 }}>follow-up…</span>
                <Kbd>⏎</Kbd>
              </div>
            </div>
          </div>
        </div>
      </PairedSpaceChrome>
    </TwoWindow>
  );
}

function MarkdownPreview() {
  return (
    <div style={{ fontFamily: SHIP_FONT, color: 'var(--foreground)', fontSize: 13, lineHeight: 1.6 }}>
      {/* frontmatter */}
      <div style={{
        padding: '8px 12px', borderRadius: 6,
        background: 'color-mix(in oklch, var(--foreground) 4%, transparent)',
        border: '1px solid var(--border)',
        marginBottom: 16,
        fontFamily: SHIP_MONO, fontSize: 11.5, color: 'var(--muted-foreground)',
        lineHeight: 1.7,
      }}>
        <div><span style={{ color: 'var(--syntax-keyword, oklch(0.5 0.18 270))' }}>name:</span> refactor-modules</div>
        <div><span style={{ color: 'var(--syntax-keyword, oklch(0.5 0.18 270))' }}>trigger:</span> when:asked</div>
        <div><span style={{ color: 'var(--syntax-keyword, oklch(0.5 0.18 270))' }}>tools:</span> [edit, shell, search]</div>
      </div>

      <div style={{ fontSize: 18, fontWeight: 600, letterSpacing: '-0.01em', marginBottom: 8 }}>Refactor modules</div>
      <p style={{ margin: '0 0 12px' }}>
        Use when the user asks to split a file along architectural seams.
        Always plan the cut <i>before</i> editing.
      </p>

      <h3 style={{ fontSize: 14, fontWeight: 600, margin: '14px 0 6px' }}>Process</h3>
      <ol style={{ paddingLeft: 22, margin: 0, display: 'flex', flexDirection: 'column', gap: 4 }}>
        <li>Read the file and identify <b>2–4 seams</b> by concern.</li>
        <li style={{
          background: 'color-mix(in oklch, oklch(0.7 0.16 80) 8%, transparent)',
          borderRadius: 4, padding: '2px 4px', position: 'relative',
        }}>
          <span style={{ position: 'absolute', left: -22, top: 4, fontFamily: SHIP_MONO, fontSize: 9, color: 'oklch(0.55 0.18 60)' }}>●</span>
          Propose the split. <s style={{ opacity: 0.45 }}>Always include registry as a separate module.</s>
        </li>
        <li>Cut, run tests after each move.</li>
        <li>Update imports in callers; never broaden visibility silently.</li>
      </ol>

      <h3 style={{ fontSize: 14, fontWeight: 600, margin: '14px 0 6px' }}>Rules</h3>
      <ul style={{ paddingLeft: 22, margin: 0, display: 'flex', flexDirection: 'column', gap: 4, color: 'var(--muted-foreground)' }}>
        <li>Don't rename symbols in the same pass.</li>
        <li style={{
          background: 'color-mix(in oklch, oklch(0.7 0.16 80) 8%, transparent)',
          borderRadius: 4, padding: '2px 4px', position: 'relative',
        }}>
          <span style={{ position: 'absolute', left: -22, top: 4, fontFamily: SHIP_MONO, fontSize: 9, color: 'oklch(0.55 0.18 60)' }}>●</span>
          If the file is &lt; 800 lines, ask before splitting.
        </li>
      </ul>
    </div>
  );
}

function InspectorVal({ children }) {
  return <span style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--foreground)' }}>{children}</span>;
}

function ToolChip({ name, scope, on }) {
  return (
    <div style={{
      margin: '2px 14px 4px', padding: '5px 8px',
      borderRadius: 6, background: on ? 'var(--background)' : 'transparent',
      border: '1px solid ' + (on ? 'var(--border)' : 'color-mix(in oklch, var(--border) 50%, transparent)'),
      display: 'flex', alignItems: 'center', gap: 6,
      opacity: on ? 1 : 0.5,
    }}>
      <span style={{ width: 6, height: 6, borderRadius: '50%', background: on ? 'var(--primary)' : 'var(--muted-foreground)' }} />
      <span style={{ fontFamily: SHIP_MONO, fontSize: 11, fontWeight: 600 }}>{name}</span>
      <span style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)', marginLeft: 'auto' }}>{scope}</span>
    </div>
  );
}

function AgentChip({ name, sub, tone }) {
  return (
    <div style={{
      margin: '2px 14px 4px', padding: '6px 8px',
      borderRadius: 6, background: 'var(--background)',
      border: '1px solid var(--border)',
      display: 'flex', alignItems: 'center', gap: 7,
    }}>
      <div style={{
        width: 20, height: 20, borderRadius: 5,
        background: 'color-mix(in oklch, oklch(0.6 0.18 280) 20%, transparent)',
        color: 'oklch(0.6 0.18 280)',
        display: 'grid', placeItems: 'center',
      }}>
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8"><rect x="3" y="9" width="18" height="11" rx="2" /><path d="M12 5v4M9 5h6" /></svg>
      </div>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontSize: 11.5, fontWeight: 600 }}>{name}</div>
        <div style={{ fontFamily: SHIP_MONO, fontSize: 9.5, color: 'var(--muted-foreground)' }}>{sub}</div>
      </div>
      <Dot tone={tone} size={6} />
    </div>
  );
}

function ChatMsg({ role, children, accent }) {
  if (role === 'user') {
    return (
      <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
        <div style={{
          maxWidth: '78%',
          padding: '8px 12px', borderRadius: 10,
          background: 'rgba(255,255,255,0.05)',
          border: '1px solid #332d22',
          fontSize: 12.5,
          lineHeight: 1.55,
        }}>{children}</div>
      </div>
    );
  }
  return (
    <div style={{ display: 'flex', gap: 8 }}>
      <div style={{
        width: 20, height: 20, flexShrink: 0,
        borderRadius: 5,
        background: 'color-mix(in oklch, ' + accent + ' 24%, transparent)',
        color: accent,
        display: 'grid', placeItems: 'center', marginTop: 2,
      }}>
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8"><rect x="3" y="9" width="18" height="11" rx="2" /><path d="M12 5v4M9 5h6" /></svg>
      </div>
      <div style={{
        flex: 1, fontSize: 12.5, color: '#e8e1cf', lineHeight: 1.55,
      }}>{children}</div>
    </div>
  );
}

function ChatToolCall({ name, cmd, result, status }) {
  const accent = status === 'run' ? 'oklch(0.65 0.18 60)' : 'oklch(0.6 0.13 145)';
  return (
    <div style={{ display: 'flex', gap: 8 }}>
      <div style={{ width: 20, flexShrink: 0 }} />
      <div style={{
        flex: 1,
        padding: '7px 10px', borderRadius: 7,
        background: 'rgba(0,0,0,0.25)',
        border: '1px solid #332d22',
        fontFamily: SHIP_MONO, fontSize: 11,
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <span style={{
            padding: '1px 5px', borderRadius: 3,
            background: 'color-mix(in oklch, ' + accent + ' 22%, transparent)',
            color: accent, fontSize: 10, fontWeight: 600,
          }}>{name}</span>
          <span style={{ color: '#a59c89' }}>{cmd}</span>
          {status === 'run' && <Dot tone="live" size={6} />}
        </div>
        {result ? (
          <div style={{ marginTop: 4, fontSize: 10.5, color: '#7c7565' }}>→ {result}</div>
        ) : null}
      </div>
    </div>
  );
}

Object.assign(window, {
  PairedHero, PairedSpaces, PairedPipelines, PairedSkills,
  PAIRED_W, PAIRED_H, HERO_W, HERO_H,
});
