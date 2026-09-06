<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

import { h, resolveComponent } from 'vue'
import type { TableColumn } from '@nuxt/ui'
import { absoluteTime, actionLabel, relativeTime } from '../../utils/audit-time'

const UButton = resolveComponent('UButton')
const UCheckbox = resolveComponent('UCheckbox')
const UBadge = resolveComponent('UBadge')

type FieldOption = { id: string; value: string; label: string }
type Field = { id: string; name: string; type: string; required: boolean; is_status: boolean; options: FieldOption[]; can_view?: boolean; can_edit?: boolean }
type EntityPermission = { role: string; can_view: boolean; can_edit: boolean }
type Entity = { id: string; name: string; label: string; fields: Field[]; permission?: EntityPermission }
type Document = { id: string; entity_id: string; payload: Record<string, unknown>; created_at: string; updated_at: string }
type DocumentList = { items: Document[]; total: number }
type AuditEntry = { id: string; action: string; payload: Record<string, unknown>; created_at: string; actor?: string | null }
type AuditList = { items: AuditEntry[]; total: number }
type WorkbookSheet = { entity_id: string; rows: { id: string; payload: Record<string, unknown> }[]; errors: string[] }
type WorkbookPreview = { sheets: WorkbookSheet[] }
type WorkbookResult = { sheets: { entity_id: string; created: number; updated: number }[] }

const route = useRoute()
const router = useRouter()
const entityId = computed(() => String(route.params.entity || ''))
const panelOpen = ref(false)
const selected = ref<Document | null>(null)
const payload = reactive<Record<string, unknown>>({})
const initialPayload = ref<Record<string, unknown>>({})
const discardOpen = ref(false)
const saving = ref(false)
const deleting = ref(false)
const transitioningAction = ref<string | null>(null)
const error = ref('')
const fieldErrors = reactive<Record<string, string>>({})
const exporting = ref(false)
const importing = ref(false)
const importPreview = ref<{ rows: { id: string; payload: Record<string, unknown> }[]; errors: string[] } | null>(null)
const importCsv = ref('')
const importInput = ref<HTMLInputElement | null>(null)
const workbookPreview = ref<WorkbookPreview | null>(null)
const workbookBytes = ref<ArrayBuffer | null>(null)
const toast = useToast()
const deleteOpen = ref(false)
const conflictOpen = ref(false)
const selectedUpdatedAt = ref('')

// --- Breadcrumb ---
const breadcrumbItems = computed(() => [
  { label: 'Entities', to: '/admin/meta/entity' },
  { label: entity.value?.label || entityId.value }
])

// --- Keyboard shortcuts ---
function onKeydown(event: KeyboardEvent) {
  const target = event.target as HTMLElement | null
  const isTyping = !!target && (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable)
  if (event.key.toLowerCase() === 'n' && !isTyping && !panelOpen.value) {
    event.preventDefault()
    openCreate()
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  window.addEventListener('focus', refetchOnFocus)
  document.addEventListener('visibilitychange', onVisibilityChange)
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  window.removeEventListener('focus', refetchOnFocus)
  document.removeEventListener('visibilitychange', onVisibilityChange)
})

// --- Dirty-state protection ---
const dirty = computed(() => JSON.stringify(payload) !== JSON.stringify(initialPayload.value))

function requestClose() {
  if (dirty.value) {
    discardOpen.value = true
  } else {
    panelOpen.value = false
  }
}

function handlePanelOpenChange(value: boolean) {
  if (!value && dirty.value) {
    discardOpen.value = true
    return
  }
  panelOpen.value = value
}

function discardChanges() {
  discardOpen.value = false
  panelOpen.value = false
}

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

type EntityView = { id: string; name: string; config: Record<string, unknown> }

