/* collections.components.js — sui-collection-* "Data Studio" component set,
   mounted on the shared ComponentViewer. Data mirrors fixtures/collections.demo.json.
   Hard rule honored: every surface is typed from the collection schema. */
(function () {
  "use strict";
  const I = {
    db:'<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.7 4 3 9 3s9-1.3 9-3V5M3 12c0 1.7 4 3 9 3s9-1.3 9-3"/>',
    lock:'<rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>',
    table:'<path d="M3 3h18v18H3z"/><path d="M3 9h18M3 15h18M9 3v18"/>',
    play:'<path d="M5 3l14 9-14 9z"/>', plus:'<path d="M12 5v14M5 12h14"/>',
    alert:'<path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"/><path d="M12 9v4M12 17h.01"/>',
    file:'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/>',
    search:'<circle cx="11" cy="11" r="8"/><path d="M21 21l-4.3-4.3"/>',
    code:'<path d="M16 18l6-6-6-6M8 6l-6 6 6 6"/>', grid:'<path d="M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3z"/>',
  };
  const svg = (p,w) => '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="'+(w||1.7)+'" stroke-linecap="round" stroke-linejoin="round">'+p+'</svg>';

  const COLLS = [
    {id:'notes',title:'Notes',count:3,system:false},{id:'tasks',title:'Tasks',count:5,system:false},
    {id:'contacts',title:'Contacts',count:2,system:false},{id:'feedback',title:'Feedback',count:8,system:true},
    {id:'annotations',title:'Annotations',count:14,system:true},
  ];
  const TASK_SCHEMA = [
    {name:'title',type:'text',required:true},{name:'status',type:'text',required:true,default:'todo',indexed:true},
    {name:'estimate_h',type:'real'},{name:'priority',type:'integer',default:2,indexed:true},
    {name:'done',type:'bool',default:false},{name:'due',type:'timestamp',indexed:true},
    {name:'labels',type:'json'},{name:'external_id',type:'text',unique:true},
  ];
  const NOTE_SCHEMA = [
    {name:'title',type:'text',required:true},{name:'content',type:'text'},
    {name:'priority',type:'integer',default:1,indexed:true},{name:'pinned',type:'bool',default:false},
  ];
  const TASKS = [
    {title:'Substrate Phase 1 ports',status:'done',estimate_h:4.0,priority:1,done:true,due:'2026-05-31',labels:['substrate','arc'],external_id:'SHIP-201'},
    {title:'ship-collection integrator',status:'in_progress',estimate_h:6.5,priority:1,done:false,due:'2026-06-01',labels:['collections'],external_id:'SHIP-210'},
    {title:'Cutover + 27 consumers',status:'todo',estimate_h:8.0,priority:2,done:false,due:null,labels:['collections','supervised'],external_id:'SHIP-211'},
    {title:'@ship/db TS type gen',status:'todo',estimate_h:5.0,priority:3,done:false,due:null,labels:['toolchain'],external_id:'SHIP-212'},
    {title:'Collections Data Studio',status:'todo',estimate_h:7.0,priority:2,done:false,due:null,labels:['ui','collections'],external_id:'SHIP-213'},
  ];

  // ── sui-collection-list ───────────────────────────────────────────
  function clist(s) {
    if (s==='loading') return '<div class="cl">'+[0,1,2].map(()=>'<div class="cl-row"><div class="sk" style="width:26px;height:26px;border-radius:6px"></div><div style="flex:1"><div class="sk" style="height:11px;width:40%;margin-bottom:6px"></div><div class="sk" style="height:8px;width:24%"></div></div></div>').join('')+'</div>';
    if (s==='empty') return '<div class="cl"><div class="state"><div class="ic">'+svg(I.db)+'</div><h3>No collections yet</h3><p>Create one to start storing structured data.</p><button class="btn btn-primary btn-sm">'+svg(I.plus)+'New collection</button></div></div>';
    return '<div class="cl">'+COLLS.map((c,i)=>'<div class="cl-row"'+(i===1?' data-selected':'')+'><div class="cl-ic">'+svg(c.system?I.lock:I.db)+'</div>'+
      '<div class="cl-nm"><b>'+c.title+'</b><span>'+c.id+(c.system?' · system · read-only':'')+'</span></div>'+
      '<span class="cl-count">'+c.count+'</span></div>').join('')+'</div>';
  }
  // ── sui-collection-table ──────────────────────────────────────────
  function fmtCell(f, v) {
    if (v===null||v===undefined) return '<span class="cell-null">null</span>';
    if (f.name==='status') return '<span class="status-pill '+v+'">'+v.replace('_',' ')+'</span>';
    if (f.type==='bool') return '<span class="spill"><span class="dot-bool '+(v?'on':'off')+'"></span>'+v+'</span>';
    if (f.type==='json') return v.map(x=>'<span class="tag-pill">'+x+'</span>').join(' ');
    if (f.type==='timestamp') return '<span class="mono">'+String(v).slice(0,10)+'</span>';
    if (f.name==='external_id') return '<span class="mono">'+v+'</span>';
    if (f.type==='integer'||f.type==='real') return '<span class="num" style="display:inline">'+v+'</span>';
    if (f.name==='title') return '<span class="colname">'+v+'</span>';
    return v;
  }
  function ctable(s) {
    if (s==='loading') return '<div class="ctbl-wrap"><table class="ctbl"><thead><tr>'+TASK_SCHEMA.slice(0,6).map(f=>'<th>'+f.name+'</th>').join('')+'</tr></thead><tbody>'+[0,1,2,3].map(()=>'<tr>'+TASK_SCHEMA.slice(0,6).map(()=>'<td><div class="sk" style="height:11px;width:70%"></div></td>').join('')+'</tr>').join('')+'</tbody></table></div>';
    if (s==='empty') return '<div class="ctbl-wrap"><div class="state"><div class="ic">'+svg(I.table)+'</div><h3>No items in tasks</h3><p>Add the first item, or edit the schema.</p><button class="btn btn-primary btn-sm">'+svg(I.plus)+'New item</button></div></div>';
    const live = s==='live';
    const head = '<tr>'+TASK_SCHEMA.map(f=>'<th>'+f.name+'<br><span class="col-type">'+f.type+(f.indexed?' · idx':'')+(f.unique?' · uniq':'')+'</span></th>').join('')+'</tr>';
    const rows = TASKS.map((t,i)=>'<tr'+(live&&i===1?' style="background:color-mix(in oklch,var(--success) 14%,transparent)"':'')+'>'+TASK_SCHEMA.map(f=>'<td'+((f.type==='integer'||f.type==='real')?' class="num"':'')+'>'+fmtCell(f,t[f.name])+'</td>').join('')+'</tr>').join('');
    return '<div class="ctbl-wrap"><table class="ctbl"><thead>'+head+'</thead><tbody>'+rows+'</tbody></table></div>'+
      (live?'<div class="stage-note">Columns derive from the <b>tasks</b> schema. Highlighted row = a <code>Watch</code> merge applied in place.</div>':'<div class="stage-note">Every column is <b>typed from the schema</b> — change the schema, the columns change.</div>');
  }
  // ── sui-item-detail ───────────────────────────────────────────────
  function cdetail(s) {
    const it = TASKS[1];
    if (s==='raw') return '<div class="cdetail"><div class="cd-raw">{\n  "id": "task-collection",\n  "data": {\n    "title": "ship-collection integrator",\n    "status": "in_progress",\n    "estimate_h": 6.5,\n    "priority": 1,\n    "done": false,\n    "due": "2026-06-01T00:00:00Z",\n    "labels": ["collections"],\n    "external_id": "SHIP-210"\n  }\n}</div></div>';
    return '<div class="cdetail">'+TASK_SCHEMA.map(f=>'<div class="cd-row"><dt>'+f.name+'<div class="ty">'+f.type+'</div></dt><dd>'+fmtCell(f,it[f.name])+'</dd></div>').join('')+'</div>';
  }
  // ── sui-item-editor ───────────────────────────────────────────────
  function ceditor(s) {
    const edit = s==='edit';
    const vals = edit?{title:'Auth via JWT',content:'Rotate signing key per space.',priority:5,pinned:true}:{title:'',content:'',priority:1,pinned:false};
    return '<form class="cform" onsubmit="return false">'+NOTE_SCHEMA.map(f=>{
      const lab = '<label>'+f.name+(f.required?' <span class="req">*</span>':'')+' <span class="ty">'+f.type+(f.default!==undefined?' · default '+f.default:'')+'</span></label>';
      if (f.type==='bool') return '<div class="cf-field cf-bool" style="flex-direction:row;align-items:center;gap:9px"><input type="checkbox" '+(vals[f.name]?'checked':'')+'>'+lab+'</div>';
      const t = f.type==='integer'?'number':'text';
      const inv = (!edit&&f.required);
      return '<div class="cf-field">'+lab+'<input class="cf-input'+(inv?' invalid':'')+'" type="'+t+'" value="'+(vals[f.name]??'')+'" placeholder="'+f.name+'">'+(inv?'<div class="cf-err">'+f.name+' is required by the schema.</div>':'')+'</div>';
    }).join('')+'<div class="stage-row" style="margin-top:4px"><button class="btn btn-primary btn-sm">'+(edit?'Save changes':'Create item')+'</button><button class="btn btn-ghost btn-sm">Cancel</button></div>'+
      '<div class="stage-note">Fields + validation are <b>generated from the schema</b> (calls <code>'+(edit?'UpdateItem':'CreateItem')+'</code>). <code>json</code> fields fall back to a raw editor.</div></form>';
  }
  // ── sui-schema-editor ─────────────────────────────────────────────
  function cschema(s) {
    const rows = TASK_SCHEMA.map(f=>{
      const cons = [f.required?'<span class="cons req">req</span>':'',f.unique?'<span class="cons uniq">unique</span>':'',f.indexed?'<span class="cons idx">indexed</span>':'',f.default!==undefined?'<span class="cons">default '+f.default+'</span>':''].join('');
      return '<div class="sf-row"><span class="sf-name">'+f.name+'</span><span class="sf-type">'+f.type+'</span><span class="sf-cons">'+cons+'</span></div>';
    }).join('');
    const warn = s==='reproject' ? '<div class="reproj">'+svg(I.alert)+'<div>Removing field <b>estimate_h</b> will rebuild the projection from the event log. Existing item data is preserved; the column is dropped. Continue?</div></div>' : '';
    return '<div>'+rows+'<div class="stage-row" style="margin-top:8px"><button class="btn btn-soft btn-sm">'+svg(I.plus)+'Add field</button><button class="btn btn-ghost btn-sm">Raw editor</button></div>'+warn+
      '<div class="stage-note">Guided field list + raw editor (toggle). Destructive <code>UpdateSchema</code> surfaces the <b>reproject warning</b> — never hidden.</div></div>';
  }
  // ── sui-query-editor ──────────────────────────────────────────────
  function cqedit(s) {
    if (s==='empty') return '<div class="qed"><div class="qed-head">'+svg(I.code,1.6)+' query · notes</div><div class="qed-body" style="color:var(--muted-foreground)">where(<span class="qed-ac">field</span>, op, value)…</div></div><div class="stage-note">Field/op autocomplete from the schema. Rich Kysely-TS authoring waits on <code>@ship/db</code>.</div>';
    return '<div class="qed"><div class="qed-head">'+svg(I.code,1.6)+' query · notes · <b style="color:var(--foreground)">high_priority_pinned</b></div>'+
      '<div class="qed-body"><span class="fn">where</span>(<span class="qed-ac">priority</span>, <span class="str">\'&gt;=\'</span>, minPriority)<br><span class="fn">.where</span>(<span class="qed-ac">pinned</span>, <span class="str">\'=\'</span>, <span class="kw">true</span>)<br><span class="fn">.orderBy</span>(<span class="qed-ac">priority</span>, <span class="str">\'desc\'</span>)<span class="fn">.limit</span>(50)</div></div>'+
      '<div class="stage-note">Captured as an AST → handed to the runner. Save as named query → <code>.ship/collections/queries/</code>.</div>';
  }
  // ── sui-query-runner ──────────────────────────────────────────────
  function cqrun(s) {
    if (s==='error') return '<div class="qrun"><div class="qrun-meta" style="color:var(--destructive)">'+svg(I.alert,1.6)+' Query rejected</div><div style="padding:14px;font-size:var(--text-sm);color:var(--destructive)">JOINs are not supported. Collections query a single table.</div></div>';
    if (s==='empty') return '<div class="qrun"><div class="qrun-meta"><span><b>0</b> rows</span><span>·</span><span><b>0.6</b> ms</span></div><div class="state" style="padding:36px"><div class="ic">'+svg(I.search)+'</div><h3>0 rows</h3><p>The query ran in 0.6ms and matched nothing.</p></div></div>';
    return '<div class="qrun"><div class="qrun-meta"><span><b>2</b> rows</span><span>·</span><span><b>1.4</b> ms</span><span>·</span><span>cursor: <span class="mono">∅</span></span></div>'+
      '<div class="qrun-sql">SELECT * FROM notes WHERE priority &gt;= ? AND pinned = 1 ORDER BY priority DESC LIMIT 50</div>'+
      '<div class="ctbl-wrap" style="border:0"><table class="ctbl"><thead><tr><th>title<br><span class="col-type">text</span></th><th>priority<br><span class="col-type">integer</span></th><th>pinned<br><span class="col-type">bool</span></th></tr></thead><tbody>'+
      '<tr><td class="colname">Auth via JWT</td><td class="num">5</td><td><span class="spill"><span class="dot-bool on"></span>true</span></td></tr>'+
      '<tr><td class="colname">Checkout notes</td><td class="num">3</td><td><span class="spill"><span class="dot-bool on"></span>true</span></td></tr>'+
      '</tbody></table></div></div><div class="stage-note">Typed results + row count + elapsed. Result columns are typed from the schema too.</div>';
  }
  // ── sui-collection-picker ─────────────────────────────────────────
  function cpicker() {
    return '<div class="stage-row"><select class="sel" style="min-width:200px"><option>Notes · 3 items</option><option>Tasks · 5 items</option><option>Contacts · 2 items</option></select><span class="stage-note" style="margin:0">Compact selector — composes into upload, query, anywhere "pick a collection" is needed.</span></div>';
  }
  const watch = s => '<div class="stage-row"><button class="wb" data-state="'+s+'"><span class="wb-dot"></span><span>'+(s==='live'?'Live':s==='reconnecting'?'Reconnecting…':'Paused')+'</span></button></div>';

  const api = (p,e,s) => ({props:p||[],events:e||[],slots:s||[]});
  ComponentViewer.mount({
    eyebrow:'domain handoff · collections · component-driven',
    title:'Collections — Data Studio',
    domainAccent:'var(--app-accent-collections)',
    intro:'The reference two-layer domain: a <b>component set</b> that assembles into the <b>Data Studio</b> (explore · schema · query). The one hard rule — <b>every surface is typed from the collection schema</b> — holds across all of these. Switch <b>ship themes</b> top-right.',
    storagePrefix:'colcomp',
    links:[{label:'handoff.md', href:'handoff.md', ghost:true},{label:'spec.md', href:'spec.md', ghost:true},{label:'Artifacts components →', href:'../artifacts/components.html', ghost:true, icon:svg(I.grid,1.8)}],
    components:[
      {id:'list',tag:'sui-collection-list',badges:['new'],rpc:['List'],state:'ok',states:['ok','loading','empty'],render:clist,
        purpose:'Index of the space\u2019s collections; flags <b>system</b> collections (read-only). Selection drives every other surface.',
        api:api([['selected-id','string','Active collection.'],['show-system','boolean','Include system collections.']],[['sui-select','{ id }','Collection chosen.']])},
      {id:'table',tag:'sui-collection-table',badges:['new'],rpc:['QueryItems','Watch','Get'],state:'ok',states:['ok','loading','empty','live'],render:ctable,
        purpose:'The data explorer — rows with <b>columns derived from the schema</b>, cursor-paginated, <span class="calls">live</span>. Consumes <code>Watch</code> frames to stay current.',
        api:api([['collection','string','Which collection to browse.'],['cursor','string','Pagination cursor.'],['live','boolean','Apply Watch patches in place.']],[['sui-row-click','{ id }','Open item detail.']])},
      {id:'item-detail',tag:'sui-item-detail',badges:['new'],rpc:['GetItem'],state:'typed',states:['typed','raw'],render:cdetail,
        purpose:'One item rendered by its <b>typed fields</b>, with a raw-JSON fallback. Drill-in from a table row.',
        api:api([['collection','string',''],['id','string','Item id.'],['raw','boolean','Force raw-JSON view.']])},
      {id:'item-editor',tag:'sui-item-editor',badges:['new'],rpc:['CreateItem','UpdateItem'],state:'create',states:['create','edit'],render:ceditor,
        purpose:'Create / edit an item as a <b>form generated from the schema</b>; validates client-side (mirrors server rejection).',
        api:api([['collection','string',''],['id','string','Omit to create.']],[['sui-submit','{ data }','Create or update.']])},
      {id:'schema-editor',tag:'sui-schema-editor',badges:['new'],rpc:['CreateSchema','UpdateSchema'],state:'fields',states:['fields','reproject'],render:cschema,
        purpose:'Author/edit the schema — name · type · required · indexed · unique · default. Guided list + raw (toggle). Surfaces the <b>reproject warning</b> on destructive change.',
        api:api([['collection','string',''],['mode','guided|raw','Editor surface.']],[['sui-save','{ schema }','calls Create/UpdateSchema.']])},
      {id:'query-editor',tag:'sui-query-editor',badges:['new'],rpc:['none — emits AST'],state:'written',states:['empty','written'],render:cqedit,
        purpose:'Write a query with <b>field/op autocomplete from the schema</b>. Produces a captured AST handed to the runner; can save as a named query.',
        api:api([['collection','string',''],['value','QueryAST','Controlled query.']],[['sui-run','QueryAST','Run requested.'],['sui-save','{ name, ast }','Save named query.']])},
      {id:'query-runner',tag:'sui-query-runner',badges:['new'],rpc:['QueryItems'],state:'results',states:['results','empty','error'],render:cqrun,
        purpose:'Run a query and render <b>typed results + row count + elapsed</b>. Pairs with the editor for the test loop.',
        api:api([['ast','QueryAST','Inline AST, or…'],['named-query','string','…a saved query.']],[['sui-cursor','{ next }','Page through results.']])},
      {id:'collection-picker',tag:'sui-collection-picker',badges:['new'],rpc:['List'],state:'idle',states:['idle'],render:cpicker,
        purpose:'Compact collection selector for anywhere another surface needs "pick a collection" (e.g. artifact upload).',
        api:api([['value','string','Selected id.']],[['sui-change','{ id }','Selection changed.']])},
      {id:'watch-badge',tag:'sui-watch-badge',badges:['shared'],rpc:['Watch status'],state:'live',states:['live','reconnecting','paused'],render:watch,
        purpose:'<b>Shared with artifacts</b> — the same live / reconnecting / paused indicator on the table + detail.',
        api:api([['state','live|reconnecting|paused',''],['label','boolean','Show text label.']])},
    ],
  });
})();
