<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

import { GridItem, GridLayout } from 'grid-layout-plus'
import { Bar, Line, Pie } from 'vue-chartjs'
import {
  ArcElement,
  BarElement,
  CategoryScale,
  Chart as ChartJS,
  Legend,
  LinearScale,
  LineElement,
  PointElement,
  Title,
  Tooltip
} from 'chart.js'

ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale, LineElement, PointElement, ArcElement)

type CoreDashboard = { id: string; name: string; description: string; layout: unknown[]; filters: Record<string, unknown>; roles: string[]; users: string[]; active: boolean }
type Entity = { id: string; label: string }
type EntityField = { id: string; name: string; type: string; is_status: boolean }
type EntityDetail = Entity & { fields: EntityField[] }
type ReportFilter = { field: string; op: string; value: string }
type ReportAggregate = { field: string; op: string; alias: string }
type ReportSort = { field: string; direction: string }
type WidgetConfig = {
  fields: string[]
  filters: ReportFilter[]
  group_by: string[]
  sort: ReportSort[]
  aggregates: ReportAggregate[]
}
type DashboardWidget = {
  id: string
  kind: string
  title: string
  entity_id: string
  config: WidgetConfig
  x: number
  y: number
  w: number
  h: number
}
type WidgetResult = { widget: DashboardWidget; result: { columns: string[]; rows: Record<string, unknown>[]; total: number } }

const WIDGET_KINDS = [
  { label: 'KPI', value: 'kpi', icon: 'i-lucide-gauge' },
  { label: 'Table', value: 'table', icon: 'i-lucide-table' },
  { label: 'List', value: 'list', icon: 'i-lucide-list' },
  { label: 'Bar chart', value: 'bar', icon: 'i-lucide-chart-bar' },
  { label: 'Line chart', value: 'line', icon: 'i-lucide-chart-line' },
  { label: 'Pie chart', value: 'pie', icon: 'i-lucide-chart-pie' },
  { label: 'Area chart', value: 'area', icon: 'i-lucide-chart-area' }
]
const FILTER_OPS = [
  { label: 'equals', value: 'eq' },
  { label: 'not equals', value: 'neq' },
  { label: 'contains', value: 'contains' },
  { label: 'starts with', value: 'starts_with' },
  { label: 'ends with', value: 'ends_with' },
  { label: 'greater than', value: 'gt' },
  { label: 'greater or equal', value: 'gte' },
  { label: 'less than', value: 'lt' },
  { label: 'less or equal', value: 'lte' },
  { label: 'in list', value: 'in' },
  { label: 'is empty', value: 'is_null' },
  { label: 'is not empty', value: 'not_null' }
]
const AGGREGATE_OPS = [
  { label: 'Count', value: 'count' },
  { label: 'Sum', value: 'sum' },
  { label: 'Average', value: 'avg' },
  { label: 'Min', value: 'min' },
  { label: 'Max', value: 'max' }
]
const SORT_DIRECTIONS = [
  { label: 'Ascending', value: 'asc' },
  { label: 'Descending', value: 'desc' }
]
const CHART_COLORS = ['#00A155', '#3B82F6', '#F59E0B', '#EF4444', '#8B5CF6', '#06B6D4', '#EC4899']

const toast = useToast()
const dashboards = ref<CoreDashboard[]>([])
const selected = ref<CoreDashboard | null>(null)
const saving = ref(false)
const loading = ref(false)
const loadError = ref('')
const saveError = ref('')
const form = reactive({ name: '', description: '', active: true, roles: '', users: '', filters: '{}' })
const widgets = ref<DashboardWidget[]>([])
const widgetsDirty = ref(false)
const widgetResults = ref<Record<string, WidgetResult['result']>>({})
const previewing = ref<Record<string, boolean>>({})
const previewErrors = ref<Record<string, string>>({})

const { data: entities } = await useFetch<Entity[]>('/api/entities')

function emptyConfig(): WidgetConfig {
  return { fields: [], filters: [], group_by: [], sort: [], aggregates: [] }
}

