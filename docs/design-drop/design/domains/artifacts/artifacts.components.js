/* artifacts.components.js — sui-artifact-* specs + renderers, mounted on the
   shared ComponentViewer. Fixtures mirror fixtures/artifacts.demo.json. */
(function () {
  "use strict";
  const I = {
    image:'<path d="M3 3h18v18H3z" fill="none"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/>',
    md:'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M9 13l1.5 2L12 13l1.5 2L15 13"/>',
    json:'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M8.5 12.5s-1 0-1 1.5 1 1.5 1 1.5M15.5 12.5s1 0 1 1.5-1 1.5-1 1.5"/>',
    text:'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6M9 13h6M9 17h6"/>',
    binary:'<path d="M21 16V8a2 2 0 0 0-1-1.7l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.7l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><path d="M3.3 7L12 12l8.7-5M12 22V12"/>',
    download:'<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>',
    upload:'<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/>',
    trash:'<path d="M3 6h18M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>',
    edit:'<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.1 2.1 0 0 1 3 3L12 15l-4 1 1-4z"/>',
    promote:'<path d="M12 19V5M5 12l7-7 7 7"/>', check:'<path d="M20 6L9 17l-5-5"/>',
    search:'<circle cx="11" cy="11" r="8"/><path d="M21 21l-4.3-4.3"/>',
    alert:'<path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"/><path d="M12 9v4M12 17h.01"/>',
    file:'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/>',
    grid:'<path d="M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3z"/>',
  };
  const svg = (p, w) => '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="'+(w||1.7)+'" stroke-linecap="round" stroke-linejoin="round">'+p+'</svg>';
  const NOW = Date.parse('2026-06-01T00:30:00Z');
  const ART = [
    { id:'art-diff', name:'gate.diff', mime:'text/plain', size:3072, owner:'gatekeeper', collection:null, viewer:'text', viewer_url:'/api/v1/artifacts/art-diff', vlabel:'view/git', retained:false, created:'2026-05-31T22:00:00Z' },
    { id:'art-arch-png', name:'architecture.png', mime:'image/png', size:184320, owner:'claude.opus', collection:null, viewer:'image', viewer_url:'/api/v1/artifacts/art-arch-png', vlabel:'canvas:image', retained:true, created:'2026-05-31T20:10:00Z' },
    { id:'art-spike-md', name:'kysely-spike-findings.md', mime:'text/markdown', size:6144, owner:'claude.opus', collection:'notes', viewer:'markdown', viewer_url:'/api/v1/artifacts/art-spike-md', vlabel:'canvas:doc', retained:false, created:'2026-05-31T19:00:00Z' },
    { id:'art-cov-json', name:'coverage-report.json', mime:'application/json', size:20480, owner:'rust-lane', collection:null, viewer:'json', viewer_url:'/api/v1/artifacts/art-cov-json', vlabel:'json', retained:false, created:'2026-05-30T11:00:00Z' },
    { id:'art-bin-wasm', name:'module.wasm', mime:'application/wasm', size:1048576, owner:null, collection:null, viewer:'binary', viewer_url:null, vlabel:'binary', retained:true, created:'2026-05-24T08:00:00Z' },
  ];
  const fmtSize = b => b>=1048576?(b/1048576).toFixed(1)+' MB':b>=1024?Math.round(b/1024)+' KB':b+' B';
  const fmtAge = iso => { const d=(NOW-Date.parse(iso))/1000; return d<3600?Math.max(1,Math.round(d/60))+'m':d<86400?Math.round(d/3600)+'h':Math.round(d/86400)+'d'; };
  const esc = s => s.replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));
  const ficon = v => ({image:I.image,markdown:I.md,json:I.json,text:I.text,binary:I.binary}[v]||I.file);

  function rowHTML(a, sel) {
    const vchip = a.viewer_url ? '<span class="chip t-accent">↗ '+a.vlabel+'</span>' : '<span class="chip">⊘ '+a.vlabel+'</span>';
    const col = a.collection ? '<span class="chip">'+a.collection+'</span>' : '';
    const dot = a.retained ? '<span class="st-dot" data-durable></span>' : '<span class="st-dot" data-scratch></span>';
    return '<div class="a-row"'+(sel?' data-selected':'')+'>'+dot+
      '<div class="a-name"><span class="ficon">'+svg(ficon(a.viewer))+'</span><span class="nm"><b>'+a.name+'</b><span class="vrow">'+vchip+col+'</span></span></div>'+
      '<span class="a-type">'+a.mime+'</span><span class="a-actor">'+(a.owner||'—')+'</span><span class="a-age">'+fmtAge(a.created)+'</span><span class="a-size">'+fmtSize(a.size)+'</span><span></span>'+
      '<span class="a-acts">'+(a.retained?'':'<button class="act act-go" title="Promote">'+svg(I.promote)+'</button>')+
      '<button class="act" title="Rename">'+svg(I.edit)+'</button><button class="act act-go" title="Download">'+svg(I.download)+'</button><button class="act act-danger" title="Delete">'+svg(I.trash)+'</button></span></div>';
  }
  const head = '<div class="a-head"><span></span><span>Name</span><span>Type</span><span>Actor</span><span>Age</span><span class="r">Size</span><span></span></div>';
  function listState(s) {
    if (s==='loading') return '<div class="browser">'+head+[0,1,2].map(()=>'<div class="skrow"><div class="sk sk-ic"></div><div class="sk-tx"><div class="sk" style="height:11px;width:42%"></div><div class="sk" style="height:9px;width:64%"></div></div></div>').join('')+'</div>';
    if (s==='error') return '<div class="browser"><div class="state" data-tone="error"><div class="ic">'+svg(I.alert)+'</div><h3>Couldn\u2019t load artifacts</h3><p>The artifacts service didn\u2019t respond. Retry.</p><button class="btn btn-primary btn-sm">Retry</button></div></div>';
    if (s==='empty') return '<div class="browser"><div class="state"><div class="ic">'+svg(I.file)+'</div><h3>No artifacts yet</h3><p>Run a workflow, or drop a file to upload one.</p><button class="btn btn-accent btn-sm">'+svg(I.upload)+'Upload</button></div></div>';
    if (s==='empty-filtered') return '<div class="browser"><div class="state"><div class="ic">'+svg(I.search)+'</div><h3>No matches</h3><p>5 artifacts stored, none match the current filters.</p><button class="btn btn-ghost btn-sm">Clear filters</button></div></div>';
    return '<div class="browser">'+head+'<div class="a-list">'+ART.map((a,i)=>rowHTML(a,i===0)).join('')+'</div></div>';
  }
  function jsonHTML(v,d){ d=d||0; if(v===null)return '<span class="jp">null</span>'; if(typeof v==='number')return '<span class="jn">'+v+'</span>'; if(typeof v==='boolean')return '<span class="jb">'+v+'</span>'; if(typeof v==='string')return '<span class="js">"'+v+'"</span>'; const arr=Array.isArray(v),e=arr?v.map((x,i)=>[i,x]):Object.entries(v); if(!e.length)return arr?'[]':'{}'; const pad='  '.repeat(d+1),pe='  '.repeat(d); return (arr?'[':'{')+e.map(([k,val])=>'\n'+pad+(arr?'':'<span class="jk">"'+k+'"</span><span class="jp">: </span>')+jsonHTML(val,d+1)).join('<span class="jp">,</span>')+'\n'+pe+(arr?']':'}'); }
  const COV={ total:{lines:{pct:87.4,covered:1843,total:2109},branches:{pct:79.1}}, files:[{path:'crates/ship-collection/src/lib.rs',lines_pct:92.3},{path:'crates/ship-collection/src/query.rs',lines_pct:74.0}] };
  const DIFF=[['hd','--- a/query.rs'],['hd','+++ b/query.rs'],['ctx','@@ query compiler @@'],['del','-  fn compile(ast) -> String {'],['add','+  fn compile(ast) -> Result<String, QueryError> {'],['del','-    Kind::Join => panic!("unsupported"),'],['add','+    Kind::Join => return Err(QueryError::Unsupported),']];
  const MD='<h1>Kysely spike — findings</h1><p>Captured the query AST as a Kysely <code>OperationNode</code> and compiled through a Rust executor. <strong>It works</strong> — single-table selects round-trip.</p><h2>Open</h2><ul><li>Autocomplete waits on <code>@ship/db</code></li><li>Named-query persistence</li></ul>';
  function archSVG(){ return '<svg viewBox="0 0 360 200" xmlns="http://www.w3.org/2000/svg"><rect width="360" height="200" fill="#fff"/><rect x="20" y="18" width="86" height="44" rx="6" fill="#f7e9cf" stroke="#c79330" stroke-width="1.6"/><text x="63" y="44" font-size="11" font-weight="600" text-anchor="middle" fill="#7a5b1c">Rail · Nav</text><rect x="128" y="18" width="212" height="104" rx="6" fill="#fbf2dd" stroke="#c79330" stroke-width="1.6"/><text x="234" y="40" font-size="11" font-weight="600" text-anchor="middle" fill="#7a5b1c">View (sui-*)</text><rect x="20" y="138" width="320" height="44" rx="6" fill="#e3edf3" stroke="#6f93a8" stroke-width="1.6"/><text x="180" y="165" font-size="11" font-weight="600" text-anchor="middle" fill="#3c5666">Daemon · RPC services</text></svg>'; }
  function preview(kind) {
    const a = ART.find(x=>x.viewer===kind)||ART[0]; let body;
    if (kind==='image') body='<div class="pv-image"><div class="frame">'+archSVG()+'</div></div>';
    else if (kind==='markdown') body='<div class="pv-md">'+MD+'</div>';
    else if (kind==='json') body='<div class="pv-json"><div class="jl">'+jsonHTML(COV)+'</div></div>';
    else if (kind==='text') body='<pre class="pv-pre">'+DIFF.map(([c,t])=>'<span class="dl '+({add:'add',del:'del',hd:'hd'}[c]||'')+'">'+esc(t)+'</span>').join('\n')+'</pre>';
    else body='<div class="pv-binary"><div class="ic">'+svg(I.binary)+'</div><p><b>module.wasm</b> — 1.0 MB<br>Binary file. Download to view.</p><a class="btn btn-primary btn-sm">'+svg(I.download)+'Download</a></div>';
    return '<div class="preview-host"><div class="pv-head"><div class="pv-ti"><b>'+a.name+'</b><div class="pv-meta">'+a.mime+' · '+fmtSize(a.size)+'</div></div></div><div class="pv-body" style="max-height:320px">'+body+'</div></div>';
  }
  function filters(active) {
    return '<div class="filters" style="display:flex;gap:8px;flex-wrap:wrap;align-items:center;width:100%">'+
      '<div class="search"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.3-4.3"/></svg><input type="search" placeholder="Search by name, id, actor…" '+(active?'value="coverage"':'')+'></div>'+
      '<select class="sel"><option>'+(active?'JSON':'All types')+'</option></select><select class="sel"><option>'+(active?'rust-lane':'All actors')+'</option></select><select class="sel"><option>'+(active?'Last 24h':'Any age')+'</option></select>'+
      '<button class="lnk">'+(active?'Clear · 3 active':'Clear')+'</button></div>';
  }
  function actions(kind) {
    const scr = kind==='scratch';
    return '<div class="stage-row">'+(scr?'<button class="btn btn-soft btn-sm">'+svg(I.promote)+'Promote</button>':'<button class="btn btn-ghost btn-sm" disabled>'+svg(I.check)+'Durable</button>')+
      '<button class="btn btn-ghost btn-sm">'+svg(I.edit)+'Rename</button><button class="btn btn-ghost btn-sm">'+svg(I.download)+'Download</button><button class="btn btn-ghost btn-sm" style="color:var(--destructive)">'+svg(I.trash)+'Delete</button></div>'+
      '<div class="stage-note">'+(scr?'Scratch blob → <b>Promote</b> enabled (<code>Promote</code>). Delete asks to confirm.':'Already durable → Promote shown as done. Rename calls <code>Patch</code>.')+'</div>';
  }
  const watch = s => '<div class="stage-row"><button class="wb" data-state="'+s+'"><span class="wb-dot"></span><span>'+(s==='live'?'Live':s==='reconnecting'?'Reconnecting…':'Paused')+'</span></button></div>';
  const upload = () => '<div class="stage-inner" style="max-width:440px"><div class="dropzone"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/></svg><div>Drop a file here, or <b>browse</b></div></div><div class="upl-field"><label>Collection <span class="muted" style="font-weight:400">(optional)</span></label><select class="sel"><option>No collection</option><option>notes</option></select></div></div>';

  const api = (p,e,s) => ({ props:p||[], events:e||[], slots:s||[] });
  ComponentViewer.mount({
    eyebrow:'domain handoff · artifacts · component-driven',
    title:'Artifacts — components',
    domainAccent:'var(--app-accent-canvas)',
    intro:'The handoff is a <b>set of components</b>, not a screen. Each <code>sui-artifact-*</code> is designed here in isolation — every state, its API contract, its own status. Switch <b>ship themes</b> top-right to prove they\u2019re token-driven. The <a href="artifacts.html" style="color:var(--primary)">composed view</a> is just how they assemble.',
    storagePrefix:'artcomp',
    links:[{label:'See the composition →', href:'artifacts.html', icon:svg(I.grid,1.8)}, {label:'handoff.md', href:'handoff.md', ghost:true}, {label:'spec.md', href:'spec.md', ghost:true}],
    composition:{ label:'Artifacts view →', href:'artifacts.html' },
    components:[
      { id:'list', tag:'sui-artifact-list', badges:['new'], rpc:['List','Watch'], state:'ok', states:['ok','loading','empty','empty-filtered','error'], render:listState,
        purpose:'The browser — one row per artifact with type, actor, age, size, viewer + durable/scratch status. Filterable, selectable, and <span class="calls">live</span>: applies <code>Watch</code> patches in place.',
        api:api([['density','"comfortable"|"compact"','Row height + padding.'],['selected-id','string','Highlights the active row.'],['live','boolean','Subscribe to Watch + apply patches.']],[['sui-row-click','{ id }','Row activated → open in preview.'],['sui-action','{ act, id }','Promote / rename / delete / download.']]) },
      { id:'preview', tag:'sui-artifact-preview', badges:['new'], rpc:['bytes via viewer_url'], state:'image', states:['image','markdown','json','text','binary'], render:preview,
        purpose:'Renders one artifact <b>by mime</b> — image inline, markdown rendered, JSON tree, text/diff, binary→download. The <span class="calls">mime→viewer mapping is the core logic</span> (mirrors <code>viewers.ts</code>).',
        api:api([['artifact','ArtifactRecord','The record to render.'],['viewer','auto|image|markdown|json|text|binary','Override the mime→viewer resolution.']],[['sui-download','{ id }','User asked to download bytes.']],[['header','','Optional custom title bar.']]) },
      { id:'filters', tag:'sui-artifact-filters', badges:['new'], rpc:['none — filters result'], state:'empty', states:['empty','active'], render:s=>filters(s==='active'),
        purpose:'Filter the list by <b>type · actor · age · collection</b>, plus free-text search. Emits a filter set; reflects active filters with a clearable count.',
        api:api([['facets','{ mimes, actors, collections }','Options for the selects.'],['value','FilterSet','Controlled current filters.']],[['sui-change','FilterSet','Filters changed.'],['sui-clear','—','All filters reset.']]) },
      { id:'actions', tag:'sui-artifact-actions', badges:['new'], rpc:['Promote','Patch','Delete'], state:'scratch', states:['scratch','durable'], render:actions,
        purpose:'The artifact-specific verbs: <b>Promote</b> (scratch→durable), <b>Rename</b> (<code>Patch</code>), <b>Delete</b> (confirm), Download. Promote only offered for scratch blobs.',
        api:api([['artifact','ArtifactRecord','Drives which verbs are enabled.']],[['sui-promote','{ id }','calls Promote.'],['sui-rename','{ id, name }','calls Patch.'],['sui-delete','{ id }','calls Delete after confirm.']]) },
      { id:'watch-badge', tag:'sui-watch-badge', badges:['new','shared'], rpc:['Watch status'], state:'live', states:['live','reconnecting','paused'], render:watch,
        purpose:'Small <b>live / reconnecting / paused</b> indicator for any reactive surface. <b>Shared with collections</b> — reflects the <code>Watch</code> connection, owns no data.',
        api:api([['state','live|reconnecting|paused','Connection status to display.'],['label','boolean','Show the text label beside the dot.']]) },
      { id:'upload', tag:'sui-artifact-upload', badges:['new'], rpc:['create path'], state:'idle', states:['idle'], render:upload,
        purpose:'Drop or select a file → create an artifact, optionally into a target collection (composes <code>sui-collection-picker</code>).',
        api:api([['accept','string','MIME accept filter.'],['collections','Collection[]','Options for the target picker.']],[['sui-submit','{ files, collection }','Files chosen → create.']]) },
    ],
  });
})();
