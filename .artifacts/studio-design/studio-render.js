/* ===== Studio renderer =====
 * Reads location.hash (#<domain>[/<tab>]) and re-renders rail/browse/detail.
 * Per-domain templates kept as plain string functions for visual readability.
 */

// ─── utilities ─────────────────────────────────────────────────────

const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => (
  { '&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;' }[c]
));

const provBlob = (p) => `<span class="lib-prov" data-prov="${esc(p)}">${esc(p[0].toUpperCase())}</span>`;
const provBlobs = (arr) => (arr || []).map(provBlob).join('');

const statusChip = (status) => {
  if (!status || status === 'idle') return '';
  const tone = status === 'active' ? 'success'
             : status === 'running' ? 'primary'
             : status === 'connected' ? 'success'
             : status === 'disabled' ? 'muted'
             : 'muted';
  const pulse = (status === 'active' || status === 'running' || status === 'connected') ? 'pulse' : '';
  return `<sui-chip tone="${tone}" variant="soft" size="sm" class="lib-row-chip">
    <sui-status-dot slot="leading" tone="${tone}" size="sm" ${pulse}></sui-status-dot>
    ${esc(status)}
  </sui-chip>`;
};

// Domain header text
const DOMAIN_META = {
  agents:      { title: 'Agents',      loc: '.ship/agents/',         newLabel: 'New agent' },
  skills:      { title: 'Skills',      loc: 'docs/<skill>/SKILL.md', newLabel: 'New skill' },
  mcp:         { title: 'MCP servers', loc: '.ship/mcp.jsonc',       newLabel: 'New server' },
  pipelines:   { title: 'Pipelines',   loc: '.ship/pipelines/',      newLabel: 'New pipeline' },
  themes:      { title: 'Themes',      loc: 'theme packs',           newLabel: 'New theme' },
  components:  { title: 'Components',  loc: 'packs/',                newLabel: 'New component' },
  permissions: { title: 'Permissions', loc: '.ship/permissions.jsonc', newLabel: 'New preset' },
};

const DOMAIN_FILTERS = {
  agents:      [['All','8'], ['Project','5'], ['User','3']],
  skills:      [['All','14'], ['Public','11'], ['Internal','3'], ['Used','12']],
  mcp:         [['All','5'], ['Project','2'], ['Global','3'], ['Enabled','4']],
  pipelines:   [['All','6'], ['Project','6'], ['Auto-merge','2']],
  themes:      [['All','7'], ['Light','3'], ['Dark','3'], ['Special','1']],
  components:  [['All','80'], ['Primitives','80'], ['Charts','8'], ['Chrome','22']],
  permissions: [['All','4'], ['In use','3']],
};

// ─── rail ──────────────────────────────────────────────────────────

function renderRail(activeDomain) {
  const html = DOMAINS.map(group => `
    <div class="lib-rail-group">
      <div class="lib-rail-h">${esc(group.group)}</div>
      ${group.items.map(d => `
        <button class="lib-rail-item ${d.id === activeDomain ? 'is-active' : ''}"
                data-target="${d.id}" type="button">
          <span class="lib-rail-glyph hue-${d.hue}">${esc(d.glyph)}</span>
          <span class="lib-rail-label">${esc(d.label)}</span>
          <span class="lib-rail-count">${d.count}</span>
        </button>
      `).join('')}
    </div>
  `).join('');

  document.getElementById('rail').innerHTML = html + `
    <div class="lib-rail-foot">
      <div class="lib-rail-foot-label">Workspace</div>
      <div class="lib-rail-foot-val">
        <span class="lib-rail-foot-dot"></span>
        ship-monorepo
      </div>
    </div>
  `;

  document.querySelectorAll('.lib-rail-item').forEach(b => {
    b.addEventListener('click', () => {
      location.hash = b.getAttribute('data-target');
    });
  });
}

// ─── browse pane ──────────────────────────────────────────────────

function renderBrowse(domain, items, selectedId) {
  const meta = DOMAIN_META[domain];
  const filters = DOMAIN_FILTERS[domain] || [];

  const filterPills = filters.map(([label, count], i) => `
    <button class="lib-pill ${i === 0 ? 'is-active' : ''}" type="button">
      ${esc(label)} <span>${esc(count)}</span>
    </button>
  `).join('');

  let listHtml;
  if (domain === 'components') {
    // Components are grouped, not flat
    listHtml = COMPONENT_GROUPS.map(g => `
      <div class="lib-comp-group">
        <div class="lib-comp-group-h">${esc(g.group)}<span>${g.items.length}</span></div>
        ${g.items.map(tag => `
          <a class="lib-comp-row ${tag === selectedId ? 'is-selected' : ''}"
             data-id="${esc(tag)}" href="#components/${esc(tag)}">
            <code>${esc(tag)}</code>
          </a>
        `).join('')}
      </div>
    `).join('');
  } else {
    listHtml = items.map(item => (ROWS[domain] || (() => ''))(item, selectedId)).join('');
  }

  const subParts = [`<span><strong>${items.length || (domain === 'components' ? 80 : 0)}</strong> total</span>`];
  subParts.push(`<span class="lib-dot"></span><span><code>${esc(meta.loc)}</code></span>`);

  document.getElementById('browse').innerHTML = `
    <header class="lib-browse-h">
      <div class="lib-browse-titles">
        <h2 class="lib-browse-title">${esc(meta.title)}</h2>
        <p class="lib-browse-sub">${subParts.join('')}</p>
      </div>
      <sui-button tone="primary" variant="solid" size="sm">
        <sui-icon slot="leading" name="plus" size="xs"></sui-icon>
        New
      </sui-button>
    </header>
    <div class="lib-browse-filters">
      <div class="lib-search">
        <sui-icon name="search" size="sm" tone="muted"></sui-icon>
        <input type="search" placeholder="Filter ${esc(meta.title.toLowerCase())}…" />
      </div>
      <div class="lib-pills" role="group">${filterPills}</div>
    </div>
    <div class="lib-list ${domain === 'components' ? 'is-grouped' : ''}" role="list">
      ${listHtml}
    </div>
  `;

  document.querySelectorAll('#browse .lib-pill').forEach(p => {
    p.addEventListener('click', () => {
      document.querySelectorAll('#browse .lib-pill').forEach(x => x.classList.remove('is-active'));
      p.classList.add('is-active');
    });
  });
  document.querySelectorAll('#browse .lib-row, #browse .lib-comp-row').forEach(r => {
    r.addEventListener('click', (e) => {
      e.preventDefault();
      const id = r.getAttribute('data-id');
      location.hash = `${domain}/${id}`;
    });
  });
}

