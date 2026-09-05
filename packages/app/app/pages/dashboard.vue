<script setup lang="ts">
type Count = { status: string; count: number }
type Entity = { id: string; label: string }

const { data: entities } = await useFetch<Entity[]>('/api/meta/entities')
const entityId = ref('work_order')
const { data: counts, status, error, refresh } = await useFetch<Count[]>(
  () => `/api/dashboard/counts?entity_id=${encodeURIComponent(entityId.value)}`,
  { watch: [entityId] }
)
</script>

<template>
  <UDashboardPanel id="dashboard">
    <template #header>
      <UDashboardNavbar title="Dashboard">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" :loading="status === 'pending'" @click="refresh()">Refresh</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mb-4 flex flex-wrap items-center gap-2">
        <USelectMenu
          v-model="entityId"
          :items="(entities || []).map(e => ({ label: e.label, value: e.id }))"
          value-key="value"
          class="w-56"
          aria-label="Select entity"
        />
      </div>

      <UAlert v-if="error" color="error" title="Cannot load dashboard" :description="error.message">
        <template #actions>
          <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
        </template>
      </UAlert>

      <div v-else-if="status === 'pending'" class="grid grid-cols-1 gap-4 sm:grid-cols-3" aria-busy="true">
        <USkeleton v-for="index in 3" :key="index" class="h-24 w-full" />
      </div>

      <div v-else-if="!counts?.length" class="flex flex-col items-center gap-3 py-16 text-center">
        <UIcon name="i-lucide-inbox" class="h-10 w-10 text-muted" />
        <p class="text-sm text-muted">No records yet for this entity.</p>
        <UButton icon="i-lucide-plus" :to="`/app/${encodeURIComponent(entityId)}`">Create first record</UButton>
      </div>

      <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-3">
        <UCard v-for="item in counts" :key="item.status">
          <p class="text-sm text-muted">{{ item.status }}</p>
          <p class="text-3xl font-semibold">{{ item.count }}</p>
        </UCard>
      </div>
    </template>
  </UDashboardPanel>
</template>
