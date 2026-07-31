<!--
  <ship-git-diff-hunk> — SUI port of <ship-git-diff-hunk>.

  Renders a single hunk header line (e.g. "@@ -10,7 +10,8 @@"). Purely
  presentational — no controller. The daemon emits the header text into
  the `header` attribute; the template renders it inside a mono pill.

  Prop / event / slot parity vs <ship-git-diff-hunk>
  ─────────────────────────────────────────────────────────────────────────────
  Lit                          SUI                      Notes
  ─────────────────────────────────────────────────────────────────────────────
  header (string, '')          header (string, '',      ✓ reflects
                                  reflect)
  ─────────────────────────────────────────────────────────────────────────────
-->

<!-- <prop name="header" type="string" default="" reflect /> -->

<!-- style block removed: use sx / external CSS -->

{header()}
