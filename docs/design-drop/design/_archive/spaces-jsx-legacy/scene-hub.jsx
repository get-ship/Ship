// scene-hub.jsx — The new Ship.app. System hub: spaces / studio / marketplace
// / activity / settings / account. Spaces is the default surface.

const HUB_W = 1280;
const HUB_H = 800;

// Fixture: a handful of registered spaces. Mixed running/idle. The two
// "running" entries are the ones we'll also mock as their own OS windows
// in scene-windows so the cross-references hold up.
const SPACES = [
  { id: 'ship', name: 'Ship', sub: 'this project', icon: 'S', accent: 'oklch(0.62 0.16 35)', theme: 'ship-dark', running: true, agents: 3, last: 'now' },
  { id: 'paper', name: 'Paperback', sub: 'publishing', icon: 'P', accent: 'oklch(0.55 0.14 28)', theme: 'paper', running: true, agents: 1, last: 'now' },
  { id: 'lane', name: 'Rust Lane', sub: 'cargo + clippy', icon: '⌘', accent: 'oklch(0.6 0.18 280)', theme: 'console', running: false, agents: 0, last: '2h' },
  { id: 'voice', name: 'Brand Voice', sub: 'tone studio', icon: 'V', accent: 'oklch(0.7 0.16 140)', theme: 'halo', running: false, agents: 0, last: 'yesterday' },
  { id: 'mesh', name: 'Mesh', sub: 'infra mgmt', icon: 'M', accent: 'oklch(0.6 0.16 240)', theme: 'contrast', running: false, agents: 0, last: '3d' },
  { id: 'lab', name: 'Lab', sub: 'experiments', icon: 'L', accent: 'oklch(0.62 0.18 305)', theme: 'glass', running: false, agents: 0, last: '1w' },
];

const HUB_NAV = [
  { id: 'spaces', label: 'Spaces', icon: 'grid', count: SPACES.length },
  { id: 'studio', label: 'Studio', icon: 'wand', hint: 'IDE' },
  { id: 'authority', label: 'Authority', icon: 'shield', count: 2, tone: 'attn' },
  { id: 'marketplace', label: 'Marketplace', icon: 'store' },
  { id: 'activity', label: 'Activity', icon: 'pulse', live: true },
  null, // divider
  { id: 'settings', label: 'Settings', icon: 'gear' },
  { id: 'account', label: 'Account', icon: 'user' },
];

function HubSidebar({ active = 'spaces' }) {
  return (
    <aside style={{
      width: 208, flexShrink: 0,
      borderRight: '1px solid var(--border)',
      background: 'color-mix(in oklch, var(--background) 92%, var(--card))',
      display: 'flex', flexDirection: 'column',
      padding: '14px 10px 12px',
      gap: 2,
    }}>
      {/* brand */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '4px 8px 14px', borderBottom: '1px solid var(--border)', marginBottom: 8 }}>
        <SpaceMark size={24} accent="var(--primary)" char="S" />
        <div>
          <div style={{ fontSize: 13, fontWeight: 600, letterSpacing: '-0.005em' }}>Ship</div>
          <MicroLabel style={{ fontSize: 9, letterSpacing: '0.16em' }}>hub · 1.4.2</MicroLabel>
        </div>
      </div>
      {HUB_NAV.map((item, i) => {
        if (item == null) return <div key={'div-'+i} style={{ height: 1, background: 'var(--border)', margin: '8px 4px' }} />;
        const isActive = item.id === active;
        return (
          <div key={item.id} style={{
            display: 'flex', alignItems: 'center', gap: 10,
            padding: '7px 10px', borderRadius: 7,
            cursor: 'pointer',
            background: isActive ? 'color-mix(in oklch, var(--primary) 10%, transparent)' : 'transparent',
            color: isActive ? 'var(--foreground)' : 'var(--foreground)',
            fontWeight: isActive ? 600 : 500,
            fontSize: 13,
            position: 'relative',
          }}>
            {isActive ? (
              <span style={{ position: 'absolute', left: 0, top: 8, bottom: 8, width: 2, borderRadius: 1, background: 'var(--primary)' }} />
            ) : null}
            <span style={{
              width: 16, height: 16,
              display: 'grid', placeItems: 'center',
              color: isActive ? 'var(--primary)' : 'var(--muted-foreground)',
            }}>
              <NavGlyph kind={item.icon} />
            </span>
            <span style={{ flex: 1 }}>{item.label}</span>
            {item.count != null ? (
              <span style={{
                fontFamily: SHIP_MONO, fontSize: 10.5,
                color: item.tone === 'attn' ? 'var(--primary-foreground, white)' : 'var(--muted-foreground)',
                background: item.tone === 'attn' ? 'var(--primary)' : 'transparent',
                padding: item.tone === 'attn' ? '1px 6px' : 0,
                borderRadius: 999,
                fontWeight: item.tone === 'attn' ? 600 : 400,
              }}>{item.count}</span>
            ) : null}
            {item.live ? <Dot tone="live" size={6} /> : null}
            {item.hint ? (
              <span style={{ fontFamily: SHIP_MONO, fontSize: 9, color: 'var(--muted-foreground)', letterSpacing: '0.08em' }}>{item.hint}</span>
            ) : null}
          </div>
        );
      })}
      <div style={{ flex: 1 }} />
      {/* Quick launcher pill — convenience surface */}
      <div style={{
        padding: '8px 10px', borderRadius: 8, border: '1px dashed var(--border)',
        display: 'flex', alignItems: 'center', gap: 8,
        color: 'var(--muted-foreground)', fontSize: 12,
        background: 'var(--background)',
        marginTop: 8,
      }}>
        <Kbd>⌘</Kbd><Kbd>Space</Kbd>
        <span style={{ marginLeft: 'auto', fontFamily: SHIP_MONO, fontSize: 10 }}>launch</span>
      </div>
    </aside>
  );
}

