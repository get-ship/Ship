<scope>
  props: {
    rows = 24
    cols = 80
    'font-size' = 0
    readonly = false
    theme: 'light' | 'dark' | 'auto' = 'auto'
    fit = false
  }
  style stub = { size({ min-h: 24 }).bg(surface).border({ width: 1, color: border }).radius(md).pad(2) }
  style stubText = { type({ family: mono, size: xs }).fg(muted-foreground) }
</scope>
<div data-test="ship-terminal-stub" style(stub)>
  <span style(stubText)>terminal (stub)</span>
</div>