function toWidget(raw: unknown, index: number): DashboardWidget {
  const w = (raw || {}) as Record<string, unknown>
  const config = (w.config || {}) as Record<string, unknown>
  const asStrings = (v: unknown): string[] => Array.isArray(v) ? v.map(x => String(x)) : []
  return {
    id: typeof w.id === 'string' && w.id ? w.id : `widget_${Date.now()}_${index}`,
    kind: typeof w.kind === 'string' && w.kind ? w.kind : 'kpi',
    title: typeof w.title === 'string' ? w.title : '',
    entity_id: typeof w.entity_id === 'string' ? w.entity_id : '',
    config: {
      fields: asStrings(config.fields),
      filters: Array.isArray(config.filters) ? (config.filters as ReportFilter[]) : [],
      group_by: asStrings(config.group_by),
      sort: Array.isArray(config.sort) ? (config.sort as ReportSort[]) : [],
      aggregates: Array.isArray(config.aggregates) ? (config.aggregates as ReportAggregate[]) : []
    },
    x: Number(w.x) || 0,
    y: Number(w.y) || 0,
    w: Math.max(1, Number(w.w) || 4),
    h: Math.max(1, Number(w.h) || 4)
  }
}

function markWidgetsDirty() {
  widgetsDirty.value = true
}

// grid-layout-plus binding: the grid needs `{i,x,y,w,h}` items keyed by
// widget id. Two-way sync: layout edits flow back through
// `onGridLayoutUpdated`; widget add/remove flows forward through this
// computed's getter.
const gridLayoutItems = computed({
  get: () => widgets.value.map(w => ({ i: w.id, x: w.x, y: w.y, w: w.w, h: w.h })),
  set: (items: { i: string | number; x: number; y: number; w: number; h: number }[]) => {
    onGridLayoutUpdated(items)
  }
})

function onGridLayoutUpdated(newLayout: { i: string | number; x: number; y: number; w: number; h: number }[]) {
  const byId = new Map(newLayout.map(item => [String(item.i), item]))
  let changed = false
  for (const widget of widgets.value) {
    const item = byId.get(widget.id)
    if (!item) continue
    if (widget.x !== item.x || widget.y !== item.y || widget.w !== item.w || widget.h !== item.h) {
      widget.x = item.x
      widget.y = item.y
      widget.w = item.w
      widget.h = item.h
      changed = true
    }
  }
  if (changed) markWidgetsDirty()
}

async function load() {
  loading.value = true
  loadError.value = ''
  try {
    dashboards.value = await $fetch<CoreDashboard[]>('/api/meta/dashboards')
  } catch (cause: any) {
    loadError.value = cause?.data?.message || cause?.statusMessage || 'Unable to load dashboards'
  } finally {
    loading.value = false
  }
}
function select(d: CoreDashboard) {
  selected.value = d
  Object.assign(form, { name: d.name, description: d.description, active: d.active, roles: d.roles.join(','), users: d.users.join(','), filters: JSON.stringify(d.filters, null, 2) })
  widgets.value = Array.isArray(d.layout) ? d.layout.map((w, i) => toWidget(w, i)) : []
  widgetsDirty.value = false
  widgetResults.value = {}
  previewErrors.value = {}
}
function reset() {
  selected.value = null
  Object.assign(form, { name: '', description: '', active: true, roles: '', users: '', filters: '{}' })
  widgets.value = []
  widgetsDirty.value = false
  widgetResults.value = {}
  previewErrors.value = {}
}
async function save() {
  saving.value = true
  saveError.value = ''
  try {
    const payload = {
      name: form.name,
      description: form.description,
      active: form.active,
      roles: form.roles.split(',').map(x => x.trim()).filter(Boolean),
      users: form.users.split(',').map(x => x.trim()).filter(Boolean),
      layout: widgets.value,
      filters: JSON.parse(form.filters)
    }
    if (selected.value) await $fetch(`/api/meta/dashboards/${selected.value.id}`, { method: 'PUT', body: payload })
    else await $fetch('/api/meta/dashboards', { method: 'POST', body: payload })
    await load(); reset()
  } catch (cause: any) {
    saveError.value = cause?.data?.message || cause?.statusMessage || 'Unable to save dashboard'
  } finally { saving.value = false }
}
async function remove(id: string) {
  try {
    await $fetch(`/api/meta/dashboards/${id}`, { method: 'DELETE' }); await load(); if (selected.value?.id === id) reset()
  } catch (cause: any) {
    toast.add({ title: 'Unable to delete dashboard', description: cause?.data?.message || cause?.statusMessage || 'Delete failed', color: 'error', icon: 'i-lucide-alert-circle' })
  }
}