function NavGlyph({ kind }) {
  const props = { width: 14, height: 14, viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', strokeWidth: 1.8, strokeLinecap: 'round', strokeLinejoin: 'round' };
  switch (kind) {
    case 'grid': return <svg {...props}><rect x="3" y="3" width="7" height="7" rx="1" /><rect x="14" y="3" width="7" height="7" rx="1" /><rect x="3" y="14" width="7" height="7" rx="1" /><rect x="14" y="14" width="7" height="7" rx="1" /></svg>;
    case 'wand': return <svg {...props}><path d="M15 4V2M15 16v-2M8 9h2M20 9h2M17.8 11.8l1.4 1.4M17.8 6.2l1.4-1.4M3 21l9-9" /></svg>;
    case 'store': return <svg {...props}><path d="M3 9l1-5h16l1 5M3 9v11h18V9M3 9h18M9 13h6" /></svg>;
    case 'pulse': return <svg {...props}><path d="M3 12h4l3-8 4 16 3-8h4" /></svg>;
    case 'shield': return <svg {...props}><path d="M12 2l8 3v6c0 5-3.5 9-8 11-4.5-2-8-6-8-11V5l8-3z" /><path d="M9 12l2 2 4-4" /></svg>;
    case 'gear': return <svg {...props}><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 0 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1A2 2 0 1 1 4.3 17l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 0 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1A2 2 0 1 1 7 4.3l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 0 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 0 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z" /></svg>;
    case 'user': return <svg {...props}><circle cx="12" cy="8" r="4" /><path d="M4 21a8 8 0 0 1 16 0" /></svg>;
    default: return null;
  }
}

// One space card in the grid.
function SpaceCard({ space, size = 'lg' }) {
  const w = size === 'lg' ? 240 : 180;
  return (
    <article style={{
      width: w,
      background: 'var(--card)',
      border: '1px solid var(--border)',
      borderRadius: 12,
      padding: 16,
      display: 'flex', flexDirection: 'column',
      gap: 10,
      position: 'relative',
      cursor: 'pointer',
      boxShadow: '0 1px 0 color-mix(in oklch, var(--foreground) 4%, transparent)',
      overflow: 'hidden',
    }}>
      {/* accent strip */}
      <span style={{ position: 'absolute', top: 0, left: 0, right: 0, height: 3, background: space.accent }} />
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <SpaceMark size={42} accent={space.accent} char={space.icon} />
        {space.running ? (
          <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
            <Dot tone="live" size={6} />
            <span style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)' }}>{space.agents} agent{space.agents !== 1 ? 's' : ''}</span>
          </div>
        ) : (
          <span style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)' }}>{space.last}</span>
        )}
      </div>
      <div>
        <div style={{ fontSize: 14, fontWeight: 600, letterSpacing: '-0.005em' }}>{space.name}</div>
        <div style={{ fontSize: 11.5, color: 'var(--muted-foreground)', marginTop: 2, fontFamily: SHIP_MONO }}>{space.sub}</div>
      </div>
      <div style={{
        display: 'flex', alignItems: 'center', gap: 6,
        paddingTop: 10, borderTop: '1px solid color-mix(in oklch, var(--border) 70%, transparent)',
        fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)',
      }}>
        <span style={{
          padding: '2px 6px', borderRadius: 4,
          background: 'color-mix(in oklch, var(--foreground) 5%, transparent)',
        }}>{space.theme}</span>
        <span style={{ marginLeft: 'auto' }}>{space.running ? 'open →' : 'launch →'}</span>
      </div>
    </article>
  );
}

