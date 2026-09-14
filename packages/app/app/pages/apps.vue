<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

import { groupEntitiesByModule, moduleRoute } from '~/utils/module-apps'

type Entity = { id: string; label: string; module?: string | null }

const { data: entities, status, error, refresh } = await useFetch<Entity[]>('/api/entities')
const apps = computed(() => groupEntitiesByModule(entities.value || []))
</script>

<template>
  <UDashboardPanel id="apps">
    <template #header>
      <UDashboardNavbar title="Apps">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="status === 'pending'" @click="refresh()">Refresh</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <UAlert v-if="error" color="error" title="Cannot load apps" :description="error.message">
        <template #actions>
          <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
        </template>
      </UAlert>

      <div v-else-if="status === 'pending'" class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3" aria-busy="true">
        <USkeleton v-for="index in 6" :key="index" class="h-32 w-full" />
      </div>

      <div v-else-if="!apps.length" class="flex flex-col items-center gap-3 py-16 text-center">
        <UIcon name="i-lucide-layout-grid" class="h-10 w-10 text-muted" />
        <p class="text-sm text-muted">No apps yet. Create an entity to get started.</p>
        <UButton icon="i-lucide-plus" to="/admin/meta/entity">Open Entity Manager</UButton>
      </div>

      <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <UCard v-for="app in apps" :key="app.name" :to="moduleRoute(app.name)">
          <div class="flex items-center gap-3">
            <UIcon :name="app.icon" class="h-8 w-8 text-primary" />
            <div class="min-w-0">
              <h2 class="truncate text-base font-semibold">{{ app.label }}</h2>
              <p class="text-sm text-muted">{{ app.entities.length }} {{ app.entities.length === 1 ? 'entity' : 'entities' }}</p>
            </div>
          </div>
          <div class="mt-3 flex flex-wrap gap-1">
            <UBadge v-for="entity in app.entities.slice(0, 4)" :key="entity.id" color="neutral" variant="subtle">
              {{ entity.label }}
            </UBadge>
            <UBadge v-if="app.entities.length > 4" color="neutral" variant="subtle">
              +{{ app.entities.length - 4 }} more
            </UBadge>
          </div>
        </UCard>
      </div>
    </template>
  </UDashboardPanel>
</template>
