import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { WorkspaceSegmentedControl } from '../WorkspaceSegmentedControl'

describe('WorkspaceSegmentedControl', () => {
  const segments = [
    { value: 'condition', label: '触发条件', count: 2 },
    { value: 'action', label: '执行动作', count: 1 },
  ]

  it('renders all segments with counts', () => {
    render(
      <WorkspaceSegmentedControl
        segments={segments}
        value="condition"
        onChange={() => {}}
        accent="indigo"
      />
    )
    expect(screen.getByRole('tab', { name: /触发条件/ })).toHaveAttribute('aria-selected', 'true')
    expect(screen.getByRole('tab', { name: /执行动作/ })).toHaveAttribute('aria-selected', 'false')
    expect(screen.getByText('2')).toBeInTheDocument()
    expect(screen.getByText('1')).toBeInTheDocument()
  })

  it('calls onChange with the selected segment value', () => {
    const onChange = vi.fn()
    render(
      <WorkspaceSegmentedControl
        segments={segments}
        value="condition"
        onChange={onChange}
        accent="indigo"
      />
    )
    fireEvent.click(screen.getByRole('tab', { name: /执行动作/ }))
    expect(onChange).toHaveBeenCalledWith('action')
  })
})