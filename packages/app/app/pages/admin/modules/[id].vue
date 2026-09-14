<script setup lang="ts">
definePageMeta({ middleware: 'auth' })

type ModuleFieldOption = { value: string; label: string }
type ModuleEntityField = {
  name: string
  label?: string
  description?: string
  type: string
  required?: boolean
  is_status?: boolean
  ref_entity?: string
  computed_expr?: string
  options?: ModuleFieldOption[]
  unique?: boolean
  min_value?: number | null
  max_value?: number | null
  pattern?: string | null
  min_length?: number | null
  max_length?: number | null
  default?: string | null
  auto_number?: { prefix?: string; width?: number } | null
  readonly?: boolean
  hidden?: boolean
  searchable?: boolean
  sortable?: boolean
  filterable?: boolean
  indexed?: boolean
  precision?: number | null
  help_text?: string | null
}
type ModuleEntityView = { name: string; config?: Record<string, unknown> }
type ModuleEntityReport = { name: string; config?: Record<string, unknown> }
type ModuleEntityNotification = { target_url: string; trigger?: string; active?: boolean }
type ModuleLayoutSection = { id: string; label: string; fields: string[] }
type ModuleEntity = {
  name: string
  label?: string
  description?: string
  fields: ModuleEntityField[]
  workflow?: { states: { name: string; label: string }[]; transitions: { from_state: string; to_state: string; action: string }[] }
  views?: ModuleEntityView[]
  form_layout?: { config: { sections: ModuleLayoutSection[] } }
  reports?: ModuleEntityReport[]
  notifications?: ModuleEntityNotification[]
}
type ModuleRelation = {
  id: string
  source_entity_id: string
  target_entity_id: string
  name: string
  relation_type: string
  on_delete: string
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
  definition: { entities?: ModuleEntity[]; settings?: Record<string, unknown> }
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
const entityForm = reactive({ name: '', label: '', description: '' })
const settingsText = ref('{}')
const settingsError = ref('')
const fieldForms = ref<Record<string, {
  name: string
  label: string
  type: string
  required: boolean
  is_status: boolean
  ref_entity: string
  computed_expr: string
  unique: boolean
  min_value: number | null
  max_value: number | null
  pattern: string
  min_length: number | null
  max_length: number | null
  default_value: string
  auto_number_prefix: string
  auto_number_width: number | null
  readonly: boolean
  hidden: boolean
  searchable: boolean
  sortable: boolean
  filterable: boolean
  indexed: boolean
  precision: number | null
  help_text: string
}>>({})
const newState = reactive({ entity: '', name: '', label: '' })
const newTransition = reactive({ entity: '', from_state: '', to_state: '', action: '' })
const newOption = reactive({ entity: '', field: '', value: '', label: '' })
const newView = reactive({ entity: '', name: '' })
const newReport = reactive({ entity: '', name: '', group_by: '', chart_type: '' })
const newNotification = reactive({ entity: '', target_url: '', trigger: 'transition', active: true })
const newRelation = reactive({ source: '', target: '', name: '', relation_type: 'many_to_one', on_delete: 'restrict' })
const layoutForms = ref<Record<string, { label: string }>>({})
const layoutFieldPick = ref<Record<string, string>>({})

const entities = computed(() => module.value?.definition?.entities || [])
watch(module, value => { settingsText.value = JSON.stringify(value?.definition?.settings || {}, null, 2) }, { immediate: true })
const isDraft = computed(() => module.value?.status === 'draft')
const isReview = computed(() => module.value?.status === 'review')
const isPublished = computed(() => module.value?.status === 'published')
const isEnabled = computed(() => module.value?.status === 'enabled')
const isDisabled = computed(() => module.value?.status === 'disabled')
const isArchived = computed(() => module.value?.status === 'archived')

const typeItems = [
  { label: 'Text', value: 'text' },
  { label: 'Long text', value: 'long_text' },
  { label: 'Number', value: 'number' },
  { label: 'Decimal', value: 'decimal' },
  { label: 'Currency', value: 'currency' },
  { label: 'Percentage', value: 'percentage' },
  { label: 'Integer', value: 'integer' },
  { label: 'Boolean', value: 'boolean' },
  { label: 'Date', value: 'date' },
  { label: 'Datetime', value: 'datetime' },
  { label: 'Time', value: 'time' },
  { label: 'Select', value: 'select' },
  { label: 'Multi-select', value: 'multi_select' },
  { label: 'Email', value: 'email' },
  { label: 'Phone', value: 'phone' },
  { label: 'URL', value: 'url' },
  { label: 'JSON', value: 'json' },
  { label: 'File', value: 'file' },
  { label: 'Image', value: 'image' },
  { label: 'Reference', value: 'reference' },
  { label: 'Computed', value: 'computed' },
  { label: 'Formula', value: 'formula' },
  { label: 'Auto-number', value: 'auto_number' }
]

const relationTypeItems = [
  { label: 'One to one', value: 'one_to_one' },
  { label: 'One to many', value: 'one_to_many' },
  { label: 'Many to one', value: 'many_to_one' },
  { label: 'Many to many', value: 'many_to_many' }
]

const deleteRuleItems = [
  { label: 'Restrict', value: 'restrict' },
  { label: 'Set null', value: 'set_null' },
  { label: 'Cascade', value: 'cascade' }
]

const chartTypeItems = [
  { label: 'Bar', value: 'bar' },
  { label: 'Pie', value: 'pie' }
]

const triggerItems = [
  { label: 'Transition', value: 'transition' }
]

function blankFieldForm() {
  return {
    name: '', label: '', type: 'text', required: false, is_status: false,
    ref_entity: '', computed_expr: '', unique: false,
    min_value: null as number | null, max_value: null as number | null,
    pattern: '', min_length: null as number | null, max_length: null as number | null,
    default_value: '', auto_number_prefix: '', auto_number_width: null as number | null,
    readonly: false, hidden: false, searchable: false, sortable: false,
    filterable: false, indexed: false, precision: null as number | null, help_text: ''
  }
}

function fieldFormFor(entityName: string) {
  if (!fieldForms.value[entityName]) {
    fieldForms.value[entityName] = blankFieldForm()
  }
  return fieldForms.value[entityName]!
}

function isNumericType(type: string) {
  return ['number', 'decimal', 'currency', 'percentage', 'integer'].includes(type)
}

function isTextType(type: string) {
  return ['text', 'long_text', 'textarea', 'email', 'phone', 'url'].includes(type)
}

function layoutFormFor(entityName: string) {
  if (!layoutForms.value[entityName]) layoutForms.value[entityName] = { label: '' }
  return layoutForms.value[entityName]!
}

function materializedId(entityName: string) {
  return module.value ? `${module.value.id}_${entityName}` : ''
}

function materializedFieldId(entityName: string, fieldName: string) {
  return `${materializedId(entityName)}_${fieldName}`
}

function fieldDisplayName(entityName: string, ref: string) {
  // Layout sections store materialized field IDs (matching the entity
  // manager convention); fall back to showing the raw ref for legacy
  // definitions that stored plain field names.
  const prefix = `${materializedId(entityName)}_`
  return ref.startsWith(prefix) ? ref.slice(prefix.length) : ref
}

function cloneDefinition(): { entities: ModuleEntity[]; settings?: Record<string, unknown> } {
  return JSON.parse(JSON.stringify(module.value?.definition || { entities: [] }))
}

async function saveSettings() {
  settingsError.value = ''
  try {
    const definition = cloneDefinition()
    definition.settings = JSON.parse(settingsText.value || '{}')
    await saveDefinition(definition, 'Module settings saved')
  } catch {
    settingsError.value = 'Settings must be valid JSON'
  }
}

async function saveDefinition(definition: { entities: ModuleEntity[]; settings?: Record<string, unknown> }, message = 'Draft saved') {
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
  if (!/^[a-z][a-z0-9_]*$/.test(entityForm.name.trim())) {
    saveError.value = 'entity name must be lowercase snake_case (e.g. vehicle)'
    return
  }
  const definition = cloneDefinition()
  definition.entities.push({
    name: entityForm.name.trim(),
    label: entityForm.label.trim() || entityForm.name.trim(),
    ...(entityForm.description.trim() ? { description: entityForm.description.trim() } : {}),
    fields: []
  })
  entityForm.name = ''
  entityForm.label = ''
  entityForm.description = ''
  await saveDefinition(definition, 'Entity added')
}

async function removeEntity(name: string) {
  const definition = cloneDefinition()
  definition.entities = definition.entities.filter((e: ModuleEntity) => e.name !== name)
  await saveDefinition(definition, 'Entity removed')
}

function buildFieldPayload(form: ReturnType<typeof blankFieldForm>) {
  const payload: ModuleEntityField = {
    name: form.name.trim(),
    ...(form.label.trim() ? { label: form.label.trim() } : {}),
    type: form.type,
    ...(form.required ? { required: true } : {}),
    ...(form.is_status ? { is_status: true } : {}),
    ...(form.ref_entity.trim() ? { ref_entity: form.ref_entity.trim() } : {}),
    ...(form.computed_expr.trim() ? { computed_expr: form.computed_expr.trim() } : {}),
    ...(form.unique ? { unique: true } : {}),
    ...(form.min_value !== null ? { min_value: form.min_value } : {}),
    ...(form.max_value !== null ? { max_value: form.max_value } : {}),
    ...(form.pattern.trim() ? { pattern: form.pattern.trim() } : {}),
    ...(form.min_length !== null ? { min_length: form.min_length } : {}),
    ...(form.max_length !== null ? { max_length: form.max_length } : {}),
    ...(form.default_value.trim() ? { default: form.default_value.trim() } : {}),
    ...((form.auto_number_prefix.trim() || form.auto_number_width !== null)
      ? { auto_number: { ...(form.auto_number_prefix.trim() ? { prefix: form.auto_number_prefix.trim() } : {}), ...(form.auto_number_width !== null ? { width: form.auto_number_width } : {}) } }
      : {}),
    ...(form.readonly ? { readonly: true } : {}),
    ...(form.hidden ? { hidden: true } : {}),
    ...(form.searchable ? { searchable: true } : {}),
    ...(form.sortable ? { sortable: true } : {}),
    ...(form.filterable ? { filterable: true } : {}),
    ...(form.indexed ? { indexed: true } : {}),
    ...(form.precision !== null ? { precision: form.precision } : {}),
    ...(form.help_text.trim() ? { help_text: form.help_text.trim() } : {})
  }
  return payload
}

async function addField(entityName: string) {
  const form = fieldFormFor(entityName)
  if (!form.name.trim()) return
  if (!/^[a-z][a-z0-9_]*$/.test(form.name.trim())) {
    saveError.value = 'field name must be lowercase snake_case (e.g. plate_number)'
    return
  }
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === entityName)
  if (!entity) return
  if ((entity.fields || []).some(f => f.name === form.name.trim())) {
    saveError.value = `field already exists: ${form.name.trim()}`
    return
  }
  if (form.is_status && form.type !== 'select') {
    saveError.value = 'status field must be of type select'
    return
  }
  if (form.type === 'reference' && !form.ref_entity.trim()) {
    saveError.value = 'reference fields require a ref_entity'
    return
  }
  if ((entity.fields || []).some(f => f.is_status) && form.is_status) {
    saveError.value = 'entity must have at most one status field'
    return
  }
  entity.fields = entity.fields || []
  entity.fields.push(buildFieldPayload(form))
  fieldForms.value[entityName] = blankFieldForm()
  await saveDefinition(definition, 'Field added')
}

