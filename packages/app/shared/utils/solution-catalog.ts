// Solution Library catalog: the ready-made ERP module packages shipped
// in `packages/erp/`. Imported as JSON so the catalog is bundled at build
// time and works identically in dev and production (no runtime file reads).
//
// Lives in `shared/` (not `server/`) on purpose: the desktop build's
// `modules/desktop.ts` imports this from client-side code (the generated
// `#build/fetch.mjs`), and Nuxt's server/client bundle isolation
// (`impound`) blocks client code from importing anything under `server/`.
// This file has no Vue or Nitro dependencies, so it is safe to share.
import accounting from '../../../erp/accounting.module.json'
import dormitory from '../../../erp/dormitory.module.json'
import hr from '../../../erp/hr.module.json'
import inventory from '../../../erp/inventory.module.json'
import manufacturing from '../../../erp/manufacturing.module.json'
import pos from '../../../erp/pos.module.json'
import sales from '../../../erp/sales.module.json'
import vehicle from '../../../erp/vehicle.module.json'

type PackageEntity = { name?: string; label?: string; fields?: unknown[] }
export type ShippedPackage = {
  schema_version?: number
  kind?: string
  manifest?: { name?: string; version?: string }
  module?: {
    name?: string
    label?: string
    description?: string
    icon?: string
    color?: string
    definition?: { entities?: PackageEntity[] }
    relations?: unknown[]
    actions?: unknown[]
    automations?: unknown[]
    dashboards?: { name?: string }[]
  }
}

const PACKAGES: Record<string, ShippedPackage> = {
  accounting: accounting as ShippedPackage,
  dormitory: dormitory as ShippedPackage,
  hr: hr as ShippedPackage,
  inventory: inventory as ShippedPackage,
  manufacturing: manufacturing as ShippedPackage,
  pos: pos as ShippedPackage,
  sales: sales as ShippedPackage,
  vehicle: vehicle as ShippedPackage
}

export function solutionCatalog() {
  return Object.entries(PACKAGES).map(([key, pkg]) => {
    const entities = pkg.module?.definition?.entities || []
    return {
      key,
      name: pkg.module?.name || key,
      label: pkg.module?.label || key,
      description: pkg.module?.description || '',
      icon: pkg.module?.icon || 'i-lucide-box',
      color: pkg.module?.color || 'blue',
      version: pkg.manifest?.version || '1.0.0',
      entities: entities.map(e => ({
        name: e.name || '',
        label: e.label || e.name || '',
        fields: Array.isArray(e.fields) ? e.fields.length : 0
      })),
      relations: pkg.module?.relations?.length || 0,
      actions: pkg.module?.actions?.length || 0,
      automations: pkg.module?.automations?.length || 0,
      dashboards: (pkg.module?.dashboards || []).map(d => d?.name || '')
    }
  })
}

export function solutionPackage(name: string): ShippedPackage | null {
  return PACKAGES[name] || null
}
