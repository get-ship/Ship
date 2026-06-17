// scene-authority.jsx — Hub as the surface for elevated commands + identity.
// The hub is the *only* surface that can grant capabilities, approve
// destructive commands, or unlock identity. Spaces ask, hub decides.

const AUTH_W = 1280;
const AUTH_H = 800;

// ──────────────────────────────────────────────────────────────
// Fixtures
// ──────────────────────────────────────────────────────────────
const PENDING = [
  {
    id: 'cap-1',
    kind: 'capability',
    space: { name: 'Paperback', icon: 'P', accent: 'oklch(0.55 0.14 28)' },
    agent: 'Editor',
    title: 'Read files under ~/Documents/drafts/',
    scope: 'fs:read',
    why: 'to import “workspace-tab.md” into the current draft',
    age: '12s',
    risk: 'medium',
  },
  {
    id: 'cmd-1',
    kind: 'command',
    space: { name: 'Ship', icon: 'S', accent: 'oklch(0.62 0.16 35)' },
    agent: 'Rust Lane',
    title: 'git push --force-with-lease origin main',
    scope: 'git:write',
    why: 'rebase fixed 3 merge commits, history needs to ship',
    age: '38s',
    risk: 'high',
  },
];

const RECENT = [
  { space: 'Ship', icon: 'S', accent: 'oklch(0.62 0.16 35)', agent: 'Rust Lane', what: 'cargo test --workspace', verdict: 'allow', via: 'policy', when: '1m' },
  { space: 'Paperback', icon: 'P', accent: 'oklch(0.55 0.14 28)', agent: 'Publish', what: 'POST mastodon.social/api/v1/statuses', verdict: 'allow', via: 'grant · always', when: '6m' },
  { space: 'Ship', icon: 'S', accent: 'oklch(0.62 0.16 35)', agent: 'Loader', what: 'fetch openai.com', verdict: 'deny', via: 'you', when: '14m', reason: 'no key configured' },
  { space: 'Ship', icon: 'S', accent: 'oklch(0.62 0.16 35)', agent: 'Doc Lane', what: 'write ~/code/ship/skills/*.md', verdict: 'allow', via: 'grant · session', when: '22m' },
  { space: 'Paperback', icon: 'P', accent: 'oklch(0.55 0.14 28)', agent: 'Editor', what: 'read clipboard', verdict: 'allow', via: 'you · once', when: '38m' },
];

const GRANTS = [
  { space: 'Ship', icon: 'S', accent: 'oklch(0.62 0.16 35)', cap: 'fs:read · ~/code/ship', duration: 'always', uses: 4216 },
  { space: 'Ship', icon: 'S', accent: 'oklch(0.62 0.16 35)', cap: 'shell · cargo, rustc, clippy', duration: 'always', uses: 188 },
  { space: 'Paperback', icon: 'P', accent: 'oklch(0.55 0.14 28)', cap: 'oauth · mastodon.social', duration: 'always', uses: 14 },
  { space: 'Paperback', icon: 'P', accent: 'oklch(0.55 0.14 28)', cap: 'fs:read · ~/Writing/paperback', duration: 'always', uses: 902 },
  { space: 'Ship', icon: 'S', accent: 'oklch(0.62 0.16 35)', cap: 'net · api.openai.com', duration: 'session', uses: 47 },
];

const IDENTITIES = [
  { svc: 'GitHub', handle: 'maddy/ship', tone: 'ok' },
  { svc: 'Mastodon', handle: '@maddy@hachyderm.io', tone: 'ok' },
  { svc: 'OpenAI', handle: 'sk-…3f2c', tone: 'ok' },
  { svc: 'Anthropic', handle: 'sk-…9b1a', tone: 'ok' },
  { svc: 'Linear', handle: '—', tone: 'idle' },
];

// ──────────────────────────────────────────────────────────────
// Shared bits
// ──────────────────────────────────────────────────────────────
function RiskTag({ level }) {
  const t = {
    low: { label: 'low', color: 'var(--muted-foreground)', bg: 'color-mix(in oklch, var(--foreground) 6%, transparent)' },
    medium: { label: 'medium', color: 'oklch(0.55 0.16 60)', bg: 'color-mix(in oklch, oklch(0.65 0.18 60) 14%, transparent)' },
    high: { label: 'high', color: 'var(--destructive, oklch(0.55 0.2 25))', bg: 'color-mix(in oklch, var(--destructive, oklch(0.55 0.2 25)) 14%, transparent)' },
  }[level];
  return (
    <span style={{
      fontFamily: SHIP_MONO, fontSize: 10, fontWeight: 600,
      letterSpacing: '0.08em', textTransform: 'uppercase',
      padding: '2px 7px', borderRadius: 999,
      color: t.color, background: t.bg,
    }}>{t.label}</span>
  );
}