// ─── per-domain rows ──────────────────────────────────────────────

const ROWS = {

  agents: (a, sel) => `
    <a class="lib-row ${a.id === sel ? 'is-selected' : ''}" role="listitem"
       data-id="${esc(a.id)}" href="#agents/${esc(a.id)}">
      <span class="lib-row-glyph hue-${a.hue}">${esc(a.letter)}</span>
      <div class="lib-row-body">
        <div class="lib-row-line">
          <strong class="lib-row-name">${esc(a.id)}</strong>
          <span class="lib-row-ver">v ${esc(a.version)}</span>
        </div>
        <p class="lib-row-desc">${esc(a.description)}</p>
        <div class="lib-row-meta">
          <span>${a.skills.length} skill${a.skills.length === 1 ? '' : 's'}</span><span class="lib-row-sep">·</span>
          <span>${a.mcp.length} MCP</span><span class="lib-row-sep">·</span>
          <span>${a.rules_count} rule${a.rules_count === 1 ? '' : 's'}</span>
          ${a.tags[0] ? `<span class="lib-row-sep">·</span><span class="lib-row-tag">${esc(a.tags[0])}</span>` : ''}
        </div>
      </div>
      <div class="lib-row-side">${provBlobs(a.providers)}</div>
    </a>
  `,

  skills: (s, sel) => `
    <a class="lib-row ${s.id === sel ? 'is-selected' : ''}" role="listitem"
       data-id="${esc(s.id)}" href="#skills/${esc(s.id)}">
      <span class="lib-row-glyph hue-${s.hue}">${esc(s.letter)}</span>
      <div class="lib-row-body">
        <div class="lib-row-line">
          <strong class="lib-row-name">${esc(s.id)}</strong>
          <span class="lib-row-ver">v ${esc(s.version)}</span>
          ${s.audience !== 'public' ? `<sui-chip tone="muted" variant="outline" size="sm" class="lib-row-chip">${esc(s.audience)}</sui-chip>` : ''}
        </div>
        <p class="lib-row-desc">${esc(s.description)}</p>
        <div class="lib-row-meta">
          ${s.tags.map(t => `<span class="lib-row-tag">${esc(t)}</span>`).join(' ')}
          <span class="lib-row-sep">·</span>
          <span>used by ${s.used_by}</span>
        </div>
      </div>
    </a>
  `,

  mcp: (m, sel) => `
    <a class="lib-row ${m.id === sel ? 'is-selected' : ''}" role="listitem"
       data-id="${esc(m.id)}" href="#mcp/${esc(m.id)}">
      <span class="lib-row-glyph hue-${m.hue}">${esc(m.letter)}</span>
      <div class="lib-row-body">
        <div class="lib-row-line">
          <strong class="lib-row-name">${esc(m.id)}</strong>
          <span class="lib-tport">${esc(m.transport)}</span>
          ${!m.enabled ? `<sui-chip tone="muted" variant="outline" size="sm" class="lib-row-chip">disabled</sui-chip>` : ''}
        </div>
        <p class="lib-row-desc">${esc(m.description)}</p>
        <div class="lib-row-meta">
          <span>${esc(m.scope)}</span><span class="lib-row-sep">·</span>
          <span>${m.tool_count} tool${m.tool_count === 1 ? '' : 's'}</span><span class="lib-row-sep">·</span>
          <span>used by ${m.used_by} agent${m.used_by === 1 ? '' : 's'}</span>
        </div>
      </div>
    </a>
  `,

  pipelines: (p, sel) => `
    <a class="lib-row ${p.id === sel ? 'is-selected' : ''}" role="listitem"
       data-id="${esc(p.id)}" href="#pipelines/${esc(p.id)}">
      <span class="lib-row-glyph hue-${p.hue}">${esc(p.letter)}</span>
      <div class="lib-row-body">
        <div class="lib-row-line">
          <strong class="lib-row-name">${esc(p.id)}</strong>
          ${p.auto_merge ? `<sui-chip tone="muted" variant="outline" size="sm" class="lib-row-chip">auto-merge</sui-chip>` : ''}
        </div>
        <div class="lib-mini-dag">
          ${p.steps.map((s, i) => `
            ${i > 0 ? '<span class="lib-mini-edge"></span>' : ''}
            <span class="lib-mini-step" data-type="${esc(s.type)}" title="${esc(s.id)} (${esc(s.type)})"></span>
          `).join('')}
        </div>
        <p class="lib-row-desc">${esc(p.description)}</p>
        <div class="lib-row-meta">
          <span>${p.steps.length} step${p.steps.length === 1 ? '' : 's'}</span>
          <span class="lib-row-sep">·</span>
          <span>${p.vars_count} var${p.vars_count === 1 ? '' : 's'}</span>
        </div>
      </div>
    </a>
  `,

  themes: (t, sel) => `
    <a class="lib-row lib-row-theme ${t.id === sel ? 'is-selected' : ''}" role="listitem"
       data-id="${esc(t.id)}" href="#themes/${esc(t.id)}">
      <span class="lib-row-swatch">
        ${t.swatches.map(c => `<span style="background:${esc(c)}"></span>`).join('')}
      </span>
      <div class="lib-row-body">
        <div class="lib-row-line">
          <strong class="lib-row-name">${esc(t.id)}</strong>
          <sui-chip tone="muted" variant="outline" size="sm" class="lib-row-chip">${esc(t.kind)}</sui-chip>
        </div>
        <p class="lib-row-desc">${esc(t.description)}</p>
        <div class="lib-row-meta">
          <span>${t.overrides} token override${t.overrides === 1 ? '' : 's'}</span>
        </div>
      </div>
    </a>
  `,

  permissions: (p, sel) => `
    <a class="lib-row ${p.id === sel ? 'is-selected' : ''}" role="listitem"
       data-id="${esc(p.id)}" href="#permissions/${esc(p.id)}">
      <span class="lib-row-glyph hue-${p.hue}">${esc(p.letter)}</span>
      <div class="lib-row-body">
        <div class="lib-row-line">
          <strong class="lib-row-name">${esc(p.id)}</strong>
          ${p.used_by === 0 ? `<sui-chip tone="muted" variant="outline" size="sm" class="lib-row-chip">unused</sui-chip>` : ''}
        </div>
        <p class="lib-row-desc">${esc(p.description)}</p>
        <div class="lib-row-meta lib-perm-counts">
          <span class="lib-perm-c is-allow">allow ${p.allow.length}</span>
          <span class="lib-perm-c is-ask">ask ${p.ask.length}</span>
          <span class="lib-perm-c is-deny">deny ${p.deny.length}</span>
          <span class="lib-row-sep">·</span>
          <span>used by ${p.used_by}</span>
        </div>
      </div>
    </a>
  `,
};

