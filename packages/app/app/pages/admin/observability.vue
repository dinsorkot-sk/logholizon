<script setup lang="ts">
import { h, resolveComponent } from 'vue'
import type { TableColumn } from '@nuxt/ui'
import { absoluteTime, relativeTime } from '../../utils/audit-time'

definePageMeta({ middleware: 'auth' })

const UButton = resolveComponent('UButton')
const UBadge = resolveComponent('UBadge')

type ObservabilityLog = {
  id: string
  occurred_at: string
  level: string
  category: string
  action: string
  actor: string | null
  request_id: string | null
  correlation_id: string | null
  target_type: string | null
  target_id: string | null
  status_code: number | null
  duration_ms: number | null
  message: string
  metadata: Record<string, unknown>
}
type ObservabilityList = { total: number; items: ObservabilityLog[] }
type ObservabilityMetrics = {
  observability_events: number
  errors: number
  security_denials: number
  logins: number
  pending_automation: number
  pending_webhooks: number
}

const toast = useToast()
const limit = ref(50)
const offset = ref(0)
const categoryFilter = ref('all')
const levelFilter = ref('all')
const actorFilter = ref('')
const requestIdFilter = ref('')
const correlationIdFilter = ref('')

const logsUrl = computed(() => {
  const params = new URLSearchParams({
    limit: String(limit.value),
    offset: String(offset.value)
  })
  if (categoryFilter.value && categoryFilter.value !== 'all') params.set('category', categoryFilter.value)
  if (levelFilter.value && levelFilter.value !== 'all') params.set('level', levelFilter.value)
  if (actorFilter.value.trim()) params.set('actor', actorFilter.value.trim())
  if (requestIdFilter.value.trim()) params.set('request_id', requestIdFilter.value.trim())
  if (correlationIdFilter.value.trim()) params.set('correlation_id', correlationIdFilter.value.trim())
  return `/api/admin/observability/logs?${params.toString()}`
})
const { data: logs, status, error, refresh } = await useFetch<ObservabilityList>(logsUrl, { watch: [logsUrl] })
const { data: metrics, refresh: refreshMetrics } = await useFetch<ObservabilityMetrics>('/api/admin/observability/metrics')

const categoryItems = [
  { label: 'All categories', value: 'all' },
  { label: 'Request', value: 'request' },
  { label: 'Security', value: 'security' },
  { label: 'Automation', value: 'automation' },
  { label: 'Webhook', value: 'webhook' },
  { label: 'Workflow', value: 'workflow' },
  { label: 'Report', value: 'report' },
  { label: 'System', value: 'system' }
]

const levelItems = [
  { label: 'All levels', value: 'all' },
  { label: 'Info', value: 'info' },
  { label: 'Warn', value: 'warn' },
  { label: 'Error', value: 'error' }
]

const total = computed(() => logs.value?.total || 0)
const pageStart = computed(() => (total.value === 0 ? 0 : offset.value + 1))
const pageEnd = computed(() => Math.min(offset.value + limit.value, total.value))
const hasNext = computed(() => pageEnd.value < total.value)
const hasPrev = computed(() => offset.value > 0)

function nextPage() {
  offset.value += limit.value
}
function prevPage() {
  offset.value = Math.max(0, offset.value - limit.value)
}
function applyFilters() {
  offset.value = 0
  refresh()
}
function refreshAll() {
  refresh()
  refreshMetrics()
}

function levelColor(level: string) {
  switch (level) {
    case 'error': return 'error'
    case 'warn': return 'warning'
    default: return 'success'
  }
}

function categoryColor(category: string) {
  switch (category) {
    case 'security': return 'error'
    case 'automation': return 'primary'
    case 'webhook': return 'info'
    case 'workflow': return 'warning'
    case 'report': return 'info'
    default: return 'neutral'
  }
}

const metricCards = computed(() => [
  { label: 'Events', value: metrics.value?.observability_events ?? 0, icon: 'i-lucide-activity', color: 'primary' },
  { label: 'Errors', value: metrics.value?.errors ?? 0, icon: 'i-lucide-alert-triangle', color: 'error' },
  { label: 'Security denials', value: metrics.value?.security_denials ?? 0, icon: 'i-lucide-shield-alert', color: 'warning' },
  { label: 'Logins', value: metrics.value?.logins ?? 0, icon: 'i-lucide-log-in', color: 'success' },
  { label: 'Pending automation', value: metrics.value?.pending_automation ?? 0, icon: 'i-lucide-workflow', color: 'info' },
  { label: 'Pending webhooks', value: metrics.value?.pending_webhooks ?? 0, icon: 'i-lucide-webhook', color: 'neutral' }
])