async function removeField(entityName: string, fieldName: string) {
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === entityName)
  if (!entity) return
  entity.fields = entity.fields.filter((f: ModuleEntityField) => f.name !== fieldName)
  await saveDefinition(definition, 'Field removed')
}

async function addOption() {
  if (!newOption.entity || !newOption.field || !newOption.value.trim() || !newOption.label.trim()) return
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === newOption.entity)
  const field = entity?.fields?.find(f => f.name === newOption.field)
  if (!field) return
  if (field.type !== 'select' && field.type !== 'multi_select') {
    saveError.value = 'options are only valid for select fields'
    return
  }
  field.options = field.options || []
  if (field.options.some(o => o.value === newOption.value.trim())) {
    saveError.value = `option already exists: ${newOption.value.trim()}`
    return
  }
  field.options.push({ value: newOption.value.trim(), label: newOption.label.trim() })
  newOption.value = ''
  newOption.label = ''
  await saveDefinition(definition, 'Option added')
}

async function removeOption(entityName: string, fieldName: string, value: string) {
  const definition = cloneDefinition()
  const field = definition.entities
    .find((e: ModuleEntity) => e.name === entityName)?.fields
    ?.find(f => f.name === fieldName)
  if (!field?.options) return
  if (field.options.length <= 1) {
    saveError.value = 'select field must have at least one option'
    return
  }
  field.options = field.options.filter(o => o.value !== value)
  await saveDefinition(definition, 'Option removed')
}