// ─── detail pane ──────────────────────────────────────────────────

function renderDetail(domain, item, tab) {
  if (!item) {
    document.getElementById('detail').innerHTML = `
      <div class="lib-empty">
        <p>Pick a ${esc(DOMAIN_META[domain]?.title.toLowerCase() || 'thing')} to inspect.</p>
      </div>`;
    return;
  }
  const renderer = DETAILS[domain]?.[tab] || DETAILS[domain]?.overview || (() => '<div class="lib-empty"><p>Not yet designed.</p></div>');
  document.getElementById('detail').innerHTML = renderer(item, tab);

  // Tab navigation
  document.querySelectorAll('#detail .lib-tab').forEach(t => {
    t.addEventListener('click', () => {
      const tabId = t.getAttribute('data-tab');
      location.hash = `${domain}/${item.id}/${tabId}`;
    });
  });
}

// Generic detail header
function detailHeader(domain, item, tabs, activeTab) {
  return `
    <header class="lib-detail-top">
      <div class="lib-detail-id">
        <span class="lib-detail-glyph hue-${item.hue}">${esc(item.letter || item.id[0].toUpperCase())}</span>
        <div class="lib-detail-id-text">
          <div class="lib-detail-title">${esc(item.name || item.id)}</div>
          <div class="lib-detail-path">
            <code>${esc(item.id)}</code>
            <span class="lib-dot"></span>
            <span class="lib-detail-file">${esc(item.path || pathFor(domain, item))}</span>
            ${item.version ? `<span class="lib-dot"></span><span>v ${esc(item.version)}</span>` : ''}
          </div>
        </div>
      </div>
      <div class="lib-detail-actions">
        <sui-button variant="ghost" size="sm">
          <sui-icon slot="leading" name="copy" size="xs"></sui-icon>
          Duplicate
        </sui-button>
        <sui-button tone="primary" variant="solid" size="sm">
          <sui-icon slot="leading" name="pencil" size="xs"></sui-icon>
          Edit
        </sui-button>
      </div>
    </header>
    <nav class="lib-tabs" role="tablist">
      ${tabs.map(([id, label, count]) => `
        <button class="lib-tab ${id === activeTab ? 'is-active' : ''}" role="tab" data-tab="${esc(id)}">
          ${esc(label)}${count != null ? `<span>${esc(count)}</span>` : ''}
        </button>
      `).join('')}
    </nav>
  `;
}

function pathFor(domain, item) {
  if (domain === 'agents')      return `.ship/agents/${item.id}.jsonc`;
  if (domain === 'skills')      return `docs/${item.id}/SKILL.md`;
  if (domain === 'mcp')         return `.ship/mcp.jsonc#${item.id}`;
  if (domain === 'pipelines')   return `.ship/pipelines/${item.id}.yaml`;
  if (domain === 'themes')      return `packs/themes/${item.id}.css`;
  if (domain === 'components')  return `packs/primitives/${item.id || item.tag}/${item.id || item.tag}.sui`;
  if (domain === 'permissions') return `.ship/permissions.jsonc#${item.id}`;
  return '';
}

// ─── per-domain detail bodies ─────────────────────────────────────

