<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

type ModuleEntityField = {
  name: string
  label?: string
  type: string
  required?: boolean
  is_status?: boolean
  ref_entity?: string
  computed_expr?: string
  options?: { value: string; label: string }[]
}
type ModuleEntity = {
  name: string
  label?: string
  fields: ModuleEntityField[]
  workflow?: { states: { name: string; label: string }[]; transitions: { from_state: string; to_state: string; action: string }[] }
  views?: { name: string; config?: Record<string, unknown> }[]
}
type ModuleVersion = { id: string; module_id: string; version: number; actor: string | null; created_at: string }
type Module = {
  id: string
  name: string
  label: string
  description: string
  icon: string
  color: string
  owner: string
  status: string
  version: number
  definition: { entities?: ModuleEntity[] }
}

const toast = useToast()
const route = useRoute()
const moduleId = computed(() => String(route.params.id || ''))
const moduleUrl = computed(() => moduleId.value ? `/api/modules/${encodeURIComponent(moduleId.value)}` : '')
const { data: module, status, error, refresh } = await useFetch<Module>(moduleUrl, { watch: [moduleUrl] })
const versionsUrl = computed(() => moduleId.value ? `/api/modules/${encodeURIComponent(moduleId.value)}/versions` : '')
const { data: versions, refresh: refreshVersions } = await useFetch<ModuleVersion[]>(versionsUrl, { watch: [versionsUrl] })

const saving = ref(false)
const saveError = ref('')
const publishing = ref(false)
const entityForm = reactive({ name: '', label: '' })
const fieldForms = ref<Record<string, { name: string; type: string; required: boolean; is_status: boolean; ref_entity: string; computed_expr: string }>>({})
const newState = reactive({ entity: '', name: '', label: '' })
const newTransition = reactive({ entity: '', from_state: '', to_state: '', action: '' })

const entities = computed(() => module.value?.definition?.entities || [])
const isDraft = computed(() => module.value?.status === 'draft')
const isReview = computed(() => module.value?.status === 'review')
const isPublished = computed(() => module.value?.status === 'published')
const isEnabled = computed(() => module.value?.status === 'enabled')
const isDisabled = computed(() => module.value?.status === 'disabled')
const isArchived = computed(() => module.value?.status === 'archived')

const typeItems = [
  { label: 'Text', value: 'text' },
  { label: 'Number', value: 'number' },
  { label: 'Date', value: 'date' },
  { label: 'Select', value: 'select' },
  { label: 'Checkbox', value: 'checkbox' },
  { label: 'Textarea', value: 'textarea' },
  { label: 'Currency', value: 'currency' },
  { label: 'Reference', value: 'reference' },
  { label: 'Computed', value: 'computed' }
]

function fieldFormFor(entityName: string) {
  if (!fieldForms.value[entityName]) {
    fieldForms.value[entityName] = { name: '', type: 'text', required: false, is_status: false, ref_entity: '', computed_expr: '' }
  }
  return fieldForms.value[entityName]!
}

function cloneDefinition() {
  return JSON.parse(JSON.stringify(module.value?.definition || { entities: [] }))
}

async function saveDefinition(definition: { entities: ModuleEntity[] }, message = 'Draft saved') {
  saveError.value = ''
  saving.value = true
  try {
    await $fetch(`/api/modules/${encodeURIComponent(moduleId.value)}`, { method: 'PUT', body: { definition } })
    await refresh()
    toast.add({ title: message, color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    saveError.value = cause?.data?.message || cause?.statusMessage || 'Save failed'
  } finally {
    saving.value = false
  }
}

async function addEntity() {
  if (!entityForm.name.trim()) return
  const definition = cloneDefinition()
  definition.entities.push({ name: entityForm.name.trim(), label: entityForm.label.trim() || entityForm.name.trim(), fields: [] })
  entityForm.name = ''
  entityForm.label = ''
  await saveDefinition(definition, 'Entity added')
}

async function removeEntity(name: string) {
  const definition = cloneDefinition()
  definition.entities = definition.entities.filter((e: ModuleEntity) => e.name !== name)
  await saveDefinition(definition, 'Entity removed')
}

async function addField(entityName: string) {
  const form = fieldFormFor(entityName)
  if (!form.name.trim()) return
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === entityName)
  if (!entity) return
  entity.fields.push({
    name: form.name.trim(),
    type: form.type,
    required: form.required,
    is_status: form.is_status,
    ...(form.ref_entity.trim() ? { ref_entity: form.ref_entity.trim() } : {}),
    ...(form.computed_expr.trim() ? { computed_expr: form.computed_expr.trim() } : {})
  })
  fieldForms.value[entityName] = { name: '', type: 'text', required: false, is_status: false, ref_entity: '', computed_expr: '' }
  await saveDefinition(definition, 'Field added')
}

async function removeField(entityName: string, fieldName: string) {
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === entityName)
  if (!entity) return
  entity.fields = entity.fields.filter((f: ModuleEntityField) => f.name !== fieldName)
  await saveDefinition(definition, 'Field removed')
}