async function addView() {
  if (!newView.entity || !newView.name.trim()) return
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === newView.entity)
  if (!entity) return
  entity.views = entity.views || []
  if (entity.views.some(v => v.name === newView.name.trim())) {
    saveError.value = `view already exists: ${newView.name.trim()}`
    return
  }
  entity.views.push({ name: newView.name.trim(), config: {} })
  newView.name = ''
  await saveDefinition(definition, 'View added')
}

async function removeView(entityName: string, viewName: string) {
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === entityName)
  if (!entity?.views) return
  entity.views = entity.views.filter(v => v.name !== viewName)
  await saveDefinition(definition, 'View removed')
}

async function addReport() {
  if (!newReport.entity || !newReport.name.trim() || !newReport.group_by.trim()) return
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === newReport.entity)
  if (!entity) return
  entity.reports = entity.reports || []
  if (entity.reports.some(r => r.name === newReport.name.trim())) {
    saveError.value = `report already exists: ${newReport.name.trim()}`
    return
  }
  entity.reports.push({
    name: newReport.name.trim(),
    config: {
      group_by: newReport.group_by.trim(),
      ...(newReport.chart_type ? { chart_type: newReport.chart_type } : {})
    }
  })
  newReport.name = ''
  newReport.group_by = ''
  await saveDefinition(definition, 'Report added')
}

