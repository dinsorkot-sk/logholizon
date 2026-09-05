<script setup lang="ts">
import { h, resolveComponent } from 'vue'
import type { TableColumn } from '@nuxt/ui'

const UButton = resolveComponent('UButton')
const UBadge = resolveComponent('UBadge')

type Entity = { id: string; name: string; label: string }
type FieldOption = { id: string; value: string; label: string }
type Field = { id: string; name: string; type: string; required: boolean; position: number; options: FieldOption[] }
type EntityDetail = Entity & { fields: Field[] }

const toast = useToast()
const { data: entities, status, refresh } = await useFetch<Entity[]>('/api/meta/entities')

const search = ref('')
const selectedId = ref('')
const detailUrl = computed(() => selectedId.value ? `/api/meta/entities/${encodeURIComponent(selectedId.value)}` : '')
const { data: detail, status: detailStatus, error: detailError, refresh: refreshDetail } = await useFetch<EntityDetail>(detailUrl, { immediate: false })

const filteredEntities = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return entities.value || []
  return (entities.value || []).filter(e =>
    e.id.toLowerCase().includes(q) || e.label.toLowerCase().includes(q)
  )
})

const tabItems = [
  { label: 'Fields', icon: 'i-lucide-table', slot: 'fields' },
  { label: 'Permissions', icon: 'i-lucide-shield', slot: 'permissions' },
  { label: 'Views', icon: 'i-lucide-eye', slot: 'views' }
]

const typeItems = [
  { label: 'Text', value: 'text', icon: 'i-lucide-type' },
  { label: 'Number', value: 'number', icon: 'i-lucide-hash' },
  { label: 'Date', value: 'date', icon: 'i-lucide-calendar' },
  { label: 'Select', value: 'select', icon: 'i-lucide-chevrons-up-down' }
]

function typeBadgeColor(type: string) {
  switch (type) {
    case 'text': return 'info'
    case 'select': return 'primary'
    case 'number': return 'warning'
    default: return 'neutral'
  }
}

function selectEntity(id: string) {
  selectedId.value = id
  refreshDetail()
}

// --- Create entity ---
const createOpen = ref(false)
const createForm = reactive({ id: '', name: '', label: '' })
const creating = ref(false)
const createError = ref('')