const { data: entities } = await useFetch<{ id: string; label: string }[]>('/api/entities')
const { data: entity, status: entityStatus, error: entityError } = await useFetch<Entity>(
  () => `/api/entities/${encodeURIComponent(entityId.value)}`,
  { watch: [entityId] }
)
const canEdit = computed(() => entity.value?.permission?.can_edit ?? true)
const viewableFields = computed(() => (entity.value?.fields || []).filter(f => f.can_view ?? true))
const editableFields = computed(() => viewableFields.value.filter(f => f.can_edit ?? true))
const formFields = computed(() => viewableFields.value.filter(f => !f.is_status))
function isFieldEditable(name: string) {
  return editableFields.value.some(f => f.name === name)
}
const activeViewId = computed(() => {
  const view = route.query.view
  return typeof view === 'string' && view.trim() ? view : ''
})
const { data: activeView } = await useFetch<EntityView>(
  () => activeViewId.value ? `/api/views/${encodeURIComponent(activeViewId.value)}` : '',
  { watch: [activeViewId], immediate: false }
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
  if (activeViewId.value) params.set('view_id', activeViewId.value)
  return `/api/documents?${params.toString()}`
})

function clearView() {
  router.push({ path: route.path, query: {} })
}
const { data: documents, status: documentsStatus, error: documentsError, refresh } = await useFetch<DocumentList>(
  documentsUrl,
  { watch: [documentsUrl] }
)
const { data: workflow, status: workflowStatus, error: workflowError, refresh: refreshWorkflow } = await useFetch<{ states: { id: string; name: string; label: string }[]; transitions: { id: string; action: string; from_state: string; to_state: string }[] }>(
  () => `/api/entities/${encodeURIComponent(entityId.value)}/workflow`,
  { watch: [entityId] }
)
const auditId = computed(() => selected.value?.id || '')
const { data: audit, status: auditStatus, refresh: refreshAudit } = await useFetch<AuditList>(
  () => `/api/documents/${encodeURIComponent(auditId.value)}/audit`,
  { watch: [auditId], immediate: false }
)

function emptyPayload() {
  const fields = viewableFields.value
  const statusField = fields.find(f => f.is_status)
  const defaultStatus = workflow.value?.states[0]?.name || statusField?.options[0]?.value || ''
  return defaultPayload(fields, defaultStatus)
}

function openCreate() {
  selected.value = null
  Object.assign(payload, emptyPayload())
  initialPayload.value = { ...payload }
  Object.keys(fieldErrors).forEach(key => delete fieldErrors[key])
  error.value = ''
  panelOpen.value = true
}

function openEdit(document: Document) {
  selected.value = document
  selectedUpdatedAt.value = document.updated_at
  Object.keys(payload).forEach(key => delete payload[key])
  Object.assign(payload, emptyPayload(), document.payload)
  initialPayload.value = { ...payload }
  Object.keys(fieldErrors).forEach(key => delete fieldErrors[key])
  error.value = ''
  panelOpen.value = true
  refreshAudit()
}

function isConflict(cause: any) {
  return cause?.statusCode === 409 || cause?.data?.code === 'conflict'
}

async function reloadLatest() {
  if (!selected.value) return
  try {
    const fresh = await $fetch<Document>(`/api/documents/${encodeURIComponent(selected.value.id)}`)
    selected.value = fresh
    selectedUpdatedAt.value = fresh.updated_at
    Object.keys(payload).forEach(key => delete payload[key])
    Object.assign(payload, emptyPayload(), fresh.payload)
    initialPayload.value = { ...payload }
    conflictOpen.value = false
    await refresh()
    await refreshAudit()
    toast.add({ title: 'Reloaded latest version', color: 'info', icon: 'i-lucide-refresh-cw' })
  } catch (cause: any) {
    toast.add({ title: 'Unable to reload', description: cause?.data?.message || cause?.statusMessage || 'Reload failed', color: 'error', icon: 'i-lucide-alert-circle' })
  }
}

function validate() {
  Object.keys(fieldErrors).forEach(key => delete fieldErrors[key])
  Object.assign(fieldErrors, validatePayload(viewableFields.value, payload))
  return Object.keys(fieldErrors).length === 0
}