async function removeReport(entityName: string, reportName: string) {
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === entityName)
  if (!entity?.reports) return
  entity.reports = entity.reports.filter(r => r.name !== reportName)
  await saveDefinition(definition, 'Report removed')
}

async function addNotification() {
  if (!newNotification.entity || !newNotification.target_url.trim()) return
  const url = newNotification.target_url.trim()
  if (!(url.startsWith('http://') || url.startsWith('https://'))) {
    saveError.value = 'target_url must start with http:// or https://'
    return
  }
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === newNotification.entity)
  if (!entity) return
  entity.notifications = entity.notifications || []
  entity.notifications.push({
    target_url: url,
    trigger: newNotification.trigger,
    active: newNotification.active
  })
  newNotification.target_url = ''
  await saveDefinition(definition, 'Notification rule added')
}

async function removeNotification(entityName: string, index: number) {
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === entityName)
  if (!entity?.notifications) return
  entity.notifications.splice(index, 1)
  await saveDefinition(definition, 'Notification rule removed')
}

function slugifySection(label: string, existing: ModuleLayoutSection[]) {
  const id = label.toLowerCase().replace(/[^a-z0-9]+/g, '_').replace(/^_|_$/g, '')
    || `section_${existing.length + 1}`
  return id
}

async function addLayoutSection(entityName: string) {
  const form = layoutFormFor(entityName)
  if (!form.label.trim()) return
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === entityName)
  if (!entity) return
  const sections = entity.form_layout?.config?.sections || []
  const id = slugifySection(form.label.trim(), sections)
  if (sections.some(s => s.id === id)) {
    saveError.value = `section already exists: ${id}`
    return
  }
  sections.push({ id, label: form.label.trim(), fields: [] })
  entity.form_layout = { config: { sections } }
  form.label = ''
  await saveDefinition(definition, 'Section added')
}

async function removeLayoutSection(entityName: string, sectionId: string) {
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === entityName)
  if (!entity?.form_layout?.config?.sections) return
  entity.form_layout.config.sections = entity.form_layout.config.sections.filter(s => s.id !== sectionId)
  await saveDefinition(definition, 'Section removed')
}

async function addFieldToLayout(entityName: string, sectionId: string) {
  const picked = (layoutFieldPick.value[`${entityName}:${sectionId}`] || '').trim()
  if (!picked) return
  const definition = cloneDefinition()
  const entity = definition.entities.find((e: ModuleEntity) => e.name === entityName)
  const section = entity?.form_layout?.config?.sections?.find(s => s.id === sectionId)
  if (!section) return
  if (!(entity?.fields || []).some(f => f.name === picked)) {
    saveError.value = `unknown field: ${picked}`
    return
  }
  // Store the materialized field ID so the runtime (which resolves layout
  // entries by field ID) renders the section correctly after publish.
  const fieldId = materializedFieldId(entityName, picked)
  const assigned = new Set((entity?.form_layout?.config?.sections || []).flatMap(s => s.fields))
  if (assigned.has(fieldId) || assigned.has(picked)) {
    saveError.value = `field already in layout: ${picked}`
    return
  }
  section.fields.push(fieldId)
  layoutFieldPick.value[`${entityName}:${sectionId}`] = ''
  await saveDefinition(definition, 'Layout updated')
}

