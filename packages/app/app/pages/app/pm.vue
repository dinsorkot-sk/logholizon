<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

type PmSummary = { open: number; overdue: number; done_this_week: number; total: number }

const { data: summary, status, error, refresh } = await useFetch<PmSummary>('/api/dashboard/pm?entity_id=pm_schedule')

const cards = computed(() => [
  { label: 'Open', value: summary.value?.open ?? 0, icon: 'i-lucide-circle-dot', to: '/app/pm_schedule?status=open', color: 'info' as const },
  { label: 'Overdue', value: summary.value?.overdue ?? 0, icon: 'i-lucide-alert-triangle', to: '/app/pm_schedule?status=overdue', color: 'error' as const },
  { label: 'Done this week', value: summary.value?.done_this_week ?? 0, icon: 'i-lucide-check-circle', to: '/app/pm_schedule?status=done', color: 'success' as const },
  { label: 'Total', value: summary.value?.total ?? 0, icon: 'i-lucide-list', to: '/app/pm_schedule', color: 'neutral' as const }
])
</script>

<template>
  <UDashboardPanel id="pm-dashboard">
    <template #header>
      <UDashboardNavbar title="PM Dashboard">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="status === 'pending'" @click="refresh()">Refresh</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <UAlert v-if="error" color="error" title="Cannot load PM dashboard" :description="error.message">
        <template #actions>
          <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
        </template>
      </UAlert>

      <div v-else-if="status === 'pending'" class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4" aria-busy="true">
        <USkeleton v-for="index in 4" :key="index" class="h-24 w-full" />
      </div>

      <div v-else class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <UCard v-for="card in cards" :key="card.label" :to="card.to">
          <div class="flex items-center justify-between">
            <p class="text-sm text-muted">{{ card.label }}</p>
            <UIcon :name="card.icon" class="h-4 w-4 text-muted" />
          </div>
          <p class="mt-1 text-3xl font-semibold">{{ card.value }}</p>
        </UCard>
      </div>

      <div class="mt-6">
        <UCard>
          <template #header>
            <div class="flex items-center justify-between">
              <h2 class="text-sm font-semibold">PM Schedule</h2>
              <UButton size="sm" icon="i-lucide-arrow-right" :to="'/app/pm_schedule'">Open list</UButton>
            </div>
          </template>
          <p class="text-sm text-muted">Manage preventive maintenance schedules, due dates, and completion status.</p>
        </UCard>
      </div>
    </template>
  </UDashboardPanel>
</template>