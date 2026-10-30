import React from 'react'
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { CloudAiAddDialog, type CloudAiEditTarget } from './CloudAiAddDialog'

const { listModels } = vi.hoisted(() => ({ listModels: vi.fn() }))
vi.mock('@/lib/api', () => ({ api: { listAgentRouterModels: listModels } }))
vi.mock('@/components/plugins/UniversalPluginConfigDialog', () => ({ API_KEY_MASK: '********' }))
vi.mock('./AgentRouterServiceDialog', () => ({ AgentRouterServiceDialog: ({ onDirect }: { onDirect: () => void }) =>
  <div>OpenClaw setup<button onClick={onDirect}>Direct connection</button></div>,
}))
vi.mock('@/components/dialog/UnifiedFormDialog', () => ({
  UnifiedFormDialog: ({ open, children, onSubmit, submitDisabled }: {
    open: boolean; children: React.ReactNode; onSubmit: () => void; submitDisabled: boolean
  }) => open ? <div>{children}<button disabled={submitDisabled} onClick={onSubmit}>Save</button></div> : null,
}))
vi.mock('@/components/ui/select', () => ({
  Select: ({ value, onValueChange, children }: { value: string; onValueChange: (value: string) => void; children: React.ReactNode }) =>
    <select value={value} onChange={event => onValueChange(event.target.value)}>{children}</select>,
  SelectTrigger: () => null,
  SelectValue: () => null,
  SelectContent: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  SelectItem: ({ value, children }: { value: string; children: React.ReactNode }) => <option value={value}>{children}</option>,
}))

const editing: CloudAiEditTarget = {
  id: 'saved-router', name: 'AgentRouter', backend_type: 'openai',
  endpoint: 'https://agentrouter.org/v1', model: 'glm-5.3', api_key_configured: true,
}

function setup(existing?: CloudAiEditTarget) {
  const props = { open: true, onOpenChange: vi.fn(), onSubmit: vi.fn().mockResolvedValue(undefined), editing: existing }
  return { ...render(<CloudAiAddDialog {...props} />), props }
}

beforeEach(() => {
  listModels.mockReset()
  listModels.mockResolvedValue({ models: [{ id: 'claude-opus-4-8', name: 'Claude Opus' }, { id: 'glm-5.3', name: 'GLM' }] })
})

describe('AgentRouter provider', () => {
  it('opens optional OpenClaw setup when AgentRouter is selected', () => {
    setup()
    fireEvent.change(screen.getAllByRole('combobox')[0], { target: { value: 'agentrouter' } })
    expect(screen.getByText('OpenClaw setup')).toBeInTheDocument()
    expect(listModels).not.toHaveBeenCalled()
  })
  it('loads the catalog on key blur and saves the selected exact ID with the recommended protocol', async () => {
    const { props } = setup()
    fireEvent.change(screen.getAllByRole('combobox')[0], { target: { value: 'agentrouter' } })
    fireEvent.click(screen.getByRole('button', { name: 'Direct connection' }))
    const key = screen.getByLabelText('plugins:llm.apiKey')
    fireEvent.change(key, { target: { value: 'account-test-key' } })
    fireEvent.blur(key)
    await screen.findByRole('button', { name: /Claude Opus/ })
    expect(listModels).toHaveBeenCalledWith({ api_key: 'account-test-key' })
    fireEvent.change(screen.getByLabelText('plugins:llm.model'), { target: { value: 'claude' } })
    expect(screen.queryByRole('button', { name: /GLM/ })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: /Claude Opus/ }))
    fireEvent.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => expect(props.onSubmit).toHaveBeenCalledWith(expect.objectContaining({
      name: 'AgentRouter', backend_type: 'anthropic', endpoint: 'https://agentrouter.org',
      model: 'claude-opus-4-8', api_key: 'account-test-key',
    }), undefined))
  })

  it('discovers using a stored backend ID without sending the API key mask', async () => {
    setup({ ...editing, endpoint: 'https://agentrouter.org/v1/' })
    expect(screen.getByDisplayValue('https://agentrouter.org/v1')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'plugins:llm.cloudModelsLoad' }))
    await waitFor(() => expect(listModels).toHaveBeenCalledWith({ backend_id: 'saved-router' }))
  })

  it('ignores a catalog from an old key after credentials change', async () => {
    let finish!: (value: { models: Array<{ id: string; name: string }> }) => void
    listModels.mockReturnValue(new Promise(resolve => { finish = resolve }))
    setup(editing)
    fireEvent.click(screen.getByRole('button', { name: 'plugins:llm.cloudModelsLoad' }))
    fireEvent.change(screen.getByLabelText('plugins:llm.apiKey'), { target: { value: 'new-test-key' } })
    await act(async () => finish({ models: [{ id: 'old-private-model', name: 'Old private model' }] }))
    expect(screen.queryByText('Old private model')).not.toBeInTheDocument()
  })

  it('keeps manual model entry available if the provider denies discovery', async () => {
    listModels.mockRejectedValue(new Error('Provider rejected access'))
    const { props } = setup(editing)
    fireEvent.click(screen.getByRole('button', { name: 'plugins:llm.cloudModelsLoad' }))
    expect(await screen.findByRole('alert')).toHaveTextContent('Provider rejected access')
    fireEvent.change(screen.getByLabelText('plugins:llm.model'), { target: { value: 'account-specific-model' } })
    fireEvent.click(screen.getByRole('button', { name: 'Save' }))
    await waitFor(() => expect(props.onSubmit).toHaveBeenCalledWith(expect.objectContaining({ model: 'account-specific-model' }), editing.id))
    expect(props.onSubmit.mock.calls[0][0]).not.toHaveProperty('api_key')
  })

  it('requires a new key when switching an existing unrelated provider to AgentRouter', () => {
    setup({ ...editing, endpoint: 'https://api.other-provider.example/v1' })
    fireEvent.change(screen.getAllByRole('combobox')[0], { target: { value: 'agentrouter' } })
    fireEvent.click(screen.getByRole('button', { name: 'Direct connection' }))
    fireEvent.change(screen.getByLabelText('plugins:llm.model'), { target: { value: 'glm-5.3' } })
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'plugins:llm.cloudModelsLoad' })).toBeDisabled()
    expect(listModels).not.toHaveBeenCalled()
  })

  it('clears catalog and context overrides when a new dialog is opened', async () => {
    const { props, rerender } = setup({ ...editing, max_context: 32768 })
    expect(screen.getByDisplayValue('32768')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'plugins:llm.cloudModelsLoad' }))
    await screen.findByRole('button', { name: /GLM/ })
    rerender(<CloudAiAddDialog {...props} open={false} />)
    rerender(<CloudAiAddDialog {...props} editing={undefined} />)
    expect(screen.queryByDisplayValue('32768')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /GLM/ })).not.toBeInTheDocument()
  })
})
