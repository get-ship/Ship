const TONES_CSS =
  ":host([tone='default'])     { --tone: var(--foreground);       --tone-fg: var(--foreground);       --tone-contrast: var(--background); }\n" +
  ":host([tone='primary'])     { --tone: var(--primary);          --tone-fg: var(--primary);          --tone-contrast: var(--primary-foreground); }\n" +
  ":host([tone='info'])        { --tone: var(--info);             --tone-fg: var(--info);             --tone-contrast: var(--info-foreground); }\n" +
  ":host([tone='success'])     { --tone: var(--success);          --tone-fg: var(--success);          --tone-contrast: var(--success-foreground); }\n" +
  ":host([tone='warning'])     { --tone: var(--warning);          --tone-fg: var(--warning);          --tone-contrast: var(--warning-foreground); }\n" +
  ":host([tone='destructive']) { --tone: var(--destructive);      --tone-fg: var(--destructive);      --tone-contrast: var(--destructive-foreground); }\n" +
  ":host([tone='accent'])      { --tone: var(--accent);           --tone-fg: var(--accent);           --tone-contrast: var(--accent-foreground); }\n" +
  ":host([tone='muted'])       { --tone: var(--muted-foreground); --tone-fg: var(--muted-foreground); --tone-contrast: var(--background); }\n"

function makeSuiTonesSheet(): CSSStyleSheet | null {
  if (typeof CSSStyleSheet === 'undefined') return null
  try {
    const sheet = new CSSStyleSheet()
    sheet.replaceSync(TONES_CSS)
    return sheet
  } catch {
    return null
  }
}

export const suiTonesSheet: CSSStyleSheet | null = makeSuiTonesSheet()