// --- Widget editor (add/edit panel) ---
const widgetOpen = ref(false)
const editingWidget = ref<DashboardWidget | null>(null)
const widgetForm = reactive({ kind: 'kpi', title: '', entity_id: '' })
const widgetConfig = ref<WidgetConfig>(emptyConfig())
const widgetError = ref('')
const widgetDetailUrl = computed(() => widgetForm.entity_id ? `/api/entities/${encodeURIComponent(widgetForm.entity_id)}` : '')
const { data: widgetDetail } = await useFetch<EntityDetail>(widgetDetailUrl, { immediate: false, watch: [widgetDetailUrl] })
const widgetFields = computed(() => widgetDetail.value?.fields || [])
const widgetFieldItems = computed(() => widgetFields.value.map(f => ({ label: `${f.name} (${f.type})`, value: f.name })))
const widgetSelectFields = computed(() => widgetFields.value.filter(f => f.type === 'select').map(f => ({ label: f.name, value: f.name })))
const widgetNumberFields = computed(() => widgetFields.value.filter(f => ['number', 'decimal', 'currency', 'percentage', 'integer'].includes(f.type)).map(f => ({ label: f.name, value: f.name })))

function openAddWidget() {
  editingWidget.value = null
  widgetForm.kind = 'kpi'
  widgetForm.title = ''
  widgetForm.entity_id = (entities.value || [])[0]?.id || ''
  widgetConfig.value = emptyConfig()
  widgetError.value = ''
  widgetOpen.value = true
}

function openEditWidget(widget: DashboardWidget) {
  editingWidget.value = widget
  widgetForm.kind = widget.kind
  widgetForm.title = widget.title
  widgetForm.entity_id = widget.entity_id
  widgetConfig.value = JSON.parse(JSON.stringify(widget.config)) as WidgetConfig
  widgetError.value = ''
  widgetOpen.value = true
}

function addFilterRow() {
  widgetConfig.value.filters.push({ field: widgetFields.value[0]?.name || '', op: 'eq', value: '' })
}

function removeFilterRow(index: number) {
  widgetConfig.value.filters.splice(index, 1)
}

function addAggregateRow() {
  widgetConfig.value.aggregates.push({ field: '', op: 'count', alias: '' })
}

function removeAggregateRow(index: number) {
  widgetConfig.value.aggregates.splice(index, 1)
}

function addSortRow() {
  widgetConfig.value.sort.push({ field: widgetFields.value[0]?.name || '', direction: 'asc' })
}

function removeSortRow(index: number) {
  widgetConfig.value.sort.splice(index, 1)
}

function saveWidget() {
  widgetError.value = ''
  if (!widgetForm.entity_id) {
    widgetError.value = 'entity is required'
    return
  }
  if (!widgetForm.title.trim()) {
    widgetError.value = 'title is required'
    return
  }
  const config = widgetConfig.value
  if (widgetForm.kind === 'kpi' && !config.aggregates.length) {
    widgetError.value = 'KPI widgets need at least one aggregate (e.g. count)'
    return
  }
  if (['bar', 'line', 'pie', 'area'].includes(widgetForm.kind) && !config.group_by.length) {
    widgetError.value = 'chart widgets need at least one group-by field'
    return
  }
  if (editingWidget.value) {
    editingWidget.value.kind = widgetForm.kind
    editingWidget.value.title = widgetForm.title.trim()
    editingWidget.value.entity_id = widgetForm.entity_id
    editingWidget.value.config = JSON.parse(JSON.stringify(config)) as WidgetConfig
  } else {
    const maxY = widgets.value.reduce((max, w) => Math.max(max, w.y + w.h), 0)
    widgets.value.push({
      id: `widget_${Date.now()}`,
      kind: widgetForm.kind,
      title: widgetForm.title.trim(),
      entity_id: widgetForm.entity_id,
      config: JSON.parse(JSON.stringify(config)) as WidgetConfig,
      x: 0,
      y: maxY,
      w: widgetForm.kind === 'table' ? 12 : 6,
      h: widgetForm.kind === 'table' ? 6 : 4
    })
  }
  widgetOpen.value = false
  markWidgetsDirty()
}

function removeWidget(id: string) {
  widgets.value = widgets.value.filter(w => w.id !== id)
  delete widgetResults.value[id]
  delete previewErrors.value[id]
  markWidgetsDirty()
}

// --- Widget live preview (ad-hoc report run, no save needed) ---
async function previewWidget(widget: DashboardWidget) {
  if (!widget.entity_id) return
  previewing.value[widget.id] = true
  delete previewErrors.value[widget.id]
  try {
    const result = await $fetch<{ columns: string[]; rows: Record<string, unknown>[]; total: number }>(
      `/api/entities/${encodeURIComponent(widget.entity_id)}/reports/preview`,
      { method: 'POST', body: { config: widget.config } }
    )
    widgetResults.value[widget.id] = result
  } catch (cause: any) {
    previewErrors.value[widget.id] = cause?.data?.message || cause?.statusMessage || 'Preview failed'
  } finally {
    previewing.value[widget.id] = false
  }
}

