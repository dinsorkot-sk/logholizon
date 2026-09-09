<script setup lang="ts">
definePageMeta({ middleware: 'auth' })
type Field = { id: string; name: string; label?: string; description?: string; type: string; required?: boolean; readonly?: boolean; hidden?: boolean; sortable?: boolean; can_edit?: boolean; options?: { value: string; label: string }[]; ref_entity?: string | null; is_status?: boolean }
type Entity = { id: string; name: string; label: string; module?: string | null; fields: Field[]; permission?: { can_view: boolean; can_edit: boolean } }
type Row = { id: string; payload: Record<string, unknown>; created_at: string; updated_at: string }
type List = { items: Row[]; total: number }
type Layout = { config?: { sections?: { id: string; label: string; fields: string[] }[] } }
const route = useRoute()
const moduleName = computed(() => String(route.params.module || ''))
const entityName = computed(() => String(route.params.entity || ''))
const { data: entities } = await useFetch<Entity[]>('/api/entities')
const entity = computed(() => (entities.value || []).find(e => e.name === entityName.value && String(e.module || '') === moduleName.value))
const entityId = computed(() => entity.value?.id || '')
const canEdit = computed(() => entity.value?.permission?.can_edit ?? true)
const apiBase = computed(() => `/api/modules/${encodeURIComponent(moduleName.value)}/entities/${encodeURIComponent(entityName.value)}`)
const { data: layout } = await useFetch<Layout>(() => entityId.value ? `/api/entities/${encodeURIComponent(entityId.value)}/form-layout` : '', { watch: [entityId], immediate: false })
const fields = computed(() => entity.value?.fields || [])
const referenceOptions = ref<Record<string, { id: string; label: string }[]>>({})
watch(fields, async value => { for (const f of value.filter(f => f.type === 'reference' && f.ref_entity)) { const id = f.ref_entity!; if (!referenceOptions.value[id]) { try { referenceOptions.value[id] = await $fetch(`/api/entities/${encodeURIComponent(id)}/options?limit=50`) } catch { referenceOptions.value[id] = [] } } } }, { immediate: true })
const search = ref('')
const limit = ref(50)
const offset = ref(0)
const sortBy = ref('')
const sortDir = ref('desc')
const listUrl = computed(() => `${apiBase.value}?limit=${limit.value}&offset=${offset.value}${search.value ? `&search=${encodeURIComponent(search.value)}` : ''}${sortBy.value ? `&sort_by=${encodeURIComponent(sortBy.value)}&sort_dir=${sortDir.value}` : ''}`)
const { data: records, status, error, refresh } = await useFetch<List>(listUrl, { watch: [listUrl] })
const selected = ref<Row | null>(null)
const formOpen = ref(false)
const form = ref<Record<string, unknown>>({})
const saving = ref(false)
const formError = ref('')
const selectedRows = ref(new Set<string>())
const sections = computed(() => layout.value?.config?.sections || null)
function blank() { return Object.fromEntries(fields.value.filter(f => !f.hidden).map(f => [f.name, f.type === 'boolean' ? false : ''])) }
function openCreate() { selected.value = null; form.value = blank(); formError.value = ''; formOpen.value = true }
function openEdit(row: { id: string; payload: Record<string, unknown>; created_at?: string; updated_at?: string }) { selected.value = row as Row; form.value = { ...blank(), ...row.payload }; formError.value = ''; formOpen.value = true }
function toggle(id: string) { const next = new Set(selectedRows.value); next.has(id) ? next.delete(id) : next.add(id); selectedRows.value = next }
function sort(field: string) { if (sortBy.value === field) sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'; else { sortBy.value = field; sortDir.value = 'asc' }; offset.value = 0 }
async function save() { saving.value = true; formError.value = ''; try { if (selected.value) await $fetch(`${apiBase.value}/${encodeURIComponent(selected.value.id)}`, { method: 'PUT', body: { payload: form.value, expected_updated_at: selected.value.updated_at } }); else await $fetch(apiBase.value, { method: 'POST', body: { payload: form.value } }); formOpen.value = false; await refresh() } catch (e: any) { formError.value = e?.data?.message || e?.statusMessage || 'Unable to save record' } finally { saving.value = false } }
const start = computed(() => records.value?.total ? offset.value + 1 : 0)
const end = computed(() => Math.min(offset.value + limit.value, records.value?.total || 0))
</script>

<template>
  <UDashboardPanel :id="`dynamic-${moduleName}-${entityName}`">
    <template #header><UDashboardNavbar :title="entity?.label || entityName"><template #leading><UDashboardSidebarCollapse /></template><template #right><UButton icon="i-lucide-plus" :disabled="!canEdit" @click="openCreate">New record</UButton></template></UDashboardNavbar></template>
    <template #body>
      <UAlert v-if="!entity" color="error" title="Entity not found" description="The module or entity is not available to your account." />
      <template v-else>
        <div class="mb-4 flex flex-wrap gap-2"><UInput v-model="search" icon="i-lucide-search" placeholder="Search records…" class="w-64" /><UButton variant="outline" icon="i-lucide-refresh-cw" :loading="status === 'pending'" @click="() => refresh()">Refresh</UButton><UBadge v-if="!canEdit" variant="subtle">Read-only</UBadge></div>
        <UAlert v-if="error" color="error" title="Cannot load records" :description="error.message" class="mb-4" />
        <DynamicList v-else :fields="fields" :rows="records?.items || []" :total="records?.total || 0" :selected="selectedRows" @select="toggle" @open="openEdit" @sort="sort" />
        <div v-if="records?.total" class="mt-3 flex items-center justify-between text-sm text-muted"><span>Showing {{ start }}–{{ end }} of {{ records.total }}</span><div class="flex gap-2"><UButton size="sm" variant="outline" :disabled="offset === 0" @click="offset = Math.max(0, offset - limit)">Prev</UButton><UButton size="sm" variant="outline" :disabled="end >= records.total" @click="offset += limit">Next</UButton></div></div>
      </template>
    </template>
  </UDashboardPanel>
  <USlideover v-model:open="formOpen" :title="selected ? `Edit ${entity?.label}` : `New ${entity?.label}`"><template #body><DynamicForm :fields="fields" :model-value="form" :sections="sections" :reference-options="referenceOptions" :disabled="!canEdit" @update:model-value="form = $event" /><UAlert v-if="formError" class="mt-4" color="error" :title="formError" /></template><template #footer><div class="flex justify-end gap-2"><UButton variant="ghost" @click="formOpen = false">Cancel</UButton><UButton :loading="saving" :disabled="!canEdit" @click="save">Save</UButton></div></template></USlideover>
</template>