async function addState() {
  if (!newState.entity || !newState.name.trim()) return
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === newState.entity)
  if (!entity) return
  entity.workflow = entity.workflow || { states: [], transitions: [] }
  entity.workflow.states.push({ name: newState.name.trim(), label: newState.label.trim() || newState.name.trim() })
  newState.name = ''
  newState.label = ''
  await saveDefinition(definition, 'State added')
}

async function addTransition() {
  if (!newTransition.entity || !newTransition.from_state || !newTransition.to_state || !newTransition.action.trim()) return
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === newTransition.entity)
  if (!entity) return
  entity.workflow = entity.workflow || { states: [], transitions: [] }
  entity.workflow.transitions.push({ from_state: newTransition.from_state, to_state: newTransition.to_state, action: newTransition.action.trim() })
  newTransition.action = ''
  await saveDefinition(definition, 'Transition added')
}

async function review() {
  try {
    await $fetch(`/api/modules/${encodeURIComponent(moduleId.value)}/review`, { method: 'POST' })
    await refresh()
    toast.add({ title: 'Module submitted for review', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    saveError.value = cause?.data?.message || cause?.statusMessage || 'Review submission failed'
  }
}

async function enable() {
  try {
    await $fetch(`/api/modules/${encodeURIComponent(moduleId.value)}/enable`, { method: 'POST' })
    await refresh()
  } catch (cause: any) {
    saveError.value = cause?.data?.message || cause?.statusMessage || 'Enable failed'
  }
}

async function disable() {
  try {
    await $fetch(`/api/modules/${encodeURIComponent(moduleId.value)}/disable`, { method: 'POST' })
    await refresh()
  } catch (cause: any) {
    saveError.value = cause?.data?.message || cause?.statusMessage || 'Disable failed'
  }
}

async function publish() {
  publishing.value = true
  saveError.value = ''
  try {
    await $fetch(`/api/modules/${encodeURIComponent(moduleId.value)}/publish`, { method: 'POST' })
    await refresh()
    await refreshVersions()
    toast.add({ title: 'Module published', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    saveError.value = cause?.data?.message || cause?.statusMessage || 'Publish failed'
  } finally {
    publishing.value = false
  }
}

async function archive() {
  try {
    await $fetch(`/api/modules/${encodeURIComponent(moduleId.value)}/archive`, { method: 'POST' })
    await refresh()
  } catch (cause: any) {
    saveError.value = cause?.data?.message || cause?.statusMessage || 'Archive failed'
  }
}

async function restore() {
  try {
    await $fetch(`/api/modules/${encodeURIComponent(moduleId.value)}/restore`, { method: 'POST' })
    await refresh()
  } catch (cause: any) {
    saveError.value = cause?.data?.message || cause?.statusMessage || 'Restore failed'
  }
}

async function rollback(version: number) {
  try {
    await $fetch(`/api/modules/${encodeURIComponent(moduleId.value)}/rollback`, { method: 'POST', body: { version } })
    await refresh()
    await refreshVersions()
    toast.add({ title: `Rolled back to v${version}`, color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    saveError.value = cause?.data?.message || cause?.statusMessage || 'Rollback failed'
  }
}
</script>

<template>
  <UDashboardPanel id="module-detail">
    <template #header>
      <UDashboardNavbar :title="module?.label || 'Module'">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
        <template #right>
          <UButton variant="ghost" to="/admin/modules" icon="i-lucide-arrow-left">Modules</UButton>
          <UButton v-if="isDraft" variant="outline" icon="i-lucide-send" @click="review">Submit for review</UButton>
          <UButton v-if="isReview" :loading="publishing" icon="i-lucide-rocket" @click="publish">Publish</UButton>
          <UButton v-if="isPublished" variant="outline" icon="i-lucide-play" @click="enable">Enable</UButton>
          <UButton v-if="isEnabled" variant="outline" icon="i-lucide-pause" @click="disable">Disable</UButton>
          <UButton v-if="isPublished || isDisabled" variant="outline" @click="archive">Archive</UButton>
          <UButton v-if="isArchived" variant="outline" @click="restore">Restore to draft</UButton>
        </template>
      </UDashboardNavbar>
    </template>
    <template #body>
      <div class="mx-auto max-w-5xl space-y-4">
        <UBreadcrumb :items="[{ label: 'Modules', to: '/admin/modules' }, { label: module?.label || moduleId }]" />
        <UAlert v-if="error" color="error" title="Cannot load module" :description="error.message" />
        <UAlert v-if="saveError" color="error" :title="saveError" />
        <UCard v-if="module">
          <template #header>
            <div class="flex items-center justify-between">
              <h2 class="text-sm font-semibold">Definition · {{ module.owner }}/{{ module.name }} · v{{ module.version }}</h2>
              <UBadge :color="module.status === 'published' ? 'success' : module.status === 'archived' ? 'neutral' : 'warning'" variant="subtle">{{ module.status }}</UBadge>
            </div>
          </template>
          <p class="text-sm text-muted">Entities materialize as <span class="font-mono">{{ module.id }}_&lt;entity&gt;</span> on publish. Draft edits never touch live records.</p>
        </UCard>
        <UCard>
          <template #header><h2 class="text-sm font-semibold">Entities</h2></template>
          <div class="mb-3 grid grid-cols-1 gap-2 sm:grid-cols-[1fr_1fr_auto]">
            <UInput v-model="entityForm.name" placeholder="vehicle (snake_case)" />
            <UInput v-model="entityForm.label" placeholder="Vehicle" />
            <UButton :loading="saving" :disabled="!entityForm.name.trim()" @click="addEntity">Add entity</UButton>
          </div>
          <div v-for="entity in entities" :key="entity.name" class="mb-4 rounded-lg border border-muted p-3">
            <div class="flex items-center justify-between">
              <h3 class="font-mono text-sm font-semibold">{{ entity.name }} <span class="text-muted">· {{ entity.label || entity.name }}</span></h3>
              <UButton size="xs" variant="ghost" color="error" @click="removeEntity(entity.name)">Remove</UButton>
            </div>
            <div class="mt-2 space-y-1">
              <div v-for="field in entity.fields" :key="field.name" class="flex items-center justify-between rounded bg-muted/40 px-2 py-1 text-sm">
                <span class="font-mono">{{ field.name }} <span class="text-muted">· {{ field.type }}{{ field.required ? ' · required' : '' }}{{ field.is_status ? ' · status' : '' }}{{ field.ref_entity ? ` → ${field.ref_entity}` : '' }}</span></span>
                <UButton size="xs" variant="ghost" color="error" @click="removeField(entity.name, field.name)">Remove</UButton>
              </div>
              <p v-if="!entity.fields.length" class="text-xs text-muted">No fields yet.</p>
            </div>
            <div class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-[1fr_auto_auto_auto_auto_auto]">
              <UInput v-model="fieldFormFor(entity.name).name" placeholder="plate_number" />
              <USelectMenu v-model="fieldFormFor(entity.name).type" :items="typeItems" value-key="value" class="sm:w-32" />
              <UInput v-model="fieldFormFor(entity.name).ref_entity" placeholder="ref entity" class="sm:w-32" />
              <UInput v-model="fieldFormFor(entity.name).computed_expr" placeholder="{code}" class="sm:w-32" />
              <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="fieldFormFor(entity.name).required" /> Req</label>
              <UButton size="sm" :disabled="!fieldFormFor(entity.name).name.trim()" @click="addField(entity.name)">Add field</UButton>
            </div>
          </div>
        </UCard>
        <UCard>
          <template #header><h2 class="text-sm font-semibold">Workflow</h2></template>
          <div class="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_1fr_1fr_auto]">
            <USelectMenu v-model="newState.entity" :items="entities.map(e => ({ label: e.label || e.name, value: e.name }))" value-key="value" placeholder="Entity…" />
            <UInput v-model="newState.name" placeholder="maintenance" />
            <UInput v-model="newState.label" placeholder="Maintenance" />
            <UButton :disabled="!newState.entity || !newState.name.trim()" @click="addState">Add state</UButton>
          </div>
          <div class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-[1fr_1fr_1fr_1fr_auto]">
            <USelectMenu v-model="newTransition.entity" :items="entities.map(e => ({ label: e.label || e.name, value: e.name }))" value-key="value" placeholder="Entity…" />
            <UInput v-model="newTransition.from_state" placeholder="active" />
            <UInput v-model="newTransition.to_state" placeholder="maintenance" />
            <UInput v-model="newTransition.action" placeholder="send_to_maintenance" />
            <UButton @click="addTransition">Add transition</UButton>
          </div>
          <div v-for="entity in entities" :key="`wf-${entity.name}`" class="mt-2 text-sm">
            <p class="font-mono">{{ entity.name }}: {{ (entity.workflow?.states || []).map(s => s.name).join(' → ') || 'no states' }}</p>
          </div>
        </UCard>
        <UCard>
          <template #header><h2 class="text-sm font-semibold">Versions</h2></template>
          <ol v-if="(versions || []).length" class="space-y-1">
            <li v-for="version in versions || []" :key="version.id" class="flex items-center justify-between rounded bg-muted/40 px-3 py-2 text-sm">
              <span class="font-mono">v{{ version.version }} · {{ version.actor || 'system' }} · {{ version.created_at }}</span>
              <UButton size="xs" variant="outline" @click="rollback(version.version)">Rollback</UButton>
            </li>
          </ol>
          <p v-else class="text-sm text-muted">No published versions yet. Publish to snapshot.</p>
        </UCard>
      </div>
    </template>
  </UDashboardPanel>
</template>