function CreateSpaceCard() {
  return (
    <article style={{
      width: 240,
      border: '1.5px dashed var(--border)',
      borderRadius: 12,
      padding: 16,
      display: 'flex', flexDirection: 'column',
      alignItems: 'center', justifyContent: 'center',
      gap: 12,
      minHeight: 220,
      cursor: 'pointer',
      color: 'var(--muted-foreground)',
    }}>
      <div style={{
        width: 42, height: 42,
        borderRadius: 14,
        border: '1.5px dashed var(--border)',
        display: 'grid', placeItems: 'center',
        color: 'var(--primary)',
      }}>
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><path d="M12 5v14M5 12h14" /></svg>
      </div>
      <div style={{ textAlign: 'center' }}>
        <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--foreground)' }}>New space</div>
        <div style={{ fontSize: 11.5, marginTop: 4, fontFamily: SHIP_MONO }}>name, icon, theme</div>
      </div>
    </article>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard A: Hub · Spaces  (default state)
// ──────────────────────────────────────────────────────────────
function HubSpaces() {
  const running = SPACES.filter(s => s.running);
  const idle = SPACES.filter(s => !s.running);
  return (
    <WinFrame title="Ship" subtitle="hub" style={{ width: HUB_W, height: HUB_H }}>
      <div style={{ flex: 1, display: 'flex', minHeight: 0 }}>
        <HubSidebar active="spaces" />
        <main style={{ flex: 1, padding: '24px 32px 28px', overflow: 'hidden', display: 'flex', flexDirection: 'column', gap: 24 }}>
          {/* header */}
          <div style={{ display: 'flex', alignItems: 'flex-end', gap: 16 }}>
            <div>
              <div style={{ fontSize: 22, fontWeight: 600, letterSpacing: '-0.015em' }}>Your spaces</div>
              <div style={{ color: 'var(--muted-foreground)', marginTop: 2, fontSize: 13 }}>
                Each space is its own window with its own theme, dock and chrome.
              </div>
            </div>
            <div style={{ flex: 1 }} />
            <div style={{
              display: 'flex', alignItems: 'center', gap: 8,
              padding: '6px 10px',
              border: '1px solid var(--border)',
              borderRadius: 8,
              fontSize: 12,
              color: 'var(--muted-foreground)',
              background: 'var(--card)',
              minWidth: 220,
            }}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="11" cy="11" r="7" /><path d="M21 21l-4-4" /></svg>
              <span style={{ flex: 1 }}>Find a space…</span>
              <Kbd>⌘K</Kbd>
            </div>
            <button style={{
              padding: '7px 14px',
              borderRadius: 8,
              border: '1px solid var(--primary)',
              background: 'var(--primary)',
              color: 'var(--primary-foreground)',
              fontSize: 12.5, fontWeight: 600,
              fontFamily: SHIP_FONT,
              cursor: 'pointer',
              display: 'flex', alignItems: 'center', gap: 6,
            }}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round"><path d="M12 5v14M5 12h14" /></svg>
              New space
            </button>
          </div>

          {/* Running now strip */}
          {running.length > 0 && (
            <section>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10, marginBottom: 10 }}>
                <Dot tone="live" size={7} />
                <MicroLabel>Running now</MicroLabel>
                <span style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)' }}>{running.length} window{running.length !== 1 ? 's' : ''}</span>
              </div>
              <div style={{ display: 'flex', gap: 14, flexWrap: 'wrap' }}>
                {running.map(s => (
                  <RunningRow key={s.id} space={s} />
                ))}
              </div>
            </section>
          )}

          {/* All spaces grid */}
          <section style={{ flex: 1, minHeight: 0, overflow: 'hidden' }}>
            <MicroLabel style={{ marginBottom: 12 }}>All spaces · {SPACES.length}</MicroLabel>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, 240px)', gap: 16 }}>
              {idle.map(s => <SpaceCard key={s.id} space={s} />)}
              <CreateSpaceCard />
            </div>
          </section>
        </main>
      </div>
      <HubFooter />
    </WinFrame>
  );
}