const logColumns: TableColumn<ObservabilityLog>[] = [
  {
    accessorKey: 'occurred_at',
    header: 'When',
    cell: ({ row }) => h('span', { title: absoluteTime(row.original.occurred_at) }, relativeTime(row.original.occurred_at))
  },
  {
    accessorKey: 'level',
    header: 'Level',
    cell: ({ row }) => h(UBadge, { color: levelColor(row.original.level), variant: 'subtle' }, () => row.original.level)
  },
  {
    accessorKey: 'category',
    header: 'Category',
    cell: ({ row }) => h(UBadge, { color: categoryColor(row.original.category), variant: 'subtle' }, () => row.original.category)
  },
  {
    accessorKey: 'action',
    header: 'Action',
    cell: ({ row }) => h('span', { class: 'font-mono text-sm' }, row.original.action)
  },
  {
    accessorKey: 'actor',
    header: 'Actor',
    cell: ({ row }) => h('span', { class: 'text-sm' }, row.original.actor || 'system')
  },
  {
    accessorKey: 'status_code',
    header: 'Status',
    cell: ({ row }) => row.original.status_code != null ? h('span', { class: 'font-mono text-sm' }, String(row.original.status_code)) : h('span', { class: 'text-muted' }, '—')
  },
  {
    accessorKey: 'duration_ms',
    header: 'Duration',
    cell: ({ row }) => row.original.duration_ms != null ? h('span', { class: 'font-mono text-sm' }, `${row.original.duration_ms}ms`) : h('span', { class: 'text-muted' }, '—')
  },
  {
    accessorKey: 'message',
    header: 'Message',
    cell: ({ row }) => h('span', { class: 'text-sm text-muted' }, row.original.message || '—')
  }
]

function copyRequestId(id: string | null) {
  if (!id) return
  navigator.clipboard.writeText(id)
  toast.add({ title: 'Request ID copied', color: 'success' })
}
</script>

<template>
  <UDashboardPanel id="observability">
    <template #header>
      <UDashboardNavbar title="Observability">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="status === 'pending'" @click="refreshAll">Refresh</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-6xl space-y-4">
        <div class="grid grid-cols-2 gap-3 md:grid-cols-3 lg:grid-cols-6">
          <UCard v-for="card in metricCards" :key="card.label" class="!p-3">
            <div class="flex items-center gap-2 text-muted">
              <UIcon :name="card.icon" class="h-4 w-4" />
              <span class="text-xs">{{ card.label }}</span>
            </div>
            <p class="mt-1 text-2xl font-semibold">{{ card.value }}</p>
          </UCard>
        </div>

        <div class="flex flex-wrap items-center gap-2">
          <USelectMenu
            v-model="categoryFilter"
            :items="categoryItems"
            value-key="value"
            class="w-44"
            aria-label="Filter by category"
            @update:model-value="applyFilters"
          />
          <USelectMenu
            v-model="levelFilter"
            :items="levelItems"
            value-key="value"
            class="w-36"
            aria-label="Filter by level"
            @update:model-value="applyFilters"
          />
          <UInput
            v-model="actorFilter"
            icon="i-lucide-user"
            placeholder="Actor…"
            class="w-40"
            @keyup.enter="applyFilters"
          >
            <template v-if="actorFilter" #trailing>
              <UButton size="xs" variant="link" color="neutral" icon="i-lucide-x" @click="actorFilter = ''; applyFilters()" />
            </template>
          </UInput>
          <UInput
            v-model="requestIdFilter"
            icon="i-lucide-fingerprint"
            placeholder="Request ID…"
            class="w-48"
            @keyup.enter="applyFilters"
          >
            <template v-if="requestIdFilter" #trailing>
              <UButton size="xs" variant="link" color="neutral" icon="i-lucide-x" @click="requestIdFilter = ''; applyFilters()" />
            </template>
          </UInput>
          <UInput
            v-model="correlationIdFilter"
            icon="i-lucide-link"
            placeholder="Correlation ID…"
            class="w-48"
            @keyup.enter="applyFilters"
          >
            <template v-if="correlationIdFilter" #trailing>
              <UButton size="xs" variant="link" color="neutral" icon="i-lucide-x" @click="correlationIdFilter = ''; applyFilters()" />
            </template>
          </UInput>
        </div>

        <UAlert v-if="error" color="error" title="Cannot load observability logs" :description="error.message">
          <template #actions>
            <UButton size="sm" variant="outline" @click="refreshAll">Retry</UButton>
          </template>
        </UAlert>

        <div v-else-if="status === 'pending'" class="space-y-3" aria-busy="true">
          <USkeleton v-for="index in 4" :key="index" class="h-10 w-full" />
        </div>

        <template v-else>
          <UCard>
            <UTable :data="logs?.items || []" :columns="logColumns" :get-row-id="(row: ObservabilityLog) => row.id" class="w-full">
              <template #empty>
                <div class="flex flex-col items-center gap-3 py-16 text-center">
                  <UIcon name="i-lucide-activity" class="h-10 w-10 text-muted" />
                  <p class="text-sm text-muted">No observability events match these filters.</p>
                </div>
              </template>
              <template #request_id-cell="{ row }">
                <UButton
                  v-if="row.original.request_id"
                  size="xs"
                  variant="link"
                  color="neutral"
                  icon="i-lucide-copy"
                  class="font-mono text-xs"
                  @click="copyRequestId(row.original.request_id)"
                >
                  {{ row.original.request_id.slice(0, 12) }}…
                </UButton>
                <span v-else class="text-muted">—</span>
              </template>
            </UTable>
          </UCard>

          <div v-if="total > 0" class="flex items-center justify-between text-sm text-muted">
            <p>Showing {{ pageStart }}–{{ pageEnd }} of {{ total }}</p>
            <div class="flex gap-2">
              <UButton size="sm" variant="outline" :disabled="!hasPrev" @click="prevPage">Prev</UButton>
              <UButton size="sm" variant="outline" :disabled="!hasNext" @click="nextPage">Next</UButton>
            </div>
          </div>
        </template>
      </div>
    </template>
  </UDashboardPanel>
</template>