async function removeFieldFromLayout(entityName: string, sectionId: string, ref: string) {
  const definition = cloneDefinition()
  const section = definition.entities
    .find((e: ModuleEntity) => e.name === entityName)
    ?.form_layout?.config?.sections?.find(s => s.id === sectionId)
  if (!section) return
  section.fields = section.fields.filter(f => f !== ref)
  await saveDefinition(definition, 'Layout updated')
}

function unassignedLayoutFields(entity: ModuleEntity) {
  const assigned = new Set((entity.form_layout?.config?.sections || []).flatMap(s => s.fields))
  const prefix = `${materializedId(entity.name)}_`
  return (entity.fields || []).filter(f =>
    !f.is_status && !assigned.has(f.name) && !assigned.has(`${prefix}${f.name}`)
  )
}

// --- Relations (post-publish, on materialized entities) ---
const relationsByEntity = ref<Record<string, ModuleRelation[]>>({})
const relationsLoading = ref(false)
const relationsError = ref('')

async function loadRelations() {
  if (!module.value || module.value.status === 'draft') {
    relationsByEntity.value = {}
    return
  }
  relationsLoading.value = true
  relationsError.value = ''
  try {
    const next: Record<string, ModuleRelation[]> = {}
    for (const entity of entities.value) {
      const id = materializedId(entity.name)
      try {
        next[entity.name] = await $fetch<ModuleRelation[]>(`/api/meta/entities/${encodeURIComponent(id)}/relations`)
      } catch {
        next[entity.name] = []
      }
    }
    relationsByEntity.value = next
  } catch (cause: any) {
    relationsError.value = cause?.data?.message || cause?.statusMessage || 'Unable to load relations'
  } finally {
    relationsLoading.value = false
  }
}

watch([module, entities], () => { loadRelations() }, { immediate: true })

