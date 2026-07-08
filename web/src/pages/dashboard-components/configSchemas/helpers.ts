/**
 * Updater factory for configSchema modules.
 *
 * Creates the standard updater callbacks that each schema function needs:
 * updateConfig, updateNestedConfig, updateDataSource, updateDataMapping.
 */

import type { SchemaContext, Updaters } from './types'

export function makeUpdaters(
  config: any,
  ctx: SchemaContext,
): Updaters {
  const { setConfigTitle, selectedComponent, updateComponent, setComponentConfig } = ctx

  const updateConfig = (key: string) => (value: any) => {
    if (key === 'title') {
      setConfigTitle(value)
      if (selectedComponent) {
        updateComponent(selectedComponent.id, { title: value }, false)
      }
    }
    setComponentConfig(prev => {
      const updated = { ...prev, [key]: value }
      return updated
    })
  }

  const updateNestedConfig = (parent: string, key: string) => (value: any) => {
    setComponentConfig(prev => ({
      ...prev,
      [parent]: { ...prev[parent], [key]: value },
    }))
  }

  const updateDataSource = (ds: any) => {
    setComponentConfig(prev => ({ ...prev, dataSource: ds }))
  }

  const updateDataMapping = (newMapping: any) => {
    setComponentConfig(prev => ({ ...prev, dataMapping: newMapping }))
  }

  return { updateConfig, updateNestedConfig, updateDataSource, updateDataMapping }
}