function ScopeTag({ scope }) {
  const [kind] = scope.split(':');
  const palette = {
    fs: 'oklch(0.6 0.16 240)',
    shell: 'oklch(0.6 0.18 305)',
    net: 'oklch(0.65 0.16 145)',
    oauth: 'oklch(0.65 0.16 80)',
    git: 'oklch(0.6 0.18 25)',
  };
  const c = palette[kind] || 'var(--muted-foreground)';
  return (
    <span style={{
      fontFamily: SHIP_MONO, fontSize: 10.5,
      padding: '2px 7px', borderRadius: 4,
      color: c, background: 'color-mix(in oklch, ' + c + ' 12%, transparent)',
      border: '1px solid color-mix(in oklch, ' + c + ' 30%, transparent)',
    }}>{scope}</span>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 1: Authority queue — main hub page
// ──────────────────────────────────────────────────────────────
function HubAuthority() {
  return (
    <WinFrame title="Ship" subtitle="hub · authority" style={{ width: AUTH_W, height: AUTH_H }}>
      <div style={{ flex: 1, display: 'flex', minHeight: 0 }}>
        <HubSidebar active="authority" />
        <main style={{ flex: 1, padding: '24px 32px 28px', overflow: 'hidden', display: 'flex', flexDirection: 'column', gap: 22 }}>
          {/* header */}
          <div style={{ display: 'flex', alignItems: 'flex-end', gap: 16 }}>
            <div>
              <div style={{ fontSize: 22, fontWeight: 600, letterSpacing: '-0.015em', display: 'flex', alignItems: 'center', gap: 10 }}>
                <ShieldIcon />
                Authority
              </div>
              <div style={{ color: 'var(--muted-foreground)', marginTop: 2, fontSize: 13 }}>
                Capability requests, command approvals, and the identities your spaces speak for.
              </div>
            </div>
            <div style={{ flex: 1 }} />
            <button style={{
              padding: '6px 12px', borderRadius: 7,
              border: '1px solid var(--border)', background: 'var(--card)',
              fontSize: 12, fontFamily: SHIP_FONT,
              color: 'var(--foreground)', cursor: 'pointer',
              display: 'flex', alignItems: 'center', gap: 6,
            }}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8"><path d="M21 12a9 9 0 1 1-3-6.7M21 3v6h-6" /></svg>
              Replay decisions
            </button>
            <button style={{
              padding: '7px 14px', borderRadius: 7,
              border: '1px solid var(--border)', background: 'var(--background)',
              fontSize: 12, fontFamily: SHIP_FONT, fontWeight: 600,
              cursor: 'pointer',
            }}>Policy…</button>
          </div>

          {/* Pending */}
          <section style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <Dot tone="live" />
              <MicroLabel>Awaiting your decision</MicroLabel>
              <span style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)' }}>{PENDING.length} pending · oldest 38s</span>
            </div>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
              {PENDING.map(p => <PendingCard key={p.id} req={p} />)}
            </div>
          </section>

          {/* Two-column lower section */}
          <section style={{ flex: 1, minHeight: 0, display: 'grid', gridTemplateColumns: '1.4fr 1fr', gap: 14, overflow: 'hidden' }}>
            {/* Recent decisions */}
            <div style={{
              border: '1px solid var(--border)', borderRadius: 10,
              background: 'var(--card)',
              display: 'flex', flexDirection: 'column', minHeight: 0, overflow: 'hidden',
            }}>
              <div style={{ display: 'flex', alignItems: 'center', padding: '10px 14px', borderBottom: '1px solid var(--border)' }}>
                <MicroLabel>Decisions log</MicroLabel>
                <span style={{ flex: 1 }} />
                <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>last 1h · {RECENT.length} of 88</span>
              </div>
              <div style={{ overflow: 'auto', flex: 1 }}>
                {RECENT.map((r, i) => (
                  <div key={i} style={{
                    display: 'grid',
                    gridTemplateColumns: '110px 18px minmax(0, 1fr) 100px 48px',
                    alignItems: 'center', gap: 10,
                    padding: '9px 14px',
                    borderBottom: i < RECENT.length - 1 ? '1px solid color-mix(in oklch, var(--border) 50%, transparent)' : 'none',
                    fontSize: 12,
                  }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                      <SpaceMark size={14} accent={r.accent} char={r.icon} />
                      <span style={{ fontFamily: SHIP_MONO, fontSize: 11 }}>{r.space}</span>
                    </div>
                    <VerdictDot v={r.verdict} />
                    <div style={{ display: 'flex', gap: 6, alignItems: 'baseline', minWidth: 0 }}>
                      <span style={{ fontWeight: 600 }}>{r.agent}</span>
                      <span style={{ color: 'var(--muted-foreground)', fontFamily: SHIP_MONO, fontSize: 11, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{r.what}</span>
                    </div>
                    <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>{r.via}</span>
                    <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)', textAlign: 'right' }}>{r.when}</span>
                  </div>
                ))}
              </div>
            </div>

            {/* Active grants */}
            <div style={{
              border: '1px solid var(--border)', borderRadius: 10,
              background: 'var(--card)',
              display: 'flex', flexDirection: 'column', minHeight: 0, overflow: 'hidden',
            }}>
              <div style={{ display: 'flex', alignItems: 'center', padding: '10px 14px', borderBottom: '1px solid var(--border)' }}>
                <MicroLabel>Active grants</MicroLabel>
                <span style={{ flex: 1 }} />
                <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>{GRANTS.length} active</span>
              </div>
              <div style={{ overflow: 'auto', flex: 1 }}>
                {GRANTS.map((g, i) => (
                  <div key={i} style={{
                    padding: '10px 14px',
                    borderBottom: i < GRANTS.length - 1 ? '1px solid color-mix(in oklch, var(--border) 50%, transparent)' : 'none',
                    display: 'flex', alignItems: 'center', gap: 10, fontSize: 12,
                  }}>
                    <SpaceMark size={20} accent={g.accent} char={g.icon} />
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--foreground)', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{g.cap}</div>
                      <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)' }}>{g.duration} · {g.uses.toLocaleString()} use{g.uses !== 1 ? 's' : ''}</div>
                    </div>
                    <span style={{
                      fontFamily: SHIP_MONO, fontSize: 10,
                      padding: '2px 7px', borderRadius: 999,
                      border: '1px solid var(--border)',
                      color: 'var(--muted-foreground)', cursor: 'pointer',
                    }}>revoke</span>
                  </div>
                ))}
              </div>
            </div>
          </section>
        </main>
      </div>
      <HubFooter />
    </WinFrame>
  );
}

function PendingCard({ req }) {
  const isCmd = req.kind === 'command';
  return (
    <div style={{
      padding: 16, borderRadius: 10,
      background: 'var(--card)',
      border: '1px solid color-mix(in oklch, var(--primary) 28%, var(--border))',
      boxShadow: '0 0 0 4px color-mix(in oklch, var(--primary) 6%, transparent)',
      display: 'flex', flexDirection: 'column', gap: 12,
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <SpaceMark size={26} accent={req.space.accent} char={req.space.icon} />
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{ fontSize: 12, fontFamily: SHIP_MONO, color: 'var(--muted-foreground)' }}>
            <b style={{ color: 'var(--foreground)', fontWeight: 600 }}>{req.agent}</b> · {req.space.name}
          </div>
          <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)', letterSpacing: '0.08em', textTransform: 'uppercase', marginTop: 2 }}>
            {isCmd ? 'wants to run' : 'requests capability'}
          </div>
        </div>
        <RiskTag level={req.risk} />
        <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>{req.age}</span>
      </div>

      {/* command/capability body */}
      <div style={{
        padding: '10px 12px', borderRadius: 7,
        background: isCmd ? '#1d1a14' : 'color-mix(in oklch, var(--foreground) 4%, transparent)',
        color: isCmd ? '#e0d8c6' : 'var(--foreground)',
        fontFamily: SHIP_MONO, fontSize: 12.5,
        display: 'flex', alignItems: 'center', gap: 8,
        wordBreak: 'break-all',
      }}>
        {isCmd ? <span style={{ color: '#7c7565' }}>$</span> : <ScopeTag scope={req.scope} />}
        <span style={{ flex: 1 }}>{req.title}</span>
      </div>

      <div style={{ fontSize: 12, color: 'var(--muted-foreground)', lineHeight: 1.5 }}>
        <b style={{ color: 'var(--foreground)', fontWeight: 600 }}>Why</b> · {req.why}
      </div>

      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <button style={{
          padding: '7px 12px', borderRadius: 7,
          border: '1px solid var(--border)', background: 'var(--background)',
          fontSize: 12, fontFamily: SHIP_FONT, color: 'var(--destructive, oklch(0.55 0.2 25))',
          fontWeight: 600, cursor: 'pointer',
        }}>Deny</button>
        <span style={{ flex: 1 }} />
        <button style={{
          padding: '7px 12px', borderRadius: 7,
          border: '1px solid var(--border)', background: 'var(--background)',
          fontSize: 12, fontFamily: SHIP_FONT, cursor: 'pointer',
        }}>Allow once</button>
        <button style={{
          padding: '7px 12px', borderRadius: 7,
          border: '1px solid var(--primary)', background: 'var(--primary)',
          color: 'var(--primary-foreground)',
          fontSize: 12, fontFamily: SHIP_FONT, fontWeight: 600, cursor: 'pointer',
        }}>Allow this session</button>
      </div>
    </div>
  );
}

function VerdictDot({ v }) {
  if (v === 'allow') return <Dot tone="ok" size={8} />;
  if (v === 'deny') return <Dot tone="err" size={8} />;
  return <Dot tone="idle" size={8} />;
}

function ShieldIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6">
      <path d="M12 2l8 3v6c0 5-3.5 9-8 11-4.5-2-8-6-8-11V5l8-3z" />
      <path d="M9 12l2 2 4-4" />
    </svg>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 2: Capability request — modal pulled forward in the hub
// ──────────────────────────────────────────────────────────────
function HubCapabilityRequest() {
  return (
    <div style={{ position: 'relative', width: AUTH_W, height: AUTH_H }}>
      {/* dimmed hub behind */}
      <div style={{
        position: 'absolute', inset: 0,
        filter: 'blur(2px) saturate(0.7) brightness(0.94)',
        transform: 'scale(1.01)',
        opacity: 0.55,
        pointerEvents: 'none',
      }}>
        <HubAuthority />
      </div>
      <div style={{
        position: 'absolute', inset: 0,
        background: 'color-mix(in oklch, #1a1610 35%, transparent)',
      }} />

      {/* dialog */}
      <div style={{
        position: 'absolute', top: '50%', left: '50%',
        transform: 'translate(-50%, -50%)',
        width: 520,
        background: 'var(--background)',
        border: '1px solid var(--border)',
        borderRadius: 14,
        boxShadow: '0 40px 100px -28px rgba(0,0,0,0.5), 0 4px 12px -2px rgba(0,0,0,0.15)',
        overflow: 'hidden',
        fontFamily: SHIP_FONT,
      }}>
        {/* header strip */}
        <div style={{
          padding: '14px 18px 12px',
          background: 'linear-gradient(180deg, color-mix(in oklch, var(--primary) 10%, transparent), transparent)',
          borderBottom: '1px solid var(--border)',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <span style={{
              width: 28, height: 28, borderRadius: 8,
              background: 'color-mix(in oklch, var(--primary) 16%, transparent)',
              color: 'var(--primary)',
              display: 'grid', placeItems: 'center',
            }}>
              <ShieldIcon />
            </span>
            <div style={{ flex: 1 }}>
              <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)', letterSpacing: '0.1em', textTransform: 'uppercase' }}>
                Capability request
              </div>
              <div style={{ fontSize: 14, fontWeight: 600, marginTop: 2 }}>Editor wants to read your filesystem</div>
            </div>
            <RiskTag level="medium" />
          </div>
        </div>

        <div style={{ padding: '16px 18px', display: 'flex', flexDirection: 'column', gap: 14 }}>
          {/* asking agent */}
          <div style={{
            display: 'flex', alignItems: 'center', gap: 10,
            padding: 12, borderRadius: 8,
            background: 'var(--card)',
            border: '1px solid var(--border)',
          }}>
            <SpaceMark size={32} accent="oklch(0.55 0.14 28)" char="P" />
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontSize: 12.5, fontWeight: 600 }}>Editor</div>
              <div style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)' }}>Paperback · skill v2.1 · signed</div>
            </div>
            <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>just now</span>
          </div>

          {/* scope */}
          <div>
            <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)', letterSpacing: '0.12em', textTransform: 'uppercase', marginBottom: 6 }}>Scope</div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <ScopeTag scope="fs:read" />
              <code style={{ fontFamily: SHIP_MONO, fontSize: 12.5, color: 'var(--foreground)' }}>~/Documents/drafts/</code>
              <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)', marginLeft: 'auto' }}>recursive · 28 files</span>
            </div>
          </div>

          {/* reasoning */}
          <div>
            <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)', letterSpacing: '0.12em', textTransform: 'uppercase', marginBottom: 6 }}>Why</div>
            <div style={{ fontSize: 12.5, color: 'var(--foreground)', lineHeight: 1.55 }}>
              To import <code style={{ fontFamily: SHIP_MONO, background: 'color-mix(in oklch, var(--foreground) 5%, transparent)', padding: '0 4px', borderRadius: 3 }}>workspace-tab.md</code> into the current draft —
              you dragged it into the editor 4 seconds ago.
            </div>
          </div>

          {/* duration radio */}
          <div>
            <div style={{ fontFamily: SHIP_MONO, fontSize: 10, color: 'var(--muted-foreground)', letterSpacing: '0.12em', textTransform: 'uppercase', marginBottom: 8 }}>For how long</div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 8 }}>
              {[
                ['Just this file', 'one-shot'],
                ['Until I close Paperback', 'session', true],
                ['Always', 'persistent'],
              ].map(([l, k, sel], i) => (
                <div key={k} style={{
                  padding: '8px 10px', borderRadius: 7,
                  border: sel ? '1.5px solid var(--primary)' : '1px solid var(--border)',
                  background: sel ? 'color-mix(in oklch, var(--primary) 8%, var(--background))' : 'var(--background)',
                  cursor: 'pointer',
                }}>
                  <div style={{ fontSize: 11.5, fontWeight: 600 }}>{l}</div>
                  <div style={{ fontFamily: SHIP_MONO, fontSize: 9.5, color: 'var(--muted-foreground)', marginTop: 1 }}>{k}</div>
                </div>
              ))}
            </div>
          </div>

          {/* footer */}
          <div style={{ display: 'flex', gap: 8, alignItems: 'center', paddingTop: 6 }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>
              <Kbd>⎋</Kbd> to deny
            </div>
            <span style={{ flex: 1 }} />
            <button style={{
              padding: '8px 14px', borderRadius: 8,
              border: '1px solid var(--border)', background: 'var(--background)',
              fontSize: 12.5, fontFamily: SHIP_FONT, color: 'var(--destructive, oklch(0.55 0.2 25))',
              fontWeight: 600, cursor: 'pointer',
            }}>Deny</button>
            <button style={{
              padding: '8px 16px', borderRadius: 8,
              border: '1px solid var(--primary)', background: 'var(--primary)',
              color: 'var(--primary-foreground)',
              fontSize: 12.5, fontFamily: SHIP_FONT, fontWeight: 600, cursor: 'pointer',
              display: 'flex', alignItems: 'center', gap: 6,
            }}>
              Allow
              <Kbd>⏎</Kbd>
            </button>
          </div>
        </div>

        {/* tail */}
        <div style={{
          padding: '8px 18px',
          background: 'var(--card)',
          borderTop: '1px solid var(--border)',
          fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)',
          display: 'flex', alignItems: 'center', gap: 8,
        }}>
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 2l8 3v6c0 5-3.5 9-8 11-4.5-2-8-6-8-11V5l8-3z" /></svg>
          Decided in <b style={{ color: 'var(--foreground)' }}>hub</b> · never inside a space.
          <span style={{ marginLeft: 'auto' }}>policy: <u>fs:read prompts always</u></span>
        </div>
      </div>
    </div>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 3: Command approval — full-page detail view
