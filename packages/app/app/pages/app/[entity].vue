<script setup lang="ts">
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

const { data: entities } = await useFetch<{ id: string; label: string }[]>('/api/meta/entities')
const { data: entity, status: entityStatus, error: entityError } = await useFetch<Entity>(
  () => `/api/meta/entities/${encodeURIComponent(entityId.value)}`,
  { watch: [entityId] }
)
const { data: documents, status: documentsStatus, refresh } = await useFetch<DocumentList>(
  () => `/api/documents?entity_id=${encodeURIComponent(entityId.value)}`,
  { watch: [entityId] }
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
  try {
    const nextPayload = normalizedPayload()
    if (selected.value) {
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
  } catch (cause: any) {
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to save record'
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
  } catch (cause: any) {
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to transition record'
  } finally {
    transitioning.value = false
  }
}

function availableActions() {
  const status = selected.value?.payload.status
  return workflow.value?.transitions.filter(item => item.from_state === status) || []
}

async function remove() {
  if (!selected.value || !confirm('Delete this record permanently?')) return
  deleting.value = true
  error.value = ''
  try {
    await $fetch(`/api/documents/${encodeURIComponent(selected.value.id)}`, { method: 'DELETE' })
    panelOpen.value = false
    await refresh()
  } catch (cause: any) {
    error.value = cause?.data?.message || cause?.statusMessage || 'Unable to delete record'
  } finally {
    deleting.value = false
  }
}

function display(value: unknown) {
  if (value === '' || value === null || value === undefined) return '—'
  return String(value)
}

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
  } finally {
    exporting.value = false
  }
}

async function previewImport(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return
  importing.value = true
  try {
    importCsv.value = await file.text()
    importPreview.value = await $fetch<{ rows: { id: string; payload: Record<string, unknown> }[]; errors: string[] }>(
      `/api/meta/entities/${encodeURIComponent(entityId.value)}/import-preview` as '/api/meta/entities/[id]/import-preview',
      { method: 'POST', body: await file.text(), headers: { 'content-type': 'text/csv' } }
    )
  } finally { importing.value = false }
}

async function confirmImport() {
  if (!importCsv.value || importPreview.value?.errors.length) return
  importing.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(entityId.value)}/import-confirm`, { method: 'POST', body: importCsv.value, headers: { 'content-type': 'text/csv' } })
    importPreview.value = null; importCsv.value = ''; await refresh()
  } finally { importing.value = false }
}
</script>

<template>
  <UContainer class="py-8">
    <UAlert
      v-if="entityStatus === 'error'"
      color="error"
      title="Cannot load entity"
      :description="entityError?.message || 'Check the entity ID and Rust core connection.'"
    />

    <template v-else-if="entity">
      <div class="mb-6 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 class="text-xl font-semibold">{{ entity.label }}</h1>
          <p class="font-mono text-sm text-gray-500">{{ entity.name }}</p>
        </div>
        <div class="flex gap-2">
          <select
            aria-label="Select entity"
            class="rounded border px-3 py-2 text-sm"
            :value="entityId"
            @change="router.push(`/app/${encodeURIComponent(($event.target as HTMLSelectElement).value)}`)"
          >
            <option v-for="item in entities || []" :key="item.id" :value="item.id">{{ item.label }}</option>
          </select>
          <UButton variant="outline" icon="i-lucide-download" :loading="exporting" @click="exportCsv">Export CSV</UButton>
          <input ref="importInput" type="file" accept=".csv,text/csv" class="hidden" @change="previewImport">
          <UButton variant="outline" icon="i-lucide-upload" :loading="importing" @click="importInput?.click()">Import CSV</UButton>
          <UButton icon="i-lucide-plus" @click="openCreate">New record</UButton>
        </div>
        <UAlert v-if="importPreview" class="w-full" :color="importPreview.errors.length ? 'error' : 'success'" :title="`${importPreview.rows.length} rows previewed`" :description="importPreview.errors.join('; ') || 'Ready to import.'">
          <template #actions><UButton :disabled="!!importPreview.errors.length" :loading="importing" size="sm" @click="confirmImport">Confirm import</UButton></template>
        </UAlert>
      </div>

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
        <template v-else>
          <table class="w-full text-sm">
            <caption class="sr-only">{{ entity.label }} records</caption>
            <thead>
              <tr class="border-b text-left text-gray-500">
                <th v-for="field in entity.fields" :key="field.id" scope="col" class="px-3 py-2">{{ field.name }}</th>
                <th scope="col" class="px-3 py-2"><span class="sr-only">Actions</span></th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="document in documents?.items || []" :key="document.id" class="border-b last:border-0 hover:bg-gray-50">
                <td v-for="field in entity.fields" :key="field.id" class="px-3 py-2">
                  <UBadge v-if="field.name === 'status'" variant="subtle">{{ display(document.payload[field.name]) }}</UBadge>
                  <template v-else>{{ display(document.payload[field.name]) }}</template>
                </td>
                <td class="px-3 py-2 text-right"><UButton size="xs" variant="ghost" @click="openEdit(document)">Edit</UButton></td>
              </tr>
              <tr v-if="!documents?.items.length">
                <td :colspan="entity.fields.length + 1" class="py-10 text-center text-gray-500">No records yet.</td>
              </tr>
            </tbody>
          </table>
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
              <select v-if="field.type === 'select' && field.name !== 'status'" v-model="payload[field.name] as string" class="w-full rounded border px-3 py-2">
                <option value="">Select…</option>
                <option v-for="option in field.options" :key="option.id" :value="option.value">{{ option.label }}</option>
              </select>
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
                    <p class="text-sm font-medium">{{ entry.action }}</p>
                    <p class="text-xs text-gray-500">{{ entry.created_at }}</p>
                    <p v-if="entry.payload.status" class="text-xs text-gray-500">status: {{ display(entry.payload.status) }}</p>
                  </div>
                </li>
                <li v-if="!audit?.items.length" class="text-sm text-gray-500">No history yet.</li>
              </ol>
            </div>
            <div class="flex justify-between gap-2 pt-2">
              <div class="flex gap-2">
                <UButton v-for="item in availableActions()" :key="item.action" :loading="transitioning" @click="transition(item.action)">{{ item.action }}</UButton>
                <UButton v-if="selected" color="error" variant="ghost" :loading="deleting" @click="remove">Delete</UButton>
              </div>
              <div class="ml-auto flex gap-2"><UButton variant="ghost" @click="panelOpen = false">Cancel</UButton><UButton type="submit" :loading="saving">Save</UButton></div>
            </div>
          </UForm>
        </template>
      </USlideover>
    </template>
  </UContainer>
</template>