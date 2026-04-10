import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import { DaemonRequired } from '../DaemonRequired'

afterEach(cleanup)

describe('DaemonRequired', () => {
  it('renders a message explaining the daemon is required', () => {
    render(<DaemonRequired />)
    expect(screen.getByText(/daemon/i)).toBeTruthy()
  })

  it('shows connection instructions or install prompt', () => {
    render(<DaemonRequired />)
    // Should tell the user how to connect or install
    expect(
      screen.getByText(/connect/i) || screen.getByText(/install/i),
    ).toBeTruthy()
  })

  it('renders without crashing when no props are given', () => {
    const { container } = render(<DaemonRequired />)
    expect(container.innerHTML).not.toBe('')
  })

  it('is visually distinct — has a recognizable role or landmark', () => {
    render(<DaemonRequired />)
    // Should be a status region or similar accessible landmark
    const el =
      screen.queryByRole('status') ??
      screen.queryByRole('alert') ??
      screen.queryByRole('region')
    expect(el).toBeTruthy()
  })
})