function widgetChartData(widget: DashboardWidget) {
  const result = widgetResults.value[widget.id]
  const groupField = widget.config.group_by[0] || 'group'
  const valueField = result?.columns.find(c => c !== groupField) || result?.columns[0] || 'value'
  return {
    labels: (result?.rows || []).map(r => String(r[groupField] ?? '—')),
    datasets: [{
      label: widget.title,
      data: (result?.rows || []).map(r => Number(r[valueField]) || 0),
      backgroundColor: CHART_COLORS
    }]
  }
}

const widgetChartOptions = { responsive: true, maintainAspectRatio: false, plugins: { legend: { display: false }, title: { display: false } } }

function widgetKpiValue(widget: DashboardWidget): string {
  const result = widgetResults.value[widget.id]
  if (!result?.rows?.length) return '—'
  const first = result.rows[0] as Record<string, unknown>
  const value = first[result.columns[0] || ''] ?? Object.values(first)[0]
  return String(value ?? '—')
}

await load()
</script>

<template>
  <UDashboardPanel id="dashboards">
    <template #header>
      <UDashboardNavbar title="Dashboard Builder">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-5xl space-y-4">
        <p class="text-sm text-muted">Generic dashboards powered by saved report configurations.</p>
        <div class="flex justify-end">
          <UButton icon="i-lucide-plus" @click="reset">New dashboard</UButton>
        </div>
        <UAlert v-if="loadError" color="error" title="Cannot load dashboards" :description="loadError">
          <template #actions>
            <UButton size="sm" variant="outline" @click="load()">Retry</UButton>
          </template>
        </UAlert>
        <div v-else-if="loading" class="grid gap-6 lg:grid-cols-[280px_1fr]">
          <USkeleton class="h-48 w-full" />
          <USkeleton class="h-48 w-full" />
        </div>
        <div v-else class="grid gap-6 lg:grid-cols-[280px_1fr]">
          <UCard>
            <div v-if="!dashboards.length" class="py-8 text-center text-sm text-muted">No dashboards yet.</div>
            <div v-else class="space-y-2"><button v-for="d in dashboards" :key="d.id" class="w-full rounded px-3 py-2 text-left hover:bg-muted" @click="select(d)"><div class="font-medium">{{ d.name }}</div><div class="text-xs text-muted">{{ d.layout.length }} widgets</div></button></div>
          </UCard>
          <UCard>
            <div class="grid gap-4 md:grid-cols-2"><UInput v-model="form.name" placeholder="Dashboard name" /><UInput v-model="form.description" placeholder="Description" /><UInput v-model="form.roles" placeholder="Roles, comma separated" /><UInput v-model="form.users" placeholder="Users, comma separated" /><UTextarea v-model="form.filters" :rows="5" placeholder="Dashboard filters JSON" /></div>
            <div class="mt-4 flex items-center justify-between">
              <p class="text-sm text-muted">{{ widgets.length }} widgets · drag to move, resize from the corner</p>
              <UButton size="sm" icon="i-lucide-plus" @click="openAddWidget">Add widget</UButton>
            </div>
            <div v-if="!widgets.length" class="mt-2 rounded border border-dashed border-default px-3 py-6 text-center text-sm text-muted">
              No widgets yet. Add a KPI, table, or chart widget to build this dashboard — no JSON needed.
            </div>
            <ClientOnly>
              <GridLayout
                v-if="widgets.length"
                v-model:layout="gridLayoutItems"
                :col-num="12"
                :row-height="48"
                :margin="[8, 8]"
                :is-draggable="true"
                :is-resizable="true"
                :vertical-compact="true"
                :use-css-transforms="true"
                class="mt-2"
                @layout-updated="onGridLayoutUpdated"
              >
                <GridItem
                  v-for="widget in widgets"
                  :key="widget.id"
                  :i="widget.id"
                  :x="widget.x"
                  :y="widget.y"
                  :w="widget.w"
                  :h="widget.h"
                  :min-w="2"
                  :min-h="2"
                  drag-allow-from=".widget-drag-handle"
                  drag-ignore-from=".widget-no-drag"
                >
                  <UCard class="h-full">
                    <template #header>
                      <div class="flex items-center justify-between gap-2">
                        <div class="flex min-w-0 items-center gap-1.5">
                          <UIcon name="i-lucide-grip-vertical" class="widget-drag-handle size-4 shrink-0 cursor-grab text-muted active:cursor-grabbing" :title="`Drag ${widget.title} to move`" />
                          <p class="truncate text-sm font-semibold">{{ widget.title || '(untitled)' }}</p>
                          <UBadge color="neutral" variant="subtle" size="xs">{{ widget.kind }}</UBadge>
                        </div>
                        <div class="widget-no-drag flex shrink-0 gap-1">
                          <UButton size="xs" variant="ghost" icon="i-lucide-play" :loading="previewing[widget.id]" @click="previewWidget(widget)" />
                          <UButton size="xs" variant="ghost" icon="i-lucide-pencil" @click="openEditWidget(widget)" />
                          <UButton size="xs" variant="ghost" color="error" icon="i-lucide-trash" @click="removeWidget(widget.id)" />
                        </div>
                      </div>
                    </template>
                    <div class="widget-no-drag min-h-24">
                      <div v-if="previewErrors[widget.id]" class="py-2 text-center text-xs text-error">{{ previewErrors[widget.id] }}</div>
                      <div v-else-if="!widgetResults[widget.id]" class="flex flex-col items-center gap-2 py-4 text-center">
                        <p class="text-xs text-muted">No preview yet.</p>
                        <UButton size="xs" variant="outline" icon="i-lucide-play" @click="previewWidget(widget)">Run preview</UButton>
                      </div>
                      <div v-else-if="widget.kind === 'kpi'" class="py-2 text-center">
                        <p class="text-3xl font-semibold">{{ widgetKpiValue(widget) }}</p>
                      </div>
                      <div v-else-if="widget.kind === 'table' || widget.kind === 'list'" class="max-h-64 overflow-auto">
                        <table class="w-full text-xs">
                          <thead><tr><th v-for="col in (widgetResults[widget.id]?.columns || [])" :key="col" class="px-2 py-1 text-left font-medium">{{ col }}</th></tr></thead>
                          <tbody><tr v-for="(row, ri) in (widgetResults[widget.id]?.rows || []).slice(0, 10)" :key="ri"><td v-for="col in (widgetResults[widget.id]?.columns || [])" :key="col" class="border-t border-default px-2 py-1">{{ String(row[col] ?? '—') }}</td></tr></tbody>
                        </table>
                      </div>
                      <div v-else class="h-48">
                        <Bar v-if="widget.kind === 'bar'" :data="widgetChartData(widget)" :options="widgetChartOptions" />
                        <Line v-else-if="widget.kind === 'line' || widget.kind === 'area'" :data="widgetChartData(widget)" :options="widgetChartOptions" />
                        <Pie v-else-if="widget.kind === 'pie'" :data="widgetChartData(widget)" :options="widgetChartOptions" />
                      </div>
                    </div>
                  </UCard>
                </GridItem>
              </GridLayout>
              <template #fallback>
                <div class="mt-2 space-y-2">
                  <UCard v-for="widget in widgets" :key="widget.id">
                    <div class="flex items-center justify-between">
                      <p class="text-sm font-semibold">{{ widget.title || '(untitled)' }}</p>
                      <UBadge color="neutral" variant="subtle" size="xs">{{ widget.kind }}</UBadge>
                    </div>
                  </UCard>
                </div>
              </template>
            </ClientOnly>
            <UBadge v-if="widgetsDirty" class="mt-2" color="warning" variant="subtle">Unsaved widget changes</UBadge>
            <UAlert v-if="saveError" class="mt-4" color="error" :title="saveError" />
            <div class="mt-4 flex gap-2"><UCheckbox v-model="form.active" label="Active" /><UButton :loading="saving" label="Save" @click="save" /><UButton v-if="selected" color="error" variant="soft" label="Delete" @click="remove(selected.id)" /></div>
          </UCard>
        </div>
      </div>

      <USlideover v-model:open="widgetOpen" :title="editingWidget ? 'Edit widget' : 'Add widget'">
        <template #body>
          <div class="space-y-4">
            <UFormField label="Widget type">
              <USelectMenu v-model="widgetForm.kind" :items="WIDGET_KINDS" value-key="value" class="w-full" />
            </UFormField>
            <UFormField label="Title">
              <UInput v-model="widgetForm.title" placeholder="e.g. Orders by status" class="w-full" />
            </UFormField>
            <UFormField label="Entity">
              <USelectMenu v-model="widgetForm.entity_id" :items="(entities || []).map(e => ({ label: e.label, value: e.id }))" value-key="value" class="w-full" />
            </UFormField>
            <UFormField label="Fields" hint="Columns shown for table/list widgets">
              <USelectMenu v-model="widgetConfig.fields" :items="widgetFieldItems" value-key="value" multiple class="w-full" placeholder="Pick fields…" />
            </UFormField>
            <div>
              <div class="mb-2 flex items-center justify-between">
                <p class="text-sm font-medium">Filters</p>
                <UButton size="xs" variant="outline" icon="i-lucide-plus" @click="addFilterRow">Add filter</UButton>
              </div>
              <div v-if="!widgetConfig.filters.length" class="py-2 text-center text-xs text-muted">No filters — all records included.</div>
              <div v-for="(filter, fi) in widgetConfig.filters" :key="fi" class="mb-2 flex items-center gap-2">
                <USelectMenu v-model="filter.field" :items="widgetFieldItems" value-key="value" size="xs" class="w-32" aria-label="Filter field" />
                <USelectMenu v-model="filter.op" :items="FILTER_OPS" value-key="value" size="xs" class="w-32" aria-label="Filter operator" />
                <UInput v-model="filter.value" size="xs" class="flex-1" placeholder="Value" />
                <UButton size="xs" variant="ghost" color="error" icon="i-lucide-trash" @click="removeFilterRow(fi)" />
              </div>
            </div>
            <UFormField label="Group by" hint="Required for chart widgets">
              <USelectMenu v-model="widgetConfig.group_by" :items="widgetSelectFields.length ? widgetSelectFields : widgetFieldItems" value-key="value" multiple class="w-full" placeholder="Pick group-by fields…" />
            </UFormField>
            <div>
              <div class="mb-2 flex items-center justify-between">
                <p class="text-sm font-medium">Aggregates</p>
                <UButton size="xs" variant="outline" icon="i-lucide-plus" @click="addAggregateRow">Add aggregate</UButton>
              </div>
              <div v-if="!widgetConfig.aggregates.length" class="py-2 text-center text-xs text-muted">No aggregates — KPI widgets need at least one (e.g. count).</div>
              <div v-for="(agg, ai) in widgetConfig.aggregates" :key="ai" class="mb-2 flex items-center gap-2">
                <USelectMenu v-model="agg.op" :items="AGGREGATE_OPS" value-key="value" size="xs" class="w-28" aria-label="Aggregate operation" />
                <USelectMenu v-model="agg.field" :items="agg.op === 'count' ? [{ label: '(any)', value: '' }, ...widgetNumberFields, ...widgetFieldItems] : (widgetNumberFields.length ? widgetNumberFields : widgetFieldItems)" value-key="value" size="xs" class="w-32" aria-label="Aggregate field" />
                <UInput v-model="agg.alias" size="xs" class="flex-1" placeholder="Alias (optional)" />
                <UButton size="xs" variant="ghost" color="error" icon="i-lucide-trash" @click="removeAggregateRow(ai)" />
              </div>
            </div>
            <div>
              <div class="mb-2 flex items-center justify-between">
                <p class="text-sm font-medium">Sort</p>
                <UButton size="xs" variant="outline" icon="i-lucide-plus" @click="addSortRow">Add sort</UButton>
              </div>
              <div v-if="!widgetConfig.sort.length" class="py-2 text-center text-xs text-muted">No sorting.</div>
              <div v-for="(s, si) in widgetConfig.sort" :key="si" class="mb-2 flex items-center gap-2">
                <USelectMenu v-model="s.field" :items="widgetFieldItems" value-key="value" size="xs" class="w-40" aria-label="Sort field" />
                <USelectMenu v-model="s.direction" :items="SORT_DIRECTIONS" value-key="value" size="xs" class="w-32" aria-label="Sort direction" />
                <UButton size="xs" variant="ghost" color="error" icon="i-lucide-trash" @click="removeSortRow(si)" />
              </div>
            </div>
            <UAlert v-if="widgetError" color="error" :title="widgetError" />
          </div>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="widgetOpen = false">Cancel</UButton>
            <UButton @click="saveWidget">{{ editingWidget ? 'Update widget' : 'Add widget' }}</UButton>
          </div>
        </template>
      </USlideover>
    </template>
  </UDashboardPanel>
</template>

