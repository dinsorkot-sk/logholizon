import { describe, expect, it } from 'vitest'
import { solutionCatalog, solutionPackage } from '../server/api/solutions/catalog'

describe('solutionCatalog', () => {
  it('lists all eight shipped ERP solutions', () => {
    const catalog = solutionCatalog()
    expect(catalog.map(s => s.key).sort()).toEqual([
      'accounting',
      'dormitory',
      'hr',
      'inventory',
      'manufacturing',
      'pos',
      'sales',
      'vehicle'
    ])
  })

  it('exposes install-relevant metadata per solution', () => {
    for (const solution of solutionCatalog()) {
      expect(solution.name).toBeTruthy()
      expect(solution.label).toBeTruthy()
      expect(solution.version).toMatch(/^\d+\.\d+\.\d+$/)
      expect(solution.entities.length).toBeGreaterThan(0)
      for (const entity of solution.entities) {
        expect(entity.name).toBeTruthy()
        expect(entity.fields).toBeGreaterThan(0)
      }
    }
  })

  it('resolves full packages by key and rejects unknown keys', () => {
    const vehicle = solutionPackage('vehicle')
    expect(vehicle?.kind).toBe('logholizon.module')
    expect(vehicle?.module?.name).toBe('vehicle')
    expect(vehicle?.manifest?.name).toBe('vehicle')
    expect(solutionPackage('ghost')).toBeNull()
  })
})
