<script setup lang="ts">
import type { CommandPaletteGroup, NavigationMenuItem } from '@nuxt/ui'

const open = ref(false)
const commandOpen = ref(false)
const { user } = useAuth()
const { data: entities, status, error, refresh } = await useFetch<{ id: string; label: string; module?: string | null }[]>('/api/entities')

const isAdmin = computed(() => user.value?.role === 'admin')

const mainLinks = computed<NavigationMenuItem[]>(() => {
  const links: NavigationMenuItem[] = [
    { label: 'Apps', icon: 'i-lucide-layout-grid', to: '/apps' },
    { label: 'Dashboard', icon: 'i-lucide-house', to: '/dashboard' },
    { label: 'PM Dashboard', icon: 'i-lucide-clipboard-list', to: '/app/pm' },
    { label: 'Accounting', icon: 'i-lucide-calculator', to: '/app/accounting' },
    { label: 'Stock', icon: 'i-lucide-package', to: '/app/stock' },
    { label: 'Sales', icon: 'i-lucide-shopping-cart', to: '/app/sales' },
    { label: 'Purchases', icon: 'i-lucide-truck', to: '/app/purchases' },
    { label: 'Reports', icon: 'i-lucide-chart-bar', to: '/app/reports' }
  ]
  if (isAdmin.value) {
    links.push(
      { label: 'Entity Manager', icon: 'i-lucide-layout-grid', to: '/admin/meta/entity' },
      { label: 'Workflow Builder', icon: 'i-lucide-git-branch', to: '/admin/meta/workflow' },
      { label: 'Users', icon: 'i-lucide-users', to: '/admin/users' },
      { label: 'Audit Log', icon: 'i-lucide-history', to: '/admin/audit' },
      { label: 'Settings', icon: 'i-lucide-settings', to: '/admin/settings' }
    )
  }
  return links
})

const entityLinks = computed<NavigationMenuItem[]>(() => {
  const groups = new Map<string, { id: string; label: string }[]>()
  for (const e of entities.value || []) {
    const module = (e.module || '').trim() || 'Other'
    if (!groups.has(module)) groups.set(module, [])
    groups.get(module)!.push(e)
  }
  const names = [...groups.keys()].sort((a, b) => {
    if (a === 'Other') return 1
    if (b === 'Other') return -1
    return a.localeCompare(b)
  })
  if (names.length <= 1 && !names[0]) return []
  // Single group without a real module name: keep the flat list.
  if (names.length === 1 && names[0] === 'Other') {
    return (groups.get('Other') || []).map(e => ({
      label: e.label,
      icon: 'i-lucide-table',
      to: `/app/${encodeURIComponent(e.id)}`
    }))
  }
  return names.map(name => ({
    label: name,
    icon: 'i-lucide-box',
    to: `/app/modules/${encodeURIComponent(name)}`,
    children: (groups.get(name) || []).map(e => ({
      label: e.label,
      icon: 'i-lucide-table',
      to: `/app/${encodeURIComponent(e.id)}`
    }))
  }))
})