function normalizedPayload() {
  return normalizePayload(editableFields.value, payload)
}

async function save() {
  if (!validate() || !entity.value) return
  saving.value = true
  error.value = ''
  const isEdit = !!selected.value
  try {
    const nextPayload = normalizedPayload()
    if (isEdit && selected.value) {
      const updated = await $fetch<Document>(`/api/documents/${encodeURIComponent(selected.value.id)}`, {
        method: 'PUT',
        body: { payload: nextPayload, expected_updated_at: selectedUpdatedAt.value || undefined }
      })
      selected.value = updated
      selectedUpdatedAt.value = updated.updated_at
    } else {
      await $fetch('/api/documents', {
        method: 'POST',
        body: { id: crypto.randomUUID(), entity_id: entity.value.id, payload: nextPayload }
      })
    }
    initialPayload.value = { ...nextPayload }
    panelOpen.value = false
    await refresh()
    toast.add({ title: isEdit ? 'Record updated' : 'Record created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    if (isConflict(cause)) {
      conflictOpen.value = true
      return
    }
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to save record'
    toast.add({ title: 'Unable to save record', description: error.value, color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    saving.value = false
  }
}

async function transition(action: string) {
  if (!selected.value) return
  transitioningAction.value = action
  error.value = ''
  try {
    const updated = await $fetch<Document>(`/api/documents/${encodeURIComponent(selected.value.id)}/transition`, { method: 'POST', body: { action, expected_updated_at: selectedUpdatedAt.value || undefined } })
    selected.value = updated
    selectedUpdatedAt.value = updated.updated_at
    await refresh()
    await refreshAudit()
    toast.add({ title: 'Record transitioned', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    if (isConflict(cause)) {
      conflictOpen.value = true
      return
    }
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to transition record'
    toast.add({ title: 'Unable to transition record', description: error.value, color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    transitioningAction.value = null
  }
}

function availableActions() {
  const status = selected.value?.payload[statusField.value?.name || '']
  return workflow.value?.transitions.filter(item => item.from_state === status) || []
}

async function remove() {
  if (!selected.value) return
  deleting.value = true
  error.value = ''
  try {
    await $fetch(`/api/documents/${encodeURIComponent(selected.value.id)}`, { method: 'DELETE' })
    initialPayload.value = { ...payload }
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

function transitionLabel(action: string) {
  return actionLabel(action)
}

function snapshotKey() {
  return (documents.value?.items || []).map(d => `${d.id}@${d.updated_at}`).join(',')
}

async function refetchOnFocus() {
  const before = snapshotKey()
  await refresh()
  if (before && before !== snapshotKey()) {
    toast.add({ title: 'Records updated by another user', color: 'info', icon: 'i-lucide-refresh-cw' })
  }
}

function onVisibilityChange() {
  if (!document.hidden) void refetchOnFocus()
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
  for (const field of viewableFields.value) {
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
const statusField = computed(() => entity.value?.fields.find(f => f.is_status))
const statusItems = computed(() => [
  { label: 'All statuses', value: 'all' },
  ...(statusField.value?.options || []).map(o => ({ label: o.label, value: o.value }))
])

// --- Audit log helpers ---
function statusLabel(value: unknown) {
  if (value === undefined || value === null || value === '') return null
  const option = statusField.value?.options.find(o => o.value === value)
  return option?.label || String(value)
}

const auditItems = computed(() => {
  const items = audit.value?.items || []
  const statusName = statusField.value?.name
  return items.map((entry, index) => {
    const next = items[index + 1]
    const to = statusName ? entry.payload[statusName] : undefined
    const from = statusName ? next?.payload[statusName] : undefined
    return {
      ...entry,
      fromLabel: statusLabel(from),
      toLabel: statusLabel(to)
    }
  })
})

// --- Column visibility ---
watch(entity, (e) => {
  if (e) {
    visibleColumns.value = new Set(e.fields.filter(f => f.can_view ?? true).map(f => f.name))
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
  for (const field of viewableFields.value.filter(f => visibleColumns.value.has(f.name))) {
    cols.push({
      accessorKey: `payload.${field.name}`,
      header: field.name,
      enableSorting: true,
      cell: ({ row }) => {
        const value = row.original.payload[field.name]
        if (field.is_status) {
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
    const csv = await $fetch<string>(`/api/entities/${encodeURIComponent(entityId.value)}/export`)
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
    if (file.name.toLowerCase().endsWith('.xlsx')) {
      workbookBytes.value = await file.arrayBuffer()
      workbookPreview.value = await $fetch<WorkbookPreview>('/api/entities/import-preview', {
        method: 'POST',
        body: new Uint8Array(workbookBytes.value),
        headers: { 'content-type': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' }
      })
      importPreview.value = null
      importCsv.value = ''
    } else {
      importCsv.value = await file.text()
      importPreview.value = await $fetch<{ rows: { id: string; payload: Record<string, unknown> }[]; errors: string[] }>(
        `/api/entities/${encodeURIComponent(entityId.value)}/import-preview`,
        { method: 'POST', body: importCsv.value, headers: { 'content-type': 'text/csv' } }
      )
      workbookPreview.value = null
      workbookBytes.value = null
    }
  } catch (cause: any) {
    importPreview.value = null
    workbookPreview.value = null
    workbookBytes.value = null
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to preview import'
  } finally {
    input.value = ''
    importing.value = false
  }
}

const workbookErrors = computed(() => (workbookPreview.value?.sheets || []).flatMap(sheet => sheet.errors))
const workbookRowCount = computed(() => (workbookPreview.value?.sheets || []).reduce((total, sheet) => total + sheet.rows.length, 0))

async function exportWorkbook() {
  exporting.value = true
  try {
    const bytes = await $fetch<Blob>('/api/entities/export', { responseType: 'blob' })
    const url = URL.createObjectURL(new Blob([bytes], { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' }))
    const link = document.createElement('a')
    link.href = url
    link.download = 'logholizon.xlsx'
    link.click()
    URL.revokeObjectURL(url)
    toast.add({ title: 'Export complete', description: 'logholizon.xlsx downloaded', color: 'success', icon: 'i-lucide-download' })
  } catch (cause: any) {
    toast.add({ title: 'Unable to export', description: cause?.data?.message || cause?.statusMessage || 'Export failed', color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    exporting.value = false
  }
}

async function confirmWorkbookImport() {
  if (!workbookBytes.value || workbookErrors.value.length) return
  importing.value = true
  error.value = ''
  try {
    const result = await $fetch<WorkbookResult>('/api/entities/import-confirm', {
      method: 'POST',
      body: new Uint8Array(workbookBytes.value),
      headers: { 'content-type': 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' }
    })
    workbookPreview.value = null
    workbookBytes.value = null
    await refresh()
    const summary = result.sheets.map(sheet => `${sheet.entity_id}: ${sheet.created} created, ${sheet.updated} updated`).join('; ')
    toast.add({ title: 'Import complete', description: summary, color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to import workbook'
    toast.add({ title: 'Unable to import workbook', description: error.value, color: 'error', icon: 'i-lucide-alert-circle' })
  } finally {
    importing.value = false
  }
}

async function confirmImport() {
  if (!importCsv.value || importPreview.value?.errors.length) return
  importing.value = true
  error.value = ''
  try {
    const result = await $fetch<{ created: number; updated: number }>(`/api/entities/${encodeURIComponent(entityId.value)}/import-confirm`, { method: 'POST', body: importCsv.value, headers: { 'content-type': 'text/csv' } })
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
      <UDashboardNavbar>
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #title>
          <UBreadcrumb :items="breadcrumbItems" />
        </template>
        <template #right>
          <UButton icon="i-lucide-plus" :disabled="!canEdit" @click="openCreate">New record</UButton>
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
                <label v-for="field in viewableFields" :key="field.id" class="flex cursor-pointer items-center gap-2 rounded px-2 py-1 text-sm hover:bg-default">
                  <UCheckbox :model-value="visibleColumns.has(field.name)" @update:model-value="toggleColumn(field.name)" />
                  <span class="font-mono">{{ field.name }}</span>
                </label>
              </div>
            </template>
          </UPopover>
          <UButton variant="outline" icon="i-lucide-download" :loading="exporting" @click="exportCsv">Export CSV</UButton>
          <UButton variant="outline" icon="i-lucide-file-spreadsheet" :loading="exporting" @click="exportWorkbook">Export Excel</UButton>
          <input ref="importInput" type="file" accept=".csv,.xlsx,text/csv,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" class="hidden" @change="previewImport">
          <UButton variant="outline" icon="i-lucide-upload" :loading="importing" :disabled="!canEdit" @click="importInput?.click()">Import CSV/Excel</UButton>
          <UBadge v-if="!canEdit" color="neutral" variant="subtle">Read-only</UBadge>
      </div>
      <div v-if="activeViewId" class="mb-3 flex items-center gap-2">
        <UBadge color="primary" variant="subtle" icon="i-lucide-eye">View: {{ activeView?.name || activeViewId }}</UBadge>
        <UButton size="xs" variant="ghost" @click="clearView">Clear</UButton>
      </div>
        <UAlert v-if="importPreview" class="w-full" :color="importPreview.errors.length ? 'error' : 'success'" :title="`${importPreview.rows.length} rows previewed`">
          <template #description>
            <ul v-if="importPreview.errors.length" class="list-disc space-y-1 pl-4">
              <li v-for="(message, index) in importPreview.errors" :key="index" class="text-sm">{{ message }}</li>
            </ul>
            <p v-else class="text-sm">Ready to import.</p>
          </template>
          <template #actions><UButton :disabled="!!importPreview.errors.length" :loading="importing" size="sm" @click="confirmImport">Confirm import</UButton></template>
        </UAlert>
        <UAlert v-if="workbookPreview" class="w-full" :color="workbookErrors.length ? 'error' : 'success'" :title="`${workbookRowCount} rows previewed across ${workbookPreview.sheets.length} sheets`">
          <template #description>
            <ul v-if="workbookErrors.length" class="list-disc space-y-1 pl-4">
              <li v-for="(message, index) in workbookErrors" :key="index" class="text-sm">{{ message }}</li>
            </ul>
            <ul v-else class="list-disc space-y-1 pl-4">
              <li v-for="sheet in workbookPreview.sheets" :key="sheet.entity_id" class="font-mono text-sm">{{ sheet.entity_id }}: {{ sheet.rows.length }} rows</li>
            </ul>
          </template>
          <template #actions><UButton :disabled="!!workbookErrors.length" :loading="importing" size="sm" @click="confirmWorkbookImport">Confirm import</UButton></template>
        </UAlert>

      <UAlert
        v-if="!entity.fields.length"
        color="warning"
        title="This entity has no fields"
        description="Add fields in Entity Manager before creating records."
        class="mb-4"
      />

      <div v-if="!entity.fields.length" class="flex flex-col items-center gap-3 py-16 text-center">
        <UIcon name="i-lucide-table-properties" class="h-10 w-10 text-muted" />
        <p class="text-sm text-muted">This entity has no fields yet.</p>
        <UButton icon="i-lucide-settings-2" :to="'/admin/meta/entity'">Add fields in Entity Manager</UButton>
      </div>

      <UCard v-else>
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

      <USlideover :open="panelOpen" :title="selected ? `Edit ${entity.label}` : `New ${entity.label}`" @update:open="handlePanelOpenChange">
        <template #body>
          <UForm id="record-form" data-testid="record-form" class="space-y-4" @submit="save">
            <UFormField
              v-for="field in formFields"
              :key="field.id"
              :label="field.name"
              :required="field.required"
              :error="fieldErrors[field.name]"
              :hint="isFieldEditable(field.name) ? undefined : 'View only'"
            >
              <USelectMenu
                v-if="field.type === 'select' && !field.is_status"
                v-model="payload[field.name] as string"
                :items="field.options.map(o => ({ label: o.label, value: o.value }))"
                value-key="value"
                placeholder="Select…"
                class="w-full"
                :disabled="!isFieldEditable(field.name)"
              />
              <UInput v-else v-model="payload[field.name] as string" :disabled="!isFieldEditable(field.name)" :type="field.type === 'date' ? 'date' : field.type === 'number' ? 'number' : 'text'" />
            </UFormField>
            <UAlert v-if="error" color="error" :title="error" />
            <div v-if="selected" class="border-t pt-4">
              <h2 class="mb-2 text-sm font-semibold">History</h2>
              <div v-if="auditStatus === 'pending'" class="py-4 text-sm text-muted">Loading history…</div>
              <UAlert v-else-if="auditStatus === 'error'" color="error" title="Cannot load history" />
              <ol v-else class="space-y-3">
                <li v-for="entry in auditItems" :key="entry.id" class="flex gap-3">
                  <span class="mt-1.5 h-2 w-2 shrink-0 rounded-full bg-default" aria-hidden="true" />
                  <div class="min-w-0">
                    <p class="text-sm font-medium">{{ transitionLabel(entry.action) }}</p>
                    <UTooltip :text="absoluteTime(entry.created_at)">
                      <p class="text-xs text-muted">by {{ entry.actor || 'system' }} · {{ relativeTime(entry.created_at) }}</p>
                    </UTooltip>
                    <p v-if="entry.fromLabel || entry.toLabel" class="text-xs text-muted">
                      {{ entry.fromLabel || '—' }} → {{ entry.toLabel || '—' }}
                    </p>
                  </div>
                </li>
                <li v-if="!auditItems.length" class="text-sm text-muted">No history yet.</li>
              </ol>
            </div>
          </UForm>
        </template>
        <template #footer>
          <div class="space-y-3">
            <UAlert
              v-if="workflowStatus === 'error'"
              color="error"
              title="Cannot load workflow"
              :description="workflowError?.message || 'Check the Rust core connection.'"
            >
              <template #actions>
                <UButton size="sm" variant="outline" @click="refreshWorkflow()">Retry</UButton>
              </template>
            </UAlert>
            <div class="flex justify-between gap-2">
              <div class="flex gap-2">
                <UButton
                  v-for="item in availableActions()"
                  :key="item.action"
                  :loading="transitioningAction === item.action"
                  :disabled="!canEdit || (transitioningAction !== null && transitioningAction !== item.action)"
                  @click="transition(item.action)"
                >{{ transitionLabel(item.action) }}</UButton>
                <p v-if="selected && !availableActions().length && workflowStatus === 'success'" class="self-center text-xs text-muted">No actions available for this status.</p>
                <UButton v-if="selected" color="error" variant="ghost" :disabled="!canEdit" @click="deleteOpen = true">Delete</UButton>
              </div>
              <div class="ml-auto flex gap-2"><UButton variant="ghost" @click="requestClose">Cancel</UButton><UButton type="submit" form="record-form" :loading="saving" :disabled="!canEdit">Save</UButton></div>
            </div>
          </div>
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

      <UModal v-model:open="conflictOpen" title="Record changed by another user">
        <template #body>
          <p class="text-sm text-muted">
            Someone else saved this record while you were editing. Reload to see their changes, or discard your edits.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="conflictOpen = false; discardChanges()">Discard my edits</UButton>
            <UButton @click="reloadLatest">Reload latest</UButton>
          </div>
        </template>
      </UModal>

      <UModal v-model:open="discardOpen" title="Discard changes?">
        <template #body>
          <p class="text-sm text-muted">
            You have unsaved changes in this record. Discard them and close the panel?
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="discardOpen = false">Keep editing</UButton>
            <UButton color="error" @click="discardChanges">Discard</UButton>
          </div>
        </template>
      </UModal>
    </template>
    </template>
  </UDashboardPanel>
</template>