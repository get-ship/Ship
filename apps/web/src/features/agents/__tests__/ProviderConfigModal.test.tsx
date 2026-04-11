import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup, fireEvent } from '@testing-library/react'
import { ProviderConfigModal } from '../dialogs/ProviderConfigModal'

// Mock sub-tabs to avoid deep dependency chains
vi.mock('../dialogs/ProviderGeneralTab', () => ({
  ProviderGeneralTab: () => (
    <div data-testid="general-tab">
      <span>Model override</span>
    </div>
  ),
}))

vi.mock('../dialogs/ProviderHooksTab', () => ({
  ProviderHooksTab: () => <div data-testid="hooks-tab">Hooks editor</div>,
}))

vi.mock('../dialogs/ProviderPassthroughTab', () => ({
  ProviderPassthroughTab: () => (
    <div data-testid="passthrough-tab">
      <span>Provider-specific settings</span>
    </div>
  ),
}))

// Mock ProviderLogo to avoid image loading
vi.mock('#/features/compiler/ProviderLogo', () => ({
  ProviderLogo: ({ provider }: { provider: string }) => <span data-testid="provider-logo">{provider}</span>,
}))

afterEach(cleanup)

const baseProps = {
  open: true,
  onOpenChange: vi.fn(),
  provider: 'claude',
  model: null as string | null,
  hooks: [],
  providerSettings: {} as Record<string, unknown>,
  defaultModel: 'sonnet',
  onSave: vi.fn(),
}

describe('ProviderConfigModal', () => {
  it('renders provider name in header', () => {
    render(<ProviderConfigModal {...baseProps} />)
    expect(screen.getByText('Claude Configuration')).toBeTruthy()
  })

  it('renders nothing when closed', () => {
    const { container } = render(<ProviderConfigModal {...baseProps} open={false} />)
    expect(container.innerHTML).toBe('')
  })

  it('shows General tab content by default', () => {
    render(<ProviderConfigModal {...baseProps} />)
    expect(screen.getByTestId('general-tab')).toBeTruthy()
    expect(screen.getByText('Model override')).toBeTruthy()
  })

  it('shows Hooks tab for all providers', () => {
    for (const provider of ['claude', 'gemini', 'codex', 'cursor', 'opencode']) {
      const { unmount } = render(<ProviderConfigModal {...baseProps} provider={provider} />)
      expect(screen.getByText('Hooks')).toBeTruthy()
      unmount()
    }
  })

  it('shows Passthrough tab button', () => {
    render(<ProviderConfigModal {...baseProps} />)
    expect(screen.getByText('Passthrough')).toBeTruthy()
  })

  it('switches to Passthrough tab on click', () => {
    render(<ProviderConfigModal {...baseProps} />)
    fireEvent.click(screen.getByText('Passthrough'))
    expect(screen.getByTestId('passthrough-tab')).toBeTruthy()
    expect(screen.getByText('Provider-specific settings')).toBeTruthy()
  })

  it('switches to Hooks tab on click', () => {
    render(<ProviderConfigModal {...baseProps} provider="claude" />)
    fireEvent.click(screen.getByText('Hooks'))
    expect(screen.getByTestId('hooks-tab')).toBeTruthy()
  })

  it('calls onSave with config when Save is clicked', () => {
    const onSave = vi.fn()
    render(<ProviderConfigModal {...baseProps} onSave={onSave} />)
    fireEvent.click(screen.getByText('Save'))
    expect(onSave).toHaveBeenCalledTimes(1)
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        model: null,
        hooks: [],
        providerSettings: {},
      }),
    )
  })

  it('calls onOpenChange(false) when Cancel is clicked', () => {
    const onOpenChange = vi.fn()
    render(<ProviderConfigModal {...baseProps} onOpenChange={onOpenChange} />)
    fireEvent.click(screen.getByText('Cancel'))
    expect(onOpenChange).toHaveBeenCalledWith(false)
  })
})
