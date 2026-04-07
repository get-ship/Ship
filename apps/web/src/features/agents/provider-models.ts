// Known models per provider -- fixed UI constants for autocomplete.
// These are the models available through each provider's API.

export const PROVIDER_MODELS: Record<string, string[]> = {
  claude: [
    'claude-opus-4-20250514',
    'claude-sonnet-4-20250514',
    'claude-sonnet-4-6-20250627',
    'claude-haiku-4-20250414',
    'claude-3-5-sonnet-20241022',
    'claude-3-5-haiku-20241022',
  ],
  gemini: [
    'gemini-2.5-pro',
    'gemini-2.5-flash',
    'gemini-2.0-flash',
    'gemini-2.0-flash-lite',
  ],
  codex: [
    'codex-mini-latest',
    'o4-mini',
    'o3',
    'gpt-4.1',
    'gpt-4.1-mini',
  ],
  cursor: [
    'claude-sonnet-4-20250514',
    'claude-opus-4-20250514',
    'gpt-4.1',
    'gemini-2.5-pro',
  ],
  opencode: [
    'claude-sonnet-4-20250514',
    'claude-opus-4-20250514',
    'gemini-2.5-pro',
    'gpt-4.1',
    'o4-mini',
  ],
}
