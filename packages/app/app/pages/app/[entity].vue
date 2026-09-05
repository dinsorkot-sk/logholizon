<script setup lang="ts">
import { h, resolveComponent } from 'vue'
import type { TableColumn } from '@nuxt/ui'

const UButton = resolveComponent('UButton')
const UCheckbox = resolveComponent('UCheckbox')
const UBadge = resolveComponent('UBadge')

type FieldOption = { id: string; value: string; label: string }
type Field = { id: string; name: string; type: string; required: boolean; options: FieldOption[] }
type Entity = { id: string; name: string; label: string; fields: Field[] }
type Document = { id: string; entity_id: string; payload: Record<string, unknown> }
type DocumentList = { items: Document[]; total: number }
type AuditEntry = { id: string; action: string; payload: Record<string, unknown>; created_at: string }
type AuditList = { items: AuditEntry[]; total: number }

const route = useRoute()
const router = useRouter()
const entityId = computed(() => String(route.params.entity || ''))
const panelOpen = ref(false)
const selected = ref<Document | null>(null)
const payload = reactive<Record<string, unknown>>({})
const saving = ref(false)
const deleting = ref(false)
const transitioning = ref(false)
const error = ref('')
const fieldErrors = reactive<Record<string, string>>({})
const exporting = ref(false)
const importing = ref(false)
const importPreview = ref<{ rows: { id: string; payload: Record<string, unknown> }[]; errors: string[] } | null>(null)
const importCsv = ref('')
const importInput = ref<HTMLInputElement | null>(null)
const toast = useToast()
const deleteOpen = ref(false)

// --- List state (search / filter / sort / pagination / bulk / columns) ---
const limit = ref(50)
const offset = ref(0)
const search = ref('')
const statusFilter = ref('all')
const sortBy = ref('')
const sortDir = ref('desc')
const selectedRows = ref<Set<string>>(new Set())
const visibleColumns = ref<Set<string>>(new Set())
const bulkDeleteOpen = ref(false)
const bulkDeleting = ref(false)

const { data: entities } = await useFetch<{ id: string; label: string }[]>('/api/meta/entities')
const { data: entity, status: entityStatus, error: entityError } = await useFetch<Entity>(
  () => `/api/meta/entities/${encodeURIComponent(entityId.value)}`,
  { watch: [entityId] }
)
const documentsUrl = computed(() => {
  const params = new URLSearchParams({
    entity_id: entityId.value,
    limit: String(limit.value),
    offset: String(offset.value)
  })
  if (search.value.trim()) params.set('search', search.value.trim())
  if (statusFilter.value && statusFilter.value !== 'all') params.set('status', statusFilter.value)
  if (sortBy.value) params.set('sort_by', sortBy.value)
  if (sortDir.value) params.set('sort_dir', sortDir.value)
  return `/api/documents?${params.toString()}`
})
const { data: documents, status: documentsStatus, error: documentsError, refresh } = await useFetch<DocumentList>(
  documentsUrl,
  { watch: [documentsUrl] }
)
const { data: workflow } = await useFetch<{ states: { name: string; label: string }[]; transitions: { action: string; from_state: string; to_state: string }[] }>(
  () => `/api/meta/entities/${encodeURIComponent(entityId.value)}/workflow`,
  { watch: [entityId] }
)
const auditId = computed(() => selected.value?.id || '')
const { data: audit, status: auditStatus, refresh: refreshAudit } = await useFetch<AuditList>(
  () => `/api/documents/${encodeURIComponent(auditId.value)}/audit`,
  { watch: [auditId], immediate: false }
)

function emptyPayload() {
  return Object.fromEntries((entity.value?.fields || []).map(field => [field.name, '']))
}

function openCreate() {
  selected.value = null
  Object.assign(payload, emptyPayload())
  Object.keys(fieldErrors).forEach(key => delete fieldErrors[key])
  error.value = ''
  panelOpen.value = true
}

