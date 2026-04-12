// Pipeline Monitor — first web component view.
// No build step, no framework. Uses injected shipApi.

class ShipPipelineMonitor extends HTMLElement {
  connectedCallback() {
    this.attachShadow({ mode: 'open' })
    this.shadowRoot.innerHTML = `
      <style>
        :host { display: block; font-family: system-ui, -apple-system, sans-serif; color: var(--foreground, #e4e4e7); }
        .container { padding: 16px; }
        .title { font-size: 14px; font-weight: 600; margin-bottom: 12px; }
        .empty { font-size: 13px; color: var(--muted, #71717a); }
        .job { border: 1px solid var(--border, #27272a); border-radius: 8px; padding: 12px; margin-bottom: 8px; }
        .job-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
        .job-slug { font-size: 13px; font-weight: 600; }
        .job-status { font-size: 11px; padding: 2px 8px; border-radius: 4px; font-weight: 500; }
        .status-dispatched { background: #0c4a6e; color: #7dd3fc; }
        .status-completed { background: #064e3b; color: #6ee7b7; }
        .status-failed { background: #7f1d1d; color: #fca5a5; }
        .status-pending { background: #27272a; color: #a1a1aa; }
        .status-merged { background: #3b0764; color: #c4b5fd; }
        .pipeline { display: flex; align-items: center; gap: 0; overflow-x: auto; padding: 4px 0; }
        .step { display: flex; flex-direction: column; gap: 2px; border: 1px solid var(--border, #27272a); border-radius: 6px; padding: 8px 10px; min-width: 100px; font-size: 11px; }
        .step-completed { border-color: #059669; background: rgba(5, 150, 105, 0.05); }
        .step-running { border-color: #0284c7; background: rgba(2, 132, 199, 0.05); }
        .step-failed { border-color: #dc2626; background: rgba(220, 38, 38, 0.05); }
        .step-name { font-weight: 600; }
        .step-type { font-size: 9px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--muted, #71717a); }
        .step-evidence { font-size: 10px; color: var(--muted, #71717a); margin-top: 2px; }
        .arrow { padding: 0 4px; color: var(--muted, #71717a); font-size: 12px; flex-shrink: 0; }
        .loading { font-size: 13px; color: var(--muted, #71717a); padding: 16px; }
        .error { font-size: 13px; color: #fca5a5; padding: 16px; }
      </style>
      <div class="container">
        <div class="loading">Loading jobs...</div>
      </div>
    `
    this._load()
    this._interval = setInterval(() => this._load(), 5000)
  }

  disconnectedCallback() {
    if (this._interval) clearInterval(this._interval)
    if (this._unsub) this._unsub()
  }

  async _load() {
    const api = this.shipApi
    if (!api) {
      this._render('<div class="error">No API available</div>')
      return
    }

    try {
      const jobs = await api.getJobs()
      const pipelineJobs = jobs.filter(j => j.steps && j.steps.steps && j.steps.steps.length > 0)

      if (pipelineJobs.length === 0) {
        this._render('<div class="empty">No pipeline jobs found.</div>')
        return
      }

      const html = pipelineJobs.map(job => this._renderJob(job)).join('')
      this._render(html)
    } catch (err) {
      this._render(`<div class="error">Failed to load: ${err.message}</div>`)
    }
  }

  _renderJob(job) {
    const steps = job.steps?.steps || []
    const completed = job.completed_steps || []
    const current = job.current_step

    const stepsHtml = steps.map((step, i) => {
      let cls = 'step'
      let status = 'pending'
      if (completed.includes(step.id)) { cls += ' step-completed'; status = 'done' }
      else if (current === step.id) {
        cls += job.status === 'failed' ? ' step-failed' : ' step-running'
        status = job.status === 'failed' ? 'failed' : 'running'
      }

      const arrow = i < steps.length - 1 ? '<span class="arrow">\u2192</span>' : ''

      return `
        <div class="${cls}">
          <span class="step-name">${step.id}</span>
          <span class="step-type">${step.type}</span>
          <span class="step-evidence">${status}</span>
        </div>
        ${arrow}
      `
    }).join('')

    const statusCls = `job-status status-${job.status}`

    return `
      <div class="job">
        <div class="job-header">
          <span class="job-slug">${job.slug}</span>
          <span class="${statusCls}">${job.status}</span>
        </div>
        <div class="pipeline">${stepsHtml}</div>
      </div>
    `
  }

  _render(content) {
    const container = this.shadowRoot?.querySelector('.container')
    if (container) container.innerHTML = `<div class="title">Pipeline Monitor</div>${content}`
  }
}

if (!customElements.get('ship-pipeline-monitor')) {
  customElements.define('ship-pipeline-monitor', ShipPipelineMonitor)
}
