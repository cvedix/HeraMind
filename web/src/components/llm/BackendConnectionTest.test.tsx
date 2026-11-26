import React from 'react'
import { act, fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { BackendConnectionTest } from './BackendConnectionTest'
import type { BackendTestResult } from '@/types'

describe('BackendConnectionTest', () => {
  it('shows the actual response and latency, preventing duplicate requests while pending', async () => {
    let finish!: (result: BackendTestResult) => void
    const onTest = vi.fn(() => new Promise<BackendTestResult>(resolve => { finish = resolve }))
    render(<BackendConnectionTest onTest={onTest} />)
    fireEvent.click(screen.getByRole('button', { name: 'llm.testResponse' }))
    const pending = screen.getByRole('button', { name: 'llm.testResponsePending' })
    expect(pending).toBeDisabled()
    fireEvent.click(pending)
    expect(onTest).toHaveBeenCalledTimes(1)
    await act(async () => finish({ success: true, response: 'OK from the model', latency_ms: 321 }))
    expect(screen.getByRole('status')).toHaveTextContent('OK from the model')
    expect(screen.getByRole('status')).toHaveTextContent('321ms')
    expect(screen.getByRole('button', { name: 'llm.testResponse' })).toBeEnabled()
  })

  it('shows provider errors and allows another test', async () => {
    const onTest = vi.fn().mockResolvedValueOnce({ success: false, error: 'Model is not available' })
      .mockResolvedValueOnce({ success: true, response: 'OK', latency_ms: 42 })
    render(<BackendConnectionTest onTest={onTest} />)
    fireEvent.click(screen.getByRole('button'))
    expect(await screen.findByRole('alert')).toHaveTextContent('Model is not available')
    fireEvent.click(screen.getByRole('button'))
    expect(await screen.findByRole('status')).toHaveTextContent('OK')
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })

  it('handles network exceptions without leaving the control disabled', async () => {
    render(<BackendConnectionTest onTest={vi.fn().mockRejectedValue(new Error('Network timeout'))} />)
    fireEvent.click(screen.getByRole('button'))
    expect(await screen.findByRole('alert')).toHaveTextContent('Network timeout')
    expect(screen.getByRole('button')).toBeEnabled()
  })
})
