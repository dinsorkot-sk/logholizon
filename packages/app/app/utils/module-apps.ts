export type ModuleApp = {
  name: string
  label: string
  icon: string
  color: 'primary' | 'success' | 'info' | 'warning' | 'error' | 'neutral'
  entities: { id: string; label: string }[]
}

const MODULE_ICONS: { icon: string; color: ModuleApp['color'] }[] = [
  { icon: 'i-lucide-box', color: 'primary' },
  { icon: 'i-lucide-package', color: 'info' },
  { icon: 'i-lucide-briefcase', color: 'success' },
  { icon: 'i-lucide-factory', color: 'warning' },
  { icon: 'i-lucide-shopping-cart', color: 'error' },
  { icon: 'i-lucide-wrench', color: 'neutral' }
]

function hashName(name: string) {
  let hash = 0
  for (let i = 0; i < name.length; i++) {
    hash = (hash * 31 + name.charCodeAt(i)) | 0
  }
  return Math.abs(hash)
}

export function moduleVisual(name: string) {
  if (name === 'Other') return { icon: 'i-lucide-layout-grid', color: 'neutral' as const }
  return MODULE_ICONS[hashName(name) % MODULE_ICONS.length]!
}

export function groupEntitiesByModule<T extends { id: string; label: string; module?: string | null }>(
  entities: T[]
): ModuleApp[] {
  const groups = new Map<string, T[]>()
  for (const entity of entities) {
    const name = (entity.module || '').trim() || 'Other'
    if (!groups.has(name)) groups.set(name, [])
    groups.get(name)!.push(entity)
  }
  const names = [...groups.keys()].sort((a, b) => {
    if (a === 'Other') return 1
    if (b === 'Other') return -1
    return a.localeCompare(b)
  })
  return names.map(name => ({
    name,
    label: name,
    ...moduleVisual(name),
    entities: (groups.get(name) || []).map(e => ({ id: e.id, label: e.label }))
  }))
}

export function moduleRoute(name: string) {
  return `/app/modules/${encodeURIComponent(name)}`
}
