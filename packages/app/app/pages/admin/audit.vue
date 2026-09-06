<script setup lang="ts">
import { h, resolveComponent } from 'vue'
import type { TableColumn } from '@nuxt/ui'
import { absoluteTime, actionLabel, relativeTime } from '../../utils/audit-time'

definePageMeta({ middleware: 'auth' })

const UButton = resolveComponent('UButton')
const UBadge = resolveComponent('UBadge')

type Entity = { id: string; label: string }
type AuditEntry = {
  id: string
  entity_id: string
  entity_label: string
  doc_id: string
  action: string
  payload: Record<string, unknown>
  created_at: string
  actor?: string | null
}
type AuditList = { items: AuditEntry[]; total: number }

const toast = useToast()
const { data: entities } = await useFetch<Entity[]>('/api/entities')

const limit = ref(50)
const offset = ref(0)
const entityFilter = ref('all')
const actionFilter = ref('all')
const search = ref('')

const auditUrl = computed(() => {
  const params = new URLSearchParams({
    limit: String(limit.value),
    offset: String(offset.value)
  })
  if (entityFilter.value && entityFilter.value !== 'all') params.set('entity_id', entityFilter.value)
  if (actionFilter.value && actionFilter.value !== 'all') params.set('action', actionFilter.value)
  if (search.value.trim()) params.set('search', search.value.trim())
  return `/api/audit?${params.toString()}`
})
const { data: audit, status, error, refresh } = await useFetch<AuditList>(auditUrl, { watch: [auditUrl] })

const entityItems = computed(() => [
  { label: 'All entities', value: 'all' },
  ...((entities.value || []).map(e => ({ label: e.label, value: e.id })))
])

const actionItems = [
  { label: 'All actions', value: 'all' },
  { label: 'Created', value: 'create' },
  { label: 'Updated', value: 'update' },
  { label: 'Deleted', value: 'delete' },
  { label: 'Transitioned', value: 'transition' },
  { label: 'Imported', value: 'import' }
]

const total = computed(() => audit.value?.total || 0)
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

function actionColor(action: string) {
  switch (action) {
    case 'create': return 'success'
    case 'delete': return 'error'
    case 'update': return 'info'
    case 'transition': return 'primary'
    default: return 'neutral'
  }
}

const auditColumns: TableColumn<AuditEntry>[] = [
  {
    accessorKey: 'entity_label',
    header: 'Entity'
  },
  {
    accessorKey: 'doc_id',
    header: 'Document',
    cell: ({ row }) => h('span', { class: 'font-mono' }, row.original.doc_id)
  },
  {
    accessorKey: 'action',
    header: 'Action',
    cell: ({ row }) => h(UBadge, { color: actionColor(row.original.action), variant: 'subtle' }, () => actionLabel(row.original.action))
  },
  {
    accessorKey: 'actor',
    header: 'By',
    cell: ({ row }) => h('span', { class: 'text-sm' }, row.original.actor || 'system')
  },
  {
    accessorKey: 'created_at',
    header: 'When',
    cell: ({ row }) => h('span', { title: absoluteTime(row.original.created_at) }, relativeTime(row.original.created_at))
  }
]
</script>

<template>
  <UDashboardPanel id="audit-log">
    <template #header>
      <UDashboardNavbar title="Audit Log">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" icon="i-lucide-refresh-cw" :loading="status === 'pending'" @click="refresh()">Refresh</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-5xl space-y-4">
        <div class="flex flex-wrap items-center gap-2">
          <USelectMenu
            v-model="entityFilter"
            :items="entityItems"
            value-key="value"
            class="w-48"
            aria-label="Filter by entity"
            @update:model-value="applyFilters"
          />
          <USelectMenu
            v-model="actionFilter"
            :items="actionItems"
            value-key="value"
            class="w-44"
            aria-label="Filter by action"
            @update:model-value="applyFilters"
          />
          <UInput
            v-model="search"
            icon="i-lucide-search"
            placeholder="Search document ID…"
            class="w-56"
            @keyup.enter="applyFilters"
          >
            <template v-if="search" #trailing>
              <UButton size="xs" variant="link" color="neutral" icon="i-lucide-x" @click="search = ''; applyFilters()" />
            </template>
          </UInput>
        </div>

        <UAlert v-if="error" color="error" title="Cannot load audit log" :description="error.message">
          <template #actions>
            <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
          </template>
        </UAlert>

        <div v-else-if="status === 'pending'" class="space-y-3" aria-busy="true">
          <USkeleton v-for="index in 4" :key="index" class="h-10 w-full" />
        </div>

        <template v-else>
          <UCard>
            <UTable :data="audit?.items || []" :columns="auditColumns" :get-row-id="(row: AuditEntry) => row.id" class="w-full">
              <template #empty>
                <div class="flex flex-col items-center gap-3 py-16 text-center">
                  <UIcon name="i-lucide-history" class="h-10 w-10 text-muted" />
                  <p class="text-sm text-muted">No audit entries match these filters.</p>
                </div>
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