function RunningRow({ space }) {
  return (
    <div style={{
      display: 'flex', alignItems: 'center', gap: 12,
      padding: '10px 14px',
      borderRadius: 10,
      background: 'color-mix(in oklch, var(--card) 75%, transparent)',
      border: '1px solid var(--border)',
      minWidth: 260,
      cursor: 'pointer',
    }}>
      <SpaceMark size={32} accent={space.accent} char={space.icon} />
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ fontSize: 13, fontWeight: 600 }}>{space.name}</div>
        <div style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>
          {space.agents} agent{space.agents !== 1 ? 's' : ''} · {space.theme}
        </div>
      </div>
      <Dot tone="live" />
      <button style={{
        padding: '4px 10px', borderRadius: 6,
        border: '1px solid var(--border)', background: 'var(--background)',
        fontSize: 11, fontFamily: SHIP_MONO,
        color: 'var(--foreground)', cursor: 'pointer',
      }}>focus</button>
    </div>
  );
}

function HubFooter() {
  return (
    <footer style={{
      height: 26, flexShrink: 0,
      borderTop: '1px solid var(--border)',
      background: 'color-mix(in oklch, var(--background) 92%, var(--card))',
      display: 'flex', alignItems: 'center', gap: 16,
      padding: '0 14px',
      fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)',
    }}>
      <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}><Dot tone="ok" size={6} />daemon online</span>
      <span>·</span>
      <span>2 spaces running</span>
      <span>·</span>
      <span>4 agents</span>
      <span style={{ marginLeft: 'auto' }}>v1.4.2</span>
    </footer>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard B: Hub · Activity  — live monitoring across all spaces
