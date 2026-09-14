import { describe, expect, it } from 'vitest'
import { groupEntitiesByModule, moduleRoute, moduleVisual } from '../app/utils/module-apps'

const entities = [
  { id: 'work_order', label: 'Work Order', module: null },
  { id: 'product', label: 'Product', module: 'Stock' },
  { id: 'stock_move', label: 'Stock Move', module: 'Stock' }
]

describe('groupEntitiesByModule', () => {
  it('groups by module with Other last', () => {
    const apps = groupEntitiesByModule(entities)
    expect(apps.map(a => a.name)).toEqual(['Stock', 'Other'])
    expect(apps[0]!.entities.map(e => e.id)).toEqual(['product', 'stock_move'])
    expect(apps[1]!.entities.map(e => e.id)).toEqual(['work_order'])
  })

  it('assigns stable icon and color per module', () => {
    expect(moduleVisual('Stock')).toEqual(moduleVisual('Stock'))
    expect(moduleVisual('Other')).toEqual({ icon: 'i-lucide-layout-grid', color: 'neutral' })
  })
})

describe('moduleRoute', () => {
  it('encodes module names in routes', () => {
    expect(moduleRoute('Stock')).toBe('/app/modules/Stock')
    expect(moduleRoute('My Module')).toBe('/app/modules/My%20Module')
  })
})
