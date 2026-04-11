import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import { ProvidersSection } from '../sections/ProvidersSection'

// Mock ProviderLogo to avoid image loading in tests
vi.mock('#/features/compiler/ProviderLogo', () => ({
  ProviderLogo: ({ provider }: { provider: string }) => <span data-testid={`logo-${provider}`}>{provider}</span>,
}))

// Mock ProviderConfigModal to avoid deep dependency chains
vi.mock('../dialogs/ProviderConfigModal', () => ({
  ProviderConfigModal: () => <div data-testid="config-modal" />,
}))

// Mock useProviderData to avoid daemon queries
vi.mock('#/features/agents/useProviderData', () => ({
  useProviderData: () => ({
    providers: [
      { id: 'claude', available: true, binary: 'claude', version: '1.0.0' },
      { id: 'gemini', available: true, binary: 'gemini', version: '2.1.0' },
      { id: 'codex', available: false, binary: 'codex', version: null },
    ],
    defaultProvider: 'claude',
    defaultModel: 'sonnet',
    isLoading: false,
  }),
}))

afterEach(cleanup)

const baseProps = {
  providers: ['claude', 'gemini', 'codex'],
  model: null as string | null,
  hooks: [],
  providerSettings: {},
  onChangeModel: vi.fn(),
  onChangeHooks: vi.fn(),
  onChangeProviderSettings: vi.fn(),
}

describe('ProvidersSection', () => {
  it('renders one card per provider', () => {
    render(<ProvidersSection {...baseProps} />)
    const buttons = screen.getAllByRole('button')
    expect(buttons).toHaveLength(3)
    expect(screen.getByText('Claude')).toBeTruthy()
    expect(screen.getByText('Gemini')).toBeTruthy()
    expect(screen.getByText('Codex')).toBeTruthy()
  })

  it('returns null when providers is empty', () => {
    const { container } = render(<ProvidersSection {...baseProps} providers={[]} />)
    expect(container.innerHTML).toBe('')
  })

  it('shows model in summary when set', () => {
    render(<ProvidersSection {...baseProps} model="claude-sonnet-4" />)
    expect(screen.getAllByText(/claude-sonnet-4/).length).toBeGreaterThan(0)
  })

  it('shows defaults when no customizations', () => {
    render(<ProvidersSection {...baseProps} />)
    const defaults = screen.getAllByText('defaults')
    expect(defaults.length).toBe(3)
  })

  it('shows hook count for all providers', () => {
    render(
      <ProvidersSection
        {...baseProps}
        hooks={[
          { id: '1', trigger: 'PreToolUse', command: 'echo test' },
          { id: '2', trigger: 'Stop', command: 'echo done' },
        ]}
      />,
    )
    const hookTexts = screen.getAllByText(/2 hooks/)
    expect(hookTexts.length).toBe(3)
  })

  it('derives provider count from props, not hardcoded', () => {
    render(<ProvidersSection {...baseProps} providers={['cursor']} />)
    expect(screen.getByText('Cursor')).toBeTruthy()
    expect(screen.queryByText('Claude')).toBeNull()
  })

  it('shows provider settings count in summary', () => {
    render(
      <ProvidersSection
        {...baseProps}
        providers={['claude']}
        providerSettings={{ claude: { theme: 'dark', auto_updates: true, model: 'test' } }}
      />,
    )
    expect(screen.getByText(/3 custom settings/)).toBeTruthy()
  })

  it('renders correct count text in section header', () => {
    render(<ProvidersSection {...baseProps} />)
    expect(screen.getByText('3 providers')).toBeTruthy()
  })

  it('renders singular provider text', () => {
    render(<ProvidersSection {...baseProps} providers={['claude']} />)
    expect(screen.getByText('1 provider')).toBeTruthy()
  })

  it('shows default badge on default provider', () => {
    render(<ProvidersSection {...baseProps} />)
    expect(screen.getByText('default')).toBeTruthy()
  })

  it('shows availability status from API data', () => {
    render(<ProvidersSection {...baseProps} />)
    expect(screen.getByText('not installed')).toBeTruthy()
    expect(screen.getByText('1.0.0')).toBeTruthy()
    expect(screen.getByText('2.1.0')).toBeTruthy()
  })
})