const DETAILS = {

  agents: {
    overview: (a) => detailHeader('agents', a, [
      ['overview','Overview'], ['skills','Skills', a.skills.length],
      ['mcp','MCP', a.mcp.length], ['permissions','Permissions'],
      ['wire','Wire'],
    ], 'overview') + `
      <div class="lib-detail-body">
        <div class="lib-detail-grid">

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h"><h3>Description</h3></div>
            <p class="lib-card-prose">${esc(a.description)}</p>
            <div class="lib-tags">
              ${a.tags.map(t => `<sui-chip tone="muted" variant="soft" size="sm">${esc(t)}</sui-chip>`).join('')}
            </div>
          </section>

          <section class="lib-card">
            <div class="lib-card-h"><h3>Identity</h3><sui-button variant="ghost" size="sm">Edit</sui-button></div>
            <dl class="lib-kv">
              <dt>Role</dt>          <dd>${esc(a.role)}</dd>
              <dt>Tools preset</dt>  <dd><code>${esc(a.role === 'lead' ? 'orchestrator' : 'worker')}</code></dd>
              <dt>Source</dt>        <dd>${esc(a.source)}</dd>
              <dt>Version</dt>       <dd>v ${esc(a.version)}</dd>
            </dl>
          </section>

          <section class="lib-card">
            <div class="lib-card-h"><h3>Compiles to</h3><a href="#agents/${esc(a.id)}/wire" class="lib-card-h-link">View wire →</a></div>
            <div class="lib-prov-grid">
              ${['anthropic','openai','gemini'].map(p => `
                <div class="lib-prov-card ${a.providers.includes(p) ? '' : 'is-disabled'}">
                  ${provBlob(p)}
                  <span>${p}</span>
                </div>
              `).join('')}
            </div>
          </section>

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h">
              <h3>Skills</h3>
              <span class="lib-card-h-sub">${a.skills.length} activated · referenced by id</span>
              <sui-button variant="ghost" size="sm">Manage</sui-button>
            </div>
            <div class="lib-skill-grid">
              ${a.skills.map(sid => {
                const sk = SKILLS.find(s => s.id === sid) || { id: sid, version: '?', description: '', tags: [] };
                return `
                  <a class="lib-skill" href="#skills/${esc(sk.id)}">
                    <div class="lib-skill-head">
                      <strong>${esc(sk.id)}</strong>
                      <sui-chip tone="muted" variant="outline" size="sm">v ${esc(sk.version)}</sui-chip>
                    </div>
                    <p>${esc(sk.description || '—')}</p>
                  </a>
                `;
              }).join('')}
            </div>
          </section>

          <section class="lib-card">
            <div class="lib-card-h"><h3>MCP servers</h3><sui-button variant="ghost" size="sm">Configure</sui-button></div>
            <div class="lib-mcp-list">
              ${a.mcp.length === 0 ? '<p class="lib-empty-inline">No MCP servers attached.</p>' :
                a.mcp.map(mid => {
                  const m = MCPS.find(x => x.id === mid) || { id: mid, transport: 'stdio', scope: 'project' };
                  return `
                    <a class="lib-mcp" href="#mcp/${esc(m.id)}">
                      <span class="lib-mcp-glyph">${m.transport === 'http' ? '⎈' : m.transport === 'sse' ? '⟂' : '⌘'}</span>
                      <div class="lib-mcp-body">
                        <strong>${esc(m.id)}</strong>
                        <div class="lib-mcp-meta"><code>${esc(m.transport)}</code> · scope: ${esc(m.scope)}</div>
                      </div>
                    </a>
                  `;
                }).join('')
              }
            </div>
          </section>

          <section class="lib-card">
            <div class="lib-card-h"><h3>Permissions</h3><a href="#agents/${esc(a.id)}/permissions" class="lib-card-h-link">Edit →</a></div>
            <div class="lib-perm-summary">
              <sui-chip tone="primary" variant="soft" size="sm">${esc(a.preset)}</sui-chip>
              <div class="lib-perm-row"><span class="lib-perm-c is-allow">allow</span><code>Read, Edit, Bash(cargo *), Bash(git *)</code></div>
              <div class="lib-perm-row"><span class="lib-perm-c is-ask">ask</span><code>Write, Bash(rm *)</code></div>
              <div class="lib-perm-row"><span class="lib-perm-c is-deny">deny</span><code>mcp__github__delete*</code></div>
            </div>
          </section>

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h">
              <h3>Validity</h3>
              <span class="lib-card-h-sub">local definition checks — no runtime traffic</span>
            </div>
            <ul class="lib-check-list">
              <li class="is-ok"><span class="lib-check-icon">✓</span> JSONC parses cleanly</li>
              <li class="is-ok"><span class="lib-check-icon">✓</span> All ${a.skills.length} skill refs resolve</li>
              <li class="is-ok"><span class="lib-check-icon">✓</span> All ${a.mcp.length} MCP refs resolve</li>
              <li class="is-ok"><span class="lib-check-icon">✓</span> Permission preset <code>${esc(a.preset)}</code> exists</li>
              <li class="is-ok"><span class="lib-check-icon">✓</span> ${a.providers.length} provider${a.providers.length === 1 ? '' : 's'} configured</li>
            </ul>
          </section>

        </div>
      </div>
    `,

    permissions: (a) => detailHeader('agents', a, [
      ['overview','Overview'], ['skills','Skills', a.skills.length],
      ['mcp','MCP', a.mcp.length], ['permissions','Permissions'],
      ['wire','Wire'],
    ], 'permissions') + `
      <div class="lib-detail-body">
        <div class="lib-perm-page">

          <div class="lib-perm-preset-bar">
            <div class="lib-perm-preset-left">
              <span class="lib-card-h-sub">Inherits preset</span>
              <sui-chip tone="primary" variant="soft" size="sm">${esc(a.preset)}</sui-chip>
              <a href="#permissions/${esc(a.preset)}" class="lib-card-h-link">View preset →</a>
            </div>
            <div class="lib-perm-preset-right">
              <span class="lib-card-h-sub">Effective resolution</span>
              <code>preset + agent overrides</code>
            </div>
          </div>

          <div class="lib-perm-matrix">

            <div class="lib-perm-col is-allow">
              <div class="lib-perm-col-h">
                <span class="lib-perm-c is-allow">allow</span>
                <span class="lib-perm-col-count">4</span>
                <sui-button variant="ghost" size="sm">
                  <sui-icon slot="leading" name="plus" size="xs"></sui-icon>Add
                </sui-button>
              </div>
              <ul class="lib-perm-list">
                <li><code>Read</code><span class="lib-perm-src">preset</span></li>
                <li><code>Edit</code><span class="lib-perm-src">preset</span></li>
                <li><code>Bash(cargo *)</code><span class="lib-perm-src">override</span></li>
                <li><code>Bash(git *)</code><span class="lib-perm-src">override</span></li>
              </ul>
            </div>

            <div class="lib-perm-col is-ask">
              <div class="lib-perm-col-h">
                <span class="lib-perm-c is-ask">ask</span>
                <span class="lib-perm-col-count">2</span>
                <sui-button variant="ghost" size="sm">
                  <sui-icon slot="leading" name="plus" size="xs"></sui-icon>Add
                </sui-button>
              </div>
              <ul class="lib-perm-list">
                <li><code>Write</code><span class="lib-perm-src">override</span></li>
                <li><code>Bash(rm *)</code><span class="lib-perm-src">preset</span></li>
              </ul>
            </div>

            <div class="lib-perm-col is-deny">
              <div class="lib-perm-col-h">
                <span class="lib-perm-c is-deny">deny</span>
                <span class="lib-perm-col-count">1</span>
                <sui-button variant="ghost" size="sm">
                  <sui-icon slot="leading" name="plus" size="xs"></sui-icon>Add
                </sui-button>
              </div>
              <ul class="lib-perm-list">
                <li><code>mcp__github__delete*</code><span class="lib-perm-src">override</span></li>
              </ul>
            </div>
          </div>

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h"><h3>Test a tool pattern</h3><span class="lib-card-h-sub">Dry-run the resolver against a tool name to see which bucket it falls into.</span></div>
            <div class="lib-perm-tester">
              <input class="lib-perm-tester-input" placeholder="Bash(cargo test)" value="Bash(cargo test)" />
              <span class="lib-perm-tester-arrow">→</span>
              <span class="lib-perm-c is-allow">allow</span>
              <span class="lib-perm-tester-rule">matched <code>Bash(cargo *)</code> in agent overrides</span>
            </div>
          </section>

        </div>
      </div>
    `,

    wire: (a) => {
      const wire = WIRE[a.id];
      const harnesses = wire ? Object.keys(wire) : [];
      const active = harnesses[0];
      const w = active ? wire[active] : null;
      return detailHeader('agents', a, [
        ['overview','Overview'], ['skills','Skills', a.skills.length],
        ['mcp','MCP', a.mcp.length], ['permissions','Permissions'],
        ['wire','Wire'],
      ], 'wire') + `
        <div class="lib-detail-body">
          <div class="lib-wire-page">

            <div class="lib-wire-bar">
              <div class="lib-wire-bar-l">
                <span class="lib-card-h-sub">Compiled output</span>
                <div class="lib-wire-harnesses" role="tablist">
                  ${['anthropic','openai','gemini','cursor','opencode'].map(h => `
                    <button class="lib-wire-harness ${h === active ? 'is-active' : ''} ${a.providers.includes(h) ? '' : 'is-disabled'}"
                      type="button" data-harness="${esc(h)}" ${a.providers.includes(h) ? '' : 'disabled'}>
                      ${provBlob(h)}
                      <span>${esc(h)}</span>
                    </button>
                  `).join('')}
                </div>
              </div>
              <div class="lib-wire-bar-r">
                <span class="lib-card-h-sub">SDK</span>
                <code class="lib-wire-sdk">ship.agents.wire({ id: "${esc(a.id)}" })</code>
              </div>
            </div>

            ${!w ? `<p class="lib-empty-inline" style="padding: var(--space-4)">No wire output available for this provider.</p>` : `
              <div class="lib-wire-grid">

                <section class="lib-card">
                  <div class="lib-card-h"><h3>Destination</h3></div>
                  <dl class="lib-kv">
                    <dt>File</dt>          <dd><code class="lib-mono-break">${esc(w.file)}</code></dd>
                    <dt>Model</dt>         <dd><code>${esc(w.model)}</code></dd>
                    <dt>Tools preset</dt>  <dd><code>${esc(w.tools_preset)}</code></dd>
                    <dt>Env keys</dt>      <dd>${Object.keys(w.env || {}).length}</dd>
                  </dl>
                </section>

                <section class="lib-card">
                  <div class="lib-card-h"><h3>Tools</h3><span class="lib-card-h-sub">${w.tools.length} resolved</span></div>
                  <ul class="lib-wire-tools">
                    ${w.tools.map(t => `<li><code>${esc(t)}</code></li>`).join('')}
                  </ul>
                </section>

                <section class="lib-card">
                  <div class="lib-card-h"><h3>MCP servers</h3><span class="lib-card-h-sub">${w.mcp_servers.length} attached</span></div>
                  <ul class="lib-wire-tools">
                    ${w.mcp_servers.map(m => `<li><code>${esc(m)}</code></li>`).join('')}
                  </ul>
                </section>

                <section class="lib-card lib-card-wide">
                  <div class="lib-card-h"><h3>System message</h3><span class="lib-card-h-sub">rules + inline prompt, post-template</span><sui-button variant="ghost" size="sm">Copy</sui-button></div>
                  <pre class="lib-wire-sys">${esc(w.system)}</pre>
                </section>

                ${Object.keys(w.env || {}).length > 0 ? `
                  <section class="lib-card lib-card-wide">
                    <div class="lib-card-h"><h3>Env</h3></div>
                    <dl class="lib-kv">
                      ${Object.entries(w.env).map(([k, v]) => `<dt><code>${esc(k)}</code></dt><dd><code>${esc(v)}</code></dd>`).join('')}
                    </dl>
                  </section>
                ` : ''}

              </div>
            `}

          </div>
        </div>
      `;
    },
  },

  skills: {
    overview: (s) => detailHeader('skills', s, [
      ['overview','Overview'], ['invoke','Invoke'], ['body','Body'], ['tools','Tools'],
      ['used','Used by', s.used_by],
    ], 'overview') + `
      <div class="lib-detail-body">
        <div class="lib-detail-grid">

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h"><h3>Description</h3></div>
            <p class="lib-card-prose">${esc(s.description)}</p>
            <div class="lib-tags">
              ${s.tags.map(t => `<sui-chip tone="muted" variant="soft" size="sm">${esc(t)}</sui-chip>`).join('')}
            </div>
          </section>

          <section class="lib-card">
            <div class="lib-card-h"><h3>Frontmatter</h3></div>
            <dl class="lib-kv">
              <dt>name</dt>         <dd><code>${esc(s.id)}</code></dd>
              <dt>stable-id</dt>    <dd><code>${esc(s.id)}</code></dd>
              <dt>version</dt>      <dd>${esc(s.version)}</dd>
              <dt>audience</dt>     <dd>${esc(s.audience)}</dd>
              <dt>authors</dt>      <dd>${s.authors.map(a => `<code>${esc(a)}</code>`).join(', ')}</dd>
              <dt>compatibility</dt><dd>${esc(s.compat)}</dd>
            </dl>
          </section>

          <section class="lib-card">
            <div class="lib-card-h"><h3>Artifacts</h3><span class="lib-card-h-sub">what this skill produces</span></div>
            <div class="lib-tags">
              ${s.artifacts.map(a => `<sui-chip tone="info" variant="soft" size="sm"><code>${esc(a)}</code></sui-chip>`).join('')}
            </div>
            <div class="lib-empty-inline" style="margin-top: 12px">Used by ${s.used_by} agent${s.used_by === 1 ? '' : 's'}.</div>
          </section>

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h"><h3>SKILL.md preview</h3><sui-button variant="ghost" size="sm">Open editor</sui-button></div>
            <pre class="lib-md-preview">---
name: ${esc(s.id)}
stable-id: ${esc(s.id)}
description: ${esc(s.description)}
tags: [${s.tags.map(t => esc(t)).join(', ')}]
authors: [${s.authors.map(a => esc(a)).join(', ')}]
version: ${esc(s.version)}
artifacts: [${s.artifacts.map(a => esc(a)).join(', ')}]
---

# ${esc(s.id)}

${esc(s.description)}

## When to use

Trigger this skill when the planner detects an opportunity to <em>${esc(s.tags[0] || 'apply')}</em>.
Surface it via the launcher under <strong>${esc(s.tags[0] || 'workflow')}</strong>.

## Steps

1. Read the surrounding context (file, diff, or selection).
2. Produce a minimal artifact in the declared format (<code>${esc(s.artifacts[0] || 'markdown')}</code>).
3. Attach to the active run scope; promote to project on user request.</pre>
          </section>

        </div>
      </div>
    `,

    invoke: (s) => {
      const hist = INVOKE_HISTORY[s.id] || [];
      const latest = hist[0];
      return detailHeader('skills', s, [
        ['overview','Overview'], ['invoke','Invoke'], ['body','Body'], ['tools','Tools'],
        ['used','Used by', s.used_by],
      ], 'invoke') + `
        <div class="lib-detail-body">
          <div class="lib-invoke-page">

            <section class="lib-invoke-form lib-card">
              <div class="lib-card-h">
                <h3>Invoke</h3>
                <span class="lib-card-h-sub">ship.skills.invoke({ id: "${esc(s.id)}", vars, workspace_id?, session_id? })</span>
              </div>

              <div class="lib-invoke-section">
                <label class="lib-invoke-label">Vars</label>
                <div class="lib-invoke-vars">
                  <div class="lib-invoke-var">
                    <input class="lib-invoke-k" value="target" />
                    <input class="lib-invoke-v" value="crates/runtime/src/scheduler.rs" />
                    <button class="lib-invoke-x" type="button" aria-label="Remove">×</button>
                  </div>
                  <div class="lib-invoke-var">
                    <input class="lib-invoke-k" value="strict" />
                    <input class="lib-invoke-v" value="true" />
                    <button class="lib-invoke-x" type="button" aria-label="Remove">×</button>
                  </div>
                  <button class="lib-invoke-add" type="button">
                    <sui-icon name="plus" size="xs"></sui-icon>
                    Add var
                  </button>
                </div>
              </div>

              <div class="lib-invoke-grid">
                <div class="lib-invoke-section">
                  <label class="lib-invoke-label">Workspace</label>
                  <div class="lib-invoke-select">
                    <code>ship-monorepo</code>
                    <sui-icon name="chevron-down" size="xs" tone="muted"></sui-icon>
                  </div>
                </div>
                <div class="lib-invoke-section">
                  <label class="lib-invoke-label">Session</label>
                  <div class="lib-invoke-radio">
                    <label><input type="radio" name="sess" checked /> New session</label>
                    <label><input type="radio" name="sess" /> Attach to <code>s_a1b2c3</code></label>
                  </div>
                </div>
              </div>

              <div class="lib-invoke-actions">
                <span class="lib-card-h-sub">Produces: ${s.artifacts.map(a => `<code>${esc(a)}</code>`).join(' · ')}</span>
                <sui-button tone="primary" variant="solid" size="md">
                  <sui-icon slot="leading" name="play" size="xs"></sui-icon>
                  Invoke skill
                </sui-button>
              </div>
            </section>

            <section class="lib-invoke-result lib-card">
              <div class="lib-card-h">
                <h3>Last result</h3>
                ${latest ? `<sui-chip tone="${latest.status === 'ok' ? 'success' : 'muted'}" variant="soft" size="sm">${esc(latest.status)}</sui-chip>` : ''}
              </div>
              ${!latest ? `
                <p class="lib-empty-inline">No invocations from this Studio session yet.</p>
              ` : `
                <dl class="lib-kv">
                  <dt>run_id</dt>          <dd><code>${esc(latest.run_id)}</code></dd>
                  <dt>session_id</dt>      <dd><code>${esc(latest.session_id)}</code></dd>
                  <dt>reused_session</dt>  <dd>${latest.reused_session ? 'yes' : 'no'}</dd>
                  <dt>mode</dt>            <dd><code>library</code></dd>
                </dl>
                ${latest.artifacts.length > 0 ? `
                  <div class="lib-invoke-artifacts">
                    <div class="lib-card-h-sub" style="margin-bottom:6px">Artifacts</div>
                    ${latest.artifacts.map(a => `
                      <a class="lib-artifact" href="#">
                        <sui-icon name="file-text" size="sm"></sui-icon>
                        <code>${esc(a)}</code>
                        <span class="lib-card-h-sub">via ship.artifacts.get</span>
                      </a>
                    `).join('')}
                  </div>
                ` : ''}
              `}
            </section>

            <section class="lib-card lib-card-wide">
              <div class="lib-card-h">
                <h3>This Studio session</h3>
                <span class="lib-card-h-sub">${hist.length} invocation${hist.length === 1 ? '' : 's'} · local, not persisted to runtime</span>
              </div>
              ${hist.length === 0 ? `<p class="lib-empty-inline">No invocations yet.</p>` : `
                <table class="lib-invoke-table">
                  <thead><tr><th>when</th><th>run_id</th><th>vars</th><th>status</th><th>artifacts</th></tr></thead>
                  <tbody>
                    ${hist.map(r => `
                      <tr>
                        <td>${esc(r.when)}</td>
                        <td><code>${esc(r.run_id)}</code></td>
                        <td>${Object.entries(r.vars).map(([k,v]) => `<code>${esc(k)}=${esc(v)}</code>`).join(' ')}</td>
                        <td><sui-chip tone="${r.status === 'ok' ? 'success' : 'muted'}" variant="soft" size="sm">${esc(r.status)}</sui-chip></td>
                        <td>${r.artifacts.length > 0 ? r.artifacts.map(a => `<code>${esc(a)}</code>`).join(' ') : '—'}</td>
                      </tr>
                    `).join('')}
                  </tbody>
                </table>
              `}
            </section>

          </div>
        </div>
      `;
    },
  },

  mcp: {
    overview: (m) => detailHeader('mcp', m, [
      ['overview','Overview'], ['tools','Tools', m.tool_count],
      ['env','Env', m.env_count], ['used','Used by', m.used_by],
    ], 'overview') + `
      <div class="lib-detail-body">
        <div class="lib-detail-grid">

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h"><h3>Description</h3></div>
            <p class="lib-card-prose">${esc(m.description)}</p>
          </section>

          <section class="lib-card">
            <div class="lib-card-h"><h3>Connection</h3><sui-button variant="ghost" size="sm">Edit</sui-button></div>
            <dl class="lib-kv">
              <dt>Transport</dt>  <dd><code>${esc(m.transport)}</code></dd>
              <dt>Scope</dt>      <dd>${esc(m.scope)}</dd>
              <dt>${m.transport === 'stdio' ? 'Command' : 'URL'}</dt>
              <dd><code class="lib-mono-break">${esc(m.command || m.url)}</code></dd>
              <dt>Enabled</dt>    <dd>${m.enabled ? 'yes' : '<span class="lib-perm-c is-deny">no</span>'}</dd>
            </dl>
          </section>

          <section class="lib-card">
            <div class="lib-card-h"><h3>Test connection</h3><span class="lib-card-h-sub">on demand</span></div>
            <p class="lib-empty-inline">
              Spawn the server, list tools, and check the schema parses. Doesn't affect runtime sessions.
            </p>
            <div class="lib-mcp-test">
              <sui-button variant="outline" size="sm">
                <sui-icon slot="leading" name="zap" size="xs"></sui-icon>
                Run test
              </sui-button>
              <span class="lib-card-h-sub">last tested: never (this Studio session)</span>
            </div>
          </section>

          <section class="lib-card">
            <div class="lib-card-h"><h3>Definition</h3></div>
            <div class="lib-stat-grid">
              <div><span class="lib-stat-n">${m.tool_count}</span><span class="lib-stat-l">tools</span></div>
              <div><span class="lib-stat-n">${m.env_count}</span><span class="lib-stat-l">env keys</span></div>
              <div><span class="lib-stat-n">${m.used_by}</span><span class="lib-stat-l">used by</span></div>
            </div>
          </section>

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h"><h3>Exposed tools</h3><span class="lib-card-h-sub">${m.tool_count} total · declared in mcp.jsonc</span></div>
            <div class="lib-tool-grid">
              ${Array.from({ length: Math.min(m.tool_count, 8) }, (_, i) => {
                const names = m.id === 'filesystem' ? ['read','write','list','glob','grep','stat','mkdir','rm'] :
                              m.id === 'github' ? ['list_issues','get_issue','create_issue','update_issue','list_prs','get_pr','create_pr','review_pr'] :
                              m.id === 'postgres' ? ['select','explain','schemas','tables'] :
                              m.id === 'brave-search' ? ['search'] :
                              ['search','fetch','index'];
                return `
                  <div class="lib-tool">
                    <code>mcp__${esc(m.id.replace(/-/g,'_'))}__${esc(names[i] || `tool_${i}`)}</code>
                    <span class="lib-tool-meta">read · idempotent</span>
                  </div>
                `;
              }).join('')}
              ${m.tool_count > 8 ? `<div class="lib-tool lib-tool-more">+${m.tool_count - 8} more</div>` : ''}
            </div>
          </section>

        </div>
      </div>
    `,
  },

  pipelines: {
    overview: (p) => detailHeader('pipelines', p, [
      ['overview','Overview'], ['steps','Steps', p.steps.length],
      ['vars','Vars', p.vars_count],
    ], 'overview') + `
      <div class="lib-detail-body">
        <div class="lib-pipe-page">

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h"><h3>Description</h3>${p.auto_merge ? '<sui-chip tone="primary" variant="soft" size="sm">auto-merge on green</sui-chip>' : ''}</div>
            <p class="lib-card-prose">${esc(p.description)}</p>
          </section>

          <section class="lib-card lib-card-wide lib-pipe-graph-card">
            <div class="lib-card-h">
              <h3>Graph</h3>
              <span class="lib-card-h-sub">${p.steps.length} step${p.steps.length === 1 ? '' : 's'} · DAG</span>
              <sui-button variant="ghost" size="sm">Open in editor</sui-button>
            </div>
            <div class="lib-pipe-graph">
              ${p.steps.map((s, i) => `
                ${i > 0 ? '<div class="lib-pipe-edge"><span></span></div>' : ''}
                <div class="lib-pipe-node" data-type="${esc(s.type)}">
                  <div class="lib-pipe-node-h">
                    <span class="lib-pipe-node-type">${esc(s.type)}</span>
                    <strong>${esc(s.id)}</strong>
                  </div>
                  ${s.agent ? `<a href="#agents/${esc(s.agent)}" class="lib-pipe-node-agent">${provBlob('anthropic')}${esc(s.agent)}</a>` :
                              `<div class="lib-pipe-node-cmd"><code>shell</code></div>`}
                </div>
              `).join('')}
            </div>
          </section>

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h">
              <h3>Validity</h3>
              <span class="lib-card-h-sub">local definition checks</span>
            </div>
            <ul class="lib-check-list">
              <li class="is-ok"><span class="lib-check-icon">✓</span> YAML parses cleanly</li>
              <li class="is-ok"><span class="lib-check-icon">✓</span> ${p.steps.length} step ids are unique</li>
              <li class="is-ok"><span class="lib-check-icon">✓</span> DAG has no cycles</li>
              <li class="is-ok"><span class="lib-check-icon">✓</span> All <code>after:</code> refs resolve</li>
              <li class="is-ok"><span class="lib-check-icon">✓</span> ${p.steps.filter(s => s.agent).length} agent ref${p.steps.filter(s => s.agent).length === 1 ? '' : 's'} resolve</li>
            </ul>
          </section>

        </div>
      </div>
    `,
  },

  themes: {
    overview: (t) => detailHeader('themes', t, [
      ['tokens','Tokens', t.overrides], ['preview','Preview'],
      ['source','Source'],
    ], 'tokens') + `
      <div class="lib-detail-body">
        <div class="lib-theme-page">

          <section class="lib-theme-editor">
            <div class="lib-theme-eh">
              <div>
                <h3>Tokens</h3>
                <p class="lib-card-h-sub">${t.overrides} overridden vs Ship defaults</p>
              </div>
              <div class="lib-pills">
                <button class="lib-pill is-active">All</button>
                <button class="lib-pill">Foundation</button>
                <button class="lib-pill">Tone</button>
                <button class="lib-pill">Typography</button>
                <button class="lib-pill">Motion</button>
              </div>
            </div>
            <div class="lib-theme-rows">
              ${[
                ['--primary', t.swatches[1], 'color'],
                ['--background', t.swatches[0], 'color'],
                ['--foreground', t.swatches[2], 'color'],
                ['--muted-foreground', t.swatches[3], 'color'],
                ['--accent', t.swatches[1], 'color'],
                ['--radius-md', '8px', 'len'],
                ['--radius-lg', '12px', 'len'],
                ['--font-body', '"Söhne", system-ui, sans-serif', 'text'],
                ['--font-display', '"Söhne Breit", "Söhne", sans-serif', 'text'],
                ['--duration-medium', '220ms', 'len'],
              ].map(([name, val, kind]) => `
                <div class="lib-tok-row" data-kind="${kind}">
                  <label class="lib-tok-name"><code>${esc(name)}</code></label>
                  ${kind === 'color' ? `<span class="lib-tok-swatch" style="background:${esc(val)}"></span>` : ''}
                  <input class="lib-tok-input" type="text" value="${esc(val)}" />
                </div>
              `).join('')}
            </div>
          </section>

          <section class="lib-theme-preview">
            <div class="lib-theme-ph">
              <h3>Live preview</h3>
              <span class="lib-card-h-sub">Updates as you type</span>
            </div>
            <div class="lib-theme-preview-frame" style="--p:${esc(t.swatches[1])};--b:${esc(t.swatches[0])};--f:${esc(t.swatches[2])};--m:${esc(t.swatches[3])}">
              <div class="lib-tp-row">
                <button class="lib-tp-btn lib-tp-primary">Primary</button>
                <button class="lib-tp-btn lib-tp-outline">Outline</button>
                <button class="lib-tp-btn lib-tp-ghost">Ghost</button>
              </div>
              <div class="lib-tp-card">
                <strong>Card title</strong>
                <p>Cards sit on background, text reads foreground.</p>
              </div>
              <div class="lib-tp-input"><span>Search…</span></div>
              <div class="lib-tp-nav">
                <a class="is-active">Dashboard</a>
                <a>Pipelines</a>
                <a>Agents</a>
              </div>
              <div class="lib-tp-stack">
                <div class="lib-tp-toast">info — preview reflects tone ramp</div>
                <div class="lib-tp-toast is-ok">success — token saved</div>
                <div class="lib-tp-toast is-err">error — bad token value</div>
              </div>
            </div>
          </section>

        </div>
      </div>
    `,
  },

  components: {
    overview: (c) => {
      const props = SELECTED_COMPONENT.props;
      return detailHeader('components', { ...c, name: c.tag || c.id, id: c.tag || c.id, hue: 'green', letter: '<>' }, [
        ['overview','Overview'], ['props','Props', props.length],
        ['examples','Examples'], ['source','Source'],
      ], 'overview') + `
        <div class="lib-detail-body">
          <div class="lib-detail-grid">

            <section class="lib-card lib-card-wide">
              <div class="lib-card-h"><h3>${esc(c.tag || c.id)}</h3><span class="lib-card-h-sub">${esc(SELECTED_COMPONENT.desc)}</span></div>
              <div class="lib-comp-preview">
                <div class="lib-comp-stage">
                  <div class="lib-cp-row">
                    <sui-button variant="solid" tone="primary">Primary</sui-button>
                    <sui-button variant="solid" tone="destructive">Destructive</sui-button>
                    <sui-button variant="solid" tone="success">Success</sui-button>
                    <sui-button variant="solid" tone="warning">Warning</sui-button>
                  </div>
                  <div class="lib-cp-row">
                    <sui-button variant="soft" tone="primary">Soft</sui-button>
                    <sui-button variant="outline" tone="primary">Outline</sui-button>
                    <sui-button variant="ghost">Ghost</sui-button>
                    <sui-button variant="solid" tone="primary" disabled>Disabled</sui-button>
                  </div>
                  <div class="lib-cp-row">
                    <sui-button size="sm" tone="primary" variant="solid">Small</sui-button>
                    <sui-button size="md" tone="primary" variant="solid">Medium</sui-button>
                    <sui-button size="lg" tone="primary" variant="solid">Large</sui-button>
                  </div>
                </div>
              </div>
            </section>

            <section class="lib-card lib-card-wide">
              <div class="lib-card-h"><h3>Props</h3></div>
              <table class="lib-prop-table">
                <thead><tr><th>name</th><th>type</th><th>default</th></tr></thead>
                <tbody>
                  ${props.map(p => `<tr><td><code>${esc(p.name)}</code></td><td><code>${esc(p.type)}</code></td><td><code>${esc(p.default)}</code></td></tr>`).join('')}
                </tbody>
              </table>
            </section>

          </div>
        </div>
      `;
    }
  },

  permissions: {
    overview: (p) => detailHeader('permissions', p, [
      ['overview','Overview'], ['used','Used by', p.used_by], ['source','Source'],
    ], 'overview') + `
      <div class="lib-detail-body">
        <div class="lib-perm-page">

          <section class="lib-card lib-card-wide">
            <div class="lib-card-h"><h3>Description</h3></div>
            <p class="lib-card-prose">${esc(p.description)}</p>
          </section>

          <div class="lib-perm-matrix">
            ${[['allow', p.allow], ['ask', p.ask], ['deny', p.deny]].map(([k, list]) => `
              <div class="lib-perm-col is-${k}">
                <div class="lib-perm-col-h">
                  <span class="lib-perm-c is-${k}">${k}</span>
                  <span class="lib-perm-col-count">${list.length}</span>
                  <sui-button variant="ghost" size="sm">
                    <sui-icon slot="leading" name="plus" size="xs"></sui-icon>Add
                  </sui-button>
                </div>
                <ul class="lib-perm-list">
                  ${list.length === 0 ? '<li class="lib-perm-empty">—</li>' :
                    list.map(t => `<li><code>${esc(t)}</code></li>`).join('')}
                </ul>
              </div>
            `).join('')}
          </div>

        </div>
      </div>
    `,
  },

};

// ─── router ───────────────────────────────────────────────────────

function route() {
  const hash = location.hash.slice(1) || 'agents';
  const parts = hash.split('/');
  const domain = parts[0];
  let id = parts[1];
  const tab = parts[2] || 'overview';

  const items = ({ agents: AGENTS, skills: SKILLS, mcp: MCPS, pipelines: PIPELINES,
                  themes: THEMES, permissions: PERMISSIONS })[domain] || [];

  let selected;
  if (domain === 'components') {
    selected = items.find(x => x.id === id) || SELECTED_COMPONENT;
    selected = { ...SELECTED_COMPONENT, id: id || SELECTED_COMPONENT.tag, tag: id || SELECTED_COMPONENT.tag };
  } else {
    selected = items.find(x => x.id === id) || items[0];
    if (selected) id = selected.id;
  }

  document.querySelector('.library').setAttribute('data-domain', domain);
  renderRail(domain);
  renderBrowse(domain, items, id);
  renderDetail(domain, selected, tab);
}

window.addEventListener('hashchange', route);
route();
