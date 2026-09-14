<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

import { groupEntitiesByModule } from '~/utils/module-apps'

type Entity = { id: string; label: string; module?: string | null }
type Count = { status: string; count: number }

const route = useRoute()
const moduleName = computed(() => String(route.params.name || ''))
const { data: entities, status, error, refresh } = await useFetch<Entity[]>('/api/entities')
const app = computed(() => groupEntitiesByModule(entities.value || []).find(a => a.name === moduleName.value))

const counts = ref<Record<string, Count[]>>({})
const countsPending = ref(false)

async function loadCounts() {
  if (!app.value) return
  countsPending.value = true
  try {
    const entries = await Promise.all(
      app.value.entities.map(async (entity) => {
        try {
          const rows = await $fetch<Count[]>(`/api/dashboard/counts?entity_id=${encodeURIComponent(entity.id)}`)
          return [entity.id, rows] as const
        } catch {
          return [entity.id, []] as const
        }
      })
    )
    counts.value = Object.fromEntries(entries)
  } finally {
    countsPending.value = false
  }
}

watch(app, loadCounts, { immediate: true })

function entityTotal(id: string) {
  return (counts.value[id] || []).reduce((sum, row) => sum + row.count, 0)
}
</script>

<template>
  <UDashboardPanel :id="`module-${moduleName}`">
    <template #header>
      <UDashboardNavbar :title="app?.label || moduleName">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="status === 'pending' || countsPending" @click="() => { refresh(); loadCounts() }">Refresh</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <UBreadcrumb
        class="mb-4"
        :items="[{ label: 'Apps', to: '/apps' }, { label: app?.label || moduleName }]"
      />

      <UAlert v-if="error" color="error" title="Cannot load module" :description="error.message">
        <template #actions>
          <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
        </template>
      </UAlert>

      <div v-else-if="status === 'pending'" class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3" aria-busy="true">
        <USkeleton v-for="index in 3" :key="index" class="h-32 w-full" />
      </div>

      <div v-else-if="!app" class="flex flex-col items-center gap-3 py-16 text-center">
        <UIcon name="i-lucide-search-x" class="h-10 w-10 text-muted" />
        <p class="text-sm text-muted">No module named {{ moduleName }}.</p>
        <UButton icon="i-lucide-layout-grid" to="/apps">Back to Apps</UButton>
      </div>

      <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <UCard v-for="entity in app.entities" :key="entity.id" :to="`/app/${encodeURIComponent(entity.id)}`">
          <div class="flex items-center justify-between">
            <p class="text-sm text-muted">{{ entity.label }}</p>
            <UIcon :name="app.icon" class="h-4 w-4 text-muted" />
          </div>
          <p class="mt-1 text-3xl font-semibold">{{ entityTotal(entity.id) }}</p>
          <p class="mt-1 text-xs text-muted">records</p>
        </UCard>
      </div>
    </template>
  </UDashboardPanel>
</template>