// --- Command palette (⌘K) ---
const commandGroups = computed<CommandPaletteGroup[]>(() => {
  const nav: CommandPaletteGroup['items'] = [
    { label: 'Apps', icon: 'i-lucide-layout-grid', to: '/apps', kbds: ['g', 'a'] },
    { label: 'Dashboard', icon: 'i-lucide-house', to: '/dashboard', kbds: ['g', 'd'] },
    { label: 'PM Dashboard', icon: 'i-lucide-clipboard-list', to: '/app/pm', kbds: ['g', 'p'] },
    { label: 'Accounting', icon: 'i-lucide-calculator', to: '/app/accounting', kbds: ['g', 'c'] },
    { label: 'Stock', icon: 'i-lucide-package', to: '/app/stock' },
    { label: 'Sales', icon: 'i-lucide-shopping-cart', to: '/app/sales' },
    { label: 'Purchases', icon: 'i-lucide-truck', to: '/app/purchases' },
    { label: 'Reports', icon: 'i-lucide-chart-bar', to: '/app/reports', kbds: ['g', 'r'] }
  ]
  if (isAdmin.value) {
    nav.push(
      { label: 'Entity Manager', icon: 'i-lucide-layout-grid', to: '/admin/meta/entity', kbds: ['g', 'e'] },
      { label: 'Workflow Builder', icon: 'i-lucide-git-branch', to: '/admin/meta/workflow', kbds: ['g', 'w'] },
      { label: 'Users', icon: 'i-lucide-users', to: '/admin/users', kbds: ['g', 'u'] },
      { label: 'Audit Log', icon: 'i-lucide-history', to: '/admin/audit', kbds: ['g', 'a'] },
      { label: 'Settings', icon: 'i-lucide-settings', to: '/admin/settings', kbds: ['g', 's'] }
    )
  }
  const entityItems: CommandPaletteGroup['items'] = []
  const modules = new Map<string, { id: string; label: string }[]>()
  for (const e of entities.value || []) {
    const module = (e.module || '').trim() || 'Other'
    if (!modules.has(module)) modules.set(module, [])
    modules.get(module)!.push(e)
  }
  const moduleItems: CommandPaletteGroup['items'] = [...modules.keys()].sort().map(name => ({
    label: name,
    suffix: `${(modules.get(name) || []).length} entities`,
    icon: 'i-lucide-box',
    to: `/app/modules/${encodeURIComponent(name)}`
  }))
  for (const name of [...modules.keys()].sort()) {
    for (const e of modules.get(name) || []) {
      entityItems.push({
        label: e.label,
        suffix: name === 'Other' ? e.id : `${name} · ${e.id}`,
        icon: 'i-lucide-table',
        to: `/app/${encodeURIComponent(e.id)}`
      })
    }
  }
  return [
    { id: 'navigation', label: 'Navigation', items: nav },
    { id: 'modules', label: 'Apps', items: moduleItems },
    { id: 'entities', label: 'Entities', items: entityItems }
  ]
})

function onCommandSelect() {
  commandOpen.value = false
}

function onKeydown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault()
    commandOpen.value = true
  }
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <UDashboardGroup unit="rem">
    <UDashboardSidebar
      id="default"
      v-model:open="open"
      collapsible
      resizable
      class="bg-elevated/25"
      :ui="{ footer: 'lg:border-t lg:border-default' }"
    >
      <template #header="{ collapsed }">
        <div class="flex h-12 items-center gap-2 px-3">
          <span class="text-lg font-bold text-primary">L</span>
          <span v-if="!collapsed" class="font-semibold">LOGHOLIZON</span>
        </div>
      </template>

      <template #default="{ collapsed }">
        <UAlert
          v-if="status === 'error'"
          color="error"
          title="Cannot load entities"
          :description="error?.message || 'Check the Rust core connection.'"
          class="mb-3"
        >
          <template #actions>
            <UButton size="xs" variant="outline" @click="refresh()">Retry</UButton>
          </template>
        </UAlert>
        <UNavigationMenu :collapsed="collapsed" :items="mainLinks" orientation="vertical" tooltip popover />
        <UNavigationMenu
          v-if="entityLinks.length"
          :collapsed="collapsed"
          :items="entityLinks"
          orientation="vertical"
          tooltip
          class="mt-4"
        />
      </template>

      <template #footer="{ collapsed }">
        <UserMenu :collapsed="collapsed" />
      </template>
    </UDashboardSidebar>

    <slot />

    <UModal v-model:open="commandOpen" :ui="{ content: 'max-w-lg' }">
      <template #content>
        <UCommandPalette
          :groups="commandGroups"
          placeholder="Search pages and entities…"
          close
          class="h-80"
          @update:model-value="onCommandSelect"
          @update:open="commandOpen = $event"
        />
      </template>
    </UModal>
  </UDashboardGroup>
</template>
