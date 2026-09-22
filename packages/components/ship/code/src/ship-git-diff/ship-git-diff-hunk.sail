<!--
  <ship-git-diff-hunk> — hunk header line (`@@ -a,b +c,d @@`).
-->
<scope>
  props: {
    header = ''
  }
  style hunk = { type({ family: mono, size: xs }).fg(info).pad({ x: 3, y: 1 }).bg(muted).size({ w: max, min-w: 0 }) }
</scope>

<span style(hunk)>{header()}</span>
