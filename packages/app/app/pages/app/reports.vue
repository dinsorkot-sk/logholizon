<script setup lang="ts">
import { Bar, Pie } from 'vue-chartjs'
import {
  ArcElement,
  BarElement,
  CategoryScale,
  Chart as ChartJS,
  Legend,
  LinearScale,
  Title,
  Tooltip
} from 'chart.js'

definePageMeta({ middleware: 'auth' })

ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale, ArcElement)

type Entity = { id: string; label: string }
type EntityField = { id: string; name: string; type: string; is_status: boolean }
type EntityDetail = Entity & { fields: EntityField[] }
type Bucket = { status: string; count: number }
type Report = { id: string; entity_id: string; name: string; config: { group_by?: string; chart_type?: string }; created_by: string | null }

const toast = useToast()
const { user } = useAuth()
const isAdmin = computed(() => user.value?.role === 'admin')

const { data: entities } = await useFetch<Entity[]>('/api/entities')
const entityId = ref('work_order')
watch(entities, (list) => {
  const first = list?.[0]
  if (list?.length && first && !list.some(e => e.id === entityId.value)) {
    entityId.value = first.id
  }
}, { immediate: true })

const { data: detail } = await useFetch<EntityDetail>(
  () => `/api/entities/${encodeURIComponent(entityId.value)}`,
  { watch: [entityId] }
)
const groupableFields = computed(() =>
  (detail.value?.fields || []).filter(f => f.type === 'select')
)
const groupBy = ref('')
watch(groupableFields, (fields) => {
  if (!fields.some(f => f.name === groupBy.value)) {
    groupBy.value = fields.find(f => f.is_status)?.name || fields[0]?.name || ''
  }
}, { immediate: true })

const chartType = ref<'bar' | 'pie'>('bar')
const chartItems = [
  { label: 'Bar', value: 'bar' },
  { label: 'Pie', value: 'pie' }
]

const aggregateUrl = computed(() => {
  if (!entityId.value || !groupBy.value) return ''
  return `/api/reports/aggregate?entity_id=${encodeURIComponent(entityId.value)}&group_by=${encodeURIComponent(groupBy.value)}`
})
const { data: buckets, status, error, refresh } = await useFetch<Bucket[]>(aggregateUrl, { watch: [aggregateUrl] })

const total = computed(() => (buckets.value || []).reduce((sum, b) => sum + b.count, 0))
const chartData = computed(() => ({
  labels: (buckets.value || []).map(b => b.status || '—'),
  datasets: [{
    label: groupBy.value,
    data: (buckets.value || []).map(b => b.count),
    backgroundColor: ['#00A155', '#3B82F6', '#F59E0B', '#EF4444', '#8B5CF6', '#06B6D4', '#EC4899']
  }]
}))
const chartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: { legend: { display: chartType.value === 'pie' }, title: { display: false } }
}))

function exportBucketsCsv() {
  const rows = [['bucket', 'count'], ...((buckets.value || []).map(b => [b.status, String(b.count)]))]
  const csv = rows.map(cells => cells.map(csvCell).join(',')).join('\n')
  const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv;charset=utf-8' }))
  const link = document.createElement('a')
  link.href = url
  link.download = `${entityId.value}-${groupBy.value}.csv`
  link.click()
  URL.revokeObjectURL(url)
  toast.add({ title: 'Export complete', color: 'success', icon: 'i-lucide-download' })
}