// (high-risk shell ops get the room they deserve)
// ──────────────────────────────────────────────────────────────
function HubCommandApproval() {
  return (
    <WinFrame title="Ship" subtitle="hub · authority › request" style={{ width: AUTH_W, height: AUTH_H }}>
      <div style={{ flex: 1, display: 'flex', minHeight: 0 }}>
        <HubSidebar active="authority" />
        <main style={{ flex: 1, padding: '20px 28px 24px', overflow: 'hidden', display: 'flex', flexDirection: 'column', gap: 18 }}>
          {/* breadcrumb */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12, color: 'var(--muted-foreground)' }}>
            <span style={{ cursor: 'pointer' }}>Authority</span>
            <span>›</span>
            <span style={{ cursor: 'pointer' }}>Pending</span>
            <span>›</span>
            <SpaceMark size={14} accent="oklch(0.62 0.16 35)" char="S" />
            <span style={{ color: 'var(--foreground)', fontFamily: SHIP_MONO }}>Ship / Rust Lane / cmd-1</span>
            <span style={{ flex: 1 }} />
            <span style={{ fontFamily: SHIP_MONO, fontSize: 11 }}>waiting · 38s</span>
          </div>

          {/* header */}
          <div style={{
            display: 'flex', alignItems: 'flex-start', gap: 16,
            padding: '16px 18px', borderRadius: 12,
            background: 'color-mix(in oklch, var(--destructive, oklch(0.55 0.2 25)) 8%, var(--card))',
            border: '1px solid color-mix(in oklch, var(--destructive, oklch(0.55 0.2 25)) 30%, var(--border))',
          }}>
            <span style={{
              width: 38, height: 38, borderRadius: 10,
              background: 'color-mix(in oklch, var(--destructive, oklch(0.55 0.2 25)) 18%, transparent)',
              color: 'var(--destructive, oklch(0.55 0.2 25))',
              display: 'grid', placeItems: 'center',
              flexShrink: 0,
            }}>
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8"><path d="M12 9v4M12 17h.01M10.3 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" /></svg>
            </span>
            <div style={{ flex: 1, minWidth: 0 }}>
              <div style={{ fontFamily: SHIP_MONO, fontSize: 10, letterSpacing: '0.12em', textTransform: 'uppercase', color: 'var(--destructive, oklch(0.55 0.2 25))', fontWeight: 600 }}>
                High-risk command · rewrites remote history
              </div>
              <div style={{ fontSize: 19, fontWeight: 600, letterSpacing: '-0.01em', marginTop: 4, textWrap: 'balance' }}>
                Approve <code style={{ fontFamily: SHIP_MONO, fontSize: 17, background: 'color-mix(in oklch, var(--foreground) 6%, transparent)', padding: '1px 6px', borderRadius: 4 }}>git push --force-with-lease</code> in Ship?
              </div>
              <div style={{ marginTop: 6, fontSize: 12.5, color: 'var(--muted-foreground)' }}>
                Rust Lane wants to push <b style={{ color: 'var(--foreground)' }}>3 rebased commits</b> to <code style={{ fontFamily: SHIP_MONO }}>origin/main</code>.
                History on the remote will be rewritten.
              </div>
            </div>
            <RiskTag level="high" />
          </div>

          {/* split: diff/preview + reasoning */}
          <div style={{ flex: 1, display: 'grid', gridTemplateColumns: '1.6fr 1fr', gap: 14, minHeight: 0, overflow: 'hidden' }}>
            {/* left: command + diff */}
            <div style={{
              border: '1px solid var(--border)', borderRadius: 10,
              background: 'var(--card)',
              display: 'flex', flexDirection: 'column', minHeight: 0, overflow: 'hidden',
            }}>
              <div style={{ display: 'flex', alignItems: 'center', padding: '10px 14px', borderBottom: '1px solid var(--border)' }}>
                <MicroLabel>Command</MicroLabel>
                <span style={{ flex: 1 }} />
                <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>cwd ~/code/ship · shell: zsh</span>
              </div>
              {/* terminal block */}
              <div style={{
                padding: '14px 16px',
                background: '#16140f', color: '#e8e1cf',
                fontFamily: SHIP_MONO, fontSize: 13,
                lineHeight: 1.55,
                borderBottom: '1px solid var(--border)',
              }}>
                <div><span style={{ color: '#7c7565' }}>$</span> git push <span style={{ color: '#e0a16b' }}>--force-with-lease</span> origin <span style={{ color: '#b08fd9' }}>main</span></div>
                <div style={{ color: '#7c7565', marginTop: 6, fontSize: 11.5 }}>
                  to git@github.com:maddy/ship.git · expects origin/main = b3f1c2a
                </div>
              </div>

              {/* diff */}
              <div style={{ display: 'flex', alignItems: 'center', padding: '10px 14px', borderBottom: '1px solid var(--border)' }}>
                <MicroLabel>Range</MicroLabel>
                <span style={{ flex: 1 }} />
                <span style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>3 commits · 7 files · +148 −82</span>
              </div>
              <div style={{ flex: 1, overflow: 'auto', padding: '6px 14px 12px' }}>
                {[
                  ['e8a2c1f', 'refactor: extract sui-runtime entry', '+62 −24'],
                  ['4b71d09', 'fix: collapse duplicate theme tokens', '+71 −44'],
                  ['12fc8e3', 'chore: drop dead .sui sources', '+15 −14'],
                ].map(([sha, msg, stats]) => (
                  <div key={sha} style={{
                    display: 'grid', gridTemplateColumns: '70px 1fr auto',
                    gap: 12, alignItems: 'center',
                    padding: '7px 0',
                    borderBottom: '1px solid color-mix(in oklch, var(--border) 50%, transparent)',
                    fontSize: 12,
                  }}>
                    <code style={{ fontFamily: SHIP_MONO, color: 'var(--muted-foreground)', fontSize: 11.5 }}>{sha}</code>
                    <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{msg}</span>
                    <span style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)' }}>{stats}</span>
                  </div>
                ))}
                <div style={{
                  marginTop: 10, padding: '8px 10px', borderRadius: 7,
                  background: 'color-mix(in oklch, var(--warning, oklch(0.7 0.16 80)) 10%, transparent)',
                  border: '1px solid color-mix(in oklch, var(--warning, oklch(0.7 0.16 80)) 30%, var(--border))',
                  fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--foreground)',
                  display: 'flex', gap: 8, alignItems: 'flex-start',
                }}>
                  <span style={{ color: 'oklch(0.55 0.18 60)' }}>!</span>
                  <span>2 collaborators have pulled origin/main since last fetch. They'll need to <code>git reset --hard</code> after this lands.</span>
                </div>
              </div>
            </div>

            {/* right: agent's reasoning + actor */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: 12, minHeight: 0, overflow: 'hidden' }}>
              <div style={{
                padding: 14, borderRadius: 10,
                background: 'var(--card)', border: '1px solid var(--border)',
              }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                  <span style={{
                    width: 28, height: 28, borderRadius: 7,
                    background: 'color-mix(in oklch, oklch(0.6 0.18 280) 16%, transparent)',
                    color: 'oklch(0.6 0.18 280)',
                    display: 'grid', placeItems: 'center',
                  }}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8"><rect x="3" y="9" width="18" height="11" rx="2" /><path d="M12 5v4M9 5h6M8 14h.01M16 14h.01" /></svg>
                  </span>
                  <div>
                    <div style={{ fontSize: 13, fontWeight: 600 }}>Rust Lane</div>
                    <div style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>agent · in Ship</div>
                  </div>
                </div>
                <div style={{ marginTop: 12, fontSize: 12.5, lineHeight: 1.6, color: 'var(--foreground)' }}>
                  “Rebased <code style={{ fontFamily: SHIP_MONO, background: 'color-mix(in oklch, var(--foreground) 5%, transparent)', padding: '0 4px', borderRadius: 3 }}>main</code> onto <code style={{ fontFamily: SHIP_MONO, background: 'color-mix(in oklch, var(--foreground) 5%, transparent)', padding: '0 4px', borderRadius: 3 }}>origin/main</code> to drop 3 merge commits.
                  Tests green locally. Force-with-lease keeps us safe if origin moved.”
                </div>
              </div>

              <div style={{
                padding: 14, borderRadius: 10,
                background: 'var(--card)', border: '1px solid var(--border)',
                flex: 1, minHeight: 0, overflow: 'hidden', display: 'flex', flexDirection: 'column', gap: 8,
              }}>
                <MicroLabel>Touched paths</MicroLabel>
                <div style={{ flex: 1, overflow: 'auto', fontFamily: SHIP_MONO, fontSize: 11.5, color: 'var(--muted-foreground)', display: 'flex', flexDirection: 'column', gap: 3 }}>
                  {[
                    'packages/ui-components/src/tokens/themes.css',
                    'packages/ui-components/src/tokens/tokens.css',
                    'apps/site/src/styles/global.css',
                    'apps/site/public/api/assets/view-tokens.css',
                    'views/theme-studio/style.css',
                    'docs/ship-themes/SKILL.md',
                    'packages/components/primitives/src/index.ts',
                  ].map(p => (
                    <div key={p} style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
                      <span style={{ color: 'oklch(0.55 0.13 145)' }}>~</span>
                      <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{p}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {/* footer actions */}
          <div style={{
            display: 'flex', gap: 10, alignItems: 'center',
            padding: '12px 14px', borderRadius: 10,
            background: 'color-mix(in oklch, var(--card) 60%, var(--background))',
            border: '1px solid var(--border)',
          }}>
            <span style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)' }}>
              Auth required · <b style={{ color: 'var(--foreground)' }}>Touch ID</b> to confirm
            </span>
            <span style={{ flex: 1 }} />
            <button style={{
              padding: '8px 14px', borderRadius: 8,
              border: '1px solid var(--border)', background: 'var(--background)',
              fontSize: 12.5, fontFamily: SHIP_FONT, color: 'var(--destructive, oklch(0.55 0.2 25))',
              fontWeight: 600, cursor: 'pointer',
            }}>Deny</button>
            <button style={{
              padding: '8px 14px', borderRadius: 8,
              border: '1px solid var(--border)', background: 'var(--background)',
              fontSize: 12.5, fontFamily: SHIP_FONT, cursor: 'pointer',
            }}>Open in shell…</button>
            <button style={{
              padding: '8px 18px', borderRadius: 8,
              border: '1px solid var(--destructive, oklch(0.55 0.2 25))',
              background: 'var(--destructive, oklch(0.55 0.2 25))',
              color: 'white',
              fontSize: 12.5, fontFamily: SHIP_FONT, fontWeight: 600, cursor: 'pointer',
              display: 'flex', alignItems: 'center', gap: 8,
            }}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4"><path d="M9 11V7a3 3 0 0 1 6 0v4M5 11h14v10H5z" /></svg>
              Approve with Touch ID
            </button>
          </div>
        </main>
      </div>
    </WinFrame>
  );
}

// ──────────────────────────────────────────────────────────────
// Artboard 4: Identity / unlock — first-time-of-day hub auth + linked identities
// ──────────────────────────────────────────────────────────────
function HubIdentity() {
  return (
    <WinFrame title="Ship" subtitle="hub · locked" style={{ width: AUTH_W, height: AUTH_H }}>
      <div style={{
        flex: 1,
        display: 'grid', gridTemplateColumns: '1fr 1fr',
        background: 'linear-gradient(140deg, color-mix(in oklch, var(--primary) 7%, var(--background)) 0%, var(--background) 70%)',
        minHeight: 0,
      }}>
        {/* left — unlock */}
        <div style={{
          padding: '60px 64px',
          display: 'flex', flexDirection: 'column', justifyContent: 'center', gap: 28,
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
            <SpaceMark size={56} accent="var(--primary)" char="S" />
            <div>
              <div style={{ fontSize: 28, fontWeight: 600, letterSpacing: '-0.02em' }}>Ship</div>
              <div style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)', letterSpacing: '0.08em', textTransform: 'uppercase' }}>hub · locked</div>
            </div>
          </div>

          <div>
            <div style={{ fontSize: 17, fontWeight: 600, letterSpacing: '-0.01em', textWrap: 'balance' }}>
              Unlock the hub to grant capabilities and approve commands.
            </div>
            <div style={{ marginTop: 8, fontSize: 13, color: 'var(--muted-foreground)', lineHeight: 1.55, textWrap: 'pretty', maxWidth: 420 }}>
              Spaces stay running while the hub is locked. Read-only is fine.
              Anything elevated — fs writes, network, shell — pauses until you unlock.
            </div>
          </div>

          {/* user pod */}
          <div style={{
            display: 'flex', alignItems: 'center', gap: 14,
            padding: 14, borderRadius: 12,
            background: 'var(--card)', border: '1px solid var(--border)',
            maxWidth: 420,
          }}>
            <div style={{
              width: 44, height: 44, borderRadius: '50%',
              background: 'linear-gradient(135deg, oklch(0.65 0.16 60), oklch(0.55 0.18 35))',
              display: 'grid', placeItems: 'center',
              color: 'white', fontFamily: SHIP_FONT, fontWeight: 700, fontSize: 16,
            }}>M</div>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: 13.5, fontWeight: 600 }}>Maddy</div>
              <div style={{ fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)' }}>last unlock · 6h ago</div>
            </div>
            <button style={{
              padding: '10px 16px', borderRadius: 8,
              border: '1px solid var(--primary)', background: 'var(--primary)',
              color: 'var(--primary-foreground)',
              fontSize: 12.5, fontFamily: SHIP_FONT, fontWeight: 600, cursor: 'pointer',
              display: 'flex', alignItems: 'center', gap: 8,
            }}>
              <FingerprintIcon />
              Touch ID
            </button>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: 10, fontFamily: SHIP_MONO, fontSize: 11, color: 'var(--muted-foreground)' }}>
            <span style={{ flex: 1, height: 1, background: 'var(--border)' }} />
            or
            <span style={{ flex: 1, height: 1, background: 'var(--border)' }} />
          </div>

          <div style={{ display: 'flex', gap: 8 }}>
            <div style={{
              flex: 1, maxWidth: 420,
              padding: '10px 14px',
              border: '1px solid var(--border)', borderRadius: 8,
              background: 'var(--background)',
              fontFamily: SHIP_MONO, fontSize: 12.5,
              color: 'var(--muted-foreground)',
              display: 'flex', alignItems: 'center', gap: 8,
            }}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8"><circle cx="12" cy="12" r="9" /><path d="M12 3a14 14 0 0 1 0 18M12 3a14 14 0 0 0 0 18M3 12h18" /></svg>
              Use passphrase…
              <span style={{ flex: 1 }} />
              <Kbd>⌘</Kbd><Kbd>↵</Kbd>
            </div>
          </div>
        </div>

        {/* right — linked identities (what the hub speaks for) */}
        <div style={{
          padding: 32,
          background: 'color-mix(in oklch, var(--card) 60%, var(--background))',
          borderLeft: '1px solid var(--border)',
          display: 'flex', flexDirection: 'column', gap: 16, minHeight: 0,
        }}>
          <div>
            <MicroLabel>Linked identities</MicroLabel>
            <div style={{ fontSize: 14, fontWeight: 600, marginTop: 6, letterSpacing: '-0.005em' }}>
              The hub holds your keys. Spaces never see them.
            </div>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {IDENTITIES.map(id => (
              <div key={id.svc} style={{
                display: 'flex', alignItems: 'center', gap: 12,
                padding: '10px 14px', borderRadius: 9,
                background: 'var(--card)', border: '1px solid var(--border)',
              }}>
                <ServiceMark name={id.svc} />
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: 12.5, fontWeight: 600 }}>{id.svc}</div>
                  <div style={{ fontFamily: SHIP_MONO, fontSize: 10.5, color: 'var(--muted-foreground)' }}>{id.handle}</div>
                </div>
                {id.tone === 'ok' ? (
                  <span style={{
                    display: 'inline-flex', alignItems: 'center', gap: 5,
                    fontFamily: SHIP_MONO, fontSize: 10.5,
                    color: 'var(--success, oklch(0.55 0.13 145))',
                  }}><Dot tone="ok" size={6} />linked</span>
                ) : (
                  <span style={{
                    fontFamily: SHIP_MONO, fontSize: 10.5,
                    color: 'var(--muted-foreground)',
                    cursor: 'pointer',
                    padding: '3px 8px', borderRadius: 999,
                    border: '1px solid var(--border)',
                  }}>link…</span>
                )}
              </div>
            ))}
          </div>

          <div style={{ flex: 1 }} />

          <div style={{
            padding: '12px 14px', borderRadius: 9,
            border: '1px dashed var(--border)',
            background: 'transparent',
            fontSize: 11.5, color: 'var(--muted-foreground)', lineHeight: 1.55,
          }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
              <ShieldIcon />
              <b style={{ color: 'var(--foreground)', fontWeight: 600, fontFamily: SHIP_FONT, fontSize: 12 }}>How this works</b>
            </div>
            Spaces ask the hub for tokens via short-lived handles. The hub mints a scoped grant
            (e.g. <code style={{ fontFamily: SHIP_MONO, fontSize: 11 }}>oauth:mastodon.social#post</code>), tags it to the space, and forgets
            the key. Revoking here cuts every space at once.
          </div>
        </div>
      </div>
    </WinFrame>
  );
}

function FingerprintIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round">
      <path d="M12 11v4M9 12v2a3 3 0 0 0 6 0M6 10a6 6 0 0 1 12 0M4 12c0-4.4 3.6-8 8-8a8 8 0 0 1 8 8M8 16a4 4 0 0 0 8 0v-3" />
    </svg>
  );
}

function ServiceMark({ name }) {
  const colors = {
    GitHub: ['#1f1f1f', '#fff'],
    Mastodon: ['#6364ff', '#fff'],
    OpenAI: ['#0f1e1c', '#a0e3d2'],
    Anthropic: ['#cc785c', '#fff'],
    Linear: ['#5e6ad2', '#fff'],
  };
  const [bg, fg] = colors[name] || ['#666', '#fff'];
  return (
    <div style={{
      width: 28, height: 28, borderRadius: 6,
      background: bg, color: fg,
      display: 'grid', placeItems: 'center',
      fontFamily: SHIP_MONO, fontWeight: 700, fontSize: 13,
      flexShrink: 0,
    }}>{name[0]}</div>
  );
}

Object.assign(window, { HubAuthority, HubCapabilityRequest, HubCommandApproval, HubIdentity, AUTH_W, AUTH_H });