function openEdit(document: Document) {
  selected.value = document
  Object.keys(payload).forEach(key => delete payload[key])
  Object.assign(payload, emptyPayload(), document.payload)
  Object.keys(fieldErrors).forEach(key => delete fieldErrors[key])
  error.value = ''
  panelOpen.value = true
  refreshAudit()
}

function validate() {
  Object.keys(fieldErrors).forEach(key => delete fieldErrors[key])
  Object.assign(fieldErrors, validatePayload(entity.value?.fields || [], payload))
  return Object.keys(fieldErrors).length === 0
}

function normalizedPayload() {
  return normalizePayload(entity.value?.fields || [], payload)
}

async function save() {
  if (!validate() || !entity.value) return
  saving.value = true
  error.value = ''
  const isEdit = !!selected.value
  try {
    const nextPayload = normalizedPayload()
    if (isEdit && selected.value) {
      await $fetch(`/api/documents/${encodeURIComponent(selected.value.id)}`, {
        method: 'PUT',
        body: { payload: nextPayload }
      })
    } else {
      await $fetch('/api/documents', {
        method: 'POST',
        body: { id: crypto.randomUUID(), entity_id: entity.value.id, payload: nextPayload }
      })
    }
    panelOpen.value = false
    await refresh()
    toast.add({ title: isEdit ? 'Record updated' : 'Record created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to save record'
    toast.add({ title: 'Unable to save record', description: error.value, color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    saving.value = false
  }
}

async function transition(action: string) {
  if (!selected.value) return
  transitioning.value = true
  error.value = ''
  try {
    await $fetch(`/api/documents/${encodeURIComponent(selected.value.id)}/transition`, { method: 'POST', body: { action } })
    await refresh()
    await refreshAudit()
    toast.add({ title: 'Record transitioned', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to transition record'
    toast.add({ title: 'Unable to transition record', description: error.value, color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    transitioning.value = false
  }
}

function availableActions() {
  const status = selected.value?.payload.status
  return workflow.value?.transitions.filter(item => item.from_state === status) || []
}

async function remove() {
  if (!selected.value) return
  deleting.value = true
  error.value = ''
  try {
    await $fetch(`/api/documents/${encodeURIComponent(selected.value.id)}`, { method: 'DELETE' })
    panelOpen.value = false
    deleteOpen.value = false
    await refresh()
    toast.add({ title: 'Record deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to delete record'
    toast.add({ title: 'Unable to delete record', description: error.value, color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    deleting.value = false
  }
}

function display(value: unknown) {
  if (value === '' || value === null || value === undefined) return '—'
  return String(value)
}

// --- Label mapping ---
function fieldLabel(field: Field, value: unknown) {
  if (value === '' || value === null || value === undefined) return '—'
  if (field.type === 'select') {
    const option = field.options.find(o => o.value === value)
    return option?.label || String(value)
  }
  return String(value)
}

const transitionLabels: Record<string, string> = {
  submit: 'Submit',
  approve: 'Approve',
  reject: 'Reject',
  done: 'Mark Done',
  complete: 'Complete',
  transition: 'Status changed',
  create: 'Created',
  update: 'Updated',
  delete: 'Deleted',
  import: 'Imported'
}

function transitionLabel(action: string) {
  return transitionLabels[action] || action
}

function relativeTime(iso: string) {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return iso
  const diff = Date.now() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  if (minutes < 1) return 'just now'
  if (minutes < 60) return `${minutes} min ago`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} hour${hours > 1 ? 's' : ''} ago`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days} day${days > 1 ? 's' : ''} ago`
  return date.toLocaleDateString()
}

// --- Pagination ---
const total = computed(() => documents.value?.total || 0)
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

// --- Sort ---
const sortItems = computed(() => {
  const items = [
    { label: 'Created (newest)', value: 'created__desc' },
    { label: 'Created (oldest)', value: 'created__asc' }
  ]
  for (const field of entity.value?.fields || []) {
    items.push({ label: `${field.name} (A–Z)`, value: `${field.name}__asc` })
    items.push({ label: `${field.name} (Z–A)`, value: `${field.name}__desc` })
  }
  return items
})
const sortValue = computed({
  get: () => (sortBy.value ? `${sortBy.value}__${sortDir.value}` : 'created__desc'),
  set: (value: string) => {
    const idx = value.lastIndexOf('__')
    if (idx === -1) {
      sortBy.value = ''
      sortDir.value = 'desc'
    } else {
      sortBy.value = value.slice(0, idx)
      sortDir.value = value.slice(idx + 2)
    }
    applyFilters()
  }
})

// --- Status filter ---
const statusField = computed(() => entity.value?.fields.find(f => f.name === 'status'))
const statusItems = computed(() => [
  { label: 'All statuses', value: 'all' },
  ...(statusField.value?.options || []).map(o => ({ label: o.label, value: o.value }))
])

// --- Column visibility ---
watch(entity, (e) => {
  if (e) {
    visibleColumns.value = new Set(e.fields.map(f => f.name))
  }
}, { immediate: true })

function toggleColumn(name: string) {
  const next = new Set(visibleColumns.value)
  if (next.has(name)) next.delete(name)
  else next.add(name)
  visibleColumns.value = next
}

// --- Bulk selection ---
const allSelected = computed(() => {
  const items = documents.value?.items || []
  return items.length > 0 && items.every(d => selectedRows.value.has(d.id))
})

function toggleAll() {
  const items = documents.value?.items || []
  const next = new Set(selectedRows.value)
  if (allSelected.value) {
    items.forEach(d => next.delete(d.id))
  } else {
    items.forEach(d => next.add(d.id))
  }
  selectedRows.value = next
}

function toggleRow(id: string) {
  const next = new Set(selectedRows.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedRows.value = next
}

async function bulkDelete() {
  bulkDeleting.value = true
  try {
    for (const id of selectedRows.value) {
      await $fetch(`/api/documents/${encodeURIComponent(id)}`, { method: 'DELETE' })
    }
    selectedRows.value = new Set()
    bulkDeleteOpen.value = false
    await refresh()
    toast.add({ title: 'Records deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    toast.add({
      title: 'Unable to delete records',
      description: cause?.data?.message || cause?.statusMessage || 'Delete failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    bulkDeleting.value = false
  }
}

// --- UTable (TanStack via Nuxt UI) ---
const tableData = computed(() => documents.value?.items || [])

const sorting = ref<{ id: string; desc: boolean }[]>([])
watch(sorting, (value) => {
  if (value[0]?.id) {
    sortBy.value = value[0].id.replace('payload.', '')
    sortDir.value = value[0].desc ? 'desc' : 'asc'
    applyFilters()
  }
})

const rowSelection = computed({
  get: () => Object.fromEntries([...selectedRows.value].map(id => [id, true])),
  set: (value: Record<string, boolean>) => {
    selectedRows.value = new Set(Object.keys(value).filter(k => value[k]))
  }
})

const columnVisibility = computed({
  get: () => Object.fromEntries([...visibleColumns.value].map(c => [c, true])),
  set: (value: Record<string, boolean>) => {
    visibleColumns.value = new Set(Object.keys(value).filter(k => value[k]))
  }
})

const tableColumns = computed<TableColumn<Document>[]>(() => {
  const cols: TableColumn<Document>[] = [{
    id: 'select',
    header: () => h(UCheckbox, {
      'model-value': allSelected.value,
      'onUpdate:model-value': toggleAll,
      'aria-label': 'Select all'
    }),
    cell: ({ row }) => h(UCheckbox, {
      'model-value': selectedRows.value.has(row.original.id),
      'onUpdate:model-value': () => toggleRow(row.original.id),
      'aria-label': `Select ${row.original.id}`
    }),
    meta: { class: { th: 'w-10', td: 'w-10' } }
  }]
  for (const field of entity.value?.fields.filter(f => visibleColumns.value.has(f.name)) || []) {
    cols.push({
      accessorKey: `payload.${field.name}`,
      header: field.name,
      enableSorting: true,
      cell: ({ row }) => {
        const value = row.original.payload[field.name]
        if (field.name === 'status') {
          return h(UBadge, { variant: 'subtle' }, () => fieldLabel(field, value))
        }
        return fieldLabel(field, value)
      }
    })
  }
  cols.push({
    id: 'actions',
    header: () => h('span', { class: 'sr-only' }, 'Actions'),
    cell: ({ row }) => h(UButton, {
      size: 'xs',
      variant: 'ghost',
      onClick: () => openEdit(row.original)
    }, () => 'Edit'),
    meta: { class: { td: 'text-right' } }
  })
  return cols
})

async function exportCsv() {
  exporting.value = true
  try {
    const csv = await $fetch<string>(`/api/meta/entities/${encodeURIComponent(entityId.value)}/export`)
    const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv;charset=utf-8' }))
    const link = document.createElement('a')
    link.href = url
    link.download = `${entityId.value}.csv`
    link.click()
    URL.revokeObjectURL(url)
    toast.add({ title: 'Export complete', description: `${entityId.value}.csv downloaded`, color: 'success', icon: 'i-lucide-download' })
  } catch (cause: any) {
    toast.add({ title: 'Unable to export', description: cause?.data?.message || cause?.statusMessage || 'Export failed', color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    exporting.value = false
  }
}

async function previewImport(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  importing.value = true
  error.value = ''
  try {
    importCsv.value = await file.text()
    importPreview.value = await $fetch<{ rows: { id: string; payload: Record<string, unknown> }[]; errors: string[] }>(
      `/api/meta/entities/${encodeURIComponent(entityId.value)}/import-preview` as '/api/meta/entities/[id]/import-preview',
      { method: 'POST', body: importCsv.value, headers: { 'content-type': 'text/csv' } }
    )
  } catch (cause: any) {
    importPreview.value = null
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to preview CSV'
  } finally {
    input.value = ''
    importing.value = false
  }
}

async function confirmImport() {
  if (!importCsv.value || importPreview.value?.errors.length) return
  importing.value = true
  error.value = ''
  try {
    const result = await $fetch<{ created: number; updated: number }>(`/api/meta/entities/${encodeURIComponent(entityId.value)}/import-confirm`, { method: 'POST', body: importCsv.value, headers: { 'content-type': 'text/csv' } })
    importPreview.value = null
    importCsv.value = ''
    await refresh()
    toast.add({ title: 'Import complete', description: `${result.created} created, ${result.updated} updated`, color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to import CSV'
    toast.add({ title: 'Unable to import CSV', description: error.value, color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    importing.value = false
  }
}
</script>

<template>
  <UDashboardPanel :id="`entity-${entityId}`">
    <template #header>
      <UDashboardNavbar :title="entity?.label || entityId">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton icon="i-lucide-plus" @click="openCreate">New record</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
    <UAlert
      v-if="entityStatus === 'error'"
      color="error"
      title="Cannot load entity"
      :description="entityError?.message || 'Check the entity ID and Rust core connection.'"
    />

    <template v-else-if="entity">
      <p class="font-mono text-sm text-muted">{{ entity.name }}</p>
      <div class="mb-4 flex flex-wrap items-center gap-2">
          <USelectMenu
            :model-value="entityId"
            :items="(entities || []).map(e => ({ label: e.label, value: e.id }))"
            value-key="value"
            class="w-48"
            aria-label="Select entity"
            @update:model-value="(value: string) => router.push(`/app/${encodeURIComponent(value)}`)"
          />
          <UInput
            v-model="search"
            icon="i-lucide-search"
            placeholder="Search…"
            class="w-56"
            @keyup.enter="applyFilters"
          >
            <template v-if="search" #trailing>
              <UButton size="xs" variant="link" color="neutral" icon="i-lucide-x" @click="search = ''; applyFilters()" />
            </template>
          </UInput>
          <USelectMenu
            v-if="statusField"
            v-model="statusFilter"
            :items="statusItems"
            value-key="value"
            class="w-40"
            @update:model-value="applyFilters"
          />
          <USelectMenu v-model="sortValue" :items="sortItems" value-key="value" class="w-48" />
          <UPopover>
            <UButton variant="outline" icon="i-lucide-settings-2">Columns</UButton>
            <template #content>
              <div class="w-52 p-2">
                <label v-for="field in entity.fields" :key="field.id" class="flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-sm hover:bg-default">
                  <UCheckbox :model-value="visibleColumns.has(field.name)" @update:model-value="toggleColumn(field.name)" />
                  <span class="font-mono">{{ field.name }}</span>
                </label>
              </div>
            </template>
          </UPopover>
          <UButton variant="outline" icon="i-lucide-download" :loading="exporting" @click="exportCsv">Export CSV</UButton>
          <input ref="importInput" type="file" accept=".csv,text/csv" class="hidden" @change="previewImport">
          <UButton variant="outline" icon="i-lucide-upload" :loading="importing" @click="importInput?.click()">Import CSV</UButton>
      </div>
        <UAlert v-if="importPreview" class="w-full" :color="importPreview.errors.length ? 'error' : 'success'" :title="`${importPreview.rows.length} rows previewed`" :description="importPreview.errors.join('; ') || 'Ready to import.'">
          <template #actions><UButton :disabled="!!importPreview.errors.length" :loading="importing" size="sm" @click="confirmImport">Confirm import</UButton></template>
        </UAlert>

      <UAlert
        v-if="!entity.fields.length"
        color="warning"
        title="This entity has no fields"
        description="Add fields in Entity Manager before creating records."
        class="mb-4"
      />

      <UCard>
        <div v-if="documentsStatus === 'pending'" class="space-y-3" aria-busy="true">
          <USkeleton v-for="index in 4" :key="index" class="h-10 w-full" />
        </div>
        <UAlert
          v-else-if="documentsStatus === 'error'"
          color="error"
          title="Cannot load records"
          :description="documentsError?.message || 'Check the Rust core connection.'"
          class="m-4"
        >
          <template #actions>
            <UButton size="sm" variant="outline" @click="refresh()">Retry</UButton>
          </template>
        </UAlert>
        <template v-else>
          <div v-if="selectedRows.size" class="flex items-center gap-2 border-b px-3 py-2">
            <p class="text-sm font-medium">{{ selectedRows.size }} selected</p>
            <UButton size="xs" color="error" variant="ghost" @click="bulkDeleteOpen = true">Delete</UButton>
            <UButton size="xs" variant="ghost" @click="selectedRows = new Set()">Clear</UButton>
          </div>
          <UTable
            :data="tableData"
            :columns="tableColumns"
            v-model:sorting="sorting"
            v-model:row-selection="rowSelection"
            v-model:column-visibility="columnVisibility"
            :get-row-id="(row: Document) => row.id"
            class="w-full"
          >
            <template #empty>
              <div class="py-10 text-center">
                <p class="text-sm text-muted">No records yet for {{ entity.label }}.</p>
                <UButton size="sm" icon="i-lucide-plus" class="mt-2" @click="openCreate">Create first record</UButton>
              </div>
            </template>
          </UTable>
          <div v-if="total > limit" class="flex items-center justify-between border-t px-3 py-2">
            <p class="text-sm text-muted">Showing {{ pageStart }}–{{ pageEnd }} of {{ total }}</p>
            <div class="flex gap-2">
              <UButton size="sm" variant="outline" :disabled="!hasPrev" @click="prevPage">Prev</UButton>
              <UButton size="sm" variant="outline" :disabled="!hasNext" @click="nextPage">Next</UButton>
            </div>
          </div>
        </template>
      </UCard>

      <USlideover v-model:open="panelOpen" :title="selected ? `Edit ${entity.label}` : `New ${entity.label}`">
        <template #body>
          <UForm class="space-y-4" @submit="save">
            <UFormField
              v-for="field in entity.fields.filter(item => item.name !== 'status')"
              :key="field.id"
              :label="field.name"
              :required="field.required"
              :error="fieldErrors[field.name]"
            >
              <USelectMenu
                v-if="field.type === 'select' && field.name !== 'status'"
                v-model="payload[field.name] as string"
                :items="[{ label: 'Select…', value: '' }, ...field.options.map(o => ({ label: o.label, value: o.value }))]"
                value-key="value"
                class="w-full"
              />
              <UInput v-else v-model="payload[field.name] as string" :type="field.type === 'date' ? 'date' : field.type === 'number' ? 'number' : 'text'" />
            </UFormField>
            <UAlert v-if="error" color="error" :title="error" />
            <div v-if="selected" class="border-t pt-4">
              <h2 class="mb-2 text-sm font-semibold">History</h2>
              <div v-if="auditStatus === 'pending'" class="py-4 text-sm text-gray-500">Loading history…</div>
              <UAlert v-else-if="auditStatus === 'error'" color="error" title="Cannot load history" />
              <ol v-else class="space-y-3">
                <li v-for="entry in audit?.items || []" :key="entry.id" class="flex gap-3">
                  <span class="mt-1.5 h-2 w-2 shrink-0 rounded-full bg-gray-400" aria-hidden="true" />
                  <div class="min-w-0">
                    <p class="text-sm font-medium">{{ transitionLabel(entry.action) }}</p>
                    <p class="text-xs text-gray-500">{{ relativeTime(entry.created_at) }}</p>
                    <p v-if="entry.payload.status" class="text-xs text-gray-500">status: {{ display(entry.payload.status) }}</p>
                  </div>
                </li>
                <li v-if="!audit?.items.length" class="text-sm text-gray-500">No history yet.</li>
              </ol>
            </div>
            <div class="flex justify-between gap-2 pt-2">
              <div class="flex gap-2">
                <UButton v-for="item in availableActions()" :key="item.action" :loading="transitioning" @click="transition(item.action)">{{ transitionLabel(item.action) }}</UButton>
                <UButton v-if="selected" color="error" variant="ghost" @click="deleteOpen = true">Delete</UButton>
              </div>
              <div class="ml-auto flex gap-2"><UButton variant="ghost" @click="panelOpen = false">Cancel</UButton><UButton type="submit" :loading="saving">Save</UButton></div>
            </div>
          </UForm>
        </template>
      </USlideover>

      <UModal v-model:open="deleteOpen" :title="`Delete ${entity?.label || 'record'}`">
        <template #body>
          <p class="text-sm text-muted">
            This will permanently delete this record. This action cannot be undone.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="deleteOpen = false">Cancel</UButton>
            <UButton color="error" :loading="deleting" @click="remove">Delete</UButton>
          </div>
        </template>
      </UModal>

      <UModal v-model:open="bulkDeleteOpen" :title="`Delete ${selectedRows.size} records`">
        <template #body>
          <p class="text-sm text-muted">
            This will permanently delete {{ selectedRows.size }} selected records. This action cannot be undone.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="bulkDeleteOpen = false">Cancel</UButton>
            <UButton color="error" :loading="bulkDeleting" @click="bulkDelete">Delete</UButton>
          </div>
        </template>
      </UModal>
    </template>
    </template>
  </UDashboardPanel>
</template>