// ──────────────────────────────────────────────────────────────
function HubActivity() {
  const events = [
    { space: 'Ship', accent: SPACES[0].accent, icon: 'S', agent: 'Rust Lane', op: 'cargo check', tone: 'live', when: '2s' },
    { space: 'Paperback', accent: SPACES[1].accent, icon: 'P', agent: 'Editor', op: 'draft tweet thread', tone: 'live', when: '14s' },
    { space: 'Ship', accent: SPACES[0].accent, icon: 'S', agent: 'Research', op: 'summarise pdf', tone: 'live', when: '47s' },
    { space: 'Ship', accent: SPACES[0].accent, icon: 'S', agent: 'Doc Lane', op: 'generate skill', tone: 'ok', when: '4m' },
    { space: 'Ship', accent: SPACES[0].accent, icon: 'S', agent: 'Loader', op: 'fetch & embed', tone: 'err', when: '7m' },
    { space: 'Paperback', accent: SPACES[1].accent, icon: 'P', agent: 'Publish', op: 'cross-post mastodon', tone: 'ok', when: '12m' },
    { space: 'Rust Lane', accent: SPACES[2].accent, icon: '⌘', agent: '—', op: 'space closed', tone: 'idle', when: '2h' },
  ];
  return (
    <WinFrame title="Ship" subtitle="hub · activity" style={{ width: HUB_W, height: HUB_H }}>
      <div style={{ flex: 1, display: 'flex', minHeight: 0 }}>
        <HubSidebar active="activity" />
        <main style={{ flex: 1, padding: '24px 32px 28px', overflow: 'hidden', display: 'flex', flexDirection: 'column', gap: 20 }}>
          <div style={{ display: 'flex', alignItems: 'flex-end', gap: 16 }}>
            <div>
              <div style={{ fontSize: 22, fontWeight: 600, letterSpacing: '-0.015em', display: 'flex', alignItems: 'center', gap: 10 }}>
                <Dot tone="live" />
                Activity
              </div>
              <div style={{ color: 'var(--muted-foreground)', marginTop: 2, fontSize: 13 }}>
                Live across all spaces, even ones you're not focused on.
              </div>
            </div>
            <div style={{ flex: 1 }} />
            <div style={{ display: 'flex', gap: 8 }}>
              {SPACES.filter(s => s.running).map(s => (
                <span key={s.id} style={{
                  display: 'inline-flex', alignItems: 'center', gap: 6,
                  padding: '4px 8px 4px 4px', borderRadius: 999,
                  border: '1px solid var(--border)', background: 'var(--card)',
                  fontSize: 11, fontFamily: SHIP_MONO,
                }}>
                  <SpaceMark size={18} accent={s.accent} char={s.icon} />
                  {s.name}
                </span>
              ))}
            </div>
          </div>

          {/* metrics row */}
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 14 }}>
            {[
              { lab: 'Events / min', val: '38', delta: '+12%', tone: 'up' },
              { lab: 'Agents running', val: '4', delta: 'across 2 spaces', tone: 'flat' },
              { lab: 'CPU', val: '21%', delta: 'normal', tone: 'flat' },
              { lab: 'Errors (1h)', val: '1', delta: 'Loader', tone: 'err' },
            ].map(m => (
              <div key={m.lab} style={{
                background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10,
                padding: '12px 14px',
              }}>
                <MicroLabel>{m.lab}</MicroLabel>
                <div style={{ display: 'flex', alignItems: 'baseline', gap: 8, marginTop: 6 }}>
                  <div style={{ fontSize: 28, fontWeight: 700, letterSpacing: '-0.02em', fontVariantNumeric: 'tabular-nums' }}>{m.val}</div>
                  <div style={{
                    fontFamily: SHIP_MONO, fontSize: 11,
                    color: m.tone === 'up' ? 'var(--success, oklch(0.55 0.13 145))'
                         : m.tone === 'err' ? 'var(--destructive, oklch(0.55 0.2 25))'
                         : 'var(--muted-foreground)',
                  }}>{m.delta}</div>
                </div>
              </div>
            ))}
          </div>

          {/* event stream */}
          <div style={{
            flex: 1, minHeight: 0, overflow: 'hidden',
            background: 'var(--card)', border: '1px solid var(--border)', borderRadius: 10,
            display: 'flex', flexDirection: 'column',
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '10px 14px', borderBottom: '1px solid var(--border)' }}>
              <MicroLabel>Stream</MicroLabel>
              <Dot tone="live" size={6} />
              <span style={{ flex: 1 }} />
              <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>tail · 50/s</span>
            </div>
            <div style={{ overflow: 'auto', flex: 1 }}>
              {events.map((e, i) => (
                <div key={i} style={{
                  display: 'grid',
                  gridTemplateColumns: '92px 22px minmax(0, 1fr) auto',
                  gap: 12,
                  alignItems: 'center',
                  padding: '8px 14px',
                  borderBottom: i < events.length - 1 ? '1px solid color-mix(in oklch, var(--border) 60%, transparent)' : 'none',
                  fontSize: 12.5,
                }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                    <SpaceMark size={16} accent={e.accent} char={e.icon} />
                    <span style={{ fontFamily: SHIP_MONO, fontSize: 11 }}>{e.space}</span>
                  </div>
                  <Dot tone={e.tone} />
                  <div style={{ display: 'flex', gap: 8, alignItems: 'baseline', minWidth: 0 }}>
                    <span style={{ fontWeight: 600 }}>{e.agent}</span>
                    <span style={{ color: 'var(--muted-foreground)', fontFamily: SHIP_MONO, fontSize: 11, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{e.op}</span>
                  </div>
                  <span style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)' }}>{e.when}</span>
                </div>
              ))}
            </div>
          </div>
        </main>
      </div>
      <HubFooter />
    </WinFrame>
  );
}

Object.assign(window, { HubSpaces, HubActivity, HubSidebar, HubFooter, NavGlyph, SPACES, HUB_W, HUB_H });
