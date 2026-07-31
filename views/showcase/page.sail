<div>
  <header>
    <h1>sui/charts</h1>
    <p>Pack <strong>sui/charts</strong> — SVG-rendered chart primitives. Each card shows one component with representative prop/variant combinations.</p>
  </header>

  <!-- ── sui-area-chart ─────────────────────────────────────────────────── -->
  <section>
    <h2>sui-area-chart</h2>

    <div>
      <div>
        <h3>single series, no grid</h3>
        <sail-area-chart
          series='[{"name":"Revenue","data":[12,28,18,42,35,60,55]}]'
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          label="revenue area chart">
        </sail-area-chart>
      </div>

      <div>
        <h3>two series + grid + points</h3>
        <sail-area-chart
          series='[{"name":"Downloads","data":[5,14,9,22,18,30,27],"tone":"primary"},{"name":"Uploads","data":[2,6,4,10,8,14,12],"tone":"success"}]'
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          grid
          points
          label="downloads vs uploads area chart">
          <span slot="legend-entry" style="color:var(--primary)">Downloads</span>
          <span>Uploads</span>
        </sail-area-chart>
      </div>

      <div>
        <h3>loading state</h3>
        <sail-area-chart loading label="loading area chart"></sail-area-chart>
      </div>
    </div>
  </section>

  <!-- ── sui-line-chart ─────────────────────────────────────────────────── -->
  <section>
    <h2>sui-line-chart</h2>

    <div>
      <div>
        <h3>single series</h3>
        <sail-line-chart
          series='[{"name":"Latency","data":[80,72,95,60,88,74,65]}]'
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          label="latency line chart">
        </sail-line-chart>
      </div>

      <div>
        <h3>two series + grid + points</h3>
        <sail-line-chart
          series='[{"name":"p50","data":[40,38,45,36,42,39,37],"tone":"primary"},{"name":"p99","data":[120,115,140,105,130,118,110],"tone":"destructive"}]'
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          grid
          points
          label="p50 vs p99 line chart">
        </sail-line-chart>
      </div>

      <div>
        <h3>loading state</h3>
        <sail-line-chart loading label="loading line chart"></sail-line-chart>
      </div>
    </div>
  </section>

  <!-- ── sui-bar-chart ──────────────────────────────────────────────────── -->
  <section>
    <h2>sui-bar-chart</h2>

    <div>
      <div>
        <h3>vertical · tone=primary</h3>
        <sail-bar-chart
          data="12,28,18,42,35,60,55"
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          tone="primary"
          orientation="vertical"
          label="weekly bar chart">
        </sail-bar-chart>
      </div>

      <div>
        <h3>vertical · tone=success</h3>
        <sail-bar-chart
          data="5,14,9,22,18,30,27"
          labels="Mon,Tue,Wed,Thu,Fri,Sat,Sun"
          tone="success"
          orientation="vertical"
          label="success bar chart">
        </sail-bar-chart>
      </div>

      <div>
        <h3>horizontal · tone=warning</h3>
        <sail-bar-chart
          data="30,55,40,70,25"
          labels="Alpha,Beta,Gamma,Delta,Epsilon"
          tone="warning"
          orientation="horizontal"
          label="horizontal bar chart">
        </sail-bar-chart>
      </div>

      <div>
        <h3>horizontal · tone=destructive</h3>
        <sail-bar-chart
          data="8,16,12,20,5"
          labels="Q1,Q2,Q3,Q4,Q5"
          tone="destructive"
          orientation="horizontal"
          label="destructive bar chart">
        </sail-bar-chart>
      </div>

      <div>
        <h3>loading state</h3>
        <sail-bar-chart loading label="loading bar chart"></sail-bar-chart>
      </div>
    </div>
  </section>

  <!-- ── sui-donut ──────────────────────────────────────────────────────── -->
  <section>
    <h2>sui-donut</h2>

    <div>
      <div>
        <h3>default size, center label</h3>
        <sail-donut
          data='[{"label":"TypeScript","value":58,"tone":"primary"},{"label":"Rust","value":30,"tone":"success"},{"label":"Other","value":12,"tone":"muted"}]'
          label="language breakdown donut">
          <span>58%</span>
        </sail-donut>
      </div>

      <div>
        <h3>larger size, thicker ring</h3>
        <sail-donut
          data='[{"label":"CPU","value":42,"tone":"destructive"},{"label":"Memory","value":35,"tone":"warning"},{"label":"Free","value":23,"tone":"muted"}]'
          size="160"
          thickness="24"
          label="resource usage donut">
          <span style="font-size:var(--text-sm)">Resources</span>
        </sail-donut>
      </div>

      <div>
        <h3>compact, thin ring</h3>
        <sail-donut
          data='[{"label":"Done","value":72,"tone":"success"},{"label":"Remaining","value":28,"tone":"muted"}]'
          size="80"
          thickness="8"
          label="progress donut">
          <span style="font-size:var(--text-xs)">72%</span>
        </sail-donut>
      </div>

      <div>
        <h3>loading state</h3>
        <sail-donut loading label="loading donut"></sail-donut>
      </div>
    </div>
  </section>

  <!-- ── sui-gauge ──────────────────────────────────────────────────────── -->
  <section>
    <h2>sui-gauge</h2>

    <div>
      <div>
        <h3>tone=primary, 65%</h3>
        <sail-gauge
          value="65"
          min="0"
          max="100"
          tone="primary"
          label="cpu gauge">
          <span>65%</span>
        </sail-gauge>
      </div>

      <div>
        <h3>tone=success, 85%</h3>
        <sail-gauge
          value="85"
          min="0"
          max="100"
          tone="success"
          label="health gauge">
          <span>85%</span>
        </sail-gauge>
      </div>

      <div>
        <h3>tone=warning, 45%</h3>
        <sail-gauge
          value="45"
          min="0"
          max="100"
          tone="warning"
          label="memory gauge">
          <span>45%</span>
        </sail-gauge>
      </div>

      <div>
        <h3>tone=destructive, 90%</h3>
        <sail-gauge
          value="90"
          min="0"
          max="100"
          tone="destructive"
          label="disk gauge">
          <span>90%</span>
        </sail-gauge>
      </div>

      <div>
        <h3>threshold-driven color, high=70</h3>
        <sail-gauge
          value="55"
          min="0"
          max="100"
          low-threshold="30"
          high-threshold="70"
          size="160"
          thickness="16"
          label="threshold gauge">
          <span>55 / 100</span>
        </sail-gauge>
      </div>

      <div>
        <h3>loading state</h3>
        <sail-gauge loading label="loading gauge"></sail-gauge>
      </div>
    </div>
  </section>

  <!-- ── sui-heatmap ────────────────────────────────────────────────────── -->
  <section>
    <h2>sui-heatmap</h2>

    <div>
      <div>
        <h3>tone=primary, 4×7 grid</h3>
        <sail-heatmap
          data="0,3,7,2,5,8,4,1,6,9,3,7,2,5,0,4,8,6,3,7,2,5,9,4,1,6,3,8"
          rows="Mon,Tue,Wed,Thu"
          cols="W1,W2,W3,W4,W5,W6,W7"
          tone="primary"
          cell-size="24"
          label="activity heatmap">
        </sail-heatmap>
      </div>

      <div>
        <h3>tone=success, compact cells</h3>
        <sail-heatmap
          data="0,2,5,8,3,6,9,1,4,7,2,5,8,3,6,9,1,4,7,2,5,8,3,6,9,1,4,7"
          rows="Mon,Tue,Wed,Thu"
          cols="W1,W2,W3,W4,W5,W6,W7"
          tone="success"
          cell-size="16"
          gap="2"
          label="success heatmap">
        </sail-heatmap>
      </div>

      <div>
        <h3>tone=warning</h3>
        <sail-heatmap
          data="1,3,6,2,8,4,9,0,5,7,3,6,2,8,4,9,0,5,7,3,6,2,8,4,9,0,5,7"
          rows="Mon,Tue,Wed,Thu"
          cols="W1,W2,W3,W4,W5,W6,W7"
          tone="warning"
          cell-size="20"
          label="warning heatmap">
        </sail-heatmap>
      </div>

      <div>
        <h3>loading state</h3>
        <sail-heatmap loading label="loading heatmap"></sail-heatmap>
      </div>
    </div>
  </section>

  <!-- ── sui-sparkline ──────────────────────────────────────────────────── -->
  <section>
    <h2>sui-sparkline</h2>

    <div>
      <div>
        <h3>tone variants</h3>
        <div>
          <span>primary</span>
          <sail-sparkline data="4,7,3,9,5,11,8" tone="primary" width="120" height="32"></sail-sparkline>
          <span>success</span>
          <sail-sparkline data="2,5,8,6,9,7,10" tone="success" width="120" height="32"></sail-sparkline>
          <span>warning</span>
          <sail-sparkline data="9,6,8,4,7,5,6" tone="warning" width="120" height="32"></sail-sparkline>
          <span>destructive</span>
          <sail-sparkline data="3,8,5,10,6,9,4" tone="destructive" width="120" height="32"></sail-sparkline>
          <span>accent</span>
          <sail-sparkline data="6,4,9,3,8,5,7" tone="accent" width="120" height="32"></sail-sparkline>
          <span>muted</span>
          <sail-sparkline data="5,7,4,8,6,9,3" tone="muted" width="120" height="32"></sail-sparkline>
        </div>
      </div>

      <div>
        <h3>fill=true vs fill=false</h3>
        <div>
          <span>no fill</span>
          <sail-sparkline data="4,7,3,9,5,11,8" tone="primary" width="120" height="32"></sail-sparkline>
          <span>fill</span>
          <sail-sparkline data="4,7,3,9,5,11,8" tone="primary" fill width="120" height="32"></sail-sparkline>
        </div>
      </div>

      <div>
        <h3>loading state</h3>
        <sail-sparkline loading tone="primary" width="120" height="32"></sail-sparkline>
      </div>
    </div>
  </section>
</div>
