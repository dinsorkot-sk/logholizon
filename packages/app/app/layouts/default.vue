<script setup lang="ts">
import type { CommandPaletteGroup, NavigationMenuItem } from '@nuxt/ui'

const open = ref(false)
const commandOpen = ref(false)
const { data: entities, status, error, refresh } = await useFetch<{ id: string; label: string }[]>('/api/meta/entities')

const mainLinks = computed<NavigationMenuItem[]>(() => [
  { label: 'Dashboard', icon: 'i-lucide-house', to: '/dashboard' },
  { label: 'Entity Manager', icon: 'i-lucide-layout-grid', to: '/admin/meta/entity' }
])

const entityLinks = computed<NavigationMenuItem[]>(() => (entities.value || []).map(e => ({
  label: e.label,
  icon: 'i-lucide-table',
  to: `/app/${encodeURIComponent(e.id)}`
})))

// --- Command palette (⌘K) ---
const commandGroups = computed<CommandPaletteGroup[]>(() => [
  {
    id: 'navigation',
    label: 'Navigation',
    items: [
      { label: 'Dashboard', icon: 'i-lucide-house', to: '/dashboard', kbds: ['g', 'd'] },
      { label: 'Entity Manager', icon: 'i-lucide-layout-grid', to: '/admin/meta/entity', kbds: ['g', 'e'] }
    ]
  },
  {
    id: 'entities',
    label: 'Entities',
    items: (entities.value || []).map(e => ({
      label: e.label,
      suffix: e.id,
      icon: 'i-lucide-table',
      to: `/app/${encodeURIComponent(e.id)}`
    }))
  }
])

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

const environment = process.env.NODE_ENV === 'production' ? 'prod' : 'dev'
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
        <div class="px-3 py-2 text-xs text-muted">
          <span v-if="!collapsed">LOGHOLIZON · {{ environment }}</span>
          <span v-else>LH</span>
        </div>
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