async function addRelation() {
  relationsError.value = ''
  if (!newRelation.source || !newRelation.target || !newRelation.name.trim()) {
    relationsError.value = 'source, target, and name are required'
    return
  }
  saving.value = true
  try {
    await $fetch(`/api/meta/entities/${encodeURIComponent(materializedId(newRelation.source))}/relations`, {
      method: 'POST',
      body: {
        name: newRelation.name.trim(),
        target_entity_id: materializedId(newRelation.target),
        relation_type: newRelation.relation_type,
        on_delete: newRelation.on_delete
      }
    })
    newRelation.name = ''
    await loadRelations()
    toast.add({ title: 'Relation created', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    relationsError.value = cause?.data?.message || cause?.statusMessage || 'Failed to create relation'
  } finally {
    saving.value = false
  }
}

async function removeRelation(id: string) {
  relationsError.value = ''
  try {
    await $fetch(`/api/meta/relations/${encodeURIComponent(id)}`, { method: 'DELETE' })
    await loadRelations()
    toast.add({ title: 'Relation deleted', color: 'success', icon: 'i-lucide-check' })
  } catch (cause: any) {
    relationsError.value = cause?.data?.message || cause?.statusMessage || 'Failed to delete relation'
  }
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
          <div class="mb-3 grid grid-cols-1 gap-2 sm:grid-cols-[1fr_1fr_1fr_auto]">
            <UInput v-model="entityForm.name" placeholder="vehicle (snake_case)" />
            <UInput v-model="entityForm.label" placeholder="Vehicle" />
            <UInput v-model="entityForm.description" placeholder="Description (optional)" />
            <UButton :loading="saving" :disabled="!entityForm.name.trim()" @click="addEntity">Add entity</UButton>
          </div>
          <div v-for="entity in entities" :key="entity.name" class="mb-4 rounded-lg border border-muted p-3">
            <div class="flex items-center justify-between">
              <h3 class="font-mono text-sm font-semibold">{{ entity.name }} <span class="text-muted">· {{ entity.label || entity.name }}</span></h3>
              <UButton size="xs" variant="ghost" color="error" @click="removeEntity(entity.name)">Remove</UButton>
            </div>
            <p v-if="entity.description" class="mt-1 text-xs text-muted">{{ entity.description }}</p>
            <div class="mt-2 space-y-1">
              <div v-for="field in entity.fields" :key="field.name" class="flex items-center justify-between rounded bg-muted/40 px-2 py-1 text-sm">
                <span class="font-mono">{{ field.name }} <span class="text-muted">· {{ field.type }}{{ field.required ? ' · required' : '' }}{{ field.is_status ? ' · status' : '' }}{{ field.unique ? ' · unique' : '' }}{{ field.ref_entity ? ` → ${field.ref_entity}` : '' }}{{ field.computed_expr ? ` = ${field.computed_expr}` : '' }}{{(field.options || []).length ? ` · ${(field.options || []).length} options` : '' }}</span></span>
                <UButton size="xs" variant="ghost" color="error" @click="removeField(entity.name, field.name)">Remove</UButton>
              </div>
              <p v-if="!entity.fields.length" class="text-xs text-muted">No fields yet.</p>
            </div>
            <details class="mt-2 rounded border border-muted p-2">
              <summary class="cursor-pointer text-xs font-semibold text-muted">Add field</summary>
              <div class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-3">
                <UInput v-model="fieldFormFor(entity.name).name" placeholder="plate_number" />
                <UInput v-model="fieldFormFor(entity.name).label" placeholder="Label (optional)" />
                <USelectMenu v-model="fieldFormFor(entity.name).type" :items="typeItems" value-key="value" placeholder="Type" />
              </div>
              <div class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-3">
                <UInput v-model="fieldFormFor(entity.name).ref_entity" placeholder="ref entity (reference only)" />
                <UInput v-model="fieldFormFor(entity.name).computed_expr" placeholder="{code} (computed/formula)" />
                <UInput v-model="fieldFormFor(entity.name).help_text" placeholder="Help text (optional)" />
              </div>
              <div class="mt-2 grid grid-cols-2 gap-2 sm:grid-cols-4">
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="fieldFormFor(entity.name).required" /> Required</label>
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="fieldFormFor(entity.name).is_status" /> Status</label>
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="fieldFormFor(entity.name).unique" /> Unique</label>
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="fieldFormFor(entity.name).readonly" /> Readonly</label>
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="fieldFormFor(entity.name).hidden" /> Hidden</label>
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="fieldFormFor(entity.name).searchable" /> Searchable</label>
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="fieldFormFor(entity.name).sortable" /> Sortable</label>
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="fieldFormFor(entity.name).filterable" /> Filterable</label>
                <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="fieldFormFor(entity.name).indexed" /> Indexed</label>
              </div>
              <div v-if="isNumericType(fieldFormFor(entity.name).type)" class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-3">
                <UInput v-model.number="fieldFormFor(entity.name).min_value" type="number" placeholder="min value" />
                <UInput v-model.number="fieldFormFor(entity.name).max_value" type="number" placeholder="max value" />
                <UInput v-model.number="fieldFormFor(entity.name).precision" type="number" placeholder="precision 0–38" />
              </div>
              <div v-if="isTextType(fieldFormFor(entity.name).type)" class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-3">
                <UInput v-model="fieldFormFor(entity.name).pattern" placeholder="regex pattern" />
                <UInput v-model.number="fieldFormFor(entity.name).min_length" type="number" placeholder="min length" />
                <UInput v-model.number="fieldFormFor(entity.name).max_length" type="number" placeholder="max length" />
              </div>
              <div v-if="fieldFormFor(entity.name).type === 'auto_number'" class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-2">
                <UInput v-model="fieldFormFor(entity.name).auto_number_prefix" placeholder="prefix (e.g. VEH-)" />
                <UInput v-model.number="fieldFormFor(entity.name).auto_number_width" type="number" placeholder="width 1–10" />
              </div>
              <div class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-2">
                <UInput v-model="fieldFormFor(entity.name).default_value" placeholder="default value (optional)" />
              </div>
              <div class="mt-2 flex justify-end">
                <UButton size="sm" :disabled="!fieldFormFor(entity.name).name.trim()" @click="addField(entity.name)">Add field</UButton>
              </div>
            </details>
          </div>
        </UCard>
        <UCard>
          <template #header><h2 class="text-sm font-semibold">Select options</h2></template>
          <p class="mb-2 text-xs text-muted">Options live in the definition and materialize on publish. Select fields need at least one option.</p>
          <div class="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_1fr_1fr_1fr_auto]">
            <USelectMenu v-model="newOption.entity" :items="entities.map(e => ({ label: e.label || e.name, value: e.name }))" value-key="value" placeholder="Entity…" />
            <USelectMenu v-model="newOption.field" :items="(entities.find(e => e.name === newOption.entity)?.fields || []).filter(f => f.type === 'select' || f.type === 'multi_select').map(f => ({ label: f.label || f.name, value: f.name }))" value-key="value" placeholder="Field…" />
            <UInput v-model="newOption.value" placeholder="value" />
            <UInput v-model="newOption.label" placeholder="Label" />
            <UButton :disabled="!newOption.entity || !newOption.field || !newOption.value.trim() || !newOption.label.trim()" @click="addOption">Add option</UButton>
          </div>
          <div v-for="entity in entities" :key="`opt-${entity.name}`" class="mt-2">
            <div v-for="field in (entity.fields || []).filter(f => (f.options || []).length)" :key="`${entity.name}.${field.name}`" class="mt-1 text-sm">
              <p class="font-mono">{{ entity.name }}.{{ field.name }}</p>
              <div class="mt-1 flex flex-wrap gap-1">
                <UBadge v-for="opt in field.options" :key="opt.value" variant="subtle">{{ opt.value }} · {{ opt.label }} <button type="button" class="ml-1 underline" @click="removeOption(entity.name, field.name, opt.value)">×</button></UBadge>
              </div>
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
            <p v-if="(entity.workflow?.transitions || []).length" class="font-mono text-muted">{{ (entity.workflow?.transitions || []).map(t => `${t.from_state} --${t.action}--> ${t.to_state}`).join(' · ') }}</p>
          </div>
        </UCard>
        <UCard>
          <template #header><h2 class="text-sm font-semibold">Views</h2></template>
          <div class="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_1fr_auto]">
            <USelectMenu v-model="newView.entity" :items="entities.map(e => ({ label: e.label || e.name, value: e.name }))" value-key="value" placeholder="Entity…" />
            <UInput v-model="newView.name" placeholder="All vehicles" />
            <UButton :disabled="!newView.entity || !newView.name.trim()" @click="addView">Add view</UButton>
          </div>
          <div v-for="entity in entities" :key="`view-${entity.name}`" class="mt-2 text-sm">
            <div v-for="view in entity.views || []" :key="view.name" class="flex items-center justify-between rounded bg-muted/40 px-2 py-1">
              <span class="font-mono">{{ entity.name }} · {{ view.name }}</span>
              <UButton size="xs" variant="ghost" color="error" @click="removeView(entity.name, view.name)">Remove</UButton>
            </div>
          </div>
        </UCard>
        <UCard>
          <template #header><h2 class="text-sm font-semibold">Form layout</h2></template>
          <p class="mb-2 text-xs text-muted">Sections reference field names. Unassigned fields render under “Other”.</p>
          <div v-for="entity in entities" :key="`layout-${entity.name}`" class="mb-3 rounded-lg border border-muted p-3">
            <h3 class="font-mono text-sm font-semibold">{{ entity.name }}</h3>
            <div v-for="section in entity.form_layout?.config?.sections || []" :key="section.id" class="mt-2 rounded bg-muted/40 p-2 text-sm">
              <div class="flex items-center justify-between">
                <span class="font-semibold">{{ section.label }} <span class="font-mono text-muted">· {{ section.id }}</span></span>
                <UButton size="xs" variant="ghost" color="error" @click="removeLayoutSection(entity.name, section.id)">Remove</UButton>
              </div>
              <div class="mt-1 flex flex-wrap gap-1">
                <UBadge v-for="fname in section.fields" :key="fname" variant="subtle">{{ fieldDisplayName(entity.name, fname) }} <button type="button" class="ml-1 underline" @click="removeFieldFromLayout(entity.name, section.id, fname)">×</button></UBadge>
                <span v-if="!section.fields.length" class="text-xs text-muted">No fields assigned.</span>
              </div>
              <div class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-[1fr_auto]">
                <USelectMenu v-model="layoutFieldPick[`${entity.name}:${section.id}`]" :items="unassignedLayoutFields(entity).map(f => ({ label: f.label || f.name, value: f.name }))" value-key="value" placeholder="Add field…" />
                <UButton size="xs" @click="addFieldToLayout(entity.name, section.id)">Add</UButton>
              </div>
            </div>
            <div class="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-[1fr_auto]">
              <UInput v-model="layoutFormFor(entity.name).label" placeholder="New section label" />
              <UButton size="sm" :disabled="!layoutFormFor(entity.name).label.trim()" @click="addLayoutSection(entity.name)">Add section</UButton>
            </div>
          </div>
        </UCard>
        <UCard>
          <template #header><h2 class="text-sm font-semibold">Reports</h2></template>
          <div class="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_1fr_1fr_1fr_auto]">
            <USelectMenu v-model="newReport.entity" :items="entities.map(e => ({ label: e.label || e.name, value: e.name }))" value-key="value" placeholder="Entity…" />
            <UInput v-model="newReport.name" placeholder="Utilization" />
            <UInput v-model="newReport.group_by" placeholder="group_by field" />
            <USelectMenu v-model="newReport.chart_type" :items="chartTypeItems" value-key="value" placeholder="Chart (optional)" />
            <UButton :disabled="!newReport.entity || !newReport.name.trim() || !newReport.group_by.trim()" @click="addReport">Add report</UButton>
          </div>
          <div v-for="entity in entities" :key="`report-${entity.name}`" class="mt-2 text-sm">
            <div v-for="report in entity.reports || []" :key="report.name" class="flex items-center justify-between rounded bg-muted/40 px-2 py-1">
              <span class="font-mono">{{ entity.name }} · {{ report.name }} · group_by={{ report.config?.group_by }}</span>
              <UButton size="xs" variant="ghost" color="error" @click="removeReport(entity.name, report.name)">Remove</UButton>
            </div>
          </div>
        </UCard>
        <UCard>
          <template #header><h2 class="text-sm font-semibold">Notifications</h2></template>
          <div class="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_2fr_1fr_auto_auto]">
            <USelectMenu v-model="newNotification.entity" :items="entities.map(e => ({ label: e.label || e.name, value: e.name }))" value-key="value" placeholder="Entity…" />
            <UInput v-model="newNotification.target_url" placeholder="https://example.com/hook" />
            <USelectMenu v-model="newNotification.trigger" :items="triggerItems" value-key="value" />
            <label class="flex items-center gap-1 text-xs text-muted"><UCheckbox v-model="newNotification.active" /> Active</label>
            <UButton :disabled="!newNotification.entity || !newNotification.target_url.trim()" @click="addNotification">Add rule</UButton>
          </div>
          <div v-for="entity in entities" :key="`notif-${entity.name}`" class="mt-2 text-sm">
            <div v-for="(rule, index) in entity.notifications || []" :key="`${entity.name}-${index}`" class="flex items-center justify-between rounded bg-muted/40 px-2 py-1">
              <span class="font-mono">{{ entity.name }} · {{ rule.trigger || 'transition' }} → {{ rule.target_url }}</span>
              <UButton size="xs" variant="ghost" color="error" @click="removeNotification(entity.name, index)">Remove</UButton>
            </div>
          </div>
        </UCard>
        <UCard>
          <template #header>
            <div class="flex items-center justify-between">
              <h2 class="text-sm font-semibold">Relations</h2>
              <UButton size="xs" variant="ghost" icon="i-lucide-refresh-cw" :loading="relationsLoading" @click="loadRelations()">Refresh</UButton>
            </div>
          </template>
          <p v-if="isDraft" class="mb-2 text-xs text-muted">Publish first — relations attach to materialized entities ({{ module?.id }}_&lt;entity&gt;).</p>
          <UAlert v-if="relationsError" color="error" :title="relationsError" />
          <div v-else class="grid grid-cols-1 gap-2 sm:grid-cols-[1fr_1fr_1fr_1fr_1fr_auto]">
            <USelectMenu v-model="newRelation.source" :items="entities.map(e => ({ label: e.label || e.name, value: e.name }))" value-key="value" placeholder="Source…" />
            <USelectMenu v-model="newRelation.target" :items="entities.map(e => ({ label: e.label || e.name, value: e.name }))" value-key="value" placeholder="Target…" />
            <UInput v-model="newRelation.name" placeholder="rental_vehicle" />
            <USelectMenu v-model="newRelation.relation_type" :items="relationTypeItems" value-key="value" />
            <USelectMenu v-model="newRelation.on_delete" :items="deleteRuleItems" value-key="value" />
            <UButton :disabled="isDraft || !newRelation.source || !newRelation.target || !newRelation.name.trim()" @click="addRelation">Add relation</UButton>
          </div>
          <div v-for="entity in entities" :key="`rel-${entity.name}`" class="mt-2 text-sm">
            <div v-for="rel in relationsByEntity[entity.name] || []" :key="rel.id" class="flex items-center justify-between rounded bg-muted/40 px-2 py-1">
              <span class="font-mono">{{ rel.name }} · {{ rel.relation_type }} · {{ rel.on_delete }}</span>
              <UButton size="xs" variant="ghost" color="error" @click="removeRelation(rel.id)">Remove</UButton>
            </div>
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
