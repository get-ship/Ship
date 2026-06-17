/* Artifacts view — interactions + fixture rendering.
   Fixtures mirror design/domains/artifacts/fixtures/artifacts.demo.json
   (inlined so the mock is self-contained). Preview "bytes" are
   representative mock content — the real viewer fetches viewer_url. */

(function () {
  "use strict";

  // ── icons (lucide-ish) ────────────────────────────────────────────
  const I = {
    image:'<path d="M3 3h18v18H3z" fill="none"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/>',
    md:'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M9 13l1.5 2L12 13l1.5 2L15 13" fill="none"/>',
    json:'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M8.5 12.5s-1 0-1 1.5 1 1.5 1 1.5M15.5 12.5s1 0 1 1.5-1 1.5-1 1.5"/>',
    text:'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6M9 13h6M9 17h6"/>',
    binary:'<path d="M21 16V8a2 2 0 0 0-1-1.7l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.7l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><path d="M3.3 7L12 12l8.7-5M12 22V12"/>',
    download:'<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>',
    upload:'<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>',
    trash:'<path d="M3 6h18M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>',
    edit:'<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.1 2.1 0 0 1 3 3L12 15l-4 1 1-4z"/>',
    promote:'<path d="M12 19V5M5 12l7-7 7 7"/>',
    search:'<circle cx="11" cy="11" r="8"/><path d="M21 21l-4.3-4.3"/>',
    chev:'<path d="M9 18l6-6-6-6"/>',
    x:'<path d="M18 6L6 18M6 6l12 12"/>',
    check:'<path d="M20 6L9 17l-5-5"/>',
    alert:'<path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"/><path d="M12 9v4M12 17h.01"/>',
    file:'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/>',
    home:'<path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="M9 22V12h6v10"/>',
    users:'<path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.9"/>',
    db:'<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.7 4 3 9 3s9-1.3 9-3V5M3 12c0 1.7 4 3 9 3s9-1.3 9-3"/>',
    net:'<circle cx="12" cy="5" r="2"/><circle cx="5" cy="19" r="2"/><circle cx="19" cy="19" r="2"/><path d="M12 7v4M12 11l-6 6M12 11l6 6"/>',
    term:'<path d="M4 17l6-6-6-6M12 19h8"/>',
    spark:'<path d="M12 3l1.9 5.8L20 11l-6.1 2.2L12 19l-1.9-5.8L4 11l6.1-2.2z"/>',
    git:'<circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="9" r="3"/><path d="M18 12a6 6 0 0 1-6 6H6"/>',
    play:'<path d="M5 3l14 9-14 9z"/>',
    reset:'<path d="M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5"/>',
  };
  const svg = (p, w) => '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="'+(w||1.7)+'" stroke-linecap="round" stroke-linejoin="round">'+p+'</svg>';

  // ── fixtures ──────────────────────────────────────────────────────
  const NOW = Date.parse('2026-06-01T00:30:00Z');
  let ART = [
    { id:'art-diff', name:'gate.diff', mime:'text/plain', size:3072, owner:'gatekeeper', collection:null, scope:'space:demo', viewer:'text', viewer_url:'/api/v1/artifacts/art-diff', vlabel:'view/git', retained:false, created:'2026-05-31T22:00:00Z' },
    { id:'art-arch-png', name:'architecture.png', mime:'image/png', size:184320, owner:'claude.opus', collection:null, scope:'space:demo', viewer:'image', viewer_url:'/api/v1/artifacts/art-arch-png', vlabel:'canvas:image', retained:true, created:'2026-05-31T20:10:00Z' },
    { id:'art-spike-md', name:'kysely-spike-findings.md', mime:'text/markdown', size:6144, owner:'claude.opus', collection:'notes', scope:'space:demo', viewer:'markdown', viewer_url:'/api/v1/artifacts/art-spike-md', vlabel:'canvas:doc', retained:false, created:'2026-05-31T19:00:00Z' },
    { id:'art-cov-json', name:'coverage-report.json', mime:'application/json', size:20480, owner:'rust-lane', collection:null, scope:'space:demo', viewer:'json', viewer_url:'/api/v1/artifacts/art-cov-json', vlabel:'json', retained:false, created:'2026-05-30T11:00:00Z' },
    { id:'art-bin-wasm', name:'module.wasm', mime:'application/wasm', size:1048576, owner:null, collection:null, scope:'space:demo', viewer:'binary', viewer_url:null, vlabel:'binary', retained:true, created:'2026-05-24T08:00:00Z' },
  ];

  // preview "bytes" (mock content)
  const BYTES = {
    'art-spike-md': { kind:'md', html:
      '<h1>Kysely spike — findings</h1>'+
      '<p>Captured the query AST as a Kysely <code>OperationNode</code> and compiled it through a small Rust executor. <strong>It works</strong> — single-table selects, where/order/limit round-trip cleanly.</p>'+
      '<h2>What landed</h2>'+
      '<ul><li>AST → SQL compile path (no JOINs — single table only)</li><li>Typed result rows decoded from <code>data_json</code></li><li>Param binding for <code>?</code> placeholders</li></ul>'+
      '<h2>Open</h2>'+
      '<ul><li>Autocomplete waits on <code>@ship/db</code> type-gen</li><li>Named-query persistence → <code>.ship/collections/queries/</code></li></ul>'+
      '<pre><code>SELECT * FROM notes\n  WHERE priority >= ?\n  ORDER BY priority DESC LIMIT 50</code></pre>' },
    'art-cov-json': { kind:'json', data:{
      total:{ lines:{ pct:87.4, covered:1843, total:2109 }, branches:{ pct:79.1, covered:402, total:508 } },
      generated_at:'2026-05-30T11:00:00Z',
      files:[
        { path:'crates/ship-collection/src/lib.rs', lines_pct:92.3, uncovered:[88,91,140] },
        { path:'crates/ship-collection/src/query.rs', lines_pct:74.0, uncovered:[22,23,24,51] },
      ],
    }},
    'art-diff': { kind:'diff', lines:[
      ['hd','--- a/crates/ship-collection/src/query.rs'],
      ['hd','+++ b/crates/ship-collection/src/query.rs'],
      ['ctx','@@ gate: query compiler @@'],
      ['del','-    fn compile(ast: &Node) -> String {'],
      ['add','+    fn compile(ast: &Node) -> Result<String, QueryError> {'],
      ['ctx','         match ast.kind {'],
      ['del','-            Kind::Join => panic!("unsupported"),'],
      ['add','+            Kind::Join => return Err(QueryError::Unsupported("JOINs")),'],
      ['ctx','             Kind::Select => { /* … */ }'],
      ['ctx','         }'],
    ]},
    'art-bin-new': { kind:'text', text:'[22:45:01] rust-lane · build start (release)\n[22:45:09] compiling ship-collection v0.4.0\n[22:46:30] warning: 2 unused imports\n[22:47:12] finished release [optimized] in 131.4s\n[22:47:12] artifact: target/release/libship_collection.rlib' },
  };

  // ── helpers ───────────────────────────────────────────────────────
  const $ = (s, r) => (r||document).querySelector(s);
  const fmtSize = b => b >= 1048576 ? (b/1048576).toFixed(1)+' MB' : b >= 1024 ? Math.round(b/1024)+' KB' : b+' B';
  const fmtAge = iso => {
    const d = (NOW - Date.parse(iso)) / 1000;
    if (d < 3600) return Math.max(1, Math.round(d/60)) + 'm';
    if (d < 86400) return Math.round(d/3600) + 'h';
    return Math.round(d/86400) + 'd';
  };
  const viewerOf = a => a.viewer; // image|markdown|json|text|binary

  // ── state ─────────────────────────────────────────────────────────
  let selId = ART[0].id;
  let listState = 'ok';
  let filters = { q:'', mime:'', actor:'', age:'' };

  const ageOk = (iso, f) => {
    if (!f) return true;
    const h = (NOW - Date.parse(iso)) / 3.6e6;
    return f === '24h' ? h <= 24 : f === '7d' ? h <= 168 : h <= 720;
  };
  const mimeBucket = m => m.startsWith('image/') ? 'image' : m.includes('json') ? 'json' : m.includes('markdown') ? 'text' : m.startsWith('text/') ? 'text' : 'binary';
  function filtered() {
    return ART.filter(a => {
      if (filters.mime && mimeBucket(a.mime) !== filters.mime) return false;
      if (filters.actor && a.owner !== filters.actor) return false;
      if (!ageOk(a.created, filters.age)) return false;
      if (filters.q) {
        const q = filters.q.toLowerCase();
        if (!((a.name+' '+a.id+' '+(a.owner||'')).toLowerCase().includes(q))) return false;
      }
      return true;
    });
  }

  // ── render: list ──────────────────────────────────────────────────
  const ficon = v => ({image:I.image, markdown:I.md, json:I.json, text:I.text, binary:I.binary}[v] || I.file);
  function rowHTML(a) {
    const hasViewer = !!a.viewer_url;
    const vchip = hasViewer
      ? '<span class="chip chip-viewer">↗ '+a.vlabel+'</span>'
      : '<span class="chip chip-none">⊘ '+a.vlabel+'</span>';
    const col = a.collection ? '<span class="chip chip-col">'+a.collection+'</span>' : '';
    const dot = a.retained ? '<span class="st-dot" data-durable title="Durable — retained"></span>'
                           : '<span class="st-dot" data-scratch title="Scratch — promote to retain"></span>';
    const promote = a.retained ? '' : '<button class="act act-go" data-act="promote" title="Promote to durable">'+svg(I.promote)+'</button>';
    return '<div class="a-row" data-id="'+a.id+'"'+(a.id===selId?' data-selected':'')+'>'+
      dot+
      '<div class="a-name"><span class="ficon">'+svg(ficon(a.viewer))+'</span>'+
        '<span class="nm"><b>'+a.name+'</b><span class="vrow">'+vchip+col+'</span></span></div>'+
      '<span class="a-type">'+a.mime+'</span>'+
      '<span class="a-actor">'+(a.owner || '—')+'</span>'+
      '<span class="a-age">'+fmtAge(a.created)+'</span>'+
      '<span class="a-size">'+fmtSize(a.size)+'</span>'+
      '<span></span>'+
      '<span class="a-acts">'+promote+
        '<button class="act" data-act="rename" title="Rename">'+svg(I.edit)+'</button>'+
        '<a class="act act-go" href="'+(a.viewer_url||'#')+'" data-act="download" title="Download" onclick="event.stopPropagation()">'+svg(I.download)+'</a>'+
        '<button class="act act-danger" data-act="delete" title="Delete">'+svg(I.trash)+'</button>'+
      '</span>'+
    '</div>';
  }
  const headHTML =
    '<div class="a-head"><span></span><span>Name</span><span>Type</span>'+
    '<span class="c-actor">Actor</span><span>Age</span><span class="r">Size</span><span></span></div>';

  function renderList() {
    const b = $('#browser');
    if (listState === 'loading') {
      b.innerHTML = headHTML + [0,1,2,3].map(()=>'<div class="skrow"><div class="sk sk-ic"></div><div class="sk-tx"><div class="sk" style="height:11px;width:42%"></div><div class="sk" style="height:9px;width:64%"></div></div></div>').join('');
      return;
    }
    if (listState === 'error') {
      b.innerHTML = '<div class="state" data-tone="error"><div class="ic">'+svg(I.alert)+'</div>'+
        '<h3>Couldn\u2019t load artifacts</h3><p>The artifacts service didn\u2019t respond. Check the daemon and retry.</p>'+
        '<button class="btn btn-primary btn-sm" id="retry">Retry</button></div>';
      $('#retry').onclick = () => setState('ok');
      return;
    }
    if (listState === 'empty') {
      b.innerHTML = '<div class="state"><div class="ic">'+svg(I.file)+'</div>'+
        '<h3>No artifacts yet</h3><p>Run a workflow, or drop a file onto this page to upload one.</p>'+
        '<button class="btn btn-accent btn-sm" data-upload>'+svg(I.upload)+'Upload</button></div>';
      return;
    }
    const rows = filtered();
    if (rows.length === 0) {
      b.innerHTML = '<div class="state"><div class="ic">'+svg(I.search)+'</div>'+
        '<h3>No matches</h3><p>'+ART.length+' artifacts stored, none match the current filters.</p>'+
        '<button class="btn btn-ghost btn-sm" id="clearf">Clear filters</button></div>';
      $('#clearf').onclick = clearFilters;
      return;
    }
    b.innerHTML = headHTML + '<div class="a-list">' + rows.map(rowHTML).join('') + '</div>';
  }

  // ── render: preview ───────────────────────────────────────────────
  function jsonHTML(v, depth) {
    depth = depth || 0;
    if (v === null) return '<span class="jp">null</span>';
    if (typeof v === 'number') return '<span class="jn">'+v+'</span>';
    if (typeof v === 'boolean') return '<span class="jb">'+v+'</span>';
    if (typeof v === 'string') return '<span class="js">"'+v+'"</span>';
    const arr = Array.isArray(v);
    const entries = arr ? v.map((x,i)=>[i,x]) : Object.entries(v);
    if (entries.length === 0) return arr ? '[]' : '{}';
    const pad = '  '.repeat(depth+1), padEnd = '  '.repeat(depth);
    let out = '<span class="jt" data-tog><span class="car">▾</span>'+(arr?'[':'{')+'</span><span class="jbody">';
    out += entries.map(([k,val]) =>
      '\n'+pad+(arr?'':'<span class="jk">"'+k+'"</span><span class="jp">: </span>')+jsonHTML(val, depth+1)
    ).join('<span class="jp">,</span>');
    out += '\n'+padEnd+(arr?']':'}')+'</span>';
    return out;
  }

  function renderPreview() {
    const pv = $('#preview');
    const a = ART.find(x => x.id === selId);
    if (!a) {
      pv.innerHTML = '<div class="pv-empty">'+svg(I.file)+'<div>Select an artifact to preview</div></div>';
      return;
    }
    const hasViewer = !!a.viewer_url;
    const promoteBtn = a.retained
      ? '<button class="btn btn-ghost btn-sm" disabled style="opacity:.55">'+svg(I.check)+'Durable</button>'
      : '<button class="btn btn-ghost btn-sm" data-act="promote" data-id="'+a.id+'">'+svg(I.promote)+'Promote</button>';
    let body;
    const v = viewerOf(a);
    if (v === 'image') {
      body = '<div class="pv-image"><div class="frame">'+archSVG()+'</div></div>';
    } else if (v === 'markdown') {
      body = '<div class="pv-md">'+BYTES['art-spike-md'].html+'</div>';
    } else if (v === 'json') {
      body = '<div class="pv-json"><div class="jl">'+jsonHTML(BYTES['art-cov-json'].data)+'</div></div>';
    } else if (v === 'text') {
      const src = BYTES[a.id] || BYTES['art-bin-new'];
      if (src.kind === 'diff') body = '<pre class="pv-pre">'+src.lines.map(([c,t])=>'<span class="dl '+({add:'add',del:'del',hd:'hd'}[c]||'')+'">'+esc(t)+'</span>').join('\n')+'</pre>';
      else body = '<pre class="pv-pre">'+esc(src.text)+'</pre>';
    } else {
      body = '<div class="pv-binary"><div class="ic">'+svg(I.binary)+'</div>'+
        '<p><b>'+a.name+'</b> — '+fmtSize(a.size)+'<br>Binary file. Download to view.</p>'+
        '<a class="btn btn-primary btn-sm" href="'+(a.viewer_url||'#')+'">'+svg(I.download)+'Download</a></div>';
    }
    pv.innerHTML =
      '<div class="pv-head"><div class="pv-ti" id="pvti"><b>'+a.name+'</b>'+
        '<div class="pv-meta">'+a.mime+' · '+fmtSize(a.size)+' · '+(a.owner||'no actor')+
          (hasViewer?' · <span style="color:var(--accent)">↗ '+a.vlabel+'</span>':'')+'</div></div>'+
        '<button class="pv-x" data-pv-close title="Close">'+svg(I.x)+'</button></div>'+
      '<div class="pv-acts">'+promoteBtn+
        '<button class="btn btn-ghost btn-sm" data-act="rename" data-id="'+a.id+'">'+svg(I.edit)+'Rename</button>'+
        '<a class="btn btn-ghost btn-sm" href="'+(a.viewer_url||'#')+'">'+svg(I.download)+'Download</a>'+
        '<span style="flex:1"></span>'+
        '<button class="btn btn-ghost btn-sm" data-act="delete" data-id="'+a.id+'" style="color:var(--destructive)">'+svg(I.trash)+'Delete</button>'+
      '</div>'+
      '<div class="pv-body">'+body+'</div>';
  }
  const esc = s => s.replace(/[&<>]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));
  function archSVG() {
    return '<svg viewBox="0 0 360 230" xmlns="http://www.w3.org/2000/svg" font-family="ui-monospace, monospace">'+
      '<rect width="360" height="230" fill="#ffffff"/>'+
      '<rect x="20" y="20" width="90" height="48" rx="6" fill="#f7e9cf" stroke="#c79330" stroke-width="1.6"/><text x="65" y="48" font-size="11" font-weight="600" text-anchor="middle" fill="#7a5b1c">Rail · Nav</text>'+
      '<rect x="135" y="20" width="205" height="120" rx="6" fill="#fbf2dd" stroke="#c79330" stroke-width="1.6"/><text x="237" y="44" font-size="11" font-weight="600" text-anchor="middle" fill="#7a5b1c">View (sui-*)</text>'+
      '<rect x="155" y="58" width="75" height="64" rx="4" fill="#ffffff" stroke="#d8b46a"/><rect x="245" y="58" width="75" height="64" rx="4" fill="#ffffff" stroke="#d8b46a"/>'+
      '<rect x="20" y="158" width="320" height="52" rx="6" fill="#e3edf3" stroke="#6f93a8" stroke-width="1.6"/><text x="180" y="188" font-size="11" font-weight="600" text-anchor="middle" fill="#3c5666">Daemon · RPC services</text>'+
      '<path d="M65 68 V158" stroke="#b9893a" stroke-width="1.6" fill="none" stroke-dasharray="4 3"/>'+
      '<path d="M237 140 V158" stroke="#b9893a" stroke-width="1.6" fill="none" stroke-dasharray="4 3"/>'+
      '</svg>';
  }

  // ── actions ───────────────────────────────────────────────────────
  function toast(msg, tone) {
    const t = document.createElement('div');
    t.className = 'toast'; t.dataset.tone = tone || 'success';
    const ic = tone === 'danger' ? I.trash : tone === 'warning' ? I.alert : I.check;
    t.innerHTML = svg(ic) + '<span>' + msg + '</span>';
    $('#toasts').appendChild(t);
    setTimeout(() => { t.style.transition = 'opacity .3s'; t.style.opacity = '0'; setTimeout(()=>t.remove(), 300); }, 2600);
  }
  function promote(id) {
    const a = ART.find(x=>x.id===id); if (!a || a.retained) return;
    a.retained = true; renderList(); renderPreview();
    toast('Promoted ' + a.name + ' — now durable', 'success');
  }
  function del(id) {
    const a = ART.find(x=>x.id===id); if (!a) return;
    confirmDialog('Delete artifact?', '\u201c'+a.name+'\u201d will be removed. Content-addressed blobs are GC\u2019d when unreferenced. This can\u2019t be undone.', () => {
      ART = ART.filter(x=>x.id!==id);
      if (selId === id) selId = ART[0] ? ART[0].id : null;
      renderList(); renderPreview(); updateCount();
      toast('Deleted ' + a.name, 'danger');
    });
  }
  function rename(id) {
    selId = id; renderList(); renderPreview();
    const ti = $('#pvti'); if (!ti) return;
    const a = ART.find(x=>x.id===id);
    ti.innerHTML = '<div class="pv-rename"><input id="rn" value="'+a.name+'"><button class="btn btn-primary btn-sm" id="rnok">Save</button></div>';
    const inp = $('#rn'); inp.focus(); inp.select();
    const commit = () => {
      const nv = inp.value.trim();
      if (nv && nv !== a.name) { a.name = nv; toast('Renamed to ' + nv, 'success'); }
      renderList(); renderPreview();
    };
    $('#rnok').onclick = commit;
    inp.onkeydown = e => { if (e.key==='Enter') commit(); if (e.key==='Escape') renderPreview(); };
  }

  // confirm dialog
  function confirmDialog(title, body, onYes) {
    const back = $('#confirm');
    back.querySelector('h3').textContent = title;
    back.querySelector('.dlg-body').textContent = body;
    back.hidden = false;
    const yes = back.querySelector('[data-yes]'), no = back.querySelector('[data-no]');
    const close = () => { back.hidden = true; yes.onclick = no.onclick = null; };
    yes.onclick = () => { close(); onYes(); };
    no.onclick = close;
  }

  // ── live replay (Watch stream) ────────────────────────────────────
  let liveTimers = [];
  function playLive() {
    resetLive();
    setState('ok');
    setWatch('live');
    // frame 2: append build.log
    liveTimers.push(setTimeout(() => {
      const nu = { id:'art-new', name:'build.log', mime:'text/plain', size:9001, owner:'rust-lane', collection:null, scope:'space:demo', viewer:'text', viewer_url:'/api/v1/artifacts/art-new', vlabel:'view/git', retained:false, created:'2026-05-31T22:45:00Z' };
      BYTES['art-new'] = BYTES['art-bin-new'];
      ART = [nu, ...ART]; renderList(); updateCount();
      const row = $('.a-row[data-id="art-new"]');
      if (row) { row.classList.add('is-new'); row.insertAdjacentHTML('beforeend','<span class="new-tag">new</span>'); }
      pulseWatch();
    }, 700));
    // frame 3: remove module.wasm
    liveTimers.push(setTimeout(() => {
      const row = $('.a-row[data-id="art-bin-wasm"]');
      pulseWatch();
      if (row) {
        row.classList.add('is-removing');
        setTimeout(() => {
          ART = ART.filter(x=>x.id!=='art-bin-wasm');
          if (selId==='art-bin-wasm') { selId = ART[0]?ART[0].id:null; }
          renderList(); renderPreview(); updateCount();
        }, 320);
      }
    }, 2100));
  }
  function resetLive() {
    liveTimers.forEach(clearTimeout); liveTimers = [];
    ART = ART.filter(x => x.id !== 'art-new');
    if (!ART.find(x=>x.id==='art-bin-wasm')) {
      ART.push({ id:'art-bin-wasm', name:'module.wasm', mime:'application/wasm', size:1048576, owner:null, collection:null, scope:'space:demo', viewer:'binary', viewer_url:null, vlabel:'binary', retained:true, created:'2026-05-24T08:00:00Z' });
    }
    renderList(); renderPreview(); updateCount();
  }
  let watchState = 'live';
  function setWatch(s) { watchState = s; const w = $('#wb'); w.dataset.state = s; w.querySelector('.wb-tx').textContent = s==='live'?'Live':s==='reconnecting'?'Reconnecting…':'Paused'; }
  function pulseWatch() { const w = $('#wb'); w.animate([{transform:'scale(1)'},{transform:'scale(1.06)'},{transform:'scale(1)'}], {duration:360, easing:'ease-out'}); }

  function updateCount() {
    const vis = filtered().length;
    $('#count').innerHTML = '<b>'+vis+'</b> of <b>'+ART.length+'</b> artifact'+(ART.length===1?'':'s')+(vis<ART.length?' · filtered':'');
  }

  // ── state + controls ──────────────────────────────────────────────
  function setState(s) {
    listState = s;
    document.querySelectorAll('[data-state-btn]').forEach(b => b.setAttribute('aria-pressed', b.dataset.stateBtn===s));
    if (s === 'filtered') { listState='ok'; filters = { q:'', mime:'json', actor:'', age:'24h' }; syncFilterInputs(); }
    renderList(); updateCount();
  }
  function clearFilters() { filters = { q:'', mime:'', actor:'', age:'' }; syncFilterInputs(); renderList(); updateCount(); }
  function syncFilterInputs() {
    $('#f-q').value = filters.q; $('#f-mime').value = filters.mime; $('#f-actor').value = filters.actor; $('#f-age').value = filters.age;
  }

  function setLayout(l) {
    $('#split').dataset.layout = l;
    document.querySelectorAll('[data-layout-btn]').forEach(b => b.setAttribute('aria-pressed', b.dataset.layoutBtn===l));
    const pv = $('#preview'), bk = $('#pvback');
    if (l === 'drawer') { const has = !!selId; pv.classList.toggle('hidden', !has); bk.classList.toggle('show', has); }
    else { pv.classList.remove('hidden'); bk.classList.remove('show'); }
  }
  function setTheme(t) {
    document.documentElement.dataset.theme = t;
    document.documentElement.classList.toggle('dark', t === 'ship-dark');
    document.querySelectorAll('[data-theme-btn]').forEach(b => b.setAttribute('aria-pressed', b.dataset.themeBtn===t));
  }

  // ── wire up ───────────────────────────────────────────────────────
  function init() {
    renderList(); renderPreview(); updateCount();

    // actor filter options
    const actors = [...new Set(ART.map(a=>a.owner).filter(Boolean))];
    $('#f-actor').insertAdjacentHTML('beforeend', actors.map(a=>'<option value="'+a+'">'+a+'</option>').join(''));

    // delegated clicks on list
    $('#browser').addEventListener('click', e => {
      const actBtn = e.target.closest('[data-act]');
      const row = e.target.closest('.a-row');
      if (actBtn && row) {
        const id = row.dataset.id, act = actBtn.dataset.act;
        if (act === 'download') return;
        e.stopPropagation();
        if (act === 'promote') promote(id);
        else if (act === 'rename') rename(id);
        else if (act === 'delete') del(id);
        return;
      }
      if (e.target.closest('[data-upload]')) { openUpload(); return; }
      if (row) { selId = row.dataset.id; renderList(); renderPreview(); if ($('#split').dataset.layout==='drawer') setLayout('drawer'); }
    });

    // preview actions (delegated)
    $('#preview').addEventListener('click', e => {
      const b = e.target.closest('[data-act]');
      if (b) { const id = b.dataset.id; const act = b.dataset.act;
        if (act==='promote') promote(id); else if (act==='rename') rename(id); else if (act==='delete') del(id); return; }
      if (e.target.closest('[data-pv-close]')) { selId=null; renderList(); renderPreview(); if ($('#split').dataset.layout==='drawer') setLayout('drawer'); }
      const tog = e.target.closest('.jt[data-tog]');
      if (tog) { const body = tog.nextElementSibling; const car = tog.querySelector('.car');
        const open = car.textContent==='▾'; car.textContent = open?'▸':'▾'; if (body) body.style.display = open?'none':''; }
    });

    // filters
    $('#f-q').addEventListener('input', e => { filters.q = e.target.value; renderList(); updateCount(); });
    ['mime','actor','age'].forEach(k => $('#f-'+k).addEventListener('change', e => { filters[k]=e.target.value; renderList(); updateCount(); }));
    $('#clear-filters').addEventListener('click', clearFilters);

    // watch badge cycle
    $('#wb').addEventListener('click', () => setWatch(watchState==='live'?'reconnecting':watchState==='reconnecting'?'paused':'live'));

    // upload
    document.querySelectorAll('[data-upload]').forEach(b => b.addEventListener('click', openUpload));
    $('#upload-x').onclick = closeUpload; $('#upload-cancel').onclick = closeUpload;
    $('#upload-submit').onclick = () => { closeUpload(); toast('Uploaded — artifact created', 'success'); };

    // review controls
    document.querySelectorAll('[data-state-btn]').forEach(b => b.onclick = () => setState(b.dataset.stateBtn));
    document.querySelectorAll('[data-layout-btn]').forEach(b => b.onclick = () => setLayout(b.dataset.layoutBtn));
    document.querySelectorAll('[data-theme-btn]').forEach(b => b.onclick = () => setTheme(b.dataset.themeBtn));
    $('#play-live').onclick = playLive;
    $('#reset-live').onclick = () => { resetLive(); setWatch('live'); setState('ok'); };
  }
  function openUpload() { $('#upload').hidden = false; }
  function closeUpload() { $('#upload').hidden = true; }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init);
  else init();
})();