function csvCell(value: string) {
  return /[",\n]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value
}

// --- Saved reports ---
const reportsUrl = computed(() => `/api/entities/${encodeURIComponent(entityId.value)}/reports`)
const { data: reports, refresh: refreshReports } = await useFetch<Report[]>(reportsUrl, { watch: [reportsUrl] })
const reportOpen = ref(false)
const reportForm = reactive({ name: '' })
const reportError = ref('')
const savingReport = ref(false)
const deletingReport = ref(false)
const reportToDelete = ref<Report | null>(null)
const deleteReportOpen = ref(false)

function openAddReport() {
  reportForm.name = ''
  reportError.value = ''
  reportOpen.value = true
}

async function saveReport() {
  reportError.value = ''
  if (!reportForm.name.trim() || !groupBy.value) {
    reportError.value = 'name and group-by are required'
    return
  }
  savingReport.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(entityId.value)}/reports`, {
      method: 'POST',
      body: { name: reportForm.name, config: { group_by: groupBy.value, chart_type: chartType.value } }
    })
    reportOpen.value = false
    await refreshReports()
    toast.add({ title: 'Report saved', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    reportError.value = e?.data?.message || e?.statusMessage || 'Failed to save report'
  } finally {
    savingReport.value = false
  }
}

function openReport(report: Report) {
  const config = report.config || {}
  if (typeof config.group_by === 'string' && config.group_by) groupBy.value = config.group_by
  if (config.chart_type === 'pie' || config.chart_type === 'bar') chartType.value = config.chart_type
}

function confirmDeleteReport(report: Report) {
  reportToDelete.value = report
  deleteReportOpen.value = true
}

async function removeReport() {
  if (!reportToDelete.value) return
  deletingReport.value = true
  try {
    await $fetch(`/api/meta/reports/${encodeURIComponent(reportToDelete.value.id)}`, { method: 'DELETE' })
    deleteReportOpen.value = false
    reportToDelete.value = null
    await refreshReports()
    toast.add({ title: 'Report deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to delete report',
      description: e?.data?.message || e?.statusMessage || 'Delete failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    deletingReport.value = false
  }
}
</script>

<template>
  <UDashboardPanel id="reports">
    <template #header>
      <UDashboardNavbar title="Reports">
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
            v-model="entityId"
            :items="(entities || []).map(e => ({ label: e.label, value: e.id }))"
            value-key="value"
            class="w-56"
            aria-label="Select entity"
          />
          <USelectMenu
            v-model="groupBy"
            :items="groupableFields.map(f => ({ label: f.name, value: f.name }))"
            value-key="value"
            class="w-48"
            aria-label="Group by field"
            placeholder="Group by…"
          />
          <USelectMenu
            v-model="chartType"
            :items="chartItems"
            value-key="value"
            class="w-32"
            aria-label="Chart type"
          />
          <UButton variant="outline" icon="i-lucide-download" :disabled="!buckets?.length" @click="exportBucketsCsv">Export CSV</UButton>
          <UButton v-if="isAdmin" icon="i-lucide-plus" :disabled="!groupBy" @click="openAddReport">Save report</UButton>
        </div>

        <UAlert v-if="error" color="error" title="Cannot load report" :description="error.message">
          <template #actions>
            <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
          </template>
        </UAlert>

        <div v-else-if="status === 'pending'" class="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <USkeleton class="h-64 w-full" />
          <USkeleton class="h-64 w-full" />
        </div>

        <div v-else-if="!groupBy" class="flex flex-col items-center gap-3 py-16 text-center">
          <UIcon name="i-lucide-chart-bar" class="h-10 w-10 text-muted" />
          <p class="text-sm text-muted">This entity has no select fields to group by.</p>
        </div>

        <div v-else-if="!buckets?.length" class="flex flex-col items-center gap-3 py-16 text-center">
          <UIcon name="i-lucide-inbox" class="h-10 w-10 text-muted" />
          <p class="text-sm text-muted">No records yet for this report.</p>
          <UButton icon="i-lucide-plus" :to="`/app/${encodeURIComponent(entityId)}`">Create first record</UButton>
        </div>

        <div v-else class="grid grid-cols-1 gap-4 lg:grid-cols-2">
          <UCard>
            <template #header>
              <div class="flex items-center justify-between">
                <h2 class="text-sm font-semibold">{{ groupBy }} distribution</h2>
                <UBadge color="neutral" variant="subtle">{{ total }} total</UBadge>
              </div>
            </template>
            <div class="h-64">
              <ClientOnly>
                <Bar v-if="chartType === 'bar'" :data="chartData" :options="chartOptions" />
                <Pie v-else :data="chartData" :options="chartOptions" />
                <template #fallback>
                  <div class="flex h-full items-center justify-center text-sm text-muted">Loading chart…</div>
                </template>
              </ClientOnly>
            </div>
          </UCard>
          <UCard>
            <template #header>
              <h2 class="text-sm font-semibold">Buckets</h2>
            </template>
            <UTable
              :data="buckets || []"
              :columns="[
                { accessorKey: 'status', header: groupBy, cell: ({ row }) => row.original.status || '—' },
                { accessorKey: 'count', header: 'Count' }
              ]"
              :get-row-id="(row: Bucket) => row.status"
              class="w-full"
            />
          </UCard>
        </div>

        <UCard>
          <template #header>
            <div class="flex items-center justify-between">
              <h2 class="text-sm font-semibold">Saved reports</h2>
              <p class="text-xs text-muted">{{ (reports || []).length }} reports</p>
            </div>
          </template>
          <div v-if="!(reports || []).length" class="py-4 text-center text-sm text-muted">
            No saved reports for this entity yet.
          </div>
          <div v-else class="space-y-2">
            <div v-for="report in reports || []" :key="report.id" class="flex items-center justify-between gap-4 rounded-lg border border-default px-4 py-3">
              <div>
                <p class="text-sm font-medium">{{ report.name }}</p>
                <p class="font-mono text-xs text-muted">{{ String(report.config?.group_by || '') }} · {{ String(report.config?.chart_type || 'bar') }}</p>
              </div>
              <div class="flex items-center gap-1">
                <UButton size="xs" variant="ghost" @click="openReport(report)">Open</UButton>
                <UButton v-if="isAdmin" size="xs" variant="ghost" color="error" @click="confirmDeleteReport(report)">Delete</UButton>
              </div>
            </div>
          </div>
        </UCard>
      </div>

      <UModal v-model:open="reportOpen" title="Save report">
        <template #body>
          <UForm class="space-y-4" @submit="saveReport">
            <UFormField label="Name" hint="e.g. Work orders by status">
              <UInput v-model="reportForm.name" placeholder="By status" />
            </UFormField>
            <UAlert v-if="reportError" color="error" :title="reportError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="reportOpen = false">Cancel</UButton>
            <UButton :loading="savingReport" @click="saveReport">Save report</UButton>
          </div>
        </template>
      </UModal>

      <UModal v-model:open="deleteReportOpen" title="Delete report">
        <template #body>
          <p class="text-sm text-muted">
            This will permanently delete the report
            <span class="font-mono">{{ reportToDelete?.name }}</span>. This action cannot be undone.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="deleteReportOpen = false">Cancel</UButton>
            <UButton color="error" :loading="deletingReport" @click="removeReport">Delete</UButton>
          </div>
        </template>
      </UModal>
    </template>
  </UDashboardPanel>
</template>
