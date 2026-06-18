/* renders.js — render functions for the recently-added sui-* packs.
   Each returns token-faithful HTML for a given state. Markup mirrors the real
   pack anatomy (shadow-DOM structure, prop/event names, ARIA) so the catalog
   reads as SHIP, not an approximation. Exposed on window.SUIR for the config. */
(function () {
  "use strict";
  const I = {
    chevron:'<path d="M6 9l6 6 6-6"/>', up:'<path d="M18 15l-6-6-6 6"/>', dn:'<path d="M6 9l6 6 6-6"/>',
    check:'<path d="M20 6L9 17l-5-5"/>', search:'<circle cx="11" cy="11" r="8"/><path d="M21 21l-4.3-4.3"/>',
    cal:'<rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/>',
    left:'<path d="M15 18l-6-6 6-6"/>', right:'<path d="M9 18l6-6-6-6"/>', plus:'<path d="M12 5v14M5 12h14"/>',
    send:'<path d="M22 2L11 13M22 2l-7 20-4-9-9-4z"/>', paperclip:'<path d="M21.4 11.05 12.25 20.2a5 5 0 0 1-7.07-7.07l9.19-9.19a3 3 0 1 1 4.24 4.24l-9.2 9.19a1 1 0 0 1-1.41-1.41l8.49-8.49"/>',
    play:'<path d="M5 3l14 9-14 9z"/>', img:'<rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="9" cy="9" r="2"/><path d="M21 15l-5-5L5 21"/>',
    spin:'<path d="M21 12a9 9 0 1 1-6.2-8.5"/>', alert:'<path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"/><path d="M12 9v4M12 17h.01"/>',
    move:'<path d="M5 9l-3 3 3 3M9 5l3-3 3 3M15 19l-3 3-3-3M19 9l3 3-3 3M2 12h20M12 2v20"/>',
    pen:'<path d="M12 19l7-7 3 3-7 7-3-3z"/><path d="M18 13l-1.5-7.5L2 2l3.5 14.5L13 18z"/><path d="M2 2l7.6 7.6"/>',
    sq:'<rect x="4" y="4" width="16" height="16" rx="1"/>', arrow:'<path d="M5 12h14M12 5l7 7-7 7"/>',
    grid:'<path d="M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3z"/>', sort:'<path d="M3 6h18M6 12h12M10 18h4"/>',
    folder:'<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
    file:'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/>',
    layout:'<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M9 21V9"/>',
  };
  const svg = (p, w) => '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="' + (w || 1.7) + '" stroke-linecap="round" stroke-linejoin="round">' + p + '</svg>';
  const note = t => '<div class="stage-note">' + t + '</div>';

  /* ───────────────────────── FORMS ───────────────────────── */
  const TEAM = [['agent-router','Agent Router'],['mesh-gateway','Mesh Gateway'],['authority','Authority'],['pipeline-exec','Pipeline Exec'],['substrate-db','Substrate DB']];

  function combobox(s) {
    const open = s !== 'closed';
    const q = s === 'filtered' ? 'me' : '';
    const rows = TEAM.filter(o => o[1].toLowerCase().includes(q))
      .map((o, i) => '<li class="pop-row" role="option" data-value="' + o[0] + '"' + (i === 0 && open ? ' data-active' : '') + (o[0] === 'mesh-gateway' ? ' aria-selected="true"' : '') +
        '>' + (q ? o[1].replace(/(me)/i, '<mark>$1</mark>') : o[1]) + (o[0] === 'mesh-gateway' ? '<span class="chk">' + svg(I.check) + '</span>' : '') + '</li>').join('');
    const list = s === 'empty'
      ? '<div class="pop-empty">No services match <b>“zzz”</b></div>'
      : '<ul class="pop" role="listbox" style="margin-top:6px">' + rows + '</ul>';
    return '<div style="max-width:280px">' +
      '<div class="field-input" role="combobox" aria-expanded="' + open + '"' + (open ? ' data-open' : '') + '>' +
        svg(I.search) + '<span' + (open ? '' : ' class="ph"') + '>' + (s === 'empty' ? 'zzz' : open && s !== 'filtered' ? 'Mesh Gateway' : q ? 'me' : 'Mesh Gateway') + '</span>' +
        '<span class="chev">' + svg(I.chevron) + '</span></div>' +
      (open ? list : '') + '</div>' +
      note('Type-to-filter single select. <span class="kbd">↑</span><span class="kbd">↓</span> move · <span class="kbd">↵</span> commit · <span class="kbd">esc</span> close. Emits <code>sui-change</code> <code>{value,label}</code>; <code>aria-activedescendant</code> tracks the highlight.');
  }

  function autocomplete(s) {
    const loading = s === 'loading';
    const sug = [['SHIP-210','ship-collection integrator'],['SHIP-211','Cutover + 27 consumers'],['SHIP-213','Collections Data Studio']];
    return '<div style="max-width:280px">' +
      '<div class="field-input" data-open><span>collect</span><span style="margin-left:auto">' + (loading ? '<span style="display:inline-flex;color:var(--muted-foreground)" class="spin">' + svg(I.spin) + '</span>' : '') + '</span></div>' +
      (loading
        ? '<div class="pop" style="margin-top:6px"><div class="pop-empty">Searching…</div></div>'
        : '<ul class="pop" role="listbox" style="margin-top:6px">' + sug.map((r, i) =>
            '<li class="pop-row"' + (i === 0 ? ' data-active' : '') + '><span style="font-family:var(--font-mono);font-size:11px;color:var(--muted-foreground)">' + r[0] + '</span><span style="overflow:hidden;text-overflow:ellipsis">' + r[1] + '</span></li>').join('') + '</ul>') +
      '</div>' +
      note('Free-text input with <b>async suggestions</b> (debounced). Unlike combobox the typed value is valid on its own — suggestions augment, they don\'t constrain. ' + (loading ? 'Pending request shows an inline spinner, never a layout shift.' : 'Emits <code>sui-input</code> while typing, <code>sui-select</code> on pick.'));
  }

  function datepicker(s) {
    const range = s === 'range';
    return '<div style="max-width:300px">' +
      '<div class="field-input" data-open style="max-width:300px">' + svg(I.cal) +
        '<span>' + (range ? 'May 26 – Jun 1, 2026' : 'Jun 1, 2026') + '</span><span class="chev">' + svg(I.chevron) + '</span></div>' +
      '<div class="pop" style="margin-top:6px;max-width:300px">' + calMonthInner(range) + '</div></div>' +
      note('Text field + calendar popover; shares the <code>sui-calendar</code> grid engine. <code>range</code> prop flips to two-ended selection. Keyboard: arrows move the focused day, <span class="kbd">↵</span> selects, <span class="kbd">esc</span> closes back to the field.');
  }
  function calMonthInner(range) {
    let cells = '';
    for (let d = 1; d <= 30; d++) {
      const inR = range && d >= 26 && d <= 31;
      const end = range && (d === 26 || d === 31);
      cells += '<div class="cal-cell" style="min-height:30px;text-align:center" ' + (d === 1 ? 'data-today' : '') + (inR ? ' data-range' : '') + (end ? ' data-range-end' : '') + '><span class="dn">' + d + '</span></div>';
    }
    return '<div style="padding:8px"><div class="cal-bar" style="border:0;padding:2px 4px 8px"><span class="cal-title" style="font-size:var(--text-sm)">June 2026</span><span class="cal-nav" style="margin-left:auto"><button>' + svg(I.left) + '</button><button>' + svg(I.right) + '</button></span></div>' +
      '<div class="cal-grid" style="grid-template-columns:repeat(7,1fr)">' + ['S','M','T','W','T','F','S'].map(x => '<div class="cal-dow">' + x + '</div>').join('') + cells + '</div></div>';
  }

  function colorpicker(s) {
    const swatches = ['--app-accent-dashboard','--app-accent-pipelines','--app-accent-agents','--app-accent-skills','--app-accent-git','--app-accent-collections','--app-accent-canvas','--app-accent-chat'];
    if (s === 'custom') {
      return '<div style="max-width:220px"><div class="field-input" data-open style="max-width:220px"><span class="swatch" style="background:var(--app-accent-agents)"></span><span style="font-family:var(--font-mono);font-size:var(--text-sm)">oklch(.58 .22 295)</span><span class="chev">' + svg(I.chevron) + '</span></div>' +
        '<div class="pop" style="margin-top:6px;max-width:220px"><div class="cp-area"><span class="cp-dot"></span></div><div class="cp-hue"></div>' +
        '<div style="display:flex;gap:6px;padding:0 10px 10px"><span class="kv" style="flex:1;font-size:11px;font-family:var(--font-mono);padding:5px 8px;border:1px solid var(--border);border-radius:var(--radius-sm)">oklch(.58 .22 295)</span></div></div></div>' +
        note('Custom mode: an OKLCH area + hue rail. <b>Stores OKLCH, not hex</b> — so picked colors survive the same theme-aware lightness mapping as tokens. Emits <code>sui-change</code> with the resolved value.');
    }
    return '<div style="max-width:220px"><div class="field-input" data-open style="max-width:220px"><span class="swatch" style="background:var(--app-accent-agents)"></span><span style="font-family:var(--font-mono);font-size:var(--text-sm)">agents</span><span class="chev">' + svg(I.chevron) + '</span></div>' +
      '<div class="pop" style="margin-top:6px;max-width:220px"><div class="cp-grid">' + swatches.map((c, i) => '<span class="cp-sw"' + (i === 2 ? ' data-on' : '') + ' style="background:var(' + c + ')"></span>').join('') + '</div>' +
      '<div style="padding:0 10px 10px"><button class="lnk" style="font-size:11px">Custom…</button></div></div></div>' +
      note('Swatch grid bound to the <b>app-accent token ramp</b> — the curated palette, not a free-for-all. Drives per-app theming in settings.');
  }

  /* ───────────────────────── DATA ───────────────────────── */
  const RUNS = [
    ['router-sync','agents','ok',412,'1.2s','98.6'],
    ['mesh-relay','mesh','ok',388,'0.9s','99.1'],
    ['nightly-reindex','pipelines','warn',1204,'14.7s','91.0'],
    ['authority-rotate','authority','ok',12,'0.3s','100'],
    ['csv-ingest','pipelines','err',0,'—','0'],
  ];
  function gx(on) { return '<span class="gx-check"' + (on ? ' data-on' : '') + '>' + (on ? svg(I.check, 2.4) : '') + '</span>'; }
  function statusPill(v) {
    const m = { ok: ['t-success', 'ok'], warn: ['t-warning', 'warn'], err: ['t-destructive', 'error'] }[v];
    return '<span class="chip ' + m[0] + '">' + m[1] + '</span>';
  }
  function table(s) {
    if (s === 'loading') return '<div class="grid"><table class="grid-tbl"><thead><tr><th></th><th>run</th><th>app</th><th>status</th><th class="num">calls</th></tr></thead><tbody>' +
      [0,1,2,3].map(() => '<tr><td>' + gx() + '</td>' + [0,1,2,3].map(() => '<td><div class="sk" style="height:11px;width:70%"></div></td>').join('') + '</tr>').join('') + '</tbody></table></div>';
    if (s === 'empty') return '<div class="grid"><div class="state"><div class="ic">' + svg(I.grid) + '</div><h3>No runs match</h3><p>Clear the filter, or widen the time window.</p></div></div>';
    const grouped = s === 'grouped', sorted = s === 'sorted', sel = s === 'selected';
    const rows = [...RUNS];
    if (sorted) rows.sort((a, b) => b[3] - a[3]);
    const body = grouped
      ? '<tr class="grp-row" data-open><td colspan="6"><span class="twist"></span>pipelines · 3 runs</td></tr>' +
        rows.filter(r => r[1] === 'pipelines').map(r => rowHtml(r, sel)).join('') +
        '<tr class="grp-row" data-open><td colspan="6"><span class="twist"></span>agents · 1 run</td></tr>' +
        rows.filter(r => r[1] === 'agents').map(r => rowHtml(r, sel)).join('')
      : rows.map((r, i) => rowHtml(r, sel && (i === 1 || i === 3))).join('');
    const sortHead = (label, col, active) => '<th' + (active ? ' aria-sort="descending"' : '') + (col === 'calls' ? ' class="num"' : '') + '><span class="sort">' + label + ' ' + svg(active ? I.dn : I.sort, 2) + '</span><span class="grip"></span></th>';
    return '<div class="grid"><table class="grid-tbl"><thead><tr>' +
      '<th style="width:28px">' + gx(sel) + '</th>' + sortHead('run', 'run', false) + sortHead('app', 'app', false) + sortHead('status', 'status', false) + sortHead('calls', 'calls', sorted) + '<th>p99</th></tr></thead>' +
      '<tbody>' + body + '</tbody></table>' +
      '<div class="grid-foot"><span><b>' + RUNS.length + '</b> runs</span>' + (sel ? '<span>· <b>2</b> selected</span>' : '') + '<span class="pager"><button>' + svg(I.left, 2) + '</button><button aria-current>1</button><button>2</button><button>3</button><button>' + svg(I.right, 2) + '</button></span></div></div>' +
      note('The reference engine: <b>sort · filter · group · select · resize · paginate</b>, row-virtualized. ' + (sorted ? 'Sorted by <code>calls</code> desc — the active header carries <code>aria-sort</code>.' : grouped ? 'Grouped by <code>app</code> with collapsible group rows.' : 'Columns are resizable (drag the grip); selection is checkbox + range.'));
  }
  function rowHtml(r, sel) {
    return '<tr' + (sel ? ' aria-selected="true"' : '') + '><td>' + gx(sel) + '</td><td class="cap">' + r[0] + '</td><td><span class="chip">' + r[1] + '</span></td><td>' + statusPill(r[2]) + '</td><td class="num">' + r[3] + '</td><td class="num">' + r[5] + '</td></tr>';
  }

  function treegrid(s) {
    const open = s !== 'collapsed';
    const twist = (o, leaf) => leaf ? '<span class="tw-leaf"></span>' : '<span class="tw-ind"' + (o ? ' data-open' : '') + '><span class="twist"></span></span>';
    const ic = p => '<span class="tw-ic" style="display:inline-flex">' + svg(p, 1.7) + '</span>';
    const row = (depth, content, app, calls, st) => '<tr><td style="padding-left:' + (10 + depth * 18) + 'px">' + content + '</td><td><span class="chip">' + app + '</span></td><td class="num">' + calls + '</td><td>' + statusPill(st) + '</td></tr>';
    let body = row(0, twist(open) + ic(I.folder) + '<b>pipelines</b>', 'pipelines', '1,216', 'warn');
    if (open) {
      body += row(1, twist(true) + ic(I.folder) + 'nightly-reindex', 'pipelines', '1,204', 'warn');
      body += row(2, twist(false, true) + ic(I.file) + 'reindex · shard 0', 'pipelines', '602', 'ok');
      body += row(2, twist(false, true) + ic(I.file) + 'reindex · shard 1', 'pipelines', '602', 'warn');
      body += row(1, twist(false, true) + ic(I.file) + 'csv-ingest', 'pipelines', '0', 'err');
    }
    body += row(0, twist(false) + ic(I.folder) + '<b>agents</b>', 'agents', '412', 'ok');
    return '<div class="grid"><table class="grid-tbl"><thead><tr><th>node</th><th>app</th><th class="num">calls</th><th>status</th></tr></thead><tbody>' + body + '</tbody></table></div>' +
      note('Hierarchy × columns — the tree lives in column 1, the data grid spans the rest. Twisties expand/collapse; <span class="kbd">→</span>/<span class="kbd">←</span> open/close, <span class="kbd">↑</span><span class="kbd">↓</span> walk visible rows. Shares the <code>sui-table</code> column model.');
  }

  function sheet(s) {
    const cols = ['A','B','C','D'];
    const data = [
      ['run','calls','p99','cost'],
      ['router-sync','412','98.6','=B2*0.004'],
      ['mesh-relay','388','99.1','=B3*0.004'],
      ['nightly','1204','91.0','=B4*0.004'],
      ['','=SUM(B2:B4)','=AVERAGE(C2:C4)','=SUM(D2:D4)'],
    ];
    const editing = s === 'editing', formula = s === 'formula', range = s === 'selection';
    let grid = '<tr><th></th>' + cols.map(c => '<th>' + c + '</th>').join('') + '</tr>';
    data.forEach((rowv, r) => {
      grid += '<tr><th>' + (r + 1) + '</th>' + rowv.map((v, c) => {
        const isF = String(v).startsWith('=');
        const sel = (formula && r === 4 && c === 0) || (editing && r === 1 && c === 0);
        const anchor = range && r === 1 && c === 1;
        const inRange = range && c === 1 && r >= 1 && r <= 3;
        const cls = (isF ? 'f' : (!isNaN(parseFloat(v)) && v !== '' ? 'n' : '')) + (editing && r === 1 && c === 0 ? ' editing' : '');
        const attr = (sel ? ' data-sel' : '') + (anchor ? ' data-anchor' : '') + (inRange && !anchor ? ' data-range' : '');
        const shownF = isF ? (formula || editing ? '' : evalish(v)) : v;
        const inner = (editing && r === 1 && c === 0) ? '<input value="router-sync" />' : (isF && !(formula) ? evalish(v) : v);
        const fill = (range && r === 3 && c === 1) ? '<span class="sheet-fill"></span>' : '';
        return '<td class="' + cls + '"' + attr + '>' + inner + fill + '</td>';
      }).join('') + '</tr>';
    });
    const fxbar = formula
      ? '<span class="sheet-name">D5</span><span class="sheet-fx">ƒx</span><span class="sheet-formula active">=SUM(D2:D4)</span>'
      : range ? '<span class="sheet-name">B2:B4</span><span class="sheet-fx">ƒx</span><span class="sheet-formula">412</span>'
      : '<span class="sheet-name">A2</span><span class="sheet-fx">ƒx</span><span class="sheet-formula">router-sync</span>';
    return '<div class="sheet" style="max-width:420px"><div class="sheet-bar">' + fxbar + '</div><table class="sheet-grid">' + grid + '</table></div>' +
      note('Spreadsheet primitive: <b>A1 addressing · formula engine · range-select · fill · copy/paste · keyboard grid</b>. ' + (formula ? 'Formula cells (<code>=SUM</code>, <code>=AVERAGE</code>) recompute through a DAG; circular refs are caught.' : range ? 'Drag-select a range; the corner handle fills down.' : editing ? 'Double-click or type to edit in place; <span class="kbd">↵</span> commits, <span class="kbd">esc</span> reverts.' : 'Just landed — engine + interactivity.'));
  }
  function evalish(f) {
    const m = { '=B2*0.004': '1.65', '=B3*0.004': '1.55', '=B4*0.004': '4.82', '=SUM(B2:B4)': '2004', '=AVERAGE(C2:C4)': '96.2', '=SUM(D2:D4)': '8.02' };
    return m[f] || f;
  }

  function calendar(s) {
    if (s === 'week') return calWeek();
    const range = s === 'range';
    const evs = { 6: [['Substrate cutover','s']], 12: [['Mesh GA','a'],['Authority review','']], 13: [['Standup','']], 20: [['Pipeline freeze','s']], 24: [['Demo','a'],['Retro','']] };
    let cells = '';
    for (let i = 0; i < 35; i++) {
      const day = i - 0; const inMonth = day >= 1 && day <= 30; const dn = inMonth ? day : (day < 1 ? 30 + day : day - 30);
      const inR = range && day >= 12 && day <= 20; const end = range && (day === 12 || day === 20);
      const ev = evs[day] || [];
      cells += '<div class="cal-cell"' + (!inMonth ? ' data-out' : '') + (day === 1 ? ' data-today' : '') + (inR ? ' data-range' : '') + (end ? ' data-range-end' : '') + '>' +
        '<span class="dn">' + dn + '</span>' +
        ev.slice(0, 2).map(e => '<div class="cal-ev ' + e[1] + '">' + e[0] + '</div>').join('') +
        (ev.length > 2 ? '<div class="cal-more">+' + (ev.length - 2) + ' more</div>' : '') + '</div>';
    }
    return '<div class="cal"><div class="cal-bar"><span class="cal-title">June 2026</span><span class="seg" style="margin-left:auto"><button>Month</button><button aria-pressed="true">' + (range ? 'Month' : 'Month') + '</button></span>' +
      '<span class="cal-nav"><button>' + svg(I.left) + '</button><button>' + svg(I.right) + '</button></span></div>' +
      '<div class="cal-grid">' + ['Sun','Mon','Tue','Wed','Thu','Fri','Sat'].map(x => '<div class="cal-dow">' + x + '</div>').join('') + cells + '</div></div>' +
      note('Month / week / day views, typed events (<b>all-day vs timed</b> discriminated union). ' + (range ? '<b>Range-select</b> + drag-create/move just landed — drag across days to author a span.' : 'Keyboard grid: arrows walk days, <span class="kbd">↵</span> opens, drag to create.'));
  }
  function calWeek() {
    const hours = ['09','10','11'];
    let body = '';
    hours.forEach(h => { body += '<div class="tlab">' + h + ':00</div>' + [0,1,2].map(() => '<div class="tcol"><div class="tcell"></div></div>').join(''); });
    return '<div class="cal"><div class="cal-bar"><span class="cal-title">Jun 10 – 12</span><span class="seg" style="margin-left:auto"><button>Day</button><button aria-pressed="true">Week</button><button>Month</button></span></div>' +
      '<div class="cal-grid" style="grid-template-columns:40px repeat(3,1fr)"><div class="cal-dow"></div><div class="cal-dow">Mon 10</div><div class="cal-dow">Tue 11</div><div class="cal-dow">Wed 12</div></div>' +
      '<div class="cal-week" style="grid-template-columns:40px repeat(3,1fr);position:relative">' + body +
        '<div class="tev" style="grid-column:2;top:6px;height:42px">Standup</div>' +
        '<div class="tev" style="grid-column:4;top:34px;height:54px;background:var(--tone-accent-bg);color:var(--tone-accent-fg);border-left-color:var(--accent)">Mesh GA review</div>' +
      '</div></div>' +
      note('Week view lays timed events into <b>collision lanes</b>; all-day events get their own band. The view-model is pure — positions (<code>topPct</code>/<code>heightPct</code>) are computed, the template just paints.');
  }

  /* ───────────────────────── CANVAS ───────────────────────── */
  function board(s) {
    const connecting = s === 'connecting', sel = s === 'selected';
    const node = (x, y, t, sub, on) => '<div class="bd-node"' + (on ? ' data-selected' : '') + ' style="left:' + x + 'px;top:' + y + 'px"><div class="bn-t">' + t + '</div><div class="bn-s">' + sub + '</div>' +
      '<span class="bd-port" style="left:-5px;top:50%;transform:translateY(-50%)"></span><span class="bd-port" style="right:-5px;top:50%;transform:translateY(-50%)"></span></div>';
    return '<div class="canvas-stage">' +
      '<div class="canvas-tools"><button aria-pressed="true">' + svg(I.move, 1.8) + '</button><button>' + svg(I.sq, 1.8) + '</button><button>' + svg(I.arrow, 1.8) + '</button></div>' +
      '<svg class="edges" style="position:absolute;inset:0;width:100%;height:100%;pointer-events:none">' +
        '<path class="bd-edge" data-incident d="M134,58 C190,58 180,108 232,108" marker-end="url(#ah)"></path>' +
        '<path class="bd-edge" d="M134,58 C190,58 180,170 232,170"></path>' +
        (connecting ? '<path class="bd-edge pending" d="M340,122 C380,122 380,150 410,150"></path>' : '') +
        '<defs><marker id="ah" markerWidth="8" markerHeight="8" refX="6" refY="4" orient="auto"><path d="M0,0 L8,4 L0,8" fill="none" stroke="var(--primary)" stroke-width="1.5"/></marker></defs></svg>' +
      node(40, 38, 'router-sync', 'agents', sel) + node(232, 88, 'mesh-relay', 'mesh', false) + node(232, 150, 'authority', 'authority', false) +
      '<div class="canvas-zoom">100%</div></div>' +
      note('Node/edge diagram on the pan/zoom substrate, <b>dagre auto-layout</b>. ' + (connecting ? 'Dragging from a <b>port</b> draws a pending dashed edge until it lands on a target port.' : sel ? 'Selected node carries <code>data-selected</code>; its edges light up via <code>data-incident</code>.' : 'SVG-native edges; nodes are positioned in world coordinates.'));
  }
  function draw(s) {
    const tool = s === 'shapes' ? 1 : s === 'select' ? 2 : 0;
    return '<div class="canvas-stage">' +
      '<div class="canvas-tools"><button aria-pressed="' + (tool === 0) + '">' + svg(I.pen, 1.8) + '</button><button aria-pressed="' + (tool === 1) + '">' + svg(I.sq, 1.8) + '</button><button aria-pressed="' + (tool === 2) + '">' + svg(I.move, 1.8) + '</button></div>' +
      '<svg style="position:absolute;inset:0;width:100%;height:100%">' +
        '<rect class="dw-shape" x="48" y="44" width="120" height="64" rx="6"></rect>' +
        '<path class="dw-stroke" d="M210,150 q20,-50 50,-20 q30,30 60,-10"></path>' +
        (tool === 2 ? '<rect class="dw-sel" x="42" y="38" width="132" height="76"></rect><rect class="dw-handle" x="38" y="34" width="8" height="8"></rect><rect class="dw-handle" x="170" y="34" width="8" height="8"></rect><rect class="dw-handle" x="38" y="110" width="8" height="8"></rect><rect class="dw-handle" x="170" y="110" width="8" height="8"></rect>' : '') +
      '</svg><div class="canvas-zoom">fit</div></div>' +
      note('SVG whiteboard — freehand strokes + shapes with a <b>rough.js</b> hand-drawn render. ' + (tool === 1 ? 'Shape tool: drag to size, snaps to grid.' : tool === 2 ? 'Select tool exposes resize handles + bounding box.' : 'Pen tool captures a smoothed stroke path.'));
  }

  /* ───────────────────────── CHAT ───────────────────────── */
  function messageList(s) {
    if (s === 'empty') return '<div class="grid" style="border:0"><div class="state"><div class="ic">' + svg(I.send) + '</div><h3>Start the thread</h3><p>Ask the agent anything about this run.</p></div></div>' + note('Virtualized thread — only the viewport window + overscan render, so a 10k-message session stays smooth.');
    const streaming = s === 'streaming';
    return '<div class="chat">' +
      '<div class="msg user"><div class="av">YOU</div><div class="bub">Why did <b>nightly-reindex</b> drop to 91% p99?</div></div>' +
      '<div class="msg assistant"><div class="av">SH</div><div class="bub">Shard 1 hit lock contention during the cutover window. I can re-run it off-peak.' + (streaming ? '' : ' <span class="cite"><span class="cn">1</span></span>') + '</div></div>' +
      (streaming
        ? '<div class="msg assistant"><div class="av">SH</div><div class="bub stream">Checking the pipeline history now</div></div>'
        : '<div class="msg user"><div class="av">YOU</div><div class="bub">Do it.</div></div>') +
      '</div>' +
      note('Roles, bubbles, inline citations, streaming caret. Auto-scrolls only while <b>pinned to the bottom</b> — scroll up and new messages won\'t yank you down (<code>stickyBottom</code>).');
  }
  function composer(s) {
    const multi = s === 'multiline', disabled = s === 'disabled';
    return '<div class="composer' + (disabled ? '' : ' focus') + '">' +
      '<button class="ghost-ic">' + svg(I.paperclip, 1.6) + '</button>' +
      (multi
        ? '<textarea class="cf" rows="3">Re-run nightly-reindex off-peak and\nnotify me when shard 1 clears.</textarea>'
        : '<input class="cf" ' + (disabled ? 'placeholder="Agent is responding…" disabled' : 'value="Do it."') + ' />') +
      '<div class="cbtns"><button class="send"' + (disabled ? ' disabled' : '') + '>' + svg(I.send, 1.8) + '</button></div></div>' +
      (multi ? '<div style="margin-top:8px"><span class="attach">' + svg(I.file, 1.6) + 'run-1204.log</span></div>' : '') +
      note('Input with attachments. <span class="kbd">↵</span> submits' + (multi ? ', <span class="kbd">⇧</span>+<span class="kbd">↵</span> newline (multiline)' : '') + '. Emits <code>sui-composer-submit</code> <code>{text,attachments}</code>; ' + (disabled ? 'disabled while the agent streams.' : 'attachments are host-owned — the composer only echoes them.'));
  }
  function toolCall(s) {
    const st = s === 'running' ? 'running' : s === 'error' ? 'error' : 'success';
    const sIcon = st === 'running' ? '<span class="spin" style="display:inline-flex">' + svg(I.spin, 2) + '</span>' : st === 'error' ? svg(I.alert, 2) : svg(I.check, 2.4);
    const label = st === 'running' ? 'running' : st === 'error' ? 'failed' : '142ms';
    return '<div class="chat"><div class="msg assistant"><div class="av">SH</div><div class="tool" data-state="' + st + '">' +
      '<div class="tool-h"><span style="color:var(--muted-foreground)">' + svg(I.grid, 1.7) + '</span><span class="tn">query_pipeline_runs</span><span class="ts">' + sIcon + label + '</span></div>' +
      '<div class="tool-b">{ "app": "pipelines", "since": "24h", "status": "warn" }' + (st === 'success' ? '\n→ 1 run · nightly-reindex · p99 91.0' : st === 'error' ? '\n✗ timeout after 30s' : '') + '</div></div></div></div>' +
      note('Renders an agent tool invocation: name, typed args, and a <b>running / success / error</b> state. The body collapses by default; result streams in when the call resolves.');
  }
  function citation(s) {
    const open = s === 'hover';
    return '<div class="chat"><div class="msg assistant"><div class="av">SH</div><div><div class="bub">The cutover preserved item data — the projection rebuilt from the event log <span class="cite"><span class="cn">1</span> source</span>.</div>' +
      (open ? '<div class="cite-card"><div class="src">substrate-adr-014.md · §4</div><div class="ct">“Removing a field rebuilds the projection from the event log. Existing item data is preserved; the column is dropped.”</div></div>' : '') +
      '</div></div></div>' +
      note('Inline source chip that expands to the cited span on hover/focus. Numbered, <code>aria-describedby</code> links the claim to the source card.');
  }

  /* ───────────────────────── MEDIA / TOOLING ───────────────────────── */
  function media(s) {
    if (s === 'csv') return '<div class="media" style="max-width:420px"><div class="grid" style="border:0;border-radius:0"><table class="grid-tbl"><thead><tr><th>run</th><th>app</th><th class="num">calls</th></tr></thead><tbody>' +
      RUNS.slice(0, 3).map(r => '<tr><td class="cap">' + r[0] + '</td><td>' + r[1] + '</td><td class="num">' + r[3] + '</td></tr>').join('') + '</tbody></table></div>' +
      '<div class="media-bar"><span class="mime">text/csv</span><span>runs-24h.csv · 5 rows · 3 cols</span></div></div>' +
      note('CSV renderer parses + types columns into a read-only <code>sui-table</code>. Part of the mime-detecting <code>sui-media</code> set (image · audio-video · csv).');
    if (s === 'audio') return '<div class="media" style="max-width:420px"><div class="media-av"><span class="play">' + svg(I.play) + '</span><div class="wave">' +
      Array.from({ length: 38 }, (_, i) => '<i class="' + (i < 14 ? 'p' : '') + '" style="height:' + (20 + Math.abs(Math.sin(i * 0.9)) * 70) + '%"></i>').join('') + '</div></div>' +
      '<div class="media-bar"><span class="mime">audio/mpeg</span><span class="media-time">0:38 / 2:14</span></div></div>' +
      note('Audio/video renderer with a scrub waveform + transport. Same shell handles <code>audio/*</code> and <code>video/*</code> by mime.');
    return '<div class="media" style="max-width:420px"><div class="media-img">' + svg(I.img) + '</div><div class="media-bar"><span class="mime">image/png</span><span>diagram-export.png · 1240×820</span></div></div>' +
      note('Image renderer with zoom-to-fit + intrinsic-size readout. <b>pdf and map are <code>ship-*</code> renderers</b>, dispatched by the <code>ship-viewer</code> mime router.');
  }
  function viewer(s) {
    const tab = s === 'matrix' ? 1 : s === 'source' ? 2 : 0;
    let main;
    if (tab === 1) {
      main = '<div class="cvx-matrix">' + ['rest','hover','focus','disabled','loading','active'].map(st =>
        '<div><div class="cvx-cellh">' + st + '</div><div class="cvx-cell"><button class="btn ' + (st === 'active' ? 'btn-primary' : 'btn-ghost') + ' btn-sm"' + (st === 'disabled' ? ' disabled' : '') + '>Run</button></div></div>').join('') + '</div>';
    } else if (tab === 2) {
      main = '<div class="cvx-canvas" style="display:block;font-family:var(--font-mono);font-size:11px;text-align:left;padding:14px;line-height:1.7"><span style="color:var(--syntax-tag)">&lt;sui-button</span> <span style="color:var(--syntax-attr)">variant</span>=<span style="color:var(--syntax-string)">"primary"</span> <span style="color:var(--syntax-attr)">size</span>=<span style="color:var(--syntax-string)">"sm"</span><span style="color:var(--syntax-tag)">&gt;</span>Run<span style="color:var(--syntax-tag)">&lt;/sui-button&gt;</span></div>';
    } else {
      main = '<div class="cvx-canvas"><button class="btn btn-primary btn-sm">Run</button></div>';
    }
    return '<div class="cvx" style="max-width:460px"><div class="cvx-main"><div class="cvx-tabs"><button aria-pressed="' + (tab === 0) + '">Render</button><button aria-pressed="' + (tab === 1) + '">Matrix</button><button aria-pressed="' + (tab === 2) + '">Source</button></div>' + main + '</div>' +
      '<div class="cvx-knobs"><h5>Props</h5>' +
        '<div class="knob"><label>variant</label><div class="kv">primary ▾</div></div>' +
        '<div class="knob"><label>size</label><div class="kv">sm ▾</div></div>' +
        '<div class="knob"><label>radius</label><div class="ktrack"></div></div>' +
        '<div class="knob"><label>disabled</label><div class="kv">false</div></div>' +
      '</div></div>' +
      note('The review surface itself: <b>live render · prop-knobs · variant/state matrix · source panel</b>, with the theme switcher. This whole catalog is what it produces. (Native shell is Tauri; this runs in a plain browser.)');
  }

  window.SUIR = {
    svg, I,
    combobox, autocomplete, datepicker, colorpicker,
    table, treegrid, sheet, calendar,
    board, draw,
    messageList, composer, toolCall, citation,
    media, viewer,
  };
})();