async function createEntity() {
  createError.value = ''
  if (!createForm.id.trim() || !createForm.name.trim() || !createForm.label.trim()) {
    createError.value = 'id, name, and label are required'
    return
  }
  if (!/^[a-z][a-z0-9_]*$/.test(createForm.id)) {
    createError.value = 'id must be lowercase snake_case (e.g. work_order)'
    return
  }
  creating.value = true
  try {
    await $fetch('/api/meta/entities', { method: 'POST', body: { ...createForm } })
    createOpen.value = false
    createForm.id = ''
    createForm.name = ''
    createForm.label = ''
    await refresh()
    toast.add({ title: 'Entity created', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    createError.value = e?.data?.message || e?.statusMessage || 'Failed to create entity'
  } finally {
    creating.value = false
  }
}

// --- Edit entity ---
const editOpen = ref(false)
const editForm = reactive({ name: '', label: '' })
const editing = ref(false)
const editError = ref('')

function openEditEntity() {
  if (!detail.value) return
  editForm.name = detail.value.name
  editForm.label = detail.value.label
  editError.value = ''
  editOpen.value = true
}

async function saveEntity() {
  if (!detail.value) return
  editError.value = ''
  if (!editForm.name.trim() || !editForm.label.trim()) {
    editError.value = 'name and label are required'
    return
  }
  editing.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(detail.value.id)}`, {
      method: 'PUT',
      body: { ...editForm }
    })
    editOpen.value = false
    await refresh()
    await refreshDetail()
    toast.add({ title: 'Entity updated', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    editError.value = e?.data?.message || e?.statusMessage || 'Failed to update entity'
  } finally {
    editing.value = false
  }
}

// --- Delete entity ---
const deleteEntityOpen = ref(false)
const deletingEntity = ref(false)

async function removeEntity() {
  if (!detail.value) return
  deletingEntity.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(detail.value.id)}`, { method: 'DELETE' })
    deleteEntityOpen.value = false
    selectedId.value = ''
    await refresh()
    toast.add({ title: 'Entity deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to delete entity',
      description: e?.data?.message || e?.statusMessage || 'Delete failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    deletingEntity.value = false
  }
}

// --- Field editor ---
const fieldOpen = ref(false)
const editingField = ref<Field | null>(null)
const fieldForm = reactive({ name: '', type: 'text', required: false })
const fieldError = ref('')
const savingField = ref(false)
const newOption = reactive({ value: '', label: '' })
const optionError = ref('')

const editingFieldOptions = computed(() => {
  if (!editingField.value) return []
  return detail.value?.fields.find(f => f.id === editingField.value?.id)?.options || []
})

function openAddField() {
  editingField.value = null
  fieldForm.name = ''
  fieldForm.type = 'text'
  fieldForm.required = false
  fieldError.value = ''
  optionError.value = ''
  newOption.value = ''
  newOption.label = ''
  fieldOpen.value = true
}

function openEditField(field: Field) {
  editingField.value = field
  fieldForm.name = field.name
  fieldForm.type = field.type
  fieldForm.required = field.required
  fieldError.value = ''
  optionError.value = ''
  newOption.value = ''
  newOption.label = ''
  fieldOpen.value = true
}

async function saveField() {
  if (!detail.value) return
  fieldError.value = ''
  if (!fieldForm.name.trim()) {
    fieldError.value = 'name is required'
    return
  }
  if (!/^[a-z][a-z0-9_]*$/.test(fieldForm.name)) {
    fieldError.value = 'name must be lowercase snake_case (e.g. work_order)'
    return
  }
  savingField.value = true
  try {
    if (editingField.value) {
      await $fetch(`/api/meta/fields/${encodeURIComponent(editingField.value.id)}`, {
        method: 'PUT',
        body: { name: fieldForm.name, type: fieldForm.type, required: fieldForm.required }
      })
      fieldOpen.value = false
      await refreshDetail()
      toast.add({ title: 'Field updated', color: 'success', icon: 'i-lucide-check' })
    } else {
      const created = await $fetch<Field>(`/api/meta/entities/${encodeURIComponent(detail.value.id)}/fields`, {
        method: 'POST',
        body: { name: fieldForm.name, type: fieldForm.type, required: fieldForm.required }
      })
      await refreshDetail()
      if (created.type === 'select') {
        // keep slideover open so the user can add options right away
        editingField.value = created
        toast.add({ title: 'Field created — add options', color: 'success', icon: 'i-lucide-check' })
      } else {
        fieldOpen.value = false
        toast.add({ title: 'Field created', color: 'success', icon: 'i-lucide-check' })
      }
    }
  } catch (e: any) {
    fieldError.value = e?.data?.message || e?.statusMessage || 'Failed to save field'
  } finally {
    savingField.value = false
  }
}

async function addOption() {
  if (!editingField.value) return
  optionError.value = ''
  if (!newOption.value.trim() || !newOption.label.trim()) {
    optionError.value = 'value and label are required'
    return
  }
  try {
    await $fetch(`/api/meta/fields/${encodeURIComponent(editingField.value.id)}/options`, {
      method: 'POST',
      body: { value: newOption.value, label: newOption.label }
    })
    newOption.value = ''
    newOption.label = ''
    await refreshDetail()
  } catch (e: any) {
    optionError.value = e?.data?.message || e?.statusMessage || 'Failed to add option'
  }
}

async function removeOption(option: FieldOption) {
  if (!editingField.value) return
  optionError.value = ''
  try {
    await $fetch(`/api/meta/options/${encodeURIComponent(option.id)}`, { method: 'DELETE' })
    await refreshDetail()
  } catch (e: any) {
    optionError.value = e?.data?.message || e?.statusMessage || 'Failed to remove option'
  }
}

// --- Delete field ---
const deleteFieldOpen = ref(false)
const deletingField = ref(false)
const fieldToDelete = ref<Field | null>(null)

function confirmDeleteField(field: Field) {
  fieldToDelete.value = field
  deleteFieldOpen.value = true
}

async function removeField() {
  if (!fieldToDelete.value) return
  deletingField.value = true
  try {
    await $fetch(`/api/meta/fields/${encodeURIComponent(fieldToDelete.value.id)}`, { method: 'DELETE' })
    deleteFieldOpen.value = false
    fieldToDelete.value = null
    await refreshDetail()
    toast.add({ title: 'Field deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (e: any) {
    toast.add({
      title: 'Unable to delete field',
      description: e?.data?.message || e?.statusMessage || 'Delete failed',
      color: 'error',
      icon: 'i-lucide-alert-circle'
    })
  } finally {
    deletingField.value = false
  }
}

// --- Field table (UTable) ---
const fieldColumns: TableColumn<Field>[] = [
  {
    accessorKey: 'name',
    header: 'Name',
    cell: ({ row }) => h('span', { class: 'font-mono' }, row.original.name)
  },
  {
    accessorKey: 'type',
    header: 'Type',
    cell: ({ row }) => h(UBadge, { color: typeBadgeColor(row.original.type), variant: 'subtle' }, () => row.original.type)
  },
  {
    accessorKey: 'required',
    header: 'Required',
    cell: ({ row }) => row.original.required ? 'Yes' : 'No'
  },
  {
    id: 'actions',
    header: () => h('span', { class: 'sr-only' }, 'Actions'),
    cell: ({ row }) => h('div', { class: 'flex justify-end gap-1' }, [
      h(UButton, { size: 'xs', variant: 'ghost', onClick: () => openEditField(row.original) }, () => 'Edit'),
      h(UButton, { size: 'xs', variant: 'ghost', color: 'error', onClick: () => confirmDeleteField(row.original) }, () => 'Delete')
    ])
  }
]
</script>

<template>
  <UDashboardPanel id="entity-manager">
    <template #header>
      <UDashboardNavbar title="Entity Manager">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton
            icon="i-lucide-refresh-cw"
            variant="ghost"
            :loading="status === 'pending'"
            @click="refresh()"
          >
            Refresh
          </UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="grid h-full grid-cols-1 gap-4 lg:grid-cols-[280px_1fr]">
        <!-- Left: entity list -->
        <UCard class="h-fit lg:h-full">
          <UInput v-model="search" icon="i-lucide-search" placeholder="Search entities…" class="mb-3 w-full" />
          <div class="space-y-1">
            <UButton
              v-for="entity in filteredEntities"
              :key="entity.id"
              variant="ghost"
              class="w-full justify-between"
              :class="selectedId === entity.id ? 'bg-primary/10 text-primary' : ''"
              @click="selectEntity(entity.id)"
            >
              <span class="truncate">{{ entity.label }}</span>
              <span class="shrink-0 font-mono text-xs text-muted">{{ entity.id }}</span>
            </UButton>
            <p v-if="!filteredEntities.length" class="py-6 text-center text-sm text-muted">
              No entities found.
            </p>
          </div>
          <UButton icon="i-lucide-plus" class="mt-4 w-full" @click="createOpen = true">
            New Entity
          </UButton>
        </UCard>

        <!-- Right: detail -->
        <div v-if="!selectedId" class="flex flex-col items-center justify-center gap-3 py-24 text-center">
          <UIcon name="i-lucide-mouse-pointer-click" class="h-10 w-10 text-muted" />
          <p class="text-sm text-muted">Select an entity to manage its fields.</p>
        </div>

        <UCard v-else-if="detailStatus === 'pending'" class="h-fit">
          <USkeleton v-for="index in 4" :key="index" class="mb-3 h-8 w-full" />
        </UCard>

        <UCard v-else-if="detailError" class="h-fit">
          <UAlert
            color="error"
            title="Cannot load entity"
            :description="detailError.message || 'Check the Rust core connection.'"
          >
            <template #actions>
              <UButton size="sm" variant="outline" @click="refreshDetail()">Retry</UButton>
            </template>
          </UAlert>
        </UCard>

        <UCard v-else-if="detail" class="h-fit">
          <div class="mb-4 flex items-start justify-between gap-4">
            <div>
              <h2 class="text-lg font-semibold">{{ detail.label }}</h2>
              <p class="font-mono text-sm text-muted">{{ detail.name }}</p>
            </div>
            <div class="flex gap-2">
              <UButton size="sm" variant="outline" icon="i-lucide-pencil" @click="openEditEntity">
                Edit
              </UButton>
              <UButton size="sm" color="error" variant="ghost" icon="i-lucide-trash" @click="deleteEntityOpen = true">
                Delete
              </UButton>
            </div>
          </div>

          <UTabs :items="tabItems" class="w-full">
            <template #fields>
              <div class="flex items-center justify-between py-3">
                <p class="text-sm text-muted">{{ detail.fields.length }} fields</p>
                <UButton size="sm" icon="i-lucide-plus" @click="openAddField">Add field</UButton>
              </div>
              <UTable :data="detail.fields" :columns="fieldColumns" :get-row-id="(row: Field) => row.id" class="w-full">
                <template #empty>
                  <div class="py-10 text-center text-muted">
                    No fields yet. Add your first field to start creating records.
                  </div>
                </template>
              </UTable>
            </template>
            <template #permissions>
              <div class="py-10 text-center text-sm text-muted">Permissions are not available yet.</div>
            </template>
            <template #views>
              <div class="py-10 text-center text-sm text-muted">Views are not available yet.</div>
            </template>
          </UTabs>
        </UCard>
      </div>

      <!-- Create entity modal -->
      <UModal v-model:open="createOpen" title="New Entity">
        <template #body>
          <UForm class="space-y-4" @submit="createEntity">
            <UFormField label="ID" hint="lowercase, no spaces (e.g. work_order)">
              <UInput v-model="createForm.id" placeholder="work_order" />
            </UFormField>
            <UFormField label="Name">
              <UInput v-model="createForm.name" placeholder="work_order" />
            </UFormField>
            <UFormField label="Label">
              <UInput v-model="createForm.label" placeholder="Work Order" />
            </UFormField>
            <UAlert v-if="createError" color="error" :title="createError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="createOpen = false">Cancel</UButton>
            <UButton :loading="creating" @click="createEntity">Create Entity</UButton>
          </div>
        </template>
      </UModal>

      <!-- Edit entity modal -->
      <UModal v-model:open="editOpen" title="Edit Entity">
        <template #body>
          <UForm class="space-y-4" @submit="saveEntity">
            <UFormField label="Name">
              <UInput v-model="editForm.name" />
            </UFormField>
            <UFormField label="Label">
              <UInput v-model="editForm.label" />
            </UFormField>
            <UAlert v-if="editError" color="error" :title="editError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="editOpen = false">Cancel</UButton>
            <UButton :loading="editing" @click="saveEntity">Save</UButton>
          </div>
        </template>
      </UModal>

      <!-- Delete entity modal -->
      <UModal v-model:open="deleteEntityOpen" :title="`Delete ${detail?.label || 'entity'}`">
        <template #body>
          <p class="text-sm text-muted">
            This will permanently delete this entity and all its fields. This action cannot be undone.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="deleteEntityOpen = false">Cancel</UButton>
            <UButton color="error" :loading="deletingEntity" @click="removeEntity">Delete</UButton>
          </div>
        </template>
      </UModal>

      <!-- Field slideover -->
      <USlideover v-model:open="fieldOpen" :title="editingField ? 'Edit field' : 'Add field'">
        <template #body>
          <UForm class="space-y-4" @submit="saveField">
            <UFormField label="Name" hint="lowercase, no spaces (e.g. work_order)">
              <UInput v-model="fieldForm.name" placeholder="title" />
            </UFormField>
            <UFormField label="Type">
              <USelectMenu v-model="fieldForm.type" :items="typeItems" value-key="value" class="w-full" />
            </UFormField>
            <UFormField label="Required">
              <USwitch v-model="fieldForm.required" />
            </UFormField>

            <div v-if="fieldForm.type === 'select'">
              <div class="mb-2 flex items-center justify-between">
                <p class="text-sm font-medium">Options</p>
                <p v-if="editingField" class="text-xs text-muted">{{ editingFieldOptions.length }} options</p>
              </div>
              <div v-if="editingField" class="space-y-2">
                <div v-for="option in editingFieldOptions" :key="option.id" class="flex items-center gap-2">
                  <span class="w-24 truncate font-mono text-xs text-muted">{{ option.value }}</span>
                  <span class="flex-1 text-sm">{{ option.label }}</span>
                  <UButton size="xs" variant="ghost" color="error" icon="i-lucide-x" @click="removeOption(option)" />
                </div>
                <p v-if="!editingFieldOptions.length" class="text-sm text-muted">No options yet.</p>
              </div>
              <p v-else class="text-sm text-muted">Save the field first, then add options.</p>
              <div v-if="editingField" class="mt-3 flex items-center gap-2">
                <UInput v-model="newOption.value" placeholder="value (e.g. open)" class="w-28" />
                <UInput v-model="newOption.label" placeholder="label (e.g. Open)" class="flex-1" />
                <UButton size="sm" icon="i-lucide-plus" @click="addOption" />
              </div>
              <UAlert v-if="optionError" color="error" :title="optionError" class="mt-2" />
            </div>

            <UAlert v-if="fieldError" color="error" :title="fieldError" />
          </UForm>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="fieldOpen = false">Cancel</UButton>
            <UButton :loading="savingField" @click="saveField">
              {{ editingField ? 'Save' : 'Create field' }}
            </UButton>
          </div>
        </template>
      </USlideover>

      <!-- Delete field modal -->
      <UModal v-model:open="deleteFieldOpen" title="Delete field">
        <template #body>
          <p class="text-sm text-muted">
            This will permanently delete the field
            <span class="font-mono">{{ fieldToDelete?.name }}</span>. This action cannot be undone.
          </p>
        </template>
        <template #footer>
          <div class="flex justify-end gap-2">
            <UButton variant="ghost" @click="deleteFieldOpen = false">Cancel</UButton>
            <UButton color="error" :loading="deletingField" @click="removeField">Delete</UButton>
          </div>
        </template>
      </UModal>
    </template>
  </UDashboardPanel>
</template>