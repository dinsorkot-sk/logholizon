<script setup lang="ts">
import type { NavigationMenuItem } from '@nuxt/ui'

const open = ref(false)
const { data: entities } = await useFetch<{ id: string; label: string }[]>('/api/meta/entities')

const mainLinks = computed<NavigationMenuItem[]>(() => [
  { label: 'Dashboard', icon: 'i-lucide-house', to: '/dashboard' },
  { label: 'Entity Manager', icon: 'i-lucide-layout-grid', to: '/admin/meta/entity' }
])

const entityLinks = computed<NavigationMenuItem[]>(() => (entities.value || []).map(e => ({
  label: e.label,
  icon: 'i-lucide-table',
  to: `/app/${encodeURIComponent(e.id)}`
})))
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
          <span v-if="!collapsed">Rust core owns persistence</span>
          <span v-else>RS</span>
        </div>
      </template>
    </UDashboardSidebar>

    <slot />
  </UDashboardGroup>
</template>
